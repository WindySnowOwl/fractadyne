//! Custom formulas on the GPU (design/custom-formulas.md §4.5).
//!
//! A formula's IR ([`fractadyne_core::ir`]) becomes WGSL written in the shader's own df32 helpers
//! (`c_sqr`, `c_mul`, `df_abs`, …): `custom_step` (the direct step) and, when the formula is
//! perturbable ([`fractadyne_core::ir::perturb`]), `custom_pstep` (its perturbed step, δz' from the
//! reference `Z`, `δz` and `δc`). They are spliced into the fixed module at marked slots in
//! `iterate_at`:
//!
//! - `@@CUSTOM_STEP` / `@@CUSTOM_SMOOTH` — the direct path's formula step and smooth value;
//! - `@@CUSTOM_FE` — the floatexp perturbation path (mode 2), replaced by a leaner loop around the
//!   generated floatexp step `custom_fstep` (no BLA, SA, glitch detection or derivative, which are
//!   most of the fixed path's compile time, §3 A6);
//! - `@@CUSTOM_PSTEP` / `@@CUSTOM_REBASE` / `@@CUSTOM_SMOOTH0` — the df32 perturbation path's
//!   (mode 0) formula step, rebase and smooth value. The rebase is PHASE-ALIGNED for a hybrid: the
//!   reference index restarts at `iter mod phases`, not 0, so reference and pixel stay in the same
//!   phase (identical to the fixed module's for one phase);
//! - the resumable chunk passes' slots likewise, and `@@CUSTOM_CHUNK_FE` — the whole floatexp chunk
//!   entry point, replaced by the chunked form of the same loop.
//!
//! The fixed module carries only the marker COMMENTS, so every built-in pipeline compiles from the
//! same code as before. A custom module renders under [`fractadyne_core::formula::CUSTOM`], an id no
//! built-in branch matches, so Newton's special case and the built-in distance estimate stay off.
//!
//! Each IR operation maps to the helper the built-ins use for it, in the same order the CPU
//! interpreter uses (`Sqr` → `c_sqr`, `Mul` → `c_mul`, `PowI` → the same square-and-multiply chain),
//! so a generated built-in step is the built-in step's own helper calls.

use fractadyne_core::ir::{Formula, Func, Op, Program, Val};

const FIXED: &str = include_str!("mandelbrot.wgsl");
const STEP_BEGIN: &str = "// @@CUSTOM_STEP_BEGIN";
const STEP_END: &str = "// @@CUSTOM_STEP_END";
const SMOOTH_BEGIN: &str = "// @@CUSTOM_SMOOTH_BEGIN";
const SMOOTH_END: &str = "// @@CUSTOM_SMOOTH_END";
const FE_BEGIN: &str = "// @@CUSTOM_FE_BEGIN";
const FE_END: &str = "// @@CUSTOM_FE_END";
const PSTEP_BEGIN: &str = "// @@CUSTOM_PSTEP_BEGIN";
const PSTEP_END: &str = "// @@CUSTOM_PSTEP_END";
const REBASE_BEGIN: &str = "// @@CUSTOM_REBASE_BEGIN";
const REBASE_END: &str = "// @@CUSTOM_REBASE_END";
const SMOOTH0_BEGIN: &str = "// @@CUSTOM_SMOOTH0_BEGIN";
const SMOOTH0_END: &str = "// @@CUSTOM_SMOOTH0_END";
// The resumable chunk pass (`fs_iterate_chunk`) and its resolve: the same five slots again, so a
// custom formula splits its iterations across passes like a built-in.
const CHUNK_STEP_BEGIN: &str = "// @@CUSTOM_CHUNK_STEP_BEGIN";
const CHUNK_STEP_END: &str = "// @@CUSTOM_CHUNK_STEP_END";
const CHUNK_SMOOTH_BEGIN: &str = "// @@CUSTOM_CHUNK_SMOOTH_BEGIN";
const CHUNK_SMOOTH_END: &str = "// @@CUSTOM_CHUNK_SMOOTH_END";
const CHUNK_PSTEP_BEGIN: &str = "// @@CUSTOM_CHUNK_PSTEP_BEGIN";
const CHUNK_PSTEP_END: &str = "// @@CUSTOM_CHUNK_PSTEP_END";
const CHUNK_REBASE_BEGIN: &str = "// @@CUSTOM_CHUNK_REBASE_BEGIN";
const CHUNK_REBASE_END: &str = "// @@CUSTOM_CHUNK_REBASE_END";
const CHUNK_SMOOTH0_BEGIN: &str = "// @@CUSTOM_CHUNK_SMOOTH0_BEGIN";
const CHUNK_SMOOTH0_END: &str = "// @@CUSTOM_CHUNK_SMOOTH0_END";
const CHUNK_FE_BEGIN: &str = "// @@CUSTOM_CHUNK_FE_BEGIN";
const CHUNK_FE_END: &str = "// @@CUSTOM_CHUNK_FE_END";
const RESOLVE_DE_BEGIN: &str = "// @@CUSTOM_RESOLVE_DE_BEGIN";
const RESOLVE_DE_END: &str = "// @@CUSTOM_RESOLVE_DE_END";
/// Every slot, in file order: `(begin, end)`.
const SLOTS: [(&str, &str); 13] = [
    (STEP_BEGIN, STEP_END),
    (SMOOTH_BEGIN, SMOOTH_END),
    (FE_BEGIN, FE_END),
    (PSTEP_BEGIN, PSTEP_END),
    (REBASE_BEGIN, REBASE_END),
    (SMOOTH0_BEGIN, SMOOTH0_END),
    (CHUNK_STEP_BEGIN, CHUNK_STEP_END),
    (CHUNK_SMOOTH_BEGIN, CHUNK_SMOOTH_END),
    (CHUNK_PSTEP_BEGIN, CHUNK_PSTEP_END),
    (CHUNK_REBASE_BEGIN, CHUNK_REBASE_END),
    (CHUNK_SMOOTH0_BEGIN, CHUNK_SMOOTH0_END),
    (CHUNK_FE_BEGIN, CHUNK_FE_END),
    (RESOLVE_DE_BEGIN, RESOLVE_DE_END),
];

/// How much precision the generated step carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Precision {
    /// df32 throughout (~48 bits): the ring operations and division.
    Df32,
    /// An elementary function (or complex power) evaluates in `f32` (~24 bits): on the direct
    /// path the depth limit drops accordingly (their df32 forms are future work). A perturbed
    /// step does not mind: its functions take full-size reference values, where absolute
    /// accuracy suffices, and the small offsets go through relative-accuracy forms.
    F32,
}

/// A custom formula's complete shader module, ready to compile.
#[derive(Clone, Debug)]
pub struct CustomShader {
    /// The WGSL module: the fixed module with the generated step spliced in.
    pub source: String,
    /// FNV-1a of `source` — the pipeline-cache key.
    pub key: u64,
    /// The escape degree smooth colouring divides by (2 where the formula has none).
    pub power: f32,
    pub precision: Precision,
    /// The dispatch ceiling's per-step cost factor relative to Mandelbrot ([`cost_factor`]).
    pub cost_factor: f64,
    /// `Ok` when the module carries a perturbed step, in df32 (mode 0) and floatexp (mode 2);
    /// otherwise why not — the formula then renders on the direct path only.
    pub perturbation: Result<(), fractadyne_core::ir::perturb::NotPerturbable>,
}

