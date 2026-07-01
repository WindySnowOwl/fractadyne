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

impl FractadyneApp {
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
    fn build_bla(
        &self,
        mode: u32,
        julia: bool,
        orbit: &[[f32; 4]],
        span_mantissa: [f64; 2],
        dx: f64,
        dy: f64,
        delta_exp: i32,
    ) -> Option<std::sync::Arc<Vec<[f32; 4]>>> {
        if !self.use_bla
            || mode != 2
            || julia
            || self.fractal.formula_id() != 0
            || fractadyne_gpu::method_needs_aux(self.color_method)
        {
            return None;
        }
        // Worst-case |δc| over the view (absolute): (|center−ref| + ½·diagonal)·2^delta_exp.
        let roff = (dx * dx + dy * dy).sqrt();
        let half_diag =
            0.5 * (span_mantissa[0] * span_mantissa[0] + span_mantissa[1] * span_mantissa[1]).sqrt();
        let dc_max = fractadyne_core::FloatExp::from_f64((roff + half_diag).max(1e-300))
            .mul_pow2(delta_exp as f64);
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
            self.prof.set(profile::ProfSetup { reference_ms, series_ms: t_sa.elapsed().as_secs_f64() * 1000.0 });
            if let Some(data) = self.build_bla(mode, julia, &orbit, scale.span_mantissa, dx, dy, delta_exp) {
                bla = data;
                bla_on = 1;
            }
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
        // Iteration cap + work budget are *interaction-aware*. While moving, cap hard (50k) with
        // a tight budget so motion stays responsive. Once the view settles, a still frame can
        // afford far more — the interactive budget is ~200× below the GPU watchdog (TDR ≈ 2 s) —
        // so raise both the cap and the budget and let the view sharpen to the zoom-appropriate
        // count (matching an export). `zoom_iter_cap` still bounds it, so it never over-resolves
        // sub-pixel "dust" into per-pixel noise. A high manual base is honoured on settle too.
        let (budget, iter_cap): (u64, u32) = if interacting {
            (WORK_BUDGET, 50_000)
        } else {
            (WORK_BUDGET.saturating_mul(6), 500_000)
        };
        let gpu_iter = eff_iter.min(iter_cap).min(zoom_iter_cap(log2mag).max(256));
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
        let reproject = reproject
            .filter(|_| mode != 1 && self.ref_cache[vi].ref_pt.is_some());

        let mut ref_offset = [0.0_f32; 4];
        let mut sa = fractadyne_core::SeriesSkip::NONE;
        let mut bla = std::sync::Arc::new(Vec::new());
        let mut bla_on = 0u32;
        if mode != 1 && reproject.is_none() {
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
            let needs_quality = precision > self.ref_cache[vi].orbit_prec
                || gpu_iter > self.ref_cache[vi].orbit_iter;
            let gone = drift.map_or(true, |(dx, dy)| dx > 1.5 || dy > 1.5);
            let recompute = if interacting {
                // Adaptive throttle: keep recompute to ≲ ~30% of wall time by spacing
                // refreshes at ~2.5× the last recompute's duration (so a slow debug
                // bignum doesn't stall motion, while a fast release build refreshes
                // often). Min 90 ms.
                let spacing = (self.perf.recompute_ms / 1000.0 * 2.5).max(0.09);
                let throttle_ok = self.ref_cache[vi]
                    .last_recompute
                    .map_or(true, |t| t.elapsed().as_secs_f64() > spacing);
                gone || ((out_of_view || needs_quality) && throttle_ok)
            } else {
                out_of_view || needs_quality
            };
            if recompute {
                let t = Instant::now();
                // Build only to the live-capped count: the GPU never iterates past `gpu_iter`,
                // so a longer bignum orbit would just slow navigation (exports build their own
                // full-length orbit in `current_export_request_for`).
                let (orbit, orbit_len, rp) =
                    self.compute_reference(&center_bf, span, gpu_iter, precision, julia, None);
                let vc = &mut self.ref_cache[vi];
                vc.ref_pt = Some(rp);
                vc.orbit = orbit;
                vc.orbit_len = orbit_len;
                vc.orbit_prec = precision;
                vc.orbit_iter = gpu_iter;
                vc.orbit_id = vc.orbit_id.wrapping_add(1);
                vc.last_recompute = Some(Instant::now());
                self.perf.recompute_ms = t.elapsed().as_secs_f64() * 1000.0;
                self.perf.recompute_total += 1;
                self.perf.rate_count += 1;
            }
            let rp = self.ref_cache[vi].ref_pt.as_ref().unwrap();
            // δ = center − reference, carried as a mantissa scaled by 2^-delta_exp
            // (so it stays O(1) in df32 at any depth; the GPU re-applies the exponent).
            let dx = fractadyne_core::ref_offset_mantissa(&center_bf[0], &rp[0], delta_exp, precision);
            let dy = fractadyne_core::ref_offset_mantissa(&center_bf[1], &rp[1], delta_exp, precision);
            let dxh = dx as f32;
            let dyh = dy as f32;
            ref_offset = [dxh, dyh, (dx - dxh as f64) as f32, (dy - dyh as f64) as f32];
            // Series approximation (cached per reference): seed δz to skip early iterations.
            // Both perturbation paths (mode 0 df32 and mode 2 floatexp) use it.
            if (mode == 0 || mode == 2)
                && !julia
                && fractal.formula_id() <= 3
                && !fractadyne_gpu::method_needs_aux(self.color_method)
                && self.series_approx
            {
                let oid = self.ref_cache[vi].orbit_id;
                if self.ref_cache[vi].sa_key != (oid, gpu_iter) {
                    let rp2 = self.ref_cache[vi].ref_pt.clone().unwrap();
                    let len = self.ref_cache[vi].orbit_len;
                    let computed = self
                        .series_skip_for(&rp2, span_mantissa, dx, dy, delta_exp, mode, julia, gpu_iter, len, precision);
                    self.ref_cache[vi].sa = computed;
                    self.ref_cache[vi].sa_key = (oid, gpu_iter);
                }
                sa = self.ref_cache[vi].sa;
            }
            // BLA tree (rebuilt per frame when enabled — off by default, so normally free;
            // caching per reference is a follow-up). Skips iterations throughout the orbit.
            let orbit = self.ref_cache[vi].orbit.clone();
            if let Some(data) = self.build_bla(mode, julia, &orbit, span_mantissa, dx, dy, delta_exp) {
                bla = data;
                bla_on = 1;
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
            view_id,
        }
    }
}
