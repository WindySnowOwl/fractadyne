//! Extended-range floating point: `FloatExp` (`m·2^e`) and its complex form `CFloatExp`.
//!
//! Used for the viewport scale and BLA / perturbation coefficients, whose magnitudes run far
//! past f64's ~1e±308 range. Arithmetic is exposed as `std::ops` operators (impls below).

use astro_float::BigFloat;

fn ldexp_f64(m: f64, e: i32) -> f64 {
    if e >= -1022 {
        m * 2f64.powi(e.min(1023))
    } else {
        // Subnormal range: split the shift so neither factor overflows/underflows alone.
        m * 2f64.powi(-1022) * 2f64.powi((e + 1022).max(-1074))
    }
}

/// A real number `m · 2^e` with an `i32` base-2 exponent, so its magnitude reaches far past
/// `f64`'s ~1e±308 range. Used for the viewport scale (`units_per_pixel`), which a plain
/// `f64` underflows at extreme zoom. The mantissa is normalized to `|m| ∈ [1, 2)` (carrying
/// the sign), or exactly `0`. Inputs are always well-conditioned (a normalized mantissa
/// times a pixel count or zoom factor), so normalization needs only a small shift.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatExp {
    pub m: f64,
    pub e: i32,
}

// Extended-range arithmetic as `std::ops`, so call sites read as `a * b - c` rather than
// `a.mul(b).sub(c)`. `mul` adds exponents; `add` aligns to the larger exponent (the smaller
// mantissa is dropped once it falls past f64's ~53 bits); `sub` is `self + (−o)`.
impl std::ops::Mul for FloatExp {
    type Output = FloatExp;
    fn mul(self, o: FloatExp) -> FloatExp {
        FloatExp::norm(self.m * o.m, self.e + o.e)
    }
}
impl std::ops::Add for FloatExp {
    type Output = FloatExp;
    fn add(self, o: FloatExp) -> FloatExp {
        if self.m == 0.0 {
            return o;
        }
        if o.m == 0.0 {
            return self;
        }
        let (hi, lo) = if self.e >= o.e { (self, o) } else { (o, self) };
        let de = hi.e - lo.e;
        if de > 120 {
            return hi;
        }
        FloatExp::norm(hi.m + lo.m * 2f64.powi(-de), hi.e)
    }
}
impl std::ops::Sub for FloatExp {
    type Output = FloatExp;
    fn sub(self, o: FloatExp) -> FloatExp {
        self + FloatExp { m: -o.m, e: o.e }
    }
}

impl FloatExp {
    pub const ZERO: FloatExp = FloatExp { m: 0.0, e: 0 };

    fn norm(mut m: f64, mut e: i32) -> FloatExp {
        if m == 0.0 || !m.is_finite() {
            return FloatExp { m, e: 0 };
        }
        let a = m.abs();
        if !(1.0..2.0).contains(&a) {
            let k = a.log2().floor() as i32;
            m *= 2f64.powi(-k);
            e += k;
            while m.abs() >= 2.0 {
                m *= 0.5;
                e += 1;
            }
            while m.abs() < 1.0 {
                m *= 2.0;
                e -= 1;
            }
        }
        FloatExp { m, e }
    }

    pub fn from_f64(v: f64) -> FloatExp {
        FloatExp::norm(v, 0)
    }

    /// Construct from a raw `(mantissa, exponent)` pair, normalizing. Used to reload a
    /// persisted scale (`m`, `e` stored separately so it survives past `f64` range).
    pub fn new(m: f64, e: i32) -> FloatExp {
        FloatExp::norm(m, e)
    }

    /// Saturating conversion to `f64` (`0` on underflow, `±∞` on overflow).
    pub fn to_f64(self) -> f64 {
        if self.m == 0.0 {
            0.0
        } else if self.e > 1023 {
            self.m * f64::INFINITY
        } else if self.e < -1074 {
            0.0
        } else {
            ldexp_f64(self.m, self.e)
        }
    }

    pub fn mul_f64(self, k: f64) -> FloatExp {
        FloatExp::norm(self.m * k, self.e)
    }

