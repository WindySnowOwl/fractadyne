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
    budget_base_in(
        fe_budget,
        bootstrap,
        crate::tunables::cost().tdr_min_steps,
        crate::tunables::cost().tdr_steps_ceil,
    )
}

/// [`budget_base`] with its two overridable bounds (`TDR_MIN_STEPS`, `TDR_STEPS_CEIL`) passed in: the
/// form a lifted decision calls, since it may read only its inputs (`render::decide`).
pub(crate) fn budget_base_in(fe_budget: u64, bootstrap: u64, min_steps: u64, ceil: u64) -> u64 {
    match fe_budget {
        0 => bootstrap,
        b => b.clamp(min_steps, ceil),
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

/// PRESENT-THROTTLE detector step. A frame that submitted NO new GPU work, with none of its
/// earlier work still queued, yet still took `thresh_ms` or longer measured the PRESENT PATH,
/// not the queue. `dispatched` is therefore "dispatched, OR has work of its own outstanding" at
/// the caller (completion callbacks, `Perf::view_outstanding`): a hold behind a running pass is
/// the backlog, and counting it inverted the detector exactly there (design U11/U12). A run of such frames is
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

/// Arithmetic modes are `RenderMode::to_u32()` ∈ {0, 1, 2}; `u32::MAX` means "none yet" and is
/// simply out of range, so [`slot`] rejects it along with anything a future mode adds.
pub(crate) const MODE_RATE_SLOTS: usize = 3;

fn slot(mode: u32) -> Option<usize> {
    ((mode as usize) < MODE_RATE_SLOTS).then_some(mode as usize)
}

/// The rate a wall-clock cut may take the motion estimate to from `cur`, given the interval's
/// own `wall_rate`: the interval's rate, bounded to `cur / MOTION_CUT_MAX` in one event. `None` =
/// the interval does not say the rate is lower (or says nothing usable), so leave it alone.
/// Pure, so the bound is pinned by test — see the constant for the 6–9× cuts it exists for.
pub(crate) fn bounded_motion_cut(cur: f64, wall_rate: f64) -> Option<f64> {
    if !wall_rate.is_finite() || !(wall_rate > 0.0) {
        return None;
    }
    if !(cur > 0.0) {
        return Some(wall_rate); // nothing measured yet: the interval is the first measurement
    }
    if wall_rate >= cur {
        return None;
    }
    Some(wall_rate.max(cur / crate::tunables::MOTION_CUT_MAX))
}

/// What an emergency retreat reached (design/live-render-robustness.md §5.6, W3 gate 2).
///
/// ⭐"Nothing happened" is a VALUE, not a silence. A retreat used to shed the band ledger and cut
/// the budget and log the same line whether or not that could make the next dispatch any smaller
/// than the one that came back lethal; a retreat at the floor reported a retreat. Now each one says
/// which it was, in its always-on log line and in the frame record (`retreat`, schema 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum RetreatOutcome {
    /// The next dispatch can be smaller than the lethal one: `from` → `to` (iterations for an
    /// in-flight chunk pass, nominal steps for a frame's reading).
    Applied { from: u64, to: u64 },
    /// What the retreat drives is already at its floor, and the floor is not below the dispatch
    /// that was lethal: nothing it can reach makes the next one smaller.
    NoActuatorReachable { at: u64 },
}

impl RetreatOutcome {
    /// The frame record's `retreat` value: 1 applied, 2 no actuator reachable (0 = no retreat).
    pub(crate) fn code(self) -> u8 {
        match self {
            RetreatOutcome::Applied { .. } => 1,
            RetreatOutcome::NoActuatorReachable { .. } => 2,
        }
    }
}

/// The outcome of a retreat from a `lethal` dispatch when the least the next one can carry is
/// `next` (same unit). `#[must_use]`: a caller that drops the outcome has gone back to the silence.
#[must_use]
pub(crate) fn retreat_outcome(lethal: u64, next: u64) -> RetreatOutcome {
    if next < lethal {
        RetreatOutcome::Applied { from: lethal, to: next }
    } else {
        RetreatOutcome::NoActuatorReachable { at: lethal }
    }
}

/// How long the pin gate waits for the pass in flight to complete before it releases the next one
/// anyway, microseconds (findings U3/U47, design §5.6).
///
/// ⚠**A latency escape, demoted to a "the callback is not coming" escape.** The flat
/// `PIN_COMPLETION_TIMEOUT_US` (170 ms) exists for harnesses that render without the event loop, so
/// never register the completion callback (`registered` false). As a latency bound it opened the
/// gate on exactly the hardware it protects: a pin pass that runs longer than 170 ms got a second
/// one behind it in the queue. With the callback registered the completion WILL come, so the wait
/// scales with the frame interval (`frame_dt_ms`; a pass that long stretches the frames around it)
/// to `PIN_TIMEOUT_FRAMES` of them, never under the flat value and never past
/// `SETTLE_COMPLETION_TIMEOUT_US`, past which a completion is a lost device, not a slow pass.
/// `scaled` false (`PIN_TIMEOUT_SCALED=0`) is the flat timeout, always.
pub(crate) fn pin_gate_timeout_us(registered: bool, frame_dt_ms: f64, scaled: bool) -> u64 {
    let flat = crate::tunables::PIN_COMPLETION_TIMEOUT_US;
    if !scaled || !registered {
        return flat;
    }
    let dt_us = if frame_dt_ms.is_finite() && frame_dt_ms > 0.0 {
        (frame_dt_ms * crate::tunables::PIN_TIMEOUT_FRAMES * 1000.0) as u64
    } else {
        0
    };
    dt_us.max(flat).min(crate::tunables::SETTLE_COMPLETION_TIMEOUT_US.max(flat))
}

/// May the pin pass in flight be PRICED now (and the next one released)? By its own GPU timestamp
/// (`gpu_ms` ≥ 0) if one has landed, else as soon as its completion has been seen (`inflight ==
/// 0`: one pass in flight, `PIN_INFLIGHT_MAX`), and in any case once `PIN_PRICE_WAIT_FRAMES` have
/// passed (a completion that is not coming).
///
/// ⚠**Why the completion clause.** Priced by a reading or a 5-frame wait alone, a pin pass went
/// unpriced whenever a new pass replaced it first: without GPU timestamps none was ever priced (the
/// band licence never grew, a pin crawled ~1,500 iterations a frame toward a 1,000,000 ask, and
/// `--motiontest` failed A2 with `FRACTADYNE_NO_TIMESTAMPS=1`, on main's build too), and with them
/// 34 of 622 were.
pub(crate) fn pin_pass_priceable(gpu_ms: f64, frames_since: u64, inflight: u32) -> bool {
    gpu_ms >= 0.0 || inflight == 0 || frames_since >= crate::tunables::PIN_PRICE_WAIT_FRAMES
}

/// One reading of `Meter::mode_rate`: `cur` (0 = none yet), a reading's `rate`, and the seconds
/// since the slot last took one. A slower reading is adopted at once (the pessimistic latch); a
/// faster one raises the slot by at most 2× per `MODE_RATE_DOUBLING_S`, and never past itself.
pub(crate) fn mode_rate_step(cur: f64, rate: f64, elapsed_s: f64) -> f64 {
    if cur.is_nan() || cur <= 0.0 || rate <= cur {
        return rate;
    }
    let cap = cur * 2f64.powf(elapsed_s.max(0.0) / crate::tunables::MODE_RATE_DOUBLING_S);
    rate.min(cap)
}

/// The rate a motion pass may LIFT the motion estimate to from `cur`, given the rate it proved:
/// at most `cur × MOTION_LIFT_MAX` in one lift, the mirror of [`bounded_motion_cut`]. `None` = it
/// proves nothing higher (or nothing usable).
pub(crate) fn bounded_motion_lift(cur: f64, proved: f64) -> Option<f64> {
    if !proved.is_finite() || proved <= 0.0 {
        return None;
    }
    if cur.is_nan() || cur <= 0.0 {
        return Some(proved);
    }
    if proved <= cur {
        return None;
    }
    Some(proved.min(cur * crate::tunables::MOTION_LIFT_MAX))
}

/// The measured rates the pricing family reads, per view and per arithmetic mode (design/
/// live-render-robustness.md §5.1, step M3). `Perf` owns one and passes in what only it knows: the
/// view's current mode, and whether a reading straddles that view's last mode switch.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Meter {
    /// ⭐Measured nominal steps per millisecond, per view and per arithmetic mode (0.0 = never
    /// measured). This is what makes the opening guess hardware-derived instead of assumed; see
    /// `render::bootstrap_steps` and `tunables::TDR_BOOTSTRAP_MS`.
    ///
    /// ⚠**Latches the MINIMUM, never an average.** Within one mode the per-step rate swings by two
    /// orders of magnitude depending on whether a BLA tree is live — the same mode-2 view measured
    /// ~5.7e5 steps/ms with `bla_skip=0` right after a crossover and is worth a budget 3.4× larger
    /// once BLA is skipping (2026-08-15 field report). An average, or a last-value, would let a
    /// BLA-live reading size the opening dispatch of a no-BLA re-entry, which is exactly the frame
    /// that lost the device. The pessimistic latch costs at most a few ×1.5 climb frames; the
    /// optimistic one costs the device, and the asymmetry is the whole design.
    ///
    /// ⭐**…but not for the whole session (U23).** A slower reading is still adopted at once; a
    /// faster one may raise it by at most 2× per `MODE_RATE_DOUBLING_S` ([`mode_rate_step`]). As a
    /// session-long minimum it kept the worst reading of any pass forever: on the RTX 3080 budget
    /// trap the small moving passes of an escaped-reference storm took it from 4e7 to 1.5e6
    /// steps/ms and the opening guess from 4e8 to 5.9e7, on the RX 6800 XT to 6.8e5 and 2.7e7, for
    /// the rest of the session; with B4 that guess is the unconfident admission. Recovery cannot
    /// make anything unsafe: both readers are clamped to their historical constants (the opening
    /// guess to `TDR_BOOTSTRAP_STEPS`, the chunk floor to 256 iterations).
    pub(crate) mode_rate: [[f64; MODE_RATE_SLOTS]; 2],
    /// When each `mode_rate` slot last took a reading, seconds of process time.
    pub(crate) mode_rate_t: [[f64; MODE_RATE_SLOTS]; 2],
    /// SMOOTHNESS-pacing rate per mode: nominal steps per ms as currently measured (an
    /// asymmetric EMA — a slower reading is adopted at once, a faster one is blended in), as
    /// opposed to `mode_rate`'s all-time pessimistic minimum. `mode_rate` sizes SAFETY floors and
    /// opening guesses, where the worst case is the only honest number; this sizes the motion
    /// and pinned-refresh chunk passes to `MOTION_PASS_MS` (`render::motion_pass_steps`), where
    /// the CURRENT regime is the honest number and the TDR budget remains the ceiling. GPU
    /// timestamps only, like `mode_rate` (see `apply_iterate_measurement`).
    pub(crate) motion_rate: [[f64; MODE_RATE_SLOTS]; 2],
}

