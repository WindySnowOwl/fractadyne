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

/// ⭐**Rotating a gradient rotates a RING, not a strip.** A segment that ends up straddling the
/// t = 1 / t = 0 seam must be SPLIT and both halves kept: sorting alone would drop the far half and
/// leave a flat clamp where colour used to be.
#[test]
fn rotation_splits_the_straddling_segment() {
    let g = Gradient::from_stops("t", &[(0.0, [0.0; 3]), (1.0, [1.0; 3])]);
    let r = g.rotated(0.25);
    // One segment in, two out — the split.
    assert_eq!(g.segments.len(), 1);
    assert_eq!(r.segments.len(), 2);
    // Still covers 0..1 with no gaps, so nothing was lost or invented.
    assert_eq!(r.segments.first().unwrap().left, 0.0);
    assert_eq!(r.segments.last().unwrap().right, 1.0);
    for w in r.segments.windows(2) {
        assert!((w[0].right - w[1].left).abs() < 1e-6, "gap at the split");
    }
    // The colour that was at 0 is now at 0.25 (probed just past, since black->white is not
    // seamless and 0.25 is a genuine discontinuity).
    assert!(r.eval(0.26)[0] < 0.05, "the ramp's dark end did not move to 0.25");
    assert!(r.eval(0.24)[0] > 0.95, "the ramp's bright end did not wrap to just below 0.25");
    // Every value the original held is still somewhere in the rotated ring.
    for i in 0..=20 {
        let want = g.eval(i as f32 / 20.0)[0];
        let found = (0..=400).any(|k| (r.eval(k as f32 / 400.0)[0] - want).abs() < 0.02);
        assert!(found, "value {want} vanished from the rotated gradient");
    }
    // A full turn, and a zero turn, are both the identity.
    assert_eq!(g.rotated(0.0), g);
    assert_eq!(g.rotated(1.0), g);
    // Non-finite input does not produce a NaN gradient.
    assert!(g.rotated(f32::NAN).eval(0.5).iter().all(|v| v.is_finite()));
}

/// Stops survive a round trip through the segment model, so the app can keep persisting a custom
/// palette as stops — including after a rotation, which is what `.ugr` import needs.
#[test]
fn stops_round_trip_through_segments() {
    let stops = vec![
        (0.0, [0.1, 0.2, 0.3]),
        (0.4, [0.9, 0.1, 0.2]),
        (1.0, [0.3, 0.7, 0.5]),
    ];
    let g = Gradient::from_stops("t", &stops);
    let back = g.to_stops();
    assert_eq!(back.len(), stops.len());
    for (a, b) in back.iter().zip(&stops) {
        assert!((a.0 - b.0).abs() < 1e-6, "position {a:?} vs {b:?}");
        for ch in 0..3 {
            assert!((a.1[ch] - b.1[ch]).abs() < 1e-6, "colour {a:?} vs {b:?}");
        }
    }
    // Rebuilding from the round-tripped stops gives the same gradient.
    assert_eq!(Gradient::from_stops("t", &back), g);
    // And a rotated gradient's stops rebuild it too (this is what .ugr rotation relies on).
    let r = g.rotated(0.3);
    let rebuilt = Gradient::from_stops("t", &r.to_stops());
    for i in 0..=50 {
        let t = i as f32 / 50.0;
        for ch in 0..3 {
            assert!(
                (rebuilt.eval(t)[ch] - r.eval(t)[ch]).abs() < 1e-5,
                "rotated round trip diverged at t={t}"
            );
        }
    }
}

/// ⭐A hard jump survives the stop round trip as a DUPLICATE POSITION. The app persists a custom
/// palette as stops, so without this a rotated (or otherwise discontinuous) gradient would come
/// back from a restart with its edge smoothed into a ramp — silently, and only visible as "the
/// colours look softer than when I imported them".
#[test]
fn a_hard_jump_round_trips_as_a_duplicate_position() {
    let g = Gradient {
        name: "jump".into(),
        segments: vec![
            Segment::linear(0.0, 0.5, [1.0, 0.0, 0.0, 1.0], [1.0, 1.0, 0.0, 1.0]),
            // Starts at BLUE where the previous ended at yellow — a genuine edge.
            Segment::linear(0.5, 1.0, [0.0, 0.0, 1.0, 1.0], [0.0, 1.0, 1.0, 1.0]),
        ],
    };
    let stops = g.to_stops();
    let at_half: Vec<_> = stops.iter().filter(|(p, _)| (*p - 0.5).abs() < 1e-6).collect();
    assert_eq!(at_half.len(), 2, "the edge did not become a duplicate position: {stops:?}");
    assert_eq!(at_half[0].1, [1.0, 1.0, 0.0], "first of the pair closes the old segment");
    assert_eq!(at_half[1].1, [0.0, 0.0, 1.0], "second of the pair opens the new one");

    // Rebuilt, the edge is still an edge — yellow just below, blue just above.
    let back = Gradient::from_stops("jump", &stops);
    assert!(back.eval(0.49)[1] > 0.9 && back.eval(0.49)[2] < 0.1, "should still be yellow below");
    assert!(back.eval(0.51)[2] > 0.9 && back.eval(0.51)[1] < 0.1, "should still be blue above");
    // And it matches the original everywhere away from the edge itself.
    for i in 0..=100 {
        let t = i as f32 / 100.0;
        if (t - 0.5).abs() < 0.01 {
            continue;
        }
        for ch in 0..3 {
            assert!((back.eval(t)[ch] - g.eval(t)[ch]).abs() < 1e-5, "diverged at t={t}");
        }
    }
    // A continuous gradient gains no spurious duplicates.
    let smooth = Gradient::from_stops("s", &[(0.0, [0.0; 3]), (0.5, [1.0; 3]), (1.0, [0.0; 3])]);
    assert_eq!(smooth.to_stops().len(), 3);
}

