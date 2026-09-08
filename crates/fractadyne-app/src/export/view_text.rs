//! The shareable view document: markers, comments, positional diagnostics and the checksum.
//!
//! ⭐**What this format is actually for.** A location travels by clipboard — a forum post, a chat
//! message, a text file mailed to someone. Everything here exists because that journey is lossy in
//! ways the sender never sees: a selection that starts one line too late, a client that reflows
//! whitespace, a word processor that helpfully replaces a hyphen with a minus sign.

use crate::export::{
    view_checksum_state, view_digest, view_field_pairs, wrap_view_text, ChecksumState,
    VIEW_BEGIN_MARKER, VIEW_END_MARKER,
};

/// A minimal but complete view, as `view_metadata` writes one.
fn bare() -> String {
    "app=Fractadyne\nformat_version=1\nfractal=Mandelbrot\njulia=0\n\
     center_re=-0.743643887037158704752191506114774\n\
     center_im=0.131825904205311970493132056385139\n\
     upp_log2=-9.96578428466208700e1\nmax_iter=60000\nauto_iter=1\n\
     palette=0\ncycle=0.27\noffset=0.1\naa=1\n"
        .to_string()
}

// ---------------------------------------------------------------- markers

/// ⚠⚠**No `=` anywhere in a marker.** A build older than this format splits every line on `=` and
/// reports anything unrecognized as an unknown key; a marker containing one would make every such
/// build complain about a bogus field on a file that is perfectly good for it.
#[test]
fn markers_are_invisible_to_an_older_reader() {
    for m in [VIEW_BEGIN_MARKER, VIEW_END_MARKER] {
        assert!(!m.contains('='), "{m:?} would parse as a key=value line");
        assert!(m.starts_with('#'), "{m:?} must be a comment");
    }
    assert_ne!(VIEW_BEGIN_MARKER, VIEW_END_MARKER);
    // And neither may contain the other's phrase, or the scanner cannot tell them apart.
    assert!(!VIEW_BEGIN_MARKER.contains("END FRACTADYNE VIEW"));
    assert!(!VIEW_END_MARKER.contains("BEGIN FRACTADYNE VIEW"));
}

#[test]
fn a_wrapped_view_carries_guidance_markers_and_a_checksum() {
    let w = wrap_view_text(&bare());
    assert!(w.contains(VIEW_BEGIN_MARKER) && w.contains(VIEW_END_MARKER));
    assert!(w.contains("checksum="), "a wrapped view must carry its digest");
    assert!(
        w.lines().next().unwrap().starts_with('#'),
        "the first line should tell a human what this is"
    );
    // Every original field survives the wrap untouched.
    for (k, v) in view_field_pairs(&bare()) {
        assert!(w.contains(&format!("{k}={v}")), "wrapping lost {k}");
    }
}

// ---------------------------------------------------------------- checksum

#[test]
fn a_wrapped_view_verifies() {
    assert_eq!(view_checksum_state(&wrap_view_text(&bare())), ChecksumState::Match);
}

/// ⭐No checksum is not a failure — it is every file written before the field existed, and every
/// view read out of a PNG chunk.
#[test]
fn a_view_without_a_checksum_is_absent_not_broken() {
    assert_eq!(view_checksum_state(&bare()), ChecksumState::Absent);
}

/// ⭐⭐**The property the whole design rests on.** The digest is over the FIELDS, so everything a
/// clipboard legitimately does to text leaves it alone. If this were a hash of the raw bytes, each
/// of these would report corruption, users would learn the warning is noise, and it would stop
/// being worth showing.
#[test]
fn transport_damage_that_is_not_data_loss_does_not_trip_it() {
    let w = wrap_view_text(&bare());
    let cases: Vec<(&str, String)> = vec![
        ("CRLF endings", w.replace('\n', "\r\n")),
        ("classic-Mac endings", w.replace('\n', "\r")),
        ("no trailing newline", w.trim_end().to_string()),
        ("extra blank lines", w.replace('\n', "\n\n")),
        ("trailing spaces", w.replace('\n', "   \n")),
        ("an added comment", format!("# found on the forum\n{w}")),
        (
            "reordered fields",
            {
                let mut ls: Vec<&str> = w.lines().collect();
                ls.reverse();
                ls.join("\n")
            },
        ),
    ];
    for (name, text) in cases {
        assert_eq!(
            view_checksum_state(&text),
            ChecksumState::Match,
            "{name} must NOT be reported as corruption"
        );
    }
}

/// ⭐⭐And the repair layer runs before the comparison, so damage that COULD be undone is undone
/// and the view verifies — which is the right answer, because the data did survive.
#[test]
fn repairable_paste_damage_still_verifies() {
    let w = wrap_view_text(&bare());
    let mangled = w
        .replace("center_re=-", "center_re=\u{2212}") // a chat client's minus sign
        .replace("max_iter=", "max_iter=\u{200B}"); // an invisible space
    assert_ne!(mangled, w, "the guard: the text really was changed");
    assert_eq!(
        view_checksum_state(&mangled),
        ChecksumState::Match,
        "damage that `clean` can undo must not be reported as corruption"
    );
}

/// ⛔But real data loss must be caught — that is the entire point.
#[test]
fn actual_corruption_is_caught() {
    let w = wrap_view_text(&bare());
    let cases: Vec<(&str, String)> = vec![
        ("a changed digit", w.replace("0.131825904205311", "0.131825904205312")),
        ("a dropped field", w.replace("max_iter=60000\n", "")),
        ("a truncated coordinate", w.replace("114774", "1147")),
        ("an added field", w.replace("aa=1\n", "aa=1\ncycle=9.9\n")),
    ];
    for (name, text) in cases {
        assert!(
            matches!(view_checksum_state(&text), ChecksumState::Mismatch { .. }),
            "{name} slipped past the checksum"
        );
    }
}

/// ⚠The thumbnail is excluded on purpose: someone shortening a paste by deleting the huge `thumb`
/// line still has a view that describes the right place, and calling it corrupt would be wrong.
/// The thumbnail is a PNG and carries its own CRC-32 per chunk.
#[test]
fn stripping_the_thumbnail_does_not_break_the_checksum() {
    let with_thumb = format!("{}thumb=iVBORw0KGgo=\n", bare());
    let w = wrap_view_text(&with_thumb);
    assert_eq!(view_checksum_state(&w), ChecksumState::Match);
    let stripped: String = w.lines().filter(|l| !l.starts_with("thumb=")).collect::<Vec<_>>().join("\n");
    assert!(!stripped.contains("thumb="), "the guard: it really was removed");
    assert_eq!(
        view_checksum_state(&stripped),
        ChecksumState::Match,
        "a view is not corrupt for having lost its picture"
    );
}

/// The digest must not depend on the order fields arrive in, because a paste can reorder nothing
/// but our own writer might.
#[test]
fn the_digest_is_order_independent_but_value_sensitive() {
    let a = view_digest([("center_re", "-0.5"), ("max_iter", "100")].into_iter());
    let b = view_digest([("max_iter", "100"), ("center_re", "-0.5")].into_iter());
    assert_eq!(a, b);
    let c = view_digest([("center_re", "-0.6"), ("max_iter", "100")].into_iter());
    assert_ne!(a, c, "a changed value must change the digest");
    assert_eq!(a.len(), 16, "16 hex digits of FNV-1a 64");
    // ⚠A key/value boundary confusion would let "ab"+"c" collide with "a"+"bc".
    let d = view_digest([("center_r", "e-0.5"), ("max_iter", "100")].into_iter());
    assert_ne!(a, d, "the key/value separator must be part of the digest");
}
