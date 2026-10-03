//! Image export of an L-system view (design/lsystems.md §7). The view is walked at the export's
//! size with the screen's order and the screen's line width scaled to the export's pixels — so the
//! image is the picture on screen at the export's resolution — framed as every export is (the
//! smallest rectangle of the export's aspect that holds the window's view). The segment pass draws
//! it offscreen, tile by tile, and each texel is coloured as the screen's colour pass colours it:
//! `palette(value + offset)` from the baked LUT, the interior colour where nothing is drawn; a
//! pixel is the mean of its texels (the export's supersampling).

use super::*;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

/// Texels a tile holds on a side (a 64 MiB read-back at most).
const TILE: u32 = 2048;

/// An L-system export, gathered on the UI thread and rendered anywhere.
pub(crate) struct Job {
    system: LSystem,
    tables: Arc<Tables>,
    big: Option<Arc<BigTables>>,
    ex: Option<Arc<Expansion>>,
    key: WalkKey,
    depth_scale: f64,
    /// The line width, export pixels.
    width: f32,
    progress: f32,
    ss: u32,
    lut: fractadyne_color::segment::Lut,
    offset: f32,
    interior: [f32; 3],
}

/// A rendered export: its size, its pixels (RGBA, display space, rows from the top), and whether
/// the walk stopped at its budget (the picture is then incomplete, and the status says so).
pub(crate) struct Rendered {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pixels: Vec<f32>,
    pub(crate) stopped: bool,
    /// The deep tables and the built word it used (a tour keeps them for its next frame).
    big: Option<Arc<BigTables>>,
    ex: Option<Arc<Expansion>>,
}

impl Job {
    /// Walk, draw and colour. `progress` runs to 1000 (permille of the tiles); `cancel` stops it
    /// between tiles.
    pub(crate) fn render(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        progress: &AtomicU32,
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Result<Rendered, String> {
        self.render_in_tiles(device, queue, progress, cancel, TILE)
    }

    /// [`Job::render`], in tiles of `tile` texels a side (the self-test renders the same export in
    /// small tiles and in one, which must agree texel for texel).
    pub(crate) fn render_in_tiles(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        progress: &AtomicU32,
        cancel: &std::sync::atomic::AtomicBool,
        tile: u32,
    ) -> Result<Rendered, String> {
        let tile = tile.max(1);
        let [w, h] = self.key.size;
        let out = walk_segments(&self.system, &self.tables, self.big.clone(), self.ex.clone(), &self.key, self.depth_scale);
        let frame = LSystemFrame {
            segments: Arc::new(out.segments),
            triangles: Arc::new(out.triangles),
            segments_id: 1,
            scale: 1.0,
            offset: [0.0, 0.0],
            width: self.width,
            progress: self.progress,
        };
        let ss = self.ss.max(1);
        let (all_w, all_h) = (w * ss, h * ss);
        let (nx, ny) = (all_w.div_ceil(tile), all_h.div_ceil(tile));
        let mut pixels = vec![0.0f32; w as usize * h as usize * 4];
        let mut off = fractadyne_gpu::lsystem::Offscreen::new(device);
        let k = 1.0 / (ss * ss) as f32;
        for ty in 0..ny {
            for tx in 0..nx {
                if cancel.load(Relaxed) {
                    return Err("canceled".into());
                }
                let (ox, oy) = (tx * tile, ty * tile);
                let (tw, th) = ((all_w - ox).min(tile), (all_h - oy).min(tile));
                // The tile's texels are the whole target's, shifted: its view sits off-centre by the
                // tile's place (y up, texels to pixels).
                let tile = LSystemFrame {
                    offset: [
                        (0.5 * (all_w - tw) as f32 - ox as f32) / ss as f32,
                        -(0.5 * (all_h - th) as f32 - oy as f32) / ss as f32,
                    ],
                    ..frame.clone()
                };
                let values = off.render(device, queue, &tile, [tw, th], ss)?;
                for y in 0..th {
                    let row = ((oy + y) / ss) as usize * w as usize;
                    for x in 0..tw {
                        let c = self.colour(values[(y * tw + x) as usize]);
                        let i = (row + ((ox + x) / ss) as usize) * 4;
                        pixels[i] += c[0] * k;
                        pixels[i + 1] += c[1] * k;
                        pixels[i + 2] += c[2] * k;
                    }
                }
                progress.store(((ty * nx + tx + 1) * 1000 / (nx * ny)).min(1000), Relaxed);
            }
        }
        for a in pixels.iter_mut().skip(3).step_by(4) {
            *a = 1.0;
        }
        Ok(Rendered { width: w, height: h, pixels, stopped: out.stats.stopped, big: out.big, ex: out.ex })
    }

    /// A texel's colour: the screen colour pass's for an L-system.
    fn colour(&self, v: f32) -> [f32; 3] {
        if v < 0.0 {
            return self.interior;
        }
        let c = self.lut.sample(v + self.offset);
        [c[0], c[1], c[2]]
    }
}

/// The self-test's check of the tiling: Bourke's mango leaf (lines and fills) exported at an odd
/// size, 2× supersampled, in tiles of 64 texels and in one — the same picture, but for edge texels
/// a tile's shifted f32 arithmetic rounds the other way (a few in a thousand at most). A tile placed
/// wrong shifts everything it holds.
pub(crate) fn tiling_check(device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue) -> Result<String, String> {
    let system = library::find("Mango leaf").and_then(|e| e.system().ok()).ok_or("the mango leaf is in the library")?;
    let tables = Arc::new(Tables::new(&system));
    let order = system.order.unwrap_or(DEFAULT_ORDER);
    let b = lsystem::bounds(&tables, order, 1 << 21).ok_or("it draws")?;
    let size = [333u32, 221];
    let upp = ((b[2] - b[0]) / f64::from(size[0])).max((b[3] - b[1]) / f64::from(size[1])) * 1.1;
    let centre = [BigFloat::from_f64(0.5 * (b[0] + b[2]), 128), BigFloat::from_f64(0.5 * (b[1] + b[3]), 128)];
    let key = WalkKey { system: 1, tables: 1, order, centre, upp_log2: upp.log2(), size, colouring: Colouring::Depth, margin: 2.0, deep: false };
    let ramp = (0..256).map(|i| {
        let t = i as f32 / 255.0;
        [t, 0.5, 1.0 - t, 1.0]
    });
    let job = Job {
        system,
        tables,
        big: None,
        ex: None,
        key,
        depth_scale: 24.0,
        width: 1.5,
        progress: 1.0,
        ss: 2,
        lut: fractadyne_color::segment::Lut { entries: ramp.collect(), smooth: true },
        offset: 0.0,
        interior: [0.0, 0.0, 0.0],
    };
    let (p, c) = (AtomicU32::new(0), std::sync::atomic::AtomicBool::new(false));
    let small = job.render_in_tiles(device, queue, &p, &c, 64)?;
    let whole = job.render_in_tiles(device, queue, &p, &c, 1 << 14)?;
    let px = |r: &Rendered, i: usize| [r.pixels[4 * i], r.pixels[4 * i + 1], r.pixels[4 * i + 2]];
    let n = (size[0] * size[1]) as usize;
    let differ = (0..n).filter(|&i| px(&small, i).iter().zip(px(&whole, i)).any(|(a, b)| (a - b).abs() > 1e-5)).count();
    let lit = (0..n).filter(|&i| px(&whole, i) != [0.0, 0.0, 0.0]).count();
    let tiles = size[0].div_ceil(32) * size[1].div_ceil(32);
    if lit < n / 10 {
        Err(format!("only {lit} of {n} pixels drawn: too little to check"))
    } else if differ * 200 > n {
        Err(format!("{differ} of {n} pixels differ between {tiles} tiles and one"))
    } else {
        Ok(format!("{tiles} tiles = one, but for {differ} of {n} pixels on an edge; {lit} drawn"))
    }
}

impl FractadyneApp {
    /// The export of the L-system view at the Export dialog's size, aspect and supersampling.
    pub(crate) fn lsystem_export_job(&self) -> Job {
        let (w, h) = (self.export.width.max(1), self.export_height());
        let (vw, vh) = (self.viewport.width_px.max(1.0), self.viewport.height_px.max(1.0));
        // Contain (as `build_export_job`): the export's world span holds the window's, so a world
        // unit is `factor` times as many export pixels as screen pixels, inverted.
        let factor = vh.max(vw * f64::from(h) / f64::from(w)) / f64::from(h);
        self.lsystem_job([w, h], factor)
    }

