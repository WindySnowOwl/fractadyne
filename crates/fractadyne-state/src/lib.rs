//! App state & persistence (DESIGN.md §13).
//!
//! M1: serialize the session (location, zoom, coloring) to a human-readable TOML
//! file in the OS config dir, written atomically (temp + rename). Bookmarks /
//! presets library and arbitrary-precision coordinates come later.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// On-disk schema version of the persisted session. Bump this when a change could make an
/// **older** build misread a **newer** file (e.g. a field's meaning changes). The schema is
/// otherwise additive — new fields use `#[serde(default)]`, so a newer file still loads
/// best-effort on an older build; the version only drives the "saved by a newer Fractadyne"
/// warning. A file with no `state_version` (written before versioning) is treated as v1.
pub const STATE_FORMAT_VERSION: u32 = 1;

fn default_state_version() -> u32 {
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
    /// Scale mantissa. Paired with `units_per_pixel_e` (a base-2 exponent) so deep-zoom
    /// locations past `f64`'s ~1e308× range survive quit/restart. Old saves (no exponent
    /// field) stored the full `f64` value here with an implicit exponent of 0 — still valid.
    pub units_per_pixel: f64,
    #[serde(default)]
    pub units_per_pixel_e: i32,
    pub max_iter: u32,
    pub auto_iter: bool,
    pub palette_idx: usize,
    pub cycle: f32,
    pub offset: f32,
    /// Log-scaled palette mapping (default off) — applies wherever normalization is active.
    /// `serde(default)` keeps older session files loadable.
    #[serde(default)]
    pub log_palette: bool,
    /// Live deep-palette auto-normalization (default on). `serde(default)` keeps older files loadable.
    #[serde(default = "default_true")]
    pub normalize_live: bool,
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
    /// Auto-zoom (autopilot) dive limit as log2(magnification); the depth at which the hands-free
    /// dive stops. Default 900 (≈1e271×). Past the smooth regime the autopilot switches to a
    /// stepped dive to reach this depth.
    #[serde(default = "default_autopilot_dive_log2")]
    pub autopilot_dive_log2: f64,
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
    /// Last-used export settings (remembered across sessions).
    #[serde(default = "default_export_width")]
    pub export_width: u32,
    #[serde(default = "default_export_ss")]
    pub export_ss: u32,
    #[serde(default = "default_export_format")]
    pub export_format: String,
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
    /// Burn the zoom/coordinate HUD into exports (also settable via the `--show-location` CLI flag).
    #[serde(default)]
    pub show_location: bool,
    /// Palette animation mode: "off" | "forward" | "reverse" | "pingpong".
    #[serde(default = "default_palette_anim")]
    pub palette_anim: String,
    /// Palette animation speed (offset cycles per second).
    #[serde(default = "default_palette_anim_speed")]
    pub palette_anim_speed: f32,
    /// Distance-estimate relief lighting: enabled, light angle (rad), relief strength.
    #[serde(default)]
    pub light: bool,
    #[serde(default = "default_light_angle")]
    pub light_angle: f32,
    #[serde(default = "default_light_height")]
    pub light_height: f32,
    /// Rotate the relief light direction over time.
    #[serde(default)]
    pub light_anim: bool,
    /// Distance-estimate glow: enabled, blend strength, contour width, animate flag.
    #[serde(default)]
    pub de: bool,
    #[serde(default = "default_de_strength")]
    pub de_strength: f32,
    #[serde(default = "default_de_width")]
    pub de_width: f32,
    #[serde(default)]
    pub de_anim: bool,
    /// Coloring method: "smooth" | "stripe" | "triangle" | "trap" | "distance" |
    /// "decomposition".
    #[serde(default = "default_color_method")]
    pub color_method: String,
    /// Stripe-average angular frequency (method = "stripe").
    #[serde(default = "default_stripe_freq")]
    pub stripe_freq: f32,
    /// Orbit-trap shape: "point" | "cross" | "circle" (method = "trap").
    #[serde(default = "default_trap_type")]
    pub trap_type: String,
    /// Show the minimap overview ("you are here").
    #[serde(default)]
    pub minimap: bool,
    /// Custom gradient stops `[pos, r, g, b]` (DISPLAY-space RGB) from the gradient editor.
    #[serde(default)]
    pub custom_palette: Vec<[f32; 4]>,
    /// Read `custom_palette` as **bands** (one flat colour per entry, no interpolation) rather
    /// than as gradient stops — Fractint `.map` semantics, set by importing one. `false` for
    /// every session written before palette import existed, which is the right reading of an
    /// editor-authored gradient.
    #[serde(default)]
    pub custom_palette_flat: bool,
    /// A full SEGMENT gradient, when the palette is one a stop list cannot express.
    ///
    /// ⭐**This exists because `.ggr` is a superset of `custom_palette`.** A GIMP gradient carries a
    /// per-segment midpoint, one of five blend curves and a colour space (RGB or either way round
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
    /// Whether the right-hand control panel is shown.
    #[serde(default = "default_true")]
    pub right_panel_open: bool,
    /// Active fractal family (name, e.g. "Mandelbrot", "Burning Ship") — so the view you
    /// left is fully restored (the center/zoom already are).
    #[serde(default = "default_fractal")]
    pub fractal: String,
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
    /// Update-check track: `"stable"` (default) or `"beta"` (include pre-releases).
    #[serde(default = "default_update_track")]
    pub update_track: String,
    /// Automatically check for updates on launch (off by default — no network calls unless enabled).
    #[serde(default)]
    pub update_check_on_launch: bool,
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

fn default_true() -> bool {
    true
}

fn default_dual_split() -> f32 {
    0.5
}

fn default_autopilot_dive_log2() -> f64 {
    900.0 // ≈ 1e271×
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

fn default_finish_sound() -> bool {
    true
}
fn default_aa() -> u32 {
    2
}

fn default_fps_cap() -> f64 {
    60.0
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

fn default_export_width() -> u32 {
    3840
}

fn default_export_ss() -> u32 {
    2
}

fn default_export_format() -> String {
    "png".to_string()
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

fn default_de_strength() -> f32 {
    0.6
}

fn default_de_width() -> f32 {
    1.0
}

fn default_color_method() -> String {
    "smooth".to_string()
}

fn default_stripe_freq() -> f32 {
    6.0
}

fn default_trap_type() -> String {
    "point".to_string()
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            state_version: STATE_FORMAT_VERSION,
            center_x: -0.5,
            center_y: 0.0,
            center_x_str: String::new(),
            center_y_str: String::new(),
            units_per_pixel: 3.0 / 720.0,
            units_per_pixel_e: 0,
            max_iter: 256,
            auto_iter: true,
            palette_idx: 0,
            cycle: 0.27,
            log_palette: false,
            normalize_live: true,
            offset: 0.1,
            zoom_rate: default_zoom_rate(),
            click_zoom: false,
            click_zoom_factor: default_click_zoom_factor(),
            autopilot_dive_log2: default_autopilot_dive_log2(),
            work_budget_scale: default_work_budget_scale(),
            min_motion_res: default_min_motion_res(),
            prefer_detail: false,
            aa: default_aa(),
            finish_sound: default_finish_sound(),
            fps_cap: default_fps_cap(), // 60 (0 = uncapped)
            export_width: default_export_width(),
            export_ss: default_export_ss(),
            export_format: default_export_format(),
            export_dir: None,
            last_dir: None,
            last_script: None,
            welcome_seen: false,
            export_dual_mode: default_export_dual_mode(),
            export_aspect: default_export_aspect(),
            show_location: false,
            palette_anim: default_palette_anim(),
            palette_anim_speed: default_palette_anim_speed(),
            light: false,
            light_angle: default_light_angle(),
            light_height: default_light_height(),
            light_anim: false,
            de: false,
            de_strength: default_de_strength(),
            de_width: default_de_width(),
            de_anim: false,
            color_method: default_color_method(),
            stripe_freq: default_stripe_freq(),
            trap_type: default_trap_type(),
            minimap: false,
            custom_palette: Vec::new(),
            custom_palette_flat: false,
            custom_segments: Vec::new(),
            use_custom_palette: false,
            use_duotone: false,
            use_binary: false,
            duotone_lo: default_duotone_lo(),
            duotone_hi: default_duotone_hi(),
            right_panel_open: true,
            fractal: default_fractal(),
            julia_mode: false,
            julia_c_re: default_julia_c_re(),
            julia_c_im: default_julia_c_im(),
            dual: false,
            dual_split: default_dual_split(),
            series_approx: true,
            glitch_correct: true,
            use_bla: true,
            watermark: true,
            ui_scale: default_ui_scale(),
            theme: default_theme(),
            update_track: default_update_track(),
            update_check_on_launch: false,
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
}

/// Load the saved session together with a status describing whether this build can fully
/// account for it. Missing/corrupt ⇒ defaults + `Fresh`; a newer-version file loads
/// best-effort with `Newer(v)` so the app can warn.
pub fn load_with_status() -> (SessionState, StateLoad) {
    match state_path().and_then(|p| std::fs::read_to_string(p).ok()) {
        Some(text) => parse_with_status(&text),
        None => (SessionState::default(), StateLoad::Fresh),
    }
}

/// Parse persisted-session TOML into a state + status. Split out from [`load_with_status`] (which
/// supplies the file contents) so the version/corruption handling is unit-testable.
fn parse_with_status(text: &str) -> (SessionState, StateLoad) {
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
        // Unparseable: if the version says it's from the future, warn; otherwise treat as a
        // legacy/corrupt file and quietly fall back to defaults (no false alarm on old saves).
        Err(_) if probed > STATE_FORMAT_VERSION => (SessionState::default(), StateLoad::Newer(probed)),
        Err(_) => (SessionState::default(), StateLoad::Unreadable),
    }
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

/// Save the session atomically (write temp, then rename). Best-effort: ignores
/// errors so persistence never disrupts the app.
pub fn save(state: &SessionState) {
    let Some(path) = state_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let Ok(text) = toml::to_string_pretty(state) else { return };
    let tmp = path.with_extension("toml.tmp");
    if std::fs::write(&tmp, text).is_ok() {
        let _ = std::fs::rename(&tmp, &path);
    }
}

#[cfg(test)]
mod tests;
