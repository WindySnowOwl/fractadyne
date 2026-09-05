use super::*;
use crate::PRESETS;

fn approx(a: [f32; 4], b: [f32; 4], tol: f32, what: &str) {
    for i in 0..4 {
        assert!((a[i] - b[i]).abs() <= tol, "{what}: channel {i}, {a:?} vs {b:?}");
    }
}

/// A two-stop linear gradient is the plain lerp everyone expects, midpoint centred.
#[test]
fn linear_segment_is_a_lerp() {
    let g = Gradient::from_stops("t", &[(0.0, [0.0; 3]), (1.0, [1.0; 3])]);
    assert_eq!(g.segments.len(), 1);
    for (t, want) in [(0.0, 0.0), (0.25, 0.25), (0.5, 0.5), (0.75, 0.75), (1.0, 1.0)] {
        approx(g.eval(t), [want, want, want, 1.0], 1e-6, &format!("t={t}"));
    }
}

/// ⭐The midpoint moves where the blend reaches 50% WITHOUT adding a stop — GIMP's model, and the
/// thing a plain stop list cannot express. A midpoint at 0.25 must put mid-grey a quarter of the
/// way across, and must still be exactly 0.5 there.
#[test]
fn midpoint_shifts_the_halfway_point() {
    let seg = Segment {
        left: 0.0,
        mid: 0.25,
        right: 1.0,
        left_color: [0.0, 0.0, 0.0, 1.0],
        right_color: [1.0, 1.0, 1.0, 1.0],
        blend: Blend::Linear,
        space: Space::Rgb,
    };
    let g = Gradient { name: "t".into(), segments: vec![seg] };
    approx(g.eval(0.25), [0.5, 0.5, 0.5, 1.0], 1e-6, "at the midpoint");
    // Linear on each side of it: half of 0.25 is a quarter of the way to 0.5.
    approx(g.eval(0.125), [0.25, 0.25, 0.25, 1.0], 1e-6, "below");
    approx(g.eval(0.625), [0.75, 0.75, 0.75, 1.0], 1e-6, "above");
}

/// Every blend function runs 0 → 1 end to end, monotonically.
///
/// ⚠**Only three of the five pass through 0.5 at the midpoint.** Linear, curved and sine do by
/// construction; GIMP's two sphere blends deliberately do not — a sphere-increasing segment is
/// `sqrt(1 - (f-1)^2)`, which is 0.866 at the halfway point. Asserting the tidier invariant on all
/// five is what this test did first, and it "caught" correct code.
#[test]
fn every_blend_runs_end_to_end() {
    for blend in [
        Blend::Linear,
        Blend::Curved,
        Blend::Sine,
        Blend::SphereIncreasing,
        Blend::SphereDecreasing,
    ] {
        let seg = Segment {
            left: 0.0,
            mid: 0.5,
            right: 1.0,
            left_color: [0.0; 4],
            right_color: [1.0; 4],
            blend,
            space: Space::Rgb,
        };
        let g = Gradient { name: "t".into(), segments: vec![seg] };
        assert!(g.eval(0.0)[0].abs() < 1e-5, "{blend:?} at 0");
        assert!((g.eval(1.0)[0] - 1.0).abs() < 1e-5, "{blend:?} at 1");
        // Monotonic: none of the five doubles back.
        let mut prev = -1.0;
        for i in 0..=64 {
            let v = g.eval(i as f32 / 64.0)[0];
            assert!(v >= prev - 1e-5, "{blend:?} went backwards at {i}: {v} after {prev}");
            prev = v;
        }
    }
    // The three that are midpoint-symmetric.
    for blend in [Blend::Linear, Blend::Curved, Blend::Sine] {
        let seg = Segment {
            left: 0.0,
            mid: 0.5,
            right: 1.0,
            left_color: [0.0; 4],
            right_color: [1.0; 4],
            blend,
            space: Space::Rgb,
        };
        let g = Gradient { name: "t".into(), segments: vec![seg] };
        assert!((g.eval(0.5)[0] - 0.5).abs() < 1e-5, "{blend:?} at the midpoint");
    }
    // …and the two that are not, pinned so the asymmetry is a decision and not a drift.
    let sphere = |blend| {
        let seg = Segment {
            left: 0.0,
            mid: 0.5,
            right: 1.0,
            left_color: [0.0; 4],
            right_color: [1.0; 4],
            blend,
            space: Space::Rgb,
        };
        Gradient { name: "t".into(), segments: vec![seg] }.eval(0.5)[0]
    };
    assert!((sphere(Blend::SphereIncreasing) - 0.75f32.sqrt()).abs() < 1e-5);
    assert!((sphere(Blend::SphereDecreasing) - (1.0 - 0.75f32.sqrt())).abs() < 1e-5);
}

