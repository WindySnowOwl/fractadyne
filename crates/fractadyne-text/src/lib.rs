//! Line-ending and copy-paste hygiene for every text format Fractadyne reads.
//!
//! ⭐⭐**Why this is a crate and not a helper in one parser.** Locations, palettes and other apps'
//! files all arrive by the same routes — a download, a forum post, a chat message, a paste between
//! two editors — and every one of those routes can mangle the bytes on the way. Doing this in one
//! place means a `.ggr` from 2003 and a location pasted out of a browser get the same treatment,
//! and that the repairs are TESTED once rather than three times, differently.
//!
//! Dependency-free on purpose: `fractadyne-color` has no dependencies at all, and a shared helper
//! that forced a numerics crate into its build graph would be a bad trade for two hundred lines.
//!
//! Two problems are handled here.
//!
//! **1. Line endings.** `str::lines()` splits on `\n` and strips a trailing `\r`, so it handles
//! Unix and DOS — and NOT a lone `\r`, the classic-Mac ending that still turns up in old palette
//! files and in text mangled by a transfer that rewrote endings badly. Such a file arrives as ONE
//! enormous line, which every `key=value` parser reads as a single unparseable field. [`lines_any`]
//! treats all three as terminators.
//!
//! **2. Paste damage.** Text that has been through a word processor, a chat client, a web page or
//! a PDF comes back subtly changed, and the changes are invisible:
//!
//! | what arrives | instead of | why it matters |
//! |---|---|---|
//! | `−` U+2212 MINUS SIGN | `-` HYPHEN | **a negative coordinate stops parsing** |
//! | `–` `—` en/em dash | `-` | same |
//! | `"` `"` `'` `'` smart quotes | `"` `'` | quoted values stop matching |
//! | U+00A0 NO-BREAK SPACE | space | trims to nothing, or a number gains a space |
//! | U+200B ZERO WIDTH SPACE | nothing | **completely invisible**; the value silently fails |
//! | U+FEFF BOM | nothing | a leading BOM breaks the very first key |
//! | full-width `０-９` `．` `＋` `－` | ASCII | numbers stop being numbers |
//!
//! ⭐⭐**These are REPAIRED, not rejected, and every repair is REPORTED.** Rejecting a pasted
//! location because a chat client turned a hyphen into a minus sign is technically defensible and
//! practically useless — the user cannot see the difference and has no way to act on "invalid
//! input". Repairing silently is worse: it hides that their clipboard is mangling data, which they
//! will hit again. So [`clean`] fixes what has an unambiguous intent and hands back a [`Repair`]
//! for each one, with the line and column to point at.
//!
//! ⚠**Bidi controls are removed and flagged.** U+202A–U+202E and U+2066–U+2069 can reorder how a
//! line DISPLAYS without changing what it parses to — the Trojan Source trick. Nothing legitimate
//! puts them in a coordinate.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests;

pub mod base64;

/// What [`clean`] did to one character, and where it was in the ORIGINAL text.
///
/// ⭐Line and column are 1-based and counted in CHARACTERS, not bytes — the numbers a person sees
/// in an editor. A byte offset would be correct and unusable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repair {
    pub line: usize,
    pub col: usize,
    /// The character that was found, for display (`U+200B`).
    pub found: char,
    /// Human name of the offender, e.g. `"zero-width space"`.
    pub name: &'static str,
    /// What it became — `None` when it was simply removed.
    pub replaced_with: Option<char>,
}

impl std::fmt::Display for Repair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, col {}: {} (U+{:04X})", self.line, self.col, self.name, self.found as u32)?;
        match self.replaced_with {
            Some(c) => write!(f, " replaced with '{c}'"),
            None => write!(f, " removed"),
        }
    }
}

/// The result of [`clean`]: text safe to parse, plus what had to be fixed to get there.
#[derive(Debug, Clone, Default)]
pub struct Cleaned {
    /// The repaired text, with every line ending normalized to `\n`.
    pub text: String,
    /// One entry per repaired character, in order.
    pub repairs: Vec<Repair>,
}

