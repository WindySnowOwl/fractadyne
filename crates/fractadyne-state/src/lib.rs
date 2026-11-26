//! App state & persistence (DESIGN.md §13).
//!
//! M1: serialize the session (location, zoom, coloring) to a human-readable TOML
//! file in the OS config dir, written atomically (temp + rename). Bookmarks /
//! presets library and arbitrary-precision coordinates come later.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod durable;

/// On-disk schema version of the persisted session. Bump this when a change could make an
/// **older** build misread a **newer** file (e.g. a field's meaning changes). The schema is
/// otherwise additive — new fields use `#[serde(default)]`, so a newer file still loads
/// best-effort on an older build; the version only drives the "saved by a newer Fractadyne"
/// warning. A file with no `state_version` (written before versioning) is treated as v1.
pub const STATE_FORMAT_VERSION: u32 = 1;

fn default_state_version() -> u32 {
    1
}

/// The largest "Divide iterations" value ([`SessionState::iter_div`]). At the default Cycle a
/// palette then spans about half a million iterations; a view that needs more than that is one for
/// normalization, which fits the palette to the view's range instead.
pub const ITER_DIV_MAX: u32 = 10_000;

/// `1`: the iteration count is colored as it is.
pub fn default_iter_div() -> u32 {
    1
}

/// Persisted session. The center is stored at **full precision** as decimal strings
/// (`center_x_str`/`center_y_str`) so deep-zoom locations survive quit/restart; the
/// `f64` `center_x`/`center_y` remain for display and backward compatibility with
/// older session files (used as a fallback when the strings are absent).
/// One segment of a custom gradient, mirroring `fractadyne_color::segment::Segment`.
///
/// ⚠**`blend` and `space` are a FILE FORMAT**: they are GIMP's `.ggr` numbering (blend 0 linear,
/// 1 curved, 2 sine, 3 sphere-increasing, 4 sphere-decreasing; space 0 RGB, 1 HSV counter-
/// clockwise, 2 HSV clockwise), and they are written into the user's session file. Renumbering
/// them would silently re-interpret every saved gradient, so the mapping lives in
/// `fractadyne-color` next to a test that pins it in both directions.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PaletteSegment {
    pub left: f32,
    pub mid: f32,
    pub right: f32,
    /// DISPLAY-space RGBA, `0..1` — the space the renderer writes straight to the framebuffer.
    pub left_color: [f32; 4],
    pub right_color: [f32; 4],
    #[serde(default)]
    pub blend: u8,
    #[serde(default)]
    pub space: u8,
    /// Control points for blend kind 5 (cubic-Bézier ease): `[x1, y1, x2, y2]`. Ignored by kinds
    /// 0–4, which have no parameters.
    ///
    /// ⭐**The kind stays a bare number and the parameters travel BESIDE it**, so the `.ggr`
    /// numbering — which is what `blend` is — keeps meaning exactly what it always did, and the
    /// round-trip test that pins it stays honest.
    /// ⚠A session written before kind 5 existed defaults this to `[0.0; 4]`, which happens to be
    /// the IDENTITY curve (`x(u) = y(u) = u³`, so `y = x`) rather than something arbitrary — so a
    /// file that somehow carries kind 5 with no parameters renders as a straight line.
    #[serde(default)]
    pub blend_params: [f32; 4],
}

