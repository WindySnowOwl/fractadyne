//! Which wgpu uncaptured errors the app survives, and which still kill it.
//!
//! ⚠⚠**The message below is VERBATIM from a real crash report** (`crash-1788788315-0.txt`,
//! 0.2.41-beta.60, 2026-09-07): the author dragged the window between two monitors of different
//! scaling, the open window-growth bug ran the window to 9374×6039 physical, eframe asked wgpu to
//! configure a surface that big, and the uncaptured-error handler turned the refusal into a panic —
//! taking nine and a half minutes of work with it, because a killed process never reaches the
//! session save. Pinning the real text is the point: a paraphrase would drift from what wgpu
//! actually emits and the guard would silently stop matching.

use super::is_survivable_wgpu_error;

/// The exact text, reflowed only where the report wrapped it.
const REAL_CRASH: &str = "Validation Error\n\nCaused by:\n  In Surface::configure\n    `Surface` \
     width and height must be within the maximum supported texture size. Requested was \
     (9374, 6039), maximum extent for either dimension is 8192.";

#[test]
fn the_oversized_surface_from_the_real_crash_is_survivable() {
    assert!(
        is_survivable_wgpu_error(REAL_CRASH),
        "the message that actually killed the app must be recognised"
    );
}

/// ⚠**The guard that keeps this from becoming "never panic".** Every other validation error is a
/// bug in our own rendering, and the self-test and goldens depend on those still being loud. If
/// this test ever goes green for one of these, the handler has stopped being a handler.
#[test]
fn ordinary_validation_errors_still_kill_the_process() {
    for msg in [
        "Validation Error\n\nCaused by:\n  In Device::create_bind_group\n    Number of bindings in \
         bind group descriptor (3) does not match the number of bindings defined in the bind group \
         layout (4)",
        "Validation Error\n\nCaused by:\n  In RenderPass::set_pipeline\n    Render pipeline targets \
         are incompatible with render pass",
        "Validation Error\n\nCaused by:\n  In Device::create_buffer\n    Not enough memory left",
        "Out of Memory\n\nCaused by:\n  Failed to allocate a buffer",
    ] {
        assert!(
            !is_survivable_wgpu_error(msg),
            "this must still panic, or a real rendering bug becomes invisible:\n{msg}"
        );
    }
}

/// wgpu has reworded this across versions, so both spellings are accepted — the cost of a miss is
/// the crash this whole module exists to prevent.
#[test]
fn both_spellings_of_the_size_refusal_are_recognised() {
    assert!(is_survivable_wgpu_error(
        "`Surface` width and height must be within the maximum supported texture size"
    ));
    assert!(is_survivable_wgpu_error(
        "Texture dimension exceeds the maximum supported texture size for this device"
    ));
    // ⚠And a message that merely mentions a surface is NOT enough on its own.
    assert!(!is_survivable_wgpu_error(
        "In Surface::get_current_texture\n    The surface is outdated"
    ));
}
