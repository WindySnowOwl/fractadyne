//! Design validation (design/custom-formulas.md §3): eight of the ten built-in formula families are
//! programs in the Fraktaler-3 / Kalles Fraktaler "hybrid" opcode model. For each, the opcode program
//! applied to every point of the core's own f64 orbit (`orbit_points`, the step shared with the
//! bignum reference) must reproduce the next point — the step itself, not two orbits compared after
//! chaos has amplified rounding. (Measured: bit for bit, all eight.) Phoenix reads the previous
//! iterate and Newton converges, so neither is an opcode program.

use fractadyne_core::{formula, orbit_points};

#[derive(Clone, Copy, Debug)]
enum Op {
    /// Save the current Z for a later `Mul`.
    Store,
    /// Z = Z².
    Sqr,
    /// Z = Z × stored.
    Mul,
    /// Re Z = |Re Z|.
    AbsX,
    /// Im Z = |Im Z|.
    AbsY,
    /// Re Z = −Re Z.
    NegX,
    /// Im Z = −Im Z.
    NegY,
}

/// One iteration: the ops in order, then `+ c` (implicit and last, as in the opcode model).
fn step(ops: &[Op], z: (f64, f64), c: (f64, f64)) -> (f64, f64) {
    let (mut x, mut y) = z;
    let (mut sx, mut sy) = (0.0, 0.0);
    for op in ops {
        match op {
            Op::Store => (sx, sy) = (x, y),
            Op::Sqr => (x, y) = (x * x - y * y, 2.0 * x * y),
            Op::Mul => (x, y) = (x * sx - y * sy, x * sy + y * sx),
            Op::AbsX => x = x.abs(),
            Op::AbsY => y = y.abs(),
            Op::NegX => x = -x,
            Op::NegY => y = -y,
        }
    }
    (x + c.0, y + c.1)
}

fn programs() -> Vec<(u32, &'static str, Vec<Op>)> {
    use Op::*;
    vec![
        (formula::MANDELBROT, "Mandelbrot", vec![Sqr]),
        (formula::MULTIBROT3, "Multibrot 3", vec![Store, Sqr, Mul]),
        (formula::MULTIBROT4, "Multibrot 4", vec![Sqr, Sqr]),
        (formula::MULTIBROT5, "Multibrot 5", vec![Store, Sqr, Sqr, Mul]),
        (formula::TRICORN, "Tricorn", vec![NegY, Sqr]),
        (formula::BURNING_SHIP, "Burning Ship", vec![AbsX, AbsY, Sqr]),
        (formula::CELTIC, "Celtic", vec![Sqr, AbsX]),
        (formula::BUFFALO, "Buffalo", vec![Sqr, AbsX, AbsY]),
    ]
}

/// A small deterministic generator (no dev-dependency needed).
fn lcg(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((*state >> 11) as f64) / ((1u64 << 53) as f64)
}

#[test]
fn eight_built_in_families_are_opcode_programs() {
    let mut seed = 0x5eed_u64;
    for (id, name, ops) in programs() {
        let (mut steps, mut worst) = (0usize, 0.0_f64);
        for trial in 0..2_000 {
            let c = (lcg(&mut seed) * 4.0 - 2.0, lcg(&mut seed) * 4.0 - 2.0);
            // Half Mandelbrot-style (z0 = 0), half Julia-style (z0 anywhere).
            let z0 = if trial % 2 == 0 { (0.0, 0.0) } else { (lcg(&mut seed) * 4.0 - 2.0, lcg(&mut seed) * 4.0 - 2.0) };
            let orbit = orbit_points(z0, c, id, 40, 1.0e8);
            for w in orbit.windows(2) {
                let got = step(&ops, w[0], c);
                let scale = w[1].0.abs().max(w[1].1.abs()).max(1.0);
                let err = (got.0 - w[1].0).abs().max((got.1 - w[1].1).abs()) / scale;
                worst = worst.max(err);
                steps += 1;
            }
        }
        assert!(steps > 10_000, "{name}: too few steps compared ({steps})");
        assert!(worst <= 4.0 * f64::EPSILON, "{name} (id {id}): worst relative step error {worst:e}");
        eprintln!("{name}: {steps} steps, worst relative error {worst:e}");
    }
}
