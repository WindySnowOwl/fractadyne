use super::*;

fn customised() -> SessionState {
    SessionState {
        max_iter: 123_456,
        color_method: "stripe".into(),
        stripe_freq: 7.5,
        light: true,
        de: true,
        palette_idx: 3,
        custom_formula: "z = z^3 + c".into(),
        custom_params: vec![[0.5, -0.25]],
        use_custom_palette: true,
        custom_palette: vec![[1.0, 0.5, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]],
        watermark: false,
        series_approx: false,
        // Things that are NOT render settings and must not travel:
        last_script: Some("C:/Users/robin/secret/tour.toml".into()),
        export_dir: Some("D:/private".into()),
        ui_scale: 2.0,
        update_track: "beta".into(),
        ..SessionState::default()
    }
}

type Mutation = Box<dyn Fn(&mut RenderSettings)>;

/// ⭐What a client writes is the controller's render settings and NOTHING else: listed fields
/// carry over, unlisted ones stay at their defaults — no path, no UI state, no update channel.
#[test]
fn listed_fields_travel_and_nothing_else_does() {
    let src = customised();
    let rs = RenderSettings::from_session(&src);
    rs.validate().expect("a real session's settings validate");
    let out = rs.to_session();
    assert_eq!(out.max_iter, 123_456);
    assert_eq!(out.color_method, "stripe");
    assert_eq!(out.stripe_freq, 7.5);
    assert!(out.light && out.de && out.use_custom_palette);
    assert_eq!(out.custom_formula, src.custom_formula);
    assert_eq!(out.custom_params, src.custom_params);
    assert_eq!(out.custom_palette, src.custom_palette);
    assert!(!out.watermark && !out.series_approx);
    let d = SessionState::default();
    assert_eq!(out.last_script, d.last_script, "a path travelled");
    assert_eq!(out.export_dir, d.export_dir, "a path travelled");
    assert_eq!(out.ui_scale, d.ui_scale);
    assert_eq!(out.update_track, d.update_track);
    // And the round trip is exact.
    assert_eq!(RenderSettings::from_session(&out), rs);
}

#[test]
fn the_defaults_validate_and_survive_json() {
    let rs = RenderSettings::from_session(&SessionState::default());
    rs.validate().expect("defaults validate");
    let back: RenderSettings = serde_json::from_str(&serde_json::to_string(&rs).unwrap()).unwrap();
    assert_eq!(back, rs);
}

#[test]
fn out_of_range_settings_are_refused() {
    let base = RenderSettings::from_session(&SessionState::default());
    let cases: Vec<(&str, Mutation)> = vec![
        ("zero iterations", Box::new(|r| r.max_iter = 0)),
        ("huge iterations", Box::new(|r| r.max_iter = 4_000_000_000)),
        ("NaN cycle", Box::new(|r| r.cycle = f32::NAN)),
        ("infinite Julia c", Box::new(|r| r.julia_c_re = f64::INFINITY)),
        ("a path as a method", Box::new(|r| r.color_method = "C:\\x\ny".into())),
        ("enormous formula", Box::new(|r| r.custom_formula = "z".repeat(9000))),
        ("escape codes in a formula", Box::new(|r| r.custom_formula = "z\u{1b}[2J".into())),
        ("too many parameters", Box::new(|r| r.custom_params = vec![[0.0, 0.0]; 9])),
        ("NaN palette", Box::new(|r| r.custom_palette = vec![[f32::NAN, 0.0, 0.0, 1.0]])),
    ];
    for (what, f) in cases {
        let mut r = base.clone();
        f(&mut r);
        assert!(r.validate().is_err(), "accepted {what}");
    }
    let unknown = serde_json::to_string(&base).unwrap().replacen('{', "{\"export_dir\":\"D:/x\",", 1);
    assert!(serde_json::from_str::<RenderSettings>(&unknown).is_err(), "an unlisted field was accepted");
}
