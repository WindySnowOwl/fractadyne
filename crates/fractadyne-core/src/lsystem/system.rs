//! A system and its text (design/lsystems.md §4.1–4.2): the alphabet's roles, the axiom, the
//! productions, the angle — read from the native line format and written back to it.
//!
//! A system is untrusted input (a file, a pasted entry): every word and number is checked, and
//! refused with its line and column rather than drawn wrong.

use std::fmt;

/// The most tokens the axiom or one production may hold.
pub const MAX_WORD: usize = 4096;

/// The angle `+` and `-` turn by.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Angle {
    /// A division of the circle: `360° / n` (Fractint's `Angle n`).
    Division(u32),
    /// Any angle, in degrees.
    Degrees(f64),
}

impl Angle {
    /// The most divisions of the circle an angle may name.
    pub const MAX_DIVISION: u32 = 36_000;

    pub fn degrees(self) -> f64 {
        match self {
            Angle::Division(n) => 360.0 / n as f64,
            Angle::Degrees(d) => d,
        }
    }

    /// The division of the circle this angle is, if it is one — `Degrees(60)` counts (6), so a
    /// system written either way turns by exact table steps.
    pub fn division(self) -> Option<u32> {
        match self {
            Angle::Division(n) => Some(n),
            Angle::Degrees(d) if d > 0.0 => {
                let n = 360.0 / d;
                let r = n.round();
                ((n - r).abs() < 1e-9 && r >= 1.0 && r <= Self::MAX_DIVISION as f64).then_some(r as u32)
            }
            Angle::Degrees(_) => None,
        }
    }
}

/// One token of a word.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tok {
    /// A symbol: one that draws or moves (`F`, `G`, …) or a variable that only rewrites.
    Sym(u8),
    /// `+` (1) / `-` (−1): turn left / right by the angle.
    Turn(i8),
    /// `|`: turn around (with an odd division, the largest turn under 180°).
    Around,
    /// `!`: swap the meanings of left and right.
    Reverse,
    /// `[`: save the turtle's whole state.
    Push,
    /// `]`: restore the last saved state.
    Pop,
    /// `@x` (`@Ix` = 1/x, `@Qx` = √x): multiply the step by a factor.
    Scale(f64),
    /// `\a` (+a) / `/a` (−a): turn left by `a` degrees.
    TurnBy(f64),
    /// `Cn`: set the colour index.
    SetColour(i32),
    /// `<n` (+n) / `>n` (−n): raise or lower the colour index.
    AddColour(i32),
    /// `{`: start a filled polygon at the turtle (ABOP's leaves). Every step taken inside it —
    /// drawing or not — adds its end as a vertex, and draws no line.
    PolyStart,
    /// `}`: fill the polygon.
    PolyEnd,
    /// `.`: add the turtle's position to the polygon as a vertex.
    Vertex,
}

/// What a symbol does when the turtle reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// Nothing (a variable).
    None,
    /// Step forward, drawing.
    Draw,
    /// Step forward without drawing.
    Move,
}

/// How the segments are coloured (design/lsystems.md §5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Colouring {
    /// Position along the curve: the segment's index.
    Position,
    /// Bracket depth: trunk to twigs.
    Depth,
    /// The direction the segment is drawn in.
    Heading,
    /// The colour index (`C`, `<`, `>`).
    Index,
    /// One colour.
    Plain,
}

impl Colouring {
    pub const ALL: [Colouring; 5] =
        [Colouring::Position, Colouring::Depth, Colouring::Heading, Colouring::Index, Colouring::Plain];

    pub fn key(self) -> &'static str {
        match self {
            Colouring::Position => "position",
            Colouring::Depth => "depth",
            Colouring::Heading => "heading",
            Colouring::Index => "index",
            Colouring::Plain => "plain",
        }
    }

    pub fn from_key(s: &str) -> Option<Colouring> {
        Self::ALL.into_iter().find(|c| c.key().eq_ignore_ascii_case(s))
    }
}