/// A custom formula's cost per step relative to Mandelbrot, for the dispatch ceiling
/// (`validation/calibration/ceilings.toml` holds the measured built-ins'). ESTIMATED from the
/// generated code until the compile-time calibration pass exists (design §4.6): each operation's
/// df32 helper cost in units of one `df_add`/`df_mul`, plus a fixed per-iteration loop cost,
/// relative to Mandelbrot's, times a 1.5 margin. Against the measured direct factors this
/// over-prices every built-in — the safe direction for a device-loss guard (Multibrot 4: 2.1
/// against 1.54 measured; Multibrot 5: 2.9 against 1.77; Burning Ship: 1.6 against 0.81).
pub fn cost_factor(formula: &Formula) -> f64 {
    const LOOP: f64 = 5.0; // escape test, counters, bookkeeping — per iteration, any formula
    const MANDELBROT: f64 = 5.0 + 2.0 + LOOP; // c_sqr + c_add + the loop
    let per_phase: Vec<f64> = formula
        .phases()
        .iter()
        .map(|p| {
            p.insts()
                .iter()
                .map(|op| match *op {
                    Op::Z | Op::C | Op::ZPrev | Op::Param(_) | Op::Const(..) | Op::Delta | Op::DeltaC => 0.0,
                    Op::Neg(_) | Op::Conj(_) | Op::Re(_) | Op::Im(_) => 0.0,
                    Op::AbsRe(_) | Op::AbsIm(_) => 0.5,
                    Op::DiffAbsRe(..) | Op::DiffAbsIm(..) => 2.0,
                    Op::Add(..) | Op::Sub(..) | Op::Scale(..) => 2.0,
                    Op::Norm(_) => 3.0,
                    Op::Sqr(_) => 5.0,
                    Op::Mul(..) => 6.0,
                    Op::PowI(_, n) => {
                        let bits = 31 - n.leading_zeros();
                        5.0 * bits as f64 + 6.0 * (n.count_ones() - 1) as f64
                    }
                    Op::Div(..) => 15.0,
                    Op::Func(Func::Tan | Func::Tanh, _) => 18.0,
                    Op::Func(Func::Sin | Func::Cos | Func::Sinh | Func::Cosh, _) => 8.0,
                    Op::Func(Func::SinSmall | Func::SinhSmall | Func::Expm1, _) => 10.0,
                    Op::DiffTanh(..) | Op::DiffTan(..) => 50.0,
                    Op::Func(..) => 4.0,
                    Op::Pow(..) => 12.0,
                })
                .sum::<f64>()
        })
        .collect();
    let mean = per_phase.iter().sum::<f64>() / per_phase.len().max(1) as f64;
    (1.5 * (mean + LOOP) / MANDELBROT).max(1.0)
}

#[derive(Clone, Debug, PartialEq)]
pub enum CustomError {
    Ir(fractadyne_core::ir::IrError),
    /// A constant or parameter outside `f32`'s range, which the GPU cannot hold.
    OutOfRange(f64),
    /// The fixed shader's `marker` is missing or not unique (a shader edit broke the slots).
    Marker(&'static str),
    /// naga rejected the generated module — a code generator bug, reported rather than handed to
    /// the driver.
    Invalid(String),
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomError::Ir(e) => write!(f, "{e}"),
            CustomError::OutOfRange(v) => write!(f, "constant {v:e} is outside the GPU's range"),
            CustomError::Marker(m) => write!(f, "shader slot {m} missing or duplicated"),
            CustomError::Invalid(e) => write!(f, "generated shader rejected: {e}"),
        }
    }
}

impl std::error::Error for CustomError {}

impl From<fractadyne_core::ir::IrError> for CustomError {
    fn from(e: fractadyne_core::ir::IrError) -> Self {
        CustomError::Ir(e)
    }
}

/// Build (and validate) the shader module for `formula` with its parameter values baked in.
pub fn build(formula: &Formula, params: &[(f64, f64)]) -> Result<CustomShader, CustomError> {
    let need = formula.param_count();
    if params.len() < need {
        return Err(fractadyne_core::ir::IrError::MissingParam {
            index: (need - 1) as u16,
            supplied: params.len(),
        }
        .into());
    }
    let power = formula.escape_degree().filter(|d| *d > 1.0 && d.is_finite()).unwrap_or(2.0) as f32;
    let (mut step, precision) = step_source(formula, params)?;
    let perturbation = fractadyne_core::ir::perturb::perturbed_formula(formula);
    match &perturbation {
        Ok(p) => {
            step.push_str(&pstep_source(p, params)?);
            step.push_str(&fstep_source(p, params)?);
        }
        // Never dispatched (the app renders a non-perturbable formula directly), but the
        // perturbation paths call them, so they must exist.
        Err(_) => step.push_str(
            "fn custom_pstep(z: Cdf, dz: Cdf, dc: Cdf, iter: u32) -> Cdf { return dz; }
fn custom_fstep(z: Cdf, dz: Fe, dc: Fe, iter: u32) -> Fe { return dz; }
",
        ),
    }
    step.push_str(&format!("const CUSTOM_PHASES: u32 = {}u;\n", formula.phases().len()));
    let source = splice(&step, power)?;
    validate(&source)?;
    Ok(CustomShader {
        key: fnv1a(source.as_bytes()),
        source,
        power,
        precision,
        cost_factor: cost_factor(formula),
        perturbation: perturbation.map(|_| ()),
    })
}

/// `custom_step(…, iter)`-style dispatch over the phases: `name{k}(args)` for phase `iter mod n`.
fn phase_dispatch(out: &mut String, name: &str, args: &str, n: usize) {
    if n == 1 {
        out.push_str(&format!("    return {name}0({args});\n"));
    } else {
        out.push_str(&format!("    switch (iter % {n}u) {{\n"));
        for k in 0..n - 1 {
            out.push_str(&format!("        case {k}u: {{ return {name}{k}({args}); }}\n"));
        }
        out.push_str(&format!("        default: {{ return {name}{}({args}); }}\n", n - 1));
        out.push_str("    }\n");
    }
}

/// The generated functions: one per phase, and `custom_step` choosing by iteration.
fn step_source(formula: &Formula, params: &[(f64, f64)]) -> Result<(String, Precision), CustomError> {
    let mut out = String::new();
    let mut precision = Precision::Df32;
    for (k, prog) in formula.phases().iter().enumerate() {
        out.push_str(&format!("fn custom_phase{k}(z: Cdf, c: Cdf, zp: Cdf) -> Cdf {{\n"));
        if phase_body(prog, params, &mut out)? == Precision::F32 {
            precision = Precision::F32;
        }
        out.push_str("}\n");
    }
    out.push_str("fn custom_step(z: Cdf, c: Cdf, zp: Cdf, iter: u32) -> Cdf {\n");
    phase_dispatch(&mut out, "custom_phase", "z, c, zp", formula.phases().len());
    out.push_str("}\n");
    out.push_str(F32_HELPERS);
    out.push_str(FE_HELPERS);
    Ok((out, precision))
}