/// ⭐**`factor` is the curve the editor's preview draws**, so it must BE the function `eval` uses —
/// not a second implementation that can drift. Checked by reconstructing a colour from the factor
/// and requiring it to match what `eval` produced.
#[test]
fn the_exposed_factor_is_the_one_eval_uses() {
    for blend in [
        Blend::Linear,
        Blend::Curved,
        Blend::Sine,
        Blend::SphereIncreasing,
        Blend::SphereDecreasing,
    ] {
        let seg = Segment {
            left: 0.2,
            mid: 0.45, // off-centre, so a wrong normalisation would show
            right: 0.8,
            left_color: [0.0, 0.25, 1.0, 1.0],
            right_color: [1.0, 0.75, 0.0, 1.0],
            blend,
            space: Space::Rgb,
        };
        let g = Gradient { name: "t".into(), segments: vec![seg] };
        for i in 0..=20 {
            let t = 0.2 + (0.6 * i as f32 / 20.0);
            let f = seg.factor(t);
            let want = [
                seg.left_color[0] + (seg.right_color[0] - seg.left_color[0]) * f,
                seg.left_color[1] + (seg.right_color[1] - seg.left_color[1]) * f,
                seg.left_color[2] + (seg.right_color[2] - seg.left_color[2]) * f,
            ];
            for ch in 0..3 {
                assert!(
                    (g.eval(t)[ch] - want[ch]).abs() < 1e-5,
                    "{blend:?} at t={t}: factor and eval disagree"
                );
            }
        }
        // Ends are pinned for every curve.
        assert!(seg.factor(0.2).abs() < 1e-5, "{blend:?} at the left end");
        assert!((seg.factor(0.8) - 1.0).abs() < 1e-5, "{blend:?} at the right end");
    }
    // ⚠And the asymmetry a prettified preview would hide: sphere-increasing is 0.866 at halfway.
    let s = Segment {
        left: 0.0, mid: 0.5, right: 1.0,
        left_color: [0.0; 4], right_color: [1.0; 4],
        blend: Blend::SphereIncreasing, space: Space::Rgb,
    };
    assert!((s.factor(0.5) - 0.75f32.sqrt()).abs() < 1e-5, "got {}", s.factor(0.5));
}

/// ⭐⭐**The hue-undefined warning fires exactly when the trap applies.** An HSV segment with an
/// unsaturated endpoint sweeps the whole wheel, because `rgb_to_hsv` reports hue 0 for greys — a
/// black→red segment goes through green. RGB segments are never affected, and a segment between two
/// saturated colours is doing what the user asked.
#[test]
fn the_hue_undefined_warning_fires_only_where_it_applies() {
    let seg = |lc: [f32; 4], rc: [f32; 4], space| Segment {
        left: 0.0, mid: 0.5, right: 1.0,
        left_color: lc, right_color: rc, blend: Blend::Linear, space,
    };
    const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
    const GREY: [f32; 4] = [0.5, 0.5, 0.5, 1.0];
    const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
    const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

    for (lc, rc, why) in [(BLACK, RED, "black"), (GREY, RED, "grey"), (WHITE, RED, "white"),
                          (RED, BLACK, "unsaturated on the right")] {
        assert!(seg(lc, rc, Space::HsvCcw).hue_undefined_endpoint(), "{why} should warn");
        assert!(seg(lc, rc, Space::HsvCw).hue_undefined_endpoint(), "{why}, clockwise");
        // ⚠The same endpoints in RGB are perfectly ordinary — the warning is about the SPACE.
        assert!(!seg(lc, rc, Space::Rgb).hue_undefined_endpoint(), "{why} in RGB must not warn");
    }
    // Two saturated colours in HSV is the intended use, not a trap.
    assert!(!seg(RED, BLUE, Space::HsvCcw).hue_undefined_endpoint());
    // And the measured consequence is real: black -> red really does pass through green.
    let g = Gradient { name: "t".into(), segments: vec![seg(BLACK, RED, Space::HsvCcw)] };
    let c = g.eval(0.25);
    assert!(c[1] > c[0] && c[1] > c[2], "expected green-dominant at t=0.25, got {c:?}");
}
