//! The fine escape histogram the live palette window takes its top from (`CTR_NORM_HIST`).

use crate::{
    norm_hist_bucket, NormHist, COUNTER_SLOTS, CTR_NORM_HIST, CTR_RENORM, NORM_HIST_BUCKETS,
    NORM_HIST_PER_OCTAVE,
};

/// A histogram of `values` measured from `base`, bucketed exactly as the shader does.
fn hist_of(values: &[f32], base: f32) -> NormHist {
    let mut counts = vec![0u32; NORM_HIST_BUCKETS];
    for &v in values {
        counts[norm_hist_bucket(v, base)] += 1;
    }
    NormHist { base, counts }
}

#[test]
fn the_histogram_is_the_last_thing_in_the_counter_buffer() {
    assert_eq!(CTR_RENORM + 1, CTR_NORM_HIST);
    assert_eq!(CTR_NORM_HIST + NORM_HIST_BUCKETS, COUNTER_SLOTS);
    let mut slots = [0u32; COUNTER_SLOTS];
    slots[CTR_NORM_HIST] = 3;
    slots[COUNTER_SLOTS - 1] = 4;
    let h = NormHist::from_slots(&slots, 12.5);
    assert_eq!((h.base, h.counts.len(), h.counts[0], h.counts[NORM_HIST_BUCKETS - 1]), (12.5, NORM_HIST_BUCKETS, 3, 4));
    assert_eq!(h.total(), 7);
}

/// The shader's own copy of the slot, the bucket count and the bucket formula ("keep in sync"): a
/// drift would add into another counter's slot, or past the buffer, silently.
#[test]
fn the_shader_agrees_on_the_fine_histogram() {
    let wgsl = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/mandelbrot.wgsl"));
    let slot = format!("const CTR_NORM_HIST: u32 = {CTR_NORM_HIST}u;");
    assert!(wgsl.contains(&slot), "mandelbrot.wgsl must declare `{slot}`");
    let formula = format!(
        "clamp(floor(log2(x) * {}.0), 0.0, {}.0)",
        NORM_HIST_PER_OCTAVE,
        NORM_HIST_BUCKETS - 1
    );
    assert!(wgsl.contains(&formula), "norm_hist_commit must bucket as `{formula}`");
    assert!(wgsl.contains("let x = max(sm - iu.norm_base, 0.0) + 1.0;"), "measured from the base");
}

#[test]
fn buckets_are_fine_near_the_base_and_clamp_at_both_ends() {
    // At or below the base: bucket 0, whatever the value.
    assert_eq!(norm_hist_bucket(100.0, 100.0), 0);
    assert_eq!(norm_hist_bucket(3.0, 100.0), 0);
    // One doubling of (v - base + 1) is NORM_HIST_PER_OCTAVE buckets.
    assert_eq!(norm_hist_bucket(101.0, 100.0), NORM_HIST_PER_OCTAVE);
    assert_eq!(norm_hist_bucket(103.0, 100.0), 2 * NORM_HIST_PER_OCTAVE);
    // Past 2^26 above the base: the open-ended last bucket.
    assert_eq!(norm_hist_bucket(1.0e9, 0.0), NORM_HIST_BUCKETS - 1);
    // A narrow deep band (the [181573, 182297] field case) spans over a hundred buckets from its base.
    let span = norm_hist_bucket(182_297.0, 181_573.0) - norm_hist_bucket(181_573.0, 181_573.0);
    assert!(span > 100, "{span} buckets");
}

#[test]
fn a_quantile_ignores_a_few_late_escapers() {
    // The 2026-10-25 shape: the picture escapes between 3,900 and 6,400, and 1% of it - pixels
    // beside a minibrot - escapes up to the ask at 80,000.
    let mut v: Vec<f32> = (0..9900).map(|i| 3900.0 + 2500.0 * i as f32 / 9899.0).collect();
    v.extend((0..100).map(|i| 6400.0 + 73_600.0 * i as f32 / 99.0));
    let h = hist_of(&v, 3900.0);
    let p98 = h.quantile(0.98).unwrap();
    let p99 = h.quantile(0.99).unwrap();
    assert!((6000.0..6500.0).contains(&p98), "p98 {p98}");
    assert!((6150.0..6700.0).contains(&p99), "p99 {p99}");
    let top = h.quantile(1.0).unwrap();
    assert!(top > 70_000.0, "the whole range is still there at q = 1: {top}");
}

#[test]
fn quantiles_move_continuously_as_the_picture_drifts() {
    // A distribution sliding upward a little at a time moves its quantile a little at a time, not
    // in bucket-sized steps: the interpolation inside a bucket is what keeps the window from
    // stepping.
    let mut last: Option<f64> = None;
    for k in 0..200 {
        let shift = k as f32 * 2.0;
        let v: Vec<f32> = (0..5000).map(|i| 2000.0 + shift + 1500.0 * (i as f32 / 4999.0).powi(2)).collect();
        let q = hist_of(&v, 2000.0).quantile(0.98).unwrap();
        if let Some(p) = last {
            assert!((q - p).abs() < 40.0, "step {k}: {p} -> {q}");
        }
        last = Some(q);
    }
}

#[test]
fn an_empty_histogram_or_a_q_out_of_range_has_no_quantile() {
    let h = NormHist { base: 0.0, counts: vec![0; NORM_HIST_BUCKETS] };
    assert_eq!(h.quantile(0.5), None);
    let h = hist_of(&[10.0, 20.0], 0.0);
    assert_eq!(h.quantile(0.0), None);
    assert_eq!(h.quantile(1.5), None);
    assert!(h.quantile(1.0).is_some());
}
