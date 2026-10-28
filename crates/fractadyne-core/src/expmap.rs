//! Exponential-map strips: their geometry (`design/zoom-video.md` §3).
//!
//! A zoom video dives straight into one centre. Rendering it frame by frame computes each annulus
//! around the centre once per frame it appears in; an exponential map renders each annulus ONCE, as
//! a strip whose columns are the angle around the centre and whose rows are the logarithm of the
//! distance from it (Munafo's exponential map, https://mrob.com/pub/muency/exponentialmap.html).
//! The map is conformal, so a strip's texels are square in the plane. Built from that published
//! maths only: zoomasm, Kalles Fraktaler and Fraktaler-3, which use the same idea, are AGPL, and
//! their source is never read (design §2).
//!
//! Fractadyne's own convention, used everywhere inside the app (a zoomasm-compatible writer
//! converts):
//!
//! - **Strip `k`** covers radii from `R₀·2^-k` (its row 0, outermost) inward by one octave plus
//!   [`EXPMAP_OVERLAP_ROWS`] rows, shared with strip `k + 1` so assembly can blend across the seam.
//!   `R₀` is half the diagonal of the dive's first frame: strip 0 reaches that frame's corners.
//! - **Texel `(i, j)`** of strip `k` is at angle `θ = 2π(i + ½)/W`, anticlockwise from the positive
//!   real axis, and radius `r = R₀·2^-k·exp(−2π(j + ½)/W)`; `W` is the strip's width in texels.
//! - `W = q·π·diagonal` (frame pixels): the circle through a frame's corners then gets `q` texels a
//!   pixel, and every other circle more, so the strips never undersample a frame.
//!
//! **Octave rows** ([`StripRows::Octave`], the layout Kalles Fraktaler writes and zoomasm reads):
//! strip `k`'s `rows` rows cover exactly one octave with no overlap, and texel `(i, j)` is sampled
//! at its CORNER, as KF samples it: `θ = 2πi/W`, `r = R₀·2^-k·2^(−j/rows)` (both measured from
//! KF 2.15.5's own strips, which match that to 1e-7 of an iteration). `rows` is the whole number
//! nearest `W·ln2/2π`, so a texel is square to within half a row in `rows` (0.2% at 640x360).

use crate::floatexp::FloatExp;
use crate::viewport::Viewport;
use std::f64::consts::{LN_2, PI, TAU};

/// Rows a conformal strip has beyond one octave, shared with the next strip.
pub const EXPMAP_OVERLAP_ROWS: u32 = 1;

/// How a strip's rows cover its octave of radius.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StripRows {
    /// Square texels (`ln r` falls `2π/W` a row, as the angle steps), one octave rounded up plus
    /// [`EXPMAP_OVERLAP_ROWS`]: Fractadyne's own strips.
    Conformal,
    /// Exactly one octave in `rows` rows (`ln r` falls `ln2/rows` a row), no overlap: Kalles
    /// Fraktaler's layout, which zoomasm reads.
    Octave,
}

/// The strips of one dive: their size and where strip 0 starts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExpMap {
    /// Texels around a full circle (`W`).
    pub width: u32,
    /// Rows in one strip: one octave ([`ExpMap::rows_per_octave`], rounded up) and the overlap,
    /// or exactly one octave ([`StripRows::Octave`]).
    pub rows: u32,
    /// `log2 R₀`: strip 0's outer radius, in c-plane units.
    pub r0_log2: f64,
    /// How the rows cover the octave.
    pub layout: StripRows,
}

/// Rows in one conformal strip of width `width`: one octave, rounded up, and the overlap.
pub fn expmap_rows(width: u32) -> u32 {
    (width as f64 * LN_2 / TAU).ceil() as u32 + EXPMAP_OVERLAP_ROWS
}

impl ExpMap {
    /// The strips for frames of `width_px × height_px`, `oversample` texels a pixel at the frame's
    /// corners, for a dive whose first frame is at `start_log2mag` (the app's magnification, so a
    /// frame's height spans `Viewport::REFERENCE_HEIGHT · 2^-start_log2mag`).
    pub fn for_frames(width_px: f64, height_px: f64, oversample: f64, start_log2mag: f64) -> Self {
        let diag = width_px.hypot(height_px);
        let width = (oversample.max(1.0) * PI * diag).ceil() as u32;
        // Half the first frame's diagonal: `upp·diag/2`, `upp = REFERENCE_HEIGHT / (h·2^mag)`.
        let r0_log2 = (Viewport::REFERENCE_HEIGHT * diag / (2.0 * height_px)).log2() - start_log2mag;
        ExpMap { width, rows: expmap_rows(width), r0_log2, layout: StripRows::Conformal }
    }

