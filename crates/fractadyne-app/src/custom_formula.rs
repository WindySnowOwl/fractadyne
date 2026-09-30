//! The session's custom formula (design/custom-formulas.md): the text the user wrote, its
//! parameter values, the formula IR parsed from it and the shader module generated from that.
//! [`FractalKind::Custom`](crate::fractal::FractalKind::Custom) renders it.
//!
//! The TEXT is the formula's identity: it is what sessions and view files store, and everything
//! else is rebuilt from it. A formula that no longer parses (a file from a newer build, a hand
//! edit) is reported and the view falls back to Mandelbrot rather than rendering something else.

use fractadyne_core::ir::{self, parse::MAX_PARAMS};
use fractadyne_gpu::custom::CustomShader;
use std::sync::Arc;

pub(crate) struct CustomFormula {
    /// What the user wrote, verbatim.
    pub(crate) source: String,
    /// `p1`…`p5`, always [`MAX_PARAMS`] long (unused ones are 0).
    pub(crate) params: Vec<(f64, f64)>,
    pub(crate) formula: ir::Formula,
    pub(crate) shader: Arc<CustomShader>,
}

impl CustomFormula {
    /// Parse `source`, bake `params`, generate and validate the shader. The error is a sentence
    /// for the user (a parse error names its line and column).
    pub(crate) fn compile(source: &str, params: &[(f64, f64)]) -> Result<Self, String> {
        let formula = ir::parse::parse(source).map_err(|e| e.to_string())?;
        let mut params = params.to_vec();
        params.resize(MAX_PARAMS, (0.0, 0.0));
        let shader = fractadyne_gpu::custom::build(&formula, &params).map_err(|e| e.to_string())?;
        Ok(CustomFormula { source: source.to_string(), params, formula, shader: Arc::new(shader) })
    }

    /// How many of `p1`…`p5` the formula reads.
    pub(crate) fn params_used(&self) -> usize {
        self.formula.param_count()
    }

    /// The `.fdn` form of the source: one line, with `\` and line breaks escaped
    /// (`\\`, `\n`) — a view file holds one `key=value` per line.
    pub(crate) fn source_line(&self) -> String {
        escape_line(&self.source)
    }

    /// The `.fdn` form of the parameters: `re,im;re,im;…` for the ones the formula reads.
    pub(crate) fn params_line(&self) -> String {
        self.params[..self.params_used()]
            .iter()
            .map(|(re, im)| format!("{re:?},{im:?}"))
            .collect::<Vec<_>>()
            .join(";")
    }
}

/// One line on how deep `formula` renders, for the dialog — of the text as typed, so the answer
/// is there before Apply. Both limits are MEASURED.
///
/// With a perturbed step (`ir::perturb`): perturbation in df32, and in floatexp past 1e28× as for
/// the built-ins — `z² + c` matches the built-in Mandelbrot on the deepest corpus spiral at 1e38×,
/// 1e50× and 1e100× (mean Δ 3.5–3.9 per channel, the filament aliasing the two show at 1e12×
/// too), where df32 alone had broken at 1e38× (67).
///
/// Without one, the direct path, NOT the double-single theory: current NVIDIA and AMD shader
/// compilers fold the error-free transforms df32 relies on (`--gputest`; RTX 3080 / NVIDIA 616.92:
/// df_add error 8.1e-8), so the pixel's `c` itself is single precision. At −1.64+0.36i and
/// 549,309× a formula with NO functions broke into the same 17×4-px bricks as one with sin and cos
/// (338 distinct values of 48,400 pixels): the limit is where one f32 step of `c` spans a pixel,
/// whatever the formula's precision tier.
pub(crate) fn depth_note_for(formula: &ir::Formula) -> String {
    match ir::perturb::perturbed_formula(formula) {
        Ok(_) => "Deep zoom by perturbation, extended range past 1e28x as for the built-in fractals \
                  (every step iterated: no series approximation or BLA for custom formulas yet)."
            .to_string(),
        Err(why) => format!(
            "Direct rendering ({} has no deep-zoom form yet): sharp until one single-precision step \
             of c spans a pixel — about 1e4x to 1e5x, less far from the origin.",
            why.0
        ),
    }
}

/// `\` → `\\`, line break → `\n` (CR LF and lone CR included), so any source fits one line.
pub(crate) fn escape_line(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut chars = src.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                out.push_str("\\n");
            }
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out
}

/// The inverse of [`escape_line`]. An unknown escape is kept as written.
pub(crate) fn unescape_line(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// `re,im;re,im;…` → pairs. `None` if any part is not a finite number.
pub(crate) fn parse_params_line(line: &str) -> Option<Vec<(f64, f64)>> {
    if line.trim().is_empty() {
        return Some(Vec::new());
    }
    line.split(';')
        .map(|pair| {
            let (re, im) = pair.split_once(',')?;
            let (re, im): (f64, f64) = (re.trim().parse().ok()?, im.trim().parse().ok()?);
            (re.is_finite() && im.is_finite()).then_some((re, im))
        })
        .collect()
}

#[cfg(test)]
#[path = "custom_formula_tests.rs"]
mod tests;
