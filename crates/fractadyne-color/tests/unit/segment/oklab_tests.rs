//! "Blend in OKLab" (coloring plan C7): `Gradient::bake_oklab`.
use super::*;
use crate::adjust::srgb_to_oklab;
use crate::PRESETS;

fn lab(c: [f32; 4]) -> [f64; 3] {
    srgb_to_oklab([c[0] as f64, c[1] as f64, c[2] as f64])
}

fn chroma(l: [f64; 3]) -> f64 {
    l[1].hypot(l[2])
}

/// Texel `i`'s position, as `bake` samples it.
fn pos(i: usize, n: usize) -> f32 {
    (i as f32 + 0.5) / n as f32
}

/// The definition, on every preset: inside each blended segment an entry has the lightness of the
/// straight line between the segment's ends in OKLab, its hue where the line has one, and no more
/// chroma than the line (less only where the line leaves the gamut). The sRGB bake, the CONTROL,
/// strays from that lightness, or this would test nothing.
#[test]
fn every_preset_blends_along_the_oklab_line_between_its_stops() {
    let n = LUT_SIZE;
    let (mut checked, mut fitted, mut control_worst) = (0usize, 0usize, 0.0f64);
    for p in PRESETS {
        let g = Gradient::from_stops(p.name, p.stops);
        let (ok, srgb) = (g.bake_oklab(n), g.bake(n));
        for i in 0..n {
            let t = pos(i, n);
            let Some(k) = g.segment_at(t) else { continue };
            let s = &g.segments[k];
            if s.is_flat() {
                assert_eq!(ok.entries[i], srgb.entries[i], "{}: a band is the same in both", p.id);
                continue;
            }
            let (a, b, f) = (lab(s.left_color), lab(s.right_color), s.factor(t) as f64);
            let line = [0, 1, 2].map(|j| a[j] + (b[j] - a[j]) * f);
            let got = lab(ok.entries[i]);
            assert!((got[0] - line[0]).abs() < 2e-3, "{} entry {i}: lightness {} against the line's {}", p.id, got[0], line[0]);
            assert!(chroma(got) <= chroma(line) + 2e-3, "{} entry {i}: more chroma than the line", p.id);
            if chroma(line) > 0.03 && chroma(got) > 0.01 {
                let dh = (got[2].atan2(got[1]) - line[2].atan2(line[1])).abs();
                let dh = dh.min(std::f64::consts::TAU - dh);
                assert!(dh < 0.02, "{} entry {i}: hue moved {dh} rad", p.id);
            }
            fitted += (chroma(got) < chroma(line) - 2e-3) as usize;
            control_worst = control_worst.max((lab(srgb.entries[i])[0] - line[0]).abs());
            checked += 1;
        }
    }
    assert!(checked > 50_000, "blended entries checked: {checked}");
    // The fit ran somewhere (prototype: 39 presets have an entry outside the gamut).
    assert!(fitted > 100, "entries fitted into the gamut: {fitted}");
    assert!(control_worst > 0.08, "the sRGB bake's largest lightness gap from the line: {control_worst}");
}

/// What it is for: between two distant hues a blend of the display values sinks darker in the
/// middle (red to green through olive); in OKLab the middle's lightness is the ends' mean.
#[test]
fn red_to_green_keeps_its_lightness_in_the_middle() {
    let g = Gradient::from_stops("rg", &[(0.0, [1.0, 0.0, 0.0]), (1.0, [0.0, 0.8, 0.0])]);
    let (ok, srgb) = (g.bake_oklab(LUT_SIZE), g.bake(LUT_SIZE));
    let mid = LUT_SIZE / 2;
    let ends = (lab([1.0, 0.0, 0.0, 1.0])[0] + lab([0.0, 0.8, 0.0, 1.0])[0]) / 2.0;
    let (l_ok, l_srgb) = (lab(ok.entries[mid])[0], lab(srgb.entries[mid])[0]);
    assert!((l_ok - ends).abs() < 5e-3, "OKLab middle {l_ok} against the ends' mean {ends}");
    assert!(ends - l_srgb > 0.08, "the sRGB middle {l_srgb} should sink below {ends}");
}

/// An HSV sweep, a band and the alpha are left as `bake` has them; so is a gradient of bands only,
/// fetch mode included.
#[test]
fn hsv_segments_bands_and_alpha_are_left_alone() {
    let hsv = Segment { space: Space::HsvCcw, ..Segment::linear(0.0, 0.4, [1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 0.2]) };
    let band = Segment::flat(0.4, 0.7, [0.2, 0.6, 0.3, 1.0]);
    let rgb = Segment::linear(0.7, 1.0, [0.1, 0.2, 0.9, 0.0], [0.9, 0.8, 0.1, 1.0]);
    let g = Gradient { name: "mix".into(), segments: vec![hsv, band, rgb] };
    let (ok, srgb) = (g.bake_oklab(LUT_SIZE), g.bake(LUT_SIZE));
    let mut blended_differs = false;
    for i in 0..LUT_SIZE {
        let t = pos(i, LUT_SIZE);
        if t < 0.7 {
            assert_eq!(ok.entries[i], srgb.entries[i], "entry {i} (HSV or band)");
        } else {
            assert!((ok.entries[i][3] - srgb.entries[i][3]).abs() < 1e-6, "entry {i}: alpha");
            blended_differs |= (0..3).any(|j| (ok.entries[i][j] - srgb.entries[i][j]).abs() > 0.02);
        }
    }
    assert!(blended_differs, "the RGB segment should blend differently");
    let bands = Gradient::from_bands("bands", &[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
    assert_eq!(bands.bake_oklab(LUT_SIZE), bands.bake(LUT_SIZE));
}

/// A segment's ends are its stops' colors, within the OKLab round trip.
#[test]
fn a_blend_starts_and_ends_at_its_stops() {
    for p in PRESETS.iter().take(40) {
        let g = Gradient::from_stops(p.name, p.stops);
        for s in g.segments.iter().filter(|s| !s.is_flat()) {
            let (a, b) = (lab(s.left_color), lab(s.right_color));
            for (t, want) in [(s.left, s.left_color), (s.right, s.right_color)] {
                let got = s.eval_oklab(t, a, b);
                for j in 0..3 {
                    assert!((got[j] - want[j]).abs() < 1e-4, "{}: {got:?} against the stop {want:?}", p.id);
                }
            }
        }
    }
}

/// A Bézier that overshoots its ends stays a color: lightness in 0..1, channels in 0..1.
#[test]
fn an_overshooting_bezier_stays_in_range() {
    let s = Segment {
        blend: Blend::Bezier([0.2, -0.9, 0.8, 1.9]),
        ..Segment::linear(0.0, 1.0, [0.05, 0.05, 0.4, 1.0], [1.0, 0.95, 0.6, 1.0])
    };
    let g = Gradient { name: "over".into(), segments: vec![s] };
    let over = (0..=64).any(|k| !(0.0..=1.0).contains(&s.factor(k as f32 / 64.0)));
    assert!(over, "the curve should overshoot, or this tests nothing");
    for e in g.bake_oklab(LUT_SIZE).entries {
        assert!(e.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)), "{e:?}");
    }
}
