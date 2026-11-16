use super::*;

/// A session the app wrote itself is left exactly as it is: nothing in range is touched.
#[test]
fn a_session_in_range_is_left_alone() {
    let mut s = SessionState::default();
    assert!(sanitize(&mut s).is_empty());
    assert_eq!(s, SessionState::default());
    // Non-default but in range: the user's own settings.
    let mut s = SessionState { aa: 8, zoom_rate: 4.0, export_width: 7680, export_ss: 4, fps_cap: 0.0, ..Default::default() };
    s.coloring.cycle = 0.37;
    s.coloring.light_angle = 1234.5;
    let before = s.clone();
    assert!(sanitize(&mut s).is_empty());
    assert_eq!(s, before);
}

/// Review findings 6 and 9: values a hand edit, a damaged file or a bug could leave are clamped to
/// what the app can set, and what is not a number goes back to its default — each one named.
#[test]
fn values_the_app_cannot_set_are_clamped_or_reset_and_named() {
    let d = SessionState::default();
    let mut s = SessionState {
        aa: 1000,
        julia_c_re: f64::NAN,
        julia_c_im: 0.2,
        zoom_rate: f32::INFINITY,
        fps_cap: -5.0,
        export_width: 1_000_000,
        export_ss: 0,
        ui_scale: 0.0,
        julia_view_upp_log2: Some(f64::NAN),
        max_iter: 2_000_000_000,
        ..Default::default()
    };
    s.coloring.light_angle = f32::NAN;
    s.coloring.cycle = f32::NAN;
    s.coloring.stripe_freq = 1000.0;
    s.coloring.color_method = "no-such-method".into();
    let fixed = sanitize(&mut s);
    assert_eq!(s.aa, 16);
    assert_eq!((s.julia_c_re, s.julia_c_im), (d.julia_c_re, d.julia_c_im), "a pair with a NaN half goes back whole");
    assert_eq!(s.zoom_rate, d.zoom_rate);
    assert_eq!(s.fps_cap, d.fps_cap);
    assert_eq!((s.export_width, s.export_ss), (32_768, 1));
    assert_eq!(s.ui_scale, 0.6);
    assert_eq!(s.julia_view_upp_log2, None);
    assert_eq!(s.max_iter, crate::MAX_ITER_LIMIT);
    assert_eq!(s.coloring.light_angle, d.coloring.light_angle);
    assert_eq!(s.coloring.cycle, d.coloring.cycle);
    assert_eq!(s.coloring.stripe_freq, 24.0);
    assert_eq!(s.coloring.color_method, d.coloring.color_method);
    for name in [
        "aa", "julia_c", "zoom_rate", "fps_cap", "export_width", "export_ss", "ui_scale", "julia_view_upp_log2",
        "max_iter", "light angle", "cycle", "stripe density", "coloring method",
    ] {
        assert!(fixed.contains(&name), "{name} not named in {fixed:?}");
    }
    // And the result is stable: a second pass changes nothing.
    let again = s.clone();
    assert!(sanitize(&mut s).is_empty());
    assert_eq!(s, again);
}
