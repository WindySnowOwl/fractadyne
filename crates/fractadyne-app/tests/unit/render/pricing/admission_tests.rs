//! B4's admission (design/live-render-robustness.md §5.1), shadow: `admit`, `budget_confident`, the
//! wall-clock cap's step, and the step count that cap prices (`key_change_steps`).

use super::*;

#[test]
fn admit_never_raises_the_learned_size() {
    for (learned, wall, boot, conf) in [
        (1_000u64, 0u64, 10_000u64, true),
        (1_000, 50_000, 10_000, true),
        (1_000, 50_000, 10_000, false),
        (5e10 as u64, 4e8 as u64, 4e8 as u64, false),
    ] {
        assert!(admit(learned, wall, boot, conf).0 <= learned, "{learned} {wall} {boot} {conf}");
    }
}

#[test]
fn no_wall_cap_bounds_nothing() {
    assert_eq!(admit(3e10 as u64, 0, 4e8 as u64, true), (3e10 as u64, AdmitBound::Learned));
}

#[test]
fn the_wall_cap_binds_when_it_is_lower() {
    assert_eq!(admit(3e10 as u64, 2e10 as u64, 4e8 as u64, true), (2e10 as u64, AdmitBound::Wall));
    assert_eq!(admit(1e10 as u64, 2e10 as u64, 4e8 as u64, true), (1e10 as u64, AdmitBound::Learned));
}

#[test]
fn an_unconfident_budget_admits_at_most_the_opening_guess() {
    assert_eq!(admit(3e10 as u64, 2e10 as u64, 4e8 as u64, false), (4e8 as u64, AdmitBound::Unconfident));
    // …but never more than the learned size: an opening guess above it is not a bound.
    assert_eq!(admit(1e8 as u64, 0, 4e8 as u64, false), (1e8 as u64, AdmitBound::Learned));
}

#[test]
fn confidence_is_lost_only_by_an_outstanding_dispatch_going_unread() {
    // Idle: the last reading is newer than the last dispatch.
    assert!(budget_confident(100, 100, 102, 500));
    // Dispatching every frame since 499 (the first after the reading at 498): still in flight…
    assert!(budget_confident(499, 510, 498, 499 + CONFIDENT_FRAMES - 1));
    // …and CONFIDENT_FRAMES after that first unread dispatch, however recent the newest.
    assert!(!budget_confident(499, 499 + CONFIDENT_FRAMES - 1, 498, 499 + CONFIDENT_FRAMES));
}

#[test]
fn the_first_dispatch_after_an_idle_hold_keeps_the_learned_budget() {
    // The last reading fed the budget at 100; the view sat idle until it dispatched at 400. Measured
    // from the reading, as it was until W6, the next frame was unconfident on the spot and the
    // opening guess bound it (34 such frames in three RTX 3080 runs, and not one unconfident frame
    // that had waited) — before the reading could land.
    assert!(budget_confident(400, 400, 100, 401));
    assert!(budget_confident(400, 410, 100, 400 + CONFIDENT_FRAMES - 1));
    assert!(!budget_confident(400, 410, 100, 400 + CONFIDENT_FRAMES));
}

#[test]
fn there_is_no_wall_cap_until_a_frame_runs_slow() {
    assert_eq!(wall_cap_step(0, 1e10 as u64, 30.0, false), 0);
    assert_eq!(wall_cap_step(0, 1e8 as u64, 399.0, false), 0);
}

#[test]
fn a_dispatch_over_the_target_but_inside_the_accept_window_sets_nothing() {
    // A settled pass is sized to land near 400 ms and its completion reads a frame or two late.
    let accept = crate::tunables::cost().tdr_latency_accept_ms;
    assert_eq!(wall_cap_step(0, 1e10 as u64, 450.0, false), 0);
    assert_eq!(wall_cap_step(0, 1e10 as u64, accept, false), 0);
    assert!(wall_cap_step(0, 1e10 as u64, accept + 1.0, false) > 0);
    // …and does not tighten an existing cap either.
    assert_eq!(wall_cap_step(4e9 as u64, 4e9 as u64, 500.0, false), 4e9 as u64);
}

#[test]
fn a_slow_frame_caps_below_the_dispatch_that_ran_slow() {
    let cap = wall_cap_step(0, 1e10 as u64, 1_200.0, false);
    assert!(cap > 0 && cap < 1e10 as u64, "{cap:e}");
}

#[test]
fn a_slow_frame_tightens_an_existing_cap() {
    let cap = wall_cap_step(5e9 as u64, 1e10 as u64, 1_200.0, false);
    assert!(cap < 5e9 as u64, "{cap:e}");
}

#[test]
fn a_fast_representative_frame_relaxes_the_cap_and_never_tightens_it() {
    let cap = wall_cap_step(4e9 as u64, 4e9 as u64, 100.0, false);
    assert!(cap > 4e9 as u64, "{cap:e}");
    assert!(wall_cap_step(4e9 as u64, 4e9 as u64, 390.0, false) >= 4e9 as u64);
}

#[test]
fn a_fast_dispatch_far_smaller_than_the_cap_says_nothing() {
    assert_eq!(wall_cap_step(4e9 as u64, 1e8 as u64, 5.0, false), 4e9 as u64);
}

#[test]
fn a_cap_relaxed_to_the_ceiling_is_dropped() {
    let ceil = crate::tunables::cost().tdr_steps_ceil;
    assert_eq!(wall_cap_step(ceil / 10 * 9, ceil / 10 * 9, 50.0, false), 0);
}

#[test]
fn an_unusable_reading_leaves_the_cap_alone() {
    for (steps, ms) in [(0u64, 3_000.0), (1e9 as u64, 0.0), (1e9 as u64, f64::NAN), (1e9 as u64, f64::INFINITY)] {
        assert_eq!(wall_cap_step(7e9 as u64, steps, ms, false), 7e9 as u64, "{steps} {ms}");
        assert_eq!(wall_cap_step(0, steps, ms, false), 0, "{steps} {ms}");
    }
}

#[test]
fn a_key_changed_settle_tile_is_priced_as_its_rect() {
    // The RX 6800 XT budget trap's first tile: 660×660 at ss 2 of a 1117×821 frame.
    let (spx, ss, iter) = (1117u64 * 821, 2u32, 4626u32);
    let tile = [228, 80, 660, 660];
    assert_eq!(key_change_steps(spx, ss, iter, Some(tile)), tile_steps(tile, ss, iter));
    assert_eq!(tile_steps(tile, ss, iter), 660 * 660 * 4 * 4626);
    assert!(key_change_steps(spx, ss, iter, Some(tile)) < key_change_steps(spx, ss, iter, None));
}

#[test]
fn an_untiled_or_held_key_change_is_priced_as_the_whole_frame() {
    let (spx, ss, iter) = (1117u64 * 821, 2u32, 4626u32);
    let whole = spx * 4 * 4626;
    assert_eq!(key_change_steps(spx, ss, iter, None), whole);
    assert_eq!(key_change_steps(spx, ss, iter, Some([0, 0, 0, 0])), whole);
    assert_eq!(key_change_steps(spx, ss, 0, None), spx * 4);
}