/// How a picture is colored: every coloring setting that is saved or sent (coloring plan B4). One
/// struct, so a setting added here travels everywhere it does: the session file (flattened into
/// [`SessionState`], so the keys are the session's own), a render farm's job (nested in its
/// `RenderSettings`) and, through the app, views and the command line.
///
/// The method and the trap are stored by key, as the session always stored them; the app reads
/// them through its method table (`fractadyne_core::coloring`). The animations (palette, light
/// and glow) are not here: they move a picture, they do not describe one, and a farm job turns
/// them off.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ColoringSpec {
    /// The preset's number in an older build's list (`fractadyne_color::LEGACY_PRESET_ORDER`):
    /// written so an older build reopening this session keeps the palette, and read only when
    /// `palette_id` is absent (a session saved before ids).
    pub palette_idx: usize,
    /// The preset's id (`fractadyne_color::Palette::id`), what a session is read by. Empty in a
    /// session saved before ids; `serde(default)` keeps those loadable.
    #[serde(default)]
    pub palette_id: String,
    pub cycle: f32,
    /// "Divide iterations": the Smooth method's iteration count is divided by this before it is
    /// colored, so one trip through the palette takes this many times more iterations than the
    /// Cycle slider alone allows (its slowest is one palette per 250 iterations). `1` = off, the
    /// default; `serde(default)` keeps older session files loadable. Kalles Fraktaler's "Divide
    /// iteration" is the same idea.
    #[serde(default = "default_iter_div")]
    pub iter_div: u32,
    pub offset: f32,
    /// Log-scaled palette mapping (default off) — applies wherever normalization is active.
    /// `serde(default)` keeps older session files loadable.
    #[serde(default)]
    pub log_palette: bool,
    /// The curve the palette map takes a value through wherever the palette is normalized (coloring
    /// plan C2, `fractadyne_core::coloring::Transfer` by key: linear, sqrt, cbrt, root4, log,
    /// loglog, square). Log is written as older builds write it, `log_palette` on with `transfer`
    /// linear, so a log view reads whole in either; `transfer = "log"` reads as log too.
    #[serde(default = "default_transfer")]
    pub transfer: String,
    /// How the palette continues past its end (`Wrap` by key: repeat, mirror, clamp).
    #[serde(default = "default_wrap")]
    pub wrap: String,
    /// The flat bands a palette cycle is cut into; 0 for none.
    #[serde(default)]
    pub steps: u32,
    /// Equalize colors (coloring plan C6): the share, 0 to 1, of histogram coloring mixed into the
    /// fitted palette map, so each color takes about the same area of the picture (Smooth, wherever
    /// the palette is normalized). 0, the default: the fitted map alone.
    #[serde(default)]
    pub equalize: f32,
    /// Blend in OKLab (coloring plan C7): a palette's RGB segments blended in OKLab, a perceptual
    /// space, rather than between their display (sRGB-encoded) values. Off by default.
    #[serde(default)]
    pub palette_oklab: bool,
    /// Live deep-palette auto-normalization (default on). `serde(default)` keeps older files loadable.
    #[serde(default = "default_true")]
    pub normalize_live: bool,
    /// Fit the palette to the measured escape range unconditionally, instead of only when the view
    /// aliases (default off). `serde(default)` = `false` keeps older session files loadable.
    ///
    /// ⭐**The companion to `normalize_live`, not a replacement for it.** `normalize_live` is an
    /// ALIASING guard: it engages on `mean|Δ smooth-iter| × cycle > 0.5` (Nyquist), so on a
    /// genuinely smooth view it declines by design and the palette is left alone. That is correct
    /// anti-aliasing and it is NOT what the name promises — users read "normalize" as "fit the
    /// palette to this view's range", asked for it repeatedly, and got a control that appeared to
    /// do nothing. Rather than move the calibrated threshold (which has regressed in both
    /// directions before — flat-gray at shallow depth, and "cities at night"), the range fit is its
    /// own switch.
    #[serde(default)]
    pub normalize_fit: bool,
    /// Distance-estimate relief lighting: enabled, light angle (rad), relief strength.
    #[serde(default)]
    pub light: bool,
    #[serde(default = "default_light_angle")]
    pub light_angle: f32,
    #[serde(default = "default_light_height")]
    pub light_height: f32,
    /// The relief's surface and highlight (coloring plan C5): its slope from the distance (45
    /// degrees `light_steep_px` output pixels from the set, steeper nearer, flat far away) instead
    /// of 45 degrees everywhere; on it, a highlight of `light_highlight` strength, `light_shininess`
    /// and `light_highlight_color` (display RGB), drawn only with the slope from the distance.
    #[serde(default)]
    pub light_steep: bool,
    #[serde(default = "default_light_steep_px")]
    pub light_steep_px: f32,
    #[serde(default = "default_light_highlight")]
    pub light_highlight: f32,
    #[serde(default = "default_light_shininess")]
    pub light_shininess: f32,
    #[serde(default = "default_light_highlight_color")]
    pub light_highlight_color: [f32; 3],
    /// Distance-estimate glow: enabled, blend strength, contour width.
    #[serde(default)]
    pub de: bool,
    #[serde(default = "default_de_strength")]
    pub de_strength: f32,
    #[serde(default = "default_de_width")]
    pub de_width: f32,
    /// Edge shading (coloring plan C3): escaped pixels near the set shaded toward `edge_color`, or
    /// toward the inside's color with `edge_inside` (the set's edge smoothed by its distance), by
    /// `edge_strength` at the set, fading out `edge_width` output pixels from it along `edge_curve`
    /// (`fractadyne_core::coloring::EdgeCurve` by key: linear, sqrt, square, smooth, hard).
    #[serde(default)]
    pub edge: bool,
    #[serde(default)]
    pub edge_inside: bool,
    #[serde(default = "default_edge_width")]
    pub edge_width: f32,
    #[serde(default = "default_edge_strength")]
    pub edge_strength: f32,
    #[serde(default = "default_edge_curve")]
    pub edge_curve: String,
    /// Display-space RGB.
    #[serde(default)]
    pub edge_color: [f32; 3],
    /// Equipotential lines (coloring plan C10): lines along the levels of the smooth count,
    /// `equipotential_width` output pixels wide and at least `equipotential_spacing` apart at every
    /// depth (finer levels fade in as the lines spread), mixed toward `equipotential_color` (display
    /// RGB) by `equipotential_strength` on a line.
    #[serde(default)]
    pub equipotentials: bool,
    #[serde(default = "default_equipotential_spacing")]
    pub equipotential_spacing: f32,
    #[serde(default = "default_equipotential_width")]
    pub equipotential_width: f32,
    #[serde(default = "default_equipotential_strength")]
    pub equipotential_strength: f32,
    #[serde(default)]
    pub equipotential_color: [f32; 3],
    /// Coloring method: "smooth" | "stripe" | "triangle" | "trap" | "distance" |
    /// "decomposition".
    #[serde(default = "default_color_method")]
    pub color_method: String,
    /// Stripe-average angular frequency (method = "stripe").
    #[serde(default = "default_stripe_freq")]
    pub stripe_freq: f32,
    /// Stripe average over the TAIL only (method = "stripe"): an exponentially-weighted window
    /// over the last `stripe_tail_len` iterations instead of the whole orbit, so a deep view —
    /// where every orbit shadows the reference for all but its last few hundred iterates — keeps
    /// its contrast. Off = the classic full-orbit average.
    #[serde(default)]
    pub stripe_tail: bool,
    /// Window length in iterations for `stripe_tail`.
    #[serde(default = "default_stripe_tail_len")]
    pub stripe_tail_len: u32,
    /// Orbit-trap shape, by key (`fractadyne_core::coloring::TrapType`: "point", "cross", "circle",
    /// "line", "square", "hexagon", "star", "sine", "s-curve"; method = "trap").
    #[serde(default = "default_trap_type")]
    pub trap_type: String,
    /// Where the orbit trap sits in the orbit's plane, its turn in degrees counterclockwise and its
    /// size (coloring plan F4, `fractadyne_core::coloring::TrapPlace`): 0, 0, 0 and 1, each shape as
    /// it always was. A shape ignores what it does not use (the point its turn and size, the cross
    /// its size, the circle its turn). A session from before them reads the defaults.
    #[serde(default)]
    pub trap_x: f32,
    #[serde(default)]
    pub trap_y: f32,
    #[serde(default)]
    pub trap_rotation: f32,
    #[serde(default = "default_trap_size")]
    pub trap_size: f32,
    /// The inside's own trap, when the orbit trap method's inside is colored by a trap of its own:
    /// its shape's key, or empty for the method's trap (the default, and every session from before
    /// it). Its placement as the trap's above.
    #[serde(default)]
    pub interior_trap: String,
    #[serde(default)]
    pub interior_trap_x: f32,
    #[serde(default)]
    pub interior_trap_y: f32,
    #[serde(default)]
    pub interior_trap_rotation: f32,
    #[serde(default = "default_trap_size")]
    pub interior_trap_size: f32,
    /// The sine wave trap's phase, in degrees (a full wave every 360°), the method's and the
    /// inside's own; 0, and absent from every session from before it: as before.
    #[serde(default)]
    pub trap_phase: f32,
    #[serde(default)]
    pub interior_trap_phase: f32,
    /// How many odd harmonics the square wave trap sums, the method's and the inside's own
    /// (`fractadyne_core::coloring::TrapPlace::harmonics`, 1 to 32); 5, and absent from every session
    /// from before it.
    #[serde(default = "default_trap_harmonics")]
    pub trap_harmonics: u32,
    #[serde(default = "default_trap_harmonics")]
    pub interior_trap_harmonics: u32,
    /// How many of the sine wave trap's harmonics are drawn over it, the method's and the inside's own
    /// (`fractadyne_core::coloring::TrapPlace::sine_harmonics`, 1 to 32); 1, the wave alone, and
    /// absent from every session from before it.
    #[serde(default = "default_sine_harmonics")]
    pub trap_sine_harmonics: u32,
    #[serde(default = "default_sine_harmonics")]
    pub interior_trap_sine_harmonics: u32,
    /// The view's trap outline (coloring plan F4, part 4), which an Outline trap measures from: its
    /// text (`fractadyne_core::outline::Outline::text`), empty for none, and what it was loaded from.
    #[serde(default)]
    pub trap_outline: String,
    #[serde(default)]
    pub trap_outline_name: String,
    /// The orbit trap's TAIL (coloring plan F4, part 5), the method's trap and the inside's alike:
    /// every hit's distance doubles each `trap_tail_len` iterations it lies before the orbit's end
    /// (`fractadyne_core::coloring::Trap::tail`), so a deep view, where every orbit shares its
    /// nearest approach with the reference's, draws again. Off, and absent from every session from
    /// before it: the nearest approach of the whole orbit.
    #[serde(default)]
    pub trap_tail: bool,
    /// The tail's length in iterations, 4 to 2^20 (`fractadyne_core::coloring::TRAP_TAIL_*`).
    #[serde(default = "default_trap_tail_len")]
    pub trap_tail_len: u32,
    /// Custom gradient stops `[pos, r, g, b]` (DISPLAY-space RGB) from the gradient editor.
    #[serde(default)]
    pub custom_palette: Vec<[f32; 4]>,
    /// Read `custom_palette` as **bands** (one flat color per entry, no interpolation) rather
    /// than as gradient stops — Fractint `.map` semantics, set by importing one. `false` for
    /// every session written before palette import existed, which is the right reading of an
    /// editor-authored gradient.
    #[serde(default)]
    pub custom_palette_flat: bool,
    /// A full SEGMENT gradient, when the palette is one a stop list cannot express.
    ///
    /// ⭐**This exists because `.ggr` is a superset of `custom_palette`.** A GIMP gradient carries a
    /// per-segment midpoint, one of five blend curves and a color space (RGB or either way round
    /// the hue wheel); a `[pos, r, g, b]` list holds none of those, so persisting an imported
    /// `.ggr` as stops would have quietly flattened every curve and every hue sweep on the first
    /// restart. When this is non-empty it WINS over `custom_palette`.
    #[serde(default)]
    pub custom_segments: Vec<PaletteSegment>,
    /// Use the custom gradient instead of the selected preset.
    #[serde(default)]
    pub use_custom_palette: bool,
    /// Duotone palette (two-color ramp) / binary palette (flat in-set vs out-of-set),
    /// sharing the two colors (DISPLAY-space RGB).
    #[serde(default)]
    pub use_duotone: bool,
    #[serde(default)]
    pub use_binary: bool,
    #[serde(default = "default_duotone_lo")]
    pub duotone_lo: [f32; 3],
    #[serde(default = "default_duotone_hi")]
    pub duotone_hi: [f32; 3],
    /// What colors the inside of the set (coloring plan B5): `"solid"` or `"trap"`, or empty for the
    /// method's own (the orbit trap colors it by its trap, every other method one color).
    #[serde(default)]
    pub interior: String,
    /// The interior palette, a preset's id; empty for the outside palette.
    #[serde(default)]
    pub interior_palette: String,
    /// The color of a solid inside (coloring plan C1), display-space RGB. Duotone and Binary keep
    /// their own (their dark color); this is every other palette's.
    #[serde(default = "default_interior_color")]
    pub interior_color: [f32; 3],
    /// Fit the inside's palette to the trap values measured inside the set on screen (C4): 1.5
    /// sweeps over their 1st to 99th percentiles. Live views and exports of them; an offline render
    /// has nothing measured and keeps the fixed rate.
    #[serde(default)]
    pub interior_fit: bool,
    /// Anti-alias the palette (coloring plan C8): where its bands are finer than a pixel, the pixel
    /// shows their average, over a footprint sized from the distance estimate (the next pixels'
    /// coordinates where there is none). On by default, a session from before it included; a view
    /// that turns it off says `palette_aa=0`.
    #[serde(default = "default_true")]
    pub palette_aa: bool,
    /// The palette's brightness, contrast and saturation (user request 2026-11-15;
    /// `fractadyne_color::adjust`): -1..1, -1..1 and 0..2, neutral at 0, 0 and 1. Applied to the
    /// baked palette, the inside's own included. A session or view from before them reads neutral.
    #[serde(default)]
    pub palette_brightness: f32,
    #[serde(default)]
    pub palette_contrast: f32,
    #[serde(default = "default_palette_saturation")]
    pub palette_saturation: f32,
    /// The dual view's Julia pane colored with the Mandelbrot pane's palette map, one iteration on
    /// (coloring plan C11): the Julia point z = c then shows the color of the Mandelbrot point c,
    /// whose orbit it continues. Escape-count coloring only. Off: each pane its own map.
    #[serde(default)]
    pub julia_linked: bool,
    /// A custom coloring hook's text (coloring plan B10, `fractadyne_gpu::color_hook`): a WGSL
    /// `custom_color` that colors the view in place of the method. Empty: none. The text, never a
    /// path, so a farm job and a shared view carry the hook itself.
    #[serde(default)]
    pub coloring_hook: String,
    /// The hook's parameters `p1`..`p4`; empty for the hook's own defaults.
    #[serde(default)]
    pub coloring_hook_params: Vec<f32>,
}

