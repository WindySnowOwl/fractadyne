//! The **MPFR** reference-orbit backend (`rug`), behind the off-by-default `rug` feature.
//!
//! Exists to make the deep-frame hot loop faster: the reference-orbit build is `max_iter × step`
//! in bignum and dominates a deep frame, and astro-float exposes no destination-reuse arithmetic
//! at all — every `add`/`sub`/`mul` allocates, which at low precision costs more than the
//! arithmetic does. Measured on this machine (validation probe, 2026-08-26), an MPFR orbit runs
//! **3.2–4.7× faster** than astro-float from 2 to 129 limbs; the win is mostly *allocation* at 2
//! limbs (2.74× of it) and almost entirely *multiply algorithm* at 129 limbs (4.13× of it).
//!
//! # This backend is bit-identical to astro-float, and that is not an accident
//!
//! It reproduces astro-float's arithmetic exactly, so the F3 corpus goldens keep gating both and
//! no deep render needs a second blessed set. Three conditions carry that, each measured rather
//! than assumed (9 operand classes × 32 trials × 7 precisions, plus 20,000 iterations of the real
//! `z²+c` recurrence — zero divergence), and each is implemented below with a comment saying so:
//!
//! 1. **`Round::Zero`.** astro-float's `RM` is `RoundingMode::None` — "skip rounding operation",
//!    i.e. truncation. MPFR's *default* nearest rounding does **not** match.
//! 2. **Word-granular precision.** astro-float rounds a requested precision up to whole 64-bit
//!    words, so this backend must run at `p.div_ceil(64) * 64`, not at `p`.
//! 3. **Truncating `f64` extraction.** `rug::Float::to_f64()` rounds to nearest and would shift
//!    emitted samples by ~1 ulp *from an identical bignum state*; `crate::to_f64` truncates.
//!
//! ⚠**Licensing.** `rug`, `gmp-mpfr-sys` and the GMP/MPFR they link are **LGPL-3.0+**, against
//! this project's MIT OR Apache-2.0. The obligations attach to *conveying a binary*, not to
//! building or benchmarking one — which is why this feature is off by default and why a release
//! artifact built with it is a separate decision, not a side effect.

use crate::backend::RefBackend;
use crate::fractal::Field;
use astro_float::{BigFloat, Sign};
use rug::{float::Round, integer::Order, Float, Integer};

/// Condition 1: every rounded operation truncates toward zero, matching `RoundingMode::None`.
const RZ: Round = Round::Zero;

impl Field for Float {
    /// MPFR working precision in bits — **already word-rounded** by [`RefBackend::ctx_for`]
    /// (condition 2). Nothing in this file may re-derive it from `p`.
    type Ctx = u32;

    #[inline]
    fn fmul(&self, o: &Float, p: u32) -> Float {
        Float::with_val_round(p, self * o, RZ).0
    }
    #[inline]
    fn fadd(&self, o: &Float, p: u32) -> Float {
        Float::with_val_round(p, self + o, RZ).0
    }
    #[inline]
    fn fsub(&self, o: &Float, p: u32) -> Float {
        Float::with_val_round(p, self - o, RZ).0
    }
    #[inline]
    fn fabs(&self) -> Float {
        // Exact: clearing the sign cannot round, so this keeps the value's own precision.
        Float::with_val(self.prec(), self.abs_ref())
    }
    #[inline]
    fn fdouble(&self) -> Float {
        // Exact ×2 via the binary exponent — the counterpart of astro-float's `double_bf`, and the
        // reason a complex square costs one multiply less than a general complex multiply.
        let mut f = self.clone();
        f <<= 1;
        f
    }
}

impl RefBackend for Float {
    const BIT: u32 = 1;

    #[inline]
    fn ctx_for(p: usize) -> u32 {
        // Condition 2. astro-float allocates whole 64-bit words, so matching its *requested*
        // precision would silently be a different arithmetic.
        (p.div_ceil(64) * 64) as u32
    }

    fn from_carrier(v: &BigFloat, ctx: u32) -> Self {
        // ⚠Convert at the CARRIER VALUE'S OWN width, not at `ctx`.
        //
        // astro-float's `a.mul(&b, p, RM)` keeps each operand at whatever width it already has and
        // rounds only the RESULT to `p`. MPFR has the identical model (operands carry their own
        // precision; the destination's precision governs the rounding), so matching it means
        // handing over the operand intact.
        //
        // Truncating to `ctx` here instead changes the INPUT, and then the two backends are not
        // doing the same sum. That is reachable in the app, not a theoretical worry: a pasted
        // coordinate is parsed by `parse_bf_prec`, which returns the literal's NATURAL width
        // whenever that already exceeds the requested precision (`if min_prec <= natural { return
        // Some(auto) }`) — so a 33-digit centre viewed at a shallow zoom is a 2-word value being
        // used at p=64. Caught by `the_mpfr_backend_is_byte_identical_to_astro_float`, which
        // diverged on 81 of 1800 cases, every one of them at p=64.
        let words = v.mantissa_digits().map(|d| d.len()).unwrap_or(0);
        let prec = if words == 0 { ctx } else { (words * 64) as u32 };
        astro_to_rug(v, prec.max(64))
    }

