//! The render-affecting settings a job carries (design §9).
//!
//! A tour states its camera path, palettes and budgets, but a render also reads the machine's saved
//! session: colouring method, lighting, distance glow, the palette when no keyframe names one, the
//! iteration base when the script gives none, SA/BLA/glitch switches, the watermark. Rendered on
//! machines with different sessions, one tour came out as several. So the controller reads these
//! from ITS session into a [`RenderSettings`], and each client writes a FRESH session from it
//! (`SessionState::default()` plus exactly these fields) for its render processes.
//!
//! Typed and checked, never a session file: a controller can set what is listed here and nothing
//! else — no path, no window state, no update channel.

use fractadyne_state::{PaletteSegment, SessionState};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RenderSettings {
    pub max_iter: u32,
    pub auto_iter: bool,
    pub palette_idx: usize,
    pub cycle: f32,
    pub offset: f32,
    pub log_palette: bool,
    pub light: bool,
    pub light_angle: f32,
    pub light_height: f32,
    pub de: bool,
    pub de_strength: f32,
    pub de_width: f32,
    pub color_method: String,
    pub stripe_freq: f32,
    pub stripe_tail: bool,
    pub stripe_tail_len: u32,
    pub trap_type: String,
    pub custom_palette: Vec<[f32; 4]>,
    pub custom_palette_flat: bool,
    pub custom_segments: Vec<PaletteSegment>,
    pub use_custom_palette: bool,
    pub use_duotone: bool,
    pub use_binary: bool,
    pub duotone_lo: [f32; 3],
    pub duotone_hi: [f32; 3],
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
            palette_idx: s.palette_idx,
            cycle: s.cycle,
            offset: s.offset,
            log_palette: s.log_palette,
            light: s.light,
            light_angle: s.light_angle,
            light_height: s.light_height,
            de: s.de,
            de_strength: s.de_strength,
            de_width: s.de_width,
            color_method: s.color_method.clone(),
            stripe_freq: s.stripe_freq,
            stripe_tail: s.stripe_tail,
            stripe_tail_len: s.stripe_tail_len,
            trap_type: s.trap_type.clone(),
            custom_palette: s.custom_palette.clone(),
            custom_palette_flat: s.custom_palette_flat,
            custom_segments: s.custom_segments.clone(),
            use_custom_palette: s.use_custom_palette,
            use_duotone: s.use_duotone,
            use_binary: s.use_binary,
            duotone_lo: s.duotone_lo,
            duotone_hi: s.duotone_hi,
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
            palette_idx: self.palette_idx,
            cycle: self.cycle,
            offset: self.offset,
            log_palette: self.log_palette,
            light: self.light,
            light_angle: self.light_angle,
            light_height: self.light_height,
            de: self.de,
            de_strength: self.de_strength,
            de_width: self.de_width,
            color_method: self.color_method.clone(),
            stripe_freq: self.stripe_freq,
            stripe_tail: self.stripe_tail,
            stripe_tail_len: self.stripe_tail_len,
            trap_type: self.trap_type.clone(),
            custom_palette: self.custom_palette.clone(),
            custom_palette_flat: self.custom_palette_flat,
            custom_segments: self.custom_segments.clone(),
            use_custom_palette: self.use_custom_palette,
            use_duotone: self.use_duotone,
            use_binary: self.use_binary,
            duotone_lo: self.duotone_lo,
            duotone_hi: self.duotone_hi,
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
        if self.palette_idx > 1024 || self.stripe_tail_len > 1 << 20 {
            return Err("palette index or stripe tail out of range".into());
        }
        for (w, v) in [
            ("cycle", self.cycle),
            ("offset", self.offset),
            ("light angle", self.light_angle),
            ("light height", self.light_height),
            ("glow strength", self.de_strength),
            ("glow width", self.de_width),
            ("stripe frequency", self.stripe_freq),
        ] {
            finite32(w, v)?;
        }
        finite64("Julia c", self.julia_c_re)?;
        finite64("Julia c", self.julia_c_im)?;
        word("colouring method", &self.color_method)?;
        word("trap", &self.trap_type)?;
        word("fractal", &self.fractal)?;
        textual("custom formula", &self.custom_formula, 8 * 1024)?;
        textual("L-system", &self.lsystem, 64 * 1024)?;
        if self.custom_params.len() > 8 || self.custom_params.iter().flatten().any(|v| !v.is_finite()) {
            return Err("custom formula parameters out of range".into());
        }
        if self.custom_palette.len() > 256 || self.custom_segments.len() > 512 {
            return Err("custom palette too large".into());
        }
        let colours_ok = self.custom_palette.iter().flatten().all(|v| v.is_finite())
            && self.duotone_lo.iter().chain(&self.duotone_hi).all(|v| v.is_finite())
            && self.custom_segments.iter().all(|g| {
                [g.left, g.mid, g.right].iter().chain(&g.left_color).chain(&g.right_color).chain(&g.blend_params).all(|v| v.is_finite())
            });
        if !colours_ok {
            return Err("a palette colour is not a finite number".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
