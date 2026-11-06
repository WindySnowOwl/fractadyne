use super::*;

/// A histogram with `n` samples at each of `values` (bucketed exactly as the shader does).
fn hist_of(values: &[(f32, u32)]) -> Vec<u32> {
    let mut h = vec![0u32; TRAP_HIST_BUCKETS];
    for &(v, n) in values {
        let k = ((v - TRAP_HIST_LO) * TRAP_HIST_PER_UNIT).clamp(0.0, (TRAP_HIST_BUCKETS - 1) as f32) as usize;
        h[k] += n;
    }
    h
}

/// ⭐The case it exists for: most of a view inside two units, a few pixels on nuclei far above. The
/// window spans the bulk; the stragglers do not stretch it.
#[test]
fn a_few_stragglers_do_not_set_the_window() {
    let mut vals: Vec<(f32, u32)> = (0..=22).map(|i| (0.85 + i as f32 * 0.1, 100)).collect();
    vals.push((10.3, 3));
    vals.push((66.0, 2));
    let (lo, hi) = trap_window(&hist_of(&vals), TRAP_WINDOW_QUANTILES).expect("a window");
    assert!((0.8..0.95).contains(&lo), "lo {lo}");
    assert!((2.9..3.2).contains(&hi), "hi {hi}");
}

/// Negative values (an orbit that never comes within 1 of the trap) are windowed like any other.
#[test]
fn negative_values_window_too() {
    let vals: Vec<(f32, u32)> = (0..=40).map(|i| (-7.5 + i as f32 * 0.25, 50)).collect();
    let (lo, hi) = trap_window(&hist_of(&vals), TRAP_WINDOW_QUANTILES).expect("a window");
    assert!(lo < -7.0 && hi > 2.0, "({lo}, {hi})");
}

/// No orbit-trap frame (nothing counted), too few samples, or a single bucket: no window, so the
/// app keeps the one it has instead of fitting to nothing.
#[test]
fn nothing_to_fit_is_no_window() {
    assert_eq!(trap_window(&vec![0u32; TRAP_HIST_BUCKETS], TRAP_WINDOW_QUANTILES), None);
    assert_eq!(trap_window(&hist_of(&[(1.0, 15)]), TRAP_WINDOW_QUANTILES), None);
    assert_eq!(trap_window(&hist_of(&[(1.0, 5_000)]), TRAP_WINDOW_QUANTILES), None);
}

/// The slots sit after the escape histogram, and the shader's constants say the same.
#[test]
fn the_slots_agree_with_the_shader() {
    assert_eq!(COUNTER_SLOTS, CTR_TRAP_HIST + TRAP_HIST_BUCKETS);
    let wgsl = include_str!("../../src/mandelbrot.wgsl");
    assert!(wgsl.contains(&format!("const CTR_TRAP_HIST: u32 = {CTR_TRAP_HIST}u;")));
    assert!(wgsl.contains(&format!("const TRAP_HIST_BUCKETS: u32 = {TRAP_HIST_BUCKETS}u;")));
    assert!(wgsl.contains(&format!("const TRAP_HIST_LO: f32 = {TRAP_HIST_LO:.1};")));
    assert!(wgsl.contains(&format!("const TRAP_HIST_PER_UNIT: f32 = {TRAP_HIST_PER_UNIT:.1};")));
}
