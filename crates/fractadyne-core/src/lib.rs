//! Core numerics for Fractadyne.
//!
//! The viewport center is **arbitrary precision** (`astro_float::BigFloat`) at a
//! mantissa size that scales with zoom, so position stays sub-pixel at *any* depth
//! (no coordinate jump, ever). `units_per_pixel` is a plain f64 scale. The reference
//! orbit is iterated in bignum and stored as `f32` hi/lo pairs (df64) for the GPU.
//!
//! Bignum is slow, so the reference orbit should be recomputed only when the
//! reference point changes (the app caches it), not every frame.
//!
//! ## Naming conventions (glossary)
//!
//! Short identifiers recur throughout this crate; they mean:
//! - `p` — working **precision** in mantissa bits for a `BigFloat` (scales with zoom depth).
//! - `bf` — a `BigFloat`, or a shorthand constructor from an f64.
//! - `(cx, cy)` — the complex parameter *c* (real, imaginary). `(zx, zy)` — the iterate *z*.
//! - `(dzx, dzy)` / `dc` — perturbation **deltas** (δz, δc) relative to the reference orbit.
//! - `FloatExp` (`m`,`e`) — an extended-range float `m·2^e` (see the `FloatExp` docs); `df64`/df32
//!   — an f32 hi+lo pair (double-single) carrying ~46 bits for the GPU.
//! - `RM` — `RoundingMode`; `SA` — series approximation; `BLA` — bivariate linear approximation.

pub use astro_float::BigFloat;
use astro_float::{RoundingMode, Sign};

const RM: RoundingMode = RoundingMode::None;

/// Fast `BigFloat` → `f64` (correctly rounded). Replicates astro-float's internal
/// `to_f64` (which is test-only) from the public mantissa/exponent/sign accessors.
/// `Word` is `u64` on 64-bit targets; the most-significant word is the last one.
pub fn to_f64(bf: &BigFloat) -> f64 {
    let digits = match bf.mantissa_digits() {
        Some(d) if !d.is_empty() => d,
        _ => return 0.0,
    };
    let exp = match bf.exponent() {
        Some(e) => e as i64,
        None => return 0.0,
    };
    let neg = matches!(bf.sign(), Some(Sign::Neg));
    let mantissa = *digits.last().unwrap(); // top 64 bits (normalized MSW)
    if mantissa == 0 {
        return 0.0;
    }
    let mut e: i64 = exp + 1023;
    let mut ret: u64 = 0;
    if e >= 2047 {
        if neg {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        }
    } else if e <= 0 {
        let shift = -e;
        if shift < 52 {
            ret |= mantissa >> (shift as u64 + 12);
            if neg {
                ret |= 0x8000_0000_0000_0000u64;
            }
            f64::from_bits(ret)
        } else {
            0.0
        }
    } else {
        let m = mantissa << 1;
        e -= 1;
        if neg {
            ret |= 1;
        }
        ret <<= 11;
        ret |= e as u64;
        ret <<= 52;
        ret |= m >> 12;
        f64::from_bits(ret)
    }
}

/// Full-precision decimal string of a `BigFloat` (for export metadata).
pub fn to_decimal_string(bf: &BigFloat) -> String {
    bf.to_string()
}

/// Parse a decimal string back to a `BigFloat` (round-trips `to_decimal_string`). Rejects
/// non-finite results (NaN / ±∞) so a malformed or out-of-range coordinate can't slip
/// through — callers treat `None` as "invalid input".
pub fn parse_bf(s: &str) -> Option<BigFloat> {
    s.trim()
        .parse::<BigFloat>()
        .ok()
        .filter(|b| !b.is_nan() && !b.is_inf())
}

/// A location parsed from a Kalles Fraktaler `.kfr` file: full-precision center, zoom
/// (magnification), and optional iteration count.
pub struct KfrView {
    pub cx: BigFloat,
    pub cy: BigFloat,
    pub zoom: f64,
    pub iterations: Option<u32>,
}

/// Parse a positive, finite zoom from a `.kfr` `Zoom:` value; an over-range value (e.g.
/// `1E1000`) is clamped rather than rejected.
fn parse_kfr_zoom(s: &str) -> Option<f64> {
    let z: f64 = s.trim().parse().ok()?;
    if z.is_nan() || z <= 0.0 {
        return None;
    }
    Some(z.min(1.0e300)) // inf.min ⇒ clamp; matches the viewport's f64 upp range
}

/// Parse a **Kalles Fraktaler `.kfr`** location (a simple `Key: value` text format) into a
/// [`KfrView`]. **Hardened for untrusted input:** total size and line/value lengths are
/// bounded, only the `Re`/`Im`/`Zoom`/`Iterations` keys are read (everything else ignored —
/// no formulas, paths, or code), the center is validated through [`parse_bf`] (rejecting
/// non-finite), and zoom/iterations are clamped. Returns `None` unless a valid Re, Im, and
/// Zoom are present.
pub fn parse_kfr(text: &str) -> Option<KfrView> {
    if text.len() > 4_000_000 {
        return None; // refuse absurdly large inputs
    }
    let (mut re, mut im, mut zoom, mut iters) = (None, None, None, None);
    for line in text.lines().take(20_000) {
        let Some((key, val)) = line.split_once(':') else {
            continue;
        };
        let (key, val) = (key.trim(), val.trim());
        if val.len() > 100_000 {
            continue; // bounded value length (deep coords are long, but not unbounded)
        }
        if key.eq_ignore_ascii_case("Re") {
            re = Some(val);
        } else if key.eq_ignore_ascii_case("Im") {
            im = Some(val);
        } else if key.eq_ignore_ascii_case("Zoom") {
            zoom = parse_kfr_zoom(val);
        } else if key.eq_ignore_ascii_case("Iterations") {
            iters = val.parse::<u64>().ok().map(|v| v.min(1_000_000) as u32);
        }
        // every other key is ignored by design
    }
    Some(KfrView {
        cx: parse_bf(re?)?,
        cy: parse_bf(im?)?,
        zoom: zoom?,
        iterations: iters,
    })
}

/// Mantissa bits needed to position sub-pixel at the given magnification (+ guard).
///
/// Note: `mag` is `f64`, so this saturates near `1e308×` (and the viewport's `f64`
/// `units_per_pixel` is the actual render-depth ceiling). To reason about precision past
/// that — e.g. for extreme-depth *arithmetic* validation — use [`precision_for_octaves`].
pub fn precision_for_magnification(mag: f64) -> usize {
    let octaves = mag.max(1.0).log2().ceil() as usize;
    (octaves + 64).max(64)
}

/// Mantissa bits for a magnification of `2^octaves` (+ 64 guard bits). Unlike
/// [`precision_for_magnification`] this takes the octave count directly, so it stays valid
/// for magnifications far beyond `f64` range (e.g. `1e1000000×` ≈ 3.32M octaves → ~3.32M
/// bits) — used by the extreme-depth validation battery.
pub fn precision_for_octaves(octaves: u64) -> usize {
    (octaves as usize).saturating_add(64).max(64)
}

/// Leading base-2 bits of agreement between two `BigFloat`s: ≈ `−log₂(|a−b| / |b|)`.
/// Returns `p` when they match to the working precision (the difference rounds to zero).
fn agree_bits(a: &BigFloat, b: &BigFloat, p: usize) -> i64 {
    let diff = a.sub(b, p, RM);
    let ed = match diff.exponent() {
        Some(e) => e as i64,
        None => return p as i64, // difference is exactly zero → full agreement
    };
    let eb = b.exponent().map(|e| e as i64).unwrap_or(0);
    (eb - ed).max(0)
}

/// A full-mantissa interior point (inside the main cardioid, so it never escapes), seeded
/// by `√½` so every limb is populated — exercising real carries in the bignum multiply
/// rather than the sparse mantissas a dyadic point like `c = −0.5` would produce.
fn deep_test_point(p: usize) -> (BigFloat, BigFloat) {
    let s = BigFloat::from_f64(0.5, p).sqrt(p, RM); // 0.70710678… (irrational ⇒ full mantissa)
    let cx = s
        .mul(&BigFloat::from_f64(0.3, p), p, RM)
        .sub(&BigFloat::from_f64(0.5, p), p, RM); // ≈ −0.288 (interior)
    let cy = s.mul(&BigFloat::from_f64(0.01, p), p, RM); // ≈ 0.0071
    (cx, cy)
}

fn iter_zsq_c(cx: &BigFloat, cy: &BigFloat, k: u32, p: usize) -> (BigFloat, BigFloat) {
    let mut zx = BigFloat::from_f64(0.0, p);
    let mut zy = BigFloat::from_f64(0.0, p);
    for _ in 0..k {
        let x2 = zx.mul(&zx, p, RM);
        let y2 = zy.mul(&zy, p, RM);
        let nzy = double_bf(&zx.mul(&zy, p, RM)).add(cy, p, RM);
        zx = x2.sub(&y2, p, RM).add(cx, p, RM);
        zy = nzy;
    }
    (zx, zy)
}

/// Extreme-depth **precision self-consistency** probe (needs no external oracle): iterate
/// `z²+c` for `k` steps from a full-mantissa interior point at precision `p`, and again at
/// `p + guard` bits, then return how many leading base-2 bits of the result agree.
///
/// This is the standard precision-doubling validation technique: if the `p`-bit answer is
/// stable under a precision increase it is almost certainly correct. Sound `p`-bit
/// arithmetic gives agreement ≈ `p − log₂(k)`; a precision-propagation or arithmetic bug at
/// that bit-width collapses it. Feasible to any depth (cost ∝ `k · M(p)` with FFT multiply),
/// unlike a per-pixel dwell oracle.
pub fn deep_consistency_bits(p: usize, guard: usize, k: u32) -> i64 {
    let pg = p + guard;
    let (cx, cy) = deep_test_point(pg);
    let lo = iter_zsq_c(&cx, &cy, k, p);
    let hi = iter_zsq_c(&cx, &cy, k, pg);
    agree_bits(&lo.0, &hi.0, pg).min(agree_bits(&lo.1, &hi.1, pg))
}

/// Round-trip a full-mantissa coordinate through decimal at precision `p`
/// (`to_decimal_string` → `parse_bf`) and return the bits of agreement — validates the
/// persisted-coordinate I/O path (the deep-zoom save/restore format) at extreme precision.
pub fn deep_roundtrip_bits(p: usize) -> i64 {
    let (cx, _) = deep_test_point(p);
    match parse_bf(&to_decimal_string(&cx)) {
        Some(back) => agree_bits(&cx, &back, p),
        None => -1,
    }
}

fn bf(v: f64, p: usize) -> BigFloat {
    BigFloat::from_f64(v, p)
}

/// Exact multiply-by-two via a base-2 exponent bump — far cheaper than the full bignum multiply
/// that was forming `2xy` every reference-orbit iteration. Exact (no rounding); zero stays zero.
fn double_bf(x: &BigFloat) -> BigFloat {
    let mut d = x.clone();
    if let Some(e) = d.exponent() {
        d.set_exponent(e + 1);
    }
    d
}

/// Multiply a bignum by a small non-negative integer via shift-and-add (doublings + adds) — much
/// cheaper than a full-precision multiply by a constant `BigFloat`. Used in the hot series-
/// approximation recurrence, whose factors (`d`, `C(d,2)`, `2·C(d,2)`, `C(d,3)`) are all small ints.
fn mul_u32_bf(x: &BigFloat, n: u32, p: usize) -> BigFloat {
    if n == 0 {
        return bf(0.0, p);
    }
    let mut acc: Option<BigFloat> = None;
    let mut base = x.clone(); // x · 2^k
    let mut k = n;
    loop {
        if k & 1 == 1 {
            acc = Some(match acc {
                Some(a) => a.add(&base, p, RM),
                None => base.clone(),
            });
        }
        k >>= 1;
        if k == 0 {
            break;
        }
        base = double_bf(&base);
    }
    acc.unwrap()
}

/// Linear interpolation between two `BigFloat`s at precision `p`: `a + (b − a)·t`.
pub fn lerp_bf(a: &BigFloat, b: &BigFloat, t: f64, p: usize) -> BigFloat {
    let f = bf(t, p);
    a.add(&b.sub(a, p, RM).mul(&f, p, RM), p, RM)
}

// ---------------- extended-range scale (FloatExp) ---------------------------------

fn ldexp_f64(m: f64, e: i32) -> f64 {
    if e >= -1022 {
        m * 2f64.powi(e.min(1023))
    } else {
        // Subnormal range: split the shift so neither factor overflows/underflows alone.
        m * 2f64.powi(-1022) * 2f64.powi((e + 1022).max(-1074))
    }
}

/// A real number `m · 2^e` with an `i32` base-2 exponent, so its magnitude reaches far past
/// `f64`'s ~1e±308 range. Used for the viewport scale (`units_per_pixel`), which a plain
/// `f64` underflows at extreme zoom. The mantissa is normalized to `|m| ∈ [1, 2)` (carrying
/// the sign), or exactly `0`. Inputs are always well-conditioned (a normalized mantissa
/// times a pixel count or zoom factor), so normalization needs only a small shift.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatExp {
    pub m: f64,
    pub e: i32,
}

impl FloatExp {
    pub const ZERO: FloatExp = FloatExp { m: 0.0, e: 0 };

    fn norm(mut m: f64, mut e: i32) -> FloatExp {
        if m == 0.0 || !m.is_finite() {
            return FloatExp { m, e: 0 };
        }
        let a = m.abs();
        if !(1.0..2.0).contains(&a) {
            let k = a.log2().floor() as i32;
            m *= 2f64.powi(-k);
            e += k;
            while m.abs() >= 2.0 {
                m *= 0.5;
                e += 1;
            }
            while m.abs() < 1.0 {
                m *= 2.0;
                e -= 1;
            }
        }
        FloatExp { m, e }
    }

    pub fn from_f64(v: f64) -> FloatExp {
        FloatExp::norm(v, 0)
    }

    /// Construct from a raw `(mantissa, exponent)` pair, normalizing. Used to reload a
    /// persisted scale (`m`, `e` stored separately so it survives past `f64` range).
    pub fn new(m: f64, e: i32) -> FloatExp {
        FloatExp::norm(m, e)
    }

    /// Saturating conversion to `f64` (`0` on underflow, `±∞` on overflow).
    pub fn to_f64(self) -> f64 {
        if self.m == 0.0 {
            0.0
        } else if self.e > 1023 {
            self.m * f64::INFINITY
        } else if self.e < -1074 {
            0.0
        } else {
            ldexp_f64(self.m, self.e)
        }
    }

    pub fn mul_f64(self, k: f64) -> FloatExp {
        FloatExp::norm(self.m * k, self.e)
    }

