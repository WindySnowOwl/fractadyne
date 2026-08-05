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

/// GPU time a settled floatexp frame is aimed at. Comfortably under the ~2 s GPU watchdog AND short
/// enough that the UI thread keeps pumping messages between frames (Windows paints a window "Not
/// Responding" after ~5 s), so an unaffordable view degrades in resolution instead of hanging.
pub(crate) const TDR_BUDGET_MS: f64 = 900.0;
/// Per-measurement budget change limits. Growth is capped well under 2× so the next frame cannot leap
/// from the target into the watchdog; shrink is allowed to halve at once.
///
/// The budget is retargeted by the measured-time RATIO, deliberately without modelling cost as
/// `steps ∝ time`. That model is false at deep interior views: every pixel runs the full iteration
/// count on a dependent chain, so a small frame is LATENCY-bound (~89.9k iterations ≈ 415 ms here no
/// matter how few pixels) and only becomes throughput-bound once it saturates GPU occupancy. Assuming
/// proportionality made the loop conclude a shrunk frame should be fast, and it drove the view to a
/// postage stamp trying to reach a target that no resolution could reach. A ratio search needs no such
/// assumption — it just walks toward whatever size actually measures near the target.
pub(crate) const TDR_GROW_MAX: f64 = 1.5;
pub(crate) const TDR_SHRINK_MAX: f64 = 0.5;
/// Cost of the very first floatexp frame, before any measurement exists. Deliberately tiny — a few ms
/// even on a GPU orders of magnitude slower than a desktop discrete part — so the bootstrap frame is
/// safe on hardware we have never seen. Everything above it is measured, not assumed.
pub(crate) const TDR_BOOTSTRAP_STEPS: u64 = 400_000_000;
/// Never exceed this even if a view measures absurdly cheap (a lone quick interval shouldn't uncork a
/// multi-second dispatch).
pub(crate) const TDR_STEPS_CEIL: u64 = 300_000_000_000;
/// Most budget-sized dispatches a tiled settle may spend sharpening one frame. Bounds both the total
/// settle time (~tiles × TDR_BUDGET_MS of GPU work) and how far the settled resolution may exceed
/// what a single dispatch affords; a view too costly even for this many tiles renders shrunk, tiled.
pub(crate) const TDR_MAX_TILES: u64 = 16;

/// One view's tiled-settle progress. `None` geometry = ARMED: the view has rendered its coarse
/// single-dispatch frame under this key and the next settled frame may start the grid. Tiles then
/// advance one per app frame, center-out.
pub(crate) struct TileGrid {
    /// `(orbit_id, gpu_iter, view_gen, panel_px, settings_hash)` — identifies the exact frame
    /// content being refined. The generation counter stands in for the view position: at 1e106× the
    /// f64 center is degenerate (nearby views project to identical values), so instead ANY motion
    /// frame (interacting, reproject, autopilot) bumps the generation, invalidating the grid.
    /// Composing tiles over a texture whose content belongs to a different view would splice two
    /// views into one image. `panel_px` is the RAW panel resolution, before any budget-derived
    /// shrink: the shrunk resolution jitters with the budget (that jitter is exactly what the
    /// geometry pin exists to absorb), but the panel itself only changes on a real window/layout
    /// resize — which must invalidate the grid, or a completed pin would keep serving the old-size
    /// frame forever. `settings_hash` covers every iterate-affecting setting the GPU's IterKey
    /// watches (coloring method / stripe / trap / SA / BLA / Julia c): a setting change under a
    /// pinned grid must re-arm it, because the GPU-side re-render is scissored to the current tile
    /// and would otherwise splice new-settings data into an old-settings frame.
    pub key: (u64, u32, u64, [u32; 2], u64),
    /// `(resolution, ss, tile side in base px)` — frozen when the grid starts, so a budget update
    /// mid-grid can't reshape the rects underneath the cursor. `None` until the first tiled frame.
    pub geo: Option<([u32; 2], u32, u32)>,
    /// Next tile index (in center-out order) to render.
    pub next: u32,
}

/// Result of a multi-reference glitch-corrected iteration render (`render_corrected_iter`):
/// the merged raw iteration buffer plus per-render telemetry summed across the base pass and
/// every correction pass (so a colored corrected export can report real GPU counters/time
/// instead of the zeros `color_iter_buffer` alone produces).
pub(crate) struct CorrectedIter {
    pub pixels: Vec<f32>,
    pub refs_used: usize,
    pub residual: usize,
    pub counters: [u64; fractadyne_gpu::COUNTER_SLOTS],
    pub iterate_ms: f64,
}

// `FRACTADYNE_TRACE` tracing moved to `diag` (categories: req, ref, gpu, tile, glitch —
// see diag::trace_on). The per-frame GPU sizing trace is the `tile` category below: deep-zoom
// frames are sized by a feedback loop against the GPU watchdog and the UI thread, and its
// inputs are invisible from outside — a device-lost crash and a hung window can both come from
// the same over-large dispatch, while the obvious knobs (`max_iter`, `aa`) turn out not to move
// it at all because the resolution shrink normalises the frame back onto the budget. Reading
// the real numbers settles in one run what guessing at the constants does not.

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
    /// Stripe frequency the BLA's `agg_stripe` lane was built with, so the live path can detect a
    /// frequency-slider change against it and rebuild (see `RefCache::bla_stripe_freq`).
    bla_stripe_freq: f64,
    /// Trap type the BLA's `agg_trap` lane was built with (same rebuild-on-change purpose).
    bla_trap_type: u32,
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
    /// Full-precision running state at the orbit's end, cached so a deeper same-point rebuild can
    /// extend it (see [`RefCache::orbit_tail`]). `None` when the orbit escaped (complete) or is empty.
    /// (The precision it was built at — depth precision + reuse HEADROOM — travels in `prec`.)
    orbit_tail: Option<fractadyne_core::OrbitTail>,
}

/// A cached reference the worker may EXTEND instead of rebuilding from scratch: the prior orbit's
/// point, df32 samples, full-precision tail, and the precision it was built at. Supplied by
/// `build_params` when the current recompute is a deeper zoom at a still-in-view reference.
struct ReuseRef {
    point: [fractadyne_core::BigFloat; 2],
    prefix: std::sync::Arc<Vec<[f32; 4]>>,
    tail: fractadyne_core::OrbitTail,
    prec: usize,
}

/// One script-playback lookahead slot: an in-flight (or finished, held) future-reference build,
/// tagged with the log2 magnification it targets so the queue spaces targets without duplicates.
/// See [`FractadyneApp::playback_ref_prefetch`].
pub(crate) struct RefPrefetchSlot {
    rx: Option<std::sync::mpsc::Receiver<RecomputeResult>>,
    ready: Option<RecomputeResult>,
    target_l2: f64,
}

/// Owned, `Send` inputs for an off-thread reference recompute.
struct RecomputeInputs {
    center_bf: [fractadyne_core::BigFloat; 2],
    span: (fractadyne_core::FloatExp, fractadyne_core::FloatExp),
    span_mantissa: fractadyne_core::SpanMantissa,
    delta_exp: i32,
    gpu_iter: u32,
    /// Ceiling on the reference-orbit LENGTH (samples) for THIS build, on top of the global
    /// device-buffer cap. The LIVE path sets it to `LIVE_REF_CAP` so the reference + BLA stay
    /// small (freeze-safe) while pixels iterate PAST it to `gpu_iter` by rebasing — decoupling the
    /// preview's iteration depth from the reference size. Export sets `u32::MAX` (full appetite;
    /// only the device buffer bounds it). See `build_reference_from_point`.
    orbit_len_cap: u32,
    precision: usize,
    julia: bool,
    formula: u32,
    julia_c: (f64, f64),
    do_sa: bool,
    bla_dc_max: Option<fractadyne_core::FloatExp>,
    /// Stripe-average frequency to bake into the BLA `agg_stripe` lane (the aggregate is
    /// freq-specific). Irrelevant unless the stripe coloring rides BLA, but always carried.
    stripe_freq: f64,
    /// Orbit-trap type (0 point / 1 cross / 2 circle) to bake into the BLA `agg_trap` lane (the
    /// aggregate is trap-type-specific). Irrelevant unless orbit-trap rides BLA, but always carried.
    trap_type: u32,
    /// A prior reference the worker may extend instead of rebuilding (deeper zoom, same in-view
    /// point). `None` forces a fresh best-reference pick + full orbit build.
    reuse: Option<ReuseRef>,
}

/// Aux coloring aggregates for the BLA tree, derived from a reference orbit. Triangle-inequality
/// needs `cmag` (= |c_ref| = |Z_1|, since Mandelbrot's Z_0 = 0 ⇒ Z_1 = c) and `power` (= 2, because
/// BLA is Mandelbrot-only) — both reference-intrinsic, so the tree caches per reference with no live
/// dependency. Point-trap uses the default trap aggregate (trap_type 0). `stripe_freq` stays default
/// (stripe's per-node aggregate isn't folded yet — it would need a rebuild on the freq slider).
fn aux_agg_from_orbit(orbit: &[[f32; 4]], stripe_freq: f64, trap_type: u32) -> fractadyne_core::AuxAggParams {
    let cmag = orbit
        .get(1)
        .map(|z| {
            // `sample_xy` decodes extended-range dip samples (NaN-marked); orbit[1] = c is never
            // one in practice, but stay marker-safe.
            let (x, y) = fractadyne_core::sample_xy(z);
            (x * x + y * y).sqrt()
        })
        .unwrap_or(0.0);
    // `stripe_freq` and `trap_type` must be the LIVE values: the aux aggregates they select
    // (stripe's Σ(0.5+0.5·sin(freq·arg Z)); trap's running min of aux_trap_dist(Z, trap_type)) are
    // parameter-specific, so a stripe/cross/circle-trap BLA tree rebuilds when its slider changes
    // (see the live rebuild in build_params). `power` stays 2 (BLA is Mandelbrot-only).
    fractadyne_core::AuxAggParams { trap_type, stripe_freq, cmag, power: 2.0 }
}

/// Device-derived ceiling on the stored reference-orbit LENGTH (in samples), set once at startup
/// from `max_storage_buffer_binding_size`. The orbit and its BLA tree upload together as one storage
/// buffer sized ~9× the orbit (16 B/sample); past this the bind exceeds the GPU limit and the live
/// path panics in `make_iter_bg` (the same overflow the export path returns `OrbitTooLarge` for).
/// We cap the orbit BUILD, never the render's `max_iter`: a deep INTERIOR reference that never
/// escapes is truncated to fit, while pixels still iterate to the full count by REBASING past the
/// truncated orbit. An escaping reference shorter than the cap (every corpus location — loc 15's
/// 918 516-sample orbit sits just under the ~928 k cap on a 128 MB device) is therefore untouched.
/// Unset (tests / before device init) ⇒ `u32::MAX`, i.e. no cap.
static ORBIT_LEN_CAP: std::sync::OnceLock<u32> = std::sync::OnceLock::new();

/// Record the orbit-length ceiling from the GPU's storage-buffer binding limit (bytes). Called once
/// at startup from the render state. Orbit + BLA share one binding and total ~9× the orbit at 16 B
/// each, so the orbit budget is `limit / 16 / 9`, less a margin for the BLA's exact ratio varying a
/// little above 8×. First value wins (idempotent).
pub(crate) fn init_orbit_len_cap(max_storage_buffer_binding_size: u32) {
    let limit = max_storage_buffer_binding_size as u64;
    let cap = (limit / 16 / 9).saturating_sub(4096).clamp(4096, u32::MAX as u64) as u32;
    let _ = ORBIT_LEN_CAP.set(cap);
    crate::diag::log_line(
        "gpu",
        &format!("reference-orbit length cap = {cap} samples (storage-binding limit {limit} B)"),
    );
}

