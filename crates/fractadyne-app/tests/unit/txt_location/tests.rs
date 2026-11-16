use super::*;

/// The formats `text` parses as.
fn location_kinds(text: &str) -> Vec<TxtKind> {
    findings(text).kinds()
}

fn corpus(name: &str) -> String {
    let p = format!("{}/../../validation/corpus/locations/{name}", env!("CARGO_MANIFEST_DIR"));
    String::from_utf8(std::fs::read(&p).unwrap_or_else(|e| panic!("{p}: {e}"))).unwrap()
}

/// `s` in an encoding, with or without its byte-order mark.
fn encode(s: &str, enc: &str) -> Vec<u8> {
    let u16s = |le: bool| s.encode_utf16().flat_map(move |u| if le { u.to_le_bytes() } else { u.to_be_bytes() }).collect::<Vec<u8>>();
    let u32s = |le: bool| s.chars().flat_map(move |c| if le { (c as u32).to_le_bytes() } else { (c as u32).to_be_bytes() }).collect::<Vec<u8>>();
    match enc {
        "UTF-8" => s.as_bytes().to_vec(),
        "UTF-8 + BOM" => [&[0xEF, 0xBB, 0xBF][..], s.as_bytes()].concat(),
        "UTF-16 LE + BOM" => [vec![0xFF, 0xFE], u16s(true)].concat(),
        "UTF-16 BE + BOM" => [vec![0xFE, 0xFF], u16s(false)].concat(),
        "UTF-16 LE" => u16s(true),
        "UTF-16 BE" => u16s(false),
        "UTF-32 LE + BOM" => [vec![0xFF, 0xFE, 0, 0], u32s(true)].concat(),
        "UTF-32 BE + BOM" => [vec![0, 0, 0xFE, 0xFF], u32s(false)].concat(),
        other => panic!("{other}"),
    }
}

const ENCODINGS: [&str; 8] =
    ["UTF-8", "UTF-8 + BOM", "UTF-16 LE + BOM", "UTF-16 BE + BOM", "UTF-16 LE", "UTF-16 BE", "UTF-32 LE + BOM", "UTF-32 BE + BOM"];

/// `s` with its line endings replaced: the four a file can arrive with, and a twice-converted one.
fn with_endings(s: &str, e: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n").replace('\n', e)
}

fn temp_file(tag: &str, bytes: &[u8]) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("fractadyne-txt-{tag}-{}.txt", std::process::id()));
    std::fs::write(&p, bytes).unwrap();
    p
}

// Every encoding is recognized as itself and decodes to the same text.
#[test]
fn each_encoding_is_inferred_and_decoded() {
    let s = "app=Fractadyne\r\ncenter_re=-0.75\r\nnotes=Grüße 日本 \u{1F600}\r\n";
    for enc in ENCODINGS {
        let d = decode_location_bytes(&encode(s, enc)).unwrap_or_else(|e| panic!("{enc}: {e:?}"));
        assert_eq!(d.info.encoding, enc);
        assert_eq!(d.text, s, "{enc}");
    }
}

// Not UTF-8: Windows-1252, the encoding of most old Windows text. A note's "café" and its curly
// quotes survive (no replacement character), and the location still reads.
#[test]
fn text_that_is_not_utf8_is_read_as_windows_1252() {
    let d = decode_location_bytes(b"Comment: caf\xE9 \x93ok\x94 \x80\nRe: -0.75\nIm: 0.1\nZoom: 1E10\n").unwrap();
    assert_eq!(d.info.encoding, "Windows-1252");
    assert!(d.text.starts_with("Comment: café “ok” €\n"), "{:?}", d.text);
    assert_eq!(location_kinds(&d.text), vec![TxtKind::KallesFraktaler]);
    // The five bytes Windows-1252 leaves undefined become controls, and are removed and counted.
    let d = decode_location_bytes(b"Re: -0.75\x81\x8D\nIm: 0.1\nZoom: 1E10\n").unwrap();
    assert_eq!((d.text.as_str(), d.info.removed), ("Re: -0.75\nIm: 0.1\nZoom: 1E10\n", 2));
}

