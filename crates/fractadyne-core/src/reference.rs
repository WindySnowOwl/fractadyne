//! Perturbation deep-zoom numerics: the bignum reference orbit (`step_bf` / `reference_orbit`),
//! series approximation, the BLA tree, nucleus / period finding, and multi-reference glitch
//! correction. The heart of the deep-zoom engine; keyed on the `u32` formula ids in
//! [`crate::formula`].

use crate::backend::RefBackend;
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

/// Below this magnitude a sample's plain-df32 storage degrades (f32 min normal is ~1.2e-38, and
/// GPU arithmetic flushes subnormals to zero), so it is stored in the extended form instead.
const EXT_SAMPLE_THRESHOLD: f64 = 1.0e-36;

/// Pack one orbit sample for the GPU.
///
/// Normal samples are the classic df32 pair-of-pairs `[re_hi, im_hi, re_lo, im_lo]`. A sample whose
/// magnitude is below [`EXT_SAMPLE_THRESHOLD`] would flush toward zero in that form — and a
/// deep-minibrot-family reference passes through such near-nucleus dips PERIODICALLY (validation
/// corpus 11–15: |Z| ~ 1e-71 every 4383 iterations). Zeroing a dip drops the `2Z·δz` term of the
/// perturbation recurrence at exactly those iterations; once the view is deeper than
/// ~(dip magnitude ÷ per-period orbit growth), the dropped term dominates the true `δz'` and every
/// pixel's accumulated divergence is annihilated each period — the whole frame renders as interior
/// (corpus 14/15 went uniform past ~1e142× while 13 at 1e141× still matched Fraktaler-3). Such
/// samples are stored EXTENDED-RANGE instead: `[NaN marker, exponent, m_re, m_im]` with mantissas
/// scaled by 2^-exponent (the leading one normalized to [1,2)); the shader decodes via `orbit_fe`.
/// NaN can never occur in a normal sample (the bignum pipeline yields finite values), so the marker
/// is unambiguous.
pub(crate) fn pack_sample(xv: f64, yv: f64) -> [f32; 4] {
    let mag = xv.abs().max(yv.abs());
    if mag != 0.0 && mag < EXT_SAMPLE_THRESHOLD {
        // Marker layout [0.0, k, m_re + 4.0, m_im]: PROVABLY unambiguous against a legit df32
        // sample, whose invariant is `hi == 0.0 ⇒ lo == 0.0` (lo = f32(v − hi) = f32(v) = hi),
        // while here lane 2 is ≥ 2.0. Deliberately NOT a NaN marker: WGSL gives no NaN
        // guarantees — shader compilers may assume finite floats and fold `x != x` to false,
        // which silently disabled the first version of this encoding on the GPU.
        let k = mag.log2().floor() as i32;
        let s = (2.0f64).powi(-k);
        return [0.0, k as f32, (xv * s) as f32 + 4.0, (yv * s) as f32];
    }
    let (xh, xl) = split_df64(xv);
    let (yh, yl) = split_df64(yv);
    [xh, yh, xl, yl]
}

/// Decode either sample form back to `(re, im)` as `f64` (whose range covers the extended form).
/// CPU-side consumers (BLA build, aux aggregates, tests) must use this rather than summing the
/// lanes, or an extended sample's NaN marker poisons the arithmetic.
pub fn sample_xy(s: &[f32; 4]) -> (f64, f64) {
    if s[0] == 0.0 && s[2].abs() >= 2.0 {
        let sc = (2.0f64).powi(s[1] as i32);
        return ((s[2] as f64 - 4.0) * sc, s[3] as f64 * sc);
    }
    (s[0] as f64 + s[2] as f64, s[1] as f64 + s[3] as f64)
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
    step_gen::<BigFloat>(zx, zy, cx, cy, formula, <BigFloat as RefBackend>::ctx_for(p))
}

/// [`step_bf`], generic over the arithmetic [`RefBackend`].
///
/// Every perturbation-capable family shares one `Field`-generic step (`crate::fractal`), which
/// reproduces the former hand-written arms bit-for-bit (guarded by the exact SA cross-check tests
/// + goldens). Phoenix / Newton never reach here (they use [`phoenix_step_gen`] / the direct
/// path). An unknown id defensively falls back to the Mandelbrot step — the former `_` default.
pub(crate) fn step_gen<B: RefBackend>(
    zx: &B,
    zy: &B,
    cx: &B,
    cy: &B,
    formula: u32,
    ctx: B::Ctx,
) -> (B, B) {
    crate::fractal::trait_step(formula, zx, zy, cx, cy, ctx)
        .unwrap_or_else(|| crate::fractal::trait_step(formula::MANDELBROT, zx, zy, cx, cy, ctx).unwrap())
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
    phoenix_step_gen::<BigFloat>(zx, zy, zpx, zpy, cx, cy, <BigFloat as RefBackend>::ctx_for(p))
}

/// [`phoenix_step_bf`], generic over the arithmetic [`RefBackend`]. The operation order is the
/// former hand-written `BigFloat` body verbatim — `fdouble` is `double_bf` (an exponent bump), and
/// `fmul`/`fadd`/`fsub` are the same rounded ops in the same sequence — so this is bit-identical,
/// not merely equivalent.
#[allow(clippy::too_many_arguments)]
fn phoenix_step_gen<B: RefBackend>(
    zx: &B,
    zy: &B,
    zpx: &B,
    zpy: &B,
    cx: &B,
    cy: &B,
    ctx: B::Ctx,
) -> (B, B) {
    let x2 = zx.fmul(zx, ctx);
    let y2 = zy.fmul(zy, ctx);
    let sx = x2.fsub(&y2, ctx);
    let sy = zx.fmul(zy, ctx).fdouble();
    let half = B::from_f64(0.5, ctx);
    let hpx = zpx.fmul(&half, ctx);
    let hpy = zpy.fmul(&half, ctx);
    (sx.fadd(cx, ctx).fsub(&hpx, ctx), sy.fadd(cy, ctx).fsub(&hpy, ctx))
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
    let (o, l, _) = reference_orbit_t(z0x, z0y, cx, cy, formula, max_iter, p);
    (o, l)
}

/// As [`reference_orbit`], in an explicitly named backend. See [`reference_orbit_t_in`].
#[allow(clippy::too_many_arguments)]
pub fn reference_orbit_in(
    backend: crate::BackendChoice,
    z0x: &BigFloat,
    z0y: &BigFloat,
    cx: &BigFloat,
    cy: &BigFloat,
    formula: u32,
    max_iter: u32,
    p: usize,
) -> (Vec<[f32; 4]>, u32) {
    let (o, l, _) = reference_orbit_t_in(backend, z0x, z0y, cx, cy, formula, max_iter, p);
    (o, l)
}

/// The full-precision running state at the end of a reference orbit, so a later call can **extend**
/// it (via [`extend_reference_orbit`]) to a larger `max_iter` without recomputing the shared prefix
/// — the deep-zoom win, since the orbit build (`max_iter × step` in bignum) dominates a deep frame.
/// `escaped` marks a *complete* orbit (bailed the escape radius): nothing more to extend.
#[derive(Clone)]
pub struct OrbitTail {
    pub zx: BigFloat,
    pub zy: BigFloat,
    /// Previous iterate `Z_{n-1}` (Phoenix's two-term recurrence; zero/unused for other formulas).
    pub zpx: BigFloat,
    pub zpy: BigFloat,
    pub escaped: bool,
    /// Which [`crate::BackendChoice`] built this tail (its `RefBackend::BIT`).
    ///
    /// `extend_reference_orbit` resumes in THIS backend rather than the currently selected one.
    /// The extend contract is byte-identity with a fresh build, and the only way to keep that
    /// promise across a mid-session backend switch is to finish the orbit the way it started.
    pub backend: u32,
}

/// Append `Z_{n+1..max_iter}` (df64 samples) to `out`, which already holds `Z_0..Z_n`, iterating from
/// the running state `(zx, zy)` with previous iterate `(zpx, zpy)`. Returns the final [`OrbitTail`].
/// Shared by the fresh build and the extend path so both emit **byte-identical** samples.
#[allow(clippy::too_many_arguments)]
fn run_orbit(
    out: &mut Vec<[f32; 4]>,
    bit: u32,
    zx: BigFloat,
    zy: BigFloat,
    zpx: BigFloat,
    zpy: BigFloat,
    cx: &BigFloat,
    cy: &BigFloat,
    formula: u32,
    n: u32,
    max_iter: u32,
    p: usize,
) -> OrbitTail {
    dispatch_orbit(bit, out, zx, zy, zpx, zpy, cx, cy, formula, n, max_iter, p)
        .expect("a BackendChoice variant only exists when its backend is compiled in")
}

/// Run the orbit in the backend identified by `bit`. The ONE place the backend is chosen.
///
/// An unrecognised `bit` cannot arise from a fresh build (it comes from the selection, which is a
/// typed enum); it can only reach here from an [`OrbitTail`] tagged by a backend this binary does
/// not have. That is not reachable today — a tail never leaves the process — and if it ever
/// becomes reachable, falling back to a *different* backend would silently break the extend
/// contract, so it refuses instead (see the caller).
#[allow(clippy::too_many_arguments)]
fn dispatch_orbit(
    bit: u32,
    out: &mut Vec<[f32; 4]>,
    zx: BigFloat,
    zy: BigFloat,
    zpx: BigFloat,
    zpy: BigFloat,
    cx: &BigFloat,
    cy: &BigFloat,
    formula: u32,
    n: u32,
    max_iter: u32,
    p: usize,
) -> Option<OrbitTail> {
    match bit {
        0 => Some(run_orbit_carrier::<BigFloat>(out, zx, zy, zpx, zpy, cx, cy, formula, n, max_iter, p)),
        #[cfg(feature = "rug")]
        1 => {
            // Prefer the backend's allocation-free loop where it has one; fall back to the generic
            // path otherwise, which is still MPFR — just allocating per operation. The two are held
            // byte-identical by the cross-backend matrix, which covers every formula id.
            let fast = crate::backend_rug::try_run_orbit_inplace(
                out, &zx, &zy, cx, cy, formula, n, max_iter, p,
            );
            match fast {
                Some((tzx, tzy, escaped)) => {
                    // The fast path bypasses `run_orbit_carrier`, which is where the observation
                    // is normally recorded -- so record it here, after the work, or the backend
                    // stamp silently under-reports whenever the fast path is the one that ran.
                    crate::backend::note_observed::<rug::Float>();
                    Some(OrbitTail {
                    zx: tzx,
                    zy: tzy,
                    zpx,
                    zpy,
                    escaped,
                    backend: <rug::Float as RefBackend>::BIT,
                    })
                }
                None => Some(run_orbit_carrier::<rug::Float>(
                    out, zx, zy, zpx, zpy, cx, cy, formula, n, max_iter, p,
                )),
            }
        }
        _ => None,
    }
}

