//! What a live frame IS, and the identity of the iteration walk it belongs to
//! (design/live-render-robustness.md §5.3, W4).
//!
//! `build_params` produces nine kinds of frame through staged mutation, and nothing used to name
//! which one a frame was: the record carried a dozen flags (`interacting`, `pin_frame`, `tiled`,
//! `chunked`, `present`, …) whose combinations a reader had to decode, and the decisions that
//! depend on the kind — the dispatch stamp, the held-frame floor — each re-inferred it from a
//! proxy. [`FrameKind`] is decided once, from the values the frame actually handed the GPU, and
//! recorded (frame record `frame_kind`, schema 10).
//!
//! [`WalkSig`] is the walk's identity: everything that shapes a resumable walk, so that any change
//! to it restarts the walk rather than resuming per-pixel state computed for another view. It is
//! built in ONE place ([`WalkSig::of`]); its fields are private, so a walk identity cannot be
//! assembled by hand somewhere that forgets an input.

use super::*;

/// What a live frame did, decided from what it handed the GPU (not from what was requested).
///
/// The codes are the frame record's `frame_kind` and are part of the schema: append, never
/// renumber. 0 is "not built this frame".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum FrameKind {
    /// A hold with nothing behind it: no frame has been rendered for this view yet (a cold start
    /// waiting for its reference). The display serves an empty texture.
    Placeholder = 1,
    /// Re-samples the held texture under a transform — a pan drag, the reuse hold between moving
    /// refreshes, a freeze while a reference catches up. Iterates nothing.
    Reproject = 2,
    /// One pass of a pinned refresh (a chunked walk or a split), composing under the held
    /// snapshot while the view moves. Its first pass is the frame that started the pin.
    PinWalk = 3,
    /// A moving frame that iterates and is shown as it is: a one-pass motion preview or a live
    /// refresh.
    Motion = 4,
    /// A settled frame that dispatches nothing new: the texture already holds it (an unchanged
    /// key, a walk's tail, a grid waiting its turn).
    Hold = 5,
    /// One pass of a settled chunked walk.
    SettledWalk = 6,
    /// One tile of a settled grid.
    Tile = 7,
    /// A settled frame rendering a progressive-supersampling sample (a sub-pixel-shifted view).
    Accum = 8,
    /// A settled frame rendered whole, in one dispatch.
    Settled = 9,
}

impl FrameKind {
    /// The record's code.
    pub(crate) fn code(self) -> u8 {
        self as u8
    }

    /// The record's codes, by name. `scripts/framelog.py`'s `KIND` table must say the same, and a
    /// test holds it to that.
    #[cfg(test)]
    pub(crate) const NAMES: [(u8, &'static str); 9] = [
        (1, "placeholder"),
        (2, "reproject"),
        (3, "pin-walk"),
        (4, "motion"),
        (5, "hold"),
        (6, "settled-walk"),
        (7, "tile"),
        (8, "accum"),
        (9, "settled"),
    ];
}

/// The facts [`classify`] decides a frame's kind from: what the frame handed the GPU.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FrameFacts {
    /// The colour pass re-samples the held texture (`MandelbrotParams::reproject`).
    pub(crate) reproject: bool,
    /// A frame has been rendered for this view (`frozen_center`), so a reprojection has
    /// something to show.
    pub(crate) has_frozen: bool,
    pub(crate) interacting: bool,
    /// A pass of a pinned refresh: a pin frame, or the frame that started the pin.
    pub(crate) pin_pass: bool,
    /// A real pass goes out this frame (not deduped, not a zero-area hold rect).
    pub(crate) dispatches: bool,
    pub(crate) tile: bool,
    pub(crate) chunked: bool,
    /// The dispatch renders a supersampling sample (a non-zero jitter).
    pub(crate) accum: bool,
}

/// A frame's kind. Total and mutually exclusive: the first rule that applies decides.
pub(crate) fn classify(f: FrameFacts) -> FrameKind {
    if f.reproject {
        if f.has_frozen { FrameKind::Reproject } else { FrameKind::Placeholder }
    } else if f.pin_pass {
        FrameKind::PinWalk
    } else if f.interacting {
        FrameKind::Motion
    } else if !f.dispatches {
        FrameKind::Hold
    } else if f.accum {
        FrameKind::Accum
    } else if f.tile {
        FrameKind::Tile
    } else if f.chunked {
        FrameKind::SettledWalk
    } else {
        FrameKind::Settled
    }
}

