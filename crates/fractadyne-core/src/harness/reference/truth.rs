//! THE REFERENCE-GRADE DWELL (`--truth`): a pixel's escape count with none of the approximations
//! a renderer makes, at every pixel of a view, so a render can be checked against something it
//! did not compute itself. Attached to `reference` (`#[path]`) and re-exported, as the oracle is.
//!
//! WHY (user request, 2026-11-03: golden references "created with no approximations or
//! optimizations that have the least chance of error", every pixel, filaments included). That day
//! twelve test views drew a flat disc or one flat colour for every reference pick, and the pick
//! audit, which compares two picks, called them VACUOUS: two renders sharing a fault agree.
//! [`naive_dwell_bf`] is the independent truth, but at ~0.1 s a point at 1e999× it can judge a
//! sample of pixels, not a frame, and a thin filament is exactly what a sample misses.
//!
//! HOW. Perturbation, but every iteration a FULL step: no BLA, no series approximation, no
//! renormalized step, no f32 or df32 arithmetic. The reference orbit is iterated in arbitrary
//! precision and every sample kept as a double-double (~106 bits) with its own 64-bit exponent
//! ([`Cdx`]), so no dip under- or overflows (the 2026-11-03 bug stored dips below 2^-1022 as 0).
//! The pixel's delta is carried the same way, with Zhuoran rebasing (|z| < |δz|, or the
//! reference's end) and a cancellation guard: a step where `z = Z + δz` loses more than
//! [`TRUTH_CANCEL_BITS`] to cancellation hands the pixel to plain arbitrary-precision iteration
//! ([`truth_naive`]) instead of trusting it. No code is shared with the GPU shader or the
//! reference builder. The harness checks this engine against [`truth_naive`] at a sample of
//! pixels and refuses a view where they disagree at a pixel whose answer is stable.
//!
//! Counts are recorded at three escape radii, since renderers differ: 2 (Fractal Zoomer's
//! default), 256 (Fractadyne's) and 4096 (Imagina's). The steps between radii vary across a
//! view (4 to 8 between 2 and 256 on bench scene 62), so no single offset converts one to another.

use super::*;

/// A double-double `hi + lo` with `|lo| <= ulp(hi) / 2`: ~106 bits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dd {
    pub hi: f64,
    pub lo: f64,
}

const DD_SPLITTER: f64 = 134_217_729.0; // 2^27 + 1

#[inline(always)]
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let bb = s - a;
    (s, (a - (s - bb)) + (b - bb))
}

#[inline(always)]
fn quick_two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    (s, b - (s - a))
}

#[inline(always)]
fn split(a: f64) -> (f64, f64) {
    let t = DD_SPLITTER * a;
    let hi = t - (t - a);
    (hi, a - hi)
}

/// `a·b` exactly as a sum of two doubles (Dekker; no FMA, so it is the same on every CPU).
#[inline(always)]
fn two_prod(a: f64, b: f64) -> (f64, f64) {
    let p = a * b;
    let (ah, al) = split(a);
    let (bh, bl) = split(b);
    (p, ((ah * bh - p) + ah * bl + al * bh) + al * bl)
}

impl Dd {
    pub const ZERO: Dd = Dd { hi: 0.0, lo: 0.0 };

    #[inline(always)]
    pub fn add(self, b: Dd) -> Dd {
        let (s1, s2) = two_sum(self.hi, b.hi);
        let (t1, t2) = two_sum(self.lo, b.lo);
        let (s1, s2) = quick_two_sum(s1, s2 + t1);
        let (hi, lo) = quick_two_sum(s1, s2 + t2);
        Dd { hi, lo }
    }

    #[inline(always)]
    pub fn neg(self) -> Dd {
        Dd { hi: -self.hi, lo: -self.lo }
    }

    #[inline(always)]
    pub fn sub(self, b: Dd) -> Dd {
        self.add(b.neg())
    }

    #[inline(always)]
    pub fn mul(self, b: Dd) -> Dd {
        let (p, e) = two_prod(self.hi, b.hi);
        let (hi, lo) = quick_two_sum(p, e + (self.hi * b.lo + self.lo * b.hi));
        Dd { hi, lo }
    }

    /// Times an exact power of two.
    #[inline(always)]
    fn scale(self, s: f64) -> Dd {
        Dd { hi: self.hi * s, lo: self.lo * s }
    }
}

