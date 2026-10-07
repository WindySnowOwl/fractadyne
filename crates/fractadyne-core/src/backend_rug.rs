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

    /// The finder's MPFR passes reproduce astro-float BIT FOR BIT: the Newton pass (`Z`, `dZ/dc`)
    /// and the atom-size pass against the astro recurrences (`cmul_bf`, `step_bf`, exact doubling,
    /// `cinv_bf`'s order), at the period-998 seahorse point. A finder that answered differently per
    /// backend would hand a benchmark a different nucleus depending on the build.
    #[test]
    fn the_finder_passes_are_bit_identical_to_astro_float() {
        // 320 bits runs the one-core passes; 17,000 is above `PAR_MIN_BITS`, where the Newton pass
        // runs its seven products on seven cores — the results must not move either way.
        for p in [320usize, 17_000] {
            finder_passes_identical_at(p);
        }
    }

    fn finder_passes_identical_at(p: usize) {
        use crate::reference::{cmul_bf, step_bf};
        use crate::bignum::RM;
        let cx = crate::parse_bf_prec("-0.7436438870371587", p).unwrap();
        let cy = crate::parse_bf_prec("0.1318259042053122", p).unwrap();
        let bf = |v: f64| BigFloat::from_f64(v, p);
        let bits = |v: &BigFloat| canon(v);
        // Newton pass, 998 steps.
        let (mut zx, mut zy, mut dx, mut dy) = (bf(0.0), bf(0.0), bf(0.0), bf(0.0));
        let (one, two) = (bf(1.0), bf(2.0));
        for _ in 0..998 {
            let (mx, my) = cmul_bf(&zx, &zy, &dx, &dy, p);
            let ndx = mx.mul(&two, p, RM).add(&one, p, RM);
            let ndy = my.mul(&two, p, RM);
            let (nx, ny) = step_bf(&zx, &zy, &cx, &cy, 0, p);
            (zx, zy, dx, dy) = (nx, ny, ndx, ndy);
        }
        let [rzx, rzy, rdx, rdy] = zd_pass(&cx, &cy, 998, p);
        assert_eq!(
            [bits(&rzx), bits(&rzy), bits(&rdx), bits(&rdy)],
            [bits(&zx), bits(&zy), bits(&dx), bits(&dy)],
            "Newton pass"
        );
        // Atom-size pass, 997 steps.
        let (mut zx, mut zy) = (bf(0.0), bf(0.0));
        let (mut lx, mut ly, mut bx, mut by) = (bf(1.0), bf(0.0), bf(1.0), bf(0.0));
        for _ in 1..998 {
            (zx, zy) = step_bf(&zx, &zy, &cx, &cy, 0, p);
            let (mx, my) = cmul_bf(&zx, &zy, &lx, &ly, p);
            lx = mx.add(&mx, p, RM);
            ly = my.add(&my, p, RM);
            let d = lx.mul(&lx, p, RM).add(&ly.mul(&ly, p, RM), p, RM);
            bx = bx.add(&lx.div(&d, p, RM), p, RM);
            by = by.add(&bf(0.0).sub(&ly, p, RM).div(&d, p, RM), p, RM);
        }
        let [rlx, rly, rbx, rby] = atom_size_pass(&cx, &cy, 998, p).expect("pass");
        assert_eq!(
            [bits(&rlx), bits(&rly), bits(&rbx), bits(&rby)],
            [bits(&lx), bits(&ly), bits(&bx), bits(&by)],
            "atom-size pass"
        );
    }

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
        // The same schedule state the fresh build above started from (on or off with the switch).
        let sched = crate::reference::SchedState::fresh(p, crate::formula::MANDELBROT, true);
        let tail = super::try_run_orbit_inplace(
            &mut got, &z0, &z0, &cx, &cy, crate::formula::MANDELBROT, 0, 12, p, None, sched, None,
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

// ---- Products across cores --------------------------------------------------------------------
//
// A deep orbit step is a handful of INDEPENDENT full-width products: `z² + c` is x², y² and x·y,
// and the finder's Newton step adds the four products of z·dz/dc. At 100,000 bits one product
// costs ~0.15 ms and the step is strictly sequential from one to the next, so the only
// parallelism is inside the step: each product on its own core. A worker owns a fixed list of
// `inputs[a] × inputs[b]` products, computed with the very same rounded MPFR multiply the
// sequential loop does — the results are byte-identical, only the wall time changes. Workers spin
// between steps (a sleeping thread takes tens of microseconds to wake on Windows, a spinning one
// well under one); the build is the only thing those cores are doing.

/// Below this working width splitting does not pay. In isolation (`par_crossover` in the tests) the
/// crossover is between 4,096 bits (0.85x) and 8,192 (1.2x); in the app it is higher, because a
/// render's build runs BESIDE the pick's walk of the same orbit and its all-core phase 1, and the
/// workers then share cores: measured on a `--render` (kit lane, two reps each), the 1e3000
/// minibrot at 10,158 bits built its orbit in 0.9-1.1 s split against 0.6-0.8 s not, while the
/// 1e10000 one at 33,292 bits took 9.2-10.1 s against 13.8-14.2 s. `FRACTADYNE_ORBIT_THREADS=1`
/// turns the split off (A/B runs); `FRACTADYNE_ORBIT_PAR_MIN_BITS` moves the threshold.
const PAR_MIN_BITS: u32 = 16_384;

fn par_enabled(ctx: u32) -> bool {
    static CFG: std::sync::OnceLock<(bool, u32)> = std::sync::OnceLock::new();
    let (off, min) = *CFG.get_or_init(|| {
        let off = std::env::var("FRACTADYNE_ORBIT_THREADS").is_ok_and(|v| v.trim() == "1");
        let min = std::env::var("FRACTADYNE_ORBIT_PAR_MIN_BITS")
            .ok()
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(PAR_MIN_BITS);
        (off, min)
    });
    !off && ctx >= min && !crate::backend::sequential_products()
}

fn spin_until(mut ready: impl FnMut() -> bool) {
    let mut spins = 0u32;
    while !ready() {
        spins = spins.wrapping_add(1);
        if spins < 1 << 14 {
            std::hint::spin_loop();
        } else {
            std::thread::yield_now(); // oversubscribed: give the core back rather than starve
        }
    }
}

/// The step's inputs, each worker's products, and the step handshake.
struct ParProducts {
    inputs: std::sync::RwLock<Vec<Float>>,
    /// `outs[w][k]` = worker `w`'s `k`-th product; worker 0 is the calling thread.
    outs: Vec<std::sync::Mutex<Vec<Float>>>,
    jobs: Vec<Vec<(usize, usize)>>,
    gen: std::sync::atomic::AtomicU64,
    done: std::sync::atomic::AtomicU64,
    stop: std::sync::atomic::AtomicBool,
}

impl ParProducts {
    fn new(inputs: Vec<Float>, jobs: Vec<Vec<(usize, usize)>>, ctx: u32) -> Self {
        let outs = jobs
            .iter()
            .map(|j| std::sync::Mutex::new(j.iter().map(|_| Float::with_val(ctx, 0)).collect()))
            .collect();
        ParProducts {
            inputs: std::sync::RwLock::new(inputs),
            outs,
            jobs,
            gen: std::sync::atomic::AtomicU64::new(0),
            done: std::sync::atomic::AtomicU64::new(0),
            stop: std::sync::atomic::AtomicBool::new(false),
        }
    }

    fn compute(&self, w: usize) {
        use rug::ops::AssignRound;
        let inp = self.inputs.read().expect("inputs");
        let mut out = self.outs[w].lock().expect("outputs");
        for (k, &(a, b)) in self.jobs[w].iter().enumerate() {
            out[k].assign_round(&inp[a] * &inp[b], RZ);
        }
    }

    /// One step's products: this thread computes worker 0's share, then waits for the rest.
    fn step(&self) {
        use std::sync::atomic::Ordering::{AcqRel, Acquire};
        let g = self.gen.fetch_add(1, AcqRel) + 1;
        self.compute(0);
        let want = g * (self.jobs.len() as u64 - 1);
        spin_until(|| self.done.load(Acquire) >= want);
    }

    fn worker(&self, w: usize) {
        use std::sync::atomic::Ordering::{AcqRel, Acquire};
        let mut seen = 0u64;
        loop {
            spin_until(|| self.gen.load(Acquire) != seen || self.stop.load(Acquire));
            if self.stop.load(Acquire) {
                return;
            }
            seen += 1; // the caller waits for every worker before the next step: exactly one
            self.compute(w);
            self.done.fetch_add(1, AcqRel);
        }
    }

    /// Run `f` with workers `1..` live on scoped threads, stopping them however `f` ends.
    fn with_workers<R>(&self, f: impl FnOnce(&Self) -> R) -> R {
        struct Stop<'a>(&'a std::sync::atomic::AtomicBool);
        impl Drop for Stop<'_> {
            fn drop(&mut self) {
                self.0.store(true, std::sync::atomic::Ordering::Release);
            }
        }
        std::thread::scope(|s| {
            for w in 1..self.jobs.len() {
                s.spawn(move || self.worker(w));
            }
            let _stop = Stop(&self.stop);
            f(self)
        })
    }

    /// After a step: the inputs to write the next values into, and every worker's products.
    fn combine<R>(
        &self,
        f: impl FnOnce(&mut Vec<Float>, &mut [std::sync::MutexGuard<'_, Vec<Float>>]) -> R,
    ) -> R {
        let mut outs: Vec<_> = self.outs.iter().map(|m| m.lock().expect("outputs")).collect();
        let mut inp = self.inputs.write().expect("inputs");
        f(&mut inp, &mut outs)
    }

    fn read<R>(&self, f: impl FnOnce(&[Float]) -> R) -> R {
        f(&self.inputs.read().expect("inputs"))
    }
}

/// [`par_mandel_walk`] at one precision throughout.
fn par_mandel_walk_fixed(
    zx: Float,
    zy: Float,
    rcx: &Float,
    rcy: &Float,
    ctx: u32,
    mut each: impl FnMut(&Float, &Float) -> bool,
    steps: u32,
) -> (Float, Float) {
    par_mandel_walk(zx, zy, rcx, rcy, ctx, |x, y| (each(x, y), ctx), steps)
}

/// Move `v` into `slot`, leaving `v` as scratch pinned at `ctx` (see the swap note in
/// `try_run_orbit_inplace`: an entry value can be wider than `ctx`, and a destination must not
/// inherit that width).
fn swap_in(slot: &mut Float, v: &mut Float, ctx: u32) {
    core::mem::swap(slot, v);
    if v.prec() != ctx {
        v.set_prec(ctx);
    }
}

/// `z ← z² + c` (Mandelbrot) for `n` steps from `(zx, zy)`, the three products on three cores:
/// `x²` here, `y²` and `x·y` on two workers; the combination is `csqr`'s, op for op.
/// `each(zx, zy) -> (stop?, bits)` sees every new value and names the precision the next one is
/// computed at (`ctx` is the first's; see `crate::reference::Schedule`). Returns the final `(zx, zy)`.
fn par_mandel_walk(
    zx: Float,
    zy: Float,
    rcx: &Float,
    rcy: &Float,
    ctx: u32,
    mut each: impl FnMut(&Float, &Float) -> (bool, u32),
    steps: u32,
) -> (Float, Float) {
    use rug::ops::{AddAssignRound, SubAssignRound};
    if split_enabled(ctx) {
        return split_mandel_walk(zx, zy, rcx, rcy, ctx, each, steps);
    }
    let par = ParProducts::new(vec![zx, zy], vec![vec![(0, 0)], vec![(1, 1)], vec![(0, 1)]], ctx);
    let mut cur = ctx;
    par.with_workers(|par| {
        for _ in 0..steps {
            par.step();
            par.combine(|inp, outs| {
                let (o0, rest) = outs.split_at_mut(1);
                let (o1, o2) = rest.split_at_mut(1);
                let (x2, y2, t) = (&mut o0[0][0], &o1[0][0], &mut o2[0][0]);
                *t <<= 1; // exact
                x2.sub_assign_round(y2, RZ);
                x2.add_assign_round(rcx, RZ);
                t.add_assign_round(rcy, RZ);
                swap_in(&mut inp[0], x2, cur);
                swap_in(&mut inp[1], t, cur);
            });
            let (stop, next) = par.read(|z| each(&z[0], &z[1]));
            if stop {
                break;
            }
            if next != cur {
                // The next step's products and sums round to `next` bits.
                par.combine(|_, outs| {
                    for o in outs.iter_mut() {
                        for f in o.iter_mut() {
                            f.set_prec(next);
                        }
                    }
                });
                cur = next;
            }
        }
    });
    let mut z = par.inputs.into_inner().expect("inputs");
    let zy = z.pop().expect("zy");
    let zx = z.pop().expect("zx");
    (zx, zy)
}

// ---- Products split across cores --------------------------------------------------------------
//
// One product per core still leaves a step at the cost of its slowest product, x·y: ~140 µs at
// 100,000 bits. Wider than `SPLIT_MIN_BITS`, each product is split again. A significand
// m = h·2^k + l squares EXACTLY as h²·2^2k + ((h+l)² − h² − l²)·2^k + l² (Karatsuba; x·y likewise
// from h·h', l·l' and (h+l)(h'+l')), so the step's three products are nine half-width integer
// products on nine cores (~57 µs for the widest at 100,000 bits), then three threads each assemble
// one exact product and truncate it to the step's precision. A correctly rounded result is unique,
// so that is the very value MPFR's RZ multiply returns: the walk stays byte-identical
// (`tests/backend_rug_identity.rs`), only the wall time changes.

/// Below this working width the split does not pay. Measured in isolation, at 100,000 bits MPFR's
/// x·y takes 139 µs against 57 µs for the widest half-width product plus ~8 µs to assemble and
/// round; at 16,384 bits 9.7 µs against 4.0 + 0.8, where two extra handshakes a step eat the
/// difference. `FRACTADYNE_ORBIT_SPLIT_MIN_BITS` moves it (a huge value turns the split off).
const SPLIT_MIN_BITS: u32 = 32_768;

fn split_min_bits() -> u32 {
    static MIN: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *MIN.get_or_init(|| {
        std::env::var("FRACTADYNE_ORBIT_SPLIT_MIN_BITS")
            .ok()
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(SPLIT_MIN_BITS)
    })
}

/// Whether a walk of working width `ctx` splits its products: wide enough, and a machine with the
/// cores for it. The pick's centre walk and the build's run side by side, so that is two walks'
/// `SPLIT_THREADS`; with fewer cores the threads only take turns (measured: oversubscribed product
/// threads, 32 pool walks × 9, made the 1e10003 minibrot's reference 3.7× slower).
fn split_enabled(ctx: u32) -> bool {
    static CORES: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let cores = *CORES.get_or_init(|| {
        std::thread::available_parallelism().map_or(1, |n| n.get()) >= 2 * SPLIT_THREADS
    });
    ctx >= split_min_bits() && (cores || FORCE_SPLIT.load(std::sync::atomic::Ordering::Relaxed))
}

static FORCE_SPLIT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Tests: split wide walks whatever the core count (a CI runner has four).
#[doc(hidden)]
pub fn force_split() {
    FORCE_SPLIT.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// Steps the split has taken in this process: lets a test show the split really ran.
static SPLIT_STEPS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[doc(hidden)]
pub fn split_steps() -> u64 {
    SPLIT_STEPS.load(std::sync::atomic::Ordering::Relaxed)
}

/// Threads of a split step, the calling thread included: one per half-width product.
const SPLIT_THREADS: usize = 9;

/// A split step's shared state. Stage A: thread `w` splits the significands it needs and computes
/// half-width product `w` — 0–2 build x², 3–5 y² (h², l², (h+l)²), 6–8 x·y (h·h', l·l',
/// (h+l)(h'+l')); stage B: threads 0–2 each assemble one exact product and round it. A step too
/// narrow to split (the schedule can lower the precision mid-walk), or with a zero input, is the
/// three plain MPFR products on threads 0–2.
///
/// ⚠Measured, not assumed: each thread splitting its own operands into FRESH integers is the
/// fastest of three layouts at the 1e30000 path (3 interleaved runs each): 57.8–58.3 s, against
/// 61.6 s splitting both significands once on the calling thread and 62.4 s with every thread
/// reusing its own buffers.
struct ParSplit {
    inputs: std::sync::RwLock<Vec<Float>>,
    subs: Vec<std::sync::Mutex<Integer>>,
    /// x², y² and x·y at the step's precision.
    prods: Vec<std::sync::Mutex<Float>>,
    /// Stage B's scratch, one per assembling thread.
    scratch: Vec<std::sync::Mutex<(Integer, Integer)>>,
    /// The inputs' significand exponents (`z = m·2^e`), from stage A for stage B.
    exps: [std::sync::atomic::AtomicI32; 2],
    /// The split point `k` of this step, and whether it splits at all.
    k: std::sync::atomic::AtomicU32,
    split: std::sync::atomic::AtomicBool,
    /// 0 = stage A, 1 = stage B.
    stage: std::sync::atomic::AtomicU32,
    gen: std::sync::atomic::AtomicU64,
    done: std::sync::atomic::AtomicU64,
    stop: std::sync::atomic::AtomicBool,
}

impl ParSplit {
    fn new(zx: Float, zy: Float, ctx: u32) -> Self {
        use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU64};
        use std::sync::Mutex;
        ParSplit {
            inputs: std::sync::RwLock::new(vec![zx, zy]),
            subs: (0..SPLIT_THREADS).map(|_| Mutex::new(Integer::new())).collect(),
            prods: (0..3).map(|_| Mutex::new(Float::with_val(ctx, 0))).collect(),
            scratch: (0..3).map(|_| Mutex::new((Integer::new(), Integer::new()))).collect(),
            exps: [AtomicI32::new(0), AtomicI32::new(0)],
            k: AtomicU32::new(0),
            split: AtomicBool::new(false),
            stage: AtomicU32::new(0),
            gen: AtomicU64::new(0),
            done: AtomicU64::new(0),
            stop: AtomicBool::new(false),
        }
    }

    /// `|m|`'s halves at `k` (`h`, `l`) for `z = m·2^e`, and `e`.
    fn halves(z: &Float, k: u32) -> (Integer, Integer, i32) {
        let (m, e) = z.to_integer_exp().expect("a split input is finite and non-zero");
        let m = m.abs();
        (Integer::from(&m >> k), m.keep_bits(k), e)
    }

    fn stage_a(&self, w: usize) {
        use rug::{ops::AssignRound, Assign};
        use std::sync::atomic::Ordering::Relaxed;
        if !self.split.load(Relaxed) {
            if w < 3 {
                let inp = self.inputs.read().expect("inputs");
                let (a, b) = [(0, 0), (1, 1), (0, 1)][w];
                self.prods[w].lock().expect("products").assign_round(&inp[a] * &inp[b], RZ);
            }
            return;
        }
        let k = self.k.load(Relaxed);
        let inp = self.inputs.read().expect("inputs");
        let mut out = self.subs[w].lock().expect("half products");
        if w < 6 {
            let (h, l, e) = Self::halves(&inp[w / 3], k);
            match w % 3 {
                0 => {
                    self.exps[w / 3].store(e, Relaxed);
                    out.assign(h.square_ref());
                }
                1 => out.assign(l.square_ref()),
                _ => out.assign((h + l).square_ref()),
            }
        } else {
            let (xh, xl, _) = Self::halves(&inp[0], k);
            let (yh, yl, _) = Self::halves(&inp[1], k);
            match w {
                6 => out.assign(&xh * &yh),
                7 => out.assign(&xl * &yl),
                _ => out.assign(&(xh + xl) * &(yh + yl)),
            }
        }
    }

    /// Product `w` from its three half-width products: h·2^2k + (s − h − l)·2^k + l, its sign,
    /// truncated to the product's precision, then scaled by its exponent (exact).
    fn stage_b(&self, w: usize) {
        use rug::{ops::{AssignRound, NegAssign}, Assign};
        use std::sync::atomic::Ordering::Relaxed;
        if w >= 3 || !self.split.load(Relaxed) {
            return;
        }
        let k = self.k.load(Relaxed);
        let (h, l, s) = (
            self.subs[3 * w].lock().expect("half products"),
            self.subs[3 * w + 1].lock().expect("half products"),
            self.subs[3 * w + 2].lock().expect("half products"),
        );
        let mut scratch = self.scratch[w].lock().expect("scratch");
        let (exact, mid) = &mut *scratch;
        mid.assign(&*s - &*h);
        *mid -= &*l;
        *mid <<= k;
        exact.assign(&*h << (2 * k));
        *exact += &*mid;
        *exact += &*l;
        let (ex, ey) = (self.exps[0].load(Relaxed), self.exps[1].load(Relaxed));
        let e = match w {
            0 => 2 * ex,
            1 => 2 * ey,
            _ => {
                let inp = self.inputs.read().expect("inputs");
                if inp[0].is_sign_negative() != inp[1].is_sign_negative() {
                    exact.neg_assign();
                }
                ex + ey
            }
        };
        let mut p = self.prods[w].lock().expect("products");
        p.assign_round(&*exact, RZ);
        *p <<= e;
    }

    fn run_stage(&self, stage: u32) {
        use std::sync::atomic::Ordering::{AcqRel, Acquire, Relaxed};
        self.stage.store(stage, Relaxed);
        let g = self.gen.fetch_add(1, AcqRel) + 1;
        if stage == 0 {
            self.stage_a(0);
        } else {
            self.stage_b(0);
        }
        let want = g * (SPLIT_THREADS as u64 - 1);
        spin_until(|| self.done.load(Acquire) >= want);
    }

    /// One step's three products at the current precision `cur`.
    fn step(&self, cur: u32) {
        use std::sync::atomic::Ordering::Relaxed;
        let split = cur >= split_min_bits() && self.read(|z| !z[0].is_zero() && !z[1].is_zero());
        if split {
            let k = self.read(|z| z[0].prec().max(z[1].prec())) / 2;
            self.k.store(k, Relaxed);
            SPLIT_STEPS.fetch_add(1, Relaxed);
        }
        self.split.store(split, Relaxed);
        self.run_stage(0);
        if split {
            self.run_stage(1);
        }
    }

    fn worker(&self, w: usize) {
        use std::sync::atomic::Ordering::{AcqRel, Acquire, Relaxed};
        let mut seen = 0u64;
        loop {
            spin_until(|| self.gen.load(Acquire) != seen || self.stop.load(Acquire));
            if self.stop.load(Acquire) {
                return;
            }
            seen += 1; // the caller waits for every worker before the next stage: exactly one
            if self.stage.load(Relaxed) == 0 {
                self.stage_a(w);
            } else {
                self.stage_b(w);
            }
            self.done.fetch_add(1, AcqRel);
        }
    }

    /// Run `f` with workers `1..` live on scoped threads, stopping them however `f` ends.
    fn with_workers<R>(&self, f: impl FnOnce(&Self) -> R) -> R {
        struct Stop<'a>(&'a std::sync::atomic::AtomicBool);
        impl Drop for Stop<'_> {
            fn drop(&mut self) {
                self.0.store(true, std::sync::atomic::Ordering::Release);
            }
        }
        std::thread::scope(|s| {
            for w in 1..SPLIT_THREADS {
                s.spawn(move || self.worker(w));
            }
            let _stop = Stop(&self.stop);
            f(self)
        })
    }

    fn read<R>(&self, f: impl FnOnce(&[Float]) -> R) -> R {
        f(&self.inputs.read().expect("inputs"))
    }
}

