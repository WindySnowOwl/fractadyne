//! The render-affecting settings a job carries (design §9).
//!
//! A tour states its camera path, palettes and budgets, but a render also reads the machine's saved
//! session: coloring method, lighting, distance glow, the palette when no keyframe names one, the
//! iteration base when the script gives none, SA/BLA/glitch switches, the watermark. Rendered on
//! machines with different sessions, one tour came out as several. So the controller reads these
//! from ITS session into a [`RenderSettings`], and each client writes a FRESH session from it
//! (`SessionState::default()` plus exactly these fields) for its render processes.
//!
//! Typed and checked, never a session file: a controller can set what is listed here and nothing
//! else — no path, no window state, no update channel.

use fractadyne_state::{ColoringSpec, SessionState};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RenderSettings {
    pub max_iter: u32,
    pub auto_iter: bool,
    /// How the picture is colored: the session's own struct, so every coloring setting the
    /// session saves travels with the job (coloring plan B4; protocol 9). Before, this listed
    /// its own copies and missed the palette's id and the live normalization.
    /// ⚠The struct cannot refuse unknown keys itself (the session flattens it, and serde does not
    /// combine the two), so this field is read through [`strict_coloring`], which does.
    #[serde(deserialize_with = "strict_coloring")]
    pub coloring: ColoringSpec,
    pub fractal: String,
    pub custom_formula: String,
    pub custom_params: Vec<[f64; 2]>,
    pub lsystem: String,
    pub julia_mode: bool,
    pub julia_c_re: f64,
    pub julia_c_im: f64,
    pub series_approx: bool,
    pub glitch_correct: bool,
    pub use_bla: bool,
    pub watermark: bool,
    pub show_watermark: bool,
    pub show_location: bool,
    pub orbit_normalize: bool,
}

impl RenderSettings {
    /// The listed fields of a session.
    pub fn from_session(s: &SessionState) -> Self {
        Self {
            max_iter: s.max_iter,
            auto_iter: s.auto_iter,
            coloring: s.coloring.clone(),
            fractal: s.fractal.clone(),
            custom_formula: s.custom_formula.clone(),
            custom_params: s.custom_params.clone(),
            lsystem: s.lsystem.clone(),
            julia_mode: s.julia_mode,
            julia_c_re: s.julia_c_re,
            julia_c_im: s.julia_c_im,
            series_approx: s.series_approx,
            glitch_correct: s.glitch_correct,
            use_bla: s.use_bla,
            watermark: s.watermark,
            show_watermark: s.show_watermark,
            show_location: s.show_location,
            orbit_normalize: s.orbit_normalize,
        }
    }

    /// A fresh session with exactly these settings — everything else at its default. Animations
    /// stay off (the default), which is also what `--farm-child` pins.
    pub fn to_session(&self) -> SessionState {
        SessionState {
            max_iter: self.max_iter,
            auto_iter: self.auto_iter,
            coloring: self.coloring.clone(),
            fractal: self.fractal.clone(),
            custom_formula: self.custom_formula.clone(),
            custom_params: self.custom_params.clone(),
            lsystem: self.lsystem.clone(),
            julia_mode: self.julia_mode,
            julia_c_re: self.julia_c_re,
            julia_c_im: self.julia_c_im,
            series_approx: self.series_approx,
            glitch_correct: self.glitch_correct,
            use_bla: self.use_bla,
            watermark: self.watermark,
            show_watermark: self.show_watermark,
            show_location: self.show_location,
            orbit_normalize: self.orbit_normalize,
            ..SessionState::default()
        }
    }

