use super::*;
use crate::lsystem::{exercises, library, walk, LSystem, View, WalkOptions};
use std::collections::HashMap;

fn bfv(v: f64, p: usize) -> BigFloat {
    BigFloat::from_f64(v, p)
}

fn deep_view(cx: BigFloat, cy: BigFloat, upp_log2: f64, size: [f64; 2]) -> DeepView {
    DeepView { centre: [cx, cy], upp_log2, size, margin: 1.5 }
}

fn collect_f64(t: &Tables, v: &View, order: u32) -> Vec<Segment> {
    let mut out = Vec::new();
    walk(t, v, &WalkOptions { order, lod_px: 0.0, budget: u64::MAX }, &mut |s| out.push(*s));
    out
}

fn collect_deep(t: &Tables, bt: &BigTables, v: &DeepView, order: u32, switch_px: f64) -> (Vec<Segment>, WalkStats) {
    let mut out = Vec::new();
    let stats = deep_walk(t, bt, v, &WalkOptions { order, lod_px: 0.0, budget: u64::MAX }, switch_px, &mut |s| out.push(*s));
    (out, stats)
}

fn near(a: [f64; 2], b: [f64; 2], tol: f64) -> bool {
    (a[0] - b[0]).abs() <= tol && (a[1] - b[1]).abs() <= tol
}

/// Segments the two walks disagree on must be ones on the edge of the view, where a culling
/// decision can round either way; every other one must be the same segment, to `tol` pixels.
fn agree(name: &str, a: &[Segment], b: &[Segment], size: [f64; 2], tol: f64) -> usize {
    let inner = |s: &Segment| {
        let inside = |p: [f64; 2]| p[0].abs() < 0.5 * size[0] - 4.0 && p[1].abs() < 0.5 * size[1] - 4.0;
        inside(s.a) || inside(s.b)
    };
    let bi: HashMap<u64, &Segment> = b.iter().map(|s| (s.index as u64, s)).collect();
    let ai: HashMap<u64, &Segment> = a.iter().map(|s| (s.index as u64, s)).collect();
    let mut same = 0;
    for s in a {
        match bi.get(&(s.index as u64)) {
            Some(t) => {
                assert!(near(s.a, t.a, tol) && near(s.b, t.b, tol), "{name}: segment {}: {s:?}\n vs {t:?}", s.index);
                assert_eq!((s.depth, s.colour), (t.depth, t.colour), "{name}: segment {}", s.index);
                same += 1;
            }
            None => assert!(!inner(s), "{name}: segment {} only in the first walk: {s:?}", s.index),
        }
    }
    for s in b {
        assert!(ai.contains_key(&(s.index as u64)) || !inner(s), "{name}: segment {} only in the second walk: {s:?}", s.index);
    }
    same
}

/// A deterministic stream of numbers in [0, 1).
fn lcg(seed: &mut u64) -> f64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (*seed >> 11) as f64 / (1u64 << 53) as f64
}

#[test]
fn the_deep_walk_is_the_f64_walk_where_both_can_go() {
    // Switching to f64 at 16 px puts the BigFloat walk through nearly every level.
    let mut seed = 3;
    for s in exercises::all() {
        let t = Tables::new(&s);
        let Some(order) = (2..=6u32).rev().find(|&n| (1.0..=40_000.0).contains(&t.axiom_entry(n).n)) else { continue };
        let all = collect_f64(&t, &View::EVERYTHING, order);
        if all.is_empty() {
            continue;
        }
        let bt = BigTables::new(&s, &t, 160, order).unwrap();
        let mut same = 0;
        for _ in 0..4 {
            let on = all[(lcg(&mut seed) * all.len() as f64) as usize];
            let c = [0.5 * (on.a[0] + on.b[0]), 0.5 * (on.a[1] + on.b[1])];
            let extent = all.iter().flat_map(|g| [g.a[0].abs(), g.a[1].abs()]).fold(1e-9, f64::max);
            let upp = extent / 300.0 / (1.0 + 20.0 * lcg(&mut seed));
            let size = [200.0, 140.0];
            let a = collect_f64(&t, &View { centre: c, upp, size, margin: 1.5 }, order);
            let (b, _) = collect_deep(&t, &bt, &deep_view(bfv(c[0], 160), bfv(c[1], 160), upp.log2(), size), order, 16.0);
            same += agree(&s.name, &a, &b, size, 1e-6);
        }
        assert!(same > 0, "{}: the views held nothing to compare", s.name);
    }
}

