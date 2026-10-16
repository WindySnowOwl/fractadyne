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
//! ⭐**A SECOND ORACLE, at 4× the ask (W8, U34).** "The same iteration budget" has a blind spot: a
//! view the ask itself starves is black live AND black offline, so it matched and passed. Where the
//! live frame is black enough for it to matter, the view is rendered again at four times the
//! oracle's iteration count; what that resolves and the live frame did not is starvation. Whether
//! that is a defect depends on what the app TELLS the user — the status bar's own diagnosis
//! (`limit_status`, from the same `capped_frac` / `iter_exhausted` / plateau / maxed fields the
//! frame record carries): "iter capped" or "depth limit" is the honest report of a real limit,
//! while silence, a still-climbing budget, or an "interior" conclusion (plateau, exhausted) that
//! 4× the ask contradicts is a black picture the app does not explain.
//!
//! Verdict per checkpoint (`black` = pixels that never escaped, i.e. interior-colored; see `grade`):
//!   - **FAIL** live is ≥10 points blacker than the oracle — it resolved materially less.
//!   - **STARVED** live is ≥10 points blacker than the 4× oracle, and the app does not say why.
//!   - **capped** as STARVED, but the status bar reports the limit. Not a failure.
//!   - **warn** ≥3 points blacker than either oracle, or the colored images differ by a mean of
//!     ≥24/255 at the SAME blackness (colour alone is not evidence of a defect at a dense deep field).
//! Failures dump `live`/`truth`(/`truth4x`) PNGs so the difference can be looked at, not just read.
//!
//! Exit codes follow `crate::verdict`: 0 no drift from the blessed baseline, 2 drift (or, with no
//! baseline, a failing checkpoint), 3 nothing judged (`--quick`, no checkpoint captured, or a
//! checkpoint in the window the tour never reached).

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
    /// The oracle's iteration count, and the second oracle's (0 = it did not run: the live frame
    /// was not black enough for starvation to change the grade, or 4× the ask is past the limit).
    ask: u32,
    ask4: u32,
    truth4_black: f64,
    /// The app's own account of the view at the checkpoint — what the frame record carries and
    /// the status bar reads: the settled capped fraction (`None` = no settled reading), the
    /// climb's conclusions, and the diagnosis `limit_status` would show (`None` = silent).
    capped_frac: Option<f64>,
    iter_exhausted: bool,
    iter_plateau: bool,
    budget_maxed: bool,
    app_says: Option<&'static str>,
    /// Escape-iteration range (max − min smooth iteration of escaped pixels) of the live frame and
    /// the oracle: context, so a FLAT frame (detail collapsed to one colour) is visible in the log
    /// even where the black fractions agree.
    live_range: f64,
    truth_range: f64,
}

/// Points of excess blackness that fail a checkpoint, and that warn.
const FAIL_PT: f64 = 10.0;
const WARN_PT: f64 = 3.0;
/// Mean sRGB difference (0–255) that warns at the same blackness.
const SRGB_WARN: f64 = 24.0;
/// The second oracle's multiple of the ask (U34).
const STARVE_ORACLE_MULT: u32 = 4;

/// A checkpoint's grade. `Fail` and `Starved` fail it; the word is what the baseline records.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Grade {
    /// The oracle did not run (`--quick`).
    Unjudged,
    Ok,
    Warn,
    /// Starved, and the status bar says so ("iter capped" / "depth limit").
    Capped,
    Fail,
    /// Starved, and the app is silent or claims interior.
    Starved,
}

impl Grade {
    fn word(self) -> &'static str {
        match self {
            Grade::Unjudged => "----",
            Grade::Ok => "ok",
            Grade::Warn => "warn",
            Grade::Capped => "capped",
            Grade::Fail => "FAIL",
            Grade::Starved => "STARVED",
        }
    }
    fn fails(self) -> bool {
        matches!(self, Grade::Fail | Grade::Starved)
    }
}

