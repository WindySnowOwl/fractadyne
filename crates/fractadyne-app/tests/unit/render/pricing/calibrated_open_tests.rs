//! The calibrated opening and climb (2026-11-18): `calibrated_climb`, and `Perf::bootstrap_steps`
//! raised to the opening `build_params` stamps for the view's mode.

use super::*;

const T: f64 = crate::tunables::TDR_BUDGET_MS_DEFAULT;

/// A reading far under the target grows the budget up to ×4, where `budget_step` alone walks ×1.5.
#[test]
fn a_reading_far_under_the_target_climbs_up_to_fourfold() {
    let cur = 400_000_000;
    let (next, _) = budget_step(cur, cur, 0.6, false).expect("a full-size reading prices");
    assert_eq!(next, 600_000_000, "budget_step's own walk is ×1.5");
    assert_eq!(calibrated_climb(cur, next, 0.6, false), 1_600_000_000);
}

/// The climb lands no further than `target / CLIMB_HEADROOM` by the reading's own rate: between
/// ×1.5 and ×4 it takes the factor that predicts exactly that.
#[test]
fn the_climb_stops_at_its_headroom() {
    let cur = 1_000_000_000_u64;
    let ms = T / CLIMB_HEADROOM / 2.5; // 20 ms: ×2.5 predicts 50 ms
    let (next, _) = budget_step(cur, cur, ms, false).unwrap();
    let grown = calibrated_climb(cur, next, ms, false);
    assert_eq!(grown, 2_500_000_000);
    assert!(grown as f64 / cur as f64 * ms <= T / CLIMB_HEADROOM + 1e-9);
}

/// Near the target the reading's own ×1.5 (or less) stands: the climb never adds to it.
#[test]
fn near_the_target_the_walk_is_unchanged() {
    let cur = 1_000_000_000;
    for ms in [T / CLIMB_HEADROOM, 100.0, 300.0, 390.0] {
        let (next, _) = budget_step(cur, cur, ms, false).unwrap();
        assert_eq!(calibrated_climb(cur, next, ms, false), next, "{ms} ms");
    }
}

/// Only growth is touched: a shrink, a hold and an unusable time pass through unchanged.
#[test]
fn it_never_shrinks_holds_or_reads_a_bad_time() {
    let cur = 1_000_000_000;
    assert_eq!(calibrated_climb(cur, cur / 2, 0.5, false), cur / 2, "a shrink stays a shrink");
    assert_eq!(calibrated_climb(cur, cur, 0.5, false), cur, "a hold stays a hold");
    for ms in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(calibrated_climb(cur, cur + 1, ms, false), cur + 1, "{ms}");
    }
}

/// Never past the regime's own ceiling, like `budget_step`.
#[test]
fn it_never_passes_the_regimes_ceiling() {
    let c = crate::tunables::cost();
    let cur = c.tdr_steps_ceil / 2;
    assert_eq!(calibrated_climb(cur, cur + 1, 0.1, false), c.tdr_steps_ceil);
    let cur = c.explicit_steps_ceil / 2;
    assert_eq!(calibrated_climb(cur, cur + 1, 0.1, true), c.explicit_steps_ceil);
}

/// ⭐The maintainer's df32 entry (2026-11-18, RTX 3080): readings at ~6.7e8 steps/ms against a
/// ceiling of 1.449e10. From the calibrated opening (1.449e9) the climb reaches the ceiling in two
/// readings; from the old 4e8 cap at ×1.5 it took nine — at one reading every 0.3–3 s, the 6.5 s blur.
#[test]
fn the_sessions_df32_entry_reaches_the_ceiling_in_two_readings_not_nine() {
    const RATE: f64 = 4.0e8 / 0.6;
    const CEIL: u64 = 14_490_000_000;
    let readings = |mut cur: u64, climb: bool| {
        let mut n = 0;
        while cur < CEIL {
            let ms = cur as f64 / RATE;
            let (next, _) = budget_step(cur, cur, ms, false).unwrap();
            cur = if climb { calibrated_climb(cur, next, ms, false) } else { next };
            n += 1;
            assert!(n < 50, "no progress");
        }
        n
    };
    assert_eq!(readings(1_449_000_000, true), 2);
    assert_eq!(readings(400_000_000, false), 9);
}

/// `Perf::bootstrap_steps` takes the larger of the rate-derived guess and the calibrated opening —
/// only when the opening was stamped for the view's CURRENT mode; the dead-man's guess never moves.
#[test]
fn the_opening_is_raised_only_for_the_mode_it_was_stamped_for() {
    let mut p = crate::Perf::default();
    let guess = p.bootstrap_steps_measured(0);
    assert_eq!(p.bootstrap_steps(0), guess, "nothing stamped: the guess");
    assert!(!p.calibrated_mode(0));

    p.budget_mode[0] = 0;
    p.calibrated_open[0] = (0, guess * 4);
    assert_eq!(p.bootstrap_steps(0), guess * 4, "stamped for this mode: the opening");
    assert_eq!(p.bootstrap_steps_measured(0), guess, "the dead-man's target does not move");
    assert!(p.calibrated_mode(0));
    assert_eq!(p.bootstrap_steps(1), p.bootstrap_steps_measured(1), "the other view is its own");

    p.calibrated_open[0] = (0, guess / 4);
    assert_eq!(p.bootstrap_steps(0), guess, "a smaller opening never lowers the guess");

    // Stamped for another mode (the frame before a switch): neither the opening nor the climb.
    p.calibrated_open[0] = (1, guess * 4);
    assert_eq!(p.bootstrap_steps(0), guess);
    assert!(!p.calibrated_mode(0));

    // Stamped with none (floatexp, an uncalibrated card): the climb is not licensed.
    p.calibrated_open[0] = (0, 0);
    assert!(!p.calibrated_mode(0));
}

/// A reading of a pass dispatched before the view's last mode switch prices nothing of the new mode
/// (the maintainer's df32 entry at frame 712 took direct's frame-711 reading); the switch frame's own
/// pass, and every later one, do; no pass frame, or no switch yet, is never "before".
#[test]
fn a_reading_of_the_old_modes_pass_is_before_the_switch() {
    assert!(reading_before_switch(711, 712), "direct's last pass, read after the switch");
    assert!(!reading_before_switch(712, 712), "the switch frame's pass is the new mode's");
    assert!(!reading_before_switch(715, 712));
    assert!(!reading_before_switch(0, 712), "a reading that names no frame");
    assert!(!reading_before_switch(711, u64::MAX), "no switch yet");
}

/// The cross-mode opening raises the guess only for the mode it was stamped for and only while that
/// mode's budget is unmeasured: once a reading lands it sizes nothing, the admission and jam bounds
/// included.
#[test]
fn the_cross_mode_opening_counts_only_until_the_new_mode_measures() {
    let mut p = crate::Perf::default();
    p.budget_mode[0] = 0;
    let base = p.bootstrap_steps(0);
    p.cross_open[0] = (0, base * 10);
    assert_eq!(p.bootstrap_steps(0), base * 10, "unmeasured: the cross-mode opening");
    p.fe_budget[0] = 123_456_789;
    assert_eq!(p.bootstrap_steps(0), base, "measured: it no longer counts");
    p.fe_budget[0] = 0;
    p.cross_open[0] = (1, base * 10);
    assert_eq!(p.bootstrap_steps(0), base, "stamped for another mode");
    assert_eq!(p.bootstrap_steps_measured(0), p.meter.bootstrap_steps(0, 0), "the dead-man's guess never moves");
}
