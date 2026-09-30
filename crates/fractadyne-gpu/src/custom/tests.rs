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
    for m in SLOTS.iter().flat_map(|(b, e)| [*b, *e]) {
        assert_eq!(FIXED.matches(m).count(), 1, "{m}");
        let (s, e) = marker_line(FIXED, m).unwrap();
        assert!(FIXED[s..e].trim_start().starts_with("//"), "{m} must sit on a comment line");
    }
    // And the fixed module itself still validates (the markers are comments; nothing else moved).
    validate(FIXED).unwrap();
}

#[test]
fn every_built_in_step_generates_a_valid_module_with_its_own_floatexp_paths() {
    let region = |b, e| {
        let (s0, _) = marker_line(FIXED, b).unwrap();
        let (_, s1) = marker_line(FIXED, e).unwrap();
        &FIXED[s0..s1]
    };
    let fe = region(FE_BEGIN, FE_END);
    assert!(fe.len() > 10_000, "the replaced region should be the fixed floatexp path ({} bytes)", fe.len());
    assert!(fe.contains("Floatexp perturbation (mode 2)"), "the replaced region is not the floatexp path");
    let chunk_fe = region(CHUNK_FE_BEGIN, CHUNK_FE_END);
    assert!(chunk_fe.contains("fn fs_iterate_chunk_fe(") && chunk_fe.contains("Series-approximation seeding"));
    assert!(chunk_fe.contains("STEP-BOUNDED PASSES") && chunk_fe.trim_end().ends_with("CUSTOM_CHUNK_FE_END"));
    let mut built = 0;
    for id in 0..f::COUNT {
        let shader = build(&single(builtin_step(id).unwrap()), &[]).unwrap_or_else(|e| panic!("formula {id}: {e}"));
        let s = &shader.source;
        assert!(s.contains("var zn: Cdf = custom_tame(custom_step(z, c, zprev, iter));"));
        // The direct, df32 and floatexp smooth values, clamped.
        assert_eq!(s.matches("let smit = max(f32(iter) + 1.0 - nu, 0.0);").count(), 4, "every smooth value clamped");
        // And the resumable chunk passes: the direct step (z_{n-1} carried in the derivative's
        // slot), their smooth values, perturbed steps and rebases, and the resolve's DE.
        assert!(s.contains("let zn = custom_tame(custom_step(z, c, zp, iter));"), "the chunk pass's direct step");
        assert_eq!(s.matches("max(f32(iter) + 1.0 - nu, 0.0);").count(), 6, "all six smooth values clamped");
        assert_eq!(s.matches("dz = custom_tame(custom_pstep(z, dz, dc, iter));").count(), 2, "both mode-0 step slots, tamed");
        // Mode 2: the generated floatexp step (and the df32 tail's df32 one), in both passes.
        assert_eq!(s.matches("dz = custom_tame_fe(custom_fstep(Z, dz, dc, iter));").count(), 2, "both mode-2 slots");
        assert_eq!(s.matches("custom_tame(custom_pstep(Z, fe_to_cdf(dz), tail_dc(dc), iter))").count(), 2, "both tails");
        assert_eq!(s.matches("let base = iter % CUSTOM_PHASES;").count(), 4, "all four rebases phase-aligned");
        assert!(s.contains("dz = cset(df_sub(z_full_re, r0.re), df_sub(z_full_im, r0.im));"), "the chunk rebase's names");
        assert!(s.contains("    let nrm = vec2<f32>(0.0, 0.0);\n    let de = 1.0e30;\n"), "the resolve's no-DE values");
        assert!(!s.contains("let de = de_log2(mag2, d.x * d.x + d.y * d.y, sm.w);"), "the resolve's DE survived");
        assert!(!s.contains(fe), "formula {id}: the fixed floatexp path is still present");
        assert!(!s.contains(chunk_fe), "formula {id}: the fixed floatexp chunk pass is still present");
        assert_eq!(s.matches("fn fs_iterate_chunk_fe(").count(), 1, "formula {id}: one floatexp chunk entry point");
        assert!(SLOTS.iter().all(|(b, e)| !s.contains(b) && !s.contains(e)), "a marker survived");
        assert!(s.contains("fn fs_iterate(") && s.contains("fn vs_split_tiles("));
        assert_eq!(shader.precision, Precision::Df32, "formula {id}");
        // Every step but Phoenix's is perturbable (it reads z_prev); Newton's divides, which has
        // a perturbed form.
        assert_eq!(shader.perturbation.is_ok(), id != f::PHOENIX, "formula {id}: {:?}", shader.perturbation);
        assert_eq!(s.contains("fn custom_pphase0("), id != f::PHOENIX);
        assert_eq!(s.contains("fn custom_fphase0("), id != f::PHOENIX);
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
        // Every function has a perturbed form, carried through the small-argument helpers.
        assert!(s.perturbation.is_ok(), "{name}: {:?}", s.perturbation);
        let small = [
            "cf_sin_small(", "cf_sinh_small(", "cf_expm1(", "cf_tanh_diff(", "cf_tan_diff(", "cf_log_diff(", "cf_sqrt_diff(",
        ];
        let pstep = &s.source[s.source.find("fn custom_pphase0(").expect("a perturbed step")..];
        assert!(small.iter().any(|h| pstep.contains(h)), "{name}: no small-argument helper in its perturbed step");
    }
    // The shader's tanh_diff branches where the interpreter's does.
    let split = format!("if (abs(p.re.x) >= {:?}) {{", fractadyne_core::ir::TANH_DIFF_SPLIT as f32);
    assert!(F32_HELPERS.contains(&split), "cf_tanh_diff's split is not TANH_DIFF_SPLIT ({split})");
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
    // NaN is an overflowed component (∞ − ∞ in a df32 add, on a compiler that keeps the two-sum):
    // as big as ∞, with no sign to keep.
    assert_eq!(tame_f64((f64::NAN, -1.0e16)), (t, -t), "NaN overflowed: +TAME");
    assert_eq!(tame_f64((f64::NAN, 3.0)), (t, 3.0), "NaN beside an ordinary part");
    assert_eq!(tame_f64((f64::NAN, f64::NAN)), (t, t), "all NaN: |z| as both parts overflowed");
    // The shader source carries the same threshold, as bits.
    let s = build(&single(builtin_step(f::MANDELBROT).unwrap()), &[]).unwrap().source;
    assert!(s.contains(&format!("{:#x}u", TAME.to_bits())));
}

