//! `--chunk-sweep` — the uncensored per-window cost profile of a chunked iterate.
//!
//! ## Why this exists
//!
//! The 2026-08-22 field device loss (`crash-1787401025-0`) was sized by `chunk_band_license`, not
//! by the frame-cost budget and not by resolution: the budget authorised 24,457 iterations and the
//! pass ran 1,024. The manifest's `chunk=[768,1792)` forces the ladder
//! `[0,256) → [256,768) → [768,1792)`, which forces `bands[0]` = 0 → 512 → 1024, and in
//! `chunk_band_update` the ONLY branch producing those values is the ×2 fast lane at
//! `price_ms <= target*0.5 = 200 ms`. So the controller recorded both pre-fatal passes at ≤200 ms.
//!
//! **Either those prices were lies or they were true, and the two answers need different fixes:**
//!
//! - **Fabricated** — the release gate is `last_dt_ms < CHUNK_DRAIN_DT_MS`, a wall interval rather
//!   than a completion signal, and the swapchain absorbed the submissions. Then an honest clock
//!   (retire on `Queue::on_submitted_work_done`) is the fix.
//! - **Honest** — `desired_maximum_frame_latency: Some(1)` back-pressured the acquire as designed,
//!   the prices were real, and cost genuinely CLIFFS by ≥8× between iteration 768 and 1792. Then
//!   the clock is a no-op and the sizer needs an axis it does not have.
//!
//! Nothing in the field evidence separates them, because **every cost signal the app owns is
//! unusable for this question**. A GPU timestamp cannot arm on a saturated queue (`arm_ts` requires
//! `TimingState::Idle`), and the wall fallback reads a frame interval, which wgpu-core clamps at
//! `FRAME_TIMEOUT_MS = 1000` in `present.rs` — so a frame that overran by 3 s and one that overran
//! by 1.1 s report the same number. Both fatal manifests pin their intervals at 1002/1005 ms across
//! a 484× spread in nominal work.
//!
//! This harness measures the one thing they cannot: **one submission, alone in the queue, timed
//! against its own `poll(Wait)`**. No swapchain, no present, no censoring, no deferral.
//!
//! ## What it reports
//!
//! - **The window axis.** For each window size, the wall cost of every position in `[0, iters)`.
//!   If cost is proportional to the window, sizing the window is a sufficient actuator. If a
//!   position costs many times its neighbours at the same size, it is not.
//! - **The area axis.** The same view at successively fewer pixels, at one fixed window. The fit
//!   `wall ~ area^a` is the go/no-go for any spatial sub-budget scheme: `a ≈ 0` means removing
//!   pixels buys nothing and the "spatial splits are the wrong axis" verdict extends to bounded
//!   windows too.
//! - **Pixel identity.** Every window size must resolve to the same image;
//!   `render_iter_chunked`'s contract is bit-identical output for any `chunk_iters`. A sweep is a
//!   free opportunity to assert it, so it does.
//!
//! ## Reading it honestly
//!
//! ⚠**The reference must be the LIVE one.** The request comes from `autopilot_probe_request`
//! (`RefSource::Live`), which borrows the resident orbit instead of building a fresh one at export
//! appetite — a longer reference rebases differently and would measure a different regime than the
//! one that died. The report prints `orbit_len` so a silent fallback to a fresh build is visible
//! rather than assumed.
//!
//! ⚠**The ladder aborts on the first window past `SAFETY_MS`.** The point is to characterise the
//! curve, not to reproduce the crash: escalation stops as soon as a window is slow enough that
//! doubling it again would approach the ~2 s watchdog. A sweep that stopped early says so.
//!
//! ⚠**An empty pass list is not a fast render.** `render_iter_chunked_timed` falls back to one
//! unbounded `render_iter` dispatch outside its supported scope (mode, formula > 3, attachment
//! bytes). That is reported as NOT MEASURED, never as a result.

use crate::FractadyneApp;

