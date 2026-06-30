//! Scripting & benchmark: TOML keyframe camera tours (`Tools -> Play script`) and the
//! built-in benchmark tour, plus the playback engine that glides center+zoom along the
//! timeline and samples FPS/CPU/RAM. `Playback`/`Bench` are pub(crate) (held as app state).

use crate::{process_memory, version_string, FractadyneApp, FractalKind};
use serde::Deserialize;

/// On-disk script format (TOML). A keyframe with no `center_x`/`center_y` inherits
/// the previous keyframe's center (handy for pure zoom-in tours).
#[derive(Deserialize, Default)]
struct ScriptFile {
    #[serde(default)]
    name: String,
    #[serde(default, rename = "loop")]
    loop_: bool,
    #[serde(default)]
    keyframe: Vec<KeyframeFile>,
}

#[derive(Deserialize, Clone)]
struct KeyframeFile {
    /// Seconds to glide here from the previous keyframe.
    #[serde(default)]
    secs: f64,
    #[serde(default)]
    center_x: Option<String>,
    #[serde(default)]
    center_y: Option<String>,
    #[serde(default = "one_f64")]
    mag: f64,
    #[serde(default)]
    fractal: Option<String>,
    #[serde(default)]
    julia: Option<bool>,
}

fn one_f64() -> f64 {
    1.0
}

/// A resolved keyframe (parsed center + cumulative time on the timeline).
struct Kf {
    at: f64,
    cx: fractadyne_core::BigFloat,
    cy: fractadyne_core::BigFloat,
    logmag: f64,
    fractal: FractalKind,
    julia: bool,
}

/// Aggregates sampled while a benchmark tour plays.
pub(crate) struct Bench {
    frames: u64,
    sum_frame_ms: f64,
    sum_cpu_ms: f64,
    min_fps: f64,
    max_fps: f64,
    peak_ram: u64,
    sum_ram: u64,
    warmup_left: u32,
}

impl Bench {
    fn new() -> Self {
        Bench {
            frames: 0,
            sum_frame_ms: 0.0,
            sum_cpu_ms: 0.0,
            min_fps: f64::INFINITY,
            max_fps: 0.0,
            peak_ram: 0,
            sum_ram: 0,
            warmup_left: 12,
        }
    }
}

/// An active camera tour (and optional benchmark sampling).
pub(crate) struct Playback {
    pub(crate) name: String,
    kfs: Vec<Kf>,
    pub(crate) total: f64,
    pub(crate) t0: Option<f64>,
    loop_: bool,
    pub(crate) bench: Option<Bench>,
}

/// Resolve a parsed script file into a playable tour (parses centers, accumulates
/// keyframe times, fills inherited centers). Returns `None` if it has no keyframes.
fn resolve_script(sf: ScriptFile, bench: Option<Bench>) -> Option<Playback> {
    if sf.keyframe.is_empty() {
        return None;
    }
    let mut kfs = Vec::with_capacity(sf.keyframe.len());
    let mut at = 0.0;
    let mut last = ("-0.5".to_string(), "0.0".to_string());
    for k in &sf.keyframe {
        at += k.secs.max(0.0);
        if let Some(x) = &k.center_x {
            last.0 = x.clone();
        }
        if let Some(y) = &k.center_y {
            last.1 = y.clone();
        }
        let cx = fractadyne_core::parse_bf(&last.0)?;
        let cy = fractadyne_core::parse_bf(&last.1)?;
        let fractal = k
            .fractal
            .as_deref()
            .and_then(FractalKind::from_name)
            .unwrap_or(FractalKind::Mandelbrot);
        kfs.push(Kf {
            at,
            cx,
            cy,
            logmag: k.mag.max(1.0).ln(),
            fractal,
            julia: k.julia.unwrap_or(false),
        });
    }
    let total = kfs.last().map(|k| k.at).unwrap_or(0.0);
    Some(Playback {
        name: if sf.name.is_empty() {
            "Script".to_string()
        } else {
            sf.name
        },
        kfs,
        total,
        t0: None,
        loop_: sf.loop_,
        bench,
    })
}

