//! What a fatal error tells the person beyond the error: the account and the tips (`diag::advice`).

use super::advice::*;

fn eris() -> Gpu {
    Gpu { name: "Intel(R) HD Graphics 4400".into(), api: "OpenGL", driver: "4.3.0 - Build 20.19.15.4531".into(), vendor: Some("Intel") }
}

fn at<'a>(failure: &'a Failure, phase: Phase, compiling: &'a [String], gpu: Option<&'a Gpu>) -> Situation<'a> {
    Situation { failure, phase, compiling, gpu, activity: None, remote_session: false }
}

/// The English phrases, as `ui::crash_dialog_text` prepares them.
fn english() -> Phrases {
    crate::ui::crash_dialog_text().phrases
}

#[test]
fn erises_crash_reads_as_a_driver_crash_while_compiling_with_driver_tips() {
    // 2026-11-09: ig75icd64.dll died linking fs_iterate, the renderer's first pipeline.
    let f = Failure::Native { module: Some("ig75icd64.dll".into()) };
    let compiling = vec!["fractadyne.iter_pipeline fs_iterate (on main, 5.2 s)".to_string()];
    let g = eris();
    let s = at(&f, Phase::PreparingShaders, &compiling, Some(&g));
    assert_eq!(summary(&s), Summary::DriverWhileCompiling);
    assert_eq!(tips(&s), vec![Tip::UpdateDriver, Tip::OtherCard, Tip::NewerFractadyne], "at start-up it happens every time: no 'start again'");
    let a = compose(&s, &english());
    assert_eq!(
        a.what,
        "The graphics driver (ig75icd64.dll) crashed while it was compiling Fractadyne's shaders.\n\
         When: at start-up, while preparing the graphics shaders\n\
         Shader being compiled: fractadyne.iter_pipeline fs_iterate (on main, 5.2 s)\n\
         Graphics: Intel(R) HD Graphics 4400 (OpenGL, driver 4.3.0 - Build 20.19.15.4531)"
    );
    assert!(a.tips.starts_with("• Install the newest graphics driver for this card from Intel's website."), "{}", a.tips);
    assert!(a.tips.contains("• If this computer has a second graphics card"), "{}", a.tips);
    assert!(a.tips.contains("--list-adapters") && a.tips.contains("--adapter 2"), "{}", a.tips);
    assert_eq!(a.tips.lines().count(), 3, "{}", a.tips);
    assert!(!a.what.contains('{') && !a.tips.contains('{'), "every placeholder filled: {a:?}");
}

#[test]
fn a_driver_crash_mid_session_says_start_again_too() {
    let f = Failure::Native { module: Some("nvoglv64.dll".into()) };
    let g = Gpu { name: "NVIDIA GeForce RTX 3080".into(), api: "Vulkan", driver: "NVIDIA 566.36".into(), vendor: Some("NVIDIA") };
    let mut s = at(&f, Phase::Running, &[], Some(&g));
    s.activity = Some("live: building the reference orbit");
    assert_eq!(summary(&s), Summary::Driver);
    assert_eq!(tips(&s), vec![Tip::UpdateDriver, Tip::OtherCard, Tip::StartAgain, Tip::NewerFractadyne]);
    let a = compose(&s, &english());
    assert!(a.what.starts_with("The graphics driver (nvoglv64.dll) crashed.\nWhen: while Fractadyne was running\n"), "{}", a.what);
    assert!(a.what.contains("\nDoing: live: building the reference orbit\n"), "{}", a.what);
    assert!(a.tips.contains("from NVIDIA's website"), "{}", a.tips);
}

#[test]
fn a_bug_in_fractadyne_mid_session_does_not_blame_the_driver() {
    let f = Failure::Panic("index out of bounds: the len is 3 but the index is 7".into());
    let g = eris();
    let s = at(&f, Phase::Running, &[], Some(&g));
    assert_eq!(summary(&s), Summary::Bug);
    assert_eq!(tips(&s), vec![Tip::StartAgain, Tip::NewerFractadyne]);
    let a = compose(&s, &english());
    assert!(a.what.starts_with("Fractadyne stopped on an error in its own code.\n"), "{}", a.what);
    assert!(!a.tips.contains("driver"), "{}", a.tips);
}

