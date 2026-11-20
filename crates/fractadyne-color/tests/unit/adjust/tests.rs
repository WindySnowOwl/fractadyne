use super::{oklab_to_linear, oklab_to_srgb, srgb_to_oklab, with_lightness, PaletteAdjust};
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

/// The fringe's step (C9): a color that fits the gamut at the new lightness keeps its hue and
/// chroma exactly, at exactly that lightness.
#[test]
fn with_lightness_sets_the_lightness_and_keeps_a_color_that_fits() {
    // A grayish teal, from dark to light: its chroma fits the gamut at every one of these. (A
    // stronger teal does not at 0.3: dark cyans are few, and its chroma gives way there, as the next
    // test asks.)
    let c = [0.45f64, 0.52, 0.52];
    let [_, a0, b0] = srgb_to_oklab(c);
    for l in [0.3, 0.45, 0.6, 0.75] {
        let [l1, a1, b1] = srgb_to_oklab(with_lightness(c, l));
        assert!((l1 - l).abs() < 1e-4, "lightness {l1} for {l}");
        assert!((a1 - a0).abs() < 1e-4 && (b1 - b0).abs() < 1e-4, "hue and chroma moved at {l}: ({a1}, {b1}) for ({a0}, {b0})");
    }
}

/// A vivid color asked to be very light: the chroma gives way, the lightness and the hue do not, and
/// the result is in the gamut. Black and white keep no hue: gray at the asked lightness.
#[test]
fn with_lightness_reduces_the_chroma_to_fit_and_keeps_the_hue() {
    for c in [[1.0f64, 0.0, 0.0], [0.0, 0.3, 1.0], [0.1, 0.9, 0.1]] {
        let [_, a0, b0] = srgb_to_oklab(c);
        for l in [0.12, 0.9, 0.97] {
            let out = with_lightness(c, l);
            let [l1, a1, b1] = srgb_to_oklab(out);
            let lin = oklab_to_linear([l1, a1, b1]);
            assert!(lin.iter().all(|v| (-1e-6..=1.0 + 1e-6).contains(v)), "{c:?} at {l} out of gamut: {lin:?}");
            assert!((l1 - l).abs() < 2e-3, "{c:?} at {l}: lightness {l1}");
            let (c0, c1) = (a0.hypot(b0), a1.hypot(b1));
            assert!(c1 <= c0 + 1e-6, "{c:?} at {l}: chroma grew from {c0} to {c1}");
            if c1 > 0.01 {
                let dh = (b1.atan2(a1) - b0.atan2(a0)).abs();
                assert!(dh.min(std::f64::consts::TAU - dh) < 1e-3, "{c:?} at {l}: hue moved by {dh}");
            }
        }
    }
    for c in [[0.0f64, 0.0, 0.0], [1.0, 1.0, 1.0]] {
        let [l1, a1, b1] = srgb_to_oklab(with_lightness(c, 0.5));
        assert!((l1 - 0.5).abs() < 1e-4 && a1.hypot(b1) < 1e-4, "{c:?}: ({l1}, {a1}, {b1})");
    }
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
