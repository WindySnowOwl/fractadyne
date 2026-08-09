//! `--livetest`: validates what the LIVE view actually SHOWS during scripted playback.
//!
//! Every other harness here measures timing (`--divetest`), offline output (`--render`, the
//! goldens, the F3 corpus) or isolated sub-passes (`--reusetest`). None of them can see the class
//! of defect that has produced eight separate bugs in this codebase: *the live view is black (or
//! flat) where the same view renders fine offline*. That failure is invisible to a timing harness
//! (the frames are fast — they're cheap **because** they're starved) and invisible to an offline
//! harness (the offline path doesn't have the live path's caps).
//!
//! So this harness plays a tour through the IDENTICAL live machinery `--divetest` drives —
//! `advance_playback_core` (pacer, camera sampling, reference lookahead) and `build_params`
//! (reuse-hold, freeze guard, motion budget, iteration caps) — but keeps the PIXELS, and at each
//! checkpoint renders the same view through the offline path as an oracle:
//!
//! > **the live view should show what an offline render of the same view, at the same iteration
//! > budget, shows.**
//!
//! Everything else — the pixel clamp, the adaptive budget's climb, the plateau detector, the
//! reference-install cadence, reprojection staleness — is machinery in service of that contract,
//! so any deviation is a live-path defect with a number attached. Checkpoints are keyframe HOLDS:
//! the camera is stationary and the view has had the whole hold to resolve, so a starved or stale
//! frame there is unambiguously a defect rather than motion.
//!
//! Verdict per checkpoint (`black` = pixels that never escaped, i.e. interior-colored):
//!   - **FAIL** live is ≥10 points blacker than the oracle — it resolved materially less.
//!   - **warn** ≥3 points blacker, or the colored images differ by a mean of ≥24/255 at the SAME
//!     blackness (see `verdict`: colour alone is not evidence of a defect at a dense deep field).
//! Failures dump `live`/`truth` PNGs so the difference can be looked at, not just read.

use crate::FractadyneApp;
use std::time::Instant;

/// One validated moment of the tour.
struct Checkpoint {
    /// Tour time (s) and the keyframe id it belongs to.
    t: f64,
    id: String,
    log10mag: f64,
    /// Iteration budget the live frame actually ran at, and the appetite it was derived from.
    gpu_iter: u32,
    eff_iter: u32,
    boost: f64,
    /// Live reference state at the moment of capture.
    orbit_len: u32,
    partial: bool,
    /// Resolution the live frame was iterated at (motion-res can reduce it).
    res: [u32; 2],
    /// Milliseconds since the previous frame that really re-iterated (0 = this frame did).
    stale_ms: f64,
    /// Fraction of pixels that never escaped — the "black screen" measure.
    live_black: f64,
    truth_black: f64,
    /// Perceptual difference of the colored frames (0–255), when the oracle ran.
    srgb_mean: f64,
    srgb_max: u32,
    truth_ran: bool,
}

impl Checkpoint {
    /// Excess blackness over the oracle, in percentage points — the headline number.
    fn excess_black(&self) -> f64 {
        (self.live_black - self.truth_black) * 100.0
    }
    /// Excess blackness is the GATE; the sRGB difference only ever warns.
    ///
    /// The two metrics are not equally trustworthy. "The live view resolved less of this view than
    /// an offline render did" is unambiguous. A colour difference at the same blackness is not: at
    /// a dense deep field the palette maps a huge escape-iteration range into one cycle, so colour
    /// is hypersensitive to sub-iteration differences between two independently picked references
    /// — the same aliasing that made corpus locations 14/15 look wrong when the rendering was
    /// right. Measured at the 6.5e94× hold: live and offline agree to +0.0 points of blackness at
    /// the full 4 000 000 iterations the script asked for, and still differ by 28.6 sRGB. Failing
    /// that would be calling a correct render broken.
    fn verdict(&self) -> &'static str {
        if !self.truth_ran {
            return "----";
        }
        if self.excess_black() >= 10.0 {
            "FAIL"
        } else if self.excess_black() >= 3.0 || self.srgb_mean >= 24.0 {
            "warn"
        } else {
            "ok"
        }
    }
}

