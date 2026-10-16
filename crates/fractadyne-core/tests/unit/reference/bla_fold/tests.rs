use super::*;

/// Every fold family: the power-2 originals and the power families' folds.
const FOLDS: [u32; 16] = [
    formula::BURNING_SHIP,
    formula::TRICORN,
    formula::CELTIC,
    formula::BUFFALO,
    formula::BURNING_SHIP3,
    formula::BURNING_SHIP4,
    formula::BURNING_SHIP5,
    formula::TRICORN3,
    formula::TRICORN4,
    formula::TRICORN5,
    formula::CELTIC3,
    formula::CELTIC4,
    formula::CELTIC5,
    formula::BUFFALO3,
    formula::BUFFALO4,
    formula::BUFFALO5,
];

fn fe(v: f64) -> FloatExp {
    FloatExp::from_f64(v)
}

/// The spectral norm against a brute-force maximum of |M·v| over unit vectors, on a rotation-scale
/// (where the closed form's discriminant cancels), a reflection, a shear and a rank-one matrix.
#[test]
fn the_spectral_norm_is_exact_for_2x2() {
    for m in [
        [0.6, -0.8, 0.8, 0.6],
        [3.0, 4.0, 4.0, -3.0],
        [1.0, 7.0, 0.0, 1.0],
        [2.0, 4.0, 1.0, 2.0],
        [1e-30, -2e-30, 3e-30, 5e-31],
    ] {
        let mat = Mat2Fe([fe(m[0]), fe(m[1]), fe(m[2]), fe(m[3])]);
        let brute = (0..36_000)
            .map(|k| {
                let t = (k as f64) * std::f64::consts::TAU / 36_000.0;
                let (x, y) = (t.cos(), t.sin());
                ((m[0] * x + m[1] * y).powi(2) + (m[2] * x + m[3] * y).powi(2)).sqrt()
            })
            .fold(0.0f64, f64::max);
        let got = mat.norm().to_f64();
        assert!((got - brute).abs() <= 1e-6 * brute, "{m:?}: norm {got} vs {brute}");
        assert!(got >= brute * (1.0 - 1e-12), "{m:?}: the norm must bound every |M·v|");
    }
}

/// A one-sample "orbit" holding `z` exactly (f32-representable parts), for level-0 nodes.
fn orbit_at(z: (f32, f32)) -> Vec<[f32; 4]> {
    vec![[z.0, z.1, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0]]
}

/// Level 0 IS each fold's linearisation: within the node's radius, `A·δ + δc` is the exact
/// perturbed step to eps (1e-6) of the step's size, in every direction — and one notch past a
/// fold's radius, in the direction that crosses the fold, it is not. Reference values sit in every
/// quadrant, so every sign of every fold is taken.
#[test]
fn level_zero_is_each_folds_linearisation() {
    let eps = 1.0e-6;
    let dc = (3.0e-13, -2.0e-13);
    let dc_c = CFloatExp { re: fe(dc.0), im: fe(dc.1) };
    for f in FOLDS {
        let (shape, d) = fold_shape(f).unwrap();
        for z in [(0.375f32, 0.625f32), (-0.875, 0.25), (-0.5, -0.4375), (0.6875, -0.8125)] {
            let orbit = orbit_at(z);
            let node = fold_level0_node(0, &orbit, eps, AuxAggParams::default(), shape, d);
            let r = node.r.to_f64();
            assert!(r > 0.0, "formula {f} at {z:?}: zero radius off every axis");
            for k in 0..16 {
                let t = (k as f64) * std::f64::consts::TAU / 16.0;
                let e = (0.999 * r * t.cos(), 0.999 * r * t.sin());
                let lin = node.a.apply(CFloatExp { re: fe(e.0), im: fe(e.1) }) + dc_c;
                let exact = fold_pert_step(f, (z.0 as f64, z.1 as f64), e, dc);
                let (lx, ly) = (lin.re.to_f64(), lin.im.to_f64());
                let err = ((lx - exact.0).powi(2) + (ly - exact.1).powi(2)).sqrt();
                let size = (exact.0.powi(2) + exact.1.powi(2)).sqrt();
                assert!(
                    err <= 1.5 * eps * size,
                    "formula {f} at {z:?}, δ angle {t:.2}: linear {lx:e},{ly:e} vs exact {:e},{:e} (rel {:.2e})",
                    exact.0,
                    exact.1,
                    err / size
                );
            }
        }
    }
}

