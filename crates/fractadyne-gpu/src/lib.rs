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

mod export;
pub use export::*;
pub mod timing;

// RGBA: r = smooth iteration value, g/b = slope normal (x,y), a = reserved (DE).
pub(crate) const ITER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct IterUniforms {
    pub(crate) step: [f32; 4],       // per-texel complex step, df32: (x_hi, x_lo, y_hi, y_lo)
    pub(crate) ref_offset: [f32; 4], // (center − reference) df32: (re_hi, im_hi, re_lo, im_lo)
    pub(crate) center: [f32; 4],     // view center df32 (direct mode)
    pub(crate) julia_c: [f32; 4],    // Julia parameter df32 (re_hi, im_hi, re_lo, im_lo)
    pub(crate) res: [f32; 2],        // FULL iteration resolution (across all export tiles)
    pub(crate) px_offset: [f32; 2],  // this tile's top-left texel in the full grid (0 for live)
    pub(crate) max_iter: u32,
    pub(crate) orbit_len: u32,
    pub(crate) mode: u32,    // 0 = perturbation, 1 = direct df32
    pub(crate) formula: u32, // escape-time formula id
    pub(crate) julia: u32,   // 0 = Mandelbrot mode, 1 = Julia mode
    pub(crate) delta_exp: i32, // shared base-2 exponent of the δ mantissas (step / ref_offset)
    pub(crate) color_method: u32, // selected coloring method (drives aux accumulation)
    pub(crate) stripe_freq: f32,  // stripe-average angular frequency
    pub(crate) trap_type: u32,    // orbit-trap shape: 0 point, 1 cross, 2 circle
    pub(crate) aux_on: u32,       // 1 = accumulate orbit statistics into the aux target
    pub(crate) sa_skip: u32,      // series-approximation skip (0 = none): seed δz at this iteration
    pub(crate) glitch_on: u32,    // 1 = flag Pauldelbrot-glitched pixels (multi-reference correction pass)
    pub(crate) sa_a: [f32; 4],    // order-3 series coeffs (complex df32 mantissa): δz ≈ A·δc + B·δc² + C·δc³
    pub(crate) sa_b: [f32; 4],
    pub(crate) sa_c: [f32; 4],
    pub(crate) sa_a_exp: i32,     // per-coefficient base-2 exponents (floatexp)
    pub(crate) sa_b_exp: i32,
    pub(crate) sa_c_exp: i32,
    pub(crate) bla_on: u32,       // 1 = use the BLA tree (appended after the orbit at index orbit_len)
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ColorUniforms {
    pub(crate) stop_count: u32,
    pub(crate) cycle: f32,
    pub(crate) offset: f32,
    pub(crate) ss: u32,
    pub(crate) light: u32,         // 0 = off, 1 = slope/relief lighting from the stored normal
    pub(crate) light_angle: f32,   // light direction (radians)
    pub(crate) light_height: f32,  // relief strength (smaller = stronger; ~1.5 default)
    pub(crate) de_on: u32,         // 0 = off, 1 = distance-estimate glow
    pub(crate) de_strength: f32,
    pub(crate) de_width: f32,
    pub(crate) de_phase: f32,
    pub(crate) color_method: u32,
    pub(crate) aa_filter: u32,
    pub(crate) reproject: u32,      // 1 = reprojection: sample the frozen iteration texture scaled+translated
    pub(crate) uv_offset: [f32; 2], // uv translation applied when reproject == 1
    pub(crate) uv_scale: f32,       // uv scale about centre (1 = pan only; <1 = zoomed since the frozen frame)
    pub(crate) vig_on: u32,         // 1 = spotlight vignette (dim outside a soft circle)
    pub(crate) vig_dim: f32,
    pub(crate) vig_soft: f32,
    pub(crate) vig_center: [f32; 2], // screen uv
    pub(crate) vig_radius: f32,
    pub(crate) _pad_vig: f32,
    pub(crate) interior_col: [f32; 4],
    pub(crate) stops: [[f32; 4]; 8],
    pub(crate) out_res: [f32; 2], // output rect size in px; with `reproject`, aspect-fits a frozen frame
    pub(crate) _pad_out: [f32; 2],
}

/// Spotlight vignette parameters (dim outside a soft circle), shared by the live and export paths.
#[derive(Clone, Copy, Default)]
pub struct Vignette {
    pub on: u32,
    pub dim: f32,
    pub soft: f32,
    pub center: [f32; 2],
    pub radius: f32,
}

/// When the iteration pass must re-run.
///
/// NOTE for anyone adding a field: the app's tiled settle (see `MandelbrotParams::tile`) re-renders
/// SCISSORED tiles under an app-side key that must change whenever this key changes — a key change
/// the app misses re-renders only the current tile rect and splices new-output data into an
/// old-output frame. If the new field changes the iterate's OUTPUT, mirror it in the app's
/// `settings_hash` (fractadyne-app/src/render.rs, build_params).
#[derive(Clone, Copy, PartialEq)]
struct IterKey {
    ref_offset: RefOffset,
    center: [f32; 4],
    julia_c: [f32; 4],
    span_mantissa: fractadyne_core::SpanMantissa,
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

/// GPU-timestamp capture around the LIVE `iterate_pass`, so the app can size deep frames against what
/// the iterate ACTUALLY costs on this GPU at this view.
///
/// Every cheaper proxy was tried and is wrong. A wall-clock frame interval is dominated by repaint
/// scheduling (~420 ms here regardless of frame size). CPU time around the offscreen `render_iter` is
/// ~440 ms of fixed per-call overhead. And the offscreen `render_export` pass — even timestamped — runs
/// ~25× slower per step than the live pass, so it under-reads the rate and shrinks the view to a
/// postage stamp. Only the live pass measures the live pass.
///
/// The result lands two frames late (record → submit → map), which is fine: the budget it feeds is a
/// slow-moving control, and until it arrives the frame uses the tiny bootstrap budget.
struct IterTiming {
    qs: wgpu::QuerySet,
    /// `resolve_query_set` target (QUERY_RESOLVE | COPY_SRC), then copied into `read`.
    resolve: wgpu::Buffer,
    /// MAP_READ staging copy: two u64 ticks.
    read: wgpu::Buffer,
    state: TimingState,
    done: Arc<std::sync::atomic::AtomicBool>,
}

#[derive(Clone, Copy, PartialEq)]
enum TimingState {
    /// Nothing in flight — the next iterate may bracket itself with timestamps.
    Idle,
    /// Timestamps written + resolved into `read`; the encoder has not been submitted yet.
    Recorded,
    /// `map_async` issued; waiting for the callback.
    Mapping,
}

impl IterTiming {
    fn new(device: &wgpu::Device) -> Option<Self> {
        if !device.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
            return None;
        }
        let buf = |label, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: 16,
                usage,
                mapped_at_creation: false,
            })
        };
        Some(Self {
            qs: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("fractadyne.iterate_ts"),
                ty: wgpu::QueryType::Timestamp,
                count: 2,
            }),
            resolve: buf(
                "fractadyne.iterate_ts.resolve",
                wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            ),
            read: buf(
                "fractadyne.iterate_ts.read",
                wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            ),
            state: TimingState::Idle,
            done: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }

    /// Advance the readback state machine; publishes the last completed pass time (ms, f64 bits) into
    /// `out`. Called once per `prepare`, before the pass is (re)recorded.
    fn pump(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        out: Option<&Arc<std::sync::atomic::AtomicU64>>,
    ) {
        use std::sync::atomic::Ordering::SeqCst;
        match self.state {
            // The encoder that wrote these timestamps was submitted after last frame's `prepare`
            // returned, so the copy is queued now and `map_async` can be issued against it.
            TimingState::Recorded => {
                self.done.store(false, SeqCst);
                let done = self.done.clone();
                self.read.slice(..).map_async(wgpu::MapMode::Read, move |r| {
                    if r.is_ok() {
                        done.store(true, SeqCst);
                    }
                });
                self.state = TimingState::Mapping;
            }
            TimingState::Mapping => {
                let _ = device.poll(wgpu::Maintain::Poll);
                if self.done.load(SeqCst) {
                    {
                        let data = self.read.slice(..).get_mapped_range();
                        let ticks: [u64; 2] = bytemuck::pod_read_unaligned(&data[..16]);
                        let ns = ticks[1].saturating_sub(ticks[0]) as f64
                            * queue.get_timestamp_period() as f64;
                        if let Some(o) = out {
                            o.store((ns / 1.0e6).to_bits(), SeqCst);
                        }
                    }
                    self.read.unmap();
                    self.state = TimingState::Idle;
                }
            }
            TimingState::Idle => {}
        }
    }
}

