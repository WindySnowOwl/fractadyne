//! `decide(inputs) -> plan`: the live frame's decisions as pure functions over a RECORDED struct
//! (design/live-render-robustness.md §5.4, W5).
//!
//! Every controller in the frame loop was already pure in its parts, but nothing could feed a field
//! session through them: a field log became a test only by a human transcribing numbers (U32). Here
//! a stage of `build_params` is lifted into a function of one inputs struct — every value the stage
//! reads, including the per-view state it carries between frames and the tunables it consults — that
//! returns one decision struct, including the state it writes. Both structs serialize, so a session
//! can record them per frame (`FRACTADYNE_RECORD_DECIDE`), `--replay` can recompute every decision
//! from the recording and diff it field by field, and a recording committed under
//! `validation/replays/` is a regression test that runs on the CPU alone.
//!
//! ⚠The boundary (§5.4): only decisions lift. The reference lifecycle (threads, installs), the GPU
//! sinks and the readback machines stay imperative and enter as INPUTS. A stage that cannot be
//! stated as inputs → decision is not lifted.

use serde::{Deserialize, Serialize};

/// What [`decide_moving`] reads: `bp_moving_scale`'s inputs, one view, one frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct MovingInputs {
    /// The frame's pixel count and iteration ask, and the work budget the shallow bound divides.
    pub(crate) px: u64,
    pub(crate) gpu_iter: u32,
    pub(crate) budget: u64,
    pub(crate) interacting: bool,
    pub(crate) pin_frame: bool,
    pub(crate) view: u8,
    pub(crate) is_pert: bool,
    /// The panel's pixel count (`Resolution::px` of the frame's resolution at this stage).
    pub(crate) panel_px: u64,
    /// "Prefer detail while zooming" or the auto-zoom's Quality priority.
    pub(crate) prefer_detail: bool,
    pub(crate) tour_playing: bool,
    /// The chunk path can serve this frame (mode, formula, colouring, device), so a perturbation
    /// refresh is spread over a pin's passes.
    pub(crate) can_chunk: bool,
    pub(crate) zoom_oct_s: f64,
    pub(crate) frame_ms: f64,
    pub(crate) last_dt_ms: f64,
    /// The interval just ended followed a real re-iterate frame of this view.
    pub(crate) prev_real: bool,
    /// The smoothness rate a refresh is priced at (`Perf::motion_rate_now`).
    pub(crate) motion_rate: f64,
    pub(crate) min_motion_res: f64,
    /// Carried per view: the last pin's frames and resolution, the AIMD scale and whether a real
    /// frame has stepped it, the visible-frame ladder's rung and the pass budget it was sized at.
    pub(crate) pin_frames_last: u64,
    pub(crate) pin_res_last: f64,
    pub(crate) motion_res: f64,
    pub(crate) motion_res_measured: bool,
    pub(crate) visible_res: f64,
    pub(crate) motion_pass_steps_last: u64,
    /// The escape tops a moving frame may be sized to reach: the palette window's (0 = none), the
    /// last complete walk's (0 = none), and that walk's histogram.
    pub(crate) norm_hi: f64,
    pub(crate) esc_top: f64,
    pub(crate) esc_hist: Option<Vec<u32>>,
    /// The overridable tunables this stage reads (`tunables::cost()`), so a replay of a session run
    /// under `--set` is exact.
    pub(crate) tdr_bootstrap_steps: u64,
    pub(crate) motion_need_source: u64,
    pub(crate) motion_need_quantile: f64,
}

/// What [`decide_moving`] decides, and the state it writes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct MovingDecision {
    /// The scale before the visible-frame ladder (a live refresh's) and after it.
    pub(crate) motion_scale: f64,
    pub(crate) res_scale: f64,
    /// The per-view state after this frame.
    pub(crate) motion_res: f64,
    pub(crate) motion_res_measured: bool,
    pub(crate) visible_res: f64,
    /// `[need, ladder target, pre-ladder scale]` for the frame record (all 0 when not sized moving).
    pub(crate) vis_sizing: [f64; 3],
    /// The rate-aware refresh cap, when it was derived: its value, whether a measurement or the
    /// model gave it, whether the refresh is a pinned one, and the passes the zoom rate allows.
    pub(crate) rate_cap: Option<RateCap>,
    /// The AIMD stepped on this frame's interval.
    pub(crate) aimd_stepped: bool,
}

/// [`MovingDecision::rate_cap`]'s detail.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct RateCap {
    pub(crate) cap: f64,
    pub(crate) measured: bool,
    pub(crate) pinned_refresh: bool,
    pub(crate) frames_allowed: u64,
}

