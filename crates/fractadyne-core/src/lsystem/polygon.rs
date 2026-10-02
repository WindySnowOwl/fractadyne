//! Filling the turtle's polygons (design/lsystems.md §9.6): clip to the view, then cut into
//! triangles for the GPU.
//!
//! A polygon from a walk can reach far past the view (a filled snowflake at 1e30× is 1e30 pixels
//! across), so it is clipped to a rectangle around the view first: Sutherland–Hodgman against a
//! convex rectangle keeps the fill inside the rectangle exactly, and leaves coordinates a
//! triangulation can work with. Then ear clipping: exact for a simple polygon, convex or not. An
//! outline that crosses itself has no ears to clip once it is tangled; what is left is filled as a
//! fan from its first vertex — an approximation, for outlines the turtle drew crossed.

/// `pts` clipped to the rectangle `|x| ≤ half[0]`, `|y| ≤ half[1]`.
pub fn clip_to_rect(pts: &[[f64; 2]], half: [f64; 2]) -> Vec<[f64; 2]> {
    let mut poly = pts.to_vec();
    // Each edge of the rectangle: the inside test and the crossing point on its line.
    for (axis, sign) in [(0usize, 1.0f64), (0, -1.0), (1, 1.0), (1, -1.0)] {
        if poly.is_empty() {
            break;
        }
        let limit = half[axis];
        let inside = |p: &[f64; 2]| sign * p[axis] <= limit;
        let cross = |a: &[f64; 2], b: &[f64; 2]| {
            let t = (sign * limit - a[axis]) / (b[axis] - a[axis]);
            let mut q = [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])];
            q[axis] = sign * limit;
            q
        };
        let mut out = Vec::with_capacity(poly.len() + 4);
        for i in 0..poly.len() {
            let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
            match (inside(&a), inside(&b)) {
                (true, true) => out.push(b),
                (true, false) => out.push(cross(&a, &b)),
                (false, true) => {
                    out.push(cross(&a, &b));
                    out.push(b);
                }
                (false, false) => {}
            }
        }
        poly = out;
    }
    poly
}

/// Twice the signed area (positive: anticlockwise).
pub fn signed_area2(pts: &[[f64; 2]]) -> f64 {
    let n = pts.len();
    (0..n).map(|i| {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        a[0] * b[1] - b[0] * a[1]
    })
    .sum()
}

fn cross(o: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
}

/// Triangles covering the polygon `pts`, as index triples into it.
pub fn triangulate(pts: &[[f64; 2]]) -> Vec<[usize; 3]> {
    // Drop repeated vertices (a step of zero length; the closing vertex).
    let mut idx: Vec<usize> = Vec::with_capacity(pts.len());
    for i in 0..pts.len() {
        if idx.last().is_none_or(|&j| pts[j] != pts[i]) {
            idx.push(i);
        }
    }
    while idx.len() > 1 && pts[idx[0]] == pts[*idx.last().expect("non-empty")] {
        idx.pop();
    }
    let mut out = Vec::new();
    if idx.len() < 3 {
        return out;
    }
    let ring: Vec<[f64; 2]> = idx.iter().map(|&i| pts[i]).collect();
    let area = signed_area2(&ring);
    // (Not `signum`, which calls 0.0 positive.)
    if area == 0.0 || !area.is_finite() {
        return out;
    }
    let orient = area.signum();
    let convex = |a: [f64; 2], b: [f64; 2], c: [f64; 2]| cross(a, b, c) * orient > 0.0;
    let inside = |p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]| {
        let (d1, d2, d3) = (cross(a, b, p) * orient, cross(b, c, p) * orient, cross(c, a, p) * orient);
        d1 >= 0.0 && d2 >= 0.0 && d3 >= 0.0
    };
    let mut v = idx;
    let mut misses = 0;
    let mut i = 0;
    while v.len() > 3 {
        let n = v.len();
        let (pi, ci, ni) = (v[(i + n - 1) % n], v[i % n], v[(i + 1) % n]);
        let (a, b, c) = (pts[pi], pts[ci], pts[ni]);
        let ear = convex(a, b, c)
            && !v.iter().any(|&k| {
                k != pi && k != ci && k != ni && {
                    let p = pts[k];
                    p != a && p != b && p != c && inside(p, a, b, c)
                }
            });
        if ear {
            out.push([pi, ci, ni]);
            v.remove(i % n);
            misses = 0;
            if i >= v.len() {
                i = 0;
            }
        } else {
            i = (i + 1) % n;
            misses += 1;
            if misses > n {
                // No ear left: a tangled (self-crossing) remainder. A fan from its first vertex.
                for k in 1..v.len() - 1 {
                    out.push([v[0], v[k], v[k + 1]]);
                }
                return out;
            }
        }
    }
    out.push([v[0], v[1], v[2]]);
    out
}

#[cfg(test)]
mod tests;
