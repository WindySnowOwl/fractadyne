//! What the hygiene layer must guarantee, stated as the failures it exists to prevent.

use super::*;

// ---------------------------------------------------------------- line endings

/// ⭐⭐**The failure this prevents.** `str::lines()` does not treat a lone `\r` as a terminator, so
/// a classic-Mac or badly-transferred file arrives as ONE line — and a `key=value` parser reads the
/// whole file as a single unparseable field, reporting "no keys found" for a file that is fine.
#[test]
fn all_three_line_endings_split_the_same_way() {
    let want = vec!["a=1", "b=2", "c=3"];
    for (name, raw) in [
        ("unix", "a=1\nb=2\nc=3"),
        ("dos", "a=1\r\nb=2\r\nc=3"),
        ("classic mac", "a=1\rb=2\rc=3"),
        // ⚠And the genuinely nasty one: a file edited on two machines.
        ("mixed", "a=1\r\nb=2\rc=3"),
    ] {
        let got: Vec<&str> = numbered_lines(raw).into_iter().map(|(_, l)| l).collect();
        assert_eq!(got, want, "{name} endings did not split correctly");
        let nums: Vec<usize> = numbered_lines(raw).into_iter().map(|(n, _)| n).collect();
        assert_eq!(nums, vec![1, 2, 3], "{name} produced the wrong line NUMBERS");
    }
}

/// ⚠⚠A `\r\n` must count as ONE ending. The obvious `split(['\n','\r'])` makes it two and every
/// line number after the first is wrong — which is worse than no line numbers, because a wrong
/// pointer sends someone to the wrong line with confidence.
#[test]
fn crlf_is_one_ending_not_two() {
    let lines = numbered_lines("first\r\nsecond\r\nthird");
    assert_eq!(lines.len(), 3, "CRLF produced spurious blank lines: {lines:?}");
    assert_eq!(lines[2], (3, "third"));

    let naive: Vec<&str> = "first\r\nsecond".split(['\n', '\r']).collect();
    assert_eq!(naive.len(), 3, "the control: the naive split really does over-split");
}

#[test]
fn clean_normalizes_every_ending_to_newline() {
    let c = clean("a\r\nb\rc\nd");
    assert_eq!(c.text, "a\nb\nc\nd");
    assert!(c.repairs.is_empty(), "line endings are not 'damage' and must not be reported");
}

#[test]
fn empty_and_trailing_newline_are_not_a_phantom_line() {
    assert!(numbered_lines("").is_empty());
    assert_eq!(numbered_lines("only\n").len(), 1);
    assert_eq!(numbered_lines("a\n\nb").len(), 3, "a genuinely blank line is a line");
}

// ---------------------------------------------------------------- paste damage

/// ⭐⭐**The repair that actually matters.** A negative coordinate through a chat client or a word
/// processor comes back with U+2212 MINUS SIGN, which `f64::from_str` and our bignum parser both
/// refuse — and it is visually identical to a hyphen, so the report "invalid number" points at
/// something the user can see nothing wrong with.
#[test]
fn a_unicode_minus_sign_becomes_a_hyphen() {
    let c = clean("center_re=\u{2212}0.743643887037158");
    assert_eq!(c.text, "center_re=-0.743643887037158");
    assert_eq!(c.repairs.len(), 1);
    assert_eq!(c.repairs[0].name, "Unicode minus sign");
    assert_eq!(c.repairs[0].line, 1);
    assert_eq!(c.repairs[0].col, 11, "the column must point AT the offending character");
    assert!(c.text.parse::<f64>().is_err(), "sanity: the whole line is not a number");
    assert!(c.text["center_re=".len()..].parse::<f64>().is_ok(), "the value now parses");
}

#[test]
fn invisible_characters_are_removed() {
    for (ch, name) in [
        ('\u{200B}', "zero-width space"),
        ('\u{FEFF}', "byte-order mark"),
        ('\u{00AD}', "soft hyphen"),
        ('\u{2060}', "word joiner"),
    ] {
        let c = clean(&format!("max_iter={ch}60000"));
        assert_eq!(c.text, "max_iter=60000", "{name} was not removed");
        assert_eq!(c.repairs.len(), 1);
        assert_eq!(c.repairs[0].name, name);
        assert_eq!(c.text["max_iter=".len()..].parse::<u32>(), Ok(60000));
    }
}

/// ⚠A BOM at the very start breaks the FIRST key, which is usually `app=Fractadyne` — so the file
/// is rejected as "not a Fractadyne file" rather than as "has a BOM".
#[test]
fn a_leading_bom_does_not_break_the_first_key() {
    let c = clean("\u{FEFF}app=Fractadyne\nmax_iter=7");
    assert!(c.text.starts_with("app=Fractadyne"));
    assert_eq!(c.repairs.len(), 1);
    assert_eq!(c.repairs[0].line, 1);
    assert_eq!(c.repairs[0].col, 1);
}

#[test]
fn smart_quotes_and_dashes_become_their_ascii_forms() {
    let c = clean("notes=\u{201C}the \u{2018}deep\u{2019} spiral\u{201D} \u{2013} take 2");
    assert_eq!(c.text, "notes=\"the 'deep' spiral\" - take 2");
    assert_eq!(c.repairs.len(), 5);
}

#[test]
fn nbsp_becomes_a_plain_space() {
    let c = clean("notes=deep\u{00A0}spiral");
    assert_eq!(c.text, "notes=deep spiral");
    assert_eq!(c.repairs[0].replaced_with, Some(' '));
}

