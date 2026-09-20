//! Auto-zoom autopilot: a hands-free dive toward detail (View menu / A key; Esc or any input stops).
//!
//! ⭐**The goal is a point IN THE FRACTAL, and it is a point ON THE BOUNDARY.** The first version
//! re-picked the single highest-scoring cell of its probe every evaluation and eased a screen-space
//! pivot toward it. Measured from a user's 2.6e19 spiral view (2026-09-19): consecutive picks
//! jumped a median 0.37 screen heights, 72% of evaluations jumped over a quarter of the screen, and
//! the pivot — chasing picks that alternated between clusters in opposite corners — settled on
//! their AVERAGE, which is the flat gap between them. Every pick was detail; the point actually
//! being zoomed was not, and after 89 s the dive reached 1e37 with no boundary left in view. So:
//!
//! - the goal is scored on the shader's distance estimate and must itself be a boundary cell —
//!   only boundary points have unbounded detail, so zooming about one never runs out of it. What
//!   the score MAXIMISES is how many octaves the dive can descend about the cell ([`Probe::depth`],
//!   floored at the depth the estimate is trustworthy to); boundary density over a neighbourhood
//!   and nearness to the centre only discount that. Maximising the DENSITY instead — which is what
//!   the first rewrite did — aims at the centroid of the structure, and the centroid of a spiral is
//!   its empty eye;
//! - it is carried through every zoom and pan (`after_zoom`), so it stays on its content between
//!   evaluations instead of staying on a SCREEN position the content has moved away from;
//! - it is kept on its piece of the edge — each look moves it back onto the edge ACROSS the edge
//!   only, using the probe's distance estimate and normal, never along it (`project_onto_edge`) —
//!   and abandoned only for a region twice as rich (with a cooldown) or when it is lost;
//! - the camera eases: the aim closes on the goal, the aim drifts to the centre of the screen, and
//!   the zoom speed ramps in and slows through a turn.

use crate::{FractadyneApp, ZOOM_RATE};
use eframe::egui;

/// Auto-zoom autopilot: baseline seconds between target re-evaluations (grows adaptively with
/// depth, since deep frames are slower — see `autopilot_step`).
const AUTOPILOT_EVAL_INTERVAL: f64 = 0.35;
/// Depth (log₂ magnification) up to which the dive glides smoothly. Past this, each frame is too
/// slow to animate a continuous glide, so the autopilot switches to a stepped dive: pick a target,
/// then jump the zoom by a fixed factor. Choppy, but it keeps descending toward detail at extreme
/// depth. The overall stop depth is the user's dive limit (`autopilot_dive_log2`).
const AUTOPILOT_SMOOTH_LOG2: f64 = 900.0; // ≈ 1e271×
/// Stepped-dive magnification jump per re-evaluation, in log₂ (2.0 ⇒ ×4 per step).
const AUTOPILOT_STEP_LOG2: f64 = 2.0;
/// Cells in the steering probe (the old 56×56), laid out at the view's aspect so they are square.
const PROBE_CELLS: f64 = 3136.0;
/// Time constant (s) of EACH of the two eases that carry the aim onto the goal (see `glide_step`),
/// net of the zoom pushing the goal away — so the approach takes the same time at every zoom-rate
/// setting. Two stages of 0.35 s settle 95% of a turn in ≈1.7 s.
const AIM_TAU: f64 = 0.35;
/// Time constant (s, at the default zoom rate) of the aim drifting to the middle of the screen.
const CENTER_TAU: f64 = 2.0;
/// How far off centre (fraction of the screen's short side) the aim may sit before the pan above
/// touches it at all. Inside this, the dive is a pure zoom and the picture does not slide.
const CENTER_DEAD: f64 = 0.10;
/// Time constant (s) of every zoom-speed change: the start, a slow-down for a turn, the resume.
const SPEED_TAU: f64 = 0.4;
/// Seconds after a retarget during which only a LOST goal may retarget again.
const RETARGET_COOLDOWN: f64 = 1.5;

impl FractadyneApp {
    /// Toggle the auto-zoom autopilot (single view only).
    pub(crate) fn toggle_autopilot(&mut self, ctx: &egui::Context) {
        self.autopilot.active = !self.autopilot.active;
        if self.autopilot.active {
            self.autopilot.aim = (0.5, 0.5);
            self.autopilot.lead = (0.5, 0.5);
            self.autopilot.goal = None;
            self.autopilot.speed = 0.0;
            self.autopilot.retarget_t = f64::NEG_INFINITY;
            self.autopilot.eval_t = 0.0; // force an evaluation next frame
            self.home_anim = None;
            self.stop_playback();
            self.set_toast("Autopilot on — diving toward detail (any input stops)", ctx);
        } else {
            self.pointer.zoom_vel = 0.0;
            self.autopilot.stepping = false;
            self.set_toast("Autopilot off", ctx);
        }
    }

