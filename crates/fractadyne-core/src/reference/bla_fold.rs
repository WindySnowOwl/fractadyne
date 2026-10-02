//! BLA for the fold families (design/power-families.md §4.5, phase 4): Burning Ship, Tricorn, Celtic
//! and Buffalo, at power 2 (ids 4–7) and at powers 3–5 (ids 13–24).
//!
//! A fold makes one step's linear part a REAL 2×2 map, not a complex multiply. Near a reference value
//! with no part on an axis, `|x|` is `±x` and `conj` is a reflection, so with `M(c)` the matrix of
//! multiplication by `c`:
//!
//! - Burning Ship `(|x| + i|y|)^d + c`: `A = M(d·F^(d−1))·diag(sgn X, sgn Y)`, `F = |X| + i|Y|`;
//!   linear while neither part of z changes sign: `|δz| < min(|X|, |Y|)`.
//! - Tricorn `conj(z)^d + c`: `A = M(d·conj(Z)^(d−1))·diag(1, −1)`; no fold to cross.
//! - Celtic `|Re w| + i·Im w + c`, `w = z^d`: `A = diag(sgn Re W, 1)·M(d·Z^(d−1))`; linear while
//!   `|δw| < |Re W|`, i.e. `|δz| < |Re W| / ((1 + eps)·|d·Z^(d−1)|)`.
//! - Buffalo: Celtic with `|Im w|` folded too: `diag(sgn Re W, sgn Im W)`, the lesser of the two.
//!
//! Every shape also drops the power's higher terms, as the `z^d + c` tree does: `|δz| ≤
//! 2·eps·|Z|/(d−1)`. Merging composes `A = A_y·A_x`, `B = A_y·B_x + B_y` and shrinks the radius by the
//! operator norms — the EXACT spectral norm: a bound such as Frobenius (√2 high on a rotation)
//! compounds over the tree's levels.
//!
//! A node packs into the complex tree's 16 floats (`bla_to_gpu`): each matrix as four f32 mantissas
//! under one exponent instead of a complex df32 mantissa; exponents, radius, span and the aux
//! aggregates where they always are — so `apply_bla_aux` patches a fold tree unchanged.

use super::*;
use crate::formula::Shape;

/// A real 2×2 matrix in extended range, row-major `[a00, a01, a10, a11]`, acting on `(re, im)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat2Fe(pub [FloatExp; 4]);

fn fe_neg(v: FloatExp) -> FloatExp {
    FloatExp { m: -v.m, e: v.e }
}

impl Mat2Fe {
    pub fn identity() -> Mat2Fe {
        let (o, z) = (FloatExp::from_f64(1.0), FloatExp::ZERO);
        Mat2Fe([o, z, z, o])
    }

    /// Multiplication by the complex number `c`: `[[re, −im], [im, re]]`.
    pub fn complex(c: CFloatExp) -> Mat2Fe {
        Mat2Fe([c.re, fe_neg(c.im), c.im, c.re])
    }

    /// `diag(s0, s1)·self` (signs, or 1).
    fn scale_rows(self, s0: f64, s1: f64) -> Mat2Fe {
        let m = self.0;
        Mat2Fe([m[0].mul_f64(s0), m[1].mul_f64(s0), m[2].mul_f64(s1), m[3].mul_f64(s1)])
    }

    /// `self·diag(s0, s1)`.
    fn scale_cols(self, s0: f64, s1: f64) -> Mat2Fe {
        let m = self.0;
        Mat2Fe([m[0].mul_f64(s0), m[1].mul_f64(s1), m[2].mul_f64(s0), m[3].mul_f64(s1)])
    }

    /// `self·o`.
    pub fn mul(self, o: Mat2Fe) -> Mat2Fe {
        let (a, b) = (self.0, o.0);
        Mat2Fe([
            a[0] * b[0] + a[1] * b[2],
            a[0] * b[1] + a[1] * b[3],
            a[2] * b[0] + a[3] * b[2],
            a[2] * b[1] + a[3] * b[3],
        ])
    }

