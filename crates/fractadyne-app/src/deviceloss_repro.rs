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
    /// `--m-jump`: before measuring, do the minibrot-finder jump (the `M` key) from the view — the
    /// field crash was "hit M during a zoom", which lands on the minibrot's OWN deep scale, a far
    /// hotter interior than the view M was pressed from.
    m_jump: bool,
    /// `--throttled-iter N`: the per-dispatch budget the LIVE frame-cost throttle would hand this
    /// view — the field's `gpu_iter` (2,843,648 in crash-1789092955). When it is BELOW the reference's
    /// SA skip, `usable_sa_skip` refuses the skip and the shader grinds from zero: the SA-skip
    /// INVERSION. The harness reproduces that decision and measures what `sa_skip_rescue` does to it.
    throttled_iter: Option<u32>,
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
        let m_jump = args.iter().any(|a| a == "--m-jump");
        let throttled_iter = named("--throttled-iter").map(|n| n as u32).filter(|n| *n > 0);
        Some(Self { center, zoom_log2, ref_iter, m_jump, throttled_iter, done: false })
    }
}

/// One measured (pixel count, worst floor-window wall_ms) point.
struct AreaPoint {
    px: f64,
    worst_ms: f64,
}

/// The live throttle's per-dispatch budget in the field crash whose SA-skip inversion this harness
/// reproduces: `crash-1789092955` manifest `gpu_iter=2843648`, below its reference's SA skip of
/// 7,452,443. Override with `--throttled-iter N`.
const FIELD_THROTTLED_ITER: u32 = 2_843_648;

