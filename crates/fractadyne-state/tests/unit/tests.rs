use super::*;

fn roundtrip(s: &SessionState) -> SessionState {
    let text = toml::to_string_pretty(s).expect("serialize");
    toml::from_str(&text).expect("deserialize")
}

// The reported bug: the *uncapped* frame-rate (once `None`) was dropped by TOML and
// reloaded as the default 60. Now stored as `0.0` (uncapped), it round-trips.
#[test]
fn fps_cap_roundtrips_including_uncapped() {
    let mut s = SessionState { fps_cap: 0.0, ..Default::default() }; // uncapped
    assert_eq!(roundtrip(&s).fps_cap, 0.0);
    s.fps_cap = 120.0;
    assert_eq!(roundtrip(&s).fps_cap, 120.0);
}

// The view state (fractal family, Julia, dual) and the SA preference now persist.
#[test]
fn view_and_preference_fields_roundtrip() {
    let s = SessionState {
        fractal: "Burning Ship".to_string(),
        julia_mode: true,
        julia_c_re: 0.285,
        julia_c_im: 0.01,
        dual: true,
        series_approx: false,
        show_orbits: true,
        orbit_normalize: true,
        orbit_anim: true,
        orbit_anim_speed: 4.5,
        ui_scale: 1.25,
        click_zoom: true,
        click_zoom_factor: 50.0,
        ..SessionState::default()
    };
    let r = roundtrip(&s);
    assert_eq!(r.fractal, "Burning Ship");
    assert!(r.julia_mode && r.dual && !r.series_approx);
    assert_eq!(r.julia_c_re, 0.285);
    assert_eq!(r.julia_c_im, 0.01);
    assert!(r.show_orbits && r.orbit_normalize && r.orbit_anim);
    assert_eq!(r.orbit_anim_speed, 4.5);
    assert_eq!(r.ui_scale, 1.25);
    assert!(r.click_zoom);
    assert_eq!(r.click_zoom_factor, 50.0);
}

/// Checklist step 101: "it reopens on the SAME view you left, with the same fractal, palette
/// and settings." `view_and_preference_fields_roundtrip` covers the fractal and the
/// preferences but NOT the view itself or the coloring, which is the half a user would
/// actually notice — reopening somewhere else is the whole failure.
///
/// The center is checked as a STRING at 60 digits. `center_x`/`center_y` are f64 and exist
/// only as a fallback for old saves; a deep session that round-tripped through them would
/// come back looking fine and be somewhere else entirely, which is the same defect the .kfr
/// import carries a test for.
#[test]
fn a_deep_view_and_its_colouring_survive_a_restart() {
    const CX: &str = "-1.768624905085617234635344164507495322634855357705911897031344";
    const CY: &str = "0.004196591767043058673358411994627633757134484760240109338718";
    let s = SessionState {
        center_x_str: CX.to_string(),
        center_y_str: CY.to_string(),
        // Depth past f64's range: mantissa plus a base-2 exponent. 2^-1669 is about 1e502x,
        // and an implementation that dropped the exponent would land at 1x while every other
        // field still looked correct.
        units_per_pixel: 1.0,
        units_per_pixel_e: -1669,
        max_iter: 200_000,
        auto_iter: false,
        coloring: ColoringSpec {
            palette_idx: 2,
            cycle: 0.51,
            offset: 0.10,
            ..ColoringSpec::default()
        },
        ..SessionState::default()
    };
    let r = roundtrip(&s);

    assert_eq!(r.center_x_str, CX, "deep center lost digits across a save/load");
    assert_eq!(r.center_y_str, CY, "deep center lost digits across a save/load");
    assert_eq!(r.units_per_pixel, 1.0);
    assert_eq!(r.units_per_pixel_e, -1669, "the depth exponent was dropped");
    assert_eq!(r.max_iter, 200_000);
    assert!(!r.auto_iter);
    assert_eq!(r.coloring.palette_idx, 2);
    assert_eq!(r.coloring.cycle, 0.51);
    assert_eq!(r.coloring.offset, 0.10);
}

/// A custom gradient is a user's own work, and it is the one setting they cannot recreate
/// from memory if a restart drops it.
#[test]
fn a_custom_palette_survives_a_restart() {
    let stops = vec![
        [0.0, 0.0, 0.0, 1.0],
        [0.25, 0.9, 0.1, 0.2],
        [1.0, 0.13, 0.42, 0.87],
    ];
    let s = SessionState {
        coloring: ColoringSpec {
            custom_palette: stops.clone(),
            use_custom_palette: true,
            ..ColoringSpec::default()
        },
        ..SessionState::default()
    };
    let r = roundtrip(&s);
    assert!(r.coloring.use_custom_palette);
    assert_eq!(r.coloring.custom_palette, stops, "the custom gradient did not round-trip");
    // An editor-authored gradient is stops, never bands.
    assert!(!r.coloring.custom_palette_flat);
}

