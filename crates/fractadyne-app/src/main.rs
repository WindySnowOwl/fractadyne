//! Fractadyne — native fractal explorer.
//!
//! M0 shell: an egui window (menu bar + canvas + status bar) with the Mandelbrot
//! drawn on the GPU. Pan with left-drag, zoom with the wheel (cursor-centered).
//! Box-zoom and the full panel set arrive in later milestones (see TODO.md).

use eframe::egui;
use fractadyne_core::Viewport;
use fractadyne_gpu::{add_mandelbrot, install_renderer, MandelbrotParams};
use serde::Deserialize;
use std::time::Instant;

/// Lightweight per-frame performance/diagnostic tracking, shown in an overlay.
/// On by default for now; toggle via the View menu or the `--no-perf` CLI flag.
struct Perf {
    enabled: bool,
    last_frame: Option<Instant>,
    /// Smoothed wall-clock interval between frames (ms) → FPS.
    frame_ms: f64,
    /// Smoothed CPU time spent in `update()` (ms).
    cpu_ms: f64,
    /// Duration of the most recent reference-orbit recompute (ms).
    recompute_ms: f64,
    /// Total reference recomputes since launch.
    recompute_total: u64,
    /// Recomputes counted in the current 1 s window, and the last computed rate.
    rate_count: u32,
    rate_t0: Option<Instant>,
    recompute_per_s: f32,
    /// Diagnostics from the last view-0 (Mandelbrot) build.
    last_mode: u32,
    last_eff_iter: u32,
    last_precision: usize,
    last_orbit_len: u32,
}

impl Default for Perf {
    fn default() -> Self {
        Self {
            enabled: true,
            last_frame: None,
            frame_ms: 0.0,
            cpu_ms: 0.0,
            recompute_ms: 0.0,
            recompute_total: 0,
            rate_count: 0,
            rate_t0: None,
            recompute_per_s: 0.0,
            last_mode: 0,
            last_eff_iter: 0,
            last_precision: 0,
            last_orbit_len: 0,
        }
    }
}

fn ema(prev: f64, sample: f64) -> f64 {
    if prev <= 0.0 {
        sample
    } else {
        prev * 0.9 + sample * 0.1
    }
}

/// HSV (all 0..1) → RGB (0..1). For synthesizing vivid random palette stops.
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let h6 = (h.fract() * 6.0).clamp(0.0, 6.0);
    let i = h6.floor() as i32;
    let f = h6 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    match i % 6 {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

/// Randomized, continuously-morphing palette. Holds two gradient keyframes (`from`,
/// `to`) of fixed 6 stops and blends between them; on reaching `to` it becomes the
/// new `from` and a fresh random `to` is synthesized. Stops are HSV-random with the
/// endpoints equal so the gradient still cycles seamlessly.
struct RandomPalette {
    rng: u32,
    from: [[f32; 4]; fractadyne_color::MAX_STOPS],
    to: [[f32; 4]; fractadyne_color::MAX_STOPS],
    t: f32,
}

const RAND_STOPS: usize = 6;

impl RandomPalette {
    fn new(seed: u32) -> Self {
        let mut s = RandomPalette {
            rng: seed | 1,
            from: [[0.0; 4]; fractadyne_color::MAX_STOPS],
            to: [[0.0; 4]; fractadyne_color::MAX_STOPS],
            t: 0.0,
        };
        s.from = s.gen_stops();
        s.to = s.gen_stops();
        s
    }
    fn next_u32(&mut self) -> u32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x
    }
    fn next_f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }
    /// Generate a *harmonious* gradient: a single base hue with a gentle analogous
    /// excursion (so colors flow through neighbouring hues, not a clashing rainbow), a
    /// smooth dark→bright→dark brightness arc for contrast, and matching dark endpoints
    /// for seamless cycling. The hue/brightness follow a `sin(πt)` arc so the first and
    /// last stops coincide (no seam) while the middle carries the color and light.
    fn gen_stops(&mut self) -> [[f32; 4]; fractadyne_color::MAX_STOPS] {
        let h0 = self.next_f32(); // base hue
        let hue_span = 0.05 + 0.16 * self.next_f32(); // gentle analogous shift (~18°–75°)
        let dir = if self.next_f32() < 0.5 { -1.0 } else { 1.0 };
        let sat = 0.55 + 0.30 * self.next_f32(); // moderate, constant saturation
        let v_lo = 0.16 + 0.12 * self.next_f32(); // dim (not black) endpoints
        let v_hi = 0.85 + 0.15 * self.next_f32(); // bright middle
        let mut out = [[0.0f32; 4]; fractadyne_color::MAX_STOPS];
        for i in 0..RAND_STOPS {
            let t = i as f32 / (RAND_STOPS - 1) as f32; // 0..1
            let arc = (std::f32::consts::PI * t).sin(); // 0 at ends → 1 at middle
            let h = h0 + dir * hue_span * arc;
            let v = v_lo + (v_hi - v_lo) * arc;
            let c = hsv_to_rgb(h, sat, v);
            out[i] = [c[0], c[1], c[2], t];
        }
        for i in RAND_STOPS..fractadyne_color::MAX_STOPS {
            out[i] = out[RAND_STOPS - 1];
        }
        out
    }
    /// Advance the blend; `speed` is gradient-changes per second.
    fn advance(&mut self, dt: f32, speed: f32) {
        self.t += dt * speed;
        while self.t >= 1.0 {
            self.t -= 1.0;
            self.from = self.to;
            self.to = self.gen_stops();
        }
    }
    /// Snap to a brand-new pair of gradients.
    fn reshuffle(&mut self) {
        self.from = self.gen_stops();
        self.to = self.gen_stops();
        self.t = 0.0;
    }
    /// Current blended stops for GPU upload.
    fn current(&self) -> ([[f32; 4]; fractadyne_color::MAX_STOPS], u32) {
        let mut out = self.from;
        for i in 0..RAND_STOPS {
            for k in 0..3 {
                out[i][k] = self.from[i][k] + (self.to[i][k] - self.from[i][k]) * self.t;
            }
        }
        for i in RAND_STOPS..fractadyne_color::MAX_STOPS {
            out[i] = out[RAND_STOPS - 1];
        }
        (out, RAND_STOPS as u32)
    }
}

/// Shared base-2 exponent for the floatexp perturbation δ mantissas: `floor(log2(span))`,
/// so the per-pixel offset mantissa stays O(1) in df32 at any zoom depth.
fn delta_exponent(span_x: f64) -> i32 {
    if span_x > 0.0 && span_x.is_finite() {
        span_x.log2().floor() as i32
    } else {
        0
    }
}

/// Linear RGBA interpolation between two colors (`t` in 0..1).
fn lerp_color(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    egui::Color32::from_rgba_unmultiplied(
        mix(a.r(), b.r()),
        mix(a.g(), b.g()),
        mix(a.b(), b.b()),
        mix(a.a(), b.a()),
    )
}

/// A compact toggle button drawn as two side-by-side rectangles (the dual-view
/// icon). Painted directly so it renders identically regardless of font glyphs.
fn dual_toggle_button(ui: &mut egui::Ui, selected: bool) -> egui::Response {
    let size = egui::vec2(30.0, ui.spacing().interact_size.y);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let v = ui.style().interact_selectable(&resp, selected);
    let p = ui.painter();
    p.rect_filled(rect, egui::CornerRadius::same(2), v.bg_fill);
    let inner = rect.shrink2(egui::vec2(8.0, 4.0));
    let gap = 3.0;
    let half_w = ((inner.width() - gap) * 0.5).max(1.0);
    let left = egui::Rect::from_min_size(inner.min, egui::vec2(half_w, inner.height()));
    let right = egui::Rect::from_min_size(
        egui::pos2(inner.min.x + half_w + gap, inner.min.y),
        egui::vec2(half_w, inner.height()),
    );
    let stroke = egui::Stroke::new(1.4, v.fg_stroke.color);
    p.rect_stroke(left, egui::CornerRadius::same(1), stroke, egui::StrokeKind::Inside);
    p.rect_stroke(right, egui::CornerRadius::same(1), stroke, egui::StrokeKind::Inside);
    resp
}

/// Continuous-zoom tuning.
const ZOOM_RATE: f64 = 0.462; // ln(2)/1.5 ≈ ~2× magnification per 1.5 s at full speed
const EASE_TAU: f64 = 0.15; // ease-in/out time constant (seconds)
/// Keep anti-aliasing off for this long after the last interaction, so rapid zoom
/// steps don't each trigger a full-AA render (which felt laggy).
const SETTLE_DELAY: f64 = 0.18;
/// Max GPU work per render (texels × iterations) before the OS GPU watchdog (TDR)
/// risks a device-lost crash. Supersampling auto-reduces to stay under this.
const WORK_BUDGET: u64 = 60_000_000_000;

/// Max iterates drawn by the interactive orbit overlay (shallow f64 path).
const ORBIT_MAX: usize = 512;
/// Deep (bignum) orbit cap — large enough to run past where nearby points' orbits
/// diverge (≈ the reference orbit's escape length) so the overlay responds to the
/// cursor; bounded for cost. Cached so it only recomputes when the cursor/view moves.
const ORBIT_MAX_DEEP: u32 = 8192;

/// Cache key for the interactive orbit (recompute only when these change).
#[derive(Clone, PartialEq)]
struct OrbitKey {
    px: f64,
    py: f64,
    cx: f64,
    cy: f64,
    upp: f64,
    julia: bool,
    formula: u32,
    jcx: f64,
    jcy: f64,
}
struct OrbitCacheEntry {
    key: OrbitKey,
    pts: Vec<(f64, f64)>,
}

/// Magnification at/above which perturbation switches from the fast df32 δ to the
/// floatexp δ. df32 stays clean to ~1e30×; cross over before then with margin.
const PERT_FE_THRESHOLD: f64 = 1.0e28;

/// Semantic version (from Cargo) + an auto-incrementing per-build counter (build.rs).
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const BUILD_SEQ: &str = env!("FRACT_BUILD");

/// Display string, e.g. `0.1.0 (build 42)`.
fn version_string() -> String {
    format!("{APP_VERSION} (build {BUILD_SEQ})")
}

/// "Zoom home" animation pacing: seconds per octave-ish of zoom-out, clamped so a
/// shallow view still glides and an extreme one doesn't take forever.
const HOME_SECONDS_PER_LOGMAG: f64 = 0.45;
const HOME_MIN_SECONDS: f64 = 1.5;
const HOME_MAX_SECONDS: f64 = 9.0;

/// An in-progress smooth zoom-out to the home view (started by the Home button).
struct HomeAnim {
    start_time: f64,
    duration: f64,
    /// Mandelbrot/main view: center when the animation began + its ln(magnification).
    m_start_center: (fractadyne_core::BigFloat, fractadyne_core::BigFloat),
    m_start_logmag: f64,
    /// Dual Julia view (only used when `dual`).
    j_start_center: (fractadyne_core::BigFloat, fractadyne_core::BigFloat),
    j_start_logmag: f64,
    dual: bool,
}

fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([640.0, 400.0])
            .with_title(format!("Fractadyne v{}", version_string())),
        ..Default::default()
    };

    eframe::run_native(
        "Fractadyne",
        native_options,
        Box::new(|cc| Ok(Box::new(FractadyneApp::new(cc)))),
    )
}

/// Group an integer string with commas every 3 digits (handles a leading `-`).
fn commas(s: &str) -> String {
    let neg = s.starts_with('-');
    let digits = s.trim_start_matches('-');
    let len = digits.len();
    let mut out = String::with_capacity(len + len / 3 + 1);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if neg {
        format!("-{out}")
    } else {
        out
    }
}

/// Magnification with comma-grouped integer part + 2 decimals, e.g. `1,805,359.12`.
fn fmt_zoom(mag: f64) -> String {
    if mag > 1.0e12 {
        // Deep zoom: scientific notation, 12 significant digits (a 30-digit integer is
        // unreadable). `{:.11e}` → e.g. `3.38050027227e15`.
        format!("{mag:.11e}")
    } else if mag >= 1000.0 {
        // Large integer magnification: comma-grouped, no decimals (the `.00` is clutter).
        commas(&format!("{mag:.0}"))
    } else {
        // Small zoom: up to 2 decimals, trailing zeros trimmed (e.g. `1.5`, `256`, `2.37`).
        let s = format!("{mag:.2}");
        let s = s.trim_end_matches('0').trim_end_matches('.');
        s.to_string()
    }
}

