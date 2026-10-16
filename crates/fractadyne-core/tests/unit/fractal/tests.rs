use super::*;

/// The pre-refactor inline `f64` arms, kept here ONLY to cross-check the generic step against
/// what `orbit_points` used to compute — a direct bit-identity tripwire for the migration
/// (esp. Burning Ship + Multibrot 4/5, whose deep reference orbit has no golden of its own).
fn old_f64(formula: u32, zx: f64, zy: f64, cx: f64, cy: f64) -> (f64, f64) {
    let (sx, sy) = (zx * zx - zy * zy, 2.0 * zx * zy);
    match formula {
        formula::MULTIBROT3 => (sx * zx - sy * zy + cx, sx * zy + sy * zx + cy),
        formula::MULTIBROT4 => (sx * sx - sy * sy + cx, 2.0 * sx * sy + cy),
        formula::MULTIBROT5 => {
            let (qx, qy) = (sx * sx - sy * sy, 2.0 * sx * sy);
            (qx * zx - qy * zy + cx, qx * zy + qy * zx + cy)
        }
        formula::BURNING_SHIP => (sx + cx, sy.abs() + cy),
        formula::TRICORN => (sx + cx, -sy + cy),
        formula::CELTIC => (sx.abs() + cx, sy + cy),
        formula::BUFFALO => (sx.abs() + cx, sy.abs() + cy),
        _ => (sx + cx, sy + cy), // Mandelbrot
    }
}

#[test]
fn generic_step_matches_old_f64_bitwise() {
    let cases = [
        (0.30_f64, -0.70, -0.50, 0.10),
        (1.10, 0.20, 0.00, 0.00),
        (-0.90, 0.44, 0.20, -0.30),
        (0.001, -1.3, 0.7, 0.7),
    ];
    let families = [
        formula::MANDELBROT,
        formula::MULTIBROT3,
        formula::MULTIBROT4,
        formula::MULTIBROT5,
        formula::BURNING_SHIP,
        formula::TRICORN,
        formula::CELTIC,
        formula::BUFFALO,
    ];
    for &(zx, zy, cx, cy) in &cases {
        for f in families {
            let (gx, gy) = trait_step(f, &zx, &zy, &cx, &cy, ()).unwrap();
            let (ox, oy) = old_f64(f, zx, zy, cx, cy);
            assert_eq!(gx.to_bits(), ox.to_bits(), "formula {f} re @ ({zx},{zy})");
            assert_eq!(gy.to_bits(), oy.to_bits(), "formula {f} im @ ({zx},{zy})");
        }
    }
}