    /// One frame of the auto-zoom autopilot: glide into the tracked goal, re-evaluating it every
    /// `AUTOPILOT_EVAL_INTERVAL` from a small probe render of the view (`choose_goal`). Stops on
    /// manual input, a dead end (no boundary anywhere in view), or the depth cap.
    pub(crate) fn autopilot_step(
        &mut self,
        ctx: &egui::Context,
        gpu: &Option<(eframe::wgpu::Device, eframe::wgpu::Queue)>,
    ) {
        if !self.autopilot.active {
            self.autopilot.stepping = false;
            return;
        }
        // Any manual navigation, Esc, or dual view hands control back to the user.
        let interrupted = ctx.input(|i| {
            i.pointer.any_down()
                || i.smooth_scroll_delta.y != 0.0
                || i.key_down(egui::Key::Space)
                || i.key_down(egui::Key::Escape)
        });
        if interrupted || self.dual {
            self.autopilot.active = false;
            self.autopilot.stepping = false;
            self.pointer.zoom_vel = 0.0;
            return;
        }
        // Stop at the user's dive limit.
        let l2 = self.viewport.log2_magnification();
        if l2 >= self.autopilot.dive_log2 {
            self.autopilot.active = false;
            self.autopilot.stepping = false;
            self.pointer.zoom_vel = 0.0;
            self.set_toast(
                format!(
                    "Autopilot: dive limit reached (~1e{:.0}×)",
                    self.autopilot.dive_log2 / std::f64::consts::LOG2_10
                ),
                ctx,
            );
            return;
        }
        let now = ctx.input(|i| i.time);
        let dt = (ctx.input(|i| i.stable_dt) as f64).clamp(0.0, 0.1);

        // Past the smooth regime, animating a continuous glide stalls (each frame takes too long),
        // so switch to a stepped dive: on each re-evaluation, snap to the target and JUMP the zoom.
        let stepping = l2 >= AUTOPILOT_SMOOTH_LOG2;
        // Tells the render path to render real frames between jumps (and hold the last full frame
        // meanwhile) instead of the smooth-motion freeze that would blank the screen.
        self.autopilot.stepping = stepping;

        // Adaptive re-evaluation: the target-field render + reference recompute slow down with
        // depth, so evaluate less often as frames slow (≈ once per rendered frame when deep) while
        // staying snappy when shallow.
        let frame_s = (self.perf.frame_ms / 1000.0).max(0.0);
        let eval_interval = (1.5 * frame_s).max(AUTOPILOT_EVAL_INTERVAL);

        // In stepped mode, only advance once a real (settled) frame has actually rendered at the
        // current depth (frozen_l2 caught up to now) — so each full frame stays on screen while the
        // next one computes, instead of jumping onto blanks faster than the deep reference rebuilds.
        let ready = !stepping || (l2 - self.ref_cache[0].frozen_l2) < 1.0;

        if ready && now - self.autopilot.eval_t > eval_interval {
            self.autopilot.eval_t = now;
            if let Some((dev, q)) = gpu {
                let probe = self.autopilot_probe(dev, q);
                let aspect = self.viewport.width_px / self.viewport.height_px;
                let may_retarget = now - self.autopilot.retarget_t > RETARGET_COOLDOWN;
                let steer = probe.as_ref().and_then(|p| {
                    choose_goal(p, aspect, self.autopilot.goal, may_retarget)
                });
                if crate::diag::trace_on("autopilot") {
                    crate::diag::trace(
                        "autopilot",
                        format!(
                            "eval t={now:.3} l2={l2:.4} goal_was={:?} aim={:?} speed={:.3} \
                             pick={:?} retarget={} n={} cx={} cy={}",
                            self.autopilot.goal,
                            self.autopilot.aim,
                            self.autopilot.speed,
                            steer.map(|s| s.goal),
                            steer.is_some_and(|s| s.retarget),
                            probe.as_ref().map_or("-".into(), |p| format!("{}x{}", p.w, p.h)),
                            fractadyne_core::to_decimal_string(&self.viewport.center_x),
                            fractadyne_core::to_decimal_string(&self.viewport.center_y),
                        ),
                    );
                }
                match steer {
                    Some(s) => {
                        if s.retarget {
                            self.autopilot.retarget_t = now;
                        }
                        self.autopilot.goal = Some(s.goal);
                        if stepping {
                            // Stepped dive: snap the aim to the goal, jump the zoom by a fixed
                            // factor (2^AUTOPILOT_STEP_LOG2×) about it, then bring it halfway to
                            // the centre — each step is a full re-render, so there is no glide to
                            // ease through.
                            self.autopilot.aim = s.goal;
                            self.autopilot.lead = s.goal;
                            let factor = (-AUTOPILOT_STEP_LOG2 * std::f64::consts::LN_2).exp();
                            self.autopilot_zoom(s.goal, factor);
                            let a = self.autopilot.aim;
                            self.autopilot_pan(((0.5 - a.0) * 0.5, (0.5 - a.1) * 0.5));
                        }
                    }
                    None => {
                        self.autopilot.active = false;
                        self.autopilot.stepping = false;
                        self.pointer.zoom_vel = 0.0;
                        self.set_toast("Autopilot: no detail ahead (stopped)", ctx);
                        return;
                    }
                }
            }
        }

        if !stepping {
            if let Some(goal) = self.autopilot.goal {
                self.autopilot_glide(goal, dt);
            }
        }
        self.pointer.settle_t = [now; 2]; // treat as interaction (AA off, throttled reference refresh)
        self.schedule_repaint(ctx);
    }

    /// One frame of the smooth dive. Every change is eased, so nothing the camera does is a step:
    /// the zoom speed ramps toward its target (slower through a big turn), the aim closes on the
    /// goal, and the aim drifts to the middle of the screen. All three act on points that are
    /// carried through the motion, so an ease never chases a stale screen position.
    fn autopilot_glide(&mut self, goal: (f64, f64), dt: f64) {
        let rate = ZOOM_RATE * self.render_cfg.zoom_rate as f64;
        let aspect = self.viewport.width_px / self.viewport.height_px;
        let a = &self.autopilot;
        let g = glide_step(a.aim, a.lead, goal, a.speed, rate, aspect, dt);
        self.autopilot.speed = g.speed;
        self.autopilot.lead = g.lead;
        self.autopilot.aim = g.aim;
        self.autopilot_zoom(g.aim, g.factor);
        self.autopilot_pan(g.pan);
    }

    /// Zoom about the screen-fraction `pivot`, carrying the tracked points with their content.
    fn autopilot_zoom(&mut self, pivot: (f64, f64), factor: f64) {
        let (w, h) = (self.viewport.width_px, self.viewport.height_px);
        self.viewport.zoom_at(pivot.0 * w, pivot.1 * h, factor);
        let a = &mut self.autopilot;
        a.aim = after_zoom(a.aim, pivot, factor);
        a.lead = after_zoom(a.lead, pivot, factor);
        a.goal = a.goal.map(|g| after_zoom(g, pivot, factor));
    }

    /// Move the content by `d` (screen fractions), carrying the tracked points with it.
    fn autopilot_pan(&mut self, d: (f64, f64)) {
        let (w, h) = (self.viewport.width_px, self.viewport.height_px);
        self.viewport.pan_pixels(d.0 * w, d.1 * h);
        let a = &mut self.autopilot;
        let shift = |p: (f64, f64)| (p.0 + d.0, p.1 + d.1);
        a.aim = shift(a.aim);
        a.lead = shift(a.lead);
        a.goal = a.goal.map(shift);
    }

