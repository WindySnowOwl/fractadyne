//! Property tests for the frame-cost controllers. These pin the SHAPES that have actually
//! broken here — a measured loop starving and falling back on a constant that then binds
//! something, and a slow dispatch failing to bring the budget down — rather than one past
//! incident at a time.
use super::*;

/// Deterministic xorshift64* — fixed seed, reproducible failures, no dev-dependency.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            0
        } else {
            self.next() % n
        }
    }
}

/// A spread of readings covering every regime this controller has ever misjudged: instant,
/// vsync-shaped, the 400–600 ms latency-floor window, the ~900 ms reproduced lethal band, and
/// values no healthy GPU produces.
fn a_reading(r: &mut Rng) -> f64 {
    match r.below(7) {
        0 => 0.0,
        1 => r.below(20) as f64,
        2 => 380.0 + r.below(60) as f64,
        3 => 400.0 + r.below(200) as f64,
        4 => 850.0 + r.below(400) as f64,
        5 => 1.0e6,
        _ => r.below(3000) as f64,
    }
}

#[test]
fn the_next_budget_is_always_a_legal_dispatch_size() {
    // Quantified counterpart to the pinned cases below. Whatever a dispatch reports, the
    // budget it produces must remain dispatchable: never 0 (which would stall the render
    // forever, making no progress and never timing out) and never above the regime's ceiling.
    let c = crate::tunables::cost();
    let mut r = Rng(0x5EED_1234_ABCD_0001);
    let mut decided = 0u32;
    for _ in 0..8000 {
        let cur = 1 + r.below(c.tdr_steps_ceil);
        let steps = r.below(c.tdr_steps_ceil.saturating_mul(2));
        let ms = a_reading(&mut r);
        let explicit = r.below(2) == 1;
        if let Some((next, _ok)) = budget_step(cur, steps, ms, explicit) {
            decided += 1;
            let ceil =
                if explicit { c.explicit_steps_ceil } else { c.tdr_steps_ceil };
            assert!(
                next >= c.tdr_min_steps,
                "budget fell under the floor: cur={cur} steps={steps} ms={ms} \
                 explicit={explicit} -> {next} < {}",
                c.tdr_min_steps
            );
            assert!(
                next <= ceil,
                "budget exceeded its ceiling: cur={cur} steps={steps} ms={ms} \
                 explicit={explicit} -> {next} > {ceil}"
            );
        }
    }
    // Anti-vacuity guard. `budget_step` returns an Option, so a change that made it decline
    // most readings would leave every assertion above unexecuted and this test still green —
    // the same way an interior-filled view let the bignum oracle pass without ever comparing
    // an escape count (see the G1 note in selftest.rs).
    assert!(decided > 4000, "only {decided}/8000 readings produced a budget — sweep went vacuous");
}

#[test]
fn a_lethal_reading_never_asks_for_more_work() {
    // The emergency retreat is allowed to shrink FURTHER than TDR_SHRINK_MAX — that is its
    // whole purpose, getting out of the lethal band in one step instead of three. What it must
    // never do is come back asking for more: a dispatch that already measured near the TDR
    // deadline is the strongest evidence available that per-step cost has collapsed.
    //
    // The `max(cur, floor)` is not slack: when `cur` sits below TDR_MIN_STEPS the clamp raises
    // the result, and the floor legitimately wins over the retreat.
    let c = crate::tunables::cost();
    let mut r = Rng(0xFEED_FACE_0000_0007);
    let mut decided = 0u32;
    for _ in 0..4000 {
        let cur = 1 + r.below(c.tdr_steps_ceil);
        let steps = r.below(c.tdr_steps_ceil.saturating_mul(2));
        let ms = c.tdr_lethal_ms + r.below(5000) as f64;
        let explicit = r.below(2) == 1;
        if let Some((next, _ok)) = budget_step(cur, steps, ms, explicit) {
            decided += 1;
            assert!(
                next <= cur.max(c.tdr_min_steps),
                "a {ms} ms reading RAISED the budget: cur={cur} steps={steps} \
                 explicit={explicit} -> {next}"
            );
        }
    }
    assert!(decided > 2000, "only {decided}/4000 lethal readings decided — sweep went vacuous");
}