/// The perturbed step: `custom_pstep(Z, δz, δc, iter)` → δz', over `custom_pphase{k}(Z, C, δz, δc)`
/// with `C` the REFERENCE's c — the Julia constant, or the view centre less the reference offset
/// (which is how the perturbation paths define δc: pixel − reference).
fn pstep_source(perturbed: &Formula, params: &[(f64, f64)]) -> Result<String, CustomError> {
    let mut out = String::new();
    for (k, prog) in perturbed.phases().iter().enumerate() {
        out.push_str(&format!("fn custom_pphase{k}(z: Cdf, c: Cdf, dz: Cdf, dc: Cdf) -> Cdf {{\n"));
        phase_body(prog, params, &mut out)?;
        out.push_str("}\n");
    }
    out.push_str(
        "fn custom_cref() -> Cdf {
    if (iu.julia == 1u) {
        return cset(vec2<f32>(iu.julia_c.x, iu.julia_c.z), vec2<f32>(iu.julia_c.y, iu.julia_c.w));
    }
    let dsc = exp2(f32(iu.delta_exp));
    return cset(
        df_sub(vec2<f32>(iu.center.x, iu.center.z), df_mul_f32(vec2<f32>(iu.ref_offset.x, iu.ref_offset.z), dsc)),
        df_sub(vec2<f32>(iu.center.y, iu.center.w), df_mul_f32(vec2<f32>(iu.ref_offset.y, iu.ref_offset.w), dsc)),
    );
}
fn custom_pstep(z: Cdf, dz: Cdf, dc: Cdf, iter: u32) -> Cdf {
    let c = custom_cref();
",
    );
    phase_dispatch(&mut out, "custom_pphase", "z, c, dz, dc", perturbed.phases().len());
    out.push_str("}\n");
    Ok(out)
}

/// One `let` per instruction.
fn phase_body(prog: &Program, params: &[(f64, f64)], out: &mut String) -> Result<Precision, CustomError> {
    let mut precision = Precision::Df32;
    let v = |i: Val| format!("v{}", i.index());
    for (i, op) in prog.insts().iter().enumerate() {
        let (expr, f32_tier) = cdf_expr(i, op, params, &v, out)?;
        if f32_tier {
            precision = Precision::F32;
        }
        out.push_str(&format!("    let v{i} = {expr};\n"));
    }
    out.push_str(&format!("    return v{};\n", prog.out().index()));
    Ok(precision)
}

/// The df32 expression for instruction `i`, with operands named by `v`, and whether it runs at the
/// `f32` tier. A power chain's temporaries go to `out` first.
fn cdf_expr(
    i: usize,
    op: &Op,
    params: &[(f64, f64)],
    v: &dyn Fn(Val) -> String,
    out: &mut String,
) -> Result<(String, bool), CustomError> {
    let mut f32_tier = false;
    let expr = match *op {
        Op::Z => "z".to_string(),
        Op::C => "c".to_string(),
        Op::ZPrev => "zp".to_string(),
        Op::Param(p) => {
            let (re, im) = params[p as usize];
            format!("cset({}, {})", df(re)?, df(im)?)
        }
        Op::Const(re, im) => format!("cset({}, {})", df(re)?, df(im)?),
        Op::Add(a, b) => format!("c_add({}, {})", v(a), v(b)),
        Op::Sub(a, b) => format!("c_sub({}, {})", v(a), v(b)),
        Op::Mul(a, b) => format!("c_mul({}, {})", v(a), v(b)),
        Op::Div(a, b) => format!("c_div({}, {})", v(a), v(b)),
        Op::Sqr(a) => format!("c_sqr({})", v(a)),
        Op::PowI(a, n) => power_chain(i, &v(a), n, "c_sqr", "c_mul", out),
        Op::Scale(a, k) => {
            if (k as f32) as f64 == k {
                format!("c_scale({}, {})", v(a), lit(k as f32))
            } else {
                let d = df(k)?;
                format!("cset(df_mul({a}.re, {d}), df_mul({a}.im, {d}))", a = v(a))
            }
        }
        Op::Neg(a) => format!("cset(-{a}.re, -{a}.im)", a = v(a)),
        Op::Conj(a) => format!("c_conj({})", v(a)),
        Op::AbsRe(a) => format!("cset(df_abs({a}.re), {a}.im)", a = v(a)),
        Op::AbsIm(a) => format!("cset({a}.re, df_abs({a}.im))", a = v(a)),
        Op::Re(a) => format!("cset({}.re, vec2<f32>(0.0, 0.0))", v(a)),
        Op::Im(a) => format!("cset({}.im, vec2<f32>(0.0, 0.0))", v(a)),
        Op::Norm(a) => format!(
            "cset(df_add(df_mul({a}.re, {a}.re), df_mul({a}.im, {a}.im)), vec2<f32>(0.0, 0.0))",
            a = v(a)
        ),
        // Perturbed programs (`ir::perturb`): δz, δc, and the abs folds' diffabs.
        Op::Delta => "dz".to_string(),
        Op::DeltaC => "dc".to_string(),
        Op::DiffAbsRe(b, p) => format!("cset(df_diffabs({b}.re, {p}.re), {p}.im)", b = v(b), p = v(p)),
        Op::DiffAbsIm(b, p) => format!("cset({p}.re, df_diffabs({b}.im, {p}.im))", b = v(b), p = v(p)),
        Op::Pow(a, b) => {
            f32_tier = true;
            format!("cf_pow({}, {})", v(a), v(b))
        }
        Op::DiffTanh(b, p) => {
            f32_tier = true;
            format!("cf_tanh_diff({}, {})", v(b), v(p))
        }
        Op::DiffTan(b, p) => {
            f32_tier = true;
            format!("cf_tan_diff({}, {})", v(b), v(p))
        }
        Op::Func(f, a) => {
            f32_tier = true;
            format!("cf_{}({})", func_name(f), v(a))
        }
    };
    Ok((expr, f32_tier))
}

fn func_name(f: Func) -> &'static str {
    match f {
        Func::Exp => "exp",
        Func::Log => "log",
        Func::Sqrt => "sqrt",
        Func::Sin => "sin",
        Func::Cos => "cos",
        Func::Tan => "tan",
        Func::Sinh => "sinh",
        Func::Cosh => "cosh",
        Func::Tanh => "tanh",
        Func::SinSmall => "sin_small",
        Func::SinhSmall => "sinh_small",
        Func::Expm1 => "expm1",
    }
}

/// The CPU interpreter's integer-power chain: square, and multiply by the base on each set bit
/// below the top one. Its temporaries go to `out`; the last one's name is returned.
fn power_chain(i: usize, base: &str, n: u32, sqr: &str, mul: &str, out: &mut String) -> String {
    let mut r = base.to_string();
    for (j, bit) in (0..31 - n.leading_zeros()).rev().enumerate() {
        let t = format!("v{i}_{j}");
        let sq = format!("{sqr}({r})");
        let e = if (n >> bit) & 1 == 1 { format!("{mul}({sq}, {base})") } else { sq };
        out.push_str(&format!("    let {t} = {e};\n"));
        r = t;
    }
    r
}

/// How the floatexp step carries a value of a perturbed program (`fphase_body`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    /// Reference-side (no δ in it): df32, as the df32 step computes it.
    Ref,
    /// A reference value plus a perturbation (`W = B + P`, `B + P/2`): full size, so df32 holds
    /// it — the perturbation's own digits below df32's reach are negligible beside `B`.
    Full,
    /// A perturbation: floatexp, which never underflows.
    Small,
}