/// The estimate must over-price every measured built-in (direct factors from
/// validation/calibration/ceilings.toml, RTX 3080): a guard against device loss may err only high.
#[test]
fn the_cost_estimate_over_prices_every_measured_built_in() {
    let measured = [
        (f::MANDELBROT, 1.00),
        (f::MULTIBROT3, 1.27),
        (f::MULTIBROT4, 1.54),
        (f::MULTIBROT5, 1.77),
        (f::TRICORN, 0.86),
        (f::BURNING_SHIP, 0.81),
        (f::CELTIC, 0.87),
        (f::BUFFALO, 0.94),
        (f::PHOENIX, 1.24),
    ];
    for (id, direct) in measured {
        let est = cost_factor(&single(builtin_step(id).unwrap()));
        assert!(est >= direct * 1.15, "formula {id}: estimate {est:.2} vs measured {direct}");
        assert!(est <= direct * 3.5, "formula {id}: estimate {est:.2} is uselessly high vs {direct}");
    }
    // A dearer formula prices dearer.
    let deep = |src_ops: u32| {
        let mut b = Builder::new();
        let z = b.push(Op::Z);
        let p = b.push(Op::PowI(z, src_ops));
        let c = b.push(Op::C);
        let out = b.push(Op::Add(p, c));
        cost_factor(&single(b.finish(out).unwrap()))
    };
    assert!(deep(9) > deep(3));
}

/// The floatexp step keeps every perturbation in floatexp: each rule lands on its floatexp form,
/// and df32 appears only for reference-side and full-size values.
#[test]
fn the_floatexp_step_keeps_the_perturbations_in_floatexp() {
    let fphase = |formula: &Formula| {
        let s = build(formula, &[]).unwrap().source;
        let at = s.find("fn custom_fphase0(").expect("a floatexp step");
        let end = at + s[at..].find("\n}\n").expect("its end");
        s[at..end].to_string()
    };
    let parse = |src: &str| fractadyne_core::ir::parse::parse(src).unwrap();
    // Mandelbrot: P(z² + c) = (2Z + δz)·δz + δc. The full-size 2Z + δz is the one df32 value built
    // from a perturbation (the only `fe_to_cdf`); the product and the sum stay floatexp, and so
    // does the result.
    let m = fphase(&single(builtin_step(f::MANDELBROT).unwrap()));
    assert_eq!(m.matches("fe_to_cdf(").count(), 1, "{m}");
    assert!(m.contains("fe_mul_cdf(") && m.contains("fe_add("), "{m}");
    assert!(!m.contains("fe_from_cdf("), "a floatexp result, not a collapsed one: {m}");
    // Each fold and function on its floatexp form.
    for (formula, helper) in [
        (single(builtin_step(f::BURNING_SHIP).unwrap()), "sf_diffabs("),
        (parse("sin(z) + c"), "fe_sin_small("),
        (parse("sinh(z) + c"), "fe_sinh_small("),
        (parse("exp(z) + c"), "fe_expm1("),
        (parse("tanh(z) + c"), "fe_tanh_diff("),
        (parse("tan(z) + c"), "fe_tan_diff("),
        (parse("z^2 + c/(z + 2)"), "fe_div_cdf("),
        (parse("log(z*z + 1) + c"), "fe_log_diff("),
        (parse("sqrt(z) + c"), "fe_sqrt_diff("),
        (parse("z^2.5 + c"), "fe_pow_diff("),
        (parse("(z + 1)^(0.5, 0.25) + c"), "fe_pow_diff("),
    ] {
        let p = fphase(&formula);
        assert!(p.contains(helper), "{helper} missing: {p}");
        assert!(!p.contains("fe_from_cdf(v"), "a perturbation collapsed and re-expanded: {p}");
    }
    // A non-perturbable formula (a power whose exponent varies) still gets the (never dispatched)
    // stub the mode-2 loop calls.
    let s = build(&parse("z^c + c"), &[]).unwrap().source;
    assert!(s.contains("fn custom_fstep(z: Cdf, dz: Fe, dc: Fe, iter: u32) -> Fe { return dz; }"));
}

#[test]
fn the_key_follows_the_source() {
    let m = build(&single(builtin_step(f::MANDELBROT).unwrap()), &[]).unwrap();
    let again = build(&single(builtin_step(f::MANDELBROT).unwrap()), &[]).unwrap();
    let ship = build(&single(builtin_step(f::BURNING_SHIP).unwrap()), &[]).unwrap();
    assert_eq!(m.key, again.key);
    assert_ne!(m.key, ship.key);
}
