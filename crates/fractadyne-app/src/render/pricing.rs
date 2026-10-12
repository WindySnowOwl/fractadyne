//! The live view's frame-pricing family (design/live-render-robustness.md §5.1, step M2): what a
//! dispatch is measured to cost, what the next one may cost, and the backstops for when the
//! measurement does not arrive — the reading rule and its probe, the reading pool, the wall clock,
//! the blind tripwire and dead-man, motion jams and the chunk-band licences. Moved verbatim from
//! `render.rs`, which re-exports it; the property tests stay beside their subjects there.

/// ⭐The OPENING GUESS, derived from a measured per-step rate rather than assumed.
///
/// `rate_this_mode` is steps-per-ms measured in the mode we are about to dispatch in;
/// `rate_other_mode` is the most pessimistic rate measured in any OTHER mode. Both are 0.0 when
/// unknown. The result is always clamped into `[TDR_MIN_STEPS, TDR_BOOTSTRAP_STEPS]`, so this can
/// only ever lower the guess — a device that has measured nothing gets exactly the historical
/// constant, and no rate, however optimistic, can raise the opening dispatch above it.
///
/// The three cases, in order of how much we actually know:
///  1. **This mode measured here** — the honest answer: `rate × TDR_BOOTSTRAP_MS`.
///  2. **Only another mode measured** — that rate bounds the device's speed but says nothing about
///     this mode's per-step cost, which the field report has at ~152× worse across 0→2. Divide by
///     `MODE_RATE_UNKNOWN_MARGIN` before believing it. This is the case the 2026-08-15 RX 6800 XT
///     device loss lands in.
///  3. **Nothing measured at all** — the first frame of the session; keep the historical constant,
///     because there is no information to do better with and lowering it blindly was tried,
///     measured, and reverted (see `TDR_BOOTSTRAP_STEPS`).
pub(crate) fn bootstrap_steps(rate_this_mode: f64, rate_other_mode: f64) -> u64 {
    let c = crate::tunables::cost();
    let rate = if rate_this_mode > 0.0 {
        rate_this_mode
    } else if rate_other_mode > 0.0 {
        rate_other_mode / c.mode_rate_unknown_margin
    } else {
        return c.tdr_bootstrap_steps;
    };
    // NaN/inf cannot reach here (both inputs are finite and positive), but the cast still has to be
    // saturating: a very fast rate overflows u64 before the clamp would catch it.
    let steps = (rate * c.tdr_bootstrap_ms).max(0.0);
    (steps as u64).clamp(c.tdr_min_steps, c.tdr_bootstrap_steps)
}

/// The budget a measurement or a dispatch should start from. Zero means "nothing measured yet in
/// this mode", the one case the opening guess exists for; anything else is a real measurement and
/// is used AS IS.
///
/// ⚠This is a function because three call sites — the dispatch sizing in `build_params`, the
/// controller in `update`, and the livetest harness's copy of it — each made the decision
/// separately, and two of them made it wrong the same way: `fe_budget.max(TDR_BOOTSTRAP_STEPS)`,
/// which hoists a converged-low budget back to the opening guess on every reading so it can never
/// settle below `bootstrap × TDR_SHRINK_MAX`. The guess is a starting point, never a floor.
///
/// `bootstrap` is what `bootstrap_steps` derived for this view and mode.
pub(crate) fn budget_base(fe_budget: u64, bootstrap: u64) -> u64 {
    match fe_budget {
        0 => bootstrap,
        b => b.clamp(crate::tunables::cost().tdr_min_steps, crate::tunables::cost().tdr_steps_ceil),
    }
}

/// A dispatch prices the budget only if it is REPRESENTATIVE of it — at least this fraction of
/// `cur`. A smaller one finished quickly because it is small, and reading a budget off it would
/// calibrate against a per-step rate the full frame never runs at.
///
/// ⚠The VALUE is beta.11 determinism-critical and must not move without that lens; it is named
/// here only so the two readers cannot drift. Both are below: [`budget_step`], which discards a
/// reading under it, and [`probe_would_price`], which declines to MANUFACTURE one.
pub(crate) const PRICE_REPRESENTATIVE_FRAC: f64 = 0.7;

/// Would a forced budget-climb probe of `steps` against budget `cur` yield a reading the pricing
/// rule can actually use?
///
/// ⭐**A probe whose reading is pre-destined for the discard pile is pure waste, and it never
/// terminates.** The climb probe fires while the budget has not converged, and a frame smaller
/// than [`PRICE_REPRESENTATIVE_FRAC`] of the budget cannot converge it — [`budget_step`] drops
/// exactly that reading — so the two rules chase each other forever: probe, discard, still
/// unconverged, probe again. Measured on the DEFAULT home view (2026-09-04): `steps=4.288e8`
/// against `budget=9.000e8` (the bootstrap) = 0.48 of it, a real 36.9 ms GPU dispatch every 3
/// frames for as long as the app is left open, on a static picture already at native resolution.
///
/// Declining is safe because the deadlock this probe exists to break is the OPPOSITE regime: a
/// view floored to 16×16 by a budget too small to render it, where the frame is budget-SIZED by
/// construction and so passes this test comfortably. A frame far under budget is not floored —
/// nothing is waiting on a bigger budget, and there is nothing to climb toward.
pub(crate) fn probe_would_price(steps: u64, cur: u64) -> bool {
    (steps as f64) >= cur as f64 * PRICE_REPRESENTATIVE_FRAC
}

