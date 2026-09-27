//! The step histogram's readback, including the bucket the shader deliberately never writes.

use crate::{
    grad_hist_from_slots, COUNTER_SLOTS, CTR_ESC_HIST, CTR_GRAD_HIST, CTR_GRAD_N, ESC_HIST_BUCKETS,
    GRAD_HIST_BUCKETS,
};

#[test]
fn bucket_zero_is_the_samples_no_other_bucket_counted() {
    // ⭐The shader skips bucket 0's atomic (the hot address in any smooth region) and the readback
    // reconstructs it from CTR_GRAD_N, which counts the same samples in the same branch.
    let mut slots = [0u32; COUNTER_SLOTS];
    slots[CTR_GRAD_N] = 1000;
    slots[CTR_GRAD_HIST + 3] = 120; // [4, 8)
    slots[CTR_GRAD_HIST + 7] = 80; // [64, 128)
    let h = grad_hist_from_slots(&slots);
    assert_eq!(h[0], 800, "N minus everything counted above one iteration");
    assert_eq!(h[3], 120);
    assert_eq!(h[7], 80);
    assert_eq!(h.iter().sum::<u32>(), 1000, "the histogram accounts for every sample exactly once");
}

#[test]
fn a_frame_with_no_steps_over_one_iteration_is_all_bucket_zero() {
    let mut slots = [0u32; COUNTER_SLOTS];
    slots[CTR_GRAD_N] = 5000;
    let h = grad_hist_from_slots(&slots);
    assert_eq!(h[0], 5000);
    assert_eq!(h[1..].iter().sum::<u32>(), 0);
}

#[test]
fn an_inconsistent_readback_saturates_instead_of_wrapping() {
    // Should never happen — both counts come from one branch — but a torn or garbage readback must
    // not wrap u32 into a four-billion-sample bucket 0 that would swamp the fraction.
    let mut slots = [0u32; COUNTER_SLOTS];
    slots[CTR_GRAD_N] = 10;
    slots[CTR_GRAD_HIST + 5] = 50;
    assert_eq!(grad_hist_from_slots(&slots)[0], 0);
}

#[test]
fn the_histogram_fits_the_counter_buffer() {
    // The slot map: the step histogram, then the escape histogram as the tail, nothing overlapping.
    assert_eq!(CTR_GRAD_HIST + GRAD_HIST_BUCKETS, CTR_ESC_HIST);
    assert_eq!(CTR_ESC_HIST + ESC_HIST_BUCKETS, COUNTER_SLOTS);
    assert!(CTR_GRAD_HIST > CTR_GRAD_N, "the histogram must not overlap the gradient sum/count");
}

/// The shader keeps its own copy of the escape histogram's slot and bucket count ("keep in sync"):
/// a drift would write the histogram over another counter, or past the buffer, silently.
#[test]
fn the_shader_agrees_on_the_escape_histogram() {
    let wgsl = include_str!("mandelbrot.wgsl");
    let slot = format!("const CTR_ESC_HIST: u32 = {CTR_ESC_HIST}u;");
    assert!(wgsl.contains(&slot), "mandelbrot.wgsl must declare `{slot}`");
    let top = format!("clamp(floor(log2(max(sm, 1.0))), 0.0, {}.0)", ESC_HIST_BUCKETS - 1);
    assert!(wgsl.contains(&top), "esc_hist_commit must clamp to the last bucket: `{top}`");
}
