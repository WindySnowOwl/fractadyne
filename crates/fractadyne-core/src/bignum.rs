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

/// `BigFloat` → extended-range [`FloatExp`] (top 53 bits of the mantissa, full exponent).
/// Unlike [`to_f64`] this never under/overflows: a 1e-4000 candidate offset or a 1e-71
/// near-nucleus orbit dip keeps its true magnitude. Reads the normalized top mantissa word
/// directly (value = `0.MSW… × 2^exp`), so it costs the same as `to_f64` — no bignum clone.
pub(crate) fn bf_to_floatexp(b: &BigFloat) -> crate::floatexp::FloatExp {
    use crate::floatexp::FloatExp;
    let digits = match b.mantissa_digits() {
        Some(d) if !d.is_empty() => d,
        _ => return FloatExp::ZERO,
    };
    // ⚠`exponent()` is `Some(0)` for ZERO — zero is detected by the mantissa, never the exponent.
    let exp = match b.exponent() {
        Some(e) => e,
        None => return FloatExp::ZERO,
    };
    let msw = *digits.last().unwrap();
    if msw == 0 {
        return FloatExp::ZERO;
    }
    let m = (msw as f64) / 18446744073709551616.0; // ÷2^64 — m ∈ [0.5, 1)
    let m = if matches!(b.sign(), Some(Sign::Neg)) { -m } else { m };
    FloatExp::new(m, exp)
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

/// Parse an **Imagina TEXT location** (`FileType::ImaginaText` in Imagina's `File.cpp`) into a
/// [`KfrView`], so Imagina locations can be imported the same way `.kfr` ones are.
///
/// Format, from Imagina's own writer: an indented hierarchy of `Key: value`, with a `Location`
/// block carrying `Size`, `Re`, `Im` and `Iterations`, plus a top-level `Formula`. Keys are matched
/// on their LAST path component so both the flat (`Location.Re:`) and indented (`Location:` then
/// `Re:`) spellings work — the writer emits the nested form, but hand-edited and tool-generated
/// files in the wild use the dotted one.
///
/// ⚠**`Size` is a HALF-HEIGHT, so magnification = 2/Size.** Imagina's binary header calls the same
/// quantity `HalfH`, and our magnification is `REFERENCE_HEIGHT (4) / view_height`, so
/// `4 / (2·Size)`. That is the one inferred quantity here — the field's semantics are not stated in
/// the text format itself — and it is pinned by a test below precisely so a correction is a
/// one-constant change rather than an archaeology exercise.
///
/// The **binary** `.im` format is deliberately NOT handled: its payload needs `HRReal`'s layout and
/// GMP `mpf` raw streams, neither of which is documented in the source we can read, and guessing at
/// a binary layout produces an importer that looks like it works. Callers should refuse it by its
/// magic (`0x000A0D56504D49FF`, i.e. the bytes `FF 49 4D 50 56 0D 0A 00`) with a clear message.
///
/// Hardened exactly as [`parse_kfr`] is: bounded size and value lengths, an allow-list of keys
/// (no formulas-as-code, no paths), the centre validated through [`parse_bf`], and clamped
/// zoom/iterations. Returns `None` unless a valid `Re`, `Im` and `Size` are present.
pub fn parse_imagina_text(text: &str) -> Option<KfrView> {
    if text.len() > 4_000_000 {
        return None;
    }
    let (mut re, mut im, mut size, mut iters) = (None, None, None, None);
    for line in text.lines().take(20_000) {
        let Some((key, val)) = line.split_once(':') else {
            continue;
        };
        let (key, val) = (key.trim(), val.trim());
        if val.len() > 100_000 || val.is_empty() {
            continue;
        }
        // Match the last dotted component so `Location.Re` and a nested `Re` both land.
        let leaf = key.rsplit('.').next().unwrap_or(key).trim();
        if leaf.eq_ignore_ascii_case("Re") {
            re = Some(val);
        } else if leaf.eq_ignore_ascii_case("Im") {
            im = Some(val);
        } else if leaf.eq_ignore_ascii_case("Size") {
            // HRReal prints as a plain decimal or with an exponent; both parse as f64 here, and a
            // value past f64 range is out of our viewport's reach anyway.
            size = val.parse::<f64>().ok().filter(|v| v.is_finite() && *v > 0.0);
        } else if leaf.eq_ignore_ascii_case("Iterations") {
            iters = val.parse::<u64>().ok().map(|v| v.min(1_000_000) as u32);
        }
        // every other key, `Formula` included, is ignored by design
    }
    let size = size?;
    let zoom = (2.0 / size).clamp(1.0, 1.0e300);
    Some(KfrView { cx: parse_bf(re?)?, cy: parse_bf(im?)?, zoom, iterations: iters })
}

/// The Imagina BINARY signature, so callers can refuse a `.im` file with a useful message instead
/// of parsing its bytes as text and silently importing nothing.
pub const IMAGINA_BINARY_MAGIC: [u8; 8] = [0xFF, 0x49, 0x4D, 0x50, 0x56, 0x0D, 0x0A, 0x00];

#[cfg(test)]
mod location_format_tests {
    // Parsers for location files written by OTHER programs (Imagina .imag, Kalles
    // Fraktaler / Fraktaler-3 .kfr), plus the go-to round trip that shares their
    // precision requirement. All untrusted input: a shared location is the artifact
    // people pass around on a forum.
    // ---- .kfr import (manual checklist step 73) and go-to round-trip (step 68) ----------------

    /// A 62-digit centre from a Kalles Fraktaler / Fraktaler-3 location file. The whole point of
    /// importing one is to land on a DEEP coordinate, so the property that matters is that the
    /// digits survive: a parser that quietly went through f64 would keep about 17 of them and
    /// still produce a plausible-looking view a long way from the one that was shared.
    const KFR_DEEP: &str = "\
Re: -1.7686249050856172346353441645074953226348553577059118970313449\n\
Im: 0.0041965917670430586733584119946276337571344847602401093387185\n\
Zoom: 1E30\n\
Iterations: 250000\n\
Colors: 16\n";

    #[test]
    fn kfr_import_keeps_every_digit_of_a_deep_centre() {
        let v = super::parse_kfr(KFR_DEEP).expect(".kfr with Re/Im/Zoom must parse");
        assert!((v.zoom - 1.0e30).abs() / 1.0e30 < 1e-12, "zoom {} != 1e30", v.zoom);
        assert_eq!(v.iterations, Some(250_000));

        // Round-trip the centre back to decimal and compare digit strings. Comparing f64s here
        // would be exactly the bug this guards: two centres that differ in the 30th digit are
        // the same f64 and utterly different views at 1e30x.
        for (got, want) in [
            (&v.cx, "-1.7686249050856172346353441645074953226348553577059118970313449"),
            (&v.cy, "0.0041965917670430586733584119946276337571344847602401093387185"),
        ] {
            let s = super::to_decimal_string(got);
            let (a, b) = (digits(&s), digits(want));
            let keep = b.len().min(a.len());
            assert!(
                keep >= 55 && a[..keep] == b[..keep],
                "centre lost precision:\n  got  {s}\n  want {want}"
            );
        }
    }

    /// Significant digits, sign and point removed, leading zeros dropped — so two spellings of
    /// the same number compare equal.
    fn digits(s: &str) -> String {
        let d: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
        d.trim_start_matches('0').to_string()
    }

    /// The parser must REFUSE a file that is not a location, rather than inventing a view from
    /// whatever it found. Absent is not the same as zero.
    #[test]
    fn kfr_import_refuses_a_file_that_is_not_a_location() {
        for bad in [
            "",
            "hello world",
            "Iterations: 500\nColors: 16\n",              // no Re/Im/Zoom
            "Re: -1.5\nZoom: 1E6\n",                       // no Im
            "Re: -1.5\nIm: 0.0\n",                         // no Zoom
            "Re: not-a-number\nIm: 0.0\nZoom: 1E6\n",     // unparseable coordinate
        ] {
            assert!(super::parse_kfr(bad).is_none(), "should have been refused: {bad:?}");
        }
    }

    /// Step 68: the go-to dialog is pre-filled with `to_decimal_string` and read back with
    /// `parse_bf`, so that pair must be lossless at the precision a deep view needs. astro-float's
    /// FromStr takes its precision from the DIGIT COUNT, which is why this is worth pinning
    /// rather than assuming.
    #[test]
    fn go_to_round_trips_a_deep_coordinate() {
        let want = "-1.7686249050856172346353441645074953226348553577059118970313449";
        let parsed = super::parse_bf(want).expect("a 62-digit coordinate must parse");
        let back = super::to_decimal_string(&parsed);
        let (a, b) = (digits(&back), digits(want));
        let keep = b.len().min(a.len());
        assert!(
            keep >= 55 && a[..keep] == b[..keep],
            "go-to round trip lost precision:\n  got  {back}\n  want {want}"
        );

        // And re-parsing what we printed must land on the same value, not merely something that
        // prints the same.
        let again = super::parse_bf(&back).expect("our own output must parse");
        assert_eq!(
            super::to_decimal_string(&again),
            back,
            "printing, parsing and printing again must be stable"
        );
    }

    use super::{parse_imagina_text, to_f64, IMAGINA_BINARY_MAGIC};

    #[test]
    fn the_nested_form_imagina_writes_is_parsed() {
        let t = "Formula: Mandelbrot
Location:
	Size: 2e-30
	Re: -0.75
	Im: 0.1
	Iterations: 25000
";
        let v = parse_imagina_text(t).expect("nested form must parse");
        assert!((to_f64(&v.cx) + 0.75).abs() < 1e-12);
        assert!((to_f64(&v.cy) - 0.1).abs() < 1e-12);
        assert_eq!(v.iterations, Some(25_000));
        // Size is a half-height: mag = 2/Size = 1e30.
        assert!((v.zoom / 1.0e30 - 1.0).abs() < 1e-9, "zoom was {}", v.zoom);
    }

    #[test]
    fn the_dotted_form_is_parsed_identically() {
        let nested = "Location:
  Size: 4
  Re: -0.5
  Im: 0
";
        let dotted = "Location.Size: 4
Location.Re: -0.5
Location.Im: 0
";
        let a = parse_imagina_text(nested).expect("nested");
        let b = parse_imagina_text(dotted).expect("dotted");
        assert_eq!(a.zoom, b.zoom);
        assert_eq!(to_f64(&a.cx), to_f64(&b.cx));
    }

    #[test]
    fn a_full_precision_centre_survives_verbatim() {
        // The whole point of importing: deep centres are long, and truncating one silently moves
        // the view. 139 digits, the length our own session files already carry.
        let re = "-0.101096363845622131810062384757351929938361014185318540959576769264716835033666295089126713641250096615102645646890476648163450651052568";
        let t = format!("Location:
 Size: 1e-100
 Re: {re}
 Im: 0.5
");
        let v = parse_imagina_text(&t).expect("deep centre must parse");
        let round = crate::to_decimal_string(&v.cx);
        assert!(round.starts_with("-1.0109636384562213181006238475735192993836101418531854095957676926471683503366629508912671364125"),
            "precision lost on import: {round}");
    }

    #[test]
    fn junk_and_hostile_input_is_refused_not_guessed() {
        assert!(parse_imagina_text("").is_none());
        assert!(parse_imagina_text("Formula: Mandelbrot
").is_none()); // no location
        assert!(parse_imagina_text("Location:
 Size: 0
 Re: 0
 Im: 0
").is_none()); // zero size
        assert!(parse_imagina_text("Location:
 Size: -1
 Re: 0
 Im: 0
").is_none());
        assert!(parse_imagina_text("Location:
 Size: nan
 Re: 0
 Im: 0
").is_none());
        assert!(parse_imagina_text("Location:
 Size: 1
 Re: not-a-number
 Im: 0
").is_none());
        // A binary .im must not be coaxed into a text parse.
        assert_eq!(IMAGINA_BINARY_MAGIC[0], 0xFF);
        assert_eq!(&IMAGINA_BINARY_MAGIC[1..5], b"IMPV");
    }

    #[test]
    fn a_shallow_location_clamps_rather_than_inverting() {
        // Size larger than the whole set: mag would fall below 1, which our viewport treats as the
        // home view. Clamp, never produce a sub-1 or negative magnification.
        let v = parse_imagina_text("Location:
 Size: 1e6
 Re: 0
 Im: 0
").expect("parses");
        assert_eq!(v.zoom, 1.0);
    }
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