/// The default role of a symbol: `F` and `D` draw; `f`, `G` and `M` move (ABOP's `f`, Fractint's
/// `G` and `M`); everything else is a variable.
pub fn default_role(c: u8) -> Role {
    match c {
        b'F' | b'D' => Role::Draw,
        b'f' | b'G' | b'M' => Role::Move,
        _ => Role::None,
    }
}

/// One of a symbol's productions: the word it rewrites to, and its weight among the symbol's
/// alternatives (1 for a symbol with one).
#[derive(Clone, Debug, PartialEq)]
pub struct Production {
    pub weight: f64,
    pub word: Vec<Tok>,
}

/// The seed a system's random choices follow when it names none.
pub const DEFAULT_SEED: u64 = 1;

/// The most alternatives a symbol may have: one for each variant a node can take
/// ([`super::variant::VARIANTS`]) — more could never all be drawn.
pub const MAX_ALTERNATIVES: usize = super::variant::VARIANTS as usize;

/// A bracketed 0L system with turtle commands: deterministic (D0L), or stochastic where a symbol
/// has weighted alternatives.
#[derive(Clone, Debug, PartialEq)]
pub struct LSystem {
    pub name: String,
    pub angle: Angle,
    /// The turtle's first heading, in degrees anticlockwise from +x (a plant grows up: 90).
    pub heading: f64,
    pub axiom: Vec<Tok>,
    /// What each symbol rewrites to (empty: itself) — one production, or several weighted
    /// alternatives. 256 entries.
    pub rules: Vec<Vec<Production>>,
    /// What each symbol does when read. 256 entries.
    pub roles: Vec<Role>,
    /// The colouring the system asks for (`None`: the default, [`LSystem::colouring`]).
    pub colour: Option<Colouring>,
    /// A fixed order to draw at (`None`: the order follows the zoom).
    pub order: Option<u32>,
    /// What a stochastic system's choices follow ([`super::variant`]): the same seed, the same
    /// picture.
    pub seed: u64,
    /// A parametric or context-sensitive system's grammar ([`super::expand`]): it is drawn by
    /// building its word, and `axiom` and `rules` are empty.
    pub expanded: Option<std::sync::Arc<super::expand::Expanded>>,
}

/// Why a system was refused: what and where (1-based line and column; column 0 = the whole line).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.col > 0 {
            write!(f, "line {}, column {}: {}", self.line, self.col, self.message)
        } else {
            write!(f, "line {}: {}", self.line, self.message)
        }
    }
}

impl std::error::Error for ParseError {}

pub(crate) fn fail<T>(line: usize, col: usize, message: impl Into<String>) -> Result<T, ParseError> {
    Err(ParseError { line, col, message: message.into() })
}

/// Whether `c` is a command character (one that can never be a symbol).
fn is_command(c: u8) -> bool {
    matches!(c, b'+' | b'-' | b'|' | b'!' | b'[' | b']' | b'@' | b'\\' | b'/' | b'<' | b'>' | b'{' | b'}' | b'.')
}

/// Whether `c` may be a symbol (and so have a production).
pub fn is_symbol(c: u8) -> bool {
    c.is_ascii_graphic() && !is_command(c) && !matches!(c, b'=' | b'#' | b';')
}

/// Reads a number at `s[i..]`: digits with at most one point. Returns it and where it ended.
fn number(s: &[u8], mut i: usize) -> Option<(f64, usize)> {
    let start = i;
    let mut point = false;
    while i < s.len() && (s[i].is_ascii_digit() || (s[i] == b'.' && !point)) {
        point |= s[i] == b'.';
        i += 1;
    }
    let text = std::str::from_utf8(&s[start..i]).ok()?;
    if !text.bytes().any(|b| b.is_ascii_digit()) {
        return None;
    }
    let v: f64 = text.parse().ok()?;
    v.is_finite().then_some((v, i))
}

