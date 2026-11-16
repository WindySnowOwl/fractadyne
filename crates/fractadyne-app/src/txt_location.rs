//! Location files read as text: `.fdn`, and `.txt`. Fractal Forums accepts `.txt` attachments but
//! not `.fdn`, so a location shared there arrives as `name.fdn.txt` (user, 2026-11-16); a Kalles
//! Fraktaler `.kfr` can arrive the same way. This module reads such a file, says which program's
//! format it holds, and — when it holds neither, or cannot be read — says why and what it inferred
//! (user, 2026-11-16: "if it fails, try to give useful info about why and what format the app
//! inferred").
//!
//! ⭐⭐**Untrusted input, sanitized before any parser sees it.** A `.txt` can be anything: a renamed
//! image, a log of gigabytes, a file saved by Notepad in UTF-16. So:
//! - the size is checked before reading and the read itself is bounded ([`TXT_MAX`]), so a file
//!   that grows while it is read cannot get past the check;
//! - a file whose first bytes are a binary format's signature (PNG, PDF, ZIP…) is refused, as is
//!   one holding a zero character after decoding — never parsed as text;
//! - the encoding is inferred: a UTF-32, UTF-8 or UTF-16 byte-order mark; else UTF-16 without one
//!   when every other byte is zero; else UTF-8; else (not valid UTF-8) Windows-1252, the encoding
//!   of most old Windows text, so a note's "café" survives instead of becoming "caf�";
//! - control characters other than tab, line feed and carriage return are removed (and counted);
//! - line endings are left alone and counted: `fractadyne_text::clean` normalizes CR, LF and CRLF
//!   (mixed too) for every parser.
//!
//! What it holds is decided by the two existing gates, unchanged — [`crate::location_text_verdict`]
//! (the `.fdn` gate) and `fractadyne_core::parse_kfr` (the hardened `.kfr` parser) — on the text
//! after `clean`, as their loaders do; the loaders then validate field by field as before.

use std::io::Read;

/// The most a location text file may hold: the `.kfr` reader's cap. The `.fdn` route applies its
/// own, smaller one (`SHARE_MAX`) on top.
pub(crate) const TXT_MAX: u64 = 4_000_000;

/// Why a file could not be read as location text.
#[derive(Debug)]
pub(crate) enum TxtError {
    /// Over the cap.
    TooLarge { size: u64, limit: u64 },
    /// Binary data: where it shows (the first zero byte, or 0 for a signature), and the format the
    /// first bytes match, if one is known.
    Binary { offset: usize, looks_like: Option<&'static str> },
    Io(std::io::Error),
}

/// How the bytes were read as text: what the messages report the app inferred.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextInfo {
    /// `UTF-8`, `UTF-8 + BOM`, `UTF-16 LE + BOM`, `UTF-16 LE` (no mark: inferred from its zero
    /// bytes), the same for BE, `UTF-32 LE + BOM` / `BE`, or `Windows-1252` (not valid UTF-8).
    pub(crate) encoding: &'static str,
    /// Line breaks by kind: `\r\n`, a lone `\n`, a lone `\r`.
    pub(crate) crlf: usize,
    pub(crate) lf: usize,
    pub(crate) cr: usize,
    /// Control characters removed.
    pub(crate) removed: usize,
}

impl TextInfo {
    /// The line endings found, as tokens every language shows as they are: `CRLF`, `LF`, `CR`, or
    /// several with their counts (`CRLF 12 + CR 12`, the mark of a file converted twice). `None`
    /// for a single line.
    pub(crate) fn endings(&self) -> Option<String> {
        let kinds: Vec<(&str, usize)> =
            [("CRLF", self.crlf), ("LF", self.lf), ("CR", self.cr)].into_iter().filter(|&(_, n)| n > 0).collect();
        match kinds.as_slice() {
            [] => None,
            [(k, _)] => Some((*k).to_string()),
            _ => Some(kinds.iter().map(|(k, n)| format!("{k} {n}")).collect::<Vec<_>>().join(" + ")),
        }
    }
}

/// A file's text and how it was read.
#[derive(Debug)]
pub(crate) struct Decoded {
    pub(crate) text: String,
    pub(crate) info: TextInfo,
}

/// Read `path` as location text, sanitized (see the module doc).
pub(crate) fn read_location_text(path: &std::path::Path) -> Result<Decoded, TxtError> {
    read_bounded(path, TXT_MAX)
}

