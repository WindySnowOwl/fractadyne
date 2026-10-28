//! The selftest's `expmap` group (`design/zoom-video.md` §7, phase E0's gate): a strip computes, at
//! every texel, what an INDEPENDENT arbitrary-precision iteration computes at the point the CPU
//! geometry (`fractadyne_core::ExpMap`) says that texel is. The oracle (`naive_dwell_bf`) knows
//! nothing of references, series, BLA or the shader's polar placement, so an error in any of them —
//! a flipped axis, a half-texel shift, a radius off by a factor, a wrong count or fraction in the
//! strip's own channels — shows as texels that disagree.
//!
//! ⭐It scores a CONTROL: the same samples against the oracle a quarter turn away. A location that
//! looks the same rotated would let a wrong placement pass, so a control that mostly AGREES makes
//! the check vacuous, and it says so rather than passing.
//! ⭐A mismatch is excused only when the oracle's own answer changes within ±`STENCIL_TEXEL` of the
//! texel: the GPU places a strip sample to ~1e-4 of a texel (f32), so there either answer is right.
//! Both layouts are judged: Fractadyne's conformal strips (sampled at texel centres) and the
//! octave strips the zoomasm format writes (sampled at texel corners, as Kalles Fraktaler does) —
//! a sample half a texel from where the geometry says is a wrong texel, not an excused one.

use fractadyne_core::{add_floatexp, naive_dwell_bf, ExpMap, StripRows};

/// The oracle stencil, in texels: well above the GPU's placement error, well below a texel.
const STENCIL_TEXEL: f64 = 1.0e-3;
/// The smooth fraction agrees within this where the count does. The GPU's is f32 from the f32
/// head of the final z, and the sample point itself is good to ~1e-4 texel.
pub(crate) const NF_TOL: f64 = 0.05;
/// The control must disagree on at least this share of the samples for the check to mean anything.
pub(crate) const CONTROL_MIN: f64 = 0.3;
/// And at least this share must agree outright: a strip whose samples all landed where the oracle
/// cannot settle (and are excused) has not been checked.
pub(crate) const MATCHED_MIN: f64 = 0.6;
/// A distance estimate at or below this (log2 texels) is deep in a boundary, where the direct
/// path's f32 derivative cannot give a normal (see the normal check).
const DEEP_BOUNDARY_DE_LOG2: f32 = -30.0;

/// What one strip's comparison found.
pub(crate) struct StripCheck {
    /// The request's arithmetic mode (1 direct, 0 df32 perturbation, 2 floatexp).
    pub(crate) mode: u32,
    /// The request had a BLA table (the df32 band's strips render as floatexp with it off).
    pub(crate) bla: bool,
    /// The strip rendered on the dive's reference (a perturbation strip must: it is what a dive does).
    pub(crate) reused: bool,
    pub(crate) width: u32,
    pub(crate) rows: u32,
    /// Escaped texels compared, and how many agreed exactly (count and fraction).
    pub(crate) sampled: usize,
    pub(crate) matched: usize,
    /// Disagreed, but the oracle's own answer changes within the stencil: excused.
    pub(crate) sensitive: usize,
    /// Disagreed where the oracle is stable: each described.
    pub(crate) wrong: Vec<String>,
    /// Texels the GPU called interior, checked; and those the oracle saw escape.
    pub(crate) interior: usize,
    pub(crate) interior_wrong: usize,
    /// The largest fraction error where the counts agreed, and that texel described.
    pub(crate) nf_err: f64,
    pub(crate) nf_worst: String,
    /// Escaped texels whose normal is not unit length, and the first described.
    pub(crate) normals_bad: usize,
    pub(crate) normal_worst: String,
    /// Escaped texels deep in a boundary (a distance estimate of 2^-30 texels or less), where the
    /// direct path's f32 derivative gives no normal. Counted, not failed.
    pub(crate) derivative_overflow: usize,
    /// Of `sampled`, how many the quarter-turn control disagreed on.
    pub(crate) control_differ: usize,
    /// Every tile took the chunked path (a strip must not mix entry points).
    pub(crate) chunked_all: bool,
}

impl StripCheck {
    pub(crate) fn pass(&self) -> bool {
        self.sampled >= 16
            && (self.mode == 1 || self.reused)
            && self.matched as f64 >= MATCHED_MIN * self.sampled as f64
            && self.wrong.is_empty()
            && self.interior_wrong == 0
            && self.nf_err <= NF_TOL
            && self.normals_bad == 0
            && self.chunked_all
            && self.control_differ as f64 >= CONTROL_MIN * self.sampled as f64
    }
}