/// ⭐An imported `.map` is the same `custom_palette` field read as BANDS, and losing that one flag
/// across a restart would silently smooth a Fractint palette into a gradient — the exact
/// substitution `design/palette-import.md` exists to prevent. It defaults to `false`, so a session
/// written before palette import existed still reads its stops as stops.
#[test]
fn an_imported_map_stays_banded_across_a_restart() {
    let s = SessionState {
        coloring: ColoringSpec {
            custom_palette: vec![[0.0, 1.0, 0.0, 0.0], [0.5, 0.0, 1.0, 0.0], [1.0, 0.0, 0.0, 1.0]],
            custom_palette_flat: true,
            use_custom_palette: true,
            ..ColoringSpec::default()
        },
        ..SessionState::default()
    };
    assert!(roundtrip(&s).coloring.custom_palette_flat, "the .map banding flag did not round-trip");
}

// A legacy file (only the original required fields) must still load, filling new fields
// from their defaults.
#[test]
fn legacy_file_loads_with_defaults() {
    let legacy = "center_x = -0.5\ncenter_y = 0.0\nunits_per_pixel = 0.004\n\
                  max_iter = 256\nauto_iter = true\npalette_idx = 0\ncycle = 0.27\noffset = 0.1\n";
    let s: SessionState = toml::from_str(legacy).expect("legacy load");
    assert_eq!(s.fps_cap, 60.0); // default cap
    assert_eq!(s.fractal, "Mandelbrot");
    assert!(s.series_approx);
}

// A legacy file (no `state_version`) is treated as v1 and loads without a warning.
#[test]
fn missing_version_is_ok_not_a_warning() {
    let legacy = "center_x = -0.5\ncenter_y = 0.0\nunits_per_pixel = 0.004\n\
                  max_iter = 256\nauto_iter = true\npalette_idx = 0\ncycle = 0.27\noffset = 0.1\n";
    let (s, status) = parse_with_status(legacy);
    assert_eq!(s.state_version, 1);
    assert_eq!(status, StateLoad::Ok);
}

// A file from a FUTURE build (higher state_version) still loads best-effort, but flags Newer
// so the app can warn — even when unknown extra keys are present.
#[test]
fn newer_version_file_warns() {
    let future = format!(
        "state_version = {}\ncenter_x = -0.5\ncenter_y = 0.0\nunits_per_pixel = 0.004\n\
         max_iter = 256\nauto_iter = true\npalette_idx = 0\ncycle = 0.1\noffset = 0.0\n\
         some_future_key = \"whatever\"\n",
        STATE_FORMAT_VERSION + 5
    );
    let (_s, status) = parse_with_status(&future);
    assert_eq!(status, StateLoad::Newer(STATE_FORMAT_VERSION + 5));
}

// The current-version round-trip is understood (Ok).
#[test]
fn current_version_roundtrips_ok() {
    let text = toml::to_string_pretty(&SessionState::default()).expect("serialize");
    let (s, status) = parse_with_status(&text);
    assert_eq!(s.state_version, STATE_FORMAT_VERSION);
    assert_eq!(status, StateLoad::Ok);
}

// Garbage that isn't valid TOML falls back to defaults without a scary version warning — but
// it reports UNREADABLE, not Fresh: a file that exists and was ignored is a different fact
// from no file at all, and a harness staging a session needs to tell them apart.
#[test]
fn corrupt_file_falls_back_to_defaults_and_says_so() {
    let (_s, status) = parse_with_status("this is not : valid = toml [[[");
    assert_eq!(status, StateLoad::Unreadable);
}

// A session missing a REQUIRED field (one without `serde(default)`) is the harness hazard: it
// looks like a plausible file and renders with defaults. Since 2026-11-16 what it has is kept,
// and the keys it lacks are NAMED, so the status still says the file was not all there.
#[test]
fn partial_session_is_named_not_silently_default() {
    let (s, status) = parse_with_status("center_x = -0.25\ncenter_y = 0.5\n");
    let mut want: Vec<String> =
        ["auto_iter", "cycle", "max_iter", "offset", "palette_idx", "units_per_pixel"].map(String::from).to_vec();
    let StateLoad::Partial(mut got) = status else { panic!("{status:?}") };
    got.sort();
    want.sort();
    assert_eq!(got, want, "the required keys the file lacks are named");
    assert_eq!((s.center_x, s.center_y), (-0.25, 0.5), "what the file has is kept");
}

/// Review finding 1 (2026-11-16): ONE value of the wrong type made the whole session unreadable,
/// the app started on defaults, and the first save replaced every setting the user had. Now that
/// key alone goes back to its default and is named.
#[test]
fn one_bad_value_costs_that_setting_alone() {
    let text = "state_version = 1\ncenter_x = -0.75\ncenter_y = 0.1\nunits_per_pixel = 0.001\n\
                max_iter = 4321\nauto_iter = false\npalette_idx = 1\npalette_id = \"ice\"\ncycle = 0.37\n\
                offset = 0.25\ntheme = \"light\"\nexport_width = 1234\nlanguage = \"de\"\naa = \"three\"\n";
    let (s, status) = parse_with_status(text);
    assert_eq!(status, StateLoad::Partial(vec!["aa".to_string()]));
    assert_eq!(s.aa, SessionState::default().aa, "the bad key is back at its default");
    assert_eq!((s.max_iter, s.theme.as_str(), s.export_width, s.language.as_str()), (4321, "light", 1234, "de"));
    assert_eq!(s.coloring.palette_id, "ice", "a flattened coloring key is kept too");
    assert!(!s.auto_iter);
    // Anti-vacuity: none of those is the default.
    let d = SessionState::default();
    assert!(d.max_iter != 4321 && d.theme != "light" && d.export_width != 1234 && d.language != "de");
}

