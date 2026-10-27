use super::{
    home_start_logmag, home_stop_done, home_stops, HOME_MAX_STOPS, HOME_SMOOTH_LOG2, HOME_STEP_FROM_LOG2,
    HOME_STOP_MAX_SECONDS, HOME_STOP_SHOW_SECONDS,
};
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

/// The user's 2026-10-26 recording: Home from 1.46e4850× (2^16,111) glided at ~1,800 octaves a
/// second and showed solid panels of colour. Stepped, it stops ten times, each about half the last
/// depth, down to the depth the glide takes over at.
#[test]
fn a_home_from_extreme_depth_stops_at_halving_depths() {
    let start = 4850.0 * std::f64::consts::LOG2_10 + 1.46_f64.log2();
    let stops = home_stops(start);
    assert_eq!(stops.len(), 10);
    assert_eq!(*stops.last().unwrap(), HOME_SMOOTH_LOG2, "the last stop is where the glide begins");
    let mut prev = start;
    for s in &stops {
        let ratio = s / prev;
        assert!((0.45..0.53).contains(&ratio), "about half the last depth: {prev} → {s}");
        prev = *s;
    }
}

#[test]
fn a_shallower_home_stops_less_often_and_one_from_a_glide_depth_never() {
    // From HOME_STEP_FROM_LOG2 or shallower (or no number): no stops, the glide as before.
    for l2 in [0.0, HOME_SMOOTH_LOG2, 40.0, HOME_STEP_FROM_LOG2, f64::NAN] {
        assert!(home_stops(l2).is_empty(), "{l2}");
    }
    // Just past it: two stops (~2^28, 2^12).
    let s = home_stops(HOME_STEP_FROM_LOG2 + 1.0);
    assert_eq!(s.len(), 2, "{s:?}");
    // 2^1663 (the corpus view that lost the RTX 3080) seven, 2^262 four; never more than the cap,
    // always deepest first and ending where the glide begins.
    assert_eq!(home_stops(1663.37).len(), 7);
    assert_eq!(home_stops(262.92).len(), 4);
    assert!(home_stops(1.0e12).len() <= HOME_MAX_STOPS);
    for start in [80.0, 262.92, 1663.37, 13287.4, 1.0e6] {
        let s = home_stops(start);
        assert!(s.windows(2).all(|w| w[0] > w[1]) && s[0] < start, "{start}: {s:?}");
        assert_eq!(*s.last().unwrap(), HOME_SMOOTH_LOG2, "{start}");
    }
}

#[test]
fn a_stop_shows_its_picture_or_gives_way_when_none_comes() {
    let at = 10.0;
    // Finished at +0.8 s: on screen for HOME_STOP_SHOW_SECONDS, then the next stop.
    let fin = Some(at + 0.8);
    assert!(!home_stop_done(at + 0.8 + HOME_STOP_SHOW_SECONDS - 0.01, at, fin));
    assert!(home_stop_done(at + 0.8 + HOME_STOP_SHOW_SECONDS, at, fin));
    // A picture that finishes late still gets its full showing, past the wait limit.
    let late = Some(at + HOME_STOP_MAX_SECONDS - 0.1);
    assert!(!home_stop_done(at + HOME_STOP_MAX_SECONDS + 0.1, at, late));
    // No picture: the stop gives way at HOME_STOP_MAX_SECONDS.
    assert!(!home_stop_done(at + HOME_STOP_MAX_SECONDS - 0.01, at, None));
    assert!(home_stop_done(at + HOME_STOP_MAX_SECONDS, at, None));
}
