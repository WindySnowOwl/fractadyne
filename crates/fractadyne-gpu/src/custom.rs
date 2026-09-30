//! Custom formulas on the GPU (design/custom-formulas.md §4.5).
//!
//! A formula's IR ([`fractadyne_core::ir`]) becomes WGSL written in the shader's own df32 helpers
//! (`c_sqr`, `c_mul`, `df_abs`, …): `custom_step` (the direct step) and, when the formula is
//! perturbable ([`fractadyne_core::ir::perturb`]), `custom_pstep` (its perturbed step, δz' from the
//! reference `Z`, `δz` and `δc`). They are spliced into the fixed module at marked slots in
//! `iterate_at`:
//!
//! - `@@CUSTOM_STEP` / `@@CUSTOM_SMOOTH` — the direct path's formula step and smooth value;
//! - `@@CUSTOM_CUT` — the floatexp perturbation path (mode 2), removed: custom formulas have none
//!   yet, and it is most of the iterate pipeline's compile time (§3 A6);
//! - `@@CUSTOM_PSTEP` / `@@CUSTOM_REBASE` / `@@CUSTOM_SMOOTH0` — the df32 perturbation path's
//!   (mode 0) formula step, rebase and smooth value. The rebase is PHASE-ALIGNED for a hybrid: the
//!   reference index restarts at `iter mod phases`, not 0, so reference and pixel stay in the same
//!   phase (identical to the fixed module's for one phase).
//!
//! The fixed module carries only the marker COMMENTS, so every built-in pipeline compiles from the
//! same code as before. A custom module renders under [`fractadyne_core::formula::CUSTOM`], an id no
//! built-in branch matches, so Newton's special case and the built-in distance estimate stay off.
//!
//! Each IR operation maps to the helper the built-ins use for it, in the same order the CPU
//! interpreter uses (`Sqr` → `c_sqr`, `Mul` → `c_mul`, `PowI` → the same square-and-multiply chain),
//! so a generated built-in step is the built-in step's own helper calls.

use fractadyne_core::ir::{Formula, Func, Op, Program};

const FIXED: &str = include_str!("mandelbrot.wgsl");
const STEP_BEGIN: &str = "// @@CUSTOM_STEP_BEGIN";
const STEP_END: &str = "// @@CUSTOM_STEP_END";
const SMOOTH_BEGIN: &str = "// @@CUSTOM_SMOOTH_BEGIN";
const SMOOTH_END: &str = "// @@CUSTOM_SMOOTH_END";
const CUT_BEGIN: &str = "// @@CUSTOM_CUT_BEGIN";
const CUT_END: &str = "// @@CUSTOM_CUT_END";
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
const RESOLVE_DE_BEGIN: &str = "// @@CUSTOM_RESOLVE_DE_BEGIN";
const RESOLVE_DE_END: &str = "// @@CUSTOM_RESOLVE_DE_END";
/// Every slot, in file order: `(begin, end)`.
const SLOTS: [(&str, &str); 12] = [
    (STEP_BEGIN, STEP_END),
    (SMOOTH_BEGIN, SMOOTH_END),
    (CUT_BEGIN, CUT_END),
    (PSTEP_BEGIN, PSTEP_END),
    (REBASE_BEGIN, REBASE_END),
    (SMOOTH0_BEGIN, SMOOTH0_END),
    (CHUNK_STEP_BEGIN, CHUNK_STEP_END),
    (CHUNK_SMOOTH_BEGIN, CHUNK_SMOOTH_END),
    (CHUNK_PSTEP_BEGIN, CHUNK_PSTEP_END),
    (CHUNK_REBASE_BEGIN, CHUNK_REBASE_END),
    (CHUNK_SMOOTH0_BEGIN, CHUNK_SMOOTH0_END),
    (RESOLVE_DE_BEGIN, RESOLVE_DE_END),
];

