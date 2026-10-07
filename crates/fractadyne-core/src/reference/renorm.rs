//! ⭐⭐THE RENORMALIZED STEP (Zhuoran's "AT"; Imagina uses it): near a minibrot, one period of
//! the iteration is itself a Mandelbrot map, and a pixel there can take a whole period per step.
//!
//! For the iteration `f(z) = z² + c`, the `L`-step map from `z` is `F(z) = g(z²)` with
//! `g(w) = f^{L−1}(w + c)` — the first step is `z² + c`. Expanded about `z = 0`,
//!
//! ```text
//!   F(z) = Z_L(c) + a·z² + O(z⁴),    a = g′(0) = Π_{k=1}^{L−1} 2·z_k.
//! ```
//!
//! In `u = a·z` that is **`u ↦ u² + c′`** with `c′ = a·Z_L(c)`, the Mandelbrot map itself. Taking it
//! about this reference (`c = c₀ + δc`, orbit `Z_k`):
//!
//! ```text
//!   c′ = RefC + (a·b)·δc,   RefC = a·Z_L,   b = dZ_L/dc,
//! ```
//!
//! so a pixel starts at `u₀ = 0` and iterates `u² + c′` — one step per `L` iterations — instead of
//! perturbation steps over the period. The step is itself PERTURBED, about the reference's own
//! u-orbit `U_k = a·Z_{k·L}` (read from the orbit the shader already has): `δu′ = 2U·δu + δu² + δc′`
//! with `δc′ = (a·b)·δc`, rebasing like the main loop. Plain `u` in df32 failed a tuned minibrot —
//! at a u-space return `|u|` is tiny and `u²` fell under df32's resolution beside `c′ ≈ 1`, so every
//! cycle restarted the pixel (median error 34 iterations at the ladder's period 15,248). It leaves
//! (`z = Z_{k·L} + δu/a` at iteration `k·L`) when `|u|` passes the radius where the model stops
//! holding, and a pixel that never leaves within the cap never escapes.
//!
//! **Which `L`.** Any index where the orbit comes back near 0: a new minimum of `|Z_n|`. A
//! minibrot-centred reference has its own period there (`|Z_p| ≈ 0`, `RefC ≈ 0`), and a tuned
//! minibrot inside another also has its parent's period (the orbit nearly returns every 953
//! iterations inside the period-953 minibrot). The longest usable one wins.
//!
//! **When it holds** — each condition is a test below, all at a relative error of `2^EPS` per step:
//! - **The quartic term.** In `u`, `F` is `u² + c′ + κ·u⁴ + …`, `κ = g″(0)/(2a³) = ρ_L/(2a)` with
//!   `ρ = g″/g′²` carried by `ρ_{k+1} = 1/(2Z_k²) + ρ_k/(2Z_k)`. It is negligible for `|u| ≤ R`,
//!   `R² = 2^EPS/|κ|`; past `R` the pixel leaves. `R` under [`RENORM_R_MIN_LOG2`] cannot even hold
//!   the set (`|u| ≤ 2`), and the step is not offered — a tuned minibrot's own period, whose `κ`
//!   carries its parent's near-returns, is like that (2^-22 at the ladder's period 121,984).
//! - **The pixel's own `a`.** It is this reference's times `1 + η`, `η ≈ Σ δz_k/Z_k ≈ δc·σ`,
//!   `σ = Σ_{k=1}^{L−1} (dZ_k/dc)/Z_k`: a per-pixel `|δc|` limit.
//! - **`c′` linear in `δc`.** Dropped: `a·(d²Z_L/dc²)·δc²/2`. Another `|δc|` limit.
//! - No precision rule (Imagina's asks the view to span `2^-16` of `|RefC|` in `c′`): `δc′` and `δu`
//!   are floatexp relative to the u-reference, so a tuned minibrot deep inside its parent — period
//!   121,984 inside 953, whose view is ~1e-13 of `RefC` — takes the parent's step too.
//!
//! All of it is long products and sums, run in an extended-range accumulator: `|a|` runs far
//! outside `f64`, and the orbit dips below `f64`'s range in extended samples
//! ([`super::pack_sample`]).

use crate::{CFloatExp, FloatExp};

