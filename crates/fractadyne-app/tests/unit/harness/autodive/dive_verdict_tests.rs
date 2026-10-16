//! `--autodive`'s verdict, judged from the frame record (W8, design §7.8).
use super::{autodive_verdict, DiveTally, SLOW_AFTER_RETREAT_MAX};
use crate::harness::verdict::{PASS, RED, VACUOUS};

const LETHAL_MS: f64 = 900.0;

/// Feed `(interval ms, retreat code, lethal reading)` frames.
fn tally(frames: &[(f64, u8, bool)]) -> DiveTally {
    let mut t = DiveTally::default();
    for &(dt, retreat, lethal) in frames {
        t.feed(dt, retreat, lethal, LETHAL_MS);
    }
    t
}

#[test]
fn a_run_that_never_reached_the_regime_is_vacuous_not_a_pass() {
    // The 2026-08 RX 6800 XT run: 900 s to 1e112x with a 19.5 ms peak. Nothing was tested.
    let t = tally(&[(17.0, 0, false); 500]);
    assert_eq!(autodive_verdict(&t).0, VACUOUS);
}

#[test]
fn the_field_after_arm_passes() {
    // The beta.132 AFTER arms on PLUTO: a ~1,020 ms frame trips the lethal band and the retreat;
    // ONE more ~1,400 ms frame follows (its dispatch was already queued); then 92–105 ms.
    let mut f = vec![(330.0, 0, false), (1020.0, 1, true), (1400.0, 0, false)];
    f.extend([(100.0, 0, false); 50]);
    let t = tally(&f);
    assert_eq!(t.slow_run_after, 2);
    let (code, why) = autodive_verdict(&t);
    assert_eq!(code, PASS, "{why}");
}

#[test]
fn a_retreat_that_reached_nothing_is_red() {
    let t = tally(&[(1020.0, 2, true), (80.0, 0, false)]);
    let (code, why) = autodive_verdict(&t);
    assert_eq!(code, RED);
    assert!(why.contains("REACHED NOTHING"), "{why}");
}

#[test]
fn lethal_intervals_that_keep_coming_after_a_retreat_are_red() {
    // The budget came down on paper and the frames did not: the BEFORE arm's shape (the machine
    // hung after 640 ms, then 2,017 ms, with the budget still at 1.515e11).
    let mut f = vec![(1020.0, 1, true)];
    f.extend([(1500.0, 0, false); (SLOW_AFTER_RETREAT_MAX + 1) as usize]);
    let t = tally(&f);
    let (code, why) = autodive_verdict(&t);
    assert_eq!(code, RED, "{why}");
    assert!(why.contains("did not move"), "{why}");
    // …but the same intervals BEFORE any retreat are the regime being reached, not a failed retreat.
    let mut g = vec![(1500.0, 0, false); 3];
    g.extend([(1020.0, 1, true), (80.0, 0, false)]);
    assert_eq!(autodive_verdict(&tally(&g)).0, PASS);
}

#[test]
fn a_lethal_reading_with_no_retreat_at_all_is_red() {
    let t = tally(&[(1020.0, 0, true), (80.0, 0, false)]);
    assert_eq!(autodive_verdict(&t).0, RED);
}

#[test]
fn a_pass_judged_on_a_record_with_gaps_is_vacuous_but_a_red_stays_red() {
    let mut f = vec![(1020.0, 1, true)];
    f.extend([(80.0, 0, false); 10]);
    let mut t = tally(&f);
    assert_eq!(autodive_verdict(&t).0, PASS);
    t.lost = 5;
    let (code, why) = autodive_verdict(&t);
    assert_eq!(code, VACUOUS, "{why}");
    assert!(why.contains("LOST"), "{why}");
    // What WAS read failing is a failure whatever else went unread.
    let mut r = tally(&[(1020.0, 2, true)]);
    r.lost = 5;
    assert_eq!(autodive_verdict(&r).0, RED);
}
