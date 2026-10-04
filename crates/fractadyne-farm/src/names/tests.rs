use super::*;

#[test]
fn ordinary_job_ids_and_prefixes_pass() {
    for ok in ["tour", "grand-tour", "a1b2c3d4e5f60718", "frame_2026", "X"] {
        assert!(check_file_part("prefix", ok).is_ok(), "{ok}");
    }
    assert!(check_file_part("prefix", &"a".repeat(MAX_NAME)).is_ok());
}

/// ⛔Every shape a hostile or broken controller could use to leave its folder.
#[test]
fn anything_that_could_escape_a_folder_is_refused() {
    for bad in [
        "", "..", "../x", "a/b", "a\\b", "c:x", "a:b", "a b", "a.png", ".hidden", "-rf", "a\0b",
        "tour\n", "CON", "con", "NUL", "com1", "LPT9", "aux", "é",
    ] {
        assert!(check_file_part("prefix", bad).is_err(), "accepted {bad:?}");
    }
    assert!(check_file_part("prefix", &"a".repeat(MAX_NAME + 1)).is_err());
    // A device-like name that is not one is fine.
    assert!(check_file_part("prefix", "COM10").is_ok());
    assert!(check_file_part("prefix", "console").is_ok());
}

#[test]
fn display_names_allow_text_but_not_control_characters() {
    assert!(check_display_name("PLUTO (RX 6800 XT)").is_ok());
    assert!(check_display_name("工作站").is_ok());
    for bad in ["", "a\nb", "\u{1b}[31mred", &"x".repeat(65)] {
        assert!(check_display_name(bad).is_err(), "accepted {bad:?}");
    }
}

#[test]
fn frame_names_match_the_tour_renderer() {
    assert_eq!(frame_file_name("tour", 7), "tour_00007.png");
    assert_eq!(frame_file_name("tour", 123_456), "tour_123456.png");
}

#[test]
fn a_share_folder_is_built_from_checked_names_only() {
    let root = std::path::Path::new("S:/renders");
    assert_eq!(
        share_dir(root, "a1b2c3d4e5f60718", "STUDIO PC (local)").unwrap(),
        root.join("fractadyne-farm").join("a1b2c3d4e5f60718").join("STUDIO_PC__local_")
    );
    // A job id is a name, never a path.
    for bad in ["../x", "a/b", "C:", ""] {
        assert!(share_dir(root, bad, "PLUTO").is_err(), "accepted job id {bad:?}");
    }
    // A machine name that tries to climb is flattened into one component.
    let d = share_dir(root, "j", "../../etc").unwrap();
    assert_eq!(d.parent().unwrap(), root.join("fractadyne-farm").join("j"));
}