    fn to_carrier(&self, ctx: u32) -> BigFloat {
        rug_to_astro(self, (ctx as usize) / 64)
    }

    #[inline]
    fn from_f64(v: f64, ctx: u32) -> Self {
        Float::with_val(ctx, v)
    }

    #[inline]
    fn to_f64_trunc(&self) -> f64 {
        // Condition 3. `to_f64()` alone rounds to nearest; truncating to f64's 53 significant bits
        // first reproduces `crate::to_f64`'s `ret |= m >> 12`.
        if self.is_zero() || !self.is_finite() {
            return self.to_f64();
        }
        let mut g = Float::with_val(self.prec(), self);
        g.set_prec_round(53, RZ);
        g.to_f64()
    }
}

/// Exact `BigFloat` → `Float`. The mantissa rides across as an integer, so no digit is lost.
///
/// Both libraries normalize the significand to `[0.5, 1)` — astro-float stores `mantissa · 2^e`
/// with the top bit set, and MPFR's `get_exp` uses the same convention — which is what makes the
/// two comparable at all.
fn astro_to_rug(v: &BigFloat, prec: u32) -> Float {
    let Some((words, _n, s, e, _)) = v.as_raw_parts() else {
        return Float::with_val(prec, f64::NAN); // NaN / ±∞ carry no mantissa
    };
    if words.iter().all(|&w| w == 0) {
        return Float::with_val(prec, 0u32);
    }
    let int = Integer::from_digits(words, Order::Lsf);
    let nb = (words.len() * 64) as i32; // value = int · 2^(e − nb)
    let mut f = Float::with_val(prec.max(nb as u32), &int); // exact: `int` has at most `nb` bits
    f <<= e - nb;
    if matches!(s, Sign::Neg) {
        f = -f;
    }
    if f.prec() != prec {
        f.set_prec_round(prec, RZ);
    }
    f
}

/// Exact `Float` → `BigFloat`, the direction [`crate::OrbitTail`] needs so a tail stays in the
/// carrier type and a later extend can resume from it.
fn rug_to_astro(f: &Float, nwords: usize) -> BigFloat {
    if f.is_nan() {
        return BigFloat::from_f64(f64::NAN, nwords * 64);
    }
    if f.is_infinite() {
        return BigFloat::from_f64(
            if f.is_sign_negative() { f64::NEG_INFINITY } else { f64::INFINITY },
            nwords * 64,
        );
    }
    if f.is_zero() {
        return BigFloat::from_f64(0.0, nwords * 64);
    }
    let Some(e) = f.get_exp() else {
        return BigFloat::from_f64(0.0, nwords * 64);
    };
    let nb = (nwords * 64) as i32;
    let mut g = Float::with_val(f.prec(), f.abs_ref());
    g <<= nb - e; // now exactly an integer in [2^(nb−1), 2^nb)
    let Some(int) = g.to_integer() else {
        return BigFloat::from_f64(0.0, nwords * 64);
    };
    let mut w = int.to_digits::<u64>(Order::Lsf);
    w.resize(nwords, 0);
    BigFloat::from_words(&w, if f.is_sign_negative() { Sign::Neg } else { Sign::Pos }, e)
}