/// Parses a word: `col0` is the 1-based column of `s`'s first byte, for errors. With `fold`, letters
/// are upper-cased first (Fractint ignores case).
pub(crate) fn parse_word(s: &str, line: usize, col0: usize, fold: bool) -> Result<Vec<Tok>, ParseError> {
    parse_word_spans(s, line, col0, fold).map(|(w, _)| w)
}

/// A parametric word's module: its command or symbol, and the text of each argument with the
/// column it starts at (`F(x*2, 3)`: `x*2` and `3`).
pub(crate) type ArgModule = (Tok, Vec<(String, usize)>);

/// Parses a parametric word (ABOP §1.10): [`parse_word`]'s modules, each optionally followed by its
/// arguments in parentheses — `A(x, y)`, `F(l*0.5)`, `+(30)`. The arguments' text is returned
/// unparsed (they are expressions in the production's parameters).
pub(crate) fn parse_word_args(s: &str, line: usize, col0: usize) -> Result<Vec<ArgModule>, ParseError> {
    // Each `( … )` group, blanked out of the word (so the columns stay put), and where it starts.
    let b = s.as_bytes();
    let mut blank = b.to_vec();
    let mut groups: Vec<(usize, Vec<(String, usize)>)> = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b')' {
            return fail(line, col0 + i, "')' without a '(' before it");
        }
        if b[i] != b'(' {
            i += 1;
            continue;
        }
        let open = i;
        let (mut depth, mut j, mut start) = (0usize, i, i + 1);
        let mut args = Vec::new();
        loop {
            let Some(&c) = b.get(j) else { return fail(line, col0 + open, "a '(' is never closed") };
            match c {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        args.push((s[start..j].to_string(), col0 + start));
                        break;
                    }
                }
                b',' if depth == 1 => {
                    args.push((s[start..j].to_string(), col0 + start));
                    start = j + 1;
                }
                _ => {}
            }
            j += 1;
        }
        if args.len() == 1 && args[0].0.trim().is_empty() {
            args.clear();
        }
        blank[open..=j].fill(b' ');
        // `\(a)`, `/(a)` and `@(f)` read as `\1`, `/1` and `@1`, their argument the factor.
        if open > 0 && matches!(b[open - 1], b'\\' | b'/' | b'@') {
            blank[open] = b'1';
        }
        groups.push((open, args));
        i = j + 1;
    }
    let text = std::str::from_utf8(&blank).expect("ASCII blanks in valid UTF-8");
    let (toks, starts) = parse_word_spans(text, line, col0, false)?;
    let mut out: Vec<ArgModule> = toks.into_iter().map(|t| (t, Vec::new())).collect();
    for (open, args) in groups {
        // The module just before the group (ABOP sets them apart: `F (x)`).
        let Some(k) = starts.iter().rposition(|&st| st < open) else {
            return fail(line, col0 + open, "'(' must follow a symbol (its arguments)");
        };
        if !out[k].1.is_empty() || matches!(out[k].0, Tok::Push | Tok::Pop) {
            return fail(line, col0 + open, "'(' must follow a symbol (its arguments)");
        }
        out[k].1 = args;
    }
    Ok(out)
}

