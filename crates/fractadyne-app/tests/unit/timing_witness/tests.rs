use super::*;

#[test]
fn a_window_runs_from_the_arming_to_the_frames_completion() {
    let w = Witness::default();
    w.stamp_done(10, 1_025_000);
    let v = w.judge(10, 1_000_000).unwrap();
    assert_eq!(v.window_ms, 25.0);
    assert!(!v.impossible(25.0) && !v.impossible(25.4));
    assert!(v.impossible(26.0), "a reading longer than the window it ran inside is impossible");
}

#[test]
fn no_verdict_without_a_completion_an_arming_time_or_for_a_frame_that_left_the_ring() {
    let w = Witness::default();
    assert_eq!(w.judge(10, 1_000_000), None, "completion not seen yet");
    w.stamp_done(10, 1_025_000);
    assert_eq!(w.judge(10, 0), None, "a reading published without its arming time");
    assert!(w.judge(10, 1_000_000).is_some());
    // The same slot reused by a later frame: the old frame's verdict is gone, not misattributed.
    w.stamp_done(10 + RING as u64, 5_000_000);
    assert_eq!(w.judge(10, 1_000_000), None);
}

#[test]
fn the_queue_is_empty_only_when_the_previous_frame_finished_before_the_arming() {
    let w = Witness::default();
    w.stamp_done(9, 990_000);
    w.stamp_done(10, 1_020_000);
    assert!(w.judge(10, 1_000_000).unwrap().queue_empty);
    // Previous frame completed AFTER the arming: not empty.
    w.stamp_done(9, 1_010_000);
    assert!(!w.judge(10, 1_000_000).unwrap().queue_empty);
    // Previous frame's completion never seen: unknown is not empty.
    let w = Witness::default();
    w.stamp_done(10, 1_020_000);
    assert!(!w.judge(10, 1_000_000).unwrap().queue_empty);
}

#[test]
fn the_tally_reports_every_thirty_seconds_and_counts_the_impossible() {
    let mut w = Witness::default();
    let v = Verdict { window_ms: 20.0, queue_empty: true };
    assert_eq!(w.tally(5.0, &v, 0), None);
    assert_eq!(w.tally(50.0, &v, 10_000_000), None);
    let line = w.tally(10.0, &v, 30_000_000).expect("a summary after 30 s");
    assert!(line.contains("3 GPU reading(s)"), "{line}");
    assert!(line.contains("IMPOSSIBLE (longer than the window) 1, worst 30.0 ms over"), "{line}");
    assert!(line.contains("3 with the queue empty"), "{line}");
    assert_eq!(w.tally(5.0, &v, 30_000_001), None, "reset after reporting");
}
