use super::*;

const BUDGET: u32 = 1 << 21;

fn close(a: [f64; 2], b: [f64; 2], tol: f64) -> bool {
    (a[0] - b[0]).hypot(a[1] - b[1]) <= tol
}

/// The period-2 bulb's cycle in closed form: `z± = (−1 ± √(−3 − 4c)) / 2`, multiplier `4(1 + c)`.
fn bulb2(c: [f64; 2]) -> ([f64; 2], [f64; 2], f64) {
    // √(w) for w = −3 − 4c, principal branch.
    let w = [-3.0 - 4.0 * c[0], -4.0 * c[1]];
    let r = w[0].hypot(w[1]);
    let s = [((r + w[0]) / 2.0).sqrt(), (((r - w[0]) / 2.0).sqrt()).copysign(w[1])];
    let a = [(-1.0 + s[0]) / 2.0, s[1] / 2.0];
    let b = [(-1.0 - s[0]) / 2.0, -s[1] / 2.0];
    let lambda = 4.0 * (1.0 + c[0]).hypot(c[1]);
    if a[0].hypot(a[1]) <= b[0].hypot(b[1]) { (a, b, lambda) } else { (b, a, lambda) }
}

/// The trap-shape study's view U, the edge of the period-2 bulb (multiplier 0.976): the cycle
/// `{z+, z−}` in orbit order from the point nearest 0, as the closed form has it.
#[test]
fn the_period_2_bulb_s_cycle_is_the_closed_form() {
    let c = [-0.9173735445860097, 0.22966943698602013];
    let (zp, zm, lambda) = bulb2(c);
    let cyc = attracting_cycle(c, 2, BUDGET, MAX_PERIOD).expect("an attracting cycle");
    assert_eq!(cyc.period(), 2);
    assert!(close(cyc.points[0], zp, 1e-12) && close(cyc.points[1], zm, 1e-12), "{:?} vs {zp:?} {zm:?}", cyc.points);
    assert!((cyc.log2_multiplier - lambda.log2()).abs() < 1e-9, "{} vs {}", cyc.log2_multiplier, lambda.log2());
}

/// The main cardioid: one point, the attracting fixed point `(1 − √(1 − 4c)) / 2`; the center 0 is
/// superattracting.
#[test]
fn the_main_cardioid_s_fixed_point() {
    let zero = attracting_cycle([0.0, 0.0], 2, BUDGET, MAX_PERIOD).expect("c = 0");
    assert_eq!(zero.points, vec![[0.0, 0.0]]);
    assert_eq!(zero.log2_multiplier, f64::NEG_INFINITY);
    let c = [0.2, 0.1];
    let cyc = attracting_cycle(c, 2, BUDGET, MAX_PERIOD).expect("inside the cardioid");
    assert_eq!(cyc.period(), 1);
    // 1 − 4c = 0.2 − 0.4i; z* = (1 − √(1 − 4c)) / 2.
    let w: [f64; 2] = [0.2, -0.4];
    let r = w[0].hypot(w[1]);
    let s = [((r + w[0]) / 2.0).sqrt(), -((r - w[0]) / 2.0).sqrt()];
    let z = [(1.0 - s[0]) / 2.0, -s[1] / 2.0];
    assert!(close(cyc.points[0], z, 1e-12), "{:?} vs {z:?}", cyc.points[0]);
    assert!(cyc.log2_multiplier < 0.0);
}

/// A period-3 component (the "rabbit", near its nucleus) and a Multibrot 3 fixed point: each point
/// is the image of the one before, and the cycle closes.
#[test]
fn other_periods_and_degrees_close_on_themselves() {
    for (c, d, p) in [([-0.1226, 0.7449], 2u32, 3usize), ([0.1, 0.05], 3, 1), ([-1.7549, 0.0], 2, 3)] {
        let cyc = attracting_cycle(c, d, BUDGET, MAX_PERIOD).unwrap_or_else(|| panic!("c {c:?} d {d}"));
        assert_eq!(cyc.period(), p, "c {c:?} d {d}");
        for k in 0..p {
            let next = step(cyc.points[k], c, d);
            assert!(close(next, cyc.points[(k + 1) % p], 1e-12), "c {c:?}: point {k} does not map to the next");
        }
        assert!(cyc.log2_multiplier < 0.0);
    }
}

/// Outside the set the orbit escapes: no cycle.
#[test]
fn no_cycle_outside_the_set() {
    for c in [[0.3, 0.0], [1.0, 0.0], [-2.1, 0.0], [0.0, 1.5], [0.5, 0.5]] {
        assert_eq!(attracting_cycle(c, 2, BUDGET, MAX_PERIOD), None, "c {c:?}");
    }
    // A cycle that does not attract is refused even when an orbit sits on it: the repelling fixed
    // point of c = 0.2 + 0.1i, (1 + √(1 − 4c)) / 2, where |f'| = |2z| is about 1.6.
    let w: [f64; 2] = [0.2, -0.4];
    let r = w[0].hypot(w[1]);
    let s = [((r + w[0]) / 2.0).sqrt(), -((r - w[0]) / 2.0).sqrt()];
    let rep = [(1.0 + s[0]) / 2.0, s[1] / 2.0];
    assert!(close(step(rep, [0.2, 0.1], 2), rep, 1e-12), "the fixed point");
    assert_eq!(cycle_of_orbit(&[rep, rep, rep, rep], None, 2, MAX_PERIOD), None);
}

