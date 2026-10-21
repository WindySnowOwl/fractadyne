use super::*;

/// The widest one-frame move of a window, as a fraction of its width, over `frames` frames.
fn walk(mut d: Option<DrawnNorm>, frames: usize, want: impl Fn(usize) -> (f32, f32), held: bool) -> (Option<DrawnNorm>, f32) {
    let mut worst = 0.0_f32;
    for k in 0..frames {
        let prev = d.map(|x| x.win);
        d = drawn_norm_step(d, Some(want(k)), held);
        if let (Some(p), Some(n)) = (prev, d.map(|x| x.win)) {
            let width = (p.1 - p.0).max(1.0);
            worst = worst.max(((n.0 - p.0).abs() + (n.1 - p.1).abs()) / width);
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
    // Live at [1000, 2000]; a worker frame goes into the hold wanting [1400, 2600]: no snap.
    let live = drawn_norm_step(None, Some((1000.0, 2000.0)), false);
    let (d, worst) = walk(live, 60, |_| (1400.0, 2600.0), true);
    assert!(worst <= 0.13, "a frame moved the drawn window {:.0}% of its width", worst * 100.0);
    let d = d.unwrap();
    assert_eq!(d.win, (1400.0, 2600.0), "and it arrives");
    assert!(d.tracking);
    // The first frame after the switch did move, but only part of the way.
    let first = drawn_norm_step(live, Some((1400.0, 2600.0)), true).unwrap().win;
    assert!(first.0 > 1000.0 && first.0 < 1400.0, "{first:?}");
}

#[test]
fn every_change_of_source_glides_back_and_forth() {
    // Held at [1400, 2600], tracking; the hold gives way to a live picture at [1000, 2000].
    let held = drawn_norm_step(None, Some((1400.0, 2600.0)), true);
    let first = drawn_norm_step(held, Some((1000.0, 2000.0)), false).unwrap();
    assert!(!first.tracking && first.win.0 > 1000.0, "the release glides: {first:?}");
    // A NEW held picture (another worker frame) is another source too, while the old one tracked.
    let (h1, _) = walk(None, 1, |_| (1400.0, 2600.0), true);
    let next = drawn_norm_step(h1, Some((1500.0, 2700.0)), true).unwrap();
    assert!(!next.tracking && next.win.0 < 1500.0, "{next:?}");
    // While live and tracking, the moving live window is followed, never glided.
    let l = drawn_norm_step(None, Some((1000.0, 2000.0)), false);
    assert_eq!(drawn_norm_step(l, Some((1020.0, 2040.0)), false).unwrap().win, (1020.0, 2040.0));
}

#[test]
fn a_jump_to_another_picture_snaps_and_classic_colours_draw_no_window() {
    // A window twenty-odd times another's is another place (`norm_glide_is_reacquire`): it is taken outright.
    let held = drawn_norm_step(None, Some((40.0, 60.0)), true);
    assert_eq!(drawn_norm_step(held, Some((40_000.0, 60_000.0)), false).unwrap().win, (40_000.0, 60_000.0));
    // Classic colouring: nothing to draw, and the next window starts from itself.
    assert_eq!(drawn_norm_step(held, None, false), None);
    assert_eq!(drawn_norm_step(None, Some((5.0, 9.0)), false).unwrap().win, (5.0, 9.0));
}