/// [`parse_word`], with the byte offset each token starts at.
fn parse_word_spans(s: &str, line: usize, col0: usize, fold: bool) -> Result<(Vec<Tok>, Vec<usize>), ParseError> {
    let owned;
    let b = if fold {
        owned = s.to_ascii_uppercase();
        owned.as_bytes()
    } else {
        s.as_bytes()
    };
    if !s.is_ascii() {
        let at = s.char_indices().find(|(_, c)| !c.is_ascii()).map_or(0, |(i, _)| i);
        return fail(line, col0 + at, "only ASCII symbols are allowed");
    }
    let mut out = Vec::new();
    let mut starts = Vec::new();
    // The open brackets and braces, innermost last: each closes in the word that opens it.
    let mut open: Vec<u8> = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        let col = col0 + i;
        let at = i;
        i += 1;
        let tok = match c {
            b' ' | b'\t' => continue,
            b'+' => Tok::Turn(1),
            b'-' => Tok::Turn(-1),
            b'|' => Tok::Around,
            b'!' => Tok::Reverse,
            b'[' => {
                open.push(b'[');
                Tok::Push
            }
            b']' => {
                match open.pop() {
                    Some(b'[') => {}
                    Some(_) => return fail(line, col, "']' closes a '{' (close the polygon first)"),
                    None => return fail(line, col, "']' without a '[' before it"),
                }
                Tok::Pop
            }
            b'{' => {
                if open.contains(&b'{') {
                    return fail(line, col, "a '{' inside a polygon (polygons do not nest)");
                }
                open.push(b'{');
                Tok::PolyStart
            }
            b'}' => {
                match open.pop() {
                    Some(b'{') => {}
                    Some(_) => return fail(line, col, "'}' closes a '[' (close the branch first)"),
                    None => return fail(line, col, "'}' without a '{' before it"),
                }
                Tok::PolyEnd
            }
            b'.' => Tok::Vertex,
            b'@' => {
                let (mut inverse, mut root) = (false, false);
                while i < b.len() && matches!(b[i], b'I' | b'i' | b'Q' | b'q') {
                    if matches!(b[i], b'I' | b'i') {
                        inverse = true;
                    } else {
                        root = true;
                    }
                    i += 1;
                }
                let Some((v, j)) = number(b, i) else {
                    return fail(line, col, "'@' needs a number (a step factor)");
                };
                i = j;
                let mut f = if root { v.sqrt() } else { v };
                if inverse {
                    f = 1.0 / f;
                }
                if !(1e-6..=1e6).contains(&f) {
                    return fail(line, col, "a step factor must be between 0.000001 and 1000000");
                }
                Tok::Scale(f)
            }
            b'\\' | b'/' => {
                let Some((v, j)) = number(b, i) else {
                    return fail(line, col, format!("'{}' needs a number (degrees)", c as char));
                };
                i = j;
                if v > 1e6 {
                    return fail(line, col, "a turn of more than 1000000 degrees");
                }
                Tok::TurnBy(if c == b'\\' { v } else { -v })
            }
            b'<' | b'>' => {
                let n = match number(b, i) {
                    Some((v, j)) if v.fract() == 0.0 && v <= 1e6 => {
                        i = j;
                        v as i32
                    }
                    Some(_) => return fail(line, col, "a colour step must be a whole number up to 1000000"),
                    None => 1,
                };
                Tok::AddColour(if c == b'<' { n } else { -n })
            }
            b'C' if i < b.len() && b[i].is_ascii_digit() => {
                let Some((v, j)) = number(b, i) else { unreachable!("a digit follows") };
                if v.fract() != 0.0 || v > 1e6 {
                    return fail(line, col, "a colour must be a whole number up to 1000000");
                }
                i = j;
                Tok::SetColour(v as i32)
            }
            c if is_symbol(c) => Tok::Sym(c),
            c => return fail(line, col, format!("'{}' cannot be used in a word", c as char)),
        };
        out.push(tok);
        starts.push(at);
        if out.len() > MAX_WORD {
            return fail(line, col, format!("a word may hold at most {MAX_WORD} symbols"));
        }
    }
    match open.last() {
        Some(b'{') => fail(line, col0 + b.len(), "a '{' is never closed"),
        Some(_) => fail(line, col0 + b.len(), "a '[' is never closed"),
        None => Ok((out, starts)),
    }
}

