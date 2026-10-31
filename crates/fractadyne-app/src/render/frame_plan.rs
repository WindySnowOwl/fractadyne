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
///
/// It serializes for the decision recordings (`render::decide`): a replayed `Resolution` is one this
/// module made in the session that recorded it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
///
/// It serializes for the decision recordings (`render::decide`): a replayed `WalkSig` is one
/// [`WalkSig::of`] made in the session that recorded it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
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

/// The held frame's display transform `[uv_off.x, uv_off.y, uv_scale]` (the colour pass's
/// reprojection, `fs_color`) for `gap_oct` octaves of zoom since it was rendered and the view's
/// centre `off` from the held frame's centre, in spans of the view now (x right, y up), with the
/// [`hold_floor`] code of the scale's bound.
///
/// ⭐⭐**The scale is bounded; the offset never is.** The shader maps the view's centre to
/// `0.5 − uv_off` in the held frame, so `uv_off` must be the centre's TRUE place there: `off` times
/// the true scale. It was `off` times the BOUNDED scale, which a 100× click binds at 8×: the offset
/// came out 12.5 times too far, and any click more than about 4% of the width from the centre put
/// the whole held picture outside the frame, which the shader fills with the frame's average colour.
/// A flat screen for as long as the reference took to build (user, 2026-10-31: "rather than going
/// to a solid color screen while computing, magnify the image around where it will be zoomed").
/// Bounded, the held frame shows the clicked place magnified as far as the bound allows, about the
/// point that becomes the view's centre. Where no bound binds (every glide and dive) the two scales
/// are the same number, and nothing changes. A stepped Home keeps its own pair
/// ([`home_step_hold_scale`], [`home_step_hold_shift`]).
pub(crate) fn hold_transform(gap_oct: f64, jumped: bool, stepping: bool, off: (f64, f64)) -> ([f32; 3], u8) {
    let (scale, code) = hold_scale(gap_oct, jumped);
    let (true_scale, _) = hold_scale(gap_oct, false);
    let shift = home_step_hold_shift(true_scale, stepping);
    ([(-off.0 as f32) * shift, (off.1 as f32) * shift, home_step_hold_scale(scale, stepping)], code)
}

/// A held frame's scale while a stepped Home is under way (`HomeSteps`, `stepping`): never smaller
/// than full size. Each stop is up to thousands of octaves out from the frame on screen, and at its
/// true scale (up to the 2^40 the hold allows) that frame shrinks to a speck and the rest of the
/// view fills with its average colour (`fs_color`): a solid panel, the 2026-10-26 recording. At
/// full size the last finished picture stays until the stop's own replaces it. A zoom in (scale
/// below 1) is not touched.
pub(crate) fn home_step_hold_scale(scale: f32, stepping: bool) -> f32 {
    if stepping { scale.min(1.0) } else { scale }
}

/// The held frame's translation factor during a stepped Home: 0, the picture centred. The stops'
/// centres move toward Home's (`home_lerp`), and at the clamped scale ([`home_step_hold_scale`]) the
/// offset in the stop's view places nothing truly: it slid the picture off-centre and the gap filled
/// with its average colour (2^262 to 2^12, 2026-10-27). Otherwise the scale, as before.
pub(crate) fn home_step_hold_shift(scale: f32, stepping: bool) -> f32 {
    if stepping { 0.0 } else { scale }
}

/// The share of the panel's width a stepped Home's stop renders at ([`home_stop_res_scale`]).
pub(crate) const HOME_STOP_RES_SCALE: f64 = 0.25;

/// A frame's scale (`bp_moving_scale`) at a stepped Home's stop: a still view rendered to be SEEN on
/// the way out, not settled on, so at most [`HOME_STOP_RES_SCALE`] of the panel. At full size a stop
/// past 2^1000 was 1.6 million pixels of ~116,000 iterations each, and no stop there finished in
/// the 1.5 s a stop then had (2^1663 on the RTX 3080, 2026-10-26: the start picture stayed up
/// through five stops).
/// A moving frame, and any frame outside a stepped Home, keeps its own scale.
pub(crate) fn home_stop_res_scale(scale: f64, interacting: bool, stepping: bool) -> f64 {
    if stepping && !interacting { scale.min(HOME_STOP_RES_SCALE) } else { scale }
}

/// The base of a stepped Home stop's iteration ask ([`home_stop_iter`]).
pub(crate) const HOME_STOP_ITER_BASE: u32 = 2_000;