/// The MOVING frame's scale (W4's `bp_moving_scale`, lifted): the work budget's bound, the
/// rate-aware refresh cap, the measured AIMD and the visible-frame ladder. Pure: the same inputs
/// give the same decision, which is what makes a recording replayable.
#[must_use]
pub(crate) fn decide_moving(i: &MovingInputs) -> MovingDecision {
    use super::{
        motion_need, motion_pass_steps, motion_res_step, refresh_passes_allowed, refresh_res_cap,
        visible_frame_reaches, visible_res_hold, visible_res_target,
    };
    let (px, gpu_iter, budget) = (i.px, i.gpu_iter, i.budget);
    let (interacting, pin_frame, is_pert) = (i.interacting, i.pin_frame, i.is_pert);
    // GPU-watchdog safety: if even the capped work won't fit the budget, reduce the
    // iteration-texture resolution (the color pass box-filters the upscale).
    let want = px.saturating_mul(gpu_iter.max(1) as u64);
    let budget_res_scale = if want > budget {
        (budget as f64 / want as f64).sqrt()
    } else {
        1.0
    };
    // ADAPTIVE MOTION RESOLUTION (AIMD). `budget_res_scale` sizes the moving frame from
    // `px·gpu_iter` — the *no-BLA-skip* cost. Where the BLA skips (the common deep case) the real
    // GPU cost is a tiny fraction of that, so the budget over-shrinks the moving frame (≈0.47 →
    // 668 px at 1e263×) and the reprojected/held texture goes blocky — the "distorted overlapping
    // tiles" seen when a zoom crosses from detail into the exterior. Instead drive the deep moving
    // res_scale by the MEASURED frame interval: additive-increase while frames stay near vsync (BLA
    // is skipping → headroom to sharpen), multiplicative-decrease when they run long (no skip →
    // back off). Adapts per view, needs no GPU-timestamp readback, and cannot hang — a slow frame
    // immediately shrinks the next, and the TDR cap below is still the hard watchdog floor. Only
    // deep perturbation motion is affected; shallow/settled keep the budget scale. Each view steps
    // its own scale (U56; `Perf::motion_res`).
    // ⭐A SETTLED frame is NOT sized by the work budget any more. `budget_res_scale` above
    // divides the panel by a fixed constant (`WORK_BUDGET` × 6 for settle = 3.6e11 nominal
    // steps) that measures nothing — the very shape this file spends pages warning about, and
    // the actual binder behind "an explicit iteration count is silently overridden". At an
    // explicit 10,000,000 iterations it shrank a 1445×1134 panel to 214×168 BEFORE any of the
    // measured machinery below got a look in, so raising the measured budget, the tile
    // allowance, and the grid-upgrade rule all changed nothing: the frame had already been
    // clamped upstream. Traced as `fresh=214x168 allowed=5.536e11` — the measured bound was
    // three times looser than the constant that won.
    //
    // A settled frame now carries its full resolution into the measured path, where
    // `tdr_allowed` (measured, per dispatch) shrinks it if it must and the tiled settle
    // spreads what remains over budget-sized dispatches. MOTION keeps the old bound in both
    // its forms — deep motion on the AIMD `motion_res`, shallow motion on the work budget —
    // because a moving frame has no settle grid to spread cost over.
    //
    // ⚠SCOPED TO `!auto_iter`, i.e. to the iteration count the USER typed, which is the case
    // the bypass exists for. Applying it to the AUTO budget as well regressed the adaptive
    // iteration boost, and the mechanism is worth recording: the boost adapts on the
    // capped-pixel fraction, which the GPU only reports for a FULL-FRAME iterate. A settled
    // auto frame used to be shrunk to fit one dispatch, so every settled frame was a full
    // frame and fed the boost; at full resolution it tiles instead, and the boost then only
    // gets the one coarse arm frame per grid. Measured on `--livetest` over the grand tour:
    // the boost reached ×2.56/×6.40 where it had reached ×6.40/×16/×64, and the 1e63 hold went
    // from 53.9% black to 100%. Resolution and the iteration budget are competing for the same
    // frames here; that trade is its own problem (TODO), and A1 does not require touching it.
    // ⭐A SETTLED view the user is SITTING AT is not sized by the work budget. `budget_res_scale`
    // above divides the panel by a fixed constant (`WORK_BUDGET` × 6 for settle = 3.6e11 nominal
    // steps) that measures nothing — the shape this file spends pages warning about, and the
    // reason a deep view with a high explicit iteration count renders soft. Reported from the
    // field at 6.46e94× with Iterations 4,000,000 and auto-scale off: `want` is ~6.6e12, so the
    // shrink lands at ~0.23 of native linear and the frame is visibly blocky. The measured
    // `tdr_allowed` bound and the tiled settle below are what should size it instead.
    //
    // ⚠GATED ON `!tour_playing()`, and that gate is load-bearing — not a stylistic choice.
    // Applying this during scripted playback regressed the tour badly: tiled settle and the
    // adaptive iteration boost compete for the same settled frames (the boost only adapts on a
    // capped-pixel fraction the GPU reports for a FULL-FRAME iterate), so a tiling hold feeds
    // the boost one coarse arm frame per grid instead of a reading per frame. Measured on
    // `--livetest`: the boost fell to ×1.60 from ×6.40/×16/×64 and the 1e61 hold went from
    // 42.1% black to 100%. The distinction that matters is TIME: a scripted hold has a few
    // seconds on a clock and must spend them on iterations, while a view the user is parked at
    // has as long as it likes and should spend it on resolution. A FINISHED tour is not
    // "playing", so a parked tour sharpens — which is the reported case.
    // RATE-AWARE REFRESH RESOLUTION (design/live-zoom-smoothing.md P-A/P-D). A moving
    // refresh is `passes` chunk passes of `MOTION_PASS_MS` each, and `passes` comes from the
    // observed zoom speed: the held frame may not magnify past `HELD_MAX_OCT` before the
    // refresh lands (nor take longer than `REFRESH_TARGET_S`). What does not fit in those
    // passes at this ask comes out of the resolution — pixels are the actuator, never the
    // iteration count. Applied as a CAP on both motion branches below (the AIMD scale for
    // deep perturbation motion, the work-budget scale for shallow/direct), so a zoom that is
    // slow enough keeps whatever sharpness those controllers earned. Direct mode has no pin
    // (every frame is one dispatch), so it gets a single pass. "Prefer detail while zooming"
    // is the user's explicit choice of native-resolution refreshes and is left alone.
    let mut rate_cap = None;
    let rate_res_cap = if interacting && !pin_frame && !i.prefer_detail {
        // Only a frame the chunk path can serve is spread over passes (the pin); an aux
        // colouring, a formula past the chunk shaders' scope or a device without the state
        // targets renders each refresh as ONE dispatch, and then one pass is all there is.
        // (`can_chunk`: the caller decides it from the mode, the formula, the colouring and the
        // device's state targets.)
        let can_chunk = i.can_chunk;
        // A PINNED refresh spans many frames, so no single frame interval prices it; an
        // unpinned one IS this frame, and the AIMD below prices it from its own interval.
        let pinned_refresh = is_pert && can_chunk;
        let frames_allowed = refresh_passes_allowed(
            i.zoom_oct_s,
            crate::tunables::HELD_MAX_OCT,
            crate::tunables::REFRESH_TARGET_S,
            i.frame_ms.max(1.0) / 1000.0,
        );
        // ⭐⭐THE MEASURED ANSWER REPLACES THE MODEL — it does not merely bound it. This was
        // `model.min(measured)`, and a pessimistic model is then a permanent CEILING: the
        // pin's resolution is baked from the cap, so `pin_res_last` comes back capped, the
        // measured half can raise it only ×1.25, and the model floors it again on the very
        // next frame. Measured on the 1× → 1e100 dive at 4.0× (build 3247): the AIMD read
        // 1.00 (native fits) at every depth past 1e8 while the dispatch went out at 438 of
        // 1452 px — 9% of the pixels — for the WHOLE dive, at a flat 17.8 ms with the GPU
        // idle. That is the same guess-as-a-clamp shape as the bootstrap floor in
        // `budget_step` and the opening guess in `motion_rate_now`: a model is what you use
        // until you have measured, never a bound on what you measured.
        // What actually happened to the LAST pin: `frames_took` at `res_took` ⇒ the next
        // refresh runs at `res_took · sqrt(frames_allowed / frames_took)` (cost ∝ pixels).
        // Growth is bounded to ×1.25 per pin so an early finish sharpens gently; a pin the
        // view outran shrinks as hard as the ratio says.
        let pin_measured = (pinned_refresh && i.pin_frames_last > 0).then(|| {
            let took = i.pin_frames_last as f64;
            let want = frames_allowed.max(1) as f64;
            let scale = (want / took).sqrt().min(1.25);
            (i.pin_res_last * scale)
                .clamp(i.min_motion_res, 1.0)
        });
        // ⛔⛔**A PINNED refresh falls back to the MODEL, never to the AIMD** — tried, and it
        // put a 50-OCTAVE-stale frame on screen (a uniform smear of a frame magnified 10^15×,
        // seen live at 2^262 on 2026-09-17). The deadlock: the pin measurement can only be
        // produced BY a pin that completes, and a pin completes only if something bounds its
        // size. The model is that bound. Remove it and the first pin at a new depth goes out
        // at native resolution against a 557,000-iteration ask, never finishes inside the
        // drift window, abandons, and the frozen frame it was going to replace ages without
        // limit while the view keeps zooming. The same shape as "a probe whose reading
        // another rule discards never terminates": a measurement that can only be produced by
        // the thing it is meant to size needs a guess to get started, and that is the model's
        // whole job. The AIMD is the fallback for an UNPINNED refresh only, where the frame
        // itself is the refresh and there is nothing to fail to complete.
        let measured = pin_measured.or_else(|| {
            (!pinned_refresh && i.motion_res_measured).then_some(1.0)
        });
        let cap = measured.unwrap_or_else(|| {
            // Nothing measured in this regime yet — the opening guess, so the first refresh
            // after a jump or a mode switch is not a native-resolution TDR-sized dispatch.
            let pass = motion_pass_steps(
                i.motion_rate,
                crate::tunables::MOTION_PASS_MS,
                u64::MAX,
                i.tdr_bootstrap_steps,
            );
            let passes = if pinned_refresh {
                // The frames the rate allows, counted as passes. With one pin pass in flight
                // (`PIN_INFLIGHT_MAX`) a pin dispatches nearer one pass per two frames, but each
                // pass is priced and grows; this sizing is the one the 2026-10-12 A/B measured.
                frames_allowed.max(1)
            } else {
                1
            };
            refresh_res_cap(
                pass,
                passes,
                gpu_iter,
                i.panel_px,
                i.min_motion_res,
            )
        });
        rate_cap = Some(RateCap {
            cap,
            measured: measured.is_some(),
            pinned_refresh,
            frames_allowed: frames_allowed as u64,
        });
        cap
    } else {
        1.0
    };
    let mut motion_res = i.motion_res;
    let mut motion_res_measured = i.motion_res_measured;
    let mut aimd_stepped = false;
    let res_scale = if pin_frame {
        // Pinned refresh: geometry is captured, not derived — the motion scale was baked into
        // the pin's resolution at pin time, and applying it again would double-shrink (the
        // final geometry is forced to the pin's below, after the ss chain).
        1.0
    } else if !interacting && !i.tour_playing {
        1.0
    } else if interacting && i.prefer_detail {
        // "Prefer detail while zooming" (or the autopilot's Quality priority): the periodic
        // refresh frames (REFRESH_OCTAVES cadence,
        // below) render at NATIVE resolution instead of the AIMD-adapted motion resolution —
        // full-detail frames streamed at their real cost, the hold reprojecting between them.
        // The budget shrink stays as the safety net if a native refresh can't fit one dispatch.
        // (Direct mode keeps only that safety net, as it always has — it has no hold to
        // reproject between refreshes, so "native" there means every frame at native.)
        if is_pert { 1.0 } else { budget_res_scale }
    } else if interacting {
        // ⭐THE MEASURED CONTROLLER RUNS FOR EVERY MOVING FRAME, shallow included. It used to
        // be gated on `is_pert`, so a DIRECT-mode zoom was sized by `budget_res_scale` (a
        // constant-derived no-skip bound) capped by the rate model — two unmeasured numbers,
        // no feedback between them. At shallow depth nearly every pixel escapes in a handful
        // of iterations, so that nominal cost overstates the real one by orders of magnitude
        // and the cap sat on the floor while frames ran at 17.8 ms with the GPU idle: the
        // "detail lost at low zoom" report of 2026-09-16. The same AIMD the deep path uses
        // finds the honest resolution here in one step per real frame, and direct mode keeps
        // `budget_res_scale` underneath it as the watchdog bound it always had.
        // ⭐Each view steps its OWN scale, on the interval after its own real frame (U56): the
        // second panel used to move at view 0's scale, which only view 0's frames stepped.
        {
            // Adapt ONLY on the interval following a REAL re-iterate frame. During a dive most
            // frames are ~free reprojections; adapting on those (the old behaviour) pushed the
            // res toward native (+0.03/frame), so every rare real refresh rendered near-native
            // at 33–64 ms — 2–4 dropped vsyncs each, the divetest-measured "stepped zooming".
            // A reprojection interval says nothing about iterate cost → hold. The raw interval
            // (`last_dt_ms`, not the EMA `frame_ms` — the EMA smooths exactly the spikes this
            // must react to) after a real frame IS that frame's cost signal:
            //   > 24 ms → proportional cut toward a ~20 ms interval (cost ∝ res², so scale by
            //             sqrt(target/dt) — one step, not a hunt), floored at `min_motion_res`
            //             (user-configurable sharpness floor, default 0.30).
            //   < 17 ms → the real frame fit a vsync: grow gently toward native.
            let dt = i.last_dt_ms;
            if i.prev_real && dt > 0.0 {
                // Pure, so the thresholds are pinned by test rather than by reading — see
                // `motion_res_step`. `dt` already has the fps_cap sleep discounted.
                motion_res = motion_res_step(motion_res, dt, i.min_motion_res);
                // A real frame has now been priced at a known resolution: from here the
                // opening guess is retired and this loop owns an unpinned refresh's size.
                motion_res_measured = true;
                aimd_stepped = true;
            }
        }
        // Direct mode keeps the work-budget bound it always had underneath the measured
        // scale; deep perturbation does not (the budget prices the no-skip cost and
        // over-shrinks exactly where the BLA skips — the reason this AIMD exists).
        let scale = motion_res.min(rate_res_cap);
        if is_pert { scale } else { scale.min(budget_res_scale) }
    } else {
        budget_res_scale.min(rate_res_cap)
    };
    // ⭐⭐**AND NOTHING ABOVE MAY SIZE A MOVING FRAME SO SMALL IT CANNOT SHOW ANYTHING.** Every
    // branch above trades resolution against frame TIME, and none of them can tell a frame that
    // finished from one that committed no pixel — because the blank one is the FASTER of the
    // two, so they sharpen it. This cap feeds back on iterations WALKED instead, and sits
    // beneath both `min_motion_res` and `prefer_detail` (see `Perf::visible_res`).
    // ⚠It sizes a frame so ONE CHUNK PASS reaches the view's escapes; a live-refresh frame is
    // one pass over the whole ask, which reaches them all, so it keeps the scale from before
    // this ladder (`motion_scale`) — the ladder shrank the first df32 frames to 145×110.
    let motion_scale = res_scale;
    let mut visible_res = i.visible_res;
    let mut vis_sizing = [0.0; 3];
    let res_scale = if interacting && !pin_frame {
        // The TOP of the view's escape range, not its first escape: reaching the first escape
        // commits one pixel, reaching the top commits the picture.
        // With `MOTION_NEED_QUANTILE` below 1, how far MOST of it escapes (`motion_need`).
        // ⚠U24: WHICH top (`MOTION_NEED_SOURCE`). 0, shipped: the palette window's — which a
        // moving frame's own one-pass preview feeds (a preview is "complete": it is what is on
        // screen), so a frame that stops short lowers the top the next frame is sized to reach.
        // A guess reading a value downstream of what it sizes (principle P3). 1: the top of the
        // view's last COMPLETE walk (`Perf::esc_top`, beside the histogram, which was already
        // kept from complete walks only for this reason); none yet = no opinion.
        let top = if i.motion_need_source == 1 { i.esc_top } else { i.norm_hi };
        let need = motion_need(
            top,
            i.esc_hist.as_deref(),
            i.motion_need_quantile,
        );
        let target = visible_res_target(
            i.motion_pass_steps_last,
            i.panel_px,
            need,
        );
        let reaches = visible_frame_reaches(
            i.motion_pass_steps_last,
            i.panel_px,
            visible_res,
            need,
        );
        let held = visible_res_hold(visible_res, target, reaches);
        visible_res = held;
        // Into the frame record: what sized this moving frame, and which of the two caps bound
        // it — `res_scale` (the AIMD / rate / budget caps above) or the ladder's `held`.
        vis_sizing = [need, target, res_scale];
        res_scale.min(held)
    } else {
        res_scale
    };
    MovingDecision {
        motion_scale,
        res_scale,
        motion_res,
        motion_res_measured,
        visible_res,
        vis_sizing,
        rate_cap,
        aimd_stepped,
    }
}