/// GPU readings [`budget_step`] would discard (under [`PRICE_REPRESENTATIVE_FRAC`] of the budget,
/// and not slow), pooled per view until together they are representative (`READING_POOL`).
///
/// ⭐⭐**WHY: WHILE THE VIEW MOVES, THE FRAME BUDGET CANNOT LEARN.** A moving frame's work goes out as
/// several SMALL chunk passes, and every one of them is under 0.7× the budget, so every reading is
/// discarded — measured on the RX 6800 XT (2026-09-27, schema-2 record): 339 of 339 and 307 of 312
/// readings while tap-zooming, against half used while still. The budget froze at 4.56e7 for 26 s
/// of taps after a mode switch had reset it, capped every motion pass (87–88% of moving frames), and
/// held the moving picture on the lowest resolution rung; it rose to 1.17e9 the moment motion
/// stopped. Pooled, those readings ARE representative: their steps together reach the bar a single
/// reading must, and their combined rate `Σsteps / Σms` is what the budget is priced on.
///
/// ⚠Pessimistic by construction: every small pass carries its fixed cost (`PASS_FIXED_MS`), so the
/// pooled rate is lower than the true per-step rate and the budget grows more slowly than it could,
/// never faster. It goes through [`budget_step`] like any reading — the reference-build gate, the
/// lethal retreat and the target all apply. It cannot unfreeze a budget stuck HIGH (the 2026-09-21
/// field shape): small passes would need thousands of readings to reach 0.7× of 1.5e11; that is the
/// wall-clock backstop's job, not this.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ReadingPool {
    pub(crate) steps: u64,
    pub(crate) ms: f64,
    pub(crate) n: u32,
    /// The frame of the pool's first reading: a pool older than [`POOL_MAX_FRAMES`], or begun
    /// before the view's last mode switch, mixes regimes and is dropped.
    pub(crate) since_frame: u64,
}

/// A pool older than this (≈2 s at 60 Hz) is dropped rather than priced: its readings describe
/// views the zoom has left.
pub(crate) const POOL_MAX_FRAMES: u64 = 120;

/// Add one discarded reading to `pool`; `Some((Σsteps, Σms, n))` — and an emptied pool — once the
/// pooled steps reach [`PRICE_REPRESENTATIVE_FRAC`] of `cur`. A pool begun before `switch_frame`
/// (the view's last mode switch) or older than [`POOL_MAX_FRAMES`] starts over with this reading.
pub(crate) fn pool_step(
    pool: &mut ReadingPool,
    steps: u64,
    ms: f64,
    cur: u64,
    frame: u64,
    switch_frame: u64,
) -> Option<(u64, f64, u32)> {
    if !(ms > 0.0) || !ms.is_finite() || steps == 0 {
        return None;
    }
    let stale = pool.n > 0
        && (frame.saturating_sub(pool.since_frame) > POOL_MAX_FRAMES
            || (switch_frame != u64::MAX && pool.since_frame < switch_frame));
    if pool.n == 0 || stale {
        *pool = ReadingPool { since_frame: frame, ..Default::default() };
    }
    pool.steps = pool.steps.saturating_add(steps);
    pool.ms += ms;
    pool.n += 1;
    if probe_would_price(pool.steps, cur) {
        let out = (pool.steps, pool.ms, pool.n);
        *pool = ReadingPool::default();
        Some(out)
    } else {
        None
    }
}