    /// Render the steering probe: a small iteration field of the current view, `PROBE_CELLS` cells
    /// laid out at the view's aspect. Square cells matter: the shader measures its distance
    /// estimate in HORIZONTAL steps, so only a square cell makes that estimate read in cells on
    /// both axes. (The old probe was 56×56 on a 16:10 view.) `None` = the render failed.
    fn autopilot_probe(&self, dev: &eframe::wgpu::Device, q: &eframe::wgpu::Queue) -> Option<Probe> {
        let aspect = (self.viewport.width_px / self.viewport.height_px).clamp(0.25, 4.0);
        let w = (PROBE_CELLS * aspect).sqrt().round().clamp(16.0, 128.0) as usize;
        let h = (PROBE_CELLS / w as f64).round().clamp(16.0, 128.0) as usize;
        // ⭐Borrows the LIVE view's resident reference instead of building a full-appetite one on
        // this thread. The probe is a HEURISTIC for choosing a pivot: it wants the picture on
        // screen, not export accuracy, and it used to buy that picture with a synchronous bignum
        // reference build per evaluation. See `autopilot_probe_request`.
        let mut req = self.autopilot_probe_request(&self.viewport, self.julia_mode);
        req.width = w as u32;
        req.height = h as u32;
        req.ss = 1;
        // The builder above stamped the crash manifest with the PANEL dims before these
        // overrides — in the Radeon autodive triage (crash-1787261212-0) that stamp read as a
        // 3840x2903 ss=2 export and misdirected the diagnosis toward the export path, when the
        // probe merely SURFACED a loss the inflated budget had already caused. State what this
        // probe actually submits, so the next crash report names the right suspect.
        crate::diag::set_manifest(format!(
            "PROBE {w}x{h} ss=1 mode={} (autopilot target probe, synchronous)",
            req.mode
        ));
        let px = fractadyne_gpu::render_iter(dev, q, &req).ok()?.pixels;
        if px.len() < w * h * 4 {
            return None;
        }
        // Dev hook (DIAGNOSTICS.md): every probe as raw f32, so a reported dive can be replayed and
        // the chooser judged on the fields it actually saw.
        if let Ok(dir) = std::env::var("FRACTADYNE_AUTOPILOT_DUMP") {
            use std::sync::atomic::{AtomicU32, Ordering};
            static SEQ: AtomicU32 = AtomicU32::new(0);
            let k = SEQ.fetch_add(1, Ordering::Relaxed);
            let bytes: Vec<u8> = px.iter().flat_map(|v| v.to_le_bytes()).collect();
            let _ = std::fs::write(format!("{dir}/probe-{k:05}.f32"), bytes);
        }
        // Texels are (smooth_iter, normal.x, normal.y, DE_log2) — see `fractadyne_gpu::render_iter`.
        Some(Probe {
            w,
            h,
            iter: px.chunks_exact(4).take(w * h).map(|t| t[0]).collect(),
            de_log2: px.chunks_exact(4).take(w * h).map(|t| t[3]).collect(),
            normal: px.chunks_exact(4).take(w * h).map(|t| (t[1], t[2])).collect(),
        })
    }
}

// ---- Steering: pure functions of the probe, so they can be tested without a GPU. ----

/// Distance-estimate sentinel: at or above this the cell has no estimate (interior, or a family
/// whose shader tracks no derivative — the iterate pass writes 1e30).
const DE_NONE: f32 = 1.0e29;
/// Radius (cells) of the neighbourhood the detail density is averaged over. Averaging is what
/// makes the score a smooth field: the old per-cell argmax flipped between clusters every look.
const DENSITY_R: usize = 3;
/// How far (cells) an evaluation may look for the edge again when the goal has drifted off it,
/// without that counting as a retarget.
const SNAP_R: usize = 2;
/// A cell is ON the boundary — a legal goal — at or above this weight.
const ON_BOUNDARY: f32 = 0.5;
/// The farthest (cells) a cell's distance estimate is trusted to locate the edge: past this the
/// straight-edge approximation behind [`Probe::project_onto_edge`] is not local any more.
const EDGE_TRUST: f64 = 2.0;
/// A region elsewhere must beat the current goal's by this factor to take over.
const RETARGET_GAIN: f64 = 2.0;
/// Width of the centre preference, as a fraction of the screen's short side. Tight on purpose: at
/// the old probe's 40%-at-the-edge bias, picks sat a median 0.32 screen heights off centre.
const CENTER_SIGMA: f64 = 0.25;
/// The deepest distance estimate the shader is trusted to report, in log₂ cells.
///
/// ⚠**The deepest readings are fantasies, and a chooser that maximises depth would chase exactly
/// them.** Checked against an independent 260-digit distance estimate on the probes of a reported
/// dive (2026-09-19, 2^486→2^548): the shader's estimate is EXACT — 0.00 octaves of error — for
/// every reading down to about −12, drifts ~2 octaves by −16, and is **22 to 28 octaves wrong**
/// below −32 (a cell reporting −35.4 was really at −11.2). The extremes come from pixels whose
/// perturbation has lost its derivative, and they are the ones a depth-ranking score would pick
/// first. So every reading is floored here: a cell may be credited with at most 12 octaves of
/// descent, which is still twelve times what one evaluation interval spends.
const DE_TRUST_LOG2: f32 = -12.0;
/// How far from the centre (fraction of the screen's short side) a NEW target may be chosen. A
/// target beyond this is a long pan at depth, so the search stays inside this region unless there
/// is no boundary in it at all.
const TARGET_MAX_OFF: f64 = 0.35;
/// How far a target already being tracked may be carried before it is dropped for a fresh one
/// inside [`TARGET_MAX_OFF`]. The gap between the two is hysteresis: without it a goal resting just
/// inside the bound would be re-picked every look.
const TARGET_KEEP_OFF: f64 = 0.45;

/// The steering probe: a small render of the current view with square cells. Per cell: the smooth
/// escape count (`< 0` = interior), the distance estimate as log₂ CELLS (`>= DE_NONE` = none), and
/// the unit gradient of the escape potential in COMPLEX orientation (+y = +imaginary; `(0, 0)` =
/// none) — it points away from the set, so the edge lies along its negative.
pub(crate) struct Probe {
    pub(crate) w: usize,
    pub(crate) h: usize,
    pub(crate) iter: Vec<f32>,
    pub(crate) de_log2: Vec<f32>,
    pub(crate) normal: Vec<(f32, f32)>,
}

/// What an evaluation decided: the goal (screen fraction) and whether it moved to a different
/// region rather than refining the one it had.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Steer {
    pub(crate) goal: (f64, f64),
    pub(crate) retarget: bool,
}

impl Probe {
    fn interior(&self, k: usize) -> bool {
        self.iter[k] < 0.0
    }

