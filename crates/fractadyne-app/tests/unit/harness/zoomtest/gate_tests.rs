//! `--zoomtest-gate`: design/live-zoom-smoothing.md's P-G matrix criteria (W8).
use super::{held_max_oct, parse_zoom_gate, zoom_gate_failures, ZoomGate};

#[test]
fn the_held_bound_is_the_matrix_at_its_two_points() {
    assert!((held_max_oct(1.0) - 0.5).abs() < 1e-12);
    assert!((held_max_oct(4.0) - 1.0).abs() < 1e-12);
}

#[test]
fn a_smooth_glide_passes_and_each_criterion_fails_on_its_own() {
    assert!(zoom_gate_failures(ZoomGate::Smooth, 4.0, 48.0, 9, 0, 0.9).is_empty());
    assert_eq!(zoom_gate_failures(ZoomGate::Smooth, 4.0, 85.8, 9, 0, 0.9).len(), 1, "max interval");
    assert_eq!(zoom_gate_failures(ZoomGate::Smooth, 4.0, 48.0, 9, 1, 0.9).len(), 1, "an interval over 100 ms");
    // The 15-minute W6 dive at 1x: held 1.70 octaves against 0.5.
    assert_eq!(zoom_gate_failures(ZoomGate::Smooth, 1.0, 48.0, 0, 0, 1.70).len(), 1, "held");
}

#[test]
fn the_appetite_row_asks_only_for_no_interval_over_33_ms() {
    assert!(zoom_gate_failures(ZoomGate::Appetite, 4.0, 33.0, 0, 0, 5.0).is_empty());
    assert_eq!(zoom_gate_failures(ZoomGate::Appetite, 4.0, 40.0, 1, 0, 0.0).len(), 1);
}

#[test]
fn the_flag_reads_its_two_rows_and_nothing_else() {
    assert_eq!(parse_zoom_gate(None), Some(ZoomGate::Smooth));
    assert_eq!(parse_zoom_gate(Some("smooth")), Some(ZoomGate::Smooth));
    assert_eq!(parse_zoom_gate(Some("appetite")), Some(ZoomGate::Appetite));
    assert_eq!(parse_zoom_gate(Some("strict")), None);
}
