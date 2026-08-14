//! Development profiling harness (`--profile`).
//!
//! Renders a set of benchmark *regions* (named view specs spanning the depth/mode regimes),
//! times the costly stages — bignum **reference orbit**, **series-approximation** setup, and
//! the **GPU iterate / full render** passes — and writes a structured JSON log (with full run
//! context) to `logs/` so bottlenecks are obvious and optimizations can be validated by
//! diffing two runs. Disabled in normal use; opt-in via the CLI, so there's no overhead.
//!
//! The heavy logic lives here (not in `main.rs`) to keep the binary's glue readable.

use crate::{FractadyneApp, FractalKind};
use fractadyne_core::Viewport;
use std::time::Instant;

/// Per-render setup timings recorded (via a `Cell`) by `current_export_request_for`.
#[derive(Clone, Copy, Default)]
pub struct ProfSetup {
    /// Reference-orbit compute (arbitrary precision, CPU), ms.
    pub reference_ms: f64,
    /// Series-approximation coefficient compute + skip selection (CPU), ms.
    pub series_ms: f64,
    /// BLA tree build (`build_bla`, CPU), ms — 0 when BLA is off. This is a *per-frame* cost in
    /// the live view (the tree isn't cached yet), so it matters for the on-by-default decision.
    pub bla_ms: f64,
}

/// One benchmark region: a view to render and time.
#[derive(Clone)]
pub struct ProfRegion {
    pub name: String,
    pub fractal: FractalKind,
    pub cx: String,
    pub cy: String,
    pub zoom_log2: f64,
    pub iter: u32,
    pub size: u32,
    pub ss: u32,
    pub method: u32,
}

const LOG2_10: f64 = std::f64::consts::LOG2_10;

/// Built-in regions spanning the render regimes: direct (home), df32 perturbation
/// (1e4–1e20×), and floatexp + series-approximation (1e30×, with a stripe variant that
/// disables SA) — so a single run exercises every hot path.
pub fn default_regions() -> Vec<ProfRegion> {
    // A known infinitely-deep "seahorse" point: structured across all these scales.
    let (sx, sy) = (
        "-0.7436438870371587047521915061147707",
        "0.131825904205311970493132056385139",
    );
    let m = |name: &str, cx: &str, cy: &str, zoom10: f64, iter: u32, method: u32| ProfRegion {
        name: name.into(),
        fractal: FractalKind::Mandelbrot,
        cx: cx.into(),
        cy: cy.into(),
        zoom_log2: zoom10 * LOG2_10,
        iter,
        size: 512,
        ss: 1,
        method,
    };
    vec![
        m("home", "-0.5", "0.0", 0.0, 512, 0),
        m("seahorse-1e4", sx, sy, 4.0, 1_500, 0),
        m("seahorse-1e6", sx, sy, 6.0, 2_500, 0),
        m("deep-1e12", sx, sy, 12.0, 6_000, 0),
        m("deep-1e20", sx, sy, 20.0, 12_000, 0),
        m("deep-1e30-sa", sx, sy, 30.0, 25_000, 0), // floatexp + series approximation
        m("deep-1e30-stripe", sx, sy, 30.0, 25_000, 1), // floatexp, SA off (aux needs every iter)
        // Deep-interior, dip-carrying orbit (validation corpus 14, ~1.2e148×): the regime of
        // every recent pathology — extended-range samples, heavy rebasing, latency-bound
        // dispatches, and the ~50× export-throughput gap — was previously not profiled at
        // all (design/diagnostics.md D3.4/F16). ~800k iterations; expect seconds per rep.
        m(
            "deep-interior-1e148",
            "-0.3158354656090698908113251908145989842764104941136552011217533774266655202463327904910559501703762081531934176786217990113494418705307973163264218287292234362119",
            "0.6533553743954627788289923830392687875350977003260517837408108019649970888461393846103786781501651324966145060684808980380361143296058258024081840162818693511972",
            148.077,
            800_000,
            0,
        ),
    ]
}

/// Optional regions file (`--regions PATH`, TOML): `[[region]]` tables override the built-ins.
/// Bounded/clamped (dev input, but kept tidy). Returns `None` on any problem (caller falls
/// back to the built-ins, reporting why).
pub fn load_regions(path: &std::path::Path) -> Result<Vec<ProfRegion>, String> {
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if meta.len() > 1_000_000 {
        return Err("regions file too large".into());
    }
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

    #[derive(serde::Deserialize)]
    struct File {
        #[serde(default)]
        region: Vec<Spec>,
    }
    #[derive(serde::Deserialize)]
    struct Spec {
        name: String,
        fractal: Option<String>,
        cx: String,
        cy: String,
        zoom_log2: Option<f64>,
        zoom: Option<f64>, // plain magnification alternative
        iter: Option<u32>,
        size: Option<u32>,
        ss: Option<u32>,
        method: Option<String>,
    }
    let f: File = toml::from_str(&text).map_err(|e| e.to_string())?;
    if f.region.is_empty() {
        return Err("no [[region]] entries".into());
    }
    let out = f
        .region
        .into_iter()
        .take(256)
        .map(|s| {
            let zoom_log2 = s
                .zoom_log2
                .or_else(|| s.zoom.map(|z| z.max(1.0).log2()))
                .unwrap_or(0.0)
                .clamp(0.0, 1.0e6);
            ProfRegion {
                name: s.name.chars().take(48).collect(),
                fractal: s
                    .fractal
                    .as_deref()
                    .and_then(FractalKind::from_name)
                    .unwrap_or(FractalKind::Mandelbrot),
                cx: s.cx.chars().take(2048).collect(),
                cy: s.cy.chars().take(2048).collect(),
                zoom_log2,
                iter: s.iter.unwrap_or(2_000).clamp(64, 200_000),
                size: s.size.unwrap_or(512).clamp(16, 4_096),
                ss: s.ss.unwrap_or(1).clamp(1, 8),
                method: crate::ColorMethod::from_key(s.method.as_deref().unwrap_or("smooth")).to_u32(),
            }
        })
        .collect();
    Ok(out)
}

