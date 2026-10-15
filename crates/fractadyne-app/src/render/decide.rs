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
    /// The per-pass fixed cost, ms: this adapter's calibration (`calibration::pass_fixed_ms`, or
    /// `PASS_FIXED_MS` under `--set`). ⚠An input, never read in here: a replay runs in a process with
    /// no adapter, where the calibration reads the default — the first `--uitest` recording's one
    /// mismatch (0.21 ms on the RTX 3080 against the 0.2 default).
    pub(crate) pass_fixed_ms: f64,
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
        motion_need, motion_pass_steps_fixed, motion_res_step, refresh_passes_allowed, refresh_res_cap,
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
            let pass = motion_pass_steps_fixed(
                i.motion_rate,
                crate::tunables::MOTION_PASS_MS,
                i.pass_fixed_ms,
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
        // ⭐W6 (U42): a snapshot only of a texture KNOWN to hold a picture, exactly as the pin
        // branch below takes one. This branch took it of whatever the texture held — a cleared
        // texture's blank, an abandoned walk's residue — and served it for the whole compose. With
        // nothing verified the display shows the compose live (the GPU gate needs a snapshot), the
        // honest progressive-plus-spinner; the GPU also checks the snapshot's pixels are the view
        // the app names (`MandelbrotParams::hold_stamp`).
        let fresh = !hold_active && i.live_verified;
        if fresh {
            hold_verified = Some(true);
        }
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

/// The adaptive live-iteration climb's state, one view, carried between frames (`Perf::iter_boost`,
/// `iter_probe`, `iter_plateau`, `iter_stall`, `iter_stall_base`, `iter_exhausted`, `iter_repicked`,
/// `iter_verdict`, and the settled reading published for the status bar: `capped_frac`,
/// `budget_measured`, `budget_maxed`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct IterState {
    pub(crate) boost: f64,
    pub(crate) probe: Option<(f64, f64)>,
    pub(crate) plateau: bool,
    pub(crate) stall: u8,
    pub(crate) stall_base: f64,
    pub(crate) exhausted: bool,
    pub(crate) repicked: bool,
    pub(crate) verdict: Option<u32>,
    pub(crate) capped_frac: Option<f64>,
    pub(crate) budget_measured: u32,
    pub(crate) budget_maxed: bool,
}

/// What [`decide_iter`] reads: `bp_frame_budget`'s climb inputs, one view, one frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct IterInputs {
    pub(crate) interacting: bool,
    /// Auto-iterations (the zoom cap × the boost) rather than the user's explicit count.
    pub(crate) auto_iter: bool,
    pub(crate) eff_iter: u32,
    pub(crate) log2mag: f64,
    /// The frame cost's hard iteration ceiling (`FrameCost::iter_cap`).
    pub(crate) iter_cap: u32,
    /// The MAXITER reading drained this frame, `(frac_f32_bits << 32) | armed_max_iter`; `None` when
    /// nothing was published (only a pass that reached its ask publishes).
    pub(crate) reading: Option<u64>,
    /// The view's reference: its length in samples, and whether it is evidence for the seed (an
    /// orbit that escaped on its own, not a truncated partial — see `boost_seed_from_reference`).
    pub(crate) ref_len: u32,
    pub(crate) ref_escaped: bool,
    /// A reference build for this view is in flight (`recompute_rx`).
    pub(crate) recompute_in_flight: bool,
    pub(crate) state: IterState,
}

/// What [`decide_iter`] decides, and the state it writes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct IterDecision {
    pub(crate) state: IterState,
    /// The budget the reading is judged at (the boost after any seed) — the normalization
    /// signature's ask — and whether a reading reached its ask this frame, and whether it was then
    /// discarded as measured at another budget.
    pub(crate) budget_now: u32,
    pub(crate) reached_ask: bool,
    pub(crate) stale: bool,
    /// Re-pick the view's reference at the full ask: the caller clears the incumbent (no
    /// reuse-extend) and drops any in-flight extend of it.
    pub(crate) repick: bool,
    /// The frame's ask from the climb's final boost (`Perf::iter_planned`), and that ask under the
    /// frame cost's ceiling.
    pub(crate) planned: u32,
    pub(crate) gpu_iter: u32,
    /// What the climb did, in order, for the trace.
    pub(crate) events: Vec<IterEvent>,
}

/// One step of the climb, for the `gpu` trace (`IterEvent::trace`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum IterEvent {
    /// Motion ended a run of raises that had bought nothing: the boost went back to where it began.
    FlatRunEnded { stall: u8, from: f64, to: f64 },
    /// The boost was seeded from the reference orbit's length.
    Seeded { from: f64, to: f64, ref_len: u32 },
    /// All-capped at the full appetite: the reference is re-picked at the full ask.
    Repick { frac: f64, budget: u32 },
    /// Still all-capped after that re-pick: interior; the boost reverts to 1.
    Exhausted { frac: f64, budget: u32, from: f64 },
    /// Several raises bought nothing on a partly capped frame: interior; back to where the run began.
    Plateau { to: f64 },
    /// Capped: the boost was raised (`flat`: the run of fruitless raises so far, if this was one).
    Raised { frac: f64, from: f64, to: f64, flat: Option<u8> },
}

impl IterEvent {
    /// Traced before the frame's normalization drains (as the climb's first half always was); the
    /// conclusions are traced after them.
    pub(crate) fn before_drains(&self) -> bool {
        matches!(self, IterEvent::FlatRunEnded { .. } | IterEvent::Seeded { .. })
    }

    pub(crate) fn trace(&self) {
        let line = match *self {
            IterEvent::FlatRunEnded { stall, from, to: back } => format!(
                "adaptive iter: motion ends a flat run ({}/{}) — boost {:.1}→{back:.1}",
                stall,
                crate::ITER_STALL_LIMIT,
                from
            ),
            IterEvent::Seeded { from: old, to: seed, ref_len } => {
                format!("adaptive iter: seeded boost {old:.1}→{seed:.1} from reference len {ref_len}")
            }
            IterEvent::Repick { frac, budget: budget_now } => format!(
                "adaptive iter: {:.1}% capped at the full appetite ({budget_now}) — \
                 re-picking the reference at the full ask before concluding interior",
                frac * 100.0
            ),
            IterEvent::Exhausted { frac, budget: budget_now, from: boost } => format!(
                "adaptive iter: exhausted — {:.1}% capped at the full appetite \
                 ({budget_now}) even after a fresh appetite pick; reverting boost {boost:.1}→1.0",
                frac * 100.0
            ),
            IterEvent::Plateau { to: back } => format!("adaptive iter: plateau (interior) — boost back to {back:.1}"),
            IterEvent::Raised { frac, from: boost, to, flat } => format!(
                "adaptive iter: {:.1}% capped — boost {boost:.1}→{:.1}{}",
                frac * 100.0,
                to,
                if let Some(stall) = flat {
                    format!(" (flat {}/{})", stall, crate::ITER_STALL_LIMIT)
                } else {
                    String::new()
                }
            ),
        };
        crate::diag::trace("gpu", line);
    }
}

/// The ADAPTIVE LIVE ITERATION climb (`bp_frame_budget`'s, lifted): the boost on the zoom cap, its
/// seed from the reference, the probe that raises it while raising helps, the plateau and the
/// exhausted revert, the appetite re-pick, the verdict, and the frame's resulting ask. Pure.
#[must_use]
pub(crate) fn decide_iter(i: &IterInputs) -> IterDecision {
    use super::{boost_seed_from_reference, live_iter_budget};
    // ADAPTIVE LIVE ITERATION BUDGET (settled frames only). `zoom_iter_cap`'s 256/octave slope
    // under-budgets dense near-Misiurewicz fields (escape counts run ~226–300/octave there), so
    // a settled deep view can show smooth "capped blobs" where the export shows dendrites — at
    // the reported 1.7e55× spar view, 40% of pixels hit the cap (maxiter counter) while the
    // full appetite escapes every one. The GPU's MAXITER event counter (read back per settled
    // full-frame iterate) drives a probe: raise the cap multiplier while raising measurably
    // reduces the capped fraction; a raise that doesn't help means TRUE interior in view (a
    // minibrot) → revert and stop (`iter_plateau`), so interior views never pay a runaway
    // budget.
    //
    // "Doesn't help" needs EVIDENCE, not one flat step. Measured at the 3.3e61× three-spar:
    // the capped fraction is exactly 100% at ×1.0, ×1.6 AND ×2.6, then collapses to 12% at
    // ×4.1 and 0.6% at ×6.6. Latching on the first flat step reverted to boost 1.0 and left
    // the view permanently BLACK — every pixel capped, painted as interior. So the probe must
    // see `crate::ITER_STALL_LIMIT` flat steps in a row before concluding there is nothing to find.
    let mut s = i.state;
    let interacting = i.interacting;
    let mut events = Vec::new();
    let mut repick = false;
    // Where the climb stood before this frame — its verdict below asks whether it moved.
    let (boost_in, repicked_in) = (s.boost, s.repicked);
    // The probe baseline and the interior plateau are properties of ONE settled view: any
    // motion invalidates both (comparing a new view's capped fraction against another
    // view's baseline falsely reads "the raise didn't help" and latches the plateau —
    // navigating to a starved view then stuck at boost 1.0 forever). The boost itself is
    // kept: it's a multiplier on the depth-scaled cap, so it carries sensibly and the
    // probe re-converges from wherever it is.
    if interacting {
        // ⭐…except a run of raises that has so far bought NOTHING (`iter_stall` > 0: every
        // raise since `iter_stall_base` measured flat). Motion used to interrupt it keeping
        // the boost it had reached: a glide that began while the 1× view's interior (3.0%
        // capped at every step) was still being probed carried ×4.1 into every frame after —
        // 22k iterations at the df32 switch where 5.4k resolve the picture, 2.5× over the
        // dispatch ceiling. Revert to where the fruitless run began, as its conclusion would.
        if s.stall > 0 {
            let back = s.stall_base;
            events.push(IterEvent::FlatRunEnded { stall: s.stall, from: s.boost, to: back });
            s.boost = back;
        }
        s.probe = None;
        s.plateau = false;
        s.stall = 0;
        s.exhausted = false;
        s.repicked = false; // a new settle earns a fresh appetite re-pick
        s.capped_frac = None; // a moving frame's reading describes another view
    }
    let v = i.reading.unwrap_or(u64::MAX);
    // The GPU publishes this reading only from a pass that REACHED ITS ASK (the walk's end),
    // whatever budget it was armed at — the one test of "this reading saw the whole walk".
    let reached_ask = v != u64::MAX;
    // The reading is `(frac_f32_bits << 32) | armed_max_iter`. Only adapt on readings
    // measured AT the current boost's budget: the readback lands ~2 frames late, and a
    // stale pre-raise reading ("still 100% capped") right after a raise falsely reads as
    // "the raise didn't help" → the interior plateau latches and the view sticks black at
    // fully-starved views (observed live at a 100%-capped spar session).
    // ⭐SEED THE BOOST FROM THE REFERENCE ORBIT, so the budget reaches this location's real
    // need in ONE settle instead of climbing there blind. The depth slope (`zoom_iter_cap`)
    // leaves a settled deep view capping pixels at, say, 69k while the reference we already
    // built for it is 256k+ samples long — pure waste, and the direct cause of a minibrot
    // rendering solid black under auto-iterations when a forced high count resolves it. The
    // reference length is the honest per-location estimate (no pixel can outrun the orbit),
    // so jump the boost to it rather than walking up in slow 2.5× steps that a deep mode-2
    // view spends seconds on and that the interior-plateau guard can abandon before arriving.
    //
    // Raise-only, settled-only, auto-only, and never past a latched interior plateau: the
    // existing climb/plateau/exhausted logic keeps ownership of termination. A genuine
    // interior seeded high is still fully capped, so it climbs the rest to the appetite and
    // reverts to the cheap budget exactly as before — just without the long black stare on
    // the way. The seed feeds `boost_now` below, so the probe's `budget_now` and the frame's
    // dispatched budget (which reads `iter_boost[vb]` afresh) stay the one shared value.
    if !interacting && i.auto_iter && !s.plateau {
        let cap = crate::zoom_iter_cap(i.log2mag).max(256) as f64;
        let ref_len = i.ref_len;
        // The orbit's length is evidence only if the orbit ENDED ON ITS OWN (see
        // `boost_seed_from_reference`). A truncated coarse reference is not a finished
        // orbit at all, so it never qualifies either (`IterInputs::ref_escaped`).
        let escaped = i.ref_escaped;
        let seed = boost_seed_from_reference(ref_len, cap, escaped);
        if seed > s.boost {
            let old = s.boost;
            s.boost = seed;
            // The climb, if the seed is not enough, re-measures from here; a stale baseline
            // from before the jump would read the first post-seed frame as "unhelpful".
            s.probe = None;
            s.stall = 0;
            events.push(IterEvent::Seeded { from: old, to: seed, ref_len });
        }
    }
    let boost_now = s.boost;
    let budget_now = live_iter_budget(i.eff_iter, i.log2mag, boost_now, !i.auto_iter);
    let (v, frac_bits) = match v {
        u64::MAX => (u64::MAX, 0u32),
        packed => {
            let armed_iter = packed as u32;
            if armed_iter == budget_now {
                (packed, (packed >> 32) as u32)
            } else {
                (u64::MAX, 0) // stale budget — discard
            }
        }
    };
    let stale = reached_ask && v == u64::MAX;
    // Adapt only while settled and only when the zoom cap is the binding constraint
    // (a frame already at the full appetite can't be helped by raising it).
    let boost = boost_now;
    let cap_bound = budget_now < i.eff_iter.min(crate::MAX_ITER_LIMIT);
    // Publish the capped fraction for the status-bar limit diagnostic — INDEPENDENT of the
    // adapt path below. When the user's own Iterations setting is the binding constraint
    // (`cap_bound` false) the probe never runs, and that is precisely when they most need
    // to be told the budget is the limit: the probe can't raise past their setting.
    if v != u64::MAX && !interacting {
        let frac = f32::from_bits(frac_bits) as f64;
        if frac.is_finite() {
            s.capped_frac = Some(frac);
            // Can the app still raise this budget on its own? Only while the zoom cap
            // binds AND the boost has headroom.
            s.budget_maxed = !cap_bound || boost_now >= crate::ITER_BOOST_MAX * 0.99;
            s.budget_measured = budget_now;
        }
    }
    // (`bp_frame_budget` drains the escape-range, gradient and content readings between the
    // publish above and the conclusions below. Neither side reads what the other writes, so
    // the climb decides whole, before them, and its conclusions' traces follow them.)
    if v != u64::MAX && !interacting && !cap_bound && !s.plateau {
        // At FULL APPETITE and still essentially all-capped: raising is no longer
        // possible, so the frame is flat at the most this view can be given. Whether that
        // is deep interior (normal) or a view needing more than the app can offer, the
        // IMAGE is identical at a cheaper budget — so revert to the cheap one, stop
        // probing, and record it for the status bar to report honestly. Without this the
        // budget stays pinned at the appetite forever, paying millions of iterations per
        // pixel to render the same flat frame.
        let frac = f32::from_bits(frac_bits) as f64;
        if frac.is_finite() && frac > 0.98 && boost > 1.0 {
            // ⭐Before concluding INTERIOR and reverting to black, try ONE fresh reference
            // pick at the full appetite. The reference in hand may have been chosen cheaply
            // during motion — the view CENTRE, which escapes early and covers this view
            // badly — and the settled path reuse-EXTENDED it rather than re-picking. A pick
            // at the appetite runs the deep-perturb engine and finds a better point (measured
            // 2026-09-10 at 9.34e80×: the export/appetite pick lands OFF-CENTRE, offset
            // ≈(−0.04,+0.02), and resolves where the reused motion pick stays solid black).
            // Clearing `ref_pt` forces the next build to pick fresh instead of reuse — and it
            // picks at the appetite ask, because the boost is at the appetite right here.
            // ⚠Wait for that build to land before deciding: reverting while it is in flight
            // would drop the boost, and the re-pick would then rebuild at the throttled ask
            // and pick the poor centre all over again. Once per settle (`iter_repicked`): if
            // the appetite-picked reference is STILL all-capped, the view is genuinely
            // interior and the revert below stands.
            if !s.repicked {
                s.repicked = true;
                // The caller forces a fresh pick (no reuse-extend) and drops any in-flight
                // extend of the poor incumbent.
                repick = true;
                s.probe = None; // re-measure cleanly against the new reference
                s.stall = 0;
                events.push(IterEvent::Repick { frac, budget: budget_now });
            } else if !i.recompute_in_flight {
                // The appetite re-pick has landed and it is still all-capped: genuinely
                // interior. Revert to the cheap budget (same flat image) as before.
                s.plateau = true;
                s.exhausted = true;
                s.boost = 1.0;
                events.push(IterEvent::Exhausted { frac, budget: budget_now, from: boost });
            }
        }
    }
    if v != u64::MAX && !interacting && cap_bound && !s.plateau {
        let frac = f32::from_bits(frac_bits) as f64;
        // 0.05%: low enough to chase a thin Misiurewicz-spar core (the 1.7e55× case
        // converges 43% → 0.12% → 0.001% over two raises), high enough to ignore stray
        // undecided pixels; the plateau probe still stops true interiors regardless.
        if frac.is_finite() && frac > 0.0005 {
            let unhelpful = s.probe.is_some_and(|(pb, pf)| boost > pb && frac > pf * 0.8);
            // Step size scales with how far from resolved we are. A nearly all-capped
            // frame is a flat/black screen and may need several times the budget, so
            // walking up in 1.6× steps spends many settle cycles showing the user
            // nothing; once mostly resolved, fine steps avoid overshooting the cost.
            let step = if frac > 0.9 { 2.5 } else { 1.6 };
            // A frame that is essentially ALL capped looks identical whether we revert or
            // keep climbing, so a flat step there is NOT evidence of interior — it can
            // simply mean the budget is still orders of magnitude short. Measured at
            // 6.5e94×: that view needs ~121× the depth-scaled cap, so three flat 2.5×
            // steps only reached 15.6×, "proved" interior, reverted to 1.0 — and left a
            // permanently BLACK screen on a view that resolves fine at 10M. While fully
            // capped, climb to the ceiling; the `!cap_bound` branch above concludes it.
            let fully_capped = frac > 0.98;
            if !fully_capped && unhelpful && s.stall + 1 >= crate::ITER_STALL_LIMIT {
                // Several raises bought nothing on a PARTIALLY capped frame — the
                // remaining capped pixels are interior. Revert to where the run began.
                let back = s.stall_base;
                s.plateau = true;
                s.boost = back;
                events.push(IterEvent::Plateau { to: back });
            } else {
                if unhelpful && !fully_capped {
                    if s.stall == 0 {
                        // Remember where to fall back to if this run stays fruitless.
                        s.stall_base = s.probe.map_or(boost, |(pb, _)| pb);
                    }
                    s.stall += 1;
                } else {
                    s.stall = 0;
                }
                s.probe = Some((boost, frac));
                s.boost = (boost * step).min(crate::ITER_BOOST_MAX);
                events.push(IterEvent::Raised {
                    frac,
                    from: boost,
                    to: s.boost,
                    flat: unhelpful.then_some(s.stall),
                });
            }
        } else if frac.is_finite() {
            s.probe = Some((boost, frac));
            s.stall = 0;
        }
    }
    // ⭐THE CLIMB'S VERDICT (`Perf::iter_verdict`): a reading measured at this budget that
    // left it where it was. Supersampling begins on nothing less (`accum_inputs_pending`):
    // every raise and revert re-decides the colour range and restarts a run.
    let moved = s.boost != boost_in || s.repicked != repicked_in;
    if interacting || moved {
        s.verdict = None;
    } else if v != u64::MAX {
        s.verdict = Some(budget_now);
    }
    // The boost applies to ALL frames, motion included: gating it to settled frames made every
    // MOTION refresh frame render at the unboosted cap — at a starved view (43–96% capped)
    // that frame is nearly all-interior, so zooming painted BLACK over the good reprojection
    // until settle re-boosted ("screen goes black until it repaints"). A boosted motion frame
    // costs up to boost× more, which the motion-res controller absorbs as resolution — softer
    // beats black. (ADAPTATION stays settled-only; this is only the budget's application.)
    // ✅A1 RESOLVED (v0.2.40-beta.53): an explicit iteration count is honoured verbatim, in
    // auto mode the zoom cap × boost applies as before — one shared formula with the boost
    // probe's `budget_now`, so the two can never drift (`live_iter_budget`; the drift failure
    // mode and the freeze-guard coupling that blocked this for two releases are documented on
    // the function). `iter_cap` (= MAX_ITER_LIMIT on both branches) folds into it.
    // ⛔TOUR GLIDES MUST PRICE THE SCRIPT'S ASK TOO (attempted beta.80, REVERTED same day):
    // pricing glide frames at the zoom cap × boost rendered the grand tour's hard spar holds
    // 100% BLACK on `--livetest` (hold-e82: cap ~80k where the field needs the scripted 2M —
    // per-keyframe budgets exist precisely because the depth formula under-budgets hard
    // fields), and an ease-out approach is nominally "gliding" while visually parked, so the
    // starved frames are exactly the ones the viewer stares at.
    // `planned` is what the climb's verdict must name (`Perf::iter_planned`).
    let planned = live_iter_budget(i.eff_iter, i.log2mag, s.boost, !i.auto_iter);
    let gpu_iter = planned.min(i.iter_cap);
    IterDecision { state: s, budget_now, reached_ask, stale, repick, planned, gpu_iter, events }
}

