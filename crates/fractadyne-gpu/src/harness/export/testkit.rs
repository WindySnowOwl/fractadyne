//! The harnesses' ways into the export renderer: the single-dispatch control the self-test holds
//! tiled exports to, the split, timed and chunked iterate passes `--selftest`, `--chunk-sweep` and
//! `--deviceloss-repro` measure with, and `--gputest`'s primitive check. Attached to `export`
//! (`#[path]`) for its private passes, and re-exported: callers name them as before.

use super::*;

/// [`render_export`] with the chunked per-tile iterate disabled — the single-dispatch control
/// for the selftest's bit-identity gate. Not for production use: this is exactly the unbounded
/// dispatch shape that lost the device (crash-1787292746).
#[doc(hidden)]
pub fn render_export_unchunked(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    req: &ExportRequest,
    progress: &std::sync::atomic::AtomicU32,
    cancel: &std::sync::atomic::AtomicBool,
) -> Result<ExportResult, GpuError> {
    render_export_impl(device, queue, req, progress, cancel, false)
}

/// `render_iter` drawn as the live view's SPLIT refresh draws a frame (`MandelbrotParams::split`):
/// `k` passes of `fs_iterate` over `vs_split_tiles`, one checkerboard set of 16-texel tiles each,
/// each its own submission, the texture cleared before the first and kept after. `only` renders
/// just that one set over the cleared texture; `k` = 1 draws every tile through the same geometry
/// (the cost claim's control). For `--selftest`: all `k` sets must compose the frame bit for bit,
/// and one set must cover only its share — and cost about that share.
///
/// `iterate_ms` here is the WALL time of the passes, each from its submission to its completion
/// (the pipeline is compiled before, the readback after). On the RX 6800 XT the GPU timestamps of a
/// COLD pass — the first of a new pipeline, which is all `render_iter` ever times — read a whole
/// 1024² pass at 1.75 ms against ~190 ms; warmed (`time_iter_pass`) they agree with the wall within 1%.
pub fn render_iter_split(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    req: &ExportRequest,
    k: u32,
    only: Option<u32>,
) -> Result<ExportResult, GpuError> {
    render_iter_passes(device, queue, req, Some((k.max(1), only)), None)
}

/// One `render_iter` pass timed both ways, `reps` times: `[GPU timestamp ms, wall ms]` each (the
/// timestamp NaN without `TIMESTAMP_QUERY`). Wall is submission to completion with the pass alone in
/// the queue, the pipeline built and warmed by one untimed pass first. `tiles` draws it through the
/// split refresh's tile geometry (`vs_split_tiles`, every tile) instead of the full-screen triangle.
/// For `--selftest`: the live refresh PRICES frames from these timestamps, so they must describe
/// their own pass. (The RX 6800 XT's read a COLD 1024² pass at 1.75 ms whose split twin took 190
/// wall; warmed, 157.8 against 156.6.)
pub fn time_iter_pass(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    req: &ExportRequest,
    tiles: bool,
    reps: u32,
) -> Result<Vec<[f64; 2]>, GpuError> {
    let mut out = Vec::new();
    render_iter_passes(device, queue, req, tiles.then_some((1, None)), Some((reps, &mut out)))?;
    Ok(out)
}

/// `render_iter`, but with **iteration-range tiling**: the `0..max_iter` loop is split across
/// `ceil(max_iter / chunk_iters)` bounded GPU submissions, carrying per-pixel state (z, dz, iter,
/// status) between passes in ping-pong textures — so an arbitrarily high iteration count can never
/// run as a single watchdog-tripping dispatch. Output is bit-identical to `render_iter` for the
/// supported scope (the resumable shader replicates the direct branch's arithmetic and order
/// exactly, and the state carries full df32 precision). Scope: DIRECT (`mode == 1`), df32
/// perturbation (`mode == 0`) and floatexp perturbation (`mode == 2`), formulas whose
/// `FormulaCaps::resumable_passes` is set, aux coloring off (glitch detection is supported since
/// beta.124 — a glitched pixel settles as `ST_GLITCHED` and resolves to the same `-2` sentinel the
/// single pass emits); anything else falls back to plain `render_iter`.
/// Always `ss = 1`, like `render_iter`. `iterate_ms` is not measured on this path (0.0).
///
/// Mode 2 runs the four-target `fs_iterate_chunk_fe` entry point instead — floatexp state does not
/// fit in three (see `chunking_mode2_available`) — and needs 64 bytes/sample rather than 48.
pub fn render_iter_chunked(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    req: &ExportRequest,
    chunk_iters: u32,
) -> Result<ExportResult, GpuError> {
    render_iter_chunked_timed(device, queue, req, chunk_iters, &mut Vec::new())
}

