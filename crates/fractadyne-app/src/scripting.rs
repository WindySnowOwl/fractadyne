//! Scripting & benchmark: TOML keyframe camera tours (`Tools -> Play script`) and the
//! built-in benchmark tour, plus the playback engine that glides center+zoom along the
//! timeline and samples FPS/CPU/RAM. `Playback`/`Bench` are pub(crate) (held as app state).

use crate::{process_memory, version_string, FractadyneApp, FractalKind};
use serde::Deserialize;

/// Script schema version. Bump on a breaking change to an existing key's meaning; purely
/// additive new keys don't need it (unknown keys are ignored, missing ones default — so old
/// and new builds interoperate). A script whose `format_version` exceeds this is from a newer
/// build: we still play the parts we understand but warn that newer features may not apply.
pub(crate) const SCRIPT_FORMAT_VERSION: u32 = 1;

/// On-disk script format (TOML). A keyframe with no `center_x`/`center_y` inherits
/// the previous keyframe's center (handy for pure zoom-in tours). Captions are timed
/// independently of keyframes (so narration can span or overlap camera moves).
#[derive(Deserialize, Default)]
struct ScriptFile {
    #[serde(default)]
    name: String,
    /// Schema version the script was authored for (see `SCRIPT_FORMAT_VERSION`).
    #[serde(default)]
    format_version: Option<u32>,
    #[serde(default, rename = "loop")]
    loop_: bool,
    #[serde(default)]
    keyframe: Vec<KeyframeFile>,
    #[serde(default)]
    caption: Vec<CaptionFile>,
    #[serde(default)]
    callout: Vec<CalloutFile>,
    #[serde(default)]
    spotlight: Vec<SpotlightFile>,
}

/// A spotlight vignette: dim everything outside a soft circle centred on a fractal coordinate.
#[derive(Deserialize, Clone)]
struct SpotlightFile {
    center_x: String,
    center_y: String,
    /// Circle radius as a fraction of the frame height (default 0.25).
    #[serde(default)]
    radius: Option<f64>,
    #[serde(default)]
    at: f64,
    #[serde(default)]
    secs: f64,
    #[serde(default)]
    fade: Option<f64>,
    /// How dark outside the circle, 0..1 (default 0.7).
    #[serde(default)]
    dim: Option<f64>,
    /// Soft-edge width as a fraction of the frame height (default 0.08).
    #[serde(default)]
    softness: Option<f64>,
}

/// A labeled marker anchored to a complex coordinate (tracks the point as the view moves).
#[derive(Deserialize, Clone)]
struct CalloutFile {
    text: String,
    center_x: String,
    center_y: String,
    #[serde(default)]
    at: f64,
    #[serde(default)]
    secs: f64,
    #[serde(default)]
    fade: Option<f64>,
    #[serde(default)]
    size: Option<f64>,
}

/// A timed on-screen caption (narration overlay). Additive to the camera path.
#[derive(Deserialize, Clone)]
struct CaptionFile {
    /// The text to show (supports `\n` for multiple lines).
    text: String,
    /// When it appears on the timeline (seconds from the start).
    #[serde(default)]
    at: f64,
    /// How long it stays (seconds). 0 or omitted ⇒ until the tour ends.
    #[serde(default)]
    secs: f64,
    /// Screen anchor: `top`, `center`, or `bottom` (default).
    #[serde(default)]
    pos: Option<String>,
    /// Fade in/out time (seconds) at each end. Default 0.4.
    #[serde(default)]
    fade: Option<f64>,
    /// Font size in points. Default 22.
    #[serde(default)]
    size: Option<f64>,
}

/// Where a caption sits on screen.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum CaptionPos {
    Top,
    Center,
    Bottom,
}

/// A resolved caption ready to draw.
pub(crate) struct Caption {
    pub(crate) text: String,
    pub(crate) start: f64,
    pub(crate) end: f64,
    pub(crate) fade: f64,
    pub(crate) pos: CaptionPos,
    pub(crate) size: f32,
}

/// Eased fade opacity (0..1) for a timed annotation window `[start, end]` at tour time `t`.
fn fade_alpha(t: f64, start: f64, end: f64, fade: f64) -> f32 {
    if t < start || t > end {
        return 0.0;
    }
    let f = fade.max(1.0e-3);
    let a = ((t - start) / f).min(1.0);
    let b = ((end - t) / f).min(1.0);
    (a.min(b).clamp(0.0, 1.0)) as f32
}

