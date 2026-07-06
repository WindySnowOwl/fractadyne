//! Perturbation deep-zoom numerics: the bignum reference orbit (`step_bf` / `reference_orbit`),
//! series approximation, the BLA tree, nucleus / period finding, and multi-reference glitch
//! correction. The heart of the deep-zoom engine; keyed on the `u32` formula ids in
//! [`crate::formula`].

use crate::bignum::*;
use crate::floatexp::*;
use crate::formula;
use astro_float::BigFloat;

/// Split an `f64` into a `(hi, lo)` `f32` pair (df64, ~14 digits).
fn split_df64(v: f64) -> (f32, f32) {
    let hi = v as f32;
    let lo = (v - hi as f64) as f32;
    (hi, lo)
}

/// Complex multiply in arbitrary precision: `(ax+i·ay)·(bx+i·by)`.
pub(crate) fn cmul_bf(
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
pub(crate) fn step_bf(
    zx: &BigFloat,
    zy: &BigFloat,
    cx: &BigFloat,
    cy: &BigFloat,
    formula: u32,
    p: usize,
) -> (BigFloat, BigFloat) {
    // Every perturbation-capable family now shares one `Field`-generic step (`crate::fractal`),
    // which reproduces the former hand-written arms bit-for-bit (guarded by the exact SA
    // cross-check tests + goldens). Phoenix / Newton never reach here (they use `phoenix_step_bf`
    // / the direct path). An unknown id defensively falls back to the Mandelbrot step — the
    // former `_` default.
    crate::fractal::trait_step(formula, zx, zy, cx, cy, p)
        .unwrap_or_else(|| crate::fractal::trait_step(formula::MANDELBROT, zx, zy, cx, cy, p).unwrap())
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
        if formula == formula::NEWTON {
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
        // Migrated families ({Mandelbrot, Multibrot3/4/5, Burning Ship}) go through the one
        // `Field`-generic step shared with `step_bf` (bit-identical in f64); the rest stay inline.
        let (nx, ny) = if let Some(r) = crate::fractal::trait_step(formula, &zx, &zy, &c.0, &c.1, ()) {
            r
        } else {
            // Only Phoenix (two-term, needs the previous iterate) remains inline; everything else
            // is migrated to the shared step above, so a non-Phoenix id here is the defensive
            // Mandelbrot default.
            let (sx, sy) = (zx * zx - zy * zy, 2.0 * zx * zy); // z²
            match formula {
                formula::PHOENIX => (sx + c.0 - 0.5 * px, sy + c.1 - 0.5 * py), // Phoenix (p=−0.5)
                _ => (sx + c.0, sy + c.1),                                      // (Mandelbrot default)
            }
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
        let (nzx, nzy) = if formula == formula::PHOENIX {
            phoenix_step_bf(&zx, &zy, &zpx, &zpy, cx, cy, p)
        } else {
            step_bf(&zx, &zy, cx, cy, formula, p)
        };
        if formula == formula::PHOENIX {
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
        formula::MULTIBROT3 => 3,
        formula::MULTIBROT4 => 4,
        formula::MULTIBROT5 => 5,
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

/// One BLA node: the linear map `δz' = A·δz + B·δc`, valid while `|δz| ≤ r`, covering `span`
/// consecutive reference iterations starting at this node's (aligned) index.
///
/// The `agg_*` fields are the per-node **aux coloring aggregates** over the node's landing iterates
/// `Z_{start+1..=start+span}` (a pure function of the reference orbit, since `z_k ≈ Z_k` while the
/// skip is valid): the min trap distance, the Σ triangle-inequality terms, and the Σ stripe terms.
/// A pixel folds these into its running aux stat in O(1) on a skip (aux⇄BLA coexistence) instead of
/// dropping the skipped iterations. They compose associatively in [`bla_merge`] (min / sum).
#[derive(Clone, Copy)]
pub struct BlaNode {
    pub a: CFloatExp,
    pub b: CFloatExp,
    pub r: FloatExp,
    pub span: u32,
    pub agg_trap: f64,
    pub agg_tia: f64,
    pub agg_stripe: f64,
}

/// Coloring parameters the per-node aux aggregates depend on. Only the `agg_*` lanes depend on
/// these (not the A/B/r geometry), so they can be recomputed cheaply when coloring changes while
/// the tree geometry stays cached. `Default` yields inert values for callers that don't fold aux.
#[derive(Clone, Copy)]
pub struct AuxAggParams {
    pub trap_type: u32, // 0 = point |z|, 1 = cross min(|x|,|y|), 2 = circle ||z|−1|
    pub stripe_freq: f64,
    pub cmag: f64, // |c| of the reference (triangle-inequality)
    pub power: f64, // formula degree (triangle-inequality)
}
impl Default for AuxAggParams {
    fn default() -> Self {
        AuxAggParams { trap_type: 0, stripe_freq: 1.0, cmag: 0.0, power: 2.0 }
    }
}

// Per-iterate aux stat contributions, mirroring the GPU `aux_step` (used to seed the BLA
// aggregates; the shader folds the same quantities on a skip).
pub(crate) fn aux_trap_dist(zx: f64, zy: f64, trap_type: u32) -> f64 {
    match trap_type {
        1 => zx.abs().min(zy.abs()),
        2 => ((zx * zx + zy * zy).sqrt() - 1.0).abs(),
        _ => (zx * zx + zy * zy).sqrt(),
    }
}
pub(crate) fn aux_stripe_term(zx: f64, zy: f64, freq: f64) -> f64 {
    0.5 + 0.5 * (freq * zy.atan2(zx)).sin()
}
pub(crate) fn aux_tia_term(prev_abs: f64, cur_abs: f64, cmag: f64, power: f64) -> f64 {
    let m = prev_abs.max(1e-12).powf(power);
    let lower = (m - cmag).abs();
    let upper = m + cmag;
    ((cur_abs - lower) / (upper - lower).max(1e-9)).clamp(0.0, 1.0)
}

/// Merge two consecutive BLA nodes (`x` then `y`). Composition `A=A_y·A_x`, `B=A_y·B_x+B_y`;
/// validity `|δz|≤r_x` **and** `|A_x δz + B_x δc|≤r_y` ⇒ `r = min(r_x, (r_y − |B_x|·δc_max)/|A_x|)`.
fn bla_merge(x: BlaNode, y: BlaNode, dc_max: FloatExp) -> BlaNode {
    let a = y.a * x.a;
    let b = y.a * x.b + y.b;
    let a1 = x.a.abs();
    let b1 = x.b.abs();
    let t = y.r - b1 * dc_max;
    let t = if t.m < 0.0 { FloatExp::ZERO } else { t };
    let r2 = if a1.m == 0.0 { FloatExp::ZERO } else { t * a1.recip() };
    let r = if x.r.lt(r2) { x.r } else { r2 };
    BlaNode {
        a,
        b,
        r,
        span: x.span + y.span,
        // Aux aggregates compose associatively: trap is a running min, TIA/stripe running sums.
        // `x` covers the earlier range, so its TIA carries the correct entry `prev` (|Z| at the
        // merged node's start); simple addition preserves the whole chain across the join.
        agg_trap: x.agg_trap.min(y.agg_trap),
        agg_tia: x.agg_tia + y.agg_tia,
        agg_stripe: x.agg_stripe + y.agg_stripe,
    }
}

/// Build the BLA binary tree for a **Mandelbrot** reference `orbit` (df32 `[x_hi,y_hi,x_lo,
/// y_lo]`). `dc_max` is the worst-case `|δc|` over the view; `eps` the per-step linear
/// tolerance (smaller ⇒ more accurate but fewer skips). `levels[l][j]` covers the steps
/// starting at `j·2^l`; level 0 has one node per step `n` (using `Zₙ`), higher levels merge
/// pairs (an odd tail carries up with its smaller span).
pub fn build_bla_mandel(
    orbit: &[[f32; 4]],
    dc_max: FloatExp,
    eps: f64,
    aux: AuxAggParams,
) -> Vec<Vec<BlaNode>> {
    let nstep = orbit.len().saturating_sub(1);
    if nstep == 0 {
        return Vec::new();
    }
    let one = CFloatExp { re: FloatExp::from_f64(1.0), im: FloatExp::ZERO };
    let mut lvl0 = Vec::with_capacity(nstep);
    for n in 0..nstep {
        let z = orbit[n];
        let zr = z[0] as f64 + z[2] as f64; // Z_n real
        let zi = z[1] as f64 + z[3] as f64; // Z_n imag
        let a = CFloatExp { re: FloatExp::from_f64(2.0 * zr), im: FloatExp::from_f64(2.0 * zi) };
        let r = a.abs().mul_f64(eps); // |2Z|·eps : drops δz² with rel error ≤ eps
        // Aux aggregate for this node's single landing iterate Z_{n+1} (the shader accumulates the
        // POST-step value). TIA's `prev` is |Z_n|; node 0 lands on the global first iterate z_1,
        // whose TIA is skipped by the `n>=1` guard, so its TIA seed is 0.
        let z1 = orbit[n + 1];
        let (z1r, z1i) = (z1[0] as f64 + z1[2] as f64, z1[1] as f64 + z1[3] as f64);
        let agg_trap = aux_trap_dist(z1r, z1i, aux.trap_type);
        let agg_stripe = aux_stripe_term(z1r, z1i, aux.stripe_freq);
        let agg_tia = if n == 0 {
            0.0
        } else {
            aux_tia_term(
                (zr * zr + zi * zi).sqrt(),
                (z1r * z1r + z1i * z1i).sqrt(),
                aux.cmag,
                aux.power,
            )
        };
        lvl0.push(BlaNode { a, b: one, r, span: 1, agg_trap, agg_tia, agg_stripe });
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
            // 4th vec4: span + the aux coloring aggregates (was 3 unused lanes) — folded on a skip
            // so stripe/TIA/trap can coexist with BLA iteration-skipping (agg over Z_{start+1..span}).
            out.push([
                node.span as f32,
                node.agg_trap as f32,
                node.agg_tia as f32,
                node.agg_stripe as f32,
            ]);
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
            let ndz = node.a * dz + node.b * dc_c; // δz = A·δz + B·δc
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
        dz = two_z * dz + dz * dz + dc_c;
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
        let (nzx, nzy) = if formula == formula::PHOENIX {
            phoenix_step_bf(&zx, &zy, &zpx, &zpy, cx, cy, p)
        } else {
            step_bf(&zx, &zy, cx, cy, formula, p)
        };
        if formula == formula::PHOENIX {
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
        formula::MANDELBROT => Some(2),
        formula::MULTIBROT3 => Some(3),
        formula::MULTIBROT4 => Some(4),
        formula::MULTIBROT5 => Some(5),
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

// ---------------- aux⇄BLA coexistence: Phase-0 de-risk oracle -----------------------------------
// Measures the per-pixel error of folding a per-BLA-node reference-orbit aggregate on each skip
// (vs exact per-iteration accumulation), so we know BEFORE any GPU work whether the aux coloring
// stats can safely ride BLA/SA iteration-skipping. Trap is the canary: a min over ~the same values,
// so its error must be tiny; a large trap error means the ORACLE is buggy, not the method.
#[cfg(test)]
mod aux_bla_oracle {
    use super::*;

    const FREQ: f64 = 6.0; // stripe angular frequency
    const POWER: f64 = 2.0; // Mandelbrot TIA power

    #[inline]
    fn mag(x: f64, y: f64) -> f64 {
        (x * x + y * y).sqrt()
    }
    #[inline]
    fn stripe_term(x: f64, y: f64) -> f64 {
        0.5 + 0.5 * (FREQ * y.atan2(x)).sin()
    }
    #[inline]
    fn tia_term(prev: f64, cur: f64, cmag: f64) -> f64 {
        let m = prev.max(1e-12).powf(POWER);
        let lower = (m - cmag).abs();
        let upper = m + cmag;
        ((cur - lower) / (upper - lower).max(1e-9)).clamp(0.0, 1.0)
    }
    #[inline]
    fn zval(orbit: &[[f32; 4]], k: u32) -> (f64, f64) {
        let z = orbit[k as usize];
        (z[0] as f64 + z[2] as f64, z[1] as f64 + z[3] as f64)
    }

    /// Brute-force a node's aggregate over its landing range Z_{start+1..=start+span} — the ground
    /// truth the precomputed `BlaNode.agg_*` (seeded + merged in `build_bla_mandel`) must match. The
    /// `k >= 2` guard mirrors the global TIA `n>=1` skip of the very first landing iterate z_1.
    fn brute_node_agg(orbit: &[[f32; 4]], start: u32, span: u32, cmag: f64) -> (f64, f64, f64) {
        let (mut trap, mut ss, mut ts) = (1e30f64, 0.0f64, 0.0f64);
        for k in (start + 1)..=(start + span) {
            let (zx, zy) = zval(orbit, k);
            trap = trap.min(mag(zx, zy)); // point trap
            ss += stripe_term(zx, zy);
            if k >= 2 {
                let (px, py) = zval(orbit, k - 1);
                ts += tia_term(mag(px, py), mag(zx, zy), cmag);
            }
        }
        (trap, ts, ss)
    }

    /// Running aux state mirroring the shader's `aux_step` (point trap, stripe, power-2 TIA).
    #[derive(Clone, Copy)]
    struct Aux {
        trap: f64,
        ssum: f64,
        tsum: f64,
        n: f64,
        prev_abs: f64,
    }
    impl Aux {
        fn init(z0: (f64, f64)) -> Self {
            Aux { trap: 1e30, ssum: 0.0, tsum: 0.0, n: 0.0, prev_abs: mag(z0.0, z0.1) }
        }
        /// One *actual* post-step iterate z (exact path, and BLA full steps).
        fn push_exact(&mut self, z: (f64, f64), cmag: f64) {
            let cur = mag(z.0, z.1);
            self.trap = self.trap.min(cur);
            self.ssum += stripe_term(z.0, z.1);
            if self.n >= 1.0 {
                self.tsum += tia_term(self.prev_abs, cur, cmag);
            }
            self.prev_abs = cur;
            self.n += 1.0;
        }
        /// Fold a precomputed per-node aggregate on a skip (m → nm) — the actual GPU behavior: add
        /// the node's Σ/min, advance the count by its span, and restore the exit `prev_abs` to the
        /// *actual* landing |z_nm| (exact seam, since the shader carries δz there). The TIA entry
        /// seam (reference |Z_m| as the first `prev`) is already baked into `node.agg_tia`.
        fn fold_node(&mut self, node: &BlaNode, landing_abs: f64) {
            self.trap = self.trap.min(node.agg_trap);
            self.ssum += node.agg_stripe;
            self.tsum += node.agg_tia;
            self.n += node.span as f64;
            self.prev_abs = landing_abs;
        }
        fn stripe_avg(&self) -> f64 {
            self.ssum / self.n.max(1.0)
        }
        fn tia_avg(&self) -> f64 {
            self.tsum / (self.n - 1.0).max(1.0)
        }
    }

    /// Exact perturbation iteration (no skips), accumulating aux at every step.
    fn run_exact(
        orbit: &[[f32; 4]],
        dc: (f64, f64),
        bailout2: f64,
        max_iter: u32,
        cmag: f64,
    ) -> (Option<f64>, Aux) {
        let dc_c = CFloatExp { re: FloatExp::from_f64(dc.0), im: FloatExp::from_f64(dc.1) };
        let mut dz = CFloatExp { re: FloatExp::ZERO, im: FloatExp::ZERO };
        let mut m: u32 = 0;
        let nstep = orbit.len().saturating_sub(1) as u32;
        let mut aux = Aux::init(bla_full_z(orbit, 0, &dz));
        loop {
            if m >= max_iter || m >= nstep {
                return (None, aux);
            }
            let (zrx, zry) = zval(orbit, m);
            let two_z = CFloatExp {
                re: FloatExp::from_f64(2.0 * zrx),
                im: FloatExp::from_f64(2.0 * zry),
            };
            dz = two_z * dz + dz * dz + dc_c;
            m += 1;
            let (zx, zy) = bla_full_z(orbit, m, &dz);
            aux.push_exact((zx, zy), cmag);
            let mag2 = zx * zx + zy * zy;
            if mag2 > bailout2 {
                let nu = (mag2.ln() * 0.5 / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
                return (Some(m as f64 + 1.0 - nu), aux);
            }
        }
    }

    /// BLA-skipping iteration mirroring `bla_iterate`, folding the reference aggregate on skips.
    /// Returns (escape, aux, skips, full_steps).
    fn run_folded(
        orbit: &[[f32; 4]],
        levels: &[Vec<BlaNode>],
        dc: (f64, f64),
        bailout2: f64,
        max_iter: u32,
        cmag: f64,
    ) -> (Option<f64>, Aux, u32, u32) {
        let dc_c = CFloatExp { re: FloatExp::from_f64(dc.0), im: FloatExp::from_f64(dc.1) };
        let mut dz = CFloatExp { re: FloatExp::ZERO, im: FloatExp::ZERO };
        let mut m: u32 = 0;
        let nstep = orbit.len().saturating_sub(1) as u32;
        let mut aux = Aux::init(bla_full_z(orbit, 0, &dz));
        let (mut skips, mut fulls) = (0u32, 0u32);
        loop {
            if m >= max_iter || m >= nstep {
                return (None, aux, skips, fulls);
            }
            let dzmag = dz.abs();
            let mut applied = false;
            for l in (0..levels.len()).rev() {
                let step = 1u32 << l;
                if (m & (step - 1)) != 0 {
                    continue;
                }
                let Some(&node) = levels[l].get((m >> l) as usize) else { continue };
                if m + node.span > nstep || !dzmag.lt(node.r) {
                    continue;
                }
                let ndz = node.a * dz + node.b * dc_c;
                let nm = m + node.span;
                let (zx, zy) = bla_full_z(orbit, nm, &ndz);
                if zx * zx + zy * zy > bailout2 {
                    continue;
                }
                aux.fold_node(&node, mag(zx, zy));
                dz = ndz;
                m = nm;
                applied = true;
                skips += 1;
                break;
            }
            if applied {
                continue;
            }
            let (zrx, zry) = zval(orbit, m);
            let two_z = CFloatExp {
                re: FloatExp::from_f64(2.0 * zrx),
                im: FloatExp::from_f64(2.0 * zry),
            };
            dz = two_z * dz + dz * dz + dc_c;
            m += 1;
            fulls += 1;
            let (zx, zy) = bla_full_z(orbit, m, &dz);
            aux.push_exact((zx, zy), cmag);
            let mag2 = zx * zx + zy * zy;
            if mag2 > bailout2 {
                let nu = (mag2.ln() * 0.5 / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
                return (Some(m as f64 + 1.0 - nu), aux, skips, fulls);
            }
        }
    }

    /// Sweep one view; print per-method fold error + the view's min |Z| (the stripe stressor).
    /// Returns the trap max error (the oracle-bug canary) so the caller can assert on it.
    fn analyze(label: &str, cx_str: &str, cy_str: &str, octaves: u64, dc_ext: f64) -> f64 {
        let p = crate::precision_for_octaves(octaves);
        let cx = crate::parse_bf(cx_str).unwrap();
        let cy = crate::parse_bf(cy_str).unwrap();
        let z0 = BigFloat::from_f64(0.0, p);
        let max_iter = 6000u32;
        let (orbit, len) = reference_orbit(&z0, &z0, &cx, &cy, formula::MANDELBROT, max_iter, p);
        let cmag = mag(to_f64(&cx), to_f64(&cy));
        let bailout2 = 1.0e10f64;
        let dc_max = FloatExp::from_f64(dc_ext * 1.5);
        let aux_p = AuxAggParams { trap_type: 0, stripe_freq: FREQ, cmag, power: POWER };
        let levels = build_bla_mandel(&orbit, dc_max, 1.0e-6, aux_p);

        // Phase-1 precompute check: every node's seeded+merged aggregate must equal a brute-force
        // over its landing range (catches a wrong seed index, a broken merge, or a mis-placed guard).
        for (l, level) in levels.iter().enumerate() {
            for (j, node) in level.iter().enumerate() {
                let start = (j as u32) << l;
                let (bt, bx, bs) = brute_node_agg(&orbit, start, node.span, cmag);
                let close = |a: f64, b: f64| (a - b).abs() <= 1e-9 * a.abs().max(1.0) + 1e-12;
                assert!(close(bt, node.agg_trap), "{label}: l{l} j{j} trap {bt} vs {}", node.agg_trap);
                assert!(close(bx, node.agg_tia), "{label}: l{l} j{j} tia {bx} vs {}", node.agg_tia);
                assert!(close(bs, node.agg_stripe), "{label}: l{l} j{j} stripe {bs} vs {}", node.agg_stripe);
            }
        }

        // Min |Z| along the reference (how ill-conditioned stripe's arg gets here). Skip Z_0 = 0
        // (the trivial Mandelbrot critical point) — aux accumulates z_{k≥1}, not z_0.
        let min_z = orbit
            .iter()
            .take(len as usize)
            .skip(1)
            .map(|z| mag(z[0] as f64 + z[2] as f64, z[1] as f64 + z[3] as f64))
            .fold(f64::INFINITY, f64::min);

        let grid = 48u32;
        let (mut tmax, mut tsum) = (0.0f64, 0.0f64);
        let (mut smax, mut ssum) = (0.0f64, 0.0f64);
        let (mut xmax, mut xsum) = (0.0f64, 0.0f64);
        let (mut cnt, mut esc_mismatch, mut tot_skips, mut tot_full) = (0u32, 0u32, 0u64, 0u64);
        for i in 0..grid {
            for j in 0..grid {
                let dcx = -dc_ext + 2.0 * dc_ext * (i as f64) / ((grid - 1) as f64);
                let dcy = -dc_ext + 2.0 * dc_ext * (j as f64) / ((grid - 1) as f64);
                let (ee, ea) = run_exact(&orbit, (dcx, dcy), bailout2, max_iter, cmag);
                let (fe, fa, sk, fl) = run_folded(&orbit, &levels, (dcx, dcy), bailout2, max_iter, cmag);
                let (Some(ei), Some(fi)) = (ee, fe) else { continue };
                cnt += 1;
                tot_skips += sk as u64;
                tot_full += fl as u64;
                if (ei - fi).abs() > 0.5 {
                    esc_mismatch += 1;
                }
                tmax = tmax.max((ea.trap - fa.trap).abs());
                tsum += (ea.trap - fa.trap).abs();
                smax = smax.max((ea.stripe_avg() - fa.stripe_avg()).abs());
                ssum += (ea.stripe_avg() - fa.stripe_avg()).abs();
                xmax = xmax.max((ea.tia_avg() - fa.tia_avg()).abs());
                xsum += (ea.tia_avg() - fa.tia_avg()).abs();
            }
        }
        let c = cnt.max(1) as f64;
        eprintln!("\n=== {label}: ref len {len}, min|Z| {min_z:.3e}, {cnt} escaping px, {esc_mismatch} escape mismatch ===");
        eprintln!("  BLA: {tot_skips} skips / {tot_full} full steps");
        eprintln!("  trap   : max {:.3e}  mean {:.3e}", tmax, tsum / c);
        eprintln!("  stripe : max {:.3e}  mean {:.3e}   (0..1)", smax, ssum / c);
        eprintln!("  TIA    : max {:.3e}  mean {:.3e}   (0..1)", xmax, xsum / c);
        assert!(tot_skips > 0, "{label}: BLA never skipped — oracle exercised no folds");
        tmax
    }

    #[test]
    fn aux_bla_fold_error() {
        // The seahorse valley is a structured boundary point whose orbit spirals near 0 (low |Z|),
        // so it already exercises stripe's ill-conditioned regime. Two pixel extents (shallower =
        // larger δc = looser BLA) bracket how the fold error scales with the perturbation size.
        let mut trap_worst = 0.0f64;
        let sx = "-0.7436438870371587047521915061147707";
        let sy = "0.131825904205311970493132056385139";
        trap_worst = trap_worst.max(analyze("seahorse δc~2e-12", sx, sy, 40, 2.0e-12));
        trap_worst = trap_worst.max(analyze("seahorse δc~2e-6", sx, sy, 40, 2.0e-6));
        // Canary across all views: trap's fold is a min over ~identical values → tiny error;
        // a large value signals an oracle bug (indexing / seam), not a method verdict.
        assert!(
            trap_worst < 0.05,
            "trap fold error {trap_worst:.3e} too large — oracle bug, not a method verdict"
        );
    }
}
