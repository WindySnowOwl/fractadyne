//! Render-request builders — the performance-critical bridge from app state to the GPU.
//! Picks/caches the arbitrary-precision reference orbit, computes the series-approximation
//! skip, and assembles the live (`MandelbrotParams`) and offscreen (`ExportRequest`) jobs,
//! choosing the direct / df32-perturbation / floatexp render mode by depth.

use crate::{profile, zoom_iter_cap, FractadyneApp, FractalKind, PERT_FE_THRESHOLD, WORK_BUDGET};
use fractadyne_core::Viewport;
use fractadyne_gpu::MandelbrotParams;
use std::time::Instant;

/// BLA per-step linear tolerance (drops δz² with relative error ≤ this). Smaller ⇒ more
/// accurate but fewer/smaller skips; 1e-6 keeps pixel error negligible while still merging.
const BLA_EPS: f64 = 1.0e-6;

/// A completed reference recompute (orbit + series-approximation + BLA), ready to install into a
/// view's reference cache. Produced by [`recompute_worker`], off the render thread, so the slow
/// deep-zoom bignum work never blocks a frame.
pub(crate) struct RecomputeResult {
    orbit: std::sync::Arc<Vec<[f32; 4]>>,
    orbit_len: u32,
    rp: [fractadyne_core::BigFloat; 2],
    sa: fractadyne_core::SeriesSkip,
    bla: std::sync::Arc<Vec<[f32; 4]>>,
    bla_dc_max_log2: f64,
    prec: usize,
    iter: u32,
    ref_ms: f64,
}

/// Owned, `Send` inputs for an off-thread reference recompute.
struct RecomputeInputs {
    center_bf: [fractadyne_core::BigFloat; 2],
    span: (fractadyne_core::FloatExp, fractadyne_core::FloatExp),
    span_mantissa: [f64; 2],
    delta_exp: i32,
    gpu_iter: u32,
    precision: usize,
    julia: bool,
    formula: u32,
    julia_c: (f64, f64),
    do_sa: bool,
    bla_dc_max: Option<fractadyne_core::FloatExp>,
}

/// Pick a reference, iterate its orbit, and compute the series-approximation skip + BLA tree — the
/// slow arbitrary-precision work. Pure and `Send`, so it runs on a worker thread; mirrors the
/// synchronous `compute_reference` + `series_skip_for` + `build_bla`.
fn recompute_worker(inp: RecomputeInputs) -> RecomputeResult {
    use fractadyne_core as fc;
    let t = Instant::now();
    let rp = fc::best_reference(
        &inp.center_bf,
        [inp.span.0, inp.span.1],
        inp.formula,
        inp.julia,
        [inp.julia_c.0, inp.julia_c.1],
        inp.gpu_iter,
        inp.precision,
    );
    let zero = fc::BigFloat::from_f64(0.0, inp.precision);
    let (z0x, z0y, cx0, cy0) = if inp.julia {
        (
            rp[0].clone(),
            rp[1].clone(),
            fc::BigFloat::from_f64(inp.julia_c.0, inp.precision),
            fc::BigFloat::from_f64(inp.julia_c.1, inp.precision),
        )
    } else {
        (zero.clone(), zero, rp[0].clone(), rp[1].clone())
    };
    let (o, len) = fc::reference_orbit(&z0x, &z0y, &cx0, &cy0, inp.formula, inp.gpu_iter, inp.precision);
    let orbit = std::sync::Arc::new(o);
    let ref_ms = t.elapsed().as_secs_f64() * 1000.0;
    // Series approximation for the chosen reference.
    let sa = if inp.do_sa {
        let dx = fc::ref_offset_mantissa(&inp.center_bf[0], &rp[0], inp.delta_exp, inp.precision);
        let dy = fc::ref_offset_mantissa(&inp.center_bf[1], &rp[1], inp.delta_exp, inp.precision);
        let roff = (dx * dx + dy * dy).sqrt();
        let half_diag = 0.5
            * (inp.span_mantissa[0] * inp.span_mantissa[0] + inp.span_mantissa[1] * inp.span_mantissa[1]).sqrt();
        let log2_max_dc = inp.delta_exp as f64 + (roff + half_diag).max(1e-300).log2();
        fc::series_skip(&rp[0], &rp[1], log2_max_dc, inp.gpu_iter, len, inp.formula, inp.precision)
    } else {
        fc::SeriesSkip::NONE
    };
    // BLA tree (Mandelbrot deep only; `None` otherwise). Built with the same conservative dc_max
    // the live path uses so the main thread reuses it across pans.
    let (bla, bla_dc_max_log2) = match inp.bla_dc_max {
        Some(dc_max) => {
            let levels = fc::build_bla_mandel(&orbit, dc_max, BLA_EPS);
            let arc = if levels.is_empty() {
                std::sync::Arc::new(Vec::new())
            } else {
                std::sync::Arc::new(fc::bla_to_gpu(&levels))
            };
            (arc, dc_max.log2())
        }
        None => (std::sync::Arc::new(Vec::new()), f64::NEG_INFINITY),
    };
    RecomputeResult {
        orbit,
        orbit_len: len,
        rp,
        sa,
        bla,
        bla_dc_max_log2,
        prec: inp.precision,
        iter: inp.gpu_iter,
        ref_ms,
    }
}