/// `2^k` for `k` in f64's normal range, exactly.
#[inline(always)]
fn pow2(k: i64) -> f64 {
    debug_assert!((-1022..=1023).contains(&k));
    f64::from_bits(((k + 1023) as u64) << 52)
}

/// A complex double-double with one shared 64-bit binary exponent: `(re + i·im)·2^e`, the larger
/// part's high word in `[1, 2)`, or zero. Its range is that of the exponent, so a reference dip of
/// 2^-1656 (or 2^-100000) is held to ~106 bits like any other value.
#[derive(Clone, Copy, Debug)]
pub struct Cdx {
    pub re: Dd,
    pub im: Dd,
    pub e: i64,
}

const CDX_ZERO_E: i64 = i64::MIN / 4;

impl Cdx {
    pub const ZERO: Cdx = Cdx { re: Dd::ZERO, im: Dd::ZERO, e: CDX_ZERO_E };

    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        self.e == CDX_ZERO_E
    }

    #[inline(always)]
    fn norm(re: Dd, im: Dd, e: i64) -> Cdx {
        let m = re.hi.abs().max(im.hi.abs());
        if m == 0.0 {
            return Cdx::ZERO;
        }
        let bits = ((m.to_bits() >> 52) & 0x7ff) as i64;
        if bits == 0 {
            // A subnormal leading word (a cancellation of ~1000 bits): lift it by an exact 2^600.
            let s = pow2(600);
            return Cdx::norm(re.scale(s), im.scale(s), e - 600);
        }
        let k = bits - 1023;
        if k == 0 {
            return Cdx { re, im, e };
        }
        if k > 1022 {
            // Never from normalized operands (their products stay below 16); exact in two steps.
            let s = pow2(-512);
            return Cdx::norm(re.scale(s), im.scale(s), e + 512);
        }
        let s = pow2(-k);
        Cdx { re: re.scale(s), im: im.scale(s), e: e + k }
    }

    #[inline(always)]
    pub fn mul(self, o: Cdx) -> Cdx {
        if self.is_zero() || o.is_zero() {
            return Cdx::ZERO;
        }
        let re = self.re.mul(o.re).sub(self.im.mul(o.im));
        let im = self.re.mul(o.im).add(self.im.mul(o.re));
        Cdx::norm(re, im, self.e + o.e)
    }

    #[inline(always)]
    pub fn sqr(self) -> Cdx {
        if self.is_zero() {
            return Cdx::ZERO;
        }
        let re = self.re.mul(self.re).sub(self.im.mul(self.im));
        let im = self.re.mul(self.im).scale(2.0);
        Cdx::norm(re, im, self.e + self.e)
    }

    #[inline(always)]
    pub fn twice(self) -> Cdx {
        if self.is_zero() {
            return self;
        }
        Cdx { e: self.e + 1, ..self }
    }

    #[inline(always)]
    pub fn add(self, o: Cdx) -> Cdx {
        if self.is_zero() {
            return o;
        }
        if o.is_zero() {
            return self;
        }
        let (hi, lo) = if self.e >= o.e { (self, o) } else { (o, self) };
        let de = hi.e - lo.e;
        if de > 110 {
            return hi; // below hi's last bit
        }
        let s = pow2(-de);
        Cdx::norm(hi.re.add(lo.re.scale(s)), hi.im.add(lo.im.scale(s)), hi.e)
    }

    /// `|self|²` of the leading words, relative to `2^(2e)`: in `[1, 8)`.
    #[inline(always)]
    fn m2(&self) -> f64 {
        self.re.hi * self.re.hi + self.im.hi * self.im.hi
    }

    /// `|self| < |o|` (zero is smaller than anything non-zero).
    #[inline(always)]
    pub fn mag_lt(&self, o: &Cdx) -> bool {
        if o.is_zero() {
            return false;
        }
        if self.is_zero() {
            return true;
        }
        let d = self.e - o.e;
        if d < -2 {
            return true;
        }
        if d > 2 {
            return false;
        }
        self.m2() * pow2(2 * d) < o.m2()
    }

    /// `|self|² > 4`: past Fractal Zoomer's radius, where the f64 continuation takes over.
    #[inline(always)]
    pub fn past_r2(&self) -> bool {
        if self.is_zero() || self.e <= -1 {
            return false; // |z| < 2·sqrt(2)·2^-1 < 2
        }
        if self.e >= 2 {
            return true; // |z| >= 4
        }
        self.m2() * pow2(2 * self.e) > 4.0
    }

    /// The value as f64s (only where its exponent is small; the continuation past radius 2).
    pub fn to_f64s(&self) -> (f64, f64) {
        if self.is_zero() {
            return (0.0, 0.0);
        }
        let s = pow2(self.e.clamp(-1022, 1023));
        ((self.re.hi + self.re.lo) * s, (self.im.hi + self.im.lo) * s)
    }

    /// From two arbitrary-precision values, to ~106 bits of the larger.
    pub fn from_bf(x: &BigFloat, y: &BigFloat) -> Cdx {
        let ex = if x.is_zero() { None } else { x.exponent() };
        let ey = if y.is_zero() { None } else { y.exponent() };
        let e = match (ex, ey) {
            (None, None) => return Cdx::ZERO,
            (a, b) => a.unwrap_or(i32::MIN).max(b.unwrap_or(i32::MIN)),
        };
        // Each part scaled by 2^-e (exact), then split into a double and the double of the rest.
        let part = |v: &BigFloat, ev: Option<i32>| -> Dd {
            let Some(ev) = ev else { return Dd::ZERO };
            let d = i64::from(ev) - i64::from(e);
            if d < -1000 {
                return Dd::ZERO; // ~900 bits below the larger part
            }
            let mut s = v.clone();
            s.set_exponent((d) as i32);
            let p = s.mantissa_max_bit_len().unwrap_or(128).max(128);
            let hi = to_f64(&s);
            let rest = s.sub(&BigFloat::from_f64(hi, p), p, RM);
            Dd { hi, lo: to_f64(&rest) }.add(Dd::ZERO)
        };
        Cdx::norm(part(x, ex), part(y, ey), i64::from(e))
    }
}