fn koch() -> LSystem {
    library::find("Koch curve").unwrap().system().unwrap()
}

/// The deep-zoom gate the design names: the Koch curve zoomed 3^k about its start is the Koch curve
/// again, k orders up — segment for segment, in pixels, at 3^40 (past f64) and 3^200 (1e95).
#[test]
fn the_koch_curve_zoomed_by_three_to_the_k_is_itself() {
    let s = koch();
    let t = Tables::new(&s);
    let (n, size) = (6u32, [300.0, 200.0]);
    let upp: f64 = 0.3 / 300.0;
    let shallow = collect_f64(&t, &View { centre: [0.2, 0.05], upp, size, margin: 1.5 }, n);
    assert!(shallow.len() > 100, "{}", shallow.len());
    for k in [40u32, 200] {
        let p = deep_precision(&t, n + k, upp.log2() - f64::from(k) * 3f64.log2(), 1.0);
        let bt = BigTables::new(&s, &t, p, n + k).unwrap();
        let third_k = BigFloat::from_f64(1.0, p).div(&BigFloat::from_f64(3.0, p).powi(k as usize, p, RM), p, RM);
        let cx = decimal(0.2, p).mul(&third_k, p, RM);
        let cy = decimal(0.05, p).mul(&third_k, p, RM);
        let view = deep_view(cx, cy, upp.log2() - f64::from(k) * 3f64.log2(), size);
        let (deep, stats) = collect_deep(&t, &bt, &view, n + k, switch_px(&t));
        let same = agree(&format!("3^{k}"), &shallow, &deep, size, 1e-6);
        assert!(same * 10 >= shallow.len() * 9, "3^{k}: {same} of {} compared", shallow.len());
        // And it cost what the shallow view did: the deep levels add a few nodes each.
        assert!(stats.nodes < 4 * shallow.len() as u64 + 20 * u64::from(k), "3^{k}: {stats:?}");
    }
}

/// The same about the curve's END, (1, 0): the last quarter, scaled by 3 about the end, is the
/// whole curve. Near the start the coordinates are tiny and any precision holds them (that test
/// passed with the tables cut to 64 bits); here they are 1 − 3^-k, and every bit counts.
#[test]
fn the_koch_curve_zoomed_about_its_end_is_itself() {
    let s = koch();
    let t = Tables::new(&s);
    let (n, size) = (6u32, [300.0, 200.0]);
    let upp: f64 = 0.3 / 300.0;
    // Centred 0.2 before the end, 0.05 up.
    let shallow = collect_f64(&t, &View { centre: [0.8, 0.05], upp, size, margin: 1.5 }, n);
    for k in [40u32, 200] {
        let upp_log2 = upp.log2() - f64::from(k) * 3f64.log2();
        let p = deep_precision(&t, n + k, upp_log2, 1.0);
        let bt = BigTables::new(&s, &t, p, n + k).unwrap();
        let third_k = BigFloat::from_f64(1.0, p).div(&BigFloat::from_f64(3.0, p).powi(k as usize, p, RM), p, RM);
        let one = BigFloat::from_f64(1.0, p);
        let cx = one.sub(&decimal(0.2, p).mul(&third_k, p, RM), p, RM);
        let cy = decimal(0.05, p).mul(&third_k, p, RM);
        let (deep, _) = collect_deep(&t, &bt, &deep_view(cx, cy, upp_log2, size), n + k, switch_px(&t));
        // Indices differ (4^(n+k) − 4^n + i): match by where the segments are.
        let inner = |g: &Segment| g.a[0].abs() < 140.0 && g.a[1].abs() < 90.0;
        let mut matched = 0;
        for g in shallow.iter().filter(|g| inner(g)) {
            // To 1e-3 px: the f64 walk places a subtree of up to 2^31.7 px to ~2⁻¹² px (its tables
            // lose 0.4 bits a level), and here the coordinates are of that size, not tiny.
            let d = deep.iter().find(|d| near(g.a, d.a, 1e-3) && near(g.b, d.b, 1e-3));
            let closest = || deep.iter().min_by(|x, y| {
                let dist = |s: &Segment| (s.a[0] - g.a[0]).hypot(s.a[1] - g.a[1]);
                dist(x).total_cmp(&dist(y))
            });
            assert!(d.is_some(), "3^{k}: nothing at {:?} (shallow {g:?}); closest deep {:?}", g.a, closest());
            matched += 1;
        }
        assert!(matched > 100, "3^{k}: {matched}");
        assert!(deep.len().abs_diff(shallow.len()) * 20 < shallow.len(), "3^{k}: {} vs {}", deep.len(), shallow.len());
    }
}

