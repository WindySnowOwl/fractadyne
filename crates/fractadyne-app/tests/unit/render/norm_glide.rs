use super::*;

/// The user's colouring in the 2026-10-25 report: cycle 0.52, linear.
const SHAPE: NormShape = NormShape { sweeps: 0.5 + 0.52 * 6.0, log_palette: false };
const FRAME: f32 = 1.0 / 60.0;

fn pace(dt_s: f32, can_cut: bool) -> NormPace {
    NormPace { dt_s, shape: SHAPE, can_cut }
}

#[test]
fn the_first_window_and_a_snap_arrive_outright() {
    assert_eq!(norm_glide_step(None, (100.0, 200.0), false, pace(FRAME, true)), (100.0, 200.0));
    // A settled view deciding its mapping, or the hold-break recovery, arrives outright (and the
    // drawn map's change is dissolved, not cut: `palette_fade_step`).
    assert_eq!(norm_glide_step(Some((1.0, 2.0)), (900.0, 1000.0), true, pace(FRAME, true)), (900.0, 1000.0));
}

#[test]
fn a_small_step_glides_no_faster_than_the_eye_can_follow_and_arrives() {
    // A reading lands with the window 6% lower and 10% wider: under a cycle of colour change at the
    // edges, so it is glided, never cut.
    let (mut shown, target) = ((1000.0_f32, 2000.0_f32), (1060.0_f32, 2160.0_f32));
    assert!(window_change(shown, target, SHAPE) < GLIDE_CUT);
    let mut frames = 0;
    while shown != target {
        let next = norm_glide_step(Some(shown), target, false, pace(FRAME, true));
        let moved = window_change(shown, next, SHAPE);
        assert!(moved <= PALETTE_MAX_SPEED * FRAME * 1.001 + 1e-6, "frame {frames}: {moved} cycles");
        shown = next;
        frames += 1;
        assert!(frames < 240, "still gliding after 4 s: {shown:?}");
    }
    // It took a moment, not a frame: the eye sees a drift.
    assert!(frames > 10, "{frames} frames");
}

#[test]
fn a_far_target_is_cut_for_the_dissolve_or_glided_at_the_speed_limit_while_one_runs() {
    // The 2026-10-25 strobe: the top leapt 4,650 -> 26,781 in one reading. Swept, it moved every
    // colour through palette cycles at 0.2 a frame; now it is cut, and the drawn map dissolves.
    let shown = (2020.0_f32, 4650.0_f32);
    let target = (2044.0_f32, 26_781.0_f32);
    assert!(window_change(shown, target, SHAPE) > GLIDE_CUT);
    assert_eq!(norm_glide_step(Some(shown), target, false, pace(FRAME, true)), target);
    // While a dissolve is still under way no cut is taken (it would cut the dissolve short): the
    // window glides at the speed limit instead.
    let next = norm_glide_step(Some(shown), target, false, pace(FRAME, false));
    let moved = window_change(shown, next, SHAPE);
    assert!(moved > 0.0 && moved <= PALETTE_MAX_SPEED * FRAME * 1.001 + 1e-6, "{moved}");
}

#[test]
fn a_zooms_own_drift_is_tracked_closely_without_cuts() {
    // The user's dive before the minibrot: the window grew ~8% a second (534..978 to 1,595..3,339
    // over 14 s). Tracked at 60 fps it lags by a few percent of its width and is never cut.
    let (mut shown, mut target) = ((534.0_f32, 978.0_f32), (534.0_f32, 978.0_f32));
    let g = 1.08_f32.powf(FRAME);
    for _ in 0..600 {
        target = (target.0 * g, target.1 * g);
        let gap = window_change(shown, target, SHAPE);
        assert!(gap < GLIDE_CUT, "a cut on ordinary drift: {gap} cycles");
        shown = norm_glide_step(Some(shown), target, false, pace(FRAME, true));
    }
    let lag = (target.1 - shown.1) / (shown.1 - shown.0);
    assert!(lag < 0.05, "the shown window lags the target by {lag:.3} of its width");
}

#[test]
fn the_glide_is_paced_by_time_not_frames() {
    // The same second at 30 and at 120 frames per second ends in nearly the same place.
    let (from, target) = ((1000.0_f32, 2000.0_f32), (1050.0_f32, 2150.0_f32));
    let run = |fps: f32| {
        let mut s = from;
        for _ in 0..(fps as usize / 4) {
            s = norm_glide_step(Some(s), target, false, pace(1.0 / fps, true));
        }
        s
    };
    let (a, b) = (run(30.0), run(120.0));
    assert!((a.1 - b.1).abs() < 0.02 * (b.1 - b.0), "{a:?} vs {b:?}");
    assert!(a.1 > from.1 && a.1 < target.1, "a quarter second in, part of the way: {a:?}");
}

#[test]
fn another_location_is_re_acquired_but_a_zoom_s_own_drift_is_not() {
    // A jump to another location (the e10000 Misiurewicz case) is taken outright, cut or not.
    assert!(norm_glide_is_reacquire((6.0, 191.0), (181_573.0, 182_297.0)));
    assert_eq!(
        norm_glide_step(Some((6.0, 191.0)), (181_573.0, 182_297.0), false, pace(FRAME, false)),
        (181_573.0, 182_297.0)
    );
    // ⛔But a zoom's ORDINARY drift is not another location: a factor of two is the same place.
    assert!(!norm_glide_is_reacquire((100.0, 250.0), (200.0, 500.0)));
    assert!(!norm_glide_is_reacquire((45_075.0, 63_736.0), (50_000.0, 70_000.0)));
    assert!(!norm_glide_is_reacquire((100.0, 250.0), (800.0, 1000.0)));
}

#[test]
fn the_glide_arrives_exactly_in_finite_frames() {
    // W6 (U58): the SSAA average's colour signature includes the shown window, so a glide that only
    // APPROACHES (an f32 ulp at a time) would hold the average off.
    let mut shown = (1000.0_f32, 2000.0_f32);
    let target = (1040.0, 2080.0);
    let mut frames = 0;
    while shown != target {
        shown = norm_glide_step(Some(shown), target, false, pace(FRAME, true));
        frames += 1;
        assert!(frames < 300, "still gliding after {frames} frames: {shown:?}");
    }
    // …and the final landing is invisible: under half a percent of the window.
    let near = (1039.5_f32, 2079.5_f32);
    assert_eq!(norm_glide_step(Some(near), target, false, pace(FRAME, true)), target);
}

#[test]
fn a_change_of_window_is_measured_in_palette_cycles_as_the_screen_shows_it() {
    // The phase is carried (`phase_carry_step`), so a window that only SLIDES moves no colour.
    let a = (1000.0_f32, 2000.0_f32);
    assert_eq!(window_change(a, a, SHAPE), 0.0);
    assert!(window_change(a, (1100.0, 2100.0), SHAPE) < 1e-3);
    // Widening the top of a 1,000..2,000 window to 2,200 recolours its ends about the new middle
    // (1,600): each by a tenth of the sweeps.
    let c = window_change(a, (1000.0, 2200.0), SHAPE);
    let at_ends = SHAPE.sweeps * 0.1;
    assert!((c - at_ends).abs() < 0.01, "{c} vs {at_ends}");
    // Log (a heavy-tailed range): moving the floor moves the colours just above it most, and the
    // measure sees that.
    let log = NormShape { sweeps: SHAPE.sweeps, log_palette: true };
    let floor_moved = window_change((10.0, 50_000.0), (20.0, 50_000.0), log);
    assert!(floor_moved > 0.5, "{floor_moved}");
}