    // NOTE: these inherent `mul`/`add`/`sub` methods are intentionally NOT `std::ops` impls yet.
    // Migrating the ~100 precedence-sensitive call sites (`a.add(b).mul(c)` ≠ `a + b * c`) to
    // operators is a Phase 1 readability task done alongside the `floatexp` module extraction — see
    // REFACTOR-PLAN.md. Allowed per-method (not crate-wide) so the lint keeps guarding new code.
    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, o: FloatExp) -> FloatExp {
        FloatExp::norm(self.m * o.m, self.e + o.e)
    }

    pub fn recip(self) -> FloatExp {
        FloatExp::norm(1.0 / self.m, -self.e)
    }

    /// Sum, aligning to the larger exponent (the smaller is dropped past f64's ~53 bits).
    #[allow(clippy::should_implement_trait)] // see the note on `mul` above (Phase 1 ops-trait migration)
    pub fn add(self, o: FloatExp) -> FloatExp {
        if self.m == 0.0 {
            return o;
        }
        if o.m == 0.0 {
            return self;
        }
        let (hi, lo) = if self.e >= o.e { (self, o) } else { (o, self) };
        let de = hi.e - lo.e;
        if de > 120 {
            return hi;
        }
        FloatExp::norm(hi.m + lo.m * 2f64.powi(-de), hi.e)
    }

    #[allow(clippy::should_implement_trait)] // see the note on `mul` above (Phase 1 ops-trait migration)
    pub fn sub(self, o: FloatExp) -> FloatExp {
        self.add(FloatExp { m: -o.m, e: o.e })
    }

    /// Magnitude (drops the sign).
    pub fn abs(self) -> FloatExp {
        FloatExp { m: self.m.abs(), e: self.e }
    }

    /// Square root (`≥ 0` inputs; clamps negatives/zero to `0`).
    pub fn sqrt(self) -> FloatExp {
        if self.m <= 0.0 {
            return FloatExp::ZERO;
        }
        if self.e & 1 == 0 {
            FloatExp::norm(self.m.sqrt(), self.e / 2)
        } else {
            // odd exponent: pull one power of two under the root (works for e<0 too).
            FloatExp::norm((self.m * 2.0).sqrt(), (self.e - 1) / 2)
        }
    }

    /// Signed value comparison (`self < o`).
    pub fn lt(self, o: FloatExp) -> bool {
        self.sub(o).m < 0.0
    }

    /// GPU upload form: `(mantissa f32, exponent)` — mantissa is normalized to `[1,2)` (or 0).
    pub fn to_f32_exp(self) -> (f32, i32) {
        (self.m as f32, self.e)
    }

    /// `log2` of the magnitude (finite for any representable value; `−∞` for `0`).
    pub fn log2(self) -> f64 {
        if self.m == 0.0 {
            f64::NEG_INFINITY
        } else {
            self.m.abs().log2() + self.e as f64
        }
    }

    /// Multiply by `2^x` for an arbitrary `f64` exponent (split into integer + fraction).
    pub fn mul_pow2(self, x: f64) -> FloatExp {
        let fl = x.floor();
        FloatExp::norm(self.m * 2f64.powf(x - fl), self.e + fl as i32)
    }

    /// As a `BigFloat` at precision `p` (exact — mantissa scaled by `2^e` via the exponent).
    pub fn to_bf(self, p: usize) -> BigFloat {
        let mut b = BigFloat::from_f64(self.m, p);
        if self.m != 0.0 {
            if let Some(be) = b.exponent() {
                b.set_exponent(be + self.e);
            }
        }
        b
    }
}

/// GPU scale parameters for a view: the shared base-2 exponent and the span *mantissas*
/// (`span · 2^-delta_exp`, always O(1)), computed without ever forming the raw span (which
/// underflows `f64` past ~1e308×). The GPU builds its per-texel step as
/// `span_mantissa / texdim` and passes `delta_exp` to the shader unchanged.
#[derive(Clone, Copy, Debug)]
pub struct GpuScale {
    pub delta_exp: i32,
    pub span_mantissa: [f64; 2],
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
    pub const REFERENCE_HEIGHT: f64 = 3.0;

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
            span_mantissa: [sx.mul_pow2(s).to_f64(), sy.mul_pow2(s).to_f64()],
        }
    }

    /// `log2` of the magnification — finite at any depth (unlike `magnification()`, which
    /// saturates to `∞` past ~1e308×). `magnification = REFERENCE_HEIGHT / (height · upp)`.
    pub fn log2_magnification(&self) -> f64 {
        (Self::REFERENCE_HEIGHT / self.height_px).log2() - self.units_per_pixel.log2()
    }

    pub fn magnification(&self) -> f64 {
        FloatExp::from_f64(Self::REFERENCE_HEIGHT / self.height_px)
            .mul(self.units_per_pixel.recip())
            .to_f64()
    }

    pub fn recommended_max_iter(&self, base: u32) -> u32 {
        let octaves = self.log2_magnification().max(0.0);
        // The iteration count a given depth genuinely wants (~220 per octave). This is the
        // *export* / full-quality appetite; the live preview caps it lower (see `build_params`)
        // for responsiveness, so deep views can look smoother on screen than in an export.
        (base + (octaves * 220.0) as u32).min(500_000)
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

/// Split an `f64` into a `(hi, lo)` `f32` pair (df64, ~14 digits).
fn split_df64(v: f64) -> (f32, f32) {
    let hi = v as f32;
    let lo = (v - hi as f64) as f32;
    (hi, lo)
}

/// Complex multiply in arbitrary precision: `(ax+i·ay)·(bx+i·by)`.
fn cmul_bf(
    ax: &BigFloat,
    ay: &BigFloat,
    bx: &BigFloat,
    by: &BigFloat,
    p: usize,
) -> (BigFloat, BigFloat) {
    let rx = ax.mul(bx, p, RM).sub(&ay.mul(by, p, RM), p, RM);
    let ry = ax.mul(by, p, RM).add(&ay.mul(bx, p, RM), p, RM);
    (rx, ry)
}

/// One reference iteration `Z → f(Z) + c` in arbitrary precision, per formula id
/// (must match the GPU shader). Only the analytic families that support perturbation
/// are handled; others fall back to z²+c (they use the direct path instead).
fn step_bf(
    zx: &BigFloat,
    zy: &BigFloat,
    cx: &BigFloat,
    cy: &BigFloat,
    formula: u32,
    p: usize,
) -> (BigFloat, BigFloat) {
    match formula {
        1 => {
            // z³ + c
            let (sx, sy) = cmul_bf(zx, zy, zx, zy, p);
            let (rx, ry) = cmul_bf(&sx, &sy, zx, zy, p);
            (rx.add(cx, p, RM), ry.add(cy, p, RM))
        }
        2 => {
            // z⁴ + c
            let (sx, sy) = cmul_bf(zx, zy, zx, zy, p);
            let (rx, ry) = cmul_bf(&sx, &sy, &sx, &sy, p);
            (rx.add(cx, p, RM), ry.add(cy, p, RM))
        }
        3 => {
            // z⁵ + c
            let (sx, sy) = cmul_bf(zx, zy, zx, zy, p);
            let (qx, qy) = cmul_bf(&sx, &sy, &sx, &sy, p);
            let (rx, ry) = cmul_bf(&qx, &qy, zx, zy, p);
            (rx.add(cx, p, RM), ry.add(cy, p, RM))
        }
        4 => {
            // Tricorn: conj(z)² + c = (x²−y²+cx, −2xy+cy)
            let x2 = zx.mul(zx, p, RM);
            let y2 = zy.mul(zy, p, RM);
            let txy = double_bf(&zx.mul(zy, p, RM));
            (x2.sub(&y2, p, RM).add(cx, p, RM), cy.sub(&txy, p, RM))
        }
        5 => {
            // Burning Ship: real = x²−y²+cx, imag = |2xy|+cy
            let x2 = zx.mul(zx, p, RM);
            let y2 = zy.mul(zy, p, RM);
            let xy2 = double_bf(&zx.mul(zy, p, RM));
            (x2.sub(&y2, p, RM).add(cx, p, RM), xy2.abs().add(cy, p, RM))
        }
        6 => {
            // Celtic: real = |x²−y²|+cx, imag = 2xy+cy
            let x2 = zx.mul(zx, p, RM);
            let y2 = zy.mul(zy, p, RM);
            let xy2 = double_bf(&zx.mul(zy, p, RM));
            (x2.sub(&y2, p, RM).abs().add(cx, p, RM), xy2.add(cy, p, RM))
        }
        7 => {
            // Buffalo: real = |x²−y²|+cx, imag = |2xy|+cy
            let x2 = zx.mul(zx, p, RM);
            let y2 = zy.mul(zy, p, RM);
            let xy2 = double_bf(&zx.mul(zy, p, RM));
            (x2.sub(&y2, p, RM).abs().add(cx, p, RM), xy2.abs().add(cy, p, RM))
        }
        _ => {
            // Mandelbrot: z² + c
            let x2 = zx.mul(zx, p, RM);
            let y2 = zy.mul(zy, p, RM);
            (
                x2.sub(&y2, p, RM).add(cx, p, RM),
                double_bf(&zx.mul(zy, p, RM)).add(cy, p, RM),
            )
        }
    }
}

/// One arbitrary-precision Phoenix step: `z' = z² + c − 0.5·z_prev` (p = −0.5). Kept separate from
/// [`step_bf`] because it needs the previous iterate `z_prev`, which the reference loop threads.
fn phoenix_step_bf(
    zx: &BigFloat,
    zy: &BigFloat,
    zpx: &BigFloat,
    zpy: &BigFloat,
    cx: &BigFloat,
    cy: &BigFloat,
    p: usize,
) -> (BigFloat, BigFloat) {
    let x2 = zx.mul(zx, p, RM);
    let y2 = zy.mul(zy, p, RM);
    let sx = x2.sub(&y2, p, RM);
    let sy = double_bf(&zx.mul(zy, p, RM));
    let half = BigFloat::from_f64(0.5, p);
    let hpx = zpx.mul(&half, p, RM);
    let hpy = zpy.mul(&half, p, RM);
    (sx.add(cx, p, RM).sub(&hpx, p, RM), sy.add(cy, p, RM).sub(&hpy, p, RM))
}

/// Compute the **orbit** of a point in `f64`, for the interactive orbit overlay.
/// Mirrors the shader's direct iteration per `formula` (ids match `formula_id`).
/// Returns the successive iterates `z₀, z₁, …` (including the start) until the
/// orbit escapes (`|z|² > bailout2`), Newton converges, or `max_points` is hit.
/// For escape-time families `z₀ = 0, c = point`; for Julia/Newton `z₀ = point`.
pub fn orbit_points(
    z0: (f64, f64),
    c: (f64, f64),
    formula: u32,
    max_points: usize,
    bailout2: f64,
) -> Vec<(f64, f64)> {
    let mut pts = Vec::with_capacity(max_points.min(1024));
    let (mut zx, mut zy) = z0;
    let (mut px, mut py) = (0.0_f64, 0.0_f64); // previous iterate (Phoenix)
    pts.push((zx, zy));
    for _ in 0..max_points {
        if formula == 9 {
            // Newton: z ← z − (z³−1)/(3z²); converges to a cube root of unity.
            let (z2x, z2y) = (zx * zx - zy * zy, 2.0 * zx * zy);
            let (z3x, z3y) = (z2x * zx - z2y * zy, z2x * zy + z2y * zx);
            let (fx, fy) = (z3x - 1.0, z3y);
            let (dx, dy) = (3.0 * z2x, 3.0 * z2y);
            let dd = dx * dx + dy * dy;
            if dd == 0.0 {
                break;
            }
            zx -= (fx * dx + fy * dy) / dd;
            zy -= (fy * dx - fx * dy) / dd;
            pts.push((zx, zy));
            if fx * fx + fy * fy < 1.0e-12 {
                break;
            }
            continue;
        }
        let (sx, sy) = (zx * zx - zy * zy, 2.0 * zx * zy); // z²
        let (nx, ny) = match formula {
            1 => (sx * zx - sy * zy + c.0, sx * zy + sy * zx + c.1), // z³
            2 => (sx * sx - sy * sy + c.0, 2.0 * sx * sy + c.1),     // z⁴
            3 => {
                let (qx, qy) = (sx * sx - sy * sy, 2.0 * sx * sy); // z⁴
                (qx * zx - qy * zy + c.0, qx * zy + qy * zx + c.1) // z⁵
            }
            4 => (sx + c.0, -sy + c.1),                           // Tricorn z̄²+c
            5 => (sx + c.0, sy.abs() + c.1),                      // Burning Ship
            6 => (sx.abs() + c.0, sy + c.1),                      // Celtic
            7 => (sx.abs() + c.0, sy.abs() + c.1),                // Buffalo
            8 => (sx + c.0 - 0.5 * px, sy + c.1 - 0.5 * py),      // Phoenix (p=−0.5)
            _ => (sx + c.0, sy + c.1),                            // Mandelbrot z²+c
        };
        px = zx;
        py = zy;
        zx = nx;
        zy = ny;
        pts.push((zx, zy));
        if zx * zx + zy * zy > bailout2 {
            break;
        }
    }
    pts
}

/// Compute a **reference orbit** starting at `Z₀ = (z0x, z0y)` with constant
/// `c = (cx, cy)`, iterated in arbitrary precision per `formula`, returned as df64
/// `[re.hi, im.hi, re.lo, im.lo]` pairs for the GPU. For the Mandelbrot-set view
/// `Z₀ = 0` and `c` is the reference point; for a Julia view `Z₀` is the reference
/// point and `c` is the Julia constant.
pub fn reference_orbit(
    z0x: &BigFloat,
    z0y: &BigFloat,
    cx: &BigFloat,
    cy: &BigFloat,
    formula: u32,
    max_iter: u32,
    p: usize,
) -> (Vec<[f32; 4]>, u32) {
    let mut out = Vec::with_capacity(max_iter as usize + 1);
    let mut zx = z0x.clone();
    let mut zy = z0y.clone();
    // Previous iterate, for Phoenix's two-term recurrence (unused by other formulas). Starts at 0.
    let mut zpx = BigFloat::from_f64(0.0, p);
    let mut zpy = BigFloat::from_f64(0.0, p);
    let (xh, xl) = split_df64(to_f64(&zx));
    let (yh, yl) = split_df64(to_f64(&zy));
    out.push([xh, yh, xl, yl]); // Z_0
    let mut n = 0u32;
    while n < max_iter {
        let (nzx, nzy) = if formula == 8 {
            phoenix_step_bf(&zx, &zy, &zpx, &zpy, cx, cy, p)
        } else {
            step_bf(&zx, &zy, cx, cy, formula, p)
        };
        if formula == 8 {
            // Shift z_prev ← z (before z ← z'); std::mem::replace avoids a bignum clone.
            zpx = std::mem::replace(&mut zx, nzx);
            zpy = std::mem::replace(&mut zy, nzy);
        } else {
            zx = nzx;
            zy = nzy;
        }
        let xv = to_f64(&zx);
        let yv = to_f64(&zy);
        let (xh, xl) = split_df64(xv);
        let (yh, yl) = split_df64(yv);
        out.push([xh, yh, xl, yl]);
        n += 1;
        if xv * xv + yv * yv > 1.0e12 {
            break;
        }
    }
    let len = out.len() as u32;
    (out, len)
}

/// Series-approximation skip: how many initial perturbation iterations can be replaced by a
/// polynomial, plus the order-3 coefficients at that point (as floatexp components for the
/// GPU). `δz_n ≈ A_n·δc + B_n·δc² + C_n·δc³`; the GPU seeds δz from this and starts iterating
/// at `skip`. `skip == 0` ⇒ not worth it / not applicable.
#[derive(Clone, Copy)]
pub struct SeriesSkip {
    pub skip: u32,
    /// Each coefficient as a complex df32 mantissa `[re_hi, re_lo, im_hi, im_lo]` × `2^exp`.
    pub a: [f32; 4],
    pub a_exp: i32,
    pub b: [f32; 4],
    pub b_exp: i32,
    pub c: [f32; 4],
    pub c_exp: i32,
}

impl SeriesSkip {
    pub const NONE: SeriesSkip = SeriesSkip {
        skip: 0,
        a: [0.0; 4],
        a_exp: 0,
        b: [0.0; 4],
        b_exp: 0,
        c: [0.0; 4],
        c_exp: 0,
    };
}

/// A complex bignum coefficient → shared-exponent floatexp `([re_hi,re_lo,im_hi,im_lo], exp)`.
fn coeff_to_fe(re: &BigFloat, im: &BigFloat) -> ([f32; 4], i32) {
    let e = match (re.exponent(), im.exponent()) {
        (None, None) => return ([0.0; 4], 0),
        (a, b) => a.unwrap_or(i32::MIN).max(b.unwrap_or(i32::MIN)),
    };
    let scaled = |v: &BigFloat| -> (f32, f32) {
        match v.exponent() {
            Some(ev) => {
                let mut s = v.clone();
                s.set_exponent(ev - e); // multiply by 2^-e → mantissa in [-2, 2)
                split_df64(to_f64(&s))
            }
            None => (0.0, 0.0),
        }
    };
    let (rh, rl) = scaled(re);
    let (ih, il) = scaled(im);
    ([rh, rl, ih, il], e)
}

/// `log2` of a complex bignum's magnitude (≈ max component exponent; `−∞` for zero).
fn log2_cmag(re: &BigFloat, im: &BigFloat) -> f64 {
    match (re.exponent(), im.exponent()) {
        (None, None) => f64::NEG_INFINITY,
        (a, b) => a.unwrap_or(i32::MIN).max(b.unwrap_or(i32::MIN)) as f64,
    }
}

/// Compute the [`SeriesSkip`] for a reference at `c = (cx, cy)` of the polynomial family
/// `formula` (Mandelbrot `z²+c` = 0, Multibrot `z³/z⁴/z⁵+c` = 1/2/3). Iterates the reference
/// together with the order-3 series coefficients in arbitrary precision, and skips while the
/// cubic term stays below `2^EPS_LOG2` of the linear term at the worst-case corner `|δc|`
/// (given as `log2_max_dc`) — which guarantees validity, and that no pixel escapes, before
/// `skip`. `orbit_len` bounds the skip below the reference length. Only the holomorphic
/// polynomial families have this expansion; callers must not pass others.
pub fn series_skip(
    cx: &BigFloat,
    cy: &BigFloat,
    log2_max_dc: f64,
    max_iter: u32,
    orbit_len: u32,
    formula: u32,
    p: usize,
) -> SeriesSkip {
    if !log2_max_dc.is_finite() {
        return SeriesSkip::NONE;
    }
    const EPS_LOG2: f64 = -16.0; // cubic term ≤ 2⁻¹⁶ of linear ⇒ ample accuracy
    const MIN_SKIP: u32 = 8; // below this the bookkeeping isn't worth it
    let limit = max_iter.min(orbit_len.saturating_sub(2));
    // Degree d of z^d + c, and the binomial weights that appear in the order-3 recurrence.
    let deg: u32 = match formula {
        1 => 3,
        2 => 4,
        3 => 5,
        _ => 2,
    };
    let one = bf(1.0, p);
    // Recurrence factors — all small exact integers, applied via `mul_u32_bf` (shift-and-add).
    let d_u = deg;
    let c2_u = deg * (deg - 1) / 2; // C(d,2)
    let two_c2_u = deg * (deg - 1); // 2·C(d,2)
    let c3_u = deg * (deg - 1) * (deg - 2) / 6; // C(d,3) (0 for d=2)
    let (mut zx, mut zy) = (bf(0.0, p), bf(0.0, p));
    let (mut ax, mut ay) = (bf(0.0, p), bf(0.0, p));
    let (mut bx, mut by) = (bf(0.0, p), bf(0.0, p));
    let (mut cxx, mut cyy) = (bf(0.0, p), bf(0.0, p));
    let mut best: Option<(u32, [BigFloat; 6])> = None;
    for n in 1..=limit {
        // Advance the order-3 coefficients for z^d + c, using Z_{n-1} (current z):
        //   A' = d·Z^{d-1}·A + 1
        //   B' = d·Z^{d-1}·B + C(d,2)·Z^{d-2}·A²
        //   C' = d·Z^{d-1}·C + 2·C(d,2)·Z^{d-2}·A·B + C(d,3)·Z^{d-3}·A³
        let (p1x, p1y) = cpow_bf(&zx, &zy, deg - 1, p); // Z^{d-1}
        let (a2x, a2y) = cmul_bf(&ax, &ay, &ax, &ay, p); // A²
        let (abx, aby) = cmul_bf(&ax, &ay, &bx, &by, p); // A·B
        // Z^{d-2} is the identity (= 1) for d = 2 → `None` skips that whole complex multiply.
        let p2 = (deg >= 3).then(|| cpow_bf(&zx, &zy, deg - 2, p));
        let zp2 = |wx: &BigFloat, wy: &BigFloat| match &p2 {
            Some((p2x, p2y)) => cmul_bf(p2x, p2y, wx, wy, p),
            None => (wx.clone(), wy.clone()),
        };
        // A' = d·Z^{d-1}·A + 1
        let (t, u) = cmul_bf(&p1x, &p1y, &ax, &ay, p);
        let na_x = mul_u32_bf(&t, d_u, p).add(&one, p, RM);
        let na_y = mul_u32_bf(&u, d_u, p);
        // B' = d·Z^{d-1}·B + C(d,2)·Z^{d-2}·A²
        let (t, u) = cmul_bf(&p1x, &p1y, &bx, &by, p);
        let (v, w) = zp2(&a2x, &a2y);
        let nb_x = mul_u32_bf(&t, d_u, p).add(&mul_u32_bf(&v, c2_u, p), p, RM);
        let nb_y = mul_u32_bf(&u, d_u, p).add(&mul_u32_bf(&w, c2_u, p), p, RM);
        // C' = d·Z^{d-1}·C + 2·C(d,2)·Z^{d-2}·A·B + C(d,3)·Z^{d-3}·A³
        let (t, u) = cmul_bf(&p1x, &p1y, &cxx, &cyy, p);
        let (v, w) = zp2(&abx, &aby);
        let mut nc_x = mul_u32_bf(&t, d_u, p).add(&mul_u32_bf(&v, two_c2_u, p), p, RM);
        let mut nc_y = mul_u32_bf(&u, d_u, p).add(&mul_u32_bf(&w, two_c2_u, p), p, RM);
        if deg >= 3 {
            let (a3x, a3y) = cmul_bf(&a2x, &a2y, &ax, &ay, p); // A³
            // Z^{d-3} is the identity for d = 3.
            let (x3, y3) = if deg >= 4 {
                let (p3x, p3y) = cpow_bf(&zx, &zy, deg - 3, p);
                cmul_bf(&p3x, &p3y, &a3x, &a3y, p)
            } else {
                (a3x, a3y)
            };
            nc_x = nc_x.add(&mul_u32_bf(&x3, c3_u, p), p, RM);
            nc_y = nc_y.add(&mul_u32_bf(&y3, c3_u, p), p, RM);
        }
        // Advance the reference: Z_n = Z_{n-1}^d + c.
        let (nzx, nzy) = step_bf(&zx, &zy, cx, cy, formula, p);
        zx = nzx;
        zy = nzy;
        ax = na_x;
        ay = na_y;
        bx = nb_x;
        by = nb_y;
        cxx = nc_x;
        cyy = nc_y;
        // Validity (in log space → no overflow): cubic·|δc|³ ≤ 2^EPS · linear·|δc|.
        let la = log2_cmag(&ax, &ay);
        let lc = log2_cmag(&cxx, &cyy);
        if !la.is_finite() {
            continue;
        }
        let valid = lc + 2.0 * log2_max_dc < la + EPS_LOG2;
        if n >= MIN_SKIP {
            if valid {
                best = Some((n, [ax.clone(), ay.clone(), bx.clone(), by.clone(), cxx.clone(), cyy.clone()]));
            } else {
                break; // coefficients only grow ⇒ once invalid, stays invalid
            }
        }
        // Stop if the reference itself escaped.
        if to_f64(&zx) * to_f64(&zx) + to_f64(&zy) * to_f64(&zy) > 1.0e12 {
            break;
        }
    }
    match best {
        None => SeriesSkip::NONE,
        Some((skip, k)) => {
            let (a, a_exp) = coeff_to_fe(&k[0], &k[1]);
            let (b, b_exp) = coeff_to_fe(&k[2], &k[3]);
            let (c, c_exp) = coeff_to_fe(&k[4], &k[5]);
            SeriesSkip { skip, a, a_exp, b, b_exp, c, c_exp }
        }
    }
}

// ---------------- BLA (bilinear approximation) -----------------------------------
// Skips iterations *throughout* the orbit (series approximation only skips the start). While
// |δz| is small the Mandelbrot step δz' = 2Zδz + δz² + δc is ≈ linear: δz' ≈ Aδz + Bδc with
// A = 2Z, B = 1, dropping δz². Consecutive linear steps compose, so a binary tree of merged
// steps lets a pixel skip 2^l iterations at once whenever |δz| ≤ the merged validity radius.
// (Zhuoran's BLA, as used in Kalles Fraktaler 2+ / Fraktaler-3.)

/// Complex value with `FloatExp` parts — BLA coefficients span far beyond f64's exponent
/// range (a merged `A` is a long product of `2Zₙ`).
#[derive(Clone, Copy)]
pub struct CFloatExp {
    pub re: FloatExp,
    pub im: FloatExp,
}

impl CFloatExp {
    // See the note on `FloatExp::mul` — deferred `std::ops` migration (REFACTOR-PLAN.md Phase 1).
    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, o: CFloatExp) -> CFloatExp {
        CFloatExp {
            re: self.re.mul(o.re).sub(self.im.mul(o.im)),
            im: self.re.mul(o.im).add(self.im.mul(o.re)),
        }
    }
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, o: CFloatExp) -> CFloatExp {
        CFloatExp { re: self.re.add(o.re), im: self.im.add(o.im) }
    }
    /// Magnitude `|a| = hypot(re, im)` in extended range.
    pub fn abs(self) -> FloatExp {
        self.re.mul(self.re).add(self.im.mul(self.im)).sqrt()
    }

    /// GPU upload form: a **shared-exponent** df32 mantissa `[re_hi, re_lo, im_hi, im_lo]`
    /// (both parts scaled to one base-2 exponent) plus that exponent — matches the SA coeff
    /// layout / the shader's `Fe` (df32 mantissa + i32 exponent).
    pub fn to_mantissa_exp(self) -> ([f32; 4], i32) {
        let e = match (self.re.m == 0.0, self.im.m == 0.0) {
            (true, true) => return ([0.0; 4], 0),
            (false, true) => self.re.e,
            (true, false) => self.im.e,
            (false, false) => self.re.e.max(self.im.e),
        };
        let split = |f: FloatExp| -> (f32, f32) {
            let v = f.m * 2f64.powi(f.e - e); // value·2^−e, in (−2, 2]
            let hi = v as f32;
            (hi, (v - hi as f64) as f32)
        };
        let (rh, rl) = split(self.re);
        let (ih, il) = split(self.im);
        ([rh, rl, ih, il], e)
    }
}