/// Convert the carrier state into backend `B`, run the loop, convert the tail back.
///
/// **This is the only place a backend swap touches.** Dispatch is once per orbit build, and the
/// conversions are O(p) against `max_iter` bignum steps inside — free at any depth we render. For
/// `B = BigFloat` every conversion is a `clone`, so the result is the former hand-written loop's
/// output verbatim; [`OrbitTail`] therefore stays in the carrier type and an `extend` can resume
/// from a tail regardless of which backend produced it.
#[allow(clippy::too_many_arguments)]
fn run_orbit_carrier<B: RefBackend>(
    out: &mut Vec<[f32; 4]>,
    zx: BigFloat,
    zy: BigFloat,
    zpx: BigFloat,
    zpy: BigFloat,
    cx: &BigFloat,
    cy: &BigFloat,
    formula: u32,
    n: u32,
    max_iter: u32,
    p: usize,
) -> OrbitTail {
    let ctx = B::ctx_for(p);
    let (bzx, bzy) = (B::from_carrier(&zx, ctx), B::from_carrier(&zy, ctx));
    let (bzpx, bzpy) = (B::from_carrier(&zpx, ctx), B::from_carrier(&zpy, ctx));
    let (bcx, bcy) = (B::from_carrier(cx, ctx), B::from_carrier(cy, ctx));
    let (zx, zy, zpx, zpy, escaped) =
        run_orbit_gen::<B>(out, bzx, bzy, bzpx, bzpy, &bcx, &bcy, formula, n, max_iter, ctx);
    // Stamp AFTER the work, not before: `crate::backend::status_line` is quoted by the startup
    // log, crash reports and every gate, and must report what ran rather than what was asked for.
    crate::backend::note_observed::<B>();
    OrbitTail {
        zx: zx.to_carrier(ctx),
        zy: zy.to_carrier(ctx),
        zpx: zpx.to_carrier(ctx),
        zpy: zpy.to_carrier(ctx),
        escaped,
        backend: B::BIT,
    }
}

/// [`run_orbit`], generic over the arithmetic [`RefBackend`] — the loop that is `max_iter × step`
/// in bignum and dominates a deep frame (the blessed bench-matrix baseline puts it at 66% of
/// `deep-interior-1e148`). Returns the running state plus whether the orbit escaped.
///
/// The emitted samples come from [`RefBackend::to_f64_trunc`], **not** a library's own `to_f64`:
/// `crate::to_f64` truncates, and a round-to-nearest conversion would shift samples by ~1 ulp even
/// with a bit-identical bignum state. See `backend.rs` for the full contract.
#[allow(clippy::too_many_arguments)]
fn run_orbit_gen<B: RefBackend>(
    out: &mut Vec<[f32; 4]>,
    mut zx: B,
    mut zy: B,
    mut zpx: B,
    mut zpy: B,
    cx: &B,
    cy: &B,
    formula: u32,
    mut n: u32,
    max_iter: u32,
    ctx: B::Ctx,
) -> (B, B, B, B, bool) {
    let mut escaped = false;
    while n < max_iter {
        let (nzx, nzy) = if formula == formula::PHOENIX {
            phoenix_step_gen(&zx, &zy, &zpx, &zpy, cx, cy, ctx)
        } else {
            step_gen(&zx, &zy, cx, cy, formula, ctx)
        };
        if formula == formula::PHOENIX {
            // Shift z_prev ← z (before z ← z'); std::mem::replace avoids a bignum clone.
            zpx = std::mem::replace(&mut zx, nzx);
            zpy = std::mem::replace(&mut zy, nzy);
        } else {
            zx = nzx;
            zy = nzy;
        }
        let xv = zx.to_f64_trunc();
        let yv = zy.to_f64_trunc();
        out.push(pack_sample(xv, yv));
        n += 1;
        if xv * xv + yv * yv > 1.0e12 {
            escaped = true;
            break;
        }
    }
    (zx, zy, zpx, zpy, escaped)
}

/// As [`reference_orbit`], but also returns the full-precision [`OrbitTail`] so the orbit can be
/// resumed/extended later (see [`extend_reference_orbit`]).
pub fn reference_orbit_t(
    z0x: &BigFloat,
    z0y: &BigFloat,
    cx: &BigFloat,
    cy: &BigFloat,
    formula: u32,
    max_iter: u32,
    p: usize,
) -> (Vec<[f32; 4]>, u32, OrbitTail) {
    reference_orbit_t_in(crate::backend::selected(), z0x, z0y, cx, cy, formula, max_iter, p)
}

/// As [`reference_orbit_t`], but in an **explicitly named** backend rather than the session's
/// selection.
///
/// The selection is a process-wide one-shot — right for an application, useless for the two things
/// that most need to compare backends: a bit-identity test and a cost A/B, both of which must run
/// both arithmetics in ONE process to be free of build and machine confounds. Those are exactly
/// the measurements a second backend has to justify itself with, so the capability is part of the
/// API rather than a test-only back door.
#[allow(clippy::too_many_arguments)]
pub fn reference_orbit_t_in(
    backend: crate::BackendChoice,
    z0x: &BigFloat,
    z0y: &BigFloat,
    cx: &BigFloat,
    cy: &BigFloat,
    formula: u32,
    max_iter: u32,
    p: usize,
) -> (Vec<[f32; 4]>, u32, OrbitTail) {
    let bit = backend.bit();
    let mut out = Vec::with_capacity(max_iter as usize + 1);
    let zx = z0x.clone();
    let zy = z0y.clone();
    // Previous iterate, for Phoenix's two-term recurrence (unused by other formulas). Starts at 0.
    let zpx = BigFloat::from_f64(0.0, p);
    let zpy = BigFloat::from_f64(0.0, p);
    let (xh, xl) = split_df64(to_f64(&zx));
    let (yh, yl) = split_df64(to_f64(&zy));
    out.push([xh, yh, xl, yl]); // Z_0
    let tail = run_orbit(&mut out, bit, zx, zy, zpx, zpy, cx, cy, formula, 0, max_iter, p);
    let len = out.len() as u32;
    (out, len, tail)
}