/// The iteration ask at a stepped Home's stop: at most what the stop's DEPTH wants
/// (`recommended_iterations` over [`HOME_STOP_ITER_BASE`], ~220 an octave), not the user's own
/// count. A stop completes when its walk reaches the ask, and a still view's interior pixels walk
/// all of it: a fixed 2,008,192 (e4000, auto iterations off) made the four shallowest stops, 2^98 to
/// 2^12, give way after their 3 s each (PLUTO, dev.47) — 12 s of one held picture. The stop is a
/// preview; a pixel that would escape past the cap reads as interior there.
///
/// ⚠Applied to the climb's APPETITE (`IterInputs::eff_iter`), never to its output: the climb judges
/// a MAXITER reading only when it was armed at its own budget, and a cap on the ask it planned had
/// every reading at a stop discarded as taken at another budget — `climb-starved` in all four PLUTO
/// runs of dev.48 (a probe whose readings are all discarded never terminates).
pub(crate) fn home_stop_iter(ask: u32, log2mag: f64, interacting: bool, stepping: bool) -> u32 {
    if stepping && !interacting {
        ask.min(fractadyne_core::recommended_iterations(HOME_STOP_ITER_BASE, log2mag))
    } else {
        ask
    }
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
    /// module is a place that can restart a walk unseen. W5: the decision itself is the pure
    /// `decide::decide_moving` over a recorded inputs struct (its notes moved with it); this gathers
    /// the inputs, applies the state it writes, emits the traces and records it for replay.
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
        let vb = (view_id as usize).min(1);
        // Only a frame the chunk path can serve is spread over passes (the pin); an aux colouring,
        // a formula past the chunk shaders' scope or a device without the state targets renders
        // each refresh as ONE dispatch, and then one pass is all there is.
        let chunk_mode = self.live_render_mode(fractal, julia, magnification);
        let can_chunk = super::actuators::can_chunk(self.perf.actuators.as_ref(), chunk_mode)
            && self.resumable(fractal)
            && !self.coloring.color_method.needs_aux();
        let c = crate::tunables::cost();
        // W5: every value the decision reads, gathered — the decision itself is the pure
        // `decide::decide_moving`, so a recording of these inputs replays it exactly.
        let inputs = super::decide::MovingInputs {
            px,
            gpu_iter,
            budget,
            interacting,
            pin_frame,
            view: vb as u8,
            is_pert,
            panel_px: resolution.px(),
            prefer_detail: self.prefer_detail_effective(),
            tour_playing: self.tour_playing(),
            can_chunk,
            zoom_oct_s: self.perf.zoom_oct_s,
            frame_ms: self.perf.frame_ms,
            last_dt_ms: self.perf.last_dt_ms,
            prev_real: self.perf.prev_real[vb],
            motion_rate: self.perf.motion_rate_now(vb),
            min_motion_res: self.render_cfg.min_motion_res as f64,
            pin_frames_last: self.perf.pin_frames_last[vb],
            pin_res_last: self.perf.pin_res_last[vb],
            motion_res: self.perf.motion_res[vb],
            motion_res_measured: self.perf.motion_res_measured[vb],
            visible_res: self.perf.visible_res[vb],
            motion_pass_steps_last: self.perf.motion_pass_steps_last[vb],
            norm_hi: self.perf.norm_range[vb].map_or(0.0, |(_, hi)| hi as f64),
            esc_top: self.perf.esc_top[vb].map_or(0.0, |t| t as f64),
            esc_hist: self.perf.esc_hist[vb].map(|h| h.to_vec()),
            tdr_bootstrap_steps: c.tdr_bootstrap_steps,
            motion_need_source: c.motion_need_source,
            motion_need_quantile: c.motion_need_quantile,
            pass_fixed_ms: crate::calibration::pass_fixed_ms(),
        };
        let d = super::decide::decide_moving(&inputs);
        if view_id == 0 && crate::diag::trace_on("gpu") {
            if let Some(rc) = d.rate_cap {
                crate::diag::trace(
                    "gpu",
                    format!(
                        "res-cap f={} cap={:.3} from={} pinned={} aimd={:.3} allowed={} pin_took={} \
                         pin_res={:.3} ask={gpu_iter}",
                        self.perf.frame_idx,
                        rc.cap,
                        if rc.measured { "measured" } else { "model" },
                        rc.pinned_refresh,
                        inputs.motion_res,
                        rc.frames_allowed,
                        inputs.pin_frames_last,
                        inputs.pin_res_last,
                    ),
                );
            }
        }
        if d.aimd_stepped && crate::diag::trace_on("gpu") {
            crate::diag::trace(
                "gpu",
                format!("aimd: v{vb} real-frame dt={:.1}ms res={:.2}", inputs.last_dt_ms, inputs.motion_res),
            );
        }
        if view_id == 0 && d.visible_res != inputs.visible_res && crate::diag::trace_on("gpu") {
            crate::diag::trace(
                "gpu",
                format!(
                    "visible-res f={} {:.3} -> {:.3} (target {:.3}, pass {} steps, need {:.0})",
                    self.perf.frame_idx,
                    inputs.visible_res,
                    d.visible_res,
                    d.vis_sizing[1],
                    inputs.motion_pass_steps_last,
                    d.vis_sizing[0],
                ),
            );
        }
        // The state the decision writes.
        self.perf.motion_res[vb] = d.motion_res;
        self.perf.motion_res_measured[vb] = d.motion_res_measured;
        self.perf.visible_res[vb] = d.visible_res;
        self.perf.vis_sizing[vb] = d.vis_sizing;
        self.record_decision("moving", vb, &inputs, &d);
        // A stepped Home's stop renders small (`home_stop_res_scale`), after the recorded decision
        // so its replay is untouched.
        let stop = self.perf.hold_home_step[vb];
        MovingScale {
            motion_scale: home_stop_res_scale(d.motion_scale, interacting, stop),
            res_scale: home_stop_res_scale(d.res_scale, interacting, stop),
        }
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
#[path = "../../tests/unit/render/frame_plan/tests.rs"]
mod tests;
