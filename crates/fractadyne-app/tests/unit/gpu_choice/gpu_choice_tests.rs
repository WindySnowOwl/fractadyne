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
