use super::*;
use crate::segment::LUT_SIZE;

/// The first sixteen lines of Fractint's `default.map` — the EGA/VGA text colours. Real data, not
/// a synthesised fixture: `design/palette-import.md` records that entries 0–15 are a system
/// palette that happens to live in the file, and these are the exact values.
const DEFAULT_MAP_HEAD: &str = "\
0 0 0
0 0 168
0 168 0
0 168 168
168 0 0
168 0 168
168 84 0
168 168 168
84 84 84
84 84 252
84 252 84
84 252 252
252 84 84
252 84 252
252 252 84
252 252 252
";

/// A `.map` reads one colour per line, in file order, with the values it carries.
#[test]
fn reads_one_color_per_line() {
    let m = parse_map(DEFAULT_MAP_HEAD).unwrap();
    assert_eq!(m.colors.len(), 16);
    assert_eq!(m.colors[0], [0.0, 0.0, 0.0]);
    // ⭐168 168 168 is the line that used to be read as three `#114488` hex shorthands by the
    // tolerant paste parser (fixed in beta.21). A dedicated .map parser cannot make that mistake
    // at all, which is half the reason it exists.
    for ch in 0..3 {
        assert!((m.colors[7][ch] - 168.0 / 255.0).abs() < 1e-6);
    }
    assert!((m.colors[15][0] - 252.0 / 255.0).abs() < 1e-6);
}

/// ⭐Fractint's values are 6-bit VGA written out ×4, so its white is **252, not 255**. That is
/// detected and reported — and deliberately left alone, because Fractint's own images carry the
/// same 252 and rescaling would fail every comparison against them by a few percent.
#[test]
fn detects_the_six_bit_vga_table_without_rescaling() {
    let m = parse_map(DEFAULT_MAP_HEAD).unwrap();
    assert!(m.vga_6bit, "all multiples of 4 with a 252 maximum is the 6-bit signature");
    assert!(
        (m.colors[15][0] - 252.0 / 255.0).abs() < 1e-6,
        "white was rescaled to 255 - a Fractint render would no longer match",
    );
    // A table that reaches 255 is an 8-bit file (KF writes these) and must not be flagged.
    assert!(!parse_map("0 0 0\n255 255 255\n").unwrap().vga_6bit);
    // Multiples of 4 that never reach 252 are not the signature either.
    assert!(!parse_map("0 0 0\n4 8 12\n").unwrap().vga_6bit);
}

/// Comments, blank lines and trailing colour names are all part of real files.
#[test]
fn tolerates_what_real_files_carry() {
    let m = parse_map("; a comment header\n\n255 0 0 red\n0 255 0   green\n\n0 0 255 ; blue\n")
        .unwrap();
    assert_eq!(m.colors.len(), 3);
    assert_eq!(m.colors[0], [1.0, 0.0, 0.0]);
    assert_eq!(m.colors[2], [0.0, 0.0, 1.0]);
}

/// A `.map` is a 256-entry table; a longer file is truncated rather than silently producing a
/// palette the source application could not have shown.
#[test]
fn stops_at_256_entries() {
    let long: String = (0..300).map(|i| format!("{} 0 0\n", i % 256)).collect();
    assert_eq!(parse_map(&long).unwrap().colors.len(), 256);
}

/// ⚠A named `.map` has declared its format, so a bad line is an error with a line number — not
/// something to guess around the way the paste box does.
#[test]
fn rejects_what_is_not_a_map() {
    for (text, want) in [
        ("", "found 0 colours"),
        ("255 0 0\n", "found 1 colour"),
        ("255 0\n0 0 0\n", "line 1"),
        ("255 0 0\n300 0 0\n", "line 2"),
        ("#ff0000\n#00ff00\n", "line 1"),
    ] {
        let e = parse_map(text).unwrap_err();
        assert!(e.contains(want), "{text:?} gave {e:?}, expected it to mention {want:?}");
    }
}

/// ⭐⭐**The case the whole design exists for**: a `.map` imported as bands keeps its hard steps
/// all the way through the bake the GPU fetches from. A smooth import of the same file must NOT —
/// that is the user's choice, and the two must be visibly different.
#[test]
fn bands_survive_to_the_lut_and_smoothing_is_a_real_alternative() {
    let m = parse_map(DEFAULT_MAP_HEAD).unwrap();

    let banded = m.bands("default");
    assert!(banded.is_flat());
    let lut = banded.bake(LUT_SIZE);
    assert!(!lut.smooth, "a banded .map must nearest-fetch, or the bands blur");
    let mut seen: Vec<u32> = (0..4096)
        .map(|i| (lut.sample(i as f32 / 4096.0)[0] * 255.0).round() as u32)
        .collect();
    seen.sort_unstable();
    seen.dedup();
    // The 16 head entries carry 4 distinct red values (0, 84, 168, 252) and no others: sampling
    // the LUT densely must never produce an in-between value.
    assert_eq!(seen, vec![0, 84, 168, 252], "the bands were interpolated away");

    let smoothed = m.smooth("default");
    assert!(!smoothed.is_flat());
    let slut = smoothed.bake(LUT_SIZE);
    assert!(slut.smooth);
    let distinct = {
        let mut v: Vec<u32> = (0..4096)
            .map(|i| (slut.sample(i as f32 / 4096.0)[0] * 255.0).round() as u32)
            .collect();
        v.sort_unstable();
        v.dedup();
        v.len()
    };
    assert!(distinct > 50, "smoothing produced only {distinct} distinct values - it did not blend");
}