/// A gradient segment with a value out of its field's range (`blend` is a u8) costs the gradient,
/// not the session.
#[test]
fn a_bad_gradient_segment_costs_the_gradient_alone() {
    let text = "center_x = -0.5\ncenter_y = 0.0\nunits_per_pixel = 0.004\nmax_iter = 999\nauto_iter = true\n\
                palette_idx = 0\ncycle = 0.27\noffset = 0.1\n\n[[custom_segments]]\nleft = 0.0\nmid = 0.5\n\
                right = 1.0\nleft_color = [0.0, 0.0, 0.0, 1.0]\nright_color = [1.0, 1.0, 1.0, 1.0]\nblend = 300\n";
    let (s, status) = parse_with_status(text);
    assert_eq!(status, StateLoad::Partial(vec!["custom_segments".to_string()]));
    assert_eq!(s.max_iter, 999);
    assert!(s.coloring.custom_segments.is_empty());
}

/// A newer build's file that this build cannot read whole still says Newer (so the app warns), and
/// still keeps what reads.
#[test]
fn a_newer_file_that_does_not_read_whole_keeps_what_reads() {
    let text = format!(
        "state_version = {}\ncenter_x = -0.5\ncenter_y = 0.0\nunits_per_pixel = 0.004\nmax_iter = 777\n\
         auto_iter = true\npalette_idx = 0\ncycle = 0.27\noffset = 0.1\naa = \"three\"\n",
        STATE_FORMAT_VERSION + 1
    );
    let (s, status) = parse_with_status(&text);
    assert_eq!(status, StateLoad::Newer(STATE_FORMAT_VERSION + 1));
    assert_eq!(s.max_iter, 777);
}

/// A file with a byte that is not UTF-8 (an edit in a program saving in the system's code page)
/// read as NO file, and the next save replaced it. It now reads, says Partial, and a copy of it is
/// kept before the app can save over it.
#[test]
fn a_session_that_is_not_utf8_is_read_and_kept() {
    let t = TempConfig::new("not_utf8_test");
    let mut bytes = b"center_x = -0.5\ncenter_y = 0.0\nunits_per_pixel = 0.004\nmax_iter = 888\nauto_iter = true\n\
                      palette_idx = 0\ncycle = 0.27\noffset = 0.1\nexport_dir = 'D:\\caf"
        .to_vec();
    bytes.extend_from_slice(b"\xE9'\n");
    let path = t.cfg.join("session.toml");
    std::fs::write(&path, &bytes).unwrap();
    let (s, status, kept) = load_for_app();
    assert_eq!(status, StateLoad::Partial(Vec::new()));
    assert_eq!(s.max_iter, 888);
    let kept = kept.expect("a copy is kept");
    assert_eq!(kept.file_name().unwrap(), "session.unreadable.toml");
    assert_eq!(std::fs::read(&kept).unwrap(), bytes, "the copy is the file as it was");
    // A second launch on the same file reuses the copy instead of making another.
    let (_, _, again) = load_for_app();
    assert_eq!(again.as_deref(), Some(kept.as_path()));
    assert!(!t.cfg.join("session.unreadable-2.toml").exists());
}

/// The app's whole fix, through the real files: a damaged session is copied aside on load, so the
/// save that follows cannot destroy it, and the save keeps every setting that read.
#[test]
fn a_damaged_session_is_kept_before_the_app_saves_over_it() {
    let t = TempConfig::new("kept_test");
    let path = t.cfg.join("session.toml");
    let text = "center_x = -0.5\ncenter_y = 0.0\nunits_per_pixel = 0.004\nmax_iter = 4321\nauto_iter = true\n\
                palette_idx = 0\ncycle = 0.27\noffset = 0.1\ntheme = \"light\"\naa = \"three\"\n";
    std::fs::write(&path, text).unwrap();
    let (s, status, kept) = load_for_app();
    assert_eq!(status, StateLoad::Partial(vec!["aa".to_string()]));
    save(&s).expect("save");
    assert_eq!(std::fs::read_to_string(kept.expect("kept")).unwrap(), text, "the damaged file survives the save");
    let (back, status) = load_with_status();
    assert_eq!(status, StateLoad::Ok);
    assert_eq!((back.max_iter, back.theme.as_str()), (4321, "light"));
    // A readable session keeps no copy.
    let (_, status, kept) = load_for_app();
    assert_eq!((status, kept), (StateLoad::Ok, None));
}

