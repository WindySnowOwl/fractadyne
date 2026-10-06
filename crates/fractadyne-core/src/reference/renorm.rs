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