    /// A tour frame (scripting): the viewport, already the frame's size, as an image — keeping the
    /// deep tables and built word it used for the next frame.
    pub(crate) fn lsystem_tour_frame(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        size: [u32; 2],
    ) -> Result<Rendered, String> {
        let (p, c) = (AtomicU32::new(0), std::sync::atomic::AtomicBool::new(false));
        let job = self.lsystem_job(size, 1.0);
        let r = job.render(device, queue, &p, &c)?;
        let st = &mut self.lsystem;
        if let Some(b) = &r.big {
            st.big = Some((job.key.tables, b.clone()));
        }
        if let Some(x) = &r.ex {
            st.expansion = Some((job.key.tables, job.key.order, x.clone()));
        }
        Ok(r)
    }

    /// The view walked at `size` pixels, a world unit `1 / factor` times as many of them as of the
    /// screen's (the line width scaled with it), coloured and supersampled as the Export dialog says.
    fn lsystem_job(&self, size: [u32; 2], factor: f64) -> Job {
        let [w, h] = size;
        let st = &self.lsystem;
        let upp_log2 = self.viewport.units_per_pixel.log2() + factor.log2();
        let order = st.order_at(&self.viewport);
        let width = (f64::from(st.width) / factor) as f32;
        let key = WalkKey {
            system: st.system_id,
            tables: st.tables_id,
            order,
            centre: [self.viewport.center_x.clone(), self.viewport.center_y.clone()],
            upp_log2,
            size: [w, h],
            colouring: st.colouring(),
            margin: 0.5 * width + 1.0,
            deep: st.needs_deep(upp_log2, order),
        };
        let (entries, smooth) = self.active_lut();
        let bg = self.interior_color();
        Job {
            system: st.drawn_system(),
            tables: st.tables.clone(),
            big: st.big.as_ref().filter(|(id, _)| *id == key.tables).map(|(_, b)| b.clone()),
            ex: st.expansion.as_ref().filter(|(id, o, _)| *id == key.tables && *o == key.order).map(|(_, _, x)| x.clone()),
            depth_scale: st.depth_scale(order),
            key,
            width,
            progress: st.progress,
            ss: self.export.ss.max(1),
            lut: fractadyne_color::segment::Lut { entries: entries.to_vec(), smooth },
            offset: self.coloring.offset,
            interior: [bg[0], bg[1], bg[2]],
        }
    }
}