fn default_transfer() -> String {
    "linear".to_string()
}

fn default_wrap() -> String {
    "repeat".to_string()
}

fn default_palette_saturation() -> f32 {
    1.0
}

/// The near-black every solid inside had before C1.
pub fn default_interior_color() -> [f32; 3] {
    [0.02, 0.02, 0.03]
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SessionState {
    /// Schema version this file was written with (see [`STATE_FORMAT_VERSION`]). Missing ⇒ v1.
    #[serde(default = "default_state_version")]
    pub state_version: u32,
    pub center_x: f64,
    pub center_y: f64,
    /// Full-precision center (decimal). Empty ⇒ fall back to the `f64` fields.
    #[serde(default)]
    pub center_x_str: String,
    #[serde(default)]
    pub center_y_str: String,
    /// The center's source expression, when it was entered as one: the `center_re_expr=` /
    /// `center_im_expr=` (+ `center_re_offset=` / `center_im_offset=`) lines a view carries. Restored
    /// so a deeper zoom after a restart stays exact, re-derived from the expression instead of
    /// frozen at the digits `center_x_str` was written with. Empty for a plain decimal center.
    #[serde(default)]
    pub center_expr: String,
    /// Scale mantissa. Paired with `units_per_pixel_e` (a base-2 exponent) so deep-zoom
    /// locations past `f64`'s ~1e308× range survive quit/restart. Old saves (no exponent
    /// field) stored the full `f64` value here with an implicit exponent of 0 — still valid.
    pub units_per_pixel: f64,
    #[serde(default)]
    pub units_per_pixel_e: i32,
    pub max_iter: u32,
    pub auto_iter: bool,
    /// How the picture is colored ([`ColoringSpec`]), flattened: its fields are this file's
    /// keys exactly as they were before it existed, so sessions read both ways.
    #[serde(flatten)]
    pub coloring: ColoringSpec,
    /// The file the coloring hook was loaded from, watched while the app runs (B10). The session's
    /// alone: a farm job and a view carry the hook's text (`ColoringSpec::coloring_hook`), never
    /// the user's folders.
    #[serde(default)]
    pub coloring_hook_path: String,
    /// Continuous-zoom speed multiplier (1.0 = default ~2× per 1.5 s). `serde(default)`
    /// keeps older session files (written before this field) loadable.
    #[serde(default = "default_zoom_rate")]
    pub zoom_rate: f32,
    /// Click-to-zoom tool: when on, a left-click in the single view dives in by `click_zoom_factor`
    /// (right-click backs out), recentered on the point. Off by default; `serde(default)` = `false`
    /// keeps older session files loadable.
    #[serde(default)]
    pub click_zoom: bool,
    /// Magnification per click-to-zoom click (2–100×). `serde(default)` seeds older files at 10×.
    #[serde(default = "default_click_zoom_factor")]
    pub click_zoom_factor: f32,
    /// Click-to-zoom: after the jump, solve for the nearest minibrot nucleus and settle onto it.
    /// Off by default. The solve is arbitrary-precision and runs off the UI thread, so the view
    /// jumps immediately and corrects itself when the answer lands.
    #[serde(default)]
    pub click_zoom_snap: bool,
    /// While zooming in with the wheel or Space, pull the zoom toward a nearby minibrot nucleus
    /// (or the real axis) found near the cursor, so it lands on it. On by default; `serde(default)`
    /// turns it on for older session files too.
    #[serde(default = "default_snap_zoom")]
    pub snap_zoom: bool,
    /// Auto-zoom (autopilot) dive limit as log2(magnification); the depth at which the hands-free
    /// dive stops. Default 900 (≈1e271×). Past the smooth regime the autopilot switches to a
    /// stepped dive to reach this depth.
    #[serde(default = "default_autopilot_dive_log2")]
    pub autopilot_dive_log2: f64,
    /// What the auto-zoom puts first: `"speed"` (zoom at the set rate; the screen refreshes with
    /// the first frame proven to hold detail, at adaptive resolution) or `"quality"` (every
    /// refresh is a fully resolved native-resolution frame, and the zoom is paced so the held
    /// frame never magnifies far before the next one lands). Unknown values read as `speed`.
    #[serde(default = "default_autopilot_priority")]
    pub autopilot_priority: String,
    /// What the auto-zoom aims at: `"detail"` (the richest edge in view, re-chosen as it goes)
    /// or `"misiurewicz"` (the nearest Misiurewicz point, solved once and held exactly — the
    /// structure around it repeats forever). Unknown values read as `detail`.
    #[serde(default = "default_autopilot_target")]
    pub autopilot_target: String,
    /// Live-render work-budget multiplier (× the built-in `WORK_BUDGET`). Higher renders the live
    /// deep-zoom view at fuller resolution (crisper) at the cost of frame-rate / GPU-watchdog margin;
    /// does not affect exports. Default 1.0.
    #[serde(default = "default_work_budget_scale")]
    pub work_budget_scale: f64,
    /// Floor on the adaptive motion resolution (0.30–1.0): the lowest fraction of native a moving
    /// deep-zoom frame shrinks to. Higher = sharper (less pixelated) motion, lower frame rate.
    /// Default 0.30. `serde(default)` keeps older session files loadable.
    #[serde(default = "default_min_motion_res")]
    pub min_motion_res: f32,
    /// Prefer detail over motion smoothness while zooming (reproject the last detailed frame
    /// during motion instead of re-iterating coarse). Default off.
    #[serde(default)]
    pub prefer_detail: bool,
    /// A second graphics device for the live view (Advanced ▸ Second graphics card): the
    /// `--list-adapters` number of a Vulkan card, or `same` for a second device on the window's own
    /// card, that renders frames of a moving deep view and some of the supersampling samples while
    /// it settles; `off` for none. Empty = never chosen, which means `same` (the default since
    /// 2026-10-09; before it, empty meant off). `serde(default)` keeps older session files loadable.
    #[serde(default)]
    pub live_worker_gpu: String,
    /// Draw the elapsed-time overlay over the view: a large clock reading the SAME `+12.345s` the
    /// log stamps every line with, so a screen recording of a live problem can be lined up against
    /// the log frame by frame. A diagnostic, off by default. `serde(default)` = `false` keeps older
    /// session files loadable.
    #[serde(default)]
    pub show_timestamp: bool,
    /// While the auto-zoom runs, outline the region of the view it is zooming into and join its
    /// corners to the corners of the screen, so where the dive is heading can be seen before it
    /// gets there. Off by default. `serde(default)` = `false` keeps older session files loadable.
    #[serde(default)]
    pub show_zoom_target: bool,
    /// The live-render front panel over the view (View ▸ Live render panel, key `B`): `"off"`,
    /// `"strip"`, `"panel"` or `"full"`. Empty or unknown = off. `serde(default)` keeps older
    /// session files loadable.
    #[serde(default)]
    pub live_panel: String,
    /// The view corner the panel snaps to: `"tl"`, `"tr"`, `"bl"` or `"br"`. Empty = top-left.
    #[serde(default)]
    pub live_panel_corner: String,
    /// The panel's opacity, 0.3–1.0. `serde(default)` = the 0.95 default.
    #[serde(default = "default_live_panel_opacity")]
    pub live_panel_opacity: f32,
    /// Supersampling / anti-alias factor (1 = off, 2/3/4/8 = N×N).
    #[serde(default = "default_aa")]
    pub aa: u32,
    /// Play a sound when a render/export finishes (user request 2026-08-16, FRACTINT-style).
    /// Default ON; the checkbox lives next to the other render settings.
    #[serde(default = "default_finish_sound")]
    pub finish_sound: bool,
    /// Frame-rate cap in FPS; **`0` = uncapped**. Defaults to 60. Stored as a plain `f64`
    /// (not `Option`) so the *uncapped* choice round-trips: TOML omits `None`, which would
    /// otherwise reload as the default 60 instead of staying uncapped.
    #[serde(default = "default_fps_cap")]
    pub fps_cap: f64,
    /// Size limit of the on-disk reference-orbit cache, in MB (File ▸ Settings ▸ Reference
    /// cache…). Default 1024. `serde(default)` keeps older session files loadable.
    #[serde(default = "default_orbit_cache_mb")]
    pub orbit_cache_mb: u32,
    /// Last-used export settings (remembered across sessions).
    #[serde(default = "default_export_width")]
    pub export_width: u32,
    #[serde(default = "default_export_ss")]
    pub export_ss: u32,
    #[serde(default = "default_export_format")]
    pub export_format: String,
    /// A JPG export's quality, 1 to 100 (`fractadyne_export::JPEG_QUALITY_DEFAULT`).
    #[serde(default = "default_export_jpeg_quality")]
    pub export_jpeg_quality: u8,
    /// An AVIF export's quality, 1 to 100 (`fractadyne_export::AVIF_QUALITY_DEFAULT`).
    #[serde(default = "default_export_avif_quality")]
    pub export_avif_quality: u8,
    /// A GIF export's palette size, 2 to 256, and whether it is dithered.
    #[serde(default = "default_export_gif_colors")]
    pub export_gif_colors: u16,
    #[serde(default = "default_true")]
    pub export_gif_dither: bool,
    /// What the Snapshot button (Ctrl+S) does: `"ask"` (the default — the first press offers the
    /// choice), `"screen"` (save the view exactly as shown) or `"render"` (a full export at the
    /// export settings, in the background). Set by the first-press dialog when the user asks not
    /// to be asked again; changeable in File ▸ Settings ▸ Snapshot.
    #[serde(default = "default_snapshot_mode")]
    pub snapshot_mode: String,
    /// The accelerated build's "MPFR libraries not found" warning was dismissed with "don't show
    /// again". Only the GUI dialog is suppressed; the fallback, the log line and the stderr message
    /// are unconditional. Default false; `serde(default)` keeps older files loadable.
    #[serde(default)]
    pub mpfr_warning_suppressed: bool,
    /// Accelerated build: whether the MPFR libraries were present on the previous run (`None` until
    /// first recorded). Used to detect a state CHANGE across launches — appearing or disappearing —
    /// which is announced even when the recurring warning is silenced. Only the accelerated build
    /// writes it; the standard build preserves it.
    #[serde(default)]
    pub mpfr_present_last: Option<bool>,
    /// Last directory an export was saved to; `None` until the first export.
    #[serde(default)]
    pub export_dir: Option<String>,
    /// Last directory any open/save file dialog landed in — the shared fallback so a dialog opens
    /// where the user last browsed, across categories and across restarts. `None` on a fresh
    /// install (dialogs then fall back to a sensible per-category default like Pictures).
    #[serde(default)]
    pub last_dir: Option<String>,
    /// Path of the last camera-tour script played, so the toolbar button and menu can default to
    /// it. `None` until the first script is played; a missing file falls back to the picker.
    #[serde(default)]
    pub last_script: Option<String>,
    /// Whether the first-run welcome overlay has been dismissed. `false` (the default, and the
    /// state of a fresh install with no session file) shows the overlay once on launch.
    #[serde(default)]
    pub welcome_seen: bool,
    /// Dual-view export layout: "side" | "separate" | "active".
    #[serde(default = "default_export_dual_mode")]
    pub export_dual_mode: String,
    /// Export aspect ratio: "window" (match the live view) or a fixed ratio key ("16:9", "1:1", …).
    #[serde(default = "default_export_aspect")]
    pub export_aspect: String,
    /// Embed a thumbnail when saving or sharing a view. See `ShareDialog::include_thumb`.
    #[serde(default = "default_true")]
    pub share_include_thumb: bool,

    /// Open the finished image in the system viewer when an export succeeds.
    /// ⚠Defaults OFF: an export can be a batch of one among many, and a viewer window
    /// stealing focus mid-session is not something to opt people into silently.
    #[serde(default)]
    pub export_open_after: bool,
    /// Render exports on every graphics card of this machine (Export ▸ Use all graphics cards):
    /// the image split by tile across them. Off by default. `serde(default)` keeps older session
    /// files loadable.
    #[serde(default)]
    pub export_all_gpus: bool,
    /// Burn the zoom/coordinate HUD into exports (also settable via the `--show-location` CLI flag).
    #[serde(default)]
    pub show_location: bool,
    /// Palette animation mode: "off" | "forward" | "reverse" | "pingpong".
    #[serde(default = "default_palette_anim")]
    pub palette_anim: String,
    /// Palette animation speed (offset cycles per second).
    #[serde(default = "default_palette_anim_speed")]
    pub palette_anim_speed: f32,
    /// Rotate the relief light direction over time.
    #[serde(default)]
    pub light_anim: bool,
    /// Cycle the sine wave trap's phase over time (the palette's Speed): each frame iterates again.
    #[serde(default)]
    pub trap_phase_anim: bool,
    #[serde(default)]
    pub de_anim: bool,
    /// Show the minimap overview ("you are here").
    #[serde(default)]
    pub minimap: bool,
    /// Whether the right-hand control panel is shown.
    #[serde(default = "default_true")]
    pub right_panel_open: bool,
    /// Whether the Performance section is shown at the bottom of the control panel.
    ///
    /// ⭐**Off by default, and persisted so turning it on is a decision you make once.** Its
    /// readouts (`mode perturb df32`, `precision 84 bit`, `orbit len 12833`) are diagnostics: they
    /// read as an error report to someone who has just opened a fractal viewer, and they were the
    /// largest block on a first-run panel. ⚠It also gates a periodic repaint that refreshes them,
    /// so leaving it off lets an idle deep view actually idle.
    ///
    /// ⚠**This is the only control-panel section whose state survives a restart.** eframe is built
    /// without `persistence`, so egui throws away every `CollapsingHeader`'s open/closed state on
    /// exit — the other sections reopen at their `default_open` every launch. This one is persisted
    /// by the app because turning the panel off and having it come back is the specific complaint.
    #[serde(default)]
    pub perf_panel: bool,
    /// Active fractal family (name, e.g. "Mandelbrot", "Burning Ship") — so the view you
    /// left is fully restored (the center/zoom already are).
    #[serde(default = "default_fractal")]
    pub fractal: String,
    /// The custom formula's source text (what `fractal = "Custom"` renders) and its parameters
    /// p1…p5 as `[re, im]` — empty until one has been written. Kept when another family is chosen,
    /// so switching back finds it.
    #[serde(default)]
    pub custom_formula: String,
    #[serde(default)]
    pub custom_params: Vec<[f64; 2]>,
    /// The formula editor shows the formula typeset (Textbook) rather than as text. Off by default,
    /// so a session from before the toggle opens the editor as it always did.
    #[serde(default)]
    pub formula_textbook: bool,
    /// The Life universe (design/automata.md), as the lines a Life view file carries (`rule=`,
    /// `pattern=`, `pattern_origin=`, `pattern_generation=`, `generation=`, `pattern_name=`). Kept
    /// whichever family is shown, so switching back finds it; empty in a session from before Life.
    #[serde(default)]
    pub life: String,
    /// The L-system (design/lsystems.md), as the lines an L-system view file carries (`lsystem=`,
    /// `lsystem_order=`, …). Kept whichever family is shown; empty in a session from before
    /// L-systems.
    #[serde(default)]
    pub lsystem: String,
    /// Julia mode + parameter `c` (the view state that pairs with center/zoom).
    #[serde(default)]
    pub julia_mode: bool,
    #[serde(default = "default_julia_c_re")]
    pub julia_c_re: f64,
    #[serde(default = "default_julia_c_im")]
    pub julia_c_im: f64,
    /// Dual (Mandelbrot ↔ Julia) view.
    #[serde(default)]
    pub dual: bool,
    /// Dual-view split position as a fraction of the width (draggable separator). Default 0.5.
    #[serde(default = "default_dual_split")]
    pub dual_split: f32,
    /// The dual view's Julia pane camera, which pans and zooms on its own: its center at full
    /// precision and its scale as `log2(units per pixel)`. Empty / `None` = the pane's home view, as
    /// in a session from before it was kept (2026-11-16: a restart reset the pane, review finding 8).
    #[serde(default)]
    pub julia_view_center_re: String,
    #[serde(default)]
    pub julia_view_center_im: String,
    #[serde(default)]
    pub julia_view_upp_log2: Option<f64>,
    /// Series approximation (iteration-skipping) preference. Default on.
    #[serde(default = "default_true")]
    pub series_approx: bool,
    /// Multi-reference glitch correction for exports (single + dual, up to ~32 MP / the texture
    /// limit, non-aux coloring). Detects perturbation glitches and re-renders them against extra
    /// references until clean. Default on — glitch-free shared images out of the box.
    #[serde(default = "default_true")]
    pub glitch_correct: bool,
    /// BLA (bilinear approximation) acceleration for deep floatexp Mandelbrot. Default on — it
    /// skips iterations throughout the orbit (measured ~5× faster GPU render at 1e30×) and the
    /// tree is cached per reference, so its build cost is one-time like the reference orbit.
    #[serde(default = "default_true")]
    pub use_bla: bool,
    /// Draw the discreet "Fd" brand mark in the lower-right of the live view and exports.
    #[serde(default = "default_true")]
    pub watermark: bool,
    /// UI scale (egui zoom factor) — scales the interface fonts + widgets. 1.0 = default.
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,
    /// UI theme: `"dark"` (default) or `"light"`.
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Update-check track: `"stable"` (default) or `"beta"` (include pre-releases). The app moves
    /// a session that came back as DEFAULTS onto `"beta"` when the running build is itself a
    /// pre-release (`update::default_track_for`); a saved value is never touched.
    #[serde(default = "default_update_track")]
    pub update_track: String,
    /// Automatically check for updates on launch (off by default — no network calls unless enabled).
    #[serde(default)]
    pub update_check_on_launch: bool,
    /// The interface's language, a BCP 47 tag (`de`, `zh-Hans`): set on the first run from the
    /// system's languages, and by the language menu. Empty = not chosen yet, i.e. the first run
    /// (design/multilingual.md §8).
    #[serde(default)]
    pub language: String,
    /// The system's preferred language found at the last launch. When a launch finds another one
    /// that the app has, it asks once whether to switch.
    #[serde(default)]
    pub language_system_seen: String,
    /// The decimal mark numbers take: `"."` or `","`, or empty for the language's own.
    #[serde(default)]
    pub decimal_mark: String,
    /// Draw the "Fd" brand mark on the live view and exports. Default ON; the first-run overlay
    /// offers the opt-out, because someone publishing frames wants that choice up front rather than
    /// after discovering it in a menu.
    #[serde(default = "default_true")]
    pub show_watermark: bool,
    /// Stop offering to send a report after an unclean shutdown ("Don't ask again").
    #[serde(default)]
    pub crash_prompt_disabled: bool,
    /// Interactive orbit overlay: shown, normalized-to-view, animated (racing dot), and its
    /// animation speed.
    #[serde(default)]
    pub show_orbits: bool,
    #[serde(default)]
    pub orbit_normalize: bool,
    #[serde(default)]
    pub orbit_anim: bool,
    #[serde(default = "default_orbit_anim_speed")]
    pub orbit_anim_speed: f32,
}

fn default_duotone_lo() -> [f32; 3] {
    [0.04, 0.05, 0.12] // deep navy
}

fn default_duotone_hi() -> [f32; 3] {
    [0.95, 0.80, 0.45] // warm cream
}

fn default_dual_split() -> f32 {
    0.5
}

fn default_autopilot_dive_log2() -> f64 {
    900.0 // ≈ 1e271×
}

fn default_autopilot_priority() -> String {
    "speed".to_string()
}

fn default_autopilot_target() -> String {
    "detail".to_string()
}

fn default_update_track() -> String {
    "stable".to_string()
}

fn default_min_motion_res() -> f32 {
    0.30
}

fn default_work_budget_scale() -> f64 {
    1.0
}

fn default_zoom_rate() -> f32 {
    1.0
}

fn default_click_zoom_factor() -> f32 {
    10.0
}

fn default_snap_zoom() -> bool {
    true
}

fn default_finish_sound() -> bool {
    true
}
fn default_live_panel_opacity() -> f32 {
    0.95
}

fn default_aa() -> u32 {
    2
}

fn default_fps_cap() -> f64 {
    60.0
}

fn default_orbit_cache_mb() -> u32 {
    1024
}

fn default_fractal() -> String {
    "Mandelbrot".to_string()
}

fn default_julia_c_re() -> f64 {
    -0.8
}

fn default_julia_c_im() -> f64 {
    0.156
}

fn default_orbit_anim_speed() -> f32 {
    10.0
}

fn default_ui_scale() -> f32 {
    1.0
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_true() -> bool {
    true
}

fn default_export_width() -> u32 {
    3840
}

fn default_export_ss() -> u32 {
    2
}

fn default_snapshot_mode() -> String {
    "ask".to_string()
}

fn default_export_format() -> String {
    "png".to_string()
}

fn default_export_jpeg_quality() -> u8 {
    90
}

fn default_export_avif_quality() -> u8 {
    85
}

fn default_export_gif_colors() -> u16 {
    256
}

fn default_export_dual_mode() -> String {
    "side".to_string()
}

fn default_export_aspect() -> String {
    "window".to_string()
}

fn default_palette_anim() -> String {
    "off".to_string()
}

fn default_palette_anim_speed() -> f32 {
    0.15
}

fn default_light_angle() -> f32 {
    2.2 // radians (~126°), light from upper-left
}

fn default_light_height() -> f32 {
    1.2
}

fn default_light_steep_px() -> f32 {
    8.0
}

fn default_light_highlight() -> f32 {
    0.5
}

fn default_light_shininess() -> f32 {
    64.0
}

fn default_light_highlight_color() -> [f32; 3] {
    [1.0; 3]
}

fn default_de_strength() -> f32 {
    0.6
}

fn default_de_width() -> f32 {
    1.0
}

fn default_edge_width() -> f32 {
    2.0
}

fn default_edge_strength() -> f32 {
    1.0
}

fn default_edge_curve() -> String {
    "linear".to_string()
}

fn default_equipotential_spacing() -> f32 {
    16.0
}

fn default_equipotential_width() -> f32 {
    1.5
}

fn default_equipotential_strength() -> f32 {
    0.6
}

fn default_color_method() -> String {
    "smooth".to_string()
}

fn default_stripe_freq() -> f32 {
    6.0
}

fn default_stripe_tail_len() -> u32 {
    // Measured at 4.45e246× (2026-09-16): 16 gives the classic bold banding, 64 a subtler
    // version, 256+ is visually the full-orbit mean again — the stripe term of a chaotic orbit
    // averages to 0.5 ± 0.35/√N, so only a SHORT window keeps the last iterates' structure.
    16
}

fn default_trap_type() -> String {
    "point".to_string()
}

fn default_trap_size() -> f32 {
    1.0
}

/// `fractadyne_core::coloring::HARMONICS_DEFAULT` (this crate does not depend on core; the app's
/// tests hold the two equal).
pub const DEFAULT_TRAP_HARMONICS: u32 = 5;

fn default_trap_harmonics() -> u32 {
    DEFAULT_TRAP_HARMONICS
}

fn default_sine_harmonics() -> u32 {
    1
}

/// `fractadyne_core::coloring::TRAP_TAIL_DEFAULT` (the app's tests hold the two equal): at the
/// seahorse at 1e20x the line trap's bands thread the valley at 256 (spread 8.1, against 0.000 for
/// the whole orbit; 2026-11-26).
pub const DEFAULT_TRAP_TAIL_LEN: u32 = 256;

fn default_trap_tail_len() -> u32 {
    DEFAULT_TRAP_TAIL_LEN
}

impl Default for ColoringSpec {
    fn default() -> Self {
        Self {
            palette_idx: 0,
            palette_id: String::new(),
            cycle: 0.27,
            iter_div: default_iter_div(),
            offset: 0.1,
            log_palette: false,
            transfer: default_transfer(),
            wrap: default_wrap(),
            steps: 0,
            normalize_live: true,
            normalize_fit: false,
            light: false,
            light_angle: default_light_angle(),
            light_height: default_light_height(),
            light_steep: false,
            light_steep_px: default_light_steep_px(),
            light_highlight: default_light_highlight(),
            light_shininess: default_light_shininess(),
            light_highlight_color: default_light_highlight_color(),
            de: false,
            de_strength: default_de_strength(),
            de_width: default_de_width(),
            edge: false,
            edge_inside: false,
            edge_width: default_edge_width(),
            edge_strength: default_edge_strength(),
            edge_curve: default_edge_curve(),
            edge_color: [0.0; 3],
            equipotentials: false,
            equipotential_spacing: default_equipotential_spacing(),
            equipotential_width: default_equipotential_width(),
            equipotential_strength: default_equipotential_strength(),
            equipotential_color: [0.0; 3],
            color_method: default_color_method(),
            stripe_freq: default_stripe_freq(),
            stripe_tail: false,
            stripe_tail_len: default_stripe_tail_len(),
            trap_type: default_trap_type(),
            trap_x: 0.0,
            trap_y: 0.0,
            trap_rotation: 0.0,
            trap_size: default_trap_size(),
            interior_trap: String::new(),
            interior_trap_x: 0.0,
            interior_trap_y: 0.0,
            interior_trap_rotation: 0.0,
            interior_trap_size: default_trap_size(),
            trap_phase: 0.0,
            interior_trap_phase: 0.0,
            trap_harmonics: DEFAULT_TRAP_HARMONICS,
            interior_trap_harmonics: DEFAULT_TRAP_HARMONICS,
            trap_sine_harmonics: 1,
            interior_trap_sine_harmonics: 1,
            trap_outline: String::new(),
            trap_outline_name: String::new(),
            trap_tail: false,
            trap_tail_len: DEFAULT_TRAP_TAIL_LEN,
            custom_palette: Vec::new(),
            custom_palette_flat: false,
            custom_segments: Vec::new(),
            use_custom_palette: false,
            use_duotone: false,
            use_binary: false,
            duotone_lo: default_duotone_lo(),
            duotone_hi: default_duotone_hi(),
            interior: String::new(),
            interior_palette: String::new(),
            interior_color: default_interior_color(),
            interior_fit: false,
            palette_aa: true,
            palette_brightness: 0.0,
            palette_contrast: 0.0,
            palette_saturation: 1.0,
            julia_linked: false,
            equalize: 0.0,
            palette_oklab: false,
            coloring_hook: String::new(),
            coloring_hook_params: Vec::new(),
        }
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            state_version: STATE_FORMAT_VERSION,
            center_x: -0.5,
            center_y: 0.0,
            center_x_str: String::new(),
            center_y_str: String::new(),
            center_expr: String::new(),
            units_per_pixel: 3.0 / 720.0,
            units_per_pixel_e: 0,
            max_iter: 256,
            auto_iter: true,
            coloring: ColoringSpec::default(),
            coloring_hook_path: String::new(),
            zoom_rate: default_zoom_rate(),
            click_zoom: false,
            click_zoom_factor: default_click_zoom_factor(),
            click_zoom_snap: false,
            snap_zoom: default_snap_zoom(),
            autopilot_dive_log2: default_autopilot_dive_log2(),
            autopilot_priority: default_autopilot_priority(),
            autopilot_target: default_autopilot_target(),
            work_budget_scale: default_work_budget_scale(),
            min_motion_res: default_min_motion_res(),
            prefer_detail: false,
            live_worker_gpu: String::new(),
            show_timestamp: false,
            show_zoom_target: false,
            live_panel: String::new(),
            live_panel_corner: String::new(),
            live_panel_opacity: default_live_panel_opacity(),
            aa: default_aa(),
            finish_sound: default_finish_sound(),
            fps_cap: default_fps_cap(), // 60 (0 = uncapped)
            orbit_cache_mb: default_orbit_cache_mb(),
            export_width: default_export_width(),
            export_ss: default_export_ss(),
            export_format: default_export_format(),
            export_jpeg_quality: default_export_jpeg_quality(),
            export_avif_quality: default_export_avif_quality(),
            export_gif_colors: default_export_gif_colors(),
            export_gif_dither: true,
            snapshot_mode: default_snapshot_mode(),
            mpfr_warning_suppressed: false,
            mpfr_present_last: None,
            export_dir: None,
            last_dir: None,
            last_script: None,
            welcome_seen: false,
            export_dual_mode: default_export_dual_mode(),
            export_aspect: default_export_aspect(),
            share_include_thumb: true,
            export_open_after: false,
            export_all_gpus: false,
            show_location: false,
            palette_anim: default_palette_anim(),
            palette_anim_speed: default_palette_anim_speed(),
            light_anim: false,
            trap_phase_anim: false,
            de_anim: false,
            minimap: false,
            right_panel_open: true,
            perf_panel: false,
            fractal: default_fractal(),
            custom_formula: String::new(),
            custom_params: Vec::new(),
            formula_textbook: false,
            life: String::new(),
            lsystem: String::new(),
            julia_mode: false,
            julia_c_re: default_julia_c_re(),
            julia_c_im: default_julia_c_im(),
            dual: false,
            dual_split: default_dual_split(),
            julia_view_center_re: String::new(),
            julia_view_center_im: String::new(),
            julia_view_upp_log2: None,
            series_approx: true,
            glitch_correct: true,
            use_bla: true,
            watermark: true,
            ui_scale: default_ui_scale(),
            theme: default_theme(),
            update_track: default_update_track(),
            update_check_on_launch: false,
            language: String::new(),
            language_system_seen: String::new(),
            decimal_mark: String::new(),
            show_watermark: true,
            crash_prompt_disabled: false,
            show_orbits: false,
            orbit_normalize: false,
            orbit_anim: false,
            orbit_anim_speed: default_orbit_anim_speed(),
        }
    }
}

/// The config directory that holds all persisted state (session, bookmarks, thumbnails).
///
/// Normally the OS per-user config dir. The `FRACTADYNE_CONFIG_DIR` environment variable
/// overrides it — useful for sandboxing (tests, CI, portable installs) and, importantly, so the
/// destructive [`reset_all`] can be exercised against a throwaway directory instead of your real
/// data. (On Windows the `directories` crate resolves the default via the Win32 known-folder API,
/// which ignores `APPDATA`, so this explicit override is the only reliable sandbox.)
pub fn config_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("FRACTADYNE_CONFIG_DIR") {
        if !dir.is_empty() {
            return Some(PathBuf::from(dir));
        }
    }
    directories::ProjectDirs::from("com", "Fractadyne", "Fractadyne")
        .map(|d| d.config_dir().to_path_buf())
}