    /// Every field in range. Run by the client on arrival; a failure refuses the job.
    pub fn validate(&self) -> Result<(), String> {
        let finite32 = |what: &str, v: f32| if v.is_finite() { Ok(()) } else { Err(format!("{what} is not a finite number")) };
        let finite64 = |what: &str, v: f64| if v.is_finite() { Ok(()) } else { Err(format!("{what} is not a finite number")) };
        let word = |what: &str, s: &str| {
            if s.len() <= 64 && s.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
                Ok(())
            } else {
                Err(format!("{what} \"{}\" is not a setting name", s.chars().take(40).collect::<String>()))
            }
        };
        let textual = |what: &str, s: &str, max: usize| {
            if s.len() > max {
                Err(format!("{what} is {} bytes, over {max}", s.len()))
            } else if s.chars().any(|c| c.is_control() && c != '\n' && c != '\t' && c != '\r') {
                Err(format!("{what} contains control characters"))
            } else {
                Ok(())
            }
        };
        if self.max_iter == 0 || self.max_iter > 10_000_000 {
            return Err(format!("iteration base {} is out of range", self.max_iter));
        }
        if self.coloring.palette_idx > 1024 || self.coloring.stripe_tail_len > 1 << 20 {
            return Err("palette index or stripe tail out of range".into());
        }
        if !(1..=fractadyne_state::ITER_DIV_MAX).contains(&self.coloring.iter_div) {
            return Err(format!("iteration divisor {} is out of range", self.coloring.iter_div));
        }
        for (w, v) in [
            ("cycle", self.coloring.cycle),
            ("offset", self.coloring.offset),
            ("light angle", self.coloring.light_angle),
            ("light height", self.coloring.light_height),
            ("glow strength", self.coloring.de_strength),
            ("glow width", self.coloring.de_width),
            ("stripe frequency", self.coloring.stripe_freq),
        ] {
            finite32(w, v)?;
        }
        finite64("Julia c", self.julia_c_re)?;
        finite64("Julia c", self.julia_c_im)?;
        word("coloring method", &self.coloring.color_method)?;
        word("trap", &self.coloring.trap_type)?;
        word("fractal", &self.fractal)?;
        textual("custom formula", &self.custom_formula, 8 * 1024)?;
        textual("L-system", &self.lsystem, 64 * 1024)?;
        if self.custom_params.len() > 8 || self.custom_params.iter().flatten().any(|v| !v.is_finite()) {
            return Err("custom formula parameters out of range".into());
        }
        if self.coloring.custom_palette.len() > 256 || self.coloring.custom_segments.len() > 512 {
            return Err("custom palette too large".into());
        }
        let colours_ok = self.coloring.custom_palette.iter().flatten().all(|v| v.is_finite())
            && self.coloring.duotone_lo.iter().chain(&self.coloring.duotone_hi).all(|v| v.is_finite())
            && self.coloring.custom_segments.iter().all(|g| {
                [g.left, g.mid, g.right].iter().chain(&g.left_color).chain(&g.right_color).chain(&g.blend_params).all(|v| v.is_finite())
            });
        if !colours_ok {
            return Err("a palette color is not a finite number".into());
        }
        Ok(())
    }
}

/// [`RenderSettings::coloring`], refusing a key [`ColoringSpec`] does not have, as the rest of the
/// struct does (`deny_unknown_fields`): a job names exactly what it sets. The known keys are the
/// spec's own, read off a serialized default, so a field added to it is known here at once.
fn strict_coloring<'de, D: serde::Deserializer<'de>>(d: D) -> Result<ColoringSpec, D::Error> {
    use serde::de::Error;
    let map = serde_json::Map::<String, serde_json::Value>::deserialize(d)?;
    let known = serde_json::to_value(ColoringSpec::default()).map_err(D::Error::custom)?;
    if let Some(k) = map.keys().find(|k| known.get(k.as_str()).is_none()) {
        return Err(D::Error::custom(format!("unknown coloring field `{k}`")));
    }
    serde_json::from_value(serde_json::Value::Object(map)).map_err(D::Error::custom)
}

#[cfg(test)]
#[path = "../tests/unit/settings/tests.rs"]
mod tests;