#[test]
fn a_nonsense_frame_time_cannot_wedge_the_controller() {
    // Frame times come from a clock, and clocks lie: a zero, a negative interval across a
    // clock adjustment, or a NaN from a division by an elapsed time of zero. None of those
    // may panic, and none may produce an illegal budget. This is a robustness floor, not a
    // claim about which direction the controller should move.
    let c = crate::tunables::cost();
    for &ms in &[0.0, -0.0, -1.0, -1.0e9, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::MIN_POSITIVE] {
        for &cur in &[c.tdr_min_steps, 1_000_000_000, c.tdr_steps_ceil] {
            for &explicit in &[false, true] {
                if let Some((next, _ok)) = budget_step(cur, cur, ms, explicit) {
                    let ceil =
                        if explicit { c.explicit_steps_ceil } else { c.tdr_steps_ceil };
                    assert!(
                        next >= c.tdr_min_steps && next <= ceil,
                        "ms={ms} cur={cur} explicit={explicit} -> illegal budget {next}"
                    );
                }
            }
        }
    }
}

#[test]
fn slow_reading_always_lowers_the_budget() {
    // Whatever the dispatch's nominal size, taking longer than the target must shrink.
    for &cur in &[crate::tunables::cost().tdr_bootstrap_steps * 4, 1_000_000_000, 50_000_000_000, crate::tunables::cost().tdr_steps_ceil] {
        for &steps in &[1u64, cur / 3, cur, cur * 4] {
            for explicit in [false, true] {
                let (next, _) = budget_step(cur, steps, crate::tunables::cost().tdr_budget_ms * 2.0, explicit)
                    .expect("a slow reading is never discarded");
                assert!(
                    next < cur,
                    "slow reading grew the budget: cur={cur} steps={steps} -> {next}"
                );
            }
        }
    }
}

#[test]
fn a_slow_small_dispatch_pulls_the_budget_to_its_own_size() {
    // The beta.40 defect: a 1451 ms dispatch of 1.03e10 steps was DISCARDED as undersized and
    // the budget stayed at 1.663e11, far above anything the GPU could finish.
    let (next, _) = budget_step(166_300_000_000, 10_300_000_000, 1451.4, false).unwrap();
    assert!(next <= 10_300_000_000, "budget must come down to the size that measured slow");
}

#[test]
fn an_explicit_iteration_count_is_honoured_verbatim() {
    // A1's exact symptom: 10,000,000 in the Iterations box at ~1e9x read back as iter 82,741
    // (zoom cap x boost). Explicit means THIS count.
    assert_eq!(live_iter_budget(10_000_000, 30.0, 1.0, true), 10_000_000);
    // ...bounded by the absolute limit only.
    assert_eq!(
        live_iter_budget(u32::MAX, 30.0, 1.0, true),
        crate::MAX_ITER_LIMIT
    );
    // Auto mode: unchanged behaviour — the zoom-appropriate cap times the boost binds.
    let auto = live_iter_budget(10_000_000, 30.0, 1.0, false);
    assert!(auto < 20_000, "auto at 1e9x should sit near the zoom cap, got {auto}");
    let boosted = live_iter_budget(10_000_000, 30.0, 4.0, false);
    assert_eq!(boosted, auto * 4, "the boost is a plain multiplier on the cap");
    // And an ask below every cap passes through untouched in both modes.
    assert_eq!(live_iter_budget(2_000, 30.0, 1.0, false), 2_000);
    assert_eq!(live_iter_budget(2_000, 30.0, 1.0, true), 2_000);
}