/// Per-view GPU resources (one set per on-screen panel, keyed by `view_id`). Each
/// view owns its offscreen iteration texture, uniforms, orbit buffer, and caching
/// state so two panels (e.g. Mandelbrot + Julia) can render without clobbering.
struct ViewResources {
    /// GPU timing for the live iterate pass; `None` when the device lacks `TIMESTAMP_QUERY`.
    timing: Option<IterTiming>,
    /// The tile rect rendered last (base px), alongside `last_iter_key`: a tiled settle re-renders
    /// when the RECT advances even though the key (view/orbit/size) is unchanged.
    last_tile: Option<[u32; 4]>,
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
    /// Nearest-neighbour upscale of the old iteration/aux textures into a resized pair — runs once
    /// when a tiled settle grows the texture, so the display never drops to black mid-refine.
    seed_pipeline: wgpu::RenderPipeline,
    iter_bgl: wgpu::BindGroupLayout,
    color_bgl: wgpu::BindGroupLayout,
    views: std::collections::HashMap<u32, ViewResources>,
}

pub(crate) fn make_iter_texture(device: &wgpu::Device, size: [u32; 2]) -> wgpu::TextureView {
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

pub(crate) fn make_orbit_buffer(device: &wgpu::Device, capacity: u32) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("fractadyne.orbit_buf"),
        size: (capacity.max(1) as u64) * 16, // df64 vec4<f32> = 16 bytes
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

pub(crate) fn make_iter_bg(
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

pub(crate) fn make_color_bg(
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

pub(crate) fn fullscreen_pipeline(
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

/// The WGSL shader module (iterate + color passes). Created per call site — the live view builds it
/// once at init and each export builds it once; both are occasional, so the compile cost is
/// negligible and a single definition keeps the four former copies from drifting.
pub(crate) fn shader_module(device: &wgpu::Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fractadyne.shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("mandelbrot.wgsl").into()),
    })
}

/// A fragment-visible uniform buffer at `binding` 0 — shared by both bind-group layouts.
fn uniform_bgl_entry() -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

/// Bind-group layout for the **iterate** pass: uniforms (0) + the read-only reference/BLA orbit
/// storage buffer (1). Identical for the live view and every export path.
pub(crate) fn iter_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("fractadyne.iter_bgl"),
        entries: &[
            uniform_bgl_entry(),
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
    })
}

