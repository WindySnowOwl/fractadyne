//! `--chunk-sweep`'s fixed-cost fit and gate (design §5.2, §7.3 gate 1).

use super::*;

fn walk(window: u32, passes: usize, total_ms: f64) -> Walk {
    Walk { window, passes, total_ms }
}

#[test]
fn the_fit_recovers_a_known_fixed_cost() {
    // 0.4 ms per pass on top of 100 ms of work and a 5 ms first-pass setup.
    let w: Vec<Walk> = [(128, 32), (256, 16), (512, 8), (1024, 4), (2048, 2)]
        .iter()
        .map(|&(win, n)| walk(win, n, 0.4 * n as f64 + 105.0))
        .collect();
    let f = fit_fixed_term(&w).unwrap();
    assert!((f.fixed - 0.4).abs() < 1e-9, "{f:?}");
    assert!((f.rest - 105.0).abs() < 1e-9, "{f:?}");
    assert!(f.resolved(), "an exact line is resolved");
}

#[test]
fn the_rtx_3080_sweep_of_2026_10_13_resolves_its_fixed_cost() {
    // The all-interior view with the warm-up, 1280x787.
    let w = [
        walk(64, 64, 126.2),
        walk(128, 32, 125.5),
        walk(256, 16, 120.1),
        walk(512, 8, 117.2),
        walk(1024, 4, 115.3),
        walk(2048, 2, 114.8),
    ];
    let f = fit_fixed_term(&w).unwrap();
    assert!((0.15..0.25).contains(&f.fixed), "{f:?}");
    assert!(f.resolved(), "{f:?}");
}

#[test]
fn the_rx_6800_xt_sweep_of_2026_10_13_is_unresolved_not_negative() {
    // dev.5 at the all-interior view: totals RISE with window size, so the slope is below zero.
    let w = [
        walk(64, 64, 230.1),
        walk(128, 32, 204.3),
        walk(256, 16, 207.7),
        walk(512, 8, 216.7),
        walk(1024, 4, 228.9),
        walk(2048, 2, 240.2),
    ];
    let f = fit_fixed_term(&w).unwrap();
    assert!(!f.resolved(), "{f:?}");
    // And it is never held to a calibration: an unresolved fit cannot say the value moved.
    let red = sweep_gate(&w, Some(f), Some(0.2), None);
    assert!(red.is_empty(), "{red:?}");
}

#[test]
fn the_rx_6800_xt_sweep_of_2026_09_27_fits_about_a_quarter_millisecond() {
    // The archived window axis at the all-interior view, 1280x787 (request 20260927-220955-mahc).
    let w = [
        walk(64, 64, 210.8),
        walk(128, 32, 214.0),
        walk(256, 16, 203.8),
        walk(512, 8, 199.0),
        walk(1024, 4, 197.5),
        walk(2048, 2, 193.1),
    ];
    let f = fit_fixed_term(&w).unwrap();
    assert!((0.2..0.35).contains(&f.fixed), "{f:?}");
}

#[test]
fn too_few_walks_or_no_spread_is_no_fit() {
    assert!(fit_fixed_term(&[walk(64, 64, 1.0), walk(128, 32, 1.0)]).is_none());
    assert!(fit_fixed_term(&[walk(64, 4, 1.0), walk(128, 4, 2.0), walk(256, 4, 3.0)]).is_none());
    // Walks that did nothing past the skip do not count.
    assert!(fit_fixed_term(&[walk(64, 0, 0.0), walk(128, 0, 0.0), walk(256, 4, 3.0)]).is_none());
}

#[test]
fn a_sweep_that_chunked_nothing_is_vacuous_not_a_pass() {
    let red = sweep_gate(&[walk(64, 0, 0.0), walk(128, 0, 0.0)], None, Some(0.3), None);
    assert!(red.iter().any(|r| r.starts_with("VACUOUS")), "{red:?}");
}

fn exact(fixed: f64) -> Option<Fit> {
    Some(Fit { fixed, rest: 100.0, se: 0.0 })
}

#[test]
fn the_fixed_cost_is_held_to_the_calibration_within_2x() {
    let w = [walk(128, 32, 1.0), walk(256, 16, 1.0), walk(512, 8, 1.0)];
    assert!(sweep_gate(&w, exact(0.30), Some(0.28), None).is_empty());
    assert!(sweep_gate(&w, exact(0.55), Some(0.28), None).is_empty(), "1.96x is inside");
    assert!(!sweep_gate(&w, exact(0.60), Some(0.28), None).is_empty(), "2.1x is outside");
    assert!(!sweep_gate(&w, exact(0.10), Some(0.28), None).is_empty(), "0.36x is outside");
    // A card with no calibrated value is not held to anything; the default is not a measurement.
    assert!(sweep_gate(&w, exact(5.0), None, None).is_empty());
}

#[test]
fn an_odd_height_cliff_past_10x_fails() {
    let rows = [(1.0, 482, 470.0), (1.0, 484, 480.0), (1.0, 483, 5261.0)];
    let p = parity_ratio(&rows).unwrap();
    assert!(p > 10.0, "{p}");
    let w = [walk(128, 32, 1.0), walk(256, 16, 1.0), walk(512, 8, 1.0)];
    assert!(!sweep_gate(&w, exact(0.3), None, Some(p)).is_empty());
    // The padded build: odd within a few percent of even.
    let flat = [(0.5, 296, 18.9), (0.5, 298, 19.1), (0.5, 297, 20.0)];
    assert!(parity_ratio(&flat).unwrap() < 1.2);
}