#[test]
fn a_wall_probe_is_not_priced_by_the_frame_that_submitted_it() {
    // Arm on frame 10 with a vsync-shaped interval — the lie that killed the device.
    let (p, out) = wall_probe_step(None, true, 400_000_000, 10, 18.4);
    assert!(out.is_none(), "the submitting frame must not price its own dispatch");
    // Frame 11: still too early, but the stall lands here.
    let (p, out) = wall_probe_step(p, true, 999, 11, 1070.0);
    assert!(out.is_none());
    // Frame 12: two intervals after arming, priced at the MAX seen, not the first or the mean.
    let (p, out) = wall_probe_step(p, true, 999, 12, 20.0);
    assert_eq!(p, None, "the probe clears when it resolves");
    let (ms, steps) = out.expect("a probe armed on frame 10 resolves by frame 12");
    assert_eq!(steps, 400_000_000, "priced against the count IT dispatched");
    assert!((ms - 1070.0).abs() < 1e-9, "the stall is the signal, got {ms}");
}

#[test]
fn a_wall_probe_only_arms_on_a_frame_that_really_iterated() {
    assert_eq!(wall_probe_step(None, false, 400_000_000, 10, 18.4), (None, None));
    assert_eq!(wall_probe_step(None, true, 0, 10, 18.4), (None, None));
}

#[test]
fn a_converged_low_budget_is_never_hoisted_back_to_the_guess() {
    let boot = crate::tunables::cost().tdr_bootstrap_steps;
    let low = crate::tunables::cost().tdr_min_steps * 2;
    assert!(low < boot);
    assert_eq!(budget_base(low, boot), low, "a real measurement is used as is");
    assert_eq!(budget_base(0, boot), boot, "only 'unmeasured' gets the guess");
    assert_eq!(budget_base(1, boot), crate::tunables::cost().tdr_min_steps, "but never below the absolute floor");
}

#[test]
fn the_opening_guess_is_derived_from_a_measured_rate() {
    let c = crate::tunables::cost();
    // Nothing measured anywhere: unchanged from before this existed. This is the case the
    // 4e8 → 1e8 experiment regressed (`--livetest` seahorse-2, 480×270 → 100×56), so it is
    // pinned rather than left to inference.
    assert_eq!(bootstrap_steps(0.0, 0.0), c.tdr_bootstrap_steps);

    // A rate fast enough to ask for more than the constant is CAPPED by it: the derived guess
    // may only ever lower the opening dispatch.
    let fast = (c.tdr_bootstrap_steps as f64 / c.tdr_bootstrap_ms) * 10.0;
    assert_eq!(bootstrap_steps(fast, 0.0), c.tdr_bootstrap_steps);

    // The field case (2026-08-15, RX 6800 XT): mode 2 with no BLA measured 5.909e8 steps in
    // 1038 ms. The guess derived from that rate must be worth about TDR_BOOTSTRAP_MS, i.e.
    // ~25× smaller than the constant that lost the device.
    let amd_mode2 = 5.909e8 / 1038.0;
    let got = bootstrap_steps(amd_mode2, 0.0);
    assert!(got < c.tdr_bootstrap_steps / 10, "got {got}");
    let implied_ms = got as f64 / amd_mode2;
    assert!(
        (implied_ms - c.tdr_bootstrap_ms).abs() < 1.0,
        "the derived guess should be worth ~{} ms, got {implied_ms:.1}",
        c.tdr_bootstrap_ms
    );

    // Only ANOTHER mode measured — mode 0 on that same card, ~8.65e7 steps/ms. Believing it
    // directly would size a lethal opening dispatch in mode 2, so the margin divides it down.
    let amd_mode0 = 7.785e10 / 900.0;
    let cross = bootstrap_steps(0.0, amd_mode0);
    assert!(cross < bootstrap_steps(amd_mode0, 0.0), "the margin must bite");
    assert!(
        (cross as f64) / amd_mode2 < c.tdr_latency_accept_ms,
        "the first frame in an unmeasured mode must not be watchdog-relevant"
    );

    // The floor still holds: an absurdly slow rate cannot drive the guess under TDR_MIN_STEPS.
    assert_eq!(bootstrap_steps(1e-9, 0.0), c.tdr_min_steps);
}

