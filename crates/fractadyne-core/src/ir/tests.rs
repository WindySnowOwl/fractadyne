//! Phase 1 gates (design/custom-formulas.md §5): the built-in steps reproduced through the IR bit
//! for bit — `f64` orbits against `crate::orbit_points`, bignum orbits against
//! `crate::reference_orbit_t_in` (samples, length and the full-precision tail). Every gate counts
//! what it compared and asserts a floor, so a gate that silently compares nothing goes red.

use super::*;
use crate::formula as f;
use crate::BackendChoice;

/// A small deterministic generator (no dev-dependency needed).
fn lcg(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((*state >> 11) as f64) / ((1u64 << 53) as f64)
}

fn bits(p: (f64, f64)) -> (u64, u64) {
    (p.0.to_bits(), p.1.to_bits())
}

fn bytes(v: &BigFloat) -> Vec<u8> {
    let mut out = Vec::new();
    crate::bfbytes::write_bf(v, &mut out);
    out
}

fn bf(s: &str, p: usize) -> BigFloat {
    crate::parse_bf_prec(s, p).expect("test literal parses")
}

/// The built-ins that iterate with an escape test: the eight opcode families and Phoenix.
const ESCAPE_FAMILIES: [(u32, &str); 9] = [
    (f::MANDELBROT, "Mandelbrot"),
    (f::MULTIBROT3, "Multibrot 3"),
    (f::MULTIBROT4, "Multibrot 4"),
    (f::MULTIBROT5, "Multibrot 5"),
    (f::TRICORN, "Tricorn"),
    (f::BURNING_SHIP, "Burning Ship"),
    (f::CELTIC, "Celtic"),
    (f::BUFFALO, "Buffalo"),
    (f::PHOENIX, "Phoenix"),
];

fn builtin(id: u32) -> Formula {
    Formula::single(builtin_step(id).expect("every built-in has a step program"))
}

/// The power families (design/power-families.md), every id from `FAMILY_FIRST`.
fn power_families() -> Vec<(u32, String)> {
    (f::FAMILY_FIRST..f::COUNT)
        .map(|id| {
            let (shape, d) = f::family(id).expect("an id in the family range");
            (id, format!("{shape:?} {d}"))
        })
        .collect()
}

/// The hand-written power-family step (`fractal.rs`, the f64 overlay and the bignum reference)
/// is its IR program bit for bit — the program the generated shader module is built from, and
/// what the parser reads `abs(z)^3 + c` and the like as — with the program's escape degree `d`.
#[test]
fn power_families_are_their_programs_bit_for_bit() {
    let mut seed = 0x6b_u64;
    let mut bseed = 0x5eed_u64;
    assert_eq!(power_families().len(), 15);
    for (id, name) in power_families() {
        let formula = builtin(id);
        let (_, d) = f::family(id).unwrap();
        assert_eq!(formula.escape_degree(), Some(f64::from(d)), "{name}");
        assert_eq!(f::power(id), d, "{name}");
        let mut points = 0usize;
        for trial in 0..1_000 {
            let c = (lcg(&mut seed) * 2.4 - 1.2, lcg(&mut seed) * 2.4 - 1.2);
            let z0 = if trial % 2 == 0 { (0.0, 0.0) } else { (lcg(&mut seed) * 2.4 - 1.2, lcg(&mut seed) * 2.4 - 1.2) };
            let want = crate::orbit_points(z0, c, id, 300, 1.0e8);
            let got = orbit_points(&formula, z0, c, &[], 300, 1.0e8).unwrap();
            assert_eq!(got.len(), want.len(), "{name}: orbit length, trial {trial}");
            for (k, (g, w)) in got.iter().zip(&want).enumerate() {
                assert_eq!(bits(*g), bits(*w), "{name}: trial {trial}, point {k}");
            }
            points += want.len();
        }
        assert!(points > 5_000, "{name}: too few points compared ({points})");
        let mut samples = 0;
        for p in [128usize, 320] {
            for (z0, c) in bignum_cases(p, &mut bseed) {
                samples += compare_reference(BackendChoice::Astro, &name, id, &formula, (&z0.0, &z0.1), (&c.0, &c.1), 1_000, p);
            }
        }
        eprintln!("{name}: {points} f64 points and {samples} bignum samples identical");
    }
}

