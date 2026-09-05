use super::*;

/// The signature is what makes "the same view" mean the same thing to the frame that SUBMITS a
/// reading and the frame that drains it, so it must key on everything that changes the escape
/// range: where you are, how deep, and how many iterations were asked for.
#[test]
fn the_signature_keys_on_view_and_ask() {
    let base = norm_signature(-0.75, 0.1, 20.0, 4096);
    assert_eq!(base, norm_signature(-0.75, 0.1, 20.0, 4096), "must be a pure function");
    for (what, other) in [
        ("centre x", norm_signature(-0.7500001, 0.1, 20.0, 4096)),
        ("centre y", norm_signature(-0.75, 0.1000001, 20.0, 4096)),
        ("depth", norm_signature(-0.75, 0.1, 20.5, 4096)),
        ("iteration ask", norm_signature(-0.75, 0.1, 20.0, 8192)),
    ] {
        assert_ne!(base, other, "{what} did not change the signature");
    }
}

/// ⚠**Never zero**, because zero is the "nothing echoed back yet" sentinel on the readback sink.
/// A real signature that collided with it would read as "no reading" and silently fall back.
#[test]
fn the_signature_is_never_the_no_reading_sentinel() {
    // The XOR of four zero-bit inputs is 0, which is exactly the case that must be remapped.
    assert_eq!(norm_signature(0.0, 0.0, 0.0, 0), 1);
    assert_ne!(norm_signature(0.0, 0.0, 0.0, 0), 0);
}

/// ⭐⭐**The field case, 2026-09-04.** An e10000 Misiurewicz jump: a reading measured at the OLD
/// view lands two frames after the jump. Before the signature travelled with it, it was attributed
/// to the view on screen and a shallow `[6, 191]` range got adopted and LOCKED for a view whose
/// real escapes sit at `[181573, 182297]` — one flat colour under the log mapping.
#[test]
fn a_reading_from_the_view_before_a_jump_is_not_current() {
    let before = norm_signature(-0.75, 0.1, 20.0, 4096);
    let after = norm_signature(-1.25, 0.05, 33_000.0, 1_000_000);
    assert_ne!(before, after);
    // Settled at the new view, a reading carrying the OLD signature is not ours.
    assert!(!norm_reading_is_current(before, after, false), "the stale reading was accepted");
    // The reading that actually belongs to this view is.
    assert!(norm_reading_is_current(after, after, false));
}

/// ⚠**While interacting every reading counts.** The view changes each frame, so a signature match
/// would essentially never happen and the normalization would starve exactly when it is chasing.
/// Motion already uses an EMA because its readings are approximate by nature.
#[test]
fn interaction_accepts_every_reading() {
    let a = norm_signature(-0.75, 0.1, 20.0, 4096);
    let b = norm_signature(-1.25, 0.05, 33_000.0, 1_000_000);
    assert!(norm_reading_is_current(a, b, true), "motion must not starve the normalization");
    assert!(norm_reading_is_current(0, b, true));
}

/// ⚠A frame that armed no counter readback echoes nothing, and `0` says so. Falling back to
/// accepting is the behaviour that shipped before the signature existed — a new mechanism that
/// silently discarded every reading on a path it did not cover would be far worse than the race
/// it replaces.
#[test]
fn nothing_echoed_falls_back_to_accepting() {
    let live = norm_signature(-0.75, 0.1, 20.0, 4096);
    assert!(norm_reading_is_current(0, live, false));
}

/// The attribution fix and the `norm_hold_break` heal are independent, and both stay: one stops
/// the wrong-adopt happening, the other recovers if it ever does. The healing rule is unchanged by
/// this work, and this pins that — the field range pair is the one from the report.
#[test]
fn the_defeasible_hold_still_heals_the_case_it_was_written_for() {
    // Held on the stale shallow reading, the real view's range is wildly disjoint.
    assert!(norm_hold_break((6.0, 191.0), (181_573.0, 182_297.0)));
    // A refinement of the held window is NOT a break — supersampling stages and settle tiles
    // legitimately share the held frame's range.
    assert!(!norm_hold_break((181_573.0, 182_297.0), (181_600.0, 182_200.0)));
}