impl Caption {
    /// Opacity (0..1) of this caption at tour time `t`, with eased fade in/out; 0 = not shown.
    pub(crate) fn alpha_at(&self, t: f64) -> f32 {
        fade_alpha(t, self.start, self.end, self.fade)
    }
}

/// A labeled marker anchored to a fractal coordinate — tracks the point as the view pans/zooms.
pub(crate) struct Callout {
    pub(crate) text: String,
    pub(crate) cx: fractadyne_core::BigFloat,
    pub(crate) cy: fractadyne_core::BigFloat,
    pub(crate) start: f64,
    pub(crate) end: f64,
    pub(crate) fade: f64,
    pub(crate) size: f32,
}

impl Callout {
    pub(crate) fn alpha_at(&self, t: f64) -> f32 {
        fade_alpha(t, self.start, self.end, self.fade)
    }
}

/// A spotlight vignette anchored to a fractal coordinate (dims everything outside a soft circle).
pub(crate) struct Spotlight {
    pub(crate) cx: fractadyne_core::BigFloat,
    pub(crate) cy: fractadyne_core::BigFloat,
    pub(crate) radius: f32,
    pub(crate) soft: f32,
    pub(crate) dim: f32,
    pub(crate) start: f64,
    pub(crate) end: f64,
    pub(crate) fade: f64,
}

impl Spotlight {
    pub(crate) fn alpha_at(&self, t: f64) -> f32 {
        fade_alpha(t, self.start, self.end, self.fade)
    }
}

/// The GPU vignette for the first spotlight active at tour time `t`, anchored via `vp` (so it
/// tracks its point + stays a constant on-screen size). Off (`on == 0`) when none is active.
pub(crate) fn vignette_for(spots: &[Spotlight], vp: &fractadyne_core::Viewport, t: f64) -> fractadyne_gpu::Vignette {
    for sp in spots {
        let a = sp.alpha_at(t);
        if a <= 0.0 {
            continue;
        }
        let (vpx, vpy) = vp.complex_to_pixel(&sp.cx, &sp.cy);
        return fractadyne_gpu::Vignette {
            on: 1,
            dim: sp.dim * a, // fade the dimming in/out with the window
            soft: sp.soft,
            center: [(vpx / vp.width_px) as f32, (vpy / vp.height_px) as f32],
            radius: sp.radius,
        };
    }
    fractadyne_gpu::Vignette::default()
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
    /// Timed narration overlays (drawn by the app over the fractal + into exported frames).
    pub(crate) captions: Vec<Caption>,
    /// Coordinate-anchored labeled markers (drawn over the fractal + into exported frames).
    pub(crate) callouts: Vec<Callout>,
    /// Spotlight vignettes (dim outside a soft circle; applied in the color shader).
    pub(crate) spotlights: Vec<Spotlight>,
    /// Current tour time (seconds), updated each frame — so the caption overlay knows what to show.
    pub(crate) cur_t: f64,
}

impl Playback {
    /// Sample the eased camera state at time `e` (seconds, expected in `[0, total]`):
    /// the segment-interpolated center, `ln(magnification)`, fractal, and Julia flag.
    /// Shared by live playback and the headless tour renderer.
    pub(crate) fn sample(&self, e: f64) -> (fractadyne_core::BigFloat, fractadyne_core::BigFloat, f64, FractalKind, bool) {
        let n = self.kfs.len();
        let mut i = n - 1;
        for j in 0..n.saturating_sub(1) {
            if e <= self.kfs[j + 1].at {
                i = j;
                break;
            }
        }
        if i + 1 < n {
            let a = &self.kfs[i];
            let b = &self.kfs[i + 1];
            let seg = (b.at - a.at).max(1.0e-9);
            let u = ((e - a.at) / seg).clamp(0.0, 1.0);
            let ease = u * u * (3.0 - 2.0 * u); // smoothstep
            let lm = a.logmag + (b.logmag - a.logmag) * ease;
            // Precision from octaves (log2 mag) so it stays valid past f64's 1e308× ceiling.
            let octaves = (lm / std::f64::consts::LN_2).max(0.0).ceil() as u64;
            let p = fractadyne_core::precision_for_octaves(octaves);
            (
                fractadyne_core::lerp_bf(&a.cx, &b.cx, ease, p),
                fractadyne_core::lerp_bf(&a.cy, &b.cy, ease, p),
                lm,
                a.fractal,
                a.julia,
            )
        } else {
            let a = &self.kfs[i];
            (a.cx.clone(), a.cy.clone(), a.logmag, a.fractal, a.julia)
        }
    }
}

