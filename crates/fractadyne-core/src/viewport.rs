//! The [`Viewport`] — an arbitrary-precision view center plus a `FloatExp` scale — and the
//! coordinate math (pan / zoom / lerp, GPU scale params). Uses `bignum` for the center and
//! `floatexp` for the extended-range scale that survives past f64's ~1e308× at extreme zoom.

use crate::bignum::*;
use crate::floatexp::*;
use astro_float::BigFloat;

/// GPU scale parameters for a view: the shared base-2 exponent and the span *mantissas*
/// (`span · 2^-delta_exp`, always O(1)), computed without ever forming the raw span (which
/// underflows `f64` past ~1e308×). The GPU builds its per-texel step as
/// `span_mantissa / texdim` and passes `delta_exp` to the shader unchanged.
#[derive(Clone, Copy, Debug)]
pub struct GpuScale {
    pub delta_exp: i32,
    pub span_mantissa: SpanMantissa,
}

/// The complex view span as an O(1) mantissa per axis (`span · 2^-delta_exp`): `x` (width) and `y`
/// (height). Newtyped so an axis mix-up is a named-field bug, not a silent `[0]`/`[1]` index swap;
/// `y` is derived as `x · aspect`. It never crosses a GPU/Pod/WGSL boundary (it's lowered CPU-side
/// into the per-texel `step`), so it's a plain public-axis struct with no layout constraint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpanMantissa {
    pub x: f64,
    pub y: f64,
}

impl SpanMantissa {
    #[inline]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// `(center − reference) · 2^-delta_exp` as an O(1) `f64` mantissa — the per-view reference
/// offset for the GPU, computed in bignum then exponent-shifted so it never underflows.
pub fn ref_offset_mantissa(center: &BigFloat, reference: &BigFloat, delta_exp: i32, p: usize) -> f64 {
    let mut d = center.sub(reference, p, RM);
    if let Some(e) = d.exponent() {
        d.set_exponent(e - delta_exp);
    }
    to_f64(&d)
}

/// A rectangular view into the complex plane.
#[derive(Clone, Debug)]
pub struct Viewport {
    pub center_x: BigFloat,
    pub center_y: BigFloat,
    /// Complex-plane units per pixel (isotropic), as an extended-range `FloatExp` so it
    /// does not underflow past ~1e308× zoom. Smaller ⇒ deeper zoom.
    pub units_per_pixel: FloatExp,
    pub width_px: f64,
    pub height_px: f64,
    /// Mantissa bits for the center (grows with zoom).
    pub precision: usize,
}

impl Viewport {
    /// Complex-plane height of the home view (magnification = 1). Chosen as 4.0 so our
    /// magnification EQUALS the Kalles Fraktaler / Fraktaler-3 "zoom" — that community references a
    /// vertical extent of 4 at zoom 1 — removing the old 4/3 conversion (a source of cross-app
    /// friction). `magnification = REFERENCE_HEIGHT / (height_px · units_per_pixel)`.
    pub const REFERENCE_HEIGHT: f64 = 4.0;

    pub fn new(width_px: f64, height_px: f64) -> Self {
        let height_px = height_px.max(1.0);
        let precision = 64;
        Self {
            center_x: bf(-0.5, precision),
            center_y: bf(0.0, precision),
            units_per_pixel: FloatExp::from_f64(Self::REFERENCE_HEIGHT / height_px),
            width_px: width_px.max(1.0),
            height_px,
            precision,
        }
    }

    pub fn set_size(&mut self, width_px: f64, height_px: f64) {
        self.width_px = width_px.max(1.0);
        self.height_px = height_px.max(1.0);
    }

    /// Restore the default view (center and zoom), keeping the current size.
    pub fn reset(&mut self) {
        self.precision = 64;
        self.center_x = bf(-0.5, self.precision);
        self.center_y = bf(0.0, self.precision);
        self.units_per_pixel = FloatExp::from_f64(Self::REFERENCE_HEIGHT / self.height_px);
    }

    fn refresh_precision(&mut self) {
        self.precision = precision_for_octaves(self.log2_magnification().max(0.0).ceil() as u64);
    }

    /// Complex offset of a pixel from the view center, as a `BigFloat` (extended-range scale).
    fn pixel_offset(&self, dpx: f64) -> BigFloat {
        self.units_per_pixel.mul_f64(dpx).to_bf(self.precision)
    }

