//! GPU rendering for Fractadyne.
//!
//! Two-pass perturbation: an *iteration pass* renders the smooth escape value per
//! pixel as a δz perturbation of a **reference orbit** (computed on the CPU at
//! arbitrary precision by the app and handed in here), into an offscreen `R32Float`
//! texture; a *coloring pass* samples it and applies the palette (with SSAA).
//!
//! This crate stays pure f32/df64 — all high-precision work happens in the app/core.
//! The reference orbit arrives as an `Arc<Vec<[f32;4]>>` and is uploaded only when
//! its `orbit_id` changes (the app caches it across frames).

use egui_wgpu::wgpu;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};
use std::sync::Arc;

// RGBA: r = smooth iteration value, g/b = slope normal (x,y), a = reserved (DE).
const ITER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct IterUniforms {
    step: [f32; 4],       // per-texel complex step, df32: (x_hi, x_lo, y_hi, y_lo)
    ref_offset: [f32; 4], // (center − reference) df32: (re_hi, im_hi, re_lo, im_lo)
    center: [f32; 4],     // view center df32 (direct mode)
    julia_c: [f32; 4],    // Julia parameter df32 (re_hi, im_hi, re_lo, im_lo)
    res: [f32; 2],        // FULL iteration resolution (across all export tiles)
    px_offset: [f32; 2],  // this tile's top-left texel in the full grid (0 for live)
    max_iter: u32,
    orbit_len: u32,
    mode: u32,    // 0 = perturbation, 1 = direct df32
    formula: u32, // escape-time formula id
    julia: u32,   // 0 = Mandelbrot mode, 1 = Julia mode
    delta_exp: i32, // shared base-2 exponent of the δ mantissas (step / ref_offset)
    color_method: u32, // selected coloring method (drives aux accumulation)
    stripe_freq: f32,  // stripe-average angular frequency
    trap_type: u32,    // orbit-trap shape: 0 point, 1 cross, 2 circle
    aux_on: u32,       // 1 = accumulate orbit statistics into the aux target
    sa_skip: u32,      // series-approximation skip (0 = none): seed δz at this iteration
    glitch_on: u32,    // 1 = flag Pauldelbrot-glitched pixels (multi-reference correction pass)
    sa_a: [f32; 4],    // order-3 series coeffs (complex df32 mantissa): δz ≈ A·δc + B·δc² + C·δc³
    sa_b: [f32; 4],
    sa_c: [f32; 4],
    sa_a_exp: i32,     // per-coefficient base-2 exponents (floatexp)
    sa_b_exp: i32,
    sa_c_exp: i32,
    bla_on: u32,       // 1 = use the BLA tree (appended after the orbit at index orbit_len)
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ColorUniforms {
    stop_count: u32,
    cycle: f32,
    offset: f32,
    ss: u32,
    light: u32,         // 0 = off, 1 = slope/relief lighting from the stored normal
    light_angle: f32,   // light direction (radians)
    light_height: f32,  // relief strength (smaller = stronger; ~1.5 default)
    de_on: u32,         // 0 = off, 1 = distance-estimate glow
    de_strength: f32,
    de_width: f32,
    de_phase: f32,
    color_method: u32,
    aa_filter: u32,
    reproject: u32,      // 1 = reprojection: sample the frozen iteration texture scaled+translated
    uv_offset: [f32; 2], // uv translation applied when reproject == 1
    uv_scale: f32,       // uv scale about centre (1 = pan only; <1 = zoomed since the frozen frame)
    _pad_uv: [f32; 3],
    interior_col: [f32; 4],
    stops: [[f32; 4]; 8],
}

/// When the iteration pass must re-run.
#[derive(Clone, Copy, PartialEq)]
struct IterKey {
    ref_offset: [f32; 4],
    center: [f32; 4],
    julia_c: [f32; 4],
    span_mantissa: [f64; 2],
    max_iter: u32,
    size: [u32; 2],
    orbit_id: u64,
    mode: u32,
    formula: u32,
    julia: u32,
    delta_exp: i32,
    color_method: u32,
    stripe_freq: f32,
    trap_type: u32,
    sa_skip: u32,
    bla_on: u32,
}

/// Per-view GPU resources (one set per on-screen panel, keyed by `view_id`). Each
/// view owns its offscreen iteration texture, uniforms, orbit buffer, and caching
/// state so two panels (e.g. Mandelbrot + Julia) can render without clobbering.
struct ViewResources {
    iter_uniform: wgpu::Buffer,
    color_uniform: wgpu::Buffer,
    iter_bg: wgpu::BindGroup,
    orbit_buf: wgpu::Buffer,
    orbit_cap: u32,
    color_bg: wgpu::BindGroup,
    tex_view: wgpu::TextureView,
    aux_view: wgpu::TextureView,
    size: [u32; 2],
    /// Supersampling factor the current iteration texture was rendered at. Needed so a pan
    /// reprojection colors the frozen texture with the ss it was built with (not the live ss).
    last_ss: u32,
    /// The iteration texture holds a real render (set after the first iterate, cleared on resize).
    /// Reprojection needs this rather than `last_iter_key.is_some()`: the key is cleared whenever a
    /// new orbit is uploaded, but the texture still holds the last good frame — so a freeze can
    /// keep showing it instead of falling through and re-iterating a stale reference (black frame).
    rendered: bool,
    last_orbit_id: u64,
    last_iter_key: Option<IterKey>,
}

struct Renderer {
    iter_pipeline: wgpu::RenderPipeline,
    color_pipeline: wgpu::RenderPipeline,
    iter_bgl: wgpu::BindGroupLayout,
    color_bgl: wgpu::BindGroupLayout,
    views: std::collections::HashMap<u32, ViewResources>,
}

fn make_iter_texture(device: &wgpu::Device, size: [u32; 2]) -> wgpu::TextureView {
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fractadyne.iter_tex"),
        size: wgpu::Extent3d {
            width: size[0].max(1),
            height: size[1].max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: ITER_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    tex.create_view(&wgpu::TextureViewDescriptor::default())
}

fn make_orbit_buffer(device: &wgpu::Device, capacity: u32) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("fractadyne.orbit_buf"),
        size: (capacity.max(1) as u64) * 16, // df64 vec4<f32> = 16 bytes
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn make_iter_bg(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    iter_uniform: &wgpu::Buffer,
    orbit_buf: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("fractadyne.iter_bg"),
        layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: iter_uniform.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: orbit_buf.as_entire_binding() },
        ],
    })
}