/// What [`decide_probe`] reads: the budget-climb probe's inputs (`bp_chunk_tiling`), one view, one
/// frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct ProbeInputs {
    pub(crate) interacting: bool,
    pub(crate) offscreen: bool,
    pub(crate) will_reproject: bool,
    pub(crate) tour_playing: bool,
    pub(crate) key_changed: bool,
    /// A settle tile dispatches this frame; the chunk range this frame dispatches.
    pub(crate) tile: bool,
    pub(crate) chunk_range: Option<[u32; 2]>,
    /// The view's work budget has converged (`fe_budget_ok`).
    pub(crate) budget_ok: bool,
    /// This frame, and the frame of the view's last real dispatch (`fe_dispatch_frame`).
    pub(crate) frame: u64,
    pub(crate) dispatch_frame: u64,
    pub(crate) spx: u64,
    pub(crate) ss: u32,
    pub(crate) gpu_iter: u32,
    /// The view's measured work budget (0 = none yet) and the bootstrap that stands in for it.
    pub(crate) fe_budget: u64,
    pub(crate) bootstrap_steps: u64,
    /// The overridable tunables it reads: `EXPLICIT_DISPATCH_CAP`, and the bounds a measured budget
    /// is clamped to (`TDR_MIN_STEPS`, `TDR_STEPS_CEIL`; see `budget_base`).
    pub(crate) explicit_dispatch_cap: u64,
    pub(crate) tdr_min_steps: u64,
    pub(crate) tdr_steps_ceil: u64,
}

/// What [`decide_probe`] decides.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct ProbeDecision {
    /// Force a re-measure this frame: a fresh probe nonce under an unchanged key.
    pub(crate) fired: bool,
    /// The nominal steps the forced pass carries, which its measurement is paired with (0 unless
    /// fired).
    pub(crate) steps: u64,
}

/// The BUDGET-CLIMB PROBE (`bp_chunk_tiling`'s, lifted): whether a settled view whose work budget has
/// not converged, and where nothing else dispatches this frame, forces a re-measure. Pure.
#[must_use]
pub(crate) fn decide_probe(i: &ProbeInputs) -> ProbeDecision {
    use super::{budget_base_in, probe_would_price};
    let (interacting, offscreen, will_reproject, key_changed) =
        (i.interacting, i.offscreen, i.will_reproject, i.key_changed);
    let (spx, ss, gpu_iter) = (i.spx, i.ss, i.gpu_iter);
    let mut probe_fired = false;
    let mut steps = 0;
    // Budget-climb probe: on a settled view whose budget hasn't converged and where NOTHING
    // else will dispatch this frame, force a re-measure. Without this the climb deadlocks at
    // the resolution floor: one ×1.5 budget step is too small to unfloor a 16×16 frame under a
    // huge iteration count, so the key never changes → no dispatch → no measurement → the
    // budget freezes one step off the bootstrap and the view stays pixellated forever (seen
    // at the 197k× spar with an explicit 4M count after restart — interaction used to mask it
    // by re-keying every frame, and before beta.65/66 this state usually crashed first).
    // Each forced dispatch is budget-sized by the shrink above, so it is TDR-safe by
    // construction; convergence (`fe_budget_ok`) stops the probe and hands off to the settle.
    if !interacting
        && !offscreen
        && !will_reproject
        && !i.tour_playing
        && !key_changed
        && !i.tile
        && i.chunk_range.is_none()
        && !i.budget_ok
        // PACED: at least 3 frames since the last real dispatch. An unpaced probe re-created
        // the beta.48 death — near convergence each dispatch is ~TDR_BUDGET_MS (400 ms since
        // 2026-08-16; it was 900 ms when this was written, which is what made it lethal), and
        // firing one EVERY frame ran ~1 s dispatches back-to-back with zero idle → device lost
        // at f=68 of the first climb soak. The gap leaves present-able frames between
        // dispatches (~⅓ duty), which is the profile the converged controller already runs.
        && i.frame.saturating_sub(i.dispatch_frame) >= 3
        // ⭐AND ONLY IF THE READING WOULD BE PRICED. A frame far under budget cannot converge
        // it — the pricing rule discards exactly that reading — so probing for one is an
        // infinite loop of real GPU dispatches on a static view. See `probe_would_price`;
        // this is the same threshold, read from the same constant, so the rule that spends
        // the dispatch and the rule that spends the reading can never disagree.
        && probe_would_price(
            spx.saturating_mul((ss as u64).saturating_mul(ss as u64))
                .saturating_mul(gpu_iter.max(1) as u64),
            budget_base_in(i.fe_budget, i.bootstrap_steps, i.tdr_min_steps, i.tdr_steps_ceil),
        )
        // CEILINGED: the probe stops growing the budget at ~2e10 nominal (~60–90 ms real in
        // the no-skip regime) instead of riding the controller to its target. The
        // second climb soak PROVED the target is lethal in this regime even paced: at a
        // 99-sample escaped reference with bla_skip=0 (nominal = real, zero preemption inside
        // the fullscreen draw), the ×1.5 growth step from ~600 ms landed a ~1.3 s dispatch and
        // lost the device — the intermittent Event-153 marginality of the beta.48 saga, made
        // reproducible. Below the ceiling the deadlock is still broken (16×16 → ~84×66 at a
        // 4M explicit ask, stable and alive); a native-res render of such an ask needs the
        // chunked path extended to perturbation modes (TODO), not bigger single dispatches.
        && budget_base_in(i.fe_budget, i.bootstrap_steps, i.tdr_min_steps, i.tdr_steps_ceil)
            < i.explicit_dispatch_cap
    {
        // The caller takes a fresh probe nonce (a re-measure under an unchanged key).
        probe_fired = true;
        // This frame now runs a real pass under an unchanged key — pair the measurement
        // with its cost (the same bookkeeping the `key_changed` block below does).
        steps = spx
            .saturating_mul((ss as u64).saturating_mul(ss as u64))
            .saturating_mul(gpu_iter.max(1) as u64);
    }
    ProbeDecision { fired: probe_fired, steps }
}

/// The chunk walk's band ledger and pass bookkeeping, one view, carried between frames
/// (`Perf::chunk_inflight`, `chunk_bands`, `chunk_pass_dt`, `motion_step_last`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct WalkLedger {
    /// The one unpriced pass in flight: its iterations, its band, the wall accumulated since it went
    /// out, and whether the lethal band has already shed the licences for it.
    pub(crate) inflight: Option<(u32, u8, f64, bool)>,
    /// Per iteration band: the licence (iterations a pass may carry there) its prices have earned.
    pub(crate) bands: [u32; crate::tunables::CHUNK_BANDS],
    /// The last priced pass's wall price, ms.
    pub(crate) pass_dt: f64,
    /// The last moving / pin pass's iteration step (the growth limiter's memory).
    pub(crate) motion_step_last: u32,
    /// A pin pass RELEASED at its completion before its GPU reading landed, whose price waits for
    /// that reading: `(size, band, the frame it went out, its completion stand-in, ms)`. See
    /// [`decide_walk_price`]'s deferred feed. (Absent when none, so a recording made before it
    /// compares equal wherever nothing was parked.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) late: Option<(u32, u8, u64, f64)>,
}

/// What [`decide_walk_price`] reads: `bp_chunk_tiling`'s pricing and sizing inputs, one view, one frame
/// of a chunk-governed compose.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct WalkPriceInputs {
    pub(crate) interacting: bool,
    pub(crate) pin_frame: bool,
    pub(crate) auto_iter: bool,
    pub(crate) spx: u64,
    pub(crate) ss: u32,
    /// One dispatch's step budget (the admission when it applies).
    pub(crate) tdr_steps: u64,
    pub(crate) gpu_iter: u32,
    /// The view has a measured work budget (`fe_budget` ≠ 0); the worst rate measured in this mode,
    /// steps per ms; the moving-pass rate estimate (`Perf::motion_rate_now`).
    pub(crate) budget_measured: bool,
    pub(crate) worst_rate: Option<f64>,
    pub(crate) motion_rate: f64,
    /// Frame timing: the last frame interval, the real frame period, the present-throttle count,
    /// and this frame.
    pub(crate) last_dt_ms: f64,
    pub(crate) frame_period_ms: f64,
    pub(crate) present_throttle: u32,
    pub(crate) frame: u64,
    /// The last pin pass: its GPU timestamp reading (-1 = none), the frame it went out, the pin
    /// passes in the queue, its completion stamp and its dispatch stamp (µs, `app_micros`).
    pub(crate) pin_pass_gpu_ms: f64,
    pub(crate) pin_pass_frame: u64,
    pub(crate) pin_inflight: u32,
    pub(crate) pin_done_at: u64,
    pub(crate) pin_dispatch_at: u64,
    /// The GPU reading of the released pin pass waiting in `ledger.late`, ms (< 0 = not landed),
    /// paired by the frame the reading names (`Perf::pin_late_gpu_ms`).
    #[serde(default = "no_reading")]
    pub(crate) late_gpu_ms: f64,
    /// The device gives no GPU timestamps: a pin pass's completion stand-in is the only price it
    /// will ever have, so it is fed at once.
    #[serde(default)]
    pub(crate) no_timestamps: bool,
    pub(crate) ledger: WalkLedger,
    /// The overridable tunables it reads, and this adapter's per-pass fixed cost (see
    /// `MovingInputs::pass_fixed_ms`).
    pub(crate) tdr_budget_ms: f64,
    pub(crate) tdr_explicit_budget_ms: f64,
    pub(crate) tdr_lethal_ms: f64,
    pub(crate) tdr_bootstrap_steps: u64,
    pub(crate) admission: u64,
    pub(crate) pass_fixed_ms: f64,
}

/// What [`decide_walk_price`] decides, and the state it writes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct WalkPriceDecision {
    /// The least a pass may carry (iterations), the wall-clock target its passes are priced
    /// against, the pass's step budget, and its iteration step (growth-limited when moving or pinned).
    pub(crate) floor: u32,
    pub(crate) target_ms: f64,
    pub(crate) pass_steps: u64,
    pub(crate) budget_step: u32,
    pub(crate) ledger: WalkLedger,
    /// The lethal-band shed's outcome, when one happened this frame.
    pub(crate) retreat: Option<super::RetreatOutcome>,
    pub(crate) events: Vec<WalkPriceEvent>,
}

/// A recorded input's "no reading" (`WalkPriceInputs::late_gpu_ms` in recordings made before it).
fn no_reading() -> f64 {
    -1.0
}

/// What the pricing did, for the caller's log and trace.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum WalkPriceEvent {
    /// The in-flight pass reached the lethal band before draining: the licences were shed.
    LethalShed { size: u32, band: u8, acc: f64, floor: u32, outcome: super::RetreatOutcome },
    /// A pin pass was priced (`acc` -1: a released pass, priced by its reading when it landed;
    /// -2: one whose reading never came, priced by its completion stand-in).
    PinPriced { band: u8, size: u32, price: f64, acc: f64 },
    /// A pin pass was released at its completion and its price deferred to its reading.
    PinDeferred { band: u8, size: u32, stand_in: f64 },
}