/// Writes a word back as text that [`parse_word`] reads to the same tokens.
pub fn word_text(word: &[Tok]) -> String {
    let mut s = String::new();
    for (k, t) in word.iter().enumerate() {
        match *t {
            Tok::Sym(c) => s.push(c as char),
            Tok::Turn(1) => s.push('+'),
            Tok::Turn(_) => s.push('-'),
            Tok::Around => s.push('|'),
            Tok::Reverse => s.push('!'),
            Tok::Push => s.push('['),
            Tok::Pop => s.push(']'),
            Tok::Scale(f) => s.push_str(&format!("@{f}")),
            Tok::TurnBy(a) if a >= 0.0 => s.push_str(&format!("\\{a}")),
            Tok::TurnBy(a) => s.push_str(&format!("/{}", -a)),
            Tok::SetColour(n) => s.push_str(&format!("C{n}")),
            Tok::AddColour(n) if n >= 0 => s.push_str(&format!("<{n}")),
            Tok::AddColour(n) => s.push_str(&format!(">{}", -(n as i64))),
            Tok::PolyStart => s.push('{'),
            Tok::PolyEnd => s.push('}'),
            Tok::Vertex => s.push('.'),
        }
        // A number followed by a digit symbol or a `.` would read as one longer number.
        let numeric = matches!(t, Tok::Scale(_) | Tok::TurnBy(_) | Tok::SetColour(_) | Tok::AddColour(_));
        let next_digit = matches!(word.get(k + 1), Some(Tok::Sym(c)) if c.is_ascii_digit()) || matches!(word.get(k + 1), Some(Tok::Vertex));
        // `C` then a digit symbol would read as a colour.
        let c_then_digit =
            matches!(t, Tok::Sym(b'C')) && matches!(word.get(k + 1), Some(Tok::Sym(c)) if c.is_ascii_digit());
        if (numeric && next_digit) || c_then_digit {
            s.push(' ');
        }
    }
    s
}

impl LSystem {
    /// An empty system: angle 90°, no axiom, the default roles.
    pub fn new(name: impl Into<String>) -> LSystem {
        LSystem {
            name: name.into(),
            angle: Angle::Degrees(90.0),
            heading: 0.0,
            axiom: Vec::new(),
            rules: vec![Vec::new(); 256],
            roles: (0..=255u8).map(default_role).collect(),
            colour: None,
            order: None,
            seed: DEFAULT_SEED,
            expanded: None,
        }
    }

    /// The word `c` rewrites to, if it rewrites — its first alternative, if it has several (see
    /// [`LSystem::productions`]).
    pub fn rule(&self, c: u8) -> Option<&[Tok]> {
        self.rules[c as usize].first().map(|p| p.word.as_slice())
    }

    /// Every production of `c` (empty: it does not rewrite).
    pub fn productions(&self, c: u8) -> &[Production] {
        &self.rules[c as usize]
    }

    /// Gives `c` the one production `word`.
    pub fn set_rule(&mut self, c: u8, word: Vec<Tok>) {
        self.rules[c as usize] = vec![Production { weight: 1.0, word }];
    }

    /// Whether any symbol chooses among alternatives.
    pub fn is_stochastic(&self) -> bool {
        self.rules.iter().any(|r| r.len() > 1)
    }

    /// Whether any word uses a bracket.
    pub fn branches(&self) -> bool {
        self.expanded.as_ref().is_some_and(|e| e.branches) || self.words().any(|w| w.contains(&Tok::Push))
    }

    /// Whether any word sets or steps the colour index.
    pub fn uses_colour_index(&self) -> bool {
        self.expanded.as_ref().is_some_and(|e| e.colour_index)
            || self.words().any(|w| w.iter().any(|t| matches!(t, Tok::SetColour(_) | Tok::AddColour(_))))
    }

    /// The axiom and every production's word.
    pub fn words(&self) -> impl Iterator<Item = &Vec<Tok>> {
        std::iter::once(&self.axiom).chain(self.rules.iter().flatten().map(|p| &p.word))
    }

    /// The colouring to draw with (design/lsystems.md §9.4): the system's own; else the colour
    /// index if it uses one, bracket depth if it branches, position along the curve if not.
    pub fn colouring(&self) -> Colouring {
        self.colour.unwrap_or(if self.uses_colour_index() {
            Colouring::Index
        } else if self.branches() {
            Colouring::Depth
        } else {
            Colouring::Position
        })
    }