/// One step of the frame-budget search, as a pure function so it can be property-tested.
///
/// `cur` is the current budget (already floored at the bootstrap), `steps` the nominal size of the
/// dispatch that was measured, `ms` how long it took. Returns the next budget and whether the
/// reading counts as CONVERGED, or `None` when the reading carries no usable signal.
pub(crate) fn budget_step(cur: u64, steps: u64, ms: f64, explicit: bool) -> Option<(u64, bool)> {
    // Two regimes, one arithmetic. Since 2026-08-16 both converge on the SAME real target (400 ms);
    // `explicit` still differs in carrying a hard nominal ceiling, because skip effectiveness is
    // least predictable there. ~900 ms real is the reproduced lethal band, and two field losses
    // arrived through the AUTO regime at ~1030 ms — which is why the auto target no longer sits
    // inside it (see TDR_BUDGET_MS). The regime lives INSIDE this function so the app controller, the
    // livetest harness, and the tests cannot select it differently (the beta.40 lesson: a harness
    // whose controller differs from the app's measures a view the app never renders).
    let (target_ms, ceil) = if explicit {
        (crate::tunables::cost().tdr_explicit_budget_ms, crate::tunables::cost().explicit_steps_ceil)
    } else {
        (crate::tunables::cost().tdr_budget_ms, crate::tunables::cost().tdr_steps_ceil)
    };
    let slow = ms > target_ms;
    // A fast dispatch far under budget size finished quickly because it is SMALL. One-sided: a
    // SLOW undersized dispatch is the strongest evidence the budget is too high.
    if !probe_would_price(steps, cur) && !slow {
        return None;
    }
    // ⭐LATENCY-FLOOR GUARD. A dispatch that is already SMALL (sub-bootstrap nominal) and still
    // measures slow is latency-bound, not over-sized — a 16×16 frame at a deep iteration ask
    // runs a few hundred dependent-chain milliseconds *no matter how few pixels*, so shrinking
    // further buys nothing and corners the controller at the floor (measured on `--livetest`:
    // six deep holds pinned at TDR_MIN_STEPS/16×16 with every reading 250–330 ms "slow"; the
    // pre-existing "steps ∝ time is FALSE at latency-bound frames" lesson, met from the shrink
    // side). Hold position and report converged: the view is as good as this target allows, and
    // anything that grows the frame (a settle tile, a regime change, motion) re-measures.
    //
    // ⚠BOUNDED by TDR_LATENCY_ACCEPT_MS: a floor NEAR the watchdog band must NOT be held — a
    // 16×16 dispatch at a 2–4M ask has a multi-second chain latency, and "hold position" there
    // is the beta.48 death loop (sub-bootstrap dispatches at ~1 s, back to back, until Event
    // 153). Past the accept bound the shrink proceeds: it may not beat chain latency, but fewer
    // resident warps is the only safe direction, and the honest fix for that regime is iteration
    // CHUNKING (mode-2 extension, TODO), not bigger holds.
    //
    // ⚠**THE DISPATCH MUST BE REPRESENTATIVE OF THE BUDGET** (`steps >= cur * 0.7`), and leaving that
    // out was a real exposure. The other two conditions test the dispatch's ABSOLUTE size and its
    // time; neither notices whether `cur` is small. So a clamped edge tile, a chunk step, or a
    // motion-shrunk frame measuring inside the accept window while `cur` sat at a converged 3e11 was
    // read as "this is the latency floor, hold position" AND returned converged — which arms the
    // tiled settle on a budget calibrated for a per-step rate that no longer exists. But an
    // UNDERSIZED dispatch taking 500 ms is the strongest available evidence that per-step cost has
    // COLLAPSED (~2e5 steps/ms here); the next full-size tile at `cur` then prices in tens of
    // seconds, and the emergency retreat cannot save it because its first reading lands two frames
    // late, behind two more budget-sized dispatches. That is the shape of both 2026 field losses.
    //
    // ⚠It was UNREACHABLE in the auto regime until 2026-08-16 — the comment here used to say so,
    // resting on auto's `slow` starting at 900 ms, past the 600 ms accept bound. Lowering the auto
    // target to 400 ms opened a live 400–600 ms window, and the auto ceiling is 3e11 against
    // explicit's 6e10, so it opened it where the blast radius is largest. 0.7 is deliberately the
    // same "this dispatch reflects the budget" threshold the discard test above uses.
    if slow
        && steps <= crate::tunables::cost().tdr_bootstrap_steps
        && ms <= crate::tunables::cost().tdr_latency_accept_ms
        && probe_would_price(steps, cur)
    {
        return Some((cur, true));
    }
    // ⭐EMERGENCY RETREAT. Past `TDR_LETHAL_MS` the shrink cap is removed and the raw ratio applies.
    //
    // The cap is right while walking toward a target and actively harmful once a reading lands in
    // the watchdog band: the 2026-08-16 device loss measured 1033 ms wanting ×0.39, got clamped to
    // ×0.5, and needed two-plus readings to recover — with readings deferred ~2 frames and every
    // frame between them another ~1 s dispatch. It died after three. The controller had already
    // SEEN the danger on the first reading; what killed it was how slowly it was allowed to back
    // away. Overshooting into too small a frame is cheap and self-correcting (the next reading
    // grows it at ×1.5); overshooting into the watchdog is not.
    let lethal = ms >= crate::tunables::cost().tdr_lethal_ms;
    let factor = if lethal {
        (target_ms / ms).min(crate::tunables::cost().tdr_grow_max)
    } else {
        (target_ms / ms).clamp(crate::tunables::cost().tdr_shrink_max, crate::tunables::cost().tdr_grow_max)
    };
    let base = if slow { cur.min(steps.max(1)) } else { cur };
    let next = ((base as f64 * factor) as u64).clamp(crate::tunables::cost().tdr_min_steps, ceil);
    // Converged when the factor sits in the dead band — or when the clamp pinned `next` at `cur`
    // (a ceiling-pinned explicit budget is as converged as it is allowed to get; this is what
    // lets `fe_budget_ok` arm the tiled settle without a special case).
    Some((next, next == cur || (0.8..=1.25).contains(&factor)))
}

