use super::*;

/// Every curve runs from 0 at the set to 1 at the shading's width, never falls, and is clamped
/// outside 0..=1; the keys and GPU numbers round-trip.
#[test]
fn every_edge_curve_runs_from_zero_to_one_and_never_falls() {
    for c in EdgeCurve::ALL {
        assert_eq!(c.apply(0.0), 0.0, "{c:?} at the set");
        assert_eq!(c.apply(1.0), 1.0, "{c:?} at the width");
        assert_eq!(c.apply(-1.0), 0.0, "{c:?} below 0");
        assert_eq!(c.apply(7.0), 1.0, "{c:?} past the width");
        let mut prev = 0.0;
        for i in 0..=100 {
            let y = c.apply(i as f64 / 100.0);
            assert!(y >= prev, "{c:?} falls at {i}%");
            assert!((0.0..=1.0).contains(&y), "{c:?} leaves 0..1 at {i}%");
            prev = y;
        }
        assert_eq!(EdgeCurve::parse_key(c.key()), Some(c));
        assert_eq!(EdgeCurve::from_gpu_id(c.gpu_id()), Some(c));
    }
    assert_eq!(EdgeCurve::parse_key("log"), None);
    // The GPU numbers are the list's order (the shader's `edge_curve_of` switch).
    let ids: Vec<u32> = EdgeCurve::ALL.iter().map(|c| c.gpu_id()).collect();
    assert_eq!(ids, (0..EdgeCurve::ALL.len() as u32).collect::<Vec<_>>());
}

/// The curves differ where they should: at the middle of the width, Square below Linear below
/// Square root; Smooth is Linear's 0.5 there; Hard is 0 until the width.
#[test]
fn the_curves_shape_the_line_as_named() {
    let half = |c: EdgeCurve| c.apply(0.5);
    assert!(half(EdgeCurve::Square) < half(EdgeCurve::Linear));
    assert!(half(EdgeCurve::Linear) < half(EdgeCurve::Sqrt));
    assert_eq!(half(EdgeCurve::Smooth), 0.5);
    assert!(EdgeCurve::Smooth.apply(0.1) < EdgeCurve::Linear.apply(0.1));
    assert_eq!(EdgeCurve::Hard.apply(0.999), 0.0);
}

/// The weight: the strength at the set, nothing at and past the width, the strength scaling it,
/// and a width of 0 read as the narrowest.
#[test]
fn the_edge_weight_is_the_strength_at_the_set_and_gone_at_the_width() {
    for c in EdgeCurve::ALL {
        assert_eq!(edge_weight(0.0, 2.0, 0.8, c), 0.8, "{c:?}");
        assert_eq!(edge_weight(2.0, 2.0, 1.0, c), 0.0, "{c:?}");
        assert_eq!(edge_weight(50.0, 2.0, 1.0, c), 0.0, "{c:?}");
        assert_eq!(edge_weight(0.5, 2.0, 0.0, c), 0.0, "{c:?} at no strength");
    }
    assert!((edge_weight(1.0, 2.0, 1.0, EdgeCurve::Linear) - 0.5).abs() < 1e-12);
    assert!((edge_weight(1.0, 2.0, 0.5, EdgeCurve::Linear) - 0.25).abs() < 1e-12);
    assert_eq!(edge_weight(0.01, 0.0, 1.0, EdgeCurve::Linear), 1.0 - 0.01 / EDGE_WIDTH_MIN as f64);
    assert_eq!(edge_weight(1.0, 0.0, 1.0, EdgeCurve::Linear), 0.0);
}
