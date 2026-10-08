//! The live renderer on a device of its own (design/multi-gpu-live.md L3).
//!
//! A motion refresh rendered on a second GPU has to be the frame the window's GPU would render from
//! the same params — the export renderer is not that frame (`fs_iterate` and the chunk entry points
//! are separately compiled programs that differ on a few hundred pixels, export.rs), so the
//! worker runs THIS renderer: [`LiveTwin`] holds its own [`Renderer`] and drives the window's
//! `prepare` path headless, with the same entry points, uniforms and chunk state. [`GBuffer`] is
//! what comes back: the iteration and aux planes the colour pass reads.

use crate::{GpuError, MandelbrotParams, Renderer, ITER_FORMAT};
use egui_wgpu::wgpu;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};

/// One view's G-buffer, read back: the iteration plane (`smooth_iter, normal.x, normal.y,
/// DE_log2`; `r < 0` = interior or glitched) and the aux plane, `Rgba32Float`, row-major, at the
/// texture's size (resolution × ss).
#[derive(Clone, Debug, Default)]
pub struct GBuffer {
    pub width: u32,
    pub height: u32,
    pub ss: u32,
    pub iter: Vec<f32>,
    pub aux: Vec<f32>,
}

impl GBuffer {
    /// Whether this G-buffer is a whole texture of `size` texels built at `ss`.
    pub fn fits(&self, size: [u32; 2], ss: u32) -> bool {
        let n = size[0] as usize * size[1] as usize * 4;
        [self.width, self.height] == size && self.ss == ss && self.iter.len() == n && self.aux.len() == n
    }
}

/// The live renderer on a device of its own, driven without a window: each [`Self::frame`] is one
/// frame's `prepare` (the iterate and resolve passes the params ask for), submitted and waited for.
pub struct LiveTwin {
    resources: CallbackResources,
}

impl LiveTwin {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mut resources = CallbackResources::default();
        // The target format only shapes the present/accumulate pipelines, which a twin never runs.
        resources.insert(Renderer::new(device, queue, wgpu::TextureFormat::Rgba8Unorm));
        Self { resources }
    }

    /// Prepare one frame for `p` and wait for its passes. `p` should be [`MandelbrotParams::headless`].
    pub fn frame(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, p: &MandelbrotParams) -> Result<(), GpuError> {
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("fractadyne.twin.frame") });
        let sd = ScreenDescriptor { size_in_pixels: [p.resolution[0].max(1), p.resolution[1].max(1)], pixels_per_point: 1.0 };
        let mut cbs = p.prepare(device, queue, &sd, &mut enc, &mut self.resources);
        cbs.push(enc.finish());
        queue.submit(cbs);
        crate::export::await_submitted(device, queue, None, None).into_result()
    }

    /// Read back view `view_id`'s G-buffer.
    pub fn gbuffer(&self, device: &wgpu::Device, queue: &wgpu::Queue, view_id: u32) -> Result<GBuffer, GpuError> {
        let r = self.resources.get::<Renderer>().ok_or_else(|| GpuError::Readback("the twin has no renderer".into()))?;
        r.read_gbuffer(device, queue, view_id)
    }
}

/// Read back the WINDOW's G-buffer of view `view_id` (between frames, from the UI thread): what
/// a twin's frame is compared with.
pub fn read_window_gbuffer(rs: &egui_wgpu::RenderState, view_id: u32) -> Result<GBuffer, GpuError> {
    let renderer = rs.renderer.read();
    let r = renderer
        .callback_resources
        .get::<Renderer>()
        .ok_or_else(|| GpuError::Readback("the window has no renderer".into()))?;
    r.read_gbuffer(&rs.device, &rs.queue, view_id)
}

impl Renderer {
    /// View `view_id`'s G-buffer, read back. The live textures are attachments and samplers only
    /// (no `COPY_SRC`), so they are first drawn into readable twins by the seed pipeline — the
    /// same exact `textureLoad` copy the hold snapshot makes (`hold_copy`) — and those are read.
    pub(crate) fn read_gbuffer(&self, device: &wgpu::Device, queue: &wgpu::Queue, view_id: u32) -> Result<GBuffer, GpuError> {
        let view = self
            .views
            .get(&view_id)
            .filter(|v| v.rendered)
            .ok_or_else(|| GpuError::Readback(format!("view {view_id} holds no rendered frame")))?;
        let (w, h) = (view.size[0].max(1), view.size[1].max(1));
        let extent = wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 };
        let target = |label| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: ITER_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        };
        let (ti, ta) = (target("fractadyne.gbuffer.iter"), target("fractadyne.gbuffer.aux"));
        let (vi, va) = (ti.create_view(&Default::default()), ta.create_view(&Default::default()));
        let row = w * 16; // Rgba32Float
        let bpr = row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer = |label| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: bpr as u64 * h as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let (bi, ba) = (buffer("fractadyne.gbuffer.iter_read"), buffer("fractadyne.gbuffer.aux_read"));
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("fractadyne.gbuffer.read") });
        {
            let attach = |v| {
                Some(wgpu::RenderPassColorAttachment {
                    view: v,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
                })
            };
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fractadyne.gbuffer.seed"),
                color_attachments: &[attach(&vi), attach(&va)],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.seed_pipeline);
            pass.set_bind_group(0, &view.color_bg, &[]);
            pass.draw(0..3, 0..1);
        }
        for (t, b) in [(&ti, &bi), (&ta, &ba)] {
            enc.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo { texture: t, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                wgpu::TexelCopyBufferInfo {
                    buffer: b,
                    layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(bpr), rows_per_image: Some(h) },
                },
                extent,
            );
        }
        queue.submit(std::iter::once(enc.finish()));
        let read = |b: &wgpu::Buffer| -> Result<Vec<f32>, GpuError> {
            let (tx, rx) = std::sync::mpsc::channel();
            b.slice(..).map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });
            crate::export::await_readback(device, &rx, None, None).into_result()?;
            let data = b.slice(..).get_mapped_range();
            let mut out = Vec::with_capacity(w as usize * h as usize * 4);
            for r in 0..h as usize {
                let s = r * bpr as usize;
                out.extend_from_slice(bytemuck::cast_slice(&data[s..s + row as usize]));
            }
            drop(data);
            b.unmap();
            Ok(out)
        };
        Ok(GBuffer { width: w, height: h, ss: view.last_ss.max(1), iter: read(&bi)?, aux: read(&ba)? })
    }
}

impl MandelbrotParams {
    /// This frame's params for a headless renderer ([`LiveTwin`]): every sink dropped — a twin's
    /// readings must never land in the window's — and no hold, reprojection or supersampling.
    pub fn headless(&self) -> Self {
        let mut p = self.clone();
        p.iterate_ms = None;
        p.iterate_steps = None;
        p.iterate_frame = None;
        p.iterate_armed_us = None;
        p.live_timing = false;
        p.live_ms = None;
        p.live_steps = None;
        p.live_frame = None;
        p.pass_clock = None;
        p.maxiter_count = None;
        p.norm_range = None;
        p.grad_range = None;
        p.grad_hist = None;
        p.work_counters = None;
        p.norm_sig_out = None;
        p.norm_complete_out = None;
        p.content_out = None;
        p.content_stamp_out = None;
        p.hold_copy = false;
        p.display_hold = false;
        p.reproject = 0;
        p.accum_present = false;
        p.accum_commit = false;
        p.accum_reset = false;
        p.accum_external = None;
        p.accum_folds = None;
        p.adopt = None;
        p
    }
}
