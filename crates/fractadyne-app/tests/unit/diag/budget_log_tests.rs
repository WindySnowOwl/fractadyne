use super::*;

/// Held for the WHOLE of each test that touches the global ring. ⚠The ring's own mutex does
/// not serialise them — it is released between calls — and two of these tests raced: one
/// clearing the ring and asserting it empty while the other filled it (seen as "decision 10…25"
/// inside an "empty" report once the test mix changed the scheduling).
static SERIAL: Mutex<()> = Mutex::new(());

/// The ring must keep the MOST RECENT decisions and stay bounded: a crash report that
/// carried the first 24 decisions of a fourteen-minute session would describe the healthy
/// start and say nothing about the seconds that killed it.
#[test]
fn the_ring_keeps_the_last_decisions_and_is_bounded() {
    let _g = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    if let Ok(mut v) = BUDGET_LOG.lock() {
        v.clear();
    }
    for i in 0..(BUDGET_LOG_CAP + 10) {
        budget_note(format!("decision {i}"));
    }
    let h = budget_history().expect("recorded decisions");
    assert_eq!(h.lines().count(), BUDGET_LOG_CAP);
    assert!(h.contains(&format!("decision {}", BUDGET_LOG_CAP + 9)), "keeps the newest");
    assert!(!h.contains("decision 0 "), "drops the oldest:\n{h}");
    if let Ok(mut v) = BUDGET_LOG.lock() {
        v.clear();
    }
}

/// The report must actually carry the budget section, and it must say WHICH of the two
/// situations held. A device loss where the controller was handed a slow reading and chose to
/// keep the budget is a different bug from one where it was handed nothing at all, and the
/// 2026-09-21 RX 6800 XT loss could not be told apart without this.
#[test]
fn the_crash_report_carries_the_budget_decisions() {
    let _g = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    if let Ok(mut v) = BUDGET_LOG.lock() {
        v.clear();
    }
    let empty = compose_crash_report("wgpu device lost (Unknown)", "<device-lost handler>", "");
    assert!(empty.contains("budget  : no frame-budget decision was recorded"), "{empty}");

    budget_note("v0 gpu_iterate=12.0ms steps=4.017e10 budget=1.515e11 DISCARDED".into());
    let filled = compose_crash_report("wgpu device lost (Unknown)", "<device-lost handler>", "");
    assert!(filled.contains("budget  : the last frame-budget decisions"), "{filled}");
    assert!(filled.contains("DISCARDED"), "the decision itself must survive:
{filled}");
    assert!(filled.contains("budget=1.515e11"), "and what it was judged against");
    if let Ok(mut v) = BUDGET_LOG.lock() {
        v.clear();
    }
}

/// The report must carry the `frames:` section it is handed, and name the session — the id is
/// what joins a report to its `frames.bin` and its companion `-frames.jsonl`.
#[test]
fn the_crash_report_carries_the_frames_section_and_the_session() {
    let section = frame_record::crash_section(&[], None);
    let r = compose_crash_report("wgpu device lost (Unknown)", "<device-lost handler>", &section);
    assert!(r.contains("frames  : no frame was recorded before the crash"), "{r}");
    assert!(
        r.contains(&format!("session : {:016x}", frame_record::session_id())),
        "the session id joins the report to its frame files:\n{r}"
    );
    // The section sits after the budget block and before the backtrace.
    let (b, f, t) = (r.find("budget  :").unwrap(), r.find("frames  :").unwrap(), r.find("backtrace").unwrap());
    assert!(b < f && f < t, "{r}");
}

/// Nothing recorded must read as "nothing recorded", not as an empty section a reader could
/// mistake for "the controller made no decisions because it was never asked".
#[test]
fn an_empty_ring_reports_nothing_rather_than_an_empty_block() {
    let _g = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    if let Ok(mut v) = BUDGET_LOG.lock() {
        v.clear();
    }
    assert!(budget_history().is_none());
}