/// Fraction of pixels that never escaped, from an RGBA32F iterate buffer (channel 0 < 0 = interior).
/// This is what "black" means before any palette is applied — a pixel the renderer could not
/// resolve is indistinguishable from a pixel that genuinely belongs to the set.
fn interior_frac(px: &[f32]) -> f64 {
    let n = px.len() / 4;
    if n == 0 {
        return 0.0;
    }
    px.chunks_exact(4).filter(|p| p[0] < 0.0).count() as f64 / n as f64
}

/// Max + mean absolute per-channel difference of two sRGB8 buffers (0–255), as the goldens use.
fn img_diff(a: &[u8], b: &[u8]) -> (u32, f64) {
    if a.len() != b.len() || a.is_empty() {
        return (255, 255.0);
    }
    let (mut max, mut sum) = (0u32, 0u64);
    for (&x, &y) in a.iter().zip(b) {
        let d = (x as i32 - y as i32).unsigned_abs();
        max = max.max(d);
        sum += d as u64;
    }
    (max, sum as f64 / a.len() as f64)
}


// ---------------------------------------------------------------------------------------------
// BASELINE. `--livetest` used to exit non-zero whenever any checkpoint failed — and three of the
// grand tour's checkpoints fail today for a documented reason (the `LIVE_REF_CAP` pixel clamp, an
// open item in TODO.md). A harness that can never go green is not a gate: establishing that
// "18 ok / 1 warn / 3 FAIL at these exact numbers" WAS the expected state cost two full 331 s runs
// of a rebuilt pre-change binary during the beta.47 work, and skipping that would have shipped the
// iteration-boost regression it caught.
//
// So compare against a blessed baseline and fail on CHANGE, not on failure — the contract
// `--bench-matrix` already uses. A recorded FAIL is fine; a checkpoint that differs from what was
// recorded is not.
const LIVETEST_BASELINE_DIR: &str = "benchmarks";

/// Percentage-point tolerance (as a fraction) on the black measurements. These were bit-stable
/// across repeated runs on one machine; the slack only stops a single pixel reading as drift.
const BLACK_TOL: f64 = 0.005;
/// Relative tolerance on the iteration/orbit COUNTS. They are wall-clock sensitive in a way the
/// outcome is not: a change that only shifts WHEN the checkpoint samples the appetite moves them a
/// few iterations in 146,000 with the verdict, black fractions, boost and resolution all identical
/// (measured after the A3 refusal fix: 146,112 → 146,109). Exact matching turned that into a red
/// gate, which is how a gate starts getting ignored. The regressions this must catch are nothing
/// like it — 146,112 → 86,227 is 41% — so 1% keeps all the signal and none of the noise.
const COUNT_TOL: f64 = 0.01;

/// Did a count move by more than `COUNT_TOL`, relative to the blessed value?
fn count_drifted(base: u32, cur: u32) -> bool {
    if base == cur {
        return false;
    }
    let denom = base.max(1) as f64;
    ((cur as f64 - base as f64).abs() / denom) > COUNT_TOL
}

