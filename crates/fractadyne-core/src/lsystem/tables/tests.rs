use super::*;
use crate::lsystem::{library, reference, LSystem};

fn sys(name: &str) -> LSystem {
    library::find(name).unwrap().system().unwrap()
}

fn close(a: [f64; 2], b: [f64; 2], tol: f64) -> bool {
    (a[0] - b[0]).abs() <= tol && (a[1] - b[1]).abs() <= tol
}

#[test]
fn the_koch_tables_are_the_koch_numbers() {
    let t = Tables::new(&sys("Koch curve"));
    for d in 0..=30u32 {
        let e = t.entry(b'F', d);
        let three = 3f64.powi(d as i32);
        assert!(close(e.fx.d, [three, 0.0], three * 1e-12), "d={d}: {:?}", e.fx.d);
        assert_eq!(e.n, 4f64.powi(d as i32), "d={d}");
        assert!((e.r - three).abs() <= three * 1e-12, "d={d}: r={}", e.r);
        assert_eq!(e.fx.turns, 0);
    }
    assert!((t.growth - 3.0).abs() < 1e-9, "{}", t.growth);
}

#[test]
fn the_hilbert_curve_ends_a_row_along() {
    let t = Tables::new(&sys("Hilbert curve"));
    for d in 1..=30u32 {
        let want = 2f64.powi(d as i32) - 1.0;
        let e = t.entry(b'X', d);
        assert!(close(e.fx.d, [want, 0.0], want * 1e-12), "d={d}: {:?}", e.fx.d);
        assert_eq!(e.n, 4f64.powi(d as i32) - 1.0);
    }
}

#[test]
fn each_curve_grows_by_its_own_factor() {
    for (name, g) in [
        ("Koch curve", 3.0),
        ("Koch snowflake", 3.0),
        ("Hilbert curve", 2.0),
        ("Peano curve", 3.0),
        ("Heighway dragon", 2f64.sqrt()),
        ("Levy C curve", 2f64.sqrt()),
        ("Terdragon", 3f64.sqrt()),
        ("Gosper curve", 7f64.sqrt()),
        ("Quadratic Gosper curve", 5.0),
        ("Sierpinski arrowhead", 2.0),
        ("Quadratic Koch island", 4.0),
        ("Plant (ABOP 1.24d)", 2.0),
    ] {
        let t = Tables::new(&sys(name));
        assert!((t.growth - g).abs() < 1e-6 * g, "{name}: {} (want {g})", t.growth);
        assert!(t.grows(), "{name}");
    }
    // A stem that lengthens by a step a generation does not grow by a factor.
    assert!(!Tables::new(&sys("Saupe's bush")).grows());
}

#[test]
fn a_picture_that_alternates_steps_its_order_by_two() {
    for (name, p) in [
        ("Sierpinski arrowhead", 2),
        ("Weed", 2),
        ("Koch curve", 1),
        ("Hilbert curve", 1),
        ("Heighway dragon", 1),
        ("Plant (ABOP 1.24f)", 1),
        ("Bush", 1),
    ] {
        let t = Tables::new(&sys(name));
        assert_eq!(t.period, p, "{name}");
        let n = t.auto_order(1000.0, 1.5).unwrap();
        assert_eq!((t.max_depth - n) % p, 0, "{name}: order {n} is out of phase");
        assert_eq!(t.in_phase(n + 1), if p == 2 { n } else { n + 1 }, "{name}");
    }
}