/// The chunk walk's PRICING and SIZING (`bp_chunk_tiling`'s, lifted): the wall-aware floor, the
/// in-flight pass's price (the wall-clock retreat into the lethal band, the drain, the band ledger),
/// the pass's step budget and the iteration step with its growth limiter. Pure.
#[must_use]
pub(crate) fn decide_walk_price(i: &WalkPriceInputs) -> WalkPriceDecision {
    use super::{
        chunk_band_retreat, chunk_band_update_with_lane, chunk_step_factor, motion_pass_steps_fixed,
        pin_fast_lane, pin_pass_priceable, wall_shed_now,
    };
    let (interacting, pin_frame, spx, ss) = (i.interacting, i.pin_frame, i.spx, i.ss);
    let (tdr_steps, gpu_iter) = (i.tdr_steps, i.gpu_iter);
    let mut led = i.ledger;
    let mut retreat = None;
    let mut events = Vec::new();
    let ss2 = (ss as u64).saturating_mul(ss as u64);
    // The 256-iteration floor keeps a MEASURED progression from thrashing in slivers, but
    // an UNMEASURED opening dispatch must honour the bootstrap bound at ANY panel size —
    // floor × a large panel exceeds it severalfold (1445×1134 × 256 = 4.195e8 vs the 4e8
    // bootstrap, caught by the "unmeasured budget bounds the FIRST dispatch" selftest; a
    // 4K panel would be 5×). Until a measurement exists the plain division is the bound;
    // the floor applies from the first measured budget onward.
    // ⭐⭐**A WALL-AWARE FLOOR, AND IT CAN ONLY EVER GO LOWER THAN 256.**
    //
    // `budget_step` below is wall-derived (it divides a measured step budget by the pixel
    // count), so the controller already sizes itself from evidence — right up until this
    // floor overrides it. And the floor binds precisely when `budget_step` wants to go
    // BELOW 256, i.e. exactly when the measurement is saying "this region is hostile, go
    // smaller". A fixed ITERATION count cannot bound WALL time, because cost per iteration
    // varies by orders of magnitude between regions.
    //
    // ⛔The 256 rested on a claim that has gone stale: "~256 iterations is ~70 ms even at
    // the worst rate ever measured on this hardware" (see `chunk_band_license`). The
    // 2026-08-22 field loss ran **1024 iterations in 1216 ms** — 1.19 ms/iteration — so 256
    // there is **~304 ms**, not ~70, and a region only **3× more hostile than that** makes
    // the floor ITSELF lethal (900 ms / 256 = 3.5 ms per iteration).
    //
    // So derive it from the worst rate actually measured for this mode (`mode_rate` is
    // kept as a running MINIMUM, i.e. already pessimistic) and take whichever is SMALLER.
    // ⚠`min`, deliberately: in a benign region the computed value is far above 256 and
    // this changes nothing, so the blast radius is confined to regions whose own
    // measurements say the old floor was too big. It can never raise the floor, never
    // enlarge a pass, and never slow a healthy walk.
    let floor = if !i.budget_measured {
        1
    } else {
        let affordable = i.worst_rate.map(|rate| {
            // steps/ms ÷ steps-per-iteration = iterations per ms, times the target wall.
            let per_iter = spx.saturating_mul(ss2).max(1) as f64;
            ((i.tdr_budget_ms * rate) / per_iter) as u32
        });
        affordable.map_or(256, |a| a.clamp(1, 256))
    };
    // ⭐…and never above what B4's admission lets one pass carry (`tdr_steps` is the
    // admission here). The floor is the least a pass may carry and the admission the most;
    // where they cross, the admission wins. Without this an unconfident settle on the
    // RTX 3080 trap walked 256-iteration passes (3.9e8 steps) against an admission of
    // 4.8e7: the pass size was admitted and the floor lifted it straight back.
    let floor = if i.admission == 1 {
        let per_iter = spx.saturating_mul(ss2).max(1);
        floor.min(((tdr_steps / per_iter) as u32).max(1))
    } else {
        floor
    };
    // ---- PRICE-SERIALIZED WALKING, pricing half (design/mode2-chunking.md §12) ----
    // The settled walk holds AT MOST ONE unpriced pass in flight. The in-flight pass
    // accumulates every following frame interval until a frame comes back quicker than
    // CHUNK_DRAIN_DT_MS — with one pass in flight, a quick present is PROOF the queue
    // drained — and that sum is the pass's wall price (conservative: it includes present
    // overhead, which is the safe direction). This is the signal a saturated queue
    // cannot silence; the fatal sessions' timestamps were silent precisely because
    // saturation never let them arm.
    // A PIN is priced and serialized exactly like the settled walk (design/live-zoom-
    // smoothing.md P-A, second round), against the smoothness target rather than the
    // TDR one: its passes are what the viewer's cadence is made of.
    let target_ms = if pin_frame {
        crate::tunables::MOTION_PIN_TARGET_MS
    } else if i.auto_iter {
        i.tdr_budget_ms
    } else {
        i.tdr_explicit_budget_ms
    };
    // ⭐THE DEFERRED FEED (W8, 2026-10-14). A pin pass is released at its completion (one pass in
    // flight, `PIN_INFLIGHT_MAX`), which on the RTX 3080 comes two frames after its dispatch, a
    // frame BEFORE its GPU reading. Priced there by the completion stand-in (the wall time less
    // `PIN_PASS_FRAMES` of pipeline latency, floored at 0), a 12.5 ms pass read 0.0 ms, took the
    // ×16 fast lane (`pin_fast_lane`), and the next pin opened at 8,720 iterations: a 60–115 ms
    // frame per pin at 1x on the (49,3) spiral, on every run whose per-step cost was high (the 1x
    // P-G row's > 50 ms frames: 36–143 a run since `c012f32`, against beta.19's 0–2). Waiting for
    // the reading instead costs a frame per pass, and the pins fell behind (e4000 at 4x: 0–2
    // adoptions in 100 octaves against 35; the 20-octave 4x lag 0.46–0.61 → 1.20–1.30 octaves).
    // So the pass is released at its completion as before and only its PRICE waits: parked in
    // `late`, fed here when the reading that names its dispatch frame lands, and fed its stand-in
    // only if that reading has not come within `PIN_PRICE_WAIT_FRAMES`.
    if let Some((size, band, sent, stand_in)) = led.late {
        let price = if i.late_gpu_ms >= 0.0 {
            Some((i.late_gpu_ms, -1.0))
        } else if i.frame.saturating_sub(sent) >= crate::tunables::PIN_PRICE_WAIT_FRAMES {
            Some((stand_in, -2.0))
        } else {
            None
        };
        if let Some((price, acc)) = price {
            let pin_target = crate::tunables::MOTION_PIN_TARGET_MS;
            led.pass_dt = price;
            chunk_band_update_with_lane(
                &mut led.bands,
                band as usize,
                size,
                price,
                pin_target,
                pin_fast_lane(price, pin_target),
            );
            events.push(WalkPriceEvent::PinPriced { band, size, price, acc });
            led.late = None;
        }
    }
    if !interacting || pin_frame {
        if let Some((size, band, acc, shed)) = led.inflight {
            let acc = acc + i.last_dt_ms.max(0.0);
            // ⭐⭐WALL-CLOCK RETREAT. Until this existed the ledger learned NOTHING about a
            // pass until it drained or hit the wedge backstop — and the backstop is
            // `target × 60`, i.e. ~24 s at a 400 ms target, long after a device would be
            // gone. That is the gap the 2026-08-22 field loss fell through: the release
            // gate below wants a QUICK PRESENT as proof the queue drained, and a queue on
            // its way to losing the device never produces one, so the one pass whose price
            // would shed the licence is exactly the pass that cannot be priced.
            //
            // Accumulated wall is a conservative LOWER BOUND on what this pass has already
            // cost: if `acc` has reached the lethal band the pass has *already* been that
            // expensive, no drain required to know it. So shed the ledger on that alone.
            //
            // ⚠SHED, DO NOT RELEASE. The next dispatch stays blocked until the genuine
            // drain below — "releasing the next dispatch onto a pass that is merely SLOW is
            // how a 10 s single got company on its way to the TDR", and that reasoning is
            // untouched. This only updates what the NEXT pass would be allowed to ask for.
            // ⚠LATCHED (`shed`) so a pass sitting in the band for many frames sheds once
            // rather than re-shedding every frame — the ledger is already at the floor.
            // Under present throttling the wall accumulator measures the
            // COMPOSITOR (see `present_throttle_step`): a shed on it would floor the
            // licences off an idle GPU — the 2026-09-04 field log's exact lines.
            // ⭐The decision itself is `wall_shed_now` — a pure predicate with its own
            // tests, so the rule that runs here and the rule under test cannot drift.
            let throttled = i.present_throttle >= super::PRESENT_THROTTLE_FRAMES;
            let shed = if wall_shed_now(
                shed,
                throttled,
                acc,
                i.tdr_lethal_ms,
            ) {
                chunk_band_retreat(&mut led.bands);
                // What the shed REACHED (W3 gate 2): the next pass opens no lower than the
                // walk's floor, so it is smaller than this one only if the floor is.
                let outcome = super::retreat_outcome(size as u64, floor as u64);
                // The caller records it (`Perf::retreat_last`, `retreat_pending`) and logs
                // the shed, always on (`WalkPriceEvent::LethalShed`).
                retreat = Some(outcome);
                events.push(WalkPriceEvent::LethalShed { size, band, acc, floor, outcome });
                true
            } else {
                shed
            };
            // ⚠The backstop is a WEDGE escape only (a compositor that never goes
            // idle), far beyond any survivable pass — releasing the next dispatch onto
            // a pass that is merely SLOW is how a 10 s single got company on its way
            // to the TDR. The genuine release is the quick frame.
            if throttled && acc > target_ms * 6.0 {
                // Present-throttled: the quick present this gate waits for CANNOT
                // arrive, and `acc` is present-wait, not cost. Release on a short
                // backstop WITHOUT feeding either ledger — a poisoned price would
                // floor the very licences the shed above is protected from. Still
                // serialized-in-effect: at most one pass per backstop interval, so
                // passes cannot stack toward a TDR while throttled.
                led.inflight = None;
            } else if (if pin_frame {
                // A PIN pass is PRICED by its own GPU timestamp reading (a few frames;
                // bounded, then the completion stamp stands in) — never by a quick
                // present: the swap chain hides a pass's cost for two frames, so the
                // quick interval that follows a 100 ms pass proves nothing about it.
                // Dispatch is gated separately, on completions (`Perf::pin_inflight`,
                // `PIN_INFLIGHT_MAX`), and once its completion is seen the completion
                // stamp prices it if no reading has (`pin_pass_priceable`).
                pin_pass_priceable(
                    i.pin_pass_gpu_ms,
                    i.frame.saturating_sub(i.pin_pass_frame),
                    i.pin_inflight,
                )
            } else {
                i.last_dt_ms < crate::tunables::CHUNK_DRAIN_DT_MS
            }) || acc > target_ms * 60.0
            {
                // Drained (or the absolute backstop, so a machine whose presents never
                // come back quick cannot wedge the walk): price the pass, feed both
                // ledgers, release the next dispatch. A pin's price is its GPU timestamp
                // reading when it landed; otherwise the wall time from dispatch to the
                // completion stamp (`Perf::pin_done_at`), or the frame-accumulated `acc`,
                // each less the pipeline latency every pass pays regardless — those two
                // are quantized to the queue's poll cadence and stand in only when the
                // reading never came.
                let price = if pin_frame {
                    let gpu = i.pin_pass_gpu_ms;
                    if gpu >= 0.0 {
                        gpu
                    } else {
                        let done = i.pin_done_at;
                        let sent = i.pin_dispatch_at;
                        let wall = if done > sent && sent > 0 { (done - sent) as f64 / 1000.0 } else { acc };
                        // Real frame periods (`Perf::frame_period_ms`): a completion is
                        // seen at the poll, which comes once a frame INCLUDING the cap's
                        // sleep. `frame_ms` (sleep excluded, ~3 ms at a capped 60 Hz)
                        // left ~27 ms of pipeline latency in every price, and without GPU
                        // timestamps — where this stamp is the only price — every pin pass
                        // read ~25 ms against a 20 ms target and its licence halved.
                        (wall - crate::tunables::PIN_PASS_FRAMES as f64 * i.frame_period_ms)
                            .max(0.0)
                    }
                } else {
                    acc
                };
                // Released at its completion with its reading still to come (the deferred feed,
                // above): park the price, release the pass.
                let defer = pin_frame
                    && i.pin_pass_gpu_ms < 0.0
                    && !i.no_timestamps
                    && i.frame.saturating_sub(i.pin_pass_frame) < crate::tunables::PIN_PRICE_WAIT_FRAMES
                    && acc <= target_ms * 60.0;
                if defer {
                    // One parked at a time: an older one still waiting stands on its stand-in.
                    if let Some((s0, b0, _, p0)) = led.late {
                        led.pass_dt = p0;
                        chunk_band_update_with_lane(
                            &mut led.bands,
                            b0 as usize,
                            s0,
                            p0,
                            target_ms,
                            pin_fast_lane(p0, target_ms),
                        );
                        events.push(WalkPriceEvent::PinPriced { band: b0, size: s0, price: p0, acc: -2.0 });
                    }
                    led.late = Some((size, band, i.pin_pass_frame, price));
                    events.push(WalkPriceEvent::PinDeferred { band, size, stand_in: price });
                } else {
                    led.pass_dt = price;
                    chunk_band_update_with_lane(
                        &mut led.bands,
                        band as usize,
                        size,
                        price,
                        target_ms,
                        if pin_frame { pin_fast_lane(price, target_ms) } else { 2 },
                    );
                    if pin_frame {
                        // Traced by the caller under `tile` (`WalkPriceEvent::PinPriced`).
                        events.push(WalkPriceEvent::PinPriced { band, size, price, acc });
                    }
                }
                led.inflight = None;
            } else {
                led.inflight = Some((size, band, acc, shed));
            }
        }
    } else {
        // Motion abandons the walk (the cursor resets below); a stale in-flight record
        // must not price itself against unrelated frames after the next settle.
        led.inflight = None;
    }
    // Wall-clock pass pricing: a settled pass is the budget scaled by what the previous
    // pass actually cost. MOTION and PIN passes are paced for SMOOTHNESS instead
    // (design/live-zoom-smoothing.md P-A): `tdr_steps` targets TDR safety (400 ms real),
    // and sizing a moving pass to it was the 200–445 ms refresh stall after every jump and
    // mode switch. `motion_pass_steps` sizes the pass from the mode's measured
    // steps-per-ms to `MOTION_PASS_MS`, with the rate-derived bootstrap as the opening
    // guess and the TDR budget as the ceiling — a pin's first pass is no longer "the
    // budget's size".
    let pass_steps = if interacting || pin_frame {
        let nominal = motion_pass_steps_fixed(
            i.motion_rate,
            crate::tunables::MOTION_PASS_MS,
            i.pass_fixed_ms,
            tdr_steps,
            i.tdr_bootstrap_steps,
        );
        if pin_frame {
            // The nominal rate is one model of cost; the wall price of the previous pass
            // is the other, and where the GPU is chain-bound (fast-escape regions) only
            // the wall price is right. Both bound the pass.
            (nominal as f64 * chunk_step_factor(led.pass_dt, target_ms))
                as u64
        } else {
            nominal
        }
    } else {
        (tdr_steps as f64 * chunk_step_factor(led.pass_dt, target_ms))
            as u64
    };
    // Per-frame iteration step that keeps this frame's dispatch inside the budget.
    // ⚠`tdr_steps` — ONE dispatch budget — NOT `tdr_allowed`. The allowance multiplies
    // the budget by `max_tiles` on a settled tiling-eligible frame because TILES spend it
    // as many bounded dispatches; a chunk pass is ONE submission, and sizing it from the
    // allowance dispatched single passes worth exactly 16 budgets — field device loss
    // 2026-08-19 (crash-1787158916-0, RTX 3080, minibrot interior at an explicit 4M,
    // settled with the budget still CLIMBING so the allowance was pinned at
    // TDR_MAX_TILES): 9.600e11-step passes against a 6.000e10 budget, 1136 ms and 912 ms
    // lethal-band readings, and the emergency retreat could not help because the next
    // pass was again 16× the retreated budget. `bla_skip` collapsing to 0 at the interior
    // made nominal cost real cost at exactly that moment. Pinned by the "a settled
    // chunked pass stays inside ONE dispatch budget" selftest.
    let budget_step = ((pass_steps / spx.saturating_mul(ss2).max(1)) as u32)
        .clamp(floor, gpu_iter.max(floor));
    // GROWTH LIMITER (motion / pin passes only): at most double the previous pass. The
    // rate estimate above lags its dispatches by 2–3 frames, so a cheap sliver can prime
    // a pass 20× its size before the reading that would have said otherwise lands —
    // measured as a 257 ms pass right after a 256-iteration one. Doubling meets a cost
    // cliff with one ~2× overshoot, and the wall-clock cut halves the memory next frame.
    let budget_step = if interacting || pin_frame {
        let last = led.motion_step_last;
        let lim = if last == 0 {
            crate::tunables::MOTION_STEP_OPEN
        } else {
            last.saturating_mul(2)
        };
        let s = budget_step.min(lim.max(floor));
        led.motion_step_last = s;
        s
    } else {
        budget_step
    };
    WalkPriceDecision { floor, target_ms, pass_steps, budget_step, ledger: led, retreat, events }
}

/// The chunk walk's progression, one view, carried between frames (`Perf::chunk_sig`,
/// `chunk_walk_view`, `chunk_walk_epoch`, `chunk_running`, `tail_retries`, `chunk_cursor`, `chunk_idx`,
/// `chunk_last_range`, `chunk_walk_end`, `walk_restarts`, `walk_restart_pending`, `chunk_count_walk`,
/// `motion_pass_steps_last`, `chunk_motion_frames`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct WalkState {
    pub(crate) sig: super::WalkSig,
    pub(crate) walk_view: super::WalkSig,
    pub(crate) epoch: u64,
    pub(crate) running: Option<(u64, u32, u32)>,
    pub(crate) tail_retries: u32,
    pub(crate) cursor: u32,
    pub(crate) idx: u32,
    pub(crate) last_range: Option<[u32; 2]>,
    pub(crate) walk_end: u32,
    pub(crate) restarts: u64,
    pub(crate) restart_pending: bool,
    pub(crate) count_walk: u64,
    pub(crate) motion_pass_steps_last: u64,
    pub(crate) motion_frames: u64,
}

/// What [`decide_walk_id`] reads: the walk identity's inputs (`bp_chunk_tiling`), one view, one frame
/// of a chunk-governed compose.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct WalkIdInputs {
    pub(crate) interacting: bool,
    pub(crate) pin_frame: bool,
    /// A settled mode-2 walk (it counts its running pixels).
    pub(crate) fe_walk: bool,
    /// Everything the signature folds: the f64 centre and magnification, the exact position
    /// (`pos_sig`), the settings hash (`view_key.4`), the series seed (`series_seed_bits`), the
    /// supersampling jitter's bits, the ask, the texture and its ss.
    pub(crate) center: [f64; 2],
    pub(crate) magnification: f64,
    pub(crate) pos_sig: u64,
    pub(crate) settings_hash: u64,
    pub(crate) series_seed: u64,
    pub(crate) jitter_bits: [u32; 2],
    pub(crate) gpu_iter: u32,
    pub(crate) resolution: super::Resolution,
    pub(crate) ss: u32,
    /// The pass budget this frame was given (`decide_walk_price`'s `pass_steps`).
    pub(crate) pass_steps: u64,
    /// The view's reference: present, its length, its SA skip, and whether it is a partial.
    pub(crate) has_ref: bool,
    pub(crate) orbit_len: u32,
    pub(crate) sa_skip: u32,
    pub(crate) partial: bool,
    /// The overridable `CHUNK_CHARGE`.
    pub(crate) chunk_charge: u64,
    pub(crate) state: WalkState,
}