/// [`par_mandel_walk`] with each product split across cores (see `ParSplit`); the combination
/// after the products is `par_mandel_walk`'s, op for op.
fn split_mandel_walk(
    zx: Float,
    zy: Float,
    rcx: &Float,
    rcy: &Float,
    ctx: u32,
    mut each: impl FnMut(&Float, &Float) -> (bool, u32),
    steps: u32,
) -> (Float, Float) {
    use rug::ops::{AddAssignRound, SubAssignRound};
    let par = ParSplit::new(zx, zy, ctx);
    let mut cur = ctx;
    par.with_workers(|par| {
        for _ in 0..steps {
            par.step(cur);
            {
                let mut p: Vec<_> = par.prods.iter().map(|m| m.lock().expect("products")).collect();
                let mut inp = par.inputs.write().expect("inputs");
                let (p0, rest) = p.split_at_mut(1);
                let (p1, p2) = rest.split_at_mut(1);
                let (x2, y2, t) = (&mut *p0[0], &*p1[0], &mut *p2[0]);
                *t <<= 1; // exact
                x2.sub_assign_round(y2, RZ);
                x2.add_assign_round(rcx, RZ);
                t.add_assign_round(rcy, RZ);
                swap_in(&mut inp[0], x2, cur);
                swap_in(&mut inp[1], t, cur);
            }
            let (stop, next) = par.read(|z| each(&z[0], &z[1]));
            if stop {
                break;
            }
            if next != cur {
                // The next step's products and sums round to `next` bits.
                for m in &par.prods {
                    m.lock().expect("products").set_prec(next);
                }
                cur = next;
            }
        }
    });
    let mut z = par.inputs.into_inner().expect("inputs");
    let zy = z.pop().expect("zy");
    let zx = z.pop().expect("zx");
    (zx, zy)
}

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
    mut probe: Option<&mut crate::reference::PeriodProbe<'_>>,
    sched: Option<crate::reference::SchedState>,
    mut fe: Option<&mut Vec<crate::floatexp::CFloatExp>>,
) -> Option<(BigFloat, BigFloat, bool, Option<crate::reference::SchedState>)> {
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
    // THE PRECISION SCHEDULE (`crate::reference::Schedule`): Mandelbrot only, as in the generic loop.
    let mut sched = crate::reference::Schedule::new(
        p,
        crate::to_f64(cx),
        crate::to_f64(cy),
        sched.filter(|_| formula == fam::MANDELBROT),
    );

    if formula == fam::MANDELBROT && par_enabled(ctx) && n < max_iter {
        let mut escaped = false;
        let steps = max_iter - n;
        let (fx, fy) = par_mandel_walk(
            zx,
            zy,
            &rcx,
            &rcy,
            sched.bits(),
            |zx, zy| {
                let xv = zx.to_f64_trunc();
                let yv = zy.to_f64_trunc();
                out.push(crate::reference::pack_sample(xv, yv));
                if let Some(s) = fe.as_deref_mut() {
                    s.push(crate::floatexp::CFloatExp { re: RefBackend::to_floatexp(zx), im: RefBackend::to_floatexp(zy) });
                }
                n += 1;
                crate::reference::count_reference_step(n);
                if xv * xv + yv * yv > 1.0e12 {
                    escaped = true;
                    return (true, ctx);
                }
                // Before the probe, as in the generic loop: a stopped tail is ready for the next step.
                if sched.active() {
                    let z = crate::floatexp::CFloatExp {
                        re: RefBackend::to_floatexp(zx),
                        im: RefBackend::to_floatexp(zy),
                    };
                    sched.advance(z, xv, yv);
                }
                if let Some(pr) = probe.as_deref_mut() {
                    let exact = || crate::floatexp::CFloatExp {
                        re: RefBackend::to_floatexp(zx),
                        im: RefBackend::to_floatexp(zy),
                    };
                    if pr.step(xv, yv, exact) {
                        return (true, ctx);
                    }
                }
                (false, sched.bits())
            },
            steps,
        );
        return Some((fx.to_carrier(ctx), fy.to_carrier(ctx), escaped, sched.state()));
    }

    // The whole point: allocated once, reused for every iteration.
    let mut x2 = Float::with_val(ctx, 0);
    let mut y2 = Float::with_val(ctx, 0);
    let mut t = Float::with_val(ctx, 0);

    let mut escaped = false;
    while n < max_iter {
        // The destinations at this step's bits (the schedule's, else `ctx`) — see the pin below.
        let bits = sched.bits();
        if x2.prec() != bits {
            x2.set_prec(bits); // content is dead -- it is overwritten just below
        }
        if y2.prec() != bits {
            y2.set_prec(bits);
        }
        if t.prec() != bits {
            t.set_prec(bits);
        }
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
        // path does); only the destinations must be pinned. The pin is at the top of the loop, to
        // the step's own bits: one comparison per destination per iteration, firing when the
        // swap handed a scratch the entry width or the schedule moved.

        let xv = zx.to_f64_trunc();
        let yv = zy.to_f64_trunc();
        out.push(crate::reference::pack_sample(xv, yv));
        if let Some(s) = fe.as_deref_mut() {
            s.push(crate::floatexp::CFloatExp { re: RefBackend::to_floatexp(&zx), im: RefBackend::to_floatexp(&zy) });
        }
        n += 1;
        crate::reference::count_reference_step(n);
        if xv * xv + yv * yv > 1.0e12 {
            escaped = true;
            break;
        }
        if sched.active() {
            let z = crate::floatexp::CFloatExp {
                re: RefBackend::to_floatexp(&zx),
                im: RefBackend::to_floatexp(&zy),
            };
            sched.advance(z, xv, yv);
        }
        if let Some(pr) = probe.as_deref_mut() {
            let exact = || crate::floatexp::CFloatExp {
                re: RefBackend::to_floatexp(&zx),
                im: RefBackend::to_floatexp(&zy),
            };
            if pr.step(xv, yv, exact) {
                break;
            }
        }
    }
    Some((zx.to_carrier(ctx), zy.to_carrier(ctx), escaped, sched.state()))
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
    mut probe: Option<&mut crate::reference::PeriodProbe<'_>>,
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
    // THE PRECISION SCHEDULE, as the build walks the same orbit (`crate::reference::Schedule`).
    let mut sched = crate::reference::Schedule::new(
        p,
        crate::to_f64(cx),
        crate::to_f64(cy),
        crate::reference::SchedState::fresh(p, formula, z0x.is_zero() && z0y.is_zero()),
    );

    if let Some(s) = samples.as_deref_mut() {
        s.push(CFloatExp {
            re: RefBackend::to_floatexp(&zx),
            im: RefBackend::to_floatexp(&zy),
        });
    }
    let mut n = 0u32;
    if par_enabled(ctx) && max_iter > 0 {
        // The three products on three cores (see `par_mandel_walk`); same count semantics.
        let mut periodic = false;
        par_mandel_walk(
            zx,
            zy,
            &rcx,
            &rcy,
            sched.bits(),
            |zx, zy| {
                n += 1;
                crate::reference::count_reference_step(n);
                if let Some(s) = samples.as_deref_mut() {
                    s.push(CFloatExp { re: RefBackend::to_floatexp(zx), im: RefBackend::to_floatexp(zy) });
                }
                let xv = zx.to_f64_trunc();
                let yv = zy.to_f64_trunc();
                if xv * xv + yv * yv > 1.0e12 {
                    return (true, ctx);
                }
                if sched.active() {
                    sched.advance(CFloatExp { re: RefBackend::to_floatexp(zx), im: RefBackend::to_floatexp(zy) }, xv, yv);
                }
                if let Some(pr) = probe.as_deref_mut() {
                    let exact = || CFloatExp { re: RefBackend::to_floatexp(zx), im: RefBackend::to_floatexp(zy) };
                    if pr.step(xv, yv, exact) {
                        periodic = true;
                        return (true, ctx);
                    }
                }
                (false, sched.bits())
            },
            max_iter,
        );
        return Some(if periodic { max_iter } else { n });
    }
    while n < max_iter {
        // Destinations at this step's bits — the schedule's, else `ctx`; the swap can also hand a
        // scratch the (wider) entry width. Same pin, same reasoning as try_run_orbit_inplace.
        let bits = sched.bits();
        if x2.prec() != bits {
            x2.set_prec(bits);
        }
        if y2.prec() != bits {
            y2.set_prec(bits);
        }
        if t.prec() != bits {
            t.set_prec(bits);
        }
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
        n += 1;
        crate::reference::count_reference_step(n);
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
        if sched.active() {
            sched.advance(CFloatExp { re: RefBackend::to_floatexp(&zx), im: RefBackend::to_floatexp(&zy) }, xv, yv);
        }
        if let Some(pr) = probe.as_deref_mut() {
            let exact = || CFloatExp { re: RefBackend::to_floatexp(&zx), im: RefBackend::to_floatexp(&zy) };
            if pr.step(xv, yv, exact) {
                return Some(max_iter);
            }
        }
    }
    Some(n)
}

