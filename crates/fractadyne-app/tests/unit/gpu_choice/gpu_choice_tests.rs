use super::*;
use eframe::wgpu::Backend;

fn rows() -> Vec<(String, Backend)> {
    vec![
        ("NVIDIA GeForce RTX 3080".into(), Backend::Gl),
        ("NVIDIA GeForce RTX 3080".into(), Backend::Vulkan),
        ("AMD Radeon(TM) Graphics".into(), Backend::Vulkan),
        ("AMD Radeon RX 6800 XT".into(), Backend::Vulkan),
    ]
}

#[test]
fn a_number_is_the_listing_s_own_1_based_number() {
    assert_eq!(pick("1", &rows()), Ok(0));
    assert_eq!(pick("4", &rows()), Ok(3));
    assert_eq!(pick(" 2 ", &rows()), Ok(1));
    assert!(pick("0", &rows()).is_err(), "numbered from 1");
    assert!(pick("5", &rows()).unwrap_err().contains("4 adapter(s)"));
    // Three digits or more is a model number, part of a name.
    assert_eq!(pick("6800", &rows()), Ok(3));
    assert_eq!(pick("3080", &rows()), Ok(1));
}

#[test]
fn a_name_matches_any_part_case_insensitively_and_prefers_vulkan() {
    // Both entries of the 3080 match; the Vulkan one is taken although GL is listed first.
    assert_eq!(pick("rtx 3080", &rows()), Ok(1));
    assert_eq!(pick("6800", &rows()), Ok(3));
    // Several Vulkan matches: the first.
    assert_eq!(pick("amd", &rows()), Ok(2));
    assert!(pick("intel", &rows()).unwrap_err().contains("no adapter"));
    assert!(pick("", &rows()).is_err());
}

#[test]
fn the_window_reads_the_cards_back_from_the_listing() {
    use eframe::wgpu::DeviceType;
    let row = |name: &str, backend, kind| Row { name: name.into(), backend, kind, driver: "a driver · 1.2".into() };
    let rows = vec![
        row("NVIDIA GeForce RTX 3080", Backend::Vulkan, DeviceType::DiscreteGpu),
        row("AMD Radeon(TM) Graphics", Backend::Vulkan, DeviceType::IntegratedGpu),
        row("llvmpipe (LLVM 17.0.6, 256 bits)", Backend::Vulkan, DeviceType::Cpu),
        row("NVIDIA GeForce RTX 3080/PCIe/SSE2", Backend::Gl, DeviceType::Other),
    ];
    // The numbers are the listing's own: the software and OpenGL entries are left out, not renumbered.
    assert_eq!(cards_in_listing(&listing(&rows)), vec![(1, "NVIDIA GeForce RTX 3080".to_string()), (2, "AMD Radeon(TM) Graphics".to_string())]);
    assert!(cards_in_listing("No graphics adapter found (Vulkan or OpenGL).\n").is_empty());
}

#[test]
fn the_flag_without_a_value_is_an_error_not_any_adapter() {
    let a = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(spec(&a(&["--render-tour", "t.toml"])), Ok(None));
    assert_eq!(spec(&a(&["--adapter", "2", "--render-tour", "t.toml"])), Ok(Some("2".into())));
    assert!(spec(&a(&["--render-tour", "t.toml", "--adapter"])).is_err());
    assert!(spec(&a(&["--adapter", "--render-tour", "t.toml"])).is_err());
}

#[test]
fn a_vulkan_1_0_loader_is_left_out_and_a_1_1_one_kept() {
    // Venus (2026-11-10): the 2016 loader that crashed the app listing cards.
    let venus = loader::Loader { path: r"C:\WINDOWS\system32\vulkan-1.dll".into(), version: Some([1, 0, 26, 0]), api_1_1: false };
    let why = venus.too_old().expect("a Vulkan 1.0 loader is left out");
    assert!(why.contains(r"system32\vulkan-1.dll") && why.contains("version 1.0.26.0"), "{why}");
    assert!(why.contains("OpenGL") && why.contains("WGPU_BACKEND=vulkan"), "says what runs instead, and the way back: {why}");
    // The listing's readers must not take the note for a card (`cards_in_listing`).
    assert!(cards_in_listing(&format!("Vulkan is left out: {why}\n")).is_empty());
    assert_eq!(venus.line(), r"C:\WINDOWS\system32\vulkan-1.dll 1.0.26.0 (Vulkan 1.0)");
    let current = loader::Loader { version: Some([1, 3, 290, 0]), api_1_1: true, ..venus.clone() };
    assert_eq!(current.too_old(), None);
    assert!(current.line().ends_with("1.3.290.0 (Vulkan 1.1 or later)"), "{}", current.line());
    let unstated = loader::Loader { version: None, ..venus };
    assert!(unstated.too_old().expect("still 1.0").contains("no version stated"));
}

