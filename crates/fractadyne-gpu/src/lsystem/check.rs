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
    // Group 0 as the segment shader sees it: only the counters (binding 2).
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("lsystem.check.counters"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let counters = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("lsystem.check.counters"),
        size: 4 * 64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("lsystem.check.group0"),
        layout: &bgl,
        entries: &[wgpu::BindGroupEntry { binding: 2, resource: counters.as_entire_binding() }],
    });
    let mut r = LSystemRenderer::new(device, &bgl);
    r.update(device, queue, frame, size, ss);
    let texture = |label| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d { width: size[0], height: size[1], depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: crate::ITER_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    };
    let (main, aux) = (texture("lsystem.check.main"), texture("lsystem.check.aux"));
    let views = [main.create_view(&Default::default()), aux.create_view(&Default::default())];
    let row = (u64::from(size[0]) * 16).div_ceil(256) * 256;
    let read = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("lsystem.check.read"),
        size: row * u64::from(size[1]),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&Default::default());
    {
        let attach = |v, clear| {
            Some(wgpu::RenderPassColorAttachment {
                view: v,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(clear), store: wgpu::StoreOp::Store },
            })
        };
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("lsystem.check.segments"),
            color_attachments: &[attach(&views[0], CLEAR_MAIN), attach(&views[1], CLEAR_AUX)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        r.draw(&mut pass, &group0);
    }
    enc.copy_texture_to_buffer(
        main.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &read,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row as u32), rows_per_image: None },
        },
        wgpu::Extent3d { width: size[0], height: size[1], depth_or_array_layers: 1 },
    );
    queue.submit([enc.finish()]);
    let slice = read.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = device.poll(wgpu::Maintain::Wait);
    rx.recv().map_err(|e| e.to_string())?.map_err(|e| e.to_string())?;
    let floats: Vec<f32> = bytemuck::cast_slice(&slice.get_mapped_range()).to_vec();
    read.unmap();
    let per_row = (row / 4) as usize;
    Ok((0..size[1] as usize).flat_map(|y| (0..size[0] as usize).map(move |x| (y, x))).map(|(y, x)| floats[y * per_row + x * 4]).collect())
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
    let segs: Vec<([f64; 2], [f64; 2], f32)> = frame.segments.iter().map(|s| (to_texel(s.a), to_texel(s.b), s.value)).collect();
    let tris: Vec<([[f64; 2]; 3], f32)> =
        frame.triangles.iter().map(|t| ([to_texel(t.a), to_texel(t.b), to_texel(t.c)], t.value)).collect();
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
            TriangleInstance { a: p(a0), b: p(a1), c: p(a2), value: 0.01 + (k as f32 + 0.5) / n as f32 * 0.98 }
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
            SegmentInstance { a, b: [a[0] + len * ang.cos(), a[1] + len * ang.sin()], value: (k as f32 + 0.5) / n as f32 }
        })
        .collect()
}

/// The pass covers what the model says, texel for texel: thin and thick lines, at 1 and 2 texels a
/// pixel, drawn where their walk was and moved under a view that zoomed and panned since.
pub fn coverage(device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<Outcome> {
    let mut out = Vec::new();
    for (width, ss, scale, offset, tris, label) in [
        (1.0f32, 1u32, 1.0f32, [0.0f32, 0.0f32], 0usize, "1 px lines"),
        (3.0, 1, 1.0, [0.0, 0.0], 0, "3 px lines"),
        (1.5, 2, 1.0, [0.0, 0.0], 0, "1.5 px lines at 2 texels a pixel"),
        (2.0, 1, 1.37, [6.25, -3.5], 0, "a walk moved and scaled under the view"),
        (1.5, 1, 1.0, [0.0, 0.0], 40, "filled triangles under lines"),
    ] {
        let size = [97u32 * ss, 61 * ss];
        let segs = sample_segments(0x5EED + u64::from(ss) * 31 + width.to_bits() as u64, 160, 97.0, 61.0);
        let triangles = sample_triangles(0x7A1 + tris as u64, tris, 97.0, 61.0);
        let frame = LSystemFrame { segments: Arc::new(segs), triangles: Arc::new(triangles), segments_id: 1, scale, offset, width };
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