/// The blends are genuinely different curves, not five names for a lerp — a sphere-increasing
/// segment bulges above the line and a sphere-decreasing one below it.
#[test]
fn blends_are_distinguishable() {
    let at = |blend| {
        let seg = Segment {
            left: 0.0,
            mid: 0.5,
            right: 1.0,
            left_color: [0.0; 4],
            right_color: [1.0; 4],
            blend,
            space: Space::Rgb,
        };
        Gradient { name: "t".into(), segments: vec![seg] }.eval(0.25)[0]
    };
    let lin = at(Blend::Linear);
    assert!((lin - 0.25).abs() < 1e-6);
    assert!(at(Blend::SphereIncreasing) > lin + 0.1, "sphere-increasing should bulge up");
    assert!(at(Blend::SphereDecreasing) < lin - 0.02, "sphere-decreasing should sag");
    assert!(at(Blend::Sine) < lin, "sine eases in");
}

/// ⭐An HSV segment sweeps the LONG way round the hue wheel — the thing RGB interpolation cannot
/// do. Red → red-ish going counter-clockwise must pass through green and blue, so the midpoint is
/// nowhere near red; the RGB reading of the same endpoints would barely move.
#[test]
fn hsv_segments_sweep_the_hue_wheel() {
    let mk = |space| {
        let seg = Segment {
            left: 0.0,
            mid: 0.5,
            right: 1.0,
            left_color: [1.0, 0.0, 0.0, 1.0],       // hue 0
            right_color: [1.0, 0.0, 0.0, 1.0],      // hue 0 again — a full turn
            blend: Blend::Linear,
            space,
        };
        Gradient { name: "t".into(), segments: vec![seg] }
    };
    // Counter-clockwise a third of the way round is hue 1/3 = pure green.
    approx(mk(Space::HsvCcw).eval(1.0 / 3.0), [0.0, 1.0, 0.0, 1.0], 1e-5, "ccw third");
    // Clockwise the same distance goes the other way: hue 2/3 = pure blue.
    approx(mk(Space::HsvCw).eval(1.0 / 3.0), [0.0, 0.0, 1.0, 1.0], 1e-5, "cw third");
    // In RGB those endpoints are identical, so the segment is flat red throughout.
    approx(mk(Space::Rgb).eval(1.0 / 3.0), [1.0, 0.0, 0.0, 1.0], 1e-6, "rgb third");
}

/// Round-tripping RGB through HSV must not move a colour, or every HSV segment endpoint drifts.
#[test]
fn hsv_round_trips() {
    for c in [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.2, 0.6, 0.9],
        [0.5, 0.5, 0.5],
        [0.0, 0.0, 0.0],
        [1.0, 1.0, 1.0],
        [0.98, 0.13, 0.44],
    ] {
        let (h, s, v) = rgb_to_hsv(c);
        let back = hsv_to_rgb(h, s, v);
        for i in 0..3 {
            assert!((back[i] - c[i]).abs() < 1e-5, "{c:?} -> ({h},{s},{v}) -> {back:?}");
        }
    }
}

