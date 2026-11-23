//! `NormMap::one_iteration_on` (coloring plan C11): the shifted map colors a value as the map
//! colors the value one higher, under every curve the shader's `coord_under` takes (C2).

use super::NormMap;

/// The palette coordinate the shader gives escape value `v` under `m` (`coord_under` in
/// `mandelbrot.wgsl`, before the wrap): a curve map takes v to T(max(v − lo, 0)) (`transfer_of`),
/// a linear one leaves it as it is; then times the rate plus the offset. Written here from the
/// shader, not from `NormMap::coord`, so the two are checked against each other.
fn coord(m: NormMap, v: f32) -> f32 {
    let d = (v - m.lo).max(0.0);
    let pv = match m.mode {
        0 => v,
        1 => (d + 1.0).ln(),
        2 => d.sqrt(),
        3 => d.cbrt(),
        4 => d.sqrt().sqrt(),
        5 => ((d + 1.0).ln() + 1.0).ln(),
        6 => d * d,
        other => panic!("no curve {other}"),
    };
    pv * m.cycle + m.offset
}

const MAPS: [NormMap; 8] = [
    // The classic map: Cycle's rate, the offset.
    NormMap { cycle: 0.0202, offset: 0.35, mode: 0, lo: 0.0 },
    // A normalized linear map over a deep window.
    NormMap { cycle: 2.0e-4, offset: 0.1 - 45_075.0 * 2.0e-4, mode: 0, lo: 0.0 },
    // A normalized log map from a floor of 12.
    NormMap { cycle: 0.31, offset: 0.6, mode: 1, lo: 12.0 },
    // The other curves (C2) from the same floor: square root, cube root, fourth root, log-log,
    // square, each at the rate that spreads 2.12 sweeps over a window 200 wide.
    NormMap { cycle: 2.12 / 14.142, offset: 0.6, mode: 2, lo: 12.0 },
    NormMap { cycle: 2.12 / 5.848, offset: 0.6, mode: 3, lo: 12.0 },
    NormMap { cycle: 2.12 / 3.761, offset: 0.6, mode: 4, lo: 12.0 },
    NormMap { cycle: 2.12 / 1.857, offset: 0.6, mode: 5, lo: 12.0 },
    NormMap { cycle: 2.12 / 40_000.0, offset: 0.6, mode: 6, lo: 12.0 },
];

#[test]
fn one_iteration_on_colors_v_as_the_map_colors_v_plus_one() {
    for m in MAPS {
        for v in [0.0f32, 3.25, 11.5, 12.0, 12.7, 100.0, 211.0, 45_100.5] {
            let (shifted, plain) = (coord(m.one_iteration_on(), v), coord(m, v + 1.0));
            assert!((shifted - plain).abs() <= 1.0e-4 * plain.abs().max(1.0), "{m:?} at {v}: {shifted} against {plain}");
        }
    }
}

#[test]
fn the_shift_moves_the_colors() {
    // Anti-vacuity: a value the map spreads colors differently once shifted (by a cycle in the
    // linear maps, 2e-4 for the deep one, and a share of a cycle under the curves here).
    for m in MAPS {
        let v = m.lo + 20.0;
        assert!((coord(m.one_iteration_on(), v) - coord(m, v)).abs() > 1.0e-5, "{m:?}: the shift changed nothing");
    }
}

#[test]
fn the_cpu_coordinate_is_the_shaders() {
    // `NormMap::coord`, which the phase carry and the dissolve's jump test evaluate maps with,
    // against the shader's definition above, under every curve.
    for m in MAPS {
        for v in [0.0f32, 3.25, 11.5, 12.0, 12.7, 100.0, 211.0, 45_100.5] {
            let (cpu, gpu) = (m.coord(v as f64) as f32, coord(m, v));
            assert!((cpu - gpu).abs() <= 1.0e-4 * gpu.abs().max(1.0), "{m:?} at {v}: {cpu} against {gpu}");
        }
    }
}