    /// Reads the native format (design/lsystems.md §4.2):
    ///
    /// ```text
    /// # Koch snowflake
    /// angle 60
    /// axiom F++F++F
    /// F = F-F++F-F
    /// ```
    ///
    /// Keys: `name`, `angle` (degrees, or `/n` for a division of the circle), `heading`, `axiom`,
    /// `draw` / `move` / `variables` (symbols given that role), `colour`, `order`, `seed`; a
    /// production is `X = word`, or, for a symbol that chooses among alternatives at random,
    /// `X (weight) = word` once for each (ABOP's `F →(.33) F[+F]F`); `#` starts a comment line.
    pub fn parse(text: &str) -> Result<LSystem, ParseError> {
        let mut sys = LSystem::new("");
        let mut have_axiom = false;
        let mut have_angle = false;
        // Which symbols' productions were written with a weight.
        let mut weighted = [false; 256];
        // Pasted text: a lone CR ends a line, and a Unicode minus (as books print `F−F`) is '-'.
        let cleaned = fractadyne_text::clean(text);
        // A parametric or context-sensitive system (a `define`, an `ignore`, or a production with
        // parameters, a condition or a context) is read whole as one (super::expand), its axiom
        // and productions into `items`.
        let expanded = cleaned.text.lines().any(|raw| expanded_line(raw.trim()));
        let mut items: Vec<super::expand::Item> = Vec::new();
        for (k, raw) in cleaned.text.lines().enumerate() {
            let line = k + 1;
            let mut lead = raw.len() - raw.trim_start().len();
            let mut body = raw.trim();
            // ABOP's `#define` and `#ignore` (any other `#` line is a comment).
            if let Some(rest) = body.strip_prefix('#').filter(|r| key_word(r).is_some_and(|k| k == "define" || k == "ignore")) {
                body = rest;
                lead += 1;
            }
            if body.is_empty() || body.starts_with('#') {
                continue;
            }
            let sep = if key_word(body).is_some() { None } else { separator(body) };
            if let (true, Some((at, len))) = (expanded, sep) {
                let lhs = &body[..at];
                if production_lhs(lhs).is_some_and(|(_, w)| w.is_some_and(|t| t.parse::<f64>().is_ok())) {
                    return fail(line, lead + 1, "weighted alternatives cannot be combined with parameters or contexts");
                }
                items.push(super::expand::Item::Production {
                    line,
                    col: lead + 1,
                    lhs: lhs.to_string(),
                    rhs: body[at + len..].to_string(),
                    rhs_col: lead + at + len + 1,
                });
                continue;
            }
            // A production: one symbol (and a weight), then '='.
            if let Some((c, weight)) = sep.and_then(|(eq, _)| production_lhs(&body[..eq])) {
                let eq = sep.expect("found above").0;
                if !is_symbol(c) {
                    return fail(line, lead + 1, format!("'{}' cannot have a production", c as char));
                }
                let w = match weight {
                    None => 1.0,
                    Some(text) => match text.parse::<f64>() {
                        Ok(w) if w.is_finite() && w > 0.0 && w <= 1e6 => w,
                        _ => return fail(line, lead + 1, "a weight is a number above 0 ('X (0.3) = word')"),
                    },
                };
                let have = &sys.rules[c as usize];
                if !have.is_empty() && (weight.is_none() || !weighted[c as usize]) {
                    return fail(
                        line,
                        lead + 1,
                        format!("a second production for '{}' (alternatives each carry a weight: '{} (0.5) = word')", c as char, c as char),
                    );
                }
                if have.len() >= MAX_ALTERNATIVES {
                    return fail(line, lead + 1, format!("at most {MAX_ALTERNATIVES} alternatives for '{}'", c as char));
                }
                weighted[c as usize] = weight.is_some();
                let word = parse_word(&body[eq + 1..], line, lead + eq + 2, false)?;
                sys.rules[c as usize].push(Production { weight: w, word });
                continue;
            }
            let (key, value) = match body.find(char::is_whitespace) {
                Some(sp) => (&body[..sp], body[sp..].trim_start()),
                None => (body, ""),
            };
            let vcol = lead + 1 + (body.len() - value.len());
            // ABOP writes `#ignore: +-F`.
            let key = key.trim_end_matches(':');
            match key.to_ascii_lowercase().as_str() {
                "name" => sys.name = value.to_string(),
                "angle" => {
                    sys.angle = parse_angle(value).ok_or_else(|| ParseError {
                        line,
                        col: vcol,
                        message: "an angle is degrees (60, 22.5) or a division of the circle (/6)".into(),
                    })?;
                    have_angle = true;
                }
                "heading" => {
                    sys.heading = value
                        .parse::<f64>()
                        .ok()
                        .filter(|h| h.is_finite() && h.abs() <= 360.0)
                        .ok_or_else(|| ParseError { line, col: vcol, message: "a heading is degrees, −360 to 360".into() })?;
                }
                "axiom" if expanded => items.push(super::expand::Item::Axiom { line, col: vcol, text: value.to_string() }),
                "axiom" => {
                    if have_axiom {
                        return fail(line, 0, "a second axiom");
                    }
                    sys.axiom = parse_word(value, line, vcol, false)?;
                    have_axiom = true;
                }
                "define" => {
                    let (name, expr) = value.split_once(char::is_whitespace).unwrap_or((value, ""));
                    items.push(super::expand::Item::Define { line, col: vcol, name: name.to_string(), value: expr.to_string() });
                }
                "ignore" => items.push(super::expand::Item::Ignore { line, col: vcol, chars: value.to_string() }),
                "draw" | "move" | "variables" => {
                    let role = match key.to_ascii_lowercase().as_str() {
                        "draw" => Role::Draw,
                        "move" => Role::Move,
                        _ => Role::None,
                    };
                    for (j, c) in value.bytes().enumerate() {
                        if c == b' ' || c == b'\t' {
                            continue;
                        }
                        if !is_symbol(c) {
                            return fail(line, vcol + j, format!("'{}' is not a symbol", c as char));
                        }
                        sys.roles[c as usize] = role;
                    }
                }
                "colour" | "color" => {
                    sys.colour = Some(Colouring::from_key(value).ok_or_else(|| ParseError {
                        line,
                        col: vcol,
                        message: "a colouring is position, depth, heading, index or plain".into(),
                    })?);
                }
                "order" => {
                    sys.order = Some(value.parse::<u32>().ok().filter(|&n| n <= MAX_ORDER).ok_or_else(|| {
                        ParseError { line, col: vcol, message: format!("an order is a whole number up to {MAX_ORDER}") }
                    })?);
                }
                "seed" => {
                    sys.seed = value.parse::<u64>().map_err(|_| ParseError {
                        line,
                        col: vcol,
                        message: "a seed is a whole number, 0 or more".into(),
                    })?;
                }
                _ => return fail(line, lead + 1, format!("unknown key '{key}' (a production is 'X = word')")),
            }
        }
        if expanded {
            sys.expanded = Some(std::sync::Arc::new(super::expand::build(&items)?));
        } else if !have_axiom || sys.axiom.is_empty() {
            return fail(0, 0, "no axiom (the word the system starts from: 'axiom F')");
        }
        if !have_angle {
            return fail(0, 0, "no angle ('angle 60', or 'angle /6' for a sixth of the circle)");
        }
        Ok(sys)
    }