impl FractadyneApp {
    /// Install a finished recompute into view `vi`'s reference cache (bumps `orbit_id`, refreshes
    /// SA + BLA), and record the recompute cost. Called on the main thread for both the sync
    /// (cold-start) path and completed async jobs.
    fn install_recompute(&mut self, vi: usize, res: RecomputeResult) {
        let vc = &mut self.ref_cache[vi];
        vc.ref_pt = Some(res.rp);
        vc.orbit = res.orbit;
        vc.orbit_len = res.orbit_len;
        vc.orbit_prec = res.prec;
        vc.orbit_iter = res.iter;
        vc.orbit_id = vc.orbit_id.wrapping_add(1);
        vc.last_recompute = Some(Instant::now());
        vc.sa = res.sa;
        vc.sa_key = (vc.orbit_id, res.iter);
        vc.bla = res.bla;
        vc.bla_id = vc.orbit_id;
        vc.bla_dc_max_log2 = res.bla_dc_max_log2;
        self.perf.recompute_ms = res.ref_ms;
        self.perf.recompute_total += 1;
        self.perf.rate_count += 1;
    }
    /// Pick a reference point and compute its high-precision orbit for the current
    /// formula, arranging `Z₀`/`c` for Mandelbrot vs Julia mode. Returns the orbit,
    /// its length, and the chosen reference point (for the δ-offset).
    pub(crate) fn compute_reference(
        &self,
        center_bf: &[fractadyne_core::BigFloat; 2],
        span: (fractadyne_core::FloatExp, fractadyne_core::FloatExp),
        eff_iter: u32,
        precision: usize,
        julia: bool,
        ref_override: Option<[fractadyne_core::BigFloat; 2]>,
    ) -> (
        std::sync::Arc<Vec<[f32; 4]>>,
        u32,
        [fractadyne_core::BigFloat; 2],
    ) {
        let formula = self.fractal.formula_id();
        let (jcx, jcy) = self.julia_c;
        // A correct perturbation render is invariant to which valid in-view reference is
        // used; `ref_override` lets the validator force a specific reference (Phase 1.2).
        let rp = ref_override.unwrap_or_else(|| {
            fractadyne_core::best_reference(
                center_bf,
                [span.0, span.1],
                formula,
                julia,
                [jcx, jcy],
                eff_iter,
                precision,
            )
        });
        let zero = fractadyne_core::BigFloat::from_f64(0.0, precision);
        let (z0x, z0y, cx0, cy0) = if julia {
            (
                rp[0].clone(),
                rp[1].clone(),
                fractadyne_core::BigFloat::from_f64(jcx, precision),
                fractadyne_core::BigFloat::from_f64(jcy, precision),
            )
        } else {
            (zero.clone(), zero, rp[0].clone(), rp[1].clone())
        };
        let (o, l) =
            fractadyne_core::reference_orbit(&z0x, &z0y, &cx0, &cy0, formula, eff_iter, precision);
        (std::sync::Arc::new(o), l, rp)
    }

    /// Series-approximation skip for the current view, or `NONE` when not applicable. Both
    /// perturbation paths benefit — df32 (`mode 0`) and floatexp (`mode 2`) — for
    /// **Mandelbrot** with a non-aux coloring method; the direct path (`mode 1`) and Julia
    /// iterate from 0. (The coefficients are mode-independent; only the GPU seed differs.)
    #[allow(clippy::too_many_arguments)]
    fn series_skip_for(
        &self,
        ref_pt: &[fractadyne_core::BigFloat; 2],
        span_mantissa: [f64; 2],
        ref_dx: f64,
        ref_dy: f64,
        delta_exp: i32,
        mode: u32,
        julia: bool,
        eff_iter: u32,
        orbit_len: u32,
        precision: usize,
    ) -> fractadyne_core::SeriesSkip {
        // SA applies to the holomorphic polynomial families: Mandelbrot (0) and
        // Multibrot 3/4/5 (1/2/3). Tricorn (anti-holomorphic) and the abs families don't
        // have this δc expansion.
        if !self.series_approx
            || mode == 1
            || julia
            || self.fractal.formula_id() > 3
            || fractadyne_gpu::method_needs_aux(self.color_method)
        {
            return fractadyne_core::SeriesSkip::NONE;
        }
        // Worst-case corner |δc| = |center − reference| + half the view diagonal, taken in
        // log2 so it never underflows (both are mantissas sharing the 2^delta_exp scale).
        let roff = (ref_dx * ref_dx + ref_dy * ref_dy).sqrt();
        let half_diag =
            0.5 * (span_mantissa[0] * span_mantissa[0] + span_mantissa[1] * span_mantissa[1]).sqrt();
        let log2_max_dc = delta_exp as f64 + (roff + half_diag).max(1e-300).log2();
        fractadyne_core::series_skip(
            &ref_pt[0],
            &ref_pt[1],
            log2_max_dc,
            eff_iter,
            orbit_len,
            self.fractal.formula_id(),
            precision,
        )
    }

