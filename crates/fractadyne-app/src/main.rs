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

/// Auto-zoom autopilot: seconds between target re-evaluations, and the depth (zoom
/// octaves ≈ log₂ magnification) at which it stops — kept in the smooth, fast regime.
const AUTOPILOT_EVAL_INTERVAL: f64 = 0.35;
const AUTOPILOT_LOG2_CAP: f64 = 900.0; // ≈ 1e271×

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

// ---- Fractadyne branding (matches design/Fractadyne.dc.html) ----
/// Brand accent (amber #E0A030) + logotype text color (#E6E7EA). The wordmark is
/// "Fracta" (text) + "dyne" (amber).
const BRAND_ACCENT: egui::Color32 = egui::Color32::from_rgb(0xE0, 0xA0, 0x30);
const BRAND_TEXT: egui::Color32 = egui::Color32::from_rgb(0xE6, 0xE7, 0xEA);

/// Apply the Fractadyne dark theme (charcoal panels + amber accents, per the design).
fn apply_brand_theme(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    let panel = egui::Color32::from_rgb(0x1A, 0x1B, 0x1E);
    v.panel_fill = panel;
    v.window_fill = panel;
    v.extreme_bg_color = egui::Color32::from_rgb(0x10, 0x10, 0x15);
    v.faint_bg_color = egui::Color32::from_rgb(0x23, 0x24, 0x28);
    v.hyperlink_color = BRAND_ACCENT;
    v.selection.bg_fill = egui::Color32::from_rgb(0x4A, 0x38, 0x14);
    v.selection.stroke = egui::Stroke::new(1.0, BRAND_ACCENT);
    v.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(0x23, 0x24, 0x28);
    v.widgets.inactive.bg_fill = egui::Color32::from_rgb(0x2C, 0x2E, 0x33);
    v.widgets.hovered.bg_fill = egui::Color32::from_rgb(0x3A, 0x3D, 0x44);
    v.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(0x3A, 0x3D, 0x44);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, BRAND_ACCENT);
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, BRAND_ACCENT);
    v.widgets.active.bg_fill = egui::Color32::from_rgb(0x45, 0x49, 0x52);
    v.widgets.active.bg_stroke = egui::Stroke::new(1.0, BRAND_ACCENT);
    v.widgets.open.bg_fill = egui::Color32::from_rgb(0x2C, 0x2E, 0x33);
    ctx.set_visuals(v);
}

/// The two-color "Fractadyne" logotype (Fracta + amber dyne), for the top bar.
fn brand_wordmark(ui: &mut egui::Ui) {
    let font = egui::FontId::proportional(15.0);
    let mut job = egui::text::LayoutJob::default();
    job.append(
        "Fracta",
        0.0,
        egui::TextFormat { font_id: font.clone(), color: BRAND_TEXT, ..Default::default() },
    );
    job.append(
        "dyne",
        0.0,
        egui::TextFormat { font_id: font, color: BRAND_ACCENT, ..Default::default() },
    );
    ui.add_space(2.0);
    ui.label(job);
}

/// Procedural window icon: concentric amber rings on a dark disc (transparent corners).
fn brand_icon() -> egui::IconData {
    let n: u32 = 64;
    let mut rgba = vec![0u8; (n * n * 4) as usize];
    let center = (n as f32 - 1.0) * 0.5;
    let acc = [0xE0_u8, 0xA0, 0x30];
    let acc2 = [0xFF_u8, 0xCB, 0x6B];
    let bg = [0x10_u8, 0x10, 0x15];
    for y in 0..n {
        for x in 0..n {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let d = (dx * dx + dy * dy).sqrt() / (n as f32 * 0.5);
            let i = ((y * n + x) * 4) as usize;
            if d > 1.0 {
                rgba[i + 3] = 0; // transparent outside the disc
                continue;
            }
            // concentric rings blending the two accents over the dark disc
            let ring = 0.5 + 0.5 * (d * 22.0).cos();
            let t = (ring * (1.0 - d)).clamp(0.0, 1.0);
            let blend = 0.5 + 0.5 * (d * 6.0).cos();
            let a0 = [
                (acc[0] as f32 * blend + acc2[0] as f32 * (1.0 - blend)),
                (acc[1] as f32 * blend + acc2[1] as f32 * (1.0 - blend)),
                (acc[2] as f32 * blend + acc2[2] as f32 * (1.0 - blend)),
            ];
            for k in 0..3 {
                rgba[i + k] = (bg[k] as f32 + (a0[k] - bg[k] as f32) * t) as u8;
            }
            rgba[i + 3] = 255;
        }
    }
    egui::IconData { rgba, width: n, height: n }
}

// ---- coloring-method <-> persisted-string mapping ----
/// Coloring methods, in selection order (index = GPU `color_method` id).
const COLOR_METHODS: [(&str, &str); 6] = [
    ("smooth", "Smooth iteration"),
    ("stripe", "Stripe average"),
    ("triangle", "Triangle inequality"),
    ("trap", "Orbit trap"),
    ("distance", "Distance estimate"),
    ("decomposition", "Decomposition"),
];
const TRAP_TYPES: [(&str, &str); 3] =
    [("point", "Point"), ("cross", "Cross"), ("circle", "Circle")];

fn method_from_str(s: &str) -> u32 {
    COLOR_METHODS.iter().position(|(k, _)| *k == s).unwrap_or(0) as u32
}
fn method_to_str(m: u32) -> &'static str {
    COLOR_METHODS.get(m as usize).map(|(k, _)| *k).unwrap_or("smooth")
}
fn trap_from_str(s: &str) -> u32 {
    TRAP_TYPES.iter().position(|(k, _)| *k == s).unwrap_or(0) as u32
}
fn trap_to_str(t: u32) -> &'static str {
    TRAP_TYPES.get(t as usize).map(|(k, _)| *k).unwrap_or("point")
}

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
fn zoom_iter_cap(octaves: f64) -> u32 {
    let o = octaves.max(0.0);
    (2000.0 + o * 256.0).min(u32::MAX as f64) as u32
}

/// Plain-f64 **smooth** Mandelbrot dwell, matching the shader exactly (bailout 256,
/// `smooth = iter + 1 − log₂(½·ln|z|² / ln2)`). f64 is dead-accurate at the depths the
/// self-test uses, so this is independent ground truth for the GPU perturbation path.
fn mandel_smooth_f64(cx: f64, cy: f64, max: u32) -> Option<f32> {
    const BAIL2: f64 = 256.0 * 256.0;
    let (mut zx, mut zy) = (0.0_f64, 0.0_f64);
    for iter in 1..=max {
        let (nzx, nzy) = (zx * zx - zy * zy + cx, 2.0 * zx * zy + cy);
        zx = nzx;
        zy = nzy;
        let m2 = zx * zx + zy * zy;
        if m2 > BAIL2 {
            let nu = (m2.ln() * 0.5 / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
            return Some(iter as f32 + 1.0 - nu as f32);
        }
    }
    None
}

/// FNV-1a 64-bit checksum (no deps) — a content fingerprint for golden images so a
/// third party can confirm they're looking at the same reference bytes.
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Unix seconds → "YYYY-MM-DD HH:MM:SS UTC" (civil-date algorithm; no chrono dependency).
fn utc_string(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let r = (secs % 86_400) as i64;
    let (hh, mm, ss) = (r / 3600, (r % 3600) / 60, r % 60);
    let z = days + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}:{ss:02} UTC")
}

/// Per-channel 8-bit image difference: `(max abs, mean abs)`. Mismatched sizes → worst.
fn img_diff(a: &[u8], b: &[u8]) -> (u32, f64) {
    if a.len() != b.len() || a.is_empty() {
        return (255, 255.0);
    }
    let (mut max, mut sum) = (0u32, 0u64);
    for (&x, &y) in a.iter().zip(b) {
        let d = (x as i32 - y as i32).unsigned_abs();
        if d > max {
            max = d;
        }
        sum += d as u64;
    }
    (max, sum as f64 / a.len() as f64)
}

/// One row of the validation report.
struct SelfCheck {
    category: &'static str,
    name: String,
    params: String,
    result: String,
    threshold: &'static str,
    pass: bool,
}

/// Machine-readable validation catalog (`validation/catalog.toml`) — locations with
/// independently verifiable answers, consumed by `--selftest` (Phase 6.1 / 6.6).
#[derive(serde::Deserialize, Default)]
struct Catalog {
    #[serde(default)]
    nucleus: Vec<NucleusEntry>,
    #[serde(default)]
    membership: Vec<MemberEntry>,
}

#[derive(serde::Deserialize)]
struct NucleusEntry {
    name: String,
    #[serde(default)]
    fractal: Option<String>,
    center_x: String,
    center_y: String,
    zoom: f64,
    period: u32,
    #[serde(default)]
    nucleus_x: Option<String>,
    #[serde(default)]
    nucleus_y: Option<String>,
}

#[derive(serde::Deserialize)]
struct MemberEntry {
    name: String,
    center_x: String,
    center_y: String,
    interior: bool,
}

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
fn help_h(ui: &mut egui::Ui, t: &str) {
    ui.add_space(2.0);
    ui.label(egui::RichText::new(t).size(18.0).strong().color(BRAND_ACCENT));
    ui.add_space(4.0);
}
fn help_sub(ui: &mut egui::Ui, t: &str) {
    ui.add_space(8.0);
    ui.label(egui::RichText::new(t).strong().color(BRAND_TEXT));
    ui.add_space(2.0);
}
fn help_p(ui: &mut egui::Ui, t: &str) {
    ui.label(t);
    ui.add_space(3.0);
}
fn help_bullet(ui: &mut egui::Ui, t: &str) {
    ui.horizontal_top(|ui| {
        ui.add_space(4.0);
        ui.label("•");
        ui.add(egui::Label::new(t).wrap());
    });
}
/// A monospace key + wrapped description row (shortcuts / CLI flags).
fn help_kv(ui: &mut egui::Ui, k: &str, v: &str) {
    ui.horizontal_top(|ui| {
        ui.add_sized(
            [180.0, 0.0],
            egui::Label::new(egui::RichText::new(k).monospace()).wrap(),
        );
        ui.add(egui::Label::new(v).wrap());
    });
}

fn help_overview(ui: &mut egui::Ui) {
    help_h(ui, "Overview");
    help_p(
        ui,
        "Fractadyne is a native fractal explorer built for ultra-deep zooming and speed. \
         It draws \"escape-time\" fractals — images created by repeating one simple formula \
         at every pixel.",
    );
    help_sub(ui, "What is an escape-time fractal?");
    help_p(
        ui,
        "For each pixel the program runs a formula such as z → z² + c over and over, starting \
         from zero. If the running value stays small forever, the pixel belongs to the set and \
         is drawn dark. If it eventually grows without bound (\"escapes\"), the pixel is outside \
         the set, and its color records how many steps that took. The infinitely detailed \
         border between \"stays\" and \"escapes\" is the fractal.",
    );
    help_sub(ui, "What you can do");
    help_bullet(ui, "Pan and zoom essentially without limit (position is exact at any depth).");
    help_bullet(ui, "Switch between ten fractal families, and view any as a Julia set.");
    help_bullet(ui, "Recolor with preset or custom gradients and several coloring methods.");
    help_bullet(ui, "Add 3D relief lighting and glowing boundary contours.");
    help_bullet(ui, "Snap to minibrots, bookmark spots, and export high-resolution images.");
    help_bullet(ui, "Run scripted tours and a hardware benchmark.");
    help_sub(ui, "First steps");
    help_p(
        ui,
        "Open the Locations menu and pick \"Seahorse Valley\", then roll the mouse wheel to \
         zoom in. Drag to pan. Press F1 at any time to return to this help.",
    );
}

fn help_navigation(ui: &mut egui::Ui) {
    help_h(ui, "Navigation");
    help_sub(ui, "Mouse");
    help_kv(ui, "Left-drag", "Pan the view.");
    help_kv(ui, "Mouse wheel", "Zoom in/out toward the cursor.");
    help_kv(ui, "Right-drag", "Box zoom — drag a rectangle to zoom into it.");
    help_sub(ui, "Continuous zoom & home");
    help_kv(ui, "Hold Space", "Smoothly zoom in, anchored at the cursor.");
    help_kv(ui, "Hold Shift+Space", "Smoothly zoom out.");
    help_p(
        ui,
        "The Zoom-home button animates a gentle fly-back to the full view. \"Zoom speed\" in the \
         right panel sets the continuous-zoom rate.",
    );
    help_sub(ui, "History & precise moves");
    help_kv(ui, "Backspace", "Undo the previous view.");
    help_kv(ui, "Shift+Backspace / Ctrl+Y", "Redo.");
    help_p(
        ui,
        "View → \"Go to location…\" lets you read, type, paste, or copy the exact center and zoom \
         (full precision) — the easy way to share or revisit a spot. The Bookmarks menu saves and \
         recalls locations.",
    );
    help_sub(ui, "Finding detail");
    help_p(
        ui,
        "Minibrots are tiny copies of the whole set hidden along the boundary. Center one roughly \
         and press M (or View → \"Find minibrot center\") to Newton-snap exactly onto its center \
         and read its period.",
    );
    help_p(
        ui,
        "View → \"Minimap overview\" shows a small map of the whole set with a \"you are here\" \
         marker and the current zoom depth; click the map to jump to a region.",
    );
}

fn help_options(ui: &mut egui::Ui) {
    help_h(ui, "Coloring & options");
    help_p(ui, "All of these live in the right-hand panel (and persist between sessions).");
    help_sub(ui, "Palette");
    help_p(
        ui,
        "Palette chooses a preset gradient, your Custom one, or a two-color mode. \"Edit \
         gradient…\" opens an editor where each color stop has a color and a position (0–1); add \
         up to eight stops or copy a preset to start from.",
    );
    help_p(
        ui,
        "Duotone maps the value to a smooth two-color ramp (Shadow → Highlight). Binary (set) \
         is a flat two-color view — one solid color for points inside the set and another for \
         outside, with no gradient — the clearest way to see the set's shape.",
    );
    help_p(
        ui,
        "Cycle sets how many times the gradient repeats across the iteration range (tighter or \
         looser bands). Offset rotates the whole gradient.",
    );
    help_p(
        ui,
        "Animate cycles the colors over time — Forward, Reverse, or Ping-pong shift the offset; \
         Random continuously synthesizes smoothly morphing, harmonious gradients. Speed controls \
         cycles (or morphs) per second; \"Shuffle gradient\" rolls a new one in Random mode.",
    );
    help_sub(ui, "Coloring method — how the data becomes color");
    help_kv(ui, "Smooth iteration", "Classic continuous bands by escape time.");
    help_kv(ui, "Stripe average", "Flowing bands from the orbit's angle (Stripe density slider).");
    help_kv(ui, "Triangle inequality", "Fine texture from where each step lands between bounds.");
    help_kv(ui, "Orbit trap", "Distance of the orbit to a shape (point/cross/circle); colors interior too.");
    help_kv(ui, "Distance estimate", "Shades by nearness to the boundary.");
    help_kv(ui, "Decomposition", "Cells from the final escape angle.");
    help_sub(ui, "3D relief lighting");
    help_p(
        ui,
        "Shades the surface from the boundary's slope (the derivative) for an embossed, lit look. \
         Light angle sets the direction; Relief sets strength (lower = sharper); \"Rotate light\" \
         animates it. Holomorphic families only (Mandelbrot / Multibrot).",
    );
    help_sub(ui, "Distance glow");
    help_p(
        ui,
        "Bright contour bands that densify into glowing filaments near the boundary. Glow is the \
         blend amount, Band width the spacing, and \"Animate glow\" flows them.",
    );
    help_sub(ui, "Quality & iterations");
    help_p(
        ui,
        "Iterations is the maximum number of steps before a pixel is treated as inside the set; \
         \"Auto-scale\" raises it automatically as you zoom (deeper detail needs more). Anti-alias \
         supersamples still images (2×–8×) once the view settles, taming the fine exterior \"dust\".",
    );
    help_sub(ui, "Other");
    help_p(
        ui,
        "Zoom speed sets the continuous-zoom rate; the FPS cap limits frame rate; Dual view shows a \
         Mandelbrot set and its Julia set side by side (the cursor sets the Julia parameter live).",
    );
}

