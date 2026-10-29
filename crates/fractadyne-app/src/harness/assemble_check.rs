//! The selftest's `assemble` group (`design/zoom-video.md` §7, phase E2's gate): a frame assembled
//! from strips is the frame a flat render makes, up to where the samples sit.
//!
//! **Geometry, exactly.** Synthetic strips whose every texel holds its own number: each sample of
//! an assembled frame must hold the number of the texel `ExpMap::locate` (the CPU definition) names
//! for it, at several depths, both supersamplings and a rotated frame; its distance estimate must be
//! rescaled from the texel's size to the sample spacing exactly as the geometry says, and its normal
//! turned with the frame. A sample is excused only within 1e-3 of a texel of a texel's edge, where
//! the shader's f32 and the CPU's f64 may fall either side.
//!
//! **In pieces.** The geometry check runs again with each strip held in three pieces around its
//! circle (`fractadyne_gpu::strip_pieces`, as a strip wider than the card's textures is held): every
//! sample must read the very texel it read before.
//!
//! **Only what the colouring reads** (`design/zoom-video.md` §8.6). A real dive's frames, coloured
//! six ways (smooth and distance colouring, each with and without glow and relief), from the whole
//! window and from one holding only what that colouring reads, in pieces: byte for byte the same.
//! The CONTROL: the count-only window's samples differ from the whole window's in the normal and
//! distance estimate it leaves out, and a colouring that reads them is refused on it.
//!
//! **Pictures, against a flat render, scored against a CONTROL.** A real dive's strips assembled at
//! several depths, against `render_iter` of the same frame. Sample for sample they differ wherever a
//! pixel is chaotic — a sample snapped to its texel's point moves by up to half a texel — so the
//! check is relative: the assembled frame must agree with the flat render at least as well as the
//! SAME flat render moved by a third of a pixel does. And the distance estimate must carry no bias
//! (a wrong texel-to-sample rescale is a constant offset).

use crate::assemble::placement;
use fractadyne_core::{add_floatexp, ExpMap, FloatExp, Viewport};
use fractadyne_gpu::{Assembler, StripShape};

/// A sample this close to a texel's edge (in texels) may fall either side of it.
const EDGE_TEXEL: f64 = 1.0e-3;

/// The widest piece the pieced geometry check holds its 346-texel strips in: three pieces, the last
/// narrower (116, 116, 114).
const GEOMETRY_PIECE_MAX: u32 = 120;
const GEOMETRY_PIECES: u32 = 3;

/// What the synthetic-strip check found.
pub(crate) struct GeometryCheck {
    pub(crate) frames: usize,
    /// The most pieces a strip was held in (3 for the pieced windows).
    pub(crate) pieces: u32,
    pub(crate) samples: usize,
    /// Within [`EDGE_TEXEL`] of an edge: either texel is right.
    pub(crate) excused: usize,
    /// Read another texel: each described.
    pub(crate) wrong: Vec<String>,
    /// The largest error of a sample's rescaled distance estimate (log2) and of its normal.
    pub(crate) de_err: f64,
    pub(crate) normal_err: f64,
}

impl GeometryCheck {
    pub(crate) fn pass(&self) -> bool {
        self.frames >= 16
            && self.pieces == GEOMETRY_PIECES
            && self.samples > 10_000
            && self.wrong.is_empty()
            && self.de_err < 1.0e-3
            && self.normal_err < 1.0e-5
    }
}

/// One depth of the flat comparison: the assembled frame against the flat render, and the control
/// (the flat render moved by a third of a pixel) against it.
pub(crate) struct FlatCompare {
    pub(crate) log2mag: f64,
    pub(crate) interior_agree: f64,
    pub(crate) control_interior_agree: f64,
    /// Escaped in both, smooth counts within 0.1 of an iteration.
    pub(crate) close: f64,
    pub(crate) control_close: f64,
    /// The median signed difference of log2 DE where the smooth counts agree within 0.05.
    pub(crate) de_bias: f64,
    /// The largest difference between the assembler's coloured frame and the export colour pass
    /// ([`fractadyne_gpu::color_iter_buffer_ss`]) run on the same assembled main target: 0 when
    /// an assembled frame is coloured exactly as any export is.
    pub(crate) colour_diff: f32,
}

