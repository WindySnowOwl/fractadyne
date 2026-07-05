//! Offscreen high-resolution export: render a whole frame (perturbation, all modes) to a
//! texture and read it back to the CPU. `render_export` (colored image), `render_iter` (raw
//! iteration data as EXR), and `color_iter_buffer` (color a CPU-side iteration buffer — the
//! glitch-correction path). Uses the shared pipeline/uniform scaffolding from the crate root.

use crate::{
    color_bind_group_layout, fullscreen_pipeline, iter_bind_group_layout, make_color_bg,
    make_iter_bg, make_iter_texture, make_orbit_buffer, method_needs_aux, shader_module,
    ColorUniforms, IterUniforms, RefOffset, Vignette, ITER_FORMAT,
};
use egui_wgpu::wgpu;
use std::sync::Arc;

const EXPORT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

/// Failure modes of the offscreen GPU render / readback path. `Canceled`'s `Display` is exactly
/// `"canceled"` (load-bearing: callers may still string-compare it) and `Readback` folds the mpsc
/// / `map_async` readback failures — neither of which any caller distinguishes further.
#[derive(Debug, thiserror::Error)]
pub enum GpuError {
    /// The caller's cancel flag was set mid-render (a tiled export aborts between tiles).
    #[error("canceled")]
    Canceled,
    /// A GPU buffer readback failed — the mpsc channel dropped or `map_async` reported an error.
    #[error("GPU readback failed: {0}")]
    Readback(String),
}

/// Everything needed to render one frame offscreen at an arbitrary resolution.
/// Mirrors the live view's parameters (the app fills it from the current view).
#[derive(Clone)]
pub struct ExportRequest {
    pub width: u32,
    pub height: u32,
    pub ss: u32,
    /// Complex span *mantissa* (`span · 2^-delta_exp`, O(1)) — see [`fractadyne_core::GpuScale`].
    pub span_mantissa: fractadyne_core::SpanMantissa,
    pub center: [f32; 4],
    pub ref_offset: RefOffset,
    pub delta_exp: i32,
    /// Series-approximation skip (0 = none) + order-3 coeffs (complex df32 mantissa × 2^exp).
    pub sa_skip: u32,
    /// 1 = flag Pauldelbrot-glitched pixels with a `-2` sentinel in the iteration texture's `r`
    /// channel (multi-reference correction passes read this back). 0 for normal rendering.
    pub glitch_on: u32,
    /// Spotlight vignette (guided-tour exports); `on == 0` disables it.
    pub vignette: Vignette,
    pub sa_a: [f32; 4],
    pub sa_a_exp: i32,
    pub sa_b: [f32; 4],
    pub sa_b_exp: i32,
    pub sa_c: [f32; 4],
    pub sa_c_exp: i32,
    pub julia_c: [f32; 4],
    pub orbit: Arc<Vec<[f32; 4]>>,
    pub orbit_len: u32,
    /// BLA tree (flattened) appended after the orbit; `bla_on = 1` enables the traversal.
    pub bla: Arc<Vec<[f32; 4]>>,
    pub bla_on: u32,
    pub max_iter: u32,
    pub mode: u32,
    pub formula: u32,
    pub julia: u32,
    pub cycle: f32,
    pub offset: f32,
    pub stop_count: u32,
    pub stops: [[f32; 4]; 8],
    pub light: u32,
    pub light_angle: f32,
    pub light_height: f32,
    pub de_on: u32,
    pub de_strength: f32,
    pub de_width: f32,
    pub de_phase: f32,
    pub color_method: u32,
    pub stripe_freq: f32,
    pub trap_type: u32,
    pub aa_filter: u32,
    pub interior_col: [f32; 4],
}

/// The actual `(width, height)` an export produced after clamping to the GPU's
/// max texture dimension, plus the effective supersampling used.
pub struct ExportResult {
    pub width: u32,
    pub height: u32,
    pub ss: u32,
    /// Linear RGBA, row-major, `width*height*4` floats.
    pub pixels: Vec<f32>,
}