/// Coordinate with fractional digits grouped in 5s by spaces, e.g.
/// `-0.64939 71837 00000`.
fn fmt_coord(v: f64) -> String {
    let sign = if v.is_sign_negative() { "-" } else { "+" };
    let s = format!("{:.15}", v.abs());
    match s.split_once('.') {
        Some((int_part, frac)) => {
            let mut g = String::with_capacity(frac.len() + frac.len() / 5);
            for (i, c) in frac.chars().enumerate() {
                if i > 0 && i % 5 == 0 {
                    g.push(' ');
                }
                g.push(c);
            }
            format!("{sign}{int_part}.{g}")
        }
        None => format!("{sign}{s}"),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExportFormat {
    Png,
    Exr,
}

/// How the dual view is exported.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DualExport {
    /// Both panels stitched into one image (Mandelbrot | Julia).
    SideBySide,
    /// Two files (`…_map` and `…_julia`).
    Separate,
    /// Just the main/left panel.
    ActiveOnly,
}

/// Current + peak process working-set bytes (for the benchmark RAM metric).
#[cfg(windows)]
fn process_memory() -> (u64, u64) {
    #[repr(C)]
    struct Pmc {
        cb: u32,
        page_fault_count: u32,
        peak_working_set: usize,
        working_set: usize,
        quota_peak_paged: usize,
        quota_paged: usize,
        quota_peak_nonpaged: usize,
        quota_nonpaged: usize,
        pagefile: usize,
        peak_pagefile: usize,
    }
    extern "system" {
        fn GetCurrentProcess() -> isize;
        fn K32GetProcessMemoryInfo(h: isize, c: *mut Pmc, cb: u32) -> i32;
    }
    unsafe {
        let mut pmc: Pmc = std::mem::zeroed();
        pmc.cb = std::mem::size_of::<Pmc>() as u32;
        if K32GetProcessMemoryInfo(GetCurrentProcess(), &mut pmc, pmc.cb) != 0 {
            (pmc.working_set as u64, pmc.peak_working_set as u64)
        } else {
            (0, 0)
        }
    }
}
#[cfg(not(windows))]
fn process_memory() -> (u64, u64) {
    (0, 0)
}

/// Host system facts shown in benchmark reports (gathered once at startup).
struct SysInfo {
    cpu: String,
    logical: usize,
    physical: usize,
    l2_kb: u64,
    l3_kb: u64,
    vram_mb: u64,
}

fn gather_system_info() -> SysInfo {
    let logical = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(0);
    let (physical, l2_kb, l3_kb) = cpu_topology();
    SysInfo {
        cpu: cpu_brand(),
        logical,
        physical: if physical == 0 { logical } else { physical },
        l2_kb,
        l3_kb,
        vram_mb: gpu_vram_bytes() / (1024 * 1024),
    }
}

/// CPU brand string via the CPUID extended leaves (no dependencies).
#[cfg(target_arch = "x86_64")]
fn cpu_brand() -> String {
    use std::arch::x86_64::__cpuid;
    unsafe {
        if __cpuid(0x8000_0000).eax < 0x8000_0004 {
            return String::new();
        }
        let mut bytes = Vec::with_capacity(48);
        for leaf in [0x8000_0002u32, 0x8000_0003, 0x8000_0004] {
            let r = __cpuid(leaf);
            for v in [r.eax, r.ebx, r.ecx, r.edx] {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
        }
        String::from_utf8_lossy(&bytes)
            .trim_matches(|c: char| c == '\0' || c.is_whitespace())
            .to_string()
    }
}
#[cfg(not(target_arch = "x86_64"))]
fn cpu_brand() -> String {
    String::new()
}

/// Physical core count + total L2/L3 cache (KB) via GetLogicalProcessorInformation.
#[cfg(windows)]
fn cpu_topology() -> (usize, u64, u64) {
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Slpi {
        processor_mask: usize,
        relationship: u32,
        _pad: u32,
        info: [u8; 16], // union; for RelationCache it holds CACHE_DESCRIPTOR
    }
    #[repr(C)]
    struct CacheDescriptor {
        level: u8,
        assoc: u8,
        line_size: u16,
        size: u32,
        ctype: u32,
    }
    extern "system" {
        fn GetLogicalProcessorInformation(buf: *mut Slpi, len: *mut u32) -> i32;
    }
    unsafe {
        let mut len: u32 = 0;
        GetLogicalProcessorInformation(std::ptr::null_mut(), &mut len);
        let sz = std::mem::size_of::<Slpi>() as u32;
        if len == 0 || sz == 0 {
            return (0, 0, 0);
        }
        let count = (len / sz) as usize;
        let mut buf = vec![
            Slpi {
                processor_mask: 0,
                relationship: 0,
                _pad: 0,
                info: [0u8; 16],
            };
            count
        ];
        if GetLogicalProcessorInformation(buf.as_mut_ptr(), &mut len) == 0 {
            return (0, 0, 0);
        }
        let (mut physical, mut l2, mut l3) = (0usize, 0u64, 0u64);
        for e in &buf {
            match e.relationship {
                0 => physical += 1, // RelationProcessorCore
                2 => {
                    // RelationCache
                    let cd = &*(e.info.as_ptr() as *const CacheDescriptor);
                    match cd.level {
                        2 => l2 += cd.size as u64,
                        3 => l3 += cd.size as u64,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        (physical, l2 / 1024, l3 / 1024)
    }
}
#[cfg(not(windows))]
fn cpu_topology() -> (usize, u64, u64) {
    (0, 0, 0)
}

/// Dedicated VRAM (bytes) read from the display-adapter registry keys (largest of
/// the first few adapters). Best-effort; returns 0 if unavailable.
#[cfg(windows)]
fn gpu_vram_bytes() -> u64 {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "advapi32")]
    extern "system" {
        fn RegGetValueW(
            hkey: isize,
            subkey: *const u16,
            value: *const u16,
            flags: u32,
            ptype: *mut u32,
            pdata: *mut core::ffi::c_void,
            pcb: *mut u32,
        ) -> i32;
    }
    let hklm: isize = 0x8000_0002u32 as i32 as isize; // HKEY_LOCAL_MACHINE
    let wide = |s: &str| -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    };
    let value = wide("HardwareInformation.qwMemorySize");
    let mut best = 0u64;
    for i in 0..8 {
        let key = wide(&format!(
            "SYSTEM\\CurrentControlSet\\Control\\Class\\{{4d36e968-e325-11ce-bfc1-08002be10318}}\\{i:04}"
        ));
        let mut data = [0u8; 8];
        let mut cb = 8u32;
        let rc = unsafe {
            RegGetValueW(
                hklm,
                key.as_ptr(),
                value.as_ptr(),
                0x0000_ffff, // RRF_RT_ANY
                std::ptr::null_mut(),
                data.as_mut_ptr() as *mut core::ffi::c_void,
                &mut cb,
            )
        };
        if rc == 0 {
            best = best.max(u64::from_le_bytes(data));
        }
    }
    best
}
#[cfg(not(windows))]
fn gpu_vram_bytes() -> u64 {
    0
}

// ---- Scripting: keyframe camera tours (also drives the benchmark) ----

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
struct Bench {
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
struct Playback {
    name: String,
    kfs: Vec<Kf>,
    total: f64,
    t0: Option<f64>,
    loop_: bool,
    bench: Option<Bench>,
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

/// Palette animation mode (continuously shifts the color offset).
#[derive(Clone, Copy, PartialEq, Eq)]
enum PaletteAnim {
    Off,
    Forward,
    Reverse,
    PingPong,
    Random,
}

impl PaletteAnim {
    const ALL: [PaletteAnim; 5] = [
        PaletteAnim::Off,
        PaletteAnim::Forward,
        PaletteAnim::Reverse,
        PaletteAnim::PingPong,
        PaletteAnim::Random,
    ];
    fn name(self) -> &'static str {
        match self {
            PaletteAnim::Off => "Off",
            PaletteAnim::Forward => "Forward",
            PaletteAnim::Reverse => "Reverse",
            PaletteAnim::PingPong => "Ping-pong",
            PaletteAnim::Random => "Random gradients",
        }
    }
    fn key(self) -> &'static str {
        match self {
            PaletteAnim::Off => "off",
            PaletteAnim::Forward => "forward",
            PaletteAnim::Reverse => "reverse",
            PaletteAnim::PingPong => "pingpong",
            PaletteAnim::Random => "random",
        }
    }
    fn from_key(s: &str) -> PaletteAnim {
        match s {
            "forward" => PaletteAnim::Forward,
            "reverse" => PaletteAnim::Reverse,
            "pingpong" => PaletteAnim::PingPong,
            "random" => PaletteAnim::Random,
            _ => PaletteAnim::Off,
        }
    }
}

/// A render+write job handed to the background export worker.
enum ExportJob {
    Single(fractadyne_gpu::ExportRequest),
    SideBySide(fractadyne_gpu::ExportRequest, fractadyne_gpu::ExportRequest),
    Separate(fractadyne_gpu::ExportRequest, fractadyne_gpu::ExportRequest),
}

/// Stitch two rendered images horizontally (left | right) into one RGBA buffer.
fn stitch_side_by_side(
    a: &fractadyne_gpu::ExportResult,
    b: &fractadyne_gpu::ExportResult,
) -> (u32, u32, Vec<f32>) {
    let h = a.height.max(b.height);
    let w = a.width + b.width;
    let mut px = vec![0.0f32; (w as usize) * (h as usize) * 4];
    for i in (3..px.len()).step_by(4) {
        px[i] = 1.0; // opaque black background (for any height mismatch)
    }
    let blit = |dst: &mut [f32], src: &[f32], sw: u32, sh: u32, x0: u32| {
        let row = sw as usize * 4;
        for y in 0..sh as usize {
            let s = y * row;
            let d = (y * w as usize + x0 as usize) * 4;
            dst[d..d + row].copy_from_slice(&src[s..s + row]);
        }
    };
    blit(&mut px, &a.pixels, a.width, a.height, 0);
    blit(&mut px, &b.pixels, b.width, b.height, a.width);
    (w, h, px)
}

/// Derive `…_map` / `…_julia` sibling paths for a "separate" dual export.
fn separate_paths(path: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("export");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
    let dir = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    (
        dir.join(format!("{stem}_map.{ext}")),
        dir.join(format!("{stem}_julia.{ext}")),
    )
}

/// Read one `key=value` line out of an embedded metadata blob.
fn meta_get(meta: &str, key: &str) -> String {
    meta.lines()
        .find_map(|l| {
            l.split_once('=')
                .filter(|(k, _)| k.trim() == key)
                .map(|(_, v)| v.trim().to_string())
        })
        .unwrap_or_default()
}

/// Per-view cached perturbation reference orbit (arbitrary precision).
struct RefCache {
    ref_pt: Option<[fractadyne_core::BigFloat; 2]>,
    orbit: std::sync::Arc<Vec<[f32; 4]>>,
    orbit_len: u32,
    /// Bumped whenever `orbit` changes (tells the GPU to re-upload).
    orbit_id: u64,
    /// Precision / iteration count the cached orbit was computed at.
    orbit_prec: usize,
    orbit_iter: u32,
    /// When the orbit was last recomputed (throttles refresh during interaction).
    last_recompute: Option<Instant>,
}

impl Default for RefCache {
    fn default() -> Self {
        Self {
            ref_pt: None,
            orbit: std::sync::Arc::new(Vec::new()),
            orbit_len: 0,
            orbit_id: 0,
            orbit_prec: 0,
            orbit_iter: 0,
            last_recompute: None,
        }
    }
}

/// A saved view (bookmark). `meta` is the same key=value view-metadata blob used by
/// exports, restorable via `load_view_metadata`.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Bookmark {
    name: String,
    meta: String,
}

/// TOML wrapper for the bookmarks file (`[[bookmark]]` array).
#[derive(Default, serde::Serialize, serde::Deserialize)]
struct BookmarkFile {
    #[serde(default)]
    bookmark: Vec<Bookmark>,
}

/// One exported image discovered by the gallery browser.
struct GalleryEntry {
    path: std::path::PathBuf,
    meta: String,
    fractal: String,
    zoom: String,
    saved: String,
    notes: String,
    app_version: String,
    saved_unix: u64,
    thumb: Option<egui::TextureHandle>,
    thumb_tried: bool,
}

/// Formula reference shown in the info panel.
struct FractalInfo {
    formula: &'static str,
    about: &'static str,
    reference: &'static str,
}

/// Escape-time fractal families. `formula_id` must match the shader's `fs_iterate`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum FractalKind {
    Mandelbrot,
    Multibrot3,
    Multibrot4,
    Multibrot5,
    Tricorn,
    BurningShip,
    Celtic,
    Buffalo,
    Phoenix,
    Newton,
}

impl FractalKind {
    const ALL: [FractalKind; 10] = [
        FractalKind::Mandelbrot,
        FractalKind::Multibrot3,
        FractalKind::Multibrot4,
        FractalKind::Multibrot5,
        FractalKind::Tricorn,
        FractalKind::BurningShip,
        FractalKind::Celtic,
        FractalKind::Buffalo,
        FractalKind::Phoenix,
        FractalKind::Newton,
    ];

    fn name(self) -> &'static str {
        match self {
            FractalKind::Mandelbrot => "Mandelbrot",
            FractalKind::Multibrot3 => "Multibrot 3",
            FractalKind::Multibrot4 => "Multibrot 4",
            FractalKind::Multibrot5 => "Multibrot 5",
            FractalKind::Tricorn => "Tricorn",
            FractalKind::BurningShip => "Burning Ship",
            FractalKind::Celtic => "Celtic",
            FractalKind::Buffalo => "Buffalo",
            FractalKind::Phoenix => "Phoenix",
            FractalKind::Newton => "Newton",
        }
    }

    fn from_name(name: &str) -> Option<FractalKind> {
        FractalKind::ALL.into_iter().find(|k| k.name() == name)
    }

    fn formula_id(self) -> u32 {
        match self {
            FractalKind::Mandelbrot => 0,
            FractalKind::Multibrot3 => 1,
            FractalKind::Multibrot4 => 2,
            FractalKind::Multibrot5 => 3,
            FractalKind::Tricorn => 4,
            FractalKind::BurningShip => 5,
            FractalKind::Celtic => 6,
            FractalKind::Buffalo => 7,
            FractalKind::Phoenix => 8,
            FractalKind::Newton => 9,
        }
    }

    /// Default view center (x, y) for this fractal.
    fn default_center(self) -> (f64, f64) {
        match self {
            FractalKind::Mandelbrot | FractalKind::Celtic => (-0.5, 0.0),
            FractalKind::BurningShip | FractalKind::Buffalo => (-0.5, -0.5),
            _ => (0.0, 0.0),
        }
    }

    /// Whether a Julia variant is meaningful (Newton has no parameter `c`).
    fn supports_julia(self) -> bool {
        !matches!(self, FractalKind::Newton)
    }

    /// Whether deep zoom (CPU reference + GPU perturbation) is implemented. The
    /// analytic polynomial families qualify; abs-based and Newton stay on the direct
    /// path (clean to ~1e6×).
    fn supports_perturbation(self) -> bool {
        matches!(
            self,
            FractalKind::Mandelbrot
                | FractalKind::Multibrot3
                | FractalKind::Multibrot4
                | FractalKind::Multibrot5
                | FractalKind::Tricorn
        )
    }

    fn info(self) -> FractalInfo {
        match self {
            FractalKind::Mandelbrot => FractalInfo {
                formula: "z -> z^2 + c    (z0 = 0)",
                about: "The canonical escape-time fractal: the set of c for which the \
                        orbit of 0 stays bounded. Its boundary is infinitely intricate.",
                reference: "https://en.wikipedia.org/wiki/Mandelbrot_set",
            },
            FractalKind::Multibrot3 => FractalInfo {
                formula: "z -> z^3 + c",
                about: "A Multibrot set - the Mandelbrot construction at a higher power. \
                        Power d gives (d-1)-fold rotational symmetry.",
                reference: "https://en.wikipedia.org/wiki/Multibrot_set",
            },
            FractalKind::Multibrot4 => FractalInfo {
                formula: "z -> z^4 + c",
                about: "Multibrot at power 4: threefold symmetry, broad bulbs.",
                reference: "https://en.wikipedia.org/wiki/Multibrot_set",
            },
            FractalKind::Multibrot5 => FractalInfo {
                formula: "z -> z^5 + c",
                about: "Multibrot at power 5: fourfold symmetry.",
                reference: "https://en.wikipedia.org/wiki/Multibrot_set",
            },
            FractalKind::Tricorn => FractalInfo {
                formula: "z -> conj(z)^2 + c",
                about: "The Tricorn (Mandelbar): conjugates z each step. This \
                        anti-holomorphic map yields a three-cornered shape.",
                reference: "https://en.wikipedia.org/wiki/Tricorn_(mathematics)",
            },
            FractalKind::BurningShip => FractalInfo {
                formula: "z -> (|Re z| + i|Im z|)^2 + c",
                about: "Absolute values of z's parts are taken before squaring; the \
                        result resembles a ship in flames.",
                reference: "https://en.wikipedia.org/wiki/Burning_Ship_fractal",
            },
            FractalKind::Celtic => FractalInfo {
                formula: "Re -> |Re(z^2)| + cx;  Im -> Im(z^2) + cy",
                about: "A Burning-Ship relative that takes the absolute value of only \
                        the real part of z^2, producing celtic-knot / heart motifs.",
                reference: "https://paulbourke.net/fractals/burnship/",
            },
            FractalKind::Buffalo => FractalInfo {
                formula: "Re -> |Re(z^2)| + cx;  Im -> |Im(z^2)| + cy",
                about: "An abs-variant taking absolute values of both components of z^2.",
                reference: "https://paulbourke.net/fractals/burnship/",
            },
            FractalKind::Phoenix => FractalInfo {
                formula: "z' = z^2 + c + p*z_prev    (p = -0.5)",
                about: "The Phoenix uses the previous iterate too, giving flame-like \
                        filaments. Try its Julia form via Julia mode.",
                reference: "https://paulbourke.net/fractals/phoenix/",
            },
            FractalKind::Newton => FractalInfo {
                formula: "z -> z - (z^3 - 1)/(3 z^2)",
                about: "Newton's root-finding iteration for z^3 = 1, colored by how fast \
                        each point converges. A convergence (not escape) fractal.",
                reference: "https://en.wikipedia.org/wiki/Newton_fractal",
            },
        }
    }
}

struct FractadyneApp {
    viewport: Viewport,
    /// Which fractal is being rendered (single-view mode).
    fractal: FractalKind,
    /// Julia constant `c` (complex). In dual view it's driven by the Mandelbrot cursor.
    julia_c: (f64, f64),
    /// Single-view Julia mode: show the Julia set of the current formula for `julia_c`.
    julia_mode: bool,
    /// Dual view: if `Some`, the Julia `c` is pinned to this Mandelbrot point (a marker
    /// is drawn there) instead of following the cursor. Click to pin, click it to release.
    julia_pin: Option<(f64, f64)>,
    /// Dual linked view: Mandelbrot (left) ↔ Julia (right).
    dual: bool,
    /// Borderless fullscreen state (toolbar toggle).
    fullscreen: bool,
    /// Viewport for the Julia panel in dual view.
    julia_viewport: Viewport,
    /// Complex coordinate under the cursor (for the status bar); `None` when off-canvas.
    pointer_complex: Option<(f64, f64)>,
    /// App-time of the last interaction; AA stays off until `SETTLE_DELAY` after it.
    settle_t: f64,
    /// Active smooth "zoom out to home" animation (Home button); `None` when idle.
    home_anim: Option<HomeAnim>,
    /// Draw the iteration orbit of the point under the cursor.
    show_orbits: bool,
    /// Fit the orbit into a fixed inset (good view at any zoom) instead of overlaying
    /// it on the fractal through the viewport.
    orbit_normalize: bool,
    /// Animate a dot racing out along the orbit, with a cycling color.
    orbit_anim: bool,
    /// Racing-dot speed (iterates per second along the path).
    orbit_anim_speed: f32,
    /// Position along the orbit path (segment units) and the dot's hue (0..1).
    orbit_phase: f32,
    orbit_hue: f32,
    /// Palette animation mode + speed (offset cycles/sec), and the ping-pong direction.
    palette_anim: PaletteAnim,
    palette_anim_speed: f32,
    anim_dir: f32,
    /// State for the randomized morphing-gradient palette mode.
    random_palette: RandomPalette,
    /// Cache for the interactive orbit overlay (avoids recomputing the bignum orbit
    /// every frame when the cursor/view haven't moved).
    orbit_cache: std::cell::RefCell<Option<OrbitCacheEntry>>,
    /// Active scripted camera tour / benchmark (None when idle).
    playback: Option<Playback>,
    /// Last benchmark report text + whether its window is open.
    bench_report: Option<String>,
    bench_open: bool,
    /// GPU adapter name (for benchmark reports).
    gpu_name: String,
    /// Host system facts (CPU / cores / cache / VRAM) for benchmark reports.
    sysinfo: SysInfo,
    /// CLI auto-benchmark: run on startup, save to this path, then quit.
    auto_benchmark: bool,
    auto_benchmark_out: Option<std::path::PathBuf>,
    auto_benchmark_done: bool,
    /// CLI render-and-exit: render one image to `auto_render_out`, then quit.
    auto_render: bool,
    auto_render_out: Option<std::path::PathBuf>,
    auto_render_done: bool,
    /// Frame-rate cap (FPS); `None` = uncapped.
    fps_cap: Option<f64>,
    /// Export dialog state.
    export_open: bool,
    export_width: u32,
    export_ss: u32,
    export_format: ExportFormat,
    export_dual_mode: DualExport,
    export_notes: String,
    export_status: Option<String>,
    /// In-flight background export; receives the final status message when done.
    export_task: Option<std::sync::mpsc::Receiver<String>>,
    /// Export progress in permille (0–1000) and a cooperative cancel flag.
    export_progress: std::sync::Arc<std::sync::atomic::AtomicU32>,
    export_cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// Last directory an export was saved to (persisted; defaults the Save dialog).
    export_last_dir: Option<std::path::PathBuf>,
    /// Gallery browser state.
    gallery_open: bool,
    gallery_dir: std::path::PathBuf,
    gallery_entries: Vec<GalleryEntry>,
    /// Bookmarks (saved views), persisted to the config dir; + window/input state.
    bookmarks: Vec<Bookmark>,
    bookmarks_open: bool,
    bookmark_name: String,
    /// Performance/diagnostic tracking + overlay.
    perf: Perf,
    max_iter: u32,
    /// Box-zoom (right-drag) start position in screen points; `None` when idle.
    box_start: Option<egui::Pos2>,
    /// Eased continuous-zoom velocity (log-rate per second; + = in, - = out, 0 = idle).
    zoom_vel: f64,
    /// Last cursor position over the canvas, for cursor-anchored continuous zoom.
    last_cursor: Option<egui::Pos2>,
    /// Selected palette index into `fractadyne_color::PRESETS`.
    palette_idx: usize,
    /// Color cycle density slider (0..1; mapped to a shader multiplier).
    cycle: f32,
    /// Palette offset slider (0..1).
    offset: f32,
    /// Distance-estimate relief lighting (slope shading from the derivative).
    light: bool,
    light_angle: f32,  // radians
    light_height: f32, // relief strength (smaller = sharper)
    /// Distance-estimate glow (contour bands near the boundary), + animation.
    de: bool,
    de_strength: f32,
    de_width: f32,
    de_anim: bool,
    de_phase: f32, // runtime (animated)
    /// Auto-scale iteration count with zoom depth (else use `max_iter` as-is).
    auto_iter: bool,
    /// Continuous-zoom speed multiplier (1.0 = default `ZOOM_RATE`).
    zoom_rate: f32,
    /// Supersampling / anti-alias factor (1 = off, 2 = 2×2, 3 = 3×3).
    aa: u32,
    /// Per-view perturbation reference cache (index 0 = main/left, 1 = dual Julia).
    /// Separate caches let both dual panels use perturbation without thrashing.
    ref_cache: [RefCache; 2],
    /// Last snapshot used for change detection (debounced auto-save).
    last_state: fractadyne_state::SessionState,
    /// App-time (s) of the last change while unsaved; `None` when clean.
    dirty_since: Option<f64>,
}

