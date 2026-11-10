use super::*;

const S: fn(u64) -> Duration = Duration::from_secs;
const MS: fn(u64) -> Duration = Duration::from_millis;

#[test]
fn it_waits_for_a_measured_frame_first() {
    assert!(!may_start(None, true, Some(S(60))));
    assert!(!may_start(None, true, None));
}

#[test]
fn a_still_view_left_alone_starts_it() {
    assert!(may_start(Some(MS(0)), true, Some(STILL_INPUT)));
    assert!(may_start(Some(S(3)), true, Some(S(30))));
    // Where input cannot be read, a still view is enough.
    assert!(may_start(Some(S(3)), true, None));
}

#[test]
fn a_moving_view_or_recent_input_holds_it_back_until_the_wait_runs_out() {
    assert!(!may_start(Some(S(3)), false, Some(S(30))));
    assert!(!may_start(Some(S(3)), true, Some(MS(500))));
    assert!(may_start(Some(WAIT_MAX), false, Some(MS(0))));
    assert!(may_start(Some(WAIT_MAX + S(1)), false, None));
}

#[test]
fn the_clock_counts_a_span_under_way_and_after_it_ends() {
    let c = BusyClock::default();
    let t0 = Instant::now();
    assert_eq!(c.total_ms(t0), 0.0);
    let held = c.hold();
    let later = Instant::now() + MS(250);
    let under_way = c.total_ms(later);
    assert!((249.0..260.0).contains(&under_way), "{under_way}");
    drop(held);
    // Ended now: what it held is kept, and nothing more accrues.
    let done = c.total_ms(later + S(10));
    assert!((0.0..50.0).contains(&done), "{done}");
    assert_eq!(c.total_ms(later + S(20)), done);
}

#[test]
fn each_interval_takes_only_the_time_held_since_the_last() {
    let clock = Arc::new(BusyClock::default());
    let mut slot = Some((clock.clone(), 0.0));
    let held = clock.hold();
    let t = Instant::now();
    let a = overlap_ms(&mut slot, t + MS(100));
    assert!((99.0..110.0).contains(&a), "{a}");
    let b = overlap_ms(&mut slot, t + MS(300));
    assert!((199.0..201.0).contains(&b), "{b}");
    // A later read at an earlier instant takes nothing back.
    assert_eq!(overlap_ms(&mut slot, t + MS(200)), 0.0);
    drop(held);
    assert!(slot.is_some(), "the thread has not finished");
    clock.finish();
    let _ = overlap_ms(&mut slot, Instant::now());
    assert!(slot.is_none(), "a finished clock leaves the slot once read");
    assert_eq!(overlap_ms(&mut slot, Instant::now() + S(5)), 0.0);
}