/// min / median / mean / max of a sample (ms). Sorts in place.
struct Stat {
    min: f64,
    median: f64,
    mean: f64,
    max: f64,
}
fn stat(v: &mut [f64]) -> Stat {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len().max(1);
    let mid = (v.len() / 2).min(v.len().saturating_sub(1));
    Stat {
        min: v.first().copied().unwrap_or(0.0),
        median: v.get(mid).copied().unwrap_or(0.0),
        mean: v.iter().sum::<f64>() / n as f64,
        max: v.last().copied().unwrap_or(0.0),
    }
}

/// JSON string escape (values here are simple, but be safe with `"` and `\`).
fn js(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}
fn stat_json(s: &Stat) -> String {
    format!(
        "{{\"min\":{:.4},\"median\":{:.4},\"mean\":{:.4},\"max\":{:.4}}}",
        s.min, s.median, s.mean, s.max
    )
}

impl FractadyneApp {
    /// Run the profiling pass over `regions` (`reps` measured renders each), write a JSON log
    /// to `out`, print a human summary, and return the log path.
    pub fn run_profile(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        regions: &[ProfRegion],
        reps: u32,
        out: &std::path::Path,
    ) {
        use std::sync::atomic::{AtomicBool, AtomicU32};
        let progress = AtomicU32::new(0);
        let cancel = AtomicBool::new(false);
        let reps = reps.clamp(1, 100);

        let mut json = String::new();
        json.push_str("{\n");
        json.push_str("  \"tool\": \"fractadyne --profile\",\n");
        json.push_str(&format!("  \"version\": {},\n", js(&crate::version_string())));
        json.push_str(&format!("  \"utc\": {},\n", js(&crate::utc_string(now_unix()))));
        let sys = crate::gather_system_info();
        json.push_str(&format!("  \"gpu\": {},\n", js(&self.gpu_name)));
        json.push_str(&format!("  \"cpu\": {},\n", js(&sys.cpu)));
        json.push_str(&format!("  \"os\": {},\n", js(std::env::consts::OS)));
        json.push_str(&format!("  \"series_approx\": {},\n", self.render_cfg.series_approx));
        json.push_str(&format!("  \"use_bla\": {},\n", self.render_cfg.use_bla));
        json.push_str(&format!("  \"reps\": {reps},\n"));
        json.push_str("  \"regions\": [\n");

        println!(
            "Fractadyne profiling — {} regions × {reps} reps (SA {}, BLA {})",
            regions.len(),
            if self.render_cfg.series_approx { "on" } else { "off" },
            if self.render_cfg.use_bla { "on" } else { "off" },
        );
        println!(
            "  {:<20} {:>5} {:>7} {:>8} {:>7} {:>8} {:>9} {:>8} {:>8} {:>9}",
            "region", "mode", "skip", "ref ms", "bla ms", "iter ms", "render ms", "gpu-it", "gpu-col",
            "total ms"
        );
        println!(
            "  (iter/render = CPU wall-clock incl. submit+readback; gpu-it/gpu-col = pure-GPU pass \
             time via timestamps)"
        );

        for (ri, r) in regions.iter().enumerate() {
            // Configure state for this region.
            self.set_fractal(r.fractal);
            self.julia_mode = false;
            self.dual = false;
            self.coloring.color_method = crate::ColorMethod::from_u32(r.method);
            self.render_cfg.auto_iter = false;
            self.render_cfg.max_iter = r.iter;
            self.export.width = r.size;
            self.export.ss = r.ss;
            self.invalidate_refs();

            let mut vp = Viewport::new(r.size as f64, r.size as f64);
            let cx = fractadyne_core::parse_bf(&r.cx).unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(-0.5, 64));
            let cy = fractadyne_core::parse_bf(&r.cy).unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(0.0, 64));
            vp.set_center_log2mag(cx, cy, r.zoom_log2);

            // Build the request once — this records reference/series timings in `self.prof`.
            let req = self.current_export_request_for(&vp, false);
            let setup = self.prof.get();

