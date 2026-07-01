//! App state & persistence (DESIGN.md §13).
//!
//! M1: serialize the session (location, zoom, coloring) to a human-readable TOML
//! file in the OS config dir, written atomically (temp + rename). Bookmarks /
//! presets library and arbitrary-precision coordinates come later.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Persisted session. The center is stored at **full precision** as decimal strings
/// (`center_x_str`/`center_y_str`) so deep-zoom locations survive quit/restart; the
/// `f64` `center_x`/`center_y` remain for display and backward compatibility with
/// older session files (used as a fallback when the strings are absent).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SessionState {
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
    /// Continuous-zoom speed multiplier (1.0 = default ~2× per 1.5 s). `serde(default)`
    /// keeps older session files (written before this field) loadable.
    #[serde(default = "default_zoom_rate")]
    pub zoom_rate: f32,
    /// Supersampling / anti-alias factor (1 = off, 2/3/4/8 = N×N).
    #[serde(default = "default_aa")]
    pub aa: u32,
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
    /// Dual-view export layout: "side" | "separate" | "active".
    #[serde(default = "default_export_dual_mode")]
    pub export_dual_mode: String,
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
    /// Custom gradient stops `[pos, r, g, b]` (linear RGB) from the gradient editor.
    #[serde(default)]
    pub custom_palette: Vec<[f32; 4]>,
    /// Use the custom gradient instead of the selected preset.
    #[serde(default)]
    pub use_custom_palette: bool,
    /// Duotone palette (two-color ramp) / binary palette (flat in-set vs out-of-set),
    /// sharing the two colors (linear RGB).
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
    /// Series approximation (iteration-skipping) preference. Default on.
    #[serde(default = "default_true")]
    pub series_approx: bool,
    /// Multi-reference glitch correction for exports (slower, glitch-free). Default off.
    #[serde(default)]
    pub glitch_correct: bool,
    /// BLA (bilinear approximation) acceleration for deep floatexp Mandelbrot. Default off
    /// (opt-in: it skips iterations throughout the orbit but rebuilds a tree per frame).
    #[serde(default)]
    pub use_bla: bool,
    /// UI scale (egui zoom factor) — scales the interface fonts + widgets. 1.0 = default.
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,
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

fn default_zoom_rate() -> f32 {
    1.0
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
            offset: 0.1,
            zoom_rate: default_zoom_rate(),
            aa: default_aa(),
            fps_cap: default_fps_cap(), // 60 (0 = uncapped)
            export_width: default_export_width(),
            export_ss: default_export_ss(),
            export_format: default_export_format(),
            export_dir: None,
            export_dual_mode: default_export_dual_mode(),
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
            series_approx: true,
            glitch_correct: false,
            use_bla: false,
            ui_scale: default_ui_scale(),
            show_orbits: false,
            orbit_normalize: false,
            orbit_anim: false,
            orbit_anim_speed: default_orbit_anim_speed(),
        }
    }
}

fn state_path() -> Option<PathBuf> {
    let dirs = directories::ProjectDirs::from("com", "Fractadyne", "Fractadyne")?;
    Some(dirs.config_dir().join("session.toml"))
}

/// Load the saved session, or [`SessionState::default`] if none/unreadable.
pub fn load() -> SessionState {
    state_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
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
mod tests {
    use super::*;

    fn roundtrip(s: &SessionState) -> SessionState {
        let text = toml::to_string_pretty(s).expect("serialize");
        toml::from_str(&text).expect("deserialize")
    }

    // The reported bug: the *uncapped* frame-rate (once `None`) was dropped by TOML and
    // reloaded as the default 60. Now stored as `0.0` (uncapped), it round-trips.
    #[test]
    fn fps_cap_roundtrips_including_uncapped() {
        let mut s = SessionState::default();
        s.fps_cap = 0.0; // uncapped
        assert_eq!(roundtrip(&s).fps_cap, 0.0);
        s.fps_cap = 120.0;
        assert_eq!(roundtrip(&s).fps_cap, 120.0);
    }

    // The view state (fractal family, Julia, dual) and the SA preference now persist.
    #[test]
    fn view_and_preference_fields_roundtrip() {
        let s = SessionState {
            fractal: "Burning Ship".to_string(),
            julia_mode: true,
            julia_c_re: 0.285,
            julia_c_im: 0.01,
            dual: true,
            series_approx: false,
            show_orbits: true,
            orbit_normalize: true,
            orbit_anim: true,
            orbit_anim_speed: 4.5,
            ui_scale: 1.25,
            ..SessionState::default()
        };
        let r = roundtrip(&s);
        assert_eq!(r.fractal, "Burning Ship");
        assert!(r.julia_mode && r.dual && !r.series_approx);
        assert_eq!(r.julia_c_re, 0.285);
        assert_eq!(r.julia_c_im, 0.01);
        assert!(r.show_orbits && r.orbit_normalize && r.orbit_anim);
        assert_eq!(r.orbit_anim_speed, 4.5);
        assert_eq!(r.ui_scale, 1.25);
    }

    // A legacy file (only the original required fields) must still load, filling new fields
    // from their defaults.
    #[test]
    fn legacy_file_loads_with_defaults() {
        let legacy = "center_x = -0.5\ncenter_y = 0.0\nunits_per_pixel = 0.004\n\
                      max_iter = 256\nauto_iter = true\npalette_idx = 0\ncycle = 0.27\noffset = 0.1\n";
        let s: SessionState = toml::from_str(legacy).expect("legacy load");
        assert_eq!(s.fps_cap, 60.0); // default cap
        assert_eq!(s.fractal, "Mandelbrot");
        assert!(s.series_approx);
    }
}
