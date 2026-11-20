use super::{oklab_to_srgb, srgb_to_oklab, PaletteAdjust};
use crate::segment::Lut;

fn grid() -> Vec<[f32; 3]> {
    let steps = [0.0f32, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0];
    let mut v = Vec::new();
    for r in steps {
        for g in steps {
            for b in steps {
                v.push([r, g, b]);
            }
        }
    }
    v
}

fn lab(c: [f32; 3]) -> [f64; 3] {
    srgb_to_oklab([c[0] as f64, c[1] as f64, c[2] as f64])
}

#[test]
fn srgb_to_oklab_and_back_is_the_same_color() {
    for c in grid() {
        let back = oklab_to_srgb(lab(c));
        for k in 0..3 {
            assert!((back[k] - c[k] as f64).abs() < 1e-5, "{c:?} came back as {back:?}");
        }
    }
}

#[test]
fn neutral_leaves_the_table_bit_for_bit() {
    let entries: Vec<[f32; 4]> = grid().into_iter().map(|c| [c[0], c[1], c[2], 1.0]).collect();
    let mut lut = Lut { entries: entries.clone(), smooth: true };
    PaletteAdjust::NEUTRAL.apply(&mut lut);
    assert_eq!(lut.entries, entries);
    assert!(PaletteAdjust::default().is_neutral());
}

#[test]
fn brightness_lifts_the_midtones_and_keeps_black_and_white() {
    let up = PaletteAdjust { brightness: 0.5, ..PaletteAdjust::NEUTRAL };
    let down = PaletteAdjust { brightness: -0.5, ..PaletteAdjust::NEUTRAL };
    let mid = [0.4f32, 0.3, 0.6];
    assert!(lab(up.color(mid))[0] > lab(mid)[0] + 0.05);
    assert!(lab(down.color(mid))[0] < lab(mid)[0] - 0.05);
    for end in [[0.0f32; 3], [1.0f32; 3]] {
        for adj in [up, down] {
            let c = adj.color(end);
            for k in 0..3 {
                assert!((c[k] - end[k]).abs() < 1e-4, "{end:?} -> {c:?}");
            }
        }
    }
}

#[test]
fn contrast_spreads_lightness_from_the_middle_and_keeps_black_and_white() {
    let more = PaletteAdjust { contrast: 0.6, ..PaletteAdjust::NEUTRAL };
    let less = PaletteAdjust { contrast: -0.6, ..PaletteAdjust::NEUTRAL };
    let (dark, light) = ([0.15f32, 0.15, 0.15], [0.8f32, 0.8, 0.8]);
    let spread = |a: PaletteAdjust| lab(a.color(light))[0] - lab(a.color(dark))[0];
    let base = lab(light)[0] - lab(dark)[0];
    assert!(spread(more) > base + 0.05, "{} vs {base}", spread(more));
    assert!(spread(less) < base - 0.05, "{} vs {base}", spread(less));
    for end in [[0.0f32; 3], [1.0f32; 3]] {
        let c = more.color(end);
        assert!((c[0] - end[0]).abs() < 1e-4);
    }
}

#[test]
fn saturation_zero_is_gray_and_more_keeps_the_hue_inside_the_gamut() {
    let gray = PaletteAdjust { saturation: 0.0, ..PaletteAdjust::NEUTRAL };
    let vivid = PaletteAdjust { saturation: 2.0, ..PaletteAdjust::NEUTRAL };
    for c in grid() {
        let g = lab(gray.color(c));
        assert!(g[1].abs() < 1e-3 && g[2].abs() < 1e-3, "{c:?} -> chroma {g:?}");
        let (before, after) = (lab(c), lab(vivid.color(c)));
        let chroma = |v: [f64; 3]| v[1].hypot(v[2]);
        if chroma(before) > 0.02 {
            // The hue held (inside the gamut, chroma given up rather than a channel clipped).
            let turn = (after[2].atan2(after[1]) - before[2].atan2(before[1])).sin().abs();
            assert!(turn < 0.02, "{c:?}: hue turned by {turn}");
            assert!(chroma(after) >= chroma(before) - 1e-3, "{c:?}: chroma fell");
        }
        let out = vivid.color(c);
        assert!(out.iter().all(|v| (0.0..=1.0).contains(v)), "{c:?} -> {out:?}");
    }
}

#[test]
fn values_out_of_range_or_not_numbers_are_held_in_range() {
    let a = PaletteAdjust { brightness: 7.0, contrast: f32::NAN, saturation: -3.0 }.clamped();
    assert_eq!(a, PaletteAdjust { brightness: 1.0, contrast: 0.0, saturation: 0.0 });
}
