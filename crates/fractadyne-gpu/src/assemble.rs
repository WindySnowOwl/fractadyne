//! Zoom-video frames assembled from exponential-map strips (`design/zoom-video.md` §6, phase E2).
//!
//! An [`Assembler`] keeps a window of strips on the GPU — a ring of array layers, strip `k` in
//! layer `k mod layers` — and makes a frame in two passes:
//!
//! 1. **Reproject** (`assemble.wgsl`, a module of its own): fill a frame-shaped main target, the
//!    frame's `(w·ss)×(h·ss)` sample grid, from the nearest strip texel of each sample, the
//!    distance estimate rescaled from the texel's size to the sample spacing and the normal turned
//!    with the frame. The result is what an iterate pass would have written at those samples, up
//!    to where the texels sit.
//! 2. **Colour**: the app's own colour pass (`fs_color` from the unchanged iterate module), with
//!    the same uniform [`crate::color_iter_buffer_ss`] builds: every colouring that reads only the
//!    main target (smooth, distance, relief, glow), each sample shaded and the `ss×ss` averaged.
//!    The aux statistics (stripe, TIA, trap) are not in a strip.
//!
//! The geometry is [`fractadyne_core::ExpMap::locate`]'s, which the shader mirrors; the CPU keeps
//! the large part of every depth exact ([`FramePlacement`]).

use egui_wgpu::wgpu;

use crate::export::{await_readback, color_uniforms_for, ExportRequest, GpuError};

/// Mirrors `AssembleU` in `assemble.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct AssembleUniforms {
    pub(crate) res: [f32; 2],
    pub(crate) width: f32,
    pub(crate) rows: f32,
    pub(crate) rows_per_octave: f32,
    pub(crate) sample_offset: f32,
    pub(crate) depth_frac: f32,
    pub(crate) rel_base: i32,
    pub(crate) first_layer: i32,
    pub(crate) layers: i32,
    pub(crate) rel_last: i32,
    pub(crate) turns: f32,
    pub(crate) rot_cos: f32,
    pub(crate) rot_sin: f32,
    pub(crate) log2_row_step: f32,
    pub(crate) _pad: f32,
}

/// The strips' shape (from `fractadyne_core::ExpMap`): what the reprojection needs to find a texel.
#[derive(Clone, Copy, Debug)]
pub struct StripShape {
    /// Texels around a strip.
    pub width: u32,
    /// Rows in a strip.
    pub rows: u32,
    /// Rows in one octave of radius.
    pub rows_per_octave: f64,
    /// Where in its texel a strip samples (0.5: the centre).
    pub sample_offset: f64,
    /// A row's step in `ln r`.
    pub row_step: f64,
    /// The dive's strip count: strips `0..strips`.
    pub strips: u64,
}

/// Where a frame sits in the dive. `depth` is `r0_log2 − log2(sample spacing)` — how many octaves
/// inside strip 0's outer radius a point one sample from the centre is — split into whole octaves
/// and a fraction, so a frame thousands of octaves deep is placed as exactly as the first.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FramePlacement {
    pub depth_whole: i64,
    pub depth_frac: f64,
    /// The frame's rotation, in turns: the plane is drawn turned by this much clockwise (a sample at
    /// screen angle `α` shows the plane at `α + turns`).
    pub turns: f64,
}

impl FramePlacement {
    /// The placement for `depth` octaves (any real), split exactly when `depth` is an `f64` whose
    /// whole part it holds exactly (below 2^52).
    pub fn new(depth: f64, turns: f64) -> Self {
        let whole = depth.floor();
        FramePlacement { depth_whole: whole as i64, depth_frac: depth - whole, turns }
    }
}

