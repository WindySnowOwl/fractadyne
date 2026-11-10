use super::*;

const PX: u64 = (SIDE as u64) * (SIDE as u64);

/// A card as the probe sees it: each pass costs the GPU a fixed part plus `ms_per_step` a nominal
/// step, and the wall adds `latency_ms` per submission. `ts_scale` < 1 models a GPU clock that
/// reads low (0 = the card gives no timestamp).
#[derive(Clone, Copy)]
struct Card {
    ms_per_step: f64,
    fixed_ms: f64,
    latency_ms: f64,
    ts_scale: f64,
}

impl Card {
    fn honest(ms_per_step: f64) -> Self {
        Card { ms_per_step, fixed_ms: 0.2, latency_ms: 0.3, ts_scale: 1.0 }
    }
    /// `[timestamp ms, wall ms]` of one pass at `iters`, as `render_iter_probe` reports it.
    fn pass(self, iters: u32) -> [f64; 2] {
        let gpu = self.fixed_ms + self.ms_per_step * PX as f64 * f64::from(iters);
        let ts = if self.ts_scale > 0.0 { gpu * self.ts_scale } else { f64::NAN };
        [ts, gpu + self.latency_ms]
    }
}

/// Drive the ramp over a modeled card; returns what it found and every pass's real GPU time.
fn ramp(card: Card, fast: f64) -> (Finding, Vec<f64>) {
    let mut iters = ITERS_START;
    let mut ran = Vec::new();
    loop {
        let t = card.pass(iters);
        ran.push(t[1] - card.latency_ms);
        match step(PX, iters, gpu_ms(t), t[1], fast).expect("a modeled time is usable") {
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
    // Intel HD Graphics 4400 in floatexp (2026-11-10): ~1.15e-5 ms a step, 48 ms for the first pass.
    let t = Card::honest(1.15e-5).pass(ITERS_START);
    match step(PX, ITERS_START, gpu_ms(t), t[1], 1e-7) {
        Some(Step::Done(Finding::Measured { iters, ms_per_step, .. })) => {
            assert_eq!(iters, ITERS_START);
            // The fixed part errs slow, the safe side, and barely at this length.
            assert!(ms_per_step >= 1.15e-5 && ms_per_step < 1.15e-5 * 1.01, "{ms_per_step}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn unusable_times_are_refused() {
    for ms in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(step(PX, 64, ms, 5.0, 1e-7), None, "gpu {ms}");
        assert_eq!(step(PX, 64, 5.0, ms, 1e-7), None, "wall {ms}");
    }
    assert_eq!(step(0, 64, 5.0, 5.0, 1e-7), None);
    assert_eq!(step(PX, 0, 5.0, 5.0, 1e-7), None);
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
fn a_pass_is_timed_by_the_gpu_clock_where_it_fits_the_wall() {
    // PLUTO's RTX 3070 (dev.8): ~12 ms of submission latency around ~1.9 ms of GPU work.
    assert_eq!(gpu_ms([1.9, 13.9]), 1.9);
    // No timestamp, or one that cannot be this pass's (longer than its wall): the wall.
    assert_eq!(gpu_ms([f64::NAN, 10.0]), 10.0);
    assert_eq!(gpu_ms([0.0, 10.0]), 10.0);
    assert_eq!(gpu_ms([30.0, 10.0]), 10.0);
    // Within the clocks' resolution of the wall is still this pass.
    assert_eq!(gpu_ms([10.3, 10.0]), 10.3);
}

/// ⭐The safety property: from a card 10× faster than an RTX 3080 to one 20× slower than Intel HD
/// Graphics 4400, no pass the ramp asks for costs the GPU more than the first one or ~2.5× the
/// target — the probe itself can never be the dispatch that trips a watchdog — and a card is either
/// measured to within its fixed cost or read as faster than anything it could change. With and
/// without a timestamp, and with a submission latency of 0.3 or 12 ms.
#[test]
fn no_card_gets_a_long_pass_and_every_card_gets_an_answer() {
    let mut cost = 5e-10;
    while cost < 2e-4 {
        for fast in [1.92e-8, 9.9e-8, 1e-7] {
            for (latency_ms, ts_scale) in [(0.3, 1.0), (12.0, 1.0), (0.3, 0.0)] {
                let card = Card { latency_ms, ts_scale, ..Card::honest(cost) };
                let (found, ran) = ramp(card, fast);
                let first = ran[0];
                for &ms in &ran[1..] {
                    assert!(ms <= (2.5 * PASS_TARGET_MS).max(first), "cost {cost:e}: a {ms:.1} ms pass after {ran:?}");
                }
                match found {
                    Finding::Measured { ms_per_step, .. } => {
                        assert!(ms_per_step >= cost, "cost {cost:e} read as {ms_per_step:e}");
                        if ts_scale > 0.0 {
                            assert!(
                                ms_per_step <= cost * 1.1 || ms_per_step < fast * 1.1,
                                "cost {cost:e} read as {ms_per_step:e} (latency {latency_ms})"
                            );
                        }
                    }
                    Finding::Fast { ms_per_step, .. } => {
                        assert!(cost < fast, "cost {cost:e} is not under {fast:e}");
                        assert!(ms_per_step < fast);
                    }
                }
            }
        }
        cost *= 1.7;
    }
}

/// A GPU clock that reads low (here a tenth) cannot make the ramp run a long pass: the wall bounds
/// each step. At worst it reads the card as fast and leaves it on the defaults, as before the probe.
#[test]
fn a_clock_that_reads_low_cannot_make_a_long_pass() {
    let mut cost = 1e-9;
    while cost < 2e-4 {
        let card = Card { ts_scale: 0.1, ..Card::honest(cost) };
        let (_, ran) = ramp(card, 1.92e-8);
        let first = ran[0];
        for &ms in &ran[1..] {
            assert!(ms <= (PASS_WALL_MAX_MS * 1.05).max(first), "cost {cost:e}: a {ms:.1} ms pass after {ran:?}");
        }
        cost *= 1.7;
    }
}

/// A fast card is done in a few small passes, latency or not: the RTX 3080's direct cost
/// (4.92e-9 ms/step) against the default ceiling's 1.92e-8. With PLUTO's ~12 ms of submission
/// latency this read as 9.7e-8 when the wall counted (dev.8).
#[test]
fn a_fast_card_is_done_in_a_few_small_passes() {
    for latency_ms in [0.3, 12.0] {
        let (found, ran) = ramp(Card { latency_ms, ..Card::honest(4.92e-9) }, 1.92e-8);
        assert!(matches!(found, Finding::Fast { .. }), "latency {latency_ms}: {found:?}");
        assert!(ran.len() <= 4, "{ran:?}");
        assert!(ran.iter().sum::<f64>() < 10.0, "{ran:?}");
    }
}