impl crate::FractadyneApp {
    /// Render strip `k` of a dive into `(cx, cy)` (frames of `frame` pixels starting at 1×, strips
    /// in `layout`) at `cap` iterations, and compare up to `samples` escaped texels (and a few
    /// interior ones) with the oracle. Mandelbrot: the caller sets the fractal.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn expmap_strip_check(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        cx: &str,
        cy: &str,
        frame: (f64, f64),
        layout: StripRows,
        k: u64,
        cap: u32,
        samples: usize,
    ) -> Result<StripCheck, String> {
        let p0 = fractadyne_core::precision_for_octaves(k + 64);
        let cxb = fractadyne_core::parse_bf_prec(cx, p0).ok_or("unreadable centre")?;
        let cyb = fractadyne_core::parse_bf_prec(cy, p0).ok_or("unreadable centre")?;
        let map = match layout {
            StripRows::Conformal => ExpMap::for_frames(frame.0, frame.1, 1.0, 0.0),
            StripRows::Octave => ExpMap::for_frames(frame.0, frame.1, 1.0, 0.0).with_octave_rows(),
        };
        // As a dive renders it: on the dive's one reference, built here four strips deeper (finer
        // precision, a radius 16× smaller) — so the reuse, and the series and BLA worked out again
        // for THIS strip's radius, are what the oracle judges.
        let dive_ref = self.dive_reference(&map, k + 4, &cxb, &cyb, cap);
        let s = self.render_strip(device, queue, &map, k, &cxb, &cyb, cap, dive_ref.as_ref())?;
        let (r, req) = (s.result, s.req);
        let p = crate::expmap_render::strip_view(&map, k, &cxb, &cyb).precision + 64;
        let w = map.width as usize;
        let at = |i: f64, j: f64| -> Option<(u32, f32)> {
            let (re, im) = map.texel_offset(k, i, j);
            naive_dwell_bf(&add_floatexp(&cxb, re, p), &add_floatexp(&cyb, im, p), cap, 256.0 * 256.0, p)
        };
        // A spread of texels over the whole strip, every row and column region.
        let stride = ((w * map.rows as usize) / (samples * 6)).max(1);
        let mut c = StripCheck {
            mode: req.mode,
            bla: req.bla_on != 0,
            reused: s.reused,
            width: map.width,
            rows: map.rows,
            sampled: 0,
            matched: 0,
            sensitive: 0,
            wrong: Vec::new(),
            interior: 0,
            interior_wrong: 0,
            nf_err: 0.0,
            nf_worst: String::new(),
            normals_bad: 0,
            normal_worst: String::new(),
            derivative_overflow: 0,
            control_differ: 0,
            chunked_all: r.tiles_total > 0 && r.tiles_chunked == r.tiles_total,
        };
        let mut t = 7usize; // not 0: row 0 column 0 is one point on the strip's edge
        while t < w * map.rows as usize && (c.sampled < samples || c.interior < 4) {
            let (i, j) = ((t % w) as f64, (t / w) as f64);
            let m = &r.pixels[t * 4..t * 4 + 4];
            let a = &r.aux[t * 4..t * 4 + 4];
            t += stride;
            if m[0] >= 0.0 {
                if c.sampled >= samples {
                    continue;
                }
                c.sampled += 1;
                let n_gpu = (a[0] as u32) << 12 | a[1] as u32;
                let nf_gpu = a[2] as f64;
                // The normal is a unit vector wherever the derivative is representable. The direct
                // path's derivative is an f32 pair: deep in a boundary (a distance estimate of 2^-30
                // texels and less) it, or the product the normal is formed from, overflows and the
                // normal has no direction — as in any direct render; the floatexp paths keep it.
                if !(m[3] > DEEP_BOUNDARY_DE_LOG2) {
                    c.derivative_overflow += 1;
                } else if ((m[1] * m[1] + m[2] * m[2]) - 1.0).abs() > 1e-3 {
                    if c.normals_bad == 0 {
                        c.normal_worst = format!("({i}, {j}): ({}, {}), smooth {}, DE 2^{}", m[1], m[2], m[0], m[3]);
                    }
                    c.normals_bad += 1;
                }
                let o = at(i, j);
                // The control: the same row, a quarter turn round.
                let ctl = at((i + w as f64 / 4.0) % w as f64, j);
                if ctl.map(|x| x.0) != Some(n_gpu) {
                    c.control_differ += 1;
                }
                match o {
                    Some((n, smooth)) if n == n_gpu && (nf_gpu - (smooth as f64 - n as f64)).abs() <= NF_TOL => {
                        let err = (nf_gpu - (smooth as f64 - n as f64)).abs();
                        if err > c.nf_err {
                            c.nf_err = err;
                            c.nf_worst = format!("({i}, {j}) n {n}: NF gpu {nf_gpu:.4}, oracle {:.4}", smooth as f64 - n as f64);
                        }
                        c.matched += 1;
                    }
                    _ => {
                        // Is the GPU's smooth count the oracle's somewhere within the stencil? Next to
                        // the boundary (a distance estimate of a fraction of a texel) the final |z|,
                        // and with it the fraction, moves a lot in 1e-4 of a texel; and where the
                        // oracle escapes at some stencil points and not others, so may the GPU.
                        let s_gpu = n_gpu as f64 + nf_gpu;
                        let d = STENCIL_TEXEL;
                        let around: Vec<Option<(u32, f32)>> = std::iter::once(o)
                            .chain([(d, 0.0), (-d, 0.0), (0.0, d), (0.0, -d)].iter().map(|&(di, dj)| at(i + di, j + dj)))
                            .collect();
                        let s: Vec<f64> = around.iter().filter_map(|x| x.map(|(_, v)| v as f64)).collect();
                        let (lo, hi) = s.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &v| (a.min(v), b.max(v)));
                        // The oracle's COUNT changes within the stencil (or it escapes at some points
                        // and not others): the texel is chaotic at the scale the GPU places samples.
                        let unsettled = around.iter().any(|x| x.map(|v| v.0) != o.map(|v| v.0));
                        if unsettled || (s_gpu >= lo - NF_TOL && s_gpu <= hi + NF_TOL) {
                            c.sensitive += 1;
                        } else if c.wrong.len() < 6 {
                            c.wrong.push(format!(
                                "({i}, {j}): gpu {s_gpu:.4}, oracle {:?} (within the stencil {lo:.4}..{hi:.4})",
                                o.map(|x| x.1)
                            ));
                        } else {
                            c.wrong.push(String::new());
                        }
                    }
                }
            } else if m[0] > -1.5 && c.interior < 4 {
                c.interior += 1;
                if at(i, j).is_some() {
                    c.interior_wrong += 1;
                }
            }
        }
        Ok(c)
    }
}