/// Window sizes walked on the size axis, smallest first. 256 is the live path's own floor
/// (`render.rs`'s `floor` when `fe_budget != 0`) and 1024 is what the fatal pass ran; the sizes
/// either side of them are what make the trend readable rather than a two-point line.
const WINDOWS: &[u32] = &[64, 128, 256, 512, 1024, 2048];

/// Resolution scales for the area axis. 1 → 1/8 linear spans 64× in AREA, which is the range the
/// CELLWALK go/no-go needs to fit an exponent over.
const AREA_SCALES: &[f64] = &[1.0, 0.5, 0.25, 0.125];

/// Heights walked on the height axis, as offsets from the settled render height. The filed
/// defect reported ~25x for an ODD height; +-4 around the base separates "parity" from a
/// specific alignment (4? 8?), which a two-point odd/even comparison cannot.
const HEIGHT_OFFSETS: &[i32] = &[-4, -3, -2, -1, 0, 1, 2, 3, 4];

/// Stop escalating the window once a single one costs this much. Deliberately below the ~2 s
/// Windows TDR delay and above the 900 ms lethal band, so the sweep can measure INSIDE the band
/// that kills without stepping past it — the next size up is ~2×, which is the margin this leaves.
const SAFETY_MS: f64 = 1200.0;

/// How many iterations the sweep walks. The fatal ladder covered `[0,1792)`, so the default
/// bracket contains it with room either side; every window size walks the SAME bracket, which is
/// what makes the per-position costs comparable across sizes.
const DEFAULT_ITERS: u32 = 2048;

/// Give up waiting for the view to settle after this many frames. The wait itself ends as soon as
/// the reference STOPS GROWING (see `STABLE_FRAMES`) — a fixed countdown was the first version and
/// it was wrong: the adaptive iteration boost ratchets ×1.6 per step, each raise rebuilds the
/// reference longer, and a 240-frame wait sampled that ladder mid-climb (orbit_len 83,357 against
/// the field session's settled 111,885). Measuring a reference that is still growing measures a
/// regime the view never rests in.
const SETTLE_CAP_FRAMES: u32 = 3600;

/// Consecutive frames the reference length must hold before the view counts as settled.
const STABLE_FRAMES: u32 = 90;

pub(crate) struct ChunkSweep {
    iters: u32,
    waited: u32,
    last_len: u32,
    stable: u32,
    done: bool,
}

impl ChunkSweep {
    /// Parse `--chunk-sweep [ITERS]`. A bare flag walks `DEFAULT_ITERS`.
    pub(crate) fn from_args(args: &[String]) -> Option<Self> {
        let i = args.iter().position(|a| a == "--chunk-sweep")?;
        let iters = args
            .get(i + 1)
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|n| *n > 0)
            .unwrap_or(DEFAULT_ITERS);
        Some(Self { iters, waited: 0, last_len: u32::MAX, stable: 0, done: false })
    }
}

/// One measured (window size, cursor position) cell.
struct Cell {
    window: u32,
    start: u32,
    wall_ms: f64,
    /// A tail window clipped by `max_iter` covers fewer iterations than it asked for. Comparing a
    /// 1-iteration remnant against a full window and calling the ratio a cost cliff is how the
    /// first run of this harness manufactured an "8.5x spread" out of arithmetic.
    full: bool,
}

impl FractadyneApp {
    /// Drive the sweep. Returns `true` once the app should close.
    pub(crate) fn chunk_sweep_step(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
    ) -> bool {
        let len = self.ref_cache[0].orbit_len;
        let partial = self.ref_cache[0].partial;
        let Some(sw) = self.chunk_sweep.as_mut() else { return false };
        if sw.done {
            return true;
        }
        // Settled = the reference has stopped growing. The boost ladder (×1.6 per raise, each one
        // forcing a longer rebuild) is what makes a fixed countdown unsafe, so wait on the thing
        // that actually stops changing rather than on a clock.
        sw.waited += 1;
        if len == sw.last_len && len > 0 && !partial {
            sw.stable += 1;
        } else {
            sw.last_len = len;
            sw.stable = 0;
        }
        if sw.stable < STABLE_FRAMES && sw.waited < SETTLE_CAP_FRAMES {
            return false;
        }
        let settled = sw.stable >= STABLE_FRAMES;
        let waited = sw.waited;
        let iters = sw.iters;
        sw.done = true;
        if !settled {
            println!(
                "⚠NOT SETTLED after {waited} frames — orbit_len is still moving (now {len},                  partial={partial}). Measuring anyway, but the reference is not the one a rested                  view uses, and a growing reference rebases differently."
            );
        }
        self.run_chunk_sweep(device, queue, iters);
        true
    }

