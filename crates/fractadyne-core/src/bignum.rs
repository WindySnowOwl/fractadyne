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
///
/// ⭐Also the test the app uses to decide whether a coordinate is worth PRESERVING as an
/// expression: a plain decimal cannot gain precision when re-parsed (its digits are all there is),
/// so only a NON-literal — a rational or a function/constant expression — is re-derivable to a
/// deeper zoom's precision, and only that is stored alongside the resolved centre.
pub fn is_decimal_literal(t: &str) -> bool {
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

/// Why a coordinate expression could not be read: a plain-language `reason`, and where in the
/// text it was found — `pos` is the 0-based CHARACTER index into the string exactly as given
/// (leading whitespace included), or `None` when the problem is the value as a whole (empty
/// input, a non-finite result, an imaginary part where a real was required).
///
/// `Display` renders both: `unknown function "sinn" — the functions are … (at character 6)`.
/// Callers prefix the FIELD the text came from; the message itself never names one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExprError {
    pub pos: Option<usize>,
    pub reason: String,
}

impl ExprError {
    fn whole(reason: impl Into<String>) -> Self {
        ExprError { pos: None, reason: reason.into() }
    }
}

impl core::fmt::Display for ExprError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.pos {
            Some(p) => write!(f, "{} (at character {})", self.reason, p + 1),
            None => f.write_str(&self.reason),
        }
    }
}

impl std::error::Error for ExprError {}

/// The functions the expression grammar accepts: `(name, signature, what it does)`.
///
/// ⭐This table is the ONE source for the in-app help page, the "unknown function" message and
/// the "is a function — write …" hint, and `expr_tables_match_the_evaluator` (core tests) holds
/// it equal to what [`parse_complex_expr`] actually evaluates — so the help cannot list a
/// function the parser rejects, nor the parser accept one the help omits.
pub const EXPR_FUNCTIONS: &[(&str, &str, &str)] = &[
    ("sqrt", "sqrt(z)", "Square root, principal branch. Works on negative and complex values: sqrt(-1) = i."),
    ("cbrt", "cbrt(x)", "Real cube root: cbrt(-8) = -2."),
    (
        "root",
        "root(x, n)",
        "Real n-th root, n a whole number from 1 to 1000000. Odd roots of a negative are allowed (root(-32, 5) = -2); even ones are not — use sqrt.",
    ),
    ("sin", "sin(x)", "Sine of an angle in radians."),
    ("cos", "cos(x)", "Cosine of an angle in radians."),
    ("tan", "tan(x)", "Tangent of an angle in radians."),
    ("asin", "asin(x)", "Inverse sine, in radians; needs -1 ≤ x ≤ 1."),
    ("acos", "acos(x)", "Inverse cosine, in radians; needs -1 ≤ x ≤ 1."),
    ("atan", "atan(x)", "Inverse tangent, in radians."),
    ("ln", "ln(x)", "Natural logarithm; needs x > 0."),
    ("log", "log(x)", "Base-10 logarithm; needs x > 0."),
    ("exp", "exp(x)", "e raised to the power x."),
    ("abs", "abs(z)", "Absolute value; for a complex value, its modulus |z|."),
];

/// The named constants the grammar knows: `(name, what it is)`. Names are case-insensitive.
/// Same contract as [`EXPR_FUNCTIONS`]: the help renders this list and a test pins it.
pub const EXPR_CONSTANTS: &[(&str, &str)] = &[
    ("pi", "3.14159… — half a turn, in radians"),
    ("tau", "6.28318… — a full turn (2·pi), in radians"),
    ("e", "2.71828… — the base of natural logarithms"),
    ("phi", "1.61803… — the golden ratio, (1 + sqrt(5))/2"),
    ("i", "the imaginary unit; also a suffix on a number, as in 0.25i"),
];

fn expr_function_names() -> String {
    EXPR_FUNCTIONS.iter().map(|f| f.0).collect::<Vec<_>>().join(", ")
}

fn expr_constant_names() -> String {
    let names: Vec<&str> = EXPR_CONSTANTS.iter().map(|c| c.0).collect();
    let (last, rest) = names.split_last().expect("constants table is non-empty");
    format!("{} and {last}", rest.join(", "))
}

