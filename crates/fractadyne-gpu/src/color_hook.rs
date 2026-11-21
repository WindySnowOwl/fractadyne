//! A custom coloring hook (design/coloring-plan.md B10): a short WGSL function the user writes,
//! `fn custom_color(l: Lanes) -> vec3<f32>`, colors every sample of the view in place of the
//! method's color, live on the GPU at any depth.
//!
//! [`build`] makes its module: the fixed module's coloring pass, cut out with its vertex stage and
//! the generated ids, `shade_aa` and `shade` renamed `builtin_shade_aa` and `builtin_shade`; the
//! interface (`color_hook.wgsl`: the `Lanes` struct, `p1()`..`p4()`, `palette_map`,
//! `palette_inside`, and the `shade_aa` / `shade` that call the hook); then the user's text, last,
//! so an error's line is the text's own line plus a fixed offset. Only the color pipeline is built
//! from it ([`ColorHook::pipeline`]): the iteration, and so every pixel's numbers, are the view's
//! as they are without a hook. Small next to the whole renderer, so it compiles in milliseconds and
//! a saved file recolors at once.
//!
//! It is checked before any device sees it, so a mistake is a sentence naming the text's line and
//! never a driver-side validation panic: naga parses and validates the whole module, and then
//! [`build`] refuses what would get past validation but not past the color pass (each refusal
//! names its line):
//! - a loop of any kind (`for`, `while`, `loop`): the color pass runs once for every sample of every
//!   pixel and must finish inside the GPU's watchdog, and a loop is where a hook would not. WGSL has
//!   no recursion, so a hook without loops is bounded by its own length. The interface's helpers
//!   (`with_lightness`) keep their own fixed loops;
//! - a module-scope `var` (a resource binding would not match the color pass's layout, and the
//!   pipeline would fail where the user cannot see why), an `override`, and an entry point.

use egui_wgpu::wgpu;
use egui_wgpu::wgpu::naga;
use fractadyne_core::coloring::{Stat, StatSet};

/// The hook function every hook defines.
pub const ENTRY: &str = "custom_color";
/// `p1`..`p4`.
pub const MAX_PARAMS: usize = 4;
/// The longest hook text accepted, bytes: a view file carries the text.
pub const MAX_TEXT: usize = 64 * 1024;

/// The example hooks: `(file name, text)`. Smooth and the fringe written out, and two ideas.
pub const EXAMPLES: &[(&str, &str)] = &[
    ("smooth.wgsl", include_str!("hooks/smooth.wgsl")),
    ("fringe.wgsl", include_str!("hooks/fringe.wgsl")),
    ("transfer-curves.wgsl", include_str!("hooks/transfer-curves.wgsl")),
    ("darken-near-set.wgsl", include_str!("hooks/darken-near-set.wgsl")),
];

/// Where the coloring pass starts in `mandelbrot.wgsl`, where it ends, and where the vertex stage
/// before the arithmetic ends.
const COLOR_BEGIN: &str = "// ---------------- coloring pass (samples the iteration texture) ----------------";
const COLOR_END: &str = "// ---------------- progressive on-settle supersampling";
const HEADER_END: &str = "// ---------------- double-single (df32) arithmetic";
/// The line before the user's text.
const USER_MARK: &str = "// ---- The coloring hook's own text (fractadyne_gpu::color_hook) ----";
/// The module's entry points, all from the fixed coloring pass; any other is the hook's.
const OWN_ENTRY_POINTS: [&str; 3] = ["vs_main", "fs_seed", "fs_color"];

