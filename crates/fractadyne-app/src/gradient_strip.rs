//! Tests for the gradient-strip geometry — the testable half of the P3′ stop strip.
//!
//! ⚠⚠**No harness in this repo drives hover, drag or scroll**, so the widget as a whole rests on
//! the author's eye. What is *not* eye-only is which marker a pointer names, which segment a click
//! names, and where the selection lands after a delete — the three things that are silently wrong
//! rather than visibly wrong. `design/gradient-curves.md` §9.5.

use super::{
    nudge_step, pick_segment, pick_stop, pick_stop_ring, ring_point, ring_pos, sel_after_remove,
    on_ring_track, segment_for_stop, strip_pos, strip_x, wrapped_distance, EDITOR_MAX_STOPS,
};

/// The strip the editor actually draws: a ~490 px content width inside the 520 px window.
const X0: f32 = 12.0;
const W: f32 = 490.0;

/// egui's pointer travel before a press becomes a drag.
///
/// ⭐⭐**Why the editor hit-tests the PRESS ORIGIN.** `Response::drag_started()` does not fire
/// until the pointer has moved at least this far, so hit-testing at `interact_pointer_pos()` in
/// that frame asks about a point that is *somewhere else*. ⚠⚠**And this is only the LOWER bound**:
/// the drift is however far the pointer travelled in the frame that crossed it, so an ordinary
/// flick — tens of pixels in one 60 Hz frame — lands arbitrarily far away.
///
/// ⚠It lives here, in the tests, because that is the only place it is *used*: it mirrors a
/// number egui owns, and reasoning about the size of the gap is a test's job. The production
/// consequence is recorded on `ColoringConfig::drag_stop`.
const DRAG_THRESHOLD_PX: f32 = 6.0;

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

/// ⭐⭐**The reported bug, pinned as arithmetic.** "The drag points sometimes hang up where they
/// stop moving": `Response::drag_started()` does not fire until the pointer has already travelled
/// egui's drag threshold, so hit-testing at `interact_pointer_pos()` in that frame asks about a
/// point ~6 px from where the user pressed. At the 32-stop cap that is enough to name the WRONG
/// marker, or — landing mid-gap — no marker at all, which cancels the drag silently and leaves the
/// stop exactly where it was. The editor therefore hit-tests the PRESS ORIGIN.
///
/// ⚠This is a test about a gap between two positions, not about egui: it shows that the answer
/// changes, which is the fact that makes using the wrong one a bug.
///
/// ⚠⚠**The first version of this test asserted the wrong premise** — that the 6 px threshold alone
/// carries the pointer out of the marker's 9 px catch zone. It does not (15.8 − 9 = 6.8 > 6), and
/// the assertion failed, which is the useful part: **the threshold is a LOWER bound, not the
/// drift.** The drift is however far the pointer moved in the frame that crossed it, so a flick is
/// unbounded. That is the real mechanism, and it is why the fix cannot be "widen the catch radius".
#[test]
fn the_drag_threshold_re_targets_the_marker_so_the_press_origin_is_the_one_to_test() {
    let n = EDITOR_MAX_STOPS;
    let positions: Vec<f32> = (0..n).map(|i| i as f32 / (n - 1) as f32).collect();
    let spacing = W / (n - 1) as f32;
    let radius = 9.0_f32;
    assert!(
        spacing * 0.55 > DRAG_THRESHOLD_PX,
        "vacuous unless the probed drift ({} px) is past the threshold ({DRAG_THRESHOLD_PX} px) — \
         i.e. actually reachable by the time `drag_started` fires",
        spacing * 0.55
    );
    let press = strip_x(positions[5], X0, W);
    assert_eq!(pick_stop(&positions, press, X0, W, radius), Some(5), "the press names marker 5");

    // Drifting toward the next marker: past the halfway point the nearest marker is 6, so a
    // hit-test after the threshold grabs the neighbour.
    let drifted = press + spacing * 0.55;
    assert_eq!(
        pick_stop(&positions, drifted, X0, W, radius),
        Some(6),
        "post-threshold this names the WRONG marker — the drag would move a stop the user never \
         grabbed"
    );

    // ⭐⭐**The silent-cancel case needs FEW stops, not many** — and few stops is the common case.
    // At the 32-stop cap the catch zones overlap everywhere (spacing 15.8 < 2 × 9), so a drift
    // always names *something*, merely the wrong thing. On a 7-stop gradient — the shape actually
    // being edited in the bug report — the gaps are ~82 px, so most of the strip is dead band: a
    // flick lands on nothing, `drag_stop` is cleared, and the stop sits still for the whole
    // gesture with no feedback at all. That is the reported hang.
    let few: Vec<f32> = (0..7).map(|i| i as f32 / 6.0).collect();
    let few_spacing = W / 6.0;
    assert!(
        few_spacing * 0.5 > radius,
        "vacuous unless a {few_spacing} px gap actually leaves a dead band outside both radii"
    );
    let press = strip_x(few[2], X0, W);
    assert_eq!(pick_stop(&few, press, X0, W, radius), Some(2), "the press names stop 2");
    assert_eq!(
        pick_stop(&few, press + few_spacing * 0.5, X0, W, radius),
        None,
        "post-threshold this names NOTHING — the drag is cancelled and the stop never moves"
    );
    // Even a modest one-frame flick is already past every marker's zone.
    assert_eq!(pick_stop(&few, press + 20.0, X0, W, radius), None, "a 20 px flick lands nowhere");
}