            // Warm up (shader/pipeline compile, first upload), then measure GPU passes.
            let _ = fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel);
            let (mut iter_ms, mut render_ms) = (Vec::new(), Vec::new());
            let (mut gpu_it, mut gpu_col) = (Vec::new(), Vec::new());
            for _ in 0..reps {
                let t = Instant::now();
                let _ = fractadyne_gpu::render_iter(device, queue, &req);
                iter_ms.push(t.elapsed().as_secs_f64() * 1000.0);
                let t = Instant::now();
                // Capture per-pass GPU timestamps around the full render (iterate + color).
                let (_, ts) = fractadyne_gpu::timing::capture(|| {
                    fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel)
                });
                render_ms.push(t.elapsed().as_secs_f64() * 1000.0);
                if ts.captured {
                    gpu_it.push(ts.iterate_ms);
                    gpu_col.push(ts.color_ms);
                }
            }
            let iter_s = stat(&mut iter_ms);
            let render_s = stat(&mut render_ms);
            let total = setup.reference_ms + setup.series_ms + setup.bla_ms + render_s.median;
            let gpu_it_med = (!gpu_it.is_empty()).then(|| stat(&mut gpu_it).median);
            let gpu_col_med = (!gpu_col.is_empty()).then(|| stat(&mut gpu_col).median);
            let fmt_opt = |v: Option<f64>| v.map_or_else(|| "n/a".to_string(), |x| format!("{x:.2}"));

            println!(
                "  {:<20} {:>5} {:>7} {:>8.2} {:>7.2} {:>8.2} {:>9.2} {:>8} {:>8} {:>9.2}",
                r.name, req.mode, req.sa_skip, setup.reference_ms, setup.bla_ms, iter_s.median,
                render_s.median, fmt_opt(gpu_it_med), fmt_opt(gpu_col_med), total
            );

            json.push_str("    {\n");
            json.push_str(&format!("      \"name\": {},\n", js(&r.name)));
            json.push_str(&format!("      \"fractal\": {},\n", js(r.fractal.name())));
            json.push_str(&format!("      \"center_x\": {},\n", js(&r.cx)));
            json.push_str(&format!("      \"center_y\": {},\n", js(&r.cy)));
            json.push_str(&format!("      \"zoom_log2\": {:.4},\n", r.zoom_log2));
            json.push_str(&format!("      \"zoom_log10\": {:.2},\n", r.zoom_log2 / LOG2_10));
            json.push_str(&format!("      \"size\": [{}, {}],\n", req.width, req.height));
            json.push_str(&format!("      \"ss\": {},\n", req.ss));
            json.push_str(&format!("      \"method\": {},\n", req.color_method));
            json.push_str(&format!("      \"mode\": {},\n", req.mode));
            json.push_str(&format!("      \"eff_iter\": {},\n", req.max_iter));
            json.push_str(&format!("      \"sa_skip\": {},\n", req.sa_skip));
            json.push_str(&format!("      \"orbit_len\": {},\n", req.orbit_len));
            json.push_str(&format!("      \"precision_bits\": {},\n", vp.precision));
            json.push_str("      \"timings_ms\": {\n");
            json.push_str(&format!("        \"reference\": {:.4},\n", setup.reference_ms));
            json.push_str(&format!("        \"series_skip\": {:.4},\n", setup.series_ms));
            json.push_str(&format!("        \"bla_build\": {:.4},\n", setup.bla_ms));
            json.push_str(&format!("        \"gpu_iterate\": {},\n", stat_json(&iter_s)));
            json.push_str(&format!("        \"gpu_render\": {},\n", stat_json(&render_s)));
            let jnum = |v: Option<f64>| v.map_or_else(|| "null".to_string(), |x| format!("{x:.4}"));
            json.push_str(&format!(
                "        \"gpu_iterate_pass_ms\": {},\n",
                jnum(gpu_it_med)
            ));
            json.push_str(&format!(
                "        \"gpu_color_pass_ms\": {}\n",
                jnum(gpu_col_med)
            ));
            json.push_str("      }\n");
            json.push_str("    }");
            json.push_str(if ri + 1 < regions.len() { ",\n" } else { "\n" });
        }
        json.push_str("  ]\n}\n");

        if let Some(dir) = out.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        match std::fs::write(out, &json) {
            Ok(()) => println!("\nprofile log → {}", out.display()),
            Err(e) => eprintln!("\nprofile log write failed: {e}"),
        }
    }
}

/// Per-frame record of the `--divetest` harness.
struct DiveFrame {
    l2: f64,
    frame_ms: f64,
    build_ms: f64,
    gpu_ms: f64,
    motion_res: f64,
    real: bool,
    lag: f64,
    orbit_id: u64,
}

