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

/// The power families as one types them (the collection ships several): each the built-in bit for
/// bit, so a custom formula and the family it spells agree (design/power-families.md).
#[test]
fn typed_power_families_are_the_built_ins_bit_for_bit() {
    for d in 6..=8u32 {
        same_as_builtin(&format!("z^{d} + c"), f::MULTIBROT6 + d - 6);
    }
    for d in 3..=5u32 {
        same_as_builtin(&format!("abs(z)^{d} + c"), f::BURNING_SHIP3 + d - 3);
        same_as_builtin(&format!("conj(z)^{d} + c"), f::TRICORN3 + d - 3);
        same_as_builtin(&format!("abs(real(z^{d})) + flip(imag(z^{d})) + c"), f::CELTIC3 + d - 3);
        same_as_builtin(&format!("abs(z^{d}) + c"), f::BUFFALO3 + d - 3);
    }
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
    assert!(err("if (|z| > 4)").message.contains("`endif`"));
    assert!(err("z = z^2\nelse\nz = c").message.contains("without an `if`"));
    assert!(err("z = 0: z = z^2: z = c").message.contains("second ':'"));
    assert!(err("if = 3").message.contains("cannot be assigned"));
    assert!(err("z = z & c").message.contains("&&"));
    assert!(err("{ z = z^2 + c }").message.contains(".frm"));
    // `init` is no keyword: the language's init section is everything before a ':'.
    assert!(err("init: z = 0").message.contains("`init` is used before it is assigned"));
    assert!(err("c = 3").message.contains("cannot be assigned"));
    assert!(err("sin = 3").message.contains("cannot be assigned"));
    assert!(err("z = (z, 1)").message.contains("two real numbers"));
    assert!(err("z = foo(z)").message.contains("unknown function `foo`"));
    // Another notation's name for a function says the language's; any case, as names are read.
    let e = err("z = LN(z) + c");
    assert!(e.message.contains("natural logarithm is written `log`"), "{e}");
    assert!(err("z = cot(z)").message.contains("`cotan`"));
    // Only as a call: a variable may be called ln.
    assert!(parse("ln = 2, z = z^2 + ln*c").is_ok());
    assert_eq!(our_spelling("Ln"), Some("log"));
    assert_eq!(our_spelling("log"), None);
    // Every spelling's target is a function, and none of them is one already.
    for (other, ours, _) in OTHER_SPELLINGS {
        assert!(is_function(ours) && !is_function(other), "{other} -> {ours}");
    }
    assert!(err("z = z^2 + p6").message.contains("used before it is assigned"), "p6 is not a parameter");
    let e = err("z = z^2\n  + c @");
    assert_eq!((e.line, e.col), (2, 7), "{e}");
    assert!(err("").message.contains("no step"));
    assert!(err("; only a comment").message.contains("no step"));
    assert!(err("t = z").message.contains("no step"), "assigning only a temporary is not a step");
}

// ---- Fractint's sections ----

/// `orbit_points` of `src` and of a hand-written loop agree bit for bit over a grid of pixels.
fn same_orbits(src: &str, hand: impl Fn((f64, f64)) -> Vec<(f64, f64)>) {
    let f = parse(src).unwrap_or_else(|e| panic!("{src:?}: {e}"));
    let mut points = 0;
    for j in 0..12 {
        for i in 0..16 {
            let c = (-2.0 + 3.0 * i as f64 / 15.0, -1.4 + 2.8 * j as f64 / 11.0);
            let got = orbit_points(&f, (0.0, 0.0), c, &[(0.6, 0.0)], 300, 1.0e300).unwrap();
            let want = hand(c);
            assert_eq!(
                got.iter().map(|p| bits(*p)).collect::<Vec<_>>(),
                want.iter().map(|p| bits(*p)).collect::<Vec<_>>(),
                "{src:?} at {c:?}"
            );
            points += want.len();
        }
    }
    assert!(points > 500, "{src:?}: too few points ({points})");
}

fn cmul(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}
fn csq(a: (f64, f64)) -> (f64, f64) {
    (a.0 * a.0 - a.1 * a.1, 2.0 * (a.0 * a.1))
}
fn cadd(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 + b.0, a.1 + b.1)
}
fn norm(a: (f64, f64)) -> f64 {
    a.0 * a.0 + a.1 * a.1
}