fn make_color_bg(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    color_uniform: &wgpu::Buffer,
    tex_view: &wgpu::TextureView,
    aux_view: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("fractadyne.color_bg"),
        layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: color_uniform.as_entire_binding() },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(tex_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(aux_view),
            },
        ],
    })
}

fn fullscreen_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    fs_entry: &str,
    formats: &[wgpu::TextureFormat],
    label: &str,
) -> wgpu::RenderPipeline {
    let targets: Vec<Option<wgpu::ColorTargetState>> = formats
        .iter()
        .map(|&format| {
            Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })
        })
        .collect();
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(fs_entry),
            targets: &targets,
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

impl Renderer {
    fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fractadyne.shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("mandelbrot.wgsl").into()),
        });

        let uniform_entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let iter_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fractadyne.iter_bgl"),
            entries: &[
                uniform_entry,
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let tex_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let color_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fractadyne.color_bgl"),
            entries: &[uniform_entry, tex_entry(1), tex_entry(2)],
        });

        let iter_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fractadyne.iter_layout"),
            bind_group_layouts: &[&iter_bgl],
            push_constant_ranges: &[],
        });
        let color_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fractadyne.color_layout"),
            bind_group_layouts: &[&color_bgl],
            push_constant_ranges: &[],
        });

        let iter_pipeline = fullscreen_pipeline(
            device, &shader, &iter_layout, "fs_iterate", &[ITER_FORMAT, ITER_FORMAT],
            "fractadyne.iter_pipeline",
        );
        let color_pipeline = fullscreen_pipeline(
            device, &shader, &color_layout, "fs_color", &[target_format],
            "fractadyne.color_pipeline",
        );

        Self {
            iter_pipeline,
            color_pipeline,
            iter_bgl,
            color_bgl,
            views: std::collections::HashMap::new(),
        }
    }
}