/// The floatexp perturbed step: `custom_fstep(Z, δz, δc, iter)` → δz' with δz, δc and every
/// perturbation in between in floatexp (`Fe`), so nothing underflows past f32's exponent floor
/// (1e-38) — which is where the df32 step stops, at ~1e36×. Reference-side and full-size values
/// stay df32. Same phases and `C` as [`pstep_source`].
fn fstep_source(perturbed: &Formula, params: &[(f64, f64)]) -> Result<String, CustomError> {
    let mut out = String::new();
    for (k, prog) in perturbed.phases().iter().enumerate() {
        out.push_str(&format!("fn custom_fphase{k}(z: Cdf, c: Cdf, dz: Fe, dc: Fe) -> Fe {{\n"));
        fphase_body(prog, params, &mut out)?;
        out.push_str("}\n");
    }
    out.push_str("fn custom_fstep(z: Cdf, dz: Fe, dc: Fe, iter: u32) -> Fe {\n    let c = custom_cref();\n");
    phase_dispatch(&mut out, "custom_fphase", "z, c, dz, dc", perturbed.phases().len());
    out.push_str("}\n");
    Ok(out)
}

/// One `let` per instruction of a perturbed program, each value in its [`Class`]'s form. The
/// rule table (`ir::perturb`) only ever combines a perturbation with a full-size value by `·`,
/// `/`, a fold or a difference function, and adds one to a reference value to make a `Full` —
/// all of which have floatexp forms. Anything else still gets code: a perturbation where df32 is
/// needed is collapsed to df32 (exact while it is inside f32's range, as the df32 step has it).
fn fphase_body(prog: &Program, params: &[(f64, f64)], out: &mut String) -> Result<(), CustomError> {
    let mut class: Vec<Class> = Vec::with_capacity(prog.insts().len());
    for (i, op) in prog.insts().iter().enumerate() {
        let c = |x: Val| class[x.index()];
        let v = |x: Val| format!("v{}", x.index());
        let small = |x: Val| c(x) == Class::Small;
        // An operand as df32, collapsing a perturbation.
        let d = |x: Val| if small(x) { format!("fe_to_cdf(v{})", x.index()) } else { v(x) };
        let fe = match *op {
            Op::Delta => Some("dz".to_string()),
            Op::DeltaC => Some("dc".to_string()),
            Op::Add(a, b) if small(a) && small(b) => Some(format!("fe_add({}, {})", v(a), v(b))),
            Op::Sub(a, b) if small(a) && small(b) => Some(format!("fe_sub({}, {})", v(a), v(b))),
            Op::Mul(a, b) if small(a) && small(b) => Some(format!("fe_mul({}, {})", v(a), v(b))),
            Op::Mul(a, b) if small(a) => Some(format!("fe_mul_cdf({}, {})", v(a), v(b))),
            Op::Mul(a, b) if small(b) => Some(format!("fe_mul_cdf({}, {})", v(b), v(a))),
            Op::Div(a, b) if small(a) && small(b) => Some(format!("fe_div({}, {})", v(a), v(b))),
            Op::Div(a, b) if small(a) => Some(format!("fe_div_cdf({}, {})", v(a), v(b))),
            Op::Sqr(a) if small(a) => Some(format!("fe_sqr({})", v(a))),
            Op::PowI(a, n) if small(a) => Some(power_chain(i, &v(a), n, "fe_sqr", "fe_mul", out)),
            Op::Scale(a, k) if small(a) => Some(if (k as f32) as f64 == k {
                format!("fe_scale({}, {})", v(a), lit(k as f32))
            } else {
                let dk = df(k)?;
                format!("fe_norm(cset(df_mul({a}.m.re, {dk}), df_mul({a}.m.im, {dk})), {a}.e)", a = v(a))
            }),
            Op::Neg(a) if small(a) => Some(format!("fe_neg({})", v(a))),
            Op::Conj(a) if small(a) => Some(format!("fe_conj({})", v(a))),
            Op::Re(a) if small(a) => Some(format!("fe_re({})", v(a))),
            Op::Im(a) if small(a) => Some(format!("fe_im({})", v(a))),
            Op::Norm(a) if small(a) => Some(format!("fe_re(fe_mul({a}, fe_conj({a})))", a = v(a))),
            Op::DiffAbsRe(b, p) if small(p) => Some(format!(
                "fe_from_sf(sf_diffabs(sf_from_df({b}.re), sf_re({p})), sf_im({p}))",
                b = d(b),
                p = v(p)
            )),
            Op::DiffAbsIm(b, p) if small(p) => Some(format!(
                "fe_from_sf(sf_re({p}), sf_diffabs(sf_from_df({b}.im), sf_im({p})))",
                b = d(b),
                p = v(p)
            )),
            Op::DiffTanh(b, p) if small(p) => Some(format!("fe_tanh_diff({}, {})", d(b), v(p))),
            Op::DiffTan(b, p) if small(p) => Some(format!("fe_tan_diff({}, {})", d(b), v(p))),
            Op::Func(f @ (Func::SinSmall | Func::SinhSmall | Func::Expm1), a) if small(a) => {
                Some(format!("fe_{}({})", func_name(f), v(a)))
            }
            _ => None,
        };
        let (expr, cl) = match fe {
            Some(e) => (e, Class::Small),
            None => {
                if matches!(op, Op::ZPrev) {
                    return Err(CustomError::Invalid("the previous iterate in a perturbed step".into()));
                }
                let all_ref = op.operands().all(|x| c(x) == Class::Ref);
                let (e, _) = cdf_expr(i, op, params, &d, out)?;
                (e, if all_ref { Class::Ref } else { Class::Full })
            }
        };
        out.push_str(&format!("    let v{i} = {expr};\n"));
        class.push(cl);
    }
    let o = prog.out().index();
    if class[o] == Class::Small {
        out.push_str(&format!("    return v{o};\n"));
    } else {
        out.push_str(&format!("    return fe_from_cdf(v{o});\n"));
    }
    Ok(())
}

