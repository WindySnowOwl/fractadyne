//! The coloring a view carries (coloring plan step A1): the method and its settings, the relief
//! light, the distance glow, the Duotone and Binary palettes and the live normalization, as fields
//! of a `.fdn` file, a bookmark, a Share text and an exported image's embedded view. The palette,
//! the cycle, the offset, "Divide iterations" and the custom gradient travel already (`export.rs`).
//!
//! ⭐**A view carries what its picture depends on, and nothing else.** A key is written only when
//! its setting changes the picture and differs from the default, so an ordinary Smooth view is
//! written byte for byte as before these keys existed, and an older build opening it has nothing
//! to report. (An older build opening a view that does carry them names them as unknown fields
//! and keeps its own coloring, as it does for any newer key.)
//!
//! ⭐**Reading follows `iter_div`.** A file that states its coloring (`cycle=`, which every view
//! has carried from the start) and leaves a key out was written at that key's default, so the
//! setting returns to the default instead of keeping this session's. A file that states no
//! coloring (a hand-written location) leaves it all alone. A setting the file's picture does not
//! depend on (the stripe density of a Smooth view, the light's angle with the light off, the two
//! colors outside Duotone and Binary) is neither written nor reset, so opening a view does not undo
//! what was tuned for another method.
//!
//! ⚠A view saved before these keys existed states its coloring and carries none of them, so it
//! opens in Smooth with the light, the glow and the two-color palettes off. It never recorded what
//! it was saved with; the defaults are what most such views used.

use crate::slider_range;
use crate::{ColorMethod, TrapType};

/// Duotone or Binary: the two-color palettes, which win over the preset and the custom gradient
/// (`chosen_gradient`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TwoColor {
    Off,
    Duotone,
    Binary,
}

impl TwoColor {
    /// The two palettes a view can name (`Off` is the absence of the key).
    pub(crate) const ON: [TwoColor; 2] = [TwoColor::Duotone, TwoColor::Binary];

    /// The `palette_mode=` value; `None` for `Off`, which is never written.
    pub(crate) fn key(self) -> Option<&'static str> {
        match self {
            TwoColor::Off => None,
            TwoColor::Duotone => Some("duotone"),
            TwoColor::Binary => Some("binary"),
        }
    }

    pub(crate) fn from_key(s: &str) -> Option<TwoColor> {
        Self::ON.into_iter().find(|m| m.key() == Some(s))
    }
}

/// Every key this module writes and reads. The reader's own list (`KNOWN_VIEW_KEYS`) names each
/// one too, and a unit test holds the two lists and the writer's output together.
pub(crate) const KEYS: &[&str] = &[
    "method",
    "stripe_freq",
    "stripe_tail",
    "stripe_tail_len",
    "trap",
    "light",
    "light_angle",
    "light_height",
    "glow",
    "glow_strength",
    "glow_width",
    "palette_mode",
    "palette_lo",
    "palette_hi",
    "normalize_live",
    "normalize_fit",
    "log_scale",
];

/// The coloring settings a view carries, in one place, apart from the app (whose `ColoringConfig`
/// and `EffectsConfig` hold them beside much else), so the writer and the reader can be tested on
/// their own.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ViewColoring {
    pub(crate) method: ColorMethod,
    /// Stripe method: density, and the tail-only average with its length.
    pub(crate) stripe_freq: f32,
    pub(crate) stripe_tail: bool,
    pub(crate) stripe_tail_len: u32,
    /// Orbit trap method: the shape.
    pub(crate) trap: TrapType,
    /// Relief lighting: on, angle (radians), the "Relief" slider.
    pub(crate) light: bool,
    pub(crate) light_angle: f32,
    pub(crate) light_height: f32,
    /// Distance glow (the app's `de`): on, the "Glow" and "Band width" sliders.
    pub(crate) glow: bool,
    pub(crate) glow_strength: f32,
    pub(crate) glow_width: f32,
    /// Duotone or Binary, and the two colors they share (display-space RGB): Duotone's shadow and
    /// highlight, Binary's in-set and out-of-set colors.
    pub(crate) two_color: TwoColor,
    pub(crate) lo: [f32; 3],
    pub(crate) hi: [f32; 3],
    /// "Normalize deep colors", "Always fit range" and "Log color scale".
    pub(crate) normalize_live: bool,
    pub(crate) normalize_fit: bool,
    pub(crate) log_scale: bool,
}