impl crate::FractadyneApp {
    /// `--divetest FILE`: headless live-dive performance harness. Plays REAL-TIME windows of a tour
    /// at a series of depths (default every 100 decades to the tour's max), driving the IDENTICAL
    /// live machinery the GUI runs — `advance_playback_core` (pipeline pacer + camera sampling +
    /// reference lookahead) and `build_params` (reuse-hold / freeze / motion budget) — and, when a
    /// frame really re-iterates, the GPU iterate itself. Frames are vsync-paced (16.7 ms), so
    /// wall-clock dynamics (build races, install cadence, pacing) match the live app; per-window
    /// stats expose exactly what a user would see: fps, hitch counts, real-refresh cadence/cost,
    /// pacer engagement, reference installs.
    pub(crate) fn run_divetest(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        tour: &std::path::Path,
        out: &std::path::Path,
    ) {
        const WARMUP_SECS: f64 = 6.0;
        const MEASURE_SECS: f64 = 12.0;
        const VSYNC: f64 = 1.0 / 60.0;
        let resolution = [1280u32, 720u32];

        // Pin a deterministic live-ish config (mirrors the GUI dive DEFAULTS — notably
        // `min_motion_res` 0.30: a session floor of 1.0 forbids the motion-res controller from
        // trading resolution for smoothness, pinning refresh frames at native cost — set
        // FRACTADYNE_DIVETEST_SESSION_RES=1 to keep the session's floor for user-repro runs).
        self.julia_mode = false;
        self.dual = false;
        self.coloring.color_method = crate::ColorMethod::Smooth;
        self.render_cfg.auto_iter = true;
        if std::env::var("FRACTADYNE_DIVETEST_SESSION_RES").is_err() {
            self.render_cfg.min_motion_res = 0.30;
        }
        self.viewport.set_size(resolution[0] as f64, resolution[1] as f64);

        // Probe the tour's depth range and choose windows every 100 decades.
        let probe = match crate::scripting::parse_tour_file(tour) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("divetest: cannot load {}: {e}", tour.display());
                crate::exit(2);
            }
        };
        let max_l10 = probe.sample(probe.total).logmag / std::f64::consts::LN_10;
        // FRACTADYNE_DIVETEST_WINDOWS="300,700" overrides the default every-100-decades sweep —
        // fast targeted iteration on one band.
        let windows: Vec<f64> = match std::env::var("FRACTADYNE_DIVETEST_WINDOWS") {
            Ok(list) => list.split(',').filter_map(|s| s.trim().parse::<f64>().ok()).collect(),
            Err(_) => (1..=12).map(|k| k as f64 * 100.0).filter(|d| *d < max_l10 - 5.0).collect(),
        };
        // Tour time at which the zoom first reaches `d` decades (bisection; sample() is cheap).
        let time_at = |pb: &crate::scripting::Playback, d: f64| -> Option<f64> {
            let l10 = |t: f64| pb.sample(t).logmag / std::f64::consts::LN_10;
            if l10(pb.total) < d {
                return None;
            }
            let (mut lo, mut hi) = (0.0, pb.total);
            for _ in 0..48 {
                let mid = 0.5 * (lo + hi);
                if l10(mid) >= d {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            Some(hi)
        };

        println!(
            "Fractadyne divetest — {} · {} windows ({WARMUP_SECS:.0}s warmup + {MEASURE_SECS:.0}s measured each) · {}×{} vsync-paced",
            tour.display(),
            windows.len(),
            resolution[0],
            resolution[1]
        );
        println!(
            "  {:>6} {:>6} {:>8} {:>8} {:>8} {:>7} {:>8} {:>8} {:>9} {:>8} {:>8}",
            "depth", "fps", "p50 ms", "p95 ms", "max ms", ">33ms", "real/s", "real p95", "installs", "paced%", "oct/s"
        );

        let t_base = Instant::now();
        let mut json_rows = String::new();
        for (wi, d) in windows.iter().enumerate() {
            let mut pb = match crate::scripting::parse_tour_file(tour) {
                Ok(p) => p,
                Err(_) => break,
            };
            let Some(e_start) = time_at(&pb, *d) else { continue };
            // Cold reference state per window, then seed the tour clock at the window start.
            self.invalidate_refs();
            pb.cur_t = e_start; // seed the clock at the window start (was: shift the origin)
            pb.started = false;
            pb.last_now = None;
            self.playback = Some(pb);

            let mut frames: Vec<DiveFrame> = Vec::with_capacity(2048);
            let w_start = t_base.elapsed().as_secs_f64();
            loop {
                let now = t_base.elapsed().as_secs_f64();
                if now - w_start > WARMUP_SECS + MEASURE_SECS {
                    break;
                }
                match self.advance_playback_core(now) {
                    crate::scripting::PlaybackTick::Playing => {}
                    _ => break, // tour ended inside the window
                }
                let ft = Instant::now();
                // Mirror `draw_central`'s per-frame build exactly (single view, mid-dive).
                let center_bf = [self.viewport.center_x.clone(), self.viewport.center_y.clone()];
                let center = self.viewport.center_f64();
                let span = self.viewport.complex_span_fe();
                let mag = self.viewport.magnification();
                let l2 = self.viewport.log2_magnification();
                let eff_iter = if self.render_cfg.auto_iter {
                    self.viewport.recommended_max_iter(self.render_cfg.max_iter)
                } else {
                    self.render_cfg.max_iter
                };
                let params = self.build_params(
                    center_bf, center, span, mag, l2, self.fractal, false, eff_iter, true, 1,
                    resolution, 0, None,
                );
                let real = params.reproject == 0;
                let build_ms = ft.elapsed().as_secs_f64() * 1000.0;
                let mut gpu_ms = 0.0;
                if real {
                    let req = params_to_request(&params);
                    // Pure-GPU pass time via TIMESTAMP_QUERY — `render_iter`'s wall time includes
                    // a full iterate-texture readback (~15 MB at 720p) the GUI never pays; using
                    // it as the cost signal would overfeed the motion-res controller.
                    let (_, ts) = fractadyne_gpu::timing::capture(|| {
                        fractadyne_gpu::render_iter(device, queue, &req)
                    });
                    gpu_ms = if ts.captured { ts.iterate_ms + ts.color_ms } else {
                        ft.elapsed().as_secs_f64() * 1000.0 - build_ms
                    };
                }
                let frame_ms = build_ms + gpu_ms;
                // Mirror the GUI's frame-interval capture (stamped at the next frame's start
                // there): the interval after a frame = max(its cost, vsync). The motion-res
                // controller reads these, so the harness adapts identically to the live app.
                self.perf.last_dt_ms = frame_ms.max(VSYNC * 1000.0);
                self.perf.frame_ms = frame_ms.max(VSYNC * 1000.0);
                if now - w_start > WARMUP_SECS {
                    frames.push(DiveFrame {
                        l2,
                        frame_ms,
                        build_ms,
                        gpu_ms,
                        motion_res: self.perf.motion_res,
                        real,
                        lag: self.ref_cache[0].last_depth_lag,
                        orbit_id: self.ref_cache[0].orbit_id,
                    });
                }
                // Vsync pacing: the live app can't run faster than the display.
                let spare = VSYNC - ft.elapsed().as_secs_f64();
                if spare > 0.0 {
                    std::thread::sleep(std::time::Duration::from_secs_f64(spare));
                }
            }
            self.playback = None;
            if frames.len() < 30 {
                println!("  1e{:<4} (window too short — skipped)", *d as u64);
                continue;
            }

            // Stats.
            let n = frames.len() as f64;
            let secs = n * 0.0; // recomputed below from times: frames are vsync-paced or longer
            let _ = secs;
            let mut all: Vec<f64> = frames.iter().map(|f| f.frame_ms).collect();
            all.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let pct = |v: &[f64], q: f64| v[((v.len() as f64 * q) as usize).min(v.len() - 1)];
            let (p50, p95, pmax) = (pct(&all, 0.5), pct(&all, 0.95), *all.last().unwrap());
            let wall_secs: f64 = all.iter().map(|ms| (ms / 1000.0).max(VSYNC)).sum();
            let fps = n / wall_secs;
            let hitches = all.iter().filter(|ms| **ms > 33.0).count();
            let mut reals: Vec<f64> =
                frames.iter().filter(|f| f.real).map(|f| f.frame_ms).collect();
            reals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let real_per_s = reals.len() as f64 / wall_secs;
            let real_p95 = if reals.is_empty() { 0.0 } else { pct(&reals, 0.95) };
            let installs = frames.last().unwrap().orbit_id.saturating_sub(frames[0].orbit_id);
            let paced =
                frames.iter().filter(|f| f.lag > crate::PACE_LAG_LO).count() as f64 / n * 100.0;
            let oct_s = (frames.last().unwrap().l2 - frames[0].l2) / wall_secs;
            // Split the real frames' cost (CPU build vs pure-GPU) + the controller's res range —
            // pins WHERE an over-budget real frame spends its time.
            let mut rb: Vec<f64> = frames.iter().filter(|f| f.real).map(|f| f.build_ms).collect();
            let mut rg: Vec<f64> = frames.iter().filter(|f| f.real).map(|f| f.gpu_ms).collect();
            rb.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            rg.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let (rb95, rg95) = if rb.is_empty() { (0.0, 0.0) } else { (pct(&rb, 0.95), pct(&rg, 0.95)) };
            let res_lo = frames.iter().map(|f| f.motion_res).fold(1.0f64, f64::min);
            let res_hi = frames.iter().map(|f| f.motion_res).fold(0.0f64, f64::max);
            println!(
                "  1e{:<4} {:>6.1} {:>8.1} {:>8.1} {:>8.1} {:>7} {:>8.1} {:>8.1} {:>9} {:>7.0}% {:>8.2}  build95 {:>6.1} gpu95 {:>6.1} res {:.2}-{:.2}",
                *d as u64, fps, p50, p95, pmax, hitches, real_per_s, real_p95, installs, paced, oct_s,
                rb95, rg95, res_lo, res_hi
            );
            json_rows.push_str(&format!(
                "{}{{\"depth\":{d},\"fps\":{fps:.2},\"p50\":{p50:.2},\"p95\":{p95:.2},\"max\":{pmax:.2},\
                 \"hitches\":{hitches},\"real_per_s\":{real_per_s:.2},\"real_p95\":{real_p95:.2},\
                 \"installs\":{installs},\"paced_pct\":{paced:.1},\"oct_s\":{oct_s:.3}}}",
                if wi == 0 { "" } else { ",\n" }
            ));
        }
        let json = format!(
            "{{\n\"tool\": \"fractadyne --divetest\",\n\"version\": {},\n\"tour\": {},\n\"windows\": [\n{json_rows}\n]\n}}\n",
            js(&crate::version_string()),
            js(&tour.display().to_string()),
        );
        if let Some(dir) = out.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        match std::fs::write(out, &json) {
            Ok(()) => println!("\ndivetest log → {}", out.display()),
            Err(e) => eprintln!("\ndivetest log write failed: {e}"),
        }
    }
}

