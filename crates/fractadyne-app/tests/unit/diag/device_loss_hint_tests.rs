//! `device_loss_hint` — the crash report carries a driver-and-report hint ONLY for a GPU device loss.
//!
//! The 2026-09-11 parabolic-point device loss was an NVIDIA Vulkan driver bug (deterministic on
//! 596.21, gone on 616.92, no app change); the 2026-09-12 export loss happened ON 616.92. So the
//! hint asks for a current driver AND for the report either way — never "update first" alone,
//! which would talk a user on a current driver out of sending the one capture that matters.
//! A panic or OOM must NOT carry that line — a driver hint there is misdirection.

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
            h.contains("driver is current")
                && h.contains("issues/1")
                && h.contains("crash-view")
                && !h.contains("first")
                && h.starts_with("hint    :")
                && h.ends_with('\n'),
            "expected a current-driver + attach-the-report hint for {msg:?}, got {h:?}"
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
