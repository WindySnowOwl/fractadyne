//! The motion pacer's LIFT (U40): only a dispatch made while moving, bounded per lift.

use super::*;
use crate::tunables::MOTION_LIFT_MAX;

#[test]
fn one_lift_cannot_raise_the_motion_rate_past_the_bound() {
    // The RTX 3080 trap's one-frame lift was 350x.
    let cur = 1.4e7;
    assert_eq!(bounded_motion_lift(cur, 4.9e9), Some(cur * MOTION_LIFT_MAX));
    assert_eq!(bounded_motion_lift(cur, cur * 1.5), Some(cur * 1.5));
    assert_eq!(bounded_motion_lift(cur, cur * 0.5), None, "a lift never lowers");
    assert_eq!(bounded_motion_lift(0.0, 3e7), Some(3e7), "the first measurement is taken");
    assert_eq!(bounded_motion_lift(cur, f64::NAN), None);
    assert_eq!(bounded_motion_lift(cur, 0.0), None);
}

#[test]
fn repeated_lifts_still_recover_from_a_cut() {
    // Deadlock-proof: every fitting motion frame lifts, x2 each, so a 512x cut is undone in 9.
    let mut r = 1e6;
    for _ in 0..9 {
        r = bounded_motion_lift(r, 1e12).unwrap();
    }
    assert_eq!(r, 1e6 * MOTION_LIFT_MAX.powi(9));
}

#[test]
fn a_still_dispatch_followed_by_a_quick_frame_lifts_nothing() {
    let mut m = Meter::default();
    m.record_motion_rate(0, 2, false, 10.0, 100_000_000); // ~1e7 steps/ms
    let before = m.motion_rate_now(0, 2);
    // The trap: a 1.45e10-step settle tile, then a 3 ms frame.
    assert!(!m.motion_wall_cut(0, 2, 14_500_000_000, 3.0, false));
    assert_eq!(m.motion_rate_now(0, 2), before);
    // A motion pass that fit its frame lifts it, bounded.
    assert!(!m.motion_wall_cut(0, 2, 14_500_000_000, 3.0, true));
    assert_eq!(m.motion_rate_now(0, 2), before * MOTION_LIFT_MAX);
}