impl ViewResources {
    fn new(
        device: &wgpu::Device,
        iter_bgl: &wgpu::BindGroupLayout,
        color_bgl: &wgpu::BindGroupLayout,
    ) -> Self {
        let uniform = |label, size| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let iter_uniform =
            uniform("fractadyne.iter_uniform", std::mem::size_of::<IterUniforms>() as u64);
        let color_uniform =
            uniform("fractadyne.color_uniform", std::mem::size_of::<ColorUniforms>() as u64);

        let orbit_cap = 1024u32;
        let orbit_buf = make_orbit_buffer(device, orbit_cap);
        let iter_bg = make_iter_bg(device, iter_bgl, &iter_uniform, &orbit_buf);

        let size = [1u32, 1u32];
        let tex_view = make_iter_texture(device, size);
        let aux_view = make_iter_texture(device, size);
        let color_bg =
            make_color_bg(device, color_bgl, &color_uniform, &tex_view, &aux_view);

        Self {
            iter_uniform,
            color_uniform,
            iter_bg,
            orbit_buf,
            orbit_cap,
            color_bg,
            tex_view,
            aux_view,
            size,
            last_ss: 1,
            rendered: false,
            last_orbit_id: u64::MAX,
            last_iter_key: None,
        }
    }

    fn resize(
        &mut self,
        device: &wgpu::Device,
        color_bgl: &wgpu::BindGroupLayout,
        size: [u32; 2],
    ) {
        self.tex_view = make_iter_texture(device, size);
        self.aux_view = make_iter_texture(device, size);
        self.color_bg = make_color_bg(
            device, color_bgl, &self.color_uniform, &self.tex_view, &self.aux_view,
        );
        self.size = size;
        self.last_iter_key = None;
        self.rendered = false; // the new texture is blank until re-iterated
    }

    fn ensure_orbit_capacity(
        &mut self,
        device: &wgpu::Device,
        iter_bgl: &wgpu::BindGroupLayout,
        samples: u32,
    ) {
        if samples > self.orbit_cap {
            self.orbit_cap = samples.next_power_of_two();
            self.orbit_buf = make_orbit_buffer(device, self.orbit_cap);
            self.iter_bg = make_iter_bg(device, iter_bgl, &self.iter_uniform, &self.orbit_buf);
        }
    }
}

/// Per-frame data the app hands to the paint callback. The reference orbit is
/// precomputed (arbitrary precision) by the app and shared via `Arc`.
#[derive(Clone)]
pub struct MandelbrotParams {
    pub orbit: Arc<Vec<[f32; 4]>>,
    /// Changes whenever `orbit` changes — triggers a GPU re-upload.
    pub orbit_id: u64,
    pub orbit_len: u32,
    /// BLA tree (flattened, 4 `[f32;4]` per node), uploaded right after the orbit. Empty when
    /// BLA is off. `bla_on = 1` activates the shader's BLA traversal (Mandelbrot mode 2).
    pub bla: Arc<Vec<[f32; 4]>>,
    pub bla_on: u32,
    /// (view center − reference) *mantissa* (scaled by 2^-delta_exp), df32.
    pub ref_offset: [f32; 4],
    /// Shared base-2 exponent of the δ mantissas (ref_offset and the per-texel step).
    pub delta_exp: i32,
    /// Series-approximation skip (0 = none) + order-3 coefficients (complex df32 mantissa ×
    /// 2^exp): the GPU seeds δz ≈ A·δc + B·δc² + C·δc³ and starts iterating at `sa_skip`.
    pub sa_skip: u32,
    pub sa_a: [f32; 4],
    pub sa_a_exp: i32,
    pub sa_b: [f32; 4],
    pub sa_b_exp: i32,
    pub sa_c: [f32; 4],
    pub sa_c_exp: i32,
    /// View center df32 (re_hi, im_hi, re_lo, im_lo) — used by the direct path.
    pub center: [f32; 4],
    /// Julia parameter df32 (re_hi, im_hi, re_lo, im_lo).
    pub julia_c: [f32; 4],
    /// 0 = perturbation (deep), 1 = direct df32 (shallow, glitch-free).
    pub mode: u32,
    /// Escape-time formula id (see the shader's `fs_iterate`).
    pub formula: u32,
    /// 0 = Mandelbrot mode (z0=0, c=pixel), 1 = Julia mode (z0=pixel, c=const).
    pub julia: u32,
    /// Complex span *mantissa* (`span · 2^-delta_exp`, O(1)) — see [`fractadyne_core::GpuScale`].
    pub span_mantissa: [f64; 2],
    pub max_iter: u32,
    pub cycle: f32,
    pub offset: f32,
    pub stop_count: u32,
    pub stops: [[f32; 4]; 8],
    /// Slope/relief lighting from the distance-estimate normal.
    pub light: u32,
    pub light_angle: f32,
    pub light_height: f32,
    /// Distance-estimate glow (contour bands; `de_phase` animates them).
    pub de_on: u32,
    pub de_strength: f32,
    pub de_width: f32,
    pub de_phase: f32,
    /// Coloring method: 0 smooth, 1 stripe avg, 2 triangle-ineq, 3 orbit trap,
    /// 4 distance, 5 decomposition. Methods 1/2/3/5 accumulate orbit statistics.
    pub color_method: u32,
    pub stripe_freq: f32,
    pub trap_type: u32,
    /// Color-pass box-filter taps per axis (≥1). >1 anti-aliases an upscaled iteration
    /// texture (set when the work-budget reduced its resolution below the display).
    pub aa_filter: u32,
    /// Color for in-set (non-escaping) pixels (rgb + unused). Lets binary/duotone modes
    /// pick the interior color; defaults to near-black.
    pub interior_col: [f32; 4],
    pub resolution: [u32; 2],
    pub ss: u32,
    /// Pan reprojection: 1 = don't re-iterate, just translate the frozen iteration texture by
    /// `uv_offset` (used while dragging so the detailed image slides with the cursor and only
    /// the newly-exposed edge is blank until the view settles).
    pub reproject: u32,
    pub uv_offset: [f32; 2],
    /// Reprojection scale about the view centre: 1.0 = pan only; <1.0 = the view has zoomed in
    /// since the frozen frame, so the held detail is magnified to follow the zoom.
    pub uv_scale: f32,
    /// Which on-screen panel this is (distinct GPU resources per id).
    pub view_id: u32,
}