// ---- The nucleus finder's passes (Mandelbrot), in MPFR --------------------------------------
//
// The finder walks the critical orbit in full precision several times per solve: the period
// detection, every Newton step (Z and dZ/dc), the period reduction and the atom size. At a
// 1e30000 view that is ~50,000-100,000 bits over ~820,000 steps per pass, and in astro-float one
// solve took 9+ hours; the render's reference orbit at the same width runs 4.6x faster here.
// Each pass mirrors its astro-float twin in `reference.rs` operation for operation, every op
// rounded toward zero (astro-float's `RoundingMode::None` truncates), so the two backends agree.

/// `Z ← Z² + c` in place, in `csqr`'s exact order (see `try_run_orbit_inplace`).
#[inline]
fn sq_add_c(zx: &mut Float, zy: &mut Float, x2: &mut Float, y2: &mut Float, t: &mut Float, rcx: &Float, rcy: &Float, ctx: u32) {
    use rug::ops::{AddAssignRound, AssignRound, SubAssignRound};
    x2.assign_round(&*zx * &*zx, RZ);
    y2.assign_round(&*zy * &*zy, RZ);
    t.assign_round(&*zx * &*zy, RZ);
    *t <<= 1; // exact
    x2.sub_assign_round(&*y2, RZ);
    x2.add_assign_round(rcx, RZ);
    t.add_assign_round(rcy, RZ);
    core::mem::swap(zx, x2);
    core::mem::swap(zy, t);
    if x2.prec() != ctx {
        x2.set_prec(ctx);
    }
    if t.prec() != ctx {
        t.set_prec(ctx);
    }
}

