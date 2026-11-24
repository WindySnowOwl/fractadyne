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

/// The slots sit after the escape histogram, the inside's after them (C4), and the shader's
/// constants say the same.
#[test]
fn the_slots_agree_with_the_shader() {
    assert_eq!(CTR_TRAP_HIST_IN, CTR_TRAP_HIST + TRAP_HIST_BUCKETS);
    assert_eq!(COUNTER_SLOTS, CTR_TRAP_HIST_IN + TRAP_HIST_BUCKETS);
    let wgsl = include_str!("../../src/mandelbrot.wgsl");
    assert!(wgsl.contains(&format!("const CTR_TRAP_HIST: u32 = {CTR_TRAP_HIST}u;")));
    assert!(wgsl.contains(&format!("const CTR_TRAP_HIST_IN: u32 = {CTR_TRAP_HIST_IN}u;")));
    // Every interior exit counts into the inside's histogram, every escape does not.
    assert_eq!(wgsl.matches("trap_hist_commit(aux, gx, gy, true);").count(), 3);
    assert_eq!(wgsl.matches("trap_hist_commit(aux, gx, gy, false);").count(), 3);
    assert!(wgsl.contains(&format!("const TRAP_HIST_BUCKETS: u32 = {TRAP_HIST_BUCKETS}u;")));
    assert!(wgsl.contains(&format!("const TRAP_HIST_LO: f32 = {TRAP_HIST_LO:.1};")));
    assert!(wgsl.contains(&format!("const TRAP_HIST_PER_UNIT: f32 = {TRAP_HIST_PER_UNIT:.1};")));
}

/// The trap's distribution (coloring plan C6c): its ends the outer edges of the lowest and highest
/// buckets in use, every knot never below the one before, and the share below a knot its own (the
/// samples spread evenly inside a bucket); fewer than 16 samples say nothing.
#[test]
fn the_trap_distribution_is_the_histograms_quantiles() {
    use fractadyne_core::coloring::{cdf_at, EQ_QUANTILES};
    let vals: Vec<(f32, u32)> = (0..=60).map(|i| (-1.0 + i as f32 * 0.1, 10 + (i % 7) * 5)).chain([(9.6, 4), (40.0, 1)]).collect();
    let h = hist_of(&vals);
    let q = trap_quantiles(&h).expect("a distribution");
    let edge = |k: usize| TRAP_HIST_LO + k as f32 / TRAP_HIST_PER_UNIT;
    let (first, last) = (h.iter().position(|&n| n > 0).unwrap(), h.iter().rposition(|&n| n > 0).unwrap());
    assert_eq!((q[0], q[EQ_QUANTILES]), (edge(first), edge(last + 1)));
    assert!(q.windows(2).all(|w| w[0] <= w[1]), "never decreasing");
    // The share at or below a knot, counted from the buckets (spread evenly inside each): its own.
    let total: f64 = h.iter().map(|&n| n as f64).sum();
    for i in (8..EQ_QUANTILES).step_by(8) {
        let v = q[i];
        let mut below = 0.0;
        for (k, &n) in h.iter().enumerate() {
            let (a, b) = (edge(k), edge(k + 1));
            below += n as f64 * (((v - a) / (b - a)) as f64).clamp(0.0, 1.0);
        }
        assert!((below / total - i as f64 / EQ_QUANTILES as f64).abs() < 1.0e-4, "knot {i} at {v}: share {}", below / total);
        // And the CDF Equalize reads off the knots agrees.
        assert!((cdf_at(&q, v as f64) - i as f64 / EQ_QUANTILES as f64).abs() <= 1.0 / EQ_QUANTILES as f64);
    }
    assert!(trap_quantiles(&hist_of(&[(1.0, 15)])).is_none(), "15 samples");
    assert!(trap_quantiles(&hist_of(&[(1.0, 16)])).is_some(), "16 samples");
}