impl Meter {
    /// The WORST rate (steps per ms) measured for this view's current mode, if any has been.
    /// `mode_rate` is kept as a running MINIMUM, so this is already the pessimistic extreme rather
    /// than an average — which is what a safety floor has to be sized from.
    pub(crate) fn worst_rate_steps_per_ms(&self, v: usize, mode: u32) -> Option<f64> {
        let r = slot(mode).map_or(0.0, |s| self.mode_rate[v][s]);
        (r.is_finite() && r > 0.0).then_some(r)
    }

    /// Fold one priced dispatch into this view's per-mode rate, keeping the PESSIMISTIC extreme,
    /// which recovers slowly ([`mode_rate_step`]). See the `mode_rate` field comment for why this
    /// is a min and not an average. `now_s` is process time in seconds.
    pub(crate) fn record_mode_rate(
        &mut self,
        v: usize,
        mode: u32,
        straddles_switch: bool,
        ms: f64,
        steps: u64,
        now_s: f64,
    ) {
        let (Some(s), true) = (slot(mode), ms > 0.0 && steps > 0) else {
            return;
        };
        // ⚠A measurement is DEFERRED — `wall_probe_step` holds a reading for two frames, and the
        // timestamp readback lags similarly — so `budget_mode` at record time is not necessarily
        // the mode the priced dispatch ran in. Straight after a crossover that misattributes the
        // OLD mode's rate to the new one, and the old mode is the fast one (~152× at 0→2 in the
        // field report), which is precisely the optimistic direction this whole mechanism exists to
        // avoid. The ceiling clamp in `bootstrap_steps` means such a poisoned rate can never be
        // worse than the historical constant, but it would silently defeat the fix in the one
        // window that motivated it. Skipping the frames that straddle the switch is enough: the
        // dispatch behind a reading taken 3+ frames later is unambiguously in the current mode.
        if straddles_switch {
            return;
        }
        let rate = steps as f64 / ms;
        if !rate.is_finite() || rate <= 0.0 {
            return;
        }
        let elapsed = (now_s - self.mode_rate_t[v][s]).max(0.0);
        self.mode_rate[v][s] = mode_rate_step(self.mode_rate[v][s], rate, elapsed);
        self.mode_rate_t[v][s] = now_s;
    }

