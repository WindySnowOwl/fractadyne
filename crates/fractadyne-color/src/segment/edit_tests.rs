use super::*;

/// The stop list every default gradient is built from — a preset-shaped ramp.
fn stops() -> Vec<(f32, [f32; 3])> {
    vec![
        (0.0, [0.0, 0.0, 0.0]),
        (0.35, [0.9, 0.1, 0.2]),
        (0.7, [0.2, 0.6, 0.9]),
        (1.0, [1.0, 1.0, 1.0]),
    ]
}

fn plain() -> Gradient {
    Gradient::from_stops("t", &stops())
}

/// A gradient carrying the things a stop list cannot hold, so every operation below can be checked
/// for preserving them.
fn rich() -> Gradient {
    Gradient {
        name: "rich".into(),
        segments: vec![
            Segment {
                left: 0.0,
                mid: 0.1, // deliberately NOT centred
                right: 0.5,
                left_color: [1.0, 0.0, 0.0, 1.0],
                right_color: [0.0, 1.0, 0.0, 0.5],
                blend: Blend::Sine,
                space: Space::HsvCcw,
            },
            Segment {
                left: 0.5,
                mid: 0.9,
                right: 1.0,
                left_color: [0.0, 1.0, 0.0, 0.5],
                right_color: [0.0, 0.0, 1.0, 1.0],
                blend: Blend::SphereIncreasing,
                space: Space::Rgb,
            },
        ],
    }
}

fn same_gradient(a: &Gradient, b: &Gradient, what: &str) {
    for i in 0..=200 {
        let t = i as f32 / 200.0;
        for ch in 0..3 {
            assert!(
                (a.eval(t)[ch] - b.eval(t)[ch]).abs() < 1e-5,
                "{what}: diverged at t={t} ch{ch} ({:?} vs {:?})",
                a.eval(t),
                b.eval(t)
            );
        }
    }
}

/// Stops are segment boundaries: N segments, N+1 stops, and reading them back gives what went in.
#[test]
fn stops_are_segment_boundaries() {
    let g = plain();
    assert_eq!(g.segments.len(), 3);
    assert_eq!(g.stop_count(), 4);
    for (i, (pos, rgb)) in stops().iter().enumerate() {
        let (p, c) = g.stop(i).expect("stop in range");
        assert!((p - pos).abs() < 1e-6, "stop {i} position");
        assert_eq!(c, *rgb, "stop {i} colour");
    }
    assert!(g.stop(4).is_none(), "past the end");
    assert_eq!(Gradient::default().stop_count(), 0);
    assert!(Gradient::default().stop(0).is_none());
}

/// ⭐⭐**THE ZERO-DRIFT CONTRACT.** On a default gradient — `Linear`/`Rgb`, centred midpoints, which
/// is exactly what `from_stops` produces for every preset, every pasted palette and every session
/// written before P1 — each edit must give the gradient `from_stops` would give for the edited stop
/// list. If this fails, migrating the editor to segments moves pixels, and the whole phase was
/// supposed to move none.
#[test]
fn every_edit_matches_from_stops_on_a_default_gradient() {
    // Recolour.
    let mut g = plain();
    g.set_stop_color(1, [0.3, 0.4, 0.5]);
    let mut s = stops();
    s[1].1 = [0.3, 0.4, 0.5];
    same_gradient(&g, &Gradient::from_stops("t", &s), "set_stop_color");

    // Move.
    let mut g = plain();
    g.set_stop_position(1, 0.2);
    let mut s = stops();
    s[1].0 = 0.2;
    same_gradient(&g, &Gradient::from_stops("t", &s), "set_stop_position");

    // Insert.
    let mut g = plain();
    let at = g.insert_stop(0.5).expect("inside a segment");
    let (pos, rgb) = g.stop(at).unwrap();
    let mut s = stops();
    s.insert(2, (pos, rgb));
    same_gradient(&g, &Gradient::from_stops("t", &s), "insert_stop");

    // Remove.
    let mut g = plain();
    g.remove_stop(1);
    let mut s = stops();
    s.remove(1);
    same_gradient(&g, &Gradient::from_stops("t", &s), "remove_stop");
}