    /// The native text: [`LSystem::parse`] reads it back to the same system.
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        if !self.name.is_empty() {
            s.push_str(&format!("name {}\n", self.name));
        }
        match self.angle {
            Angle::Division(n) => s.push_str(&format!("angle /{n}\n")),
            Angle::Degrees(d) => s.push_str(&format!("angle {d}\n")),
        }
        if self.heading != 0.0 {
            s.push_str(&format!("heading {}\n", self.heading));
        }
        for (key, role) in [("draw", Role::Draw), ("move", Role::Move), ("variables", Role::None)] {
            let changed: String = (0..=255u8)
                .filter(|&c| is_symbol(c) && self.roles[c as usize] == role && default_role(c) != role)
                .map(|c| c as char)
                .collect();
            if !changed.is_empty() {
                s.push_str(&format!("{key} {changed}\n"));
            }
        }
        if let Some(c) = self.colour {
            s.push_str(&format!("colour {}\n", c.key()));
        }
        if let Some(n) = self.order {
            s.push_str(&format!("order {n}\n"));
        }
        if self.seed != DEFAULT_SEED || self.is_stochastic() {
            s.push_str(&format!("seed {}\n", self.seed));
        }
        if let Some(e) = &self.expanded {
            // Its definitions, axiom and productions as written.
            for l in &e.lines {
                s.push_str(l);
                s.push('\n');
            }
            return s;
        }
        s.push_str(&format!("axiom {}\n", word_text(&self.axiom)));
        for c in 0..=255u8 {
            let ps = self.productions(c);
            for p in ps {
                if ps.len() > 1 || p.weight != 1.0 {
                    s.push_str(&format!("{} ({}) = {}\n", c as char, p.weight, word_text(&p.word)));
                } else {
                    s.push_str(&format!("{} = {}\n", c as char, word_text(&p.word)));
                }
            }
        }
        s
    }
}