/// `D ← 2·Z·D + 1` in place — `cmul_bf(Z, D)` (4 rounded muls, a rounded sub, a rounded add),
/// then `×2` (exact) and `+1` rounded, as the astro Newton loop does. Uses `Z` BEFORE its step.
#[inline]
#[allow(clippy::too_many_arguments)]
fn d_update(dx: &mut Float, dy: &mut Float, zx: &Float, zy: &Float, a: &mut Float, b: &mut Float, c: &mut Float, e: &mut Float, one: &Float) {
    use rug::ops::{AddAssignRound, AssignRound, SubAssignRound};
    a.assign_round(zx * &*dx, RZ);
    b.assign_round(zy * &*dy, RZ);
    a.sub_assign_round(&*b, RZ); // Re(Z·D)
    c.assign_round(zx * &*dy, RZ);
    e.assign_round(zy * &*dx, RZ);
    c.add_assign_round(&*e, RZ); // Im(Z·D)
    *a <<= 1;
    a.add_assign_round(one, RZ);
    *c <<= 1;
    core::mem::swap(dx, a);
    core::mem::swap(dy, c);
}

/// The Newton pass: `Z_n` and `D_n = dZ_n/dc` after `n` steps from `Z_0 = D_0 = 0`, as carriers.
/// `at(step, zx, zy, dx, dy)` sees every step and may stop the walk (the period reduction).
fn zd_walk(
    cx: &BigFloat,
    cy: &BigFloat,
    n: u32,
    p: usize,
    mut at: impl FnMut(u32, &Float, &Float, &Float, &Float) -> bool,
) -> [BigFloat; 4] {
    let ctx = <Float as RefBackend>::ctx_for(p);
    let rcx = <Float as RefBackend>::from_carrier(cx, ctx);
    let rcy = <Float as RefBackend>::from_carrier(cy, ctx);
    let mut zx = Float::with_val(ctx, 0);
    let mut zy = Float::with_val(ctx, 0);
    let mut dx = Float::with_val(ctx, 0);
    let mut dy = Float::with_val(ctx, 0);
    let one = Float::with_val(ctx, 1);
    if par_enabled(ctx) && n > 0 {
        return zd_walk_par(zx, zy, dx, dy, &rcx, &rcy, &one, n, ctx, at);
    }
    let (mut x2, mut y2, mut t) = (Float::with_val(ctx, 0), Float::with_val(ctx, 0), Float::with_val(ctx, 0));
    let (mut a, mut b, mut c, mut e) =
        (Float::with_val(ctx, 0), Float::with_val(ctx, 0), Float::with_val(ctx, 0), Float::with_val(ctx, 0));
    for step in 1..=n {
        d_update(&mut dx, &mut dy, &zx, &zy, &mut a, &mut b, &mut c, &mut e, &one);
        sq_add_c(&mut zx, &mut zy, &mut x2, &mut y2, &mut t, &rcx, &rcy, ctx);
        crate::reference::count_reference_step(step);
        if at(step, &zx, &zy, &dx, &dy) {
            break;
        }
    }
    [zx.to_carrier(ctx), zy.to_carrier(ctx), dx.to_carrier(ctx), dy.to_carrier(ctx)]
}

