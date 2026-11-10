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
