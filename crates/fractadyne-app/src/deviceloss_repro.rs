//! `--deviceloss-repro` — a SAFE, deterministic reproduction of the deep-interior device loss.
//!
//! ## The specimen
//!
//! The 2026-09-10 field capture (`crash-1789060790`) lost the GPU at a deep INTERIOR minibrot with
//! `Iterations (base) = 10,000,000`. Because the view is interior the reference orbit never escapes,
//! so it builds all the way to the ~7.4M-sample device buffer cap (`partial=true`), and a single
//! full-resolution GPU iterate dispatch there overran the driver watchdog. The new, unexplained
//! finding: the frame-cost throttle had already SHED to its 256-iteration floor, and the device
//! **still** died — so the cost is not the iteration window the throttle shrinks.
//!
//! ## Why this harness is safe
//!
//! It never submits the lethal dispatch. The cost of one 256-iteration ("floor") window is set by
//! `pixels × 256 × per-step-cost`, and — crucially — is INDEPENDENT of the reference LENGTH (a
//! 256-iteration window reads at most 256 reference entries whatever the orbit's total size). So the
//! danger is characterised by measuring the floor-window wall cost at SMALL pixel counts, timed the
//! honest way (`render_iter_chunked_timed` → one submission alone against `poll(Wait)`, the same
//! signal `--chunk-sweep` uses), scaling the area up only while it stays under a safety bound, and
//! EXTRAPOLATING to the field's full resolution. A lethal full-res dispatch is predicted, never run.
//!
//! ## What it answers
//!
//! 1. Is the full-resolution FLOOR window (256 iterations) predicted to exceed the watchdog? If yes,
//!    shedding the iteration window cannot save this view — the field's "shed-to-floor still lost".
//! 2. How does cost scale with pixel AREA? `wall ~ area^1` means a spatial split (tiling the frame
//!    below the iteration floor) IS a sufficient actuator — the one axis nothing else reaches — and
//!    says how many tiles bound one dispatch under the frame budget.
//!
//! Invoked as `--deviceloss-repro [ZOOM_LOG2] [REF_ITER]` (defaults: 54 ≈ 1.8e16×, ref 1,000,000).
//! The view is fixed to the Seahorse period-998 nucleus — a KNOWN interior point (its orbit is
//! periodic, so it never escapes), the field's own family. The reference length does not change the
//! per-window cost, so a smaller `REF_ITER` builds faster and measures the same thing.

use crate::FractadyneApp;

/// The Seahorse period-998 nucleus — the deep test battery's `NX/NY`, an EXACT interior point (its
/// critical orbit is periodic ⇒ bounded ⇒ the reference never escapes ⇒ it builds to the device cap,
/// `partial=true`, which is the specimen). Carried to full precision so it lands on the nucleus.
const NX: &str = "-0.7436438870371588707780645434936425750476099623212550602141";
const NY: &str = "0.1318259042053122928210973548747672652629885996790429749374";

/// The field capture's live render resolution — the full-res target the safe measurements
/// extrapolate to. `crash-1789060790` manifest: `1728x1201`.
const TARGET_W: u32 = 1728;
const TARGET_H: u32 = 1201;

/// The live path's own iteration-window floor (`render.rs` `floor` when `fe_budget != 0`). The whole
/// question is whether even THIS is lethal at full resolution.
const FLOOR_WINDOW: u32 = 256;

/// Stop scaling the area up once one floor-window submission costs this much. Below the ~2 s Windows
/// TDR and above the ~900 ms lethal band, so the sweep measures INSIDE the dangerous band without
/// stepping past it (same bound and reasoning as `chunksweep::SAFETY_MS`).
const SAFETY_MS: f64 = 1200.0;

/// The frame-cost budget's wall target — one dispatch should finish well inside this. Used only to
/// report how many spatial tiles would bound the full-res floor window (the fix's sizing target).
const BUDGET_TARGET_MS: f64 = 400.0;

/// Pixel dimensions walked on the area axis, smallest first — a true sub-rect at native scale (fewer
/// pixels of the SAME view), so each measures the real per-pixel cost rather than a coarser grid.
const AREA_STEPS: &[(u32, u32)] =
    &[(128, 96), (192, 144), (256, 192), (384, 288), (512, 384), (768, 576), (1024, 768)];

