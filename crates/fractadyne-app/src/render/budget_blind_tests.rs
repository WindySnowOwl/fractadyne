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