/// The SA-skip inversion reproduced against the built reference (see `measure_sa_skip_inversion`).
struct Inversion {
    /// The live throttle's per-dispatch budget (the field's `gpu_iter`).
    throttled: u32,
    /// The reference's SA skip — above `throttled` here, which is what gets it refused.
    skip: u32,
    /// `sa_skip_rescue(throttled, skip, orbit_len)`: the raised budget, or `throttled` when no rescue.
    rescued_iter: u32,
    /// `usable_sa_skip(skip, rescued_iter)`: the skip the rescued dispatch is actually handed.
    rescued_skip: u32,
    /// Wall of the rescued dispatch's real pass(es) at the safe area, when measured.
    rescued_ms: Option<f64>,
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
        let (zoom_log2, ref_iter, center, m_jump, throttled_iter) =
            (r.zoom_log2, r.ref_iter, r.center.clone(), r.m_jump, r.throttled_iter);
        self.harness.deviceloss_repro.as_mut().unwrap().done = true;
        self.run_deviceloss_repro(device, queue, zoom_log2, ref_iter, center, m_jump, throttled_iter);
        true
    }

    fn run_deviceloss_repro(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        zoom_log2: f64,
        ref_iter: u32,
        center: Option<(String, String)>,
        m_jump: bool,
        throttled_iter: Option<u32>,
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

        // Reproduce "hit M during a zoom": jump to the minibrot the finder lands on — its OWN deep
        // scale, a far hotter interior than the view M was pressed from, and where the field crashed.
        // Replicates find_minibrot's core (find_nucleus → newton_raphson_target → jump), no toast.
        if m_jump {
            let mag_l2 = self.viewport.log2_magnification();
            let ctr = [self.viewport.center_x.clone(), self.viewport.center_y.clone()];
            let max_period = self
                .viewport
                .recommended_max_iter(self.render_cfg.max_iter)
                .clamp(1_000, 100_000);
            match fractadyne_core::find_nucleus(&ctr, mag_l2, 0, max_period) {
                Some(n) => {
                    let period = n.period;
                    let (jx, jy, target) = self.newton_raphson_target(n.cx, n.cy, period, 0);
                    let t = target.filter(|t| *t > mag_l2).unwrap_or(mag_l2);
                    self.viewport.set_center_log2mag(jx, jy, t);
                    self.viewport.precision =
                        fractadyne_core::precision_for_octaves(t.max(0.0).ceil() as u64);
                    println!(
                        "M-jump    : found a period-{period} minibrot ⇒ jumped to 2^{t:.1} (~{:.2e}x)",
                        2f64.powf(t.min(1020.0))
                    );
                }
                None => {
                    println!("M-jump    : no minibrot found near the centre — measuring the view as given.")
                }
            }
        }

        // Build the reference SYNCHRONOUSLY (a multi-minute bignum build for a large interior orbit;
        // the watchdog will note the pause — that is expected). `current_export_request_for` runs
        // `recompute_worker` inline and carries the orbit into the request.
        println!(
            "view      : centre {} , {}\n\
             zoom      : 2^{:.1} (~{:.2e}x){}\n\
             building reference at iter={ref_iter} (a non-escaping/interior orbit builds to the cap) ...",
            fractadyne_core::to_decimal_string(&self.viewport.center_x),
            fractadyne_core::to_decimal_string(&self.viewport.center_y),
            self.viewport.log2_magnification(),
            2f64.powf(self.viewport.log2_magnification().min(1020.0)),
            if m_jump { "  [after M-jump]" } else if home { "  [canonical Seahorse-998 nucleus]" } else { "  [--center override]" },
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

        // ── ⭐PARITY axis at the FIELD geometry. The 2026-09-11 live loss (and the user's 5e10×
        // crawl) both ran a 1441x1102 target — an ODD width. The odd-HEIGHT penalty is padded away
        // in `make_state_textures` (beta.137), but odd WIDTH was never measured: every sweep and the
        // area axis above force even dims. Two readings per geometry: the NO-OP windows below the
        // skip (the live walk starts at 0, so these are exactly the passes the field ran first) and
        // the REAL windows past it. A geometry whose no-op window is already slow skips its real
        // bracket — ~1,000 slow windows would be minutes for a number the no-op reading already gave.
        println!(
            "\n── WIDTH/HEIGHT PARITY at the field geometry ── (floor window {FLOOR_WINDOW}: no-op windows below the skip vs real windows past it)"
        );
        for &(w, h) in &[(1440u32, 1102u32), (1441, 1102), (1442, 1102), (1440, 1101), (1441, 1101)] {
            let parity = format!(
                "{} width, {} height",
                if w % 2 == 0 { "even" } else { "ODD" },
                if h % 2 == 0 { "even" } else { "ODD" }
            );
            let mut req = base.clone();
            req.width = w;
            req.height = h;
            req.ss = 1;
            // (a) four floor windows from 0 — all below the skip when it is ≥ 1,024, i.e. no-ops.
            req.max_iter = (FLOOR_WINDOW * 4).min(bracket);
            let mut passes = Vec::new();
            let noop = match fractadyne_gpu::render_iter_chunked_timed(device, queue, &req, FLOOR_WINDOW, &mut passes) {
                Ok(_) if !passes.is_empty() => {
                    let worst = passes.iter().map(|p| p.wall_ms).fold(0.0_f64, f64::max);
                    let free = passes.iter().filter(|p| p.end_iter <= skip).count();
                    println!(
                        "  {w:>5}x{h:<5} [{parity}]: {} windows from 0 ({free} below the skip): worst {worst:8.1} ms",
                        passes.len()
                    );
                    Some(worst)
                }
                Ok(_) => {
                    println!("  {w:>5}x{h:<5} [{parity}]: NOT MEASURED (unsupported-scope fallback)");
                    None
                }
                Err(e) => {
                    println!("  {w:>5}x{h:<5} [{parity}]: GPU ERROR — {e}");
                    None
                }
            };
            // (b) the real windows past the skip, only when (a) says the geometry is sane.
            match noop {
                Some(ms) if ms <= 50.0 => {
                    req.max_iter = bracket;
                    let mut passes = Vec::new();
                    if fractadyne_gpu::render_iter_chunked_timed(device, queue, &req, FLOOR_WINDOW, &mut passes).is_ok() {
                        let worst_real = passes
                            .iter()
                            .filter(|p| p.end_iter > skip)
                            .map(|p| p.wall_ms)
                            .fold(0.0_f64, f64::max);
                        let worst_free = passes
                            .iter()
                            .filter(|p| p.end_iter <= skip)
                            .map(|p| p.wall_ms)
                            .fold(0.0_f64, f64::max);
                        println!(
                            "  {w:>5}x{h:<5} [{parity}]: full bracket, {} windows: worst no-op {worst_free:8.1} ms · worst real {worst_real:8.1} ms",
                            passes.len()
                        );
                    }
                }
                Some(ms) => println!(
                    "  {w:>5}x{h:<5} [{parity}]: real bracket SKIPPED — a no-op window already costs {ms:.0} ms (> 50 ms)"
                ),
                None => {}
            }
        }

        // ── ⭐the HOTTEST floor window across the FULL iteration range, at a fixed safe area. The
        // area axis above measured only the SHALLOW start of the orbit — the CHEAP part. On a
        // structured deep view the per-step cost is highest DEEP, where pixels' deltas have grown and
        // rebase every few steps; that is where the field's render actually spent its time. Walk
        // [0, cap) once in floor windows and take the worst, with its position and the trend.
        let (safe_w, safe_h) = (512u32, 384u32);
        let safe_px = (safe_w as f64) * (safe_h as f64);
        let walk_cap = base.orbit_len.min(1_000_000);
        let (mut hot_ms, mut hot_pos) = (0.0_f64, 0u32);
        let mut trend: Vec<(u32, f64)> = Vec::new();
        {
            let mut req = base.clone();
            req.width = safe_w;
            req.height = safe_h;
            req.ss = 1;
            req.max_iter = walk_cap;
            let mut passes = Vec::new();
            if fractadyne_gpu::render_iter_chunked_timed(device, queue, &req, FLOOR_WINDOW, &mut passes)
                .is_ok()
            {
                let step = (passes.len() / 6).max(1);
                for (i, p) in passes.iter().enumerate().filter(|(_, p)| p.end_iter > skip) {
                    if p.wall_ms > hot_ms {
                        hot_ms = p.wall_ms;
                        hot_pos = p.start_iter;
                    }
                    if i % step == 0 {
                        trend.push((p.start_iter, p.wall_ms));
                    }
                }
            }
        }
        println!(
            "\n── hottest floor window across [0,{walk_cap}) at {safe_w}x{safe_h} ── (the DEEP part the area axis missed)"
        );
        for (pos, ms) in &trend {
            println!("  iter {pos:>9}: {ms:6.1} ms");
        }
        let shallow_ms = pts.first().map(|p| p.worst_ms).unwrap_or(0.0);
        println!(
            "  ⇒ hottest {hot_ms:.1} ms at iteration {hot_pos} — {:.1}× the shallow start ({shallow_ms:.1} ms)",
            if shallow_ms > 0.0 { hot_ms / shallow_ms } else { 0.0 }
        );

        // ── window-size cost at the safe area — the actuator the field's loss actually used (its
        // lethal in-flight pass was a 10,236-iteration WINDOW, not the 256 floor).
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

        // ── ⭐THE SA-SKIP INVERSION — crash-1789092955 (2026-09-11), the loss the sweeps above could
        // not reproduce. They price windows PAST the skip, which is exactly the cheap path the live
        // frame did NOT take: the throttle handed it a budget BELOW the reference's SA skip, so
        // `usable_sa_skip` refused the skip and the shader ground from iteration ZERO across the
        // whole budget. Reproduce that decision against the real reference, let the verdict price
        // the from-zero window from the fit (never run), and MEASURE the dispatch `sa_skip_rescue`
        // issues instead.
        let inversion = self.measure_sa_skip_inversion(
            device,
            queue,
            &base,
            throttled_iter.unwrap_or(FIELD_THROTTLED_ITER),
            safe_w,
            safe_h,
        );
        self.report_deviceloss_verdict(&pts, &win_pts, hot_ms, safe_px, target_px, inversion.as_ref());
    }

    /// Reproduce the SA-skip inversion against the built reference: what the live throttle's budget
    /// does to the skip, and what the dispatch becomes with the rescue. Only the RESCUED dispatch is
    /// submitted — it is the small window past the skip, which is the rescue's precondition; the
    /// refused, from-zero one is priced by the verdict from the window fit and never run.
    fn measure_sa_skip_inversion(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        base: &fractadyne_gpu::ExportRequest,
        throttled: u32,
        safe_w: u32,
        safe_h: u32,
    ) -> Option<Inversion> {
        let skip = base.sa_skip;
        let orbit_len = base.orbit_len;
        println!(
            "\n── the SA-skip INVERSION (crash-1789092955) ── throttled budget {throttled} vs SA skip {skip}, orbit_len {orbit_len}"
        );
        let refused_skip = crate::render::usable_sa_skip(skip, throttled);
        if skip <= throttled || refused_skip != 0 {
            println!(
                "  no inversion at this reference: the skip ({skip}) is not above the throttled budget\n\
                 \x20 ({throttled}), so `usable_sa_skip` keeps it applied. A fuller reference (larger REF_ITER on\n\
                 \x20 a non-escaping/parabolic view) or a lower --throttled-iter is needed to reproduce it."
            );
            return None;
        }
        let rescued_iter = crate::render::sa_skip_rescue(throttled, skip, orbit_len);
        let rescued_skip = crate::render::usable_sa_skip(skip, rescued_iter);
        println!(
            "  usable_sa_skip({skip}, {throttled}) = {refused_skip}  ⇒ REFUSED: the live frame seeds the shader at\n\
             \x20 iteration 0 and grinds [0, {throttled}) — the field's 1.86e10-step, 33 s frame.\n\
             \x20 sa_skip_rescue({throttled}, {skip}, {orbit_len}) = {rescued_iter}  ⇒ usable_sa_skip → {rescued_skip}{}",
            if rescued_iter == throttled {
                "  (rescue does NOT apply: the post-skip window is not the cheaper path)"
            } else {
                "  (APPLIED: the shader seeds at the skip)"
            }
        );
        if rescued_iter == throttled {
            return Some(Inversion { throttled, skip, rescued_iter, rescued_skip, rescued_ms: None });
        }
        // Time the rescued dispatch for real. It is the window [skip, orbit_len−1) — provably small,
        // that being the rescue's precondition. Walked in bounded windows rather than one wide pass
        // so that even if the seed were somehow not honoured no single submission could approach
        // the watchdog; windows ending at or below the skip break on entry and cost ~0 ms, so only
        // the pass past the skip carries the real work.
        const RESCUE_WALK_WINDOW: u32 = 4096;
        let mut req = base.clone();
        req.width = safe_w;
        req.height = safe_h;
        req.ss = 1;
        req.max_iter = rescued_iter;
        req.sa_skip = rescued_skip;
        let mut passes = Vec::new();
        let rescued_ms = match fractadyne_gpu::render_iter_chunked_timed(
            device,
            queue,
            &req,
            RESCUE_WALK_WINDOW,
            &mut passes,
        ) {
            Ok(_) if !passes.is_empty() => {
                let real: Vec<&_> = passes.iter().filter(|p| p.end_iter > skip).collect();
                let ms: f64 = real.iter().map(|p| p.wall_ms).sum();
                println!(
                    "  rescued dispatch at {safe_w}x{safe_h}: [{rescued_skip}, {rescued_iter}) = {} iteration(s) past the skip,\n\
                     \x20 in {} real pass(es) of {} walked: {ms:.1} ms MEASURED",
                    rescued_iter.saturating_sub(rescued_skip),
                    real.len(),
                    passes.len(),
                );
                Some(ms)
            }
            Ok(_) => {
                println!("  rescued dispatch: NOT MEASURED (scope fallback)");
                None
            }
            Err(e) => {
                println!("  rescued dispatch: GPU ERROR — {e}");
                None
            }
        };
        Some(Inversion { throttled, skip, rescued_iter, rescued_skip, rescued_ms })
    }

    fn report_deviceloss_verdict(
        &self,
        pts: &[AreaPoint],
        win_pts: &[(f64, f64)],
        hot_ms: f64,
        safe_px: f64,
        target_px: f64,
        inversion: Option<&Inversion>,
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
                "  ⇒ The full-res floor window is ~{worst_case:.0} ms — under the lethal band at this depth\n\
                 \x20  (but this is the SHALLOW start of the orbit; see the deep-hot figure below)."
            );
        }

        // ⭐The DEEP hottest floor window scaled to full res — the real floor cost, since the shallow
        // figure above is the cheap start of the orbit. This is the number that decides whether
        // shedding to the 256 floor can save the view.
        if hot_ms > 0.0 {
            let hot_full = hot_ms * (target_px / safe_px).powf(a);
            println!(
                "  HOT (deep) floor window at full res: ~{hot_full:.0} ms  (from {hot_ms:.1} ms at the safe area × area^{a:.2})"
            );
            if hot_full >= 1000.0 {
                let tiles = (hot_full / BUDGET_TARGET_MS).ceil() as u32;
                println!(
                    "  ⇒ ⭐THE DEEP FLOOR WINDOW IS LETHAL AT FULL RESOLUTION (~{hot_full:.0} ms ≥ the ~1 s watchdog).\n\
                     \x20  Shedding the iteration window to 256 CANNOT save this view — REPRODUCED. AREA is\n\
                     \x20  the only actuator left below the floor: ~{tiles} native-scale tiles keep each\n\
                     \x20  dispatch under the {BUDGET_TARGET_MS:.0} ms budget."
                );
            } else if hot_full >= SAFETY_MS * 0.5 {
                println!(
                    "  ⇒ The deep floor window lands in the lethal BAND (~{hot_full:.0} ms) — marginal; a\n\
                     \x20  slightly hotter location (or a deeper reference) tips it past the watchdog."
                );
            } else {
                println!(
                    "  ⇒ Even the deep floor window is ~{hot_full:.0} ms — under the band. A hotter location\n\
                     \x20  (deeper structure) or a fuller reference (larger REF_ITER) is needed to reproduce."
                );
            }
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
            // ── ⭐THE INVERSION VERDICT: the from-zero window a refused skip forces, priced by this
            // window fit (never run), against the rescued dispatch that was actually measured.
            if let Some(inv) = inversion {
                let unrescued_full = (winter + b * (inv.throttled as f64).ln()).exp() * area_factor;
                println!(
                    "\n  SA-skip inversion (skip {} refused under budget {}): the live frame becomes ONE from-zero\n\
                     \x20 window of {} iterations at full res — priced by this window fit at ~{:.0} ms; the\n\
                     \x20 field's frame measured 32,993 ms. {}",
                    inv.skip,
                    inv.throttled,
                    inv.throttled,
                    unrescued_full,
                    if unrescued_full >= 1000.0 {
                        "⇒ LETHAL — reproduced from the fit."
                    } else {
                        "(under the watchdog by this fit, which prices the structured post-skip region — read as a floor)"
                    }
                );
                match inv.rescued_ms {
                    Some(ms) => {
                        let rescued_full = ms * area_factor;
                        println!(
                            "  with sa_skip_rescue: the dispatch is [{}, {}) — {} iteration(s) past the skip —\n\
                             \x20 MEASURED {ms:.1} ms at the safe area ⇒ ~{rescued_full:.0} ms at full res: {}.",
                            inv.rescued_skip,
                            inv.rescued_iter,
                            inv.rescued_iter.saturating_sub(inv.rescued_skip),
                            if rescued_full < SAFETY_MS * 0.5 {
                                "under the band — the inversion is DEFUSED"
                            } else {
                                "still in the band — the post-skip window is not small here"
                            }
                        );
                    }
                    None if inv.rescued_iter == inv.throttled => println!(
                        "  with sa_skip_rescue: does not apply here (the post-skip window is not the cheaper\n\
                         \x20 path) — this view is the OTHER regime: bound the from-zero first dispatch instead."
                    ),
                    None => println!("  with sa_skip_rescue: rescued dispatch not measured."),
                }
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