fn state_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("session.toml"))
}

/// Whether this profile already holds a saved session — i.e. the app has run here before.
///
/// A harness needs it to tell "the previous session exited cleanly" from "there was no previous
/// session", which are different facts and only one of them is evidence.
pub fn has_saved_session() -> bool {
    state_path().is_some_and(|p| p.is_file())
}

/// Human-readable path where application state is stored (for Help / warnings). Returns a
/// placeholder if the OS config directory can't be determined.
pub fn state_location_display() -> String {
    config_dir()
        .map(|d| d.display().to_string())
        .unwrap_or_else(|| "(could not determine the OS config directory)".to_string())
}

/// Outcome of [`load_with_status`], so the caller can warn when a file it can't fully
/// account for was loaded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateLoad {
    /// No saved state (fresh install / after a reset) — defaults returned.
    Fresh,
    /// A session file EXISTS but does not parse, so defaults were returned instead — the settings
    /// in that file had no effect at all. Split out from [`StateLoad::Fresh`] because the two are
    /// indistinguishable to a caller that only sees the state, and for a HARNESS that stages a
    /// session (the corpus generator) they mean opposite things: "fresh" is the intended sandbox,
    /// "unreadable" is a silently-ignored staging and a wrong render. Not user-facing on its own —
    /// an old/corrupt file is a normal thing to shrug off — but it is logged.
    Unreadable,
    /// Loaded and the schema version is understood.
    Ok,
    /// Loaded best-effort, but the file was written by a **newer** Fractadyne
    /// (its `state_version`, which exceeds [`STATE_FORMAT_VERSION`]) — some settings may
    /// not apply, and re-saving will downgrade the file to this build's format.
    Newer(u32),
    /// Loaded, but some of it did not read: these keys (as the file names them) went back to their
    /// defaults, and every other setting in the file was kept. Empty when every key read but the
    /// file was not UTF-8 (its unreadable bytes are replaced). Before 2026-11-16 one such value
    /// made the whole file [`StateLoad::Unreadable`], and the next save replaced every setting.
    Partial(Vec<String>),
}

