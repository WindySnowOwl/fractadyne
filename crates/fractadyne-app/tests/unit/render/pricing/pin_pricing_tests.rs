//! Every pin pass is priced, timestamps or not (`pin_pass_priceable`, `PIN_INFLIGHT_MAX`).

use super::*;
use crate::tunables::{PIN_INFLIGHT_MAX, PIN_PRICE_WAIT_FRAMES};

#[test]
fn a_timestamp_prices_the_pass_at_once() {
    assert!(pin_pass_priceable(3.2, 0, 1));
}

#[test]
fn without_a_reading_a_completed_pass_is_priced_at_once() {
    // The no-timestamps motiontest: a new pass every frame restarted the wait, so none was priced.
    assert!(pin_pass_priceable(-1.0, 1, 0));
    assert!(!pin_pass_priceable(-1.0, 1, 1), "not before its completion is seen");
}

#[test]
fn a_completion_that_is_not_coming_releases_after_the_wait() {
    assert!(pin_pass_priceable(-1.0, PIN_PRICE_WAIT_FRAMES, 1));
}

#[test]
fn two_pin_passes_are_in_flight_with_timestamps_and_one_without() {
    const { assert!(PIN_INFLIGHT_MAX == 2) };
    assert_eq!(crate::tunables::pin_inflight_max(true), 2);
    assert_eq!(crate::tunables::pin_inflight_max(false), 1);
}
