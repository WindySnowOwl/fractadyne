//! The target box is a statement about the camera — "this region fills the screen next" — so it
//! is pinned against the zoom's own arithmetic, not against how it looks.

use super::{target_rect, ZOOM_TARGET_FACTOR};

/// Where a screen-fraction point lands after `Viewport::zoom_at(pivot, 1/factor)`, i.e. a
/// magnification by `factor` about `pivot` (the same map `autopilot::after_zoom` applies).
fn after_zoom(x: (f64, f64), pivot: (f64, f64), factor: f64) -> (f64, f64) {
    (pivot.0 + (x.0 - pivot.0) * factor, pivot.1 + (x.1 - pivot.1) * factor)
}

#[test]
fn the_box_is_exactly_what_fills_the_screen_after_the_zoom() {
    for aim in [(0.5, 0.5), (0.3, 0.7), (0.0, 0.0), (1.0, 1.0), (0.83, 0.12)] {
        let (min, max) = target_rect(aim, ZOOM_TARGET_FACTOR);
        // Its corners map onto the screen's corners under the zoom about the aim.
        let tl = after_zoom(min, aim, ZOOM_TARGET_FACTOR);
        let br = after_zoom(max, aim, ZOOM_TARGET_FACTOR);
        assert!((tl.0).abs() < 1e-12 && (tl.1).abs() < 1e-12, "aim {aim:?}: top-left → {tl:?}");
        assert!((br.0 - 1.0).abs() < 1e-12 && (br.1 - 1.0).abs() < 1e-12, "aim {aim:?}: bottom-right → {br:?}");
        // The aim itself is inside the box (it is the fixed point).
        assert!(min.0 <= aim.0 && aim.0 <= max.0 && min.1 <= aim.1 && aim.1 <= max.1);
    }
}

#[test]
fn the_box_never_leaves_the_screen_and_never_degenerates() {
    for aim in [(0.5, 0.5), (0.0, 1.0), (1.0, 0.0), (0.999, 0.001)] {
        let (min, max) = target_rect(aim, ZOOM_TARGET_FACTOR);
        assert!(min.0 >= 0.0 && min.1 >= 0.0 && max.0 <= 1.0 + 1e-12 && max.1 <= 1.0 + 1e-12, "{aim:?}");
        assert!((max.0 - min.0 - 1.0 / ZOOM_TARGET_FACTOR).abs() < 1e-12);
    }
    // Garbage in: a full-screen box, never a NaN or an inverted one.
    let (min, max) = target_rect((f64::NAN, 0.5), f64::NAN);
    assert_eq!((min, max), ((0.0, 0.0), (1.0, 1.0)));
    let (min, max) = target_rect((0.5, 0.5), 0.5);
    assert_eq!((min, max), ((0.0, 0.0), (1.0, 1.0)));
}
