//! Custom formulas on the GPU (design/custom-formulas.md §4.5).
//!
//! A formula's IR ([`fractadyne_core::ir`]) becomes a WGSL `custom_step` function written in the
//! shader's own df32 helpers (`c_sqr`, `c_mul`, `df_abs`, …), spliced into the fixed module at two
//! marked slots inside `iterate_at`:
//!
//! - `@@CUSTOM_STEP` — the direct path's formula step, replaced by a call to `custom_step`;
//! - `@@CUSTOM_CUT` — the perturbation paths, removed. A custom formula renders in direct mode only
//!   until design phase 4 derives its perturbed step, and those paths are most of the iterate
//!   pipeline's compile time (§3 A6: ~4 s with every formula, ~1 s specialised).
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
const CUT_BEGIN: &str = "// @@CUSTOM_CUT_BEGIN";
const CUT_END: &str = "// @@CUSTOM_CUT_END";

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
    let (step, precision) = step_source(formula, params)?;
    let source = splice(&step, power)?;
    validate(&source)?;
    Ok(CustomShader { key: fnv1a(source.as_bytes()), source, power, precision })
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
    let n = formula.phases().len();
    out.push_str("fn custom_step(z: Cdf, c: Cdf, zp: Cdf, iter: u32) -> Cdf {\n");
    if n == 1 {
        out.push_str("    return custom_phase0(z, c, zp);\n");
    } else {
        out.push_str(&format!("    switch (iter % {n}u) {{\n"));
        for k in 0..n - 1 {
            out.push_str(&format!("        case {k}u: {{ return custom_phase{k}(z, c, zp); }}\n"));
        }
        out.push_str(&format!("        default: {{ return custom_phase{}(z, c, zp); }}\n", n - 1));
        out.push_str("    }\n");
    }
    out.push_str("}\n");
    out.push_str(F32_HELPERS);
    Ok((out, precision))
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

/// The fixed module with the step slot replaced by `custom_step` and the perturbation paths cut.
fn splice(step_fns: &str, power: f32) -> Result<String, CustomError> {
    let (s0, _) = marker_line(FIXED, STEP_BEGIN)?;
    let (_, s1) = marker_line(FIXED, STEP_END)?;
    let (c0, _) = marker_line(FIXED, CUT_BEGIN)?;
    let (_, c1) = marker_line(FIXED, CUT_END)?;
    if !(s0 < s1 && s1 <= c0 && c0 < c1) {
        return Err(CustomError::Marker(STEP_BEGIN));
    }
    let mut out = String::with_capacity(FIXED.len() + step_fns.len());
    out.push_str(&FIXED[..s0]);
    out.push_str("                var zn: Cdf = custom_step(z, c, zprev, iter);\n");
    out.push_str("                power_f = CUSTOM_POWER;\n");
    out.push_str(&FIXED[s1..c0]);
    out.push_str("    return FragOut(vec4<f32>(-1.0, 0.0, 0.0, 1.0e30), AUX_NONE);\n");
    out.push_str(&FIXED[c1..]);
    out.push_str("\n// ---------------- generated: custom formula (custom.rs) ----------------\n");
    out.push_str(&format!("const CUSTOM_POWER: f32 = {};\n", lit(power)));
    out.push_str(step_fns);
    Ok(out)
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
