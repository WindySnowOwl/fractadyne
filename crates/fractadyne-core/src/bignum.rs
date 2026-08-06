//! Arbitrary-precision (`astro_float::BigFloat`) helpers: fast `f64` conversion, decimal / KFR
//! parsing, precision-for-zoom, and the small bignum arithmetic used by the reference orbit and
//! viewport. The numeric leaf of the crate — depends only on `astro_float`.

use astro_float::{BigFloat, Consts, Radix, RoundingMode, Sign};

pub(crate) const RM: RoundingMode = RoundingMode::None;

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
    let mantissa = *digits.last().unwrap(); // top 64 bits (normalized MSW)
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

/// `log₂|v|`, valid across the whole `BigFloat` exponent range — where [`to_f64`] would
/// saturate to `0` or `∞`. `-∞` for zero.
///
/// astro-float stores `v = mantissa · 2^exponent` with the mantissa normalized to `[0.5, 1)`,
/// so the most-significant word alone fixes the fractional part of the log to well past `f64`
/// resolution. Needed wherever a quantity is astronomically large or small by construction —
/// the minibrot size estimate, whose value at depth is ~`2^-3322` and beyond.
pub fn log2_abs(v: &BigFloat) -> f64 {
    let (Some(e), Some(d)) = (v.exponent(), v.mantissa_digits()) else {
        return f64::NEG_INFINITY;
    };
    let Some(&msw) = d.last() else {
        return f64::NEG_INFINITY;
    };
    if msw == 0 {
        return f64::NEG_INFINITY;
    }
    let word_bits = (core::mem::size_of_val(&msw) * 8) as i32;
    e as f64 + ((msw as f64) / 2f64.powi(word_bits)).log2()
}

/// `arg(x + iy)` in radians, across the whole exponent range. Uses the ratio `y/x` (safe at any
/// scale — `BigFloat` division can't overflow the way squaring the components could), then fixes
/// the quadrant from the signs.
pub fn arg_bf(x: &BigFloat, y: &BigFloat, p: usize) -> f64 {
    use core::f64::consts::{FRAC_PI_2, PI};
    let y_neg = matches!(y.sign(), Some(Sign::Neg));
    if x.is_zero() {
        return if y_neg { -FRAC_PI_2 } else { FRAC_PI_2 };
    }
    // |y/x| may overflow f64 → atan(±∞) = ±π/2, which is the right answer anyway.
    let t = to_f64(&y.div(x, p, RM)).atan();
    if matches!(x.sign(), Some(Sign::Neg)) {
        if y_neg {
            t - PI
        } else {
            t + PI
        }
    } else {
        t
    }
}

/// Parse a decimal string back to a `BigFloat` (round-trips `to_decimal_string`), or an exact
/// rational expression like `-3/4`. Rejects non-finite results (NaN / ±∞) so a malformed or
/// out-of-range coordinate can't slip through — callers treat `None` as "invalid input".
///
/// Uses the default working precision for rational arithmetic; a caller that knows the target
/// zoom should use [`parse_bf_prec`] so an inexact rational (e.g. `37/100`) carries enough
/// digits for the depth it's about to be viewed at.
pub fn parse_bf(s: &str) -> Option<BigFloat> {
    parse_bf_prec(s, 0)
}