/// What [`decide_present`] reads: `bp_present_gate`'s inputs, one view, one frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct PresentInputs {
    /// The arithmetic mode renders directly (no reference, nothing to hold or pin).
    pub(crate) direct: bool,
    /// The iteration axis owns this compose, and the range this frame dispatches.
    pub(crate) chunk_over: bool,
    pub(crate) chunk_range: Option<[u32; 2]>,
    /// The climb probe re-measures this frame; a settle tile is dispatched this frame.
    pub(crate) probe_fired: bool,
    pub(crate) tile_work: bool,
    /// An armed settle grid (its coarse frame about to render) or one with tiles left.
    pub(crate) grid_composing: bool,
    pub(crate) res_scale: f64,
    pub(crate) interacting: bool,
    pub(crate) offscreen: bool,
    pub(crate) will_reproject: bool,
    pub(crate) key_changed: bool,
    pub(crate) gpu_iter: u32,
    pub(crate) ss: u32,
    pub(crate) spx: u64,
    /// The user's "prefer detail" (the setting itself, not the auto-zoom's effective one).
    pub(crate) prefer_detail: bool,
    pub(crate) tour_playing: bool,
    /// Anti-aliasing the user asked for.
    pub(crate) aa: u32,
    /// Carried per view: the hold snapshot is up; a pin is running; the live texture diverges from
    /// the frozen bookkeeping; the live texture is known to hold a picture; no settled-cost
    /// measurement or probe exists yet.
    pub(crate) hold_active: bool,
    pub(crate) pin_active: bool,
    pub(crate) chunk_dirty: bool,
    pub(crate) live_verified: bool,
    pub(crate) aa_unmeasured: bool,
    /// The frame, which an armed probe is stamped with.
    pub(crate) frame: u64,
}