/// A step whose `z = Z + δz` comes out this many bits below the larger of the two has lost them to
/// cancellation; the pixel goes to [`truth_naive`] rather than continue on what is left.
pub const TRUTH_CANCEL_BITS: i64 = 40;

/// The reference orbit's samples, iterated until `|Z|² > 1e12` or `max_iter`.
pub struct TruthReference {
    pub z: Vec<Cdx>,
    pub escaped: bool,
    pub cx: BigFloat,
    pub cy: BigFloat,
}

/// Iterate `z → z² + c` from 0 in `p`-bit arbitrary precision, keeping every sample as a [`Cdx`].
/// Its own loop, not the reference builder's: the truth must not share the code it judges.
pub fn truth_reference(cx: &BigFloat, cy: &BigFloat, max_iter: u32, p: usize) -> TruthReference {
    let mut zx = bf(0.0, p);
    let mut zy = bf(0.0, p);
    let mut z = Vec::with_capacity(max_iter as usize / 4 + 2);
    z.push(Cdx::ZERO);
    let two = bf(2.0, p);
    let mut escaped = false;
    for _ in 0..max_iter {
        let x2 = zx.mul(&zx, p, RM);
        let y2 = zy.mul(&zy, p, RM);
        let nzy = zx.mul(&zy, p, RM).mul(&two, p, RM).add(cy, p, RM);
        zx = x2.sub(&y2, p, RM).add(cx, p, RM);
        zy = nzy;
        z.push(Cdx::from_bf(&zx, &zy));
        let (x, y) = (to_f64(&zx), to_f64(&zy));
        if x * x + y * y > 1.0e12 {
            escaped = true;
            break;
        }
    }
    TruthReference { z, escaped, cx: cx.clone(), cy: cy.clone() }
}

/// A pixel's escape counts: the first iteration `n` with `|z_n| > R` for R = 2, 256 and 4096
/// (-1: no escape within the cap), and the smooth count at 256 as Fractadyne computes it,
/// `n + 1 - log2(log2 |z_n|)`. The two outer radii are reached in f64 past radius 2 (a handful
/// of steps of an orbit already escaping), so they may run a few iterations past the cap.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TruthDwell {
    pub n2: i64,
    pub n256: i64,
    pub n4096: i64,
    pub smooth256: f64,
}

impl TruthDwell {
    pub const INTERIOR: TruthDwell = TruthDwell { n2: -1, n256: -1, n4096: -1, smooth256: -1.0 };
}