impl StateLoad {
    /// Whether the file on disk holds something this build did not take in, so it must be kept as
    /// it was before a save replaces it ([`load_for_app`]).
    pub fn file_needs_keeping(&self) -> bool {
        matches!(self, StateLoad::Unreadable | StateLoad::Newer(_) | StateLoad::Partial(_))
    }
}

/// Load the saved session together with a status describing whether this build can fully
/// account for it. Missing ⇒ defaults + `Fresh`; a file that does not parse whole keeps every
/// setting that reads (`Partial`); a newer-version file loads best-effort with `Newer(v)` so the
/// app can warn. Reads only: [`load_for_app`] also keeps a copy of a file it could not take in.
pub fn load_with_status() -> (SessionState, StateLoad) {
    let Some(bytes) = state_path().and_then(|p| std::fs::read(p).ok()) else {
        return (SessionState::default(), StateLoad::Fresh);
    };
    // ⚠Not `read_to_string`: a file with one byte that is not UTF-8 (an edit in a program that
    // saves in the system's code page) read as NO file, and the next save replaced it.
    let (text, lossy) = durable::text_of(&bytes);
    match parse_with_status(&text) {
        (s, StateLoad::Ok) if lossy => (s, StateLoad::Partial(Vec::new())),
        other => other,
    }
}

