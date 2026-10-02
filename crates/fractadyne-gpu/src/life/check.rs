//! Device checks for the Life stepper and its display pass: the self-test's `life` rows, and the
//! GPU crate's ignored tests. Integer automata have exact answers, so every check is exact — the GPU
//! equals the CPU tile stepper cell for cell, known facts hold to the cell, display values are the
//! numbers the shader is defined to write.

use super::*;
use fractadyne_core::life::{parse_rle, CellWindow};

/// One check's verdict: what was checked, at what, and `Ok(what was seen)` or `Err(why not)`.
pub struct Outcome {
    pub name: String,
    pub params: String,
    pub result: Result<String, String>,
}

fn outcome(name: &str, params: impl Into<String>, result: Result<String, String>) -> Outcome {
    Outcome { name: name.into(), params: params.into(), result }
}

/// A random soup of `states`-valued cells (xorshift, so the same on every machine).
pub(crate) fn soup(u: &mut Universe, seed: u64, x: i64, y: i64, w: i64, h: i64, states: u16) {
    let mut s = seed;
    for j in 0..h {
        for i in 0..w {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            if s % 100 < 35 {
                u.set(x + i, y + j, 1 + ((s >> 20) % u64::from(states - 1)) as u8);
            }
        }
    }
}

/// Where two cell lists first differ, for a failure message.
fn first_difference(a: &[(i64, i64, u8)], b: &[(i64, i64, u8)]) -> String {
    let i = a.iter().zip(b).position(|(p, q)| p != q).unwrap_or(a.len().min(b.len()));
    format!("{} vs {} cells; first difference {:?} vs {:?}", a.len(), b.len(), a.get(i), b.get(i))
}

/// The rules every stepper check runs: totalistic, Generations, Hensel, von Neumann, B0.
pub const RULES: [&str; 10] = ["B3/S23", "B36/S23", "B2/S", "B2/S/C3", "B2/S345/C4", "B3/S2-i34q", "B2-a/S12", "B2/S013V", "B0/S", "B01/S1"];