/// Grade one checkpoint. Excess blackness is the GATE; the sRGB difference only ever warns.
///
/// - `excess_pt`: live minus the oracle at the ask, in points — the live path's own shortfall.
/// - `starve_pt`: live minus the oracle at 4× the ask (`None` = that oracle did not run) — what a
///   larger budget resolves that the screen does not.
/// - `reported`: the status bar names the limit ("iter capped" / "depth limit").
///
/// A live-path shortfall outranks starvation: the frame failed to show what its own ask shows.
///
/// The two black metrics and the colour metric are not equally trustworthy. "The live view resolved
/// less of this view than an offline render did" is unambiguous. A colour difference at the same
/// blackness is not: at a dense deep field the palette maps a huge escape-iteration range into one
/// cycle, so colour is hypersensitive to sub-iteration differences between two independently picked
/// references — the same aliasing that made corpus locations 14/15 look wrong when the rendering
/// was right. Measured at the 6.5e94× hold: live and offline agree to +0.0 points of blackness at
/// the full 4 000 000 iterations the script asked for, and still differ by 28.6 sRGB. Failing that
/// would be calling a correct render broken.
fn grade(truth_ran: bool, excess_pt: f64, starve_pt: Option<f64>, srgb_mean: f64, reported: bool) -> Grade {
    if !truth_ran {
        return Grade::Unjudged;
    }
    let starve = starve_pt.unwrap_or(0.0);
    if excess_pt >= FAIL_PT {
        Grade::Fail
    } else if starve >= FAIL_PT {
        if reported { Grade::Capped } else { Grade::Starved }
    } else if excess_pt >= WARN_PT || starve >= WARN_PT || srgb_mean >= SRGB_WARN {
        Grade::Warn
    } else {
        Grade::Ok
    }
}

/// Does the status-bar diagnosis name a limit the user can see and act on? "iter exhausted" does
/// not count: it concludes the view is (probably) interior, which is exactly what a 4× oracle that
/// resolves the black contradicts.
fn reports_the_limit(app_says: Option<&str>) -> bool {
    app_says.is_some_and(|s| s.contains("iter capped") || s.contains("depth limit"))
}

/// The status bar says "iter capped" — "raise Iterations to resolve it" — where 4× the ask
/// resolves under a tenth of the black it blames on the budget. Context, never a grade (the grade
/// is about the screen): `limit_status`'s "escapes press the ceiling" test reads any view with
/// interior in it at an exact budget as starved, because the escapes just outside the set always
/// finish near whatever ceiling there is (the grand tour's home view: 9.42% black at 2,000,
/// 9.41% at 8,000, and the status bar says "iter capped").
fn false_alarm(app_says: Option<&str>, live_black: f64, starve_pt: Option<f64>) -> bool {
    app_says.is_some_and(|s| s.contains("iter capped")) && starve_pt.is_some_and(|s| s < 0.1 * live_black * 100.0)
}

