//! Pin passes are priced without GPU timestamps too (`pin_pass_priceable`, `pin_inflight_max`).

use super::*;
use crate::tunables::{PIN_INFLIGHT_MAX, PIN_PRICE_WAIT_FRAMES};

#[test]
fn a_timestamp_prices_the_pass_at_once() {
    assert!(pin_pass_priceable(3.2, 0, false, 2));
}

#[test]
fn with_timestamps_a_pass_waits_for_its_reading_then_the_frame_limit() {
    assert!(!pin_pass_priceable(-1.0, 1, false, 0));
    assert!(pin_pass_priceable(-1.0, PIN_PRICE_WAIT_FRAMES, false, 2));
}

#[test]
fn without_timestamps_a_completed_pass_is_priced_at_once() {
    // The no-timestamps motiontest: a new pass every frame restarted the wait, so none was priced.
    assert!(pin_pass_priceable(-1.0, 1, true, 0));
    assert!(!pin_pass_priceable(-1.0, 1, true, 1), "not before its completion is seen");
}

#[test]
fn without_timestamps_one_pin_pass_is_in_flight() {
    assert_eq!(pin_inflight_max(true), 1);
    assert_eq!(pin_inflight_max(false), PIN_INFLIGHT_MAX);
}