impl Cleaned {
    /// A short human summary of the repairs, collapsed by kind — the form to put in a load report.
    ///
    /// ⚠Collapsed deliberately: a location pasted out of a word processor can carry dozens of
    /// smart quotes, and thirty near-identical lines is not a better message than one.
    pub fn summary(&self) -> Option<String> {
        if self.repairs.is_empty() {
            return None;
        }
        let mut kinds: Vec<(&'static str, usize, &Repair)> = Vec::new();
        for r in &self.repairs {
            match kinds.iter_mut().find(|(n, _, _)| *n == r.name) {
                Some((_, n, _)) => *n += 1,
                None => kinds.push((r.name, 1, r)),
            }
        }
        let parts: Vec<String> = kinds
            .iter()
            .map(|(name, n, first)| {
                if *n == 1 {
                    format!("{name} at line {}, col {}", first.line, first.col)
                } else {
                    format!("{n}× {name} (first at line {}, col {})", first.line, first.col)
                }
            })
            .collect();
        Some(format!(
            "repaired copy-and-paste damage: {}",
            parts.join("; ")
        ))
    }
}

/// Classify one character. `None` means "leave it alone".
///
/// ⛔**Only characters whose intent is unambiguous appear here.** A general "fold everything to
/// ASCII" would corrupt a legitimate note or gradient name written in another language; the point
/// is to fix what a clipboard broke, not to impose ASCII on the user's own text.
fn repair_for(c: char) -> Option<(&'static str, Option<char>)> {
    Some(match c {
        // ---- invisible: removed outright
        '\u{200B}' => ("zero-width space", None),
        '\u{200C}' => ("zero-width non-joiner", None),
        '\u{200D}' => ("zero-width joiner", None),
        '\u{2060}' => ("word joiner", None),
        '\u{FEFF}' => ("byte-order mark", None),
        '\u{00AD}' => ("soft hyphen", None),
        '\u{180E}' => ("Mongolian vowel separator", None),
        // ---- bidi controls: display-reordering, never legitimate here
        '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' => ("bidirectional control", None),
        // ---- spaces that are not the space key
        '\u{00A0}' => ("no-break space", Some(' ')),
        '\u{2007}' => ("figure space", Some(' ')),
        '\u{2009}' => ("thin space", Some(' ')),
        '\u{202F}' => ("narrow no-break space", Some(' ')),
        '\u{3000}' => ("ideographic space", Some(' ')),
        // ---- smart quotes
        '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => ("smart quote", Some('\'')),
        '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => ("smart double quote", Some('"')),
        // ---- dashes. ⚠⚠The one that actually bites: U+2212 in a negative coordinate.
        '\u{2010}' | '\u{2011}' => ("typographic hyphen", Some('-')),
        '\u{2012}' => ("figure dash", Some('-')),
        '\u{2013}' => ("en dash", Some('-')),
        '\u{2014}' => ("em dash", Some('-')),
        '\u{2015}' => ("horizontal bar", Some('-')),
        '\u{2212}' => ("Unicode minus sign", Some('-')),
        '\u{FE63}' => ("small hyphen-minus", Some('-')),
        '\u{FF0D}' => ("full-width hyphen-minus", Some('-')),
        // ---- full-width forms of the characters numbers are made of
        '\u{FF10}'..='\u{FF19}' => (
            "full-width digit",
            Some(char::from(b'0' + (c as u32 - 0xFF10) as u8)),
        ),
        '\u{FF0B}' => ("full-width plus", Some('+')),
        '\u{FF0E}' => ("full-width full stop", Some('.')),
        '\u{FF1D}' => ("full-width equals", Some('=')),
        '\u{FF1A}' => ("full-width colon", Some(':')),
        '\u{FF0C}' => ("full-width comma", Some(',')),
        _ => return None,
    })
}

/// Normalize line endings to `\n` and repair copy-and-paste damage.
///
/// ⭐Positions in the returned [`Repair`]s refer to the INPUT, so they match what the user sees in
/// whatever they pasted from — not to the cleaned output, whose columns have already shifted.
pub fn clean(input: &str) -> Cleaned {
    let mut text = String::with_capacity(input.len());
    let mut repairs = Vec::new();
    let (mut line, mut col) = (1usize, 1usize);
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            // ⚠A lone `\r` is a line ending too; `\r\n` is one ending, not two.
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                text.push('\n');
                line += 1;
                col = 1;
            }
            '\n' => {
                text.push('\n');
                line += 1;
                col = 1;
            }
            _ => {
                match repair_for(c) {
                    Some((name, replacement)) => {
                        repairs.push(Repair { line, col, found: c, name, replaced_with: replacement });
                        if let Some(r) = replacement {
                            text.push(r);
                        }
                    }
                    None => text.push(c),
                }
                col += 1;
            }
        }
    }
    Cleaned { text, repairs }
}

/// Iterate lines split on `\r\n`, `\n` **or** a lone `\r`, yielding `(1-based line number, line)`.
///
/// ⭐Use this for any text that did NOT come through [`clean`]. Text that did is already normalized
/// and plain `.lines()` is enough — but this stays correct either way.
///
/// ⚠A naive `split(['\n', '\r'])` is NOT this: it turns every `\r\n` into a spurious empty line and
/// throws off every line number after the first one, which is exactly the kind of off-by-N that
/// makes a diagnostic worse than useless.
pub fn lines_any(s: &str) -> impl Iterator<Item = (usize, &str)> {
    numbered_lines(s).into_iter()
}

/// Split into logical lines, correctly collapsing `\r\n`, and number them from 1.
pub fn numbered_lines(s: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let (mut start, mut i, mut n) = (0usize, 0usize, 1usize);
    while i < bytes.len() {
        match bytes[i] {
            b'\r' => {
                out.push((n, &s[start..i]));
                n += 1;
                i += if i + 1 < bytes.len() && bytes[i + 1] == b'\n' { 2 } else { 1 };
                start = i;
            }
            b'\n' => {
                out.push((n, &s[start..i]));
                n += 1;
                i += 1;
                start = i;
            }
            _ => i += 1,
        }
    }
    if start < bytes.len() {
        out.push((n, &s[start..]));
    }
    out
}
