//! The pseudo-language `en-XA` (design/multilingual.md §10): English with its letters accented, 40%
//! longer, in brackets. On screen, text without brackets is still hard-coded English, and text cut
//! off at the bracket is text a real translation will overflow.

pub(crate) const TAG: &str = "en-XA";
pub(crate) const NAME: &str = "[Pšéüðö-Éñĝłîšħ]";
pub(crate) const ENGLISH_NAME: &str = "Pseudo-language (tests layouts)";

/// `s` as the pseudo-language shows it.
pub(crate) fn transform(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    let letters = s.chars().filter(|c| c.is_alphabetic()).count();
    let mut out = String::with_capacity(s.len() * 2 + 8);
    out.push('[');
    for c in s.chars() {
        out.push(accent(c));
    }
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
