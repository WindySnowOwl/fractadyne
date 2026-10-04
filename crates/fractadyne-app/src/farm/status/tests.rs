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