/// [`load_with_status`] for the app that will save over the file: when the file holds something
/// this build did not take in (unreadable, partly read, or from a newer build), a copy of it is
/// kept first (`session.unreadable.toml`, or `session.newer.toml`), and its path is returned so the
/// user can be told where it is.
pub fn load_for_app() -> (SessionState, StateLoad, Option<PathBuf>) {
    let (s, status) = load_with_status();
    let kept = if status.file_needs_keeping() {
        let tag = if matches!(status, StateLoad::Newer(_)) { "newer" } else { "unreadable" };
        state_path().and_then(|p| durable::keep_copy(&p, tag).ok())
    } else {
        None
    };
    (s, status, kept)
}

/// Parse persisted-session TOML into a state + status. Split out from [`load_with_status`] (which
/// supplies the file contents) so the version/corruption handling is unit-testable; public for the
/// app's self-test `round-trip`, which reads a session back without touching the file.
pub fn parse_with_status(text: &str) -> (SessionState, StateLoad) {
    // Probe just the version first, so a newer file that no longer fully parses still warns
    // rather than silently reverting to defaults.
    #[derive(Deserialize)]
    struct Probe {
        #[serde(default = "default_state_version")]
        state_version: u32,
    }
    let probed = toml::from_str::<Probe>(text)
        .map(|p| p.state_version)
        .unwrap_or(STATE_FORMAT_VERSION);
    match toml::from_str::<SessionState>(text) {
        Ok(s) if s.state_version <= STATE_FORMAT_VERSION => (s, StateLoad::Ok),
        Ok(s) => {
            let v = s.state_version;
            (s, StateLoad::Newer(v))
        }
        Err(_) => match salvage(text) {
            // A TOML document with values that do not read: every key that reads is kept.
            Some((s, _)) if s.state_version > STATE_FORMAT_VERSION || probed > STATE_FORMAT_VERSION => {
                let v = s.state_version.max(probed);
                (s, StateLoad::Newer(v))
            }
            Some((s, dropped)) => (s, StateLoad::Partial(dropped)),
            // Not TOML at all: if the version says it's from the future, warn; otherwise defaults.
            None if probed > STATE_FORMAT_VERSION => (SessionState::default(), StateLoad::Newer(probed)),
            None => (SessionState::default(), StateLoad::Unreadable),
        },
    }
}

