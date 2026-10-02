use super::*;
use crate::lsystem::{library, reference, LSystem};

fn collect(t: &Tables, view: &View, opts: &WalkOptions) -> (Vec<Segment>, WalkStats) {
    let mut v = Vec::new();
    let stats = walk(t, view, opts, &mut |s| v.push(*s));
    (v, stats)
}

/// The reference's segments at `order`, in `view`'s pixels.
fn reference_segments(s: &LSystem, t: &Tables, order: u32, view: &View, limit: usize) -> Option<Vec<Segment>> {
    let word = reference::expand(s, order, limit)?;
    let u = t.step(order);
    let start = [-view.centre[0] / view.upp, -view.centre[1] / view.upp];
    Some(reference::draw(s, &word, start, [u[0] / view.upp, u[1] / view.upp]))
}

fn same(a: &Segment, b: &Segment, tol: f64) -> bool {
    let near = |p: [f64; 2], q: [f64; 2]| (p[0] - q[0]).abs() <= tol && (p[1] - q[1]).abs() <= tol;
    let dh = (a.heading - b.heading).abs();
    near(a.a, b.a)
        && near(a.b, b.b)
        && a.index == b.index
        && a.span == b.span
        && a.depth == b.depth
        && a.colour == b.colour
        && (dh < 1e-9 || 1.0 - dh < 1e-9)
}

/// A view framing the picture at `order`: `px` pixels across its larger side.
fn framing(t: &Tables, order: u32, px: f64) -> View {
    let b = bounds(t, order, 1 << 22).expect("it draws");
    let (w, h) = ((b[2] - b[0]).max(1e-12), (b[3] - b[1]).max(1e-12));
    let upp = w.max(h) / px;
    View { centre: [(b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0], upp, size: [w / upp + 2.0, h / upp + 2.0], margin: 1.0 }
}

#[test]
fn the_walk_is_the_reference_when_nothing_is_culled() {
    for s in crate::lsystem::exercises::all() {
        let e = &s;
        let t = Tables::new(&s);
        for order in 0..=7 {
            if t.axiom_entry(order).n == 0.0 {
                continue; // Hilbert's `X` draws nothing until it has rewritten once.
            }
            let view = framing(&t, order, 1000.0);
            let everything = View { size: [f64::INFINITY; 2], ..view };
            let Some(want) = reference_segments(&s, &t, order, &everything, 200_000) else { break };
            let (got, stats) = collect(&t, &everything, &WalkOptions { order, lod_px: 0.0, budget: u64::MAX });
            assert!(!stats.stopped);
            assert_eq!(got.len(), want.len(), "{} order {order}", e.name);
            for (k, (g, w)) in got.iter().zip(&want).enumerate() {
                assert!(same(g, w, 1e-7), "{} order {order} segment {k}:\n walk {g:?}\n  ref {w:?}", e.name);
            }
        }
    }
}

/// A deterministic stream of numbers in [0, 1).
fn lcg(seed: &mut u64) -> f64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (*seed >> 11) as f64 / (1u64 << 53) as f64
}

/// Whether a segment comes within `margin` pixels of the view, by points along it: independent of
/// the walk's discs. (Sampling can only miss a touch, which weakens the test, never falsifies it.)
fn touches(view: &View, s: &Segment) -> bool {
    let (hw, hh) = (view.size[0] / 2.0 + view.margin, view.size[1] / 2.0 + view.margin);
    (0..=64).any(|k| {
        let f = k as f64 / 64.0;
        let p = [s.a[0] + f * (s.b[0] - s.a[0]), s.a[1] + f * (s.b[1] - s.a[1])];
        p[0].abs() <= hw && p[1].abs() <= hh
    })
}

#[test]
fn culling_keeps_every_segment_that_touches_the_view_and_no_stranger() {
    let mut seed = 7;
    for s in crate::lsystem::exercises::all() {
        let e = &s;
        let t = Tables::new(&s);
        let order = (0..=7).rev().find(|&n| t.axiom_entry(n).n <= 60_000.0).unwrap_or(0);
        let whole = framing(&t, order, 800.0);
        // The curve in world units, to centre views on (a random point of the frame is mostly
        // empty space once zoomed in, and an empty view tests nothing).
        let world = reference_segments(&s, &t, order, &View::EVERYTHING, 1_000_000).unwrap();
        if world.is_empty() {
            continue;
        }
        let mut kept_total = 0;
        for _ in 0..8 {
            // A view on a random segment of the curve, zoomed in up to 50×.
            let zoom = 1.0 + 49.0 * lcg(&mut seed);
            let on = world[(lcg(&mut seed) * world.len() as f64) as usize];
            let view = View {
                centre: [0.5 * (on.a[0] + on.b[0]), 0.5 * (on.a[1] + on.b[1])],
                upp: whole.upp / zoom,
                size: [320.0, 200.0],
                margin: 1.5,
            };
            let want = reference_segments(&s, &t, order, &view, 1_000_000).unwrap();
            let (got, _) = collect(&t, &view, &WalkOptions { order, lod_px: 0.0, budget: u64::MAX });
            kept_total += got.len();
            let by_index: std::collections::HashMap<u64, &Segment> = want.iter().map(|w| (w.index as u64, w)).collect();
            for g in &got {
                let w = by_index.get(&(g.index as u64)).unwrap_or_else(|| panic!("{}: no segment {}", e.name, g.index));
                let tol = 1e-6 * (1.0 + g.a[0].abs().max(g.a[1].abs()));
                assert!(same(g, w, tol), "{}: walk {g:?}\n ref {w:?}", e.name);
            }
            let kept: std::collections::HashSet<u64> = got.iter().map(|g| g.index as u64).collect();
            for w in want.iter().filter(|w| touches(&view, w)) {
                assert!(kept.contains(&(w.index as u64)), "{}: segment {} touches the view but was culled: {w:?}", e.name, w.index);
            }
        }
        assert!(kept_total >= 8, "{}: the views held only {kept_total} segments", e.name);
    }
}