/// What [`decide_walk_id`] decides, and the state it writes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct WalkIdDecision {
    pub(crate) state: WalkState,
    /// This frame's walk signature (on a pin frame, the one its captured inputs make).
    pub(crate) sig: super::WalkSig,
    /// The cursor this frame's pass starts at, and the live range of the walk: `[walk_floor, walk_end)`.
    pub(crate) cur: u32,
    pub(crate) walk_floor: u32,
    pub(crate) walk_end: u32,
    /// The walk restarted (the caller drops the pass in flight); and whether its band ledger goes with
    /// it (another view or ask, not a supersampling sample's restart).
    pub(crate) restarted: bool,
    pub(crate) clear_ledger: bool,
    /// A restart that threw away progress: the cursor lost, of what end, and the old signature.
    pub(crate) restart_lost: Option<(u32, u32, super::WalkSig)>,
}

/// The chunk walk's IDENTITY (`bp_chunk_tiling`'s, lifted): the signature that keys the walk (and the
/// jitter-free one that keys its band ledger), the restart and whether it clears the ledger, the
/// counted restart that lost progress, and the live range the walk covers (the SA-skip floor, the
/// partial reference's capped end). Pure.
#[must_use]
pub(crate) fn decide_walk_id(i: &WalkIdInputs) -> WalkIdDecision {
    use super::{chunk_restart_clears_ledger, sa_skip_rescue, usable_sa_skip, WalkSig};
    let (interacting, pin_frame, fe_walk, gpu_iter, ss) = (i.interacting, i.pin_frame, i.fe_walk, i.gpu_iter, i.ss);
    let mut st = i.state;
    let mut restarted = false;
    let mut clear_ledger = false;
    let mut restart_lost = None;
    // View signature: anything that shapes the render restarts the progression.
    // ⚠The REFERENCE LENGTH is part of it. `orbit_len` feeds both the per-pass BLA table
    // (mode 2 rebuilds it every pass) and the `ref_n + 1 >= orbit_len` rebase trigger, so a
    // reference extended mid-settle — the e72/e82/e94 shape, and mode 2 is exactly where
    // that happens — would have a pass resume `ref_n` against a DIFFERENT orbit than the
    // pass that stored it. The install runs later in this frame than this check, so the
    // change is seen one frame late; that is harmless, because a sig change resets the
    // cursor to 0 and the offending pass's state is discarded rather than resumed.
    // Progressive-SSAA jitter is part of the walk's identity: each accumulation sample is a
    // sub-pixel-shifted view, so a new jitter must RESTART the chunk walk from scratch (else
    // the resumable per-pixel state carries the previous sample's positions and every sample
    // comes out identical — no de-speckle). Fold the jitter bits into the sig hash.
    let jbits = (i.jitter_bits[0] as u64) ^ (i.jitter_bits[1] as u64).rotate_left(23);
    // ⚠`pos_sig` (the caller's exact view position: offset from the reference, span
    // exponent + mantissa) is part of it, not just the f64 centre. The f64 centre stops
    // distinguishing nearby views at depth (its ulp is 1/190 of the view width at 2^49;
    // one value covers the whole neighbourhood at 2^800), so a small pan or a re-picked
    // reference at the same f64 point kept the signature, the walk RESUMED its per-pixel
    // state against the moved view, and the old view's escaped pixels stayed on screen
    // under the new one — a translucent copy of another location (field report
    // 2026-09-16). Restarting the walk is output-neutral (bit-identity contract); only
    // its cost is paid, and only when the view really moved.
    // ⚠And WHAT is iterated: the settings hash (`view_key.4` — formula id, custom shader
    // key, Julia and its c, the iterate-affecting colour inputs). It was missing, so a
    // formula switch at an unchanged view resumed the previous formula's finished walk and
    // re-showed its image: measured when custom formulas gained resumable passes — the
    // uitest's Custom step at home (−0.5+0i, Mandelbrot's home too) kept showing the
    // formula applied three steps earlier. The tiled settle's key had the same hole once
    // (`settings_hash`'s own note).
    // ⚠And the SERIES SEED (`series_seed_bits`). The first pass seeds every pixel from the
    // series at `sa.skip` and later passes only resume, so a series refined mid-walk
    // (9.3e78×: skip 5513 → 5535 four frames after the walk began, 2026-10-07) left that
    // walk's pixels seeded from the old one while any later walk seeds from the new: the
    // settled picture then depended on whether the walk happened to start before the
    // refinement — 79 pixels of a converged supersampling average, run to run.
    let view_bits = i.center[0].to_bits()
        ^ i.center[1].to_bits().rotate_left(17)
        ^ i.magnification.to_bits().rotate_left(34)
        ^ (i.orbit_len as u64).rotate_left(51)
        ^ i.pos_sig.rotate_left(41)
        ^ i.settings_hash.rotate_left(29)
        ^ i.series_seed.rotate_left(13);
    let walk_view = WalkSig::of(view_bits, 0, gpu_iter, i.resolution, ss);
    let sig = WalkSig::of(view_bits, jbits, gpu_iter, i.resolution, ss);
    if pin_frame {
        // The pin owns the progression: every sig input above is captured, so the sig is
        // constant by construction, and the interacting reset below — the §10 defect — is
        // bypassed. The freeze verdict cannot flip a pin frame into a reprojection (the
        // pre-step abandons on every input that could: orbit change, caller reproject,
        // drift, pan, panel), so the cursor advance further down is safe — and
        // belt-checked at the commit point after the freeze anyway.
    } else if st.sig != sig || interacting {
        restarted = true;
        // ⭐⭐A NEW SUPERSAMPLING SAMPLE KEEPS THE LEDGER. Each sample of a settled view's
        // progressive supersampling is the same view at a new sub-pixel jitter: the walk
        // must restart (its per-pixel state is position-specific), but the bands' prices
        // are the ORBIT's, the reason the ledger already survives pins and same-sig
        // restarts. Clearing it made every sample re-climb every band from the floor, ×2 a
        // pass: ~90 of a sample's ~158 frames at 9.3e78× were 256..65536-iteration passes
        // at one a frame (2026-10-07). Output-neutral: the window size never changes a
        // pixel (the chunked iterate's bit-identity contract).
        if chunk_restart_clears_ledger(
            st.sig,
            st.walk_view,
            sig,
            walk_view,
            interacting,
        ) {
            // Another view/ask: its prices describe another walk entirely.
            // ⭐Not during INTERACTION: successive pinned refreshes of a glide are the
            // same orbit a few hundredths of an octave apart, and the band ledger is a
            // property of the ORBIT's iteration bands, not of the view. Clearing it per
            // pin made every pin re-climb from the floor in every band (~50 passes,
            // 0.8 s) — at 4.0× that is past the 2-octave drift abandon, so no pin ever
            // adopted (traced: 14 started, 7 adopted, 6 abandoned for drift, the frozen
            // frame aged 13 octaves on screen). The ledger is cleared when the reference
            // POINT changes instead (`install_recompute`).
            // (The caller zeroes the band licences and the last price: `clear_ledger`.)
            clear_ledger = true;
        }
        // ⭐The band ledger SURVIVES a same-sig restart on purpose: every hold/refresh
        // cycle restarts the walk, and re-crossing the wrap-storm band with amnesia is
        // how the cliff got re-rolled dozens of times in one session.
        // W4's A4: a restart that throws a part-way walk away is the iteration-starvation
        // shape (a resolution rung re-chosen every few frames restarted the walk every few
        // frames, and 36% of a dive was blank). Counted and recorded, pending until the
        // frame record is written.
        if super::frame_plan::walk_restart_discards(st.cursor, st.walk_end) {
            st.restart_pending = true;
            st.restarts = st.restarts.wrapping_add(1);
            // Traced by the caller under `tile`: what was lost, and the signatures.
            restart_lost = Some((st.cursor, st.walk_end, st.sig));
        }
        st.sig = sig;
        st.walk_view = walk_view;
        st.epoch = st.epoch.wrapping_add(1).max(1);
        st.running = None;
        st.tail_retries = 0;
        st.cursor = 0;
        st.idx = 0;
        st.last_range = None;
        // (The caller drops the pass in flight: `restarted`.)
        // A restarted walk's escape bands are a different progression's data.
    }
    let cur = st.cursor;
    // A walk is never behind its own reading: a cursor that went back was reset somewhere
    // this stage did not see, so its bound describes another walk — drop it.
    if st.running.is_some_and(|(_, at, _)| cur < at) {
        st.running = None;
    }
    // A settled mode-2 walk counts its running pixels (`walk_charged_px` sizes by them).
    if fe_walk && !interacting && !pin_frame && i.chunk_charge == 1 {
        st.count_walk = st.epoch;
    }
    if interacting && !pin_frame {
        // The pass budget a MOVING frame was actually given, which is what decides whether
        // it can walk far enough to commit a pixel at all. Recorded BEFORE the growth
        // limiter, because the limiter bounds how fast the step may climb, not how much
        // work the frame is allowed — and `visible_res_target` must size pixels against
        // the allowance, not against a step still on its way up to it. Pins are excluded:
        // their small per-dispatch band accumulates across passes, so one pin step says
        // nothing about whether a single frame can stand on its own.
        st.motion_pass_steps_last = i.pass_steps;
    }
    // ⭐⭐THE SA-SKIP FLOOR. When a reference is SA-seeded, the `start_iter==0` pass places
    // EVERY pixel at `iter = sa_skip` in one dispatch — so iterations [0, sa_skip) are
    // covered for free by that single pass. Marching the cursor from 0 through them in
    // floor windows dispatches thousands of full-frame NO-OPS (each pixel loads its state,
    // sees iter=sa_skip ≥ the window end, and stores it back unchanged) — every one paying
    // a full-resolution chunk+resolve pass and a present cycle. At the period-3 bulb root
    // sa_skip≈7.45M and the floor window is 256, so ~29,000 dead passes at ~1 s each: the
    // "still spinning" crawl (crash-1789092955 / the 5.1e10× field report). The walk must
    // cover only the LIVE range [sa_skip, walk_end): the first pass keeps its START at 0
    // (so the shader SA-seeds), but jumps its END past the skip; windows past it resume
    // from state as before.
    //
    // `walk_end` mirrors `build_mandelbrot_params`' `shader_iter` cap. A PARTIAL reference
    // caps the shader at orbit_len-1 (it cannot iterate past the orbit it holds), so
    // [shader_iter, gpu_iter) is a SECOND dead range; targeting the uncapped `gpu_iter`
    // marched that too. `sa_skip_rescue` keeps the two in lockstep with the params builder.
    // ⚠MOTION frames are EXEMPT — `(0, gpu_iter)` keeps them bit-for-bit on the historical
    // path. They reset the cursor to 0 every frame (the sig/interacting reset above), so they
    // never accumulate the dead prefix: there is no crawl to fix, and a motion preview is
    // meant to be `[0, step]`. Confirmed by an A/B: the `--motiontest` A2 adoption flake rate
    // is identical with and without this fix (baseline failed MORE often than the fix under
    // the same load). The floor/cap is strictly a SETTLED-walk fix; pins inherit it only once
    // they settle (`interacting` false).
    let (walk_floor, walk_end) = if interacting || !i.has_ref {
        (0u32, gpu_iter)
    } else {
        let raw_skip = i.sa_skip;
        let capped = if i.partial {
            gpu_iter.min(i.orbit_len.saturating_sub(1))
        } else {
            gpu_iter
        };
        let we = sa_skip_rescue(capped, raw_skip, i.orbit_len);
        (usable_sa_skip(raw_skip, we), we)
    };
    // Where this walk ends, for the next restart to judge whether it lost anything.
    st.walk_end = walk_end;
    // Gate observability (design/mode2-chunking.md §11): a chunk-eligible frame built
    // during interaction is the regime the motion-presentation assertions cover —
    // `--motiontest` fails a run that never produced any (anti-vacuity).
    if interacting {
        st.motion_frames = st.motion_frames.wrapping_add(1);
    }
    WalkIdDecision { state: st, sig, cur, walk_floor, walk_end, restarted, clear_ledger, restart_lost }
}

/// The chunk walk's dispatch bookkeeping, one view, carried between frames (`Perf::pin_inflight`,
/// `pin_reg_pending`, `pin_dispatch_at`, `pin_escapes`, `pin_pass_steps`, `pin_pass_frame`,
/// `pin_pass_gpu_ms`, `chunk_last_range`, `chunk_inflight`, `chunk_idx`, `chunk_cursor`, `chunk_pending`,
/// `chunk_charged_px`, `fe_steps_last`, `tail_retries`, `probe_nonce`, `norm_complete_submit`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct DispatchState {
    pub(crate) pin_inflight: u32,
    pub(crate) pin_reg_pending: u8,
    pub(crate) pin_dispatch_at: u64,
    pub(crate) pin_escapes: u32,
    pub(crate) pin_pass_steps: u64,
    pub(crate) pin_pass_frame: u64,
    pub(crate) pin_pass_gpu_ms: f64,
    pub(crate) last_range: Option<[u32; 2]>,
    pub(crate) inflight: Option<(u32, u8, f64, bool)>,
    pub(crate) idx: u32,
    pub(crate) cursor: u32,
    pub(crate) pending: bool,
    pub(crate) charged_px: u64,
    pub(crate) fe_steps_last: u64,
    pub(crate) tail_retries: u32,
    pub(crate) probe_nonce: u32,
    pub(crate) norm_complete_submit: bool,
}

/// What [`decide_walk_dispatch`] reads: the chunk walk's dispatch inputs (`bp_chunk_tiling`), one view,
/// one frame of a chunk-governed compose.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct WalkDispatchInputs {
    pub(crate) interacting: bool,
    pub(crate) pin_frame: bool,
    pub(crate) fe_walk: bool,
    /// The walk (`decide_walk_id`): its cursor and live range.
    pub(crate) cur: u32,
    pub(crate) walk_floor: u32,
    pub(crate) walk_end: u32,
    /// The pass's sizing (`decide_walk_price`): the floor, the step budget and the iteration step.
    pub(crate) floor: u32,
    pub(crate) pass_steps: u64,
    pub(crate) budget_step: u32,
    pub(crate) gpu_iter: u32,
    pub(crate) spx: u64,
    pub(crate) ss: u32,
    /// This frame, the clock (µs, `app_micros`), and the last frame interval.
    pub(crate) frame: u64,
    pub(crate) now_us: u64,
    pub(crate) last_dt_ms: f64,
    /// The worst rate measured in this mode; the band licences after this frame's pricing; the
    /// reference's length; the walk's epoch and its running-pixel reading; the calibration's occupancy
    /// knee (`calibration::knee_px`).
    pub(crate) worst_rate: Option<f64>,
    pub(crate) bands: [u32; crate::tunables::CHUNK_BANDS],
    pub(crate) orbit_len: u32,
    pub(crate) epoch: u64,
    pub(crate) running: Option<(u64, u32, u32)>,
    pub(crate) knee_px: u64,
    /// Supersampling runs on the view; the signature of the walk's last complete reading, and of the
    /// frame being submitted (the tail's read retry).
    pub(crate) accum_active: bool,
    pub(crate) walk_read_sig: u64,
    pub(crate) norm_sig_submit: u64,
    /// The overridable tunables it reads.
    pub(crate) pin_timeout_scaled: u64,
    pub(crate) chunk_charge: u64,
    pub(crate) chunk_shuffle: u64,
    pub(crate) state: DispatchState,
}

/// What [`decide_walk_dispatch`] decides, and the state it writes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct WalkDispatchDecision {
    /// The iteration range this frame dispatches and the walk's pass index; a real pass is planned.
    pub(crate) chunk_range: Option<[u32; 2]>,
    pub(crate) chunk_idx: u32,
    pub(crate) pass: bool,
    pub(crate) state: DispatchState,
    /// A pin pass went out: the caller stamps its dispatch time.
    pub(crate) pin_dispatched: bool,
    pub(crate) events: Vec<DispatchEvent>,
}

/// What the dispatch did, for the caller's log and trace.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum DispatchEvent {
    /// The pin gate escaped on the wall clock (`n`: the view's escapes so far).
    PinEscape { age_us: u64, limit_us: u64, registered: bool, n: u32 },
    /// The pass was charged for the pixels still running, not the frame.
    Charge { running: u32, charged: u64, all_px: u64, from: u32, to: u32 },
}