/// Bind-group layout for the **color** pass: uniforms (0) + the iteration texture (1) + the aux
/// statistics texture (2). Identical for the live view and every export path.
pub(crate) fn color_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("fractadyne.color_bgl"),
        entries: &[uniform_bgl_entry(), tex_entry(1), tex_entry(2)],
    })
}

impl Renderer {
    fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = shader_module(device);
        let iter_bgl = iter_bind_group_layout(device);
        let color_bgl = color_bind_group_layout(device);

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
        // Upscale-copy for a tiled settle's resize (see `MandelbrotParams::tile`). Shares the color
        // pass's layout: fs_seed reads only the two texture bindings.
        let seed_pipeline = fullscreen_pipeline(
            device, &shader, &color_layout, "fs_seed", &[ITER_FORMAT, ITER_FORMAT],
            "fractadyne.seed_pipeline",
        );

        Self {
            iter_pipeline,
            color_pipeline,
            seed_pipeline,
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
            timing: IterTiming::new(device),
            last_tile: None,
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
        self.last_tile = None;
        self.rendered = false; // the new texture is blank until re-iterated
    }

    /// Resize for a tiled settle: like [`resize`](Self::resize), but seed the new textures with a
    /// nearest-neighbour upscale of the old ones before any tile lands. The refine then happens in
    /// place — coarse everywhere, sharpening tile by tile — instead of against a black texture.
    /// `rendered` stays true: the seeded content is real (just coarse), so reprojection and the
    /// color pass may keep using it mid-grid.
    fn seeded_resize(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        color_bgl: &wgpu::BindGroupLayout,
        seed_pipeline: &wgpu::RenderPipeline,
        size: [u32; 2],
    ) {
        let old_tex = std::mem::replace(&mut self.tex_view, make_iter_texture(device, size));
        let old_aux = std::mem::replace(&mut self.aux_view, make_iter_texture(device, size));
        self.color_bg = make_color_bg(
            device, color_bgl, &self.color_uniform, &self.tex_view, &self.aux_view,
        );
        // fs_seed reads only the texture bindings; the uniform slot is filled to satisfy the layout.
        let seed_bg = make_color_bg(device, color_bgl, &self.color_uniform, &old_tex, &old_aux);
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
                label: Some("fractadyne.seed_pass"),
                color_attachments: &[attach(&self.tex_view), attach(&self.aux_view)],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(seed_pipeline);
            pass.set_bind_group(0, &seed_bg, &[]);
            pass.draw(0..3, 0..1);
        }
        self.size = size;
        self.last_iter_key = None;
        self.last_tile = None;
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

/// The `(center − reference)` offset as a df32 (double-single) pair — the reference-orbit
/// anchor for perturbation. Typed so the four packed `f32` limbs can't be transposed: the
/// hi/lo split happens once in [`RefOffset::from_df32`], and the packed GPU layout
/// `[re_hi, im_hi, re_lo, im_lo]` (what the shader reads as real `(.x,.z)` / imag `(.y,.w)`)
/// exists only inside [`RefOffset::to_array`], used at the `IterUniforms` boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RefOffset {
    re_hi: f32,
    im_hi: f32,
    re_lo: f32,
    im_lo: f32,
}