#[test]
fn an_undersized_slow_reading_never_holds_a_large_budget_as_converged() {
    // ⭐The exposure the 900→400 target change opened, and the reason the latency-floor guard
    // needs a SECOND condition. The guard tests the dispatch's ABSOLUTE size (steps <=
    // TDR_BOOTSTRAP_STEPS) but never whether the BUDGET is small. So a clamped edge tile, a
    // chunk step, or a motion-shrunk frame that measures inside the 400–600 ms window while
    // `cur` sits at a converged 3e11 was read as "this is the latency floor, hold position" —
    // and returned ok = true, which arms the tiled settle on a budget calibrated for a per-step
    // rate that no longer exists. An undersized dispatch taking 450 ms is the strongest
    // available evidence that per-step cost has COLLAPSED; the next full-size tile at `cur`
    // then prices in tens of seconds, and the emergency retreat cannot help because its first
    // reading arrives two frames late, behind two more budget-sized dispatches.
    let c = crate::tunables::cost();
    let cur = c.tdr_steps_ceil; // 3e11, a converged deep budget
    let steps = c.tdr_bootstrap_steps / 4; // 1e8, plainly undersized against cur
    let ms = (c.tdr_budget_ms + c.tdr_latency_accept_ms) / 2.0; // 500 ms: slow, under the accept bound
    assert!(ms > c.tdr_budget_ms && ms <= c.tdr_latency_accept_ms, "test sits in the window");

    let (next, _ok) = budget_step(cur, steps, ms, false).expect("a slow reading carries signal");
    assert!(next < cur, "an undersized slow reading must LOWER a large budget, not hold it");
    // The budget must land near the DISPATCH's own size, not merely somewhere below `cur`:
    // `base = min(cur, steps)` is what makes the collapse proportionate to the evidence.
    assert!(
        next <= steps,
        "must re-price to the measured dispatch, got {next:.3e} against steps {steps:.3e}"
    );
    // And the result must be honest about time: at `next`, this per-step rate implies ~target.
    let implied_ms = ms * (next as f64 / steps as f64);
    assert!(
        implied_ms <= c.tdr_budget_ms * 1.05,
        "the re-priced budget should imply ~{:.0}ms, got {implied_ms:.0}ms",
        c.tdr_budget_ms
    );
    // NOTE: `ok` is deliberately not asserted false. Once the hold is gated on the dispatch
    // being representative, this path SHRINKS rather than holds, so a converged flag here sits
    // on a corrected budget (8e7 implying 400 ms) and arming the settle on it is safe. The
    // danger was never the flag; it was the flag on a budget that had not been re-priced.
}

#[test]
fn a_reading_in_the_lethal_band_retreats_in_one_step() {
    // The 2026-08-16 device loss, as numbers: budget pinned at the 3e11 ceiling, a tile
    // measured 1033 ms. Clamped at TDR_SHRINK_MAX the controller could only reach 1.5e11 and
    // would spend another ~1 s frame per reading getting the rest of the way. It died after
    // three. Past the band the raw ratio must apply.
    let c = crate::tunables::cost();
    let cur = c.tdr_steps_ceil;
    let (next, _) = budget_step(cur, cur, 1033.0, false).expect("a slow reading carries signal");
    let capped = (cur as f64 * c.tdr_shrink_max) as u64;
    assert!(
        next < capped,
        "must retreat further than the ×{} cap allows: got {next:.3e}, cap floor {capped:.3e}",
        c.tdr_shrink_max
    );
    // And it should land near the target in ONE move, not merely somewhat lower.
    let implied_ms = 1033.0 * (next as f64 / cur as f64);
    assert!(
        implied_ms <= c.tdr_budget_ms * 1.05,
        "one retreat should reach the target: implied {implied_ms:.0}ms vs target {:.0}ms",
        c.tdr_budget_ms
    );
}