impl FlatCompare {
    pub(crate) fn pass(&self) -> bool {
        self.interior_agree >= self.control_interior_agree - 0.002
            && self.close >= self.control_close - 0.01
            && self.de_bias.abs() < 0.01
            && self.colour_diff == 0.0
    }
}

/// `(interior agreement, share escaped-in-both within 0.1)` of `a` against `b` (main targets).
fn agreement(a: &[f32], b: &[f32]) -> (f64, f64) {
    let n = a.len() / 4;
    let (mut agree, mut both, mut close) = (0usize, 0usize, 0usize);
    for t in 0..n {
        let (sa, sb) = (a[t * 4], b[t * 4]);
        agree += ((sa < 0.0) == (sb < 0.0)) as usize;
        if sa >= 0.0 && sb >= 0.0 {
            both += 1;
            close += ((sa - sb).abs() < 0.1) as usize;
        }
    }
    (agree as f64 / n.max(1) as f64, close as f64 / both.max(1) as f64)
}

impl crate::FractadyneApp {
    /// The synthetic-strip geometry check (see the module note).
    pub(crate) fn assemble_geometry_check(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
    ) -> Result<GeometryCheck, String> {
        let (fw, fh) = (96u32, 54u32);
        let map = ExpMap::for_frames(fw as f64, fh as f64, 1.0, 0.0);
        let strips = 24u64;
        let (w, rows) = (map.width as usize, map.rows as usize);
        let id = |k: u64, i: usize, j: usize| (k as usize * w * rows + j * w + i) as f32 + 1.0;
        let shape = StripShape {
            width: map.width,
            rows: map.rows,
            rows_per_octave: map.rows_per_octave(),
            sample_offset: map.sample_offset(),
            row_step: map.row_step(),
            strips,
        };
        let o = map.sample_offset();
        let rpo = map.rows_per_octave();
        let mut c = GeometryCheck { frames: 0, pieces: 0, samples: 0, excused: 0, wrong: Vec::new(), de_err: 0.0, normal_err: 0.0 };
        for (ss, piece_max) in [(1u32, 0u32), (2, 0), (1, GEOMETRY_PIECE_MAX), (2, GEOMETRY_PIECE_MAX)] {
            let (sw, sh) = (fw * ss, fh * ss);
            let layout = fractadyne_gpu::WindowLayout { texels: fractadyne_gpu::WindowTexels::All, piece_max };
            let mut asm = Assembler::with_layout(device, shape, [fw, fh], ss, fractadyne_gpu::layers_for_frame(sw, sh), layout)
                .map_err(|e| e.to_string())?;
            c.pieces = c.pieces.max(asm.held().1);
            // The first frame (strip 0's corners), fractional depths, a frame past the dive's end
            // (clamped to its last strip), and a rotated frame.
            for (l, turns) in [(0.0, 0.0), (2.37, 0.0), (7.5, 0.0), (11.91, 0.137), (18.25, 0.0), (21.0, 0.0)] {
                let place = placement(&map, l, fh as f64, ss, turns);
                let (first, last) = asm.strips_for(&place).ok_or("a frame outside the dive")?;
                for k in first..=last {
                    let mut main = vec![0f32; w * rows * 4];
                    for t in 0..w * rows {
                        main[t * 4] = id(k, t % w, t / w);
                        main[t * 4 + 1] = 1.0; // a normal along +re
                    }
                    asm.put_strip(queue, k, &main).map_err(|e| e.to_string())?;
                }
                let got = asm.reprojected(device, queue, &place).map_err(|e| e.to_string())?;
                c.frames += 1;
                let phi = turns * std::f64::consts::TAU;
                for y in 0..sh as usize {
                    for x in 0..sw as usize {
                        let s = &got[(y * sw as usize + x) * 4..][..4];
                        let dx = x as f64 + 0.5 - sw as f64 / 2.0;
                        let dy = sh as f64 / 2.0 - (y as f64 + 0.5);
                        let depth = place.depth_whole as f64 + place.depth_frac - 0.5 * (dx * dx + dy * dy).max(0.25).log2();
                        let t = dy.atan2(dx) / std::f64::consts::TAU + turns;
                        let Some((mut k, i, mut j)) = map.locate(depth, t) else {
                            if s[0] != -2.0 {
                                c.wrong.push(format!("ss {ss}, 2^{l}: sample ({x}, {y}) outside the dive read {}", s[0]));
                            }
                            continue;
                        };
                        if k >= strips {
                            (k, j) = (strips - 1, (rows - 1) as u32);
                        }
                        c.samples += 1;
                        let want = id(k, i as usize, j as usize);
                        if s[0] != want {
                            // Near a texel's edge either texel is right.
                            let v = (depth - depth.floor()) * rpo + 0.5 - o;
                            let u = t.rem_euclid(1.0) * w as f64 + 0.5 - o;
                            let edge = |p: f64| (p - p.round()).abs() < EDGE_TEXEL;
                            if edge(v) || edge(u) || (depth - depth.round()).abs() * rpo < EDGE_TEXEL {
                                c.excused += 1;
                            } else if c.wrong.len() < 8 {
                                c.wrong.push(format!("ss {ss}, 2^{l}: sample ({x}, {y}) read texel #{} for #{want} ({k}, {i}, {j})", s[0]));
                            } else {
                                c.wrong.push(String::new());
                            }
                            continue;
                        }
                        // The distance estimate: 0 in the texel's size, so log2(texel / sample).
                        let de = (place.depth_whole - k as i64) as f64 + place.depth_frac - (j as f64 + o) / rpo + map.row_step().log2();
                        c.de_err = c.de_err.max((s[3] as f64 - de).abs());
                        // The normal: +re in the plane, turned by −φ on the screen.
                        let n_err = (s[1] as f64 - phi.cos()).abs().max((s[2] as f64 + phi.sin()).abs());
                        c.normal_err = c.normal_err.max(n_err);
                    }
                }
            }
        }
        Ok(c)
    }