impl Default for ViewColoring {
    /// A new session's coloring (`SessionState::default`), so the defaults live in one place.
    fn default() -> Self {
        let s = fractadyne_state::SessionState::default();
        ViewColoring {
            method: ColorMethod::from_key(&s.color_method),
            stripe_freq: s.stripe_freq,
            stripe_tail: s.stripe_tail,
            stripe_tail_len: s.stripe_tail_len,
            trap: TrapType::from_key(&s.trap_type),
            light: s.light,
            light_angle: s.light_angle,
            light_height: s.light_height,
            glow: s.de,
            glow_strength: s.de_strength,
            glow_width: s.de_width,
            two_color: if s.use_binary {
                TwoColor::Binary
            } else if s.use_duotone {
                TwoColor::Duotone
            } else {
                TwoColor::Off
            },
            lo: s.duotone_lo,
            hi: s.duotone_hi,
            normalize_live: s.normalize_live,
            normalize_fit: s.normalize_fit,
            log_scale: s.log_palette,
        }
    }
}

/// What a field says: nothing, a usable value (already clamped into range), or something that
/// cannot be used, which keeps the current setting.
enum Said<T> {
    Absent,
    Value(T),
    Unusable,
}

/// The new value for a setting, or `None` to keep the current one: the file's value when it has
/// one, the default when the file leaves the key out of a picture that depends on it.
fn resolve<T>(said: Said<T>, reset_when_absent: bool, default: T) -> Option<T> {
    match said {
        Said::Value(v) => Some(v),
        Said::Absent if reset_when_absent => Some(default),
        Said::Absent | Said::Unusable => None,
    }
}

fn flag(b: bool) -> String {
    if b { "1" } else { "0" }.to_string()
}

fn rgb(c: [f32; 3]) -> String {
    format!("{},{},{}", c[0], c[1], c[2])
}

impl ViewColoring {
    /// "Normalize deep colors" decides only for Smooth (`live_norm_window`).
    fn uses_normalize_live(method: ColorMethod) -> bool {
        method == ColorMethod::Smooth
    }

    /// "Always fit range" and the log scale act on Smooth's escape range and on the orbit trap's
    /// values (`live_norm_window`, `norm_map_for`).
    fn uses_fit(method: ColorMethod) -> bool {
        matches!(method, ColorMethod::Smooth | ColorMethod::OrbitTrap)
    }

