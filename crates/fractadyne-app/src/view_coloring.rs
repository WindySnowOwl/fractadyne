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
//!
//! ⭐**Since B4 the rules live in a table** ([`crate::coloring_fields`]), which writes and reads
//! these keys on the session's [`ColoringSpec`]; the app's views use it directly. [`ViewColoring`]
//! is a typed front on it (the method and the trap as enums, Duotone or Binary as one choice) for
//! code that sets a coloring by hand: the harnesses and this module's tests.

use crate::coloring_fields;
use crate::{ColorMethod, InteriorMode, TrapPlace, TrapType};
use fractadyne_state::ColoringSpec;

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

/// Every key a view's coloring writes and reads: the table's ([`coloring_fields::KEYS`]).
pub(crate) const KEYS: &[&str] = &coloring_fields::KEYS;

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
    /// Orbit trap method: the shape, and where it sits, turned and sized (F4).
    pub(crate) trap: TrapType,
    pub(crate) trap_place: TrapPlace,
    /// The inside's own trap (the trap method's inside colored by a trap of its own), `None` for
    /// the method's, and its placement.
    pub(crate) interior_trap: Option<TrapType>,
    pub(crate) interior_trap_place: TrapPlace,
    /// Relief lighting: on, angle (radians), the "Relief" slider.
    pub(crate) light: bool,
    pub(crate) light_angle: f32,
    pub(crate) light_height: f32,
    /// The relief's surface and highlight (C5): the slope from the distance and its steepness, the
    /// highlight's strength, shininess and color.
    pub(crate) light_steep: bool,
    pub(crate) light_steep_px: f32,
    pub(crate) light_highlight: f32,
    pub(crate) light_shininess: f32,
    pub(crate) light_highlight_color: [f32; 3],
    /// Distance glow (the app's `de`): on, the "Glow" and "Band width" sliders.
    pub(crate) glow: bool,
    pub(crate) glow_strength: f32,
    pub(crate) glow_width: f32,
    /// Edge shading (C3): on, toward the inside's color, its width, strength, curve and color.
    pub(crate) edge: bool,
    pub(crate) edge_inside: bool,
    pub(crate) edge_width: f32,
    pub(crate) edge_strength: f32,
    pub(crate) edge_curve: fractadyne_core::coloring::EdgeCurve,
    pub(crate) edge_color: [f32; 3],
    /// Equipotential lines (C10): on, their spacing, width, strength and color.
    pub(crate) equipotentials: bool,
    pub(crate) equipotential_spacing: f32,
    pub(crate) equipotential_width: f32,
    pub(crate) equipotential_strength: f32,
    pub(crate) equipotential_color: [f32; 3],
    /// Duotone or Binary, and the two colors they share (display-space RGB): Duotone's shadow and
    /// highlight, Binary's in-set and out-of-set colors.
    pub(crate) two_color: TwoColor,
    pub(crate) lo: [f32; 3],
    pub(crate) hi: [f32; 3],
    /// "Normalize deep colors", "Always fit range" and "Log color scale".
    pub(crate) normalize_live: bool,
    pub(crate) normalize_fit: bool,
    pub(crate) log_scale: bool,
    /// The palette map's curve, how the palette continues past its end, and its steps (C2). The
    /// curve as the view's `transfer` key says it (`log_scale` is its own key: older files').
    pub(crate) transfer: fractadyne_core::coloring::Transfer,
    pub(crate) wrap: fractadyne_core::coloring::Wrap,
    pub(crate) steps: u32,
    /// What colors the inside (B5): `None`, the method's own. Its palette, a preset's id; empty for
    /// the outside palette.
    pub(crate) interior: Option<InteriorMode>,
    pub(crate) interior_palette: String,
    /// A solid inside's color (C1), display-space RGB.
    pub(crate) interior_color: [f32; 3],
    /// The inside's palette fitted to its measured range (C4).
    pub(crate) interior_fit: bool,
    /// The palette anti-aliased over each pixel's footprint (C8).
    pub(crate) palette_aa: bool,
    /// The palette's brightness, contrast and saturation (`fractadyne_color::adjust`).
    pub(crate) palette_brightness: f32,
    pub(crate) palette_contrast: f32,
    pub(crate) palette_saturation: f32,
    /// The palette blended in OKLab (C7).
    pub(crate) palette_oklab: bool,
    /// The dual view's Julia pane on the set's palette map, one iteration on (C11).
    pub(crate) julia_linked: bool,
    /// Equalize colors: the share of histogram coloring in the fitted map (C6).
    pub(crate) equalize: f32,
}

impl Default for ViewColoring {
    /// A new session's coloring (`ColoringSpec::default`), so the defaults live in one place.
    fn default() -> Self {
        ViewColoring::from_spec(&ColoringSpec::default())
    }
}

