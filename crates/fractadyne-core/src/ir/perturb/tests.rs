use super::*;
use crate::ir::{builtin_step, parse::parse, reference_orbit_in, step_f64, step_perturbed_f64};
use crate::BackendChoice;
use astro_float::BigFloat;

fn lcg(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((*state >> 11) as f64) / ((1u64 << 53) as f64)
}

const P: usize = 256;

fn bf(v: f64) -> BigFloat {
    BigFloat::from_f64(v, P)
}

/// One exact step `f(z, c)` in 256-bit bignum, from bignum inputs.
fn exact_step(prog: &Program, z: (&BigFloat, &BigFloat), c: (&BigFloat, &BigFloat), params: &[(f64, f64)]) -> (BigFloat, BigFloat) {
    let f = Formula::single(prog.clone());
    let (_, _, tail) = reference_orbit_in(BackendChoice::Astro, &f, z.0, z.1, c.0, c.1, params, 1, P).unwrap();
    (tail.zx, tail.zy)
}

/// The formulas the rule table must handle: every built-in step it can express, and typed ones
/// covering the rest of the ring (parameters, products of two varying values, |z|², re/im, a
/// high power, conj, abs on a sum).
fn cases() -> Vec<(String, Program, Vec<(f64, f64)>)> {
    let mut v: Vec<(String, Program, Vec<(f64, f64)>)> = (0..8)
        .map(|id| (format!("built-in {id}"), builtin_step(id).unwrap(), vec![]))
        .collect();
    for (src, params) in [
        ("z^3 - p1*z + c", vec![(0.5, -0.25)]),
        ("z^2 + c*z + (0.25, 0.1)", vec![]),
        ("z*z + |z|*c*0.1 + c", vec![]),
        ("real(z)*z + imag(z)*c + c", vec![]),
        ("z^7 + c", vec![]),
        ("conj(z)^3 + p1*c", vec![(1.0, 0.5)]),
        ("abs(z + c)^2 - z + c", vec![]),
        ("t = sqr(z), z = t*t - t + c", vec![]),
        // Division and the functions with a perturbed form, alone and composed.
        ("z^2 + c/(z + 2)", vec![]),
        ("(z^2 + p1)/(z - p1) + c", vec![(0.75, 0.25)]),
        ("sin(z) + c", vec![]),
        ("sin(z) + cos(z)*cos(z + 3.14159) + c", vec![]),
        ("exp(z) + c", vec![]),
        ("sinh(z) + cosh(z)*c", vec![]),
        ("tan(z) + c", vec![]),
        ("tanh(z*z) + c", vec![]),
        // The branch-cut functions and fixed-exponent powers, alone and composed.
        ("log(z*z + 1) + c", vec![]),
        ("z^2 + 0.3*log(z + 2) + c", vec![]),
        ("sqrt(z*z*z + c) + c", vec![]),
        ("z^2 + c*sqrt(z + 1)", vec![]),
        ("z^2.5 + c", vec![]),
        ("z^p1 + c", vec![(2.2, 0.3)]),
        ("sqrt(z)^3 + c", vec![]),
    ] {
        v.push((src.to_string(), parse(src).unwrap().phases()[0].clone(), params));
    }
    v
}