/// The keys a line can start with.
const KEYS: [&str; 13] =
    ["name", "angle", "heading", "axiom", "draw", "move", "variables", "colour", "color", "order", "seed", "define", "ignore"];

/// The key a line starts with, if it starts with one (`ignore: +-F` included).
fn key_word(body: &str) -> Option<&'static str> {
    let w = body.split(|c: char| c.is_whitespace() || c == ':').next()?;
    KEYS.iter().find(|k| k.eq_ignore_ascii_case(w)).copied()
}

/// Where a production's left side ends, and how long the separator is: at its `→` (as ABOP prints
/// them), or else at its first `=` that is not part of a comparison (`==`, `<=`, `>=`, `!=` in a
/// condition).
fn separator(body: &str) -> Option<(usize, usize)> {
    if let Some(i) = body.find('→') {
        return Some((i, '→'.len_utf8()));
    }
    let b = body.as_bytes();
    (0..b.len()).find(|&i| {
        b[i] == b'='
            && !matches!(i.checked_sub(1).map(|j| b[j]), Some(b'=' | b'<' | b'>' | b'!'))
            && b.get(i + 1) != Some(&b'=')
    })
    .map(|i| (i, 1))
}

/// Whether a line makes its system parametric or context-sensitive (see [`super::expand`]).
fn expanded_line(body: &str) -> bool {
    if let Some(rest) = body.strip_prefix('#') {
        return matches!(key_word(rest), Some("define" | "ignore"));
    }
    match key_word(body) {
        Some(k) => k == "define" || k == "ignore",
        None => separator(body).is_some_and(|(at, _)| super::expand::is_expanded_lhs(&body[..at])),
    }
}

/// A production's left side — a symbol, or a symbol and a weight in parentheses (`X (0.3)`) — as
/// the symbol and the weight's text. `None`: not a production's (a key line with an `=` in it).
fn production_lhs(lhs: &str) -> Option<(u8, Option<&str>)> {
    let lhs = lhs.trim();
    let c = *lhs.as_bytes().first()?;
    if !c.is_ascii() {
        return None;
    }
    if lhs.len() == 1 {
        return Some((c, None));
    }
    let inner = lhs[1..].trim_start().strip_prefix('(')?.strip_suffix(')')?;
    Some((c, Some(inner.trim())))
}

/// The highest order a system may be drawn at.
pub const MAX_ORDER: u32 = 4096;

fn parse_angle(v: &str) -> Option<Angle> {
    let v = v.trim();
    if let Some(n) = v.strip_prefix('/') {
        let n: u32 = n.trim().parse().ok()?;
        return (1..=Angle::MAX_DIVISION).contains(&n).then_some(Angle::Division(n));
    }
    let d: f64 = v.parse().ok()?;
    (d.is_finite() && d.abs() <= 360.0).then_some(Angle::Degrees(d))
}

#[cfg(test)]
mod tests;
