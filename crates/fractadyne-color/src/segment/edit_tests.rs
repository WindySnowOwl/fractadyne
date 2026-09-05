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
