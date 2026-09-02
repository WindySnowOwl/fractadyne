use super::*;

/// The marker must follow the hand. Drag right and the view moves right; drag down and it
/// moves down - which in complex coordinates means y DECREASES, since screen +y is complex
/// -y. Getting this backwards produces a minimap that fights the user, and no screenshot or
/// render gate can see it.
/// The regression this pins: a chunked or tiled refinement in flight is BUSY, on its own,
/// whatever the iteration setting. It was gated behind `!auto_iter` on the theory that
/// auto-iter settles are sub-second - false at depth, where they run for many seconds and are
/// exactly when the cue is wanted (1e31x, refining 75.7%, no spinner; 2026-08-29).
#[test]
fn a_refinement_in_flight_is_busy_by_itself() {
    let busy = FractadyneApp::spinner_busy;
    assert!(busy(false, false, true, false), "a chunked refinement alone must be busy");
    assert!(busy(false, true, false, false), "a settle grid alone must be busy");
    assert!(busy(true, false, false, false), "a reference build alone must be busy");
    assert!(!busy(false, false, false, false), "nothing pending is not busy");
}

/// Tour playback re-invalidates the reference every keyframe, so the spinner would strobe
/// through a whole tour. Suppression there is deliberate, not an oversight to be tidied away.
#[test]
fn tour_playback_suppresses_the_spinner() {
    let busy = FractadyneApp::spinner_busy;
    for (r, t, c) in [(true, false, false), (false, true, false), (false, false, true)] {
        assert!(busy(r, t, c, false), "control: {r} {t} {c} should be busy when not touring");
        assert!(!busy(r, t, c, true), "touring must suppress {r} {t} {c}");
    }
}

#[test]
fn minimap_drag_signs() {
    let size = egui::vec2(196.0, 147.0);

    let (dx, dy) = FractadyneApp::minimap_drag_to_complex(egui::vec2(10.0, 0.0), size);
    assert!(dx > 0.0, "dragging right must increase x, got {dx}");
    assert_eq!(dy, 0.0);

    let (dx, dy) = FractadyneApp::minimap_drag_to_complex(egui::vec2(0.0, 10.0), size);
    assert_eq!(dx, 0.0);
    assert!(dy < 0.0, "dragging down must decrease y, got {dy}");

    // Scale: a drag across the whole map is the map's whole width in complex units.
    let (full, _) = FractadyneApp::minimap_drag_to_complex(egui::vec2(size.x, 0.0), size);
    assert!(
        (full - 2.0 * MINIMAP_HX).abs() < 1.0e-12,
        "a full-width drag should span the map: {full} vs {}",
        2.0 * MINIMAP_HX
    );
}
