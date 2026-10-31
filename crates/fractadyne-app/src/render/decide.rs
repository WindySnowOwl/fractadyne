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
    /// The held picture stands across a JUMP and the new view's palette window is not adopted yet
    /// (the caller: normalised colouring, the view's mapping not locked, within
    /// `tunables::JUMP_MAP_WAIT_S` of the compose ending).
    #[serde(default, skip_serializing_if = "is_false")]
    pub(crate) await_map: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
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
    /// The hold stays up with nothing composing, for the new view's palette window (`await_map`).
    #[serde(default, skip_serializing_if = "is_false")]
    pub(crate) awaiting_map: bool,
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
    // ⭐A direct view's chunked walk holds too. The direct exclusion was written for the
    // prefer-detail tiles (cheap, and each tile complete), and it caught the walks direct mode
    // later gained: when the iteration climb raised the ask on a settled direct view, the new walk
    // was shown from its first pass — 7 iterations, every pixel still pending — and the screen
    // went EMPTY for a frame (the user's 2026-10-25 video at 99.78×, ask 29,970 → 47,953;
    // reproduced with `--zoomtest-click`, ask 9,469 → 15,151). The hold machinery is the same in
    // both modes (`ContentTrack` verifies direct frames; the GPU copy is mode-blind).
    let gate_capable = (i.prefer_detail || chunk_over)
        && !offscreen
        && !interacting
        && (!i.direct || chunk_over);
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
    // ⭐A held picture standing across a JUMP stays up once its compose is done, until the new
    // view's palette window is adopted (`await_map`). Revealed at once, the new picture was drawn
    // under the OLD view's window for the frames its reading took to come back, then dissolved
    // from it: after a 100× click at 2^429 the whole new picture lay in 1% of the old window, one
    // pale colour that took 0.4 s to fade into the picture (2026-10-31). Only an existing hold is
    // kept; none is started for it.
    let awaiting_map = !composing && i.await_map && i.hold_active;
    let mut pin_gate = false;
    let mut hold_active = i.hold_active;
    let mut hold_verified = None;
    let (hold_copy, display_hold) = if gate_capable && (composing || awaiting_map) {
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
    PresentDecision {
        hold_copy,
        display_hold,
        pin_gate,
        aa_filter,
        hold_active,
        hold_verified,
        aa_probe,
        awaiting_map: awaiting_map && gate_capable,
    }
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

/// Pin passes whose price may wait for their own reading at once (`WalkLedger::late`).
pub(crate) const LATE_MAX: usize = 4;

/// `WalkLedger::late` holds nothing (so a recording made before it compares equal).
fn no_late(l: &[Option<(u32, u8, u64, f64)>; LATE_MAX]) -> bool {
    l.iter().all(Option::is_none)
}

/// A work memory of 0 (none) is left out of a recording, as a ledger recorded before it had none.
fn no_work(w: &u64) -> bool {
    *w == 0
}

/// A band-cost memory holding nothing is left out of a recording (`WalkLedger::band_ms`), as a
/// ledger recorded before it had none.
fn no_cost(c: &[f32; crate::tunables::CHUNK_BANDS]) -> bool {
    c.iter().all(|&m| m == 0.0)
}

/// The same for the iterations and reference lengths beside it (`WalkLedger::band_ms_iters`,
/// `band_ms_len`).
fn no_cost_len(c: &[u32; crate::tunables::CHUNK_BANDS]) -> bool {
    c.iter().all(|&m| m == 0)
}

/// Pixel counts of nothing parked are left out of a recording (`WalkLedger::late_px`).
fn no_late_px(l: &[u32; LATE_MAX]) -> bool {
    l.iter().all(|&p| p == 0)
}

/// A recorded input's pin cap where the recording predates it: one.
fn one() -> u32 {
    1
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
    /// The same step as WORK, iterations × the pixels it was charged for (`WalkPriceInputs::
    /// growth_in_work`: at least the occupancy knee's);
    /// 0 = none, and in recordings made before it; absent when 0, so they replay as recorded).
    #[serde(default, skip_serializing_if = "no_work")]
    pub(crate) motion_work_last: u64,
    /// Pin passes whose PRICE waits for their own GPU reading: `(size, band, the frame it went out,
    /// a stand-in price in ms, < 0 = none)`. A pass is parked here when it is released at its
    /// completion before its reading landed (its completion stand-in is the fallback), or when it is
    /// still in flight as the next pin pass goes out (two in flight, `PIN_INFLIGHT_MAX`; no
    /// stand-in). See [`decide_walk_price`]'s deferred feed. (Absent when empty.)
    #[serde(default, skip_serializing_if = "no_late")]
    pub(crate) late: [Option<(u32, u8, u64, f64)>; LATE_MAX],
    /// Per iteration band: what the GPU last timed a pin pass's iterations there at, ms (a span's
    /// price shared out in proportion to what its bands cost before), the iterations that cost
    /// covered, and the reference length it was measured against (`PIN_SPAN_PRICED`,
    /// `feed_pin_cost`; 0 = none). A span is bounded by these, not by the licences
    /// (`pin_span_end_priced`). (Absent when empty, and empty unless `PIN_SPAN_PRICED` is on.)
    #[serde(default, skip_serializing_if = "no_cost")]
    pub(crate) band_ms: [f32; crate::tunables::CHUNK_BANDS],
    #[serde(default, skip_serializing_if = "no_cost_len")]
    pub(crate) band_ms_iters: [u32; crate::tunables::CHUNK_BANDS],
    #[serde(default, skip_serializing_if = "no_cost_len")]
    pub(crate) band_ms_len: [u32; crate::tunables::CHUNK_BANDS],
    /// …and the pixels the pass it was measured on ran at (`band_ms`; 0 = none): a cost measured on a
    /// 200k-pixel pin said nothing about a 1.6M-pixel one, and spans sized from it let the refresh
    /// sizing reach full-resolution pins that walked 30 frames (§7.10).
    #[serde(default, skip_serializing_if = "no_cost_len")]
    pub(crate) band_ms_px: [u32; crate::tunables::CHUNK_BANDS],
    /// The pixels each `late` pass ran at, by slot (a reading can land after the next pin started at
    /// another size). (Absent when empty.)
    #[serde(default, skip_serializing_if = "no_late_px")]
    pub(crate) late_px: [u32; LATE_MAX],
    /// The settled walk's licence past the ESCAPE ONSET (`WALK_ONSET`, `walk_onset_start`), as WORK:
    /// iterations × the pixels a pass is charged for, earned by those passes' own prices with a band
    /// licence's rule (`licence_update`) and shed with the band licences. 0 = unvisited, which opens
    /// at the floor. A band is one octave and the cost is not uniform inside one: at the 2^704 view
    /// band 10 ([131k, 262k)) is nearly free up to ~220k and ~100x dearer past it, where the pixels
    /// begin to escape, so the band's licence, earned on its cheap front, carried a 32,585-iteration
    /// pass into the dear part. ⚠Work, not iterations: the region runs on into the tail, where a
    /// few hundred thousand pixels still run and a cheap pass carries 20-30k iterations; a licence
    /// in iterations earned there opened the next sample's first dear pass at 20,608 over all 9.2M
    /// (2026-10-21). (Absent when 0.)
    #[serde(default, skip_serializing_if = "zero_u64")]
    pub(crate) onset_work: u64,
    /// Per iteration band: the pixels the settled pass that last priced its licence was charged for
    /// (`WALK_ONSET`; 0 = unknown, or priced by a pin). A licence is a count of iterations, and it
    /// holds only at the pixels it was earned over: inside band 11 at the 2^704 view the running
    /// pixels fall from ~1.4M to a few hundred thousand, so a licence earned at the band's end opened
    /// the next sample's first pass there at 29,596 iterations, 922 ms. A settled pass charged for
    /// more pixels takes the licence scaled down to the same work (`decide_walk_dispatch`); never up.
    /// (Absent when empty.)
    #[serde(default, skip_serializing_if = "no_cost_len")]
    pub(crate) band_px: [u32; crate::tunables::CHUNK_BANDS],
    /// A STEP-BOUNDED settled walk's last priced pass (`WALK_STEPS`): its step cap and its wall
    /// price, ms, which size the next pass (`walk_step_cap`). 0 = none. (Absent when 0, as in every
    /// recording made before it.)
    #[serde(default, skip_serializing_if = "zero_u32")]
    pub(crate) step_cap_last: u32,
    #[serde(default, skip_serializing_if = "zero_f64")]
    pub(crate) step_wall_last: f64,
}

/// A zero left out of a recording (`WalkLedger::step_wall_last`).
fn zero_f64(v: &f64) -> bool {
    *v == 0.0
}

/// The `band` a step-bounded pass in flight is recorded under (`WalkLedger::inflight`): not an
/// iteration band, so its price feeds `step_cap_last` / `step_wall_last` and no band's licence.
pub(crate) const STEP_PASS_BAND: u8 = u8::MAX;

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
    /// The rate whole moving frames run at (`Perf::frame_rate_now`, `Meter::frame_rate`; 0 = none
    /// measured, as on a card whose moving frames all start pins): it bounds a non-pin moving frame's
    /// pass, never the ladder's resolution. Recordings made before it read 0, which bounds nothing.
    #[serde(default)]
    pub(crate) frame_rate: f64,
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
    /// GPU readings landed for the passes parked in `ledger.late`: `(the dispatch frame the
    /// reading names, ms)` (`Perf::late_readings`).
    #[serde(default)]
    pub(crate) late_readings: Vec<(u64, f64)>,
    /// Pin passes allowed in flight (`tunables::pin_inflight_max`): with two, a pass still in flight
    /// when the next goes out is parked rather than overwritten.
    #[serde(default = "one")]
    pub(crate) pin_inflight_max: u32,
    /// The device gives no GPU timestamps: a pin pass's completion stand-in is the only price it
    /// will ever have, so it is fed at once.
    #[serde(default)]
    pub(crate) no_timestamps: bool,
    /// The growth limiter compares WORK (iterations × pixels, `WalkLedger::motion_work_last`), not
    /// iterations (`tunables::GROWTH_IN_WORK_DEFAULT`); recordings made before it read false.
    #[serde(default)]
    pub(crate) growth_in_work: bool,
    /// The adapter's occupancy knee, pixels (`calibration::knee_px`; 0 = unknown): below it a pass
    /// costs its iteration window, not pixels × iterations, so work is charged at the knee.
    #[serde(default)]
    pub(crate) knee_px: u64,
    /// A pin pass the GPU timed far under the pin target lets the next grow by the band ledger's
    /// proportional lane (`tunables::PIN_GROWTH_LANE_DEFAULT`); recordings made before it read false.
    #[serde(default)]
    pub(crate) pin_growth_lane: bool,
    /// Pin passes' GPU prices also feed the band-cost memory (`WalkLedger::band_ms`,
    /// `tunables::PIN_SPAN_PRICED_DEFAULT`), against the installed reference's length (0 = unknown);
    /// recordings made before them read false and 0.
    #[serde(default)]
    pub(crate) span_priced: bool,
    #[serde(default)]
    pub(crate) orbit_len: u32,
    /// Has the settled pass in flight COMPLETED on the GPU (`WALK_SERIALIZE`: every registration
    /// of this view's work confirmed by its completion callback, or the completion overdue past
    /// `SETTLE_COMPLETION_TIMEOUT_US`)? `None` = the quick frame decides, as before (recordings made
    /// before it, and `WALK_SERIALIZE` 0).
    #[serde(default)]
    pub(crate) walk_done: Option<bool>,
    /// The settled pass in flight's first live iteration and the pixels it was charged for
    /// (`DispatchState::inflight_lo`, `inflight_px`; 0 = unknown), and where the view's escape-onset
    /// region starts (`walk_onset_start`; 0 = unknown or `WALK_ONSET` 0): a completed pass that
    /// started in the region prices its licence too (`WalkLedger::onset_work`). Recordings made
    /// before them read 0, which prices nothing there.
    #[serde(default)]
    pub(crate) inflight_lo: u32,
    #[serde(default)]
    pub(crate) inflight_px: u64,
    #[serde(default)]
    pub(crate) onset_start: u32,
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

/// What the pricing did, for the caller's log and trace.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum WalkPriceEvent {
    /// The in-flight pass reached the lethal band before draining: the licences were shed.
    LethalShed { size: u32, band: u8, acc: f64, floor: u32, outcome: super::RetreatOutcome },
    /// A pin pass was priced (`acc` -1: a parked pass, priced by its reading when it landed;
    /// -2: one whose reading never came, priced by its completion stand-in).
    PinPriced { band: u8, size: u32, price: f64, acc: f64 },
    /// A pin pass's price was deferred to its reading: released at its completion (`stand_in` its
    /// completion stand-in), or still in flight as the next went out (`stand_in` < 0).
    PinDeferred { band: u8, size: u32, stand_in: f64 },
    /// A parked pass whose reading never came and which had no stand-in: unpriced.
    PinUnpriced { band: u8, size: u32 },
}

