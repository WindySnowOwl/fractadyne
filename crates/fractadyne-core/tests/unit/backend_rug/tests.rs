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