/// Extend a previously-built (truncated) orbit to a larger `max_iter` **without recomputing the
/// shared prefix**. `prefix` = the cached `Z_0..Z_k` samples; `tail` = the full-precision running
/// state at `Z_k` from [`reference_orbit_t`]. **Requires the identical `(cx, cy, formula, p)`** the
/// prefix was built with — then the result is byte-identical to a fresh `reference_orbit` to
/// `max_iter`. If the cached orbit already escaped (a complete reference) or is already long enough,
/// it is returned unchanged.
pub fn extend_reference_orbit(
    prefix: &[[f32; 4]],
    tail: &OrbitTail,
    cx: &BigFloat,
    cy: &BigFloat,
    formula: u32,
    max_iter: u32,
    p: usize,
) -> (Vec<[f32; 4]>, u32, OrbitTail) {
    let n = prefix.len().saturating_sub(1) as u32; // last cached sample is Z_n
    if tail.escaped || prefix.is_empty() || n >= max_iter {
        return (prefix.to_vec(), prefix.len() as u32, tail.clone());
    }
    let mut out = Vec::with_capacity(max_iter as usize + 1);
    out.extend_from_slice(prefix);
    // ⚠`tail.backend`, NOT the current selection. This function promises a result byte-identical
    // to a fresh build, and the only way to keep that across a mid-session backend switch is to
    // finish the orbit in the arithmetic that started it.
    let extended = dispatch_orbit(
        tail.backend,
        &mut out,
        tail.zx.clone(),
        tail.zy.clone(),
        tail.zpx.clone(),
        tail.zpy.clone(),
        cx,
        cy,
        formula,
        n,
        max_iter,
        p,
    );
    let Some(new_tail) = extended else {
        // The prefix was built by a backend this binary does not have. Finishing it in a different
        // arithmetic would break the byte-identity contract silently, so decline instead: the
        // caller sees a short orbit, which is an outcome it already handles (escaped / long
        // enough), and can rebuild from scratch.
        return (prefix.to_vec(), prefix.len() as u32, tail.clone());
    };
    let len = out.len() as u32;
    (out, len, new_tail)
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
    // Cost-bounded: see SA_COST_BUDGET. At blessed depths the budget exceeds the natural walk and
    // this line is a no-op; at extreme depth it is the difference between a 258 s and a ~30 s
    // build for the same frame.
    let limit = max_iter.min(orbit_len.saturating_sub(2)).min(sa_step_budget(p));
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
        // `sample_xy`, not lane sums: an extended-range dip sample carries a NaN marker.
        let (zr, zi) = sample_xy(&orbit[n]);
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

/// Quick iteration cap for the FIRST-pass reference-candidate ranking (a point surviving this long
/// is a candidate worth a deeper look; keeps the 5×5 bignum grid scan cheap). The survivors are then
/// deep-ranked to the full render length — see [`best_reference`].
const REF_SCORE_SCAN: u32 = 4096;

/// Cost ceiling for the series-approximation coefficient walk, in **steps × bits²** — the walk's
/// actual cost unit, since each SA step is ~20 full-precision bignum multiplies and a bignum
/// multiply scales with precision².
///
/// ⭐⭐**WHY A BUDGET EXISTS (measured at 2.37e4000×, 13,353 bits, 2026-09-01)**: the SA walk ran
/// 439,915 steps and cost **258.2 s of a 405 s reference build — 64% of the whole build** (orbit
/// 32.8 s, candidate scoring 113.7 s, BLA 0.4 s), to compute a per-pixel iteration skip whose
/// entire GPU saving is a fraction of a **0.58 s** iterate pass. SA's cost grows ~linearly in
/// depth on top of precision², while its value is capped by the frame it feeds; past some depth
/// it is a pure loss, and this is the deterministic bound that stops it.
///
/// ⭐**THE CONSTANT IS PLACED ABOVE EVERY BLESSED FIXTURE'S NATURAL WALK, so blessed outputs are
/// bit-identical BY CONSTRUCTION** — where the budget does not bite, the walk is the walk it
/// always was. The corpus's worst-case bound (walk ≤ its iteration count) is 6.99e12 (row 20,
/// 600,008 iterations at 3,413 bits); `sa_budget_clears_every_blessed_fixture` pins every deep
/// row against this constant, so shrinking it below a blessed fixture's need is a red test, not
/// a silent re-bless. ⚠Where it DOES bite (past ~e1300 at corpus-scale iteration counts), the
/// capped walk still emits a VALID skip — validity is proven per-step in the loop, so stopping
/// early only shortens the skip, never corrupts it — but pixels drift in low bits against an
/// uncapped render, which is why the line sits above the fixtures rather than at the knee.
const SA_COST_BUDGET: u64 = 15_000_000_000_000;

/// The SA walk length [`SA_COST_BUDGET`] buys at working precision `p`.
fn sa_step_budget(p: usize) -> u32 {
    let bits2 = (p as u64).saturating_mul(p as u64).max(1);
    (SA_COST_BUDGET / bits2).min(u32::MAX as u64) as u32
}

/// Deep-scan at most this many survivors in phase 2 (bounds a pathological build; the survivors are
/// in candidate order, so these are the most central — the likeliest good references).
const REF_DEEP_MAX: usize = 16;

/// Extra scoring precision for the CLIFF RESCUE passes in [`best_reference_diag`]. Matches the
/// app's `REF_PREC_HEADROOM` (the orbit is built 128 bits above the request), so a rescued score
/// describes exactly the orbit that will be built — scoring BELOW the build precision is the
/// blindness this exists to cure.
const REF_RESCUE_EXTRA_BITS: usize = 128;

/// Why/how a pick was made — the observability half of the reference-lifecycle redesign
/// (`design/reference-lifecycle.md` L0/L1). Everything here is deterministic.
#[derive(Clone, Copy, Debug)]
pub struct RefPickDiag {
    /// Precision the WINNING selection was scored at (`p`, or `p + 128` after a rescue rescan).
    pub scoring_prec: usize,
    /// Phase-1 survivor count of the winning pass.
    pub survivors: usize,
    /// The winner's deep score at `scoring_prec` (≥ `max_iter` means it survives the render).
    pub winner_len: u32,
    /// `Some("rescan")` — no phase-1 survivor at `p`, whole selection redone at `p + 128`;
    /// `Some("centre")` — the `p`-winner escaped early and the centre, rescored at `p + 128`,
    /// survives the full render. `None` — the plain pick stood.
    pub rescued: Option<&'static str>,
    /// The winning pass had no survivor either: the pick is the longest ESCAPER (deep exterior).
    pub fallback_escaper: bool,
}

/// One selection pass at a fixed precision: phase 1 (cheap rank to `quick`) + phase 2 (deep-rank
/// survivors). Factored out of [`best_reference`] so the cliff rescue can rerun it at higher
/// precision; the selection semantics inside are byte-identical to the original.
enum PickPass {
    /// A phase-1 survivor won; `deep_len` is its phase-2 score (early-break semantics preserved).
    Winner { point: [BigFloat; 2], deep_len: u32, survivors: usize },
    /// Every candidate escaped within `quick` — the longest escaper and its length.
    NoSurvivor { point: [BigFloat; 2], esc_len: u32 },
}

/// Pick a reference within the view with the longest orbit (prefers an interior point). For a Julia
/// view, candidates are `Z₀` values with the fixed `julia_c`; otherwise they are `c` values with
/// `Z₀ = 0`. Returns the chosen bignum point.
///
/// ⭐**Cliff rescue** (2026-08-09, `design/reference-lifecycle.md` L1): orbit arithmetic at `p`
/// bits is only true for a precision-dependent number of iterations before rounding error, amplified
/// by a repelling orbit, makes it spuriously escape — measured at the three-spar Misiurewicz centre
/// as ESCAPE CLIFFS: 128 bits → 570, 160 → 84,941, 207 → 570,711, 286+ → survives (test
/// `escape_length_vs_precision`). Below the cliff this function used to go BLIND: every candidate
/// "escaped" inside the quick scan, phase 1 had no survivor, and the longest-escaper fallback
/// institutionalised a short-escaper pick (the grand tour's 626-sample reference — `bla_skip=0`,
/// ~90× frame cost, the 2:58 device loss). Two rescues, both deterministic and both scoped to the
/// already-suspicious outcomes so healthy picks are byte-identical to the old selection:
/// - no phase-1 survivor at `p` → redo the whole selection at `p + 128` (the orbit-build
///   precision, so the rescued score describes the orbit that will actually be built);
/// - the winner escapes before `max_iter` → rescore the CENTRE alone at `p + 128`; take the
///   centre only if it then survives the full render (the unambiguous case).
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
    best_reference_diag(center, span, formula, julia, julia_c, max_iter, p).0
}

/// [`best_reference`] plus the pick diagnostics — see [`RefPickDiag`].
#[allow(clippy::too_many_arguments)]
pub fn best_reference_diag(
    center: &[BigFloat; 2],
    span: [FloatExp; 2],
    formula: u32,
    julia: bool,
    julia_c: [f64; 2],
    max_iter: u32,
    p: usize,
) -> ([BigFloat; 2], RefPickDiag) {
    match pick_pass(center, span, formula, julia, julia_c, max_iter, p) {
        PickPass::Winner { point, deep_len, survivors } if deep_len >= max_iter => (
            point,
            RefPickDiag {
                scoring_prec: p,
                survivors,
                winner_len: deep_len,
                rescued: None,
                fallback_escaper: false,
            },
        ),
        PickPass::Winner { point, deep_len, survivors } => {
            // The winner escapes before the render budget — either a genuine boundary/exterior
            // view (every point escapes) or a precision cliff between `quick` and `max_iter`.
            // Rescoring the centre at the BUILD precision separates them: only a cliff un-blinds.
            let p2 = p + REF_RESCUE_EXTRA_BITS;
            let jcx = bf(julia_c[0], p2);
            let jcy = bf(julia_c[1], p2);
            let zero = bf(0.0, p2);
            let centre_len = if julia {
                orbit_length_bf(&center[0], &center[1], &jcx, &jcy, formula, max_iter, p2)
            } else {
                orbit_length_bf(&zero, &zero, &center[0], &center[1], formula, max_iter, p2)
            };
            if centre_len >= max_iter {
                (
                    [center[0].clone(), center[1].clone()],
                    RefPickDiag {
                        scoring_prec: p2,
                        survivors,
                        winner_len: centre_len,
                        rescued: Some("centre"),
                        fallback_escaper: false,
                    },
                )
            } else {
                (
                    point,
                    RefPickDiag {
                        scoring_prec: p,
                        survivors,
                        winner_len: deep_len,
                        rescued: None,
                        fallback_escaper: false,
                    },
                )
            }
        }
        PickPass::NoSurvivor { .. } => {
            // Not one candidate outlived the quick scan at `p`. A genuinely deep-exterior view
            // looks like this — but so does the cliff, and below the cliff every score is
            // fiction. Redo the WHOLE selection at the build precision; a still-empty pass is
            // then a real exterior view and the longest escaper (at truthful scores) stands.
            let p2 = p + REF_RESCUE_EXTRA_BITS;
            match pick_pass(center, span, formula, julia, julia_c, max_iter, p2) {
                PickPass::Winner { point, deep_len, survivors } => (
                    point,
                    RefPickDiag {
                        scoring_prec: p2,
                        survivors,
                        winner_len: deep_len,
                        rescued: Some("rescan"),
                        fallback_escaper: false,
                    },
                ),
                PickPass::NoSurvivor { point, esc_len } => (
                    point,
                    RefPickDiag {
                        scoring_prec: p2,
                        survivors: 0,
                        winner_len: esc_len,
                        rescued: Some("rescan"),
                        fallback_escaper: true,
                    },
                ),
            }
        }
    }
}