#[test]
fn an_overlapping_curve_stops_refining_where_it_saturates() {
    // Tiles overlaps itself: 1.6, 3.1, 12, 46 segments a square step at orders 1, 3, 7, 11.
    let t = Tables::new(&sys("Tiles"));
    for px in [250.0, 1e6] {
        let by_step = (0..=t.max_depth)
            .filter(|&n| (t.max_depth - n).is_multiple_of(t.period))
            .find(|&n| norm(t.step(n)) * px <= 1.5)
            .unwrap();
        let n = t.auto_order(px, 1.5).unwrap();
        assert!(n < by_step, "{n} vs {by_step}");
        assert!(t.density(n) <= MAX_DENSITY);
        assert!(t.density(n + t.period) > MAX_DENSITY, "held back no further than the cap");
    }
    // The plane-filling curves that do not overlap never meet the cap: about one segment a
    // square step at every order.
    for name in ["Hilbert curve", "Peano curve", "Moore curve", "Quadratic Gosper curve"] {
        let t = Tables::new(&sys(name));
        for n in [6, 12, 30] {
            assert!((t.density(n) - 1.0).abs() < 0.1, "{name} order {n}: {}", t.density(n));
        }
    }
    for name in ["Hilbert curve", "Peano curve", "Gosper curve", "Heighway dragon", "Koch snowflake", "Cross"] {
        let t = Tables::new(&sys(name));
        for px in [100.0, 1e4, 1e8] {
            let n = t.auto_order(px, 1.5).unwrap();
            assert!(norm(t.step(n)) * px <= 1.5, "{name} at {px}: order {n} held back");
        }
    }
}

#[test]
fn the_tables_go_as_deep_as_f64_allows() {
    let t = Tables::new(&sys("Koch curve"));
    // 4^d segments passes 1e250 at d = 415.
    assert_eq!(t.max_depth, 415);
    // 2^d segments passes 1e250 at d = 831.
    assert_eq!(Tables::new(&sys("Heighway dragon")).max_depth, 830);
}

/// A deterministic stream of numbers in [0, 1).
fn lcg(seed: &mut u64) -> f64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (*seed >> 11) as f64 / (1u64 << 53) as f64
}

fn random_effect(t: &Tables, seed: &mut u64) -> Effect {
    let n = t.division.unwrap_or(1) as f64;
    Effect {
        d: [lcg(seed) * 4.0 - 2.0, lcg(seed) * 4.0 - 2.0],
        turns: if t.division.is_some() { (lcg(seed) * n) as i64 } else { 0 },
        free: if lcg(seed) < 0.5 { 0.0 } else { lcg(seed) * TAU },
        flip: lcg(seed) < 0.5,
        scale: 0.5 + lcg(seed),
        colour: ColourFx { set: (lcg(seed) < 0.3).then_some(7), add: (lcg(seed) * 5.0) as i32 },
    }
}

#[test]
fn composing_effects_is_associative() {
    let mut seed = 1;
    for name in ["Koch curve", "Plant (ABOP 1.24a)", "Pentaplexity"] {
        let t = Tables::new(&sys(name));
        for _ in 0..200 {
            let (a, b, c) = (random_effect(&t, &mut seed), random_effect(&t, &mut seed), random_effect(&t, &mut seed));
            let l = t.then(&t.then(&a, &b), &c);
            let r = t.then(&a, &t.then(&b, &c));
            assert!(close(l.d, r.d, 1e-9), "{name}: {:?} vs {:?}", l.d, r.d);
            assert_eq!(l.turns, r.turns);
            let df = (l.free - r.free).abs();
            assert!(df < 1e-9 || (TAU - df) < 1e-9, "{name}: free {} vs {}", l.free, r.free);
            assert_eq!(l.flip, r.flip);
            assert!((l.scale - r.scale).abs() < 1e-12);
            assert_eq!(l.colour, r.colour);
        }
    }
}

/// The reference's end point and segments for the axiom at `order`, from the origin with step 1
/// along +x and no heading.
fn reference_run(s: &LSystem, order: u32) -> Option<([f64; 2], Vec<crate::lsystem::Segment>)> {
    let mut s = s.clone();
    s.heading = 0.0;
    let word = reference::expand(&s, order, 400_000)?;
    let run = reference::run(&s, &word, [0.0, 0.0], [1.0, 0.0]);
    Some((run.end, run.segments))
}

