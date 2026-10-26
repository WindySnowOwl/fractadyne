use super::{series_fits, series_log2_dc, SERIES_RADIUS_SLACK_LOG2};
use fractadyne_core::{add_floatexp, BigFloat, FloatExp, SpanMantissa};

const PREC: usize = 1856;
const W: f64 = 1440.0;
const H: f64 = 1102.0;
/// The 2026-10-26 Home glide's start: 1440 × 1102 px at 2^-1671.477 a pixel (2^1663.37).
const START_UPP_LOG2: f64 = -1671.477;

fn centre() -> [BigFloat; 2] {
    [BigFloat::from_f64(-0.75, PREC), BigFloat::from_f64(0.1, PREC)]
}

/// A view `px` pixels of 2^`upp_log2` from `centre()` (along the real axis), as `build_params`
/// carries it: the span's exponent and its mantissa under it, with `extra` more bits moved from the
/// exponent into the mantissa (`delta_exp` is a convention, not part of the geometry).
fn view(upp_log2: f64, extra: i32) -> (i32, SpanMantissa) {
    let span_x_log2 = W.log2() + upp_log2;
    let delta_exp = span_x_log2.floor() as i32 - extra;
    let x = (span_x_log2 - delta_exp as f64).exp2();
    (delta_exp, SpanMantissa::new(x, x * H / W))
}

fn offset(px: f64, upp_log2: f64) -> [BigFloat; 2] {
    let c = centre();
    [add_floatexp(&c[0], FloatExp::from_f64(px).mul_pow2(upp_log2), PREC), c[1].clone()]
}

fn asks(rp: &[BigFloat; 2], upp_log2: f64) -> f64 {
    let (e, m) = view(upp_log2, 0);
    series_log2_dc(&centre(), rp, e, PREC, m)
}

/// The frame that dispatched the pass which lost the RTX 3080 drew 2^1625.3 with the series its
/// reference brought from 2^1663.37: 38 octaves past the radius the series was walked for.
#[test]
fn the_glide_from_2_1663_asked_38_octaves_past_its_series() {
    let rp = centre();
    let built = asks(&rp, START_UPP_LOG2);
    // The half-diagonal of 1440 × 1102 px, 906.6 px.
    assert!((built - (START_UPP_LOG2 + (0.5 * (W * W + H * H).sqrt()).log2())).abs() < 1e-9, "{built}");
    let lethal = asks(&rp, START_UPP_LOG2 + (1663.37 - 1625.3));
    assert!((lethal - built - 38.07).abs() < 1e-9, "{}", lethal - built);
    assert!(!series_fits(built, lethal));
}

#[test]
fn a_zoom_out_inside_the_slack_keeps_it_and_past_it_refuses_it() {
    let rp = centre();
    let built = asks(&rp, START_UPP_LOG2);
    for (out, fits) in [(0.0, true), (0.99, true), (SERIES_RADIUS_SLACK_LOG2, true), (1.01, false), (4.0, false)] {
        assert_eq!(series_fits(built, asks(&rp, START_UPP_LOG2 + out)), fits, "{out} octaves out");
    }
}

#[test]
fn a_zoom_in_or_a_pan_inside_the_radius_keeps_it() {
    // Built with its reference 300 px off the centre: the offset is part of the radius.
    let rp = offset(300.0, START_UPP_LOG2);
    let built = asks(&rp, START_UPP_LOG2);
    assert!(built > asks(&centre(), START_UPP_LOG2));
    for zoom_in in [0.5, 10.0, 200.0] {
        assert!(series_fits(built, asks(&rp, START_UPP_LOG2 - zoom_in)), "{zoom_in} octaves in");
    }
    // The same view panned: the reference 700 px off fits (300 + 907 → 700 + 907: +0.41 octave);
    // a view width away (2000 px) asks 1.42 octaves more.
    assert!(series_fits(built, asks(&offset(700.0, START_UPP_LOG2), START_UPP_LOG2)));
    assert!(!series_fits(built, asks(&offset(2000.0, START_UPP_LOG2), START_UPP_LOG2)));
}

#[test]
fn the_radius_is_the_geometry_not_the_exponent_it_is_carried_in() {
    let rp = offset(300.0, START_UPP_LOG2);
    let (e, m) = view(START_UPP_LOG2, 0);
    let (e5, m5) = view(START_UPP_LOG2, 5);
    assert_eq!(e - e5, 5);
    let a = series_log2_dc(&centre(), &rp, e, PREC, m);
    let b = series_log2_dc(&centre(), &rp, e5, PREC, m5);
    assert!((a - b).abs() < 1e-9, "{a} vs {b}");
}

#[test]
fn no_radius_or_no_number_refuses_it() {
    // A series installed without its radius (`NEG_INFINITY`) is never applied.
    assert!(!series_fits(f64::NEG_INFINITY, -1661.0));
    assert!(!series_fits(-1661.0, f64::NAN));
    assert!(!series_fits(f64::NAN, -1661.0));
    assert!(!series_fits(-1661.0, f64::INFINITY));
}
