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

// ── Blend kind 5: the cubic-Bézier ease ─────────────────────────────────────────────────────

/// The solver has to be a FUNCTION on `0..1` before it can be an ease: pinned endpoints and no
/// NaN anywhere — including for the degenerate handles a user can drag it into.
#[test]
fn the_bezier_ease_is_a_well_behaved_function() {
    let cases: [[f32; 4]; 7] = [
        BEZIER_IDENTITY,
        [0.0; 4],               // the serde default — also the identity
        [0.42, 0.0, 0.58, 1.0], // ease-in-out
        [0.25, 0.1, 0.25, 1.0], // CSS "ease"
        [0.0, 0.0, 1.0, 1.0],   // handles pinned to the ends
        [1.0, 0.0, 0.0, 1.0],   // reversed handles — a legal, very steep S
        [-3.0, 0.5, 4.0, 0.5],  // x out of range: must be clamped, not explode
    ];
    for p in cases {
        assert_eq!(bezier_ease(p, 0.0), 0.0, "{p:?} must start at 0");
        assert_eq!(bezier_ease(p, 1.0), 1.0, "{p:?} must end at 1");
        for k in 0..=200 {
            let t = k as f32 / 200.0;
            let y = bezier_ease(p, t);
            assert!(y.is_finite(), "{p:?} at t={t} produced {y}");
        }
        // Out-of-range and non-finite inputs resolve rather than propagate.
        assert!(bezier_ease(p, -1.0).is_finite());
        assert!(bezier_ease(p, 2.0).is_finite());
        assert!(bezier_ease(p, f32::NAN).is_finite());
    }
}

/// ⭐**The identity has to be EXACT**, in both parameterisations, or "convert this linear segment
/// to an editable curve" would move pixels the moment it was pressed — and this phase is supposed
/// to be zero-drift until a user deliberately bends something.
#[test]
fn the_identity_bezier_is_exactly_linear() {
    for p in [BEZIER_IDENTITY, [0.0; 4]] {
        for k in 0..=100 {
            let t = k as f32 / 100.0;
            let y = bezier_ease(p, t);
            assert!((y - t).abs() < 1.0e-4, "{p:?}: y({t}) = {y}, expected {t}");
        }
    }
}

/// ⚠**`y` is deliberately NOT clamped by the ease** — an overshooting curve is the point of
/// letting the handles leave the box — but the COLOUR must still be in gamut. The clamp lives in
/// `Segment::eval`, and this pins both halves of that split.
#[test]
fn overshoot_survives_the_factor_and_is_clamped_at_the_colour() {
    let over = [0.3, 4.0, 0.7, -3.0]; // swings well outside 0..1 on both sides
    let mut saw_above = false;
    let mut saw_below = false;
    for k in 0..=100 {
        let y = bezier_ease(over, k as f32 / 100.0);
        saw_above |= y > 1.01;
        saw_below |= y < -0.01;
    }
    assert!(
        saw_above && saw_below,
        "the probe curve must actually overshoot, or this proves nothing"
    );

    let seg = Segment {
        left: 0.0,
        mid: 0.5,
        right: 1.0,
        left_color: [0.2, 0.4, 0.6, 1.0],
        right_color: [0.8, 0.5, 0.1, 1.0],
        blend: Blend::Bezier(over),
        space: Space::Rgb,
    };
    let g = Gradient { name: "t".into(), segments: vec![seg] };
    for k in 0..=200 {
        let c = g.eval(k as f32 / 200.0);
        for (ch, v) in c.iter().enumerate() {
            assert!((0.0..=1.0).contains(v), "channel {ch} left gamut at {v}");
        }
    }
}

/// ⭐⭐**Kind 5 IGNORES the midpoint** (`design/gradient-curves.md` §9.4 decision 1): the handles
/// already say where the curve reaches halfway, so composing GIMP's midpoint warp on top would be
/// two knobs for one shape and the handles would lie about where the curve goes.
///
/// ⚠"Ignored" is a claim about the EVALUATOR, not about storage — `mid` still persists, which is
/// what lets switching to Bézier and back restore the original curve exactly.
#[test]
fn bezier_ignores_the_midpoint() {
    let base = Segment {
        left: 0.0,
        mid: 0.5,
        right: 1.0,
        left_color: [0.0, 0.0, 0.0, 1.0],
        right_color: [1.0, 1.0, 1.0, 1.0],
        blend: Blend::Bezier([0.42, 0.0, 0.58, 1.0]),
        space: Space::Rgb,
    };
    let mut shifted = base;
    shifted.mid = 0.15;
    for k in 0..=100 {
        let t = k as f32 / 100.0;
        assert_eq!(base.factor(t), shifted.factor(t), "the midpoint changed a Bézier at t={t}");
    }
    // The control: on a kind 0–4 segment the very same midpoint move DOES change the curve, so
    // the equality above is a property of kind 5 and not of the inputs this test happened to use.
    let mut lin = base;
    lin.blend = Blend::Linear;
    let mut lin_shifted = lin;
    lin_shifted.mid = 0.15;
    assert_ne!(lin.factor(0.3), lin_shifted.factor(0.3), "the control must be sensitive to mid");
}

