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

/// A finding stands against a re-measure within [`CONFIRM_FASTER`], as the slower of the two; a
/// re-measure that reads faster than that sends the mode back to be measured from the start.
#[test]
fn a_measurement_stands_unless_its_repeat_reads_much_faster() {
    let px = PX as f64;
    let first = Finding::Measured { iters: 586, ms: 14.3, wall_ms: 15.0, ms_per_step: 14.3 / (px * 586.0) };
    let c = first.ms_per_step();
    assert_eq!(confirm(first, c), Confirm::Stands(first));
    assert_eq!(confirm(first, c / 1.1), Confirm::Stands(first), "jitter, faster");
    match confirm(first, c * 1.1) {
        Confirm::Stands(Finding::Measured { iters, ms_per_step, .. }) => {
            assert_eq!(iters, 586);
            assert_eq!(ms_per_step, c * 1.1, "the slower reading stands");
        }
        other => panic!("{other:?}"),
    }
    // The RTX 3070's cold direct reading against its warm one (PLUTO, dev.23).
    assert_eq!(confirm(first, 1.07e-8), Confirm::Again);
    assert_eq!(confirm(first, c / (CONFIRM_FASTER * 1.01)), Confirm::Again);
    // A re-measure that cannot be used, and a fast bound, leave the finding as it was.
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(confirm(first, bad), Confirm::Stands(first), "{bad}");
    }
    let fast = Finding::Fast { iters: 256, ms: 0.72, ms_per_step: 1.08e-8 };
    assert_eq!(confirm(fast, 1e-6), Confirm::Stands(fast));
}

/// A card whose GPU clock is still coming up, as the probe sees it: every pass runs `idle_factor`
/// times slower until the probe has kept it busy (by the wall) for `up_after_ms`, then at its
/// working clock. One `call` is one `render_iter_probe`: a warm-up and [`REPS`] timed passes, each
/// with `latency_ms` of submission. `longest_ms` is the longest pass the GPU ran.
struct Warming {
    wall_ms: f64,
    up_after_ms: f64,
    idle_factor: f64,
    latency_ms: f64,
    longest_ms: f64,
    calls: u32,
}

impl Warming {
    fn new(up_after_ms: f64, idle_factor: f64) -> Self {
        Warming { wall_ms: 0.0, up_after_ms, idle_factor, latency_ms: 12.0, longest_ms: 0.0, calls: 0 }
    }
    fn call(&mut self, ms_per_step: f64, iters: u32) -> Vec<[f64; 2]> {
        self.calls += 1;
        let mut reps = Vec::new();
        for i in 0..=REPS {
            let f = if self.wall_ms < self.up_after_ms { self.idle_factor } else { 1.0 };
            let gpu = 0.2 + ms_per_step * f * PX as f64 * f64::from(iters);
            self.longest_ms = self.longest_ms.max(gpu);
            self.wall_ms += gpu + self.latency_ms;
            if i > 0 {
                reps.push([gpu, gpu + self.latency_ms]);
            }
        }
        reps
    }
    /// The slowest timed rep's GPU time and wall, as the probe takes them.
    fn slowest(reps: &[[f64; 2]]) -> (f64, f64) {
        let gpu = reps.iter().map(|&t| gpu_ms(t)).fold(f64::NAN, f64::max);
        let wall = reps.iter().map(|&[_, w]| w).fold(f64::NAN, f64::max);
        (gpu, wall)
    }
    /// The ramp (`speed_probe::ramp`) over this card for a mode costing `cost` a step.
    fn ramp(&mut self, cost: f64, fast: f64) -> Finding {
        let mut iters = ITERS_START;
        loop {
            let (gpu, wall) = Self::slowest(&self.call(cost, iters));
            match step(PX, iters, gpu, wall, fast).expect("a modeled time is usable") {
                Step::Done(f) => return f,
                Step::Next(n) => iters = n,
            }
        }
    }
    /// Both laps over the three modes (`costs`, `fasts`, in [`MODES`] order): the findings that
    /// stand, and how many times each mode was measured again.
    fn probe(&mut self, costs: [f64; 3], fasts: [f64; 3]) -> [(Finding, u32); 3] {
        let lap1: Vec<Finding> = (0..3).map(|k| self.ramp(costs[k], fasts[k])).collect();
        let mut out = Vec::new();
        for k in 0..3 {
            let remeasure = |gpu: &mut Self, f: Finding| {
                let iters = match f {
                    Finding::Measured { iters, .. } | Finding::Fast { iters, .. } => iters,
                };
                let (ms, _) = Self::slowest(&gpu.call(costs[k], iters));
                Ok(ms / (PX as f64 * f64::from(iters)))
            };
            out.push(confirmed(lap1[k], self, remeasure, |gpu| Ok(gpu.ramp(costs[k], fasts[k]))));
        }
        out.try_into().expect("three modes")
    }
}