/// The original selection body, at one precision. See [`best_reference`] for the semantics.
#[allow(clippy::too_many_arguments)]
fn pick_pass(
    center: &[BigFloat; 2],
    span: [FloatExp; 2],
    formula: u32,
    julia: bool,
    julia_c: [f64; 2],
    max_iter: u32,
    p: usize,
) -> PickPass {
    // Score candidates by orbit length in **bignum** (f64 coords collapse to the same value at deep
    // zoom, which broke reference selection on cold jumps). TWO PHASES: rank cheaply to `quick`, then
    // DEEP-rank the survivors to the full render length and take the longest-surviving. A reference
    // that survives the whole render needs NO rebasing, which is what keeps a continuous deep zoom
    // smooth; ranking only to `quick` couldn't tell a filament point (survives to `max_iter`) from a
    // near-exterior point that escapes just past `quick`, so at near-filament spots it picked an
    // escaping reference → heavy rebasing → the deep-zoom "jump on zoom". Deep-ranking only survivors
    // (centre first, early-break once one survives the full render) keeps the common boundary case
    // cheap; a genuine deep-exterior dive (no survivor) falls back to the longest-escaping point.
    let quick = max_iter.min(REF_SCORE_SCAN);
    let jcx = bf(julia_c[0], p);
    let jcy = bf(julia_c[1], p);
    let zero = bf(0.0, p);
    let score = |zx: &BigFloat, zy: &BigFloat, cap: u32| -> u32 {
        if julia {
            orbit_length_bf(zx, zy, &jcx, &jcy, formula, cap, p)
        } else {
            orbit_length_bf(&zero, &zero, zx, zy, formula, cap, p)
        }
    };
    // Candidate points: the centre first, then a 5×5 grid at several span fractions (fine→coarse), so
    // the search reliably samples the central detail even when the boundary is a thin filament (a
    // single coarse ±0.5-span grid falls into the gaps between filaments at deep zoom).
    const N: usize = 5;
    const SCALES: [f64; 4] = [0.04, 0.12, 0.28, 0.5];
    let mut cands: Vec<[BigFloat; 2]> = Vec::with_capacity(1 + SCALES.len() * N * N);
    cands.push([center[0].clone(), center[1].clone()]);
    for &sc in &SCALES {
        for j in 0..N {
            for i in 0..N {
                let fx = (i as f64 / (N as f64 - 1.0) - 0.5) * 2.0 * sc;
                let fy = (j as f64 / (N as f64 - 1.0) - 0.5) * 2.0 * sc;
                // Offsets via the extended-range span so the grid doesn't collapse to the centre
                // past ~1e308× (where an f64 span would underflow to 0).
                let px = center[0].add(&span[0].mul_f64(fx).to_bf(p), p, RM);
                let py = center[1].add(&span[1].mul_f64(fy).to_bf(p), p, RM);
                cands.push([px, py]);
            }
        }
    }
    // Phase 1 — cheap rank to `quick`, scored across ALL CORES (each candidate's bignum orbit is
    // independent; the selection below reads the in-candidate-order score array, so the chosen
    // reference is IDENTICAL to the old sequential scan — threading changes wall-clock only). This
    // scan is the dominant cold-recompute cost at depth (~0.8 s at 1e400×, ~7.6 s at 1e1216×
    // sequential — the live-dive "monocolor" stall), and it parallelizes near-linearly.
    let idxs: Vec<usize> = (0..cands.len()).collect();
    let scores = par_orbit_scores(&cands, &idxs, quick, julia, &jcx, &jcy, formula, p);
    let mut survivors: Vec<usize> = Vec::new();
    let (mut esc_i, mut esc_len) = (0usize, 0u32);
    for (idx, &len) in scores.iter().enumerate() {
        if len >= quick {
            survivors.push(idx);
        } else if len > esc_len {
            esc_len = len;
            esc_i = idx;
        }
    }
    if survivors.is_empty() {
        // Deep exterior — or a precision cliff; the caller's rescue pass tells them apart.
        return PickPass::NoSurvivor { point: cands[esc_i].clone(), esc_len };
    }
    // Phase 2 — deep-rank survivors to the full render length; take the longest-surviving. The centre
    // (cands[0], usually best on the boundary) is scanned first — alone, so the common boundary case
    // stays a single deep orbit — and only if it does NOT survive the whole render do the remaining
    // survivors get a parallel deep scan. The selection loop then replicates the old sequential
    // early-break semantics over the in-order results, so the pick is identical.
    // `dl >= max_iter` ⇒ a reference that survives the whole render (no rebasing at all).
    let first = survivors[0];
    let dl0 = score(&cands[first][0], &cands[first][1], max_iter);
    if dl0 >= max_iter {
        return PickPass::Winner {
            point: cands[first].clone(),
            deep_len: dl0,
            survivors: survivors.len(),
        };
    }
    let rest: Vec<usize> = survivors.iter().skip(1).take(REF_DEEP_MAX - 1).copied().collect();
    let deep = par_orbit_scores(&cands, &rest, max_iter, julia, &jcx, &jcy, formula, p);
    let (mut best_i, mut best_len) = (first, dl0);
    for (&idx, &dl) in rest.iter().zip(deep.iter()) {
        if dl > best_len {
            best_len = dl;
            best_i = idx;
            if dl >= max_iter {
                break;
            }
        }
    }
    PickPass::Winner {
        point: cands[best_i].clone(),
        deep_len: best_len,
        survivors: survivors.len(),
    }
}

/// Score `idxs`-selected candidates' orbit lengths (up to `cap`) in parallel across the machine's
/// cores, returning the scores in `idxs` order. Each candidate is an independent bignum orbit, and
/// bignum arithmetic is deterministic, so the scores — and any selection made from them — are
/// byte-identical to a sequential scan; only the wall-clock changes. Threads get their own clones
/// of the shared scoring constants, so this needs `BigFloat: Send` only.
#[allow(clippy::too_many_arguments)]
fn par_orbit_scores(
    cands: &[[BigFloat; 2]],
    idxs: &[usize],
    cap: u32,
    julia: bool,
    jcx: &BigFloat,
    jcy: &BigFloat,
    formula: u32,
    p: usize,
) -> Vec<u32> {
    let score_one = |c: &[BigFloat; 2], jcx: &BigFloat, jcy: &BigFloat, zero: &BigFloat| -> u32 {
        if julia {
            orbit_length_bf(&c[0], &c[1], jcx, jcy, formula, cap, p)
        } else {
            orbit_length_bf(zero, zero, &c[0], &c[1], formula, cap, p)
        }
    };
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get()).min(idxs.len());
    if threads <= 1 {
        let zero = bf(0.0, p);
        return idxs.iter().map(|&i| score_one(&cands[i], jcx, jcy, &zero)).collect();
    }
    let mut out = vec![0u32; idxs.len()];
    let chunk = idxs.len().div_ceil(threads);
    std::thread::scope(|s| {
        for (idx_chunk, out_chunk) in idxs.chunks(chunk).zip(out.chunks_mut(chunk)) {
            s.spawn(|| {
                let (jcx, jcy, zero) = (jcx.clone(), jcy.clone(), bf(0.0, p));
                for (&i, o) in idx_chunk.iter().zip(out_chunk.iter_mut()) {
                    *o = score_one(&cands[i], &jcx, &jcy, &zero);
                }
            });
        }
    });
    out
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

/// The linear scale and orientation of the minibrot ("atom") at a nucleus.
pub struct AtomSize {
    /// `log₂` of the atom's linear size — approximately the WIDTH of the embedded copy in the
    /// complex plane. Held as a log because a deep minibrot's size (`~2^-3322` at 1e1000×, and
    /// far smaller past that) has no `f64` representation at all.
    pub log2_size: f64,
    /// Orientation of the embedded copy in radians: the argument of the complex size estimate.
    /// `0` means the copy sits the same way up as the whole set; `±π` means inverted.
    pub orientation: f64,
}

/// Complex reciprocal `1/(x+iy)`; `None` on a zero (or non-finite) argument.
fn cinv_bf(x: &BigFloat, y: &BigFloat, p: usize) -> Option<(BigFloat, BigFloat)> {
    let d = x.mul(x, p, RM).add(&y.mul(y, p, RM), p, RM);
    if d.is_zero() || d.is_nan() || d.is_inf() {
        return None;
    }
    Some((x.div(&d, p, RM), neg_bf(y, p).div(&d, p, RM)))
}

/// Size and orientation of the minibrot whose nucleus is `c` and whose period is `period`
/// (Munafo's size estimate).
///
/// Iterating the critical orbit and accumulating `Λ = ∏ 2·z_i` and `B = 1 + Σ 1/Λ_i` gives
/// `size = 1/(B·Λ²)` — a complex quantity whose magnitude is the copy's linear scale and whose
/// argument is its orientation. Two exact anchors fix the convention: the whole set (period 1)
/// returns 1, matching the main cardioid's width from −0.75 to 0.25; and the period-2 disk
/// returns 0.5, matching its span from −1.25 to −0.75.
///
/// **Quadratic only.** The estimate is derived from `z² + c`; the Multibrot families need a
/// different derivative recurrence, so they return `None` rather than a plausible wrong number.
/// Cost is one `period`-step arbitrary-precision pass — the same order as the Newton solve that
/// produced the nucleus.
pub fn nucleus_size(
    cx: &BigFloat,
    cy: &BigFloat,
    period: u32,
    formula: u32,
    p: usize,
) -> Option<AtomSize> {
    if formula != formula::MANDELBROT || period == 0 {
        return None;
    }
    let (mut zx, mut zy) = (bf(0.0, p), bf(0.0, p));
    let (mut lx, mut ly) = (bf(1.0, p), bf(0.0, p)); // Λ — running derivative product
    let (mut bx, mut by) = (bf(1.0, p), bf(0.0, p)); // B — running 1 + Σ 1/Λ
    for _ in 1..period {
        let (nx, ny) = step_bf(&zx, &zy, cx, cy, formula, p);
        zx = nx;
        zy = ny;
        let (mx, my) = cmul_bf(&zx, &zy, &lx, &ly, p);
        lx = mx.add(&mx, p, RM); // Λ ← 2·z·Λ
        ly = my.add(&my, p, RM);
        let (ix, iy) = cinv_bf(&lx, &ly, p)?;
        bx = bx.add(&ix, p, RM);
        by = by.add(&iy, p, RM);
    }
    // size = 1/(B·Λ²) — taken as a log so the deep case can't underflow.
    let (l2x, l2y) = cmul_bf(&lx, &ly, &lx, &ly, p);
    let (dx, dy) = cmul_bf(&bx, &by, &l2x, &l2y, p);
    let denom2 = dx.mul(&dx, p, RM).add(&dy.mul(&dy, p, RM), p, RM);
    if denom2.is_zero() || denom2.is_nan() || denom2.is_inf() {
        return None;
    }
    let log2_size = -0.5 * log2_abs(&denom2);
    if !log2_size.is_finite() {
        return None;
    }
    Some(AtomSize { log2_size, orientation: -arg_bf(&dx, &dy, p) })
}