    /// Complex coordinate under a pixel (origin top-left, +y down on screen).
    pub fn pixel_to_complex(&self, px: f64, py: f64) -> (BigFloat, BigFloat) {
        let p = self.precision;
        let ox = self.pixel_offset(px - self.width_px * 0.5);
        let oy = self.pixel_offset(py - self.height_px * 0.5);
        (self.center_x.add(&ox, p, RM), self.center_y.sub(&oy, p, RM))
    }

    /// Screen pixel a complex coordinate maps to (inverse of `pixel_to_complex`; origin top-left,
    /// +y down). Stays exact at any depth: the offset from centre is formed as a mantissa scaled by
    /// the `units_per_pixel` exponent (so tiny deep-zoom deltas don't underflow f64). Used to anchor
    /// tour callouts to a fractal coordinate so they track the point as the view pans/zooms.
    pub fn complex_to_pixel(&self, cx: &BigFloat, cy: &BigFloat) -> (f64, f64) {
        let (m, e) = (self.units_per_pixel.m, self.units_per_pixel.e);
        // ref_offset_mantissa(a, b, e) = (a − b)·2^−e; dividing by the mantissa `m` gives (a−b)/upp.
        let dx = ref_offset_mantissa(cx, &self.center_x, e, self.precision) / m;
        let dy = ref_offset_mantissa(cy, &self.center_y, e, self.precision) / m;
        (self.width_px * 0.5 + dx, self.height_px * 0.5 - dy)
    }

    pub fn pan_pixels(&mut self, dx: f64, dy: f64) {
        let p = self.precision;
        let ox = self.pixel_offset(dx);
        let oy = self.pixel_offset(dy);
        self.center_x = self.center_x.sub(&ox, p, RM);
        self.center_y = self.center_y.add(&oy, p, RM);
    }

    /// Zoom by `factor` (< 1 zooms in) keeping the complex point under `(px,py)` fixed.
    pub fn zoom_at(&mut self, px: f64, py: f64, factor: f64) {
        let (cx, cy) = self.pixel_to_complex(px, py);
        self.units_per_pixel = self.units_per_pixel.mul_f64(factor);
        self.refresh_precision();
        let p = self.precision;
        let ox = self.pixel_offset(px - self.width_px * 0.5);
        let oy = self.pixel_offset(py - self.height_px * 0.5);
        self.center_x = cx.sub(&ox, p, RM);
        self.center_y = cy.add(&oy, p, RM);
    }

    /// Zoom so the pixel rectangle fits the view; its center becomes the view center.
    pub fn zoom_to_rect(&mut self, px0: f64, py0: f64, px1: f64, py1: f64) {
        let (cx, cy) = self.pixel_to_complex((px0 + px1) * 0.5, (py0 + py1) * 0.5);
        let box_w = (px1 - px0).abs().max(1.0);
        let box_h = (py1 - py0).abs().max(1.0);
        // upp *= max(box_w/width, box_h/height) (the shared upp factor cancels in the ratio).
        let t = (box_w / self.width_px).max(box_h / self.height_px);
        self.units_per_pixel = self.units_per_pixel.mul_f64(t);
        self.refresh_precision();
        self.center_x = cx;
        self.center_y = cy;
    }

    /// One frame of a smooth "zoom out to home" animation.
    ///
    /// Sets the magnification to `exp(new_logmag)` (so `new_logmag == 0` is home,
    /// 1×) and glides the center from `start_center` toward `home`. The center
    /// fraction is `1 - 1/magnification`, which keeps the original focal point a
    /// roughly fixed distance from screen-center throughout the zoom-out (a plain
    /// linear lerp would fling it off-screen at depth, since on-screen distance
    /// scales with magnification). At `new_logmag == 0` the center is exactly `home`.
    pub fn home_lerp(
        &mut self,
        home: (f64, f64),
        start_center: &(BigFloat, BigFloat),
        new_logmag: f64,
    ) {
        let upp_home = Self::REFERENCE_HEIGHT / self.height_px;
        // upp = upp_home · e^(−logmag) = upp_home · 2^(−logmag/ln2) (extended range).
        self.units_per_pixel =
            FloatExp::from_f64(upp_home).mul_pow2(-new_logmag / std::f64::consts::LN_2);
        self.refresh_precision();
        let p = self.precision;
        let frac = 1.0 - (-new_logmag).exp(); // 0 at home → ~1 when deep
        let hx = bf(home.0, p);
        let hy = bf(home.1, p);
        let f = bf(frac, p);
        // center = home + (start - home) * frac
        self.center_x = hx.add(&start_center.0.sub(&hx, p, RM).mul(&f, p, RM), p, RM);
        self.center_y = hy.add(&start_center.1.sub(&hy, p, RM).mul(&f, p, RM), p, RM);
    }