#[test]
fn the_shrink_cap_still_applies_below_the_lethal_band() {
    // The cap exists so the ratio search walks instead of lurching; only the emergency case
    // bypasses it. Pick a reading where the cap actually BINDS but the band does not: the
    // clamp only engages once target/ms < shrink_max, i.e. past 800 ms at a 400 ms target, so
    // the window where it governs is 800–900 ms. (My first version of this test used 600 ms,
    // where the ratio is 0.667 and the clamp is not reached at all — it asserted the cap was
    // in force somewhere it never applies, and failed for the right reason.)
    let c = crate::tunables::cost();
    let ms = (c.tdr_budget_ms / c.tdr_shrink_max) + 10.0; // just past where the clamp engages
    assert!(ms < c.tdr_lethal_ms, "no window between the clamp and the band: {ms}ms");
    let cur = c.tdr_steps_ceil;
    let (next, _) = budget_step(cur, cur, ms, false).expect("slow reading");
    let capped = (cur as f64 * c.tdr_shrink_max) as u64;
    assert_eq!(next, capped, "below the band the ×{} cap governs", c.tdr_shrink_max);
}

#[test]
fn both_regimes_now_aim_outside_the_lethal_band() {
    // The invariant the 900→400 change exists to establish: a CONVERGED controller must not be
    // parked inside the band, in either regime. Two field losses happened because auto was.
    let c = crate::tunables::cost();
    assert!(c.tdr_budget_ms < c.tdr_lethal_ms, "auto target is inside the lethal band");
    assert!(c.tdr_explicit_budget_ms < c.tdr_lethal_ms, "explicit target is inside the band");
    assert!(
        c.tdr_budget_ms * 2.0 <= c.tdr_lethal_ms,
        "auto target should keep the >=2x margin the explicit one is documented to have"
    );
}

#[test]
fn a_capped_frame_rate_must_not_ratchet_motion_resolution_to_the_floor() {
    // ⭐The bug: the frame interval is frame-start to frame-start and the deliberate fps_cap
    // sleep sits INSIDE it, so with a 30 fps cap every frame read ~33 ms however cheap it was.
    // 33 > 24, so this shrank on EVERY real frame and reached the 0.30 floor in about five of
    // them — deep motion pinned at 30% linear resolution for the session, on hardware with
    // headroom to spare. The fix discounts the sleep before pricing (Perf::cap_sleep_ms); this
    // pins both halves of the consequence.
    let floor = 0.30;

    // A genuinely cheap frame under a 30 fps cap: 33.3 ms raw, ~4 ms of real work.
    // Priced RAW (the bug) it collapses; priced DISCOUNTED it grows toward native.
    let mut raw = 1.0;
    for _ in 0..6 {
        raw = motion_res_step(raw, 33.3, floor);
    }
    assert!(raw <= floor + 1e-9, "raw pricing reaches the floor — this is the bug: {raw:.3}");

    let mut fixed = 1.0;
    for _ in 0..6 {
        fixed = motion_res_step(fixed, 4.0, floor);
    }
    assert_eq!(fixed, 1.0, "a 4 ms frame must stay at native, not shrink");

    // The controller still does its job on frames that are genuinely expensive.
    assert!(motion_res_step(1.0, 40.0, floor) < 1.0, "a real 40 ms frame must shrink");
    // And the deadband holds, so it settles instead of hunting.
    assert_eq!(motion_res_step(0.8, 20.0, floor), 0.8, "17..=24 ms holds");
    // The floor is respected however slow the frame is.
    assert!(motion_res_step(0.31, 5000.0, floor) >= floor);
}

#[test]
fn a_pending_rebuild_refuses_growth_but_never_blocks_a_shrink() {
    let cur = 4_520_000_000u64;
    let up = cur * 3 / 2; // a ×1.5 growth step
    let down = cur / 2;

    // Idle: both directions honoured, convergence passes through.
    assert_eq!(budget_after_build_gate(cur, up, true, false), (up, true));
    assert_eq!(budget_after_build_gate(cur, down, true, false), (down, true));

    // Rebuild in flight: growth refused, shrink honoured. The asymmetry IS the safety property
    // — a frame priced against an orbit about to be replaced can warn us, but cannot reward us.
    assert_eq!(budget_after_build_gate(cur, up, true, true), (cur, false));
    assert_eq!(budget_after_build_gate(cur, down, true, true), (down, false));

    // Convergence is withheld while building, which keeps the tiled settle disarmed. Arming a
    // full-resolution grid on a budget about to be invalidated is what made 2026-08-16 fatal.
    assert!(!budget_after_build_gate(cur, cur, true, true).1);
}