/// Whether a coloring method needs the per-iteration orbit statistics (aux target).
/// Coloring methods that accumulate per-orbit statistics into the aux target (stripe,
/// triangle-inequality, orbit-trap, decomposition) — these need every iterate, so series
/// approximation must be disabled for them.
pub fn method_needs_aux(method: u32) -> bool {
    matches!(method, 1 | 2 | 3 | 5)
}

impl CallbackTrait for MandelbrotParams {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &ScreenDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let r = match resources.get_mut::<Renderer>() {
            Some(r) => r,
            None => return Vec::new(),
        };

        // Ensure this panel's resources exist, then borrow shared layouts/pipeline
        // (disjoint fields) alongside the per-view resources.
        if !r.views.contains_key(&self.view_id) {
            let vr = ViewResources::new(device, &r.iter_bgl, &r.color_bgl);
            r.views.insert(self.view_id, vr);
        }
        let iter_bgl = &r.iter_bgl;
        let color_bgl = &r.color_bgl;
        let iter_pipeline = &r.iter_pipeline;
        let view = r.views.get_mut(&self.view_id).unwrap();

        // The offscreen iteration texture is (resolution × ss). It must not exceed
        // the device's max 2D texture dimension (e.g. 8192) or wgpu fatally errors —
        // this happens on large/maximized windows. Drop supersampling first (keeps
        // native resolution), then clamp the base as a last resort.
        let max_dim = device.limits().max_texture_dimension_2d;
        let base = [
            self.resolution[0].clamp(1, max_dim),
            self.resolution[1].clamp(1, max_dim),
        ];
        let mut ss = self.ss.max(1);
        while ss > 1
            && (base[0].saturating_mul(ss) > max_dim || base[1].saturating_mul(ss) > max_dim)
        {
            ss -= 1;
        }
        let size = [(base[0] * ss).min(max_dim), (base[1] * ss).min(max_dim)];
        // Pan reprojection: keep the frozen iteration texture (only valid once something has
        // been rendered into it). Skip the resize so the texture isn't cleared, and color it
        // with the ss it was built at.
        let reproject = self.reproject == 1 && view.rendered;
        if !reproject && size != view.size {
            view.resize(device, color_bgl, size);
        }
        let color_ss = if reproject { view.last_ss.max(1) } else { ss };

        // Coloring uniform is cheap — refresh every frame so recolor is instant.
        let cu = ColorUniforms {
            stop_count: self.stop_count,
            cycle: self.cycle,
            offset: self.offset,
            ss: color_ss,
            light: self.light,
            light_angle: self.light_angle,
            light_height: self.light_height,
            de_on: self.de_on,
            de_strength: self.de_strength,
            de_width: self.de_width,
            de_phase: self.de_phase,
            color_method: self.color_method,
            aa_filter: self.aa_filter.max(1),
            reproject: reproject as u32,
            uv_offset: self.uv_offset,
            uv_scale: if self.uv_scale > 0.0 { self.uv_scale } else { 1.0 },
            _pad_uv: [0.0; 3],
            interior_col: self.interior_col,
            stops: self.stops,
        };
        queue.write_buffer(&view.color_uniform, 0, bytemuck::bytes_of(&cu));

