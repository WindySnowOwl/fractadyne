use super::*;

#[test]
fn the_probe_runs_only_just_below_the_switch() {
    let switch = crate::PERT_FE_THRESHOLD.log2();
    assert!(pre_switch_window(switch - 1.0, switch, 2.0));
    assert!(pre_switch_window(switch - 2.0, switch, 2.0));
    assert!(!pre_switch_window(switch - 2.1, switch, 2.0), "too far ahead");
    assert!(!pre_switch_window(switch, switch, 2.0), "at the switch it is floatexp already");
    assert!(!pre_switch_window(f64::NAN, switch, 2.0));
}

#[test]
fn the_probe_is_the_knees_size_with_a_bounded_iteration_count() {
    // The RTX 3080's knee: 512×512, ~30 iterations for 8e6 steps.
    assert_eq!(pre_switch_probe_shape(262_144, 8_000_000, 100_000), (512, 30));
    // The RX 6800 XT's: 724×724, ~15.
    assert_eq!(pre_switch_probe_shape(524_288, 8_000_000, 100_000), (725, 15));
    // Never fewer than 8 iterations (the pass's fixed cost would dominate), never past the ask.
    assert_eq!(pre_switch_probe_shape(262_144, 1_000, 100_000).1, 8);
    assert_eq!(pre_switch_probe_shape(262_144, 8_000_000, 20).1, 20);
    // An unknown knee still probes at a size, not a pixel.
    assert_eq!(pre_switch_probe_shape(0, 8_000_000, 100_000).0, 64);
}

#[test]
fn a_probe_counts_only_when_its_pixels_ran_every_iteration() {
    // r < 0 = still running at the last iteration (not escaped).
    let running = |n: usize, escaped: usize| -> Vec<f32> {
        (0..n).flat_map(|i| [if i < escaped { 5.0 } else { -1.0 }, 0.0, 0.0, 0.0]).collect()
    };
    assert_eq!(pre_switch_probe_representative(&running(100, 0), 10, 10), (1.0, true));
    assert_eq!(pre_switch_probe_representative(&running(100, 2), 10, 10), (0.98, true));
    // Pixels that escaped early ran fewer steps than counted: the pass priced floatexp cheap.
    assert!(!pre_switch_probe_representative(&running(100, 3), 10, 10).1);
    // So did glitched ones (-2, out of the loop at the flag).
    let mut glitched = running(100, 0);
    for i in 0..3 {
        glitched[i * 4] = -2.0;
    }
    assert!(!pre_switch_probe_representative(&glitched, 10, 10).1);
    // A short readback is not a measurement.
    assert!(!pre_switch_probe_representative(&running(50, 0), 10, 10).1);
}

#[test]
fn a_probe_records_its_slowest_pass_and_never_more_than_df32_allows() {
    let steps = 7_864_320; // 512×512 × 30
    // The slowest of its passes.
    let r = pre_switch_probe_rate(steps, &[3.05, 3.18, 3.11], 2.7e7, 8.0).unwrap();
    assert!((r - steps as f64 / 3.18).abs() < 1.0, "{r}");
    // The fast readings of 2026-10-18 (0.61 ms for a pass that read 3.1 elsewhere) are capped at
    // df32 / 8, not believed.
    assert_eq!(pre_switch_probe_rate(steps, &[0.61, 0.61], 2.7e7, 8.0), Some(2.7e7 / 8.0));
    // One pass is not a probe; no df32 rate to check against, or an unusable time: nothing.
    assert_eq!(pre_switch_probe_rate(steps, &[3.0], 2.7e7, 8.0), None);
    assert_eq!(pre_switch_probe_rate(steps, &[3.0, 3.0], 0.0, 8.0), None);
    assert_eq!(pre_switch_probe_rate(steps, &[0.0, 3.0], 2.7e7, 8.0), None);
    assert_eq!(pre_switch_probe_rate(steps, &[f64::NAN, 3.0], 2.7e7, 8.0), None);
}

#[test]
fn a_pass_is_timed_by_the_gpu_when_it_can_be_and_by_the_wall_when_not() {
    assert_eq!(pre_switch_pass_ms([3.1, 3.4]), 3.1);
    // No TIMESTAMP_QUERY: the timestamp is NaN.
    assert_eq!(pre_switch_pass_ms([f64::NAN, 3.4]), 3.4);
    assert_eq!(pre_switch_pass_ms([0.0, 3.4]), 3.4);
}