#[test]
fn the_2026_08_16_budget_inflation_cannot_recur_while_building() {
    // The measured path: 4.520e10 climbing at ×1.5 per reading to the 3e11 ceiling across ~five
    // cheap readings taken around one reference build. With the gate, none of those readings
    // banks anything, so the tiled grid is never sized against the inflated figure.
    let c = crate::tunables::cost();
    let mut b = 45_200_000_000u64;
    for _ in 0..5 {
        let (next, _) = budget_step(b, b, 50.0, false).unwrap_or((b, true));
        let (gated, _) = budget_after_build_gate(b, next, true, true);
        b = gated;
    }
    assert_eq!(b, 45_200_000_000, "no growth may be banked during a rebuild");
    assert!(b < c.tdr_steps_ceil, "and it must not have reached the ceiling");
}

#[test]
fn the_budget_can_shrink_below_the_opening_guess() {
    // The floor used to be TDR_BOOTSTRAP_STEPS, so a regime where the opening guess is itself
    // too expensive (measured: 4e8 steps = 780 ms at mode 2 with orbit_len=626) left the
    // controller with nowhere to go. A safety valve has to move toward safety.
    let (next, _) = budget_step(crate::tunables::cost().tdr_bootstrap_steps, crate::tunables::cost().tdr_bootstrap_steps, 780.0, false)
        .expect("a reading at the opening guess is usable");
    // 780 ms is under the 900 ms target, so this one still grows — the point is the next ones.
    let mut b = crate::tunables::cost().tdr_bootstrap_steps;
    for _ in 0..8 {
        b = budget_step(b, b, 2_000.0, false).expect("a slow reading is never discarded").0;
    }
    assert!(
        b < crate::tunables::cost().tdr_bootstrap_steps,
        "sustained 2 s frames must drive the budget below the opening guess, got {b}"
    );
    assert!(b >= crate::tunables::cost().tdr_min_steps);
    let _ = next;
}

#[test]
fn budget_never_leaves_its_clamps() {
    for &ms in &[0.05, 1.0, crate::tunables::cost().tdr_budget_ms, 5_000.0] {
        for &cur in &[crate::tunables::cost().tdr_min_steps, crate::tunables::cost().tdr_bootstrap_steps, crate::tunables::cost().tdr_steps_ceil] {
            if let Some((next, _)) = budget_step(cur, cur, ms, false) {
                assert!((crate::tunables::cost().tdr_min_steps..=crate::tunables::cost().tdr_steps_ceil).contains(&next));
            }
            if let Some((next, _)) = budget_step(cur, cur, ms, true) {
                assert!((crate::tunables::cost().tdr_min_steps..=crate::tunables::cost().explicit_steps_ceil).contains(&next));
            }
        }
    }
}

#[test]
fn growth_is_bounded_so_one_reading_cannot_reach_the_watchdog() {
    let cur = 1_000_000_000;
    let (next, _) = budget_step(cur, cur, 0.01, false).unwrap();
    assert!(next as f64 <= cur as f64 * crate::tunables::cost().tdr_grow_max + 1.0);
}