impl RefOffset {
    /// All-zero offset — the direct (mode 1) path has no reference.
    pub const ZERO: RefOffset = RefOffset { re_hi: 0.0, im_hi: 0.0, re_lo: 0.0, im_lo: 0.0 };

    /// Pack the real/imag offset mantissas (already scaled by `2^-delta_exp`, so O(1)) as a df32
    /// pair, splitting each into hi/lo `f32` limbs. The one place the hi/lo split is authored.
    #[inline]
    pub fn from_df32(re: f64, im: f64) -> RefOffset {
        let re_hi = re as f32;
        let im_hi = im as f32;
        RefOffset {
            re_hi,
            im_hi,
            re_lo: (re - re_hi as f64) as f32,
            im_lo: (im - im_hi as f64) as f32,
        }
    }

    /// Lower to the packed GPU array `[re_hi, im_hi, re_lo, im_lo]` — the shader reads real from
    /// `(.x, .z)` and imag from `(.y, .w)`. Used ONLY at the `IterUniforms` (Pod) boundary.
    #[inline]
    pub fn to_array(self) -> [f32; 4] {
        [self.re_hi, self.im_hi, self.re_lo, self.im_lo]
    }
}

impl Default for RefOffset {
    fn default() -> Self {
        RefOffset::ZERO
    }
}

