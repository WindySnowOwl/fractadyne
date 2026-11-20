//! Brightness, contrast and saturation of a palette (user request, 2026-11-15): three sliders that
//! change how a palette looks without editing its stops, for presets, saved gradients and imported
//! ones alike.
//!
//! ⭐**They change the baked table, not the picture's data.** A palette reaches the GPU as a table
//! (`segment::Lut`); [`PaletteAdjust::apply`] rewrites its entries, so a change re-bakes and
//! recolors and never calls for a new iteration. Everything that reads the table follows by
//! construction: the palette's anti-aliasing (its prefix sum is taken from the adjusted entries),
//! the inside's palette, thumbnails, exports.
//!
//! ⭐**In OKLab, a perceptual space.** The table holds display (sRGB-encoded) colors; each entry is
//! taken to OKLab, adjusted, and brought back:
//! - brightness moves the midtones' lightness along a power curve that keeps black and white, so
//!   nothing clips (`L^(2^-b)`);
//! - contrast pushes lightness away from the middle (or pulls it in) along an S-curve that keeps
//!   black and white (`L^a / (L^a + (1-L)^a)`, `a = 2^(1.5 c)`);
//! - saturation scales chroma, keeping lightness and hue (0 is gray).
//!
//! An adjusted color outside the sRGB gamut keeps its lightness and hue and gives up chroma until
//! it fits, so a saturated palette does not shift hue at the gamut's edge as clipping each channel
//! would. ⭐**Neutral is no change at all**: [`PaletteAdjust::apply`] returns before touching the
//! table, so every saved view, golden and export without adjustments is bit for bit as before.

use crate::segment::Lut;

/// Brightness: -1 (darker midtones) to 1 (lighter); 0 is no change.
pub const BRIGHTNESS_RANGE: std::ops::RangeInclusive<f32> = -1.0..=1.0;
/// Contrast: -1 (flatter) to 1 (stronger); 0 is no change.
pub const CONTRAST_RANGE: std::ops::RangeInclusive<f32> = -1.0..=1.0;
/// Saturation: 0 (gray) to 2 (twice the chroma); 1 is no change.
pub const SATURATION_RANGE: std::ops::RangeInclusive<f32> = 0.0..=2.0;

/// A palette's brightness, contrast and saturation (see the module).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaletteAdjust {
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
}

impl Default for PaletteAdjust {
    fn default() -> Self {
        Self::NEUTRAL
    }
}

impl PaletteAdjust {
    /// No change.
    pub const NEUTRAL: PaletteAdjust = PaletteAdjust { brightness: 0.0, contrast: 0.0, saturation: 1.0 };

    /// The three values clamped into their ranges; a value that is not a number reads neutral.
    pub fn clamped(self) -> Self {
        let f = |v: f32, r: std::ops::RangeInclusive<f32>, neutral: f32| if v.is_finite() { v.clamp(*r.start(), *r.end()) } else { neutral };
        PaletteAdjust {
            brightness: f(self.brightness, BRIGHTNESS_RANGE, 0.0),
            contrast: f(self.contrast, CONTRAST_RANGE, 0.0),
            saturation: f(self.saturation, SATURATION_RANGE, 1.0),
        }
    }

    /// Whether it changes nothing (then [`Self::apply`] leaves the table untouched).
    pub fn is_neutral(&self) -> bool {
        *self == Self::NEUTRAL
    }

    /// Adjust every entry of `lut` (alpha and the fetch mode kept). Neutral returns at once.
    pub fn apply(&self, lut: &mut Lut) {
        if self.is_neutral() {
            return;
        }
        let a = self.clamped();
        for e in &mut lut.entries {
            let [r, g, b] = a.color([e[0], e[1], e[2]]);
            e[0] = r;
            e[1] = g;
            e[2] = b;
        }
    }

    /// One display (sRGB-encoded, 0..1) color, adjusted.
    pub fn color(&self, rgb: [f32; 3]) -> [f32; 3] {
        let [l, a, b] = srgb_to_oklab([rgb[0] as f64, rgb[1] as f64, rgb[2] as f64]);
        let mut l = l.clamp(0.0, 1.0);
        if self.brightness != 0.0 {
            l = l.powf((-self.brightness as f64).exp2());
        }
        if self.contrast != 0.0 {
            let k = (1.5 * self.contrast as f64).exp2();
            let (p, q) = (l.powf(k), (1.0 - l).powf(k));
            l = if p + q > 0.0 { p / (p + q) } else { l };
        }
        let s = self.saturation as f64;
        let out = in_gamut(l, a * s, b * s);
        [out[0] as f32, out[1] as f32, out[2] as f32]
    }
}

/// The display color for OKLab `(l, a, b)`, giving up chroma (lightness and hue kept) until it is
/// inside the sRGB gamut.
fn in_gamut(l: f64, a: f64, b: f64) -> [f64; 3] {
    const EPS: f64 = 1e-7;
    let fits = |lin: [f64; 3]| lin.iter().all(|v| (-EPS..=1.0 + EPS).contains(v));
    let full = oklab_to_linear([l, a, b]);
    if fits(full) {
        return encode(full);
    }
    // Bisect the chroma scale: 0 (gray at this lightness) always fits.
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    for _ in 0..24 {
        let mid = 0.5 * (lo + hi);
        if fits(oklab_to_linear([l, a * mid, b * mid])) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    encode(oklab_to_linear([l, a * lo, b * lo]))
}

fn encode(lin: [f64; 3]) -> [f64; 3] {
    lin.map(|v| {
        let v = v.clamp(0.0, 1.0);
        if v <= 0.0031308 {
            12.92 * v
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        }
    })
}

/// sRGB-encoded display color (0..1) to OKLab (Björn Ottosson's matrices; the same as the tag
/// measurer's in `tags.rs`).
pub fn srgb_to_oklab(rgb: [f64; 3]) -> [f64; 3] {
    let lin = |v: f64| {
        let v = v.clamp(0.0, 1.0);
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let (r, g, b) = (lin(rgb[0]), lin(rgb[1]), lin(rgb[2]));
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}

/// OKLab to linear-light sRGB (not clamped: a channel outside 0..1 is out of gamut).
pub fn oklab_to_linear(lab: [f64; 3]) -> [f64; 3] {
    let [l, a, b] = lab;
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;
    let (l3, m3, s3) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
    [
        4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3,
        -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3,
        -0.0041960863 * l3 - 0.7034186147 * m3 + 1.7076147010 * s3,
    ]
}

/// OKLab to a display (sRGB-encoded) color, clamped into 0..1 channel by channel.
pub fn oklab_to_srgb(lab: [f64; 3]) -> [f64; 3] {
    encode(oklab_to_linear(lab))
}

#[cfg(test)]
#[path = "../tests/unit/adjust/tests.rs"]
mod tests;
