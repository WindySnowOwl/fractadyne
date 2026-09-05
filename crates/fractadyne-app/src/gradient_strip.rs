//! Tests for the gradient-strip geometry — the testable half of the P3′ stop strip.
//!
//! ⚠⚠**No harness in this repo drives hover, drag or scroll**, so the widget as a whole rests on
//! the author's eye. What is *not* eye-only is which marker a pointer names, which segment a click
//! names, and where the selection lands after a delete — the three things that are silently wrong
//! rather than visibly wrong. `design/gradient-curves.md` §9.5.

use super::{
    nudge_step, pick_segment, pick_stop, sel_after_remove, segment_for_stop, strip_pos, strip_x,
    EDITOR_MAX_STOPS,
};

/// The strip the editor actually draws: a ~490 px content width inside the 520 px window.
const X0: f32 = 12.0;
const W: f32 = 490.0;

#[test]
fn position_and_pixel_round_trip_and_clamp() {
    for i in 0..=100 {
        let p = i as f32 / 100.0;
        let back = strip_pos(strip_x(p, X0, W), X0, W);
        assert!((back - p).abs() < 1e-4, "{p} → {} → {back}", strip_x(p, X0, W));
    }
    // Outside the strip is the nearest end, never an out-of-range position: a drag that leaves the
    // window must pin the stop at 0 or 1, not send it to -0.3.
    assert_eq!(strip_pos(X0 - 200.0, X0, W), 0.0);
    assert_eq!(strip_pos(X0 + W + 200.0, X0, W), 1.0);
    assert_eq!(strip_x(-5.0, X0, W), X0);
    assert_eq!(strip_x(9.0, X0, W), X0 + W);
    // A degenerate strip (a collapsed window, a first frame before layout) must not divide by zero.
    assert_eq!(strip_pos(50.0, X0, 0.0), 0.0);
    assert_eq!(strip_x(0.5, X0, 0.0), X0);
    assert_eq!(strip_x(0.5, X0, -10.0), X0);
}

/// ⭐⭐**The reason `pick_stop` is nearest-wins and not first-in-range.** At the 32-stop cap the
/// markers are ~15.8 px apart and the catch radius is 9 px, so a band in the middle of every gap
/// is inside BOTH neighbours' radii. First-in-range names the left marker throughout that band,
/// which reads as the strip grabbing the wrong handle for a third of its width.
///
/// ⚠**The obvious version of this test is vacuous** — aiming at a marker's dead centre, or a few
/// pixels off it, leaves only one marker in range, so nearest and first-in-range agree and the
/// mutation survives. It was written that way first and passed against a deliberately broken
/// `pick_stop`. The probe has to land in the overlap band, and the assertions below pin that it
/// does rather than assuming it.
#[test]
fn the_nearest_marker_wins_inside_the_overlap_band() {
    let n = EDITOR_MAX_STOPS;
    let positions: Vec<f32> = (0..n).map(|i| i as f32 / (n - 1) as f32).collect();
    let spacing = W / (n - 1) as f32;
    let radius = 9.0_f32;
    // The probe sits just past halfway, so it is nearer the RIGHT marker while still inside the
    // LEFT one's radius. Both facts are asserted, or the test proves nothing.
    let off = spacing * 0.52;
    assert!(off <= radius, "probe at {off} px is outside the left marker's {radius} px radius");
    assert!(spacing - off <= radius, "probe is outside the right marker's radius");
    assert!(off > spacing - off, "probe must be NEARER the right marker");
    for i in 0..n - 1 {
        let x = strip_x(positions[i], X0, W) + off;
        assert_eq!(
            pick_stop(&positions, x, X0, W, radius),
            Some(i + 1),
            "overlap band between markers {i} and {}",
            i + 1
        );
    }
    // And every marker is still reachable by aiming at it.
    for (i, &p) in positions.iter().enumerate() {
        let x = strip_x(p, X0, W);
        assert_eq!(pick_stop(&positions, x, X0, W, radius), Some(i), "dead centre of marker {i}");
        assert_eq!(pick_stop(&positions, x - 3.0, X0, W, radius), Some(i), "3 px left of {i}");
        assert_eq!(pick_stop(&positions, x + 3.0, X0, W, radius), Some(i), "3 px right of {i}");
    }
}

