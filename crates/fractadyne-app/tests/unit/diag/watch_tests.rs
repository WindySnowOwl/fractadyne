//! The watchdog's verdict (`watch_verdict`): a frame that stops, or a due frame that never comes,
//! is a possible hang; a window waiting for input is not, however long it waits.

use super::*;

#[test]
fn a_stalled_frame_is_a_possible_hang_after_ten_seconds() {
    assert_eq!(watch_verdict(9, Loop::InFrame), None);
    assert_eq!(watch_verdict(10, Loop::InFrame), Some(""));
    assert_eq!(watch_verdict(3_600, Loop::InFrame), Some(""));
}

#[test]
fn a_due_frame_that_does_not_come_is_one_too_and_says_where_it_stopped() {
    assert_eq!(watch_verdict(9, Loop::Due), None);
    let between = watch_verdict(10, Loop::Due).expect("a due frame ten seconds late is reported");
    assert!(between.contains("between frames"), "{between}");
}

#[test]
fn a_window_waiting_for_input_is_never_one() {
    // The maintainer's logs: idle gaps up to 1,491 s, each reported every 30 s as a possible hang.
    for s in [10, 477, 1_491, 86_400] {
        assert_eq!(watch_verdict(s, Loop::Idle), None, "{s} s idle");
    }
}

#[test]
fn a_run_with_no_frame_loop_is_judged_as_one_stuck_in_a_frame() {
    // A command-line render never begins a frame: the state it starts in must be the strict one.
    assert_eq!(Loop::InFrame as u8, 0);
    assert_eq!(watch_verdict(10, Loop::InFrame), Some(""));
}
