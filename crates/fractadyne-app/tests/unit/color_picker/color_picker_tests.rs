//! Tests for the colour picker's vocabulary and its round trip.
//!
//! ⚠The square and the strip are pointer widgets and rest on the author's eye, as everything
//! drag-driven in this app does. What does not is the naming the user reads and the HSV round trip
//! the two of them ride on.

use super::ROW_LABELS;
use fractadyne_color::segment::{hsv_to_rgb, rgb_to_hsv};

/// ⭐⭐**The whole reason this picker exists.** egui's stock one labels its two numeric modes `U8`
/// and `F` — the internal names for "gamma byte" and "linear float" — which are precise and mean
/// nothing to someone who has not read egui's source. These names have to say what the numbers
/// ARE, and a test is the only thing that stops them drifting back toward jargon.
#[test]
fn the_row_labels_are_in_the_users_vocabulary_not_the_implementations() {
    assert_eq!(ROW_LABELS, ["0–255", "0–1", "Hex"]);
    for l in ROW_LABELS {
        assert!(!l.is_empty());
        // ⛔The specific jargon that was replaced, plus the family it came from.
        for banned in ["U8", "u8", "F32", "f32", "linear", "gamma", "sRGB"] {
            assert!(!l.contains(banned), "{l:?} leaks the implementation's word {banned:?}");
        }
    }
    // ⭐All three are on screen at once — there is no mode to be in the wrong one of.
    assert_eq!(ROW_LABELS.len(), 3);
}

/// The square and the strip both round-trip through HSV, so a colour dragged and left alone has to
/// come back unchanged — otherwise merely opening the picker would nudge the gradient.
#[test]
fn hsv_round_trips_for_every_colour_the_square_can_produce() {
    for hi in 0..24 {
        for si in 0..=8 {
            for vi in 0..=8 {
                let (h, s, v) = (hi as f32 / 24.0, si as f32 / 8.0, vi as f32 / 8.0);
                let rgb = hsv_to_rgb(h, s, v);
                let (h2, s2, v2) = rgb_to_hsv(rgb);
                let back = hsv_to_rgb(h2, s2, v2);
                for c in 0..3 {
                    assert!(
                        (rgb[c] - back[c]).abs() < 1.0e-4,
                        "h{h} s{s} v{v}: {rgb:?} -> {back:?}"
                    );
                }
            }
        }
    }
}

/// ⭐⭐**Why the picker keeps its own hue instead of reading it back off the colour each frame.**
/// A grey has no hue, and `rgb_to_hsv` reports 0 — red — for anything unsaturated. Recomputing
/// every frame would snap the hue cursor to the far left the moment the user dragged the square to
/// the greys, losing the hue they were working in.
#[test]
fn an_unsaturated_colour_has_no_hue_to_read_back() {
    for grey in [0.0_f32, 0.25, 0.5, 1.0] {
        let (h, s, _) = rgb_to_hsv([grey, grey, grey]);
        assert_eq!(s, 0.0, "a grey must report zero saturation");
        assert_eq!(h, 0.0, "and its hue reads as 0 (red) — which is why it must not be trusted");
    }
    // A saturated colour DOES have a real hue, which is the case the picker trusts.
    let (h, s, _) = rgb_to_hsv([0.0, 0.0, 1.0]);
    assert!(s > 0.9);
    assert!((h - 2.0 / 3.0).abs() < 1.0e-3, "pure blue sits two thirds round the wheel, got {h}");
}