/// ⭐⭐**A `.map` imports as BANDS.** 256 flat segments must come back as 256 hard steps, and the
/// LUT they bake into must reproduce them exactly — this is the case the whole segment model
/// exists for, and the one an interpolating bake would quietly smooth away.
#[test]
fn map_bands_survive_the_bake() {
    let colors: Vec<[f32; 3]> = (0..256).map(|i| [i as f32 / 255.0; 3]).collect();
    let g = Gradient::from_bands("map", &colors);
    assert!(g.is_flat(), "a band gradient must report itself flat");
    let lut = g.bake(LUT_SIZE);
    assert!(!lut.smooth, "a flat gradient must bake to a nearest-fetch LUT");
    // 1024 / 256 = 4 entries per band, and all four carry the band's colour exactly.
    for band in 0..256usize {
        for k in 0..4usize {
            let e = lut.entries[band * 4 + k];
            assert_eq!(e[0], colors[band][0], "band {band} entry {k}");
        }
    }
    // Sampling the LUT the way the shader does lands on the same 256 values, and only those.
    let mut seen: Vec<f32> = (0..4096).map(|i| lut.sample(i as f32 / 4096.0)[0]).collect();
    seen.sort_by(f32::total_cmp);
    seen.dedup();
    assert_eq!(seen.len(), 256, "the bands smeared: {} distinct values", seen.len());
}

/// ⭐**Acceptance criterion 2 from `design/palette-import.md` §4, as a test**: the LUT's error
/// against the gradient it came from must SHRINK as the LUT grows. If it does not, the difference
/// is a bug in the bake, not quantisation — and re-blessing the goldens would enshrine it.
#[test]
fn bake_error_shrinks_with_lut_size() {
    // Kinks at awkward positions, so stop boundaries do NOT land on entry centres. Seamless ends
    // (both black) because the LUT fetch WRAPS: a non-seamless palette gets a legitimate one-entry
    // ramp across the t = 1 / t = 0 seam, which would dominate this metric and hide the thing it
    // is trying to measure.
    let g = Gradient::from_stops(
        "kinky",
        &[
            (0.0, [0.0, 0.0, 0.0]),
            (0.137, [0.9, 0.1, 0.2]),
            (0.401, [0.1, 0.8, 0.3]),
            (0.638, [0.2, 0.2, 0.95]),
            (1.0, [0.0, 0.0, 0.0]),
        ],
    );
    let err_at = |n: usize| {
        let lut = g.bake(n);
        (0..20_000)
            .map(|i| {
                let t = i as f32 / 20_000.0;
                let (a, b) = (lut.sample(t), g.eval(t));
                (0..3).map(|c| (a[c] - b[c]).abs()).fold(0.0f32, f32::max)
            })
            .fold(0.0f32, f32::max)
    };
    let (e256, e1024, e4096) = (err_at(256), err_at(1024), err_at(4096));
    assert!(e1024 < e256 * 0.5, "1024 did not improve on 256: {e1024} vs {e256}");
    assert!(e4096 < e1024 * 0.5, "4096 did not improve on 1024: {e4096} vs {e1024}");
    // And at the size we ship, the error is already below a display LSB everywhere.
    assert!(e1024 < 1.0 / 255.0, "1024-entry error {e1024} exceeds one 8-bit level");
}

/// A smooth gradient's LUT is not allowed to be flat, and vice versa — the flag is derived, so a
/// mistake here silently picks the wrong fetch mode for every render.
#[test]
fn smooth_flag_follows_the_segments() {
    assert!(Gradient::from_stops("s", &[(0.0, [0.0; 3]), (1.0, [1.0; 3])]).bake(64).smooth);
    assert!(!Gradient::from_bands("b", &[[0.0; 3], [1.0; 3]]).bake(64).smooth);
    // A single-colour palette IS flat — binary/duotone-style palettes must not be smoothed.
    assert!(!Gradient::from_stops("one", &[(0.3, [0.7, 0.8, 0.9])]).bake(64).smooth);
    // Mixed: one band plus one ramp is not flat, so it stays interpolated.
    let mixed = Gradient {
        name: "m".into(),
        segments: vec![
            Segment::flat(0.0, 0.5, [1.0, 0.0, 0.0, 1.0]),
            Segment::linear(0.5, 1.0, [1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]),
        ],
    };
    assert!(!mixed.is_flat());
    assert!(mixed.bake(64).smooth);
}

/// ⚠Stops that do not reach the ends clamp to the nearest endpoint colour. The old shader walk
/// fell back to the FIRST stop's colour past the LAST stop — a wrap no one asked for and the one
/// behaviour change in this layer that is visible without a LUT.
#[test]
fn partial_coverage_clamps_to_the_ends() {
    let g = Gradient::from_stops("t", &[(0.25, [1.0, 0.0, 0.0]), (0.75, [0.0, 0.0, 1.0])]);
    approx(g.eval(0.0), [1.0, 0.0, 0.0, 1.0], 1e-6, "below the first stop");
    approx(g.eval(0.1), [1.0, 0.0, 0.0, 1.0], 1e-6, "still below");
    approx(g.eval(1.0), [0.0, 0.0, 1.0, 1.0], 1e-6, "above the last stop");
    approx(g.eval(0.5), [0.5, 0.0, 0.5, 1.0], 1e-6, "between them");
}