#[test]
fn a_mirrored_subtree_is_stepped_over_mirrored() {
    // `!` then X (which goes 2 along, 1 left: mirrored, 1 right), then F from where X ended. A view
    // around that F alone sees X culled; the F is where it is only if X was stepped over mirrored.
    let s = LSystem::parse("angle 90\naxiom !XF\nX = F+F-F\n").unwrap();
    let t = Tables::new(&s);
    assert!(!t.grows());
    let upp = norm(t.step(1));
    let u = t.step(1);
    assert!((u[0] - upp).abs() < 1e-12 && u[1].abs() < 1e-12, "step along +x: {u:?}");
    // 20 px a step; the view (16 px square) is centred on the F's far end, (3, −1) steps: X's reach
    // (√5 steps, 44.7 px from the origin) misses it by 9 px.
    let view = View { centre: [3.0 * upp, -upp], upp: upp / 20.0, size: [16.0, 16.0], margin: 0.0 };
    let (got, stats) = collect(&t, &view, &WalkOptions { order: 1, lod_px: 0.0, budget: u64::MAX });
    assert_eq!(stats.nodes, 1, "X was looked at once, and stepped over");
    assert_eq!(got.len(), 1, "{got:?}");
    let g = got[0];
    assert!((g.a[0] + 20.0).abs() < 1e-9 && g.a[1].abs() < 1e-9 && g.b[0].abs() < 1e-9 && g.b[1].abs() < 1e-9, "{g:?}");
    assert_eq!(g.index, 3.0, "the F after X's three");
}

#[test]
fn level_of_detail_bounds_the_segments_by_the_pixels() {
    for e in library::SYSTEMS {
        let s = e.system().unwrap();
        let t = Tables::new(&s);
        let px = 600.0;
        let home = framing(&t, crate::lsystem::framing_order(&t, 20_000.0), px);
        // At the app's step (3 px: a step no wider than the lines draws a plane-filler solid).
        let order = t.auto_order(1.0 / home.upp, 3.0).unwrap_or_else(|| s.order.unwrap_or(6)).min(t.max_depth);
        let (_, stats) = collect(&t, &home, &WalkOptions { order, lod_px: 1.5, budget: u64::MAX });
        let pixels = home.size[0] * home.size[1];
        assert!(
            (stats.segments as f64) <= 4.0 * pixels,
            "{}: {} segments for {pixels} pixels at order {order}",
            e.name,
            stats.segments
        );
    }
}

#[test]
fn a_view_deep_in_a_curve_costs_what_its_pixels_cost() {
    // The Koch curve at order 40 has 4^40 ≈ 1.2e24 segments; a 400×300 view near its start, at
    // 3^30 ≈ 2e14×, draws a few thousand of them.
    let s = library::find("Koch curve").unwrap().system().unwrap();
    let t = Tables::new(&s);
    let upp = 3f64.powi(-30) / 2.0;
    let view = View { centre: [200.0 * upp, 0.0], upp, size: [400.0, 300.0], margin: 1.0 };
    let order = t.auto_order(1.0 / upp, 1.5).unwrap();
    assert!(order >= 30, "{order}");
    let (segs, stats) = collect(&t, &view, &WalkOptions { order, lod_px: 1.0, budget: u64::MAX });
    assert!(!segs.is_empty());
    assert!(stats.segments < 20_000 && stats.nodes < 200_000, "{stats:?}");
    // Every segment is in or next to the view.
    for g in &segs {
        assert!(touches(&View { margin: 400.0, ..view }, g), "{g:?}");
    }
}

#[test]
fn the_budget_stops_the_walk_and_says_so() {
    let s = library::find("Peano curve").unwrap().system().unwrap();
    let t = Tables::new(&s);
    let (segs, stats) = collect(&t, &View::EVERYTHING, &WalkOptions { order: 5, lod_px: 0.0, budget: 1000 });
    assert_eq!(segs.len(), 1000);
    assert!(stats.stopped);
    let (segs, stats) = collect(&t, &View::EVERYTHING, &WalkOptions { order: 2, lod_px: 0.0, budget: 1000 });
    assert_eq!(segs.len(), 80);
    assert!(!stats.stopped);
}

#[test]
fn a_picture_stays_in_place_as_the_order_rises() {
    for e in library::SYSTEMS {
        let s = e.system().unwrap();
        let t = Tables::new(&s);
        if !t.grows() {
            continue;
        }
        let top = crate::lsystem::framing_order(&t, 200_000.0);
        let low = top - t.period;
        let a = bounds(&t, low, 1 << 22).unwrap();
        let b = bounds(&t, top, 1 << 22).unwrap();
        // Within a few of the coarser order's steps.
        let tol = 4.0 * norm(t.step(low));
        for k in 0..4 {
            assert!((a[k] - b[k]).abs() <= tol, "{} orders {low}/{top}: {a:?} vs {b:?} (tol {tol})", e.name);
        }
    }
}