/// Relative error per step the model may make, as log₂ (`2^-24` ≈ 6e-8, finer than the BLA's
/// 1e-6 per skip).
pub const RENORM_EPS_LOG2: f64 = -24.0;
/// The exit radius must hold the set with room: `R ≥ 2^4`.
pub const RENORM_R_MIN_LOG2: f64 = 4.0;
/// `R` is capped so `R²` stays an f32 in the shader.
const RENORM_R_MAX_LOG2: f64 = 60.0;
/// The shortest step worth taking. A renormalized step costs several perturbation steps' worth of
/// floatexp, and `L = 1` (an empty product: κ = 0, every limit infinite) is the plain iteration
/// without the BLA — it passed every test at the ladder's period-128 minibrot and would have
/// replaced its BLA skips with one step per iteration.
pub const RENORM_MIN_LEN: u32 = 64;

/// A complex `m·2^e` with `f64` mantissas sharing one exponent: the period-long products and sums
/// below would overflow or underflow `f64` many times over. Renormalized only when the mantissa
/// drifts far from 1, so a step costs a few multiplies, and every rescale is an exact power of two.
#[derive(Clone, Copy, Debug)]
struct Xc {
    re: f64,
    im: f64,
    e: i64,
}

const XC_ZERO: Xc = Xc { re: 0.0, im: 0.0, e: 0 };

/// `2^k` for `k` in f64's normal range — exact.
fn pow2(k: i64) -> f64 {
    f64::from_bits(((k + 1023) as u64) << 52)
}

impl Xc {
    fn real(v: f64) -> Xc {
        Xc { re: v, im: 0.0, e: 0 }
    }

    fn is_zero(self) -> bool {
        self.re == 0.0 && self.im == 0.0
    }

    /// Bring the larger mantissa to `[1, 2)` when it has left `[2^-400, 2^400]`.
    fn norm(self) -> Xc {
        let m = self.re.abs().max(self.im.abs());
        if m == 0.0 || !m.is_finite() {
            return Xc { re: self.re, im: self.im, e: 0 };
        }
        if (pow2(-400)..=pow2(400)).contains(&m) {
            return self;
        }
        let k = ((m.to_bits() >> 52) & 0x7ff) as i64 - 1023;
        let s = pow2(-k);
        Xc { re: self.re * s, im: self.im * s, e: self.e + k }
    }

    fn mul(self, o: Xc) -> Xc {
        Xc {
            re: self.re * o.re - self.im * o.im,
            im: self.re * o.im + self.im * o.re,
            e: self.e + o.e,
        }
        .norm()
    }

    fn scale2(self) -> Xc {
        Xc { re: self.re * 2.0, im: self.im * 2.0, e: self.e }
    }

    fn add(self, o: Xc) -> Xc {
        if self.is_zero() {
            return o;
        }
        if o.is_zero() {
            return self;
        }
        let (hi, lo) = if self.e >= o.e { (self, o) } else { (o, self) };
        let de = hi.e - lo.e;
        if de > 1000 {
            return hi;
        }
        let s = pow2(-de);
        Xc { re: hi.re + lo.re * s, im: hi.im + lo.im * s, e: hi.e }.norm()
    }

    /// `1/self`. The mantissa is within `[2^-400, 2^400]` after `norm`, so `|m|²` stays in range.
    fn recip(self) -> Xc {
        let d = self.re * self.re + self.im * self.im;
        Xc { re: self.re / d, im: -self.im / d, e: -self.e }.norm()
    }

    /// `log2|self|` (`−∞` for zero).
    fn log2_abs(self) -> f64 {
        if self.is_zero() {
            return f64::NEG_INFINITY;
        }
        self.re.hypot(self.im).log2() + self.e as f64
    }

    fn from_cfe(c: CFloatExp) -> Xc {
        let e = match (c.re.m == 0.0, c.im.m == 0.0) {
            (true, true) => return XC_ZERO,
            (false, true) => c.re.e,
            (true, false) => c.im.e,
            (false, false) => c.re.e.max(c.im.e),
        } as i64;
        let part = |f: FloatExp| if f.m == 0.0 || f.e as i64 - e < -1000 { 0.0 } else { f.m * pow2(f.e as i64 - e) };
        Xc { re: part(c.re), im: part(c.im), e }.norm()
    }