    /// Build the BLA tree (GPU-flattened) for a Mandelbrot deep view, or `None` when BLA
    /// doesn't apply (disabled, not floatexp/Mandelbrot/non-Julia, or an aux coloring method
    /// that BLA would skip). `dx`/`dy` are the reference-offset mantissas and `span_mantissa`
    /// the view span — both scaled by `2^delta_exp` — used for the worst-case `|δc|`.
    /// Whether BLA applies to this render (deep floatexp Mandelbrot, non-Julia, non-aux coloring).
    fn bla_eligible(&self, mode: u32, julia: bool) -> bool {
        self.use_bla
            && mode == 2
            && !julia
            && self.fractal.formula_id() == 0
            && !fractadyne_gpu::method_needs_aux(self.color_method)
    }

    /// Conservative worst-case `|δc|` (absolute, `·2^delta_exp`) for any pixel a reference serves:
    /// the view half-diagonal plus the drift the reference stays valid over (recomputed past ~1.5
    /// spans). Deliberately **independent of the current center offset**, so the BLA tree built
    /// with it is valid for every pixel across pans within one reference — letting it be cached per
    /// `orbit_id` instead of rebuilt each frame. A larger `dc_max` only shrinks the skip radii
    /// (safer, never wrong); the few skips lost vs. a per-frame-tight bound are bought back many
    /// times over by not rebuilding the tree every frame.
    fn bla_dc_max(span_mantissa: [f64; 2], delta_exp: i32) -> fractadyne_core::FloatExp {
        let diag = (span_mantissa[0] * span_mantissa[0] + span_mantissa[1] * span_mantissa[1]).sqrt();
        fractadyne_core::FloatExp::from_f64((2.5 * diag).max(1e-300)).mul_pow2(delta_exp as f64)
    }

    /// Build the BLA tree (GPU-packed) for a reference orbit + worst-case `|δc|`. `None` if BLA
    /// produced no usable levels. Eligibility (`bla_eligible`) is the caller's gate.
    fn build_bla(
        &self,
        orbit: &[[f32; 4]],
        dc_max: fractadyne_core::FloatExp,
    ) -> Option<std::sync::Arc<Vec<[f32; 4]>>> {
        let levels = fractadyne_core::build_bla_mandel(orbit, dc_max, BLA_EPS);
        if levels.is_empty() {
            return None;
        }
        Some(std::sync::Arc::new(fractadyne_core::bla_to_gpu(&levels)))
    }

    /// Build an export request for a given viewport + Julia flag at the export
    /// resolution. Recomputes a fresh reference orbit (deep) without touching the live
    /// cache. Height is derived from the viewport's aspect (square pixels).
    pub(crate) fn current_export_request_for(
        &self,
        vp: &Viewport,
        julia: bool,
    ) -> fractadyne_gpu::ExportRequest {
        let log2mag = vp.log2_magnification();
        let width = self.export_width.max(1);
        // height from aspect: span_y/span_x = height_px/width_px (the scale cancels).
        let height = ((width as f64) * vp.height_px / vp.width_px).round().max(1.0) as u32;
        let mag = vp.magnification(); // saturates to ∞ past 1e308×; fine for the mode compares
        let eff_iter = if self.auto_iter {
            vp.recommended_max_iter(self.max_iter)
        } else {
            self.max_iter
        }
        // Cap at the zoom-appropriate count (same as the live view): avoids noise from
        // over-resolving sub-pixel dust, and keeps the export fast/responsive.
        .min(zoom_iter_cap(log2mag).max(256));
        let mode: u32 = if !self.fractal.supports_perturbation() || mag < 1.0e4 {
            1
        } else if mag >= PERT_FE_THRESHOLD {
            2
        } else {
            0
        };
        let precision = vp.precision; // maintained by the viewport; valid at any depth
        let (cx, cy) = vp.center_f64();
        let scale = vp.gpu_scale();
        let delta_exp = scale.delta_exp;

        let mut ref_offset = [0.0_f32; 4];
        let mut orbit = std::sync::Arc::new(Vec::new());
        let mut orbit_len = 0u32;
        let mut sa = fractadyne_core::SeriesSkip::NONE;
        let mut bla = std::sync::Arc::new(Vec::new());
        let mut bla_on = 0u32;
        if mode != 1 {
            let center_bf = [vp.center_x.clone(), vp.center_y.clone()];
            let t_ref = std::time::Instant::now();
            let (orbit_arc, len, rp) =
                self.compute_reference(&center_bf, vp.complex_span_fe(), eff_iter, precision, julia, None);
            let reference_ms = t_ref.elapsed().as_secs_f64() * 1000.0;
            orbit = orbit_arc;
            orbit_len = len;
            let dx = fractadyne_core::ref_offset_mantissa(&vp.center_x, &rp[0], delta_exp, precision);
            let dy = fractadyne_core::ref_offset_mantissa(&vp.center_y, &rp[1], delta_exp, precision);
            let dxh = dx as f32;
            let dyh = dy as f32;
            ref_offset = [dxh, dyh, (dx - dxh as f64) as f32, (dy - dyh as f64) as f32];
            let t_sa = std::time::Instant::now();
            sa = self.series_skip_for(&rp, scale.span_mantissa, dx, dy, delta_exp, mode, julia, eff_iter, len, precision);
            let series_ms = t_sa.elapsed().as_secs_f64() * 1000.0;
            let t_bla = std::time::Instant::now();
            if self.bla_eligible(mode, julia) {
                let dc_max = Self::bla_dc_max(scale.span_mantissa, delta_exp);
                if let Some(data) = self.build_bla(&orbit, dc_max) {
                    bla = data;
                    bla_on = 1;
                }
            }
            let bla_ms = t_bla.elapsed().as_secs_f64() * 1000.0;
            self.prof.set(profile::ProfSetup { reference_ms, series_ms, bla_ms });
        }

        let cxh = cx as f32;
        let cyh = cy as f32;
        let center = [cxh, cyh, (cx - cxh as f64) as f32, (cy - cyh as f64) as f32];
        let (jcx, jcy) = self.julia_c;
        let jcxh = jcx as f32;
        let jcyh = jcy as f32;
        let julia_c = [jcxh, jcyh, (jcx - jcxh as f64) as f32, (jcy - jcyh as f64) as f32];
        let (stops, stop_count) = self.active_stops();

        fractadyne_gpu::ExportRequest {
            width,
            height,
            ss: self.export_ss.max(1),
            span_mantissa: scale.span_mantissa,
            center,
            ref_offset,
            delta_exp,
            sa_skip: sa.skip,
            glitch_on: 0, // enabled per-pass by the multi-reference correction path
            watermark: self.watermark as u32,
            sa_a: sa.a,
            sa_a_exp: sa.a_exp,
            sa_b: sa.b,
            sa_b_exp: sa.b_exp,
            sa_c: sa.c,
            sa_c_exp: sa.c_exp,
            julia_c,
            orbit,
            orbit_len,
            bla,
            bla_on,
            max_iter: eff_iter,
            mode,
            formula: self.fractal.formula_id(),
            julia: julia as u32,
            cycle: self.color_cycle(),
            offset: self.offset,
            stop_count,
            stops,
            light: self.light as u32,
            light_angle: self.light_angle,
            light_height: self.light_height,
            de_on: self.de as u32,
            de_strength: self.de_strength,
            de_width: self.de_width,
            de_phase: self.de_phase,
            color_method: self.color_method,
            stripe_freq: self.stripe_freq,
            trap_type: self.trap_type,
            aa_filter: 1,
            interior_col: self.interior_color(),
        }
    }