/// The GPU stepper equals the CPU tile stepper, cell for cell, through tile growth and freeing: per
/// rule, soups on the plane, a torus and a bounded plane, compared after 1, 2, 13, 16, 17, 40 and
/// 111 more generations (200 in all — across batch boundaries), populations included, breaches zero.
pub fn stepper_matches_cpu(device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<Outcome> {
    let mut gpu = LifeGpu::new(device, 4096);
    let topologies = [
        Topology::Plane,
        Topology::Torus { width: 128, height: 64 },
        Topology::Bounded { x: -37, y: 11, width: 100, height: 70 },
    ];
    let mut out = Vec::new();
    for (k, r) in RULES.iter().enumerate() {
        let rule = Rule::parse(r).expect("a check rule parses");
        let verdict = (|| -> Result<String, String> {
            let mut compared = 0u64;
            for (t, &topology) in topologies.iter().enumerate() {
                let mut cpu = Universe::new(rule.clone(), topology).map_err(|e| e.to_string())?;
                soup(&mut cpu, 3 + 11 * k as u64 + t as u64, -20, 5, 48, 40, rule.states());
                gpu.load(queue, &cpu).map_err(|e| e.to_string())?;
                for step in [1u64, 2, 13, 16, 17, 40, 111] {
                    cpu.step_n(step);
                    gpu.step(device, queue, step).map_err(|e| e.to_string())?;
                    let back = gpu.download(device, queue).map_err(|e| e.to_string())?;
                    let at = format!("{topology:?} at generation {}", cpu.generation());
                    if back.generation() != cpu.generation() || back.background() != cpu.background() {
                        return Err(format!("{at}: generation/background {}/{} vs {}/{}", back.generation(), back.background(), cpu.generation(), cpu.background()));
                    }
                    let (a, b) = (back.cells(), cpu.cells());
                    if a != b {
                        return Err(format!("{at}: {}", first_difference(&a, &b)));
                    }
                    if gpu.population() != cpu.population() {
                        return Err(format!("{at}: population {} vs {}", gpu.population(), cpu.population()));
                    }
                    compared += b.len() as u64;
                }
                if gpu.breaches() != 0 {
                    return Err(format!("{topology:?}: the halo was breached {} times", gpu.breaches()));
                }
            }
            Ok(format!("equal at 21 checkpoints ({compared} cells compared), 0 breaches"))
        })();
        out.push(outcome("GPU stepper = CPU tile stepper", format!("{r}: soups on plane / torus / bounded, 200 generations"), verdict));
    }
    out
}

/// A glider driven 1,000 cells across tile corners keeps its five cells and leaves no tiles
/// behind; a Seeds explosion in a 9-tile pool stops with `PoolFull`, exact to where it stopped.
pub fn tile_set_and_pool(device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<Outcome> {
    let glider = (|| -> Result<String, String> {
        let mut cpu = Universe::new(Rule::life(), Topology::Plane).map_err(|e| e.to_string())?;
        for (x, y) in [(61, 60), (62, 61), (60, 62), (61, 62), (62, 62)] {
            cpu.set(x, y, 1);
        }
        let mut gpu = LifeGpu::new(device, 64);
        gpu.load(queue, &cpu).map_err(|e| e.to_string())?;
        let mut most = 0;
        for _ in 0..250 {
            gpu.step(device, queue, 16).map_err(|e| e.to_string())?;
            most = most.max(gpu.tile_count());
        }
        cpu.step_n(4000);
        let back = gpu.download(device, queue).map_err(|e| e.to_string())?;
        if back.cells() != cpu.cells() {
            return Err(first_difference(&back.cells(), &cpu.cells()));
        }
        if most > 16 {
            return Err(format!("{most} tiles stored at once (a glider and its halo need at most 16)"));
        }
        Ok(format!("at most {most} tiles, {} breaches", gpu.breaches()))
    })();
    let full = (|| -> Result<String, String> {
        let mut seeds = Universe::new(Rule::parse("B2/S").expect("Seeds"), Topology::Plane).map_err(|e| e.to_string())?;
        seeds.set(0, 0, 1);
        seeds.set(1, 0, 1);
        let mut small = LifeGpu::new(device, 9);
        small.load(queue, &seeds).map_err(|e| e.to_string())?;
        let mut stopped = None;
        for _ in 0..20 {
            if let Err(e) = small.step(device, queue, 16) {
                stopped = Some(e);
                break;
            }
        }
        match stopped {
            Some(LifeGpuError::PoolFull { capacity: 9, .. }) => {}
            other => return Err(format!("expected a full pool, got {other:?}")),
        }
        seeds.step_n(small.generation());
        let back = small.download(device, queue).map_err(|e| e.to_string())?;
        if back.cells() != seeds.cells() {
            return Err(format!("not exact where it stopped: {}", first_difference(&back.cells(), &seeds.cells())));
        }
        Ok(format!("stopped at generation {} with {} cells, exact", small.generation(), seeds.population()))
    })();
    vec![
        outcome("GPU tile set follows the pattern", "a glider, 1,000 cells across tile corners", glider),
        outcome("a full pool stops, never drops cells", "Seeds from a domino in a 9-tile pool", full),
    ]
}

/// LifeWiki's long-run facts, on the GPU: the R-pentomino has 116 cells at generation 1103 (and
/// still at 2000), acorn 633 at 5206, Gosper's gun gains a glider (5 cells) every 30 generations.
pub fn known_facts(device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<Outcome> {
    let run = |rle: &str, checkpoints: &[(u64, u64)]| -> Result<String, String> {
        let mut u = Universe::new(Rule::life(), Topology::Plane).map_err(|e| e.to_string())?;
        for &(x, y, s) in &parse_rle(rle).map_err(|e| e.to_string())?.cells {
            u.set(x, y, s);
        }
        let mut gpu = LifeGpu::new(device, 1024);
        gpu.load(queue, &u).map_err(|e| e.to_string())?;
        let mut seen = Vec::new();
        for &(generation, want) in checkpoints {
            gpu.step(device, queue, generation - gpu.generation()).map_err(|e| e.to_string())?;
            if gpu.population() != want {
                return Err(format!("population {} at generation {generation}, want {want}", gpu.population()));
            }
            seen.push(format!("{want}@{generation}"));
        }
        Ok(seen.join(", "))
    };
    // The gun is back in its own phase every 30 generations with one more glider out.
    let gun: Vec<(u64, u64)> = (2..=12).map(|k| (30 * k, 36 + 5 * k)).collect();
    vec![
        outcome("GPU Life facts", "R-pentomino settles at 1103 with 116", run("b2o$2o$bo!", &[(1103, 116), (2000, 116)])),
        outcome("GPU Life facts", "acorn settles at 5206 with 633", run("bo$3bo$2o2b3o!", &[(5206, 633), (6000, 633)])),
        outcome(
            "GPU Life facts",
            "Gosper gun: +5 cells every 30 generations",
            run("24bo$22bobo$12b2o6b2o12b2o$11bo3bo4b2o12b2o$2o8bo5bo3b2o$2o8bo3bob2o4bobo$10bo5bo7bo$11bo3bo$12b2o!", &gun),
        ),
    ]
}

/// Draw `u` through the display pass into a `size` texture and read back main.r per texel.
pub fn render_display(device: &wgpu::Device, queue: &wgpu::Queue, u: &Universe, window: CellWindow, size: [u32; 2]) -> Result<Vec<f32>, String> {
    // Group 0 as the display shader sees it: only the counters (binding 2).
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("life.check.counters"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: storage(false),
            count: None,
        }],
    });
    let counters = buffer(device, "life.check.counters", 4 * 64, wgpu::BufferUsages::STORAGE);
    let group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("life.check.group0"),
        layout: &bgl,
        entries: &[wgpu::BindGroupEntry { binding: 2, resource: counters.as_entire_binding() }],
    });
    let mut lr = LifeRenderer::with_capacity(device, &bgl, 1024);
    let frame = LifeFrame {
        load_id: 1,
        load: Arc::new(u.clone()),
        target: u.generation(),
        max_steps: 0,
        window,
        status: Arc::new(Mutex::new(LifeStatus::default())),
        download: false,
    };
    lr.update(device, queue, &frame, size, 1);
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
    let (main, aux) = (texture("life.check.main"), texture("life.check.aux"));
    let views = [main.create_view(&Default::default()), aux.create_view(&Default::default())];
    let row = (u64::from(size[0]) * 16).div_ceil(256) * 256;
    let read = buffer(device, "life.check.read", row * u64::from(size[1]), wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST);
    let mut enc = device.create_command_encoder(&Default::default());
    {
        let attach = |v| {
            Some(wgpu::RenderPassColorAttachment {
                view: v,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
            })
        };
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("life.check.display"),
            color_attachments: &[attach(&views[0]), attach(&views[1])],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        lr.draw(&mut pass, &group0);
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
    let words = read_back(device, &read, row * u64::from(size[1])).map_err(|e| e.to_string())?;
    let floats: &[f32] = bytemuck::cast_slice(&words);
    let per_row = (row / 4) as usize;
    Ok((0..size[1] as usize).flat_map(|y| (0..size[0] as usize).map(move |x| (y, x))).map(|(y, x)| floats[y * per_row + x * 4]).collect())
}