#[test]
fn a_latency_floor_is_held_not_shrunk_into_the_corner() {
    // The 16×16 deep-hold collapse (grand tour, six checkpoints 480×270 → 16×16): a small
    // mode-2 dispatch measures ~250–450 ms regardless of pixel count (chain latency), and a
    // target below that floor shrank the budget to TDR_MIN_STEPS and pinned it there. A
    // small slow dispatch inside the accept window must HOLD, converged.
    let (next, ok) = budget_step(20_000_000, 50_000_000, 450.0, true)
        .expect("a small slow reading is never discarded");
    assert_eq!(next, 20_000_000, "the floor is held, not shrunk");
    assert!(ok, "an accepted floor reads as converged so the settle can take over");
    // But a floor NEAR the watchdog band is the beta.48 death loop — shrink proceeds.
    let (next, _) = budget_step(400_000_000, 400_000_000, 1500.0, true).unwrap();
    assert!(next < 400_000_000, "a watchdog-relevant floor must still shrink");
    // The same shape in AUTO also shrinks. ⚠This used to hold because auto's slow threshold
    // (900 ms) sat past the accept bound so the guard could not fire at all; since 2026-08-16
    // auto's target is 400 ms, so it now passes for the ORIGINAL reason instead — 1070 ms is
    // past the 600 ms accept bound, which is the beta.48 death-loop case.
    let (next, _) = budget_step(400_000_000, 400_000_000, 1070.0, false).unwrap();
    assert!(next < 400_000_000);
}

#[test]
fn explicit_budget_measures_past_the_nominal_cap_and_stops_at_its_ceiling() {
    // The 2026-08-12 field report (scripted dive, 5111×2158 window, ~1.29M explicit iters at
    // e216): cap-sized dispatches (2e10 nominal) measured 54.3 ms real — 4× of safe headroom
    // the flat cap wasted on 26-pixel blocks — while the frozen budget above the cap ignored
    // every reading. Measured growth must walk past the old cap and pin at the explicit
    // ceiling, converged (`ok`), with the real cost still far under the ~0.9 s lethal band.
    let skip_rate = crate::tunables::cost().explicit_dispatch_cap as f64 / 54.3; // nominal steps per real ms, measured
    let mut b = crate::tunables::cost().explicit_dispatch_cap;
    for _ in 0..16 {
        let ms = b as f64 / skip_rate; // cost tracks nominal size at a fixed skip rate
        assert!(ms < 300.0, "explicit dispatches must stay far under the lethal band: {ms}");
        let (next, ok) =
            budget_step(b, b, ms, true).expect("a budget-sized reading is never discarded");
        if next == b {
            assert!(ok, "a pinned explicit budget must read as converged");
            break;
        }
        b = next;
    }
    assert_eq!(b, crate::tunables::cost().explicit_steps_ceil, "growth stops exactly at the explicit ceiling");
    // And the regime still shrinks on a genuinely slow reading (skip collapse).
    let (next, _) = budget_step(crate::tunables::cost().explicit_steps_ceil, crate::tunables::cost().explicit_steps_ceil, 700.0, true).unwrap();
    assert!(next < crate::tunables::cost().explicit_steps_ceil);
}

#[test]
fn starvation_needs_an_outstanding_dispatch() {
    // Idle view: the last dispatch was already priced. Never starved, however long it sits.
    assert!(!measurement_starved(10, 10, 10_000, 30));
    assert!(!measurement_starved(5, 10, 10_000, 30));
    // A dispatch went out after the last reading and nothing came back.
    assert!(measurement_starved(11, 10, 40, 30));
    assert!(!measurement_starved(11, 10, 39, 30));
}

#[test]
fn starvation_is_measured_from_the_reading_not_the_dispatch() {
    // A settling view dispatches a tile EVERY frame, so "frames since the newest dispatch" is
    // 0 or 1 forever and would never trip however starved the loop is.
    let now = 500;
    assert!(measurement_starved(now, 10, now, 30));
}

#[test]
fn an_unmeasured_budget_never_binds_resolution_below_a_measured_one() {
    // The invariant. `settle_max_tiles` is the allowance the resolution shrink is sized
    // against; a view that has measured nothing must not be given LESS room than one that has.
    assert!(crate::tunables::cost().tdr_max_tiles >= 1);
    let unmeasured_allowance = crate::tunables::cost().tdr_bootstrap_steps.saturating_mul(crate::tunables::cost().tdr_max_tiles);
    assert!(
        unmeasured_allowance > crate::tunables::cost().tdr_bootstrap_steps,
        "tiling must be able to exceed a single bootstrap dispatch, or the bootstrap              constant becomes a permanent resolution cap"
    );
}
