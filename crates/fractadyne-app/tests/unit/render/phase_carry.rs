use super::*;

const FRAME: f32 = 1.0 / 60.0;
/// The user's colouring in the 2026-10-25 report: cycle 0.52 is 3.62 sweeps across the window.
const SWEEPS: f32 = 0.5 + 0.52 * 6.0;

/// The normalised linear map of window `w` (`norm_map_for`), offset 0.0286 as the user had it.
fn map_of(w: (f32, f32)) -> NormMap {
    let c = SWEEPS / (w.1 - w.0);
    NormMap { cycle: c, offset: 0.0286 - w.0 * c, mode: 0, lo: 0.0 }
}

/// The palette coordinate of escape value `v` under `m` with `phase` added, as the shader takes it.
fn coord(m: &NormMap, phase: f32, v: f32) -> f64 {
    v as f64 * m.cycle as f64 + m.offset as f64 + phase as f64
}

/// The distance between two palette coordinates, a whole cycle being no distance at all.
fn wrapped(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(1.0);
    d.min(1.0 - d)
}

#[test]
fn a_sliding_window_keeps_every_colour_while_the_view_moves() {
    // A zoom slides the window up 2 iterations a frame, keeping its width: every value keeps its
    // colour, frame after frame, where the bare map rotates them all through the palette.
    let mut st = PhaseCarry::default();
    let mut prev: Option<(NormMap, f32)> = None;
    for k in 0..300 {
        let w = (1000.0 + 2.0 * k as f32, 2000.0 + 2.0 * k as f32);
        let m = map_of(w);
        let (s, phase) = phase_carry_step(st, m, Some(w), 7, false, FRAME);
        st = s;
        assert!(phase.abs() <= 0.5 + 1e-6, "{phase}");
        if let Some((pm, pp)) = prev {
            for v in [1200.0_f32, 1500.0, 1900.0, 2300.0] {
                let d = wrapped(coord(&m, phase, v), coord(&pm, pp, v));
                assert!(d < 1e-3, "frame {k}: value {v} moved {d} cycles");
            }
        }
        prev = Some((m, phase));
    }
    // Over those 300 frames the bare map rotated the palette by 3.62 × 600/1000 ≈ 2.2 cycles.
    let bare = coord(&map_of((1600.0, 2600.0)), 0.0, 1500.0) - coord(&map_of((1000.0, 2000.0)), 0.0, 1500.0);
    assert!(bare.abs() > 2.0, "{bare}");
}

#[test]
fn a_widening_window_recolours_about_its_middle() {
    let w0 = (1000.0_f32, 2000.0_f32);
    let (st, _) = phase_carry_step(PhaseCarry::default(), map_of(w0), Some(w0), 7, false, FRAME);
    // Wider by a tenth about its middle: the middle keeps its colour, the ends move.
    let w1 = (950.0_f32, 2050.0_f32);
    let (_, phase) = phase_carry_step(st, map_of(w1), Some(w1), 7, false, FRAME);
    let (m0, m1) = (map_of(w0), map_of(w1));
    assert!(wrapped(coord(&m1, phase, 1500.0), coord(&m0, 0.0, 1500.0)) < 1e-4);
    let end = wrapped(coord(&m1, phase, 2000.0), coord(&m0, 0.0, 2000.0));
    assert!((end - SWEEPS as f64 * 0.5 * (1.0 - 1.0 / 1.1)).abs() < 0.01, "{end}");
}

#[test]
fn a_view_at_rest_gives_the_phase_back_and_is_coloured_by_its_view_alone() {
    // Slide while moving, then stop: the carried phase falls to nothing at PHASE_SETTLE_SPEED,
    // and the map left is the view's own — the same however the view was reached.
    let mut st = PhaseCarry::default();
    let mut phase = 0.0;
    for k in 0..40 {
        let w = (1000.0 + 5.0 * k as f32, 2000.0 + 5.0 * k as f32);
        (st, phase) = phase_carry_step(st, map_of(w), Some(w), 7, false, FRAME);
    }
    assert!(phase.abs() > 0.05, "something was carried: {phase}");
    let rest = (1195.0_f32, 2195.0_f32);
    let mut frames = 0;
    let mut last = phase.abs();
    while phase != 0.0 {
        (st, phase) = phase_carry_step(st, map_of(rest), Some(rest), 7, true, FRAME);
        assert!(phase.abs() <= last + 1e-6, "it only shrinks");
        assert!(last - phase.abs() <= PHASE_SETTLE_SPEED * FRAME + 1e-6, "no faster than the settle speed");
        last = phase.abs();
        frames += 1;
        assert!(frames < 60, "half a cycle settles within a second");
    }
}

#[test]
fn classic_colours_a_change_of_kind_or_of_the_users_colouring_carry_nothing() {
    let w = (1000.0_f32, 2000.0_f32);
    let (st, _) = phase_carry_step(PhaseCarry::default(), map_of(w), Some(w), 7, false, FRAME);
    let w2 = (1100.0_f32, 2100.0_f32);
    let (moved, phase) = phase_carry_step(st, map_of(w2), Some(w2), 7, false, FRAME);
    assert!(phase != 0.0);
    // Classic colouring (no window): nothing carried, and the next window starts afresh.
    let (st2, p) = phase_carry_step(moved, map_of(w2), None, 7, false, FRAME);
    assert_eq!((p, st2), (0.0, PhaseCarry::default()));
    // The user's colouring changed: the map takes it at once.
    let (_, p) = phase_carry_step(moved, map_of(w), Some(w), 8, false, FRAME);
    assert_eq!(p, 0.0);
    // Linear to log: another kind of map, dissolved elsewhere; nothing carried across it.
    let log = NormMap { cycle: 0.4, offset: 0.0286, mode: 1, lo: 1000.0 };
    let (_, p) = phase_carry_step(moved, log, Some(w), 7, false, FRAME);
    assert_eq!(p, 0.0);
}