pub(crate) struct DeviceLossRepro {
    /// `--center X Y` — target an arbitrary view (the field's exact hot location, a session view).
    /// `None` uses the canonical Seahorse-998 nucleus. Parsed here rather than relying on the
    /// startup view-override, which does not reach this mode.
    center: Option<(String, String)>,
    zoom_log2: f64,
    ref_iter: u32,
    done: bool,
}

impl DeviceLossRepro {
    /// Parse `--deviceloss-repro [ZOOM_LOG2] [REF_ITER]`, plus optional `--center X Y`,
    /// `--zoom-log2 L`, `--iter N` anywhere on the line (named forms win over the positionals).
    pub(crate) fn from_args(args: &[String]) -> Option<Self> {
        let i = args.iter().position(|a| a == "--deviceloss-repro")?;
        // Positionals directly after the flag, used only when they actually parse as a number (so
        // `--deviceloss-repro --center …` does not read "--center" as the zoom).
        let pos = |k: usize| args.get(i + k).and_then(|v| v.parse::<f64>().ok()).filter(|n| *n > 0.0);
        let named = |name: &str| {
            args.iter()
                .position(|a| a == name)
                .and_then(|j| args.get(j + 1))
                .and_then(|v| v.parse::<f64>().ok())
        };
        let center = args
            .iter()
            .position(|a| a == "--center")
            .and_then(|j| Some((args.get(j + 1)?.clone(), args.get(j + 2)?.clone())));
        let zoom_log2 = named("--zoom-log2").or_else(|| pos(1)).unwrap_or(54.0);
        let ref_iter = named("--iter")
            .or_else(|| pos(2))
            .map(|n| n as u32)
            .unwrap_or(1_000_000)
            .clamp(50_000, 100_000_000);
        Some(Self { center, zoom_log2, ref_iter, done: false })
    }
}

/// One measured (pixel count, worst floor-window wall_ms) point.
struct AreaPoint {
    px: f64,
    worst_ms: f64,
}

impl FractadyneApp {
    /// Drive the repro. Runs once, then asks the app to close. Returns `true` when done.
    pub(crate) fn deviceloss_repro_step(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
    ) -> bool {
        let Some(r) = self.harness.deviceloss_repro.as_ref() else { return false };
        if r.done {
            return true;
        }
        let (zoom_log2, ref_iter, center) = (r.zoom_log2, r.ref_iter, r.center.clone());
        self.harness.deviceloss_repro.as_mut().unwrap().done = true;
        self.run_deviceloss_repro(device, queue, zoom_log2, ref_iter, center);
        true
    }