/// Parse a **real** coordinate, reporting WHY it could not be read: a plain decimal, or an exact
/// rational / function expression (`-3/4`, `(1+2)/8`, `-0.5 + 0.25*cos(pi/4)`). Arithmetic runs
/// at `min_prec` bits or higher (see [`expr_precision`]). An expression with a nonzero imaginary
/// part is an error here — [`parse_complex_expr`] is the entry point for those.
pub fn parse_real_expr(s: &str, min_prec: usize) -> Result<BigFloat, ExprError> {
    let t = s.trim();
    if is_decimal_literal(t) {
        let mut cc = Consts::new().map_err(|_| ExprError::whole("arithmetic constants unavailable"))?;
        return parse_literal(t, min_prec, &mut cc)
            .ok_or_else(|| ExprError::whole(format!("the number {t:?} is out of range")));
    }
    let (re, im) = parse_complex_expr(s, min_prec)?;
    if !im.is_zero() {
        return Err(ExprError::whole(
            "this value must be real, but the expression has an imaginary part",
        ));
    }
    Ok(re)
}

/// Parse a **real** coordinate: [`parse_real_expr`] with the reason discarded — `None` is
/// "invalid input". Kept for the many call sites that only need the value; anything that shows
/// a message to a person should use the `Result` form.
pub fn parse_bf_prec(s: &str, min_prec: usize) -> Option<BigFloat> {
    parse_real_expr(s, min_prec).ok()
}

/// Parse a **complex** coordinate expression into `(re, im)`: decimals, an `i` suffix, the four
/// arithmetic operators, powers, parentheses, functions and constants. Written for exact landmark
/// entry — the canonical cases are `(37+16i)/100`, a point *exactly* on ∂M that cannot be typed
/// as a terminating decimal, and the polar form `x0 + r*cos(theta)` typed with literal values.
/// Arithmetic runs at `min_prec` bits or higher, so an inexact result (e.g. `cos(pi/3)` composed
/// at depth) carries enough digits for the zoom it's about to be viewed at.
///
/// Grammar (total; nesting, `^` chains and function calls all bounded by [`EXPR_MAX_DEPTH`]):
/// `sum := product (('+'|'-') product)*` · `product := unary (('*'|'/') unary)*` ·
/// `unary := ('+'|'-')* power` · `power := atom ['^' unary]` (right-assoc, so `2^-3` works and
/// `-2^2 = -4`) · `atom := '(' sum ')' | name '(' sum [',' sum] ')' | constant | 'i' | number ['i']`
///
/// **Functions** (angles in radians): `sqrt` (complex-capable, principal branch), `cbrt` (real,
/// odd — `cbrt(-8) = -2`), `root(x,n)` (real x, integer n ≥ 1; odd roots of negatives allowed),
/// `sin cos tan asin acos atan`, `ln` (natural), `log` (base 10), `exp`, `abs` (complex → |z|).
/// All others take REAL arguments and reject a nonzero imaginary part rather than guess a branch.
/// **Constants**: `pi`, `e`, `tau`, `phi`. Names are case-insensitive.
/// **`^`**: real base — any real exponent if the base is positive, integer exponents if negative
/// (fractional powers of a negative are complex and branch-ambiguous — use `root`); complex base —
/// integer exponents to ±4096. `0^0 = 1`.
///
/// Every refusal comes back as an [`ExprError`] that says what was found, where, and — for the
/// common slips (a typographic minus, `×`, `2pi`, a decimal comma, a bare `sin`) — what to type
/// instead. A coordinate must never be silently half-read, so trailing text is an error too.
pub fn parse_complex_expr(s: &str, min_prec: usize) -> Result<(BigFloat, BigFloat), ExprError> {
    let lead = s.len() - s.trim_start().len();
    let t = s.trim();
    if t.is_empty() {
        return Err(ExprError::whole("nothing to read — enter a number or an expression"));
    }
    // Byte offset in the trimmed text → character index in the text as given.
    let at = |err: ErrAt| ExprError {
        pos: Some(s[..(lead + err.pos).min(s.len())].chars().count()),
        reason: err.reason,
    };
    let p = expr_precision(t, min_prec);
    let cc = Consts::new().map_err(|_| ExprError::whole("arithmetic constants unavailable"))?;
    let mut e = Expr { b: t.as_bytes(), i: 0, p, depth: 0, cc };
    let v = e.sum().map_err(at)?;
    e.ws();
    if e.i != e.b.len() {
        // Trailing text — never silently ignore part of a coordinate.
        return Err(at(e.expected_operator(e.i, None)));
    }
    let bad = |b: &BigFloat| b.is_nan() || b.is_inf();
    if bad(&v.0) || bad(&v.1) {
        return Err(ExprError::whole("the result is not a finite number"));
    }
    Ok(v)
}