/// ⭐**Inserting a stop into a LINEAR segment must not change the picture at all** — it adds a
/// handle, nothing else. This is the property a user relies on when they add a stop to adjust it
/// later, and it is what makes `insert_stop` safe to expose.
#[test]
fn inserting_a_stop_into_a_linear_segment_changes_nothing() {
    let before = plain();
    for at in [0.1, 0.35001, 0.5, 0.9] {
        let mut g = plain();
        if g.insert_stop(at).is_some() {
            same_gradient(&g, &before, &format!("insert at {at}"));
        }
    }
    // A position on top of an existing stop, or outside the range, inserts nothing.
    let mut g = plain();
    assert!(g.insert_stop(0.35).is_none(), "on an existing boundary");
    assert!(g.insert_stop(0.0).is_none() && g.insert_stop(1.0).is_none(), "at the ends");
    assert!(g.insert_stop(f32::NAN).is_none());
    assert_eq!(g.segments.len(), 3, "nothing was inserted");
}

/// ⚠**The three things a stop list cannot hold survive every edit** — that is the entire point of
/// making the editor segment-native. Blend, colour space and the midpoint FRACTION are preserved on
/// every segment an operation does not remove.
#[test]
fn edits_preserve_blend_space_and_midpoint_fraction() {
    // Recolour keeps everything but the colour.
    let mut g = rich();
    g.set_stop_color(1, [0.5, 0.5, 0.5]);
    assert_eq!(g.segments[0].blend, Blend::Sine);
    assert_eq!(g.segments[0].space, Space::HsvCcw);
    assert!((g.segments[0].mid - 0.1).abs() < 1e-6, "midpoint moved on a recolour");
    assert_eq!(g.segments[1].blend, Blend::SphereIncreasing);
    // Both sides of the boundary took the new colour, so the edge closed.
    assert_eq!(g.segments[0].right_color[..3], [0.5, 0.5, 0.5]);
    assert_eq!(g.segments[1].left_color[..3], [0.5, 0.5, 0.5]);
    // ⚠Alpha is NOT clobbered by a colour edit — the picker has no alpha channel.
    assert_eq!(g.segments[0].right_color[3], 0.5);

    // Moving a boundary keeps the midpoint FRACTION, not its position.
    let mut g = rich();
    let frac_before = (g.segments[0].mid - g.segments[0].left) / (g.segments[0].right - g.segments[0].left);
    g.set_stop_position(1, 0.25);
    let s = &g.segments[0];
    let frac_after = (s.mid - s.left) / (s.right - s.left);
    assert!((frac_before - frac_after).abs() < 1e-5, "fraction {frac_before} -> {frac_after}");
    assert!((s.right - 0.25).abs() < 1e-6, "the boundary did not move");
    assert!(s.mid < 0.25, "an off-centre midpoint must stay inside its own segment");
    assert_eq!(s.blend, Blend::Sine, "blend survived the move");

    // A split inherits blend and space on BOTH halves.
    let mut g = rich();
    g.insert_stop(0.25).expect("inside segment 0");
    assert_eq!(g.segments.len(), 3);
    assert_eq!(g.segments[0].blend, Blend::Sine);
    assert_eq!(g.segments[1].blend, Blend::Sine);
    assert_eq!(g.segments[0].space, Space::HsvCcw);
    assert_eq!(g.segments[1].space, Space::HsvCcw);
    // The join is exact: the split colour is evaluated, not guessed.
    assert_eq!(g.segments[0].right_color, g.segments[1].left_color);
}

/// ⚠**Splitting a CURVED segment is approximate, and that is inherent** — half of a sine curve is
/// not a sine curve. The endpoints and the split point still match exactly; the shape between them
/// moves. Pinned so the editor can warn instead of quietly reshaping someone's gradient.
#[test]
fn splitting_a_curved_segment_is_exact_at_the_joins_and_approximate_between() {
    let before = rich();
    let mut g = rich();
    g.insert_stop(0.25).expect("inside segment 0");
    for t in [0.0, 0.25, 0.5, 1.0] {
        for ch in 0..3 {
            assert!(
                (g.eval(t)[ch] - before.eval(t)[ch]).abs() < 1e-5,
                "the joins must be exact, t={t}"
            );
        }
    }
    let moved = (0..=40)
        .map(|i| i as f32 / 40.0 * 0.5)
        .any(|t| (0..3).any(|ch| (g.eval(t)[ch] - before.eval(t)[ch]).abs() > 0.01));
    assert!(moved, "a split sine segment should reshape between the joins - if not, say so instead");
}

