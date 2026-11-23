//! The selftest's `video-equalize` group (coloring plan C6b): a zoom video equalizes its frames by
//! distributions read from its strips' block summaries, blended between octaves.
//!
//! **The reading, against the frame's own.** A real dive's strips (the seahorse, 640x360 frames to
//! 2^20), each summarized as a video summarizes it (8 values a block, by area): at every half
//! octave, the distribution `frame_quantiles` reads against the exact one of the frame the
//! assembler makes there (its main target's every escaped sample), as Equalize colors' remap over
//! the frame's window, in palette sweeps, under the linear and the log map. The CONTROL: the same
//! frames read from the blocks' means alone (what the summary held before), which must be visibly
//! worse, or the comparison sees nothing.
//!
//! **The colors, against their definition.** Between two octaves (halfway, where the blend is
//! furthest from either anchor's), the frame the assembler colors with the video's own map
//! (`apply_video_norm`, `apply_video_eq`, the anchors' blends `NormAnchors::range`) against the CPU's
//! reading of the palette at each escaped sample's equalized position, and with the share at 0 as
//! the control; and the property it is for: each of 16 colors covers about the same share of the
//! frame.

use crate::assemble::{frame_blocks, frame_quantiles, frame_range, placement, strip_range_ranked, NormAnchors, Seen, EQ_ANCHOR_STEP};
use fractadyne_core::coloring::{eq_knots, eq_remap, equalized, quantiles_of_weighted, Quantiles, Transfer, EQ_KNOTS};
use fractadyne_core::{ExpMap, Viewport};
use fractadyne_gpu::{Assembler, StripShape};
use std::collections::HashMap;
use std::sync::Arc;

/// The reading at one depth: Equalize colors' remap from it against the frame's own, in palette
/// sweeps, the largest over the window and the mean over the frame's escaped samples (the worse of
/// the linear and the log map), and the control's largest.
pub(crate) struct ReadingAt {
    pub(crate) log2mag: f64,
    pub(crate) max: f64,
    pub(crate) mean: f64,
    pub(crate) control_max: f64,
    /// Milliseconds the reading took (`frame_quantiles`).
    pub(crate) ms: f64,
}

/// The colors at one depth under one curve.
pub(crate) struct ColorsAt {
    pub(crate) log2mag: f64,
    pub(crate) transfer: Transfer,
    /// The linear-to-log dissolve's weight there (0 or 1 for the CPU's reading to apply).
    pub(crate) log_weight: f32,
    /// Escaped samples more than one 8-bit level from the CPU's colors, the largest gap, and the
    /// control's share (the share at 0).
    pub(crate) apart: f64,
    pub(crate) worst: f64,
    pub(crate) control: f64,
    /// The fullest of 16 colors' share of the window's samples: equalized as the video does, by the
    /// frame's own exact distribution (the ideal), and fitted.
    pub(crate) fullest_eq: f64,
    pub(crate) fullest_own: f64,
    pub(crate) fullest_fit: f64,
}

/// Equalize colors' remap error of `est` against `exact` over the window `[lo, top]`, in sweeps:
/// `(largest, mean over the escaped samples of main)`, the worse of the linear and the log map.
fn remap_error(est: &Quantiles, exact: &Quantiles, lo: f32, top: f32, sweeps: f64, main: &[f32]) -> (f64, f64) {
    let (mut worst, mut mean) = (0.0f64, 0.0f64);
    for t in [Transfer::Linear, Transfer::Log] {
        let (ke, kx) = (eq_knots(est, lo as f64, top as f64, t), eq_knots(exact, lo as f64, top as f64, t));
        worst = worst.max((0..=EQ_KNOTS).map(|j| (ke[j] - kx[j]).abs() as f64).fold(0.0, f64::max) * sweeps);
        let span = t.apply((top - lo) as f64);
        let (mut s, mut n) = (0.0f64, 0usize);
        for px in main.chunks_exact(4) {
            let v = px[0] as f64;
            if v >= 0.0 {
                let u = (t.apply((v - lo as f64).max(0.0)) / span).min(1.0);
                s += (eq_remap(&ke, u) - eq_remap(&kx, u)).abs() * sweeps;
                n += 1;
            }
        }
        mean = mean.max(s / n.max(1) as f64);
    }
    (worst, mean)
}