/// Is the whole string a well-formed decimal literal (`[+-]?digits[.digits][eE[+-]digits]`)?
///
/// Two jobs. It routes plain coordinates down the verbatim `FromStr` path, which preserves every
/// digit of a pasted deep-zoom center (see `deep_roundtrip_bits`) — and it is the *validation*
/// `FromStr` doesn't do: astro-float happily parses `"1 2"` as `1`, silently dropping the rest of
/// what the user typed. A coordinate must never be half-read.
fn is_decimal_literal(t: &str) -> bool {
    let b = t.as_bytes();
    let mut i = 0;
    if matches!(b.get(i), Some(b'+') | Some(b'-')) {
        i += 1;
    }
    let mut saw_digit = false;
    while matches!(b.get(i), Some(c) if c.is_ascii_digit()) {
        saw_digit = true;
        i += 1;
    }
    if b.get(i) == Some(&b'.') {
        i += 1;
        while matches!(b.get(i), Some(c) if c.is_ascii_digit()) {
            saw_digit = true;
            i += 1;
        }
    }
    if !saw_digit {
        return false;
    }
    if matches!(b.get(i), Some(c) if c | 32 == b'e') {
        i += 1;
        if matches!(b.get(i), Some(b'+') | Some(b'-')) {
            i += 1;
        }
        let ds = i;
        while matches!(b.get(i), Some(c) if c.is_ascii_digit()) {
            i += 1;
        }
        if i == ds {
            return false; // "1e" — an exponent marker with no exponent
        }
    }
    i == b.len()
}

/// Working precision for expression arithmetic: enough bits to hold every digit typed, with
/// headroom for division rounding, but never below `floor` (a caller passing the target zoom's
/// precision keeps an inexact rational usable at that depth).
fn expr_precision(s: &str, floor: usize) -> usize {
    let digits = s.bytes().filter(u8::is_ascii_digit).count();
    // ~3.33 bits per decimal digit; ×4 leaves comfortable headroom, +64 guard bits.
    floor.max(64).max(digits.saturating_mul(4).saturating_add(64))
}

/// Parse a **real** coordinate: a plain decimal, or an exact rational expression (`-3/4`,
/// `(1+2)/8`). Rational arithmetic runs at `min_prec` bits or higher (see [`expr_precision`]).
/// An expression with a nonzero imaginary part is rejected — use [`parse_complex_prec`] for
/// those.
pub fn parse_bf_prec(s: &str, min_prec: usize) -> Option<BigFloat> {
    let t = s.trim();
    if is_decimal_literal(t) {
        let mut cc = Consts::new().ok()?;
        return parse_literal(t, min_prec, &mut cc);
    }
    let (re, im) = parse_complex_prec(t, min_prec)?;
    im.is_zero().then_some(re)
}

/// Parse a **complex** coordinate expression into `(re, im)`: decimals, an `i` suffix, the four
/// arithmetic operators and parentheses. Written for exact-rational landmark entry — the
/// canonical case is `(37+16i)/100`, a point that is *exactly* on ∂M and cannot be typed as a
/// terminating decimal. Arithmetic runs at `min_prec` bits or higher.
///
/// Grammar (total, no recursion depth beyond the input's own nesting):
/// `sum := product (('+'|'-') product)*` · `product := unary (('*'|'/') unary)*` ·
/// `unary := ('+'|'-')* atom` · `atom := '(' sum ')' | 'i' | number ['i']`
pub fn parse_complex_prec(s: &str, min_prec: usize) -> Option<(BigFloat, BigFloat)> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let p = expr_precision(t, min_prec);
    let mut e = Expr { b: t.as_bytes(), i: 0, p, depth: 0, cc: Consts::new().ok()? };
    let v = e.sum()?;
    e.ws();
    if e.i != e.b.len() {
        return None; // trailing garbage — never silently ignore part of a coordinate
    }
    let bad = |b: &BigFloat| b.is_nan() || b.is_inf();
    (!bad(&v.0) && !bad(&v.1)).then_some(v)
}

/// A complex value mid-evaluation.
type Cx = (BigFloat, BigFloat);

