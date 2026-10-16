//! `mode_rate_step` and `Meter::record_mode_rate`: the per-mode pessimistic rate recovers slowly.

use super::*;
use crate::tunables::MODE_RATE_DOUBLING_S;

#[test]
fn the_first_reading_is_adopted() {
    assert_eq!(mode_rate_step(0.0, 3e7, 0.0), 3e7);
}

#[test]
fn a_slower_reading_is_adopted_at_once() {
    assert_eq!(mode_rate_step(4e7, 1.5e6, 0.0), 1.5e6);
    assert_eq!(mode_rate_step(4e7, 1.5e6, 1_000.0), 1.5e6);
}

#[test]
fn a_faster_reading_raises_it_at_most_twofold_per_doubling_time() {
    let cur = 1.5e6;
    assert_eq!(mode_rate_step(cur, 4e7, 0.0), cur);
    let one = mode_rate_step(cur, 4e7, MODE_RATE_DOUBLING_S);
    assert!((one - 2.0 * cur).abs() < 1.0, "{one}");
    // …and never past the reading itself.
    assert_eq!(mode_rate_step(cur, 2e6, 100.0 * MODE_RATE_DOUBLING_S), 2e6);
}

#[test]
fn readings_compound_to_the_same_recovery_however_often_they_come() {
    // One reading after 10 s, or 200 readings 50 ms apart: the same ×2.
    let mut fine = 1.5e6;
    for _ in 0..200 {
        fine = mode_rate_step(fine, 4e7, MODE_RATE_DOUBLING_S / 200.0);
    }
    assert!((fine / 3e6 - 1.0).abs() < 1e-9, "{fine}");
}

#[test]
fn the_meter_keeps_a_slow_reading_and_lets_it_go_slowly() {
    let mut m = Meter::default();
    m.record_mode_rate(0, 0, false, 100.0, 4_000_000_000, 1.0); // 4e7 steps/ms
    m.record_mode_rate(0, 0, false, 5.0, 7_500_000, 2.0); // 1.5e6: a storm pass
    assert_eq!(m.worst_rate_steps_per_ms(0, 0), Some(1.5e6));
    m.record_mode_rate(0, 0, false, 100.0, 4_000_000_000, 2.0 + MODE_RATE_DOUBLING_S);
    let r = m.worst_rate_steps_per_ms(0, 0).unwrap();
    assert!((r / 3e6 - 1.0).abs() < 1e-9, "{r}");
    // The other view's slot is its own.
    assert_eq!(m.worst_rate_steps_per_ms(1, 0), None);
}