/// The chunk walk's DISPATCH (`bp_chunk_tiling`'s, lifted): the serialized hold (the settled walk's
/// one unpriced pass; the pin gate, and its wall-clock escape), the regional licence, the
/// running-pixel charge, the step and the window's end, the cursor's advance and the pin pass's
/// bookkeeping, the finished walk's tail and its read retry, and whether this pass's reading will
/// describe the picture on screen. Pure.
#[must_use]
pub(crate) fn decide_walk_dispatch(i: &WalkDispatchInputs) -> WalkDispatchDecision {
    use super::{chunk_band_end, chunk_band_license, chunk_band_of, pin_band_license, tail_read_retry, walk_charged_px};
    let (interacting, pin_frame, fe_walk) = (i.interacting, i.pin_frame, i.fe_walk);
    let (cur, walk_floor, walk_end) = (i.cur, i.walk_floor, i.walk_end);
    let (floor, pass_steps, budget_step, gpu_iter, spx) = (i.floor, i.pass_steps, i.budget_step, i.gpu_iter, i.spx);
    let ss2 = (i.ss as u64).saturating_mul(i.ss as u64);
    let mut st = i.state;
    let chunk_range: Option<[u32; 2]>;
    let chunk_idx: u32;
    let mut chunk_pass = false;
    let mut pin_dispatched = false;
    let mut events = Vec::new();
    if cur < walk_end {
        // ---- PRICE-SERIALIZED WALKING, dispatch half ----
        // A settled pass may only launch when the previous one has been PRICED
        // (chunk_inflight is None). While one is in flight, hold the LAST DISPATCHED
        // range — the (key, tile, chunk, probe) triple stays unchanged, the GPU dedupes
        // it, zero work runs, and the drain frame this produces is the very measurement
        // that releases the next pass. Motion frames keep their per-frame cadence
        // (the cursor resets each frame anyway) and PIN frames their own (§10-§11).
        let serialized_hold = if pin_frame {
            // A pin holds while `PIN_INFLIGHT_MAX` of its passes are still in the queue
            // (completion-counted), not while one is unpriced. A completion that never
            // comes (a harness with no `update`, a lost device) times out on the wall
            // clock: the gate must never depend on a signal that can fail to arrive.
            let age_us = i.now_us.saturating_sub(st.pin_dispatch_at);
            // `registered`: `update` has armed this pass's completion callback (the
            // registrations it owes are drained there), so the completion WILL come and
            // the wait scales with the frames (`pin_gate_timeout_us`, U3/U47).
            let registered = st.pin_reg_pending == 0;
            let limit_us = super::pin_gate_timeout_us(
                registered,
                i.last_dt_ms,
                i.pin_timeout_scaled == 1,
            );
            if st.pin_inflight > 0 && age_us > limit_us {
                // ⚠An ESCAPE, counted and (the first per view) logged: the next pin pass
                // goes out with this one possibly still running.
                st.pin_escapes = st.pin_escapes.saturating_add(1);
                // The caller logs the first per view, always on, and traces the rest under `tile`.
                events.push(DispatchEvent::PinEscape { age_us, limit_us, registered, n: st.pin_escapes });
                st.pin_inflight = 0;
            }
            st.pin_inflight >= crate::tunables::PIN_INFLIGHT_MAX
                && st.last_range.is_some()
        } else {
            !interacting
                && st.inflight.is_some()
                && st.last_range.is_some()
        };
        if serialized_hold {
            chunk_range = st.last_range;
            chunk_idx = st.idx;
            // Still mid-flight: keep repaints coming so the drain frames actually happen.
            st.pending = true;
        } else {
            // The REGIONAL license (see chunk_band_license): the walk may spend the
            // budget's sizing only where prices have earned it; new territory starts at
            // half its predecessor's license, so a cost cliff is met with at most ONE
            // modestly-sized unpriced pass — and serialization guarantees its price is
            // seen before anything else launches.
            // The first settling pass has cur=0 but does its real work at `walk_floor` (the
            // SA seed); every later pass has cur ≥ walk_floor. `real_lo` is where this
            // window's real iterations begin, so the band, the step count and the price all
            // describe the work actually done, not the free prefix folded into the first
            // dispatch. (Motion frames carry `walk_floor = 0`, so `real_lo = cur` there.)
            let real_lo = cur.max(walk_floor);
            let band = chunk_band_of(real_lo) as u8;
            // EVERY pass takes the regional licence — settled, pinned, and the moving
            // frame that OPENS a pin. A band's price is earned by its own passes, and a
            // walk crossing into a hot band meets it with one floor-sized pass, not a
            // licence-sized one. The opening pass used to be exempt ("motion frames
            // restart each frame"), and with the nominal-rate hint inflated by a deduped
            // reading it went out at the TDR budget's size: 400–440 ms, once per pin, at
            // every zoom rate (build 3222). The ledger persists across the pins of a
            // glide, so an opening pass in a priced band is already a full-sized one.
            // A PIN's floor is sized from ITS target, the way the settled floor is sized
            // from the TDR target: the worst rate ever measured in this mode, times
            // `MOTION_PIN_TARGET_MS`, over this frame's pixels — capped at 256. In a
            // benign regime that is ≥ 256 (no effect); in the rebase storm at the end of
            // an ESCAPED reference (cur ≈ orbit_len, every iteration a rebase, measured
            // ~2e6 nominal steps/ms) it lets the ledger's cliff rule shrink a pass to a
            // few dozen iterations instead of stopping at 256 ≈ 200 ms. Never below the
            // wall-aware TDR floor's own minimum of 1, never above 256: a pin crawling
            // at a handful of iterations per frame in a benign region never adopts, and
            // the frozen frame then ages on screen without bound (measured 13 octaves,
            // 11,000×, before a pin floor existed).
            let pin_floor = if pin_frame {
                let per_iter = spx.saturating_mul(ss2).max(1) as f64;
                i.worst_rate
                    .map(|rate| {
                        ((crate::tunables::MOTION_PIN_TARGET_MS * rate) / per_iter) as u32
                    })
                    .map_or(crate::tunables::MOTION_STEP_OPEN / 2, |f| {
                        f.clamp(1, crate::tunables::MOTION_STEP_OPEN / 2)
                    })
            } else {
                floor.max(256)
            };
            let licence = if pin_frame {
                // A pin inherits past the first-wrap storm (see `pin_band_license`);
                // the settled walk opens every cold band at the floor.
                pin_band_license(
                    &i.bands,
                    band as usize,
                    pin_floor,
                    real_lo,
                    i.orbit_len,
                )
            } else {
                chunk_band_license(&i.bands, band as usize, pin_floor)
            };
            // ⭐⭐CHARGE THE PIXELS STILL RUNNING (settled mode-2 walk). `budget_step` divides the
            // pass budget by EVERY texel, but a walk's later passes iterate only the texels
            // still running: at 9.3e78× the tail passes were ~5 ms of a 400 ms-model budget
            // (2026-10-07), one a frame. This walk's own count (`chunk_running`) is an UPPER
            // bound for every later pass (a pixel never restarts), and below the occupancy
            // knee a pass is charged the knee, as the calibration ceiling charges it: the
            // worst-case model is unchanged, only the pixel count is honest. Unknown (no
            // reading of THIS walk yet, no knee) charges the whole frame, as before. The band
            // licence below still caps every pass by its own band's prices.
            let all_px = spx.saturating_mul(ss2);
            let charged = if fe_walk && !interacting && !pin_frame && i.chunk_charge == 1 {
                walk_charged_px(
                    all_px,
                    i.knee_px,
                    i.epoch,
                    real_lo,
                    i.running,
                )
            } else {
                all_px
            };
            let budget_step = if charged < all_px {
                st.charged_px = charged;
                let charged_step =
                    ((pass_steps / charged.max(1)) as u32).clamp(floor, gpu_iter.max(floor));
                // Traced by the caller under `tile`.
                events.push(DispatchEvent::Charge {
                    running: i.running.map_or(0, |(_, _, n)| n),
                    charged,
                    all_px,
                    from: budget_step,
                    to: charged_step,
                });
                charged_step
            } else {
                budget_step
            };
            let step = budget_step.min(licence);
            let step = if i.chunk_shuffle == 1 && !interacting && !pin_frame {
                // Test hook (`CHUNK_SHUFFLE`): an uneven window, the same every run.
                let h = (st.idx as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 60;
                ((step as u64 * (16 + h) / 32) as u32).max(1)
            } else {
                step
            };
            let step = if pin_frame { step.max(pin_floor.min(gpu_iter.max(1))) } else { step };
            let end = real_lo.saturating_add(step).min(walk_end);
            // A PIN pass stops at the end of the band it started in. The licence is the
            // START band's, and a pass that runs on into unvisited bands carries it there
            // — traced: a 21,480-iteration pass licensed in [8k,16k) ran through the
            // orbit's rebase storm at cur ≈ orbit_len (10–70× the cold rate) and cost
            // 380 ms. Entering each band with that band's own licence (the floor when
            // unvisited) is what the ledger was built for; it costs a pin at most one
            // extra pass per band. Output is unchanged by pass boundaries (the chunked
            // iterate's bit-identity contract).
            let end = if pin_frame { end.min(chunk_band_end(real_lo)).max(real_lo + 1).min(walk_end) } else { end };
            // START stays `cur` (0 on the first pass) so the shader takes its SA-seed
            // branch; the live work is [real_lo, end).
            chunk_range = Some([cur, end]);
            chunk_idx = st.idx;
            // Moving frames restart at 0 next frame (the sig reset above); settled frames
            // and PIN frames advance until the full count is honoured. A pin keeps
            // `chunk_pending` up through its completing pass, so repaints keep coming for
            // the adoption frame even if the user's input pauses inside the settle window.
            if !interacting || pin_frame {
                st.cursor = end;
                st.idx = chunk_idx.wrapping_add(1);
                st.pending = end < walk_end || pin_frame;
                // THE COMPLETION EVENT: the walk's whole-ask escape range feeds the
                // palette window here, once. The completing pass's own band is still in
                // flight (readings lag) — it folds into the NEXT walk's accumulator,
                // which at the same view is a head start, not pollution. A restarted
                // walk that never completed feeds nothing (its accumulator is cleared
                // with the reset above).
                // (The walk-COMPLETION feed that used to live here is gone: readings are
                // whole-frame now, so they feed the EMA directly at the drain. The event
                // it waited for did not reliably fire — see the drain's note.)
            }
            st.last_range = chunk_range;
            if !interacting || pin_frame {
                st.inflight =
                    Some((end.saturating_sub(real_lo), band, 0.0, false));
            }
            if pin_frame {
                // One pin pass in the queue: the completion callback (armed next frame,
                // once eframe has submitted this one) is what releases the next.
                st.pin_inflight = st.pin_inflight.saturating_add(1);
                st.pin_reg_pending = st.pin_reg_pending.saturating_add(1);
                pin_dispatched = true; // the caller stamps it on the clock (`Perf::pin_dispatch_at`)
                st.pin_pass_steps =
                    spx.saturating_mul(ss2).saturating_mul((end - real_lo).max(1) as u64);
                st.pin_pass_frame = i.frame;
                st.pin_pass_gpu_ms = -1.0;
            }
            // This frame runs a real bounded pass; pair the measurement with ITS cost —
            // the iterations past the SA seed, not the free [0, sa_skip) prefix — over the
            // texels it was CHARGED for (the whole frame unless the running count applied).
            st.fe_steps_last = charged.saturating_mul((end - real_lo).max(1) as u64);
            chunk_pass = true;
        }
    } else {
        // Progression complete: emit the empty tail range — the (key, tile, chunk) triple
        // stops changing after one cheap pass-through frame, and the view is served from
        // the texture like any settled one. The tail sits at `walk_end` (the capped ask),
        // not the raw `gpu_iter`, so a partial reference completes instead of marching the
        // dead [shader_iter, gpu_iter) range forever.
        chunk_range = Some([walk_end, walk_end]);
        chunk_idx = st.idx;
        // A finished PIN waits for the reading that proves its picture (`pin_verdict`).
        // The counter readback arms only when idle, so the completing pass may have gone
        // out unarmed; re-dispatching the (empty, cheap) tail under a fresh probe nonce
        // keeps readings coming until one carries the finished cursor.
        // ⭐So does a finished SETTLED walk, until its complete reading lands: the iteration
        // climb and the colour range both decide on it, and supersampling waits for both
        // (`accum_inputs_pending`). Missed, nothing else dispatched on a still view, and the
        // run that began anyway was what retried it — restarting at every reading it brought.
        let retry = tail_read_retry(
            interacting,
            i.accum_active,
            i.walk_read_sig,
            i.norm_sig_submit,
            st.tail_retries,
        );
        if pin_frame || retry {
            st.probe_nonce = st.probe_nonce.wrapping_add(1);
        }
        if retry && !pin_frame {
            st.tail_retries += 1;
        }
    }
    // ⭐Does this pass's escape-range reading describe the picture that will be ON SCREEN?
    // The resolve commits the range of everything escaped SO FAR, so a pass that stops
    // short of the walk's end reports a top no higher than the walk has got. That is the
    // picture when it is what gets displayed — a moving view's one-pass preview, which is
    // `[0, step]` by design — but not for a pinned refresh (the screen holds the last
    // complete frame until the walk lands; the pin-start below re-marks its opening pass)
    // nor for a settled walk, whose partials fill in toward a range they have not reached.
    // Fed as if whole, a pinned walk's readings made the palette window a SAWTOOTH,
    // measured 2026-09-19 on a 2e13 autopilot dive: tops of 1,758 → 3,150 → 6,582 →
    // 10,238 and back to 1,758, about 2.5 times a second, which the EMA and the glide
    // turned into a rhythmic "breathing" of every colour on screen.
    st.norm_complete_submit = (interacting && !pin_frame)
        || chunk_range.is_none_or(|[_, end]| end >= walk_end);
    WalkDispatchDecision { chunk_range, chunk_idx, pass: chunk_pass, state: st, pin_dispatched, events }
}

/// The recording's format version: bumped whenever an inputs or decision struct changes shape, and a
/// replay refuses a recording of another (it would compare the wrong things). 2: `MovingInputs`
/// carries `pass_fixed_ms`. (The `regime` context lines are additive: a replay that cannot read them
/// counts them unreadable and fails, which is the safe direction.)
pub(crate) const DECIDE_SCHEMA: u32 = 2;

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
                // The card it ran on, once the calibration has named it (a GUI run always has by its
                // first decision): a fixture set's recordings must span more than one.
                "adapter": crate::calibration::adapter(),
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

/// The §6.7 regime facts of one frame record, written beside the decisions as a `"kind":"regime"`
/// line: the fields `soak::RegimeAcc` folds, so a recording says whether its run ENTERED the
/// escaped-reference regime — judged by the soak's own predicate, read from the frame record, never
/// from a timing instrument. Context, not a decision: nothing is recomputed from it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct RegimeFacts {
    pub(crate) frame: u64,
    pub(crate) t_ms: u64,
    pub(crate) view: u8,
    pub(crate) has_ref: bool,
    pub(crate) ref_partial: bool,
    pub(crate) ref_len: u32,
    pub(crate) gpu_iter: u32,
    pub(crate) bla_on: bool,
    pub(crate) ctr_new: bool,
    pub(crate) ctr_rebase: u64,
    pub(crate) ctr_bla_skip: u64,
    pub(crate) dispatched: bool,
    pub(crate) chunked: bool,
    pub(crate) fe_budget: u64,
}

impl RegimeFacts {
    pub(crate) fn of(r: &crate::diag::frame_record::FrameRecord) -> RegimeFacts {
        RegimeFacts {
            frame: r.frame,
            t_ms: r.t_ms,
            view: r.view,
            has_ref: r.has_ref,
            ref_partial: r.ref_partial,
            ref_len: r.ref_len,
            gpu_iter: r.gpu_iter,
            bla_on: r.bla_on,
            ctr_new: r.ctr_new,
            ctr_rebase: r.ctr_rebase,
            ctr_bla_skip: r.ctr_bla_skip,
            dispatched: r.dispatched,
            chunked: r.chunked,
            fe_budget: r.fe_budget,
        }
    }

    /// Back to a frame record, every other field at its default, for `RegimeAcc::add`.
    pub(crate) fn record(&self) -> crate::diag::frame_record::FrameRecord {
        crate::diag::frame_record::FrameRecord {
            kind: crate::diag::frame_record::kind::FRAME,
            frame: self.frame,
            t_ms: self.t_ms,
            view: self.view,
            has_ref: self.has_ref,
            ref_partial: self.ref_partial,
            ref_len: self.ref_len,
            gpu_iter: self.gpu_iter,
            bla_on: self.bla_on,
            ctr_new: self.ctr_new,
            ctr_rebase: self.ctr_rebase,
            ctr_bla_skip: self.ctr_bla_skip,
            dispatched: self.dispatched,
            chunked: self.chunked,
            fe_budget: self.fe_budget,
            ..Default::default()
        }
    }
}

