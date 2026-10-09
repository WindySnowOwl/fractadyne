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
/// texture's size (resolution × ss). `aux` is empty when it was not read: the colour pass reads it
/// only for the orbit-statistics methods (`method_needs_aux`), and a frame installed without it
/// gets a zero plane.
#[derive(Clone, Debug, Default)]
pub struct GBuffer {
    pub width: u32,
    pub height: u32,
    pub ss: u32,
    pub iter: Vec<f32>,
    pub aux: Vec<f32>,
}

impl GBuffer {
    /// Whether the planes fill the texture (`aux` may be empty).
    pub fn whole(&self) -> bool {
        let n = self.width as usize * self.height as usize * 4;
        n > 0 && self.iter.len() == n && (self.aux.is_empty() || self.aux.len() == n)
    }

    /// Whether this G-buffer is a whole texture of `size` texels built at `ss`.
    pub fn fits(&self, size: [u32; 2], ss: u32) -> bool {
        self.whole() && [self.width, self.height] == size && self.ss == ss
    }

    /// Whether anything was drawn: a cleared texture reads 0 everywhere, and no pixel of a
    /// perturbation view escapes at iteration 0 (interior reads negative).
    pub fn drawn(&self) -> bool {
        self.iter.chunks(4).any(|t| t[0] != 0.0)
    }
}

/// A G-buffer handed to the device that will show it, ready to install (`MandelbrotParams::adopt`).
/// [`Self::upload`] runs on any thread and touches no queue: the planes are written into buffers
/// mapped at creation, and the device copies them into its textures inside the frame that adopts
/// them ([`Self::stage`]).
///
/// ⛔⭐Not `queue.write_texture`. wgpu-core 24 holds the device's `pending_writes` lock for the whole
/// of a `write_texture`, its staging copy included, and every `queue.submit` takes that lock too: a
/// worker thread writing a frame stalled the window's next present for as long as the copy took. On
/// PLUTO with the RTX 3070 drawing the window, ~150 frames a run went over 33 ms, almost all of them
/// the frame before an adoption (2026-10-08); with the RX 6800 XT drawing it, 1–3.
#[derive(Debug)]
pub struct AdoptFrame {
    pub width: u32,
    pub height: u32,
    pub ss: u32,
    pub(crate) iter: wgpu::TextureView,
    /// A 1×1 zero texture when the G-buffer had no aux plane: every read of it is 0.
    pub(crate) aux: wgpu::TextureView,
    /// The written buffers and the textures they go to, until the adopting frame copies them.
    pending: std::sync::Mutex<Vec<(wgpu::Buffer, wgpu::Texture)>>,
    bytes_per_row: u32,
}