/// The fold's own radius, where it binds: a reference value (f32, as an orbit sample) within ~1e-7 of
/// the fold — y ≈ 0 for Burning Ship, Re W ≈ 0 (Re and Im for Buffalo) at arg z = π/(2d) for Celtic
/// and Buffalo. Elsewhere the power's radius (2·eps·|Z|/(d−1) ~ 1e-6) is the smaller, so a fold
/// radius that was dropped altogether passed every other test (mutation, 2026-10-01). Here: the
/// radius must be the fold's (well under the power's), the linearisation must hold inside it, and
/// twice past it, moved straight across the fold, it must fail.
#[test]
fn the_fold_radius_binds_next_to_a_fold() {
    let eps = 1.0e-6;
    for f in FOLDS {
        let (shape, d) = fold_shape(f).unwrap();
        if shape == Shape::Tricorn {
            continue; // nothing to cross
        }
        let df = f64::from(d);
        let z = match shape {
            Shape::BurningShip => (0.5f32, 3.0e-8f32),
            _ => {
                let t = std::f64::consts::PI / (2.0 * df) + 1.0e-7;
                ((0.7 * t.cos()) as f32, (0.7 * t.sin()) as f32)
            }
        };
        let (zx, zy) = (z.0 as f64, z.1 as f64);
        let node = fold_level0_node(0, &orbit_at(z), eps, AuxAggParams::default(), shape, d);
        let r_power = 2.0 * eps * (zx * zx + zy * zy).sqrt() / (df - 1.0);
        let r = node.r.to_f64();
        assert!(r < 0.5 * r_power, "formula {f}: the fold radius {r:e} does not bind (power's {r_power:e})");
        // The direction that moves the folded value straight across its axis.
        let across = match shape {
            Shape::BurningShip => (0.0, -1.0),
            _ => {
                // δw ≈ g·δ with g = d·Z^(d−1): δ = −sgn(Re W)/g moves Re W toward 0 and past it.
                let mul = |a: (f64, f64), b: (f64, f64)| (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0);
                let g = (0..d - 1).fold((df, 0.0), |a, _| mul(a, (zx, zy)));
                let w = (0..d).fold((1.0, 0.0), |a, _| mul(a, (zx, zy)));
                let s = -w.0.signum();
                let m2 = g.0 * g.0 + g.1 * g.1;
                let q = (s * g.0 / m2, -s * g.1 / m2);
                let n = (q.0 * q.0 + q.1 * q.1).sqrt();
                (q.0 / n, q.1 / n)
            }
        };
        let rel = |k: f64| {
            let e = (k * r * across.0, k * r * across.1);
            let lin = node.a.apply(CFloatExp { re: fe(e.0), im: fe(e.1) });
            let exact = fold_pert_step(f, (zx, zy), e, (0.0, 0.0));
            let (lx, ly) = (lin.re.to_f64(), lin.im.to_f64());
            ((lx - exact.0).powi(2) + (ly - exact.1).powi(2)).sqrt() / (exact.0.powi(2) + exact.1.powi(2)).sqrt()
        };
        assert!(rel(0.999) <= 1.5 * eps, "formula {f}: inside the fold radius, rel err {:.2e}", rel(0.999));
        assert!(rel(2.0) > 0.1, "formula {f}: twice past the fold radius the map is still linear ({:.2e})", rel(2.0));
    }
}

/// An interior reference per fold family, off every axis: a grid point whose f64 orbit stays bounded
/// for 3,000 steps with its last 500 samples clear of the folds (so the tree can skip there), and
/// whose orbit attracts: a neighbour 1e-9 away ends within 1e-6 of it. ⚠Bounded is not enough:
/// c = −1 − i is a Burning Ship FIXED point (−1 + i, exactly, so the bignum reference sits on it
/// forever) but a repelling one (×2√2 a step), and every perturbation of it overflows; and a fold's
/// bounded orbits can be chaotic (Burning Ship at −1 − 0.4i), where any approximation — the tree's
/// skips included — is amplified away from the exact steps.
fn interior_point(f: u32) -> Option<(f64, f64)> {
    let (shape, d) = fold_shape(f).unwrap();
    for i in 0..41 {
        for j in 0..41 {
            let c = (-1.0 + 0.05 * i as f64, -1.0 + 0.05 * j as f64);
            let pts = orbit_points((0.0, 0.0), c, f, 3000, 65536.0);
            if pts.len() <= 3000 {
                continue; // escaped
            }
            let near = orbit_points((0.0, 0.0), (c.0 + 1e-9, c.1 + 1e-9), f, 3000, 65536.0);
            let (a, b) = (pts[3000], *near.last().unwrap());
            if near.len() <= 3000 || ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt() > 1e-6 {
                continue; // repelling, or chaotic
            }
            let clear = pts[2500..].iter().all(|&(x, y)| {
                let w = (0..d).fold((1.0, 0.0), |a: (f64, f64), _| (a.0 * x - a.1 * y, a.0 * y + a.1 * x));
                let fold_gap = match shape {
                    Shape::BurningShip => x.abs().min(y.abs()),
                    Shape::Tricorn => 1.0,
                    Shape::Celtic => w.0.abs(),
                    _ => w.0.abs().min(w.1.abs()),
                };
                fold_gap > 1e-3 && x * x + y * y > 1e-6
            });
            if clear {
                return Some(c);
            }
        }
    }
    None
}