/// [`zd_walk`] with its seven products on seven cores: `z² + c` needs x², y² and x·y, and
/// `D ← 2·Z·D + 1` needs Zx·Dx, Zy·Dy, Zx·Dy and Zy·Dx — all of the OLD Z and D, so all
/// independent. Combined exactly as `d_update` then `sq_add_c` do, op for op.
#[allow(clippy::too_many_arguments)]
fn zd_walk_par(
    zx: Float,
    zy: Float,
    dx: Float,
    dy: Float,
    rcx: &Float,
    rcy: &Float,
    one: &Float,
    n: u32,
    ctx: u32,
    mut at: impl FnMut(u32, &Float, &Float, &Float, &Float) -> bool,
) -> [BigFloat; 4] {
    use rug::ops::{AddAssignRound, SubAssignRound};
    // inputs: 0 Zx, 1 Zy, 2 Dx, 3 Dy
    let jobs = vec![vec![(0, 0)], vec![(1, 1)], vec![(0, 1)], vec![(0, 2)], vec![(1, 3)], vec![(0, 3)], vec![(1, 2)]];
    let par = ParProducts::new(vec![zx, zy, dx, dy], jobs, ctx);
    par.with_workers(|par| {
        for step in 1..=n {
            par.step();
            par.combine(|inp, outs| {
                let mut it = outs.iter_mut();
                let x2 = &mut it.next().unwrap()[0];
                let y2 = &it.next().unwrap()[0];
                let t = &mut it.next().unwrap()[0];
                let a = &mut it.next().unwrap()[0]; // Zx·Dx
                let b = &it.next().unwrap()[0]; // Zy·Dy
                let c = &mut it.next().unwrap()[0]; // Zx·Dy
                let e = &it.next().unwrap()[0]; // Zy·Dx
                // d_update: a − b, c + e, ×2, +1 on the real part.
                a.sub_assign_round(b, RZ);
                c.add_assign_round(e, RZ);
                *a <<= 1;
                a.add_assign_round(one, RZ);
                *c <<= 1;
                // sq_add_c
                *t <<= 1;
                x2.sub_assign_round(y2, RZ);
                x2.add_assign_round(rcx, RZ);
                t.add_assign_round(rcy, RZ);
                swap_in(&mut inp[0], x2, ctx);
                swap_in(&mut inp[1], t, ctx);
                swap_in(&mut inp[2], a, ctx);
                swap_in(&mut inp[3], c, ctx);
            });
            crate::reference::count_reference_step(step);
            if par.read(|v| at(step, &v[0], &v[1], &v[2], &v[3])) {
                break;
            }
        }
    });
    let v = par.inputs.into_inner().expect("inputs");
    [v[0].to_carrier(ctx), v[1].to_carrier(ctx), v[2].to_carrier(ctx), v[3].to_carrier(ctx)]
}