/// From the first sample past radius 2 (iteration `n`), on to 256 and 4096 in f64.
fn finish(z: (f64, f64), c: (f64, f64), n: i64) -> TruthDwell {
    let (mut x, mut y) = z;
    let mut n_at = n;
    let mut m2 = x * x + y * y;
    let step = |x: f64, y: f64| (x * x - y * y + c.0, 2.0 * x * y + c.1);
    let mut guard = 0;
    while m2 <= 65536.0 && guard < 64 {
        (x, y) = step(x, y);
        m2 = x * x + y * y;
        n_at += 1;
        guard += 1;
    }
    let n256 = n_at;
    let smooth256 = n256 as f64 + 1.0 - (0.5 * m2.log2()).log2();
    while m2 <= 4096.0 * 4096.0 && guard < 64 {
        (x, y) = step(x, y);
        m2 = x * x + y * y;
        n_at += 1;
        guard += 1;
    }
    TruthDwell { n2: n, n256, n4096: n_at, smooth256 }
}

/// One pixel by full-step perturbation against `r`: `dc = c - c_ref` as a [`Cdx`], `c` the pixel's
/// point as f64s (for the continuation past radius 2). `None`: a step lost more than
/// [`TRUTH_CANCEL_BITS`] to cancellation, so the answer is not to be trusted ([`truth_naive`]).
pub fn truth_pixel(r: &TruthReference, dc: Cdx, c: (f64, f64), max_iter: u32) -> Option<TruthDwell> {
    let len = r.z.len();
    let mut dz = Cdx::ZERO;
    let mut ref_n = 0usize;
    let mut iter = 0u32;
    loop {
        if iter >= max_iter {
            return Some(TruthDwell::INTERIOR);
        }
        // δz' = 2·Z·δz + δz² + δc
        dz = r.z[ref_n].mul(dz).twice().add(dz.sqr()).add(dc);
        ref_n += 1;
        iter += 1;
        let zn = r.z[ref_n];
        let z = zn.add(dz);
        if z.past_r2() {
            return Some(finish(z.to_f64s(), c, i64::from(iter)));
        }
        if !z.is_zero() && z.e < zn.e.max(dz.e) - TRUTH_CANCEL_BITS {
            return None;
        }
        if z.mag_lt(&dz) || ref_n + 1 >= len {
            dz = z;
            ref_n = 0;
        }
    }
}

/// A pixel's counts at `(px, py)`: [`truth_pixel`] against `r`, or [`truth_naive`] where it will
/// not vouch for its answer (`true` in the second place: the plain iteration answered).
pub fn truth_at(r: &TruthReference, px: &BigFloat, py: &BigFloat, max_iter: u32, p: usize) -> (TruthDwell, bool) {
    let dc = Cdx::from_bf(&px.sub(&r.cx, p, RM), &py.sub(&r.cy, p, RM));
    match truth_pixel(r, dc, (to_f64(px), to_f64(py)), max_iter) {
        Some(d) => (d, false),
        None => (truth_naive(px, py, max_iter, p), true),
    }
}

/// The same counts by plain `p`-bit arbitrary-precision iteration (no reference, no perturbation):
/// the fallback for a pixel [`truth_pixel`] will not vouch for, and the check on it.
pub fn truth_naive(cx: &BigFloat, cy: &BigFloat, max_iter: u32, p: usize) -> TruthDwell {
    let mut zx = bf(0.0, p);
    let mut zy = bf(0.0, p);
    let two = bf(2.0, p);
    let c = (to_f64(cx), to_f64(cy));
    for n in 1..=max_iter {
        let x2 = zx.mul(&zx, p, RM);
        let y2 = zy.mul(&zy, p, RM);
        let nzy = zx.mul(&zy, p, RM).mul(&two, p, RM).add(cy, p, RM);
        zx = x2.sub(&y2, p, RM).add(cx, p, RM);
        zy = nzy;
        let (x, y) = (to_f64(&zx), to_f64(&zy));
        if x * x + y * y > 4.0 {
            return finish((x, y), c, i64::from(n));
        }
    }
    TruthDwell::INTERIOR
}

#[cfg(test)]
#[path = "../../../tests/unit/harness/reference/truth/tests.rs"]
mod tests;
