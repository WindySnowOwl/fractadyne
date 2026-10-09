//! MOTION REFRESHES FROM THE SECOND GPU (design/multi-gpu-live.md L3).
//!
//! While the view moves, the worker (`gpu_worker`) renders whole frames of it through the live
//! renderer and sends back their G-buffers; the window adopts each one that is newer than the
//! frame on screen as the frame it reprojects. The window's own refreshes carry on, on their own
//! clock (`RefCache::local_l2`), so the two GPUs' frames interleave and the display always shows
//! the newest real one.
//!
//! - **Offer** (`FractadyneApp::live_offer`, end of `build_params`): with the worker idle, a moving
//!   frame's params go to the worker as a whole frame of the LIVE view at the worker's own
//!   resolution, with a ticket recording the view. A pin frame's params describe the pinned view;
//!   they are re-aimed at the live view when it lies inside the pinned one, where every
//!   approximation made for the pinned view (series skip, BLA radii) still holds ([`LiveAim`]).
//!   A pan drag's frame computes no reference offset and offers nothing.
//! - **Collect** (`FractadyneApp::pump_live`): a finished frame waits in `Perf::live_ready`; a newer one
//!   replaces it.
//! - **Adopt** (`FractadyneApp::live_take`, in `build_params` once the frame's kind is known): only a
//!   frame that shows the live view better than the one on screen ([`frame_quality`]: sharper once
//!   magnified, and covering it), and only where it can be shown ([`adopt_target`]). While a
//!   worker frame is on screen, a window pin that cannot show the view better is abandoned
//!   (`PinStop::Superseded`). At equal resolutions the newest frame wins; a worker that has
//!   dropped its resolution does not replace a sharper pin with a blurrier frame.

use std::sync::Arc;
use std::time::Instant;

use crate::FractadyneApp;

/// What the app keeps of a motion refresh it gave the worker: the view the frame shows.
#[derive(Clone)]
pub(crate) struct LiveTicket {
    pub(crate) gen: u64,
    /// The frame whose params it renders — compared with `RefCache::frozen_frame`.
    pub(crate) frame: u64,
    pub(crate) center_bf: [fractadyne_core::BigFloat; 2],
    pub(crate) log2mag: f64,
    pub(crate) res: [u32; 2],
    /// log2 of its texels across the panel's width (`RefCache::frozen_res`).
    pub(crate) res_log2: f64,
    pub(crate) at: Instant,
}

/// A finished motion refresh waiting to be adopted.
pub(crate) struct LiveDelivery {
    pub(crate) ticket: LiveTicket,
    pub(crate) frame: Arc<fractadyne_gpu::AdoptFrame>,
}

/// The live view a pin frame's params are re-aimed at (`build_params` saves it before shadowing
/// the view with the pin's): centre, magnification, and span as the GPU takes it.
pub(crate) struct LiveAim {
    pub(crate) center_bf: [fractadyne_core::BigFloat; 2],
    pub(crate) log2mag: f64,
    pub(crate) delta_exp: i32,
    pub(crate) span: fractadyne_core::SpanMantissa,
    pub(crate) precision: usize,
}

/// Whether a view of span `(lx, ly)` centred `(dx, dy)` from another view's centre lies inside
/// that view, of span `(tx, ty)` — all in one exponent's units.
pub(crate) fn view_inside(dx: f64, dy: f64, lx: f64, ly: f64, tx: f64, ty: f64) -> bool {
    let slack = 1.0 + 1.0e-9;
    dx.abs() + 0.5 * lx <= 0.5 * tx * slack && dy.abs() + 0.5 * ly <= 0.5 * ty * slack
}

/// One motion's worth of counts, logged when the motion ends.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct LiveStats {
    pub(crate) offered: u32,
    pub(crate) delivered: u32,
    pub(crate) adopted_live: u32,
    pub(crate) adopted_hold: u32,
    /// No better than the frame already on screen when it came to be adopted ([`frame_quality`]).
    pub(crate) stale: u32,
    /// Overtaken by a real frame of this device at the current view.
    pub(crate) overtaken: u32,
    /// Window pins abandoned because an adopted frame was newer.
    pub(crate) superseded: u32,
    pub(crate) failed: u32,
    /// Pin frames that offered nothing: the live view had left the pinned one.
    pub(crate) outside: u32,
    pub(crate) ms_sum: f64,
    pub(crate) passes: u32,
    pub(crate) max_pass_ms: f64,
}

/// Where a worker frame goes on this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Adopt {
    /// This frame renders the current view itself and latches it: the worker's is older.
    Drop,
    /// Into the live G-buffer: this frame reprojects it, or snapshots it into the hold
    /// (`hold_copy`) before a walk composes over the live texture.
    Live,
    /// Into the hold the display serves while a walk composes underneath.
    Hold,
}