/// ⭐A `Linear` segment converted to an editable Bézier must render IDENTICALLY — the button is
/// "make this editable", not "change this". The fit is exact for anything already in the cubic
/// family and merely close for the sine and sphere blends, which are not cubics at all.
#[test]
fn fitting_a_bezier_is_exact_for_linear_and_close_for_the_rest() {
    let seg = |b: Blend| Segment {
        left: 0.0,
        mid: 0.5,
        right: 1.0,
        left_color: [0.0; 4],
        right_color: [1.0; 4],
        blend: b,
        space: Space::Rgb,
    };
    let lin = seg(Blend::Linear);
    let fitted = Blend::fit_to(|t| lin.factor(t));
    for k in 0..=100 {
        let t = k as f32 / 100.0;
        assert!(
            (bezier_ease(fitted, t) - lin.factor(t)).abs() < 1.0e-4,
            "a fitted LINEAR segment must be exact at t={t}"
        );
    }
    // ⚠⚠**The others are approximations, and HOW approximate differs by an order of magnitude.**
    // A single tolerance covering all four would hide that, so each kind is pinned against what it
    // actually achieves — and the spheres are asserted to be the BAD case, not merely tolerated.
    // Measured 2026-09-05: Curved 0 (it is a cubic), Sine 0.003, both spheres 0.136.
    let worst_of = |b: Blend| {
        let s = seg(b);
        let (f, worst) = Blend::fit_with_error(|t| s.factor(t));
        // `fit_with_error` must agree with an independent sweep, or the number the editor shows
        // the user is its own opinion rather than a measurement.
        let swept = (0..=400)
            .map(|k| {
                let t = k as f32 / 400.0;
                (bezier_ease(f, t) - s.factor(t)).abs()
            })
            .fold(0.0_f32, f32::max);
        assert!((swept - worst).abs() < 0.01, "{b:?}: reported {worst}, swept {swept}");
        worst
    };
    assert!(worst_of(Blend::Curved) < 1.0e-4, "Curved at a centred midpoint IS a cubic - exact");
    assert!(worst_of(Blend::Sine) < 0.01, "Sine is very close to a cubic");
    // ⭐The spheres have a VERTICAL TANGENT at one end and no cubic with finite control points
    // does, so this error is structural. Bounded from BOTH sides: too small would mean the fit
    // silently changed and the editor's warning is now overstated.
    for b in [Blend::SphereIncreasing, Blend::SphereDecreasing] {
        let w = worst_of(b);
        assert!((0.10..0.18).contains(&w), "{b:?} fitted with worst error {w}, outside 0.10..0.18");
    }
}