#[test]
fn ring_positions_and_points_round_trip_with_the_seam_at_twelve_oclock() {
    let (cx, cy, r) = (200.0_f32, 150.0_f32, 80.0_f32);
    for i in 0..64 {
        let p = i as f32 / 64.0;
        let (x, y) = ring_point(cx, cy, r, p);
        let back = ring_pos(cx, cy, x, y);
        assert!(wrapped_distance(back, p) < 1e-4, "{p} → ({x},{y}) → {back}");
    }
    // ⭐The seam — where a cycled palette jumps from 1.0 back to 0.0 — sits at the TOP, which is
    // the whole reason to offer the ring at all.
    let (x, y) = ring_point(cx, cy, r, 0.0);
    assert!((x - cx).abs() < 1e-3 && y < cy, "position 0 must be straight up, got ({x},{y})");
    // Clockwise: a quarter turn is to the RIGHT, not the left.
    let (x, y) = ring_point(cx, cy, r, 0.25);
    assert!(x > cx && (y - cy).abs() < 1e-3, "0.25 must be at 3 o'clock, got ({x},{y})");
    assert!((ring_pos(cx, cy, cx + r, cy) - 0.25).abs() < 1e-4);
    assert!((ring_pos(cx, cy, cx, cy + r) - 0.5).abs() < 1e-4, "0.5 is 6 o'clock");
    // Degenerate: the centre has no angle and must not produce a NaN position.
    assert_eq!(ring_pos(cx, cy, cx, cy), 0.0);
    assert!(ring_pos(cx, cy, cx, cy).is_finite());
}