#[test]
fn f64_orbits_match_the_hand_written_ones_bit_for_bit() {
    let mut seed = 0x1a_2b_u64;
    for (id, name) in ESCAPE_FAMILIES {
        let formula = builtin(id);
        let mut points = 0usize;
        for trial in 0..2_000 {
            let c = (lcg(&mut seed) * 4.0 - 2.0, lcg(&mut seed) * 4.0 - 2.0);
            // Half parameter-plane (z0 = 0), half Julia-style (z0 anywhere).
            let z0 = if trial % 2 == 0 {
                (0.0, 0.0)
            } else {
                (lcg(&mut seed) * 4.0 - 2.0, lcg(&mut seed) * 4.0 - 2.0)
            };
            let want = crate::orbit_points(z0, c, id, 300, 1.0e8);
            let got = orbit_points(&formula, z0, c, &[], 300, 1.0e8).unwrap();
            assert_eq!(got.len(), want.len(), "{name}: orbit length, trial {trial}");
            for (k, (g, w)) in got.iter().zip(&want).enumerate() {
                assert_eq!(bits(*g), bits(*w), "{name}: trial {trial}, point {k}");
            }
            points += want.len();
        }
        eprintln!("{name}: {points} f64 points identical");
        assert!(points > 20_000, "{name}: too few points compared ({points})");
    }
}

#[test]
fn newton_step_matches_bit_for_bit() {
    let prog = builtin_step(f::NEWTON).unwrap();
    assert!(prog.bignum_evaluable(), "Newton divides, and division has a bignum form");
    let mut seed = 0x9e37_u64;
    let mut steps = 0usize;
    for _ in 0..2_000 {
        let z0 = (lcg(&mut seed) * 4.0 - 2.0, lcg(&mut seed) * 4.0 - 2.0);
        // Newton ignores c; its hand-written loop stops on convergence, which is not the step's
        // business — compare consecutive points.
        let orbit = crate::orbit_points(z0, (0.0, 0.0), f::NEWTON, 60, 1.0e8);
        for w in orbit.windows(2) {
            let got = step_f64(&prog, w[0], (0.0, 0.0), (0.0, 0.0), &[]).unwrap();
            assert_eq!(bits(got), bits(w[1]), "Newton step from {:?}", w[0]);
            steps += 1;
        }
    }
    eprintln!("Newton: {steps} steps identical");
    assert!(steps > 10_000, "too few Newton steps compared ({steps})");
}

/// One bignum comparison: the IR orbit against the hand-written reference orbit, in `backend`.
/// Returns the number of samples compared.
#[allow(clippy::too_many_arguments)]
fn compare_reference(
    backend: BackendChoice,
    name: &str,
    id: u32,
    formula: &Formula,
    z0: (&BigFloat, &BigFloat),
    c: (&BigFloat, &BigFloat),
    max_iter: u32,
    p: usize,
) -> usize {
    let (want, wlen, wtail) = crate::reference_orbit_t_in(backend, z0.0, z0.1, c.0, c.1, id, max_iter, p);
    let (got, glen, gtail) =
        reference_orbit_in(backend, formula, z0.0, z0.1, c.0, c.1, &[], max_iter, p).unwrap();
    let at = format!("{name} p={p} c=({}, {})", crate::to_f64(c.0), crate::to_f64(c.1));
    assert_eq!(glen, wlen, "{at}: orbit length");
    for (k, (g, w)) in got.iter().zip(&want).enumerate() {
        assert_eq!(g.map(f32::to_bits), w.map(f32::to_bits), "{at}: sample {k}");
    }
    assert_eq!(bytes(&gtail.zx), bytes(&wtail.zx), "{at}: tail zx");
    assert_eq!(bytes(&gtail.zy), bytes(&wtail.zy), "{at}: tail zy");
    assert_eq!(bytes(&gtail.zpx), bytes(&wtail.zpx), "{at}: tail zpx");
    assert_eq!(bytes(&gtail.zpy), bytes(&wtail.zpy), "{at}: tail zpy");
    assert_eq!(gtail.escaped, wtail.escaped, "{at}: escaped");
    assert_eq!(gtail.backend, wtail.backend, "{at}: backend stamp");
    want.len()
}