/// Removing a stop merges two segments; the LEFT one's attributes win, which is a choice and is
/// documented as one.
#[test]
fn removing_a_stop_merges_and_the_left_segment_wins() {
    let mut g = rich();
    g.remove_stop(1);
    assert_eq!(g.segments.len(), 1);
    let s = &g.segments[0];
    assert_eq!(s.left, 0.0);
    assert_eq!(s.right, 1.0);
    assert_eq!(s.blend, Blend::Sine, "the LEFT segment's blend wins");
    assert_eq!(s.space, Space::HsvCcw, "the LEFT segment's space wins");
    assert_eq!(s.left_color, [1.0, 0.0, 0.0, 1.0], "outer endpoints survive");
    assert_eq!(s.right_color, [0.0, 0.0, 1.0, 1.0]);
    // The left segment's midpoint FRACTION (0.1 of 0..0.5 = 0.2) carries onto the merged span.
    assert!((s.mid - 0.2).abs() < 1e-5, "merged midpoint fraction, got {}", s.mid);
}

/// Out-of-range and end-stop operations are refused rather than panicking or silently corrupting
/// the span contract (a gradient must keep covering 0..1).
#[test]
fn end_stops_and_bad_indices_are_refused() {
    let before = plain();
    for i in [0usize, 3, 4, 99] {
        let mut g = plain();
        g.set_stop_position(i, 0.5);
        same_gradient(&g, &before, &format!("set_stop_position({i}) should be a no-op"));
        let mut g = plain();
        g.remove_stop(i);
        assert_eq!(g.segments.len(), 3, "remove_stop({i}) should be a no-op");
    }
    // Coverage is intact after a legal move.
    let mut g = plain();
    g.set_stop_position(1, 0.99);
    assert_eq!(g.segments.first().unwrap().left, 0.0);
    assert_eq!(g.segments.last().unwrap().right, 1.0);
    for w in g.segments.windows(2) {
        assert!((w[0].right - w[1].left).abs() < 1e-6, "a gap opened");
        assert!(w[0].right > w[0].left, "a segment collapsed");
    }
    // A move past a neighbour clamps instead of reordering.
    let mut g = plain();
    g.set_stop_position(2, 0.01);
    assert!(g.stop(2).unwrap().0 > g.stop(1).unwrap().0, "stops must stay ordered");
    // A colour set on a bad index changes nothing.
    let mut g = plain();
    g.set_stop_color(99, [1.0, 0.0, 0.0]);
    same_gradient(&g, &before, "set_stop_color out of range");
}

/// A one-segment gradient is the floor: its stop can be recoloured, and nothing can be removed.
#[test]
fn a_single_segment_gradient_survives_editing() {
    let mut g = Gradient::from_stops("t", &[(0.0, [0.0; 3]), (1.0, [1.0; 3])]);
    assert_eq!(g.stop_count(), 2);
    g.remove_stop(1);
    assert_eq!(g.segments.len(), 1, "the last segment cannot be removed");
    g.set_stop_color(0, [0.2, 0.4, 0.6]);
    assert_eq!(g.eval(0.0)[..3], [0.2, 0.4, 0.6]);
}