/// ⚠⚠**§4 trap 5, re-measured on a CURVE-HEAVY gradient.** The LUT's acceptance criterion — error
/// must SHRINK as the table grows — was established on piecewise-LINEAR gradients, and a curve has
/// more curvature between samples, so that evidence did not carry over on its own. This is the
/// measurement, and it is the gate saying 1024 entries are still enough now that a segment can be
/// an arbitrary cubic.
///
/// ⭐Counting DIFFERING ENTRIES rather than max error, for the reason the palette-LUT work
/// recorded: max error saturates at the output quantum and reads the same at every table size,
/// which is how a metric can look flat while the thing it measures improves tenfold.
#[test]
fn lut_error_still_shrinks_as_the_table_grows_on_a_curve_heavy_gradient() {
    // Every segment a different steep cubic, so between-sample curvature is as bad as the model
    // allows — a gentler gradient would make this pass without testing anything.
    let params: [[f32; 4]; 4] = [
        [0.9, 0.0, 0.1, 1.0],
        [0.0, 1.0, 1.0, 0.0],
        [0.8, 0.05, 0.2, 0.95],
        [0.05, 0.9, 0.95, 0.1],
    ];
    let colors = [[0.0, 0.0, 0.0], [1.0, 0.1, 0.0], [0.1, 0.9, 0.2], [0.0, 0.2, 1.0], [1.0; 3]];
    let segments: Vec<Segment> = (0..4)
        .map(|i| {
            let (l, r) = (i as f32 / 4.0, (i + 1) as f32 / 4.0);
            Segment {
                left: l,
                mid: 0.5 * (l + r),
                right: r,
                left_color: [colors[i][0], colors[i][1], colors[i][2], 1.0],
                right_color: [colors[i + 1][0], colors[i + 1][1], colors[i + 1][2], 1.0],
                blend: Blend::Bezier(params[i]),
                space: Space::Rgb,
            }
        })
        .collect();
    let g = Gradient { name: "curvy".into(), segments };

    // Each table against the gradient itself, probed at a fixed dense set of positions.
    let differing = |n: usize| {
        let lut = g.bake(n);
        let quantum = 1.0 / 255.0; // the output is 8-bit; finer than this is not a visible error
        (0..4096)
            .filter(|&k| {
                let t = (k as f32 + 0.5) / 4096.0;
                let (a, b) = (lut.sample(t), g.eval(t));
                (0..3).any(|c| (a[c] - b[c]).abs() > quantum)
            })
            .count()
    };
    let (at_1024, at_4096) = (differing(1024), differing(4096));
    assert!(
        at_4096 < at_1024,
        "LUT error did not shrink with table size on a curved gradient: {at_1024} -> {at_4096}"
    );
    // ⭐The number that matters for shipping: at the size the renderer actually uses, a curved
    // gradient must already be visually exact almost everywhere.
    assert!(
        at_1024 * 20 < 4096,
        "1024 entries left {at_1024} of 4096 probes visibly wrong on a curve-heavy gradient - the \
         table may no longer be big enough now that segments can be arbitrary cubics"
    );
}

/// ⭐⭐**A cycled palette's seam is invisible on a bar and unavoidable in the render.** The shader
/// takes `fract()` of the palette coordinate, so `t = 1` and `t = 0` are adjacent pixels; if the
/// ends differ, every sweep of the palette shows a hard edge there. This pins both the detector
/// and the fix, including that the fix moves the END and leaves everything else alone.
#[test]
fn seamlessness_is_detected_and_can_be_forced() {
    let stops = [
        (0.0_f32, [0.1_f32, 0.2, 0.3]),
        (0.4, [0.9, 0.1, 0.2]),
        (0.75, [0.2, 0.6, 0.9]),
        (1.0, [1.0, 1.0, 0.0]),
    ];
    let g = Gradient::from_stops("t", &stops);
    assert!(!g.is_seamless(), "the probe gradient must NOT be seamless, or this proves nothing");

    let mut fixed = g.clone();
    fixed.make_seamless();
    assert!(fixed.is_seamless());
    // ⚠**Compared with a tolerance, and the reason is worth knowing.** `is_seamless` compares the
    // stored endpoints exactly, but `eval(1.0)` runs the lerp `a + (b - a) * 1.0`, which is not
    // bit-identically `b` — measured 0.19999999 against 0.2. The seam is closed to well within the
    // 1/255 the renderer can express; demanding bit-equality here would be testing f32, not the
    // feature.
    approx(fixed.eval(1.0), fixed.eval(0.0), 1.0e-6, "the two ends must now be the same colour");

    // ⚠**The END moved, not the start.** Position 0 is where a preset's defining colour sits;
    // pulling the start toward the end would change the gradient's identity to fix its join.
    assert_eq!(fixed.eval(0.0), g.eval(0.0), "the start must be untouched");
    assert_ne!(fixed.eval(1.0), g.eval(1.0), "the end must be the thing that moved");

    // ⭐Everything BETWEEN the ends is untouched — only the final segment's right endpoint moves,
    // so a seamless toggle cannot quietly restyle the middle of someone's gradient.
    assert_eq!(fixed.segments.len(), g.segments.len());
    for i in 0..g.segments.len() - 1 {
        assert_eq!(fixed.segments[i], g.segments[i], "segment {i} changed");
    }
    let (a, b) = (g.segments.last().unwrap(), fixed.segments.last().unwrap());
    assert_eq!((a.left, a.mid, a.right, a.left_color), (b.left, b.mid, b.right, b.left_color));

    // Idempotent, and a no-op on something already seamless.
    let mut again = fixed.clone();
    again.make_seamless();
    assert_eq!(again, fixed);

    // Degenerate inputs must not panic: an empty gradient is vacuously seamless.
    let mut empty = Gradient { name: "e".into(), segments: vec![] };
    assert!(empty.is_seamless());
    empty.make_seamless();
    assert!(empty.segments.is_empty());
}

