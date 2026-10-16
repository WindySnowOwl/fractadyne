use super::*;

/// The measured anchors from the backlog item ("the solver now reaches depths the renderer
/// does not"): the e500 hero resolves at 200,000 iterations; ~1e2000 is solid black at
/// 200,000 and resolves at 1,000,000. The classifier must flag exactly the black cases.
#[test]
fn the_measured_depth_anchors_are_classified_correctly() {
    let e500 = 500.0 * 10f64.log2();
    let e2000 = 2000.0 * 10f64.log2();
    assert_eq!(
        deep_jump_iter_shortfall(e500, 200_000, false),
        None,
        "200k at e500 renders the hero — no warning"
    );
    let starved = deep_jump_iter_shortfall(e2000, 200_000, false);
    assert!(starved.is_some(), "200k at e2000 was measured solid black");
    let (have, typical) = starved.unwrap();
    assert_eq!(have, 200_000);
    assert!(
        typical >= 1_000_000,
        "the suggestion must be a known-good count (1M resolves at e2000; suggested {typical})"
    );
    assert_eq!(
        deep_jump_iter_shortfall(e2000, 1_000_000, false),
        None,
        "1M at e2000 resolves the spiral — no warning"
    );
}

/// Auto-iteration's budget follows the jump the way it follows a hand zoom, so it must
/// never warn — at any depth, with any base count.
#[test]
fn auto_iterations_never_warn() {
    for l2 in [10.0, 2_000.0, 200_000.0] {
        assert_eq!(deep_jump_iter_shortfall(l2, 100, true), None);
    }
}

/// Below the engagement floor a low explicit count is an ordinary choice (fast escapes
/// dominate shallow views), not the flat-frame trap — a 1e6× view with 1,000 iterations
/// must not nag.
#[test]
fn shallow_views_are_never_nagged() {
    assert_eq!(deep_jump_iter_shortfall(1.0e6f64.log2(), 1_000, false), None);
    assert_eq!(deep_jump_iter_shortfall(499.0, 1, false), None, "just under the floor");
}

/// A non-finite target (unparsed/absurd zoom field) is not a warning, it is nothing.
#[test]
fn a_broken_target_is_silent() {
    assert_eq!(deep_jump_iter_shortfall(f64::NAN, 100, false), None);
    assert_eq!(deep_jump_iter_shortfall(f64::INFINITY, 100, false), None);
}