/// Parse a **complex** coordinate expression: [`parse_complex_expr`] with the reason discarded.
pub fn parse_complex_prec(s: &str, min_prec: usize) -> Option<(BigFloat, BigFloat)> {
    parse_complex_expr(s, min_prec).ok()
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
fn neg_bf(x: &BigFloat, p: usize) -> BigFloat {
    cx_zero(p).sub(x, p, RM)
}
fn abs_bf(x: &BigFloat, p: usize) -> BigFloat {
    if matches!(x.sign(), Some(Sign::Neg)) { neg_bf(x, p) } else { x.clone() }
}
/// `|z| = √(re²+im²)`. Expression values are coordinate-scale, so the squares can't meaningfully
/// overflow — and if they do, the resulting `inf` is rejected by the caller's finiteness check.
fn cx_abs(a: &Cx, p: usize) -> BigFloat {
    a.0.mul(&a.0, p, RM).add(&a.1.mul(&a.1, p, RM), p, RM).sqrt(p, RM)
}
/// Principal complex square root: real input stays on the real/imaginary axes exactly
/// (`sqrt(-1) = i`); otherwise `u = √((|z|+re)/2)`, `v = sign(im)·√((|z|−re)/2)`.
fn cx_sqrt(a: &Cx, p: usize) -> Cx {
    if a.1.is_zero() {
        return if matches!(a.0.sign(), Some(Sign::Neg)) {
            (cx_zero(p), neg_bf(&a.0, p).sqrt(p, RM))
        } else {
            (a.0.sqrt(p, RM), cx_zero(p))
        };
    }
    let half = BigFloat::from_f64(0.5, p);
    let m = cx_abs(a, p);
    let u = m.add(&a.0, p, RM).mul(&half, p, RM).sqrt(p, RM);
    let v = m.sub(&a.0, p, RM).mul(&half, p, RM).sqrt(p, RM);
    let v = if matches!(a.1.sign(), Some(Sign::Neg)) { neg_bf(&v, p) } else { v };
    (u, v)
}
/// Is `x` small enough to hand to a transcendental? Argument reduction for `sin(1e99999)` costs
/// time proportional to the EXPONENT, and no coordinate legitimately feeds a trig/exp argument
/// past ±2^32 — this is untrusted input (the `.kfr` threat model), so refuse rather than pay.
fn small_enough(x: &BigFloat) -> bool {
    x.exponent().is_none_or(|e| e <= 32)
}
/// The exponent as an exact `i64`, or `None` if it isn't an integer (or is absurdly large).
/// Exactness is checked against the bignum itself, not just the `f64` image, so `2.0000…1`
/// at deep precision can't masquerade as `2`.
fn int_exponent(ex: &BigFloat, p: usize) -> Option<i64> {
    let f = to_f64(ex);
    if !f.is_finite() || f.fract() != 0.0 || f.abs() > 9.0e15 {
        return None;
    }
    ex.sub(&BigFloat::from_f64(f, p), p, RM).is_zero().then_some(f as i64)
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

/// A parse failure inside [`Expr`]: the BYTE offset into the trimmed text plus the reason.
/// [`parse_complex_expr`] turns the offset into a character index in the caller's string.
struct ErrAt {
    pos: usize,
    reason: String,
}

impl ErrAt {
    fn new(pos: usize, reason: impl Into<String>) -> Self {
        ErrAt { pos, reason: reason.into() }
    }
}

type ExprResult<T> = Result<T, ErrAt>;

fn too_deep(pos: usize) -> ErrAt {
    ErrAt::new(
        pos,
        format!("too deeply nested — at most {EXPR_MAX_DEPTH} levels of parentheses, function calls and ^"),
    )
}

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
    /// The character starting at byte `i`. The cursor only ever advances over ASCII, so `i` is
    /// always on a character boundary of the (valid UTF-8) input.
    fn char_at(&self, i: usize) -> Option<char> {
        std::str::from_utf8(self.b.get(i..)?).ok()?.chars().next()
    }
    fn signature(name: &str) -> Option<&'static str> {
        EXPR_FUNCTIONS.iter().find(|f| f.0.eq_ignore_ascii_case(name)).map(|f| f.1)
    }

    // ---- the messages ----------------------------------------------------------------------

    /// Something other than a value where one was required (the start of the input, after an
    /// operator, after `(`).
    fn expected_value(&self, i: usize) -> ErrAt {
        const WANT: &str = "a number, a constant, a function or '('";
        let Some(c) = self.char_at(i) else {
            return ErrAt::new(i, format!("the expression ends where {WANT} was expected"));
        };
        match c {
            ')' => ErrAt::new(i, "unexpected ')' — there is nothing to evaluate before it"),
            '.' => ErrAt::new(i, "unexpected '.' — a number needs a digit, e.g. 0.5"),
            _ => self.unexpected(i, c, WANT),
        }
    }

    /// Something other than an operator (or the end / the closing bracket) after a complete
    /// value. `closing` names the bracket that would also be acceptable, inside `(` or a call.
    fn expected_operator(&self, i: usize, closing: Option<&str>) -> ErrAt {
        let want = match closing {
            Some(c) => format!("an operator (+ - * / ^) or {c}"),
            None => "an operator (+ - * / ^) or the end of the expression".to_string(),
        };
        let Some(c) = self.char_at(i) else {
            return ErrAt::new(i, format!("the expression ends where {want} was expected"));
        };
        let hint = match c {
            '(' => Some(
                "implicit multiplication is not supported — write '*' before the parenthesis \
                 (and a constant such as pi is not a function)",
            ),
            c if c.is_ascii_alphabetic() => {
                Some("write '*' between a number and a name, e.g. 2*pi")
            }
            c if c.is_ascii_digit() => {
                Some("two numbers in a row — remove the space, or put an operator between them")
            }
            _ => None,
        };
        match hint {
            Some(h) => ErrAt::new(i, format!("unexpected {c:?} — {h}")),
            None => self.unexpected(i, c, &want),
        }
    }

    /// The typo hints shared by both contexts: the characters a word processor, a keyboard
    /// layout or a maths habit puts into a coordinate, each with what to type instead.
    fn unexpected(&self, i: usize, c: char, want: &str) -> ErrAt {
        let hint = match c {
            '\u{2212}' | '\u{2013}' | '\u{2014}' => "that is a typographic dash; use the ASCII minus '-'",
            '\u{d7}' | '\u{b7}' | '\u{2219}' | '\u{22c5}' => "use '*' to multiply",
            '\u{f7}' => "use '/' to divide",
            '\u{221a}' => "write sqrt(…)",
            '\u{3c0}' => "write pi",
            '\u{b2}' => "write ^2",
            '\u{b3}' => "write ^3",
            '\u{b0}' => {
                "angles are in radians — multiply degrees by pi/180, or use the polar entry with \
                 the degrees unit"
            }
            '[' | ']' | '{' | '}' => "only round parentheses ( ) can group",
            '"' | '\'' | '\u{201c}' | '\u{201d}' | '\u{2018}' | '\u{2019}' => "remove the quotes",
            ',' => {
                "the decimal point is '.'; a comma only separates the two arguments of root(x, n)"
            }
            '=' => "enter just the value, without an '='",
            _ => "",
        };
        if hint.is_empty() {
            ErrAt::new(i, format!("unexpected {c:?}; expected {want}"))
        } else {
            ErrAt::new(i, format!("unexpected {c:?} — {hint}"))
        }
    }

    // ---- the grammar -----------------------------------------------------------------------

    fn sum(&mut self) -> ExprResult<Cx> {
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
                _ => return Ok(acc),
            }
        }
    }

    fn product(&mut self) -> ExprResult<Cx> {
        let mut acc = self.unary()?;
        loop {
            match self.peek() {
                Some(b'*') => {
                    self.i += 1;
                    let r = self.unary()?;
                    acc = cx_mul(&acc, &r, self.p);
                }
                Some(b'/') => {
                    let op = self.i;
                    self.i += 1;
                    let r = self.unary()?;
                    acc = cx_div(&acc, &r, self.p).ok_or_else(|| ErrAt::new(op, "division by zero"))?;
                }
                _ => return Ok(acc),
            }
        }
    }

    /// Signs are consumed iteratively, not by self-recursion — a pasted `-----…` must not be
    /// able to blow the stack (untrusted input, same threat model as the depth cap).
    fn unary(&mut self) -> ExprResult<Cx> {
        let mut neg = false;
        loop {
            match self.peek() {
                Some(b'-') => {
                    self.i += 1;
                    neg = !neg;
                }
                Some(b'+') => {
                    self.i += 1;
                }
                _ => break,
            }
        }
        let v = self.power()?;
        Ok(if neg { cx_neg(&v, self.p) } else { v })
    }

    /// `atom ['^' unary]` — right-associative (`2^3^2 = 2^(3^2) = 512`), and the exponent
    /// re-enters `unary` so `2^-3` parses. Unary minus binds looser: `-2^2 = -4`.
    fn power(&mut self) -> ExprResult<Cx> {
        let base = self.atom()?;
        if self.peek() != Some(b'^') {
            return Ok(base);
        }
        let op = self.i;
        if self.depth >= EXPR_MAX_DEPTH {
            return Err(too_deep(op));
        }
        self.i += 1;
        self.depth += 1;
        let e = self.unary()?;
        self.depth -= 1;
        self.pow_value(&base, &e, op)
    }

    fn atom(&mut self) -> ExprResult<Cx> {
        match self.peek() {
            None => Err(self.expected_value(self.i)),
            Some(b'(') => {
                let open = self.i;
                if self.depth >= EXPR_MAX_DEPTH {
                    return Err(too_deep(open));
                }
                self.i += 1;
                self.depth += 1;
                let v = self.sum()?;
                self.depth -= 1;
                self.ws();
                if self.b.get(self.i) != Some(&b')') {
                    if self.i >= self.b.len() {
                        return Err(ErrAt::new(open, "this '(' is never closed"));
                    }
                    return Err(self.expected_operator(self.i, Some("')'")));
                }
                self.i += 1;
                Ok(v)
            }
            Some(c) if c.is_ascii_alphabetic() => self.word(),
            Some(c) if c.is_ascii_digit() || c == b'.' => self.number(),
            Some(_) => Err(self.expected_value(self.i)),
        }
    }

    /// An alphanumeric name: the imaginary unit, a constant, or a function call. Lexed as a
    /// whole word so `pi` can never half-read as `p·i`, and unknown names are a parse error —
    /// a coordinate must never be silently misread.
    fn word(&mut self) -> ExprResult<Cx> {
        let start = self.i;
        while matches!(self.b.get(self.i), Some(c) if c.is_ascii_alphanumeric()) {
            self.i += 1;
        }
        let text = self.b; // copy the slice ref out so `self` can be borrowed mutably below
        let name = std::str::from_utf8(&text[start..self.i]).unwrap_or("");
        let p = self.p;
        let eq = |n: &str| name.eq_ignore_ascii_case(n);
        if eq("i") {
            return Ok((cx_zero(p), BigFloat::from_f64(1.0, p)));
        }
        if eq("pi") {
            return Ok((self.cc.pi(p, RM), cx_zero(p)));
        }
        if eq("e") {
            return Ok((self.cc.e(p, RM), cx_zero(p)));
        }
        if eq("tau") {
            return Ok((double_bf(&self.cc.pi(p, RM)), cx_zero(p)));
        }
        if eq("phi") {
            let v = BigFloat::from_f64(5.0, p)
                .sqrt(p, RM)
                .add(&BigFloat::from_f64(1.0, p), p, RM)
                .mul(&BigFloat::from_f64(0.5, p), p, RM);
            return Ok((v, cx_zero(p)));
        }
        // Not a constant: a function call, or a mistake — and the two mistakes read differently.
        self.ws();
        if self.b.get(self.i) == Some(&b'(') {
            return self.call(name, start);
        }
        Err(match Self::signature(name) {
            Some(sig) => ErrAt::new(start, format!("{name:?} is a function — write {sig}, with parentheses")),
            None => ErrAt::new(
                start,
                format!(
                    "unknown name {name:?} — the constants are {}; the functions are {}",
                    expr_constant_names(),
                    expr_function_names()
                ),
            ),
        })
    }

    /// `name '(' sum [',' sum] ')'` — `root` is the only two-argument function. `start` is the
    /// name's position, which is where an error about the call as a whole points.
    fn call(&mut self, name: &str, start: usize) -> ExprResult<Cx> {
        let Some(sig) = Self::signature(name) else {
            return Err(ErrAt::new(
                start,
                format!("unknown function {name:?} — the functions are {}", expr_function_names()),
            ));
        };
        if self.depth >= EXPR_MAX_DEPTH {
            return Err(too_deep(start));
        }
        self.i += 1; // past '('
        self.depth += 1;
        if self.peek() == Some(b')') {
            return Err(ErrAt::new(start, format!("{name} needs an argument — write {sig}")));
        }
        let a = self.sum()?;
        let second = if self.peek() == Some(b',') {
            self.i += 1;
            Some(self.sum()?)
        } else {
            None
        };
        self.depth -= 1;
        self.ws();
        if self.b.get(self.i) != Some(&b')') {
            if self.i >= self.b.len() {
                return Err(ErrAt::new(start, format!("missing ')' to close {name}(")));
            }
            return Err(self.expected_operator(self.i, Some("')'")));
        }
        self.i += 1;
        self.apply(name, sig, &a, second.as_ref(), start)
    }

    fn apply(&mut self, name: &str, sig: &str, a: &Cx, second: Option<&Cx>, at: usize) -> ExprResult<Cx> {
        let p = self.p;
        let eq = |n: &str| name.eq_ignore_ascii_case(n);
        // The real-argument gate: reject a nonzero imaginary part instead of guessing a branch.
        let real = |v: &Cx| v.1.is_zero().then(|| v.0.clone());
        let needs_real = || ErrAt::new(at, format!("{name} needs a real argument, but this one has an imaginary part"));
        if eq("root") {
            // n-th root: real x, integer n ≥ 1; odd roots of negatives are real and allowed.
            let x = real(a).ok_or_else(needs_real)?;
            let Some(second) = second else {
                return Err(ErrAt::new(at, format!("root takes two arguments — write {sig}")));
            };
            let n = real(second)
                .and_then(|n| int_exponent(&n, p))
                .ok_or_else(|| ErrAt::new(at, "root(x, n): n must be a whole number"))?;
            if !(1..=1_000_000).contains(&n) {
                return Err(ErrAt::new(at, "root(x, n): n must be between 1 and 1000000"));
            }
            if x.is_zero() {
                return Ok((cx_zero(p), cx_zero(p)));
            }
            let neg = matches!(x.sign(), Some(Sign::Neg));
            if neg && n % 2 == 0 {
                return Err(ErrAt::new(
                    at,
                    "root(x, n): an even root of a negative number is complex — use sqrt for the \
                     principal value, or an odd n",
                ));
            }
            let inv_n = BigFloat::from_f64(1.0, p).div(&BigFloat::from_f64(n as f64, p), p, RM);
            let v = abs_bf(&x, p).pow(&inv_n, p, RM, &mut self.cc);
            return Ok((if neg { neg_bf(&v, p) } else { v }, cx_zero(p)));
        }
        if second.is_some() {
            return Err(ErrAt::new(at, format!("{name} takes one argument — write {sig}")));
        }
        if eq("sqrt") {
            return Ok(cx_sqrt(a, p));
        }
        if eq("abs") {
            return Ok(if a.1.is_zero() {
                (abs_bf(&a.0, p), cx_zero(p))
            } else {
                (cx_abs(a, p), cx_zero(p))
            });
        }
        if eq("cbrt") {
            let x = real(a).ok_or_else(needs_real)?;
            let neg = matches!(x.sign(), Some(Sign::Neg));
            let v = abs_bf(&x, p).cbrt(p, RM);
            return Ok((if neg { neg_bf(&v, p) } else { v }, cx_zero(p)));
        }
        let x = real(a).ok_or_else(needs_real)?;
        let v = if eq("sin") || eq("cos") || eq("tan") || eq("exp") {
            if !small_enough(&x) {
                // Argument reduction at absurd magnitude — see `small_enough`.
                return Err(ErrAt::new(at, format!("{name}: the argument is too large (the limit is ±2^32, about 4.3e9)")));
            }
            if eq("sin") {
                x.sin(p, RM, &mut self.cc)
            } else if eq("cos") {
                x.cos(p, RM, &mut self.cc)
            } else if eq("tan") {
                x.tan(p, RM, &mut self.cc)
            } else {
                x.exp(p, RM, &mut self.cc)
            }
        } else if eq("asin") {
            x.asin(p, RM, &mut self.cc)
        } else if eq("acos") {
            x.acos(p, RM, &mut self.cc)
        } else if eq("atan") {
            x.atan(p, RM, &mut self.cc)
        } else if eq("ln") {
            x.ln(p, RM, &mut self.cc)
        } else if eq("log") {
            x.log10(p, RM, &mut self.cc)
        } else {
            // Unreachable while `signature` and this chain agree; the test pins that.
            return Err(ErrAt::new(at, format!("unknown function {name:?}")));
        };
        // A domain error (ln of a negative, asin(2)) comes back NaN and is rejected here, so it
        // can't hide inside a larger expression that happens to survive the top-level check.
        if v.is_nan() {
            let why = if eq("ln") || eq("log") {
                format!("{name} needs a positive argument")
            } else if eq("asin") || eq("acos") {
                format!("{name} needs an argument between -1 and 1")
            } else {
                format!("{name} is undefined for this argument")
            };
            return Err(ErrAt::new(at, why));
        }
        Ok((v, cx_zero(p)))
    }

    /// `base ^ exponent`. The exponent must be real; see the grammar doc for the base cases.
    /// `at` is the `^`, which is where every refusal here points.
    fn pow_value(&mut self, base: &Cx, e: &Cx, at: usize) -> ExprResult<Cx> {
        let p = self.p;
        if !e.1.is_zero() {
            return Err(ErrAt::new(at, "the exponent must be real"));
        }
        let ex = &e.0;
        if !small_enough(ex) {
            return Err(ErrAt::new(at, "the exponent is too large (the limit is ±2^32, about 4.3e9)"));
        }
        if base.1.is_zero() {
            let b = &base.0;
            if b.is_zero() {
                // 0^0 = 1 (the parser convention), 0^positive = 0, 0^negative undefined.
                if ex.is_zero() {
                    return Ok((BigFloat::from_f64(1.0, p), cx_zero(p)));
                }
                return if matches!(ex.sign(), Some(Sign::Neg)) {
                    Err(ErrAt::new(at, "0 raised to a negative power is undefined"))
                } else {
                    Ok((cx_zero(p), cx_zero(p)))
                };
            }
            if !matches!(b.sign(), Some(Sign::Neg)) {
                let v = b.pow(ex, p, RM, &mut self.cc);
                if v.is_nan() {
                    return Err(ErrAt::new(at, "this power is undefined"));
                }
                return Ok((v, cx_zero(p)));
            }
            // Negative real base: integer exponents only (odd roots go through `root`/`cbrt`).
            let Some(n) = int_exponent(ex, p) else {
                return Err(ErrAt::new(
                    at,
                    "a negative number to a fractional power is complex and ambiguous — use \
                     root(x, n) or cbrt(x) for a real root, or sqrt(x) for the principal complex one",
                ));
            };
            let v = abs_bf(b, p).pow(ex, p, RM, &mut self.cc);
            if v.is_nan() {
                return Err(ErrAt::new(at, "this power is undefined"));
            }
            return Ok((if n & 1 == 1 { neg_bf(&v, p) } else { v }, cx_zero(p)));
        }
        // Complex base: small integer exponents by binary powering.
        let Some(n) = int_exponent(ex, p) else {
            return Err(ErrAt::new(at, "a complex base needs a whole-number exponent"));
        };
        if n.unsigned_abs() > 4096 {
            return Err(ErrAt::new(at, "a complex base takes an exponent between -4096 and 4096"));
        }
        let mut acc = (BigFloat::from_f64(1.0, p), cx_zero(p));
        let mut sq = base.clone();
        let mut k = n.unsigned_abs();
        while k > 0 {
            if k & 1 == 1 {
                acc = cx_mul(&acc, &sq, p);
            }
            k >>= 1;
            if k > 0 {
                sq = cx_mul(&sq, &sq, p);
            }
        }
        if n < 0 {
            cx_div(&(BigFloat::from_f64(1.0, p), cx_zero(p)), &acc, p)
                .ok_or_else(|| ErrAt::new(at, "division by zero — the base is 0"))
        } else {
            Ok(acc)
        }
    }

    /// A decimal literal (with optional fraction and exponent) and an optional `i` suffix.
    fn number(&mut self) -> ExprResult<Cx> {
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
            return Err(self.expected_value(start));
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
        let lit = std::str::from_utf8(&text[start..self.i]).unwrap_or("");
        let p = self.p;
        let v = parse_literal(lit, p, &mut self.cc)
            .ok_or_else(|| ErrAt::new(start, format!("the number {lit:?} is out of range")))?;
        if matches!(self.b.get(self.i), Some(b'i') | Some(b'I')) {
            self.i += 1;
            Ok((cx_zero(self.p), v))
        } else {
            Ok((v, cx_zero(self.p)))
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
    // ⭐⭐**Line endings and clipboard damage, before anything else.** These files arrive by
    // download and by paste — a `.map` from a 2003 archive can carry lone-CR endings, which
    // `str::lines()` does not split on, so the whole file reads as ONE line and parses as
    // nothing. A palette pasted out of a forum post arrives with smart quotes and en dashes.
    // `clean` normalizes CR / CRLF / LF and repairs both, so everything below can assume `\n`.
    let cleaned = fractadyne_text::clean(text);
    let text = cleaned.text.as_str();
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
    // ⭐⭐**Line endings and clipboard damage, before anything else.** These files arrive by
    // download and by paste — a `.map` from a 2003 archive can carry lone-CR endings, which
    // `str::lines()` does not split on, so the whole file reads as ONE line and parses as
    // nothing. A palette pasted out of a forum post arrives with smart quotes and en dashes.
    // `clean` normalizes CR / CRLF / LF and repairs both, so everything below can assume `\n`.
    let cleaned = fractadyne_text::clean(text);
    let text = cleaned.text.as_str();
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
mod location_format_tests;

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

/// `a − b` at precision `p`. A thin wrapper so callers outside this crate (the app, computing the
/// offset of a view centre from an anchor expression, and testing whether a centre still sits
/// exactly on that anchor) can do coordinate arithmetic without importing astro-float's rounding
/// vocabulary — and get the same rounding the viewport's own centre math uses.
pub fn bf_sub(a: &BigFloat, b: &BigFloat, p: usize) -> BigFloat {
    a.sub(b, p, RM)
}

/// `a + b` at precision `p`. Companion to [`bf_sub`], for reconstructing a centre from an anchor
/// expression plus a saved offset.
pub fn bf_add(a: &BigFloat, b: &BigFloat, p: usize) -> BigFloat {
    a.add(b, p, RM)
}