        // Upload the reference orbit only when it changed (app caches it). The BLA tree (if
        // any) is appended right after it in the same buffer, starting at index `orbit_len`.
        if self.orbit_id != view.last_orbit_id && !self.orbit.is_empty() {
            let total = self.orbit.len() + self.bla.len();
            view.ensure_orbit_capacity(device, iter_bgl, total as u32);
            queue.write_buffer(&view.orbit_buf, 0, bytemuck::cast_slice(self.orbit.as_slice()));
            if !self.bla.is_empty() {
                let off = (self.orbit.len() * 16) as u64; // 16 B per [f32;4]
                queue.write_buffer(&view.orbit_buf, off, bytemuck::cast_slice(self.bla.as_slice()));
            }
            view.last_orbit_id = self.orbit_id;
            view.last_iter_key = None; // force re-iterate against the new orbit
        }

        // Re-iterate only when the view / orbit changed.
        let key = IterKey {
            ref_offset: self.ref_offset,
            center: self.center,
            julia_c: self.julia_c,
            span_mantissa: self.span_mantissa,
            max_iter: self.max_iter,
            size,
            orbit_id: self.orbit_id,
            mode: self.mode,
            formula: self.formula,
            julia: self.julia,
            delta_exp: self.delta_exp,
            color_method: self.color_method,
            stripe_freq: self.stripe_freq,
            trap_type: self.trap_type,
            sa_skip: self.sa_skip,
            bla_on: self.bla_on,
        };
        if !reproject && view.last_iter_key != Some(key) {
            // Per-texel step *mantissa*: span_mantissa (= span · 2^-delta_exp, already O(1))
            // divided by the texture dim. The shared exponent carries the true scale, so no
            // tiny span / huge 2^-delta_exp ever appears here → no underflow/overflow at depth.
            let split = |v: f64| -> (f32, f32) {
                let hi = v as f32;
                (hi, (v - hi as f64) as f32)
            };
            let (sxh, sxl) = split(self.span_mantissa[0] / size[0] as f64);
            let (syh, syl) = split(self.span_mantissa[1] / size[1] as f64);
            let iu = IterUniforms {
                step: [sxh, sxl, syh, syl],
                ref_offset: self.ref_offset,
                center: self.center,
                julia_c: self.julia_c,
                res: [size[0] as f32, size[1] as f32],
                px_offset: [0.0, 0.0],
                max_iter: self.max_iter,
                orbit_len: self.orbit_len,
                mode: self.mode,
                formula: self.formula,
                julia: self.julia,
                delta_exp: self.delta_exp,
                color_method: self.color_method,
                stripe_freq: self.stripe_freq,
                trap_type: self.trap_type,
                aux_on: method_needs_aux(self.color_method) as u32,
                sa_skip: self.sa_skip,
                glitch_on: 0, // live view never runs glitch detection (single-ref + rebasing)
                sa_a: self.sa_a,
                sa_b: self.sa_b,
                sa_c: self.sa_c,
                sa_a_exp: self.sa_a_exp,
                sa_b_exp: self.sa_b_exp,
                sa_c_exp: self.sa_c_exp,
                bla_on: self.bla_on,
            };
            queue.write_buffer(&view.iter_uniform, 0, bytemuck::bytes_of(&iu));
            {
                let attach = |v| Some(wgpu::RenderPassColorAttachment {
                    view: v,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                });
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("fractadyne.iterate_pass"),
                    color_attachments: &[attach(&view.tex_view), attach(&view.aux_view)],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(iter_pipeline);
                pass.set_bind_group(0, &view.iter_bg, &[]);
                pass.draw(0..3, 0..1);
            }
            view.last_iter_key = Some(key);
            view.last_ss = ss; // remember the ss this texture was built at (for reprojection)
            view.rendered = true; // the texture now holds a real frame (survives orbit swaps)
        }

        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &CallbackResources,
    ) {
        if let Some(r) = resources.get::<Renderer>() {
            if let Some(view) = r.views.get(&self.view_id) {
                render_pass.set_pipeline(&r.color_pipeline);
                render_pass.set_bind_group(0, &view.color_bg, &[]);
                render_pass.draw(0..3, 0..1);
            }
        }
    }
}