/// Gate a budget step on whether a REPLACEMENT REFERENCE is currently being built for this view.
///
/// Returns the budget to install and whether it may be treated as converged.
///
/// ⭐**Growth is refused; shrink is not.** A frame rendered while a rebuild is in flight is priced
/// against a reference that is about to be thrown away — usually a cheap frame, because the pixels
/// are still clamped to the old (shorter) orbit — so growth measured there is unearned. Shrink is
/// always honoured: a slow frame is bad news regardless of what happens next.
///
/// **The incident.** 2026-08-16 (`reports/fractadyne-report.-2026-08-16a.txt`), a settled tiled view
/// at zoom 3.4e38. Inside the ~2 s window containing one reference build, `fe_budget` went
/// 4.520e10 → 3.000e11 — the `TDR_STEPS_CEIL` — which is ×1.5 growth across roughly five readings
/// on frames cheap enough that none crossed the slow-frame log threshold. The new reference then
/// landed, per-frame nominal cost jumped 27× to 7.546e11, and the tiled grid dispatched tiles sized
/// against that inflated budget: 1033 ms and 1026 ms, then the device was gone.
///
/// The budget was earned under one cost regime and spent under another. Nominal steps are supposed
/// to be regime-independent currency, and they are not — per-step real cost moves with skip
/// effectiveness, and a pending rebuild is a loud advance warning that skip effectiveness is about
/// to change. This declines to bank a windfall measured in the last moments before that.
///
/// ⚠Also withholds CONVERGENCE while a build is in flight, which keeps the tiled settle disarmed:
/// arming a full-resolution tiled grid on a budget that is about to be invalidated is the specific
/// mistake that made the above fatal rather than merely slow.
pub(crate) fn budget_after_build_gate(cur: u64, next: u64, ok: bool, building: bool) -> (u64, bool) {
    if building && next > cur {
        (cur, false)
    } else {
        (next, ok && !building)
    }
}

/// Has measurement STARVED for this view? True when a real dispatch has gone out since the last
/// timing came back and nothing has arrived for `limit` frames. An idle view (no dispatch since
/// the last reading) is never starved — there is nothing outstanding to price.
pub(crate) fn measurement_starved(
    dispatch_frame: u64,
    reading_frame: u64,
    now: u64,
    limit: u64,
) -> bool {
    dispatch_frame > reading_frame && now.saturating_sub(reading_frame) >= limit
}

/// One step of the WALL-CLOCK cost probe: fold `dt` into the in-flight probe and, once two frames
/// have passed since it was armed, return the `(ms, steps)` pair to price.
///
/// ⚠A dispatch is not priced by the interval it was submitted in — submission is asynchronous, so
/// the frame that queues 2 s of GPU work still returns in a vsync-shaped ~17 ms and the cost only
/// surfaces when the queue stalls a later acquire. Pricing in place read **18.4 ms for a 1070 ms
/// frame** at the beta.48 device loss and grew the budget on it. `aa_probe` already had this right;
/// this is the same shape, extracted so the deferral is pinned by a test rather than by reading.
///
/// `probe` is `(steps, armed_frame, max_dt_seen)`. `dispatched` says this frame really re-iterated
/// and `steps` is what it dispatched.
pub(crate) fn wall_probe_step(
    probe: Option<(u64, u64, f64)>,
    dispatched: bool,
    steps: u64,
    frame_idx: u64,
    dt_ms: f64,
) -> (Option<(u64, u64, f64)>, Option<(f64, u64)>) {
    match probe {
        Some((s, armed, max_dt)) => {
            let max_dt = max_dt.max(dt_ms);
            if frame_idx >= armed + 2 {
                (None, Some((max_dt, s)))
            } else {
                (Some((s, armed, max_dt)), None)
            }
        }
        None if dispatched && steps > 0 => (Some((steps, frame_idx, dt_ms)), None),
        None => (None, None),
    }
}

/// PRESENT-THROTTLE detector step. A frame that submitted NO new GPU work yet still took
/// `thresh_ms` or longer measured the PRESENT PATH, not the queue: with nothing newly
/// dispatched, the interval is acquire/present/idle by construction. A run of such frames is
/// a compositor throttling this window (occluded/background windows present at ~1 Hz on
/// Windows), and under it every wall-clock premise in the controller inverts:
///   * the chunk drain's "a quick present proves the queue drained" can NEVER pass, so a
///     serialized pass wedges, its wall accumulator inflates on pure present-wait, and the
///     band licences get lethal-shed with the GPU idle;
///   * the wedged (held, deduped) pass produces no new timestamps, so the measurement-
///     starvation latch fires and hands pricing to the wall clock;
///   * the wall clock then prices real dispatches at the throttle interval (~1 s) and
///     crushes the frame budget to its floor.
/// Measured end to end in the 2026-09-04 field log (6 h at ~1.26 fps, budget pinned at
/// bootstrap, `IN-FLIGHT PASS IN THE LETHAL BAND: acc=949–1117ms (no drain yet)`, dozens of
/// starvation latch/unlatch cycles) and reproduced from a copy of the session at idle. The
/// counter resets on any frame that dispatched or presented quickly; `present_throttled`
/// (>= [`PRESENT_THROTTLE_FRAMES`]) gates all three mechanisms — they FREEZE rather than
/// walk on poisoned readings, and the drain releases on a short un-priced backstop so the
/// walk still completes (bounded to one pass per backstop, so passes cannot stack).
pub(crate) fn present_throttle_step(count: u32, dispatched: bool, dt_ms: f64, thresh_ms: f64) -> u32 {
    if !dispatched && dt_ms >= thresh_ms {
        count.saturating_add(1)
    } else {
        0
    }
}