/// Built-in benchmark tour: a steady zoom into a Seahorse-Valley spiral over a fixed
/// timeline, so successive builds / machines render the same work.
fn benchmark_playback() -> Playback {
    let cx = "-0.743643887037158704752191506114774";
    let cy = "0.131825904205311970493132056385139";
    let mags = [1.0, 1.0e3, 1.0e6, 1.0e9, 1.0e12];
    let mut keyframe = Vec::new();
    for (i, &m) in mags.iter().enumerate() {
        keyframe.push(KeyframeFile {
            secs: if i == 0 { 0.0 } else { 4.0 },
            center_x: Some(cx.to_string()),
            center_y: Some(cy.to_string()),
            mag: m,
            fractal: Some("Mandelbrot".to_string()),
            julia: Some(false),
        });
    }
    let sf = ScriptFile {
        name: "Built-in benchmark".to_string(),
        loop_: false,
        keyframe,
    };
    resolve_script(sf, Some(Bench::new())).expect("valid benchmark script")
}

impl FractadyneApp {
    /// Start the built-in benchmark tour.
    pub(crate) fn start_benchmark(&mut self) {
        self.dual = false; // benchmark measures the single-view pipeline
        self.playback = Some(benchmark_playback());
    }

    /// Load a camera-tour script (TOML) via a file dialog and start playing it.
    pub(crate) fn load_script(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Fractadyne script (TOML)", &["toml"])
            .pick_file()
        else {
            return;
        };
        match std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| toml::from_str::<ScriptFile>(&t).ok())
            .and_then(|sf| resolve_script(sf, None))
        {
            Some(pb) => self.playback = Some(pb),
            None => self.bench_report = Some(format!("Could not load script:\n{}", path.display())),
        }
    }

    /// Advance the active camera tour by one frame; drives the view and, for a
    /// benchmark, samples performance. Returns true while still playing.
    pub(crate) fn advance_playback(&mut self, ctx: &egui::Context) -> bool {
        let Some(mut pb) = self.playback.take() else {
            return false;
        };
        let now = ctx.input(|i| i.time);
        let t0 = *pb.t0.get_or_insert(now);
        let mut elapsed = now - t0;
        if pb.loop_ && pb.total > 0.0 && elapsed >= pb.total {
            pb.t0 = Some(now);
            elapsed = 0.0;
        }
        let finished = !pb.loop_ && elapsed >= pb.total;
        let e = elapsed.clamp(0.0, pb.total);

        // Locate the active segment and interpolate (eased) center + log-magnification.
        let n = pb.kfs.len();
        let mut i = n - 1;
        for j in 0..n.saturating_sub(1) {
            if e <= pb.kfs[j + 1].at {
                i = j;
                break;
            }
        }
        let (cx, cy, logmag, fractal, julia) = if i + 1 < n {
            let a = &pb.kfs[i];
            let b = &pb.kfs[i + 1];
            let seg = (b.at - a.at).max(1.0e-9);
            let u = ((e - a.at) / seg).clamp(0.0, 1.0);
            let ease = u * u * (3.0 - 2.0 * u);
            let lm = a.logmag + (b.logmag - a.logmag) * ease;
            let p = fractadyne_core::precision_for_magnification(lm.exp());
            (
                fractadyne_core::lerp_bf(&a.cx, &b.cx, ease, p),
                fractadyne_core::lerp_bf(&a.cy, &b.cy, ease, p),
                lm,
                a.fractal,
                a.julia,
            )
        } else {
            let a = &pb.kfs[i];
            (a.cx.clone(), a.cy.clone(), a.logmag, a.fractal, a.julia)
        };
        if fractal != self.fractal || julia != self.julia_mode {
            self.fractal = fractal;
            self.julia_mode = julia && fractal.supports_julia();
            self.invalidate_refs();
        }
        self.viewport.set_center_mag(cx, cy, logmag.exp());
        self.settle_t = now; // glide → cheap (interacting) render path

        // Benchmark sampling (skip warm-up frames).
        if let Some(b) = pb.bench.as_mut() {
            if b.warmup_left > 0 {
                b.warmup_left -= 1;
            } else if self.perf.frame_ms > 0.0 {
                b.frames += 1;
                b.sum_frame_ms += self.perf.frame_ms;
                b.sum_cpu_ms += self.perf.cpu_ms;
                let fps = 1000.0 / self.perf.frame_ms;
                b.min_fps = b.min_fps.min(fps);
                b.max_fps = b.max_fps.max(fps);
                let (ws, peak) = process_memory();
                b.peak_ram = b.peak_ram.max(peak).max(ws);
                b.sum_ram += ws;
            }
        }

        if finished {
            if let Some(b) = pb.bench.take() {
                self.bench_report = Some(self.format_bench(&pb, &b));
                self.bench_open = true;
            }
            return false; // pb dropped → playback stops at the final keyframe
        }
        self.playback = Some(pb);
        true
    }

    /// Build a human-readable benchmark report.
    pub(crate) fn format_bench(&self, pb: &Playback, b: &Bench) -> String {
        let f = b.frames.max(1) as f64;
        let avg_frame = b.sum_frame_ms / f;
        let avg_fps = if avg_frame > 0.0 { 1000.0 / avg_frame } else { 0.0 };
        let avg_cpu = b.sum_cpu_ms / f;
        let avg_gpu = (b.sum_frame_ms - b.sum_cpu_ms).max(0.0) / f;
        let avg_ram = b.sum_ram / b.frames.max(1);
        let mb = |bytes: u64| bytes as f64 / (1024.0 * 1024.0);
        let si = &self.sysinfo;
        let cache = if si.l3_kb > 0 {
            format!("L2 {} KB · L3 {} MB", si.l2_kb, si.l3_kb / 1024)
        } else if si.l2_kb > 0 {
            format!("L2 {} KB", si.l2_kb)
        } else {
            "—".to_string()
        };
        let vram = if si.vram_mb > 0 {
            format!("{} MB", si.vram_mb)
        } else {
            "—".to_string()
        };
        format!(
            "Fractadyne benchmark — {tour}\n\
             version    v{ver}\n\
             cpu        {cpu}\n\
             cores      {phys} physical / {logi} logical\n\
             cache      {cache}\n\
             gpu        {gpu}\n\
             vram       {vram}\n\
             frames     {frames}  over {dur:.0}s\n\
             ----------------------------------------\n\
             avg FPS    {afps:8.1}\n\
             min FPS    {minf:8.1}\n\
             max FPS    {maxf:8.1}\n\
             avg frame  {aframe:8.2} ms\n\
             avg CPU    {acpu:8.2} ms\n\
             avg GPU    {agpu:8.2} ms   (frame − cpu)\n\
             avg RAM    {aram:8.1} MB\n\
             peak RAM   {pram:8.1} MB\n\
             ----------------------------------------\n\
             score      {score:8.0}   (avg FPS × 100)",
            tour = pb.name,
            ver = version_string(),
            cpu = if si.cpu.is_empty() { "—" } else { &si.cpu },
            phys = si.physical,
            logi = si.logical,
            cache = cache,
            gpu = self.gpu_name,
            vram = vram,
            frames = b.frames,
            dur = pb.total,
            afps = avg_fps,
            minf = if b.min_fps.is_finite() { b.min_fps } else { 0.0 },
            maxf = b.max_fps,
            aframe = avg_frame,
            acpu = avg_cpu,
            agpu = avg_gpu,
            aram = mb(avg_ram),
            pram = mb(b.peak_ram),
            score = avg_fps * 100.0,
        )
    }
}
