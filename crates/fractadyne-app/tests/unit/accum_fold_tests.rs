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

/// The time cap ends a costly run (the 2^704 view at 5120×2054: 4–6 s a sample, 24 of them ~2
/// minutes), but only on a run that already holds `ACCUM_MIN_SAMPLES`; 0 turns it off.
#[test]
fn the_time_cap_ends_a_costly_run_once_it_holds_enough_samples() {
    let min = crate::tunables::ACCUM_MIN_SAMPLES;
    assert!(!accum_time_capped(min + 5, 19_999, 20_000), "inside the cap");
    assert!(accum_time_capped(min, 20_000, 20_000));
    assert!(!accum_time_capped(min - 1, 60_000, 20_000), "too few samples to stop on");
    assert!(!accum_time_capped(23, 600_000, 0), "off");
}

/// The cap ends a run in the frame the worker's sample was counted (its fold asks for the next index
/// right after): the sample attached to that frame's fold is one of the run's count and must stay,
/// while the worker stops and what it finished but nobody counted goes. Dropped, the run converged
/// "at 8 samples, but the average holds 7" (3 runs of 7 at the 2^704 view, 2026-10-21).
#[test]
fn the_time_cap_keeps_the_worker_sample_it_already_counted() {
    use fractadyne_gpu::AccumSample;
    use std::sync::Arc;
    let sample = |w| Arc::new(AccumSample { width: w, height: 1, rgba: vec![0.0; 4 * w as usize] });
    let mut p = Perf::default();
    p.accum_count[0] = 8;
    p.accum_run[0] = 5;
    p.accum_ext_attach[0] = Some(sample(1));
    p.accum_ext_ready[0] = vec![sample(2)];
    p.accum_free[0] = vec![3];
    p.worker_job = Some((0, 5, 9));
    assert_eq!(p.accum_cap(0, 8), Some((0, 5)), "the worker's job is cancelled");
    assert_eq!(p.accum_goal(0), 8);
    assert_eq!(p.accum_ext_attach[0].as_ref().map(|s| s.width), Some(1), "the counted sample still folds");
    assert!(p.accum_ext_ready[0].is_empty() && p.accum_free[0].is_empty());
    assert_eq!(p.accum_run[0], 6, "a sample of the old run that comes back is not this one's");

    // Any other stop (a new run, a view change) drops the attached sample with the rest.
    p.accum_ext_attach[0] = Some(sample(1));
    assert_eq!(p.accum_worker_stop(0), Some((0, 5)));
    assert!(p.accum_ext_attach[0].is_none());
    // Another view's job is not this view's to cancel.
    p.worker_job = Some((1, 2, 0));
    assert_eq!(p.accum_cap(0, 4), None);
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
