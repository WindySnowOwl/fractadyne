use super::*;

#[test]
fn an_opening_pass_prices_the_next_opening_at_once_and_forgets_slowly() {
    let mut p = WalkPricer::default();
    let area = 4_000u64;
    let budget = WALK_PASS_MS * 1.0e6;
    // Unmeasured: the prior, one pass's wall at 1.5 ns a pixel-step.
    assert_eq!(p.open(area), (budget / 1.5 / area as f64) as u32);
    // Measured dearer: the next opening takes the dearer price at once.
    p.observe_open(area, 100, 3.0 * 100.0 * area as f64 / 1.0e6);
    assert_eq!(p.open(area), (budget / 3.0 / area as f64) as u32);
    // Cheaper: a fifth of the way per walk, never the whole way at once.
    p.observe_open(area, 100, 1.0 * 100.0 * area as f64 / 1.0e6);
    let ns = 3.0 + 0.2 * (1.0 - 3.0);
    assert_eq!(p.open(area), (budget / ns / area as f64) as u32);
    // A cheap card's opening is bounded by pixel-steps; a big frame's is floored at 256.
    assert_eq!(WalkPricer { open_ns: 1.0e-6, pass_ms: WALK_PASS_MS }.open(1), WALK_OPEN_MAX_PX_STEPS as u32);
    assert_eq!(WalkPricer::default().open(4_000_000), WALK_MIN_ITERS);
}

#[test]
fn a_window_follows_its_wall_grows_at_most_twofold_and_stays_under_the_ceiling() {
    let p = WalkPricer::default();
    let area = 400_000u64;
    // On target: unchanged. Hot: shrinks in proportion. Cheap: at most doubles.
    assert_eq!(p.next(area, 4000, WALK_PASS_MS), 4000);
    assert_eq!(p.next(area, 4000, 4.0 * WALK_PASS_MS), 1000);
    assert_eq!(p.next(area, 4000, 1.0), 8000);
    // Never below the floor, never above the nominal ceiling.
    assert_eq!(p.next(area, 300, 100.0 * WALK_PASS_MS), WALK_MIN_ITERS);
    assert_eq!(p.next(area, 140_000, 1.0), (WALK_MAX_PX_STEPS / area as f64) as u32);
    // A garbage wall keeps the window.
    assert_eq!(p.next(area, 1000, f64::NAN), 1000);
    // A shorter pass target sizes both the opening and the steps for it.
    let short = WalkPricer::default().with_pass_ms(10.0);
    assert_eq!(short.open(4_000), (10.0e6 / 1.5 / 4_000.0) as u32);
    assert_eq!(short.next(area, 4000, 10.0), 4000);
    assert_eq!(short.next(area, 4000, 40.0), 1000);
}