/// An init section runs once, from z₀; a final comparison is the bailout, evaluated with the
/// step's values — Fractint's Mandelbrot is the Mandelbrot set with escape radius 2.
#[test]
fn an_init_section_and_a_bailout() {
    let f = parse("z = 0.5:\n  z = z*z + pixel\n  |z| <= 4").unwrap();
    assert!(f.init().is_some() && f.has_bailout() && f.vars() == 0);
    assert!(!f.bignum_evaluable(), "a sectioned formula has no reference orbit");
    same_orbits("z = 0.5:\n  z = z*z + pixel\n  |z| <= 4", |c| {
        let mut z = (0.5, 0.0);
        let mut out = vec![z];
        for _ in 0..300 {
            z = cadd(csq(z), c);
            out.push(z);
            if !(norm(z) <= 4.0) {
                break;
            }
        }
        out
    });
    // Without a comparison at the end, the escape radius decides, as before.
    assert!(!parse("z = 0.5: z = z^2 + c").unwrap().has_bailout());
}

/// Variables persist from step to step: Manowar carries the step before's z, and a name read
/// before its statement is its value from the step before (0 at first).
#[test]
fn variables_persist_from_step_to_step() {
    let f = parse("z = pixel, z1 = pixel:\n  t = z\n  z = sqr(z) + z1 + pixel\n  z1 = t\n  |z| <= 4").unwrap();
    assert_eq!(f.vars(), 1, "z1 carries over; t does not");
    same_orbits("z = pixel, z1 = pixel:\n  t = z\n  z = sqr(z) + z1 + pixel\n  z1 = t\n  |z| <= 4", |c| {
        let (mut z, mut z1) = (c, c);
        let mut out = vec![z];
        for _ in 0..300 {
            let t = z;
            z = cadd(cadd(csq(z), z1), c);
            z1 = t;
            out.push(z);
            if !(norm(z) <= 4.0) {
                break;
            }
        }
        out
    });
    // No init section: `w` starts at 0 and is read before its statement each step.
    same_orbits("z = z^2 + w + c\nw = z*p1", |c| {
        let (mut z, mut w) = ((0.0, 0.0), (0.0, 0.0));
        let mut out = vec![z];
        for _ in 0..300 {
            z = cadd(cadd(csq(z), w), c);
            w = cmul(z, (0.6, 0.0));
            out.push(z);
            if norm(z) > 1.0e300 {
                break;
            }
        }
        out
    });
    // A name nothing assigns is still a mistake.
    assert!(parse("z = z^2 + zz + c").unwrap_err().message.contains("`zz` is used before it is assigned"));
}

/// `if … elseif … else … endif` keeps one branch's values; a variable a branch leaves alone keeps
/// its value from before the block.
#[test]
fn if_blocks_keep_one_branch() {
    let src = "z = pixel:\n  if (real(z) >= 0)\n    z = (z - 1)*p1\n  elseif (imag(z) > 0.5)\n    z = z*z\n  else\n    z = (z + 1)*p1\n  endif\n  |z| <= 4";
    same_orbits(src, |c| {
        let mut z = c;
        let mut out = vec![z];
        let p1 = (0.6, 0.0);
        for _ in 0..300 {
            z = if z.0 >= 0.0 {
                cmul((z.0 - 1.0, z.1), p1)
            } else if z.1 > 0.5 {
                cmul(z, z)
            } else {
                cmul((z.0 + 1.0, z.1), p1)
            };
            out.push(z);
            if !(norm(z) <= 4.0) {
                break;
            }
        }
        out
    });
    // A branch that leaves `w` alone: `w` keeps its value (carried, as it is read before it is set).
    let f = parse("if (real(z) > 0)\n  w = z\nendif\nz = z^2 + w + c").unwrap();
    assert_eq!(f.vars(), 1);
}

