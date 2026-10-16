//! The independent bignum dwell oracles the harnesses judge a render by. Attached to `reference`
//! (`#[path]`) and re-exported. Moved from `reference.rs`.

use super::*;

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

/// [`naive_dwell_bf`] for every formula the reference builder iterates (Mandelbrot mode:
/// `Z₀ = 0`, `c` = the point). It is the point's OWN orbit in arbitrary precision
/// ([`reference_orbit`]), scanned for the first sample past `bailout2` — no perturbation, no
/// rebasing, no BLA, so it can judge a perturbation render of any family. The smooth count takes
/// the formula's degree as its log base, as the shader's `power_f` does ([`formula::power`]).
pub fn formula_dwell(
    cx: &BigFloat,
    cy: &BigFloat,
    formula: u32,
    max: u32,
    bailout2: f64,
    p: usize,
) -> Option<(u32, f32)> {
    let zero = bf(0.0, p);
    let (orbit, len) = reference_orbit(&zero, &zero, cx, cy, formula, max, p);
    let power = f64::from(formula::power(formula));
    for (n, s) in orbit.iter().enumerate().take(len as usize).skip(1) {
        let (x, y) = sample_xy(s);
        let m2 = x * x + y * y;
        if m2 > bailout2 {
            let nu = (m2.ln() * 0.5 / std::f64::consts::LN_2).ln() / power.ln();
            return Some((n as u32, n as f32 + 1.0 - nu as f32));
        }
    }
    None
}