/// Consecutive no-work slow frames that establish "this window's presents are throttled".
/// Five ≥100 ms present-only intervals is half a second of proof; a genuinely busy queue
/// cannot produce them (busy queues stall frames that DID dispatch, which reset the count).
pub(crate) const PRESENT_THROTTLE_FRAMES: u32 = 5;

/// Should an IN-FLIGHT pass shed the band ledger on its accumulated wall time alone?
///
/// ⭐⭐The wall-clock retreat's whole decision, as a pure predicate. Until beta.141 the ledger
/// learned nothing about a pass until it DRAINED (a quick present as proof the queue emptied) or
/// hit the wedge backstop at `target × 60` — ~24 s at a 400 ms target, long after a device would be
/// gone. That is the gap the 2026-08-22 field loss fell through: a queue on its way to losing the
/// device never produces a quick present, so the one pass whose price would shed the licence is
/// exactly the pass that cannot be priced. Accumulated wall is a conservative LOWER BOUND — if it
/// has reached the lethal band the pass has *already* been that expensive, no drain required.
///
/// It was extracted from `update()` because the item that asked for it recorded the gap plainly:
/// the decision had **empirical verification (46 observed fires) but no unit test**, being inline
/// state across frames rather than a predicate. One observed run is not a gate; this is.
///
/// Three guards, and each one is a bug that happened:
/// - ⚠**`shed` latches.** A pass sitting in the band for many frames sheds ONCE — the ledger is
///   already at the floor, and re-shedding every frame is noise that hides the first one.
/// - ⚠⚠**`throttled` vetoes.** Under present throttling the accumulator measures the COMPOSITOR,
///   not the GPU (see `present_throttle_step`), so shedding on it would floor every licence off an
///   *idle* card. Those are the 2026-09-04 field log's exact lines, and this is the same
///   present-throttle poisoning of a wall-clock controller that the dual-view Julia saga turned on.
/// - ⚠A NaN or negative accumulator never sheds, and the `>=` alone is what enforces that: every
///   comparison against NaN is false, and a negative is below any threshold. ⭐An explicit
///   `is_finite()` guard was written here first and was WRONG — it also rejected `+∞`, which is the
///   most lethal reading there is and exactly what this should fire on. The test caught it.
///
/// ⭐**Shed, do not release**: this says only what the NEXT pass may ask for. The next dispatch
/// stays blocked until the genuine drain — "releasing the next dispatch onto a pass that is merely
/// SLOW is how a 10 s single got company on its way to the TDR".
pub(crate) fn wall_shed_now(shed: bool, throttled: bool, acc_ms: f64, lethal_ms: f64) -> bool {
    !shed && !throttled && acc_ms >= lethal_ms
}

/// Consecutive wall-slow frames with no slow reading before the budget-blind tripwire fires.
///
/// Eight is about a second of a dive in this regime and well short of the three-or-four fatal
/// frames both recorded device losses took from first warning sign to the end, so the warning
/// lands while there is still something to read. Low enough to catch the episode, high enough
/// that an ordinary hitch — a reference install, an alt-tab, one expensive settle — does not
/// trip it.
pub(crate) const BUDGET_BLIND_FRAMES: u32 = 8;

/// ⭐⭐**IS THE FRAME BUDGET STEERING BLIND?** True when the wall clock has called this many
/// frames in a row slow while the budget controller has been handed no slow reading at all.
///
/// The two measure different things ON PURPOSE — a GPU timestamp brackets the dispatch, the wall
/// interval contains the queue, the present and the compositor — and the difference is normally
/// what you want: pricing the budget by the wall would hand it to vsync. But when they diverge
/// for a sustained run, the controller is not merely reading a different number, it is reading a
/// number that cannot see the danger: on the 2026-09-21 RX 6800 XT loss the wall showed twenty
/// frames from 200 ms to 1027 ms while the always-on lethal-band line never fired even once,
/// because every reading the controller received described a short pass sitting in a long queue.
/// The budget stayed at 1.515e11 throughout, and since `chunk_over` compares against that budget,
/// the frame was never chunked either — the guard that bounds such a dispatch was switched off by
/// the same blindness.
///
/// ⭐From beta.131 it ACTS (`DEAD_MAN`, default on; see [`dead_man_budget`]). It reported only
/// until the reaction's §8 prerequisites were in hand: the field trap reproduced on stock logic
/// (`FRACTADYNE_SEED_BUDGET`, 2026-09-27: every reading discarded, budget 1.515e11 the whole run)
/// and zero natural fires across ~45 healthy runs on both cards (the two in the batteries were
/// `--recordtest`'s injected phase). The first field record of a whole episode is still owed.
pub(crate) fn budget_blind(slow_wall_frames: u32, slow_readings: u32, warned: bool) -> bool {
    !warned && slow_readings == 0 && slow_wall_frames >= BUDGET_BLIND_FRAMES
}

