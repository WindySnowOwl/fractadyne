//! A Life-like rule as one 512-bit table (design/automata.md §4.1), and the notations that compile
//! to it: B/S (and the older S/B), von Neumann `V`, Hensel's non-totalistic letters, `MAP` strings,
//! and Generations' state counts. The steppers only ever see the table, so a notation costs the
//! parser and nothing else.

use super::hensel::{self, Mask, M_EDGES};
use std::fmt;

/// Bits of a table index: the 3×3 neighbourhood, NW most significant and SE least, row by row —
/// the order `MAP` strings use. (Life's `MAP` string decodes to B3/S23 under it, which pins the
/// centre's place and the bit order; the order of the eight neighbours among themselves is
/// LifeWiki's documented one, which no isotropic rule can check.)
pub const NW: usize = 1 << 8;
pub const N: usize = 1 << 7;
pub const NE: usize = 1 << 6;
pub const W: usize = 1 << 5;
pub const CENTRE: usize = 1 << 4;
pub const E: usize = 1 << 3;
pub const SW: usize = 1 << 2;
pub const S: usize = 1 << 1;
pub const SE: usize = 1;

/// The most states a Generations rule may have (a cell is a byte).
pub const MAX_STATES: u16 = 256;

/// A rule: for each 3×3 neighbourhood (a table index), whether the centre is alive next generation;
/// and the number of states — 2 for a binary rule, 3 or more for Generations, where state 1 is alive,
/// 2…C−1 are dying, and only state-1 cells count as neighbours.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Rule {
    table: [u64; 8],
    states: u16,
}

/// Why a rule string was refused, worded for the panel that shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleError(pub String);

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RuleError {}

fn err<T>(msg: impl Into<String>) -> Result<T, RuleError> {
    Err(RuleError(msg.into()))
}

/// The neighbour arrangement in a table index (the index without its centre bit).
#[inline]
pub(crate) fn mask_of(idx: usize) -> Mask {
    (((idx >> 5) << 4) | (idx & 0xF)) as Mask
}

/// The table index of an arrangement and a centre.
#[inline]
pub(crate) fn index_of(mask: Mask, centre: bool) -> usize {
    let m = mask as usize;
    ((m >> 4) << 5) | (m & 0xF) | if centre { CENTRE } else { 0 }
}

/// A set of neighbour arrangements: one bit per [`Mask`].
type MaskSet = [bool; 256];

impl Rule {
    /// Conway's Life, B3/S23.
    pub fn life() -> Rule {
        Rule::parse("B3/S23").expect("Life parses")
    }

    /// A rule from its table and state count. Refuses a state count outside 2…[`MAX_STATES`], and a
    /// Generations rule with B0 (birth with no live neighbours), whose background would need
    /// states the engine does not track.
    pub fn from_table(table: [u64; 8], states: u16) -> Result<Rule, RuleError> {
        if !(2..=MAX_STATES).contains(&states) {
            return err(format!("a rule has 2 to {MAX_STATES} states, not {states}"));
        }
        let rule = Rule { table, states };
        if states > 2 && rule.alive(0) {
            return err("B0 (birth with no live neighbours) is not supported in Generations rules");
        }
        Ok(rule)
    }

    /// The table: bit `i` of the 512 is whether neighbourhood `i` is alive next generation.
    pub fn table(&self) -> [u64; 8] {
        self.table
    }

    /// The table as 16 little-endian `u32` words, for a GPU uniform.
    pub fn table_words(&self) -> [u32; 16] {
        let mut w = [0u32; 16];
        for (i, &t) in self.table.iter().enumerate() {
            w[2 * i] = t as u32;
            w[2 * i + 1] = (t >> 32) as u32;
        }
        w
    }

    /// The number of states: 2 for a binary rule.
    pub fn states(&self) -> u16 {
        self.states
    }

    /// Whether neighbourhood `idx` (a table index) is alive next generation.
    #[inline]
    pub fn alive(&self, idx: usize) -> bool {
        (self.table[idx >> 6] >> (idx & 63)) & 1 != 0
    }