/// Per-frame data the app hands to the paint callback. The reference orbit is
/// precomputed (arbitrary precision) by the app and shared via `Arc`.
#[derive(Clone)]
pub struct MandelbrotParams {
    /// Sink for the live iterate pass's measured GPU time (milliseconds, as `f64::to_bits`). The app
    /// sizes deep floatexp frames against it; see [`IterTiming`]. `None` disables capture. Written a
    /// couple of frames after the pass it describes, and only for frames that actually re-iterate.
    pub iterate_ms: Option<Arc<std::sync::atomic::AtomicU64>>,
    /// Tiled settle: `Some([x, y, w, h])` in BASE pixels renders only that sub-rect of the iteration
    /// texture this frame (scissored, `LoadOp::Load`), leaving the rest intact. The uniforms are
    /// identical to a full-frame render and the fragment coordinates are absolute in the texture, so
    /// a completed grid of tiles is exactly the frame ONE dispatch would have produced — just split
    /// into dispatches the GPU watchdog and the UI thread can tolerate. A zero-area rect is a "hold":
    /// render nothing, keep the texture (used when another view owns this frame's tile slot).
    /// `None` = the ordinary full-frame path, unchanged.
    pub tile: Option<[u32; 4]>,
    pub orbit: Arc<Vec<[f32; 4]>>,
    /// Changes whenever `orbit` changes — triggers a GPU re-upload.
    pub orbit_id: u64,
    pub orbit_len: u32,
    /// BLA tree (flattened, 4 `[f32;4]` per node), uploaded right after the orbit. Empty when
    /// BLA is off. `bla_on = 1` activates the shader's BLA traversal (Mandelbrot mode 2).
    pub bla: Arc<Vec<[f32; 4]>>,
    pub bla_on: u32,
    /// (view center − reference) *mantissa* (scaled by 2^-delta_exp), df32.
    pub ref_offset: RefOffset,
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
    pub span_mantissa: fractadyne_core::SpanMantissa,
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
    /// Spotlight vignette (guided tours); `on == 0` disables it.
    pub vignette: Vignette,
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
        let seed_pipeline = &r.seed_pipeline;
        let view = r.views.get_mut(&self.view_id).unwrap();