    fn neighbours(&self, i: usize, j: usize) -> impl Iterator<Item = usize> + '_ {
        let (w, h) = (self.w, self.h);
        [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)].into_iter().filter_map(move |(di, dj)| {
            let (ni, nj) = (i as isize + di, j as isize + dj);
            (ni >= 0 && nj >= 0 && (ni as usize) < w && (nj as usize) < h)
                .then(|| nj as usize * w + ni as usize)
        })
    }

    /// How many octaves the view can descend about this cell before the set's nearest point is off
    /// screen: the UNCLAMPED form of the middle case of [`Probe::boundary_weights`], floored at
    /// [`DE_TRUST_LOG2`]. Cells with no estimate fall back to their boundary weight (the abs
    /// families carry no derivative, so depth is not available to rank them by).
    ///
    /// ⭐⭐**A SCORE CANNOT RANK WHAT IT SATURATES ON.** `boundary_weights` reaches 1.0 at a HALF
    /// CELL and stays there — and on a deep view a half cell is ~11 screen pixels, so a median 42%
    /// of the eligible cells were tied at exactly 1.0 (measured over the 69 probes of a reported
    /// dive). The chooser maximised a DENSITY of those tied weights, and the maximum of a density
    /// over a blob is the blob's CENTROID — which, for the spiral that dominates every deep view,
    /// is the smooth eye. That is the "it zooms into a flat region" report, exactly: the goal had a
    /// median 4.0 octaves of descent in it (worst 0.8) while a fully credible 12 was available
    /// inside the bound at every single look. Ranking by depth costs nothing elsewhere — the
    /// deepest cell is also the DENSER one (0.884 against 0.798 over the same probes).
    fn depth(&self, k: usize, weights: &[f32]) -> f64 {
        if self.interior(k) {
            return 0.0;
        }
        if self.de_log2[k] >= DE_NONE {
            return weights[k] as f64;
        }
        (1.0 - self.de_log2[k].max(DE_TRUST_LOG2) as f64).max(0.0)
    }

    /// How surely the set's boundary passes through each cell: 1 = on it, 0 = well clear of it.
    /// Interior cells are 0 — never a goal. This is the QUALIFIER (is this cell a legal goal at
    /// all); [`Probe::depth`] is what ranks the cells that qualify.
    pub(crate) fn boundary_weights(&self) -> Vec<f32> {
        let mut out = vec![0.0f32; self.w * self.h];
        for j in 0..self.h {
            for i in 0..self.w {
                let k = j * self.w + i;
                if self.interior(k) {
                    continue;
                }
                out[k] = if self.neighbours(i, j).any(|n| self.interior(n)) {
                    1.0 // the set's edge runs between this cell and its interior neighbour
                } else if self.de_log2[k] < DE_NONE {
                    // Estimate ≤ ½ cell → 1, ≥ 2 cells → 0, log-linear between. Measured on the
                    // report's probes: flat exterior reads 4–16 cells, structure far below one.
                    ((1.0 - self.de_log2[k]) * 0.5).clamp(0.0, 1.0)
                } else {
                    // No estimate: the escape-count step to a neighbour stands in for it
                    // (≈1.4 counts per cell puts the boundary about a cell away).
                    let c = self.iter[k];
                    let g = self
                        .neighbours(i, j)
                        .filter(|&n| !self.interior(n))
                        .map(|n| (self.iter[n] - c).abs())
                        .fold(0.0f32, f32::max);
                    (g - 0.5).clamp(0.0, 1.0)
                };
            }
        }
        out
    }

    /// Move the screen-fraction point `g` onto the edge line of cell `(i, j)`: the line through
    /// the cell's own edge estimate (centre − DE·n) perpendicular to its normal `n`. `None` when
    /// the cell carries no usable estimate (interior, a family without one, a degenerate normal, or
    /// an edge farther than [`EDGE_TRUST`]).
    ///
    /// ⭐⭐**Across the edge, never along it.** Keeping a goal on the boundary used to mean snapping
    /// it to the CENTRE of a boundary cell. At the 4× zoom-speed setting the view grows almost an
    /// octave between looks, so that sub-cell error nearly doubles each time: the goal slid off the
    /// edge, was snapped to another centre up to two cells away, slid off again — a random walk the
    /// camera chased across a third of the screen in 3.5 s (a user's 1e118 dive, 2026-09-19). The
    /// projection removes only the component of the error that takes the goal OFF the edge; its
    /// place along the edge — which piece of structure the dive is heading into — is left alone.
    /// Verified on real probe fields: one step along −n (screen y flipped) halves the distance to
    /// the edge (median 2.0 → 0.9 cells); along +n it lands on the edge 0% of the time.
    pub(crate) fn project_onto_edge(&self, g: (f64, f64), i: usize, j: usize) -> Option<(f64, f64)> {
        let k = j * self.w + i;
        if self.interior(k) || self.de_log2[k] >= DE_NONE {
            return None;
        }
        let d = (self.de_log2[k] as f64).exp2();
        let (nx, ny) = (self.normal[k].0 as f64, -(self.normal[k].1 as f64)); // screen y is down
        if !(d <= EDGE_TRUST) || nx.hypot(ny) < 0.5 {
            return None;
        }
        // In cell units, where the cells are square.
        let (w, h) = (self.w as f64, self.h as f64);
        let (ex, ey) = (i as f64 + 0.5 - d * nx, j as f64 + 0.5 - d * ny);
        let (gx, gy) = (g.0 * w, g.1 * h);
        let off = (gx - ex) * nx + (gy - ey) * ny;
        if off.abs() > SNAP_R as f64 + 1.0 {
            return None; // not this cell's edge
        }
        Some(((gx - off * nx) / w, (gy - off * ny) / h))
    }
}

/// Mean of `v` over the `(2r+1)²` window around every cell, clipped at the edges.
fn box_mean(v: &[f32], w: usize, h: usize, r: usize) -> Vec<f64> {
    // Summed-area table with a zero row and column in front.
    let mut sat = vec![0.0f64; (w + 1) * (h + 1)];
    for j in 0..h {
        let mut row = 0.0;
        for i in 0..w {
            row += v[j * w + i] as f64;
            sat[(j + 1) * (w + 1) + i + 1] = sat[j * (w + 1) + i + 1] + row;
        }
    }
    let mut out = vec![0.0; w * h];
    for j in 0..h {
        for i in 0..w {
            let (i0, i1) = (i.saturating_sub(r), (i + r + 1).min(w));
            let (j0, j1) = (j.saturating_sub(r), (j + r + 1).min(h));
            let s = sat[j1 * (w + 1) + i1] - sat[j0 * (w + 1) + i1] - sat[j1 * (w + 1) + i0]
                + sat[j0 * (w + 1) + i0];
            out[j * w + i] = s / ((i1 - i0) * (j1 - j0)) as f64;
        }
    }
    out
}

