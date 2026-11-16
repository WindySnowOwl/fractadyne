use super::*;

fn corpus(name: &str) -> Vec<u8> {
    let p = format!("{}/../../validation/corpus/locations/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read(&p).unwrap_or_else(|e| panic!("{p}: {e}"))
}

fn utf16(s: &str, le: bool) -> Vec<u8> {
    let mut out = if le { vec![0xFF, 0xFE] } else { vec![0xFE, 0xFF] };
    for u in s.encode_utf16() {
        out.extend_from_slice(&if le { u.to_le_bytes() } else { u.to_be_bytes() });
    }
    out
}

fn temp_file(tag: &str, bytes: &[u8]) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("fractadyne-txt-{tag}-{}.txt", std::process::id()));
    std::fs::write(&p, bytes).unwrap();
    p
}

#[test]
fn a_utf8_bom_is_dropped_and_the_text_kept() {
    let t = decode_location_bytes(b"\xEF\xBB\xBFapp=Fractadyne\ncenter_re=-0.75\n").unwrap();
    assert_eq!(t, "app=Fractadyne\ncenter_re=-0.75\n");
}

// Notepad's "Unicode" save is UTF-16 with a BOM: a location saved that way must read the same.
#[test]
fn utf16_with_a_bom_decodes_both_byte_orders() {
    let s = "app=Fractadyne\r\ncenter_re=-0.75\r\nnotes=Grüße 日本\r\n";
    for le in [true, false] {
        assert_eq!(decode_location_bytes(&utf16(s, le)).unwrap(), s, "le={le}");
    }
}

// A renamed image (or any binary) is refused, never handed to a parser as text.
#[test]
fn a_nul_byte_refuses_the_file_as_binary() {
    let png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR";
    assert!(matches!(decode_location_bytes(png), Err(TxtError::Binary)));
    assert!(matches!(decode_location_bytes(b"Re: 0.1\0\nIm: 0.2\n"), Err(TxtError::Binary)));
    // UTF-16 text with a NUL CHARACTER in it is binary too; its zero bytes as encoding are not.
    assert!(decode_location_bytes(&utf16("Re: 0.1\n", true)).is_ok());
    assert!(matches!(decode_location_bytes(&utf16("Re: 0.1\0\n", true)), Err(TxtError::Binary)));
}

// Repair, don't reject: a stray Latin-1 byte in a note must not cost the location.
#[test]
fn invalid_utf8_is_replaced_not_refused() {
    let t = decode_location_bytes(b"notes=caf\xE9\nRe: -0.75\nIm: 0.1\nZoom: 1E10\n").unwrap();
    assert!(t.starts_with("notes=caf\u{FFFD}\n"), "{t:?}");
    assert!(fractadyne_core::parse_kfr(&t).is_some());
}

#[test]
fn control_characters_go_except_tab_and_line_breaks() {
    let t = decode_location_bytes(b"Re:\t-0.75\x07\r\nIm: 0.1\x1b[0m\nZoom: 1E10\x7f\n\x1a").unwrap();
    assert_eq!(t, "Re:\t-0.75\r\nIm: 0.1[0m\nZoom: 1E10\n");
}

#[test]
fn the_read_is_bounded() {
    let p = temp_file("cap", b"0123456789");
    assert_eq!(read_bounded(&p, 10).unwrap(), "0123456789");
    assert!(matches!(read_bounded(&p, 9), Err(TxtError::TooLarge)));
    let _ = std::fs::remove_file(&p);
    assert!(matches!(read_bounded(&p, 9), Err(TxtError::Io(_))), "a missing file is an I/O error");
    assert_eq!(TXT_MAX, 4_000_000, "the .kfr reader's cap");
}

// The validation set's own files, as they are and as an editor or a download might change them:
// each is recognized as exactly its own format.
#[test]
fn the_corpus_files_are_recognized_by_their_content() {
    let fdn = String::from_utf8(corpus("02-seahorse-1e4.fdn")).unwrap();
    let kfr = String::from_utf8(corpus("02-seahorse-1e4.kfr")).unwrap();
    let crlf = |s: &str| s.replace("\r\n", "\n").replace('\n', "\r\n");
    let cr = |s: &str| s.replace("\r\n", "\n").replace('\n', "\r");
    for (what, text, want) in [
        ("fdn", fdn.clone(), TxtKind::Fractadyne),
        ("fdn CRLF", crlf(&fdn), TxtKind::Fractadyne),
        ("fdn CR", cr(&fdn), TxtKind::Fractadyne),
        ("fdn UTF-16", decode_location_bytes(&utf16(&fdn, true)).unwrap(), TxtKind::Fractadyne),
        ("kfr", kfr.clone(), TxtKind::KallesFraktaler),
        ("kfr CRLF", crlf(&kfr), TxtKind::KallesFraktaler),
        ("kfr UTF-16", decode_location_bytes(&utf16(&kfr, true)).unwrap(), TxtKind::KallesFraktaler),
    ] {
        assert_eq!(location_kinds(&text), vec![want], "{what}");
    }
}

// What the Open dialog must ask about: text that is neither, and text that reads as both.
#[test]
fn neither_and_both_are_reported_as_such() {
    assert_eq!(location_kinds("Shopping list:\n- milk\n- eggs\n"), vec![]);
    assert_eq!(location_kinds(""), vec![]);
    let both = "app=Fractadyne\ncenter_re=-0.75\ncenter_im=0.1\nRe: -0.75\nIm: 0.1\nZoom: 1E10\n";
    assert_eq!(location_kinds(both), vec![TxtKind::Fractadyne, TxtKind::KallesFraktaler]);
}

// Clipboard damage a forum post adds (a U+2212 minus, a no-break space) is repaired before the
// formats are told apart, as their own loaders repair it.
#[test]
fn clipboard_damage_does_not_hide_the_format() {
    let kfr = "Re: \u{2212}0.75\nIm:\u{00A0}0.1\nZoom: 1E10\n";
    assert_eq!(location_kinds(kfr), vec![TxtKind::KallesFraktaler]);
}

// A file name with a right-to-left override must not display as another name: the controls go
// from what is shown (only; the file is opened by its real path).
#[test]
fn a_shown_path_has_no_bidirectional_controls() {
    let p = std::path::Path::new("C:\\Downloads\\spiral\u{202E}txt.exe");
    assert_eq!(shown_path(p), "C:\\Downloads\\spiraltxt.exe");
    for c in ['\u{061C}', '\u{200E}', '\u{200F}', '\u{202A}', '\u{202B}', '\u{202C}', '\u{202D}', '\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}'] {
        let name = format!("a{c}b.txt");
        assert_eq!(shown_path(std::path::Path::new(&name)), "ab.txt", "U+{:04X}", c as u32);
    }
    assert_eq!(shown_path(std::path::Path::new("Grüße – 日本.fdn.txt")), "Grüße – 日本.fdn.txt", "other text untouched");
}