impl crate::FractadyneApp {
    /// Both halves of the check (see the module note) on one dive into `(cx, cy)`: `frame` pixels
    /// from 1x to `2^end`, every strip at `cap`; the reading at each of `depths`, the colors at each
    /// of `mids` under each of `curves`. The caller sets the coloring (smooth, point sampled, the
    /// share and cycle) and restores it.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn video_equalize_check(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        (cx, cy): (&str, &str),
        frame: (u32, u32),
        end: f64,
        cap: u32,
        depths: &[f64],
        mids: &[f64],
        curves: &[Transfer],
    ) -> Result<(Vec<ReadingAt>, Vec<ColorsAt>), String> {
        let (fw, fh) = frame;
        let map = ExpMap::for_frames(fw as f64, fh as f64, 1.0, 0.0);
        let strips = map.strips_to(end, fh as f64);
        let p0 = fractadyne_core::precision_for_octaves(strips + 64);
        let cxb = fractadyne_core::parse_bf_prec(cx, p0).ok_or("unreadable center")?;
        let cyb = fractadyne_core::parse_bf_prec(cy, p0).ok_or("unreadable center")?;
        let saved = (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.export.width, self.export.ss, self.coloring.transfer);
        let result = (|| -> Result<(Vec<ReadingAt>, Vec<ColorsAt>), String> {
            let dive_ref = self.dive_reference(&map, strips - 1, &cxb, &cyb, cap);
            let mut rendered = Vec::with_capacity(strips as usize);
            for k in 0..strips {
                rendered.push(self.render_strip(device, queue, &map, k, &cxb, &cyb, cap, dive_ref.as_ref())?.result.pixels);
            }
            let (w, rows) = (map.width as usize, map.rows as usize);
            let ranges: HashMap<u64, Arc<crate::assemble::StripRange>> = rendered
                .iter()
                .enumerate()
                .map(|(k, p)| (k as u64, Arc::new(strip_range_ranked(p, w, rows, Some(map.rows_per_octave())))))
                .collect();
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
            let sweeps = 0.5 + self.coloring.cycle as f64 * 6.0;
            let q_top = crate::tunables::cost().norm_top_quantile;
            let (fwf, fhf) = (fw as f64, fh as f64);
            let at = |asm: &mut Assembler, l: f64| -> Result<Vec<f32>, String> {
                let place = placement(&map, l, fhf, 1, 0.0);
                let (first, last) = asm.strips_for(&place).ok_or("a frame outside the dive")?;
                for k in first..=last {
                    if !asm.has_strip(k) {
                        asm.put_strip(queue, k, &rendered[k as usize]).map_err(|e| e.to_string())?;
                    }
                }
                asm.reprojected(device, queue, &place).map_err(|e| e.to_string())
            };
            // The reading, against the frame's own.
            let mut readings = Vec::new();
            for &l in depths {
                let main = at(&mut asm, l)?;
                let exact = crate::render::buffer_quantiles(&main).ok_or("nothing escaped")?;
                let (lo, top) = frame_range(&map, &ranges, strips, l, fwf, fhf, q_top)?.ok_or("no window")?;
                let t0 = std::time::Instant::now();
                let est = frame_quantiles(&map, &ranges, strips, l, fwf, fhf)?.ok_or("no reading")?;
                let ms = t0.elapsed().as_secs_f64() * 1000.0;
                // The control: the blocks' means alone, each weighted by its area.
                let (mut clo, mut chi, mut atoms) = (f32::INFINITY, f32::NEG_INFINITY, Vec::new());
                frame_blocks(&map, &ranges, strips, l, fwf, fhf, false, |seen| {
                    if let Seen::Block(r, b, rad) = seen {
                        clo = clo.min(r.lo[b]);
                        chi = chi.max(r.hi[b]);
                        atoms.push((r.mean[b], r.n[b] as f64 * rad * rad));
                    }
                })?;
                let control = quantiles_of_weighted(&mut atoms, clo, chi).ok_or("no control reading")?;
                let (max, mean) = remap_error(&est, &exact, lo, top, sweeps, &main);
                let (control_max, _) = remap_error(&control, &exact, lo, top, sweeps, &main);
                readings.push(ReadingAt { log2mag: l, max, mean, control_max, ms });
            }
            // The colors, against their definition.
            let mut colors = Vec::new();
            for &t in curves {
                self.coloring.transfer = t;
                // As the video anchors them: ranges every octave, distributions every quarter.
                let mut anchors = NormAnchors::new(0.0, end);
                let mut eq_anchors: NormAnchors<Quantiles> = NormAnchors::every(0.0, end, EQ_ANCHOR_STEP);
                for &l in mids {
                    let (lo, top) = anchors.range(l, |la| frame_range(&map, &ranges, strips, la, fwf, fhf, q_top))?.ok_or("nothing escaped at the anchors")?;
                    let q = eq_anchors.range(l, |la| frame_quantiles(&map, &ranges, strips, la, fwf, fhf))?.ok_or("the anchors hold no distribution")?;
                    let main = at(&mut asm, l)?;
                    let mut c = self.current_export_request_for(&Viewport::new(fwf, fhf), false);
                    c.aa_palette = fractadyne_gpu::PaletteAa::Off;
                    self.apply_video_norm(&mut c, lo, top);
                    self.apply_video_eq(&mut c, lo, top, &q);
                    let log_weight = crate::assemble::log_weight(lo, top, t != Transfer::Linear);
                    let place = placement(&map, l, fhf, 1, 0.0);
                    let got = asm.frame(device, queue, &place, &c).map_err(|e| e.to_string())?;
                    // The map as the request carries it: the curve of its own form, from its floor.
                    let curve = if c.norm_mode == 0 { Transfer::Linear } else { t };
                    let eq = c.equalize.ok_or("the request is not equalized")?;
                    let (share, eq_sweeps) = (eq.share as f64, eq.sweeps as f64);
                    let x_of = |v: f32| -> f64 {
                        let d = (v - c.norm_lo) as f64;
                        (if curve == Transfer::Linear { d } else { curve.apply(d.max(0.0)) }) * c.cycle as f64
                    };
                    let pos = |v: f32, share: f64| equalized(x_of(v), share, eq_sweeps, &eq.knots) + c.offset as f64;
                    let own = crate::render::buffer_quantiles(&main).ok_or("nothing escaped")?;
                    let own_knots = eq_knots(&own, lo as f64, top as f64, curve);
                    let pos_own = |v: f32| equalized(x_of(v), share, eq_sweeps, &own_knots);
                    let (lut, smooth) = (c.lut.clone(), c.lut_smooth);
                    let read = |p: f64| -> [f64; 3] {
                        let n = lut.len();
                        let rgb = |e: [f32; 4]| [e[0] as f64, e[1] as f64, e[2] as f64];
                        let tt = p - p.floor();
                        if !smooth {
                            return rgb(lut[((tt * n as f64) as usize).min(n - 1)]);
                        }
                        let x = tt * n as f64 - 0.5;
                        let i0 = x.floor() as i64;
                        let (a, b) = (rgb(lut[i0.rem_euclid(n as i64) as usize]), rgb(lut[(i0 + 1).rem_euclid(n as i64) as usize]));
                        let f = x - x.floor();
                        [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f, a[2] + (b[2] - a[2]) * f]
                    };
                    let (mut n, mut bad, mut worst, mut cbad) = (0usize, 0usize, 0.0f64, 0usize);
                    let (mut bins_eq, mut bins_own, mut bins_fit, mut inside) = ([0usize; 16], [0usize; 16], [0usize; 16], 0usize);
                    for (px, m) in got.chunks_exact(4).zip(main.chunks_exact(4)) {
                        let v = m[0];
                        if v < 0.0 {
                            continue;
                        }
                        let off = |want: [f64; 3]| (0..3).map(|k| (px[k] as f64 - want[k]).abs() * 255.0).fold(0.0, f64::max);
                        let d = off(read(pos(v, share)));
                        n += 1;
                        worst = worst.max(d);
                        bad += (d > 1.0) as usize;
                        cbad += (off(read(pos(v, 0.0))) > 1.0) as usize;
                        // Equal areas over the window's samples, a whole number of sweeps.
                        if x_of(v) <= eq_sweeps {
                            inside += 1;
                            for (bins, p) in [(&mut bins_eq, pos(v, share) - c.offset as f64), (&mut bins_own, pos_own(v)), (&mut bins_fit, pos(v, 0.0) - c.offset as f64)] {
                                bins[((p - p.floor()) * 16.0).floor().clamp(0.0, 15.0) as usize] += 1;
                            }
                        }
                    }
                    let fullest = |b: &[usize; 16]| b.iter().copied().max().unwrap_or(0) as f64 / inside.max(1) as f64;
                    colors.push(ColorsAt {
                        log2mag: l,
                        transfer: t,
                        log_weight,
                        apart: bad as f64 / n.max(1) as f64,
                        worst,
                        control: cbad as f64 / n.max(1) as f64,
                        fullest_eq: fullest(&bins_eq),
                        fullest_own: fullest(&bins_own),
                        fullest_fit: fullest(&bins_fit),
                    });
                }
            }
            Ok((readings, colors))
        })();
        (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.export.width, self.export.ss, self.coloring.transfer) = saved;
        result
    }
}