#[test]
fn the_perturbed_step_is_the_exact_difference_without_its_cancellation() {
    let mut seed = 0x9e77_u64;
    for (name, prog, params) in cases() {
        let pert = perturbed(&prog).unwrap_or_else(|e| panic!("{name}: {e}"));
        let (mut n, mut worst, mut naive_worst) = (0usize, 0.0f64, 0.0f64);
        for trial in 0..400 {
            // A reference point ON an orbit (not a random one), so Z has realistic magnitudes.
            let c = (lcg(&mut seed) * 2.0 - 1.5, lcg(&mut seed) * 2.0 - 1.0);
            let formula = Formula::single(prog.clone());
            let orbit = crate::ir::orbit_points(&formula, (0.0, 0.0), c, &params, 12, 1.0e6).unwrap();
            let z = orbit[orbit.len() / 2];
            // δ at 1e-8 … 1e-13 of the values, Mandelbrot-style (δc too) and Julia-style (δc = 0).
            let scale = 10f64.powf(-8.0 - 5.0 * lcg(&mut seed));
            let dz = ((lcg(&mut seed) - 0.5) * scale, (lcg(&mut seed) - 0.5) * scale);
            let dc = if trial % 2 == 0 { ((lcg(&mut seed) - 0.5) * scale, (lcg(&mut seed) - 0.5) * scale) } else { (0.0, 0.0) };
            // Exact: f(Z+δz, C+δc) − f(Z, C) in 256 bits (f64 + f64 is exact at that width).
            let (zx, zy, cx, cy) = (bf(z.0), bf(z.1), bf(c.0), bf(c.1));
            let (pzx, pzy) = (zx.add(&bf(dz.0), P, crate::RM), zy.add(&bf(dz.1), P, crate::RM));
            let (pcx, pcy) = (cx.add(&bf(dc.0), P, crate::RM), cy.add(&bf(dc.1), P, crate::RM));
            let (fx, fy) = exact_step(&prog, (&zx, &zy), (&cx, &cy), &params);
            let (gx, gy) = exact_step(&prog, (&pzx, &pzy), (&pcx, &pcy), &params);
            let want = (crate::to_f64(&gx.sub(&fx, P, crate::RM)), crate::to_f64(&gy.sub(&fy, P, crate::RM)));
            let mag = want.0.hypot(want.1);
            if !(mag > 0.0) || !mag.is_finite() {
                continue;
            }
            let got = step_perturbed_f64(&pert, z, c, dz, dc, &params).unwrap();
            let err = (got.0 - want.0).hypot(got.1 - want.1) / mag;
            worst = worst.max(err);
            // The control: the naive f64 difference of two full steps.
            let a = step_f64(&prog, (z.0 + dz.0, z.1 + dz.1), (c.0 + dc.0, c.1 + dc.1), (0.0, 0.0), &params).unwrap();
            let b = step_f64(&prog, z, c, (0.0, 0.0), &params).unwrap();
            naive_worst = naive_worst.max(((a.0 - b.0) - want.0).hypot((a.1 - b.1) - want.1) / mag);
            n += 1;
        }
        eprintln!("{name}: {n} cases, perturbed worst {worst:.2e}, naive worst {naive_worst:.2e}");
        assert!(n >= 300, "{name}: too few cases ({n})");
        assert!(worst < 1e-9, "{name}: perturbed step off by {worst:e}");
        assert!(naive_worst > 1e-4, "{name}: the control should fail ({naive_worst:e}) — the test cannot discriminate");
    }
}

/// Iterated: the perturbed orbit `Z_n + δz_n` (f64 δ along a bignum reference) tracks the pixel's
/// own bignum orbit, for a single-phase and a two-phase (hybrid) formula, at a 1e-20 offset — far
/// below anything f64 could resolve directly.
#[test]
fn a_perturbed_orbit_tracks_the_pixels_exact_orbit() {
    // Each c inside its formula's set, so the orbit stays bounded for the whole comparison.
    for (name, formula, c) in [
        ("z^3 - 0.5z + c", parse("z^3 - 0.5*z + c").unwrap(), (-0.1528, 0.6397)),
        (
            "Mandelbrot/Burning Ship hybrid",
            Formula::new(vec![builtin_step(0).unwrap(), builtin_step(5).unwrap()]).unwrap(),
            (-0.2, 0.1),
        ),
    ] {
        let pert = perturbed_formula(&formula).unwrap();
        let k = formula.phases().len();
        let dc = (3.0e-20, -2.0e-20);
        let (cx, cy) = (bf(c.0), bf(c.1));
        let (pcx, pcy) = (cx.add(&bf(dc.0), P, crate::RM), cy.add(&bf(dc.1), P, crate::RM));
        let (mut zx, mut zy) = (bf(0.0), bf(0.0)); // reference
        let (mut px, mut py) = (bf(0.0), bf(0.0)); // pixel, exact
        let mut dz = (0.0f64, 0.0f64);
        let mut steps = 0;
        for n in 0..200 {
            let phase = &formula.phases()[n % k];
            let z = (crate::to_f64(&zx), crate::to_f64(&zy));
            dz = step_perturbed_f64(&pert.phases()[n % k], z, c, dz, dc, &[]).unwrap();
            (zx, zy) = exact_step(phase, (&zx, &zy), (&cx, &cy), &[]);
            (px, py) = exact_step(phase, (&px, &py), (&pcx, &pcy), &[]);
            // The pixel's true deviation from the reference, and ours.
            let want = (
                crate::to_f64(&px.sub(&zx, P, crate::RM)),
                crate::to_f64(&py.sub(&zy, P, crate::RM)),
            );
            let mag = want.0.hypot(want.1);
            if mag > 1e-3 || crate::to_f64(&zx).hypot(crate::to_f64(&zy)) > 1e3 {
                break; // the deviation left the small regime (or the orbit escaped)
            }
            let err = (dz.0 - want.0).hypot(dz.1 - want.1) / mag.max(1e-300);
            assert!(err < 1e-6, "{name}: step {n}: δz {dz:?} vs exact {want:?} (rel {err:e})");
            steps += 1;
        }
        eprintln!("{name}: {steps} perturbed steps track the exact pixel orbit");
        assert!(steps >= 50, "{name}: too few steps compared ({steps})");
    }
}