#[test]
fn this_machine_s_loader_reads_without_crashing() {
    // No loader (a machine without Vulkan) is `None`; one that is found names its file.
    if let Some(l) = loader::read() {
        assert!(l.path.to_lowercase().ends_with("vulkan-1.dll"), "{l:?}");
        if l.api_1_1 {
            assert_eq!(l.too_old(), None, "{l:?}");
            assert!(l.version.is_none_or(|v| (v[0], v[1]) >= (1, 1)), "a 1.1 loader's file says 1.1 or later: {l:?}");
        }
    }
}

#[test]
fn a_vulkan_failure_reads_back_and_applies_only_to_its_build_and_loader() {
    let f = fallback::Failure {
        app: "0.3.0-beta.27 (build 1, gabc)".into(),
        loader: r"C:\WINDOWS\SYSTEM32\vulkan-1.dll 1.4.363.1 (Vulkan 1.1 or later)".into(),
        adapter: "GeForce GT 650M".into(),
        reason: "WGPU error: Parent device is lost".into(),
        when: "2026-11-10 14:40".into(),
        unexplained: String::new(),
    };
    let text = fallback::render(&f).expect("renders");
    assert!(text.starts_with('#'), "the file says what it is");
    assert_eq!(fallback::parse(&text), Some(f.clone()));
    assert!(fallback::applies(&f, &f.app, &f.loader));
    assert!(!fallback::applies(&f, "0.3.0-beta.28 (build 2, gdef)", &f.loader), "a new build tries Vulkan again");
    assert!(!fallback::applies(&f, &f.app, r"C:\WINDOWS\SYSTEM32\vulkan-1.dll 1.4.400.0"), "a new loader too");
}

#[test]
fn a_damaged_or_oversized_failure_file_is_ignored() {
    assert_eq!(fallback::parse("not toml at all ["), None);
    assert_eq!(fallback::parse("app = 1"), None);
    let long = "x".repeat(2000);
    let text = format!("app = \"{long}\"\nloader = \"l\"\nadapter = \"a\"\nreason = \"r\"\n");
    assert_eq!(fallback::parse(&text), None);
    // `when` may be missing (an older file): read as empty.
    let f = fallback::parse("app = \"a\"\nloader = \"l\"\nadapter = \"c\"\nreason = \"r\"\n").expect("reads");
    assert_eq!(f.when, "");
    assert_eq!(f.unexplained, "", "a file without it is in force, as part 1 wrote them");
}

/// A dead session's marker, as `diag::marker_body` writes it, then what `mark_panicked` appends.
fn marker(phase: &str, api: Option<&str>, extra: &[&str]) -> String {
    let mut m = format!("{APP}\nstarted 2026-11-10 21:00:00Z\npid 4242\nphase {phase}\n");
    if let Some(a) = api {
        m.push_str(&format!("api {a}\ncard NVIDIA GeForce GT 650M\n"));
    }
    for l in extra {
        m.push_str(l);
        m.push('\n');
    }
    m
}

const APP: &str = "fractadyne 0.3.0-beta.27 (build 7, gabc)";
const LOADER: &str = r"C:\WINDOWS\SYSTEM32\vulkan-1.dll 1.4.363.1 (Vulkan 1.1 or later)";
const CRASHED: [&str; 2] = [
    "panicked crash-20261110-210001.txt",
    "what a native crash: access violation (0xc0000005) reading address 0x10, in the graphics driver: nvoglv64.dll+0x1a2b, on thread 'main'",
];

fn judge(m: &str, saved: Option<&fallback::Failure>) -> fallback::Verdict {
    fallback::judge(m, APP, LOADER, saved, "2026-11-10 21:01")
}

#[test]
fn a_reported_crash_at_start_up_on_vulkan_moves_the_next_start_to_opengl() {
    let fallback::Verdict::Switch(f) = judge(&marker("preparing-shaders", Some("Vulkan"), &CRASHED), None) else {
        panic!("a reported crash switches at once")
    };
    assert_eq!((f.app.as_str(), f.loader.as_str(), f.adapter.as_str()), (APP, LOADER, "NVIDIA GeForce GT 650M"));
    assert_eq!(f.unexplained, "", "in force");
    assert!(f.reason.starts_with("a native crash: access violation") && f.reason.ends_with(", at start-up, while preparing the graphics shaders"), "{}", f.reason);
    // The same while starting the device; a panic reads the same way.
    let panicked = ["panicked crash-x.txt", "what a panic: Validation Error"];
    assert!(matches!(judge(&marker("starting-device", Some("Vulkan"), &panicked), None), fallback::Verdict::Switch(_)));
    // Written and read back, it is in force for this build and loader.
    let back = fallback::parse(&fallback::render(&f).unwrap()).unwrap();
    assert!(back.unexplained.is_empty() && fallback::applies(&back, APP, LOADER));
}