    pub fn add(self, o: Mat2Fe) -> Mat2Fe {
        let (a, b) = (self.0, o.0);
        Mat2Fe([a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]])
    }

    /// `self·v`.
    pub fn apply(self, v: CFloatExp) -> CFloatExp {
        let m = self.0;
        CFloatExp { re: m[0] * v.re + m[1] * v.im, im: m[2] * v.re + m[3] * v.im }
    }

    /// The spectral norm (largest singular value), exact for 2×2:
    /// `σ² = (F + √(F² − 4·det²))/2` with `F` the squared Frobenius norm. `F² − 4·det²` is
    /// `(σ₁² − σ₂²)²`, so where the two coincide (a rotation-scale) it cancels to ~0 and the root
    /// clamps there, leaving `σ = √(F/2)` — exact.
    pub fn norm(self) -> FloatExp {
        let m = self.0;
        let f = m[0] * m[0] + m[1] * m[1] + m[2] * m[2] + m[3] * m[3];
        let det = m[0] * m[3] - m[1] * m[2];
        let disc = (f * f - det * det * FloatExp::from_f64(4.0)).sqrt();
        ((f + disc).mul_f64(0.5)).sqrt()
    }

    /// GPU form: four f32 mantissas under one shared base-2 exponent.
    pub fn to_gpu(self) -> ([f32; 4], i32) {
        let nz = self.0.iter().filter(|v| v.m != 0.0);
        let Some(e) = nz.map(|v| v.e).max() else { return ([0.0; 4], 0) };
        let q = |v: FloatExp| if v.m == 0.0 { 0.0 } else { (v.m * 2f64.powi(v.e - e)) as f32 };
        ([q(self.0[0]), q(self.0[1]), q(self.0[2]), q(self.0[3])], e)
    }
}

/// A fold family's shape and power: the power-2 originals and the power families' folds (`None` for
/// every `z^d + c` family and the rest).
pub fn fold_shape(formula: u32) -> Option<(Shape, u32)> {
    match formula {
        formula::BURNING_SHIP => Some((Shape::BurningShip, 2)),
        formula::TRICORN => Some((Shape::Tricorn, 2)),
        formula::CELTIC => Some((Shape::Celtic, 2)),
        formula::BUFFALO => Some((Shape::Buffalo, 2)),
        f => formula::family(f).filter(|(s, _)| *s != Shape::Multibrot),
    }
}

/// One fold-tree node: `δz' = A·δz + B·δc` for real 2×2 `A`, `B`, valid while `|δz| ≤ r`, over
/// `span` reference steps; the aux aggregates as [`BlaNode`]'s.
#[derive(Clone, Copy, Debug)]
pub struct BlaFoldNode {
    pub a: Mat2Fe,
    pub b: Mat2Fe,
    pub r: FloatExp,
    pub span: u32,
    pub agg_trap: f64,
    pub agg_tia: f64,
    pub agg_stripe: f64,
}

fn cpow(w: CFloatExp, k: u32) -> CFloatExp {
    let mut r = CFloatExp { re: FloatExp::from_f64(1.0), im: FloatExp::ZERO };
    for _ in 0..k {
        r = r * w;
    }
    r
}

fn sgn(v: f64) -> f64 {
    if v < 0.0 { -1.0 } else { 1.0 }
}

/// The single-step node at orbit sample `n` (see the module notes for each shape's A and radius).
fn fold_level0_node(n: usize, orbit: &[[f32; 4]], eps: f64, aux: AuxAggParams, shape: Shape, d: u32) -> BlaFoldNode {
    // `sample_xy`, not lane sums: an extended-range dip sample carries a NaN marker.
    let (zr, zi) = sample_xy(&orbit[n]);
    let z = CFloatExp { re: FloatExp::from_f64(zr), im: FloatExp::from_f64(zi) };
    let df = f64::from(d);
    let r_power = z.abs().mul_f64(2.0 * eps / (df - 1.0));
    let (a, r_fold) = match shape {
        Shape::BurningShip => {
            let f = CFloatExp { re: FloatExp::from_f64(zr.abs()), im: FloatExp::from_f64(zi.abs()) };
            let a = Mat2Fe::complex(cpow(f, d - 1).mul_f64(df)).scale_cols(sgn(zr), sgn(zi));
            (a, Some(FloatExp::from_f64(zr.abs().min(zi.abs()))))
        }
        Shape::Tricorn => {
            let zc = CFloatExp { re: z.re, im: fe_neg(z.im) };
            (Mat2Fe::complex(cpow(zc, d - 1).mul_f64(df)).scale_cols(1.0, -1.0), None)
        }
        Shape::Celtic | Shape::Buffalo => {
            let g = cpow(z, d - 1).mul_f64(df); // d·Z^(d−1)
            let w = cpow(z, d);
            let buffalo = shape == Shape::Buffalo;
            let (wr, wi) = (w.re.to_f64(), w.im.to_f64());
            let a = Mat2Fe::complex(g).scale_rows(sgn(wr), if buffalo { sgn(wi) } else { 1.0 });
            let lim = if buffalo { w.re.abs().min_fe(w.im.abs()) } else { w.re.abs() };
            let gn = g.abs().mul_f64(1.0 + eps);
            (a, Some(if gn.m == 0.0 { FloatExp::ZERO } else { lim * gn.recip() }))
        }
        Shape::Multibrot => unreachable!("a z^d + c family takes the complex tree"),
    };
    let r = match r_fold {
        Some(rf) if rf.lt(r_power) => rf,
        _ => r_power,
    };
    let [agg_trap, agg_tia, agg_stripe] = bla_level0_aux(n, orbit, aux);
    BlaFoldNode { a, b: Mat2Fe::identity(), r, span: 1, agg_trap, agg_tia, agg_stripe }
}