/// ⚠**Alpha counts.** A gradient whose ends match in RGB but differ in opacity still has a seam,
/// and `make_seamless` has to close that one too — otherwise the checkbox would claim a job done
/// that a future alpha-aware renderer would show is not.
#[test]
fn a_seam_in_alpha_is_still_a_seam() {
    let mut g = Gradient {
        name: "t".into(),
        segments: vec![Segment {
            left: 0.0,
            mid: 0.5,
            right: 1.0,
            left_color: [0.2, 0.4, 0.6, 1.0],
            right_color: [0.2, 0.4, 0.6, 0.25],
            blend: Blend::Linear,
            space: Space::Rgb,
        }],
    };
    assert!(!g.is_seamless(), "same RGB, different alpha, is not seamless");
    g.make_seamless();
    assert!(g.is_seamless());
    assert_eq!(g.segments[0].right_color[3], 1.0);
}

/// ⭐⭐**"Save and share" only means anything if it ROUND-TRIPS.** A `.ggr` writer that produces a
/// plausible file nobody can read back is worse than none, so this writes a gradient using every
/// feature the format carries, parses it with our own importer, and compares the BAKE — what the
/// GPU fetches — rather than the text.
#[test]
fn a_ggr_written_here_reads_back_identically() {
    let g = Gradient {
        name: "Round trip".into(),
        segments: vec![
            Segment {
                left: 0.0,
                mid: 0.08,
                right: 0.3,
                left_color: [0.0, 0.0, 0.0, 1.0],
                right_color: [0.9, 0.1, 0.2, 1.0],
                blend: Blend::SphereIncreasing,
                space: Space::Rgb,
            },
            Segment {
                left: 0.3,
                mid: 0.55,
                right: 1.0,
                left_color: [0.9, 0.1, 0.2, 1.0],
                right_color: [0.2, 0.6, 0.9, 1.0],
                blend: Blend::Sine,
                space: Space::HsvCw,
            },
        ],
    };
    assert_eq!(crate::segment::ggr_lossy_segments(&g), 0, "kinds 0-4 are GIMP's own — lossless");
    let text = crate::segment::write_ggr(&g);
    assert!(text.starts_with("GIMP Gradient\nName: Round trip\n2\n"), "header:\n{text}");
    let back = crate::import::parse_ggr(&text).expect("our own output must parse");
    assert_eq!(g.bake(LUT_SIZE), back.bake(LUT_SIZE), "the gradient came back different");
    assert_eq!(back.segments[0].blend, Blend::SphereIncreasing);
    assert_eq!(back.segments[1].space, Space::HsvCw);
    assert!((back.segments[0].mid - 0.08).abs() < 1.0e-5, "the midpoint must survive");
}

/// ⭐⭐**An identity Bézier is written as plain linear and reported as LOSSLESS**, because it *is*
/// linear — bit-identically. Without that exemption every gradient made in the editor would warn
/// about approximation that did not happen, and a warning that fires when nothing was lost is one
/// people learn to skip.
#[test]
fn an_unbent_bezier_exports_as_linear_and_a_bent_one_is_reported() {
    let seg = |b: Blend| Gradient {
        name: "b".into(),
        segments: vec![Segment {
            left: 0.0,
            mid: 0.5,
            right: 1.0,
            left_color: [0.0, 0.0, 0.0, 1.0],
            right_color: [1.0, 1.0, 1.0, 1.0],
            blend: b,
            space: Space::Rgb,
        }],
    };
    // Both spellings of the identity, including the all-zero serde default.
    for p in [BEZIER_IDENTITY, [0.0; 4]] {
        let g = seg(Blend::Bezier(p));
        assert!(crate::segment::bezier_is_identity(p), "{p:?} should read as the identity");
        assert_eq!(crate::segment::ggr_lossy_segments(&g), 0, "an unbent curve loses nothing");
        let back = crate::import::parse_ggr(&crate::segment::write_ggr(&g)).unwrap();
        assert_eq!(back.segments[0].blend, Blend::Linear, "written as GIMP's linear");
        // ⚠**Compared with a tolerance, and my own comment was wrong until this failed.** The
        // identity Bézier is linear to ~1e-4, not bit-identically: the ease solves `x(u) = t`
        // numerically and lands within a rounding error of the straight line. 1e-4 is a quarter
        // of the 1/255 the output can express, so it is visually exact — which is the property
        // the exemption actually needs.
        let (a, b) = (g.bake(LUT_SIZE), back.bake(LUT_SIZE));
        assert_eq!(a.entries.len(), b.entries.len());
        let worst = a
            .entries
            .iter()
            .zip(&b.entries)
            .flat_map(|(x, y)| (0..4).map(move |c| (x[c] - y[c]).abs()))
            .fold(0.0_f32, f32::max);
        assert!(worst < 1.0 / 255.0, "an unbent curve drifted by {worst}, more than one output level");
    }
    // A bent one is counted, and the count is what the UI warns from.
    let bent = seg(Blend::Bezier([0.9, 0.0, 0.1, 1.0]));
    assert!(!crate::segment::bezier_is_identity([0.9, 0.0, 0.1, 1.0]));
    assert_eq!(crate::segment::ggr_lossy_segments(&bent), 1);
    // ⚠And it really is lossy — the control that stops this being a warning about nothing.
    let back = crate::import::parse_ggr(&crate::segment::write_ggr(&bent)).unwrap();
    assert_ne!(bent.bake(LUT_SIZE), back.bake(LUT_SIZE), "a bent curve must actually differ");
}