    /// Reciprocal `1/self` in extended range. (`*`/`+`/`-` are `std::ops` impls, above.)
    pub fn recip(self) -> FloatExp {
        FloatExp::norm(1.0 / self.m, -self.e)
    }

    /// Magnitude (drops the sign).
    pub fn abs(self) -> FloatExp {
        FloatExp { m: self.m.abs(), e: self.e }
    }

    /// Square root (`≥ 0` inputs; clamps negatives/zero to `0`).
    pub fn sqrt(self) -> FloatExp {
        if self.m <= 0.0 {
            return FloatExp::ZERO;
        }
        if self.e & 1 == 0 {
            FloatExp::norm(self.m.sqrt(), self.e / 2)
        } else {
            // odd exponent: pull one power of two under the root (works for e<0 too).
            FloatExp::norm((self.m * 2.0).sqrt(), (self.e - 1) / 2)
        }
    }

    /// Signed value comparison (`self < o`).
    pub fn lt(self, o: FloatExp) -> bool {
        (self - o).m < 0.0
    }

    /// GPU upload form: `(mantissa f32, exponent)` — mantissa is normalized to `[1,2)` (or 0).
    pub fn to_f32_exp(self) -> (f32, i32) {
        (self.m as f32, self.e)
    }

    /// `log2` of the magnitude (finite for any representable value; `−∞` for `0`).
    pub fn log2(self) -> f64 {
        if self.m == 0.0 {
            f64::NEG_INFINITY
        } else {
            self.m.abs().log2() + self.e as f64
        }
    }

    /// Multiply by `2^x` for an arbitrary `f64` exponent (split into integer + fraction).
    pub fn mul_pow2(self, x: f64) -> FloatExp {
        let fl = x.floor();
        FloatExp::norm(self.m * 2f64.powf(x - fl), self.e + fl as i32)
    }

    /// As a `BigFloat` at precision `p` (exact — mantissa scaled by `2^e` via the exponent).
    pub fn to_bf(self, p: usize) -> BigFloat {
        let mut b = BigFloat::from_f64(self.m, p);
        if self.m != 0.0 {
            if let Some(be) = b.exponent() {
                b.set_exponent(be + self.e);
            }
        }
        b
    }
}

/// Complex value with `FloatExp` parts — BLA coefficients span far beyond f64's exponent
/// range (a merged `A` is a long product of `2Zₙ`).
#[derive(Clone, Copy)]
pub struct CFloatExp {
    pub re: FloatExp,
    pub im: FloatExp,
}

impl std::ops::Mul for CFloatExp {
    type Output = CFloatExp;
    fn mul(self, o: CFloatExp) -> CFloatExp {
        CFloatExp {
            re: self.re * o.re - self.im * o.im,
            im: self.re * o.im + self.im * o.re,
        }
    }
}
impl std::ops::Add for CFloatExp {
    type Output = CFloatExp;
    fn add(self, o: CFloatExp) -> CFloatExp {
        CFloatExp { re: self.re + o.re, im: self.im + o.im }
    }
}

impl CFloatExp {
    /// Magnitude `|a| = hypot(re, im)` in extended range.
    pub fn abs(self) -> FloatExp {
        (self.re * self.re + self.im * self.im).sqrt()
    }

    /// GPU upload form: a **shared-exponent** df32 mantissa `[re_hi, re_lo, im_hi, im_lo]`
    /// (both parts scaled to one base-2 exponent) plus that exponent — matches the SA coeff
    /// layout / the shader's `Fe` (df32 mantissa + i32 exponent).
    pub fn to_mantissa_exp(self) -> ([f32; 4], i32) {
        let e = match (self.re.m == 0.0, self.im.m == 0.0) {
            (true, true) => return ([0.0; 4], 0),
            (false, true) => self.re.e,
            (true, false) => self.im.e,
            (false, false) => self.re.e.max(self.im.e),
        };
        let split = |f: FloatExp| -> (f32, f32) {
            let v = f.m * 2f64.powi(f.e - e); // value·2^−e, in (−2, 2]
            let hi = v as f32;
            (hi, (v - hi as f64) as f32)
        };
        let (rh, rl) = split(self.re);
        let (ih, il) = split(self.im);
        ([rh, rl, ih, il], e)
    }
}