/// How much precision the generated step carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Precision {
    /// df32 throughout (~48 bits): the ring operations and division.
    Df32,
    /// An elementary function (or complex power) evaluates in `f32` (~24 bits): the depth limit
    /// drops accordingly. Their df32 forms are future work.
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
    /// `Ok` when the module carries a perturbed step (the df32 perturbation path, mode 0, renders
    /// it); otherwise why not — the formula then renders on the direct path only.
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
        Ok(p) => step.push_str(&pstep_source(p, params)?),
        // Never dispatched (the app renders a non-perturbable formula directly), but the mode-0
        // path calls it, so it must exist.
        Err(_) => step.push_str("fn custom_pstep(z: Cdf, dz: Cdf, dc: Cdf, iter: u32) -> Cdf { return dz; }\n"),
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
    let v = |i: fractadyne_core::ir::Val| format!("v{}", i.index());
    for (i, op) in prog.insts().iter().enumerate() {
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
            Op::PowI(a, n) => {
                // The CPU interpreter's chain: square, and multiply by the base on each set bit
                // below the top one.
                let base = v(a);
                let mut r = base.clone();
                for (j, bit) in (0..31 - n.leading_zeros()).rev().enumerate() {
                    let t = format!("v{i}_{j}");
                    let sq = format!("c_sqr({r})");
                    let e = if (n >> bit) & 1 == 1 { format!("c_mul({sq}, {base})") } else { sq };
                    out.push_str(&format!("    let {t} = {e};\n"));
                    r = t;
                }
                r
            }
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
                precision = Precision::F32;
                format!("cf_pow({}, {})", v(a), v(b))
            }
            Op::Func(f, a) => {
                precision = Precision::F32;
                let name = match f {
                    Func::Exp => "exp",
                    Func::Log => "log",
                    Func::Sqrt => "sqrt",
                    Func::Sin => "sin",
                    Func::Cos => "cos",
                    Func::Tan => "tan",
                    Func::Sinh => "sinh",
                    Func::Cosh => "cosh",
                    Func::Tanh => "tanh",
                };
                format!("cf_{name}({})", v(a))
            }
        };
        out.push_str(&format!("    let v{i} = {expr};\n"));
    }
    out.push_str(&format!("    return v{};\n", prog.out().index()));
    Ok(precision)
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
fn cf_tan(a: Cdf) -> Cdf { return cf_div(cf_sin(a), cf_cos(a)); }
fn cf_sinh(a: Cdf) -> Cdf { return cf_make(sinh(a.re.x) * cos(a.im.x), cosh(a.re.x) * sin(a.im.x)); }
fn cf_cosh(a: Cdf) -> Cdf { return cf_make(cosh(a.re.x) * cos(a.im.x), sinh(a.re.x) * sin(a.im.x)); }
fn cf_tanh(a: Cdf) -> Cdf { return cf_div(cf_sinh(a), cf_cosh(a)); }
fn cf_pow(a: Cdf, b: Cdf) -> Cdf {
    if (a.re.x == 0.0 && a.im.x == 0.0) { return cf_make(0.0, 0.0); }
    return cf_exp(cf_mul(b, cf_log(a)));
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
    let fills: [String; 12] = [
        format!(
            "                var zn: Cdf = custom_tame(custom_step(z, c, zprev, iter));\n{}",
            power_line("                ")
        ),
        smooth.clone(),
        String::new(),
        format!("            dz = custom_pstep(z, dz, dc, iter);\n{}", power_line("            ")),
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
        format!("            dz = custom_pstep(z, dz, dc, iter);\n{}", power_line("            ")),
        rebase("z_full_re", "z_full_im"),
        "            smit = max(f32(iter) + 1.0 - nu, 0.0);\n".to_string(),
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

/// The largest component magnitude a step may leave: past it (or non-finite) a component is
/// replaced by `±TAME`. [`tame_f64`] is the CPU mirror.
pub const TAME: f32 = 1.0e15;

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
