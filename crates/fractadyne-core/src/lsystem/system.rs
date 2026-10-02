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

/// A bracketed D0L system with turtle commands.
#[derive(Clone, Debug, PartialEq)]
pub struct LSystem {
    pub name: String,
    pub angle: Angle,
    /// The turtle's first heading, in degrees anticlockwise from +x (a plant grows up: 90).
    pub heading: f64,
    pub axiom: Vec<Tok>,
    /// What each symbol rewrites to (`None`: itself). 256 entries.
    pub rules: Vec<Option<Vec<Tok>>>,
    /// What each symbol does when read. 256 entries.
    pub roles: Vec<Role>,
    /// The colouring the system asks for (`None`: the default, [`LSystem::colouring`]).
    pub colour: Option<Colouring>,
    /// A fixed order to draw at (`None`: the order follows the zoom).
    pub order: Option<u32>,
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
    // The open brackets and braces, innermost last: each closes in the word that opens it.
    let mut open: Vec<u8> = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        let col = col0 + i;
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
        if out.len() > MAX_WORD {
            return fail(line, col, format!("a word may hold at most {MAX_WORD} symbols"));
        }
    }
    match open.last() {
        Some(b'{') => fail(line, col0 + b.len(), "a '{' is never closed"),
        Some(_) => fail(line, col0 + b.len(), "a '[' is never closed"),
        None => Ok(out),
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
            rules: vec![None; 256],
            roles: (0..=255u8).map(default_role).collect(),
            colour: None,
            order: None,
        }
    }

    /// The production for `c`, if it rewrites.
    pub fn rule(&self, c: u8) -> Option<&[Tok]> {
        self.rules[c as usize].as_deref()
    }

    /// Whether any word uses a bracket.
    pub fn branches(&self) -> bool {
        self.words().any(|w| w.contains(&Tok::Push))
    }

    /// Whether any word sets or steps the colour index.
    pub fn uses_colour_index(&self) -> bool {
        self.words().any(|w| w.iter().any(|t| matches!(t, Tok::SetColour(_) | Tok::AddColour(_))))
    }

    fn words(&self) -> impl Iterator<Item = &Vec<Tok>> {
        std::iter::once(&self.axiom).chain(self.rules.iter().flatten())
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
    /// `draw` / `move` / `variables` (symbols given that role), `colour`, `order`; a production is
    /// `X = word`; `#` starts a comment line.
    pub fn parse(text: &str) -> Result<LSystem, ParseError> {
        let mut sys = LSystem::new("");
        let mut have_axiom = false;
        let mut have_angle = false;
        // Pasted text: a lone CR ends a line, and a Unicode minus (as books print `F−F`) is '-'.
        let cleaned = fractadyne_text::clean(text);
        for (k, raw) in cleaned.text.lines().enumerate() {
            let line = k + 1;
            let lead = raw.len() - raw.trim_start().len();
            let body = raw.trim();
            if body.is_empty() || body.starts_with('#') {
                continue;
            }
            // A production: one symbol, then '='.
            if let Some(eq) = body.find('=') {
                let lhs = body[..eq].trim();
                if lhs.len() == 1 {
                    let c = lhs.as_bytes()[0];
                    if !is_symbol(c) {
                        return fail(line, lead + 1, format!("'{}' cannot have a production", c as char));
                    }
                    if sys.rules[c as usize].is_some() {
                        return fail(line, lead + 1, format!("a second production for '{}'", c as char));
                    }
                    let word = parse_word(&body[eq + 1..], line, lead + eq + 2, false)?;
                    sys.rules[c as usize] = Some(word);
                    continue;
                }
            }
            let (key, value) = match body.find(char::is_whitespace) {
                Some(sp) => (&body[..sp], body[sp..].trim_start()),
                None => (body, ""),
            };
            let vcol = lead + 1 + (body.len() - value.len());
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
                "axiom" => {
                    if have_axiom {
                        return fail(line, 0, "a second axiom");
                    }
                    sys.axiom = parse_word(value, line, vcol, false)?;
                    have_axiom = true;
                }
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
                _ => return fail(line, lead + 1, format!("unknown key '{key}' (a production is 'X = word')")),
            }
        }
        if !have_axiom || sys.axiom.is_empty() {
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
        s.push_str(&format!("axiom {}\n", word_text(&self.axiom)));
        for c in 0..=255u8 {
            if let Some(w) = self.rule(c) {
                s.push_str(&format!("{} = {}\n", c as char, word_text(w)));
            }
        }
        s
    }
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