/// [`zd_walk`] for exactly `n` steps — one Newton step's orbit.
pub(crate) fn zd_pass(cx: &BigFloat, cy: &BigFloat, n: u32, p: usize) -> [BigFloat; 4] {
    zd_walk(cx, cy, n, p, |_, _, _, _, _| false)
}

/// `log2|x|` to the octave, read off the exponent — astro-float's `log2_abs_bf` (same
/// normalization: `0.m × 2^e`, `m ∈ [0.5, 1)`); `-∞` for zero.
fn exp_l2(x: &Float) -> f64 {
    if x.is_zero() {
        return f64::NEG_INFINITY;
    }
    match x.get_exp() {
        Some(e) => e as f64,
        None => f64::INFINITY,
    }
}

/// The period reduction (`reduce_period`'s twin): the first DIVISOR `n` of `p_est` whose
/// `|Z_n| / |D_n|` is below `2^tol_log2`.
pub(crate) fn period_scan(cx: &BigFloat, cy: &BigFloat, p_est: u32, tol_log2: f64, p: usize) -> Option<u32> {
    let mut found = None;
    zd_walk(cx, cy, p_est, p, |n, zx, zy, dx, dy| {
        if p_est % n != 0 {
            return false;
        }
        let d_l2 = exp_l2(dx).max(exp_l2(dy));
        if !d_l2.is_finite() {
            return false;
        }
        if exp_l2(zx).max(exp_l2(zy)) - d_l2 < tol_log2 {
            found = Some(n);
            return true;
        }
        false
    });
    found
}