pub fn install_renderer(render_state: &egui_wgpu::RenderState) {
    let renderer = Renderer::new(&render_state.device, render_state.target_format);
    render_state
        .renderer
        .write()
        .callback_resources
        .insert(renderer);
}

pub fn add_mandelbrot(painter: &egui::Painter, rect: egui::Rect, params: MandelbrotParams) {
    painter.add(egui_wgpu::Callback::new_paint_callback(rect, params));
}

const EXPORT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

/// Everything needed to render one frame offscreen at an arbitrary resolution.
/// Mirrors the live view's parameters (the app fills it from the current view).
#[derive(Clone)]
pub struct ExportRequest {
    pub width: u32,
    pub height: u32,
    pub ss: u32,
    /// Complex span *mantissa* (`span · 2^-delta_exp`, O(1)) — see [`fractadyne_core::GpuScale`].
    pub span_mantissa: [f64; 2],
    pub center: [f32; 4],
    pub ref_offset: [f32; 4],
    pub delta_exp: i32,
    /// Series-approximation skip (0 = none) + order-3 coeffs (complex df32 mantissa × 2^exp).
    pub sa_skip: u32,
    /// 1 = flag Pauldelbrot-glitched pixels with a `-2` sentinel in the iteration texture's `r`
    /// channel (multi-reference correction passes read this back). 0 for normal rendering.
    pub glitch_on: u32,
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
) -> Result<ExportResult, String> {
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
    let tile = by_tex.min(by_buf).min(by_work).min(2048).max(1);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fractadyne.export_shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("mandelbrot.wgsl").into()),
    });

    let uniform_entry = wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    let iter_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("export.iter_bgl"),
        entries: &[
            uniform_entry,
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let tex_entry = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    let color_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("export.color_bgl"),
        entries: &[uniform_entry, tex_entry(1), tex_entry(2)],
    });
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
        _pad_uv: [0.0; 3],
        interior_col: req.interior_col,
        stops: req.stops,
    };
    queue.write_buffer(&color_uniform, 0, bytemuck::bytes_of(&cu));
    let split = |v: f64| -> (f32, f32) {
        let hi = v as f32;
        (hi, (v - hi as f64) as f32)
    };
    // Step mantissa = span_mantissa (already × 2^-delta_exp) / texdim — O(1), no overflow.
    let (sxh, sxl) = split(req.span_mantissa[0] / (w as f64 * ss as f64));
    let (syh, syl) = split(req.span_mantissa[1] / (h as f64 * ss as f64));

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
                return Err("canceled".to_string());
            }
            let tw = tile.min(w - tx0);
            let iw = tw * ss;
            let ih = th * ss;

            let iu = IterUniforms {
                step: [sxh, sxl, syh, syl],
                ref_offset: req.ref_offset,
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
                .map_err(|e| e.to_string())?
                .map_err(|e| e.to_string())?;

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
) -> Result<ExportResult, String> {
    let max_dim = device.limits().max_texture_dimension_2d;
    let w = req.width.clamp(1, max_dim);
    let h = req.height.clamp(1, max_dim);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fractadyne.selftest_shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("mandelbrot.wgsl").into()),
    });
    let iter_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("selftest.iter_bgl"),
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
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
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
    let (sxh, sxl) = split(req.span_mantissa[0] / w as f64);
    let (syh, syl) = split(req.span_mantissa[1] / h as f64);
    let iu = IterUniforms {
        step: [sxh, sxl, syh, syl],
        ref_offset: req.ref_offset,
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
    rx.recv().map_err(|e| e.to_string())?.map_err(|e| e.to_string())?;

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
) -> Result<ExportResult, String> {
    let w = req.width.max(1);
    let h = req.height.max(1);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fractadyne.color_iter_shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("mandelbrot.wgsl").into()),
    });
    let uniform_entry = wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    let tex_entry = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    let color_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("coloriter.bgl"),
        entries: &[uniform_entry, tex_entry(1), tex_entry(2)],
    });
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
        _pad_uv: [0.0; 3],
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
    rx.recv().map_err(|e| e.to_string())?.map_err(|e| e.to_string())?;

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