    fn run_deviceloss_repro(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        zoom_log2: f64,
        ref_iter: u32,
        center: Option<(String, String)>,
    ) {
        println!(
            "Fractadyne device-loss repro — {}\n\
             Safe, deterministic reproduction of the deep-interior device loss: measures the honest\n\
             floor-window submission cost at small areas and extrapolates to full resolution. It\n\
             predicts the lethal full-res dispatch; it never submits one.\n",
            crate::version_string()
        );

        // The view: `--center X Y` targets an arbitrary location (the field's exact hot spot — a
        // STRUCTURED deep view is far hotter per step than a solid interior body, which is where the
        // lethal regime lives); otherwise the canonical Seahorse-998 nucleus (a KNOWN interior point).
        let (cx_s, cy_s, custom) = match &center {
            Some((x, y)) => (x.clone(), y.clone(), true),
            None => (NX.to_string(), NY.to_string(), false),
        };
        let (Some(cx), Some(cy)) =
            (fractadyne_core::parse_bf(&cx_s), fractadyne_core::parse_bf(&cy_s))
        else {
            println!("ABORT: the centre coordinates did not parse.");
            return;
        };
        self.fractal = crate::FractalKind::Mandelbrot;
        self.julia_mode = false;
        self.viewport.set_size(TARGET_W as f64, TARGET_H as f64);
        self.viewport.set_center_log2mag(cx, cy, zoom_log2);
        self.viewport.precision =
            fractadyne_core::precision_for_octaves(zoom_log2.max(0.0).ceil() as u64);
        self.render_cfg.auto_iter = false;
        self.render_cfg.max_iter = ref_iter;
        let home = !custom;

        // Build the reference SYNCHRONOUSLY (a multi-minute bignum build for a large interior orbit;
        // the watchdog will note the pause — that is expected). `current_export_request_for` runs
        // `recompute_worker` inline and carries the orbit into the request.
        println!(
            "view      : centre {} , {}\n\
             zoom      : 2^{zoom_log2:.1} (~{:.2e}x){}\n\
             building reference at iter={ref_iter} (a non-escaping/interior orbit builds to the cap) ...",
            fractadyne_core::to_decimal_string(&self.viewport.center_x),
            fractadyne_core::to_decimal_string(&self.viewport.center_y),
            2f64.powf(zoom_log2.min(1020.0)),
            if home { "  [canonical Seahorse-998 nucleus]" } else { "  [--center override]" },
        );
        let t0 = std::time::Instant::now();
        let base = self.current_export_request_for(&self.viewport, false);
        let build_s = t0.elapsed().as_secs_f64();
        println!(
            "reference : built in {build_s:.1}s — orbit_len={} partial={} sa_skip={} mode={} formula={} prec={}",
            base.orbit_len,
            // `partial` is not on the request; a non-escaping orbit that reached the cap shows as
            // orbit_len == the ask (or the device cap). Report orbit_len; the interior condition is
            // that it did NOT escape early.
            base.orbit_len >= ref_iter.min(crate::render::orbit_len_cap()),
            base.sa_skip,
            base.mode,
            base.formula,
            self.viewport.precision,
        );
        if base.orbit_len == 0 {
            println!("ABORT: no reference (orbit_len=0) — a non-perturbation view. Pick a deeper zoom.");
            return;
        }
        if base.mode > 2 || base.formula > 3 {
            println!(
                "ABORT: mode {} / formula {} is outside the chunk shaders' scope; render_iter_chunked_timed\n\
                 would fall back to one unbounded dispatch and measure nothing.",
                base.mode, base.formula
            );
            return;
        }

        // Bracket a couple of REAL floor windows just past the SA skip (windows ending at/below
        // sa_skip are free by construction — the shader seeds iter=sa_skip). Two windows past it is
        // enough to read the worst; walking deeper only multiplies the count.
        let skip = base.sa_skip;
        let bracket = skip.saturating_add(FLOOR_WINDOW * 3);
        let target_px = (TARGET_W as f64) * (TARGET_H as f64);

        println!(
            "\n── floor-window ({FLOOR_WINDOW} iters) cost vs pixel area ── (worst single submission past the SA skip)"
        );
        let mut pts: Vec<AreaPoint> = Vec::new();
        for &(w, h) in AREA_STEPS {
            let (w, h) = (w & !1u32, h & !1u32);
            let mut req = base.clone();
            req.width = w;
            req.height = h;
            req.ss = 1;
            req.max_iter = bracket;
            let mut passes = Vec::new();
            match fractadyne_gpu::render_iter_chunked_timed(device, queue, &req, FLOOR_WINDOW, &mut passes) {
                Ok(_) if !passes.is_empty() => {
                    let worst = passes
                        .iter()
                        .filter(|p| p.end_iter > skip)
                        .map(|p| p.wall_ms)
                        .fold(0.0_f64, f64::max);
                    let px = (w as f64) * (h as f64);
                    println!("  {w:>5}x{h:<5} ({px:>10.0} px): worst floor window {worst:8.1} ms");
                    pts.push(AreaPoint { px, worst_ms: worst });
                    if worst > SAFETY_MS {
                        println!(
                            "  ── STOPPED: {worst:.0} ms is past the {SAFETY_MS:.0} ms safety bound; the next\n\
                             \x20   area up would approach the ~2 s watchdog. Full res is extrapolated, not run."
                        );
                        break;
                    }
                }
                Ok(_) => {
                    println!("  {w:>5}x{h:<5}: NOT MEASURED (unsupported-scope fallback ran one unbounded dispatch)");
                }
                Err(e) => println!("  {w:>5}x{h:<5}: GPU ERROR — {e}"),
            }
        }

        // ── axis 2: WINDOW size at a fixed safe area — the actuator the field's loss actually used.
        // The area axis fixed the window at the floor and found it cheap; the field's lethal
        // in-flight pass was a 10,236-iteration WINDOW, not the 256 floor. Grow the window at a small
        // (safe) area, then combine with the area exponent to predict the window size whose full-res
        // dispatch hits the watchdog.
        let (safe_w, safe_h) = (512u32, 384u32);
        let safe_px = (safe_w as f64) * (safe_h as f64);
        println!(
            "\n── window-size cost at {safe_w}x{safe_h} (safe) ── (worst single submission, one full window past the SA skip)"
        );
        let mut win_pts: Vec<(f64, f64)> = Vec::new();
        for &win in &[FLOOR_WINDOW, 1024u32, 4096, 16384, 65536, 262144] {
            let mut req = base.clone();
            req.width = safe_w;
            req.height = safe_h;
            req.ss = 1;
            req.max_iter = skip.saturating_add(win.saturating_mul(2));
            let mut passes = Vec::new();
            match fractadyne_gpu::render_iter_chunked_timed(device, queue, &req, win, &mut passes) {
                Ok(_) if !passes.is_empty() => {
                    let worst = passes
                        .iter()
                        .filter(|p| p.end_iter > skip && p.end_iter - p.start_iter == win)
                        .map(|p| p.wall_ms)
                        .fold(0.0_f64, f64::max);
                    if worst > 0.0 {
                        println!("  window {win:>7}: worst {worst:8.1} ms");
                        win_pts.push((win as f64, worst));
                    }
                    if worst > SAFETY_MS {
                        println!("  ── STOPPED: past the {SAFETY_MS:.0} ms safety bound.");
                        break;
                    }
                }
                Ok(_) => println!("  window {win:>7}: NOT MEASURED (scope fallback)"),
                Err(e) => println!("  window {win:>7}: GPU ERROR — {e}"),
            }
        }

        self.report_deviceloss_verdict(&pts, &win_pts, safe_px, target_px);
    }

