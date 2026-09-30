use super::*;
use crate::formula as f;
use crate::ir::{orbit_points, reference_orbit_in, step_f64};
use crate::BackendChoice;
use astro_float::BigFloat;

fn lcg(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((*state >> 11) as f64) / ((1u64 << 53) as f64)
}

fn bits(p: (f64, f64)) -> (u64, u64) {
    (p.0.to_bits(), p.1.to_bits())
}

fn step(src: &str, z: (f64, f64), c: (f64, f64), params: &[(f64, f64)]) -> (f64, f64) {
    let formula = parse(src).unwrap_or_else(|e| panic!("{src:?}: {e}"));
    step_f64(&formula.phases()[0], z, c, (0.0, 0.0), params).unwrap()
}

/// A typed formula reproduces the built-in's orbits bit for bit, in f64 and in bignum.
fn same_as_builtin(src: &str, id: u32) {
    let typed = parse(src).unwrap_or_else(|e| panic!("{src:?}: {e}"));
    let mut seed = 0xfeed_u64 ^ id as u64;
    let mut points = 0;
    for _ in 0..500 {
        let c = (lcg(&mut seed) * 3.0 - 2.0, lcg(&mut seed) * 3.0 - 1.5);
        let want = crate::orbit_points((0.0, 0.0), c, id, 200, 1.0e8);
        let got = orbit_points(&typed, (0.0, 0.0), c, &[], 200, 1.0e8).unwrap();
        assert_eq!(
            got.iter().map(|p| bits(*p)).collect::<Vec<_>>(),
            want.iter().map(|p| bits(*p)).collect::<Vec<_>>(),
            "{src:?} vs formula {id} at {c:?}"
        );
        points += want.len();
    }
    assert!(points > 5_000, "{src:?}: too few points ({points})");
    let p = 192;
    let zero = BigFloat::from_f64(0.0, p);
    let (cx, cy) = (crate::parse_bf_prec("-0.1528", p).unwrap(), crate::parse_bf_prec("0.0397", p).unwrap());
    let (want, wl, _) = crate::reference_orbit_t_in(BackendChoice::Astro, &zero, &zero, &cx, &cy, id, 1_000, p);
    let (got, gl, _) = reference_orbit_in(BackendChoice::Astro, &typed, &zero, &zero, &cx, &cy, &[], 1_000, p).unwrap();
    assert_eq!(gl, wl, "{src:?}: bignum orbit length");
    assert!(got.iter().zip(&want).all(|(a, b)| a.map(f32::to_bits) == b.map(f32::to_bits)), "{src:?}: bignum samples");
}

#[test]
fn typed_built_ins_are_the_built_ins_bit_for_bit() {
    same_as_builtin("z^2 + c", f::MANDELBROT);
    same_as_builtin("z = z*z + c", f::MANDELBROT);
    same_as_builtin("sqr(z) + pixel", f::MANDELBROT);
    same_as_builtin("z^3 + c", f::MULTIBROT3);
    same_as_builtin("z^4 + c", f::MULTIBROT4);
    same_as_builtin("t = sqr(z), z = t*t + c", f::MULTIBROT4);
    same_as_builtin("z^5 + c", f::MULTIBROT5);
    same_as_builtin("conj(z)^2 + c", f::TRICORN);
    same_as_builtin("abs(z)^2 + c", f::BURNING_SHIP);
}

#[test]
fn statements_temporaries_and_comments() {
    let z = (0.3, -0.7);
    let c = (-0.5, 0.1);
    let want = step("z^2 + c", z, c, &[]);
    // Newline-separated, comments, a temporary, and `z` re-read after assignment.
    assert_eq!(bits(step("; Mandelbrot\nt = z*z ; square\nz = t\nz = z + c\n", z, c, &[])), bits(want));
    // Case-insensitive names.
    assert_eq!(bits(step("Z = Z*Z + C", z, c, &[])), bits(want));
    // A bare expression after an assignment is the new z.
    assert_eq!(bits(step("z = 5, z^2 + c", z, c, &[])), bits(step("25 + c", z, c, &[])));
}

