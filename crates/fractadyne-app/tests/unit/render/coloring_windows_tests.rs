use super::coloring_windows;

const ESCAPE: [Option<(f32, f32)>; 2] = [Some((721.3, 1245.3)), Some((721.5, 1266.9))];
const TRAP: [Option<(f32, f32)>; 2] = [Some((10.0, 14.58)), Some((10.0, 14.89))];

/// Smooth and the other methods are colored by the escape windows, fitted or not.
#[test]
fn the_escape_methods_restart_for_the_escape_windows() {
    assert_eq!(coloring_windows(false, true, ESCAPE, TRAP), ESCAPE);
    assert_eq!(coloring_windows(false, false, ESCAPE, TRAP), ESCAPE);
}

/// ⭐⭐The orbit trap with "Always fit range" is colored by the trap's window alone: the escape
/// window, still measured and moving with the samples, must not restart its average (it did, for
/// as long as a Multibrot 8 view stayed on screen, 2026-11-07).
#[test]
fn a_fitted_orbit_trap_restarts_for_its_own_window_only() {
    assert_eq!(coloring_windows(true, true, ESCAPE, TRAP), TRAP);
}

/// Unticked, the orbit trap keeps its fixed scaling: neither window colors it.
#[test]
fn an_unfitted_orbit_trap_restarts_for_neither_window() {
    assert_eq!(coloring_windows(true, false, ESCAPE, TRAP), [None, None]);
}