/// `f32` complex elementary functions over the high words (the `Precision::F32` tier). Same
/// formulas and branch choices as the CPU interpreter's `f64` ones.
const F32_HELPERS: &str = "\
fn cf_make(x: f32, y: f32) -> Cdf { return cset(vec2<f32>(x, 0.0), vec2<f32>(y, 0.0)); }
fn cf_mul(a: Cdf, b: Cdf) -> Cdf {
    return cf_make(a.re.x * b.re.x - a.im.x * b.im.x, a.re.x * b.im.x + a.im.x * b.re.x);
}
fn cf_div(a: Cdf, b: Cdf) -> Cdf {
    let d = b.re.x * b.re.x + b.im.x * b.im.x;
    return cf_make((a.re.x * b.re.x + a.im.x * b.im.x) / d, (a.im.x * b.re.x - a.re.x * b.im.x) / d);
}
fn cf_exp(a: Cdf) -> Cdf { let r = exp(a.re.x); return cf_make(r * cos(a.im.x), r * sin(a.im.x)); }
fn cf_log(a: Cdf) -> Cdf {
    return cf_make(log(length(vec2<f32>(a.re.x, a.im.x))), atan2(a.im.x, a.re.x));
}
fn cf_sqrt(a: Cdf) -> Cdf {
    let x = a.re.x;
    let y = a.im.x;
    if (x == 0.0 && y == 0.0) { return cf_make(0.0, y); }
    let t = sqrt((abs(x) + length(vec2<f32>(x, y))) * 0.5);
    if (x >= 0.0) { return cf_make(t, y / (2.0 * t)); }
    return cf_make(abs(y) / (2.0 * t), select(t, -t, y < 0.0));
}
fn cf_sin(a: Cdf) -> Cdf { return cf_make(sin(a.re.x) * cosh(a.im.x), cos(a.re.x) * sinh(a.im.x)); }
fn cf_cos(a: Cdf) -> Cdf { return cf_make(cos(a.re.x) * cosh(a.im.x), -(sin(a.re.x) * sinh(a.im.x))); }
// tan and tanh as sin/cos (sinh/cosh) overflow f32 once the imaginary (real) part passes ~44 —
// inf/inf = NaN where the value is ±i (±1). The double-angle forms divided through by cosh 2y
// (cosh 2x) tend cleanly to the limit instead.
// The real tanh is clamped past 20, where it is ±1 in f32: a driver may compute it through exp,
// which overflows the same way.
fn rf_tanh(x: f32) -> f32 {
    if (abs(x) > 20.0) { return sign(x); }
    return tanh(x);
}
fn cf_tan(a: Cdf) -> Cdf {
    let ch = cosh(2.0 * a.im.x);
    let d = cos(2.0 * a.re.x) / ch + 1.0;
    return cf_make(sin(2.0 * a.re.x) / ch / d, rf_tanh(2.0 * a.im.x) / d);
}
fn cf_sinh(a: Cdf) -> Cdf { return cf_make(sinh(a.re.x) * cos(a.im.x), cosh(a.re.x) * sin(a.im.x)); }
fn cf_cosh(a: Cdf) -> Cdf { return cf_make(cosh(a.re.x) * cos(a.im.x), sinh(a.re.x) * sin(a.im.x)); }
fn cf_tanh(a: Cdf) -> Cdf {
    let ch = cosh(2.0 * a.re.x);
    let d = cos(2.0 * a.im.x) / ch + 1.0;
    return cf_make(rf_tanh(2.0 * a.re.x) / d, sin(2.0 * a.im.x) / ch / d);
}
fn cf_pow(a: Cdf, b: Cdf) -> Cdf {
    if (a.re.x == 0.0 && a.im.x == 0.0) { return cf_make(0.0, 0.0); }
    return cf_exp(cf_mul(b, cf_log(a)));
}
// Perturbed steps' small-argument forms (`ir::perturb`): accurate RELATIVE to a small argument,
// where the GPU's own sin, sinh and exp − 1 are accurate only in absolute terms (~5e-7 — all of a
// 1e-10 offset). Taylor series through the 9th power below 0.5 (truncation under 1e-9 relative),
// the built-ins above it.
fn rf_sin_small(x: f32) -> f32 {
    if (abs(x) >= 0.5) { return sin(x); }
    let q = x * x;
    return x * (1.0 - q / 6.0 * (1.0 - q / 20.0 * (1.0 - q / 42.0 * (1.0 - q / 72.0))));
}
fn rf_sinh_small(x: f32) -> f32 {
    if (abs(x) >= 0.5) { return sinh(x); }
    let q = x * x;
    return x * (1.0 + q / 6.0 * (1.0 + q / 20.0 * (1.0 + q / 42.0 * (1.0 + q / 72.0))));
}
fn rf_expm1(x: f32) -> f32 {
    if (abs(x) >= 0.5) { return exp(x) - 1.0; }
    return x * (1.0 + x / 2.0 * (1.0 + x / 3.0 * (1.0 + x / 4.0 * (1.0 + x / 5.0
        * (1.0 + x / 6.0 * (1.0 + x / 7.0 * (1.0 + x / 8.0 * (1.0 + x / 9.0))))))));
}
fn cf_sin_small(a: Cdf) -> Cdf {
    return cf_make(rf_sin_small(a.re.x) * cosh(a.im.x), cos(a.re.x) * rf_sinh_small(a.im.x));
}
fn cf_sinh_small(a: Cdf) -> Cdf {
    return cf_make(rf_sinh_small(a.re.x) * cos(a.im.x), cosh(a.re.x) * rf_sin_small(a.im.x));
}
// e^x·cos y − 1 = expm1(x)·cos y − 2·sin²(y/2): no difference of near-equal terms.
fn cf_expm1(a: Cdf) -> Cdf {
    let s = rf_sin_small(0.5 * a.im.x);
    return cf_make(rf_expm1(a.re.x) * cos(a.im.x) - 2.0 * s * s, exp(a.re.x) * rf_sin_small(a.im.x));
}
// The perturbed tanh and tan (`ir::tanh_diff`, branch for branch): tanh(b + p) − tanh(b) as
// sinh(p)·sech(b)·sech(b + p) below |Re p| = 40 (TANH_DIFF_SPLIT), the plain difference above.
// sech a = 2·e^−s / (1 + e^−2s) with s = ±a, Re s ≥ 0: e^−s is at most 1 in size, so nothing
// overflows. tan a = −i·tanh(i·a).
fn cf_sech(a: Cdf) -> Cdf {
    let sg = select(1.0, -1.0, a.re.x < 0.0);
    let r = exp(-sg * a.re.x);
    let y = sg * a.im.x;
    let e = cf_make(r * cos(y), -r * sin(y));
    let e2 = cf_mul(e, e);
    return cf_div(cf_make(2.0 * e.re.x, 2.0 * e.im.x), cf_make(1.0 + e2.re.x, e2.im.x));
}
fn cf_tanh_diff(b: Cdf, p: Cdf) -> Cdf {
    let w = cf_make(b.re.x + p.re.x, b.im.x + p.im.x);
    if (abs(p.re.x) >= 40.0) {
        let t = cf_tanh(w);
        let u = cf_tanh(b);
        return cf_make(t.re.x - u.re.x, t.im.x - u.im.x);
    }
    return cf_mul(cf_mul(cf_sinh_small(p), cf_sech(b)), cf_sech(w));
}
fn cf_tan_diff(b: Cdf, p: Cdf) -> Cdf {
    let d = cf_tanh_diff(cf_make(-b.im.x, b.re.x), cf_make(-p.im.x, p.re.x));
    return cf_make(d.im.x, -d.re.x);
}
";