/// The Hilbert curve's `i`-th cell, by the classic index-to-coordinates algorithm — independent of
/// the L-system, the turtle and the tables.
fn d2xy(n: u32, mut d: u128) -> (i128, i128) {
    let (mut x, mut y) = (0i128, 0i128);
    let mut s = 1i128;
    while s < (1i128 << n) {
        let rx = 1 & (d / 2) as i128;
        let ry = 1 & (d ^ rx as u128) as i128;
        if ry == 0 {
            if rx == 1 {
                x = s - 1 - x;
                y = s - 1 - y;
            }
            std::mem::swap(&mut x, &mut y);
        }
        x += s * rx;
        y += s * ry;
        d /= 4;
        s *= 2;
    }
    (x, y)
}

#[test]
fn hilbert_vertices_land_on_the_lattice_past_f64() {
    let s = library::find("Hilbert curve").unwrap().system().unwrap();
    let t = Tables::new(&s);
    // The L-system's orientation against d2xy's: a lattice symmetry found at small orders by the
    // naive turtle (unit steps from the origin), for each parity of the order.
    let transform = |n: u32| -> (i128, i128, i128, i128, i128, i128) {
        let word = crate::lsystem::reference::expand(&s, n, 1 << 20).unwrap();
        let segs = crate::lsystem::reference::draw(&s, &word, [0.0, 0.0], [1.0, 0.0]);
        let v = |i: usize| -> (i128, i128) {
            let p = if i == 0 { segs[0].a } else { segs[i - 1].b };
            (p[0].round() as i128, p[1].round() as i128)
        };
        let m = (1i128 << n) - 1;
        for (a, b, c, d) in [(1, 0, 0, 1), (0, -1, 1, 0), (-1, 0, 0, -1), (0, 1, -1, 0), (1, 0, 0, -1), (-1, 0, 0, 1), (0, 1, 1, 0), (0, -1, -1, 0)] {
            for (ox, oy) in [(0, 0), (m, 0), (0, m), (m, m), (-m, 0), (0, -m), (-m, -m), (m, -m), (-m, m)] {
                let ok = (0..=segs.len()).all(|i| {
                    let (x, y) = d2xy(n, i as u128);
                    (a * x + b * y + ox, c * x + d * y + oy) == v(i)
                });
                if ok {
                    return (a, b, c, d, ox / m.max(1), oy / m.max(1));
                }
            }
        }
        panic!("no lattice symmetry maps d2xy onto the L-system at order {n}");
    };
    let (t4, t5) = (transform(4), transform(5));
    assert_eq!(transform(6), t4, "the orientation repeats every two orders");
    let mut seed = 11;
    for n in [60u32, 62] {
        let (a, b, c, d, ox, oy) = if n % 2 == 0 { t4 } else { t5 };
        let m = (1i128 << n) - 1;
        // World = vertex / (2^n − 1): the Hilbert measure is 2^n − 1 steps along +x.
        assert!(t.step(n.min(t.max_depth))[1].abs() < 1e-300);
        let upp_log2 = -(m as f64).log2() - 2.0; // 4 px a step
        let p = deep_precision(&t, n, upp_log2, 1.0);
        let bt = BigTables::new(&s, &t, p, n).unwrap();
        for _ in 0..3 {
            let i = ((lcg(&mut seed) * 0.98 + 0.01) * ((1u128 << (2 * n)) - 1) as f64) as u128;
            let vert = |i: u128| {
                let (x, y) = d2xy(n, i);
                (a * x + b * y + ox * m, c * x + d * y + oy * m)
            };
            let (vx, vy) = vert(i);
            let world = |q: i128| BigFloat::parse(&q.to_string(), astro_float::Radix::Dec, p, RM, &mut Consts::new().unwrap()).div(&BigFloat::parse(&m.to_string(), astro_float::Radix::Dec, p, RM, &mut Consts::new().unwrap()), p, RM);
            let view = deep_view(world(vx), world(vy), upp_log2, [64.0, 64.0]);
            let (segs, stats) = collect_deep(&t, &bt, &view, n, switch_px(&t));
            assert!(!segs.is_empty() && stats.nodes < 4000, "order {n}: {stats:?}");
            // Every end is a lattice point (4 px apart), and the steps into and out of vertex i
            // are there: from i−1 to i and from i to i+1, as d2xy says.
            for g in &segs {
                for q in [g.a, g.b] {
                    let (gx, gy) = (q[0] / 4.0, q[1] / 4.0);
                    assert!((gx - gx.round()).abs() < 1e-6 && (gy - gy.round()).abs() < 1e-6, "order {n}: {q:?} is off the lattice");
                }
            }
            let has = |from: (i128, i128), to: (i128, i128)| {
                let f = [4.0 * (from.0 - vx) as f64, 4.0 * (from.1 - vy) as f64];
                let t = [4.0 * (to.0 - vx) as f64, 4.0 * (to.1 - vy) as f64];
                segs.iter().any(|g| near(g.a, f, 1e-6) && near(g.b, t, 1e-6))
            };
            assert!(has(vert(i - 1), vert(i)), "order {n}, vertex {i}: the step in is missing");
            assert!(has(vert(i), vert(i + 1)), "order {n}, vertex {i}: the step out is missing");
        }
    }
}

