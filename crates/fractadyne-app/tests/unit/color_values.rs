//! Tests for the hex / 0–255 readouts beside a colour swatch.
//!
//! ⭐These are the numbers a user reads off the editor and types back in, so the only property that
//! really matters is that the loop closes: what the field SHOWS must parse back to the colour it
//! was showing. A rounding mismatch here does not crash or look broken — it just means a colour
//! copied out of the editor and pasted back in is a shade different, every time.

use super::{hex_of, parse_hex_rgb, rgb_bytes, stop_color32};

#[test]
fn hex_round_trips_through_the_field_for_every_byte() {
    // ⚠Every 0–255 value, not a handful: the failure mode is a rounding rule that is right in the
    // middle of the range and off by one at the ends, which a spot check walks straight past.
    for v in 0..=255u8 {
        let c = [f32::from(v) / 255.0, 0.0, 1.0];
        let text = hex_of(c);
        let back = parse_hex_rgb(&text).expect("our own output must parse");
        assert_eq!(rgb_bytes(back), rgb_bytes(c), "{text} did not survive the round trip");
        assert_eq!(rgb_bytes(c)[0], v, "byte {v} displayed as {}", rgb_bytes(c)[0]);
    }
}

/// ⚠**Rounding, not truncation.** `(v * 255.0) as u8` reports 0.5 as 127 and 0.999 as 254, so the
/// hex shown differs from the hex that produced the colour and the field appears to eat input.
#[test]
fn the_bytes_are_rounded() {
    assert_eq!(rgb_bytes([0.5, 0.5, 0.5])[0], 128);
    assert_eq!(rgb_bytes([1.0, 1.0, 1.0]), [255, 255, 255]);
    assert_eq!(rgb_bytes([0.0, 0.0, 0.0]), [0, 0, 0]);
    // Out of range clamps rather than wrapping — a Bézier overshoot can hand this a value past 1.
    assert_eq!(rgb_bytes([1.4, -0.3, 0.0]), [255, 0, 0]);
}

/// ⭐Three-digit shorthand expands by DUPLICATION, the CSS rule. Zero-padding instead gives a
/// different colour that still looks plausible — `#fff` would come out mid-grey rather than white.
#[test]
fn short_hex_expands_the_css_way() {
    assert_eq!(parse_hex_rgb("#fff"), parse_hex_rgb("#ffffff"));
    assert_eq!(parse_hex_rgb("#000"), parse_hex_rgb("#000000"));
    assert_eq!(parse_hex_rgb("f80"), parse_hex_rgb("#ff8800"));
    assert_eq!(rgb_bytes(parse_hex_rgb("#fff").unwrap()), [255, 255, 255]);
}

#[test]
fn parsing_accepts_the_forms_a_user_pastes_and_rejects_the_rest() {
    for good in ["#ff8800", "ff8800", "  #FF8800  ", "#f80", "F80"] {
        assert!(parse_hex_rgb(good).is_some(), "{good} should parse");
    }
    // ⚠A partial edit must NOT parse: the field applies on every keystroke, so "#ff88" resolving
    // to something would repaint the gradient with a colour the user is halfway through typing.
    for bad in ["", "#", "#ff", "#ff88", "#ff88000", "#gggggg", "rgb(1,2,3)", "#ff 88 00"] {
        assert!(parse_hex_rgb(bad).is_none(), "{bad} should NOT parse");
    }
}

/// The swatch, the hex string and the bytes must all name the same colour — three renderings of
/// one value, and the point of showing them together is that they agree.
#[test]
fn the_swatch_and_the_readouts_agree() {
    for c in [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.2, 0.6, 0.9], [0.545, 0.102, 0.102]] {
        let [r, g, b] = rgb_bytes(c);
        let sw = stop_color32(c);
        assert_eq!([sw.r(), sw.g(), sw.b()], [r, g, b], "swatch disagrees with the bytes");
        assert_eq!(hex_of(c), format!("#{r:02x}{g:02x}{b:02x}"));
    }
}