// A binary format's signature refuses the file before any decoding, and is named.
#[test]
fn a_binary_signature_is_refused_and_named() {
    for (bytes, name) in [
        (&b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR"[..], "PNG"),
        (b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n", "PDF"),
        (b"PK\x03\x04\x14\0", "ZIP"),
        (b"\x76\x2F\x31\x01\x02\0\0\0", "OpenEXR"),
        (b"\xFF\xD8\xFF\xE0", "JPEG"),
        (&fractadyne_core::IMAGINA_BINARY_MAGIC[..], "Imagina .im"),
    ] {
        match decode_location_bytes(bytes) {
            Err(TxtError::Binary { offset: 0, looks_like: Some(n) }) => assert_eq!(n, name),
            other => panic!("{name}: {other:?}"),
        }
    }
}

// Otherwise a zero character refuses it, and the message can say where: the byte offset.
#[test]
fn a_zero_character_refuses_the_file_and_says_where() {
    let at = |b: &[u8]| match decode_location_bytes(b) {
        Err(TxtError::Binary { offset, looks_like }) => (offset, looks_like),
        other => panic!("{other:?}"),
    };
    assert_eq!(at(b"Re: 0.1\0\nIm: 0.2\n"), (7, None));
    assert_eq!(at(b"MZ\x90\0\x03\0"), (3, Some("EXE")));
    // UTF-16's own zero bytes are encoding; a zero CHARACTER in it is binary: mark + 2 per unit.
    assert!(decode_location_bytes(&encode("Re: 0.1\n", "UTF-16 LE + BOM")).is_ok());
    assert_eq!(at(&encode("Re: 0.1\0\n", "UTF-16 LE + BOM")), (2 + 2 * 7, None));
}

#[test]
fn control_characters_go_except_tab_and_line_breaks_and_are_counted() {
    let d = decode_location_bytes(b"Re:\t-0.75\x07\r\nIm: 0.1\x1b[0m\nZoom: 1E10\x7f\n\x1a").unwrap();
    assert_eq!(d.text, "Re:\t-0.75\r\nIm: 0.1[0m\nZoom: 1E10\n");
    assert_eq!(d.info.removed, 4);
}

// Line endings are counted by kind and left for `clean` to normalize; the window names them.
#[test]
fn line_endings_are_counted_and_named() {
    let info = |b: &[u8]| decode_location_bytes(b).unwrap().info;
    let i = info(b"a\r\nb\r\nc\r\n");
    assert_eq!((i.crlf, i.lf, i.cr, i.endings()), (3, 0, 0, Some("CRLF".into())));
    assert_eq!(info(b"a\nb\n").endings(), Some("LF".into()));
    assert_eq!(info(b"a\rb\r").endings(), Some("CR".into()));
    assert_eq!(info(b"a\r\nb\nc\n").endings(), Some("CRLF 1 + LF 2".into()));
    // Converted twice (CRLF written through a text-mode CR→CRLF): CR + CRLF on every line.
    assert_eq!(info(b"a\r\r\nb\r\r\n").endings(), Some("CRLF 2 + CR 2".into()));
    assert_eq!(info(b"Re: 1").endings(), None);
}

#[test]
fn the_read_is_bounded_and_the_refusal_gives_the_sizes() {
    let p = temp_file("cap", b"0123456789");
    assert_eq!(read_bounded(&p, 10).unwrap().text, "0123456789");
    assert!(matches!(read_bounded(&p, 9), Err(TxtError::TooLarge { size: 10, limit: 9 })));
    let _ = std::fs::remove_file(&p);
    assert!(matches!(read_bounded(&p, 9), Err(TxtError::Io(_))), "a missing file is an I/O error");
    assert_eq!(TXT_MAX, 4_000_000, "the .kfr reader's cap");
}

// The validation set's own files, in every encoding and with every line ending a download, an
// editor or a forum could give them: each is recognized as exactly its own format.
#[test]
fn the_corpus_files_are_recognized_in_every_encoding_and_line_ending() {
    let (fdn, kfr) = (corpus("02-seahorse-1e4.fdn"), corpus("02-seahorse-1e4.kfr"));
    let mut n = 0;
    for (text, want) in [(&fdn, TxtKind::Fractadyne), (&kfr, TxtKind::KallesFraktaler)] {
        for ending in ["\n", "\r\n", "\r", "\r\r\n"] {
            for enc in ENCODINGS {
                let d = decode_location_bytes(&encode(&with_endings(text, ending), enc)).unwrap();
                assert_eq!(location_kinds(&d.text), vec![want], "{want:?} {enc} {ending:?}");
                n += 1;
            }
        }
    }
    assert_eq!(n, 2 * 4 * 8);
}

// What each gate says, so a failure can say why.
#[test]
fn the_findings_say_why_a_text_is_not_a_location() {
    let f = findings("Shopping list:\n- milk\n");
    assert_eq!(f.fdn, FdnFinding::NoKeys);
    assert_eq!(f.kfr, KfrFinding::Missing(vec!["Re", "Im", "Zoom"]));
    assert!(f.kinds().is_empty());
    assert_eq!(findings("  \n").fdn, FdnFinding::Empty);
    assert_eq!(findings("Re: -0.75\nIm: 0.1\nIterations: 500\n").kfr, KfrFinding::Missing(vec!["Zoom"]));
    assert_eq!(
        findings("Re: -0.75\nIm: 0.1\nZoom: 1E10x\n").kfr,
        KfrFinding::Unreadable { key: "Zoom", value: "1E10x".into() }
    );
    assert_eq!(findings("re: abc\nIm: 0.1\nZoom: 1E10\n").kfr, KfrFinding::Unreadable { key: "Re", value: "abc".into() });
    // A long unreadable value is quoted shortened: its first 40 characters and an ellipsis.
    let long = format!("Re: {}\nIm: 0.1\nZoom: 1E10\n", "x".repeat(60));
    assert_eq!(findings(&long).kfr, KfrFinding::Unreadable { key: "Re", value: format!("{}…", "x".repeat(40)) });
    let big = format!("app=Fractadyne\ncenter_re=-0.75\nnotes={}\n", "x".repeat(crate::SHARE_MAX));
    assert!(matches!(findings(&big).fdn, FdnFinding::TooLarge { .. }));
    let both = "app=Fractadyne\ncenter_re=-0.75\ncenter_im=0.1\nRe: -0.75\nIm: 0.1\nZoom: 1E10\n";
    assert_eq!(findings(both).kinds(), vec![TxtKind::Fractadyne, TxtKind::KallesFraktaler]);
}

// Clipboard damage a forum post adds (a U+2212 minus, a no-break space) is repaired before the
// formats are told apart, as their own loaders repair it.
#[test]
fn clipboard_damage_does_not_hide_the_format() {
    let kfr = "Re: \u{2212}0.75\nIm:\u{00A0}0.1\nZoom: 1E10\n";
    assert_eq!(location_kinds(kfr), vec![TxtKind::KallesFraktaler]);
}

// The words the window and the messages show (English).
#[test]
fn the_messages_say_what_was_inferred_and_why() {
    let d = decode_location_bytes(&encode("Re: -0.75\r\r\nIm: 0.1\x07\r\r\n", "UTF-16 LE + BOM")).unwrap();
    let lines = read_as_lines(&d.info);
    assert!(lines[0].contains("UTF-16 LE + BOM") && lines[0].contains("CRLF 2 + CR 2"), "{lines:?}");
    assert!(lines[1].contains('1'), "{lines:?}");
    let one = read_as_lines(&decode_location_bytes(b"Re: 1").unwrap().info);
    assert!(one.len() == 1 && one[0].contains("UTF-8") && one[0].contains("one line"), "{one:?}");
    let kfr = kfr_line(&findings(&d.text).kfr);
    assert!(kfr.contains("Zoom") && kfr.contains("missing"), "{kfr}");
    assert!(fdn_line(&FdnFinding::NoKeys).contains("app=Fractadyne"));
    let bin = read_error_message(&TxtError::Binary { offset: 0, looks_like: Some("PNG") }, "a.txt");
    assert!(bin.contains("a.txt") && bin.contains("binary") && bin.contains("PNG"), "{bin}");
    let big = read_error_message(&TxtError::TooLarge { size: 5_000_000, limit: TXT_MAX }, "a.txt");
    assert!(big.contains("a.txt") && big.contains("limit"), "{big}");
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