/// Choose the goal for the next stretch of the dive.
///
/// `aspect` = view width / height. `current` = the goal being tracked (screen fraction, already
/// carried through every zoom and pan since it was chosen), `None` on the first evaluation.
/// `may_retarget` = false inside the cooldown after a retarget; a LOST goal (off screen, or no
/// boundary within reach) retargets regardless. `None` = no boundary anywhere in view: a genuine
/// dead end.
pub(crate) fn choose_goal(
    p: &Probe,
    aspect: f64,
    current: Option<(f64, f64)>,
    may_retarget: bool,
) -> Option<Steer> {
    let (w, h) = (p.w, p.h);
    let wt = p.boundary_weights();
    let density = box_mean(&wt, w, h, DENSITY_R);
    let inside: Vec<f32> = p.iter.iter().map(|&s| if s < 0.0 { 1.0 } else { 0.0 }).collect();
    let inside = box_mean(&inside, w, h, DENSITY_R);
    // Richness of a cell's neighbourhood: boundary density, discounted where the interior (flat
    // black) fills it.
    let rich = |k: usize| density[k] * (1.0 - 0.5 * inside[k]);
    let frac = |i: usize, j: usize| ((i as f64 + 0.5) / w as f64, (j as f64 + 0.5) / h as f64);
    let short = aspect.min(1.0);
    let centre_w = |f: (f64, f64)| {
        let r = ((f.0 - 0.5) * aspect).hypot(f.1 - 0.5) / short;
        (-r * r / (2.0 * CENTER_SIGMA * CENTER_SIGMA)).exp()
    };
    // What a candidate is worth: how far the dive can descend about it ([`Probe::depth`] — the term
    // that was saturated, and the reason the old score settled in the eye of a spiral), how much
    // structure surrounds it, and how short the journey to it is. All three are wanted, so they
    // multiply; depth carries the range (0 to 13), the other two are fractions that discount it.
    let score = |k: usize, at: (f64, f64)| p.depth(k, &wt) * rich(k) * centre_w(at);

    // ⭐⭐**A NEW TARGET MUST FIT THE BOUNDED REGION AROUND THE CENTRE.** The centre preference only
    // discourages an edge target; it does not forbid one, and a target near the edge is a long pan
    // — at depth the camera would travel most of a screen to reach it. Candidates are therefore
    // limited to `TARGET_MAX_OFF` of the screen's short side from the centre, and only when NOTHING
    // qualifies does the search widen — and then to the boundary cell CLOSEST to the centre, the
    // shortest journey to structure, rather than the richest one wherever it happens to be.
    let off_centre = |f: (f64, f64)| ((f.0 - 0.5) * aspect).hypot(f.1 - 0.5) / short;
    let mut best: Option<(usize, usize, f64)> = None;
    let mut nearest: Option<(usize, usize, f64)> = None;
    for j in 0..h {
        for i in 0..w {
            let k = j * w + i;
            if wt[k] < ON_BOUNDARY {
                continue;
            }
            let f = frac(i, j);
            let d = off_centre(f);
            if nearest.is_none_or(|b| d < b.2) {
                nearest = Some((i, j, d));
            }
            if d > TARGET_MAX_OFF {
                continue;
            }
            let s = score(k, f);
            if best.is_none_or(|b| s > b.2) {
                best = Some((i, j, s));
            }
        }
    }
    // A new goal goes ON the edge its cell's estimate locates, not at the cell's centre.
    let on_edge = |i: usize, j: usize| p.project_onto_edge(frac(i, j), i, j).unwrap_or(frac(i, j));
    let (bi, bj, best_score) = match (best, nearest) {
        (Some(b), _) => b,
        // Nothing within the bound: head for the closest structure instead, at its own score.
        (None, Some((i, j, _))) => (i, j, score(j * w + i, frac(i, j))),
        (None, None) => return None, // no boundary anywhere in view: a genuine dead end
    };
    let global = Steer { goal: on_edge(bi, bj), retarget: true };

    // A goal the view has carried past the bound is not kept: the dive re-picks inside it rather
    // than panning back across the screen. The slack over `TARGET_MAX_OFF` is what stops a goal
    // sitting just inside the bound from being re-picked every look.
    let in_bounds = |g: &(f64, f64)| {
        (0.0..1.0).contains(&g.0) && (0.0..1.0).contains(&g.1) && off_centre(*g) <= TARGET_KEEP_OFF
    };
    let Some(g) = current.filter(in_bounds) else {
        return Some(global); // the first pick, or the goal left the bounded region
    };
    let gi = ((g.0 * w as f64) as usize).min(w - 1);
    let gj = ((g.1 * h as f64) as usize).min(h - 1);
    // The cell whose edge the goal belongs to: its own while that is still on the boundary, else
    // the NEAREST boundary cell within reach (the richest one was a hill-climb, and at the 4×
    // zoom-speed setting a hill-climb re-aimed every look is exactly the wandering this prevents).
    let anchor = if wt[gj * w + gi] >= ON_BOUNDARY {
        Some((gi, gj))
    } else {
        let mut nearest: Option<(usize, usize, f64)> = None;
        for j in gj.saturating_sub(SNAP_R)..(gj + SNAP_R + 1).min(h) {
            for i in gi.saturating_sub(SNAP_R)..(gi + SNAP_R + 1).min(w) {
                if wt[j * w + i] < ON_BOUNDARY {
                    continue;
                }
                let d = (i as f64 - gi as f64).hypot(j as f64 - gj as f64);
                if nearest.is_none_or(|b| d < b.2) {
                    nearest = Some((i, j, d));
                }
            }
        }
        nearest.map(|(i, j, _)| (i, j))
    };
    let Some((ai, aj)) = anchor else {
        return Some(global); // lost: the goal drifted off the boundary into flat or interior
    };
    // Back onto the edge, moving only across it. Without an estimate (a family with no distance
    // estimate, or an interior-adjacent cell) the goal stays put on its own cell, or takes the
    // centre of the nearest boundary cell.
    let refined = p
        .project_onto_edge(g, ai, aj)
        .unwrap_or(if (ai, aj) == (gi, gj) { g } else { frac(ai, aj) });
    // ⭐**Score the goal where it IS, not at the cell that located its edge.** The anchor is only
    // the cell whose estimate the projection used; when the goal had drifted, that is a cell up to
    // `SNAP_R` OFF the edge, and its own depth is the depth of a point off the edge. Judging the
    // goal by it under-rates the goal by exactly the amount the projection just recovered — enough,
    // on a plain straight edge, to lose the goal to a retarget every other look.
    let ri = ((refined.0 * w as f64) as usize).min(w - 1);
    let rj = ((refined.1 * h as f64) as usize).min(h - 1);
    let kept = score(rj * w + ri, refined).max(score(aj * w + ai, refined));
    if may_retarget && best_score > RETARGET_GAIN * kept {
        return Some(global);
    }
    Some(Steer { goal: refined, retarget: false })
}

/// Where a screen-fraction point lands after `Viewport::zoom_at(pivot, factor)`: the pivot's
/// content stays put and everything else scales away from it by `1/factor`.
pub(crate) fn after_zoom(x: (f64, f64), pivot: (f64, f64), factor: f64) -> (f64, f64) {
    (pivot.0 + (x.0 - pivot.0) / factor, pivot.1 + (x.1 - pivot.1) / factor)
}

/// One frame of the smooth dive's camera, decided: zoom about the new `aim` by `factor`, then move
/// the content by `pan` (screen fractions).
#[derive(Clone, Copy, Debug)]
pub(crate) struct GlideStep {
    pub(crate) aim: (f64, f64),
    pub(crate) lead: (f64, f64),
    pub(crate) speed: f64,
    pub(crate) factor: f64,
    pub(crate) pan: (f64, f64),
}

