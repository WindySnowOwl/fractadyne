use super::*;
use fractadyne_farm::proto::Policy;

#[test]
fn frame_done_lines_parse_exactly_as_the_renderer_prints_them() {
    let sha = "e54f3cb027ffc5ea707912eb948fe9cbaca0ea2eaa451afbcc4ebec674cd2ccc";
    assert_eq!(
        parse_child_line(&format!("frame-done index=12 bytes=74101 sha256={sha} ms=32")),
        ChildLine::Done { index: 12, bytes: 74101, sha256: sha.into(), ms: 32 }
    );
    assert_eq!(
        parse_child_line("frame-failed index=3 reason=\"read back: denied\""),
        ChildLine::Failed { index: 3, reason: "read back: denied".into() }
    );
    // Anything else — progress, a damaged line, a short digest — is not a frame report.
    for other in ["  frame 10/19  (1s elapsed)", "frame-done index=x bytes=1 sha256=00 ms=1", "frame-done index=1 bytes=2 ms=3", &format!("frame-done index=1 bytes=2 sha256={} ms=3", &sha[..10])] {
        assert!(matches!(parse_child_line(other), ChildLine::Other(_)), "{other}");
    }
}

#[test]
fn gpu_facts_come_from_the_render_log() {
    let log = "[fd-start] [+ 0.001s] fractadyne …\n[fd-gpu] [+    0.979s] reference-orbit length cap = 7452444 samples (storage-binding limit 1073741824 B)\n[fd-wgpu] [+    0.985s] adapter: NVIDIA GeForce RTX 3080 · Vulkan · capability: TIMESTAMP_QUERY=true\n";
    let log = format!("{log}[fd-wgpu] [+    0.986s] driver: NVIDIA 581.42\n");
    assert_eq!(
        gpu_facts(&log),
        GpuFacts { adapter: Some("NVIDIA GeForce RTX 3080 · Vulkan".into()), driver: Some("NVIDIA 581.42".into()), orbit_len_cap: Some(7_452_444) }
    );
    assert_eq!(gpu_facts("nothing useful"), GpuFacts::default());
    assert_eq!(gpu_facts("[fd-wgpu] [+ 1s] driver: unknown\n").driver, None);
    assert_eq!(driver_text("NVIDIA", "581.42"), "NVIDIA 581.42");
    assert_eq!(driver_text("", " "), "unknown");
}

#[test]
fn gpus_differ_by_model_then_api_then_driver() {
    use fractadyne_farm::proto::GpuInfo;
    let g = |adapter: &str, driver: &str| GpuInfo { adapter: adapter.into(), driver: driver.into(), orbit_len_cap: 0 };
    let here = g("NVIDIA GeForce RTX 3080 · Vulkan", "NVIDIA 581.42");
    assert_eq!(gpu_difference(&here, &here), None);
    assert_eq!(gpu_difference(&here, &g("AMD Radeon RX 6800 XT · Vulkan", "AMD proprietary driver 25.9.1")), Some(GpuDiff::Model));
    assert_eq!(gpu_difference(&here, &g("NVIDIA GeForce RTX 3080 · Dx12", "NVIDIA 581.42")), Some(GpuDiff::Backend));
    assert_eq!(gpu_difference(&here, &g("NVIDIA GeForce RTX 3080 · Vulkan", "NVIDIA 576.02")), Some(GpuDiff::Driver));
    assert_eq!(gpu_difference(&here, &g("NVIDIA GeForce RTX 3080 · Vulkan", "")), None, "an unknown driver is not a mismatch");
    assert_eq!(gpu_text(&here), "NVIDIA GeForce RTX 3080 · Vulkan, driver NVIDIA 581.42");
}

fn bundle() -> Bundle {
    Bundle {
        name: "Gate".into(),
        script: "format_version = 2".into(),
        settings: RenderSettings::from_session(&fractadyne_state::SessionState::default()),
        anchors: None,
        orbit_len_cap: Some(928_000),
        width: 1920,
        height: 1080,
        fps: 30.0,
        ss: 2,
        prefix: "gate".into(),
        frames: 1801,
        share: false,
    }
}

fn policy() -> Policy {
    Policy { max_width: 3840, max_height: 2160, max_ss: 4, max_iter: 10_000_000 }
}

/// A client refuses a job outside its policy, naming the limit — never clamps it.
#[test]
fn a_job_outside_the_clients_policy_is_refused_by_name() {
    assert!(bundle().check(&policy(), Some(7_452_444)).is_ok());
    let cases: Vec<(Bundle, Option<u64>, &str)> = vec![
        (Bundle { width: 7680, height: 4320, ..bundle() }, None, "larger than this machine allows"),
        (Bundle { ss: 8, ..bundle() }, None, "supersampling"),
        (Bundle { prefix: "../x".into(), ..bundle() }, None, "prefix"),
        (Bundle { fps: f64::NAN, ..bundle() }, None, "frame rate"),
        (Bundle { frames: 0, ..bundle() }, None, "frame count"),
        (bundle(), Some(500_000), "reference samples"),
    ];
    for (b, cap, needle) in cases {
        let e = b.check(&policy(), cap).expect_err(needle);
        assert!(e.contains(needle), "{needle}: {e}");
    }
    let mut greedy = bundle();
    greedy.settings.max_iter = 5_000_000;
    let tight = Policy { max_iter: 1_000_000, ..policy() };
    assert!(greedy.check(&tight, None).unwrap_err().contains("iteration base"));
}

/// The self-check tour is a real, resolvable one-frame tour.
#[test]
fn the_self_check_tour_resolves_to_one_frame() {
    let pb = crate::scripting::parse_tour_text(SELF_CHECK_TOUR).expect("parses");
    assert_eq!(crate::scripting::tour_frame_count(pb.total, 1.0), 1);
}