/// The tree's skips reproduce the exact perturbation at an interior reference for every fold family
/// while skipping most steps (B7).
#[test]
fn fold_bla_reproduces_exact_perturbation() {
    let p = 96;
    let target: u32 = 2000;
    let dc = (1.0e-9_f64, 0.5e-9_f64);
    let dc_c = CFloatExp { re: fe(dc.0), im: fe(dc.1) };
    for f in FOLDS {
        let c = interior_point(f).unwrap_or_else(|| panic!("formula {f}: no interior grid point off the folds"));
        let (orbit, len) = reference_orbit(&bf(0.0, p), &bf(0.0, p), &bf(c.0, p), &bf(c.1, p), f, target, p);
        assert!(len >= target, "formula {f} at {c:?}: reference escaped early (len={len})");
        let levels = build_bla_fold(&orbit, fe(1.2e-9), 1.0e-6, AuxAggParams::default(), f);
        let z_at = |m: u32| {
            let z = orbit[m as usize];
            (z[0] as f64 + z[2] as f64, z[1] as f64 + z[3] as f64)
        };
        let (mut dz, mut m, mut ops) = ((0.0f64, 0.0f64), 0u32, 0u32);
        while m < target {
            ops += 1;
            let dzc = CFloatExp { re: fe(dz.0), im: fe(dz.1) };
            let mut used = false;
            for l in (0..levels.len()).rev() {
                if (m & ((1u32 << l) - 1)) != 0 {
                    continue;
                }
                let Some(&node) = levels[l].get((m >> l) as usize) else { continue };
                if m + node.span > target || !dzc.abs().lt(node.r) {
                    continue;
                }
                let n = node.a.apply(dzc) + node.b.apply(dc_c);
                dz = (n.re.to_f64(), n.im.to_f64());
                m += node.span;
                used = true;
                break;
            }
            if !used {
                dz = fold_pert_step(f, z_at(m), dz, dc);
                m += 1;
            }
        }
        let mut e = (0.0f64, 0.0f64);
        for m in 0..target {
            e = fold_pert_step(f, z_at(m), e, dc);
        }
        let err = ((dz.0 - e.0).powi(2) + (dz.1 - e.1).powi(2)).sqrt();
        let mag = (e.0 * e.0 + e.1 * e.1).sqrt().max(1e-300);
        assert!(err / mag < 1.0e-3, "formula {f} at {c:?}: BLA vs exact rel err {:.2e} (ops={ops})", err / mag);
        assert!(ops < target / 4, "formula {f} at {c:?}: BLA didn't skip enough (ops={ops} of {target})");
    }
}

/// The GPU form keeps the complex tree's 16-float node: same count and position of every lane the
/// shader and `apply_bla_aux` read outside the two matrices.
#[test]
fn the_fold_tree_packs_as_the_complex_one() {
    let p = 96;
    let f = formula::BURNING_SHIP;
    let c = interior_point(f).unwrap();
    let (orbit, _) = reference_orbit(&bf(0.0, p), &bf(0.0, p), &bf(c.0, p), &bf(c.1, p), f, 300, p);
    let aux = AuxAggParams { trap_type: 0, stripe_freq: 3.0, cmag: 0.5, power: 2.0 };
    let fold = bla_fold_to_gpu(&build_bla_fold(&orbit, fe(1e-9), 1e-6, aux, f));
    let complex = bla_to_gpu(&build_bla(&orbit, fe(1e-9), 1e-6, aux, 2));
    assert_eq!(fold.len(), complex.len(), "the same level layout");
    for (i, (a, b)) in fold.iter().zip(&complex).enumerate() {
        if i % 4 == 3 {
            assert_eq!(a, b, "node {}: span and the aux aggregates", i / 4);
        }
    }
    assert_eq!(bla_tree_gpu(&orbit, fe(1e-9), 1e-6, aux, f), fold, "the dispatcher takes the fold tree");
    assert!(bla_tree_gpu(&orbit, fe(1e-9), 1e-6, aux, formula::PHOENIX).is_empty(), "no tree for Phoenix");
}