/// One BLA node: the linear map `δz' = A·δz + B·δc`, valid while `|δz| ≤ r`, covering `span`
/// consecutive reference iterations starting at this node's (aligned) index.
#[derive(Clone, Copy)]
pub struct BlaNode {
    pub a: CFloatExp,
    pub b: CFloatExp,
    pub r: FloatExp,
    pub span: u32,
}

/// Merge two consecutive BLA nodes (`x` then `y`). Composition `A=A_y·A_x`, `B=A_y·B_x+B_y`;
/// validity `|δz|≤r_x` **and** `|A_x δz + B_x δc|≤r_y` ⇒ `r = min(r_x, (r_y − |B_x|·δc_max)/|A_x|)`.
fn bla_merge(x: BlaNode, y: BlaNode, dc_max: FloatExp) -> BlaNode {
    let a = y.a.mul(x.a);
    let b = y.a.mul(x.b).add(y.b);
    let a1 = x.a.abs();
    let b1 = x.b.abs();
    let t = y.r.sub(b1.mul(dc_max));
    let t = if t.m < 0.0 { FloatExp::ZERO } else { t };
    let r2 = if a1.m == 0.0 { FloatExp::ZERO } else { t.mul(a1.recip()) };
    let r = if x.r.lt(r2) { x.r } else { r2 };
    BlaNode { a, b, r, span: x.span + y.span }
}

/// Build the BLA binary tree for a **Mandelbrot** reference `orbit` (df32 `[x_hi,y_hi,x_lo,
/// y_lo]`). `dc_max` is the worst-case `|δc|` over the view; `eps` the per-step linear
/// tolerance (smaller ⇒ more accurate but fewer skips). `levels[l][j]` covers the steps
/// starting at `j·2^l`; level 0 has one node per step `n` (using `Zₙ`), higher levels merge
/// pairs (an odd tail carries up with its smaller span).
pub fn build_bla_mandel(orbit: &[[f32; 4]], dc_max: FloatExp, eps: f64) -> Vec<Vec<BlaNode>> {
    let nstep = orbit.len().saturating_sub(1);
    if nstep == 0 {
        return Vec::new();
    }
    let one = CFloatExp { re: FloatExp::from_f64(1.0), im: FloatExp::ZERO };
    let mut lvl0 = Vec::with_capacity(nstep);
    for z in orbit.iter().take(nstep) {
        let zr = z[0] as f64 + z[2] as f64; // df32 → f64 (Z real)
        let zi = z[1] as f64 + z[3] as f64; // Z imag
        let a = CFloatExp { re: FloatExp::from_f64(2.0 * zr), im: FloatExp::from_f64(2.0 * zi) };
        let r = a.abs().mul_f64(eps); // |2Z|·eps : drops δz² with rel error ≤ eps
        lvl0.push(BlaNode { a, b: one, r, span: 1 });
    }
    let mut levels = vec![lvl0];
    while levels.last().unwrap().len() > 1 {
        let prev = levels.last().unwrap();
        let mut next = Vec::with_capacity(prev.len().div_ceil(2));
        let mut j = 0;
        while j < prev.len() {
            if j + 1 < prev.len() {
                next.push(bla_merge(prev[j], prev[j + 1], dc_max));
            } else {
                next.push(prev[j]); // odd tail carries up (smaller span, still aligned)
            }
            j += 2;
        }
        levels.push(next);
    }
    levels
}

/// Flatten a BLA tree to the GPU buffer layout: 4 `vec4<f32>` per node, levels concatenated
/// (level 0 first). Per node: `[A mantissa]`, `[B mantissa]`, `[a_exp, b_exp, r_exp, r_mant]`,
/// `[span, 0, 0, 0]` (exponents/span as exact f32; small enough). The shader reconstructs the
/// per-level offsets from `orbit_len` (level 0 has `orbit_len−1` nodes, each level halves).
pub fn bla_to_gpu(levels: &[Vec<BlaNode>]) -> Vec<[f32; 4]> {
    let total: usize = levels.iter().map(|l| l.len()).sum();
    let mut out = Vec::with_capacity(total * 4);
    for level in levels {
        for node in level {
            let (am, ae) = node.a.to_mantissa_exp();
            let (bm, be) = node.b.to_mantissa_exp();
            let (rm, re) = node.r.to_f32_exp();
            out.push(am);
            out.push(bm);
            out.push([ae as f32, be as f32, re as f32, rm]);
            out.push([node.span as f32, 0.0, 0.0, 0.0]);
        }
    }
    out
}