/// Across the branch cut (the negative real axis) the principal `log`, `sqrt` and powers JUMP, and
/// the perturbed step must jump with them: `Log W − Log B` is `log1p(P/B) ∓ 2πi` there, and
/// `√W − √B` is nearly `∓2i√|B|`, not the tiny `P/(√W + √B)` — a pixel whose orbit crosses where
/// the reference's does not. Against the exact difference in 256-bit bignum, for references above,
/// below and ON the cut (a zero imaginary part is its upper side), not crossing, crossing the
/// positive axis (no cut), and a power's reference at 0 (`0^k = 0`, as after every rebase).
#[test]
fn the_branch_cut_functions_jump_where_their_principal_values_do() {
    let one = |src: &str, params: &[(f64, f64)], z: (f64, f64), dz: (f64, f64)| {
        let prog = parse(src).unwrap().phases()[0].clone();
        let (zx, zy, zero) = (bf(z.0), bf(z.1), bf(0.0));
        let (wx, wy) = (zx.add(&bf(dz.0), P, crate::RM), zy.add(&bf(dz.1), P, crate::RM));
        let (fx, fy) = exact_step(&prog, (&zx, &zy), (&zero, &zero), params);
        let (gx, gy) = exact_step(&prog, (&wx, &wy), (&zero, &zero), params);
        let want = (crate::to_f64(&gx.sub(&fx, P, crate::RM)), crate::to_f64(&gy.sub(&fy, P, crate::RM)));
        let pert = perturbed(&prog).unwrap();
        let got = step_perturbed_f64(&pert, z, (0.0, 0.0), dz, (0.0, 0.0), params).unwrap();
        let err = (got.0 - want.0).hypot(got.1 - want.1) / want.0.hypot(want.1);
        assert!(err < 1.0e-9, "{src} at {z:?} + {dz:?}: {got:?} vs exact {want:?} ({err:e})");
        want
    };
    let mut jumps = 0;
    for (src, params) in [("log(z)", vec![]), ("sqrt(z)", vec![]), ("z^2.5", vec![]), ("z^p1", vec![(0.4, 0.3)])] {
        for (z, dz, crosses) in [
            ((-1.3, 2.0e-11), (1.0e-12, -5.0e-11), true),
            ((-1.3, -2.0e-11), (0.0, 5.0e-11), true),
            ((-1.3, 0.0), (3.0e-12, -1.0e-11), true),
            ((-1.3, 0.0), (3.0e-12, 1.0e-11), false),
            ((-1.3, 2.0e-11), (1.0e-12, 1.0e-11), false),
            ((0.8, 1.0e-3), (1.0e-9, -2.0e-3), false),
        ] {
            let want = one(src, &params, z, dz);
            // A crossing moves the value by O(1), anything else by O(|δz|) (≤ 3e-3 here).
            assert_eq!(want.0.hypot(want.1) > 0.1, crosses, "{src} at {z:?} + {dz:?}: {want:?}");
            jumps += crosses as usize;
        }
        if src != "log(z)" {
            one(src, &params, (0.0, 0.0), (1.0e-12, 2.0e-12));
        }
    }
    assert_eq!(jumps, 12);
}

#[test]
fn what_cannot_be_perturbed_yet_says_what() {
    let err = |src: &str| perturbed(&parse(src).unwrap().phases()[0]).unwrap_err().0;
    assert_eq!(err("z^c + c"), "a power whose exponent varies");
    assert_eq!(err("z^(z*0.5) + c"), "a power whose exponent varies");
    assert!(perturbable(&parse("z^2 + c").unwrap()));
    assert!(perturbable(&parse("z/c + c").unwrap()), "division has a perturbed form");
    assert!(perturbable(&parse("sin(z) + cos(z)*cos(z) + c").unwrap()), "so do sin and cos");
    for src in ["log(z) + c", "sqrt(z) + c", "z^2.5 + c", "z^p1 + c", "(z + 1)^(0.5, 0.25) + c"] {
        assert!(perturbable(&parse(src).unwrap()), "{src}: log, sqrt and fixed-exponent powers perturb");
    }
    assert!(!perturbable(&Formula::single(builtin_step(crate::formula::PHOENIX).unwrap())), "Phoenix reads z_prev");
    // A step that ignores z and c perturbs to zero.
    let flat = perturbed(&parse("(0.5, 0.5)").unwrap().phases()[0]).unwrap();
    assert_eq!(step_perturbed_f64(&flat, (1.0, 2.0), (3.0, 4.0), (1e-9, 0.0), (0.0, 1e-9), &[]).unwrap(), (0.0, 0.0));
    // Mandelbrot's perturbed step reads only Z, δz and δc — no reference c, no abs folds.
    let m = perturbed(&builtin_step(0).unwrap()).unwrap();
    assert!(!m.insts().iter().any(|op| matches!(op, Op::C | Op::DiffAbsRe(..) | Op::DiffAbsIm(..))), "{:?}", m.insts());
    let ship = perturbed(&builtin_step(crate::formula::BURNING_SHIP).unwrap()).unwrap();
    assert!(ship.insts().iter().any(|op| matches!(op, Op::DiffAbsRe(..))));
}
