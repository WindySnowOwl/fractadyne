use super::home_start_logmag;
use fractadyne_core::{BigFloat, Viewport};

fn view_at(l2: f64) -> Viewport {
    let mut vp = Viewport::new(1440.0, 1102.0);
    vp.set_center_log2mag(BigFloat::from_f64(-0.75, 128), BigFloat::from_f64(0.1, 128), l2);
    vp
}

/// Home clicked past ~1e308×: the glide used to start from `ln(∞)` and set the scale to NaN on
/// every frame (the user's 2026-10-25 session at 2^1214; reproduced from 2^1663).
#[test]
fn a_home_glide_starts_from_a_finite_depth_past_1e308() {
    let vp = view_at(1663.37);
    assert!(vp.magnification().is_infinite(), "the f64 magnification overflows here: the trap");
    let start = home_start_logmag(&vp);
    assert!((start - 1663.37 * std::f64::consts::LN_2).abs() < 1e-6, "{start}");
    // Every frame of the glide is then a usable view, easing the depth toward Home.
    let centre = (vp.center_x.clone(), vp.center_y.clone());
    for remain in [1.0, 0.75, 0.5, 0.1, 1e-3] {
        let mut v = vp.clone();
        v.home_lerp((-0.5, 0.0), &centre, start * remain);
        assert!(v.is_usable(), "remain {remain}: scale m={} e={}", v.units_per_pixel.m, v.units_per_pixel.e);
        assert!((v.log2_magnification() - 1663.37 * remain).abs() < 1e-6, "remain {remain}");
    }
}

#[test]
fn home_starts_from_zero_at_or_above_home_and_for_an_unusable_view() {
    assert_eq!(home_start_logmag(&view_at(0.0)), 0.0);
    assert_eq!(home_start_logmag(&view_at(-1.5)), 0.0);
    let mut nan = view_at(1214.47);
    nan.units_per_pixel = nan.units_per_pixel.mul_f64(f64::NAN);
    assert_eq!(home_start_logmag(&nan), 0.0, "an unusable view goes straight Home");
}
