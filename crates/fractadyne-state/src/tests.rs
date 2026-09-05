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
/// preferences but NOT the view itself or the colouring, which is the half a user would
/// actually notice — reopening somewhere else is the whole failure.
///
/// The centre is checked as a STRING at 60 digits. `center_x`/`center_y` are f64 and exist
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
        palette_idx: 2,
        cycle: 0.51,
        offset: 0.10,
        ..SessionState::default()
    };
    let r = roundtrip(&s);

    assert_eq!(r.center_x_str, CX, "deep centre lost digits across a save/load");
    assert_eq!(r.center_y_str, CY, "deep centre lost digits across a save/load");
    assert_eq!(r.units_per_pixel, 1.0);
    assert_eq!(r.units_per_pixel_e, -1669, "the depth exponent was dropped");
    assert_eq!(r.max_iter, 200_000);
    assert!(!r.auto_iter);
    assert_eq!(r.palette_idx, 2);
    assert_eq!(r.cycle, 0.51);
    assert_eq!(r.offset, 0.10);
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
        custom_palette: stops.clone(),
        use_custom_palette: true,
        ..SessionState::default()
    };
    let r = roundtrip(&s);
    assert!(r.use_custom_palette);
    assert_eq!(r.custom_palette, stops, "the custom gradient did not round-trip");
    // An editor-authored gradient is stops, never bands.
    assert!(!r.custom_palette_flat);
}

/// ⭐An imported `.map` is the same `custom_palette` field read as BANDS, and losing that one flag
/// across a restart would silently smooth a Fractint palette into a gradient — the exact
/// substitution `design/palette-import.md` exists to prevent. It defaults to `false`, so a session
/// written before palette import existed still reads its stops as stops.
#[test]
fn an_imported_map_stays_banded_across_a_restart() {
    let s = SessionState {
        custom_palette: vec![[0.0, 1.0, 0.0, 0.0], [0.5, 0.0, 1.0, 0.0], [1.0, 0.0, 0.0, 1.0]],
        custom_palette_flat: true,
        use_custom_palette: true,
        ..SessionState::default()
    };
    assert!(roundtrip(&s).custom_palette_flat, "the .map banding flag did not round-trip");
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

// A session missing a REQUIRED field (one without `serde(default)`) is the harness hazard:
// it looks like a plausible file, parses as nothing, and renders with defaults.
#[test]
fn partial_session_is_unreadable_not_silently_default() {
    let (_s, status) = parse_with_status("center_x = -0.5\ncenter_y = 0.0\n");
    assert_eq!(status, StateLoad::Unreadable);
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
        normalize_live: false,
        log_palette: true,
        zoom_rate: 2.75,
        fps_cap: 0.0, // uncapped — the value that a naive Option round trip loses
        aa: 4,
        max_iter: 750_000,
        auto_iter: false,
        palette_idx: 3,
        minimap: true,
        right_panel_open: false,
        finish_sound: false,
        color_method: "stripe".to_string(),
        palette_anim: "pingpong".to_string(),
        palette_anim_speed: 0.42,
        export_width: 3840,
        export_ss: 2,
        export_format: "exr".to_string(),
        export_dir: Some("D:/renders".to_string()),
        welcome_seen: true,
        ..SessionState::default()
    };
    save(&want);
    assert!(t.cfg.join("session.toml").is_file(), "save() wrote no session file");

    let (got, status) = load_with_status();
    assert_eq!(status, StateLoad::Ok, "the session we just wrote did not read back cleanly");
    assert!(!got.normalize_live, "normalize_live");
    assert!(got.log_palette, "log_palette");
    assert_eq!(got.zoom_rate, 2.75, "zoom_rate");
    assert_eq!(got.fps_cap, 0.0, "fps_cap (uncapped)");
    assert_eq!(got.aa, 4, "aa");
    assert_eq!(got.max_iter, 750_000, "max_iter");
    assert!(!got.auto_iter, "auto_iter");
    assert_eq!(got.palette_idx, 3, "palette_idx");
    assert!(got.minimap, "minimap");
    assert!(!got.right_panel_open, "right_panel_open");
    assert!(!got.finish_sound, "finish_sound");
    assert_eq!(got.color_method, "stripe", "color_method");
    assert_eq!(got.palette_anim, "pingpong", "palette_anim");
    assert_eq!(got.palette_anim_speed, 0.42, "palette_anim_speed");
    assert_eq!(got.export_width, 3840, "export_width");
    assert_eq!(got.export_ss, 2, "export_ss");
    assert_eq!(got.export_format, "exr", "export_format");
    assert_eq!(got.export_dir.as_deref(), Some("D:/renders"), "export_dir");
    assert!(got.welcome_seen, "welcome_seen");

    // Anti-vacuity: every value above differs from the default, so none of these assertions
    // could be satisfied by a load that quietly returned `SessionState::default()`.
    let d = SessionState::default();
    assert_ne!(d.zoom_rate, want.zoom_rate);
    assert_ne!(d.color_method, want.color_method);
    assert_ne!(d.aa, want.aa);
    assert_ne!(d.normalize_live, want.normalize_live);
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