/// The display pass writes the values it is defined to: zoomed in, 1 for a live cell, the
/// Generations fade for a dying one, interior (< 0) for a dead one — every texel; zoomed out, the
/// fraction of live cells under the texel; far out, a stored tile's density from its population.
pub fn display(device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<Outcome> {
    let mut out = Vec::new();
    // Zoomed in: 8 px a cell over cells 8..16, a glider and a dying cell (Star Wars, 4 states).
    let states = (|| -> Result<String, String> {
        let mut u = Universe::new(Rule::parse("B2/S345/C4").expect("Star Wars"), Topology::Plane).map_err(|e| e.to_string())?;
        for (x, y, s) in [(10, 9, 1), (11, 10, 1), (9, 11, 1), (10, 11, 1), (11, 11, 1), (14, 14, 2), (15, 14, 3)] {
            u.set(x, y, s);
        }
        let window = CellWindow { tile_x0: 0, tile_y0: 0, origin: [8.0, 8.0], cells_per_px: 0.125 };
        let px = render_display(device, queue, &u, window, [64, 64])?;
        let mut bad = 0;
        for y in 0..64usize {
            for x in 0..64usize {
                let (cx, cy) = (8 + (x / 8) as i64, 8 + (y / 8) as i64);
                let got = px[y * 64 + x];
                let ok = match u.get(cx, cy) {
                    0 => got == -1.0,
                    1 => got == 1.0,
                    // A dying state's fade is a float quotient: a GPU may divide through a
                    // reciprocal, so within an f32 ulp or two, not bit for bit.
                    s => (got - (1.0 - 0.9 * f32::from(s - 1) / 3.0)).abs() <= 1e-6,
                };
                bad += usize::from(!ok);
            }
        }
        if bad > 0 { Err(format!("{bad} of 4,096 texels differ")) } else { Ok("all 4,096 texels as defined".into()) }
    })();
    out.push(outcome("Life display values", "8 px a cell: live, dying (Star Wars), dead", states));
    // Zoomed out: 2 cells a pixel; a diagonal pair in each 2×2 is half alive.
    let density = (|| -> Result<String, String> {
        let mut u = Universe::new(Rule::life(), Topology::Plane).map_err(|e| e.to_string())?;
        for j in 0..8i64 {
            for i in 0..8i64 {
                u.set(2 * i, 2 * j, 1);
                u.set(2 * i + 1, 2 * j + 1, 1);
            }
        }
        let window = CellWindow { tile_x0: 0, tile_y0: 0, origin: [0.0, 0.0], cells_per_px: 2.0 };
        let px = render_display(device, queue, &u, window, [16, 16])?;
        let wrong: Vec<usize> = (0..256).filter(|&i| {
            let (x, y) = (i % 16, i / 16);
            let want = if x < 8 && y < 8 { 0.5 } else { -1.0 };
            px[i] != want
        }).collect();
        if wrong.is_empty() { Ok("every texel 0.5 or interior".into()) } else { Err(format!("{} texels wrong, first {:?} = {}", wrong.len(), wrong[0], px[wrong[0]])) }
    })();
    out.push(outcome("Life display values", "2 cells a pixel: half-alive blocks read 0.5", density));
    // Far out: 128 cells a pixel; one full tile at (1, 1) is density 1 in its bin.
    let coarse = (|| -> Result<String, String> {
        let mut u = Universe::new(Rule::life(), Topology::Plane).map_err(|e| e.to_string())?;
        for y in 64..128 {
            for x in 64..128 {
                u.set(x, y, 1);
            }
        }
        let window = CellWindow { tile_x0: 0, tile_y0: 0, origin: [0.0, 0.0], cells_per_px: 128.0 };
        let px = render_display(device, queue, &u, window, [4, 4])?;
        // Texel (0, 0)'s centre is cell (64, 64): the full tile; every other texel's bin is empty.
        let want: Vec<f32> = (0..16).map(|i| if i == 0 { 1.0 } else { -1.0 }).collect();
        if px == want { Ok("the full tile reads 1, the rest interior".into()) } else { Err(format!("read {px:?}")) }
    })();
    out.push(outcome("Life display values", "128 cells a pixel: a full tile's bin", coarse));
    out
}
