use super::*;

/// The rule `accum_confirm` applies after `build_params`: only a real, complete frame at the run's
/// supersampling factor may join the running average. Each rejected case is a picture the average
/// would have carried forever: the 2026-09-15 field report (dual view, 3.7e144×) was the first one.
#[test]
fn a_fold_needs_a_real_complete_frame_at_the_run_ss() {
    // Sample 0 sets the ss; later samples must match it.
    assert!(accum_fold_clean(0, false, false, false, false, false, 0, 2, 1));
    assert!(accum_fold_clean(0, false, false, false, false, false, 3, 2, 2));
    // The ghost: sample 0 was the held REPROJECTION of the previous view.
    assert!(!accum_fold_clean(1, false, false, false, false, false, 0, 2, 2));
    // A pinned/held display is not this view's frame either.
    assert!(!accum_fold_clean(0, true, false, false, false, false, 1, 2, 2));
    assert!(!accum_fold_clean(0, false, true, false, false, false, 1, 2, 2));
    // `build_params` started a settle grid / chunk progression THIS frame — the pre-build `busy`
    // flags said idle, the frame is not complete.
    assert!(!accum_fold_clean(0, false, false, true, false, false, 0, 2, 2));
    assert!(!accum_fold_clean(0, false, false, false, true, false, 0, 2, 2));
    // A reference build in flight: the next install re-iterates everything.
    assert!(!accum_fold_clean(0, false, false, false, false, true, 0, 2, 2));
    // A mid-ramp stage (ss=1) must not fold into an ss=2 run.
    assert!(!accum_fold_clean(0, false, false, false, false, false, 3, 1, 2));
}

/// Sample 0 is the pixel centre (so the hand-off from the ordinary settle is seamless); every
/// later sample stays inside the pixel.
#[test]
fn jitter_sequence_starts_at_the_pixel_centre_and_stays_in_the_pixel() {
    assert_eq!(accum_jitter_seq(0), [0.0, 0.0]);
    for i in 1..64 {
        let j = accum_jitter_seq(i);
        assert!((-0.5..0.5).contains(&j[0]) && (-0.5..0.5).contains(&j[1]), "sample {i}: {j:?}");
    }
}

/// Supersampling begins on a chunk-walked settle only once the adaptive limit has a verdict at the
/// budget it plans and the colour range has had its complete reading (2026-10-07: a run begun
/// during the climb restarted at every raise, for minutes).
#[test]
fn a_walked_settle_begins_only_once_the_limit_and_the_range_are_decided() {
    use AccumWait::*;
    let (sig, other) = (0x5151, 0x7272);
    // Decided: the verdict names the planned budget, and the complete reading is this view's.
    assert_eq!(accum_inputs_pending(true, true, false, Some(197_132), 197_132, sig, sig), None);
    // The climb has not read the new budget yet, or its last reading moved it.
    assert_eq!(accum_inputs_pending(true, true, false, None, 197_132, sig, sig), Some(IterationLimit));
    assert_eq!(
        accum_inputs_pending(true, true, false, Some(123_207), 197_132, sig, sig),
        Some(IterationLimit)
    );
    // The limit is decided but the range has not had its complete reading at this view and ask.
    assert_eq!(accum_inputs_pending(true, true, false, Some(197_132), 197_132, other, sig), Some(ColourRange));
    // No climb to wait for: an explicit count, or a latched interior plateau. The range still waits.
    assert_eq!(accum_inputs_pending(true, false, false, None, 1_000_000, sig, sig), None);
    assert_eq!(accum_inputs_pending(true, true, true, None, 197_132, sig, sig), None);
    assert_eq!(accum_inputs_pending(true, false, false, None, 1_000_000, other, sig), Some(ColourRange));
    // A one-pass settle cannot re-send for its reading: it begins as before.
    assert_eq!(accum_inputs_pending(false, true, false, None, 197_132, other, sig), None);
}