fn cx_zero(p: usize) -> BigFloat {
    BigFloat::from_f64(0.0, p)
}
fn cx_add(a: &Cx, b: &Cx, p: usize) -> Cx {
    (a.0.add(&b.0, p, RM), a.1.add(&b.1, p, RM))
}
fn cx_sub(a: &Cx, b: &Cx, p: usize) -> Cx {
    (a.0.sub(&b.0, p, RM), a.1.sub(&b.1, p, RM))
}
fn cx_neg(a: &Cx, p: usize) -> Cx {
    (cx_zero(p).sub(&a.0, p, RM), cx_zero(p).sub(&a.1, p, RM))
}
fn cx_mul(a: &Cx, b: &Cx, p: usize) -> Cx {
    (
        a.0.mul(&b.0, p, RM).sub(&a.1.mul(&b.1, p, RM), p, RM),
        a.0.mul(&b.1, p, RM).add(&a.1.mul(&b.0, p, RM), p, RM),
    )
}
/// `(a+bi)/(c+di) = ((ac+bd) + (bc−ad)i) / (c²+d²)`. `None` on a zero (or non-finite) divisor.
fn cx_div(a: &Cx, b: &Cx, p: usize) -> Option<Cx> {
    let den = b.0.mul(&b.0, p, RM).add(&b.1.mul(&b.1, p, RM), p, RM);
    if den.is_zero() || den.is_nan() || den.is_inf() {
        return None;
    }
    let re = a.0.mul(&b.0, p, RM).add(&a.1.mul(&b.1, p, RM), p, RM);
    let im = a.1.mul(&b.0, p, RM).sub(&a.0.mul(&b.1, p, RM), p, RM);
    Some((re.div(&den, p, RM), im.div(&den, p, RM)))
}

/// Parse one decimal literal at **at least** `min_prec` bits.
///
/// astro-float's `FromStr` sizes precision from the input's own digit count, so `0.37` lands at
/// the minimum width — fine for round-tripping a pasted coordinate, useless at 1e50× where that
/// same point needs hundreds of bits. When the caller's floor exceeds the literal's natural
/// width we re-parse from the decimal text at the floor, which yields correctly-rounded extra
/// digits (widening the low-precision value would only pad it with zeros).
fn parse_literal(lit: &str, min_prec: usize, cc: &mut Consts) -> Option<BigFloat> {
    let finite = |b: &BigFloat| !b.is_nan() && !b.is_inf();
    let auto = lit.parse::<BigFloat>().ok().filter(finite)?;
    let natural = auto.mantissa_digits().map(|d| d.len() * 64).unwrap_or(0);
    if min_prec <= natural {
        return Some(auto);
    }
    let v = BigFloat::parse(lit, Radix::Dec, min_prec, RoundingMode::ToEven, cc);
    finite(&v).then_some(v)
}

/// Recursive-descent state. `depth` bounds nesting so a pasted `((((…` can't blow the stack:
/// coordinate entry is untrusted input (same threat model as the `.kfr` parser).
struct Expr<'a> {
    b: &'a [u8],
    i: usize,
    p: usize,
    depth: u32,
    cc: Consts,
}

const EXPR_MAX_DEPTH: u32 = 32;