/// What [`decide_present`] decides, and the state it writes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct PresentDecision {
    /// Take the hold snapshot this frame; serve it instead of the live texture; and whether the hold
    /// is a pin's (or a diverged texture's) rather than a settled compose's.
    pub(crate) hold_copy: bool,
    pub(crate) display_hold: bool,
    pub(crate) pin_gate: bool,
    /// Colour-pass box-filter width when true supersampling wasn't affordable.
    pub(crate) aa_filter: u32,
    /// The view's hold flag after this frame; the snapshot's verified flag if this frame sets it
    /// (`None` = unchanged); the settled-cost probe this frame arms, if any.
    pub(crate) hold_active: bool,
    pub(crate) hold_verified: Option<bool>,
    pub(crate) aa_probe: Option<AaProbe>,
}

/// A settled-cost probe armed this frame: at this ss, on this frame, pricing these nominal steps.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct AaProbe {
    pub(crate) ss: u32,
    pub(crate) frame: u64,
    pub(crate) steps: u64,
}

/// The PRESENT gate (`bp_present_gate`, lifted): whether the display serves the held snapshot while
/// compose work runs underneath, whether this frame takes the snapshot, the settled-cost probe, and
/// the colour-pass filter. Pure.
#[must_use]
pub(crate) fn decide_present(i: &PresentInputs) -> PresentDecision {
    let (chunk_over, chunk_range, probe_fired, tile_work) = (i.chunk_over, i.chunk_range, i.probe_fired, i.tile_work);
    let (res_scale, interacting, offscreen, will_reproject) = (i.res_scale, i.interacting, i.offscreen, i.will_reproject);
    let (key_changed, gpu_iter, ss, spx) = (i.key_changed, i.gpu_iter, i.ss, i.spx);
    // ---- present-gating ("prefer detail", stage B) ----
    // While ANY compose work runs on a settled prefer-detail view — a settle tile, an armed
    // grid's coarse full frame, a chunk range, or a climb-probe re-measure — the display
    // serves a snapshot of the last complete frame (`hold_copy` takes it on the FIRST such
    // frame, before the compose pass lands) and the composite builds invisibly underneath.
    // When nothing composes, the flag drops and the finished frame reveals whole. Interaction
    // breaks the gate (Stage A's motion reprojection takes over); direct mode is excluded
    // (cheap and sharp live). Between ss-ramp stages the gate re-engages per stage, so each
    // REVEALED image is complete at its quality level — a stage-wise progressive reveal.
    // Settled CHUNKED walks hold-then-reveal BY DEFAULT (`|| chunk_over`), prefer-detail or
    // not: a walk's progressive display shows nothing until the cursor crosses the view's
    // minimum dwell — which at a deep view sits near the orbit length, so the screen is
    // solid interior color for most of the walk, indistinguishable from a hang (field
    // report: black at "refining 19.8%"). The gate serves the previous complete content
    // under its captured transform and reveals the finished frame whole; a fresh view with
    // nothing rendered falls through gracefully (the GPU gate needs `view.rendered`) to the
    // honest progressive-plus-spinner. Non-chunked tiled settles keep their progressive
    // default — each landed tile is complete content, which reads as sharpening, not as a
    // hang.
    let gate_capable = (i.prefer_detail || chunk_over)
        && !offscreen
        && !interacting
        && !i.direct;
    // ⚠`tile_work`, NOT `tile.is_some()`. A COMPLETED grid goes on emitting its final rect
    // every frame by design, so `tile.is_some()` is true forever after the first grid finishes
    // — the gate never dropped, the display kept serving the snapshot it took when the grid
    // ARMED (a coarse, budget-shrunk frame), and the sharp composite sat in the texture unseen.
    // That is the 2026-08-15 field report in full: "it does the computation but doesn't update
    // the image; it shows up as soon as I resize slightly smaller" — a resize interacts, and
    // interaction breaks the gate, revealing the texture that was finished all along. The
    // chunked path's own term already excludes its completed tail (`e > s`, and the tail emits
    // `[iter, iter]`); this is the same care, which the tile path was missing.
    // Pinned by `--selftest live-res` → "a completed tiled settle REVEALS".
    let composing = tile_work
        || chunk_range.is_some_and(|[s, e]| e > s)
        || probe_fired
        // An armed grid (the coarse arm frame is about to render) or one with tiles left
        // (`PresentInputs::grid_composing`, read from the grid by the caller).
        || i.grid_composing;
    let mut pin_gate = false;
    let mut hold_active = i.hold_active;
    let mut hold_verified = None;
    let (hold_copy, display_hold) = if gate_capable && composing {
        let fresh = !hold_active;
        hold_active = true;
        (fresh, true)
    } else if interacting
        && (i.pin_active
            // Not in DIRECT mode: nothing there can ever clear it (no pin adopts, the latch is
            // perturbation-only), so a stale flag would hold every moving frame forever. The
            // mode switch clears it; this keeps any other path from reaching that state.
            || (i.chunk_dirty && !i.direct))
    {
        // Pin CONTINUE frames, and any interacting frame while the live texture diverges
        // from the frozen bookkeeping (an abandoned pin's residue — holds, freezes, pan
        // reprojections): serve the hold snapshot, never the diverged texture. `fresh` fires
        // only on the FIRST such frame — an abandoned pin's re-pin must NOT retake the
        // snapshot over the dirty texture, and `hold_active` staying up is what prevents it.
        // (A pin START cannot be seen here — it is decided at the commit point after the
        // freeze verdict, which flips these flags itself.)
        pin_gate = true;
        // …and a snapshot may only be taken of a texture KNOWN to hold a picture (see
        // `ContentTrack`); otherwise the existing snapshot stays, or — with none — the
        // display shows live, the honest cold-start fallback.
        let fresh = !hold_active && i.live_verified;
        if fresh {
            hold_verified = Some(true);
        }
        hold_active = true;
        (fresh, true)
    } else {
        hold_active = false;
        // The GPU drops the snapshot on a frame without the flag.
        hold_verified = Some(false);
        (false, false)
    };
    let bootstrap = i.aa_unmeasured;
    let mut aa_probe = None;
    // Armed on EVERY mode. While this was floatexp-only, a df32 view's budget never left the
    // 4e8 bootstrap, so both bounds above were computed from a number that had measured
    // nothing — and the shrink they gate was itself floatexp-only, so nothing bound the frame.
    if !interacting
        && !will_reproject
        && !i.tour_playing // tour frames move every keyframe — not settled-cost data
        && (key_changed || bootstrap)
    {
        // Carry this frame's nominal step count: both arming conditions re-iterate (a changed key
        // re-runs the iterate; a bootstrap frame is the first settled one after the view moved), so
        // the interval that resolves the probe prices real GPU work and yields `fe_rate_spm`.
        let steps = spx
            .saturating_mul((ss as u64).saturating_mul(ss as u64))
            .saturating_mul(gpu_iter.max(1) as u64);
        aa_probe = Some(AaProbe { ss, frame: i.frame, steps });
    }
    // Color-pass anti-aliasing when true supersampling wasn't affordable: widen the box
    // to match an upscaled (resolution-reduced) texture, or apply a gentle 2× box when
    // the budget forced ss=1 on a settled view the user wanted anti-aliased.
    let aa_filter = if res_scale < 1.0 && !will_reproject {
        ((1.0 / res_scale).round() as u32).clamp(2, 4)
    } else if ss == 1 && i.aa > 1 && !interacting {
        2
    } else {
        1
    };
    PresentDecision { hold_copy, display_hold, pin_gate, aa_filter, hold_active, hold_verified, aa_probe }
}