/// Points for the bignum gate: fixed boundary / interior / Julia cases written as long decimals
/// (so the carrier values are wider than one word), plus random `f64` points.
fn bignum_cases(p: usize, seed: &mut u64) -> Vec<((BigFloat, BigFloat), (BigFloat, BigFloat))> {
    let zero = || BigFloat::from_f64(0.0, p);
    let mut cases = vec![
        // Seahorse-valley boundary point (chaotic: any one-bit deviation grows).
        (
            (zero(), zero()),
            (bf("-0.743643887037158704752191506114774", p), bf("0.131825904205311970493132056385139", p)),
        ),
        // Interior of the main cardioid: runs the whole budget.
        ((zero(), zero()), (bf("-0.1528", p), bf("1.0397", p))),
        ((zero(), zero()), (bf("-0.25", p), bf("0.0000000000000000000000001", p))),
        // Near 0: inside every family's set, so every family runs the whole budget here.
        ((zero(), zero()), (bf("0.0123456789012345678901234567", p), bf("-0.0098765432109876543210987", p))),
        // Julia-style start.
        (
            (bf("0.3141592653589793238462643383279502884", p), bf("-0.2718281828459045235360287471", p)),
            (bf("-0.8", p), bf("0.156", p)),
        ),
    ];
    for _ in 0..12 {
        let c = (lcg(seed) * 3.0 - 2.0, lcg(seed) * 3.0 - 1.5);
        cases.push(((zero(), zero()), (BigFloat::from_f64(c.0, p), BigFloat::from_f64(c.1, p))));
    }
    cases
}

fn bignum_gate(backend: BackendChoice) {
    let mut seed = 0x5eed_u64;
    for p in [64usize, 128, 320, 1088] {
        for (id, name) in ESCAPE_FAMILIES {
            let formula = builtin(id);
            let mut samples = 0usize;
            for (z0, c) in bignum_cases(p, &mut seed) {
                samples += compare_reference(backend, name, id, &formula, (&z0.0, &z0.1), (&c.0, &c.1), 1_000, p);
            }
            eprintln!("{} {name} p={p}: {samples} samples identical", backend.name());
            // Two cases are inside every family's set, so at least 2 × 1,001 samples.
            assert!(samples > 2_000, "{name} p={p}: too few samples compared ({samples})");
        }
    }
}

#[test]
fn bignum_orbits_match_the_reference_orbit_bit_for_bit() {
    bignum_gate(BackendChoice::Astro);
}

#[cfg(feature = "rug")]
#[test]
fn bignum_orbits_match_the_reference_orbit_bit_for_bit_in_mpfr() {
    bignum_gate(BackendChoice::Rug);
}

/// Long orbits: thousands of consecutive steps from one state. `c = −1.9` is on the real axis,
/// where the quadratic families (Mandelbrot, Tricorn, Burning Ship, Celtic, Buffalo) all reduce to
/// the bounded, chaotic real map `x² − 1.9` — a one-bit deviation anywhere would grow.
#[test]
fn long_bignum_orbits_stay_identical() {
    let p = 192;
    let zero = BigFloat::from_f64(0.0, p);
    let (cx, cy) = (bf("-1.9000000000000000000000000000000000000001", p), BigFloat::from_f64(0.0, p));
    let mut total = 0;
    for (id, name) in ESCAPE_FAMILIES {
        let n = compare_reference(BackendChoice::Astro, name, id, &builtin(id), (&zero, &zero), (&cx, &cy), 5_000, p);
        eprintln!("{name}: {n} samples of one orbit identical");
        total += n;
    }
    assert!(total > 5 * 5_000, "the five quadratic families should run the whole budget ({total})");
}