    /// The smoothness-pacing rate for this view's CURRENT mode (nominal steps per ms), or 0.0
    /// when nothing has been measured in it yet — the caller then falls back to the opening
    /// guess (`bootstrap_steps`), never to the full budget. See `Perf::motion_rate`.
    pub(crate) fn motion_rate_now(&self, v: usize, mode: u32) -> f64 {
        let slot = slot(mode);
        let this = slot.map_or(0.0, |s| self.motion_rate[v][s]);
        if this.is_finite() && this > 0.0 {
            return this;
        }
        // Nothing measured in this mode yet — the frames right after a crossover, where the
        // first refreshes are sized. The most pessimistic rate seen in any OTHER mode, under
        // the same margin the safety controller applies to the same gap (`bootstrap_steps`),
        // beats a constant: measured at the 1e4 hand-over, the constant opening guess floored at
        // `TDR_MIN_STEPS` sized 7-iteration passes and the pin never completed at 4.0×.
        let other = self.motion_rate[v]
            .iter()
            .enumerate()
            .filter(|(s, _)| slot != Some(*s))
            .map(|(_, r)| *r)
            .filter(|r| r.is_finite() && *r > 0.0)
            .fold(0.0_f64, |acc, r| if acc == 0.0 { r } else { acc.min(r) });
        if other > 0.0 {
            other / crate::tunables::cost().mode_rate_unknown_margin.max(1.0)
        } else {
            0.0
        }
    }

