use super::*;

#[test]
fn what_is_typed_is_checked_before_anything_starts() {
    let key = fractadyne_farm::key::FarmKey::generate().expect("a key").to_text();
    let ok = ClientSettings { controller: "192.168.1.20:46733".into(), name: "PLUTO".into(), ..Default::default() };
    assert_eq!(validate(&ok, &key), Ok(()));
    assert_eq!(validate(&ClientSettings { controller: "studio-pc:46733".into(), ..ok.clone() }, &key), Ok(()), "a host name");
    for bad in ["", "192.168.1.20", "192.168.1.20:", ":46733", "192.168.1.20:0", "192.168.1.20:99999"] {
        assert!(validate(&ClientSettings { controller: bad.into(), ..ok.clone() }, &key).is_err(), "accepted controller {bad:?}");
    }
    assert!(validate(&ok, "fdn1-not-a-key").is_err());
    assert!(validate(&ClientSettings { name: "".into(), ..ok.clone() }, &key).is_err());
}

#[test]
fn the_window_s_choices_reach_the_command_line() {
    let key = std::path::Path::new("k.txt");
    let plain = client_args(&ClientSettings::default(), key);
    assert!(!plain.iter().any(|a| a == "--adapters" || a == "--when-idle"), "{plain:?}");
    let s = ClientSettings { adapters: "all".into(), when_idle: true, idle_minutes: 10, ..Default::default() };
    let a = client_args(&s, key);
    let after = |f: &str| a.iter().position(|x| x == f).and_then(|i| a.get(i + 1)).cloned();
    assert_eq!(after("--adapters").as_deref(), Some("all"));
    assert_eq!(after("--when-idle").as_deref(), if cfg!(windows) { Some("10") } else { None });
}

#[test]
fn several_sessions_read_as_one_machine() {
    let s = |k: u32, phase, done, mean| ClientStatus { slot: Some(k), name: format!("PLUTO · GPU {k}"), phase, frames_done: done, mean_ms: mean, last_frame_seq: done, last_frame: Some(format!("gpu{k}.png")), ..Default::default() };
    let m = merged(&[s(1, ClientPhase::Idle, 3, Some(1000.0)), s(2, ClientPhase::Rendering, 1, Some(2000.0))]).expect("merged");
    assert_eq!(m.phase, ClientPhase::Rendering, "the busiest session's phase");
    assert_eq!(m.detail, "Rendering on 1 of 2 graphics cards", "the machine's words, not one card's run");
    assert_eq!(m.name, "PLUTO");
    assert_eq!(m.frames_done, 4);
    assert_eq!(m.mean_ms, Some(1250.0), "weighted by frames");
    assert_eq!(m.last_frame.as_deref(), Some("gpu1.png"), "the thumbnail of the session that sent the most");
    assert_eq!(m.slot, None);
    let ended = |k, code| ClientStatus { slot: Some(k), phase: ClientPhase::Ended, exit_code: Some(code), ..Default::default() };
    assert_eq!(merged(&[ended(1, 0), ended(2, 3)]).and_then(|m| m.exit_code), Some(3), "one session refused: the client did not end well");
    assert!(merged(&[]).is_none());
}

#[test]
fn every_phase_has_words() {
    for p in [ClientPhase::Connecting, ClientPhase::Checking, ClientPhase::Idle, ClientPhase::Rendering, ClientPhase::Paused, ClientPhase::Retrying, ClientPhase::Ended] {
        assert!(!phase_words(&ClientStatus { phase: p, ..Default::default() }).0.is_empty());
    }
    assert!(phase_words(&ClientStatus { phase: ClientPhase::Retrying, retry_in_s: Some(8), ..Default::default() }).0.contains("in 8 s"));
}

/// The window translates the client's status line by matching the English the client process
/// writes (`detail_words`): that process never loads a language. Reword a line in `farm::client`
/// and the window would quietly show it in English again; this pins every phrase the matcher knows
/// to both sides, so the reword fails here instead.
#[test]
fn every_status_phrase_the_window_translates_is_one_the_client_writes() {
    const CLIENT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/farm/client.rs"));
    const WINDOW: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/ui/farm_client.rs"));
    // As the sources spell them (a quote inside a string is `\"`).
    for phrase in [
        r#""Disconnected""#,
        "The job is over",
        "Paused — the controller has been told",
        "Pausing — finishing the frame in progress",
        "Cancelled the frame in progress; paused",
        "Connecting to ",
        r#"\" — running the self-check"#,
        r#"\" — waiting for its user to approve this machine"#,
        r#"\" — idle"#,
        r#"\" — receiving its bundle"#,
        r#"\" — waiting for frames to render"#,
        "Rendering frames ",
        "Rendering strips ",
        "Waiting — someone is using this machine; work resumes once nobody has for ",
        "The self-check failed — ",
        "Refused by the controller: ",
        "Refused this job: ",
        "The controller closed the connection: ",
        "The controller ended the connection: ",
    ] {
        assert!(CLIENT.contains(phrase), "farm::client no longer writes {phrase:?}: update detail_words");
        assert!(WINDOW.contains(phrase), "detail_words no longer matches {phrase:?}");
    }
}