/// The real axis above cannot see complex rounding (with `y = 0`, `(x+y)(x−y)` and `x·x − y·y` have
/// the same bits — a mutation test showed it staying green). So also a long COMPLEX orbit: `c` is
/// placed so `z^d + c` has an attracting fixed point with multiplier `0.995·e^{0.6πi}`, and the orbit
/// spirals into it over tens of thousands of steps, every one a distinct non-real value.
#[test]
fn long_complex_bignum_orbits_stay_identical() {
    let p = 192;
    let zero = BigFloat::from_f64(0.0, p);
    for (d, id, name) in [
        (2, f::MANDELBROT, "Mandelbrot"),
        (3, f::MULTIBROT3, "Multibrot 3"),
        (4, f::MULTIBROT4, "Multibrot 4"),
        (5, f::MULTIBROT5, "Multibrot 5"),
        (6, f::MULTIBROT6, "Multibrot 6"),
        (7, f::MULTIBROT7, "Multibrot 7"),
        (8, f::MULTIBROT8, "Multibrot 8"),
    ] {
        // Fixed point z* with d·z*^(d−1) = μ, and c = z* − z*^d.
        let mu = (0.995 * (0.6 * std::f64::consts::PI).cos(), 0.995 * (0.6 * std::f64::consts::PI).sin());
        let (r, t) = ((mu.0.hypot(mu.1) / d as f64).powf(1.0 / (d - 1) as f64), mu.1.atan2(mu.0) / (d - 1) as f64);
        let zs = (r * t.cos(), r * t.sin());
        let zd = (0..d - 1).fold(zs, |a, _| cmul64(a, zs));
        let (cx, cy) = (BigFloat::from_f64(zs.0 - zd.0, p), BigFloat::from_f64(zs.1 - zd.1, p));
        let n = compare_reference(BackendChoice::Astro, name, id, &builtin(id), (&zero, &zero), (&cx, &cy), 5_000, p);
        eprintln!("{name}: {n} samples of one complex orbit identical");
        assert_eq!(n, 5_001, "{name}: the orbit should stay bounded for the whole budget");
    }
}

#[test]
fn integer_powers_are_the_multibrot_chains() {
    // `PowI` squares and multiplies from the top bit: z^3 = z²·z, z^4 = (z²)², z^5 = (z²)²·z.
    let mut seed = 0x77_u64;
    for (n, id) in [(3u32, f::MULTIBROT3), (4, f::MULTIBROT4), (5, f::MULTIBROT5)] {
        let mut b = Builder::new();
        let z = b.push(Op::Z);
        let zn = b.push(Op::PowI(z, n));
        let c = b.push(Op::C);
        let out = b.push(Op::Add(zn, c));
        let formula = Formula::single(b.finish(out).unwrap());
        let mut points = 0;
        for _ in 0..500 {
            let c = (lcg(&mut seed) * 3.0 - 1.5, lcg(&mut seed) * 3.0 - 1.5);
            let want = crate::orbit_points((0.0, 0.0), c, id, 200, 1.0e8);
            let got = orbit_points(&formula, (0.0, 0.0), c, &[], 200, 1.0e8).unwrap();
            assert_eq!(got.iter().map(|p| bits(*p)).collect::<Vec<_>>(), want.iter().map(|p| bits(*p)).collect::<Vec<_>>(), "z^{n} + c at {c:?}");
            points += want.len();
        }
        assert!(points > 5_000, "z^{n}: too few points ({points})");
        let p = 256;
        let zero = BigFloat::from_f64(0.0, p);
        let (cx, cy) = (bf("-0.1528", p), bf("1.0397", p));
        compare_reference(BackendChoice::Astro, "PowI", id, &formula, (&zero, &zero), (&cx, &cy), 1_000, p);
    }
}

#[test]
fn hybrid_phases_alternate_per_iteration() {
    // Mandelbrot, Burning Ship, Tricorn in turn — checked against the hand-written steps applied
    // in the same rotation, in f64 and in bignum.
    let ids = [f::MANDELBROT, f::BURNING_SHIP, f::TRICORN];
    let formula = Formula::new(ids.iter().map(|&id| builtin_step(id).unwrap()).collect()).unwrap();
    let mut seed = 0x4242_u64;
    let mut points = 0usize;
    for _ in 0..500 {
        let c = (lcg(&mut seed) * 3.0 - 2.0, lcg(&mut seed) * 3.0 - 1.5);
        let got = orbit_points(&formula, (0.0, 0.0), c, &[], 200, 1.0e8).unwrap();
        let mut z = (0.0, 0.0);
        for (n, g) in got.iter().enumerate().skip(1) {
            z = crate::orbit_points(z, c, ids[(n - 1) % ids.len()], 1, f64::INFINITY)[1];
            assert_eq!(bits(*g), bits(z), "hybrid f64 at {c:?}, iteration {n}");
            points += 1;
        }
    }
    assert!(points > 5_000, "too few hybrid points ({points})");

    // The real axis again (all three reduce to `x² + cx` there), so the orbit runs the budget.
    let p = 256;
    let (cx, cy) = (bf("-1.7548776662466927600495", p), BigFloat::from_f64(0.0, p));
    let zero = BigFloat::from_f64(0.0, p);
    let (got, len, tail) = reference_orbit_in(BackendChoice::Astro, &formula, &zero, &zero, &cx, &cy, &[], 2_000, p).unwrap();
    let (mut zx, mut zy) = (zero.clone(), zero.clone());
    for n in 1..len as usize {
        (zx, zy) = crate::reference::step_bf(&zx, &zy, &cx, &cy, ids[(n - 1) % ids.len()], p);
        let want = pack_sample(crate::to_f64(&zx), crate::to_f64(&zy));
        assert_eq!(got[n].map(f32::to_bits), want.map(f32::to_bits), "hybrid bignum, iteration {n}");
    }
    assert_eq!(bytes(&tail.zx), bytes(&zx));
    assert_eq!(bytes(&tail.zy), bytes(&zy));
    assert!(len > 1_000, "the bignum hybrid orbit escaped too early to test ({len})");
}