/// The smooth dive's camera for one frame of `dt` seconds at zoom `rate` (nepers/s).
///
/// The aim follows the goal through a `lead` point — two cascaded eases rather than one, because a
/// single ease answers a goal that moves with an instant jump in the camera's VELOCITY, a visible
/// jolt at every retarget. Cascaded, the velocity starts from zero and builds; only acceleration
/// changes abruptly, which the eye does not read as a jerk.
pub(crate) fn glide_step(
    aim: (f64, f64),
    lead: (f64, f64),
    goal: (f64, f64),
    speed: f64,
    rate: f64,
    aspect: f64,
    dt: f64,
) -> GlideStep {
    // How far the aim still has to turn, in short-side screen units: through a big turn the zoom
    // eases down to half speed, so the dive visibly steers rather than overshooting.
    let turn = ((goal.0 - aim.0) * aspect).hypot(goal.1 - aim.1) / aspect.min(1.0);
    let want = rate * (1.0 - 0.5 * (turn / 0.25).clamp(0.0, 1.0));
    let speed = speed + (want - speed) * (1.0 - (-dt / SPEED_TAU).exp());
    // Close each stage. Zooming about the aim scales every distance on screen by e^(speed·dt), so
    // that is added to the closing rate: the NET approach runs at AIM_TAU per stage at any zoom
    // rate (the old fixed 0.5 s ease only just outran the zoom at the slider's 4× setting).
    let close = 1.0 - (-(1.0 / AIM_TAU + speed) * dt).exp();
    let lead = (lead.0 + (goal.0 - lead.0) * close, lead.1 + (goal.1 - lead.1) * close);
    let aim = (aim.0 + (lead.0 - aim.0) * close, aim.1 + (lead.1 - aim.1) * close);
    // ⭐⭐**A TARGET NEAR THE MIDDLE IS LEFT ALONE.** Zooming about a point keeps that point where
    // it is on screen, so the picture only flows outward from it — no sideways motion at all. Any
    // pan on top of that IS sideways motion, and a user sees it: centring on a fixed two seconds
    // left a 4× dive's target 3–5% off centre and drifting one way then the other ("sliding"),
    // and simply centring faster made the slide faster rather than removing it. So the pan has a
    // DEAD ZONE: inside `CENTER_DEAD` the aim is left exactly where it is and the dive is a pure
    // zoom; outside, only the excess is eased away (measured in zoom, not seconds, so it behaves
    // the same at every zoom-rate setting). Detail can sit a little off centre — it cannot wander
    // to the edge, and it never slides while it is comfortable.
    let short = aspect.min(1.0);
    let (ox, oy) = ((0.5 - aim.0) * aspect / short, (0.5 - aim.1) / short);
    let off = ox.hypot(oy);
    let pan = if off > CENTER_DEAD {
        let c = 1.0 - (-dt * (rate / ZOOM_RATE) / CENTER_TAU).exp();
        let keep = (off - CENTER_DEAD) / off; // ease away only what is past the dead zone
        ((0.5 - aim.0) * c * keep, (0.5 - aim.1) * c * keep)
    } else {
        (0.0, 0.0)
    };
    GlideStep { aim, lead, speed, factor: (-speed * dt).exp(), pan }
}

#[cfg(test)]
mod steering_tests;

/// CLI `--autodive`: drive the autopilot dive from the command line so the frame-cost controller can
/// be hammered UNPACED, and report whether the dangerous regime was actually reached.
///
/// This exists because the device-loss class could not be reproduced automatically. Every scripted
/// attempt went through `--play`, and a tour clock DILATES on a slow frame, so pressure never
/// accumulates: measured on an RTX 3080, `repro-e28-crossover` peaks at ~195 ms of measured iterate
/// against a 900 ms lethal band, and twelve arms across two nights produced zero lethal readings
/// while a human diving by hand produced 21 in one evening.
///
/// The autopilot is what a hand-driven dive *is* — it runs in the normal update loop with no clock,
/// and because it re-targets the detail-richest region it dives INTO structure, which is exactly
/// where per-step cost explodes. So this is a thin wrapper over `toggle_autopilot`, not a new
/// harness. Ruled out first, so nobody re-treads it: `--frametest` drives `build_params` and
/// reference recompute but never prices a frame (one `fd-gpu` line per run, "no reading"), and
/// `--livetest` carries its own copy of the controller.
pub(crate) struct AutoDive {
    target_log10: f64,
    timeout_s: f64,
    /// Explicit iteration count, or `None` for auto-iter.
    ///
    /// ⚠**Defaults to EXPLICIT, and that is the whole difference between a harness that reproduces
    /// and one that cannot.** The first version forced `auto_iter = true`, reasoning that both field
    /// losses ran with auto-iter on. That was wrong in a way that made the harness useless: the LIVE
    /// path deliberately caps auto-iter lower than the export appetite for responsiveness, so every
    /// frame was cheap by construction. Measured on the RX 6800 XT: 900 s of diving reached 1e112x
    /// with a peak measured iterate of **19.5 ms** against a 900 ms lethal band — lower even than the
    /// paced tour it was built to replace.
    ///
    /// The crash manifest says what the regime actually needs:
    /// `iter=250000 (gpu_iter=250000, eff=250000, boost=1.00)` with the budget ceiling at `6.000e10`
    /// — the EXPLICIT ceiling. An explicit high count with auto-iter off is what makes a deep frame
    /// expensive enough to reach the band, so that is the default here.
    iter: Option<u32>,
    t0: std::time::Instant,
    /// Frames to let the GPU and first reference come up before diving.
    warmup: u32,
    armed: bool,
    /// Last sampled measurement, to notice when a NEW one lands.
    ///
    /// ⚠Paired with the step count, because ms ALONE undercounts. Field run 2026-08-18 on the
    /// RX 6800 XT logged four LETHAL-BAND frames and this harness reported three: two consecutive
    /// readings were both exactly 1155 ms, and a value-only comparison cannot tell a repeat from a
    /// stale sample. Two identical (ms, steps) pairs back to back remain indistinguishable, so the
    /// figure is reported as DISTINCT readings rather than claimed as a total.
    last_ms: f64,
    last_steps: u64,
    readings: u32,
    lethal: u32,
    peak_ms: f64,
    deepest_log10: f64,
    /// Has the explicit iteration count been applied yet? See `EXPLICIT_FROM_LOG10`.
    iter_applied: bool,
    /// Remaining dive→Home cycles. **THE ZOOM-HOME GLIDE IS THE STRESS TEST**, and it took three
    /// failed harness designs to see it.
    ///
    /// A monotonic dive cannot stress the frame-cost controller: measured on the RX 6800 XT, diving
    /// to 1e150x produced a peak measured iterate of 36 ms against a 400 ms target, with frames
    /// CPU-bound (body 200-639 ms against 1-2 ms of wait). The controller was STARVED, not stressed —
    /// and a controller that only ever sees 36 ms frames never grows into danger. Smooth progressions
    /// let it adapt continuously, which is it working correctly.
    ///
    /// Home is the opposite: one continuous sweep from extreme depth to 1x, crossing every mode
    /// boundary while the budget is still calibrated for the deep regime. That is exactly the shape of
    /// both field losses — `mode switch to 2: budget 8.31e10 -> bootstrap 2.45e8` followed by a frame
    /// ALREADY SIZED by the old regime going out at 1.146e11 steps (405x over budget, 3040 ms), and
    /// `bla_skip` collapsing 4,457,481 -> 9,343 so per-step cost jumped ~100x under a stale budget.
    /// It is also literally the button that killed the device on 2026-08-18.
    home_left: u32,
    /// A Home glide is in flight (peaks during it are tracked separately — see `peak_home_ms`).
    homing: bool,
    homes_done: u32,
    /// Peak measured iterate observed DURING a Home glide, so the report can say whether Home is
    /// harder than the dive rather than leaving it to inference.
    peak_home_ms: f64,
    /// Cleared by the normal exit paths so the watchdog thread stands down. See `new`.
    done: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// CURRENT depth, distinct from `deepest_log10`, which is a high-water mark.
    ///
    /// ⚠Using the high-water mark for the explicit-count switch broke multi-cycle runs: after a Home
    /// glide the view is back at 1x but `deepest_log10` still reads 30, so the switch fired instantly
    /// and the re-dive started at 1x with an explicit 250k — the all-interior case where the
    /// autopilot's target search returns None and the dive dies on the spot. Measured: 2 cycles
    /// requested, 1 completed, then a timeout doing nothing.
    current_log10: f64,
}