/// One checkpoint's blessed state. Deliberately excludes `stale_ms` (a timing, not an outcome) and
/// `t` (the schedule, not the result).
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
struct BaseCp {
    verdict: String,
    gpu_iter: u32,
    eff_iter: u32,
    boost: f64,
    orbit_len: u32,
    partial: bool,
    res: [u32; 2],
    live_black: f64,
    truth_black: f64,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct LivetestBaseline {
    version: String,
    gpu: String,
    utc: String,
    tour: String,
    size: [u32; 2],
    checkpoints: std::collections::BTreeMap<String, BaseCp>,
}

/// Per (tour, size) — a `--segment` run reaches a subset of the same checkpoint ids and compares
/// only those, but a different SIZE is a different live path and deserves its own baseline.
fn baseline_path(tour: &std::path::Path, size: [u32; 2]) -> std::path::PathBuf {
    let stem = tour.file_stem().and_then(|s| s.to_str()).unwrap_or("tour");
    crate::selftest::anchored(LIVETEST_BASELINE_DIR)
        .join(format!("livetest-{stem}-{}x{}.json", size[0], size[1]))
}

impl FractadyneApp {
    /// `--livetest FILE`: play a tour in real time through the live pipeline and validate the
    /// frames it puts on screen against offline renders of the same views. Returns the number of
    /// failing checkpoints (the caller turns that into an exit code).
    pub(crate) fn run_livetest(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        tour: &std::path::Path,
        segment: Option<&str>,
        size: [u32; 2],
        out_dir: &std::path::Path,
        quick: bool,
    ) -> usize {
        const VSYNC: f64 = 1.0 / 60.0;
        let resolution = size;

        // Live-ish config: single Mandelbrot-family view, smooth coloring, and the GUI's dive
        // default for the motion-resolution floor (a session floor of 1.0 forbids resolution cuts,
        // which changes what the live path does — see --divetest).
        self.dual = false;
        self.julia_mode = false;
        self.coloring.color_method = crate::ColorMethod::Smooth;
        if std::env::var("FRACTADYNE_LIVETEST_SESSION_RES").is_err() {
            self.render_cfg.min_motion_res = 0.30;
        }
        self.viewport.set_size(resolution[0] as f64, resolution[1] as f64);

        let mut pb = match crate::scripting::parse_tour_file(tour) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("livetest: cannot load {}: {e}", tour.display());
                crate::exit(2);
            }
        };
        // Optional chapter window (reuses the script's own [[segment]] definitions).
        let (t_from, t_to) = match segment {
            Some(name) => match pb.find_segment(name) {
                Ok(s) => (s.start, s.end),
                Err(e) => {
                    eprintln!("livetest: {e}");
                    crate::exit(2);
                }
            },
            None => (0.0, pb.total),
        };
        // Checkpoints inside the window; nothing to validate without a hold.
        let mut pending: Vec<(f64, String)> = pb
            .hold_checkpoints()
            .into_iter()
            .filter(|(t, _)| *t >= t_from && *t <= t_to)
            .collect();
        if pending.is_empty() {
            eprintln!(
                "livetest: no keyframe holds in {}{} — nothing to validate (a hold is where the \
                 camera stops and the view is meant to resolve)",
                tour.display(),
                segment.map(|s| format!(" segment \"{s}\"")).unwrap_or_default()
            );
            crate::exit(2);
        }
        pending.reverse(); // pop() takes the earliest

        println!(
            "Fractadyne livetest — {} · {}×{} · {} checkpoint(s) over {:.0}s{}",
            tour.display(),
            resolution[0],
            resolution[1],
            pending.len(),
            t_to - t_from,
            if quick { " · oracle OFF (--quick)" } else { "" },
        );
        println!(
            "  {:>7} {:<16} {:>9} {:>9} {:>8} {:>7} {:>7} {:>8} {:>7}  {}",
            "t", "keyframe", "zoom", "live blk", "truth", "Δblk", "sRGB", "iters", "stale", "verdict"
        );
        println!("  {}", "-".repeat(104));

        let t_base = Instant::now();
        pb.cur_t = t_from; // seed the clock at the segment start
        pb.started = false;
        pb.last_now = None;
        self.playback = Some(pb);
        self.invalidate_refs();

        let mut results: Vec<Checkpoint> = Vec::new();
        // The last frame that really re-iterated: its pixels ARE what the screen shows until the
        // next one (everything between is a reprojection of it).
        let mut last_real: Option<(Vec<f32>, [u32; 2], f64)> = None; // (pixels, res, wall time)
        let mut fail_dumps = 0usize;