/// ⭐⭐**Every gradient the editor produces must report as stop-expressible until a user actually
/// sets a curve.** The bug this pins was live and visible in a screenshot: after P1 made the
/// editor segment-native, the "Imported gradient … Convert to editable stops" notice keyed off
/// "has segments", which every custom gradient now does — so a gradient copied from a preset
/// advertised midpoints and blend curves it did not have, and offered to convert it into itself.
///
/// ⚠The three "rich" cases are asserted one at a time. A predicate that only tested `blend` would
/// pass a test that changed all three at once.
#[test]
fn stop_expressible_tracks_content_not_the_mere_presence_of_segments() {
    let plain = Gradient::from_stops("t", &stops());
    assert!(plain.is_stop_expressible(), "from_stops IS the stop-expressible shape");
    assert!(
        Gradient::from_bands("t", &[[0.0; 3], [1.0; 3], [0.5; 3]]).is_stop_expressible(),
        "flat bands are Linear/Rgb/centred too — a .map is a colour list, not a rich gradient"
    );
    // Surviving the editor's own operations must not turn a plain gradient rich: `set_span` keeps
    // the midpoint FRACTION, which is exactly what the tolerance here exists to accept.
    let mut dragged = plain.clone();
    dragged.set_stop_position(1, 0.61);
    dragged.set_stop_position(2, 0.62);
    let i = dragged.insert_stop(0.8).expect("split");
    dragged.set_stop_color(i, [0.1, 0.2, 0.3]);
    dragged.remove_stop(1);
    assert!(dragged.is_stop_expressible(), "dragging, splitting and merging keep it plain");

    // One field at a time, each on its own copy.
    let mut curved = plain.clone();
    curved.segments[1].blend = Blend::Sine;
    assert!(!curved.is_stop_expressible(), "a blend curve is not expressible as stops");
    let mut swept = plain.clone();
    swept.segments[0].space = Space::HsvCcw;
    assert!(!swept.is_stop_expressible(), "a hue sweep is not expressible as stops");
    let mut shifted = plain.clone();
    let s = shifted.segments[2];
    shifted.segments[2].mid = s.left + 0.8 * (s.right - s.left);
    assert!(!shifted.is_stop_expressible(), "an off-centre midpoint is not expressible as stops");
}

// ── Rotation ────────────────────────────────────────────────────────────────────────────────────

/// The colour a rotated gradient shows at `t` must be the colour the original showed at `t - delta`,
/// wrapped. This is the definition of rotation, and everything below is a corner of it.
fn assert_rotated_by(orig: &Gradient, rotated: &Gradient, delta: f32, tol: f32, what: &str) {
    for i in 0..=400 {
        let t = i as f32 / 400.0;
        // ⚠Skip a hair either side of the seam and of the sample point's pre-image: both land on a
        // hard boundary where the two gradients legitimately pick different sides of a jump.
        let src = (t - delta).rem_euclid(1.0);
        if t < 1.0e-3 || t > 1.0 - 1.0e-3 || src < 1.0e-3 || src > 1.0 - 1.0e-3 {
            continue;
        }
        for ch in 0..3 {
            let (a, b) = (rotated.eval(t)[ch], orig.eval(src)[ch]);
            assert!(
                (a - b).abs() <= tol,
                "{what}: at t={t} (from {src}) channel {ch} read {a}, expected {b}"
            );
        }
    }
}

/// ⭐⭐**Rotating a linearly-blended gradient is EXACT.** This is the case that matters: it is what
/// every preset is, what `from_stops` builds, and what the editor's Bézier promotion produces. If
/// rotation could not be lossless here it would be a destructive edit dressed up as a view control,
/// and dragging the ring back and forth would slowly dissolve the user's gradient.
#[test]
fn rotating_a_linear_gradient_is_exact() {
    let g = plain();
    for delta in [0.05_f32, 0.25, 0.5, 0.731, 0.99] {
        let mut r = g.clone();
        r.rotate(delta);
        assert_rotated_by(&g, &r, delta, 1.0e-5, &format!("rotate({delta})"));
    }
}

/// ⚠**At most ONE new segment, and none when the seam lands on a stop.** Rotation cuts where the
/// new seam falls; a rotation that lands exactly on an existing boundary has nothing to cut. Without
/// this, dragging the ring would inflate the segment list toward `EDITOR_MAX_STOPS` and the editor
/// would start refusing stops the user never added.
#[test]
fn rotation_adds_at_most_one_segment_and_none_when_it_lands_on_a_stop() {
    let g = plain();
    let n = g.segments.len();

    let mut r = g.clone();
    r.rotate(0.137);
    assert_eq!(r.segments.len(), n + 1, "a cut through a segment splits exactly one of them");

    // `plain` has stops at 0.35 and 0.7. Rotating by 1 - 0.7 puts the seam exactly on the 0.7 stop.
    let mut onto = g.clone();
    onto.rotate(1.0 - 0.7);
    assert_eq!(onto.segments.len(), n, "landing on an existing stop must not cut anything");
    assert_rotated_by(&g, &onto, 0.3, 1.0e-5, "rotate onto a stop");
}

