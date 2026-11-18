//! A native modal dialog (`modal`): open only while it runs, its time handed to the frame loop
//! once, and closed again even when the call inside it panics. One test, in sequence: the counters
//! are process-wide, so two tests running at once would read each other's dialogs.

use super::*;

#[test]
fn a_dialog_is_open_while_it_runs_its_time_is_taken_once_and_a_panic_still_closes_it() {
    let _ = take_modal_ms();
    assert!(!modal_open());
    let got = modal(|| {
        assert!(modal_open(), "open while the dialog runs");
        std::thread::sleep(Duration::from_millis(40));
        7
    });
    assert_eq!(got, 7, "the dialog's answer comes back");
    assert!(!modal_open(), "closed once it returns");
    let ms = take_modal_ms();
    assert!((40.0..5_000.0).contains(&ms), "its time is handed over: {ms} ms");
    assert_eq!(take_modal_ms(), 0.0, "and only once");

    // Left open by a panic, the watchdog would never judge the UI thread again.
    let r = std::panic::catch_unwind(|| modal(|| -> () { panic!("the picker failed") }));
    assert!(r.is_err());
    assert!(!modal_open(), "closed by the unwind");
}