/// Comparisons compare real parts and give 1 or 0; `&&` and `||` take real parts too.
#[test]
fn comparisons_and_logic_give_one_or_zero() {
    let cases = [
        ("(z < 2) + 0*c", (1.5, 9.0), 1.0),
        ("(z < 2) + 0*c", (2.5, -9.0), 0.0),
        ("(z <= 2) + (z >= 2) + 0*c", (2.0, 5.0), 2.0),
        ("(z == 2) + (z != 2) + 0*c", (2.0, 1.0), 1.0),
        ("(z > 1 && z < 3) + 0*c", (2.0, 0.0), 1.0),
        ("(z > 1 && z < 3) + 0*c", (4.0, 0.0), 0.0),
        ("(z < 1 || z > 3) + 0*c", (4.0, 0.0), 1.0),
        ("(z < 1 || z > 3) + 0*c", (2.0, 0.0), 0.0),
    ];
    for (src, z, want) in cases {
        assert_eq!(step(src, z, (0.0, 0.0), &[]), (want, 0.0), "{src} at {z:?}");
    }
    // Looser than arithmetic, tighter than nothing: `a + 1 < b` is `(a + 1) < b`.
    assert_eq!(step("(z + 1 < 3) + 0*c", (1.5, 0.0), (0.0, 0.0), &[]), (1.0, 0.0));
}

/// Fractint's assignment is a value where its value is the whole of what is read: chained, or
/// opening parentheses (an `if`'s condition among them). Elsewhere an `=` is still an error.
#[test]
fn assignment_is_a_value_where_fractint_writes_one() {
    assert_eq!(step("a = b = c*2, a + b", (0.0, 0.0), (0.5, 0.25), &[]), (2.0, 1.0));
    assert_eq!(step("w = (z = z*2) + 1, w + z", (1.5, 0.0), (0.0, 0.0), &[]), (7.0, 0.0));
    assert_eq!(step("if ((d = |z|) < 4)\n z = d\nendif\nz + 0*c", (1.0, 1.0), (0.0, 0.0), &[]), (2.0, 0.0));
    assert_eq!(step("sqr(t = z + 1) + t", (1.0, 0.0), (0.0, 0.0), &[]), (6.0, 0.0));
    let e = parse("z = p1^z = 2").expect_err("an assignment inside a power");
    assert!(e.message.contains("found '='"), "{e}");
    assert!(parse("z = (c = 2)").expect_err("c").message.contains("cannot be assigned"));
    // The tree holds the inner assignment, its span inside the statement's.
    let s = crate::ir::parse::syntax("a = b = pixel").unwrap();
    assert!(matches!(&s.statements[0].body.kind, ExprKind::Assign(n, _) if n.name == "b"));
}

/// Fractint's predefined variables may be assigned; each holds its predefined value until it is
/// (in the loop, a read before the assignment is the step before's value, the first time the
/// predefined one). `c`, this language's pixel, may not.
#[test]
fn predefined_names_can_be_assigned() {
    same_orbits("pixel = pixel*0.5, z = z*z + pixel, |z| <= 4", |c| {
        let (mut p, mut z) = (c, (0.0, 0.0));
        let mut out = vec![z];
        for _ in 0..300 {
            p = (p.0 * 0.5, p.1 * 0.5);
            z = cadd(csq(z), p);
            out.push(z);
            if !(norm(z) <= 4.0) {
                break;
            }
        }
        out
    });
    // Set in the init section, read in the loop: p1 is 0.6 + 1 there.
    same_orbits("p1 = p1 + 1:\n z = z*z + p1*pixel\n |z| <= 4", |c| {
        let mut z = (0.0, 0.0);
        let mut out = vec![z];
        for _ in 0..300 {
            z = cadd(csq(z), cmul((1.6, 0.0), c));
            out.push(z);
            if !(norm(z) <= 4.0) {
                break;
            }
        }
        out
    });
    assert_eq!(step("e = 2, z*e", (1.5, 0.0), (0.0, 0.0), &[]), (3.0, 0.0));
}

/// With a bailout, a loop need not set z: the test alone decides (Fractint draws such "formulas"
/// of the pixel alone).
#[test]
fn a_bailout_alone_is_a_step() {
    let f = parse("|pixel| < 4").unwrap();
    assert_eq!(orbit_points(&f, (0.0, 0.0), (3.0, 0.0), &[], 50, 1.0e300).unwrap().len(), 2, "fails at once");
    assert_eq!(orbit_points(&f, (0.0, 0.0), (1.0, 0.0), &[], 50, 1.0e300).unwrap().len(), 51, "holds to the cap");
    assert!(parse("t = z").is_err(), "without one, a loop that sets no z is still no step");
}

