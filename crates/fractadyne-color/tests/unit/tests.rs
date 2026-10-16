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

/// ⭐⭐**A pasted colour must render as the colour that was pasted.** Stops are DISPLAY-referred
/// — the shader writes them straight into a non-sRGB framebuffer — so `#808080` is the stop
/// 128/255, NOT its linear decode.
///
/// This test previously asserted the opposite (0.2159, the sRGB→linear value) and so pinned a
/// real bug in place: measured end to end through `--render`, a uniform palette at 0.2159
/// rendered as **#373737**, while the control at 0.502 rendered as **#808080**. Every imported
/// palette was one sRGB decode too dark; the presets escaped it only because they were authored
/// by eye against the live view. See `srgb8_to_stop` and `design/palette-import.md`.
#[test]
fn palette_text_is_display_referred_not_linear() {
    let c = parse_palette_text("#808080").unwrap();
    assert!((c[0][0] - 128.0 / 255.0).abs() < 1e-6, "got {}", c[0][0]);
    // The value that used to be produced, named so the regression is unmistakable.
    assert!((c[0][0] - 0.2159).abs() > 0.2, "regressed to the linear decode: {}", c[0][0]);
    // A 0-255 triple must land on exactly the same stop as the equivalent hex.
    assert_eq!(parse_palette_text("128 128 128").unwrap()[0], c[0]);
}

/// ⭐⭐**A `.map` triple must not be read as three hex shorthands.** `168 168 168` is a real line
/// in Fractint's `default.map`, and each token is also three valid hex digits — so the
/// "hex tokens win" rule turned it into THREE `#114488` colours. Every `.map` line whose values
/// all land in 100–255 was silently mis-imported, in the format this parser advertises support
/// for. These are the exact greys and whites out of `default.map`.
#[test]
fn map_triples_beat_bare_hex_shorthand() {
    for (line, want) in [
        ("168 168 168", 168.0 / 255.0),
        ("128 128 128", 128.0 / 255.0),
        ("252 252 252", 252.0 / 255.0), // Fractint's white: 6-bit 63 x4, not 255
    ] {
        let c = parse_palette_text(line).unwrap();
        assert_eq!(c.len(), 1, "{line:?} produced {} colours, not one", c.len());
        for ch in 0..3 {
            assert!((c[0][ch] - want).abs() < 1e-6, "{line:?} ch{ch} = {}", c[0][ch]);
        }
    }
    // A whole run of .map lines still yields one colour per line.
    let m = parse_palette_text("168 168 168\n84 84 252\n252 252 252\n").unwrap();
    assert_eq!(m.len(), 3);
    // …while shorthand that CANNOT be a decimal triple keeps the hex reading.
    assert_eq!(parse_palette_text("f80").unwrap()[0], parse_palette_text("#ff8800").unwrap()[0]);
    assert_eq!(parse_palette_text("128").unwrap().len(), 1); // lone token: still hex shorthand
    assert!((parse_palette_text("128").unwrap()[0][0] - 17.0 / 255.0).abs() < 1e-6);
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

// ---------------------------------------------------------------- line-ending hygiene

/// ⭐⭐**A palette file must parse the same whatever wrote its line endings.** `str::lines()` does
/// not treat a lone `\r` as a terminator, so before the hygiene layer a classic-Mac `.map` — and
/// there are plenty in the archives these palettes come from — arrived as ONE line and parsed as
/// zero colours, reported as "no colours found" for a file that was completely fine.
#[test]
fn every_importer_handles_cr_crlf_and_lf() {
    let map_unix = "  0   0   0\n255 128  64\n 12  34  56\n";
    let ggr_unix = "GIMP Gradient\nName: t\n1\n0.0 0.5 1.0 0 0 0 1 1 1 1 1 0 0\n";

    // ⚠Typed as fn pointers: an array of closures with different bodies is an array of different
    // TYPES, which is a compile error rather than the obvious-looking table it reads as.
    let cases: [(&str, fn(&str) -> String); 4] = [
        ("unix", |s| s.to_string()),
        ("dos", |s| s.replace('\n', "\r\n")),
        ("classic mac", |s| s.replace('\n', "\r")),
        ("mixed", |s| s.replacen('\n', "\r\n", 1)),
    ];
    for (name, transform) in cases {
        let m = crate::import::parse_map(&transform(map_unix))
            .unwrap_or_else(|e| panic!("{name} .map failed: {e}"));
        assert_eq!(m.colors.len(), 3, "{name} .map lost lines");

        let g = crate::import::parse_ggr(&transform(ggr_unix))
            .unwrap_or_else(|e| panic!("{name} .ggr failed: {e}"));
        assert_eq!(g.segments.len(), 1, "{name} .ggr lost segments");

        let p = crate::parse_palette_text(&transform(map_unix))
            .unwrap_or_else(|e| panic!("{name} palette text failed: {e}"));
        assert_eq!(p.len(), 3, "{name} palette text lost lines");
    }
}

/// ⚠And the paste repairs reach the importers too — a palette copied out of a web page arrives
/// with the hyphens and quotes a browser or word processor substituted.
#[test]
fn a_pasted_palette_survives_smart_punctuation() {
    // A no-break space between columns is what a copy out of an HTML table looks like.
    let pasted = "\u{FEFF}  0\u{00A0}  0   0\n255 128  64\n";
    let m = crate::import::parse_map(pasted).expect("a pasted .map must still parse");
    assert_eq!(m.colors.len(), 2);
    assert_eq!(m.colors[1], [1.0, 128.0 / 255.0, 64.0 / 255.0]);
}