        loop {
            let now = t_base.elapsed().as_secs_f64();
            match self.advance_playback_core(now) {
                crate::scripting::PlaybackTick::Playing => {}
                _ => break, // tour ended
            }
            let e = self.playback.as_ref().map(|p| p.cur_t).unwrap_or(0.0);
            if e > t_to {
                break;
            }
            let ft = Instant::now();
            // Mirror `draw_central`'s per-frame build exactly (single view) — this is the whole
            // point: the frame under test must be built by the live path, not a convenience path.
            let center_bf = [self.viewport.center_x.clone(), self.viewport.center_y.clone()];
            let center = self.viewport.center_f64();
            let span = self.viewport.complex_span_fe();
            let mag = self.viewport.magnification();
            let l2 = self.viewport.log2_magnification();
            let eff_iter = if self.render_cfg.auto_iter {
                self.viewport.recommended_max_iter(self.render_cfg.max_iter)
            } else {
                self.render_cfg.max_iter
            };
            // `interacting` must be DERIVED, exactly as `draw_central` derives it — not assumed.
            // `--divetest` hardcodes `true` because its windows are always mid-dive, but that is
            // the very flag this harness exists to exercise: it gates the adaptive iteration
            // budget, so hardcoding it would silently make every hold look starved no matter what
            // the app does.
            let interacting = now - self.pointer.settle_t[0] < crate::SETTLE_DELAY;
            // AA stays at 1: the settle ramp trades cost for edge quality, which moves the frame
            // budget around without changing whether a pixel escapes. Holding it fixed keeps the
            // comparison about resolution of structure, which is what "black" means here.
            let params = self.build_params(
                center_bf, center, span, mag, l2, self.fractal, false, eff_iter, interacting, 1,
                resolution, 0, None,
            );
            let real = params.reproject == 0;
            let build_ms = ft.elapsed().as_secs_f64() * 1000.0;
            let mut gpu_ms = 0.0;
            if real {
                let req = crate::profile::params_to_request(&params);
                let t_gpu = Instant::now();
                // Pure-GPU iterate time via TIMESTAMP_QUERY (as `--divetest` does): `render_iter`'s
                // wall time includes a full texture readback the GUI never pays, and that time
                // feeds the watchdog budget below.
                let (rendered, ts) =
                    fractadyne_gpu::timing::capture(|| fractadyne_gpu::render_iter(device, queue, &req));
                if let Ok(r) = rendered {
                    // Stand in for the live view's GPU counter readback. The on-screen path arms a
                    // counter pass that reports `CTR_MAXITER / pixels` back through this sink, and
                    // the ADAPTIVE ITERATION BUDGET is driven entirely by it — but that pass
                    // belongs to the live view object, not the offscreen `render_iter` the harness
                    // uses, so without this the budget could never climb HERE and the harness would
                    // fail views the GUI resolves. The quantity is identical (a pixel that
                    // exhausted `max_iter` is exactly one whose smooth-iter is negative) and the
                    // packing matches `timing.rs`: (frac f32 bits << 32) | the budget it was
                    // measured at, which the controller checks so a stale reading is discarded.
                    let frac = interior_frac(&r.pixels) as f32;
                    let packed = ((frac.to_bits() as u64) << 32) | params.max_iter as u64;
                    self.perf.maxiter_sink[0]
                        .store(packed, std::sync::atomic::Ordering::SeqCst);
                    last_real = Some((r.pixels, [r.width, r.height], t_base.elapsed().as_secs_f64()));
                }
                gpu_ms = t_gpu.elapsed().as_secs_f64() * 1000.0;
                // Maintain the watchdog step budget the way the GUI's event loop does. Without
                // this it stays 0, `build_params` falls back to the pessimistic bootstrap, and
                // every deep frame is shrunk to a fraction of the requested size — the harness
                // would then be measuring a view the GUI never renders. (The GUI keeps this loop
                // in `update`; the harness has no event loop, so it runs the same arithmetic.)
                let ms = if ts.captured { ts.iterate_ms } else { gpu_ms };
                if ms > 0.01 && self.perf.fe_steps_last[0] > 0 {
                    let cur = self.perf.fe_budget[0].max(crate::render::TDR_BOOTSTRAP_STEPS);
                    let factor = (crate::render::TDR_BUDGET_MS / ms)
                        .clamp(crate::render::TDR_SHRINK_MAX, crate::render::TDR_GROW_MAX);
                    let next = ((cur as f64 * factor) as u64)
                        .clamp(crate::render::TDR_BOOTSTRAP_STEPS, crate::render::TDR_STEPS_CEIL);
                    self.perf.fe_budget_ok[0] = next == cur || (0.8..=1.25).contains(&factor);
                    self.perf.fe_budget[0] = next;
                }
            }
            let frame_ms = (build_ms + gpu_ms).max(VSYNC * 1000.0);
            self.perf.last_dt_ms = frame_ms;
            self.perf.frame_ms = frame_ms;

            // Checkpoint due? Capture on a frame that really re-iterated, so the pixels under test
            // are this view's — but never wait past the checkpoint's own keyframe, or a hold that
            // never re-iterates would silently produce no measurement at all (itself a defect,
            // recorded here as a large `stale`).
            let due = pending.last().map(|(ct, _)| e >= *ct).unwrap_or(false);
            if due && (real || last_real.is_some()) {
                let (t_cp, id) = pending.pop().unwrap();
                let Some((live_px, live_res, real_at)) = last_real.clone() else { continue };
                let stale_ms = (t_base.elapsed().as_secs_f64() - real_at) * 1000.0;
                let live_black = interior_frac(&live_px);

                // ---- oracle: the same view, same iteration budget, through the offline path ----
                // The tour clock must not run while this happens (a deep truth render can take
                // minutes), or the tour would teleport forward mid-validation.
                let pause = Instant::now();
                let (mut truth_black, mut srgb_mean, mut srgb_max, mut truth_ran) =
                    (0.0, 0.0, 0u32, false);
                if !quick {
                    let saved_w = self.export.width;
                    let saved_ss = self.export.ss;
                    self.export.width = live_res[0];
                    self.export.ss = 1;
                    let vp = self.viewport.clone();
                    let req = self.current_export_request_for(&vp, false);
                    if let Ok(t) = fractadyne_gpu::render_iter(device, queue, &req) {
                        truth_black = interior_frac(&t.pixels);
                        truth_ran = true;
                        // Perceptual diff through the REAL color pass, so the number is what a
                        // viewer would see rather than a raw escape-count distance.
                        let lc = fractadyne_gpu::color_iter_buffer(device, queue, &req, &live_px)
                            .ok()
                            .map(|r| fractadyne_export::to_srgb8(&r.pixels));
                        let tc = fractadyne_gpu::color_iter_buffer(device, queue, &req, &t.pixels)
                            .ok()
                            .map(|r| fractadyne_export::to_srgb8(&r.pixels));
                        if let (Some(l), Some(tt)) = (&lc, &tc) {
                            let (mx, mean) = img_diff(l, tt);
                            srgb_max = mx;
                            srgb_mean = mean;
                        }
                        // Dump the pair whenever the verdict will be FAIL — a number says a frame
                        // is wrong, an image says how.
                        let bad = (live_black - truth_black) * 100.0 >= 10.0;
                        if bad {
                            let _ = std::fs::create_dir_all(out_dir);
                            let stem = format!("livetest_{:03}_{id}", results.len());
                            for (tag, buf) in [("live", &live_px), ("truth", &t.pixels)] {
                                if let Ok(c) = fractadyne_gpu::color_iter_buffer(device, queue, &req, buf) {
                                    let p = out_dir.join(format!("{stem}_{tag}.png"));
                                    let _ = fractadyne_export::write_png(&p, c.width, c.height, &c.pixels, None);
                                }
                            }
                            fail_dumps += 1;
                        }
                    }
                    self.export.width = saved_w;
                    self.export.ss = saved_ss;
                }
                // Give the tour clock back the time the oracle stole. With an accumulating clock
                // that is simply "don't count the pause as elapsed": re-baseline `last_now`, and
                // the next tick's `dt` covers only the frame, not the minutes of oracle render.
                let _ = pause.elapsed();
                if let Some(p) = self.playback.as_mut() {
                    p.last_now = Some(t_base.elapsed().as_secs_f64());
                }

                let cp = Checkpoint {
                    t: t_cp,
                    id,
                    log10mag: l2 / std::f64::consts::LOG2_10,
                    gpu_iter: params.max_iter,
                    eff_iter,
                    boost: self.perf.iter_boost[0],
                    orbit_len: self.ref_cache[0].orbit_len,
                    partial: self.ref_cache[0].partial,
                    res: live_res,
                    stale_ms,
                    live_black,
                    truth_black,
                    srgb_mean,
                    srgb_max,
                    truth_ran,
                };
                println!(
                    "  {:>6.1}s {:<16} {:>9} {:>8.1}% {:>8.1}% {:>+6.1}pt {:>7.1} {:>8} {:>6.0}ms  {}",
                    cp.t,
                    cp.id,
                    format!("1e{:.0}", cp.log10mag),
                    cp.live_black * 100.0,
                    cp.truth_black * 100.0,
                    cp.excess_black(),
                    cp.srgb_mean,
                    cp.gpu_iter,
                    cp.stale_ms,
                    cp.verdict(),
                );
                results.push(cp);
                if pending.is_empty() {
                    break;
                }
            }

            // Vsync pacing: the live app can't run faster than the display, and the controllers
            // under test are tuned against that cadence.
            let spare = VSYNC - ft.elapsed().as_secs_f64();
            if spare > 0.0 {
                std::thread::sleep(std::time::Duration::from_secs_f64(spare));
            }
        }
        self.stop_playback();