/// ⚠A newline in a gradient's name would forge the segment-count line and produce a file that
/// parses as something else entirely. Names come from `.ggr` files and from a user's text field,
/// so neither is trusted.
#[test]
fn a_hostile_name_cannot_forge_the_file() {
    let g = Gradient {
        name: "evil\n99\n0 0 1 0 0 0 1 1 1 1 1 0 0".into(),
        segments: vec![Segment::linear(0.0, 1.0, [0.0; 4], [1.0; 4])],
    };
    let back = crate::import::parse_ggr(&crate::segment::write_ggr(&g))
        .expect("must still be a valid file");
    assert_eq!(back.segments.len(), 1, "the name must not inject segments");
    // An empty name still produces a valid file rather than a blank `Name:` GIMP may reject.
    let anon = Gradient { name: "   ".into(), segments: vec![Segment::linear(0.0, 1.0, [0.0; 4], [1.0; 4])] };
    assert!(crate::segment::write_ggr(&anon).contains("Name: Fractadyne"));
}

/// ⭐⭐**Promoting to Bézier must be invisible, and it must know when to refuse.**
///
/// ⚠⚠**The refusal is the important half.** `Blend::Linear` with an OFF-CENTRE midpoint is
/// piecewise linear with a kink, and no cubic reproduces a kink — promoting one would silently
/// reshape somebody's segment. Those keep `Linear`.
#[test]
fn promoting_linear_segments_to_bezier_changes_nothing_and_skips_the_kinked_ones() {
    let mut g = Gradient::from_stops(
        "t",
        &[(0.0, [0.0, 0.0, 0.0]), (0.35, [0.9, 0.1, 0.2]), (0.7, [0.2, 0.6, 0.9]), (1.0, [1.0; 3])],
    );
    // A curved segment and an off-centre linear one, neither of which may be touched.
    g.segments[1].blend = Blend::Sine;
    let s = g.segments[2];
    g.segments[2].mid = s.left + 0.2 * (s.right - s.left);
    let before = g.clone();

    g.promote_linear_to_bezier();
    assert_eq!(g.segments[0].blend, Blend::Bezier(BEZIER_IDENTITY), "a centred linear must promote");
    assert_eq!(g.segments[1].blend, Blend::Sine, "a named curve must be left alone");
    assert_eq!(g.segments[2].blend, Blend::Linear, "an off-centre linear has a KINK — leave it");

    // ⭐The whole point: the picture does not move. Compared bit-for-bit, because the identity
    // ease is answered exactly rather than solved — which is what makes this safe to do at all.
    assert_eq!(before.bake(LUT_SIZE), g.bake(LUT_SIZE), "promotion moved a pixel");
    // Idempotent — reopening the editor must not keep churning the gradient.
    let once = g.clone();
    g.promote_linear_to_bezier();
    assert_eq!(once, g);
    // And it stays expressible as stops, so the "rich gradient" notice does not start crying wolf.
    let mut plain = Gradient::from_stops("p", &[(0.0, [0.0; 3]), (1.0, [1.0; 3])]);
    assert!(plain.is_stop_expressible());
    plain.promote_linear_to_bezier();
    assert!(plain.is_stop_expressible(), "an unbent Bézier is still just a stop list");
}