fn help_fractals(ui: &mut egui::Ui) {
    help_h(ui, "Fractals");
    help_p(
        ui,
        "Every family iterates a formula with z starting at 0 and c set by the pixel (escape-time), \
         unless noted. z = x + iy is a complex number.",
    );
    help_sub(ui, "Mandelbrot");
    help_p(
        ui,
        "z → z² + c. The original. The set is every c whose orbit stays bounded; it is connected, \
         and its boundary is so crinkled it has Hausdorff dimension 2.",
    );
    help_sub(ui, "Multibrot 3 / 4 / 5");
    help_p(
        ui,
        "z → zᵈ + c for power d = 3, 4, 5. Higher powers add lobes: the set has (d−1)-fold \
         rotational symmetry (Multibrot 3 is 2-fold, 4 is 3-fold, 5 is 4-fold).",
    );
    help_sub(ui, "Tricorn (Mandelbar)");
    help_p(
        ui,
        "z → conj(z)² + c, where conj(x + iy) = x − iy. The conjugation makes it anti-holomorphic, \
         giving 3-fold symmetry and characteristic curved \"claws\".",
    );
    help_sub(ui, "Burning Ship");
    help_p(
        ui,
        "z → (|x| + i|y|)² + c — take absolute values before squaring (real = x²−y²+cx, \
         imag = 2|xy|+cy). Non-analytic; deep zooms reveal ship-like structures (traditionally \
         viewed upside-down).",
    );
    help_sub(ui, "Celtic");
    help_p(
        ui,
        "Like Mandelbrot but with the absolute value of the real part: real = |x²−y²| + cx, \
         imag = 2xy + cy. Produces heart- and shield-shaped motifs.",
    );
    help_sub(ui, "Buffalo");
    help_p(
        ui,
        "Absolute value of both parts of z²: real = |x²−y²| + cx, imag = |2xy| + cy — a cross \
         between Celtic and Burning Ship.",
    );
    help_sub(ui, "Phoenix");
    help_p(
        ui,
        "z → z² + c + p·z₋₁, where z₋₁ is the previous iterate and p is a constant (here p = −0.5). \
         The memory term produces flame-like filaments.",
    );
    help_sub(ui, "Newton");
    help_p(
        ui,
        "Newton's method for the roots of z³ − 1 = 0: z → z − (z³−1)/(3z²). Rather than escape time, \
         pixels are colored by which of the three cube roots of unity the iteration converges to \
         (the basins of attraction) and how quickly; the tangled basin boundaries are the fractal.",
    );
    help_sub(ui, "Julia mode");
    help_p(
        ui,
        "For every family except Newton you can switch to a Julia set: instead of starting z at 0 \
         with c = pixel, you fix c (a parameter) and let z start at the pixel. The Julia set is the \
         boundary between starting points that stay bounded and those that escape, for that fixed c. \
         In Dual view, moving the cursor over the Mandelbrot panel sets c live.",
    );
    help_sub(ui, "Deep-zoom support");
    help_p(
        ui,
        "Mandelbrot and Multibrot 3/4/5 support unlimited perturbation deep zoom. The abs-based \
         families (Tricorn, Burning Ship, Celtic, Buffalo) and Phoenix/Newton currently use the \
         direct path, which stays sharp to about 10⁶×.",
    );
}

fn help_methodology(ui: &mut egui::Ui) {
    help_h(ui, "How it works");
    help_sub(ui, "Escape-time & smooth color");
    help_p(
        ui,
        "Each pixel iterates the formula until its magnitude exceeds a bailout radius or it hits the \
         iteration cap. The raw step count alone makes hard bands; adding a fractional part derived \
         from the final magnitude gives continuous, bandless color.",
    );
    help_sub(ui, "Arbitrary-precision position");
    help_p(
        ui,
        "Ordinary 64-bit numbers run out of digits near 10¹⁵× zoom. Fractadyne keeps the view center \
         in arbitrary precision, with the number of digits growing as you zoom, so the location never \
         degrades — you can keep going essentially forever.",
    );
    help_sub(ui, "Perturbation");
    help_p(
        ui,
        "Iterating every pixel in high precision would be far too slow. Instead one reference pixel \
         is iterated in high precision on the CPU (the \"reference orbit\"), and every other pixel is \
         computed on the GPU as a tiny difference δ from it in fast low precision: \
         δz → 2·Z·δz + δz² + δc.",
    );
    help_sub(ui, "Unlimited depth (floatexp)");
    help_p(
        ui,
        "Past about 10²⁸× even that tiny difference underflows 32-bit range, so it is stored as a \
         mantissa plus a separate integer exponent (\"floatexp\"), removing the depth wall. The \
         engine switches automatically: direct math when shallow, perturbation when deep, and \
         floatexp when deepest.",
    );
    help_sub(ui, "Reference choice & rebasing");
    help_p(
        ui,
        "The reference is chosen (scored in high precision) so its orbit stays within the view as \
         long as possible. When the difference grows too large it is \"rebased\" back onto the \
         reference to stay accurate.",
    );
    help_sub(ui, "Distance estimation & lighting");
    help_p(
        ui,
        "Tracking the derivative dz/dc yields each pixel's distance to the boundary. That powers the \
         3D relief lighting, the distance glow, and the \"distance\" coloring method — all valid at \
         any zoom depth.",
    );
    help_sub(ui, "Anti-aliasing & safety");
    help_p(
        ui,
        "Still images are supersampled (2–8× per axis) once the view settles. A work budget keeps a \
         single GPU draw within the driver's watchdog limit by reducing resolution (never the \
         iteration count) at extreme settings, so deep views stay detailed instead of going blank.",
    );
}

fn help_command_line(ui: &mut egui::Ui) {
    help_h(ui, "Command line");
    help_p(
        ui,
        "Fractadyne can run headless for automation, golden-image checks, and benchmarking. Flags:",
    );
    help_sub(ui, "Modes");
    help_kv(ui, "--render", "Render one image and exit.");
    help_kv(ui, "--out PATH, -o PATH", "Output file (PNG or EXR by extension).");
    help_kv(ui, "--benchmark, --bench", "Run the benchmark tour and exit (use --out to save).");
    help_kv(ui, "--find-minibrot", "Print the nearby minibrot's period + center and exit.");
    help_sub(ui, "View");
    help_kv(ui, "--fractal NAME", "Family, e.g. \"Mandelbrot\" or \"Burning Ship\".");
    help_kv(ui, "--center X Y", "View center (full-precision decimals).");
    help_kv(ui, "--zoom M", "Magnification (f64, ≤ ~1e308×).");
    help_kv(ui, "--zoom-log2 L", "Magnification = 2^L — for depths past f64 range (≥ ~1e308×).");
    help_kv(ui, "--julia", "Julia mode.");
    help_kv(ui, "--julia-c RE IM", "Julia parameter c.");
    help_sub(ui, "Image & color");
    help_kv(ui, "--size W", "Image width in pixels (height from aspect).");
    help_kv(ui, "--ss N", "Supersampling 1–8.");
    help_kv(ui, "--iter N", "Maximum iterations.");
    help_kv(ui, "--palette N", "Preset palette index.");
    help_kv(ui, "--method NAME", "smooth | stripe | triangle | trap | distance | decomposition.");
    help_kv(ui, "--stripe-freq N", "Stripe density (stripe method).");
    help_kv(ui, "--trap SHAPE", "point | cross | circle (orbit-trap method).");
    help_kv(ui, "--light [--light-angle R]", "Enable 3D relief lighting.");
    help_kv(ui, "--de", "Enable distance glow.");
    help_kv(ui, "--no-perf / --perf", "Hide / show the performance panel.");
    help_sub(ui, "Validation");
    help_kv(ui, "--selftest [--bless]", "Run the validation suite; exit 0 = all passed (--bless records goldens).");
    help_kv(ui, "--render-iter -o F.exr", "Export raw iteration data (EXR) instead of a colored image.");
    help_kv(ui, "--compare A B", "Diff two renders/EXRs: max/mean Δ + difference heatmap.");
    help_kv(ui, "--import-kfr F.kfr", "Load a Kalles Fraktaler location.");
    help_kv(
        ui,
        "--validate-deep",
        "Extreme-depth precision self-consistency battery (1e1000…1e1000000×).",
    );
    help_kv(
        ui,
        "--crosscheck-f3 raw.exr",
        "Compare a Fraktaler-3 raw EXR (channel \"N\") against our CPU bignum oracle \
         (--center X Y --zoom-f3 Z [--iter K] [--er R]).",
    );
    help_sub(ui, "Example");
    ui.label(
        egui::RichText::new(
            "fractadyne --render -o out.png --fractal Mandelbrot \\\n  \
             --center -0.743644 0.131826 --zoom 2e7 --iter 6000 --method stripe --ss 3",
        )
        .monospace()
        .small(),
    );
}

fn help_shortcuts(ui: &mut egui::Ui) {
    help_h(ui, "Shortcuts");
    help_sub(ui, "Mouse");
    help_kv(ui, "Left-drag", "Pan");
    help_kv(ui, "Wheel", "Zoom at cursor");
    help_kv(ui, "Right-drag", "Box zoom");
    help_sub(ui, "Keyboard");
    help_kv(ui, "Space / Shift+Space", "Continuous zoom in / out");
    help_kv(ui, "Backspace", "Undo view");
    help_kv(ui, "Shift+Backspace / Ctrl+Y", "Redo view");
    help_kv(ui, "M", "Find minibrot center");
    help_kv(ui, "A", "Auto-zoom autopilot (dive toward detail; any input stops)");
    help_kv(ui, "Esc", "Exit fullscreen / stop a playing tour");
    help_kv(ui, "Ctrl+S", "Quick export to the last folder");
    help_kv(ui, "F1 / ?", "Open this help");
}