/// Blit a laid-out galley's glyph coverage as `color` (× `alpha`, straight over) onto a
/// **linear-RGBA** buffer, top-left at `(bx, by)` in frame pixels. `ppp` maps galley points to
/// atlas texels. Caller passes the atlas (one clone) so repeated calls don't re-clone it.
fn blit_galley(
    atlas: &egui::epaint::FontImage, px: &mut [f32], w: u32, h: u32, galley: &egui::Galley,
    bx: f32, by: f32, ppp: f32, color: [f32; 3], alpha: f32,
) {
    let aw = atlas.size[0];
    for row in &galley.rows {
        for g in &row.glyphs {
            let uv = g.uv_rect;
            if uv.max[0] <= uv.min[0] || uv.max[1] <= uv.min[1] {
                continue;
            }
            let ox = (bx + (g.pos.x + uv.offset.x) * ppp).round() as i32;
            let oy = (by + (g.pos.y + uv.offset.y) * ppp).round() as i32;
            for ty in uv.min[1]..uv.max[1] {
                for tx in uv.min[0]..uv.max[0] {
                    let cov = atlas.pixels[ty as usize * aw + tx as usize] * alpha;
                    if cov <= 0.0 {
                        continue;
                    }
                    let dx = ox + (tx - uv.min[0]) as i32;
                    let dy = oy + (ty - uv.min[1]) as i32;
                    if dx < 0 || dy < 0 || dx >= w as i32 || dy >= h as i32 {
                        continue;
                    }
                    let i = (dy as usize * w as usize + dx as usize) * 4;
                    px[i] = color[0] * cov + px[i] * (1.0 - cov);
                    px[i + 1] = color[1] * cov + px[i + 1] * (1.0 - cov);
                    px[i + 2] = color[2] * cov + px[i + 2] * (1.0 - cov);
                }
            }
        }
    }
}

/// Multiply a rectangular region toward black (the soft backing behind annotation text).
fn fill_dark(px: &mut [f32], w: u32, h: u32, x0: f32, y0: f32, x1: f32, y1: f32, amount: f32) {
    let (rx0, ry0) = (x0.max(0.0) as u32, y0.max(0.0) as u32);
    let (rx1, ry1) = ((x1.min(w as f32)) as u32, (y1.min(h as f32)) as u32);
    for y in ry0..ry1 {
        for x in rx0..rx1 {
            let i = (y as usize * w as usize + x as usize) * 4;
            px[i] *= 1.0 - amount;
            px[i + 1] *= 1.0 - amount;
            px[i + 2] *= 1.0 - amount;
        }
    }
}

fn blend_px(px: &mut [f32], w: u32, h: u32, x: i32, y: i32, color: [f32; 3], a: f32) {
    if a <= 0.0 || x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
        return;
    }
    let i = (y as usize * w as usize + x as usize) * 4;
    px[i] = color[0] * a + px[i] * (1.0 - a);
    px[i + 1] = color[1] * a + px[i + 1] * (1.0 - a);
    px[i + 2] = color[2] * a + px[i + 2] * (1.0 - a);
}

