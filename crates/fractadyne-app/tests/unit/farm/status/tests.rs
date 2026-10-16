use super::*;

#[test]
fn a_status_line_round_trips_and_other_lines_are_not_status() {
    let c = ControllerStatus {
        phase: ControllerPhase::Rendering,
        detail: "rendering".into(),
        frames: 19,
        done: 4,
        strip: "dd.a".into(),
        clients: vec![ClientRow { id: 3, name: "PLUTO".into(), probe_px: Some(812), gpu_class: Some("B".into()), ..Default::default() }],
        ..Default::default()
    };
    let l = line(&c);
    assert!(l.starts_with(PREFIX) && !l.contains('\n'));
    assert_eq!(parse::<ControllerStatus>(&l), Some(c));
    assert_eq!(parse::<ControllerStatus>(&format!("{l}\r\n")), parse::<ControllerStatus>(&l), "a CRLF line end");
    let k = ClientStatus { phase: ClientPhase::Paused, frames_done: 7, run: Some((8, 16)), ..Default::default() };
    assert_eq!(parse::<ClientStatus>(&line(&k)), Some(k));
    assert_eq!(parse::<ClientStatus>("  frame 3 sent (25 KB, 65 ms)"), None);
    assert_eq!(parse::<ClientStatus>("farm-status {not json"), None);
}

#[test]
fn commands_parse_exactly_and_print_back() {
    for c in [ClientCommand::Pause, ClientCommand::Resume, ClientCommand::CancelFrame, ClientCommand::Leave] {
        assert_eq!(ClientCommand::parse(c.text()), Some(c));
    }
    assert_eq!(ClientCommand::parse("PAUSE"), None);
    for c in [ControllerCommand::Pause, ControllerCommand::Resume, ControllerCommand::Stop, ControllerCommand::Remove(12), ControllerCommand::Readmit("STUDIO-PC (local)".into())] {
        assert_eq!(ControllerCommand::parse(&c.text()), Some(c));
    }
    assert_eq!(ControllerCommand::parse("remove"), None);
    assert_eq!(ControllerCommand::parse("remove x"), None);
    assert_eq!(ControllerCommand::parse("readmit"), None);
    assert_eq!(ControllerCommand::parse("stop now"), None);
    assert_eq!(ControllerCommand::parse("rm -rf /"), None);
}

/// The link reads status lines and other lines apart, keeps the latest status, and sees the exit —
/// driven by a stand-in program that prints what a farm process would.
#[cfg(windows)]
#[test]
fn a_link_keeps_the_latest_status_and_the_other_lines() {
    let first = line(&ClientStatus { phase: ClientPhase::Connecting, ..Default::default() });
    let last = line(&ClientStatus { phase: ClientPhase::Idle, frames_done: 3, ..Default::default() });
    // A batch file: JSON's quotes do not survive being passed through `cmd /C` as an argument.
    let dir = std::env::temp_dir().join(format!("fd-link-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let bat = dir.join("stand-in.cmd");
    std::fs::write(&bat, format!("@echo off\r\necho {first}\r\necho Connected to STUDIO-PC\r\necho {last}\r\nexit /b 7\r\n")).expect("write the stand-in");
    let mut l = Link::<ClientStatus>::spawn_program(std::path::Path::new("cmd"), &["/C".into(), bat.to_string_lossy().into_owned()], &[]).expect("cmd runs");
    let t0 = std::time::Instant::now();
    while l.running() || l.status.as_ref().is_none_or(|s| s.frames_done != 3) {
        l.poll();
        assert!(t0.elapsed() < std::time::Duration::from_secs(10), "the stand-in never finished");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(l.status.as_ref().map(|s| (s.phase, s.frames_done)), Some((ClientPhase::Idle, 3)));
    assert!(l.log.iter().any(|x| x.trim() == "Connected to STUDIO-PC"), "{:?}", l.log);
    assert!(!l.log.iter().any(|x| x.contains(PREFIX)), "a status line landed in the log");
    assert_eq!(l.exit, Some(Some(7)));
    let _ = std::fs::remove_dir_all(&dir);
}