/// The band edges land where the file says they do. With 16 entries baked into 1024, each band is
/// exactly 64 entries wide; an off-by-half-a-band bake would shift the whole palette.
#[test]
fn band_edges_land_on_the_file_entries() {
    let m = parse_map(DEFAULT_MAP_HEAD).unwrap();
    let lut = m.bands("default").bake(LUT_SIZE);
    let per = LUT_SIZE / 16;
    for (band, c) in m.colors.iter().enumerate() {
        for k in [0usize, per / 2, per - 1] {
            let e = lut.entries[band * per + k];
            assert_eq!([e[0], e[1], e[2]], *c, "band {band}, entry {k} of {per}");
        }
    }
}

// ================================================================================================
// Ultra Fractal `.ugr`
// ================================================================================================

/// Shaped after the real `blatte1.ugr` recorded in `design/palette-import.md` §2 — the free-wrapped
/// `index=`/`color=` run, the `title=`/`smooth=` prefix, and the separate `opacity:` section.
const UGR_SAMPLE: &str = "\
; a header comment UF files carry
blatte10 {
gradient:
  title=\"blatte10\" smooth=no index=0 color=3085069 index=25 color=3216141
  index=56 color=10761236
  index=399 color=144
opacity:
  smooth=no index=0 opacity=255
}
second {
gradient:
  title=\"the other one\" smooth=yes rotation=100 index=0 color=255 index=399 color=16711680
}
";

/// A `.ugr` holds MANY gradients and an importer must offer all of them — loading "the" gradient
/// would silently pick one of dozens.
#[test]
fn ugr_returns_every_gradient_in_the_file() {
    let gs = parse_ugr(UGR_SAMPLE).unwrap();
    assert_eq!(gs.len(), 2);
    assert_eq!(gs[0].name, "blatte10");
    assert_eq!(gs[0].title.as_deref(), Some("blatte10"));
    assert_eq!(gs[1].name, "second");
    assert_eq!(gs[1].title.as_deref(), Some("the other one"));
    // The index/colour run pairs up across line breaks — whitespace carries no meaning in a block.
    assert_eq!(gs[0].stops.len(), 4);
    assert_eq!(gs[0].stops.iter().map(|(i, _)| *i).collect::<Vec<_>>(), vec![0, 25, 56, 399]);
}

/// ⭐⭐**`color=` is packed BGR — red is the LOW byte.** Reading it as RGB swaps red and blue on
/// every imported gradient and still looks plausible, which is how that bug survives review. These
/// are the exact integers from the surveyed file plus two unambiguous controls.
#[test]
fn ugr_color_is_bgr_not_rgb() {
    let gs = parse_ugr(UGR_SAMPLE).unwrap();
    // color=144 = 0x000090 -> red 144, green 0, blue 0. Under an RGB reading it would be BLUE.
    let last = gs[0].stops[3].1;
    assert!((last[0] - 144.0 / 255.0).abs() < 1e-6, "red should be the low byte, got {last:?}");
    assert_eq!(last[1], 0.0);
    assert_eq!(last[2], 0.0);
    // color=3085069 = 0x2F130D -> (r, g, b) = (0x0D, 0x13, 0x2F): a dark blue-violet. Read as RGB
    // it would be (47, 19, 13), a dark BROWN - both plausible, which is the whole hazard.
    let first = gs[0].stops[0].1;
    for (ch, want) in [(0, 0x0D), (1, 0x13), (2, 0x2F)] {
        assert!(
            (first[ch] - want as f32 / 255.0).abs() < 1e-6,
            "channel {ch}: {first:?} does not decode 3085069 as BGR"
        );
    }
    // Controls: 255 = 0x0000FF is pure RED here, 16711680 = 0xFF0000 is pure BLUE.
    let g2 = &parse_ugr(UGR_SAMPLE).unwrap()[1];
    assert_eq!(g2.stops[0].1, [1.0, 0.0, 0.0]);
    assert_eq!(g2.stops[1].1, [0.0, 0.0, 1.0]);
}