/// Depth at which `--autodive` switches from auto-iter to its explicit iteration count.
///
/// ⚠It CANNOT be applied from frame one, and finding that out cost a run. The autopilot picks its
/// next target by looking for detail variance; with an explicit 250,000 at shallow depth the view is
/// essentially all-interior, the evaluator returns `None`, and the autopilot deactivates on the spot
/// (`autopilot.rs`, the `None =>` arm). Measured: the dive "finished" in 13.9 s at 1e0.8x having gone
/// nowhere. So dive shallow on auto-iter, where targeting works, and switch to the expensive explicit
/// count once there is real structure to aim at — which is also the honest reproduction of the field
/// case, where the user zoomed deep first and the big count was set at depth.
const EXPLICIT_FROM_LOG10: f64 = 6.0;

impl AutoDive {
    pub(crate) fn new(target_log10: f64, timeout_s: f64, iter: Option<u32>, home_cycles: u32) -> Self {
        // ⚠**THE TIMEOUT NEEDS ITS OWN THREAD.** Every other check in this harness runs inside
        // `autodive_frame`, i.e. inside `update()` — so `--autodive-timeout` could only fire while
        // frames were still being DELIVERED, which is exactly not the case that needs bounding. A
        // deep reference build runs arbitrary-precision on the main thread (hundreds of bits at
        // 1e150), and the historical "present wedge" blocks the main thread inside a wgpu wait
        // forever; both look identical from outside — the app reports "idle" with a frozen zoom, and
        // nothing enforced the deadline. Reported from the field 2026-08-18 at 1e149.
        //
        // The watchdog is deliberately dumb: sleep, check a flag, and hard-exit if the deadline
        // passes. It gets a grace margin so that whenever frames ARE flowing the in-frame path wins
        // and prints the full summary; the watchdog only ever speaks when nothing else can.
        //
        // ⚠This is why `--torture live/home/glide-from-depth` was the trustworthy way to run it
        // before now: the supervisor bounds each rung as a CHILD PROCESS from outside. Fixing the
        // flag does not make that redundant, it makes the two agree.
        const WATCHDOG_GRACE_S: f64 = 10.0;
        let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        {
            let done = std::sync::Arc::clone(&done);
            let budget = timeout_s + WATCHDOG_GRACE_S;
            std::thread::spawn(move || {
                let start = std::time::Instant::now();
                while start.elapsed().as_secs_f64() < budget {
                    if done.load(std::sync::atomic::Ordering::Relaxed) {
                        return;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(250));
                }
                if done.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }
                // ⚠One line on purpose. A `\` line-continuation inside a Rust string literal does
                // NOT strip the following indentation in this CRLF repo, so the message reaches the
                // user with a long run of spaces in the middle. Three messages shipped that way today
                // before it was worth writing down; see CONTRIBUTING.
                let msg = format!(
                    "WATCHDOG: no completion {budget:.0}s after the {timeout_s:.0}s timeout — the frame loop stopped delivering (deep reference build on the main thread, or a wedge). Nothing was measured."
                );
                crate::diag::log_line("autodive", &msg);
                eprintln!();
                eprintln!("--autodive: {msg}");
                crate::exit(4);
            });
        }
        Self {
            done,
            target_log10,
            timeout_s,
            iter,
            t0: std::time::Instant::now(),
            // Frames to let the GPU and first reference come up. `FRACTADYNE_AUTODIVE_WARMUP`
            // lengthens it so a dive can start from a view that has fully settled (and finished
            // its on-settle supersampling) — the state a user's dive starts from when they look
            // at a view before pressing A, which a 45-frame warmup does not reproduce.
            warmup: std::env::var("FRACTADYNE_AUTODIVE_WARMUP")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(45),
            armed: false,
            last_ms: 0.0,
            last_steps: 0,
            readings: 0,
            lethal: 0,
            peak_ms: 0.0,
            deepest_log10: 0.0,
            iter_applied: false,
            home_left: home_cycles,
            homing: false,
            homes_done: 0,
            peak_home_ms: 0.0,
            current_log10: 0.0,
        }
    }
}

