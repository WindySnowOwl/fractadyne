//! Render-request builders — the performance-critical bridge from app state to the GPU.
//! Picks/caches the arbitrary-precision reference orbit, computes the series-approximation
//! skip, and assembles the live (`MandelbrotParams`) and offscreen (`ExportRequest`) jobs,
//! choosing the direct / df32-perturbation / floatexp render mode by depth.

use crate::{profile, zoom_iter_cap, FractadyneApp, FractalKind, RenderMode, PERT_FE_THRESHOLD};
use fractadyne_core::Viewport;
use fractadyne_gpu::{MandelbrotParams, RefOffset};
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
    /// Per-stage timings (for the export `--profile` breakdown; the live path uses only `ref_ms`).
    series_ms: f64,
    bla_ms: f64,
    /// True when the orbit was TRUNCATED (built to a coarse iteration cap without escaping) — i.e.
    /// a progressive cold-start's fast first stage. The render then caps `max_iter` to the orbit
    /// length so it never rebases past the short reference (which would glitch at extreme depth).
    /// False for a full or escaped orbit.
    partial: bool,
}

/// Owned, `Send` inputs for an off-thread reference recompute.
struct RecomputeInputs {
    center_bf: [fractadyne_core::BigFloat; 2],
    span: (fractadyne_core::FloatExp, fractadyne_core::FloatExp),
    span_mantissa: fractadyne_core::SpanMantissa,
    delta_exp: i32,
    gpu_iter: u32,
    precision: usize,
    julia: bool,
    formula: u32,
    julia_c: (f64, f64),
    do_sa: bool,
    bla_dc_max: Option<fractadyne_core::FloatExp>,
}

/// Aux coloring aggregates for the BLA tree, derived from a reference orbit. Triangle-inequality
/// needs `cmag` (= |c_ref| = |Z_1|, since Mandelbrot's Z_0 = 0 ⇒ Z_1 = c) and `power` (= 2, because
/// BLA is Mandelbrot-only) — both reference-intrinsic, so the tree caches per reference with no live
/// dependency. Point-trap uses the default trap aggregate (trap_type 0). `stripe_freq` stays default
/// (stripe's per-node aggregate isn't folded yet — it would need a rebuild on the freq slider).
fn aux_agg_from_orbit(orbit: &[[f32; 4]]) -> fractadyne_core::AuxAggParams {
    let cmag = orbit
        .get(1)
        .map(|z| {
            let (x, y) = (z[0] as f64 + z[2] as f64, z[1] as f64 + z[3] as f64);
            (x * x + y * y).sqrt()
        })
        .unwrap_or(0.0);
    fractadyne_core::AuxAggParams { trap_type: 0, stripe_freq: 1.0, cmag, power: 2.0 }
}

/// Pick a reference (once) then build its orbit + series-approximation skip + BLA tree to
/// `inp.gpu_iter` — the slow arbitrary-precision work. Pure and `Send`, so it runs on a worker
/// thread; mirrors the synchronous `compute_reference` + `series_skip_for` + `build_bla`. The
/// progressive cold start (`recompute_worker_staged`) reuses the `pick`/`build` split below.
fn recompute_worker(inp: RecomputeInputs) -> RecomputeResult {
    let rp = pick_reference(&inp);
    build_reference_from_point(rp, inp.gpu_iter, inp.do_sa, &inp)
}

/// Choose the reference point for `inp`. The ranking scan is internally capped (`REF_SCORE_SCAN`),
/// so this is cheap and deterministic — the SAME point comes back for any iteration budget past the
/// cap, which is why a coarse and a full stage share it exactly.
fn pick_reference(inp: &RecomputeInputs) -> [fractadyne_core::BigFloat; 2] {
    fractadyne_core::best_reference(
        &inp.center_bf,
        [inp.span.0, inp.span.1],
        inp.formula,
        inp.julia,
        [inp.julia_c.0, inp.julia_c.1],
        inp.gpu_iter,
        inp.precision,
    )
}

/// Build the orbit (to `orbit_iter`) + series-approximation skip + BLA tree for a PRE-CHOSEN
/// reference point `rp`. Split out of `recompute_worker` so a progressive cold start builds a short
/// (coarse) orbit and the full one from the SAME point (no pop on refine). `partial` is set when the
/// orbit never escaped (a truncated reference) — read from the last sample's magnitude, so an
/// escape-exactly-at-the-cap orbit is correctly treated as complete.
fn build_reference_from_point(
    rp: [fractadyne_core::BigFloat; 2],
    orbit_iter: u32,
    do_sa: bool,
    inp: &RecomputeInputs,
) -> RecomputeResult {
    use fractadyne_core as fc;
    let t = Instant::now();
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
    let (o, len) = fc::reference_orbit(&z0x, &z0y, &cx0, &cy0, inp.formula, orbit_iter, inp.precision);
    // Truncated (coarse) iff the orbit never escaped: the last sample stays inside the escape radius
    // (|z|² ≤ 1e12, the `reference_orbit` bail). A reference that escaped — even exactly at the cap —
    // is a complete orbit and must NOT cap the render.
    let partial = o
        .last()
        .map(|z| {
            let (x, y) = (z[0] as f64 + z[2] as f64, z[1] as f64 + z[3] as f64);
            x * x + y * y <= 1.0e12
        })
        .unwrap_or(false);
    let orbit = std::sync::Arc::new(o);
    let ref_ms = t.elapsed().as_secs_f64() * 1000.0;
    // Series approximation for the chosen reference.
    let t_sa = Instant::now();
    let sa = if do_sa {
        let dx = fc::ref_offset_mantissa(&inp.center_bf[0], &rp[0], inp.delta_exp, inp.precision);
        let dy = fc::ref_offset_mantissa(&inp.center_bf[1], &rp[1], inp.delta_exp, inp.precision);
        let roff = (dx * dx + dy * dy).sqrt();
        let half_diag = 0.5
            * (inp.span_mantissa.x * inp.span_mantissa.x + inp.span_mantissa.y * inp.span_mantissa.y).sqrt();
        let log2_max_dc = inp.delta_exp as f64 + (roff + half_diag).max(1e-300).log2();
        fc::series_skip(&rp[0], &rp[1], log2_max_dc, orbit_iter, len, inp.formula, inp.precision)
    } else {
        fc::SeriesSkip::NONE
    };
    let series_ms = t_sa.elapsed().as_secs_f64() * 1000.0;
    // BLA tree (Mandelbrot deep only; `None` otherwise). Built with the same conservative dc_max
    // the live path uses so the main thread reuses it across pans.
    let t_bla = Instant::now();
    let (bla, bla_dc_max_log2) = match inp.bla_dc_max {
        Some(dc_max) => {
            let levels = fc::build_bla_mandel(&orbit, dc_max, BLA_EPS, aux_agg_from_orbit(&orbit));
            let arc = if levels.is_empty() {
                std::sync::Arc::new(Vec::new())
            } else {
                std::sync::Arc::new(fc::bla_to_gpu(&levels))
            };
            (arc, dc_max.log2())
        }
        None => (std::sync::Arc::new(Vec::new()), f64::NEG_INFINITY),
    };
    let bla_ms = t_bla.elapsed().as_secs_f64() * 1000.0;
    RecomputeResult {
        orbit,
        orbit_len: len,
        rp,
        sa,
        bla,
        bla_dc_max_log2,
        prec: inp.precision,
        iter: orbit_iter,
        ref_ms,
        series_ms,
        bla_ms,
        partial,
    }
}