    /// A real dive into `(cx, cy)`, frames of `frame` pixels from 1× to `2^end`, every strip at
    /// `cap`: assembled at each of `depths` against `render_iter` of the same frame, with the
    /// third-of-a-pixel control (see the module note). Mandelbrot: the caller sets the fractal.
    pub(crate) fn assemble_flat_check(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        cx: &str,
        cy: &str,
        frame: (u32, u32),
        end: f64,
        cap: u32,
        depths: &[f64],
    ) -> Result<Vec<FlatCompare>, String> {
        let (fw, fh) = frame;
        let map = ExpMap::for_frames(fw as f64, fh as f64, 1.0, 0.0);
        let strips = map.strips_to(end, fh as f64);
        let p0 = fractadyne_core::precision_for_octaves(strips + 64);
        let cxb = fractadyne_core::parse_bf_prec(cx, p0).ok_or("unreadable centre")?;
        let cyb = fractadyne_core::parse_bf_prec(cy, p0).ok_or("unreadable centre")?;
        let saved = (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.export.width, self.export.ss);
        let result = (|| -> Result<Vec<FlatCompare>, String> {
            // The strips, as a dive renders them: on its one reference, at one cap.
            let dive_ref = self.dive_reference(&map, strips - 1, &cxb, &cyb, cap);
            let mut rendered = Vec::with_capacity(strips as usize);
            for k in 0..strips {
                rendered.push(self.render_strip(device, queue, &map, k, &cxb, &cyb, cap, dive_ref.as_ref())?.result.pixels);
            }
            let shape = StripShape {
                width: map.width,
                rows: map.rows,
                rows_per_octave: map.rows_per_octave(),
                sample_offset: map.sample_offset(),
                row_step: map.row_step(),
                strips,
            };
            let mut asm = Assembler::new(device, shape, [fw, fh], 1, fractadyne_gpu::layers_for_frame(fw, fh)).map_err(|e| e.to_string())?;
            self.render_cfg.max_iter = cap;
            self.render_cfg.auto_iter = false;
            self.export.width = fw;
            self.export.ss = 1;
            let mut out = Vec::new();
            for &l in depths {
                let place = placement(&map, l, fh as f64, 1, 0.0);
                let (first, last) = asm.strips_for(&place).ok_or("a frame outside the dive")?;
                for k in first..=last {
                    if !asm.has_strip(k) {
                        asm.put_strip(queue, k, &rendered[k as usize]).map_err(|e| e.to_string())?;
                    }
                }
                let assembled = asm.reprojected(device, queue, &place).map_err(|e| e.to_string())?;
                let flat_at = |app: &mut Self, shift: (f64, f64)| -> Result<Vec<f32>, String> {
                    let mut vp = Viewport::new(fw as f64, fh as f64);
                    let p = fractadyne_core::precision_for_octaves(l.max(0.0) as u64 + 64);
                    // A shift of (sx, sy) pixels: upp = REFERENCE_HEIGHT / (h·2^l).
                    let upp = FloatExp::from_f64(Viewport::REFERENCE_HEIGHT / fh as f64).mul_pow2(-l);
                    let x = add_floatexp(&cxb, upp.mul_f64(shift.0), p);
                    let y = add_floatexp(&cyb, upp.mul_f64(shift.1), p);
                    vp.set_center_log2mag(x, y, l);
                    let req = app.current_export_request_for(&vp, false);
                    let r = fractadyne_gpu::render_iter(device, queue, &req).map_err(|e| e.to_string())?;
                    if (r.width, r.height) != (fw, fh) {
                        return Err(format!("the flat render came back {}x{}", r.width, r.height));
                    }
                    Ok(r.pixels)
                };
                let flat = flat_at(self, (0.0, 0.0))?;
                let control = flat_at(self, (0.3, 0.2))?;
                let (interior_agree, close) = agreement(&assembled, &flat);
                let (control_interior_agree, control_close) = agreement(&control, &flat);
                let mut dde: Vec<f64> = (0..assembled.len() / 4)
                    .filter(|&t| assembled[t * 4] >= 0.0 && flat[t * 4] >= 0.0 && (assembled[t * 4] - flat[t * 4]).abs() < 0.05)
                    .map(|t| (assembled[t * 4 + 3] - flat[t * 4 + 3]) as f64)
                    .filter(|d| d.is_finite())
                    .collect();
                dde.sort_by(|a, b| a.total_cmp(b));
                let de_bias = dde.get(dde.len() / 2).copied().unwrap_or(f64::NAN);
                // The colouring: the assembler's own pass against the export's on the same samples.
                let colour = self.current_export_request_for(&Viewport::new(fw as f64, fh as f64), false);
                let mine = asm.frame(device, queue, &place, &colour).map_err(|e| e.to_string())?;
                let theirs = fractadyne_gpu::color_iter_buffer_ss(device, queue, &colour, &assembled, 1).map_err(|e| e.to_string())?;
                let colour_diff = mine.iter().zip(&theirs.pixels).map(|(a, b)| (a - b).abs()).fold(0.0f32, f32::max);
                out.push(FlatCompare { log2mag: l, interior_agree, control_interior_agree, close, control_close, de_bias, colour_diff });
            }
            Ok(out)
        })();
        (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.export.width, self.export.ss) = saved;
        result
    }