    fn report_deviceloss_verdict(
        &self,
        pts: &[AreaPoint],
        win_pts: &[(f64, f64)],
        safe_px: f64,
        target_px: f64,
    ) {
        println!("\n── verdict ──");
        if pts.len() < 2 {
            println!("  Not enough points to fit. No verdict.");
            return;
        }
        // Fit worst_ms ~ px^a in log-log (least squares), and separately take the highest measured
        // rate (ms per pixel) as a conservative linear predictor — a≈1 is the expectation for a
        // fixed-iteration window (every pixel does the same 256 steps).
        let n = pts.len() as f64;
        let sx: f64 = pts.iter().map(|p| p.px.ln()).sum();
        let sy: f64 = pts.iter().map(|p| p.worst_ms.ln()).sum();
        let sxx: f64 = pts.iter().map(|p| p.px.ln() * p.px.ln()).sum();
        let sxy: f64 = pts.iter().map(|p| p.px.ln() * p.worst_ms.ln()).sum();
        let a = (n * sxy - sx * sy) / (n * sxx - sx * sx);
        let inter = (sy - a * sx) / n;
        let fit_at = |px: f64| (inter + a * px.ln()).exp();
        let predicted = fit_at(target_px);
        // Conservative cross-check: the largest per-pixel rate seen, scaled linearly to full res.
        let worst_rate = pts.iter().map(|p| p.worst_ms / p.px).fold(0.0_f64, f64::max);
        let linear_pred = worst_rate * target_px;

        println!("  cost ~ area^{a:.3}  (1.00 = a pixel costs the same everywhere; the expectation here)");
        println!(
            "  predicted FULL-RES floor window ({}x{} = {:.0} px): ~{:.0} ms (fit), ~{:.0} ms (worst-rate × area)",
            TARGET_W, TARGET_H, target_px, predicted, linear_pred
        );
        let worst_case = predicted.max(linear_pred);
        if worst_case >= 1000.0 {
            let tiles = (worst_case / BUDGET_TARGET_MS).ceil() as u32;
            println!(
                "  ⇒ THE FLOOR IS LETHAL AT FULL RESOLUTION (~{worst_case:.0} ms ≥ the ~1 s watchdog).\n\
                 \x20  Shedding the iteration window cannot save this view — 256 is already the floor and\n\
                 \x20  the field capture confirms shedding to it still lost the device. Reproduced.\n\
                 \x20  a≈{a:.2} means AREA is the actuator that remains: splitting one full-res floor\n\
                 \x20  dispatch into ~{tiles} native-scale tiles keeps each under the {BUDGET_TARGET_MS:.0} ms budget."
            );
        } else if worst_case >= SAFETY_MS * 0.5 {
            println!(
                "  ⇒ The full-res floor window lands in the lethal BAND (~{worst_case:.0} ms) but below the\n\
                 \x20  ~1 s watchdog — marginal. A slower region (a few× hotter) tips it over, which is the\n\
                 \x20  field variability. A spatial split still gives the margin; a≈{a:.2}."
            );
        } else {
            println!(
                "  ⇒ The full-res floor window is ~{worst_case:.0} ms — under the lethal band at this depth.\n\
                 \x20  The field loss here was a LARGER window (before the shed) or a deeper/slower region;\n\
                 \x20  re-run at a greater ZOOM_LOG2, or measure a deeper iteration bracket."
            );
        }
        // ── the WINDOW actuator: at what iteration window does a FULL-RES dispatch hit the watchdog?
        // This is the field's real question — its lethal in-flight pass was a large window, not the
        // floor. Fit worst ~ window^b at the safe area, scale to full res by the AREA exponent above,
        // and solve for the window that reaches ~1 s.
        if win_pts.len() >= 2 {
            let m = win_pts.len() as f64;
            let wx: f64 = win_pts.iter().map(|(w, _)| w.ln()).sum();
            let wy: f64 = win_pts.iter().map(|(_, t)| t.ln()).sum();
            let wxx: f64 = win_pts.iter().map(|(w, _)| w.ln() * w.ln()).sum();
            let wxy: f64 = win_pts.iter().map(|(w, t)| w.ln() * t.ln()).sum();
            let b = (m * wxy - wx * wy) / (m * wxx - wx * wx);
            let winter = (wy - b * wx) / m;
            // Scale the safe-area window fit to full res by the area exponent measured above.
            let area_factor = (target_px / safe_px).powf(a);
            println!(
                "\n  window actuator: cost ~ window^{b:.2} (1.00 = linear in iterations, the expectation).\n\
                 \x20 scaled to full res by area^{a:.2} = ×{area_factor:.1}."
            );
            // Solve worst_full(W) = 1000 ms.  exp(winter + b·lnW)·area_factor = 1000
            let w_lethal = (((1000.0_f64).ln() - winter - area_factor.ln()) / b).exp();
            if w_lethal.is_finite() && w_lethal > 0.0 {
                println!(
                    "  ⇒ at full resolution, a single window LARGER than ~{:.0} iterations is predicted to\n\
                     \x20  cross the ~1 s watchdog. The field's lethal in-flight pass was size 10,236 — {}.\n\
                     \x20  So the actuator IS the iteration window (the shed is right), and the danger is\n\
                     \x20  the FIRST large window issued before any measurement exists — never issuing one\n\
                     \x20  needs a conservative first dispatch (small window × small tile), since window^{b:.1}\n\
                     \x20  says nominal steps DO bound cost here, they just aren't known until measured.",
                    w_lethal,
                    if 10236.0 >= w_lethal { "past that line ⇒ REPRODUCED as the lethal window" } else { "below it here — a hotter (structured) location lowers this bound" }
                );
            }
        }
        println!(
            "\n  (Measured on {} — {}. Cost is hardware-specific; the SHAPE — the area and window\n\
             \x20 exponents and which axis is the actuator — is what transfers.)",
            self.gpu_name.trim(),
            self.gpu_backend.trim(),
        );
    }
}
