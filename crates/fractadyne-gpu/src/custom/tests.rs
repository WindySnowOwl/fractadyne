use super::*;
use fractadyne_core::formula as f;
use fractadyne_core::ir::{builtin_step, Builder, IrError, Val};

fn single(prog: Program) -> Formula {
    Formula::single(prog)
}

fn unary(op: fn(Val) -> Op) -> Formula {
    let mut b = Builder::new();
    let z = b.push(Op::Z);
    let v = b.push(op(z));
    let c = b.push(Op::C);
    let out = b.push(Op::Add(v, c));
    single(b.finish(out).unwrap())
}

#[test]
fn the_fixed_module_has_each_slot_marker_once_as_a_comment() {
    for m in [STEP_BEGIN, STEP_END, SMOOTH_BEGIN, SMOOTH_END, CUT_BEGIN, CUT_END] {
        assert_eq!(FIXED.matches(m).count(), 1, "{m}");
        let (s, e) = marker_line(FIXED, m).unwrap();
        assert!(FIXED[s..e].trim_start().starts_with("//"), "{m} must sit on a comment line");
    }
    // And the fixed module itself still validates (the markers are comments; nothing else moved).
    validate(FIXED).unwrap();
}

#[test]
fn every_built_in_step_generates_a_valid_module_without_the_perturbation_paths() {
    let (c0, _) = marker_line(FIXED, CUT_BEGIN).unwrap();
    let (_, c1) = marker_line(FIXED, CUT_END).unwrap();
    let cut = &FIXED[c0..c1];
    assert!(cut.len() > 20_000, "the cut region should be the perturbation paths ({} bytes)", cut.len());
    let mut built = 0;
    for id in 0..f::COUNT {
        let shader = build(&single(builtin_step(id).unwrap()), &[]).unwrap_or_else(|e| panic!("formula {id}: {e}"));
        assert!(shader.source.contains("var zn: Cdf = custom_tame(custom_step(z, c, zprev, iter));"));
        assert!(shader.source.contains("let smit = max(f32(iter) + 1.0 - nu, 0.0);"));
        assert!(!shader.source.contains(cut), "formula {id}: perturbation paths still present");
        assert!(!shader.source.contains(STEP_BEGIN) && !shader.source.contains(CUT_BEGIN));
        assert!(shader.source.contains("fn fs_iterate(") && shader.source.contains("fn vs_split_tiles("));
        assert_eq!(shader.precision, Precision::Df32, "formula {id}");
        built += 1;
    }
    assert_eq!(built, 10);
}

#[test]
fn generated_built_in_steps_are_the_built_in_helper_calls() {
    // Mandelbrot: the fixed module's `zn = c_sqr(z); ... zn = c_add(zn, c);`.
    let s = build(&single(builtin_step(f::MANDELBROT).unwrap()), &[]).unwrap().source;
    assert!(s.contains(
        "fn custom_phase0(z: Cdf, c: Cdf, zp: Cdf) -> Cdf {\n    let v0 = z;\n    let v1 = c_sqr(v0);\n    \
         let v2 = c;\n    let v3 = c_add(v1, v2);\n    return v3;\n}"
    ));
    // Multibrot 3: `c_mul(c_sqr(z), z)`.
    let s = build(&single(builtin_step(f::MULTIBROT3).unwrap()), &[]).unwrap().source;
    assert!(s.contains("let v1 = c_sqr(v0);\n    let v2 = c_mul(v1, v0);"));
    // Phoenix's `0.5` is exact in f32, so it scales with `df_mul_f32` as the built-in does.
    let s = build(&single(builtin_step(f::PHOENIX).unwrap()), &[]).unwrap().source;
    assert!(s.contains("c_scale(v4, 0.5)"));
}