    /// The value as `f64`s (0 below their range; the u-orbit is O(1) apart from its returns).
    fn to_f64s(self) -> (f64, f64) {
        if self.e < -1000 {
            return (0.0, 0.0);
        }
        let s = if self.e > 1000 { f64::INFINITY } else { pow2(self.e) };
        (self.re * s, self.im * s)
    }

    fn to_cfe(self) -> CFloatExp {
        let part = |m: f64| {
            if m == 0.0 {
                return FloatExp::ZERO;
            }
            let k = ((m.abs().to_bits() >> 52) & 0x7ff) as i64 - 1023;
            let e = (self.e + k).clamp(i32::MIN as i64 / 2, i32::MAX as i64 / 2) as i32;
            FloatExp::new(m * pow2(-k), e)
        };
        CFloatExp { re: part(self.re), im: part(self.im) }
    }
}

/// One packed orbit sample (either form, see [`super::pack_sample`]) as an exact `Xc`: an extended
/// dip keeps its own exponent instead of being multiplied out (and underflowing) in `f64`.
fn sample_xc(s: &[f32; 4]) -> Xc {
    if s[0] == 0.0 && s[2].abs() >= 2.0 {
        return Xc { re: s[2] as f64 - 4.0, im: s[3] as f64, e: s[1] as i64 }.norm();
    }
    Xc { re: s[0] as f64 + s[2] as f64, im: s[1] as f64 + s[3] as f64, e: 0 }.norm()
}

/// One renormalized step for this reference: the shader's `u ↦ u² + c′` over `len` iterations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenormStep {
    /// `L`, the iterations one step covers.
    pub len: u32,
    /// `a = Π_{k=1}^{L−1} 2·Z_k`.
    pub a: CFloatExp,
    /// `b = dZ_L/dc`.
    pub b: CFloatExp,
    /// `RefC = a·Z_L`: this reference's own `c′`.
    pub ref_c: CFloatExp,
    /// log₂ of the exit radius `R` in `u`.
    pub log2_r: f64,
    /// log₂ of the largest `|δc|` (from this reference) the model holds for.
    pub log2_dc_max: f64,
    /// log₂|κ| (diagnostics).
    pub log2_kappa: f64,
}

/// The longest usable renormalized step for this `orbit` (packed samples `Z_0 … Z_N`), or `None`
/// (see the module note for the tests).
pub fn renorm_step(orbit: &[[f32; 4]]) -> Option<RenormStep> {
    let n_max = orbit.len().checked_sub(1)?;
    let one = Xc::real(1.0);
    // State at index n (starting n = 1): A = Π_{k<n} 2Z_k, D = dZ_n/dc, E = d²Z_n/dc²,
    // S = Σ_{k<n} D_k/Z_k, rho = g″/g′² for the n-step map.
    let mut a = one;
    let mut d = one;
    let mut e = XC_ZERO;
    let mut s = XC_ZERO;
    let mut rho = XC_ZERO;
    let mut min_l2 = f64::INFINITY;
    let mut best = None;
    for (n, sample) in orbit.iter().enumerate().take(n_max + 1).skip(1) {
        let z = sample_xc(sample);
        let zl2 = z.log2_abs();
        if zl2 < min_l2 {
            min_l2 = zl2;
            if let Some(step) = candidate(n as u32, z, a, d, e, s, rho) {
                best = Some(step);
            }
        }
        if n == n_max || z.is_zero() {
            break;
        }
        let z2 = z.scale2();
        let inv_z2 = z2.recip(); // 1/(2Z_n)
        s = s.add(d.mul(z.recip()));
        rho = inv_z2.mul(inv_z2).scale2().add(rho.mul(inv_z2)); // 1/(2Z²) = 2·(1/(2Z))²
        a = a.mul(z2);
        e = e.mul(z2).add(d.mul(d).scale2());
        d = d.mul(z2).add(one);
    }
    best
}