/// ⭐⭐**A ring has no ends.** The stop nearest the seam must be reachable from BOTH sides, which
/// a linear distance cannot do — it reports 0.99 and 0.01 as 98 hundredths apart.
#[test]
fn ring_hit_testing_wraps_across_the_seam() {
    assert!((wrapped_distance(0.99, 0.01) - 0.02).abs() < 1e-6);
    assert!((wrapped_distance(0.01, 0.99) - 0.02).abs() < 1e-6, "symmetric");
    assert!((wrapped_distance(0.0, 0.5) - 0.5).abs() < 1e-6, "the far side is half a turn");

    let (cx, cy, r) = (200.0_f32, 150.0_f32, 80.0_f32);
    let positions = [0.0, 0.3, 0.62, 0.97];
    // Just anticlockwise of the seam: nearest is the 0.97 stop. ⚠A LINEAR distance ranks 0.0 as
    // 0.98 away here and would hand back stop 3 as well — the wrap only shows on the other side.
    let (x, y) = ring_point(cx, cy, r, 0.98);
    assert_eq!(pick_stop_ring(&positions, cx, cy, r, x, y, 12.0), Some(3));
    // Just clockwise of the seam: 0.0 wins, and the 0.97 stop must still be REACHABLE from here —
    // a linear distance puts it 0.974 away, outside any sane radius, so it becomes unclickable
    // from this side. That asymmetry is exactly how the bug hides.
    let (x, y) = ring_point(cx, cy, r, 0.004);
    assert_eq!(pick_stop_ring(&positions, cx, cy, r, x, y, 12.0), Some(0));
    // ⭐**The discriminating probe.** At 0.995 the wrapped distances are 0.005 to stop 0 (across
    // the seam) and 0.025 to stop 3 — so stop 0 wins. A LINEAR metric ranks stop 0 as 0.995 away
    // and hands back stop 3, i.e. the stop at the seam becomes unreachable from the near side.
    let (x, y) = ring_point(cx, cy, r, 0.995);
    assert!(
        wrapped_distance(0.995, 0.0) < wrapped_distance(0.995, 0.97),
        "the probe must actually be nearer the seam stop, or this proves nothing"
    );
    assert_eq!(pick_stop_ring(&positions, cx, cy, r, x, y, 12.0), Some(0));
    // Out in open arc, nothing is grabbed — which is what lets a click there mean "insert here".
    let (x, y) = ring_point(cx, cy, r, 0.45);
    assert_eq!(pick_stop_ring(&positions, cx, cy, r, x, y, 12.0), None);
    // ⚠The catch zone is ARC LENGTH, so the same 12 px covers a much LARGER slice of a small ring:
    // 0.02 of a turn is ~10 px at r=80 (a hit) and ~2.5 px at r=20 (a hit by a wide margin).
    // Getting this backwards — treating the radius as an angle — would make a small ring
    // unusable and a large one over-grabby.
    let (x, y) = ring_point(cx, cy, r, 0.32);
    assert_eq!(pick_stop_ring(&positions, cx, cy, r, x, y, 12.0), Some(1), "0.02 turn ≈ 10 px");
    let (x, y) = ring_point(cx, cy, 20.0, 0.36);
    assert_eq!(
        pick_stop_ring(&positions, cx, cy, 20.0, x, y, 12.0),
        Some(1),
        "0.06 of a turn is only ~7.5 px of arc on a 20 px ring, so it is still a hit"
    );
    assert_eq!(
        pick_stop_ring(&positions, cx, cy, r, ring_point(cx, cy, r, 0.36).0, ring_point(cx, cy, r, 0.36).1, 12.0),
        None,
        "the SAME angle on the big ring is ~30 px of arc — a miss. Angle alone cannot decide this."
    );
}

/// ⭐⭐**The ring's add track has to be a LINE, not a disc.** A radius test (`d <= r`) reads as
/// "anywhere inside the track", which on this layout means the entire colour wheel *and* both
/// buttons in the hole — so every click on the gradient would insert a stop. The band test is what
/// makes "click the track to add here" mean only the track.
#[test]
fn the_ring_add_track_is_a_band_not_a_disc() {
    let (cx, cy, track) = (200.0_f32, 150.0_f32, 100.0_f32);
    let tol = 8.0_f32;
    // On the line, from several angles — a click anywhere round it must add a stop there.
    for i in 0..8 {
        let (x, y) = ring_point(cx, cy, track, i as f32 / 8.0);
        assert!(on_ring_track(cx, cy, track, x, y, tol), "dead on the track at turn {i}/8");
    }
    // Just inside and just outside are still the track — it is a click target, not a hairline.
    let (x, y) = ring_point(cx, cy, track - tol + 0.5, 0.3);
    assert!(on_ring_track(cx, cy, track, x, y, tol));
    let (x, y) = ring_point(cx, cy, track + tol - 0.5, 0.3);
    assert!(on_ring_track(cx, cy, track, x, y, tol));

    // ⚠The cases a disc test gets wrong. The annulus the gradient is painted in, the empty hole
    // where the ⊕/⊖ buttons live, and the centre itself are all INSIDE the track radius and must
    // not count as clicks on it.
    let (x, y) = ring_point(cx, cy, 82.0, 0.4);
    assert!(!on_ring_track(cx, cy, track, x, y, tol), "the gradient annulus is not the track");
    let (x, y) = ring_point(cx, cy, 40.0, 0.4);
    assert!(!on_ring_track(cx, cy, track, x, y, tol), "the hole is not the track");
    assert!(!on_ring_track(cx, cy, track, cx, cy, tol), "the centre is not the track");
    assert!(!on_ring_track(cx, cy, track, cx + 16.0, cy, tol), "the ⊕ button is not the track");
    // And well outside is a miss, so a click in the window's margin does nothing.
    let (x, y) = ring_point(cx, cy, track + 30.0, 0.4);
    assert!(!on_ring_track(cx, cy, track, x, y, tol));
}