trait MinFe {
    fn min_fe(self, o: FloatExp) -> FloatExp;
}
impl MinFe for FloatExp {
    fn min_fe(self, o: FloatExp) -> FloatExp {
        if self.lt(o) { self } else { o }
    }
}

/// Merge two consecutive nodes (`x` then `y`), as [`bla_merge`] with matrices: validity `|δz| ≤ r_x`
/// and `|A_x·δz + B_x·δc| ≤ r_y` ⇒ `r = min(r_x, (r_y − ‖B_x‖·δc_max)/‖A_x‖)`.
fn fold_merge(x: BlaFoldNode, y: BlaFoldNode, dc_max: FloatExp) -> BlaFoldNode {
    let a = y.a.mul(x.a);
    let b = y.a.mul(x.b).add(y.b);
    let t = y.r - x.b.norm() * dc_max;
    let t = if t.m < 0.0 { FloatExp::ZERO } else { t };
    let an = x.a.norm();
    let r2 = if an.m == 0.0 { FloatExp::ZERO } else { t * an.recip() };
    BlaFoldNode {
        a,
        b,
        r: x.r.min_fe(r2),
        span: x.span + y.span,
        agg_trap: x.agg_trap.min(y.agg_trap),
        agg_tia: x.agg_tia + y.agg_tia,
        agg_stripe: x.agg_stripe + y.agg_stripe,
    }
}

/// The fold tree for `formula` (see [`fold_shape`]; empty for any other formula), laid out as
/// [`build_bla`]'s: `levels[l][j]` covers the steps from `j·2^l`, an odd tail carried up.
pub fn build_bla_fold(orbit: &[[f32; 4]], dc_max: FloatExp, eps: f64, aux: AuxAggParams, formula: u32) -> Vec<Vec<BlaFoldNode>> {
    let Some((shape, d)) = fold_shape(formula) else { return Vec::new() };
    let nstep = orbit.len().saturating_sub(1);
    if nstep == 0 {
        return Vec::new();
    }
    let placeholder = BlaFoldNode {
        a: Mat2Fe::identity(),
        b: Mat2Fe::identity(),
        r: FloatExp::ZERO,
        span: 0,
        agg_trap: 0.0,
        agg_tia: 0.0,
        agg_stripe: 0.0,
    };
    let mut lvl0 = vec![placeholder; nstep];
    par_fill(&mut lvl0, BLA_PAR_THRESHOLD, |n| fold_level0_node(n, orbit, eps, aux, shape, d));
    let mut levels = vec![lvl0];
    while levels.last().unwrap().len() > 1 {
        let prev = levels.last().unwrap();
        let mut next = vec![placeholder; prev.len().div_ceil(2)];
        par_fill(&mut next, BLA_PAR_THRESHOLD, |k| {
            let j = 2 * k;
            if j + 1 < prev.len() { fold_merge(prev[j], prev[j + 1], dc_max) } else { prev[j] }
        });
        levels.push(next);
    }
    levels
}

