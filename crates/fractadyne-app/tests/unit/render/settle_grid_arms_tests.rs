use super::settle_grid_arms;

/// ⛔The state a user's view sat in for five minutes (2026-11-06, frame records): a Multibrot 7
/// orbit-trap view, auto iterations, its budget cut to 2.13e7 by the wall-clock dead-man and not
/// converged, no grid — at 16x16, counted complete, nothing dispatched. An orbit-trap frame cannot
/// walk, so tiles are its only way back to native resolution: it must arm.
#[test]
fn a_frame_that_cannot_walk_arms_before_its_budget_converges() {
    assert!(settle_grid_arms(false, 21_305_289, true, false));
}

/// Unchanged: an auto-iteration view that CAN walk waits for its budget, so tiles do not starve the
/// adaptive boost of whole-frame readings (`--livetest`: the 1e61 hold 42.1% black → 100% when it
/// did not).
#[test]
fn an_auto_view_that_can_walk_still_waits_for_convergence() {
    assert!(!settle_grid_arms(false, 400_000_000, true, true));
}

/// Unchanged: converged, unmeasured, or the user's own iteration count all arm.
#[test]
fn the_cases_that_always_armed_still_do() {
    assert!(settle_grid_arms(true, 5_911_567_476, true, true));
    assert!(settle_grid_arms(false, 0, true, true));
    assert!(settle_grid_arms(false, 400_000_000, false, true));
}