impl Checkpoint {
    /// Excess blackness over the oracle, in percentage points — the headline number.
    fn excess_black(&self) -> f64 {
        (self.live_black - self.truth_black) * 100.0
    }
    /// Excess blackness over the 4× oracle, when it ran.
    fn starve_pt(&self) -> Option<f64> {
        (self.ask4 > 0).then(|| (self.live_black - self.truth4_black) * 100.0)
    }
    fn grade(&self) -> Grade {
        grade(self.truth_ran, self.excess_black(), self.starve_pt(), self.srgb_mean, reports_the_limit(self.app_says))
    }
    fn verdict(&self) -> &'static str {
        self.grade().word()
    }
    fn false_alarm(&self) -> bool {
        false_alarm(self.app_says, self.live_black, self.starve_pt())
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

/// The smooth-iteration range `(min, max)` of the escaped pixels (channel 0 ≥ 0), or `None` when
/// none escaped. Its max is what the GPU's escape-range reading reports for the frame.
fn escape_range(px: &[f32]) -> Option<(f64, f64)> {
    px.chunks_exact(4)
        .map(|p| p[0])
        .filter(|v| *v >= 0.0 && v.is_finite())
        .fold(None, |acc, v| {
            let v = v as f64;
            Some(acc.map_or((v, v), |(lo, hi): (f64, f64)| (lo.min(v), hi.max(v))))
        })
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
/// ⚠OUTCOME vs CONTEXT. Only what the viewer sees gates the run — verdict, the two black
/// fractions, and the rendered resolution. The counts (budget, appetite, boost, orbit length)
/// are reported alongside a drifting checkpoint because they are what ATTRIBUTES it, but they
/// do not trigger on their own: the reference PICK is not stable run to run in a real-time
/// paced tour, and two runs of one unchanged build produced `orbit 3090 → 20193 PARTIAL` on
/// five shallow checkpoints whose verdict, black fractions and resolution were identical. No
/// tolerance absorbs a 550% swing, and a gate that cries wolf is a gate that gets ignored.
///
/// Nothing is lost: every regression this has actually caught moved an outcome field too — the
/// stale-reprojection regression showed as verdict FAIL→warn, black 42.1%→0.1% AND res
/// 480×270→144×81, and the iteration-boost regression came with its own verdict change.
///
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
    /// The 4× oracle's black fraction, where it ran (absent from baselines blessed before it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    truth4_black: Option<f64>,
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
    crate::harness::selftest::anchored(LIVETEST_BASELINE_DIR)
        .join(format!("livetest-{stem}-{}x{}.json", size[0], size[1]))
}

impl FractadyneApp {
    /// What the status bar would say about view 0 now (`limit_status`'s label), with the escape
    /// maximum measured from the frame under test: the harness renders offscreen, so the GPU's
    /// escape-range reading that feeds the status bar in the GUI never arrives here.
    fn livetest_app_says(&self, eff_iter: u32, esc_max: Option<f64>) -> Option<&'static str> {
        let vc = &self.ref_cache[0];
        Self::limit_status(
            vc.partial,
            vc.orbit_len,
            crate::render::orbit_len_cap(),
            eff_iter,
            self.perf.capped_frac[0],
            self.perf.budget_measured[0],
            self.perf.budget_maxed[0],
            esc_max,
            self.perf.iter_plateau[0],
            self.perf.iter_exhausted[0],
        )
        .map(|s| s.0)
    }

    /// `--livetest FILE`: play a tour in real time through the live pipeline and validate the
    /// frames it puts on screen against offline renders of the same views. Returns the exit code
    /// (`crate::verdict`).
    pub(crate) fn run_livetest(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        tour: &std::path::Path,
        segment: Option<&str>,
        size: [u32; 2],
        out_dir: &std::path::Path,
        quick: bool,
    ) -> i32 {
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
            crate::exit(crate::harness::verdict::VACUOUS);
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
            "  {:>7} {:<16} {:>9} {:>9} {:>8} {:>8} {:>7} {:>7} {:>8} {:>7}  {}",
            "t", "keyframe", "zoom", "live blk", "truth", "4× ask", "Δblk", "sRGB", "iters", "stale", "verdict"
        );
        println!("  {}", "-".repeat(113));

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
                let req = crate::render::params_to_request(&params);
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
                // `render_iter` read the frame back, so every dispatch it encoded has completed —
                // what the GUI learns through its completion callbacks. The harness has no event
                // loop to arm them, so the pinned-refresh pass gate is retired here or every
                // glide's pin waits out `PIN_COMPLETION_TIMEOUT_US` per pass and drifts away.
                self.perf.retire_synchronous_dispatches(0);
                // Maintain the watchdog step budget the way the GUI's event loop does. Without
                // this it stays 0, `build_params` falls back to the pessimistic bootstrap, and
                // every deep frame is shrunk to a fraction of the requested size — the harness
                // would then be measuring a view the GUI never renders. (The GUI keeps this loop
                // in `update`; the harness has no event loop, so it calls the same function.)
                // ⚠This used to be a hand-copy of the arithmetic and had already drifted from it —
                // it re-raised the budget to the opening guess on every reading and skipped the
                // "steps « budget ⇒ the reading priced a different frame" gate. A harness whose
                // controller differs from the app's measures a view the app never renders, which
                // is the one thing it exists to rule out. Call the shared function instead.
                let ms = if ts.captured { ts.iterate_ms } else { gpu_ms };
                let steps = self.perf.fe_steps_last[0];
                if ms > 0.01 && steps > 0 {
                    // ⭐THE APP'S CONTROLLER, not a copy of it (W5's B9; U52). This harness used to
                    // keep its own copy of the budget loop, and a copy drifts: a build gate was once
                    // added to the app and missed here for a commit — the livetest then reported 0
                    // drift on a path that did not contain the change at all — and by W5 the copy
                    // still lacked the reading pool, the short-reading and dead-man clamps and the
                    // lethal-band retreat. `apply_iterate_measurement` is exactly what the GUI's
                    // event loop calls with a reading. Only a real timestamp is a GPU reading:
                    // `gpu_ms` is the harness's wall clock, priced as the app prices one.
                    let before = self.perf.fe_budget[0];
                    let src = if ts.captured { Self::SRC_GPU_ITERATE } else { "wall_iterate" };
                    let priced = self.apply_iterate_measurement(0, ms, steps, src, crate::ReadShape::default());
                    // The harness's budget walk is otherwise invisible; the 16×16 deep-hold
                    // regression was undiagnosable without frame-level sight of it.
                    if crate::diag::trace_on("gpu") {
                        crate::diag::trace(
                            "gpu",
                            format!(
                                "lt e={e:.1} res={}x{} iter={} steps={:.2e} ms={ms:.1} src={src} \
                                 budget {:.2e} -> {:.2e} priced={priced} tiling={} tile={:?}",
                                params.resolution[0],
                                params.resolution[1],
                                params.max_iter,
                                steps as f64,
                                before as f64,
                                self.perf.fe_budget[0] as f64,
                                self.perf.tile_state[0].is_some(),
                                params.tile,
                            ),
                        );
                    }
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
                // ⭐e72 (TODO.md): the reference cap a hold builds under is chosen by `interacting`,
                // so record what the checkpoint's own frame reported — a hold sampled while still
                // "interacting" is building references under the MOTION cap.
                if crate::diag::trace_on("ref") {
                    crate::diag::trace(
                        "ref",
                        format!(
                            "checkpoint {id} @{t_cp:.1}: interacting={interacting} \
                             since_settle={:.2}s orbit_len={} partial={}",
                            now - self.pointer.settle_t[0],
                            self.ref_cache[0].orbit_len,
                            self.ref_cache[0].partial,
                        ),
                    );
                }
                let Some((live_px, live_res, real_at)) = last_real.clone() else { continue };
                let stale_ms = (t_base.elapsed().as_secs_f64() - real_at) * 1000.0;
                let live_black = interior_frac(&live_px);
                let live_esc = escape_range(&live_px);
                // The app's own account, read BEFORE the oracles run: they build references and
                // render, and nothing they do may be mistaken for the live path's state.
                let capped_frac = self.perf.capped_frac[0];
                let (iter_exhausted, iter_plateau, budget_maxed) =
                    (self.perf.iter_exhausted[0], self.perf.iter_plateau[0], self.perf.budget_maxed[0]);
                let app_says = self.livetest_app_says(params.max_iter, live_esc.map(|r| r.1));

                // ---- oracle: the same view, same iteration budget, through the offline path ----
                // The tour clock must not run while this happens (a deep truth render can take
                // minutes), or the tour would teleport forward mid-validation.
                let pause = Instant::now();
                let (mut truth_black, mut srgb_mean, mut srgb_max, mut truth_ran) =
                    (0.0, 0.0, 0u32, false);
                let (mut ask, mut ask4, mut truth4_black, mut truth_range) = (0u32, 0u32, 0.0, 0.0);
                if !quick {
                    let saved_w = self.export.width;
                    let saved_ss = self.export.ss;
                    self.export.width = live_res[0];
                    self.export.ss = 1;
                    let vp = self.viewport.clone();
                    let mut req = self.current_export_request_for(&vp, false);
                    // The live buffer's height comes from the res-shrink's TRUNCATION
                    // (`(panel_h × f) as u32`); the export request re-derives height by ROUNDING
                    // the viewport aspect. At a native capture the two agree, but a hold captured
                    // mid-climb at shrunk res can differ by one row (298×167 live vs 298×168
                    // request) — and coloring the live buffer with the request's dims overruns
                    // `write_texture` with an uncaptured validation error that kills the whole
                    // run (seen at hold-e61/e72 on two builds). The buffer under comparison is
                    // the authority on its own dimensions.
                    req.width = live_res[0];
                    req.height = live_res[1];
                    ask = req.max_iter;
                    if let Ok(t) = fractadyne_gpu::render_iter(device, queue, &req) {
                        truth_black = interior_frac(&t.pixels);
                        truth_range = escape_range(&t.pixels).map_or(0.0, |(lo, hi)| hi - lo);
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
                        // ---- second oracle: the same view at 4× the ask (U34) ----
                        // Only where it can change the grade: starvation is live black that a
                        // bigger budget resolves, so a frame under `WARN_PT` black has none to
                        // show. The count is honoured verbatim (auto-iter off for the build), with
                        // a reference built for it — a reference built for the ask would clamp
                        // the longer pixels to its own length.
                        let want4 = ask.saturating_mul(STARVE_ORACLE_MULT).min(crate::MAX_ITER_LIMIT);
                        let mut truth4: Option<(Vec<f32>, fractadyne_gpu::ExportRequest)> = None;
                        if live_black * 100.0 >= WARN_PT && want4 > ask {
                            let saved = (self.render_cfg.auto_iter, self.render_cfg.max_iter);
                            self.render_cfg.auto_iter = false;
                            self.render_cfg.max_iter = want4;
                            let mut req4 = self.current_export_request_for(&vp, false);
                            (self.render_cfg.auto_iter, self.render_cfg.max_iter) = saved;
                            req4.width = live_res[0];
                            req4.height = live_res[1];
                            if let Ok(t4) = fractadyne_gpu::render_iter(device, queue, &req4) {
                                ask4 = req4.max_iter;
                                truth4_black = interior_frac(&t4.pixels);
                                truth4 = Some((t4.pixels, req4));
                            }
                        }
                        // Dump the images whenever the checkpoint fails — a number says a frame
                        // is wrong, an image says how.
                        let starve_pt = (ask4 > 0).then(|| (live_black - truth4_black) * 100.0);
                        let g = grade(
                            true,
                            (live_black - truth_black) * 100.0,
                            starve_pt,
                            srgb_mean,
                            reports_the_limit(app_says),
                        );
                        if g.fails() {
                            let _ = std::fs::create_dir_all(out_dir);
                            let stem = format!("livetest_{:03}_{id}", results.len());
                            let mut dumps: Vec<(&str, &[f32], &fractadyne_gpu::ExportRequest)> =
                                vec![("live", live_px.as_slice(), &req), ("truth", t.pixels.as_slice(), &req)];
                            if let Some((px4, r4)) = &truth4 {
                                dumps.push(("truth4x", px4.as_slice(), r4));
                            }
                            for (tag, buf, r) in dumps {
                                if let Ok(c) = fractadyne_gpu::color_iter_buffer(device, queue, r, buf) {
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
                    ask,
                    ask4,
                    truth4_black,
                    capped_frac,
                    iter_exhausted,
                    iter_plateau,
                    budget_maxed,
                    app_says,
                    live_range: live_esc.map_or(0.0, |(lo, hi)| hi - lo),
                    truth_range,
                };
                println!(
                    "  {:>6.1}s {:<16} {:>9} {:>8.1}% {:>8.1}% {:>8} {:>+6.1}pt {:>7.1} {:>8} {:>6.0}ms  {}",
                    cp.t,
                    cp.id,
                    format!("1e{:.0}", cp.log10mag),
                    cp.live_black * 100.0,
                    cp.truth_black * 100.0,
                    if cp.ask4 > 0 { format!("{:.1}%", cp.truth4_black * 100.0) } else { "-".to_string() },
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
        let count = |g: Grade| results.iter().filter(|c| c.grade() == g).count();
        let (warns, capped, starved) = (count(Grade::Warn), count(Grade::Capped), count(Grade::Starved));
        let fails = count(Grade::Fail) + starved;
        // Checkpoints in the window the tour never captured: nothing was judged there.
        let missed: Vec<String> = pending.iter().rev().map(|(_, id)| id.clone()).collect();
        println!();
        for c in &results {
            if matches!(c.grade(), Grade::Ok | Grade::Unjudged) {
                continue;
            }
            let num = |v: u32| crate::commas(&v.to_string());
            let at4 = if c.ask4 > 0 {
                format!(" and {:.1}% at 4× ({})", c.truth4_black * 100.0, num(c.ask4))
            } else {
                String::new()
            };
            let account = format!(
                "the app says {} (capped {}, plateau {}, exhausted {}, maxed {})",
                c.app_says.map_or("nothing".to_string(), |s| format!("\"{s}\"")),
                c.capped_frac.map_or("none".to_string(), |f| format!("{:.1}%", f * 100.0)),
                c.iter_plateau,
                c.iter_exhausted,
                c.budget_maxed,
            );
            println!(
                concat!(
                    "  {} {} at t={:.1}s (1e{:.0}×): live {:.1}% black vs {:.1}% at the ask ({}){}; {}; ",
                    "budget {} (appetite {}, boost ×{:.2}), orbit {}{}, {}×{}, {:.0}ms stale, sRGB Δ {:.1}/{}, ",
                    "escape range live {:.0} / offline {:.0}"
                ),
                c.verdict(),
                c.id,
                c.t,
                c.log10mag,
                c.live_black * 100.0,
                c.truth_black * 100.0,
                num(c.ask),
                at4,
                account,
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
                c.live_range,
                c.truth_range,
            );
        }
        let opt = |v: Option<f64>| v.map_or("null".to_string(), |f| format!("{f:.5}"));
        let json: String = results
            .iter()
            .map(|c| {
                format!(
                    concat!(
                        "  {{\"t\":{:.2},\"id\":\"{}\",\"log10mag\":{:.3},\"live_black\":{:.5},",
                        "\"truth_black\":{:.5},\"excess_black_pt\":{:.3},\"srgb_mean\":{:.3},",
                        "\"srgb_max\":{},\"gpu_iter\":{},\"eff_iter\":{},\"boost\":{:.4},",
                        "\"orbit_len\":{},\"partial\":{},\"res\":[{},{}],\"stale_ms\":{:.1},",
                        "\"ask\":{},\"ask4\":{},\"truth4_black\":{},\"starve_pt\":{},",
                        "\"capped_frac\":{},\"iter_exhausted\":{},\"iter_plateau\":{},\"budget_maxed\":{},",
                        "\"app_says\":{},\"false_alarm\":{},\"live_range\":{:.1},\"truth_range\":{:.1},",
                        "\"verdict\":\"{}\"}}"
                    ),
                    c.t, c.id, c.log10mag, c.live_black, c.truth_black, c.excess_black(),
                    c.srgb_mean, c.srgb_max, c.gpu_iter, c.eff_iter, c.boost, c.orbit_len,
                    c.partial, c.res[0], c.res[1], c.stale_ms,
                    c.ask, c.ask4, opt((c.ask4 > 0).then_some(c.truth4_black)), opt(c.starve_pt()),
                    opt(c.capped_frac), c.iter_exhausted, c.iter_plateau, c.budget_maxed,
                    c.app_says.map_or("null".to_string(), |s| format!("{s:?}")), c.false_alarm(),
                    c.live_range, c.truth_range,
                    c.verdict(),
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
            "\n{} checkpoint(s): {} ok, {warns} warn, {capped} capped, {} FAIL, {starved} STARVED{}\nlog → {}",
            results.len(),
            count(Grade::Ok),
            fails - starved,
            if fail_dumps > 0 { format!(" ({fail_dumps} failing checkpoint(s)' images dumped to {})", out_dir.display()) } else { String::new() },
            path.display(),
        );
        if fails > starved {
            println!(
                "\nA FAIL means the live view showed materially less than an offline render of the\n\
                 SAME view at the SAME iteration budget. Read the context: `PARTIAL` orbit ⇒ the\n\
                 pixel clamp; budget « appetite ⇒ the live cap or the adaptive boost never climbed;\n\
                 large `stale` ⇒ the screen is showing an old frame reprojected."
            );
        }
        if starved > 0 {
            println!(concat!(
                "\nA STARVED means an offline render at 4× the iteration ask resolves materially more ",
                "of the view than the screen showed, and the app does not tell the user why: it says ",
                "nothing (a budget still climbing at the end of the hold, or no settled reading), or ",
                "it concluded interior (plateau, exhausted) and the 4× render shows otherwise. Where the ",
                "status bar names the limit (\"iter capped\", \"depth limit\") the grade is `capped`."
            ));
        }
        let alarms: Vec<&str> = results.iter().filter(|c| c.false_alarm()).map(|c| c.id.as_str()).collect();
        if !alarms.is_empty() {
            println!(
                concat!(
                    "\n{} false alarm(s) (context, not graded): the status bar says \"iter capped\" where 4× the ask ",
                    "resolves under a tenth of the black: {}"
                ),
                alarms.len(),
                alarms.join(", ")
            );
        }
        if !missed.is_empty() {
            println!(
                "\n{} checkpoint(s) in the window never captured (the tour ended first): {}",
                missed.len(),
                missed.join(", ")
            );
        }
        let vacuous = results.is_empty() || !missed.is_empty();
        let code = if quick {
            // The oracle is off, so every grade is `----`: nothing was judged, and a baseline
            // blessed from it would record nothing either.
            println!("\n--quick: the oracle is off, so nothing was judged (not compared, not blessed).");
            crate::harness::verdict::VACUOUS
        } else if self.selftest.bless {
            if self.bless_livetest(&results, tour, resolution) == 0 { crate::harness::verdict::PASS } else { crate::harness::verdict::RED }
        } else {
            let drift = self.compare_livetest(&results, tour, resolution, fails);
            crate::harness::verdict::code(drift > 0, vacuous)
        };
        println!("--livetest: VERDICT {}", crate::harness::verdict::word(code));
        code
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
                            truth4_black: (c.ask4 > 0).then_some(c.truth4_black),
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
            let mut ctx: Vec<String> = Vec::new();
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
            // The 4× oracle, where both runs have it (a baseline blessed before it has none; the
            // grade it produces is compared through `verdict` either way).
            if let (Some(b4), true) = (b.truth4_black, c.ask4 > 0) {
                if (c.truth4_black - b4).abs() > BLACK_TOL {
                    d.push(format!("4× black {:.1}% → {:.1}%", b4 * 100.0, c.truth4_black * 100.0));
                }
            }
            if count_drifted(b.gpu_iter, c.gpu_iter) {
                ctx.push(format!("budget {} → {}", b.gpu_iter, c.gpu_iter));
            }
            if count_drifted(b.eff_iter, c.eff_iter) {
                ctx.push(format!("appetite {} → {}", b.eff_iter, c.eff_iter));
            }
            if (c.boost - b.boost).abs() > 1.0e-6 {
                ctx.push(format!("boost ×{:.2} → ×{:.2}", b.boost, c.boost));
            }
            if count_drifted(b.orbit_len, c.orbit_len) || c.partial != b.partial {
                ctx.push(format!(
                    "orbit {}{} → {}{}",
                    b.orbit_len,
                    if b.partial { " PARTIAL" } else { "" },
                    c.orbit_len,
                    if c.partial { " PARTIAL" } else { "" }
                ));
            }
            // Resolution ATTRIBUTES, it does not GATE (demoted 2026-08-09 after flapping the
            // gate four times in one day). A checkpoint can legitimately capture mid budget
            // re-climb after an install derate — `seahorse-2` recorded 480×270, 100×56 and
            // 150×84 across runs of IDENTICAL builds with verdict and both black fractions
            // unchanged, and no tolerance absorbs that. Nothing real is lost: every regression
            // this harness has actually caught moved verdict or blackness too (the beta.47
            // stale-reprojection showed verdict FAIL→warn AND black 42.1%→0.1%; the A2
            // resolution collapse showed at every deep checkpoint's verdict). A res change with
            // NO outcome change is a capture-timing artifact, so it is reported as context.
            if c.res != b.res {
                ctx.push(format!(
                    "res {}×{} → {}×{}",
                    b.res[0], b.res[1], c.res[0], c.res[1]
                ));
            }
            if !d.is_empty() {
                drift += 1;
                let why = if ctx.is_empty() {
                    String::new()
                } else {
                    format!("   [context: {}]", ctx.join("; "))
                };
                println!("  ✗ {:<16} {}{}", c.id, d.join("; "), why);
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
            // (Any FAILs recorded in the baseline itself are expected states, not news — though
            // since beta.50 the blessed grand-tour baseline carries none.)
            println!("No live-path drift vs the blessed baseline.");
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

#[cfg(test)]
#[path = "../../tests/unit/harness/livetest/tests.rs"]
mod tests;