#[test]
fn a_view_at_1e95_costs_what_its_pixels_cost() {
    let s = library::find("Heighway dragon").unwrap().system().unwrap();
    let t = Tables::new(&s);
    let upp_log2 = -316.0; // 1e95
    let order = t.auto_order_log2(-upp_log2, 3.0).unwrap();
    assert!(order > t.max_depth.min(600) && order <= crate::lsystem::MAX_ORDER, "{order}");
    let p = deep_precision(&t, order, upp_log2, 1.0);
    let bt = BigTables::new(&s, &t, p, order).unwrap();
    // A point on the curve: the start of the dragon's middle segment, found shallow and
    // refined by walking — the view must hold some of the curve.
    let all = collect_f64(&t, &View::EVERYTHING, 12);
    let mid = all[all.len() / 2].a;
    let view = deep_view(bfv(mid[0], p), bfv(mid[1], p), upp_log2, [400.0, 300.0]);
    let mut segs = Vec::new();
    let stats = deep_walk(&t, &bt, &view, &WalkOptions { order, lod_px: 1.5, budget: 1 << 22 }, switch_px(&t), &mut |s| segs.push(*s));
    assert!(!stats.stopped);
    assert!(stats.segments < 4 * 400 * 300, "{stats:?}");
    assert!(stats.nodes < 200_000, "{stats:?}");
}

/// A system that does not grow keeps a step of 1 along its heading (the measure's direction plays
/// no part): the deep walk drew the scaled tree turned by it, 2 segments for the f64 walk's 15,573.
#[test]
fn a_picture_that_does_not_grow_is_drawn_along_its_heading() {
    let s = LSystem::parse("angle 25\nheading 90\naxiom F\nF = F[@0.6+F]@0.8F[!+F]-F\n").unwrap();
    let t = Tables::new(&s);
    assert!(!t.grows());
    let all = collect_f64(&t, &View::EVERYTHING, 6);
    let c = all[193].a;
    let size = [200.0, 140.0];
    let a = collect_f64(&t, &View { centre: c, upp: 0.01, size, margin: 1.5 }, 6);
    let bt = BigTables::new(&s, &t, 160, 6).unwrap();
    let (b, _) = collect_deep(&t, &bt, &deep_view(bfv(c[0], 160), bfv(c[1], 160), 0.01f64.log2(), size), 6, switch_px(&t));
    assert!(agree("scaled tree", &a, &b, size, 1e-6) > 1000);
}

