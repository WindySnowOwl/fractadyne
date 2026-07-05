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
                method: crate::method_from_str(s.method.as_deref().unwrap_or("smooth")),
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
        json.push_str(&format!("  \"series_approx\": {},\n", self.series_approx));
        json.push_str(&format!("  \"use_bla\": {},\n", self.use_bla));
        json.push_str(&format!("  \"reps\": {reps},\n"));
        json.push_str("  \"regions\": [\n");

        println!(
            "Fractadyne profiling — {} regions × {reps} reps (SA {}, BLA {})",
            regions.len(),
            if self.series_approx { "on" } else { "off" },
            if self.use_bla { "on" } else { "off" },
        );
        println!(
            "  {:<20} {:>5} {:>8} {:>9} {:>8} {:>9} {:>10} {:>10}",
            "region", "mode", "skip", "ref ms", "bla ms", "iter ms", "render ms", "total ms"
        );

        for (ri, r) in regions.iter().enumerate() {
            // Configure state for this region.
            self.set_fractal(r.fractal);
            self.julia_mode = false;
            self.dual = false;
            self.color_method = r.method;
            self.auto_iter = false;
            self.max_iter = r.iter;
            self.export_width = r.size;
            self.export_ss = r.ss;
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
            for _ in 0..reps {
                let t = Instant::now();
                let _ = fractadyne_gpu::render_iter(device, queue, &req);
                iter_ms.push(t.elapsed().as_secs_f64() * 1000.0);
                let t = Instant::now();
                let _ = fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel);
                render_ms.push(t.elapsed().as_secs_f64() * 1000.0);
            }
            let iter_s = stat(&mut iter_ms);
            let render_s = stat(&mut render_ms);
            let total = setup.reference_ms + setup.series_ms + setup.bla_ms + render_s.median;

            println!(
                "  {:<20} {:>5} {:>8} {:>9.2} {:>8.2} {:>9.2} {:>10.2} {:>10.2}",
                r.name, req.mode, req.sa_skip, setup.reference_ms, setup.bla_ms, iter_s.median, render_s.median, total
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
            json.push_str(&format!("        \"gpu_render\": {}\n", stat_json(&render_s)));
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
        ss: p.ss.max(1),
        span_mantissa: p.span_mantissa,
        center: p.center,
        ref_offset: p.ref_offset,
        delta_exp: p.delta_exp,
        sa_skip: p.sa_skip,
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
        // A point with structure all the way down (seahorse valley).
        let cx = fractadyne_core::parse_bf("-0.7436438870371587047521915061147707")
            .unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(-0.5, 64));
        let cy = fractadyne_core::parse_bf("0.131825904205311970493132056385139")
            .unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(0.0, 64));
        self.set_fractal(FractalKind::Mandelbrot);
        self.julia_mode = false;
        self.dual = false;
        self.color_method = 0;
        self.auto_iter = true;
        self.invalidate_refs();
        let size = size.clamp(64, 4096);

        let steps = steps.clamp(1, 2000);
        let hold = hold.clamp(1, 60);
        let mut recs: Vec<FrameRec> = Vec::with_capacity((steps * hold) as usize);

        println!(
            "Fractadyne frame test — {steps} steps × {hold} hold, {size}px, dive → 1e{target_log10:.0}× (BLA {})",
            if self.use_bla { "on" } else { "off" }
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
                let eff_iter = self.viewport.recommended_max_iter(self.max_iter);
                let t = Instant::now();
                let params = self.build_params(
                    center_bf, center, span, mag, l2, self.fractal, false, eff_iter, false, self.aa,
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
        json.push_str(&format!("  \"use_bla\": {},\n", self.use_bla));
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