/// The step at `L = n` if it passes every test in the module note.
#[allow(clippy::too_many_arguments)]
fn candidate(n: u32, z: Xc, a: Xc, d: Xc, e: Xc, s: Xc, rho: Xc) -> Option<RenormStep> {
    if n < RENORM_MIN_LEN || a.is_zero() || d.is_zero() {
        return None;
    }
    let kappa = rho.mul(a.scale2().recip());
    let log2_kappa = kappa.log2_abs();
    let log2_r = ((RENORM_EPS_LOG2 - log2_kappa) / 2.0).min(RENORM_R_MAX_LOG2);
    if !(log2_r >= RENORM_R_MIN_LOG2) {
        return None;
    }
    let ref_c = a.mul(z);
    // Per-pixel |δc|: the a-mismatch |σ·δc| ≤ ε and the dropped |a·E·δc²/2| ≤ ε.
    let by_sigma = RENORM_EPS_LOG2 - s.log2_abs();
    let by_curve = (RENORM_EPS_LOG2 + 1.0 - a.log2_abs() - e.log2_abs()) / 2.0;
    let log2_dc_max = by_sigma.min(by_curve);
    Some(RenormStep {
        len: n,
        a: a.to_cfe(),
        b: d.to_cfe(),
        ref_c: ref_c.to_cfe(),
        log2_r,
        log2_dc_max,
        log2_kappa,
    })
}

/// ⭐⭐THE U-SPACE BLA: the BLA tree (GPU-flattened, [`super::bla_to_gpu`]) over this step's
/// u-reference `U_k = a·Z_{k·L}`, `k = 0..=K`, `K = (orbit.len() − 1)/L` — the samples the
/// shader's `rn_ref_u` reads. A renormalized step is still one step per `L` iterations, and near
/// a tuned minibrot a pixel takes thousands of them: at the ladder's period 951,094 (= 953·998)
/// the u-reference repeats every 998 steps and a pixel follows it for ~7 cycles, ~7,400 steps,
/// 97% of that render. In u-space the parent's near-returns are gone (they are what the step
/// absorbed), so the linear skip that cannot cross them in z-space merges across a u-cycle: the
/// perturbed u-step `δu′ = 2U·δu + δu² + δc′` is the Mandelbrot one, with `c′`'s offset `δc′ =
/// (a·b)·δc`, and its tree is [`super::build_bla_mandel`]'s over the u-samples. `dc_max` is the
/// view's worst `|δc|` (as for the z-space tree); a pixel the step takes is also within the step's
/// own limit, the smaller of the two is used. Empty when there is nothing to skip (`K < 2`).
pub fn renorm_bla_gpu(orbit: &[[f32; 4]], step: &RenormStep, dc_max: FloatExp, eps: f64) -> Vec<[f32; 4]> {
    let l = step.len as usize;
    if l == 0 || orbit.is_empty() {
        return Vec::new();
    }
    let k = (orbit.len() - 1) / l;
    if k < 2 {
        return Vec::new();
    }
    let a = Xc::from_cfe(step.a);
    let u: Vec<[f32; 4]> = (0..=k)
        .map(|j| {
            let (x, y) = a.mul(sample_xc(&orbit[j * l])).to_f64s();
            super::pack_sample(x, y)
        })
        .collect();
    let lim = FloatExp::from_f64(1.0).mul_pow2(step.log2_dc_max);
    let dc = if dc_max.lt(lim) { dc_max } else { lim };
    let dcp = (step.a * step.b).abs() * dc;
    let tree = super::build_bla_mandel(&u, dcp, eps, super::AuxAggParams::default());
    // A view that is shallow in u-space (its |δc′| near the radii themselves: the ladder's 2.1e57
    // scene spans ~1e-4 of c′) merges nothing, and a tree that only ever offers single steps
    // costs a search per step for no skip (measured there: 655 → 738 ms).
    if tree.iter().skip(1).all(|level| level.iter().all(|n| n.r.m <= 0.0)) {
        return Vec::new();
    }
    super::bla_to_gpu(&tree)
}

