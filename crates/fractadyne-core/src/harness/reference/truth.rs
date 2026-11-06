//! THE REFERENCE-GRADE DWELL (`--truth`): a pixel's escape count with none of the approximations
//! a renderer makes, at every pixel of a view, so a render can be checked against something it
//! did not compute itself. Attached to `reference` (`#[path]`) and re-exported, as the oracle is.
//!
//! WHY (user request, 2026-11-03: golden references "created with no approximations or
//! optimizations that have the least chance of error", every pixel, filaments included). That day
//! twelve test views drew a flat disc or one flat color for every reference pick, and the pick
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
//! Each route follows the orbit in its own precision all the way out to radius 4096.
//! ⛔Both routes used to hand the orbit to f64 at radius 2, on the theory that an orbit past 2
//! is a handful of steps from 256. Near c = -2 it is not: the orbit hovers just outside |z| = 2
//! (the fixed point 2 sits ON that circle) for as many steps as the zoom is deep, and f64 cannot
//! follow it. The tip ladder found it (2026-11-04): its 4x self-similarity (an oracle sharing
//! nothing with this engine) showed smooth counts off by up to 0.24 and whole counts flipped, and
//! the plain-iteration check, which shared the f64 step, agreed with the wrong answers.
//! ⛔And the radius tests themselves are exact where f64 cannot decide: every point left of -2 has
//! |c| > 2 by as little as the zoom is deep (1e-100 at 1e100x), and the orbit then hovers on the
//! circle |z| = 2, so "is |z| past 2?" taken in f64 sees 2.0 and says no. At 1e100x the two routes
//! disagreed by one at radius 2 on 17 of 200 checked pixels (2026-11-04). Within 1e-9 of a radius
//! each route now decides in its own precision: [`truth_naive`] in arbitrary precision,
//! [`truth_direct`] in double-double, and [`truth_pixel`] from the reference's own exact
//! `|Z|² - 4` (kept per sample) plus the delta's terms, or not at all ([`truth_naive`] then).

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

    #[inline(always)]
    fn conj(self) -> Cdx {
        Cdx { im: self.im.neg(), ..self }
    }

    /// The real part, as a [`Cdx`].
    #[inline(always)]
    fn real(self) -> Cdx {
        if self.is_zero() {
            return self;
        }
        Cdx::norm(self.re, Dd::ZERO, self.e)
    }

    /// `|self|² > r2`, to double-double precision (`r2` a radius², 4 to 2^24).
    fn mag2_gt(&self, r2: f64) -> bool {
        if self.is_zero() || self.e < -500 {
            return false;
        }
        if self.e > 500 {
            return true;
        }
        let m = self.re.mul(self.re).add(self.im.mul(self.im)).scale(pow2(2 * self.e));
        m.hi > r2 || (m.hi == r2 && m.lo > 0.0)
    }

    /// `|self|²` as an f64, for the escape radii and the smooth count: 0 below 2^-500, infinite
    /// above 2^500 (the radii are 2 to 4096).
    #[inline(always)]
    pub fn mag2(&self) -> f64 {
        if self.is_zero() || self.e < -500 {
            return 0.0;
        }
        if self.e > 500 {
            return f64::INFINITY;
        }
        let (x, y) = (self.re.hi + self.re.lo, self.im.hi + self.im.lo);
        (x * x + y * y) * pow2(2 * self.e)
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

/// The reference orbit's samples, iterated until `|Z|² > 1e12` or `max_iter`, and each sample's
/// `|Z|² - 4` taken in the orbit's own precision (real, as a [`Cdx`]): the exact radius-2 test.
pub struct TruthReference {
    pub z: Vec<Cdx>,
    pub m4: Vec<Cdx>,
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
    let mut m4 = Vec::with_capacity(max_iter as usize / 4 + 2);
    let (zero, two, four) = (bf(0.0, p), bf(2.0, p), bf(4.0, p));
    z.push(Cdx::ZERO);
    m4.push(Cdx::from_bf(&bf(-4.0, p), &zero));
    let mut escaped = false;
    for _ in 0..max_iter {
        let x2 = zx.mul(&zx, p, RM);
        let y2 = zy.mul(&zy, p, RM);
        let nzy = zx.mul(&zy, p, RM).mul(&two, p, RM).add(cy, p, RM);
        zx = x2.sub(&y2, p, RM).add(cx, p, RM);
        zy = nzy;
        z.push(Cdx::from_bf(&zx, &zy));
        let m = zx.mul(&zx, p, RM).add(&zy.mul(&zy, p, RM), p, RM).sub(&four, p, RM);
        m4.push(Cdx::from_bf(&m, &zero));
        let (x, y) = (to_f64(&zx), to_f64(&zy));
        if x * x + y * y > 1.0e12 {
            escaped = true;
            break;
        }
    }
    TruthReference { z, m4, escaped, cx: cx.clone(), cy: cy.clone() }
}

/// A pixel's escape counts: the first iteration `n` with `|z_n| > R` for R = 2, 256 and 4096
/// (-1: that radius not passed within the cap), and the smooth count at 256 as Fractadyne computes
/// it, `n + 1 - log2(log2 |z_n|)` (-1 with `n256`).
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

/// Within this (relative) of a radius², f64 does not decide whether `|z|` is past it.
const RADIUS_BAND: f64 = 1e-9;

/// The three radii as an orbit passes them, fed `|z_n|²` at each iteration `n`.
struct Radii(TruthDwell);

impl Radii {
    fn new() -> Radii {
        Radii(TruthDwell::INTERIOR)
    }

    /// Record iteration `n` from `|z_n|²` as an f64; within [`RADIUS_BAND`] of a radius² the f64
    /// cannot decide, and `exact(R²)` does, in the route's own precision (None: it cannot either,
    /// and nor does this). `Some(true)` once past the last radius.
    #[inline(always)]
    fn see(&mut self, n: u32, m2: f64, exact: impl Fn(f64) -> Option<bool>) -> Option<bool> {
        let past = |r2: f64| if (m2 - r2).abs() > RADIUS_BAND * r2 { Some(m2 > r2) } else { exact(r2) };
        let d = &mut self.0;
        if d.n2 < 0 && past(4.0)? {
            d.n2 = i64::from(n);
        }
        if d.n256 < 0 && past(65536.0)? {
            d.n256 = i64::from(n);
            d.smooth256 = f64::from(n) + 1.0 - (0.5 * m2.log2()).log2();
        }
        if past(4096.0 * 4096.0)? {
            d.n4096 = i64::from(n);
            return Some(true);
        }
        Some(false)
    }
}

/// One pixel by full-step perturbation against `r`, `dc = c - c_ref` as a [`Cdx`]. `None`: a step
/// lost more than [`TRUTH_CANCEL_BITS`] to cancellation, so the answer is not to be trusted
/// ([`truth_naive`]).
pub fn truth_pixel(r: &TruthReference, dc: Cdx, max_iter: u32) -> Option<TruthDwell> {
    let len = r.z.len();
    let mut dz = Cdx::ZERO;
    let mut ref_n = 0usize;
    let mut radii = Radii::new();
    for iter in 1..=max_iter {
        // δz' = 2·Z·δz + δz² + δc
        dz = r.z[ref_n].mul(dz).twice().add(dz.sqr()).add(dc);
        ref_n += 1;
        let zn = r.z[ref_n];
        let z = zn.add(dz);
        if !z.is_zero() && z.e < zn.e.max(dz.e) - TRUTH_CANCEL_BITS {
            return None;
        }
        // Radius 2 exactly: |z|² - 4 = (|Z|² - 4) + 2·Re(conj(Z)·δz) + |δz|², each term to ~106
        // bits of itself; a sum that cancels below 2^-90 of the largest term decides nothing.
        let m4 = r.m4[ref_n];
        let exact = |r2: f64| -> Option<bool> {
            if r2 != 4.0 {
                return Some(z.mag2_gt(r2));
            }
            let (a, b, c) = (m4, zn.conj().mul(dz).real().twice(), dz.conj().mul(dz).real());
            let s = a.add(b).add(c);
            if s.is_zero() || s.e < a.e.max(b.e).max(c.e) - 90 {
                return None;
            }
            Some(s.re.hi > 0.0)
        };
        match radii.see(iter, z.mag2(), exact) {
            None => return None,
            Some(true) => return Some(radii.0),
            Some(false) => {}
        }
        if z.mag_lt(&dz) || ref_n + 1 >= len {
            dz = z;
            ref_n = 0;
        }
    }
    Some(radii.0)
}

/// A pixel's counts at `(px, py)`: [`truth_pixel`] against `r`, or [`truth_naive`] where it will
/// not vouch for its answer (`true` in the second place: the plain iteration answered).
pub fn truth_at(r: &TruthReference, px: &BigFloat, py: &BigFloat, max_iter: u32, p: usize) -> (TruthDwell, bool) {
    let dc = Cdx::from_bf(&px.sub(&r.cx, p, RM), &py.sub(&r.cy, p, RM));
    match truth_pixel(r, dc, max_iter) {
        Some(d) => (d, false),
        None => (truth_naive(px, py, max_iter, p), true),
    }
}

/// The same counts by plain `p`-bit arbitrary-precision iteration (no reference, no perturbation):
/// the fallback for a pixel [`truth_pixel`] will not vouch for, and the check on it.
pub fn truth_naive(cx: &BigFloat, cy: &BigFloat, max_iter: u32, p: usize) -> TruthDwell {
    let mut zx = bf(0.0, p);
    let mut zy = bf(0.0, p);
    let (zero, two) = (bf(0.0, p), bf(2.0, p));
    let mut radii = Radii::new();
    for n in 1..=max_iter {
        let x2 = zx.mul(&zx, p, RM);
        let y2 = zy.mul(&zy, p, RM);
        let nzy = zx.mul(&zy, p, RM).mul(&two, p, RM).add(cy, p, RM);
        zx = x2.sub(&y2, p, RM).add(cx, p, RM);
        zy = nzy;
        let (x, y) = (to_f64(&zx), to_f64(&zy));
        // In arbitrary precision near a radius (the sign survives any exponent through a Cdx).
        let exact = |r2: f64| -> Option<bool> {
            let m = zx.mul(&zx, p, RM).add(&zy.mul(&zy, p, RM), p, RM).sub(&bf(r2, p), p, RM);
            let s = Cdx::from_bf(&m, &zero);
            Some(!s.is_zero() && s.re.hi > 0.0)
        };
        if radii.see(n, x * x + y * y, exact) == Some(true) {
            break;
        }
    }
    radii.0
}

/// The deepest pixel step [`truth_direct`] is trusted with: below it the point itself would keep
/// too few bits under double-double's ~106 (2^-80 leaves 26, against a stencil 2^-10 px away).
pub const TRUTH_DIRECT_MIN_STEP: f64 = 8.271_806_125_530_277e-25; // 2^-80

/// Plain iteration in double-double (~106 bits; no reference, no perturbation), for a SHALLOW
/// view (`--truth-direct`) whose escape counts run too long for a reference: every reference
/// sample is kept (40 bytes), so the π points' 31-million-iteration orbits near c = -3/4 and 1/4
/// would need over a gigabyte per candidate. Shares no code with the shader, the reference
/// builder or [`truth_naive`] (the check on it) beyond the radii.
pub fn truth_direct(cx: &BigFloat, cy: &BigFloat, max_iter: u32) -> TruthDwell {
    let c = Cdx::from_bf(cx, cy);
    let (mut cr, mut ci) = (c.re, c.im);
    if c.is_zero() {
        (cr, ci) = (Dd::ZERO, Dd::ZERO);
    } else {
        let mut e = c.e;
        while e < -1000 {
            // Below 2^-1000 a part is far under the orbit's last bit; it may flush to zero.
            let s = pow2(-1000);
            (cr, ci) = (cr.scale(s), ci.scale(s));
            e += 1000;
        }
        let s = pow2(e.min(1023));
        (cr, ci) = (cr.scale(s), ci.scale(s));
    }
    let (mut x, mut y) = (Dd::ZERO, Dd::ZERO);
    let mut radii = Radii::new();
    for n in 1..=max_iter {
        let x2 = x.mul(x);
        let y2 = y.mul(y);
        let xy = x.mul(y);
        y = xy.scale(2.0).add(ci);
        x = x2.sub(y2).add(cr);
        let (xf, yf) = (x.hi + x.lo, y.hi + y.lo);
        let exact = |r2: f64| -> Option<bool> {
            let m = x.mul(x).add(y.mul(y));
            Some(m.hi > r2 || (m.hi == r2 && m.lo > 0.0))
        };
        if radii.see(n, xf * xf + yf * yf, exact) == Some(true) {
            break;
        }
    }
    radii.0
}

#[cfg(test)]
#[path = "../../../tests/unit/harness/reference/truth/tests.rs"]
mod tests;