#[test]
fn every_table_entry_matches_the_reference() {
    for s in crate::lsystem::exercises::all() {
        let e = &s;
        let t = Tables::new(&s);
        for order in 0..=8 {
            let Some((end, segs)) = reference_run(&s, order) else { break };
            let a = t.axiom_entry(order);
            assert_eq!(a.n, segs.len() as f64, "{} order {order}: segment count", e.name);
            let tol = 1e-9 * (1.0 + a.r.abs());
            assert!(close(a.fx.d, end, tol), "{} order {order}: end {:?} vs {:?}", e.name, a.fx.d, end);
            // The reach bounds every point drawn, and is reached (it is a maximum, not a guess).
            let far = segs.iter().flat_map(|g| [g.a, g.b]).map(norm).fold(-1.0f64, f64::max);
            if segs.is_empty() {
                assert_eq!(a.r, -1.0, "{} order {order}", e.name);
            } else {
                assert!(far <= a.r + tol, "{} order {order}: drew at {far}, reach {}", e.name, a.r);
            }
        }
    }
}

#[test]
fn the_step_keeps_a_curve_in_place_as_the_order_rises() {
    // The Koch curve's vertices persist from order to order: its start and end are exact.
    let t = Tables::new(&sys("Koch curve"));
    for n in 0..40 {
        let u = t.step(n);
        let end = [u[0] * 3f64.powi(n as i32), u[1] * 3f64.powi(n as i32)];
        assert!(close(end, t.step(0), 1e-12), "order {n}: {end:?}");
    }
    // The dragon turns 45° an order; the step turns it back, so the whole curve's chord (axiom
    // `FX`: a step, then X) converges — to within the one step, 2^(−n/2).
    let t = Tables::new(&sys("Heighway dragon"));
    let span = |n: u32| mul(t.step(n), t.axiom_entry(n).fx.d);
    for n in 45..100 {
        assert!(close(span(n), span(n + 1), 1e-6), "order {n}: {:?} vs {:?}", span(n), span(n + 1));
    }
}

/// The step shrinks with every order (in phase), from order 0 — where a curve's measured symbol
/// may draw nothing yet (the dragon's X): a step that collapsed there put the order that follows
/// the zoom at 0 and drew a single segment (the uitest's first L-system screenshot).
#[test]
fn the_step_shrinks_with_every_order_from_the_first() {
    for e in library::SYSTEMS {
        let t = Tables::new(&e.system().unwrap());
        if !t.grows() {
            continue;
        }
        let p = t.period;
        for n in 0..t.max_depth.min(120).saturating_sub(p) {
            let (a, b) = (norm(t.step(n)), norm(t.step(n + p)));
            // Never growing (the dragon's orders 0 and 1 both step 1: its X draws nothing at 0)…
            assert!(b <= a * (1.0 + 1e-12), "{} order {n}: step {a} then {b}", e.name);
            // …and by about the growth factor: never a collapse.
            let ratio = a / b;
            let g = t.growth.powi(p as i32);
            assert!(ratio < 4.0 * g && ratio > 0.25 * g.min(4.0), "{} order {n}: steps {a} / {b} = {ratio} (growth {g})", e.name);
        }
        // So the order that follows the zoom is the first whose step is at most a pixel and a
        // half — unless the density cap held it back, and then only as far as the cap.
        for px in [300.0, 1e5] {
            let n = t.auto_order(px, 1.5).unwrap();
            if norm(t.step(n)) * px <= 1.5 {
                assert!(n < p || norm(t.step(n - p)) * px > 1.5, "{}: order {n} at {px} is more than needed", e.name);
            } else {
                assert!(t.density(n + p) > MAX_DENSITY, "{}: order {n} at {px} held back below the cap", e.name);
            }
        }
    }
}

#[test]
fn the_auto_order_puts_a_step_at_about_a_pixel() {
    let t = Tables::new(&sys("Koch curve"));
    // One world unit = 1000 px: 3^-n ≤ 1.5/1000 ⇒ n = 6 (3^6 = 729 ≥ 667).
    assert_eq!(t.auto_order(1000.0, 1.5), Some(6));
    // Zooming in by the growth factor raises the order by one.
    assert_eq!(t.auto_order(3000.0, 1.5), Some(7));
    assert_eq!(Tables::new(&sys("Saupe's bush")).auto_order(1000.0, 1.5), None);
}
