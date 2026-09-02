use super::*;

/// No anchors = no mapping: the caller falls back to the legacy EMA, never to a made-up range.
#[test]
fn no_anchors_yields_none() {
    assert_eq!(norm_anchor_range(&[], 1.0), None);
}

/// One anchor holds for the whole tour — before, at, and after its time.
#[test]
fn a_single_anchor_is_constant() {
    let a = [(2.0, (10.0, 50.0))];
    for t in [0.0, 2.0, 9.0] {
        assert_eq!(norm_anchor_range(&a, t), Some((10.0, 50.0)));
    }
}

/// Between two anchors the range is the straight line through them; the endpoints are hit
/// exactly (a frame landing on a keyframe uses that keyframe's canonical range verbatim).
#[test]
fn two_anchors_interpolate_linearly_and_hit_the_endpoints() {
    let a = [(0.0, (0.0, 100.0)), (4.0, (40.0, 300.0))];
    assert_eq!(norm_anchor_range(&a, 0.0), Some((0.0, 100.0)));
    assert_eq!(norm_anchor_range(&a, 4.0), Some((40.0, 300.0)));
    assert_eq!(norm_anchor_range(&a, 2.0), Some((20.0, 200.0)));
    // Clamped outside the anchored span — the tour cannot ask for a time it doesn't have,
    // but a rounding edge must not extrapolate.
    assert_eq!(norm_anchor_range(&a, -1.0), Some((0.0, 100.0)));
    assert_eq!(norm_anchor_range(&a, 9.0), Some((40.0, 300.0)));
}

/// Two anchors at the same instant (a zero-length segment) must not divide by zero — the
/// earlier anchor wins up to the shared time.
#[test]
fn coincident_anchor_times_do_not_nan() {
    let a = [(1.0, (0.0, 10.0)), (1.0, (100.0, 200.0))];
    let (lo, hi) = norm_anchor_range(&a, 1.0).unwrap();
    assert!(lo.is_finite() && hi.is_finite());
}
