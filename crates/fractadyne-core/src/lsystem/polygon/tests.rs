use super::*;

fn area_of(pts: &[[f64; 2]], tris: &[[usize; 3]]) -> f64 {
    tris.iter().map(|t| 0.5 * cross(pts[t[0]], pts[t[1]], pts[t[2]]).abs()).sum()
}

#[test]
fn ear_clipping_covers_simple_polygons_exactly() {
    let star: Vec<[f64; 2]> = (0..10)
        .map(|k| {
            let a = std::f64::consts::TAU * f64::from(k) / 10.0;
            let r = if k % 2 == 0 { 1.0 } else { 0.4 };
            [r * a.cos(), r * a.sin()]
        })
        .collect();
    let comb = vec![[0.0, 0.0], [5.0, 0.0], [5.0, 3.0], [4.0, 3.0], [4.0, 1.0], [3.0, 1.0], [3.0, 3.0], [2.0, 3.0], [2.0, 1.0], [1.0, 1.0], [1.0, 3.0], [0.0, 3.0]];
    let square_cw = vec![[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
    for (name, pts) in [("star", star), ("comb", comb), ("clockwise square", square_cw)] {
        let tris = triangulate(&pts);
        assert_eq!(tris.len(), pts.len() - 2, "{name}");
        let want = 0.5 * signed_area2(&pts).abs();
        assert!((area_of(&pts, &tris) - want).abs() < 1e-12 * want.max(1.0), "{name}: {} vs {want}", area_of(&pts, &tris));
    }
}

#[test]
fn repeated_vertices_and_slivers_are_harmless() {
    // The turtle closes a polygon on its first vertex, and a zero-length step repeats one.
    let pts = vec![[0.0, 0.0], [2.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0], [0.0, 0.0]];
    let tris = triangulate(&pts);
    assert_eq!(tris.len(), 2);
    assert!((area_of(&pts, &tris) - 4.0).abs() < 1e-12);
    assert!(triangulate(&[[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]]).is_empty(), "a degenerate outline fills nothing");
    assert!(triangulate(&[[0.0, 0.0], [1.0, 0.0]]).is_empty());
}

#[test]
fn clipping_keeps_the_fill_inside_the_rectangle() {
    // A big triangle over the rectangle |x| ≤ 1, |y| ≤ 1, its corners far outside.
    let big = vec![[-1e30, -1e30], [1e30, -1e30], [0.0, 1e30]];
    let c = clip_to_rect(&big, [1.0, 1.0]);
    assert!((0.5 * signed_area2(&c).abs() - 4.0).abs() < 1e-9, "the whole rectangle: {c:?}");
    // A square half in, half out.
    let half = vec![[0.0, -0.5], [3.0, -0.5], [3.0, 0.5], [0.0, 0.5]];
    let c = clip_to_rect(&half, [1.0, 1.0]);
    assert!((0.5 * signed_area2(&c).abs() - 1.0).abs() < 1e-12);
    assert!(c.iter().all(|p| p[0].abs() <= 1.0 + 1e-12 && p[1].abs() <= 1.0 + 1e-12));
    // Wholly outside.
    assert!(clip_to_rect(&[[5.0, 5.0], [6.0, 5.0], [6.0, 6.0]], [1.0, 1.0]).len() < 3);
}