/// The orbit-build length ceiling — `u32::MAX` (no cap) when unset (tests / before device init).
fn orbit_len_cap() -> u32 {
    ORBIT_LEN_CAP.get().copied().unwrap_or(u32::MAX)
}

/// Pick a reference (once) then build its orbit + series-approximation skip + BLA tree to
/// `inp.gpu_iter` — the slow arbitrary-precision work. Pure and `Send`, so it runs on a worker
/// thread; mirrors the synchronous `compute_reference` + `series_skip_for` + `build_bla`. The
/// progressive cold start (`recompute_worker_staged`) reuses the `pick`/`build` split below.
fn recompute_worker(inp: RecomputeInputs) -> RecomputeResult {
    // The bignum orbit build is the longest silent phase in the app — name it for the
    // watchdog/crash report before starting (D1.2).
    crate::diag::breadcrumb(format!(
        "building reference: iter={} prec={}",
        inp.gpu_iter, inp.precision
    ));
    // Deep-dive reuse: when the prior reference is still valid for this (deeper) frame, EXTEND its
    // orbit instead of recomputing every bignum step (the orbit build dominates a deep frame). Falls
    // back to a fresh pick + full build when there's no reusable orbit or it no longer qualifies.
    if let Some(res) = try_reuse_reference(&inp) {
        return res;
    }
    // pick_reference scores candidate orbits in bignum; at extreme depth (cold, no reusable
    // reference) this is the DOMINANT export cost — ~7 s of a ~15 s me148 render, far more
    // than the orbit build (~1 s) or BLA build (~0.8 s). Timed here so the lever is visible.
    let t_pick = Instant::now();
    let rp = pick_reference(&inp);
    if crate::diag::trace_on("ref") {
        crate::diag::trace("ref", format!("pick_reference (candidate scoring) took {:.0}ms", t_pick.elapsed().as_secs_f64() * 1000.0));
    }
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

/// Extra orbit precision above the depth's exact requirement, so successive DEEPER rebuilds within
/// this band can extend the cached orbit (see [`try_reuse_reference`]) instead of recomputing it.
/// The orbit is stored as df32 with ample accuracy headroom, so building at a higher precision
/// leaves the df32 samples byte-identical — this only grows the accumulation margin, not the render.
const REF_PREC_HEADROOM: usize = 128;

/// Hard drift ceiling for reusing a cached reference: a point beyond this fraction of a span
/// off-centre is re-anchored (fresh pick) instead. Held at the `out_of_view` gate that already
/// filters the caller, so a reused reference is never worse than one the live path already trusts.
const REUSE_MAX_DRIFT: f64 = 0.7;

/// Build the orbit (to `orbit_iter`, at reuse-headroom precision) + series-approximation skip + BLA
/// tree for a PRE-CHOSEN reference point `rp`. Split out of `recompute_worker` so a progressive cold
/// start builds a short (coarse) orbit and the full one from the SAME point (no pop on refine).
fn build_reference_from_point(
    rp: [fractadyne_core::BigFloat; 2],
    orbit_iter: u32,
    do_sa: bool,
    inp: &RecomputeInputs,
) -> RecomputeResult {
    use fractadyne_core as fc;
    let t = Instant::now();
    let orbit_prec = inp.precision + REF_PREC_HEADROOM;
    let zero = fc::BigFloat::from_f64(0.0, orbit_prec);
    let (z0x, z0y, cx0, cy0) = if inp.julia {
        (
            rp[0].clone(),
            rp[1].clone(),
            fc::BigFloat::from_f64(inp.julia_c.0, orbit_prec),
            fc::BigFloat::from_f64(inp.julia_c.1, orbit_prec),
        )
    } else {
        (zero.clone(), zero, rp[0].clone(), rp[1].clone())
    };
    // Cap the stored orbit LENGTH to the smaller of this build's cap (`inp.orbit_len_cap` — the
    // LIVE path keeps the reference small for freeze safety) and the GPU storage-binding limit
    // (`orbit_len_cap()`). `orbit_iter` (the render's iteration budget, passed unchanged to
    // `finish_reference`) is untouched — pixels rebase past the truncated orbit to reach the full
    // count. An escaping reference shorter than the cap builds identically (it stops at escape).
    let (o, len, tail) = fc::reference_orbit_t(
        &z0x, &z0y, &cx0, &cy0, inp.formula,
        orbit_iter.min(inp.orbit_len_cap).min(orbit_len_cap()),
        orbit_prec,
    );
    let ref_ms = t.elapsed().as_secs_f64() * 1000.0;
    finish_reference(rp, o, len, tail, orbit_prec, orbit_iter, do_sa, inp, ref_ms)
}

/// Assemble a `RecomputeResult` from an already-built (fresh or extended) orbit: derive the
/// truncated/`partial` flag + the extendable `orbit_tail`, then build the series-approximation skip
/// and the BLA tree. Shared by the fresh build and the reuse/extend path so both produce identical
/// downstream fields for a given orbit. `partial` (never escaped) marks a truncated reference — the
/// render caps iterations to it, and only such an orbit carries a tail worth extending.
#[allow(clippy::too_many_arguments)] // REFACTOR-PLAN Phase 2/4: fold into a reference-inputs struct
fn finish_reference(
    rp: [fractadyne_core::BigFloat; 2],
    o: Vec<[f32; 4]>,
    len: u32,
    tail: fractadyne_core::OrbitTail,
    orbit_prec: usize,
    orbit_iter: u32,
    do_sa: bool,
    inp: &RecomputeInputs,
    ref_ms: f64,
) -> RecomputeResult {
    use fractadyne_core as fc;
    let partial = !tail.escaped;
    // A SHORT ESCAPED reference (deep EXTERIOR): the orbit escapes early (e.g. ~3.3k iters at 1e261×)
    // and this orbit geometry has an EARLY-ITERATION perturbation glitch. A BLA view normally turns
    // SERIES APPROXIMATION off ("BLA subsumes SA") — which leaves that glitch EXPOSED and shatters the
    // view into "distorted overlapping tiles" (the e260 exterior artifact; confirmed headlessly:
    // independent of BLA eps / dc_max / max-skip-level — it was never the skip validity). SA masks the
    // glitch by seeding δz analytically PAST the early iterations, so force it back on here. The BLA is
    // KEPT (SA seeds, the BLA skips from there): dropping it instead would make a nearby MINIBROT's
    // interior/late-escaping boundary pixels iterate to max_iter un-accelerated (slow, and a capped
    // stand-in gives a hard borderless minibrot). Surviving references (partial: reached the cap
    // without escaping) already run SA-or-BLA correctly. Gate on the escape being well short of the
    // budget so this only touches genuine short escapers.
    let short_escaper = inp.bla_dc_max.is_some()
        && !partial
        && (len as u64).saturating_mul(2) < inp.gpu_iter.max(1) as u64;
    // Keep the tail even for a COMPLETE (escaped) orbit: it can't be EXTENDED, but it can still be
    // REUSED as-is (same point + orbit) so a rebuild doesn't re-pick a fresh reference — which at
    // extreme depth renders a hair differently each time and makes the view "jump" on zoom.
    let orbit_tail = Some(tail);
    let orbit = std::sync::Arc::new(o);
    // Series approximation for the chosen reference (at the exact depth precision, not the headroom).
    // Forced on for a short escaper (see above) even though `do_sa` was cleared by BLA eligibility.
    let t_sa = Instant::now();
    let sa = if do_sa || short_escaper {
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
    // BLA tree (Mandelbrot deep only; empty otherwise). Built with the same conservative dc_max the
    // live path uses so the main thread reuses it across pans.
    //
    // SKIP the BLA for a SHORT ESCAPED reference (deep EXTERIOR). At such a spot every candidate
    // orbit escapes early (e.g. ~3.3k iters at 1e261×) and most pixels escape in tens of iterations,
    // so the BLA saves almost nothing — but its linear skips accumulate df32 coefficient error at
    // this orbit geometry and shatter the view into "distorted overlapping tiles" (confirmed
    // headlessly: BLA on = blocks, BLA off = pristine, independent of eps / dc_max / max-skip-level).
    // A SURVIVING reference (`partial`: reached the iter cap without escaping — deep interior/
    // boundary) KEEPS its BLA: there it's both essential for speed and accurate. Gate on the escape
    // being well short of the budget so a rare long escaper (where the skip still pays for itself)
    // keeps its tree.
    let t_bla = Instant::now();
    let (bla, bla_dc_max_log2) = match inp.bla_dc_max {
        Some(dc_max) => {
            let levels = fc::build_bla_mandel(
                &orbit,
                dc_max,
                BLA_EPS,
                aux_agg_from_orbit(&orbit, inp.stripe_freq, inp.trap_type),
            );
            let arc = if levels.is_empty() {
                std::sync::Arc::new(Vec::new())
            } else {
                std::sync::Arc::new(fc::bla_to_gpu(&levels))
            };
            (arc, dc_max.log2())
        }
        _ => (std::sync::Arc::new(Vec::new()), f64::NEG_INFINITY),
    };
    let bla_ms = t_bla.elapsed().as_secs_f64() * 1000.0;
    crate::diag::breadcrumb(format!("reference built: len={len} iter={orbit_iter} prec={orbit_prec}"));
    if crate::diag::trace_on("ref") {
        // Timing is on the line now (design/diagnostics.md F14): the cold export reference
        // build dominates a deep render (~12 s of a ~15 s 2560×1440 me148 render, vs ~2.7 s
        // GPU), and this splits it into orbit / SA / BLA so the lever is visible. `ref_ms` is
        // the orbit build (fresh, or the reuse/extend time on that path).
        crate::diag::trace(
            "ref",
            format!(
                "len={len} iter={orbit_iter} prec={orbit_prec} partial={partial} \
                 escaped={} sa_skip={} bla_dc_max_log2={bla_dc_max_log2:.1} bla_nodes={} \
                 | orbit_ms={ref_ms:.0} sa_ms={series_ms:.0} bla_ms={bla_ms:.0}",
                orbit_tail.is_none() && !partial,
                sa.skip,
                bla.len(),
            ),
        );
    }
    RecomputeResult {
        orbit,
        orbit_len: len,
        rp,
        sa,
        bla,
        bla_dc_max_log2,
        bla_stripe_freq: inp.stripe_freq,
        bla_trap_type: inp.trap_type,
        prec: orbit_prec,
        iter: orbit_iter,
        ref_ms,
        series_ms,
        bla_ms,
        partial,
        orbit_tail,
    }
}

/// Extend a cached reference (`inp.reuse`) to the current depth instead of rebuilding it from
/// scratch — the deep-dive win (the bignum orbit build dominates a deep frame). Returns `None`
/// (→ fresh build) when there's no reusable reference, it lacks the precision headroom for this
/// depth, or its point has drifted too far to remain a valid in-view reference. A COMPLETE (escaped)
/// orbit IS reusable — `extend_reference_orbit` returns it unchanged — which keeps the SAME reference
/// across rebuilds instead of re-picking a fresh one (at extreme depth, re-picking a different valid
/// reference every ~0.16 octave made the view "jump" on zoom, since the render isn't perfectly
/// invariant there). Perturbation is invariant to *which* valid in-view reference is used, so a
/// reused reference renders the same image as a fresh one — the drift gate keeps it valid.
fn try_reuse_reference(inp: &RecomputeInputs) -> Option<RecomputeResult> {
    use fractadyne_core as fc;
    let reuse = inp.reuse.as_ref()?;
    if reuse.prec < inp.precision || reuse.prefix.is_empty() {
        return None;
    }
    // Re-verify the point is still a good in-view reference (defence in depth; the caller already
    // gated on `!out_of_view`, but this snapshot could in principle lag).
    let dx = fc::ref_offset_mantissa(&inp.center_bf[0], &reuse.point[0], inp.delta_exp, inp.precision)
        / inp.span_mantissa.x;
    let dy = fc::ref_offset_mantissa(&inp.center_bf[1], &reuse.point[1], inp.delta_exp, inp.precision)
        / inp.span_mantissa.y;
    if dx.abs() > REUSE_MAX_DRIFT || dy.abs() > REUSE_MAX_DRIFT {
        return None;
    }
    let t = Instant::now();
    let (cx0, cy0) = if inp.julia {
        (
            fc::BigFloat::from_f64(inp.julia_c.0, reuse.prec),
            fc::BigFloat::from_f64(inp.julia_c.1, reuse.prec),
        )
    } else {
        (reuse.point[0].clone(), reuse.point[1].clone())
    };
    // Same caps as the fresh build (see `orbit_len_cap`): don't extend a reused orbit past this
    // build's length cap (LIVE freeze safety) or the GPU buffer. `inp.gpu_iter` (the render
    // budget) still flows to `finish_reference` below unchanged — only the stored length is bounded.
    let (o, len, tail) = fc::extend_reference_orbit(
        &reuse.prefix,
        &reuse.tail,
        &cx0,
        &cy0,
        inp.formula,
        inp.gpu_iter.min(inp.orbit_len_cap).min(orbit_len_cap()),
        reuse.prec,
    );
    let ref_ms = t.elapsed().as_secs_f64() * 1000.0;
    Some(finish_reference(
        reuse.point.clone(),
        o,
        len,
        tail,
        reuse.prec,
        inp.gpu_iter,
        inp.do_sa,
        inp,
        ref_ms,
    ))
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
        vc.orbit_tail = res.orbit_tail; // extendable tail for the next deeper rebuild (None if escaped)
        vc.orbit_id = vc.orbit_id.wrapping_add(1);
        vc.last_recompute = Some(Instant::now());
        vc.sa = res.sa;
        vc.sa_key = (vc.orbit_id, res.iter);
        vc.bla = res.bla;
        vc.bla_id = vc.orbit_id;
        vc.bla_dc_max_log2 = res.bla_dc_max_log2;
        vc.bla_stripe_freq = res.bla_stripe_freq;
        vc.bla_trap_type = res.bla_trap_type;
        self.perf.recompute_ms = res.ref_ms;
        self.perf.recompute_total += 1;
        self.perf.rate_count += 1;
    }

    /// Script-playback reference LOOKAHEAD: a tour knows its whole future camera path, so build the
    /// references the dive is ABOUT to need on otherwise-idle cores while the current one serves the
    /// view. A small QUEUE of slots covers the next `PREFETCH_SLOTS × PREFETCH_OCT` octaves
    /// concurrently, so even the fastest dive phase always has the next reference finished before it
    /// arrives. Each pump: (1) collect finished builds, (2) install the one whose depth-validity
    /// window the dive has reached (the same `0.85..1.1` BLA-lag window `build_params` steers by, so
    /// it reads as freshly built), (3) top the queue back up, each new slot `PREFETCH_OCT` octaves
    /// past the deepest queued target, built exactly as `build_params` would build it there.
    ///
    /// Purely ADDITIVE to the reactive rebuild path: a build that misses its window (or a tour that
    /// pans/changes fractal) is simply dropped and the reactive path covers it as before. Cleared on
    /// tour start/end and `invalidate_refs` so a stale prefetch can never install.
    pub(crate) fn playback_ref_prefetch(&mut self, pb: &crate::scripting::Playback, e: f64) {
        /// Slot spacing (octaves). Sets the ACTIVE reference's peak lag between installs: an
        /// install restores lag ≈ 1.0, and the next slot becomes installable when the view reaches
        /// its window ⇒ peak active lag = 1.0 + spacing − 0.14. That peak MUST stay below
        /// `PACE_LAG_LO` (1.5) and `DEEP_LAG_HOLD` (1.8), or the tail of every inter-install
        /// interval rhythmically clips the pacer / freeze-reproject zones — the residual "visible
        /// jerkiness from ~e400" with 1.0 spacing (peak 1.86: below ~e400 the reactive path patched
        /// the tail in ~10–30 ms, past it builds cost 70–130 ms and lose the race at the fast dive
        /// phase). 0.5 ⇒ peak lag ≈ 1.36 — clear of both thresholds.
        const PREFETCH_OCT: f64 = 0.5;
        /// Lookahead depth (slots × PREFETCH_OCT octaves = ~3 octaves, ~0.3 s of runway at the
        /// fastest ~10 oct/s phase vs 0.1–0.6 s per build; consumption 2 installs/octave). Each
        /// build's candidate scoring already fans out across all cores, so concurrent slots briefly
        /// oversubscribe threads — harmless for compute-bound bursts.
        const PREFETCH_SLOTS: usize = 6;
        const LN_2: f64 = std::f64::consts::LN_2;
        // 1) Collect finished builds into their slots.
        for slot in &mut self.ref_prefetch {
            if let Some(rx) = slot.rx.take() {
                match rx.try_recv() {
                    Ok(res) => slot.ready = Some(res),
                    Err(std::sync::mpsc::TryRecvError::Empty) => slot.rx = Some(rx),
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {} // worker died → culled
                }
            }
        }
        // 2) Install the slot whose validity window the dive has reached; cull slots the dive
        //    already zoomed past (lag below the window ⇒ their BLA would read as out-of-range)
        //    and dead workers. Early slots (lag above the window) are held.
        let scale = self.viewport.gpu_scale();
        let needed = Self::bla_dc_max(scale.span_mantissa, scale.delta_exp).log2();
        let mut install: Option<RecomputeResult> = None;
        self.ref_prefetch.retain_mut(|slot| {
            if slot.rx.is_none() && slot.ready.is_none() {
                return false; // worker died without a result
            }
            let Some(res) = &slot.ready else { return true }; // still building
            let lag = res.bla_dc_max_log2 - needed;
            if !lag.is_finite() || lag < 0.86 {
                return false; // window missed / no BLA — reactive path covers it
            }
            if lag <= 1.09 {
                if install.is_none() {
                    install = slot.ready.take();
                }
                return false;
            }
            true // early — hold until the dive arrives
        });
        if let Some(res) = install {
            if res.prec >= self.viewport.precision {
                if crate::diag::trace_on("ref") {
                    crate::diag::trace(
                        "ref",
                        format!("lookahead install: len={} prec={}", res.orbit_len, res.prec),
                    );
                }
                self.install_recompute(0, res); // seamless swap — no reactive stall
            }
        }
        // 3) Top the queue back up (single main view only; a future fractal/julia/dual switch
        //    means a prefetched reference wouldn't match — stop at those segments).
        if self.dual || self.julia_mode {
            return;
        }
        let cur_l2 = pb.sample(e).logmag / LN_2;
        // Cull overshot slots: a slot targeting far past the queue's span can only exist if the
        // tour was re-timed or a target overshot — it would sit "held" for minutes, wasting its
        // queue position while the reactive path fills the gap (exactly the beta.9 field failure:
        // coarse probe steps on an easing tour built slots +46…+293 octaves ahead).
        let max_ahead = cur_l2 + PREFETCH_OCT * (PREFETCH_SLOTS as f64 + 1.0) + 1.0;
        self.ref_prefetch.retain(|s| s.target_l2 <= max_ahead);
        while self.ref_prefetch.len() < PREFETCH_SLOTS {
            // Next target: PREFETCH_OCT octaves past the deepest queued target (or the current
            // depth). Find the first script time that reaches it — a coarse scan for a bracket,
            // then BISECTION to the crossing (samples are cheap keyframe interpolation), so the
            // built target sits AT next_l2 rather than wherever a coarse probe step lands (on an
            // easing tour those overshoot by orders of magnitude → far-future dead slots). No
            // bracket ⇒ the tour is slow/at rest/zooming out — nothing to prefetch.
            let deepest = self
                .ref_prefetch
                .iter()
                .map(|s| s.target_l2)
                .fold(cur_l2, f64::max);
            let next_l2 = deepest + PREFETCH_OCT;
            let reaches = |tau: f64| pb.sample((e + tau).min(pb.total)).logmag / LN_2 >= next_l2;
            let (mut lo, mut hi) = (0.0_f64, f64::NAN);
            for tau in [0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0] {
                if reaches(tau) {
                    hi = tau;
                    break;
                }
                lo = tau;
            }
            if !hi.is_finite() {
                break;
            }
            for _ in 0..24 {
                let mid = 0.5 * (lo + hi);
                if reaches(mid) {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            let s = pb.sample((e + hi).min(pb.total));
            if s.fractal != self.fractal || s.julia || s.dual {
                break;
            }
            // Build the FUTURE frame's recompute inputs exactly as `build_params` will when the
            // dive gets there (same precision/iter/caps/BLA dc_max), so the installed result is
            // indistinguishable from a reactive rebuild at that depth.
            let target_l2 = s.logmag / LN_2;
            let mut vp =
                fractadyne_core::Viewport::new(self.viewport.width_px, self.viewport.height_px);
            vp.set_center_log2mag(s.cx, s.cy, target_l2);
            let mag = vp.magnification();
            let mode = RenderMode::select(self.fractal.supports_perturbation(), mag);
            if mode.is_direct() {
                break; // shallow frames rebuild in microseconds — nothing to prefetch
            }
            let l2 = vp.log2_magnification();
            let precision = fractadyne_core::precision_for_octaves(l2.max(0.0).ceil() as u64);
            let eff_iter = if self.render_cfg.auto_iter {
                vp.recommended_max_iter(self.render_cfg.max_iter)
            } else {
                self.render_cfg.max_iter
            };
            let gpu_iter = eff_iter.min(500_000).min(crate::zoom_iter_cap(l2).max(256));
            let ref_build_iter = gpu_iter.saturating_add(32 * 256).min(500_000);
            let span = vp.complex_span_fe();
            let scale = vp.gpu_scale();
            let (span_mantissa, delta_exp) = (scale.span_mantissa, scale.delta_exp);
            let bla_will_build = self.bla_eligible(mode, false);
            let do_sa = self.fractal.formula_id() <= 3
                && !self.coloring.color_method.blocks_iter_skip()
                && self.render_cfg.series_approx
                && !bla_will_build;
            // Hand the worker the cached orbit for extend/as-is reuse — `try_reuse_reference`
            // re-validates drift/precision against the FUTURE frame and falls back to a fresh pick.
            let vc = &self.ref_cache[0];
            let reuse = match (vc.ref_pt.clone(), vc.orbit_tail.clone()) {
                (Some(point), Some(tail)) if !vc.orbit.is_empty() => Some(ReuseRef {
                    point,
                    prefix: vc.orbit.clone(),
                    tail,
                    prec: vc.orbit_prec,
                }),
                _ => None,
            };
            let inputs = RecomputeInputs {
                center_bf: [vp.center_x.clone(), vp.center_y.clone()],
                span,
                span_mantissa,
                delta_exp,
                gpu_iter: ref_build_iter,
                orbit_len_cap: crate::LIVE_REF_CAP,
                precision,
                julia: false,
                formula: self.fractal.formula_id(),
                julia_c: self.julia_c,
                do_sa,
                bla_dc_max: bla_will_build
                    .then(|| Self::bla_dc_max(span_mantissa, delta_exp).mul_pow2(1.0)),
                stripe_freq: self.coloring.stripe_freq as f64,
                trap_type: self.coloring.trap_type as u32,
                reuse,
            };
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let _ = tx.send(recompute_worker(inputs));
            });
            self.ref_prefetch.push(RefPrefetchSlot { rx: Some(rx), ready: None, target_l2 });
        }
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
        // The persisted tree's stripe-frequency / trap-type aren't stored (not part of the view-key),
        // so mark them unknown — a stripe/trap render then rebuilds once against the live params
        // (cheap, ~20 ms).
        vc.bla_stripe_freq = f64::NEG_INFINITY;
        vc.bla_trap_type = u32::MAX;
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
            orbit_len_cap: u32::MAX, // export: full appetite (only the device buffer bounds it)
            precision,
            julia,
            formula: self.fractal.formula_id(),
            julia_c: self.julia_c,
            do_sa,
            bla_dc_max,
            stripe_freq: self.coloring.stripe_freq as f64,
            trap_type: self.coloring.trap_type as u32,
            reuse: None, // one-shot export: always a fresh build
        }
    }

    /// Build the BLA tree (GPU-flattened) for a Mandelbrot deep view, or `None` when BLA
    /// doesn't apply (disabled, not floatexp/Mandelbrot/non-Julia, or an aux coloring method
    /// that BLA would skip). `dx`/`dy` are the reference-offset mantissas and `span_mantissa`
    /// the view span — both scaled by `2^delta_exp` — used for the worst-case `|δc|`.
    /// Whether BLA applies to this render (deep floatexp Mandelbrot, non-Julia, non-aux coloring).
    fn bla_eligible(&self, mode: RenderMode, julia: bool) -> bool {
        // Aux coloring blocks iteration-skipping — EXCEPT the methods whose per-BLA-node aggregate is
        // folded on each skip (GPU-validated: the fold render matches the full render), which ride BLA
        // at full speed instead of paying full floatexp iterations. Decomposition isn't skip-safe (its
        // angular cells shift under the approximation) — it stays gated.
        let method = self.coloring.color_method;
        // Aux methods with a GPU-validated BLA-skip fold: orbit-trap (all trap types — the `agg_trap`
        // lane is built for the live trap_type), triangle-inequality (reference-intrinsic aggregate),
        // and stripe-average (Σ stripe terms — freq-specific). The trap-type/frequency the aggregate
        // was built with is tracked so the tree rebuilds when the slider changes (see build_params).
        let aux_bla_ok = method.to_u32() == 3 // OrbitTrap (point / cross / circle)
            || method.to_u32() == 2 // TriangleIneq
            || method.to_u32() == 1; // Stripe average
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
        let levels = fractadyne_core::build_bla_mandel(
            orbit,
            dc_max,
            BLA_EPS,
            aux_agg_from_orbit(orbit, self.coloring.stripe_freq as f64, self.coloring.trap_type as u32),
        );
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
                .min(zoom_iter_cap(log2mag).max(256))
        } else {
            // Auto-iter OFF is an explicit instruction — honor the count verbatim. The cap is an
            // auto-mode nicety; applied here it silently rendered deep validation-corpus locations
            // interior-black (their structure escapes above the cap) while Fraktaler-3 used the
            // full count, breaking the corpus contract of "same iterations, both apps".
            self.render_cfg.max_iter
        };
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
            // Cap at the zoom-appropriate count: avoids noise from over-resolving sub-pixel dust,
            // and keeps the export fast/responsive.
            vp.recommended_max_iter(self.render_cfg.max_iter)
                .min(zoom_iter_cap(log2mag).max(256))
        } else {
            // Auto-iter OFF is an explicit instruction — honor the count verbatim (must stay in
            // lock-step with `export_reference_inputs_for` above). Capping it silently rendered
            // deep validation-corpus locations interior-black (their structure escapes above the
            // cap) while Fraktaler-3 used the full count — breaking the corpus contract of "same
            // iterations, both apps".
            self.render_cfg.max_iter
        };
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
        // The effective render manifest: kept globally for crash reports (D1.3) and printed
        // under the `req` trace category. This is the record that killed F8 — it states what
        // a render was *actually* asked to do, not what the caller believed.
        let manifest = format!(
            "mode={} iter={eff_iter} orbit_len={orbit_len} sa_skip={} bla_on={bla_on} \
             delta_exp={delta_exp} span_m=({:.6e},{:.6e}) ref_off={:?} prec={precision} \
             size={width}x{height} ss={}",
            mode.to_u32(),
            sa.skip,
            scale.span_mantissa.x,
            scale.span_mantissa.y,
            ref_offset,
            self.export.ss.max(1),
        );
        crate::diag::set_manifest(manifest.clone());
        crate::diag::trace("req", manifest);

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
        deadline: Option<Instant>,
    ) -> Option<CorrectedIter> {
        // Per-dispatch work budget for the tiled correction renders (nominal steps = tile²·max_iter).
        // Smaller than the export path's (fractadyne-gpu `TILE_WORK_BUDGET` = 2e10) because
        // correction passes run with BLA OFF, so deep-interior "dark core" pixels cost ~50× the
        // nominal per-pixel work; small tiles keep each GPU dispatch well inside the TDR window and
        // leave the loop interruptible (between tiles) by `deadline`. This is the fix for the
        // >1h uninterruptible-dispatch pathology (TODO.md Open bugs).
        const CORRECT_WORK_BUDGET: u64 = 2_000_000_000;
        let mut req = self.current_export_request_for(vp, julia);
        req.width = width;
        req.height = height;
        req.ss = 1;
        req.glitch_on = 1;
        // Accumulate GPU counters + iterate time across the base pass and every correction
        // pass, so the colored result carries real telemetry (color_iter_buffer alone reports
        // zeros, which is why a glitch-corrected export used to log all-zero counters).
        let mut counters = [0u64; fractadyne_gpu::COUNTER_SLOTS];
        let mut iterate_ms = 0.0f64;
        let base = fractadyne_gpu::render_iter_tiled(device, queue, &req, CORRECT_WORK_BUDGET, deadline).ok()?;
        for (c, v) in counters.iter_mut().zip(base.counters) {
            *c += v;
        }
        iterate_ms += base.iterate_ms;
        let mut merged = base.pixels;
        // Direct path never glitches; nothing to correct.
        if RenderMode::from_u32(req.mode).is_direct() {
            return Some(CorrectedIter { pixels: merged, refs_used: 1, residual: 0, counters, iterate_ms });
        }
        let center_bf = [vp.center_x.clone(), vp.center_y.clone()];
        let span = vp.complex_span_fe();
        let precision = vp.precision;
        let eff_iter = req.max_iter;
        let delta_exp = req.delta_exp;
        let (w, h) = (width as usize, height as usize);
        let mut refs_used = 1usize;
        // The multi-reference loop is the app's worst historical time sink (>1 h at 1e500×,
        // F3/F14) — every pass names itself for the watchdog and, under the `glitch` trace
        // category, reports its cost, so "slow" and "hung" are distinguishable.
        let t_glitch = Instant::now();

        for _ in 1..max_refs {
            // Time-box: if the deadline has passed, stop and return the best-effort merge so far
            // (partial correction beats an unbounded hang; the caller colors what we have).
            if deadline.is_some_and(|d| Instant::now() >= d) {
                crate::diag::breadcrumb(format!(
                    "glitch correction: time-boxed after {} refs, {:.1}s",
                    refs_used,
                    t_glitch.elapsed().as_secs_f64()
                ));
                break;
            }
            // Glitched pixels carry the -2 sentinel (r < -1.5); interior is -1, escaped ≥ 0.
            let glitch: Vec<usize> = (0..w * h).filter(|&i| merged[i * 4] < -1.5).collect();
            if glitch.is_empty() {
                break;
            }
            crate::diag::breadcrumb(format!(
                "glitch correction: pass {}/{max_refs}, {} px glitched, {:.1}s elapsed",
                refs_used,
                glitch.len(),
                t_glitch.elapsed().as_secs_f64(),
            ));
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
            // Tiled + deadline-aware: a pass over the dark cores (BLA off) is split into short
            // dispatches; if the deadline lands mid-pass the tiled render returns Canceled and we
            // keep the merge accumulated by earlier passes rather than blocking on one huge dispatch.
            let pass_res = match fractadyne_gpu::render_iter_tiled(device, queue, &r, CORRECT_WORK_BUDGET, deadline) {
                Ok(p) => p,
                Err(_) => break,
            };
            for (c, v) in counters.iter_mut().zip(pass_res.counters) {
                *c += v;
            }
            iterate_ms += pass_res.iterate_ms;
            let pass = pass_res.pixels;
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
        if crate::diag::trace_on("glitch") {
            crate::diag::trace(
                "glitch",
                format!(
                    "correction done: refs={refs_used} residual={residual} in {:.1}s",
                    t_glitch.elapsed().as_secs_f64()
                ),
            );
        }
        Some(CorrectedIter { pixels: merged, refs_used, residual, counters, iterate_ms })
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
        deadline: Option<Instant>,
    ) -> Option<fractadyne_gpu::ExportResult> {
        // The multi-reference ITERATION is now tiled + deadline-bounded (see render_corrected_iter),
        // but the final COLOR pass still colors the merged buffer in one un-tiled texture set
        // (≈ 32 B/px), so this path is still bounded by the GPU's max 2-D texture dimension and a
        // conservative area cap to avoid OOM. Above either, fall back to the tiled (uncorrected)
        // path. ~32 MP covers 4K/5K/6K comfortably. `deadline` time-boxes the correction loop.
        const MAX_CORRECT_PX: u64 = 32_000_000;
        let max_dim = device.limits().max_texture_dimension_2d;
        if width > max_dim
            || height > max_dim
            || (width as u64) * (height as u64) > MAX_CORRECT_PX
            || self.coloring.color_method.needs_aux()
        {
            return None;
        }
        let ci = self.render_corrected_iter(device, queue, vp, julia, width, height, 64, deadline)?;
        let mut req = self.current_export_request_for(vp, julia);
        req.width = width;
        req.height = height;
        let mut res = fractadyne_gpu::color_iter_buffer(device, queue, &req, &ci.pixels).ok()?;
        // color_iter_buffer only colors; carry the correction's accumulated counters/time so
        // the perf line and counters reflect what the multi-reference render actually did.
        res.counters = ci.counters;
        res.iterate_ms = ci.iterate_ms;
        Some(res)
    }

    /// **Auto-normalized** export: color the smooth-iter field with the palette CYCLE mapped to the
    /// frame's actual escape-value range, instead of the fixed `cycle`. At extreme depth the
    /// smooth-iter counts are ~1e5–1e6 and vary steeply, so a fixed cycle aliases a correct escape
    /// field into per-pixel speckle (corpus 14/15). Two passes: the tiled iteration buffer
    /// (`render_iter_tiled`, any size), then a CPU min/max over the escaped pixels to set
    /// `cycle = sweeps/range`, `offset = -min·cycle`, then color; supersampled and box-downsampled.
    /// Returns `None` (caller falls back to the normal export) for aux coloring, an all-interior
    /// frame (nothing to normalize), or a supersampled size past the single-texture color cap.
    pub(crate) fn render_export_normalized(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        vp: &Viewport,
        julia: bool,
        width: u32,
        height: u32,
        ss: u32,
    ) -> Option<fractadyne_gpu::ExportResult> {
        const MAX_PX: u64 = 40_000_000; // single-texture color pass (~6K·6K); above → fall back
        let ss = ss.max(1);
        let (iw, ih) = (width * ss, height * ss);
        let max_dim = device.limits().max_texture_dimension_2d;
        if iw > max_dim
            || ih > max_dim
            || (iw as u64) * (ih as u64) > MAX_PX
            || self.coloring.color_method.needs_aux()
        {
            return None;
        }
        let mut req = self.current_export_request_for(vp, julia);
        req.width = iw;
        req.height = ih;
        req.ss = 1;
        // Pass 1 — supersampled iteration buffer (tiled → bounded dispatches, any size).
        let iter = fractadyne_gpu::render_iter_tiled(device, queue, &req, 20_000_000_000, None).ok()?;
        // Escape-value range over escaped pixels (channel 0 = smooth iter; < 0 = interior).
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for px in iter.pixels.chunks_exact(4) {
            let v = px[0];
            if v >= 0.0 {
                lo = lo.min(v);
                hi = hi.max(v);
            }
        }
        if hi < lo {
            return None; // all-interior frame: nothing to normalize → normal path
        }
        let range = (hi - lo).max(1.0);
        // With normalize on, the `cycle` slider means palette SWEEPS across the escape range.
        let sweeps = 0.5 + self.coloring.cycle * 6.0;
        req.cycle = sweeps / range;
        req.offset = -lo * req.cycle + self.coloring.offset;
        // Pass 2 — color the buffer with the normalized cycle.
        let mut res = fractadyne_gpu::color_iter_buffer(device, queue, &req, &iter.pixels).ok()?;
        // Box-downsample the supersampled colored buffer to the output resolution.
        if ss > 1 {
            let (ow, oh) = (width as usize, height as usize);
            let sw = iw as usize;
            let inv = 1.0 / (ss * ss) as f32;
            let mut out = vec![0.0f32; ow * oh * 4];
            for oy in 0..oh {
                for ox in 0..ow {
                    for c in 0..4 {
                        let mut s = 0.0f32;
                        for dy in 0..ss as usize {
                            for dx in 0..ss as usize {
                                s += res.pixels[((oy * ss as usize + dy) * sw + ox * ss as usize + dx) * 4 + c];
                            }
                        }
                        out[(oy * ow + ox) * 4 + c] = s * inv;
                    }
                }
            }
            res.pixels = out;
        }
        res.width = width;
        res.height = height;
        res.ss = ss;
        res.iterate_ms = iter.iterate_ms;
        res.counters = iter.counters;
        Some(res)
    }

    /// Advance a view's tiled settle by one tile and return its rect (base px). Grid geometry is
    /// frozen at grid start; the cursor walks the tiles CENTER-OUT, so the part of the image the
    /// user is looking at sharpens first. Returns a zero-area "hold" rect when another view already
    /// spent this frame's tile (two budget-sized dispatches in one submission could pair up past the
    /// watchdog), and repeats the final rect once the grid is complete (the GPU dedupes it, so a
    /// finished view costs nothing per frame).
    fn next_settle_tile(
        &mut self,
        vidx: usize,
        resolution: [u32; 2],
        ss: u32,
        gpu_iter: u32,
        tdr_steps: u64,
    ) -> [u32; 4] {
        let Some(state) = self.perf.tile_state[vidx].as_mut() else {
            return [0, 0, 0, 0]; // unreachable: `tiling` guarantees an armed state
        };
        // Freeze geometry on the first tile: the side is what one dispatch budget affords at this
        // ss/iteration count, so every tile lands near TDR_BUDGET_MS. A mid-grid budget update
        // reshapes nothing; it applies to the NEXT grid (or the next ss stage).
        let (res, geo_ss, side) = match state.geo {
            Some(g) if g.0 == resolution && g.1 == ss => g,
            _ => {
                let per_px = (gpu_iter.max(1) as u64).saturating_mul((ss as u64) * (ss as u64));
                let side = ((tdr_steps / per_px.max(1)) as f64).sqrt() as u32;
                let side = side.clamp(16, resolution[0].max(resolution[1]).max(16));
                state.geo = Some((resolution, ss, side));
                state.next = 0;
                (resolution, ss, side)
            }
        };
        let _ = geo_ss;
        let cols = res[0].div_ceil(side).max(1);
        let rows = res[1].div_ceil(side).max(1);
        let n = cols * rows;
        // Center-out visit order: sort tile indices by their center's distance from the grid center.
        // n ≤ ~TDR_MAX_TILES + rounding, so sorting per frame is trivial.
        let mut order: Vec<u32> = (0..n).collect();
        let (cx, cy) = ((cols as f64 - 1.0) / 2.0, (rows as f64 - 1.0) / 2.0);
        let dist = |i: u32| {
            let (x, y) = ((i % cols) as f64 - cx, (i / cols) as f64 - cy);
            x * x + y * y
        };
        order.sort_by(|&a, &b| dist(a).partial_cmp(&dist(b)).unwrap_or(std::cmp::Ordering::Equal));
        let done = state.next >= n;
        let idx = order[state.next.min(n - 1) as usize];
        let rect = |i: u32| {
            let x = (i % cols) * side;
            let y = (i / cols) * side;
            [x, y, side.min(res[0] - x), side.min(res[1] - y)]
        };
        if done {
            self.perf.tile_pending[vidx] = false;
            return rect(idx); // repeat the final rect: (key, tile) unchanged → GPU skips
        }
        // Hold while the OTHER view is busy: its tile (turn token), its own budget-sized
        // re-iterate (two ~TDR_BUDGET_MS dispatches in one egui submission pair up toward the
        // watchdog), or the user interacting in it (a ~1 s tile per frame would pin the
        // interactive panel at ~1 fps for the grid's whole duration). Draw order means the
        // guard sees the other view's markers one frame late in one direction — a residual,
        // rare single overlap, still inside the watchdog with TDR_BUDGET_MS' ~2.2x margin.
        let other = 1 - vidx;
        let f = self.perf.frame_idx;
        if self.perf.tile_turn == f
            || f.saturating_sub(self.perf.fe_iter_frame[other]) <= 1
            || f.saturating_sub(self.perf.interact_frame[other]) <= 1
        {
            self.perf.tile_pending[vidx] = true;
            return [0, 0, 0, 0];
        }
        self.perf.tile_turn = self.perf.frame_idx;
        state.next += 1;
        self.perf.tile_pending[vidx] = state.next < n;
        rect(idx)
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
        let budget_res_scale = if want > budget {
            (budget as f64 / want as f64).sqrt()
        } else {
            1.0
        };
        // ADAPTIVE MOTION RESOLUTION (AIMD). `budget_res_scale` sizes the moving frame from
        // `px·gpu_iter` — the *no-BLA-skip* cost. Where the BLA skips (the common deep case) the real
        // GPU cost is a tiny fraction of that, so the budget over-shrinks the moving frame (≈0.47 →
        // 668 px at 1e263×) and the reprojected/held texture goes blocky — the "distorted overlapping
        // tiles" seen when a zoom crosses from detail into the exterior. Instead drive the deep moving
        // res_scale by the MEASURED frame interval: additive-increase while frames stay near vsync (BLA
        // is skipping → headroom to sharpen), multiplicative-decrease when they run long (no skip →
        // back off). Adapts per view, needs no GPU-timestamp readback, and cannot hang — a slow frame
        // immediately shrinks the next, and the TDR cap below is still the hard watchdog floor. Only
        // deep perturbation motion is affected; shallow/settled keep the budget scale. The AIMD step
        // runs once per frame (view 0); both views read the shared scale.
        let res_scale = if interacting && is_pert {
            if view_id == 0 {
                let fm = self.perf.frame_ms;
                if fm > 26.0 {
                    // Floor is user-configurable (`min_motion_res`): raising it caps how far the
                    // moving/frozen frame may shrink — so a deep continuous zoom stays sharper
                    // (smaller upsampled blocks) at the cost of frame rate. Default 0.30.
                    self.perf.motion_res =
                        (self.perf.motion_res * 0.82).max(self.render_cfg.min_motion_res as f64);
                } else if fm > 0.0 && fm < 19.0 {
                    self.perf.motion_res = (self.perf.motion_res + 0.03).min(1.0);
                }
                // 19..=26 ms (≈38–53 fps): deadband — hold, so it settles instead of hunting.
            }
            self.perf.motion_res
        } else {
            budget_res_scale
        };
        // REUSE-FIRST ZOOM hold decision (used by BOTH the native-res gate here and the freeze
        // trigger below — they must agree). While interacting, hold + reproject the last good frame
        // (scaled to follow the zoom) instead of re-iterating. Once the held frame has been magnified
        // past REFRESH_OCTAVES, take ONE real (res-scaled) frame to refresh detail at the new depth —
        // otherwise a continuous zoom keeps upsampling the frozen texture into ever-larger blocks
        // until it settles. `frozen_l2 − log2mag` = octaves zoomed since the held frame was rendered;
        // a real frame resets it (updates `frozen_l2` below).
        //
        // This now applies to floatexp (mode 2, >~1e28×) as well as df32 (mode 0). It used to hold
        // floatexp THROUGHOUT — a fast dive past ~1e28× went increasingly blocky until you stopped to
        // let it settle — because a floatexp refresh could trigger a multi-second bignum reference
        // rebuild. It no longer can: the reference/BLA build is off-thread (v0.1.57 orbit REUSE cut it
        // ~20×), and the refresh frame itself is only a cheap res-scaled + TDR-bounded GPU iterate. If
        // the reference is still too short for the new depth, the `depth_lag` gate below holds/
        // reprojects instead — so a refresh never renders on a too-short reference (the old ~5 s-spin
        // hazard). Net: floatexp streams real detail every REFRESH_OCTAVES of a continuous dive.
        const REFRESH_OCTAVES: f64 = 0.5;
        // Time floor on the hold: the octave gate alone starves a SLOW deep dive of real frames —
        // an ease-out tour decelerating through ~2 oct/s drops below ~4 real updates/s (0.5 oct
        // apart in ZOOM is seconds apart in TIME), which reads as visible stepping ("jerkiness
        // from ~e590", onset tracking the RATE, not the depth). Refresh whenever the held frame is
        // older than this, even if it hasn't drifted an octave yet; a fast dive still refreshes on
        // the octave gate first. Real refresh frames are res-scaled + TDR-bounded, so ~7/s is
        // affordable at any depth the live view reaches.
        const REFRESH_MAX_SECS: f64 = 0.15;
        let vc = &self.ref_cache[view_id as usize];
        let frozen_drift = (vc.frozen_l2 - log2mag).abs();
        let frozen_fresh = vc
            .frozen_at
            .is_none_or(|t| t.elapsed().as_secs_f64() < REFRESH_MAX_SECS);
        let reuse_hold = is_pert
            && interacting
            && !self.autopilot.stepping
            && frozen_drift < REFRESH_OCTAVES
            && frozen_fresh;
        // A reprojection/freeze frame runs NO iterate (it re-samples the frozen texture), so the
        // motion res_scale saves nothing on it — and worse, it shrinks the frame's base below the
        // frozen texture's settle-time resolution, so the color-pass aspect-fit `fit = out_res /
        // frozen_screen_dim` goes < 1 and MAGNIFIES the held frame (a spurious zoom-in) while also
        // amplifying the pan translation by 1/fit (the "drag is exaggerated / double acceleration"
        // at deep zoom). Keep native resolution on a held frame so fit ≈ 1; a mode-0 REFRESH frame
        // (reuse_hold false) re-iterates and is res-scaled like the old behaviour, so it stays cheap.
        let will_reproject = reproject.is_some()
            || reuse_hold
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
        // multiple seconds without BLA and freezes the app (Windows resets the GPU after ~2s).
        //
        // This budget used to be the constant 3e11 "steps ≈ 0.85 s". That was calibrated on a view
        // where BLA skipped ~200×, so the NOMINAL count `spx·ss²·gpu_iter` overstated real work by the
        // same factor. On a deep INTERIOR minibrot BLA cannot skip — every pixel runs the full
        // iteration count — so nominal == actual and the constant was ~15× too generous: a 1e106×
        // minibrot submitted a ~5.5e10-step frame (≈2.2 s at the measured ~2.5e7 steps/ms) as ONE
        // dispatch, Windows reset the GPU, and the app died on LOAD before any measurement existed.
        //
        // So size the frame from MEASUREMENT rather than from an assumed skip factor. `Perf::fe_budget`
        // is a closed loop on wall-clock cost: each resolved probe retargets it to whatever step count
        // would have taken `TDR_BUDGET_MS`. Nothing here is calibrated to one GPU — before any
        // measurement exists the frame uses TDR_BOOTSTRAP_STEPS, tiny on any hardware, and the loop
        // climbs from there. A slower GPU just measures a longer frame and settles at a smaller budget
        // for the same wall-clock target; a faster one climbs to the ceiling and renders native res.
        //
        // The budget bounds the frame's FULL cost `spx·ss²·gpu_iter`, not just the ss=1 part: the
        // resolution shrink below fits `spx·gpu_iter` under it and `max_ss_tdr` then keeps ss² inside
        // the same envelope. Nothing may override that cap — a previous revision let a measured-AA
        // extension push ss past it, which quadrupled the frame to ~3.2 s and, while that stopped short
        // of the watchdog, it blocked the UI thread and hung the window ("Not Responding").
        let vidx = view_id as usize;
        // One-shot offscreen renders (`--render`, `--render-tour`, exports) never resolve a probe —
        // they draw a single frame — so an adaptive budget would strand them on the pessimistic seed
        // and silently shrink their internal resolution. They are not the watchdog case that motivated
        // this (they have always run at the ceiling and never tripped it), so leave them exactly as
        // they were: deep exports and the validation corpus must stay bit-for-bit reproducible.
        let offscreen = self.auto_render || self.playback.is_some();
        // A reproject frame re-samples the frozen texture, so it must land on the SAME resolution as
        // the frame that produced it; recomputing from a moving budget would drift `fit` off 1.
        let tdr_steps = if offscreen {
            TDR_STEPS_CEIL
        } else if will_reproject && self.perf.frozen_budget[vidx] != 0 {
            self.perf.frozen_budget[vidx]
        } else {
            // Zero until the first probe resolves; the loop in `update` maintains it thereafter.
            match self.perf.fe_budget[vidx] {
                0 => TDR_BOOTSTRAP_STEPS,
                b => b.clamp(TDR_BOOTSTRAP_STEPS, TDR_STEPS_CEIL),
            }
        };
        if !offscreen && !will_reproject {
            self.perf.frozen_budget[vidx] = tdr_steps;
        }
        // TILED SETTLE. A single dispatch caps a settled deep frame at whatever resolution fits the
        // watchdog budget — at a worst-case interior view that is a fraction of the panel, displayed
        // upscaled and permanently blocky. So once the view is SETTLED, spend up to TDR_MAX_TILES
        // budget-sized dispatches instead: keep (near-)native resolution and render it as a grid of
        // scissored tiles, one per frame, composing into exactly the frame one big dispatch would
        // have produced. Motion is untouched — it keeps the cheap shrunk single-dispatch frames.
        //
        // Any motion invalidates the grid via a view GENERATION (the f64 center cannot key a view at
        // depth — see `TileGrid::key`), and a grid only STARTS after one coarse full frame has
        // rendered under the same key (the ARMED step): that frame is what guarantees the texture the
        // tiles compose over shows this view's content everywhere, and it is also what the seeded
        // resize upscales. Requires a CONVERGED budget (`fe_budget_ok`): grids sized off a
        // still-climbing budget restart at every measurement, and bootstrap-sized tiles would mean
        // hundreds of dispatches.
        if interacting || will_reproject || self.autopilot.active {
            self.perf.view_gen[vidx] = self.perf.frame_idx;
        }
        if interacting {
            self.perf.interact_frame[vidx] = self.perf.frame_idx;
        }
        let can_tile = is_fe
            && self.allow_tiled_settle
            && !offscreen
            && !interacting
            && !will_reproject
            && self.perf.fe_budget[vidx] != 0;
        // Everything the ITERATE's output depends on that isn't covered by orbit_id / gen /
        // resolution, folded to one hash. The GPU re-renders whenever ITS IterKey changes
        // (color method, stripe frequency, trap type, SA/BLA toggles, Julia c, …) — but under a
        // pinned grid that re-render is SCISSORED to the current tile, so any such change the
        // app-side key misses would update one tile rect and leave the rest of the texture holding
        // data computed for the OLD settings, cached indefinitely (e.g. switching Smooth→Stripe
        // would show stripe coloring reading stale zero aux everywhere but one corner). Any field
        // added to the GPU IterKey that changes the iterate's OUTPUT must be reflected here.
        let settings_hash = {
            use std::hash::{Hash, Hasher};
            let mut h = std::hash::DefaultHasher::new();
            self.coloring.color_method.to_u32().hash(&mut h);
            self.coloring.stripe_freq.to_bits().hash(&mut h);
            self.coloring.trap_type.to_u32().hash(&mut h);
            self.render_cfg.series_approx.hash(&mut h);
            self.render_cfg.use_bla.hash(&mut h);
            eff_iter.hash(&mut h);
            if julia {
                self.julia_c.0.to_bits().hash(&mut h);
                self.julia_c.1.to_bits().hash(&mut h);
            }
            h.finish()
        };
        // `resolution` here is still the raw panel size: the motion res_scale doesn't apply to
        // settled frames, and the budget shrink / geometry pin happen below.
        let view_key = (
            self.ref_cache[vidx].orbit_id,
            gpu_iter,
            self.perf.view_gen[vidx],
            resolution,
            settings_hash,
        );
        let tiling = can_tile
            && match &self.perf.tile_state[vidx] {
                Some(g) if g.key == view_key => true,
                _ if self.perf.fe_budget_ok[vidx] => {
                    // ARM: this frame renders the coarse single-dispatch full frame; the grid may
                    // start next frame, seeded from it. Arming (but ONLY arming) waits for a
                    // CONVERGED budget: a grid sized off a still-climbing budget restarts at every
                    // measurement. Once armed/running, an `ok` flap must NOT tear the grid down —
                    // that threw away completed near-native frames whenever a stray measurement
                    // nudged the budget.
                    self.perf.tile_state[vidx] =
                        Some(TileGrid { key: view_key, geo: None, next: 0 });
                    false
                }
                _ => false,
            };
        if !can_tile && !will_reproject {
            // Leaving settle: drop the grid so the next settle re-arms from a fresh coarse frame.
            // (Held/reproject frames keep it — they don't change what the texture shows.)
            self.perf.tile_state[vidx] = None;
            self.perf.tile_pending[vidx] = false;
        }
        // A tiled frame may spend TDR_MAX_TILES dispatch budgets in total; everything else gets one.
        let tdr_allowed = if tiling {
            tdr_steps.saturating_mul(TDR_MAX_TILES)
        } else {
            tdr_steps
        };
        let spx = (resolution[0] as u64) * (resolution[1] as u64);
        // First line of defence is lowering ss (`max_ss_tdr` below), but that floors at 1. If even ss=1
        // would blow the budget — `spx·gpu_iter` alone > the budget, i.e. a large floatexp panel at
        // a high fixed iteration count — the ss cap can't help, so shrink the render RESOLUTION too.
        // Applied to reproject frames AS WELL: the frozen texture they re-sample was rendered by a
        // settled frame that took this same shrink, so a reproject frame must take it too for the color
        // pass aspect-fit `fit = out_res / frozen_screen_dim` to stay 1. This used to hold because the
        // shrink was deterministic (same panel, same iter count, constant budget); now that the budget
        // adapts, the reproject frame reuses the producing frame's stored `frozen_budget` to land on the
        // identical resolution. Skipping the shrink here (as v0.1.46 briefly did) leaves the reproject
        // frame at native res over a shrunk frozen texture → fit > 1 → the held image displays SHRUNK
        // (average-color borders) and the pan translation runs slow by the same factor — the inverse of
        // the v0.1.44 magnify. (The MOTION res_scale above stays gated on `!will_reproject`: unlike this
        // shrink it differs between moving and settled frames, which is what caused the v0.1.44 bug.)
        let (resolution, spx) = {
            let iter_cost = spx.saturating_mul(gpu_iter.max(1) as u64);
            if is_fe && iter_cost > tdr_allowed {
                let f = (tdr_allowed as f64 / iter_cost as f64).sqrt();
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
            ((tdr_allowed / spx.saturating_mul(gpu_iter.max(1) as u64).max(1)) as f64)
                .sqrt()
                .max(1.0) as u32
        } else {
            u32::MAX
        };
        // `max_ss_tdr` used to be extendable past its own value by a measured-AA allowance, on the
        // grounds that it assumed BLA skips NOTHING and so over-throttled views where BLA is effective.
        // That reasoning is now folded into `tdr_steps` itself, which is DERIVED from measurement — a
        // BLA-effective view simply measures a high rate and gets a big budget. Letting AA override the
        // cap on top of that double-counts, and it is exactly what hung the app: ss=2 quadrupled a
        // budget-sized frame to ~3.2 s, which stopped short of the watchdog but blocked the UI thread.
        // The cap is therefore hard — the frame's full `spx·ss²·gpu_iter` stays inside the budget.
        let vs = view_id as usize;
        if interacting {
            self.perf.aa_measured[vs] = None;
            self.perf.aa_probe[vs] = None;
        }
        // `aa_target` is 1 while moving and ramps up over settled frames (progressive settle); clamp
        // to what the per-frame budget affords (`max_ss`) and what the watchdog / UI thread allow.
        let ss = aa_target.min(max_ss).min(max_ss_tdr).max(1);
        // PIN the frame geometry to an existing grid's. Resolution and ss above are derived from the
        // LIVE budget, which jitters a few percent per measurement — recomputing them each frame
        // restarts the grid every few tiles (observed: every ~4) and it never finishes; worse, after
        // completion the same jitter would resize the texture and throw the sharp frame away. So
        // while the grid's key holds, its stored geometry wins. The one deliberate exception: a
        // COMPLETED grid steps aside when the freshly computed ss came out strictly higher — that is
        // the AA ramp's next stage becoming genuinely affordable (a new grid, seeded from the
        // completed one). A merely jittered resolution can't unpin anything.
        let (resolution, ss) = if tiling {
            match &self.perf.tile_state[vidx] {
                Some(st) if st.geo.is_some() => {
                    let (gres, gss, side) = st.geo.unwrap();
                    let cols = gres[0].div_ceil(side).max(1);
                    let rows = gres[1].div_ceil(side).max(1);
                    let upgrade = st.next >= cols * rows && ss > gss;
                    if upgrade { (resolution, ss) } else { (gres, gss) }
                }
                _ => (resolution, ss),
            }
        } else {
            (resolution, ss)
        };
        let spx = (resolution[0] as u64) * (resolution[1] as u64);
        // Arm a probe only on a frame that actually RE-ITERATES (a cached frame's interval is
        // ~vsync and would wildly over-authorize): the iterate re-runs when its key inputs change
        // (ss stage, resolution, reference), and on the first settled frame after an interaction
        // (the view moved, so that frame re-iterates even with an unchanged key — this bootstraps
        // the ladder on views whose static cap is 1, which otherwise never see an ss change).
        let key = (ss, resolution, self.ref_cache[vs].orbit_id);
        let key_changed = key != self.perf.aa_last_key[vs];
        self.perf.aa_last_key[vs] = key;
        // Emit this frame's settle tile, if the (now-final) resolution and ss still exceed one
        // dispatch. `tile` travels into `MandelbrotParams` below; a real (non-hold) rect also
        // reprices the timing sink, since the timestamped dispatch is the tile, not a full frame.
        let mut tile: Option<[u32; 4]> = None;
        if tiling {
            let total = spx
                .saturating_mul((ss as u64).saturating_mul(ss as u64))
                .saturating_mul(gpu_iter.max(1) as u64);
            if total > tdr_steps {
                let rect = self.next_settle_tile(vidx, resolution, ss, gpu_iter, tdr_steps);
                if rect[2] > 0 && rect[3] > 0 {
                    self.perf.fe_steps_last[vs] = (rect[2] as u64)
                        .saturating_mul(rect[3] as u64)
                        .saturating_mul((ss as u64).saturating_mul(ss as u64))
                        .saturating_mul(gpu_iter.max(1) as u64);
                }
                tile = Some(rect);
                // The texture a later reproject frame re-samples is (near-)native here, NOT shrunk
                // to one dispatch — a reproject frame must reproduce that, or fit drifts off 1.
                self.perf.frozen_budget[vidx] = u64::MAX;
            } else {
                // The budget grew enough that the whole frame fits one dispatch after all.
                self.perf.tile_state[vidx] = None;
                self.perf.tile_pending[vidx] = false;
            }
        }
        if crate::diag::trace_on("tile") && is_fe {
            let steps = spx
                .saturating_mul((ss as u64).saturating_mul(ss as u64))
                .saturating_mul(gpu_iter.max(1) as u64);
            crate::diag::trace(
                "tile",
                format!(
                    "f={} view={vs} res={}x{} ss={ss} gpu_iter={gpu_iter} steps={:.3e} \
                     budget={:.3e} reproj={will_reproject} iterates={key_changed} tile={:?} pending={} \
                     interacting={interacting} can_tile={can_tile} tiling={tiling} key={view_key:?} \
                     allow={} offs={offscreen}",
                    self.perf.frame_idx,
                    resolution[0],
                    resolution[1],
                    steps as f64,
                    tdr_steps as f64,
                    tile,
                    self.perf.tile_pending[vs],
                    self.allow_tiled_settle,
                ),
            );
        }
        let bootstrap =
            self.perf.aa_measured[vs].is_none() && self.perf.aa_probe[vs].is_none();
        if is_fe
            && !interacting
            && !will_reproject
            && self.playback.is_none() // tour frames move every keyframe — not settled-cost data
            && (key_changed || bootstrap)
        {
            // Carry this frame's nominal step count: both arming conditions re-iterate (a changed key
            // re-runs the iterate; a bootstrap frame is the first settled one after the view moved), so
            // the interval that resolves the probe prices real GPU work and yields `fe_rate_spm`.
            let steps = spx
                .saturating_mul((ss as u64).saturating_mul(ss as u64))
                .saturating_mul(gpu_iter.max(1) as u64);
            self.perf.aa_probe[vs] = Some((ss, self.perf.frame_idx, 0.0, steps));
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
            let bla_active = mode.is_floatexp()
                && !self.ref_cache[vi].bla.is_empty()
                && self.ref_cache[vi].bla_dc_max_log2.is_finite();
            let depth_lag = if bla_active {
                self.ref_cache[vi].bla_dc_max_log2
                    - Self::bla_dc_max(span_mantissa, delta_exp).log2()
            } else {
                0.0
            };
            // Published for the playback pacer: script playback dilates the tour clock when this
            // lag says the async reference pipeline is falling behind the dive (see
            // `advance_playback`), so the screen keeps a fresh reference instead of a stale blur.
            self.ref_cache[vi].last_depth_lag = depth_lag;
            // While moving, tolerate the reference lagging in precision (it has 64 guard bits, good
            // for ~40 more octaves) so we don't rebuild the slow bignum orbit every octave — that
            // was the "zoom, pause, zoom" stepping on a deep dive. On settle we rebuild at the first
            // bit of lag so the still frame is at full precision. The orbit's iteration headroom
            // (`ref_build_iter`) covers the matching depth range, so iters rarely force a rebuild.
            let prec_headroom = if interacting { 32 } else { 0 };
            // `gpu_iter > orbit_iter` grows a TRUNCATED reference so it serves deeper pixels. A
            // COMPLETE (escaped) reference is already final — pixels past its escape rebase — so a
            // rebuild would just re-pick the same-length escaped orbit; gating on `partial` stops it
            // firing EVERY frame (which, at a location whose reference always escapes, re-picked a
            // fresh reference each frame → the deep-zoom "jumping").
            let needs_quality = precision > self.ref_cache[vi].orbit_prec + prec_headroom
                || (self.ref_cache[vi].partial && gpu_iter > self.ref_cache[vi].orbit_iter);
            // Refresh whenever the reference has left the view or the depth/iters outgrew it. The
            // old motion throttle is gone: the recompute is now off-thread and gated to one job per
            // view (`recompute_rx`), so spawns are naturally paced by the compute itself — no need
            // to space them artificially (which only made deep references refresh more slowly).
            // In mode 2, refresh the reference/BLA as soon as it lags (off-thread, cheap) so fresh
            // references keep coming; freeze (below) just above that while each refresh lands. A
            // fresh install sits at depth_lag ≈ 1 (the BLA is built with 1 octave of dc_max headroom),
            // valid across a narrow window around it:
            //   depth_lag > 1.1  → zoomed IN past the headroom (the mode-2 shader starts to spin).
            //   depth_lag < 0.85 → zoomed OUT so the view span now EXCEEDS the BLA's dc_max at the
            //     edges: the perturbation applies BLA nodes outside their validity and renders block/
            //     "tile" artifacts. This was the e260 zoom-out tiling — a settled view sat at
            //     depth_lag 0.69 with an escaped reference and never rebuilt, because only the zoom-IN
            //     edge had a trigger. Rebuild whenever the BLA leaves the window in EITHER direction;
            //     the off-thread orbit-reuse rebuild is cheap and resets depth_lag to ≈ 1. (Lower
            //     bound 0.85: below the ≈1.0 fresh-install point with margin so it can't thrash, well
            //     above the measured 0.69 tiling onset so it rebuilds before artifacts show. Zoom-IN
            //     never drops below 1.0, so this leaves the dive path untouched.)
            let bla_out_of_range = bla_active && !(0.85..=1.1).contains(&depth_lag);
            let recompute = out_of_view || needs_quality || bla_out_of_range;
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
                // Deep-dive reuse: on a deeper-zoom refresh at a still-in-view reference (NOT an
                // out-of-view re-anchor), hand the worker the cached orbit so it can EXTEND it rather
                // than recompute every bignum step. The worker re-validates drift/precision and falls
                // back to a fresh build otherwise. A truncated orbit is EXTENDED; a complete (escaped)
                // orbit is reused AS-IS (keeps the same reference, avoiding a re-pick jump) — both
                // qualify. Only a cold reference (no orbit/tail) has none.
                let reuse = if out_of_view {
                    None
                } else {
                    let vc = &self.ref_cache[vi];
                    match (vc.ref_pt.clone(), vc.orbit_tail.clone()) {
                        (Some(point), Some(tail)) if !vc.orbit.is_empty() => {
                            Some(ReuseRef { point, prefix: vc.orbit.clone(), tail, prec: vc.orbit_prec })
                        }
                        _ => None,
                    }
                };
                let inputs = RecomputeInputs {
                    center_bf: center_bf.clone(),
                    span,
                    span_mantissa,
                    delta_exp,
                    gpu_iter: ref_build_iter,
                    // LIVE freeze safety: keep the reference orbit + BLA small (the ~4M-node BLA of
                    // a full-appetite reference at a deep tip overloads the GPU present — the old
                    // freeze). Pixels iterate to the full `gpu_iter` by rebasing past this short
                    // reference (an export at this depth resolves the same detail from a ~100k
                    // reference), so the preview loses no border detail — only the reference is bounded.
                    orbit_len_cap: crate::LIVE_REF_CAP,
                    precision,
                    julia,
                    formula: self.fractal.formula_id(),
                    julia_c: self.julia_c,
                    do_sa,
                    bla_dc_max: bla_will_build
                        .then(|| Self::bla_dc_max(span_mantissa, delta_exp).mul_pow2(1.0)),
                    stripe_freq: self.coloring.stripe_freq as f64,
                    trap_type: self.coloring.trap_type as u32,
                    reuse,
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
            // depth-stale reference. Three triggers combine into `too_stale`:
            //  • `drift > 1.5` (above): the reference drifted out of view → δc too large → glitchy.
            //  • `reuse_hold` (decided above): the REUSE-FIRST hold — scale/pan the last good frame to
            //    follow the zoom rather than re-iterate, until it has drifted REFRESH_OCTAVES, then take
            //    one real frame. Applies to both df32 and (since v0.1.58) floatexp. `!stepping` there
            //    lets the autopilot's stepped dive render real frames between jumps.
            //  • `depth_lag > DEEP_LAG_HOLD`: octaves the view has zoomed past the cached BLA's build
            //    depth (its dc_max shrinks with the span). The reference stays USABLE far past a fresh
            //    install (~1 octave of headroom): the mode-2 shader only starts spinning near ~3 octaves
            //    of lag, and even that is now TDR-bounded to ~0.85 s (not the old ~5 s "Not Responding"
            //    hang). This used to hold at 1.2 — barely past the 1.1 the recompute SPAWNS at — so on a
            //    fast dive past ~1e58× the ~15–30 ms rebuild couldn't land in that 0.1-octave window and
            //    the view held (blocky) until you paused for it to catch up. Holding at 1.8 keeps the
            //    frame rendering real (if progressively-less-skipped) detail while the off-thread rebuild
            //    catches up, so a continuous dive stays sharp far deeper; beyond it, a depth-matched
            //    reference lands and the view snaps to full sharpness. (`is_direct`, mode 1 <1e4×,
            //    re-renders every frame — cheap, sharp, no frozen texture to reproject.)
            const DEEP_LAG_HOLD: f64 = 1.8;
            too_stale = too_stale || reuse_hold || depth_lag > DEEP_LAG_HOLD;
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
                        // uv_scale = span_now/span_frozen = 2^(l2_frozen − l2_now); ≤ 1 as we dive in.
                        // The lower bound must stay tiny: `uv_off` below is `px · scale`, so if `scale`
                        // is clamped ABOVE its true value (while the view keeps diving) the held frame
                        // translates by too much and visibly SLIDES/jitters. The old 1e-4 floor (≈13
                        // octaves of frozen-drift) was hit whenever the off-thread rebuild fell behind on
                        // a fast dive past ~1e100×, producing exactly that. f32 represents down to ~2^-126,
                        // so floor at 2^-40 (~40 octaves — unreachable in a real dive) → the reprojection
                        // stays correctly positioned; a very stale frozen frame just magnifies (blocky)
                        // in place instead of sliding.
                        // scale = 2^(l2_frozen − l2_now): ≤ 1 zooming IN (frozen frame magnifies),
                        // > 1 zooming OUT (frozen frame shrinks toward centre, the average fills the
                        // revealed border). The old upper clamp of 1.0 pinned zoom-out reprojection at
                        // 1:1 — the held frame stayed full-size instead of shrinking, so a zoom-out
                        // held its stale (too-magnified) detail until the refresh snapped it smaller.
                        // Allow > 1; the shader maps out-of-[0,1] samples to the frame average, so a
                        // very stale zoom-out just shows a shrinking patch on the average field. Bounds
                        // stay finite for f32 (2^±40, ~40 octaves — unreachable in a real drift).
                        let scale = ((self.ref_cache[vi].frozen_l2 - log2mag) as f32)
                            .exp2()
                            .clamp(9.094_947e-13, 1.099_512e12); // 2^-40 .. 2^40
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
            // view it renders — the next freeze reprojects the resulting texture relative to it —
            // and WHEN, so the reuse-hold's time floor can age it (see `REFRESH_MAX_SECS`).
            if reproject.is_none() {
                self.ref_cache[vi].frozen_center = Some(center_bf.clone());
                self.ref_cache[vi].frozen_l2 = log2mag;
                self.ref_cache[vi].frozen_at = Some(Instant::now());
            }
            // δ = center − reference, carried as a mantissa scaled by 2^-delta_exp (O(1) in df32 at
            // any depth; the GPU re-applies the exponent). Skipped during a cold-start hold — no
            // reference yet, so this frame is a reprojection freeze and ref_offset stays ZERO/unused.
            if let Some(rp) = self.ref_cache[vi].ref_pt.as_ref() {
                let dx = fractadyne_core::ref_offset_mantissa(&center_bf[0], &rp[0], delta_exp, precision);
                let dy = fractadyne_core::ref_offset_mantissa(&center_bf[1], &rp[1], delta_exp, precision);
                ref_offset = RefOffset::from_df32(dx, dy);
            }
            // SHORT ESCAPED reference (deep EXTERIOR): mirror `finish_reference` — the BLA turned SA
            // off ("BLA subsumes SA"), which exposed an early-iteration perturbation glitch as tiles.
            // The BLA is kept (speed), but SA must be read back so it seeds δz past the glitch. Under
            // BLA eligibility `do_sa` is false, so without this the (off-thread-computed) SA would sit
            // unused. Same predicate as `finish_reference` keeps the two paths in lock-step.
            let short_escaper = self.bla_eligible(mode, julia)
                && !self.ref_cache[vi].partial
                && (self.ref_cache[vi].orbit_len as u64).saturating_mul(2) < ref_build_iter.max(1) as u64;
            // Series approximation travels with the reference (computed off-thread); read it back.
            if do_sa || short_escaper {
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
                // Stripe (method 1) and orbit-trap (method 3) bake a live coloring param into their
                // aggregate lane — the stripe frequency, and the trap type — so a change to that
                // slider stales the tree (unlike TIA, whose aggregate is reference-intrinsic). Rebuild
                // when the active method's param drifts; cheap (~20 ms, off the deep-zoom hot path).
                let aux_stale = (self.coloring.color_method.to_u32() == 1
                    && (self.coloring.stripe_freq as f64 - self.ref_cache[vi].bla_stripe_freq).abs() > 1.0e-9)
                    || (self.coloring.color_method.to_u32() == 3
                        && self.coloring.trap_type as u32 != self.ref_cache[vi].bla_trap_type);
                let vc = &self.ref_cache[vi];
                // Rebuild if the orbit changed, the current view needs a bigger dc_max than the cached
                // tree was built for (tiny epsilon guards float noise), or the active aux param moved.
                if vc.bla_id != oid || need_log2 > vc.bla_dc_max_log2 + 1.0e-6 || aux_stale {
                    let orbit = self.ref_cache[vi].orbit.clone();
                    // Build with 2× headroom (dc_max·2 ⇒ +1 in log2) so continuous zoom-out doesn't
                    // rebuild every frame; still valid (a larger dc_max only shrinks skip radii).
                    let build_dc = dc_max.mul_pow2(1.0);
                    let built = self.build_bla(&orbit, build_dc);
                    let vc = &mut self.ref_cache[vi];
                    vc.bla = built.unwrap_or_else(|| std::sync::Arc::new(Vec::new()));
                    vc.bla_id = oid;
                    vc.bla_dc_max_log2 = build_dc.log2();
                    vc.bla_stripe_freq = self.coloring.stripe_freq as f64;
                    vc.bla_trap_type = self.coloring.trap_type as u32;
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
            // interior) until the full reference lands and refines it. (Do NOT cap escaped-short
            // references: those pixels ESCAPE, they don't rebase-glitch — the deep-exterior "tiles"
            // were the BLA/SA issue, fixed in `finish_reference`. Capping an escaped-short reference
            // instead collapses every late-escaping BOUNDARY pixel to interior → a hard black border
            // with no filament detail around a deep minibrot.)
            gpu_iter.min(self.ref_cache[vi].orbit_len.saturating_sub(1))
        } else {
            gpu_iter
        };

        // Record the nominal cost of a frame that will actually re-iterate, so `update` can price the
        // measurement that comes back for it. Only such frames run the pass, so only they arm a
        // timestamp — but the SINK must be attached on EVERY frame: the readback lands a couple of
        // frames later, by which time the view is usually serving cached/reproject frames, and a
        // sink-less pump would silently discard the reading.
        if is_fe && key_changed && !will_reproject {
            self.perf.fe_steps_last[vs] = spx
                .saturating_mul((ss as u64).saturating_mul(ss as u64))
                .saturating_mul(gpu_iter.max(1) as u64);
            self.perf.fe_iter_frame[vs] = self.perf.frame_idx;
        }
        let iterate_ms = is_fe.then(|| self.perf.iterate_ms[vs].clone());

        MandelbrotParams {
            iterate_ms,
            tile,
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

#[cfg(test)]
mod reuse_tests {
    //! Verify the deep-dive reference-reuse plumbing end-to-end at the orbit level: `recompute_worker`
    //! extends a cached orbit instead of rebuilding, and `try_reuse_reference`'s gates reject any
    //! reference that would be invalid to reuse. (The extended orbit's byte-identity to a fresh build,
    //! and render invariance to the chosen valid reference, are proven separately in core + selftest.)
    use super::*;
    use fractadyne_core::{parse_bf, Viewport};

    // A Mandelbrot recompute for a view at (cx, cy, log2mag) — no BLA/SA, so the orbit is isolated.
    fn inputs_for(cx: &str, cy: &str, log2mag: f64, gpu_iter: u32, reuse: Option<ReuseRef>) -> RecomputeInputs {
        let mut vp = Viewport::new(256.0, 256.0);
        vp.set_center_log2mag(parse_bf(cx).unwrap(), parse_bf(cy).unwrap(), log2mag);
        let scale = vp.gpu_scale();
        RecomputeInputs {
            center_bf: [vp.center_x.clone(), vp.center_y.clone()],
            span: vp.complex_span_fe(),
            span_mantissa: scale.span_mantissa,
            delta_exp: scale.delta_exp,
            gpu_iter,
            orbit_len_cap: u32::MAX,
            precision: vp.precision,
            julia: false,
            formula: 0,
            julia_c: (0.0, 0.0),
            do_sa: false,
            bla_dc_max: None,
            stripe_freq: 1.0,
            trap_type: 0,
            reuse,
        }
    }

    // Seahorse boundary point: survives thousands of iters, so a short build is truncated/extendable.
    const SX: &str = "-0.7436438870371587047521915061147707";
    const SY: &str = "0.131825904205311970493132056385139";
    const L2: f64 = 26.6; // ~1e8× (mode-0 df32 perturbation)

    #[test]
    fn reuse_extends_cached_orbit_in_place() {
        let a = recompute_worker(inputs_for(SX, SY, L2, 3000, None));
        assert!(a.partial, "short seahorse orbit should be truncated (extendable)");
        let tail = a.orbit_tail.clone().expect("a truncated orbit carries a tail");
        let reuse = ReuseRef { point: a.rp.clone(), prefix: a.orbit.clone(), tail, prec: a.prec };
        // A deeper-iter rebuild at the same view must EXTEND a's orbit, not rebuild it.
        let b = recompute_worker(inputs_for(SX, SY, L2, 6000, Some(reuse)));
        assert!(b.orbit_len > a.orbit_len, "reuse should have extended the orbit");
        assert_eq!(b.rp, a.rp, "reuse must keep the cached reference point");
        assert_eq!(b.prec, a.prec, "extend must stay at the cached (headroom) precision");
        // Byte-identical prefix ⇒ it truly extended (a fresh build at this depth would differ).
        assert_eq!(&b.orbit[..a.orbit.len()], &a.orbit[..], "extended orbit must preserve the prefix");
    }

    #[test]
    fn reuse_gates_reject_invalid_references() {
        let a = recompute_worker(inputs_for(SX, SY, L2, 3000, None));
        let tail = a.orbit_tail.clone().expect("tail");
        let mk = |r: ReuseRef| inputs_for(SX, SY, L2, 6000, Some(r));

        // Valid reference → reuse fires.
        let ok = ReuseRef { point: a.rp.clone(), prefix: a.orbit.clone(), tail: tail.clone(), prec: a.prec };
        assert!(try_reuse_reference(&mk(ok)).is_some(), "a valid in-view reference must reuse");

        // Escaped (complete) orbit → reused AS-IS, not extended: there's nothing past the escape,
        // but keeping the same reference avoids a re-pick "jump" on rebuild (deep-dive reuse policy
        // since v0.1.64/65). So reuse fires (Some) and the orbit length is unchanged.
        let mut esc_tail = tail.clone();
        esc_tail.escaped = true;
        let esc = ReuseRef { point: a.rp.clone(), prefix: a.orbit.clone(), tail: esc_tail, prec: a.prec };
        let re = try_reuse_reference(&mk(esc)).expect("an escaped orbit is reused as-is");
        assert_eq!(re.orbit_len, a.orbit_len, "an escaped orbit is reused unchanged, not extended");

        // Cached precision below this depth's need → headroom exhausted.
        let lowp = ReuseRef { point: a.rp.clone(), prefix: a.orbit.clone(), tail: tail.clone(), prec: 8 };
        assert!(try_reuse_reference(&mk(lowp)).is_none(), "insufficient precision must not reuse");

        // Point far off-centre (origin is ~0.75 away, ≫ a 1e8× span) → drifted out of validity.
        let far = [parse_bf("0.0").unwrap(), parse_bf("0.0").unwrap()];
        let drift = ReuseRef { point: far, prefix: a.orbit.clone(), tail, prec: a.prec };
        assert!(try_reuse_reference(&mk(drift)).is_none(), "a drifted point must not reuse");
    }

    // The orbit-length cap must fit the orbit + BLA (~9× the orbit at 16 B/sample) inside the GPU
    // storage-binding limit, yet stay ABOVE every escaping corpus reference so those views build
    // unchanged. Loc 15's 918 516-sample reference is the deepest such orbit (right at the 128 MB
    // edge) and is the binding invariant: the cap must clear it, or the deep-dendrite corpus render
    // regresses. (The cap only ever truncates a NON-escaping deep-interior reference.)
    #[test]
    fn orbit_len_cap_fits_binding_and_clears_corpus() {
        const LIMIT_128MB: u32 = 134_217_728; // wgpu default max_storage_buffer_binding_size
        const LOC15_ORBIT: u64 = 918_516; // deepest escaping corpus reference (v0.2.18 dendrites)
        init_orbit_len_cap(LIMIT_128MB);
        let cap = orbit_len_cap() as u64;
        // Orbit + BLA (~9×) at 16 B/sample must fit the binding.
        assert!(cap * 9 * 16 <= LIMIT_128MB as u64, "cap {cap} + BLA overruns the 128 MB binding");
        // …and clear the deepest escaping corpus reference so it is never truncated.
        assert!(cap > LOC15_ORBIT, "cap {cap} must exceed loc 15's {LOC15_ORBIT}-sample orbit");
    }
}
