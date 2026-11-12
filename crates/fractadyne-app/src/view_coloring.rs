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
use crate::{ColorMethod, TrapType};
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
            light: s.light,
            light_angle: s.light_angle,
            light_height: s.light_height,
            glow: s.de,
            glow_strength: s.de_strength,
            glow_width: s.de_width,
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
            light: self.light,
            light_angle: self.light_angle,
            light_height: self.light_height,
            de: self.glow,
            de_strength: self.glow_strength,
            de_width: self.glow_width,
            use_binary: self.two_color == TwoColor::Binary,
            use_duotone: self.two_color == TwoColor::Duotone,
            duotone_lo: self.lo,
            duotone_hi: self.hi,
            normalize_live: self.normalize_live,
            normalize_fit: self.normalize_fit,
            log_palette: self.log_scale,
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