/// ⭐⭐THE LETHAL-BAND TRIGGER (beta.132): ONE frame at or past `TDR_LETHAL_MS` by the wall, with a
/// repaint requested and no slow reading, latches the tripwire at once — [`budget_blind`]'s eight
/// frames are far too slow for the card that fails. Measured on PLUTO (2026-09-27, the dead-man
/// BEFORE arm, a learned 1.515e11 at the crash view): whole-frame dispatches of 315, 1,013 and
/// 1,393 ms at the start, and 640 then 2,017 ms at the settle — the machine hung on the fifth. The
/// longest run over 400 ms was 2, so the 8-frame trigger never fired; this one would have at the
/// 1,013 ms frame, derating the budget before any of the later dispatches were sized. The cost of a
/// false trigger is a derate (a few coarse frames while the budget re-climbs); of a missed one, the
/// machine.
pub(crate) fn budget_blind_lethal(dt_ms: f64, busy: bool, slow_readings: u32, warned: bool) -> bool {
    !warned && busy && slow_readings == 0 && dt_ms >= crate::tunables::cost().tdr_lethal_ms
}

/// The part of a frame interval the budget-blind tripwire may judge: the wall interval less the
/// time the frame spent in a SYNCHRONOUS OFFSCREEN render on the UI thread (a CLI `--render`, a
/// `--render-tour`, a glitch-corrected export). That render is the app's own blocking work, not a
/// live dispatch the frame budget sized, and nothing the dead-man does can shorten it.
///
/// Without this, every CLI render past `TDR_LETHAL_MS` (0.9 s) fired the lethal trigger on the
/// frame after it and failed its own log check — exit 1 on a render that succeeded (found
/// 2026-09-27: beta.131 exit 0, beta.134 exit 1 on the same 7.7 s benchmark render; the trigger
/// arrived in beta.132). Only the offscreen part is discounted: anything else in the frame body
/// still counts, which is how `--recordtest` injects its blind and wedged frames.
pub(crate) fn tripwire_dt(last_dt_ms: f64, offscreen_ms: f64) -> f64 {
    (last_dt_ms - offscreen_ms.max(0.0)).max(0.0)
}

/// ⭐⭐THE WALL-CLOCK DEAD-MAN's budget (design §5.1, §8): when [`budget_blind`] latches, the view's
/// budget drops to its bootstrap — the opening guess a view uses before anything is measured,
/// itself capped at `TDR_BOOTSTRAP_STEPS` — and may not grow again until the latch clears.
///
/// Every dispatch path sizes from this one number (`tdr_steps`: the chunk step, `chunk_over`'s
/// comparison, the resolution shrink, the tile count), so lowering it bounds all of them without a
/// new actuator — the 2026-09-21 failure was a budget that "switched off" chunking by being too
/// large. It adopts no theory of why the readings were blind; the wall is its only witness. A
/// budget already at or under the bootstrap, or unmeasured (0), is left alone.
pub(crate) fn dead_man_budget(cur: u64, bootstrap: u64) -> u64 {
    if cur == 0 {
        0
    } else {
        cur.min(bootstrap)
    }
}

/// Does a budget decision EXPLAIN a run of slow frames, so the tripwire may start counting again?
///
/// ⭐⭐**Only a DECREASE does.** The tripwire used to restart on any budget that MOVED — growth
/// included (finding U15). A budget climbing on every short reading while the wall climbs toward a
/// second is the runaway this instrument exists for, and resetting on each of those readings kept
/// its count below [`BUDGET_BLIND_FRAMES`] forever: the one always-on warning for the class was
/// least sensitive exactly where the budget was running away. A shrink is the controller reacting
/// to the slowness; growth is the controller being told the opposite of what the wall says.
pub(crate) fn blind_reset_on(cur: u64, next: u64) -> bool {
    next < cur
}

/// Does this dispatch count toward the motion-jam backlog? Full-size means >= 0.7x the learned
/// budget — deliberately the SAME representativeness threshold `budget_step` uses to accept a
/// reading, so "counts as backlog" and "would move the budget" are one definition.
pub(crate) fn motion_jam_counts(dispatch_steps: u64, learned_budget: u64) -> bool {
    dispatch_steps as f64 >= learned_budget as f64 * 0.7
}

/// The motion-jam gate: clamp this frame's allowance to the bootstrap? Pure so the rules are
/// pinned by test — motion only (settled paths serialize themselves), never on a reproject frame
/// (nothing dispatches), never offscreen (exports have their own wall-priced cap).
pub(crate) fn motion_jammed(
    unpriced: u32,
    max: u64,
    interacting: bool,
    offscreen: bool,
    will_reproject: bool,
) -> bool {
    interacting && !offscreen && !will_reproject && unpriced as u64 >= max
}