impl Expr<'_> {
    fn ws(&mut self) {
        while matches!(self.b.get(self.i), Some(c) if c.is_ascii_whitespace()) {
            self.i += 1;
        }
    }
    fn peek(&mut self) -> Option<u8> {
        self.ws();
        self.b.get(self.i).copied()
    }

    fn sum(&mut self) -> Option<Cx> {
        let mut acc = self.product()?;
        loop {
            match self.peek() {
                Some(b'+') => {
                    self.i += 1;
                    let r = self.product()?;
                    acc = cx_add(&acc, &r, self.p);
                }
                Some(b'-') => {
                    self.i += 1;
                    let r = self.product()?;
                    acc = cx_sub(&acc, &r, self.p);
                }
                _ => return Some(acc),
            }
        }
    }

    fn product(&mut self) -> Option<Cx> {
        let mut acc = self.unary()?;
        loop {
            match self.peek() {
                Some(b'*') => {
                    self.i += 1;
                    let r = self.unary()?;
                    acc = cx_mul(&acc, &r, self.p);
                }
                Some(b'/') => {
                    self.i += 1;
                    let r = self.unary()?;
                    acc = cx_div(&acc, &r, self.p)?;
                }
                _ => return Some(acc),
            }
        }
    }

    fn unary(&mut self) -> Option<Cx> {
        match self.peek() {
            Some(b'-') => {
                self.i += 1;
                let v = self.unary()?;
                Some(cx_neg(&v, self.p))
            }
            Some(b'+') => {
                self.i += 1;
                self.unary()
            }
            _ => self.atom(),
        }
    }

    fn atom(&mut self) -> Option<Cx> {
        match self.peek()? {
            b'(' => {
                if self.depth >= EXPR_MAX_DEPTH {
                    return None;
                }
                self.i += 1;
                self.depth += 1;
                let v = self.sum()?;
                self.depth -= 1;
                self.ws();
                if self.b.get(self.i) != Some(&b')') {
                    return None;
                }
                self.i += 1;
                Some(v)
            }
            b'i' | b'I' => {
                self.i += 1;
                Some((cx_zero(self.p), BigFloat::from_f64(1.0, self.p)))
            }
            _ => self.number(),
        }
    }

    /// A decimal literal (with optional fraction and exponent) and an optional `i` suffix.
    fn number(&mut self) -> Option<Cx> {
        self.ws();
        let start = self.i;
        let mut saw_digit = false;
        while matches!(self.b.get(self.i), Some(c) if c.is_ascii_digit()) {
            saw_digit = true;
            self.i += 1;
        }
        if self.b.get(self.i) == Some(&b'.') {
            self.i += 1;
            while matches!(self.b.get(self.i), Some(c) if c.is_ascii_digit()) {
                saw_digit = true;
                self.i += 1;
            }
        }
        if !saw_digit {
            return None;
        }
        // Exponent — only if it's actually well-formed, so the `e` of a stray token doesn't
        // swallow input (and `1e` alone stays a parse error rather than becoming `1`).
        if matches!(self.b.get(self.i), Some(c) if c | 32 == b'e') {
            let save = self.i;
            self.i += 1;
            if matches!(self.b.get(self.i), Some(b'+') | Some(b'-')) {
                self.i += 1;
            }
            let ds = self.i;
            while matches!(self.b.get(self.i), Some(c) if c.is_ascii_digit()) {
                self.i += 1;
            }
            if self.i == ds {
                self.i = save;
            }
        }
        let text = self.b; // copy the slice ref out so `self.cc` can be borrowed mutably below
        let lit = std::str::from_utf8(text.get(start..self.i)?).ok()?;
        let p = self.p;
        let v = parse_literal(lit, p, &mut self.cc)?;
        if matches!(self.b.get(self.i), Some(b'i') | Some(b'I')) {
            self.i += 1;
            Some((cx_zero(self.p), v))
        } else {
            Some((v, cx_zero(self.p)))
        }
    }
}

/// A location parsed from a Kalles Fraktaler `.kfr` file: full-precision center, zoom
/// (magnification), and optional iteration count.
pub struct KfrView {
    pub cx: BigFloat,
    pub cy: BigFloat,
    pub zoom: f64,
    pub iterations: Option<u32>,
}

/// Parse a positive, finite zoom from a `.kfr` `Zoom:` value; an over-range value (e.g.
/// `1E1000`) is clamped rather than rejected.
fn parse_kfr_zoom(s: &str) -> Option<f64> {
    let z: f64 = s.trim().parse().ok()?;
    if z.is_nan() || z <= 0.0 {
        return None;
    }
    Some(z.min(1.0e300)) // inf.min ⇒ clamp; matches the viewport's f64 upp range
}