    /// Render the iteration buffer with **multi-reference glitch correction** (offscreen). Renders
    /// with the base reference (Pauldelbrot flagging on), then repeatedly drops a fresh reference
    /// into the largest remaining glitched region, re-renders, and adopts the now-correct pixels —
    /// until nothing is glitched or `max_refs` is hit. Returns the merged raw RGBA32F iteration
    /// buffer (`w*h*4`) plus `(references_used, residual_glitches)`. Single-texture (bounded by the
    /// GPU max dim); the caller colors the result. Perturbation modes only (direct has no glitches).
    pub(crate) fn render_corrected_iter(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        vp: &Viewport,
        julia: bool,
        width: u32,
        height: u32,
        max_refs: usize,
    ) -> Option<(Vec<f32>, usize, usize)> {
        let mut req = self.current_export_request_for(vp, julia);
        req.width = width;
        req.height = height;
        req.ss = 1;
        req.glitch_on = 1;
        let mut merged = fractadyne_gpu::render_iter(device, queue, &req).ok()?.pixels;
        // Direct path (mode 1) never glitches; nothing to correct.
        if req.mode == 1 {
            return Some((merged, 1, 0));
        }
        let center_bf = [vp.center_x.clone(), vp.center_y.clone()];
        let span = vp.complex_span_fe();
        let precision = vp.precision;
        let eff_iter = req.max_iter;
        let delta_exp = req.delta_exp;
        let (w, h) = (width as usize, height as usize);
        let mut refs_used = 1usize;

        for _ in 1..max_refs {
            // Glitched pixels carry the -2 sentinel (r < -1.5); interior is -1, escaped ≥ 0.
            let glitch: Vec<usize> = (0..w * h).filter(|&i| merged[i * 4] < -1.5).collect();
            if glitch.is_empty() {
                break;
            }
            // New reference at the glitched pixel nearest the region's centroid.
            let (mut sx, mut sy) = (0.0f64, 0.0f64);
            for &i in &glitch {
                sx += (i % w) as f64;
                sy += (i / w) as f64;
            }
            let n = glitch.len() as f64;
            let (cxp, cyp) = (sx / n, sy / n);
            let seed = *glitch
                .iter()
                .min_by(|&&a, &&b| {
                    let da = ((a % w) as f64 - cxp).powi(2) + ((a / w) as f64 - cyp).powi(2);
                    let db = ((b % w) as f64 - cxp).powi(2) + ((b / w) as f64 - cyp).powi(2);
                    da.partial_cmp(&db).unwrap()
                })
                .unwrap();
            // Bignum coordinate of that export pixel's CENTER (the +0.5 matches the shader's texel
            // center), mapped into the viewport's pixel space. Getting this exact makes δc = 0 at
            // the seed pixel, so it renders exactly and can't glitch against its own reference —
            // which guarantees each pass resolves at least the seed and the loop converges.
            let vpx = ((seed % w) as f64 + 0.5) * (vp.width_px / w as f64);
            let vpy = ((seed / w) as f64 + 0.5) * (vp.height_px / h as f64);
            let (rx, ry) = vp.pixel_to_complex(vpx, vpy);
            let (orbit, len, rp) =
                self.compute_reference(&center_bf, span, eff_iter, precision, julia, Some([rx, ry]));
            let dx = fractadyne_core::ref_offset_mantissa(&center_bf[0], &rp[0], delta_exp, precision);
            let dy = fractadyne_core::ref_offset_mantissa(&center_bf[1], &rp[1], delta_exp, precision);
            let (dxh, dyh) = (dx as f32, dy as f32);

            // Re-reference pass: fresh orbit, no SA/BLA (they were built for the base reference).
            let mut r = req.clone();
            r.orbit = orbit;
            r.orbit_len = len;
            r.ref_offset = [dxh, dyh, (dx - dxh as f64) as f32, (dy - dyh as f64) as f32];
            r.sa_skip = 0;
            r.bla_on = 0;
            r.bla = std::sync::Arc::new(Vec::new());
            let pass = fractadyne_gpu::render_iter(device, queue, &r).ok()?.pixels;
            refs_used += 1;
            // Adopt pixels this reference resolved (no longer glitched).
            for &i in &glitch {
                if pass[i * 4] >= -1.5 {
                    merged[i * 4..i * 4 + 4].copy_from_slice(&pass[i * 4..i * 4 + 4]);
                }
            }
        }
        // Residual glitches after the final pass (0 = fully corrected).
        let residual = (0..w * h).filter(|&i| merged[i * 4] < -1.5).count();
        Some((merged, refs_used, residual))
    }