#[test]
fn only_this_build_at_start_up_on_an_api_it_chose_by_itself_counts() {
    use fallback::Verdict::Nothing;
    // Start-up was over: a crash later is not start-up's.
    assert_eq!(judge(&marker("running", Some("Vulkan"), &CRASHED), None), Nothing);
    // Another build's start-up says nothing about this one.
    let other = marker("preparing-shaders", Some("Vulkan"), &CRASHED).replacen(APP, "fractadyne 0.3.0-beta.26 (build 6, gdef)", 1);
    assert_eq!(judge(&other, None), Nothing);
    // The person chose the API (WGPU_BACKEND).
    assert_eq!(judge(&marker("preparing-shaders", Some("Vulkan"), &["backend-env vulkan", CRASHED[0], CRASHED[1]]), None), Nothing);
    // No card yet: the API is not known.
    assert_eq!(judge(&marker("opening-window", None, &CRASHED), None), Nothing);
    // OpenGL by choice of wgpu (no Vulkan here): nothing to fall back to.
    assert_eq!(judge(&marker("preparing-shaders", Some("OpenGL"), &CRASHED), None), Nothing);
    // A marker from before the phase lines, or a damaged one.
    assert_eq!(judge(&format!("{APP}\nstarted x\npid 1\n"), None), Nothing);
    assert_eq!(judge("", None), Nothing);
}

#[test]
fn an_end_with_no_crash_report_needs_a_second_in_a_row_at_the_same_step() {
    // ⚠A person ending a frozen window during a long compile looks the same from outside.
    let m = marker("preparing-shaders", Some("Vulkan"), &[]);
    let fallback::Verdict::Note(first) = judge(&m, None) else { panic!("noted, not in force") };
    assert_eq!(first.unexplained, "preparing-shaders");
    // Saved, a noted end is not in force: `remembered` leaves it out (it reads `unexplained`).
    let text = fallback::render(&first).unwrap();
    assert!(text.starts_with("# A start-up on Vulkan ended here with no crash report"), "its header says it is not in force: {text}");
    let back = fallback::parse(&text).unwrap();
    assert_eq!(back, first);
    // The second, at the same step: in force.
    let fallback::Verdict::Switch(f) = judge(&m, Some(&first)) else { panic!("twice in a row switches") };
    assert_eq!(f.unexplained, "");
    assert!(f.reason.contains("twice in a row") && f.reason.contains("no crash report"), "{}", f.reason);
    // At another step: noted again, at that step.
    let fallback::Verdict::Note(other) = judge(&marker("starting-device", Some("Vulkan"), &[]), Some(&first)) else {
        panic!("a different step starts the count again")
    };
    assert_eq!(other.unexplained, "starting-device");
    // A noted end from another build does not count.
    let old = fallback::Failure { app: "fractadyne 0.3.0-beta.26 (build 6, gdef)".into(), ..first.clone() };
    assert!(matches!(judge(&m, Some(&old)), fallback::Verdict::Note(_)));
}

#[test]
fn opengl_put_in_vulkan_s_place_that_crashes_too_forgets_the_failure() {
    let saved = fallback::Failure {
        app: APP.into(),
        loader: LOADER.into(),
        adapter: "NVIDIA GeForce GT 650M".into(),
        reason: "r".into(),
        when: String::new(),
        unexplained: String::new(),
    };
    let on_gl = |extra: &[&str]| marker("preparing-shaders", Some("OpenGL"), &[&["vulkan-left-out"][..], extra].concat());
    assert_eq!(judge(&on_gl(&CRASHED), Some(&saved)), fallback::Verdict::Forget);
    // With no crash report (perhaps the person ended a long compile), it stays: a reported crash only.
    assert_eq!(judge(&on_gl(&[]), Some(&saved)), fallback::Verdict::Nothing);
    // Nothing saved (a session started again whose failure could not be written): nothing to forget.
    assert_eq!(judge(&on_gl(&CRASHED), None), fallback::Verdict::Nothing);
}

#[cfg(windows)]
#[test]
fn the_restarted_process_gets_its_names_once_and_the_rest_sorted() {
    use std::ffi::OsString;
    let base = vec![
        (OsString::from("Path"), OsString::from(r"C:\bin")),
        // A name differing only in case is the same name to Windows: replaced, not doubled.
        (OsString::from("wgpu_backend"), OsString::from("vulkan")),
        (OsString::from("=C:"), OsString::from(r"C:\work")),
        (OsString::from("APPDATA"), OsString::from("x")),
    ];
    let b = fallback::same_start::block(base, &[("WGPU_BACKEND", "gl"), (fallback::RELAUNCH_ENV, "lost")]);
    assert_eq!(&b[b.len() - 2..], &[0, 0], "the last string, then the empty one that ends the block");
    let text = String::from_utf16(&b[..b.len() - 2]).expect("UTF-16");
    let entries: Vec<&str> = text.split('\0').collect();
    assert_eq!(
        entries,
        [r"=C:=C:\work", "APPDATA=x", "FRACTADYNE_VULKAN_FELL_BACK=lost", r"Path=C:\bin", "WGPU_BACKEND=gl"]
    );
}
