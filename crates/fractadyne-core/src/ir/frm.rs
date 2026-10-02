//! Fractint's `.frm` formula files (design/custom-formulas.md §4.2 item 2): each entry read, and
//! its body put into the formula language — which is Fractint's own, but for two things:
//!
//! - `fn1`…`fn4`, the slots Fractint fills with a function chosen on its parameter screen, become
//!   Fractint's default choices (sin, sqr, sinh, cosh), which the user can edit afterwards;
//! - `c`, an ordinary variable in Fractint but the pixel here, is renamed (`c_`), so a formula
//!   that sets `c = pixel` and iterates with it computes what Fractint computes.
//!
//! Names are lower-cased (Fractint's are case-blind). The entry `name(SYMMETRY) { body }` — the
//! symmetry is a rendering hint and is dropped; `comment { … }` blocks and text outside braces are
//! commentary. A body that still does not read says why (an unsupported feature — `whitesq`,
//! `rand`, … — is named), and is reported, never mis-read.

use super::parse::{parse, syntax, unassigned_name};
use super::syntax::ExprKind;

/// Fractint's default functions for the `fn1`…`fn4` slots.
pub const FN_DEFAULTS: [&str; 4] = ["sin", "sqr", "sinh", "cosh"];

/// One entry of a `.frm` file.
#[derive(Clone, Debug, PartialEq)]
pub struct FrmEntry {
    pub name: String,
    /// The body in the formula language, ready to parse.
    pub source: String,
    /// What the translation changed (`fn1` → `sin`, `c` → `c_`), for the import report.
    pub notes: Vec<String>,
    /// Whether it reads, and why not (the parser's message).
    pub reads: Result<(), String>,
}

/// Every entry of a `.frm` file as read from disk: UTF-8 if it is, else Latin-1 (the DOS-era files
/// are, and their accents sit in comments).
pub fn read_frm_bytes(bytes: &[u8]) -> Vec<FrmEntry> {
    match std::str::from_utf8(bytes) {
        Ok(text) => read_frm(text),
        Err(_) => read_frm(&bytes.iter().map(|&b| b as char).collect::<String>()),
    }
}

/// Every entry of a `.frm` file, in order.
pub fn read_frm(text: &str) -> Vec<FrmEntry> {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    // Fractint's line continuation: a backslash ending a line (spaces after it allowed) — not one
    // ending a comment, which would swallow the next line's code.
    let mut joined = String::with_capacity(text.len());
    for line in text.split('\n') {
        match line.trim_end().strip_suffix('\\').filter(|_| !line.contains(';')) {
            Some(head) => {
                joined.push_str(head);
                joined.push(' ');
            }
            None => {
                joined.push_str(line);
                joined.push('\n');
            }
        }
    }
    let text = joined;
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(open) = find_open(&text, i) {
        // The header is the last line of code before the brace (usually the brace's own line).
        let header = text[i..open].lines().map(|l| l.split(';').next().unwrap_or("").trim()).rfind(|l| !l.is_empty()).unwrap_or("");
        let Some(close) = find_close(&text, open + 1) else { break };
        let body = &text[open + 1..close];
        i = close + 1;
        let name = header.split(['(', '[']).next().unwrap_or("").trim();
        // `comment { … }` is documentation, by Fractint's convention.
        if name.is_empty() || name.eq_ignore_ascii_case("comment") {
            continue;
        }
        let (source, mut notes) = translate(body.trim_matches('\n'));
        let source = final_test(source, &mut notes);
        let (source, reads) = zero_unset(source, &mut notes);
        out.push(FrmEntry { name: name.to_string(), source, notes, reads });
    }
    out
}

/// The next `{` at or after `from` that is not inside a comment.
fn find_open(text: &str, from: usize) -> Option<usize> {
    scan(text, from, b'{')
}

/// The `}` closing an entry, skipping comments.
fn find_close(text: &str, from: usize) -> Option<usize> {
    scan(text, from, b'}')
}

fn scan(text: &str, from: usize, want: u8) -> Option<usize> {
    let b = text.as_bytes();
    let mut k = from;
    while k < b.len() {
        match b[k] {
            b';' => {
                while k < b.len() && b[k] != b'\n' {
                    k += 1;
                }
            }
            c if c == want => return Some(k),
            _ => k += 1,
        }
    }
    None
}

/// Fractint ignores blanks inside a formula: `end if` is `endif`, `else if` is `elseif`, `sqrt 5`
/// the variable `sqrt5`. So blanks between two word characters of a line's code are dropped, and
/// the words so joined are returned.
fn join_words(body: &str) -> (String, Vec<String>) {
    let word = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '.';
    let mut out = String::with_capacity(body.len());
    let mut joined = Vec::new();
    for (k, line) in body.split('\n').enumerate() {
        if k > 0 {
            out.push('\n');
        }
        let (code, comment) = line.split_at(line.find(';').unwrap_or(line.len()));
        let chars: Vec<char> = code.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == ' ' || c == '\t' {
                let mut j = i;
                while j < chars.len() && (chars[j] == ' ' || chars[j] == '\t') {
                    j += 1;
                }
                let before = out.chars().last().filter(|&b| word(b));
                let after = chars.get(j).copied().filter(|&a| word(a));
                if let (Some(_), Some(_)) = (before, after) {
                    if i > 0 {
                        // The joined word, for the note: back to its start, on to its end.
                        let start = chars[..i].iter().rposition(|&b| !word(b)).map_or(0, |p| p + 1);
                        let end = chars[j..].iter().position(|&a| !word(a)).map_or(chars.len(), |p| j + p);
                        let w: String = chars[start..end].iter().filter(|&&ch| ch != ' ' && ch != '\t').collect();
                        joined.push(w.to_ascii_lowercase());
                    }
                    i = j;
                    continue;
                }
                out.extend(&chars[i..j]);
                i = j;
                continue;
            }
            out.push(c);
            i += 1;
        }
        out.push_str(comment);
    }
    joined.dedup();
    (out, joined)
}

