//! Core numerics for Fractadyne.
//!
//! The viewport center is **arbitrary precision** (`astro_float::BigFloat`) at a
//! mantissa size that scales with zoom, so position stays sub-pixel at *any* depth
//! (no coordinate jump, ever). `units_per_pixel` is a plain f64 scale. The reference
//! orbit is iterated in bignum and stored as `f32` hi/lo pairs (df64) for the GPU.
//!
//! Bignum is slow, so the reference orbit should be recomputed only when the
//! reference point changes (the app caches it), not every frame.

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
    let mantissa = *digits.last().unwrap() as u64; // top 64 bits (normalized MSW)
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

/// Mantissa bits needed to position sub-pixel at the given magnification (+ guard).
pub fn precision_for_magnification(mag: f64) -> usize {
    let octaves = mag.max(1.0).log2().ceil() as usize;
    (octaves + 64).max(64)
}

fn bf(v: f64, p: usize) -> BigFloat {
    BigFloat::from_f64(v, p)
}

/// Linear interpolation between two `BigFloat`s at precision `p`: `a + (b − a)·t`.
pub fn lerp_bf(a: &BigFloat, b: &BigFloat, t: f64, p: usize) -> BigFloat {
    let f = bf(t, p);
    a.add(&b.sub(a, p, RM).mul(&f, p, RM), p, RM)
}