/// The recording's format version: bumped whenever an inputs or decision struct changes shape, and a
/// replay refuses a recording of another (it would compare the wrong things).
pub(crate) const DECIDE_SCHEMA: u32 = 1;

/// `FRACTADYNE_RECORD_DECIDE=<file>`: every lifted decision of every built frame, one JSON line each
/// (`{"kind":"decision","frame","view","stage","inputs","decision"}`) after a header line naming the
/// build and the tunables. Off by default — an opt-in instrument for making replay fixtures and for
/// reading what a session decided. Line-buffered, so a run that ends through `crate::exit` keeps
/// every line it wrote.
static RECORDER: std::sync::OnceLock<Option<std::sync::Mutex<std::io::LineWriter<std::fs::File>>>> =
    std::sync::OnceLock::new();

fn recorder() -> Option<&'static std::sync::Mutex<std::io::LineWriter<std::fs::File>>> {
    RECORDER
        .get_or_init(|| {
            use std::io::Write;
            let path = std::env::var_os("FRACTADYNE_RECORD_DECIDE")?;
            let file = std::fs::File::create(&path).ok()?;
            let mut w = std::io::LineWriter::new(file);
            let header = serde_json::json!({
                "kind": "header",
                "schema": DECIDE_SCHEMA,
                "version": crate::sysinfo::version_string(),
                "tunables": crate::tunables::status_line(),
            });
            writeln!(w, "{header}").ok()?;
            crate::diag::log_line(
                "decide",
                &format!("recording lifted decisions to {} (FRACTADYNE_RECORD_DECIDE)", path.to_string_lossy()),
            );
            Some(std::sync::Mutex::new(w))
        })
        .as_ref()
}