impl crate::FractadyneApp {
    /// `--resizetest`: headless window-resize regression harness. Settles a DEEP view, then
    /// scripts a drag-resize (stepped size changes at ~60 Hz) through the REAL per-frame logic
    /// (`build_params` with the same resize-counts-as-interacting stamping `draw_central` does)
    /// and asserts the aspect invariant on every tick: a frame either
    ///   (a) REPROJECTS the held frame — the color pass's per-axis `fit = out_res/tex` is a 1:1
    ///       pixel mapping (aspect-preserving by construction), or
    ///   (b) RE-ITERATES at the current size (content matches the rect).
    /// Either way what the app paints cannot be stretched; a tick that re-iterates at a STALE
    /// size, or reprojects with mismatched inputs, is the app-side stretch bug. If this harness
    /// passes and a live window still stretches during drag-resize, the stretch is
    /// compositor-level (slow presents), not paint-level. Exit 0 = invariant held on all ticks.
    pub(crate) fn run_resizetest(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
    ) {
        const SETTLE_DELAY: f64 = 0.35; // mirror main.rs SETTLE_DELAY
        // A deep filament view (the dive-to-view target region at 1e300×) — the regime the
        // squashed-resize report came from.
        let cx = "-0.10109636384562216102578544573862256546380544282625348387693117766078084074047058427482121981051677903340453190855674119397154614426091188235570390768030161575841814343545448230681066640294193782674118916078195264489953969953344668233931266990098175571504252065388669918613443339616006105627753851442596095607035682216432799561330369267147603777972840547126801980704959564343122613074621896737872910571815679801197486243277885212232140346141476363055768794030853505141349183029015625069901969710301398305392417144156069597140451840313064629757293752694923598744652623932038999304619097152290211115114321808028789403122382122426681233709818005501104114164866955494431100070201222185701964043942380903139582696661410506981033531993920115166766865499029937517718480859741678429471778875835091458778409768052331281855418374275568747857427817878588975428498776440958365571143980553929580426851853970861581967408482954948615588204167423489425979292655194076761074235209884803548623974441682647098190126510057828334902994426890932017548072978965178763527302460666281401438540176035039880523180757953729595833877855592110412384316138244258042800106144270831186332997088681779330774172810346897306169609186015417079821641600400593193981268337275751132474252918216";
        let cy = "0.9562865108091415007710960577299774358098333365105291700343143215005246590657167325269784107873398072043444724926469284366752406567465722656200815719741551313831228054884443810571677870203738395055477279309548311953190385170024877917670113105649462054553485859930615004427397447762045980085973915977603864276670744042253335354323566859404712327457083787857148248615873869675060108354066122688803113441095405220822267412663824851779098634770481811864806258738023969258382884081571925537012152280038588430181998328078474272633699174145410614739462589128789157270309137280445851229295437246369133142664831678997110648342958496799411796053782879854029239135856497881774132969669652699204486924289172505386561146364971863878183988732262227332465841765012882266031595005109538147888898264288529633218462512748520811096416738441262363354058945001574839594474384589481122468123965611462097003237015619675431384515236428681192402632704494401393363553995382534130865149497598845865720492821177842454782140474274743600236820990552140248357737361972320431809526352413494013760129709944358620070346268288974825610996815010703236471629927357824532578002161368350336523576700466236404066703410118983273506666673552161007021682783983310680516466618258915183846702688575";
        let cx = fractadyne_core::parse_bf(cx).unwrap();
        let cy = fractadyne_core::parse_bf(cy).unwrap();

        self.julia_mode = false;
        self.dual = false;
        self.coloring.color_method = crate::ColorMethod::Smooth;
        self.render_cfg.auto_iter = true;

        let (w0, h0) = (1280u32, 720u32);
        self.viewport.set_size(w0 as f64, h0 as f64);
        self.viewport
            .set_center_log2mag(cx, cy, 300.0 * std::f64::consts::LOG2_10);

        // One frame at t, at size (w,h), with the resize-stamping draw_central does. Returns
        // (reproject, rendered_res) where rendered_res is the res the frame ITERATED at (None if
        // it reprojected). `settle_t` drives `interacting` exactly like the GUI.
        let t0 = Instant::now();
        let mut last_real_res: Option<[u32; 2]> = None;
        let frame = |app: &mut crate::FractadyneApp,
                     w: u32,
                     h: u32,
                     last_real_res: &mut Option<[u32; 2]>|
         -> (bool, [u32; 2]) {
            let now = t0.elapsed().as_secs_f64();
            let (nw, nh) = (w as f64, h as f64);
            let resized = (nw - app.viewport.width_px).abs() > 0.5
                || (nh - app.viewport.height_px).abs() > 0.5;
            if resized {
                app.pointer.settle_t[0] = now;
            }
            app.viewport.set_size(nw, nh);
            let interacting = now - app.pointer.settle_t[0] < SETTLE_DELAY;
            let center_bf = [app.viewport.center_x.clone(), app.viewport.center_y.clone()];
            let center = app.viewport.center_f64();
            let span = app.viewport.complex_span_fe();
            let mag = app.viewport.magnification();
            let l2 = app.viewport.log2_magnification();
            let eff_iter = if app.render_cfg.auto_iter {
                app.viewport.recommended_max_iter(app.render_cfg.max_iter)
            } else {
                app.render_cfg.max_iter
            };
            let params = app.build_params(
                center_bf, center, span, mag, l2, app.fractal, false, eff_iter, interacting, 1,
                [w, h], 0, None,
            );
            let reproject = params.reproject == 1;
            if !reproject {
                let req = params_to_request(&params);
                let _ = fractadyne_gpu::render_iter(device, queue, &req);
                *last_real_res = Some(params.resolution);
            }
            (reproject, params.resolution)
        };

        // Settle the view: run frames at the initial size until a real (non-reproject) frame has
        // rendered and the reference is installed (cold start is async).
        println!("resizetest — settling a 1e300 filament view at {w0}×{h0}…");
        let settle_deadline = Instant::now() + std::time::Duration::from_secs(30);
        loop {
            let (repro, _) = frame(self, w0, h0, &mut last_real_res);
            if !repro && self.ref_cache[0].ref_pt.is_some() {
                break;
            }
            if Instant::now() > settle_deadline {
                eprintln!("resizetest: view failed to settle (no real frame in 30 s)");
                crate::exit(2);
            }
            std::thread::sleep(std::time::Duration::from_millis(16));
        }

        // Scripted drag-resize: shrink the height 720→400 in 20 px steps (~60 Hz ticks), then
        // widen back. Every tick must satisfy the invariant.
        println!("resizetest — scripted drag-resize (aspect invariant per tick):");
        let mut steps: Vec<(u32, u32)> = Vec::new();
        let mut h = h0 as i32;
        while h > 400 {
            h -= 20;
            steps.push((w0, h as u32));
        }
        for _ in 0..6 {
            steps.push((w0, 400)); // pause at the extreme (still within SETTLE_DELAY)
        }
        let mut w = w0 as i32;
        while w > 800 {
            w -= 24;
            steps.push((w as u32, 400));
        }
        let mut violations = 0u32;
        let mut reprojected = 0u32;
        let mut real = 0u32;
        for (i, (w, h)) in steps.iter().enumerate() {
            let (repro, res) = frame(self, *w, *h, &mut last_real_res);
            if repro {
                reprojected += 1;
                // Reproject paints the held texture with a 1:1 pixel fit — aspect-correct for ANY
                // held size. Nothing further to check.
            } else {
                real += 1;
                // A real frame must have iterated at THIS tick's size (any uniform res_scale of it
                // keeps the aspect; a mismatched aspect = painting stale-size content stretched).
                // Tolerance: the res_scale shrink truncates each axis to an integer, so tiny budget
                // frames are off by up to a pixel per axis — compare against the worst rounding
                // error at the rendered size, not a fixed ratio.
                let want = *w as f64 / *h as f64;
                let got = res[0] as f64 / res[1].max(1) as f64;
                let tol = want * (1.0 / res[0].max(1) as f64 + 1.0 / res[1].max(1) as f64);
                if (want - got).abs() > tol {
                    violations += 1;
                    println!(
                        "  tick {i}: STRETCH — rect {w}×{h} (aspect {want:.3}) but iterated {}×{} (aspect {got:.3})",
                        res[0], res[1]
                    );
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
        println!(
            "  {} ticks: {reprojected} reprojected (aspect-fit), {real} re-iterated, {violations} violations",
            steps.len()
        );
        if violations == 0 {
            println!(
                "resizetest PASS — every painted resize frame is aspect-correct app-side. If the\n\
                 live window still stretches during a drag, the stretch is compositor-level\n\
                 (presents lagging the OS resize stream), not in what the app paints."
            );
            crate::exit(0);
        }
        crate::exit(1);
    }
}

/// Current Unix time (seconds); 0 if the clock is before the epoch.
fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Convert a live `MandelbrotParams` into an `ExportRequest` that renders the SAME view reusing
/// its already-computed reference orbit (no recompute) — lets the frame-timing harness render each
/// frame exactly as the live view would, without paying the reference cost twice.
pub(crate) fn params_to_request(p: &fractadyne_gpu::MandelbrotParams) -> fractadyne_gpu::ExportRequest {
    fractadyne_gpu::ExportRequest {
        width: p.resolution[0].max(1),
        height: p.resolution[1].max(1),
        work_budget: None,
        ss: p.ss.max(1),
        span_mantissa: p.span_mantissa,
        center: p.center,
        ref_offset: p.ref_offset,
        delta_exp: p.delta_exp,
        sa_skip: p.sa_skip,
        // Profiling measures cost, not colour: the mapping never affects iterate time, so this
        // path stays on the classic linear map regardless of the session's setting.
        norm_mode: 0,
        norm_lo: 0.0,
        glitch_on: 0,
        vignette: p.vignette,
        sa_a: p.sa_a,
        sa_a_exp: p.sa_a_exp,
        sa_b: p.sa_b,
        sa_b_exp: p.sa_b_exp,
        sa_c: p.sa_c,
        sa_c_exp: p.sa_c_exp,
        julia_c: p.julia_c,
        orbit: p.orbit.clone(),
        orbit_len: p.orbit_len,
        bla: p.bla.clone(),
        bla_on: p.bla_on,
        max_iter: p.max_iter,
        mode: p.mode,
        formula: p.formula,
        julia: p.julia,
        cycle: p.cycle,
        offset: p.offset,
        stop_count: p.stop_count,
        stops: p.stops,
        light: p.light,
        light_angle: p.light_angle,
        light_height: p.light_height,
        de_on: p.de_on,
        de_strength: p.de_strength,
        de_width: p.de_width,
        de_phase: p.de_phase,
        color_method: p.color_method,
        stripe_freq: p.stripe_freq,
        trap_type: p.trap_type,
        aa_filter: p.aa_filter,
        interior_col: p.interior_col,
    }
}

/// One recorded frame of a `--frametest` dive.
struct FrameRec {
    log10: f64,
    build_ms: f64, // CPU: reference-cache management (recompute when needed) — the stutter source
    gpu_ms: f64,   // GPU: render the frame reusing the cached reference
    settle: bool,  // true = a just-changed view (interacting=false → recompute may fire)
}

impl FractadyneApp {
    /// Frame-timing / stutter harness (`--frametest`). Simulates a stepped deep-zoom dive on the
    /// **live** path — `build_params` (which owns the reference cache + recompute) then a GPU render
    /// of each frame — and records per-frame CPU/GPU time. Each "step" deepens the zoom and holds
    /// for a few frames: the first frame of a step is where the reference recompute (the stall)
    /// lands, the rest run from cache. Reports interframe stats + a stutter count, so an
    /// optimization (e.g. async recompute) can be validated automatically. Writes a JSON log.
    #[allow(clippy::too_many_arguments)] // REFACTOR-PLAN Phase 2/4: group the frametest config into a struct
    pub fn run_frametest(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        steps: u32,
        hold: u32,
        target_log10: f64,
        size: u32,
        out: &std::path::Path,
    ) {
        // Dive point: `--center X Y` when given (measure a REAL deep-dive line), else the seahorse
        // (structure all the way down, but only 34 digits — precision-noise past ~1e34×).
        let (def_x, def_y) = (
            "-0.7436438870371587047521915061147707",
            "0.131825904205311970493132056385139",
        );
        let (sx, sy) = self
            .frametest_center
            .clone()
            .unwrap_or_else(|| (def_x.to_string(), def_y.to_string()));
        let cx = fractadyne_core::parse_bf(&sx)
            .unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(-0.5, 64));
        let cy = fractadyne_core::parse_bf(&sy)
            .unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(0.0, 64));
        self.set_fractal(FractalKind::Mandelbrot);
        self.julia_mode = false;
        self.dual = false;
        self.coloring.color_method = crate::ColorMethod::Smooth;
        self.render_cfg.auto_iter = true;
        self.invalidate_refs();
        let size = size.clamp(64, 4096);

        let steps = steps.clamp(1, 2000);
        let hold = hold.clamp(1, 60);
        let mut recs: Vec<FrameRec> = Vec::with_capacity((steps * hold) as usize);

        println!(
            "Fractadyne frame test — {steps} steps × {hold} hold, {size}px, dive → 1e{target_log10:.0}× (BLA {})",
            if self.render_cfg.use_bla { "on" } else { "off" }
        );

        for step in 0..steps {
            let log2mag = (target_log10 * (step + 1) as f64 / steps as f64) * LOG2_10;
            for h in 0..hold {
                // A step's first frame is a just-settled view (recompute may fire); the rest reuse
                // the cache — mirroring "zoom a bit, then pause".
                let settle = h == 0;
                self.viewport
                    .set_center_log2mag(cx.clone(), cy.clone(), log2mag);
                let center_bf = [self.viewport.center_x.clone(), self.viewport.center_y.clone()];
                let center = self.viewport.center_f64();
                let span = self.viewport.complex_span_fe();
                let mag = self.viewport.magnification();
                let l2 = self.viewport.log2_magnification();
                let eff_iter = self.viewport.recommended_max_iter(self.render_cfg.max_iter);
                let t = Instant::now();
                let params = self.build_params(
                    center_bf, center, span, mag, l2, self.fractal, false, eff_iter, false, self.render_cfg.aa,
                    [size, size], 0, None,
                );
                let build_ms = t.elapsed().as_secs_f64() * 1000.0;
                let req = params_to_request(&params);
                let t = Instant::now();
                let _ = fractadyne_gpu::render_iter(device, queue, &req);
                let gpu_ms = t.elapsed().as_secs_f64() * 1000.0;
                recs.push(FrameRec { log10: l2 / LOG2_10, build_ms, gpu_ms, settle });
            }
        }

        // The reference recompute (build_ms) is the interframe stall the async work targets — it
        // blocks the frame on the main thread. Count a "recompute stall" as build_ms > 16 ms (a
        // hitch that alone drops a frame below 60 fps). The GPU render is a separate, steady cost
        // (unchanged by async), reported for context.
        const STALL_MS: f64 = 16.0;
        let mut totals: Vec<f64> = recs.iter().map(|r| r.build_ms + r.gpu_ms).collect();
        let mut builds: Vec<f64> = recs.iter().map(|r| r.build_ms).collect();
        let mut gpus: Vec<f64> = recs.iter().map(|r| r.gpu_ms).collect();
        let ts = stat(&mut totals);
        let bs = stat(&mut builds);
        let gs = stat(&mut gpus);
        let p = |v: &mut [f64], q: f64| {
            v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            v[((v.len() as f64 * q) as usize).min(v.len().saturating_sub(1))]
        };
        let b95 = p(&mut builds, 0.95);
        let stalls = recs.iter().filter(|r| r.build_ms > STALL_MS).count();
        let worst = bs.max;

        println!(
            "  frames {}  build ms (CPU recompute — the stall): median {:.1} p95 {:.1} max {:.1}",
            recs.len(), bs.median, b95, bs.max
        );
        println!(
            "  recompute stalls (build >{STALL_MS:.0}ms): {stalls}   |   gpu render median {:.1} ms (context, steady)",
            gs.median
        );
        let _ = ts;

        // JSON.
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str("  \"tool\": \"fractadyne --frametest\",\n");
        json.push_str(&format!("  \"version\": {},\n", js(&crate::version_string())));
        json.push_str(&format!("  \"utc\": {},\n", js(&crate::utc_string(now_unix()))));
        json.push_str(&format!("  \"gpu\": {},\n", js(&self.gpu_name)));
        json.push_str(&format!("  \"use_bla\": {},\n", self.render_cfg.use_bla));
        json.push_str(&format!("  \"steps\": {steps}, \"hold\": {hold}, \"size\": {size},\n"));
        json.push_str(&format!("  \"stall_ms\": {STALL_MS},\n"));
        json.push_str(&format!("  \"recompute_stalls\": {stalls},\n"));
        json.push_str(&format!(
            "  \"build_ms\": {{\"median\":{:.3},\"p95\":{:.3},\"max\":{:.3}}},\n",
            bs.median, b95, bs.max
        ));
        json.push_str(&format!(
            "  \"gpu_ms\": {{\"median\":{:.3},\"max\":{:.3}}},\n",
            gs.median, gs.max
        ));
        json.push_str(&format!("  \"worst_build_ms\": {worst:.3},\n"));
        json.push_str("  \"frames\": [\n");
        for (i, r) in recs.iter().enumerate() {
            json.push_str(&format!(
                "    {{\"log10\":{:.2},\"settle\":{},\"build_ms\":{:.3},\"gpu_ms\":{:.3}}}",
                r.log10, r.settle, r.build_ms, r.gpu_ms
            ));
            json.push_str(if i + 1 < recs.len() { ",\n" } else { "\n" });
        }
        json.push_str("  ]\n}\n");
        if let Some(dir) = out.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        match std::fs::write(out, &json) {
            Ok(()) => println!("frametest log → {}", out.display()),
            Err(e) => eprintln!("frametest log write failed: {e}"),
        }
    }
}