pub(crate) fn chunk_band_license(
    bands: &[u32; crate::tunables::CHUNK_BANDS],
    band: usize,
    floor: u32,
) -> u32 {
    let b = bands[band.min(crate::tunables::CHUNK_BANDS - 1)];
    if b > 0 {
        b.max(floor)
    } else {
        // ⚠UNVISITED territory opens at the FLOOR — never at a neighbour's license. The final
        // repro death was exactly an inherited license: a 36k-step pass earned on a band's cold
        // beginning ran TEN SECONDS when the band turned hot mid-way (the storm starts before
        // the wrap point, inside the band). The floor is the worst-case-prior sizing: first contact
        // with ANY region — however hostile — must be a survivable single, and everything bigger
        // must be earned by that region's own prices.
        //
        // ⛔**THE "~70 ms" THIS COMMENT USED TO QUOTE FOR 256 ITERATIONS IS STALE — corrected
        // 2026-08-24.** The 2026-08-22 field loss ran **1024 iterations in 1216 ms**, i.e.
        // **1.19 ms per iteration**, so 256 iterations on that region is **~304 ms** (~380 ms if
        // priced sub-linearly at the sweep's measured 1.79× per doubling). Still survivable
        // against the 900 ms lethal band, but by ~2.4×, not the ~13× the old figure implied — and
        // a region only **3× more hostile** would make the floor ITSELF lethal
        // (900 / 256 = 3.5 ms per iteration).
        // ⭐Which is why `budget_step`'s floor is no longer a bare 256: it is now the SMALLER of
        // 256 and what the worst measured rate says fits the target wall. This licence floor is
        // still a constant, because an unvisited band has no measured rate of its own to size
        // from — that is the whole point of it — but it inherits the caller's `floor`.
        floor
    }
}

/// Which band of the iteration axis does a cursor at `cur` belong to?
///
/// **OCTAVES FROM A FIXED BASE, NOT A FRACTION OF THE ASK** — band 0 is `[0, 256)`, band `k` is
/// `[256·2^(k-1), 256·2^k)`, and the last band absorbs everything above.
///
/// ⭐⭐**WHY THIS CHANGED (2026-08-24).** It used to be `cur · CHUNK_BANDS / gpu_iter`, i.e. the
/// ask cut into 16 equal parts — so a band's WIDTH grew with the ask, and the ledger's resolution
/// collapsed at exactly the depths where device losses happen. At the 2026-08-22 field loss's ask
/// of **231,676** iterations each band spanned **14,480** iterations, and the fatal window
/// `[768, 1792)` sat in band 0 — holding a licence earned by the passes at iterations 0..768, the
/// cheap start of the orbit. Under octaves that window is band 2, `[512, 1024)`, whose licence can
/// only have been earned inside those 512 iterations: a **28× finer** attribution right where it
/// mattered.
///
/// ⭐This is the ledger's own warning, one level up. `chunk_band_license` already refuses to let
/// UNVISITED territory inherit a neighbour's licence, because "a 36k-step pass earned on a band's
/// cold beginning ran TEN SECONDS when the band turned hot mid-way — the storm starts before the
/// wrap point, INSIDE the band". That fix bounded the gap BETWEEN bands; nothing bounded the gap
/// WITHIN one, and a 14,480-iteration band has a great deal of "within".
///
/// ⭐**Octaves rather than fixed width** because cost structure along an orbit is scale-free: the
/// interesting features (wrap points, storms) sit at multiplicatively-spaced depths, and a fixed
/// width fine enough for `cur = 768` would need thousands of bands to reach a 4M ask. 16 octaves
/// from 256 cover `cur` up to 4.2M, past any ask this path runs.
pub(crate) fn chunk_band_of(cur: u32) -> usize {
    let base = 256u32;
    if cur < base {
        return 0;
    }
    // `k` such that cur ∈ [base·2^(k-1), base·2^k): the octave index of cur/base.
    let octave = u32::BITS - (cur / base).leading_zeros(); // cur/base ≥ 1 ⇒ octave ≥ 1
    (octave as usize).min(crate::tunables::CHUNK_BANDS - 1)
}

/// The first iteration PAST the band `cur` sits in — where a pin pass that started at `cur` must
/// stop so the next band is entered under its own licence (`chunk_band_of` gives the band; this
/// is its upper edge). Band 0 is `[0, 256)`; band k ≥ 1 is `[256·2^(k−1), 256·2^k)`. The last
/// band is open-ended (its "end" saturates), matching the ledger's clamp.
pub(crate) fn chunk_band_end(cur: u32) -> u32 {
    let base = 256u32;
    if cur < base {
        return base;
    }
    let octave = u32::BITS - (cur / base).leading_zeros();
    if octave as usize >= crate::tunables::CHUNK_BANDS - 1 {
        return u32::MAX;
    }
    base.checked_shl(octave).unwrap_or(u32::MAX)
}

