//! The pseudo-language `en-XA` (design/multilingual.md §10): English with its letters accented, 40%
//! longer, in brackets. On screen, text without brackets is still hard-coded English, and text cut
//! off at the bracket is text a real translation will overflow.

pub(crate) const TAG: &str = "en-XA";
pub(crate) const NAME: &str = "[Pšéüðö-Éñĝłîšħ]";
pub(crate) const ENGLISH_NAME: &str = "Pseudo-language (tests layouts)";

/// A message's literal text accented: the bundle's transform, which Fluent applies to the text a
/// message writes and not to its arguments, so a path or a name passed in shows as it is.
pub(crate) fn accent_text(s: &str) -> std::borrow::Cow<'_, str> {
    if s.chars().any(|c| accent(c) != c) {
        std::borrow::Cow::Owned(s.chars().map(accent).collect())
    } else {
        std::borrow::Cow::Borrowed(s)
    }
}

/// A formatted message bracketed and 40% longer.
pub(crate) fn wrap(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    let letters = s.chars().filter(|c| c.is_alphabetic()).count();
    let mut out = String::with_capacity(s.len() * 2 + 8);
    out.push('[');
    out.push_str(s);
    // Translations run longer than English: German and French by 20–40%.
    let pad = (letters * 2).div_ceil(5);
    if pad > 0 {
        out.push(' ');
        out.extend(std::iter::repeat_n('~', pad));
    }
    out.push(']');
    out
}

/// An accented look-alike of an ASCII letter (others unchanged), from Latin-1 and Latin Extended-A,
/// which the interface fonts cover.
fn accent(c: char) -> char {
    match c {
        'a' => 'å',
        'c' => 'ç',
        'e' => 'é',
        'g' => 'ĝ',
        'i' => 'î',
        'n' => 'ñ',
        'o' => 'ö',
        's' => 'š',
        'u' => 'ü',
        'y' => 'ý',
        'z' => 'ž',
        'A' => 'Å',
        'C' => 'Ç',
        'E' => 'É',
        'I' => 'Î',
        'N' => 'Ñ',
        'O' => 'Ö',
        'S' => 'Š',
        'U' => 'Ü',
        'Z' => 'Ž',
        c => c,
    }
}