/// Parse a **Kalles Fraktaler `.kfr`** location (a simple `Key: value` text format) into a
/// [`KfrView`]. **Hardened for untrusted input:** total size and line/value lengths are
/// bounded, only the `Re`/`Im`/`Zoom`/`Iterations` keys are read (everything else ignored —
/// no formulas, paths, or code), the center is validated through [`parse_bf`] (rejecting
/// non-finite), and zoom/iterations are clamped. Returns `None` unless a valid Re, Im, and
/// Zoom are present.
pub fn parse_kfr(text: &str) -> Option<KfrView> {
    if text.len() > 4_000_000 {
        return None; // refuse absurdly large inputs
    }
    let (mut re, mut im, mut zoom, mut iters) = (None, None, None, None);
    for line in text.lines().take(20_000) {
        let Some((key, val)) = line.split_once(':') else {
            continue;
        };
        let (key, val) = (key.trim(), val.trim());
        if val.len() > 100_000 {
            continue; // bounded value length (deep coords are long, but not unbounded)
        }
        if key.eq_ignore_ascii_case("Re") {
            re = Some(val);
        } else if key.eq_ignore_ascii_case("Im") {
            im = Some(val);
        } else if key.eq_ignore_ascii_case("Zoom") {
            zoom = parse_kfr_zoom(val);
        } else if key.eq_ignore_ascii_case("Iterations") {
            iters = val.parse::<u64>().ok().map(|v| v.min(1_000_000) as u32);
        }
        // every other key is ignored by design
    }
    Some(KfrView {
        cx: parse_bf(re?)?,
        cy: parse_bf(im?)?,
        zoom: zoom?,
        iterations: iters,
    })
}

/// Mantissa bits needed to position sub-pixel at the given magnification (+ guard).
///
/// Note: `mag` is `f64`, so this saturates near `1e308×` (and the viewport's `f64`
/// `units_per_pixel` is the actual render-depth ceiling). To reason about precision past
/// that — e.g. for extreme-depth *arithmetic* validation — use [`precision_for_octaves`].
pub fn precision_for_magnification(mag: f64) -> usize {
    let octaves = mag.max(1.0).log2().ceil() as usize;
    (octaves + 64).max(64)
}

/// Mantissa bits for a magnification of `2^octaves` (+ 64 guard bits). Unlike
/// [`precision_for_magnification`] this takes the octave count directly, so it stays valid
/// for magnifications far beyond `f64` range (e.g. `1e1000000×` ≈ 3.32M octaves → ~3.32M
/// bits) — used by the extreme-depth validation battery.
pub fn precision_for_octaves(octaves: u64) -> usize {
    (octaves as usize).saturating_add(64).max(64)
}

/// Leading base-2 bits of agreement between two `BigFloat`s: ≈ `−log₂(|a−b| / |b|)`.
/// Returns `p` when they match to the working precision (the difference rounds to zero).
fn agree_bits(a: &BigFloat, b: &BigFloat, p: usize) -> i64 {
    let diff = a.sub(b, p, RM);
    let ed = match diff.exponent() {
        Some(e) => e as i64,
        None => return p as i64, // difference is exactly zero → full agreement
    };
    let eb = b.exponent().map(|e| e as i64).unwrap_or(0);
    (eb - ed).max(0)
}

/// A full-mantissa interior point (inside the main cardioid, so it never escapes), seeded
/// by `√½` so every limb is populated — exercising real carries in the bignum multiply
/// rather than the sparse mantissas a dyadic point like `c = −0.5` would produce.
fn deep_test_point(p: usize) -> (BigFloat, BigFloat) {
    let s = BigFloat::from_f64(0.5, p).sqrt(p, RM); // 0.70710678… (irrational ⇒ full mantissa)
    let cx = s
        .mul(&BigFloat::from_f64(0.3, p), p, RM)
        .sub(&BigFloat::from_f64(0.5, p), p, RM); // ≈ −0.288 (interior)
    let cy = s.mul(&BigFloat::from_f64(0.01, p), p, RM); // ≈ 0.0071
    (cx, cy)
}