/// Shed the whole band ledger for a view: every band reopens at the floor (`chunk_band_license`
/// treats 0 as unvisited). Called by the emergency retreat when a frame prices in the lethal band.
///
/// ⭐⭐**WHY THE RETREAT MUST DO THIS ITSELF, rather than trust `chunk_band_update` to shed.**
/// The ordinary cliff rule quarters a band on a price past 2× target — but it only runs on a pass
/// the walk judges DRAINED (`last_dt_ms < CHUNK_DRAIN_DT_MS`, the quick present). A queue on its
/// way to losing the device is saturated, presents stop coming back quick, and the hot pass stays
/// `chunk_inflight` — unpriced. So the one pass whose price would shed the licence is exactly the
/// pass the gate can silence, and the band keeps a licence earned on this region's cheap frames.
/// That is the 2026-08-22 field loss's shape: `chunk=[768,1792)`, a 1024-iteration window still
/// sized by a licence, at a moment nothing was being priced.
///
/// ⭐**Shedding is OUTPUT-NEUTRAL, which is what makes it safe on a hot path.** The window size
/// sets only where a pass stops and resumes, never what it computes: the chunked iterate's
/// contract is bit-identical output for any `chunk_iters`, gated by the "chunked render is
/// bit-identical" selftests and by `--chunk-sweep`'s pixel-identity assertion. Shedding costs
/// throughput (the walk re-climbs from the floor, ~8 priced passes on the ×2 fast lane) and
/// nothing else. On a frame that came within ~2× of losing the device, that is the right trade.
///
/// ⚠ALL bands, not the current one. The lethal reading arrives ~2 frames after the dispatch it
/// prices, so by the time it lands the cursor has moved and the band it refers to is no longer
/// the band about to be dispatched — the same staleness that keeps the window off the lethal log
/// line. Shedding only "the" band would shed the wrong one.
pub(crate) fn chunk_band_retreat(bands: &mut [u32; crate::tunables::CHUNK_BANDS]) {
    *bands = [0; crate::tunables::CHUNK_BANDS];
}

/// Feed one priced pass into the band ledger. Acceptance is AT the target (a price accepted
/// above it and then grown compounds straight into the ~900 ms lethal band — measured, twice);
/// a price past 2× target is a cliff and QUARTERS the record (never zeroes it: a shed band is
/// still priced knowledge — "small is what this region affords" — where zero would fall back to
/// the neighbour's license, which is exactly the license that just failed). Prices between 1×
/// and 2× hold: neither evidence for growth nor a cliff.
#[cfg_attr(not(test), allow(dead_code))] // the ×2 form is the ledger's documented contract; the live path passes its lane explicitly
pub(crate) fn chunk_band_update(
    bands: &mut [u32; crate::tunables::CHUNK_BANDS],
    band: usize,
    size: u32,
    price_ms: f64,
    target_ms: f64,
) {
    chunk_band_update_with_lane(bands, band, size, price_ms, target_ms, 2);
}

/// `chunk_band_update` with the fast lane's growth factor as a parameter. The settled walk keeps
/// ×2 (it has the 400 ms target's worth of time); a PINNED refresh uses ×4, because its prices
/// are completion-based with the pipeline latency subtracted, so "clearly cheap" there means the
/// pass finished inside its own frame — and a pin ramping ×2 per pass through eight cold bands
/// (~36 passes at one pass per two frames) outran the 2-octave drift abandon at 4.0× every time,
/// leaving the held frame 8 octaves stale at the 1e4 hand-over (measured, build 3228).
pub(crate) fn chunk_band_update_with_lane(
    bands: &mut [u32; crate::tunables::CHUNK_BANDS],
    band: usize,
    size: u32,
    price_ms: f64,
    target_ms: f64,
    fast_lane: u32,
) {
    let band = band.min(crate::tunables::CHUNK_BANDS - 1);
    // NaN/negative are garbage; ZERO is not — it is a pass whose accumulated wall time was
    // literally unmeasurable (headless harnesses, and any pass cheaper than the clock), which is
    // the strongest possible "clearly cheap" and takes the fast lane below.
    if !price_ms.is_finite() || price_ms < 0.0 {
        return;
    }
    if price_ms <= target_ms * 0.5 {
        // Clearly cheap: the fast lane (×2 settled, ×4 pinned) is what keeps healthy walks
        // quick despite every band opening at the floor — full size in ~8 priced passes.
        bands[band] = bands[band].max(size.saturating_mul(fast_lane.max(2)));
    } else if price_ms <= target_ms {
        bands[band] = bands[band].max(size.saturating_add(size / 4));
    } else if price_ms > target_ms * 2.0 {
        // A cliff: quarter the record (never to zero — a shed band is still priced knowledge;
        // zero would reopen at the floor, which is FINE, but keeping a small non-zero record
        // spares the re-climb from scratch).
        bands[band] = (size / 4).max(1);
    } else {
        // Above target but short of a cliff: drift DOWN (×0.5). Holding the line here let an
        // over-target size re-dispatch itself forever inside the (1x, 2x] gap.
        bands[band] = (size / 2).max(1);
    }
}
