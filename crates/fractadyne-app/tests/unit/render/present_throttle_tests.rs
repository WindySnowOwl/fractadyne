use super::{present_throttle_step, PRESENT_THROTTLE_FRAMES};

/// A run of no-work slow frames counts up: each one measured the present path alone, and
/// five of them (half a second at the 100 ms drain threshold) is the throttle verdict.
#[test]
fn no_work_slow_frames_accumulate_to_the_verdict() {
    let mut c = 0u32;
    for _ in 0..PRESENT_THROTTLE_FRAMES {
        c = present_throttle_step(c, false, 250.0, 100.0);
    }
    assert!(c >= PRESENT_THROTTLE_FRAMES);
}

/// A frame that DISPATCHED resets the count even when slow — a busy queue stalls frames
/// that did submit work, and those must never read as compositor throttling.
#[test]
fn a_dispatching_frame_resets_the_count() {
    let mut c = 0u32;
    for _ in 0..10 {
        c = present_throttle_step(c, false, 969.0, 100.0);
    }
    assert!(c >= PRESENT_THROTTLE_FRAMES);
    c = present_throttle_step(c, true, 969.0, 100.0);
    assert_eq!(c, 0, "a slow frame WITH a dispatch is queue evidence, not throttle evidence");
}

/// A quick present resets the count: the compositor is serving this window again.
#[test]
fn a_quick_present_resets_the_count() {
    let mut c = 7;
    c = present_throttle_step(c, false, 12.0, 100.0);
    assert_eq!(c, 0);
}

/// The 2026-09-04 field shape: ~1 s present-only intervals, held (deduped) chunk pass, no
/// dispatches — the verdict must arrive well inside the old 30-frame starvation window, so
/// the latch guard wins the race against the wall-fallback latch.
#[test]
fn the_field_shape_trips_the_verdict_before_the_starvation_latch() {
    let mut c = 0u32;
    for i in 0..30u32 {
        c = present_throttle_step(c, false, 969.0, 100.0);
        if i + 1 == PRESENT_THROTTLE_FRAMES {
            assert!(c >= PRESENT_THROTTLE_FRAMES, "verdict at frame {}", i + 1);
        }
    }
}
