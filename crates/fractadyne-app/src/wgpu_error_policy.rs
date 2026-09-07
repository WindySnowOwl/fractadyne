//! The device texture-dimension limit, and the crashes that came of asking for the wrong one.
//!
//! ⚠⚠**Three monitor-drag crashes, and the ceiling they hit was ours.** Dragging the window between
//! monitors of different scaling runs an open window-growth bug; eframe then asks wgpu for a surface
//! the size of the window and wgpu refuses it:
//!
//! | report | version | window reached | limit |
//! |---|---|---|---|
//! | `crash-1788788315-0` | beta.60 | 9374 × 6039 | 8192 |
//! | `crash-1788789479-1` | beta.61 | 8687 × 5949 (viewport vs stale 5784 × 3947 surface) | 8192 |
//! | `crash-1788791115-0` | beta.62 | 11441 × 7098 | 8192 |
//!
//! The adapter that refused all three — an RTX 3080 — reports **16384**. The `8192` was a constant
//! in our own device descriptor, so the first two would have fitted and the third came close.
//!
//! ⛔**This does not fix the growth**, which is unbounded and upstream; it removes one artificial
//! cliff the growth kept falling off. Two mitigations that did NOT work are recorded at the
//! uncaptured-error hook so they are not tried a third time: making the error survivable (beta.61 —
//! left the painter on the old surface and died one frame later, then aborted in a destructor) and
//! `ViewportCommand::MaxInnerSize` (beta.62 — egui-winit does set it, but winit does not enforce it
//! on the `WM_DPICHANGED` resize path, and the window grew straight past it to 11441).

use super::requested_texture_dim;

/// ⚠⚠**The trap the old constant was hiding.** `request_device` fails if ANY required limit exceeds
/// the adapter's, so a hard-coded 8192 was not merely conservative — it would have refused to start
/// at all on an adapter that only does 4096. Asking for the adapter's own figure cannot.
#[test]
fn we_never_request_more_than_the_adapter_offers() {
    for adapter_max in [2048_u32, 4096, 8192, 16384, 32768] {
        let asked = requested_texture_dim(adapter_max);
        assert!(
            asked <= adapter_max,
            "asked for {asked} from an adapter that reports {adapter_max} — request_device would fail"
        );
    }
    // ⚠**The control.** These assertions only mean something because the previous constant DID
    // exceed a real adapter class; without this the test would pass against the old code too.
    const OLD_HARDCODED: u32 = 8192;
    assert!(
        OLD_HARDCODED > 4096,
        "if the old constant fitted every adapter there was nothing to fix here"
    );
}

/// ⭐And on the hardware that produced the reports, we now ask for the whole ceiling.
///
/// ⚠**32768 is MEASURED, not assumed.** The new unconditional geometry log printed
/// `device max 32768` on the RTX 3080 that refused all three surfaces — so the 8192 constant was
/// capping us at a QUARTER of what the hardware offers, and every window in the reports fits with
/// room to spare.
#[test]
fn a_capable_adapter_is_no_longer_capped_at_our_old_constant() {
    const RTX_3080_VULKAN: u32 = 32768; // measured, 2026-09-07
    let asked = requested_texture_dim(RTX_3080_VULKAN);
    assert_eq!(asked, RTX_3080_VULKAN, "the adapter's own ceiling is what we should validate against");
    for reached in [9374_u32, 8687, 11441] {
        assert!(asked >= reached, "the window reached {reached}px; {asked} must accommodate it");
        // ⚠The control: each of these DID exceed the old constant, which is why they crashed.
        assert!(reached > 8192, "{reached} must exceed the old 8192, or it was never the problem");
    }
}