/// The session in a TOML document that does not deserialize whole: starting from the defaults,
/// each of the file's keys is taken if the session still reads with it, and named in the second
/// value if not. `None` when the text is not a TOML document at all.
///
/// ⭐Key by key against the DEFAULTS, so a value of the wrong type (`aa = "three"`), a value out
/// of a field's range (`blend = 300` in a gradient segment) or a missing required key costs that
/// key alone. Before 2026-11-16 any of them cost the whole file (review finding 1).
fn salvage(text: &str) -> Option<(SessionState, Vec<String>)> {
    let file: toml::Table = text.parse().ok()?;
    let toml::Value::Table(mut kept) = toml::Value::try_from(SessionState::default()).ok()? else {
        return None;
    };
    let reads = |t: &toml::Table| toml::Value::Table(t.clone()).try_into::<SessionState>().is_ok();
    // A key the session requires (one without `serde(default)`) that the file lacks is named too:
    // it is as much a setting that did not come back as one that did not read, and a harness that
    // stages a session must be able to tell (`partial_session_is_named_not_silently_default`).
    let mut dropped: Vec<String> = kept
        .keys()
        .filter(|k| !file.contains_key(*k))
        .filter(|k| {
            let mut t = kept.clone();
            t.remove(*k);
            !reads(&t)
        })
        .cloned()
        .collect();
    for (key, value) in file {
        let mut candidate = kept.clone();
        candidate.insert(key.clone(), value);
        if reads(&candidate) {
            kept = candidate;
        } else {
            dropped.push(key);
        }
    }
    let s = toml::Value::Table(kept).try_into::<SessionState>().ok()?;
    Some((s, dropped))
}