#[test]
fn signs_fold_into_the_next_operation_and_materialise_at_the_output() {
    // −(z²) + c must be c − z² (the sign folds into the add), and −(z² + c) must come out negated.
    let mut b = Builder::new();
    let z = b.push(Op::Z);
    let s = b.push(Op::Sqr(z));
    let ns = b.push(Op::Neg(s));
    let c = b.push(Op::C);
    let folded = b.push(Op::Add(ns, c));
    let sum = b.push(Op::Add(s, c));
    let negated = b.push(Op::Neg(sum));
    let insts = b.finish(negated).unwrap().insts().to_vec();
    let folded = Program::new(insts.clone(), folded).unwrap();
    let negated = Program::new(insts, negated).unwrap();
    let mut seed = 0x31_u64;
    for _ in 0..1_000 {
        let z = (lcg(&mut seed) * 4.0 - 2.0, lcg(&mut seed) * 4.0 - 2.0);
        let c = (lcg(&mut seed) * 4.0 - 2.0, lcg(&mut seed) * 4.0 - 2.0);
        let (sx, sy) = (z.0 * z.0 - z.1 * z.1, 2.0 * z.0 * z.1);
        assert_eq!(bits(step_f64(&folded, z, c, (0.0, 0.0), &[]).unwrap()), bits((c.0 - sx, c.1 - sy)));
        assert_eq!(bits(step_f64(&negated, z, c, (0.0, 0.0), &[]).unwrap()), bits((-(sx + c.0), -(sy + c.1))));
    }
    // The same two programs in bignum, against astro-float's own operations.
    let p = 192;
    let (zx, zy) = (bf("0.3141592653589793238462643383279502884", p), bf("-0.2718281828459045235360287471", p));
    let (cx, cy) = (bf("-0.8", p), bf("0.156", p));
    let rm = crate::RM;
    let sx = zx.mul(&zx, p, rm).sub(&zy.mul(&zy, p, rm), p, rm);
    let sy = crate::double_bf(&zx.mul(&zy, p, rm));
    for (prog, wx, wy) in [
        (&folded, cx.sub(&sx, p, rm), cy.sub(&sy, p, rm)),
        (&negated, sx.add(&cx, p, rm).neg(), sy.add(&cy, p, rm).neg()),
    ] {
        let (_, _, tail) =
            reference_orbit_in(BackendChoice::Astro, &Formula::single(prog.clone()), &zx, &zy, &cx, &cy, &[], 1, p)
                .unwrap();
        assert_eq!(bytes(&tail.zx), bytes(&wx));
        assert_eq!(bytes(&tail.zy), bytes(&wy));
    }
}

#[test]
fn parameters_are_read_from_the_supplied_values() {
    // z² + p0 with p0 = c is Mandelbrot at c.
    let mut b = Builder::new();
    let z = b.push(Op::Z);
    let s = b.push(Op::Sqr(z));
    let k = b.push(Op::Param(0));
    let out = b.push(Op::Add(s, k));
    let formula = Formula::single(b.finish(out).unwrap());
    assert_eq!(formula.param_count(), 1);
    let c = (-0.7454, 0.1130);
    let want = crate::orbit_points((0.0, 0.0), c, f::MANDELBROT, 500, 1.0e8);
    let got = orbit_points(&formula, (0.0, 0.0), (9.0, 9.0), &[c], 500, 1.0e8).unwrap();
    assert_eq!(got.iter().map(|p| bits(*p)).collect::<Vec<_>>(), want.iter().map(|p| bits(*p)).collect::<Vec<_>>());
    assert_eq!(
        orbit_points(&formula, (0.0, 0.0), c, &[], 10, 4.0),
        Err(IrError::MissingParam { index: 0, supplied: 0 })
    );
}

