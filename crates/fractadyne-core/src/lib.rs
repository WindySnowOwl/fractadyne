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

/// Parse a decimal string back to a `BigFloat` (round-trips `to_decimal_string`).
pub fn parse_bf(s: &str) -> Option<BigFloat> {
    s.trim().parse::<BigFloat>().ok().filter(|b| !b.is_nan())
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
}