/// A rotation that is a whole number of turns is the identity — including the `0.0` case, which has
/// to be caught before the cut, or an exact no-op would still split a segment.
#[test]
fn a_whole_turn_changes_nothing() {
    let g = plain();
    for delta in [0.0_f32, 1.0, -1.0, 2.0] {
        let mut r = g.clone();
        r.rotate(delta);
        assert_eq!(r.segments.len(), g.segments.len(), "rotate({delta}) must not cut");
        same_gradient(&g, &r, &format!("rotate({delta})"));
    }
}

/// Rotating forward then back returns the original picture. ⚠The segment COUNT does not come back —
/// the two cuts are real — which is why the editor rotates from a pre-drag baseline rather than
/// accumulating; this pins the colours, which is what the user sees.
#[test]
fn rotating_there_and_back_restores_the_picture() {
    let g = plain();
    let mut r = g.clone();
    r.rotate(0.42);
    r.rotate(-0.42);
    same_gradient(&g, &r, "rotate(0.42) then rotate(-0.42)");
}

/// The gradient still COVERS 0..1 after rotating, with no sliver left uncovered at either end.
/// ⚠A last segment ending at 0.99999994 is invisible in every test that samples on a grid and shows
/// up as a hairline of the wrong colour at the seam — the one place a rotation puts the user's eye.
#[test]
fn a_rotated_gradient_still_covers_the_whole_range() {
    for delta in [0.05_f32, 0.3333, 0.5, 0.87] {
        let mut r = plain();
        r.rotate(delta);
        assert_eq!(r.segments.first().unwrap().left, 0.0, "delta={delta}: a gap at the bottom");
        assert_eq!(r.segments.last().unwrap().right, 1.0, "delta={delta}: a gap at the top");
        for w in r.segments.windows(2) {
            assert!(
                (w[1].left - w[0].right).abs() < 1.0e-6,
                "delta={delta}: a gap between segments at {}",
                w[0].right
            );
            assert!(w[0].right > w[0].left, "delta={delta}: a segment collapsed or inverted");
        }
        for s in &r.segments {
            assert!(s.mid >= s.left && s.mid <= s.right, "delta={delta}: a midpoint left its span");
        }
    }
}

/// ⚠**Rotation carries the rich properties a stop list cannot hold** — a segment that is not cut
/// must arrive with its curve, its colour space and its midpoint FRACTION intact. The one segment
/// the new seam cuts is the approximate case, and it is approximate in exactly the way adding a stop
/// there already is; this checks the others are untouched.
#[test]
fn rotation_preserves_the_curves_of_the_segments_it_does_not_cut() {
    let g = rich(); // segments [0, 0.5] Sine/HsvCcw and [0.5, 1] SphereIncreasing/Rgb
    let mut r = g.clone();
    // Land the seam exactly on the 0.5 stop, so NOTHING is cut and both segments must survive whole.
    r.rotate(0.5);
    assert_eq!(r.segments.len(), 2, "landing on the stop must not cut");
    // The two have swapped ends; each keeps its own blend, space and span.
    assert_eq!(r.segments[0].blend, Blend::SphereIncreasing);
    assert_eq!(r.segments[0].space, Space::Rgb);
    assert_eq!(r.segments[1].blend, Blend::Sine);
    assert_eq!(r.segments[1].space, Space::HsvCcw);
    // The midpoint FRACTION rides along: the second segment was mid 0.1 in span [0, 0.5] = 0.2 of
    // the way across, and must still be 0.2 of the way across wherever it has landed.
    let s = r.segments[1];
    let frac = (s.mid - s.left) / (s.right - s.left);
    assert!((frac - 0.2).abs() < 1.0e-5, "the midpoint fraction moved: {frac}");
    assert_rotated_by(&g, &r, 0.5, 1.0e-4, "rotate a rich gradient onto a stop");
}
