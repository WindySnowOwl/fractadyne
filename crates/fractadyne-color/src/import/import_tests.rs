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