/// Load the saved session, or [`SessionState::default`] if none/unreadable.
pub fn load() -> SessionState {
    load_with_status().0
}

/// Permanently delete **all** persisted application state — session, bookmarks, and cached
/// bookmark thumbnails — by removing the config directory. Returns `Ok(true)` if state was
/// removed, `Ok(false)` if there was nothing to remove.
pub fn reset_all() -> std::io::Result<bool> {
    match config_dir() {
        Some(dir) if dir.exists() => {
            std::fs::remove_dir_all(&dir)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Save the session atomically ([`durable::write_atomic`]: a temp file named for this process,
/// flushed, then renamed over the old one). The error is returned for the app to log: persistence
/// must never disrupt the app, but a session that silently stops saving is a lost setting nobody
/// can explain.
///
/// ⚠The temp file used to be `session.toml.tmp` for every running copy of the app, so two copies
/// saving at once could write into each other's temp file and rename a mix of both into place.
pub fn save(state: &SessionState) -> std::io::Result<()> {
    let Some(path) = state_path() else { return Ok(()) };
    let text = toml::to_string_pretty(state).map_err(std::io::Error::other)?;
    durable::write_atomic(&path, text.as_bytes())
}

#[cfg(test)]
#[path = "../tests/unit/tests.rs"]
mod tests;
