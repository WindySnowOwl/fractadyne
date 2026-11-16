//! Locations saved as `.txt`. Fractal Forums accepts `.txt` attachments but not `.fdn`, so a location
//! shared there arrives as `name.fdn.txt` (user, 2026-11-16); a Kalles Fraktaler `.kfr` can arrive
//! the same way. This module reads such a file and says which program's format it holds; the
//! Open dialog loads it through that program's own loader, or asks when it cannot tell.
//!
//! ⭐⭐**Untrusted input, sanitized before any parser sees it.** A `.txt` can be anything: a renamed
//! image, a log of gigabytes, a file saved by Notepad in UTF-16. So:
//! - the size is checked before reading and the read itself is bounded ([`TXT_MAX`]), so a file
//!   that grows while it is read cannot get past the check;
//! - bytes become text by UTF-8 (a BOM dropped) or by UTF-16 when the file starts with its BOM;
//!   invalid UTF-8 is replaced, not refused (repair, don't reject: it can only be in a note — every
//!   field a parser reads is ASCII);
//! - a NUL means binary data, which is refused rather than parsed as text;
//! - every other control character except tab, line feed and carriage return is removed.
//!
//! What it then holds is decided by the two existing gates, unchanged: [`crate::location_text_verdict`]
//! (the `.fdn` gate) and `fractadyne_core::parse_kfr` (the hardened `.kfr` parser), both run on the
//! text after `fractadyne_text::clean` (line endings, clipboard damage), as their own loaders do.
//! The loaders then apply their own bounds and field-by-field validation as for `.fdn` and `.kfr`.

use std::io::Read;

/// The most a location text file may hold: the `.kfr` reader's cap. The `.fdn` route applies its
/// own, smaller one (`SHARE_MAX`) on top.
pub(crate) const TXT_MAX: u64 = 4_000_000;

/// Why a file could not be read as location text.
#[derive(Debug)]
pub(crate) enum TxtError {
    /// Over [`TXT_MAX`] bytes.
    TooLarge,
    /// It holds binary data (a NUL byte): not a text file.
    Binary,
    Io(std::io::Error),
}

/// A program whose location format the text parses as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TxtKind {
    Fractadyne,
    KallesFraktaler,
}

/// Read `path` as location text, sanitized (see the module doc).
pub(crate) fn read_location_text(path: &std::path::Path) -> Result<String, TxtError> {
    read_bounded(path, TXT_MAX)
}

/// [`read_location_text`] with the cap as a parameter (the tests use a small one).
pub(crate) fn read_bounded(path: &std::path::Path, cap: u64) -> Result<String, TxtError> {
    let len = std::fs::metadata(path).map_err(TxtError::Io)?.len();
    if len > cap {
        return Err(TxtError::TooLarge);
    }
    let mut bytes = Vec::with_capacity(len as usize);
    std::fs::File::open(path)
        .map_err(TxtError::Io)?
        .take(cap + 1)
        .read_to_end(&mut bytes)
        .map_err(TxtError::Io)?;
    if bytes.len() as u64 > cap {
        return Err(TxtError::TooLarge); // it grew after the size check
    }
    decode_location_bytes(&bytes)
}

/// Bytes → location text: UTF-16 by its BOM, else UTF-8 (BOM dropped, invalid sequences replaced);
/// a NUL refuses it as binary; other control characters but tab, LF and CR are removed.
pub(crate) fn decode_location_bytes(bytes: &[u8]) -> Result<String, TxtError> {
    let utf16 = |le: bool| {
        let units: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|p| if le { u16::from_le_bytes([p[0], p[1]]) } else { u16::from_be_bytes([p[0], p[1]]) })
            .collect();
        String::from_utf16_lossy(&units)
    };
    let text = match bytes {
        [0xFF, 0xFE, ..] => utf16(true),
        [0xFE, 0xFF, ..] => utf16(false),
        [0xEF, 0xBB, 0xBF, rest @ ..] => String::from_utf8_lossy(rest).into_owned(),
        _ => String::from_utf8_lossy(bytes).into_owned(),
    };
    if text.contains('\0') {
        return Err(TxtError::Binary);
    }
    Ok(text.chars().filter(|&c| !c.is_control() || matches!(c, '\t' | '\n' | '\r')).collect())
}

/// A path as the prompt and the messages show it: the bidirectional controls removed, so a file
/// name cannot display as another (U+202E shows `location.txt.exe` reversed from that point). Only
/// what is shown; the file is opened by its real path.
pub(crate) fn shown_path(p: &std::path::Path) -> String {
    p.display()
        .to_string()
        .chars()
        .filter(|c| !matches!(c, '\u{061C}' | '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'))
        .collect()
}

/// The formats `text` parses as: none, one, or (in principle) both.
pub(crate) fn location_kinds(text: &str) -> Vec<TxtKind> {
    let cleaned = fractadyne_text::clean(text);
    let t = cleaned.text.as_str();
    let mut kinds = Vec::new();
    if crate::location_text_verdict(t).is_ok() {
        kinds.push(TxtKind::Fractadyne);
    }
    if fractadyne_core::parse_kfr(t).is_some() {
        kinds.push(TxtKind::KallesFraktaler);
    }
    kinds
}

#[cfg(test)]
#[path = "../tests/unit/txt_location/tests.rs"]
mod tests;
