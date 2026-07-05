//! Fractadyne — native fractal explorer (the desktop binary).
//!
//! # Where things live (crate layering)
//!
//! The workspace separates the logical layers into crates (so the numerics and GPU code are
//! reusable and testable independently of the UI):
//!
//! - `fractadyne-core`  — numerics: arbitrary-precision [`Viewport`], reference orbits,
//!   perturbation/`FloatExp` scale, series approximation, minibrot finder, parsers.
//! - `fractadyne-gpu`   — wgpu pipelines + the WGSL shader (`mandelbrot.wgsl`); the live
//!   paint callback and the offscreen `render_export` / `render_iter`.
//! - `fractadyne-color` — palette presets / gradient sampling.
//! - `fractadyne-state` — session persistence (TOML).
//! - `fractadyne-export`— PNG / OpenEXR encode + decode.
//! - `fractadyne-app`   — *this crate*: the window, input, egui UI, and the glue that
//!   drives the above. The `fractadyne` binary.
//!
//! # This crate's modules
//!
//! Extracted modules (cohesive units lifted out to keep `main.rs` navigable):
//! - [`cli`]      — headless CLI modes (`--find-minibrot`, `--compare`, `--crosscheck-f3`,
//!   `--validate-deep`), so `fn main` is just dispatch + window setup.
//! - [`render`]   — the render-request builders: reference orbit, series-skip, mode select,
//!   `MandelbrotParams`/`ExportRequest` assembly (the performance-critical bridge to the GPU).
//! - [`export`]   — render-to-file (PNG/EXR), the reloadable view-metadata blob, Open-view.
//! - [`autopilot`]— the hands-free auto-zoom dive.
//! - [`fractal`]  — the [`FractalKind`] domain enum (families, formula ids, descriptions).
//! - [`help`]     — the in-app Help window content.
//! - [`theme`]    — branding colors, dark visuals, wordmark, window icon.
//! - [`sysinfo`]  — version string, UTC formatter, CPU/VRAM/RAM probes.
//! - [`selftest`] — the `--selftest` GPU validation suite (render-path cross-checks + goldens).
//! - [`profile`]  — the `--profile` development profiling harness (benchmark regions → logs).
//!
//! `main.rs` itself now holds: small shared types/helpers (`Perf`, `RandomPalette`,
//! formatting, tunables); `fn main` (CLI dispatch + window setup); and [`FractadyneApp`] —
//! all app state plus the remaining behavior (UI panels/dialogs/toolbar/menus, bookmarks,
//! goto/locations, gallery, scripting, minimap, coloring) and the `eframe::App` `update`.
//!
//! Modularization is ongoing — the bulk left is the UI (the large `update` method and the
//! dialog/panel renderers); navigation, scripting, and coloring are also candidates. Moving
//! items between modules in one crate has no runtime cost (no added indirection; inlining is
//! unaffected) — purely an
//! organization/readability change.

use eframe::egui;
use fractadyne_core::Viewport;
use fractadyne_gpu::{add_mandelbrot, install_renderer};

mod autopilot;
mod cli;
mod error;
mod export;
mod fractal;
mod help;
mod profile;
mod render;
mod scripting;
mod selftest;
mod sysinfo;
mod theme;
pub(crate) use fractal::FractalKind;
pub(crate) use scripting::{BenchDepth, BenchRes, Playback, StdBench};
pub(crate) use sysinfo::*;
pub(crate) use theme::*;
use help::*;
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
    last_sa_skip: u32,
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
            last_sa_skip: 0,
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

/// Parse a `--size` value as either a bare width (`1920`) or `WIDTHxHEIGHT` (`5120x2160`,
/// case-insensitive `x`/`X`/`×`). Returns `(width, height)` where each is `Some` when present and
/// parseable. This lets callers accept both forms; a bare width leaves the height to `--height` or
/// an aspect-ratio default.
fn parse_size(s: &str) -> (Option<u32>, Option<u32>) {
    let sep = |c: char| c == 'x' || c == 'X' || c == '×';
    match s.split_once(sep) {
        Some((w, h)) => (w.trim().parse().ok(), h.trim().parse().ok()),
        None => (s.trim().parse().ok(), None),
    }
}

/// Tokenize an args (response) file: whitespace-separated tokens, honoring `"…"` / `'…'` quoting so
/// values with spaces survive, with `#` starting a comment to end of line (outside quotes). One
/// token per argument — the same as typing them on the command line.
fn tokenize_args_file(text: &str) -> Vec<String> {
    let mut toks = Vec::new();
    for line in text.lines() {
        let mut cur = String::new();
        let mut in_tok = false;
        let mut quote: Option<char> = None;
        for c in line.chars() {
            match quote {
                Some(q) => {
                    if c == q {
                        quote = None;
                    } else {
                        cur.push(c);
                    }
                }
                None if c == '"' || c == '\'' => {
                    quote = Some(c);
                    in_tok = true;
                }
                None if c == '#' => break, // comment to end of line
                None if c.is_whitespace() => {
                    if in_tok {
                        toks.push(std::mem::take(&mut cur));
                        in_tok = false;
                    }
                }
                None => {
                    cur.push(c);
                    in_tok = true;
                }
            }
        }
        if in_tok {
            toks.push(cur);
        }
    }
    toks
}

/// Expand `@FILE` response-file arguments and `--args-file FILE` in `raw`, recursively (bounded), so
/// an entire command line can be kept in a text file. Each referenced file's tokens are spliced in
/// place; every other argument passes through untouched. `#` comments and quoting are supported.
/// A missing/unreadable file is a hard error (the `@`/`--args-file` sigil is an explicit request).
fn expand_arg_files(raw: &[String]) -> Result<Vec<String>, String> {
    fn go(args: &[String], out: &mut Vec<String>, depth: u32) -> Result<(), String> {
        if depth > 16 {
            return Err("--args-file nesting too deep (cycle?)".to_string());
        }
        let mut i = 0;
        while i < args.len() {
            let a = &args[i];
            let path = if let Some(p) = a.strip_prefix('@') {
                Some(p.to_string())
            } else if a == "--args-file" || a == "--args" {
                i += 1;
                Some(args.get(i).ok_or("--args-file needs a file path")?.clone())
            } else {
                None
            };
            match path {
                Some(p) => {
                    let text = std::fs::read_to_string(&p).map_err(|e| format!("args file '{p}': {e}"))?;
                    go(&tokenize_args_file(&text), out, depth + 1)?;
                }
                None => out.push(a.clone()),
            }
            i += 1;
        }
        Ok(())
    }
    let mut out = Vec::new();
    go(raw, &mut out, 0)?;
    Ok(out)
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
        for (i, slot) in out.iter_mut().enumerate().take(RAND_STOPS) {
            let t = i as f32 / (RAND_STOPS - 1) as f32; // 0..1
            let arc = (std::f32::consts::PI * t).sin(); // 0 at ends → 1 at middle
            let h = h0 + dir * hue_span * arc;
            let v = v_lo + (v_hi - v_lo) * arc;
            let c = hsv_to_rgb(h, sat, v);
            *slot = [c[0], c[1], c[2], t];
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
        // Per-channel lerp of two parallel stop arrays (from/to) into out; explicit indices read
        // clearer here than a double `enumerate`, so the range-loop lint is allowed locally.
        #[allow(clippy::needless_range_loop)]
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

// `lerp_color` / `dual_toggle_button` moved to `theme.rs` (re-exported below).

/// Continuous-zoom tuning.
pub(crate) const ZOOM_RATE: f64 = 0.462; // ln(2)/1.5 ≈ ~2× magnification per 1.5 s at full speed
const EASE_TAU: f64 = 0.15; // ease-in/out time constant (seconds)
/// Keep anti-aliasing off for this long after the last interaction, so rapid zoom
/// steps don't each trigger a full-AA render (which felt laggy).
const SETTLE_DELAY: f64 = 0.18;

/// Anti-alias supersampling for progressive-settle stage `frame`, ramping 1→2→4→… up to `target`.
/// A settled view refines from an instant coarse frame to full AA over a few frames, rather than
/// blocking on one expensive full-AA frame. `frame` is capped so the shift can't overflow.
fn aa_ramp(frame: u32, target: u32) -> u32 {
    (1u32 << frame.min(5)).min(target.max(1))
}
/// Max GPU work per render (texels × iterations) before the OS GPU watchdog (TDR)
/// risks a device-lost crash. Supersampling auto-reduces to stay under this.
pub(crate) const WORK_BUDGET: u64 = 60_000_000_000;


/// Upper bound on a loaded/pasted `.fdn` location blob (untrusted input). A real one is
/// well under 1 KB; this caps parse work without rejecting anything legitimate.
const SHARE_MAX: usize = 256 * 1024;

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
pub(crate) const PERT_FE_THRESHOLD: f64 = 1.0e28;

// Version / system-info / time helpers moved to `sysinfo.rs` (re-exported below).

// ---- Fractadyne branding (matches design/Fractadyne.dc.html) ----
// Branding (BRAND_ACCENT/BRAND_TEXT, apply_brand_theme, brand_wordmark, brand_icon) moved
// to `theme.rs` (re-exported below).

// ---- coloring-method / orbit-trap enums (Phase 4: typed dispatch) ----
/// Coloring method — a typed replacement for the former bare `u32`. Discriminants match the GPU
/// `color_method` ids in `mandelbrot.wgsl` `fs_color` (serialized via [`ColorMethod::to_u32`]); the
/// `key` is the stable string persisted in the session / `.fdn`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum ColorMethod {
    #[default]
    Smooth = 0,
    Stripe = 1,
    TriangleIneq = 2,
    OrbitTrap = 3,
    Distance = 4,
    Decomposition = 5,
}
impl ColorMethod {
    pub(crate) const ALL: [ColorMethod; 6] = [
        ColorMethod::Smooth,
        ColorMethod::Stripe,
        ColorMethod::TriangleIneq,
        ColorMethod::OrbitTrap,
        ColorMethod::Distance,
        ColorMethod::Decomposition,
    ];
    /// GPU dispatch id (matches the shader).
    pub(crate) fn to_u32(self) -> u32 {
        self as u32
    }
    /// Stable persisted key.
    pub(crate) fn key(self) -> &'static str {
        match self {
            ColorMethod::Smooth => "smooth",
            ColorMethod::Stripe => "stripe",
            ColorMethod::TriangleIneq => "triangle",
            ColorMethod::OrbitTrap => "trap",
            ColorMethod::Distance => "distance",
            ColorMethod::Decomposition => "decomposition",
        }
    }
    /// Human label for the UI combo box.
    pub(crate) fn label(self) -> &'static str {
        match self {
            ColorMethod::Smooth => "Smooth iteration",
            ColorMethod::Stripe => "Stripe average",
            ColorMethod::TriangleIneq => "Triangle inequality",
            ColorMethod::OrbitTrap => "Orbit trap",
            ColorMethod::Distance => "Distance estimate",
            ColorMethod::Decomposition => "Decomposition",
        }
    }
    pub(crate) fn from_key(s: &str) -> ColorMethod {
        ColorMethod::ALL.into_iter().find(|m| m.key() == s).unwrap_or_default()
    }
    pub(crate) fn from_u32(v: u32) -> ColorMethod {
        ColorMethod::ALL.into_iter().find(|m| m.to_u32() == v).unwrap_or_default()
    }
    /// Methods that accumulate the aux statistics texture (stripe / triangle-ineq / decomposition).
    pub(crate) fn needs_aux(self) -> bool {
        matches!(
            self,
            ColorMethod::Stripe | ColorMethod::TriangleIneq | ColorMethod::Decomposition
        )
    }
}

/// Orbit-trap shape (only meaningful for [`ColorMethod::OrbitTrap`]). Discriminants match the shader.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum TrapType {
    #[default]
    Point = 0,
    Cross = 1,
    Circle = 2,
}
impl TrapType {
    pub(crate) const ALL: [TrapType; 3] = [TrapType::Point, TrapType::Cross, TrapType::Circle];
    pub(crate) fn to_u32(self) -> u32 {
        self as u32
    }
    pub(crate) fn key(self) -> &'static str {
        match self {
            TrapType::Point => "point",
            TrapType::Cross => "cross",
            TrapType::Circle => "circle",
        }
    }
    pub(crate) fn label(self) -> &'static str {
        match self {
            TrapType::Point => "Point",
            TrapType::Cross => "Cross",
            TrapType::Circle => "Circle",
        }
    }
    pub(crate) fn from_key(s: &str) -> TrapType {
        TrapType::ALL.into_iter().find(|t| t.key() == s).unwrap_or_default()
    }
}

/// The numeric representation the renderer uses, chosen by depth (matches the shader's `mode`).
/// Serialized to `u32` only when written into the GPU uniforms / export request.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum RenderMode {
    /// df32 perturbation of a reference orbit (`1e4× … PERT_FE_THRESHOLD`).
    Df32Pert = 0,
    /// Direct df32 from z₀, no reference (shallow `< 1e4×`, or a non-perturbation formula).
    Direct = 1,
    /// Extended-range floatexp perturbation (`≥ PERT_FE_THRESHOLD`).
    Floatexp = 2,
}
impl RenderMode {
    /// Pick the representation for a view: direct when shallow or the formula has no perturbation,
    /// then df32, switching to floatexp past `PERT_FE_THRESHOLD`. The one place this is decided.
    pub(crate) fn select(supports_perturbation: bool, mag: f64) -> RenderMode {
        if !supports_perturbation || mag < 1.0e4 {
            RenderMode::Direct
        } else if mag < PERT_FE_THRESHOLD {
            RenderMode::Df32Pert
        } else {
            RenderMode::Floatexp
        }
    }
    pub(crate) fn to_u32(self) -> u32 {
        self as u32
    }
    pub(crate) fn from_u32(v: u32) -> RenderMode {
        match v {
            1 => RenderMode::Direct,
            2 => RenderMode::Floatexp,
            _ => RenderMode::Df32Pert,
        }
    }
    /// Direct path — no reference orbit, never glitches.
    pub(crate) fn is_direct(self) -> bool {
        matches!(self, RenderMode::Direct)
    }
    /// Extended-range floatexp path (the deep, ~5×-costlier mode-2).
    pub(crate) fn is_floatexp(self) -> bool {
        matches!(self, RenderMode::Floatexp)
    }
}

/// Fixed export aspect ratios (key, width ÷ height). "window" (not listed) matches the live view.
const EXPORT_ASPECTS: [(&str, f64); 8] = [
    ("16:9", 16.0 / 9.0),
    ("16:10", 16.0 / 10.0),
    ("3:2", 3.0 / 2.0),
    ("4:3", 4.0 / 3.0),
    ("1:1", 1.0),
    ("2:3", 2.0 / 3.0),
    ("9:16", 9.0 / 16.0),
    ("2:1", 2.0),
];


/// Curated famous Mandelbrot locations: (name, center_x, center_y, magnification).
/// Coordinates are full-precision strings so deep entries land exactly.
const FAMOUS: &[(&str, &str, &str, f64)] = &[
    ("Seahorse Valley", "-0.74364388703", "0.13182590421", 2.0e3),
    ("Elephant Valley", "0.2925755", "-0.0149977", 1.5e3),
    ("Triple Spiral", "-0.088643135", "0.654461185", 1.2e3),
    ("Double Spiral", "-0.7470837", "0.1080358", 3.0e3),
    ("Spiral Galaxy", "-0.7269", "0.1889", 2.0e3),
    ("Mini Mandelbrot", "-1.7687788", "0.0017388", 6.0e3),
    ("Deep Seahorse", "-0.743643887037151", "0.131825904205330", 1.0e7),
];

/// Zoom-appropriate iteration cap. A very high manual iteration count over-resolves the
/// boundary's sub-pixel "dust" into per-pixel noise (and starves the render budget); this
/// caps the count at a generous, zoom-scaled value so normal auto-iteration is never
/// limited but an inflated base is. Used for both the live view and exports so they match.
/// Zoom-appropriate iteration cap from the zoom **octaves** (`log2(magnification)`), taken
/// directly so it stays finite past 1e308× where `magnification()` saturates to `∞`.
pub(crate) fn zoom_iter_cap(octaves: f64) -> u32 {
    let o = octaves.max(0.0);
    (2000.0 + o * 256.0).min(u32::MAX as f64) as u32
}

// Self-test helpers + run_selftest moved to selftest.rs.

/// Plain-f64 Mandelbrot escape test: `Some(iter)` if it escapes within `max`, else
/// `None` (treated as interior). Used by the random-location boundary search.
fn mandel_escapes(cx: f64, cy: f64, max: u32) -> Option<u32> {
    let (mut zx, mut zy) = (0.0_f64, 0.0_f64);
    for i in 0..max {
        let (x2, y2) = (zx * zx, zy * zy);
        if x2 + y2 > 4.0 {
            return Some(i);
        }
        zy = 2.0 * zx * zy + cy;
        zx = x2 - y2 + cx;
    }
    None
}

/// Sample packed gradient stops (`[r, g, b, pos]`, ascending) at `t∈0..1` — mirrors the
/// shader's `palette()` — and gamma-encode to a display `Color32`.
fn sample_stops(stops: &[[f32; 4]; fractadyne_color::MAX_STOPS], n: u32, t: f32) -> egui::Color32 {
    let t = t.fract();
    let mut col = [stops[0][0], stops[0][1], stops[0][2]];
    let n = n.max(1) as usize;
    for i in 0..n.saturating_sub(1) {
        let (a, b) = (stops[i], stops[i + 1]);
        if t >= a[3] && t <= b[3] {
            let f = (t - a[3]) / (b[3] - a[3]).max(1.0e-6);
            col = [
                a[0] + (b[0] - a[0]) * f,
                a[1] + (b[1] - a[1]) * f,
                a[2] + (b[2] - a[2]) * f,
            ];
            break;
        }
    }
    let g = |c: f32| (c.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0 + 0.5) as u8;
    egui::Color32::from_rgb(g(col[0]), g(col[1]), g(col[2]))
}

// ---------------- Help content -------------------------------------------------
// In-app Help content lives in help.rs (the help_* section fns; imported via use help::*).

// ---- minimap overview ----
/// Fixed complex region the minimap thumbnail covers (center + half-extents), so the
/// "you are here" marker projects consistently regardless of the screen aspect.
const MINIMAP_CX: f64 = -0.5;
const MINIMAP_CY: f64 = 0.0;
const MINIMAP_HX: f64 = 1.6;
const MINIMAP_HY: f64 = 1.2;
/// Thumbnail render resolution (display size is scaled down in the overlay).
const MINIMAP_TW: u32 = 240;
const MINIMAP_TH: u32 = 180;

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
    // Expand `@response-file` args and `--args-file FILE` so the whole command line can live in a
    // text file (see `expand_arg_files`). Do it once, up front, so every consumer sees the result.
    let args = match expand_arg_files(&std::env::args().collect::<Vec<_>>()) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("fractadyne: {e}");
            std::process::exit(2);
        }
    };
    if cli::run_headless(&args) {
        return Ok(());
    }

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([640.0, 400.0])
            .with_title(format!("Fractadyne v{}", version_string()))
            .with_icon(brand_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Fractadyne",
        native_options,
        Box::new(move |cc| Ok(Box::new(FractadyneApp::new(cc, &args)))),
    )
}

