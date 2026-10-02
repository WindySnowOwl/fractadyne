use super::*;

/// `life.wgsl` parses and validates with naga — every entry point, so a typo fails here rather than
/// at the first pipeline the app builds. (Stepping on a device is the self-test's `life-gpu` rows.)
#[test]
fn the_life_shader_validates() {
    use egui_wgpu::wgpu::naga;
    let module = naga::front::wgsl::parse_str(SOURCE).unwrap_or_else(|e| panic!("{}", e.emit_to_string(SOURCE)));
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(&module)
        .unwrap_or_else(|e| panic!("{}", e.emit_to_string(SOURCE)));
    let names: Vec<&str> = module.entry_points.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["cs_step", "cs_stats"]);
}

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

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    }))?;
    let limits = wgpu::Limits::default().using_resolution(adapter.limits());
    block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("life.test"),
            required_features: wgpu::Features::empty(),
            required_limits: limits,
            memory_hints: wgpu::MemoryHints::default(),
        },
        None,
    ))
    .ok()
}

fn soup(u: &mut Universe, seed: u64, x: i64, y: i64, w: i64, h: i64, states: u16) {
    let mut s = seed;
    for j in 0..h {
        for i in 0..w {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            if s % 100 < 35 {
                u.set(x + i, y + j, 1 + ((s >> 20) % u64::from(states - 1)) as u8);
            }
        }
    }
}

/// On a real device: the GPU stepper equals the CPU tile stepper cell for cell, through tile
/// growth and freeing, for every kind of rule and topology, with the breach tripwire silent.
/// `cargo test -p fractadyne-gpu --release -- --ignored` (needs a GPU; the self-test runs the same
/// check in the app).
#[test]
#[ignore = "needs a GPU"]
fn the_gpu_stepper_matches_the_cpu_one() {
    let Some((device, queue)) = device() else {
        eprintln!("no GPU adapter: skipped");
        return;
    };
    let mut gpu = LifeGpu::new(&device, 4096);
    let rules = ["B3/S23", "B36/S23", "B2/S", "B2/S/C3", "B2/S345/C4", "B3/S2-i34q", "B2-a/S12", "B2/S013V", "B0/S", "B01/S1"];
    let topologies = [
        Topology::Plane,
        Topology::Torus { width: 128, height: 64 },
        Topology::Bounded { x: -37, y: 11, width: 100, height: 70 },
    ];
    for (k, r) in rules.iter().enumerate() {
        let rule = Rule::parse(r).unwrap();
        for (t, &topology) in topologies.iter().enumerate() {
            let mut cpu = Universe::new(rule.clone(), topology).unwrap();
            soup(&mut cpu, 3 + 11 * k as u64 + t as u64, -20, 5, 48, 40, rule.states());
            gpu.load(&queue, &cpu).unwrap();
            for step in [1u64, 2, 13, 16, 17, 40, 111] {
                cpu.step_n(step);
                gpu.step(&device, &queue, step).unwrap();
                let back = gpu.download(&device, &queue).unwrap();
                assert_eq!(back.generation(), cpu.generation());
                assert_eq!(back.background(), cpu.background(), "{r} on {topology:?}");
                let (a, b) = (back.cells(), cpu.cells());
                if a != b {
                    let i = a.iter().zip(&b).position(|(p, q)| p != q).unwrap_or(a.len().min(b.len()));
                    panic!(
                        "{r} on {topology:?} at generation {}: GPU {} cells, CPU {}; first difference {:?} vs {:?}",
                        cpu.generation(),
                        a.len(),
                        b.len(),
                        a.get(i),
                        b.get(i)
                    );
                }
                assert_eq!(gpu.population(), cpu.population(), "{r} on {topology:?}: the stats pass");
            }
            assert_eq!(gpu.breaches(), 0, "{r} on {topology:?}: the halo was breached");
        }
    }
}

/// A glider driven 1,000 cells across tile corners keeps its five cells and leaves no tiles behind;
/// a soup that outgrows a small pool stops with `PoolFull`, its last state still exact.
#[test]
#[ignore = "needs a GPU"]
fn the_gpu_tile_set_follows_the_pattern_and_reports_a_full_pool() {
    let Some((device, queue)) = device() else {
        eprintln!("no GPU adapter: skipped");
        return;
    };
    let mut cpu = Universe::new(Rule::life(), Topology::Plane).unwrap();
    for (x, y) in [(61, 60), (62, 61), (60, 62), (61, 62), (62, 62)] {
        cpu.set(x, y, 1);
    }
    let mut gpu = LifeGpu::new(&device, 64);
    gpu.load(&queue, &cpu).unwrap();
    let mut most = 0;
    for _ in 0..250 {
        gpu.step(&device, &queue, 16).unwrap();
        most = most.max(gpu.tile_count());
    }
    cpu.step_n(4000);
    assert_eq!(gpu.download(&device, &queue).unwrap().cells(), cpu.cells());
    assert!(most <= 16, "the glider's tiles and their halo, saw {most}");
    assert_eq!(gpu.breaches(), 0);

    // Seeds explodes at the speed of light: 9 tiles cannot hold it for long.
    let mut seeds = Universe::new(Rule::parse("B2/S").unwrap(), Topology::Plane).unwrap();
    seeds.set(0, 0, 1);
    seeds.set(1, 0, 1);
    let mut small = LifeGpu::new(&device, 9);
    small.load(&queue, &seeds).unwrap();
    let mut stopped = None;
    for _ in 0..20 {
        if let Err(e) = small.step(&device, &queue, 16) {
            stopped = Some(e);
            break;
        }
    }
    assert!(matches!(stopped, Some(LifeGpuError::PoolFull { capacity: 9, .. })), "{stopped:?}");
    seeds.step_n(small.generation());
    assert_eq!(small.download(&device, &queue).unwrap().cells(), seeds.cells(), "exact up to where it stopped");
}

/// The host's `Gen` record matches the shader's uniform: the 512-bit rule, then four u32.
#[test]
fn the_generation_record_has_the_shaders_size() {
    assert_eq!(GEN_SIZE, 16 * 4 + 4 * 4);
    assert!(GEN_SIZE <= GEN_STRIDE && GEN_STRIDE % 256 == 0);
}