#[test]
fn programs_are_validated() {
    use Op::*;
    assert_eq!(Program::new(vec![], Val(0)), Err(IrError::Empty));
    assert_eq!(Program::new(vec![Z, Sqr(Val(1))], Val(1)), Err(IrError::BadOperand { inst: 1, operand: 1 }));
    assert_eq!(Program::new(vec![Sqr(Val(1)), Z], Val(1)), Err(IrError::BadOperand { inst: 0, operand: 1 }));
    assert_eq!(Program::new(vec![Z], Val(1)), Err(IrError::BadOutput(1)));
    assert_eq!(Program::new(vec![Z, PowI(Val(0), 0)], Val(1)), Err(IrError::ZeroPower { inst: 1 }));
    assert_eq!(Program::new(vec![Const(f64::NAN, 0.0)], Val(0)), Err(IrError::NonFinite { inst: 0 }));
    assert_eq!(Program::new(vec![Z, Scale(Val(0), f64::INFINITY)], Val(1)), Err(IrError::NonFinite { inst: 1 }));
    assert_eq!(Formula::new(vec![]), Err(IrError::Empty));
    assert_eq!(lower_opcodes(&[Opcode::Sqr, Opcode::Mul]), Err(IrError::MulBeforeStore { opcode: 1 }));
    // Programs with no bignum form (a perturbed step's own operations) are refused by the bignum
    // interpreter rather than half-run.
    let diff = Formula::single(Program::new(vec![Z, C, DiffTanh(Val(0), Val(1))], Val(2)).unwrap());
    let p = 128;
    let z = BigFloat::from_f64(0.5, p);
    assert_eq!(
        reference_orbit_in(BackendChoice::Astro, &diff, &z, &z, &z, &z, &[], 10, p).map(|r| r.1),
        Err(IrError::NotBignum)
    );
}

/// Division and the functions in bignum follow the `f64` orbit, as far as `f64` can follow it:
/// the first steps agree to near `f64` rounding, before chaos amplifies it.
#[test]
fn bignum_functions_follow_the_f64_orbit() {
    let p = 256;
    let mut compared = 0;
    for src in [
        "sin(z) + c",
        "cos(z)*cos(z) + c",
        "exp(z) + c",
        "sinh(z) - cosh(z)*c",
        "tan(z) + c",
        "tanh(z) + c",
        "z^2 + c/(z + 2)",
        "log(z*z + 1) + c",
        "sqrt(z) + c",
        "sqrt(z^4 + c)",
        "z^2.5 + c",
        "(z + 1)^(0.5, 0.25) + c",
    ] {
        let formula = crate::ir::parse::parse(src).unwrap();
        let (c, z0) = ((0.21, -0.37), (0.0, 0.0));
        let want = orbit_points(&formula, z0, c, &[], 8, 1.0e8).unwrap();
        let bf = |v: f64| BigFloat::from_f64(v, p);
        let (orbit, _, _) =
            reference_orbit_in(BackendChoice::Astro, &formula, &bf(z0.0), &bf(z0.1), &bf(c.0), &bf(c.1), &[], 8, p)
                .unwrap_or_else(|e| panic!("{src}: {e}"));
        for (k, (w, s)) in want.iter().zip(&orbit).enumerate() {
            let g = crate::sample_xy(s);
            let err = (g.0 - w.0).hypot(g.1 - w.1) / w.0.hypot(w.1).max(1.0);
            assert!(err < 1.0e-12, "{src}: step {k}: bignum {g:?} vs f64 {w:?} ({err:e})");
            compared += 1;
        }
    }
    assert!(compared >= 50, "too few points compared ({compared})");
}