    /// Fold one priced dispatch into this view's per-mode SMOOTHNESS rate. Asymmetric on
    /// purpose: a reading slower than the estimate replaces it (the next pass must not be sized
    /// off a regime that has just got more expensive — entering a minibrot interior, a BLA tree
    /// that stopped skipping), a faster one is blended in at `MOTION_RATE_GROW` so one lucky
    /// early-escape pass cannot size the next one at a hundred times the work. The same
    /// mode-switch straddle rule as `record_mode_rate`.
    pub(crate) fn record_motion_rate(&mut self, v: usize, mode: u32, straddles_switch: bool, ms: f64, steps: u64) {
        const MOTION_RATE_GROW: f64 = 0.3;
        let (Some(s), true) = (slot(mode), ms > 0.0 && steps > 0) else {
            return;
        };
        if straddles_switch {
            return;
        }
        // The rate of the VARIABLE part of the pass: with `PASS_FIXED_MS` set, a small pass's fixed
        // cost no longer reads as dear steps (`motion_pass_steps` spends the same split). 0 = off.
        let rate = steps as f64 / super::variable_pass_ms(ms, crate::calibration::pass_fixed_ms());
        if !rate.is_finite() || rate <= 0.0 {
            return;
        }
        let cur = self.motion_rate[v][s];
        // A reading pulls the estimate DOWN at once and pushes it UP at `MOTION_RATE_GROW`.
        // ⛔NO GATE ON THE WAY UP — two were tried and both deadlocked the same way: "only a
        // pass ≥ ¼ of the current sizing may grow it" and "only a reading ≥ 1 ms may grow it".
        // A pessimistic cut sizes the next passes below any such gate, so no pass can ever
        // qualify again, the pin crawls at its floor and the frozen frame ages 8–13 octaves on
        // screen. The estimate is a HINT: the pass growth limiter (×2 per pass), the band
        // licence, the wall-clock cut and the TDR budget are the guards, and a deduped 0.2 ms
        // reading that inflates the hint is caught by those on the next pass.
        // ⭐The way DOWN is bounded to `MOTION_CUT_MAX` per reading (see that constant): a PIN
        // pass's timestamp — walked past the BLA-skippable range at a per-step cost nothing like
        // a motion frame's — landed here and cut the motion budget 9× in one reading at 2^800,
        // and the next motion frame painted one flat colour. Three such readings still find a
        // 9× slower regime; one reading no longer does, because one reading from a different
        // kind of pass is not a measurement of this one.
        self.motion_rate[v][s] = match bounded_motion_cut(cur, rate) {
            Some(cut) => cut,
            None if cur <= 0.0 => rate,
            None => cur + (rate - cur) * MOTION_RATE_GROW,
        };
    }