/// Group an integer string with commas every 3 digits (handles a leading `-`).
fn commas(s: &str) -> String {
    let neg = s.starts_with('-');
    let digits = s.trim_start_matches('-');
    let len = digits.len();
    let mut out = String::with_capacity(len + len / 3 + 1);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
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

/// Space-group the fractional digits of a scientific-notation string's mantissa (in 5s),
/// matching the coordinate readout: `3.38050027227e15` → `3.38050 02722 7e15`. The exponent
/// is left intact. For display only — `parse_zoom_to_log2` strips spaces, so it round-trips.
fn group_sci_mantissa(s: &str) -> String {
    let (mant, exp) = match s.split_once(['e', 'E']) {
        Some((m, e)) => (m, Some(e)),
        None => (s, None),
    };
    let grouped = match mant.split_once('.') {
        Some((int_part, frac)) => {
            let mut g = String::with_capacity(frac.len() + frac.len() / 5);
            for (i, c) in frac.chars().enumerate() {
                if i > 0 && i % 5 == 0 {
                    g.push(' ');
                }
                g.push(c);
            }
            format!("{int_part}.{g}")
        }
        None => mant.to_string(),
    };
    match exp {
        Some(e) => format!("{grouped}e{e}"),
        None => grouped,
    }
}

/// Magnification with comma-grouped integer part + 2 decimals, e.g. `1,805,359.12`.
fn fmt_zoom(mag: f64) -> String {
    if mag > 1.0e12 {
        // Deep zoom: scientific notation, 12 significant digits (a 30-digit integer is
        // unreadable). `{:.11e}` → e.g. `3.38050027227e15`, with the mantissa space-grouped.
        group_sci_mantissa(&format!("{mag:.11e}"))
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

/// Format magnification from `log2(magnification)` — stays correct past `f64`'s 1e308×
/// (where `magnification()` saturates to `∞`), formatting `2^log2mag` via base-10.
pub(crate) fn fmt_zoom_log2(log2mag: f64) -> String {
    if log2mag <= 1020.0 {
        fmt_zoom(2f64.powf(log2mag.max(0.0)))
    } else {
        let log10 = log2mag * std::f64::consts::LOG10_2;
        let e = log10.floor();
        let m = 10f64.powf(log10 - e);
        group_sci_mantissa(&format!("{m:.2}e{e:.0}"))
    }
}

/// Magnification as a plain scientific string (no grouping), parseable by
/// [`parse_zoom_to_log2`] and valid past f64 range — used to pre-fill the go-to field.
fn fmt_zoom_field(log2mag: f64) -> String {
    if log2mag <= 1020.0 {
        format!("{:e}", 2f64.powf(log2mag.max(0.0)))
    } else {
        let log10 = log2mag * std::f64::consts::LOG10_2;
        let e = log10.floor();
        let m = 10f64.powf(log10 - e);
        format!("{m:.6}e{e:.0}")
    }
}

/// Parse a magnification string (plain or scientific, e.g. `256`, `1.5e400`) into
/// `log2(magnification)`, reading the base-10 exponent directly so values far past f64
/// range still work. Grouping (`,` `_` spaces) is ignored. `None` on garbage / non-positive.
fn parse_zoom_to_log2(s: &str) -> Option<f64> {
    let t: String = s.chars().filter(|c| !matches!(c, ',' | '_' | ' ' | '\t')).collect();
    if t.is_empty() {
        return None;
    }
    let (mant, exp) = match t.split_once(['e', 'E']) {
        Some((m, x)) => (m, x.parse::<f64>().ok()?),
        None => (t.as_str(), 0.0),
    };
    let m: f64 = mant.parse().ok()?;
    if !(m.is_finite() && m > 0.0 && exp.is_finite()) {
        return None;
    }
    Some(m.log2() + exp * std::f64::consts::LOG2_10)
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

/// Decompose a decimal/scientific number string into positional parts (sign, integer
/// digits, fractional digits) — `astro_float`'s `Display` is scientific (`7.43e-1`,
/// `5.e-1`), so shift the point by the exponent to recover plain `0.743…` form.
fn decimal_parts(s: &str) -> (&'static str, String, String) {
    let (sign, body) = match s.strip_prefix('-') {
        Some(r) => ("-", r),
        None => ("+", s.trim_start_matches('+')),
    };
    let (mant, exp) = match body.split_once(['e', 'E']) {
        Some((m, e)) => (m, e.trim().parse::<i32>().unwrap_or(0)),
        None => (body, 0),
    };
    let (mant_int, mant_frac) = match mant.split_once('.') {
        Some((i, f)) => (i, f),
        None => (mant, ""),
    };
    let digits: String = format!("{mant_int}{mant_frac}");
    // Digits to the left of the point: the mantissa's integer width, shifted by the exponent.
    let point = mant_int.len() as i32 + exp;
    let n = digits.len() as i32;
    if point <= 0 {
        (sign, "0".to_string(), format!("{}{digits}", "0".repeat((-point) as usize)))
    } else if point >= n {
        (sign, format!("{digits}{}", "0".repeat((point - n) as usize)), String::new())
    } else {
        let p = point as usize;
        (sign, digits[..p].to_string(), digits[p..].to_string())
    }
}

/// Format an arbitrary-precision coordinate for the status bar, showing enough significant
/// fractional digits for the current zoom (an `f64` readout freezes at ~15 digits, so deep
/// pans look static). Past a width threshold the middle is elided — `leading … frontier` —
/// so the deepest, *changing* digits stay on screen, e.g. `-0.74364 38870 … 06114 7740`.
pub(crate) fn fmt_coord_deep(v: &fractadyne_core::BigFloat, log2mag: f64) -> String {
    // Group a digit run in 5s for readability.
    let group = |digits: &str| -> String {
        let mut g = String::with_capacity(digits.len() + digits.len() / 5);
        for (i, c) in digits.chars().enumerate() {
            if i > 0 && i % 5 == 0 {
                g.push(' ');
            }
            g.push(c);
        }
        g
    };
    let (sign, int_part, frac) = decimal_parts(&fractadyne_core::to_decimal_string(v));
    if frac.is_empty() {
        return format!("{sign}{int_part}");
    }
    // Significant fractional digits worth showing ≈ decimal octaves of zoom + guard, capped
    // by what's actually stored. Floor of 15 keeps shallow views looking like `fmt_coord`
    // (but never asks for more digits than exist — avoids a min>max clamp on short coords).
    let want = (log2mag * std::f64::consts::LOG10_2).max(0.0).ceil() as usize + 6;
    let lo = 15.min(frac.len());
    let d = want.clamp(lo, frac.len());
    const HEAD: usize = 10; // leading digits kept
    const TAIL: usize = 10; // frontier digits kept
    if d <= HEAD + TAIL + 5 {
        format!("{sign}{int_part}.{}", group(&frac[..d]))
    } else {
        format!(
            "{sign}{int_part}.{} … {}",
            group(&frac[..HEAD]),
            group(&frac[d - TAIL..d])
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExportFormat {
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

// System-facts helpers (process_memory, SysInfo, gather_system_info, CPU/VRAM probes)
// moved to sysinfo.rs (re-exported below).
// ---- Scripting: keyframe camera tours (also drives the benchmark) ----

// Scripting/benchmark types + helpers moved to scripting.rs.

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
    pub(crate) const ALL: [PaletteAnim; 5] = [
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
    pub(crate) fn key(self) -> &'static str {
        match self {
            PaletteAnim::Off => "off",
            PaletteAnim::Forward => "forward",
            PaletteAnim::Reverse => "reverse",
            PaletteAnim::PingPong => "pingpong",
            PaletteAnim::Random => "random",
        }
    }
    pub(crate) fn from_key(s: &str) -> PaletteAnim {
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
pub(crate) enum ExportJob {
    Single(fractadyne_gpu::ExportRequest),
    SideBySide(fractadyne_gpu::ExportRequest, fractadyne_gpu::ExportRequest),
    Separate(fractadyne_gpu::ExportRequest, fractadyne_gpu::ExportRequest),
}

/// Stitch two rendered images horizontally (left | right) into one RGBA buffer.
pub(crate) fn stitch_side_by_side(
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
pub(crate) fn separate_paths(path: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
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
    /// Cached series-approximation skip + coefficients for this reference, and the
    /// `(orbit_id, eff_iter)` it was computed for (recompute only when that changes — the
    /// bignum coefficient iteration is as costly as the reference orbit itself).
    sa: fractadyne_core::SeriesSkip,
    sa_key: (u64, u32),
    /// Cached BLA tree (GPU-packed) for this reference, the `orbit_id` it was built for, and the
    /// `log2` of the worst-case `|δc|` (`dc_max`) it was built with. Rebuilt only when the orbit
    /// changes or the view zooms out enough that a larger `dc_max` is needed — so BLA doesn't pay a
    /// per-frame tree rebuild (panning within a reference reuses it). Empty = none/not built.
    bla: std::sync::Arc<Vec<[f32; 4]>>,
    bla_id: u64,
    bla_dc_max_log2: f64,
    /// View (center + log2 magnification) the current iteration texture was rendered at. A freeze
    /// uses this to zoom-reproject the frozen frame — scaling/panning it to follow the dive until a
    /// fresh reference lands (see `build_params`). `None` until the first real render.
    frozen_center: Option<[fractadyne_core::BigFloat; 2]>,
    frozen_l2: f64,
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
            sa: fractadyne_core::SeriesSkip::NONE,
            sa_key: (u64::MAX, u32::MAX),
            bla: std::sync::Arc::new(Vec::new()),
            bla_id: u64::MAX,
            bla_dc_max_log2: f64::NEG_INFINITY,
            frozen_center: None,
            frozen_l2: 0.0,
        }
    }
}

/// A saved view (bookmark). `meta` is the same key=value view-metadata blob used by
/// exports, restorable via `load_view_metadata`.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Bookmark {
    name: String,
    meta: String,
    /// Thumbnail id — a small PNG preview lives at `bookmark_thumbs/<thumb>.png`. Empty when
    /// none (older bookmarks, or the render hasn't happened yet).
    #[serde(default)]
    thumb: String,
}

/// TOML wrapper for the bookmarks file (`[[bookmark]]` array).
#[derive(Default, serde::Serialize, serde::Deserialize)]
struct BookmarkFile {
    #[serde(default)]
    bookmark: Vec<Bookmark>,
}

/// A navigation history entry (location only) for undo/redo.
#[derive(Clone)]
struct ViewSnapshot {
    cx: fractadyne_core::BigFloat,
    cy: fractadyne_core::BigFloat,
    upp: fractadyne_core::FloatExp,
    prec: usize,
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

// `FractalInfo` / `FractalKind` moved to `fractal.rs` (re-exported at the top of this file).

/// An in-progress **zoom box** (Shift+drag): rubber-band rectangle from `start` to `end`
/// (egui points) that, on release, zooms so the box fills the panel. `is_julia` tags which
/// panel it belongs to (dual view).
struct ZoomBox {
    start: egui::Pos2,
    end: egui::Pos2,
    is_julia: bool,
}

/// Constrain a free drag (`start`→`end`) to the panel's aspect ratio, anchored at `start` and
/// clamped inside `rect`, so the resulting box zooms to fill without distortion.
fn aspect_zoom_box(start: egui::Pos2, end: egui::Pos2, rect: egui::Rect) -> egui::Rect {
    let aspect = (rect.width() / rect.height().max(1.0)).max(1e-3);
    let (dx, dy) = ((end.x - start.x).abs(), (end.y - start.y).abs());
    let sx = if end.x >= start.x { 1.0 } else { -1.0 };
    let sy = if end.y >= start.y { 1.0 } else { -1.0 };
    // Enclose the drag, then clamp to what fits inside `rect` from `start` (keeping aspect).
    let maxw = if sx > 0.0 { rect.max.x - start.x } else { start.x - rect.min.x };
    let maxh = if sy > 0.0 { rect.max.y - start.y } else { start.y - rect.min.y };
    let w = dx.max(dy * aspect).min(maxw.max(0.0)).min(maxh.max(0.0) * aspect);
    let corner = egui::pos2(start.x + sx * w, start.y + sy * (w / aspect));
    egui::Rect::from_two_pos(start, corner)
}

/// State of the "Go to location" dialog (transient — not persisted). First of the Phase-2a
/// field groups that break up the flat `FractadyneApp` struct (see REFACTOR-PLAN.md).
#[derive(Default)]
struct GotoDialog {
    open: bool,
    x: String,
    y: String,
    zoom: String,
    msg: Option<String>,
}

/// State of the "Share location" (`.fdn`) dialog (transient).
#[derive(Default)]
struct ShareDialog {
    open: bool,
    text: String,
    msg: Option<String>,
}

/// Benchmark-configuration dialog state (transient).
struct BenchConfig {
    standard: bool,
    res: BenchRes,
    depth: BenchDepth,
    burnin: bool,
    passes: u32,
}
impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            standard: true,
            res: BenchRes::P1080,
            depth: BenchDepth::Standard,
            burnin: false,
            passes: 10,
        }
    }
}

/// Gallery-browser dialog state (transient): open flag, scanned folder, and its entries.
#[derive(Default)]
struct GalleryState {
    open: bool,
    dir: std::path::PathBuf,
    entries: Vec<GalleryEntry>,
}

/// Navigation history for undo/redo of view changes (transient).
#[derive(Default)]
struct NavHistory {
    undo: Vec<ViewSnapshot>,
    redo: Vec<ViewSnapshot>,
    was_interacting: bool,
}

/// Export-dialog state: the dialog's options, the in-flight background export, and the persisted
/// target folder. Grouped from the former flat `export_*` fields (Phase 2a).
struct ExportState {
    open: bool,
    width: u32,
    ss: u32,
    format: ExportFormat,
    dual_mode: DualExport,
    /// Aspect ratio: "window" (match the live view) or a fixed key ("16:9", "1:1", …).
    aspect: String,
    notes: String,
    status: Option<String>,
    /// In-flight background export; receives the final status message when done.
    task: Option<std::sync::mpsc::Receiver<String>>,
    /// Deep single-view export whose reference orbit is building off-thread (before the render).
    prep: Option<crate::export::ExportPrep>,
    /// Progress in permille (0–1000) and a cooperative cancel flag.
    progress: std::sync::Arc<std::sync::atomic::AtomicU32>,
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// Last directory an export was saved to (persisted; defaults the Save dialog).
    last_dir: Option<std::path::PathBuf>,
    /// When the in-flight export began (deep exports: at the reference build). Drives the live
    /// "elapsed" readout and the final total-time report.
    started: Option<std::time::Instant>,
}

/// Relief-lighting + distance-estimate-glow effect settings (mostly persisted; `de_phase` is
/// runtime animation state). Grouped from the former flat `light_*` / `de_*` fields (Phase 2a).
struct EffectsConfig {
    /// Distance-estimate relief lighting (slope shading from the derivative).
    light: bool,
    light_angle: f32,  // radians
    light_height: f32, // relief strength (smaller = sharper)
    light_anim: bool,  // rotate the light direction over time
    /// Distance-estimate glow (contour bands near the boundary), + animation.
    de: bool,
    de_strength: f32,
    de_width: f32,
    de_anim: bool,
    de_phase: f32, // runtime (animated)
}

/// Auto-zoom autopilot state (transient except `dive_log2`, which is persisted). Grouped from the
/// former flat `autopilot*` fields (Phase 2a).
struct AutopilotState {
    /// Continuously diving toward the detail-richest region.
    active: bool,
    /// Screen-fraction pivot (0..1) currently zooming about; eased toward `goal` each frame.
    target: (f64, f64),
    /// Latest *evaluated* target (every `AUTOPILOT_EVAL_INTERVAL`); the goal `target` chases.
    goal: (f64, f64),
    /// App-time of the last target re-evaluation.
    eval_t: f64,
    /// True during the deep *stepped* dive (past the smooth regime) — see autopilot.rs / render.rs.
    stepping: bool,
    /// Dive limit as log2(magnification); persisted.
    dive_log2: f64,
}

/// Transient pointer / zoom / pan interaction state (not persisted): the in-progress zoom-box and
/// box-zoom gestures, pan reprojection, eased continuous-zoom, cursor tracking, and the per-view
/// settle-quality timers.
#[derive(Default)]
struct PointerState {
    /// In-progress Shift+drag zoom box (rubber-band → zoom to fill); `None` when not dragging.
    zoom_box: Option<ZoomBox>,
    /// Box-zoom (right-drag) start position in screen points; `None` when idle.
    box_start: Option<egui::Pos2>,
    /// Pan reprojection: accumulated device-pixel drag offset since the current pan began, and
    /// which view (0 = main/left, 1 = julia) is being panned. While a pan is pending the frozen
    /// iteration texture is translated by `pan_px` instead of re-rendered; cleared on settle.
    pan_px: egui::Vec2,
    pan_view: Option<u32>,
    /// Eased continuous-zoom velocity (log-rate per second; + = in, - = out, 0 = idle).
    zoom_vel: f64,
    /// Last cursor position over the canvas, for cursor-anchored continuous zoom.
    last_cursor: Option<egui::Pos2>,
    /// Complex coordinate under the cursor (for the status bar); `None` when off-canvas.
    pointer_complex: Option<(f64, f64)>,
    /// App-time of the last interaction, **per view** (`[0]` = main/Mandelbrot, `[1]` = Julia); each
    /// view stays at the coarse "moving" quality until `SETTLE_DELAY` after its own last change. Kept
    /// per-view so the dual view's Julia `c` cursor-drag doesn't force the Mandelbrot panel to
    /// re-render at low resolution.
    settle_t: [f64; 2],
    /// Progressive-AA settle stage, per view. On settle the anti-aliasing ramps 1×→2×→4×→… up to the
    /// chosen level over consecutive frames (each schedules the next); reset to 0 while interacting.
    settle_frame: [u32; 2],
}

/// How the fractal is *computed* (not colored): iteration budget + auto-scale, the perturbation
/// accelerators, continuous-zoom speed, the live work-budget, and anti-aliasing. All persisted —
/// round-trips 1:1 through `SessionState`. Grouped from the former flat compute fields (Phase 2a).
struct RenderConfig {
    /// Iteration budget ceiling.
    max_iter: u32,
    /// Auto-scale iteration count with zoom depth (else use `max_iter` as-is).
    auto_iter: bool,
    /// Series approximation (iteration-skipping) for deep Mandelbrot renders. Default on.
    series_approx: bool,
    /// Multi-reference glitch correction for exports (multi-pass, glitch-free). Default off.
    glitch_correct: bool,
    /// BLA (bilinear approximation): skips iterations throughout the orbit for deep floatexp
    /// Mandelbrot renders. Off by default while it's validated; enable to accelerate.
    use_bla: bool,
    /// Continuous-zoom speed multiplier (1.0 = default `ZOOM_RATE`).
    zoom_rate: f32,
    /// Live-render work-budget multiplier (× `WORK_BUDGET`); persisted. Higher = crisper live deep
    /// zoom (fuller resolution) at lower FPS / less GPU-watchdog margin. Exports are unaffected.
    work_budget_scale: f64,
    /// Supersampling / anti-alias factor (1 = off, 2 = 2×2, 3 = 3×3).
    aa: u32,
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
    /// Dual-view split position (fraction of width) — the draggable separator between panels.
    dual_split: f32,
    /// Borderless fullscreen state (toolbar toggle).
    fullscreen: bool,
    /// Viewport for the Julia panel in dual view.
    julia_viewport: Viewport,
    /// Pointer / zoom / pan interaction state (zoom box, pan reprojection, eased zoom, settle timers).
    pointer: PointerState,
    /// Active smooth "zoom out to home" animation (Home button); `None` when idle.
    home_anim: Option<HomeAnim>,
    /// Auto-zoom autopilot state.
    autopilot: AutopilotState,
    /// Draw the iteration orbit of the point under the cursor.
    show_orbits: bool,
    /// A tour-scripted orbit point (complex) to draw when there's no cursor (during playback);
    /// `None` = use the cursor. Set each frame by `advance_playback`.
    tour_orbit: Option<(f64, f64)>,
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
    /// The "Benchmark…" configuration dialog (mode / resolution / burn-in).
    bench_dialog_open: bool,
    bench_cfg: BenchConfig,
    /// In-flight standardized benchmark, advanced one pass per frame from `update()`.
    std_bench: Option<StdBench>,
    /// CLI `--benchmark-std [--res RES] [--burnin N]`: run headless, save, quit.
    auto_stdbench: bool,
    auto_stdbench_done: bool,
    std_res: BenchRes,
    std_passes: u32,
    std_depth: BenchDepth,
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
    /// `--render-iter`: write the raw iteration texture as EXR instead of a colored image.
    render_iter_mode: bool,
    /// CLI `--render-tour FILE`: render a keyframe tour to a PNG frame sequence, then quit.
    render_tour: Option<std::path::PathBuf>,
    render_tour_done: bool,
    tour_fps: f64,
    tour_size: (u32, u32),
    tour_ss: u32,
    tour_out: std::path::PathBuf,
    /// CLI `--prefix NAME`: frame-name prefix (frames `<prefix>_00000.png`). Defaults to the tour
    /// script's file stem, or "frame".
    tour_prefix: String,
    /// CLI `--overwrite` / `-y`: replace existing frames without prompting.
    tour_overwrite: bool,
    /// CLI `--resume`: skip already-rendered frames and render only the missing ones (restart an
    /// interrupted render).
    tour_resume: bool,
    /// CLI `--mp4 [PATH]`: after rendering the tour, assemble the frames into an mp4 via ffmpeg.
    tour_mp4: Option<std::path::PathBuf>,
    /// CLI `--selftest`: run the GPU validation suite, print a report, and exit.
    selftest: bool,
    selftest_done: bool,
    /// CLI `--profile`: run the profiling harness (benchmark regions), log to `logs/`, exit.
    profile: bool,
    profile_done: bool,
    profile_reps: u32,
    profile_regions: Option<String>,
    profile_out: Option<std::path::PathBuf>,
    /// CLI `--frametest`: run the frame-timing / stutter harness (deep-zoom dive), log, exit.
    frametest: bool,
    frametest_steps: u32,
    frametest_hold: u32,
    frametest_dive: f64,
    /// Per-render setup timings (reference / series-skip), recorded by
    /// `current_export_request_for` via a `Cell` and read by the profiler.
    prof: std::cell::Cell<profile::ProfSetup>,
    /// Frame-rate cap (FPS); `None` = uncapped.
    fps_cap: Option<f64>,
    /// Export dialog + in-flight background export state.
    export: ExportState,
    /// Gallery browser state.
    gallery: GalleryState,
    /// Bookmarks (saved views), persisted to the config dir; + window/input state.
    bookmarks: Vec<Bookmark>,
    bookmarks_open: bool,
    bookmark_name: String,
    /// A just-added bookmark whose thumbnail still needs rendering (deferred to `update`,
    /// where the GPU is available and the current view still matches the bookmark).
    pending_thumb: Option<usize>,
    /// Decoded bookmark-thumbnail textures, keyed by thumb id (lazy-loaded for the dialog).
    thumb_cache: std::collections::HashMap<String, egui::TextureHandle>,
    /// Navigation history (location undo/redo) + settle-edge tracking.
    nav: NavHistory,
    /// "Go to location" dialog state.
    goto: GotoDialog,
    /// Share-location (`.fdn`) dialog: open flag, editable location text, and an error line.
    share: ShareDialog,
    /// Transient status toast (message, time set) — e.g. minibrot-finder result.
    toast: Option<(String, f64)>,
    /// A toast queued from a context without an `egui::Context` (e.g. a bookmark auto-save
    /// failure in `save_bookmarks`); drained into `toast` early in `update()`.
    pending_toast: Option<String>,
    /// "Reset application state" confirmation dialog open.
    reset_confirm_open: bool,
    /// Set after a reset: stop autosaving so we don't recreate the just-deleted state file.
    suppress_autosave: bool,
    /// A one-shot warning to show as a toast on the first frame (e.g. a newer-version session).
    pending_state_warning: Option<String>,
    /// Keyboard/help overlay window open, and the selected Help section index.
    help_open: bool,
    help_section: usize,
    /// Whether the right-hand control panel is shown.
    right_panel_open: bool,
    /// Compute knobs (iteration budget, perturbation accelerators, zoom speed, work-budget, AA).
    render_cfg: RenderConfig,
    /// Draw the discreet "Fd" brand mark in the lower-right of the live view and exports. On by
    /// default; toggleable.
    watermark: bool,
    /// Burn a zoom/coordinate HUD (top-left) into rendered frames. CLI `--show-location`, or a tour
    /// script's `show_location`. Off by default; only applied on the ctx-bearing render paths
    /// (`--render`, `--render-tour`).
    show_location: bool,
    /// Pre-rasterized "Fd" mark for stamping into exports (built lazily from the font atlas on the
    /// main thread; the export worker has no egui context). `None` until first built.
    watermark_overlay: Option<export::WmOverlay>,
    /// UI scale (egui zoom factor): scales the interface fonts + widgets. 1.0 = default.
    ui_scale: f32,
    /// Minimap overview: enabled flag, cached home-view thumbnail, and the key
    /// (formula, palette, method) the thumbnail was rendered for (re-render on change).
    minimap: bool,
    minimap_tex: Option<egui::TextureHandle>,
    minimap_key: Option<(u32, usize, u32, u32)>,
    /// Custom gradient (editor): stops as `[pos, r, g, b]` (linear RGB). When
    /// `use_custom_palette` is set, this overrides the preset selection. `palette_rev`
    /// bumps on every edit (so caches like the minimap thumbnail refresh).
    custom_palette: Vec<[f32; 4]>,
    use_custom_palette: bool,
    palette_editor_open: bool,
    palette_rev: u32,
    /// Two-color palette modes sharing the `lo`/`hi` colors (linear RGB), overriding
    /// preset/custom: **duotone** maps the value to a smooth `lo → hi → lo` ramp; **binary**
    /// paints a flat `hi` exterior with a flat `lo` interior (just in-set vs out-of-set).
    use_duotone: bool,
    use_binary: bool,
    duotone_lo: [f32; 3],
    duotone_hi: [f32; 3],
    /// Performance/diagnostic tracking + overlay.
    perf: Perf,
    /// Selected palette index into `fractadyne_color::PRESETS`.
    palette_idx: usize,
    /// Color cycle density slider (0..1; mapped to a shader multiplier).
    cycle: f32,
    /// Palette offset slider (0..1).
    offset: f32,
    /// Relief lighting + distance-estimate glow effect settings.
    effects: EffectsConfig,
    /// Coloring method (0 smooth, 1 stripe, 2 triangle-ineq, 3 orbit trap,
    /// 4 distance, 5 decomposition) + its parameters.
    color_method: ColorMethod,
    stripe_freq: f32,
    trap_type: TrapType,
    /// Per-view perturbation reference cache (index 0 = main/left, 1 = dual Julia).
    /// Separate caches let both dual panels use perturbation without thrashing.
    ref_cache: [RefCache; 2],
    /// In-flight off-thread reference recompute per view (`None` = idle). Keeps the deep-zoom
    /// bignum recompute off the render thread: the frame keeps using the cached reference until the
    /// worker's result arrives (see `build_params`).
    recompute_rx: [Option<std::sync::mpsc::Receiver<crate::render::RecomputeResult>>; 2],
    /// Last snapshot used for change detection (debounced auto-save).
    last_state: fractadyne_state::SessionState,
    /// App-time (s) of the last change while unsaved; `None` when clean.
    dirty_since: Option<f64>,
}

impl FractadyneApp {
    fn new(cc: &eframe::CreationContext<'_>, args: &[String]) -> Self {
        // A missing wgpu render state means the GPU backend failed to initialize. Report it
        // plainly and exit cleanly rather than surfacing a Rust panic + backtrace to the user.
        let render_state = match cc.wgpu_render_state.as_ref() {
            Some(rs) => rs,
            None => {
                eprintln!(
                    "Fractadyne requires the wgpu GPU backend (eframe Renderer::Wgpu), which failed \
                     to initialize. Check that your GPU drivers support Vulkan/DX12/Metal."
                );
                std::process::exit(1);
            }
        };
        install_renderer(render_state);
        apply_brand_theme(&cc.egui_ctx);
        let gpu_name = render_state.adapter.get_info().name;

        // CLI modes (headless, for automation / debugging):
        //   --benchmark [--out PATH]                    run the built-in benchmark, save, quit
        //   --render --out IMG [view options]           render one image, save, quit
        // Render view options: --fractal NAME, --center X Y, --zoom MAG, --size W,
        //   --ss N, --iter N, --julia, --julia-c RE IM, --palette IDX.
        // `args` already has any `@response-file` / `--args-file` expanded (see `main`).
        let out_path = args
            .iter()
            .position(|a| a == "--out" || a == "-o")
            .and_then(|i| args.get(i + 1))
            .map(std::path::PathBuf::from);
        let auto_benchmark = args.iter().any(|a| a == "--benchmark" || a == "--bench");
        // Standardized (pinned-settings) benchmark: --benchmark-std [--res RES] [--burnin N].
        // --burnin/--res on their own also imply it.
        let render_iter_mode = args.iter().any(|a| a == "--render-iter");
        let auto_render = args.iter().any(|a| a == "--render") || render_iter_mode;
        let selftest = args.iter().any(|a| a == "--selftest");
        let profile = args.iter().any(|a| a == "--profile");
        let frametest = args.iter().any(|a| a == "--frametest");
        let val = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1));
        // Standardized benchmark: --benchmark-std, or implied by --res / --burnin.
        let std_res = val("--res").and_then(|s| BenchRes::from_token(s)).unwrap_or(BenchRes::P1080);
        // --burnin may carry a pass count (`--burnin 20`) or stand alone (defaults to 10).
        let burnin_flag = args.iter().any(|a| a == "--burnin");
        let std_passes = if burnin_flag {
            val("--burnin").and_then(|s| s.parse::<u32>().ok()).unwrap_or(10).clamp(1, 500)
        } else {
            1
        };
        // Dive depth: --depth <standard|ultra> (or the shorthand --ultra). Also implies a
        // standardized run.
        let ultra_flag = args.iter().any(|a| a == "--ultra");
        let std_depth = val("--depth")
            .and_then(|s| BenchDepth::from_token(s))
            .unwrap_or(if ultra_flag { BenchDepth::Ultra } else { BenchDepth::Standard });
        let auto_stdbench = args.iter().any(|a| a == "--benchmark-std" || a == "--std-benchmark")
            || burnin_flag
            || ultra_flag
            || args.iter().any(|a| a == "--res" || a == "--depth");
        // --render-tour FILE [--fps N] [--size W] [--height H] [--ss N] [--out DIR] [--mp4 [PATH]]
        let render_tour = val("--render-tour").map(std::path::PathBuf::from);
        let tour_fps = val("--fps").and_then(|s| s.parse::<f64>().ok()).filter(|f| *f > 0.0).unwrap_or(30.0);
        // --size accepts a bare width (`1920`) or `WIDTHxHEIGHT` (`5120x2160`). Explicit --height
        // overrides the height from --size; otherwise fall back to a 16:9 default.
        let (size_w, size_h) = val("--size").map(|s| parse_size(s)).unwrap_or((None, None));
        let tour_w = size_w.unwrap_or(1280).clamp(16, 16384);
        let tour_h = val("--height")
            .and_then(|s| s.parse::<u32>().ok())
            .or(size_h)
            .unwrap_or((tour_w * 9 / 16).max(16))
            .clamp(16, 16384);
        let tour_ss = val("--ss").and_then(|s| s.parse::<u32>().ok()).unwrap_or(1).clamp(1, 8);
        let tour_out = out_path.clone().unwrap_or_else(|| std::path::PathBuf::from("frames"));
        // Frame-name prefix: --prefix NAME, else the tour script's file stem, else "frame".
        // Frames are written `<prefix>_00000.png`; the mp4 default becomes `<prefix>.mp4`.
        let tour_prefix = val("--prefix").cloned().unwrap_or_else(|| {
            render_tour
                .as_ref()
                .and_then(|p| p.file_stem())
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "frame".to_string())
        });
        // --overwrite / -y: replace existing frames without prompting.
        let tour_overwrite = args.iter().any(|a| a == "--overwrite" || a == "-y");
        // --resume: keep already-rendered frames and render only the missing ones (restart).
        let tour_resume = args.iter().any(|a| a == "--resume");
        // --mp4 [PATH]: presence enables ffmpeg encoding after render. A following non-flag token is
        // the output path; otherwise default to `<out-dir>/<prefix>.mp4`.
        let tour_mp4 = args.iter().position(|a| a == "--mp4").map(|i| {
            args.get(i + 1)
                .filter(|s| !s.starts_with('-'))
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| tour_out.join(format!("{tour_prefix}.mp4")))
        });
        let profile_reps = val("--reps").and_then(|s| s.parse().ok()).unwrap_or(5u32);
        let profile_regions = val("--regions").cloned();
        let frametest_steps = val("--steps").and_then(|s| s.parse().ok()).unwrap_or(40u32);
        let frametest_hold = val("--hold").and_then(|s| s.parse().ok()).unwrap_or(4u32);
        let frametest_dive = val("--dive").and_then(|s| s.parse::<f64>().ok()).unwrap_or(30.0);
        let auto_benchmark_out = out_path.clone();
        let auto_render_out = out_path.clone();

        // Restore the last session (or defaults). The center comes from the
        // full-precision decimal strings when present (deep-zoom locations survive
        // restart); older session files without them fall back to the f64 fields.
        let (s, state_load) = fractadyne_state::load_with_status();
        // Surface a warning (once the UI is up) if the session file was written by a newer build
        // than this one can fully account for.
        let pending_state_warning = match state_load {
            fractadyne_state::StateLoad::Newer(v) => Some(format!(
                "This session was saved by a newer Fractadyne (state v{v}; this build handles v{}). \
                 Some settings may not apply, and saving will rewrite it in this build's format.",
                fractadyne_state::STATE_FORMAT_VERSION
            )),
            _ => None,
        };
        let mut viewport = Viewport::new(1280.0, 720.0);
        viewport.center_x = fractadyne_core::parse_bf(&s.center_x_str)
            .unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(s.center_x, 64));
        viewport.center_y = fractadyne_core::parse_bf(&s.center_y_str)
            .unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(s.center_y, 64));
        viewport.units_per_pixel =
            fractadyne_core::FloatExp::new(s.units_per_pixel, s.units_per_pixel_e);
        viewport.precision =
            fractadyne_core::precision_for_octaves(viewport.log2_magnification().max(0.0).ceil() as u64);
        // Restore the saved fractal family (so the view you left is fully recreated).
        let fractal = FractalKind::from_name(&s.fractal).unwrap_or(FractalKind::Mandelbrot);

        let mut app = Self {
            viewport,
            fractal,
            julia_c: (s.julia_c_re, s.julia_c_im),
            julia_mode: s.julia_mode && fractal.supports_julia(),
            julia_pin: None,
            dual: s.dual && fractal.supports_julia(),
            dual_split: s.dual_split.clamp(0.15, 0.85),
            fullscreen: false,
            julia_viewport: {
                let mut v = Viewport::new(800.0, 800.0);
                v.center_x = fractadyne_core::BigFloat::from_f64(0.0, 64);
                v.center_y = fractadyne_core::BigFloat::from_f64(0.0, 64);
                v
            },
            pointer: PointerState::default(),
            home_anim: None,
            autopilot: AutopilotState {
                active: false,
                target: (0.5, 0.5),
                goal: (0.5, 0.5),
                eval_t: 0.0,
                stepping: false,
                dive_log2: s.autopilot_dive_log2,
            },
            show_orbits: s.show_orbits,
            tour_orbit: None,
            orbit_normalize: s.orbit_normalize,
            orbit_anim: s.orbit_anim,
            orbit_anim_speed: s.orbit_anim_speed,
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
            bench_dialog_open: false,
            bench_cfg: BenchConfig::default(),
            std_bench: None,
            auto_stdbench,
            auto_stdbench_done: false,
            std_res,
            std_passes,
            std_depth,
            gpu_name,
            sysinfo: gather_system_info(),
            auto_benchmark,
            auto_benchmark_out,
            auto_benchmark_done: false,
            auto_render,
            auto_render_out,
            auto_render_done: false,
            render_iter_mode,
            render_tour,
            render_tour_done: false,
            tour_fps,
            tour_size: (tour_w, tour_h),
            tour_ss,
            tour_out,
            tour_prefix,
            tour_overwrite,
            tour_resume,
            tour_mp4,
            selftest,
            selftest_done: false,
            profile,
            profile_done: false,
            profile_reps,
            profile_regions,
            profile_out: out_path.clone(),
            frametest,
            frametest_steps,
            frametest_hold,
            frametest_dive,
            prof: std::cell::Cell::new(profile::ProfSetup::default()),
            fps_cap: (s.fps_cap > 0.0).then_some(s.fps_cap), // 0 = uncapped
            export: ExportState {
                open: false,
                width: s.export_width,
                ss: s.export_ss,
                format: if s.export_format == "exr" {
                    ExportFormat::Exr
                } else {
                    ExportFormat::Png
                },
                dual_mode: match s.export_dual_mode.as_str() {
                    "separate" => DualExport::Separate,
                    "active" => DualExport::ActiveOnly,
                    _ => DualExport::SideBySide,
                },
                aspect: s.export_aspect.clone(),
                notes: String::new(),
                status: None,
                task: None,
                prep: None,
                progress: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
                cancel: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                last_dir: s.export_dir.clone().map(std::path::PathBuf::from),
                started: None,
            },
            gallery: GalleryState { dir: Self::pictures_dir(), ..Default::default() },
            bookmarks: Self::load_bookmarks(),
            bookmarks_open: false,
            pending_thumb: None,
            thumb_cache: std::collections::HashMap::new(),
            bookmark_name: String::new(),
            nav: NavHistory::default(),
            goto: GotoDialog::default(),
            share: ShareDialog::default(),
            toast: None,
            pending_toast: None,
            reset_confirm_open: false,
            suppress_autosave: false,
            pending_state_warning,
            help_open: false,
            help_section: 0,
            right_panel_open: s.right_panel_open,
            minimap: s.minimap,
            minimap_tex: None,
            minimap_key: None,
            custom_palette: s.custom_palette.clone(),
            use_custom_palette: s.use_custom_palette,
            palette_editor_open: false,
            palette_rev: 0,
            use_duotone: s.use_duotone,
            use_binary: s.use_binary,
            duotone_lo: s.duotone_lo,
            duotone_hi: s.duotone_hi,
            perf: Perf {
                // Default on; `--no-perf` disables, `--perf` forces on.
                enabled: !std::env::args().any(|a| a == "--no-perf"),
                ..Perf::default()
            },
            render_cfg: RenderConfig {
                max_iter: s.max_iter,
                auto_iter: s.auto_iter,
                series_approx: s.series_approx,
                glitch_correct: s.glitch_correct,
                use_bla: s.use_bla,
                zoom_rate: s.zoom_rate,
                work_budget_scale: s.work_budget_scale.clamp(0.25, 8.0),
                aa: s.aa,
            },
            palette_idx: s.palette_idx,
            cycle: s.cycle,
            offset: s.offset,
            effects: EffectsConfig {
                light: s.light,
                light_angle: s.light_angle,
                light_height: s.light_height,
                light_anim: s.light_anim,
                de: s.de,
                de_strength: s.de_strength,
                de_width: s.de_width,
                de_anim: s.de_anim,
                de_phase: 0.0,
            },
            color_method: ColorMethod::from_key(&s.color_method),
            stripe_freq: s.stripe_freq,
            trap_type: TrapType::from_key(&s.trap_type),
            watermark: s.watermark,
            show_location: s.show_location
                || args.iter().any(|a| a == "--show-location" || a == "--hud"),
            watermark_overlay: None,
            ui_scale: s.ui_scale.clamp(0.6, 2.5),
            ref_cache: [RefCache::default(), RefCache::default()],
            recompute_rx: [None, None],
            last_state: s,
            dirty_since: None,
        };
        // `--bla` / `--no-bla` force BLA on/off for any headless mode (profiling / benchmark /
        // render), so it can be compared without a session file. Applied unconditionally (not just
        // the `--render` path) since `--profile`/`--benchmark` don't call `apply_cli_render`;
        // `--no-bla` wins if both are given.
        if args.iter().any(|a| a == "--bla") {
            app.render_cfg.use_bla = true;
        }
        if args.iter().any(|a| a == "--no-bla") {
            app.render_cfg.use_bla = false;
        }
        if args.iter().any(|a| a == "--no-watermark") {
            app.watermark = false;
        }
        if args.iter().any(|a| a == "--watermark") {
            app.watermark = true;
        }
        if args.iter().any(|a| a == "--glitch") {
            app.render_cfg.glitch_correct = true;
        }
        if args.iter().any(|a| a == "--no-glitch") {
            app.render_cfg.glitch_correct = false;
        }
        if app.auto_benchmark {
            app.start_benchmark();
        }
        if app.auto_render {
            app.apply_cli_render(args);
        }
        // `--import-kfr FILE`: load a Kalles Fraktaler location at startup (and before any
        // `--render`), so it works both live and headless.
        if let Some(p) = args
            .iter()
            .position(|a| a == "--import-kfr")
            .and_then(|i| args.get(i + 1))
        {
            match app.load_kfr_file(std::path::Path::new(p)) {
                Ok(m) => println!("{m}"),
                Err(e) => eprintln!("--import-kfr: {e}"),
            }
        }
        app.nav.undo.push(app.snapshot_view()); // baseline for navigation undo
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
        // --size: a bare width (`1920`) or `WIDTHxHEIGHT` (`5120x2160`). Set the viewport dimensions
        // *before* the center/zoom below — magnification is defined relative to the viewport height —
        // and drive the export width. A bare width keeps the current aspect.
        let (size_w, size_h) = val("--size").map(|s| parse_size(s)).unwrap_or((None, None));
        if let Some(w) = size_w {
            let w = w.clamp(16, 16384);
            self.export.width = w;
            let h = size_h.map(|h| h.clamp(16, 16384)).unwrap_or_else(|| {
                ((w as f64) * self.viewport.height_px / self.viewport.width_px.max(1.0))
                    .round()
                    .clamp(16.0, 16384.0) as u32
            });
            self.viewport.width_px = w as f64;
            self.viewport.height_px = h as f64;
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
        // `--zoom` is f64 (≤ ~1e308×); `--zoom-log2 L` sets magnification = 2^L for
        // arbitrary depth past the f64 range (e.g. L=1100 ≈ 1e331×).
        if let Some(l) = val("--zoom-log2").and_then(|s| s.parse::<f64>().ok()) {
            self.viewport.set_center_log2mag(cx, cy, l);
        } else {
            let zoom = val("--zoom").and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0);
            self.viewport.set_center_mag(cx, cy, zoom.max(1.0e-300));
        }
        if let Some(ss) = val("--ss").and_then(|s| s.parse::<u32>().ok()) {
            self.export.ss = ss.clamp(1, 8);
        }
        if let Some(it) = val("--iter").and_then(|s| s.parse::<u32>().ok()) {
            self.render_cfg.max_iter = it.clamp(16, 200_000);
            self.render_cfg.auto_iter = false;
        }
        if let Some(p) = val("--palette").and_then(|s| s.parse::<usize>().ok()) {
            self.palette_idx = p.min(fractadyne_color::PRESETS.len() - 1);
            // A preset overrides any persisted binary/duotone/custom palette.
            self.use_binary = false;
            self.use_duotone = false;
            self.use_custom_palette = false;
        }
        if args.iter().any(|a| a == "--binary") {
            self.use_binary = true;
            self.use_duotone = false;
            self.use_custom_palette = false;
        }
        if args.iter().any(|a| a == "--light") {
            self.effects.light = true;
        }
        if let Some(a) = val("--light-angle").and_then(|s| s.parse::<f32>().ok()) {
            self.effects.light_angle = a;
        }
        if args.iter().any(|a| a == "--de") {
            self.effects.de = true;
        }
        if let Some(m) = val("--method") {
            self.color_method = ColorMethod::from_key(m);
        }
        if let Some(f) = val("--stripe-freq").and_then(|s| s.parse::<f32>().ok()) {
            self.stripe_freq = f.clamp(1.0, 24.0);
        }
        if let Some(t) = val("--trap") {
            self.trap_type = TrapType::from_key(t);
        }
        // Output format from the file extension.
        if let Some(out) = &self.auto_render_out {
            if out.extension().and_then(|e| e.to_str()) == Some("exr") {
                self.export.format = ExportFormat::Exr;
            } else {
                self.export.format = ExportFormat::Png;
            }
        }
    }

    /// Snapshot current state and save it ~1 s after the last change (or on close).
    fn autosave(&mut self, ctx: &egui::Context) {
        // After a state reset the user chose not to persist this session — don't recreate the
        // file we just deleted (on the idle timer or on close).
        if self.suppress_autosave {
            return;
        }
        let cur = fractadyne_state::SessionState {
            state_version: fractadyne_state::STATE_FORMAT_VERSION,
            center_x: fractadyne_core::to_f64(&self.viewport.center_x),
            center_y: fractadyne_core::to_f64(&self.viewport.center_y),
            center_x_str: fractadyne_core::to_decimal_string(&self.viewport.center_x),
            center_y_str: fractadyne_core::to_decimal_string(&self.viewport.center_y),
            units_per_pixel: self.viewport.units_per_pixel.m,
            units_per_pixel_e: self.viewport.units_per_pixel.e,
            max_iter: self.render_cfg.max_iter,
            auto_iter: self.render_cfg.auto_iter,
            palette_idx: self.palette_idx,
            cycle: self.cycle,
            offset: self.offset,
            zoom_rate: self.render_cfg.zoom_rate,
            autopilot_dive_log2: self.autopilot.dive_log2,
            work_budget_scale: self.render_cfg.work_budget_scale,
            aa: self.render_cfg.aa,
            fps_cap: self.fps_cap.unwrap_or(0.0), // None (uncapped) → 0, so it round-trips
            export_width: self.export.width,
            export_ss: self.export.ss,
            export_format: match self.export.format {
                ExportFormat::Png => "png".to_string(),
                ExportFormat::Exr => "exr".to_string(),
            },
            export_dir: self
                .export.last_dir
                .as_ref()
                .map(|p| p.display().to_string()),
            export_dual_mode: match self.export.dual_mode {
                DualExport::SideBySide => "side".to_string(),
                DualExport::Separate => "separate".to_string(),
                DualExport::ActiveOnly => "active".to_string(),
            },
            export_aspect: self.export.aspect.clone(),
            show_location: self.show_location,
            palette_anim: self.palette_anim.key().to_string(),
            palette_anim_speed: self.palette_anim_speed,
            light: self.effects.light,
            light_angle: self.effects.light_angle,
            light_height: self.effects.light_height,
            light_anim: self.effects.light_anim,
            de: self.effects.de,
            de_strength: self.effects.de_strength,
            de_width: self.effects.de_width,
            de_anim: self.effects.de_anim,
            color_method: self.color_method.key().to_string(),
            stripe_freq: self.stripe_freq,
            trap_type: self.trap_type.key().to_string(),
            minimap: self.minimap,
            custom_palette: self.custom_palette.clone(),
            use_custom_palette: self.use_custom_palette,
            use_duotone: self.use_duotone,
            use_binary: self.use_binary,
            duotone_lo: self.duotone_lo,
            duotone_hi: self.duotone_hi,
            right_panel_open: self.right_panel_open,
            fractal: self.fractal.name().to_string(),
            julia_mode: self.julia_mode,
            julia_c_re: self.julia_c.0,
            julia_c_im: self.julia_c.1,
            dual: self.dual,
            dual_split: self.dual_split,
            series_approx: self.render_cfg.series_approx,
            glitch_correct: self.render_cfg.glitch_correct,
            use_bla: self.render_cfg.use_bla,
            watermark: self.watermark,
            ui_scale: self.ui_scale,
            show_orbits: self.show_orbits,
            orbit_normalize: self.orbit_normalize,
            orbit_anim: self.orbit_anim,
            orbit_anim_speed: self.orbit_anim_speed,
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
            self.dual = false; // no Julia counterpart → dual view is meaningless
        }
        let (cx, cy) = kind.default_center();
        self.viewport.reset();
        self.viewport.center_x = fractadyne_core::BigFloat::from_f64(cx, 64);
        self.viewport.center_y = fractadyne_core::BigFloat::from_f64(cy, 64);
        self.pointer.zoom_vel = 0.0;
        self.invalidate_refs(); // dynamics changed → drop the cached reference orbits
    }

    /// Drop both per-view reference caches (call when the formula/mode/center changes
    /// such that the cached references no longer apply).
    fn invalidate_refs(&mut self) {
        self.ref_cache[0].ref_pt = None;
        self.ref_cache[1].ref_pt = None;
        // Drop any in-flight recompute — its result is for the old fractal/mode and must not
        // install (would render the wrong formula until the next recompute).
        self.recompute_rx = [None, None];
    }

    /// Request the next animation frame. Frame pacing (the cap) is enforced at the end
    /// of `update`; this just keeps the animation loop alive.
    fn schedule_repaint(&self, ctx: &egui::Context) {
        ctx.request_repaint();
    }

    // compute_reference / series_skip_for / current_export_request_for moved to render.rs.

    /// Build the export job for the current state (single view, or dual per the chosen
    /// layout).
    /// Export image height (px) for the current width + aspect setting. "window" matches the live
    /// view's pixel aspect (same as the render); a fixed key uses that ratio. Uses the pixel aspect,
    /// not `complex_span` (which saturates to 0 past ~1e308× → a bogus 1-px height).
    fn export_height(&self) -> u32 {
        let ratio = if self.export.aspect == "window" {
            (self.viewport.width_px / self.viewport.height_px.max(1.0)).max(1.0e-6)
        } else {
            EXPORT_ASPECTS
                .iter()
                .find(|(k, _)| *k == self.export.aspect)
                .map(|(_, r)| *r)
                .unwrap_or(self.viewport.width_px / self.viewport.height_px.max(1.0))
        };
        ((self.export.width as f64) / ratio).round().max(1.0) as u32
    }

    fn build_export_job(&self) -> ExportJob {
        // Apply the chosen aspect: override the request height (the render centers the extra/fewer
        // rows on the same center; width stays `export_width`). For "window" this equals the height
        // the request already derived, so it's a no-op.
        let h = self.export_height();
        let fit = |mut req: fractadyne_gpu::ExportRequest| {
            req.height = h;
            // Keep the per-texel step isotropic (the GPU derives step = span/resolution per axis):
            // set the vertical span to match the horizontal step × the chosen height, so the fractal
            // isn't stretched when the aspect differs from the window. No-op for "Match window".
            req.span_mantissa[1] = req.span_mantissa[0] * (h as f64 / req.width.max(1) as f64);
            req
        };
        if self.dual {
            let map = fit(self.current_export_request_for(&self.viewport, false));
            let jul = fit(self.current_export_request_for(&self.julia_viewport, true));
            match self.export.dual_mode {
                DualExport::SideBySide => ExportJob::SideBySide(map, jul),
                DualExport::Separate => ExportJob::Separate(map, jul),
                DualExport::ActiveOnly => ExportJob::Single(map),
            }
        } else {
            ExportJob::Single(fit(self.current_export_request_for(&self.viewport, self.julia_mode)))
        }
    }

    /// Default Pictures directory (fallback: current dir).
    fn pictures_dir() -> std::path::PathBuf {
        directories::UserDirs::new()
            .and_then(|u| u.picture_dir().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."))
    }

    /// Path to the bookmarks file in the config dir (honours `FRACTADYNE_CONFIG_DIR`).
    fn bookmarks_path() -> Option<std::path::PathBuf> {
        fractadyne_state::config_dir().map(|d| d.join("bookmarks.toml"))
    }

    /// Load saved bookmarks (empty list if none / unreadable).
    fn load_bookmarks() -> Vec<Bookmark> {
        Self::bookmarks_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|t| toml::from_str::<BookmarkFile>(&t).ok())
            .map(|f| f.bookmark)
            .unwrap_or_default()
    }

    /// Persist bookmarks. A write failure loses durable user data, so it's surfaced as a toast
    /// (queued in `pending_toast` since this runs from contexts without an `egui::Context`).
    fn save_bookmarks(&mut self) {
        let Some(path) = Self::bookmarks_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let file = BookmarkFile {
            bookmark: self.bookmarks.clone(),
        };
        let result = match toml::to_string_pretty(&file) {
            Ok(text) => std::fs::write(&path, text),
            Err(e) => {
                self.pending_toast = Some(format!("Couldn't serialize bookmarks: {e}"));
                return;
            }
        };
        if let Err(e) = result {
            self.pending_toast = Some(format!("Couldn't save bookmarks: {e}"));
        }
    }

    /// Add a bookmark of the current view (auto-names it if `name` is blank).
    fn add_bookmark(&mut self, name: &str) {
        let name = if name.trim().is_empty() {
            format!(
                "{} {}×",
                self.fractal.name(),
                fmt_zoom_log2(self.viewport.log2_magnification())
            )
        } else {
            name.trim().to_string()
        };
        self.bookmarks.push(Bookmark {
            name,
            meta: self.view_metadata(),
            thumb: String::new(),
        });
        // Render the preview later this frame (GPU available in `update`, view unchanged).
        self.pending_thumb = Some(self.bookmarks.len() - 1);
        self.save_bookmarks();
    }

    /// Directory holding bookmark thumbnail PNGs (honours `FRACTADYNE_CONFIG_DIR`).
    fn bookmark_thumbs_dir() -> Option<std::path::PathBuf> {
        fractadyne_state::config_dir().map(|d| d.join("bookmark_thumbs"))
    }

    fn bookmark_thumb_path(id: &str) -> Option<std::path::PathBuf> {
        Self::bookmark_thumbs_dir().map(|d| d.join(format!("{id}.png")))
    }

    /// Render + save the pending bookmark's thumbnail (a small PNG of the current view). Called
    /// from `update` where the GPU device/queue are available.
    fn process_pending_thumb(&mut self, gpu: &Option<(eframe::wgpu::Device, eframe::wgpu::Queue)>) {
        let Some(i) = self.pending_thumb else { return };
        let Some((dev, q)) = gpu else { return };
        self.pending_thumb = None;
        if i >= self.bookmarks.len() {
            return;
        }
        // Small render of the current view (matches the just-saved bookmark).
        let w = 160u32;
        let h = ((w as f64) * self.viewport.height_px / self.viewport.width_px.max(1.0))
            .round()
            .clamp(60.0, 160.0) as u32;
        let mut req = self.current_export_request_for(&self.viewport, self.julia_mode);
        req.width = w;
        req.height = h;
        req.ss = 1;
        // Keep the zoom-appropriate iteration count (the reference orbit is already built at that
        // depth). Clamping this low renders deep views as all-interior — a solid black thumbnail.
        req.max_iter = req.max_iter.max(200);
        let progress = std::sync::atomic::AtomicU32::new(0);
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let Ok(res) = fractadyne_gpu::render_export(dev, q, &req, &progress, &cancel) else {
            return;
        };
        // A stable id from the current time (nanos) + index; write the PNG.
        let id = format!(
            "{}-{i}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        let Some(path) = Self::bookmark_thumb_path(&id) else { return };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if fractadyne_export::write_png(&path, res.width, res.height, &res.pixels, None).is_ok() {
            self.bookmarks[i].thumb = id;
            self.save_bookmarks();
        }
    }

    /// Fetch (lazily loading + caching) the texture for a bookmark thumbnail id.
    fn bookmark_thumb_texture(&mut self, ctx: &egui::Context, id: &str) -> Option<egui::TextureHandle> {
        if id.is_empty() {
            return None;
        }
        if let Some(tex) = self.thumb_cache.get(id) {
            return Some(tex.clone());
        }
        let path = Self::bookmark_thumb_path(id)?;
        let (w, h, rgba) = fractadyne_export::read_png_rgba8(&path).ok()?;
        let img = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
        let tex = ctx.load_texture(format!("bmthumb.{id}"), img, egui::TextureOptions::LINEAR);
        self.thumb_cache.insert(id.to_string(), tex.clone());
        Some(tex)
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

    // view_metadata / load_view_metadata / open_view moved to export.rs.
    /// Scan the gallery folder for exported PNG/EXR files with Fractadyne metadata,
    /// newest first. Thumbnails load lazily afterward.
    fn scan_gallery(&mut self) {
        self.gallery.entries.clear();
        let Ok(rd) = std::fs::read_dir(&self.gallery.dir) else {
            return;
        };
        for path in rd.flatten().map(|e| e.path()) {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase())
                .unwrap_or_default();
            let meta = match ext.as_str() {
                "png" => fractadyne_export::read_png_metadata(&path).ok().flatten(),
                "exr" => fractadyne_export::read_exr_metadata(&path).ok().flatten(),
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
            self.gallery.entries.push(GalleryEntry {
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
        self.gallery.entries
            .sort_by_key(|e| std::cmp::Reverse(e.saved_unix));
    }

    // export_ext / start_export / quick_export / render_to_file(_iter) / start_export_to moved to export.rs.

    // build_params moved to render.rs.

    /// Palette-cycle scaling for the GPU. The bounded statistical methods (stripe /
    /// triangle-inequality / decomposition) produce a 0..1 value, so they want a few
    /// cycles across the palette; the unbounded ones (iteration / trap / distance) use
    /// the fine per-unit scaling.
    fn color_cycle(&self) -> f32 {
        if self.color_method.needs_aux() {
            0.5 + self.cycle * 4.0
        } else {
            0.004 + self.cycle * 0.06
        }
    }

    /// The live-render work budget (`WORK_BUDGET`) scaled by the user's `work_budget_scale`. Higher
    /// lets deep/large frames render at fuller resolution (crisper) before the color pass falls back
    /// to a box-filtered upscale — at the cost of frame-rate and GPU-watchdog margin. Export is
    /// unaffected (always full resolution).
    pub(crate) fn effective_work_budget(&self) -> u64 {
        ((WORK_BUDGET as f64) * self.render_cfg.work_budget_scale.clamp(0.25, 8.0)).max(1.0e9) as u64
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
        let shift = ctx.input(|i| i.modifiers.shift);

        // Zoom box (Shift+drag): rubber-band a rectangle; on release, zoom so it fills the
        // panel. Handled before borrowing `vp` (uses the separate `self.pointer.zoom_box` field).
        if resp.drag_started() && shift {
            if let Some(p) = resp.interact_pointer_pos() {
                self.pointer.zoom_box = Some(ZoomBox { start: p, end: p, is_julia });
            }
        }
        let mut apply_zoom: Option<(f64, f64, f64)> = None; // (box_cx_px, box_cy_px, factor)
        let mut zoom_boxing = false;
        if self.pointer.zoom_box.as_ref().is_some_and(|z| z.is_julia == is_julia) {
            zoom_boxing = true;
            if let Some(cur) = resp.interact_pointer_pos() {
                self.pointer.zoom_box.as_mut().unwrap().end = cur;
            }
            let zb = self.pointer.zoom_box.as_ref().unwrap();
            let boxr = aspect_zoom_box(zb.start, zb.end, rect);
            // Foreground layer so the box draws above the fractal paint callback.
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("fract_zoom_box"),
            ));
            painter.rect_filled(
                boxr,
                egui::CornerRadius::ZERO,
                egui::Color32::from_rgba_unmultiplied(0xE0, 0xA0, 0x30, 32),
            );
            painter.rect_stroke(
                boxr,
                egui::CornerRadius::ZERO,
                egui::Stroke::new(1.5, BRAND_ACCENT),
                egui::StrokeKind::Inside,
            );
            if resp.drag_stopped() {
                // Ignore a tiny box (an accidental Shift-click).
                if boxr.width() > 6.0 && boxr.height() > 6.0 {
                    apply_zoom = Some((
                        (boxr.center().x - rect.min.x) as f64 * ppp,
                        (boxr.center().y - rect.min.y) as f64 * ppp,
                        (boxr.width() / rect.width()) as f64, // < 1 ⇒ zoom in
                    ));
                }
                self.pointer.zoom_box = None;
                zoom_boxing = false;
            }
        }

        let vp = if is_julia {
            &mut self.julia_viewport
        } else {
            &mut self.viewport
        };
        vp.set_size(rect.width() as f64 * ppp, rect.height() as f64 * ppp);
        if let Some((bcx, bcy, factor)) = apply_zoom {
            let (w, h) = (vp.width_px, vp.height_px);
            vp.pan_pixels(w * 0.5 - bcx, h * 0.5 - bcy); // box center → screen center
            vp.zoom_at(w * 0.5, h * 0.5, factor); // then zoom the box to fill
        }
        // Plain drag pans (unless we're dragging a zoom box).
        if !zoom_boxing && resp.dragged_by(egui::PointerButton::Primary) {
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
        // Drawing a zoom box drags the pointer but does NOT move the view, so it must not
        // count as "active" (that would drop the render back to the coarse moving preview).
        // Only the actual zoom application (apply_zoom) counts.
        let active = (resp.dragged() && !zoom_boxing)
            || apply_zoom.is_some()
            || (scroll != 0.0 && hovering)
            || (self.pointer.zoom_vel.abs() > 1e-3 && hovering);
        let view = is_julia as usize;
        if active {
            self.pointer.settle_t[view] = now;
        }
        let interacting = now - self.pointer.settle_t[view] < SETTLE_DELAY;

        let eff_iter = if self.render_cfg.auto_iter {
            vp.recommended_max_iter(self.render_cfg.max_iter)
        } else {
            self.render_cfg.max_iter
        };
        let center_bf = [vp.center_x.clone(), vp.center_y.clone()];
        let center = vp.center_f64();
        let span = vp.complex_span_fe();
        let mag = vp.magnification();
        let log2mag = vp.log2_magnification();
        // Progressive settle AA (after the `vp` borrow ends): coarse (1×) while moving, then refine
        // 1×→2×→4×→… up to the chosen level over consecutive frames, so a heavy view never blocks on
        // one full-AA frame.
        let aa_target = if interacting {
            self.pointer.settle_frame[view] = 0;
            1
        } else {
            let ss = aa_ramp(self.pointer.settle_frame[view], self.render_cfg.aa);
            if ss < self.render_cfg.aa {
                self.pointer.settle_frame[view] += 1;
                self.schedule_repaint(ctx); // render the next, sharper AA stage
            }
            ss
        };
        let res = [
            (rect.width() as f64 * ppp) as u32,
            (rect.height() as f64 * ppp) as u32,
        ];
        let view_id = if is_julia { 1 } else { 0 };
        // Pan reprojection: while dragging, keep the last detailed frame and just translate it
        // with the cursor; re-render (at settled quality) only once the drag stops and the view
        // settles. Without this the coarse moving preview shows no detail at deep zoom, so you
        // can't see what you're panning toward. `pan_px` accumulates the same device-pixel
        // deltas fed to `pan_pixels`, so the slide matches the eventual settled render exactly.
        if resp.drag_started_by(egui::PointerButton::Primary) && !zoom_boxing {
            self.pointer.pan_px = egui::Vec2::ZERO;
            self.pointer.pan_view = Some(view_id);
        }
        if self.pointer.pan_view == Some(view_id)
            && !zoom_boxing
            && resp.dragged_by(egui::PointerButton::Primary)
        {
            let d = resp.drag_delta();
            self.pointer.pan_px += egui::vec2(d.x * ppp as f32, d.y * ppp as f32);
        }
        let reproject = if self.pointer.pan_view == Some(view_id) && interacting {
            self.schedule_repaint(ctx); // keep rendering until the view settles
            Some([
                self.pointer.pan_px.x / res[0].max(1) as f32,
                self.pointer.pan_px.y / res[1].max(1) as f32,
            ])
        } else {
            if self.pointer.pan_view == Some(view_id) {
                self.pointer.pan_view = None; // settled → next frame does a full re-iterate
            }
            None
        };
        let params = self.build_params(
            center_bf,
            center,
            span,
            mag,
            log2mag,
            self.fractal,
            is_julia,
            eff_iter,
            interacting,
            aa_target,
            res,
            view_id,
            reproject,
        );
        add_mandelbrot(ui.painter(), rect, params);
        resp
    }

    /// Dual linked view: Mandelbrot (left) ↔ Julia (right). Hovering the Mandelbrot
    /// sets the Julia parameter `c`.
    fn draw_dual(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let ppp = ctx.pixels_per_point() as f64;
        let full = ui.max_rect();
        // Split position from the persisted fraction; a small gap between the panels holds the
        // draggable separator (handled at the end of this fn, so it draws on top).
        const HANDLE_W: f32 = 6.0;
        let mid = full.min.x + full.width() * self.dual_split.clamp(0.15, 0.85);
        let left = egui::Rect::from_min_max(full.min, egui::pos2(mid - HANDLE_W * 0.5, full.max.y));
        let right = egui::Rect::from_min_max(egui::pos2(mid + HANDLE_W * 0.5, full.min.y), full.max);
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
        let rate = ZOOM_RATE * self.render_cfg.zoom_rate as f64;
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
        self.pointer.zoom_vel += (target - self.pointer.zoom_vel) * ease;
        if target != 0.0 || self.pointer.zoom_vel.abs() > 1e-3 {
            self.schedule_repaint(ctx);
        }
        if self.pointer.zoom_vel.abs() > 1e-3 {
            if let Some((p, r, is_julia)) = panel {
                let l = p - r.min;
                let factor = (-self.pointer.zoom_vel * dt).exp();
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
                    self.pointer.settle_t[1] = ctx.input(|i| i.time); // only the Julia view changed
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
                self.pointer.settle_t[1] = ctx.input(|i| i.time); // only the Julia panel is changing
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

        // Orbit overlay on the hovered panel — or, during a tour, a scripted point on the Mandelbrot.
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
            } else if let Some((ox, oy)) = self.tour_orbit {
                let pr = self.viewport.precision;
                let cpx = self.viewport.complex_to_pixel(
                    &fractadyne_core::BigFloat::from_f64(ox, pr),
                    &fractadyne_core::BigFloat::from_f64(oy, pr),
                );
                let painter = ui.painter_at(left);
                self.draw_orbit(&painter, left, &self.viewport, cpx, false, ppp);
            }
        }

        // Draggable panel separator (fills the reserved gap at `mid`; drawn on top).
        let handle = egui::Rect::from_min_max(
            egui::pos2(mid - HANDLE_W * 0.5, full.min.y),
            egui::pos2(mid + HANDLE_W * 0.5, full.max.y),
        );
        let sep = ui.interact(handle, ui.id().with("dual_split"), egui::Sense::drag());
        if sep.dragged() {
            if let Some(p) = sep.interact_pointer_pos() {
                self.dual_split = ((p.x - full.min.x) / full.width().max(1.0)).clamp(0.15, 0.85);
            }
        }
        if sep.hovered() || sep.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
        let sep_col = if sep.hovered() || sep.dragged() {
            BRAND_ACCENT
        } else {
            egui::Color32::from_gray(50)
        };
        ui.painter().rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(mid - 0.5, full.min.y),
                egui::pos2(mid + 0.5, full.max.y),
            ),
            egui::CornerRadius::ZERO,
            sep_col,
        );

        self.pointer.pointer_complex = pc;
    }

    /// Screen position (points) of a complex coordinate in the Mandelbrot viewport,
    /// within the given panel rect. Inverse of `complex_at_pixel_f64`.
    fn complex_screen_pos(&self, c: (f64, f64), rect: egui::Rect, ppp: f64) -> egui::Pos2 {
        let (cx, cy) = self.viewport.center_f64();
        let upp = self.viewport.units_per_pixel.to_f64();
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
            upp: vp.units_per_pixel.to_f64(),
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
            let upp = vp.units_per_pixel.to_f64();
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
        if p.last_sa_skip > 0 {
            ui.monospace(format!("SA skip    {:>7}", p.last_sa_skip));
        }
        ui.monospace(format!("aa         {}x", self.render_cfg.aa));
        ui.monospace(format!("dual       {}", self.dual));
        ui.monospace(format!("zoom       {}×", fmt_zoom_log2(self.viewport.log2_magnification())));

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
            let c_per_panel = self.viewport.width_px * self.viewport.units_per_pixel.to_f64();
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

    /// Snapshot the current location for navigation history.
    fn snapshot_view(&self) -> ViewSnapshot {
        ViewSnapshot {
            cx: self.viewport.center_x.clone(),
            cy: self.viewport.center_y.clone(),
            upp: self.viewport.units_per_pixel,
            prec: self.viewport.precision,
        }
    }

    /// Restore a navigation snapshot (location only).
    fn apply_snapshot(&mut self, s: &ViewSnapshot) {
        self.viewport.center_x = s.cx.clone();
        self.viewport.center_y = s.cy.clone();
        self.viewport.units_per_pixel = s.upp;
        self.viewport.precision = s.prec;
        self.pointer.zoom_vel = 0.0;
        self.invalidate_refs();
    }

    /// Record the current location onto the undo history (deduped vs. the top), and
    /// clear the redo stack. Called when the view settles and after discrete jumps.
    fn record_nav(&mut self) {
        let snap = self.snapshot_view();
        let dup = self.nav.undo.last().is_some_and(|t| {
            t.upp == snap.upp && t.cx == snap.cx && t.cy == snap.cy
        });
        if !dup {
            self.nav.undo.push(snap);
            if self.nav.undo.len() > 256 {
                self.nav.undo.remove(0);
            }
            self.nav.redo.clear();
        }
    }

    /// Step back / forward through visited locations.
    fn undo_view(&mut self) {
        if self.nav.undo.len() < 2 {
            return;
        }
        let cur = self.nav.undo.pop().unwrap();
        self.nav.redo.push(cur);
        let prev = self.nav.undo.last().unwrap().clone();
        self.apply_snapshot(&prev);
        self.nav.was_interacting = false;
    }
    fn redo_view(&mut self) {
        if let Some(s) = self.nav.redo.pop() {
            self.apply_snapshot(&s);
            self.nav.undo.push(s);
            self.nav.was_interacting = false;
        }
    }

    /// Open the go-to-location dialog, pre-filled with the current view.
    fn open_goto(&mut self) {
        self.goto.x = fractadyne_core::to_decimal_string(&self.viewport.center_x);
        self.goto.y = fractadyne_core::to_decimal_string(&self.viewport.center_y);
        self.goto.zoom = fmt_zoom_field(self.viewport.log2_magnification());
        self.goto.msg = None;
        self.goto.open = true;
    }

    /// Apply the go-to-location dialog: parse + validate, then jump (recording history).
    fn apply_goto(&mut self) {
        let cx = fractadyne_core::parse_bf(self.goto.x.trim());
        let cy = fractadyne_core::parse_bf(self.goto.y.trim());
        let log2mag = parse_zoom_to_log2(&self.goto.zoom);
        match (cx, cy, log2mag) {
            (Some(cx), Some(cy), Some(l)) if l.is_finite() => {
                // Clamp to a sane octave bound so a pasted absurd zoom can't request
                // runaway precision; well beyond any practical depth.
                self.viewport.set_center_log2mag(cx, cy, l.clamp(0.0, 1.0e6));
                self.pointer.zoom_vel = 0.0;
                self.invalidate_refs();
                self.record_nav();
                self.goto.msg = None;
                self.goto.open = false;
            }
            _ => {
                self.goto.msg = Some("Invalid input — check the coordinates and zoom.".into());
            }
        }
    }

    /// Set a transient status toast (auto-fades after a few seconds).
    fn set_toast(&mut self, msg: impl Into<String>, ctx: &egui::Context) {
        self.toast = Some((msg.into(), ctx.input(|i| i.time)));
    }

    /// "Zoom to center": find the nearby minibrot's exact nucleus (Newton-Raphson in
    /// arbitrary precision) and snap the view center to it, keeping the current zoom.
    /// Reports the period. Holomorphic families only (Mandelbrot / Multibrot).
    fn find_minibrot(&mut self, ctx: &egui::Context) {
        if !matches!(self.fractal.formula_id(), 0..=3) {
            self.set_toast(
                "Minibrot finder needs a holomorphic family (Mandelbrot / Multibrot).",
                ctx,
            );
            return;
        }
        let mag = self.viewport.magnification();
        let center = [self.viewport.center_x.clone(), self.viewport.center_y.clone()];
        let max_period =
            self.viewport.recommended_max_iter(self.render_cfg.max_iter).clamp(1_000, 100_000);
        match fractadyne_core::find_nucleus(&center, mag, self.fractal.formula_id(), max_period) {
            Some(n) => {
                self.viewport.set_center_mag(n.cx, n.cy, mag);
                self.viewport.precision =
                    fractadyne_core::precision_for_magnification(mag);
                self.pointer.zoom_vel = 0.0;
                self.invalidate_refs();
                self.record_nav();
                self.set_toast(format!("Snapped to period-{} minibrot center", n.period), ctx);
            }
            None => self.set_toast("No minibrot center found near the view center.", ctx),
        }
    }

    /// Render the static home-view thumbnail for the minimap (fixed complex region), as
    /// an egui image. Cheap (small, direct path); only called when the thumbnail key
    /// changes. Returns `None` if the GPU render fails.
    fn render_minimap_image(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
    ) -> Option<egui::ColorImage> {
        let mut vp = Viewport::new(MINIMAP_TW as f64, MINIMAP_TH as f64);
        vp.center_x = fractadyne_core::BigFloat::from_f64(MINIMAP_CX, 64);
        vp.center_y = fractadyne_core::BigFloat::from_f64(MINIMAP_CY, 64);
        vp.units_per_pixel = fractadyne_core::FloatExp::from_f64((2.0 * MINIMAP_HX) / MINIMAP_TW as f64);
        vp.precision = 64;
        let mut req = self.current_export_request_for(&vp, false);
        req.width = MINIMAP_TW;
        req.height = MINIMAP_TH;
        req.ss = 1;
        req.max_iter = req.max_iter.clamp(200, 600);
        let progress = std::sync::atomic::AtomicU32::new(0);
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let res = fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel).ok()?;
        // Linear RGBA f32 → sRGB u8 (approx gamma) for display.
        let n = (res.width * res.height) as usize;
        let mut pixels = Vec::with_capacity(n * 4);
        for i in 0..n {
            for k in 0..3 {
                let c = res.pixels[i * 4 + k].clamp(0.0, 1.0);
                pixels.push((c.powf(1.0 / 2.2) * 255.0 + 0.5) as u8);
            }
            pixels.push(255);
        }
        Some(egui::ColorImage::from_rgba_unmultiplied(
            [res.width as usize, res.height as usize],
            &pixels,
        ))
    }

    /// Refresh the cached minimap thumbnail if its key (formula / palette / method)
    /// changed. No-op when the minimap is hidden.
    fn update_minimap(
        &mut self,
        ctx: &egui::Context,
        gpu: &Option<(eframe::wgpu::Device, eframe::wgpu::Queue)>,
    ) {
        // Shown for single Mandelbrot-family views and in dual view (the left panel is the
        // Mandelbrot map); hidden only for a single Julia view, where a Mandelbrot overview
        // wouldn't correspond to the shown set.
        if !self.minimap || (self.julia_mode && !self.dual) {
            return;
        }
        // Key includes the palette identity (preset index or a sentinel) and a revision so
        // the thumbnail refreshes when the gradient / duotone colors change.
        let duo_hash = self
            .duotone_lo
            .iter()
            .chain(&self.duotone_hi)
            .fold(0u32, |a, &c| a.wrapping_mul(16_777_619) ^ c.to_bits());
        let (pal_idx, pal_rev) = if self.use_binary {
            (usize::MAX - 2, duo_hash)
        } else if self.use_duotone {
            (usize::MAX - 1, duo_hash)
        } else if self.use_custom_palette {
            (usize::MAX, self.palette_rev)
        } else {
            (self.palette_idx, 0)
        };
        let key = (self.fractal.formula_id(), pal_idx, self.color_method.to_u32(), pal_rev);
        if self.minimap_key == Some(key) && self.minimap_tex.is_some() {
            return;
        }
        if let Some((dev, q)) = gpu {
            if let Some(img) = self.render_minimap_image(dev, q) {
                let tex = ctx.load_texture("fractadyne.minimap", img, egui::TextureOptions::LINEAR);
                self.minimap_tex = Some(tex);
                self.minimap_key = Some(key);
            }
        }
    }

    /// Draw the minimap overlay (thumbnail + "you are here" marker + zoom depth), and
    /// handle click-to-jump. Anchored bottom-left, above the status bar.
    /// Draw the discreet "Fd" brand mark in the lower-right of the fractal area (live view). Uses
    /// the header font — F in the light brand text color, d in the amber accent — over a soft dark
    /// halo so it stays legible on any background. Exports rasterize the same mark (`render.rs`).
    fn draw_watermark(&self, ctx: &egui::Context, rect: egui::Rect) {
        let px = (rect.height() * 0.026).clamp(18.0, 34.0); // small (~20–30 px), discreet
        let mark = ctx.fonts(|f| {
            f.layout_job(theme::brand_mark_job(px, theme::BRAND_TEXT, theme::BRAND_ACCENT))
        });
        let halo = ctx.fonts(|f| {
            let c = egui::Color32::from_black_alpha(120);
            f.layout_job(theme::brand_mark_job(px, c, c))
        });
        let sz = mark.size();
        let pos = egui::pos2(rect.right() - sz.x - 12.0, rect.bottom() - sz.y - 10.0);
        let painter =
            ctx.layer_painter(egui::LayerId::new(egui::Order::Middle, egui::Id::new("fd_watermark")));
        // Soft glow: the dark mark stamped at a ring of small offsets, then the colored mark on top.
        for off in [
            egui::vec2(1.2, 0.0), egui::vec2(-1.2, 0.0), egui::vec2(0.0, 1.2), egui::vec2(0.0, -1.2),
            egui::vec2(1.0, 1.0), egui::vec2(-1.0, 1.0), egui::vec2(1.0, -1.0), egui::vec2(-1.0, -1.0),
        ] {
            painter.galley(pos + off, halo.clone(), egui::Color32::PLACEHOLDER);
        }
        painter.galley(pos, mark, egui::Color32::PLACEHOLDER);
    }

    fn draw_minimap(&mut self, ctx: &egui::Context) {
        if !self.minimap || (self.julia_mode && !self.dual) {
            return;
        }
        let Some(tex) = self.minimap_tex.clone() else { return };
        let disp_w = 196.0_f32;
        let disp_h = disp_w * MINIMAP_TH as f32 / MINIMAP_TW as f32;
        let mut jump: Option<(f64, f64)> = None;
        egui::Area::new(egui::Id::new("fractadyne.minimap"))
            .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(10.0, -34.0))
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .stroke(egui::Stroke::new(1.0, BRAND_ACCENT))
                    .show(ui, |ui| {
                        let (rect, resp) = ui.allocate_exact_size(
                            egui::vec2(disp_w, disp_h),
                            egui::Sense::click(),
                        );
                        let p = ui.painter_at(rect);
                        p.image(
                            tex.id(),
                            rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE,
                        );
                        // "You are here": project the current center onto the thumbnail.
                        // Clamp to the thumbnail so the marker is ALWAYS visible (at the edge
                        // if the view sits outside the minimap's fixed region), and draw it
                        // with a dark halo so it reads on any background.
                        let (cx, cy) = self.viewport.center_f64();
                        let nx = ((cx - MINIMAP_CX) / (2.0 * MINIMAP_HX) + 0.5).clamp(0.0, 1.0) as f32;
                        let ny = (0.5 - (cy - MINIMAP_CY) / (2.0 * MINIMAP_HY)).clamp(0.0, 1.0) as f32;
                        let mk = rect.min + egui::vec2(nx * rect.width(), ny * rect.height());
                        let (sx, _sy) = self.viewport.complex_span();
                        let rw = (sx / (2.0 * MINIMAP_HX)) as f32 * rect.width();
                        let amber = BRAND_ACCENT;
                        let halo = egui::Color32::from_black_alpha(190);
                        if rw >= 6.0 {
                            // Shallow: draw the actual view rectangle (auto-clipped to the map).
                            let rh = rw * rect.height() / rect.width();
                            let vr = egui::Rect::from_center_size(mk, egui::vec2(rw, rh));
                            p.rect_stroke(vr, 0.0, egui::Stroke::new(3.0, halo), egui::StrokeKind::Middle);
                            p.rect_stroke(vr, 0.0, egui::Stroke::new(1.5, amber), egui::StrokeKind::Middle);
                        } else {
                            // Deep: a bright crosshair + centre dot (the view is sub-pixel here).
                            let c = 8.0;
                            let cross = |w: f32, col: egui::Color32, p: &egui::Painter| {
                                p.line_segment([mk - egui::vec2(c, 0.0), mk + egui::vec2(c, 0.0)], egui::Stroke::new(w, col));
                                p.line_segment([mk - egui::vec2(0.0, c), mk + egui::vec2(0.0, c)], egui::Stroke::new(w, col));
                            };
                            cross(3.5, halo, &p);
                            p.circle_stroke(mk, 5.0, egui::Stroke::new(3.5, halo));
                            cross(1.5, amber, &p);
                            p.circle_stroke(mk, 5.0, egui::Stroke::new(1.5, amber));
                            p.circle_filled(mk, 2.0, amber);
                        }
                        // Zoom-depth label.
                        p.text(
                            rect.left_bottom() + egui::vec2(4.0, -3.0),
                            egui::Align2::LEFT_BOTTOM,
                            format!("{}×", fmt_zoom_log2(self.viewport.log2_magnification())),
                            egui::FontId::monospace(11.0),
                            BRAND_TEXT,
                        );
                        if resp.clicked() {
                            if let Some(pos) = resp.interact_pointer_pos() {
                                let fx = ((pos.x - rect.min.x) / rect.width()) as f64;
                                let fy = ((pos.y - rect.min.y) / rect.height()) as f64;
                                let tx = MINIMAP_CX + (fx - 0.5) * 2.0 * MINIMAP_HX;
                                let ty = MINIMAP_CY - (fy - 0.5) * 2.0 * MINIMAP_HY;
                                jump = Some((tx, ty));
                            }
                        }
                        resp.on_hover_text(
                            "Overview / you-are-here. Click to jump to that region (home zoom).",
                        );
                    });
            });
        if let Some((tx, ty)) = jump {
            self.viewport.set_center_mag(
                fractadyne_core::BigFloat::from_f64(tx, 64),
                fractadyne_core::BigFloat::from_f64(ty, 64),
                1.0,
            );
            self.pointer.zoom_vel = 0.0;
            self.invalidate_refs();
            self.record_nav();
        }
    }

    /// The custom-gradient editor window: live gradient preview, per-stop color + position
    /// controls, add/remove, and seed-from-preset. Edits bump `palette_rev`.
    fn palette_editor_window(&mut self, ctx: &egui::Context) {
        if !self.palette_editor_open {
            return;
        }
        let mut open = self.palette_editor_open;
        let mut changed = false;
        egui::Window::new("Gradient editor")
            .open(&mut open)
            .resizable(false)
            .default_width(340.0)
            .show(ctx, |ui| {
                // Live gradient preview bar.
                let (packed, n) = self.pack_custom();
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(ui.available_width(), 26.0), egui::Sense::hover());
                let pr = ui.painter_at(rect);
                let steps = rect.width().ceil().max(1.0) as usize;
                for s in 0..steps {
                    let t = s as f32 / steps as f32;
                    let x = rect.min.x + t * rect.width();
                    let col = sample_stops(&packed, n, t);
                    pr.line_segment(
                        [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                        egui::Stroke::new(1.5, col),
                    );
                }
                pr.rect_stroke(
                    rect,
                    2.0,
                    egui::Stroke::new(1.0, BRAND_ACCENT),
                    egui::StrokeKind::Inside,
                );
                ui.add_space(6.0);

                // Per-stop rows (color + position + remove).
                let mut remove: Option<usize> = None;
                let count = self.custom_palette.len();
                for i in 0..count {
                    ui.horizontal(|ui| {
                        let mut rgb = [
                            self.custom_palette[i][1],
                            self.custom_palette[i][2],
                            self.custom_palette[i][3],
                        ];
                        if ui.color_edit_button_rgb(&mut rgb).changed() {
                            self.custom_palette[i][1] = rgb[0];
                            self.custom_palette[i][2] = rgb[1];
                            self.custom_palette[i][3] = rgb[2];
                            changed = true;
                        }
                        let mut pos = self.custom_palette[i][0];
                        if ui
                            .add(egui::Slider::new(&mut pos, 0.0..=1.0).text("pos").fixed_decimals(3))
                            .changed()
                        {
                            self.custom_palette[i][0] = pos.clamp(0.0, 1.0);
                            changed = true;
                        }
                        if count > 2 && ui.button("✕").on_hover_text("Remove stop").clicked() {
                            remove = Some(i);
                        }
                    });
                }
                if let Some(i) = remove {
                    self.custom_palette.remove(i);
                    changed = true;
                }

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if self.custom_palette.len() < fractadyne_color::MAX_STOPS
                        && ui.button("➕ Add stop").clicked()
                    {
                        self.custom_palette.push([0.5, 1.0, 1.0, 1.0]);
                        changed = true;
                    }
                    ui.menu_button("Copy preset…", |ui| {
                        for (i, p) in fractadyne_color::PRESETS.iter().enumerate() {
                            if ui.button(p.name).clicked() {
                                self.custom_palette = self.preset_as_stops(i);
                                changed = true;
                                ui.close_menu();
                            }
                        }
                    });
                });
                ui.label(
                    egui::RichText::new(format!(
                        "{}/{} stops · positions may overlap; they're sorted automatically.",
                        self.custom_palette.len(),
                        fractadyne_color::MAX_STOPS
                    ))
                    .weak()
                    .small(),
                );
            });
        if changed {
            self.palette_rev = self.palette_rev.wrapping_add(1);
            self.use_custom_palette = true;
        }
        self.palette_editor_open = open;
    }

    /// Jump to a Mandelbrot location (full-precision center strings + magnification),
    /// e.g. a famous-locations entry. Switches to Mandelbrot, single view.
    fn goto_location(&mut self, cx: &str, cy: &str, mag: f64, name: &str, ctx: &egui::Context) {
        let (Some(x), Some(y)) =
            (fractadyne_core::parse_bf(cx), fractadyne_core::parse_bf(cy))
        else {
            return;
        };
        self.fractal = FractalKind::Mandelbrot;
        self.julia_mode = false;
        self.viewport.set_center_mag(x, y, mag.max(1.0));
        self.viewport.precision = fractadyne_core::precision_for_magnification(mag);
        self.pointer.zoom_vel = 0.0;
        self.invalidate_refs();
        self.record_nav();
        self.set_toast(format!("{name} · {}×", fmt_zoom(mag)), ctx);
    }

    /// Jump to a random interesting location: find a point on the set boundary by
    /// bisecting between an interior anchor and a random exterior direction, then zoom in
    /// a random amount. Boundary points are always detail-rich.
    fn random_location(&mut self, ctx: &egui::Context) {
        let mut s = (ctx.input(|i| i.time).to_bits() ^ 0x9E37_79B9_7F4A_7C15) | 1;
        let mut rnd = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            (s >> 11) as f64 / ((1u64 << 53) as f64)
        };
        let theta = rnd() * std::f64::consts::TAU;
        // Interior anchor (inside the main cardioid) → exterior point along θ.
        let (mut ix, mut iy) = (-0.5_f64, 0.0_f64);
        let (mut ox, mut oy) = (ix + 3.0 * theta.cos(), iy + 3.0 * theta.sin());
        for _ in 0..64 {
            let (mx, my) = ((ix + ox) * 0.5, (iy + oy) * 0.5);
            if mandel_escapes(mx, my, 3000).is_some() {
                ox = mx;
                oy = my;
            } else {
                ix = mx;
                iy = my;
            }
        }
        let (cx, cy) = ((ix + ox) * 0.5, (iy + oy) * 0.5);
        let mag = 10f64.powf(2.0 + rnd() * 4.0); // 1e2 .. 1e6
        self.fractal = FractalKind::Mandelbrot;
        self.julia_mode = false;
        self.viewport.set_center_mag(
            fractadyne_core::BigFloat::from_f64(cx, 64),
            fractadyne_core::BigFloat::from_f64(cy, 64),
            mag,
        );
        self.viewport.precision = fractadyne_core::precision_for_magnification(mag);
        self.pointer.zoom_vel = 0.0;
        self.invalidate_refs();
        self.record_nav();
        self.set_toast(format!("Random location · {}×", fmt_zoom(mag)), ctx);
    }

    /// The Help window: a left-hand table of contents + a scrollable content pane.
    fn help_window(&mut self, ctx: &egui::Context) {
        if !self.help_open {
            return;
        }
        const SECTIONS: [&str; 11] = [
            "Overview",
            "Navigation",
            "Coloring & options",
            "Fractals",
            "How it works",
            "Command line",
            "Shortcuts",
            "Recommended hardware",
            "Acknowledgments",
            "Licenses",
            "About",
        ];
        let mut open = self.help_open;
        // Cap the size to the screen so the content ScrollArea scrolls (rather than the window
        // growing to fit) and the window can't be resized past the screen edge (which pushed
        // the title-bar close button off-screen).
        let max_h = (ctx.screen_rect().height() - 80.0).max(360.0);
        let max_w = (ctx.screen_rect().width() - 40.0).max(480.0);
        egui::Window::new("Fractadyne Help")
            .open(&mut open)
            .default_size([800.0, 560.0])
            .min_width(480.0) // keep room for the content beside the fixed-width contents list
            .min_height(300.0)
            .max_width(max_w)
            .max_height(max_h)
            .constrain(true) // keep the whole window on-screen
            .resizable(true)
            .show(ctx, |ui| {
                // Manual two-column split (fixed contents list + scrollable content pane).
                // Explicit widths make the content both wrap AND fill the window, so the
                // window's resize grip stays at the true bottom-right — nested SidePanel/
                // CentralPanel mis-reported the width and stranded the grip beside the list.
                let toc_w = 165.0_f32;
                let avail = ui.available_size();
                ui.horizontal_top(|ui| {
                    // Contents list (fixed width).
                    ui.allocate_ui_with_layout(
                        egui::vec2(toc_w, avail.y),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_min_width(toc_w);
                            ui.set_max_width(toc_w);
                            ui.add_space(4.0);
                            for (i, name) in SECTIONS.iter().enumerate() {
                                ui.selectable_value(&mut self.help_section, i, *name);
                            }
                        },
                    );
                    ui.separator();
                    ui.add_space(8.0); // left inset so the separator doesn't touch the text
                    let content_w = ui.available_width().max(240.0);
                    ui.allocate_ui_with_layout(
                        egui::vec2(content_w, avail.y),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_min_width(content_w);
                            ui.set_max_width(content_w);
                            // Solid, only-when-needed scrollbar (not floating/hover-only).
                            ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .scroll_bar_visibility(
                                    egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                                )
                                .show(ui, |ui| {
                                    ui.set_max_width(content_w - 18.0); // leave room for the bar
                                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                                    match self.help_section {
                                        0 => help_overview(ui),
                                        1 => help_navigation(ui),
                                        2 => help_options(ui),
                                        3 => help_fractals(ui),
                                        4 => help_methodology(ui),
                                        5 => help_command_line(ui),
                                        6 => help_shortcuts(ui),
                                        7 => help_hardware(ui),
                                        8 => help_acknowledgments(ui),
                                        9 => help_licenses(ui),
                                        _ => help_about(ui),
                                    }
                                });
                        },
                    );
                });
            });
        self.help_open = open;
    }

    /// Load a Kalles Fraktaler `.kfr` location file and jump to it. Defensive: bounds the
    /// file size and delegates to the hardened `parse_kfr`. (KF's zoom and ours are both
    /// linear magnification from the home view — close enough that the location lands at
    /// essentially the right place and scale.)
    fn load_kfr_file(&mut self, path: &std::path::Path) -> Result<String, String> {
        let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
        if meta.len() > 4_000_000 {
            return Err("file too large (not a .kfr location?)".into());
        }
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let v = fractadyne_core::parse_kfr(&text)
            .ok_or("not a valid .kfr location (need Re / Im / Zoom)")?;
        let zoom = v.zoom;
        self.fractal = FractalKind::Mandelbrot;
        self.julia_mode = false;
        if let Some(it) = v.iterations {
            self.render_cfg.max_iter = it.clamp(64, 50_000);
            self.render_cfg.auto_iter = false;
        }
        self.viewport.set_center_mag(v.cx, v.cy, zoom.max(1.0));
        self.viewport.precision = fractadyne_core::precision_for_magnification(zoom);
        self.pointer.zoom_vel = 0.0;
        self.invalidate_refs();
        self.record_nav();
        Ok(format!("Imported .kfr location @ {}×", fmt_zoom(zoom)))
    }

    /// Open the Share-location dialog, pre-filled with the current view as `.fdn` text.
    fn open_share(&mut self) {
        self.share.text = self.view_metadata();
        self.share.msg = None;
        self.share.open = true;
    }

    /// Apply the Share dialog's text as a location (hardened: bounded, allow-list parse via
    /// `load_view_metadata`, every field validated/clamped — no paths or code).
    fn apply_share_text(&mut self, ctx: &egui::Context) {
        let t = self.share.text.trim();
        if t.is_empty() || t.len() > SHARE_MAX {
            self.share.msg = Some("Nothing to load (or text too large).".into());
            return;
        }
        // Must look like a Fractadyne location (has our app tag or a center field).
        if meta_get(t, "app") != "Fractadyne" && meta_get(t, "center_x").is_empty() {
            self.share.msg = Some("Not a Fractadyne location.".into());
            return;
        }
        let t = t.to_string();
        let report = self.load_view_metadata(&t); // performs the jump + records history
        let zoom = fmt_zoom_log2(self.viewport.log2_magnification());
        match report.note() {
            None => {
                self.set_toast(format!("Loaded location @ {zoom}×"), ctx);
                self.share.open = false;
            }
            // Keep the dialog open and surface the report rather than silently jumping.
            Some(n) => {
                self.share.msg = Some(format!("Loaded @ {zoom}× — {n}."));
            }
        }
    }

    /// Save the Share dialog's text to a `.fdn` file.
    fn save_share_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Fractadyne location", &["fdn"])
            .set_file_name("location.fdn")
            .save_file()
        {
            match std::fs::write(&path, self.share.text.as_bytes()) {
                Ok(()) => self.share.msg = Some("Saved.".into()),
                Err(e) => self.share.msg = Some(format!("Save failed: {e}")),
            }
        }
    }

    /// Load a `.fdn` file into the Share dialog's text box (size-bounded; not auto-applied,
    /// so the user can review before jumping).
    fn load_share_file(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Fractadyne location", &["fdn"])
            .pick_file()
        else {
            return;
        };
        match std::fs::metadata(&path) {
            Ok(m) if (m.len() as usize) <= SHARE_MAX => match std::fs::read_to_string(&path) {
                Ok(t) => {
                    self.share.text = t;
                    self.share.msg = Some("Loaded into the box — review, then Apply.".into());
                }
                Err(e) => self.share.msg = Some(format!("Read failed: {e}")),
            },
            Ok(_) => self.share.msg = Some("File too large (not a .fdn location?).".into()),
            Err(e) => self.share.msg = Some(format!("Read failed: {e}")),
        }
    }

    /// File-dialog import of a Kalles Fraktaler `.kfr` location.
    fn import_kfr(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Kalles Fraktaler location", &["kfr"])
            .pick_file()
        else {
            return;
        };
        match self.load_kfr_file(&path) {
            Ok(m) => self.set_toast(m, ctx),
            Err(e) => self.set_toast(format!("Import failed: {e}"), ctx),
        }
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
        self.pointer.zoom_vel = 0.0;
        self.invalidate_refs();
        self.record_nav();
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
        self.pointer.zoom_vel = 0.0;
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
        self.pointer.settle_t = [now; 2];
        self.home_anim = Some(anim);
        true
    }

    // Autopilot (toggle_autopilot / autopilot_step / autopilot_pick_target) moved to autopilot.rs.

    /// Stops uploaded to the GPU: the morphing random gradient when in Random mode,
    /// otherwise the selected preset.
    fn active_stops(&self) -> ([[f32; 4]; fractadyne_color::MAX_STOPS], u32) {
        if self.palette_anim == PaletteAnim::Random {
            self.random_palette.current()
        } else if self.use_binary {
            // Flat exterior: a single stop of the `hi` color (interior uses `lo`).
            let mut out = [[0.0f32; 4]; fractadyne_color::MAX_STOPS];
            out[0] = [self.duotone_hi[0], self.duotone_hi[1], self.duotone_hi[2], 0.0];
            (out, 1)
        } else if self.use_duotone {
            // Smooth two-color ramp lo → hi → lo (seamless under cycling).
            let (lo, hi) = (self.duotone_lo, self.duotone_hi);
            let mut out = [[0.0f32; 4]; fractadyne_color::MAX_STOPS];
            out[0] = [lo[0], lo[1], lo[2], 0.0];
            out[1] = [hi[0], hi[1], hi[2], 0.5];
            out[2] = [lo[0], lo[1], lo[2], 1.0];
            (out, 3)
        } else if self.use_custom_palette {
            self.pack_custom()
        } else {
            fractadyne_color::PRESETS[self.palette_idx].packed()
        }
    }

    /// In-set (interior) color for the GPU. Binary/duotone use the chosen `lo` color so the
    /// set reads as one solid color; otherwise the default near-black.
    fn interior_color(&self) -> [f32; 4] {
        if self.use_binary || self.use_duotone {
            [self.duotone_lo[0], self.duotone_lo[1], self.duotone_lo[2], 1.0]
        } else {
            [0.02, 0.02, 0.03, 1.0]
        }
    }

    /// Pack the custom gradient into the GPU stop format `[r, g, b, pos]` (sorted by
    /// position, count clamped to `MAX_STOPS`). Falls back to a preset if empty.
    fn pack_custom(&self) -> ([[f32; 4]; fractadyne_color::MAX_STOPS], u32) {
        if self.custom_palette.is_empty() {
            return fractadyne_color::PRESETS[self.palette_idx].packed();
        }
        let mut stops = self.custom_palette.clone();
        stops.sort_by(|a, b| a[0].total_cmp(&b[0]));
        let n = stops.len().clamp(1, fractadyne_color::MAX_STOPS);
        let mut out = [[0.0f32; 4]; fractadyne_color::MAX_STOPS];
        for (i, slot) in out.iter_mut().enumerate() {
            let s = stops[i.min(n - 1)];
            *slot = [s[1], s[2], s[3], s[0]];
        }
        (out, n as u32)
    }

    /// The given preset's stops as editable `[pos, r, g, b]` rows (to seed the editor).
    fn preset_as_stops(&self, idx: usize) -> Vec<[f32; 4]> {
        fractadyne_color::PRESETS[idx.min(fractadyne_color::PRESETS.len() - 1)]
            .stops
            .iter()
            .map(|(pos, c)| [*pos, c[0], c[1], c[2]])
            .collect()
    }

    /// Advance the palette animation for this frame (offset shift, or random morph).
    fn advance_palette_anim(&mut self, ctx: &egui::Context) {
        let dt = (ctx.input(|i| i.stable_dt) as f64).clamp(0.0, 0.1) as f32;
        // Distance-estimate glow cycling — flows the contour bands (independent of the
        // palette animation; shares the Speed slider). Phase is in cycles, period 1.
        if self.effects.de && self.effects.de_anim && self.palette_anim_speed > 0.0 {
            self.effects.de_phase = (self.effects.de_phase + self.palette_anim_speed * dt).rem_euclid(1.0);
            self.schedule_repaint(ctx);
        }
        // Rotate the relief light direction (cheap — it's a color-pass param).
        if self.effects.light && self.effects.light_anim && self.palette_anim_speed > 0.0 {
            self.effects.light_angle = (self.effects.light_angle
                + self.palette_anim_speed * dt * std::f32::consts::TAU)
                .rem_euclid(std::f32::consts::TAU);
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

    // start_benchmark / load_script / advance_playback / format_bench moved to scripting.rs.

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
        if !self.fractal.supports_julia() {
            self.dual = false; // no Julia → no dual (guards any non-UI caller)
            return;
        }
        self.dual = !self.dual;
        if self.dual {
            self.julia_viewport.reset();
            self.julia_viewport.center_x = fractadyne_core::BigFloat::from_f64(0.0, 64);
            self.julia_viewport.center_y = fractadyne_core::BigFloat::from_f64(0.0, 64);
        }
        self.invalidate_refs();
    }
}