/// [`read_location_text`] with the cap as a parameter (Load location uses `SHARE_MAX`; the tests a
/// small one).
pub(crate) fn read_bounded(path: &std::path::Path, cap: u64) -> Result<Decoded, TxtError> {
    let size = std::fs::metadata(path).map_err(TxtError::Io)?.len();
    if size > cap {
        return Err(TxtError::TooLarge { size, limit: cap });
    }
    let mut bytes = Vec::with_capacity(size as usize);
    std::fs::File::open(path)
        .map_err(TxtError::Io)?
        .take(cap + 1)
        .read_to_end(&mut bytes)
        .map_err(TxtError::Io)?;
    if bytes.len() as u64 > cap {
        // It grew after the size check.
        return Err(TxtError::TooLarge { size: bytes.len() as u64, limit: cap });
    }
    decode_location_bytes(&bytes)
}

/// The binary format whose signature `b` starts with, if one is known. Names are format names,
/// shown as they are in every language.
pub(crate) fn sniff_binary(b: &[u8]) -> Option<&'static str> {
    const SIGS: &[(&[u8], &str)] = &[
        (b"\x89PNG\r\n\x1a\n", "PNG"),
        (b"\xFF\xD8\xFF", "JPEG"),
        (b"GIF87a", "GIF"),
        (b"GIF89a", "GIF"),
        (b"\x76\x2F\x31\x01", "OpenEXR"),
        (b"PK\x03\x04", "ZIP"),
        (b"%PDF-", "PDF"),
        (b"\x1F\x8B", "gzip"),
        (b"7z\xBC\xAF\x27\x1C", "7-Zip"),
        (b"Rar!\x1A\x07", "RAR"),
        (b"II*\x00", "TIFF"),
        (b"MM\x00*", "TIFF"),
    ];
    if b.starts_with(&fractadyne_core::IMAGINA_BINARY_MAGIC) {
        return Some("Imagina .im");
    }
    if b.len() >= 12 && &b[..4] == b"RIFF" && &b[8..12] == b"WEBP" {
        return Some("WebP");
    }
    SIGS.iter().find(|(sig, _)| b.starts_with(sig)).map(|&(_, name)| name)
}

/// The weaker signatures, named only when the file is refused for its zero bytes anyway: two
/// letters a text file could begin with.
fn sniff_weak(b: &[u8]) -> Option<&'static str> {
    if b.starts_with(b"MZ") {
        Some("EXE")
    } else if b.starts_with(b"BM") {
        Some("BMP")
    } else {
        None
    }
}

/// UTF-16 without a byte-order mark: text in a Latin script has a zero in every high byte, so one
/// half of the bytes is mostly zero and the other half almost never. `Some(true)` = little-endian.
fn utf16_without_bom(b: &[u8]) -> Option<bool> {
    let pairs = b.len() / 2;
    if pairs < 2 {
        return None;
    }
    let zeros = |off: usize| b.chunks_exact(2).filter(|p| p[off] == 0).count();
    let (low, high) = (zeros(0), zeros(1)); // little-endian: low byte first
    if high * 10 >= pairs * 7 && low * 20 <= pairs {
        Some(true)
    } else if low * 10 >= pairs * 7 && high * 20 <= pairs {
        Some(false)
    } else {
        None
    }
}

/// Windows-1252: Latin-1 with printable characters in 0x80–0x9F (the five undefined bytes stay C1
/// controls, which are then removed as controls).
fn windows_1252(b: &[u8]) -> String {
    const HIGH: [u16; 32] = [
        0x20AC, 0x0081, 0x201A, 0x0192, 0x201E, 0x2026, 0x2020, 0x2021, 0x02C6, 0x2030, 0x0160, 0x2039, 0x0152,
        0x008D, 0x017D, 0x008F, 0x0090, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2013, 0x2014, 0x02DC, 0x2122,
        0x0161, 0x203A, 0x0153, 0x009D, 0x017E, 0x0178,
    ];
    b.iter()
        .map(|&c| match c {
            0x80..=0x9F => char::from_u32(u32::from(HIGH[usize::from(c - 0x80)])).unwrap_or('\u{FFFD}'),
            _ => char::from(c),
        })
        .collect()
}