    /// The window check (see the module note): a real dive into `(cx, cy)`, 160x90 frames from 1x
    /// to 2^12 at `cap`, assembled at four depths (one turned) and coloured six ways.
    pub(crate) fn assemble_window_check(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        cx: &str,
        cy: &str,
        cap: u32,
    ) -> Result<WindowCheck, String> {
        use fractadyne_gpu::{WindowLayout, WindowTexels};
        let (fw, fh) = (160u32, 90u32);
        let map = ExpMap::for_frames(fw as f64, fh as f64, 1.0, 0.0);
        let strips = map.strips_to(12.0, fh as f64);
        let p0 = fractadyne_core::precision_for_octaves(strips + 64);
        let cxb = fractadyne_core::parse_bf_prec(cx, p0).ok_or("unreadable centre")?;
        let cyb = fractadyne_core::parse_bf_prec(cy, p0).ok_or("unreadable centre")?;
        let saved = (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.export.width);
        let result = (|| -> Result<WindowCheck, String> {
            let dive_ref = self.dive_reference(&map, strips - 1, &cxb, &cyb, cap);
            let mut rendered = Vec::with_capacity(strips as usize);
            for k in 0..strips {
                rendered.push(self.render_strip(device, queue, &map, k, &cxb, &cyb, cap, dive_ref.as_ref())?.result.pixels);
            }
            let shape = StripShape {
                width: map.width,
                rows: map.rows,
                rows_per_octave: map.rows_per_octave(),
                sample_offset: map.sample_offset(),
                row_step: map.row_step(),
                strips,
            };
            let layers = fractadyne_gpu::layers_for_frame(fw, fh);
            let make = |layout: WindowLayout| Assembler::with_layout(device, shape, [fw, fh], 1, layers, layout).map_err(|e| e.to_string());
            let fill = |asm: &mut Assembler, place: &fractadyne_gpu::FramePlacement| -> Result<(), String> {
                let (first, last) = asm.strips_for(place).ok_or("a frame outside the dive")?;
                for k in first..=last {
                    if !asm.has_strip(k) {
                        asm.put_strip(queue, k, &rendered[k as usize]).map_err(|e| e.to_string())?;
                    }
                }
                Ok(())
            };
            let places: Vec<_> = [(0.0, 0.0), (3.3, 0.0), (7.9, 0.21), (12.0, 0.0)]
                .iter()
                .map(|&(l, turns)| placement(&map, l, fh as f64, 1, turns))
                .collect();
            let base_colour = self.current_export_request_for(&Viewport::new(fw as f64, fh as f64), false);
            let mut c = WindowCheck { frames: 0, pixels: 0, differ: Vec::new(), pieces: 0, left_out_differ: 0, count_differ: 0, refused: false };
            let mut whole = make(WindowLayout::default())?;
            // Smooth (0) and distance (4) colouring, with and without glow and relief.
            for (method, light, glow) in [(0u32, false, false), (0, false, true), (0, true, false), (4, false, false), (4, false, true), (4, true, true)] {
                let mut colour = base_colour.clone();
                (colour.color_method, colour.light, colour.de_on) = (method, light as u32, glow as u32);
                // A glow that shows, so a wrong distance estimate would.
                colour.de_strength = 0.8;
                let texels = WindowTexels::for_colour(method, light, glow);
                let mut held = make(WindowLayout { texels, piece_max: WINDOW_PIECE_MAX })?;
                c.pieces = c.pieces.max(held.held().1);
                let mut differ = 0usize;
                for place in &places {
                    fill(&mut whole, place)?;
                    fill(&mut held, place)?;
                    let a = whole.frame(device, queue, place, &colour).map_err(|e| e.to_string())?;
                    let b = held.frame(device, queue, place, &colour).map_err(|e| e.to_string())?;
                    differ += a.chunks_exact(4).zip(b.chunks_exact(4)).filter(|(x, y)| x.iter().zip(y.iter()).any(|(p, q)| p.to_bits() != q.to_bits())).count();
                    c.frames += 1;
                    c.pixels += a.len() / 4;
                }
                if differ > 0 {
                    c.differ.push(format!("method {method}, relief {light}, glow {glow} ({texels:?}): {differ} pixels"));
                }
            }
            // The control: the count-only window really leaves the rest out, and a colouring that
            // reads it is refused there.
            let mut count = make(WindowLayout { texels: WindowTexels::Count, piece_max: WINDOW_PIECE_MAX })?;
            for place in &places {
                fill(&mut whole, place)?;
                fill(&mut count, place)?;
                let a = whole.reprojected(device, queue, place).map_err(|e| e.to_string())?;
                let b = count.reprojected(device, queue, place).map_err(|e| e.to_string())?;
                for (x, y) in a.chunks_exact(4).zip(b.chunks_exact(4)) {
                    c.count_differ += (x[0].to_bits() != y[0].to_bits()) as usize;
                    c.left_out_differ += (x[0] >= 0.0 && x[1..].iter().zip(&y[1..]).any(|(p, q)| p.to_bits() != q.to_bits())) as usize;
                }
            }
            let mut relief = base_colour.clone();
            relief.light = 1;
            c.refused = count.frame(device, queue, &places[1], &relief).is_err();
            Ok(c)
        })();
        (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.export.width) = saved;
        result
    }
}

/// The widest piece the window check holds its 577-texel strips in: three pieces (193, 193, 191).
const WINDOW_PIECE_MAX: u32 = 200;

/// What the window check found.
pub(crate) struct WindowCheck {
    pub(crate) frames: usize,
    pub(crate) pixels: usize,
    /// Each colouring whose frames differed between the windows, described (none may).
    pub(crate) differ: Vec<String>,
    /// The most pieces a strip was held in.
    pub(crate) pieces: u32,
    /// The control: escaped samples whose normal or distance estimate differ between the whole and
    /// the count-only window (must be many), and samples whose count does (must be none).
    pub(crate) left_out_differ: usize,
    pub(crate) count_differ: usize,
    /// A relief colouring was refused on the count-only window.
    pub(crate) refused: bool,
}

impl WindowCheck {
    pub(crate) fn pass(&self) -> bool {
        self.frames == 24
            && self.differ.is_empty()
            && self.pieces == 3
            && self.left_out_differ > 1_000
            && self.count_differ == 0
            && self.refused
    }
}
