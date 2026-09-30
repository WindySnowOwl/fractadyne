//! Design validation for custom formulas (design/custom-formulas.md): how long does it take to turn
//! the renderer's WGSL into pipelines when the source CHANGES — the cost a generated per-formula
//! shader would pay. Each variant edits a constant the iterate entry points read (the escape radius),
//! so the backend compiler cannot reuse a cached binary for it. Headless (no window), Vulkan.
//!
//!   cargo run --release -p fractadyne-gpu --example shader_compile_bench

use std::time::Instant;

fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    let mut fut = std::pin::pin!(fut);
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    loop {
        match fut.as_mut().poll(&mut cx) {
            std::task::Poll::Ready(v) => return v,
            std::task::Poll::Pending => std::thread::yield_now(),
        }
    }
}

fn main() {
    use egui_wgpu::wgpu;
    let src = include_str!("../src/mandelbrot.wgsl");
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..Default::default()
    });
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .expect("no Vulkan adapter");
    let mut limits = wgpu::Limits::default().using_resolution(adapter.limits());
    // The chunk-state pipelines write 48/64 bytes per sample, as the app asks for.
    limits.max_color_attachment_bytes_per_sample = adapter.limits().max_color_attachment_bytes_per_sample;
    let (device, _queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: limits,
            memory_hints: wgpu::MemoryHints::default(),
        },
        None,
    ))
    .expect("device");
    println!("adapter: {} ({:?})", adapter.get_info().name, adapter.get_info().backend);
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() % 1_000_000)
        .unwrap_or(0);
    let two = [wgpu::TextureFormat::Rgba32Float, wgpu::TextureFormat::Rgba32Float];
    // Variant 0: the shipped source (the driver may have it cached from an earlier run).
    // 1-2: every formula, a never-used escape radius (a distinct f32, so no cached binary fits).
    // 3-6: the same, with the formula id a CONSTANT (Mandelbrot, Multibrot 3, Burning Ship,
    //      Phoenix) instead of the uniform — what a shader generated for one formula would compile.
    // 7-9: the modules `custom::build` actually generates (Mandelbrot, Burning Ship, a three-phase
    //      hybrid), which also drop the perturbation paths. Only `fs_iterate` is built from them.
    use fractadyne_core::{formula as f, ir};
    let custom = |ids: &[u32]| {
        let phases = ids.iter().map(|&id| ir::builtin_step(id).unwrap()).collect();
        fractadyne_gpu::custom::build(&ir::Formula::new(phases).unwrap(), &[]).unwrap().source
    };
    for variant in 0..10u32 {
        let r = 300 + (stamp % 5000) as u32 * 8 + variant; // an integer: exact in f32
        let base = match variant {
            7 => custom(&[f::MANDELBROT]),
            8 => custom(&[f::BURNING_SHIP]),
            9 => custom(&[f::MANDELBROT, f::BURNING_SHIP, f::MULTIBROT3]),
            _ => src.to_string(),
        };
        let mut wgsl = if variant == 0 {
            base
        } else {
            base.replace("let bail2 = 256.0 * 256.0;", &format!("let bail2 = {r}.0 * {r}.0;"))
        };
        let fixed = match variant {
            3 => Some(0u32),
            4 => Some(1),
            5 => Some(5),
            6 => Some(8),
            _ => None,
        };
        if let Some(f) = fixed {
            wgsl = wgsl.replace("iu.formula", &format!("{f}u"));
        }
        let t0 = Instant::now();
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bench"),
            source: wgpu::ShaderSource::Wgsl(wgsl.into()),
        });
        let t_module = t0.elapsed().as_secs_f64() * 1e3;
        let what = match (variant, fixed) {
            (7..=9, _) => "generated custom".to_string(),
            (_, Some(f)) => format!("formula {f} const"),
            _ => "all formulas".to_string(),
        };
        let mut line = format!("variant {variant} ({what}): module {t_module:7.1} ms");
        let entries: &[(&str, usize)] = if variant >= 7 {
            &[("fs_iterate", 2)]
        } else {
            &[("fs_iterate", 2), ("fs_iterate_chunk", 3), ("fs_iterate_chunk_fe", 4)]
        };
        for &(entry, targets) in entries {
            let formats: Vec<Option<wgpu::ColorTargetState>> = (0..targets)
                .map(|i| Some(wgpu::ColorTargetState {
                    format: two[i.min(1)],
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }))
                .collect();
            let t1 = Instant::now();
            let _p = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(entry),
                layout: None,
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some(entry),
                    targets: &formats,
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });
            line += &format!("  {entry} {:7.1} ms", t1.elapsed().as_secs_f64() * 1e3);
        }
        println!("{line}");
    }
}