/// Render `req` offscreen and read the colored image back to the CPU. Synchronous
/// (blocks until the GPU finishes). Single-texture for now, so the result is clamped
/// to the device's max 2D texture dimension (e.g. 8192); supersampling is reduced if
/// `resolution × ss` would exceed it.
pub fn render_export(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    req: &ExportRequest,
    progress: &std::sync::atomic::AtomicU32,
    cancel: &std::sync::atomic::AtomicBool,
) -> Result<ExportResult, GpuError> {
    use std::sync::atomic::Ordering::Relaxed;
    let max_dim = device.limits().max_texture_dimension_2d;
    let max_buf = device.limits().max_buffer_size;
    let w = req.width.max(1);
    let h = req.height.max(1);
    let ss = req.ss.max(1);
    let full_iw = w as f32 * ss as f32; // full iteration resolution (across all tiles)
    let full_ih = h as f32 * ss as f32;
    // Tile size (output px): keep tile×ss within the texture cap and the per-tile
    // readback buffer (tile²·16 B) within the buffer-size limit. Tiling lets exports
    // exceed the single-texture/buffer limits without crashing. Also bound each tile by
    // *iteration work* (tile²·ss²·iter) so a single GPU submission can't run long enough
    // to trip the OS watchdog (TDR ≈ 2 s → device-lost) or monopolize the shared device
    // and freeze the live UI — the render stays responsive across many short tiles.
    const TILE_WORK_BUDGET: u64 = 20_000_000_000;
    let by_tex = (max_dim / ss).max(1);
    let by_buf = (((max_buf / 16) as f64).sqrt() as u32).max(256);
    let work_per_px = (ss as u64 * ss as u64) * (req.max_iter.max(1) as u64);
    let by_work = (((TILE_WORK_BUDGET / work_per_px.max(1)) as f64).sqrt() as u32).max(64);
    let tile = by_tex.min(by_buf).min(by_work).clamp(1, 2048);

    let shader = shader_module(device);
    let iter_bgl = iter_bind_group_layout(device);
    let color_bgl = color_bind_group_layout(device);
    let iter_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("export.iter_layout"),
        bind_group_layouts: &[&iter_bgl],
        push_constant_ranges: &[],
    });
    let color_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("export.color_layout"),
        bind_group_layouts: &[&color_bgl],
        push_constant_ranges: &[],
    });
    let iter_pipeline = fullscreen_pipeline(
        device, &shader, &iter_layout, "fs_iterate", &[ITER_FORMAT, ITER_FORMAT],
        "export.iter_pipeline",
    );
    let color_pipeline = fullscreen_pipeline(
        device, &shader, &color_layout, "fs_color", &[EXPORT_FORMAT], "export.color_pipeline",
    );

    let uniform = |label, size| {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    };
    let iter_uniform = uniform("export.iter_uniform", std::mem::size_of::<IterUniforms>() as u64);
    let color_uniform = uniform("export.color_uniform", std::mem::size_of::<ColorUniforms>() as u64);

    let orbit_cap = (req.orbit.len() + req.bla.len()).max(1) as u32;
    let orbit_buf = make_orbit_buffer(device, orbit_cap);
    if !req.orbit.is_empty() {
        queue.write_buffer(&orbit_buf, 0, bytemuck::cast_slice(req.orbit.as_slice()));
    }
    if !req.bla.is_empty() {
        let off = (req.orbit.len() * 16) as u64;
        queue.write_buffer(&orbit_buf, off, bytemuck::cast_slice(req.bla.as_slice()));
    }
    let iter_bg = make_iter_bg(device, &iter_bgl, &iter_uniform, &orbit_buf);

    // Coloring uniform is constant across tiles; step is the full-resolution step.
    let cu = ColorUniforms {
        stop_count: req.stop_count,
        cycle: req.cycle,
        offset: req.offset,
        ss,
        light: req.light,
        light_angle: req.light_angle,
        light_height: req.light_height,
        de_on: req.de_on,
        de_strength: req.de_strength,
        de_width: req.de_width,
        de_phase: req.de_phase,
        color_method: req.color_method,
        aa_filter: req.aa_filter.max(1),
        reproject: 0,
        uv_offset: [0.0, 0.0],
        uv_scale: 1.0,
        vig_on: req.vignette.on,
        vig_dim: req.vignette.dim,
        vig_soft: req.vignette.soft,
        vig_center: req.vignette.center,
        vig_radius: req.vignette.radius,
        _pad_vig: 0.0,
        interior_col: req.interior_col,
        stops: req.stops,
    };
    queue.write_buffer(&color_uniform, 0, bytemuck::bytes_of(&cu));
    let split = |v: f64| -> (f32, f32) {
        let hi = v as f32;
        (hi, (v - hi as f64) as f32)
    };
    // Step mantissa = span_mantissa (already × 2^-delta_exp) / texdim — O(1), no overflow.
    let (sxh, sxl) = split(req.span_mantissa.x / (w as f64 * ss as f64));
    let (syh, syl) = split(req.span_mantissa.y / (h as f64 * ss as f64));

    let bpp = 16u32; // Rgba32Float
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let mut pixels = vec![0.0_f32; (w as usize) * (h as usize) * 4];

    let total_tiles = (w.div_ceil(tile) * h.div_ceil(tile)).max(1);
    let mut done_tiles = 0u32;

    let mut ty0 = 0u32;
    while ty0 < h {
        let th = tile.min(h - ty0);
        let mut tx0 = 0u32;
        while tx0 < w {
            if cancel.load(Relaxed) {
                return Err(GpuError::Canceled);
            }
            let tw = tile.min(w - tx0);
            let iw = tw * ss;
            let ih = th * ss;

            let iu = IterUniforms {
                step: [sxh, sxl, syh, syl],
                ref_offset: req.ref_offset.to_array(),
                center: req.center,
                julia_c: req.julia_c,
                res: [full_iw, full_ih],
                px_offset: [(tx0 * ss) as f32, (ty0 * ss) as f32],
                max_iter: req.max_iter,
                orbit_len: req.orbit_len,
                mode: req.mode,
                formula: req.formula,
                julia: req.julia,
                delta_exp: req.delta_exp,
                color_method: req.color_method,
                stripe_freq: req.stripe_freq,
                trap_type: req.trap_type,
                aux_on: method_needs_aux(req.color_method) as u32,
                sa_skip: req.sa_skip,
                glitch_on: req.glitch_on,
                sa_a: req.sa_a,
                sa_b: req.sa_b,
                sa_c: req.sa_c,
                sa_a_exp: req.sa_a_exp,
                sa_b_exp: req.sa_b_exp,
                sa_c_exp: req.sa_c_exp,
                bla_on: req.bla_on,
            };
            queue.write_buffer(&iter_uniform, 0, bytemuck::bytes_of(&iu));

            let iter_view = make_iter_texture(device, [iw, ih]);
            let aux_view = make_iter_texture(device, [iw, ih]);
            let color_tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("export.color_tex"),
                size: wgpu::Extent3d { width: tw, height: th, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: EXPORT_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let color_view = color_tex.create_view(&wgpu::TextureViewDescriptor::default());
            let color_bg =
                make_color_bg(device, &color_bgl, &color_uniform, &iter_view, &aux_view);

            let unpadded_bpr = tw * bpp;
            let padded_bpr = unpadded_bpr.div_ceil(align) * align;
            let out_buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("export.readback"),
                size: (padded_bpr * th) as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("export.encoder"),
            });
            {
                let attach = |v| Some(wgpu::RenderPassColorAttachment {
                    view: v,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                });
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("export.iter_pass"),
                    color_attachments: &[attach(&iter_view), attach(&aux_view)],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&iter_pipeline);
                pass.set_bind_group(0, &iter_bg, &[]);
                pass.draw(0..3, 0..1);
            }
            {
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("export.color_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &color_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&color_pipeline);
                pass.set_bind_group(0, &color_bg, &[]);
                pass.draw(0..3, 0..1);
            }
            enc.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &color_tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &out_buf,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(padded_bpr),
                        rows_per_image: Some(th),
                    },
                },
                wgpu::Extent3d { width: tw, height: th, depth_or_array_layers: 1 },
            );
            queue.submit(std::iter::once(enc.finish()));

            let slice = out_buf.slice(..);
            let (tx, rx) = std::sync::mpsc::channel();
            slice.map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });
            let _ = device.poll(wgpu::Maintain::Wait);
            rx.recv()
                .map_err(|e| GpuError::Readback(e.to_string()))?
                .map_err(|e| GpuError::Readback(e.to_string()))?;

            let data = slice.get_mapped_range();
            let row_floats = (tw * 4) as usize;
            for r in 0..th {
                let src = (r * padded_bpr) as usize;
                let src_row: &[f32] =
                    bytemuck::cast_slice(&data[src..src + unpadded_bpr as usize]);
                let dst = (((ty0 + r) * w + tx0) * 4) as usize;
                pixels[dst..dst + row_floats].copy_from_slice(src_row);
            }
            drop(data);
            out_buf.unmap();

            done_tiles += 1;
            progress.store(done_tiles * 1000 / total_tiles, Relaxed);
            tx0 += tw;
        }
        ty0 += th;
    }

    Ok(ExportResult { width: w, height: h, ss, pixels })
}