    /// The next state of a cell in `state` whose neighbourhood — state-1 cells as alive, the
    /// centre included — is `idx`.
    #[inline]
    pub fn next(&self, state: u8, idx: usize) -> u8 {
        if self.states == 2 || state == 0 {
            return self.alive(idx) as u8;
        }
        if state == 1 {
            return if self.alive(idx) { 1 } else { 2 };
        }
        if u16::from(state) + 1 >= self.states {
            0
        } else {
            state + 1
        }
    }

    /// The state an unbounded plane's background takes next, all of it in `bg`: it changes only
    /// under B0 — Life's stays dead, B0/S8 turns it alive for good, B0 without S8 makes it blink.
    pub fn next_background(&self, bg: u8) -> u8 {
        let idx = if bg == 1 { 511 } else { 0 };
        self.next(bg, idx)
    }

    /// Whether the rule has B0: a dead cell with no live neighbours is born.
    pub fn has_b0(&self) -> bool {
        self.alive(0)
    }

    fn uniform_over(&self, same: impl Fn(Mask, Mask) -> bool) -> bool {
        (0..2).all(|c| {
            (0..=255u8).all(|a| {
                (0..=255u8).filter(|&b| same(a, b)).all(|b| self.alive(index_of(a, c == 1)) == self.alive(index_of(b, c == 1)))
            })
        })
    }

    /// Outer totalistic: the outcome depends on the centre and the number of live neighbours only.
    pub fn is_totalistic(&self) -> bool {
        self.uniform_over(|a, b| a.count_ones() == b.count_ones())
    }

    /// Von Neumann: the corners are ignored, and the outcome depends on the number of live
    /// orthogonal neighbours only.
    pub fn is_von_neumann(&self) -> bool {
        self.uniform_over(|a, b| (a & M_EDGES).count_ones() == (b & M_EDGES).count_ones())
    }

    /// Isotropic: the outcome is the same for arrangements a rotation or reflection maps onto each
    /// other — what Hensel's letters can name.
    pub fn is_isotropic(&self) -> bool {
        self.uniform_over(|a, b| hensel::canonical(a) == hensel::canonical(b))
    }

    /// The rule's canonical string: `B3/S23` when outer totalistic, `B…/S…V` when von Neumann, Hensel
    /// letters when isotropic (each count's letters in chart order, or `-` and the letters it lacks
    /// when that is shorter), else `MAP` and the table; Generations append `/C<states>`.
    /// [`Rule::parse`] reads every one of them back to the same rule.
    pub fn canonical(&self) -> String {
        let mut s = if self.is_totalistic() {
            let counts = |c: bool| -> String {
                (0..=8u8).filter(|&n| self.alive(index_of(full(n), c))).map(|n| char::from(b'0' + n)).collect()
            };
            format!("B{}/S{}", counts(false), counts(true))
        } else if self.is_von_neumann() {
            let counts = |c: bool| -> String {
                (0..=4u8)
                    .filter(|&n| self.alive(index_of(edges(n), c)))
                    .map(|n| char::from(b'0' + n))
                    .collect()
            };
            format!("B{}/S{}V", counts(false), counts(true))
        } else if self.is_isotropic() {
            format!("B{}/S{}", self.hensel(false), self.hensel(true))
        } else {
            format!("MAP{}", encode_map(&self.table))
        };
        if self.states > 2 {
            s.push_str(&format!("/C{}", self.states));
        }
        s
    }

    /// One half of an isotropic rule in Hensel's letters.
    fn hensel(&self, centre: bool) -> String {
        let mut out = String::new();
        for n in 0..=8u8 {
            let on: Vec<char> = hensel::CHART
                .iter()
                .filter(|&&(k, _, m)| k == n && self.alive(index_of(m, centre)))
                .map(|&(_, l, _)| l)
                .collect();
            let all = hensel::CHART.iter().filter(|&&(k, _, _)| k == n).count();
            if on.is_empty() {
                continue;
            }
            out.push(char::from(b'0' + n));
            if on.len() == all {
                continue;
            }
            let off: Vec<char> = hensel::letters_of(n).filter(|l| !on.contains(l)).collect();
            if on.len() <= off.len() {
                out.extend(on);
            } else {
                out.push('-');
                out.extend(off);
            }
        }
        out
    }