    /// The view lines for this coloring: each setting the picture depends on that differs from the
    /// default, one `key=value` per line, in a fixed order. Empty for the default coloring.
    ///
    /// ⭐Floats with plain `{}`: Rust prints the shortest decimal that reads back to the same bits,
    /// so every value round-trips exactly (the custom gradient's encoding does the same).
    pub(crate) fn lines(&self) -> String {
        let d = ViewColoring::default();
        let mut out = String::new();
        let mut put = |k: &str, v: String| {
            out.push_str(k);
            out.push('=');
            out.push_str(&v);
            out.push('\n');
        };
        if self.method != d.method {
            put("method", self.method.key().to_string());
        }
        if self.method == ColorMethod::Stripe {
            if self.stripe_freq != d.stripe_freq {
                put("stripe_freq", self.stripe_freq.to_string());
            }
            if self.stripe_tail != d.stripe_tail {
                put("stripe_tail", flag(self.stripe_tail));
            }
            if self.stripe_tail && self.stripe_tail_len != d.stripe_tail_len {
                put("stripe_tail_len", self.stripe_tail_len.to_string());
            }
        }
        if self.method == ColorMethod::OrbitTrap && self.trap != d.trap {
            put("trap", self.trap.key().to_string());
        }
        if self.light != d.light {
            put("light", flag(self.light));
        }
        if self.light {
            if self.light_angle != d.light_angle {
                put("light_angle", self.light_angle.to_string());
            }
            if self.light_height != d.light_height {
                put("light_height", self.light_height.to_string());
            }
        }
        if self.glow != d.glow {
            put("glow", flag(self.glow));
        }
        if self.glow {
            if self.glow_strength != d.glow_strength {
                put("glow_strength", self.glow_strength.to_string());
            }
            if self.glow_width != d.glow_width {
                put("glow_width", self.glow_width.to_string());
            }
        }
        if let Some(k) = self.two_color.key() {
            put("palette_mode", k.to_string());
            if self.lo != d.lo {
                put("palette_lo", rgb(self.lo));
            }
            if self.hi != d.hi {
                put("palette_hi", rgb(self.hi));
            }
        }
        if Self::uses_normalize_live(self.method) && self.normalize_live != d.normalize_live {
            put("normalize_live", flag(self.normalize_live));
        }
        if Self::uses_fit(self.method) {
            if self.normalize_fit != d.normalize_fit {
                put("normalize_fit", flag(self.normalize_fit));
            }
            if self.log_scale != d.log_scale {
                put("log_scale", flag(self.log_scale));
            }
        }
        out
    }

    /// Apply a view's coloring fields to `self`, this session's coloring, by the rules at the top
    /// of this module. `get` reads a field (`None` when the file does not have it); `states` says
    /// the file states its coloring (it has `cycle=`). A value out of range is clamped and named in
    /// `clamped`; an unreadable one keeps the current setting (the reader's type table reports it
    /// with its line, and an unknown method also keeps the settings that hang off the method).
    pub(crate) fn apply(&mut self, get: &dyn Fn(&str) -> Option<String>, states: bool, clamped: &mut Vec<&'static str>) {
        let d = ViewColoring::default();
        // A number, clamped into `range` (named in `clamped` when it had to be); not finite is
        // unusable, and named too, as the cycle's reader does.
        let mut real = |key: &str, range: std::ops::RangeInclusive<f32>, name: &'static str| -> Said<f32> {
            match get(key) {
                None => Said::Absent,
                Some(v) => match v.parse::<f32>() {
                    Ok(x) if x.is_finite() => {
                        let c = x.clamp(*range.start(), *range.end());
                        if c != x {
                            clamped.push(name);
                        }
                        Said::Value(c)
                    }
                    Ok(_) => {
                        clamped.push(name);
                        Said::Unusable
                    }
                    Err(_) => Said::Unusable,
                },
            }
        };
        let stripe_freq = real("stripe_freq", slider_range::STRIPE_FREQ, "stripe density");
        let light_angle = real("light_angle", f32::MIN..=f32::MAX, "light angle");
        let light_height = real("light_height", slider_range::LIGHT_HEIGHT, "relief");
        let glow_strength = real("glow_strength", slider_range::GLOW_STRENGTH, "glow");
        let glow_width = real("glow_width", slider_range::GLOW_WIDTH, "band width");
        let on_off = |key: &str| -> Said<bool> {
            match get(key).as_deref() {
                None => Said::Absent,
                Some("1") => Said::Value(true),
                Some("0") => Said::Value(false),
                Some(_) => Said::Unusable,
            }
        };
        let method = match get("method") {
            None => Said::Absent,
            Some(v) => ColorMethod::ALL.into_iter().find(|m| m.key() == v).map_or(Said::Unusable, Said::Value),
        };
        let trap = match get("trap") {
            None => Said::Absent,
            Some(v) => TrapType::ALL.into_iter().find(|t| t.key() == v).map_or(Said::Unusable, Said::Value),
        };
        let mode = match get("palette_mode") {
            None => Said::Absent,
            Some(v) => TwoColor::from_key(&v).map_or(Said::Unusable, Said::Value),
        };
        let tail_len = match get("stripe_tail_len") {
            None => Said::Absent,
            Some(v) => match v.parse::<u64>() {
                Ok(n) => {
                    let (lo, hi) = (*slider_range::STRIPE_TAIL_LEN.start(), *slider_range::STRIPE_TAIL_LEN.end());
                    let c = n.clamp(lo as u64, hi as u64) as u32;
                    if c as u64 != n {
                        clamped.push("tail length");
                    }
                    Said::Value(c)
                }
                Err(_) => Said::Unusable,
            },
        };
        let mut color = |key: &str| -> Said<[f32; 3]> {
            let Some(v) = get(key) else { return Said::Absent };
            let parts: Vec<Option<f32>> = v.split(',').map(|t| t.trim().parse::<f32>().ok().filter(|x| x.is_finite())).collect();
            match parts.as_slice() {
                [Some(r), Some(g), Some(b)] => {
                    let c = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
                    if c != [*r, *g, *b] {
                        clamped.push("palette colors");
                    }
                    Said::Value(c)
                }
                _ => Said::Unusable,
            }
        };
        let (lo, hi) = (color("palette_lo"), color("palette_hi"));

