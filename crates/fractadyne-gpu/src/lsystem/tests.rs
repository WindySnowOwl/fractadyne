use super::*;

/// `lsystem.wgsl` validates with naga, its counter slots and subsampling grid are the iterate
/// shader's, and its `LView` matches the host's `LViewU`.
#[test]
fn the_segment_shader_validates_and_shares_the_counter_slots() {
    use egui_wgpu::wgpu::naga;
    let module = naga::front::wgsl::parse_str(SOURCE).unwrap_or_else(|e| panic!("{}", e.emit_to_string(SOURCE)));
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(&module)
        .unwrap_or_else(|e| panic!("{}", e.emit_to_string(SOURCE)));
    let names: Vec<&str> = module.entry_points.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["vs_segment", "fs_segment"]);
    for (name, slot) in
        [("CTR_ESC_MIN", crate::CTR_ESC_MIN), ("CTR_ESC_MAX", crate::CTR_ESC_MAX), ("CTR_ESC_COUNT", crate::CTR_ESC_COUNT)]
    {
        assert!(SOURCE.contains(&format!("const {name}: u32 = {slot}u;")), "{name} is slot {slot}");
    }
    assert_eq!(crate::ESC_COUNT_SUBSAMPLE, 16);
    assert!(SOURCE.contains("(tx.x & 3) == 0 && (tx.y & 3) == 0"), "commits on the iterate passes' 4x4 grid");
    let view = module.types.iter().find(|(_, t)| t.name.as_deref() == Some("LView")).expect("LView").1;
    let naga::TypeInner::Struct { span, .. } = view.inner else { panic!("a struct") };
    assert_eq!(span as usize, std::mem::size_of::<LViewU>());
    assert_eq!(std::mem::size_of::<SegmentInstance>(), 20, "the vertex layout's stride");
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
            label: Some("lsystem.test"),
            required_features: wgpu::Features::empty(),
            required_limits: limits,
            memory_hints: wgpu::MemoryHints::default(),
        },
        None,
    ))
    .ok()
}

/// The device checks, on this machine's GPU: `cargo test -p fractadyne-gpu --release --
/// --include-ignored` (needs a GPU; the self-test runs the same checks in the app).
#[test]
#[ignore = "needs a GPU"]
fn the_device_checks_pass() {
    let Some((device, queue)) = device() else {
        eprintln!("no GPU adapter: skipped");
        return;
    };
    let all = check::coverage(&device, &queue);
    for o in &all {
        eprintln!("{} ({}): {}", o.name, o.params, o.result.as_ref().map_or_else(|e| format!("FAIL {e}"), |s| s.clone()));
    }
    let failed: Vec<String> =
        all.iter().filter_map(|o| o.result.as_ref().err().map(|e| format!("{} ({}): {e}", o.name, o.params))).collect();
    assert!(failed.is_empty(), "{}", failed.join("\n"));
    assert_eq!(all.len(), 4);
}

/// The CPU model is the rule it says: a texel on a segment takes its value, one far from every
/// segment is interior, and the later of two overlapping segments wins.
#[test]
fn the_model_is_the_rule() {
    let seg = |a: [f32; 2], b: [f32; 2], value| SegmentInstance { a, b, value };
    let frame = LSystemFrame {
        segments: Arc::new(vec![seg([-10.0, 0.0], [10.0, 0.0], 0.25), seg([0.0, -10.0], [0.0, 10.0], 0.75)]),
        segments_id: 1,
        scale: 1.0,
        offset: [0.0, 0.0],
        width: 3.0,
    };
    let m = check::model(&frame, [40, 40], 1);
    let at = |x: usize, y: usize| m[y * 40 + x];
    // Texel (25, 19): centre (25.5, 19.5) = view (5.5, 0.5): on the horizontal line only.
    assert_eq!(at(25, 19), Some(0.25));
    // The crossing: the vertical line was drawn last.
    assert_eq!(at(19, 19), Some(0.75));
    assert_eq!(at(5, 5), Some(-1.0));
    // Texel (25, 21): centre at view (5.5, −1.5), exactly a half-width from the line — on its
    // edge, left to rounding.
    assert_eq!(at(25, 21), None);
}
