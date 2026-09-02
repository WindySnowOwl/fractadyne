use super::*;

#[test]
fn the_backlog_counts_only_representative_dispatches() {
    // The same 0.7 boundary budget_step uses to accept a reading.
    assert!(motion_jam_counts(70, 100));
    assert!(!motion_jam_counts(69, 100));
    // A jam-clamped bootstrap dispatch against a grown budget must never count itself,
    // or the gate would never release.
    assert!(!motion_jam_counts(4_000_000, 60_000_000_000));
}

#[test]
fn the_gate_trips_at_the_cap_and_only_during_motion() {
    assert!(motion_jammed(2, 2, true, false, false));
    assert!(!motion_jammed(1, 2, true, false, false));
    // Settled frames are the walk's business, not this gate's.
    assert!(!motion_jammed(5, 2, false, false, false));
    // Reproject frames dispatch nothing; offscreen renders have their own cap.
    assert!(!motion_jammed(5, 2, true, true, false));
    assert!(!motion_jammed(5, 2, true, false, true));
}

#[test]
fn a_large_override_disables_it() {
    // The no-rebuild bisection lever: --set MOTION_UNPRICED_MAX=999.
    assert!(!motion_jammed(u32::MAX, 999_999_999_999, true, false, false));
}