    /// Parse a rule string: `B3/S23`, `b3s23`, `23/3` (S/B), `B2/S013V` (von Neumann),
    /// `B2-a/S12` and `B3/S2-i34q` (Hensel), `MAP` + 86 base64 characters, and Generations as
    /// `B2/S/C3`, `B2/S/G3`, `B2/S/3` or `/2/3` (S/B/C). Case of the B, S, C, G and V markers is
    /// ignored; Hensel letters are lower case.
    pub fn parse(text: &str) -> Result<Rule, RuleError> {
        let t = text.trim();
        if t.is_empty() {
            return err("the rule is empty");
        }
        if t.len() >= 3 && t[..3].eq_ignore_ascii_case("MAP") {
            return parse_map(&t[3..]);
        }
        if t.contains(char::is_whitespace) {
            return err(format!("'{t}': a rule has no spaces"));
        }
        let (body, von_neumann) = match t.chars().last() {
            Some('V' | 'v') => (&t[..t.len() - 1], true),
            Some('H' | 'h') => return err("hexagonal rules (H) are not supported"),
            _ => (t, false),
        };
        let body = body.strip_suffix('/').unwrap_or(body);

        let mut birth: Option<&str> = None;
        let mut survival: Option<&str> = None;
        let mut states: Option<&str> = None;
        let mut positional: Vec<&str> = Vec::new();
        let parts: Vec<&str> = if body.contains('/') {
            body.split('/').collect()
        } else if let Some(i) = body.find(['S', 's']).filter(|_| body.starts_with(['B', 'b'])) {
            // `B3S23` / `b3s23`, no slash.
            vec![&body[..i], &body[i..]]
        } else {
            vec![body]
        };
        for p in &parts {
            match p.chars().next() {
                Some('B' | 'b') => set_once(&mut birth, &p[1..], "birth")?,
                Some('S' | 's') => set_once(&mut survival, &p[1..], "survival")?,
                Some('C' | 'c' | 'G' | 'g') if p.len() > 1 && p[1..].bytes().all(|b| b.is_ascii_digit()) => {
                    set_once(&mut states, &p[1..], "state count")?
                }
                _ => positional.push(p),
            }
        }
        if birth.is_some() || survival.is_some() {
            // B/S form; a bare number after them is the state count (`B2-a/S12/3`).
            match *positional.as_slice() {
                [] => {}
                [n] if !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) && states.is_none() => states = Some(n),
                _ => return err(format!("'{t}': expected B…/S… with an optional /C<states>")),
            }
        } else {
            // S/B or S/B/C.
            match *positional.as_slice() {
                [s, b] => (survival, birth) = (Some(s), Some(b)),
                [s, b, c] if states.is_none() => (survival, birth, states) = (Some(s), Some(b), Some(c)),
                _ => return err(format!("'{t}' is not a rule: expected B…/S…, S/B or MAP…")),
            }
        }
        let b = counts(birth.unwrap_or(""), von_neumann, "B")?;
        let s = counts(survival.unwrap_or(""), von_neumann, "S")?;
        let states = match states {
            None => 2,
            Some(n) => match n.parse::<u16>() {
                Ok(n) if (2..=MAX_STATES).contains(&n) => n,
                _ => return err(format!("'{n}' states: a rule has 2 to {MAX_STATES}")),
            },
        };
        let mut table = [0u64; 8];
        for idx in 0..512 {
            let set = if idx & CENTRE != 0 { &s } else { &b };
            if set[mask_of(idx) as usize] {
                table[idx >> 6] |= 1 << (idx & 63);
            }
        }
        Rule::from_table(table, states)
    }
}

fn set_once<'a>(slot: &mut Option<&'a str>, v: &'a str, what: &str) -> Result<(), RuleError> {
    if slot.replace(v).is_some() {
        return err(format!("the {what} is given twice"));
    }
    Ok(())
}

/// The arrangement of `n` neighbours that a totalistic rule's string stands for (any will do).
fn full(n: u8) -> Mask {
    if n == 0 {
        0
    } else {
        (0xFFu16 << (8 - n)) as Mask
    }
}

/// `n` live orthogonal neighbours and no corners.
fn edges(n: u8) -> Mask {
    [0, hensel::M_N, hensel::M_N | hensel::M_E, hensel::M_N | hensel::M_E | hensel::M_S, M_EDGES][n as usize]
}