/// Bytes → location text and how it was read (see the module doc for the rules).
pub(crate) fn decode_location_bytes(bytes: &[u8]) -> Result<Decoded, TxtError> {
    if let Some(name) = sniff_binary(bytes) {
        return Err(TxtError::Binary { offset: 0, looks_like: Some(name) });
    }
    // (encoding, text, unit size in bytes, the mark's length): the last two place a zero character.
    let units16 = |b: &[u8], le: bool| -> Vec<u16> {
        b.chunks_exact(2).map(|p| if le { u16::from_le_bytes([p[0], p[1]]) } else { u16::from_be_bytes([p[0], p[1]]) }).collect()
    };
    let utf32 = |b: &[u8], le: bool| -> String {
        b.chunks_exact(4)
            .map(|q| {
                let u = if le { u32::from_le_bytes([q[0], q[1], q[2], q[3]]) } else { u32::from_be_bytes([q[0], q[1], q[2], q[3]]) };
                char::from_u32(u).unwrap_or('\u{FFFD}')
            })
            .collect()
    };
    let (encoding, text, unit, mark): (&'static str, String, usize, usize) = match bytes {
        [0xFF, 0xFE, 0, 0, rest @ ..] => ("UTF-32 LE + BOM", utf32(rest, true), 4, 4),
        [0, 0, 0xFE, 0xFF, rest @ ..] => ("UTF-32 BE + BOM", utf32(rest, false), 4, 4),
        [0xEF, 0xBB, 0xBF, rest @ ..] => match std::str::from_utf8(rest) {
            Ok(s) => ("UTF-8 + BOM", s.to_owned(), 1, 3),
            Err(_) => ("UTF-8 + BOM", String::from_utf8_lossy(rest).into_owned(), 1, 3),
        },
        [0xFF, 0xFE, rest @ ..] => ("UTF-16 LE + BOM", String::from_utf16_lossy(&units16(rest, true)), 2, 2),
        [0xFE, 0xFF, rest @ ..] => ("UTF-16 BE + BOM", String::from_utf16_lossy(&units16(rest, false)), 2, 2),
        _ => match utf16_without_bom(bytes) {
            Some(true) => ("UTF-16 LE", String::from_utf16_lossy(&units16(bytes, true)), 2, 0),
            Some(false) => ("UTF-16 BE", String::from_utf16_lossy(&units16(bytes, false)), 2, 0),
            None => match std::str::from_utf8(bytes) {
                Ok(s) => ("UTF-8", s.to_owned(), 1, 0),
                Err(_) => ("Windows-1252", windows_1252(bytes), 1, 0),
            },
        },
    };
    if let Some(i) = text.chars().position(|c| c == '\0') {
        // Where the zero is in the file: by character for the fixed-width encodings, else the
        // first zero byte (UTF-8 and Windows-1252 write the zero character as one zero byte).
        let offset = if unit == 1 { bytes.iter().position(|&b| b == 0).unwrap_or(0) } else { mark + unit * i };
        return Err(TxtError::Binary { offset, looks_like: sniff_weak(bytes) });
    }
    let (mut crlf, mut lf, mut cr, mut removed) = (0, 0, 0, 0);
    let mut out = String::with_capacity(text.len());
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '\r' if it.peek() == Some(&'\n') => {
                it.next();
                crlf += 1;
                out.push_str("\r\n");
            }
            '\r' => {
                cr += 1;
                out.push('\r');
            }
            '\n' => {
                lf += 1;
                out.push('\n');
            }
            '\t' => out.push('\t'),
            c if c.is_control() => removed += 1,
            c => out.push(c),
        }
    }
    Ok(Decoded { text: out, info: TextInfo { encoding, crlf, lf, cr, removed } })
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

/// A program whose location format the text parses as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TxtKind {
    Fractadyne,
    KallesFraktaler,
}

/// What the `.fdn` gate makes of the text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FdnFinding {
    Yes,
    Empty,
    TooLarge { size: usize, limit: usize },
    /// Neither an `app=Fractadyne` line nor a `center_re=` line.
    NoKeys,
}

/// What the `.kfr` parser makes of the text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum KfrFinding {
    Yes,
    /// The `Re` / `Im` / `Zoom` lines that are not there.
    Missing(Vec<&'static str>),
    /// A line that is there but does not read: the key, and the value as shown (shortened).
    Unreadable { key: &'static str, value: String },
    /// All three read here, yet the parser's bounds refuse it (a value over 100,000 characters,
    /// or the key past line 20,000).
    Rejected,
}

/// Both gates' answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Findings {
    pub(crate) fdn: FdnFinding,
    pub(crate) kfr: KfrFinding,
}

impl Findings {
    /// The formats the text parses as: none, one, or both.
    pub(crate) fn kinds(&self) -> Vec<TxtKind> {
        let mut k = Vec::new();
        if self.fdn == FdnFinding::Yes {
            k.push(TxtKind::Fractadyne);
        }
        if self.kfr == KfrFinding::Yes {
            k.push(TxtKind::KallesFraktaler);
        }
        k
    }
}

/// A value as a message quotes it: at most 40 characters.
fn quoted(v: &str) -> String {
    let mut s: String = v.chars().take(40).collect();
    if v.chars().count() > 40 {
        s.push('…');
    }
    s
}

