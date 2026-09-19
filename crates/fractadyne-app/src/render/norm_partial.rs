use super::{norm_glide_step, norm_reading_bound, norm_window_feed};

/// The readings of one field walk, as measured on the 2026-09-19 2e13 autopilot dive: every pass
/// reports the floor exactly and a top no higher than the walk has reached, and only the last pass
/// covers the whole ask.
const WALK: [((f32, f32), bool); 4] = [
    ((756.0, 1758.0), false),
    ((756.0, 3150.0), false),
    ((756.0, 6582.0), false),
    ((756.0, 10238.0), true),
];

#[test]
fn a_complete_reading_is_taken_as_it_is_and_may_narrow_the_window() {
    assert_eq!(norm_reading_bound((756.0, 8000.0), true, Some((740.0, 10238.0))), (756.0, 8000.0));
}

#[test]
fn a_mid_walk_reading_keeps_its_floor_and_can_only_raise_the_top() {
    assert_eq!(norm_reading_bound((757.0, 1758.0), false, Some((756.0, 10238.0))), (757.0, 10238.0));
    assert_eq!(norm_reading_bound((757.0, 12000.0), false, Some((756.0, 10238.0))), (757.0, 12000.0));
}

#[test]
fn with_nothing_fed_a_mid_walk_reading_is_the_best_there_is() {
    assert_eq!(norm_reading_bound((734.0, 766.0), false, None), (734.0, 766.0));
}

#[test]
fn a_moving_view_whose_walk_restarts_no_longer_breathes() {
    // Six walks of the recorded sawtooth fed through the motion path (EMA + per-frame glide, three
    // frames per reading), before and after the fix. What the eye sees is the SHOWN window; the
    // measure is how far its width swings once it has settled onto the walks.
    let run = |bounded: bool| {
        let (mut fed, mut shown): (Option<(f32, f32)>, Option<(f32, f32)>) = (Some(WALK[3].0), None);
        let mut widths = Vec::new();
        for walk in 0..6 {
            for &(reading, complete) in &WALK {
                let r = if bounded { norm_reading_bound(reading, complete, fed) } else { reading };
                fed = norm_window_feed(None, r, false, fed).1;
                for _ in 0..3 {
                    shown = Some(norm_glide_step(shown, fed.unwrap(), false));
                    if walk >= 2 {
                        widths.push(shown.unwrap().1 - shown.unwrap().0);
                    }
                }
            }
        }
        let (lo, hi) = widths.iter().fold((f32::MAX, f32::MIN), |(a, b), &w| (a.min(w), b.max(w)));
        hi / lo
    };
    let before = run(false);
    let after = run(true);
    // Modelled: a 1.20× swing of the whole palette's scale, repeating with every walk.
    assert!(before > 1.15, "the recorded sawtooth should reproduce the swing ({before:.3}×)");
    assert!(after < 1.001, "the window still swings {after:.4}× with partial readings bounded");
}