/// ⚠We divide by 255, not gnofract4d's 256 — so a full byte reaches pure white instead of
/// stopping a step short. Stated in `parse_ugr`; pinned here so the choice is deliberate.
#[test]
fn ugr_divides_by_255_so_full_is_white() {
    // 16777215 = 0xFFFFFF.
    let g = &parse_ugr("w {\ngradient:\nindex=0 color=16777215 index=399 color=0\n}\n").unwrap()[0];
    assert_eq!(g.stops[0].1, [1.0, 1.0, 1.0]);
}

/// ⭐Indices run 0–399, so position is `index / 399` — not /255 and not already-normalised.
#[test]
fn ugr_index_range_is_0_to_399() {
    assert_eq!(UGR_INDEX_MAX, 399);
    let g = parse_ugr(UGR_SAMPLE).unwrap()[0].to_gradient();
    // index 0 and index 399 are the ends, so the gradient covers 0..1 with no flat clamps.
    assert_eq!(g.segments.first().unwrap().left, 0.0);
    assert_eq!(g.segments.last().unwrap().right, 1.0);
    // index 25 of 399 lands at 0.0627, and the colour there is that stop's colour.
    let at = g.eval(25.0 / 399.0);
    let want = parse_ugr(UGR_SAMPLE).unwrap()[0].stops[1].1;
    for ch in 0..3 {
        assert!((at[ch] - want[ch]).abs() < 1e-5, "index 25 landed wrong: {at:?} vs {want:?}");
    }
}

/// ⚠`rotation=` is applied, with the direction PINNED here because it is not verified against
/// Ultra Fractal itself. Ignoring a stated field is wrong for certain; applying it with a possibly
/// wrong sign is recoverable, and this test makes the correction one sign change.
#[test]
fn ugr_rotation_shifts_the_ring() {
    let gs = parse_ugr(UGR_SAMPLE).unwrap();
    let g2 = &gs[1];
    assert_eq!(g2.rotation, 100);
    let plain = Gradient::from_stops("x", &[(0.0, [1.0, 0.0, 0.0]), (1.0, [0.0, 0.0, 1.0])]);
    let rotated = g2.to_gradient();
    // Unrotated this gradient runs red -> blue. rotation=100 of 400 moves it a quarter forward, so
    // the red end now sits at 0.25. Probe just PAST the seam, not on it: red -> blue is not a
    // seamless palette, so rotating puts a genuine hard jump exactly at 0.25 and sampling there
    // reads whichever side wins the tie - a test that would be measuring the tie-break, not the
    // rotation.
    assert!(plain.eval(0.0)[0] > 0.9 && plain.eval(1.0)[2] > 0.9, "control: red -> blue");
    let just_past = rotated.eval(0.26);
    assert!(just_past[0] > 0.9 && just_past[2] < 0.1, "expected red just past 0.25, got {just_past:?}");
    let just_before = rotated.eval(0.24);
    assert!(just_before[2] > 0.9 && just_before[0] < 0.1, "expected blue before 0.25, got {just_before:?}");
    // A rotation must not lose or invent colour: still covers 0..1 contiguously.
    assert_eq!(rotated.segments.first().unwrap().left, 0.0);
    assert_eq!(rotated.segments.last().unwrap().right, 1.0);
    for w in rotated.segments.windows(2) {
        assert!((w[0].right - w[1].left).abs() < 1e-6, "rotation left a gap");
    }
}

/// The `opacity:` section is parsed and kept rather than discarded, and its `index=`/`smooth=`
/// must not leak into the gradient's own.
#[test]
fn ugr_opacity_section_is_separate() {
    let gs = parse_ugr(UGR_SAMPLE).unwrap();
    assert_eq!(gs[0].opacity, vec![(0, 1.0)]);
    assert!(!gs[0].smooth, "smooth=no on the gradient");
    assert!(gs[1].smooth, "smooth=yes on the gradient");
    // The opacity section's stops did not become colours.
    assert_eq!(gs[0].stops.len(), 4);
}

/// Blocks that are not gradients (a .ugr can sit beside formula/parameter blocks in the same
/// syntax) are skipped, not treated as an error or as an empty palette.
#[test]
fn ugr_skips_non_gradient_blocks() {
    let text = "notagradient {\n  something=1\n}\nreal {\ngradient:\nindex=0 color=0 index=399 color=255\n}\n";
    let gs = parse_ugr(text).unwrap();
    assert_eq!(gs.len(), 1);
    assert_eq!(gs[0].name, "real");
}

/// A file with no gradients at all is an error with a reason, not a silent empty import.
#[test]
fn ugr_rejects_a_file_with_no_gradients() {
    assert!(parse_ugr("").is_err());
    assert!(parse_ugr("just some text\n").is_err());
    assert!(parse_ugr("empty {\n}\n").is_err());
    // A colour with no index before it is a malformed block, and says so.
    let e = parse_ugr("b {\ngradient:\ncolor=255 index=0 color=0\n}\n").unwrap_err();
    assert!(e.contains("no index="), "got {e:?}");
}
