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
    pub units_per_pixel: f64,
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
    /// Frame-rate cap in FPS; `None` = uncapped. Defaults to 60.
    #[serde(default = "default_fps_cap")]
    pub fps_cap: Option<f64>,
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

fn default_fps_cap() -> Option<f64> {
    Some(60.0)
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
            max_iter: 256,
            auto_iter: true,
            palette_idx: 0,
            cycle: 0.27,
            offset: 0.1,
            zoom_rate: default_zoom_rate(),
            aa: default_aa(),
            fps_cap: default_fps_cap(),
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