/// One of `p1`..`p4` as the text's `// @param` line describes it.
#[derive(Clone, Debug, PartialEq)]
pub struct HookParam {
    pub name: String,
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

impl HookParam {
    /// `p1`..`p4` with no `// @param` line: 0 to 1, from the middle.
    fn unnamed(i: usize) -> Self {
        HookParam { name: format!("p{}", i + 1), min: 0.0, max: 1.0, default: 0.5 }
    }
}

/// A hook, built and checked: ready for [`ColorHook::pipeline`].
#[derive(Clone, Debug)]
pub struct ColorHook {
    /// What the user wrote, verbatim: the hook's identity, which sessions and view files keep.
    pub text: String,
    /// The module: the coloring pass, the interface, the text.
    pub source: String,
    /// FNV-1a of `source`: the pipeline's cache key.
    pub key: u64,
    /// The statistics the text reads (`l.stripe`, `l.triangle`, `l.trap`, `l.angle`): the
    /// iteration accumulates them whatever the method, as it does the method's own.
    pub stats: StatSet,
    /// `p1`..`p4`, always [`MAX_PARAMS`] long.
    pub params: Vec<HookParam>,
    /// Which of `p1()`..`p4()` the code calls (comments aside): with the ones a `// @param` line
    /// names, the sliders the dialog shows.
    pub used: [bool; MAX_PARAMS],
}

/// Why a hook does not build: where in the text, when it is in the text, and what.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HookError {
    /// The text's own line, from 1.
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub message: String,
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.line, self.column) {
            (Some(l), Some(c)) => write!(f, "line {l}, column {c}: {}", self.message),
            (Some(l), None) => write!(f, "line {l}: {}", self.message),
            _ => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for HookError {}

fn err(message: impl Into<String>) -> HookError {
    HookError { line: None, column: None, message: message.into() }
}

/// Everything before the user's text: the coloring pass with its two shading functions renamed,
/// the generated ids, the interface and [`USER_MARK`]. `Err` only if `mandelbrot.wgsl` no longer
/// has the markers this cuts at (the unit tests hold them).
fn prefix() -> Result<&'static str, HookError> {
    static PREFIX: std::sync::LazyLock<Result<String, String>> = std::sync::LazyLock::new(|| {
        let file = include_str!("mandelbrot.wgsl");
        let header = file.find(HEADER_END).ok_or("no vertex stage")?;
        let begin = file.find(COLOR_BEGIN).ok_or("no coloring pass")?;
        let end = file[begin..].find(COLOR_END).ok_or("no end to the coloring pass")? + begin;
        let mut color = file[begin..end].to_string();
        for (from, to) in [("fn shade_aa(", "fn builtin_shade_aa("), ("fn shade(", "fn builtin_shade(")] {
            if color.matches(from).count() != 1 {
                return Err(format!("`{from}` is not defined once in the coloring pass"));
            }
            color = color.replacen(from, to, 1);
        }
        // What `renderer_wgsl` appends to the file: the interior palette's place and the ids.
        let ids = &crate::renderer_wgsl()[file.len()..];
        Ok(format!("{}{color}{ids}\n{}\n{USER_MARK}\n", &file[..header], include_str!("color_hook.wgsl")))
    });
    PREFIX.as_ref().map(String::as_str).map_err(|e| err(format!("the coloring pass could not be cut out: {e}")))
}

/// `text` with its comments blanked out (each character a space, line breaks kept), so a search
/// over it sees only code and its offsets are the text's.
fn code_only(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    let mut depth = 0usize; // WGSL block comments nest
    while i < b.len() {
        let two = &b[i..(i + 2).min(b.len())];
        if depth == 0 && two == b"//" {
            while i < b.len() && b[i] != b'\n' {
                out.push(b' ');
                i += 1;
            }
        } else if two == b"/*" {
            depth += 1;
            out.extend_from_slice(b"  ");
            i += 2;
        } else if depth > 0 && two == b"*/" {
            depth -= 1;
            out.extend_from_slice(b"  ");
            i += 2;
        } else {
            out.push(if depth > 0 && b[i] != b'\n' { b' ' } else { b[i] });
            i += 1;
        }
    }
    // Only ASCII bytes were replaced, by ASCII: still UTF-8 where the text was, but a non-ASCII
    // character inside a comment was blanked byte by byte, so convert lossily.
    String::from_utf8_lossy(&out).into_owned()
}

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Whether `code` (comments blanked) calls the function `name`: the name as a whole word, then `(`.
fn calls(code: &str, name: &str) -> bool {
    code.match_indices(name).any(|(at, _)| {
        let before_ok = !code[..at].ends_with(is_ident);
        let after = &code[at + name.len()..];
        before_ok && !after.starts_with(is_ident) && after.trim_start().starts_with('(')
    })
}

