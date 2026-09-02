use super::norm_window_feed;

#[test]
fn mid_walk_readings_accumulate_and_do_not_touch_the_ema() {
    let (acc, ema) = norm_window_feed(None, (100.0, 200.0), true, Some((5.0, 10.0)));
    assert_eq!(acc, Some((100.0, 200.0)));
    assert_eq!(ema, Some((5.0, 10.0)), "the palette window must not follow one band");
    let (acc, ema) = norm_window_feed(acc, (900.0, 950.0), true, ema);
    assert_eq!(acc, Some((100.0, 950.0)), "bands widen, never replace");
    assert_eq!(ema, Some((5.0, 10.0)));
}

#[test]
fn completion_feeds_the_whole_ask_range_once_and_clears_the_accumulator() {
    let acc = Some((100.0f32, 900.0f32));
    let (acc, ema) = norm_window_feed(acc, (950.0, 1000.0), false, None);
    assert_eq!(acc, None);
    assert_eq!(ema, Some((100.0, 1000.0)), "first feed seeds the EMA with the full range");
}

#[test]
fn an_unchunked_frame_is_one_reading_one_feed_the_pre_walk_behaviour() {
    let (acc, ema) = norm_window_feed(None, (10.0, 20.0), false, Some((10.0, 20.0)));
    assert_eq!(acc, None);
    assert_eq!(ema, Some((10.0, 20.0)));
}

#[test]
fn the_ema_still_smooths_across_completed_walks() {
    let (_, ema) = norm_window_feed(None, (200.0, 300.0), false, Some((100.0, 200.0)));
    let (emn, emx) = ema.unwrap();
    assert!((emn - 130.0).abs() < 1e-3 && (emx - 230.0).abs() < 1e-3);
}
