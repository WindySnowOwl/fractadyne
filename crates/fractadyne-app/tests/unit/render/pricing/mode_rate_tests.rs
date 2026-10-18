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

#[test]
fn an_opening_rate_prices_only_the_opening_and_only_until_the_mode_measures() {
    let fe = 2; // floatexp
    let mut m = Meter::default();
    m.record_mode_rate(0, 0, false, 100.0, 2_700_000_000, 1.0); // df32: 2.7e7 steps/ms
    // Floatexp unmeasured opens at df32's rate over the margin.
    let unmeasured = m.bootstrap_steps(0, fe);
    m.record_opening_rate(0, fe, 2.5e6);
    assert!(m.has_opening_rate(0, fe));
    assert_eq!(m.bootstrap_steps(0, fe), bootstrap_steps(2.5e6, 0.0));
    assert!(m.bootstrap_steps(0, fe) > unmeasured);
    // It is not a reading: the rates the passes are sized from know nothing of it.
    assert_eq!(m.worst_rate_steps_per_ms(0, fe), None);
    assert_eq!(m.motion_rate, Meter::default().motion_rate);
    // The mode's first reading replaces it.
    m.record_mode_rate(0, fe, false, 100.0, 1_800_000_000, 2.0); // 1.8e7
    assert_eq!(m.bootstrap_steps(0, fe), bootstrap_steps(1.8e7, 0.0));
    // The other view knows nothing of it.
    assert!(!m.has_opening_rate(1, fe));
}
