//! Finding a FEATURE near a point fast enough to steer a live zoom: the nucleus of a minibrot of
//! `z² + c` within a radius, located in perturbation space against the view's own reference
//! orbit, so its cost is the period's iterations in extended-range `f64` rather than in bignum
//! (`find_nucleus` solves in arbitrary precision and takes seconds at depth).
//!
//! The method is the standard one. A disc of radius `r` about `c₀` is carried through the
//! iterations to first order, `zₙ(c) ≈ zₙ(c₀) + zₙ'(c₀)(c − c₀)`; the first `n` at which that disc
//! can contain 0 (`|zₙ| < |zₙ'|·r`) is the period of a nucleus near `c₀`. Newton on `z_p(c) = 0`,
//! `c ← c − z_p / z_p'`, then locates it, and it is kept only if it lies inside the disc.
//!
//! Perturbation: `c = C + δc` and `zₙ = Zₙ + δzₙ` for the reference orbit `Zₙ` of `C`, with
//! `δzₙ₊₁ = (2Zₙ + δzₙ)δzₙ + δc`. Where the reference ends, or `|zₙ| < |δzₙ|`, the pixel REBASES to
//! the orbit's start (`δz ← z`, `Zₙ ← Z₀ = 0`), so a period longer than the reference is found
//! too. The derivative `zₙ' = dzₙ/dc` is of the whole `z`, unaffected by rebasing:
//! `zₙ₊₁' = 2zₙzₙ' + 1`.

use crate::{CFloatExp, FloatExp};

/// A nucleus found by [`find_nucleus_near`]: its position relative to the reference, and its period.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NucleusNear {
    pub dc: CFloatExp,
    pub period: u32,
}

/// Newton steps at most, and the step (as a fraction of the search radius) that counts as
/// converged: the nucleus is wanted to a small fraction of a pixel of the radius it was sought in.
const NEWTON_STEPS: usize = 24;
const NEWTON_TOL: f64 = 1.0e-9;
/// Escape radius² for the period search: an orbit past it cannot come back to 0.
const BAILOUT2: f64 = 16.0;
/// …and for a Newton walk, which gives up only before the values overflow.
const NEWTON_BAILOUT2: f64 = 1.0e200;
/// Periods the disc test names that are tried before the search gives up.
const CANDIDATES: usize = 4;
/// How often (iterations) the search asks whether it has been cancelled.
const CANCEL_EVERY: u32 = 8192;

fn cfe(re: f64, im: f64) -> CFloatExp {
    CFloatExp { re: FloatExp::from_f64(re), im: FloatExp::from_f64(im) }
}

fn norm2(a: CFloatExp) -> FloatExp {
    a.re * a.re + a.im * a.im
}

fn cdiv(a: CFloatExp, b: CFloatExp) -> CFloatExp {
    let inv = norm2(b).recip();
    let conj = CFloatExp { re: b.re, im: FloatExp { m: -b.im.m, e: b.im.e } };
    let n = a * conj;
    CFloatExp { re: n.re * inv, im: n.im * inv }
}

/// The reference orbit as the finder reads it: `Zₙ` from the packed df64 samples
/// (`[re.hi, im.hi, re.lo, im.lo]`), or `Z ≡ 0` (the orbit of `C = 0`) when there is none, in
/// which case `δc` is `c` itself.
struct Orbit<'a> {
    samples: &'a [[f32; 4]],
    len: usize,
}

impl Orbit<'_> {
    fn z(&self, n: usize) -> CFloatExp {
        match self.samples.get(n) {
            Some(s) if n < self.len => cfe(s[0] as f64 + s[2] as f64, s[1] as f64 + s[3] as f64),
            _ => CFloatExp::ZERO,
        }
    }
    /// Whether index `n` is the reference's last usable sample (or past it).
    fn ends_at(&self, n: usize) -> bool {
        self.len > 0 && n + 1 >= self.len
    }
}

/// How a [`walk`] ended.
enum Walked {
    /// All its iterations ran: `z` and `z'` are the last ones.
    Done(CFloatExp, CFloatExp),
    /// The disc could contain 0 at this iteration (a period).
    Period(u32),
    /// The orbit escaped.
    Escaped,
}