/// The iteration texture a live frame renders at.
///
/// ⭐Constructed in ONE place — [`Resolution::panel`], at the top of `build_params`, from the panel
/// it is handed — and changed only by this module's arithmetic ([`Resolution::scaled`]); a pin and a
/// settle grid carry the one they were formed at. [`WalkSig::of`] takes a `Resolution`, so the
/// resolution a walk's identity carries can only have come through here: §5.3's B7, "nothing else can
/// restart a walk", held by the type rather than by review — resolution is part of the walk's
/// identity, so every place that may change it is a place that may restart the walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Resolution([u32; 2]);

impl Resolution {
    /// The panel, as `build_params` is handed it: the one entry.
    pub(crate) const fn panel(px: [u32; 2]) -> Resolution {
        Resolution(px)
    }

    /// Scaled by `f` (below 1 shrinks), each side floored at 16 px: the motion scale, the live
    /// refresh's pricing size and the budget fit all size a frame this way.
    pub(crate) fn scaled(self, f: f64) -> Resolution {
        Resolution([((self.0[0] as f64 * f) as u32).max(16), ((self.0[1] as f64 * f) as u32).max(16)])
    }

    /// The size, for the GPU and for arithmetic.
    pub(crate) fn get(self) -> [u32; 2] {
        self.0
    }

    /// Its pixel count.
    pub(crate) fn px(self) -> u64 {
        (self.0[0] as u64) * (self.0[1] as u64)
    }
}

impl std::ops::Index<usize> for Resolution {
    type Output = u32;
    fn index(&self, i: usize) -> &u32 {
        &self.0[i]
    }
}

/// A resumable walk's identity. A walk RESUMES its per-pixel state only while this is unchanged,
/// so it must change whenever anything that shapes the walk does — and a sig that misses an input
/// resumes a walk against a view it was not computed for (the translucent copy of another location,
/// 2026-09-16; a formula switch re-showing the previous formula; a series refined mid-walk).
///
/// ⭐[`WalkSig::of`] is the ONLY constructor, and the fields are private to this module: a walk
/// identity assembled anywhere else would be the place an input gets forgotten.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) struct WalkSig {
    /// The view, the reference length, the exact position, the settings and the series seed,
    /// folded together (`bp_chunk_tiling`'s `view_bits`), with any supersampling jitter.
    view: u64,
    /// The iteration ask the walk targets.
    ask: u32,
    /// The texture it iterates, and its supersampling factor.
    res: [u32; 2],
    ss: u32,
}

impl WalkSig {
    /// No walk: fresh state, or a harness forcing a restart. A zero sig says nothing about the view.
    pub(crate) const NONE: WalkSig = WalkSig { view: 0, ask: 0, res: [0, 0], ss: 0 };

    /// The walk identity of a view. `jitter_bits` is the supersampling sample's sub-pixel shift
    /// (0 for the jitter-free walk VIEW, which keys the band ledger): each sample is a shifted view,
    /// so a new one must restart the walk.
    pub(crate) const fn of(view_bits: u64, jitter_bits: u64, ask: u32, res: Resolution, ss: u32) -> WalkSig {
        WalkSig { view: view_bits ^ jitter_bits.rotate_left(7), ask, res: res.0, ss }
    }

    /// The iteration ask the walk targets.
    pub(crate) fn ask(self) -> u32 {
        self.ask
    }

    /// The texture it iterates.
    pub(crate) fn res(self) -> [u32; 2] {
        self.res
    }

    pub(crate) fn ss(self) -> u32 {
        self.ss
    }

    /// One word for the frame record: equal sigs give equal words; a change in any part changes
    /// it (up to a hash collision).
    pub(crate) fn word(self) -> u64 {
        if self == WalkSig::NONE {
            return 0;
        }
        let mut h = self.view;
        for x in [self.ask as u64, ((self.res[0] as u64) << 32) | self.res[1] as u64, self.ss as u64] {
            h = (h ^ x).wrapping_mul(0x0000_0100_0000_01B3).rotate_left(29);
        }
        h | 1
    }
}

/// Did starting a new walk throw progress away? The walk being replaced had advanced past its
/// first pass (`cursor > 0`) and not reached its end. A moving frame's one-pass preview never
/// commits its cursor, so restarting it discards nothing; a completed walk has nothing left to
/// lose. What is left is the W4 regression shape: a settled walk or a pinned refresh discarded
/// part-way — by a resolution rung, a jitter, an abandon — and the frame shows its blank start.
pub(crate) fn walk_restart_discards(prev_cursor: u32, prev_walk_end: u32) -> bool {
    prev_cursor > 0 && prev_cursor < prev_walk_end
}