#[test]
fn escape_degrees_follow_the_leading_term() {
    let degree = |id| builtin_step(id).unwrap().escape_degree();
    for (id, d) in [
        (f::MANDELBROT, 2.0),
        (f::MULTIBROT3, 3.0),
        (f::MULTIBROT4, 4.0),
        (f::MULTIBROT5, 5.0),
        (f::TRICORN, 2.0),
        (f::BURNING_SHIP, 2.0),
        (f::CELTIC, 2.0),
        (f::BUFFALO, 2.0),
        (f::PHOENIX, 2.0),
    ] {
        assert_eq!(degree(id), Some(d), "formula {id}");
    }
    // Newton: z − (z³ − 1)/(3z²) is degree 1 (it converges; nothing escapes).
    assert_eq!(degree(f::NEWTON), Some(1.0));
    let hybrid = Formula::new(vec![builtin_step(f::MANDELBROT).unwrap(), builtin_step(f::MULTIBROT3).unwrap()]).unwrap();
    assert!((hybrid.escape_degree().unwrap() - 6f64.sqrt()).abs() < 1e-12);
    // exp(z) + c has no power law; z^2.5 + c has degree 2.5.
    let mut b = Builder::new();
    let z = b.push(Op::Z);
    let e = b.push(Op::Func(Func::Exp, z));
    assert_eq!(b.finish(e).unwrap().escape_degree(), None);
    let mut b = Builder::new();
    let z = b.push(Op::Z);
    let k = b.push(Op::Const(2.5, 0.0));
    let p = b.push(Op::Pow(z, k));
    assert_eq!(b.finish(p).unwrap().escape_degree(), Some(2.5));
}