#[test]
fn a_bug_at_start_up_on_the_graphics_side_gets_the_graphics_tips() {
    // beta.26's laptop crash: a panic building the renderer (no storage buffers on OpenGL).
    let f = Failure::Panic("wgpu error: Validation Error\nCaused by: In Device::create_bind_group_layout".into());
    let s = at(&f, Phase::PreparingShaders, &[], None);
    assert_eq!(summary(&s), Summary::Bug);
    assert_eq!(tips(&s), vec![Tip::UpdateDriver, Tip::OtherCard, Tip::NewerFractadyne]);
    // No card noted yet: the tip names no maker, and no Graphics line appears.
    let a = compose(&s, &english());
    assert!(a.tips.starts_with("• Install the newest graphics driver from the website of the company that made the graphics card, such as Intel, NVIDIA or AMD."), "{}", a.tips);
    assert!(!a.what.contains("Graphics:"), "{}", a.what);
}

#[test]
fn a_lost_device_reads_as_such() {
    let f = Failure::Panic("wgpu error: device lost: Parent device is lost".into());
    let s = at(&f, Phase::Running, &[], None);
    assert_eq!(summary(&s), Summary::DeviceLost);
    assert_eq!(tips(&s), vec![Tip::UpdateDriver, Tip::OtherCard, Tip::StartAgain, Tip::NewerFractadyne]);
}

#[test]
fn remote_desktop_comes_first_where_the_graphics_side_failed() {
    let f = Failure::WindowOpen;
    let mut s = at(&f, Phase::OpeningWindow, &[], None);
    s.remote_session = true;
    assert_eq!(tips(&s), vec![Tip::RemoteDesktop, Tip::UpdateDriver, Tip::OtherCard, Tip::NewerFractadyne]);
    let a = compose(&s, &english());
    assert!(a.tips.starts_with("• This is a Remote Desktop session"), "{}", a.tips);
    assert_eq!(a.what, "The window could not open.\nWhen: at start-up, while opening the window");
    // ...and not for a bug in Fractadyne mid-session, which no graphics acceleration explains.
    let p = Failure::Panic("x".into());
    let mut s = at(&p, Phase::Running, &[], None);
    s.remote_session = true;
    assert!(!tips(&s).contains(&Tip::RemoteDesktop));
}

#[test]
fn an_unsupported_driver_is_not_told_to_wait_for_a_newer_fractadyne() {
    let f = Failure::Unsupported;
    let g = eris();
    let s = at(&f, Phase::StartingDevice, &[], Some(&g));
    assert_eq!(tips(&s), vec![Tip::UpdateDriver, Tip::OtherCard]);
    let a = compose(&s, &english());
    assert!(a.what.starts_with("This graphics driver lacks a feature Fractadyne needs."), "{}", a.what);
}

#[test]
fn a_crash_in_another_module_or_none_names_what_it_can() {
    let f = Failure::Native { module: Some("KERNELBASE.dll".into()) };
    let s = at(&f, Phase::Running, &[], None);
    assert_eq!(summary(&s), Summary::InModule);
    assert!(compose(&s, &english()).what.starts_with("Fractadyne crashed inside KERNELBASE.dll."));
    let f = Failure::Native { module: None };
    let s = at(&f, Phase::Unknown, &[], None);
    assert_eq!(summary(&s), Summary::Native);
    assert_eq!(compose(&s, &english()).what, "Fractadyne crashed.", "no phase known: no When line");
}

