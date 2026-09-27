//! The budget-blind tripwire: it must fire when the wall and the controller disagree for a run,
//! and stay quiet whenever the controller is actually being told about the slowness.

use super::*;

#[test]
fn it_fires_when_the_wall_is_slow_and_the_controller_heard_nothing() {
    assert!(budget_blind(BUDGET_BLIND_FRAMES, 0, false));
    assert!(budget_blind(BUDGET_BLIND_FRAMES + 50, 0, false));
}

#[test]
fn it_stays_quiet_while_the_controller_is_hearing_about_it() {
    // One slow reading is enough: the controller has the signal, whatever it decides to do.
    assert!(!budget_blind(BUDGET_BLIND_FRAMES, 1, false));
    assert!(!budget_blind(1000, 1, false));
}

#[test]
fn it_stays_quiet_on_a_short_run() {
    // A hitch is not an episode. A reference install or one expensive settle must not trip it.
    for n in 0..BUDGET_BLIND_FRAMES {
        assert!(!budget_blind(n, 0, false), "fired at {n} frames");
    }
}

#[test]
fn it_fires_once_per_episode() {
    assert!(!budget_blind(BUDGET_BLIND_FRAMES, 0, true));
    assert!(!budget_blind(10_000, 0, true));
}

/// Finding U15: only a SHRINK explains a slow run. Growth and "unchanged" do not.
#[test]
fn only_a_budget_decrease_restarts_the_count() {
    assert!(blind_reset_on(1_000, 500), "a shrink is the controller reacting");
    assert!(!blind_reset_on(1_000, 1_500), "growth is being told the opposite of the wall");
    assert!(!blind_reset_on(1_000, 1_000), "unchanged explains nothing");
}

/// The runaway U15 describes, replayed through both rules: every frame is wall-slow, and every
/// other frame a SHORT reading grows the budget ×1.5. Under the old rule ("reset whenever the
/// budget moved") the count never reached the threshold; under the new one the tripwire fires.
#[test]
fn a_budget_growing_while_the_wall_slows_now_trips_the_warning() {
    let run = |reset: fn(u64, u64) -> bool| {
        let (mut frames, readings, mut warned, mut budget) = (0u32, 0u32, false, 1_000_000u64);
        let mut fired = false;
        for f in 0..40 {
            frames += 1; // the wall called this frame slow
            if f % 2 == 1 {
                let next = budget + budget / 2; // a short reading: growth
                if reset(budget, next) {
                    frames = 0;
                    warned = false;
                }
                budget = next;
            }
            if budget_blind(frames, readings, warned) {
                warned = true;
                fired = true;
            }
        }
        fired
    };
    let old_rule: fn(u64, u64) -> bool = |cur, next| next != cur;
    assert!(!run(old_rule), "the old rule never warned — the bug, pinned");
    assert!(run(blind_reset_on), "the new rule warns during the runaway");
}

#[test]
fn the_dead_man_drops_a_high_budget_to_the_bootstrap_and_leaves_a_low_one_alone() {
    // The 2026-09-21 shape: a budget learned at 1.515e11 at a view whose frames it no longer fits.
    assert_eq!(dead_man_budget(151_500_000_000, 400_000_000), 400_000_000);
    // Already under the bootstrap: nothing to take away (the dead-man never RAISES a budget).
    assert_eq!(dead_man_budget(50_000_000, 400_000_000), 50_000_000);
    // Unmeasured stays unmeasured: its dispatches already size from the bootstrap.
    assert_eq!(dead_man_budget(0, 400_000_000), 0);
}
