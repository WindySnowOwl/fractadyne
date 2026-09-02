use super::*;

/// Each packed slot is `[r, g, b, pos]` — the order the GPU uniform expects.
#[test]
fn packed_slot_is_rgb_then_pos() {
    let p = Palette { name: "t", stops: &[(0.0, [0.1, 0.2, 0.3]), (1.0, [0.4, 0.5, 0.6])] };
    let (out, n) = p.packed();
    assert_eq!(n, 2);
    assert_eq!(out[0], [0.1, 0.2, 0.3, 0.0]);
    assert_eq!(out[1], [0.4, 0.5, 0.6, 1.0]);
}

/// Unused trailing slots repeat the last real stop (so the shader's `stop_count` bound and the
/// padded array agree — reading past `n` still yields the terminal color, never garbage).
#[test]
fn trailing_slots_repeat_last_stop() {
    let p = Palette { name: "t", stops: &[(0.0, [0.0; 3]), (0.5, [1.0; 3]), (1.0, [0.2; 3])] };
    let (out, n) = p.packed();
    assert_eq!(n, 3);
    assert_eq!(out[2], [0.2, 0.2, 0.2, 1.0]);
    for slot in &out[n as usize..] {
        assert_eq!(*slot, out[2]);
    }
}

/// The shapes people actually paste, all accepted.
#[test]
fn palette_text_accepts_common_shapes() {
    // CSS-ish hex list on one line.
    let c = parse_palette_text("#ff0000, #00ff00, #0000ff").unwrap();
    assert_eq!(c.len(), 3);
    // Pure red in sRGB is pure red in linear; the green/blue channels stay at zero.
    assert!((c[0][0] - 1.0).abs() < 1e-6 && c[0][1] == 0.0 && c[0][2] == 0.0);
    // Bare hex, one per line, mixed case.
    assert_eq!(parse_palette_text("ff0000\n00FF00\n").unwrap().len(), 2);
    // 3-digit shorthand: #f00 == #ff0000.
    assert_eq!(parse_palette_text("#f00").unwrap()[0], c[0]);
    // Fractint / KF .map triples, with a trailing comment line.
    let m = parse_palette_text("255 0 0\n0 255 0\n; a comment\n").unwrap();
    assert_eq!(m.len(), 2);
    assert_eq!(m[0], c[0]);
    // Several triples on one line.
    assert_eq!(parse_palette_text("255 0 0 0 255 0").unwrap().len(), 2);
}

/// sRGB is converted to linear, not copied through. Mid-grey is the cheap tell: 128/255 is
/// ~0.502 in sRGB but ~0.216 in linear, and getting this wrong washes out every import.
#[test]
fn palette_text_converts_srgb_to_linear() {
    let c = parse_palette_text("#808080").unwrap();
    assert!((c[0][0] - 0.2159).abs() < 1e-3, "got {}", c[0][0]);
}

/// Malformed input is rejected with a reason rather than silently half-imported.
#[test]
fn palette_text_rejects_junk() {
    assert!(parse_palette_text("").is_err());
    assert!(parse_palette_text("hello there").is_err());
    assert!(parse_palette_text("300 0 0").is_err()); // out of 0-255 range
    assert!(parse_palette_text("#ff0000 nonsense").is_err()); // half-parsed line
    assert!(parse_palette_text("255 0").is_err()); // incomplete triple
}

/// Down-sampling keeps the ends and spans the middle, so a 256-entry .map keeps its shape
/// instead of importing only its dark end.
#[test]
fn resample_keeps_endpoints() {
    let src: Vec<[f32; 3]> = (0..256).map(|i| [i as f32 / 255.0; 3]).collect();
    let out = resample_colors(&src, MAX_STOPS);
    assert_eq!(out.len(), MAX_STOPS);
    assert_eq!(out[0], src[0]);
    assert_eq!(out[MAX_STOPS - 1], src[255]);
    // Monotonic: evenly spaced samples of a ramp stay a ramp.
    assert!(out.windows(2).all(|w| w[0][0] < w[1][0]));
    // Short lists pass through untouched.
    assert_eq!(resample_colors(&src[..3], MAX_STOPS).len(), 3);
}

/// A single-stop palette fills every slot with that stop (count 1, no out-of-bounds).
#[test]
fn single_stop_fills_all_slots() {
    let p = Palette { name: "t", stops: &[(0.3, [0.7, 0.8, 0.9])] };
    let (out, n) = p.packed();
    assert_eq!(n, 1);
    assert!(out.iter().all(|s| *s == [0.7, 0.8, 0.9, 0.3]));
}

/// More stops than the GPU carries: the count saturates at `MAX_STOPS` and the first
/// `MAX_STOPS` stops are kept (no panic, no overflow).
#[test]
fn count_clamps_to_max_stops() {
    let p = Palette {
        name: "t",
        stops: &[
            (0.0, [0.0; 3]), (0.1, [1.0; 3]), (0.2, [2.0; 3]), (0.3, [3.0; 3]),
            (0.4, [4.0; 3]), (0.5, [5.0; 3]), (0.6, [6.0; 3]), (0.7, [7.0; 3]),
            (0.8, [8.0; 3]), (0.9, [9.0; 3]), (1.0, [10.0; 3]),
        ],
    };
    let (out, n) = p.packed();
    assert_eq!(n as usize, MAX_STOPS);
    assert_eq!(out[MAX_STOPS - 1], [7.0, 7.0, 7.0, 0.7]);
}

/// Every shipped preset packs within bounds, fits in `MAX_STOPS`, and keeps its first stop.
#[test]
fn presets_pack_within_bounds() {
    for p in PRESETS {
        let (out, n) = p.packed();
        assert!((1..=MAX_STOPS as u32).contains(&n), "{}: count {n} out of range", p.name);
        assert_eq!(n as usize, p.stops.len(), "{}: all presets must fit in MAX_STOPS", p.name);
        let (pos, c) = p.stops[0];
        assert_eq!(out[0], [c[0], c[1], c[2], pos], "{}: first slot", p.name);
    }
}
