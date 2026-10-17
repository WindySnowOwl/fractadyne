//! `Viewport::is_usable`: the predicate the app holds every frame's view to (a NaN scale reached
//! the live view in three field sessions with no line naming its cause).

use super::*;

#[test]
fn a_view_with_a_non_finite_or_zero_scale_is_not_usable() {
    let vp = Viewport::new(1920.0, 1200.0);
    assert!(vp.is_usable(), "the home view");
    for k in [f64::NAN, f64::INFINITY, 0.0, -1.0] {
        let mut bad = vp.clone();
        bad.units_per_pixel = bad.units_per_pixel.mul_f64(k);
        assert!(!bad.is_usable(), "a scale multiplied by {k}");
    }
    // The way it happens: one zoom by a non-finite factor.
    let mut z = vp.clone();
    z.zoom_at(960.0, 600.0, f64::NAN);
    assert!(!z.is_usable(), "zoom_at by NaN");
    // A deep but finite view is usable.
    let mut deep = vp.clone();
    deep.set_center_log2mag(bf(-0.75, 64), bf(0.1, 64), 16_609.6);
    assert!(deep.is_usable(), "a view at 2^16609.6");
}
