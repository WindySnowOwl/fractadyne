//! Equalize colors' held distribution (coloring plan C6): fed under the palette window's own decision
//! (`eq_feed`) and shown through a glide (`eq_glide`).

use super::*;
use fractadyne_core::coloring::{Quantiles, EQ_QUANTILES};

/// A quantile function rising evenly from `lo` to `hi`.
fn ramp(lo: f32, hi: f32) -> Quantiles {
    std::array::from_fn(|i| lo + (hi - lo) * i as f32 / EQ_QUANTILES as f32)
}

#[test]
fn a_settled_view_adopts_a_moving_one_eases_in_and_a_hold_keeps() {
    let (a, b) = (ramp(1000.0, 2000.0), ramp(1100.0, 2400.0));
    // Nothing held: any reading is taken.
    let held = eq_feed(None, a, NormFeed::Chase, false).unwrap();
    assert_eq!(*held, a);
    // Adopt: outright.
    assert_eq!(*eq_feed(Some(held.clone()), b, NormFeed::Adopt, false).unwrap(), b);
    // Hold: kept, unless the hold is broken.
    assert_eq!(*eq_feed(Some(held.clone()), b, NormFeed::Hold, false).unwrap(), a);
    assert_eq!(*eq_feed(Some(held.clone()), b, NormFeed::Hold, true).unwrap(), b);
    // Chase: 0.3 of the way, knot by knot, and still never decreasing.
    let eased = eq_feed(Some(held), b, NormFeed::Chase, false).unwrap();
    for i in [0usize, 100, EQ_QUANTILES] {
        let want = a[i] + 0.3 * (b[i] - a[i]);
        assert!((eased[i] - want).abs() < 1.0e-3, "knot {i}: {} against {want}", eased[i]);
    }
    assert!(eased.windows(2).all(|w| w[0] <= w[1]));
}

#[test]
fn the_shown_distribution_glides_and_lands() {
    let (a, b) = (ramp(1000.0, 2000.0), ramp(1500.0, 2600.0));
    let frame = 1.0 / 60.0;
    // Nothing shown, or a snap: the fed one outright.
    assert_eq!(*eq_glide(None, Some(&b), false, frame).unwrap(), b);
    assert_eq!(*eq_glide(Some(Box::new(a)), Some(&b), true, frame).unwrap(), b);
    // Nothing fed: nothing shown (the classic map, as the window).
    assert!(eq_glide(Some(Box::new(a)), None, false, frame).is_none());
    // A frame's step: a share of the gap (the window's time constant), no more.
    let one = eq_glide(Some(Box::new(a)), Some(&b), false, frame).unwrap();
    let share = (one[0] - a[0]) / (b[0] - a[0]);
    let want = 1.0 - (-frame / GLIDE_TAU_S).exp();
    assert!((share - want).abs() < 1.0e-3, "{share} against {want}");
    // …and it lands, exactly, within a second or so.
    let mut q = Some(Box::new(a));
    let mut frames = 0;
    while q.as_deref() != Some(&b) {
        q = eq_glide(q, Some(&b), false, frame);
        frames += 1;
        assert!(frames < 120, "never landed");
    }
}