/// The floatexp forms the generated floatexp step (`custom_fstep`) needs beyond the fixed
/// module's `fe_*` set.
const FE_HELPERS: &str = "\
fn fe_div_cdf(a: Fe, z: Cdf) -> Fe { return fe_norm(c_div(a.m, z), a.e); }
fn fe_div(a: Fe, b: Fe) -> Fe { return fe_norm(c_div(a.m, b.m), a.e - b.e); }
fn fe_re(a: Fe) -> Fe { return fe_norm(cset(a.m.re, vec2<f32>(0.0, 0.0)), a.e); }
fn fe_im(a: Fe) -> Fe { return fe_norm(cset(a.m.im, vec2<f32>(0.0, 0.0)), a.e); }
// The small-argument functions of a floatexp perturbation. Below 2^-60 the first series term IS
// the value (the next is under 2^-60 relative, past df32's 2^-48); from 2^-60 up the df32 helper on
// the collapsed value, where the argument and its square are still normal f32.
fn fe_sin_small(a: Fe) -> Fe {
    if (a.e < -60) { return a; }
    return fe_from_cdf(cf_sin_small(fe_to_cdf(a)));
}
fn fe_sinh_small(a: Fe) -> Fe {
    if (a.e < -60) { return a; }
    return fe_from_cdf(cf_sinh_small(fe_to_cdf(a)));
}
fn fe_expm1(a: Fe) -> Fe {
    if (a.e < -60) { return a; }
    return fe_from_cdf(cf_expm1(fe_to_cdf(a)));
}
// tanh(b + p) − tanh(b) for a floatexp p: cf_tanh_diff from 2^-60 up; below, sinh p = p and
// b + p = b in df32, so p·sech²b.
fn fe_tanh_diff(b: Cdf, p: Fe) -> Fe {
    if (p.e >= -60) { return fe_from_cdf(cf_tanh_diff(b, fe_to_cdf(p))); }
    let s = cf_sech(b);
    return fe_mul_cdf(p, cf_mul(s, s));
}
// tan a = −i·tanh(i·a), as cf_tan_diff.
fn fe_tan_diff(b: Cdf, p: Fe) -> Fe {
    let d = fe_tanh_diff(cf_make(-b.im.x, b.re.x), fe_make(cset(-p.m.im, p.m.re), p.e));
    return fe_make(cset(d.m.im, -d.m.re), d.e);
}
";

/// A WGSL float literal: Rust's `Debug` form is the shortest that round-trips and always has a
/// point or an exponent (`1.0`, `1e-7`), which WGSL reads as a float.
fn lit(v: f32) -> String {
    format!("{v:?}")
}

/// An `f64` as a df32 literal `vec2<f32>(hi, lo)` — the split `pack_sample` uses.
fn df(v: f64) -> Result<String, CustomError> {
    let hi = v as f32;
    if !hi.is_finite() {
        return Err(CustomError::OutOfRange(v));
    }
    let lo = (v - hi as f64) as f32;
    Ok(format!("vec2<f32>({}, {})", lit(hi), lit(lo)))
}

/// The byte range of the unique line holding `marker`, from its line start to its line end.
fn marker_line(src: &str, marker: &'static str) -> Result<(usize, usize), CustomError> {
    let mut hits = src.match_indices(marker);
    let (at, _) = hits.next().ok_or(CustomError::Marker(marker))?;
    if hits.next().is_some() {
        return Err(CustomError::Marker(marker));
    }
    let start = src[..at].rfind('\n').map_or(0, |i| i + 1);
    let end = src[at..].find('\n').map_or(src.len(), |i| at + i + 1);
    Ok((start, end))
}

/// The fixed module with every slot filled: the direct and perturbed steps replaced by the
/// generated ones, both smooth values clamped, the floatexp path cut, the rebase phase-aligned.
fn splice(step_fns: &str, power: f32) -> Result<String, CustomError> {
    // The power must stay OPAQUE to the compiler, as the fixed module's is (the uniform picks it
    // there), so `log(power_f)` in the smooth value runs on the GPU in both. Folded at compile time
    // it differs in the last bit: measured, a generated Mandelbrot then differed from the built-in
    // by 1–2 ulps of the smooth value on 1,292 pixels, and by nothing with this line. ⚠Both halves
    // are load-bearing, each found by a gate going red: the condition must be one the compiler
    // cannot decide (`iu.max_iter == 0u` was folded — the loop's own guard proves it false here),
    // and the two arms must differ (with both 2.0 for Mandelbrot, the select folds to a constant).
    let power_line = |indent: &str| {
        format!(
            "{indent}power_f = select(CUSTOM_POWER + 1.0, CUSTOM_POWER, iu.formula == {}u);\n",
            fractadyne_core::formula::CUSTOM
        )
    };
    // The smooth value `n + 1 − log(log₂|z|)/log d` goes NEGATIVE — which reads as interior — for a
    // pixel that escapes within a couple of iterations or far past the bailout: routine for a steep
    // custom formula (measured: `100·exp(c)` escaping at n = 1 gives −1.45), unreachable from the
    // built-ins' views. Clamped at 0 it stays escaped; every value ≥ 0 keeps its bits.
    let smooth = "        let smit = max(f32(iter) + 1.0 - nu, 0.0);\n".to_string();
    // `iter` has already advanced to the NEXT step's index where the rebase runs, so the reference
    // restarts at the sample whose phase that step will run.
    let rebase = |re: &str, im: &str| {
        format!(
            "            if (rebase_now || ref_n + 1u >= iu.orbit_len) {{
                n_rebase = n_rebase + 1u;
                let base = iter % CUSTOM_PHASES;
                let r0 = orbit_cdf(reference[base]);
                dz = cset(df_sub({re}, r0.re), df_sub({im}, r0.im));
                ref_n = base;
            }}
"
        )
    };
    let fills: [String; 13] = [
        format!(
            "                var zn: Cdf = custom_tame(custom_step(z, c, zprev, iter));\n{}",
            power_line("                ")
        ),
        smooth.clone(),
        fe_branch(),
        // Tamed as the direct step is: a function's escape is a JUMP (sin z from |z| ≈ 6 past f32's
        // range in one step, cosh overflowing), and an infinite or NaN δ never passes the escape
        // test — measured, 2,406 of 48,400 pixels at 1e6× read interior or 0 with a 60-iteration
        // budget. Below `TAME` it returns δ bit for bit, so the ring formulas are unchanged.
        format!("            dz = custom_tame(custom_pstep(z, dz, dc, iter));\n{}", power_line("            ")),
        rebase("zr_full", "zi_full"),
        smooth,
        // The chunk pass's direct step: a custom formula has no derivative (fs_iterate computes
        // none for it), so the state slot the built-ins carry it in carries z_{n-1} instead,
        // which starts at 0 — as `zprev` does in fs_iterate — whatever the derivative's start.
        format!(
            "            var zp = dz;
            if (iter == 0u) {{ zp = cset(zero, zero); }}
            let zn = custom_tame(custom_step(z, c, zp, iter));
            dz = z;
            z = zn;
{}",
            power_line("            ")
        ),
        "            smit_out = max(f32(iter) + 1.0 - nu, 0.0);\n".to_string(),
        format!("            dz = custom_tame(custom_pstep(z, dz, dc, iter));\n{}", power_line("            ")),
        rebase("z_full_re", "z_full_im"),
        "            smit = max(f32(iter) + 1.0 - nu, 0.0);\n".to_string(),
        chunk_fe_entry(),
        // fs_iterate's values for a formula without a derivative: no slope, no distance estimate.
        "    let nrm = vec2<f32>(0.0, 0.0);\n    let de = 1.0e30;\n".to_string(),
    ];
    let mut out = String::with_capacity(FIXED.len() + step_fns.len());
    let mut at = 0;
    for ((begin, end), fill) in SLOTS.iter().zip(&fills) {
        let (b0, _) = marker_line(FIXED, begin)?;
        let (_, e1) = marker_line(FIXED, end)?;
        if b0 < at || e1 <= b0 {
            return Err(CustomError::Marker(begin));
        }
        out.push_str(&FIXED[at..b0]);
        out.push_str(fill);
        at = e1;
    }
    out.push_str(&FIXED[at..]);
    out.push_str("\n// ---------------- generated: custom formula (custom.rs) ----------------\n");
    out.push_str(&format!("const CUSTOM_POWER: f32 = {};\n", lit(power)));
    out.push_str(&tame_source());
    out.push_str(step_fns);
    Ok(out)
}

