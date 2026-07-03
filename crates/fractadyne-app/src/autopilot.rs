//! Auto-zoom autopilot: a hands-free dive that re-targets the detail-richest region and
//! eases the zoom pivot toward it each frame (View menu / A key; Esc or any input stops).

use crate::{FractadyneApp, ZOOM_RATE};
use eframe::egui;

/// Auto-zoom autopilot: seconds between target re-evaluations, and the depth (zoom
/// octaves ≈ log₂ magnification) at which it stops — kept in the smooth, fast regime.
const AUTOPILOT_EVAL_INTERVAL: f64 = 0.35;
const AUTOPILOT_LOG2_CAP: f64 = 900.0; // ≈ 1e271×
/// Time constant (s) for easing the zoom pivot toward each newly-evaluated target — larger
/// is smoother but lags the detail more.
const AUTOPILOT_TARGET_TAU: f64 = 0.5;

impl FractadyneApp {
    /// Toggle the auto-zoom autopilot (single view only).
    pub(crate) fn toggle_autopilot(&mut self, ctx: &egui::Context) {
        self.autopilot = !self.autopilot;
        if self.autopilot {
            self.autopilot_target = (0.5, 0.5);
            self.autopilot_goal = (0.5, 0.5);
            self.autopilot_eval_t = 0.0; // force an evaluation next frame
            self.home_anim = None;
            self.playback = None;
            self.set_toast("Autopilot on — diving toward detail (any input stops)", ctx);
        } else {
            self.zoom_vel = 0.0;
            self.set_toast("Autopilot off", ctx);
        }
    }

    /// One frame of the auto-zoom autopilot: continuously zoom toward `autopilot_target`,
    /// re-evaluating the target every `AUTOPILOT_EVAL_INTERVAL` by rendering a small
    /// iteration field of the current view and steering to its detail-richest,
    /// boundary-adjacent region. Stops on manual input, a dead end, or the depth cap.
    pub(crate) fn autopilot_step(
        &mut self,
        ctx: &egui::Context,
        gpu: &Option<(eframe::wgpu::Device, eframe::wgpu::Queue)>,
    ) {
        if !self.autopilot {
            return;
        }
        // Any manual navigation (or dual view) hands control back to the user.
        let interrupted = ctx.input(|i| {
            i.pointer.any_down() || i.smooth_scroll_delta.y != 0.0 || i.key_down(egui::Key::Space)
        });
        if interrupted || self.dual {
            self.autopilot = false;
            self.zoom_vel = 0.0;
            return;
        }
        if self.viewport.log2_magnification() > AUTOPILOT_LOG2_CAP {
            self.autopilot = false;
            self.zoom_vel = 0.0;
            self.set_toast("Autopilot: depth cap reached", ctx);
            return;
        }
        let now = ctx.input(|i| i.time);
        let dt = (ctx.input(|i| i.stable_dt) as f64).clamp(0.0, 0.1);

        if now - self.autopilot_eval_t > AUTOPILOT_EVAL_INTERVAL {
            self.autopilot_eval_t = now;
            if let Some((dev, q)) = gpu {
                match self.autopilot_pick_target(dev, q) {
                    // Just update the goal — the pivot eases toward it below, every frame,
                    // so re-evaluation never jumps the zoom point.
                    Some((tx, ty)) => self.autopilot_goal = (tx, ty),
                    None => {
                        self.autopilot = false;
                        self.zoom_vel = 0.0;
                        self.set_toast("Autopilot: no detail ahead (stopped)", ctx);
                        return;
                    }
                }
            }
        }

        // Glide the zoom pivot toward the goal continuously (time-constant smoothing), so the
        // panning direction changes smoothly rather than snapping at each re-evaluation.
        let follow = 1.0 - (-dt / AUTOPILOT_TARGET_TAU).exp();
        self.autopilot_target.0 += (self.autopilot_goal.0 - self.autopilot_target.0) * follow;
        self.autopilot_target.1 += (self.autopilot_goal.1 - self.autopilot_target.1) * follow;

        // Continuously zoom in toward the (smoothly moving) target screen fraction.
        let rate = ZOOM_RATE * self.zoom_rate as f64;
        let factor = (-rate * dt).exp();
        let px = self.autopilot_target.0 * self.viewport.width_px;
        let py = self.autopilot_target.1 * self.viewport.height_px;
        self.viewport.zoom_at(px, py, factor);
        self.settle_t = [now; 2]; // treat as interaction (AA off, throttled reference refresh)
        self.schedule_repaint(ctx);
    }

    /// Render a small iteration field of the current view and return the screen-fraction
    /// of the detail-richest, boundary-adjacent region (center-biased for a stable dive).
    /// `None` when the view holds no boundary detail (dead end → stop).
    fn autopilot_pick_target(&self, dev: &eframe::wgpu::Device, q: &eframe::wgpu::Queue) -> Option<(f64, f64)> {
        const N: usize = 56;
        let mut req = self.current_export_request_for(&self.viewport, self.julia_mode);
        req.width = N as u32;
        req.height = N as u32;
        req.ss = 1;
        let px = fractadyne_gpu::render_iter(dev, q, &req).ok()?.pixels;
        if px.len() < N * N * 4 {
            return None;
        }
        let r = |i: usize, j: usize| px[(j * N + i) * 4] as f64; // smooth iter; < 0 = interior
        let (cx, cy) = ((N as f64 - 1.0) * 0.5, (N as f64 - 1.0) * 0.5);
        let maxd = (cx * cx + cy * cy).sqrt();
        let (mut best, mut best_ij) = (0.0f64, None);
        for j in 1..N - 1 {
            for i in 1..N - 1 {
                let c = r(i, j);
                if c < 0.0 {
                    continue; // never target interior cells
                }
                let nb = [r(i - 1, j), r(i + 1, j), r(i, j - 1), r(i, j + 1)];
                let touches_interior = nb.iter().any(|&v| v < 0.0);
                // Local exterior gradient (finite neighbours only) = escape-time detail.
                let grad: f64 = nb.iter().filter(|&&v| v >= 0.0).map(|&v| (v - c).abs()).sum();
                // Boundary cells (adjacent to the set) carry the richest structure.
                let mut interest = grad + if touches_interior { 50.0 } else { 0.0 };
                if interest <= 0.0 {
                    continue;
                }
                // Center bias: keep the dive stable and the focus on-screen.
                let d = (((i as f64 - cx).powi(2) + (j as f64 - cy).powi(2)).sqrt()) / maxd;
                interest *= 1.0 - 0.6 * d;
                if interest > best {
                    best = interest;
                    best_ij = Some((i, j));
                }
            }
        }
        let (bi, bj) = best_ij?;
        if best < 1.0 {
            return None; // no real detail (flat exterior / all interior) → dead end
        }
        Some(((bi as f64 + 0.5) / N as f64, (bj as f64 + 0.5) / N as f64))
    }
}
