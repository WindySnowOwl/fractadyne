use super::*;
use fractadyne_farm::proto::Policy;

#[test]
fn one_commit_is_one_commit_whatever_its_stamp_length() {
    assert!(same_commit("29988ec", "29988ecf"), "Windows and a Linux clone of the same commit");
    assert!(same_commit("29988ecf", "29988ec"));
    assert!(same_commit("29988ec", "29988ec"));
    assert!(!same_commit("29988ec", "29988ed"), "another commit");
    assert!(!same_commit("29988ec", "29988ec-dirty"), "a dirty build is not the commit");
    assert!(same_commit("29988ec-dirty", "29988ecf-dirty"));
    assert!(!same_commit("29988ec-archive", "29988ec"), "an archive build cannot vouch for its tree");
    assert!(!same_commit("2998", "29988ec"), "too short to name anything");
}

#[test]
fn a_machine_is_in_use_only_while_touched_recently_and_unlocked() {
    use std::time::Duration;
    let need = Duration::from_secs(300);
    assert!(in_use(need, Some(Duration::from_secs(10)), false));
    assert!(!in_use(need, Some(Duration::from_secs(300)), false), "idle for exactly the time asked is idle");
    assert!(!in_use(need, Some(Duration::from_secs(10)), true), "a locked screen is nobody's");
    assert!(!in_use(need, None, false), "an unreadable idle time counts as idle (the field agent's rule)");
    assert!(!in_use(Duration::ZERO, Some(Duration::ZERO), false), "--when-idle 0 never waits");
}

#[test]
fn frame_done_lines_parse_exactly_as_the_renderer_prints_them() {
    let sha = "e54f3cb027ffc5ea707912eb948fe9cbaca0ea2eaa451afbcc4ebec674cd2ccc";
    assert_eq!(
        parse_child_line(&format!("frame-done index=12 bytes=74101 sha256={sha} ms=32")),
        ChildLine::Done { index: 12, bytes: 74101, sha256: sha.into(), ms: 32, reference: None }
    );
    // Where the frame's reference came from: one of the known words, else not said.
    let with = |r: &str| parse_child_line(&format!("frame-done index=12 bytes=74101 sha256={sha} ms=32 ref={r}"));
    assert!(matches!(with("fresh"), ChildLine::Done { reference: Some(ref r), .. } if r == "fresh"));
    assert!(matches!(with("cache"), ChildLine::Done { reference: Some(ref r), .. } if r == "cache"));
    assert!(matches!(with("../etc"), ChildLine::Done { reference: None, .. }));
    assert_eq!(
        parse_child_line("frame-failed index=3 reason=\"read back: denied\""),
        ChildLine::Failed { index: 3, reason: "read back: denied".into() }
    );
    // The reference build advancing (`reference_work_lines`' format).
    assert_eq!(parse_child_line("reference-work steps=1645846"), ChildLine::ReferenceWork { steps: 1_645_846 });
    // Anything else — progress, a damaged line, a short digest — is not a frame report.
    for other in [
        "  frame 10/19  (1s elapsed)",
        "frame-done index=x bytes=1 sha256=00 ms=1",
        "frame-done index=1 bytes=2 ms=3",
        &format!("frame-done index=1 bytes=2 sha256={} ms=3", &sha[..10]),
        "reference-work steps=-4",
        "reference-work",
    ] {
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
        sharing: true,
        strips: None,
    }
}

/// A zoom video's strip job: the user's 1080p dive at 2x (`design/zoom-video.md` §8.5).
fn strip_bundle() -> Bundle {
    Bundle {
        name: "zoom-video".into(),
        script: String::new(),
        width: 13842,
        height: 1529,
        fps: 1.0,
        ss: 1,
        prefix: "strip".into(),
        frames: 132,
        strips: Some(StripJob {
            manifest: r#"{"format":"fractadyne-expmap","dive":{"strips":132}}"#.into(),
            center_re: "-1.193675944325418439658220453810454026977217510559150868117e-2".into(),
            center_im: "7.986812295926137389257603042692185301076332131668760934277e-1".into(),
            zoom_log2: 120.74482993798824,
            start_log2: 0.0,
            frame_width: 1920,
            frame_height: 1080,
            oversample: 2.0,
            cap_fixed: false,
            iter: None,
            bla: true,
        }),
        ..bundle()
    }
}