    /// Set the view to an explicit center and magnification (1× = home framing). `mag` is
    /// `f64`, so this reaches ~1e308×; for deeper jumps use [`set_center_log2mag`]. Used by
    /// the scripting / benchmark camera player.
    pub fn set_center_mag(&mut self, cx: BigFloat, cy: BigFloat, mag: f64) {
        self.units_per_pixel =
            FloatExp::from_f64(Self::REFERENCE_HEIGHT / self.height_px).mul_f64(1.0 / mag.max(1.0e-300));
        self.refresh_precision();
        self.center_x = cx;
        self.center_y = cy;
    }

    /// Set center + magnification by `log2(magnification)` — reaches arbitrary depth (past
    /// `f64`'s 1e308× limit), unlike [`set_center_mag`].
    pub fn set_center_log2mag(&mut self, cx: BigFloat, cy: BigFloat, log2mag: f64) {
        self.units_per_pixel =
            FloatExp::from_f64(Self::REFERENCE_HEIGHT / self.height_px).mul_pow2(-log2mag);
        self.refresh_precision();
        self.center_x = cx;
        self.center_y = cy;
    }

    /// Complex span (width, height), as `f64` — saturates to `0` past ~1e308×. Use
    /// [`Viewport::complex_span_fe`] for the extended-range value the deep render needs.
    pub fn complex_span(&self) -> (f64, f64) {
        let (sx, sy) = self.complex_span_fe();
        (sx.to_f64(), sy.to_f64())
    }

    /// Complex span (width, height) as extended-range `FloatExp` (no underflow at any depth).
    pub fn complex_span_fe(&self) -> (FloatExp, FloatExp) {
        (
            self.units_per_pixel.mul_f64(self.width_px),
            self.units_per_pixel.mul_f64(self.height_px),
        )
    }

    /// GPU scale: shared base-2 exponent + span mantissas (`span · 2^-delta_exp`, O(1)).
    pub fn gpu_scale(&self) -> GpuScale {
        let (sx, sy) = self.complex_span_fe();
        let delta_exp = if sx.m == 0.0 { 0 } else { sx.log2().floor() as i32 };
        let s = -(delta_exp as f64);
        GpuScale {
            delta_exp,
            span_mantissa: SpanMantissa::new(sx.mul_pow2(s).to_f64(), sy.mul_pow2(s).to_f64()),
        }
    }

    /// `log2` of the magnification — finite at any depth (unlike `magnification()`, which
    /// saturates to `∞` past ~1e308×). `magnification = REFERENCE_HEIGHT / (height · upp)`.
    pub fn log2_magnification(&self) -> f64 {
        (Self::REFERENCE_HEIGHT / self.height_px).log2() - self.units_per_pixel.log2()
    }

    pub fn magnification(&self) -> f64 {
        (FloatExp::from_f64(Self::REFERENCE_HEIGHT / self.height_px)
            * self.units_per_pixel.recip())
        .to_f64()
    }

    pub fn recommended_max_iter(&self, base: u32) -> u32 {
        let octaves = self.log2_magnification().max(0.0);
        // The iteration count a given depth genuinely wants (~220 per octave). This is the
        // *export* / full-quality appetite; the live preview caps it lower (see `build_params`)
        // for responsiveness, so deep views can look smoother on screen than in an export.
        //
        // The 2M ceiling bounds AUTO mode only (a manual slider value passes through as `base`,
        // up to the app's `MAX_ITER_LIMIT`): it keeps an auto-iter export at extreme depth
        // (e21000-class, where the formula asks for ~15M) from a runaway reference build, while
        // no longer starving deep dense fields the way the old 500k ceiling did (the 2.6e72×
        // spar needs ~1M to resolve; measured 33% capped at 500k, 0% at 1M).
        (base + (octaves * 220.0) as u32).min(2_000_000)
    }

    /// Center as `f64` (for display / coarse use).
    pub fn center_f64(&self) -> (f64, f64) {
        (to_f64(&self.center_x), to_f64(&self.center_y))
    }

    /// Complex coordinate under a pixel, as `f64` (for display; +y down on screen).
    pub fn complex_at_pixel_f64(&self, px: f64, py: f64) -> (f64, f64) {
        let (cx, cy) = self.center_f64();
        let upp = self.units_per_pixel.to_f64();
        (
            cx + (px - self.width_px * 0.5) * upp,
            cy - (py - self.height_px * 0.5) * upp,
        )
    }
}