/// Re-solve a **known** nucleus to `p` bits, Newton-iterating from an already-close seed.
///
/// [`find_nucleus`] stops once the Newton step falls below a tolerance derived from the *current
/// view span*, so its answer is only as accurate as the view that asked for it — about 1e-12 at
/// a 1e3× view. That is useless for a zoom onto the minibrot itself, whose own span may be 1e-45
/// or 1e-1000: the center would be wrong by many view widths. This continues the same Newton
/// solve at the working precision the destination needs, which costs only a handful of steps
/// because Newton doubles the correct digits each time.
///
/// Convergence is measured with [`log2_abs`], not `to_f64`, so the step magnitude stays
/// meaningful once it drops below `f64`'s smallest normal.
pub fn refine_nucleus(
    cx: &BigFloat,
    cy: &BigFloat,
    period: u32,
    formula: u32,
    p: usize,
) -> Option<(BigFloat, BigFloat)> {
    formula_power(formula)?; // reject the non-holomorphic families up front
    if period == 0 {
        return None;
    }
    let mut cx = cx.clone();
    let mut cy = cy.clone();
    let mut prev_l2 = f64::INFINITY;
    for _ in 0..32 {
        let (stepx, stepy) = nucleus_newton_step(&cx, &cy, period, formula, p)?;
        cx = cx.sub(&stepx, p, RM);
        cy = cy.sub(&stepy, p, RM);
        // |step| as a log — the whole point of this routine is steps far below f64's range.
        let step_l2 =
            0.5 * log2_abs(&stepx.mul(&stepx, p, RM).add(&stepy.mul(&stepy, p, RM), p, RM));
        if step_l2 < -(p as f64) + 8.0 {
            break; // converged to the working precision
        }
        if step_l2 >= prev_l2 {
            break; // stalled (or diverging) — take what we have rather than churn
        }
        prev_l2 = step_l2;
    }
    Some((cx, cy))
}

/// One Newton step of the nucleus equation `Z_period(c) = 0`: returns `Z_period / (dZ_period/dc)`,
/// the correction to subtract from `c`.
fn nucleus_newton_step(
    cx: &BigFloat,
    cy: &BigFloat,
    period: u32,
    formula: u32,
    p: usize,
) -> Option<(BigFloat, BigFloat)> {
    let k = formula_power(formula)?;
    let one = bf(1.0, p);
    let kf = bf(k as f64, p);
    let mut zx = bf(0.0, p);
    let mut zy = bf(0.0, p);
    let mut dx = bf(0.0, p);
    let mut dy = bf(0.0, p);
    for _ in 0..period {
        // D_{n+1} = k·Z_n^{k-1}·D_n + 1 ;  Z_{n+1} = Z_n^k + c
        let (zk1x, zk1y) =
            if k == 2 { (zx.clone(), zy.clone()) } else { cpow_bf(&zx, &zy, k - 1, p) };
        let (mzx, mzy) = cmul_bf(&zk1x, &zk1y, &dx, &dy, p);
        let ndx = mzx.mul(&kf, p, RM).add(&one, p, RM);
        let ndy = mzy.mul(&kf, p, RM);
        let (nzx, nzy) = step_bf(&zx, &zy, cx, cy, formula, p);
        zx = nzx;
        zy = nzy;
        dx = ndx;
        dy = ndy;
    }
    let denom = dx.mul(&dx, p, RM).add(&dy.mul(&dy, p, RM), p, RM);
    if denom.is_zero() || denom.is_nan() || denom.is_inf() {
        return None;
    }
    let (numx, numy) = cmul_bf(&zx, &zy, &dx, &neg_bf(&dy, p), p);
    Some((numx.div(&denom, p, RM), numy.div(&denom, p, RM)))
}

/// `log₂` of how far a claimed nucleus sits from the true one: the magnitude of the residual
/// Newton step `Z_period / (dZ_period/dc)`, which to first order *is* the center error.
///
/// This is the check a Newton-Raphson zoom rests on. The destination view spans roughly the
/// atom's own size, so the center must be accurate to far better than that or the jump lands on
/// empty space. Comparing this against [`AtomSize::log2_size`] is a self-validating test — it
/// needs no reference coordinate, just the defining property of a nucleus. `-∞` = exact.
pub fn nucleus_residual_log2(
    cx: &BigFloat,
    cy: &BigFloat,
    period: u32,
    formula: u32,
    p: usize,
) -> Option<f64> {
    let (sx, sy) = nucleus_newton_step(cx, cy, period, formula, p)?;
    Some(0.5 * log2_abs(&sx.mul(&sx, p, RM).add(&sy.mul(&sy, p, RM), p, RM)))
}

/// The exact center of a Misiurewicz (pre-periodic) point found near a view center.
#[derive(Debug, PartialEq)]
pub struct Misiurewicz {
    pub preperiod: u32,
    pub period: u32,
    pub cx: BigFloat,
    pub cy: BigFloat,
}

/// The multiplier `λ = (f^p)′` of the repelling cycle a Misiurewicz point lands on.
pub struct Multiplier {
    /// `log₂|λ|` — the **zoom period**: the view self-repeats every `log₂|λ|` octaves of zoom, so
    /// a dive centered here is periodic at that scale. A log because at high period `|λ|` is
    /// astronomically large.
    pub log2_abs: f64,
    /// `arg λ` in radians — the **spiral twist**: how far the structure rotates over one zoom
    /// period. Zero means the repetition is straight (self-similar without spiralling).
    pub arg: f64,
}

/// Multiplier of the repelling cycle at the Misiurewicz point `c` with pre-period `k` and period
/// `p`: iterate the critical orbit `k` steps to reach the cycle, then accumulate the derivative
/// `∏ k·z^(k−1)` around the `p` cycle points.
///
/// Two exact cases pin it. At `c = −2` (the antenna tip, k=2 p=1) the orbit reaches the fixed
/// point 2 and `λ = 4` exactly and real — which is why the tip repeats without spiralling. At
/// `c = i` (k=2 p=2) the cycle is `{−1+i, −i}` and `λ = 4(1+i)`, so `|λ| = 4√2` and the structure
/// twists 45° per zoom period.
pub fn misiurewicz_multiplier(
    cx: &BigFloat,
    cy: &BigFloat,
    preperiod: u32,
    period: u32,
    formula: u32,
    p: usize,
) -> Option<Multiplier> {
    let k = formula_power(formula)?;
    if preperiod == 0 || period == 0 {
        return None;
    }
    let kf = bf(k as f64, p);
    // Run the critical orbit up to the entry point of the cycle.
    let (mut zx, mut zy) = (bf(0.0, p), bf(0.0, p));
    for _ in 0..preperiod {
        let (nx, ny) = step_bf(&zx, &zy, cx, cy, formula, p);
        zx = nx;
        zy = ny;
    }
    // λ = ∏ over the cycle of d/dz (z^k + c) = k·z^(k−1).
    let (mut lx, mut ly) = (bf(1.0, p), bf(0.0, p));
    for _ in 0..period {
        let (dx, dy) =
            if k == 2 { (zx.clone(), zy.clone()) } else { cpow_bf(&zx, &zy, k - 1, p) };
        let (tx, ty) = cmul_bf(&lx, &ly, &dx, &dy, p);
        lx = tx.mul(&kf, p, RM);
        ly = ty.mul(&kf, p, RM);
        let (nx, ny) = step_bf(&zx, &zy, cx, cy, formula, p);
        zx = nx;
        zy = ny;
    }
    let m2 = lx.mul(&lx, p, RM).add(&ly.mul(&ly, p, RM), p, RM);
    if m2.is_zero() || m2.is_nan() || m2.is_inf() {
        return None;
    }
    let log2_abs = 0.5 * log2_abs(&m2);
    log2_abs.is_finite().then(|| Multiplier { log2_abs, arg: arg_bf(&lx, &ly, p) })
}

/// Newton-solve the Misiurewicz point of pre-period `preperiod` (k) and period `period` (p) nearest
/// the `center` seed: the `c` whose critical orbit is pre-periodic with `Z_{k+p}(c) = Z_k(c)`.
/// Mirrors [`find_nucleus`] but on the pre-periodicity equation `F(c) = Z_{k+p} − Z_k`, with
/// `F'(c) = D_{k+p} − D_k` (`D = dZ/dc`). Seeded from where you're looking, so it snaps onto the
/// branch/spiral center you're near. `None` if it doesn't converge to a nearby point that is
/// genuinely pre-periodic (a `(k,p)` mismatch or runaway).
/// Detect a Misiurewicz point's `(preperiod, period)` from the critical orbit at `(cx, cy)`.
///
/// A Misiurewicz point is strictly PRE-periodic: the critical orbit runs for `k` steps and then
/// lands exactly on a repelling `p`-cycle. So the pair to find is the `(m, n-m)` minimising
/// `|z_n - z_m|` — at the true point that separation is zero, and at a view centred near one it is
/// about the width of the view.
///
/// This is the pre-periodic counterpart of [`detect_period`], and it is what lets the Misiurewicz
/// solver work the way the minibrot one already does: without it a caller has to KNOW the two
/// numbers, which in practice means nobody uses it and dives to a spiral centre by hand.
///
/// ⚠**The scan cannot be done in `f64`.** The orbit values are order 1, so an `f64` copy resolves
/// differences no finer than ~1e-16, while the separation that identifies the pair is the width of
/// the view — 6.6e-39 at the 1.6e39x location this was written for. Every candidate would read as
/// exactly zero. `f64` is therefore used only as a cheap PRE-FILTER (a true near-return collapses
/// to zero there, so it always survives), and the ranking is done in full precision on the handful
/// that pass.
///
/// Returns `None` if the orbit escapes (not pre-periodic) or nothing repeats within the bounds.
pub fn detect_misiurewicz(
    cx: &BigFloat,
    cy: &BigFloat,
    formula: u32,
    max_iter: u32,
    max_period: u32,
    p: usize,
) -> Option<(u32, u32)> {
    detect_misiurewicz_at_scale(cx, cy, formula, max_iter, max_period, p, None)
}