        // Drain any timestamp readback armed on an earlier frame before this frame may re-arm it.
        if let Some(t) = view.timing.as_mut() {
            t.pump(device, queue, self.iterate_ms.as_ref());
        }

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
        // A zero-area tile is a "hold" frame (another view owns this frame's tile slot): render
        // nothing, keep the texture exactly as it is — like a reprojection with no translation.
        let hold = self.tile.is_some_and(|t| t[2] == 0 || t[3] == 0);
        if !reproject && !hold && size != view.size {
            if self.tile.is_some() && view.rendered {
                // Entering (or re-entering) a tiled settle at a new texture size: seed the resized
                // textures from the old content so the refine happens in place, not against black.
                view.seeded_resize(device, encoder, color_bgl, seed_pipeline, size);
            } else {
                view.resize(device, color_bgl, size);
            }
        }
        // A held frame (reprojection or tile-hold) displays the EXISTING texture, so it must be
        // colored at the ss and size it was built with, not this frame's request.
        let held = reproject || hold;
        let color_ss = if held { view.last_ss.max(1) } else { ss };

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
            vig_on: self.vignette.on,
            vig_dim: self.vignette.dim,
            vig_soft: self.vignette.soft,
            vig_center: self.vignette.center,
            vig_radius: self.vignette.radius,
            _pad_vig: 0.0,
            interior_col: self.interior_col,
            stops: self.stops,
            // For a reprojection the frozen iteration texture may have been rendered at a REDUCED
            // resolution (motion `res_scale`) — which now happens mid-dive since v0.1.58/59 refresh
            // floatexp while moving. The color-pass aspect-fit is `fit = out_res / screen_dim` where
            // `screen_dim` is the frozen texture's base resolution; if `out_res` were the current
            // (native) frame size, `fit = 1/res_scale > 1` would shrink the frozen content into a
            // central patch ringed by the average color (a "circle" that resizes as `res_scale`
            // varies and slides with `uv_off` — the deep-zoom "jumping" artifact). Use the frozen
            // texture's own base resolution so `fit = 1` (identity) and the sampler upscales it to
            // fill the view. Native (settled) frozen frames are unchanged. (Trade-off: a window
            // resize *during* a reprojection no longer aspect-fits until the next real frame — rare
            // and self-correcting.)
            out_res: if held {
                [
                    (view.size[0] as f32 / color_ss as f32).max(1.0),
                    (view.size[1] as f32 / color_ss as f32).max(1.0),
                ]
            } else {
                [base[0] as f32, base[1] as f32]
            },
            _pad_out: [0.0, 0.0],
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
        // Re-render when the key changed (new view/orbit/size) OR when a tiled settle advanced to a
        // new rect under an unchanged key. A completed grid keeps sending its final rect, so
        // `(key, tile)` stops changing and the frame is served from the texture like any cached one.
        if !reproject && !hold && (view.last_iter_key != Some(key) || view.last_tile != self.tile) {
            // Per-texel step *mantissa*: span_mantissa (= span · 2^-delta_exp, already O(1))
            // divided by the texture dim. The shared exponent carries the true scale, so no
            // tiny span / huge 2^-delta_exp ever appears here → no underflow/overflow at depth.
            let split = |v: f64| -> (f32, f32) {
                let hi = v as f32;
                (hi, (v - hi as f64) as f32)
            };
            let (sxh, sxl) = split(self.span_mantissa.x / size[0] as f64);
            let (syh, syl) = split(self.span_mantissa.y / size[1] as f64);
            let iu = IterUniforms {
                step: [sxh, sxl, syh, syl],
                ref_offset: self.ref_offset.to_array(),
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
            // Bracket the iterate with GPU timestamps when nothing is already in flight. This is the
            // only measurement of the deep iterate that isn't contaminated by vsync, repaint
            // scheduling, pipeline setup, or the offscreen path's different cost profile.
            let arm_ts = view
                .timing
                .as_ref()
                .is_some_and(|t| t.state == TimingState::Idle);
            // Tile rect in TEXTURE pixels (base px × ss), clamped inside the target. Fragment
            // coordinates are absolute in the texture, so a scissored draw fills exactly this
            // sub-rect with the values a full-frame draw would have put there.
            let tile_px = self.tile.map(|t| {
                let x = t[0].saturating_mul(ss).min(size[0]);
                let y = t[1].saturating_mul(ss).min(size[1]);
                let w = t[2].saturating_mul(ss).min(size[0] - x);
                let h = t[3].saturating_mul(ss).min(size[1] - y);
                [x, y, w, h]
            });
            {
                // A tile must compose with the tiles (and seed) already in the texture, so it loads
                // rather than clears. Full frames clear, as before. (`view.rendered` guards the
                // cold-start corner: a brand-new texture is zero-initialized either way.)
                let load = if tile_px.is_some() && view.rendered {
                    wgpu::LoadOp::Load
                } else {
                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                };
                let attach = |v| Some(wgpu::RenderPassColorAttachment {
                    view: v,
                    resolve_target: None,
                    ops: wgpu::Operations { load, store: wgpu::StoreOp::Store },
                });
                let ts_writes = arm_ts.then(|| wgpu::RenderPassTimestampWrites {
                    query_set: &view.timing.as_ref().unwrap().qs,
                    beginning_of_pass_write_index: Some(0),
                    end_of_pass_write_index: Some(1),
                });
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("fractadyne.iterate_pass"),
                    color_attachments: &[attach(&view.tex_view), attach(&view.aux_view)],
                    depth_stencil_attachment: None,
                    timestamp_writes: ts_writes,
                    occlusion_query_set: None,
                });
                if let Some(t) = tile_px {
                    pass.set_scissor_rect(t[0], t[1], t[2], t[3]);
                }
                pass.set_pipeline(iter_pipeline);
                pass.set_bind_group(0, &view.iter_bg, &[]);
                pass.draw(0..3, 0..1);
            }
            if arm_ts {
                let t = view.timing.as_mut().unwrap();
                encoder.resolve_query_set(&t.qs, 0..2, &t.resolve, 0);
                encoder.copy_buffer_to_buffer(&t.resolve, 0, &t.read, 0, 16);
                t.state = TimingState::Recorded;
            }
            view.last_iter_key = Some(key);
            view.last_tile = self.tile;
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

