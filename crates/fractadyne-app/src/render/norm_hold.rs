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