/// One floatexp step of a custom formula, shared verbatim by the single-pass branch and the chunk
/// pass (so where a frame splits never changes a pixel): advance δz, form the full `z` in extended
/// range, then (after the caller's escape test, [`fe_rebase`]) Zhuoran's rebase, phase-aligned.
///
/// ⭐The df32 tail, as the fixed module's mode 2 has it for Mandelbrot (`TAIL_DF32_MIN`): once
/// |δz| ≥ 2^-60 a step runs the generated df32 step instead (`custom_pstep`, ~2.5× cheaper per
/// operation), converted exactly both ways. It is MEMORYLESS — decided by this step's δz alone — so
/// a chunked walk cannot depend on where a pass ended. δc below 2^-120 is dropped there
/// (`tail_dc`), under 2^-60 relative.
fn fe_advance(ind: &str) -> String {
    format!(
        "{ind}n_full = n_full + 1u;
{ind}let Z = orbit_cdf(reference[ref_n]);
{ind}if (iu.tail_on == 1u && dz.e >= TAIL_DF32_MIN_E) {{
{ind}    n_big = n_big + 1u;
{ind}    dz = fe_from_cdf(custom_tame(custom_pstep(Z, fe_to_cdf(dz), tail_dc(dc), iter)));
{ind}}} else {{
{ind}    dz = custom_tame_fe(custom_fstep(Z, dz, dc, iter));
{ind}}}
{ind}ref_n = ref_n + 1u;
{ind}iter = iter + 1u;
{ind}// z = Z_{{n+1}} + δz in EXTENDED range: its squares underflow f32 at depth (see fs_iterate).
{ind}z_full = fe_add(orbit_fe(reference[ref_n]), dz);
{ind}zf = fe_lo_f32(z_full);
"
    )
}

/// Zhuoran's rebase in scalar floatexp (as the fixed mode 2's), onto the reference sample whose
/// phase the next step runs.
fn fe_rebase(ind: &str) -> String {
    format!(
        "{ind}if (sf_lt(fe_abs_sf(z_full), fe_abs_sf(dz)) || ref_n + 1u >= iu.orbit_len) {{
{ind}    n_rebase = n_rebase + 1u;
{ind}    let base = iter % CUSTOM_PHASES;
{ind}    dz = fe_sub(z_full, orbit_fe(reference[base]));
{ind}    ref_n = base;
{ind}}}
"
    )
}

/// The opaque escape degree (see `splice`).
fn power_select() -> String {
    format!("select(CUSTOM_POWER + 1.0, CUSTOM_POWER, iu.formula == {}u)", fractadyne_core::formula::CUSTOM)
}

/// `fs_iterate`'s mode 2 for a custom formula (the `@@CUSTOM_FE` slot).
fn fe_branch() -> String {
    format!(
        "    else if (iu.mode == 2u) {{
        // A custom formula's floatexp perturbation (custom.rs): δz and δc in floatexp, the
        // generated floatexp step, the df32 tail, the phase-aligned rebase. No BLA, SA, glitch
        // detection or derivative: a custom formula has none of them.
        let pert_m = cset(
            df_add(off_re, vec2<f32>(iu.ref_offset.x, iu.ref_offset.z)),
            df_add(off_im, vec2<f32>(iu.ref_offset.y, iu.ref_offset.w)),
        );
        let pert = fe_norm(pert_m, iu.delta_exp);
        var dz = fe_zero();
        var dc = fe_zero();
        if (iu.julia == 1u) {{ dz = pert; }} else {{ dc = pert; }}
        let cmag = select(
            length(vec2<f32>(iu.center.x, iu.center.y)),
            length(vec2<f32>(iu.julia_c.x, iu.julia_c.y)),
            iu.julia == 1u,
        );
        let r0i = reference[0];
        var aux = aux_init(vec2<f32>(r0i.x, r0i.y));
        var ref_n: u32 = 0u;
        let power_f = {power};
        var z_full = fe_zero();
        loop {{
            if (iter >= iu.max_iter) {{ break; }}
{advance}            if ((iu.aux_on & 1u) == 1u) {{ aux_step(&aux, zf, cmag, power_f); }}
            if (dot(zf, zf) > bail2) {{ escaped = true; break; }}
{rebase}        }}
        ctr_commit(n_rebase, n_ext, n_bla);
        step_commit(gx, gy, n_full, n_full, n_big, iter);
        if (!escaped) {{
            atomicAdd(&counters[CTR_MAXITER], 1u);
            let aux_out = select(AUX_NONE, aux_pack(aux, 0.0, zf), (iu.aux_on & 1u) == 1u);
            return FragOut(vec4<f32>(-1.0, 0.0, 0.0, 1.0e30), aux_out);
        }}
        let mag2 = dot(zf, zf);
        let nu = log(log(mag2) * 0.5 / log(2.0)) / log(power_f);
        let smit = max(f32(iter) + 1.0 - nu, 0.0);
        let aux_out = select(AUX_NONE, aux_pack(aux, fract(smit), zf), (iu.aux_on & 1u) == 1u);
        esc_range_commit(smit);
        esc_count_commit(vec2<i32>(i32(gx), i32(gy)));
        return FragOut(vec4<f32>(smit, 0.0, 0.0, 1.0e30), aux_out);
    }}
",
        power = power_select(),
        advance = fe_advance("            "),
        rebase = fe_rebase("            "),
    )
}

