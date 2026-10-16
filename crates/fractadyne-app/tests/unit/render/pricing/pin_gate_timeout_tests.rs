//! The pin gate's wait for a completion (`pin_gate_timeout_us`, U3/U47, design §5.6).

use super::*;
use crate::tunables::{PIN_COMPLETION_TIMEOUT_US, PIN_TIMEOUT_FRAMES, SETTLE_COMPLETION_TIMEOUT_US};

#[test]
fn a_callback_that_was_never_registered_keeps_the_flat_escape() {
    // The headless harnesses: no event loop, no callback, so the wall clock is the only release.
    for dt in [0.0, 16.7, 400.0, f64::NAN] {
        assert_eq!(pin_gate_timeout_us(false, dt, true), PIN_COMPLETION_TIMEOUT_US);
    }
}

#[test]
fn quick_frames_keep_the_flat_floor() {
    // At 60 Hz four frames is 67 ms: the floor holds, so a healthy card waits exactly as before.
    assert_eq!(pin_gate_timeout_us(true, 16.7, true), PIN_COMPLETION_TIMEOUT_US);
}

#[test]
fn slow_frames_stretch_the_wait_with_them() {
    // The RX 6800 XT's slow passes stretch the frames around them; the gate waits past the pass.
    let want = (120.0 * PIN_TIMEOUT_FRAMES * 1000.0) as u64;
    assert!(want > PIN_COMPLETION_TIMEOUT_US);
    assert_eq!(pin_gate_timeout_us(true, 120.0, true), want);
}

#[test]
fn a_completion_later_than_the_settle_bound_is_not_waited_for() {
    assert_eq!(pin_gate_timeout_us(true, 5_000.0, true), SETTLE_COMPLETION_TIMEOUT_US);
}

#[test]
fn an_unreadable_interval_is_the_flat_floor() {
    for dt in [f64::NAN, f64::INFINITY, -3.0, 0.0] {
        assert_eq!(pin_gate_timeout_us(true, dt, true), PIN_COMPLETION_TIMEOUT_US);
    }
}

#[test]
fn switched_off_is_the_flat_timeout_always() {
    for (reg, dt) in [(true, 16.7), (true, 400.0), (false, 400.0)] {
        assert_eq!(pin_gate_timeout_us(reg, dt, false), PIN_COMPLETION_TIMEOUT_US);
    }
}
