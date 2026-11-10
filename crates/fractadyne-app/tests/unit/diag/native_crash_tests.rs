//! What a crash that is not a panic says about itself (`diag::native_crash`).

use super::native_crash::{describe_called, exception_name, file_name, graphics_module};
use super::*;

/// The account of a crash with no caller found (every fault in a module, and the rest).
fn describe(code: u32, address: usize, params: &[usize], module: Option<(&str, usize)>, thread: &str) -> String {
    describe_called(code, address, params, module, None, thread)
}

#[test]
fn an_access_violation_in_a_driver_names_the_driver_and_the_address() {
    let s = describe(0xC000_0005, 0x7ffb_1234_5678, &[0, 0x10], Some(("ig75icd64.dll", 0x1a_2b3c)), "main");
    assert_eq!(
        s,
        "a native crash: access violation (0xc0000005) reading address 0x10, in the graphics driver: ig75icd64.dll+0x1a2b3c, on thread 'main'"
    );
    let w = describe(0xC000_0005, 1, &[1, 0x0], Some(("fractadyne.exe", 0x40)), "fd-ref-live");
    assert!(w.contains("writing address 0x0, in fractadyne.exe+0x40, on thread 'fd-ref-live'"), "{w}");
    let x = describe(0xC000_0005, 0xdead, &[8, 0xdead], None, "main");
    assert!(x.contains("executing address 0xdead, at 0xdead (in no loaded module)"), "{x}");
}

#[test]
fn other_exceptions_are_named_or_given_by_code() {
    assert_eq!(exception_name(0xC000_001D), Some("illegal instruction"));
    assert_eq!(exception_name(0xC000_00FD), Some("stack overflow"));
    assert_eq!(exception_name(0xC000_0094), Some("integer divide by zero"));
    assert_eq!(exception_name(0x1234_5678), None);
    // Only an access violation and an in-page error carry an address in their parameters.
    let s = describe(0xC000_001D, 0x10, &[1, 2], Some(("nvoglv64.dll", 0x10)), "main");
    assert_eq!(s, "a native crash: illegal instruction (0xc000001d), in the graphics driver: nvoglv64.dll+0x10, on thread 'main'");
    let u = describe(0x1234_5678, 0x10, &[], Some(("kernelbase.dll", 0x99)), "main");
    assert_eq!(u, "a native crash: exception 0x12345678, in kernelbase.dll+0x99, on thread 'main'");
    // Too few parameters for an access violation: no address claimed.
    let short = describe(0xC000_0005, 0x10, &[1], None, "main");
    assert!(!short.contains("address 0x"), "{short}");
}

#[test]
fn graphics_drivers_are_told_from_other_modules() {
    for m in [
        "ig75icd64.dll", "igxelpicd64.dll", "igc64.dll", "nvoglv64.dll", "nvwgf2umx.dll", "atio6axx.dll", "amdvlk64.dll",
        "OPENGL32.dll", "vulkan-1.dll", "d3d12core.dll", "dxgi.dll", "D3DCompiler_47.dll",
    ] {
        assert!(graphics_module(m), "{m} is graphics");
    }
    for m in ["fractadyne.exe", "KERNELBASE.dll", "ntdll.dll", "ucrtbase.dll", "user32.dll", "winmm.dll"] {
        assert!(!graphics_module(m), "{m} is not graphics");
    }
}

#[test]
fn a_module_path_gives_its_file_name() {
    assert_eq!(file_name(r"C:\Windows\System32\DriverStore\FileRepository\igdlh64.inf_amd64\ig75icd64.dll"), "ig75icd64.dll");
    assert_eq!(file_name("ig75icd64.dll"), "ig75icd64.dll");
    assert_eq!(file_name("/usr/lib/libGL.so.1"), "libGL.so.1");
}

#[test]
fn the_dialogs_reason_keeps_the_whole_line() {
    // The dialog shows `crash_reason` of the description: one line, well under its 400 characters.
    let s = describe(0xC000_0005, 0, &[0, 0x10], Some(("ig75icd64.dll", 0x1a_2b3c)), "main");
    assert_eq!(crash_reason(&s), s);
}

/// A call to address 0 faults in no module; the return address it left names the caller, and a
/// graphics driver's or loader's is said to be one (Venus, 2026-11-10: `--list-adapters` executed
/// 0x0 and the report could not say whose call it was).
#[test]
fn a_call_to_a_null_pointer_names_its_caller() {
    let v = describe_called(0xC000_0005, 0, &[8, 0], None, Some(("vulkan-1.dll", 0x1_2345)), "main");
    assert_eq!(
        v,
        "a native crash: access violation (0xc0000005) executing address 0x0, at 0x0 (in no loaded module), called \
         from the graphics driver: vulkan-1.dll+0x12345, on thread 'main'"
    );
    let f = describe_called(0xC000_0005, 0, &[8, 0], None, Some(("fractadyne.exe", 0x40)), "main");
    assert!(f.ends_with("(in no loaded module), called from fractadyne.exe+0x40, on thread 'main'"), "{f}");
    // A fault inside a module is placed there; a caller is not looked for.
    let m = describe_called(0xC000_0005, 0x10, &[0, 0x10], Some(("nvoglv64.dll", 0x10)), Some(("x.dll", 1)), "main");
    assert!(!m.contains("called from"), "{m}");
}