/// One window's uncensored wall cost, as measured between `queue.submit` and the `poll(Wait)`
/// that proves it finished.
///
/// ⭐**This is the only cost signal in the codebase that is neither right-censored nor deferred.**
/// The live path's two prices both are: a GPU timestamp cannot arm on a saturated queue, and the
/// wall fallback reads a frame interval, which wgpu-core caps at `FRAME_TIMEOUT_MS = 1000` in
/// `present.rs` — so a frame that overran by 3 s and one that overran by 1.1 s report the same
/// number. Here the submission is alone in the queue and the poll is a real completion, so the
/// figure is the submission's own duration.
#[derive(Clone, Copy, Debug)]
pub struct ChunkPassTiming {
    pub start_iter: u32,
    pub end_iter: u32,
    pub wall_ms: f64,
}

/// `render_iter_chunked`, additionally reporting what every individual window cost.
///
/// Split out for `--chunk-sweep`, which needs the per-window profile rather than the image: the
/// question it exists to answer is whether a window's cost is proportional to its iteration count
/// (in which case sizing the window is a sufficient actuator) or cliffs part-way through a
/// progression (in which case it is not). `passes` is CLEARED on entry and one entry is pushed per
/// submission, in cursor order. Pixels are unaffected — this adds two `Instant` reads per window
/// to a loop that already blocks on `poll(Wait)`.
///
/// ⚠An `Ok` result with `passes` still EMPTY means the unsupported-scope fallback to `render_iter`
/// fired (wrong mode, a formula without `resumable_passes`, `chunk_iters == 0`, or too few
/// color-attachment bytes) — i.e.
/// one unbounded dispatch ran instead of the windows you asked for. A caller measuring windows
/// must treat that as "not measured", never as "one fast window".
pub fn render_iter_chunked_timed(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    req: &ExportRequest,
    chunk_iters: u32,
    passes: &mut Vec<ChunkPassTiming>,
) -> Result<ExportResult, GpuError> {
    passes.clear();
    // The chunk pass writes Rgba32Float state attachments — three (48 bytes/sample) for direct and
    // mode 0, four (64) for mode 2. A device that granted less can't run it: fall back to the
    // single-pass render (the caller's TDR exposure is then what it always was; the app requests
    // min(adapter, 64) at device creation). Scope: direct (1), df32 perturbation (0) and floatexp
    // perturbation (2) with aux off, formulas with `FormulaCaps::resumable_passes` (Mandelbrot,
    // Multibrot 3–5) — the same scope the chunk shaders are written to, which is narrower than
    // `fs_iterate`'s.
    let fe = req.mode == 2; // floatexp: the four-target entry point
    let mode_ok = req.mode == 1 || req.mode == 0 || req.mode == 2;
    let attach_need = if fe { 64 } else { 48 };
    if !mode_ok
        || !req.resumable()
        || chunk_iters == 0
        || device.limits().max_color_attachment_bytes_per_sample < attach_need
    {
        return render_iter(device, queue, req);
    }
    let max_dim = device.limits().max_texture_dimension_2d;
    let w = req.width.clamp(1, max_dim);
    let h = req.height.clamp(1, max_dim);

    // A custom formula chunks through its OWN module's chunk pass: the fixed module's would read
    // formula id 1000 and iterate the Mandelbrot step.
    let shader = crate::shader_module_for(device, req.custom.as_deref());
    let iter_bgl = iter_bind_group_layout(device);
    let targets: usize = if fe { 4 } else { 3 };
    let state_bgl = crate::state_bind_group_layout_n(device, targets as u32);
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("chunked.layout"),
        bind_group_layouts: &[&iter_bgl, &state_bgl],
        push_constant_ranges: &[],
    });
    let chunk_formats = [ITER_FORMAT; 4];
    let chunk_pipeline = fullscreen_pipeline(
        device,
        &shader,
        &layout,
        if fe { "fs_iterate_chunk_fe" } else { "fs_iterate_chunk" },
        &chunk_formats[..targets],
        "chunked.chunk_pipeline",
    );
    let resolve_pipeline = fullscreen_pipeline(
        device, &shader, &layout, "fs_resolve", &[ITER_FORMAT, ITER_FORMAT],
        "chunked.resolve_pipeline",
    );

    let iter_uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("chunked.iter_uniform"),
        size: std::mem::size_of::<IterUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    // Modes 0/2 iterate against the reference orbit; direct mode binds a 1-slot placeholder. The
    // flattened BLA tree is appended after the orbit exactly as `render_iter` lays it out.
    // ⚠It must be uploaded even though mode 0 never traverses it: mode 2's chunk pass rebuilds its
    // BLA table every pass, and a chunk pass that ran with `bla_on = 0` and an empty tree is the
    // beta.101 e100 pathology verbatim — 0.04 Gsteps/s against the base pass's 174 in the same
    // frame, which reads as "chunking is slow" rather than "chunking is broken".
    let orbit_buf = req.upload_reference(device, queue)?;
    let counters_buf = crate::make_counters_buf(device);
    let counters_read = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("chunked.counters_read"),
        size: (crate::COUNTER_SLOTS * 4) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let iter_bg = make_iter_bg(device, &iter_bgl, &iter_uniform, &orbit_buf, &counters_buf);

    // Ping-pong state: pass k reads set (k % 2) and writes set ((k+1) % 2).
    let state = [
        crate::make_state_textures(device, [w, h], targets),
        crate::make_state_textures(device, [w, h], targets),
    ];
    let state_bg = [
        crate::make_state_bg(device, &state_bgl, &state[0]),
        crate::make_state_bg(device, &state_bgl, &state[1]),
    ];

    let split = |v: f64| -> (f32, f32) {
        let hi = v as f32;
        (hi, (v - hi as f64) as f32)
    };
    let (sxh, sxl) = split(req.span_mantissa.x / w as f64);
    let (syh, syl) = split(req.span_mantissa.y / h as f64);
    let mut iu = IterUniforms {
        step: [sxh, sxl, syh, syl],
        ref_offset: req.ref_offset.to_array(),
        center: req.center,
        julia_c: req.julia_c,
        res: [w as f32, h as f32],
        px_offset: [0.0, 0.0],
        max_iter: req.max_iter,
        orbit_len: req.orbit_len, // mode 0 iterates + rebases against the reference
        mode: req.mode,
        formula: req.formula,
        julia: req.julia,
        delta_exp: req.delta_exp,
        color_method: 0,
        stripe_freq: 1.0,
        trap_type: 0,
        aux_on: 0,
        sa_skip: req.sa_skip, // mode 0's SA seeding is replicated in the chunk init
        glitch_on: 0,
        sa_a: req.sa_a,
        sa_b: req.sa_b,
        sa_c: req.sa_c,
        sa_a_exp: req.sa_a_exp,
        sa_b_exp: req.sa_b_exp,
        sa_c_exp: req.sa_c_exp,
        // ⚠Passed through, not zeroed. It is inert for direct and mode 0 (neither branch reads it —
        // BLA lives in the mode-2 loop), and load-bearing for mode 2: see the orbit upload above.
        bla_on: req.bla_on,
        start_iter: 0,
        end_iter: 0,
        gather: [0; 2],
        tail: crate::tail_word(),
        rn: req.rn_uniform(),
        map: crate::StripMap::default(),
    };

    // One bounded submission per iteration range; poll-wait between them so each stays a short,
    // watchdog-friendly unit of GPU work (the entire point of this path).
    let chunks = req.max_iter.div_ceil(chunk_iters.max(1)).max(1);
    let mut read_set = 0usize;
    for k in 0..chunks {
        iu.start_iter = k * chunk_iters;
        iu.end_iter = iu.start_iter.saturating_add(chunk_iters).min(req.max_iter);
        queue.write_buffer(&iter_uniform, 0, bytemuck::bytes_of(&iu));
        let write_set = 1 - read_set;
        let t0 = std::time::Instant::now();
        let mut enc = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("chunked.enc") });
        {
            let attach = |v| Some(wgpu::RenderPassColorAttachment {
                view: v,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            });
            let attachments: Vec<_> = state[write_set].iter().map(|v| attach(v)).collect();
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("chunked.chunk_pass"),
                color_attachments: &attachments,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&chunk_pipeline);
            pass.set_bind_group(0, &iter_bg, &[]);
            pass.set_bind_group(1, &state_bg[read_set], &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit(std::iter::once(enc.finish()));
        // Bounded per-window barrier (device-loss-aware) — this is the deliberate one-submission-
        // per-poll shape the device-loss repro drives; a wedged window escapes instead of hanging.
        await_submitted(device, queue, None, None).into_result()?;
        passes.push(ChunkPassTiming {
            start_iter: iu.start_iter,
            end_iter: iu.end_iter,
            wall_ms: t0.elapsed().as_secs_f64() * 1000.0,
        });
        read_set = write_set;
    }

    // Resolve the settled state into the normal iteration G-buffer and read it back.
    let mk_out = |label: &str| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: ITER_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    };
    let main_tex = mk_out("chunked.iter_main");
    let main_view = main_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let aux_view = mk_out("chunked.iter_aux").create_view(&wgpu::TextureViewDescriptor::default());

    let bpp = 16u32;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let unpadded_bpr = w * bpp;
    let padded_bpr = unpadded_bpr.div_ceil(align) * align;
    let out_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("chunked.readback"),
        size: (padded_bpr * h) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut enc = device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("chunked.resolve_enc") });
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
            label: Some("chunked.resolve_pass"),
            color_attachments: &[attach(&main_view), attach(&aux_view)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&resolve_pipeline);
        pass.set_bind_group(0, &iter_bg, &[]);
        pass.set_bind_group(1, &state_bg[read_set], &[]);
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
    // Bounded final readback (device-loss-aware).
    await_readback(device, &rx, None, None).into_result()?;
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

    let counters = read_counters(device, queue, &counters_buf, &counters_read);
    // The doc on `max_dispatch_ms` has always promised "chunked paths measure per iteration
    // window"; this path returned 0.0 because it measured nothing. It does now.
    let max_dispatch_ms = passes.iter().map(|p| p.wall_ms).fold(0.0_f64, f64::max);
    Ok(ExportResult {
        width: w, height: h, ss: 1, pixels, iterate_ms: 0.0, color_ms: 0.0, counters,
        max_dispatch_ms, max_dispatch_work: 0, chunk_passes: 0, tiles_total: 0, tiles_chunked: 0,
        packs: 0,
        aux: Vec::new(),
    })
}