/// The statistics `code` reads: `.stripe`, `.triangle`, `.trap`, `.angle` as whole field names.
fn stats_read(code: &str) -> StatSet {
    Stat::ALL.into_iter().fold(StatSet::NONE, |set, s| {
        let field = format!(".{}", s.spec().key);
        let read = code.match_indices(&field).any(|(at, _)| !code[at + field.len()..].starts_with(is_ident));
        if read { set.with(s) } else { set }
    })
}

/// The 1-based line `byte` of `text` is on.
fn line_of(text: &str, byte: usize) -> u32 {
    text[..byte.min(text.len())].matches('\n').count() as u32 + 1
}

/// `p1`..`p4` from the text's `// @param pN "Name" min max default` lines (the name may be one word
/// unquoted; min and max default to 0 and 1, the default to their middle).
fn params_of(text: &str) -> Result<Vec<HookParam>, HookError> {
    let mut params: Vec<HookParam> = (0..MAX_PARAMS).map(HookParam::unnamed).collect();
    for (n, line) in text.lines().enumerate() {
        let Some(rest) = line.trim_start().strip_prefix("//").map(str::trim_start) else { continue };
        let Some(rest) = rest.strip_prefix("@param") else { continue };
        let bad = |why: &str| HookError {
            line: Some(n as u32 + 1),
            column: None,
            message: format!("{why} (a parameter line reads `// @param p1 \"Name\" min max default`)"),
        };
        let rest = rest.trim();
        let (which, rest) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
        let idx = match which {
            "p1" => 0,
            "p2" => 1,
            "p3" => 2,
            "p4" => 3,
            _ => return Err(bad(&format!("`{which}` is not one of p1, p2, p3, p4"))),
        };
        let rest = rest.trim_start();
        let (name, rest) = if let Some(q) = rest.strip_prefix('"') {
            let Some(close) = q.find('"') else { return Err(bad("the name has no closing quote")) };
            (q[..close].to_string(), &q[close + 1..])
        } else {
            let (w, r) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
            (w.to_string(), r)
        };
        let nums: Vec<f32> = rest
            .split_whitespace()
            .map(|t| t.parse::<f32>().ok().filter(|v| v.is_finite()))
            .collect::<Option<_>>()
            .ok_or_else(|| bad("min, max and default must be numbers"))?;
        let (min, max) = match nums.as_slice() {
            [] => (0.0, 1.0),
            [lo, hi] | [lo, hi, _] => (*lo, *hi),
            _ => return Err(bad("give min and max, then the default if any")),
        };
        if min >= max {
            return Err(bad("min must be below max"));
        }
        let default = nums.get(2).copied().unwrap_or(0.5 * (min + max)).clamp(min, max);
        let name = if name.trim().is_empty() { format!("p{}", idx + 1) } else { name.trim().to_string() };
        params[idx] = HookParam { name, min, max, default };
    }
    Ok(params)
}

/// Where a span of the module points: the text's own line and column when it is in the text, else
/// `None` (the coloring pass or the interface).
struct Locator<'a> {
    source: &'a str,
    /// The text's first byte in `source`.
    start: usize,
    /// Lines of `source` before the text's first.
    lines_before: u32,
}

impl Locator<'_> {
    fn at(&self, span: naga::Span) -> Option<(u32, u32)> {
        let range = span.to_range()?;
        if range.start < self.start {
            return None;
        }
        let loc = span.location(self.source);
        Some((loc.line_number.saturating_sub(self.lines_before), loc.line_position))
    }

    /// An error at the narrowest of `spans` in the text (a validation error carries the function's
    /// span and the statement's: the statement is where it is), else at none.
    fn error(&self, spans: impl Iterator<Item = naga::Span>, message: String) -> HookError {
        let mut in_prefix = false;
        let mut best: Option<(usize, (u32, u32))> = None;
        for s in spans {
            let Some(range) = s.to_range() else { continue };
            match self.at(s) {
                Some(at) if best.is_none_or(|(len, _)| range.len() < len) => best = Some((range.len(), at)),
                Some(_) => {}
                None => in_prefix = true,
            }
        }
        let message = tidy(&message);
        match best {
            Some((_, (line, column))) => HookError { line: Some(line), column: Some(column), message },
            None if in_prefix => HookError { line: None, column: None, message: format!("{message} (in the coloring pass this hook is joined to)") },
            None => HookError { line: None, column: None, message },
        }
    }
}