/// The arrangements one half of a rule string (`23`, `2-i34q`, `013` with `V`) selects.
fn counts(spec: &str, von_neumann: bool, half: &str) -> Result<MaskSet, RuleError> {
    let mut set = [false; 256];
    let chars: Vec<char> = spec.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let Some(n) = chars[i].to_digit(10).filter(|&d| d <= 8) else {
            return err(format!("'{}' in {half}{spec}: expected a neighbour count 0-8", chars[i]));
        };
        let n = n as u8;
        i += 1;
        let negate = chars.get(i) == Some(&'-');
        if negate {
            i += 1;
        }
        let mut letters = Vec::new();
        while let Some(&c) = chars.get(i).filter(|c| c.is_ascii_alphabetic()) {
            letters.push(c.to_ascii_lowercase());
            i += 1;
        }
        if von_neumann {
            if n > 4 {
                return err(format!("{half}{n}: a von Neumann cell has 4 neighbours"));
            }
            if negate || !letters.is_empty() {
                return err(format!("{half}{spec}: von Neumann rules take no Hensel letters"));
            }
            // The corners are ignored: every arrangement with `n` orthogonal neighbours.
            for m in 0..=255u8 {
                if (m & M_EDGES).count_ones() == u32::from(n) {
                    set[m as usize] = true;
                }
            }
            continue;
        }
        if negate && letters.is_empty() {
            return err(format!("{half}{n}-: '-' must be followed by letters"));
        }
        for &l in &letters {
            if !hensel::is_letter_of(n, l) {
                let valid: String = hensel::letters_of(n).collect();
                return err(if valid.is_empty() {
                    format!("{half}{n}{l}: {n} neighbours have no letters")
                } else {
                    format!("{half}{n}{l}: '{l}' is not a letter for {n} neighbours (they are {valid})")
                });
            }
        }
        for m in 0..=255u8 {
            if m.count_ones() != u32::from(n) {
                continue;
            }
            let (_, l) = hensel::class_of(m);
            if letters.is_empty() || letters.contains(&l) != negate {
                set[m as usize] = true;
            }
        }
    }
    Ok(set)
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// The table as `MAP` writes it: 512 bits, index 0 first, most significant bit first, in base64
/// without padding (86 characters).
fn encode_map(table: &[u64; 8]) -> String {
    let bit = |i: usize| -> u32 { ((table[i >> 6] >> (i & 63)) & 1) as u32 };
    (0..86)
        .map(|k| {
            let v = (0..6).fold(0u32, |acc, j| {
                let i = 6 * k + j;
                (acc << 1) | if i < 512 { bit(i) } else { 0 }
            });
            char::from(B64[v as usize])
        })
        .collect()
}

fn parse_map(rest: &str) -> Result<Rule, RuleError> {
    if !rest.is_ascii() {
        return err("a MAP rule is base64: ASCII letters, digits, '+' and '/'");
    }
    let (data, suffix) = rest.split_at(rest.len().min(86));
    if data.len() < 86 {
        return err(format!("a MAP rule has 86 base64 characters, this one {}", data.len()));
    }
    let suffix = suffix.strip_prefix("==").unwrap_or(suffix);
    let states = match suffix {
        "" => 2,
        s => match s.strip_prefix('/').map(|n| n.trim_start_matches(['C', 'c', 'G', 'g'])).map(str::parse::<u16>) {
            Some(Ok(n)) => n,
            _ => return err(format!("'{s}' after a MAP rule: expected /C<states>")),
        },
    };
    let mut table = [0u64; 8];
    for (k, c) in data.bytes().enumerate() {
        let Some(v) = B64.iter().position(|&b| b == c) else {
            return err(format!("'{}' is not a base64 character", char::from(c)));
        };
        for j in 0..6 {
            let i = 6 * k + j;
            if i < 512 && (v >> (5 - j)) & 1 != 0 {
                table[i >> 6] |= 1 << (i & 63);
            }
        }
    }
    Rule::from_table(table, states)
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canonical())
    }
}

impl fmt::Debug for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rule({})", self.canonical())
    }
}

impl std::str::FromStr for Rule {
    type Err = RuleError;
    fn from_str(s: &str) -> Result<Rule, RuleError> {
        Rule::parse(s)
    }
}

#[cfg(test)]
mod tests;