impl AdoptFrame {
    /// Hand `g` to `device` (`None` when its planes do not fill its size).
    pub fn upload(device: &wgpu::Device, g: &GBuffer) -> Option<Self> {
        if !g.whole() {
            return None;
        }
        let (w, h) = (g.width, g.height);
        let row = w * 16; // Rgba32Float
        let bpr = row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let texture = |tw: u32, th: u32, label| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d { width: tw, height: th, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: ITER_FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            })
        };
        let written = |data: &[f32], label| {
            let b = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: bpr as u64 * h as u64,
                usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: true,
            });
            {
                let src: &[u8] = bytemuck::cast_slice(data);
                let mut dst = b.slice(..).get_mapped_range_mut();
                for r in 0..h as usize {
                    let (s, d) = (r * row as usize, r * bpr as usize);
                    dst[d..d + row as usize].copy_from_slice(&src[s..s + row as usize]);
                }
            }
            b.unmap();
            b
        };
        let iter_t = texture(w, h, "fractadyne.adopt_iter");
        let mut pending = vec![(written(&g.iter, "fractadyne.adopt_iter_upload"), iter_t.clone())];
        let aux_t = if g.aux.is_empty() {
            texture(1, 1, "fractadyne.adopt_aux_zero")
        } else {
            let t = texture(w, h, "fractadyne.adopt_aux");
            pending.push((written(&g.aux, "fractadyne.adopt_aux_upload"), t.clone()));
            t
        };
        Some(Self {
            width: w,
            height: h,
            ss: g.ss.max(1),
            iter: iter_t.create_view(&wgpu::TextureViewDescriptor::default()),
            aux: aux_t.create_view(&wgpu::TextureViewDescriptor::default()),
            pending: std::sync::Mutex::new(pending),
            bytes_per_row: bpr,
        })
    }

    /// Copy the written planes into the textures, in the adopting frame's encoder (once).
    pub(crate) fn stage(&self, encoder: &mut wgpu::CommandEncoder) {
        let pending = self.pending.lock().map(|mut p| std::mem::take(&mut *p)).unwrap_or_default();
        for (b, t) in &pending {
            encoder.copy_buffer_to_texture(
                wgpu::TexelCopyBufferInfo {
                    buffer: b,
                    layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(self.bytes_per_row), rows_per_image: Some(self.height) },
                },
                wgpu::TexelCopyTextureInfo { texture: t, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                wgpu::Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
            );
        }
    }

    /// Whether this frame is a whole texture of `size` texels built at `ss`.
    pub fn fits(&self, size: [u32; 2], ss: u32) -> bool {
        [self.width, self.height] == size && self.ss == ss
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

    /// Render `base` (a whole frame: [`MandelbrotParams::headless`], no tile, chunk window, split
    /// or reprojection) as a walk of chunk windows priced by `pricer`, as the live view's chunked
    /// refreshes are (the chunked iterate is bit-identical for any windows). `cancel` is asked
    /// before every pass. A device or formula that cannot walk renders one pass, and only when
    /// that pass fits the opening budget.
    pub fn walk(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        base: &MandelbrotParams,
        pricer: &mut WalkPricer,
        cancel: &dyn Fn() -> bool,
    ) -> Result<WalkStats, GpuError> {
        let t0 = std::time::Instant::now();
        let ss = base.ss.max(1) as u64;
        let area = base.resolution[0].max(1) as u64 * base.resolution[1].max(1) as u64 * ss * ss;
        let max_iter = base.max_iter.max(1);
        let walks = if base.mode == 2 { crate::chunking_mode2_available(device) } else { crate::chunking_available(device) }
            && base.custom.as_ref().is_none_or(|c| c.resumable);
        let mut stats = WalkStats::default();
        if !walks {
            if area as f64 * max_iter as f64 > pricer.open_px_steps() {
                return Err(GpuError::Readback(format!(
                    "{area} samples × {max_iter} iterations is too much for one pass, and this frame cannot walk"
                )));
            }
            let mut p = base.clone();
            p.chunk_range = None;
            self.frame(device, queue, &p)?;
            stats.passes = 1;
            stats.ms = t0.elapsed().as_secs_f64() * 1000.0;
            return Ok(stats);
        }
        let mut start = 0u32;
        let mut window = pricer.open(area);
        loop {
            if cancel() {
                return Err(GpuError::Canceled);
            }
            // The first window starts at 0 and the series seeds [0, sa_skip) in one evaluation, so
            // it loops `window` real iterations past the skip.
            let skip = if stats.passes == 0 { base.sa_skip.min(max_iter) } else { start };
            let end = skip.saturating_add(window).min(max_iter);
            let mut p = base.clone();
            p.chunk_range = Some([start, end]);
            p.chunk_idx = stats.passes;
            let tp = std::time::Instant::now();
            self.frame(device, queue, &p)?;
            let ms = tp.elapsed().as_secs_f64() * 1000.0;
            let looped = end - skip;
            if stats.passes == 0 {
                pricer.observe_open(area, looped, ms);
            }
            stats.passes += 1;
            stats.max_pass_ms = stats.max_pass_ms.max(ms);
            start = end;
            if start >= max_iter {
                break;
            }
            window = pricer.next(area, looped.max(1), ms);
        }
        stats.ms = t0.elapsed().as_secs_f64() * 1000.0;
        Ok(stats)
    }

    /// Read back view `view_id`'s G-buffer; the aux plane only `with_aux`.
    pub fn gbuffer(&self, device: &wgpu::Device, queue: &wgpu::Queue, view_id: u32, with_aux: bool) -> Result<GBuffer, GpuError> {
        let r = self.resources.get::<Renderer>().ok_or_else(|| GpuError::Readback("the twin has no renderer".into()))?;
        r.read_gbuffer(device, queue, view_id, with_aux)
    }
}

