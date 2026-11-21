use super::*;

/// Every curve is 0 at 0 (the floor keeps its color), rises, and its slope is its derivative.
#[test]
fn every_curve_starts_at_zero_rises_and_has_its_slope() {
    for t in Transfer::ALL {
        assert_eq!(t.apply(0.0), 0.0, "{t:?} at 0");
        assert_eq!(t.apply(-5.0), 0.0, "{t:?} below the floor is the floor");
        let mut prev = 0.0;
        for d in [0.5, 1.0, 10.0, 1.0e3, 1.0e5] {
            let y = t.apply(d);
            assert!(y > prev, "{t:?} does not rise at {d}");
            prev = y;
            // The slope against a central difference, 1e-4 relative.
            let h = d * 1.0e-6;
            let fd = (t.apply(d + h) - t.apply(d - h)) / (2.0 * h);
            assert!((t.slope(d) - fd).abs() <= 1.0e-4 * fd.abs(), "{t:?} slope at {d}: {} vs {fd}", t.slope(d));
        }
    }
}

/// The two curves frames were drawn with before C2 keep their formulas and their GPU numbers.
#[test]
fn linear_and_log_are_the_maps_they_always_were() {
    assert_eq!((Transfer::Linear.gpu_id(), Transfer::Log.gpu_id()), (0, 1));
    assert_eq!(Transfer::Log.apply(99.0), 100f64.ln());
    assert_eq!(Transfer::Linear.apply(37.5), 37.5);
    assert_eq!(Transfer::default(), Transfer::Linear);
}

/// Keys and GPU numbers name each curve and wrap once, and read back.
#[test]
fn keys_and_numbers_round_trip() {
    let mut ids: Vec<u32> = Transfer::ALL.iter().map(|t| t.gpu_id()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), Transfer::ALL.len());
    for t in Transfer::ALL {
        assert_eq!(Transfer::parse_key(t.key()), Some(t));
        assert_eq!(Transfer::from_gpu_id(t.gpu_id()), Some(t));
    }
    for w in Wrap::ALL {
        assert_eq!(Wrap::parse_key(w.key()), Some(w));
        assert_eq!(Wrap::from_gpu_id(w.gpu_id()), Some(w));
    }
    assert_eq!(Transfer::parse_key("Log"), None, "keys are exact");
}

/// The fold of a palette position: Repeat by its fraction, Mirror back and forth, Clamp held; steps
/// before the fold, each step the color at its start.
#[test]
fn a_position_folds_as_each_wrap_says() {
    let close = |a: f64, b: f64| (a - b).abs() < 1.0e-12;
    assert!(close(wrap_coord(2.25, Wrap::Repeat, 0), 0.25));
    assert!(close(wrap_coord(-0.25, Wrap::Repeat, 0), 0.75));
    assert!(close(wrap_coord(0.25, Wrap::Mirror, 0), 0.25));
    assert!(close(wrap_coord(1.25, Wrap::Mirror, 0), 0.75), "the second cycle runs back");
    assert!(close(wrap_coord(2.25, Wrap::Mirror, 0), 0.25));
    assert!(close(wrap_coord(-0.25, Wrap::Mirror, 0), 0.25), "and below 0 too");
    assert!(close(wrap_coord(1.0, Wrap::Mirror, 0), 1.0), "the turn is the last color, not the first");
    assert!(close(wrap_coord(3.7, Wrap::Clamp, 0), 1.0));
    assert!(close(wrap_coord(-2.0, Wrap::Clamp, 0), 0.0));
    assert!(close(wrap_coord(0.37, Wrap::Clamp, 0), 0.37));
    assert!(close(wrap_coord(0.37, Wrap::Repeat, 4), 0.25), "4 steps: the band starting at 0.25");
    assert!(close(wrap_coord(1.62, Wrap::Repeat, 4), 0.5));
    assert!(close(wrap_coord(1.62, Wrap::Mirror, 4), 0.5), "stepped at 1.5, then folded");
}