    /// The rate half of `Perf::motion_wall_cut`: `steps_last` is the view's last dispatch, `dt_ms` the
    /// interval that followed it, `moving` whether that dispatch was made while the view moved.
    /// Returns whether the interval was long enough to count against the pass, in which case the
    /// caller halves its motion step too.
    pub(crate) fn motion_wall_cut(&mut self, v: usize, mode: u32, steps_last: u64, dt_ms: f64, moving: bool) -> bool {
        let Some(s) = slot(mode) else {
            return false;
        };
        let cur = self.motion_rate[v][s];
        // THE LIFT: a MOTION dispatch followed by a frame that fit its present interval proves the
        // rate was at least steps / interval. It is the deadlock-proof half of this estimator —
        // however low a cut has pushed the hint, the very next pass that fits a frame restores
        // what it just did, with no reading threshold to fail — bounded to `MOTION_LIFT_MAX` per
        // event as the cut is.
        // ⚠**Only a dispatch made while moving (U40).** Any dispatch of the last two frames used to
        // count, and a STILL one says nothing about the motion rate: submission is asynchronous, so
        // the frame after a big settle tile is quick while the GPU still runs it. On the RTX 3080
        // budget trap a 1.45e10-step tile and a 3 ms frame lifted the rate 350× (1.4e7 → 4.9e9) in
        // one frame; on the RX 6800 XT tiles and settled chunk passes lifted it 22–58×. Proving the
        // lift by a completion instead was tried and cost ~10% on-screen lag in a healthy glide (a
        // completion is noticed up to a frame late, which understates a small pass's rate).
        if dt_ms <= crate::tunables::MOTION_PASS_MS * 2.0 {
            if moving {
                if let Some(rate) = bounded_motion_lift(cur, steps_last as f64 / dt_ms) {
                    self.motion_rate[v][s] = rate;
                }
            }
            return false;
        }
        let cut_at = crate::tunables::MOTION_PASS_MS * 2.5;
        if !(dt_ms > cut_at) {
            return false;
        }
        // Only a dispatch of at least half the current sizing can be blamed for the interval:
        // a long frame behind a tiny pass was something else (a reference build on this thread,
        // a present blocked by an unrelated submission), and pricing the tiny pass at that
        // interval is the cut that never recovers.
        if cur > 0.0
            && (steps_last as f64) < 0.5 * cur * crate::tunables::MOTION_PASS_MS
        {
            return false;
        }
        let wall_rate = steps_last as f64 / dt_ms;
        if let Some(rate) = bounded_motion_cut(cur, wall_rate) {
            self.motion_rate[v][s] = rate;
        }
        true
    }