/// A JUMP IN is a frame that zooms in by more than this many octaves... ([`is_jump_in`]).
pub(crate) const JUMP_OCT_PER_FRAME: f64 = 1.5;
/// ...and by more than this many times the previous frame's step: a step from rest, not a rate.
pub(crate) const JUMP_SUDDEN: f64 = 4.0;

/// Did this frame's view JUMP in? `d_l2` is the frame's change in log2 magnification (positive =
/// in), `prev_step` the previous frame's |change| (NaN on a view's first frame, which is no jump).
///
/// ⚠Not a speed threshold. A continuous zoom can move any distance in a frame — the Home glide is a
/// smoothstep over at most 9 s, so from 1e31 it peaks near 0.3 octaves a frame and from 1e30000 at
/// hundreds, and a tour plays at whatever rate its keyframes ask. What a jump has that a glide does
/// not is SUDDENNESS: click-to-zoom, a box zoom, a bookmark, Go to and undo move the view from rest
/// in one frame, while every glide ramps (the Space glide and the auto-zoom ease their speed, the
/// Home glide and tours are eased curves). And only a jump IN matters: the floor bounds
/// magnification, and a zoom-out shows the held frame shrunk (scale > 1), which no floor touches.
pub(crate) fn is_jump_in(d_l2: f64, prev_step: f64) -> bool {
    let prev = if prev_step.is_finite() { prev_step } else { 0.0 };
    d_l2 > JUMP_OCT_PER_FRAME && d_l2 > JUMP_SUDDEN * prev
}

/// ⭐U39: does the HELD frame now stand across a jump? This replaces "no zoom velocity means a
/// jump", which only a Space glide could ever answer correctly — `zoom_vel` is written by Space
/// alone, so every auto-zoom dive, wheel zoom, Home glide and tour took the jump floor, and a held
/// frame that fell more than 3 octaves behind such a zoom was shown at the clamped scale: the wrong
/// magnification, translated too far, sliding against the moving view (`HELD_JUMP_MAX_OCT`'s own
/// doc says why a moving frame must keep its true scale). Measured on `--motiontest`'s Home glide
/// before the change: every moving frame took the jump floor, and it bound on 7 of them.
///
/// Sticky: set by a frame that jumped in ([`is_jump_in`]), kept while that held frame stays on
/// screen — after a jump the view is static, and a clamped transform has nothing to slide against —
/// and cleared when a new frame is latched as the held one (`latched`), since that frame is at the
/// new view.
pub(crate) fn hold_jump_next(prev: bool, jumped_in: bool, latched: bool) -> bool {
    if latched {
        false
    } else {
        prev || jumped_in
    }
}

/// Which bound a held frame's magnification took this frame (frame record `hold_floor`; U39):
/// 0 none (no hold transform), 1 the dive floor, 2 the jump floor without binding, 3 the jump
/// floor BINDING — the held frame is shown at less magnification than the view's, so it is the
/// wrong scale on screen — and 4 the dive floor with the held frame magnified PAST the jump floor
/// (more than `HELD_JUMP_MAX_OCT` behind: true scale, but a big flat patch; after a jump it would
/// be the 100× patch the jump floor exists to prevent). Pending per frame; the highest code wins.
pub(crate) mod hold_floor {
    // 0 (none) is the record's default: no hold transform was taken this frame.
    pub(crate) const DIVE: u8 = 1;
    pub(crate) const JUMP: u8 = 2;
    pub(crate) const JUMP_BOUND: u8 = 3;
    pub(crate) const DIVE_DEEP: u8 = 4;
}

/// The held frame's reprojection scale for `gap_oct` octaves of zoom since it was rendered
/// (`frozen_l2 − l2_now`: negative as the view dives in), bounded by [`super::hold_scale_floor`],
/// and the [`hold_floor`] code saying which bound applied.
pub(crate) fn hold_scale(gap_oct: f64, jumped: bool) -> (f32, u8) {
    let floor = super::hold_scale_floor(jumped);
    let raw = (gap_oct as f32).exp2();
    let scale = raw.clamp(floor, 1.099_512e12);
    let code = if !jumped {
        if raw < super::hold_scale_floor(true) { hold_floor::DIVE_DEEP } else { hold_floor::DIVE }
    } else if raw < floor {
        hold_floor::JUMP_BOUND
    } else {
        hold_floor::JUMP
    };
    (scale, code)
}

/// The moving frame's scale, as `build_params` sizes it: before the visible-frame ladder (the scale a
/// live refresh renders at) and after it.
pub(crate) struct MovingScale {
    pub(crate) motion_scale: f64,
    pub(crate) res_scale: f64,
}