/// The resumable floatexp chunk pass for a custom formula (the `@@CUSTOM_CHUNK_FE` slot): the fixed
/// entry point's prologue, state layout and epilogue, around the same step as [`fe_branch`]. The
/// derivative channels carry zeros (a custom formula has no derivative; its resolve reads none).
fn chunk_fe_entry() -> String {
    format!(
        "@fragment
fn fs_iterate_chunk_fe(in: VsOut) -> ChunkOut4 {{
    let step_re = iu.step.xy;
    let step_im = iu.step.zw;
    let gx = iu.px_offset.x + in.pos.x;
    let gy = iu.px_offset.y + in.pos.y;
    let coord_re = gx - iu.res.x * 0.5;
    let coord_im = iu.res.y * 0.5 - gy;
    let off_re = df_mul_f32(step_re, coord_re);
    let off_im = df_mul_f32(step_im, coord_im);
    let bail2 = 256.0 * 256.0;
    let stop = min(iu.end_iter, iu.max_iter);
    let p = vec2<i32>(i32(in.pos.x), i32(in.pos.y));
    var sz = vec4<f32>(0.0);
    var sm = vec4<f32>(0.0);
    if (iu.start_iter > 0u) {{
        sz = textureLoad(st_z, p, 0);
        sm = textureLoad(st_meta, p, 0);
        if (info_status(sm) != ST_RUNNING) {{
            return ChunkOut4(sz, textureLoad(st_dz, p, 0), sm, textureLoad(st_exp, p, 0));
        }}
    }}
    let pert_m = cset(
        df_add(off_re, vec2<f32>(iu.ref_offset.x, iu.ref_offset.z)),
        df_add(off_im, vec2<f32>(iu.ref_offset.y, iu.ref_offset.w)),
    );
    let pert = fe_norm(pert_m, iu.delta_exp);
    var dz = fe_zero();
    var dc = fe_zero();
    if (iu.julia == 1u) {{ dz = pert; }} else {{ dc = pert; }}
    var iter: u32 = 0u;
    var ref_n: u32 = 0u;
    if (iu.start_iter > 0u) {{
        // `fe_make`, not `fe_norm`: the stored pair is already normalized (see the fixed pass).
        dz = fe_make(cset(vec2<f32>(sz.x, sz.y), vec2<f32>(sz.z, sz.w)), i32(sm.w));
        iter = info_iter(sm);
        ref_n = u32(sm.z);
    }}
    let power_f = {power};
    var zf = vec2<f32>(0.0, 0.0);
    var z_full = fe_zero();
    var escaped = false;
    let iter0 = iter;
    var n_full: u32 = 0u;
    var n_big: u32 = 0u;
    var n_rebase: u32 = 0u;
    loop {{
        if (iter >= stop) {{ break; }}
        if (iu.step_cap > 0u && n_full >= iu.step_cap) {{ break; }}
{advance}        if (dot(zf, zf) > bail2) {{ escaped = true; break; }}
{rebase}    }}
    ctr_commit(n_rebase, 0u, 0u);
    step_commit(gx, gy, n_full, n_full, n_big, iter - iter0);
    if (escaped) {{
        let mag2 = dot(zf, zf);
        let nu = log(log(mag2) * 0.5 / log(2.0)) / log(power_f);
        let smit = max(f32(iter) + 1.0 - nu, 0.0);
        esc_range_commit(smit);
        // The full z as df32 for the mode-agnostic resolve (|z| > 256: fe_to_cdf is exact).
        let zc = fe_to_cdf(z_full);
        return ChunkOut4(
            vec4<f32>(zc.re.x, zc.re.y, zc.im.x, zc.im.y),
            vec4<f32>(0.0),
            info_pack(iter, ST_ESCAPED, smit, 0.0),
            vec4<f32>(0.0),
        );
    }}
    var status: f32 = ST_RUNNING;
    if (iter >= iu.max_iter) {{ status = ST_INTERIOR; }}
    if (status == ST_RUNNING && iu.step_cap > 0u) {{
        atomicAdd(&counters[CTR_CHUNK_RUNNING], 1u);
    }}
    return ChunkOut4(
        vec4<f32>(dz.m.re.x, dz.m.re.y, dz.m.im.x, dz.m.im.y),
        vec4<f32>(0.0),
        info_pack(iter, status, f32(ref_n), f32(dz.e)),
        vec4<f32>(0.0),
    );
}}
",
        power = power_select(),
        advance = fe_advance("        "),
        rebase = fe_rebase("        "),
    )
}

/// The largest component magnitude a step may leave: past it (or non-finite) a component is
/// replaced by `±TAME`. [`tame_f64`] is the CPU mirror.
pub const TAME: f32 = 1.0e15;

/// Trim a custom formula's reference orbit to samples the perturbed step can hold, returning the
/// new length. An escaping step of an explosive formula (`sin z` once |Im z| passes ~89) leaves a
/// final sample beyond f32's range — inf or NaN in the packed sample — and every pixel that
/// reaches it carries that into its value and never escapes (measured: all 494 escaping samples of
/// `sin z + c` at 1e12× read interior). Trailing samples past [`TAME`] or not finite are dropped;
/// a pixel reaching the new end rebases and takes that last step from `Z₀` in full, tamed exactly
/// as the direct step is. The built-ins never reach this (their last step from |Z| ≤ 1e6 stays in
/// range), and a custom orbit is never extended or cached, so nothing else reads the dropped tail.
pub fn trim_reference(orbit: &mut Vec<[f32; 4]>) -> u32 {
    let fits = |s: &[f32; 4]| {
        let (x, y) = fractadyne_core::sample_xy(s);
        x.is_finite() && y.is_finite() && x.abs() <= TAME as f64 && y.abs() <= TAME as f64
    };
    while orbit.len() > 1 && !fits(orbit.last().expect("non-empty")) {
        orbit.pop();
    }
    orbit.len() as u32
}

/// `custom_tame`: an escaping step of a steep formula (`exp`, `sin`, a high power) can overflow
/// f32 outright, and the smooth value of an infinite `z` is −∞, which reads as INTERIOR (measured:
/// `sin z + c` escapes to |z|² ≈ 1e50–1e158 in one step). Any component past [`TAME`], infinite or
/// NaN is replaced by `±TAME` (NaN, which has no sign, by 0; both NaN by `TAME + 0i`), so the pixel
/// escapes with a finite value. Polynomials up to degree 6 never reach it from the bailout (256⁶ ≈
/// 2.8e14), so no built-in step is affected. The tests are on the BITS: a compiler may assume floats
/// are finite and fold a NaN comparison.
fn tame_source() -> String {
    let t = TAME.to_bits();
    format!(
        "fn custom_tame(v: Cdf) -> Cdf {{
    let ax = bitcast<u32>(v.re.x) & 0x7fffffffu;
    let ay = bitcast<u32>(v.im.x) & 0x7fffffffu;
    if (ax < {t:#x}u && ay < {t:#x}u) {{ return v; }}
    var x = select(0.0, clamp(v.re.x, -{m}, {m}), ax <= 0x7f800000u);
    let y = select(0.0, clamp(v.im.x, -{m}, {m}), ay <= 0x7f800000u);
    if (ax > 0x7f800000u && ay > 0x7f800000u) {{ x = {m}; }}
    return cset(vec2<f32>(x, 0.0), vec2<f32>(y, 0.0));
}}
// The floatexp step's tame: a value under 2^41 with a finite mantissa is returned bit for bit;
// anything larger goes through `custom_tame` as df32 (exact up to 2^127, and past that already
// beyond TAME), so mode 2 escapes with the value mode 0 would.
fn custom_tame_fe(v: Fe) -> Fe {{
    let ax = bitcast<u32>(v.m.re.x) & 0x7fffffffu;
    let ay = bitcast<u32>(v.m.im.x) & 0x7fffffffu;
    if (v.e < 40 && ax < 0x7f800000u && ay < 0x7f800000u) {{ return v; }}
    return fe_from_cdf(custom_tame(fe_to_cdf(v)));
}}
",
        m = lit(TAME)
    )
}

/// `custom_tame` in `f64`, applied to an orbit's final point (the only one a tame can touch, since
/// a tamed value is always past the bailout).
pub fn tame_f64(z: (f64, f64)) -> (f64, f64) {
    let t = TAME as f64;
    let big = |v: f64| !(v.abs() < t);
    if !big(z.0) && !big(z.1) {
        return z;
    }
    let part = |v: f64| if v.is_nan() { 0.0 } else { v.clamp(-t, t) };
    let x = if z.0.is_nan() && z.1.is_nan() { t } else { part(z.0) };
    (x, part(z.1))
}

/// Parse and validate with naga, so a generator bug is an error here rather than a driver-side
/// validation panic.
fn validate(source: &str) -> Result<(), CustomError> {
    use egui_wgpu::wgpu::naga;
    let module = naga::front::wgsl::parse_str(source).map_err(|e| CustomError::Invalid(e.emit_to_string(source)))?;
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(&module)
        .map_err(|e| CustomError::Invalid(format!("{e:?}")))?;
    Ok(())
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3))
}

#[cfg(test)]
mod tests;
