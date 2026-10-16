//! The overlay's text is a CONTRACT with the log, not decoration: a reader lines a video frame up
//! against `[+12.345s]` by eye, so the format has to match digit for digit.

use super::overlay_text;

#[test]
fn the_clock_matches_the_log_stamp_digit_for_digit() {
    for t in [0.0, 1.5, 12.3456, 28.622, 99.9995, 3600.25] {
        let (clock, _) = overlay_text(t, 0);
        // `diag::stamp` is `format!("[+{:9.3}s]", elapsed_s())` — same sign, same 3 decimals.
        let from_log = format!("[+{t:9.3}s]");
        let logged = from_log.trim_start_matches("[+").trim_end_matches("s]").trim();
        assert_eq!(
            clock,
            format!("+{logged}s"),
            "overlay {clock} would not read the same as the log's {from_log}"
        );
    }
}

#[test]
fn the_frame_counter_is_there_to_break_ties_within_a_millisecond() {
    // Two frames inside the same stamp must be distinguishable, or the overlay cannot resolve the
    // thing it exists for (a pan that happens over a handful of frames).
    let (a_clock, a_frame) = overlay_text(12.345, 4200);
    let (b_clock, b_frame) = overlay_text(12.345, 4201);
    assert_eq!(a_clock, b_clock, "test premise: the same millisecond");
    assert_ne!(a_frame, b_frame);
    assert_eq!(a_frame, "frame 4200");
}