#[test]
fn full_width_digits_become_digits() {
    let c = clean("max_iter=\u{FF16}\u{FF10}\u{FF10}\u{FF10}\u{FF10}");
    assert_eq!(c.text, "max_iter=60000");
    assert_eq!(c.repairs.len(), 5);
}

/// ⚠Bidi controls can make a line DISPLAY differently from what it parses to. Nothing legitimate
/// puts them in a location, so they are removed and named.
#[test]
fn bidi_controls_are_removed_and_named() {
    let c = clean("center_re=\u{202E}-0.5");
    assert_eq!(c.text, "center_re=-0.5");
    assert_eq!(c.repairs[0].name, "bidirectional control");
}

/// ⛔**Only clipboard damage is repaired.** Folding all non-ASCII would corrupt a note or a
/// gradient name written in another language, which is the user's own text, not damage.
#[test]
fn real_text_in_other_languages_is_left_alone() {
    for s in ["notes=Tiefer Zoom — Übersicht", "notes=深いズーム", "notes=спираль", "notes=café"] {
        let c = clean(s);
        let expect_repairs = s.contains('—') as usize;
        assert_eq!(
            c.repairs.len(),
            expect_repairs,
            "{s:?} should only have its em dash touched, got {:?}",
            c.repairs
        );
        assert!(c.text.contains("notes="));
    }
}

/// Columns are counted in CHARACTERS, not bytes — the number an editor shows.
#[test]
fn columns_are_characters_not_bytes() {
    // "é" is two bytes; the offender sits at character 5 of line 2.
    let c = clean("app=Fractadyne\nnotes=café\u{200B}x");
    assert_eq!(c.repairs.len(), 1);
    assert_eq!(c.repairs[0].line, 2);
    assert_eq!(c.repairs[0].col, 11, "byte counting would say 12 here");
}

#[test]
fn the_summary_collapses_repeats() {
    let c = clean("a=\u{2018}1\u{2019}\nb=\u{2018}2\u{2019}\nc=\u{2018}3\u{2019}");
    let s = c.summary().expect("there were repairs");
    assert!(s.contains("6×"), "six smart quotes should collapse to one entry: {s}");
    assert!(s.contains("line 1"), "and name where the first one was: {s}");
    assert!(clean("a=1").summary().is_none(), "clean text must produce no summary");
}

// ---------------------------------------------------------------- base64

#[test]
fn base64_round_trips_every_byte_value_and_every_length() {
    // Every length mod 3 matters — that is what the padding is for.
    for len in 0..64usize {
        let data: Vec<u8> = (0..len).map(|i| (i * 37 + 11) as u8).collect();
        let enc = base64::encode(&data);
        assert_eq!(base64::decode(&enc).as_deref(), Some(&data[..]), "len {len}");
        assert_eq!(enc.len() % 4, 0, "len {len}: encoding must be a whole number of quanta");
    }
    let all: Vec<u8> = (0..=255u8).collect();
    assert_eq!(base64::decode(&base64::encode(&all)).as_deref(), Some(&all[..]));
}

#[test]
fn base64_matches_the_standard_alphabet() {
    // Known vectors from RFC 4648 — this is the check that we implement base64, not a lookalike.
    assert_eq!(base64::encode(b""), "");
    assert_eq!(base64::encode(b"f"), "Zg==");
    assert_eq!(base64::encode(b"fo"), "Zm8=");
    assert_eq!(base64::encode(b"foo"), "Zm9v");
    assert_eq!(base64::encode(b"foob"), "Zm9vYg==");
    assert_eq!(base64::encode(b"fooba"), "Zm9vYmE=");
    assert_eq!(base64::encode(b"foobar"), "Zm9vYmFy");
    assert_eq!(base64::decode("Zm9vYmFy").as_deref(), Some(&b"foobar"[..]));
}

/// ⚠Corrupt input must be REFUSED, not decoded to nearly-right bytes — a truncated thumbnail
/// should be reported here, not handed to the PNG decoder as a mystery failure.
#[test]
fn corrupt_base64_is_refused() {
    assert!(base64::decode("Zm9v!mFy").is_none(), "a character outside the alphabet");
    assert!(base64::decode("Zm9vYmF").is_none(), "a truncated quantum");
    assert!(base64::decode("Zg===").is_none(), "over-padded");
    assert!(base64::decode("Zg==Zg==").is_none(), "padding is terminal");
    assert!(base64::decode("Zg=Z").is_none(), "data after padding");
}

/// ⭐A value that has been through an editor may have picked up stray whitespace or a line wrap.
#[test]
fn base64_tolerates_whitespace_from_a_paste() {
    let enc = base64::encode(b"the quick brown fox");
    let wrapped = format!("{}\n  {}", &enc[..8], &enc[8..]);
    assert_eq!(base64::decode(&wrapped).as_deref(), Some(&b"the quick brown fox"[..]));
}

/// ⚠⚠The property the `.fdn` format depends on: padding contains `=`, and the reader splits on the
/// FIRST `=`. If that ever changed, every embedded thumbnail would silently truncate.
#[test]
fn padding_survives_a_key_value_split() {
    let enc = base64::encode(b"f"); // "Zg==" — the worst case, two padding characters
    let line = format!("thumb={enc}");
    let (k, v) = line.split_once('=').expect("splits");
    assert_eq!(k, "thumb");
    assert_eq!(v, "Zg==", "the value must keep its padding");
    assert_eq!(base64::decode(v).as_deref(), Some(&b"f"[..]));
}
