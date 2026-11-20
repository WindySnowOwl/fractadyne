//! `NormMap::one_iteration_on` (coloring plan C11): the shifted map colors a value as the map
//! colors the value one higher, in both of the shader's modes (`coord_under` in `mandelbrot.wgsl`).

use super::NormMap;

/// The palette coordinate the shader gives escape value `v` under `m` (`coord_under`, before the
/// wrap).
fn coord(m: NormMap, v: f32) -> f32 {
    let pv = if m.mode == 1 { ((v - m.lo).max(0.0) + 1.0).ln() } else { v };
    pv * m.cycle + m.offset
}

const MAPS: [NormMap; 3] = [
    // The classic map: Cycle's rate, the offset.
    NormMap { cycle: 0.0202, offset: 0.35, mode: 0, lo: 0.0 },
    // A normalized linear map over a deep window.
    NormMap { cycle: 2.0e-4, offset: 0.1 - 45_075.0 * 2.0e-4, mode: 0, lo: 0.0 },
    // A normalized log map from a floor of 12.
    NormMap { cycle: 0.31, offset: 0.6, mode: 1, lo: 12.0 },
];

#[test]
fn one_iteration_on_colors_v_as_the_map_colors_v_plus_one() {
    for m in MAPS {
        for v in [0.0f32, 3.25, 11.5, 12.0, 12.7, 100.0, 45_100.5] {
            let (shifted, plain) = (coord(m.one_iteration_on(), v), coord(m, v + 1.0));
            assert!((shifted - plain).abs() <= 1.0e-4 * plain.abs().max(1.0), "{m:?} at {v}: {shifted} against {plain}");
        }
    }
}

#[test]
fn the_shift_moves_the_colors() {
    // Anti-vacuity: a value the map spreads colors differently once shifted (by a cycle in the
    // linear maps, 2e-4 for the deep one, and about cycle/21 in the log map here).
    for m in MAPS {
        let v = m.lo + 20.0;
        assert!((coord(m.one_iteration_on(), v) - coord(m, v)).abs() > 1.0e-5, "{m:?}: the shift changed nothing");
    }
}