/// [`Adopt`] for a frame that is `real` (it iterates the current view and latches it: not a
/// reprojection, pin pass, pin start or split start), with the display gate as decided.
pub(crate) fn adopt_target(real: bool, display_hold: bool, hold_copy: bool) -> Adopt {
    if real {
        Adopt::Drop
    } else if display_hold && !hold_copy {
        Adopt::Hold
    } else {
        Adopt::Live
    }
}

/// A worker frame is aimed at this long from offer to answer.
pub(crate) const LIVE_TARGET_MS: f64 = 150.0;
/// The worker's resolution, as a fraction of the panel's, stays within this.
pub(crate) const LIVE_RES_MIN: f64 = 0.25;
pub(crate) const LIVE_RES_START: f64 = 0.5;

/// The worker's next resolution fraction after a frame at `s` took `ms`: the area follows the
/// time (`√(target / ms)` on each side), at most a quarter either way per frame.
pub(crate) fn live_res_next(s: f64, ms: f64) -> f64 {
    let f = if ms.is_finite() && ms > 0.0 { (LIVE_TARGET_MS / ms).sqrt().clamp(0.8, 1.25) } else { 1.0 };
    (s * f).clamp(LIVE_RES_MIN, 1.0)
}

/// How well a frame rendered at magnification `2^l2_frame`, with `2^res_log2` texels across the
/// panel's width, shows the view at `2^l2_now`, as a log2: the texels a screen pixel gets once it
/// is magnified (capped at 1), times the share of the screen it covers (a deeper frame shrinks to a
/// patch on a zoom out). 0 = sharp and whole. Two frames rank the same at every later view of a
/// dive: both are magnified alike from here on.
pub(crate) fn frame_quality(res_log2: f64, l2_frame: f64, l2_now: f64) -> f64 {
    (res_log2 + l2_frame - l2_now).min(0.0) + (l2_now - l2_frame).min(0.0)
}

/// Content tags of adopted worker frames (`ContentTrack::live_tag`): apart from every frame index.
pub(crate) const LIVE_TAG_BIT: u64 = 1 << 62;