/// Feed one pin pass's price into the band ledger (`decide_walk_price`). A pass across several bands
/// (`pin_span_end`) charges its whole price to each band it ran in, with that band's iterations: no
/// band can earn more than its own share of the pass proves, and an over-target price cuts every
/// band it may have come from.
fn feed_pin_price(led: &mut WalkLedger, band: u8, size: u32, price: f64, target_ms: f64) {
    led.pass_dt = price;
    let lane = super::pin_fast_lane(price, target_ms);
    for (b, len) in super::span_band_lengths(band, size) {
        super::chunk_band_update_with_lane(&mut led.bands, b, len, price, target_ms, lane);
        // A pin's price is not a settled pass's: its pixels are not remembered.
        led.band_px[b.min(crate::tunables::CHUNK_BANDS - 1)] = 0;
    }
}

/// Feed one GPU-timed pin pass into the band-cost memory (`WalkLedger::band_ms`, `PIN_SPAN_PRICED`):
/// for a pass in one band, its price and its iterations; for a pass across several, its price
/// shared out in proportion to what those bands cost before, each with its own iterations
/// (`span_band_lengths`), or, where one of them had no cost yet, the whole price charged to each.
/// A zero price measures nothing: the RX 6800 XT's timestamps lag a dispatch and read 0.0 for a
/// pass that ran.
fn feed_pin_cost(led: &mut WalkLedger, band: u8, size: u32, price: f64, orbit_len: u32, px: u32) {
    if !(price > 0.0) || !price.is_finite() || size == 0 {
        return;
    }
    let parts: Vec<(usize, u32)> = super::span_band_lengths(band, size)
        .map(|(b, len)| (b.min(crate::tunables::CHUNK_BANDS - 1), len))
        .filter(|&(_, len)| len > 0)
        .collect();
    let before: f64 = parts.iter().map(|&(b, _)| f64::from(led.band_ms[b])).sum();
    let all = parts.iter().all(|&(b, _)| led.band_ms[b] > 0.0);
    for &(b, len) in &parts {
        led.band_ms[b] = if parts.len() > 1 && all && before > 0.0 {
            (f64::from(led.band_ms[b]) * price / before) as f32
        } else {
            price as f32
        };
        led.band_ms_iters[b] = len;
        led.band_ms_len[b] = orbit_len;
        led.band_ms_px[b] = px;
    }
}