impl ViewColoring {
    /// The view's coloring held in a session's [`ColoringSpec`].
    pub(crate) fn from_spec(s: &ColoringSpec) -> Self {
        ViewColoring {
            method: ColorMethod::from_key(&s.color_method),
            stripe_freq: s.stripe_freq,
            stripe_tail: s.stripe_tail,
            stripe_tail_len: s.stripe_tail_len,
            trap: TrapType::from_key(&s.trap_type),
            trap_place: crate::trap_place_of(s),
            interior_trap: TrapType::parse_key(&s.interior_trap),
            interior_trap_place: crate::interior_trap_place_of(s),
            light: s.light,
            light_angle: s.light_angle,
            light_height: s.light_height,
            light_steep: s.light_steep,
            light_steep_px: s.light_steep_px,
            light_highlight: s.light_highlight,
            light_shininess: s.light_shininess,
            light_highlight_color: s.light_highlight_color,
            glow: s.de,
            glow_strength: s.de_strength,
            glow_width: s.de_width,
            edge: s.edge,
            edge_inside: s.edge_inside,
            edge_width: s.edge_width,
            edge_strength: s.edge_strength,
            edge_curve: fractadyne_core::coloring::EdgeCurve::parse_key(&s.edge_curve).unwrap_or_default(),
            edge_color: s.edge_color,
            equipotentials: s.equipotentials,
            equipotential_spacing: s.equipotential_spacing,
            equipotential_width: s.equipotential_width,
            equipotential_strength: s.equipotential_strength,
            equipotential_color: s.equipotential_color,
            // Binary first, as `chosen_gradient` reads the two switches.
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
            transfer: fractadyne_core::coloring::Transfer::parse_key(&s.transfer).unwrap_or_default(),
            wrap: fractadyne_core::coloring::Wrap::parse_key(&s.wrap).unwrap_or_default(),
            steps: s.steps,
            interior: InteriorMode::parse_key(&s.interior),
            interior_palette: s.interior_palette.clone(),
            interior_color: s.interior_color,
            interior_fit: s.interior_fit,
            palette_aa: s.palette_aa,
            palette_brightness: s.palette_brightness,
            palette_contrast: s.palette_contrast,
            palette_saturation: s.palette_saturation,
            palette_oklab: s.palette_oklab,
            julia_linked: s.julia_linked,
            equalize: s.equalize,
        }
    }

    /// `base` with this view's coloring in it: the inverse of [`Self::from_spec`] for every field
    /// a view carries, the rest (the palette, the cycle, the gradient) as `base` has them.
    pub(crate) fn to_spec(&self, base: &ColoringSpec) -> ColoringSpec {
        ColoringSpec {
            color_method: self.method.key().to_string(),
            stripe_freq: self.stripe_freq,
            stripe_tail: self.stripe_tail,
            stripe_tail_len: self.stripe_tail_len,
            trap_type: self.trap.key().to_string(),
            trap_x: self.trap_place.center[0],
            trap_y: self.trap_place.center[1],
            trap_rotation: self.trap_place.rotation,
            trap_size: self.trap_place.size,
            trap_phase: self.trap_place.phase,
            trap_harmonics: self.trap_place.harmonics,
            trap_sine_harmonics: self.trap_place.sine_harmonics,
            interior_trap: self.interior_trap.map_or(String::new(), |t| t.key().to_string()),
            interior_trap_x: self.interior_trap_place.center[0],
            interior_trap_y: self.interior_trap_place.center[1],
            interior_trap_rotation: self.interior_trap_place.rotation,
            interior_trap_size: self.interior_trap_place.size,
            interior_trap_phase: self.interior_trap_place.phase,
            interior_trap_harmonics: self.interior_trap_place.harmonics,
            interior_trap_sine_harmonics: self.interior_trap_place.sine_harmonics,
            light: self.light,
            light_angle: self.light_angle,
            light_height: self.light_height,
            light_steep: self.light_steep,
            light_steep_px: self.light_steep_px,
            light_highlight: self.light_highlight,
            light_shininess: self.light_shininess,
            light_highlight_color: self.light_highlight_color,
            de: self.glow,
            de_strength: self.glow_strength,
            de_width: self.glow_width,
            edge: self.edge,
            edge_inside: self.edge_inside,
            edge_width: self.edge_width,
            edge_strength: self.edge_strength,
            edge_curve: self.edge_curve.key().to_string(),
            edge_color: self.edge_color,
            equipotentials: self.equipotentials,
            equipotential_spacing: self.equipotential_spacing,
            equipotential_width: self.equipotential_width,
            equipotential_strength: self.equipotential_strength,
            equipotential_color: self.equipotential_color,
            use_binary: self.two_color == TwoColor::Binary,
            use_duotone: self.two_color == TwoColor::Duotone,
            duotone_lo: self.lo,
            duotone_hi: self.hi,
            normalize_live: self.normalize_live,
            normalize_fit: self.normalize_fit,
            log_palette: self.log_scale,
            transfer: self.transfer.key().to_string(),
            wrap: self.wrap.key().to_string(),
            steps: self.steps,
            interior: self.interior.map(|m| m.key().to_string()).unwrap_or_default(),
            interior_palette: self.interior_palette.clone(),
            interior_color: self.interior_color,
            interior_fit: self.interior_fit,
            palette_aa: self.palette_aa,
            palette_brightness: self.palette_brightness,
            palette_contrast: self.palette_contrast,
            palette_saturation: self.palette_saturation,
            palette_oklab: self.palette_oklab,
            julia_linked: self.julia_linked,
            equalize: self.equalize,
            ..base.clone()
        }
    }

    /// The view lines for this coloring ([`coloring_fields::view_lines`]): each setting the
    /// picture depends on that differs from the default, one `key=value` per line, in the table's
    /// order. Empty for the default coloring.
    pub(crate) fn lines(&self) -> String {
        coloring_fields::view_lines(&self.to_spec(&ColoringSpec::default()))
    }
}

#[cfg(test)]
#[path = "../tests/unit/view_coloring/round_trip.rs"]
mod round_trip;