/// naga's message without its internal handle numbers (`Function [22] 'f'`, `Some([1])`), which
/// say nothing to the person reading it.
fn tidy(message: &str) -> String {
    let mut out = String::with_capacity(message.len());
    let mut rest = message;
    while !rest.is_empty() {
        // `Some([digits])` and ` [digits]` go; anything else is kept.
        let handle = |s: &str, open: &str, close: &str| -> Option<usize> {
            let inner = s.strip_prefix(open)?;
            let digits = inner.find(|c: char| !c.is_ascii_digit())?;
            (digits > 0 && inner[digits..].starts_with(close)).then_some(open.len() + digits + close.len())
        };
        if let Some(n) = handle(rest, "Some([", "])").or_else(|| handle(rest, " [", "]")) {
            rest = &rest[n..];
            continue;
        }
        let ch = rest.chars().next().unwrap_or(' ');
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A message and the messages of what caused it, joined.
fn chain(e: &dyn std::error::Error) -> String {
    let mut s = e.to_string();
    let mut cur = e.source();
    while let Some(c) = cur {
        let m = c.to_string();
        if !s.contains(&m) {
            s.push_str(": ");
            s.push_str(&m);
        }
        cur = c.source();
    }
    s
}

/// The first loop in `block`, by its span.
fn first_loop(block: &naga::Block) -> Option<naga::Span> {
    for (stmt, span) in block.span_iter() {
        let found = match stmt {
            naga::Statement::Loop { .. } => return Some(*span),
            naga::Statement::Block(b) => first_loop(b),
            naga::Statement::If { accept, reject, .. } => first_loop(accept).or_else(|| first_loop(reject)),
            naga::Statement::Switch { cases, .. } => cases.iter().find_map(|c| first_loop(&c.body)),
            _ => None,
        };
        if found.is_some() {
            return found;
        }
    }
    None
}

/// Build and check the hook `text`: its module, its statistics and its parameters, or why not.
pub fn build(text: &str) -> Result<ColorHook, HookError> {
    // One text, one hook: a file saved with Windows line ends is the same hook as the same text with
    // Unix ones (its key, the text a view or a farm job carries, whose escaping keeps only `\n`).
    let normalized;
    let text = if text.contains('\r') {
        normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        normalized.as_str()
    } else {
        text
    };
    if text.len() > MAX_TEXT {
        return Err(err(format!("a hook of {} KB (at most {} KB)", text.len() / 1024, MAX_TEXT / 1024)));
    }
    if text.contains('\0') {
        return Err(err("the text holds a NUL character (not a text file?)"));
    }
    let code = code_only(text);
    // Said before naga, whose message for a missing function names the interface's call.
    let defines_entry = code.match_indices(ENTRY).any(|(at, _)| {
        let before = code[..at].trim_end();
        let after = &code[at + ENTRY.len()..];
        before.ends_with("fn") && !before[..before.len() - 2].ends_with(is_ident) && !after.starts_with(is_ident)
    });
    if !defines_entry {
        return Err(err(format!("the text defines no `fn {ENTRY}(l: Lanes) -> vec3<f32>`")));
    }
    let params = params_of(text)?;
    let prefix = prefix()?;
    let source = format!("{prefix}{text}\n");
    let loc = Locator { source: &source, start: prefix.len(), lines_before: prefix.matches('\n').count() as u32 };

    let module = naga::front::wgsl::parse_str(&source).map_err(|e| {
        let spans: Vec<naga::Span> = e.labels().map(|(s, _)| s).collect();
        let mut he = loc.error(spans.into_iter(), e.message().to_string());
        if he.line.is_none() {
            if let Some(l) = e.location(&source) {
                // Not on any label in the text: the parser's own position, when that is.
                if l.offset as usize >= loc.start {
                    he.line = Some(l.line_number.saturating_sub(loc.lines_before));
                    he.column = Some(l.line_position);
                }
            }
        }
        if e.message().contains("redefinition") {
            he.message.push_str(" (a name the coloring pass uses already: choose another)");
        }
        he
    })?;
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::empty())
        .validate(&module)
        .map_err(|e| loc.error(e.spans().map(|(s, _)| *s), chain(e.as_inner())))?;

    // What validation lets through and the color pass does not.
    let refuse = |span: naga::Span, why: &str| {
        let (line, column) = loc.at(span).map_or((None, None), |(l, c)| (Some(l), Some(c)));
        HookError { line, column, message: why.to_string() }
    };
    for (h, _) in module.global_variables.iter() {
        let span = module.global_variables.get_span(h);
        if loc.at(span).is_some() {
            return Err(refuse(
                span,
                "a module-scope `var`: a hook keeps its values in `let` and `var` inside its functions, and reads the view only through `l`",
            ));
        }
    }
    for (h, _) in module.overrides.iter() {
        let span = module.overrides.get_span(h);
        if loc.at(span).is_some() {
            return Err(refuse(span, "an `override`: use `const`"));
        }
    }
    if let Some(ep) = module.entry_points.iter().find(|ep| !OWN_ENTRY_POINTS.contains(&ep.name.as_str())) {
        let line = code.find(&format!("fn {}", ep.name)).map(|at| line_of(text, at));
        return Err(HookError {
            line,
            column: None,
            message: format!("`{}` is an entry point (`@fragment`, `@vertex` or `@compute`): a hook is called by the coloring pass", ep.name),
        });
    }
    for (h, f) in module.functions.iter() {
        if loc.at(module.functions.get_span(h)).is_none() {
            continue;
        }
        if let Some(span) = first_loop(&f.body) {
            return Err(refuse(
                span,
                "a loop: the color pass runs once for every sample of every pixel and must finish within the GPU's time limit, so a hook has no loops (write the steps out)",
            ));
        }
    }
    let key = crate::custom::fnv1a(source.as_bytes());
    let used = std::array::from_fn(|i| calls(&code, &format!("p{}", i + 1)));
    Ok(ColorHook { text: text.to_string(), source, key, stats: stats_read(&code), params, used })
}

impl ColorHook {
    /// Whether a `// @param` line describes parameter `i` (0 for `p1`).
    pub fn named(&self, i: usize) -> bool {
        self.params.get(i).is_some_and(|p| *p != HookParam::unnamed(i))
    }

    /// The default value of each parameter, `p1`..`p4`.
    pub fn defaults(&self) -> [f32; MAX_PARAMS] {
        std::array::from_fn(|i| self.params[i].default)
    }

    /// The hook's color pipeline, `fs_color` from its module, writing `format` through `layout` (the
    /// color pass's bind-group layout).
    pub(crate) fn pipeline(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        format: wgpu::TextureFormat,
        label: &str,
    ) -> wgpu::RenderPipeline {
        crate::pipeline_compile::set_source(self.key);
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fractadyne.color_hook"),
            source: wgpu::ShaderSource::Wgsl(self.source.as_str().into()),
        });
        crate::fullscreen_pipeline(device, &module, layout, "fs_color", &[format], label)
    }
}

/// The color pipeline for `hook`, or the fixed one from `shader` without: what every color pass
/// builds.
pub(crate) fn color_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
    label: &str,
    hook: Option<&ColorHook>,
) -> wgpu::RenderPipeline {
    match hook {
        Some(h) => h.pipeline(device, layout, format, label),
        None => crate::fullscreen_pipeline(device, shader, layout, "fs_color", &[format], label),
    }
}

#[cfg(test)]
#[path = "../tests/unit/color_hook/tests.rs"]
mod tests;
