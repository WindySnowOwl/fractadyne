use super::chunk_step_factor;

#[test]
fn a_healthy_walk_is_untouched() {
    // Present-paced frames (16–60 ms) sit far under any target: full budget, factor 1.
    assert_eq!(chunk_step_factor(16.0, 400.0), 1.0);
    assert_eq!(chunk_step_factor(399.0, 400.0), 1.0);
}

#[test]
fn a_hot_pass_prices_the_next_one_down_proportionally() {
    // The field kill: ~1000 ms passes against a 400 ms target → next pass 0.4× the budget.
    let f = chunk_step_factor(1000.0, 400.0);
    assert!((f - 0.4).abs() < 1e-12, "got {f}");
}

#[test]
fn the_floor_holds_against_absurd_readings() {
    // A 2 s (watchdog-scale) interval floors at 1/16 — the retreat and the pacer own the
    // regime below this; the factor must never zero the step.
    assert_eq!(chunk_step_factor(400.0 * 64.0, 400.0), 1.0 / 16.0);
}

#[test]
fn no_reading_means_no_opinion() {
    assert_eq!(chunk_step_factor(0.0, 400.0), 1.0);
    assert_eq!(chunk_step_factor(-5.0, 400.0), 1.0);
    assert_eq!(chunk_step_factor(f64::NAN, 400.0), 1.0);
}