    /// The opening guess for this view's CURRENT mode, derived from what has actually been measured
    /// on this device. `rate_other` is the most pessimistic rate seen in any other mode, which is
    /// all we have to go on the first time a dive crosses into a mode.
    pub(crate) fn bootstrap_steps(&self, v: usize, mode: u32) -> u64 {
        let this = slot(mode).map_or(0.0, |s| self.mode_rate[v][s]);
        let other = self.mode_rate[v]
            .iter()
            .enumerate()
            .filter(|(s, _)| slot(mode) != Some(*s))
            .map(|(_, r)| *r)
            .filter(|r| *r > 0.0)
            .fold(0.0_f64, |acc, r| if acc == 0.0 { r } else { acc.min(r) });
        bootstrap_steps(this, other)
    }
}

/// Nominal steps of one settle-grid tile: its pixels × ss² × the iteration count.
pub(crate) fn tile_steps(rect: [u32; 4], ss: u32, gpu_iter: u32) -> u64 {
    (rect[2] as u64)
        .saturating_mul(rect[3] as u64)
        .saturating_mul((ss as u64).saturating_mul(ss as u64))
        .saturating_mul(gpu_iter.max(1) as u64)
}

/// What a KEY-CHANGED, un-chunked dispatch ran, for `fe_steps_last`: the settle tile's rect when the
/// frame dispatches one, the whole frame otherwise.
///
/// ⚠The first tile of every settle grid arrives on a key change (the ss stage moves), and the
/// key-change stamp used to price it as the WHOLE frame, overwriting the rect the tile block had
/// just paired it with. RX 6800 XT budget trap, 2026-10-12: 16.97e9 against an 8.06e9 tile (2.1×),
/// so B4's wall cap learned 1.534e10 from the first slow grid where the tile said ~7.3e9, too loose
/// to bind the second grid. The wall-clock fallback prices the same slot, in the direction of growth.
/// A zero-area hold rect keeps the whole-frame stamp it always had.
pub(crate) fn key_change_steps(spx: u64, ss: u32, gpu_iter: u32, tile: Option<[u32; 4]>) -> u64 {
    match tile {
        Some(r) if r[2] > 0 && r[3] > 0 => tile_steps(r, ss, gpu_iter),
        _ => spx
            .saturating_mul((ss as u64).saturating_mul(ss as u64))
            .saturating_mul(gpu_iter.max(1) as u64),
    }
}

/// The next piece of a settle tile that fits the admission: the full tile `full` (`[x, y, w, h]`,
/// base pixels) from row `row` on, as many whole rows as `admitted` affords at `per_row` steps a
/// row, at least one. A tile the admission covers comes back whole.
///
/// ⭐A grid freezes its geometry on its first tile, so an admission that drops mid-grid (a wall cap
/// set by one of its own tiles running past `TDR_LATENCY_ACCEPT_MS`) used to leave every remaining
/// tile at the old size: on the RTX 3080, three tiles of 1.1e10–1.45e10 steps after a cap of 8.9e9.
/// Splitting the tile into rows keeps the grid's order and everything already composed.
pub(crate) fn settle_strip(full: [u32; 4], row: u32, per_row: u64, admitted: u64) -> [u32; 4] {
    let row = row.min(full[3].saturating_sub(1));
    let fit = ((admitted / per_row.max(1)).min(u32::MAX as u64) as u32).max(1);
    [full[0], full[1] + row, full[2], fit.min(full[3] - row)]
}

/// The latest completed registration of a view's work (`Perf::view_done`): its sequence, its time
/// from dispatch to the completion callback, and the steps it carried.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ViewDone {
    pub seq: u64,
    pub price_us: u64,
    pub steps: u64,
}