    /// Full glitch-corrected offscreen render → colored image. Runs multi-reference correction on
    /// the iteration buffer, then colors it. Returns `None` (caller falls back to a normal export)
    /// when the size exceeds the GPU's single-texture limit — tiling is future work — or the
    /// coloring method needs per-orbit aux statistics the merged buffer can't carry.
    pub(crate) fn render_export_corrected(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        vp: &Viewport,
        julia: bool,
        width: u32,
        height: u32,
    ) -> Option<fractadyne_gpu::ExportResult> {
        let max_dim = device.limits().max_texture_dimension_2d;
        if width > max_dim || height > max_dim || fractadyne_gpu::method_needs_aux(self.color_method) {
            return None;
        }
        let (buf, _refs, _residual) =
            self.render_corrected_iter(device, queue, vp, julia, width, height, 64)?;
        let mut req = self.current_export_request_for(vp, julia);
        req.width = width;
        req.height = height;
        fractadyne_gpu::color_iter_buffer(device, queue, &req, &buf).ok()
    }

    /// Build the GPU params for one fractal view, computing the perturbation
    /// reference (deep Mandelbrot) or selecting the direct df32 path. Shared by the
    /// single view and both panels of the dual view.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build_params(
        &mut self,
        center_bf: [fractadyne_core::BigFloat; 2],
        center: (f64, f64),
        span: (fractadyne_core::FloatExp, fractadyne_core::FloatExp),
        magnification: f64,
        log2mag: f64,
        fractal: FractalKind,
        julia: bool,
        eff_iter: u32,
        interacting: bool,
        resolution: [u32; 2],
        view_id: u32,
        // Some(uv_offset) → pan reprojection: reuse the cached orbit + frozen iteration texture
        // and translate it in the color pass (no bignum recompute, no re-iterate). Only honoured
        // at deep zoom (mode ≠ 1) once a reference exists for this view.
        reproject: Option<[f32; 2]>,
    ) -> MandelbrotParams {
        let (stops, stop_count) = self.active_stops();
        let (cx, cy) = center;
        // Extended-range scale → shared base-2 exponent + O(1) span mantissas, so nothing
        // underflows/overflows past ~1e308× (the per-pixel δ stays O(1); the GPU re-applies
        // the exponent). `span.0`/`span.1` are FloatExp, valid at any depth.
        let delta_exp = if span.0.m == 0.0 { 0 } else { span.0.log2().floor() as i32 };
        let sm = -(delta_exp as f64);
        let span_mantissa = [span.0.mul_pow2(sm).to_f64(), span.1.mul_pow2(sm).to_f64()];

        // Bound per-frame GPU work so a single render can't trip the OS GPU watchdog
        // (TDR ≈ 2 s → device-lost crash). Work ≈ texels × iterations = px·ss²·iter.
        //
        // The key balance: a huge iteration count at deep zoom on a large window resolves
        // the boundary's sub-pixel "dust" into per-pixel noise once it starves the budget
        // of resolution/anti-aliasing. So cap the live iteration count at what's affordable
        // at *native* resolution — but never below a zoom-appropriate floor, so deep
        // interiors stay resolved (clamping iterations with no floor was the old
        // uniform-screen bug). Only when even that floor can't fit (extreme depth on a very
        // large window) do we fall back to reducing the iteration-texture resolution.
        let px = (resolution[0].max(1) as u64) * (resolution[1].max(1) as u64);
        // Work budget is *interaction-aware*, the iteration count is NOT. While moving we spend a
        // tight budget (settle spends ~6×); the smaller budget shrinks the iteration-texture
        // resolution (blurrier motion, sharp on settle) via `res_scale` below. Crucially the
        // iteration COUNT stays the zoom-appropriate one in both states: a hard motion cap (was
        // 50k) starves deep views of the iterations needed to escape — past ~1e420× everything
        // reads as interior and the moving frame goes solid black. `zoom_iter_cap` already bounds
        // the count so it never over-resolves sub-pixel "dust"; let resolution, not iterations,
        // absorb the motion budget. (Full-iter deep frames are cheap — reduced res keeps them fast.)
        let (budget, iter_cap): (u64, u32) = if interacting {
            (WORK_BUDGET, 500_000)
        } else {
            (WORK_BUDGET.saturating_mul(6), 500_000)
        };
        let gpu_iter = eff_iter.min(iter_cap).min(zoom_iter_cap(log2mag).max(256));
        // Build the reference orbit a bit longer than the pixels need (`zoom_iter_cap` grows 256
        // iters/octave). The spare length lets one reference serve ~32 more octaves of zoom before
        // its orbit is too short — so a continuous dive doesn't rebuild the (slow, bignum) orbit
        // every single octave. Pixels still only iterate to `gpu_iter`; the tail is pure headroom.
        let ref_build_iter = gpu_iter.saturating_add(32 * 256).min(iter_cap);
        // GPU-watchdog safety: if even the capped work won't fit the budget, reduce the
        // iteration-texture resolution (the color pass box-filters the upscale).
        let want = px.saturating_mul(gpu_iter.max(1) as u64);
        let res_scale = if want > budget {
            (budget as f64 / want as f64).sqrt()
        } else {
            1.0
        };
        let resolution = if res_scale < 1.0 {
            [
                ((resolution[0] as f64 * res_scale) as u32).max(16),
                ((resolution[1] as f64 * res_scale) as u32).max(16),
            ]
        } else {
            resolution
        };
        let spx = (resolution[0] as u64) * (resolution[1] as u64);
        let max_ss = ((budget / spx.saturating_mul(gpu_iter.max(1) as u64).max(1)) as f64)
            .sqrt()
            .max(1.0) as u32;
        let ss = if interacting { 1 } else { self.aa.min(max_ss) };
        // Color-pass anti-aliasing when true supersampling wasn't affordable: widen the box
        // to match an upscaled (resolution-reduced) texture, or apply a gentle 2× box when
        // the budget forced ss=1 on a settled view the user wanted anti-aliased.
        let aa_filter = if res_scale < 1.0 {
            ((1.0 / res_scale).round() as u32).clamp(2, 4)
        } else if ss == 1 && self.aa > 1 && !interacting {
            2
        } else {
            1
        };

        // Render path: 1 = direct df32 (shallow / unsupported formulas), 0 = df32
        // perturbation (fast, common deep range), 2 = floatexp perturbation (past df32's
        // ~1e30× exponent limit → unlimited depth, ~1.7× costlier so only when needed).
        let mode: u32 = if !fractal.supports_perturbation() || magnification < 1.0e4 {
            1
        } else if magnification >= PERT_FE_THRESHOLD {
            2
        } else {
            0
        };
        let precision = fractadyne_core::precision_for_octaves(log2mag.max(0.0).ceil() as u64);
        let vi = view_id as usize;

        // Honour a pan reprojection only at deep zoom (the shallow direct path is cheap + already
        // detailed) and only once a reference orbit exists to have produced the frozen texture.
        // Mutable: also set below to freeze the last good frame while an off-thread recompute is in
        // flight and the cached reference has drifted fully out of view (extreme depth), so the
        // view holds instead of flashing blank.
        let mut reproject = reproject
            .filter(|_| mode != 1 && self.ref_cache[vi].ref_pt.is_some());
        // Reprojection scale about the view centre: 1.0 for a pan (drag) reprojection; set <1.0 by
        // the freeze below to zoom the held frame as the view keeps diving (zoom-reprojection).
        let mut reproject_scale = 1.0_f32;

        let mut ref_offset = [0.0_f32; 4];
        let mut sa = fractadyne_core::SeriesSkip::NONE;
        let mut bla = std::sync::Arc::new(Vec::new());
        let mut bla_on = 0u32;
        if mode != 1 && reproject.is_none() {
            // Install a finished off-thread recompute (if any) before deciding whether another is
            // needed, so the staleness/quality checks below see the fresh reference.
            if let Some(rx) = self.recompute_rx[vi].as_ref() {
                match rx.try_recv() {
                    Ok(res) => {
                        self.install_recompute(vi, res);
                        self.recompute_rx[vi] = None;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => self.recompute_rx[vi] = None,
                    Err(std::sync::mpsc::TryRecvError::Empty) => {}
                }
            }
            // Drift = |center − reference| / span, both as 2^-delta_exp mantissas so the
            // ratio is exact at any depth (raw f64 differences underflow past ~1e308×).
            let drift = self.ref_cache[vi].ref_pt.as_ref().map(|r| {
                let dx = fractadyne_core::ref_offset_mantissa(&center_bf[0], &r[0], delta_exp, precision)
                    / span_mantissa[0];
                let dy = fractadyne_core::ref_offset_mantissa(&center_bf[1], &r[1], delta_exp, precision)
                    / span_mantissa[1];
                (dx.abs(), dy.abs())
            });
            // Recomputing the reference orbit is a slow bignum job. `best_reference`
            // legitimately sits up to ~0.4 span off-center, so we must NOT treat that
            // as stale (doing so caused a per-frame recompute loop). When settled we
            // recompute whenever the reference left the view or precision/iters grew.
            //
            // During motion we used to defer entirely (until "gone"), which left a
            // stale/out-of-view/low-precision reference → soft "impressionist" frames
            // while zooming. Now we ALSO refresh during motion when the reference is
            // out of view or under-precise, but **throttled** (≤ ~1 recompute / 90 ms)
            // so the bignum cost doesn't stall every frame — keeping deep zoom sharp
            // without tanking the frame-rate. (Affordable since the release build made
            // bignum ~8× faster.)
            let out_of_view = drift.map_or(true, |(dx, dy)| dx > 0.5 || dy > 0.5);
            // Once the reference is well outside the view (≫ the ~0.9 span best_reference normally
            // sits at), the perturbation δc is large enough to render wrong/glitchy. On a fast/deep
            // dive the async recompute can lag this far — freeze the last clean frame rather than
            // paint the bad one (see the reprojection freeze below). Kept conservative so normal
            // deep motion (reference merely drifting, still usable) isn't needlessly held.
            let too_stale = drift.map_or(false, |(dx, dy)| dx > 1.5 || dy > 1.5);
            // While moving, tolerate the reference lagging in precision (it has 64 guard bits, good
            // for ~40 more octaves) so we don't rebuild the slow bignum orbit every octave — that
            // was the "zoom, pause, zoom" stepping on a deep dive. On settle we rebuild at the first
            // bit of lag so the still frame is at full precision. The orbit's iteration headroom
            // (`ref_build_iter`) covers the matching depth range, so iters rarely force a rebuild.
            let prec_headroom = if interacting { 32 } else { 0 };
            let needs_quality = precision > self.ref_cache[vi].orbit_prec + prec_headroom
                || gpu_iter > self.ref_cache[vi].orbit_iter;
            // Refresh whenever the reference has left the view or the depth/iters outgrew it. The
            // old motion throttle is gone: the recompute is now off-thread and gated to one job per
            // view (`recompute_rx`), so spawns are naturally paced by the compute itself — no need
            // to space them artificially (which only made deep references refresh more slowly).
            let recompute = out_of_view || needs_quality;
            // Whether the series approximation applies to this view (bundled into the recompute).
            let do_sa = (mode == 0 || mode == 2)
                && !julia
                && fractal.formula_id() <= 3
                && !fractadyne_gpu::method_needs_aux(self.color_method)
                && self.series_approx;
            if recompute {
                // The recompute (reference orbit + SA + BLA, all bignum) is the deep-zoom stall.
                // Run it OFF the render thread: keep drawing with the cached reference and install
                // the result when it lands (polled above). Only the very first reference (nothing
                // cached to draw with) is computed synchronously. Build to `ref_build_iter` (a bit
                // past what the pixels need) so the orbit serves a range of depths without rebuild.
                let inputs = RecomputeInputs {
                    center_bf: center_bf.clone(),
                    span,
                    span_mantissa,
                    delta_exp,
                    gpu_iter: ref_build_iter,
                    precision,
                    julia,
                    formula: self.fractal.formula_id(),
                    julia_c: self.julia_c,
                    do_sa,
                    bla_dc_max: self
                        .bla_eligible(mode, julia)
                        .then(|| Self::bla_dc_max(span_mantissa, delta_exp).mul_pow2(1.0)),
                };
                if self.ref_cache[vi].ref_pt.is_none() {
                    let res = recompute_worker(inputs); // cold start: nothing to draw with yet
                    self.install_recompute(vi, res);
                } else if self.recompute_rx[vi].is_none() {
                    let (tx, rx) = std::sync::mpsc::channel();
                    std::thread::spawn(move || {
                        let _ = tx.send(recompute_worker(inputs));
                    });
                    self.recompute_rx[vi] = Some(rx);
                }
                // else: a recompute is already in flight — keep using the cached reference.
            }
            // At extreme depth the recompute can take long enough that a fast/continuous dive
            // drifts the cached reference too far off-centre before a fresh one lands — rendering
            // with it is dark/glitchy (the "screen goes black" while zooming). Instead freeze the
            // last clean frame (via the reprojection path: prepare skips the re-iterate and holds
            // the frozen texture) until a fresh reference installs and the view snaps to it. NOT
            // gated on a job being in flight — the recompute throttle can leave gaps where nothing
            // is pending yet the reference is already too stale to paint. Only when a prior
            // reference exists (the cold start renders synchronously instead).
            //
            // Zoom-reprojection: rather than holding the frozen frame static, scale + pan it to
            // follow the zoom/motion since it was rendered, so the held detail keeps zooming
            // smoothly until the fresh reference lands and the view snaps to it. The transform maps
            // the current view back into the frozen texture (see the color shader):
            //   uv_scale = span_now/span_frozen = 2^(l2_frozen − l2_now)   (≤ 1 as we dive in)
            //   uv_off   = −pan_current · uv_scale,  pan_current = (center_now − center_frozen)/span
            // (y flips: complex-y is up, screen-uv-y is down.)
            if too_stale && self.ref_cache[vi].ref_pt.is_some() {
                match self.ref_cache[vi].frozen_center.clone() {
                    Some(fc) => {
                        let scale = ((self.ref_cache[vi].frozen_l2 - log2mag) as f32)
                            .exp2()
                            .clamp(1.0e-4, 1.0);
                        let px = fractadyne_core::ref_offset_mantissa(&center_bf[0], &fc[0], delta_exp, precision)
                            / span_mantissa[0];
                        let py = fractadyne_core::ref_offset_mantissa(&center_bf[1], &fc[1], delta_exp, precision)
                            / span_mantissa[1];
                        reproject_scale = scale;
                        reproject = Some([(-px as f32) * scale, (py as f32) * scale]);
                    }
                    None => reproject = Some([0.0, 0.0]), // nothing rendered yet → static hold
                }
            }
            // When this frame will actually re-iterate (not a freeze/pan reprojection), remember the
            // view it renders — the next freeze reprojects the resulting texture relative to it.
            if reproject.is_none() {
                self.ref_cache[vi].frozen_center = Some(center_bf.clone());
                self.ref_cache[vi].frozen_l2 = log2mag;
            }
            let rp = self.ref_cache[vi].ref_pt.as_ref().unwrap();
            // δ = center − reference, carried as a mantissa scaled by 2^-delta_exp
            // (so it stays O(1) in df32 at any depth; the GPU re-applies the exponent).
            let dx = fractadyne_core::ref_offset_mantissa(&center_bf[0], &rp[0], delta_exp, precision);
            let dy = fractadyne_core::ref_offset_mantissa(&center_bf[1], &rp[1], delta_exp, precision);
            let dxh = dx as f32;
            let dyh = dy as f32;
            ref_offset = [dxh, dyh, (dx - dxh as f64) as f32, (dy - dyh as f64) as f32];
            // Series approximation travels with the reference (computed off-thread); read it back.
            if do_sa {
                sa = self.ref_cache[vi].sa;
            }
            // BLA tree, cached per reference: reused across frames (and pans — the conservative
            // `dc_max` is offset-independent) and rebuilt only when the orbit changes or the view
            // zooms out enough to need a larger `dc_max`. Removes the ~20 ms/frame rebuild while
            // never reusing a tree whose validity radii are too optimistic for the current view.
            if self.bla_eligible(mode, julia) {
                let oid = self.ref_cache[vi].orbit_id;
                let dc_max = Self::bla_dc_max(span_mantissa, delta_exp);
                let need_log2 = dc_max.log2();
                let vc = &self.ref_cache[vi];
                // Rebuild if the orbit changed or the current view needs a bigger dc_max than the
                // cached tree was built for (tiny epsilon guards float noise).
                if vc.bla_id != oid || need_log2 > vc.bla_dc_max_log2 + 1.0e-6 {
                    let orbit = self.ref_cache[vi].orbit.clone();
                    // Build with 2× headroom (dc_max·2 ⇒ +1 in log2) so continuous zoom-out doesn't
                    // rebuild every frame; still valid (a larger dc_max only shrinks skip radii).
                    let build_dc = dc_max.mul_pow2(1.0);
                    let built = self.build_bla(&orbit, build_dc);
                    let vc = &mut self.ref_cache[vi];
                    vc.bla = built.unwrap_or_else(|| std::sync::Arc::new(Vec::new()));
                    vc.bla_id = oid;
                    vc.bla_dc_max_log2 = build_dc.log2();
                }
                let vc = &self.ref_cache[vi];
                if !vc.bla.is_empty() {
                    bla = vc.bla.clone();
                    bla_on = 1;
                }
            }
        }

        let cxh = cx as f32;
        let cyh = cy as f32;
        let center_df = [cxh, cyh, (cx - cxh as f64) as f32, (cy - cyh as f64) as f32];
        let (jcx, jcy) = self.julia_c;
        let jcxh = jcx as f32;
        let jcyh = jcy as f32;
        let julia_c = [jcxh, jcyh, (jcx - jcxh as f64) as f32, (jcy - jcyh as f64) as f32];

        if view_id == 0 {
            self.perf.last_mode = mode;
            self.perf.last_eff_iter = gpu_iter; // iterations actually rendered this frame
            self.perf.last_precision = precision;
            self.perf.last_orbit_len = self.ref_cache[vi].orbit_len;
            self.perf.last_sa_skip = sa.skip;
        }

        MandelbrotParams {
            orbit: self.ref_cache[vi].orbit.clone(),
            orbit_id: self.ref_cache[vi].orbit_id,
            orbit_len: self.ref_cache[vi].orbit_len,
            bla,
            bla_on,
            ref_offset,
            delta_exp,
            sa_skip: sa.skip,
            sa_a: sa.a,
            sa_a_exp: sa.a_exp,
            sa_b: sa.b,
            sa_b_exp: sa.b_exp,
            sa_c: sa.c,
            sa_c_exp: sa.c_exp,
            center: center_df,
            julia_c,
            mode,
            formula: fractal.formula_id(),
            julia: julia as u32,
            span_mantissa,
            max_iter: gpu_iter,
            cycle: self.color_cycle(),
            offset: self.offset,
            stop_count,
            stops,
            light: self.light as u32,
            light_angle: self.light_angle,
            light_height: self.light_height,
            de_on: self.de as u32,
            de_strength: self.de_strength,
            de_width: self.de_width,
            de_phase: self.de_phase,
            color_method: self.color_method,
            stripe_freq: self.stripe_freq,
            trap_type: self.trap_type,
            aa_filter,
            interior_col: self.interior_color(),
            resolution,
            ss,
            reproject: reproject.is_some() as u32,
            uv_offset: reproject.unwrap_or([0.0, 0.0]),
            uv_scale: reproject_scale,
            watermark: self.watermark as u32,
            view_id,
        }
    }
}