/// The node count (`[f32; 4]` × 4 per node) of [`renorm_bla_gpu`]'s tree for `K` u-steps: level 0
/// has `K` nodes and each level above half (rounded up) as many, down to one.
pub fn renorm_bla_nodes(k: u32) -> usize {
    let mut n = k as usize;
    let mut total = 0;
    while n > 0 {
        total += n;
        if n == 1 {
            break;
        }
        n = n.div_ceil(2);
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A reference orbit in f64 (shallow minibrots only), packed like the real one.
    fn orbit_f64(cx: f64, cy: f64, n: usize) -> Vec<[f32; 4]> {
        let (mut x, mut y) = (0.0f64, 0.0f64);
        let mut out = vec![super::super::pack_sample(0.0, 0.0)];
        for _ in 0..n {
            let nx = x * x - y * y + cx;
            y = 2.0 * x * y + cy;
            x = nx;
            out.push(super::super::pack_sample(x, y));
        }
        out
    }

    fn cmul(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
        (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
    }

    fn cf(c: CFloatExp) -> (f64, f64) {
        (c.re.to_f64(), c.im.to_f64())
    }

    #[test]
    fn period_two_matches_its_closed_form() {
        // c₀ = −1: Z = 0, −1, 0. F(z) = (z² − 1)² − 1 = z⁴ − 2z², so a = −2, b = dZ₂/dc = 2Z₁·1 + 1
        // = −1, RefC = a·Z₂ = 0, and in u = −2z the map is u² + c′ − u⁴/8: κ = −1/8.
        let orbit = orbit_f64(-1.0, 0.0, 2);
        let (mut a, mut d, mut e, mut s, mut rho) = (Xc::real(1.0), Xc::real(1.0), XC_ZERO, XC_ZERO, XC_ZERO);
        let z1 = sample_xc(&orbit[1]);
        let z2 = z1.scale2();
        let inv = z2.recip();
        s = s.add(d.mul(z1.recip()));
        rho = inv.mul(inv).scale2().add(rho.mul(inv));
        a = a.mul(z2);
        e = e.mul(z2).add(d.mul(d).scale2());
        d = d.mul(z2).add(Xc::real(1.0));
        let _ = (e, s);
        assert_eq!(cf(a.to_cfe()), (-2.0, 0.0));
        assert_eq!(cf(d.to_cfe()), (-1.0, 0.0));
        let kappa = rho.mul(a.scale2().recip());
        assert!((kappa.log2_abs() - (-3.0)).abs() < 1e-12, "{}", kappa.log2_abs());
        // ...and it is refused anyway: shorter than RENORM_MIN_LEN, and R² = 2^-24/(1/8) is far
        // below the set.
        assert!(candidate(2, sample_xc(&orbit[2]), a, d, e, s, rho).is_none());
    }

    /// The model against the iteration itself, at the ladder's period-953 minibrot (scene 56,
    /// 1.3e53×): for pixels around the nucleus, k renormalized steps from u = 0 must land where
    /// k·953 direct bignum iterations do, scaled by a — to within the model's error.
    #[test]
    fn renormalized_steps_follow_the_direct_iteration() {
        use crate::{parse_real_expr, to_f64, BigFloat};
        let prec = 400;
        let cx = parse_real_expr("-2.804105430550454669840777002898397927204098765083452471337241940737508121866324414807243345475023e-2", prec).unwrap();
        let cy = parse_real_expr("6.948927538996523858929943394989672880373767486737755673829879259356243644967862160620146096103277e-1", prec).unwrap();
        let zero = BigFloat::from_f64(0.0, prec);
        let (orbit, _, _) = super::super::reference_orbit_t(&zero, &zero, &cx, &cy, 0, 960, prec);
        // The view at 1.34e53×: radius 2/zoom ≈ 2^-176.
        let log2_view = (2.0f64 / 1.342765e53).log2();
        let step = renorm_step(&orbit).expect("the period is usable");
        assert_eq!(step.len, 953, "{step:?}");
        let (a, b, rc) = (cf(step.a), cf(step.b), cf(step.ref_c));
        let ab = cmul(a, b);
        let r = step.log2_r.exp2();
        let view = log2_view.exp2();
        let mut checked = 0;
        for (i, frac) in [0.002f64, 0.01, 0.05, 0.2, 0.6].iter().enumerate() {
            let th = 0.7 + i as f64;
            let dc = (view * frac * th.cos(), view * frac * th.sin());
            assert!(dc.0.hypot(dc.1).log2() <= step.log2_dc_max, "the test pixel is inside the model's |dc|");
            let cp = (rc.0 + ab.0 * dc.0 - ab.1 * dc.1, rc.1 + ab.0 * dc.1 + ab.1 * dc.0);
            let pcx = cx.add(&BigFloat::from_f64(dc.0, prec), prec, crate::bignum::RM);
            let pcy = cy.add(&BigFloat::from_f64(dc.1, prec), prec, crate::bignum::RM);
            let (mut zx, mut zy) = (zero.clone(), zero.clone());
            let mut u = (0.0f64, 0.0f64);
            for _k in 1..=6 {
                for _ in 0..953 {
                    let (nx, ny) = super::super::step_bf(&zx, &zy, &pcx, &pcy, 0, prec);
                    zx = nx;
                    zy = ny;
                }
                u = (u.0 * u.0 - u.1 * u.1 + cp.0, 2.0 * u.0 * u.1 + cp.1);
                let um = u.0.hypot(u.1);
                if um > r || um > 1e6 {
                    break;
                }
                let uz = cmul(a, (to_f64(&zx), to_f64(&zy)));
                let tol = 1e-6 * um.max(1.0);
                assert!((uz.0 - u.0).hypot(uz.1 - u.1) <= tol, "dc {dc:?}: a·z {uz:?} vs u {u:?}");
                checked += 1;
            }
        }
        assert!(checked >= 10, "the comparison ran ({checked})");
    }

    /// The u-space tree against the steps it replaces, at the ladder's period-15,248 minibrot
    /// (953 × 16, 2.1e57×): the step is the parent's 953 and the u-reference repeats every 16 steps.
    /// For a pixel inside a deep view, each level's node at `k = 2^l` (span `2^l`) must land where
    /// `2^l` perturbed u-steps do, to within the tree's tolerance.
    #[test]
    fn the_u_space_tree_follows_the_u_steps_it_skips() {
        use crate::{parse_real_expr, BigFloat};
        let prec = 400;
        let cx = parse_real_expr("-2.804105430550454669840777002898397927204098765083451958014259277838710256889075333271267217529135e-2", prec).unwrap();
        let cy = parse_real_expr("6.948927538996523858929943394989672880373767486737755680968675269305405323393245987015207130043252e-1", prec).unwrap();
        let zero = BigFloat::from_f64(0.0, prec);
        let (orbit, _, _) = super::super::reference_orbit_t(&zero, &zero, &cx, &cy, 0, 15_248, prec);
        let step = renorm_step(&orbit).expect("the parent's period is usable");
        assert_eq!(step.len, 953, "{step:?}");
        let k = ((orbit.len() - 1) / 953) as u32;
        assert_eq!(k, 16);
        // A view as deep as the ladder's 1e68 scene (whose step is this same parent's): in u-space
        // 57's own view spans ~1e-4 of c′, where |B·δc′| outweighs every merged radius — a shallow
        // view, as a z-space one at 1e4× is — while 1e68's spans ~1e-15.
        let view = (4.0f64 / 2.0e68).log2();
        let eps = 1e-6;
        let tree = renorm_bla_gpu(&orbit, &step, FloatExp::from_f64(1.0).mul_pow2(view), eps);
        assert_eq!(tree.len(), renorm_bla_nodes(k) * 4);
        let decode = |m: [f32; 4], e: f32| -> (f64, f64) {
            let s = (e as f64).exp2();
            ((m[0] as f64 + m[1] as f64) * s, (m[2] as f64 + m[3] as f64) * s)
        };
        // The u-reference, as the shader forms it, and a pixel's δc′ a thousandth of the view out.
        let a = cf(step.a);
        let uref: Vec<(f64, f64)> = (0..=k as usize)
            .map(|j| {
                let (zx, zy) = super::super::sample_xy(&orbit[j * 953]);
                cmul(a, (zx, zy))
            })
            .collect();
        // Level 0's A is 2·U_k (node 0: U_0 = 0, so it can never apply).
        for (j, u) in uref.iter().enumerate().take(k as usize) {
            let (ax, ay) = decode(tree[j * 4], tree[j * 4 + 2][0]);
            assert!((ax - 2.0 * u.0).hypot(ay - 2.0 * u.1) <= 1e-6 * (1.0 + u.0.hypot(u.1)), "node {j}");
        }
        let ab = cmul(a, cf(step.b));
        let dc = 1e-3 * view.exp2();
        let dcp = cmul(ab, (dc * 0.6, dc * 0.8));
        let plain = |mut du: (f64, f64), from: usize, n: usize| {
            for i in from..from + n {
                let u = uref[i];
                let t = cmul((2.0 * u.0, 2.0 * u.1), du);
                let sq = cmul(du, du);
                du = (t.0 + sq.0 + dcp.0, t.1 + sq.1 + dcp.1);
            }
            du
        };
        let mut off = 0usize;
        let mut len = k as usize;
        let mut checked = 0;
        for l in 0.. {
            let span = 1usize << l;
            if span < k as usize && 1 < len {
                // The node at k = span (j = 1) covers steps span..2·span.
                let node = (off + 1) * 4;
                let du0 = plain((0.0, 0.0), 0, span);
                let (am, bm, ex, sp) = (tree[node], tree[node + 1], tree[node + 2], tree[node + 3]);
                assert_eq!(sp[0] as usize, span.min(k as usize - span), "level {l} span");
                let r = (ex[3] as f64) * (ex[2] as f64).exp2();
                if du0.0.hypot(du0.1) < r && sp[0] as usize == span {
                    let (ax, ay) = decode(am, ex[0]);
                    let (bx, by) = decode(bm, ex[1]);
                    let lin = cmul((ax, ay), du0);
                    let lin = (lin.0 + bx * dcp.0 - by * dcp.1, lin.1 + bx * dcp.1 + by * dcp.0);
                    let want = plain(du0, span, span);
                    let err = (lin.0 - want.0).hypot(lin.1 - want.1);
                    assert!(err <= 1e-4 * want.0.hypot(want.1), "level {l}: skip {lin:?} vs steps {want:?}");
                    checked += 1;
                }
            }
            if len <= 1 {
                break;
            }
            off += len;
            len = len.div_ceil(2);
        }
        assert!(checked >= 3, "the comparison ran at {checked} levels");
    }

    /// Probe (ignored; `cargo test --release -p fractadyne-core -- --ignored probe_u_space --nocapture`):
    /// the shader's u-space walk, BLA and all, on the CPU at the ladder's 1e68 scene, printing what
    /// the u-reference looks like and how far each skip goes.
    #[test]
    #[ignore]
    fn probe_u_space_at_the_1e68_minibrot() {
        use crate::{parse_real_expr, BigFloat};
        let prec = 280;
        let cx = parse_real_expr("-2.8041054305504546698407770028983979272040987650834524316481736852848679363432650309792659781081363390252362581574126e-2", prec).unwrap();
        let cy = parse_real_expr("6.9489275389965238589299433949896728803737674867377557299604836990561657887294711299974639904251203515742450986635039e-1", prec).unwrap();
        let zero = BigFloat::from_f64(0.0, prec);
        let (orbit, _, _) = super::super::reference_orbit_t(&zero, &zero, &cx, &cy, 0, 951_094, prec);
        let step = renorm_step(&orbit).expect("step");
        let l = step.len as usize;
        let k = (orbit.len() - 1) / l;
        println!("step {} K {k} log2|a| {:.1} log2|ab| {:.1} R 2^{:.0}", step.len, step.a.abs().log2(), (step.a * step.b).abs().log2(), step.log2_r);
        let a = cf(step.a);
        let uref: Vec<(f64, f64)> = (0..=k).map(|j| cmul(a, super::super::sample_xy(&orbit[j * l]))).collect();
        let mut mags: Vec<(f64, usize)> = uref.iter().enumerate().map(|(j, u)| (u.0.hypot(u.1), j)).collect();
        let lp: f64 = mags.iter().skip(1).take(k - 1).map(|(m, _)| (2.0 * m).log2()).sum();
        mags.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
        println!("smallest |U_k|: {:?}", &mags[..12]);
        println!("log2 |prod 2U_k| over the cycle: {lp:.1}; median |U| {:.3}", mags[mags.len() / 2].0);
        let view = (4.0f64 / 2.142891e68).log2();
        let eps = 1e-6;
        let tree = renorm_bla_gpu(&orbit, &step, FloatExp::from_f64(2.5).mul_pow2(view), eps);
        let decode = |m: [f32; 4], e: f32| -> (f64, f64) {
            let s = (e as f64).exp2();
            ((m[0] as f64 + m[1] as f64) * s, (m[2] as f64 + m[3] as f64) * s)
        };
        let mut lv = vec![];
        let (mut off, mut len) = (0usize, k);
        loop {
            lv.push((off, len));
            if len <= 1 {
                break;
            }
            off += len;
            len = len.div_ceil(2);
        }
        let ab = cmul(a, cf(step.b));
        let r2 = (2.0 * step.log2_r).exp2();
        for frac in [0.05f64, 0.2, 0.45] {
            let dc = frac * view.exp2();
            let dcp = cmul(ab, (dc * 0.6, dc * 0.8));
            let (mut du, mut ddu, mut kr, mut kk) = ((0.0f64, 0.0f64), (0.0f64, 0.0f64), 0usize, 0usize);
            let (mut trips, mut skipped, mut rebases) = (0u64, 0u64, 0u64);
            let mut hist = [0u64; 12];
            let k_max = 28_532_820usize.div_ceil(l);
            let mut out = false;
            while kk < k_max && trips < 200_000 {
                trips += 1;
                let dum = du.0.hypot(du.1);
                let mut applied = false;
                for lvl in (0..lv.len()).rev() {
                    let stepn = 1usize << lvl;
                    if kr & (stepn - 1) != 0 {
                        continue;
                    }
                    let j = kr >> lvl;
                    if j >= lv[lvl].1 {
                        continue;
                    }
                    let n = (lv[lvl].0 + j) * 4;
                    let ex = tree[n + 2];
                    let span = tree[n + 3][0] as usize;
                    if span == 0 || kr + span > k || kk + span > k_max {
                        continue;
                    }
                    let r = ex[3] as f64 * (ex[2] as f64).exp2();
                    if !(dum < r) {
                        continue;
                    }
                    let am = decode(tree[n], ex[0]);
                    let bm = decode(tree[n + 1], ex[1]);
                    let t = cmul(am, du);
                    let t2 = cmul(bm, dcp);
                    let ndu = (t.0 + t2.0, t.1 + t2.1);
                    let nkr = kr + span;
                    let u = (uref[nkr].0 + ndu.0, uref[nkr].1 + ndu.1);
                    if u.0 * u.0 + u.1 * u.1 > r2 {
                        continue;
                    }
                    let dd = cmul(am, ddu);
                    ddu = (dd.0 + bm.0, dd.1 + bm.1);
                    du = ndu;
                    kr = nkr;
                    kk += span;
                    hist[lvl.min(11)] += 1;
                    skipped += span as u64;
                    if u.0.hypot(u.1) < du.0.hypot(du.1) || (kr + 1) * l >= orbit.len() {
                        du = u;
                        kr = 0;
                        rebases += 1;
                    }
                    applied = true;
                    break;
                }
                if applied {
                    continue;
                }
                let uk = uref[kr];
                let t = cmul((2.0 * uk.0, 2.0 * uk.1), du);
                let sq = cmul(du, du);
                du = (t.0 + sq.0 + dcp.0, t.1 + sq.1 + dcp.1);
                kr += 1;
                kk += 1;
                let u = (uref[kr].0 + du.0, uref[kr].1 + du.1);
                if u.0.hypot(u.1) < du.0.hypot(du.1) || (kr + 1) * l >= orbit.len() {
                    du = u;
                    kr = 0;
                    rebases += 1;
                }
                if u.0 * u.0 + u.1 * u.1 > r2 {
                    out = true;
                    break;
                }
            }
            println!(
                "pixel {frac}: u-steps {kk} trips {trips} ({:.1}/trip) skipped {skipped} rebases {rebases} out {out}; skips by level {hist:?}",
                kk as f64 / trips as f64
            );
        }
    }

    #[test]
    fn a_trivial_step_is_never_offered() {
        // L = 1 is an empty product (κ = 0, every limit infinite): it passes the model tests, and
        // must still never be chosen. A wide view of the main cardioid's centre has nothing else.
        let orbit = orbit_f64(-0.1, 0.05, 200);
        assert!(renorm_step(&orbit).map_or(true, |st| st.len >= RENORM_MIN_LEN), "{:?}", renorm_step(&orbit));
    }

    #[test]
    fn an_extended_dip_keeps_its_exponent() {
        let mut s = [0.0f32, -5000.0, 4.0 + 1.25, 0.0]; // 1.25·2^-5000 in the extended form
        s[1] = -5000.0;
        let x = sample_xc(&s);
        assert!((x.log2_abs() - (1.25f64.log2() - 5000.0)).abs() < 1e-9);
    }
}