    fn run_chunk_sweep(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        iters: u32,
    ) {
        println!(
            "Fractadyne chunk sweep — {}\n\
             Per-window wall cost of a chunked iterate, measured submission-alone against its own\n\
             poll(Wait). This is the only cost signal here that is neither right-censored at\n\
             wgpu-core's FRAME_TIMEOUT_MS=1000 nor deferred two frames.\n",
            crate::version_string()
        );

        let base = self.autopilot_probe_request(&self.viewport, self.julia_mode);
        let panel = [base.width, base.height];
        // THE BRACKET MUST CLEAR THE SERIES-APPROXIMATION SKIP, or it measures nothing.
        // `fs_iterate_chunk` seeds `iter = sa_skip` on the first pass (mandelbrot.wgsl ~1809), so
        // EVERY window that ends at or below `sa_skip` breaks on its first test and costs ~0 ms.
        // The first version of this harness walked [0,2048) against `sa_skip=53761` and reported
        // 16.6 ms for the whole sweep - six window sizes of pure noise, from which it cheerfully
        // derived three verdicts.
        //
        // That is not merely a harness bug, it is a MECHANISM: the live walk's band ledger prices
        // those same empty windows, and `chunk_band_update`'s x2 fast lane (`price_ms <= 200`)
        // doubles the license on each one. A ledger can therefore arrive at the first REAL window
        // holding a license earned entirely inside the skip - honest prices, measuring nothing.
        let skip = base.sa_skip;
        let iters = iters.saturating_add(skip);
        println!(
            "view      : centre {} , {}\n\
             zoom      : 2^{:.2}   ({}x{} px, mode {}, formula {}, ss forced to 1 on this path)\n\
             reference : orbit_len={} sa_skip={} bla_on={}\n\
             bracket   : iterations [0,{}) — windows ending at or below sa_skip are FREE BY
                         CONSTRUCTION (the shader seeds iter=sa_skip); they print as ~ and
                         are excluded from every figure below.\n",
            fractadyne_core::to_decimal_string(&self.viewport.center_x),
            fractadyne_core::to_decimal_string(&self.viewport.center_y),
            self.viewport.log2_magnification(),
            panel[0], panel[1], base.mode, base.formula,
            base.orbit_len, base.sa_skip, base.bla_on,
            iters,
        );
        if base.orbit_len == 0 {
            println!(
                "ABORT: the request carries NO reference (orbit_len=0). Nothing measured — a sweep\n\
                 without the live reference measures a regime the view never renders."
            );
            return;
        }

        // ---- axis 1: window size, fixed resolution ----------------------------------------
        let mut cells: Vec<Cell> = Vec::new();
        let mut reference_pixels: Option<(u32, Vec<f32>)> = None;
        let mut stopped_at: Option<u32> = None;

        println!("── window axis · {}x{} px, [0,{}) ──", panel[0], panel[1], iters);
        for &w in WINDOWS {
            let mut req = base.clone();
            req.max_iter = iters;
            let mut passes = Vec::new();
            let t0 = std::time::Instant::now();
            let r = match fractadyne_gpu::render_iter_chunked_timed(
                device, queue, &req, w, &mut passes,
            ) {
                Ok(r) => r,
                Err(e) => {
                    println!("  window {w:>5}: GPU ERROR — {e}");
                    continue;
                }
            };
            if passes.is_empty() {
                println!(
                    "  window {w:>5}: NOT MEASURED — the unsupported-scope fallback ran ONE\n\
                     \x20              unbounded render_iter dispatch instead of windows. Mode {},\n\
                     \x20              formula {}: outside the chunk shaders' scope.",
                    req.mode, req.formula
                );
                continue;
            }
            let real: Vec<&fractadyne_gpu::ChunkPassTiming> =
                passes.iter().filter(|p| p.end_iter > skip).collect();
            let max = real.iter().map(|p| p.wall_ms).fold(0.0_f64, f64::max);
            let sum: f64 = real.iter().map(|p| p.wall_ms).sum();
            print!(
                "  window {w:>5}: {:>3} passes ({:>3} past skip)  total {sum:8.1} ms  worst {max:7.1} ms  |",
                passes.len(),
                real.len()
            );
            for p in &passes {
                // A window that ends inside the skip did no work; print it as `~` so a reader
                // cannot mistake a structural zero for a measured one.
                if p.end_iter <= skip {
                    print!(" ~");
                } else {
                    print!(" {:.0}", p.wall_ms);
                    cells.push(Cell {
                        window: w,
                        start: p.start_iter,
                        wall_ms: p.wall_ms,
                        full: p.end_iter - p.start_iter == w,
                    });
                }
            }
            println!("  (ms per window, in cursor order)  {:.1} s wall", t0.elapsed().as_secs_f64());

            // Pixel identity across window sizes is this path's stated contract; assert it while
            // we are here rather than trusting it.
            match &reference_pixels {
                None => reference_pixels = Some((w, r.pixels)),
                Some((w0, px0)) => {
                    let diff = px0
                        .iter()
                        .zip(r.pixels.iter())
                        .filter(|(a, b)| a != b)
                        .count();
                    if diff > 0 {
                        println!(
                            "  ⚠PIXEL DRIFT: window {w} differs from window {w0} in {diff} floats \
                             — render_iter_chunked promises bit-identical output for ANY chunk_iters."
                        );
                    }
                }
            }

            if max > SAFETY_MS {
                stopped_at = Some(w);
                println!(
                    "  ── ladder STOPPED: a single window cost {max:.0} ms, past the {SAFETY_MS:.0} ms\n\
                     \x20    safety bound. The next size up is ~2x and would approach the ~2 s watchdog.\n\
                     \x20    Larger windows were NOT measured — do not read their absence as cheap."
                );
                break;
            }
        }

        if cells.is_empty() {
            println!("\nNothing measured. No verdict.");
            return;
        }

        // ---- axis 2: pixel area, fixed window ---------------------------------------------
        // A true SUB-RECT at native pixel scale: `Viewport::set_size` changes only `width_px`/
        // `height_px` and leaves `units_per_pixel` alone, so the complex span shrinks with the
        // pixel count and every sample sits exactly where it sat in the full frame.
        //
        // DOWNSCALING WAS TRIED FIRST AND IS INVALID. Rendering the same view at fewer, coarser
        // samples measures which points you happened to land on, not how much area costs: it
        // produced 1646 ms at 1280x966, 5262 ms at 640x483, 211 ms at 320x242 and 4887 ms at
        // 160x121 - non-monotonic by 23x, because a coarse grid either hits a long-chain interior
        // pixel or misses it. Only a sub-rect holds the sample set fixed while the count varies.
        //
        // A sub-rect is also the thing CELLWALK would actually dispatch, so this is the go/no-go
        // in its own terms rather than a proxy for it.
        let area_window = 256u32;
        println!("\n── area axis · fixed window {area_window}, [0,{iters}) ──");
        let saved_width = self.export.width;
        let mut area_pts: Vec<(f64, f64)> = Vec::new();
        for &s in AREA_SCALES {
            self.export.width = (((saved_width as f64) * s).round().max(16.0) as u32) & !1u32;
            // EVEN dimensions, forced. The four scales first ran as 1280x966, 640x483,
            // 320x242, 160x121 and came back 1640 / 5261 / 206 / 4853 ms - alternating fast/slow
            // in perfect correlation with the HEIGHT's parity, not with the pixel count. An odd
            // render-target height costs ~25x on this path, which is a real defect in its own
            // right and is filed separately; here it is simply held constant so the axis measures
            // area.
            let mut vp = self.viewport.clone();
            let even = |v: f64| -> f64 { ((v as u32).max(16) & !1u32) as f64 };
            vp.set_size(
                even(self.viewport.width_px * s),
                even(self.viewport.height_px * s),
            );
            let mut req = self.autopilot_probe_request(&vp, self.julia_mode);
            req.max_iter = iters;
            // SA OFF for this axis, and that is not optional. `sa_skip` is derived per REQUEST,
            // so it changes with resolution - the first version of this axis reported 129.6 ms at
            // 1280x966, 5114.9 ms at 640x483 and 49.2 ms at 320x242, a 40x non-monotonic spread
            // that is nothing but each row walking a different amount of real work. Forcing
            // sa_skip=0 makes every row iterate the SAME range from zero, which is the only way
            // the pixel-count question gets asked in isolation.
            let native_skip = req.sa_skip;
            req.sa_skip = 0;
            // `height` is DERIVED from the aspect ratio, so forcing the viewport even is not
            // enough - 320x241 still came out odd and cost 4908 ms against 470 ms for an even
            // 640x482 four times its size. Pin both dimensions even here, at the request.
            req.width &= !1u32;
            req.height &= !1u32;
            let px = (req.width as f64) * (req.height as f64);
            let mut passes = Vec::new();
            match fractadyne_gpu::render_iter_chunked_timed(
                device, queue, &req, area_window, &mut passes,
            ) {
                Ok(_) if !passes.is_empty() => {
                    let sum: f64 = passes.iter().map(|p| p.wall_ms).sum();
                    println!(
                        "  {:>5}x{:<5} ({:>10.0} px): total {sum:8.1} ms  worst {:7.1} ms  \
                         ({} windows, SA forced off; native sa_skip here would be {native_skip})",
                        req.width, req.height, px,
                        passes.iter().map(|p| p.wall_ms).fold(0.0_f64, f64::max),
                        passes.len(),
                    );
                    area_pts.push((px, sum));
                }
                Ok(_) => println!("  {:>5}x{:<5}: NOT MEASURED (scope fallback)", req.width, req.height),
                Err(e) => println!("  {:>5}x{:<5}: GPU ERROR — {e}", req.width, req.height),
            }
        }
        self.export.width = saved_width;

        // ---- axis 3: render-target HEIGHT ---------------------------------------------------
        // The area axis above pins BOTH dimensions even to dodge a filed defect: an odd render
        // height measured ~25x on this path (640x483 = 5261 ms against 640x482 = 473 ms, and
        // 320x241 = 4909 ms against 320x240 = 193 ms). Dodging it made that axis readable but left
        // the defect uncharacterised, so this axis measures it head-on and answers the two
        // questions the filing could not:
        //
        //   1. PARITY or ALIGNMENT? A single odd/even pair cannot tell "odd is slow" from "not a
        //      multiple of 4" or "not a multiple of 8". Walking h-4..h+4 can.
        //   2. Is it THIS path? `render_iter` runs the same view as one unbounded dispatch with no
        //      ping-pong state textures at all. If the unchunked control is flat across the same
        //      heights while the chunked arm cliffs, the state targets are the suspect and the
        //      iterate shader is exonerated. If BOTH cliff, it is not a chunking defect.
        //
        // Both arms force `sa_skip = 0` and hold width fixed, for the same reason the area axis
        // does: a derived skip that moves with the request would make every row walk a different
        // amount of real work, which is how this harness's first area axis produced a 40x
        // non-monotonic spread that meant nothing.
        let h_window = 256u32;
        let base_h = (self.viewport.height_px as u32).max(16);
        // ⚠NO WIDTH IN THIS HEADER, deliberately. `autopilot_probe_request` derives the request
        // resolution from the viewport rather than copying it (a 1920-wide window produced a
        // 1280-wide request here), so any width named up front is a guess. Every row prints the
        // width it actually rendered.
        //
        // ⚠THREE HEIGHT SCALES. The filed 25x was measured at 640x483 and 320x241 — small
        // targets, never the full frame — and the penalty does grow as the target shrinks, so a
        // sweep at one size would understate it. The SCALE IS APPLIED TO THE BASE FIRST and the
        // +-4 offsets walked around the scaled value: scaling after the offset collapses
        // neighbouring heights onto the same value (1098 and 1099 both floor to 549 at 0.5) and
        // silently measures the same row twice, which the first version of this axis did.
        //
        // ⚠**CORRECTION to an earlier note here**, which said the request width is derived from
        // the viewport. It is not. `autopilot_probe_request` (render.rs:2069) reads
        // `width = self.export.width`, and derives only the HEIGHT from the viewport's aspect —
        // so the 1280 an earlier run saw came from the session's `export_width`, not the 1920-wide
        // window. `self.export.width` IS the width knob (the area axis already drives it), and
        // both dimensions are scaled together below so the rows match the geometry the defect was
        // originally filed at (640x483, 320x241).
        println!("
── height axis · window {h_window}, [0,{iters}) ──");
        println!("      target        chunked                      unchunked control");
        let saved_export_w = self.export.width;
        for &hs in &[1.0_f64, 0.5_f64, 0.25_f64] {
        // Scale the WIDTH with the height, so a row is a smaller render rather than a squashed
        // one — the filed 25x was at 640x483 and 320x241, both halvings of 1280x966.
        self.export.width = (((saved_export_w as f64) * hs).round().max(16.0) as u32) & !1u32;
        let scaled_base = (((base_h as f64) * hs) as u32).max(32);
        for &d in HEIGHT_OFFSETS {
            let h = match scaled_base.checked_add_signed(d) {
                Some(v) if v >= 16 => v,
                _ => continue,
            };
            let mut vp = self.viewport.clone();
            vp.set_size(self.viewport.width_px, h as f64);
            let mut req = self.autopilot_probe_request(&vp, self.julia_mode);
            req.max_iter = iters;
            req.sa_skip = 0;
            req.width &= !1u32; // width held EVEN and fixed; height is the only variable
            req.height = h;     // pinned, not derived — the aspect ratio would round it back

            let mut passes = Vec::new();
            let chunked = match fractadyne_gpu::render_iter_chunked_timed(
                device, queue, &req, h_window, &mut passes,
            ) {
                Ok(_) if !passes.is_empty() => {
                    let sum: f64 = passes.iter().map(|p| p.wall_ms).sum();
                    let worst = passes.iter().map(|p| p.wall_ms).fold(0.0_f64, f64::max);
                    Some((sum, worst, passes.len()))
                }
                // An empty pass list is the unbounded fallback, never a fast render (see the
                // module header). Report it as unmeasured so it cannot read as a fast row.
                Ok(_) => None,
                Err(e) => {
                    println!("  {:>5}x{:<5}: GPU ERROR — {e}", req.width, h);
                    continue;
                }
            };
            // ⚠⚠THE CONTROL IS THE DANGEROUS ARM, NOT THE MEASURED ONE. `render_iter` runs the
            // WHOLE bracket as ONE unbounded dispatch — precisely the shape that loses the device,
            // and the shape the chunked path exists to eliminate. At [0,8192) it cost ~326 ms and
            // was fine; at the 219-window bracket the original defect was measured on, the same
            // call is ~2.2 s, i.e. past the ~2 s Windows TDR. Running it there would trade a
            // measurement for a device loss.
            //
            // The chunked total just measured is the right predictor: both arms do the same work,
            // so it estimates the single dispatch within the factor that is the whole question.
            // Gate on it with the harness's own SAFETY_MS, and SAY the control was skipped —
            // a blank column would read as "no control" rather than "control refused".
            let est_ms = chunked.map(|(sum, _, _)| sum).unwrap_or(0.0);
            let (plain, plain_ms, skipped) = if est_ms >= SAFETY_MS {
                (None, 0.0, true)
            } else {
                let t0 = std::time::Instant::now();
                let r = fractadyne_gpu::render_iter(device, queue, &req).ok();
                (r, t0.elapsed().as_secs_f64() * 1000.0, false)
            };
            let control = match (&plain, skipped) {
                (_, true) => format!("SKIPPED (est {est_ms:.0} ms >= {SAFETY_MS:.0} ms TDR guard)"),
                (Some(_), _) => format!("{plain_ms:8.1} ms (1 dispatch)"),
                (None, _) => "   GPU ERROR".to_string(),
            };

            let tag = if h % 8 == 0 {
                "h%8=0"
            } else if h % 4 == 0 {
                "h%4=0"
            } else if h % 2 == 0 {
                "even "
            } else {
                "ODD  "
            };
            match chunked {
                Some((sum, worst, n)) => println!(
                    "  {:>5}x{:<5} {tag}  total {sum:8.1} ms  worst {worst:7.1} ms ({n:>3} win)                        {control}",
                    req.width,
                    h,
                ),
                None => println!(
                    "  {:>5}x{:<5} {tag}  NOT MEASURED (scope fallback)   {control}",
                    req.width,
                    h,
                ),
            }
        }
        }
        self.export.width = saved_export_w;

        self.report_verdict(&cells, &area_pts, stopped_at);
    }

    /// Turn the measurements into the two statements the design decision needs. Deliberately
    /// separate from the measuring, so the numbers above stand on their own if the reasoning
    /// below is ever wrong.
    fn report_verdict(
        &self,
        cells: &[Cell],
        area_pts: &[(f64, f64)],
        stopped_at: Option<u32>,
    ) {
        println!("\n── verdict ──");

        // Q1: is a window's cost proportional to its iteration count? Compare the cost per
        // iteration at the SAME cursor region across sizes; a size-independent figure means the
        // window is a real actuator.
        // THE ACTUATOR QUESTION is how the WORST single submission scales with the window - that
        // is the quantity the watchdog sees. Averaging cost-per-iteration answers a different and
        // misleading question: small windows carry a fixed per-pass overhead, so their
        // per-iteration figure is inflated and a naive ratio reads as "not linear" on data that
        // is close to it. The first run of this harness did exactly that.
        let worst = |w: u32| -> Option<f64> {
            let v = cells
                .iter()
                .filter(|c| c.window == w && c.full)
                .map(|c| c.wall_ms)
                .fold(f64::NAN, f64::max);
            if v.is_nan() { None } else { Some(v) }
        };
        let sizes: Vec<(u32, f64)> =
            WINDOWS.iter().filter_map(|&w| worst(w).map(|v| (w, v))).collect();
        if sizes.len() >= 2 {
            print!("  worst FULL window by size:");
            for (w, ms) in &sizes {
                print!(" {w}->{ms:.1}ms");
            }
            println!();
            let mut ratios = Vec::new();
            for pair in sizes.windows(2) {
                if pair[1].0 == pair[0].0 * 2 {
                    ratios.push(pair[1].1 / pair[0].1.max(f64::MIN_POSITIVE));
                }
            }
            if !ratios.is_empty() {
                let mean = ratios.iter().sum::<f64>() / ratios.len() as f64;
                print!("  cost ratio per DOUBLING:");
                for r in &ratios {
                    print!(" {r:.2}x");
                }
                println!("  (mean {mean:.2}x; 2.00x is exactly linear)");
                if mean >= 1.5 {
                    println!(
                        "  => THE WINDOW IS A REAL ACTUATOR. Halving it roughly halves the worst\n\
                         \x20  submission, so the iteration axis CAN bound this dispatch and the\n\
                         \x20  defect is upstream - in whatever SIZED the window."
                    );
                } else if mean >= 1.1 {
                    println!(
                        "  => PARTIAL ACTUATOR. Halving the window buys real but sub-proportional\n\
                         \x20  relief; a fixed per-pass cost dominates at small sizes."
                    );
                } else {
                    println!(
                        "  => NOT AN ACTUATOR. Window size barely moves the worst submission, so\n\
                         \x20  sizing on the iteration axis cannot bound this regime."
                    );
                }
            }
        }

        // Q2: does cost cliff WITHIN the bracket? This is the honest-price branch's prediction:
        // the fatal ladder's third pass would have to be many times its predecessors' rate.
        let w = 256u32;
        let mut band: Vec<&Cell> = cells.iter().filter(|c| c.window == w && c.full).collect();
        band.sort_by_key(|c| c.start);
        if band.len() >= 2 {
            let lo = band.iter().map(|c| c.wall_ms).fold(f64::INFINITY, f64::min);
            let hi = band.iter().map(|c| c.wall_ms).fold(0.0_f64, f64::max);
            println!(
                "  past the skip at window {w}: [{},{}) — cheapest {lo:.1} ms, dearest {hi:.1} ms                  ({:.1}x spread across {} measured windows)",
                band.first().map(|c| c.start).unwrap_or(0),
                band.last().map(|c| c.start + w).unwrap_or(0),
                hi / lo.max(f64::MIN_POSITIVE),
                band.len()
            );
            if hi / lo.max(f64::MIN_POSITIVE) >= 4.0 {
                println!(
                    "  ⇒ A REAL CLIFF EXISTS inside one band. The honest-price branch is live: a\n\
                     \x20  ledger that grew on early windows cannot price later ones, and no\n\
                     \x20  clock fix reaches that."
                );
            } else {
                println!(
                    "  ⇒ NO CLIFF in this bracket. The ≥8x jump the honest-price branch requires\n\
                     \x20  is not here, which leaves the recorded ≤200 ms prices unexplained by\n\
                     \x20  cost — i.e. they were fabricated."
                );
            }
        }

        // Q3: the CELLWALK go/no-go. Fit wall ~ area^a by least squares in log-log.
        if area_pts.len() >= 3 {
            let n = area_pts.len() as f64;
            let sx: f64 = area_pts.iter().map(|(a, _)| a.ln()).sum();
            let sy: f64 = area_pts.iter().map(|(_, t)| t.ln()).sum();
            let sxx: f64 = area_pts.iter().map(|(a, _)| a.ln() * a.ln()).sum();
            let sxy: f64 = area_pts.iter().map(|(a, t)| a.ln() * t.ln()).sum();
            let a = (n * sxy - sx * sy) / (n * sxx - sx * sx);
            let monotonic = area_pts.windows(2).all(|w| w[1].1 <= w[0].1 * 1.15);
            println!("  area exponent: wall ~ area^{a:.3}");
            if !monotonic {
                println!(
                    "  => NO VERDICT: total cost is NOT monotonic in area, so the fit above is\n\
                     \x20  meaningless. Something other than pixel count is moving between rows;\n\
                     \x20  find it before reading an exponent off this."
                );
            } else if a < 0.2 {
                println!(
                    "  ⇒ AREA BUYS NOTHING (a≈0). Abandon spatial sub-budgets at any window size —\n\
                     \x20  the 'wrong axis' verdict extends to bounded windows too."
                );
            } else if a > 0.7 {
                println!(
                    "  ⇒ AREA IS A REAL ACTUATOR (a≈1). A cell rect has genuine multiplicative\n\
                     \x20  headroom below the 256-iteration floor, which is the one place nothing\n\
                     \x20  else can reach."
                );
            } else {
                println!("  ⇒ PARTIAL (0.2<a<0.7). A rect helps sub-linearly; weigh against its cost.");
            }
        }

        if let Some(w) = stopped_at {
            println!(
                "\n  ⚠Ladder stopped at window {w} for safety; sizes above it are UNMEASURED.\n\
                 \x20 Re-run with a smaller bracket if the curve above needs extending."
            );
        }
    }
}
