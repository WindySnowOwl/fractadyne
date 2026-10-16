//! The gallery's zoom label, pinned against the renders that exposed it: a `zoom=` field past the
//! range of a double used to read `inf×`, because `str::parse::<f64>` overflows to infinity
//! silently. The values below are the ones actually stored in the user's renders folder.

use crate::{fmt_zoom, gallery_zoom_label};

#[test]
fn a_depth_past_the_range_of_a_double_is_labelled_not_infinite() {
    // Stored verbatim by the writer (`fmt_zoom_field`) in real exports.
    for (field, want) in [
        ("1.584047e1008", "e1008×"),
        ("7.456789e37000", "e37000×"),
        ("1.2e500", "e500×"),
    ] {
        let got = gallery_zoom_label(field);
        assert!(!got.contains("inf"), "{field} labelled {got}");
        assert!(got.ends_with(want), "{field} labelled {got}, wanted …{want}");
    }
    // And the mantissa survives, not just the exponent.
    assert!(gallery_zoom_label("1.584047e1008").starts_with("1.58"));
    assert!(gallery_zoom_label("7.456789e37000").starts_with("7.46"));
}

#[test]
fn every_label_that_was_already_right_is_unchanged() {
    // ⭐The log path must not touch a finite value: the old code formatted the double itself, and a
    // round trip through log2 and back can move the last grouped digits. These are real fields from
    // the same folder (home, a 6.6e43 view, a 1.8e111 view) plus the edges of the double's range.
    for field in [
        "1.3333333333528863e0",
        "8.865878062593287e43",
        "1.806374085350e111",
        "13292294",
        "1e300",
        "1.7e308",
    ] {
        let z: f64 = field.parse().unwrap();
        assert_eq!(gallery_zoom_label(field), format!("{}×", fmt_zoom(z)), "{field}");
    }
}

#[test]
fn an_unreadable_field_gives_an_empty_label_as_before() {
    for field in ["", "   ", "deep", "e500", "-1e5", "0"] {
        let got = gallery_zoom_label(field);
        assert!(!got.contains("inf") && !got.contains("NaN"), "{field:?} labelled {got}");
    }
    assert_eq!(gallery_zoom_label(""), "");
    assert_eq!(gallery_zoom_label("deep"), "");
}