/// What each gate makes of `text`, after `fractadyne_text::clean` as their loaders run it.
pub(crate) fn findings(text: &str) -> Findings {
    let cleaned = fractadyne_text::clean(text);
    let t = cleaned.text.as_str();
    let fdn = if t.trim().is_empty() {
        FdnFinding::Empty
    } else if t.trim().len() > crate::SHARE_MAX {
        FdnFinding::TooLarge { size: t.trim().len(), limit: crate::SHARE_MAX }
    } else if crate::location_text_verdict(t).is_ok() {
        FdnFinding::Yes
    } else {
        FdnFinding::NoKeys
    };
    let kfr = if fractadyne_core::parse_kfr(t).is_some() { KfrFinding::Yes } else { kfr_detail(t) };
    Findings { fdn, kfr }
}

/// Why `parse_kfr` refused `t`, read the way it reads: `Key: value` lines, keys without regard to
/// case, the last of each key winning.
fn kfr_detail(t: &str) -> KfrFinding {
    let (mut re, mut im, mut zoom) = (None, None, None);
    for line in t.lines() {
        let Some((key, val)) = line.split_once(':') else { continue };
        let (key, val) = (key.trim(), val.trim());
        if key.eq_ignore_ascii_case("Re") {
            re = Some(val);
        } else if key.eq_ignore_ascii_case("Im") {
            im = Some(val);
        } else if key.eq_ignore_ascii_case("Zoom") {
            zoom = Some(val);
        }
    }
    let missing: Vec<&'static str> =
        [("Re", re.is_none()), ("Im", im.is_none()), ("Zoom", zoom.is_none())].into_iter().filter(|p| p.1).map(|p| p.0).collect();
    if !missing.is_empty() {
        return KfrFinding::Missing(missing);
    }
    for (key, v) in [("Re", re.unwrap()), ("Im", im.unwrap())] {
        if fractadyne_core::parse_bf(v).is_none() {
            return KfrFinding::Unreadable { key, value: quoted(v) };
        }
    }
    let z = zoom.unwrap();
    if !z.parse::<f64>().is_ok_and(|f| f > 0.0) {
        return KfrFinding::Unreadable { key: "Zoom", value: quoted(z) };
    }
    KfrFinding::Rejected
}

/// "Read as …": the encoding and the line endings inferred, and the control characters removed.
pub(crate) fn read_as_lines(info: &TextInfo) -> Vec<String> {
    let mut v = vec![match info.endings() {
        Some(e) => tr!(dlg_txt_read_as, encoding = info.encoding, endings = e),
        None => tr!(dlg_txt_read_as_one_line, encoding = info.encoding),
    }];
    if info.removed > 0 {
        v.push(tr!(dlg_txt_removed, count = info.removed));
    }
    v
}

/// The `.fdn` gate's answer as a line.
pub(crate) fn fdn_line(f: &FdnFinding) -> String {
    match f {
        FdnFinding::Yes => tr!(dlg_txt_fdn_yes),
        FdnFinding::Empty => tr!(dlg_txt_fdn_empty),
        FdnFinding::TooLarge { size, limit } => tr!(dlg_txt_fdn_too_large, size = *size, limit = *limit),
        // The line starts are file syntax, passed in so no translation can change them.
        FdnFinding::NoKeys => tr!(dlg_txt_fdn_no, app = "app=Fractadyne", center = "center_re="),
    }
}

/// The `.kfr` parser's answer as a line.
pub(crate) fn kfr_line(f: &KfrFinding) -> String {
    match f {
        KfrFinding::Yes => tr!(dlg_txt_kfr_yes),
        KfrFinding::Missing(keys) => tr!(dlg_txt_kfr_missing, keys = keys.join(", ")),
        KfrFinding::Unreadable { key, value } => tr!(dlg_txt_kfr_unreadable, key = *key, value = value.as_str()),
        KfrFinding::Rejected => tr!(dlg_txt_kfr_rejected),
    }
}

/// A read failure as the message says it: the size and the limit, where the binary data is and
/// what its first bytes match, or the system's error.
pub(crate) fn read_error_message(e: &TxtError, path: &str) -> String {
    match e {
        TxtError::TooLarge { size, limit } => tr!(export_txt_too_large, path = path, size = *size, limit = *limit),
        TxtError::Binary { offset, looks_like } => {
            let mut m = tr!(export_txt_not_text, path = path, offset = *offset);
            if let Some(f) = looks_like {
                m.push(' ');
                m.push_str(&tr!(export_txt_looks_like, format = *f));
            }
            m
        }
        TxtError::Io(e) => tr!(export_read_failed, path = path, error = e.to_string()),
    }
}

#[cfg(test)]
#[path = "../tests/unit/txt_location/tests.rs"]
mod tests;
