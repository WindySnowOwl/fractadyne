use super::*;

/// `life.wgsl` parses and validates with naga — every entry point, so a typo fails here rather than
/// at the first pipeline the app builds. (Stepping on a device is `check`'s, run below and by the
/// self-test's `life` rows.)
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

/// `life_display.wgsl` validates, and its counter slots are the iterate shader's: the live
/// normalization reads them as if an escape-time pass had written them. Its `LifeView` matches the
/// host's `LifeViewU`.
#[test]
fn the_display_shader_validates_and_shares_the_counter_slots() {
    use egui_wgpu::wgpu::naga;
    let src = DISPLAY_SOURCE;
    let module = naga::front::wgsl::parse_str(src).unwrap_or_else(|e| panic!("{}", e.emit_to_string(src)));
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(&module)
        .unwrap_or_else(|e| panic!("{}", e.emit_to_string(src)));
    for (name, slot) in
        [("CTR_ESC_MIN", crate::CTR_ESC_MIN), ("CTR_ESC_MAX", crate::CTR_ESC_MAX), ("CTR_ESC_COUNT", crate::CTR_ESC_COUNT)]
    {
        assert!(src.contains(&format!("const {name}: u32 = {slot}u;")), "{name} is slot {slot}");
    }
    assert_eq!(crate::ESC_COUNT_SUBSAMPLE, 16, "the display commits on the same 4x4 grid");
    let view = module.types.iter().find(|(_, t)| t.name.as_deref() == Some("LifeView")).expect("LifeView").1;
    let naga::TypeInner::Struct { span, .. } = view.inner else { panic!("a struct") };
    assert_eq!(span as usize, std::mem::size_of::<LifeViewU>());
}

/// The host's `Gen` record matches the shader's uniform: the 512-bit rule, then four u32.
#[test]
fn the_generation_record_has_the_shaders_size() {
    assert_eq!(GEN_SIZE, 16 * 4 + 4 * 4);
    assert!(GEN_SIZE <= GEN_STRIDE && GEN_STRIDE % 256 == 0);
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

/// Every device check, on this machine's GPU: `cargo test -p fractadyne-gpu --release --
/// --include-ignored` (needs a GPU; the self-test runs the same checks in the app).
#[test]
#[ignore = "needs a GPU"]
fn the_device_checks_pass() {
    let Some((device, queue)) = device() else {
        eprintln!("no GPU adapter: skipped");
        return;
    };
    let mut all = check::stepper_matches_cpu(&device, &queue);
    all.extend(check::tile_set_and_pool(&device, &queue));
    all.extend(check::known_facts(&device, &queue));
    all.extend(check::display(&device, &queue));
    let failed: Vec<String> = all
        .iter()
        .filter_map(|o| o.result.as_ref().err().map(|e| format!("{} ({}): {e}", o.name, o.params)))
        .collect();
    for o in &all {
        eprintln!("{} ({}): {}", o.name, o.params, o.result.as_ref().map_or_else(|e| format!("FAIL {e}"), |s| s.clone()));
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
    assert_eq!(all.len(), check::RULES.len() + 2 + 3 + 3);
}