/// Full value `z = Zₘ + δz` (f64) for a df32 reference orbit — for bailout / escape tests.
fn bla_full_z(orbit: &[[f32; 4]], m: u32, dz: &CFloatExp) -> (f64, f64) {
    let z = orbit[m as usize];
    (z[0] as f64 + z[2] as f64 + dz.re.to_f64(), z[1] as f64 + z[3] as f64 + dz.im.to_f64())
}

/// **Reference** BLA render of one pixel (the exact algorithm the GPU shader will mirror):
/// iterate the perturbation `δz` from 0, skipping with the highest valid BLA level, reverting
/// to a lower level (ultimately a full perturbation step) when a skip would overshoot the
/// escape, and taking a full step whenever `|δz|` exceeds even the level-0 radius. Returns the
/// smooth escape iteration, or `None` if bounded to `max_iter`. (Single reference — no
/// rebasing; used to validate the BLA algorithm on views one reference covers.)
pub fn bla_iterate(
    orbit: &[[f32; 4]],
    levels: &[Vec<BlaNode>],
    dc: (f64, f64),
    bailout2: f64,
    max_iter: u32,
) -> Option<f64> {
    let dc_c = CFloatExp { re: FloatExp::from_f64(dc.0), im: FloatExp::from_f64(dc.1) };
    let mut dz = CFloatExp { re: FloatExp::ZERO, im: FloatExp::ZERO };
    let mut m: u32 = 0;
    let nstep = orbit.len().saturating_sub(1) as u32;
    loop {
        if m >= max_iter || m >= nstep {
            return None; // bounded (ran to the iteration cap / end of the reference)
        }
        // Skip with the highest valid BLA level that neither runs past the reference nor
        // overshoots the escape; revert (try a lower level) on overshoot.
        let dzmag = dz.abs();
        let mut applied = false;
        for l in (0..levels.len()).rev() {
            let step = 1u32 << l;
            if (m & (step - 1)) != 0 {
                continue; // m not aligned to 2^l
            }
            let Some(&node) = levels[l].get((m >> l) as usize) else { continue };
            if m + node.span > nstep || !dzmag.lt(node.r) {
                continue;
            }
            let ndz = node.a.mul(dz).add(node.b.mul(dc_c)); // δz = A·δz + B·δc
            let nm = m + node.span;
            let (zx, zy) = bla_full_z(orbit, nm, &ndz);
            if zx * zx + zy * zy > bailout2 {
                continue; // overshoot — escaped within the span; drop to a lower level
            }
            dz = ndz;
            m = nm;
            applied = true;
            break;
        }
        if applied {
            continue;
        }
        // Full perturbation step at Zₘ: δz' = 2Zδz + δz² + δc (exact — used near the escape).
        let z = orbit[m as usize];
        let two_z = CFloatExp {
            re: FloatExp::from_f64(2.0 * (z[0] as f64 + z[2] as f64)),
            im: FloatExp::from_f64(2.0 * (z[1] as f64 + z[3] as f64)),
        };
        dz = two_z.mul(dz).add(dz.mul(dz)).add(dc_c);
        m += 1;
        let (zx, zy) = bla_full_z(orbit, m, &dz);
        let mag2 = zx * zx + zy * zy;
        if mag2 > bailout2 {
            // Smooth escape count (power 2), matching the shader's formula.
            let nu = (mag2.ln() * 0.5 / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
            return Some(m as f64 + 1.0 - nu);
        }
    }
}

/// Iterations before escape (or `max_iter`) in **arbitrary precision** — ranks
/// candidate references at deep zoom, where `orbit_length`'s `f64` coordinates would
/// all collapse to the same value and make the ranking meaningless.
#[allow(clippy::too_many_arguments)]
fn orbit_length_bf(
    z0x: &BigFloat,
    z0y: &BigFloat,
    cx: &BigFloat,
    cy: &BigFloat,
    formula: u32,
    max_iter: u32,
    p: usize,
) -> u32 {
    let (mut zx, mut zy) = (z0x.clone(), z0y.clone());
    let mut zpx = BigFloat::from_f64(0.0, p); // previous iterate (Phoenix)
    let mut zpy = BigFloat::from_f64(0.0, p);
    let mut n = 0u32;
    while n < max_iter {
        let (nzx, nzy) = if formula == 8 {
            phoenix_step_bf(&zx, &zy, &zpx, &zpy, cx, cy, p)
        } else {
            step_bf(&zx, &zy, cx, cy, formula, p)
        };
        if formula == 8 {
            zpx = std::mem::replace(&mut zx, nzx);
            zpy = std::mem::replace(&mut zy, nzy);
        } else {
            zx = nzx;
            zy = nzy;
        }
        n += 1;
        let (xv, yv) = (to_f64(&zx), to_f64(&zy));
        if xv * xv + yv * yv > 1.0e12 {
            break;
        }
    }
    n
}

/// Iteration cap for reference-candidate scoring (surviving this long ⇒ great
/// reference; bounds the cost of the 5×5 bignum search).
const REF_SCORE_SCAN: u32 = 4096;

/// Pick a reference within the view with the longest orbit (prefers an interior
/// point). For a Julia view, candidates are `Z₀` values with the fixed `julia_c`;
/// otherwise they are `c` values with `Z₀ = 0`. Returns the chosen bignum point.
#[allow(clippy::too_many_arguments)]
pub fn best_reference(
    center: &[BigFloat; 2],
    span: [FloatExp; 2],
    formula: u32,
    julia: bool,
    julia_c: [f64; 2],
    max_iter: u32,
    p: usize,
) -> [BigFloat; 2] {
    // Score candidates by orbit length in **bignum** (f64 coords collapse to the same
    // value at deep zoom, which broke reference selection on cold jumps like bookmark
    // reloads). Cap the scan: a point surviving this many iterations is already an
    // excellent reference, and rebasing covers the rest — keeps the 5×5 search cheap.
    let scan = max_iter.min(REF_SCORE_SCAN);
    let jcx = bf(julia_c[0], p);
    let jcy = bf(julia_c[1], p);
    let zero = bf(0.0, p);
    let score = |zx: &BigFloat, zy: &BigFloat| -> u32 {
        if julia {
            orbit_length_bf(zx, zy, &jcx, &jcy, formula, scan, p)
        } else {
            orbit_length_bf(&zero, &zero, zx, zy, formula, scan, p)
        }
    };
    let mut best = [center[0].clone(), center[1].clone()];
    let mut best_len = score(&center[0], &center[1]);
    if best_len >= scan {
        return best;
    }
    // Search a 5×5 grid at several **scales** (fraction of span), concentrated toward
    // the center where the user is looking. A single coarse ±0.5-span grid is too
    // sparse: at deep zoom the detail is thin filaments, and a wide window spreads the
    // grid into the gaps between them → every candidate escapes fast → a useless
    // reference → uniform render. The inner scales reliably sample the central detail
    // regardless of window width. Fine→coarse so a good hit returns early.
    const N: usize = 5;
    const SCALES: [f64; 4] = [0.04, 0.12, 0.28, 0.5];
    for &sc in &SCALES {
        for j in 0..N {
            for i in 0..N {
                let fx = (i as f64 / (N as f64 - 1.0) - 0.5) * 2.0 * sc;
                let fy = (j as f64 / (N as f64 - 1.0) - 0.5) * 2.0 * sc;
                // Offsets via the extended-range span so the grid doesn't collapse to the
                // center past ~1e308× (where an f64 span would underflow to 0).
                let px = center[0].add(&span[0].mul_f64(fx).to_bf(p), p, RM);
                let py = center[1].add(&span[1].mul_f64(fy).to_bf(p), p, RM);
                let len = score(&px, &py);
                if len > best_len {
                    best_len = len;
                    best = [px, py];
                    if best_len >= scan {
                        return best;
                    }
                }
            }
        }
    }
    best
}

/// Real difference `a - b` as `f64` (used for the small reference offset).
pub fn sub_f64(a: &BigFloat, b: &BigFloat, p: usize) -> f64 {
    to_f64(&a.sub(b, p, RM))
}

/// `a + b` (with `b` an `f64`), arbitrary precision — add a small pixel offset to a
/// full-precision center without losing the center's digits.
pub fn add_f64(a: &BigFloat, b: f64, p: usize) -> BigFloat {
    a.add(&BigFloat::from_f64(b, p), p, RM)
}

/// Naïve **arbitrary-precision** Mandelbrot dwell — an independent oracle (no perturbation,
/// no reference orbit) valid at *any* depth, since the center is bignum. Iterates `z → z² + c`
/// entirely in `astro_float`. Returns `Some((n, smooth))` on escape — `n` is the first
/// iteration with `|zₙ|² > bailout2`, `smooth = n + 1 − log₂(½·ln|z|²/ln2)` — or `None` for
/// interior (reached `max`). `bailout2` must match the renderer's (256² = 65536) for `n` to
/// agree exactly.
pub fn naive_dwell_bf(
    cx: &BigFloat,
    cy: &BigFloat,
    max: u32,
    bailout2: f64,
    p: usize,
) -> Option<(u32, f32)> {
    let mut zx = bf(0.0, p);
    let mut zy = bf(0.0, p);
    for iter in 1..=max {
        let x2 = zx.mul(&zx, p, RM);
        let y2 = zy.mul(&zy, p, RM);
        // zy = 2·zx·zy + cy ; zx = x² − y² + cx
        let nzy = zx.mul(&zy, p, RM).mul(&bf(2.0, p), p, RM).add(cy, p, RM);
        zx = x2.sub(&y2, p, RM).add(cx, p, RM);
        zy = nzy;
        let m2 = to_f64(&zx) * to_f64(&zx) + to_f64(&zy) * to_f64(&zy);
        if m2 > bailout2 {
            let nu = (m2.ln() * 0.5 / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
            return Some((iter, iter as f32 + 1.0 - nu as f32));
        }
    }
    None
}

// ---------------- period / minibrot-nucleus finder --------------------------------
// "Zoom to center": from a view center, snap to the exact center (nucleus) of the
// nearby minibrot and report its period. Mandelbrot/Multibrot only (holomorphic, so
// Newton on the critical-orbit map applies).

/// The exact center + period of a minibrot found near a view center.
pub struct Nucleus {
    pub period: u32,
    pub cx: BigFloat,
    pub cy: BigFloat,
}

/// The integer power `k` of the `z^k + c` families; `None` for the non-holomorphic
/// formulas (Tricorn, Burning Ship, …) where this method doesn't apply.
fn formula_power(formula: u32) -> Option<u32> {
    match formula {
        0 => Some(2),
        1 => Some(3),
        2 => Some(4),
        3 => Some(5),
        _ => None,
    }
}

/// `-a` in arbitrary precision.
fn neg_bf(a: &BigFloat, p: usize) -> BigFloat {
    bf(0.0, p).sub(a, p, RM)
}

/// `z^e` by repeated multiplication (small exponents only); `z^0 = 1`.
fn cpow_bf(zx: &BigFloat, zy: &BigFloat, e: u32, p: usize) -> (BigFloat, BigFloat) {
    if e == 0 {
        return (bf(1.0, p), bf(0.0, p));
    }
    let mut rx = zx.clone();
    let mut ry = zy.clone();
    for _ in 1..e {
        let (nx, ny) = cmul_bf(&rx, &ry, zx, zy, p);
        rx = nx;
        ry = ny;
    }
    (rx, ry)
}

fn mag2_bf(zx: &BigFloat, zy: &BigFloat) -> f64 {
    let (mx, my) = (to_f64(zx), to_f64(zy));
    mx * mx + my * my
}

/// Iteration `n` where the critical orbit `Z_0 = 0, Z_{n+1} = Z_n^k + c` makes its
/// closest approach to the critical point (global argmin of `|Z_n|`, n in 1..=`max`,
/// stopping on escape) — the period of the smallest atom domain enclosing `c`. This may
/// be an integer multiple of the true period for strongly-interior points, which is
/// harmless for Newton (`Z_p = 0 ⟹ Z_{2p} = 0`); the true period is recovered after
/// convergence via [`reduce_period`].
fn detect_period(cx: &BigFloat, cy: &BigFloat, formula: u32, max: u32, p: usize) -> Option<u32> {
    formula_power(formula)?;
    let mut zx = bf(0.0, p);
    let mut zy = bf(0.0, p);
    let mut best_p = 0u32;
    let mut best_m = f64::INFINITY;
    for n in 1..=max.max(1) {
        let (nx, ny) = step_bf(&zx, &zy, cx, cy, formula, p);
        zx = nx;
        zy = ny;
        let m = mag2_bf(&zx, &zy);
        if m < best_m {
            best_m = m;
            best_p = n;
        }
        if m > 1.0e12 {
            break; // escaped
        }
    }
    (best_p > 0).then_some(best_p)
}

/// True (smallest) period at a converged nucleus `c`: the first `n` for which
/// `|Z_n|² < tol²` (the critical orbit returns to 0). `None` if the orbit never returns
/// within `p_est` steps — i.e. `c` is not actually a nucleus (Newton didn't converge).
fn reduce_period(cx: &BigFloat, cy: &BigFloat, formula: u32, p_est: u32, tol2: f64, p: usize) -> Option<u32> {
    let mut zx = bf(0.0, p);
    let mut zy = bf(0.0, p);
    for n in 1..=p_est {
        let (nx, ny) = step_bf(&zx, &zy, cx, cy, formula, p);
        zx = nx;
        zy = ny;
        if mag2_bf(&zx, &zy) < tol2 {
            return Some(n);
        }
    }
    None
}

/// Find the minibrot nucleus near `center` at the given magnification, for the
/// holomorphic families. Detects the period (atom domain), then Newton-refines `c` so
/// the critical orbit closes exactly: solve `Z_period(c) = 0` via
/// `c ← c − Z_period / (dZ_period/dc)` in arbitrary precision. Returns `None` if the
/// formula is unsupported, no period is found, or Newton diverges away from the view.
pub fn find_nucleus(
    center: &[BigFloat; 2],
    mag: f64,
    formula: u32,
    max_period: u32,
) -> Option<Nucleus> {
    let p = precision_for_magnification(mag);
    let k = formula_power(formula)?;
    let p_est = detect_period(&center[0], &center[1], formula, max_period, p)?;

    let span = 3.0 / mag.max(1.0); // approx view width in complex units
    let tol = span * 1.0e-9;
    let one = bf(1.0, p);
    let kf = bf(k as f64, p);

    let mut cx = center[0].clone();
    let mut cy = center[1].clone();
    let period = p_est;
    for _ in 0..64 {
        // Z_period and its derivative D = dZ/dc, from Z_0 = 0, D_0 = 0:
        //   D_{n+1} = k·Z_n^{k-1}·D_n + 1 ;  Z_{n+1} = Z_n^k + c
        let mut zx = bf(0.0, p);
        let mut zy = bf(0.0, p);
        let mut dx = bf(0.0, p);
        let mut dy = bf(0.0, p);
        for _ in 0..period {
            let (zk1x, zk1y) = if k == 2 { (zx.clone(), zy.clone()) } else { cpow_bf(&zx, &zy, k - 1, p) };
            let (mzx, mzy) = cmul_bf(&zk1x, &zk1y, &dx, &dy, p); // Z^{k-1}·D
            let ndx = mzx.mul(&kf, p, RM).add(&one, p, RM);
            let ndy = mzy.mul(&kf, p, RM);
            let (nzx, nzy) = step_bf(&zx, &zy, &cx, &cy, formula, p);
            zx = nzx;
            zy = nzy;
            dx = ndx;
            dy = ndy;
        }
        // Newton step: c -= Z / D = Z · conj(D) / |D|²
        let denom = dx.mul(&dx, p, RM).add(&dy.mul(&dy, p, RM), p, RM);
        if to_f64(&denom) == 0.0 {
            return None;
        }
        let (numx, numy) = cmul_bf(&zx, &zy, &dx, &neg_bf(&dy, p), p);
        let stepx = numx.div(&denom, p, RM);
        let stepy = numy.div(&denom, p, RM);
        cx = cx.sub(&stepx, p, RM);
        cy = cy.sub(&stepy, p, RM);
        let stepm = (to_f64(&stepx).powi(2) + to_f64(&stepy).powi(2)).sqrt();
        if stepm < tol {
            break;
        }
    }

    // Reject runaway Newton: the nucleus should sit within a few view-widths of where
    // we started (otherwise it converged to some unrelated far component).
    let dx = sub_f64(&cx, &center[0], p);
    let dy = sub_f64(&cy, &center[1], p);
    if (dx * dx + dy * dy).sqrt() > span * 8.0 {
        return None;
    }
    // Verify Newton landed on a real nucleus and recover the true (smallest) period.
    let tol2 = (span * 1.0e-3).powi(2);
    let period = reduce_period(&cx, &cy, formula, period, tol2, p)?;
    Some(Nucleus { period, cx, cy })
}

// ============================================================================
// Multi-reference glitch correction (phase 1: CPU reference algorithm, Mandelbrot).
//
// Single-reference perturbation loses precision where a pixel's orbit dips far below the
// reference orbit (|z_n| ≪ |Z_n|) — the classic *Pauldelbrot glitch*: δz = z − Z suffers
// catastrophic cancellation, so the low-order deviation carries no real information and every
// later iterate is garbage (speckle / wrong bands). Zhuoran rebasing mitigates it but a single
// reference still can't serve pixels that live in a genuinely different part of the orbit space.
//
// Multi-reference correction detects glitched pixels (Pauldelbrot criterion) and recomputes just
// those against additional references placed *inside* each glitched region, repeating until every
// pixel is served by a reference for which it is not glitched. This is the exact CPU algorithm the
// GPU/app path will mirror — validated here first, as BLA was.
// ============================================================================

/// Result of perturbing one pixel against one reference.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Perturb {
    /// Escaped, with this smooth (fractional) iteration count.
    Escaped(f64),
    /// Never escaped within `max_iter` (in-set / interior).
    Interior,
    /// The reference is unreliable here (Pauldelbrot criterion) — needs another reference.
    Glitch,
}

/// Mandelbrot reference orbit as f64 `Z_n` (iterated in bignum, narrowed to f64). Stops at escape
/// (`|Z|² > 1e12`) like [`reference_orbit`], so `len ≤ max_iter + 1`. `Z_0 = 0`.
pub fn reference_orbit_f64(cx: &BigFloat, cy: &BigFloat, max_iter: u32, p: usize) -> Vec<(f64, f64)> {
    let mut out = Vec::with_capacity(max_iter as usize + 1);
    let (mut zx, mut zy) = (bf(0.0, p), bf(0.0, p));
    out.push((0.0, 0.0));
    let mut n = 0;
    while n < max_iter {
        let (nx, ny) = step_bf(&zx, &zy, cx, cy, 0, p);
        zx = nx;
        zy = ny;
        let (xv, yv) = (to_f64(&zx), to_f64(&zy));
        out.push((xv, yv));
        n += 1;
        if xv * xv + yv * yv > 1.0e12 {
            break;
        }
    }
    out
}

/// Single-reference Mandelbrot perturbation for one pixel, with Zhuoran rebasing and Pauldelbrot
/// glitch detection. `orbit` = reference `Z_n` (high precision, f64); `dc` = pixel_c − reference_c.
///
/// Crucially the deviation `δz`/`δc` is carried in **f32** — mirroring the GPU (df64 reference,
/// df32 delta). That precision gap is what makes glitches real *and fixable*: where a pixel's
/// orbit dips far below the reference (`|z_n|² < glitch_tol²·|Z_n|²`, Pauldelbrot), the f32 δz
/// can't represent the cancellation in `z = Z + δz`, so the pixel is wrong — but a reference
/// placed closer keeps `|δz|` small enough for f32 to hold, fixing it. Mirrors the shader loop:
/// `δz' = 2Z·δz + δz² + δc`, `z = Z_{n+1} + δz`, rebase when `|z|² < |δz|²` or the reference ends.
pub fn perturb_pixel_mandel(
    orbit: &[(f64, f64)],
    dc: (f64, f64),
    max_iter: u32,
    glitch_tol: f64,
) -> Perturb {
    let bail2 = 256.0 * 256.0;
    let tol2 = glitch_tol * glitch_tol;
    let n_ref = orbit.len();
    if n_ref == 0 {
        return Perturb::Interior;
    }
    let (dcx, dcy) = (dc.0 as f32, dc.1 as f32);
    let (mut dzx, mut dzy) = (0.0f32, 0.0f32);
    let mut ref_n = 0usize;
    let mut iter = 0u32;
    loop {
        if iter >= max_iter {
            return Perturb::Interior;
        }
        let (zrx, zry) = orbit[ref_n]; // reference kept in full f64 precision
        // δz' = 2·Z_n·δz + δz² + δc, evaluated with the reference in f64 but δz stored back as f32.
        let dxf = dzx as f64;
        let dyf = dzy as f64;
        let two_zdz = (2.0 * (zrx * dxf - zry * dyf), 2.0 * (zrx * dyf + zry * dxf));
        let dz2 = (dxf * dxf - dyf * dyf, 2.0 * dxf * dyf);
        dzx = (two_zdz.0 + dz2.0) as f32 + dcx;
        dzy = (two_zdz.1 + dz2.1) as f32 + dcy;
        ref_n += 1;
        iter += 1;
        // Full value z = Z_{n+1} + δz (the f32 δz limits how small a cancellation survives here).
        let (zrnx, zrny) = orbit[ref_n.min(n_ref - 1)];
        let zx = zrnx + dzx as f64;
        let zy = zrny + dzy as f64;
        let z2 = zx * zx + zy * zy;
        // Pauldelbrot glitch: the pixel value is anomalously small vs the reference here.
        let zr2 = zrnx * zrnx + zrny * zrny;
        if z2 < tol2 * zr2 {
            return Perturb::Glitch;
        }
        if z2 > bail2 {
            let nu = (z2.ln() * 0.5 / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
            return Perturb::Escaped(iter as f64 + 1.0 - nu);
        }
        // Rebase (Zhuoran): reference exhausted, or the perturbation now dominates the reference.
        let dzmag2 = (dzx as f64) * (dzx as f64) + (dzy as f64) * (dzy as f64);
        if z2 < dzmag2 || ref_n + 1 >= n_ref {
            dzx = zx as f32;
            dzy = zy as f32;
            ref_n = 0;
        }
    }
}

/// Per-pixel smooth-iteration grid (NaN = interior) plus diagnostics, from a multi-reference render.
pub struct MultiRefResult {
    /// `w*h` row-major smooth iteration counts; `NaN` marks interior (non-escaping) pixels.
    pub smooth: Vec<f64>,
    /// Number of references the correction ended up using.
    pub refs_used: usize,
    /// Pixels the *first* reference flagged as glitched (before any correction) — how much work
    /// multi-reference actually did.
    pub glitched_pass0: usize,
    /// Pixels still glitched after `max_refs` (0 = fully converged).
    pub unresolved: usize,
}

/// Render a `w×h` Mandelbrot grid with multi-reference glitch correction. `upp` is the f64
/// units-per-pixel (this phase targets depths where the per-pixel `δc` fits f64). The first
/// reference sits at pixel `seed`; each remaining glitched region then gets its own reference
/// (placed at the glitched pixel nearest that region's centroid), up to `max_refs`.
#[allow(clippy::too_many_arguments)]
pub fn render_multiref_mandel(
    center_x: &BigFloat,
    center_y: &BigFloat,
    upp: f64,
    w: usize,
    h: usize,
    max_iter: u32,
    glitch_tol: f64,
    seed: (usize, usize),
    max_refs: usize,
    p: usize,
) -> MultiRefResult {
    let (cx0, cy0) = (to_f64(center_x), to_f64(center_y));
    // Complex coordinate (f64) of a pixel center; +y is up (screen y grows downward).
    let pix_c = |px: usize, py: usize| -> (f64, f64) {
        (
            cx0 + (px as f64 - w as f64 * 0.5) * upp,
            cy0 - (py as f64 - h as f64 * 0.5) * upp,
        )
    };
    // Bignum coordinate of a pixel (the reference orbit needs the precision).
    let pix_c_bf = |px: usize, py: usize| -> (BigFloat, BigFloat) {
        let ox = bf((px as f64 - w as f64 * 0.5) * upp, p);
        let oy = bf((py as f64 - h as f64 * 0.5) * upp, p);
        (center_x.add(&ox, p, RM), center_y.sub(&oy, p, RM))
    };

    let mut smooth = vec![f64::NAN; w * h];
    let mut done = vec![false; w * h];
    let mut ref_px = seed;
    let mut refs_used = 0usize;
    let mut glitched_pass0 = 0usize;
    let mut unresolved = 0usize;

    for pass in 0..max_refs {
        let (rcx, rcy) = pix_c_bf(ref_px.0, ref_px.1);
        let orbit = reference_orbit_f64(&rcx, &rcy, max_iter, p);
        let rc = pix_c(ref_px.0, ref_px.1);
        refs_used += 1;
        let mut glitch_pixels: Vec<(usize, usize)> = Vec::new();
        for py in 0..h {
            for px in 0..w {
                let idx = py * w + px;
                if done[idx] {
                    continue;
                }
                let pc = pix_c(px, py);
                let dc = (pc.0 - rc.0, pc.1 - rc.1);
                match perturb_pixel_mandel(&orbit, dc, max_iter, glitch_tol) {
                    Perturb::Escaped(s) => {
                        smooth[idx] = s;
                        done[idx] = true;
                    }
                    Perturb::Interior => {
                        smooth[idx] = f64::NAN;
                        done[idx] = true;
                    }
                    Perturb::Glitch => glitch_pixels.push((px, py)),
                }
            }
        }
        if pass == 0 {
            glitched_pass0 = glitch_pixels.len();
        }
        unresolved = glitch_pixels.len(); // 0 once the region is fully served
        if glitch_pixels.is_empty() {
            break;
        }
        // Next reference: the glitched pixel nearest the centroid of the remaining glitch region
        // (a coarse "largest blob" heuristic — good enough, and the region shrinks each pass).
        let (mut sx, mut sy) = (0.0, 0.0);
        for &(gx, gy) in &glitch_pixels {
            sx += gx as f64;
            sy += gy as f64;
        }
        let n = glitch_pixels.len() as f64;
        let (cxp, cyp) = (sx / n, sy / n);
        ref_px = *glitch_pixels
            .iter()
            .min_by(|a, b| {
                let da = (a.0 as f64 - cxp).powi(2) + (a.1 as f64 - cyp).powi(2);
                let db = (b.0 as f64 - cxp).powi(2) + (b.1 as f64 - cyp).powi(2);
                da.partial_cmp(&db).unwrap()
            })
            .unwrap();
    }
    MultiRefResult { smooth, refs_used, glitched_pass0, unresolved }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} !≈ {b}");
    }

    fn octaves_for(exp_decimal: f64) -> u64 {
        (exp_decimal * std::f64::consts::LN_10 / std::f64::consts::LN_2).ceil() as u64
    }

    // The arbitrary-precision Phoenix reference orbit (z' = z² + c − 0.5·z_prev, two-term recurrence)
    // must reproduce the f64 direct orbit while both are bounded — validates the bignum step +
    // z_prev threading that the deep-zoom perturbation path relies on.
    #[test]
    fn phoenix_reference_matches_direct() {
        let p = 80;
        let c = (0.1_f64, 0.05_f64);
        let (cx, cy) = (BigFloat::from_f64(c.0, p), BigFloat::from_f64(c.1, p));
        let z0 = BigFloat::from_f64(0.0, p);
        let (orbit, _len) = reference_orbit(&z0, &z0, &cx, &cy, 8, 60, p);
        let direct = orbit_points((0.0, 0.0), c, 8, 60, 1.0e12);
        let mut compared = 0;
        for i in 0..orbit.len().min(direct.len()) {
            let (dx, dy) = direct[i];
            if dx * dx + dy * dy > 16.0 {
                break; // stop once it's escaping (large values make absolute compare meaningless)
            }
            let zx = orbit[i][0] as f64 + orbit[i][2] as f64; // df64 hi + lo
            let zy = orbit[i][1] as f64 + orbit[i][3] as f64;
            approx(zx, dx);
            approx(zy, dy);
            compared += 1;
        }
        assert!(compared >= 12, "only {compared} bounded Phoenix iterates compared");
    }

    // Extreme-depth arithmetic validation (feasible single-point form): at a magnification of
    // 1e1000× — far beyond f64 range — the arbitrary-precision z²+c iteration must be stable
    // under a precision increase (agree to ≈ p bits) and the coordinate must survive a decimal
    // round-trip. Fast (~3.4k-bit precision); the deeper 1e100000×/1e1000000× cases run via
    // `--validate-deep` (seconds–minutes) and the #[ignore]d test below.
    #[test]
    fn deep_precision_self_consistent_1e1000() {
        let p = precision_for_octaves(octaves_for(1_000.0));
        let agree = deep_consistency_bits(p, 256, 2_000);
        assert!(agree >= p as i64 - 128, "self-consistency {agree} of {p} bits");
        let rt = deep_roundtrip_bits(p);
        assert!(rt >= p as i64 - 256, "round-trip {rt} of {p} bits");
    }

    // Same check at 1e100000× (~332k-bit precision). Opt-in (slow, ~seconds):
    // `cargo test -p fractadyne-core -- --ignored deep_precision_self_consistent_1e100000`.
    #[test]
    #[ignore = "slow: ~332k-bit precision, runs in seconds"]
    fn deep_precision_self_consistent_1e100000() {
        let p = precision_for_octaves(octaves_for(100_000.0));
        let agree = deep_consistency_bits(p, 256, 400);
        assert!(agree >= p as i64 - 128, "self-consistency {agree} of {p} bits");
        let rt = deep_roundtrip_bits(p);
        assert!(rt >= p as i64 - 256, "round-trip {rt} of {p} bits");
    }

    // Two centers differing only at the ~34th significant digit must remain distinct
    // after parse_bf — i.e. parsing preserves deep digits (not truncated to f64).
    #[test]
    fn parse_bf_preserves_deep_digits() {
        let s1 = "-0.7436438870371587047521915061147707";
        let s2 = "-0.7436438870371587047521915061147000"; // differ ~digit 31
        let a = parse_bf(s1).unwrap();
        let b = parse_bf(s2).unwrap();
        let diff = to_f64(&a.sub(&b, 256, RM)).abs();
        assert!(
            diff > 1e-40,
            "parse_bf collapsed deep digits (diff={diff:e}) — bookmarks/exports would \
             lose the location at deep zoom"
        );
    }

    // Full round-trip: a deep center → decimal string → back must reproduce it to far
    // below any pixel size reachable at practical zoom. (`to_decimal_string` caps the
    // digit count, so it's not bit-exact, but the residual is ~1e-79 here — sub-pixel
    // even past ~1e60×, so bookmarks/exports restore the location, not a nearby zone.)
    #[test]
    fn center_string_roundtrip_subpixel() {
        let s = "-0.74364388703715870475219150611477078529733293886840544098878";
        let a = parse_bf(s).unwrap();
        let b = parse_bf(&to_decimal_string(&a)).unwrap();
        let diff = to_f64(&a.sub(&b, 512, RM)).abs();
        // 1e-50 ≈ one pixel at ~1e47× (1000-px wide); we expect far better (~1e-79).
        assert!(diff < 1e-50, "center string round-trip too lossy (diff={diff:e})");
    }

    #[test]
    fn to_f64_roundtrips_simple_values() {
        for &v in &[-0.5, 0.0, 1.0, -2.25, 0.131825, 1234.5] {
            approx(to_f64(&bf(v, 64)), v);
        }
    }

    #[test]
    fn center_maps_to_view_center() {
        let vp = Viewport::new(800.0, 600.0);
        let (cx, cy) = vp.pixel_to_complex(400.0, 300.0);
        approx(to_f64(&cx), to_f64(&vp.center_x));
        approx(to_f64(&cy), to_f64(&vp.center_y));
    }

    #[test]
    fn zoom_keeps_cursor_fixed() {
        let mut vp = Viewport::new(800.0, 600.0);
        let (px, py) = (123.0, 456.0);
        let before = vp.pixel_to_complex(px, py);
        let base_upp = vp.units_per_pixel;
        vp.zoom_at(px, py, 0.5);
        let after = vp.pixel_to_complex(px, py);
        approx(to_f64(&before.0), to_f64(&after.0));
        approx(to_f64(&before.1), to_f64(&after.1));
        assert!(vp.units_per_pixel.log2() < base_upp.log2());
    }

    #[test]
    fn pan_moves_center() {
        let mut vp = Viewport::new(800.0, 600.0);
        let upp = vp.units_per_pixel.to_f64();
        let (cx, cy) = vp.center_f64();
        vp.pan_pixels(10.0, -4.0);
        approx(to_f64(&vp.center_x), cx - 10.0 * upp);
        approx(to_f64(&vp.center_y), cy - 4.0 * upp);
    }

    #[test]
    fn default_magnification_is_one() {
        let vp = Viewport::new(1280.0, 720.0);
        approx(vp.magnification(), 1.0);
    }

    // The viewport scale must survive past f64's ~1e308× ceiling: precision scales up, the
    // magnitude stays exact (adjacent pixels map to distinct bignum coordinates rather than
    // collapsing to the center as an f64 `units_per_pixel` would), and the GPU scale params
    // stay well-formed (O(1) span mantissa + a deep shared exponent). This is what lets the
    // renderer reach the depths `--validate-deep` validates arithmetically.
    #[test]
    fn viewport_scale_survives_past_f64_range() {
        let mut vp = Viewport::new(1000.0, 1000.0);
        vp.set_center_log2mag(bf(-0.5, 2048), bf(0.0, 2048), 1100.0); // ≈ 1e331× (past f64)
        assert!(vp.precision > 1000, "precision did not scale: {}", vp.precision);
        assert!((vp.log2_magnification() - 1100.0).abs() < 2.0, "log2mag off: {}", vp.log2_magnification());
        // Adjacent pixels must differ (a plain-f64 upp would underflow to 0 → identical).
        let p = vp.precision;
        let (ax, _) = vp.pixel_to_complex(500.0, 500.0);
        let (bx, _) = vp.pixel_to_complex(501.0, 500.0);
        let diff = ax.sub(&bx, p, RM);
        assert!(diff.exponent().is_some(), "adjacent pixels collapsed past 1e308×");
        // GPU scale: O(1) span mantissa, deep shared exponent.
        let gs = vp.gpu_scale();
        assert!(
            gs.span_mantissa[0].abs() >= 1.0 && gs.span_mantissa[0].abs() < 4.0,
            "span mantissa not O(1): {}",
            gs.span_mantissa[0]
        );
        assert!(gs.delta_exp < -1000, "delta_exp not deep: {}", gs.delta_exp);
        // The per-pixel offset, rescaled by 2^-delta_exp, is the O(1) mantissa the GPU needs.
        let off = ref_offset_mantissa(&ax, &bx, gs.delta_exp, p);
        assert!(off.is_finite() && off != 0.0, "ref-offset mantissa degenerate: {off}");
    }

    #[test]
    fn span_matches_extent() {
        let vp = Viewport::new(800.0, 600.0);
        let (sx, sy) = vp.complex_span();
        approx(sx, vp.width_px * vp.units_per_pixel.to_f64());
        approx(sy, vp.height_px * vp.units_per_pixel.to_f64());
    }

    #[test]
    fn recommended_iter_grows_with_zoom() {
        let mut vp = Viewport::new(800.0, 600.0);
        assert_eq!(vp.recommended_max_iter(256), 256);
        vp.zoom_at(400.0, 300.0, 1.0 / 1024.0);
        assert!(vp.recommended_max_iter(256) > 256);
    }

    #[test]
    fn precision_grows_with_zoom() {
        assert_eq!(precision_for_magnification(1.0), 64);
        assert!(precision_for_magnification(1e30) > precision_for_magnification(1e3));
    }

    // Series approximation: the order-3 polynomial A·δc + B·δc² + C·δc³ at the chosen skip
    // must reproduce the exact perturbation δz after that many iterations, for a worst-case
    // corner δc — i.e. skipping those iterations introduces negligible error.
    #[test]
    fn series_skip_matches_exact_perturbation() {
        let p = 160;
        let (cx, cy) = (bf(-0.745, p), bf(0.113, p)); // seahorse boundary (slow orbit)
        let max_dc = 1.0e-9_f64; // deep-ish δc, where SA actually applies
        let log2_max_dc = max_dc.log2();
        let max_iter = 5000;
        let s = series_skip(&cx, &cy, log2_max_dc, max_iter, max_iter, 0, p);
        assert!(s.skip >= 8, "no usable skip found (skip={})", s.skip);

        // Reconstruct the (f64) coefficients and evaluate the series at a worst-case δc.
        let cof = |m: [f32; 4], e: i32| -> (f64, f64) {
            let f = 2f64.powi(e);
            ((m[0] as f64 + m[1] as f64) * f, (m[2] as f64 + m[3] as f64) * f)
        };
        let (ar, ai) = cof(s.a, s.a_exp);
        let (br, bi) = cof(s.b, s.b_exp);
        let (cr, ci) = cof(s.c, s.c_exp);
        let cmul = |a: (f64, f64), b: (f64, f64)| (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0);
        let dc = (max_dc, 0.0); // worst-case real corner
        let dc2 = cmul(dc, dc);
        let dc3 = cmul(dc2, dc);
        let t1 = cmul((ar, ai), dc);
        let t2 = cmul((br, bi), dc2);
        let t3 = cmul((cr, ci), dc3);
        let series = (t1.0 + t2.0 + t3.0, t1.1 + t2.1 + t3.1);

        // Exact perturbation in bignum (same reference as series_skip): δz'=2Zδz+δz²+δc.
        let (dcx, dcy) = (bf(max_dc, p), bf(0.0, p));
        let two = bf(2.0, p);
        let (mut zx, mut zy) = (bf(0.0, p), bf(0.0, p));
        let (mut dzx, mut dzy) = (bf(0.0, p), bf(0.0, p));
        for _ in 0..s.skip {
            let (tzx, tzy) = cmul_bf(&zx, &zy, &dzx, &dzy, p); // Z·δz
            let (d2x, d2y) = cmul_bf(&dzx, &dzy, &dzx, &dzy, p);
            let ndzx = tzx.mul(&two, p, RM).add(&d2x, p, RM).add(&dcx, p, RM);
            let ndzy = tzy.mul(&two, p, RM).add(&d2y, p, RM).add(&dcy, p, RM);
            let (nzx, nzy) = step_bf(&zx, &zy, &cx, &cy, 0, p);
            zx = nzx;
            zy = nzy;
            dzx = ndzx;
            dzy = ndzy;
        }
        let (ex, ey) = (to_f64(&dzx), to_f64(&dzy));
        let err = ((series.0 - ex).powi(2) + (series.1 - ey).powi(2)).sqrt();
        let mag = (ex * ex + ey * ey).sqrt().max(1e-300);
        assert!(err / mag < 1.0e-3, "series vs exact δz rel err {:.2e} at skip {}", err / mag, s.skip);
    }

    // Same validation for the Multibrot-3 (z³+c) coefficient recurrence — confirms the
    // generalized order-3 series (A'=3Z²A+1, B'=3Z²B+3ZA², C'=3Z²C+6ZAB+A³) reproduces the
    // exact perturbation δz' = 3Z²δz + 3Zδz² + δz³ + δc.
    #[test]
    fn series_skip_matches_exact_multibrot3() {
        let p = 160;
        let (cx, cy) = (bf(0.2, p), bf(0.2, p)); // interior z³ point → long bounded orbit
        let max_dc = 1.0e-9_f64;
        let s = series_skip(&cx, &cy, max_dc.log2(), 5000, 5000, 1, p); // formula 1 = z³
        assert!(s.skip >= 8, "no usable skip found (skip={})", s.skip);

        let cof = |m: [f32; 4], e: i32| -> (f64, f64) {
            let f = 2f64.powi(e);
            ((m[0] as f64 + m[1] as f64) * f, (m[2] as f64 + m[3] as f64) * f)
        };
        let (ar, ai) = cof(s.a, s.a_exp);
        let (br, bi) = cof(s.b, s.b_exp);
        let (cr, ci) = cof(s.c, s.c_exp);
        let cmul = |a: (f64, f64), b: (f64, f64)| (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0);
        let dc = (max_dc, 0.0);
        let dc2 = cmul(dc, dc);
        let dc3 = cmul(dc2, dc);
        let t1 = cmul((ar, ai), dc);
        let t2 = cmul((br, bi), dc2);
        let t3 = cmul((cr, ci), dc3);
        let series = (t1.0 + t2.0 + t3.0, t1.1 + t2.1 + t3.1);

        // Exact z³ perturbation in bignum, same reference: δz' = 3Z²δz + 3Zδz² + δz³ + δc.
        let (dcx, dcy) = (bf(max_dc, p), bf(0.0, p));
        let three = bf(3.0, p);
        let (mut zx, mut zy) = (bf(0.0, p), bf(0.0, p));
        let (mut dzx, mut dzy) = (bf(0.0, p), bf(0.0, p));
        for _ in 0..s.skip {
            let (z2x, z2y) = cmul_bf(&zx, &zy, &zx, &zy, p); // Z²
            let (zdx, zdy) = cmul_bf(&z2x, &z2y, &dzx, &dzy, p); // Z²δz
            let (d2x, d2y) = cmul_bf(&dzx, &dzy, &dzx, &dzy, p); // δz²
            let (zd2x, zd2y) = cmul_bf(&zx, &zy, &d2x, &d2y, p); // Zδz²
            let (d3x, d3y) = cmul_bf(&d2x, &d2y, &dzx, &dzy, p); // δz³
            let ndzx = zdx
                .mul(&three, p, RM)
                .add(&zd2x.mul(&three, p, RM), p, RM)
                .add(&d3x, p, RM)
                .add(&dcx, p, RM);
            let ndzy = zdy
                .mul(&three, p, RM)
                .add(&zd2y.mul(&three, p, RM), p, RM)
                .add(&d3y, p, RM)
                .add(&dcy, p, RM);
            dzx = ndzx;
            dzy = ndzy;
            let (nzx, nzy) = step_bf(&zx, &zy, &cx, &cy, 1, p); // Z³ + c
            zx = nzx;
            zy = nzy;
        }
        let (ex, ey) = (to_f64(&dzx), to_f64(&dzy));
        let err = ((series.0 - ex).powi(2) + (series.1 - ey).powi(2)).sqrt();
        let mag = (ex * ex + ey * ey).sqrt().max(1e-300);
        assert!(err / mag < 1.0e-3, "z³ series vs exact rel err {:.2e} at skip {}", err / mag, s.skip);
    }

    // BLA: a tree traversal must reproduce the exact (full-step) perturbation while skipping
    // most iterations. Interior reference (main cardioid) → bounded orbit, |δz| stays tiny.
    #[test]
    fn bla_reproduces_exact_perturbation() {
        let p = 96;
        let (cx, cy) = (bf(-0.5, p), bf(0.0, p)); // main-cardioid interior, never escapes
        let target: u32 = 2000;
        let (orbit, len) = reference_orbit(&bf(0.0, p), &bf(0.0, p), &cx, &cy, 0, target, p);
        assert!(len >= target, "reference escaped early (len={len})");
        let dc = (1.0e-9_f64, 0.0_f64); // worst-case corner δc
        let dc_max = FloatExp::from_f64((dc.0 * dc.0 + dc.1 * dc.1).sqrt());
        let levels = build_bla_mandel(&orbit, dc_max, 1.0e-6);
        assert!(!levels.is_empty());

        // BLA traversal: skip with the highest valid level, else a full perturbation step.
        let dc_c = CFloatExp { re: FloatExp::from_f64(dc.0), im: FloatExp::from_f64(dc.1) };
        let mut dz = CFloatExp { re: FloatExp::ZERO, im: FloatExp::ZERO };
        let (mut m, mut ops) = (0u32, 0u32);
        while m < target {
            ops += 1;
            let dzmag = dz.abs();
            let mut used = false;
            for l in (0..levels.len()).rev() {
                let step = 1u32 << l;
                if (m & (step - 1)) != 0 {
                    continue; // not aligned to 2^l
                }
                let j = (m >> l) as usize;
                let Some(&node) = levels[l].get(j) else { continue };
                if m + node.span > target || !dzmag.lt(node.r) {
                    continue;
                }
                dz = node.a.mul(dz).add(node.b.mul(dc_c)); // δz = A·δz + B·δc
                m += node.span;
                used = true;
                break;
            }
            if !used {
                let zr = orbit[m as usize][0] as f64 + orbit[m as usize][2] as f64;
                let zi = orbit[m as usize][1] as f64 + orbit[m as usize][3] as f64;
                let z = CFloatExp { re: FloatExp::from_f64(2.0 * zr), im: FloatExp::from_f64(2.0 * zi) };
                dz = z.mul(dz).add(dz.mul(dz)).add(dc_c); // δz' = 2Zδz + δz² + δc
                m += 1;
            }
        }
        let (bx, by) = (dz.re.to_f64(), dz.im.to_f64());

        // Exact perturbation (full steps, f64 — δz stays ~1e-9 for this interior reference).
        let (mut ex, mut ey) = (0.0f64, 0.0f64);
        for z in orbit.iter().take(target as usize) {
            let (zr, zi) = (z[0] as f64 + z[2] as f64, z[1] as f64 + z[3] as f64);
            let (nzx, nzy) = (
                2.0 * zr * ex - 2.0 * zi * ey + (ex * ex - ey * ey) + dc.0,
                2.0 * zr * ey + 2.0 * zi * ex + 2.0 * ex * ey + dc.1,
            );
            ex = nzx;
            ey = nzy;
        }

        let err = ((bx - ex).powi(2) + (by - ey).powi(2)).sqrt();
        let mag = (ex * ex + ey * ey).sqrt().max(1e-300);
        assert!(err / mag < 1.0e-3, "BLA vs exact rel err {:.2e} (ops={ops})", err / mag);
        assert!(ops < target / 4, "BLA didn't skip enough (ops={ops} of {target})");
    }

    // BLA end-to-end: the reference BLA render (with escape-overshoot handling) must agree
    // with a naive full-step perturbation on the escape iteration — for BLA-engaged pixels
    // (tiny δc, escape late after big skips) AND fast escapers (large δc). This validates the
    // revert-on-overshoot logic the GPU shader will mirror.
    #[test]
    fn bla_matches_naive_including_escapes() {
        let p = 96;
        // Seahorse-ish boundary reference: a long orbit with a dwell gradient nearby.
        let (cx, cy) = (
            parse_bf("-0.743643887037158704752191506114774").unwrap(),
            parse_bf("0.131825904205311970493132056385139").unwrap(),
        );
        let max_iter: u32 = 5000;
        let (orbit, _len) = reference_orbit(&bf(0.0, p), &bf(0.0, p), &cx, &cy, 0, max_iter, p);
        let bail2 = 65536.0_f64; // 256², matching the app's smooth bailout

        // Naive full-step perturbation escape count (same smooth formula as bla_iterate).
        let naive = |dc: (f64, f64)| -> Option<f64> {
            let nstep = orbit.len() - 1;
            let (mut ex, mut ey) = (0.0f64, 0.0f64);
            for m in 0..(max_iter as usize).min(nstep) {
                let z = orbit[m];
                let (zr, zi) = (z[0] as f64 + z[2] as f64, z[1] as f64 + z[3] as f64);
                let (nex, ney) = (
                    2.0 * zr * ex - 2.0 * zi * ey + (ex * ex - ey * ey) + dc.0,
                    2.0 * zr * ey + 2.0 * zi * ex + 2.0 * ex * ey + dc.1,
                );
                ex = nex;
                ey = ney;
                let zn = orbit[m + 1];
                let (zx, zy) = (zn[0] as f64 + zn[2] as f64 + ex, zn[1] as f64 + zn[3] as f64 + ey);
                let mag2 = zx * zx + zy * zy;
                if mag2 > bail2 {
                    let nu = (mag2.ln() * 0.5 / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
                    return Some((m + 1) as f64 + 1.0 - nu);
                }
            }
            None
        };

        let check = |dc_max: f64, dcs: &[(f64, f64)]| {
            let levels = build_bla_mandel(&orbit, FloatExp::from_f64(dc_max), 1.0e-6);
            for &dc in dcs {
                let b = bla_iterate(&orbit, &levels, dc, bail2, max_iter);
                let n = naive(dc);
                match (b, n) {
                    (None, None) => {}
                    (Some(bv), Some(nv)) => assert!(
                        (bv - nv).abs() < 0.5,
                        "BLA {bv} vs naive {nv} at δc={dc:?} (dc_max={dc_max})"
                    ),
                    _ => panic!("BLA/naive disagree on escape at δc={dc:?}: bla={b:?} naive={n:?}"),
                }
            }
        };

        // BLA-engaged: tiny δc near the boundary (mix of late-escaping and bounded).
        check(1.5e-3, &[(0.0, 0.0), (1.0e-3, 0.0), (-1.0e-3, 5.0e-4), (8.0e-4, -9.0e-4), (1.5e-3, 1.5e-3)]);
        // Fast escapers: large δc leave the set quickly (BLA can't engage — full steps).
        check(0.8, &[(0.2, 0.1), (0.4, -0.2), (0.5, 0.3), (-0.6, 0.0)]);
    }

    #[test]
    fn reference_orbit_starts_at_z0() {
        // Mandelbrot mode: Z0 = 0, c = reference point.
        let (orbit, len) =
            reference_orbit(&bf(0.0, 64), &bf(0.0, 64), &bf(-0.5, 64), &bf(0.0, 64), 0, 64, 64);
        assert_eq!(orbit[0], [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(len as usize, orbit.len());
        assert!(len >= 2);
    }

    #[test]
    fn best_reference_prefers_interior_center() {
        let r = best_reference(
            &[bf(-0.5, 64), bf(0.0, 64)],
            [FloatExp::from_f64(0.1), FloatExp::from_f64(0.1)],
            0,
            false,
            [0.0, 0.0],
            500,
            64,
        );
        assert_eq!(
            reference_orbit(&bf(0.0, 64), &bf(0.0, 64), &r[0], &r[1], 0, 500, 64).1,
            501
        );
    }

    // ---- multi-reference glitch correction ----

    /// Exact f64 direct escape smooth-iter (ground truth at shallow zoom, where f64 is exact).
    fn oracle_smooth_f64(cx: f64, cy: f64, max_iter: u32) -> f64 {
        let (mut zx, mut zy) = (0.0f64, 0.0f64);
        for n in 0..max_iter {
            let (nx, ny) = (zx * zx - zy * zy + cx, 2.0 * zx * zy + cy);
            zx = nx;
            zy = ny;
            let m2 = zx * zx + zy * zy;
            if m2 > 256.0 * 256.0 {
                let nu = (m2.ln() * 0.5 / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
                return (n + 1) as f64 + 1.0 - nu;
            }
        }
        f64::NAN
    }

    /// Exact bignum direct escape smooth-iter — ground truth at any depth.
    fn oracle_smooth_bf(cx: &BigFloat, cy: &BigFloat, max_iter: u32, p: usize) -> f64 {
        let (mut zx, mut zy) = (bf(0.0, p), bf(0.0, p));
        for n in 0..max_iter {
            let (nx, ny) = step_bf(&zx, &zy, cx, cy, 0, p);
            zx = nx;
            zy = ny;
            let (xv, yv) = (to_f64(&zx), to_f64(&zy));
            let m2 = xv * xv + yv * yv;
            if m2 > 256.0 * 256.0 {
                let nu = (m2.ln() * 0.5 / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
                return (n + 1) as f64 + 1.0 - nu;
            }
        }
        f64::NAN
    }

    /// Two smooth values agree, tolerating the interior-vs-late-escape ambiguity right at the
    /// boundary (a pixel that escapes at ~max_iter and one flagged interior are indistinguishable).
    fn smooth_agrees(a: f64, b: f64, max_iter: u32) -> bool {
        let near_max = |v: f64| v.is_nan() || v > max_iter as f64 - 5.0;
        if a.is_nan() && b.is_nan() {
            return true;
        }
        if near_max(a) && near_max(b) {
            return true;
        }
        (a - b).abs() < 1.0
    }

    // A genuine glitch: a small view around a real minibrot. The nucleus's critical orbit dips
    // to ~0 every `period` iterations; an off-nucleus reference does NOT dip as low, so near-
    // nucleus pixels satisfy the Pauldelbrot criterion (|z| ≪ |Z| while δz stays small — a glitch
    // rebasing cannot pre-empt). Multi-reference must detect them, add a reference inside the
    // glitched region, converge, and HEAL the image back to what an accurate reference produces.
    // Compared against a nucleus-seeded render (not a bignum oracle) so it's exact per pixel: both
    // use identical f64 math, and the small δc keeps every pixel a genuine, accurate perturbation.
    #[test]
    fn multi_reference_resolves_glitches() {
        // The large period-3 island on the real axis: low period ⇒ few iterations, so the f32-δz
        // *cancellation* glitch (fixable by a closer reference) dominates over mere accumulation.
        let nuc = find_nucleus(&[bf(-1.7548, 96), bf(0.0, 96)], 1.0e3, 0, 100)
            .expect("expected the period-3 minibrot");
        let p = 96;
        let (cx, cy) = (nuc.cx.clone(), nuc.cy.clone());
        let (w, h) = (24usize, 24usize);
        let upp = 0.010 / w as f64; // a view spanning the island + its glitch halo
        let max_iter = (nuc.period * 120).clamp(360, 4000);
        let tol = 1.0e-3;

        // Ground truth: per-pixel bignum direct iteration.
        let (cx0, cy0) = (to_f64(&cx), to_f64(&cy));
        let oracle: Vec<f64> = (0..w * h)
            .map(|i| {
                let (px, py) = (i % w, i / w);
                let pcx = bf(cx0 + (px as f64 - w as f64 * 0.5) * upp, p);
                let pcy = bf(cy0 - (py as f64 - h as f64 * 0.5) * upp, p);
                oracle_smooth_bf(&pcx, &pcy, max_iter, p)
            })
            .collect();

        // Off-nucleus corner reference, WITHOUT correction (single reference only, max_refs=1).
        let single = render_multiref_mandel(&cx, &cy, upp, w, h, max_iter, tol, (0, 0), 1, p);
        // Same corner seed, WITH multi-reference correction.
        let multi = render_multiref_mandel(&cx, &cy, upp, w, h, max_iter, tol, (0, 0), 60, p);

        let count_bad = |v: &[f64]| -> usize {
            (0..w * h).filter(|&i| !smooth_agrees(v[i], oracle[i], max_iter)).count()
        };
        let (s_bad, m_bad) = (count_bad(&single.smooth), count_bad(&multi.smooth));
        eprintln!(
            "period {} max_iter {max_iter} | single-ref wrong {s_bad}, multi-ref wrong {m_bad}, refs {}, glitch0 {}",
            nuc.period, multi.refs_used, multi.glitched_pass0
        );
        // The off-nucleus reference flags glitches; the correction adds references and converges.
        assert!(multi.glitched_pass0 > 0, "off-nucleus seed should induce glitches");
        assert!(multi.refs_used >= 2, "expected ≥2 references, used {}", multi.refs_used);
        assert_eq!(multi.unresolved, 0, "{} glitches unresolved", multi.unresolved);
        // The corrected image matches the bignum ground truth for every pixel, and never worse
        // than single-reference. (The precision gap that makes correction *necessary* — df64
        // reference vs df32 δz — lives on the GPU; this validates the algorithm is correct.)
        assert_eq!(m_bad, 0, "corrected result must match the oracle (wrong: {m_bad})");
        assert!(m_bad <= s_bad, "correction must not increase error (single {s_bad}, multi {m_bad})");
    }

    // Sanity on the perturbation math itself: a good central reference reproduces ground truth
    // (and multi-reference cleans up any far pixels it can't serve directly).
    #[test]
    fn perturbation_matches_direct() {
        let p = 64;
        let (cx, cy) = (bf(-0.5, p), bf(0.0, p));
        let (w, h) = (16usize, 16usize);
        let upp = 2.0 / w as f64;
        let max_iter = 800u32;
        let (cx0, cy0) = (to_f64(&cx), to_f64(&cy));
        let res = render_multiref_mandel(&cx, &cy, upp, w, h, max_iter, 1.0e-3, (w / 2, h / 2), 24, p);
        assert_eq!(res.unresolved, 0);
        let mut bad = 0;
        let mut worst = 0.0f64;
        for py in 0..h {
            for px in 0..w {
                let o = oracle_smooth_f64(
                    cx0 + (px as f64 - w as f64 * 0.5) * upp,
                    cy0 - (py as f64 - h as f64 * 0.5) * upp,
                    max_iter,
                );
                let g = res.smooth[py * w + px];
                if !smooth_agrees(g, o, max_iter) {
                    bad += 1;
                    if g.is_finite() && o.is_finite() {
                        worst = worst.max((g - o).abs());
                    }
                    eprintln!("mismatch ({px},{py}): got {g} oracle {o}");
                }
            }
        }
        assert_eq!(bad, 0, "{bad} pixels disagree with ground truth (worst Δ {worst})");
    }

    // The period-2 disk's nucleus is exactly c = -1. Starting near it, the finder must
    // report period 2 and Newton-snap to (-1, 0).
    #[test]
    fn find_nucleus_period2_disk() {
        let n = find_nucleus(&[bf(-1.01, 80), bf(0.012, 80)], 50.0, 0, 2000).unwrap();
        assert_eq!(n.period, 2);
        approx(to_f64(&n.cx), -1.0);
        approx(to_f64(&n.cy), 0.0);
    }

    // The period-3 bulb's nucleus is c ≈ -0.122561 + 0.744862 i.
    #[test]
    fn find_nucleus_period3_bulb() {
        let n = find_nucleus(&[bf(-0.12, 80), bf(0.74, 80)], 80.0, 0, 4000).unwrap();
        assert_eq!(n.period, 3);
        assert!((to_f64(&n.cx) - (-0.122561)).abs() < 1e-5, "cx={}", to_f64(&n.cx));
        assert!((to_f64(&n.cy) - 0.744862).abs() < 1e-5, "cy={}", to_f64(&n.cy));
    }

    // ---- analytic ground-truth validation (no external data; exact mathematics) ----

    /// Plain-f64 Mandelbrot escape-time dwell (test ground truth). `None` = interior
    /// (did not escape within `max`).
    fn dwell(cx: f64, cy: f64, max: u32) -> Option<u32> {
        let (mut zx, mut zy) = (0.0_f64, 0.0_f64);
        for i in 0..max {
            let (x2, y2) = (zx * zx, zy * zy);
            if x2 + y2 > 4.0 {
                return Some(i);
            }
            zy = 2.0 * zx * zy + cy;
            zx = x2 - y2 + cx;
        }
        None
    }

    /// Exact membership of the main cardioid: a point is inside iff
    /// `q·(q + (x − ¼)) < ¼·y²` with `q = (x − ¼)² + y²`.
    fn in_main_cardioid(x: f64, y: f64) -> bool {
        let xm = x - 0.25;
        let q = xm * xm + y * y;
        q * (q + xm) < 0.25 * y * y
    }

    /// Exact membership of the period-2 bulb: the disc of radius ¼ around c = −1.
    fn in_period2_bulb(x: f64, y: f64) -> bool {
        (x + 1.0) * (x + 1.0) + y * y < 0.0625
    }

    // Points proven interior by the closed-form cardioid/bulb tests must never escape;
    // points well outside the set must escape quickly. Validates the escape iteration.
    #[test]
    fn interior_closed_form_never_escapes() {
        let interior = [(-0.5, 0.0), (0.0, 0.0), (-0.1, 0.1), (-1.0, 0.0), (-0.9, 0.1)];
        for (x, y) in interior {
            assert!(in_main_cardioid(x, y) || in_period2_bulb(x, y), "({x},{y}) not classified interior");
            assert_eq!(dwell(x, y, 20_000), None, "interior point ({x},{y}) escaped");
        }
        let exterior = [(2.0, 0.0), (0.4, 0.4), (-0.8, 0.4), (0.3, 0.6)];
        for (x, y) in exterior {
            assert!(dwell(x, y, 20_000).is_some(), "exterior point ({x},{y}) did not escape");
        }
    }

    // The set is symmetric about the real axis: dwell(x, y) == dwell(x, −y).
    #[test]
    fn dwell_symmetric_about_real_axis() {
        for &(x, y) in &[(0.3, 0.5), (-0.75, 0.13), (-0.1, 0.9), (0.28, 0.012)] {
            assert_eq!(dwell(x, y, 5_000), dwell(x, -y, 5_000), "asymmetry at ({x},{y})");
        }
    }

    // A table of exact hyperbolic-component nuclei: the finder must recover both the
    // period and the coordinates. These are mathematical constants (Munafo mu-ency / KF).
    #[test]
    fn known_nuclei_table() {
        // (period, cx, cy, search-start offset)
        let table: &[(u32, f64, f64)] = &[
            (2, -1.0, 0.0),
            (4, -1.3107026413368328, 0.0),
            (3, -1.7548776662466927, 0.0),
            (3, -0.1225611668766536, 0.7448617666197446),
        ];
        for &(period, cx, cy) in table {
            // Start slightly off the exact nucleus so Newton has to converge to it.
            let start = [bf(cx + 1.0e-3, 96), bf(cy + 1.0e-3, 96)];
            let n = find_nucleus(&start, 200.0, 0, 8000)
                .unwrap_or_else(|| panic!("no nucleus found near period-{period} ({cx},{cy})"));
            assert_eq!(n.period, period, "wrong period for ({cx},{cy})");
            assert!((to_f64(&n.cx) - cx).abs() < 1e-9, "cx off: {} vs {cx}", to_f64(&n.cx));
            assert!((to_f64(&n.cy) - cy).abs() < 1e-9, "cy off: {} vs {cy}", to_f64(&n.cy));
        }
    }

    // Misiurewicz points are pre-periodic: the critical orbit reaches a repelling cycle
    // after a transient. c = −2 lands on the fixed point 2; c = i reaches a 2-cycle.
    #[test]
    fn misiurewicz_preperiodic() {
        // c = −2: 0 → −2 → 2 → 2 → … (fixed at 2 from iterate 2 on).
        let o = orbit_points((0.0, 0.0), (-2.0, 0.0), 0, 30, 1.0e6);
        assert!((o[2].0 - 2.0).abs() < 1e-9 && o[2].1.abs() < 1e-9);
        assert!((o[10].0 - 2.0).abs() < 1e-6, "c=-2 not fixed at 2: {:?}", o[10]);
        // c = i: 0 → i → (−1+i) → −i → (−1+i) → −i → … (pre-period 1, period 2).
        let o = orbit_points((0.0, 0.0), (0.0, 1.0), 0, 40, 1.0e6);
        let a = o[20];
        let b = o[22];
        assert!((a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6, "c=i not 2-periodic: {a:?} {b:?}");
        let mid = o[21];
        assert!((a.0 - mid.0).abs() + (a.1 - mid.1).abs() > 0.5, "c=i collapsed to a fixed point");
    }

    // ---- family symmetries (Phase 2.1; §9 claims verified here, then encoded) ----

    /// Escape-time dwell for any escape-time family (z₀ = 0, c = point), mirroring the
    /// shader's direct iteration. Bailout `|z| > 2` (classification is bailout-independent).
    fn dwell_family(formula: u32, cx: f64, cy: f64, max: u32) -> Option<u32> {
        let (mut x, mut y) = (0.0_f64, 0.0_f64);
        for i in 1..=max {
            let (nx, ny) = match formula {
                1 => (x * x * x - 3.0 * x * y * y + cx, 3.0 * x * x * y - y * y * y + cy), // z³
                2 => {
                    let (ax, ay) = (x * x - y * y, 2.0 * x * y); // z²
                    (ax * ax - ay * ay + cx, 2.0 * ax * ay + cy) // (z²)²
                }
                3 => {
                    let (ax, ay) = (x * x - y * y, 2.0 * x * y);
                    let (bx, by) = (ax * ax - ay * ay, 2.0 * ax * ay); // z⁴
                    (bx * x - by * y + cx, bx * y + by * x + cy) // z⁴·z
                }
                4 => (x * x - y * y + cx, -2.0 * x * y + cy),               // Tricorn conj(z)²
                5 => (x * x - y * y + cx, 2.0 * (x * y).abs() + cy),         // Burning Ship
                6 => ((x * x - y * y).abs() + cx, 2.0 * x * y + cy),         // Celtic
                7 => ((x * x - y * y).abs() + cx, (2.0 * x * y).abs() + cy), // Buffalo
                _ => (x * x - y * y + cx, 2.0 * x * y + cy),                 // Mandelbrot
            };
            x = nx;
            y = ny;
            if x * x + y * y > 4.0 {
                return Some(i);
            }
        }
        None
    }

    fn rot(cx: f64, cy: f64, deg: f64) -> (f64, f64) {
        let t = deg * std::f64::consts::PI / 180.0;
        (cx * t.cos() - cy * t.sin(), cx * t.sin() + cy * t.cos())
    }

    /// Count dwell mismatches over a grid under a coordinate transform.
    fn mismatches(formula: u32, xform: &dyn Fn(f64, f64) -> (f64, f64)) -> (u32, u32) {
        let (mut total, mut bad) = (0u32, 0u32);
        let mut iy = -30i32;
        while iy <= 30 {
            let mut ix = -30i32;
            while ix <= 30 {
                let (cx, cy) = (ix as f64 * 0.06, iy as f64 * 0.06);
                let (tx, ty) = xform(cx, cy);
                total += 1;
                if dwell_family(formula, cx, cy, 600) != dwell_family(formula, tx, ty, 600) {
                    bad += 1;
                }
                ix += 1;
            }
            iy += 1;
        }
        (total, bad)
    }

    // Multibrot z^d+c has (d−1)-fold rotational symmetry in c; the exact rotations (180°,
    // 90°) negate/swap coordinates in float, so dwell must match *exactly*. (§9: confirmed.)
    #[test]
    fn multibrot_rotational_symmetry_exact() {
        // Multibrot-3 (d=3): 2-fold, 180° → (−cx, −cy).
        let (_, b3) = mismatches(1, &|x, y| (-x, -y));
        assert_eq!(b3, 0, "Multibrot-3 not 180°-symmetric ({b3} mismatches)");
        // Multibrot-5 (d=5): 4-fold, 90° → (−cy, cx).
        let (_, b5) = mismatches(3, &|x, y| (-y, x));
        assert_eq!(b5, 0, "Multibrot-5 not 90°-symmetric ({b5} mismatches)");
    }

    // 120° rotations (Multibrot-4, Tricorn) involve an irrational sine, so allow a handful
    // of boundary flips from float error; the symmetry must otherwise hold. (§9: verified.)
    #[test]
    fn threefold_rotational_symmetry_120deg() {
        let (t4, b4) = mismatches(2, &|x, y| rot(x, y, 120.0)); // Multibrot-4: 3-fold
        assert!(b4 * 100 < t4, "Multibrot-4 not ~120°-symmetric ({b4}/{t4})");
        let (tt, bt) = mismatches(4, &|x, y| rot(x, y, 120.0)); // Tricorn: 3-fold
        assert!(bt * 100 < tt, "Tricorn not ~120°-symmetric ({bt}/{tt})");
    }

    // Reflection axes (§9: verify, do not assume). Celtic is symmetric about the real axis;
    // Burning Ship and Buffalo have NO axis reflection (their parts are even in x and y).
    #[test]
    fn abs_variation_reflection_axes() {
        // Celtic (6): real-axis reflection cy → −cy is exact.
        let (_, bc) = mismatches(6, &|x, y| (x, -y));
        assert_eq!(bc, 0, "Celtic not real-axis-symmetric ({bc} mismatches)");
        // Mandelbrot (0): real-axis reflection, exact.
        let (_, bm) = mismatches(0, &|x, y| (x, -y));
        assert_eq!(bm, 0, "Mandelbrot not real-axis-symmetric ({bm} mismatches)");
        // Burning Ship (5) and Buffalo (7): neither axis is a symmetry — confirm both
        // candidate reflections produce many mismatches (documents the asymmetry).
        for formula in [5u32, 7u32] {
            let (t, bx) = mismatches(formula, &|x, y| (-x, y));
            let (_, by) = mismatches(formula, &|x, y| (x, -y));
            assert!(bx * 20 > t && by * 20 > t, "formula {formula} unexpectedly symmetric (x:{bx} y:{by} / {t})");
        }
    }

    // Julia symmetry (dynamical plane): for z^d+c, f_c(ωz)=f_c(z) when ωᵈ=1, so the dwell is
    // invariant under z → ωz for every c. Quadratic case: z → −z (point symmetry).
    #[test]
    fn julia_quadratic_point_symmetry() {
        // Julia: z₀ = pixel, c fixed. dwell(−z₀) must equal dwell(z₀).
        let jc = (-0.512_511_498_387_07_f64, 0.521_295_242_424_99);
        let julia_dwell = |zx: f64, zy: f64| -> Option<u32> {
            let (mut x, mut y) = (zx, zy);
            for i in 1..=2000u32 {
                let (nx, ny) = (x * x - y * y + jc.0, 2.0 * x * y + jc.1);
                x = nx;
                y = ny;
                if x * x + y * y > 4.0 {
                    return Some(i);
                }
            }
            None
        };
        let mut iy = -20i32;
        while iy <= 20 {
            let mut ix = -20i32;
            while ix <= 20 {
                let (zx, zy) = (ix as f64 * 0.08, iy as f64 * 0.08);
                assert_eq!(julia_dwell(zx, zy), julia_dwell(-zx, -zy), "Julia not (−z)-symmetric at ({zx},{zy})");
                ix += 1;
            }
            iy += 1;
        }
    }

    // Extended landmark catalog (§2.2): exact interior/exterior across hyperbolic-component
    // boundaries — cardioid cusp c=¼, period-1↔2 neck c=−¾, period-2 disk tip c=−5/4.
    #[test]
    fn landmark_boundary_classification() {
        // Cusp at c = 1/4: just inside is interior, just outside escapes.
        assert_eq!(dwell_family(0, 0.24, 0.0, 5000), None, "inside cusp escaped");
        assert!(dwell_family(0, 0.26, 0.0, 5000).is_some(), "outside cusp didn't escape");
        // Neck c = −3/4: interior on both the cardioid and period-2 sides.
        assert_eq!(dwell_family(0, -0.74, 0.0, 5000), None, "cardioid side escaped");
        assert_eq!(dwell_family(0, -0.76, 0.0, 5000), None, "period-2 side escaped");
        // Period-2 disk |c+1| < 1/4 (centered at −1): interior. (Just past the real-axis
        // tip −5/4 is the period-4 window — also interior — so leave the disk vertically to
        // reach the exterior.)
        assert_eq!(dwell_family(0, -1.0, 0.0, 5000), None, "period-2 center escaped");
        assert_eq!(dwell_family(0, -1.24, 0.0, 5000), None, "inside period-2 disk escaped");
        assert!(dwell_family(0, -1.0, 0.5, 5000).is_some(), "exterior (−1+0.5i) didn't escape");
        // Cardioid boundary parametrization c(θ)=e^{iθ}/2 − e^{2iθ}/4: a point pulled
        // slightly inward from the boundary is interior.
        for k in 0..8 {
            let th = k as f64 * std::f64::consts::TAU / 8.0;
            let (bx, by) = (0.5 * th.cos() - 0.25 * (2.0 * th).cos(), 0.5 * th.sin() - 0.25 * (2.0 * th).sin());
            let (inx, iny) = (bx * 0.92 + 0.0 * 0.08, by * 0.92); // pull 8% toward interior point ~0
            assert_eq!(dwell_family(0, inx, iny, 5000), None, "inside cardioid boundary escaped at θ={th}");
        }
    }

    // ---- Phase 5.1: fuzz the untrusted coordinate parser (panic-free + round-trip) ----
    #[test]
    fn fuzz_parse_bf_panic_free_and_roundtrips() {
        // Deterministic pseudo-random fuzzing — `parse_bf` ingests pasted/loaded text and
        // must never panic, only return None on garbage.
        let mut s = 0x2545_f491_4f6c_dd1du64;
        let mut next = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        let charset = b"0123456789.-+eE  \t";
        for _ in 0..20_000 {
            let len = (next() % 48) as usize;
            let mut buf = String::with_capacity(len);
            for _ in 0..len {
                buf.push(charset[(next() as usize) % charset.len()] as char);
            }
            let _ = parse_bf(&buf); // must not panic
        }
        // Adversarial explicit inputs.
        for a in [
            "", " ", "\t", "-", "+", ".", "-.", "e", "E", "1e", "1e+", "-1.5e-3", "1e308",
            "1e-308", "1e1000", "-0", "0.0000", "1_000", "0x1f", "NaN", "nan", "inf", "-inf",
            "infinity", "１２３", "1.2.3", "--1", "++1", "1e1e1", ".5", "5.", "  -3.0  ",
        ] {
            let _ = parse_bf(a);
        }
        // Long inputs: bounded work, no hang/panic.
        let _ = parse_bf(&"9".repeat(5000));
        let _ = parse_bf(&format!("-0.{}", "1234567890".repeat(400)));
        // Sanity: well-formed values parse to Some; clear garbage to None. (Exact value
        // round-trips are covered by `center_string_roundtrip_subpixel`.)
        for v in ["0", "-0.5", "-1.7548776662466927", "1e-40", "3.14159"] {
            assert!(parse_bf(v).is_some(), "rejected valid input {v}");
        }
        for v in ["", "abc", "hello", "..", "+-", "inf", "-inf", "NaN"] {
            assert!(parse_bf(v).is_none(), "accepted garbage {v:?}");
        }
    }

    // ---- Phase 6.3: Kalles Fraktaler .kfr import (hardened, fuzzed) ----
    #[test]
    fn parse_kfr_valid_and_robust() {
        // A well-formed .kfr (unknown keys must be ignored, key case ignored).
        let kfr = "Re: -0.743643887037151\nIm: 0.131825904205330\nZoom: 800\n\
                   Iterations: 2000\nColors: 1 2 3\nLocation: ignored\n";
        let v = parse_kfr(kfr).expect("valid .kfr rejected");
        assert!((to_f64(&v.cx) - (-0.743643887037151)).abs() < 1e-12);
        assert!((to_f64(&v.cy) - 0.131825904205330).abs() < 1e-12);
        assert_eq!(v.zoom, 800.0);
        assert_eq!(v.iterations, Some(2000));
        // Over-range zoom clamps; iterations clamp; case-insensitive keys.
        let v = parse_kfr("re: -1\nIM: 0\nZOOM: 1E1000\niterations: 99999999999\n").unwrap();
        assert_eq!(v.zoom, 1.0e300);
        assert_eq!(v.iterations, Some(1_000_000));
        // Missing required fields → None.
        assert!(parse_kfr("Re: 0\nIm: 0\n").is_none(), "missing Zoom accepted");
        assert!(parse_kfr("Im: 0\nZoom: 2\n").is_none(), "missing Re accepted");
        // Invalid center → None (parse_bf rejects).
        assert!(parse_kfr("Re: abc\nIm: 0\nZoom: 2\n").is_none());
    }

    #[test]
    fn fuzz_parse_kfr_panic_free() {
        let mut s = 0x1234_5678_9abc_def1u64;
        let mut next = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        let charset = b"ReImZoomIterations:-+0.9eE \n\t\0xyz";
        for _ in 0..20_000 {
            let len = (next() % 200) as usize;
            let mut buf = String::with_capacity(len);
            for _ in 0..len {
                buf.push(charset[(next() as usize) % charset.len()] as char);
            }
            let _ = parse_kfr(&buf); // must not panic
        }
        // Oversized / adversarial inputs.
        let _ = parse_kfr(&"Re: 1\n".repeat(100_000));
        let _ = parse_kfr(&format!("Re: -0.{}\nIm: 0\nZoom: 2\n", "1".repeat(200_000)));
        let _ = parse_kfr(&"X".repeat(5_000_000));
        for m in ["", ":", "Re:", "Zoom:::", "Re: \0\0\0\nIm:\nZoom:e"] {
            let _ = parse_kfr(m);
        }
    }
}