/// GPU packing, [`bla_to_gpu`]'s layout with each matrix as f32 mantissas under one exponent:
/// `[A]`, `[B]`, `[a_exp, b_exp, r_exp, r_mant]`, `[span, trap, tia, stripe]`.
pub fn bla_fold_to_gpu(levels: &[Vec<BlaFoldNode>]) -> Vec<[f32; 4]> {
    let mut out = Vec::with_capacity(levels.iter().map(|l| l.len()).sum::<usize>() * 4);
    for node in levels.iter().flatten() {
        let (am, ae) = node.a.to_gpu();
        let (bm, be) = node.b.to_gpu();
        let (rm, re) = node.r.to_f32_exp();
        out.push(am);
        out.push(bm);
        out.push([ae as f32, be as f32, re as f32, rm]);
        out.push([node.span as f32, node.agg_trap as f32, node.agg_tia as f32, node.agg_stripe as f32]);
    }
    out
}

/// The GPU-packed tree for any formula `FormulaCaps::bla` grants: the complex `z^d + c` tree
/// ([`build_bla`]) or the fold tree; empty for any other formula, or an orbit too short for one.
pub fn bla_tree_gpu(orbit: &[[f32; 4]], dc_max: FloatExp, eps: f64, aux: AuxAggParams, formula: u32) -> Vec<[f32; 4]> {
    if fold_shape(formula).is_some() {
        let levels = build_bla_fold(orbit, dc_max, eps, aux, formula);
        return if levels.is_empty() { Vec::new() } else { bla_fold_to_gpu(&levels) };
    }
    if !formula::caps(formula).bla {
        return Vec::new();
    }
    let levels = build_bla(orbit, dc_max, eps, aux, formula::power(formula));
    if levels.is_empty() { Vec::new() } else { bla_to_gpu(&levels) }
}

/// `|c + δ| − |c|` with no cancellation (the perturbation of a fold).
fn diffabs(c: f64, d: f64) -> f64 {
    if c >= 0.0 {
        if c + d >= 0.0 { d } else { -(2.0 * c + d) }
    } else if c + d > 0.0 {
        2.0 * c + d
    } else {
        -d
    }
}

/// `(Z + δ)^d − Z^d` in f64 by the binomial (Horner in δ).
fn pert_pow(d: u32, z: (f64, f64), e: (f64, f64)) -> (f64, f64) {
    let mul = |a: (f64, f64), b: (f64, f64)| (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0);
    let mut zp = vec![(1.0, 0.0)];
    for _ in 1..d {
        zp.push(mul(*zp.last().unwrap(), z));
    }
    let mut acc = (0.0, 0.0);
    for k in (1..=d).rev() {
        let c = (0..k).fold(1.0, |acc, i| acc * f64::from(d - i) / f64::from(i + 1));
        let t = zp[(d - k) as usize];
        acc = mul(acc, e);
        acc = (acc.0 + c * t.0, acc.1 + c * t.1);
    }
    mul(acc, e)
}

/// The exact perturbed step of a fold family in f64 (the CPU twin of the shader's arms): the fold's
/// perturbation is `diffabs`, on z before the power (Burning Ship) or on w = z^d after it.
pub fn fold_pert_step(formula: u32, z: (f64, f64), e: (f64, f64), dc: (f64, f64)) -> (f64, f64) {
    let (shape, d) = fold_shape(formula).expect("a fold family");
    let mul = |a: (f64, f64), b: (f64, f64)| (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0);
    let dw = match shape {
        Shape::BurningShip => {
            let f = (z.0.abs(), z.1.abs());
            pert_pow(d, f, (diffabs(z.0, e.0), diffabs(z.1, e.1)))
        }
        Shape::Tricorn => pert_pow(d, (z.0, -z.1), (e.0, -e.1)),
        _ => {
            let mut w = (1.0, 0.0);
            for _ in 0..d {
                w = mul(w, z);
            }
            let p = pert_pow(d, z, e);
            let im = if shape == Shape::Buffalo { diffabs(w.1, p.1) } else { p.1 };
            (diffabs(w.0, p.0), im)
        }
    };
    (dw.0 + dc.0, dw.1 + dc.1)
}

