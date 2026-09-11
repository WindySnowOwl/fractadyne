//! `device_loss_hint` — the crash report suggests a driver update ONLY for a GPU device loss.
//!
//! The 2026-09-11 parabolic-point device loss was an NVIDIA Vulkan driver bug (deterministic on
//! 596.21, gone on 616.92, no app change), so a device-loss crash report leads with "update your
//! driver". A panic or OOM must NOT carry that line — a driver hint there is misdirection.

use super::device_loss_hint;

#[test]
fn device_loss_crashes_get_the_driver_hint() {
    // The real device-lost handler messages (main.rs), plus vendor spellings.
    for msg in [
        "wgpu device lost (Unknown): Device is lost",
        "wgpu device lost: outdated",
        "DeviceLost",
        "DXGI_ERROR_DEVICE_REMOVED",
    ] {
        let h = device_loss_hint(msg);
        assert!(
            h.contains("Update your graphics driver") && h.starts_with("hint    :") && h.ends_with('\n'),
            "expected a driver-update hint for {msg:?}, got {h:?}"
        );
    }
}

#[test]
fn other_crashes_get_no_hint() {
    // A panic or OOM is not a device loss; a driver suggestion there would be noise.
    for msg in [
        "index out of bounds: the len is 3 but the index is 7",
        "<allocator> out of memory requesting 4 GiB",
        "called `Option::unwrap()` on a `None` value",
        "",
    ] {
        assert_eq!(device_loss_hint(msg), "", "expected no hint for {msg:?}");
    }
}
