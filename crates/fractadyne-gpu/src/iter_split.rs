//! The iterate pipelines split by precision mode, for a driver whose compiler cannot take
//! `iterate_at` whole.
//!
//! `iterate_at` holds all three precision modes: direct df32 (mode 1), floatexp perturbation (mode
//! 2) and df32 perturbation (mode 0). Intel's HD Graphics 4400 OpenGL driver (Eris, 2026-11-09, on
//! the last driver the card has too) dies with an access violation linking a pipeline that large.
//! `--gl-link-bisect` found no construct to blame, only size: with every helper call inlined, as
//! that compiler does, each variant above about 830k characters of code crashed and each one below
//! linked, and the whole is about 1.0M. Each half of the split is about half of it.
//!
//! So on OpenGL every iterate pipeline built from the fixed module, or from a strip module made from
//! it, is two: one from the module with mode 2 cut out and one from the module with modes 0 and 1
//! cut out. Each draw takes the half for the mode its uniforms carry ([`ModePipelines::for_mode`]).
//! A mode runs the same text in its half as in the whole module. `FRACTADYNE_SPLIT_ITERATE=1` splits
//! on every graphics API and `=0` on none, to measure the split elsewhere.

use egui_wgpu::wgpu;

/// The marker lines each cut runs between (`mandelbrot.wgsl`).
const DIRECT: (&str, &str) = ("// @@SPLIT_DIRECT_BEGIN", "// @@SPLIT_DIRECT_END");
const FE: (&str, &str) = ("// @@CUSTOM_FE_BEGIN", "// @@CUSTOM_FE_END");
const PERT: (&str, &str) = ("// @@SPLIT_PERT_BEGIN", "// @@SPLIT_PERT_END");

/// What a half's branch for a mode it was cut without returns. No draw asks it, since each draw
/// takes the half for its mode; one that did would get the gather pass's never-read texel.
const ABSENT: &str = "{ return FragOut(vec4<f32>(-1.0, 0.0, 0.0, 1.0e30), AUX_NONE); }";

/// The byte range of the lines from the one holding `begin` through the one holding `end`, each
/// marker found once.
fn region(src: &str, (begin, end): (&str, &str)) -> Option<(usize, usize)> {
    let b = src.find(begin)?;
    let e = b + src[b..].find(end)?;
    if src[b + begin.len()..].contains(begin) || src[e + end.len()..].contains(end) {
        return None;
    }
    let start = src[..b].rfind('\n').map_or(0, |n| n + 1);
    let stop = src[e..].find('\n').map_or(src.len(), |n| e + n + 1);
    Some((start, stop))
}

/// `src` cut in two for a split iterate pipeline: (modes 0 and 1, mode 2). `None` when a marker is
/// missing or out of order, as in a custom formula's module, whose mode 2 is its own.
pub fn split_sources(src: &str) -> Option<(String, String)> {
    let d = region(src, DIRECT)?;
    let f = region(src, FE)?;
    let p = region(src, PERT)?;
    if !(d.1 <= f.0 && f.1 <= p.0) {
        return None;
    }
    let without_fe = format!("{}    else if (iu.mode == 2u) {ABSENT}\n{}", &src[..f.0], &src[f.1..]);
    let fe_only = format!(
        "{}    if (iu.mode == 1u) {ABSENT}\n{}    else {ABSENT}\n{}",
        &src[..d.0],
        &src[d.1..p.0],
        &src[p.1..]
    );
    Some((without_fe, fe_only))
}

/// Whether `device`'s iterate pipelines are split (see the module docs).
pub fn split_for(device: &wgpu::Device) -> bool {
    static FORCED: std::sync::OnceLock<Option<bool>> = std::sync::OnceLock::new();
    let forced = *FORCED.get_or_init(|| match std::env::var("FRACTADYNE_SPLIT_ITERATE").ok().as_deref() {
        Some("1") => Some(true),
        Some("0") => Some(false),
        _ => None,
    });
    forced.unwrap_or_else(|| is_gl(device))
}

/// Whether an OpenGL device is behind `device`. wgpu compiles its OpenGL backend in on Windows and
/// on Unix outside Apple's; elsewhere there is none.
#[cfg(any(windows, all(unix, not(target_vendor = "apple"))))]
fn is_gl(device: &wgpu::Device) -> bool {
    // SAFETY: the callback only asks whether the device is an OpenGL one, and keeps nothing of it.
    unsafe { device.as_hal::<wgpu::hal::api::Gles, _, _>(|d| d.is_some()) }
}

#[cfg(not(any(windows, all(unix, not(target_vendor = "apple")))))]
fn is_gl(_device: &wgpu::Device) -> bool {
    false
}

/// One iterate entry point's pipeline: whole, or the two halves of a split.
pub(crate) struct ModePipelines {
    /// The whole pipeline, or the half with modes 0 and 1.
    main: wgpu::RenderPipeline,
    /// The half with mode 2, when split.
    fe: Option<wgpu::RenderPipeline>,
}

impl ModePipelines {
    /// The pipeline for a draw whose uniforms carry `mode`.
    pub(crate) fn for_mode(&self, mode: u32) -> &wgpu::RenderPipeline {
        match (&self.fe, mode) {
            (Some(fe), 2) => fe,
            _ => &self.main,
        }
    }
}

/// The two modules a split's halves are built from.
pub(crate) struct SplitModules {
    without_fe: wgpu::ShaderModule,
    fe_only: wgpu::ShaderModule,
}

impl SplitModules {
    /// The halves of `src` (the fixed module's source, or a strip module's) on `device`, or `None`
    /// when the device takes the iterate pipelines whole.
    pub(crate) fn new(device: &wgpu::Device, src: &str, label: &str) -> Option<Self> {
        if !split_for(device) {
            return None;
        }
        // The fixed module and the strip module carry every marker (the unit tests hold them to
        // it), so a source that cannot be cut is a bug that would bring the crash back.
        let Some((without_fe, fe_only)) = split_sources(src) else {
            debug_assert!(false, "{label}: the iterate module cannot be split");
            return None;
        };
        let module = |text: String, half: &str| {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&format!("{label}.{half}")),
                source: wgpu::ShaderSource::Wgsl(text.into()),
            })
        };
        Some(Self { without_fe: module(without_fe, "df"), fe_only: module(fe_only, "fe") })
    }

    /// One entry point's pipelines: `make(module, label suffix)` on `whole` when `split` is `None`,
    /// else once per half (suffixes `.df` and `.fe`).
    pub(crate) fn build(
        split: Option<&Self>,
        whole: &wgpu::ShaderModule,
        mut make: impl FnMut(&wgpu::ShaderModule, &str) -> wgpu::RenderPipeline,
    ) -> ModePipelines {
        match split {
            None => ModePipelines { main: make(whole, ""), fe: None },
            Some(s) => ModePipelines { main: make(&s.without_fe, ".df"), fe: Some(make(&s.fe_only, ".fe")) },
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/iter_split/iter_split_tests.rs"]
mod iter_split_tests;
