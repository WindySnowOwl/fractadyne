use super::*;

#[test]
fn the_shown_palette_window_glides_instead_of_snapping() {
    // The first window has nothing to glide from.
    assert_eq!(norm_glide_step(None, (100.0, 200.0), false), (100.0, 200.0));
    // A settled view deciding its mapping, or the hold-break recovery, arrives outright.
    assert_eq!(norm_glide_step(Some((1.0, 2.0)), (900.0, 1000.0), true), (900.0, 1000.0));

    // A step in the target becomes a drift: no single frame moves the window more than a small
    // fraction of its own width, which is what "the colours bounce around" was measuring (one
    // frame moved it 109% of its width before this existed).
    let mut shown = (100.0_f32, 200.0_f32);
    let target = (300.0, 500.0);
    let mut worst = 0.0_f32;
    for _ in 0..200 {
        let next = norm_glide_step(Some(shown), target, false);
        let width = (shown.1 - shown.0).max(1.0);
        let moved = ((next.0 - shown.0).abs() + (next.1 - shown.1).abs()) / width;
        worst = worst.max(moved);
        shown = next;
    }
    assert!(worst <= 0.13, "a frame moved the window {:.0}% of its width", worst * 100.0);

    // And it ARRIVES — a glide that never gets there is just a slower wrong answer.
    assert!(
        (shown.0 - target.0).abs() < 1.0 && (shown.1 - target.1).abs() < 1.0,
        "never reached the target: {shown:?}"
    );

    // Tracking a MOVING target (a 4.0× zoom grows the escape range about 1% a frame) must not
    // fall behind by much, or the picture wears a mapping from half a second ago.
    let (mut shown, mut target) = ((100.0_f32, 200.0_f32), (100.0_f32, 200.0_f32));
    for _ in 0..600 {
        target = (target.0 * 1.0116, target.1 * 1.0116); // ~2× per second at 60 fps
        shown = norm_glide_step(Some(shown), target, false);
    }
    let lag = (target.1 - shown.1) / (shown.1 - shown.0).max(1.0);
    assert!(lag < 0.25, "the shown window lags the target by {lag:.2} of its width");
}

#[test]
fn another_location_is_re_acquired_but_a_zoom_s_own_drift_is_not() {
    // A jump to another location (the e10000 Misiurewicz case: a shallow range in use for a view
    // whose real escapes sit far above it) snaps — gliding would show a mapping known to be
    // wrong, which under the log form is one flat colour.
    assert!(norm_glide_is_reacquire((6.0, 191.0), (181_573.0, 182_297.0)));
    assert_eq!(norm_glide_step(Some((6.0, 191.0)), (181_573.0, 182_297.0), false), (181_573.0, 182_297.0));

    // ⛔But a zoom's ORDINARY drift must not: measured at shallow depth, the window is ~150
    // iterations wide and the range doubles over a few octaves. Judging that by "disjoint by four
    // window widths" snapped on nearly every reading and made the worst frame WORSE than no glide
    // (449% of the window). A factor of two is the same location, still drifting.
    assert!(!norm_glide_is_reacquire((100.0, 250.0), (200.0, 500.0)));
    assert!(!norm_glide_is_reacquire((45_075.0, 63_736.0), (50_000.0, 70_000.0)));
    // Four window widths away, but only a factor of ~2.6 — a zoom, not a jump.
    assert!(!norm_glide_is_reacquire((100.0, 250.0), (800.0, 1000.0)));

    let shown = (45_075.0_f32, 63_736.0_f32);
    let stepped = norm_glide_step(Some(shown), (50_000.0, 70_000.0), false);
    assert!(stepped.0 > shown.0 && stepped.0 < 46_200.0, "glided too far: {stepped:?}");
    assert!(stepped.1 > shown.1 && stepped.1 < 65_000.0, "glided too far: {stepped:?}");
}
