//! The iterate module an EXPONENTIAL-MAP STRIP renders with (`design/zoom-video.md` §4;
//! [`crate::StripMap`]).
//!
//! `mandelbrot.wgsl` marks six STRIP_* slots: the texel placement of the three iterate prologues
//! (`iterate_at`, `fs_iterate_chunk`, `chunk_fe_at`), the distance estimate's texel size
//! (`de_log2`), and `fs_resolve`'s texel size and output. This fills them with calls into
//! `strip.wgsl`, appended to the module. Every other pipeline compiles `mandelbrot.wgsl` as it is.
//!
//! ⭐Why a module of its own: written into the shared module, the strip code cost every other render
//! 1-2% on both of PLUTO's cards (37-38 of 43 bench-matrix segments slower; an A/A control showed
//! 24/43 and 16/43) — first as a runtime branch, then still with the branch compiled out by an
//! `override`, because the shared prologue had been restructured around it. Here the shared text is
//! main's to the character, and the strip's text exists only in a strip's pipelines.

use egui_wgpu::wgpu;

const FIXED: &str = include_str!("mandelbrot.wgsl");
const STRIP_FNS: &str = include_str!("strip.wgsl");

/// The slots, in source order, and what a strip's module puts in each.
const SLOTS: [(&str, &str, &str); 6] = [
    ("// @@STRIP_STEPLOG_BEGIN", "// @@STRIP_STEPLOG_END", "    let steplog = strip_steplog;\n"),
    (
        "// @@STRIP_AT_BEGIN",
        "// @@STRIP_AT_END",
        "    let so = strip_offset(gx, gy);\n    let off_re = so.re;\n    let off_im = so.im;\n",
    ),
    (
        "// @@STRIP_CHUNK_BEGIN",
        "// @@STRIP_CHUNK_END",
        "    let gx = iu.px_offset.x + in.pos.x;\n    let gy = iu.px_offset.y + in.pos.y;\n    let so = strip_offset(gx, gy);\n    let off_re = so.re;\n    let off_im = so.im;\n",
    ),
    (
        "// @@STRIP_FE_BEGIN",
        "// @@STRIP_FE_END",
        "    let gx = iu.px_offset.x + pos.x;\n    let gy = iu.px_offset.y + pos.y;\n    let so = strip_offset(gx, gy);\n    let off_re = so.re;\n    let off_im = so.im;\n",
    ),
    ("// @@STRIP_RESOLVE_AT_BEGIN", "// @@STRIP_RESOLVE_AT_END", "    strip_set_steplog(iu.px_offset.y + in.pos.y);\n"),
    (
        "// @@STRIP_RESOLVE_OUT_BEGIN",
        "// @@STRIP_RESOLVE_OUT_END",
        "    let nu = log(log(mag2) * 0.5 / log(2.0)) / log(chunk_power());\n    return FragOut(vec4<f32>(sm.z, nrm.x, nrm.y, de), strip_aux(info_iter(sm), 1.0 - nu, zf));\n",
    ),
];

/// The pipeline-compile account's source key for the strip module (`pipeline_compile::set_source`):
/// any fixed value other than the built-in module's 0.
const SOURCE_KEY: u64 = 0x5354_5249_5045_4d41; // "STRIPEMA"

/// The byte range of the whole line holding `marker` (newline included).
fn marker_line(src: &str, marker: &str) -> Result<(usize, usize), String> {
    let i = src.find(marker).ok_or_else(|| format!("the iterate module has no `{marker}`"))?;
    let b = src[..i].rfind('\n').map_or(0, |n| n + 1);
    let e = src[i..].find('\n').map_or(src.len(), |n| i + n + 1);
    Ok((b, e))
}

/// The strip module's WGSL: `mandelbrot.wgsl` with its STRIP_* slots filled, and `strip.wgsl`.
pub(crate) fn source() -> Result<String, String> {
    let mut out = String::with_capacity(FIXED.len() + STRIP_FNS.len() + 1024);
    let mut at = 0;
    for (begin, end, fill) in SLOTS {
        let (b0, _) = marker_line(FIXED, begin)?;
        let (_, e1) = marker_line(FIXED, end)?;
        if b0 < at || e1 <= b0 {
            return Err(format!("the iterate module's `{begin}` slot is out of order"));
        }
        out.push_str(&FIXED[at..b0]);
        out.push_str(fill);
        at = e1;
    }
    out.push_str(&FIXED[at..]);
    out.push('\n');
    out.push_str(STRIP_FNS);
    Ok(out)
}

/// The strip module, compiled. Panics only if the slots in `mandelbrot.wgsl` are broken, which the
/// unit test that validates [`source`] catches first.
pub(crate) fn module(device: &wgpu::Device) -> wgpu::ShaderModule {
    let src = source().expect("the strip module's slots");
    crate::pipeline_compile::set_source(SOURCE_KEY);
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fractadyne.strip"),
        source: wgpu::ShaderSource::Wgsl(src.into()),
    })
}
