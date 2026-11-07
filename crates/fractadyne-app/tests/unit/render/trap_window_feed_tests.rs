use super::{trap_window_feed, TRAP_WINDOW_BREAK, TRAP_WINDOW_HOLD};

/// The first reading is taken as it is, held or not.
#[test]
fn the_first_reading_is_the_window() {
    assert_eq!(trap_window_feed(None, (0.83, 4.88), false), (0.83, 4.88));
    assert_eq!(trap_window_feed(None, (0.83, 4.88), true), (0.83, 4.88));
}

/// ⭐A view at rest: supersampling moves the samples, the quantiles wobble by a bucket (1/16) or
/// two, and the window must not follow, or every frame would be a new palette map and the average
/// would restart for ever.
#[test]
fn a_wobble_of_a_bucket_or_two_keeps_the_window() {
    let held = (0.83, 4.88);
    for new in [(0.83 + 1.0 / 16.0, 4.88), (0.83, 4.88 - 2.0 / 16.0), (0.80, 4.95)] {
        assert_eq!(trap_window_feed(Some(held), new, false), held, "{new:?}");
        assert_eq!(trap_window_feed(Some(held), new, true), held, "{new:?}");
    }
}

/// ⭐⭐The reading that never let the spinner stop (2026-11-07): a Multibrot 8 view at 2^79 read
/// its window's top as 14.58 and 14.89 by turns, 6.5% of the window, past the 5% hold. At rest on
/// that view the window holds through the flip, both ways.
#[test]
fn at_rest_a_tail_quantile_flipping_between_two_buckets_keeps_the_window() {
    let a = (10.000356, 14.58);
    let b = (10.000357, 14.8925);
    // Not held (moving), the flip is a move of the view as far as the 5% rule can tell.
    assert_eq!(trap_window_feed(Some(a), b, false), b);
    // Held, it is the same picture.
    assert_eq!(trap_window_feed(Some(a), b, true), a);
    assert_eq!(trap_window_feed(Some(b), a, true), b);
}

/// Held, a reading that moves an end by more than a quarter of the window is another picture (a
/// window taken before the reference arrived), and replaces it.
#[test]
fn at_rest_a_different_picture_still_replaces_the_window() {
    let held = (10.0, 14.6);
    let past = 1.2 * TRAP_WINDOW_BREAK * (held.1 - held.0);
    assert_eq!(trap_window_feed(Some(held), (held.0, held.1 + past), true), (held.0, held.1 + past));
    assert_eq!(trap_window_feed(Some(held), (2.0, 9.0), true), (2.0, 9.0));
}

/// A real change of view moves an end by more than the hold, and the window follows it.
#[test]
fn a_change_of_view_moves_the_window() {
    let held = (0.83, 4.88);
    let span = held.1 - held.0;
    let past = 1.5 * TRAP_WINDOW_HOLD * span;
    assert_eq!(trap_window_feed(Some(held), (held.0 + past, held.1), false), (held.0 + past, held.1));
    assert_eq!(trap_window_feed(Some(held), (-7.9, 1.75), false), (-7.9, 1.75));
}

/// A window narrower than a bucket still lets a reading through once it moves a bucket's share.
#[test]
fn a_narrow_window_is_held_against_a_bucket_not_against_nothing() {
    let held = (1.0, 1.01);
    assert_eq!(trap_window_feed(Some(held), (1.001, 1.01), false), held);
    assert_eq!(trap_window_feed(Some(held), (1.5, 2.0), false), (1.5, 2.0));
}