impl crate::FractadyneApp {
    /// One frame of `--autodive`. Same in-loop shape as `uitest_frame` / `juliadive_frame`.
    pub(crate) fn autodive_frame(&mut self, ctx: &egui::Context) {
        const LOG2_10: f64 = 3.321_928_094_887_362;
        let Some(mut d) = self.harness.autodive.take() else { return };

        // Sample the controller's own measurement rather than scraping the log. `gpu_iterate=` in
        // the log only ever appears INSIDE the lethal message, so log-scraping is circular — it can
        // only report a number once the thing being detected has already happened.
        let ms = self.perf.last_iterate_ms[0];
        let steps = self.perf.fe_steps_last[0];
        if ms > 0.0 && ((ms - d.last_ms).abs() > f64::EPSILON || steps != d.last_steps) {
            d.readings += 1;
            if ms > d.peak_ms {
                d.peak_ms = ms;
            }
            if ms >= crate::tunables::cost().tdr_lethal_ms {
                d.lethal += 1;
            }
            if d.homing && ms > d.peak_home_ms {
                d.peak_home_ms = ms;
            }
            d.last_ms = ms;
            d.last_steps = steps;
        }
        let depth = self.viewport.log2_magnification() / LOG2_10;
        if depth.is_finite() {
            d.current_log10 = depth;
            if depth > d.deepest_log10 {
                d.deepest_log10 = depth;
            }
        }

        if !d.armed {
            if d.warmup > 0 {
                d.warmup -= 1;
            } else {
                // Field conditions, from the crash manifest: an EXPLICIT iteration count with
                // auto-iter OFF. See the note on `AutoDive::iter` for why auto-iter is the wrong
                // choice here despite the losses having run with it.
                // Always start on auto-iter so the autopilot's target evaluation has detail to
                // find; the explicit count lands at EXPLICIT_FROM_LOG10 (see that note).
                self.render_cfg.auto_iter = true;
                self.autopilot.dive_log2 = d.target_log10 * LOG2_10;
                self.toggle_autopilot(ctx);
                d.armed = true;
                crate::diag::log_line(
                    "autodive",
                    &format!(
                        "diving to 1e{:.0}x ({}, lethal band >= {:.0}ms)",
                        d.target_log10,
                        match d.iter {
                            Some(n) => format!("explicit iter={n}"),
                            None => "auto-iter".to_string(),
                        },
                        crate::tunables::cost().tdr_lethal_ms
                    ),
                );
            }
        }

        // Switch to the explicit count once the dive has structure to aim at.
        if d.armed && !d.iter_applied && d.current_log10 >= EXPLICIT_FROM_LOG10 {
            if let Some(n) = d.iter {
                self.render_cfg.max_iter = n.clamp(64, crate::MAX_ITER_LIMIT);
                self.render_cfg.auto_iter = false;
                crate::diag::log_line(
                    "autodive",
                    &format!("1e{:.1}x reached — switching to explicit iter={n}", d.current_log10),
                );
            }
            d.iter_applied = true;
        }

        // Test hook: freeze the frame loop on purpose so the watchdog above can be OBSERVED firing.
        // A safety net nobody has seen catch anything is a guess.
        if std::env::var("FRACTADYNE_AUTODIVE_FREEZE").is_ok() && d.armed {
            crate::diag::log_line("autodive", "FRACTADYNE_AUTODIVE_FREEZE — blocking the frame loop");
            loop {
                std::thread::sleep(std::time::Duration::from_secs(3600));
            }
        }

        let elapsed = d.t0.elapsed().as_secs_f64();
        let dive_done = d.armed && !self.autopilot.active;
        let timed_out = elapsed >= d.timeout_s;

        // ---- dive → Home cycling: the Home glide is the part that actually stresses the controller.
        if !timed_out && d.homing {
            if self.home_anim.is_none() {
                d.homes_done += 1;
                d.homing = false;
                crate::diag::log_line(
                    "autodive",
                    &format!(
                        "home glide {} complete (peak during home {:.1}ms)",
                        d.homes_done, d.peak_home_ms
                    ),
                );
                if d.home_left > 0 {
                    // Re-dive. Back to auto-iter first: at 1x an explicit 250k makes the view
                    // all-interior and the autopilot's target search returns None immediately (the
                    // 1e0.8x non-dive). The explicit count re-lands at EXPLICIT_FROM_LOG10.
                    self.render_cfg.auto_iter = true;
                    d.iter_applied = false;
                    self.autopilot.dive_log2 = d.target_log10 * LOG2_10;
                    self.toggle_autopilot(ctx);
                }
            }
            self.harness.autodive = Some(d);
            ctx.request_repaint();
            return;
        }
        if !timed_out && dive_done && d.home_left > 0 {
            d.home_left -= 1;
            d.homing = true;
            let now = ctx.input(|i| i.time);
            crate::diag::log_line(
                "autodive",
                &format!("1e{:.1}x reached — ZOOM HOME (the stress test)", d.current_log10),
            );
            self.zoom_home(now);
            self.harness.autodive = Some(d);
            ctx.request_repaint();
            return;
        }

        if dive_done || timed_out {
            // ⚠Distinguish "reached the target" from "the autopilot quit on us". The first version
            // called any inactive autopilot a "dive limit reached", which reported success at 1e0.8x
            // against a 1e45 target — a verdict that lied in the direction of looking fine.
            let reached = d.deepest_log10 >= d.target_log10 - 0.5;
            let _ = dive_done;
            let why = if timed_out {
                "timeout"
            } else if reached {
                "dive limit reached"
            } else {
                "AUTOPILOT STOPPED EARLY (no detail target, or interrupted) — dive did not finish"
            };
            // The verdict, stated so a run that never reached the regime cannot be read as a pass.
            // This is the whole point of the harness: a green exit with zero lethal readings means
            // the experiment did not run, not that the code is safe.
            let verdict = if d.lethal > 0 {
                format!("REACHED THE REGIME: {} lethal reading(s)", d.lethal)
            } else {
                "DID NOT REACH THE REGIME - no lethal reading, so nothing was tested".to_string()
            };
            crate::diag::log_line(
                "autodive",
                &format!(
                    "{why} after {elapsed:.1}s | deepest 1e{:.1}x | {} reading(s), peak measured \
                     iterate {:.1}ms | {verdict}",
                    d.deepest_log10, d.readings, d.peak_ms
                ),
            );
            println!();
            println!("--autodive: {why} after {elapsed:.1}s");
            println!("  deepest depth      1e{:.1}x", d.deepest_log10);
            println!("  iteration ask      {}",
                match d.iter { Some(n) => format!("{n} (explicit)"), None => "auto".to_string() });
            println!("  controller readings {}", d.readings);
            println!("  peak measured iterate {:.1}ms (lethal band >= {:.0}ms)",
                d.peak_ms, crate::tunables::cost().tdr_lethal_ms);
            println!("  home glides         {} (peak during home {:.1}ms)",
                d.homes_done, d.peak_home_ms);
            println!("  lethal readings     {} (distinct)", d.lethal);
            println!("  {verdict}");
            // Exit 2, not 3: this codebase reserves 2 for "ran fine, the RESULT is wrong"
            // (`--bench-matrix` uses it for algorithmic drift) and `torture::classify` maps it to
            // FailAssert, where any other non-zero code becomes FailCrash. A clean run that simply
            // did not reach the regime is an assertion failure, not a crash, and mislabelling it
            // would send whoever reads the ladder report hunting a crash that never happened.
            d.done.store(true, std::sync::atomic::Ordering::Relaxed);
            crate::exit(if d.lethal > 0 { 0 } else { 2 });
        }

        self.harness.autodive = Some(d);
        ctx.request_repaint();
    }
}