/// The pass size a pin pass of `size` iterations the GPU timed at `price_ms` vouches for: `size` ×
/// the band ledger's proportional lane (`pin_fast_lane`, ×4..×16, the grown pass predicted at no
/// more than half the target), when the price is within an eighth of the target (where that lane is
/// the model's own); 0 otherwise. Only GPU timestamp readings come here: a completion stand-in once
/// read 0.0 ms for a 12.5 ms pass (W8).
fn pin_growth_iters(size: u32, price_ms: f64, target_ms: f64) -> u32 {
    if price_ms >= 0.0 && price_ms <= target_ms / 8.0 {
        size.saturating_mul(super::pin_fast_lane(price_ms, target_ms))
    } else {
        0
    }
}

/// Park a pin pass's price in `led.late`, with the pixels it ran at (`late_px`). Full: the oldest
/// stands on its stand-in, or goes unpriced.
fn park_pin_price(led: &mut WalkLedger, events: &mut Vec<WalkPriceEvent>, pass: (u32, u8, u64, f64), px: u32, target_ms: f64) {
    if let Some(k) = led.late.iter().position(Option::is_none) {
        led.late[k] = Some(pass);
        led.late_px[k] = px;
        return;
    }
    let k = (0..LATE_MAX).min_by_key(|&k| led.late[k].map_or(u64::MAX, |l| l.2)).unwrap_or(0);
    if let Some((size, band, _, stand_in)) = led.late[k] {
        if stand_in >= 0.0 {
            feed_pin_price(led, band, size, stand_in, target_ms);
            events.push(WalkPriceEvent::PinPriced { band, size, price: stand_in, acc: -2.0 });
        } else {
            events.push(WalkPriceEvent::PinUnpriced { band, size });
        }
    }
    led.late[k] = Some(pass);
    led.late_px[k] = px;
}