#[test]
fn powers_hybrids_parameters_and_functions_generate_valid_modules() {
    // z^7 + c: the square-and-multiply chain.
    let mut b = Builder::new();
    let z = b.push(Op::Z);
    let p = b.push(Op::PowI(z, 7));
    let c = b.push(Op::C);
    let out = b.push(Op::Add(p, c));
    let s = build(&single(b.finish(out).unwrap()), &[]).unwrap();
    assert!(s.source.contains("let v1_0 = c_mul(c_sqr(v0), v0);"));
    assert!(s.source.contains("let v1_1 = c_mul(c_sqr(v1_0), v0);"));
    assert_eq!(s.power, 7.0);

    // A three-phase hybrid: a switch over the iteration.
    let hybrid = Formula::new([f::MANDELBROT, f::BURNING_SHIP, f::MULTIBROT3].iter().map(|&id| builtin_step(id).unwrap()).collect()).unwrap();
    let s = build(&hybrid, &[]).unwrap();
    assert!(s.source.contains("switch (iter % 3u)") && s.source.contains("default: { return custom_phase2(z, c, zp); }"));
    assert!((s.power - 12f32.cbrt()).abs() < 1e-6);

    // z² + p0·z + c, parameter baked as a df32 literal.
    let mut b = Builder::new();
    let z = b.push(Op::Z);
    let s2 = b.push(Op::Sqr(z));
    let k = b.push(Op::Param(0));
    let kz = b.push(Op::Mul(k, z));
    let t = b.push(Op::Add(s2, kz));
    let c = b.push(Op::C);
    let out = b.push(Op::Add(t, c));
    let formula = single(b.finish(out).unwrap());
    assert!(matches!(build(&formula, &[]), Err(CustomError::Ir(IrError::MissingParam { index: 0, supplied: 0 }))));
    let s = build(&formula, &[(0.1, -0.25)]).unwrap();
    assert!(s.source.contains(&format!("cset(vec2<f32>(0.1, {:?}), vec2<f32>(-0.25, 0.0))", (0.1 - 0.1f32 as f64) as f32)));

    // Every elementary function and a complex power: valid, and the f32 tier.
    for (name, op) in [
        ("exp", (|v| Op::Func(Func::Exp, v)) as fn(Val) -> Op),
        ("log", |v| Op::Func(Func::Log, v)),
        ("sqrt", |v| Op::Func(Func::Sqrt, v)),
        ("sin", |v| Op::Func(Func::Sin, v)),
        ("cos", |v| Op::Func(Func::Cos, v)),
        ("tan", |v| Op::Func(Func::Tan, v)),
        ("sinh", |v| Op::Func(Func::Sinh, v)),
        ("cosh", |v| Op::Func(Func::Cosh, v)),
        ("tanh", |v| Op::Func(Func::Tanh, v)),
    ] {
        let s = build(&unary(op), &[]).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(s.precision, Precision::F32, "{name}");
        assert!(s.source.contains(&format!("cf_{name}(v0)")), "{name}");
    }
    let mut b = Builder::new();
    let z = b.push(Op::Z);
    let w = b.push(Op::Const(2.5, 0.5));
    let p = b.push(Op::Pow(z, w));
    let s = build(&single(b.finish(p).unwrap()), &[]).unwrap();
    assert_eq!(s.precision, Precision::F32);
    assert_eq!(s.power, 2.0, "no power law: smooth colouring falls back to 2");

    // Everything else in the op set, in one program.
    let mut b = Builder::new();
    let z = b.push(Op::Z);
    let n = b.push(Op::Neg(z));
    let cj = b.push(Op::Conj(n));
    let ar = b.push(Op::AbsRe(cj));
    let ai = b.push(Op::AbsIm(ar));
    let re = b.push(Op::Re(ai));
    let im = b.push(Op::Im(ai));
    let nm = b.push(Op::Norm(z));
    let sc = b.push(Op::Scale(re, 0.1));
    let d = b.push(Op::Div(sc, nm));
    let s1 = b.push(Op::Sub(d, im));
    let c = b.push(Op::C);
    let out = b.push(Op::Add(s1, c));
    let s = build(&single(b.finish(out).unwrap()), &[]).unwrap();
    assert!(s.source.contains("df_mul(v5.re, vec2<f32>(0.1,"), "an inexact f32 scale uses df_mul");
    assert_eq!(s.precision, Precision::Df32);
}

#[test]
fn constants_outside_f32_range_are_refused() {
    let mut b = Builder::new();
    let z = b.push(Op::Z);
    let k = b.push(Op::Const(1.0e300, 0.0));
    let out = b.push(Op::Add(z, k));
    assert_eq!(build(&single(b.finish(out).unwrap()), &[]).map(|s| s.key), Err(CustomError::OutOfRange(1.0e300)));
}

#[test]
fn the_cpu_tame_mirrors_the_shader_rules() {
    let t = TAME as f64;
    assert_eq!(tame_f64((5.0, -6.0)), (5.0, -6.0), "ordinary values pass through");
    assert_eq!(tame_f64((1.0e20, -3.0)), (t, -3.0), "only the overflowing part is clamped");
    assert_eq!(tame_f64((-f64::INFINITY, 1.0e30)), (-t, t), "infinities keep their sign");
    assert_eq!(tame_f64((f64::NAN, -1.0e16)), (0.0, -t), "NaN has no direction");
    assert_eq!(tame_f64((f64::NAN, f64::NAN)), (t, 0.0), "all NaN still escapes");
    // The shader source carries the same threshold, as bits.
    let s = build(&single(builtin_step(f::MANDELBROT).unwrap()), &[]).unwrap().source;
    assert!(s.contains(&format!("{:#x}u", TAME.to_bits())));
}

#[test]
fn the_key_follows_the_source() {
    let m = build(&single(builtin_step(f::MANDELBROT).unwrap()), &[]).unwrap();
    let again = build(&single(builtin_step(f::MANDELBROT).unwrap()), &[]).unwrap();
    let ship = build(&single(builtin_step(f::BURNING_SHIP).unwrap()), &[]).unwrap();
    assert_eq!(m.key, again.key);
    assert_ne!(m.key, ship.key);
}