/// SETTLE-TILE SERIALIZATION: does this view's next settle tile (`next_steps`) wait? The input is
/// what the GPU has CONFIRMED by completion callback (`Perf::view_reg_pending`): `outstanding`
/// registrations of this view's work not yet complete, and `last`, the latest one that is.
///
/// - Nothing outstanding: go.
/// - `SETTLE_INFLIGHT_MAX` (3) or more outstanding: wait.
/// - Fewer: go only if the last completed work was at least as big as this tile
///   (`PRICE_REPRESENTATIVE_FRAC`) and took under `SETTLE_PIPELINE_MS` from dispatch to
///   completion. Cheap tiles then keep close to their one-per-frame cadence, and a tile whose
///   predecessor ran slow, or whose only evidence is a small pass, waits for the queue to empty.
///   Measured on the RTX 3080 (stripe at 1920x1200, where settles tile): a cheap settle reached
///   full resolution in 0.68 s unpaced, 1.39 s with one in flight, 0.91 s with this rule at two
///   outstanding and 0.80 s at three; the heavy ss 2 grid after it took 4% longer, with a worst
///   frame of 326 ms against 365 ms.
/// - Whatever the count, the gate opens once the newest registration is older than
///   `SETTLE_COMPLETION_TIMEOUT_US`: a completion that late is a lost device or a callback that is
///   not coming, and the grid must not wedge on it.
///
/// ⚠Not the frame interval. The interval after a tile is no proof it finished: on the RX 6800 XT an
/// 8.06e9-step tile was followed by a 7.4 ms interval, the next three tiles went out behind it, and
/// the stall surfaced two and three frames later (357, 713, 495 ms; budget trap, 2026-10-12).
pub(crate) fn settle_tile_waits(outstanding: u64, last: ViewDone, next_steps: u64, reg_age_us: u64) -> bool {
    use crate::tunables::{SETTLE_COMPLETION_TIMEOUT_US, SETTLE_INFLIGHT_MAX, SETTLE_PIPELINE_MS};
    if crate::tunables::cost().settle_serialize != 1 || outstanding == 0 || reg_age_us > SETTLE_COMPLETION_TIMEOUT_US {
        return false;
    }
    if outstanding >= SETTLE_INFLIGHT_MAX {
        return true;
    }
    let fast = (last.price_us as f64) / 1000.0 <= SETTLE_PIPELINE_MS;
    let representative = last.steps > 0 && probe_would_price(last.steps, next_steps);
    !(fast && representative)
}

/// Frames a view's newest dispatch may go without a reading feeding its learned budget before that
/// budget stops vouching for the next dispatch ([`budget_confident`]). A GPU reading lands two to
/// three frames after its pass, but a PINNED chunk walk in motion is priced about one frame in ten
/// (RTX 3080 zoomtest, 2026-10-12: 146 of 149 frames an 8-frame window flagged were those walks,
/// every one of them fast). Sixteen clears that cadence and still flags a real starvation — the same
/// run had one of 118 frames, the one that tripped the wall-clock fallback.
pub(crate) const CONFIDENT_FRAMES: u64 = 16;

/// `budget_confident` (design/live-render-robustness.md §5.1, B4): is this view's learned budget
/// fed by readings of the work it is sending? False once a real dispatch (`dispatch_frame`) has gone
/// [`CONFIDENT_FRAMES`] without any reading reaching the learned budget (`fed_frame`: a GPU reading,
/// or the wall clock while it stands in for one). An idle view is never unconfident: nothing is
/// outstanding. An unfed budget may be stale in either direction, and the opening guess is the
/// only size chosen to be safe without a measurement, so [`admit`] bounds such a frame by it.
pub(crate) fn budget_confident(dispatch_frame: u64, fed_frame: u64, now: u64) -> bool {
    !measurement_starved(dispatch_frame, fed_frame, now, CONFIDENT_FRAMES)
}

/// Which term bounds an admission ([`admit`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum AdmitBound {
    /// The learned size, already capped by the fixed ceiling, the explicit bound and the motion jam.
    #[default]
    Learned,
    /// The wall-clock cap.
    Wall,
    /// The opening guess, because the learned budget is unconfident.
    Unconfident,
}

impl AdmitBound {
    /// The frame record's code: 0 learned, 1 wall clock, 2 unconfident.
    pub(crate) fn code(self) -> u8 {
        match self {
            AdmitBound::Learned => 0,
            AdmitBound::Wall => 1,
            AdmitBound::Unconfident => 2,
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            AdmitBound::Learned => "learned",
            AdmitBound::Wall => "wall clock",
            AdmitBound::Unconfident => "unconfident (opening guess)",
        }
    }
}