    /// The same strips with [`StripRows::Octave`] rows: the whole number of rows nearest one
    /// conformal octave, covering exactly one octave.
    pub fn with_octave_rows(self) -> Self {
        let rows = ((self.width as f64 * LN_2 / TAU).round() as u32).max(1);
        ExpMap { rows, layout: StripRows::Octave, ..self }
    }

    /// `2π/W`: the angle of one column (and, in a conformal strip, the row step too).
    pub fn step(&self) -> f64 {
        TAU / self.width as f64
    }

    /// The natural-log radius step of one row: `2π/W` (conformal) or `ln2/rows` (octave).
    pub fn row_step(&self) -> f64 {
        match self.layout {
            StripRows::Conformal => self.step(),
            StripRows::Octave => LN_2 / self.rows as f64,
        }
    }

    /// Rows in one octave of radius: `W·ln2/2π` for a conformal strip (not a whole number in
    /// general), `rows` for an octave strip.
    pub fn rows_per_octave(&self) -> f64 {
        LN_2 / self.row_step()
    }

    /// Where in its texel a strip samples, in texels from the texel's corner along both axes:
    /// `0.5` (the centre) for a conformal strip, `0.0` (the corner, as Kalles Fraktaler samples)
    /// for an octave strip.
    pub fn sample_offset(&self) -> f64 {
        match self.layout {
            StripRows::Conformal => 0.5,
            StripRows::Octave => 0.0,
        }
    }

    /// `log2` of strip `k`'s outer radius (its row 0's outer edge).
    pub fn outer_log2(&self, k: u64) -> f64 {
        self.r0_log2 - k as f64
    }

    /// Strip `k`'s outer radius as `mantissa · 2^exp`, `mantissa ∈ [1, 2)`: the GPU's `delta_exp` and
    /// the O(1) radius it scales each texel's offset by. Exact at any depth: the octave count goes
    /// into the exponent as an integer, never through an `f64` logarithm (at 50,000 octaves that
    /// alone cost 2e-12 of the radius).
    pub fn outer_scale(&self, k: u64) -> (f64, i32) {
        let e0 = self.r0_log2.floor();
        (2f64.powf(self.r0_log2 - e0), (e0 as i64 - k as i64) as i32)
    }

    /// The sample point of texel `(i, j)` of strip `k` in polar form: (`log2` of its radius, its
    /// angle in radians). `i` and `j` are texel indices; a fractional part moves the sample from
    /// where the layout puts it ([`ExpMap::sample_offset`]: the texel's centre, or its corner). The
    /// `log2` is an `f64`, so thousands of octaves deep it is good to ~1e-12 of the radius;
    /// [`ExpMap::texel_offset`] is exact.
    pub fn texel_polar(&self, k: u64, i: f64, j: f64) -> (f64, f64) {
        let o = self.sample_offset();
        (self.outer_log2(k) - self.row_step() * (j + o) / LN_2, self.step() * (i + o))
    }

    /// Texel `(i, j)`'s sample point as an offset from the dive's centre, `(re, im)`,
    /// extended-range.
    pub fn texel_offset(&self, k: u64, i: f64, j: f64) -> (FloatExp, FloatExp) {
        let o = self.sample_offset();
        let (m, e) = self.outer_scale(k);
        let r = FloatExp::new(m * (-self.row_step() * (j + o)).exp(), e);
        let th = self.step() * (i + o);
        (r.mul_f64(th.cos()), r.mul_f64(th.sin()))
    }

    /// The size of texel `(i, j)` of strip `k` in the plane (`r·2π/W`, its width), as `log2`.
    pub fn texel_size_log2(&self, k: u64, j: f64) -> f64 {
        self.texel_polar(k, 0.0, j).0 + self.step().log2()
    }

    /// How many strips a dive to frames at `end_log2mag` needs: one an octave from the first
    /// frame's corner down to the last frame's corner, then on to half a pixel of the last frame
    /// (`log2(diagonal)` octaves more), so the last frame's centre is covered too.
    pub fn strips_to(&self, end_log2mag: f64, height_px: f64) -> u64 {
        let half_pixel_log2 = (Viewport::REFERENCE_HEIGHT / height_px).log2() - end_log2mag - 1.0;
        (self.r0_log2 - half_pixel_log2).max(1.0).ceil() as u64
    }

    /// The magnification whose frame's corners are strip `k`'s outer radius (`log2`): the frame
    /// where strip `k` first appears. Each strip is one octave deeper than the one before.
    pub fn frame_log2mag(&self, k: u64, width_px: f64, height_px: f64) -> f64 {
        let diag = width_px.hypot(height_px);
        (Viewport::REFERENCE_HEIGHT * diag / (2.0 * height_px)).log2() - self.outer_log2(k)
    }
}

#[cfg(test)]
#[path = "../tests/unit/expmap/tests.rs"]
mod tests;
