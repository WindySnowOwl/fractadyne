//! Device checks for the segment pass: the self-test's `lsystem` rows (design/lsystems.md §8,
//! phase 2's gate). The GPU's coverage is compared with a CPU model of the rule the shader states —
//! a texel takes the value of the LAST segment whose distance from the texel's centre is at most
//! the half-width, else "interior" — texel for texel. Only a texel whose centre lies within a
//! hair of a line's edge, where f32 and f64 may round to opposite sides, is left out, and those are
//! counted.

use super::*;
pub use crate::life::check::Outcome;

fn outcome(name: &str, params: impl Into<String>, result: Result<String, String>) -> Outcome {
    Outcome { name: name.into(), params: params.into(), result }
}

/// Draw `frame` into a `size` target at `ss` texels a pixel and read back main.r per texel.
pub fn render_segments(device: &wgpu::Device, queue: &wgpu::Queue, frame: &LSystemFrame, size: [u32; 2], ss: u32) -> Result<Vec<f32>, String> {
    Offscreen::new(device).render(device, queue, frame, size, ss)
}

/// The CPU model of the pass: per texel, the value of the last segment within the half-width of
/// its centre, `Some(-1)` where none is — or `None` where the answer depends on rounding (a centre
/// within `eps` of some line's edge changes the answer between `half − eps` and `half + eps`).
pub fn model(frame: &LSystemFrame, size: [u32; 2], ss: u32) -> Vec<Option<f32>> {
    let ss = f64::from(ss.max(1));
    let half = (0.5 * f64::from(frame.width) * ss).max(0.5);
    let eps = 1e-3;
    let to_texel = |p: [f32; 2]| {
        let q = [(f64::from(p[0]) * f64::from(frame.scale) + f64::from(frame.offset[0])) * ss, (f64::from(p[1]) * f64::from(frame.scale) + f64::from(frame.offset[1])) * ss];
        [0.5 * f64::from(size[0]) + q[0], 0.5 * f64::from(size[1]) - q[1]]
    };
    // Drawn on as far as `progress`: a segment past it hidden, the one it falls in shortened (in
    // f32, as the shader shortens it), a filled shape shown once it is passed.
    let p = frame.progress;
    let segs: Vec<([f64; 2], [f64; 2], f32)> = frame
        .segments
        .iter()
        .filter(|s| s.t[0] < p)
        .map(|s| {
            let b = if s.t[1] > p {
                let f = ((p - s.t[0]) / (s.t[1] - s.t[0])).clamp(0.0, 1.0);
                [s.a[0] + (s.b[0] - s.a[0]) * f, s.a[1] + (s.b[1] - s.a[1]) * f]
            } else {
                s.b
            };
            (to_texel(s.a), to_texel(b), s.value)
        })
        .collect();
    let tris: Vec<([[f64; 2]; 3], f32)> =
        frame.triangles.iter().filter(|t| t.t < p).map(|t| ([to_texel(t.a), to_texel(t.b), to_texel(t.c)], t.value)).collect();
    let dist = |p: [f64; 2], a: [f64; 2], b: [f64; 2]| {
        let ab = [b[0] - a[0], b[1] - a[1]];
        let l2 = ab[0] * ab[0] + ab[1] * ab[1];
        let t = if l2 > 0.0 { (((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / l2).clamp(0.0, 1.0) } else { 0.0 };
        (p[0] - a[0] - t * ab[0]).hypot(p[1] - a[1] - t * ab[1])
    };
    // How far inside a triangle `p` is (negative: outside), in texels: the least signed distance
    // to its edges, oriented whichever way the triangle winds.
    let inside = |p: [f64; 2], v: &[[f64; 2]; 3]| {
        let area = (v[1][0] - v[0][0]) * (v[2][1] - v[0][1]) - (v[1][1] - v[0][1]) * (v[2][0] - v[0][0]);
        if area == 0.0 {
            return f64::NEG_INFINITY;
        }
        (0..3)
            .map(|i| {
                let (a, b) = (v[i], v[(i + 1) % 3]);
                let e = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
                e * area.signum() / (b[0] - a[0]).hypot(b[1] - a[1]).max(1e-300)
            })
            .fold(f64::INFINITY, f64::min)
    };
    let mut out = Vec::with_capacity((size[0] * size[1]) as usize);
    for y in 0..size[1] {
        for x in 0..size[0] {
            let p = [f64::from(x) + 0.5, f64::from(y) + 0.5];
            // Lines lie over fills: the last line that covers it, else the last triangle.
            let winner = |grow: f64| {
                segs.iter()
                    .rev()
                    .find(|s| dist(p, s.0, s.1) <= half + grow)
                    .map(|s| s.2)
                    .or_else(|| tris.iter().rev().find(|t| inside(p, &t.0) >= -grow).map(|t| t.1))
                    .unwrap_or(-1.0)
            };
            let (lo, hi) = (winner(-eps), winner(eps));
            out.push((lo == hi).then_some(lo));
        }
    }
    out
}

/// Seeded triangles over a `w`×`h`-pixel view, each with its own value.
pub fn sample_triangles(seed: u64, n: usize, w: f32, h: f32) -> Vec<TriangleInstance> {
    let mut s = seed | 1;
    let mut next = || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        (s >> 11) as f32 / (1u64 << 53) as f32
    };
    (0..n)
        .map(|k| {
            let c = [(next() - 0.5) * 1.1 * w, (next() - 0.5) * 1.1 * h];
            let r = 3.0 + next() * 0.2 * w;
            let p = |a: f32| [c[0] + r * a.cos(), c[1] + r * a.sin()];
            let a0 = next() * std::f32::consts::TAU;
            let (a1, a2) = (a0 + 1.0 + next() * 2.0, a0 + 3.5 + next());
            let value = 0.01 + (k as f32 + 0.5) / n as f32 * 0.98;
            TriangleInstance { a: p(a0), b: p(a1), c: p(a2), value, t: k as f32 / n as f32 }
        })
        .collect()
}

/// A seeded set of segments over a `w`×`h`-pixel view: long and short, some running off the
/// edges, some of zero length (dots), each with its own value, in draw order.
pub fn sample_segments(seed: u64, n: usize, w: f32, h: f32) -> Vec<SegmentInstance> {
    let mut s = seed | 1;
    let mut next = || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        (s >> 11) as f32 / (1u64 << 53) as f32
    };
    (0..n)
        .map(|k| {
            let a = [(next() - 0.5) * 1.2 * w, (next() - 0.5) * 1.2 * h];
            let len = if k % 7 == 0 { 0.0 } else { next().powi(2) * 0.5 * w };
            let ang = next() * std::f32::consts::TAU;
            // Along the curve: the k-th of n.
            let t = [k as f32 / n as f32, (k + 1) as f32 / n as f32];
            SegmentInstance { a, b: [a[0] + len * ang.cos(), a[1] + len * ang.sin()], value: (k as f32 + 0.5) / n as f32, t }
        })
        .collect()
}