#[test]
fn pick_stop_respects_the_radius_and_the_empty_case() {
    let positions = [0.0, 0.5, 1.0];
    let mid = strip_x(0.5, X0, W);
    assert_eq!(pick_stop(&positions, mid + 8.9, X0, W, 9.0), Some(1), "just inside the radius");
    // Halfway between two far-apart markers is a click on NOTHING, which is what lets a
    // double-click there mean "insert a stop here" instead of "grab the nearest one".
    assert_eq!(pick_stop(&positions, mid + 40.0, X0, W, 9.0), None);
    assert_eq!(pick_stop(&[], mid, X0, W, 9.0), None);
    // Exactly on the radius is a hit — a strict `<` would leave a one-pixel dead ring.
    let p0 = strip_x(0.0, X0, W);
    assert_eq!(pick_stop(&positions, p0 + 9.0, X0, W, 9.0), Some(0));
}

/// Coincident markers (which a hard edge can produce if a boundary is ever exposed twice) must
/// resolve to a stable index rather than to whichever happened to be iterated first.
#[test]
fn pick_stop_breaks_ties_toward_the_lower_index() {
    let positions = [0.25, 0.25, 0.75];
    assert_eq!(pick_stop(&positions, strip_x(0.25, X0, W), X0, W, 9.0), Some(0));
}

#[test]
fn pick_segment_covers_every_position_including_the_edges() {
    // 4 boundaries → 3 segments.
    let b = [0.0, 0.2, 0.7, 1.0];
    assert_eq!(pick_segment(&b, 0.0), Some(0));
    assert_eq!(pick_segment(&b, 0.1), Some(0));
    assert_eq!(pick_segment(&b, 0.2), Some(1), "a boundary belongs to the segment it OPENS");
    assert_eq!(pick_segment(&b, 0.5), Some(1));
    assert_eq!(pick_segment(&b, 0.7), Some(2));
    assert_eq!(pick_segment(&b, 1.0), Some(2), "the last boundary closes the last segment");
    // Past either end resolves to the nearest end segment: a click a pixel outside the ribbon
    // plainly means the cell it is next to, and `None` there would make that cell's edge dead.
    assert_eq!(pick_segment(&b, -0.4), Some(0));
    assert_eq!(pick_segment(&b, 1.4), Some(2));
    // Degenerate inputs have no segment to name.
    assert_eq!(pick_segment(&[], 0.5), None);
    assert_eq!(pick_segment(&[0.0], 0.5), None);
}

/// ⚠The bug this pins: deleting a stop BELOW the selection silently re-points it at a different
/// stop, and deleting the last one leaves it reading past the end.
#[test]
fn selection_survives_a_delete() {
    // 5 stops; delete stop 1 → the stop that was 3 is now 2, and a selection on it must follow.
    assert_eq!(sel_after_remove(3, 1, 4), 2, "a delete below the selection shifts it down");
    assert_eq!(sel_after_remove(1, 3, 4), 1, "a delete above the selection leaves it alone");
    assert_eq!(sel_after_remove(3, 3, 4), 3, "deleting the selection itself keeps the slot");
    // The clamp: the selection was the highest stop and the list just got shorter.
    assert_eq!(sel_after_remove(4, 4, 4), 3);
    assert_eq!(sel_after_remove(9, 0, 3), 2);
    assert_eq!(sel_after_remove(2, 0, 0), 0, "an emptied gradient has no stop to select");
}

#[test]
fn a_stop_selects_the_segment_to_its_right_except_the_last() {
    // 5 stops → 4 segments.
    assert_eq!(segment_for_stop(0, 4), 0);
    assert_eq!(segment_for_stop(3, 4), 3);
    assert_eq!(segment_for_stop(4, 4), 3, "the last stop has no segment to its right");
    assert_eq!(segment_for_stop(0, 0), 0, "no segments at all must not underflow");
}

#[test]
fn the_fine_nudge_is_finer_than_a_pixel_and_the_coarse_one_is_not() {
    let coarse = nudge_step(false) * W;
    let fine = nudge_step(true) * W;
    assert!(coarse > 2.0, "a coarse nudge that moves under 2 px reads as nothing happening");
    assert!(fine < 1.0, "the fine nudge exists to reach positions a drag cannot address");
}