/// The body in the formula language, and what changed.
fn translate(body: &str) -> (String, Vec<String>) {
    let mut notes = Vec::new();
    let (body, joined) = join_words(body);
    if !joined.is_empty() {
        let shown: Vec<_> = joined.iter().take(3).map(String::as_str).collect();
        notes.push(format!("blanks inside names are dropped, as Fractint drops them: {}", shown.join(", ")));
    }
    let body = body.as_str();
    // Every name the body uses, to pick a free one for Fractint's `c`.
    let names = identifiers(body);
    let c_name = (0..).map(|k| format!("c{}", "_".repeat(k + 1))).find(|n| !names.iter().any(|m| m == n)).expect("unbounded");
    let mut out = String::with_capacity(body.len());
    let mut renamed_c = false;
    let mut used_fn = [false; 4];
    let b = body.as_bytes();
    let mut k = 0;
    while k < b.len() {
        let ch = b[k];
        if ch == b';' {
            let end = body[k..].find('\n').map_or(body.len(), |e| k + e);
            out.push_str(&body[k..end]);
            k = end;
            continue;
        }
        let starts_name = (ch.is_ascii_alphabetic() || ch == b'_') && (k == 0 || !(b[k - 1].is_ascii_alphanumeric() || b[k - 1] == b'_' || b[k - 1] == b'.'));
        if !starts_name {
            out.push(ch as char);
            k += 1;
            continue;
        }
        let mut e = k;
        while e < b.len() && (b[e].is_ascii_alphanumeric() || b[e] == b'_') {
            e += 1;
        }
        let word = &body[k..e];
        let lower = word.to_ascii_lowercase();
        match lower.as_str() {
            "c" => {
                out.push_str(&c_name);
                renamed_c = true;
            }
            "fn1" | "fn2" | "fn3" | "fn4" => {
                let slot = (lower.as_bytes()[2] - b'1') as usize;
                out.push_str(FN_DEFAULTS[slot]);
                used_fn[slot] = true;
            }
            // Fractint's names are case-blind; the formula language's are not.
            _ => out.push_str(&lower),
        }
        k = e;
    }
    for (slot, used) in used_fn.iter().enumerate() {
        if *used {
            notes.push(format!("fn{} is {} (Fractint's default)", slot + 1, FN_DEFAULTS[slot]));
        }
    }
    if renamed_c {
        notes.push(format!("Fractint's variable c is {c_name} (c is the pixel here)"));
    }
    (out, notes)
}

/// Fractint's last loop statement is always the bailout test, iterating while its real part is
/// not zero; this language reads a final bare expression that does not compare as the new z. So
/// such an expression is written as the test it is: `(x) != 0`.
fn final_test(source: String, notes: &mut Vec<String>) -> String {
    let Ok(tree) = syntax(&source) else { return source };
    let Some(last) = tree.statements.last() else { return source };
    let rest = &source[last.span.end..];
    let at_end = rest.lines().all(|l| l.split(';').next().unwrap_or("").trim_matches(|c: char| c.is_whitespace() || c == ',').is_empty());
    if last.target.is_some() || !at_end || matches!(last.body.kind, ExprKind::Cmp(..) | ExprKind::Logic(..)) {
        return source;
    }
    let text = &source[last.span.start..last.span.end];
    notes.push(format!("the last statement, {text}, is the bailout test (as in Fractint): {text} != 0"));
    format!("{}({}) != 0{}", &source[..last.span.start], text, rest)
}

/// A name nothing assigns is 0 in Fractint (a misspelling, often, that the formula's pictures were
/// made with); this language refuses it. So each one is set to 0 in the init section, and said.
fn zero_unset(mut source: String, notes: &mut Vec<String>) -> (String, Result<(), String>) {
    let mut zeroed = Vec::new();
    loop {
        match parse(&source) {
            Ok(_) => break,
            Err(e) => match unassigned_name(&e) {
                Some(name) if zeroed.len() < 16 && !zeroed.iter().any(|n| n == name) => {
                    let name = name.to_string();
                    source = if has_init_section(&source) { format!("{name} = 0\n{source}") } else { format!("{name} = 0:\n{source}") };
                    zeroed.push(name);
                }
                _ => {
                    for name in &zeroed {
                        notes.push(format!("{name} is never set: 0, as in Fractint"));
                    }
                    return (source, Err(e.message));
                }
            },
        }
    }
    for name in &zeroed {
        notes.push(format!("{name} is never set: 0, as in Fractint"));
    }
    (source, Ok(()))
}

/// Whether `source` has an init section: a `:` outside its comments.
fn has_init_section(source: &str) -> bool {
    source.lines().any(|l| l.split(';').next().unwrap_or("").contains(':'))
}

/// The identifiers in `text`, lower-cased, outside comments.
fn identifiers(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let code = line.split(';').next().unwrap_or("");
        for w in code.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
            if w.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
                let w = w.to_ascii_lowercase();
                if !out.contains(&w) {
                    out.push(w);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests;
