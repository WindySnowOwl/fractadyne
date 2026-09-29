use super::*;

#[test]
fn equal_parts_replace_the_remainder_row() {
    // The measured failure: 1080 rows at 1024-px tiles left a 56-px last row cut into 56² tiles.
    assert_eq!(balanced_extent(1080, 1024), 540);
    assert_eq!(balanced_extent(1920, 1024), 960);
    assert_eq!(balanced_extent(2160, 1024), 720);
    assert_eq!(balanced_extent(3840, 1024), 960);
    // The selftest's pinned frame: 220 px at a 70-px cap is four parts, i.e. 16 tiles.
    assert_eq!(balanced_extent(220, 70), 55);
    assert_eq!(balanced_extent(100, 1024), 100, "a frame smaller than a tile is one part");
    assert_eq!(balanced_extent(1, 1024), 1);
}

#[test]
fn the_occupancy_tile_never_shrinks_a_tile() {
    assert_eq!(occupancy_tile(158, 1), OCC_TILE_SAMPLES, "800k iterations: 158 px grows");
    assert_eq!(occupancy_tile(158, 2), OCC_TILE_SAMPLES / 2, "the side is in SAMPLES");
    assert_eq!(occupancy_tile(2048, 1), 2048, "a larger nominal tile is kept");
}

#[test]
fn the_first_pass_is_priced_from_the_prior() {
    let p = StepPricer::new();
    let area = 1_048_576u64;
    let want = (STEP_TARGET_MS * 1.0e6 / STEP_PRIOR_NS / area as f64) as u32;
    assert_eq!(p.cap(area), want);
    assert!(want as f64 * area as f64 <= STEP_MAX_PX_STEPS, "the prior pass is inside the ceiling");
}

#[test]
fn a_measurement_replaces_the_prior_and_then_only_raises_the_price() {
    let area = 1_048_576u64;
    let mut p = StepPricer::new();
    // A cheap first pass (0.1 ns per pixel-step) replaces the 1.5 ns guess...
    p.observe(0.1e-6 * area as f64 * 100.0, area, 100);
    let cheap = p.cap(area);
    assert!(cheap > StepPricer::new().cap(area), "evidence of cheap steps must widen the cap");
    // ...but is clamped by the ceiling, which no measurement lifts.
    assert!(cheap as f64 * area as f64 <= STEP_MAX_PX_STEPS + area as f64);
    // A dearer pass raises the price; a cheaper one after it does not lower it again.
    p.observe(1.0e-6 * area as f64 * 100.0, area, 100);
    let dear = p.cap(area);
    assert!(dear < cheap);
    p.observe(0.05e-6 * area as f64 * 100.0, area, 100);
    assert_eq!(p.cap(area), dear, "the worst cost seen only rises");
}

#[test]
fn a_nearly_finished_pass_does_not_price_steps() {
    // A few hundred active pixels leave the card idle: the wall is one chain, not per-step cost
    // (measured up to 1,500 ns per pixel-step). Pricing from it would shrink every later cap.
    let area = 1_048_576u64;
    let mut p = StepPricer::new();
    p.observe(0.1e-6 * area as f64 * 100.0, area, 100);
    let before = p.cap(area);
    p.observe(5.0, 300, 100);
    assert_eq!(p.cap(area), before);
}

#[test]
fn the_cap_has_a_floor_on_a_pathological_card() {
    let area = 1_048_576u64;
    let mut p = StepPricer::new();
    p.observe(1.0e9, area, 100); // absurdly slow
    assert_eq!(p.cap(area), STEP_MIN_CAP);
}