fn help_about(ui: &mut egui::Ui) {
    help_h(ui, "About");
    help_p(ui, &format!("Fractadyne v{}", version_string()));
    help_p(ui, "A native fractal explorer built in Rust with wgpu.");
    help_sub(ui, "License");
    help_p(ui, "MIT OR Apache-2.0 — use under either license, at your option.");
    help_p(ui, "© 2026 Rithea Hong.");
    ui.hyperlink_to("Source on GitHub \u{2197}", "https://github.com/WindySnowOwl/fractadyne");
}

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

    // Headless minibrot finder (for automation / validation):
    //   --find-minibrot --center X Y [--zoom M] [--fractal NAME]
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--find-minibrot") {
        let val = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1));
        let two = |name: &str| {
            args.iter()
                .position(|a| a == name)
                .and_then(|i| Some((args.get(i + 1)?, args.get(i + 2)?)))
        };
        let formula = val("--fractal")
            .and_then(|s| FractalKind::from_name(s))
            .map(|k| k.formula_id())
            .unwrap_or(0);
        let center = two("--center")
            .and_then(|(x, y)| Some([fractadyne_core::parse_bf(x)?, fractadyne_core::parse_bf(y)?]))
            .unwrap_or([
                fractadyne_core::BigFloat::from_f64(-0.5, 64),
                fractadyne_core::BigFloat::from_f64(0.0, 64),
            ]);
        let mag = val("--zoom").and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0);
        match fractadyne_core::find_nucleus(&center, mag, formula, 100_000) {
            Some(n) => println!(
                "period {}\ncenter_x {}\ncenter_y {}",
                n.period,
                fractadyne_core::to_decimal_string(&n.cx),
                fractadyne_core::to_decimal_string(&n.cy),
            ),
            None => println!("no minibrot center found"),
        }
        return Ok(());
    }

    // Headless A/B comparison (no GPU): diff two renders / exported iteration files.
    //   --compare A B [--out heatmap.png]
    if let Some(i) = args.iter().position(|a| a == "--compare") {
        let (a, b) = (args.get(i + 1), args.get(i + 2));
        let (Some(a), Some(b)) = (a, b) else {
            eprintln!("--compare needs two file paths");
            return Ok(());
        };
        let out = args
            .iter()
            .position(|x| x == "--out" || x == "-o")
            .and_then(|j| args.get(j + 1))
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("compare_heatmap.png"));
        let load = |p: &str| -> Option<(u32, u32, Vec<f32>)> {
            let path = std::path::Path::new(p);
            match path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref() {
                Some("exr") => fractadyne_export::read_exr_rgba_f32(path),
                Some("png") => fractadyne_export::read_png_rgba8(path)
                    .map(|(w, h, bytes)| (w, h, bytes.iter().map(|&x| x as f32).collect())),
                _ => None,
            }
        };
        match (load(a), load(b)) {
            (Some((wa, ha, pa)), Some((wb, hb, pb))) if wa == wb && ha == hb => {
                let n = (wa as usize) * (ha as usize);
                // Channel 0 (smooth iteration for EXR, red for PNG) is the primary signal.
                // Channel 0 (smooth iteration / red) is always finite. The DE/normal
                // channels carry ±∞/1e30 sentinels for interior/unavailable, so the
                // all-channel stats skip non-finite and sentinel-scale diffs.
                let (mut max0, mut sum0, mut differ) = (0.0f64, 0.0f64, 0u64);
                let (mut maxall, mut sumall, mut nall) = (0.0f64, 0.0f64, 0u64);
                for k in 0..n {
                    let d0 = (pa[k * 4] - pb[k * 4]).abs() as f64;
                    max0 = max0.max(d0);
                    sum0 += d0;
                    if d0 > 1e-6 {
                        differ += 1;
                    }
                    for c in 0..4 {
                        let d = (pa[k * 4 + c] - pb[k * 4 + c]).abs() as f64;
                        if d.is_finite() && d < 1.0e20 {
                            maxall = maxall.max(d);
                            sumall += d;
                            nall += 1;
                        }
                    }
                }
                println!("Comparison: {a}  vs  {b}");
                println!("  size {wa}×{ha}");
                println!("  channel 0: max Δ {max0:.6}, mean Δ {:.6}, {differ}/{n} pixels differ", sum0 / n as f64);
                println!("  all channels (finite): max Δ {maxall:.6}, mean Δ {:.6}", sumall / nall.max(1) as f64);
                // Heatmap of |Δ channel 0|, normalized to the max (grayscale).
                let scale = if max0 > 0.0 { 1.0 / max0 as f32 } else { 0.0 };
                let mut heat = vec![0.0f32; n * 4];
                for k in 0..n {
                    let t = (pa[k * 4] - pb[k * 4]).abs() * scale;
                    heat[k * 4] = t;
                    heat[k * 4 + 1] = t;
                    heat[k * 4 + 2] = t;
                    heat[k * 4 + 3] = 1.0;
                }
                match fractadyne_export::write_png(&out, wa, ha, &heat, None) {
                    Ok(()) => println!("  heatmap → {}", out.display()),
                    Err(e) => eprintln!("  heatmap write failed: {e}"),
                }
            }
            (Some((wa, ha, _)), Some((wb, hb, _))) => {
                eprintln!("dimension mismatch: {wa}×{ha} vs {wb}×{hb}");
                return Ok(());
            }
            _ => {
                eprintln!("failed to load one or both inputs (PNG/EXR only)");
                return Ok(());
            }
        }
        return Ok(());
    }

    // Cross-renderer validation against **Fraktaler-3** (no GPU): compare F3's raw
    // integer escape counts (EXR channel "N", UINT) against our *independent* CPU
    // arbitrary-precision dwell oracle at the identical complex coordinate of every
    // pixel. Two fully independent engines (F3's GPU perturbation vs our bignum CPU)
    // agreeing on exact integer iteration counts is the strongest external check.
    //
    //   --crosscheck-f3 raw.exr --center X Y --zoom-f3 Z [--iter K] [--er 256]
    //
    // Render the F3 side with (in a .f3.toml batch): render.save_exr = true,
    // render.exr_channels = ["N0"], image.subframes = 1, transform.exponential_map = false.
    if let Some(i) = args.iter().position(|a| a == "--crosscheck-f3") {
        let Some(file) = args.get(i + 1) else {
            eprintln!("--crosscheck-f3 needs an EXR path");
            return Ok(());
        };
        let val = |name: &str| args.iter().position(|a| a == name).and_then(|j| args.get(j + 1));
        let two = |name: &str| {
            args.iter()
                .position(|a| a == name)
                .and_then(|j| Some((args.get(j + 1)?, args.get(j + 2)?)))
        };
        let center = two("--center")
            .and_then(|(x, y)| Some([fractadyne_core::parse_bf(x)?, fractadyne_core::parse_bf(y)?]))
            .unwrap_or([
                fractadyne_core::BigFloat::from_f64(-0.5, 64),
                fractadyne_core::BigFloat::from_f64(0.0, 64),
            ]);
        let f3_zoom = val("--zoom-f3").and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0);
        let max_iter = val("--iter").and_then(|s| s.parse::<u32>().ok()).unwrap_or(10_000);
        let er = val("--er").and_then(|s| s.parse::<f64>().ok()).unwrap_or(256.0);
        let bailout2 = er * er;
        // Our magnification convention (height 3) vs F3's (height 4): mag = 0.75·zoom.
        let our_mag = 0.75 * f3_zoom;
        let prec = fractadyne_core::precision_for_magnification(our_mag).max(64);

        let Some((w, h, nch)) = fractadyne_export::read_exr_channel_f32(std::path::Path::new(file), "N")
        else {
            eprintln!(
                "could not read EXR channel \"N\" from {file}. Channels present: {:?}\n\
                 (Fraktaler-3 batch must set render.exr_channels = [\"N0\"] and render.save_exr = true.)",
                fractadyne_export::list_exr_channels(std::path::Path::new(file)).unwrap_or_default()
            );
            return Ok(());
        };
        let (wf, hf) = (w as f64, h as f64);
        // F3 pixel spacing; saved EXR is vertically flipped (vertical_flip defaults false
        // ⇒ save_exr flips), so saved row y maps to kernel row h-1-y.
        let spacing = 4.0 / f3_zoom / hf;
        let cx0 = &center[0];
        let cy0 = &center[1];
        // F3 interior sentinel: N0 = 0xFFFFFFFF (reads as ~4.29e9 in f32); exterior n = N0 - 1024.
        let is_interior_f3 = |v: f32| v > 2.0e9;
        let n_f3 = |v: f32| (v - 1024.0).round() as i64;

        eprintln!("Fraktaler-3 cross-check: {file}");
        eprintln!("  {w}×{h}, F3 zoom {f3_zoom:e} (our mag {our_mag:e}), iter {max_iter}, escape_radius {er}");
        eprintln!("  oracle: independent arbitrary-precision CPU dwell ({prec}-bit), bailout² {bailout2}");

        // F3 jitters every sample by a deterministic hash-based triangular sub-pixel offset
        // (anti-aliasing reconstruction, applied even at subframes=1). To compare integer
        // counts exactly we must sample our oracle at F3's *actual* point, not the pixel
        // centre — so replicate the kernel's jitter (hybrid.h: burtle_hash/triangle/wrap;
        // for subframe 0, dx == dy == triangle(burtle_hash(ix)/2³²)).
        let burtle_hash = |mut a: u32| -> u32 {
            a = a.wrapping_add(0x7ed5_5d16).wrapping_add(a << 12);
            a = (a ^ 0xc761_c23c) ^ (a >> 19);
            a = a.wrapping_add(0x1656_67b1).wrapping_add(a << 5);
            a = a.wrapping_add(0xd3a2_646c) ^ (a << 9);
            a = a.wrapping_add(0xfd70_46c5).wrapping_add(a << 3);
            a = (a ^ 0xb55a_4f09) ^ (a >> 16);
            a
        };
        let triangle = |h: f64| -> f64 {
            let orig = h * 2.0 - 1.0;
            let v = (orig / orig.abs().sqrt()).max(-1.0);
            v - if orig >= 0.0 { 1.0 } else { -1.0 }
        };
        let (wi, hi) = (w as i64, h as i64);

        // Pass 1: our oracle escape count at each pixel's exact (jittered) c (interior ⇒ -1).
        let oracle_n: Vec<i64> = (0..(w as usize * h as usize))
            .map(|k| {
                let (x, y) = ((k % w as usize) as i64, (k / w as usize) as i64);
                // Saved EXR is vertically flipped ⇒ kernel (i, j) = (x, h-1-y).
                let (ki, kj) = (x, hi - 1 - y);
                let ix = ((kj * wi + ki) & 0xffff_ffff) as u32;
                let jit = triangle(burtle_hash(ix) as f64 / 4_294_967_296.0);
                let ox = ((ki as f64 + 0.5 + jit) - wf / 2.0) * spacing;
                let oy = ((kj as f64 + 0.5 + jit) - hf / 2.0) * spacing;
                let cx = fractadyne_core::add_f64(cx0, ox, prec);
                let cy = fractadyne_core::add_f64(cy0, oy, prec);
                match fractadyne_core::naive_dwell_bf(&cx, &cy, max_iter, bailout2, prec) {
                    Some((n, _)) => n as i64,
                    None => -1,
                }
            })
            .collect();

        // Pass 2: compare, excluding ill-conditioned boundary pixels (a 4-neighbour
        // flips interior/exterior, or our oracle's count jumps by >2 — those pixels are
        // sub-pixel-sensitive and the two engines legitimately sample slightly differently).
        let idx = |x: usize, y: usize| y * w as usize + x;
        let (mut interior_ok, mut interior_tot) = (0u64, 0u64);
        let (mut exact, mut within1, mut smooth_tot, mut boundary, mut maxd) = (0u64, 0u64, 0u64, 0u64, 0i64);
        let (mut worst, mut worst_at) = (0i64, (0usize, 0usize));
        for y in 0..h as usize {
            for x in 0..w as usize {
                let k = idx(x, y);
                let o = oracle_n[k];
                let fi = is_interior_f3(nch[k]);
                let oi = o < 0;
                // Boundary detection via oracle neighbourhood.
                let mut steep = false;
                for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                    let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let no = oracle_n[idx(nx as usize, ny as usize)];
                    if (no < 0) != oi || (no >= 0 && o >= 0 && (no - o).abs() > 2) {
                        steep = true;
                    }
                }
                // Exclude ill-conditioned pixels from BOTH metrics. A pixel sitting on the
                // max-iteration cliff (a 4-neighbour flips interior/exterior) is `steep`, so
                // this also keeps near-cliff membership flips — legitimately ambiguous to the
                // last ULP — out of the membership stat.
                if steep {
                    boundary += 1;
                    continue;
                }
                if fi || oi {
                    interior_tot += 1;
                    if fi == oi {
                        interior_ok += 1;
                    }
                    continue;
                }
                smooth_tot += 1;
                let d = (n_f3(nch[k]) - o).abs();
                if d == 0 {
                    exact += 1;
                }
                if d <= 1 {
                    within1 += 1;
                }
                maxd = maxd.max(d);
                if d > worst {
                    worst = d;
                    worst_at = (x, y);
                }
            }
        }
        let pct = |a: u64, b: u64| if b == 0 { 100.0 } else { 100.0 * a as f64 / b as f64 };
        eprintln!(
            "  interior/exterior membership: {interior_ok}/{interior_tot} agree ({:.3}%)",
            pct(interior_ok, interior_tot)
        );
        eprintln!(
            "  smooth-region exterior counts: exact {exact}/{smooth_tot} ({:.3}%), |Δ|≤1 {within1}/{smooth_tot} ({:.3}%)",
            pct(exact, smooth_tot),
            pct(within1, smooth_tot)
        );
        eprintln!("  max |Δn| (non-boundary) {maxd} at pixel {worst_at:?}; boundary pixels excluded {boundary}");
        // PASS: membership ≥99.5%, and ≥99% of smooth-region exterior pixels match within 1.
        let pass = pct(interior_ok, interior_tot) >= 99.5 && pct(within1, smooth_tot) >= 99.0;
        println!("crosscheck-f3: {}", if pass { "PASS" } else { "FAIL" });
        std::process::exit(if pass { 0 } else { 1 });
    }

    // Extreme-depth validation battery (no GPU, no external data): exercises the
    // arbitrary-precision arithmetic core at magnifications far beyond f64 range
    // (1e1000 … 1e1000000), via precision-doubling self-consistency + coordinate round-trip.
    // A per-pixel dwell oracle is infeasible this deep; these single-point checks are not.
    //   --validate-deep [--out report.md]
    if args.iter().any(|a| a == "--validate-deep") {
        use std::time::Instant;
        let out = args
            .iter()
            .position(|x| x == "--out" || x == "-o")
            .and_then(|j| args.get(j + 1))
            .map(std::path::PathBuf::from);
        // (decimal exponent, iteration count) — fewer iters as precision grows (cost ∝ bits·k).
        let battery: &[(f64, u32)] = &[(1_000.0, 20_000), (10_000.0, 4_000), (100_000.0, 800), (1_000_000.0, 200)];
        let guard = 256usize;
        let mut rows: Vec<String> = Vec::new();
        let mut all_ok = true;
        println!("Extreme-depth precision self-consistency (arbitrary-precision arithmetic core)");
        println!(
            "{:>12} {:>11} {:>7} {:>7} {:>13} {:>13} {:>9}  {}",
            "magnif.", "bits", "limbs", "k", "agree(bits)", "rt(bits)", "time(s)", "result"
        );
        for (exp, k) in battery.iter().copied() {
            let octaves = (exp * std::f64::consts::LN_10 / std::f64::consts::LN_2).ceil() as u64;
            let p = fractadyne_core::precision_for_octaves(octaves);
            let t = Instant::now();
            let agree = fractadyne_core::deep_consistency_bits(p, guard, k);
            let rt = fractadyne_core::deep_roundtrip_bits(p);
            let secs = t.elapsed().as_secs_f64();
            // Sound p-bit arithmetic agrees to ≈ p − log₂(k); allow a generous margin.
            let pass = agree >= p as i64 - 128 && rt >= p as i64 - 256;
            all_ok &= pass;
            let verdict = if pass { "PASS" } else { "FAIL" };
            println!(
                "      1e{:<5.0} {:>11} {:>7} {:>7} {:>13} {:>13} {:>9.2}  {}",
                exp, p, p / 64, k, agree, rt, secs, verdict
            );
            rows.push(format!(
                "| 1e{:.0} | {} | {} | {} | {} | {} | {:.2} | {} |",
                exp, p, p / 64, k, agree, rt, secs, verdict
            ));
        }
        if let Some(path) = out {
            let mut md = String::new();
            md.push_str("# Extreme-depth precision validation\n\n");
            md.push_str(&format!("Fractadyne {}\n\n", version_string()));
            md.push_str(
                "Precision-doubling self-consistency of the arbitrary-precision arithmetic core, at \
                 magnifications beyond `f64` range. Iterate `z²+c` (full-mantissa interior point) at \
                 precision `p` and at `p+256`; `agree` = leading base-2 bits that match (sound ≈ \
                 `p − log₂(k)`). `rt` = bits preserved through a decimal `to_string → parse` \
                 round-trip. No GPU, no external data.\n\n",
            );
            md.push_str("| magnification | bits | limbs | k iters | agree (bits) | round-trip (bits) | time (s) | result |\n");
            md.push_str("|---|---|---|---|---|---|---|---|\n");
            for r in &rows {
                md.push_str(r);
                md.push('\n');
            }
            md.push_str(&format!("\n**Overall: {}**\n", if all_ok { "PASS" } else { "FAIL" }));
            md.push_str(
                "\n## Scope\n\nThis validates the *arithmetic and precision machinery* at extreme \
                 bit-width (the depth-critical numerics), not a full rendered image: a per-pixel \
                 arbitrary-precision dwell oracle is computationally infeasible this deep, and the \
                 renderer's `f64` `units_per_pixel` caps live zoom near 1e308× regardless. \
                 Independent per-pixel cross-checks (`--selftest`, `--crosscheck-f3`) cover the \
                 renderable depth range.\n",
            );
            if let Err(e) = std::fs::write(&path, md) {
                eprintln!("report write failed: {e}");
            } else {
                println!("report → {}", path.display());
            }
        }
        println!("validate-deep: {}", if all_ok { "PASS" } else { "FAIL" });
        std::process::exit(if all_ok { 0 } else { 1 });
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

/// Format magnification from `log2(magnification)` — stays correct past `f64`'s 1e308×
/// (where `magnification()` saturates to `∞`), formatting `2^log2mag` via base-10.
fn fmt_zoom_log2(log2mag: f64) -> String {
    if log2mag <= 1020.0 {
        fmt_zoom(2f64.powf(log2mag.max(0.0)))
    } else {
        let log10 = log2mag * std::f64::consts::LOG10_2;
        let e = log10.floor();
        let m = 10f64.powf(log10 - e);
        format!("{m:.2}e{e:.0}")
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
    if !(m.is_finite() && m > 0.0) || !exp.is_finite() {
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
    // `__cpuid` is part of the x86_64 baseline, so it is callable in safe code.
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
    /// Auto-zoom autopilot: continuously dive toward the detail-richest region.
    autopilot: bool,
    /// Screen-fraction (0..1, 0..1) the autopilot is diving toward (re-evaluated periodically).
    autopilot_target: (f64, f64),
    /// App-time of the last autopilot target re-evaluation.
    autopilot_eval_t: f64,
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
    /// `--render-iter`: write the raw iteration texture as EXR instead of a colored image.
    render_iter_mode: bool,
    /// CLI `--selftest`: run the GPU validation suite, print a report, and exit.
    selftest: bool,
    selftest_done: bool,
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
    /// Navigation history (location undo/redo) + settle-edge tracking.
    nav_undo: Vec<ViewSnapshot>,
    nav_redo: Vec<ViewSnapshot>,
    nav_was_interacting: bool,
    /// Go-to-location dialog state.
    goto_open: bool,
    goto_x: String,
    goto_y: String,
    goto_zoom: String,
    goto_msg: Option<String>,
    /// Transient status toast (message, time set) — e.g. minibrot-finder result.
    toast: Option<(String, f64)>,
    /// Keyboard/help overlay window open, and the selected Help section index.
    help_open: bool,
    help_section: usize,
    /// Whether the right-hand control panel is shown.
    right_panel_open: bool,
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
    light_anim: bool,  // rotate the light direction over time
    /// Distance-estimate glow (contour bands near the boundary), + animation.
    de: bool,
    de_strength: f32,
    de_width: f32,
    de_anim: bool,
    de_phase: f32, // runtime (animated)
    /// Coloring method (0 smooth, 1 stripe, 2 triangle-ineq, 3 orbit trap,
    /// 4 distance, 5 decomposition) + its parameters.
    color_method: u32,
    stripe_freq: f32,
    trap_type: u32,
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
        apply_brand_theme(&cc.egui_ctx);
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
        let render_iter_mode = args.iter().any(|a| a == "--render-iter");
        let auto_render = args.iter().any(|a| a == "--render") || render_iter_mode;
        let selftest = args.iter().any(|a| a == "--selftest");
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
        viewport.units_per_pixel =
            fractadyne_core::FloatExp::new(s.units_per_pixel, s.units_per_pixel_e);
        viewport.precision =
            fractadyne_core::precision_for_octaves(viewport.log2_magnification().max(0.0).ceil() as u64);

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
            autopilot: false,
            autopilot_target: (0.5, 0.5),
            autopilot_eval_t: 0.0,
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
            render_iter_mode,
            selftest,
            selftest_done: false,
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
            nav_undo: Vec::new(),
            nav_redo: Vec::new(),
            nav_was_interacting: false,
            goto_open: false,
            goto_x: String::new(),
            goto_y: String::new(),
            goto_zoom: String::new(),
            goto_msg: None,
            toast: None,
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
            light_anim: s.light_anim,
            de: s.de,
            de_strength: s.de_strength,
            de_width: s.de_width,
            de_anim: s.de_anim,
            de_phase: 0.0,
            color_method: method_from_str(&s.color_method),
            stripe_freq: s.stripe_freq,
            trap_type: trap_from_str(&s.trap_type),
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
        app.nav_undo.push(app.snapshot_view()); // baseline for navigation undo
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
        // `--zoom` is f64 (≤ ~1e308×); `--zoom-log2 L` sets magnification = 2^L for
        // arbitrary depth past the f64 range (e.g. L=1100 ≈ 1e331×).
        if let Some(l) = val("--zoom-log2").and_then(|s| s.parse::<f64>().ok()) {
            self.viewport.set_center_log2mag(cx, cy, l);
        } else {
            let zoom = val("--zoom").and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0);
            self.viewport.set_center_mag(cx, cy, zoom.max(1.0e-300));
        }
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
        if let Some(m) = val("--method") {
            self.color_method = method_from_str(m);
        }
        if let Some(f) = val("--stripe-freq").and_then(|s| s.parse::<f32>().ok()) {
            self.stripe_freq = f.clamp(1.0, 24.0);
        }
        if let Some(t) = val("--trap") {
            self.trap_type = trap_from_str(t);
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
            units_per_pixel: self.viewport.units_per_pixel.m,
            units_per_pixel_e: self.viewport.units_per_pixel.e,
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
            light_anim: self.light_anim,
            de: self.de,
            de_strength: self.de_strength,
            de_width: self.de_width,
            de_anim: self.de_anim,
            color_method: method_to_str(self.color_method).to_string(),
            stripe_freq: self.stripe_freq,
            trap_type: trap_to_str(self.trap_type).to_string(),
            minimap: self.minimap,
            custom_palette: self.custom_palette.clone(),
            use_custom_palette: self.use_custom_palette,
            use_duotone: self.use_duotone,
            use_binary: self.use_binary,
            duotone_lo: self.duotone_lo,
            duotone_hi: self.duotone_hi,
            right_panel_open: self.right_panel_open,
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
        span: (fractadyne_core::FloatExp, fractadyne_core::FloatExp),
        eff_iter: u32,
        precision: usize,
        julia: bool,
        ref_override: Option<[fractadyne_core::BigFloat; 2]>,
    ) -> (
        std::sync::Arc<Vec<[f32; 4]>>,
        u32,
        [fractadyne_core::BigFloat; 2],
    ) {
        let formula = self.fractal.formula_id();
        let (jcx, jcy) = self.julia_c;
        // A correct perturbation render is invariant to which valid in-view reference is
        // used; `ref_override` lets the validator force a specific reference (Phase 1.2).
        let rp = ref_override.unwrap_or_else(|| {
            fractadyne_core::best_reference(
                center_bf,
                [span.0, span.1],
                formula,
                julia,
                [jcx, jcy],
                eff_iter,
                precision,
            )
        });
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
        let log2mag = vp.log2_magnification();
        let width = self.export_width.max(1);
        // height from aspect: span_y/span_x = height_px/width_px (the scale cancels).
        let height = ((width as f64) * vp.height_px / vp.width_px).round().max(1.0) as u32;
        let mag = vp.magnification(); // saturates to ∞ past 1e308×; fine for the mode compares
        let eff_iter = if self.auto_iter {
            vp.recommended_max_iter(self.max_iter)
        } else {
            self.max_iter
        }
        // Cap at the zoom-appropriate count (same as the live view): avoids noise from
        // over-resolving sub-pixel dust, and keeps the export fast/responsive.
        .min(zoom_iter_cap(log2mag).max(256));
        let mode: u32 = if !self.fractal.supports_perturbation() || mag < 1.0e4 {
            1
        } else if mag >= PERT_FE_THRESHOLD {
            2
        } else {
            0
        };
        let precision = vp.precision; // maintained by the viewport; valid at any depth
        let (cx, cy) = vp.center_f64();
        let scale = vp.gpu_scale();
        let delta_exp = scale.delta_exp;

        let mut ref_offset = [0.0_f32; 4];
        let mut orbit = std::sync::Arc::new(Vec::new());
        let mut orbit_len = 0u32;
        if mode != 1 {
            let center_bf = [vp.center_x.clone(), vp.center_y.clone()];
            let (orbit_arc, len, rp) =
                self.compute_reference(&center_bf, vp.complex_span_fe(), eff_iter, precision, julia, None);
            orbit = orbit_arc;
            orbit_len = len;
            let dx = fractadyne_core::ref_offset_mantissa(&vp.center_x, &rp[0], delta_exp, precision);
            let dy = fractadyne_core::ref_offset_mantissa(&vp.center_y, &rp[1], delta_exp, precision);
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
            span_mantissa: scale.span_mantissa,
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
            cycle: self.color_cycle(),
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
            color_method: self.color_method,
            stripe_freq: self.stripe_freq,
            trap_type: self.trap_type,
            aa_filter: 1,
            interior_col: self.interior_color(),
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
                fmt_zoom_log2(self.viewport.log2_magnification())
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
             center_x={}\ncenter_y={}\nupp={:.17e}\nupp_log2={:.17e}\nzoom={}\nmax_iter={}\nauto_iter={}\n\
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
            self.viewport.units_per_pixel.to_f64(),
            // Extended-range scale (log2 of units_per_pixel) so deep (>1e308×) views reload
            // exactly; `upp` above is the saturating f64 (back-compat + human-readable).
            self.viewport.units_per_pixel.log2(),
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
        // Prefer the extended-range `upp_log2` (exact past 1e308×); fall back to the f64
        // `upp` for images saved before it existed.
        if let Some(l) = get("upp_log2").and_then(|s| s.parse::<f64>().ok()).filter(|l| l.is_finite()) {
            self.viewport.units_per_pixel = fractadyne_core::FloatExp::from_f64(1.0).mul_pow2(l);
        } else if let Some(upp) = get("upp").and_then(|s| s.parse::<f64>().ok()) {
            self.viewport.units_per_pixel = fractadyne_core::FloatExp::from_f64(upp);
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
        self.viewport.precision = fractadyne_core::precision_for_octaves(
            self.viewport.log2_magnification().max(0.0).ceil() as u64,
        );
        self.invalidate_refs();
        self.zoom_vel = 0.0;
        self.record_nav();
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

    /// Render the **raw iteration texture** for the current view and write it as an EXR
    /// (`--render-iter`): four 32-bit float channels — R = smooth iteration (negative ⇒
    /// in-set/interior), G/B = slope normal (x, y), A = log₂(distance estimate in pixels).
    /// Lets a reviewer diff iteration data directly, removing coloring as a confound.
    /// Single-tile, clamped to the GPU's max texture dimension.
    fn render_iter_to_file(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        path: &std::path::Path,
    ) -> Result<String, String> {
        let req = self.current_export_request_for(&self.viewport, self.julia_mode);
        let r = fractadyne_gpu::render_iter(device, queue, &req)?;
        let meta = format!(
            "{}\n# iteration-data EXR: R=smooth_iter (<0 = interior), G=normal.x, \
             B=normal.y, A=log2(distance_estimate_px)",
            self.view_metadata()
        );
        fractadyne_export::write_exr(path, r.width, r.height, &r.pixels, Some(&meta))?;
        Ok(format!("Saved iteration EXR {}×{} → {}", r.width, r.height, path.display()))
    }

    /// GPU validation suite (`--selftest`): renders controlled views and cross-checks the
    /// render paths against each other and against invariants. Prints a report; returns
    /// true iff every check passed. This validates the *visual/render* pipeline; exact
    /// numeric ground truth lives in `fractadyne-core`'s unit tests.
    fn run_selftest(&mut self, device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue) -> bool {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        // Deterministic params: Mandelbrot, no Julia.
        self.fractal = FractalKind::Mandelbrot;
        self.julia_mode = false;
        // Seahorse Valley — detailed at every depth tested; coordinate precise enough.
        const SX: &str = "-0.743643887037151";
        const SY: &str = "0.131825904205330";
        const N: u32 = 220;

        // Read back the raw iteration texture (smooth_iter, normal.x, normal.y, DE) — far
        // more sensitive than comparing final colors.
        let render = |req: &fractadyne_gpu::ExportRequest| -> Option<Vec<f32>> {
            fractadyne_gpu::render_iter(device, queue, req).ok().map(|r| r.pixels)
        };
        // A square request at the seahorse, then caller overrides the mode.
        let make = |cx: &str, cy: &str, mag: f64| -> fractadyne_gpu::ExportRequest {
            let mut vp = Viewport::new(N as f64, N as f64);
            vp.center_x = fractadyne_core::parse_bf(cx).unwrap();
            vp.center_y = fractadyne_core::parse_bf(cy).unwrap();
            vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mag));
            vp.precision = fractadyne_core::precision_for_magnification(mag);
            let mut req = self.current_export_request_for(&vp, false);
            req.width = N;
            req.height = N;
            req.ss = 1;
            req
        };
        // Mean |Δ| of the smooth-iteration channel over pixels that escaped in both, plus
        // the fraction differing by > 2 iterations (tolerates rare perturbation glitches).
        let compare = |a: &[f32], b: &[f32]| -> (f64, f64) {
            let (mut sum, mut n, mut big) = (0.0f64, 0u64, 0u64);
            for i in 0..(a.len() / 4) {
                let (ra, rb) = (a[i * 4], b[i * 4]);
                if ra >= 0.0 && rb >= 0.0 {
                    let d = (ra - rb).abs() as f64;
                    sum += d;
                    n += 1;
                    if d > 2.0 {
                        big += 1;
                    }
                }
            }
            if n == 0 { (f64::INFINITY, 1.0) } else { (sum / n as f64, big as f64 / n as f64) }
        };
        // Only the dwell (smooth-iter) channel needs to be finite; DE/normal channels can
        // legitimately overflow to ±inf when a mode is pushed past its range.
        let finite = |px: &[f32]| px.iter().step_by(4).all(|v| v.is_finite());

        // Independent integer-escape (`n`) bignum oracle for one (center, mag) view vs the
        // GPU `px`, on a sparse grid (slow on purpose). Each sample is classified:
        //   • both interior (CPU None, GPU < 0), or both escaped with the same n (|Δsmooth|<0.5)
        //   • boundary — ±1 iteration, or within a band of max_iter (dwell ill-conditioned)
        //   • mismatch — n off by ≥2, or interior/escaped disagreement away from the boundary.
        // Returns (checked, agree, boundary, mismatch). `max` MUST equal the GPU's max_iter
        // and bailout 256² so the integer counts are directly comparable.
        let oracle = |cx_s: &str, cy_s: &str, mag: f64, max: u32, px: &[f32]| -> (u64, u64, u64, u64) {
            let prec = fractadyne_core::precision_for_magnification(mag);
            let cx = fractadyne_core::parse_bf(cx_s).unwrap();
            let cy = fractadyne_core::parse_bf(cy_s).unwrap();
            let step = (3.0 / mag) / N as f64;
            let half = N as f64 / 2.0;
            let nn = N as usize;
            let at = |ii: usize, jj: usize| px[(jj * nn + ii) * 4];
            let gstep = (N / 5).max(1) as usize; // ~5×5 sparse grid
            let (mut checked, mut agree, mut boundary, mut mism) = (0u64, 0u64, 0u64, 0u64);
            let mut j = 0usize;
            while j < nn {
                let mut i = 0usize;
                while i < nn {
                    let g = at(i, j);
                    // Boundary detection from the GPU texture itself: a sample whose 4-neighbors
                    // flip interior↔exterior or jump in dwell is in an ill-conditioned region,
                    // where a sub-ULP coordinate difference legitimately flips n — exclude it.
                    let mut steep = false;
                    for (di, dj) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
                        let (ni, nj) = (i as isize + di, j as isize + dj);
                        if ni >= 0 && nj >= 0 && (ni as usize) < nn && (nj as usize) < nn {
                            let gn = at(ni as usize, nj as usize);
                            let flip = (g < 0.0) != (gn < 0.0);
                            let jump = g >= 0.0 && gn >= 0.0 && (g - gn).abs() > 2.0;
                            if flip || jump {
                                steep = true;
                            }
                        }
                    }
                    checked += 1;
                    if steep {
                        boundary += 1;
                        i += gstep;
                        continue;
                    }
                    let cre = fractadyne_core::add_f64(&cx, ((i as f64 + 0.5) - half) * step, prec);
                    let cim = fractadyne_core::add_f64(&cy, (half - (j as f64 + 0.5)) * step, prec);
                    let cpu = fractadyne_core::naive_dwell_bf(&cre, &cim, max, 65536.0, prec);
                    // Smooth region: GPU and CPU must agree exactly (same n).
                    match (g >= 0.0, cpu) {
                        (false, None) => agree += 1,
                        (true, Some((_n, smooth))) if (g - smooth).abs() < 0.75 => agree += 1,
                        _ => mism += 1,
                    }
                    i += gstep;
                }
                j += gstep;
            }
            (checked, agree, boundary, mism)
        };

        let mut checks: Vec<SelfCheck> = Vec::new();

        // ---- numeric & render-path checks (local closures borrow self immutably) ----
        {
            // (A) df32 perturbation vs an independent CPU f64 dwell @2e4× (f64 exact here).
            let mag = 2.0e4;
            let req = make(SX, SY, mag);
            if let Some(px) = render(&req) {
                let cx0 = fractadyne_core::to_f64(&fractadyne_core::parse_bf(SX).unwrap());
                let cy0 = fractadyne_core::to_f64(&fractadyne_core::parse_bf(SY).unwrap());
                let step = (3.0 / mag) / N as f64;
                let half = N as f64 / 2.0;
                let (mut n, mut big) = (0u64, 0u64);
                let mut k = 0usize;
                while k < (N as usize) * (N as usize) {
                    let (i, j) = ((k % N as usize) as f64, (k / N as usize) as f64);
                    let g = px[k * 4];
                    if g >= 0.0 {
                        let cre = cx0 + ((i + 0.5) - half) * step;
                        let cim = cy0 + (half - (j + 0.5)) * step;
                        if let Some(cpu) = mandel_smooth_f64(cre, cim, req.max_iter) {
                            n += 1;
                            if (g - cpu).abs() > 1.0 {
                                big += 1;
                            }
                        }
                    }
                    k += 7;
                }
                let frac = if n == 0 { 1.0 } else { big as f64 / n as f64 };
                checks.push(SelfCheck {
                    category: "Numeric",
                    name: "df32 perturbation vs CPU f64 dwell".into(),
                    params: format!("seahorse, 2e4×, {} iter, n={n}", req.max_iter),
                    result: format!("{:.1}% agree within 1 iter", (1.0 - frac) * 100.0),
                    threshold: "≥90% within 1 iter",
                    pass: frac < 0.10,
                });
                checks.push(SelfCheck {
                    category: "Finiteness",
                    name: "dwell finite (perturbation @2e4×)".into(),
                    params: "all sampled pixels".into(),
                    result: if finite(&px) { "all finite".into() } else { "NON-FINITE!".into() },
                    threshold: "all finite",
                    pass: finite(&px),
                });
            }

            // (B) floatexp vs df32 perturbation @1e10× — two representations, must agree.
            let mut a = make(SX, SY, 1.0e10);
            a.mode = 0;
            let mut b = a.clone();
            b.mode = 2;
            if let (Some(aa), Some(bb)) = (render(&a), render(&b)) {
                let (mean, frac) = compare(&aa, &bb);
                checks.push(SelfCheck {
                    category: "Numeric",
                    name: "floatexp vs df32 perturbation".into(),
                    params: "seahorse, 1e10×".into(),
                    result: format!("mean Δ={mean:.4} iter, >2iter {:.3}%", frac * 100.0),
                    threshold: "mean<0.5, <2% differ",
                    pass: mean < 0.5 && frac < 0.02,
                });
            }

            // (C) Independent bignum oracle across a DEPTH BATTERY — integer escape n, exact
            // on every non-boundary sample, testing whichever render mode the depth selector
            // actually uses (df32 perturbation through ~1e24×, floatexp at ≥1e28×). This is
            // the only check that gives *independent* deep-zoom correctness (not internal
            // consistency). Full-precision deep coordinates use a 38-digit minibrot nucleus.
            const NX: &str = "-0.74364388703715887077806454349323251348";
            const NY: &str = "0.131825904205312292821097354874199108694";
            let battery: &[(&str, &str, &str, f64)] = &[
                ("1e6x", SX, SY, 1.0e6),
                ("1e12x", SX, SY, 1.0e12),
                ("1e16x", NX, NY, 1.0e16),
                ("1e24x", NX, NY, 1.0e24),
                ("1e30x", NX, NY, 1.0e30),
            ];
            for (label, cx, cy, mag) in battery {
                let req = make(cx, cy, *mag); // mode chosen by the real depth selector
                if let Some(px) = render(&req) {
                    let (checked, agree, boundary, mism) = oracle(cx, cy, *mag, req.max_iter, &px);
                    checks.push(SelfCheck {
                        category: "Bignum oracle",
                        name: format!("naive bignum dwell vs GPU @{label}"),
                        params: format!("mode {}, {} iter, {checked} samples", req.mode, req.max_iter),
                        result: format!("{agree} agree, {boundary} boundary, {mism} mismatch"),
                        threshold: "0 hard mismatches",
                        pass: mism == 0 && checked > 0,
                    });
                } else {
                    checks.push(SelfCheck {
                        category: "Bignum oracle",
                        name: format!("naive bignum dwell vs GPU @{label}"),
                        params: "render".into(),
                        result: "render failed".into(),
                        threshold: "0 hard mismatches",
                        pass: false,
                    });
                }
            }

            // (D2) Reference independence — a correct perturbation render is invariant to the
            // chosen valid reference. Render with 3 distinct in-view references (the auto
            // `best_reference` plus two offset points), take the per-pixel majority dwell as
            // oracle-free truth, and assert the *auto* reference dissents from consensus on a
            // tiny, localized fraction (dissenters are exactly the glitched pixels). The
            // offset references are deliberately allowed to be poorer — they just provide
            // independent votes.
            {
                let mag = 1.0e8;
                let base = make(SX, SY, mag); // mode 0, best_reference
                let prec = fractadyne_core::precision_for_magnification(mag);
                let cxb = fractadyne_core::parse_bf(SX).unwrap();
                let cyb = fractadyne_core::parse_bf(SY).unwrap();
                // Actual complex span (shallow here): span_mantissa × 2^delta_exp.
                let span = base.span_mantissa[0] * 2f64.powi(base.delta_exp);
                let span_fe = fractadyne_core::FloatExp::from_f64(span);
                let with_ref = |ox: f64, oy: f64| -> fractadyne_gpu::ExportRequest {
                    let ref_pt = [
                        fractadyne_core::add_f64(&cxb, ox, prec),
                        fractadyne_core::add_f64(&cyb, oy, prec),
                    ];
                    let (orbit, len, rp) = self.compute_reference(
                        &[cxb.clone(), cyb.clone()], (span_fe, span_fe), base.max_iter, prec, false, Some(ref_pt),
                    );
                    let dx = fractadyne_core::ref_offset_mantissa(&cxb, &rp[0], base.delta_exp, prec);
                    let dy = fractadyne_core::ref_offset_mantissa(&cyb, &rp[1], base.delta_exp, prec);
                    let (dxh, dyh) = (dx as f32, dy as f32);
                    let mut r = base.clone();
                    r.orbit = orbit;
                    r.orbit_len = len;
                    r.ref_offset = [dxh, dyh, (dx - dxh as f64) as f32, (dy - dyh as f64) as f32];
                    r
                };
                let altb = with_ref(0.25 * span, 0.20 * span);
                let altc = with_ref(-0.22 * span, -0.18 * span);
                if let (Some(pa), Some(pb), Some(pc)) =
                    (render(&base), render(&altb), render(&altc))
                {
                    let nn = N as usize;
                    let eq = |x: f32, y: f32| ((x < 0.0) == (y < 0.0)) && (x < 0.0 || (x - y).abs() < 0.5);
                    // Skip boundary pixels (dwell ill-conditioned there — a sub-ULP reference
                    // difference legitimately flips n); count only smooth-region disagreement.
                    let steep = |i: usize, j: usize| -> bool {
                        let g = pa[(j * nn + i) * 4];
                        for (di, dj) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
                            let (ni, nj) = (i as isize + di, j as isize + dj);
                            if ni >= 0 && nj >= 0 && (ni as usize) < nn && (nj as usize) < nn {
                                let gn = pa[(nj as usize * nn + ni as usize) * 4];
                                if (g < 0.0) != (gn < 0.0) || (g >= 0.0 && gn >= 0.0 && (g - gn).abs() > 2.0) {
                                    return true;
                                }
                            }
                        }
                        false
                    };
                    let (mut smooth, mut auto_dissent, mut no_majority) = (0u64, 0u64, 0u64);
                    for j in 0..nn {
                        for i in 0..nn {
                            if steep(i, j) {
                                continue;
                            }
                            let k = j * nn + i;
                            let (a, b, c) = (pa[k * 4], pb[k * 4], pc[k * 4]);
                            let (ab, ac, bc) = (eq(a, b), eq(a, c), eq(b, c));
                            smooth += 1;
                            if ab || ac {
                                // auto in the majority — clean
                            } else if bc {
                                auto_dissent += 1;
                            } else {
                                no_majority += 1;
                            }
                        }
                    }
                    let frac = (auto_dissent + no_majority) as f64 / smooth.max(1) as f64;
                    checks.push(SelfCheck {
                        category: "Glitch",
                        name: "reference independence (3-ref majority)".into(),
                        params: "seahorse, 1e8×, auto vs 2 offset refs (smooth region)".into(),
                        result: format!(
                            "{} smooth px: auto dissent {auto_dissent}, no-majority {no_majority} ({:.4}%)",
                            smooth, frac * 100.0
                        ),
                        threshold: "<0.2% of smooth pixels",
                        pass: frac < 0.002,
                    });
                }
            }

            // (E) Real-axis symmetry + interior/exterior presence + finiteness @home.
            let req = make("-0.5", "0.0", 1.0);
            if let Some(px) = render(&req) {
                let w = N as usize;
                let (mut sum, mut n) = (0.0f64, 0u64);
                for y in 0..(N as usize / 2) {
                    for x in 0..w {
                        let (t, bm) = (px[(y * w + x) * 4], px[((N as usize - 1 - y) * w + x) * 4]);
                        if t >= 0.0 && bm >= 0.0 {
                            sum += (t - bm).abs() as f64;
                            n += 1;
                        }
                    }
                }
                let mean = if n == 0 { f64::INFINITY } else { sum / n as f64 };
                checks.push(SelfCheck {
                    category: "Invariant",
                    name: "real-axis mirror symmetry".into(),
                    params: "home view (-0.5, 0)".into(),
                    result: format!("mean Δ={mean:.5} iter"),
                    threshold: "mean<0.05",
                    pass: mean < 0.05,
                });
                let interior = px.iter().step_by(4).any(|&r| r < 0.0);
                let exterior = px.iter().step_by(4).any(|&r| r >= 0.0);
                checks.push(SelfCheck {
                    category: "Invariant",
                    name: "home has interior + exterior".into(),
                    params: "home view".into(),
                    result: format!("interior={interior}, exterior={exterior}"),
                    threshold: "both present",
                    pass: interior && exterior,
                });
            }
        }

        // ---- render-pipeline symmetry for the non-Mandelbrot family shaders ----
        // The bignum oracle only validates the Mandelbrot shader; these exact symmetries
        // (verified in fractadyne-core) are the main correctness signal for the other
        // analytic-family shaders. Render an origin/real-axis-centered view and compare
        // each pixel to its symmetry partner, excluding ill-conditioned boundary pixels.
        {
            let nn = N as usize;
            let steep = |px: &[f32], i: usize, j: usize| -> bool {
                let g = px[(j * nn + i) * 4];
                for (di, dj) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
                    let (ni, nj) = (i as isize + di, j as isize + dj);
                    if ni >= 0 && nj >= 0 && (ni as usize) < nn && (nj as usize) < nn {
                        let gn = px[(nj as usize * nn + ni as usize) * 4];
                        if (g < 0.0) != (gn < 0.0) || (g >= 0.0 && gn >= 0.0 && (g - gn).abs() > 2.0) {
                            return true;
                        }
                    }
                }
                false
            };
            let cases: &[(FractalKind, &str, fn(usize, usize, usize) -> (usize, usize))] = &[
                (FractalKind::Multibrot3, "Multibrot-3 180° rotation", |i, j, n| (n - 1 - i, n - 1 - j)),
                (FractalKind::Tricorn, "Tricorn real-axis reflection", |i, j, n| (i, n - 1 - j)),
                (FractalKind::Celtic, "Celtic real-axis reflection", |i, j, n| (i, n - 1 - j)),
            ];
            for &(fractal, label, partner) in cases {
                self.fractal = fractal;
                self.julia_mode = false;
                self.color_method = 0;
                self.use_custom_palette = false;
                self.auto_iter = false;
                self.max_iter = 1500;
                let mut vp = Viewport::new(N as f64, N as f64);
                vp.center_x = fractadyne_core::BigFloat::from_f64(0.0, 64);
                vp.center_y = fractadyne_core::BigFloat::from_f64(0.0, 64);
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / N as f64); // span 3, origin-centered
                vp.precision = 64;
                let mut req = self.current_export_request_for(&vp, false);
                req.width = N;
                req.height = N;
                req.ss = 1;
                let px = fractadyne_gpu::render_iter(device, queue, &req).ok().map(|r| r.pixels);
                if let Some(px) = px {
                    let (mut total, mut bad) = (0u64, 0u64);
                    for j in 0..nn {
                        for i in 0..nn {
                            if steep(&px, i, j) {
                                continue;
                            }
                            let (pi, pj) = partner(i, j, nn);
                            let (a, b) = (px[(j * nn + i) * 4], px[(pj * nn + pi) * 4]);
                            let eq = (a < 0.0) == (b < 0.0) && (a < 0.0 || (a - b).abs() < 0.5);
                            total += 1;
                            if !eq {
                                bad += 1;
                            }
                        }
                    }
                    checks.push(SelfCheck {
                        category: "Symmetry (render)",
                        name: label.into(),
                        params: format!("origin view, span 3, {total} smooth px"),
                        result: format!("{bad} asymmetric"),
                        threshold: "0 asymmetric",
                        pass: bad == 0 && total > 0,
                    });
                }
            }
        }

        // ---- invariance & consistency (Phase 3) — oracle-free, targets the tier crossovers ----
        {
            self.fractal = FractalKind::Mandelbrot;
            self.julia_mode = false;
            self.color_method = 0;
            self.use_custom_palette = false;
            self.auto_iter = false;
            let cxb = fractadyne_core::parse_bf(SX).unwrap();
            let cyb = fractadyne_core::parse_bf(SY).unwrap();
            // Build a square Mandelbrot iteration render at an explicit center/zoom/size.
            let build = |cx: &fractadyne_core::BigFloat, cy: &fractadyne_core::BigFloat,
                         mag: f64, size: u32, max: u32|
             -> Option<Vec<f32>> {
                let mut vp = Viewport::new(size as f64, size as f64);
                vp.center_x = cx.clone();
                vp.center_y = cy.clone();
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (size as f64 * mag));
                vp.precision = fractadyne_core::precision_for_magnification(mag);
                let mut req = self.current_export_request_for(&vp, false);
                req.width = size;
                req.height = size;
                req.ss = 1;
                req.max_iter = max;
                fractadyne_gpu::render_iter(device, queue, &req).ok().map(|r| r.pixels)
            };
            let steep = |px: &[f32], sz: usize, i: usize, j: usize| -> bool {
                let g = px[(j * sz + i) * 4];
                for (di, dj) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
                    let (ni, nj) = (i as isize + di, j as isize + dj);
                    if ni >= 0 && nj >= 0 && (ni as usize) < sz && (nj as usize) < sz {
                        let gn = px[(nj as usize * sz + ni as usize) * 4];
                        if (g < 0.0) != (gn < 0.0) || (g >= 0.0 && gn >= 0.0 && (g - gn).abs() > 2.0) {
                            return true;
                        }
                    }
                }
                false
            };
            let agree = |a: f32, b: f32| (a < 0.0) == (b < 0.0) && (a < 0.0 || (a - b).abs() < 0.5);
            let nn = N as usize;

            // 3.1 Resolution independence: an N×N pixel (i,j) shares its exact complex
            // coordinate with the 3N×3N pixel (3i+1, 3j+1); their dwell must match.
            if let (Some(p1), Some(p3)) =
                (build(&cxb, &cyb, 1.0e6, N, 2000), build(&cxb, &cyb, 1.0e6, N * 3, 2000))
            {
                let n3 = nn * 3;
                let (mut checked, mut bad) = (0u64, 0u64);
                for j in 0..nn {
                    for i in 0..nn {
                        if steep(&p1, nn, i, j) {
                            continue;
                        }
                        let (i3, j3) = (3 * i + 1, 3 * j + 1);
                        checked += 1;
                        if !agree(p1[(j * nn + i) * 4], p3[(j3 * n3 + i3) * 4]) {
                            bad += 1;
                        }
                    }
                }
                checks.push(SelfCheck {
                    category: "Consistency",
                    name: "resolution independence (N vs 3N)".into(),
                    params: format!("seahorse, 1e6×, {checked} smooth px"),
                    result: format!("{bad} differ"),
                    threshold: "0 differ",
                    pass: bad == 0 && checked > 0,
                });
            }

            // 3.2 Max-iter monotonic stability: a pixel already escaped at a low max_iter keeps
            // its dwell at a higher max_iter (raising the cap only escapes more interior pixels).
            if let (Some(pa), Some(pb)) =
                (build(&cxb, &cyb, 1.0e6, N, 500), build(&cxb, &cyb, 1.0e6, N, 3000))
            {
                let (mut checked, mut bad) = (0u64, 0u64);
                for k in 0..(nn * nn) {
                    let a = pa[k * 4];
                    if a >= 0.0 {
                        checked += 1;
                        if !agree(a, pb[k * 4]) {
                            bad += 1;
                        }
                    }
                }
                checks.push(SelfCheck {
                    category: "Consistency",
                    name: "max-iter monotonic stability".into(),
                    params: format!("seahorse, 1e6×, 500→3000 iter, {checked} escaped px"),
                    result: format!("{bad} changed dwell"),
                    threshold: "0 changed",
                    pass: bad == 0 && checked > 0,
                });
            }

            // 3.3 Zoom-sequence consistency ACROSS THE direct→perturbation crossover: a view at
            // 4e3× (direct) and at 1.2e4× (perturbation, 3× deeper) must agree where they
            // overlap — shallower pixel k ↔ deeper pixel (3k+1−N). Strongest test of the seam.
            if let (Some(ps), Some(pd)) =
                (build(&cxb, &cyb, 4.0e3, N, 3000), build(&cxb, &cyb, 1.2e4, N, 3000))
            {
                let (mut checked, mut bad) = (0u64, 0u64);
                for j in 0..nn {
                    for k in 0..nn {
                        let id = 3 * k as isize + 1 - nn as isize;
                        let jd = 3 * j as isize + 1 - nn as isize;
                        if id < 0 || jd < 0 || id as usize >= nn || jd as usize >= nn {
                            continue;
                        }
                        if steep(&ps, nn, k, j) {
                            continue;
                        }
                        checked += 1;
                        if !agree(ps[(j * nn + k) * 4], pd[(jd as usize * nn + id as usize) * 4]) {
                            bad += 1;
                        }
                    }
                }
                // Reference differs between the two zooms, so an isolated boundary pixel may
                // flip by 1 iteration; a true seam bug would differ over a large region.
                checks.push(SelfCheck {
                    category: "Consistency",
                    name: "zoom-sequence across direct→df32 seam".into(),
                    params: format!("seahorse, 4e3×↔1.2e4×, {checked} overlap px"),
                    result: format!("{bad} differ"),
                    threshold: "<0.1% differ",
                    pass: checked > 0 && (bad as f64) < 0.001 * checked as f64,
                });
            }

            // 3.4 Pan consistency: shift the center by an integer pixel count; the overlapping
            // region must be identical — A(i,j) == B(i−shift, j).
            let shift = (N / 4) as usize;
            let stepx = (3.0 / 1.0e6) / N as f64;
            let cxb2 = fractadyne_core::add_f64(&cxb, shift as f64 * stepx, fractadyne_core::precision_for_magnification(1.0e6));
            if let (Some(pa), Some(pb)) =
                (build(&cxb, &cyb, 1.0e6, N, 2000), build(&cxb2, &cyb, 1.0e6, N, 2000))
            {
                let (mut checked, mut bad) = (0u64, 0u64);
                for j in 0..nn {
                    for i in shift..nn {
                        if steep(&pa, nn, i, j) {
                            continue;
                        }
                        checked += 1;
                        if !agree(pa[(j * nn + i) * 4], pb[(j * nn + (i - shift)) * 4]) {
                            bad += 1;
                        }
                    }
                }
                // Reference recomputed for the shifted center, so an isolated boundary pixel
                // may flip by 1; a true offset bug would shift the whole overlap.
                checks.push(SelfCheck {
                    category: "Consistency",
                    name: "pan consistency".into(),
                    params: format!("seahorse, 1e6×, +{shift}px, {checked} overlap px"),
                    result: format!("{bad} differ"),
                    threshold: "<0.1% differ",
                    pass: checked > 0 && (bad as f64) < 0.001 * checked as f64,
                });
            }

            // 3.5 Determinism: the same request rendered twice must be bit-identical.
            if let (Some(p1), Some(p2)) =
                (build(&cxb, &cyb, 1.0e6, N, 2000), build(&cxb, &cyb, 1.0e6, N, 2000))
            {
                let identical = p1.len() == p2.len()
                    && p1.iter().zip(&p2).all(|(a, b)| a.to_bits() == b.to_bits());
                checks.push(SelfCheck {
                    category: "Consistency",
                    name: "render determinism (2 runs)".into(),
                    params: "seahorse, 1e6×".into(),
                    result: if identical { "bit-identical".into() } else { "NON-DETERMINISTIC".into() },
                    threshold: "bit-identical",
                    pass: identical,
                });
            }

            // ---- Phase 4: derivative-dependent checks (DE / dz/dc) ----
            // The distance estimate (alpha channel = log2 DE-in-pixels) and slope normal are
            // derived from the floatexp dz/dc; validate them independently of the dwell.
            if let Some(px) = build(&cxb, &cyb, 1.0e6, N, 4000) {
                let de_px = |k: usize| -> Option<f32> {
                    let a = px[k * 4 + 3];
                    (a < 20.0).then(|| 2.0_f32.powf(a)) // >=20 ⇒ "far/unavailable"
                };

                // 4.2 DE self-consistency: an exterior pixel touching the interior (boundary
                // ≤1px away) cannot have a large distance estimate — that's a direct
                // contradiction exposing a derivative-formula error.
                let (mut bnd, mut viol) = (0u64, 0u64);
                for j in 1..nn - 1 {
                    for i in 1..nn - 1 {
                        let k = j * nn + i;
                        if px[k * 4] < 0.0 {
                            continue; // interior
                        }
                        let touches_interior = [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)]
                            .iter()
                            .any(|&(di, dj)| {
                                px[(((j as isize + dj) as usize) * nn + (i as isize + di) as usize) * 4] < 0.0
                            });
                        if touches_interior {
                            bnd += 1;
                            // boundary-adjacent ⇒ DE must be small (≤ a few px), and available.
                            if de_px(k).map(|d| d > 16.0).unwrap_or(true) {
                                viol += 1;
                            }
                        }
                    }
                }
                checks.push(SelfCheck {
                    category: "Derivative",
                    name: "distance-estimate self-consistency".into(),
                    params: format!("seahorse, 1e6×, {bnd} boundary px"),
                    result: format!("{viol} with DE>16px at boundary"),
                    threshold: "<0.5% of boundary px",
                    pass: bnd > 0 && (viol as f64) < 0.005 * bnd as f64,
                });

                // 4.1 DE lower bound (Koebe ¼ theorem): a disk of radius DE/4 about an exterior
                // point contains no boundary. Verify with an INDEPENDENT CPU dwell at the disk
                // rim — catches dz/dc under-estimation (DE too large) invisible to dwell tests.
                let step = (3.0 / 1.0e6) / N as f64;
                let cx0 = fractadyne_core::to_f64(&cxb);
                let cy0 = fractadyne_core::to_f64(&cyb);
                let half = N as f64 / 2.0;
                let (mut checked, mut koebe_viol) = (0u64, 0u64);
                let g = (N / 12).max(1) as usize;
                let mut j = 0usize;
                while j < nn {
                    let mut i = 0usize;
                    while i < nn {
                        let k = j * nn + i;
                        if px[k * 4] >= 0.0 {
                            if let Some(d) = de_px(k) {
                                if (1.0..=4096.0).contains(&d) {
                                    let r = d as f64 * step * 0.25; // Koebe-safe radius (world)
                                    let cre = cx0 + ((i as f64 + 0.5) - half) * step;
                                    let cim = cy0 + (half - (j as f64 + 0.5)) * step;
                                    checked += 1;
                                    for (ox, oy) in [(r, 0.0), (-r, 0.0), (0.0, r), (0.0, -r)] {
                                        if mandel_escapes(cre + ox, cim + oy, 4000).is_none() {
                                            koebe_viol += 1;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        i += g;
                    }
                    j += g;
                }
                checks.push(SelfCheck {
                    category: "Derivative",
                    name: "DE lower bound (Koebe ¼)".into(),
                    params: format!("seahorse, 1e6×, {checked} sampled exterior px"),
                    result: format!("{koebe_viol} disks contain interior"),
                    threshold: "0",
                    pass: checked > 0 && koebe_viol == 0,
                });
            }
        }

        // ---- catalog: independently verifiable locations (Phase 6.1 / 6.6) ----
        // Loads validation/catalog.toml (committed, human-readable) and checks the build
        // against each known answer, so external validation is one command.
        if let Ok(text) = std::fs::read_to_string("validation/catalog.toml") {
            match toml::from_str::<Catalog>(&text) {
                Ok(cat) => {
                    for e in &cat.nucleus {
                        let formula = e.fractal.as_deref()
                            .and_then(FractalKind::from_name)
                            .map_or(0, |k| k.formula_id());
                        let (Some(sx), Some(sy)) =
                            (fractadyne_core::parse_bf(&e.center_x), fractadyne_core::parse_bf(&e.center_y))
                        else {
                            continue;
                        };
                        match fractadyne_core::find_nucleus(&[sx, sy], e.zoom, formula, 100_000) {
                            Some(n) => {
                                let mut pass = n.period == e.period;
                                let mut detail = format!("period {} (want {})", n.period, e.period);
                                if let (Some(nx), Some(ny)) = (&e.nucleus_x, &e.nucleus_y) {
                                    if let (Some(ex), Some(ey)) =
                                        (fractadyne_core::parse_bf(nx), fractadyne_core::parse_bf(ny))
                                    {
                                        let prec = fractadyne_core::precision_for_magnification(e.zoom * 1.0e3);
                                        let dx = fractadyne_core::sub_f64(&n.cx, &ex, prec);
                                        let dy = fractadyne_core::sub_f64(&n.cy, &ey, prec);
                                        let dist = (dx * dx + dy * dy).sqrt();
                                        let tol = (1.0e-10 / e.zoom).max(1.0e-25);
                                        pass = pass && dist < tol;
                                        detail = format!("{detail}, nucleus Δ={dist:.1e}");
                                    }
                                }
                                checks.push(SelfCheck {
                                    category: "Catalog",
                                    name: e.name.clone(),
                                    params: format!("zoom {:.0e}", e.zoom),
                                    result: detail,
                                    threshold: "period + nucleus",
                                    pass,
                                });
                            }
                            None => checks.push(SelfCheck {
                                category: "Catalog",
                                name: e.name.clone(),
                                params: "find_nucleus".into(),
                                result: "no nucleus found".into(),
                                threshold: "period + nucleus",
                                pass: false,
                            }),
                        }
                    }
                    // Membership: the independent arbitrary-precision oracle decides whether the
                    // (full-precision) point is interior, and must match the catalog's known
                    // answer. (GPU-vs-oracle agreement over full views is covered by the oracle
                    // battery; a 1×1 render at the exact δc=0 center is an unrepresentative edge
                    // case.) Precision is generous so deep points aren't truncated.
                    let prec = fractadyne_core::precision_for_magnification(1.0e40);
                    for e in &cat.membership {
                        let (Some(cx), Some(cy)) =
                            (fractadyne_core::parse_bf(&e.center_x), fractadyne_core::parse_bf(&e.center_y))
                        else {
                            continue;
                        };
                        let oracle_interior =
                            fractadyne_core::naive_dwell_bf(&cx, &cy, 200_000, 65536.0, prec).is_none();
                        checks.push(SelfCheck {
                            category: "Catalog",
                            name: e.name.clone(),
                            params: format!("interior expected {}", e.interior),
                            result: format!("oracle says interior={oracle_interior}"),
                            threshold: "matches catalog",
                            pass: oracle_interior == e.interior,
                        });
                    }
                }
                Err(err) => checks.push(SelfCheck {
                    category: "Catalog",
                    name: "parse validation/catalog.toml".into(),
                    params: "TOML".into(),
                    result: format!("parse error: {err}"),
                    threshold: "valid",
                    pass: false,
                }),
            }
        }

        // ---- golden-image regression (set deterministic coloring per spec) ----
        let bless = std::env::args().any(|a| a == "--bless");
        let report_path = std::env::args()
            .position(|a| a == "--out" || a == "-o")
            .and_then(|i| std::env::args().nth(i + 1))
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("validation/report.md"));
        let out_base = report_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let golden_dir = out_base.join("golden");
        let current_dir = out_base.join("current");
        let _ = std::fs::create_dir_all(&golden_dir);
        if !bless {
            let _ = std::fs::create_dir_all(&current_dir);
        }
        // (name, cx, cy, zoom, iter, method, palette)
        let specs: &[(&str, &str, &str, f64, u32, u32, usize)] = &[
            ("home", "-0.5", "0.0", 1.0, 800, 0, 0),
            ("seahorse", SX, SY, 2.0e3, 1500, 0, 1),
            ("seahorse-stripe-1e6", SX, SY, 1.0e6, 4000, 1, 1),
            ("elephant", "0.2925755", "-0.0149977", 1.5e3, 1500, 0, 2),
        ];
        let (gw, gh) = (320u32, 240u32);
        // (name, max Δ, mean Δ, checksum, pass, reproduce)
        let mut goldens: Vec<(String, u32, f64, u64, bool, String)> = Vec::new();
        for &(name, cx, cy, zoom, iter, method, palette) in specs {
            self.fractal = FractalKind::Mandelbrot;
            self.julia_mode = false;
            self.color_method = method;
            self.palette_idx = palette;
            self.use_custom_palette = false;
            self.use_duotone = false;
            self.use_binary = false;
            self.cycle = 0.27;
            self.offset = 0.1;
            self.stripe_freq = 6.0;
            self.light = false;
            self.de = false;
            self.auto_iter = false;
            self.max_iter = iter;
            let mut vp = Viewport::new(gw as f64, gh as f64);
            vp.center_x = fractadyne_core::parse_bf(cx).unwrap();
            vp.center_y = fractadyne_core::parse_bf(cy).unwrap();
            vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (gh as f64 * zoom));
            vp.precision = fractadyne_core::precision_for_magnification(zoom);
            let mut req = self.current_export_request_for(&vp, false);
            req.width = gw;
            req.height = gh;
            req.ss = 1;
            let reproduce = format!(
                "fractadyne --render --out {name}.png --fractal Mandelbrot --center {cx} {cy} \
                 --zoom {zoom} --size {gw} --iter {iter} --ss 1 --method {} --palette {palette}",
                method_to_str(method)
            );
            let progress = std::sync::atomic::AtomicU32::new(0);
            let cancel = std::sync::atomic::AtomicBool::new(false);
            match fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel) {
                Ok(r) => {
                    let srgb = fractadyne_export::to_srgb8(&r.pixels);
                    let sum = fnv1a64(&srgb);
                    let png_path = golden_dir.join(format!("{name}.png"));
                    if bless {
                        let _ = fractadyne_export::write_png(&png_path, r.width, r.height, &r.pixels, Some(&reproduce));
                        goldens.push((name.to_string(), 0, 0.0, sum, true, reproduce));
                    } else {
                        let cur_path = current_dir.join(format!("{name}.png"));
                        let _ = fractadyne_export::write_png(&cur_path, r.width, r.height, &r.pixels, Some(&reproduce));
                        match fractadyne_export::read_png_rgba8(&png_path) {
                            Some((w, h, gpx)) if w == r.width && h == r.height => {
                                let (max, mean) = img_diff(&srgb, &gpx);
                                goldens.push((name.to_string(), max, mean, sum, max <= 10 && mean <= 2.0, reproduce));
                            }
                            _ => goldens.push((name.to_string(), 255, 255.0, sum, false, format!("{reproduce}  [no golden — run --bless]"))),
                        }
                    }
                }
                Err(e) => goldens.push((name.to_string(), 255, 255.0, 0, false, format!("render failed: {e}"))),
            }
        }

        // ---- build the human-readable + verifiable report ----
        let sys = gather_system_info();
        let checks_pass = checks.iter().filter(|c| c.pass).count();
        let gold_pass = goldens.iter().filter(|g| g.4).count();
        let ok = checks_pass == checks.len() && (bless || gold_pass == goldens.len());

        let mut md = String::new();
        md.push_str("# Fractadyne validation report\n\n");
        md.push_str(&format!("- **Version:** {}\n", version_string()));
        md.push_str(&format!("- **Generated:** {} (unix {ts})\n", utc_string(ts)));
        md.push_str(&format!("- **GPU:** {}\n", self.gpu_name));
        md.push_str(&format!(
            "- **CPU:** {} ({} cores / {} threads, L2 {} KB, L3 {} KB)\n",
            sys.cpu, sys.physical, sys.logical, sys.l2_kb, sys.l3_kb
        ));
        md.push_str(&format!("- **OS:** {} / {}\n", std::env::consts::OS, std::env::consts::ARCH));
        md.push_str(&format!("- **Mode:** {}\n\n", if bless { "BLESS (recording references)" } else { "VALIDATE" }));
        md.push_str(
            "All checks use exact mathematics (arbitrary-precision dwell, closed-form \
             properties) or internal cross-checks — no external data. Anyone can reproduce \
             a golden image with the listed command and compare it to `golden/`.\n\n",
        );
        md.push_str("## Numeric, deep-zoom & invariant checks\n\n");
        md.push_str("| Category | Check | Parameters | Result | Threshold | Verdict |\n");
        md.push_str("|---|---|---|---|---|---|\n");
        for c in &checks {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                c.category, c.name, c.params, c.result, c.threshold,
                if c.pass { "✅ PASS" } else { "❌ FAIL" }
            ));
        }
        md.push_str(&format!("\n**{checks_pass}/{} checks passed.**\n\n", checks.len()));
        // 6.5 Documented oracle scope — state plainly what is independently checked, and
        // where it is *not*, so a reviewer knows exactly where to aim scrutiny.
        md.push_str(
            "## Coverage & scope\n\n\
             What each oracle independently verifies, and its validity range:\n\n\
             - **Naive bignum dwell** (arbitrary precision, no perturbation/reference): exact \
             integer escape count at **any depth** — the only fully independent deep-zoom \
             oracle. Tested 1e6×–1e30× across the real render modes (df32 + floatexp).\n\
             - **CPU f64 dwell**: exact only to ~f64 coordinate resolution (≲1e13×); used for \
             the shallow cross-check.\n\
             - **floatexp ↔ df32 agreement**: internal consistency in the overlap band; not an \
             external oracle by itself.\n\
             - **Reference independence**: oracle-free glitch detection (multi-reference \
             majority); confirms the chosen reference is clean, doesn't prove a coordinate.\n\
             - **Symmetries / landmarks / consistency / derivative checks**: exact mathematics, \
             any depth, but each only constrains the property it tests.\n\
             - **Catalog**: full-precision locations with externally known answers (period, \
             nucleus, membership) — reproduce independently from `validation/catalog.toml`.\n\n\
             **Not independently oracle-checked:** non-Mandelbrot family *dwell* at depth \
             (only their symmetry is checked); interior-coloring/decomposition exactness; \
             coloring beyond the integer dwell. Aim scrutiny there.\n\n",
        );
        md.push_str(&format!("## Golden images ({gw}×{gh})\n\n"));
        md.push_str(&format!(
            "Stored in `{}`. {} pixel tolerance: max ≤ 10, mean ≤ 2.0 (8-bit sRGB).\n\n",
            golden_dir.display(),
            if bless { "Recorded this run." } else { "Compared against; current renders written to `current/` for side-by-side review." }
        ));
        md.push_str("| Image | Max Δ | Mean Δ | Checksum (FNV-1a) | Verdict | Reproduce |\n");
        md.push_str("|---|---|---|---|---|---|\n");
        for g in &goldens {
            md.push_str(&format!(
                "| {} | {} | {:.3} | `{:016x}` | {} | `{}` |\n",
                g.0, g.1, g.2, g.3,
                if bless { "📷 recorded" } else if g.4 { "✅ match" } else { "❌ differ" },
                g.5
            ));
        }
        md.push_str(&format!(
            "\n**{}/{} golden images {}.**\n\n## Summary\n\n{}\n",
            gold_pass, goldens.len(),
            if bless { "recorded" } else { "within tolerance" },
            if ok { "✅ ALL CHECKS PASSED" } else { "❌ FAILURES PRESENT — see table above" }
        ));

        if let Err(e) = std::fs::write(&report_path, &md) {
            eprintln!("Failed to write report to {}: {e}", report_path.display());
        }

        // ---- concise stdout summary ----
        println!("\nFractadyne self-test — {}\n{}", if bless { "BLESS" } else { "VALIDATE" }, "=".repeat(48));
        for c in &checks {
            println!("  [{}] {} — {}", if c.pass { "PASS" } else { "FAIL" }, c.name, c.result);
        }
        for g in &goldens {
            println!(
                "  [{}] golden {} — maxΔ {} meanΔ {:.2}",
                if bless { "REC " } else if g.4 { "PASS" } else { "FAIL" }, g.0, g.1, g.2
            );
        }
        println!("{}", "=".repeat(48));
        println!("checks {checks_pass}/{}, goldens {gold_pass}/{} — {}", checks.len(), goldens.len(),
            if ok { "OK" } else { "FAILURES PRESENT" });
        println!("report → {}\n", report_path.display());
        ok
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
        span: (fractadyne_core::FloatExp, fractadyne_core::FloatExp),
        magnification: f64,
        log2mag: f64,
        fractal: FractalKind,
        julia: bool,
        eff_iter: u32,
        interacting: bool,
        resolution: [u32; 2],
        view_id: u32,
    ) -> MandelbrotParams {
        let (stops, stop_count) = self.active_stops();
        let (cx, cy) = center;
        // Extended-range scale → shared base-2 exponent + O(1) span mantissas, so nothing
        // underflows/overflows past ~1e308× (the per-pixel δ stays O(1); the GPU re-applies
        // the exponent). `span.0`/`span.1` are FloatExp, valid at any depth.
        let delta_exp = if span.0.m == 0.0 { 0 } else { span.0.log2().floor() as i32 };
        let sm = -(delta_exp as f64);
        let span_mantissa = [span.0.mul_pow2(sm).to_f64(), span.1.mul_pow2(sm).to_f64()];

        // Bound per-frame GPU work so a single render can't trip the OS GPU watchdog
        // (TDR ≈ 2 s → device-lost crash). Work ≈ texels × iterations = px·ss²·iter.
        //
        // The key balance: a huge iteration count at deep zoom on a large window resolves
        // the boundary's sub-pixel "dust" into per-pixel noise once it starves the budget
        // of resolution/anti-aliasing. So cap the live iteration count at what's affordable
        // at *native* resolution — but never below a zoom-appropriate floor, so deep
        // interiors stay resolved (clamping iterations with no floor was the old
        // uniform-screen bug). Only when even that floor can't fit (extreme depth on a very
        // large window) do we fall back to reducing the iteration-texture resolution.
        let px = (resolution[0].max(1) as u64) * (resolution[1].max(1) as u64);
        // Iterations appropriate for this zoom. A high manual base (e.g. 50,000) would
        // over-resolve the boundary's sub-pixel "dust" into per-pixel noise *and* eat the
        // whole budget (forcing low resolution + no anti-aliasing). Cap the live preview at
        // a zoom-scaled count — generous enough that normal auto-iteration is never capped,
        // but an inflated manual base is. Exports still use the full count. The cap stays
        // well above what the zoom needs, so deep interiors remain resolved (no uniform
        // screen).
        let gpu_iter = eff_iter.min(50_000).min(zoom_iter_cap(log2mag).max(256));
        // GPU-watchdog safety (TDR ≈ 2 s): if even the capped work won't fit, reduce the
        // iteration-texture resolution (the color pass box-filters the upscale). Rare now
        // that iterations are zoom-capped.
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
        // Color-pass anti-aliasing when true supersampling wasn't affordable: widen the box
        // to match an upscaled (resolution-reduced) texture, or apply a gentle 2× box when
        // the budget forced ss=1 on a settled view the user wanted anti-aliased.
        let aa_filter = if res_scale < 1.0 {
            ((1.0 / res_scale).round() as u32).clamp(2, 4)
        } else if ss == 1 && self.aa > 1 && !interacting {
            2
        } else {
            1
        };

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
        let precision = fractadyne_core::precision_for_octaves(log2mag.max(0.0).ceil() as u64);
        let vi = view_id as usize;

        let mut ref_offset = [0.0_f32; 4];
        if mode != 1 {
            // Drift = |center − reference| / span, both as 2^-delta_exp mantissas so the
            // ratio is exact at any depth (raw f64 differences underflow past ~1e308×).
            let drift = self.ref_cache[vi].ref_pt.as_ref().map(|r| {
                let dx = fractadyne_core::ref_offset_mantissa(&center_bf[0], &r[0], delta_exp, precision)
                    / span_mantissa[0];
                let dy = fractadyne_core::ref_offset_mantissa(&center_bf[1], &r[1], delta_exp, precision)
                    / span_mantissa[1];
                (dx.abs(), dy.abs())
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
                    self.compute_reference(&center_bf, span, eff_iter, precision, julia, None);
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
            let dx = fractadyne_core::ref_offset_mantissa(&center_bf[0], &rp[0], delta_exp, precision);
            let dy = fractadyne_core::ref_offset_mantissa(&center_bf[1], &rp[1], delta_exp, precision);
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
            self.perf.last_eff_iter = gpu_iter; // iterations actually rendered this frame
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
            span_mantissa,
            max_iter: gpu_iter,
            cycle: self.color_cycle(),
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
            color_method: self.color_method,
            stripe_freq: self.stripe_freq,
            trap_type: self.trap_type,
            aa_filter,
            interior_col: self.interior_color(),
            resolution,
            ss,
            view_id,
        }
    }

    /// Palette-cycle scaling for the GPU. The bounded statistical methods (stripe /
    /// triangle-inequality / decomposition) produce a 0..1 value, so they want a few
    /// cycles across the palette; the unbounded ones (iteration / trap / distance) use
    /// the fine per-unit scaling.
    fn color_cycle(&self) -> f32 {
        if matches!(self.color_method, 1 | 2 | 5) {
            0.5 + self.cycle * 4.0
        } else {
            0.004 + self.cycle * 0.06
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
        let span = vp.complex_span_fe();
        let mag = vp.magnification();
        let log2mag = vp.log2_magnification();
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
            log2mag,
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
        ui.monospace(format!("aa         {}x", self.aa));
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
        self.zoom_vel = 0.0;
        self.invalidate_refs();
    }

    /// Record the current location onto the undo history (deduped vs. the top), and
    /// clear the redo stack. Called when the view settles and after discrete jumps.
    fn record_nav(&mut self) {
        let snap = self.snapshot_view();
        let dup = self.nav_undo.last().is_some_and(|t| {
            t.upp == snap.upp && t.cx == snap.cx && t.cy == snap.cy
        });
        if !dup {
            self.nav_undo.push(snap);
            if self.nav_undo.len() > 256 {
                self.nav_undo.remove(0);
            }
            self.nav_redo.clear();
        }
    }

    /// Step back / forward through visited locations.
    fn undo_view(&mut self) {
        if self.nav_undo.len() < 2 {
            return;
        }
        let cur = self.nav_undo.pop().unwrap();
        self.nav_redo.push(cur);
        let prev = self.nav_undo.last().unwrap().clone();
        self.apply_snapshot(&prev);
        self.nav_was_interacting = false;
    }
    fn redo_view(&mut self) {
        if let Some(s) = self.nav_redo.pop() {
            self.apply_snapshot(&s);
            self.nav_undo.push(s);
            self.nav_was_interacting = false;
        }
    }

    /// Open the go-to-location dialog, pre-filled with the current view.
    fn open_goto(&mut self) {
        self.goto_x = fractadyne_core::to_decimal_string(&self.viewport.center_x);
        self.goto_y = fractadyne_core::to_decimal_string(&self.viewport.center_y);
        self.goto_zoom = fmt_zoom_field(self.viewport.log2_magnification());
        self.goto_msg = None;
        self.goto_open = true;
    }

    /// Apply the go-to-location dialog: parse + validate, then jump (recording history).
    fn apply_goto(&mut self) {
        let cx = fractadyne_core::parse_bf(self.goto_x.trim());
        let cy = fractadyne_core::parse_bf(self.goto_y.trim());
        let log2mag = parse_zoom_to_log2(&self.goto_zoom);
        match (cx, cy, log2mag) {
            (Some(cx), Some(cy), Some(l)) if l.is_finite() => {
                // Clamp to a sane octave bound so a pasted absurd zoom can't request
                // runaway precision; well beyond any practical depth.
                self.viewport.set_center_log2mag(cx, cy, l.clamp(0.0, 1.0e6));
                self.zoom_vel = 0.0;
                self.invalidate_refs();
                self.record_nav();
                self.goto_msg = None;
                self.goto_open = false;
            }
            _ => {
                self.goto_msg = Some("Invalid input — check the coordinates and zoom.".into());
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
            self.viewport.recommended_max_iter(self.max_iter).clamp(1_000, 100_000);
        match fractadyne_core::find_nucleus(&center, mag, self.fractal.formula_id(), max_period) {
            Some(n) => {
                self.viewport.set_center_mag(n.cx, n.cy, mag);
                self.viewport.precision =
                    fractadyne_core::precision_for_magnification(mag);
                self.zoom_vel = 0.0;
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
        if !self.minimap || self.dual || self.julia_mode {
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
        let key = (self.fractal.formula_id(), pal_idx, self.color_method, pal_rev);
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
    fn draw_minimap(&mut self, ctx: &egui::Context) {
        if !self.minimap || self.dual || self.julia_mode {
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
                        let (cx, cy) = self.viewport.center_f64();
                        let nx = ((cx - MINIMAP_CX) / (2.0 * MINIMAP_HX) + 0.5) as f32;
                        let ny = (0.5 - (cy - MINIMAP_CY) / (2.0 * MINIMAP_HY)) as f32;
                        let mk = rect.min + egui::vec2(nx * rect.width(), ny * rect.height());
                        let (sx, _sy) = self.viewport.complex_span();
                        let rw = (sx / (2.0 * MINIMAP_HX)) as f32 * rect.width();
                        let amber = BRAND_ACCENT;
                        if rect.contains(mk) {
                            if rw >= 5.0 {
                                // Shallow: draw the actual view rectangle.
                                let rh = rw * rect.height() / rect.width();
                                let vr = egui::Rect::from_center_size(mk, egui::vec2(rw, rh));
                                p.rect_stroke(
                                    vr,
                                    0.0,
                                    egui::Stroke::new(1.5, amber),
                                    egui::StrokeKind::Middle,
                                );
                            } else {
                                // Deep: a crosshair + dot (the view is sub-pixel here).
                                p.circle_stroke(mk, 4.0, egui::Stroke::new(1.5, amber));
                                let c = 7.0;
                                p.line_segment([mk - egui::vec2(c, 0.0), mk + egui::vec2(c, 0.0)],
                                    egui::Stroke::new(1.0, amber));
                                p.line_segment([mk - egui::vec2(0.0, c), mk + egui::vec2(0.0, c)],
                                    egui::Stroke::new(1.0, amber));
                            }
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
            self.zoom_vel = 0.0;
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
        self.zoom_vel = 0.0;
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
        self.zoom_vel = 0.0;
        self.invalidate_refs();
        self.record_nav();
        self.set_toast(format!("Random location · {}×", fmt_zoom(mag)), ctx);
    }

    /// The Help window: a left-hand table of contents + a scrollable content pane.
    fn help_window(&mut self, ctx: &egui::Context) {
        if !self.help_open {
            return;
        }
        const SECTIONS: [&str; 8] = [
            "Overview",
            "Navigation",
            "Coloring & options",
            "Fractals",
            "How it works",
            "Command line",
            "Shortcuts",
            "About",
        ];
        let mut open = self.help_open;
        egui::Window::new("Fractadyne Help")
            .open(&mut open)
            .default_size([660.0, 520.0])
            .resizable(true)
            .show(ctx, |ui| {
                let toc_w = 150.0;
                // Bound the content width so paragraphs wrap (a horizontal layout would
                // otherwise hand children unbounded width and the text wouldn't wrap).
                let content_w = (ui.available_width() - toc_w - 14.0).max(280.0);
                ui.horizontal_top(|ui| {
                    // Table of contents.
                    ui.allocate_ui_with_layout(
                        egui::vec2(toc_w, ui.available_height()),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_width(toc_w);
                            for (i, name) in SECTIONS.iter().enumerate() {
                                ui.selectable_value(&mut self.help_section, i, *name);
                            }
                        },
                    );
                    ui.separator();
                    // Content (width-bounded → labels wrap).
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        ui.set_width(content_w);
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        match self.help_section {
                            0 => help_overview(ui),
                            1 => help_navigation(ui),
                            2 => help_options(ui),
                            3 => help_fractals(ui),
                            4 => help_methodology(ui),
                            5 => help_command_line(ui),
                            6 => help_shortcuts(ui),
                            _ => help_about(ui),
                        }
                    });
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
            self.max_iter = it.clamp(64, 50_000);
            self.auto_iter = false;
        }
        self.viewport.set_center_mag(v.cx, v.cy, zoom.max(1.0));
        self.viewport.precision = fractadyne_core::precision_for_magnification(zoom);
        self.zoom_vel = 0.0;
        self.invalidate_refs();
        self.record_nav();
        Ok(format!("Imported .kfr location @ {}×", fmt_zoom(zoom)))
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
        self.zoom_vel = 0.0;
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

    /// Toggle the auto-zoom autopilot (single view only).
    fn toggle_autopilot(&mut self, ctx: &egui::Context) {
        self.autopilot = !self.autopilot;
        if self.autopilot {
            self.autopilot_target = (0.5, 0.5);
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
    fn autopilot_step(
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
                    // Ease toward the new target so the dive stays smooth.
                    Some((tx, ty)) => {
                        self.autopilot_target.0 += (tx - self.autopilot_target.0) * 0.6;
                        self.autopilot_target.1 += (ty - self.autopilot_target.1) * 0.6;
                    }
                    None => {
                        self.autopilot = false;
                        self.zoom_vel = 0.0;
                        self.set_toast("Autopilot: no detail ahead (stopped)", ctx);
                        return;
                    }
                }
            }
        }

        // Continuously zoom in toward the target screen fraction.
        let rate = ZOOM_RATE * self.zoom_rate as f64;
        let factor = (-rate * dt).exp();
        let px = self.autopilot_target.0 * self.viewport.width_px;
        let py = self.autopilot_target.1 * self.viewport.height_px;
        self.viewport.zoom_at(px, py, factor);
        self.settle_t = now; // treat as interaction (AA off, throttled reference refresh)
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
        if self.de && self.de_anim && self.palette_anim_speed > 0.0 {
            self.de_phase = (self.de_phase + self.palette_anim_speed * dt).rem_euclid(1.0);
            self.schedule_repaint(ctx);
        }
        // Rotate the relief light direction (cheap — it's a color-pass param).
        if self.light && self.light_anim && self.palette_anim_speed > 0.0 {
            self.light_angle = (self.light_angle
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
        self.update_minimap(ctx, &gpu);

        // Ctrl+S → quick export (no dialog) to the last folder.
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            if let Some((dev, q)) = &gpu {
                self.quick_export(dev.clone(), q.clone());
            }
        }

        // Esc: stop the autopilot / a playing tour first, otherwise leave fullscreen.
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.autopilot {
                self.autopilot = false;
                self.zoom_vel = 0.0;
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

        // CLI render-and-exit: render one image offscreen (or the raw iteration EXR), save
        // it, and quit.
        if self.auto_render && !self.auto_render_done {
            if let Some((dev, q)) = &gpu {
                self.auto_render_done = true;
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
                    self.render_to_file(dev, q, &out)
                };
                match result {
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

        // Auto-zoom autopilot — dive toward detail (advance before drawing so this frame
        // reflects the new view).
        self.autopilot_step(ctx, &gpu);

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

        if !self.auto_benchmark && !self.auto_render && !self.selftest {
            self.autosave(ctx); // don't let a CLI run overwrite the saved session
        }

        // Combined menu bar + action toolbar. `horizontal_wrapped` keeps them on one
        // line when the window is wide enough, and wraps the toolbar below otherwise.
        // (We place the menu buttons directly in the wrapped row rather than via
        // `menu::bar`, which would claim the full width and push the toolbar down.)
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
                        if ui.button("Go to location…").clicked() {
                            self.open_goto();
                            ui.close_menu();
                        }
                        ui.add_enabled_ui(self.nav_undo.len() > 1, |ui| {
                            if ui.button("Undo view  (Backspace)").clicked() {
                                self.undo_view();
                                ui.close_menu();
                            }
                        });
                        ui.add_enabled_ui(!self.nav_redo.is_empty(), |ui| {
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
                            let label = if self.autopilot {
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
                        fmt_zoom_log2(self.viewport.log2_magnification()),
                        fmt_zoom_log2(self.julia_viewport.log2_magnification()),
                    ));
                } else {
                    ui.monospace(format!("zoom {}×", fmt_zoom_log2(self.viewport.log2_magnification())));
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
        if !self.fullscreen && self.right_panel_open {
        egui::SidePanel::right("coloring_panel")
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.small_button("\u{23F4}").on_hover_text("Hide control panel").clicked() {
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
                    .selected_text(COLOR_METHODS[self.color_method as usize].1)
                    .show_ui(ui, |ui| {
                        for (i, (_, name)) in COLOR_METHODS.iter().enumerate() {
                            ui.selectable_value(&mut self.color_method, i as u32, *name);
                        }
                    })
                    .response
                    .on_hover_text(
                        "How escape data maps to color. Stripe / triangle-inequality / \
                         orbit-trap / decomposition reveal orbit structure; distance \
                         shades by proximity to the boundary.",
                    );
                if self.color_method == 1 {
                    ui.add(
                        egui::Slider::new(&mut self.stripe_freq, 1.0..=24.0)
                            .text("Stripe density")
                            .logarithmic(true),
                    );
                }
                if self.color_method == 3 {
                    egui::ComboBox::from_label("Trap shape")
                        .selected_text(TRAP_TYPES[self.trap_type as usize].1)
                        .show_ui(ui, |ui| {
                            for (i, (_, name)) in TRAP_TYPES.iter().enumerate() {
                                ui.selectable_value(&mut self.trap_type, i as u32, *name);
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
                    ui.checkbox(&mut self.light_anim, "Rotate light")
                        .on_hover_text("Spin the light direction over time (uses the Speed slider).");
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

                });
                egui::CollapsingHeader::new("Navigation").default_open(true).show(ui, |ui| {
                ui.add(
                    egui::Slider::new(&mut self.zoom_rate, 0.25..=4.0)
                        .text("Zoom speed")
                        .suffix("×")
                        .logarithmic(true),
                )
                .on_hover_text("Speed of hold-Space continuous zoom (1× ≈ 2× per 1.5 s).");

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
                let span_fe = self.viewport.complex_span_fe();
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
                let log2mag = self.viewport.log2_magnification();
                let resolution = [
                    (rect.width() as f64 * ppp) as u32,
                    (rect.height() as f64 * ppp) as u32,
                ];
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

        // ---- minimap overview ----
        self.draw_minimap(ctx);

        // ---- gradient editor ----
        self.palette_editor_window(ctx);

        // ---- keyboard / help overlay ----
        self.help_window(ctx);

        // ---- transient status toast (e.g. minibrot-finder result) ----
        if let Some((msg, t0)) = self.toast.clone() {
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
                                    egui::RichText::new(msg)
                                        .color(BRAND_TEXT.gamma_multiply(fade)),
                                );
                            });
                    });
                ctx.request_repaint(); // keep fading
            } else {
                self.toast = None;
            }
        }

        // ---- go to location ----
        if self.goto_open {
            let mut open = self.goto_open;
            let mut go = false;
            let mut copy = false;
            egui::Window::new("Go to location")
                .open(&mut open)
                .resizable(false)
                .default_width(420.0)
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new("Center X").weak().small());
                    ui.add(egui::TextEdit::singleline(&mut self.goto_x).desired_width(f32::INFINITY));
                    ui.label(egui::RichText::new("Center Y").weak().small());
                    ui.add(egui::TextEdit::singleline(&mut self.goto_y).desired_width(f32::INFINITY));
                    ui.label(egui::RichText::new("Zoom (magnification)").weak().small());
                    ui.add(egui::TextEdit::singleline(&mut self.goto_zoom).desired_width(220.0));
                    if let Some(m) = &self.goto_msg {
                        ui.colored_label(egui::Color32::from_rgb(0xE0, 0x6C, 0x60), m);
                    }
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        go = ui.button("Go").clicked();
                        if ui.button("Copy").on_hover_text("Copy this location to the clipboard").clicked() {
                            copy = true;
                        }
                        if ui.button("Use current").clicked() {
                            self.goto_x = fractadyne_core::to_decimal_string(&self.viewport.center_x);
                            self.goto_y = fractadyne_core::to_decimal_string(&self.viewport.center_y);
                            self.goto_zoom = fmt_zoom_field(self.viewport.log2_magnification());
                            self.goto_msg = None;
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
                    self.goto_x, self.goto_y, self.goto_zoom
                ));
            }
            if go {
                self.apply_goto(); // clears goto_open on success
            }
            // Closed if the user hit the window's ✕ (open=false) or Go succeeded.
            self.goto_open = open && self.goto_open;
        }

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

        // Navigation history: record a location each time the single view settles after
        // a pan/zoom gesture (its own dedup avoids repeats). Discrete jumps record
        // explicitly. Skipped in dual view.
        let interacting_now = ctx.input(|i| i.time) - self.settle_t < SETTLE_DELAY;
        if self.nav_was_interacting && !interacting_now && !self.dual {
            self.record_nav();
        }
        self.nav_was_interacting = interacting_now;

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
