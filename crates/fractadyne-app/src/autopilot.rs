//! Auto-zoom autopilot: a hands-free dive that re-targets the detail-richest region and
//! eases the zoom pivot toward it each frame (View menu / A key; Esc or any input stops).

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
/// Time constant (s) for easing the zoom pivot toward each newly-evaluated target — larger
/// is smoother but lags the detail more.
const AUTOPILOT_TARGET_TAU: f64 = 0.5;

impl FractadyneApp {
    /// Toggle the auto-zoom autopilot (single view only).
    pub(crate) fn toggle_autopilot(&mut self, ctx: &egui::Context) {
        self.autopilot.active = !self.autopilot.active;
        if self.autopilot.active {
            self.autopilot.target = (0.5, 0.5);
            self.autopilot.goal = (0.5, 0.5);
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

    /// One frame of the auto-zoom autopilot: continuously zoom toward `autopilot_target`,
    /// re-evaluating the target every `AUTOPILOT_EVAL_INTERVAL` by rendering a small
    /// iteration field of the current view and steering to its detail-richest,
    /// boundary-adjacent region. Stops on manual input, a dead end, or the depth cap.
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
                match self.autopilot_pick_target(dev, q) {
                    Some((tx, ty)) => {
                        self.autopilot.goal = (tx, ty);
                        if stepping {
                            // Stepped dive: snap the pivot to the detail and jump the zoom by a
                            // fixed factor (2^AUTOPILOT_STEP_LOG2×) toward it.
                            self.autopilot.target = (tx, ty);
                            let factor = (-AUTOPILOT_STEP_LOG2 * std::f64::consts::LN_2).exp();
                            let px = tx * self.viewport.width_px;
                            let py = ty * self.viewport.height_px;
                            self.viewport.zoom_at(px, py, factor);
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
            // Smooth glide: ease the pivot toward the goal (so the pan direction changes smoothly
            // rather than snapping at each re-evaluation) and zoom in continuously.
            let follow = 1.0 - (-dt / AUTOPILOT_TARGET_TAU).exp();
            self.autopilot.target.0 += (self.autopilot.goal.0 - self.autopilot.target.0) * follow;
            self.autopilot.target.1 += (self.autopilot.goal.1 - self.autopilot.target.1) * follow;
            let rate = ZOOM_RATE * self.render_cfg.zoom_rate as f64;
            let factor = (-rate * dt).exp();
            let px = self.autopilot.target.0 * self.viewport.width_px;
            let py = self.autopilot.target.1 * self.viewport.height_px;
            self.viewport.zoom_at(px, py, factor);
        }
        self.pointer.settle_t = [now; 2]; // treat as interaction (AA off, throttled reference refresh)
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
            warmup: 45,
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
        let Some(mut d) = self.autodive.take() else { return };

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
            self.autodive = Some(d);
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
            self.autodive = Some(d);
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

        self.autodive = Some(d);
        ctx.request_repaint();
    }
}
