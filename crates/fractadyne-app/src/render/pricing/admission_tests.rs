//! B4's admission (design/live-render-robustness.md §5.1), shadow: `admit`, `budget_confident` and
//! the always-on wall-clock budget's step.

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
fn an_unmeasured_wall_budget_bounds_nothing() {
    assert_eq!(admit(3e10 as u64, 0, 4e8 as u64, true), (3e10 as u64, AdmitBound::Learned));
}

#[test]
fn the_wall_budget_binds_when_it_is_lower() {
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
    assert!(budget_confident(100, 102, 500));
    // A dispatch whose reading is still in flight.
    assert!(budget_confident(500, 498, 500 + CONFIDENT_FRAMES - 1 - 2));
    // …and one that went CONFIDENT_FRAMES since the last reading without one.
    assert!(!budget_confident(500, 498, 498 + CONFIDENT_FRAMES));
}

#[test]
fn a_fast_dispatch_larger_than_the_wall_budget_proves_its_own_size() {
    let next = wall_budget_step(4e8 as u64, 1e10 as u64, 30.0, false);
    assert!(next >= 1e10 as u64, "{next:e}");
}

#[test]
fn an_unmeasured_wall_budget_starts_from_the_first_dispatch() {
    let next = wall_budget_step(0, 5e9 as u64, 100.0, false);
    assert!(next >= 5e9 as u64, "{next:e}");
}

#[test]
fn a_slow_dispatch_shrinks_the_wall_budget_below_its_own_size() {
    let next = wall_budget_step(3e10 as u64, 1e10 as u64, 1_200.0, false);
    assert!(next < 1e10 as u64, "{next:e}");
}

#[test]
fn a_fast_dispatch_far_smaller_than_the_wall_budget_says_nothing() {
    assert_eq!(wall_budget_step(3e10 as u64, 1e8 as u64, 5.0, false), 3e10 as u64);
}

#[test]
fn an_unusable_reading_leaves_the_wall_budget_alone() {
    for (steps, ms) in [(0u64, 30.0), (1e9 as u64, 0.0), (1e9 as u64, f64::NAN), (1e9 as u64, f64::INFINITY)] {
        assert_eq!(wall_budget_step(7e9 as u64, steps, ms, false), 7e9 as u64, "{steps} {ms}");
    }
}
