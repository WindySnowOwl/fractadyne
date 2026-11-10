use super::*;

const PX: u64 = (SIDE as u64) * (SIDE as u64);

/// A card as the probe sees it: each pass costs a fixed part plus `ms_per_step` a nominal step.
fn pass(ms_per_step: f64, fixed_ms: f64, iters: u32) -> f64 {
    fixed_ms + ms_per_step * PX as f64 * f64::from(iters)
}

/// Drive the ramp over a modeled card; returns what it found and every pass time it ran.
fn ramp(ms_per_step: f64, fixed_ms: f64, fast: f64) -> (Finding, Vec<f64>) {
    let mut iters = ITERS_START;
    let mut ran = Vec::new();
    loop {
        let ms = pass(ms_per_step, fixed_ms, iters);
        ran.push(ms);
        match step(PX, iters, ms, fast).expect("a modeled time is usable") {
            Step::Done(f) => return (f, ran),
            Step::Next(n) => {
                assert!(n >= iters * 2 && n <= iters * 16, "{iters} → {n}");
                iters = n;
            }
        }
    }
}

#[test]
fn a_pass_over_the_target_is_a_measurement() {
    // Intel HD Graphics 4400 in floatexp (2026-11-10): ~7.0e-6 ms a step, 29 ms for the first pass.
    let ms = pass(7.0e-6, 0.2, ITERS_START);
    assert!(ms > PASS_TARGET_MS);
    match step(PX, ITERS_START, ms, 1e-7) {
        Some(Step::Done(Finding::Measured { iters, ms_per_step, .. })) => {
            assert_eq!(iters, ITERS_START);
            // The fixed part errs slow, the safe side, and barely at this length.
            assert!(ms_per_step >= 7.0e-6 && ms_per_step < 7.0e-6 * 1.01, "{ms_per_step}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn unusable_times_are_refused() {
    for ms in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(step(PX, 64, ms, 1e-7), None, "{ms}");
    }
    assert_eq!(step(0, 64, 5.0, 1e-7), None);
    assert_eq!(step(PX, 0, 5.0, 1e-7), None);
}

#[test]
fn the_threshold_is_where_nothing_would_change() {
    // The opening: 40 ms of the 4e8 historical guess.
    assert_eq!(fast_below(0.0, 40.0, 400_000_000), 1e-7);
    // A ceiling's table cost below it governs.
    assert_eq!(fast_below(1.92e-8, 40.0, 400_000_000), 1.92e-8);
    assert_eq!(fast_below(9.9e-8, 40.0, 400_000_000), 9.9e-8);
    // A table cost above it does not (the opening would still change).
    assert_eq!(fast_below(1e-6, 40.0, 400_000_000), 1e-7);
    assert_eq!(fast_below(f64::NAN, 40.0, 400_000_000), 1e-7);
}

#[test]
fn a_pass_is_timed_at_the_slower_clock() {
    assert_eq!(pass_ms([10.0, 12.0]), 12.0);
    assert_eq!(pass_ms([12.0, 10.0]), 12.0);
    assert_eq!(pass_ms([f64::NAN, 10.0]), 10.0);
    assert_eq!(pass_ms([0.0, 10.0]), 10.0);
}

/// ⭐The safety property: from a card 10× faster than an RTX 3080 to one 20× slower than Intel HD
/// Graphics 4400, no pass the ramp asks for costs more than the first one or ~2.5× the target —
/// the probe itself can never be the dispatch that trips a watchdog — and a card is either
/// measured to within its fixed cost or read as faster than anything it could change.
#[test]
fn no_card_gets_a_long_pass_and_every_card_gets_an_answer() {
    let mut cost = 5e-10;
    while cost < 2e-4 {
        for fast in [1.92e-8, 9.9e-8, 1e-7] {
            let (found, ran) = ramp(cost, 0.2, fast);
            let first = ran[0];
            for &ms in &ran[1..] {
                assert!(ms <= (2.5 * PASS_TARGET_MS).max(first), "cost {cost:e}: a {ms:.1} ms pass after {ran:?}");
            }
            match found {
                Finding::Measured { ms_per_step, .. } => {
                    assert!(ms_per_step >= cost, "cost {cost:e} read as {ms_per_step:e}");
                    assert!(ms_per_step <= cost * 1.1 || ms_per_step < fast * 1.1, "cost {cost:e} read as {ms_per_step:e}");
                }
                Finding::Fast { ms_per_step, .. } => {
                    assert!(cost < fast, "cost {cost:e} is not under {fast:e}");
                    assert!(ms_per_step >= cost && ms_per_step < fast);
                }
            }
        }
        cost *= 1.7;
    }
}

/// A fast card is done in a few small passes: the RTX 3080's direct cost (4.92e-9 ms/step) against
/// the default ceiling's 1.92e-8.
#[test]
fn a_fast_card_is_done_in_a_few_small_passes() {
    let (found, ran) = ramp(4.92e-9, 0.2, 1.92e-8);
    assert!(matches!(found, Finding::Fast { .. }), "{found:?}");
    assert!(ran.len() <= 3, "{ran:?}");
    assert!(ran.iter().sum::<f64>() < 5.0, "{ran:?}");
}
