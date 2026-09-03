//! The **MPFR** reference-orbit backend (`rug`), behind the off-by-default `rug` feature.
//!
//! Exists to make the deep-frame hot loop faster: the reference-orbit build is `max_iter × step`
//! in bignum and dominates a deep frame, and astro-float exposes no destination-reuse arithmetic
//! at all — every `add`/`sub`/`mul` allocates, which at low precision costs more than the
//! arithmetic does.
//!
//! **Measured through `reference_orbit_in` — the shipped path, both backends in one process:**
//! `1.39× at 4 limbs rising to 4.06× at 129 limbs` (this machine, 2026-08-26). The gain grows with
//! precision because that is where the multiply algorithm dominates; at shallow precision the
//! per-iteration costs this backend does not touch (`pack_sample`, the `Vec` push, the sample
//! conversion) set a floor on what any backend swap can win.
//!
//! ⚠A synthetic loop measuring only the arithmetic reports **3.2–4.7×** for the same code. That
//! number is real but not what a frame gets, and quoting it would overstate the feature by up to
//! 2.3× at the shallow end. Re-measure through the engine, never through a kernel benchmark.
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
        // ⚠This runs TWICE PER ITERATION on the hot path, so it must not allocate. Both obvious
        // spellings do: cloning at `self.prec()` then `set_prec_round(53)` builds a full-precision
        // temporary, and `Float::with_val_round(53, …)` builds a small one. The real-engine A/B
        // showed that cost swamping the backend difference at shallow precision — 1.21× at 64 bits
        // against 1.71× for the same arithmetic measured without it. `mpfr_get_d` takes the
        // rounding mode directly and allocates nothing; `rug::Float::to_f64` is the RNDN spelling
        // of this same call.
        //
        // The two guards below are not defensive padding — they reproduce `crate::to_f64`'s exact
        // behaviour at the edges, which is what precondition 3 actually requires:
        //   * it returns `0.0` for a value carrying no mantissa, i.e. for BOTH infinity and NaN;
        //   * it returns a POSITIVE literal `0.0` when a value underflows, dropping the sign — so
        //     a `-0.0` escaping from here would reach `pack_sample`, which CAN see the difference
        //     (`split_df64(-0.0)` and `split_df64(0.0)` have different bits).
        if !self.is_finite() {
            return 0.0;
        }
        // SAFETY: `as_raw` yields a pointer to this value's initialized `mpfr_t`, valid for the
        // borrow, and `mpfr_get_d` only reads through it.
        let v = unsafe {
            gmp_mpfr_sys::mpfr::get_d(self.as_raw(), gmp_mpfr_sys::mpfr::rnd_t::RNDZ)
        };
        if v == 0.0 {
            0.0 // collapses -0.0 as well, matching `crate::to_f64`
        } else {
            v
        }
    }

    fn to_floatexp(&self) -> crate::floatexp::FloatExp {
        use crate::floatexp::FloatExp;
        if self.is_zero() {
            return FloatExp::ZERO;
        }
        // The top 64 significand bits by truncation + the binary exponent, matching
        // `bf_to_floatexp` exactly: MPFR keeps the significand normalized (top bit of the top
        // limb set, value = 0.b₁b₂… × 2^exp, the same convention astro-float uses), limbs are
        // 64-bit on this target, and the limb count is prec/64 exactly because `ctx_for`
        // word-rounds the precision — so the top limb IS astro-float's normalized MSW for the
        // identical value, and the shared `u64 as f64` rounding finishes the shared recipe.
        // Pinned bitwise by `the_pick_scoring_walk_is_backend_identical`.
        //
        // SAFETY: `as_raw` yields a pointer to this value's initialized `mpfr_t`, valid for the
        // borrow; a non-zero MPFR value always has its full complement of limbs allocated.
        let (msw, exp) = unsafe {
            let raw = self.as_raw();
            let limb_bits = 8 * std::mem::size_of::<gmp_mpfr_sys::gmp::limb_t>();
            let limbs = ((*raw).prec as usize).div_ceil(limb_bits);
            (*(*raw).d.as_ptr().add(limbs - 1) as u64, (*raw).exp)
        };
        let m = (msw as f64) / 18446744073709551616.0; // ÷2^64 — m ∈ [0.5, 1)
        let m = if self.is_sign_negative() { -m } else { m };
        FloatExp::new(m, exp as i32)
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

    /// The in-place loop's scratch buffers must stay at the working precision even though the
    /// orbit ENTERS at the carrier's own, wider width.
    ///
    /// `parse_bf_prec("0", 64)` returns a TWO-word zero, so `zx` starts at 128 bits; swapping it
    /// into a scratch buffer used to hand that width to every later destination, rounding to 128
    /// bits where astro-float rounds to 64. The cross-backend matrix caught it (99 of 1800 cases,
    /// every one at p=64); this pins it where the fix lives.
    #[test]
    fn the_inplace_loop_keeps_its_scratch_at_the_working_precision() {
        let p = 64usize;
        let z0 = crate::parse_bf_prec("0", p).unwrap();
        assert!(
            z0.mantissa_digits().map(|d| d.len()).unwrap_or(0) > p / 64,
            "this test needs an entry value wider than the working precision to mean anything"
        );
        let cx = crate::parse_bf_prec("0.4", p).unwrap();
        let cy = crate::parse_bf_prec("0.4", p).unwrap();

        let (want, _, _) = crate::reference_orbit_t_in(
            crate::BackendChoice::Astro, &z0, &z0, &cx, &cy, crate::formula::MANDELBROT, 12, p,
        );
        let mut got = vec![want[0]];
        let tail = super::try_run_orbit_inplace(
            &mut got, &z0, &z0, &cx, &cy, crate::formula::MANDELBROT, 0, 12, p,
        )
        .expect("Mandelbrot must take the in-place path");
        assert_eq!(want.len(), got.len());
        for (i, (a, b)) in want.iter().zip(&got).enumerate() {
            assert_eq!(
                a.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                b.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                "sample Z_{i} differs"
            );
        }
        assert!(!tail.2, "the test orbit must not escape");
    }

    /// Precondition 3 at the EDGES, where the hot-path conversion is easiest to get subtly wrong:
    /// underflow must yield POSITIVE zero (astro-float drops the sign there), and a non-finite
    /// value must yield `0.0` (astro-float's `to_f64` returns that for anything with no mantissa,
    /// infinities included). Neither case arises in a healthy orbit, which is exactly why they
    /// need a test rather than a comment.
    #[test]
    fn f64_extraction_matches_astro_float_at_the_edges() {
        let ctx = <Float as RefBackend>::ctx_for(128);
        // A negative value far below f64's range: astro truncates it to +0.0, and a raw MPFR
        // RNDZ conversion would hand back -0.0.
        let tiny = Float::with_val(ctx, -1.0) >> 5000i32;
        assert!(!tiny.is_zero(), "the test value must be nonzero to exercise underflow");
        assert_eq!(tiny.to_f64_trunc().to_bits(), 0.0f64.to_bits(), "underflow must give +0.0");

        for nf in [Float::with_val(ctx, f64::INFINITY), Float::with_val(ctx, f64::NAN)] {
            assert_eq!(nf.to_f64_trunc(), 0.0, "non-finite must match astro-float's 0.0");
        }

        // And ordinary values must still truncate rather than round to nearest.
        for trial in 0..32u64 {
            let v = sample(trial, 2, 0, trial % 2 == 0);
            let r = <Float as RefBackend>::from_carrier(&v, ctx);
            assert_eq!(crate::to_f64(&v).to_bits(), r.to_f64_trunc().to_bits(), "trial {trial}");
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

/// In-place orbit loop for the **squaring families** — no allocation inside the loop at all.
///
/// astro-float has no destination-reuse arithmetic, so the generic loop allocates a fresh value per
/// operation; MPFR does, and at shallow precision that allocation traffic is most of the cost.
/// Measured in the kernel probe: 438 ns/iteration allocating against 160 ns reusing, at 2 limbs.
///
/// `None` for any formula this does not implement, and the caller falls back to the generic loop —
/// which is still this backend, just allocating. Multibrot 3/4/5 need extra products, Phoenix needs
/// the previous iterate, and Newton never reaches here.
///
/// ⚠**The operation order below is `crate::fractal`'s `csqr` plus each family's arm, verbatim.**
/// This is a second copy of arithmetic that is authored once elsewhere, which is exactly the drift
/// risk the `Field`-generic step exists to remove — so it is admissible only because
/// `the_mpfr_backend_is_byte_identical_to_astro_float` compares this path against astro-float
/// across every formula id, point and precision, and that test is known to be able to go red.
#[allow(clippy::too_many_arguments)]
pub(crate) fn try_run_orbit_inplace(
    out: &mut Vec<[f32; 4]>,
    z0x: &BigFloat,
    z0y: &BigFloat,
    cx: &BigFloat,
    cy: &BigFloat,
    formula: u32,
    mut n: u32,
    max_iter: u32,
    p: usize,
) -> Option<(BigFloat, BigFloat, bool)> {
    use crate::formula as fam;
    use rug::ops::{AddAssignRound, AssignRound, SubAssignRound, SubFromRound};

    if !matches!(
        formula,
        fam::MANDELBROT | fam::TRICORN | fam::BURNING_SHIP | fam::CELTIC | fam::BUFFALO
    ) {
        return None;
    }

    let ctx = <Float as RefBackend>::ctx_for(p);
    let mut zx = <Float as RefBackend>::from_carrier(z0x, ctx);
    let mut zy = <Float as RefBackend>::from_carrier(z0y, ctx);
    let rcx = <Float as RefBackend>::from_carrier(cx, ctx);
    let rcy = <Float as RefBackend>::from_carrier(cy, ctx);

    // The whole point: allocated once, reused for every iteration.
    let mut x2 = Float::with_val(ctx, 0);
    let mut y2 = Float::with_val(ctx, 0);
    let mut t = Float::with_val(ctx, 0);

    let mut escaped = false;
    while n < max_iter {
        // z² — `csqr`'s order: each product rounded to `ctx`, then combined; the imaginary part
        // doubled by an exponent bump rather than a second multiply.
        x2.assign_round(&zx * &zx, RZ);
        y2.assign_round(&zy * &zy, RZ);
        t.assign_round(&zx * &zy, RZ);
        t <<= 1; // exact
        x2.sub_assign_round(&y2, RZ); // Re(z²)

        match formula {
            fam::MANDELBROT => {
                x2.add_assign_round(&rcx, RZ);
                t.add_assign_round(&rcy, RZ);
            }
            // `cy − Im(z²)`, not `−Im(z²) + cy`: the generic arm is written that way so BigFloat
            // matches the pre-trait `cy.sub(&txy)` exactly, and this must match the generic arm.
            fam::TRICORN => {
                x2.add_assign_round(&rcx, RZ);
                t.sub_from_round(&rcy, RZ);
            }
            fam::BURNING_SHIP => {
                x2.add_assign_round(&rcx, RZ);
                t.abs_mut(); // exact; abs BEFORE the add, as in the generic arm
                t.add_assign_round(&rcy, RZ);
            }
            fam::CELTIC => {
                x2.abs_mut();
                x2.add_assign_round(&rcx, RZ);
                t.add_assign_round(&rcy, RZ);
            }
            _ => {
                // BUFFALO — abs on both parts.
                x2.abs_mut();
                x2.add_assign_round(&rcx, RZ);
                t.abs_mut();
                t.add_assign_round(&rcy, RZ);
            }
        }
        core::mem::swap(&mut zx, &mut x2);
        core::mem::swap(&mut zy, &mut t);
        // ⚠The scratch buffers must stay at exactly `ctx`, and the swap can take that away.
        //
        // `zx`/`zy` enter at the CARRIER's own width, which is legitimately wider than `ctx` --
        // `parse_bf_prec("0", 64)` returns a TWO-word zero, and a pasted deep coordinate is wider
        // still. The generic loop is immune because every op allocates a fresh value at `ctx`, so
        // an operand's width never reaches a destination. Swapping does exactly that: after the
        // first iteration the scratch would carry the entry width, and every later `assign_round`
        // would round to 128 bits where astro-float rounds to 64.
        //
        // The first iteration legitimately uses the wide entry values as OPERANDS (as the generic
        // path does); only the destinations must be pinned. The guard costs one comparison per
        // iteration and fires at most once, since everything is `ctx` from then on.
        if x2.prec() != ctx {
            x2.set_prec(ctx); // content is dead -- it is overwritten at the top of the next pass
        }
        if t.prec() != ctx {
            t.set_prec(ctx);
        }

        let xv = zx.to_f64_trunc();
        let yv = zy.to_f64_trunc();
        out.push(crate::reference::pack_sample(xv, yv));
        n += 1;
        if xv * xv + yv * yv > 1.0e12 {
            escaped = true;
            break;
        }
    }
    Some((zx.to_carrier(ctx), zy.to_carrier(ctx), escaped))
}

/// Length-only / sample-recording twin of [`try_run_orbit_inplace`] for the PICK's scoring
/// walks (`orbit_length_bf` and its recording variant): the same preallocated-temp MPFR loop,
/// the same truncating rounds, the same swap + scratch-precision pin, the same `to_f64_trunc`
/// escape view — but the count carries `orbit_length_bf`'s semantics (the escaping step is
/// included, `max_iter` is the cap), no `[f32; 4]` build samples are produced, and the optional
/// sink records the extended-range `CFloatExp` samples the perturbation scorer consumes.
/// Mandelbrot only — the family deep zoom actually walks here; every other formula takes the
/// generic (allocating, still-MPFR) path, which the identity matrix holds byte-identical.
pub(crate) fn try_orbit_length_inplace(
    z0x: &BigFloat,
    z0y: &BigFloat,
    cx: &BigFloat,
    cy: &BigFloat,
    formula: u32,
    max_iter: u32,
    p: usize,
    mut samples: Option<&mut Vec<crate::floatexp::CFloatExp>>,
) -> Option<u32> {
    use crate::floatexp::CFloatExp;
    use rug::ops::{AddAssignRound, AssignRound, SubAssignRound};

    if formula != crate::formula::MANDELBROT {
        return None;
    }
    let ctx = <Float as RefBackend>::ctx_for(p);
    let mut zx = <Float as RefBackend>::from_carrier(z0x, ctx);
    let mut zy = <Float as RefBackend>::from_carrier(z0y, ctx);
    let rcx = <Float as RefBackend>::from_carrier(cx, ctx);
    let rcy = <Float as RefBackend>::from_carrier(cy, ctx);

    let mut x2 = Float::with_val(ctx, 0);
    let mut y2 = Float::with_val(ctx, 0);
    let mut t = Float::with_val(ctx, 0);

    if let Some(s) = samples.as_deref_mut() {
        s.push(CFloatExp {
            re: RefBackend::to_floatexp(&zx),
            im: RefBackend::to_floatexp(&zy),
        });
    }
    let mut n = 0u32;
    while n < max_iter {
        // z² + c, in `csqr`'s exact op order (see try_run_orbit_inplace).
        x2.assign_round(&zx * &zx, RZ);
        y2.assign_round(&zy * &zy, RZ);
        t.assign_round(&zx * &zy, RZ);
        t <<= 1; // exact
        x2.sub_assign_round(&y2, RZ); // Re(z²)
        x2.add_assign_round(&rcx, RZ);
        t.add_assign_round(&rcy, RZ);
        core::mem::swap(&mut zx, &mut x2);
        core::mem::swap(&mut zy, &mut t);
        // Scratch must stay at exactly `ctx` — the swap can hand it the (wider) entry width.
        // Same guard, same reasoning as try_run_orbit_inplace.
        if x2.prec() != ctx {
            x2.set_prec(ctx);
        }
        if t.prec() != ctx {
            t.set_prec(ctx);
        }
        n += 1;
        if let Some(s) = samples.as_deref_mut() {
            s.push(CFloatExp {
                re: RefBackend::to_floatexp(&zx),
                im: RefBackend::to_floatexp(&zy),
            });
        }
        let xv = zx.to_f64_trunc();
        let yv = zy.to_f64_trunc();
        if xv * xv + yv * yv > 1.0e12 {
            break;
        }
    }
    Some(n)
}



/// The series-approximation coefficient walk in MPFR — the twin of `series_skip_astro`
/// (fractadyne-core `reference.rs`), Mandelbrot (`d = 2`) only; `None` = not handled here,
/// and the caller falls through to the astro walk (correct arithmetic for every family —
/// the fallback costs speed, never bits).
///
/// Mirrored LITERALLY, operation for operation, at the same word-rounded precision with the
/// same truncate-toward-zero rounding:
///   * `cmul` is `cmul_bf`'s sequence — 4 rounded muls, a rounded sub and a rounded add, in
///     the same operand order;
///   * the d = 2 recurrence factors are 1 and 2, and astro's `mul_u32_bf` applies those as
///     the identity and one exact doubling — so this twin needs no shift-and-add mirror at
///     all (`fdouble` is the exact counterpart of `double_bf`);
///   * the orbit advances through the SAME generic step (`step_gen::<Float>`), already held
///     byte-identical by the orbit matrix;
///   * validity reads exponents by the same max-of-components rule, reproducing astro's
///     `Some(0)`-for-zero convention (a zero early `C` coefficient reads exponent 0, not −∞);
///   * the escape view is the truncating `to_f64_trunc`, the twin of `to_f64`.
///
/// Returns the walk's `best` with the six final coefficients carried back to `BigFloat`
/// EXACTLY (`to_carrier`), so the shared tail in `reference` owns the one conversion to GPU
/// values. Pinned bitwise by `the_sa_walk_is_backend_identical`.
pub(crate) fn try_series_skip_walk(
    cx: &BigFloat,
    cy: &BigFloat,
    log2_max_dc: f64,
    limit: u32,
    formula: u32,
    p: usize,
) -> Option<Option<(u32, [BigFloat; 6])>> {
    use crate::fractal::Field;

    if formula != crate::formula::MANDELBROT {
        return None;
    }

    /// `cmul_bf`'s exact operation sequence on MPFR values.
    fn cmul(ax: &Float, ay: &Float, bx: &Float, by: &Float, p: u32) -> (Float, Float) {
        let rx = ax.fmul(bx, p).fsub(&ay.fmul(by, p), p);
        let ry = ax.fmul(by, p).fadd(&ay.fmul(bx, p), p);
        (rx, ry)
    }

    /// `log2_cmag`'s rule on MPFR values: max component exponent, with astro-float's
    /// conventions — zero reads exponent 0 (astro `exponent()` is `Some(0)` for zero), and
    /// only a value with no exponent at all (NaN/∞) contributes `None`.
    fn log2_cmag_rug(re: &Float, im: &Float) -> f64 {
        let e = |v: &Float| -> Option<i64> {
            if v.is_zero() {
                Some(0)
            } else {
                v.get_exp().map(|x| x as i64)
            }
        };
        match (e(re), e(im)) {
            (None, None) => f64::NEG_INFINITY,
            (a, b) => a.unwrap_or(i64::MIN).max(b.unwrap_or(i64::MIN)) as f64,
        }
    }

    let ctx = <Float as RefBackend>::ctx_for(p);
    let one = Float::with_val(ctx, 1u32);
    let zero = || Float::with_val(ctx, 0u32);
    let rcx = <Float as RefBackend>::from_carrier(cx, ctx);
    let rcy = <Float as RefBackend>::from_carrier(cy, ctx);
    let (mut zx, mut zy) = (zero(), zero());
    let (mut ax, mut ay) = (zero(), zero());
    let (mut bx, mut by) = (zero(), zero());
    let (mut cxx, mut cyy) = (zero(), zero());
    let mut best: Option<(u32, [Float; 6])> = None;
    for n in 1..=limit {
        // For d = 2: Z^{d-1} = Z itself (astro's `cpow_bf(z, 1)` is an exact clone) and the
        // Z^{d-2} factor is the identity, so the recurrence collapses to the lines below.
        let (a2x, a2y) = cmul(&ax, &ay, &ax, &ay, ctx); // A²
        let (abx, aby) = cmul(&ax, &ay, &bx, &by, ctx); // A·B
        // A' = 2·(Z·A) + 1
        let (t, u) = cmul(&zx, &zy, &ax, &ay, ctx);
        let na_x = t.fdouble().fadd(&one, ctx);
        let na_y = u.fdouble();
        // B' = 2·(Z·B) + A²    (C(2,2)·… — the ×1 is the identity in `mul_u32_bf` too)
        let (t, u) = cmul(&zx, &zy, &bx, &by, ctx);
        let nb_x = t.fdouble().fadd(&a2x, ctx);
        let nb_y = u.fdouble().fadd(&a2y, ctx);
        // C' = 2·(Z·C) + 2·(A·B)    (C(2,3) = 0 — no third term)
        let (t, u) = cmul(&zx, &zy, &cxx, &cyy, ctx);
        let nc_x = t.fdouble().fadd(&abx.fdouble(), ctx);
        let nc_y = u.fdouble().fadd(&aby.fdouble(), ctx);
        // Advance the reference through the shared generic step (byte-identical by the
        // orbit matrix), then swap — the same order as the astro loop.
        let (nzx, nzy) = crate::reference::step_gen::<Float>(&zx, &zy, &rcx, &rcy, formula, ctx);
        zx = nzx;
        zy = nzy;
        ax = na_x;
        ay = na_y;
        bx = nb_x;
        by = nb_y;
        cxx = nc_x;
        cyy = nc_y;
        let la = log2_cmag_rug(&ax, &ay);
        let lc = log2_cmag_rug(&cxx, &cyy);
        if !la.is_finite() {
            continue;
        }
        let valid = lc + 2.0 * log2_max_dc < la + crate::reference::SA_EPS_LOG2;
        if n >= crate::reference::SA_MIN_SKIP {
            if valid {
                best = Some((
                    n,
                    [ax.clone(), ay.clone(), bx.clone(), by.clone(), cxx.clone(), cyy.clone()],
                ));
            } else {
                break; // coefficients only grow ⇒ once invalid, stays invalid
            }
        }
        // Stop if the reference itself escaped (truncating f64 view, like `to_f64`).
        let (fx, fy) = (zx.to_f64_trunc(), zy.to_f64_trunc());
        if fx * fx + fy * fy > 1.0e12 {
            break;
        }
    }
    Some(best.map(|(n, k)| {
        (
            n,
            [
                k[0].to_carrier(ctx),
                k[1].to_carrier(ctx),
                k[2].to_carrier(ctx),
                k[3].to_carrier(ctx),
                k[4].to_carrier(ctx),
                k[5].to_carrier(ctx),
            ],
        )
    }))
}
