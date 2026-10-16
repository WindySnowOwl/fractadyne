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
    m.record_motion_rate(0, 2, false, 10.0, 100_000_000, false, 0, 0); // ~1e7 steps/ms
    let before = m.motion_rate_now(0, 2);
    // The trap: a 1.45e10-step settle tile, then a 3 ms frame.
    assert!(!m.motion_wall_cut(0, 2, 14_500_000_000, 3.0, false, false, 0, 0));
    assert_eq!(m.motion_rate_now(0, 2), before);
    // A motion pass that fit its frame lifts it, bounded.
    assert!(!m.motion_wall_cut(0, 2, 14_500_000_000, 3.0, true, false, 0, 0));
    assert_eq!(m.motion_rate_now(0, 2), before * MOTION_LIFT_MAX);
}

#[test]
fn a_pin_pass_never_moves_the_rate_a_whole_moving_frame_is_sized_from() {
    // W8 rung 1c, the RX 6800 XT's 1x glide: whole moving frames read 5.6e7 nominal steps/ms, pin
    // passes a median 9.9e8.
    let mut m = Meter::default();
    m.record_motion_rate(0, 0, false, 115.0, 6_460_000_000, false, 0, 0);
    let frame = m.frame_rate_now(0, 0);
    // (The fixed cost is the process's calibration, which another test may have set.)
    let expect = 6.46e9 / crate::render::variable_pass_ms(115.0, crate::calibration::pass_fixed_ms());
    assert!((frame - expect).abs() / frame < 1e-9, "{frame} vs {expect}");
    for _ in 0..50 {
        m.record_motion_rate(0, 0, false, 1.0, 1_000_000_000, true, 0, 0);
    }
    assert_eq!(m.frame_rate_now(0, 0), frame, "50 pin readings left it alone");
    assert!(m.motion_rate_now(0, 0) > 10.0 * frame, "the pin pacing still learns from pins");
    // The lift: a pin pass that fit its frame lifts the motion rate, never the frame rate…
    assert!(!m.motion_wall_cut(0, 0, 10_000_000_000, 17.0, true, true, 0, 0));
    assert_eq!(m.frame_rate_now(0, 0), frame);
    // …and a whole moving frame that fit lifts its own.
    assert!(!m.motion_wall_cut(0, 0, 1_000_000_000, 17.0, true, false, 0, 0));
    assert!(m.frame_rate_now(0, 0) > frame);
}

#[test]
fn a_frame_below_the_knee_reads_the_rate_a_full_frame_does() {
    // W8 rung 1c: the RTX 3080 (knee 262,144 px). A full 800,812-px frame and a 15,950-px one at the
    // same 25,000 iterations: below the knee the small one costs like the knee's pixels, ~13.6 ms
    // against ~41.6. Counted at the knee, both read the same rate — uncorrected the small one read
    // 16× slower, sized the next frame smaller, and the moving frames spiralled to 145×110.
    const KNEE: u64 = 262_144;
    let eff = 4.8e8; // steps per ms, at or above the knee
    let full = (800_812u64, 800_812u64 * 25_000);
    let small = (15_950u64, 15_950u64 * 25_000);
    let ms = |px: u64, steps: u64| knee_steps(steps, px, KNEE) / eff;
    let mut a = Meter::default();
    a.record_motion_rate(0, 0, false, ms(full.0, full.1), full.1, false, full.0, KNEE);
    let mut b = Meter::default();
    b.record_motion_rate(0, 0, false, ms(small.0, small.1), small.1, false, small.0, KNEE);
    let (ra, rb) = (a.frame_rate_now(0, 0), b.frame_rate_now(0, 0));
    assert!((ra - rb).abs() / ra < 0.05, "full {ra:.3e} small {rb:.3e}");
    // Uncorrected the small frame reads ~16x slower.
    let mut c = Meter::default();
    c.record_motion_rate(0, 0, false, ms(small.0, small.1), small.1, false, 0, 0);
    assert!(c.frame_rate_now(0, 0) < ra / 10.0);
    // Above the knee nothing changes, and the pin rate never takes the correction.
    assert_eq!(knee_steps(full.1, full.0, KNEE), full.1 as f64);
    assert_eq!(knee_steps(small.1, 0, KNEE), small.1 as f64, "unknown size: as given");
    assert!(b.motion_rate_now(0, 0) < ra / 10.0, "motion_rate stays nominal");
}
