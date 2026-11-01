use super::*;

#[test]
fn a_card_slower_than_the_suite_is_sized_for_gets_the_guard() {
    // The RTX 3080's table cost and the default card's: priced only.
    assert!(!Policy::new(5.35e-9, false, true).enforce);
    assert!(!Policy::new(SIZED_FOR_MS_PER_STEP, false, false).enforce);
    // Slower: enforced, with or without an entry of its own.
    assert!(Policy::new(SIZED_FOR_MS_PER_STEP * 1.01, false, false).enforce);
    assert!(Policy::new(4.2e-6, false, true).enforce);
    // A simulation always enforces.
    assert!(Policy::new(1.0e-9, true, false).enforce);
    // A cost that is not a positive number cannot show the card is fast enough.
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(Policy::new(bad, false, true).enforce, "{bad}");
    }
}

#[test]
fn a_skip_fails_the_run_only_on_a_card_with_its_own_entry() {
    assert!(skips_fail(true, false));
    assert!(!skips_fail(false, false));
    // A preview of a slower card's run is not this card's verdict.
    assert!(!skips_fail(true, true));
    assert!(!skips_fail(false, true));
    // A simulation previews by default.
    let p = Policy::new(4.2e-6, true, true);
    assert!(p.simulated && p.preview && p.enforce);
    assert!(!Policy::new(4.2e-6, false, true).preview);
}

#[test]
fn the_simulation_reads_a_cost_and_an_optional_gate() {
    assert_eq!(parse_simulation("4.2e-6"), Ok(Simulation { ms_per_step: 4.2e-6, gate: false }));
    assert_eq!(parse_simulation(" 4.2e-6,gate "), Ok(Simulation { ms_per_step: 4.2e-6, gate: true }));
    // Unreadable is an error, never a silent default.
    for bad in ["", "fast", "0", "-1", "nan", "inf", "4.2e-6,gat", "4.2e-6,"] {
        assert!(parse_simulation(bad).is_err(), "{bad:?}");
    }
}

#[test]
fn the_heaviest_mode_prices_highest() {
    assert_eq!(mode_ratio(1), 1.0);
    assert!(mode_ratio(0) > mode_ratio(1));
    assert!(mode_ratio(2) > mode_ratio(0));
    // An unknown mode is priced as the dearest.
    assert_eq!(mode_ratio(7), mode_ratio(2));
}

#[test]
fn the_crash_case_is_refused_on_the_gt_710_and_admitted_on_the_3080() {
    // 2026-11-01: "home 1x, 50k iter" is one 220x220 pass of 50,000 iterations in direct mode, and
    // the GT 710 ran it past 2 s (so its direct step costs at least ~4e-6 ms).
    let steps = 220.0 * 220.0 * 50_000.0;
    let gt710 = Policy::new(4.2e-6, false, false);
    assert!(gt710.enforce);
    assert!(steps * gt710.ms_per_step * mode_ratio(1) > LIMIT_MS);
    let rtx3080 = Policy::new(5.35e-9, false, true);
    assert!(!rtx3080.enforce);
    assert!(steps * rtx3080.ms_per_step * mode_ratio(1) < LIMIT_MS);
}

#[test]
fn a_step_cost_is_read_from_the_slowest_wall() {
    let (ms, worst) = step_cost(&[10.0, 30.0, 20.0], 1000, 10).unwrap();
    assert_eq!(worst, 30.0);
    assert!((ms - 30.0 / 10_000.0).abs() < 1e-15);
    // Walls that are not positive numbers are not readings.
    assert_eq!(step_cost(&[f64::NAN, 0.0, -1.0], 1000, 10), None);
    assert_eq!(step_cost(&[], 1000, 10), None);
    let (_, worst) = step_cost(&[f64::NAN, 5.0], 1000, 10).unwrap();
    assert_eq!(worst, 5.0);
}