/// The reference fold-BLA render of one pixel in Mandelbrot mode, as the shader's floatexp loop: skip
/// with the highest valid level that neither runs past the reference nor overshoots the escape, else
/// a full step; after either, rebase onto the reference's start (`δz ← z`, as `Z₀ = 0`) when the
/// pixel's value has fallen below its offset or the reference runs out — Zhuoran's rule, exact for a
/// fold as for any step. `bla = false` is the same walk with no skips: plain perturbation, the
/// yardstick. In f64 (|δz| down to ~1e-300), so for views to ~1e290×.
pub fn bla_fold_iterate(
    orbit: &[[f32; 4]],
    levels: &[Vec<BlaFoldNode>],
    dc: (f64, f64),
    bailout2: f64,
    max_iter: u32,
    formula: u32,
    bla: bool,
) -> Option<f64> {
    bla_fold_walk(orbit, levels, dc, bailout2, max_iter, formula, bla, None)
}

/// [`bla_fold_iterate`], recording `(iteration, reference index, δz)` after every skip's landing
/// (`bla`) or every step (without), before any rebase, into `trace` — so a test can compare where a
/// skip landed with where the exact steps went, which a final escape count cannot tell at a chaotic
/// pixel.
#[allow(clippy::too_many_arguments)]
pub fn bla_fold_walk(
    orbit: &[[f32; 4]],
    levels: &[Vec<BlaFoldNode>],
    dc: (f64, f64),
    bailout2: f64,
    max_iter: u32,
    formula: u32,
    bla: bool,
    mut trace: Option<&mut Vec<(u32, u32, (f64, f64))>>,
) -> Option<f64> {
    let d = formula::power(formula);
    let dc_c = CFloatExp { re: FloatExp::from_f64(dc.0), im: FloatExp::from_f64(dc.1) };
    let mut dz = CFloatExp::ZERO;
    let (mut m, mut iter): (u32, u32) = (0, 0);
    let nstep = orbit.len().saturating_sub(1) as u32;
    // `sample_xy`, never a lane sum: a deep dip is stored in extended range, its exponent in a lane
    // (a 2^-300 dip summed reads as |z| ≈ 300 — an escape).
    let sample = |m: u32| sample_xy(&orbit[m as usize]);
    if nstep == 0 {
        return None;
    }
    loop {
        if iter >= max_iter {
            return None;
        }
        let dzmag = dz.abs();
        let mut applied = false;
        for l in (0..if bla { levels.len() } else { 0 }).rev() {
            if (m & ((1u32 << l) - 1)) != 0 {
                continue;
            }
            let Some(&node) = levels[l].get((m >> l) as usize) else { continue };
            if m + node.span >= nstep || !dzmag.lt(node.r) {
                continue;
            }
            let ndz = node.a.apply(dz) + node.b.apply(dc_c);
            let zn = sample(m + node.span);
            let (zx, zy) = (zn.0 + ndz.re.to_f64(), zn.1 + ndz.im.to_f64());
            if zx * zx + zy * zy > bailout2 {
                continue;
            }
            dz = ndz;
            m += node.span;
            iter += node.span;
            if let Some(t) = trace.as_deref_mut() {
                t.push((iter, m, (dz.re.to_f64(), dz.im.to_f64())));
            }
            applied = true;
            break;
        }
        if !applied {
            let e = fold_pert_step(formula, sample(m), (dz.re.to_f64(), dz.im.to_f64()), dc);
            dz = CFloatExp { re: FloatExp::from_f64(e.0), im: FloatExp::from_f64(e.1) };
            m += 1;
            iter += 1;
            if !bla {
                if let Some(t) = trace.as_deref_mut() {
                    t.push((iter, m, e));
                }
            }
            let zn = sample(m);
            let (zx, zy) = (zn.0 + e.0, zn.1 + e.1);
            let mag2 = zx * zx + zy * zy;
            if mag2 > bailout2 {
                let nu = (mag2.ln() * 0.5 / std::f64::consts::LN_2).ln() / f64::from(d).ln();
                return Some(iter as f64 + 1.0 - nu);
            }
        }
        let zn = sample(m);
        let z = (zn.0 + dz.re.to_f64(), zn.1 + dz.im.to_f64());
        let (dx, dy) = (dz.re.to_f64(), dz.im.to_f64());
        if z.0 * z.0 + z.1 * z.1 < dx * dx + dy * dy || m + 1 >= nstep {
            dz = CFloatExp { re: FloatExp::from_f64(z.0), im: FloatExp::from_f64(z.1) };
            m = 0;
        }
    }
}

#[cfg(test)]
mod tests;