impl crate::FractadyneApp {
    /// Record one lifted decision of this frame, if `FRACTADYNE_RECORD_DECIDE` asked for it.
    pub(super) fn record_decision<I: Serialize, D: Serialize>(&self, stage: &str, view: usize, inputs: &I, decision: &D) {
        use std::io::Write;
        let Some(r) = recorder() else { return };
        let line = serde_json::json!({
            "kind": "decision",
            "frame": self.perf.frame_idx,
            "view": view,
            "stage": stage,
            "inputs": inputs,
            "decision": decision,
        });
        if let Ok(mut w) = r.lock() {
            let _ = writeln!(w, "{line}");
        }
    }
}

/// What a replay found.
#[derive(Debug, Default)]
pub(crate) struct ReplayReport {
    /// The recording's header line, if it had one.
    pub(crate) header: Option<serde_json::Value>,
    /// Decisions recomputed, and how many of them were of a MOVING frame (the ones with anything to
    /// decide: a settled frame's moving scale is 1 by construction).
    pub(crate) decisions: u64,
    pub(crate) moving: u64,
    /// Decisions whose recomputation differed from the record, and the first few, field by field.
    pub(crate) mismatches: u64,
    pub(crate) first: Vec<String>,
    /// Lines that could not be read back (an unknown stage, a value JSON cannot carry, a torn line).
    pub(crate) unreadable: u64,
    /// The recording is of another schema: nothing was compared.
    pub(crate) wrong_schema: Option<u64>,
}

impl ReplayReport {
    /// Every decision recomputed exactly, and enough of them to mean something.
    pub(crate) fn passed(&self, min_decisions: u64) -> bool {
        self.wrong_schema.is_none()
            && self.mismatches == 0
            && self.unreadable == 0
            && self.decisions >= min_decisions
            && self.moving > 0
    }
}

/// Recompute every recorded decision and compare it with the one recorded, field by field.
pub(crate) fn replay_lines(text: &str) -> ReplayReport {
    let mut rep = ReplayReport::default();
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            rep.unreadable += 1;
            continue;
        };
        match v.get("kind").and_then(|k| k.as_str()) {
            Some("header") => {
                let schema = v.get("schema").and_then(|s| s.as_u64()).unwrap_or(0);
                if schema != DECIDE_SCHEMA as u64 {
                    rep.wrong_schema = Some(schema);
                    return rep;
                }
                rep.header = Some(v);
            }
            Some("decision") => {
                let stage = v.get("stage").and_then(|s| s.as_str()).unwrap_or("");
                let recorded = v.get("decision").cloned().unwrap_or(serde_json::Value::Null);
                let recomputed = match stage {
                    "moving" => {
                        let Ok(inputs) = serde_json::from_value::<MovingInputs>(
                            v.get("inputs").cloned().unwrap_or(serde_json::Value::Null),
                        ) else {
                            rep.unreadable += 1;
                            continue;
                        };
                        if inputs.interacting {
                            rep.moving += 1;
                        }
                        serde_json::to_value(decide_moving(&inputs)).unwrap_or(serde_json::Value::Null)
                    }
                    "present" => {
                        let Ok(inputs) = serde_json::from_value::<PresentInputs>(
                            v.get("inputs").cloned().unwrap_or(serde_json::Value::Null),
                        ) else {
                            rep.unreadable += 1;
                            continue;
                        };
                        serde_json::to_value(decide_present(&inputs)).unwrap_or(serde_json::Value::Null)
                    }
                    _ => {
                        rep.unreadable += 1;
                        continue;
                    }
                };
                rep.decisions += 1;
                if recomputed != recorded {
                    rep.mismatches += 1;
                    if rep.first.len() < 10 {
                        let fields: Vec<String> = match (recomputed.as_object(), recorded.as_object()) {
                            (Some(a), Some(b)) => a
                                .iter()
                                .filter(|(k, x)| b.get(*k) != Some(*x))
                                .map(|(k, x)| format!("{k}: recorded {} recomputed {x}", b.get(k).cloned().unwrap_or_default()))
                                .collect(),
                            _ => vec!["(not an object)".to_string()],
                        };
                        rep.first.push(format!(
                            "line {} frame {} view {} {stage}: {}",
                            n + 1,
                            v.get("frame").cloned().unwrap_or_default(),
                            v.get("view").cloned().unwrap_or_default(),
                            fields.join("; ")
                        ));
                    }
                }
            }
            _ => rep.unreadable += 1,
        }
    }
    rep
}