/// The ball period detection (`detect_period_ball`'s twin): the first `n ≤ max` at which a disc of
/// radius `2^log2_radius` about `c`, carried by ball arithmetic, can contain 0. `log2|Z|` through
/// [`RefBackend::to_floatexp`], which reads exactly what astro-float's `bignum::log2_abs` reads.
pub(crate) fn ball_period(cx: &BigFloat, cy: &BigFloat, max: u32, log2_radius: f64, p: usize) -> Option<u32> {
    fn log2_add(a: f64, b: f64) -> f64 {
        let (hi, lo) = if a >= b { (a, b) } else { (b, a) };
        if lo == f64::NEG_INFINITY {
            return hi;
        }
        hi + (1.0 + (lo - hi).exp2()).log2()
    }
    let ctx = <Float as RefBackend>::ctx_for(p);
    let rcx = <Float as RefBackend>::from_carrier(cx, ctx);
    let rcy = <Float as RefBackend>::from_carrier(cy, ctx);
    let mut zx = Float::with_val(ctx, 0);
    let mut zy = Float::with_val(ctx, 0);
    let (mut x2, mut y2, mut t) = (Float::with_val(ctx, 0), Float::with_val(ctx, 0), Float::with_val(ctx, 0));
    let mut lz = f64::NEG_INFINITY;
    let mut lr = f64::NEG_INFINITY;
    if par_enabled(ctx) {
        let mut found = None;
        let mut n = 0u32;
        par_mandel_walk_fixed(
            zx,
            zy,
            &rcx,
            &rcy,
            ctx,
            |zx, zy| {
                n += 1;
                lr = log2_add(log2_add(1.0 + lz, lr) + lr, log2_radius);
                crate::reference::count_reference_step(n);
                let (lx, ly) = (RefBackend::to_floatexp(zx).log2(), RefBackend::to_floatexp(zy).log2());
                lz = 0.5 * log2_add(2.0 * lx, 2.0 * ly);
                if lz <= lr {
                    found = Some(n);
                    return true;
                }
                lz > 1.0 && lz.exp2() - lr.exp2() > 2.0
            },
            max.max(1),
        );
        return found;
    }
    for n in 1..=max.max(1) {
        lr = log2_add(log2_add(1.0 + lz, lr) + lr, log2_radius);
        sq_add_c(&mut zx, &mut zy, &mut x2, &mut y2, &mut t, &rcx, &rcy, ctx);
        crate::reference::count_reference_step(n);
        let (lx, ly) = (RefBackend::to_floatexp(&zx).log2(), RefBackend::to_floatexp(&zy).log2());
        lz = 0.5 * log2_add(2.0 * lx, 2.0 * ly);
        if lz <= lr {
            return Some(n);
        }
        if lz > 1.0 && lz.exp2() - lr.exp2() > 2.0 {
            return None;
        }
    }
    None
}

/// Munafo's atom-size pass (`nucleus_size`'s twin): `Λ = ∏ 2·z_i` and `B = 1 + Σ 1/Λ_i` over
/// `period − 1` steps, returned as carriers `[Λx, Λy, Bx, By]` for the caller to finish.
pub(crate) fn atom_size_pass(cx: &BigFloat, cy: &BigFloat, period: u32, p: usize) -> Option<[BigFloat; 4]> {
    use rug::ops::{AddAssignRound, AssignRound, DivAssignRound, SubAssignRound};
    let ctx = <Float as RefBackend>::ctx_for(p);
    let rcx = <Float as RefBackend>::from_carrier(cx, ctx);
    let rcy = <Float as RefBackend>::from_carrier(cy, ctx);
    let mut zx = Float::with_val(ctx, 0);
    let mut zy = Float::with_val(ctx, 0);
    let (mut lx, mut ly) = (Float::with_val(ctx, 1), Float::with_val(ctx, 0));
    let (mut bx, mut by) = (Float::with_val(ctx, 1), Float::with_val(ctx, 0));
    let (mut x2, mut y2, mut t) = (Float::with_val(ctx, 0), Float::with_val(ctx, 0), Float::with_val(ctx, 0));
    let (mut a, mut b, mut c, mut e) =
        (Float::with_val(ctx, 0), Float::with_val(ctx, 0), Float::with_val(ctx, 0), Float::with_val(ctx, 0));
    for n in 1..period {
        sq_add_c(&mut zx, &mut zy, &mut x2, &mut y2, &mut t, &rcx, &rcy, ctx);
        crate::reference::count_reference_step(n);
        // Λ ← 2·z·Λ: cmul_bf(z, Λ), then `m + m` (exact doubling).
        a.assign_round(&zx * &lx, RZ);
        b.assign_round(&zy * &ly, RZ);
        a.sub_assign_round(&b, RZ);
        c.assign_round(&zx * &ly, RZ);
        e.assign_round(&zy * &lx, RZ);
        c.add_assign_round(&e, RZ);
        a <<= 1;
        c <<= 1;
        core::mem::swap(&mut lx, &mut a);
        core::mem::swap(&mut ly, &mut c);
        // B ← B + 1/Λ: cinv_bf — d = x² + y² (rounded), then (x/d, (0 − y)/d).
        a.assign_round(&lx * &lx, RZ);
        b.assign_round(&ly * &ly, RZ);
        a.add_assign_round(&b, RZ);
        if a.is_zero() || !a.is_finite() {
            return None;
        }
        c.assign_round(&lx / &a, RZ);
        e.assign_round(0, RZ);
        e.sub_assign_round(&ly, RZ);
        e.div_assign_round(&a, RZ);
        bx.add_assign_round(&c, RZ);
        by.add_assign_round(&e, RZ);
    }
    Some([lx.to_carrier(ctx), ly.to_carrier(ctx), bx.to_carrier(ctx), by.to_carrier(ctx)])
}