/// A strip job is checked as a dive: a centre that is a number, depths and sizes the app takes,
/// a manifest that is one, the `strip` prefix and no script — and a strip's TEXELS against the
/// policy, not its long side (a 1080p strip at 3x is 20763 wide, under a 16384×16384 policy).
#[test]
fn a_strip_job_is_checked_as_a_dive() {
    let wide = Policy { max_width: 16384, max_height: 16384, ..policy() };
    assert!(strip_bundle().check(&wide, None).is_ok());
    assert!(Bundle { width: 20763, height: 2294, ..strip_bundle() }.check(&wide, None).is_ok(), "a long strip is not a large one");
    assert!(strip_bundle().check(&Policy { max_width: 1920, max_height: 1080, ..policy() }, None).unwrap_err().contains("strips are larger"));
    let with = |f: &dyn Fn(&mut StripJob)| {
        let mut b = strip_bundle();
        f(b.strips.as_mut().unwrap());
        b
    };
    let cases: Vec<(Bundle, &str)> = vec![
        (with(&|s| s.center_re = "1; del *".into()), "centre"),
        (with(&|s| s.center_im = String::new()), "centre"),
        (with(&|s| s.zoom_log2 = f64::NAN), "depths"),
        (with(&|s| s.start_log2 = 200.0), "depths"),
        (with(&|s| s.frame_width = 0), "video frames"),
        (with(&|s| s.oversample = 9.0), "oversampling"),
        (with(&|s| s.iter = Some(20_000_000)), "iteration count"),
        (with(&|s| s.manifest = "not json".into()), "manifest"),
        (Bundle { script: "format_version = 2".into(), ..strip_bundle() }, "script"),
        (Bundle { prefix: "frame".into(), ..strip_bundle() }, "prefix"),
    ];
    for (b, needle) in cases {
        let e = b.check(&wide, None).expect_err(needle);
        assert!(e.contains(needle), "{needle}: {e}");
    }
}

/// A client renders its strips as `--render-expmap` of the dive, A-B inclusive from the half-open
/// run; the controller plans the dive with the same flags and no range.
#[test]
fn a_strip_run_is_the_dives_render_expmap() {
    let s = strip_bundle().strips.unwrap();
    let out = std::path::Path::new("C:/work/strips");
    let a = s.child_args(out, 40, 44);
    let pair = |a: &[String], k: &str| a.iter().position(|x| x == k).map(|i| a[i + 1].clone());
    assert_eq!(a[0], "--render-expmap");
    assert_eq!(pair(&a, "--expmap-strips").as_deref(), Some("40-43"), "Assign's [40, 44) is strips 40 to 43");
    let i = a.iter().position(|x| x == "--center").unwrap();
    assert_eq!((a[i + 1].as_str(), a[i + 2].as_str()), (s.center_re.as_str(), s.center_im.as_str()));
    assert_eq!(pair(&a, "--zoom-log2").as_deref(), Some("120.74482993798824"), "the depth round-trips exactly");
    assert_eq!(pair(&a, "--size").as_deref(), Some("1920x1080"));
    assert_eq!(pair(&a, "--expmap-oversample").as_deref(), Some("2"));
    assert!(a.contains(&"--bla".to_string()) && a.contains(&"--farm-child".to_string()));
    for absent in ["--expmap-start", "--expmap-cap", "--iter", "--no-bla", "--expmap-plan"] {
        assert!(!a.iter().any(|x| x == absent), "{absent}");
    }
    let s2 = StripJob { start_log2: 3.5, cap_fixed: true, iter: Some(50_000), bla: false, ..s.clone() };
    let a = s2.child_args(out, 7, 8);
    assert_eq!(pair(&a, "--expmap-strips").as_deref(), Some("7-7"));
    assert_eq!(pair(&a, "--expmap-start").as_deref(), Some("3.5"));
    assert_eq!(pair(&a, "--expmap-cap").as_deref(), Some("fixed"));
    assert_eq!(pair(&a, "--iter").as_deref(), Some("50000"));
    assert!(a.contains(&"--no-bla".to_string()) && !a.contains(&"--bla".to_string()));
    let p = s.plan_args(out);
    assert!(p.contains(&"--expmap-plan".to_string()));
    for absent in ["--expmap-strips", "--farm-child"] {
        assert!(!p.iter().any(|x| x == absent), "the plan renders no strip: {absent}");
    }
    // Every flag either line uses is one the command line knows.
    let known = crate::cli::known_long_flags();
    for f in a.iter().chain(p.iter()).filter(|x| x.starts_with("--")) {
        assert!(known.contains(f.as_str()), "{f} is not in the command-line reference");
    }
    assert_eq!(strip_bundle().unit_file_name(45), "strip_000045.exr", "the name --render-expmap writes");
    assert_eq!(bundle().unit_file_name(45), "gate_00045.png");
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
