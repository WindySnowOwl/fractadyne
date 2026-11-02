use super::*;

/// The dev box's own `wevtutil qe System … /f:text` output (2026-11-01), the computer's name
/// replaced: two NVIDIA resets with no description, then a Display event with one.
const SAMPLE: &str = "Event[0]
  Log Name: System
  Source: nvlddmkm
  Date: 2026-10-26T14:31:12.7180000Z
  Event ID: 153
  Task: None
  Level: Error
  Opcode: Info
  Keyword: Classic,
  User: N/A
  User Name: N/A
  Computer: SOMEHOST
  Description:
N/A

Event[1]
  Log Name: System
  Source: nvlddmkm
  Date: 2026-10-21T18:59:55.8320000Z
  Event ID: 153
  Task: None
  Level: Error
  Computer: SOMEHOST
  Description:
N/A

Event[2]
  Log Name: System
  Source: Display
  Date: 2026-10-11T20:13:41.7600000Z
  Event ID: 4101
  Level: Warning
  Computer: SOMEHOST
  Description:
Display driver nvlddmkm stopped responding and has successfully recovered.
";

#[test]
fn the_event_log_text_reads_as_events_without_the_computer_name() {
    let v = parse_wevtutil_text(SAMPLE);
    assert_eq!(v.len(), 3);
    assert_eq!(
        v[0],
        DriverEvent {
            time: "2026-10-26T14:31:12.7180000Z".into(),
            source: "nvlddmkm".into(),
            id: 153,
            level: "Error".into(),
            what: String::new(),
        }
    );
    assert_eq!(v[2].id, 4101);
    assert_eq!(v[2].what, "Display driver nvlddmkm stopped responding and has successfully recovered.");
    let block = events_block(&Ok(v.clone()), 30);
    assert!(!block.contains("SOMEHOST"), "{block}");
    assert!(block.contains("3 event(s)"), "{block}");
    assert!(block.contains("the NVIDIA driver reset the GPU (a TDR)"), "{block}");
    assert!(block.contains("stopped responding and was reset"), "{block}");
    assert_eq!(parse_wevtutil_text(""), vec![]);
    // wevtutil ends some fields with a NUL (the dev box, 2026-11-01): none survives.
    let nul = parse_wevtutil_text("Event[0]\n  Source: nvlddmkm\n  Event ID: 153\n  Level: Error\u{0}\n  Description: \nx\u{0}\n");
    assert_eq!((nul[0].level.as_str(), nul[0].what.as_str()), ("Error", "x"));
    assert!(!events_block(&Ok(nul), 30).contains('\u{0}'));
    assert!(events_block(&Ok(vec![]), 30).contains("none"));
    assert!(events_block(&Err("no".into()), 30).contains("not read (no)"));
}

#[test]
fn a_gpu_reset_report_is_told_from_an_ordinary_crash() {
    let e = |source: &str, id: u32, what: &str| DriverEvent {
        time: String::new(),
        source: source.into(),
        id,
        level: String::new(),
        what: what.into(),
    };
    assert_eq!(e("Windows Error Reporting", 1001, "Fault bucket LKD_0x141_Tdr:6, type 0 Event Name: LiveKernelEvent").meaning(), "a GPU reset report (LiveKernelEvent)");
    assert_eq!(e("Application Error", 1000, "Faulting application name: fractadyne.exe").meaning(), "an application crash or hang");
    assert_eq!(e("Microsoft-Windows-Kernel-Power", 41, "").meaning(), "Windows restarted without shutting down cleanly");
    assert_eq!(e("amdkmdag", 7, "").meaning(), "");
}

#[test]
fn the_queries_cover_the_field_agents_sources_and_the_window() {
    let q = event_queries(30);
    let all: String = q.iter().map(|(log, x)| format!("{log} {x}\n")).collect();
    for want in ["EventID=4101", "'Display'", "'nvlddmkm'", "'amdkmdag'", "'amdwddmg'", "EventID=41", "'Windows Error Reporting'", "'Application Hang'"] {
        assert!(all.contains(want), "{want} missing from\n{all}");
    }
    assert!(all.contains("timediff(@SystemTime) <= 2592000000"), "{all}");
}

#[test]
fn a_name_is_replaced_whatever_its_case() {
    assert_eq!(replace_ci("C:\\Users\\Ann\\x on SomeHost and SOMEHOST", "somehost", "<computer>"), "C:\\Users\\Ann\\x on <computer> and <computer>");
    assert_eq!(replace_ci("abc", "", "x"), "abc");
    assert_eq!(replace_ci("no match", "zzz", "x"), "no match");
}

#[test]
fn windows_is_named_by_its_build_not_its_product_name() {
    assert_eq!(windows_name("26100", Some(2894), "24H2", "Professional"), format!("Windows 11 Pro 24H2 (build 26100.2894), {}", std::env::consts::ARCH));
    assert_eq!(windows_name("19045", None, "22H2", "Core"), format!("Windows 10 Home 22H2 (build 19045), {}", std::env::consts::ARCH));
    assert_eq!(windows_name("", None, "", ""), format!("Windows, {}", std::env::consts::ARCH));
}

#[test]
fn the_power_plan_is_the_name_in_parentheses() {
    assert_eq!(parse_powercfg("Power Scheme GUID: 381b4222-f694-41f0-9685-ff5bb260df2e  (Balanced)\r\n"), Some("Balanced".into()));
    assert_eq!(parse_powercfg("Power Scheme GUID: 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c  (High performance)"), Some("High performance".into()));
    assert_eq!(parse_powercfg("nothing here"), None);
    assert_eq!(parse_powercfg("Power Scheme GUID: x ()"), None);
}

#[test]
fn the_gpu_timeout_settings_say_what_is_default_and_what_was_changed() {
    let d = tdr_summary(None, None, None, None);
    assert_eq!(
        d,
        "TdrDelay 2 s (default), TdrDdiDelay 5 s (default), TdrLevel 3 (default) = recover; hardware-accelerated GPU scheduling not set"
    );
    let s = tdr_summary(Some(8), Some(5), Some(0), Some(2));
    assert_eq!(
        s,
        "TdrDelay 8 s (changed from 2 s), TdrDdiDelay 5 s, TdrLevel 0 (changed from 3) = detection OFF; hardware-accelerated GPU scheduling on"
    );
}