/// `log2` of the distance, in samples, of the sample nearest a `w × h` grid's centre and of its
/// corner sample: the depths a frame reads span `depth − hi .. depth − lo`. The shader reads a
/// sample nearer than half a sample at half a sample.
pub fn sample_log2_span(w: u32, h: u32) -> (f64, f64) {
    let near = |n: u32| -> f64 { if n % 2 == 0 { 0.5 } else { 0.0 } };
    let lo = 0.5 * (near(w).powi(2) + near(h).powi(2)).max(0.25).log2();
    let (cx, cy) = (w as f64 / 2.0 - 0.5, h as f64 / 2.0 - 0.5);
    let hi = 0.5 * (cx * cx + cy * cy).max(0.25).log2();
    (lo, hi)
}

/// How far past its exact span a frame's strip range reaches, in octaves: the shader's f32
/// logarithm may put a sample right on an octave boundary on the other side of it (its error is
/// ~1e-6 of an octave), and that sample must still find its strip in the window.
const SPAN_MARGIN: f64 = 1.0e-4;

/// The strips a frame of `w × h` samples at `place` reads, `first..=last` (with [`SPAN_MARGIN`]),
/// clipped to the dive (`strips`); `None` when the frame lies wholly outside it (shallower than
/// strip 0).
pub fn strips_for_frame(place: &FramePlacement, w: u32, h: u32, strips: u64) -> Option<(u64, u64)> {
    let (lo, hi) = sample_log2_span(w, h);
    let first = place.depth_whole + (place.depth_frac - hi - SPAN_MARGIN).floor() as i64;
    let last = place.depth_whole + (place.depth_frac - lo + SPAN_MARGIN).floor() as i64;
    if last < 0 || strips == 0 {
        return None;
    }
    let last_strip = strips as i64 - 1;
    Some((first.max(0).min(last_strip) as u64, last.min(last_strip) as u64))
}

/// The array layers a window needs for frames of `w × h` samples: every strip one frame reads,
/// margins included.
pub fn layers_for_frame(w: u32, h: u32) -> u32 {
    let (lo, hi) = sample_log2_span(w, h);
    (hi - lo + 2.0 * SPAN_MARGIN).floor() as u32 + 2
}

/// The GPU memory a window of `layers` strips of `shape` takes: one 16-byte texel (four f32) each.
pub fn window_bytes(shape: StripShape, layers: u32) -> u64 {
    layers as u64 * shape.width as u64 * shape.rows as u64 * 16
}

/// Wait on a wgpu future. On native every one this module awaits (an error scope's pop) is ready
/// at once, so a poll with a no-op waker is enough and the crate needs no async runtime.
fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    let mut fut = std::pin::pin!(fut);
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    loop {
        if let std::task::Poll::Ready(v) = fut.as_mut().poll(&mut cx) {
            return v;
        }
        std::thread::yield_now();
    }
}

/// The GPU side of assembly: the strip window, the two pipelines, and the frame's targets.
pub struct Assembler {
    shape: StripShape,
    frame: [u32; 2],
    ss: u32,
    layers: u32,
    resident: Vec<Option<u64>>,
    strips_tex: wgpu::Texture,
    reproject_pipeline: wgpu::RenderPipeline,
    reproject_bg: wgpu::BindGroup,
    reproject_uniform: wgpu::Buffer,
    main_tex: wgpu::Texture,
    main_view: wgpu::TextureView,
    color_pipeline: wgpu::RenderPipeline,
    color_bg: wgpu::BindGroup,
    color_uniform: wgpu::Buffer,
    lut_buf: wgpu::Buffer,
    out_tex: wgpu::Texture,
    out_view: wgpu::TextureView,
}

const OUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