/// Deep points of the fold families, good to ~2^-112, whose 1e-30 neighbourhood is mostly STABLE: a
/// ±1e-35 nudge of δc leaves plain perturbation's count alone at most of a 5×5 grid. Each is a
/// bignum bisection of "escapes within N" (N = 2,000, or 500 for Celtic 5) from 0 to 2.5 along a
/// ray — rays every 1.875° from 180.9375°, never a symmetry axis — the first whose grid is ≥ 15 of 25
/// stable, ≥ 5 escaping, over ≥ 100 steps (or, for Tricorn and Burning Ship 4–5 and Tricorn 3–5,
/// from the first search: 7.5° rays from 183.75°, ≥ 300 steps of spread, which happened to be
/// stable). ⚠That first search alone gave NOISE for ten families — chaotic boundary points where not
/// one of 25 pixels survived the nudge, which the app draws as noise too. Celtic, Celtic 3, Buffalo 3
/// and Buffalo 5 met no ray at those bounds (192 rays, two budgets) and take the first with ≥ 10
/// stable, ≥ 3 escaping, over ≥ 20 steps.
pub(crate) const FOLD_DEEP_FIXTURES: [(u32, &str, &str); 16] = [
    (formula::CELTIC, "-7.523183301672266119158619702700220349889660622278404512203e-1", "-2.14835958763750446308384443362044395573177458962387293481e-1"),
    (formula::CELTIC3, "-1.390111097603047035481381903268406231192188759277730469272e-1", "-1.208364211005726539769261155232087101766648293103934130593e+0"),
    (formula::BUFFALO3, "4.608237148512231691928989032718950631004591668346324319821e-1", "1.154303315171477277006095095403463761887149114811342024179e-1"),
    (formula::BUFFALO5, "4.029273516218120856749052413703955216672657505935849857889e-1", "-5.821723262584372350783097339293223693418500863674671803413e-1"),
    (formula::BURNING_SHIP, "1.510426975372399155276004915912049480568178198670688953146e-1", "4.221360602684940155668847406639261668137033173647796363826e-1"),
    (formula::TRICORN, "-1.245181750030167260019164378722004235435653851940847122794e+0", "-2.037605720180463320107795585296334647328774720528326395773e-2"),
    (formula::BUFFALO, "-1.510426975372401197787866671960570444323379772291351660331e-1", "4.221360602684941728921263977360891137815433991908003217154e-1"),
    (formula::BURNING_SHIP3, "-9.15383227746129862363246806792022038019889900108861939843e-1", "-1.665596356636845051404128728242821066413195064087217571441e-1"),
    (formula::BURNING_SHIP4, "-1.059059912399875949806604684756823369616419345477182022818e+0", "-6.941445398749050928167618993678418040977879789587693625044e-2"),
    (formula::BURNING_SHIP5, "-1.069304830077022750409631353845070556935692686852387744345e+0", "-7.008594136830775532707863552269324262159198773198077894119e-2"),
    (formula::TRICORN3, "-8.301165086600040786410117965340966101130685955529819862755e-1", "-7.279926396365465868944042532991377115141048551447173006979e-1"),
    (formula::TRICORN4, "-2.596843675593252253654911553479302337079929899170186367197e-1", "-5.265878052427495866136872211038079448051950717593488419814e-1"),
    (formula::TRICORN5, "-9.033536580960661097439171803785319140128294786602977017732e-1", "-4.272544604520101032739291828337394992900827230127916689602e-1"),
    (formula::CELTIC4, "-4.234565860024477641147504048357300264219746679250861041278e-1", "-4.09820898134915444566944199392919060263917505185772053164e-1"),
    (formula::CELTIC5, "-2.281553024011103909013555011760308970573078526808823516802e-1", "-5.773582579137733907699085669559048183803546885637493650668e-1"),
    (formula::BUFFALO4, "-3.071721506556603906348160925396570822822053282996192416967e-1", "4.14173699729186134054547251993249825273527878456300080967e-1"),
];