/// `maxit` is the iteration cap (0 for a lone step), `ismand` 1; neither has a deep-zoom form.
#[test]
fn maxit_and_ismand() {
    let f = parse("z*0 + maxit + ismand").unwrap();
    assert_eq!(orbit_points(&f, (0.0, 0.0), (0.0, 0.0), &[], 7, 1.0e300).unwrap()[1], (8.0, 0.0));
    assert_eq!(step("z*0 + maxit + ismand", (0.0, 0.0), (0.0, 0.0), &[]), (1.0, 0.0));
    assert!(!f.bignum_evaluable());
}

/// Fractint's rounding, part by part; its `round` is `floor(x + 0.5)`.
#[test]
fn rounding_functions_round_each_part() {
    let z = (-1.5, 2.5);
    assert_eq!(step("floor(z)", z, (0.0, 0.0), &[]), (-2.0, 2.0));
    assert_eq!(step("ceil(z)", z, (0.0, 0.0), &[]), (-1.0, 3.0));
    assert_eq!(step("trunc(z)", z, (0.0, 0.0), &[]), (-1.0, 2.0));
    assert_eq!(step("round(z)", z, (0.0, 0.0), &[]), (-1.0, 3.0));
    assert!(!parse("floor(z) + c").unwrap().bignum_evaluable());
}

/// An `if`'s condition opens with a parenthesis and runs on (`if (a) || (b)`); a function's
/// parentheses may hold a complex constant (`sin(1, 2)`).
#[test]
fn fractints_looser_forms() {
    let src = "if (real(z) > 0) || (imag(z) > 0)\n z = z*2\nendif\nz = z + c";
    assert_eq!(step(src, (1.0, -1.0), (0.5, 0.0), &[]), (2.5, -2.0));
    assert_eq!(step(src, (-1.0, -1.0), (0.5, 0.0), &[]), (-0.5, -1.0));
    assert_eq!(step("sin(1, 2) + 0*z", (0.0, 0.0), (0.0, 0.0), &[]), step("sin((1, 2)) + 0*z", (0.0, 0.0), (0.0, 0.0), &[]));
    assert!(parse("z = sin(z, 2)").expect_err("not constant").message.contains("two real numbers"));
}

/// `p0` (and `p6` on) are ordinary names, not parameters — `p0` once underflowed the index.
#[test]
fn names_past_the_parameters_are_variables() {
    assert_eq!(step("p0 = z*2, p0 + c", (1.5, 0.0), (0.25, 0.0), &[]), (3.25, 0.0));
    assert!(parse("z = z*z + p0").is_err(), "an unset `p0` is an unknown name");
}

/// The inverse functions invert, on the principal branches; `cosxx` is the cosine's conjugate.
#[test]
fn the_inverse_functions_invert() {
    let w = (0.3, 0.2);
    let close = |a: (f64, f64), b: (f64, f64)| (a.0 - b.0).abs() < 1e-12 && (a.1 - b.1).abs() < 1e-12;
    for (inv, fwd) in [("asin", "sin"), ("acos", "cos"), ("atan", "tan"), ("asinh", "sinh"), ("acosh", "cosh"), ("atanh", "tanh")] {
        let y = step(&format!("{fwd}(z) + 0*c"), w, (0.0, 0.0), &[]);
        let back = step(&format!("{inv}(z) + 0*c"), y, (0.0, 0.0), &[]);
        assert!(close(back, w), "{inv}({fwd}({w:?})) = {back:?}");
    }
    let c = step("cos(z) + 0*c", w, (0.0, 0.0), &[]);
    assert_eq!(step("cosxx(z) + 0*c", w, (0.0, 0.0), &[]), (c.0, -c.1));
}

/// The syntax tree holds comparisons and logic, and an `if` block's statements.
#[test]
fn the_tree_holds_comparisons_and_branches() {
    use crate::ir::syntax::{ExprKind, Logic};
    let s = syntax("z = 0:\nif (real(z) > 0 && imag(z) < 1)\n  z = z^2\nendif\n|z| <= 4").unwrap();
    assert_eq!(s.statements.len(), 3, "z = 0, the branch's z = z^2, the bailout");
    assert!(matches!(s.statements[2].body.kind, ExprKind::Cmp(crate::ir::Cmp::Le, ..)));
    let any_logic = |e: &crate::ir::syntax::Expr| matches!(e.kind, ExprKind::Logic(Logic::And, ..));
    assert!(!any_logic(&s.statements[1].body), "the condition is not a statement");
}