        // The method, then what hangs off it. `method` is the file's method when the file says it
        // (or says nothing while stating its coloring, which means Smooth); `None` when it names
        // one this build does not have, or states no coloring.
        let method = resolve(method, states, d.method);
        if let Some(m) = method {
            self.method = m;
        }
        let stripe = method == Some(ColorMethod::Stripe);
        if let Some(v) = resolve(stripe_freq, stripe, d.stripe_freq) {
            self.stripe_freq = v;
        }
        let tail = resolve(on_off("stripe_tail"), stripe, d.stripe_tail);
        if let Some(t) = tail {
            self.stripe_tail = t;
        }
        if let Some(n) = resolve(tail_len, stripe && tail == Some(true), d.stripe_tail_len) {
            self.stripe_tail_len = n;
        }
        if let Some(t) = resolve(trap, method == Some(ColorMethod::OrbitTrap), d.trap) {
            self.trap = t;
        }

        let light = resolve(on_off("light"), states, d.light);
        if let Some(l) = light {
            self.light = l;
        }
        if let Some(a) = resolve(light_angle, light == Some(true), d.light_angle) {
            self.light_angle = a;
        }
        if let Some(h) = resolve(light_height, light == Some(true), d.light_height) {
            self.light_height = h;
        }
        let glow = resolve(on_off("glow"), states, d.glow);
        if let Some(g) = glow {
            self.glow = g;
        }
        if let Some(s) = resolve(glow_strength, glow == Some(true), d.glow_strength) {
            self.glow_strength = s;
        }
        if let Some(w) = resolve(glow_width, glow == Some(true), d.glow_width) {
            self.glow_width = w;
        }

        let mode = resolve(mode, states, TwoColor::Off);
        if let Some(m) = mode {
            self.two_color = m;
        }
        let two = matches!(mode, Some(TwoColor::Duotone | TwoColor::Binary));
        if let Some(c) = resolve(lo, two, d.lo) {
            self.lo = c;
        }
        if let Some(c) = resolve(hi, two, d.hi) {
            self.hi = c;
        }

        let live = method.is_some_and(Self::uses_normalize_live);
        if let Some(b) = resolve(on_off("normalize_live"), live, d.normalize_live) {
            self.normalize_live = b;
        }
        let fit = method.is_some_and(Self::uses_fit);
        if let Some(b) = resolve(on_off("normalize_fit"), fit, d.normalize_fit) {
            self.normalize_fit = b;
        }
        if let Some(b) = resolve(on_off("log_scale"), fit, d.log_scale) {
            self.log_scale = b;
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/view_coloring/round_trip.rs"]
mod round_trip;