/// [`detect_misiurewicz`], but preferring the pair whose feature is the size of the VIEW.
///
/// ⭐⭐**The closest near-return is not the most useful one.** Ranking purely by separation picks
/// whichever pair the orbit happens to match best, and at depth that is routinely a feature far
/// COARSER than what is on screen: measured at a 2.77e89× dendrite, the winner was (437,3), whose
/// point sits 4.0e-77 away — 3.7e12 view-widths, twelve decades out. The solve then finds it
/// perfectly and the caller has to reject it for being nowhere near the view.
///
/// `target_span_log2` (log2 of the view's complex width) selects instead by SCALE. The
/// neighbourhood of a pre-period-`k` point is about `1/|D_k|` across, where `D = dz/dc` is carried
/// alongside the orbit, so the pair that governs the visible structure is the one with
/// `|D_k| ≈ 1/span`.
///
/// ⚠**A LOG, not the width itself.** The linear span underflows `f64` to zero past ~1e308×, which
/// is well inside this app's range — passing it directly would silently disable the scale test at
/// exactly the depths that need it most.
///
/// ⚠`|D|` is tracked as `log2` in `f64`, accumulated as `Σ log2|2·z_i|` — the product that
/// dominates `D_{n+1} = 2·z_n·D_n + 1`. The `+1` is dropped, which is wrong while `|D|` is O(1)
/// and irrelevant once it is astronomically large, and only the ORDER OF MAGNITUDE is used here.
/// Tracking `D` itself would overflow `f64` before 1e309× and cost a bignum multiply per step.
pub fn detect_misiurewicz_at_scale(
    cx: &BigFloat,
    cy: &BigFloat,
    formula: u32,
    max_iter: u32,
    max_period: u32,
    p: usize,
    target_span_log2: Option<f64>,
) -> Option<(u32, u32)> {
    formula_power(formula)?;
    let max_iter = max_iter.clamp(8, 20_000) as usize;
    let max_period = max_period.clamp(1, 4_096) as usize;

    // The orbit, kept in full precision, with an f64 shadow for the pre-filter.
    let mut zs: Vec<(BigFloat, BigFloat)> = Vec::with_capacity(max_iter);
    let mut sh: Vec<(f64, f64)> = Vec::with_capacity(max_iter);
    // log2|dz/dc| alongside, for the scale test. See the doc comment for why it is a log sum.
    let mut l2d: Vec<f64> = Vec::with_capacity(max_iter);
    let mut acc = 0.0f64;
    let mut zx = bf(0.0, p);
    let mut zy = bf(0.0, p);
    for _ in 0..max_iter {
        // The growth factor uses the PRE-step z, matching D_{n+1} = k·z_n^{k-1}·D_n + 1.
        let za = (crate::to_f64(&zx).powi(2) + crate::to_f64(&zy).powi(2)).sqrt();
        if za > 0.0 {
            acc += (2.0 * za).log2();
        }
        let (nx, ny) = step_bf(&zx, &zy, cx, cy, formula, p);
        zx = nx;
        zy = ny;
        if mag2_bf(&zx, &zy) > 16.0 {
            break; // escaped: the orbit is not pre-periodic
        }
        sh.push((crate::to_f64(&zx), crate::to_f64(&zy)));
        zs.push((zx.clone(), zy.clone()));
        l2d.push(acc.max(0.0));
    }
    if zs.len() < 4 {
        return None;
    }

    // Pre-filter in f64, to keep the bignum ranking below off pairs whose point cannot be near the
    // view. ⭐⭐**The test is on the distance to the ROOT, not on the separation** — Newton's first
    // step, |z_n - z_m| / |F'|, with |D_n| for |F'| (the larger of the two derivatives dominates).
    //
    // ⚠⚠**A CUT ON THE RAW SEPARATION CANNOT WORK AT BOTH ENDS, and both ends were measured
    // failing.** The old fixed 1e-6 discarded everything at a 283,353x spiral, where the real
    // point's near-return is 2.9e-5 (fixed by scaling it to the view). It then failed the OTHER
    // WAY at 2.37e40x: the pair identifying the point at the view centre, (3999,4000), separates
    // by 2.6e-4 — 264x ABOVE the cut — so it was thrown out and the detector answered a pair whose
    // point is 15-23 view-widths away (user, 2026-08-31).
    //
    // ⭐The reason the two ends disagree: a deep point's signature is NOT a small separation. It is
    // one small *relative to the derivative*, and at 1e40x |D_n| is around 1e37, so a genuinely
    // near root sits behind a perfectly ordinary-looking separation. Dividing by |D_n| is what
    // makes one threshold serve every depth.
    //
    // ⚠Conservative by construction: a genuinely tiny separation underflows the f64 shadow to noise
    // or to zero, which only makes the estimate SMALLER, so this never discards a real candidate.
    /// How many view-widths from the seed a root may be and still be worth ranking.
    const NEAR_SPANS_LOG2: f64 = 3.0;
    /// Fallback when there is no view to measure against: the historical absolute cut.
    const COARSE_FLOOR: f64 = 1.0e-6;
    let near_l2 = target_span_log2.filter(|s| s.is_finite()).map(|s| s + NEAR_SPANS_LOG2);
    let mut cand: Vec<(usize, usize)> = Vec::new();
    for n in 1..sh.len() {
        let lo = n.saturating_sub(max_period);
        for m in lo..n {
            let (dx, dy) = (sh[n].0 - sh[m].0, sh[n].1 - sh[m].1);
            let keep = match near_l2 {
                // log2(sep) - log2|D_n| < log2(span) + 3, all in logs so nothing underflows.
                Some(t) => 0.5 * (dx * dx + dy * dy).max(f64::MIN_POSITIVE).log2() - l2d[n] < t,
                None => dx * dx + dy * dy < COARSE_FLOOR * COARSE_FLOOR,
            };
            if keep {
                cand.push((m, n));
            }
        }
    }
    if cand.is_empty() {
        return None;
    }

    // Rank the survivors. With a target span, by how closely the pair's own feature scale matches
    // the view; otherwise by separation, which is the historical behaviour.
    let want_l2d = target_span_log2.filter(|s| s.is_finite()).map(|s| -s);
    let mut best: Option<(BigFloat, usize, usize)> = None;
    let mut best_scale = f64::INFINITY;
    for (m, n) in cand {
        let dx = zs[n].0.sub(&zs[m].0, p, RM);
        let dy = zs[n].1.sub(&zs[m].1, p, RM);
        let d = dx.mul(&dx, p, RM).add(&dy.mul(&dy, p, RM), p, RM);
        let better = match want_l2d {
            Some(want) => {
                // Octaves between this pair's feature size and the view's. Ties (and they are
                // common, since a period's harmonics share a preperiod) fall back to separation.
                let err = (l2d.get(m).copied().unwrap_or(0.0) - want).abs();
                if (err - best_scale).abs() < 0.5 {
                    best.as_ref().is_none_or(|(bd, _, _)| d.cmp(bd).is_some_and(|o| o < 0))
                } else {
                    err < best_scale
                }
            }
            // astro-float's `cmp` yields a SIGN, not an Ordering.
            None => best.as_ref().is_none_or(|(bd, _, _)| d.cmp(bd).is_some_and(|o| o < 0)),
        };
        if better {
            if let Some(want) = want_l2d {
                best_scale = (l2d.get(m).copied().unwrap_or(0.0) - want).abs();
            }
            best = Some((d, m, n));
        }
    }
    let (bd, bm, bn) = best?;

    // Harmonics: if (k, p) fits, so does (k, 2p), (k, 3p)… and they can score almost as well.
    // Prefer the SMALLEST period whose separation is within a factor of the winner, so the answer
    // is the fundamental cycle rather than a multiple of it.
    let period = bn - bm;
    let slack = bd.mul(&bf(64.0, p), p, RM);
    let mut fundamental = period;
    for q in 1..period {
        if period % q != 0 {
            continue;
        }
        let n2 = bm + q;
        if n2 >= zs.len() {
            continue;
        }
        let dx = zs[n2].0.sub(&zs[bm].0, p, RM);
        let dy = zs[n2].1.sub(&zs[bm].1, p, RM);
        let d = dx.mul(&dx, p, RM).add(&dy.mul(&dy, p, RM), p, RM);
        if d.cmp(&slack).is_some_and(|o| o < 0) {
            fundamental = q;
            break;
        }
    }
    // `bm` indexes the orbit AFTER one step, so the preperiod is one-based.
    Some((bm as u32 + 1, fundamental as u32))
}

/// `log2|v|`, read off the exponent — to within one octave, and immune to the `f64` underflow
/// that makes every linear tolerance meaningless past ~1e308×.
///
/// ⭐This is the whole trick behind solving deeper than `f64` can express. A Newton step of 1e-58000
/// is a perfectly ordinary number in bignum and is exactly `0.0` once it touches `f64` — so a
/// tolerance test written as `to_f64(&step) < tol` reads `0.0 < 0.0`, is false, and reports a
/// converged solve as "not converged". Comparing exponents has no such floor.
fn log2_abs_bf(v: &BigFloat) -> f64 {
    // ⚠**ZERO IS `Some(0)`, NOT `None`** — measured, not assumed: astro-float stores a value as
    // `0.m × 2^e`, and `exponent()` returns `None` only for NaN/Inf. Reading `None` as "zero" made
    // an exactly-zero distance report as 2^0 = one unit, which at a 2.77e89× view is 2^295
    // view-widths — a perfect solve rejected as TooFar.
    if v.is_zero() {
        return f64::NEG_INFINITY;
    }
    match v.exponent() {
        // `0.m × 2^e` with the mantissa in [0.5, 1), so log2|v| lands in (e-1, e].
        Some(e) => e as f64,
        // NaN or Inf. Infinity, so a convergence test fails and a distance test rejects: either
        // is preferable to a NaN quietly making every comparison false.
        None => f64::INFINITY,
    }
}

/// `log2` of a complex bignum's magnitude, to within an octave (the larger component dominates).
fn log2_abs_c(x: &BigFloat, y: &BigFloat) -> f64 {
    log2_abs_bf(x).max(log2_abs_bf(y))
}

/// The two magnifications a feature solve needs. ⭐**They are not the same number** as soon as
/// the user asks to be taken deeper than they currently are, and conflating them makes the
/// headline case — *"I am on this point, now take me to 1e58000×"* — fail as `TooFar`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolveScale {
    /// log2 of the magnification the ANSWER must be accurate for. Sets the working precision and
    /// the convergence tolerance: ask for more depth and you get more digits.
    pub log2_target: f64,
    /// log2 of the magnification of the view the SEED came from. Sets only how far a converged
    /// point may be from the seed before it is reported as [`MisiurewiczMiss::TooFar`] — a
    /// judgement about the view the user is LOOKING AT, which is why it cannot use the target.
    pub log2_seed: f64,
}