/// A replay fixture must hold at least this many decisions to mean anything (§7.5's anti-vacuity).
pub(crate) const REPLAY_MIN_DECISIONS: u64 = 200;

/// `--replay <file>`: recompute every decision in a recording and diff it against the recorded one.
/// Exit 0 = every decision recomputed exactly; 2 = a mismatch, an unreadable line, another schema,
/// or too little to judge (never a pass).
pub(crate) fn run_replay(path: &str) -> ! {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("--replay: cannot read {path}: {e}");
            crate::exit(2);
        }
    };
    let rep = replay_lines(&text);
    if let Some(h) = &rep.header {
        println!(
            "--replay: {path} - recorded by {} (tunables: {})",
            h.get("version").and_then(|x| x.as_str()).unwrap_or("?"),
            h.get("tunables").and_then(|x| x.as_str()).unwrap_or("?")
        );
    }
    if let Some(s) = rep.wrong_schema {
        println!("--replay: VACUOUS - the recording is schema {s}, this build reads {DECIDE_SCHEMA}");
        crate::exit(2);
    }
    println!(
        "--replay: {} decisions recomputed ({} of a moving frame), {} differ, {} unreadable",
        rep.decisions, rep.moving, rep.mismatches, rep.unreadable
    );
    for m in &rep.first {
        println!("  {m}");
    }
    let pass = rep.passed(REPLAY_MIN_DECISIONS);
    if !pass && rep.mismatches == 0 && rep.unreadable == 0 {
        println!(
            "--replay: VACUOUS - fewer than {REPLAY_MIN_DECISIONS} decisions, or none of a moving frame"
        );
    }
    println!("--replay: {}", if pass { "PASS - every decision recomputed exactly" } else { "FAIL" });
    crate::exit(if pass { 0 } else { 2 });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moving() -> MovingInputs {
        MovingInputs {
            px: 1442 * 1102,
            gpu_iter: 10_000,
            budget: 4_000_000_000,
            interacting: true,
            pin_frame: false,
            view: 0,
            is_pert: true,
            panel_px: 1442 * 1102,
            prefer_detail: false,
            tour_playing: false,
            can_chunk: true,
            zoom_oct_s: 2.5,
            frame_ms: 16.7,
            last_dt_ms: 16.7,
            prev_real: true,
            motion_rate: 2.0e7,
            min_motion_res: 0.3,
            pin_frames_last: 0,
            pin_res_last: 0.0,
            motion_res: 0.6,
            motion_res_measured: false,
            visible_res: 1.0,
            motion_pass_steps_last: 0,
            norm_hi: 0.0,
            esc_top: 0.0,
            esc_hist: None,
            tdr_bootstrap_steps: crate::tunables::cost().tdr_bootstrap_steps,
            motion_need_source: 0,
            motion_need_quantile: 1.0,
        }
    }

    /// §7.5's gate, on the CPU alone (so CI runs it): every recording under `validation/replays/`
    /// replays to the decisions it recorded, field by field. Anti-vacuity: the directory must hold a
    /// recording, and each must carry at least `REPLAY_MIN_DECISIONS` decisions, some of a moving
    /// frame. A deliberate change to a lifted decision turns this red, and the fixtures are then
    /// re-recorded (`FRACTADYNE_RECORD_DECIDE`, see `validation/replays/README.md`).
    #[test]
    fn every_recorded_session_replays_to_its_decisions() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../validation/replays");
        let mut files: Vec<_> = std::fs::read_dir(&dir)
            .expect("validation/replays exists")
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "jsonl"))
            .collect();
        files.sort();
        assert!(!files.is_empty(), "VACUOUS: no recording in {}", dir.display());
        // §7.5's coverage term: across the set, every branch of the decision is replayed at least
        // once — a fixture set satisfied by one regime grades the controller only where it is easy.
        let branches: [(&str, fn(&MovingInputs, &MovingDecision) -> bool); 9] = [
            ("settled", |i, _| !i.interacting),
            ("a pin frame", |i, _| i.pin_frame),
            ("prefer detail", |i, _| i.interacting && i.prefer_detail && !i.pin_frame),
            ("the rate cap measured", |_, d| d.rate_cap.is_some_and(|c| c.measured)),
            ("the rate cap's model", |_, d| d.rate_cap.is_some_and(|c| !c.measured)),
            ("the AIMD stepped", |_, d| d.aimd_stepped),
            ("the ladder's rung moved", |i, d| d.visible_res != i.visible_res),
            ("the ladder bound the scale", |_, d| d.res_scale < d.motion_scale),
            ("direct-mode motion", |i, _| i.interacting && !i.is_pert),
        ];
        let present: [(&str, fn(&PresentInputs, &PresentDecision) -> bool); 5] = [
            ("a settled compose held the display", |_, d| d.display_hold && !d.pin_gate),
            ("a pin held the display", |_, d| d.pin_gate),
            ("a snapshot was taken", |_, d| d.hold_copy),
            ("the settled-cost probe armed", |_, d| d.aa_probe.is_some()),
            ("the colour-pass filter widened", |_, d| d.aa_filter > 1),
        ];
        let mut hits = [0u64; 9];
        let mut present_hits = [0u64; 5];
        for f in &files {
            for line in std::fs::read_to_string(f).expect("readable fixture").lines() {
                let v: serde_json::Value = serde_json::from_str(line).expect("a JSON line");
                if v["kind"] == "decision" && v["stage"] == "present" {
                    let i: PresentInputs = serde_json::from_value(v["inputs"].clone()).expect("inputs");
                    let d: PresentDecision = serde_json::from_value(v["decision"].clone()).expect("decision");
                    for (h, (_, f)) in present_hits.iter_mut().zip(present.iter()) {
                        *h += f(&i, &d) as u64;
                    }
                    continue;
                }
                if v["kind"] != "decision" || v["stage"] != "moving" {
                    continue;
                }
                let i: MovingInputs = serde_json::from_value(v["inputs"].clone()).expect("inputs");
                let d: MovingDecision = serde_json::from_value(v["decision"].clone()).expect("decision");
                for (h, (_, f)) in hits.iter_mut().zip(branches.iter()) {
                    *h += f(&i, &d) as u64;
                }
            }
        }
        for (h, (name, _)) in hits.iter().zip(branches.iter()) {
            assert!(*h > 0, "VACUOUS: no replayed decision where {name}");
        }
        for (h, (name, _)) in present_hits.iter().zip(present.iter()) {
            assert!(*h > 0, "VACUOUS: no replayed present decision where {name}");
        }
        for f in &files {
            let rep = replay_lines(&std::fs::read_to_string(f).expect("readable fixture"));
            assert!(
                rep.passed(REPLAY_MIN_DECISIONS),
                "{}: {} decisions ({} moving), {} differ, {} unreadable, schema {:?}\n{}",
                f.display(),
                rep.decisions,
                rep.moving,
                rep.mismatches,
                rep.unreadable,
                rep.wrong_schema,
                rep.first.join("\n")
            );
        }
    }

    #[test]
    fn a_replay_catches_a_changed_decision() {
        // The control: the gate above can go red. A record whose decision is not what the inputs
        // decide is a mismatch, named by field.
        let i = MovingInputs { norm_hi: 5_000.0, motion_pass_steps_last: 200_000_000, ..moving() };
        let mut d = decide_moving(&i);
        let good = serde_json::json!({"kind": "decision", "frame": 7, "view": 0, "stage": "moving",
            "inputs": i, "decision": d}).to_string();
        let ok = replay_lines(&good);
        assert_eq!((ok.decisions, ok.mismatches, ok.moving), (1, 0, 1));
        d.visible_res *= 0.5;
        let bad = serde_json::json!({"kind": "decision", "frame": 7, "view": 0, "stage": "moving",
            "inputs": i, "decision": d}).to_string();
        let rep = replay_lines(&bad);
        assert_eq!(rep.mismatches, 1);
        assert!(rep.first[0].contains("visible_res"), "{}", rep.first[0]);
        // Another schema is refused, not compared.
        let other = format!("{{\"kind\":\"header\",\"schema\":{}}}\n{good}", DECIDE_SCHEMA + 1);
        assert!(replay_lines(&other).wrong_schema.is_some());
        // Too little to judge is never a pass.
        assert!(!ok.passed(REPLAY_MIN_DECISIONS));
    }

    #[test]
    fn the_same_inputs_give_the_same_decision() {
        let i = moving();
        assert_eq!(decide_moving(&i), decide_moving(&i));
    }

    #[test]
    fn a_settled_frame_is_native_and_touches_no_state() {
        let i = MovingInputs { interacting: false, ..moving() };
        let d = decide_moving(&i);
        assert_eq!((d.motion_scale, d.res_scale), (1.0, 1.0));
        assert_eq!((d.motion_res, d.motion_res_measured, d.visible_res), (i.motion_res, false, i.visible_res));
        assert_eq!(d.vis_sizing, [0.0; 3]);
        assert!(d.rate_cap.is_none() && !d.aimd_stepped);
    }

    #[test]
    fn a_pin_frame_keeps_its_captured_geometry() {
        let d = decide_moving(&MovingInputs { pin_frame: true, ..moving() });
        assert_eq!((d.motion_scale, d.res_scale), (1.0, 1.0));
        assert!(!d.aimd_stepped, "a pin's passes do not step the AIMD");
    }

    #[test]
    fn the_aimd_steps_only_after_a_real_frame() {
        let stepped = decide_moving(&moving());
        assert!(stepped.aimd_stepped && stepped.motion_res_measured);
        let held = decide_moving(&MovingInputs { prev_real: false, ..moving() });
        assert!(!held.aimd_stepped && !held.motion_res_measured);
        assert_eq!(held.motion_res, 0.6);
    }

    #[test]
    fn the_ladder_sizes_a_moving_frame_to_reach_its_escapes() {
        // A pass budget that cannot reach a 5,000-iteration top over the whole panel drops the rung.
        let i = MovingInputs { norm_hi: 5_000.0, motion_pass_steps_last: 200_000_000, ..moving() };
        let d = decide_moving(&i);
        assert!(d.visible_res < 1.0, "rung {}", d.visible_res);
        assert!(d.res_scale <= d.visible_res);
        assert_eq!(d.vis_sizing[0], 5_000.0);
        // The complete walk's top is read only under MOTION_NEED_SOURCE=1.
        let other = decide_moving(&MovingInputs { esc_top: 50.0, ..i.clone() });
        assert_eq!(other, d, "source 0 ignores esc_top");
        let walk = decide_moving(&MovingInputs { esc_top: 50.0, motion_need_source: 1, ..i });
        assert_eq!(walk.vis_sizing[0], 50.0);
    }

    #[test]
    fn a_recorded_decision_round_trips() {
        let i = MovingInputs { esc_hist: Some(vec![0, 3, 7, 0]), ..moving() };
        let d = decide_moving(&i);
        let back_i: MovingInputs = serde_json::from_str(&serde_json::to_string(&i).unwrap()).unwrap();
        let back_d: MovingDecision = serde_json::from_str(&serde_json::to_string(&d).unwrap()).unwrap();
        assert_eq!(back_i, i);
        assert_eq!(back_d, d);
        assert_eq!(decide_moving(&back_i), d, "a replayed decision is the recorded one");
    }
}