/// The chunk walk's PRICING and SIZING (`bp_chunk_tiling`'s, lifted): the wall-aware floor, the
/// in-flight pass's price (the wall-clock retreat into the lethal band, the drain, the band ledger),
/// the pass's step budget and the iteration step with its growth limiter. Pure.
#[must_use]
pub(crate) fn decide_walk_price(i: &WalkPriceInputs) -> WalkPriceDecision {
    use super::{
        chunk_band_retreat, chunk_band_update_with_lane, chunk_step_factor, motion_pass_steps_fixed,
        pin_pass_priceable, wall_shed_now,
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
    // accumulates every following frame interval until its GPU completion is seen
    // (`walk_done`, `WALK_SERIALIZE`) — and that sum is the pass's wall price (conservative:
    // it includes present overhead and the completion's latency, the safe direction). This
    // is the signal a saturated queue cannot silence; the fatal sessions' timestamps were
    // silent precisely because saturation never let them arm.
    // ⛔It used to be released by the first frame quicker than CHUNK_DRAIN_DT_MS, taken as
    // proof the queue had drained. It is not: the swap chain hides a pass's cost for two
    // frames (see the release below), and passes stacked four deep (2026-10-20).
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
    // ⭐…and with two in flight (rung 1d) a pass still running as the next goes out is parked the
    // same way, with no stand-in: priced by its reading, or, after twice the wait, unpriced.
    let pin_target = crate::tunables::MOTION_PIN_TARGET_MS;
    // The pixels this frame's pin pass ran at (a parked one keeps its own, `late_px`) — kept only for
    // the band-cost memory, so a ledger without it is what a recording made before it recorded.
    let pass_px = if i.span_priced {
        u32::try_from(i.spx.saturating_mul(u64::from(i.ss).pow(2))).unwrap_or(u32::MAX)
    } else {
        0
    };
    // The largest pass a GPU-timed pin price this frame vouches for (`pin_growth_iters`; 0 = none).
    let mut lane_iters = 0u32;
    for k in 0..LATE_MAX {
        let Some((size, band, sent, stand_in)) = led.late[k] else {
            continue;
        };
        let age = i.frame.saturating_sub(sent);
        let wait = crate::tunables::PIN_PRICE_WAIT_FRAMES;
        let price = match i.late_readings.iter().find(|r| r.0 == sent) {
            Some(&(_, ms)) => Some((ms, -1.0)),
            None if stand_in >= 0.0 && age >= wait => Some((stand_in, -2.0)),
            None if stand_in < 0.0 && age >= 2 * wait => {
                events.push(WalkPriceEvent::PinUnpriced { band, size });
                led.late[k] = None;
                led.late_px[k] = 0;
                None
            }
            None => None,
        };
        if let Some((price, acc)) = price {
            if acc == -1.0 {
                lane_iters = lane_iters.max(pin_growth_iters(size, price, pin_target));
                if i.span_priced {
                    let px = led.late_px[k];
                    feed_pin_cost(&mut led, band, size, price, i.orbit_len, px);
                }
            }
            feed_pin_price(&mut led, band, size, price, pin_target);
            events.push(WalkPriceEvent::PinPriced { band, size, price, acc });
            led.late[k] = None;
            led.late_px[k] = 0;
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
                led.onset_work = 0;
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
                // ⭐A SETTLED pass is released by its COMPLETION (`WALK_SERIALIZE`, `walk_done`), for
                // the reason a pin is: the swap chain hides a pass's cost for two frames, so the
                // quick interval after it proves nothing. Read "drained" at ~10 ms, passes went out
                // one a frame and up to four stood in the queue at the 2^704 view's dense band
                // (947 and 1,011 ms frames, 2026-10-20). Its price is still the wall it accumulated
                // in flight: the completion is seen a frame or two late, the safe direction.
                i.walk_done.unwrap_or(i.last_dt_ms < crate::tunables::CHUNK_DRAIN_DT_MS)
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
                    park_pin_price(&mut led, &mut events, (size, band, i.pin_pass_frame, price), pass_px, target_ms);
                    events.push(WalkPriceEvent::PinDeferred { band, size, stand_in: price });
                } else if pin_frame {
                    if i.pin_pass_gpu_ms >= 0.0 {
                        lane_iters = lane_iters.max(pin_growth_iters(size, price, target_ms));
                        if i.span_priced {
                            feed_pin_cost(&mut led, band, size, price, i.orbit_len, pass_px);
                        }
                    }
                    feed_pin_price(&mut led, band, size, price, target_ms);
                    // Traced by the caller under `tile` (`WalkPriceEvent::PinPriced`).
                    events.push(WalkPriceEvent::PinPriced { band, size, price, acc });
                } else if band == STEP_PASS_BAND {
                    // A step-bounded pass (`WALK_STEPS`): `size` is its step cap. Its price sizes the
                    // next pass of the walk, and no iteration band learns anything from it.
                    led.step_cap_last = size;
                    led.step_wall_last = price;
                } else {
                    led.pass_dt = price;
                    chunk_band_update_with_lane(&mut led.bands, band as usize, size, price, target_ms, 2);
                    // …over the pixels it was charged for (`WalkLedger::band_px`), when a completion
                    // priced it and they are known.
                    led.band_px[(band as usize).min(crate::tunables::CHUNK_BANDS - 1)] =
                        if i.walk_done.is_some() { u32::try_from(i.inflight_px).unwrap_or(u32::MAX) } else { 0 };
                    // ...and a pass that started in the escape-onset region earns that region's
                    // licence. Only a completion's price can (a quick frame's is ~10 ms whatever the
                    // pass cost).
                    // In work: the licence at the pass's own pixels, updated, and back. Against HALF
                    // the target (`WALK_ONSET_TARGET_SHARE`): the region is where the dear work is,
                    // and its cost per iteration still varies ~2× from stretch to stretch, so a
                    // licence doubled at the full target's rule ran a pass at 909 ms (2026-10-21).
                    let onset = i.onset_start;
                    if i.walk_done.is_some() && onset > 0 && i.inflight_lo >= onset && i.inflight_px > 0 {
                        let px = i.inflight_px;
                        let cur = (led.onset_work / px).min(u64::from(u32::MAX)) as u32;
                        let aim = target_ms * super::WALK_ONSET_TARGET_SHARE;
                        let next = super::licence_update(cur, size, price, aim, 2);
                        led.onset_work = u64::from(next).saturating_mul(px);
                    }
                }
                led.inflight = None;
            } else if pin_frame && !i.no_timestamps && i.pin_inflight < i.pin_inflight_max {
                // ⭐PRICE EVERY PASS (rung 1d): the next pin pass goes out this frame with this one
                // still in flight, and its dispatch rewrites the in-flight record. Park this one's
                // price for its own reading instead of losing it: at two in flight the motiontest
                // priced 26 pin passes across 70 pins, the licences barely grew, and 50 pins drifted out.
                park_pin_price(&mut led, &mut events, (size, band, i.pin_pass_frame, -1.0), pass_px, target_ms);
                events.push(WalkPriceEvent::PinDeferred { band, size, stand_in: -1.0 });
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
    // ⭐THE FRAME RATE'S BOUND (W8 rung 1c): a whole moving frame's pass carries no more iterations
    // than the rate whole moving frames run at affords in one `MOTION_PASS_MS` pass. The pass size
    // above comes from `motion_rate`, which pin passes inflate (most of a deep pin pass's pixels have
    // escaped, so its nominal steps are cheap): on the RX 6800 XT it sized the visible ladder's frame
    // at the admission, and the whole ask went out in one 112–115 ms pass every three frames. Bounded,
    // the frame's first pass stops short of the ask and the frame becomes a pin walk at the same
    // resolution. The resolution is not touched: it keeps `motion_rate` (sized from the frame rate it
    // fell 4–8× on the RTX 3080, which measures no whole moving frame and so has no frame rate).
    let budget_step = if interacting && !pin_frame && i.frame_rate > 0.0 {
        let frame_pass = motion_pass_steps_fixed(
            i.frame_rate,
            crate::tunables::MOTION_PASS_MS,
            i.pass_fixed_ms,
            tdr_steps,
            i.tdr_bootstrap_steps,
        );
        budget_step.min(((frame_pass / spx.saturating_mul(ss2).max(1)) as u32).max(floor))
    } else {
        budget_step
    };
    // GROWTH LIMITER (motion / pin passes only): at most double the previous pass. The
    // rate estimate above lags its dispatches by 2–3 frames, so a cheap sliver can prime
    // a pass 20× its size before the reading that would have said otherwise lands —
    // measured as a 257 ms pass right after a 256-iteration one. Doubling meets a cost
    // cliff with one ~2× overshoot, and the wall-clock cut halves the memory next frame.
    // ⭐IN WORK, NOT ITERATIONS (`growth_in_work`, 2026-10-17). "At most double the previous pass"
    // means its work: the moving frames between pins run at full resolution and the pins at a
    // fraction of it, and one memory in iterations served both. Recorded through the 2^93 switch: a
    // moving frame at 1,627,654 px left 120 iterations (1.95e8 steps) and the next pin, at 25,208
    // px, restarted its doubling at 240 iterations — 6e6 steps, 1/32 of the frame before; the other
    // way round, a full-resolution pass after a small one took twice the iterations at 100× the
    // pixels. A mode switch still zeroes the memory (a floatexp step costs ~100× a df32 one).
    // ⚠Work is charged at no fewer pixels than the occupancy KNEE (`knee_px`), as the dispatch
    // ceiling charges it: below ~262k px (RTX 3080) a pass costs its iteration window, so raw
    // pixels × iterations made a 145×110 pin look 16× cheaper than it is. Counted raw, its first
    // pass opened at 3,773 iterations after a full-resolution frame and df32 pins at 2^73–2^88
    // stalled for up to 119 frames (held 8.95 octaves, one run of four, 2026-10-17).
    // ⭐…and a pin pass the GPU timed far under the pin target lets the next grow by the band
    // ledger's own proportional lane (`pin_growth_lane`): the first floatexp pin's passes priced
    // 0.16–0.85 ms against 20 while the ×2 a frame held them for five frames. The rate model (its
    // own per mode: floatexp starts from the bootstrap at the switch), the band licences and the
    // band clip still bound every pass. The licences df32 earned are LARGER than floatexp's, so
    // they never bind first; clearing them at the switch was tried and gave the lane nothing to
    // grow into (every band reopened at its 256 floor: the first floatexp pin took 256-iteration
    // passes while the lane allowed 7,899).
    let budget_step = if interacting || pin_frame {
        let px = spx.saturating_mul(ss2).max(i.knee_px).max(1);
        let doubling = if i.growth_in_work {
            match led.motion_work_last {
                0 => crate::tunables::MOTION_STEP_OPEN,
                w => (w.saturating_mul(2) / px).min(u64::from(u32::MAX)) as u32,
            }
        } else {
            match led.motion_step_last {
                0 => crate::tunables::MOTION_STEP_OPEN,
                last => last.saturating_mul(2),
            }
        };
        let lim = if pin_frame && i.pin_growth_lane { doubling.max(lane_iters) } else { doubling };
        let s = budget_step.min(lim.max(floor));
        led.motion_step_last = s;
        if i.growth_in_work {
            led.motion_work_last = u64::from(s).saturating_mul(px);
        }
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
    /// A settled mode-2 walk runs STEP-BOUNDED passes (`WALK_STEPS`): it always counts its running
    /// pixels, and its readings' cursor (every pass ends at `walk_end`) is ahead of its own until it
    /// completes. Recordings made before it read false.
    #[serde(default)]
    pub(crate) step_walk: bool,
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
    // this stage did not see, so its bound describes another walk — drop it. (A step-bounded walk's
    // passes all end at `walk_end`, so its readings are ahead of it by design; the walk number alone
    // says whose they are.)
    let step_walk = i.step_walk && fe_walk && !interacting && !pin_frame;
    if !step_walk && st.running.is_some_and(|(_, at, _)| cur < at) {
        st.running = None;
    }
    // A settled mode-2 walk counts its running pixels (`walk_charged_px` sizes by them; a
    // step-bounded walk ends on that count).
    if fe_walk && !interacting && !pin_frame && (i.chunk_charge == 1 || step_walk) {
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
    /// The settled pass in flight's first live iteration and the pixels it was charged for
    /// (`WalkPriceInputs::inflight_lo`, `inflight_px`), written for every pass and 0 while the view's
    /// escape-onset region is unknown (`WalkDispatchInputs::onset_start`): its price records the pixels
    /// in its band's `band_px`, and the onset region's price reads both. Recordings made before them
    /// read 0, and a 0 is left out, so they replay as recorded.
    #[serde(default, skip_serializing_if = "zero_u32")]
    pub(crate) inflight_lo: u32,
    #[serde(default, skip_serializing_if = "zero_u64")]
    pub(crate) inflight_px: u64,
    /// A step-bounded settled walk (`WALK_STEPS`): the step cap of its last pass, which a held frame
    /// re-sends, and the caps of all its passes so far, summed. Absent when 0, as in every recording
    /// made before them.
    #[serde(default, skip_serializing_if = "zero_u32")]
    pub(crate) step_cap: u32,
    #[serde(default, skip_serializing_if = "zero_u64")]
    pub(crate) step_sum: u64,
}

/// A zero left out of a recording (`DispatchState::inflight_lo`).
fn zero_u32(v: &u32) -> bool {
    *v == 0
}

/// A zero left out of a recording (`DispatchState::inflight_px`, `WalkLedger::onset_work`).
fn zero_u64(v: &u64) -> bool {
    *v == 0
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
    /// Pin passes allowed in flight (`tunables::pin_inflight_max`); recordings made before it read 1.
    #[serde(default = "one")]
    pub(crate) pin_inflight_max: u32,
    /// The share of one pass a pin pass across priced bands may take (`tunables::PIN_BAND_SPAN_DEFAULT`,
    /// `pin_span_end`; 0 = off); recordings made before it read 0, the one-band pass they recorded.
    #[serde(default)]
    pub(crate) pin_span_share: f64,
    /// A span is bounded by its bands' measured costs (`tunables::PIN_SPAN_PRICED_DEFAULT`,
    /// `pin_span_end_priced`): those costs, the iterations each covered and the reference length each
    /// was measured against (`WalkLedger::band_ms`); recordings made before them read false and none.
    #[serde(default)]
    pub(crate) pin_span_priced: bool,
    #[serde(default, skip_serializing_if = "no_cost")]
    pub(crate) band_ms: [f32; crate::tunables::CHUNK_BANDS],
    #[serde(default, skip_serializing_if = "no_cost_len")]
    pub(crate) band_ms_iters: [u32; crate::tunables::CHUNK_BANDS],
    #[serde(default, skip_serializing_if = "no_cost_len")]
    pub(crate) band_ms_len: [u32; crate::tunables::CHUNK_BANDS],
    #[serde(default, skip_serializing_if = "no_cost_len")]
    pub(crate) band_ms_px: [u32; crate::tunables::CHUNK_BANDS],
    /// Where the view's escape-onset region starts, and the settled walk's licence past it, in work
    /// (`WalkPriceInputs::onset_start`, `WalkLedger::onset_work`, `WALK_ONSET`). Recordings made
    /// before them read 0: no region.
    #[serde(default)]
    pub(crate) onset_start: u32,
    #[serde(default)]
    pub(crate) onset_work: u64,
    /// The pixels each band's licence was earned over by a settled pass (`WalkLedger::band_px`;
    /// none in recordings made before it, which scale nothing).
    #[serde(default, skip_serializing_if = "no_cost_len")]
    pub(crate) band_px: [u32; crate::tunables::CHUNK_BANDS],
    /// The overridable tunables it reads.
    pub(crate) pin_timeout_scaled: u64,
    pub(crate) chunk_charge: u64,
    pub(crate) chunk_shuffle: u64,
    /// A settled mode-2 walk runs STEP-BOUNDED passes (`WALK_STEPS`), each sized for
    /// `step_target_ms` from the walk's last priced pass (`WalkLedger::step_cap_last`,
    /// `step_wall_last`). Recordings made before it read false and nothing.
    #[serde(default)]
    pub(crate) step_walk: bool,
    #[serde(default)]
    pub(crate) step_target_ms: f64,
    #[serde(default)]
    pub(crate) step_cap_last: u32,
    #[serde(default)]
    pub(crate) step_wall_last: f64,
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
    /// A settled pass in a view with an escape-onset region (`WALK_ONSET`): the iterations each of
    /// its bounds allowed — the budget's step (over the pixels charged), the start band's licence
    /// (scaled by `band_px`), the onset region's licence (0 = not in it) — and the pixels charged,
    /// for the `tile` trace. (Absent otherwise, and in recordings made before it.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) bounds: Option<(u32, u32, u32, u64)>,
    /// A step-bounded pass's steps per pixel (`WALK_STEPS`; `MandelbrotParams::step_cap`); 0 = an
    /// iteration window. (Absent when 0.)
    #[serde(default, skip_serializing_if = "zero_u32")]
    pub(crate) step_cap: u32,
}

/// What the dispatch did, for the caller's log and trace.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum DispatchEvent {
    /// The pin gate escaped on the wall clock (`n`: the view's escapes so far).
    PinEscape { age_us: u64, limit_us: u64, registered: bool, n: u32 },
    /// The pass was charged for the pixels still running, not the frame.
    Charge { running: u32, charged: u64, all_px: u64, from: u32, to: u32 },
    /// A pin pass ran on through bands bounded by their measured costs (`PIN_SPAN_PRICED`): its
    /// iterations, its work (iterations × the pixels the growth limiter charges) and the cost its
    /// bands predicted, ms. The span vouches for its size, so the caller's growth limiter remembers
    /// it: without that the next band restarted its doubling from the pass before the span (at
    /// 2^260, 1,024 after a 16,128-iteration span) and could never cover a band whole to span again.
    PricedSpan { size: u32, work: u64, pred_ms: f64 },
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
    let mut bounds = None;
    // ⭐A STEP-BOUNDED SETTLED WALK (`WALK_STEPS`, mode 2): every pass runs the whole `[·, walk_end)`
    // and stops each pixel after `step_cap` executed steps, the export's and a worker's passes
    // (`LiveTwin::walk_steps`). An iteration window cannot be priced where the BLA skips: a window
    // grown on a cheap stretch crossed into one where nothing skipped whole (978 ms after ~150 ms
    // at the user's 2^60 view, `BAND_FLOATEXP`, 2026-10-30). A step costs about the same skipping or
    // not, and every pixel still running ran the whole last cap, so a pass that follows the last
    // one's wall does at most twice their work.
    let step_walk = i.step_walk && fe_walk && !interacting && !pin_frame && walk_end > 2;
    let mut step_cap = 0u32;
    // It ends when a reading of it says no pixel runs (a pixel never restarts within a walk, so any
    // pass's zero is the walk's)…
    let cur = if step_walk && cur < walk_end && st.idx > 0 && i.running.is_some_and(|(w, _, n)| w == i.epoch && n == 0) {
        st.cursor = walk_end;
        walk_end
    } else {
        cur
    };
    // …or once its passes, all run (none in flight), have given every pixel as many steps as the walk
    // has iterations: a step advances a pixel at least one iteration (a BLA skip 2^l, a renormalized
    // one a period), so none can still run. A reading that never comes (an unarmed readback, a
    // harness with no event loop) cannot leave the walk unfinished…
    let cur = if step_walk
        && cur < walk_end
        && st.idx > 0
        && st.inflight.is_none()
        && st.step_sum >= u64::from(walk_end.saturating_sub(walk_floor))
    {
        st.cursor = walk_end;
        walk_end
    } else {
        cur
    };
    // …and one that has not in this many passes is counting wrong: ended rather than left spinning.
    let cur = if step_walk && cur < walk_end && st.idx > super::STEP_WALK_MAX_PASSES {
        st.cursor = walk_end;
        walk_end
    } else {
        cur
    };
    if cur < walk_end && step_walk {
        // Price-serialized like the window walk: one unpriced pass in flight, a held frame re-sends it.
        if st.inflight.is_some() && st.last_range.is_some() {
            chunk_range = st.last_range;
            chunk_idx = st.idx;
            step_cap = st.step_cap;
            st.pending = true;
        } else {
            let all_px = spx.saturating_mul(ss2).max(1);
            let fresh = st.idx == 0;
            let active = if fresh {
                all_px
            } else {
                i.running.filter(|&(w, _, _)| w == i.epoch).map_or(all_px, |(_, _, n)| u64::from(n).max(1))
            };
            // A pass is CHARGED for the pixels still running, never fewer than the occupancy knee, and
            // for the whole frame when the knee is unknown: the window walk's charge
            // (`walk_charged_px`). Below the knee a pass is latency-bound, not cheaper per step, and its
            // reading must say so: charged at its running pixels, a thin tail's pass (2 pixels × 8,160
            // steps in 14 ms) read as a rate a hundred times too slow, dragged the mode's pessimistic
            // rate down, and with it the opening guess the dead-man falls to — 4e6 steps against
            // window walks' ~3e8 (`--recordtest`, RX 6800 XT, dev.63/64, 2026-10-31).
            let charged = if i.knee_px > 0 { active.max(i.knee_px).min(all_px) } else { all_px };
            // …and never past the pass budget (`pass_steps`: the admission, scaled by the last window
            // pass's price) over the pixels charged, the bound a window pass has over its pixels: an
            // untrusted budget opens at its guess, a retreat shrinks every pass after it.
            let budget_cap = u32::try_from(i.pass_steps / charged.max(1)).unwrap_or(u32::MAX).max(super::STEP_WALK_MIN_CAP);
            let cap = super::walk_step_cap(fresh, all_px, (i.step_cap_last, i.step_wall_last), i.step_target_ms).min(budget_cap);
            // The opening pass starts from scratch (the series seeds it); to a later one any start
            // above 0 means "resume from the state", alternating so the GPU's dedupe never mistakes
            // the next pass for the last.
            let lo = if fresh { 0 } else { 1 + (st.idx & 1) };
            chunk_range = Some([lo, walk_end]);
            chunk_idx = st.idx;
            st.cursor = cur.max(1);
            st.idx = chunk_idx.wrapping_add(1);
            st.pending = true;
            st.last_range = chunk_range;
            st.inflight = Some((cap, STEP_PASS_BAND, 0.0, false));
            st.inflight_lo = 0;
            st.inflight_px = 0;
            st.charged_px = charged;
            st.fe_steps_last = charged.saturating_mul(u64::from(cap));
            st.step_cap = cap;
            st.step_sum = if fresh { u64::from(cap) } else { st.step_sum.saturating_add(u64::from(cap)) };
            step_cap = cap;
            chunk_pass = true;
        }
    } else if cur < walk_end {
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
            st.pin_inflight >= i.pin_inflight_max
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
            // A settled pass charged for more pixels than its band's licence was earned over takes
            // the licence scaled down to the same work (`WalkLedger::band_px`): a count of iterations
            // holds only at the pixels it was priced at. Never scaled up.
            let licence = {
                let b = (band as usize).min(crate::tunables::CHUNK_BANDS - 1);
                let (earned, at) = (u64::from(i.bands[b]), u64::from(i.band_px[b]));
                if !interacting && !pin_frame && earned > 0 && at > 0 && charged > at {
                    let scaled = ((earned * at) / charged).max(1) as u32;
                    licence.min(scaled.max(pin_floor))
                } else {
                    licence
                }
            };
            let step = budget_step.min(licence);
            // ⭐…and, settled, past the view's ESCAPE ONSET no more than that region's own licence
            // (`WALK_ONSET`, `WalkLedger::onset_licence`; the floor while unvisited): an octave band
            // is one licence, and its cost is not uniform where the pixels begin to escape.
            let onset = i.onset_start;
            let onset_licence = if !interacting && !pin_frame && onset > 0 && real_lo >= onset {
                // The licence's work over the pixels this pass is charged for.
                let licence = (i.onset_work / charged.max(1)).min(u64::from(u32::MAX)) as u32;
                chunk_band_license(&[licence; crate::tunables::CHUNK_BANDS], 0, pin_floor)
            } else {
                0
            };
            if !interacting && !pin_frame && onset > 0 {
                bounds = Some((budget_step, licence, onset_licence, charged));
            }
            let step = if onset_licence > 0 { step.min(onset_licence) } else { step };
            let step = if i.chunk_shuffle == 1 && !interacting && !pin_frame {
                // Test hook (`CHUNK_SHUFFLE`): an uneven window, the same every run.
                let h = (st.idx as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 60;
                ((step as u64 * (16 + h) / 32) as u32).max(1)
            } else {
                step
            };
            let step = if pin_frame { step.max(pin_floor.min(gpu_iter.max(1))) } else { step };
            let end = real_lo.saturating_add(step).min(walk_end);
            // …and a settled pass that starts short of the onset region stops at its edge, so the
            // region is entered under its own licence.
            let end = if !interacting && !pin_frame && onset > 0 && real_lo < onset { end.min(onset) } else { end };
            // A PIN pass stops at the end of the band it started in. The licence is the
            // START band's, and a pass that runs on into unvisited bands carries it there
            // — traced: a 21,480-iteration pass licensed in [8k,16k) ran through the
            // orbit's rebase storm at cur ≈ orbit_len (10–70× the cold rate) and cost
            // 380 ms. Entering each band with that band's own licence (the floor when
            // unvisited) is what the ledger was built for; it costs a pin at most one
            // extra pass per band. Output is unchanged by pass boundaries (the chunked
            // iterate's bit-identity contract).
            let end = if pin_frame { end.min(chunk_band_end(real_lo)).max(real_lo + 1).min(walk_end) } else { end };
            // ⭐…and goes on through the next bands only while each has prices of its own and their
            // licences, shared, still fit one pass (`pin_span_end`): a pin walked one octave band per
            // frame however cheap the bands were. The span's bound is the rate model's pass size,
            // not the growth limiter's doubling, which tracks the bands' own widths and so would undo
            // it; every band past the first is bounded by its own prices.
            let (end, band) = if pin_frame && i.pin_span_share > 0.0 {
                let step_cap = ((pass_steps / spx.saturating_mul(ss2).max(1)) as u32).max(budget_step);
                if i.pin_span_priced {
                    // …bounded by what those bands COST, not by their licences (`PIN_SPAN_PRICED`).
                    let (span, pred_ms) = super::pin_span_end_priced(
                        &super::BandCosts {
                            ms: &i.band_ms,
                            iters: &i.band_ms_iters,
                            len: &i.band_ms_len,
                            px: &i.band_ms_px,
                            px_now: spx.saturating_mul(ss2),
                            knee: i.knee_px,
                        },
                        &i.bands,
                        real_lo,
                        end,
                        licence,
                        walk_end,
                        step_cap,
                        i.orbit_len,
                        i.pin_span_share * crate::tunables::MOTION_PIN_TARGET_MS,
                    );
                    if span.0 > end {
                        let size = span.0 - real_lo;
                        let px = spx.saturating_mul(ss2).max(i.knee_px).max(1);
                        events.push(DispatchEvent::PricedSpan { size, work: u64::from(size).saturating_mul(px), pred_ms });
                    }
                    span
                } else {
                    super::pin_span_end(
                        &i.bands,
                        real_lo,
                        end,
                        licence,
                        pin_floor,
                        walk_end,
                        step_cap,
                        i.orbit_len,
                        i.pin_span_share,
                    )
                }
            } else {
                (end, band)
            };
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
                // ⚠Written for EVERY pass, never left from an earlier one: its price writes these
                // pixels into its band's `band_px`. Set only with an onset known, a walk before its
                // first escape priced the pixels an EARLIER pass had left (a pin's, a fraction of the
                // panel), and each doubled licence was scaled straight back to the floor: 256-iteration
                // passes for 9.5 s at the user's 2^70 view (2026-10-30,
                // `a_settled_walk_earns_its_licence_before_the_escape_onset_is_known`). With no onset
                // known they are 0, unknown: before its first escape every pixel of a walk runs, so a
                // pass is charged them all and no later pass can be charged more.
                let known = i.onset_start > 0;
                st.inflight_lo = if known { real_lo } else { 0 };
                st.inflight_px = if known { charged } else { 0 };
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
    // (A step-bounded pass ends at `walk_end` with pixels still running: a partial like a window's.)
    st.norm_complete_submit = (interacting && !pin_frame)
        || (step_cap == 0 && chunk_range.is_none_or(|[_, end]| end >= walk_end));
    WalkDispatchDecision { chunk_range, chunk_idx, pass: chunk_pass, state: st, pin_dispatched, events, bounds, step_cap }
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

#[cfg(test)]
#[path = "../../tests/unit/render/decide/tests.rs"]
mod tests;