/// What one [`LiveTwin::walk`] took.
#[derive(Clone, Copy, Debug, Default)]
pub struct WalkStats {
    pub passes: u32,
    pub ms: f64,
    pub max_pass_ms: f64,
}

/// A walk pass is sized for this wall by default ([`WalkPricer::with_pass_ms`] sets another): half
/// a 60 Hz frame. One pass is one draw, which the GPU does not interrupt, so on a card that also
/// composites the desktop a long pass holds up every window's present: with the RX 6800 XT as the
/// worker and the RTX 3070 drawing the window (PLUTO, 2026-10-08), 40 ms passes put 62–162 frames
/// a `--zoomtest` over 33 ms, 16 ms and 8 ms passes 7–9 (1 alone), at the same worker frame rate.
pub const WALK_PASS_MS: f64 = 8.0;
/// The cost assumed for an opening pass before any is measured, ns per pixel-step: the export's
/// unobserved prior (`STEP_PRIOR_NS`), above the worst the RTX 3080 has shown.
const WALK_PRIOR_NS: f64 = 1.5;
/// No opening pass may exceed this many pixel-steps, whatever was measured (~1.5 s at the prior,
/// 0.8 s at the worst cost measured): the bound on a view far costlier than every one before it.
const WALK_OPEN_MAX_PX_STEPS: f64 = 1.0e9;
/// No later pass may exceed this many NOMINAL pixel-steps (the frame × its window, running or
/// not): `EXPLICIT_STEPS_CEIL`, the live view's own cap on a settled chunk pass.
const WALK_MAX_PX_STEPS: f64 = 6.0e10;
/// A window never shrinks below this: a bounded pass count.
const WALK_MIN_ITERS: u32 = 256;
/// A window at most doubles from one pass to the next.
const WALK_GROW: f64 = 2.0;

/// Pass sizing for a worker's walks. A walk's OPENING pass has every pixel running, so its cost
/// per pixel-step prices the next walk's opening; later passes are priced from the one before
/// within the walk (`window × target / wall`), because pixels only stop running as a walk goes on,
/// so a pass costs no more than the last one did per iteration — and they are NOT carried to the
/// next opening, which they would underprice by the fraction that had escaped.
#[derive(Clone, Debug)]
pub struct WalkPricer {
    /// ns per pixel-step of the opening passes: rises at once, falls a fifth of the way per walk.
    open_ns: f64,
    /// The wall a pass is sized for, ms.
    pass_ms: f64,
}

impl Default for WalkPricer {
    fn default() -> Self {
        Self { open_ns: WALK_PRIOR_NS, pass_ms: WALK_PASS_MS }
    }
}

impl WalkPricer {
    /// Size passes for `ms` (at least 1) from here on.
    pub fn with_pass_ms(mut self, ms: f64) -> Self {
        self.set_pass_ms(ms);
        self
    }

    pub fn set_pass_ms(&mut self, ms: f64) {
        self.pass_ms = if ms.is_finite() { ms.max(1.0) } else { WALK_PASS_MS };
    }

    /// Pixel-steps an opening pass may take.
    pub fn open_px_steps(&self) -> f64 {
        (self.pass_ms * 1.0e6 / self.open_ns).min(WALK_OPEN_MAX_PX_STEPS)
    }

    /// The opening window, iterations, for `area` samples.
    pub fn open(&self, area: u64) -> u32 {
        (self.open_px_steps() / area.max(1) as f64).clamp(WALK_MIN_ITERS as f64, u32::MAX as f64) as u32
    }

    /// An opening pass over `area` samples looped `iters` iterations in `wall_ms`.
    pub fn observe_open(&mut self, area: u64, iters: u32, wall_ms: f64) {
        if !wall_ms.is_finite() || wall_ms <= 0.0 || iters == 0 {
            return;
        }
        let ns = wall_ms * 1.0e6 / (area.max(1) as f64 * iters as f64);
        self.open_ns = if ns >= self.open_ns { ns } else { self.open_ns + 0.2 * (ns - self.open_ns) };
    }