/// PLUTO's RTX 3070 after a cold start (dev.23): direct and df32 under the default ceiling's costs,
/// floatexp at 1.163e-7; its clock ~8.6x slow until ~0.45 s into the probe, with ~12 ms of submission
/// latency. The first lap reads the slow clock (as dev.23 did: direct 9.3e-8, df32 2.8e-7), and the
/// second lap finds what a warm start finds: direct and df32 faster than anything they could
/// change, floatexp measured at its cost. Re-measures are passes the first lap already ran, at a
/// clock that has only come up: none is longer than its longest.
#[test]
fn a_clock_still_coming_up_is_measured_again() {
    let costs = [1.0e-8, 4.0e-8, 1.163e-7];
    let fasts = [1.92e-8, 9.9e-8, 1e-7];
    let mut warm = Warming::new(0.0, 1.0);
    let at_work = warm.probe(costs, fasts);
    let mut cold = Warming::new(450.0, 8.6);
    let lap1: Vec<Finding> = (0..3).map(|k| cold.ramp(costs[k], fasts[k])).collect();
    assert!(lap1[0].ms_per_step() > 5.0 * costs[0], "the model's first lap reads the slow clock: {lap1:?}");
    let longest_lap1 = cold.longest_ms;
    let mut cold = Warming::new(450.0, 8.6);
    let found = cold.probe(costs, fasts);
    for k in 0..3 {
        match (found[k].0, at_work[k].0) {
            (Finding::Fast { .. }, Finding::Fast { .. }) => {}
            (Finding::Measured { ms_per_step: c, .. }, Finding::Measured { ms_per_step: w, .. }) => {
                assert!(c >= costs[k] && c <= w * 1.02, "mode {k}: {c:e} cold against {w:e} warm");
            }
            other => panic!("mode {k}: {other:?}"),
        }
    }
    assert!(found[0].1 >= 1 && found[1].1 >= 1, "direct and df32 measured again: {found:?}");
    assert!(cold.longest_ms <= longest_lap1, "a {:.1} ms pass after a first lap of at most {longest_lap1:.1}", cold.longest_ms);
}

/// A card at its working clock from the start: the second lap changes no finding, and costs one
/// pass set for each mode the first measured (none for a fast bound).
#[test]
fn a_steady_card_is_confirmed_in_one_pass_set_a_measured_mode() {
    let fasts = [1.92e-8, 9.9e-8, 1e-7];
    for costs in [[1.0e-8, 4.0e-8, 1.163e-7], [5e-8, 3e-7, 6e-7], [1.2e-6, 1.1e-5, 1.15e-5]] {
        let mut gpu = Warming::new(0.0, 1.0);
        let lap1: Vec<Finding> = (0..3).map(|k| gpu.ramp(costs[k], fasts[k])).collect();
        let lap1_calls = gpu.calls;
        let mut gpu = Warming::new(0.0, 1.0);
        let found = gpu.probe(costs, fasts);
        let measured = lap1.iter().filter(|f| matches!(f, Finding::Measured { .. })).count() as u32;
        assert_eq!(gpu.calls, lap1_calls + measured, "{costs:?}");
        for k in 0..3 {
            assert_eq!(found[k], (lap1[k], 0), "{costs:?} mode {k}");
        }
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