impl Assembler {
    /// An assembler for frames of `frame` pixels at `ss×ss` samples a pixel, from strips of
    /// `shape`, with a window of `layers` strips (at least [`layers_for_frame`] of the sample grid).
    pub fn new(device: &wgpu::Device, shape: StripShape, frame: [u32; 2], ss: u32, layers: u32) -> Result<Self, GpuError> {
        let ss = ss.max(1);
        let (sw, sh) = (frame[0] * ss, frame[1] * ss);
        let limits = device.limits();
        let need = layers_for_frame(sw, sh);
        if layers < need {
            return Err(GpuError::Assembly(format!("{layers} strip layers cannot hold a frame's {need}")));
        }
        if shape.width > limits.max_texture_dimension_2d || shape.rows > limits.max_texture_dimension_2d {
            return Err(GpuError::Assembly(format!(
                "a {}x{} strip is larger than this GPU's {} texel textures",
                shape.width, shape.rows, limits.max_texture_dimension_2d
            )));
        }
        if layers > limits.max_texture_array_layers {
            return Err(GpuError::Assembly(format!("{layers} strip layers exceed this GPU's {}", limits.max_texture_array_layers)));
        }
        if sw > limits.max_texture_dimension_2d || sh > limits.max_texture_dimension_2d {
            return Err(GpuError::Assembly(format!("a {sw}x{sh} sample grid is larger than this GPU's textures")));
        }
        // ⭐The window is the one large allocation a zoom video makes — gigabytes at 4K or with
        // anti-aliasing — and a card without room for it must say so as an error. Outside an error
        // scope the failure reaches the app's uncaptured-error handler, which panics (measured: 3840x2160
        // at 2x, 13 strips of 27684x3058 ≈ 17.6 GB on a 10 GB card, a crash report one second in).
        device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        device.push_error_scope(wgpu::ErrorFilter::Validation);
        let made = Self::build(device, shape, frame, ss, layers);
        let invalid = block_on(device.pop_error_scope());
        let oom = block_on(device.pop_error_scope());
        match oom.or(invalid) {
            None => Ok(made),
            Some(e) => Err(GpuError::Assembly(format!(
                "the GPU could not hold the strip window ({layers} strips of {}x{}, {:.1} GB) — lower the size or the \
                 anti-aliasing: {e}",
                shape.width,
                shape.rows,
                window_bytes(shape, layers) as f64 / 1e9,
            ))),
        }
    }

