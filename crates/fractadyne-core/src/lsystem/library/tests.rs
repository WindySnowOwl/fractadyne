use super::*;
use crate::lsystem::{reference, Segment, Tables};
use std::collections::HashSet;

fn run(name: &str, order: u32) -> reference::Run {
    let mut s = find(name).unwrap().system().unwrap();
    s.heading = 0.0;
    let word = reference::expand(&s, order, 4_000_000).unwrap();
    reference::run(&s, &word, [0.0, 0.0], [1.0, 0.0])
}

fn round(p: [f64; 2]) -> (i64, i64) {
    let r = (p[0].round(), p[1].round());
    assert!((p[0] - r.0).abs() < 1e-6 && (p[1] - r.1).abs() < 1e-6, "{p:?} is not a grid point");
    (r.0 as i64, r.1 as i64)
}

/// The grid points a curve of unit steps visits, in order; asserts each step is to a neighbour.
fn vertices(segs: &[Segment]) -> Vec<(i64, i64)> {
    let mut v = vec![round(segs[0].a)];
    for g in segs {
        let (a, b) = (round(g.a), round(g.b));
        assert_eq!(a, *v.last().unwrap(), "the curve is unbroken");
        assert_eq!((a.0 - b.0).abs() + (a.1 - b.1).abs(), 1, "a unit step to a neighbour");
        v.push(b);
    }
    v
}

/// Asserts the points are every point of a `side` × `side` grid, once each.
fn fills_a_square(v: &[(i64, i64)], side: i64) {
    let set: HashSet<_> = v.iter().copied().collect();
    assert_eq!(set.len(), v.len(), "no cell visited twice");
    let (x0, y0) = (v.iter().map(|p| p.0).min().unwrap(), v.iter().map(|p| p.1).min().unwrap());
    let (x1, y1) = (v.iter().map(|p| p.0).max().unwrap(), v.iter().map(|p| p.1).max().unwrap());
    assert_eq!((x1 - x0 + 1, y1 - y0 + 1), (side, side));
    assert_eq!(v.len() as i64, side * side);
}

#[test]
fn every_entry_parses_and_names_are_unique() {
    let mut names = HashSet::new();
    for e in SYSTEMS {
        let s = e.system().unwrap_or_else(|err| panic!("{}: {err}", e.name));
        assert_eq!(s.name, e.name);
        assert!(names.insert(e.name.to_ascii_lowercase()), "{} twice", e.name);
        assert!(!e.about.is_empty());
        let t = Tables::new(&s);
        assert!(t.axiom_entry(3.min(t.max_depth)).w > 0.0, "{} draws nothing (no lines, no polygons)", e.name);
        // Everything in the set either grows by a factor or names the order to draw at.
        assert!(t.grows() || s.order.is_some(), "{} neither grows nor has an order", e.name);
    }
    for c in Category::ALL {
        assert!(SYSTEMS.iter().any(|e| e.category == c), "{c:?} is empty");
    }
}

#[test]
fn the_segment_counts_are_the_published_ones() {
    let p = |b: u64, n: u32| b.pow(n);
    for n in 0..=5u32 {
        for (name, want) in [
            ("Koch curve", p(4, n)),
            ("Koch snowflake", 3 * p(4, n)),
            ("Quadratic Koch curve", p(5, n)),
            ("Quadratic Koch island", 4 * p(8, n)),
            ("Levy C curve", p(2, n)),
            ("Heighway dragon", p(2, n)),
            ("Terdragon", p(3, n)),
            ("Sierpinski arrowhead", p(3, n)),
            ("Sierpinski triangle", p(3, n + 1)),
            ("Hilbert curve", p(4, n) - 1),
            ("Moore curve", p(4, n + 1) - 1),
            ("Peano curve", p(9, n) - 1),
            ("Gosper curve", p(7, n)),
        ] {
            assert_eq!(run(name, n).segments.len() as u64, want, "{name} order {n}");
        }
    }
}

#[test]
fn the_hilbert_and_peano_curves_visit_every_cell_once() {
    for n in 1..=5u32 {
        fills_a_square(&vertices(&run("Hilbert curve", n).segments), 1 << n);
        fills_a_square(&vertices(&run("Peano curve", n).segments), 3i64.pow(n));
    }
}

#[test]
fn the_moore_curve_is_a_loop_through_every_cell() {
    for n in 1..=5u32 {
        let v = vertices(&run("Moore curve", n).segments);
        fills_a_square(&v, 2 << n);
        let (a, b) = (v[0], *v.last().unwrap());
        assert_eq!((a.0 - b.0).abs() + (a.1 - b.1).abs(), 1, "order {n}: it ends next to its start");
    }
}

#[test]
fn the_quadratic_gosper_curve_fills_a_square_without_crossing() {
    for n in 1..=3u32 {
        let segs = run("Quadratic Gosper curve", n).segments;
        let v = vertices(&segs);
        let set: HashSet<_> = v.iter().copied().collect();
        assert_eq!(set.len(), v.len(), "order {n}: no point visited twice");
        let side = 5i64.pow(n);
        let (x0, x1) = (v.iter().map(|p| p.0).min().unwrap(), v.iter().map(|p| p.0).max().unwrap());
        let (y0, y1) = (v.iter().map(|p| p.1).min().unwrap(), v.iter().map(|p| p.1).max().unwrap());
        assert_eq!((x1 - x0, y1 - y0), (side, side), "order {n}: spans a {side}-square");
        assert_eq!(segs.len() as i64, side * side, "order {n}");
    }
}

#[test]
fn the_dragon_never_draws_an_edge_twice_and_ends_where_folding_says() {
    for n in 0..=14u32 {
        let r = run("Heighway dragon", n);
        let mut edges = HashSet::new();
        for g in &r.segments {
            let (a, b) = (round(g.a), round(g.b));
            assert!(edges.insert(if a < b { (a, b) } else { (b, a) }), "order {n}: an edge twice");
        }
        let d = r.end[0].hypot(r.end[1]);
        assert!((d - 2f64.powf(n as f64 / 2.0)).abs() < 1e-9, "order {n}: end at {d}");
    }
}

#[test]
fn the_closed_curves_close() {
    for name in ["Koch snowflake", "Quadratic Koch island", "Sierpinski curve", "Sierpinski square curve", "Pentaplexity", "Crystal", "Board", "Tiles", "Rings", "Cross"] {
        for n in 0..=3 {
            let e = run(name, n).end;
            assert!(e[0].abs() < 1e-9 && e[1].abs() < 1e-9, "{name} order {n} ends at {e:?}");
        }
    }
}

/// The filled snowflake's polygon is the snowflake's outline: its area at order n is the
/// textbook's A0·(1 + ⅓·Σ_{k<n} (4/9)^k), A0 the starting triangle's.
#[test]
fn the_filled_snowflake_has_the_snowflakes_area() {
    for n in 0..=5u32 {
        let run = run("Koch snowflake (filled)", n);
        assert!(run.segments.is_empty());
        assert_eq!(run.polygons.len(), 1);
        let area = 0.5 * crate::lsystem::polygon::signed_area2(&run.polygons[0].pts).abs();
        let side = 3f64.powi(n as i32);
        let sum: f64 = (0..n).map(|k| (4.0f64 / 9.0).powi(k as i32)).sum();
        let want = 3f64.sqrt() / 4.0 * side * side * (1.0 + sum / 3.0);
        assert!((area - want).abs() < 1e-9 * want, "order {n}: {area} vs {want}");
    }
}