#[test]
fn opcodes_negate_and_rotate_exactly() {
    let one = |ops: &[Opcode], z: (f64, f64)| step_f64(&lower_opcodes(ops).unwrap(), z, (0.0, 0.0), (0.0, 0.0), &[]).unwrap();
    let z = (0.375, -1.25);
    assert_eq!(one(&[Opcode::NegX], z), (-0.375, -1.25));
    assert_eq!(one(&[Opcode::NegY], z), (0.375, 1.25));
    assert_eq!(one(&[Opcode::AbsX, Opcode::AbsY], (-0.375, -1.25)), (0.375, 1.25));
    // Quarter turns are exact (no 6e-17 residue from f64 trigonometry).
    assert_eq!(one(&[Opcode::Rot(90.0)], z), (1.25, 0.375));
    assert_eq!(one(&[Opcode::Rot(-90.0)], z), (-1.25, -0.375));
    assert_eq!(one(&[Opcode::Rot(180.0)], z), (-0.375, 1.25));
    let r = one(&[Opcode::Rot(45.0)], (1.0, 0.0));
    assert!((r.0 - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-15 && (r.1 - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-15);
}

#[test]
fn elementary_functions_satisfy_their_identities() {
    let unary = |op: fn(Val) -> Op, z: (f64, f64)| {
        let mut b = Builder::new();
        let v = b.push(Op::Z);
        let out = b.push(op(v));
        step_f64(&b.finish(out).unwrap(), z, (0.0, 0.0), (0.0, 0.0), &[]).unwrap()
    };
    let close = |a: (f64, f64), b: (f64, f64), what: &str| {
        let err = (a.0 - b.0).abs().max((a.1 - b.1).abs()) / b.0.abs().max(b.1.abs()).max(1.0);
        assert!(err < 1e-13, "{what}: {a:?} vs {b:?}");
    };
    let mut seed = 0xe1e_u64;
    for _ in 0..500 {
        let z = (lcg(&mut seed) * 4.0 - 2.0, lcg(&mut seed) * 4.0 - 2.0);
        let e = unary(|v| Op::Func(Func::Exp, v), z);
        let l = unary(|v| Op::Func(Func::Log, v), e);
        if z.1.abs() < std::f64::consts::PI {
            close(l, z, "log(exp z)");
        }
        let r = unary(|v| Op::Func(Func::Sqrt, v), z);
        close(cmul64(r, r), z, "sqrt(z)²");
        assert!(r.0 >= 0.0, "principal root has Re ≥ 0");
        let (s, c) = (unary(|v| Op::Func(Func::Sin, v), z), unary(|v| Op::Func(Func::Cos, v), z));
        close((s.0 * s.0 - s.1 * s.1 + c.0 * c.0 - c.1 * c.1, 2.0 * (s.0 * s.1 + c.0 * c.1)), (1.0, 0.0), "sin² + cos²");
        let (sh, ch) = (unary(|v| Op::Func(Func::Sinh, v), z), unary(|v| Op::Func(Func::Cosh, v), z));
        close((ch.0 * ch.0 - ch.1 * ch.1 - sh.0 * sh.0 + sh.1 * sh.1, 2.0 * (ch.0 * ch.1 - sh.0 * sh.1)), (1.0, 0.0), "cosh² − sinh²");
        close(cmul64(unary(|v| Op::Func(Func::Tan, v), z), c), s, "tan·cos");
        close(cmul64(unary(|v| Op::Func(Func::Tanh, v), z), ch), sh, "tanh·cosh");
        close(unary(Op::Norm, z), (z.0 * z.0 + z.1 * z.1, 0.0), "norm");
    }
    // Where cosh (cos) overflows, tanh (tan) is ±1 (±i) — not the NaN of inf/inf.
    for x in [400.0, -400.0, 750.0, -750.0] {
        let t = unary(|v| Op::Func(Func::Tanh, v), (x, 0.3));
        assert_eq!(t, (f64::signum(x), 0.0), "tanh({x} + 0.3i)");
        let t = unary(|v| Op::Func(Func::Tan, v), (0.3, x));
        assert_eq!(t, (0.0, f64::signum(x)), "tan(0.3 + {x}i)");
    }
    // The perturbed tanh and tan: the plain difference wherever it does not cancel (a small b),
    // on both sides of the split, and finite where a product of factors would be inf·0.
    let binary = |op: fn(Val, Val) -> Op, b: (f64, f64), p: (f64, f64)| {
        let mut k = Builder::new();
        let (vb, vp) = (k.push(Op::Z), k.push(Op::C));
        let out = k.push(op(vb, vp));
        step_f64(&k.finish(out).unwrap(), b, p, (0.0, 0.0), &[]).unwrap()
    };
    let b = (0.2, 0.1);
    for p in [(0.7, -0.4), (30.0, 1.0), (39.9, 0.5), (40.0, 0.5), (60.0, -1.0), (800.0, 0.2), (-800.0, 0.2)] {
        let naive = |f: fn(Val) -> Op, b: (f64, f64), p: (f64, f64)| {
            let (t, u) = (unary(f, (b.0 + p.0, b.1 + p.1)), unary(f, b));
            (t.0 - u.0, t.1 - u.1)
        };
        let naive_tanh = naive(|v| Op::Func(Func::Tanh, v), b, p);
        close(binary(Op::DiffTanh, b, p), naive_tanh, &format!("DiffTanh({b:?}, {p:?})"));
        let (bi, pi) = ((b.1, b.0), (p.1, p.0));
        let naive_tan = naive(|v| Op::Func(Func::Tan, v), bi, pi);
        close(binary(Op::DiffTan, bi, pi), naive_tan, &format!("DiffTan({bi:?}, {pi:?})"));
    }
    // Far out, the product form stays RELATIVELY accurate where the difference is exactly 0.
    let d = binary(Op::DiffTanh, (30.0, 0.0), (1e-9, 0.0));
    let want = 1e-9 * 4.0 * (-60.0f64).exp(); // sinh(p)·sech²(30)
    assert!(((d.0 - want) / want).abs() < 1e-6 && d.1 == 0.0, "DiffTanh far out: {d:?} vs {want:e}");
    // Division and complex powers.
    let mut b = Builder::new();
    let z = b.push(Op::Z);
    let c = b.push(Op::C);
    let q = b.push(Op::Div(z, c));
    let back = b.push(Op::Mul(q, c));
    let division = b.finish(back).unwrap();
    let mut b = Builder::new();
    let z = b.push(Op::Z);
    let two = b.push(Op::Const(2.0, 0.0));
    let out = b.push(Op::Pow(z, two));
    let power = b.finish(out).unwrap();
    for _ in 0..500 {
        let z = (lcg(&mut seed) * 4.0 - 2.0, lcg(&mut seed) * 4.0 - 2.0);
        let c = (lcg(&mut seed) * 4.0 - 2.0, lcg(&mut seed) * 4.0 - 2.0);
        close(step_f64(&division, z, c, (0.0, 0.0), &[]).unwrap(), z, "(z / c)·c");
        close(step_f64(&power, z, c, (0.0, 0.0), &[]).unwrap(), cmul64(z, z), "z^2.0");
    }
    assert_eq!(step_f64(&power, (0.0, 0.0), (0.0, 0.0), (0.0, 0.0), &[]).unwrap(), (0.0, 0.0), "0^w = 0");
}