impl FractadyneApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let render_state = cc
            .wgpu_render_state
            .as_ref()
            .expect("Fractadyne requires the wgpu backend (eframe Renderer::Wgpu)");
        install_renderer(render_state);
        let gpu_name = render_state.adapter.get_info().name;

        // CLI modes (headless, for automation / debugging):
        //   --benchmark [--out PATH]                    run the built-in benchmark, save, quit
        //   --render --out IMG [view options]           render one image, save, quit
        // Render view options: --fractal NAME, --center X Y, --zoom MAG, --size W,
        //   --ss N, --iter N, --julia, --julia-c RE IM, --palette IDX.
        let args: Vec<String> = std::env::args().collect();
        let out_path = args
            .iter()
            .position(|a| a == "--out" || a == "-o")
            .and_then(|i| args.get(i + 1))
            .map(std::path::PathBuf::from);
        let auto_benchmark = args.iter().any(|a| a == "--benchmark" || a == "--bench");
        let auto_render = args.iter().any(|a| a == "--render");
        let auto_benchmark_out = out_path.clone();
        let auto_render_out = out_path.clone();

        // Restore the last session (or defaults). The center comes from the
        // full-precision decimal strings when present (deep-zoom locations survive
        // restart); older session files without them fall back to the f64 fields.
        let s = fractadyne_state::load();
        let mut viewport = Viewport::new(1280.0, 720.0);
        viewport.center_x = fractadyne_core::parse_bf(&s.center_x_str)
            .unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(s.center_x, 64));
        viewport.center_y = fractadyne_core::parse_bf(&s.center_y_str)
            .unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(s.center_y, 64));
        viewport.units_per_pixel = s.units_per_pixel;
        viewport.precision = fractadyne_core::precision_for_magnification(viewport.magnification());

        let mut app = Self {
            viewport,
            fractal: FractalKind::Mandelbrot,
            julia_c: (-0.8, 0.156),
            julia_mode: false,
            julia_pin: None,
            dual: false,
            fullscreen: false,
            julia_viewport: {
                let mut v = Viewport::new(800.0, 800.0);
                v.center_x = fractadyne_core::BigFloat::from_f64(0.0, 64);
                v.center_y = fractadyne_core::BigFloat::from_f64(0.0, 64);
                v
            },
            pointer_complex: None,
            settle_t: 0.0,
            home_anim: None,
            show_orbits: false,
            orbit_normalize: false,
            orbit_anim: false,
            orbit_anim_speed: 10.0,
            orbit_phase: 0.0,
            orbit_hue: 0.0,
            palette_anim: PaletteAnim::from_key(&s.palette_anim),
            palette_anim_speed: s.palette_anim_speed,
            anim_dir: 1.0,
            random_palette: RandomPalette::new(0x9E37_79B9 ^ BUILD_SEQ.len() as u32),
            orbit_cache: std::cell::RefCell::new(None),
            playback: None,
            bench_report: None,
            bench_open: false,
            gpu_name,
            sysinfo: gather_system_info(),
            auto_benchmark,
            auto_benchmark_out,
            auto_benchmark_done: false,
            auto_render,
            auto_render_out,
            auto_render_done: false,
            fps_cap: s.fps_cap,
            export_open: false,
            export_width: s.export_width,
            export_ss: s.export_ss,
            export_format: if s.export_format == "exr" {
                ExportFormat::Exr
            } else {
                ExportFormat::Png
            },
            export_dual_mode: match s.export_dual_mode.as_str() {
                "separate" => DualExport::Separate,
                "active" => DualExport::ActiveOnly,
                _ => DualExport::SideBySide,
            },
            export_notes: String::new(),
            export_status: None,
            export_task: None,
            export_progress: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
            export_cancel: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            export_last_dir: s.export_dir.clone().map(std::path::PathBuf::from),
            gallery_open: false,
            gallery_dir: Self::pictures_dir(),
            gallery_entries: Vec::new(),
            bookmarks: Self::load_bookmarks(),
            bookmarks_open: false,
            bookmark_name: String::new(),
            perf: Perf {
                // Default on; `--no-perf` disables, `--perf` forces on.
                enabled: !std::env::args().any(|a| a == "--no-perf"),
                ..Perf::default()
            },
            max_iter: s.max_iter,
            box_start: None,
            zoom_vel: 0.0,
            last_cursor: None,
            palette_idx: s.palette_idx,
            cycle: s.cycle,
            offset: s.offset,
            light: s.light,
            light_angle: s.light_angle,
            light_height: s.light_height,
            de: s.de,
            de_strength: s.de_strength,
            de_width: s.de_width,
            de_anim: s.de_anim,
            de_phase: 0.0,
            auto_iter: s.auto_iter,
            zoom_rate: s.zoom_rate,
            aa: s.aa,
            ref_cache: [RefCache::default(), RefCache::default()],
            last_state: s,
            dirty_since: None,
        };
        if app.auto_benchmark {
            app.start_benchmark();
        }
        if app.auto_render {
            app.apply_cli_render(&args);
        }
        app
    }

    /// Configure the view from `--render` CLI options (fractal / center / zoom / size
    /// / iterations / julia / palette). The actual render happens on the first frame.
    fn apply_cli_render(&mut self, args: &[String]) {
        let val = |name: &str| -> Option<&String> {
            args.iter().position(|a| a == name).and_then(|i| args.get(i + 1))
        };
        let two = |name: &str| -> Option<(&String, &String)> {
            args.iter()
                .position(|a| a == name)
                .and_then(|i| Some((args.get(i + 1)?, args.get(i + 2)?)))
        };
        if let Some(k) = val("--fractal").and_then(|s| FractalKind::from_name(s)) {
            self.fractal = k;
        }
        self.julia_mode = self.fractal.supports_julia() && args.iter().any(|a| a == "--julia");
        if let Some((re, im)) = two("--julia-c") {
            if let (Ok(r), Ok(i)) = (re.parse(), im.parse()) {
                self.julia_c = (r, i);
            }
        }
        // Center: explicit (full precision) or the fractal's default.
        let center = two("--center").and_then(|(xs, ys)| {
            Some((fractadyne_core::parse_bf(xs)?, fractadyne_core::parse_bf(ys)?))
        });
        let (cx, cy) = center.unwrap_or_else(|| {
            let (dx, dy) = self.fractal.default_center();
            (
                fractadyne_core::BigFloat::from_f64(dx, 64),
                fractadyne_core::BigFloat::from_f64(dy, 64),
            )
        });
        let zoom = val("--zoom").and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0);
        self.viewport.set_center_mag(cx, cy, zoom.max(1.0e-300));
        if let Some(w) = val("--size").and_then(|s| s.parse::<u32>().ok()) {
            self.export_width = w.clamp(16, 16384);
        }
        if let Some(ss) = val("--ss").and_then(|s| s.parse::<u32>().ok()) {
            self.export_ss = ss.clamp(1, 8);
        }
        if let Some(it) = val("--iter").and_then(|s| s.parse::<u32>().ok()) {
            self.max_iter = it.clamp(16, 200_000);
            self.auto_iter = false;
        }
        if let Some(p) = val("--palette").and_then(|s| s.parse::<usize>().ok()) {
            self.palette_idx = p.min(fractadyne_color::PRESETS.len() - 1);
        }
        if args.iter().any(|a| a == "--light") {
            self.light = true;
        }
        if let Some(a) = val("--light-angle").and_then(|s| s.parse::<f32>().ok()) {
            self.light_angle = a;
        }
        if args.iter().any(|a| a == "--de") {
            self.de = true;
        }
        // Output format from the file extension.
        if let Some(out) = &self.auto_render_out {
            if out.extension().and_then(|e| e.to_str()) == Some("exr") {
                self.export_format = ExportFormat::Exr;
            } else {
                self.export_format = ExportFormat::Png;
            }
        }
    }

    /// Snapshot current state and save it ~1 s after the last change (or on close).
    fn autosave(&mut self, ctx: &egui::Context) {
        let cur = fractadyne_state::SessionState {
            center_x: fractadyne_core::to_f64(&self.viewport.center_x),
            center_y: fractadyne_core::to_f64(&self.viewport.center_y),
            center_x_str: fractadyne_core::to_decimal_string(&self.viewport.center_x),
            center_y_str: fractadyne_core::to_decimal_string(&self.viewport.center_y),
            units_per_pixel: self.viewport.units_per_pixel,
            max_iter: self.max_iter,
            auto_iter: self.auto_iter,
            palette_idx: self.palette_idx,
            cycle: self.cycle,
            offset: self.offset,
            zoom_rate: self.zoom_rate,
            aa: self.aa,
            fps_cap: self.fps_cap,
            export_width: self.export_width,
            export_ss: self.export_ss,
            export_format: match self.export_format {
                ExportFormat::Png => "png".to_string(),
                ExportFormat::Exr => "exr".to_string(),
            },
            export_dir: self
                .export_last_dir
                .as_ref()
                .map(|p| p.display().to_string()),
            export_dual_mode: match self.export_dual_mode {
                DualExport::SideBySide => "side".to_string(),
                DualExport::Separate => "separate".to_string(),
                DualExport::ActiveOnly => "active".to_string(),
            },
            palette_anim: self.palette_anim.key().to_string(),
            palette_anim_speed: self.palette_anim_speed,
            light: self.light,
            light_angle: self.light_angle,
            light_height: self.light_height,
            de: self.de,
            de_strength: self.de_strength,
            de_width: self.de_width,
            de_anim: self.de_anim,
        };
        let now = ctx.input(|i| i.time);
        if cur != self.last_state {
            self.last_state = cur;
            // Mark dirty on the FIRST change only — don't keep pushing the timer
            // forward on every frame, or a continuously-changing field (e.g. the
            // animated palette offset) would prevent the 1 s idle save from ever
            // firing (it would only save on close). This way it saves ~every 1 s.
            self.dirty_since.get_or_insert(now);
        }
        let closing = ctx.input(|i| i.viewport().close_requested());
        if let Some(t) = self.dirty_since {
            if closing || now - t > 1.0 {
                fractadyne_state::save(&self.last_state);
                self.dirty_since = None;
            }
        }
    }

    /// Switch fractal type, resetting to that fractal's default view.
    fn set_fractal(&mut self, kind: FractalKind) {
        if self.fractal == kind {
            return;
        }
        self.fractal = kind;
        if !kind.supports_julia() {
            self.julia_mode = false;
        }
        let (cx, cy) = kind.default_center();
        self.viewport.reset();
        self.viewport.center_x = fractadyne_core::BigFloat::from_f64(cx, 64);
        self.viewport.center_y = fractadyne_core::BigFloat::from_f64(cy, 64);
        self.zoom_vel = 0.0;
        self.invalidate_refs(); // dynamics changed → drop the cached reference orbits
    }

    /// Drop both per-view reference caches (call when the formula/mode/center changes
    /// such that the cached references no longer apply).
    fn invalidate_refs(&mut self) {
        self.ref_cache[0].ref_pt = None;
        self.ref_cache[1].ref_pt = None;
    }

    /// Request the next animation frame. Frame pacing (the cap) is enforced at the end
    /// of `update`; this just keeps the animation loop alive.
    fn schedule_repaint(&self, ctx: &egui::Context) {
        ctx.request_repaint();
    }

    /// Pick a reference point and compute its high-precision orbit for the current
    /// formula, arranging `Z₀`/`c` for Mandelbrot vs Julia mode. Returns the orbit,
    /// its length, and the chosen reference point (for the δ-offset).
    fn compute_reference(
        &self,
        center_bf: &[fractadyne_core::BigFloat; 2],
        span: (f64, f64),
        eff_iter: u32,
        precision: usize,
        julia: bool,
    ) -> (
        std::sync::Arc<Vec<[f32; 4]>>,
        u32,
        [fractadyne_core::BigFloat; 2],
    ) {
        let formula = self.fractal.formula_id();
        let (jcx, jcy) = self.julia_c;
        let rp = fractadyne_core::best_reference(
            center_bf,
            [span.0, span.1],
            formula,
            julia,
            [jcx, jcy],
            eff_iter,
            precision,
        );
        let zero = fractadyne_core::BigFloat::from_f64(0.0, precision);
        let (z0x, z0y, cx0, cy0) = if julia {
            (
                rp[0].clone(),
                rp[1].clone(),
                fractadyne_core::BigFloat::from_f64(jcx, precision),
                fractadyne_core::BigFloat::from_f64(jcy, precision),
            )
        } else {
            (zero.clone(), zero, rp[0].clone(), rp[1].clone())
        };
        let (o, l) =
            fractadyne_core::reference_orbit(&z0x, &z0y, &cx0, &cy0, formula, eff_iter, precision);
        (std::sync::Arc::new(o), l, rp)
    }

    /// Build an export request for a given viewport + Julia flag at the export
    /// resolution. Recomputes a fresh reference orbit (deep) without touching the live
    /// cache. Height is derived from the viewport's aspect (square pixels).
    fn current_export_request_for(
        &self,
        vp: &Viewport,
        julia: bool,
    ) -> fractadyne_gpu::ExportRequest {
        let (span_x, span_y) = vp.complex_span();
        let width = self.export_width.max(1);
        let height = ((width as f64) * span_y / span_x).round().max(1.0) as u32;
        let eff_iter = if self.auto_iter {
            vp.recommended_max_iter(self.max_iter)
        } else {
            self.max_iter
        };
        let mag = vp.magnification();
        let mode: u32 = if !self.fractal.supports_perturbation() || mag < 1.0e4 {
            1
        } else if mag >= PERT_FE_THRESHOLD {
            2
        } else {
            0
        };
        let precision = fractadyne_core::precision_for_magnification(mag);
        let (cx, cy) = vp.center_f64();
        let delta_exp = delta_exponent(span_x);
        let dscale = 2f64.powi(-delta_exp);

        let mut ref_offset = [0.0_f32; 4];
        let mut orbit = std::sync::Arc::new(Vec::new());
        let mut orbit_len = 0u32;
        if mode != 1 {
            let center_bf = [vp.center_x.clone(), vp.center_y.clone()];
            let (orbit_arc, len, rp) =
                self.compute_reference(&center_bf, (span_x, span_y), eff_iter, precision, julia);
            orbit = orbit_arc;
            orbit_len = len;
            let dx = fractadyne_core::sub_f64(&vp.center_x, &rp[0], precision) * dscale;
            let dy = fractadyne_core::sub_f64(&vp.center_y, &rp[1], precision) * dscale;
            let dxh = dx as f32;
            let dyh = dy as f32;
            ref_offset = [dxh, dyh, (dx - dxh as f64) as f32, (dy - dyh as f64) as f32];
        }

        let cxh = cx as f32;
        let cyh = cy as f32;
        let center = [cxh, cyh, (cx - cxh as f64) as f32, (cy - cyh as f64) as f32];
        let (jcx, jcy) = self.julia_c;
        let jcxh = jcx as f32;
        let jcyh = jcy as f32;
        let julia_c = [jcxh, jcyh, (jcx - jcxh as f64) as f32, (jcy - jcyh as f64) as f32];
        let (stops, stop_count) = self.active_stops();

        fractadyne_gpu::ExportRequest {
            width,
            height,
            ss: self.export_ss.max(1),
            span: [span_x, span_y],
            center,
            ref_offset,
            delta_exp,
            julia_c,
            orbit,
            orbit_len,
            max_iter: eff_iter,
            mode,
            formula: self.fractal.formula_id(),
            julia: julia as u32,
            cycle: 0.004 + self.cycle * 0.06,
            offset: self.offset,
            stop_count,
            stops,
            light: self.light as u32,
            light_angle: self.light_angle,
            light_height: self.light_height,
            de_on: self.de as u32,
            de_strength: self.de_strength,
            de_width: self.de_width,
            de_phase: self.de_phase,
        }
    }

    /// Build the export job for the current state (single view, or dual per the chosen
    /// layout).
    fn build_export_job(&self) -> ExportJob {
        if self.dual {
            let map = self.current_export_request_for(&self.viewport, false);
            let jul = self.current_export_request_for(&self.julia_viewport, true);
            match self.export_dual_mode {
                DualExport::SideBySide => ExportJob::SideBySide(map, jul),
                DualExport::Separate => ExportJob::Separate(map, jul),
                DualExport::ActiveOnly => ExportJob::Single(map),
            }
        } else {
            ExportJob::Single(self.current_export_request_for(&self.viewport, self.julia_mode))
        }
    }

    /// Default Pictures directory (fallback: current dir).
    fn pictures_dir() -> std::path::PathBuf {
        directories::UserDirs::new()
            .and_then(|u| u.picture_dir().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."))
    }

    /// Path to the bookmarks file in the OS config dir.
    fn bookmarks_path() -> Option<std::path::PathBuf> {
        directories::ProjectDirs::from("com", "Fractadyne", "Fractadyne")
            .map(|d| d.config_dir().join("bookmarks.toml"))
    }

    /// Load saved bookmarks (empty list if none / unreadable).
    fn load_bookmarks() -> Vec<Bookmark> {
        Self::bookmarks_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|t| toml::from_str::<BookmarkFile>(&t).ok())
            .map(|f| f.bookmark)
            .unwrap_or_default()
    }

    /// Persist bookmarks (best-effort).
    fn save_bookmarks(&self) {
        let Some(path) = Self::bookmarks_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let file = BookmarkFile {
            bookmark: self.bookmarks.clone(),
        };
        if let Ok(text) = toml::to_string_pretty(&file) {
            let _ = std::fs::write(path, text);
        }
    }

    /// Add a bookmark of the current view (auto-names it if `name` is blank).
    fn add_bookmark(&mut self, name: &str) {
        let name = if name.trim().is_empty() {
            format!(
                "{} {}×",
                self.fractal.name(),
                fmt_zoom(self.viewport.magnification())
            )
        } else {
            name.trim().to_string()
        };
        self.bookmarks.push(Bookmark {
            name,
            meta: self.view_metadata(),
        });
        self.save_bookmarks();
    }

    /// UTC civil date/time `(year, month, day, hour, min, sec)` from a Unix timestamp
    /// (Hinnant's civil-from-days algorithm).
    fn civil_utc(secs: u64) -> (i64, i64, i64, u64, u64, u64) {
        let days = (secs / 86400) as i64;
        let rem = secs % 86400;
        let (hh, mm, sss) = (rem / 3600, (rem % 3600) / 60, rem % 60);
        let z = days + 719468;
        let era = if z >= 0 { z } else { z - 146096 } / 146097;
        let doe = z - era * 146097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        (y, m, d, hh, mm, sss)
    }

    /// UTC `YYYY-MM-DD HH:MM:SS` from a Unix timestamp.
    fn utc_date_string(secs: u64) -> String {
        let (y, m, d, hh, mm, sss) = Self::civil_utc(secs);
        format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}:{sss:02} UTC")
    }

    /// Filename-safe `YYYYMMDD_HHMMSS` stamp (local-readable, sorts chronologically).
    fn file_stamp(secs: u64) -> String {
        let (y, m, d, hh, mm, sss) = Self::civil_utc(secs);
        format!("{y:04}{m:02}{d:02}_{hh:02}{mm:02}{sss:02}")
    }

    /// Reloadable view-state metadata embedded in exports. The center is stored as
    /// full-precision decimal so deep-zoom positions round-trip exactly.
    fn view_metadata(&self) -> String {
        let (jcx, jcy) = self.julia_c;
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        // Latin-1 / single-line safe notes (PNG tEXt), max 120 chars.
        let notes: String = self
            .export_notes
            .chars()
            .filter(|c| !c.is_control() && (*c as u32) <= 0xFF)
            .take(120)
            .collect();
        format!(
            "app=Fractadyne\nversion={}\nformat_version=1\nsaved_unix={}\nsaved={}\n\
             notes={}\nfractal={}\njulia={}\njulia_c_re={:.17e}\njulia_c_im={:.17e}\n\
             center_x={}\ncenter_y={}\nupp={:.17e}\nzoom={}\nmax_iter={}\nauto_iter={}\n\
             palette={}\ncycle={}\noffset={}\naa={}\n",
            version_string(),
            secs,
            Self::utc_date_string(secs),
            notes,
            self.fractal.name(),
            self.julia_mode as u32,
            jcx,
            jcy,
            fractadyne_core::to_decimal_string(&self.viewport.center_x),
            fractadyne_core::to_decimal_string(&self.viewport.center_y),
            self.viewport.units_per_pixel,
            self.viewport.magnification(),
            self.max_iter,
            self.auto_iter as u32,
            self.palette_idx,
            self.cycle,
            self.offset,
            self.aa,
        )
    }

    /// Restore the view from metadata read out of an exported PNG.
    fn load_view_metadata(&mut self, meta: &str) {
        let get = |key: &str| -> Option<String> {
            meta.lines().find_map(|l| {
                l.split_once('=')
                    .filter(|(k, _)| k.trim() == key)
                    .map(|(_, v)| v.trim().to_string())
            })
        };
        if let Some(f) = get("fractal").and_then(|s| FractalKind::from_name(&s)) {
            self.fractal = f;
        }
        self.julia_mode =
            get("julia").map(|s| s == "1").unwrap_or(false) && self.fractal.supports_julia();
        if let (Some(re), Some(im)) = (
            get("julia_c_re").and_then(|s| s.parse().ok()),
            get("julia_c_im").and_then(|s| s.parse().ok()),
        ) {
            self.julia_c = (re, im);
        }
        if let Some(cx) = get("center_x").and_then(|s| fractadyne_core::parse_bf(&s)) {
            self.viewport.center_x = cx;
        }
        if let Some(cy) = get("center_y").and_then(|s| fractadyne_core::parse_bf(&s)) {
            self.viewport.center_y = cy;
        }
        if let Some(upp) = get("upp").and_then(|s| s.parse::<f64>().ok()) {
            self.viewport.units_per_pixel = upp;
        }
        if let Some(mi) = get("max_iter").and_then(|s| s.parse().ok()) {
            self.max_iter = mi;
        }
        if let Some(ai) = get("auto_iter") {
            self.auto_iter = ai == "1";
        }
        if let Some(p) = get("palette").and_then(|s| s.parse::<usize>().ok()) {
            if p < fractadyne_color::PRESETS.len() {
                self.palette_idx = p;
            }
        }
        if let Some(c) = get("cycle").and_then(|s| s.parse().ok()) {
            self.cycle = c;
        }
        if let Some(o) = get("offset").and_then(|s| s.parse().ok()) {
            self.offset = o;
        }
        if let Some(a) = get("aa").and_then(|s| s.parse().ok()) {
            self.aa = a;
        }
        if let Some(n) = get("notes") {
            self.export_notes = n;
        }
        // Match the viewport's working precision to the restored zoom; drop caches.
        self.viewport.precision =
            fractadyne_core::precision_for_magnification(self.viewport.magnification());
        self.invalidate_refs();
        self.zoom_vel = 0.0;
    }

    /// Open a previously-exported PNG/EXR and restore its view (via a native dialog).
    fn open_view(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("Fractadyne image", &["png", "exr"])
            .set_directory(Self::pictures_dir())
            .pick_file();
        let Some(path) = path else { return };
        let is_exr = path
            .extension()
            .map(|e| e.eq_ignore_ascii_case("exr"))
            .unwrap_or(false);
        let meta = if is_exr {
            fractadyne_export::read_exr_metadata(&path)
        } else {
            fractadyne_export::read_png_metadata(&path)
        };
        match meta {
            Some(m) => {
                self.load_view_metadata(&m);
                self.export_status = Some(format!("Loaded view from {}", path.display()));
            }
            None => {
                self.export_status =
                    Some("That file has no embedded Fractadyne view metadata.".to_string());
            }
        }
    }

    /// Scan the gallery folder for exported PNG/EXR files with Fractadyne metadata,
    /// newest first. Thumbnails load lazily afterward.
    fn scan_gallery(&mut self) {
        self.gallery_entries.clear();
        let Ok(rd) = std::fs::read_dir(&self.gallery_dir) else {
            return;
        };
        for path in rd.flatten().map(|e| e.path()) {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase())
                .unwrap_or_default();
            let meta = match ext.as_str() {
                "png" => fractadyne_export::read_png_metadata(&path),
                "exr" => fractadyne_export::read_exr_metadata(&path),
                _ => None,
            };
            let Some(m) = meta else { continue };
            if meta_get(&m, "app") != "Fractadyne" {
                continue;
            }
            let zoom = meta_get(&m, "zoom")
                .parse::<f64>()
                .map(|z| format!("{}×", fmt_zoom(z)))
                .unwrap_or_default();
            self.gallery_entries.push(GalleryEntry {
                fractal: meta_get(&m, "fractal"),
                zoom,
                saved: meta_get(&m, "saved"),
                notes: meta_get(&m, "notes"),
                app_version: format!("Fractadyne {}", meta_get(&m, "version")),
                saved_unix: meta_get(&m, "saved_unix").parse().unwrap_or(0),
                path,
                meta: m,
                thumb: None,
                thumb_tried: false,
            });
        }
        self.gallery_entries
            .sort_by(|a, b| b.saved_unix.cmp(&a.saved_unix));
    }

    fn export_ext(&self) -> &'static str {
        match self.export_format {
            ExportFormat::Png => "png",
            ExportFormat::Exr => "exr",
        }
    }

    /// Default timestamped export filename for the current fractal.
    fn export_default_name(&self) -> String {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        format!(
            "fractadyne_{}_{}.{}",
            self.fractal.name().replace(' ', ""),
            Self::file_stamp(stamp),
            self.export_ext(),
        )
    }

    /// Start a background export, prompting for a path (modal Save dialog).
    fn start_export(&mut self, device: eframe::wgpu::Device, queue: eframe::wgpu::Queue) {
        if self.export_task.is_some() {
            return;
        }
        let ext = self.export_ext();
        let start_dir = self
            .export_last_dir
            .clone()
            .filter(|d| d.is_dir())
            .unwrap_or_else(Self::pictures_dir);
        let path = rfd::FileDialog::new()
            .set_directory(start_dir)
            .set_file_name(self.export_default_name())
            .add_filter(ext.to_uppercase(), &[ext])
            .save_file();
        let Some(path) = path else {
            self.export_status = Some("Export canceled.".to_string());
            return;
        };
        self.start_export_to(device, queue, path);
    }

    /// Quick export (hotkey): no dialog — save to the last-used folder with an auto name.
    fn quick_export(&mut self, device: eframe::wgpu::Device, queue: eframe::wgpu::Queue) {
        if self.export_task.is_some() {
            return;
        }
        let dir = self
            .export_last_dir
            .clone()
            .filter(|d| d.is_dir())
            .unwrap_or_else(Self::pictures_dir);
        let path = dir.join(self.export_default_name());
        self.start_export_to(device, queue, path);
    }

    /// Synchronously render the current view and write it to `path` (used by the
    /// headless `--render` CLI mode). Blocks until done; returns a status message.
    fn render_to_file(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        path: &std::path::Path,
    ) -> Result<String, String> {
        use std::sync::atomic::AtomicBool;
        use std::sync::atomic::AtomicU32;
        let progress = AtomicU32::new(0);
        let cancel = AtomicBool::new(false);
        let meta = self.view_metadata();
        let fmt = self.export_format;
        let write = |p: &std::path::Path, w: u32, h: u32, px: &[f32]| match fmt {
            ExportFormat::Png => fractadyne_export::write_png(p, w, h, px, Some(&meta)),
            ExportFormat::Exr => fractadyne_export::write_exr(p, w, h, px, Some(&meta)),
        };
        let render = |req: &fractadyne_gpu::ExportRequest| {
            fractadyne_gpu::render_export(device, queue, req, &progress, &cancel)
        };
        match self.build_export_job() {
            ExportJob::Single(req) => {
                let r = render(&req)?;
                write(path, r.width, r.height, &r.pixels)?;
                Ok(format!("Saved {}×{} → {}", r.width, r.height, path.display()))
            }
            ExportJob::SideBySide(a, b) => {
                let (ra, rb) = (render(&a)?, render(&b)?);
                let (w, h, px) = stitch_side_by_side(&ra, &rb);
                write(path, w, h, &px)?;
                Ok(format!("Saved {w}×{h} → {}", path.display()))
            }
            ExportJob::Separate(a, b) => {
                let (pmap, pjul) = separate_paths(path);
                let ra = render(&a)?;
                write(&pmap, ra.width, ra.height, &ra.pixels)?;
                let rb = render(&b)?;
                write(&pjul, rb.width, rb.height, &rb.pixels)?;
                Ok(format!("Saved 2 files → {}", pmap.display()))
            }
        }
    }

    /// Render the current job on a worker thread and write to `path` (or, for dual
    /// "separate", to `path` + a sibling). The UI stays responsive; result via channel.
    fn start_export_to(
        &mut self,
        device: eframe::wgpu::Device,
        queue: eframe::wgpu::Queue,
        path: std::path::PathBuf,
    ) {
        if self.export_task.is_some() {
            return;
        }
        if let Some(parent) = path.parent() {
            self.export_last_dir = Some(parent.to_path_buf());
        }
        let job = self.build_export_job();
        let meta = self.view_metadata();
        let format = self.export_format;
        use std::sync::atomic::Ordering::Relaxed;
        self.export_progress.store(0, Relaxed);
        self.export_cancel.store(false, Relaxed);
        let progress = self.export_progress.clone();
        let cancel = self.export_cancel.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.export_task = Some(rx);
        self.export_status = Some("Rendering…".to_string());
        std::thread::spawn(move || {
            let render = |req: &fractadyne_gpu::ExportRequest| {
                fractadyne_gpu::render_export(&device, &queue, req, &progress, &cancel)
            };
            let write = |p: &std::path::Path, w: u32, h: u32, px: &[f32]| match format {
                ExportFormat::Png => fractadyne_export::write_png(p, w, h, px, Some(&meta)),
                ExportFormat::Exr => fractadyne_export::write_exr(p, w, h, px, Some(&meta)),
            };
            let msg = (|| -> Result<String, String> {
                match job {
                    ExportJob::Single(req) => {
                        let r = render(&req)?;
                        progress.store(2000, Relaxed);
                        write(&path, r.width, r.height, &r.pixels)?;
                        Ok(format!("Saved {}×{} → {}", r.width, r.height, path.display()))
                    }
                    ExportJob::SideBySide(a, b) => {
                        let ra = render(&a)?;
                        let rb = render(&b)?;
                        progress.store(2000, Relaxed);
                        let (w, h, px) = stitch_side_by_side(&ra, &rb);
                        write(&path, w, h, &px)?;
                        Ok(format!("Saved {w}×{h} → {}", path.display()))
                    }
                    ExportJob::Separate(a, b) => {
                        let (pmap, pjul) = separate_paths(&path);
                        let ra = render(&a)?;
                        write(&pmap, ra.width, ra.height, &ra.pixels)?;
                        let rb = render(&b)?;
                        progress.store(2000, Relaxed);
                        write(&pjul, rb.width, rb.height, &rb.pixels)?;
                        Ok(format!("Saved 2 files → {}", pmap.display()))
                    }
                }
            })();
            let _ = tx.send(match msg {
                Ok(m) => m,
                Err(e) if e == "canceled" => "Export canceled.".to_string(),
                Err(e) => format!("Export failed: {e}"),
            });
        });
    }

    /// Build the GPU params for one fractal view, computing the perturbation
    /// reference (deep Mandelbrot) or selecting the direct df32 path. Shared by the
    /// single view and both panels of the dual view.
    #[allow(clippy::too_many_arguments)]
    fn build_params(
        &mut self,
        center_bf: [fractadyne_core::BigFloat; 2],
        center: (f64, f64),
        span: (f64, f64),
        magnification: f64,
        fractal: FractalKind,
        julia: bool,
        eff_iter: u32,
        interacting: bool,
        resolution: [u32; 2],
        view_id: u32,
    ) -> MandelbrotParams {
        let (stops, stop_count) = self.active_stops();
        let (cx, cy) = center;
        let (span_x, span_y) = span;

        // Bound per-frame GPU work so a single render can't trip the OS GPU watchdog
        // (TDR ≈ 2 s → device-lost crash). Work ≈ texels × iterations = px·ss²·iter.
        // Cap supersampling first (keeps native resolution); only at extreme depth on a
        // very large window is the iteration count itself clamped.
        let px = (resolution[0].max(1) as u64) * (resolution[1].max(1) as u64);
        // Keep the full iteration count — clamping iterations at deep zoom made the whole
        // view escape "late"/never, rendering as flat interior (the uniform screen). To
        // stay under the GPU-watchdog budget (texels × iters), reduce the iteration-
        // texture *resolution* instead (the color pass upscales it): graceful softness
        // on a big window at extreme depth, never a blank. ss is fit afterward.
        let gpu_iter = eff_iter.min(50_000);
        let want = px.saturating_mul(gpu_iter.max(1) as u64);
        let res_scale = if want > WORK_BUDGET {
            (WORK_BUDGET as f64 / want as f64).sqrt()
        } else {
            1.0
        };
        let resolution = if res_scale < 1.0 {
            [
                ((resolution[0] as f64 * res_scale) as u32).max(16),
                ((resolution[1] as f64 * res_scale) as u32).max(16),
            ]
        } else {
            resolution
        };
        let spx = (resolution[0] as u64) * (resolution[1] as u64);
        let max_ss = ((WORK_BUDGET / spx.saturating_mul(gpu_iter.max(1) as u64).max(1)) as f64)
            .sqrt()
            .max(1.0) as u32;
        let ss = if interacting { 1 } else { self.aa.min(max_ss) };

        // Render path: 1 = direct df32 (shallow / unsupported formulas), 0 = df32
        // perturbation (fast, common deep range), 2 = floatexp perturbation (past df32's
        // ~1e30× exponent limit → unlimited depth, ~1.7× costlier so only when needed).
        let mode: u32 = if !fractal.supports_perturbation() || magnification < 1.0e4 {
            1
        } else if magnification >= PERT_FE_THRESHOLD {
            2
        } else {
            0
        };
        let precision = fractadyne_core::precision_for_magnification(magnification);
        let vi = view_id as usize;

        // Shared base-2 exponent for the δ mantissas: ~log2(span) so the per-pixel
        // offset mantissa is O(1) regardless of depth (floatexp delta → unlimited zoom).
        let delta_exp = delta_exponent(span_x);
        let dscale = 2f64.powi(-delta_exp);

        let mut ref_offset = [0.0_f32; 4];
        if mode != 1 {
            let drift = self.ref_cache[vi].ref_pt.as_ref().map(|r| {
                let rx = fractadyne_core::to_f64(&r[0]);
                let ry = fractadyne_core::to_f64(&r[1]);
                (((cx - rx) / span_x).abs(), ((cy - ry) / span_y).abs())
            });
            // Recomputing the reference orbit is a slow bignum job. `best_reference`
            // legitimately sits up to ~0.4 span off-center, so we must NOT treat that
            // as stale (doing so caused a per-frame recompute loop). When settled we
            // recompute whenever the reference left the view or precision/iters grew.
            //
            // During motion we used to defer entirely (until "gone"), which left a
            // stale/out-of-view/low-precision reference → soft "impressionist" frames
            // while zooming. Now we ALSO refresh during motion when the reference is
            // out of view or under-precise, but **throttled** (≤ ~1 recompute / 90 ms)
            // so the bignum cost doesn't stall every frame — keeping deep zoom sharp
            // without tanking the frame-rate. (Affordable since the release build made
            // bignum ~8× faster.)
            let out_of_view = drift.map_or(true, |(dx, dy)| dx > 0.5 || dy > 0.5);
            let needs_quality = precision > self.ref_cache[vi].orbit_prec
                || eff_iter > self.ref_cache[vi].orbit_iter;
            let gone = drift.map_or(true, |(dx, dy)| dx > 1.5 || dy > 1.5);
            let recompute = if interacting {
                // Adaptive throttle: keep recompute to ≲ ~30% of wall time by spacing
                // refreshes at ~2.5× the last recompute's duration (so a slow debug
                // bignum doesn't stall motion, while a fast release build refreshes
                // often). Min 90 ms.
                let spacing = (self.perf.recompute_ms / 1000.0 * 2.5).max(0.09);
                let throttle_ok = self.ref_cache[vi]
                    .last_recompute
                    .map_or(true, |t| t.elapsed().as_secs_f64() > spacing);
                gone || ((out_of_view || needs_quality) && throttle_ok)
            } else {
                out_of_view || needs_quality
            };
            if recompute {
                let t = Instant::now();
                let (orbit, orbit_len, rp) =
                    self.compute_reference(&center_bf, (span_x, span_y), eff_iter, precision, julia);
                let vc = &mut self.ref_cache[vi];
                vc.ref_pt = Some(rp);
                vc.orbit = orbit;
                vc.orbit_len = orbit_len;
                vc.orbit_prec = precision;
                vc.orbit_iter = eff_iter;
                vc.orbit_id = vc.orbit_id.wrapping_add(1);
                vc.last_recompute = Some(Instant::now());
                self.perf.recompute_ms = t.elapsed().as_secs_f64() * 1000.0;
                self.perf.recompute_total += 1;
                self.perf.rate_count += 1;
            }
            let rp = self.ref_cache[vi].ref_pt.as_ref().unwrap();
            // δ = center − reference, carried as a mantissa scaled by 2^-delta_exp
            // (so it stays O(1) in df32 at any depth; the GPU re-applies the exponent).
            let dx = fractadyne_core::sub_f64(&center_bf[0], &rp[0], precision) * dscale;
            let dy = fractadyne_core::sub_f64(&center_bf[1], &rp[1], precision) * dscale;
            let dxh = dx as f32;
            let dyh = dy as f32;
            ref_offset = [dxh, dyh, (dx - dxh as f64) as f32, (dy - dyh as f64) as f32];
        }

        let cxh = cx as f32;
        let cyh = cy as f32;
        let center_df = [cxh, cyh, (cx - cxh as f64) as f32, (cy - cyh as f64) as f32];
        let (jcx, jcy) = self.julia_c;
        let jcxh = jcx as f32;
        let jcyh = jcy as f32;
        let julia_c = [jcxh, jcyh, (jcx - jcxh as f64) as f32, (jcy - jcyh as f64) as f32];

        if view_id == 0 {
            self.perf.last_mode = mode;
            self.perf.last_eff_iter = eff_iter;
            self.perf.last_precision = precision;
            self.perf.last_orbit_len = self.ref_cache[vi].orbit_len;
        }

        MandelbrotParams {
            orbit: self.ref_cache[vi].orbit.clone(),
            orbit_id: self.ref_cache[vi].orbit_id,
            orbit_len: self.ref_cache[vi].orbit_len,
            ref_offset,
            delta_exp,
            center: center_df,
            julia_c,
            mode,
            formula: fractal.formula_id(),
            julia: julia as u32,
            span: [span_x, span_y],
            max_iter: gpu_iter,
            cycle: 0.004 + self.cycle * 0.06,
            offset: self.offset,
            stop_count,
            stops,
            light: self.light as u32,
            light_angle: self.light_angle,
            light_height: self.light_height,
            de_on: self.de as u32,
            de_strength: self.de_strength,
            de_width: self.de_width,
            de_phase: self.de_phase,
            resolution,
            ss,
            view_id,
        }
    }

    /// Render one fractal panel: navigation (drag-pan, wheel-zoom) + draw. Returns
    /// the panel's response (so the caller can read hover for the dual-view link).
    fn nav_and_draw(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        rect: egui::Rect,
        ppp: f64,
        scroll: f64,
        is_julia: bool,
    ) -> egui::Response {
        let resp = ui.allocate_rect(rect, egui::Sense::click_and_drag());
        let vp = if is_julia {
            &mut self.julia_viewport
        } else {
            &mut self.viewport
        };
        vp.set_size(rect.width() as f64 * ppp, rect.height() as f64 * ppp);
        if resp.dragged_by(egui::PointerButton::Primary) {
            let d = resp.drag_delta();
            vp.pan_pixels(d.x as f64 * ppp, d.y as f64 * ppp);
        }
        let hovering = resp.hover_pos().is_some();
        if scroll != 0.0 {
            if let Some(p) = resp.hover_pos() {
                let l = p - rect.min;
                let f = (-0.0015 * scroll).exp();
                vp.zoom_at(l.x as f64 * ppp, l.y as f64 * ppp, f);
            }
        }
        let now = ctx.input(|i| i.time);
        let active = resp.dragged()
            || (scroll != 0.0 && hovering)
            || (self.zoom_vel.abs() > 1e-3 && hovering);
        if active {
            self.settle_t = now;
        }
        let interacting = now - self.settle_t < SETTLE_DELAY;

        let eff_iter = if self.auto_iter {
            vp.recommended_max_iter(self.max_iter)
        } else {
            self.max_iter
        };
        let center_bf = [vp.center_x.clone(), vp.center_y.clone()];
        let center = vp.center_f64();
        let span = vp.complex_span();
        let mag = vp.magnification();
        let res = [
            (rect.width() as f64 * ppp) as u32,
            (rect.height() as f64 * ppp) as u32,
        ];
        let view_id = if is_julia { 1 } else { 0 };
        let params = self.build_params(
            center_bf,
            center,
            span,
            mag,
            self.fractal,
            is_julia,
            eff_iter,
            interacting,
            res,
            view_id,
        );
        add_mandelbrot(ui.painter(), rect, params);
        resp
    }

    /// Dual linked view: Mandelbrot (left) ↔ Julia (right). Hovering the Mandelbrot
    /// sets the Julia parameter `c`.
    fn draw_dual(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let ppp = ctx.pixels_per_point() as f64;
        let full = ui.max_rect();
        let mid = (full.min.x + full.max.x) * 0.5;
        let left = egui::Rect::from_min_max(full.min, egui::pos2(mid - 1.0, full.max.y));
        let right = egui::Rect::from_min_max(egui::pos2(mid + 1.0, full.min.y), full.max);
        let scroll = ctx.input(|i| i.smooth_scroll_delta.y) as f64;

        // Continuous zoom (hold Space / Shift+Space) toward the cursor, on the panel
        // it is over. Applied before drawing so this frame reflects it.
        let pointer = ctx.input(|i| i.pointer.hover_pos());
        let (space, shift) = ctx.input(|i| (i.key_down(egui::Key::Space), i.modifiers.shift));
        let dt = (ctx.input(|i| i.stable_dt) as f64).clamp(0.0, 0.1);
        let panel = pointer.and_then(|p| {
            if left.contains(p) {
                Some((p, left, false))
            } else if right.contains(p) {
                Some((p, right, true))
            } else {
                None
            }
        });
        let rate = ZOOM_RATE * self.zoom_rate as f64;
        let target = if space && panel.is_some() {
            if shift {
                -rate
            } else {
                rate
            }
        } else {
            0.0
        };
        let ease = 1.0 - (-dt / EASE_TAU).exp();
        self.zoom_vel += (target - self.zoom_vel) * ease;
        if target != 0.0 || self.zoom_vel.abs() > 1e-3 {
            self.schedule_repaint(ctx);
        }
        if self.zoom_vel.abs() > 1e-3 {
            if let Some((p, r, is_julia)) = panel {
                let l = p - r.min;
                let factor = (-self.zoom_vel * dt).exp();
                let vp = if is_julia {
                    &mut self.julia_viewport
                } else {
                    &mut self.viewport
                };
                vp.set_size(r.width() as f64 * ppp, r.height() as f64 * ppp);
                vp.zoom_at(l.x as f64 * ppp, l.y as f64 * ppp, factor);
            }
        }

        // Left: Mandelbrot (sets the Mandelbrot viewport size, renders the parameter
        // plane). Drawn first so the size is current before we read a complex coord.
        let resp_l = self.nav_and_draw(ui, ctx, left, ppp, scroll, false);

        // Click toggles the Julia pin: freeze `c` at the clicked point (and mark it),
        // or release if the click lands on the existing marker → resume live hover.
        if resp_l.clicked() {
            if let Some(pos) = resp_l.interact_pointer_pos() {
                let l = pos - left.min;
                let cc = self
                    .viewport
                    .complex_at_pixel_f64(l.x as f64 * ppp, l.y as f64 * ppp);
                let on_marker = self.julia_pin.is_some_and(|pin| {
                    (pos - self.complex_screen_pos(pin, left, ppp)).length() < 12.0
                });
                if on_marker {
                    self.julia_pin = None; // release → live hover resumes
                } else {
                    self.julia_pin = Some(cc);
                    self.julia_c = cc;
                    self.settle_t = ctx.input(|i| i.time);
                    self.ref_cache[1].ref_pt = None; // Julia changed
                }
            }
        }

        // Cursor readout + (when not pinned) live Julia `c` from the hovered panel.
        let mut pc = None;
        if let Some((p, r, is_julia)) = panel {
            let l = p - r.min;
            let coord = if is_julia {
                self.julia_viewport
                    .complex_at_pixel_f64(l.x as f64 * ppp, l.y as f64 * ppp)
            } else {
                self.viewport
                    .complex_at_pixel_f64(l.x as f64 * ppp, l.y as f64 * ppp)
            };
            pc = Some(coord);
            if !is_julia && self.julia_pin.is_none() && coord != self.julia_c {
                self.julia_c = coord; // live: cursor over Mandelbrot drives the Julia
                self.settle_t = ctx.input(|i| i.time);
                self.ref_cache[1].ref_pt = None;
                self.schedule_repaint(ctx);
            }
        }

        // Right: Julia for the current c.
        let _ = self.nav_and_draw(ui, ctx, right, ppp, scroll, true);

        // Marker at the pinned point on the Mandelbrot panel.
        if let Some(pin) = self.julia_pin {
            let sp = self.complex_screen_pos(pin, left, ppp);
            if left.contains(sp) {
                let painter = ui.painter_at(left);
                painter.circle_stroke(sp, 6.0, egui::Stroke::new(1.5, egui::Color32::WHITE));
                painter.circle_filled(sp, 1.5, egui::Color32::WHITE);
            }
        }

        // Orbit overlay on the hovered panel.
        if self.show_orbits {
            if let Some((p, r, is_julia)) = panel {
                let l = p - r.min;
                let vp = if is_julia {
                    &self.julia_viewport
                } else {
                    &self.viewport
                };
                let cpx = (l.x as f64 * ppp, l.y as f64 * ppp);
                let painter = ui.painter_at(r);
                self.draw_orbit(&painter, r, vp, cpx, is_julia, ppp);
            }
        }

        self.pointer_complex = pc;
    }

    /// Screen position (points) of a complex coordinate in the Mandelbrot viewport,
    /// within the given panel rect. Inverse of `complex_at_pixel_f64`.
    fn complex_screen_pos(&self, c: (f64, f64), rect: egui::Rect, ppp: f64) -> egui::Pos2 {
        let (cx, cy) = self.viewport.center_f64();
        let upp = self.viewport.units_per_pixel;
        let px = (c.0 - cx) / upp + self.viewport.width_px * 0.5;
        let py = self.viewport.height_px * 0.5 - (c.1 - cy) / upp;
        egui::pos2(
            rect.min.x + (px / ppp) as f32,
            rect.min.y + (py / ppp) as f32,
        )
    }

    /// Draw the iteration orbit of `point` (the cursor's complex coordinate) onto
    /// `rect`, using `vp` for the complex→screen mapping. `is_julia` selects the
    /// start/parameter convention (matches the shader): escape-time families orbit
    /// `z₀ = 0, c = point`; Julia/Newton orbit `z₀ = point` with the fixed `c`.
    ///
    /// `cursor_px` is the cursor position in device pixels within the panel. At
    /// shallow/moderate zoom the orbit is iterated in `f64` from the exact cursor
    /// point. Past ~1e12× — where `f64` can't resolve the location — it iterates in
    /// **bignum from the cursor's high-precision coordinate**, so the orbit is the real
    /// orbit *and* still follows the cursor (recomputed each move).
    fn draw_orbit(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        vp: &Viewport,
        cursor_px: (f64, f64),
        is_julia: bool,
        ppp: f64,
    ) {
        let formula = self.fractal.formula_id();
        let mag = vp.magnification();
        let (cx0, cy0) = vp.center_f64();
        let key = OrbitKey {
            px: cursor_px.0,
            py: cursor_px.1,
            cx: cx0,
            cy: cy0,
            upp: vp.units_per_pixel,
            julia: is_julia,
            formula,
            jcx: self.julia_c.0,
            jcy: self.julia_c.1,
        };
        // Reuse the cached orbit unless the cursor or view changed (the bignum orbit is
        // costly, and the app may repaint for unrelated reasons e.g. palette animation).
        let cached = self
            .orbit_cache
            .borrow()
            .as_ref()
            .filter(|e| e.key == key)
            .map(|e| e.pts.clone());
        let pts: Vec<(f64, f64)> = if let Some(p) = cached {
            p
        } else {
            let mut p: Vec<(f64, f64)> = if mag > 1.0e12 && self.fractal.supports_perturbation() {
                // Deep: iterate in bignum from the cursor's arbitrary-precision coord,
                // running out toward escape so the divergent (cursor-sensitive) tail shows.
                let (cbx, cby) = vp.pixel_to_complex(cursor_px.0, cursor_px.1);
                let prec = fractadyne_core::precision_for_magnification(mag);
                let bf = |v: f64| fractadyne_core::BigFloat::from_f64(v, prec);
                let (z0x, z0y, cx, cy) = if is_julia {
                    (cbx, cby, bf(self.julia_c.0), bf(self.julia_c.1))
                } else {
                    (bf(0.0), bf(0.0), cbx, cby)
                };
                let (orbit, len) = fractadyne_core::reference_orbit(
                    &z0x, &z0y, &cx, &cy, formula, ORBIT_MAX_DEEP, prec,
                );
                let n = (len as usize).min(orbit.len());
                orbit[..n]
                    .iter()
                    .map(|s| (s[0] as f64 + s[2] as f64, s[1] as f64 + s[3] as f64))
                    .collect()
            } else {
                let point = vp.complex_at_pixel_f64(cursor_px.0, cursor_px.1);
                let newton = formula == 9;
                let (z0, c) = if is_julia {
                    (point, self.julia_c)
                } else if newton {
                    (point, (0.0, 0.0))
                } else {
                    ((0.0, 0.0), point)
                };
                fractadyne_core::orbit_points(z0, c, formula, ORBIT_MAX, 1.0e8)
            };
            // Trim the final escape to infinity so the normalized fit / racing dot reflect
            // the real |z|≲4 trajectory rather than one huge blown-up iterate.
            if let Some(k) = p.iter().position(|q| q.0 * q.0 + q.1 * q.1 > 16.0) {
                p.truncate(k + 1);
            }
            *self.orbit_cache.borrow_mut() = Some(OrbitCacheEntry {
                key,
                pts: p.clone(),
            });
            p
        };
        if pts.len() < 2 {
            return;
        }
        let screen: Vec<egui::Pos2> = if self.orbit_normalize {
            // Normalized: fit the orbit's bounding box to the whole panel, so it reads
            // well at any zoom (the viewport mapping pushes it off-screen once you're
            // deep, since the orbit spans the whole |z|≲2 region). A faint wash keeps
            // the bright orbit legible while the fractal stays visible behind it.
            let mut min = pts[0];
            let mut max = pts[0];
            for &(x, y) in &pts {
                min.0 = min.0.min(x);
                min.1 = min.1.min(y);
                max.0 = max.0.max(x);
                max.1 = max.1.max(y);
            }
            let bw = (max.0 - min.0).max(1.0e-12);
            let bh = (max.1 - min.1).max(1.0e-12);
            let (bcx, bcy) = ((min.0 + max.0) * 0.5, (min.1 + max.1) * 0.5);
            painter.rect_filled(
                rect,
                egui::CornerRadius::ZERO,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 48),
            );
            painter.text(
                rect.min + egui::vec2(8.0, 6.0),
                egui::Align2::LEFT_TOP,
                "orbit (normalized)",
                egui::FontId::monospace(11.0),
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 140),
            );
            // Fit to the panel with an edge margin, preserving aspect ratio.
            let target = rect.shrink(28.0);
            let scale = (target.width() / bw as f32).min(target.height() / bh as f32);
            let center = rect.center();
            pts.iter()
                .map(|&(x, y)| {
                    egui::pos2(
                        center.x + ((x - bcx) as f32) * scale,
                        center.y - ((y - bcy) as f32) * scale, // flip y (screen +y down)
                    )
                })
                .collect()
        } else {
            let (cx, cy) = vp.center_f64();
            let upp = vp.units_per_pixel;
            pts.iter()
                .map(|p| {
                    let px = (p.0 - cx) / upp + vp.width_px * 0.5;
                    let py = vp.height_px * 0.5 - (p.1 - cy) / upp;
                    egui::pos2(rect.min.x + (px / ppp) as f32, rect.min.y + (py / ppp) as f32)
                })
                .collect()
        };
        let n = screen.len();
        // Per-segment polyline: thick & warm near z₀, tapering thinner and shifting
        // hue toward the tail so the direction of iteration reads at a glance.
        let start = egui::Color32::from_rgba_unmultiplied(0xFF, 0xF0, 0x60, 235); // warm yellow
        let end = egui::Color32::from_rgba_unmultiplied(0xFF, 0x30, 0x88, 120); // magenta, faded
        for i in 0..n - 1 {
            let t = i as f32 / (n - 1).max(1) as f32;
            let w = 3.4 * (1.0 - t) + 0.6; // 4.0 → 0.6 px
            painter.line_segment(
                [screen[i], screen[i + 1]],
                egui::Stroke::new(w, lerp_color(start, end, t)),
            );
        }
        painter.circle_filled(screen[0], 3.5, egui::Color32::from_rgb(0x40, 0xE0, 0x60)); // z₀
        painter.circle_filled(screen[n - 1], 3.0, egui::Color32::from_rgb(0xFF, 0x50, 0x40)); // last

        // A dot racing out along the orbit on a loop, color cycling over time.
        if self.orbit_anim && n >= 2 {
            let segs = (n - 1) as f32;
            let phase = self.orbit_phase % segs; // 0..segs, restarts at z₀
            let k = phase.floor() as usize;
            let f = phase - k as f32;
            let k2 = (k + 1).min(n - 1);
            let pos = screen[k] + (screen[k2] - screen[k]) * f;
            let col = egui::Color32::from(egui::ecolor::Hsva::new(self.orbit_hue, 0.85, 1.0, 1.0));
            let [r, g, b, _] = col.to_array();
            painter.circle_filled(pos, 8.0, egui::Color32::from_rgba_unmultiplied(r, g, b, 70));
            painter.circle_filled(pos, 4.0, col);
            painter.circle_stroke(pos, 4.0, egui::Stroke::new(1.0, egui::Color32::WHITE));
        }
    }

    /// Performance diagnostics, rendered into a docked panel section (FPS, CPU/GPU
    /// split, reference-recompute cost, and current render state).
    fn perf_section(&self, ui: &mut egui::Ui) {
        let p = &self.perf;
        let fps = if p.frame_ms > 0.0 { 1000.0 / p.frame_ms } else { 0.0 };
        let gpu_idle = (p.frame_ms - p.cpu_ms).max(0.0);
        let mode = match p.last_mode {
            1 => "direct df32",
            2 => "perturb floatexp",
            _ => "perturb df32",
        };
        ui.monospace(format!("FPS        {fps:6.1}"));
        ui.monospace(format!("frame      {:6.2} ms", p.frame_ms));
        ui.monospace(format!("cpu        {:6.2} ms", p.cpu_ms));
        ui.monospace(format!("gpu/idle   {gpu_idle:6.2} ms"));
        ui.separator();
        ui.monospace(format!("mode       {mode}"));
        ui.monospace(format!("eff iter   {:>7}", p.last_eff_iter));
        ui.monospace(format!("precision  {:>5} bit", p.last_precision));
        ui.monospace(format!("orbit len  {:>7}", p.last_orbit_len));
        ui.monospace(format!("aa         {}x", self.aa));
        ui.monospace(format!("dual       {}", self.dual));
        ui.monospace(format!("zoom       {}×", fmt_zoom(self.viewport.magnification())));

        // Julia parameter the Julia / dual view renders, plus how much c-space the
        // whole Mandelbrot panel covers. When "c/panel" drops far below one Julia
        // pixel (≈ Julia span ÷ panel width), hovering still updates c but the Julia
        // looks static — expected at deep Mandelbrot zoom, not a freeze.
        if self.dual || self.julia_mode {
            ui.separator();
            let (jr, ji) = self.julia_c;
            ui.monospace(format!("julia c.re {jr:+.15}"));
            ui.monospace(format!("julia c.im {ji:+.15}"));
            if self.julia_pin.is_some() {
                ui.monospace("julia c    pinned");
            }
            let c_per_panel = self.viewport.width_px * self.viewport.units_per_pixel;
            ui.monospace(format!("c/panel    {c_per_panel:.3e}"))
                .on_hover_text(
                    "Width of c-space spanned by the whole Mandelbrot panel. When this \
                     is far below one Julia pixel, hovering changes c but the Julia \
                     looks unchanged — expected at deep zoom, not a freeze.",
                );
        }

        ui.separator();
        ui.monospace(format!("ref recompute {:6.2} ms", p.recompute_ms));
        ui.monospace(format!("recompute/s   {:>4.0}", p.recompute_per_s));
        ui.monospace(format!("recompute tot {:>5}", p.recompute_total));
    }

    /// Reset the current view to the fractal's default (both panels in dual view).
    fn reset_view(&mut self) {
        self.viewport.reset();
        let (cx, cy) = self.fractal.default_center();
        self.viewport.center_x = fractadyne_core::BigFloat::from_f64(cx, 64);
        self.viewport.center_y = fractadyne_core::BigFloat::from_f64(cy, 64);
        if self.dual {
            self.julia_viewport.reset();
            self.julia_viewport.center_x = fractadyne_core::BigFloat::from_f64(0.0, 64);
            self.julia_viewport.center_y = fractadyne_core::BigFloat::from_f64(0.0, 64);
        }
        self.zoom_vel = 0.0;
        self.invalidate_refs();
    }

    /// Begin a smooth zoom-out back to the home view. If already at (or near) home,
    /// just snaps via `reset_view`. `now` is the current app time (`ctx.input.time`).
    fn zoom_home(&mut self, now: f64) {
        let m_logmag = self.viewport.magnification().max(1.0).ln();
        let j_logmag = if self.dual {
            self.julia_viewport.magnification().max(1.0).ln()
        } else {
            0.0
        };
        let deepest = m_logmag.max(j_logmag);
        if deepest < 0.02 {
            self.reset_view();
            return;
        }
        let duration =
            (deepest * HOME_SECONDS_PER_LOGMAG).clamp(HOME_MIN_SECONDS, HOME_MAX_SECONDS);
        self.home_anim = Some(HomeAnim {
            start_time: now,
            duration,
            m_start_center: (self.viewport.center_x.clone(), self.viewport.center_y.clone()),
            m_start_logmag: m_logmag,
            j_start_center: (
                self.julia_viewport.center_x.clone(),
                self.julia_viewport.center_y.clone(),
            ),
            j_start_logmag: j_logmag,
            dual: self.dual,
        });
        self.zoom_vel = 0.0;
    }

    /// Advance the active zoom-home animation by one frame. Returns true while it is
    /// still running (so the caller can keep requesting repaints).
    fn advance_home_anim(&mut self, ctx: &egui::Context) -> bool {
        let Some(anim) = self.home_anim.take() else {
            return false;
        };
        // Let the user grab control: any pan/zoom input cancels the glide in place.
        let interrupted = ctx.input(|i| {
            i.pointer.primary_down()
                || i.key_down(egui::Key::Space)
                || i.smooth_scroll_delta.y != 0.0
        });
        if interrupted {
            return false;
        }
        let now = ctx.input(|i| i.time);
        let u = ((now - anim.start_time) / anim.duration).clamp(0.0, 1.0);
        if u >= 1.0 {
            self.reset_view(); // exact home (center + zoom), invalidates references
            return false;
        }
        // Ease in/out (smoothstep) on the remaining log-magnification.
        let e = u * u * (3.0 - 2.0 * u);
        let remain = 1.0 - e;
        self.viewport
            .home_lerp(self.fractal.default_center(), &anim.m_start_center, anim.m_start_logmag * remain);
        if anim.dual {
            self.julia_viewport
                .home_lerp((0.0, 0.0), &anim.j_start_center, anim.j_start_logmag * remain);
        }
        // Treat the glide as interaction so AA stays off and references aren't
        // recomputed every frame (rebasing covers the motion; quality on settle).
        self.settle_t = now;
        self.home_anim = Some(anim);
        true
    }

    /// Stops uploaded to the GPU: the morphing random gradient when in Random mode,
    /// otherwise the selected preset.
    fn active_stops(&self) -> ([[f32; 4]; fractadyne_color::MAX_STOPS], u32) {
        if self.palette_anim == PaletteAnim::Random {
            self.random_palette.current()
        } else {
            fractadyne_color::PRESETS[self.palette_idx].packed()
        }
    }

    /// Advance the palette animation for this frame (offset shift, or random morph).
    fn advance_palette_anim(&mut self, ctx: &egui::Context) {
        let dt = (ctx.input(|i| i.stable_dt) as f64).clamp(0.0, 0.1) as f32;
        // Distance-estimate glow cycling — flows the contour bands (independent of the
        // palette animation; shares the Speed slider). Phase is in cycles, period 1.
        if self.de && self.de_anim && self.palette_anim_speed > 0.0 {
            self.de_phase = (self.de_phase + self.palette_anim_speed * dt).rem_euclid(1.0);
            self.schedule_repaint(ctx);
        }
        if self.palette_anim == PaletteAnim::Off || self.palette_anim_speed <= 0.0 {
            return;
        }
        let step = self.palette_anim_speed * dt;
        match self.palette_anim {
            PaletteAnim::Forward => self.offset = (self.offset + step).fract(),
            PaletteAnim::Reverse => self.offset = (self.offset - step).rem_euclid(1.0),
            PaletteAnim::PingPong => {
                self.offset += self.anim_dir * step;
                if self.offset >= 1.0 {
                    self.offset = 1.0;
                    self.anim_dir = -1.0;
                } else if self.offset <= 0.0 {
                    self.offset = 0.0;
                    self.anim_dir = 1.0;
                }
            }
            PaletteAnim::Random => self.random_palette.advance(dt, self.palette_anim_speed),
            PaletteAnim::Off => {}
        }
        self.schedule_repaint(ctx);
    }

    /// Start the built-in benchmark tour.
    fn start_benchmark(&mut self) {
        self.dual = false; // benchmark measures the single-view pipeline
        self.playback = Some(benchmark_playback());
    }

    /// Load a camera-tour script (TOML) via a file dialog and start playing it.
    fn load_script(&mut self) {
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
    fn advance_playback(&mut self, ctx: &egui::Context) -> bool {
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
    fn format_bench(&self, pb: &Playback, b: &Bench) -> String {
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

    /// Advance the orbit racing-dot animation (position along the path + hue).
    fn advance_orbit_anim(&mut self, ctx: &egui::Context) {
        if !(self.show_orbits && self.orbit_anim) {
            return;
        }
        let dt = (ctx.input(|i| i.stable_dt) as f64).clamp(0.0, 0.1) as f32;
        self.orbit_phase = (self.orbit_phase + self.orbit_anim_speed * dt) % 1.0e6;
        self.orbit_hue = (self.orbit_hue + 0.22 * dt).fract(); // ~4.5 s per color cycle
        self.schedule_repaint(ctx);
    }

    /// Zoom the main viewport about its center (factor < 1 zooms in).
    fn zoom_center(&mut self, factor: f64) {
        let (cx, cy) = (self.viewport.width_px * 0.5, self.viewport.height_px * 0.5);
        self.viewport.zoom_at(cx, cy, factor);
    }

    /// Toggle the dual linked view, framing the Julia panel when turning it on.
    fn toggle_dual(&mut self) {
        self.dual = !self.dual;
        if self.dual {
            self.julia_viewport.reset();
            self.julia_viewport.center_x = fractadyne_core::BigFloat::from_f64(0.0, 64);
            self.julia_viewport.center_y = fractadyne_core::BigFloat::from_f64(0.0, 64);
        }
        self.invalidate_refs();
    }
}

impl eframe::App for FractadyneApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let frame_start = Instant::now();
        // GPU handles for offline export (cloned Arcs; cheap).
        let gpu = frame
            .wgpu_render_state()
            .map(|rs| (rs.device.clone(), rs.queue.clone()));

        // Ctrl+S → quick export (no dialog) to the last folder.
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            if let Some((dev, q)) = &gpu {
                self.quick_export(dev.clone(), q.clone());
            }
        }

        // Esc: stop a playing tour first, otherwise leave fullscreen.
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.playback.is_some() {
                self.playback = None;
            } else if self.fullscreen {
                self.fullscreen = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
        }

        // CLI render-and-exit: render one image offscreen, save it, and quit.
        if self.auto_render && !self.auto_render_done {
            if let Some((dev, q)) = &gpu {
                self.auto_render_done = true;
                let out = self
                    .auto_render_out
                    .clone()
                    .unwrap_or_else(|| std::path::PathBuf::from("fractadyne_render.png"));
                match self.render_to_file(dev, q, &out) {
                    Ok(m) => println!("{m}"),
                    Err(e) => eprintln!("Render failed: {e}"),
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        // Scripted camera tour / benchmark: drive the view before anything renders.
        if self.playback.is_some() && self.advance_playback(ctx) {
            self.schedule_repaint(ctx);
        }

        // CLI auto-benchmark: once the tour has finished and produced a report, print
        // + save it and quit.
        if self.auto_benchmark && !self.auto_benchmark_done && self.playback.is_none() {
            if let Some(r) = self.bench_report.clone() {
                println!("{r}");
                let path = self
                    .auto_benchmark_out
                    .clone()
                    .unwrap_or_else(|| std::path::PathBuf::from("fractadyne_benchmark.txt"));
                match std::fs::write(&path, &r) {
                    Ok(()) => println!("\nSaved benchmark → {}", path.display()),
                    Err(e) => eprintln!("Failed to save benchmark to {}: {e}", path.display()),
                }
                self.auto_benchmark_done = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        // Smooth "zoom home" glide (Home button) — advance before anything draws so
        // this frame reflects the new view; keep repainting until it finishes.
        if self.home_anim.is_some() && self.advance_home_anim(ctx) {
            self.schedule_repaint(ctx);
        }

        // Palette cycling animation (shifts the color offset over time).
        self.advance_palette_anim(ctx);
        // Orbit racing-dot animation.
        self.advance_orbit_anim(ctx);

        // Poll a background export for completion.
        if let Some(rx) = &self.export_task {
            match rx.try_recv() {
                Ok(msg) => {
                    self.export_status = Some(msg);
                    self.export_task = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => ctx.request_repaint(),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => self.export_task = None,
            }
        }
        if let Some(prev) = self.perf.last_frame {
            let dt = frame_start.duration_since(prev).as_secs_f64() * 1000.0;
            self.perf.frame_ms = ema(self.perf.frame_ms, dt);
        }
        self.perf.last_frame = Some(frame_start);

        if !self.auto_benchmark && !self.auto_render {
            self.autosave(ctx); // don't let a CLI run overwrite the saved session
        }

        // Combined menu bar + action toolbar. `horizontal_wrapped` keeps them on one
        // line when the window is wide enough, and wraps the toolbar below otherwise.
        // (We place the menu buttons directly in the wrapped row rather than via
        // `menu::bar`, which would claim the full width and push the toolbar down.)
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("📂  Open view…").clicked() {
                            self.open_view();
                            ui.close_menu();
                        }
                        if ui.button("🖼  Gallery…").clicked() {
                            self.gallery_open = true;
                            self.scan_gallery();
                            ui.close_menu();
                        }
                        if ui.button("💾  Export image…").clicked() {
                            self.export_open = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("✖  Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.menu_button("Fractal", |ui| {
                        for kind in FractalKind::ALL {
                            if ui
                                .selectable_label(self.fractal == kind, kind.name())
                                .clicked()
                            {
                                self.set_fractal(kind);
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        let can_julia = self.fractal.supports_julia();
                        ui.add_enabled_ui(can_julia, |ui| {
                            if ui
                                .checkbox(&mut self.julia_mode, "Julia mode")
                                .on_hover_text(
                                    "Show the Julia set of this formula for the current c.",
                                )
                                .changed()
                            {
                                self.invalidate_refs();
                            }
                        });
                    });
                    ui.menu_button("View", |ui| {
                        if ui
                            .checkbox(&mut self.dual, "Dual view (Mandelbrot ↔ Julia)")
                            .clicked()
                            && self.dual
                        {
                            self.julia_viewport.reset();
                            self.julia_viewport.center_x =
                                fractadyne_core::BigFloat::from_f64(0.0, 64);
                            self.julia_viewport.center_y =
                                fractadyne_core::BigFloat::from_f64(0.0, 64);
                        }
                        ui.checkbox(&mut self.perf.enabled, "Performance panel");
                        ui.checkbox(&mut self.show_orbits, "Show orbits")
                            .on_hover_text(
                                "Draw the iteration path of the point under the cursor.",
                            );
                        ui.add_enabled_ui(self.show_orbits, |ui| {
                            ui.checkbox(&mut self.orbit_normalize, "    Normalize (fit to view)")
                                .on_hover_text(
                                    "Fit the orbit to the whole view so it stays well-framed \
                                     at any zoom (instead of mapped through the viewport, \
                                     where it flies off-screen when deep).",
                                );
                            ui.checkbox(&mut self.orbit_anim, "    Animate (racing dot)")
                                .on_hover_text(
                                    "Send a color-cycling dot racing out along the orbit.",
                                );
                            ui.add_enabled(
                                self.orbit_anim,
                                egui::Slider::new(&mut self.orbit_anim_speed, 1.0..=40.0)
                                    .text("Orbit speed")
                                    .suffix("/s"),
                            );
                        });
                        ui.separator();
                        ui.label("Frame-rate cap");
                        for (label, val) in [
                            ("Uncapped", None),
                            ("30 FPS", Some(30.0)),
                            ("60 FPS", Some(60.0)),
                            ("120 FPS", Some(120.0)),
                        ] {
                            if ui.selectable_label(self.fps_cap == val, label).clicked() {
                                self.fps_cap = val;
                            }
                        }
                    });
                    ui.menu_button("Tools", |ui| {
                        if ui
                            .button("Run benchmark")
                            .on_hover_text(
                                "Play a fixed deep-zoom tour and report FPS / CPU / GPU / RAM.",
                            )
                            .clicked()
                        {
                            self.start_benchmark();
                            ui.close_menu();
                        }
                        if ui.button("Play script…").clicked() {
                            self.load_script();
                            ui.close_menu();
                        }
                        if self.bench_report.is_some()
                            && ui.button("Show last benchmark").clicked()
                        {
                            self.bench_open = true;
                            ui.close_menu();
                        }
                        ui.add_enabled_ui(self.playback.is_some(), |ui| {
                            if ui.button("Stop playback").clicked() {
                                self.playback = None;
                                ui.close_menu();
                            }
                        });
                    });
                    ui.menu_button("Bookmarks", |ui| {
                        if ui.button("★  Add current view").clicked() {
                            self.add_bookmark("");
                            ui.close_menu();
                        }
                        if ui.button("Manage…").clicked() {
                            self.bookmarks_open = true;
                            ui.close_menu();
                        }
                        if !self.bookmarks.is_empty() {
                            ui.separator();
                            // Click to jump (most recent first).
                            let mut jump: Option<usize> = None;
                            for (i, b) in self.bookmarks.iter().enumerate().rev().take(12) {
                                if ui.button(&b.name).clicked() {
                                    jump = Some(i);
                                }
                            }
                            if let Some(i) = jump {
                                let meta = self.bookmarks[i].meta.clone();
                                self.load_view_metadata(&meta);
                                ui.close_menu();
                            }
                        }
                    });
                    ui.menu_button("Help", |ui| {
                        ui.label(format!("Fractadyne v{}", version_string()));
                        ui.label(egui::RichText::new("Native fractal explorer").weak().small());
                        ui.separator();
                        ui.label(egui::RichText::new("License").weak().small());
                        ui.label("MIT OR Apache-2.0");
                        ui.label(
                            egui::RichText::new("© 2026 Rithea Hong. Dual-licensed; use \
                                 under either license at your option.")
                                .weak()
                                .small(),
                        );
                        ui.hyperlink_to(
                            "Source \u{2197}",
                            "https://github.com/WindySnowOwl/fractadyne",
                        );
                    });

                ui.separator();

                // Fractal picker (the name, as a dropdown).
                let prev = self.fractal;
                let mut sel = self.fractal;
                egui::ComboBox::from_id_salt("fractal_dropdown")
                    .selected_text(self.fractal.name())
                    .show_ui(ui, |ui| {
                        for k in FractalKind::ALL {
                            ui.selectable_value(&mut sel, k, k.name());
                        }
                    });
                if sel != prev {
                    self.set_fractal(sel);
                }
                ui.separator();
                ui.add_enabled_ui(self.fractal.supports_julia() && !self.dual, |ui| {
                    if ui
                        .selectable_label(self.julia_mode, "Julia")
                        .on_hover_text("Show the Julia set of this formula")
                        .clicked()
                    {
                        self.julia_mode = !self.julia_mode;
                        self.invalidate_refs();
                    }
                });
                if dual_toggle_button(ui, self.dual)
                    .on_hover_text("Dual linked view (Mandelbrot ↔ Julia)")
                    .clicked()
                {
                    self.toggle_dual();
                }
                ui.separator();
                if ui.button("💾").on_hover_text("Export image…").clicked() {
                    self.export_open = true;
                }
                if ui.button("🖼").on_hover_text("Gallery").clicked() {
                    self.gallery_open = true;
                    self.scan_gallery();
                }
                if ui.button("📂").on_hover_text("Open view…").clicked() {
                    self.open_view();
                }
                if ui
                    .button("★")
                    .on_hover_text("Bookmark this view")
                    .clicked()
                {
                    self.add_bookmark("");
                }
                if ui.button("🔄").on_hover_text("Reset view (instant)").clicked() {
                    self.reset_view();
                }
                if ui
                    .button("🏠")
                    .on_hover_text("Zoom out to the home view (animated)")
                    .clicked()
                {
                    let now = ctx.input(|i| i.time);
                    self.zoom_home(now);
                }
                ui.separator();
                if ui
                    .button("📷")
                    .on_hover_text("Snapshot — quick export to the last folder (Ctrl+S)")
                    .clicked()
                {
                    if let Some((dev, q)) = &gpu {
                        self.quick_export(dev.clone(), q.clone());
                    }
                }
                if ui.button("🔍+").on_hover_text("Zoom in").clicked() {
                    self.zoom_center(0.5);
                }
                if ui.button("🔍−").on_hover_text("Zoom out").clicked() {
                    self.zoom_center(2.0);
                }
                if ui
                    .button("🎨")
                    .on_hover_text(format!(
                        "Next palette ({})",
                        fractadyne_color::PRESETS[self.palette_idx].name
                    ))
                    .clicked()
                {
                    self.palette_idx = (self.palette_idx + 1) % fractadyne_color::PRESETS.len();
                }
                ui.separator();
                if ui
                    .selectable_label(self.perf.enabled, "📊")
                    .on_hover_text("Performance panel")
                    .clicked()
                {
                    self.perf.enabled = !self.perf.enabled;
                }
                if ui
                    .selectable_label(self.fullscreen, "🖥")
                    .on_hover_text("Toggle fullscreen")
                    .clicked()
                {
                    self.fullscreen = !self.fullscreen;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
                }
            });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let (cf_x, cf_y) = self.viewport.center_f64();
                ui.monospace(format!("center {}, {}", fmt_coord(cf_x), fmt_coord(cf_y)));
                ui.separator();
                match self.pointer_complex {
                    Some((mx, my)) => {
                        ui.monospace(format!("cursor {}, {}", fmt_coord(mx), fmt_coord(my)))
                    }
                    None => ui.monospace("cursor —"),
                };
                ui.separator();
                if self.dual {
                    ui.monospace(format!(
                        "zoom  M {}×   J {}×",
                        fmt_zoom(self.viewport.magnification()),
                        fmt_zoom(self.julia_viewport.magnification()),
                    ));
                } else {
                    ui.monospace(format!("zoom {}×", fmt_zoom(self.viewport.magnification())));
                }
                ui.separator();
                let eff_iter = if self.auto_iter {
                    self.viewport.recommended_max_iter(self.max_iter)
                } else {
                    self.max_iter
                };
                ui.monospace(format!("iter {}", commas(&eff_iter.to_string())));
                if let Some(pb) = &self.playback {
                    let elapsed = pb.t0.map_or(0.0, |t0| ctx.input(|i| i.time) - t0);
                    let pct = if pb.total > 0.0 {
                        (elapsed / pb.total * 100.0).clamp(0.0, 100.0)
                    } else {
                        100.0
                    };
                    ui.separator();
                    let tag = if pb.bench.is_some() { "benchmark" } else { "script" };
                    ui.monospace(format!("▶ {} {tag} {pct:.0}%", pb.name));
                }
            });
        });

        // Right-hand control panel: fractal info, coloring, navigation, and the
        // optional performance section. Hidden entirely while in fullscreen.
        if !self.fullscreen {
        egui::SidePanel::right("coloring_panel")
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.add_space(6.0);
                // Per-fractal info: formula, background, reference link.
                let info = self.fractal.info();
                egui::CollapsingHeader::new(self.fractal.name())
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.monospace(info.formula);
                        ui.add_space(4.0);
                        ui.label(info.about);
                        ui.add_space(4.0);
                        ui.hyperlink_to("Reference \u{2197}", info.reference);
                    });
                ui.separator();

                ui.label(egui::RichText::new("COLORING").weak().small());
                ui.add_space(4.0);
                egui::ComboBox::from_label("Palette")
                    .selected_text(fractadyne_color::PRESETS[self.palette_idx].name)
                    .show_ui(ui, |ui| {
                        for (i, p) in fractadyne_color::PRESETS.iter().enumerate() {
                            ui.selectable_value(&mut self.palette_idx, i, p.name);
                        }
                    });
                ui.add(egui::Slider::new(&mut self.cycle, 0.0..=1.0).text("Cycle"));
                ui.add(egui::Slider::new(&mut self.offset, 0.0..=1.0).text("Offset"));
                egui::ComboBox::from_label("Animate")
                    .selected_text(self.palette_anim.name())
                    .show_ui(ui, |ui| {
                        for m in PaletteAnim::ALL {
                            ui.selectable_value(&mut self.palette_anim, m, m.name());
                        }
                    });
                ui.add_enabled(
                    self.palette_anim != PaletteAnim::Off,
                    egui::Slider::new(&mut self.palette_anim_speed, 0.01..=2.0)
                        .text("Speed")
                        .suffix("/s")
                        .logarithmic(true),
                )
                .on_hover_text(
                    "Cycle speed: color-offset cycles/sec, or (Random) gradient \
                     changes/sec.",
                );
                if self.palette_anim == PaletteAnim::Random && ui.button("Shuffle gradient").clicked()
                {
                    self.random_palette.reshuffle();
                }
                ui.separator();
                ui.checkbox(&mut self.light, "3D relief lighting")
                    .on_hover_text(
                        "Shade the surface using the distance-estimate slope — an \
                         embossed, lit look. (Holomorphic families: Mandelbrot / \
                         Multibrot.)",
                    );
                ui.add_enabled_ui(self.light, |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.light_angle, 0.0..=std::f32::consts::TAU)
                            .text("Light angle")
                            .suffix(" rad"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.light_height, 0.2..=4.0)
                            .text("Relief")
                            .logarithmic(true),
                    )
                    .on_hover_text("Lower = sharper relief; higher = softer/flatter.");
                });
                ui.checkbox(&mut self.de, "Distance glow")
                    .on_hover_text(
                        "Bright distance-estimate contour bands that densify into glowing \
                         filaments near the boundary. (Holomorphic families.)",
                    );
                ui.add_enabled_ui(self.de, |ui| {
                    ui.add(egui::Slider::new(&mut self.de_strength, 0.0..=1.0).text("Glow"));
                    ui.add(
                        egui::Slider::new(&mut self.de_width, 0.15..=4.0)
                            .text("Band width")
                            .logarithmic(true),
                    )
                    .on_hover_text("Spacing of the distance contours (octaves per band).");
                    ui.checkbox(&mut self.de_anim, "Animate glow")
                        .on_hover_text("Flow the glow bands over time (uses the Speed slider).");
                });
                ui.separator();
                ui.checkbox(&mut self.auto_iter, "Auto-scale iterations with zoom");
                let label = if self.auto_iter { "Iterations (base)" } else { "Iterations" };
                ui.add(
                    egui::Slider::new(&mut self.max_iter, 64..=50_000)
                        .logarithmic(true)
                        .text(label),
                )
                .on_hover_text(
                    "Base iteration count. With Auto-scale on, the effective count \
                     climbs with zoom depth (up to ~50,000).",
                );
                ui.separator();
                egui::ComboBox::from_label("Anti-alias")
                    .selected_text(match self.aa {
                        1 => "Off",
                        2 => "2×",
                        3 => "3×",
                        4 => "4×",
                        _ => "8×",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.aa, 1, "Off");
                        ui.selectable_value(&mut self.aa, 2, "2×");
                        ui.selectable_value(&mut self.aa, 3, "3×");
                        ui.selectable_value(&mut self.aa, 4, "4×");
                        ui.selectable_value(&mut self.aa, 8, "8×");
                    })
                    .response
                    .on_hover_text(
                        "Supersampling for still images (applied when the view settles). \
                         Higher tames the fine exterior 'dust' at the cost of render time.",
                    );

                ui.add_space(12.0);
                ui.label(egui::RichText::new("NAVIGATION").weak().small());
                ui.add_space(4.0);
                ui.add(
                    egui::Slider::new(&mut self.zoom_rate, 0.25..=4.0)
                        .text("Zoom speed")
                        .suffix("×")
                        .logarithmic(true),
                )
                .on_hover_text("Speed of hold-Space continuous zoom (1× ≈ 2× per 1.5 s).");

                // Performance section, docked at the bottom of this same panel
                // (toggle via the Perf button or the View menu).
                if self.perf.enabled {
                    ui.add_space(12.0);
                    ui.separator();
                    ui.label(egui::RichText::new("PERFORMANCE").weak().small());
                    ui.add_space(4.0);
                    self.perf_section(ui);
                }
            });
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                if self.dual {
                    self.draw_dual(ui, ctx);
                    return;
                }
                let rect = ui.max_rect();
                let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
                let ppp = ctx.pixels_per_point() as f64;

                // 1 pixel = constant complex units; resizing reveals more/less plane.
                self.viewport
                    .set_size(rect.width() as f64 * ppp, rect.height() as f64 * ppp);

                // Track the complex coordinate under the cursor (for the status bar).
                self.pointer_complex = response.hover_pos().map(|p| {
                    let l = p - rect.min;
                    self.viewport
                        .complex_at_pixel_f64(l.x as f64 * ppp, l.y as f64 * ppp)
                });

                // Pan with left-drag.
                if response.dragged_by(egui::PointerButton::Primary) {
                    let d = response.drag_delta();
                    self.viewport.pan_pixels(d.x as f64 * ppp, d.y as f64 * ppp);
                }

                // Cursor-centered wheel zoom (scroll up = zoom in).
                let scroll_y = ctx.input(|i| i.smooth_scroll_delta.y) as f64;
                if scroll_y != 0.0 {
                    if let Some(pos) = response.hover_pos() {
                        let local = pos - rect.min;
                        let factor = (-0.0015 * scroll_y).exp();
                        self.viewport
                            .zoom_at(local.x as f64 * ppp, local.y as f64 * ppp, factor);
                    }
                }

                // Continuous zoom: hold Space (in) / Shift+Space (out), toward the
                // cursor. Exponential rate, eased in/out, frame-rate independent.
                if let Some(pos) = response.hover_pos() {
                    self.last_cursor = Some(pos);
                }
                let (space, shift) =
                    ctx.input(|i| (i.key_down(egui::Key::Space), i.modifiers.shift));
                let rate = ZOOM_RATE * self.zoom_rate as f64;
                let target_vel = if space {
                    if shift { -rate } else { rate }
                } else {
                    0.0
                };
                let dt = (ctx.input(|i| i.stable_dt) as f64).clamp(0.0, 0.1);
                let ease = 1.0 - (-dt / EASE_TAU).exp();
                self.zoom_vel += (target_vel - self.zoom_vel) * ease;
                if target_vel != 0.0 || self.zoom_vel.abs() > 1e-3 {
                    self.schedule_repaint(ctx); // animate while held and during glide-out
                }
                if self.zoom_vel.abs() > 1e-3 {
                    let anchor = self.last_cursor.unwrap_or_else(|| rect.center());
                    let local = anchor - rect.min;
                    let factor = (-self.zoom_vel * dt).exp(); // vel>0 → factor<1 → zoom in
                    self.viewport
                        .zoom_at(local.x as f64 * ppp, local.y as f64 * ppp, factor);
                }

                // Box-zoom with right-drag: record the start, apply on release.
                if response.drag_started_by(egui::PointerButton::Secondary) {
                    self.box_start = response.interact_pointer_pos();
                }
                if response.drag_stopped_by(egui::PointerButton::Secondary) {
                    let end = response
                        .interact_pointer_pos()
                        .or_else(|| ctx.input(|i| i.pointer.latest_pos()));
                    if let (Some(start), Some(end)) = (self.box_start, end) {
                        let s = start - rect.min;
                        let e = end - rect.min;
                        if (e.x - s.x).abs() > 6.0 && (e.y - s.y).abs() > 6.0 {
                            self.viewport.zoom_to_rect(
                                s.x as f64 * ppp,
                                s.y as f64 * ppp,
                                e.x as f64 * ppp,
                                e.y as f64 * ppp,
                            );
                        }
                    }
                    self.box_start = None;
                }

                // Render the fractal at the current viewport with the chosen palette.
                let (span_x, span_y) = self.viewport.complex_span();
                let eff_iter = if self.auto_iter {
                    self.viewport.recommended_max_iter(self.max_iter)
                } else {
                    self.max_iter
                };
                // Quality-on-settle: skip AA while interacting (and for a short
                // settle window after), then render full AA once the view is still.
                let now = ctx.input(|i| i.time);
                let active = self.zoom_vel.abs() > 1e-3
                    || response.dragged()
                    || self.box_start.is_some()
                    || scroll_y != 0.0
                    || space;
                if active {
                    self.settle_t = now;
                }
                let interacting = now - self.settle_t < SETTLE_DELAY;

                let center_bf = [self.viewport.center_x.clone(), self.viewport.center_y.clone()];
                let center = self.viewport.center_f64();
                let mag = self.viewport.magnification();
                let resolution = [
                    (rect.width() as f64 * ppp) as u32,
                    (rect.height() as f64 * ppp) as u32,
                ];
                let params = self.build_params(
                    center_bf,
                    center,
                    (span_x, span_y),
                    mag,
                    self.fractal,
                    self.julia_mode,
                    eff_iter,
                    interacting,
                    resolution,
                    0,
                );
                add_mandelbrot(ui.painter(), rect, params);

                // Orbit overlay for the point under the cursor.
                if self.show_orbits {
                    if let Some(hp) = response.hover_pos() {
                        let l = hp - rect.min;
                        let cpx = (l.x as f64 * ppp, l.y as f64 * ppp);
                        let painter = ui.painter_at(rect);
                        self.draw_orbit(&painter, rect, &self.viewport, cpx, self.julia_mode, ppp);
                    }
                }

                // Draw the in-progress box-zoom selection on top of the fractal.
                if let Some(start) = self.box_start {
                    if response.dragged_by(egui::PointerButton::Secondary) {
                        if let Some(cur) = response.interact_pointer_pos() {
                            let sel = egui::Rect::from_two_pos(start, cur);
                            let painter = ui.painter();
                            painter.rect_filled(
                                sel,
                                egui::CornerRadius::ZERO,
                                egui::Color32::from_rgba_unmultiplied(0xE0, 0xA0, 0x30, 28),
                            );
                            painter.rect_stroke(
                                sel,
                                egui::CornerRadius::ZERO,
                                egui::Stroke::new(1.5, egui::Color32::from_rgb(0xE0, 0xA0, 0x30)),
                                egui::StrokeKind::Inside,
                            );
                        }
                    }
                }
            });

        // ---- bookmarks manager ----
        if self.bookmarks_open {
            let mut open = self.bookmarks_open;
            let mut jump: Option<usize> = None;
            let mut delete: Option<usize> = None;
            let mut changed = false;
            egui::Window::new("Bookmarks")
                .open(&mut open)
                .default_size([420.0, 460.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.bookmark_name)
                                .hint_text("name (optional)")
                                .desired_width(240.0),
                        );
                        if ui.button("★ Add current view").clicked() {
                            let name = self.bookmark_name.clone();
                            self.add_bookmark(&name);
                            self.bookmark_name.clear();
                        }
                    });
                    ui.separator();
                    if self.bookmarks.is_empty() {
                        ui.label("No bookmarks yet. Add one from the current view.");
                    }
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (i, b) in self.bookmarks.iter().enumerate() {
                            ui.horizontal(|ui| {
                                if ui.button("Go").clicked() {
                                    jump = Some(i);
                                }
                                if ui.button("🗑").on_hover_text("Delete").clicked() {
                                    delete = Some(i);
                                }
                                let zoom = meta_get(&b.meta, "zoom");
                                ui.label(&b.name);
                                if !zoom.is_empty() {
                                    ui.label(egui::RichText::new(format!("{zoom}×")).weak().small());
                                }
                            });
                        }
                    });
                });
            if let Some(i) = jump {
                let meta = self.bookmarks[i].meta.clone();
                self.load_view_metadata(&meta);
            }
            if let Some(i) = delete {
                self.bookmarks.remove(i);
                changed = true;
            }
            if changed {
                self.save_bookmarks();
            }
            self.bookmarks_open = open;
        }

        // ---- benchmark results ----
        if self.bench_open {
            let mut open = self.bench_open;
            egui::Window::new("Benchmark results")
                .open(&mut open)
                .resizable(false)
                .show(ctx, |ui| {
                    if let Some(r) = self.bench_report.clone() {
                        ui.monospace(&r);
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            if ui.button("Copy").clicked() {
                                ui.ctx().copy_text(r.clone());
                            }
                            if ui.button("Save…").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Text", &["txt"])
                                    .set_file_name("fractadyne_benchmark.txt")
                                    .save_file()
                                {
                                    let _ = std::fs::write(path, &r);
                                }
                            }
                        });
                    } else {
                        ui.label("No benchmark has been run yet.");
                    }
                });
            self.bench_open = open;
        }

        // ---- gallery browser ----
        if self.gallery_open {
            let mut open = self.gallery_open;
            let mut to_open: Option<String> = None;
            let mut do_rescan = false;
            egui::Window::new("Gallery")
                .open(&mut open)
                .default_size([540.0, 620.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Folder…").clicked() {
                            if let Some(d) = rfd::FileDialog::new()
                                .set_directory(&self.gallery_dir)
                                .pick_folder()
                            {
                                self.gallery_dir = d;
                                do_rescan = true;
                            }
                        }
                        if ui.button("Refresh").clicked() {
                            do_rescan = true;
                        }
                        ui.label(
                            egui::RichText::new(self.gallery_dir.display().to_string())
                                .weak()
                                .small(),
                        );
                    });
                    ui.separator();
                    if self.gallery_entries.is_empty() {
                        ui.label("No Fractadyne images in this folder.");
                    }
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for entry in &self.gallery_entries {
                            ui.horizontal(|ui| {
                                match &entry.thumb {
                                    Some(t) => {
                                        let s = t.size();
                                        ui.add(egui::Image::new(egui::load::SizedTexture::new(
                                            t.id(),
                                            egui::vec2(s[0] as f32, s[1] as f32),
                                        )));
                                    }
                                    None => {
                                        ui.add_sized(
                                            [160.0, 100.0],
                                            egui::Label::new(
                                                egui::RichText::new("…").weak(),
                                            ),
                                        );
                                    }
                                }
                                ui.vertical(|ui| {
                                    let title = if entry.fractal.is_empty() {
                                        "Fractadyne image"
                                    } else {
                                        &entry.fractal
                                    };
                                    ui.strong(title);
                                    ui.label(format!("zoom {}   {}", entry.zoom, entry.saved));
                                    if !entry.notes.is_empty() {
                                        ui.label(format!("\u{201c}{}\u{201d}", entry.notes));
                                    }
                                    ui.label(
                                        egui::RichText::new(&entry.app_version).weak().small(),
                                    );
                                    if ui.button("Open this view").clicked() {
                                        to_open = Some(entry.meta.clone());
                                    }
                                });
                            });
                            ui.separator();
                        }
                    });
                });
            self.gallery_open = open;
            if do_rescan {
                self.scan_gallery();
            }
            if let Some(meta) = to_open {
                self.load_view_metadata(&meta);
                self.export_status = Some("Loaded view from gallery.".to_string());
            }
            // Lazily decode one thumbnail per frame so scanning a folder never freezes.
            if let Some(entry) = self.gallery_entries.iter_mut().find(|e| !e.thumb_tried) {
                entry.thumb_tried = true;
                if let Some((tw, th, rgba)) = fractadyne_export::read_thumbnail(&entry.path, 160) {
                    let img = egui::ColorImage::from_rgba_unmultiplied(
                        [tw as usize, th as usize],
                        &rgba,
                    );
                    let name = format!("thumb:{}", entry.path.display());
                    entry.thumb =
                        Some(ctx.load_texture(name, img, egui::TextureOptions::LINEAR));
                }
                ctx.request_repaint();
            }
        }

        // ---- export dialog ----
        if self.export_open {
            let mut open = self.export_open;
            let mut do_export = false;
            egui::Window::new("Export image")
                .open(&mut open)
                .resizable(false)
                .show(ctx, |ui| {
                    egui::ComboBox::from_label("Width (px)")
                        .selected_text(format!("{}", self.export_width))
                        .show_ui(ui, |ui| {
                            for w in [1280u32, 1920, 2560, 3840, 5120, 7680] {
                                ui.selectable_value(&mut self.export_width, w, format!("{w}"));
                            }
                        });
                    egui::ComboBox::from_label("Supersampling")
                        .selected_text(format!("{}×", self.export_ss))
                        .show_ui(ui, |ui| {
                            for s in [1u32, 2, 3, 4] {
                                ui.selectable_value(&mut self.export_ss, s, format!("{s}×"));
                            }
                        });
                    ui.horizontal(|ui| {
                        ui.label("Format:");
                        ui.radio_value(&mut self.export_format, ExportFormat::Png, "PNG");
                        ui.radio_value(&mut self.export_format, ExportFormat::Exr, "OpenEXR");
                    });
                    if self.dual {
                        egui::ComboBox::from_label("Dual layout")
                            .selected_text(match self.export_dual_mode {
                                DualExport::SideBySide => "Side by side",
                                DualExport::Separate => "Separate files",
                                DualExport::ActiveOnly => "Map only",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.export_dual_mode,
                                    DualExport::SideBySide,
                                    "Side by side",
                                );
                                ui.selectable_value(
                                    &mut self.export_dual_mode,
                                    DualExport::Separate,
                                    "Separate files",
                                );
                                ui.selectable_value(
                                    &mut self.export_dual_mode,
                                    DualExport::ActiveOnly,
                                    "Map only",
                                );
                            });
                    }
                    ui.horizontal(|ui| {
                        ui.label("Notes:");
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut self.export_notes)
                                .char_limit(120)
                                .desired_width(220.0)
                                .hint_text("saved with the image (≤120 chars)"),
                        );
                        if resp.changed() && self.export_notes.chars().count() > 120 {
                            self.export_notes = self.export_notes.chars().take(120).collect();
                        }
                    });
                    let (sx, sy) = self.viewport.complex_span();
                    let h = ((self.export_width as f64) * sy / sx).round().max(1.0) as u32;
                    ui.label(format!(
                        "Output: {} × {} px   ({} chars left)",
                        self.export_width,
                        h,
                        120usize.saturating_sub(self.export_notes.chars().count()),
                    ));
                    ui.label(
                        egui::RichText::new(
                            "Rendered in tiles (no size cap) on a background thread. The \
                             file embeds the view + notes so it can be reopened via \
                             File ▸ Open view.",
                        )
                        .weak()
                        .small(),
                    );
                    ui.add_space(6.0);
                    let busy = self.export_task.is_some();
                    if busy {
                        let p = self.export_progress.load(std::sync::atomic::Ordering::Relaxed);
                        if p >= 2000 {
                            // Rendering done; encoding/writing the file (not cancelable).
                            ui.horizontal(|ui| {
                                ui.add(egui::Spinner::new());
                                ui.label("Saving…");
                            });
                        } else {
                            ui.label("Rendering…");
                            ui.add(
                                egui::ProgressBar::new(p as f32 / 1000.0).show_percentage(),
                            );
                            if ui.button("Cancel").clicked() {
                                self.export_cancel
                                    .store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                    } else if ui.button("Export").clicked() {
                        do_export = true;
                    }
                    if let Some(s) = &self.export_status {
                        ui.add_space(6.0);
                        ui.label(s);
                    }
                });
            self.export_open = open;
            if do_export {
                if let Some((dev, q)) = &gpu {
                    self.start_export(dev.clone(), q.clone());
                } else {
                    self.export_status = Some("GPU not available".to_string());
                }
            }
        }

        // ---- performance overlay + frame timing finalization ----
        let nowi = Instant::now();
        match self.perf.rate_t0 {
            Some(t0) if nowi.duration_since(t0).as_secs_f64() >= 1.0 => {
                self.perf.recompute_per_s = self.perf.rate_count as f32;
                self.perf.rate_count = 0;
                self.perf.rate_t0 = Some(nowi);
            }
            None => self.perf.rate_t0 = Some(nowi),
            _ => {}
        }
        if self.perf.enabled {
            self.schedule_repaint(ctx); // keep metrics live while the panel is shown
        }
        self.perf.cpu_ms = ema(self.perf.cpu_ms, frame_start.elapsed().as_secs_f64() * 1000.0);

        // Frame-rate cap: pace the main thread so we don't render faster than the cap
        // (paired with vsync this snaps to a clean sub-rate, e.g. 60 on a 120 Hz panel).
        if let Some(cap) = self.fps_cap {
            if cap > 0.0 {
                let target = 1.0 / cap;
                let spent = frame_start.elapsed().as_secs_f64();
                if spent < target {
                    std::thread::sleep(std::time::Duration::from_secs_f64(target - spent));
                }
            }
        }
    }
}
