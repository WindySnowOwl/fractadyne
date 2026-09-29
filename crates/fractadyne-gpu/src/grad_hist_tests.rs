//! The step histogram's readback, including the bucket the shader deliberately never writes.

use crate::{
    grad_hist_from_slots, StepStats, COUNTER_SLOTS, CTR_CHUNK_RUNNING, CTR_ESC_HIST,
    CTR_GRAD_HIST, CTR_GRAD_N, CTR_STEP_BIG, CTR_STEP_EXEC, CTR_STEP_FULL, CTR_STEP_ITER,
    CTR_STEP_PX, ESC_HIST_BUCKETS, GRAD_HIST_BUCKETS,
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
    // The slot map: the step histogram, the escape histogram, then the step accounting's one count
    // and four two-word sums as the tail, nothing overlapping.
    assert_eq!(CTR_GRAD_HIST + GRAD_HIST_BUCKETS, CTR_ESC_HIST);
    assert_eq!(CTR_ESC_HIST + ESC_HIST_BUCKETS, CTR_STEP_PX);
    assert_eq!(
        [CTR_STEP_EXEC, CTR_STEP_ITER, CTR_STEP_FULL, CTR_STEP_BIG],
        [CTR_STEP_PX + 1, CTR_STEP_PX + 3, CTR_STEP_PX + 5, CTR_STEP_PX + 7]
    );
    // ...and the step-bounded runner's one-word running count after the last two-word sum.
    assert_eq!(CTR_STEP_BIG + 2, CTR_CHUNK_RUNNING);
    assert_eq!(CTR_CHUNK_RUNNING + 1, COUNTER_SLOTS);
    assert!(CTR_GRAD_HIST > CTR_GRAD_N, "the histogram must not overlap the gradient sum/count");
}

/// The shader's own copy of the step-accounting slots ("keep in sync"): a drift would add into
/// another counter's slot, silently.
#[test]
fn the_shader_agrees_on_the_step_accounting_slots() {
    let wgsl = include_str!("mandelbrot.wgsl");
    for (name, v) in [
        ("CTR_STEP_PX", CTR_STEP_PX),
        ("CTR_STEP_EXEC", CTR_STEP_EXEC),
        ("CTR_STEP_ITER", CTR_STEP_ITER),
        ("CTR_STEP_FULL", CTR_STEP_FULL),
        ("CTR_STEP_BIG", CTR_STEP_BIG),
        ("CTR_CHUNK_RUNNING", CTR_CHUNK_RUNNING),
    ] {
        let decl = format!("const {name}: u32 = {v}u;");
        assert!(wgsl.contains(&decl), "mandelbrot.wgsl must declare `{decl}`");
    }
}

#[test]
fn step_sums_are_read_as_two_words_with_the_carry() {
    let mut slots = [0u32; COUNTER_SLOTS];
    slots[CTR_STEP_PX] = 2025;
    slots[CTR_STEP_EXEC] = 7;
    slots[CTR_STEP_ITER] = 5; // lo
    slots[CTR_STEP_ITER + 1] = 3; // hi: three carries
    let s = StepStats::from_slots(&slots);
    assert_eq!(s.sampled_px, 2025);
    assert_eq!(s.iterations, (3u64 << 32) + 5);
    assert_eq!(s.executed, 7);
    // Summed over tiles, each half is summed on its own: a lo total past 2^32 is still right.
    let mut wide = [0u64; COUNTER_SLOTS];
    wide[CTR_STEP_ITER] = (1u64 << 32) + 9; // two tiles' lo words, summed
    wide[CTR_STEP_ITER + 1] = 2; // their carries
    wide[CTR_STEP_EXEC] = 100;
    let w = StepStats::from_u64_slots(&wide);
    assert_eq!(w.iterations, (3u64 << 32) + 9);
    assert!((w.iters_per_step() - w.iterations as f64 / 100.0).abs() < 1e-6);
    assert_eq!(StepStats::default().iters_per_step(), 0.0, "nothing executed reads as 0, not NaN");
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