#[test]
fn constants_parameters_and_operators() {
    let z = (0.3, -0.7);
    let c = (-0.5, 0.1);
    // Parameters p1…p5, in order.
    assert_eq!(bits(step("z^2 + p1", z, (9.0, 9.0), &[c])), bits(step("z^2 + c", z, c, &[])));
    assert_eq!(step("p2", z, c, &[(1.0, 0.0), (0.25, -2.0)]), (0.25, -2.0));
    // Complex literals, folding, pi and e.
    assert_eq!(step("(0.5, -0.25)", z, c, &[]), (0.5, -0.25));
    assert_eq!(step("(-1.5, 2) * 2", z, c, &[]), (-3.0, 4.0));
    assert_eq!(step("2^3 - 1", z, c, &[]), (7.0, 0.0));
    assert_eq!(step("pi", z, c, &[]), (std::f64::consts::PI, 0.0));
    assert_eq!(step("e", z, c, &[]), (std::f64::consts::E, 0.0));
    // A real factor becomes a scale; a power-of-two divisor too; any other divisor divides.
    let prog = |src: &str| parse(src).unwrap().phases()[0].clone();
    assert!(prog("3*z").insts().iter().any(|op| matches!(op, Op::Scale(_, k) if *k == 3.0)));
    assert!(prog("z/4").insts().iter().any(|op| matches!(op, Op::Scale(_, k) if *k == 0.25)));
    assert!(prog("z/3").insts().iter().any(|op| matches!(op, Op::Div(..))));
    assert_eq!(step("z/3", (3.0, 6.0), c, &[]), (1.0, 2.0));
    // Unary minus binds looser than `^`; `^` is right-associative; negative and real exponents.
    assert_eq!(step("-z^2", (0.0, 1.0), c, &[]), (1.0, -0.0));
    assert_eq!(step("2^3^2", z, c, &[]), (512.0, 0.0));
    let r = step("z^-1", (0.0, 2.0), c, &[]);
    assert!((r.0).abs() < 1e-15 && (r.1 + 0.5).abs() < 1e-15);
    assert!(prog("z^2.5").insts().iter().any(|op| matches!(op, Op::Pow(..))));
    assert_eq!(step("z^0", z, c, &[]), (1.0, 0.0));
    // |z| is the SQUARED modulus (Fractint); cabs is the modulus; abs takes both parts.
    assert_eq!(step("|z|", (3.0, -4.0), c, &[]), (25.0, 0.0));
    assert_eq!(step("cabs(z)", (3.0, -4.0), c, &[]), (5.0, 0.0));
    assert_eq!(step("abs(z)", (-3.0, -4.0), c, &[]), (3.0, 4.0));
    assert_eq!(step("real(z) + imag(z)", (3.0, -4.0), c, &[]), (-1.0, 0.0));
    assert_eq!(step("flip(z)", (3.0, -4.0), c, &[]), (-4.0, 3.0));
    assert_eq!(step("conj(z)", (3.0, -4.0), c, &[]), (3.0, 4.0));
    let r = step("recip(z)", (0.0, 2.0), c, &[]);
    assert!((r.0).abs() < 1e-15 && (r.1 + 0.5).abs() < 1e-15);
    // Folded constants leave no dead instructions behind.
    assert_eq!(prog("2*3*4 + z").insts().len(), 3, "z, the folded 24, and the sum");
    assert_eq!(parse("z^2 + p3").unwrap().param_count(), 3);
}

#[test]
fn every_named_function_parses_and_evaluates() {
    let z = (0.3, -0.7);
    for name in ["exp", "log", "sqrt", "sin", "cos", "tan", "sinh", "cosh", "tanh", "sqr", "abs", "conj", "real",
        "imag", "cabs", "flip", "recip", "ident", "cotan", "cotanh"]
    {
        let r = step(&format!("{name}(z)"), z, (0.0, 0.0), &[]);
        assert!(r.0.is_finite() && r.1.is_finite(), "{name}(z) = {r:?}");
    }
    // Two spot values against f64's own functions.
    let r = step("exp(z)", (1.0, 0.0), (0.0, 0.0), &[]);
    assert!((r.0 - std::f64::consts::E).abs() < 1e-15 && r.1 == 0.0);
    let r = step("cotan(z)", (0.5, 0.0), (0.0, 0.0), &[]);
    assert!((r.0 - 1.0 / 0.5f64.tan()).abs() < 1e-14);
}

#[test]
fn errors_name_the_problem_and_the_place() {
    let err = |src: &str| parse(src).expect_err(src);
    let e = err("z = z^2 +\n");
    assert_eq!((e.line, e.message.contains("expected a value")), (1, true), "{e}");
    let e = err("z = z^2 + c\nz = w + c");
    assert_eq!((e.line, e.col), (2, 5), "{e}");
    assert!(e.message.contains("`w` is used before it is assigned"), "{e}");
    assert!(err("z = sin(z").message.contains("expected ')'"));
    assert!(err("z = fn1(z) + c").message.contains("fn1"));
    assert!(err("if (|z| > 4)").message.contains("`if`"));
    assert!(err("z = z < 2").message.contains("comparison"));
    assert!(err("init: z = 0").message.contains("sections"));
    assert!(err("c = 3").message.contains("cannot be assigned"));
    assert!(err("sin = 3").message.contains("cannot be assigned"));
    assert!(err("z = (z, 1)").message.contains("two real numbers"));
    assert!(err("z = foo(z)").message.contains("unknown function `foo`"));
    assert!(err("z = z^2 + p6").message.contains("used before it is assigned"), "p6 is not a parameter");
    let e = err("z = z^2\n  + c @");
    assert_eq!((e.line, e.col), (2, 7), "{e}");
    assert!(err("").message.contains("no step"));
    assert!(err("; only a comment").message.contains("no step"));
    assert!(err("t = z").message.contains("no step"), "assigning only a temporary is not a step");
}