    /// The window after a pass of `window` iterations over `area` samples that took `wall_ms`.
    pub fn next(&self, area: u64, window: u32, wall_ms: f64) -> u32 {
        let scaled = if wall_ms.is_finite() && wall_ms > 0.0 {
            window as f64 * (self.pass_ms / wall_ms).min(WALK_GROW)
        } else {
            window as f64
        };
        let ceiling = WALK_MAX_PX_STEPS / area.max(1) as f64;
        scaled.min(ceiling).clamp(WALK_MIN_ITERS as f64, u32::MAX as f64) as u32
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
    r.read_gbuffer(&rs.device, &rs.queue, view_id, true)
}

impl Renderer {
    /// View `view_id`'s G-buffer, read back (the aux plane only `with_aux`). The live textures are
    /// attachments and samplers only (no `COPY_SRC`), so they are first drawn into readable twins
    /// by the seed pipeline — the same exact `textureLoad` copy the hold snapshot makes
    /// (`hold_copy`) — and those are read.
    pub(crate) fn read_gbuffer(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view_id: u32,
        with_aux: bool,
    ) -> Result<GBuffer, GpuError> {
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
        let planes: &[(&wgpu::Texture, &wgpu::Buffer)] = if with_aux { &[(&ti, &bi), (&ta, &ba)] } else { &[(&ti, &bi)] };
        for (t, b) in planes {
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
        let iter = read(&bi)?;
        let aux = if with_aux { read(&ba)? } else { Vec::new() };
        Ok(GBuffer { width: w, height: h, ss: view.last_ss.max(1), iter, aux })
    }
}

impl MandelbrotParams {
    /// This frame's params for a headless renderer ([`LiveTwin`]): every sink dropped — a twin's
    /// readings must never land in the window's — and no hold, reprojection, supersampling or
    /// adoption.
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
        p.adopt_hold = false;
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_opening_pass_prices_the_next_opening_at_once_and_forgets_slowly() {
        let mut p = WalkPricer::default();
        let area = 4_000u64;
        let budget = WALK_PASS_MS * 1.0e6;
        // Unmeasured: the prior, one pass's wall at 1.5 ns a pixel-step.
        assert_eq!(p.open(area), (budget / 1.5 / area as f64) as u32);
        // Measured dearer: the next opening takes the dearer price at once.
        p.observe_open(area, 100, 3.0 * 100.0 * area as f64 / 1.0e6);
        assert_eq!(p.open(area), (budget / 3.0 / area as f64) as u32);
        // Cheaper: a fifth of the way per walk, never the whole way at once.
        p.observe_open(area, 100, 1.0 * 100.0 * area as f64 / 1.0e6);
        let ns = 3.0 + 0.2 * (1.0 - 3.0);
        assert_eq!(p.open(area), (budget / ns / area as f64) as u32);
        // A cheap card's opening is bounded by pixel-steps; a big frame's is floored at 256.
        assert_eq!(WalkPricer { open_ns: 1.0e-6, pass_ms: WALK_PASS_MS }.open(1), WALK_OPEN_MAX_PX_STEPS as u32);
        assert_eq!(WalkPricer::default().open(4_000_000), WALK_MIN_ITERS);
    }

    #[test]
    fn a_window_follows_its_wall_grows_at_most_twofold_and_stays_under_the_ceiling() {
        let p = WalkPricer::default();
        let area = 400_000u64;
        // On target: unchanged. Hot: shrinks in proportion. Cheap: at most doubles.
        assert_eq!(p.next(area, 4000, WALK_PASS_MS), 4000);
        assert_eq!(p.next(area, 4000, 4.0 * WALK_PASS_MS), 1000);
        assert_eq!(p.next(area, 4000, 1.0), 8000);
        // Never below the floor, never above the nominal ceiling.
        assert_eq!(p.next(area, 300, 100.0 * WALK_PASS_MS), WALK_MIN_ITERS);
        assert_eq!(p.next(area, 140_000, 1.0), (WALK_MAX_PX_STEPS / area as f64) as u32);
        // A garbage wall keeps the window.
        assert_eq!(p.next(area, 1000, f64::NAN), 1000);
        // A shorter pass target sizes both the opening and the steps for it.
        let short = WalkPricer::default().with_pass_ms(10.0);
        assert_eq!(short.open(4_000), (10.0e6 / 1.5 / 4_000.0) as u32);
        assert_eq!(short.next(area, 4000, 10.0), 4000);
        assert_eq!(short.next(area, 4000, 40.0), 1000);
    }
}
