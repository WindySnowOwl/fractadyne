//! The window-size cap that keeps a surface inside what the device can allocate.
//!
//! ⚠⚠**Written from two real crashes, one caused by the fix for the other.**
//!
//! `crash-1788788315-0` (beta.60): dragging between monitors of different scaling ran the open
//! window-growth bug until the window reached **9374 × 6039** physical. eframe asked for a surface
//! that big, wgpu refused (`maximum extent for either dimension is 8192`), and the uncaptured-error
//! handler panicked. 576 s of work went with it, since a dead process never reaches the save.
//!
//! `crash-1788789479-{1,2,3}` (beta.61): making that error *survivable* was worse. Carrying on past
//! a failed `Surface::configure` leaves the painter on the OLD surface while egui renders at the
//! NEW size, so the next frame died one step along — `viewport 8687×5949 not contained in render
//! target 5784×3947` — and unwinding through wgpu-hal then panicked in a destructor and aborted.
//!
//! ⇒ The error is not survivable and must stay loud. What is fixable is never reaching it: cap the
//! window so the surface it implies always fits. `MaxInnerSize` is enforced by winit while it
//! handles the resize, which is upstream of the reconfigure — a correction issued from our own
//! frame would always be one frame late, and one frame is all it took.

use super::max_window_points;

/// The device limit this app asks for, and the one both crashes hit.
const MAX_DIM: u32 = 8192;

/// The physical size the window actually reached, from the first crash report.
const CRASHED_AT: (f32, f32) = (9374.0, 6039.0);

/// ⭐⭐**The invariant**: whatever the scale factor, a window at the cap allocates a surface the
/// device can hold. Points × scale = physical, so the cap has to move with the scale — and the
/// scale is exactly what changes when a window crosses to another monitor.
#[test]
fn a_window_at_the_cap_always_fits_the_device_limit() {
    // Every scale factor Windows offers, plus the awkward ones in between.
    for ppp in [1.0_f32, 1.25, 1.5, 1.75, 2.0, 2.5, 3.0, 1.1, 2.25] {
        let cap = max_window_points(MAX_DIM, ppp);
        for side in [cap.x, cap.y] {
            let physical = side * ppp;
            assert!(
                physical <= MAX_DIM as f32 + 0.5,
                "at {ppp}× a window of {side} points is {physical} physical, over the {MAX_DIM} limit"
            );
        }
    }
}

/// ⭐**The case that actually happened.** At any plausible scale factor the cap is below the size
/// the window grew to, so winit would have stopped it short of the surface wgpu refused.
#[test]
fn the_cap_would_have_prevented_the_reported_crash() {
    for ppp in [1.0_f32, 1.25, 1.5, 2.0] {
        let cap = max_window_points(MAX_DIM, ppp);
        // What the window reached, expressed in the points winit would have been clamping.
        let (reached_w, reached_h) = (CRASHED_AT.0 / ppp, CRASHED_AT.1 / ppp);
        assert!(
            reached_w > cap.x,
            "at {ppp}× the crashing width ({reached_w} points) must exceed the cap ({}), or this \
             test proves nothing",
            cap.x
        );
        assert!(cap.x * ppp < CRASHED_AT.0, "the capped window must be smaller than the one that failed");
        let _ = reached_h;
    }
}

/// ⚠**A bad scale factor must never produce a LARGER cap.** The one direction that matters: too
/// small is a slightly restricted window, too large is the crash. egui has reported odd values
/// during a monitor transition — which is precisely when this runs.
#[test]
fn a_nonsense_scale_factor_falls_back_instead_of_uncapping() {
    for ppp in [0.0_f32, -1.0, f32::NAN, f32::INFINITY, 1.0e-9] {
        let cap = max_window_points(MAX_DIM, ppp);
        assert!(cap.x.is_finite() && cap.y.is_finite(), "cap must stay finite for ppp={ppp}");
        assert!(
            cap.x <= MAX_DIM as f32 && cap.y <= MAX_DIM as f32,
            "ppp={ppp} produced a cap of {cap:?}, which is larger than the device limit itself"
        );
        assert!(cap.x > 0.0, "ppp={ppp} produced a non-positive cap, which would pin the window shut");
    }
}