// Modal dialogs, extracted from `update()` so the event loop reads as a sequence of
// `self.draw_*_dialog(ctx)` calls rather than a wall of inline `egui::Window` blocks.
// Each is a no-op unless its `*_open` flag is set. (REFACTOR-PLAN Phase 2b.)
impl FractadyneApp {
    /// "Go to location" dialog — jump to a pasted center/zoom, or copy the current one to share.
    fn draw_goto_dialog(&mut self, ctx: &egui::Context) {
        if !self.goto.open {
            return;
        }
        let mut open = self.goto.open;
        let mut go = false;
        let mut copy = false;
        egui::Window::new("Go to location")
            .open(&mut open)
            .resizable(false)
            .default_width(420.0)
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("Center X").weak().small());
                ui.add(egui::TextEdit::singleline(&mut self.goto.x).desired_width(f32::INFINITY));
                ui.label(egui::RichText::new("Center Y").weak().small());
                ui.add(egui::TextEdit::singleline(&mut self.goto.y).desired_width(f32::INFINITY));
                ui.label(egui::RichText::new("Zoom (magnification)").weak().small());
                ui.add(egui::TextEdit::singleline(&mut self.goto.zoom).desired_width(220.0));
                if let Some(m) = &self.goto.msg {
                    ui.colored_label(egui::Color32::from_rgb(0xE0, 0x6C, 0x60), m);
                }
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    go = ui.button("Go").clicked();
                    if ui.button("Copy").on_hover_text("Copy this location to the clipboard").clicked() {
                        copy = true;
                    }
                    if ui.button("Use current").clicked() {
                        self.goto.x = fractadyne_core::to_decimal_string(&self.viewport.center_x);
                        self.goto.y = fractadyne_core::to_decimal_string(&self.viewport.center_y);
                        self.goto.zoom = fmt_zoom_field(self.viewport.log2_magnification());
                        self.goto.msg = None;
                    }
                });
                ui.label(
                    egui::RichText::new("Paste a center/zoom from someone else, or Copy to share.")
                        .weak()
                        .small(),
                );
            });
        if copy {
            ctx.copy_text(format!(
                "center_x={}\ncenter_y={}\nzoom={}",
                self.goto.x, self.goto.y, self.goto.zoom
            ));
        }
        if go {
            self.apply_goto(); // clears goto_open on success
        }
        // Closed if the user hit the window's ✕ (open=false) or Go succeeded.
        self.goto.open = open && self.goto.open;
    }

    /// "Share location" (.fdn) dialog — copy/paste/apply/save/load a self-contained location.
    fn draw_share_dialog(&mut self, ctx: &egui::Context) {
        if !self.share.open {
            return;
        }
        let mut open = self.share.open;
        let (mut copy, mut apply, mut save, mut load) = (false, false, false, false);
        egui::Window::new("Share location")
            .open(&mut open)
            .resizable(true)
            .default_width(460.0)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(
                        "A self-contained location (fractal, full-precision center, zoom, \
                         coloring). Copy it to share, or paste/load one and Apply.",
                    )
                    .weak()
                    .small(),
                );
                ui.add_space(4.0);
                egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.share.text)
                            .desired_width(f32::INFINITY)
                            .desired_rows(10)
                            .code_editor(),
                    );
                });
                if let Some(m) = &self.share.msg {
                    ui.colored_label(egui::Color32::from_rgb(0xE0, 0xA0, 0x30), m);
                }
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    apply = ui.button("Apply").on_hover_text("Jump to the location in the box").clicked();
                    copy = ui.button("Copy").on_hover_text("Copy the text to the clipboard").clicked();
                    if ui.button("Use current").clicked() {
                        self.share.text = self.view_metadata();
                        self.share.msg = None;
                    }
                    save = ui.button("Save .fdn…").clicked();
                    load = ui.button("Load .fdn…").clicked();
                });
            });
        if copy {
            ctx.copy_text(self.share.text.clone());
            self.share.msg = Some("Copied to clipboard.".into());
        }
        if save {
            self.save_share_file();
        }
        if load {
            self.load_share_file();
        }
        if apply {
            self.apply_share_text(ctx); // clears share_open on success
        }
        self.share.open = open && self.share.open;
    }

    /// "Reset application state" confirmation dialog — permanently deletes all saved data.
    fn draw_reset_dialog(&mut self, ctx: &egui::Context) {
        if !self.reset_confirm_open {
            return;
        }
        let mut open = self.reset_confirm_open;
        let (mut confirm, mut cancel) = (false, false);
        egui::Window::new("Reset application state")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .default_width(460.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.colored_label(
                    egui::Color32::from_rgb(0xE0, 0x6C, 0x60),
                    "⚠  This permanently deletes all saved Fractadyne data:",
                );
                ui.add_space(2.0);
                ui.label("• the saved session (current view, coloring, preferences)");
                ui.label("• all bookmarks and their thumbnails");
                ui.add_space(6.0);
                ui.label(egui::RichText::new("Location").weak().small());
                ui.label(
                    egui::RichText::new(fractadyne_state::state_location_display())
                        .monospace()
                        .small(),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(
                        "This can't be undone. The current session won't be re-saved on exit, \
                         so defaults load on the next launch.",
                    )
                    .weak()
                    .small(),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    cancel = ui.button("Cancel").clicked();
                    if ui
                        .add(egui::Button::new(
                            egui::RichText::new("Reset everything").color(egui::Color32::WHITE),
                        ).fill(egui::Color32::from_rgb(0xB0, 0x3A, 0x30)))
                        .clicked()
                    {
                        confirm = true;
                    }
                });
            });
        if confirm {
            match fractadyne_state::reset_all() {
                Ok(_) => {
                    // Don't recreate what we just deleted; reflect the cleared bookmarks in the UI.
                    self.suppress_autosave = true;
                    self.bookmarks.clear();
                    self.set_toast(
                        "Application state reset — defaults will load on the next launch.",
                        ctx,
                    );
                }
                Err(e) => self.set_toast(format!("Reset failed: {e}"), ctx),
            }
            self.reset_confirm_open = false;
        } else if cancel {
            self.reset_confirm_open = false;
        } else {
            self.reset_confirm_open = open && self.reset_confirm_open;
        }
    }

    /// Transient status toast (fades out over ~4.5s), e.g. the minibrot-finder result.
    fn draw_toast(&mut self, ctx: &egui::Context) {
        let Some((msg, t0)) = self.toast.clone() else {
            return;
        };
        let age = ctx.input(|i| i.time) - t0;
        if age < 4.5 {
            let fade = ((4.5 - age) / 0.6).clamp(0.0, 1.0) as f32;
            egui::Area::new(egui::Id::new("fractadyne.toast"))
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 12.0))
                .interactable(false)
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style())
                        .fill(egui::Color32::from_rgb(0x23, 0x24, 0x28).gamma_multiply(fade))
                        .stroke(egui::Stroke::new(1.0, BRAND_ACCENT.gamma_multiply(fade)))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(msg).color(BRAND_TEXT.gamma_multiply(fade)),
                            );
                        });
                });
            ctx.request_repaint(); // keep fading
        } else {
            self.toast = None;
        }
    }

    /// Bookmarks manager — add the current view, jump to / delete saved locations (with thumbnails).
    fn draw_bookmarks_dialog(&mut self, ctx: &egui::Context) {
        if !self.bookmarks_open {
            return;
        }
        let mut open = self.bookmarks_open;
        let mut jump: Option<usize> = None;
        let mut delete: Option<usize> = None;
        let mut changed = false;
        // Pre-load thumbnail textures (mutable) before the immutable draw loop below.
        let thumb_ids: Vec<String> = self.bookmarks.iter().map(|b| b.thumb.clone()).collect();
        for id in &thumb_ids {
            let _ = self.bookmark_thumb_texture(ctx, id);
        }
        egui::Window::new("Bookmarks")
            .open(&mut open)
            .default_size([440.0, 480.0])
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
                            // Thumbnail preview (fixed 80px wide, aspect-preserving).
                            if let Some(tex) = self.thumb_cache.get(&b.thumb) {
                                let sz = tex.size_vec2();
                                let w = 80.0_f32;
                                ui.add(egui::Image::new(egui::load::SizedTexture::new(
                                    tex.id(),
                                    egui::vec2(w, w * sz.y / sz.x.max(1.0)),
                                )));
                            } else {
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(80.0, 56.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    rect,
                                    egui::CornerRadius::same(2),
                                    egui::Color32::from_black_alpha(80),
                                );
                            }
                            ui.vertical(|ui| {
                                ui.label(&b.name);
                                let zoom = meta_get(&b.meta, "zoom");
                                if !zoom.is_empty() {
                                    ui.label(egui::RichText::new(format!("{zoom}×")).weak().small());
                                }
                                ui.horizontal(|ui| {
                                    if ui.button("Go").clicked() {
                                        jump = Some(i);
                                    }
                                    if ui.button("🗑").on_hover_text("Delete").clicked() {
                                        delete = Some(i);
                                    }
                                });
                            });
                        });
                        ui.separator();
                    }
                });
            });
        if let Some(i) = jump {
            let meta = self.bookmarks[i].meta.clone();
            self.load_view_metadata(&meta);
        }
        if let Some(i) = delete {
            // Remove the thumbnail file + cached texture, then the bookmark.
            let id = self.bookmarks[i].thumb.clone();
            if !id.is_empty() {
                if let Some(p) = Self::bookmark_thumb_path(&id) {
                    let _ = std::fs::remove_file(p);
                }
                self.thumb_cache.remove(&id);
            }
            self.bookmarks.remove(i);
            changed = true;
        }
        if changed {
            self.save_bookmarks();
        }
        self.bookmarks_open = open;
    }

    /// Benchmark configuration dialog — pick current-settings vs standardized (resolution/depth/
    /// burn-in) and start the run.
    fn draw_bench_config_dialog(&mut self, ctx: &egui::Context) {
        if !self.bench_dialog_open {
            return;
        }
        let mut open = self.bench_dialog_open;
        let mut run_now = false;
        egui::Window::new("Benchmark")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Mode");
                ui.radio_value(&mut self.bench_cfg.standard, false, "Current settings")
                    .on_hover_text("Play the fixed deep-zoom tour into the live view using your current resolution and settings.");
                ui.radio_value(&mut self.bench_cfg.standard, true, "Standardized")
                    .on_hover_text("Pin resolution + all render settings so the score is comparable across machines.");
                ui.add_enabled_ui(self.bench_cfg.standard, |ui| {
                    ui.separator();
                    ui.label("Resolution");
                    egui::ComboBox::from_id_salt("bench_res")
                        .selected_text(self.bench_cfg.res.label())
                        .show_ui(ui, |ui| {
                            for r in BenchRes::ALL {
                                ui.selectable_value(&mut self.bench_cfg.res, r, r.label());
                            }
                        });
                    ui.add_space(4.0);
                    ui.label("Depth");
                    egui::ComboBox::from_id_salt("bench_depth")
                        .selected_text(self.bench_cfg.depth.label())
                        .show_ui(ui, |ui| {
                            for d in BenchDepth::ALL {
                                ui.selectable_value(&mut self.bench_cfg.depth, d, d.label());
                            }
                        })
                        .response
                        .on_hover_text("Ultra dives past f64's limit (1e28×), stressing the perturbation / series-approx / BLA deep-zoom path much harder.");
                    ui.add_space(4.0);
                    ui.checkbox(&mut self.bench_cfg.burnin, "Burn-in (repeat)")
                        .on_hover_text("Run the benchmark repeatedly to reveal stability and thermal throttling.");
                    ui.add_enabled_ui(self.bench_cfg.burnin, |ui| {
                        ui.add(egui::Slider::new(&mut self.bench_cfg.passes, 2..=200).text("passes"));
                    });
                });
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Run").clicked() {
                        run_now = true;
                    }
                    if ui.button("Cancel").clicked() {
                        self.bench_dialog_open = false;
                    }
                });
                if self.bench_cfg.standard {
                    let (w, h) = self.bench_cfg.res.dims();
                    ui.add_space(2.0);
                    ui.weak(format!(
                        "Renders offscreen at {w}×{h}, {}× SS, Mandelbrot/smooth, 60-frame dive to 1e{:.0}×.",
                        scripting::STD_AA,
                        self.bench_cfg.depth.zoom_log10(),
                    ));
                }
            });
        self.bench_dialog_open = open;
        if run_now {
            self.bench_dialog_open = false;
            if self.bench_cfg.standard {
                let passes = if self.bench_cfg.burnin { self.bench_cfg.passes } else { 1 };
                let run = self.begin_standard_bench(self.bench_cfg.res, passes, self.bench_cfg.depth);
                self.std_bench = Some(run);
                ctx.request_repaint();
            } else {
                self.start_benchmark();
            }
        }
    }

    /// Standardized-benchmark progress window (advances one dive-frame per event-loop tick).
    fn draw_bench_progress_dialog(&mut self, ctx: &egui::Context) {
        let Some((label, done, total, last_fps, (fdone, ftotal))) =
            self.std_bench.as_ref().map(|r| {
                (r.res.label(), r.passes_done, r.passes_total, r.pass_fps.last().copied(), r.frame_progress())
            })
        else {
            return;
        };
        let mut cancel = false;
        egui::Window::new("Running benchmark…")
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(format!("Standardized · {label}"));
                });
                if total > 1 {
                    ui.label(format!("Burn-in pass {}/{}", (done + 1).min(total), total));
                } else {
                    ui.label("Rendering the fixed dive…");
                }
                // Per-pass frame progress so it's visibly advancing (not hung) even mid-pass.
                ui.add(
                    egui::ProgressBar::new(fdone as f32 / ftotal.max(1) as f32)
                        .text(format!("frame {fdone}/{ftotal}")),
                );
                if let Some(f) = last_fps {
                    ui.label(format!("last pass: {f:.1} fps"));
                }
                ui.add_space(4.0);
                if ui.button("Cancel").clicked() {
                    cancel = true;
                }
            });
        if cancel {
            if let Some(run) = self.std_bench.take() {
                let report = (!run.pass_fps.is_empty()).then(|| self.format_std_bench(&run));
                let snap = run.take_snapshot();
                self.restore_from_bench(snap);
                if let Some(r) = report {
                    self.bench_report = Some(r);
                    self.bench_open = true;
                }
            }
        }
    }

    /// Benchmark results window — show the report, copy/save it, or reopen the config to run again.
    fn draw_bench_results_dialog(&mut self, ctx: &egui::Context) {
        if !self.bench_open {
            return;
        }
        let mut open = self.bench_open;
        let mut run_again = false;
        // Captured inside the egui closure (which borrows `self`), surfaced as a toast after it.
        let mut bench_save: Option<std::io::Result<()>> = None;
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
                                bench_save = Some(std::fs::write(path, &r));
                            }
                        }
                        if ui.button("Run again…").clicked() {
                            run_again = true;
                        }
                    });
                } else {
                    ui.label("No benchmark has been run yet.");
                }
            });
        self.bench_open = open;
        if let Some(res) = bench_save {
            self.set_toast(
                match res {
                    Ok(()) => "Benchmark saved.".to_string(),
                    Err(e) => format!("Save failed: {e}"),
                },
                ctx,
            );
        }
        // Close the results and reopen the benchmark tool to configure another run.
        if run_again {
            self.bench_open = false;
            self.bench_dialog_open = true;
        }
    }

    /// Gallery browser — scan a folder of exported PNG/EXR images and reopen any view (lazy thumbs).
    fn draw_gallery_dialog(&mut self, ctx: &egui::Context) {
        if !self.gallery.open {
            return;
        }
        let mut open = self.gallery.open;
        let mut to_open: Option<String> = None;
        let mut do_rescan = false;
        egui::Window::new("Gallery")
            .open(&mut open)
            .default_size([540.0, 620.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Folder…").clicked() {
                        if let Some(d) = rfd::FileDialog::new()
                            .set_directory(&self.gallery.dir)
                            .pick_folder()
                        {
                            self.gallery.dir = d;
                            do_rescan = true;
                        }
                    }
                    if ui.button("Refresh").clicked() {
                        do_rescan = true;
                    }
                    ui.label(
                        egui::RichText::new(self.gallery.dir.display().to_string())
                            .weak()
                            .small(),
                    );
                });
                ui.separator();
                if self.gallery.entries.is_empty() {
                    ui.label("No Fractadyne images in this folder.");
                }
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for entry in &self.gallery.entries {
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
                                        egui::Label::new(egui::RichText::new("…").weak()),
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
                                ui.label(egui::RichText::new(&entry.app_version).weak().small());
                                if ui.button("Open this view").clicked() {
                                    to_open = Some(entry.meta.clone());
                                }
                            });
                        });
                        ui.separator();
                    }
                });
            });
        self.gallery.open = open;
        if do_rescan {
            self.scan_gallery();
        }
        if let Some(meta) = to_open {
            self.load_view_metadata(&meta);
            self.export.status = Some("Loaded view from gallery.".to_string());
        }
        // Lazily decode one thumbnail per frame so scanning a folder never freezes.
        if let Some(entry) = self.gallery.entries.iter_mut().find(|e| !e.thumb_tried) {
            entry.thumb_tried = true;
            if let Ok((tw, th, rgba)) = fractadyne_export::read_thumbnail(&entry.path, 160) {
                let img = egui::ColorImage::from_rgba_unmultiplied([tw as usize, th as usize], &rgba);
                let name = format!("thumb:{}", entry.path.display());
                entry.thumb = Some(ctx.load_texture(name, img, egui::TextureOptions::LINEAR));
            }
            ctx.request_repaint();
        }
    }

    /// Export-image dialog — width/aspect/format/HUD/folder options, then Export (auto-named into
    /// the chosen folder) or Save as… `gpu` is needed to dispatch the render.
    fn draw_export_dialog(
        &mut self,
        ctx: &egui::Context,
        gpu: &Option<(eframe::wgpu::Device, eframe::wgpu::Queue)>,
    ) {
        if !self.export.open {
            return;
        }
        let mut open = self.export.open;
        let mut do_export = false;
        let mut do_export_as = false;
        egui::Window::new("Export image")
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                egui::ComboBox::from_label("Width (px)")
                    .selected_text(format!("{}", self.export.width))
                    .show_ui(ui, |ui| {
                        for w in [1280u32, 1920, 2560, 3840, 5120, 7680] {
                            ui.selectable_value(&mut self.export.width, w, format!("{w}"));
                        }
                    });
                egui::ComboBox::from_label("Supersampling")
                    .selected_text(format!("{}×", self.export.ss))
                    .show_ui(ui, |ui| {
                        for s in [1u32, 2, 3, 4] {
                            ui.selectable_value(&mut self.export.ss, s, format!("{s}×"));
                        }
                    });
                egui::ComboBox::from_label("Aspect")
                    .selected_text(if self.export.aspect == "window" {
                        "Match window".to_string()
                    } else {
                        self.export.aspect.clone()
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.export.aspect,
                            "window".to_string(),
                            "Match window",
                        );
                        for (k, _) in EXPORT_ASPECTS {
                            ui.selectable_value(&mut self.export.aspect, k.to_string(), k);
                        }
                    });
                ui.horizontal(|ui| {
                    ui.label("Format:");
                    ui.radio_value(&mut self.export.format, ExportFormat::Png, "PNG");
                    ui.radio_value(&mut self.export.format, ExportFormat::Exr, "OpenEXR");
                });
                ui.checkbox(&mut self.show_location, "Location HUD")
                    .on_hover_text(
                        "Burn a zoom-level + coordinate panel into the top-left of the exported \
                         image (scales with the output; shows the map/Mandelbrot view's zoom + \
                         center).",
                    );
                ui.checkbox(&mut self.render_cfg.glitch_correct, "Glitch correction")
                    .on_hover_text(
                        "Multi-reference correction of perturbation glitches. Automatically \
                         skipped for very deep (floatexp) single-view exports so the reference \
                         build + render run off-thread and the app stays responsive.",
                    );
                if self.viewport.magnification() >= PERT_FE_THRESHOLD {
                    ui.label(
                        egui::RichText::new(
                            "Deep export: the reference builds off-thread (UI stays live); \
                             glitch correction is skipped at this depth.",
                        )
                        .weak()
                        .small(),
                    );
                }
                if self.dual {
                    egui::ComboBox::from_label("Dual layout")
                        .selected_text(match self.export.dual_mode {
                            DualExport::SideBySide => "Side by side",
                            DualExport::Separate => "Separate files",
                            DualExport::ActiveOnly => "Map only",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.export.dual_mode,
                                DualExport::SideBySide,
                                "Side by side",
                            );
                            ui.selectable_value(
                                &mut self.export.dual_mode,
                                DualExport::Separate,
                                "Separate files",
                            );
                            ui.selectable_value(
                                &mut self.export.dual_mode,
                                DualExport::ActiveOnly,
                                "Map only",
                            );
                        });
                }
                ui.horizontal(|ui| {
                    ui.label("Notes:");
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.export.notes)
                            .char_limit(120)
                            .desired_width(220.0)
                            .hint_text("saved with the image (≤120 chars)"),
                    );
                    if resp.changed() && self.export.notes.chars().count() > 120 {
                        self.export.notes = self.export.notes.chars().take(120).collect();
                    }
                });
                ui.label(format!(
                    "Output: {} × {} px   ({} chars left)",
                    self.export.width,
                    self.export_height(),
                    120usize.saturating_sub(self.export.notes.chars().count()),
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
                // Target directory: a persistent folder that "Export" saves into (auto name).
                let target_dir = self
                    .export.last_dir
                    .clone()
                    .filter(|d| d.is_dir())
                    .unwrap_or_else(Self::pictures_dir);
                ui.horizontal(|ui| {
                    ui.label("Folder:");
                    if ui
                        .button("Choose…")
                        .on_hover_text("Pick the target directory for exports")
                        .clicked()
                    {
                        if let Some(d) = rfd::FileDialog::new().set_directory(&target_dir).pick_folder() {
                            self.export.last_dir = Some(d);
                        }
                    }
                });
                ui.label(
                    egui::RichText::new(target_dir.display().to_string())
                        .weak()
                        .small(),
                );
                ui.add_space(6.0);
                let busy = self.export.task.is_some() || self.export.prep.is_some();
                let elapsed = self
                    .export.started
                    .map(|t| Self::fmt_export_duration(t.elapsed()));
                if self.export.prep.is_some() {
                    // Deep export: the bignum reference is building off-thread (UI stays live).
                    ui.horizontal(|ui| {
                        ui.add(egui::Spinner::new());
                        ui.label("Preparing (building reference)…");
                    });
                } else if busy {
                    let p = self.export.progress.load(std::sync::atomic::Ordering::Relaxed);
                    if p >= 2000 {
                        // Rendering done; encoding/writing the file (not cancelable).
                        ui.horizontal(|ui| {
                            ui.add(egui::Spinner::new());
                            ui.label("Saving…");
                        });
                    } else {
                        ui.label("Rendering…");
                        ui.add(egui::ProgressBar::new(p as f32 / 1000.0).show_percentage());
                        if ui.button("Cancel").clicked() {
                            self.export.cancel
                                .store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                }
                // Live elapsed readout while an export is in flight (updated each frame; the
                // busy pollers above request repaints). The final total is folded into the
                // status line on completion.
                if busy {
                    if let Some(t) = &elapsed {
                        ui.label(egui::RichText::new(format!("Elapsed: {t}")).weak().small());
                    }
                } else {
                    ui.horizontal(|ui| {
                        if ui
                            .button("Export")
                            .on_hover_text("Render and save into the folder above (auto-named)")
                            .clicked()
                        {
                            do_export = true;
                        }
                        if ui
                            .button("Save as…")
                            .on_hover_text("Choose the file name and location")
                            .clicked()
                        {
                            do_export_as = true;
                        }
                    });
                }
                if let Some(s) = &self.export.status {
                    ui.add_space(6.0);
                    ui.label(s);
                }
            });
        self.export.open = open;
        if do_export {
            if let Some((dev, q)) = gpu {
                // Save straight into the chosen folder with an auto (timestamped) name.
                let dir = self
                    .export.last_dir
                    .clone()
                    .filter(|d| d.is_dir())
                    .unwrap_or_else(Self::pictures_dir);
                let path = dir.join(self.export_default_name());
                self.start_export_to(ctx, dev.clone(), q.clone(), path);
            } else {
                self.export.status = Some("GPU not available".to_string());
            }
        }
        if do_export_as {
            if let Some((dev, q)) = gpu {
                self.start_export(ctx, dev.clone(), q.clone());
            } else {
                self.export.status = Some("GPU not available".to_string());
            }
        }
    }
    /// Top menu bar + toolbar (File / Fractal / View / Tools / Bookmarks / Locations / Help)
    /// plus the icon toolbar. Takes the `gpu` handle for the quick-export toolbar action.
    fn draw_menu_bar(
        &mut self,
        ctx: &egui::Context,
        gpu: &Option<(eframe::wgpu::Device, eframe::wgpu::Queue)>,
    ) {
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                    brand_wordmark(ui);
                    ui.separator();
                    ui.menu_button("File", |ui| {
                        if ui.button("📂  Open view…").clicked() {
                            self.open_view();
                            ui.close_menu();
                        }
                        if ui.button("🖼  Gallery…").clicked() {
                            self.gallery.open = true;
                            self.scan_gallery();
                            ui.close_menu();
                        }
                        if ui.button("💾  Export image…").clicked() {
                            self.export.open = true;
                            ui.close_menu();
                        }
                        if ui
                            .button("🔗  Share location…")
                            .on_hover_text(
                                "Copy / paste / save / load a self-contained location \
                                 (.fdn): fractal, full-precision center, zoom, coloring.",
                            )
                            .clicked()
                        {
                            self.open_share();
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui
                            .button("♻  Reset application state…")
                            .on_hover_text(
                                "Delete all saved state (session, bookmarks, thumbnails) and \
                                 start fresh. Asks for confirmation first.",
                            )
                            .clicked()
                        {
                            self.reset_confirm_open = true;
                            ui.close_menu();
                        }
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
                        if ui.button("Go to location…").clicked() {
                            self.open_goto();
                            ui.close_menu();
                        }
                        ui.add_enabled_ui(self.nav.undo.len() > 1, |ui| {
                            if ui.button("Undo view  (Backspace)").clicked() {
                                self.undo_view();
                                ui.close_menu();
                            }
                        });
                        ui.add_enabled_ui(!self.nav.redo.is_empty(), |ui| {
                            if ui.button("Redo view  (Shift+Backspace)").clicked() {
                                self.redo_view();
                                ui.close_menu();
                            }
                        });
                        ui.add_enabled_ui(matches!(self.fractal.formula_id(), 0..=3), |ui| {
                            if ui
                                .button("Find minibrot center  (M)")
                                .on_hover_text(
                                    "Newton-snap the view center to the nearby minibrot's \
                                     exact center and report its period.",
                                )
                                .clicked()
                            {
                                let ctx = ui.ctx().clone();
                                self.find_minibrot(&ctx);
                                ui.close_menu();
                            }
                        });
                        ui.add_enabled_ui(!self.dual, |ui| {
                            let label = if self.autopilot.active {
                                "Stop autopilot  (A)"
                            } else {
                                "Auto-zoom (autopilot)  (A)"
                            };
                            if ui
                                .button(label)
                                .on_hover_text(
                                    "Hands-free continuous deep zoom: dives toward the \
                                     detail-richest region, re-steering as it goes. Any \
                                     navigation input stops it.",
                                )
                                .clicked()
                            {
                                let ctx = ui.ctx().clone();
                                self.toggle_autopilot(&ctx);
                                ui.close_menu();
                            }
                        });
                        ui.separator();
                        ui.checkbox(&mut self.right_panel_open, "Control panel")
                            .on_hover_text("Show/hide the right-hand control panel.");
                        ui.checkbox(&mut self.minimap, "Minimap overview")
                            .on_hover_text(
                                "Show a small home-view overview with a \"you are here\" \
                                 marker and the zoom depth. Click it to jump to a region.",
                            );
                        ui.checkbox(&mut self.render_cfg.series_approx, "Series approximation")
                            .on_hover_text(
                                "Speed up deep Mandelbrot renders (≥1e28×) by seeding the \
                                 perturbation from a polynomial and skipping early iterations. \
                                 Identical output; turn off to compare.",
                            );
                        ui.checkbox(&mut self.render_cfg.glitch_correct, "Glitch correction (export)")
                            .on_hover_text(
                                "Multi-reference glitch correction for exported images: detects \
                                 perturbation glitches and re-renders those pixels against extra \
                                 references until clean. On by default. Applies to exports up to \
                                 ~32 MP / the GPU texture limit (non-aux coloring); larger images \
                                 and the live view fall back to the plain path.",
                            );
                        ui.checkbox(&mut self.render_cfg.use_bla, "BLA acceleration (deep zoom)")
                            .on_hover_text(
                                "Bilinear approximation: skip iterations throughout the orbit at \
                                 extreme depth (floatexp Mandelbrot, ≥1e28×) — ~5× faster GPU \
                                 render, identical output (verified by the self-test). On by \
                                 default; turn off to compare or if you hit an artifact.",
                            );
                        ui.checkbox(&mut self.watermark, "Fd watermark")
                            .on_hover_text(
                                "Draw a discreet \"Fd\" brand mark in the lower-right corner of the \
                                 live view and exported images. On by default.",
                            );
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
                        ui.separator();
                        ui.label("UI scale (font size)");
                        for (label, val) in [
                            ("80%", 0.8_f32),
                            ("90%", 0.9),
                            ("100%", 1.0),
                            ("110%", 1.1),
                            ("125%", 1.25),
                            ("150%", 1.5),
                        ] {
                            if ui
                                .selectable_label((self.ui_scale - val).abs() < 0.01, label)
                                .clicked()
                            {
                                self.ui_scale = val;
                            }
                        }
                    });
                    ui.menu_button("Tools", |ui| {
                        if ui
                            .button("Benchmark…")
                            .on_hover_text(
                                "Measure rendering speed — current settings, or a standardized \
                                 run (fixed resolution + settings) comparable across machines. \
                                 Burn-in repeats it to check stability / thermal throttling.",
                            )
                            .clicked()
                        {
                            self.bench_dialog_open = true;
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
                    ui.menu_button("Locations", |ui| {
                        let ctx = ui.ctx().clone();
                        for (name, cx, cy, mag) in FAMOUS {
                            if ui.button(*name).clicked() {
                                self.goto_location(cx, cy, *mag, name, &ctx);
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        if ui
                            .button("🎲  Random location")
                            .on_hover_text("Jump to a random detail-rich boundary point")
                            .clicked()
                        {
                            self.random_location(&ctx);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui
                            .button("Import .kfr…")
                            .on_hover_text("Load a Kalles Fraktaler location file")
                            .clicked()
                        {
                            self.import_kfr(&ctx);
                            ui.close_menu();
                        }
                    });
                    ui.menu_button("Help", |ui| {
                        if ui.button("Help & reference…  (F1)").clicked() {
                            self.help_open = true;
                            ui.close_menu();
                        }
                        ui.separator();
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
                // Dual pairs a formula with its Julia set, so it's only meaningful where a Julia
                // exists (Newton has no free parameter → no Julia → no dual).
                ui.add_enabled_ui(self.fractal.supports_julia(), |ui| {
                    if dual_toggle_button(ui, self.dual)
                        .on_hover_text("Dual linked view (Mandelbrot ↔ Julia)")
                        .clicked()
                    {
                        self.toggle_dual();
                    }
                });
                ui.separator();
                // ── File / I-O: open & browse, then save ──────────────────────────────
                if ui.button("📂").on_hover_text("Open view…").clicked() {
                    self.open_view();
                }
                if ui.button("🖼").on_hover_text("Gallery").clicked() {
                    self.gallery.open = true;
                    self.scan_gallery();
                }
                if ui.button("💾").on_hover_text("Export image…").clicked() {
                    self.export.open = true;
                }
                if ui
                    .button("📷")
                    .on_hover_text("Snapshot — quick export to the last folder (Ctrl+S)")
                    .clicked()
                {
                    if let Some((dev, q)) = gpu {
                        self.quick_export(ctx, dev.clone(), q.clone());
                    }
                }
                ui.separator();
                // ── Navigation / location: zoom, reset/home, bookmark ─────────────────
                if ui.button("🔍+").on_hover_text("Zoom in").clicked() {
                    self.zoom_center(0.5);
                }
                if ui.button("🔍−").on_hover_text("Zoom out").clicked() {
                    self.zoom_center(2.0);
                }
                // Auto-zoom (autopilot): highlighted while running; click to start/stop. Single view only.
                ui.add_enabled_ui(!self.dual, |ui| {
                    let running = self.autopilot.active;
                    if ui
                        .selectable_label(running, "🛸")
                        .on_hover_text(if running {
                            "Auto-zoom is running — click to stop"
                        } else {
                            "Auto-zoom: dive toward detail (A)"
                        })
                        .clicked()
                    {
                        self.toggle_autopilot(ctx);
                    }
                });
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
                if ui
                    .button("★")
                    .on_hover_text("Bookmark this view")
                    .clicked()
                {
                    self.add_bookmark("");
                }
                ui.separator();
                // ── Appearance / display ──────────────────────────────────────────────
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
    }
    /// Right-side control panel (Coloring + Navigation sections) plus the reopen handle shown
    /// when it's hidden. No-op in fullscreen.
    fn draw_right_panel(&mut self, ctx: &egui::Context) {
        if !self.fullscreen && self.right_panel_open {
        egui::SidePanel::right("coloring_panel")
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    // ▶ points toward the right edge the panel collapses to (it's docked right).
                    if ui.small_button("\u{23F5}").on_hover_text("Hide control panel").clicked() {
                        self.right_panel_open = false;
                    }
                    ui.label(egui::RichText::new("Controls").strong());
                });
                ui.separator();
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

                egui::CollapsingHeader::new("Coloring").default_open(true).show(ui, |ui| {
                egui::ComboBox::from_label("Method")
                    .selected_text(self.color_method.label())
                    .show_ui(ui, |ui| {
                        for m in ColorMethod::ALL {
                            ui.selectable_value(&mut self.color_method, m, m.label());
                        }
                    })
                    .response
                    .on_hover_text(
                        "How escape data maps to color. Stripe / triangle-inequality / \
                         orbit-trap / decomposition reveal orbit structure; distance \
                         shades by proximity to the boundary.",
                    );
                if self.color_method == ColorMethod::Stripe {
                    ui.add(
                        egui::Slider::new(&mut self.stripe_freq, 1.0..=24.0)
                            .text("Stripe density")
                            .logarithmic(true),
                    );
                }
                if self.color_method == ColorMethod::OrbitTrap {
                    egui::ComboBox::from_label("Trap shape")
                        .selected_text(self.trap_type.label())
                        .show_ui(ui, |ui| {
                            for t in TrapType::ALL {
                                ui.selectable_value(&mut self.trap_type, t, t.label());
                            }
                        });
                }
                let pal_name = if self.use_binary {
                    "Binary (set)"
                } else if self.use_duotone {
                    "Duotone"
                } else if self.use_custom_palette {
                    "Custom"
                } else {
                    fractadyne_color::PRESETS[self.palette_idx].name
                };
                egui::ComboBox::from_label("Palette")
                    .selected_text(pal_name)
                    .show_ui(ui, |ui| {
                        let is_preset = !self.use_custom_palette && !self.use_duotone && !self.use_binary;
                        for (i, p) in fractadyne_color::PRESETS.iter().enumerate() {
                            if ui.selectable_label(is_preset && self.palette_idx == i, p.name).clicked() {
                                self.palette_idx = i;
                                self.use_custom_palette = false;
                                self.use_duotone = false;
                                self.use_binary = false;
                            }
                        }
                        if ui.selectable_label(self.use_custom_palette, "Custom ✎").clicked() {
                            if self.custom_palette.is_empty() {
                                self.custom_palette = self.preset_as_stops(self.palette_idx);
                            }
                            self.use_custom_palette = true;
                            self.use_duotone = false;
                            self.use_binary = false;
                        }
                        if ui.selectable_label(self.use_duotone, "Duotone").clicked() {
                            self.use_duotone = true;
                            self.use_custom_palette = false;
                            self.use_binary = false;
                        }
                        if ui
                            .selectable_label(self.use_binary, "Binary (set)")
                            .on_hover_text("Flat two-color: in-set vs out-of-set, no gradient.")
                            .clicked()
                        {
                            self.use_binary = true;
                            self.use_custom_palette = false;
                            self.use_duotone = false;
                        }
                    });
                if self.use_duotone || self.use_binary {
                    // Two shared colors. (Binary: interior/exterior; duotone: shadow/highlight.)
                    let (lo_lbl, hi_lbl) = if self.use_binary {
                        ("In-set", "Out-of-set")
                    } else {
                        ("Shadow", "Highlight")
                    };
                    ui.horizontal(|ui| {
                        ui.color_edit_button_rgb(&mut self.duotone_lo);
                        ui.label(lo_lbl);
                        ui.color_edit_button_rgb(&mut self.duotone_hi);
                        ui.label(hi_lbl);
                    });
                } else if ui.button("Edit gradient…").clicked() {
                    if self.custom_palette.is_empty() {
                        self.custom_palette = self.preset_as_stops(self.palette_idx);
                    }
                    self.use_custom_palette = true;
                    self.palette_editor_open = true;
                }
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
                ui.checkbox(&mut self.effects.light, "3D relief lighting")
                    .on_hover_text(
                        "Shade the surface using the distance-estimate slope — an \
                         embossed, lit look. (Holomorphic families: Mandelbrot / \
                         Multibrot.)",
                    );
                ui.add_enabled_ui(self.effects.light, |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.effects.light_angle, 0.0..=std::f32::consts::TAU)
                            .text("Light angle")
                            .suffix(" rad"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.effects.light_height, 0.2..=4.0)
                            .text("Relief")
                            .logarithmic(true),
                    )
                    .on_hover_text("Lower = sharper relief; higher = softer/flatter.");
                    ui.checkbox(&mut self.effects.light_anim, "Rotate light")
                        .on_hover_text("Spin the light direction over time (uses the Speed slider).");
                });
                ui.checkbox(&mut self.effects.de, "Distance glow")
                    .on_hover_text(
                        "Bright distance-estimate contour bands that densify into glowing \
                         filaments near the boundary. (Holomorphic families.)",
                    );
                ui.add_enabled_ui(self.effects.de, |ui| {
                    ui.add(egui::Slider::new(&mut self.effects.de_strength, 0.0..=1.0).text("Glow"));
                    ui.add(
                        egui::Slider::new(&mut self.effects.de_width, 0.15..=4.0)
                            .text("Band width")
                            .logarithmic(true),
                    )
                    .on_hover_text("Spacing of the distance contours (octaves per band).");
                    ui.checkbox(&mut self.effects.de_anim, "Animate glow")
                        .on_hover_text("Flow the glow bands over time (uses the Speed slider).");
                });
                ui.separator();
                ui.checkbox(&mut self.render_cfg.auto_iter, "Auto-scale iterations with zoom");
                let label = if self.render_cfg.auto_iter { "Iterations (base)" } else { "Iterations" };
                ui.add(
                    egui::Slider::new(&mut self.render_cfg.max_iter, 64..=500_000)
                        .logarithmic(true)
                        .text(label),
                )
                .on_hover_text(
                    "Base iteration count. With Auto-scale on, the effective count climbs \
                     with zoom depth. While you're moving, the preview caps iterations low \
                     (50,000) for responsiveness, then sharpens to the full count when the \
                     view settles — so deep edges look smooth during motion and resolve when \
                     you stop.",
                );
                // Detail note: the coarse count only applies while moving. If the view is
                // settled and still resolution-limited (huge window at extreme depth), an
                // export renders at full resolution — otherwise the settled preview already
                // matches it. Show the current effective (settled) count so the user knows
                // the true detail level once motion stops.
                let log2mag = self.viewport.log2_magnification();
                let want_iter = if self.render_cfg.auto_iter {
                    self.viewport.recommended_max_iter(self.render_cfg.max_iter)
                } else {
                    self.render_cfg.max_iter
                };
                let settled_iter = want_iter.min(500_000).min(zoom_iter_cap(log2mag).max(256));
                let px = (self.viewport.width_px * self.viewport.height_px).max(1.0) as u64;
                let res_limited = px.saturating_mul(settled_iter.max(1) as u64)
                    > self.effective_work_budget().saturating_mul(6);
                if res_limited {
                    ui.label(
                        egui::RichText::new(format!(
                            "⚠ At this depth the settled preview renders {} iterations at \
                             reduced resolution to stay responsive. Export for full-resolution \
                             detail.",
                            commas(&settled_iter.to_string()),
                        ))
                        .small()
                        .color(theme::BRAND_ACCENT),
                    );
                }
                ui.separator();
                egui::ComboBox::from_label("Anti-alias")
                    .selected_text(match self.render_cfg.aa {
                        1 => "Off",
                        2 => "2×",
                        3 => "3×",
                        4 => "4×",
                        _ => "8×",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.render_cfg.aa, 1, "Off");
                        ui.selectable_value(&mut self.render_cfg.aa, 2, "2×");
                        ui.selectable_value(&mut self.render_cfg.aa, 3, "3×");
                        ui.selectable_value(&mut self.render_cfg.aa, 4, "4×");
                        ui.selectable_value(&mut self.render_cfg.aa, 8, "8×");
                    })
                    .response
                    .on_hover_text(
                        "Supersampling for still images (applied when the view settles). \
                         Higher tames the fine exterior 'dust' at the cost of render time.",
                    );

                });
                egui::CollapsingHeader::new("Navigation").default_open(true).show(ui, |ui| {
                ui.add(
                    egui::Slider::new(&mut self.render_cfg.zoom_rate, 0.25..=4.0)
                        .text("Zoom speed")
                        .suffix("×")
                        .logarithmic(true),
                )
                .on_hover_text("Speed of hold-Space continuous zoom (1× ≈ 2× per 1.5 s).");

                // Auto-zoom (autopilot) dive limit, edited in decimal orders (1eN×) but stored as log2.
                let mut dive_log10 = self.autopilot.dive_log2 / std::f64::consts::LOG2_10;
                if ui
                    .add(
                        egui::Slider::new(&mut dive_log10, 30.0..=5000.0)
                            .text("Auto-zoom dive limit")
                            .logarithmic(true)
                            .custom_formatter(|n, _| format!("1e{n:.0}×")),
                    )
                    .on_hover_text(
                        "Depth where auto-zoom (A key) stops. Up to ~1e271× it glides smoothly; \
                         deeper, it switches to a choppy stepped dive to reach extreme depth quickly.",
                    )
                    .changed()
                {
                    self.autopilot.dive_log2 = dive_log10 * std::f64::consts::LOG2_10;
                }

                // Live-render work budget: detail-vs-speed for the deep-zoom preview.
                ui.add(
                    egui::Slider::new(&mut self.render_cfg.work_budget_scale, 0.25..=8.0)
                        .text("Live render budget")
                        .suffix("×")
                        .logarithmic(true),
                )
                .on_hover_text(
                    "Detail vs. speed for the live view at deep zoom. Higher renders at fuller \
                     resolution (crisper — less of the \"soft\" upscaled look) but lowers frame-rate, \
                     and very high values risk a brief GPU stall on the heaviest frames. Exports are \
                     always full resolution regardless of this.",
                );

                });
                // Performance section, docked at the bottom of this same panel
                // (toggle via the Perf button or the View menu).
                if self.perf.enabled {
                    egui::CollapsingHeader::new("Performance")
                        .default_open(true)
                        .show(ui, |ui| {
                            self.perf_section(ui);
                        });
                }
            });
        }

        // Reopen handle when the control panel is hidden (and not fullscreen).
        if !self.fullscreen && !self.right_panel_open {
            egui::Area::new(egui::Id::new("panel_reopen"))
                .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 36.0))
                .show(ctx, |ui| {
                    if ui.button("\u{2630}").on_hover_text("Show control panel").clicked() {
                        self.right_panel_open = true;
                    }
                });
        }
    }
    /// Bottom status bar — center coordinate, cursor, zoom, effective iteration count, and the
    /// live script / benchmark playback progress.
    fn draw_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let l2 = self.viewport.log2_magnification();
                ui.monospace(format!(
                    "center {}, {}",
                    fmt_coord_deep(&self.viewport.center_x, l2),
                    fmt_coord_deep(&self.viewport.center_y, l2),
                ));
                ui.separator();
                match self.pointer.pointer_complex {
                    Some((mx, my)) => {
                        ui.monospace(format!("cursor {}, {}", fmt_coord(mx), fmt_coord(my)))
                    }
                    None => ui.monospace("cursor —"),
                };
                ui.separator();
                if self.dual {
                    ui.monospace(format!(
                        "zoom  M {}×   J {}×",
                        fmt_zoom_log2(self.viewport.log2_magnification()),
                        fmt_zoom_log2(self.julia_viewport.log2_magnification()),
                    ));
                } else {
                    ui.monospace(format!("zoom {}×", fmt_zoom_log2(self.viewport.log2_magnification())));
                }
                ui.separator();
                // Show the count actually rendered last frame (coarse while moving, full when
                // settled) — matches the Performance panel's "eff iter".
                let eff_iter = if self.perf.last_eff_iter > 0 {
                    self.perf.last_eff_iter
                } else {
                    let want_iter = if self.render_cfg.auto_iter {
                        self.viewport.recommended_max_iter(self.render_cfg.max_iter)
                    } else {
                        self.render_cfg.max_iter
                    };
                    want_iter.min(zoom_iter_cap(self.viewport.log2_magnification()).max(256))
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
    }
    /// Central fractal area: the render callback (or dual view), pan / zoom / click input, the
    /// orbit overlay, plus the watermark, guided-tour annotations, minimap, gradient editor,
    /// and help overlay drawn on top of it.
    fn draw_central(&mut self, ctx: &egui::Context) {
        let central = egui::CentralPanel::default()
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

                // Zoom box (Shift+drag): rubber-band a rectangle, then zoom so it fills the
                // view. Deep-zoom-correct (recenter + scale via the bignum viewport methods).
                let shift = ctx.input(|i| i.modifiers.shift);
                if response.drag_started() && shift {
                    if let Some(p) = response.interact_pointer_pos() {
                        self.pointer.zoom_box = Some(ZoomBox { start: p, end: p, is_julia: false });
                    }
                }
                let mut zoom_boxing = false;
                if self.pointer.zoom_box.as_ref().is_some_and(|z| !z.is_julia) {
                    zoom_boxing = true;
                    if let Some(cur) = response.interact_pointer_pos() {
                        self.pointer.zoom_box.as_mut().unwrap().end = cur;
                    }
                    let zb = self.pointer.zoom_box.as_ref().unwrap();
                    let boxr = aspect_zoom_box(zb.start, zb.end, rect);
                    // Foreground layer so the box draws above the fractal paint callback.
                    let painter = ctx.layer_painter(egui::LayerId::new(
                        egui::Order::Foreground,
                        egui::Id::new("fract_zoom_box"),
                    ));
                    painter.rect_filled(
                        boxr,
                        egui::CornerRadius::ZERO,
                        egui::Color32::from_rgba_unmultiplied(0xE0, 0xA0, 0x30, 32),
                    );
                    painter.rect_stroke(
                        boxr,
                        egui::CornerRadius::ZERO,
                        egui::Stroke::new(1.5, BRAND_ACCENT),
                        egui::StrokeKind::Inside,
                    );
                    if response.drag_stopped() {
                        if boxr.width() > 6.0 && boxr.height() > 6.0 {
                            let bcx = (boxr.center().x - rect.min.x) as f64 * ppp;
                            let bcy = (boxr.center().y - rect.min.y) as f64 * ppp;
                            let factor = (boxr.width() / rect.width()) as f64; // < 1 ⇒ zoom in
                            let (w, h) = (self.viewport.width_px, self.viewport.height_px);
                            self.viewport.pan_pixels(w * 0.5 - bcx, h * 0.5 - bcy);
                            self.viewport.zoom_at(w * 0.5, h * 0.5, factor);
                            self.pointer.settle_t[0] = ctx.input(|i| i.time);
                        }
                        self.pointer.zoom_box = None;
                        zoom_boxing = false;
                    }
                }

                // Track the complex coordinate under the cursor (for the status bar).
                self.pointer.pointer_complex = response.hover_pos().map(|p| {
                    let l = p - rect.min;
                    self.viewport
                        .complex_at_pixel_f64(l.x as f64 * ppp, l.y as f64 * ppp)
                });

                // Pan with left-drag (unless dragging a zoom box).
                if !zoom_boxing && response.dragged_by(egui::PointerButton::Primary) {
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
                    self.pointer.last_cursor = Some(pos);
                }
                let (space, shift) =
                    ctx.input(|i| (i.key_down(egui::Key::Space), i.modifiers.shift));
                let rate = ZOOM_RATE * self.render_cfg.zoom_rate as f64;
                let target_vel = if space {
                    if shift { -rate } else { rate }
                } else {
                    0.0
                };
                let dt = (ctx.input(|i| i.stable_dt) as f64).clamp(0.0, 0.1);
                let ease = 1.0 - (-dt / EASE_TAU).exp();
                self.pointer.zoom_vel += (target_vel - self.pointer.zoom_vel) * ease;
                if target_vel != 0.0 || self.pointer.zoom_vel.abs() > 1e-3 {
                    self.schedule_repaint(ctx); // animate while held and during glide-out
                }
                if self.pointer.zoom_vel.abs() > 1e-3 {
                    let anchor = self.pointer.last_cursor.unwrap_or_else(|| rect.center());
                    let local = anchor - rect.min;
                    let factor = (-self.pointer.zoom_vel * dt).exp(); // vel>0 → factor<1 → zoom in
                    self.viewport
                        .zoom_at(local.x as f64 * ppp, local.y as f64 * ppp, factor);
                }

                // Box-zoom with right-drag: record the start, apply on release.
                if response.drag_started_by(egui::PointerButton::Secondary) {
                    self.pointer.box_start = response.interact_pointer_pos();
                }
                if response.drag_stopped_by(egui::PointerButton::Secondary) {
                    let end = response
                        .interact_pointer_pos()
                        .or_else(|| ctx.input(|i| i.pointer.latest_pos()));
                    if let (Some(start), Some(end)) = (self.pointer.box_start, end) {
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
                    self.pointer.box_start = None;
                }

                // Render the fractal at the current viewport with the chosen palette.
                let span_fe = self.viewport.complex_span_fe();
                let eff_iter = if self.render_cfg.auto_iter {
                    self.viewport.recommended_max_iter(self.render_cfg.max_iter)
                } else {
                    self.render_cfg.max_iter
                };
                // Quality-on-settle: skip AA while interacting (and for a short
                // settle window after), then render full AA once the view is still.
                let now = ctx.input(|i| i.time);
                // Drawing a zoom box (Shift+left-drag → `zoom_boxing`, or right-drag →
                // `box_start`) drags the pointer but does NOT move the view, so it must not
                // count as "active" — otherwise the render drops to the coarse moving preview
                // while you're framing the box. The zoom is applied on release.
                let active = self.pointer.zoom_vel.abs() > 1e-3
                    || (response.dragged() && !zoom_boxing && self.pointer.box_start.is_none())
                    || scroll_y != 0.0
                    || space;
                if active {
                    self.pointer.settle_t[0] = now;
                }
                let interacting = now - self.pointer.settle_t[0] < SETTLE_DELAY;
                // Progressive settle AA (see `nav_and_draw`): refine 1×→2×→4×→… over settled frames.
                let aa_target = if interacting {
                    self.pointer.settle_frame[0] = 0;
                    1
                } else {
                    let ss = aa_ramp(self.pointer.settle_frame[0], self.render_cfg.aa);
                    if ss < self.render_cfg.aa {
                        self.pointer.settle_frame[0] += 1;
                        self.schedule_repaint(ctx);
                    }
                    ss
                };

                let center_bf = [self.viewport.center_x.clone(), self.viewport.center_y.clone()];
                let center = self.viewport.center_f64();
                let mag = self.viewport.magnification();
                let log2mag = self.viewport.log2_magnification();
                let resolution = [
                    (rect.width() as f64 * ppp) as u32,
                    (rect.height() as f64 * ppp) as u32,
                ];
                // Pan reprojection: while dragging, translate the last detailed frame instead
                // of re-rendering coarse (see `nav_and_draw` for the full rationale).
                let panning = !zoom_boxing && self.pointer.box_start.is_none();
                if response.drag_started_by(egui::PointerButton::Primary) && panning {
                    self.pointer.pan_px = egui::Vec2::ZERO;
                    self.pointer.pan_view = Some(0);
                }
                if self.pointer.pan_view == Some(0)
                    && panning
                    && response.dragged_by(egui::PointerButton::Primary)
                {
                    let d = response.drag_delta();
                    self.pointer.pan_px += egui::vec2(d.x * ppp as f32, d.y * ppp as f32);
                }
                let reproject = if self.pointer.pan_view == Some(0) && interacting {
                    self.schedule_repaint(ctx);
                    Some([
                        self.pointer.pan_px.x / resolution[0].max(1) as f32,
                        self.pointer.pan_px.y / resolution[1].max(1) as f32,
                    ])
                } else {
                    if self.pointer.pan_view == Some(0) {
                        self.pointer.pan_view = None;
                    }
                    None
                };
                let params = self.build_params(
                    center_bf,
                    center,
                    span_fe,
                    mag,
                    log2mag,
                    self.fractal,
                    self.julia_mode,
                    eff_iter,
                    interacting,
                    aa_target,
                    resolution,
                    0,
                    reproject,
                );
                add_mandelbrot(ui.painter(), rect, params);

                // Orbit overlay for the point under the cursor — or, during a tour, a scripted point.
                if self.show_orbits {
                    let cpx = response
                        .hover_pos()
                        .map(|hp| {
                            let l = hp - rect.min;
                            (l.x as f64 * ppp, l.y as f64 * ppp)
                        })
                        .or_else(|| {
                            self.tour_orbit.map(|(ox, oy)| {
                                let p = self.viewport.precision;
                                self.viewport.complex_to_pixel(
                                    &fractadyne_core::BigFloat::from_f64(ox, p),
                                    &fractadyne_core::BigFloat::from_f64(oy, p),
                                )
                            })
                        });
                    if let Some(cpx) = cpx {
                        let painter = ui.painter_at(rect);
                        self.draw_orbit(&painter, rect, &self.viewport, cpx, self.julia_mode, ppp);
                    }
                }

                // Draw the in-progress box-zoom selection on top of the fractal.
                if let Some(start) = self.pointer.box_start {
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

        // ---- brand watermark (lower-right of the fractal area) ----
        if self.watermark {
            self.draw_watermark(ctx, central.response.rect);
        }

        // ---- guided-tour annotations (captions + coordinate-anchored callouts) ----
        if self.playback.is_some() {
            let caption_rects = self.draw_captions(ctx, central.response.rect);
            self.draw_callouts(ctx, central.response.rect, &caption_rects);
        }

        // ---- minimap overview ----
        self.draw_minimap(ctx);

        // ---- gradient editor ----
        self.palette_editor_window(ctx);

        // ---- keyboard / help overlay ----
        self.help_window(ctx);
    }
}

impl eframe::App for FractadyneApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let frame_start = Instant::now();
        // Surface a deferred startup warning (e.g. a session saved by a newer build) as a toast,
        // once, on the first frame where the UI exists.
        if let Some(msg) = self.pending_state_warning.take() {
            self.set_toast(msg, ctx);
        }
        // Surface a toast queued from a context without an `egui::Context` (e.g. a bookmark
        // auto-save failure in `save_bookmarks`).
        if let Some(msg) = self.pending_toast.take() {
            self.set_toast(msg, ctx);
        }
        // Rasterize the export watermark once from the font atlas (main thread — the export worker
        // has no egui context). Lazy so it uses the loaded fonts + final DPI.
        if self.watermark && self.watermark_overlay.is_none() {
            self.watermark_overlay = export::build_watermark_overlay(ctx);
        }
        // Apply the UI scale preference (egui zoom factor) when it changes.
        if (ctx.zoom_factor() - self.ui_scale).abs() > 1.0e-4 {
            ctx.set_zoom_factor(self.ui_scale);
        }
        // GPU handles for offline export (cloned Arcs; cheap).
        let gpu = frame
            .wgpu_render_state()
            .map(|rs| (rs.device.clone(), rs.queue.clone()));
        self.update_minimap(ctx, &gpu);

        // Ctrl+S → quick export (no dialog) to the last folder.
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            if let Some((dev, q)) = &gpu {
                self.quick_export(ctx, dev.clone(), q.clone());
            }
        }

        // Esc: stop the autopilot / a playing tour first, otherwise leave fullscreen.
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.autopilot.active {
                self.autopilot.active = false;
                self.autopilot.stepping = false;
                self.pointer.zoom_vel = 0.0;
            } else if self.playback.is_some() {
                self.playback = None;
            } else if self.fullscreen {
                self.fullscreen = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
        }

        // Navigation undo/redo (Backspace / Shift+Backspace or Ctrl+Y), unless typing.
        if !ctx.wants_keyboard_input() {
            let (undo, redo) = ctx.input(|i| {
                let bs = i.key_pressed(egui::Key::Backspace);
                (
                    bs && !i.modifiers.shift,
                    (bs && i.modifiers.shift) || (i.modifiers.command && i.key_pressed(egui::Key::Y)),
                )
            });
            if undo {
                self.undo_view();
            } else if redo {
                self.redo_view();
            }
            // M: find the nearby minibrot center (single view only).
            if ctx.input(|i| i.key_pressed(egui::Key::M) && !i.modifiers.any()) && !self.dual {
                self.find_minibrot(ctx);
            }
            // A: toggle the auto-zoom autopilot (single view only).
            if ctx.input(|i| i.key_pressed(egui::Key::A) && !i.modifiers.any()) && !self.dual {
                self.toggle_autopilot(ctx);
            }
            // F1 / ? : toggle the help overlay.
            if ctx.input(|i| {
                i.key_pressed(egui::Key::F1)
                    || (i.key_pressed(egui::Key::Questionmark))
                    || (i.modifiers.shift && i.key_pressed(egui::Key::Slash))
            }) {
                self.help_open = !self.help_open;
            }
        }

        // CLI self-test: run the GPU validation suite, print the report, and exit with a
        // status code (0 = all passed).
        if self.selftest && !self.selftest_done {
            if let Some((dev, q)) = &gpu {
                self.selftest_done = true;
                let ok = self.run_selftest(dev, q);
                std::process::exit(if ok { 0 } else { 1 });
            }
        }

        // CLI profiling: render the benchmark regions, time the costly stages, log to `logs/`.
        if self.profile && !self.profile_done {
            if let Some((dev, q)) = &gpu {
                self.profile_done = true;
                let regions = match &self.profile_regions {
                    Some(path) => match profile::load_regions(std::path::Path::new(path)) {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("--regions {path}: {e}; using built-in regions");
                            profile::default_regions()
                        }
                    },
                    None => profile::default_regions(),
                };
                let secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let out = self.profile_out.clone().unwrap_or_else(|| {
                    std::path::PathBuf::from(format!("logs/profile-{}.json", Self::file_stamp(secs)))
                });
                let reps = self.profile_reps;
                self.run_profile(dev, q, &regions, reps, &out);
                std::process::exit(0);
            }
        }

        if self.frametest {
            if let Some((dev, q)) = &gpu {
                let secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let out = self.profile_out.clone().unwrap_or_else(|| {
                    std::path::PathBuf::from(format!("logs/frametest-{}.json", Self::file_stamp(secs)))
                });
                let (steps, hold, dive) = (self.frametest_steps, self.frametest_hold, self.frametest_dive);
                self.run_frametest(dev, q, steps, hold, dive, 512, &out);
                std::process::exit(0);
            }
        }

        // CLI standardized benchmark (`--benchmark-std [--res] [--burnin] [--depth]`): run all
        // passes synchronously, print + save the report, and quit.
        if self.auto_stdbench && !self.auto_stdbench_done {
            if let Some((dev, q)) = &gpu {
                self.auto_stdbench_done = true;
                let (res, passes, depth) = (self.std_res, self.std_passes, self.std_depth);
                println!(
                    "Fractadyne standardized benchmark — {} · {} × {passes} pass{}",
                    res.label(),
                    depth.label(),
                    if passes == 1 { "" } else { "es" }
                );
                let mut run = self.begin_standard_bench(res, passes, depth);
                // step_std_bench now advances one dive-frame per call; print only when a pass
                // finishes (passes_done ticks up), not on every frame.
                let mut reported = 0u32;
                loop {
                    let done = self.step_std_bench(&mut run, dev, q);
                    if run.passes_done > reported {
                        reported = run.passes_done;
                        println!(
                            "  pass {}/{}  {:.1} fps",
                            run.passes_done,
                            run.passes_total,
                            run.pass_fps.last().copied().unwrap_or(0.0)
                        );
                    }
                    if done {
                        break;
                    }
                }
                let report = self.format_std_bench(&run);
                println!("\n{report}");
                let out = self.auto_benchmark_out.clone().unwrap_or_else(|| {
                    std::path::PathBuf::from("fractadyne_benchmark.txt")
                });
                match std::fs::write(&out, &report) {
                    Ok(()) => println!("\nSaved benchmark → {}", out.display()),
                    Err(e) => eprintln!("Failed to save benchmark to {}: {e}", out.display()),
                }
                std::process::exit(0);
            }
        }

        // GUI standardized benchmark: advance one pass per frame so the window stays responsive
        // (and cancellable) between passes.
        if let Some(mut run) = self.std_bench.take() {
            if let Some((dev, q)) = &gpu {
                let done = self.step_std_bench(&mut run, dev, q);
                if done {
                    self.bench_report = Some(self.format_std_bench(&run));
                    self.bench_open = true;
                    let snap = run.take_snapshot();
                    self.restore_from_bench(snap);
                } else {
                    self.std_bench = Some(run);
                    ctx.request_repaint();
                }
            } else {
                self.std_bench = Some(run); // wait for the GPU handles
            }
        }

        // CLI render-and-exit: render one image offscreen (or the raw iteration EXR), save
        // it, and quit.
        if self.auto_render && !self.auto_render_done {
            if let Some((dev, q)) = &gpu {
                self.auto_render_done = true;
                if !self.watermark && !self.render_iter_mode {
                    println!("Note: Fd watermark is off (saved preference) — pass --watermark to include it.");
                }
                let t0 = std::time::Instant::now();
                let result = if self.render_iter_mode {
                    let out = self
                        .auto_render_out
                        .clone()
                        .unwrap_or_else(|| std::path::PathBuf::from("fractadyne_iter.exr"));
                    self.render_iter_to_file(dev, q, &out)
                } else {
                    let out = self
                        .auto_render_out
                        .clone()
                        .unwrap_or_else(|| std::path::PathBuf::from("fractadyne_render.png"));
                    self.render_to_file(ctx, dev, q, &out)
                };
                match result {
                    Ok(m) => println!("{m}  (in {})", Self::fmt_export_duration(t0.elapsed())),
                    Err(e) => eprintln!("Render failed: {e}"),
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        // CLI render-tour: render the keyframe script to a PNG frame sequence, then quit.
        if let Some(script) = self.render_tour.clone() {
            if !self.render_tour_done {
                if let Some((dev, q)) = &gpu {
                    self.render_tour_done = true;
                    if !self.watermark {
                        println!("Note: Fd watermark is off (saved preference) — pass --watermark to include it.");
                    }
                    let (w, h) = self.tour_size;
                    let (fps, ss, out) = (self.tour_fps, self.tour_ss, self.tour_out.clone());
                    let mp4 = self.tour_mp4.clone();
                    let (prefix, overwrite, resume) = (self.tour_prefix.clone(), self.tour_overwrite, self.tour_resume);
                    match self.render_tour_to_dir(ctx, dev, q, &script, fps, w, h, ss, &out, &prefix, overwrite, resume, mp4.as_deref()) {
                        Ok(m) => println!("{m}"),
                        Err(e) => eprintln!("Tour render failed: {e}"),
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
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

        // Auto-zoom autopilot — dive toward detail (advance before drawing so this frame
        // reflects the new view).
        self.autopilot_step(ctx, &gpu);

        // Palette cycling animation (shifts the color offset over time).
        self.advance_palette_anim(ctx);
        // Orbit racing-dot animation.
        self.advance_orbit_anim(ctx);

        // Poll a background export for completion.
        if let Some(rx) = &self.export.task {
            match rx.try_recv() {
                Ok(msg) => {
                    self.export.status = Some(self.finish_export_status(msg));
                    self.export.task = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => ctx.request_repaint(),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.export.task = None;
                    self.export.started = None;
                }
            }
        }
        // Poll a deep export whose reference is building off-thread. When it lands, assemble the
        // request (reusing the reference — no rebuild) and dispatch the render to a worker. Keeps the
        // UI alive during the (long) bignum reference build instead of freezing.
        if let Some(prep) = self.export.prep.take() {
            match prep.rx.try_recv() {
                Ok(res) => {
                    let (ew, eh, ess) = (self.export.width.max(1), self.export_height(), self.export.ss.max(1));
                    // Map view: reuse the just-built reference (no rebuild). Dual map is Mandelbrot.
                    let map_julia = prep.julia_vp.is_none() && prep.julia_mode;
                    let mut map_req = self.current_export_request_with_ref(&prep.map_vp, map_julia, Some(res));
                    map_req.width = ew;
                    map_req.height = eh;
                    map_req.ss = ess;
                    // Keep per-texel step isotropic for the chosen aspect (see build_export_job).
                    map_req.span_mantissa[1] = map_req.span_mantissa[0] * (eh as f64 / ew as f64);
                    let job = if let Some(jvp) = &prep.julia_vp {
                        // Dual: build the Julia panel now (usually shallow → instant) and combine.
                        let mut jul = self.current_export_request_for(jvp, true);
                        jul.width = ew;
                        jul.height = eh;
                        jul.ss = ess;
                        jul.span_mantissa[1] = jul.span_mantissa[0] * (eh as f64 / ew as f64);
                        match prep.dual_mode {
                            DualExport::SideBySide => ExportJob::SideBySide(map_req, jul),
                            DualExport::Separate => ExportJob::Separate(map_req, jul),
                            DualExport::ActiveOnly => ExportJob::Single(map_req),
                        }
                    } else {
                        ExportJob::Single(map_req)
                    };
                    let hud = self
                        .show_location
                        .then(|| crate::scripting::build_location_overlay(ctx, &prep.map_vp, eh))
                        .flatten();
                    if let Some((dev, q)) = &gpu {
                        self.spawn_export_worker(dev.clone(), q.clone(), job, prep.path, hud);
                    } else {
                        self.export.status = Some("GPU not available".to_string());
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.export.prep = Some(prep); // reference still building
                    ctx.request_repaint();
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.export.status = Some("Export failed: reference build aborted.".to_string());
                    self.export.started = None;
                }
            }
        }
        if let Some(prev) = self.perf.last_frame {
            let dt = frame_start.duration_since(prev).as_secs_f64() * 1000.0;
            self.perf.frame_ms = ema(self.perf.frame_ms, dt);
        }
        self.perf.last_frame = Some(frame_start);

        if !self.auto_benchmark && !self.auto_stdbench && !self.auto_render && !self.selftest && !self.profile && self.render_tour.is_none() && self.std_bench.is_none() {
            self.autosave(ctx); // don't let a CLI run (or a transient benchmark override) overwrite the saved session
        }

        // Combined menu bar + action toolbar. `horizontal_wrapped` keeps them on one
        // line when the window is wide enough, and wraps the toolbar below otherwise.
        // (We place the menu buttons directly in the wrapped row rather than via
        // `menu::bar`, which would claim the full width and push the toolbar down.)
        self.draw_menu_bar(ctx, &gpu);

        self.draw_status_bar(ctx);

        // Right-hand control panel: fractal info, coloring, navigation, and the
        // optional performance section. Hidden entirely while in fullscreen.
        self.draw_right_panel(ctx);

        self.draw_central(ctx);

        self.draw_toast(ctx);
        self.draw_goto_dialog(ctx);
        self.draw_share_dialog(ctx);
        self.draw_reset_dialog(ctx);
        self.draw_bookmarks_dialog(ctx);

        // Render a just-added bookmark's thumbnail (deferred here for GPU access; the current
        // view still matches the bookmark, since adding it didn't move the view).
        self.process_pending_thumb(&gpu);

        self.draw_bench_config_dialog(ctx);
        self.draw_bench_progress_dialog(ctx);
        self.draw_bench_results_dialog(ctx);

        self.draw_gallery_dialog(ctx);

        self.draw_export_dialog(ctx, &gpu);

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
        // Keep repainting while an off-thread reference recompute is in flight, so its result is
        // polled and installed (and the view sharpens) as soon as it lands.
        if self.recompute_rx.iter().any(|r| r.is_some()) {
            ctx.request_repaint();
        }
        self.perf.cpu_ms = ema(self.perf.cpu_ms, frame_start.elapsed().as_secs_f64() * 1000.0);

        // Navigation history: record a location each time the single view settles after
        // a pan/zoom gesture (its own dedup avoids repeats). Discrete jumps record
        // explicitly. Skipped in dual view.
        let interacting_now = ctx.input(|i| i.time) - self.pointer.settle_t[0] < SETTLE_DELAY;
        if self.nav.was_interacting && !interacting_now && !self.dual {
            self.record_nav();
        }
        self.nav.was_interacting = interacting_now;

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

#[cfg(test)]
mod tests {
    use super::*;

    // The go-to / metadata zoom string must round-trip through log2(magnification) at any
    // depth — including past f64's 1e308× range, where a plain f64 zoom would be ∞.
    #[test]
    fn zoom_field_log2_roundtrip() {
        for &log2mag in &[0.0_f64, 8.0, 49.83, 100.0, 1019.0, 1100.0, 5000.0, 1.0e5] {
            let s = fmt_zoom_field(log2mag);
            let back = parse_zoom_to_log2(&s).expect("parse failed");
            assert!((back - log2mag).abs() < 1e-3, "{log2mag} → {s} → {back}");
        }
        // Plain and grouped human input parses too.
        assert!((parse_zoom_to_log2("256").unwrap() - 8.0).abs() < 1e-9);
        assert!((parse_zoom_to_log2("1,024").unwrap() - 10.0).abs() < 1e-9);
        assert!(parse_zoom_to_log2("1e400").unwrap() > 1300.0); // past f64 range, no overflow
        // Garbage rejected, no panic.
        for g in ["", "abc", "-5", "0", "e", "1e", "nan", "inf"] {
            assert!(parse_zoom_to_log2(g).is_none(), "accepted {g:?}");
        }
    }

    // Phase 5.1: fuzz the view-metadata parser chain (untrusted: loaded from PNG tEXt
    // chunks / pasted). `meta_get` + the downstream numeric parsers must never panic and
    // must produce bounded output on arbitrary input.
    #[test]
    fn fuzz_metadata_parser_panic_free() {
        let mut s = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        let charset = b"=\n\r key value-+0.123eE\t\0[]\"";
        for _ in 0..20_000 {
            let len = (next() % 96) as usize;
            let mut buf = String::with_capacity(len);
            for _ in 0..len {
                buf.push(charset[(next() as usize) % charset.len()] as char);
            }
            for k in ["center_x", "center_y", "zoom", "fractal", "julia", "max_iter", "missing"] {
                let v = meta_get(&buf, k);
                assert!(v.len() <= buf.len(), "meta_get returned oversized value");
            }
            // The real downstream parsers applied to extracted values must not panic.
            let _ = fractadyne_core::parse_bf(&meta_get(&buf, "center_x"));
            let _ = fractadyne_core::parse_bf(&meta_get(&buf, "center_y"));
            let _ = meta_get(&buf, "zoom").parse::<f64>();
            let _ = meta_get(&buf, "max_iter").parse::<u32>();
            let _ = FractalKind::from_name(&meta_get(&buf, "fractal"));
        }
        // Adversarial explicit metadata blobs.
        for m in ["", "=", "\n\n\n", "center_x=", "=value", "zoom=NaN", "max_iter=-1",
                  "center_x=1e999999999", "fractal=\0\0\0", "a=b=c=d", "zoom=  inf  "] {
            let _ = fractadyne_core::parse_bf(&meta_get(m, "center_x"));
            let _ = meta_get(m, "zoom").parse::<f64>();
        }
    }
}
