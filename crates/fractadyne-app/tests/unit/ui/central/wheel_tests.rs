use super::*;

/// Octaves a zoom factor amounts to (+ = in).
fn octaves(factor: f64) -> f64 {
    -factor.log2()
}

/// Feed `n` notches a second for `secs` seconds at 60 frames a second, and return the rate.
fn steady(n: f64, secs: f64) -> f64 {
    let dt = 1.0 / 60.0;
    let mut rate = 0.0;
    for _ in 0..(secs / dt) as usize {
        rate = wheel_rate_step(rate, dt, n * dt);
    }
    rate
}

#[test]
fn a_notch_at_rest_zooms_the_resting_step_up_in_and_down_out() {
    let notch = WHEEL_POINTS_PER_NOTCH;
    assert!((octaves(wheel_zoom_factor(notch)) - WHEEL_OCT_PER_NOTCH).abs() < 1e-12, "up = in");
    assert!((octaves(wheel_zoom_factor(-notch)) + WHEEL_OCT_PER_NOTCH).abs() < 1e-12, "down = out");
    assert_eq!(wheel_zoom_factor(0.0), 1.0);
    // Faster than the old curve (exp(−0.0015 × 40): 0.087 octave a notch), and a notch is still a
    // small step: about 6.7 of them to double.
    assert!(WHEEL_OCT_PER_NOTCH > 0.0015 * 40.0 / std::f64::consts::LN_2);
    assert!(WHEEL_OCT_PER_NOTCH < 0.25);
}

#[test]
fn the_rate_reads_a_steady_wheel_in_notches_a_second() {
    for n in [2.0, 8.0, 15.0] {
        let r = steady(n, 3.0);
        assert!((r - n).abs() < 0.05 * n, "{n} notches a second read {r}");
    }
    // It forgets: a second after the wheel stops, little remains.
    let mut r = steady(10.0, 2.0);
    for _ in 0..60 {
        r = wheel_rate_step(r, 1.0 / 60.0, 0.0);
    }
    assert!(r < 0.5, "{r}");
}

#[test]
fn a_slow_wheel_keeps_the_resting_step_and_a_fast_one_accelerates() {
    // A single click (1/τ ≈ 3.3 a second the moment it lands) or a slow turn: no acceleration.
    assert_eq!(wheel_gain(wheel_rate_step(0.0, 0.0, 1.0)), 1.0);
    assert_eq!(wheel_gain(steady(3.0, 2.0)), 1.0);
    // Faster: more per notch, rising smoothly to the cap and no further.
    let g8 = wheel_gain(steady(8.0, 2.0));
    assert!(g8 > 1.8 && g8 < 2.2, "{g8}");
    assert!((wheel_gain(WHEEL_ACCEL_FROM + WHEEL_ACCEL_SPAN) - WHEEL_ACCEL_MAX).abs() < 1e-12);
    assert_eq!(wheel_gain(100.0), WHEEL_ACCEL_MAX);
    let mut last = 0.0;
    for r in 0..40 {
        let g = wheel_gain(r as f64 * 0.5);
        assert!(g >= last, "monotone");
        last = g;
    }
    // Direction does not matter to the gain.
    assert_eq!(wheel_gain(-8.0), wheel_gain(8.0));
}

#[test]
fn reversing_the_wheel_drops_the_acceleration() {
    let fast_in = steady(12.0, 1.0);
    assert!(wheel_gain(fast_in) > 2.5);
    // One notch the other way: the count starts over from that notch alone.
    let r = wheel_rate_step(fast_in, 1.0 / 60.0, -1.0);
    assert!(r < 0.0 && wheel_gain(r) == 1.0, "{r}");
}

#[test]
fn zoom_speed_at_a_few_wheel_rates() {
    // Octaves a second for a steady N notches a second: N × step × gain. Recorded so a change
    // to the constants shows up here as numbers. The old curve gave 0.087 × N throughout.
    let speed = |n: f64| n * WHEEL_OCT_PER_NOTCH * wheel_gain(n);
    assert!((speed(3.0) - 0.45).abs() < 1e-9);
    assert!((speed(8.0) - 2.4).abs() < 1e-9);
    assert!((speed(12.0) - 5.4).abs() < 1e-9);
}
