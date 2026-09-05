use super::*;

/// Twelve plausible baseline costs, spanning the two decades the matrix actually covers (15 ms to
/// 500 ms) — the spread is what lets a fit tell a fixed cost from a scale at all.
fn bases() -> Vec<f64> {
    vec![15.1, 15.8, 16.9, 17.4, 20.8, 25.3, 30.6, 62.0, 67.4, 108.9, 168.9, 504.1]
}

/// ⭐⭐**A FIXED cost is the case this exists for**, because it is the one a list of percentages
/// misreads. Add the same 9.4 ms to every segment and the fit must say so — scale 1.00, fixed 9.4 —
/// even though the per-segment percentages run from +62% down to +1.9%.
#[test]
fn a_fixed_cost_reads_as_fixed_not_as_a_scale() {
    let pts: Vec<(f64, f64)> = bases().into_iter().map(|b| (b, b + 9.4)).collect();
    let (scale, fixed) = scale_and_fixed(&pts).expect("enough points");
    assert!((scale - 1.0).abs() < 1e-6, "scale should be 1.00, got {scale}");
    assert!((fixed - 9.4).abs() < 1e-6, "fixed should be 9.4, got {fixed}");
    // The percentages this same data produces, to show what the fit is protecting a reader from.
    let worst = 9.4 / 15.1 * 100.0;
    let best = 9.4 / 504.1 * 100.0;
    assert!(worst > 60.0 && best < 2.0, "one cause, {worst:.0}% to {best:.1}% - {pts:?}");
}

/// A SCALE change is the other thing, and it must not be reported as a fixed cost: everything is
/// proportionally slower, which is a real speed difference rather than a per-call overhead.
#[test]
fn a_scale_change_reads_as_a_scale_not_as_fixed() {
    let pts: Vec<(f64, f64)> = bases().into_iter().map(|b| (b, b * 1.25)).collect();
    let (scale, fixed) = scale_and_fixed(&pts).expect("enough points");
    assert!((scale - 1.25).abs() < 1e-6, "scale should be 1.25, got {scale}");
    assert!(fixed.abs() < 1e-6, "fixed should be ~0, got {fixed}");
}

/// The two are separable when both are present — which is the real situation measured on this box
/// (a slightly FASTER renderer hiding under a fixed compile cost). Reporting only "11 slower"
/// states the opposite of the truth.
#[test]
fn a_faster_renderer_under_a_fixed_cost_is_still_visible() {
    let pts: Vec<(f64, f64)> = bases().into_iter().map(|b| (b, b * 0.93 + 10.2)).collect();
    let (scale, fixed) = scale_and_fixed(&pts).expect("enough points");
    assert!((scale - 0.93).abs() < 1e-6, "got {scale}");
    assert!((fixed - 10.2).abs() < 1e-6, "got {fixed}");
    assert!(scale < 1.0, "the renderer is faster per unit of work, and the fit must say so");
}

/// An unchanged run is scale 1, fixed 0 — so the diagnostic stays quiet when there is nothing to
/// say. A line that appears on every healthy run is the very problem this is answering.
#[test]
fn an_unchanged_run_reports_nothing_notable() {
    let pts: Vec<(f64, f64)> = bases().into_iter().map(|b| (b, b)).collect();
    let (scale, fixed) = scale_and_fixed(&pts).expect("enough points");
    assert!((scale - 1.0).abs() < 1e-9 && fixed.abs() < 1e-9);
    // The report's own thresholds: neither branch would print.
    assert!(fixed.abs() <= 3.0 && (scale - 1.0).abs() <= 0.10);
}

/// ⚠Degenerate input returns `None` rather than a garbage line. Too few points cannot separate the
/// two terms at all, and identical baselines are a vertical fit with no unique answer.
#[test]
fn degenerate_input_declines_to_fit() {
    assert!(scale_and_fixed(&[]).is_none());
    assert!(scale_and_fixed(&[(10.0, 20.0); 5]).is_none(), "fewer than six points");
    assert!(scale_and_fixed(&[(10.0, 20.0); 12]).is_none(), "every baseline identical");
    // Real noise must not stop it fitting: the same fixed cost with +/-15% jitter still reads.
    let noisy: Vec<(f64, f64)> = bases()
        .into_iter()
        .enumerate()
        .map(|(i, b)| (b, b + 9.4 + if i % 2 == 0 { b * 0.05 } else { -b * 0.05 }))
        .collect();
    let (scale, fixed) = scale_and_fixed(&noisy).expect("noisy but fittable");
    assert!((scale - 1.0).abs() < 0.10, "scale {scale} drifted under noise");
    assert!(fixed > 3.0, "the fixed cost {fixed} should still be visible under noise");
}