impl FractadyneApp {
    /// The worker's next motion refresh, offered at the end of `build_params` for view 0. `real`:
    /// `params` describe a whole frame of the view at `center_bf` / `log2mag` (a perturbation mode
    /// with its reference, not a pan drag). `aim`: on a pin frame, the live view to re-aim them at.
    /// Off the moment the view stops moving: the job in flight is cancelled and the motion's counts
    /// are logged.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn live_offer(
        &mut self,
        params: &fractadyne_gpu::MandelbrotParams,
        interacting: bool,
        real: bool,
        center_bf: &[fractadyne_core::BigFloat; 2],
        log2mag: f64,
        aim: Option<LiveAim>,
        panel: [u32; 2],
    ) {
        let v = 0;
        let Some(w) = self.gpu_worker.as_ref().filter(|w| w.alive()) else { return };
        let Some(window) = self.render_state.as_ref().map(|r| r.device.clone()) else { return };
        if crate::tunables::cost().worker_motion == 0 {
            return;
        }
        if !interacting {
            if let Some(t) = self.perf.live_job[v].take() {
                w.cancel_live(t.gen);
            }
            self.perf.live_ready[v] = None;
            let s = std::mem::take(&mut self.perf.live_stats[v]);
            if s.offered > 0 {
                crate::diag::log_line(
                    "worker",
                    &format!(
                        "motion: {} frames from the worker GPU ({} offered, {:.0} ms each, {:.1} passes, longest pass {:.0} ms, \
                         res {:.2}); adopted {} live + {} into the hold; {} no better than the screen, {} overtaken, {} pins superseded, \
                         {} pin frames outside{}",
                        s.delivered,
                        s.offered,
                        s.ms_sum / s.delivered.max(1) as f64,
                        s.passes as f64 / s.delivered.max(1) as f64,
                        s.max_pass_ms,
                        self.perf.live_res[v],
                        s.adopted_live,
                        s.adopted_hold,
                        s.stale,
                        s.overtaken,
                        s.superseded,
                        s.outside,
                        if s.failed > 0 { format!(", {} failed", s.failed) } else { String::new() },
                    ),
                );
            }
            return;
        }
        if !real || self.perf.live_job[v].is_some() || self.perf.worker_job.is_some() {
            return;
        }
        let s = self.perf.live_res[v];
        let res = [
            ((panel[0] as f64 * s).round() as u32).max(16),
            ((panel[1] as f64 * s).round() as u32).max(16),
        ];
        let mut p = params.headless();
        p.tile = None;
        p.chunk_range = None;
        p.chunk_idx = 0;
        p.split = [1, 0];
        p.ss = 1;
        p.jitter = [0.0, 0.0];
        p.probe_nonce = 0;
        p.chunk_walk = 0;
        p.resolution = res;
        // A pin frame's params describe the pinned view: re-aim them at the live one, if it lies
        // inside the pinned view (so the pin's approximations hold for it) — else offer nothing.
        let (center_bf, log2mag) = match &aim {
            None => (center_bf.clone(), log2mag),
            Some(a) => {
                let Some(rp) = self.ref_cache[v].ref_pt.as_ref() else { return };
                let de = params.delta_exp;
                let dx = fractadyne_core::ref_offset_mantissa(&a.center_bf[0], &center_bf[0], de, a.precision);
                let dy = fractadyne_core::ref_offset_mantissa(&a.center_bf[1], &center_bf[1], de, a.precision);
                let k = ((a.delta_exp - de) as f64).exp2();
                if !view_inside(dx, dy, a.span.x * k, a.span.y * k, params.span_mantissa.x, params.span_mantissa.y) {
                    self.perf.live_stats[v].outside += 1;
                    return;
                }
                p.ref_offset = fractadyne_gpu::RefOffset::from_df32(
                    fractadyne_core::ref_offset_mantissa(&a.center_bf[0], &rp[0], a.delta_exp, a.precision),
                    fractadyne_core::ref_offset_mantissa(&a.center_bf[1], &rp[1], a.delta_exp, a.precision),
                );
                p.delta_exp = a.delta_exp;
                p.span_mantissa = a.span;
                (a.center_bf.clone(), a.log2mag)
            }
        };
        self.perf.live_gen += 1;
        let gen = self.perf.live_gen;
        let job = crate::gpu_worker::LiveJob {
            view: v,
            gen,
            with_aux: fractadyne_gpu::method_needs_aux(params.color_method),
            params: p,
            window,
            pass_ms: crate::tunables::cost().worker_pass_ms,
        };
        if w.submit_live(job) {
            self.perf.live_job[v] = Some(LiveTicket {
                gen,
                frame: self.perf.frame_idx,
                center_bf,
                log2mag,
                res,
                res_log2: (res[0] as f64 / panel[0].max(1) as f64).log2(),
                at: Instant::now(),
            });
            self.perf.live_stats[v].offered += 1;
        }
    }

    /// Collect the worker's finished motion refreshes (from `pump_worker`).
    pub(crate) fn pump_live(&mut self) {
        let Some(w) = self.gpu_worker.as_ref() else { return };
        let mut answers = Vec::new();
        while let Some(d) = w.try_recv_live() {
            answers.push(d);
        }
        for d in answers {
            let v = d.view.min(1);
            let Some(ticket) = self.perf.live_job[v].take_if(|t| t.gen == d.gen) else { continue };
            let ms = ticket.at.elapsed().as_secs_f64() * 1000.0;
            let st = &mut self.perf.live_stats[v];
            match d.frame {
                Some(frame) => {
                    st.delivered += 1;
                    st.ms_sum += ms;
                    st.passes += d.walk.passes;
                    st.max_pass_ms = st.max_pass_ms.max(d.walk.max_pass_ms);
                    self.perf.live_res[v] = live_res_next(self.perf.live_res[v], ms);
                    if crate::diag::trace_on("live") {
                        crate::diag::trace(
                            "live",
                            format!(
                                "worker v={v} f={} frame of f{} at {}x{} in {ms:.0} ms ({} passes, walk {:.0} ms, longest {:.0}, read {:.1}, \
                                 upload {:.1}) next res {:.2}",
                                self.perf.frame_idx,
                                ticket.frame,
                                ticket.res[0],
                                ticket.res[1],
                                d.walk.passes,
                                d.walk.ms,
                                d.walk.max_pass_ms,
                                d.read_ms,
                                d.upload_ms,
                                self.perf.live_res[v],
                            ),
                        );
                    }
                    self.perf.live_ready[v] = Some(LiveDelivery { ticket, frame });
                }
                None => {
                    if let Some(e) = d.err {
                        st.failed += 1;
                        if st.failed == 1 {
                            crate::diag::log_line("worker", &format!("a motion refresh failed on the worker GPU: {e}"));
                        }
                    }
                }
            }
        }
    }

    /// The worker frame to adopt on this frame of view `v` (the live view at `2^l2_now`), and whether
    /// it goes into the hold — `None` when there is none, or it does not show the view better than
    /// the frame on screen ([`frame_quality`]; a tie goes to the newer), or this frame renders the
    /// current view itself ([`adopt_target`]).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn live_take(
        &mut self,
        v: usize,
        interacting: bool,
        l2_now: f64,
        real: bool,
        display_hold: bool,
        hold_copy: bool,
    ) -> Option<(LiveDelivery, bool)> {
        if v != 0 || !interacting {
            return None;
        }
        let d = self.perf.live_ready[v].take()?;
        let rc = &self.ref_cache[v];
        let better = rc.frozen_center.is_none() || {
            let new = frame_quality(d.ticket.res_log2, d.ticket.log2mag, l2_now);
            let old = frame_quality(rc.frozen_res, rc.frozen_l2, l2_now);
            new > old + 1.0e-9 || (new >= old - 1.0e-9 && d.ticket.frame > rc.frozen_frame)
        };
        let st = &mut self.perf.live_stats[v];
        if !better {
            st.stale += 1;
            return None;
        }
        match adopt_target(real, display_hold, hold_copy) {
            Adopt::Drop => {
                st.overtaken += 1;
                None
            }
            Adopt::Live => {
                st.adopted_live += 1;
                self.perf.live_adopted_total[v] += 1;
                Some((d, false))
            }
            Adopt::Hold => {
                st.adopted_hold += 1;
                self.perf.live_adopted_total[v] += 1;
                Some((d, true))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_worker_frame_goes_where_the_display_reads() {
        // A frame that renders the current view itself is newer: the worker's is dropped.
        assert_eq!(adopt_target(true, false, false), Adopt::Drop);
        // A reprojection with no gate shows the live texture: into it.
        assert_eq!(adopt_target(false, false, false), Adopt::Live);
        // A pin in flight (or its residue) serves the hold: into the hold.
        assert_eq!(adopt_target(false, true, false), Adopt::Hold);
        // A pin starting this frame snapshots the live texture after the adoption lands: into it.
        assert_eq!(adopt_target(false, true, true), Adopt::Live);
    }

    #[test]
    fn a_frame_is_ranked_by_how_it_shows_the_view_now() {
        // Sharp and current: 0. Magnified one octave: -1. Half the texels: one more octave down.
        assert_eq!(frame_quality(0.0, 10.0, 10.0), 0.0);
        assert_eq!(frame_quality(0.0, 9.0, 10.0), -1.0);
        assert_eq!(frame_quality(-1.0, 9.0, 10.0), -2.0);
        // Extra texels are not counted past one a pixel; a deeper frame covers a shrinking patch.
        assert_eq!(frame_quality(1.0, 10.0, 10.0), 0.0);
        assert_eq!(frame_quality(0.0, 11.0, 10.0), -1.0);
        // The ranking of two frames does not change as a dive goes on: a quarter-resolution frame
        // 1.5 octaves newer loses to a full-resolution one at every later view.
        for now in [10.0, 11.0, 14.0] {
            assert!(frame_quality(0.0, 9.0, now) > frame_quality(-2.0, 10.5, now));
        }
    }

    #[test]
    fn a_live_view_is_inside_a_pinned_one_only_when_all_of_it_is() {
        // Zoomed in at the same centre: inside; zoomed out: not.
        assert!(view_inside(0.0, 0.0, 0.5, 0.4, 1.0, 0.8));
        assert!(view_inside(0.0, 0.0, 1.0, 0.8, 1.0, 0.8));
        assert!(!view_inside(0.0, 0.0, 1.01, 0.8, 1.0, 0.8));
        // Panned: inside while its far edge is; a quarter span right at half the size just fits.
        assert!(view_inside(0.25, 0.0, 0.5, 0.4, 1.0, 0.8));
        assert!(!view_inside(0.26, 0.0, 0.5, 0.4, 1.0, 0.8));
        assert!(!view_inside(0.0, -0.21, 0.5, 0.4, 1.0, 0.8));
    }

    #[test]
    fn the_worker_resolution_follows_its_time() {
        // On target: unchanged. Four times too slow: a quarter of the area, capped at -20% a side.
        assert_eq!(live_res_next(0.5, LIVE_TARGET_MS), 0.5);
        assert!((live_res_next(0.5, 4.0 * LIVE_TARGET_MS) - 0.4).abs() < 1e-12);
        // Fast: grows at most a quarter, never past the panel.
        assert!((live_res_next(0.5, 1.0) - 0.625).abs() < 1e-12);
        assert_eq!(live_res_next(0.95, 1.0), 1.0);
        // Never below the floor; a garbage time keeps it.
        assert_eq!(live_res_next(LIVE_RES_MIN, 1.0e9), LIVE_RES_MIN);
        assert_eq!(live_res_next(0.5, f64::NAN), 0.5);
    }
}