/// Anti-aliased ring outline (marker) of radius `r`, line width `thick`.
fn draw_ring(px: &mut [f32], w: u32, h: u32, cx: f32, cy: f32, r: f32, thick: f32, color: [f32; 3], alpha: f32) {
    let lo = (cx - r - thick).floor() as i32;
    let hi = (cx + r + thick).ceil() as i32;
    let lo_y = (cy - r - thick).floor() as i32;
    let hi_y = (cy + r + thick).ceil() as i32;
    for y in lo_y..=hi_y {
        for x in lo..=hi {
            let d = (((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt() - r).abs();
            let a = (1.0 - (d - thick * 0.5).max(0.0)).clamp(0.0, 1.0);
            blend_px(px, w, h, x, y, color, a * alpha);
        }
    }
}

/// A short 2-px-ish leader line from `(x0,y0)` to `(x1,y1)`.
fn draw_line(px: &mut [f32], w: u32, h: u32, x0: f32, y0: f32, x1: f32, y1: f32, color: [f32; 3], alpha: f32) {
    let steps = ((x1 - x0).abs().max((y1 - y0).abs())).ceil().max(1.0) as i32;
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let x = (x0 + (x1 - x0) * t).round() as i32;
        let y = (y0 + (y1 - y0) * t).round() as i32;
        for ( dx, dy) in [(0, 0), (1, 0), (0, 1)] {
            blend_px(px, w, h, x + dx, y + dy, color, alpha);
        }
    }
}

/// Burn a caption into a **linear-RGBA** export frame — the offscreen equivalent of
/// `draw_captions`: a soft dark backing rect + white text (× `alpha`), wrapped and centred on the
/// caption's screen anchor. `atlas` is the (pre-cloned) egui font atlas.
fn stamp_caption(ctx: &egui::Context, px: &mut [f32], w: u32, h: u32, cap: &Caption, alpha: f32) {
    if alpha <= 0.0 || w == 0 || h == 0 {
        return;
    }
    let ppp = ctx.pixels_per_point();
    let pts = (cap.size * (h as f32 / 1080.0) / ppp).max(1.0);
    let galley = ctx.fonts(|f| {
        f.layout(cap.text.clone(), egui::FontId::proportional(pts), egui::Color32::WHITE, w as f32 * 0.8 / ppp)
    });
    let (gw, gh) = (galley.size().x * ppp, galley.size().y * ppp);
    let bx = w as f32 * 0.5 - gw * 0.5;
    let by = match cap.pos {
        CaptionPos::Top => h as f32 * 0.07,
        CaptionPos::Center => h as f32 * 0.5 - gh * 0.5,
        CaptionPos::Bottom => h as f32 * 0.91 - gh,
    };
    let pad = 12.0 * (h as f32 / 1080.0);
    fill_dark(px, w, h, bx - pad, by - pad, bx + gw + pad, by + gh + pad, (alpha * 0.5).min(1.0));
    // Clone the atlas AFTER layout so it contains this text's glyphs (egui fills it lazily).
    ctx.fonts(|f| blit_galley(&f.image(), px, w, h, &galley, bx, by, ppp, [1.0, 1.0, 1.0], alpha));
}

/// Burn a callout (marker ring + leader line + label) at the target's frame pixel `(vpx, vpy)`.
fn stamp_callout(ctx: &egui::Context, px: &mut [f32], w: u32, h: u32, co: &Callout, vpx: f32, vpy: f32, alpha: f32) {
    if alpha <= 0.0 {
        return;
    }
    let ppp = ctx.pixels_per_point();
    let s = (h as f32 / 1080.0).max(0.4); // scale annotation geometry to the frame
    let accent = {
        let c = egui::Rgba::from(crate::theme::BRAND_ACCENT);
        [c.r(), c.g(), c.b()]
    };
    draw_ring(px, w, h, vpx, vpy, 7.0 * s, 2.0 * s, accent, alpha);
    let pts = (co.size * s / ppp).max(1.0);
    let galley = ctx.fonts(|f| f.layout_no_wrap(co.text.clone(), egui::FontId::proportional(pts), egui::Color32::WHITE));
    let (gw, gh) = (galley.size().x * ppp, galley.size().y * ppp);
    let off = 16.0 * s;
    // Label up-right of the marker; flip to the left / below if it would leave the frame.
    let mut bx = vpx + off;
    let mut by = vpy - off - gh;
    if bx + gw + 8.0 * s > w as f32 {
        bx = vpx - off - gw;
    }
    if by < 4.0 * s {
        by = vpy + off;
    }
    draw_line(px, w, h, vpx, vpy, bx + gw * 0.5, by + gh * 0.5, accent, alpha * 0.9);
    let pad = 6.0 * s;
    fill_dark(px, w, h, bx - pad, by - pad, bx + gw + pad, by + gh + pad, (alpha * 0.55).min(1.0));
    ctx.fonts(|f| blit_galley(&f.image(), px, w, h, &galley, bx, by, ppp, [1.0, 1.0, 1.0], alpha));
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
    let captions = sf
        .caption
        .iter()
        .filter(|c| !c.text.is_empty())
        .map(|c| {
            let start = c.at.max(0.0);
            let end = if c.secs > 0.0 { start + c.secs } else { total.max(start) };
            let pos = match c.pos.as_deref().map(|s| s.to_ascii_lowercase()).as_deref() {
                Some("top") => CaptionPos::Top,
                Some("center") | Some("centre") | Some("middle") => CaptionPos::Center,
                _ => CaptionPos::Bottom,
            };
            Caption {
                text: c.text.clone(),
                start,
                end,
                fade: c.fade.unwrap_or(0.4).max(0.0),
                pos,
                size: c.size.unwrap_or(22.0).clamp(8.0, 96.0) as f32,
            }
        })
        .collect();
    let callouts = sf
        .callout
        .iter()
        .filter(|c| !c.text.is_empty())
        .filter_map(|c| {
            let cx = fractadyne_core::parse_bf(&c.center_x)?;
            let cy = fractadyne_core::parse_bf(&c.center_y)?;
            let start = c.at.max(0.0);
            let end = if c.secs > 0.0 { start + c.secs } else { total.max(start) };
            Some(Callout {
                text: c.text.clone(),
                cx,
                cy,
                start,
                end,
                fade: c.fade.unwrap_or(0.4).max(0.0),
                size: c.size.unwrap_or(18.0).clamp(8.0, 96.0) as f32,
            })
        })
        .collect();
    let spotlights = sf
        .spotlight
        .iter()
        .filter_map(|s| {
            let cx = fractadyne_core::parse_bf(&s.center_x)?;
            let cy = fractadyne_core::parse_bf(&s.center_y)?;
            let start = s.at.max(0.0);
            let end = if s.secs > 0.0 { start + s.secs } else { total.max(start) };
            Some(Spotlight {
                cx,
                cy,
                radius: s.radius.unwrap_or(0.25).clamp(0.02, 2.0) as f32,
                soft: s.softness.unwrap_or(0.08).clamp(0.0, 1.0) as f32,
                dim: s.dim.unwrap_or(0.7).clamp(0.0, 1.0) as f32,
                start,
                end,
                fade: s.fade.unwrap_or(0.4).max(0.0),
            })
        })
        .collect();
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
        captions,
        callouts,
        spotlights,
        cur_t: 0.0,
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
        ..Default::default()
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
        let parsed = std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| toml::from_str::<ScriptFile>(&t).ok());
        let script_ver = parsed.as_ref().and_then(|sf| sf.format_version).unwrap_or(0);
        match parsed.and_then(|sf| resolve_script(sf, None)) {
            Some(pb) => {
                if script_ver > SCRIPT_FORMAT_VERSION {
                    self.bench_report = Some(format!(
                        "Note: \"{}\" was authored for a newer script format (v{script_ver} > \
                         v{SCRIPT_FORMAT_VERSION}). Playing what this build understands; newer \
                         annotations may not appear.",
                        pb.name
                    ));
                    self.bench_open = true;
                }
                self.playback = Some(pb);
            }
            None => self.bench_report = Some(format!("Could not load script:\n{}", path.display())),
        }
    }

    /// Draw the active tour captions over the fractal (live playback). Each caption fades in/out
    /// per its timeline window; text is wrapped and centered on its screen anchor over a soft dark
    /// backing so it stays legible on any fractal. (Exported tour frames get the same via a
    /// rasterized overlay — see `render_tour_to_dir`.)
    pub(crate) fn draw_captions(&self, ctx: &egui::Context, rect: egui::Rect) {
        let Some(pb) = &self.playback else { return };
        if pb.captions.is_empty() {
            return;
        }
        let painter =
            ctx.layer_painter(egui::LayerId::new(egui::Order::Middle, egui::Id::new("tour_captions")));
        for cap in &pb.captions {
            let a = cap.alpha_at(pb.cur_t);
            if a <= 0.0 {
                continue;
            }
            let color = egui::Color32::from_white_alpha((a * 240.0) as u8);
            let galley = ctx.fonts(|f| {
                f.layout(cap.text.clone(), egui::FontId::proportional(cap.size), color, rect.width() * 0.8)
            });
            let sz = galley.size();
            let x = rect.center().x - sz.x * 0.5;
            let y = match cap.pos {
                CaptionPos::Top => rect.top() + rect.height() * 0.07,
                CaptionPos::Center => rect.center().y - sz.y * 0.5,
                CaptionPos::Bottom => rect.bottom() - rect.height() * 0.09 - sz.y,
            };
            let pos = egui::pos2(x, y);
            let pad = egui::vec2(12.0, 7.0);
            let bg = egui::Rect::from_min_size(pos - pad, sz + pad * 2.0);
            painter.rect_filled(bg, 5.0, egui::Color32::from_black_alpha((a * 130.0) as u8));
            painter.galley(pos, galley, color);
        }
    }

    /// Draw the active tour callouts (live playback): a marker ring at each anchored fractal
    /// coordinate — tracking the point as the view moves — plus a labeled leader. Off-screen
    /// anchors are skipped. Exported frames get the same via `stamp_callout`.
    pub(crate) fn draw_callouts(&self, ctx: &egui::Context, rect: egui::Rect) {
        let Some(pb) = &self.playback else { return };
        if pb.callouts.is_empty() {
            return;
        }
        let ppp = ctx.pixels_per_point() as f64;
        let painter =
            ctx.layer_painter(egui::LayerId::new(egui::Order::Middle, egui::Id::new("tour_callouts")));
        let with_a = |c: egui::Color32, a: f32| {
            egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (a * 255.0) as u8)
        };
        for co in &pb.callouts {
            let a = co.alpha_at(pb.cur_t);
            if a <= 0.0 {
                continue;
            }
            // The viewport tracks device pixels; convert to egui points, offset by the panel origin.
            let (vpx, vpy) = self.viewport.complex_to_pixel(&co.cx, &co.cy);
            let sp = egui::pos2(rect.min.x + (vpx / ppp) as f32, rect.min.y + (vpy / ppp) as f32);
            if !rect.contains(sp) {
                continue;
            }
            let accent = with_a(crate::theme::BRAND_ACCENT, a);
            painter.circle_stroke(sp, 7.0, egui::Stroke::new(2.0, accent));
            painter.circle_filled(sp, 1.8, accent);
            let galley = ctx.fonts(|f| {
                f.layout_no_wrap(co.text.clone(), egui::FontId::proportional(co.size), with_a(egui::Color32::WHITE, a))
            });
            let gs = galley.size();
            let off = 16.0;
            let mut lp = egui::pos2(sp.x + off, sp.y - off - gs.y);
            if lp.x + gs.x + 8.0 > rect.right() {
                lp.x = sp.x - off - gs.x;
            }
            if lp.y < rect.top() + 4.0 {
                lp.y = sp.y + off;
            }
            painter.line_segment([sp, lp + gs * 0.5], egui::Stroke::new(1.5, accent));
            let pad = egui::vec2(6.0, 4.0);
            let bg = egui::Rect::from_min_size(lp - pad, gs + pad * 2.0);
            painter.rect_filled(bg, 4.0, egui::Color32::from_black_alpha((a * 150.0) as u8));
            painter.galley(lp, galley, with_a(egui::Color32::WHITE, a));
        }
    }

    /// Render a keyframe-tour script (TOML) to a numbered PNG frame sequence — the headless
    /// `--render-tour` mode for producing a deep-zoom dive video. Steps the timeline at a
    /// fixed `fps`, rendering each frame at `width×height` (× `ss` supersampling) via the
    /// offscreen export path. Blocking; assemble the frames afterward (e.g. with ffmpeg).
    pub(crate) fn render_tour_to_dir(
        &mut self,
        ctx: &egui::Context,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        script_path: &std::path::Path,
        fps: f64,
        width: u32,
        height: u32,
        ss: u32,
        out_dir: &std::path::Path,
    ) -> Result<String, String> {
        let text = std::fs::read_to_string(script_path)
            .map_err(|e| format!("read {}: {e}", script_path.display()))?;
        let sf: ScriptFile = toml::from_str(&text).map_err(|e| format!("parse script: {e}"))?;
        if sf.format_version.unwrap_or(0) > SCRIPT_FORMAT_VERSION {
            eprintln!(
                "Warning: script format v{} is newer than this build (v{SCRIPT_FORMAT_VERSION}); \
                 newer annotations may not render.",
                sf.format_version.unwrap_or(0)
            );
        }
        let pb = resolve_script(sf, None).ok_or("script has no keyframes")?;
        std::fs::create_dir_all(out_dir)
            .map_err(|e| format!("create {}: {e}", out_dir.display()))?;
        // Single-view offscreen render at the requested frame size.
        self.dual = false;
        self.viewport = fractadyne_core::Viewport::new(width as f64, height as f64);
        self.export_width = width;
        self.export_ss = ss.max(1);
        let fps = fps.max(1.0);
        let frames: u64 = if pb.total <= 0.0 { 1 } else { (pb.total * fps).round() as u64 + 1 };
        println!(
            "Rendering tour \"{}\": {frames} frames at {width}×{height} ss{}, {fps} fps ({:.1}s)…",
            pb.name, self.export_ss, pb.total
        );
        let meta = self.view_metadata();
        for fi in 0..frames {
            let t = if pb.total <= 0.0 { 0.0 } else { (fi as f64 / fps).min(pb.total) };
            let (cx, cy, logmag, fractal, julia) = pb.sample(t);
            self.fractal = fractal;
            self.julia_mode = julia && fractal.supports_julia();
            self.viewport
                .set_center_log2mag(cx, cy, logmag / std::f64::consts::LN_2);
            // Render the frame to pixels, then burn in the watermark + any active captions.
            let mut req = self.current_export_request_for(&self.viewport, self.julia_mode);
            req.width = width;
            req.height = height;
            req.ss = self.export_ss;
            req.max_iter = req.max_iter.max(200);
            req.vignette = vignette_for(&pb.spotlights, &self.viewport, t); // spotlight for this frame
            let progress = std::sync::atomic::AtomicU32::new(0);
            let cancel = std::sync::atomic::AtomicBool::new(false);
            let res = fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel)
                .map_err(|e| format!("frame {fi}: {e}"))?;
            let mut px = res.pixels;
            let (rw, rh) = (res.width, res.height);
            self.apply_watermark(&mut px, rw, rh);
            for cap in &pb.captions {
                let a = cap.alpha_at(t);
                if a > 0.0 {
                    stamp_caption(ctx, &mut px, rw, rh, cap, a);
                }
            }
            for co in &pb.callouts {
                let a = co.alpha_at(t);
                if a <= 0.0 {
                    continue;
                }
                let (vpx, vpy) = self.viewport.complex_to_pixel(&co.cx, &co.cy);
                if vpx >= 0.0 && vpy >= 0.0 && vpx < rw as f64 && vpy < rh as f64 {
                    stamp_callout(ctx, &mut px, rw, rh, co, vpx as f32, vpy as f32, a);
                }
            }
            let frame_path = out_dir.join(format!("frame_{fi:05}.png"));
            fractadyne_export::write_png(&frame_path, res.width, res.height, &px, Some(&meta))
                .map_err(|e| format!("frame {fi}: {e}"))?;
            if fi % 10 == 0 || fi + 1 == frames {
                println!("  frame {}/{frames}", fi + 1);
            }
        }
        Ok(format!(
            "Rendered {frames} frame(s) → {}\nAssemble e.g.: ffmpeg -framerate {fps} -i \
             {}/frame_%05d.png -c:v libx264 -pix_fmt yuv420p tour.mp4",
            out_dir.display(),
            out_dir.display()
        ))
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
        pb.cur_t = e; // for the caption overlay
        let (cx, cy, logmag, fractal, julia) = pb.sample(e);
        if fractal != self.fractal || julia != self.julia_mode {
            self.fractal = fractal;
            self.julia_mode = julia && fractal.supports_julia();
            self.invalidate_refs();
        }
        // log2 path so playback stays exact past f64's 1e308× ceiling.
        self.viewport.set_center_log2mag(cx, cy, logmag / std::f64::consts::LN_2);
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
