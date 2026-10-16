//! A retreat says what it reached (`retreat_outcome`, design §5.6, W3 gate 2).

use super::*;

#[test]
fn a_retreat_below_the_lethal_dispatch_is_applied() {
    assert_eq!(retreat_outcome(1_024, 256), RetreatOutcome::Applied { from: 1_024, to: 256 });
    assert_eq!(retreat_outcome(1_024, 256).code(), 1);
}

#[test]
fn a_retreat_at_the_floor_reached_nothing() {
    // The lethal dispatch was already floor-sized: shedding to the floor is the same dispatch.
    assert_eq!(retreat_outcome(256, 256), RetreatOutcome::NoActuatorReachable { at: 256 });
    // And a floor ABOVE the lethal one (a pass the prefix made small) is no retreat either.
    assert_eq!(retreat_outcome(100, 256), RetreatOutcome::NoActuatorReachable { at: 100 });
    assert_eq!(retreat_outcome(256, 256).code(), 2);
}
