use super::*;

/// The user report this closes: "colours shift under them as a deep view settles — the
/// picture is re-mapped, not just sharpened." Same signature + already decided ⇒ every
/// further reading (ss ramp stages, settle tiles, chunked-walk refreshes) is HELD.
#[test]
fn a_decided_view_holds_through_refinement() {
    assert_eq!(norm_feed_decision(42, 42, true, false), NormFeed::Hold);
}

/// The first settled reading of a new view adopts outright — decided once, at the moment
/// the picture is visibly arriving anyway. A signature the mapping was never decided for
/// adopts even if it matches the stored one (locked = false: e.g. after `invalidate_refs`).
#[test]
fn a_new_or_undecided_view_adopts() {
    assert_eq!(norm_feed_decision(43, 42, true, false), NormFeed::Adopt, "new signature");
    assert_eq!(norm_feed_decision(42, 42, false, false), NormFeed::Adopt, "same sig, undecided");
}

/// Motion always chases with the EMA — the pre-hold behaviour, so a dive's mapping glides.
/// Interaction outranks the lock: a reading that arrives mid-drag must not adopt-and-lock
/// a mapping for a view the camera is already leaving.
#[test]
fn motion_always_chases() {
    assert_eq!(norm_feed_decision(43, 42, false, true), NormFeed::Chase);
    assert_eq!(norm_feed_decision(42, 42, true, true), NormFeed::Chase);
}

/// A stationary click-and-release over a decided view stays held: interaction chased (a
/// no-op on an unchanged view), and on release the unchanged signature + lock hold again —
/// the mapping never wobbles from merely touching the view.
#[test]
fn touching_a_decided_view_does_not_remap_it() {
    // during the press
    assert_eq!(norm_feed_decision(42, 42, true, true), NormFeed::Chase);
    // after release: sig unchanged, still locked
    assert_eq!(norm_feed_decision(42, 42, true, false), NormFeed::Hold);
}

/// A held mapping is defeasible (`norm_hold_break`): a whole-frame reading DISJOINT from the
/// held window beyond the 4x-width slack proves the hold maps a different picture. The field
/// case shaped these numbers: a stale [6, 191] adopted for a view whose escapes sit at
/// [181573, 182297] — under the log mapping, one flat colour until the next signature change.
#[test]
fn a_disjoint_reading_breaks_the_hold() {
    use super::norm_hold_break;
    // The 2026-09-04 e10000 field case: reading five orders above the held window.
    assert!(norm_hold_break((6.0, 191.0), (181_573.0, 181_752.0)));
    // And the mirror image (held deep, shallow frame arrives — e.g. after a Home jump).
    assert!(norm_hold_break((181_573.0, 182_297.0), (6.0, 191.0)));
}

/// Refinement readings — subsets, near-misses, modest extensions — must NOT break the hold:
/// that stability is the entire point of holding (the settle re-mapping report).
#[test]
fn refinement_readings_keep_the_hold() {
    use super::norm_hold_break;
    let held = (181_573.0, 182_297.0);
    assert!(!norm_hold_break(held, (181_573.0, 181_752.0)), "subset");
    assert!(!norm_hold_break(held, (181_400.0, 182_400.0)), "modest extension");
    assert!(!norm_hold_break(held, (182_300.0, 183_000.0)), "adjacent above, inside slack");
    // Slack is 4x the held width (724 -> 2896): a reading starting just inside survives...
    assert!(!norm_hold_break(held, (185_100.0, 185_200.0)));
    // ...and one clearly beyond it does not.
    assert!(norm_hold_break(held, (185_300.0, 185_400.0)));
}

/// A degenerate held window (width ~0) still gets a usable slack (the max(1.0) floor), so a
/// hold on a single-value range is breakable by genuinely different content but not by
/// jitter around the same value.
#[test]
fn degenerate_held_windows_use_the_slack_floor() {
    use super::norm_hold_break;
    assert!(!norm_hold_break((100.0, 100.0), (99.0, 103.0)));
    assert!(norm_hold_break((100.0, 100.0), (105.0, 110.0)));
}