/// Render only the **iteration pass** for a single (clamped) tile and read back the raw
/// iteration texture — RGBA32F per pixel: `(smooth_iter, normal.x, normal.y, DE_log2)`,
/// with `smooth_iter < 0` marking interior. For validation (`--selftest`): comparing raw
/// dwell across render modes is far more sensitive than comparing final colors. Always
/// `ss = 1`; size is clamped to the device's max 2D texture dimension.
pub fn render_iter(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    req: &ExportRequest,
) -> Result<ExportResult, GpuError> {
    let max_dim = device.limits().max_texture_dimension_2d;
    let w = req.width.clamp(1, max_dim);
    let h = req.height.clamp(1, max_dim);

    let shader = shader_module(device);
    let iter_bgl = iter_bind_group_layout(device);
    let iter_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("selftest.iter_layout"),
        bind_group_layouts: &[&iter_bgl],
        push_constant_ranges: &[],
    });
    let iter_pipeline = fullscreen_pipeline(
        device, &shader, &iter_layout, "fs_iterate", &[ITER_FORMAT, ITER_FORMAT],
        "selftest.iter_pipeline",
    );

    let iter_uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("selftest.iter_uniform"),
        size: std::mem::size_of::<IterUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let orbit_buf = make_orbit_buffer(device, (req.orbit.len() + req.bla.len()).max(1) as u32);
    if !req.orbit.is_empty() {
        queue.write_buffer(&orbit_buf, 0, bytemuck::cast_slice(req.orbit.as_slice()));
    }
    if !req.bla.is_empty() {
        let off = (req.orbit.len() * 16) as u64;
        queue.write_buffer(&orbit_buf, off, bytemuck::cast_slice(req.bla.as_slice()));
    }
    let iter_bg = make_iter_bg(device, &iter_bgl, &iter_uniform, &orbit_buf);

    let split = |v: f64| -> (f32, f32) {
        let hi = v as f32;
        (hi, (v - hi as f64) as f32)
    };
    let (sxh, sxl) = split(req.span_mantissa.x / w as f64);
    let (syh, syl) = split(req.span_mantissa.y / h as f64);
    let iu = IterUniforms {
        step: [sxh, sxl, syh, syl],
        ref_offset: req.ref_offset.to_array(),
        center: req.center,
        julia_c: req.julia_c,
        res: [w as f32, h as f32],
        px_offset: [0.0, 0.0],
        max_iter: req.max_iter,
        orbit_len: req.orbit_len,
        mode: req.mode,
        formula: req.formula,
        julia: req.julia,
        delta_exp: req.delta_exp,
        color_method: 0,
        stripe_freq: 1.0,
        trap_type: 0,
        aux_on: 0,
        sa_skip: req.sa_skip,
        glitch_on: req.glitch_on,
        sa_a: req.sa_a,
        sa_b: req.sa_b,
        sa_c: req.sa_c,
        sa_a_exp: req.sa_a_exp,
        sa_b_exp: req.sa_b_exp,
        sa_c_exp: req.sa_c_exp,
        bla_on: req.bla_on,
    };
    queue.write_buffer(&iter_uniform, 0, bytemuck::bytes_of(&iu));

    let tex = |label| {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: ITER_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
    };
    let aux_view = tex("selftest.iter_aux");
    let main_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("selftest.iter_main"),
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: ITER_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let main_copy_view = main_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let bpp = 16u32;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let unpadded_bpr = w * bpp;
    let padded_bpr = unpadded_bpr.div_ceil(align) * align;
    let out_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("selftest.readback"),
        size: (padded_bpr * h) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut enc =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("selftest.enc") });
    {
        let attach = |v| Some(wgpu::RenderPassColorAttachment {
            view: v,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        });
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("selftest.iter_pass"),
            color_attachments: &[attach(&main_copy_view), attach(&aux_view)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&iter_pipeline);
        pass.set_bind_group(0, &iter_bg, &[]);
        pass.draw(0..3, 0..1);
    }
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &main_tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &out_buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    queue.submit(std::iter::once(enc.finish()));

    let slice = out_buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = device.poll(wgpu::Maintain::Wait);
    rx.recv().map_err(|e| GpuError::Readback(e.to_string()))?.map_err(|e| GpuError::Readback(e.to_string()))?;

    let data = slice.get_mapped_range();
    let mut pixels = vec![0.0_f32; (w as usize) * (h as usize) * 4];
    let row_floats = (w * 4) as usize;
    for r in 0..h {
        let src = (r * padded_bpr) as usize;
        let src_row: &[f32] = bytemuck::cast_slice(&data[src..src + unpadded_bpr as usize]);
        let dst = (r as usize) * row_floats;
        pixels[dst..dst + row_floats].copy_from_slice(src_row);
    }
    drop(data);
    out_buf.unmap();

    Ok(ExportResult { width: w, height: h, ss: 1, pixels })
}