/// The series-approximation coefficient walk in MPFR — the twin of `series_skip_astro`
/// (fractadyne-core `reference.rs`), Mandelbrot (`d = 2`) only; `None` = not handled here,
/// and the caller falls through to the astro walk (correct arithmetic for every family —
/// the fallback costs speed, never bits).
///
/// Mirrored LITERALLY, operation for operation, at the same word-rounded precisions with the
/// same truncate-toward-zero rounding:
///   * the reference `Z` runs at `p`'s width (`ctx`), the coefficients at
///     `SA_COEFF_BITS`'s (`ctx_c`) — astro's `pc`. A `Z·coefficient` product takes `Z` at full
///     width in both libraries (exact product, truncated result);
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
    cancel: Option<&std::sync::atomic::AtomicBool>,
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
    let ctx_c = <Float as RefBackend>::ctx_for(p.min(crate::reference::SA_COEFF_BITS));
    let one = Float::with_val(ctx_c, 1u32);
    let zero = |w: u32| Float::with_val(w, 0u32);
    let rcx = <Float as RefBackend>::from_carrier(cx, ctx);
    let rcy = <Float as RefBackend>::from_carrier(cy, ctx);
    // Astro's per-step copy of Z truncated to the coefficient width (its `set_precision` with
    // `RoundingMode::None` keeps the top `pc` bits of a normalised mantissa = MPFR's RZ).
    let narrow = ctx_c < ctx;
    // The two chains of `series_skip_astro_piped`, mirrored. One Z-chain step yields the
    // coefficient-width copy of Z_{n-1}, advances the reference through the shared generic step
    // (byte-identical by the orbit matrix) and says whether Z_n escaped (truncating f64 view,
    // like `to_f64`).
    // And whether Z_n is within the seed bound (`sa_seed_max2`, degree 2 here).
    let seed_max2 = crate::reference::sa_seed_max2(2);
    let z_step = |zx: &mut Float, zy: &mut Float| -> (Float, Float, bool, bool) {
        let (zcx, zcy) = if narrow {
            (Float::with_val_round(ctx_c, &*zx, RZ).0, Float::with_val_round(ctx_c, &*zy, RZ).0)
        } else {
            (zx.clone(), zy.clone())
        };
        let (nzx, nzy) = crate::reference::step_gen::<Float>(zx, zy, &rcx, &rcy, formula, ctx);
        *zx = nzx;
        *zy = nzy;
        let (fx, fy) = (zx.to_f64_trunc(), zy.to_f64_trunc());
        let m2 = fx * fx + fy * fy;
        (zcx, zcy, m2 > 1.0e12, m2 <= seed_max2)
    };
    // The coefficient chain. `Err` = cancelled.
    type Walked = Result<Option<(u32, [Float; 6])>, ()>;
    let walk = |next: &mut dyn FnMut() -> Option<(Float, Float, bool, bool)>| -> Walked {
        let (mut ax, mut ay) = (zero(ctx_c), zero(ctx_c));
        let (mut bx, mut by) = (zero(ctx_c), zero(ctx_c));
        let (mut cxx, mut cyy) = (zero(ctx_c), zero(ctx_c));
        let mut best: Option<(u32, [Float; 6])> = None;
        for n in 1..=limit {
            if cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed)) {
                return Err(());
            }
            let Some((zcx, zcy, z_escaped, z_seedable)) = next() else { break };
            let (zcx, zcy) = (&zcx, &zcy);
            // For d = 2: Z^{d-1} = Z itself (astro's `cpow_bf(z, 1)` is an exact clone) and the
            // Z^{d-2} factor is the identity, so the recurrence collapses to the lines below.
            let (a2x, a2y) = cmul(&ax, &ay, &ax, &ay, ctx_c); // A²
            let (abx, aby) = cmul(&ax, &ay, &bx, &by, ctx_c); // A·B
            // A' = 2·(Z·A) + 1
            let (t, u) = cmul(zcx, zcy, &ax, &ay, ctx_c);
            let na_x = t.fdouble().fadd(&one, ctx_c);
            let na_y = u.fdouble();
            // B' = 2·(Z·B) + A²    (C(2,2)·… — the ×1 is the identity in `mul_u32_bf` too)
            let (t, u) = cmul(zcx, zcy, &bx, &by, ctx_c);
            let nb_x = t.fdouble().fadd(&a2x, ctx_c);
            let nb_y = u.fdouble().fadd(&a2y, ctx_c);
            // C' = 2·(Z·C) + 2·(A·B)    (C(2,3) = 0 — no third term)
            let (t, u) = cmul(zcx, zcy, &cxx, &cyy, ctx_c);
            let nc_x = t.fdouble().fadd(&abx.fdouble(), ctx_c);
            let nc_y = u.fdouble().fadd(&aby.fdouble(), ctx_c);
            // (The reference advanced to Z_n in the Z chain.)
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
                if valid && z_seedable {
                    best = Some((
                        n,
                        [ax.clone(), ay.clone(), bx.clone(), by.clone(), cxx.clone(), cyy.clone()],
                    ));
                } else {
                    break; // once invalid, stays invalid; past the seed bound, escaping
                }
            }
            // Stop if the reference itself escaped.
            if z_escaped {
                break;
            }
        }
        Ok(best)
    };
    let (mut zx, mut zy) = (zero(ctx), zero(ctx));
    // Pipelined by the astro walk's rule, with its constants (see `series_skip_astro_piped`).
    let best = if narrow && limit >= crate::reference::SA_PIPELINE_MIN_STEPS {
        std::thread::scope(|s| {
            let (tx, rx) = std::sync::mpsc::sync_channel(crate::reference::SA_PIPELINE_DEPTH);
            s.spawn(move || {
                for _ in 0..limit {
                    let step = z_step(&mut zx, &mut zy);
                    let escaped = step.2;
                    if tx.send(step).is_err() || escaped {
                        break;
                    }
                }
            });
            let best = walk(&mut || rx.recv().ok());
            drop(rx);
            best
        })
    } else {
        walk(&mut || Some(z_step(&mut zx, &mut zy)))
    };
    let Ok(best) = best else {
        return Some(None); // = SeriesSkip::NONE, as the astro walk returns when cancelled
    };
    Some(best.map(|(n, k)| {
        (
            n,
            [
                k[0].to_carrier(ctx_c),
                k[1].to_carrier(ctx_c),
                k[2].to_carrier(ctx_c),
                k[3].to_carrier(ctx_c),
                k[4].to_carrier(ctx_c),
                k[5].to_carrier(ctx_c),
            ],
        )
    }))
}
