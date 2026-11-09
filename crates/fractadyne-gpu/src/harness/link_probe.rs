//! `--gl-link-probe`: the renderer's first pipeline built from a given WGSL source, for finding what
//! in the shader a driver's compiler crashes on (Eris, 2026-11-09: Intel's HD Graphics 4400 OpenGL
//! driver dies linking `fs_iterate`, on the newest driver the card has too). The app's harness
//! (`--gl-link-bisect`) makes the variants and links each in a process of its own.

use egui_wgpu::wgpu;

/// The renderer's shader source, as `shader_module` compiles it.
pub const RENDERER_WGSL: &str = include_str!("../mandelbrot.wgsl");

/// Build `fs_iterate`'s pipeline from `wgsl` as `Renderer::new` builds it from the shipped source:
/// the same bind-group layout, vertex stage, targets and constants. The constants include the
/// device's product barrier, so ask [`crate::detect_product_fusion`] first, as the app does.
pub fn build_iterate_pipeline(device: &wgpu::Device, wgsl: &str) {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fractadyne.shader"),
        source: wgpu::ShaderSource::Wgsl(wgsl.into()),
    });
    let iter_bgl = crate::iter_bind_group_layout(device);
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("fractadyne.iter_layout"),
        bind_group_layouts: &[&iter_bgl],
        push_constant_ranges: &[],
    });
    let _pipeline = crate::fullscreen_pipeline(
        device,
        &shader,
        &layout,
        "fs_iterate",
        &[crate::ITER_FORMAT, crate::ITER_FORMAT],
        "fractadyne.iter_pipeline",
    );
}
