//! The hold for a clamped reference (`RefWait`, `ref_wait_step`): an empty view under a truncated
//! reference holds the dive still until the reference serves the view, and only a reference that
//! never does turns into a dead end.

use super::*;

#[test]
fn a_dive_that_is_not_holding_steers() {
    assert_eq!(ref_wait_step(None, 12.0, false), RefWait::Steer);
    // The flag alone does not start a hold: only an empty view does (the caller's job).
    assert_eq!(ref_wait_step(None, 12.0, true), RefWait::Steer);
}

#[test]
fn a_hold_lasts_while_the_reference_clamps_the_view() {
    assert_eq!(ref_wait_step(Some(10.0), 10.0, true), RefWait::Hold);
    assert_eq!(ref_wait_step(Some(10.0), 10.0 + REF_WAIT_MAX, true), RefWait::Hold);
}

#[test]
fn the_dive_resumes_once_the_reference_serves_the_view() {
    assert_eq!(ref_wait_step(Some(10.0), 11.1, false), RefWait::Resume);
    // Even after the limit: a reference that grew in time is not a dead end.
    assert_eq!(ref_wait_step(Some(10.0), 10.0 + 2.0 * REF_WAIT_MAX, false), RefWait::Resume);
}

#[test]
fn a_reference_that_never_grows_is_a_dead_end_after_the_limit() {
    assert_eq!(ref_wait_step(Some(10.0), 10.0 + REF_WAIT_MAX + 0.01, true), RefWait::GiveUp);
}

/// A 4x4 probe from smooth counts (negative = interior), at cap `cap`.
fn probe(iter: [f32; 16], cap: u32) -> Probe {
    Probe { w: 4, h: 4, iter: iter.to_vec(), de_log2: vec![1.0e30; 16], normal: vec![(0.0, 0.0); 16], cap }
}

#[test]
fn an_interior_beside_counts_at_the_cap_is_unproven() {
    // The 2^648 replay's shape: a straight edge of "interior" against escapes at 259,134-259,135
    // under a cap of 259,136.
    let x = -1.0;
    let p = probe(
        [259_134.6, 259_135.1, x, x, 259_134.2, 259_134.9, 259_135.3, x, 259_133.8, 259_134.4, 259_134.8, 259_135.2, 259_133.5, 259_134.0, 259_134.5, 259_134.9],
        259_136,
    );
    assert!(p.interior_at_cap());
}

#[test]
fn an_interior_whose_neighbours_escape_well_below_the_cap_is_the_sets() {
    // A real minibrot under a reference long enough: its neighbours escape far below the cap.
    let x = -1.0;
    let p = probe([1200.0, 1210.5, x, x, 1190.0, 1205.0, 1230.0, x, 1180.0, 1195.0, 1210.0, 1240.0, 1170.0, 1185.0, 1200.0, 1220.0], 259_136);
    assert!(!p.interior_at_cap());
}

#[test]
fn a_probe_with_no_interior_or_no_escape_is_not_called_clamped() {
    // All escaped, some at the cap: nothing reads interior, so nothing was cut.
    assert!(!probe([259_135.0; 16], 259_136).interior_at_cap());
    // All interior: no escaped cell ran up to the cap (the dead-end path decides this one).
    assert!(!probe([-1.0; 16], 259_136).interior_at_cap());
}

/// A stepped dive waiting for a frame at its depth is reported once it has waited
/// `STEP_WAIT_WARN`, and not before (2026-11-01: it waited silently for minutes).
#[test]
fn a_stepped_dive_that_waits_too_long_is_reported() {
    assert!(!step_stalled(None, 100.0));
    assert!(!step_stalled(Some(100.0), 100.0 + STEP_WAIT_WARN));
    assert!(step_stalled(Some(100.0), 100.0 + STEP_WAIT_WARN + 0.01));
}

/// A stepped dive steps on a frame within an octave of its depth, and not on one further back.
#[test]
fn a_stepped_dive_steps_only_on_a_frame_near_its_depth() {
    assert!(step_frame_ready(1032.08, 1032.08));
    assert!(step_frame_ready(1032.08, 1031.2));
    assert!(!step_frame_ready(1032.08, 1031.08));
    assert!(!step_frame_ready(1032.08, 1030.08));
}