/// Off-thread cold-start build with a PROGRESSIVE fast path. Picks the reference once, then — when
/// `progressive` and the full build is deep enough to be slow — sends a COARSE truncated orbit first
/// (so a real, if partial, image appears in ~1-2 s and panning has a frame to track), followed by
/// the FULL orbit for final detail, both from the SAME reference point so the refine doesn't shift.
/// A non-progressive call (a drift/dive refresh, which already draws real content) just sends the
/// single full build. The receiver installs each stage and keeps the channel open until this returns.
fn recompute_worker_staged(
    inp: RecomputeInputs,
    tx: std::sync::mpsc::Sender<RecomputeResult>,
    progressive: bool,
) {
    // Coarse-first only helps when the full orbit would run far past this cap; the render's own
    // `gpu_iter` is already below it at shallow depth (until ~1e17×), so this naturally no-ops for
    // normal views → a single full build.
    const COARSE_ITER: u32 = 16384;
    if progressive && COARSE_ITER < inp.gpu_iter {
        let rp = pick_reference(&inp);
        // Coarse stage skips series approximation: its `series_skip` is a bignum coefficient pass
        // that costs seconds at extreme depth (≈ as much as the whole reference), and this stage
        // exists only to put a fast, BLA-accelerated, iteration-capped preview on screen so panning
        // has a frame to track. SA travels with the FULL stage.
        let mut coarse = build_reference_from_point(rp.clone(), COARSE_ITER, false, &inp);
        // Report the FULL iteration budget so `needs_quality` doesn't re-fire every frame during the
        // refine (the full stage is pipelined via the kept channel, not that signal).
        coarse.iter = inp.gpu_iter;
        if !coarse.partial {
            let _ = tx.send(coarse); // escaped within the cap → already the complete reference
            return;
        }
        if tx.send(coarse).is_err() {
            return; // receiver dropped (view/formula changed) → abandon the full stage
        }
        let full = build_reference_from_point(rp, inp.gpu_iter, inp.do_sa, &inp);
        let _ = tx.send(full);
    } else {
        let _ = tx.send(recompute_worker(inp));
    }
}

/// The reference-dependent fields of an `ExportRequest`, assembled from a `RecomputeResult`.
/// Shared by the synchronous export path and the pipelined (precomputed) tour path so both derive
/// these fields identically.
struct RefFields {
    ref_offset: RefOffset,
    orbit: std::sync::Arc<Vec<[f32; 4]>>,
    orbit_len: u32,
    sa: fractadyne_core::SeriesSkip,
    bla: std::sync::Arc<Vec<[f32; 4]>>,
    bla_on: u32,
}

impl Default for RefFields {
    fn default() -> Self {
        Self {
            ref_offset: RefOffset::ZERO,
            orbit: std::sync::Arc::new(Vec::new()),
            orbit_len: 0,
            sa: fractadyne_core::SeriesSkip::NONE,
            bla: std::sync::Arc::new(Vec::new()),
            bla_on: 0,
        }
    }
}