/// Run the shader's primitive self-test (`fs_gputest`, see the WGSL) and return the raw
/// `(width, height, rgba_f32)` result grid: one op family per row, one hashed input set per
/// column, up to four f32 results per texel. The CPU comparison lives in the app (it needs the
/// f64 / exact-EFT oracles); this just executes the very shader code the renderer uses on THIS
/// device and reads it back.
pub fn gputest(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(u32, u32, Vec<f32>), GpuError> {
    const W: u32 = 256;
    const H: u32 = 19; // rows 0–18 of `fs_gputest`
    let shader = shader_module(device);
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("gputest.layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });
    let pipeline =
        fullscreen_pipeline(device, &shader, &layout, "fs_gputest", &[ITER_FORMAT], "gputest.pipeline");
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("gputest.out"),
        size: wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: ITER_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let bpp = 16u32;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let unpadded_bpr = W * bpp;
    let padded_bpr = unpadded_bpr.div_ceil(align) * align;
    let out_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("gputest.readback"),
        size: (padded_bpr * H) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("gputest.enc"),
    });
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("gputest.pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
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
        pass.set_pipeline(&pipeline);
        pass.draw(0..3, 0..1);
    }
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &out_buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 },
    );
    queue.submit(std::iter::once(enc.finish()));
    let slice = out_buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    // Bounded readback (device-loss-aware) — the GPU-primitive test harness.
    await_readback(device, &rx, None, None).into_result()?;
    let data = slice.get_mapped_range();
    let mut pixels = vec![0.0_f32; (W * H * 4) as usize];
    for r in 0..H {
        let src = (r * padded_bpr) as usize;
        let row: &[f32] = bytemuck::cast_slice(&data[src..src + unpadded_bpr as usize]);
        pixels[(r * W * 4) as usize..((r + 1) * W * 4) as usize].copy_from_slice(row);
    }
    drop(data);
    out_buf.unmap();
    Ok((W, H, pixels))
}
