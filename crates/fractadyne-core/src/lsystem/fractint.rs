//! Fractint's `.l` files (design/lsystems.md §4.2): named entries `Name { … }`, each holding
//! `Angle n` (a division of the circle), `Axiom word` and productions `X=word`, one to a line; `;`
//! starts a comment; case is ignored, so `f` draws as `F` does. Read from Fractint's documentation
//! of the format, not from its code.

use super::system::{fail, parse_word, Angle, LSystem, ParseError};

/// One entry of a `.l` file: its name, the line it starts on, and the system or why it was refused.
#[derive(Clone, Debug)]
pub struct LEntry {
    pub name: String,
    pub line: usize,
    pub system: Result<LSystem, ParseError>,
}

/// The most entries one file may hold.
pub const MAX_ENTRIES: usize = 4096;

/// A line of an entry's body: its line number, the column its text starts at, the text.
type BodyLine = (usize, usize, String);

/// Reads every entry of a `.l` file. An entry that does not parse is kept, with its error, so a
/// file with one bad entry still opens the rest.
pub fn parse_l_file(text: &str) -> Vec<LEntry> {
    let cleaned = fractadyne_text::clean(text);
    let mut entries = Vec::new();
    // (name, first line, body lines).
    let mut open: Option<(String, usize, Vec<BodyLine>)> = None;
    let mut pending_name: Option<(String, usize)> = None;
    for (k, raw) in cleaned.text.lines().enumerate() {
        let line = k + 1;
        let code = raw.split(';').next().unwrap_or("");
        match &mut open {
            None => {
                let Some(brace) = code.find('{') else {
                    if !code.trim().is_empty() {
                        pending_name = Some((code.trim().to_string(), line));
                    }
                    continue;
                };
                let before = code[..brace].trim();
                let (name, first) = if before.is_empty() {
                    pending_name.take().unwrap_or_default()
                } else {
                    (before.to_string(), line)
                };
                pending_name = None;
                let mut body = Vec::new();
                let rest = &code[brace + 1..];
                let ended = rest.find('}');
                let inner = &rest[..ended.unwrap_or(rest.len())];
                body.push((line, brace + 2, inner.to_string()));
                if ended.is_some() {
                    entries.push(entry(name, first, &body));
                } else {
                    open = Some((name, first, body));
                }
            }
            Some((_, _, body)) => {
                if let Some(end) = code.find('}') {
                    body.push((line, 1, code[..end].to_string()));
                    let (name, first, body) = open.take().expect("an entry is open");
                    entries.push(entry(name, first, &body));
                } else {
                    body.push((line, 1, code.to_string()));
                }
            }
        }
        if entries.len() >= MAX_ENTRIES {
            break;
        }
    }
    if let Some((name, first, _)) = open {
        entries.push(LEntry {
            name,
            line: first,
            system: fail(first, 0, "this entry's '{' is never closed by a '}'"),
        });
    }
    entries
}

fn entry(name: String, first: usize, body: &[BodyLine]) -> LEntry {
    let system = entry_system(&name, first, body);
    LEntry { name, line: first, system }
}

fn entry_system(name: &str, first: usize, body: &[BodyLine]) -> Result<LSystem, ParseError> {
    let mut sys = LSystem::new(name);
    let mut angle = None;
    let mut axiom = None;
    for (line, col0, text) in body {
        let (line, col0) = (*line, *col0);
        let lead = text.len() - text.trim_start().len();
        let t = text.trim();
        if t.is_empty() {
            continue;
        }
        let col = col0 + lead;
        let lower = t.to_ascii_lowercase();
        if let Some(v) = keyword(&lower, "angle") {
            let n: u32 = v.trim().parse().ok().filter(|n| (1..=Angle::MAX_DIVISION).contains(n)).ok_or_else(|| {
                ParseError { line, col, message: "Angle is a whole number: the divisions of the circle".into() }
            })?;
            angle = Some(Angle::Division(n));
        } else if let Some(v) = keyword(&lower, "axiom") {
            if axiom.is_some() {
                return fail(line, col, "a second Axiom");
            }
            let at = t.len() - v.len();
            axiom = Some(parse_word(&t[at..], line, col + at, true)?);
        } else if let Some(eq) = t.find('=') {
            let lhs = t[..eq].trim().to_ascii_uppercase();
            let &[c] = lhs.as_bytes() else {
                return fail(line, col, "a production is one symbol, '=', then a word");
            };
            if !super::system::is_symbol(c) {
                return fail(line, col, format!("'{}' cannot have a production", c as char));
            }
            if sys.rules[c as usize].is_some() {
                return fail(line, col, format!("a second production for '{}'", c as char));
            }
            sys.rules[c as usize] = Some(parse_word(&t[eq + 1..], line, col + eq + 1, true)?);
        } else {
            return fail(line, col, "expected Angle, Axiom or a production 'X=word'");
        }
    }
    sys.angle = angle.ok_or_else(|| ParseError { line: first, col: 0, message: "no Angle".into() })?;
    sys.axiom = axiom.filter(|a| !a.is_empty()).ok_or_else(|| ParseError {
        line: first,
        col: 0,
        message: "no Axiom".into(),
    })?;
    Ok(sys)
}

/// `rest` after `key` and whitespace, if `s` starts with the keyword `key` (lower case).
fn keyword<'a>(s: &'a str, key: &str) -> Option<&'a str> {
    let rest = s.strip_prefix(key)?;
    (rest.is_empty() || rest.starts_with(char::is_whitespace)).then(|| rest.trim_start())
}

#[cfg(test)]
mod tests;