/// Turn a completed reference recompute into the GPU request's reference fields: the δ-offset of the
/// view center from the reference, the orbit, the series-approximation skip, and the BLA table.
fn assemble_ref_fields(vp: &Viewport, precision: usize, delta_exp: i32, res: RecomputeResult) -> RefFields {
    let dx = fractadyne_core::ref_offset_mantissa(&vp.center_x, &res.rp[0], delta_exp, precision);
    let dy = fractadyne_core::ref_offset_mantissa(&vp.center_y, &res.rp[1], delta_exp, precision);
    let bla_on = if res.bla.is_empty() { 0 } else { 1 };
    RefFields {
        ref_offset: RefOffset::from_df32(dx, dy),
        orbit: res.orbit,
        orbit_len: res.orbit_len,
        sa: res.sa,
        bla: res.bla,
        bla_on,
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
        vc.partial = res.partial; // always written, so the full stage clears a prior coarse's flag
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

    /// Snapshot view 0's FULL reference for on-disk persistence (see `refcache_persist`), or `None`
    /// if there's nothing worth saving yet — no reference, an empty orbit, or only a truncated coarse
    /// one (which would render capped; better to rebuild the full one next launch). The view-key is
    /// taken from the primary viewport, so this is meaningful only for `vi == 0`.
    pub(crate) fn build_saved_ref(&self, vi: usize) -> Option<crate::refcache_persist::SavedRef> {
        let vc = &self.ref_cache[vi];
        let rp = vc.ref_pt.as_ref()?;
        if vc.partial || vc.orbit.is_empty() {
            return None;
        }
        Some(crate::refcache_persist::SavedRef {
            center_x_str: fractadyne_core::to_decimal_string(&self.viewport.center_x),
            center_y_str: fractadyne_core::to_decimal_string(&self.viewport.center_y),
            upp_e: self.viewport.units_per_pixel.e,
            formula_id: self.fractal.formula_id(),
            julia: self.julia_mode,
            julia_c: self.julia_c,
            rp_x_str: fractadyne_core::to_decimal_string(&rp[0]),
            rp_y_str: fractadyne_core::to_decimal_string(&rp[1]),
            orbit: vc.orbit.clone(),
            orbit_len: vc.orbit_len,
            orbit_iter: vc.orbit_iter,
            orbit_prec: vc.orbit_prec as u64,
            partial: vc.partial,
            sa: vc.sa,
            bla: vc.bla.clone(),
            bla_dc_max_log2: vc.bla_dc_max_log2,
        })
    }

    /// Install a persisted reference snapshot into view `vi` (reverse of `build_saved_ref`), so a
    /// restored session renders its deep view at once instead of rebuilding the (up to ~10 s) bignum
    /// orbit. Mirrors `install_recompute`'s field writes. Returns false and installs nothing if the
    /// stored reference point can't be parsed (treated as a cache miss → normal rebuild).
    pub(crate) fn install_saved_ref(&mut self, vi: usize, s: crate::refcache_persist::SavedRef) -> bool {
        let (Some(rx), Some(ry)) = (
            fractadyne_core::parse_bf(&s.rp_x_str),
            fractadyne_core::parse_bf(&s.rp_y_str),
        ) else {
            return false;
        };
        let vc = &mut self.ref_cache[vi];
        vc.ref_pt = Some([rx, ry]);
        vc.orbit = s.orbit;
        vc.orbit_len = s.orbit_len;
        vc.orbit_id = vc.orbit_id.wrapping_add(1);
        vc.orbit_prec = s.orbit_prec as usize;
        vc.orbit_iter = s.orbit_iter;
        vc.partial = s.partial;
        vc.last_recompute = Some(Instant::now());
        vc.sa = s.sa;
        vc.sa_key = (vc.orbit_id, s.orbit_iter);
        vc.bla = s.bla;
        vc.bla_id = vc.orbit_id;
        vc.bla_dc_max_log2 = s.bla_dc_max_log2;
        true
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

    /// Owned, `Send` inputs for the reference recompute of an **export** view (single view or one
    /// panel of the dual). Mirrors the live path's `RecomputeInputs`, but with export-appropriate
    /// choices: the orbit is built to exactly `eff_iter` (no live-style spare headroom), and the BLA
    /// `dc_max` is the per-frame-tight bound (no `×2` pan-reuse margin). `None` for the direct path
    /// (`mode == 1`), which iterates from 0 with no reference. Series approximation applies to the
    /// holomorphic polynomial families (Mandelbrot / Multibrot 3-5) with a non-aux coloring method.
    #[allow(clippy::too_many_arguments)] // REFACTOR-PLAN Phase 2/4: fold into a reference-inputs struct
    fn export_reference_inputs(
        &self,
        vp: &Viewport,
        julia: bool,
        mode: RenderMode,
        eff_iter: u32,
        precision: usize,
        span_mantissa: fractadyne_core::SpanMantissa,
        delta_exp: i32,
    ) -> RecomputeInputs {
        let bla_dc_max = self
            .bla_eligible(mode, julia)
            .then(|| Self::bla_dc_max(span_mantissa, delta_exp));
        // BLA subsumes SA: when a BLA tree is built for this render it skips the same initial
        // iterations the series seed would (measured: gpu-it 22→27 ms at 1e1105×), while the SA
        // coefficient pass is the DOMINANT deep build cost (~9.4 s of a ~10.8 s build at 1e1105×,
        // ~8× the whole build). So compute SA only when no BLA tree will exist for this view
        // (df32-pert mode 0, Multibrot, BLA off/aux-gated) — there it remains the only skip.
        let do_sa = (!mode.is_direct())
            && !julia
            && self.fractal.formula_id() <= 3
            && !self.coloring.color_method.blocks_iter_skip()
            && self.render_cfg.series_approx
            && bla_dc_max.is_none();
        RecomputeInputs {
            center_bf: [vp.center_x.clone(), vp.center_y.clone()],
            span: vp.complex_span_fe(),
            span_mantissa,
            delta_exp,
            gpu_iter: eff_iter,
            precision,
            julia,
            formula: self.fractal.formula_id(),
            julia_c: self.julia_c,
            do_sa,
            bla_dc_max,
        }
    }

    /// Build the BLA tree (GPU-flattened) for a Mandelbrot deep view, or `None` when BLA
    /// doesn't apply (disabled, not floatexp/Mandelbrot/non-Julia, or an aux coloring method
    /// that BLA would skip). `dx`/`dy` are the reference-offset mantissas and `span_mantissa`
    /// the view span — both scaled by `2^delta_exp` — used for the worst-case `|δc|`.
    /// Whether BLA applies to this render (deep floatexp Mandelbrot, non-Julia, non-aux coloring).
    fn bla_eligible(&self, mode: RenderMode, julia: bool) -> bool {
        // Aux coloring blocks iteration-skipping — EXCEPT point orbit-trap, whose per-node aggregate
        // (the default min-|z| packing) is folded on each BLA skip (GPU-validated: the fold render
        // matches the full render), so it rides BLA at full speed instead of paying full floatexp
        // iterations. Cross/circle trap need their own aggregate; stripe/TIA also need the SA-prefix
        // fold — those stay gated (exact) until wired.
        let method = self.coloring.color_method;
        // Aux methods with a GPU-validated BLA-skip fold ride BLA at full speed: point orbit-trap
        // (default min-|z| aggregate) and triangle-inequality (cmag/power aggregate, reference-
        // intrinsic so it caches per reference). Cross/circle trap needs a real trap_type aggregate,
        // stripe needs a rebuild on its live freq, and decomposition isn't skip-safe — those stay gated.
        let aux_bla_ok = (method.to_u32() == 3 && (self.coloring.trap_type as u32) == 0) // OrbitTrap+Point
            || method.to_u32() == 2; // TriangleIneq
        self.render_cfg.use_bla
            && mode.is_floatexp()
            && !julia
            && self.fractal.formula_id() == 0
            && (!method.blocks_iter_skip() || aux_bla_ok)
    }

    /// Conservative worst-case `|δc|` (absolute, `·2^delta_exp`) for any pixel a reference serves:
    /// the view half-diagonal plus the drift the reference stays valid over (recomputed past ~1.5
    /// spans). Deliberately **independent of the current center offset**, so the BLA tree built
    /// with it is valid for every pixel across pans within one reference — letting it be cached per
    /// `orbit_id` instead of rebuilt each frame. A larger `dc_max` only shrinks the skip radii
    /// (safer, never wrong); the few skips lost vs. a per-frame-tight bound are bought back many
    /// times over by not rebuilding the tree every frame.
    fn bla_dc_max(span_mantissa: fractadyne_core::SpanMantissa, delta_exp: i32) -> fractadyne_core::FloatExp {
        let diag = (span_mantissa.x * span_mantissa.x + span_mantissa.y * span_mantissa.y).sqrt();
        fractadyne_core::FloatExp::from_f64((2.5 * diag).max(1e-300)).mul_pow2(delta_exp as f64)
    }

    /// Build the BLA tree (GPU-packed) for a reference orbit + worst-case `|δc|`. `None` if BLA
    /// produced no usable levels. Eligibility (`bla_eligible`) is the caller's gate.
    fn build_bla(
        &self,
        orbit: &[[f32; 4]],
        dc_max: fractadyne_core::FloatExp,
    ) -> Option<std::sync::Arc<Vec<[f32; 4]>>> {
        let levels =
            fractadyne_core::build_bla_mandel(orbit, dc_max, BLA_EPS, aux_agg_from_orbit(orbit));
        if levels.is_empty() {
            return None;
        }
        Some(std::sync::Arc::new(fractadyne_core::bla_to_gpu(&levels)))
    }

    /// The reference-recompute inputs for an export view, or `None` for the direct path (`mode == 1`,
    /// no reference). Computes `mode` / `eff_iter` / `precision` / scale exactly as
    /// [`current_export_request_with_ref`], so a result built from these inputs matches that frame's
    /// synchronous reference. Used by the tour pipeline to precompute the next frame's reference.
    fn export_reference_inputs_for(&self, vp: &Viewport, julia: bool) -> Option<RecomputeInputs> {
        let log2mag = vp.log2_magnification();
        let mag = vp.magnification();
        let mode = RenderMode::select(self.fractal.supports_perturbation(), mag);
        if mode.is_direct() {
            return None;
        }
        let eff_iter = if self.render_cfg.auto_iter {
            vp.recommended_max_iter(self.render_cfg.max_iter)
        } else {
            self.render_cfg.max_iter
        }
        .min(zoom_iter_cap(log2mag).max(256));
        let scale = vp.gpu_scale();
        Some(self.export_reference_inputs(vp, julia, mode, eff_iter, vp.precision, scale.span_mantissa, scale.delta_exp))
    }

    /// Kick off an export reference recompute on a worker thread, returning a channel to await it.
    /// The tour renderer spawns frame N+1's reference here while frame N renders on the GPU, then
    /// feeds the result to [`current_export_request_with_ref`]. `None` for the direct path.
    pub(crate) fn spawn_export_reference(
        &self,
        vp: &Viewport,
        julia: bool,
    ) -> Option<std::sync::mpsc::Receiver<RecomputeResult>> {
        let inputs = self.export_reference_inputs_for(vp, julia)?;
        let (tx, rx) = std::sync::mpsc::channel();
        // Fire-and-forget deep-export reference build (see the live-path note above): if the caller
        // drops the returned `rx` (export canceled) the worker finishes and its send is discarded —
        // bounded, self-terminating, not a leak. NOTE for a future `fractadyne-render` extraction:
        // this raw `Receiver` + the `pub` `ExportPrep.rx` should be wrapped behind a method API
        // before crossing a crate boundary.
        std::thread::spawn(move || {
            let _ = tx.send(recompute_worker(inputs));
        });
        Some(rx)
    }

    /// Build an export request for a given viewport + Julia flag at the export
    /// resolution. Recomputes a fresh reference orbit (deep) without touching the live
    /// cache. Height is derived from the viewport's aspect (square pixels).
    pub(crate) fn current_export_request_for(
        &self,
        vp: &Viewport,
        julia: bool,
    ) -> fractadyne_gpu::ExportRequest {
        self.current_export_request_with_ref(vp, julia, None)
    }

    /// As [`current_export_request_for`], but reuse a `precomputed` reference recompute when it was
    /// built for this exact frame (matching iteration count + precision) — the tour pipeline
    /// computes frame N+1's reference on a worker while frame N renders. A stale/absent precompute
    /// falls back to a synchronous `recompute_worker`, which is always correct.
    pub(crate) fn current_export_request_with_ref(
        &self,
        vp: &Viewport,
        julia: bool,
        precomputed: Option<RecomputeResult>,
    ) -> fractadyne_gpu::ExportRequest {
        let log2mag = vp.log2_magnification();
        let width = self.export.width.max(1);
        // height from aspect: span_y/span_x = height_px/width_px (the scale cancels).
        let height = ((width as f64) * vp.height_px / vp.width_px).round().max(1.0) as u32;
        let mag = vp.magnification(); // saturates to ∞ past 1e308×; fine for the mode compares
        let eff_iter = if self.render_cfg.auto_iter {
            vp.recommended_max_iter(self.render_cfg.max_iter)
        } else {
            self.render_cfg.max_iter
        }
        // Cap at the zoom-appropriate count (same as the live view): avoids noise from
        // over-resolving sub-pixel dust, and keeps the export fast/responsive.
        .min(zoom_iter_cap(log2mag).max(256));
        let mode = RenderMode::select(self.fractal.supports_perturbation(), mag);
        let precision = vp.precision; // maintained by the viewport; valid at any depth
        let (cx, cy) = vp.center_f64();
        let scale = vp.gpu_scale();
        let delta_exp = scale.delta_exp;

        // Reference orbit + series-approximation + BLA — the slow bignum bundle, computed via the
        // shared `recompute_worker` (same code the live view + the pipelined tour path use, so all
        // three produce byte-identical references). Split timings recorded for `--profile`.
        let RefFields { ref_offset, orbit, orbit_len, sa, bla, bla_on } = if !mode.is_direct() {
            // Reuse the precomputed reference only if it was built for this exact frame's iteration
            // count + precision; otherwise (stale, or none) compute it now. Fallback is always safe.
            let res = match precomputed {
                Some(r) if r.iter == eff_iter && r.prec == precision => r,
                _ => {
                    let inputs = self.export_reference_inputs(vp, julia, mode, eff_iter, precision, scale.span_mantissa, delta_exp);
                    recompute_worker(inputs)
                }
            };
            self.prof.set(profile::ProfSetup {
                reference_ms: res.ref_ms,
                series_ms: res.series_ms,
                bla_ms: res.bla_ms,
            });
            assemble_ref_fields(vp, precision, delta_exp, res)
        } else {
            RefFields::default()
        };

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
            ss: self.export.ss.max(1),
            span_mantissa: scale.span_mantissa,
            center,
            ref_offset,
            delta_exp,
            sa_skip: sa.skip,
            glitch_on: 0, // enabled per-pass by the multi-reference correction path
            vignette: Default::default(), // set per-frame by the tour renderer; off for normal exports
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
            mode: mode.to_u32(),
            formula: self.fractal.formula_id(),
            julia: julia as u32,
            cycle: self.color_cycle(),
            offset: self.coloring.offset,
            stop_count,
            stops,
            light: self.effects.light as u32,
            light_angle: self.effects.light_angle,
            light_height: self.effects.light_height,
            de_on: self.effects.de as u32,
            de_strength: self.effects.de_strength,
            de_width: self.effects.de_width,
            de_phase: self.effects.de_phase,
            color_method: self.coloring.color_method.to_u32(),
            stripe_freq: self.coloring.stripe_freq,
            trap_type: self.coloring.trap_type.to_u32(),
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
    #[allow(clippy::too_many_arguments)] // REFACTOR-PLAN Phase 2/4: fold into a render-request struct
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
        // Direct path never glitches; nothing to correct.
        if RenderMode::from_u32(req.mode).is_direct() {
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
            // Re-reference pass: fresh orbit, no SA/BLA (they were built for the base reference).
            let mut r = req.clone();
            r.orbit = orbit;
            r.orbit_len = len;
            r.ref_offset = RefOffset::from_df32(dx, dy);
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
        // Correction renders the whole view in one un-tiled texture set (iter + aux + color +
        // readback ≈ 64 B/px), so it's bounded by both the GPU's max 2-D texture dimension and, to
        // avoid OOM now that it's on by default, a conservative area cap. Above either, fall back to
        // the tiled (uncorrected) path. ~32 MP covers 4K/5K/6K comfortably.
        const MAX_CORRECT_PX: u64 = 32_000_000;
        let max_dim = device.limits().max_texture_dimension_2d;
        if width > max_dim
            || height > max_dim
            || (width as u64) * (height as u64) > MAX_CORRECT_PX
            || self.coloring.color_method.needs_aux()
        {
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
        // Anti-alias supersampling target for this frame (1 while moving; ramps up on settle via the
        // caller's progressive-settle stages). Clamped to the GPU texture limit below.
        aa_target: u32,
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
        let span_mantissa =
            fractadyne_core::SpanMantissa::new(span.0.mul_pow2(sm).to_f64(), span.1.mul_pow2(sm).to_f64());

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
        // floatexp (mode 2, >= PERT_FE_THRESHOLD) costs several× more per iteration than df32, and
        // right at the df32→floatexp crossover the BLA table is still being built off-thread, so the
        // first floatexp frames run the *full* iteration count. At native resolution that single frame
        // can exceed the GPU watchdog (TDR) and freeze the whole app mid-dive — exactly the hang seen
        // when a fast live dive crosses ~1e28×. Shrink the interacting budget for mode 2 so resolution
        // (not the watchdog) absorbs the cost. Settle frames keep the full budget: by then the
        // reference + BLA have landed and the frame is cheap even in floatexp.
        let is_fe = fractal.supports_perturbation() && magnification >= PERT_FE_THRESHOLD;
        // Deep df32 *perturbation* (1e4× … 1e28×) is as GPU-heavy per moving frame as floatexp, so it
        // needs the same relief: without it a continuous zoom (esp. zoom-OUT, which gets neither
        // pan-reprojection nor the floatexp motion-freeze) renders full-budget ~150 ms frames that
        // pile into the vsync swapchain faster than the GPU drains — the event loop blocks on
        // present and the app hangs ("Not Responding"). Shrink the *moving* budget so resolution
        // absorbs the cost; settle frames keep the full `wb*6`, so the final image is unchanged.
        let is_pert = fractal.supports_perturbation() && magnification >= 1.0e4;
        let wb = self.effective_work_budget();
        let (budget, iter_cap): (u64, u32) = if interacting {
            let moving = if is_fe {
                wb / 6
            } else if is_pert {
                wb / 4
            } else {
                wb // direct/shallow: already cheap
            };
            (moving, 500_000)
        } else {
            (wb.saturating_mul(6), 500_000)
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
        // A reprojection/freeze frame runs NO iterate (it re-samples the frozen texture), so the
        // motion res_scale saves nothing on it — and worse, it shrinks the frame's base below the
        // frozen texture's settle-time resolution, so the color-pass aspect-fit `fit = out_res /
        // frozen_screen_dim` goes < 1 and MAGNIFIES the held frame (a spurious zoom-in) while also
        // amplifying the pan translation by 1/fit (the "drag is exaggerated / double acceleration"
        // at deep zoom). Keep native resolution on those frames so fit ≈ 1 and the drag tracks 1:1.
        let will_reproject = reproject.is_some()
            || (is_fe && interacting && !self.autopilot.stepping)
            || self.ref_cache[view_id as usize].ref_pt.is_none();
        let resolution = if res_scale < 1.0 && !will_reproject {
            [
                ((resolution[0] as f64 * res_scale) as u32).max(16),
                ((resolution[1] as f64 * res_scale) as u32).max(16),
            ]
        } else {
            resolution
        };
        // Hard GPU-watchdog (TDR) budget, independent of the BLA-trusting work budget. `max_ss`/`budget`
        // size the frame assuming BLA delivers its usual ~200× iteration skip — but on a boundary/
        // filament-heavy deep view (exactly where the user lingers) BLA can barely skip, so the frame's
        // true cost approaches the *no-skip* estimate `spx·ss²·gpu_iter`. At floatexp depth that frame is
        // multiple seconds without BLA and freezes the app (Windows resets the GPU after ~2s). Bound the
        // worst-case (no-BLA) floatexp iterate cost of ONE frame to ~3e11 steps ≈ 0.85s on this GPU
        // class (measured: a 1.74e12-step frame froze at ~5s). Cheap when BLA works.
        const TDR_SAFE_STEPS: u64 = 300_000_000_000;
        let spx = (resolution[0] as u64) * (resolution[1] as u64);
        // First line of defence is lowering ss (`max_ss_tdr` below), but that floors at 1. If even ss=1
        // would blow the budget — `spx·gpu_iter` alone > TDR_SAFE_STEPS, i.e. a large floatexp panel at
        // a high fixed iteration count — the ss cap can't help, so shrink the render RESOLUTION too.
        // Applied to reproject frames AS WELL: the frozen texture they re-sample was rendered by a
        // settled frame that took this same (deterministic: same panel, same iter count) shrink, so a
        // reproject frame must take it too for the color pass aspect-fit `fit = out_res /
        // frozen_screen_dim` to stay 1. Skipping it here (as v0.1.46 briefly did) leaves the reproject
        // frame at native res over a shrunk frozen texture → fit > 1 → the held image displays SHRUNK
        // (average-color borders) and the pan translation runs slow by the same factor — the inverse of
        // the v0.1.44 magnify. (The MOTION res_scale above stays gated on `!will_reproject`: unlike this
        // shrink it differs between moving and settled frames, which is what caused the v0.1.44 bug.)
        let (resolution, spx) = {
            let iter_cost = spx.saturating_mul(gpu_iter.max(1) as u64);
            if is_fe && iter_cost > TDR_SAFE_STEPS {
                let f = (TDR_SAFE_STEPS as f64 / iter_cost as f64).sqrt();
                let r = [
                    ((resolution[0] as f64 * f) as u32).max(16),
                    ((resolution[1] as f64 * f) as u32).max(16),
                ];
                (r, (r[0] as u64) * (r[1] as u64))
            } else {
                (resolution, spx)
            }
        };
        let max_ss = ((budget / spx.saturating_mul(gpu_iter.max(1) as u64).max(1)) as f64)
            .sqrt()
            .max(1.0) as u32;
        let max_ss_tdr = if is_fe {
            ((TDR_SAFE_STEPS / spx.saturating_mul(gpu_iter.max(1) as u64).max(1)) as f64)
                .sqrt()
                .max(1.0) as u32
        } else {
            u32::MAX
        };
        // Adaptive wall-clock extension of the watchdog cap. `max_ss_tdr` assumes BLA skips
        // NOTHING — right on boundary/filament-heavy views, but over-throttling the common case
        // where BLA is effective and full AA would cost milliseconds. A resolved probe (`update`
        // measures each newly-rendered settle stage's frame interval) tells us this view's REAL
        // cost, so allow at most one doubling past the measured stage, quadratically projected
        // and kept under ~800 ms (≈2.5× margin under the ~2 s watchdog). Each further doubling
        // needs its own measurement, so the extension can never outrun reality by more than 4×;
        // interaction clears the measurement (below), so it can't carry across views. Where BLA
        // truly can't skip, the measured cost is high, the extension is nil, and the static cap
        // stands — the settled frame stays bounded exactly as before.
        let vs = view_id as usize;
        if interacting {
            self.perf.aa_measured[vs] = None;
            self.perf.aa_probe[vs] = None;
        }
        let max_ss_meas = self.perf.aa_measured[vs]
            .filter(|_| is_fe)
            .map(|(s, ms)| {
                const TDR_SAFE_MS: f64 = 800.0;
                let by_cost = ((s as f64) * (TDR_SAFE_MS / ms.max(1.0)).sqrt()).floor() as u32;
                by_cost.clamp(s, s.saturating_mul(2))
            })
            .unwrap_or(0);
        // `aa_target` is 1 while moving and ramps up over settled frames (progressive settle); clamp
        // to what the per-frame budget affords (`max_ss`) and the watchdog allows (static worst-case
        // cap, extended by the measured allowance where the real cost is known).
        let ss = aa_target.min(max_ss).min(max_ss_tdr.max(max_ss_meas)).max(1);
        // Arm a probe only on a frame that actually RE-ITERATES (a cached frame's interval is
        // ~vsync and would wildly over-authorize): the iterate re-runs when its key inputs change
        // (ss stage, resolution, reference), and on the first settled frame after an interaction
        // (the view moved, so that frame re-iterates even with an unchanged key — this bootstraps
        // the ladder on views whose static cap is 1, which otherwise never see an ss change).
        let key = (ss, resolution, self.ref_cache[vs].orbit_id);
        let key_changed = key != self.perf.aa_last_key[vs];
        self.perf.aa_last_key[vs] = key;
        let bootstrap =
            self.perf.aa_measured[vs].is_none() && self.perf.aa_probe[vs].is_none();
        if is_fe
            && !interacting
            && !will_reproject
            && self.playback.is_none() // tour frames move every keyframe — not settled-cost data
            && (key_changed || bootstrap)
        {
            self.perf.aa_probe[vs] = Some((ss, self.perf.frame_idx, 0.0));
        }
        // Color-pass anti-aliasing when true supersampling wasn't affordable: widen the box
        // to match an upscaled (resolution-reduced) texture, or apply a gentle 2× box when
        // the budget forced ss=1 on a settled view the user wanted anti-aliased.
        let aa_filter = if res_scale < 1.0 && !will_reproject {
            ((1.0 / res_scale).round() as u32).clamp(2, 4)
        } else if ss == 1 && self.render_cfg.aa > 1 && !interacting {
            2
        } else {
            1
        };

        // Render path: 1 = direct df32 (shallow / unsupported formulas), 0 = df32
        // perturbation (fast, common deep range), 2 = floatexp perturbation (past df32's
        // ~1e30× exponent limit → extreme depth, ~1.7× costlier so only when needed).
        let mode = RenderMode::select(fractal.supports_perturbation(), magnification);
        let precision = fractadyne_core::precision_for_octaves(log2mag.max(0.0).ceil() as u64);
        let vi = view_id as usize;

        // Honour a pan reprojection only at deep zoom (the shallow direct path is cheap + already
        // detailed) and only once a reference orbit exists to have produced the frozen texture.
        // Mutable: also set below to freeze the last good frame while an off-thread recompute is in
        // flight and the cached reference has drifted fully out of view (extreme depth), so the
        // view holds instead of flashing blank.
        let mut reproject = reproject
            .filter(|_| !mode.is_direct() && self.ref_cache[vi].ref_pt.is_some());
        // Reprojection scale about the view centre: 1.0 for a pan (drag) reprojection; set <1.0 by
        // the freeze below to zoom the held frame as the view keeps diving (zoom-reprojection).
        let mut reproject_scale = 1.0_f32;

        let mut ref_offset = RefOffset::ZERO;
        let mut sa = fractadyne_core::SeriesSkip::NONE;
        let mut bla = std::sync::Arc::new(Vec::new());
        let mut bla_on = 0u32;
        if !mode.is_direct() && reproject.is_none() {
            // Install a finished off-thread recompute (if any) before deciding whether another is
            // needed, so the staleness/quality checks below see the fresh reference.
            // Install every stage the worker sends — a progressive cold start delivers a coarse
            // reference first, then the full one — draining all that arrived this frame and keeping
            // the channel open (Empty) until the worker finishes and drops the sender (Disconnected).
            // `take()` moves the receiver to a local so the `install_recompute(&mut self)` calls in
            // the loop don't conflict with a borrow of `self.recompute_rx`.
            if let Some(rx) = self.recompute_rx[vi].take() {
                loop {
                    match rx.try_recv() {
                        Ok(res) => self.install_recompute(vi, res),
                        Err(std::sync::mpsc::TryRecvError::Empty) => {
                            self.recompute_rx[vi] = Some(rx); // still running → keep polling
                            break;
                        }
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => break, // done → leaves None
                    }
                }
            }
            // Drift = |center − reference| / span, both as 2^-delta_exp mantissas so the
            // ratio is exact at any depth (raw f64 differences underflow past ~1e308×).
            let drift = self.ref_cache[vi].ref_pt.as_ref().map(|r| {
                let dx = fractadyne_core::ref_offset_mantissa(&center_bf[0], &r[0], delta_exp, precision)
                    / span_mantissa.x;
                let dy = fractadyne_core::ref_offset_mantissa(&center_bf[1], &r[1], delta_exp, precision)
                    / span_mantissa.y;
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
            // Refresh when the reference drifts past ~0.7 span off-centre. best_reference sits at
            // ≤ ~0.4, and the perturbation stays clean out to ~1 span, so 0.7 leaves margin while
            // recomputing far less often than the old 0.5 — a tiny margin there meant a refresh
            // after almost any motion, which at shallow depth (recompute ≈ instant) churned the
            // reference every few frames and made crisp palettes (Binary) visibly stutter.
            let out_of_view = drift.is_none_or(|(dx, dy)| dx > 0.7 || dy > 0.7);
            // Once the reference is well outside the view (≫ the ~0.9 span best_reference normally
            // sits at), the perturbation δc is large enough to render wrong/glitchy. On a fast/deep
            // dive the async recompute can lag this far — freeze the last clean frame rather than
            // paint the bad one (see the reprojection freeze below). Kept conservative so normal
            // deep motion (reference merely drifting, still usable) isn't needlessly held.
            let mut too_stale = drift.is_some_and(|(dx, dy)| dx > 1.5 || dy > 1.5);
            // Octaves the view has zoomed *in* since the cached reference/BLA were built (BLA dc_max
            // scales with the view span, so its log2 drop = octaves of zoom-in). In floatexp (mode 2)
            // the iterate shader spins ~5 s/frame on a reference this depth-stale — measured spin
            // onset ≈ 3 octaves of lag (18 ms at lag 3.1, then 5167 ms the next frame). Since a
            // *centered* dive keeps `drift ≈ 0`, the positional `too_stale` above never fires, so this
            // is the signal that catches a fast zoom-in.
            let depth_lag = {
                let vc = &self.ref_cache[vi];
                if mode.is_floatexp() && !vc.bla.is_empty() && vc.bla_dc_max_log2.is_finite() {
                    vc.bla_dc_max_log2 - Self::bla_dc_max(span_mantissa, delta_exp).log2()
                } else {
                    0.0
                }
            };
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
            // In mode 2, refresh the reference/BLA as soon as it lags (off-thread, cheap) so fresh
            // references keep coming; freeze (below) just above that while each refresh lands. A
            // fresh install sits at depth_lag ≈ 1 (the BLA is built with 1 octave of dc_max headroom),
            // so `> 1.1` refreshes ~0.1 octave into the dive.
            let recompute = out_of_view || needs_quality || depth_lag > 1.1;
            // Whether the series approximation applies to this view (bundled into the recompute).
            // BLA subsumes SA (see `export_reference_inputs`): when this view builds a BLA tree,
            // skip the SA coefficient pass — it's the dominant deep build cost (~9.4 s at 1e1105×)
            // for a skip BLA already provides (gpu-it 22→27 ms). SA stays on where BLA can't go
            // (mode 0, Multibrot, BLA off/aux-gated), where it remains the only iteration skip.
            let bla_will_build = self.bla_eligible(mode, julia);
            let do_sa = (!mode.is_direct())
                && !julia
                && fractal.formula_id() <= 3
                && !self.coloring.color_method.blocks_iter_skip()
                && self.render_cfg.series_approx
                && !bla_will_build;
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
                    bla_dc_max: bla_will_build
                        .then(|| Self::bla_dc_max(span_mantissa, delta_exp).mul_pow2(1.0)),
                };
                // Anti-churn backstop: never respawn more than ~60×/s (spaced ≥ 16 ms). The wider
                // `out_of_view` above already keeps refreshes infrequent; this just guards against a
                // pathological every-frame respawn storm (which backs up GPU orbit uploads and can
                // freeze the UI) without throttling legitimate refreshes enough to be visible.
                let spawn_ok = self.ref_cache[vi]
                    .last_recompute
                    .is_none_or(|t| t.elapsed().as_millis() >= 16);
                // A cold start (no reference yet) bypasses the anti-churn throttle so it spawns at
                // once — otherwise a jump landing within 16 ms of the last recompute could leave the
                // view with no reference AND no in-flight job, and if nothing requests a repaint it
                // would sit stuck. (The `recompute_rx.is_none()` guard still prevents a spawn storm.)
                let cold = self.ref_cache[vi].ref_pt.is_none();
                if self.recompute_rx[vi].is_none() && (spawn_ok || cold) {
                    // Off-thread even for the COLD START (ref_pt is None). It used to run INLINE here,
                    // which froze the UI ("Not Responding") for the full bignum build on every discrete
                    // jump — goto, bookmark load, undo/redo, formula switch, and every deep dual-Julia
                    // hover frame. Now the frame below freezes the last good frame (or blanks a truly
                    // fresh view) via reprojection until the job lands, and the event loop keeps
                    // repainting while `recompute_rx[vi]` is in flight — so the window stays responsive.
                    let (tx, rx) = std::sync::mpsc::channel();
                    // Fire-and-forget worker. `let _ = tx.send` deliberately discards the send:
                    // if the receiver was dropped (view/formula change → `drop_ref_caches`) the
                    // worker is orphaned and its result is stale. This is NOT a leak — the thread
                    // runs the bignum orbit to completion then exits; the only cost is bounded,
                    // self-terminating CPU on a superseded reference (a cooperative-cancel
                    // AtomicBool is deferred until profiling shows it matters). Spawn is guarded by
                    // `recompute_rx[vi].is_none()`, so receivers never accumulate.
                    // Progressive (coarse-then-full) only for a genuine cold start, and NOT for the
                    // live unpinned dual-Julia hover (view 1) which re-invalidates on every cursor
                    // move — staging there would keep burning full builds on an already-stale c — nor
                    // during tours (which re-invalidate per keyframe). Shallow views auto-skip inside
                    // the worker (COARSE_ITER exceeds their gpu_iter).
                    let progressive = cold
                        && !(self.dual && vi == 1 && self.julia_pin.is_none())
                        && self.playback.is_none();
                    std::thread::spawn(move || {
                        recompute_worker_staged(inputs, tx, progressive);
                    });
                    self.recompute_rx[vi] = Some(rx);
                }
                // else: a recompute is already in flight (or throttled) — use the cached reference.
            }
            // Freeze (reproject, which skips the expensive floatexp iterate) rather than paint with a
            // depth-stale reference, which makes the mode-2 shader spin ~5 s/frame → the "Not
            // Responding" hang on a fast dive crossing ~1e28×. Two triggers:
            //  • `mode.is_floatexp() && interacting`: never run a real floatexp iterate *while moving*. On a
            //    dive faster than the (off-thread, ~0.1–1 s) reference recompute, every reference is
            //    already stale by the time it lands, and the spin onset is data-dependent (as low as
            //    ~0.5 octave of lag), so no threshold reliably lets a real frame through without
            //    risking the spin. Reprojecting the last good frame is smooth + always cheap; the
            //    positional `drift` trigger never caught this because a centered dive keeps drift ≈ 0.
            //  • `depth_lag > 1.2`: once motion stops, keep holding until the freshly-recomputed
            //    reference (matched to this depth) lands, then render real detail — so a settle snaps
            //    to full sharpness instead of spinning on the stale reference for its first frame.
            // During the autopilot's deep *stepped* dive we WANT real frames between jumps (each one
            // held on screen while the next computes), so bypass the blanket "never iterate while
            // moving" freeze — the `depth_lag > 1.2` hold below still waits for a depth-matched
            // reference, so the real iterate never spins on a stale one.
            let motion_freeze = mode.is_floatexp() && interacting && !self.autopilot.stepping;
            too_stale = too_stale || motion_freeze || depth_lag > 1.2;
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
            // Freeze (reproject the last good frame) when the cached reference is too stale to paint,
            // OR when there is NO reference yet (cold start now runs off-thread — hold instead of
            // iterating with no orbit). A fresh view with nothing rendered falls to the static hold /
            // blank below.
            if too_stale || self.ref_cache[vi].ref_pt.is_none() {
                match self.ref_cache[vi].frozen_center.clone() {
                    Some(fc) => {
                        let scale = ((self.ref_cache[vi].frozen_l2 - log2mag) as f32)
                            .exp2()
                            .clamp(1.0e-4, 1.0);
                        let px = fractadyne_core::ref_offset_mantissa(&center_bf[0], &fc[0], delta_exp, precision)
                            / span_mantissa.x;
                        let py = fractadyne_core::ref_offset_mantissa(&center_bf[1], &fc[1], delta_exp, precision)
                            / span_mantissa.y;
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
            // δ = center − reference, carried as a mantissa scaled by 2^-delta_exp (O(1) in df32 at
            // any depth; the GPU re-applies the exponent). Skipped during a cold-start hold — no
            // reference yet, so this frame is a reprojection freeze and ref_offset stays ZERO/unused.
            if let Some(rp) = self.ref_cache[vi].ref_pt.as_ref() {
                let dx = fractadyne_core::ref_offset_mantissa(&center_bf[0], &rp[0], delta_exp, precision);
                let dy = fractadyne_core::ref_offset_mantissa(&center_bf[1], &rp[1], delta_exp, precision);
                ref_offset = RefOffset::from_df32(dx, dy);
            }
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
            self.perf.last_mode = mode.to_u32();
            self.perf.last_eff_iter = gpu_iter; // iterations actually rendered this frame
            self.perf.last_precision = precision;
            self.perf.last_orbit_len = self.ref_cache[vi].orbit_len;
            self.perf.last_sa_skip = sa.skip;
        }

        // Never submit a full-length perturbation iterate without a reference. During the async
        // cold-start (`ref_pt` None) a FRESH view can't reproject the placeholder — there is no
        // frozen frame to reproject — so the shader falls through to a real iterate, but with an
        // empty orbit every pixel runs to `max_iter` against a null reference. At deep zoom with a
        // large `max_iter` (e.g. 500k) that single GPU frame overruns the Windows GPU-watchdog (TDR)
        // timeout → the device is lost → wgpu treats it as fatal and the process aborts. Cap the
        // placeholder to a trivially cheap iterate (a flat interior fill) until the reference lands.
        // Direct mode has no reference by design, so it keeps its full iteration count.
        const PLACEHOLDER_ITER_CAP: u32 = 256;
        let shader_iter = if mode.is_direct() {
            gpu_iter
        } else if self.ref_cache[vi].ref_pt.is_none() {
            gpu_iter.min(PLACEHOLDER_ITER_CAP) // no reference yet → cheap flat placeholder (TDR-safe)
        } else if self.ref_cache[vi].partial {
            // Coarse (truncated) reference from a progressive cold start: cap at orbit_len-1 so the
            // shader never rebases past the short reference into df32-inaccurate territory (which
            // speckles at extreme depth) — a clean partial image (fast escapers correct, the rest
            // interior) until the full reference lands and refines it.
            gpu_iter.min(self.ref_cache[vi].orbit_len.saturating_sub(1))
        } else {
            gpu_iter
        };

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
            mode: mode.to_u32(),
            formula: fractal.formula_id(),
            julia: julia as u32,
            span_mantissa,
            max_iter: shader_iter,
            cycle: self.color_cycle(),
            offset: self.coloring.offset,
            stop_count,
            stops,
            light: self.effects.light as u32,
            light_angle: self.effects.light_angle,
            light_height: self.effects.light_height,
            de_on: self.effects.de as u32,
            de_strength: self.effects.de_strength,
            de_width: self.effects.de_width,
            de_phase: self.effects.de_phase,
            color_method: self.coloring.color_method.to_u32(),
            stripe_freq: self.coloring.stripe_freq,
            trap_type: self.coloring.trap_type.to_u32(),
            aa_filter,
            interior_col: self.interior_color(),
            resolution,
            ss,
            reproject: reproject.is_some() as u32,
            uv_offset: reproject.unwrap_or([0.0, 0.0]),
            uv_scale: reproject_scale,
            // Guided-tour spotlight (main view only), anchored to its fractal coordinate.
            vignette: if view_id == 0 {
                self.playback
                    .as_ref()
                    .map(|pb| crate::scripting::vignette_for(&pb.spotlights, &self.viewport, pb.cur_t))
                    .unwrap_or_default()
            } else {
                Default::default()
            },
            view_id,
        }
    }
}
