use super::*;

/// Where a point at `p` lands after zooming by `factor` about `anchor` (view pixels).
fn after(p: [f64; 2], anchor: [f64; 2], factor: f64) -> [f64; 2] {
    let m = 1.0 / factor;
    [anchor[0] + m * (p[0] - anchor[0]), anchor[1] + m * (p[1] - anchor[1])]
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

const C: [f64; 2] = [500.0, 300.0];
const R: f64 = 70.0;

#[test]
fn a_feature_under_the_cursor_halves_its_distance_per_octave() {
    // A feature right beside the cursor (attraction ~1) comes half way per octave zoomed in.
    let f = [500.0 + 1e-9, 300.0];
    let a = snap_anchor(C, f, R, 0.5);
    let f2 = after(f, a, 0.5);
    assert!((dist(f2, C) / dist(f, C) - 0.5).abs() < 1e-6, "{f2:?}");
}

#[test]
fn the_pull_tapers_to_nothing_at_the_radius() {
    // Halfway out (a = 0.5): the distance shrinks by 1 − 0.5·√0.5 an octave; at the rim it stays.
    let f = [C[0] + R / 2.0, C[1]];
    let f2 = after(f, snap_anchor(C, f, R, 0.5), 0.5);
    let want = 1.0 - SNAP_PULL * 0.5f64.sqrt();
    assert!((dist(f2, C) / dist(f, C) - want).abs() < 1e-9);
    let rim = [C[0], C[1] + R * 0.999_999];
    let r2 = after(rim, snap_anchor(C, rim, R, 0.5), 0.5);
    assert!((dist(r2, C) / dist(rim, C) - 1.0).abs() < 1e-2, "kept where it was, not flung out");
}

#[test]
fn small_steps_compose_to_the_same_octave() {
    // Sixty frames of a continuous zoom, one octave in all, pull as one octave's step does
    // (the feature's attraction rises as it nears, so a little more, never less).
    let f0 = [C[0] + 20.0, C[1] - 15.0];
    let step = 0.5f64.powf(1.0 / 60.0);
    let mut f = f0;
    for _ in 0..60 {
        f = after(f, snap_anchor(C, f, R, step), step);
    }
    let one = after(f0, snap_anchor(C, f0, R, 0.5), 0.5);
    assert!(dist(f, C) <= dist(one, C) + 1e-9, "{} vs {}", dist(f, C), dist(one, C));
    assert!(dist(f, C) > 0.5 * dist(one, C), "not wildly more either");
}

#[test]
fn nothing_changes_out_of_range_or_zooming_out() {
    let far = [C[0] + R * 1.5, C[1]];
    assert_eq!(snap_anchor(C, far, R, 0.5), C, "outside the radius");
    let near = [C[0] + 10.0, C[1]];
    assert_eq!(snap_anchor(C, near, R, 2.0), C, "a zoom out");
    assert_eq!(snap_anchor(C, near, R, 1.0), C, "no zoom");
    assert_eq!(snap_anchor(C, [f64::NAN, 0.0], R, 0.5), C, "a feature that is not on screen");
}

#[test]
fn a_feature_pulled_in_over_a_few_octaves_ends_under_the_cursor() {
    // Ten octaves of zooming in with a feature 40 px off: it ends within a pixel of the cursor,
    // where a plain zoom about the cursor would have flung it 40 000 px away.
    let mut f = [C[0] + 40.0, C[1] + 5.0];
    for _ in 0..10 {
        f = after(f, snap_anchor(C, f, R, 0.5), 0.5);
    }
    assert!(dist(f, C) < 1.0, "{}", dist(f, C));
}