impl SolveScale {
    /// Solve at the depth you are already at: seed and target are the same view.
    pub fn here(log2_mag: f64) -> Self {
        Self { log2_target: log2_mag, log2_seed: log2_mag }
    }
}

/// Why a Misiurewicz solve did not produce a usable point.
///
/// ⭐These are NOT interchangeable, and collapsing them into `None` cost a user an afternoon: a
/// solve that converges onto a real point far outside the view was reported as "no point
/// converged near the view — navigate closer", when the honest advice was the opposite. The
/// caller needs to tell "there is nothing here" from "there is something, and it is over there".
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MisiurewiczMiss {
    /// `preperiod` or `period` was zero, or the formula has no Misiurewicz points.
    BadRequest,
    /// Newton stalled — the derivative vanished, or 80 iterations did not reach tolerance.
    NotConverged,
    /// Newton found a point, but it is `log2_view_widths` octaves' worth of view-widths from the
    /// seed. A large value means the requested (k, p) describes a feature far COARSER than the
    /// current view, and the way to reach it is to zoom OUT, not in.
    ///
    /// ⚠A LOG, because the linear ratio overflows `f64` in the other direction just as readily as
    /// the tolerances underflow: a point 1e400 view-widths out is not an unusual answer here.
    TooFar { log2_view_widths: f64 },
    /// A root was found but its orbit is not actually pre-periodic with this (k, p) — the pair
    /// does not fit. `residual` is |Z_{k+p} − Z_k|.
    NotPreperiodic { residual: f64 },
}