/// Degenerate inputs produce a usable gradient rather than a panic or a NaN — these arrive from
/// parsed files and from an editor mid-drag.
#[test]
fn degenerate_gradients_are_survivable() {
    // No stops at all.
    let g = Gradient::from_stops("empty", &[]);
    assert_eq!(g.eval(0.5), [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(g.bake(16).entries.len(), 16);
    // Duplicate positions: the zero-width span is dropped, not divided by.
    let d = Gradient::from_stops("dup", &[(0.0, [1.0; 3]), (0.5, [0.0; 3]), (0.5, [1.0; 3]), (1.0, [0.0; 3])]);
    assert!(d.segments.iter().all(|s| s.right > s.left));
    assert!(d.bake(32).entries.iter().all(|e| e.iter().all(|v| v.is_finite())));
    // Non-finite and out-of-range sample positions.
    let s = Gradient::from_stops("t", &[(0.0, [0.0; 3]), (1.0, [1.0; 3])]);
    assert!(s.eval(f32::NAN).iter().all(|v| v.is_finite()));
    assert_eq!(s.eval(-5.0), [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(s.eval(5.0), [1.0, 1.0, 1.0, 1.0]);
    // A zero-length LUT never panics on sample.
    assert!(Lut { entries: vec![], smooth: true }.sample(0.5).iter().all(|v| v.is_finite()));
    // Sampling wraps rather than clamping, so a cycled palette has no seam artefact at t = 1.
    let lut = s.bake(8);
    assert_eq!(lut.sample(1.25), lut.sample(0.25));
}

/// A colour list with no positions spreads evenly, ends included.
#[test]
fn colors_spread_evenly() {
    let g = Gradient::from_colors("t", &[[0.0; 3], [0.5; 3], [1.0; 3]]);
    approx(g.eval(0.0), [0.0, 0.0, 0.0, 1.0], 1e-6, "start");
    approx(g.eval(0.5), [0.5, 0.5, 0.5, 1.0], 1e-6, "middle");
    approx(g.eval(1.0), [1.0, 1.0, 1.0, 1.0], 1e-6, "end");
    // One colour is a flat gradient, not an error.
    approx(Gradient::from_colors("one", &[[0.2, 0.4, 0.6]]).eval(0.9), [0.2, 0.4, 0.6, 1.0], 1e-6, "single");
}

/// Every shipped preset converts, covers `0..1` contiguously, and bakes without surprises — the
/// presets are the first thing routed through this layer, so a gap here is a gap on screen.
#[test]
fn presets_convert_to_contiguous_gradients() {
    for p in PRESETS {
        let g = Gradient::from_stops(p.name, p.stops);
        assert_eq!(g.segments.first().unwrap().left, 0.0, "{}: starts at 0", p.name);
        assert_eq!(g.segments.last().unwrap().right, 1.0, "{}: ends at 1", p.name);
        for w in g.segments.windows(2) {
            assert_eq!(w[0].right, w[1].left, "{}: gap between segments", p.name);
        }
        let lut = g.bake(LUT_SIZE);
        assert_eq!(lut.entries.len(), LUT_SIZE);
        assert!(lut.smooth, "{}: presets are gradients, not bands", p.name);
        assert!(
            lut.entries.iter().all(|e| e.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v))),
            "{}: baked outside 0..1",
            p.name
        );
        // The baked table must agree with the stop walk it replaces, away from the kinks.
        for (pos, c) in p.stops {
            if *pos > 0.02 && *pos < 0.98 {
                let got = lut.sample(*pos);
                for ch in 0..3 {
                    assert!(
                        (got[ch] - c[ch]).abs() < 0.01,
                        "{}: stop at {pos} ch{ch} baked to {} not {}",
                        p.name,
                        got[ch],
                        c[ch]
                    );
                }
            }
        }
    }
}
