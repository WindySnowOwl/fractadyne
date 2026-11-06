use super::{trap_window_feed, TRAP_WINDOW_HOLD};

/// The first reading is taken as it is.
#[test]
fn the_first_reading_is_the_window() {
    assert_eq!(trap_window_feed(None, (0.83, 4.88)), (0.83, 4.88));
}

/// ⭐A view at rest: supersampling moves the samples, the quantiles wobble by a bucket (1/16) or
/// two, and the window must not follow, or every frame would be a new palette map and the average
/// would restart for ever.
#[test]
fn a_wobble_of_a_bucket_or_two_keeps_the_window() {
    let held = (0.83, 4.88);
    for new in [(0.83 + 1.0 / 16.0, 4.88), (0.83, 4.88 - 2.0 / 16.0), (0.80, 4.95)] {
        assert_eq!(trap_window_feed(Some(held), new), held, "{new:?}");
    }
}

/// A real change of view moves an end by more than the hold, and the window follows it.
#[test]
fn a_change_of_view_moves_the_window() {
    let held = (0.83, 4.88);
    let span = held.1 - held.0;
    let past = 1.5 * TRAP_WINDOW_HOLD * span;
    assert_eq!(trap_window_feed(Some(held), (held.0 + past, held.1)), (held.0 + past, held.1));
    assert_eq!(trap_window_feed(Some(held), (-7.9, 1.75)), (-7.9, 1.75));
}

/// A window narrower than a bucket still lets a reading through once it moves a bucket's share.
#[test]
fn a_narrow_window_is_held_against_a_bucket_not_against_nothing() {
    let held = (1.0, 1.01);
    assert_eq!(trap_window_feed(Some(held), (1.001, 1.01)), held);
    assert_eq!(trap_window_feed(Some(held), (1.5, 2.0)), (1.5, 2.0));
}