/// Newton-solve for the Misiurewicz point of pre-period `k` and period `p` nearest `center`.
///
/// [`SolveScale`] carries the depth to solve FOR and the depth the seed came FROM, both as
/// **log2 — not the magnification itself**, so the solve reaches the depths this app renders at.
/// An `f64` magnification capped it at 2^1020 ≈ 1e307: a request for 1e58000× came back with a
/// centre good to ~307 digits, and navigating there rendered a single flat colour, because the
/// coordinate was wrong by tens of thousands of digits (field report 2026-08-30).
///
/// Returns why it failed rather than a bare `None`; see [`MisiurewiczMiss`].
pub fn find_misiurewicz(
    center: &[BigFloat; 2],
    preperiod: u32,
    period: u32,
    scale: SolveScale,
    formula: u32,
) -> Result<Misiurewicz, MisiurewiczMiss> {
    if preperiod == 0 || period == 0 {
        return Err(MisiurewiczMiss::BadRequest);
    }
    if !scale.log2_target.is_finite() || !scale.log2_seed.is_finite() {
        return Err(MisiurewiczMiss::BadRequest);
    }
    let log2_mag = scale.log2_target;
    let p = precision_for_octaves(log2_mag.max(0.0).ceil() as u64);
    let Some(k) = formula_power(formula) else {
        return Err(MisiurewiczMiss::BadRequest);
    };
    let one = bf(1.0, p);
    let kf = bf(k as f64, p);
    // The view width, and the convergence tolerance, as log2. (3/mag, and a billionth of it.)
    const LOG2_3: f64 = 1.584_962_500_721_156_2;
    const LOG2_1E9: f64 = 29.897_352_853_986_263;
    let span_l2 = LOG2_3 - log2_mag.max(0.0);
    let tol_l2 = span_l2 - LOG2_1E9;
    // The view the SEED came from, which is the one the "is this the feature you are looking at?"
    // test has to be about. Solving for a deeper target legitimately leaves the answer thousands
    // of (tiny) target view-widths from the seed — that is what asking to be taken deeper means.
    let seed_span_l2 = LOG2_3 - scale.log2_seed.max(0.0);
    let total = preperiod + period;

    let mut cx = center[0].clone();
    let mut cy = center[1].clone();
    let mut converged = false;
    for _ in 0..80 {
        // Iterate to k+p, capturing (Z_k, D_k) at the pre-period and (Z_{k+p}, D_{k+p}) at the end.
        let mut zx = bf(0.0, p);
        let mut zy = bf(0.0, p);
        let mut dx = bf(0.0, p);
        let mut dy = bf(0.0, p);
        let (mut zkx, mut zky, mut dkx, mut dky) = (bf(0.0, p), bf(0.0, p), bf(0.0, p), bf(0.0, p));
        for i in 0..total {
            // D_{n+1} = k·Z_n^{k-1}·D_n + 1 ; Z_{n+1} = Z_n^k + c  (Z_n^{k-1} uses the pre-step Z_n).
            let (zk1x, zk1y) = if k == 2 { (zx.clone(), zy.clone()) } else { cpow_bf(&zx, &zy, k - 1, p) };
            let (mzx, mzy) = cmul_bf(&zk1x, &zk1y, &dx, &dy, p);
            let ndx = mzx.mul(&kf, p, RM).add(&one, p, RM);
            let ndy = mzy.mul(&kf, p, RM);
            let (nzx, nzy) = step_bf(&zx, &zy, &cx, &cy, formula, p);
            zx = nzx;
            zy = nzy;
            dx = ndx;
            dy = ndy;
            if i + 1 == preperiod {
                zkx = zx.clone();
                zky = zy.clone();
                dkx = dx.clone();
                dky = dy.clone();
            }
        }
        // Newton: c -= F / F' = F · conj(F') / |F'|².
        let fx = zx.sub(&zkx, p, RM);
        let fy = zy.sub(&zky, p, RM);
        let dfx = dx.sub(&dkx, p, RM);
        let dfy = dy.sub(&dky, p, RM);
        let denom = dfx.mul(&dfx, p, RM).add(&dfy.mul(&dfy, p, RM), p, RM);
        // EXACTLY zero. `to_f64(&denom) == 0.0` also fired for a denom that is merely tiny —
        // routine at these depths, where |F'| is far below f64's floor — and turned a converged
        // solve into "not converged".
        if denom.is_zero() {
            return Err(MisiurewiczMiss::NotConverged);
        }
        let (numx, numy) = cmul_bf(&fx, &fy, &dfx, &neg_bf(&dfy, p), p);
        let stepx = numx.div(&denom, p, RM);
        let stepy = numy.div(&denom, p, RM);
        cx = cx.sub(&stepx, p, RM);
        cy = cy.sub(&stepy, p, RM);
        if log2_abs_c(&stepx, &stepy) < tol_l2 {
            converged = true;
            break;
        }
    }
    if !converged {
        return Err(MisiurewiczMiss::NotConverged);
    }
    // ⭐VERIFIED BEFORE THE DISTANCE CHECK, and the order is load-bearing. Newton will happily
    // converge onto SOMETHING for a (k, p) that does not fit the orbit — a hand-typed (5,332) at
    // a 2.77e89x dendrite lands 1.9e56 view-widths out — and reporting that as "a point, far
    // away" would send the user zooming out after something that is not there. Rejecting the
    // mis-fit first is what lets `TooFar` mean "a REAL point, elsewhere".
    // Verify genuine pre-periodicity: recompute the residual |Z_{k+p} − Z_k| (bounded orbit ⇒ O(1)
    // values, so an absolute floor cleanly separates a real solution (≈0) from a (k,p) mismatch).
    let (mut zx, mut zy) = (bf(0.0, p), bf(0.0, p));
    let (mut zkx, mut zky) = (bf(0.0, p), bf(0.0, p));
    for i in 0..total {
        let (nzx, nzy) = step_bf(&zx, &zy, &cx, &cy, formula, p);
        zx = nzx;
        zy = nzy;
        if i + 1 == preperiod {
            zkx = zx.clone();
            zky = zy.clone();
        }
    }
    let res =
        (to_f64(&zx.sub(&zkx, p, RM)).powi(2) + to_f64(&zy.sub(&zky, p, RM)).powi(2)).sqrt();
    if res > 1.0e-6 {
        return Err(MisiurewiczMiss::NotPreperiodic { residual: res });
    }
    // The point should sit within a few view-widths of the seed, or it is not the feature the
    // user is looking at. ⭐Reported with its DISTANCE rather than swallowed: a converged solve
    // this far out is a real point at a coarser scale, and saying how far turns a dead end into a
    // direction. (Measured at a 2.77e89× dendrite: Newton converged in two steps onto a genuine
    // (437,3) point 4.04e-77 away — 4.7e11 view-widths — and the old `None` told the user to
    // navigate CLOSER, the exact opposite of what would have found it.)
    let ddx = cx.sub(&center[0], p, RM);
    let ddy = cy.sub(&center[1], p, RM);
    let dist_l2 = log2_abs_c(&ddx, &ddy);
    if dist_l2 > seed_span_l2 + 3.0 {
        return Err(MisiurewiczMiss::TooFar { log2_view_widths: dist_l2 - seed_span_l2 });
    }
    Ok(Misiurewicz { preperiod, period, cx, cy })
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
mod misiurewicz_detection {
    use super::*;

    fn detect(re: &str, im: &str, prec: usize, max_iter: u32, max_period: u32) -> Option<(u32, u32)> {
        let cx = crate::parse_bf_prec(re, prec).expect("centre parses");
        let cy = crate::parse_bf_prec(im, prec).expect("centre parses");
        detect_misiurewicz(&cx, &cy, 0, max_iter, max_period, prec)
    }

    /// Two Misiurewicz points whose orbits can be checked by hand, so a wrong answer here is not a
    /// matter of interpretation.
    ///
    /// c = -2:  0 -> -2 -> 2 -> 2 -> ...      lands on the fixed point 2 after 2 steps => (2, 1)
    /// c =  i:  0 ->  i -> -1+i -> -i -> -1+i lands on a 2-cycle after 2 steps        => (2, 2)
    #[test]
    fn the_two_textbook_points_are_identified() {
        assert_eq!(detect("-2.0", "0.0", 128, 64, 32), Some((2, 1)), "antenna tip");
        assert_eq!(detect("0.0", "1.0", 128, 64, 32), Some((2, 2)), "c = i");
    }

    /// The spiral this was written for: the centre of a 1.6e39x view whose structure is organised
    /// around a Misiurewicz point. Independently identified as (49, 3) with mpmath, then Newton
    /// refined to 522 digits and RENDERED at 1e500x, where it shows the same spiral rather than
    /// noise — which is the check that the pair is right, not merely plausible.
    ///
    /// The separation at this centre is 6.6e-39, so this doubles as the regression pin for the
    /// f64 pre-filter: an implementation that ranked candidates in f64 would see every one of them
    /// as exactly zero and could return any pair at all.
    #[test]
    fn a_deep_spiral_centre_resolves_to_its_misiurewicz_pair() {
        let got = detect(
            "-0.088792613303098660153845052309701500569653558627720875436411309366560178250182",
            "0.654809144755247929391298652387765097829565958367438788263142461373782142678715",
            256,
            600,
            64,
        );
        assert_eq!(got, Some((49, 3)), "expected the (49,3) pair the render confirmed");
    }

    /// An escaping orbit is not pre-periodic, and a point well inside the main cardioid has no
    /// near-return to find. Both must decline rather than invent a pair.
    #[test]
    fn points_without_a_misiurewicz_pair_are_declined() {
        assert_eq!(detect("2.0", "2.0", 128, 64, 32), None, "escapes immediately");
    }
}

#[cfg(test)]
mod aux_bla_oracle {
    use super::*;

    /// EXPERIMENT (run: `cargo test -p fractadyne-core escape_length_vs_precision -- --ignored
    /// --nocapture`): at a Misiurewicz centre, how long does the computed reference orbit stay
    /// true as a function of ARITHMETIC precision? The point is parsed at full precision (116
    /// digits ≈ 385 bits); only the orbit arithmetic varies. A Misiurewicz orbit is repelling, so
    /// per-iteration rounding error is amplified until the computed orbit spuriously escapes —
    /// the hypothesis is that the grand-tour crossover's `len=626` reference (built at 207 bits),
    /// e63's "natural escape at 256,753" and e72's 602,516 are ALL this one mechanism at
    /// different precisions, and that the picker itself goes blind at low precision (no candidate
    /// survives phase 1, so `best_reference` falls back to "longest escaper").
    /// COMPANION EXPERIMENT (run: `cargo test -p fractadyne-core --release --lib
    /// escape_length_of_rounded_centre -- --ignored --nocapture`): the ROUNDED-POINT axis of the
    /// cliff. `Playback::sample` used to route a pinned-centre glide through
    /// `lerp_bf(a, a, ease, p)` with `p` = the CURRENT interpolated depth's precision — which is
    /// not the identity: it ROUNDS the centre to `p` bits, i.e. hands the reference machinery a
    /// genuinely different point. This measures the true escape length (high-precision
    /// arithmetic) of `lerp_bf(centre, centre, 0.5, k)` for the glide precisions the grand tour
    /// actually produces. However good the picker, it cannot beat a wrong input point.
    #[test]
    #[ignore]
    fn escape_length_of_rounded_centre() {
        let cxs = "-1.0109636384562213181006238475735192993836101418531854095957676926471683503366629508912671364125546238220995191834757e-1";
        let cys = "9.5628651080914147131604703998237075557983304380930462483482733212267499793490593467836270525491219946548323699651521e-1";
        let cx = crate::parse_bf(cxs).unwrap();
        let cy = crate::parse_bf(cys).unwrap();
        let max_iter = 700_000u32;
        let arith = 480usize; // far above every cliff measured on the arithmetic axis
        let zero = BigFloat::from_f64(0.0, arith);
        for k in [64usize, 78, 96, 128, 157, 206, 300] {
            let rx = crate::lerp_bf(&cx, &cx, 0.5, k);
            let ry = crate::lerp_bf(&cy, &cy, 0.5, k);
            let n = orbit_length_bf(&zero, &zero, &rx, &ry, 0, max_iter, arith);
            println!(
                "lerp'd at {k:4} bits -> true escape {n}{}",
                if n >= max_iter { "  (SURVIVED)" } else { "" }
            );
        }
    }

    #[test]
    #[ignore]
    fn escape_length_vs_precision() {
        let cxs = "-1.0109636384562213181006238475735192993836101418531854095957676926471683503366629508912671364125546238220995191834757e-1";
        let cys = "9.5628651080914147131604703998237075557983304380930462483482733212267499793490593467836270525491219946548323699651521e-1";
        let cx = crate::parse_bf(cxs).unwrap();
        let cy = crate::parse_bf(cys).unwrap();
        let max_iter = 700_000u32;
        for p in [128usize, 160, 180, 207, 240, 286, 340, 400, 480] {
            let zero = BigFloat::from_f64(0.0, p);
            let n = orbit_length_bf(&zero, &zero, &cx, &cy, 0, max_iter, p);
            println!("prec {p:4} bits -> escape length {n}{}", if n >= max_iter { "  (SURVIVED)" } else { "" });
        }
    }

    // Extending a cached (truncated) orbit must be byte-identical to a from-scratch build to the same
    // length — this is what lets a deep dive reuse the prior orbit instead of recomputing every step.
    #[test]
    fn extend_orbit_is_byte_identical_to_fresh() {
        let p = 220usize;
        let zero = BigFloat::from_f64(0.0, p);
        let cases: [(&str, &str, u32, u32); 3] = [
            // Seahorse boundary point: survives thousands of iters → the extend path actually runs.
            ("-0.7436438870371587047521915061147707", "0.131825904205311970493132056385139", 1500, 6000),
            ("-0.7436438870371587047521915061147707", "0.131825904205311970493132056385139", 64, 5000),
            // Fast-escaping exterior point: coarse already escaped → extend is a no-op, still matches.
            ("1.0", "0.5", 64, 5000),
        ];
        for (cxs, cys, coarse, full) in cases {
            let cx = crate::parse_bf(cxs).unwrap();
            let cy = crate::parse_bf(cys).unwrap();
            let (fresh, fresh_len) = reference_orbit(&zero, &zero, &cx, &cy, 0, full, p);
            let (pre, _pl, tail) = reference_orbit_t(&zero, &zero, &cx, &cy, 0, coarse, p);
            let (ext, ext_len, _t) = extend_reference_orbit(&pre, &tail, &cx, &cy, 0, full, p);
            assert_eq!(ext_len, fresh_len, "len mismatch @({cxs},{cys}) coarse {coarse} full {full}");
            assert_eq!(ext, fresh, "extended != fresh @({cxs},{cys}) coarse {coarse} full {full}");
        }
    }

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

    // The Misiurewicz finder must Newton-snap from a nearby seed onto the exact pre-periodic point.
    #[test]
    fn misiurewicz_solver_snaps_to_known_points() {
        // c = -2 is Misiurewicz (2,1): Z_1=-2, Z_2=2, Z_3=2 ⇒ Z_3=Z_2.
        let seed = [crate::parse_bf("-1.98").unwrap(), crate::parse_bf("0.01").unwrap()];
        let m = find_misiurewicz(&seed, 2, 1, SolveScale::here(0.0), 0).expect("M(2,1) near c=-2");
        assert!(
            (to_f64(&m.cx) + 2.0).abs() < 1e-12 && to_f64(&m.cy).abs() < 1e-12,
            "M(2,1) should be c=-2, got ({}, {})",
            to_f64(&m.cx),
            to_f64(&m.cy)
        );

        // The verified three-spar Misiurewicz (4,1) (see validation/misiurewicz-4-1.fdn).
        let seed = [crate::parse_bf("-0.10110").unwrap(), crate::parse_bf("0.95629").unwrap()];
        let m = find_misiurewicz(&seed, 4, 1, SolveScale::here(1.0e3f64.log2()), 0).expect("M(4,1) three-spar");
        assert!(
            (to_f64(&m.cx) - (-0.10109636384562216)).abs() < 1e-12
                && (to_f64(&m.cy) - 0.9562865108091415).abs() < 1e-12,
            "M(4,1) off: ({}, {})",
            to_f64(&m.cx),
            to_f64(&m.cy)
        );
    }
}

#[cfg(test)]
mod sa_budget_tests {
    use super::sa_step_budget;

    /// ⭐Every blessed fixture must clear the budget with margin, or a corpus/golden re-bless is
    /// being smuggled in as a "tuning" change. The rows are the corpus's deep half — (working
    /// precision, iteration count), iteration count being a HARD upper bound on the SA walk — so
    /// this fails before `--check` would, and names the row.
    #[test]
    fn sa_budget_clears_every_blessed_fixture() {
        // (slug, prec bits, iterations) — from validation/corpus/locations.toml; prec is the
        // octaves+64 the SA pass receives, and the assertion adds the 128-bit orbit headroom on
        // top so the bound holds under either precision reading. Measured actual walks are far
        // smaller (row 20: 78,231 steps where this ceiling says 600,008) — the ceiling is the
        // contract precisely so the test never depends on how early an orbit happens to escape.
        let rows: [(&str, usize, u32); 8] = [
            ("15-deep-3.7e163", 607, 1_600_000),
            ("16-deep-2.1e250", 895, 600_008),
            ("17-deep-4.2e275", 979, 600_008),
            ("09-deep-6.1e500", 1_727, 150_000),
            ("18-deep-4.1e508", 1_753, 600_008),
            ("19-deep-1.3e726", 2_476, 600_008),
            ("20-deep-1.2e1008", 3_413, 600_008),
            ("10-deep-4.6e1105", 3_737, 250_000),
        ];
        for (slug, prec, iters) in rows {
            let budget = sa_step_budget(prec + 128);
            assert!(
                budget >= iters,
                "{slug}: SA budget {budget} steps at {prec} bits is below its {iters}-iteration \
                 ceiling — the budget would change a BLESSED render; that needs a deliberate \
                 re-bless, not a constant edit"
            );
        }
    }

    /// ...and it actually bites where it exists to bite: at the measured 2.37e4000× build the
    /// natural walk was 439,915 steps at 13,353 bits (= 258 s); the budget must land well under
    /// that, or it is a no-op wearing a comment.
    #[test]
    fn sa_budget_bites_at_extreme_depth() {
        let b = sa_step_budget(13_353);
        assert!(
            b < 439_915 / 2,
            "budget at 13,353 bits is {b} steps — not meaningfully below the 439,915-step walk \
             that cost 258 s"
        );
        assert!(b >= 8, "the budget must never sit below MIN_SKIP, or SA silently dies entirely");
    }
}
