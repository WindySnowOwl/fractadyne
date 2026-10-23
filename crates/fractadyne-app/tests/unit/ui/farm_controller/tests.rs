use super::*;

#[test]
fn every_phase_has_words_and_a_stop_says_so() {
    for p in [ControllerPhase::Waiting, ControllerPhase::Preparing, ControllerPhase::Rendering, ControllerPhase::Paused, ControllerPhase::Finished] {
        assert!(!phase_words(&ControllerStatus { phase: p, ..Default::default() }).0.is_empty());
    }
    let stopped = ControllerStatus { phase: ControllerPhase::Finished, exit_code: Some(4), ..Default::default() };
    assert_eq!(phase_words(&stopped).0, "Stopped");
    let done = ControllerStatus { phase: ControllerPhase::Finished, exit_code: Some(0), ..Default::default() };
    assert_eq!(phase_words(&done), ("Complete".to_string(), 1));
}

#[test]
fn times_and_space_read_as_a_person_would_say_them() {
    assert_eq!(duration(42.4), "42 s");
    assert_eq!(duration(2290.0), "38m 10s");
    assert_eq!(duration(7500.0), "2h 05m");
    assert_eq!(duration(-3.0), "0 s");
    assert_eq!(space(412 << 30), "442 GB");
    assert_eq!(space(51_700_000_000), "51.7 GB");
    assert_eq!(space(830_000_000), "830 MB");
}

#[test]
fn the_copied_command_is_the_one_that_runs() {
    let f = FarmControllerUi { script: Some(PathBuf::from("C:/tours/grand tour.toml")), out: "C:/frames".into(), min_clients: 3, local: true, ..Default::default() };
    let a = f.args();
    assert_eq!(a[0], "--farm-render");
    assert!(a.windows(2).any(|w| w == ["--listen", "0.0.0.0:46733"]));
    assert!(a.windows(2).any(|w| w == ["--min-clients", "3"]));
    assert_eq!(a.last().map(String::as_str), Some("--local"));
    assert!(!a.iter().any(|x| x == FLAG), "the copied command is for a terminal: no --ui-status");
    assert_eq!(quote("C:/tours/grand tour.toml"), "\"C:/tours/grand tour.toml\"");
}

/// The window translates the controller's status and each client's state by matching the English
/// the controller process writes (`detail_words`, `state_words`). This pins every phrase they know to
/// `farm::controller`, so rewording one there fails here rather than turning the window English.
#[test]
fn every_status_phrase_the_window_translates_is_one_the_controller_writes() {
    const CONTROLLER: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/farm/controller.rs"));
    const WINDOW: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/ui/farm_controller.rs"));
    for phrase in [
        "Measuring the normalize anchors on this machine, once for every client",
        r#""Rendering""#,
        "Waiting for space in the output folder",
        "Paused: no new work is handed out",
        r#""Complete""#,
        "Finished with frames not rendered — run again on the same folder to resume",
        "Stopped — run again on the same folder to resume",
        r#""Finishing""#,
        "Ended (exit ",
        "Waiting for render clients: ",
        "removed by you",
        "removing — collecting its diagnostics",
        r#""gone""#,
        r#""leaving""#,
        "checking (self-check)",
        "paused by its user",
        "parked: crashed twice — rejoins when it reconnects",
        "refused this job (its own limits)",
        "starting a run",
        r#""idle""#,
        "admitted — waiting for the job",
        "removed: ",
        r#""rendering "#,
    ] {
        assert!(CONTROLLER.contains(phrase), "farm::controller no longer writes {phrase:?}: update the window's matcher");
        assert!(WINDOW.contains(phrase), "the window no longer matches {phrase:?}");
    }
}
