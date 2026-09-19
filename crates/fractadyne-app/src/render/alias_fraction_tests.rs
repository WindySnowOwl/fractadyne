//! The live-normalization guard's statistic: the share of pixel pairs whose palette step passes
//! Nyquist, read from a log₂ step histogram against the cycle in force.

use crate::render::alias_fraction;
use fractadyne_gpu::GRAD_HIST_BUCKETS as N;

/// A histogram with everything in one bucket.
fn one(b: usize) -> [f32; N] {
    let mut h = [0.0; N];
    h[b] = 1.0;
    h
}

#[test]
fn steps_well_past_nyquist_all_alias_and_steps_well_below_do_not() {
    // cycle 0.0352 (slider 0.52) puts the Nyquist step at 0.5 / 0.0352 ≈ 14.2 iterations.
    let c = 0.0352;
    // Bucket 7 = [64, 128): every pair steps far past 14.2.
    assert!((alias_fraction(&one(7), c, 0.5) - 1.0).abs() < 1e-6);
    // Bucket 2 = [2, 4): every pair steps far below it.
    assert_eq!(alias_fraction(&one(2), c, 0.5), 0.0);
    // Bucket 0 = [0, 1): the smooth exterior of a shallow panel.
    assert_eq!(alias_fraction(&one(0), c, 0.5), 0.0);
}

#[test]
fn a_smooth_half_does_not_dilute_an_aliasing_half() {
    // ⭐⭐The field case this replaces the mean for. Half the pairs are smooth exterior (steps < 1),
    // half are dense body stepping ~64–128 iterations. A MEAN over that is dragged down by the smooth
    // half; the FRACTION reports exactly what share of the picture is noise.
    let mut h = [0.0; N];
    h[0] = 0.5;
    h[7] = 0.5;
    let f = alias_fraction(&h, 0.0352, 0.5);
    assert!((f - 0.5).abs() < 1e-6, "half the picture aliases, got {f}");
}

#[test]
fn the_threshold_bucket_is_split_log_uniformly() {
    // Nyquist step exactly 2^4 = 16 sits on a bucket edge: bucket 5 = [16, 32) is wholly above.
    let c = 0.5 / 16.0;
    assert!((alias_fraction(&one(5), c, 0.5) - 1.0).abs() < 1e-6);
    assert!(alias_fraction(&one(4), c, 0.5).abs() < 1e-6, "[8,16) is wholly below 16");
    // A threshold at the geometric middle of bucket 5 (16·√2 ≈ 22.6) leaves half of it above.
    let c_mid = 0.5 / (16.0 * std::f32::consts::SQRT_2);
    let f = alias_fraction(&one(5), c_mid, 0.5);
    assert!((f - 0.5).abs() < 1e-4, "log-uniform split, got {f}");
}

#[test]
fn a_faster_palette_aliases_more_of_the_same_picture() {
    // Monotone in the cycle: the same histogram can only alias MORE as the palette speeds up. This
    // is what lets the app re-decide the moment the cycle slider moves, from the histogram it holds.
    let mut h = [0.0; N];
    for (b, w) in [(1, 0.1), (3, 0.2), (5, 0.3), (7, 0.25), (9, 0.15)] {
        h[b] = w;
    }
    let mut prev = -1.0;
    for slider in [0.0_f32, 0.1, 0.27, 0.52, 0.8, 1.0] {
        let f = alias_fraction(&h, 0.004 + slider * 0.06, 0.5);
        assert!(f >= prev - 1e-6, "slider {slider}: {f} < {prev}");
        prev = f;
    }
}

#[test]
fn degenerate_inputs_are_never_counted_as_aliasing() {
    let h = one(9);
    assert_eq!(alias_fraction(&h, 0.0, 0.5), 0.0, "a stopped palette cannot alias");
    assert_eq!(alias_fraction(&h, -1.0, 0.5), 0.0);
    assert_eq!(alias_fraction(&h, f32::NAN, 0.5), 0.0);
    assert_eq!(alias_fraction(&[0.0; N], 0.0352, 0.5), 0.0, "an empty histogram");
}