/// At each fold family's deep point, over a 5×5 grid of δc spanning 1e-30: every skip of
/// `bla_fold_walk` lands within 1e-4 of the offset where plain perturbation (the same walk, rebasing,
/// no skips) is at that iteration — compared up to each walk's first rebase, the deep descent BLA
/// exists for. And the final counts agree at nine in ten of the pixels where plain perturbation is
/// stable under ±1e-35 nudges of δc.
///
/// ⚠Not the counts alone. No finite precision is truth at a fold's chaotic pixel: it can wander for
/// hundreds of steps after its offset reaches O(1) (Burning Ship, traced: f64 perturbation followed
/// the 192-bit orbit difference to 1e-14 until step 300, then parted ×1.3 a step). And the outcome
/// can turn on the DIRECTION of an error as small as BLA's: at Celtic's fixture, δc = (−2.5, 2.5)·1e-31,
/// BLA's state at step 97 was 4e-12 from plain's (|δ| ~ 2e-6), a 1e-35 nudge of δc moved plain's by
/// 7e-11 and changed nothing, and BLA's pixel escaped at 162 where plain's never did.
#[test]
fn fold_bla_lands_where_the_exact_steps_go() {
    let p = 192;
    let max_iter: u32 = 30_000;
    let bail2 = 65536.0;
    let agree = |a: Option<f64>, b: Option<f64>| match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) => (x - y).abs() < 0.5,
        _ => false,
    };
    // A walk's records up to (not including) its first rebase: the reference index only climbs until one.
    let before_rebase = |t: &[(u32, u32, (f64, f64))]| -> usize {
        t.windows(2).position(|w| w[1].1 <= w[0].1).map_or(t.len(), |i| i + 1)
    };
    for (f, sx, sy) in FOLD_DEEP_FIXTURES {
        let at = |s: &str| parse_bf_prec(s, p).unwrap();
        let (orbit, _len) = reference_orbit(&bf(0.0, p), &bf(0.0, p), &at(sx), &at(sy), f, max_iter, p);
        let levels = build_bla_fold(&orbit, fe(1.0e-30), 1.0e-6, AuxAggParams::default(), f);
        let (mut landings, mut worst) = (0usize, 0.0f64);
        let (mut judged, mut counted_agree) = (0usize, 0usize);
        for i in 0..5 {
            for j in 0..5 {
                let dc = ((i as f64 - 2.0) * 0.25e-30, (j as f64 - 2.0) * 0.25e-30);
                let (mut tp, mut tb) = (Vec::new(), Vec::new());
                let n = bla_fold_walk(&orbit, &levels, dc, bail2, max_iter, f, false, Some(&mut tp));
                let b = bla_fold_walk(&orbit, &levels, dc, bail2, max_iter, f, true, Some(&mut tb));
                let plain_at: std::collections::HashMap<u32, (u32, (f64, f64))> =
                    tp[..before_rebase(&tp)].iter().map(|&(it, m, e)| (it, (m, e))).collect();
                for &(it, m, e) in &tb[..before_rebase(&tb)] {
                    let Some(&(pm, pe)) = plain_at.get(&it) else { continue };
                    assert_eq!(pm, m, "formula {f}, δc {dc:?}: at iteration {it} the reference index differs");
                    let rel = (e.0 - pe.0).hypot(e.1 - pe.1) / pe.0.hypot(pe.1).max(1e-300);
                    worst = worst.max(rel);
                    landings += 1;
                    assert!(
                        rel <= 1.0e-4,
                        "formula {f}, δc {dc:?}: a skip landed at iteration {it} {rel:.2e} from the exact steps"
                    );
                }
                let h = 1.0e-35;
                let plain = |d: (f64, f64)| bla_fold_iterate(&orbit, &levels, d, bail2, max_iter, f, false);
                if [(h, 0.0), (-h, 0.0), (0.0, h), (0.0, -h)].iter().all(|&(dx, dy)| agree(plain((dc.0 + dx, dc.1 + dy)), n)) {
                    judged += 1;
                    counted_agree += usize::from(agree(b, n));
                }
            }
        }
        assert!(landings >= 25, "formula {f}: only {landings} skip landings compared (worst {worst:.2e})");
        assert!(judged >= 5, "formula {f}: only {judged} of 25 δc stable");
        assert!(
            counted_agree * 10 >= judged * 9,
            "formula {f}: BLA's count agreed at {counted_agree} of {judged} stable pixels"
        );
        assert!(levels.len() > 8, "formula {f}: a tree of {} levels", levels.len());
    }
}