/// Near a component's edge the orbit settles slowly (the multiplier near 1). At 0.999 the search
/// finds the cycle: successive periods repeat before the orbit arrives, and Newton finishes the way.
/// At 0.999999 the 2-cycle sits 0.0005 either side of the repelling fixed point it budded from, and
/// Newton from an orbit that has not settled can land there; whatever the search returns, it is
/// never a wrong cycle (2026-11-24: it took the fixed point for the cycle before `true_cycle`).
#[test]
fn near_an_edge_the_cycle_is_found_or_none_never_a_wrong_one() {
    for lambda in [0.999, 0.999_999] {
        let c = [-1.0 + lambda / 4.0, 0.0];
        let found = attracting_cycle(c, 2, BUDGET, MAX_PERIOD);
        if lambda == 0.999 {
            assert!(found.is_some(), "multiplier 0.999 settles within the budget");
        }
        if let Some(cyc) = found {
            assert_eq!(cyc.period(), 2, "multiplier {lambda}");
            let (zp, zm, l) = bulb2(c);
            assert!(close(cyc.points[0], zp, 1e-9) && close(cyc.points[1], zm, 1e-9), "multiplier {lambda}: {:?}", cyc.points);
            assert!((cyc.log2_multiplier - l.log2()).abs() < 1e-6);
        }
    }
}

/// A reference orbit's samples give the same cycle: one that settled (read off its last points,
/// repeated twice) and one built to its nucleus (closed at its period, superattracting).
#[test]
fn a_reference_orbit_gives_its_cycle() {
    let c = [-0.9173735445860097, 0.22966943698602013];
    let mut z = vec![[0.0f64, 0.0]];
    for _ in 0..5000 {
        let w = *z.last().unwrap();
        z.push(step(w, c, 2));
    }
    let from_orbit = cycle_of_orbit(&z, None, 2, MAX_PERIOD).expect("settled");
    let direct = attracting_cycle(c, 2, BUDGET, MAX_PERIOD).unwrap();
    assert_eq!(from_orbit.period(), 2);
    for k in 0..2 {
        assert!(close(from_orbit.points[k], direct.points[k], 1e-9), "{k}");
    }
    // The period-3 nucleus orbit: Z_0 = 0, Z_1 = c, Z_2, Z_3 ≈ 0 (the build stops there).
    let n3 = [-0.12256116687665361, 0.7448617666197442];
    let mut orbit = vec![[0.0f64, 0.0]];
    for _ in 0..3 {
        let w = *orbit.last().unwrap();
        orbit.push(step(w, n3, 2));
    }
    let nucleus = cycle_of_orbit(&orbit, Some(3), 2, MAX_PERIOD).expect("closed");
    assert_eq!(nucleus.points.len(), 3);
    assert_eq!(nucleus.points[0], [0.0, 0.0]);
    assert_eq!(nucleus.log2_multiplier, f64::NEG_INFINITY);
    // An escaping orbit, and one that has not settled, give none.
    let mut out = vec![[0.0f64, 0.0]];
    for _ in 0..20 {
        let w = *out.last().unwrap();
        out.push(step(w, [0.5, 0.0], 2));
        if out.last().unwrap()[0].abs() > 1e3 {
            break;
        }
    }
    assert_eq!(cycle_of_orbit(&out, None, 2, MAX_PERIOD), None);
    assert_eq!(cycle_of_orbit(&z[..40], None, 2, MAX_PERIOD), None, "40 steps at multiplier 0.976 have not settled");
}

/// A reference as the app stores it (packed samples, the build's tail), built by the reference
/// builder at the view's precision: inside the period-2 bulb it settles on the closed form's cycle;
/// outside the set it escapes and has none.
#[test]
fn a_stored_reference_gives_its_cycle() {
    let bf = |s: &str| crate::parse_bf_prec(s, 128).expect("a number");
    let (zero, prec) = (bf("0"), 128usize);
    let c = [-0.9173735445860097, 0.22966943698602013];
    let (orbit, len, tail) = crate::reference_orbit_t(&zero, &zero, &bf("-0.9173735445860097"), &bf("0.22966943698602013"), 0, 20_000, prec);
    assert!(!tail.escaped);
    let cyc = cycle_of_reference(&orbit, len, Some(&tail), 2).expect("settled on its cycle");
    let (zp, zm, _) = bulb2(c);
    assert_eq!(cyc.period(), 2);
    assert!(close(cyc.points[0], zp, 1e-9) && close(cyc.points[1], zm, 1e-9), "{:?}", cyc.points);
    let (orbit, len, tail) = crate::reference_orbit_t(&zero, &zero, &bf("0.3"), &bf("0"), 0, 20_000, prec);
    assert!(tail.escaped);
    assert_eq!(cycle_of_reference(&orbit, len, Some(&tail), 2), None);
}