    /// [`Self::new`]'s allocations, once its checks have passed (inside its error scopes).
    fn build(device: &wgpu::Device, shape: StripShape, frame: [u32; 2], ss: u32, layers: u32) -> Self {
        let (sw, sh) = (frame[0] * ss, frame[1] * ss);
        let strips_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("assemble.strips"),
            size: wgpu::Extent3d { width: shape.width, height: shape.rows, depth_or_array_layers: layers },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: crate::ITER_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let strips_view = strips_tex.create_view(&wgpu::TextureViewDescriptor {
            label: Some("assemble.strips.view"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        // Pass 1: reprojection, its own module.
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fractadyne.assemble"),
            source: wgpu::ShaderSource::Wgsl(include_str!("assemble.wgsl").into()),
        });
        let reproject_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("assemble.reproject.bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let reproject_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("assemble.reproject.layout"),
            bind_group_layouts: &[&reproject_bgl],
            push_constant_ranges: &[],
        });
        let reproject_pipeline = crate::fullscreen_pipeline(
            device,
            &module,
            &reproject_layout,
            "fs_assemble",
            &[crate::ITER_FORMAT],
            "assemble.reproject.pipeline",
        );
        let reproject_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("assemble.reproject.uniform"),
            size: std::mem::size_of::<AssembleUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let reproject_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("assemble.reproject.bg"),
            layout: &reproject_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: reproject_uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&strips_view) },
            ],
        });

        // The frame-shaped main target the reprojection writes and the colour pass reads, and a
        // zeroed aux beside it (the colourings a strip supports read no aux).
        let grid = |label, usage| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d { width: sw, height: sh, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: crate::ITER_FORMAT,
                usage,
                view_formats: &[],
            })
        };
        let main_tex = grid(
            "assemble.main",
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
        );
        let main_view = main_tex.create_view(&wgpu::TextureViewDescriptor::default());
        // Created zeroed (wgpu zero-initialises a texture's first use).
        let aux_tex = grid("assemble.aux", wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST);
        let aux_view = aux_tex.create_view(&wgpu::TextureViewDescriptor::default());

        // Pass 2: the app's colour pass, from the unchanged iterate module.
        let shader = crate::shader_module(device);
        let color_bgl = crate::color_bind_group_layout(device);
        let color_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("assemble.color.layout"),
            bind_group_layouts: &[&color_bgl],
            push_constant_ranges: &[],
        });
        let color_pipeline =
            crate::fullscreen_pipeline(device, &shader, &color_layout, "fs_color", &[OUT_FORMAT], "assemble.color.pipeline");
        let color_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("assemble.color.uniform"),
            size: std::mem::size_of::<crate::ColorUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let lut_buf = crate::make_lut_buffer(device);
        let color_bg = crate::make_color_bg(device, &color_bgl, &color_uniform, &lut_buf, &main_view, &aux_view);
        let out_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("assemble.out"),
            size: wgpu::Extent3d { width: frame[0], height: frame[1], depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: OUT_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let out_view = out_tex.create_view(&wgpu::TextureViewDescriptor::default());
        Assembler {
            shape,
            frame,
            ss,
            layers,
            resident: vec![None; layers as usize],
            strips_tex,
            reproject_pipeline,
            reproject_bg,
            reproject_uniform,
            main_tex,
            main_view,
            color_pipeline,
            color_bg,
            color_uniform,
            lut_buf,
            out_tex,
            out_view,
        }
    }

    /// The window's layers.
    pub fn layers(&self) -> u32 {
        self.layers
    }

    /// Whether strip `k` is in the window.
    pub fn has_strip(&self, k: u64) -> bool {
        self.resident[(k % self.layers as u64) as usize] == Some(k)
    }

    /// Put strip `k` into the window (layer `k mod layers`, replacing whatever was there): its main
    /// target, `width × rows` texels of `(smooth count, normal x, y, log2 DE in texels)`.
    pub fn put_strip(&mut self, queue: &wgpu::Queue, k: u64, main: &[f32]) -> Result<(), GpuError> {
        let (w, h) = (self.shape.width, self.shape.rows);
        if main.len() != w as usize * h as usize * 4 {
            return Err(GpuError::Assembly(format!("strip {k} has {} values, not {w}x{h}x4", main.len())));
        }
        let layer = (k % self.layers as u64) as u32;
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.strips_tex,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: layer },
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(main),
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(w * 16), rows_per_image: Some(h) },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        self.resident[layer as usize] = Some(k);
        Ok(())
    }

    /// The strips a frame at `place` reads (`first..=last`), clipped to the dive.
    pub fn strips_for(&self, place: &FramePlacement) -> Option<(u64, u64)> {
        strips_for_frame(place, self.frame[0] * self.ss, self.frame[1] * self.ss, self.shape.strips)
    }

    /// The reprojection's uniform for a frame at `place`, after checking its strips are all in.
    fn uniforms(&self, place: &FramePlacement) -> Result<AssembleUniforms, GpuError> {
        let (first, last) = self
            .strips_for(place)
            .ok_or_else(|| GpuError::Assembly("the frame is shallower than the dive's first strip".into()))?;
        let missing: Vec<u64> = (first..=last).filter(|&k| !self.has_strip(k)).collect();
        if !missing.is_empty() {
            return Err(GpuError::Assembly(format!("strips {missing:?} of {first}-{last} are not in the window")));
        }
        let rel_base = place.depth_whole - first as i64;
        // Deeper samples clamp to the window's last strip: the dive's last, or (only for a sample
        // the margin did not reach, which it reaches) the deepest one this frame put in the window
        // — never a layer that may hold another strip.
        let rel_last = last as i64 - first as i64;
        let phi = place.turns * std::f64::consts::TAU;
        Ok(AssembleUniforms {
            res: [(self.frame[0] * self.ss) as f32, (self.frame[1] * self.ss) as f32],
            width: self.shape.width as f32,
            rows: self.shape.rows as f32,
            rows_per_octave: self.shape.rows_per_octave as f32,
            sample_offset: self.shape.sample_offset as f32,
            depth_frac: place.depth_frac as f32,
            rel_base: rel_base.clamp(i32::MIN as i64, i32::MAX as i64) as i32,
            first_layer: (first % self.layers as u64) as i32,
            layers: self.layers as i32,
            rel_last: rel_last.clamp(0, i32::MAX as i64) as i32,
            turns: place.turns.rem_euclid(1.0) as f32,
            rot_cos: phi.cos() as f32,
            rot_sin: phi.sin() as f32,
            log2_row_step: self.shape.row_step.log2() as f32,
            _pad: 0.0,
        })
    }

    fn encode_reproject(&self, enc: &mut wgpu::CommandEncoder) {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("assemble.reproject.pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.main_view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&self.reproject_pipeline);
        pass.set_bind_group(0, &self.reproject_bg, &[]);
        pass.draw(0..3, 0..1);
    }

    /// The frame at `place`, coloured with `colour`'s colouring: `w × h` RGBA display values,
    /// row-major, as an export's pixels. Every strip the frame reads must be in the window.
    pub fn frame(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, place: &FramePlacement, colour: &ExportRequest) -> Result<Vec<f32>, GpuError> {
        let u = self.uniforms(place)?;
        queue.write_buffer(&self.reproject_uniform, 0, bytemuck::bytes_of(&u));
        let (lut_len, lut_smooth) = crate::write_lut(queue, &self.lut_buf, &colour.lut, colour.lut_smooth);
        let cu = color_uniforms_for(colour, self.ss, self.frame[0], self.frame[1], lut_len, lut_smooth);
        queue.write_buffer(&self.color_uniform, 0, bytemuck::bytes_of(&cu));
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("assemble.frame") });
        self.encode_reproject(&mut enc);
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("assemble.color.pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.out_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.color_pipeline);
            pass.set_bind_group(0, &self.color_bg, &[]);
            pass.draw(0..3, 0..1);
        }
        read_texture(device, queue, enc, &self.out_tex, self.frame[0], self.frame[1])
    }

    /// The frame's main target only (no colouring): `(w·ss) × (h·ss)` samples of `(smooth count,
    /// normal x, y, log2 DE in samples)`, as an iterate pass of the same frame writes them. For
    /// checking assembly against flat renders.
    pub fn reprojected(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, place: &FramePlacement) -> Result<Vec<f32>, GpuError> {
        let u = self.uniforms(place)?;
        queue.write_buffer(&self.reproject_uniform, 0, bytemuck::bytes_of(&u));
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("assemble.reprojected") });
        self.encode_reproject(&mut enc);
        read_texture(device, queue, enc, &self.main_tex, self.frame[0] * self.ss, self.frame[1] * self.ss)
    }
}

/// Finish `enc` with a copy of `tex` (`w × h`, 16 bytes a texel) to a buffer, submit, and read it
/// back as `w·h·4` floats, row padding removed.
fn read_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mut enc: wgpu::CommandEncoder,
    tex: &wgpu::Texture,
    w: u32,
    h: u32,
) -> Result<Vec<f32>, GpuError> {
    let unpadded = w * 16;
    let padded = unpadded.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("assemble.readback"),
        size: (padded * h) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: Some(h) },
        },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    queue.submit(std::iter::once(enc.finish()));
    let slice = buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    await_readback(device, &rx, None, None).into_result()?;
    let data = slice.get_mapped_range();
    let mut out = vec![0.0f32; w as usize * h as usize * 4];
    let row = (w * 4) as usize;
    for r in 0..h as usize {
        let src = r * padded as usize;
        out[r * row..(r + 1) * row].copy_from_slice(bytemuck::cast_slice(&data[src..src + unpadded as usize]));
    }
    drop(data);
    buf.unmap();
    Ok(out)
}

#[cfg(test)]
#[path = "../tests/unit/assemble/tests.rs"]
mod tests;
