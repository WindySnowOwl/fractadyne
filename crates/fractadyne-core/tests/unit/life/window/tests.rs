use super::*;

fn big(s: &str) -> BigFloat {
    crate::parse_bf_prec(s, 160).unwrap()
}

#[test]
fn floor_is_exact_to_the_plane_s_extent() {
    for (s, want) in [
        ("0", Some(0)),
        ("0.5", Some(0)),
        ("-0.5", Some(-1)),
        ("3.75", Some(3)),
        ("-3.75", Some(-4)),
        ("-4", Some(-4)),
        ("1099511627776.25", Some(1 << 40)),
        ("-1099511627776.25", Some(-(1 << 40) - 1)),
        // 2^60 + 0.25 has no f64: the fraction would be lost by any f64 path.
        ("1152921504606846976.25", Some(1 << 60)),
        ("-1152921504606846976.25", Some(-(1 << 60) - 1)),
        ("4611686018427387903", Some((1 << 62) - 1)),
        ("9223372036854775808", None),
    ] {
        assert_eq!(floor_i64(&big(s)), want, "{s}");
    }
}

fn view(cx: &str, cy: &str, upp: f64, w: f64, h: f64) -> Viewport {
    let mut vp = Viewport::new(w, h);
    vp.precision = 160;
    vp.center_x = big(cx);
    vp.center_y = big(cy);
    vp.units_per_pixel = FloatExp::from_f64(upp);
    vp
}

/// A 10 × 10 px view of 10 × 10 cells centred on cell (0, 0) starts at cell (−5, −5): tile (−1, −1),
/// 59.5 cells into it — rows growing downward (−y).
#[test]
fn a_view_near_the_origin() {
    let w = cell_window(&view("0.5", "-0.5", 1.0, 10.0, 10.0)).unwrap();
    assert_eq!((w.tile_x0, w.tile_y0), (-1, -1));
    assert_eq!(w.origin, [59.5, 59.5]);
    assert_eq!(w.cells_per_px, 1.0);
    // Row 3 of the file (y = −3.5 at its middle) is three rows below row 0.
    let w = cell_window(&view("0.5", "-3.5", 1.0, 2.0, 2.0)).unwrap();
    assert_eq!((w.tile_y0, w.origin[1]), (0, 2.5));
}

/// 2^60 cells out, the corner's sub-cell offset is still exact.
#[test]
fn a_view_far_out_keeps_its_fraction() {
    let w = cell_window(&view("1152921504606846976.25", "-1152921504606846976.25", 0.01, 100.0, 100.0)).unwrap();
    // left = 2^60 + 0.25 − 0.5 = 2^60 − 0.25 → cell 2^60 − 1, 0.75 in.
    assert_eq!(w.tile_x0, ((1i64 << 60) - 1).div_euclid(64));
    assert_eq!(w.origin[0], 63.75);
    assert_eq!(w.tile_y0, w.tile_x0);
    assert_eq!(w.origin[1], 63.75);
    assert!(cell_window(&view("9223372036854775808", "0", 1.0, 10.0, 10.0)).is_none());
}