/// The GMP / MPFR versions actually linked, for the backend stamp. These are C libraries built at
/// compile time, so the version that matters is the one in the binary — not one named in a manifest.
pub(crate) fn linked_versions() -> String {
    let mpfr = unsafe {
        std::ffi::CStr::from_ptr(gmp_mpfr_sys::mpfr::VERSION_STRING).to_string_lossy().into_owned()
    };
    format!(
        "rug/MPFR {mpfr} + GMP {}.{}.{}",
        gmp_mpfr_sys::gmp::VERSION,
        gmp_mpfr_sys::gmp::VERSION_MINOR,
        gmp_mpfr_sys::gmp::VERSION_PATCHLEVEL
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic full-mantissa value: every limb populated, so multiplies do real carry work.
    fn sample(seed: u64, nwords: usize, exp: i32, neg: bool) -> BigFloat {
        let mut s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        let mut w = vec![0u64; nwords];
        for slot in w.iter_mut() {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            *slot = s;
        }
        *w.last_mut().unwrap() |= 1 << 63;
        BigFloat::from_words(&w, if neg { Sign::Neg } else { Sign::Pos }, exp)
    }

    fn canon(v: &BigFloat) -> (Vec<u64>, i32, bool) {
        let (w, _n, s, e, _) = v.as_raw_parts().expect("finite");
        if w.iter().all(|&x| x == 0) {
            return (vec![0; w.len()], 0, false);
        }
        (w.to_vec(), e, matches!(s, Sign::Neg))
    }

    #[test]
    fn conversion_round_trips_exactly_in_both_directions() {
        for p in [64usize, 128, 576, 2112] {
            let ctx = <Float as RefBackend>::ctx_for(p);
            for trial in 0..24u64 {
                for (exp, neg) in [(0, false), (-3, true), (77, false), (-200, true)] {
                    let v = sample(trial, p.div_ceil(64), exp, neg);
                    let back = <Float as RefBackend>::from_carrier(&v, ctx).to_carrier(ctx);
                    assert_eq!(canon(&v), canon(&back), "round trip at p={p} exp={exp}");
                }
            }
            // Zero must survive too -- it arises on the very first reference iteration.
            let z = BigFloat::from_f64(0.0, p);
            let zb = <Float as RefBackend>::from_carrier(&z, ctx).to_carrier(ctx);
            assert!(zb.is_zero(), "zero round trip at p={p}");
        }
    }

    #[test]
    fn the_three_preconditions_hold() {
        // 2: word-granular precision.
        assert_eq!(<Float as RefBackend>::ctx_for(1), 64);
        assert_eq!(<Float as RefBackend>::ctx_for(64), 64);
        assert_eq!(<Float as RefBackend>::ctx_for(65), 128);
        assert_eq!(<Float as RefBackend>::ctx_for(576), 576);

        // 1 + 3: arithmetic and f64 extraction agree with astro-float bit for bit.
        let p = 576;
        let ctx = <Float as RefBackend>::ctx_for(p);
        for trial in 0..32u64 {
            let a = sample(trial * 2 + 1, p / 64, 0, false);
            let b = sample(trial * 2 + 2, p / 64, -3, trial % 2 == 0);
            let (ra, rb) = (astro_to_rug(&a, ctx), astro_to_rug(&b, ctx));
            for (name, av, rv) in [
                ("mul", a.mul(&b, p, crate::bignum::RM), ra.fmul(&rb, ctx)),
                ("add", a.add(&b, p, crate::bignum::RM), ra.fadd(&rb, ctx)),
                ("sub", a.sub(&b, p, crate::bignum::RM), ra.fsub(&rb, ctx)),
            ] {
                assert_eq!(canon(&av), canon(&rv.to_carrier(ctx)), "{name} differs at trial {trial}");
                assert_eq!(
                    crate::to_f64(&av),
                    rv.to_f64_trunc(),
                    "{name}: f64 extraction differs at trial {trial}"
                );
            }
        }
    }

    /// Regression test for the defect the byte-identity matrix caught: an operand WIDER than the
    /// working precision must be handed to MPFR intact, because astro-float rounds the result, not
    /// the inputs. `parse_bf_prec` produces exactly this whenever a coordinate's literal carries
    /// more digits than the current zoom needs.
    #[test]
    fn an_operand_wider_than_the_working_precision_is_not_truncated_first() {
        let p = 64; // one word of working precision...
        let ctx = <Float as RefBackend>::ctx_for(p);
        // ...and a 33-digit literal, which `parse_bf_prec` returns at its natural TWO words.
        let wide = crate::parse_bf_prec("-0.743643887037158704752191506114774", p).unwrap();
        assert!(
            wide.mantissa_digits().map(|d| d.len()).unwrap_or(0) > p / 64,
            "this test needs a carrier value wider than p, or it proves nothing"
        );

        let narrow = sample(7, p / 64, 0, false);
        let (rw, rn) = (
            <Float as RefBackend>::from_carrier(&wide, ctx),
            <Float as RefBackend>::from_carrier(&narrow, ctx),
        );
        for (name, av, rv) in [
            ("add", wide.add(&narrow, p, crate::bignum::RM), rw.fadd(&rn, ctx)),
            ("sub", wide.sub(&narrow, p, crate::bignum::RM), rw.fsub(&rn, ctx)),
            ("mul", wide.mul(&narrow, p, crate::bignum::RM), rw.fmul(&rn, ctx)),
        ] {
            assert_eq!(canon(&av), canon(&rv.to_carrier(ctx)), "{name} with a wide operand");
        }
    }

    /// The negative control for precondition 1: MPFR's *default* rounding must NOT match, or the
    /// test above proves nothing about `Round::Zero` being the reason.
    #[test]
    fn round_to_nearest_would_break_bit_identity() {
        let p = 576;
        let ctx = <Float as RefBackend>::ctx_for(p);
        let mut differed = 0;
        for trial in 0..64u64 {
            let a = sample(trial * 2 + 1, p / 64, 0, false);
            let b = sample(trial * 2 + 2, p / 64, 0, false);
            let (ra, rb) = (astro_to_rug(&a, ctx), astro_to_rug(&b, ctx));
            let truncated = a.mul(&b, p, crate::bignum::RM);
            let nearest = Float::with_val_round(ctx, &ra * &rb, Round::Nearest).0;
            if canon(&truncated) != canon(&nearest.to_carrier(ctx)) {
                differed += 1;
            }
        }
        assert!(
            differed > 0,
            "round-to-nearest matched truncation on every trial — this test cannot detect a \
             rounding-mode regression, so precondition 1 is not actually being verified"
        );
    }
}