fn iter_zsq_c(cx: &BigFloat, cy: &BigFloat, k: u32, p: usize) -> (BigFloat, BigFloat) {
    let mut zx = BigFloat::from_f64(0.0, p);
    let mut zy = BigFloat::from_f64(0.0, p);
    for _ in 0..k {
        let x2 = zx.mul(&zx, p, RM);
        let y2 = zy.mul(&zy, p, RM);
        let nzy = double_bf(&zx.mul(&zy, p, RM)).add(cy, p, RM);
        zx = x2.sub(&y2, p, RM).add(cx, p, RM);
        zy = nzy;
    }
    (zx, zy)
}

/// Extreme-depth **precision self-consistency** probe (needs no external oracle): iterate
/// `z²+c` for `k` steps from a full-mantissa interior point at precision `p`, and again at
/// `p + guard` bits, then return how many leading base-2 bits of the result agree.
///
/// This is the standard precision-doubling validation technique: if the `p`-bit answer is
/// stable under a precision increase it is almost certainly correct. Sound `p`-bit
/// arithmetic gives agreement ≈ `p − log₂(k)`; a precision-propagation or arithmetic bug at
/// that bit-width collapses it. Feasible to any depth (cost ∝ `k · M(p)` with FFT multiply),
/// unlike a per-pixel dwell oracle.
pub fn deep_consistency_bits(p: usize, guard: usize, k: u32) -> i64 {
    let pg = p + guard;
    let (cx, cy) = deep_test_point(pg);
    let lo = iter_zsq_c(&cx, &cy, k, p);
    let hi = iter_zsq_c(&cx, &cy, k, pg);
    agree_bits(&lo.0, &hi.0, pg).min(agree_bits(&lo.1, &hi.1, pg))
}

/// Round-trip a full-mantissa coordinate through decimal at precision `p`
/// (`to_decimal_string` → `parse_bf`) and return the bits of agreement — validates the
/// persisted-coordinate I/O path (the deep-zoom save/restore format) at extreme precision.
pub fn deep_roundtrip_bits(p: usize) -> i64 {
    let (cx, _) = deep_test_point(p);
    match parse_bf(&to_decimal_string(&cx)) {
        Some(back) => agree_bits(&cx, &back, p),
        None => -1,
    }
}

pub(crate) fn bf(v: f64, p: usize) -> BigFloat {
    BigFloat::from_f64(v, p)
}

/// Exact multiply-by-two via a base-2 exponent bump — far cheaper than the full bignum multiply
/// that was forming `2xy` every reference-orbit iteration. Exact (no rounding); zero stays zero.
pub(crate) fn double_bf(x: &BigFloat) -> BigFloat {
    let mut d = x.clone();
    if let Some(e) = d.exponent() {
        d.set_exponent(e + 1);
    }
    d
}

/// Multiply a bignum by a small non-negative integer via shift-and-add (doublings + adds) — much
/// cheaper than a full-precision multiply by a constant `BigFloat`. Used in the hot series-
/// approximation recurrence, whose factors (`d`, `C(d,2)`, `2·C(d,2)`, `C(d,3)`) are all small ints.
pub(crate) fn mul_u32_bf(x: &BigFloat, n: u32, p: usize) -> BigFloat {
    if n == 0 {
        return bf(0.0, p);
    }
    let mut acc: Option<BigFloat> = None;
    let mut base = x.clone(); // x · 2^k
    let mut k = n;
    loop {
        if k & 1 == 1 {
            acc = Some(match acc {
                Some(a) => a.add(&base, p, RM),
                None => base.clone(),
            });
        }
        k >>= 1;
        if k == 0 {
            break;
        }
        base = double_bf(&base);
    }
    acc.unwrap()
}

/// Linear interpolation between two `BigFloat`s at precision `p`: `a + (b − a)·t`.
pub fn lerp_bf(a: &BigFloat, b: &BigFloat, t: f64, p: usize) -> BigFloat {
    let f = bf(t, p);
    a.add(&b.sub(a, p, RM).mul(&f, p, RM), p, RM)
}