        // ---- report ----
        let fails = results.iter().filter(|c| c.verdict() == "FAIL").count();
        let warns = results.iter().filter(|c| c.verdict() == "warn").count();
        println!();
        for c in &results {
            if c.verdict() == "FAIL" || c.verdict() == "warn" {
                println!(
                    "  {} {} at t={:.1}s (1e{:.0}×): live {:.1}% black vs {:.1}% offline; \
                     budget {} (appetite {}, boost ×{:.2}), orbit {}{}, {}×{}, {:.0}ms stale, sRGB Δ {:.1}/{}",
                    c.verdict(),
                    c.id,
                    c.t,
                    c.log10mag,
                    c.live_black * 100.0,
                    c.truth_black * 100.0,
                    c.gpu_iter,
                    c.eff_iter,
                    c.boost,
                    c.orbit_len,
                    if c.partial { " PARTIAL" } else { "" },
                    c.res[0],
                    c.res[1],
                    c.stale_ms,
                    c.srgb_mean,
                    c.srgb_max,
                );
            }
        }
        let json: String = results
            .iter()
            .map(|c| {
                format!(
                    "  {{\"t\":{:.2},\"id\":\"{}\",\"log10mag\":{:.3},\"live_black\":{:.5},\
                     \"truth_black\":{:.5},\"excess_black_pt\":{:.3},\"srgb_mean\":{:.3},\
                     \"srgb_max\":{},\"gpu_iter\":{},\"eff_iter\":{},\"boost\":{:.4},\
                     \"orbit_len\":{},\"partial\":{},\"res\":[{},{}],\"stale_ms\":{:.1},\
                     \"verdict\":\"{}\"}}",
                    c.t, c.id, c.log10mag, c.live_black, c.truth_black, c.excess_black(),
                    c.srgb_mean, c.srgb_max, c.gpu_iter, c.eff_iter, c.boost, c.orbit_len,
                    c.partial, c.res[0], c.res[1], c.stale_ms, c.verdict(),
                )
            })
            .collect::<Vec<_>>()
            .join(",\n");
        // Same `YYYYMMDD_HHMMSS` stamp the other harness logs use, so runs interleave in order.
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let path = out_dir.join(format!("livetest-{}.json", Self::file_stamp(secs)));
        let _ = std::fs::create_dir_all(out_dir);
        let _ = std::fs::write(
            &path,
            format!(
                "{{\n \"tour\": {:?},\n \"size\": [{},{}],\n \"checkpoints\": [\n{json}\n ]\n}}\n",
                tour.display().to_string(), resolution[0], resolution[1]
            ),
        );
        println!(
            "\n{} checkpoint(s): {} ok, {warns} warn, {fails} FAIL{}\nlog → {}",
            results.len(),
            results.len() - fails - warns,
            if fail_dumps > 0 { format!(" ({fail_dumps} image pair(s) dumped to {})", out_dir.display()) } else { String::new() },
            path.display(),
        );
        if fails > 0 {
            println!(
                "\nA FAIL means the live view showed materially less than an offline render of the\n\
                 SAME view at the SAME iteration budget. Read the context: `PARTIAL` orbit ⇒ the\n\
                 pixel clamp; budget « appetite ⇒ the live cap or the adaptive boost never climbed;\n\
                 large `stale` ⇒ the screen is showing an old frame reprojected."
            );
        }
        if self.selftest_bless {
            return self.bless_livetest(&results, tour, resolution);
        }
        self.compare_livetest(&results, tour, resolution, fails)
    }

    fn bless_livetest(
        &self,
        results: &[Checkpoint],
        tour: &std::path::Path,
        size: [u32; 2],
    ) -> usize {
        let baseline = LivetestBaseline {
            version: crate::version_string(),
            gpu: self.gpu_name.clone(),
            utc: crate::utc_string(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            ),
            tour: tour.display().to_string(),
            size,
            checkpoints: results
                .iter()
                .map(|c| {
                    (
                        c.id.clone(),
                        BaseCp {
                            verdict: c.verdict().to_string(),
                            gpu_iter: c.gpu_iter,
                            eff_iter: c.eff_iter,
                            boost: c.boost,
                            orbit_len: c.orbit_len,
                            partial: c.partial,
                            res: c.res,
                            live_black: c.live_black,
                            truth_black: c.truth_black,
                        },
                    )
                })
                .collect(),
        };
        let path = baseline_path(tour, size);
        if let Some(d) = path.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        match serde_json::to_string_pretty(&baseline)
            .map_err(|e| e.to_string())
            .and_then(|j| std::fs::write(&path, j + "\n").map_err(|e| e.to_string()))
        {
            Ok(()) => {
                println!(
                    "\nBlessed livetest baseline ({} checkpoint(s)) → {}",
                    results.len(),
                    path.display()
                );
                0
            }
            Err(e) => {
                eprintln!("\nFailed to write livetest baseline {}: {e}", path.display());
                1
            }
        }
    }

    /// Compare against the blessed baseline. Returns 0 when every checkpoint matches what was
    /// recorded — INCLUDING recorded FAILs, which is the point: the tour's deep holds fail for a
    /// known reason, and a gate that stays red on a known problem says nothing about a new one.
    fn compare_livetest(
        &self,
        results: &[Checkpoint],
        tour: &std::path::Path,
        size: [u32; 2],
        fails: usize,
    ) -> usize {
        let path = baseline_path(tour, size);
        let baseline: LivetestBaseline = match std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
        {
            Some(b) => b,
            None => {
                println!(
                    "\nNo livetest baseline at {} — re-run the same command with `--bless` to \
                     record one. Until then this run is graded on raw FAILs, which stays red while \
                     the tour's known-failing holds are unfixed.",
                    path.display()
                );
                return fails;
            }
        };
        let same_gpu = baseline.gpu == self.gpu_name;
        println!(
            "\nvs. baseline {} ({}, {}){}",
            baseline.version,
            baseline.gpu,
            baseline.utc,
            if same_gpu {
                ""
            } else {
                "  [different GPU — the live path adapts to measured cost, so drift here may be \
                 the hardware rather than a regression]"
            }
        );
        let (mut drift, mut missing) = (0usize, 0usize);
        for c in results {
            let Some(b) = baseline.checkpoints.get(&c.id) else {
                println!("  ? {:<16} not in baseline (new checkpoint)", c.id);
                missing += 1;
                continue;
            };
            let mut d: Vec<String> = Vec::new();
            if c.verdict() != b.verdict {
                d.push(format!("verdict {} → {}", b.verdict, c.verdict()));
            }
            if (c.live_black - b.live_black).abs() > BLACK_TOL {
                d.push(format!(
                    "live black {:.1}% → {:.1}%",
                    b.live_black * 100.0,
                    c.live_black * 100.0
                ));
            }
            if (c.truth_black - b.truth_black).abs() > BLACK_TOL {
                d.push(format!(
                    "offline black {:.1}% → {:.1}%",
                    b.truth_black * 100.0,
                    c.truth_black * 100.0
                ));
            }
            if count_drifted(b.gpu_iter, c.gpu_iter) {
                d.push(format!("budget {} → {}", b.gpu_iter, c.gpu_iter));
            }
            if count_drifted(b.eff_iter, c.eff_iter) {
                d.push(format!("appetite {} → {}", b.eff_iter, c.eff_iter));
            }
            if (c.boost - b.boost).abs() > 1.0e-6 {
                d.push(format!("boost ×{:.2} → ×{:.2}", b.boost, c.boost));
            }
            if count_drifted(b.orbit_len, c.orbit_len) || c.partial != b.partial {
                d.push(format!(
                    "orbit {}{} → {}{}",
                    b.orbit_len,
                    if b.partial { " PARTIAL" } else { "" },
                    c.orbit_len,
                    if c.partial { " PARTIAL" } else { "" }
                ));
            }
            if c.res != b.res {
                d.push(format!(
                    "res {}×{} → {}×{}",
                    b.res[0], b.res[1], c.res[0], c.res[1]
                ));
            }
            if !d.is_empty() {
                drift += 1;
                println!("  ✗ {:<16} {}", c.id, d.join("; "));
            }
        }
        let absent = baseline
            .checkpoints
            .keys()
            .filter(|k| !results.iter().any(|c| &c.id == *k))
            .count();
        println!(
            "\n{} checkpoint(s) · {drift} drifted · {missing} new{}",
            results.len(),
            if absent > 0 {
                format!(" · {absent} in baseline not reached")
            } else {
                String::new()
            }
        );
        if drift == 0 && missing == 0 {
            println!("No live-path drift (recorded FAILs are expected — see TODO.md).");
            0
        } else {
            println!(
                "Live-path DRIFT vs the blessed baseline. If the change is intended, re-run with \
                 `--bless`. The recorded FAILs are known and are NOT what this is reporting."
            );
            drift + missing
        }
    }
}