/// The tables do not decay with depth. Headings kept as products of unit vectors carried their
/// rounding four-fold into every level above (the Koch curve's four children): by depth 415 the
/// heading was (0, 0) and the displacement 2.4e60 for 3^415 ≈ 1e198. As whole counts they cannot.
#[test]
fn the_big_tables_hold_their_values_to_the_deepest_row() {
    let s = koch();
    let t = Tables::new(&s);
    let p = 256;
    let bt = BigTables::new(&s, &t, p, 415).unwrap();
    for d in [1u32, 2, 6, 100, 415] {
        let e = bt.entry(b'F', d);
        assert_eq!(e.fx.turns, 0, "depth {d}: the Koch curve's net turn is none");
        let want = BigFloat::from_f64(3.0, p).powi(d as usize, p, RM);
        let rel = to_f64(&e.fx.d[0].sub(&want, p, RM).div(&want, p, RM)).abs();
        assert!(rel < 1e-60, "depth {d}: x off by {rel:e}");
        assert!(log2_abs(&e.fx.d[1]) < log2_abs(&want) - 200.0, "depth {d}: y = {}", to_f64(&e.fx.d[1]));
    }
    assert_eq!(to_f64(&bt.along[0]), 1.0);
    assert!(log2_abs(&bt.along[1]) < -200.0);
    // The 90° rotations are exact.
    let h = library::find("Hilbert curve").unwrap().system().unwrap();
    let bh = BigTables::new(&h, &Tables::new(&h), p, 4).unwrap();
    let q = bh.rot(bh.modulus / 4);
    assert!(q[0].is_zero() && to_f64(&q[1]) == 1.0);
}

#[test]
fn every_written_angle_has_a_common_unit() {
    let s = LSystem::parse("angle 25.7\nheading 90\naxiom F\nF = F[+F\\12.5F]|F/0.001F\n").unwrap();
    let u = Units::of(&s).unwrap();
    // 25.7° = 257/3600 turn; 12.5° = 1/28.8 turn; 0.001° = 1/360000 turn; 90° = 1/4; | = 1/2.
    assert_eq!(u.modulus % 360_000, 0);
    assert_eq!(u.step * 3600, 257 * u.modulus);
    assert_eq!(u.around * 2, u.modulus);
    assert_eq!(u.degrees(12.5) * 288, 10 * u.modulus);
    assert_eq!(u.degrees(-0.001).rem_euclid(u.modulus) * 360_000 % u.modulus, (u.modulus * 359_999) % u.modulus);
    assert_eq!(decimal_ratio(25.7), Some((257, 10)));
    assert_eq!(decimal_ratio(-0.001), Some((-1, 1000)));
    assert_eq!(decimal_ratio(1e-7), Some((1, 10_000_000)));
}

/// The Sierpinski triangle's tables lose about a bit a level (its `F` advances 2 steps with 5
/// steps of children): at the deepest row they were noise, and the picture's orientation and
/// growth taken there came out turned 90° and ×3 for ×2. Taken at a moderate depth, exactly, they
/// are the triangle's — and the deep and f64 walks agree on it.
#[test]
fn an_ill_conditioned_curve_keeps_its_orientation_and_growth() {
    let s = library::find("Sierpinski triangle").unwrap().system().unwrap();
    let t = Tables::new(&s);
    assert!(t.loss > 0.5, "loss {}", t.loss);
    assert!(t.along_depth < t.max_depth && t.along_depth >= 8);
    assert!((t.growth - 2.0).abs() < 1e-12, "growth {}", t.growth);
    let u = t.step(6);
    assert!(u[1].abs() < 1e-15 * u[0], "the base lies along +x: {u:?}");
    let bt = BigTables::new(&s, &t, 160, 6).unwrap();
    let b = bt.step(&t, 6);
    assert!(near([to_f64(&b[0]), to_f64(&b[1])], u, 1e-18), "{} {} vs {u:?}", to_f64(&b[0]), to_f64(&b[1]));
    // The exact curves lose nothing; the 60° Koch curve a little.
    assert_eq!(Tables::new(&library::find("Hilbert curve").unwrap().system().unwrap()).loss, 0.0);
    let k = Tables::new(&koch());
    assert!((k.loss - (4.0f64 / 3.0).log2()).abs() < 0.05, "Koch loss {}", k.loss);
}