/// Color an already-computed iteration buffer (main target, `w*h*4` floats: smooth-iter, normal.x,
/// normal.y, DE) into a linear-RGBA image. The multi-reference glitch corrector merges several
/// `render_iter` passes into one glitch-free buffer, then hands it here to be colored. Supports the
/// non-aux coloring methods (smooth / distance / relief / glow — everything that reads only the
/// main target); aux methods (stripe/TIA/trap/decomposition) need per-orbit statistics the merged
/// buffer doesn't carry, so the caller renders those uncorrected.
pub fn color_iter_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    req: &ExportRequest,
    iter_pixels: &[f32],
) -> Result<ExportResult, GpuError> {
    let w = req.width.max(1);
    let h = req.height.max(1);
    let shader = shader_module(device);
    let color_bgl = color_bind_group_layout(device);
    let color_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("coloriter.layout"),
        bind_group_layouts: &[&color_bgl],
        push_constant_ranges: &[],
    });
    let color_pipeline = fullscreen_pipeline(
        device, &shader, &color_layout, "fs_color", &[EXPORT_FORMAT], "coloriter.pipeline",
    );

    let color_uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("coloriter.uniform"),
        size: std::mem::size_of::<ColorUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let cu = ColorUniforms {
        stop_count: req.stop_count,
        cycle: req.cycle,
        offset: req.offset,
        ss: 1,
        light: req.light,
        light_angle: req.light_angle,
        light_height: req.light_height,
        de_on: req.de_on,
        de_strength: req.de_strength,
        de_width: req.de_width,
        de_phase: req.de_phase,
        color_method: req.color_method,
        aa_filter: 1,
        reproject: 0,
        uv_offset: [0.0, 0.0],
        uv_scale: 1.0,
        vig_on: req.vignette.on,
        vig_dim: req.vignette.dim,
        vig_soft: req.vignette.soft,
        vig_center: req.vignette.center,
        vig_radius: req.vignette.radius,
        _pad_vig: 0.0,
        interior_col: req.interior_col,
        stops: req.stops,
    };
    queue.write_buffer(&color_uniform, 0, bytemuck::bytes_of(&cu));

    // Upload the merged iteration buffer into a sampled texture; a zeroed aux (unused for the
    // supported non-aux methods) satisfies the color bind group.
    let make_input = |label| device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: ITER_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let write = |tex: &wgpu::Texture, data: &[f32]| {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(data),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 16),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
    };
    let iter_tex = make_input("coloriter.iter");
    write(&iter_tex, iter_pixels);
    let aux_tex = make_input("coloriter.aux");
    write(&aux_tex, &vec![0.0_f32; (w as usize) * (h as usize) * 4]);
    let iter_view = iter_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let aux_view = aux_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let color_bg = make_color_bg(device, &color_bgl, &color_uniform, &iter_view, &aux_view);

    let color_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("coloriter.out"),
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: EXPORT_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let color_view = color_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let bpp = 16u32;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let unpadded_bpr = w * bpp;
    let padded_bpr = unpadded_bpr.div_ceil(align) * align;
    let out_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("coloriter.readback"),
        size: (padded_bpr * h) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut enc =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("coloriter.enc") });
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("coloriter.pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&color_pipeline);
        pass.set_bind_group(0, &color_bg, &[]);
        pass.draw(0..3, 0..1);
    }
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &color_tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &out_buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    queue.submit(std::iter::once(enc.finish()));

    let slice = out_buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = device.poll(wgpu::Maintain::Wait);
    rx.recv().map_err(|e| GpuError::Readback(e.to_string()))?.map_err(|e| GpuError::Readback(e.to_string()))?;

    let data = slice.get_mapped_range();
    let mut pixels = vec![0.0_f32; (w as usize) * (h as usize) * 4];
    let row_floats = (w * 4) as usize;
    for r in 0..h {
        let src = (r * padded_bpr) as usize;
        let src_row: &[f32] = bytemuck::cast_slice(&data[src..src + unpadded_bpr as usize]);
        let dst = (r as usize) * row_floats;
        pixels[dst..dst + row_floats].copy_from_slice(src_row);
    }
    drop(data);
    out_buf.unmap();

    Ok(ExportResult { width: w, height: h, ss: 1, pixels })
}