/// Iterate `δc` for `steps` iterations; with `r2` (a radius²), stop at the first `n` past `after`
/// at which the disc of that radius could contain 0. `None` when cancelled.
fn walk(orbit: &Orbit, dc: CFloatExp, steps: u32, r2: Option<FloatExp>, after: u32, cancel: &dyn Fn() -> bool) -> Option<Walked> {
    let one = cfe(1.0, 0.0);
    let mut dz = CFloatExp::ZERO;
    let mut d = CFloatExp::ZERO;
    let mut k = 0usize; // index into the reference
    for n in 1..=steps {
        if n % CANCEL_EVERY == 0 && cancel() {
            return None;
        }
        let zr = orbit.z(k);
        let z = zr + dz;
        d = (z * d).mul_f64(2.0) + one;
        dz = (zr.mul_f64(2.0) + dz) * dz + dc;
        k += 1;
        let zr = orbit.z(k);
        let z = zr + dz;
        let zz = norm2(z);
        // The period search stops at the escape radius; Newton's walks only where the values
        // would leave the extended range (a guess just outside the component grows before it
        // converges).
        if zz.to_f64() > if r2.is_some() { BAILOUT2 } else { NEWTON_BAILOUT2 } {
            return Some(Walked::Escaped);
        }
        if let Some(r2) = r2.filter(|_| n > after) {
            if zz.lt(norm2(d) * r2) {
                return Some(Walked::Period(n));
            }
        }
        if orbit.ends_at(k) || zz.lt(norm2(dz)) {
            dz = z;
            k = 0;
        }
    }
    Some(Walked::Done(orbit.z(k) + dz, d))
}

/// The nucleus of `z² + c` nearest in period to `c = C + dc0` within `radius` of it, if any, with
/// periods up to `max_period`. `orbit` is the reference orbit of `C` (packed df64 samples, the
/// first `orbit_len` usable), or empty for `C = 0` (`dc0` is then `c`). `cancel` is asked every few
/// thousand iterations; a cancelled search finds nothing.
pub fn find_nucleus_near(
    orbit: &[[f32; 4]],
    orbit_len: u32,
    dc0: CFloatExp,
    radius: FloatExp,
    max_period: u32,
    cancel: &dyn Fn() -> bool,
) -> Option<NucleusNear> {
    if !(radius.m > 0.0) || max_period == 0 {
        return None;
    }
    let orbit = Orbit { samples: orbit, len: (orbit_len as usize).min(orbit.len()) };
    let r2 = radius * radius;
    // The disc test is first order: a period it names may have no nucleus in the disc (Newton
    // leaves it, or fails). The next period it names is then tried, a few times.
    let mut after = 0;
    for _ in 0..CANDIDATES {
        let Walked::Period(period) = walk(&orbit, dc0, max_period, Some(r2), after, cancel)? else { return None };
        if let Some(dc) = newton(&orbit, dc0, period, r2, cancel)? {
            return Some(NucleusNear { dc, period });
        }
        after = period;
    }
    None
}

/// Newton on `z_period(c) = 0` from `dc0`: the nucleus, if it converges inside the disc of radius²
/// `r2` about `dc0` (`Some(None)`: it did not; `None`: cancelled).
fn newton(orbit: &Orbit, dc0: CFloatExp, period: u32, r2: FloatExp, cancel: &dyn Fn() -> bool) -> Option<Option<CFloatExp>> {
    let tol2 = r2.mul_f64(NEWTON_TOL * NEWTON_TOL);
    let mut dc = dc0;
    for _ in 0..NEWTON_STEPS {
        let Walked::Done(z, d) = walk(orbit, dc, period, None, 0, cancel)? else { return Some(None) };
        if norm2(d).m == 0.0 {
            return Some(None);
        }
        let step = cdiv(z, d);
        dc = dc - step;
        if !(dc.re.m.is_finite() && dc.im.m.is_finite()) || r2.mul_f64(64.0).lt(norm2(dc - dc0)) {
            return Some(None); // diverging, or gone far from the disc
        }
        if norm2(step).lt(tol2) {
            return Some((!r2.lt(norm2(dc - dc0))).then_some(dc));
        }
    }
    Some(None)
}

#[cfg(test)]
#[path = "../tests/unit/feature/tests.rs"]
mod tests;