/// Every field of the session, set to something other than its default, comes back through the
/// real file. ⭐A struct literal with NO `..Default`: a field added to `SessionState` does not
/// compile here until it is given a value, so no setting can be added without this test covering
/// its round trip (review 2026-11-16: no test held every field at once).
#[test]
fn every_session_field_survives_save_and_reload() {
    let _t = TempConfig::new("every_field_test");
    let c = ColoringSpec {
        palette_idx: 5,
        palette_id: "ice".into(),
        cycle: 0.123,
        iter_div: 7,
        offset: -0.5,
        log_palette: true,
        transfer: "log".into(),
        wrap: "mirror".into(),
        steps: 12,
        normalize_live: false,
        normalize_fit: true,
        light: true,
        light_angle: 1.25,
        light_height: 2.5,
        light_steep: true,
        light_steep_px: 20.0,
        light_highlight: 0.75,
        light_shininess: 32.0,
        light_highlight_color: [0.9, 0.7, 0.3],
        de: true,
        de_strength: 0.75,
        de_width: 1.5,
        edge: true,
        edge_inside: true,
        edge_width: 3.5,
        edge_strength: 0.4,
        edge_curve: "smooth".into(),
        edge_color: [0.1, 0.2, 0.9],
        equipotentials: true,
        equipotential_spacing: 24.0,
        equipotential_width: 2.5,
        equipotential_strength: 0.35,
        equipotential_color: [0.9, 0.8, 0.1],
        color_method: "trap".into(),
        stripe_freq: 3.5,
        stripe_tail: true,
        stripe_tail_len: 64,
        trap_type: "cross".into(),
        trap_x: 0.25,
        trap_y: -0.5,
        trap_rotation: 30.0,
        trap_size: 0.75,
        interior_trap: "circle".into(),
        interior_trap_x: 0.1,
        interior_trap_y: 0.4,
        interior_trap_rotation: 15.0,
        interior_trap_size: 0.3,
        trap_phase: 45.0,
        interior_trap_phase: -30.0,
        custom_palette: vec![[0.0, 0.1, 0.2, 0.3], [1.0, 0.4, 0.5, 0.6]],
        custom_palette_flat: true,
        custom_segments: vec![PaletteSegment {
            left: 0.0,
            mid: 0.25,
            right: 1.0,
            left_color: [0.1, 0.2, 0.3, 1.0],
            right_color: [0.9, 0.8, 0.7, 0.5],
            blend: 5,
            space: 2,
            blend_params: [0.1, 0.2, 0.3, 0.4],
        }],
        use_custom_palette: true,
        use_duotone: true,
        use_binary: true,
        duotone_lo: [0.1, 0.2, 0.3],
        duotone_hi: [0.7, 0.8, 0.9],
        interior: "trap".into(),
        interior_palette: "jellyfish".into(),
        interior_color: [0.3, 0.2, 0.1],
        interior_fit: true,
        palette_aa: false,
        palette_brightness: 0.35,
        palette_contrast: -0.45,
        palette_saturation: 1.25,
        julia_linked: true,
        equalize: 0.75,
        palette_oklab: true,
        coloring_hook: "// @param p1 gain 0 2 1\nfn custom_color(l: Lanes) -> vec3<f32> {\n    return l.color * p1();\n}\n".into(),
        coloring_hook_params: vec![0.5, 1.0, -2.0, 0.25],
    };
    let want = SessionState {
        state_version: STATE_FORMAT_VERSION,
        center_x: 0.25,
        center_y: -0.125,
        center_x_str: "2.5e-1".into(),
        center_y_str: "-1.25e-1".into(),
        center_expr: "center_re_expr=1/3\ncenter_im_expr=0\n".into(),
        units_per_pixel: 1.5,
        units_per_pixel_e: -4000,
        max_iter: 123_456,
        auto_iter: false,
        coloring: c,
        coloring_hook_path: "D:/hooks/gain.wgsl".into(),
        zoom_rate: 2.5,
        click_zoom: true,
        click_zoom_factor: 25.0,
        click_zoom_snap: true,
        snap_zoom: false,
        autopilot_dive_log2: 1234.5,
        autopilot_priority: "quality".into(),
        autopilot_target: "misiurewicz".into(),
        work_budget_scale: 3.0,
        min_motion_res: 0.75,
        prefer_detail: true,
        live_worker_gpu: "off".into(),
        show_timestamp: true,
        show_zoom_target: true,
        live_panel: "panel".into(),
        live_panel_corner: "br".into(),
        live_panel_opacity: 0.5,
        aa: 8,
        finish_sound: false,
        fps_cap: 0.0,
        orbit_cache_mb: 2048,
        export_width: 7680,
        export_ss: 3,
        export_format: "exr".into(),
        export_jpeg_quality: 55,
        export_avif_quality: 44,
        export_gif_colors: 17,
        export_gif_dither: false,
        snapshot_mode: "render".into(),
        mpfr_warning_suppressed: true,
        mpfr_present_last: Some(false),
        export_dir: Some("D:/out".into()),
        last_dir: Some("D:/in".into()),
        last_script: Some("D:/tour.toml".into()),
        welcome_seen: true,
        export_dual_mode: "separate".into(),
        export_aspect: "16:9".into(),
        share_include_thumb: false,
        export_open_after: true,
        export_all_gpus: true,
        show_location: true,
        palette_anim: "pingpong".into(),
        palette_anim_speed: 0.5,
        light_anim: true,
        trap_phase_anim: true,
        de_anim: true,
        minimap: true,
        right_panel_open: false,
        perf_panel: true,
        fractal: "Burning Ship".into(),
        custom_formula: "z = z^2 + c\n; a comment".into(),
        custom_params: vec![[0.5, -0.25]],
        formula_textbook: true,
        life: "rule=B3/S23\npattern=bo$2bo$3o!\n".into(),
        lsystem: "lsystem=name x\\naxiom F\n".into(),
        julia_mode: true,
        julia_c_re: -0.4,
        julia_c_im: 0.6,
        dual: true,
        dual_split: 0.4,
        julia_view_center_re: "1.5e-1".into(),
        julia_view_center_im: "-2.5e-1".into(),
        julia_view_upp_log2: Some(-30.5),
        series_approx: false,
        glitch_correct: false,
        use_bla: false,
        watermark: false,
        ui_scale: 1.25,
        theme: "light".into(),
        update_track: "beta".into(),
        update_check_on_launch: true,
        language: "ja".into(),
        language_system_seen: "ja-JP".into(),
        decimal_mark: ",".into(),
        show_watermark: false,
        crash_prompt_disabled: true,
        show_orbits: true,
        orbit_normalize: true,
        orbit_anim: true,
        orbit_anim_speed: 3.0,
    };
    save(&want).expect("save");
    let (got, status) = load_with_status();
    assert_eq!(status, StateLoad::Ok);
    assert_eq!(got, want, "a field did not come back as it was saved");
    // Anti-vacuity, field by field through the TOML: every key differs from the default's.
    let d = toml::Value::try_from(SessionState::default()).unwrap();
    let w = toml::Value::try_from(&want).unwrap();
    let (toml::Value::Table(d), toml::Value::Table(w)) = (d, w) else { panic!() };
    let same: Vec<&String> =
        w.iter().filter(|(k, v)| *k != "state_version" && d.get(*k) == Some(*v)).map(|(k, _)| k).collect();
    assert!(same.is_empty(), "these fields are at their default, so the test cannot see them: {same:?}");
}