/// The pass covers what the model says, texel for texel: thin and thick lines, at 1 and 2 texels a
/// pixel, drawn where their walk was and moved under a view that zoomed and panned since, and
/// drawn on part of the way.
pub fn coverage(device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<Outcome> {
    let mut out = Vec::new();
    for (width, ss, scale, offset, tris, progress, label) in [
        (1.0f32, 1u32, 1.0f32, [0.0f32, 0.0f32], 0usize, 1.0f32, "1 px lines"),
        (3.0, 1, 1.0, [0.0, 0.0], 0, 1.0, "3 px lines"),
        (1.5, 2, 1.0, [0.0, 0.0], 0, 1.0, "1.5 px lines at 2 texels a pixel"),
        (2.0, 1, 1.37, [6.25, -3.5], 0, 1.0, "a walk moved and scaled under the view"),
        (1.5, 1, 1.0, [0.0, 0.0], 40, 1.0, "filled triangles under lines"),
        // Through the 97th segment (of 160), a third of the way along it; 24 of the 40 shapes.
        (2.0, 1, 1.0, [0.0, 0.0], 40, 96.33 / 160.0, "drawn on to 60%"),
    ] {
        let size = [97u32 * ss, 61 * ss];
        let segs = sample_segments(0x5EED + u64::from(ss) * 31 + width.to_bits() as u64, 160, 97.0, 61.0);
        let triangles = sample_triangles(0x7A1 + tris as u64, tris, 97.0, 61.0);
        let frame = LSystemFrame { segments: Arc::new(segs), triangles: Arc::new(triangles), segments_id: 1, scale, offset, width, progress };
        let result = render_segments(device, queue, &frame, size, ss).and_then(|got| {
            let want = model(&frame, size, ss);
            let (mut bad, mut unsure, mut lit) = (0usize, 0usize, 0usize);
            let mut first = None;
            for (i, (g, w)) in got.iter().zip(&want).enumerate() {
                match w {
                    None => unsure += 1,
                    Some(w) => {
                        lit += usize::from(*w >= 0.0);
                        if g != w {
                            bad += 1;
                            first.get_or_insert((i % size[0] as usize, i / size[0] as usize, *g, *w));
                        }
                    }
                }
            }
            let total = got.len();
            if bad > 0 {
                let (x, y, g, w) = first.expect("a difference");
                Err(format!("{bad} of {total} texels differ (first at {x},{y}: GPU {g}, model {w}); {unsure} on an edge"))
            } else if lit < total / 20 {
                Err(format!("only {lit} of {total} texels lit: the check draws too little to mean anything"))
            } else if unsure * 50 > total {
                Err(format!("{unsure} of {total} texels on an edge: too many to leave out"))
            } else {
                Ok(format!("{} texels as the model, {lit} lit; {unsure} on an edge left out", total - unsure))
            }
        });
        out.push(outcome("L-system segment coverage", format!("{label}, 160 segments, {}×{}", size[0], size[1]), result));
    }
    out
}