/// A rectangular view into the complex plane.
#[derive(Clone, Debug)]
pub struct Viewport {
    pub center_x: BigFloat,
    pub center_y: BigFloat,
    /// Complex-plane units per pixel (isotropic). Smaller ⇒ deeper zoom.
    pub units_per_pixel: f64,
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
            units_per_pixel: Self::REFERENCE_HEIGHT / height_px,
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
        self.units_per_pixel = Self::REFERENCE_HEIGHT / self.height_px;
    }

    fn refresh_precision(&mut self) {
        self.precision = precision_for_magnification(self.magnification());
    }

    /// Complex coordinate under a pixel (origin top-left, +y down on screen).
    pub fn pixel_to_complex(&self, px: f64, py: f64) -> (BigFloat, BigFloat) {
        let p = self.precision;
        let ox = bf((px - self.width_px * 0.5) * self.units_per_pixel, p);
        let oy = bf((py - self.height_px * 0.5) * self.units_per_pixel, p);
        (self.center_x.add(&ox, p, RM), self.center_y.sub(&oy, p, RM))
    }

    pub fn pan_pixels(&mut self, dx: f64, dy: f64) {
        let p = self.precision;
        let ox = bf(dx * self.units_per_pixel, p);
        let oy = bf(dy * self.units_per_pixel, p);
        self.center_x = self.center_x.sub(&ox, p, RM);
        self.center_y = self.center_y.add(&oy, p, RM);
    }

    /// Zoom by `factor` (< 1 zooms in) keeping the complex point under `(px,py)` fixed.
    pub fn zoom_at(&mut self, px: f64, py: f64, factor: f64) {
        let (cx, cy) = self.pixel_to_complex(px, py);
        self.units_per_pixel *= factor;
        self.refresh_precision();
        let p = self.precision;
        let ox = bf((px - self.width_px * 0.5) * self.units_per_pixel, p);
        let oy = bf((py - self.height_px * 0.5) * self.units_per_pixel, p);
        self.center_x = cx.sub(&ox, p, RM);
        self.center_y = cy.add(&oy, p, RM);
    }

    /// Zoom so the pixel rectangle fits the view; its center becomes the view center.
    pub fn zoom_to_rect(&mut self, px0: f64, py0: f64, px1: f64, py1: f64) {
        let (cx, cy) = self.pixel_to_complex((px0 + px1) * 0.5, (py0 + py1) * 0.5);
        let box_w = (px1 - px0).abs().max(1.0);
        let box_h = (py1 - py0).abs().max(1.0);
        let complex_w = box_w * self.units_per_pixel;
        let complex_h = box_h * self.units_per_pixel;
        self.units_per_pixel = (complex_w / self.width_px).max(complex_h / self.height_px);
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
        self.units_per_pixel = upp_home * (-new_logmag).exp();
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

    /// Set the view to an explicit center and magnification (1× = home framing).
    /// Used by the scripting / benchmark camera player.
    pub fn set_center_mag(&mut self, cx: BigFloat, cy: BigFloat, mag: f64) {
        self.units_per_pixel = Self::REFERENCE_HEIGHT / self.height_px / mag.max(1.0e-300);
        self.refresh_precision();
        self.center_x = cx;
        self.center_y = cy;
    }

    pub fn complex_span(&self) -> (f64, f64) {
        (
            self.width_px * self.units_per_pixel,
            self.height_px * self.units_per_pixel,
        )
    }

    pub fn magnification(&self) -> f64 {
        Self::REFERENCE_HEIGHT / (self.height_px * self.units_per_pixel)
    }

    pub fn recommended_max_iter(&self, base: u32) -> u32 {
        let octaves = self.magnification().max(1.0).log2().max(0.0);
        (base + (octaves * 220.0) as u32).min(50_000)
    }

    /// Center as `f64` (for display / coarse use).
    pub fn center_f64(&self) -> (f64, f64) {
        (to_f64(&self.center_x), to_f64(&self.center_y))
    }

    /// Complex coordinate under a pixel, as `f64` (for display; +y down on screen).
    pub fn complex_at_pixel_f64(&self, px: f64, py: f64) -> (f64, f64) {
        let (cx, cy) = self.center_f64();
        (
            cx + (px - self.width_px * 0.5) * self.units_per_pixel,
            cy - (py - self.height_px * 0.5) * self.units_per_pixel,
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
            let two = bf(2.0, p);
            let x2 = zx.mul(zx, p, RM);
            let y2 = zy.mul(zy, p, RM);
            let txy = zx.mul(zy, p, RM).mul(&two, p, RM);
            (x2.sub(&y2, p, RM).add(cx, p, RM), cy.sub(&txy, p, RM))
        }
        _ => {
            // Mandelbrot: z² + c
            let two = bf(2.0, p);
            let x2 = zx.mul(zx, p, RM);
            let y2 = zy.mul(zy, p, RM);
            (
                x2.sub(&y2, p, RM).add(cx, p, RM),
                zx.mul(zy, p, RM).mul(&two, p, RM).add(cy, p, RM),
            )
        }
    }
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
    let (xh, xl) = split_df64(to_f64(&zx));
    let (yh, yl) = split_df64(to_f64(&zy));
    out.push([xh, yh, xl, yl]); // Z_0
    let mut n = 0u32;
    while n < max_iter {
        let (nzx, nzy) = step_bf(&zx, &zy, cx, cy, formula, p);
        zx = nzx;
        zy = nzy;
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
    let mut n = 0u32;
    while n < max_iter {
        let (nzx, nzy) = step_bf(&zx, &zy, cx, cy, formula, p);
        zx = nzx;
        zy = nzy;
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
    span: [f64; 2],
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
                let px = center[0].add(&bf(fx * span[0], p), p, RM);
                let py = center[1].add(&bf(fy * span[1], p), p, RM);
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

/// `z^e` for `e ≥ 1` by repeated multiplication (small exponents only).
fn cpow_bf(zx: &BigFloat, zy: &BigFloat, e: u32, p: usize) -> (BigFloat, BigFloat) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} !≈ {b}");
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
        assert!(vp.units_per_pixel < base_upp);
    }

    #[test]
    fn pan_moves_center() {
        let mut vp = Viewport::new(800.0, 600.0);
        let upp = vp.units_per_pixel;
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

    #[test]
    fn span_matches_extent() {
        let vp = Viewport::new(800.0, 600.0);
        let (sx, sy) = vp.complex_span();
        approx(sx, vp.width_px * vp.units_per_pixel);
        approx(sy, vp.height_px * vp.units_per_pixel);
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
            [0.1, 0.1],
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
}