/// A live refresh (and not a split pass) renders at the scale before the ladder, which sizes a
/// frame for ONE chunk pass; anything else at the ladder's (see `bp_moving_scale`).
pub(crate) fn refresh_scale(live_single_pass: bool, motion_scale: f64, res_scale: f64) -> f64 {
    if live_single_pass { motion_scale } else { res_scale }
}

impl crate::FractadyneApp {
    /// `build_params` stage (W4, §5.3): the MOVING frame's scale — the work budget's bound, the
    /// rate-aware refresh cap, the measured AIMD (`Perf::motion_res`, per view) and the
    /// visible-frame ladder (`Perf::visible_res`). Here, beside `Resolution`, because every one of
    /// them sizes the resolution a walk's identity carries: a writer of any of them outside this
    /// module is a place that can restart a walk unseen. Body moved verbatim from `build_params`;
    /// "above" and "below" in its notes are that function's.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn bp_moving_scale(
        &mut self,
        px: u64,
        gpu_iter: u32,
        budget: u64,
        interacting: bool,
        pin_frame: bool,
        view_id: u32,
        fractal: crate::FractalKind,
        julia: bool,
        magnification: f64,
        is_pert: bool,
        resolution: Resolution,
    ) -> MovingScale {
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
        // deep perturbation motion is affected; shallow/settled keep the budget scale. The AIMD step
        // runs once per frame (view 0); both views read the shared scale.
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
        let rate_res_cap = if interacting && !pin_frame && !self.prefer_detail_effective() {
            let vb = (view_id as usize).min(1);
            // Only a frame the chunk path can serve is spread over passes (the pin); an aux
            // colouring, a formula past the chunk shaders' scope or a device without the state
            // targets renders each refresh as ONE dispatch, and then one pass is all there is.
            let chunk_mode =
                self.render_mode(fractal, julia, magnification);
            let can_chunk = (chunk_mode.is_direct()
                || chunk_mode == RenderMode::Df32Pert
                || (chunk_mode == RenderMode::Floatexp && self.perf.chunk_fe_ok))
                && self.resumable(fractal)
                && !self.coloring.color_method.needs_aux()
                && self.perf.chunk_ok;
            // A PINNED refresh spans many frames, so no single frame interval prices it; an
            // unpinned one IS this frame, and the AIMD below prices it from its own interval.
            let pinned_refresh = is_pert && can_chunk;
            let frames_allowed = refresh_passes_allowed(
                self.perf.zoom_oct_s,
                crate::tunables::HELD_MAX_OCT,
                crate::tunables::REFRESH_TARGET_S,
                self.perf.frame_ms.max(1.0) / 1000.0,
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
            let pin_measured = (pinned_refresh && self.perf.pin_frames_last[vb] > 0).then(|| {
                let took = self.perf.pin_frames_last[vb] as f64;
                let want = frames_allowed.max(1) as f64;
                let scale = (want / took).sqrt().min(1.25);
                (self.perf.pin_res_last[vb] * scale)
                    .clamp(self.render_cfg.min_motion_res as f64, 1.0)
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
                (!pinned_refresh && self.perf.motion_res_measured[vb]).then_some(1.0)
            });
            let cap = measured.unwrap_or_else(|| {
                // Nothing measured in this regime yet — the opening guess, so the first refresh
                // after a jump or a mode switch is not a native-resolution TDR-sized dispatch.
                let pass = motion_pass_steps(
                    self.perf.motion_rate_now(vb),
                    crate::tunables::MOTION_PASS_MS,
                    u64::MAX,
                    crate::tunables::cost().tdr_bootstrap_steps,
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
                    (resolution[0] as u64) * (resolution[1] as u64),
                    self.render_cfg.min_motion_res as f64,
                )
            });
            if view_id == 0 && crate::diag::trace_on("gpu") {
                crate::diag::trace(
                    "gpu",
                    format!(
                        "res-cap f={} cap={cap:.3} from={} pinned={pinned_refresh} \
                         aimd={:.3} allowed={frames_allowed} pin_took={} pin_res={:.3} ask={gpu_iter}",
                        self.perf.frame_idx,
                        if measured.is_some() { "measured" } else { "model" },
                        self.perf.motion_res[vb],
                        self.perf.pin_frames_last[vb],
                        self.perf.pin_res_last[vb],
                    ),
                );
            }
            cap
        } else {
            1.0
        };
        let res_scale = if pin_frame {
            // Pinned refresh: geometry is captured, not derived — the motion scale was baked into
            // the pin's resolution at pin time, and applying it again would double-shrink (the
            // final geometry is forced to the pin's below, after the ss chain).
            1.0
        } else if !interacting && !self.tour_playing() {
            1.0
        } else if interacting && self.prefer_detail_effective() {
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
            let vm = (view_id as usize).min(1);
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
                let dt = self.perf.last_dt_ms;
                if self.perf.prev_real[vm] && dt > 0.0 {
                    if crate::diag::trace_on("gpu") {
                        crate::diag::trace(
                            "gpu",
                            format!("aimd: v{vm} real-frame dt={dt:.1}ms res={:.2}", self.perf.motion_res[vm]),
                        );
                    }
                    // Pure, so the thresholds are pinned by test rather than by reading — see
                    // `motion_res_step`. `dt` already has the fps_cap sleep discounted.
                    self.perf.motion_res[vm] = motion_res_step(
                        self.perf.motion_res[vm],
                        dt,
                        self.render_cfg.min_motion_res as f64,
                    );
                    // A real frame has now been priced at a known resolution: from here the
                    // opening guess is retired and this loop owns an unpinned refresh's size.
                    self.perf.motion_res_measured[vm] = true;
                }
            }
            // Direct mode keeps the work-budget bound it always had underneath the measured
            // scale; deep perturbation does not (the budget prices the no-skip cost and
            // over-shrinks exactly where the BLA skips — the reason this AIMD exists).
            let scale = self.perf.motion_res[vm].min(rate_res_cap);
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
        let res_scale = if interacting && !pin_frame {
            let vbi = (view_id as usize).min(1);
            // The TOP of the view's escape range, not its first escape: reaching the first escape
            // commits one pixel, reaching the top commits the picture.
            // With `MOTION_NEED_QUANTILE` below 1, how far MOST of it escapes (`motion_need`).
            // ⚠U24: WHICH top (`MOTION_NEED_SOURCE`). 0, shipped: the palette window's — which a
            // moving frame's own one-pass preview feeds (a preview is "complete": it is what is on
            // screen), so a frame that stops short lowers the top the next frame is sized to reach.
            // A guess reading a value downstream of what it sizes (principle P3). 1: the top of the
            // view's last COMPLETE walk (`Perf::esc_top`, beside the histogram, which was already
            // kept from complete walks only for this reason); none yet = no opinion.
            let top = if crate::tunables::cost().motion_need_source == 1 {
                self.perf.esc_top[vbi].map_or(0.0, |t| t as f64)
            } else {
                self.perf.norm_range[vbi].map_or(0.0, |(_, hi)| hi as f64)
            };
            let need = motion_need(
                top,
                self.perf.esc_hist[vbi].as_ref().map(|h| &h[..]),
                crate::tunables::cost().motion_need_quantile,
            );
            let target = visible_res_target(
                self.perf.motion_pass_steps_last[vbi],
                (resolution[0] as u64) * (resolution[1] as u64),
                need,
            );
            let reaches = visible_frame_reaches(
                self.perf.motion_pass_steps_last[vbi],
                (resolution[0] as u64) * (resolution[1] as u64),
                self.perf.visible_res[vbi],
                need,
            );
            let held = visible_res_hold(self.perf.visible_res[vbi], target, reaches);
            if view_id == 0 && held != self.perf.visible_res[vbi] && crate::diag::trace_on("gpu") {
                crate::diag::trace(
                    "gpu",
                    format!(
                        "visible-res f={} {:.3} -> {held:.3} (target {target:.3}, pass {} steps, need {need:.0})",
                        self.perf.frame_idx,
                        self.perf.visible_res[vbi],
                        self.perf.motion_pass_steps_last[vbi]
                    ),
                );
            }
            self.perf.visible_res[vbi] = held;
            // Into the frame record: what sized this moving frame, and which of the two caps bound
            // it — `res_scale` (the AIMD / rate / budget caps above) or the ladder's `held`.
            self.perf.vis_sizing[vbi] = [need, target, res_scale];
            res_scale.min(held)
        } else {
            self.perf.vis_sizing[(view_id as usize).min(1)] = [0.0; 3];
            res_scale
        };
        MovingScale { motion_scale, res_scale }
    }

    /// A mode switch's reset of the moving scale (`build_params`): the AIMD's measured flag, the
    /// visible-frame ladder's rung and the pass budget it was sized against (U51) were all earned
    /// at the old mode's cost per step. Left in place, the first frames of the new mode ran on the
    /// old mode's rung — chosen for a step that now costs several times more (or less) — until the
    /// ladder noticed.
    pub(super) fn reset_moving_scale(&mut self, v: usize) {
        self.perf.motion_res_measured[v] = false;
        self.perf.visible_res[v] = 1.0;
        self.perf.motion_pass_steps_last[v] = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settled() -> FrameFacts {
        FrameFacts { has_frozen: true, dispatches: true, ..Default::default() }
    }

    #[test]
    fn a_reprojection_with_nothing_behind_it_is_a_placeholder() {
        let f = FrameFacts { reproject: true, has_frozen: false, ..Default::default() };
        assert_eq!(classify(f), FrameKind::Placeholder);
        assert_eq!(classify(FrameFacts { has_frozen: true, ..f }), FrameKind::Reproject);
    }

    #[test]
    fn a_reprojection_wins_over_everything_it_cancels() {
        // The freeze verdict turns a planned pass into a reprojection: nothing iterated, whatever
        // was planned.
        let f = FrameFacts {
            reproject: true,
            has_frozen: true,
            interacting: true,
            pin_pass: true,
            dispatches: true,
            chunked: true,
            ..Default::default()
        };
        assert_eq!(classify(f), FrameKind::Reproject);
    }

    #[test]
    fn moving_frames_are_pins_or_motion() {
        let moving = FrameFacts { interacting: true, ..settled() };
        assert_eq!(classify(moving), FrameKind::Motion);
        assert_eq!(classify(FrameFacts { chunked: true, ..moving }), FrameKind::Motion);
        assert_eq!(classify(FrameFacts { pin_pass: true, chunked: true, ..moving }), FrameKind::PinWalk);
        // A pin's passes keep composing once the user stops: still the pin's.
        assert_eq!(classify(FrameFacts { pin_pass: true, ..settled() }), FrameKind::PinWalk);
    }

    #[test]
    fn settled_frames_split_by_what_they_dispatch() {
        assert_eq!(classify(settled()), FrameKind::Settled);
        assert_eq!(classify(FrameFacts { dispatches: false, ..settled() }), FrameKind::Hold);
        assert_eq!(classify(FrameFacts { tile: true, ..settled() }), FrameKind::Tile);
        assert_eq!(classify(FrameFacts { chunked: true, ..settled() }), FrameKind::SettledWalk);
        assert_eq!(classify(FrameFacts { accum: true, chunked: true, ..settled() }), FrameKind::Accum);
        // A tile that waits its turn dispatches nothing.
        assert_eq!(classify(FrameFacts { tile: true, dispatches: false, ..settled() }), FrameKind::Hold);
    }

    #[test]
    fn every_kind_has_one_name_and_a_distinct_nonzero_code() {
        let mut codes: Vec<u8> = FrameKind::NAMES.iter().map(|&(c, _)| c).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), FrameKind::NAMES.len());
        assert!(!codes.contains(&0), "0 is 'not built this frame'");
        for k in [
            FrameKind::Placeholder,
            FrameKind::Reproject,
            FrameKind::PinWalk,
            FrameKind::Motion,
            FrameKind::Hold,
            FrameKind::SettledWalk,
            FrameKind::Tile,
            FrameKind::Accum,
            FrameKind::Settled,
        ] {
            assert!(FrameKind::NAMES.iter().any(|&(c, _)| c == k.code()), "{k:?} has no name");
        }
    }

    /// §7.4's exit criterion, as a test rather than a grep someone has to remember to run: outside
    /// this module nothing WRITES a frame's scale — no `let res_scale` / `let budget_res_scale`
    /// computed in place, no assignment to `perf.visible_res[..]` or `perf.motion_res[..]` — and a
    /// frame's `Resolution` is made in exactly one place. A binding of a `frame_plan` function's
    /// result is not a writer. Scans every source file of the crate at test time.
    #[test]
    fn nothing_outside_frame_plan_writes_a_frames_scale_or_makes_its_resolution() {
        fn files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            for e in std::fs::read_dir(dir).expect("src dir").flatten() {
                let p = e.path();
                if p.is_dir() {
                    files(&p, out);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    out.push(p);
                }
            }
        }
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut all = Vec::new();
        files(&src, &mut all);
        assert!(all.len() > 20, "the scan must see the crate (found {} files)", all.len());
        let texts: Vec<(std::path::PathBuf, String)> =
            all.iter().map(|p| (p.clone(), std::fs::read_to_string(p).expect("readable source"))).collect();
        // Test-only files: those declared `#[cfg(test)] mod NAME;` anywhere in the crate.
        let mut test_mods = std::collections::HashSet::new();
        for (_, text) in &texts {
            let lines: Vec<&str> = text.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
            for w in lines.windows(2) {
                if w[0] == "#[cfg(test)]" {
                    if let Some(name) = w[1].strip_prefix("mod ").and_then(|r| r.strip_suffix(';')) {
                        test_mods.insert(name.to_string());
                    }
                }
            }
        }
        assert!(test_mods.contains("chunk_bands"), "the test-module scan must see `mod chunk_bands`");
        let mut writers = Vec::new();
        let mut panel_makers = 0;
        for (p, text) in &texts {
            if p.file_name().is_some_and(|n| n == "frame_plan.rs") {
                continue;
            }
            let test_only = p.file_stem().is_some_and(|s| test_mods.contains(&*s.to_string_lossy()));
            for (i, line) in text.lines().enumerate() {
                let code = line.split("//").next().unwrap_or("");
                let binds = ["let res_scale =", "let budget_res_scale ="].iter().any(|k| code.contains(k));
                let assigns = ["perf.visible_res[", "perf.motion_res["].iter().any(|k| {
                    code.find(k).is_some_and(|at| {
                        let rest = &code[at..];
                        rest.find(']').is_some_and(|e| {
                            let after = rest[e + 1..].trim_start();
                            after.starts_with('=') && !after.starts_with("==")
                        })
                    })
                });
                if (binds || assigns) && !code.contains("frame_plan::") {
                    writers.push(format!("{}:{}: {}", p.display(), i + 1, line.trim()));
                }
                if code.contains("Resolution::panel(") && !test_only {
                    panel_makers += 1;
                }
            }
        }
        assert!(writers.is_empty(), "scale writers outside frame_plan:\n{}", writers.join("\n"));
        // `render.rs` makes the frame's resolution once (top of `build_params`); tests make their own.
        assert_eq!(panel_makers, 1, "a frame's Resolution must be made in exactly one place outside tests");
    }

    #[test]
    fn framelog_names_the_kinds_as_the_code_does() {
        // The reader decodes the record by these names; a kind added here and not there would
        // print as "?" in every summary.
        let script = include_str!("../../../../scripts/framelog.py");
        for (code, name) in FrameKind::NAMES {
            assert!(
                script.contains(&format!("{code}: \"{name}\"")),
                "scripts/framelog.py KIND lacks {code}: {name:?}"
            );
        }
    }

    #[test]
    fn the_jitter_free_sig_is_the_view_and_a_sample_differs_only_there() {
        let view = WalkSig::of(0xABCD, 0, 10_000, Resolution::panel([551, 870]), 1);
        let sample = WalkSig::of(0xABCD, 0x55, 10_000, Resolution::panel([551, 870]), 1);
        assert_ne!(view, sample);
        assert_eq!((view.ask(), view.res(), view.ss()), (sample.ask(), sample.res(), sample.ss()));
        // The historical fold, so the sig is the same value it was as a tuple.
        assert_eq!(sample.view, 0xABCD ^ 0x55u64.rotate_left(7));
    }

    #[test]
    fn two_walks_differing_only_in_resolution_never_share_a_sig() {
        // §7.4: a resolution change restarts the walk — the walk-restart class is about the cost of
        // that, never about skipping it. Property over a spread of sizes.
        for w in [16_u32, 145, 551, 1452, 3840] {
            for h in [16_u32, 110, 870, 1102, 2160] {
                let a = WalkSig::of(0x1234, 0, 50_000, Resolution::panel([w, h]), 1);
                for (dw, dh) in [(1, 0), (0, 1), (w / 2, h / 2)] {
                    if dw == 0 && dh == 0 {
                        continue;
                    }
                    let b = WalkSig::of(0x1234, 0, 50_000, Resolution::panel([w + dw, h + dh]), 1);
                    assert_ne!(a, b);
                    assert_ne!(a.word(), b.word(), "{w}x{h} vs {}x{}", w + dw, h + dh);
                }
                let ss2 = WalkSig::of(0x1234, 0, 50_000, Resolution::panel([w, h]), 2);
                assert_ne!(a.word(), ss2.word());
            }
        }
    }

    #[test]
    fn the_record_word_is_zero_only_for_no_walk() {
        assert_eq!(WalkSig::NONE.word(), 0);
        assert_eq!(WalkSig::default(), WalkSig::NONE);
        assert_ne!(WalkSig::of(0, 0, 1, Resolution::panel([16, 16]), 1).word(), 0);
    }

    #[test]
    fn only_a_walk_part_way_through_is_lost_by_a_restart() {
        assert!(!walk_restart_discards(0, 10_000), "a motion preview never commits its cursor");
        assert!(walk_restart_discards(256, 10_000));
        assert!(walk_restart_discards(9_999, 10_000));
        assert!(!walk_restart_discards(10_000, 10_000), "a completed walk has nothing left to lose");
        assert!(!walk_restart_discards(0, 0), "no walk");
    }

    #[test]
    fn continuous_motion_never_reads_as_a_jump() {
        // U39: the auto-zoom, the wheel, the Home glide and tours used to take the jump floor
        // because only Space wrote a zoom velocity. A zoom that ramps is motion at any speed.
        // The Space glide / auto-zoom: eased, at most ~0.27 octaves a frame.
        let mut prev = f64::NAN;
        for step in [0.01, 0.05, 0.12, 0.2, 0.27, 0.27, 0.27] {
            assert!(!is_jump_in(step, prev), "a {step}-octave glide step");
            prev = step;
        }
        // A smoothstep dive from rest at 60 fps, fast enough to move hundreds of octaves a frame
        // at its peak (the Home glide's curve, run in): never a jump.
        let total = 99_658.0_f64; // octaves, 1e30000
        let n = 9 * 60;
        let ease = |u: f64| u * u * (3.0 - 2.0 * u);
        let mut prev = f64::NAN;
        let mut jumps = 0;
        for i in 1..=n {
            let d = total * (ease(i as f64 / n as f64) - ease((i - 1) as f64 / n as f64));
            jumps += is_jump_in(d, prev) as u32;
            prev = d;
        }
        // Its peak is ~277 octaves a frame, and its first frame from rest ~1.02 — under the jump
        // threshold, and every later step under four times the one before.
        assert_eq!(jumps, 0, "an eased curve never reads as a jump, at any depth");
        // From 1e31 the first frame moves ~0.001 octaves.
        let first = 103.0 * ease(1.0 / n as f64);
        assert!(first < 0.01 && !is_jump_in(first, f64::NAN));
        // Zooming OUT is never a jump in, however far.
        assert!(!is_jump_in(-6.64, 0.0));
        assert!(!is_jump_in(-1.0e4, 0.0));
        // A view's first frame has nothing to compare against.
        assert!(!is_jump_in(f64::NAN, f64::NAN));
    }

    #[test]
    fn a_click_from_rest_is_a_jump_and_holds_until_a_new_frame_is_latched() {
        // The 100× click (6.64 octaves) of the black-patch report, from a still view.
        assert!(is_jump_in(6.64, 0.0));
        assert!(is_jump_in(6.64, f64::NAN));
        // 4× (2 octaves) from rest too; 2× is under the floor's reach on its own.
        assert!(is_jump_in(2.0, 0.0));
        assert!(!is_jump_in(1.0, 0.0));
        // A click during a glide is sudden against the glide's step.
        assert!(is_jump_in(6.64, 0.27));
        let mut j = hold_jump_next(false, true, false);
        assert!(j);
        // The view sits still while the reference builds: the held frame still stands across it.
        for _ in 0..8 {
            j = hold_jump_next(j, false, false);
            assert!(j);
        }
        // A real frame at the new view replaces the held one: no jump between them any more.
        j = hold_jump_next(j, false, true);
        assert!(!j);
        // A jump and a latch on the same frame: the latched frame is at the new view.
        assert!(!hold_jump_next(false, true, true));
    }

    #[test]
    fn the_hold_floor_code_says_whether_the_jump_bound_moved_the_picture() {
        // A dive keeps its true scale and says so — and says when that is past the jump floor.
        let (s, c) = hold_scale(-1.0, false);
        assert_eq!((s, c), (0.5, hold_floor::DIVE));
        let (s, c) = hold_scale(-6.64, false);
        assert_eq!(c, hold_floor::DIVE_DEEP);
        assert!((s - (-6.64_f32).exp2()).abs() < 1.0e-6);
        // A jump within the bound is shown at its true scale.
        let (s, c) = hold_scale(-1.0, true);
        assert_eq!((s, c), (0.5, hold_floor::JUMP));
        // A jump past it is clamped, and the code says the screen shows the wrong scale.
        let (s, c) = hold_scale(-6.64, true);
        assert_eq!(c, hold_floor::JUMP_BOUND);
        assert_eq!(s, super::super::hold_scale_floor(true));
        // Zooming out is never floored.
        assert_eq!(hold_scale(2.0, true), (4.0, hold_floor::JUMP));
    }
}