#[test]
fn a_pipeline_compiling_mid_session_counts_as_the_graphics_side() {
    // A custom formula's pipeline, compiled long after start-up.
    let f = Failure::Native { module: Some("igc64.dll".into()) };
    let compiling = vec!["custom.iter_pipeline fs_iterate (on main, 1.0 s)".to_string()];
    let s = at(&f, Phase::Running, &compiling, None);
    assert_eq!(summary(&s), Summary::DriverWhileCompiling);
    assert!(tips(&s).starts_with(&[Tip::UpdateDriver, Tip::OtherCard]));
}

#[test]
fn the_card_its_api_and_its_maker_are_named_as_people_know_them() {
    use eframe::wgpu::Backend;
    assert_eq!(api_name(Backend::Gl), "OpenGL");
    assert_eq!(api_name(Backend::Dx12), "Direct3D 12");
    assert_eq!(api_name(Backend::Vulkan), "Vulkan");
    assert_eq!(vendor_name(0x8086, ""), Some("Intel"));
    assert_eq!(vendor_name(0x10DE, ""), Some("NVIDIA"));
    assert_eq!(vendor_name(0x1002, ""), Some("AMD"));
    // OpenGL may report no vendor id: the name says it.
    assert_eq!(vendor_name(0, "Intel(R) HD Graphics 4400"), Some("Intel"));
    assert_eq!(vendor_name(0, "AMD Radeon RX 6800 XT"), Some("AMD"));
    assert_eq!(vendor_name(0, "NVIDIA GeForce RTX 3080"), Some("NVIDIA"));
    assert_eq!(vendor_name(0, "Microsoft Basic Render Driver"), None);
    assert_eq!(driver_text("", "4.3.0 - Build 20.19.15.4531"), "4.3.0 - Build 20.19.15.4531");
    assert_eq!(driver_text("NVIDIA", "566.36"), "NVIDIA 566.36");
    assert_eq!(driver_text(" ", ""), "");
    // A driver that reports no version: the Graphics line leaves the driver out.
    let g = Gpu { name: "Some card".into(), api: "Vulkan", driver: String::new(), vendor: None };
    let f = Failure::NoDevice;
    let a = compose(&at(&f, Phase::StartingDevice, &[], Some(&g)), &english());
    assert!(a.what.ends_with("\nGraphics: Some card (Vulkan)"), "{}", a.what);
}

#[test]
fn every_phrase_is_there_and_keeps_its_placeholders() {
    let p = english();
    for (name, text, holder) in [
        ("summary_driver_compiling", &p.summary_driver_compiling, Some("{module}")),
        ("summary_driver", &p.summary_driver, Some("{module}")),
        ("summary_module", &p.summary_module, Some("{module}")),
        ("when", &p.when, Some("{phase}")),
        ("shader", &p.shader, Some("{shader}")),
        ("activity", &p.activity, Some("{activity}")),
        ("graphics", &p.graphics, Some("{driver}")),
        ("graphics_no_driver", &p.graphics_no_driver, Some("{card}")),
        ("tip_driver_vendor", &p.tip_driver_vendor, Some("{vendor}")),
        ("summary_device_lost", &p.summary_device_lost, None),
        ("summary_native", &p.summary_native, None),
        ("summary_bug", &p.summary_bug, None),
        ("summary_window", &p.summary_window, None),
        ("summary_no_device", &p.summary_no_device, None),
        ("summary_unsupported", &p.summary_unsupported, None),
        ("phase_window", &p.phase_window, None),
        ("phase_device", &p.phase_device, None),
        ("phase_shaders", &p.phase_shaders, None),
        ("phase_running", &p.phase_running, None),
        ("tip_remote", &p.tip_remote, None),
        ("tip_driver", &p.tip_driver, None),
        ("tip_other_card", &p.tip_other_card, None),
        ("tip_again", &p.tip_again, None),
        ("tip_newer", &p.tip_newer, None),
    ] {
        assert!(!text.trim().is_empty(), "{name} is empty");
        if let Some(h) = holder {
            assert!(text.contains(h), "{name} lost {h}: {text}");
        }
    }
}
