use super::*;

const SHAPE: NormShape = NormShape { sweeps: 0.5 + 0.52 * 6.0, log_palette: false };
const PACE: NormPace = NormPace { dt_s: 1.0 / 60.0, shape: SHAPE, can_cut: true };

/// The fastest one-frame change of colour (palette cycles) over `frames` frames.
fn walk(mut d: Option<DrawnNorm>, frames: usize, want: impl Fn(usize) -> (f32, f32), held: bool) -> (Option<DrawnNorm>, f32) {
    let mut worst = 0.0_f32;
    for k in 0..frames {
        let prev = d.map(|x| x.win);
        d = drawn_norm_step(d, Some(want(k)), held, PACE);
        if let (Some(p), Some(n)) = (prev, d.map(|x| x.win)) {
            worst = worst.max(window_change(p, n, SHAPE));
        }
    }
    (d, worst)
}

#[test]
fn a_live_view_draws_its_own_window_with_no_lag() {
    // The live window glides by itself (`norm_shown`); drawn, it is followed exactly.
    let (d, _) = walk(None, 30, |k| (1000.0 + 10.0 * k as f32, 2000.0 + 20.0 * k as f32), false);
    let d = d.unwrap();
    assert_eq!(d.win, (1290.0, 2580.0));
    assert!(d.tracking);
}

#[test]
fn a_held_picture_with_another_window_is_reached_by_gliding_then_tracked() {
    // Live at [1000, 2000]; a worker frame goes into the hold wanting [1040, 2100]: no snap.
    let live = drawn_norm_step(None, Some((1000.0, 2000.0)), false, PACE);
    let (d, worst) = walk(live, 120, |_| (1040.0, 2100.0), true);
    assert!(worst <= PALETTE_MAX_SPEED * PACE.dt_s * 1.001 + 1e-6, "a frame moved the colours {worst} cycles");
    let d = d.unwrap();
    assert_eq!(d.win, (1040.0, 2100.0), "and it arrives");
    assert!(d.tracking);
    // The first frame after the switch did move, but only part of the way.
    let first = drawn_norm_step(live, Some((1040.0, 2100.0)), true, PACE).unwrap().win;
    assert!(first.0 > 1000.0 && first.0 < 1040.0, "{first:?}");
}

#[test]
fn every_change_of_source_glides_back_and_forth() {
    // Held at [1040, 2100], tracking; the hold gives way to a live picture at [1000, 2000].
    let held = drawn_norm_step(None, Some((1040.0, 2100.0)), true, PACE);
    let first = drawn_norm_step(held, Some((1000.0, 2000.0)), false, PACE).unwrap();
    assert!(!first.tracking && first.win.0 > 1000.0, "the release glides: {first:?}");
    // A NEW held picture (another worker frame) is another source too, while the old one tracked.
    let (h1, _) = walk(None, 1, |_| (1040.0, 2100.0), true);
    let next = drawn_norm_step(h1, Some((1060.0, 2140.0)), true, PACE).unwrap();
    assert!(!next.tracking && next.win.0 < 1060.0, "{next:?}");
    // While live and tracking, the moving live window is followed, never glided.
    let l = drawn_norm_step(None, Some((1000.0, 2000.0)), false, PACE);
    assert_eq!(drawn_norm_step(l, Some((1020.0, 2040.0)), false, PACE).unwrap().win, (1020.0, 2040.0));
}

#[test]
fn a_far_window_or_another_picture_is_taken_outright_and_classic_colours_draw_no_window() {
    // A window twenty-odd times another's is another place (`norm_glide_is_reacquire`), and a held
    // picture whose window is more than a palette cycle away is cut: both are taken outright, and
    // the drawn map's change is dissolved downstream (`palette_fade_step`).
    let held = drawn_norm_step(None, Some((40.0, 60.0)), true, PACE);
    assert_eq!(drawn_norm_step(held, Some((40_000.0, 60_000.0)), false, PACE).unwrap().win, (40_000.0, 60_000.0));
    let live = drawn_norm_step(None, Some((1000.0, 2000.0)), false, PACE);
    assert_eq!(drawn_norm_step(live, Some((1000.0, 5000.0)), true, PACE).unwrap().win, (1000.0, 5000.0));
    // Classic colouring: nothing to draw, and the next window starts from itself.
    assert_eq!(drawn_norm_step(held, None, false, PACE), None);
    assert_eq!(drawn_norm_step(None, Some((5.0, 9.0)), false, PACE).unwrap().win, (5.0, 9.0));
}