/// Write a frame record's regime facts, if `FRACTADYNE_RECORD_DECIDE` is recording (frames only).
pub(crate) fn record_regime(r: &crate::diag::frame_record::FrameRecord) {
    use std::io::Write;
    if r.kind != crate::diag::frame_record::kind::FRAME {
        return;
    }
    let Some(w) = recorder() else { return };
    let line = serde_json::json!({ "kind": "regime", "facts": RegimeFacts::of(r) });
    if let Ok(mut w) = w.lock() {
        let _ = writeln!(w, "{line}");
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
    /// Its regime context lines, and the soak predicate's verdict over them (§6.7): whether the run
    /// ENTERED the escaped-reference regime, and the soak's own line saying how.
    pub(crate) regime_lines: u64,
    pub(crate) regime_entered: bool,
    pub(crate) regime_line: Option<String>,
    /// The adapters its header lines name (a file may hold several recordings, a header each).
    pub(crate) adapters: Vec<String>,
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
    let mut regime = crate::soak::RegimeAcc::default();
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
                if let Some(a) = v.get("adapter").and_then(|a| a.as_str()) {
                    if !rep.adapters.iter().any(|x| x == a) {
                        rep.adapters.push(a.to_string());
                    }
                }
                rep.header = Some(v);
            }
            Some("regime") => match serde_json::from_value::<RegimeFacts>(v.get("facts").cloned().unwrap_or_default()) {
                Ok(f) => {
                    rep.regime_lines += 1;
                    regime.add(&f.record());
                }
                Err(_) => rep.unreadable += 1,
            },
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
                    "iter" => {
                        let Ok(inputs) = serde_json::from_value::<IterInputs>(
                            v.get("inputs").cloned().unwrap_or(serde_json::Value::Null),
                        ) else {
                            rep.unreadable += 1;
                            continue;
                        };
                        serde_json::to_value(decide_iter(&inputs)).unwrap_or(serde_json::Value::Null)
                    }
                    "probe" => {
                        let Ok(inputs) = serde_json::from_value::<ProbeInputs>(
                            v.get("inputs").cloned().unwrap_or(serde_json::Value::Null),
                        ) else {
                            rep.unreadable += 1;
                            continue;
                        };
                        serde_json::to_value(decide_probe(&inputs)).unwrap_or(serde_json::Value::Null)
                    }
                    "walk-dispatch" => {
                        let Ok(inputs) = serde_json::from_value::<WalkDispatchInputs>(
                            v.get("inputs").cloned().unwrap_or(serde_json::Value::Null),
                        ) else {
                            rep.unreadable += 1;
                            continue;
                        };
                        if inputs.interacting {
                            rep.moving += 1;
                        }
                        serde_json::to_value(decide_walk_dispatch(&inputs)).unwrap_or(serde_json::Value::Null)
                    }
                    "walk-id" => {
                        let Ok(inputs) = serde_json::from_value::<WalkIdInputs>(
                            v.get("inputs").cloned().unwrap_or(serde_json::Value::Null),
                        ) else {
                            rep.unreadable += 1;
                            continue;
                        };
                        if inputs.interacting {
                            rep.moving += 1;
                        }
                        serde_json::to_value(decide_walk_id(&inputs)).unwrap_or(serde_json::Value::Null)
                    }
                    "walk-price" => {
                        let Ok(inputs) = serde_json::from_value::<WalkPriceInputs>(
                            v.get("inputs").cloned().unwrap_or(serde_json::Value::Null),
                        ) else {
                            rep.unreadable += 1;
                            continue;
                        };
                        if inputs.interacting {
                            rep.moving += 1;
                        }
                        serde_json::to_value(decide_walk_price(&inputs)).unwrap_or(serde_json::Value::Null)
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
    if rep.regime_lines > 0 {
        let r = regime.report();
        rep.regime_entered = r.entered;
        rep.regime_line = Some(r.line);
    }
    rep
}

/// One coverage term of a fixture set (§7.5): a branch of a lifted decision, and how many of the set's
/// recorded decisions took it.
#[derive(Debug)]
pub(crate) struct Coverage {
    pub(crate) stage: &'static str,
    pub(crate) term: &'static str,
    pub(crate) hits: u64,
}

/// Tally one recorded decision of a stage against that stage's terms (an unreadable one is the
/// replay's to count, not this).
fn tally<I: serde::de::DeserializeOwned, D: serde::de::DeserializeOwned>(
    v: &serde_json::Value,
    terms: &[(&'static str, fn(&I, &D) -> bool)],
    hits: &mut [u64],
) {
    let (Ok(i), Ok(d)) = (
        serde_json::from_value::<I>(v["inputs"].clone()),
        serde_json::from_value::<D>(v["decision"].clone()),
    ) else {
        return;
    };
    for (h, (_, f)) in hits.iter_mut().zip(terms) {
        *h += f(&i, &d) as u64;
    }
}

fn collect(out: &mut Vec<Coverage>, stage: &'static str, terms: impl Iterator<Item = &'static str>, hits: &[u64]) {
    out.extend(terms.zip(hits).map(|(term, &hits)| Coverage { stage, term, hits }));
}

/// A settled view the climb probe would consider: nothing else dispatching, the budget unconverged.
fn probe_idle(i: &ProbeInputs) -> bool {
    !i.interacting
        && !i.offscreen
        && !i.will_reproject
        && !i.tour_playing
        && !i.key_changed
        && !i.tile
        && i.chunk_range.is_none()
        && !i.budget_ok
}

/// §7.5's coverage term over a fixture set: for every branch of every lifted decision, how many of
/// the recordings' decisions took it. Across the set every one must be replayed at least once — a
/// fixture set satisfied by one regime grades the controller only where it is easy. The one place
/// the terms are stated: `cargo test` (`every_recorded_session_replays_to_its_decisions`) and
/// `--selftest replay` both read them here.
pub(crate) fn fixture_coverage<'a>(texts: impl IntoIterator<Item = &'a str>) -> Vec<Coverage> {
    // §7.5's coverage term: across the set, every branch of the decision is replayed at least
    // once — a fixture set satisfied by one regime grades the controller only where it is easy.
    let branches: [(&'static str, fn(&MovingInputs, &MovingDecision) -> bool); 9] = [
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
    let present: [(&'static str, fn(&PresentInputs, &PresentDecision) -> bool); 6] = [
        ("a settled compose held the display", |_, d| d.display_hold && !d.pin_gate),
        (
            "a settled compose refused an unverified snapshot",
            |i, d| d.display_hold && !d.pin_gate && !i.hold_active && !i.live_verified,
        ),
        ("a pin held the display", |_, d| d.pin_gate),
        ("a snapshot was taken", |_, d| d.hold_copy),
        ("the settled-cost probe armed", |_, d| d.aa_probe.is_some()),
        ("the colour-pass filter widened", |_, d| d.aa_filter > 1),
    ];
    let climb_terms: [(&'static str, fn(&IterInputs, &IterDecision) -> bool); 11] = [
        ("a settled reading was judged", |i, d| !i.interacting && d.reached_ask && !d.stale),
        ("motion reset the climb", |i, _| i.interacting),
        ("a stale reading was discarded", |_, d| d.stale),
        ("the boost was raised", |_, d| d.events.iter().any(|e| matches!(e, IterEvent::Raised { .. }))),
        ("the boost was seeded", |_, d| d.events.iter().any(|e| matches!(e, IterEvent::Seeded { .. }))),
        ("the verdict was reached", |i, d| d.state.verdict.is_some() && i.state.verdict.is_none()),
        ("an explicit count", |i, _| !i.auto_iter),
        ("the climb plateaued", |_, d| d.events.iter().any(|e| matches!(e, IterEvent::Plateau { .. }))),
        ("the appetite re-pick", |_, d| d.events.iter().any(|e| matches!(e, IterEvent::Repick { .. }))),
        ("the climb was exhausted", |_, d| d.events.iter().any(|e| matches!(e, IterEvent::Exhausted { .. }))),
        ("motion ended a flat run", |_, d| d.events.iter().any(|e| matches!(e, IterEvent::FlatRunEnded { .. }))),
    ];
    let probe_terms: [(&'static str, fn(&ProbeInputs, &ProbeDecision) -> bool); 3] = [
        ("the probe fired", |_, d| d.fired),
        ("an idle unconverged view waited for the pacing", |i, _| {
            probe_idle(i) && i.frame.saturating_sub(i.dispatch_frame) < 3
        }),
        ("an idle unconverged view the pricing rule or the ceiling held", |i, d| {
            probe_idle(i) && i.frame.saturating_sub(i.dispatch_frame) >= 3 && !d.fired
        }),
    ];
    let price_terms: [(&'static str, fn(&WalkPriceInputs, &WalkPriceDecision) -> bool); 9] = [
        ("a settled pass was priced", |i, d| {
            !i.interacting && !i.pin_frame && i.ledger.inflight.is_some() && d.ledger.inflight.is_none()
        }),
        ("a pass was held unpriced", |i, d| i.ledger.inflight.is_some() && d.ledger.inflight.is_some()),
        ("a pin pass was priced", |_, d| {
            d.events.iter().any(|e| matches!(e, WalkPriceEvent::PinPriced { .. }))
        }),
        ("the lethal band shed the licences", |_, d| {
            d.events.iter().any(|e| matches!(e, WalkPriceEvent::LethalShed { .. }))
        }),
        ("a pin pass was released before its reading, its price deferred", |_, d| {
            d.events.iter().any(|e| matches!(e, WalkPriceEvent::PinDeferred { .. }))
        }),
        ("a released pin pass was priced by its own reading", |i, d| {
            i.ledger.late.is_some() && i.late_gpu_ms >= 0.0 && d.ledger.late.is_none()
        }),
        ("motion abandoned a pass", |i, _| i.interacting && !i.pin_frame && i.ledger.inflight.is_some()),
        ("the growth limiter bound the step", |i, d| {
            (i.interacting || i.pin_frame)
                && i.ledger.motion_step_last > 0
                && d.budget_step == (2 * i.ledger.motion_step_last).max(d.floor)
        }),
        ("an unmeasured walk", |i, _| !i.budget_measured),
    ];
    // (A partial reference capping a settled walk is unit-tested only: no recorded source settles on
    // one — `a_settled_walk_covers_only_its_live_range`.)
    let id_terms: [(&'static str, fn(&WalkIdInputs, &WalkIdDecision) -> bool); 6] = [
        ("a settled walk resumed", |i, d| !d.restarted && !i.pin_frame && !i.interacting),
        ("a restart threw a part-way walk away", |_, d| d.restart_lost.is_some()),
        ("a supersampling sample kept the band ledger", |i, d| {
            d.restarted && !d.clear_ledger && !i.interacting && !i.pin_frame
        }),
        ("another view cleared the band ledger", |_, d| d.clear_ledger),
        ("a pin frame kept its walk", |i, _| i.pin_frame),
        ("the SA-skip floor", |_, d| d.walk_floor > 0),
    ];
    // (The pin gate's wall-clock escape is unit-tested only: W3 measured none on either card —
    // `a_pin_pass_escapes_a_completion_that_never_comes`.)
    let dispatch_terms: [(&'static str, fn(&WalkDispatchInputs, &WalkDispatchDecision) -> bool); 8] = [
        ("a settled pass went out", |i, d| !i.interacting && !i.pin_frame && d.pass),
        ("one unpriced pass held the walk", |i, d| !i.pin_frame && !d.pass && i.cur < i.walk_end),
        ("a pin pass went out", |_, d| d.pin_dispatched),
        ("the pin gate held", |i, d| i.pin_frame && !d.pass && i.cur < i.walk_end),
        ("a moving frame previewed", |i, d| i.interacting && !i.pin_frame && d.pass),
        ("a pass was charged for its running pixels", |_, d| {
            d.events.iter().any(|e| matches!(e, DispatchEvent::Charge { .. }))
        }),
        ("a finished walk emitted its tail", |i, _| i.cur >= i.walk_end),
        ("the tail re-sent for its reading", |i, d| {
            i.cur >= i.walk_end && d.state.tail_retries > i.state.tail_retries
        }),
    ];
    let mut h_branches = [0u64; 9];
    let mut h_present = [0u64; 5];
    let mut h_climb_terms = [0u64; 11];
    let mut h_probe_terms = [0u64; 3];
    let mut h_price_terms = [0u64; 7];
    let mut h_id_terms = [0u64; 6];
    let mut h_dispatch_terms = [0u64; 8];
    for text in texts {
        for line in text.lines() {
            let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else { continue };
            if v["kind"] != "decision" {
                continue;
            }
            match v["stage"].as_str() {
                Some("moving") => tally(&v, &branches, &mut h_branches),
                Some("present") => tally(&v, &present, &mut h_present),
                Some("iter") => tally(&v, &climb_terms, &mut h_climb_terms),
                Some("probe") => tally(&v, &probe_terms, &mut h_probe_terms),
                Some("walk-price") => tally(&v, &price_terms, &mut h_price_terms),
                Some("walk-id") => tally(&v, &id_terms, &mut h_id_terms),
                Some("walk-dispatch") => tally(&v, &dispatch_terms, &mut h_dispatch_terms),
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    collect(&mut out, "moving", branches.iter().map(|(t, _)| *t), &h_branches);
    collect(&mut out, "present", present.iter().map(|(t, _)| *t), &h_present);
    collect(&mut out, "iter", climb_terms.iter().map(|(t, _)| *t), &h_climb_terms);
    collect(&mut out, "probe", probe_terms.iter().map(|(t, _)| *t), &h_probe_terms);
    collect(&mut out, "walk-price", price_terms.iter().map(|(t, _)| *t), &h_price_terms);
    collect(&mut out, "walk-id", id_terms.iter().map(|(t, _)| *t), &h_id_terms);
    collect(&mut out, "walk-dispatch", dispatch_terms.iter().map(|(t, _)| *t), &h_dispatch_terms);
    out
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
    if let Some(r) = &rep.regime_line {
        println!("--replay: regime ({} frame lines): {r}", rep.regime_lines);
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
            pass_fixed_ms: crate::tunables::PASS_FIXED_MS_DEFAULT,
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
        let texts: Vec<String> = files.iter().map(|f| std::fs::read_to_string(f).expect("readable fixture")).collect();
        // §7.5's coverage term (`fixture_coverage`): every branch of every lifted decision replayed.
        for c in fixture_coverage(texts.iter().map(String::as_str)) {
            assert!(c.hits > 0, "VACUOUS: no replayed {} decision where {}", c.stage, c.term);
        }
        // §7.5's regime term: a fixture ENTERED the §6.7 escaped-reference regime — the soak's own
        // predicate over the recordings' regime lines — so the set does not grade the controller only
        // where it is easy (`escaped-ref.jsonl`, the 2026-09-21 crash view under REF_ESCAPE_AT).
        assert!(
            texts.iter().any(|t| replay_lines(t).regime_entered),
            "VACUOUS: no fixture entered the §6.7 escaped-reference regime"
        );
        // …and the field term: the set's recordings come from more than one card (a field capture),
        // so a hidden input that differs between cards turns a replay red here, on the CPU.
        let mut adapters: Vec<String> = texts.iter().flat_map(|t| replay_lines(t).adapters).collect();
        adapters.sort();
        adapters.dedup();
        assert!(adapters.len() >= 2, "VACUOUS: the fixtures name fewer than two adapters: {adapters:?}");
        for (f, text) in files.iter().zip(&texts) {
            let rep = replay_lines(text);
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

    /// The replay's premise, checked where it can break silently: a lifted decision reads ONLY its
    /// inputs. A global it reaches — a tunable read in place, the adapter's calibration, a clock —
    /// is an input the recording does not carry, so the replay recomputes it in another process
    /// (another card, CI with none) and differs, or worse, agrees only on the machine that recorded.
    /// The first `--uitest` recording found one: `decide_moving` reached
    /// `calibration::pass_fixed_ms()` through `motion_pass_steps`. Every `decide_*` body here and
    /// every free function it reaches (transitively, by name, across the render sources and
    /// `main.rs`) is scanned for the ways a global is read.
    #[test]
    fn a_lifted_decision_reads_no_global() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut sources = vec![root.join("render.rs"), root.join("main.rs")];
        for e in std::fs::read_dir(root.join("render")).expect("render/").flatten() {
            if e.path().extension().is_some_and(|x| x == "rs") {
                sources.push(e.path());
            }
        }
        // Free functions (column 0) by name → body, comments stripped.
        let strip = |l: &str| l.split("//").next().unwrap_or("").to_string();
        let mut fns: std::collections::HashMap<String, String> = Default::default();
        for p in &sources {
            let text = std::fs::read_to_string(p).expect("source").replace("\r\n", "\n");
            let lines: Vec<&str> = text.lines().collect();
            let mut k = 0;
            while k < lines.len() {
                let l = lines[k];
                let head = ["pub(crate) fn ", "pub(super) fn ", "pub fn ", "fn "]
                    .iter()
                    .find_map(|p| l.strip_prefix(p));
                if let Some(name) = head.and_then(|h| h.split(['(', '<']).next()) {
                    let start = k;
                    while k < lines.len() && lines[k] != "}" {
                        k += 1;
                    }
                    let body: Vec<String> = lines[start..k.min(lines.len())].iter().map(|l| strip(l)).collect();
                    fns.entry(name.to_string()).or_insert_with(|| body.join("\n"));
                }
                k += 1;
            }
        }
        let decide: Vec<String> = fns.keys().filter(|n| n.starts_with("decide_")).cloned().collect();
        assert!(decide.len() >= 3, "VACUOUS: found the lifted decisions {decide:?}");
        let forbidden = [
            "cost()",
            "calibration::",
            "overridden(",
            "app_micros(",
            "Instant::now",
            "SystemTime",
            ".load(",
            "env::var",
            "trace_on(",
        ];
        for d in &decide {
            let mut seen = std::collections::BTreeSet::new();
            let mut todo = vec![d.clone()];
            while let Some(name) = todo.pop() {
                if !seen.insert(name.clone()) {
                    continue;
                }
                let body = &fns[&name];
                for bad in forbidden {
                    assert!(
                        !body.contains(bad),
                        "{d} reaches `{name}`, which reads a global (`{bad}`): make it an input"
                    );
                }
                // Every identifier called here that is a free function of the scanned sources.
                let bytes = body.as_bytes();
                let mut s = 0;
                for (j, &b) in bytes.iter().enumerate() {
                    if !(b.is_ascii_alphanumeric() || b == b'_') {
                        if b == b'(' && j > s {
                            let id = &body[s..j];
                            if fns.contains_key(id) && !seen.contains(id) {
                                todo.push(id.to_string());
                            }
                        }
                        s = j + 1;
                    }
                }
            }
            assert!(seen.len() > 1 || d == "decide_present", "VACUOUS: {d} reached no helper");
        }
    }

    #[test]
    fn a_recorded_regime_is_judged_by_the_soaks_predicate() {
        // A storm frame: a short COMPLETE reference (655 against a 4,627 ask), BLA off, and a counter
        // reading with rebases and no BLA skips. Recorded as a regime line, it enters the regime.
        let storm = RegimeFacts {
            frame: 358,
            t_ms: 22_600,
            view: 0,
            has_ref: true,
            ref_partial: false,
            ref_len: 655,
            gpu_iter: 4_627,
            bla_on: false,
            ctr_new: true,
            ctr_rebase: 11_492_892,
            ctr_bla_skip: 0,
            dispatched: true,
            chunked: true,
            fe_budget: 1_779_000_000,
        };
        let line = |f: RegimeFacts| serde_json::json!({"kind": "regime", "facts": f}).to_string();
        let rep = replay_lines(&line(storm));
        assert_eq!((rep.regime_lines, rep.regime_entered, rep.unreadable), (1, true, 0));
        assert!(rep.regime_line.as_deref().is_some_and(|l| l.starts_with("ENTERED")), "{:?}", rep.regime_line);
        // The controls: the shape without a storm reading, and a reference long enough for its ask.
        assert!(!replay_lines(&line(RegimeFacts { ctr_new: false, ..storm })).regime_entered);
        assert!(!replay_lines(&line(RegimeFacts { ref_len: 2_000, ..storm })).regime_entered);
        assert!(!replay_lines(&line(RegimeFacts { bla_on: true, ..storm })).regime_entered);
        // And it is judged by the soak's predicate itself, on the record it came from.
        let mut acc = crate::soak::RegimeAcc::default();
        acc.add(&storm.record());
        assert!(acc.report().entered);
        assert_eq!(RegimeFacts::of(&storm.record()), storm, "the facts round-trip through the record");
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

    /// A settled view at 2^200 (zoom cap 2,000 + 200 × 256 = 53,200) under auto-iterations, with
    /// the full 10M appetite, nothing climbed yet.
    fn climb() -> IterInputs {
        IterInputs {
            interacting: false,
            auto_iter: true,
            eff_iter: crate::MAX_ITER_LIMIT,
            log2mag: 200.0,
            iter_cap: crate::MAX_ITER_LIMIT,
            reading: None,
            ref_len: 0,
            ref_escaped: false,
            recompute_in_flight: false,
            state: IterState {
                boost: 1.0,
                probe: None,
                plateau: false,
                stall: 0,
                stall_base: 1.0,
                exhausted: false,
                repicked: false,
                verdict: None,
                capped_frac: None,
                budget_measured: 0,
                budget_maxed: false,
            },
        }
    }

    /// A MAXITER reading: this capped fraction, measured by a pass armed at `armed` iterations.
    fn capped(frac: f32, armed: u32) -> Option<u64> {
        Some(((frac.to_bits() as u64) << 32) | armed as u64)
    }

    /// The next frame of the same settled view, its reading measured at the budget the last one planned.
    fn next(i: &IterInputs, d: &IterDecision, frac: f32) -> IterInputs {
        IterInputs { reading: capped(frac, d.planned), state: d.state, ..i.clone() }
    }

    #[test]
    fn a_raise_that_helps_climbs_and_a_resolved_reading_is_the_verdict() {
        let i = IterInputs { reading: capped(0.5, 53_200), ..climb() };
        let d = decide_iter(&i);
        assert_eq!(d.budget_now, 53_200);
        assert_eq!(d.events, vec![IterEvent::Raised { frac: 0.5, from: 1.0, to: 1.6, flat: None }]);
        assert_eq!((d.state.probe, d.state.capped_frac, d.state.budget_measured), (Some((1.0, 0.5)), Some(0.5), 53_200));
        assert_eq!(d.state.verdict, None, "a raise is no verdict");
        assert_eq!((d.planned, d.gpu_iter), (85_120, 85_120));
        // Resolved at the raised budget: the boost stays, and that budget is the verdict.
        let d2 = decide_iter(&next(&i, &d, 0.0001));
        assert!(d2.events.is_empty());
        assert_eq!(d2.state.boost, 1.6);
        assert_eq!(d2.state.verdict, Some(85_120));
    }

    #[test]
    fn three_fruitless_raises_on_a_partly_capped_frame_are_interior() {
        let mut i = IterInputs { reading: capped(0.5, 53_200), ..climb() };
        let mut d = decide_iter(&i);
        let mut steps = vec![];
        for _ in 0..3 {
            i = next(&i, &d, 0.5);
            d = decide_iter(&i);
            steps.extend(d.events.iter().copied());
        }
        let flats: Vec<_> = steps
            .iter()
            .map(|e| match e {
                IterEvent::Raised { flat, .. } => *flat,
                _ => None,
            })
            .collect();
        assert_eq!(flats, vec![Some(1), Some(2), None]);
        assert_eq!(steps[2], IterEvent::Plateau { to: 1.0 });
        assert!(d.state.plateau);
        assert_eq!(d.state.boost, 1.0, "back to where the fruitless run began");
        // A latched plateau stops the climb: the same reading again changes nothing.
        let after = decide_iter(&next(&i, &d, 0.5));
        assert!(after.events.is_empty());
        assert_eq!(after.state.boost, 1.0);
    }

    #[test]
    fn a_fully_capped_frame_keeps_climbing_through_flat_steps() {
        // 100% capped looks the same at every budget short of the need, so flat steps there prove
        // nothing (the 6.5e94× view needed ~121×): no plateau, the climb goes on in 2.5× steps.
        let mut i = IterInputs { reading: capped(1.0, 53_200), ..climb() };
        let mut d = decide_iter(&i);
        for _ in 0..4 {
            i = next(&i, &d, 1.0);
            d = decide_iter(&i);
        }
        assert!(!d.state.plateau);
        assert_eq!(d.state.boost, 2.5f64.powi(5));
    }

    #[test]
    fn motion_ends_a_fruitless_run_where_it_began() {
        let i = IterInputs {
            interacting: true,
            reading: capped(0.5, 218_009),
            state: IterState {
                boost: 4.096,
                probe: Some((2.56, 0.5)),
                stall: 2,
                stall_base: 1.0,
                verdict: Some(53_200),
                capped_frac: Some(0.5),
                ..climb().state
            },
            ..climb()
        };
        let d = decide_iter(&i);
        assert_eq!(d.events, vec![IterEvent::FlatRunEnded { stall: 2, from: 4.096, to: 1.0 }]);
        assert_eq!((d.state.boost, d.state.probe, d.state.stall), (1.0, None, 0));
        assert_eq!((d.state.verdict, d.state.capped_frac), (None, None), "a moving frame judges nothing");
        // A run that had not stalled keeps its boost through motion.
        let kept = decide_iter(&IterInputs { state: IterState { stall: 0, ..i.state }, ..i.clone() });
        assert_eq!(kept.state.boost, 4.096);
    }

    #[test]
    fn the_seed_needs_an_orbit_that_escaped() {
        let i = IterInputs { ref_len: 20 * 53_200, ref_escaped: true, ..climb() };
        let d = decide_iter(&i);
        assert_eq!(d.events, vec![IterEvent::Seeded { from: 1.0, to: 20.0, ref_len: 20 * 53_200 }]);
        assert_eq!((d.budget_now, d.planned), (20 * 53_200, 20 * 53_200));
        // An orbit that stopped at the length we asked for measures the ask, not the location.
        assert!(decide_iter(&IterInputs { ref_escaped: false, ..i.clone() }).events.is_empty());
        // Nor past a latched plateau, nor while moving, nor under an explicit count.
        let plateau = IterState { plateau: true, ..i.state };
        assert!(decide_iter(&IterInputs { state: plateau, ..i.clone() }).events.is_empty());
        assert!(decide_iter(&IterInputs { interacting: true, ..i.clone() }).events.is_empty());
        let explicit = decide_iter(&IterInputs { auto_iter: false, eff_iter: 70_000, ..i });
        assert!(explicit.events.is_empty());
        assert_eq!(explicit.planned, 70_000, "an explicit count is honoured verbatim");
    }

    #[test]
    fn a_reading_measured_at_another_budget_is_discarded() {
        // The readback lands frames late: a pre-raise "still capped" must not read as "the raise
        // did not help".
        let i = IterInputs { reading: capped(1.0, 33_250), ..climb() };
        let d = decide_iter(&i);
        assert!(d.reached_ask && d.stale);
        assert!(d.events.is_empty());
        assert_eq!((d.state.capped_frac, d.state.verdict), (None, None));
    }

    #[test]
    fn all_capped_at_the_full_appetite_repicks_once_then_reverts() {
        // The appetite (53,200) binds at boost 2, and every pixel is still capped there.
        let i = IterInputs {
            eff_iter: 53_200,
            reading: capped(1.0, 53_200),
            state: IterState { boost: 2.0, ..climb().state },
            ..climb()
        };
        let d = decide_iter(&i);
        assert!(d.repick && d.state.repicked);
        assert_eq!(d.events, vec![IterEvent::Repick { frac: 1.0, budget: 53_200 }]);
        assert_eq!(d.state.boost, 2.0, "the boost stays at the appetite while the re-pick builds");
        // Still in flight: wait, decide nothing.
        let wait = decide_iter(&IterInputs { recompute_in_flight: true, ..next(&i, &d, 1.0) });
        assert!(!wait.repick && wait.events.is_empty());
        assert_eq!(wait.state.boost, 2.0);
        // Landed and still all-capped: interior; the cheap budget paints the same flat frame.
        let done = decide_iter(&next(&i, &wait, 1.0));
        assert_eq!(done.events, vec![IterEvent::Exhausted { frac: 1.0, budget: 53_200, from: 2.0 }]);
        assert!(done.state.plateau && done.state.exhausted && !done.repick);
        assert_eq!(done.state.boost, 1.0);
    }

    #[test]
    fn a_recorded_climb_round_trips() {
        let i = IterInputs { reading: capped(0.37, 53_200), ref_len: 9_999, ..climb() };
        let d = decide_iter(&i);
        let line = serde_json::json!({"kind": "decision", "frame": 1, "view": 0, "stage": "iter",
            "inputs": i, "decision": d}).to_string();
        let rep = replay_lines(&line);
        assert_eq!((rep.decisions, rep.mismatches, rep.unreadable), (1, 0, 0));
    }

    /// A settled view whose budget has not converged, nothing else dispatching, the last real
    /// dispatch 10 frames ago, the bootstrap budget (4e8) in force, and a frame of 8e8 nominal steps.
    /// A settled prefer-detail view whose chunked walk composes this frame, nothing held yet.
    fn composing() -> PresentInputs {
        PresentInputs {
            direct: false,
            chunk_over: true,
            chunk_range: Some([0, 4_096]),
            probe_fired: false,
            tile_work: false,
            grid_composing: false,
            res_scale: 1.0,
            interacting: false,
            offscreen: false,
            will_reproject: false,
            key_changed: false,
            gpu_iter: 50_000,
            ss: 1,
            spx: 1_000_000,
            prefer_detail: true,
            tour_playing: false,
            aa: 1,
            hold_active: false,
            pin_active: false,
            chunk_dirty: false,
            live_verified: true,
            aa_unmeasured: false,
            frame: 100,
        }
    }

    #[test]
    fn a_settled_compose_snapshots_a_verified_texture() {
        let d = decide_present(&composing());
        assert!(d.display_hold && !d.pin_gate);
        assert!(d.hold_copy);
        assert_eq!(d.hold_verified, Some(true));
    }

    #[test]
    fn a_settled_compose_never_snapshots_an_unverified_texture() {
        // U42: it took the snapshot of whatever the texture held and served it for the whole
        // compose. Now it takes none — the display shows the compose live — and says nothing new
        // about the hold (the frame that dropped the last one already called it unverified).
        let d = decide_present(&PresentInputs { live_verified: false, ..composing() });
        assert!(d.display_hold && !d.pin_gate, "still gated: the GPU shows live for want of a snapshot");
        assert!(!d.hold_copy);
        assert_eq!(d.hold_verified, None);
        assert!(d.hold_active, "and not retaken mid-compose, over the composing texture");
        // …and a compose already under way keeps the snapshot it took, verified or not.
        let d = decide_present(&PresentInputs { live_verified: false, hold_active: true, ..composing() });
        assert!(!d.hold_copy && d.display_hold);
    }

    fn probe() -> ProbeInputs {
        ProbeInputs {
            interacting: false,
            offscreen: false,
            will_reproject: false,
            tour_playing: false,
            key_changed: false,
            tile: false,
            chunk_range: None,
            budget_ok: false,
            frame: 100,
            dispatch_frame: 90,
            spx: 1000 * 800,
            ss: 1,
            gpu_iter: 1000,
            fe_budget: 0,
            bootstrap_steps: 400_000_000,
            explicit_dispatch_cap: crate::tunables::EXPLICIT_DISPATCH_CAP_DEFAULT,
            tdr_min_steps: crate::tunables::TDR_MIN_STEPS_DEFAULT,
            tdr_steps_ceil: crate::tunables::TDR_STEPS_CEIL_DEFAULT,
        }
    }

    #[test]
    fn the_probe_re_measures_an_unconverged_idle_view() {
        let d = decide_probe(&probe());
        assert!(d.fired);
        assert_eq!(d.steps, 800_000_000, "the forced pass's nominal steps, for its measurement");
        // Anything else dispatching this frame, any motion, or a converged budget: no probe.
        for (what, i) in [
            ("moving", ProbeInputs { interacting: true, ..probe() }),
            ("a changed key", ProbeInputs { key_changed: true, ..probe() }),
            ("a settle tile", ProbeInputs { tile: true, ..probe() }),
            ("a chunk range", ProbeInputs { chunk_range: Some([0, 256]), ..probe() }),
            ("a converged budget", ProbeInputs { budget_ok: true, ..probe() }),
            ("a reprojection", ProbeInputs { will_reproject: true, ..probe() }),
            ("offscreen", ProbeInputs { offscreen: true, ..probe() }),
            ("a playing tour", ProbeInputs { tour_playing: true, ..probe() }),
        ] {
            assert_eq!(decide_probe(&i), ProbeDecision { fired: false, steps: 0 }, "{what}");
        }
    }

    #[test]
    fn the_probe_is_paced_three_frames_apart() {
        assert!(!decide_probe(&ProbeInputs { dispatch_frame: 98, ..probe() }).fired);
        assert!(decide_probe(&ProbeInputs { dispatch_frame: 97, ..probe() }).fired);
    }

    #[test]
    fn the_probe_fires_only_for_a_reading_the_pricing_rule_keeps() {
        // The boundary is `PRICE_REPRESENTATIVE_FRAC` of the budget, read from the constant (the
        // rule that discards a reading and the rule that spends a dispatch on one cannot split).
        let at = (400_000_000f64 * super::super::PRICE_REPRESENTATIVE_FRAC).ceil() as u64;
        let spx = at / 1000;
        assert_eq!(spx * 1000, at, "the boundary lands on a whole pixel count");
        assert!(decide_probe(&ProbeInputs { spx, ..probe() }).fired);
        assert!(!decide_probe(&ProbeInputs { spx: spx - 1, ..probe() }).fired);
    }

    #[test]
    fn the_probe_stops_at_the_explicit_dispatch_ceiling() {
        // A measured budget at the ceiling: the next ×1.5 step was the lethal one (beta.48).
        let cap = crate::tunables::EXPLICIT_DISPATCH_CAP_DEFAULT;
        let big = ProbeInputs { fe_budget: cap, spx: cap / 1000, ..probe() };
        assert!(!decide_probe(&big).fired);
        assert!(decide_probe(&ProbeInputs { fe_budget: cap - 1, ..big.clone() }).fired);
        // The budget is clamped to the tunable bounds first, and those are inputs.
        assert!(!decide_probe(&ProbeInputs { fe_budget: cap - 1, tdr_min_steps: cap, ..big }).fired);
    }

    /// A settled chunk walk on a 1000×800 panel at ss 1, a measured budget, stock tunables, and one
    /// 1,024-iteration pass in band 3 in flight for 50 ms.
    fn walk() -> WalkPriceInputs {
        let c = crate::tunables::cost();
        WalkPriceInputs {
            interacting: false,
            pin_frame: false,
            auto_iter: true,
            spx: 1000 * 800,
            ss: 1,
            tdr_steps: 4_000_000_000,
            gpu_iter: 100_000,
            budget_measured: true,
            worst_rate: Some(2.0e7),
            motion_rate: 2.0e7,
            last_dt_ms: 16.7,
            frame_period_ms: 16.7,
            present_throttle: 0,
            frame: 500,
            pin_pass_gpu_ms: -1.0,
            pin_pass_frame: 0,
            pin_inflight: 0,
            pin_done_at: 0,
            pin_dispatch_at: 0,
            late_gpu_ms: -1.0,
            no_timestamps: false,
            ledger: WalkLedger {
                inflight: Some((1024, 3, 50.0, false)),
                bands: [0; crate::tunables::CHUNK_BANDS],
                pass_dt: 0.0,
                motion_step_last: 0,
                late: None,
            },
            tdr_budget_ms: c.tdr_budget_ms,
            tdr_explicit_budget_ms: c.tdr_explicit_budget_ms,
            tdr_lethal_ms: c.tdr_lethal_ms,
            tdr_bootstrap_steps: c.tdr_bootstrap_steps,
            admission: c.admission,
            pass_fixed_ms: crate::tunables::PASS_FIXED_MS_DEFAULT,
        }
    }

    #[test]
    fn a_quick_frame_prices_the_pass_in_flight() {
        // With one pass in flight, a quick present proves the queue drained: the pass is priced at
        // the wall it accumulated (50 + this frame's 16.7 ms), its band earns a licence, and the next
        // dispatch is released.
        let d = decide_walk_price(&walk());
        assert_eq!(d.ledger.inflight, None);
        assert!((d.ledger.pass_dt - 66.7).abs() < 1e-9, "{}", d.ledger.pass_dt);
        assert_eq!(d.ledger.bands[3], 2048, "well under half the target: the fast lane doubles the licence");
        assert!(d.events.is_empty() && d.retreat.is_none());
    }

    /// The RTX 3080 trace at 1x on the (49,3) spiral (2026-10-14): a 545-iteration pin opening
    /// dispatched at frame 498, its completion seen at 500 (35 ms of wall at 17.5 ms frames, so the
    /// stand-in reads 0.0), its reading (12.5 ms) landing at 501.
    fn released_pin_pass() -> WalkPriceInputs {
        WalkPriceInputs {
            interacting: true,
            pin_frame: true,
            frame: 500,
            frame_period_ms: 17.5,
            pin_pass_frame: 498,
            pin_inflight: 0,
            pin_dispatch_at: 1_000_000,
            pin_done_at: 1_035_000,
            ledger: WalkLedger { inflight: Some((545, 0, 6.7, false)), ..walk().ledger },
            ..walk()
        }
    }

    #[test]
    fn a_pin_pass_released_before_its_reading_is_priced_by_that_reading() {
        let d = decide_walk_price(&released_pin_pass());
        assert_eq!(d.ledger.inflight, None, "released at its completion");
        assert_eq!(d.ledger.bands[0], 0, "not priced by a stand-in floored to 0.0");
        assert_eq!(d.ledger.late, Some((545, 0, 498, 0.0)));
        assert!(matches!(d.events[..], [WalkPriceEvent::PinDeferred { size: 545, .. }]), "{:?}", d.events);
        // The reading lands: 12.5 ms against the 20 ms target holds the licence near the pass
        // (×1.25), where the stand-in took the ×16 lane to 8,720.
        let next = WalkPriceInputs { frame: 501, late_gpu_ms: 12.5, ledger: d.ledger, ..released_pin_pass() };
        let e = decide_walk_price(&WalkPriceInputs { ledger: WalkLedger { inflight: None, ..next.ledger }, ..next });
        assert_eq!(e.ledger.bands[0], 545 + 545 / 4);
        assert_eq!(e.ledger.late, None);
        assert!((e.ledger.pass_dt - 12.5).abs() < 1e-9);
    }

    #[test]
    fn without_timestamps_the_stand_in_is_the_price_at_once() {
        let d = decide_walk_price(&WalkPriceInputs { no_timestamps: true, ..released_pin_pass() });
        assert_eq!(d.ledger.late, None);
        assert_eq!(d.ledger.bands[0], 545 * 16, "the completion stand-in is all such a device has");
    }

    #[test]
    fn a_reading_that_never_comes_leaves_the_stand_in_standing() {
        let d = decide_walk_price(&released_pin_pass());
        let wait = crate::tunables::PIN_PRICE_WAIT_FRAMES;
        let quiet = |frame| WalkPriceInputs {
            frame,
            ledger: WalkLedger { inflight: None, ..d.ledger },
            ..released_pin_pass()
        };
        assert_eq!(decide_walk_price(&quiet(498 + wait - 1)).ledger.late, d.ledger.late, "still waiting");
        let e = decide_walk_price(&quiet(498 + wait));
        assert_eq!(e.ledger.late, None);
        assert_eq!(e.ledger.bands[0], 545 * 16);
    }

    #[test]
    fn a_slow_frame_holds_the_pass_unpriced() {
        // A frame past CHUNK_DRAIN_DT_MS proves nothing (the queue may still hold the pass): keep
        // accumulating, release nothing.
        let d = decide_walk_price(&WalkPriceInputs { last_dt_ms: 150.0, ..walk() });
        assert_eq!(d.ledger.inflight, Some((1024, 3, 200.0, false)));
        assert_eq!(d.ledger.bands, [0; crate::tunables::CHUNK_BANDS]);
    }

    #[test]
    fn a_pass_in_the_lethal_band_sheds_the_licences_once() {
        let i = walk();
        let lethal = WalkPriceInputs {
            last_dt_ms: 150.0,
            ledger: WalkLedger {
                inflight: Some((1024, 3, i.tdr_lethal_ms, false)),
                bands: [4096; crate::tunables::CHUNK_BANDS],
                ..i.ledger
            },
            ..i
        };
        let d = decide_walk_price(&lethal);
        assert!(matches!(d.events[..], [WalkPriceEvent::LethalShed { size: 1024, band: 3, .. }]), "{:?}", d.events);
        assert!(d.retreat.is_some());
        assert_eq!(d.ledger.bands, [0; crate::tunables::CHUNK_BANDS], "every licence back to the floor");
        // Shed, not released: the pass stays in flight, latched, and the next frame sheds nothing more.
        assert!(matches!(d.ledger.inflight, Some((1024, 3, _, true))));
        let next = decide_walk_price(&WalkPriceInputs { ledger: d.ledger, ..lethal });
        assert!(next.events.is_empty() && next.retreat.is_none());
    }

    #[test]
    fn motion_abandons_the_pass_in_flight_and_grows_its_step_gently() {
        let d = decide_walk_price(&WalkPriceInputs { interacting: true, ..walk() });
        assert_eq!(d.ledger.inflight, None, "a moving frame restarts the walk; the record goes too");
        assert_eq!(d.ledger.motion_step_last, d.budget_step);
        assert!(d.budget_step <= crate::tunables::MOTION_STEP_OPEN, "the opening step: {}", d.budget_step);
        // Never more than double the previous moving step (nor below the floor).
        let grown = decide_walk_price(&WalkPriceInputs {
            interacting: true,
            ledger: WalkLedger { motion_step_last: 300, ..walk().ledger },
            ..walk()
        });
        assert!((grown.floor..=600).contains(&grown.budget_step), "{}", grown.budget_step);
    }

    #[test]
    fn an_unmeasured_walk_opens_at_any_floor() {
        // No measurement yet: the floor is 1, so the bootstrap bound sizes the opening pass at any
        // panel size (the 256 floor is earned by a measured budget).
        assert_eq!(decide_walk_price(&WalkPriceInputs { budget_measured: false, ..walk() }).floor, 1);
        assert_eq!(decide_walk_price(&walk()).floor, 256);
    }

    /// A settled view mid-walk (cursor 3,000 of 20,000) whose signature is already this frame's, with an
    /// SA-seeded complete reference of 50,000 samples.
    fn walk_id() -> WalkIdInputs {
        let mut i = WalkIdInputs {
            interacting: false,
            pin_frame: false,
            fe_walk: true,
            center: [-0.75, 0.1],
            magnification: 1.0e30,
            pos_sig: 0x1234_5678,
            settings_hash: 0xabcd,
            series_seed: 0x5eed,
            jitter_bits: [0, 0],
            gpu_iter: 20_000,
            resolution: super::super::Resolution::panel([1000, 800]),
            ss: 1,
            pass_steps: 4_000_000_000,
            has_ref: true,
            orbit_len: 50_000,
            sa_skip: 5_000,
            partial: false,
            chunk_charge: 1,
            state: WalkState {
                sig: super::super::WalkSig::NONE,
                walk_view: super::super::WalkSig::NONE,
                epoch: 7,
                running: None,
                tail_retries: 0,
                cursor: 3_000,
                idx: 4,
                last_range: Some([2_000, 3_000]),
                walk_end: 20_000,
                restarts: 0,
                restart_pending: false,
                count_walk: 0,
                motion_pass_steps_last: 0,
                motion_frames: 0,
            },
        };
        // Already this view's walk: its signatures are the ones this frame makes.
        let d = decide_walk_id(&WalkIdInputs { pin_frame: true, ..i.clone() });
        i.state.sig = d.sig;
        i.state.walk_view = decide_walk_id(&WalkIdInputs { pin_frame: true, jitter_bits: [0, 0], ..i.clone() }).sig;
        i
    }

    #[test]
    fn the_same_view_resumes_its_walk() {
        let i = walk_id();
        let d = decide_walk_id(&i);
        assert!(!d.restarted && !d.clear_ledger && d.restart_lost.is_none());
        assert_eq!((d.cur, d.state.cursor, d.state.idx, d.state.epoch), (3_000, 3_000, 4, 7));
        assert_eq!(d.state.count_walk, 7, "a settled mode-2 walk counts its running pixels");
    }

    #[test]
    fn every_input_the_signature_folds_restarts_the_walk() {
        // The composition, pinned: each input on its own moves the signature.
        let i = walk_id();
        let base = decide_walk_id(&i).sig;
        for (what, j) in [
            ("the centre's re", WalkIdInputs { center: [-0.7500001, 0.1], ..i.clone() }),
            ("the centre's im", WalkIdInputs { center: [-0.75, 0.1000001], ..i.clone() }),
            ("the magnification", WalkIdInputs { magnification: 1.1e30, ..i.clone() }),
            ("the reference length", WalkIdInputs { orbit_len: 50_001, ..i.clone() }),
            ("the exact position", WalkIdInputs { pos_sig: 0x1234_5679, ..i.clone() }),
            ("the settings", WalkIdInputs { settings_hash: 0xabce, ..i.clone() }),
            ("the series seed", WalkIdInputs { series_seed: 0x5eee, ..i.clone() }),
            ("the jitter", WalkIdInputs { jitter_bits: [1, 0], ..i.clone() }),
            ("the ask", WalkIdInputs { gpu_iter: 20_001, ..i.clone() }),
            ("the texture", WalkIdInputs { resolution: super::super::Resolution::panel([1000, 801]), ..i.clone() }),
            ("the ss", WalkIdInputs { ss: 2, ..i.clone() }),
        ] {
            let d = decide_walk_id(&j);
            assert_ne!(d.sig, base, "{what}");
            assert!(d.restarted && d.cur == 0, "{what}");
            assert_eq!(d.restart_lost, Some((3_000, 20_000, base)), "{what}: a part-way walk is counted lost");
            assert_eq!((d.state.restarts, d.state.epoch), (1, 8), "{what}");
        }
    }

    #[test]
    fn a_new_supersampling_sample_keeps_the_band_ledger() {
        // The same view at a new jitter restarts the walk but keeps the ORBIT's prices; another view
        // clears them.
        let i = walk_id();
        let sample = decide_walk_id(&WalkIdInputs { jitter_bits: [0x3e9e_b852, 0xbe61_47ae], ..i.clone() });
        assert!(sample.restarted && !sample.clear_ledger);
        let other = decide_walk_id(&WalkIdInputs { magnification: 2.0e30, ..i });
        assert!(other.restarted && other.clear_ledger);
    }

    #[test]
    fn motion_restarts_every_frame_and_a_pin_never_does() {
        let i = walk_id();
        let moving = decide_walk_id(&WalkIdInputs { interacting: true, ..i.clone() });
        assert!(moving.restarted && !moving.clear_ledger, "the ledger survives the pins of a glide");
        assert_eq!((moving.walk_floor, moving.walk_end), (0, 20_000), "motion walks [0, ask)");
        assert_eq!(moving.state.motion_pass_steps_last, 4_000_000_000);
        assert_eq!(moving.state.motion_frames, 1);
        // A pin owns its progression: a moved signature does not restart it.
        let pin = decide_walk_id(&WalkIdInputs { pin_frame: true, interacting: true, magnification: 2.0e30, ..i });
        assert!(!pin.restarted);
        assert_eq!(pin.cur, 3_000);
    }

    #[test]
    fn a_settled_walk_covers_only_its_live_range() {
        let i = walk_id();
        let d = decide_walk_id(&i);
        let we = super::super::sa_skip_rescue(20_000, 5_000, 50_000);
        assert_eq!((d.walk_floor, d.walk_end), (super::super::usable_sa_skip(5_000, we), we));
        assert_eq!(d.state.walk_end, d.walk_end);
        // A partial reference caps the walk at the orbit it holds.
        let partial = decide_walk_id(&WalkIdInputs { partial: true, orbit_len: 12_000, ..i.clone() });
        assert!(partial.walk_end <= 11_999, "{}", partial.walk_end);
        // No reference yet: the whole ask from 0.
        let none = decide_walk_id(&WalkIdInputs { has_ref: false, ..i });
        assert_eq!((none.walk_floor, none.walk_end), (0, 20_000));
    }

    /// A settled walk at cursor 3,000 of [0, 20,000), its last pass priced (nothing in flight), every
    /// band still cold, a 1000×800 panel at ss 1.
    fn dispatch() -> WalkDispatchInputs {
        WalkDispatchInputs {
            interacting: false,
            pin_frame: false,
            fe_walk: false,
            cur: 3_000,
            walk_floor: 0,
            walk_end: 20_000,
            floor: 256,
            pass_steps: 4_000_000_000,
            budget_step: 5_000,
            gpu_iter: 20_000,
            spx: 1000 * 800,
            ss: 1,
            frame: 500,
            now_us: 1_000_000,
            last_dt_ms: 16.7,
            worst_rate: Some(2.0e7),
            bands: [0; crate::tunables::CHUNK_BANDS],
            orbit_len: 50_000,
            epoch: 7,
            running: None,
            knee_px: 0,
            accum_active: false,
            walk_read_sig: 0,
            norm_sig_submit: 0,
            pin_timeout_scaled: 1,
            chunk_charge: 1,
            chunk_shuffle: 0,
            state: DispatchState {
                pin_inflight: 0,
                pin_reg_pending: 0,
                pin_dispatch_at: 0,
                pin_escapes: 0,
                pin_pass_steps: 0,
                pin_pass_frame: 0,
                pin_pass_gpu_ms: -1.0,
                last_range: Some([2_000, 3_000]),
                inflight: None,
                idx: 4,
                cursor: 3_000,
                pending: false,
                charged_px: 0,
                fe_steps_last: 0,
                tail_retries: 0,
                probe_nonce: 0,
                norm_complete_submit: true,
            },
        }
    }

    #[test]
    fn a_settled_pass_goes_out_at_its_band_licence_and_waits_to_be_priced() {
        let d = decide_walk_dispatch(&dispatch());
        // A cold band licenses the floor: one floor-sized pass meets a cost cliff, not a budget-sized one.
        assert_eq!(d.chunk_range, Some([3_000, 3_256]));
        assert_eq!((d.chunk_idx, d.pass), (4, true));
        assert_eq!((d.state.cursor, d.state.idx, d.state.pending), (3_256, 5, true));
        assert_eq!(d.state.inflight, Some((256, 4, 0.0, false)), "in flight, unpriced, in band 4");
        assert_eq!(d.state.last_range, d.chunk_range);
        assert_eq!(d.state.fe_steps_last, 800_000 * 256, "the measurement pairs with the work done");
        assert!(!d.state.norm_complete_submit, "a part of the walk is not the picture");
    }

    #[test]
    fn one_unpriced_pass_holds_the_settled_walk() {
        // Hold the last dispatched range: the GPU dedupes it and the drain frame prices the pass.
        let mut i = dispatch();
        i.state.inflight = Some((256, 4, 30.0, false));
        let d = decide_walk_dispatch(&i);
        assert_eq!((d.chunk_range, d.chunk_idx, d.pass), (Some([2_000, 3_000]), 4, false));
        assert_eq!((d.state.cursor, d.state.pending), (3_000, true));
    }

    #[test]
    fn a_moving_frame_previews_without_advancing_the_walk() {
        let d = decide_walk_dispatch(&WalkDispatchInputs { interacting: true, ..dispatch() });
        assert!(d.pass && d.chunk_range.is_some_and(|[s, e]| s == 3_000 && e > s));
        assert_eq!((d.state.cursor, d.state.idx, d.state.inflight), (3_000, 4, None));
        assert!(d.state.norm_complete_submit, "a moving frame's one-pass preview IS what is displayed");
    }

    #[test]
    fn a_pin_pass_escapes_a_completion_that_never_comes() {
        // One pin pass in the queue for 500 ms, registered, at a 16.7 ms frame: past the 170 ms gate.
        let mut i = WalkDispatchInputs { pin_frame: true, interacting: true, ..dispatch() };
        i.state.pin_inflight = 1;
        i.state.pin_dispatch_at = 500_000;
        let d = decide_walk_dispatch(&i);
        assert_eq!(
            d.events.first(),
            Some(&DispatchEvent::PinEscape { age_us: 500_000, limit_us: 170_000, registered: true, n: 1 })
        );
        assert!(d.pass && d.pin_dispatched, "released: the next pin pass goes out");
        assert_eq!((d.state.pin_inflight, d.state.pin_reg_pending, d.state.pin_escapes), (1, 1, 1));
        assert_eq!((d.state.pin_pass_frame, d.state.pin_pass_gpu_ms), (500, -1.0));
        assert_eq!(Some(d.state.cursor), d.chunk_range.map(|[_, e]| e), "a pin advances its walk");
        // Inside the gate it holds.
        i.state.pin_dispatch_at = 900_000;
        let held = decide_walk_dispatch(&i);
        assert!(!held.pass && held.events.is_empty() && !held.pin_dispatched);
    }

    #[test]
    fn a_finished_walk_resends_its_tail_until_its_reading_lands() {
        let i = WalkDispatchInputs { cur: 20_000, walk_read_sig: 1, norm_sig_submit: 2, ..dispatch() };
        let d = decide_walk_dispatch(&i);
        assert_eq!((d.chunk_range, d.pass), (Some([20_000, 20_000]), false));
        assert_eq!((d.state.probe_nonce, d.state.tail_retries), (1, 1), "a fresh nonce re-sends the tail");
        assert!(d.state.norm_complete_submit);
        // Its reading has landed: nothing more to send.
        let landed = decide_walk_dispatch(&WalkDispatchInputs { walk_read_sig: 2, ..i });
        assert_eq!((landed.state.probe_nonce, landed.state.tail_retries), (0, 0));
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
