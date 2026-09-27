//! Content verification (design/verified-present.md): what counts as a picture, when a walk's
//! picture has stopped changing, and how the per-view `ContentTrack` answers the present gate.

use super::*;

fn reading(tag: u64, cursor: u32, escaped: u32) -> ContentReading {
    ContentReading { tag, cursor, escaped, px: 1_600_000, esc_min_bits: 0, esc_max_bits: 0, esc_hist: [0; 24] }
}

#[test]
fn a_frame_with_no_escaped_pixel_is_not_a_picture_and_a_filament_is() {
    assert!(!content_has_detail(0, 1_600_000));
    assert!(!content_has_detail(0, 1));
    assert!(!content_has_detail(5, 0), "no pixels at all");
    // 0.1% of the frame: a thin filament in an otherwise interior view.
    assert!(content_has_detail(1_600, 1_600_000));
    assert!(!content_has_detail(1_599, 1_600_000));
    // A tiny probe frame needs at least one pixel.
    assert!(content_has_detail(1, 64));
}

#[test]
fn a_walk_converges_when_a_much_longer_pass_adds_nothing() {
    let early = reading(7, 4_000, 900_000);
    // The next pass reaches 25% further and adds 0.5% — done.
    assert!(content_converged(&early, &reading(7, 5_000, 904_000)));
    // Not far enough along to say so…
    assert!(!content_converged(&early, &reading(7, 4_900, 904_000)));
    // …nor from a pass that added a real share of the picture…
    assert!(!content_converged(&early, &reading(7, 5_000, 930_000)));
    // …nor across different renders, nor backwards.
    assert!(!content_converged(&early, &reading(8, 5_000, 904_000)));
    assert!(!content_converged(&reading(7, 5_000, 904_000), &early));
}

#[test]
fn a_small_cursor_needs_a_whole_floor_pass_more_before_it_can_converge() {
    // At cursor 256, 25% further is 320 — one more tiny pass proves nothing; the rule asks for
    // at least a floor pass (256) more.
    let early = reading(3, 256, 0);
    assert!(!content_converged(&early, &reading(3, 320, 0)));
    assert!(content_converged(&early, &reading(3, 512, 0)));
}

#[test]
fn the_live_texture_is_verified_by_a_reading_of_the_render_it_holds_or_by_construction() {
    let mut t = ContentTrack::default();
    t.live_tag = 41;
    assert!(!t.live_verified(), "nothing known yet");
    t.feed(reading(40, 1_000, 500_000));
    assert!(!t.live_verified(), "a reading of another render says nothing about this one");
    t.feed(reading(41, 1_000, 0));
    assert!(!t.live_verified(), "the render it holds, but blank");
    t.feed(reading(41, 2_000, 500_000));
    assert!(t.live_verified());
    // A new dispatch of another render un-verifies it until its own reading lands…
    t.live_tag = 42;
    assert!(!t.live_verified());
    // …unless that render was complete by construction.
    t.live_complete = true;
    assert!(t.live_verified());
}

#[test]
fn the_pin_verdict_inputs_follow_the_latest_reading_of_that_pin() {
    let mut t = ContentTrack::default();
    assert_eq!(t.detail_for(9), None);
    t.feed(reading(9, 300, 0));
    assert_eq!(t.detail_for(9), Some(false));
    assert!(!t.reading_final_for(9, 150_000));
    assert!(!t.converged_for(9));
    t.feed(reading(9, 4_500, 800_000));
    assert_eq!(t.detail_for(9), Some(true));
    t.feed(reading(9, 6_000, 802_000));
    assert!(t.converged_for(9), "the pair 4,500 → 6,000 added 0.25%");
    t.feed(reading(9, 150_000, 802_000));
    assert!(t.reading_final_for(9, 150_000));
    // Another pin's reading resets the pair and the verdict.
    t.feed(reading(10, 256, 0));
    assert_eq!(t.detail_for(9), None);
    assert!(!t.converged_for(10));
}

#[test]
fn the_refresh_period_tracks_adoptions_and_sees_a_stall() {
    let mut t = ContentTrack::default();
    t.note_adopt(1_000_000);
    assert_eq!(t.refresh_period_s, 0.0, "one adoption is not a period");
    t.note_adopt(1_400_000);
    assert!((t.refresh_period_s - 0.4).abs() < 1e-9);
    t.blank_walks = 2;
    t.note_adopt(1_800_000);
    assert_eq!(t.blank_walks, 0, "an adoption ends a blank streak");
    assert!((t.refresh_period_s - 0.4).abs() < 1e-9);
    // 0.1 s after the last adoption the period is still the EMA…
    assert!((t.refresh_period_now(1_900_000) - 0.4).abs() < 1e-9);
    // …but two seconds without one IS the period now.
    assert!((t.refresh_period_now(3_800_000) - 2.0).abs() < 1e-9);
}