/// `admit()` (design §5.1, B4) as a pure function: the steps one dispatch may carry. `learned` is the
/// size the chain in `build_params` arrived at (learned budget, fixed ceiling, explicit bound, motion
/// jam); it is further bounded by the wall-clock cap (`wall_cap`, 0 = none) and, while the learned
/// budget is unconfident, by `bootstrap`. Never raises `learned`.
///
/// ⚠SHADOW in this build: `build_params` computes and records it every frame and sizes nothing from
/// it. Where it would bind, and whether that is where it should, is the evidence the reaction waits on.
pub(crate) fn admit(learned: u64, wall_cap: u64, bootstrap: u64, confident: bool) -> (u64, AdmitBound) {
    let mut out = (learned, AdmitBound::Learned);
    if wall_cap != 0 && wall_cap < out.0 {
        out = (wall_cap, AdmitBound::Wall);
    }
    if !confident && bootstrap < out.0 {
        out = (bootstrap, AdmitBound::Unconfident);
    }
    out
}

/// One reading of the WALL-CLOCK CAP (design §5.1's `wall_budget`, made one-sided by the evidence):
/// `steps` dispatched and `ms` from its dispatch to the GPU's completion callback (`Perf::view_done`,
/// which includes any work queued ahead of it, so it errs long). 0 = no cap.
///
/// It only ever says "too high". It does not exist until a dispatch is SLOW past
/// `TDR_LATENCY_ACCEPT_MS` (600 ms): the dispatch is then shrunk by [`budget_step`]'s arithmetic
/// toward the 400 ms target (its own size times target/ms, the raw ratio past the lethal band), and
/// that is the cap. Between the target and 600 ms it neither sets nor tightens: a settled pass is
/// SIZED to land near 400 ms and its completion reads a frame or two late, so a trigger at the
/// target would make the cap a second controller of every walk. A fast reading relaxes an existing
/// cap through the same arithmetic, and a cap relaxed to the ceiling is dropped.
///
/// ⚠**Why completion, not the frame interval.** It was priced by the longest of the three frame
/// intervals after a dispatch (`wall_probe_step`). Serialized settle tiles spread a tile's cost over
/// several frames, and on the RX 6800 XT an 8.06e9-step tile of about half a second showed as 234,
/// 386 and 345 ms intervals, none of them slow, so the cap never set on the first grid
/// (2026-10-12, field build dev.2).
///
/// ⭐**Why one-sided.** Specified as a second learned budget, the wall term trails the climb: it
/// learns from whatever was dispatched, and while the view moves that is small motion passes. On
/// the RTX 3080 (2026-10-12, shadow) it bound 86% of a zoomtest's frames at a median 0.13× the
/// planned size and 99.6% of a still soak at 0.67×, every dispatch it would have shrunk taking under
/// 5 ms — it would have choked every settle after motion. What the wall clock knows that the GPU
/// timestamps may not is that a frame ran SLOW; that is the only thing it is allowed to say here.
pub(crate) fn wall_cap_step(cap: u64, steps: u64, ms: f64, explicit: bool) -> u64 {
    if steps == 0 || !ms.is_finite() || ms <= 0.0 {
        return cap;
    }
    // The target is `budget_step`'s own (both regimes aim at 400 ms); this decides only WHEN to act.
    let ceil = if explicit {
        crate::tunables::cost().explicit_steps_ceil
    } else {
        crate::tunables::cost().tdr_steps_ceil
    };
    if ms > crate::tunables::cost().tdr_latency_accept_ms {
        // The dispatch that ran slow, shrunk: `budget_step` from its own size (`min(cap, steps)`).
        let base = if cap == 0 { steps } else { cap.min(steps) };
        return budget_step(base, steps, ms, explicit).map_or(base, |(next, _)| next);
    }
    if cap == 0 {
        return 0;
    }
    match budget_step(cap, steps, ms, explicit) {
        Some((next, _)) if next >= ceil => 0,
        Some((next, _)) => next.max(cap),
        None => cap,
    }
}

#[cfg(test)]
mod admission_tests;
#[cfg(test)]
mod settle_tile_tests;
#[cfg(test)]
mod settle_strip_tests;
#[cfg(test)]
mod mode_rate_tests;
#[cfg(test)]
mod motion_lift_tests;
#[cfg(test)]
mod pin_pricing_tests;
#[cfg(test)]
mod pin_gate_timeout_tests;
#[cfg(test)]
mod retreat_tests;