/// `FRACTADYNE_CONFIG_DIR` is process-global and `cargo test` runs tests on parallel
/// threads, so every test that touches it takes this first. Without it, one test's
/// `remove_var` lands in the middle of another's `config_dir()` and the failure is a
/// once-in-a-while red that reruns green.
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// A throwaway config dir, removed when the guard drops.
struct TempConfig {
    root: std::path::PathBuf,
    cfg: std::path::PathBuf,
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl TempConfig {
    fn new(tag: &str) -> Self {
        let guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let root = std::env::temp_dir()
            .join(format!("fractadyne_{tag}_{}", std::process::id()));
        let cfg = root.join("cfg");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&cfg).unwrap();
        std::env::set_var("FRACTADYNE_CONFIG_DIR", &cfg);
        Self { root, cfg, _guard: guard }
    }
}

impl Drop for TempConfig {
    fn drop(&mut self) {
        std::env::remove_var("FRACTADYNE_CONFIG_DIR");
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// Checklist step 93, "change a setting, close and reopen: it persists and takes effect".
/// Run through the REAL file path (`save` then `load`), not a serde round trip in memory:
/// what fails in the field is the write or the read, and a value that serializes perfectly
/// into a string nobody ever wrote back is still a lost setting.
///
/// The fields swept here are deliberately the non-view ones — the view has its own tests, and
/// preferences are the half a user notices only on the next launch.
#[test]
fn a_setting_survives_save_and_reload() {
    let t = TempConfig::new("settings_test");
    let want = SessionState {
        // one of each shape: bool, float, int, string, enum-as-string, Option<String>
        zoom_rate: 2.75,
        fps_cap: 0.0, // uncapped — the value that a naive Option round trip loses
        aa: 4,
        max_iter: 750_000,
        auto_iter: false,
        minimap: true,
        right_panel_open: false,
        finish_sound: false,
        palette_anim: "pingpong".to_string(),
        palette_anim_speed: 0.42,
        export_width: 3840,
        export_ss: 2,
        export_format: "exr".to_string(),
        export_jpeg_quality: 61,
        export_avif_quality: 73,
        export_gif_colors: 9,
        export_gif_dither: false,
        export_dir: Some("D:/renders".to_string()),
        welcome_seen: true,
        coloring: ColoringSpec {
            normalize_live: false,
            normalize_fit: true,
            log_palette: true,
            palette_idx: 3,
            palette_id: "four-seasons".to_string(),
            color_method: "stripe".to_string(),
            ..ColoringSpec::default()
        },
        ..SessionState::default()
    };
    save(&want).expect("save");
    assert!(t.cfg.join("session.toml").is_file(), "save() wrote no session file");

    let (got, status) = load_with_status();
    assert_eq!(status, StateLoad::Ok, "the session we just wrote did not read back cleanly");
    assert!(!got.coloring.normalize_live, "normalize_live");
    // The range-fit switch is the half users reach for after finding the aliasing guard declines;
    // losing it on reload would put them back where they started, silently.
    assert!(got.coloring.normalize_fit, "normalize_fit");
    assert!(got.coloring.log_palette, "log_palette");
    assert_eq!(got.zoom_rate, 2.75, "zoom_rate");
    assert_eq!(got.fps_cap, 0.0, "fps_cap (uncapped)");
    assert_eq!(got.aa, 4, "aa");
    assert_eq!(got.max_iter, 750_000, "max_iter");
    assert!(!got.auto_iter, "auto_iter");
    assert_eq!(got.coloring.palette_idx, 3, "palette_idx");
    assert_eq!(got.coloring.palette_id, "four-seasons", "palette_id");
    assert!(got.minimap, "minimap");
    assert!(!got.right_panel_open, "right_panel_open");
    assert!(!got.finish_sound, "finish_sound");
    assert_eq!(got.coloring.color_method, "stripe", "color_method");
    assert_eq!(got.palette_anim, "pingpong", "palette_anim");
    assert_eq!(got.palette_anim_speed, 0.42, "palette_anim_speed");
    assert_eq!(got.export_width, 3840, "export_width");
    assert_eq!(got.export_ss, 2, "export_ss");
    assert_eq!(got.export_format, "exr", "export_format");
    assert_eq!((got.export_jpeg_quality, got.export_avif_quality, got.export_gif_colors, got.export_gif_dither), (61, 73, 9, false), "the JPG, AVIF and GIF settings");
    assert_eq!(got.export_dir.as_deref(), Some("D:/renders"), "export_dir");
    assert!(got.welcome_seen, "welcome_seen");

    // Anti-vacuity: every value above differs from the default, so none of these assertions
    // could be satisfied by a load that quietly returned `SessionState::default()`.
    let d = SessionState::default();
    assert_ne!(d.zoom_rate, want.zoom_rate);
    assert_ne!(d.coloring.color_method, want.coloring.color_method);
    assert_ne!(d.aa, want.aa);
    assert_ne!(d.coloring.normalize_live, want.coloring.normalize_live);
    assert_ne!(d.coloring.normalize_fit, want.coloring.normalize_fit);
}

/// Checklist step 96, "settings live in the user profile, not beside the executable, so two
/// builds share them and nothing needs importing".
///
/// The failure this guards is portable-app-shaped: a config dir derived from the executable's
/// location means the standard and accelerated builds, unpacked side by side, each keep their
/// own session and bookmarks — and the user's locations appear to have vanished.
#[test]
fn config_lives_in_the_user_profile() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("FRACTADYNE_CONFIG_DIR");
    let dir = config_dir().expect("no config dir on this platform");

    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .expect("current_exe");
    assert!(
        !dir.starts_with(&exe_dir),
        "config lives beside the executable ({}), so two builds would not share it",
        dir.display()
    );
    // It IS under the OS's per-user config location.
    let base = directories::ProjectDirs::from("com", "Fractadyne", "Fractadyne")
        .expect("no OS project dirs");
    assert_eq!(
        dir.as_path(),
        base.config_dir(),
        "the config dir is not the OS per-user location"
    );
    // ...and that location is a per-user one, not a shared or relative path.
    assert!(dir.is_absolute(), "config dir {} is not absolute", dir.display());
    // The override still wins — it is what makes every harness run hermetic.
    let over = std::env::temp_dir().join("fractadyne_cfg_override_probe");
    std::env::set_var("FRACTADYNE_CONFIG_DIR", &over);
    assert_eq!(config_dir().as_deref(), Some(over.as_path()), "the override was ignored");
    std::env::remove_var("FRACTADYNE_CONFIG_DIR");
}

// reset_all removes the whole config dir, and is a no-op when there's nothing to remove.
// Uses the FRACTADYNE_CONFIG_DIR override so it operates on a throwaway dir, never real data.
#[test]
fn reset_all_removes_config_dir_via_override() {
    let _env = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let root = std::env::temp_dir().join(format!("fractadyne_reset_test_{}", std::process::id()));
    let cfg = root.join("cfg");
    std::fs::create_dir_all(cfg.join("bookmark_thumbs")).unwrap();
    std::fs::write(cfg.join("session.toml"), "state_version = 1\n").unwrap();
    std::fs::write(cfg.join("bookmarks.toml"), "").unwrap();
    std::env::set_var("FRACTADYNE_CONFIG_DIR", &cfg);
    assert_eq!(config_dir().as_deref(), Some(cfg.as_path()));
    assert!(cfg.exists());
    assert!(reset_all().unwrap()); // removed
    assert!(!cfg.exists());
    assert!(!reset_all().unwrap()); // nothing left to remove
    std::env::remove_var("FRACTADYNE_CONFIG_DIR");
    let _ = std::fs::remove_dir_all(&root);
}

/// ⭐⭐**An imported `.ggr` is RICHER than a stop list, and the session must hold all of it.** The
/// midpoint, the blend curve and the color space are exactly what a `[pos, r, g, b]` list cannot
/// express, so persisting one as stops would have flattened every curve and every hue sweep on the
/// first restart — silently, and visible only as "it looks different than when I imported it".
#[test]
fn an_imported_segment_gradient_survives_a_restart() {
    let segs = vec![
        PaletteSegment {
            left: 0.0,
            mid: 0.17,           // deliberately NOT centered
            right: 0.5,
            left_color: [1.0, 0.0, 0.0, 1.0],
            right_color: [0.0, 0.0, 1.0, 0.5],
            blend: 2,            // sine
            space: 1,            // HSV counter-clockwise
            blend_params: [0.0; 4],
        },
        PaletteSegment {
            left: 0.5,
            mid: 0.75,
            right: 1.0,
            left_color: [0.0, 0.0, 1.0, 0.5],
            right_color: [1.0, 1.0, 0.0, 1.0],
            blend: 4,            // spherical decreasing
            space: 0,
            // ⭐**Kind 5's parameters, written on a kind-4 segment on purpose.** The field is
            // independent of the kind beside it, so the file format has to carry it either way —
            // a serde `default` that silently dropped a value present in the file would be
            // invisible until someone's saved Bézier came back as a straight line.
            // ⚠This pins the FORMAT, not the editor: in the app the parameters live in the
            // `Blend::Bezier` variant, so a kind 0–4 segment has none to write and switching a
            // segment away from Bézier and back re-fits rather than restoring. Stated in
            // `design/gradient-curves.md` §9.6.
            blend_params: [0.25, 0.9, 0.75, 0.1],
        },
    ];
    let s = SessionState {
        coloring: ColoringSpec {
            custom_segments: segs.clone(),
            use_custom_palette: true,
            ..ColoringSpec::default()
        },
        ..SessionState::default()
    };
    let r = roundtrip(&s);
    assert_eq!(r.coloring.custom_segments, segs, "the segment gradient did not round-trip");
    // The three things a stop list cannot hold, named individually so a partial loss is obvious.
    assert_eq!(r.coloring.custom_segments[0].mid, 0.17, "the off-center midpoint was lost");
    assert_eq!(r.coloring.custom_segments[0].blend, 2, "the blend curve was lost");
    assert_eq!(
        r.coloring.custom_segments[1].blend_params,
        [0.25, 0.9, 0.75, 0.1],
        "the Bezier control points were lost - a segment switched to kind 5 and back would come \
         back as a straight line"
    );
    assert_eq!(r.coloring.custom_segments[0].space, 1, "the color space was lost");
    assert_eq!(r.coloring.custom_segments[0].right_color[3], 0.5, "alpha was lost");
}

/// A session written before `.ggr` import existed has no segments, and must read as a plain stop
/// list rather than as an empty gradient.
#[test]
fn a_session_without_segments_still_uses_its_stops() {
    let s = SessionState {
        coloring: ColoringSpec {
            custom_palette: vec![[0.0, 1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]],
            use_custom_palette: true,
            ..ColoringSpec::default()
        },
        ..SessionState::default()
    };
    let r = roundtrip(&s);
    assert!(r.coloring.custom_segments.is_empty());
    assert_eq!(r.coloring.custom_palette.len(), 2);
}

/// ⭐⭐**The Performance panel is OFF for a new user and remembered once turned on.**
///
/// Its readouts (`mode perturb df32`, `precision 84 bit`, `orbit len 12833`) are diagnostics that
/// read as an error report to someone who has just opened a fractal viewer, and it gates a periodic
/// repaint that stops an idle deep view from idling — so off is the right first-run answer.
///
/// ⚠But off-by-default is only tolerable if turning it ON STICKS. eframe is built without
/// `persistence`, so egui discards every collapsing section's state on exit; this field is the app's
/// own storage, and without it the panel would switch itself back on at every launch — which is the
/// complaint that started this, in reverse.
#[test]
fn the_performance_panel_is_off_by_default_and_its_answer_survives_a_restart() {
    assert!(!SessionState::default().perf_panel, "a new user must not meet the diagnostics panel");

    let on = SessionState { perf_panel: true, ..Default::default() };
    assert!(roundtrip(&on).perf_panel, "turning it on must survive a restart");

    let off = SessionState { perf_panel: false, ..Default::default() };
    assert!(!roundtrip(&off).perf_panel, "turning it off must survive a restart too");
}

/// ⚠**A session written before this field existed must load, and load as OFF.** `#[serde(default)]`
/// is what makes that true; without it every older session file becomes UNREADABLE and the user
/// silently loses their view, their palette and their bookmarks' companion state at once.
#[test]
fn a_session_from_before_the_performance_field_still_loads() {
    let text = toml::to_string_pretty(&SessionState::default()).expect("serialize");
    let older: String = text
        .lines()
        .filter(|l| !l.trim_start().starts_with("perf_panel"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!older.contains("perf_panel"), "the guard: the field must really be absent");
    let back: SessionState = toml::from_str(&older).expect("an older session must still parse");
    assert!(!back.perf_panel);
}

/// ⚠**"Open when done" must default OFF and stick when turned on.** Off by default because an
/// export can be one of a batch and a viewer stealing focus mid-session is not something to opt
/// people into silently; sticky because a checkbox that forgets is worse than no checkbox.
#[test]
fn open_after_export_is_off_by_default_and_survives_a_restart() {
    assert!(
        !SessionState::default().export_open_after,
        "a fresh install must not launch a viewer behind the user's back"
    );

    let on = SessionState { export_open_after: true, ..Default::default() };
    assert!(roundtrip(&on).export_open_after, "turning it on must survive a restart");

    let off = SessionState { export_open_after: false, ..Default::default() };
    assert!(!roundtrip(&off).export_open_after, "and turning it back off must too");
}

/// ⚠A session written before this field existed must still load — and load as OFF.
#[test]
fn a_session_from_before_the_open_after_field_still_loads() {
    let text = toml::to_string_pretty(&SessionState::default()).expect("serialize");
    let older: String = text
        .lines()
        .filter(|l| !l.trim_start().starts_with("export_open_after"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!older.contains("export_open_after"), "the guard: the field must really be absent");
    let back: SessionState = toml::from_str(&older).expect("an older session must still parse");
    assert!(!back.export_open_after);
}

/// "Divide iterations" defaults to 1, survives a restart, and a session from before the field
/// still loads, at 1 (`serde(default)`): without it every older session would be unreadable.
#[test]
fn the_iteration_divisor_defaults_to_one_and_survives_a_restart() {
    assert_eq!(SessionState::default().coloring.iter_div, 1);
    let s = SessionState { coloring: ColoringSpec { iter_div: 8, ..ColoringSpec::default() }, ..SessionState::default() };
    assert_eq!(roundtrip(&s).coloring.iter_div, 8);
    let text = toml::to_string_pretty(&SessionState::default()).expect("serialize");
    let older: String =
        text.lines().filter(|l| !l.trim_start().starts_with("iter_div")).collect::<Vec<_>>().join("\n");
    assert!(!older.contains("iter_div"), "the guard: the field must really be absent");
    let back: SessionState = toml::from_str(&older).expect("an older session must still parse");
    assert_eq!(back.coloring.iter_div, 1);
}

/// ⭐A session file written before [`ColoringSpec`] existed (coloring plan B4) reads every coloring
/// setting, and this build writes each back under the same top-level key: the session flattens the
/// spec, so the file's layout did not change and an older build reads a newer file too. The fixture
/// was written by main at 542a5264 with every coloring field set away from its default (and checked
/// both ways against that build: the two files parse to the same table).
#[test]
fn a_session_written_before_the_coloring_spec_reads_and_writes_the_same_keys() {
    let old = include_str!("fixtures/session-pre-b4.toml");
    let s: SessionState = toml::from_str(old).expect("the older session file reads");
    let c = &s.coloring;
    assert_eq!((c.color_method.as_str(), c.trap_type.as_str(), c.palette_id.as_str()), ("trap", "cross", "ember"));
    assert_eq!((c.cycle, c.offset, c.iter_div, c.palette_idx, c.stripe_tail_len), (0.61, 0.33, 4, 7, 64));
    assert!(c.light && c.de && c.use_duotone && c.log_palette && c.normalize_fit && !c.normalize_live);
    assert_eq!((c.light_angle, c.light_height, c.de_strength, c.de_width, c.stripe_freq), (1.25, 0.4, 0.7, 2.5, 9.5));
    assert_eq!((c.duotone_lo, c.duotone_hi), ([0.1, 0.2, 0.3], [0.9, 0.8, 0.7]));
    assert!(c.use_custom_palette && c.custom_palette_flat && c.custom_palette.len() == 2);
    assert_eq!(c.custom_segments.len(), 1);
    assert_eq!(c.custom_segments[0].blend_params, [0.25, 0.9, 0.75, 0.1]);
    let want: toml::Table = toml::from_str(old).unwrap();
    let back: toml::Table = toml::from_str(&toml::to_string_pretty(&s).unwrap()).unwrap();
    assert!(!back.contains_key("coloring"), "the coloring was written as a table of its own");
    let keys = toml::Table::try_from(ColoringSpec::default()).unwrap();
    // Fields added after the fixture was written (B5's interior and later, B10's coloring hook):
    // absent from it, so read as their defaults and written back as such. The file itself stays as
    // that build wrote it.
    const SINCE: &[&str] = &[
        "interior", "interior_palette", "interior_color", "interior_fit", "palette_aa",
        "palette_brightness", "palette_contrast", "palette_saturation", "julia_linked",
        "coloring_hook", "coloring_hook_params", "transfer", "wrap", "steps",
        "edge", "edge_inside", "edge_width", "edge_strength", "edge_curve", "edge_color",
        "equipotentials", "equipotential_spacing", "equipotential_width", "equipotential_strength", "equipotential_color",
        "light_steep", "light_steep_px", "light_highlight", "light_shininess", "light_highlight_color",
        "equalize", "palette_oklab",
        "trap_x", "trap_y", "trap_rotation", "trap_size",
        "interior_trap", "interior_trap_x", "interior_trap_y", "interior_trap_rotation", "interior_trap_size",
        // The sine wave trap's phase (user 2026-11-25).
        "trap_phase", "interior_trap_phase",
    ];
    assert_eq!(keys.len(), 27 + SINCE.len(), "a coloring field was added: list it in SINCE");
    for k in keys.keys() {
        if SINCE.contains(&k.as_str()) {
            assert!(want.get(k).is_none(), "the fixture predates `{k}`");
            assert_eq!(back.get(k), keys.get(k), "`{k}` absent from a file did not read as its default");
        } else {
            assert_eq!(back.get(k), want.get(k), "the coloring key `{k}` changed or moved");
        }
    }
}
