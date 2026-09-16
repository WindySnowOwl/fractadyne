//! `--zoomtest [OCTAVES]` — on-screen update-latency harness for LIVE zooms.
//!
//! WHY THIS EXISTS: "the zoom feels jerky" is a statement about the interval between the frames
//! a person actually SEES, and none of the other harnesses measures that. `--divetest` is
//! headless — its cadence columns sit on a synthetic clock reconstructed from frame costs —
//! `FRACTADYNE_PERF=1` records tour-playback frames only, and `--motiontest` runs the real window
//! but asserts presentation invariants, not timing. This one runs the real windowed app, holds a
//! virtual Space key (the production glide, pacer and interactive reference lookahead included —
//! `harness_holds_space`), and stamps every `update()` with the wall-clock interval since the
//! previous one: that interval IS the on-screen update cadence, because the glide requests a
//! repaint every frame and each `update()` ends in a present. Per frame it also records what the
//! frame showed (a real re-iterate or a held reprojection, and how many octaves old the held
//! frame was), the reference-pipeline state (depth lag, installs, lookahead installs, pacer
//! throttle) and the per-frame zoom step — a long frame followed by a big step is the jerk a
//! viewer feels. Everything goes to a JSON file (`--out`, else `logs/zoomtest-<stamp>.json`) with
//! a summary — mean / p50 / p95 / p99 / max interval, hitch counts, the longest stall and where
//! it happened, real-refresh cadence, held-frame magnification, zoom-step spread — so runs across
//! builds can be diffed and a stutter attributed.
//!
//! Options: `--zoomtest [OCTAVES]` (default 40; 1e100 from 1× is 332.2), `--zoomtest-rate R`
//! (the Zoom speed slider, default 1.0), `--zoomtest-location FILE.fdn` (start view; default
//! corpus location 07 at 2^103.3 ≈ 1.3e31×, the same structure-rich deep centre `--motiontest`
//! drives, so the glide runs in the floatexp regime from its first frame), and
//! `--zoomtest-start-log2 L` (override the start magnification to 2^L while keeping the file's
//! centre — `0` starts at 1× on a deep location's centre, so the whole descent to it is measured). Exit codes follow the torture `classify`
//! contract: 0 = measured and reported, 2 = never reached the regime (fewer than `MIN_FRAMES`
//! frames, or the start reference never built), 4 = the watchdog (frame loop stopped).
//! Run it with a wiped `FRACTADYNE_CONFIG_DIR` like every other gate; `FRACTADYNE_NO_PREFETCH=1`
//! is the A/B without the interactive lookahead.

use std::time::Instant;

use crate::FractadyneApp;

/// Wall-clock phase budgets. The reference build is the only genuinely open-ended wait.
const WAIT_REF_S: f64 = 180.0;
/// After the start reference lands, let the view settle before the glide begins, so the run does
/// not start on a mid-settle grid (which would be measured as a stall it did not cause).
const QUIET_S: f64 = 2.0;
/// Cap on the glide itself — a 1× → 1e100 dive (332 octaves) at the 1.0× slider is ~500 s.
const GLIDE_MAX_S: f64 = 900.0;
/// Whole-run hard deadline, enforced by a watchdog THREAD (the `--autodive` lesson: an in-frame
/// check needs frames to still be delivered, which is exactly not the case that needs bounding).
const DEADLINE_S: f64 = 1200.0;
/// A glide that produced fewer frames than this measured nothing.
const MIN_FRAMES: usize = 120;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Jump,
    WaitRef,
    Quiet,
    Glide,
}

/// One presented frame of the glide. `dt_ms` is the wall interval since the previous `update()`
/// (the on-screen cadence); `real` says whether the frame just shown was a real re-iterate
/// (`perf.prev_real`, stamped when its params were built) or a held reprojection.
struct Frame {
    t: f64,
    dt_ms: f64,
    l2: f64,
    dl2: f64,
    real: bool,
    /// Octaves the frame ON SCREEN lags the view (`l2 − frozen_l2`; the display magnifies the last
    /// complete frame by 2^gap). This holds on `real` frames too: a PINNED refresh (mode 2, from the
    /// floatexp hand-over at ~1e28) re-iterates every frame into the G-buffer but the display keeps
    /// serving the hold snapshot of the last complete frame until the pin adopts, and a reference
    /// install aborts the pin (`PinStop::Orbit`). At 4.0× the lookahead installs every ~0.2 s, so
    /// from 1e28 to ~1e29.5 nothing complete lands and the hand-over frame is shown at 24–53×
    /// (measured 4.5–5.7 oct, 2026-09-16) — the biggest visible defect of a fast 1× → 1e100 dive.
    gap_oct: f64,
    lag: f64,
    orbit_id: u64,
    look: u64,
    /// Share of the selected zoom speed the pacer let through this frame.
    vel_frac: f64,
    res: f64,
    mode: u32,
    /// The app's own interval measure (`perf.last_dt_ms`, includes present waits).
    gui_dt_ms: f64,
}

pub(crate) struct ZoomTest {
    octaves: f64,
    rate: f32,
    location: Option<std::path::PathBuf>,
    /// Start magnification override (log2), keeping the location's centre.
    start_log2: Option<f64>,
    phase: Phase,
    t0: Instant,
    phase_t0: Instant,
    frames_total: u64,
    last_frame: Option<Instant>,
    start_l2: f64,
    prev_l2: f64,
    records: Vec<Frame>,
    /// True while the harness holds the virtual Space key — read by `draw_central` and the
    /// interactive lookahead pump exactly where they read the real key.
    driving: bool,
    /// Cleared by the report path so the watchdog thread stands down.
    done: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl ZoomTest {
    pub(crate) fn new(
        octaves: f64,
        rate: f32,
        location: Option<std::path::PathBuf>,
        start_log2: Option<f64>,
    ) -> Self {
        let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        {
            let done = std::sync::Arc::clone(&done);
            std::thread::spawn(move || {
                let start = Instant::now();
                while start.elapsed().as_secs_f64() < DEADLINE_S {
                    if done.load(std::sync::atomic::Ordering::Relaxed) {
                        return;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(250));
                }
                if done.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }
                let msg = format!(
                    "WATCHDOG: no report within {DEADLINE_S:.0}s — the frame loop stopped delivering (deep reference build on the main thread, or a wedge). Nothing was measured."
                );
                crate::diag::log_line("zoomtest", &msg);
                eprintln!();
                eprintln!("--zoomtest: {msg}");
                crate::exit(4);
            });
        }
        Self {
            octaves: octaves.max(1.0),
            rate: rate.clamp(0.25, 4.0),
            location,
            start_log2,
            phase: Phase::Jump,
            t0: Instant::now(),
            phase_t0: Instant::now(),
            frames_total: 0,
            last_frame: None,
            start_l2: 0.0,
            prev_l2: 0.0,
            records: Vec::with_capacity(8192),
            driving: false,
            done,
        }
    }
}

/// Nearest-rank percentile over a SORTED slice (no interpolation — matches `--divetest`).
fn pct(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    sorted[((sorted.len() as f64 * q) as usize).min(sorted.len() - 1)]
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

impl FractadyneApp {
    /// The harness's virtual Space key: `draw_central` and the interactive lookahead pump read it
    /// exactly where they read the real key, so the driven glide eases in, runs and eases out
    /// through the production path — pacer and prefetch included.
    pub(crate) fn harness_holds_space(&self) -> bool {
        self.harness.zoomtest.as_ref().is_some_and(|z| z.driving)
    }

    /// One step per frame, called from `update()` BEFORE the lookahead pump and the central draw
    /// (same in-loop shape as `motiontest_frame`), so the key state it sets shapes this frame and
    /// the interval it records belongs to the frame just presented.
    pub(crate) fn zoomtest_frame(&mut self, ctx: &egui::Context) {
        let Some(mut zt) = self.harness.zoomtest.take() else { return };
        zt.frames_total += 1;
        let now = Instant::now();
        let in_phase = zt.phase_t0.elapsed().as_secs_f64();

        let advance = |zt: &mut ZoomTest, p: Phase| {
            crate::diag::log_line(
                "zoomtest",
                &format!("phase {:?} -> {:?} at +{:.1}s", zt.phase, p, zt.t0.elapsed().as_secs_f64()),
            );
            zt.phase = p;
            zt.phase_t0 = Instant::now();
        };

        match zt.phase {
            Phase::Jump => {
                // The start view, in-process. A `.fdn` when given (the same allow-listed loader the
                // Open dialog and `--shot` use), else the corpus-07 deep centre `--motiontest` drives.
                match zt.location.clone() {
                    Some(loc) => match std::fs::read_to_string(&loc) {
                        Ok(text) => {
                            let load = self.load_view_metadata(&text);
                            if !load.clamped.is_empty() {
                                eprintln!("--zoomtest: clamped {}", load.clamped.join(", "));
                            }
                        }
                        Err(e) => {
                            eprintln!("fractadyne: --zoomtest: cannot read {}: {e}", loc.display());
                            self.zoomtest_abort(zt, "start location unreadable");
                        }
                    },
                    None => {
                        let cx = fractadyne_core::parse_bf_prec(crate::motiontest::CX, 192);
                        let cy = fractadyne_core::parse_bf_prec(crate::motiontest::CY, 192);
                        let (Some(cx), Some(cy)) = (cx, cy) else {
                            self.zoomtest_abort(zt, "internal: corpus centre failed to parse");
                        };
                        self.viewport.set_center_log2mag(cx, cy, crate::motiontest::LOG2_MAG);
                    }
                }
                if let Some(l2) = zt.start_log2 {
                    // Keep the location's centre (the TARGET of a centred glide) and start the
                    // descent at 2^l2 — `0` = the 1× home framing of that point.
                    let (cx, cy) = (self.viewport.center_x.clone(), self.viewport.center_y.clone());
                    self.viewport.set_center_log2mag(cx, cy, l2);
                }
                self.render_cfg.zoom_rate = zt.rate;
                self.pointer.zoom_vel = 0.0;
                self.pointer.last_cursor = None; // a centred glide, like a user with the mouse parked
                self.invalidate_refs();
                crate::diag::log_line(
                    "zoomtest",
                    &format!(
                        "start 2^{:.1} · {} octaves at zoom rate {:.2}x ({:.2} oct/s) · prefetch {}",
                        self.viewport.log2_magnification(),
                        zt.octaves,
                        zt.rate,
                        crate::ZOOM_RATE * zt.rate as f64 / std::f64::consts::LN_2,
                        if crate::render::no_prefetch() { "OFF (FRACTADYNE_NO_PREFETCH)" } else { "on" }
                    ),
                );
                advance(&mut zt, Phase::WaitRef);
            }
            Phase::WaitRef => {
                // A start at 1× renders DIRECT — no reference is ever built there, so requiring
                // one would wait forever (it did: 180 s aborts on the first 1× → 1e100 runs).
                // Demand a reference only where the start view is a perturbation view.
                let needs_ref = !crate::RenderMode::select(
                    self.fractal.supports_perturbation(),
                    false,
                    self.viewport.magnification(),
                )
                .is_direct();
                // Likewise the frozen-frame latch belongs to the perturbation freeze machinery
                // (`render.rs`, the reuse-hold/pin block) and is never written for a direct view.
                let rc = &self.ref_cache[0];
                let ready = (!needs_ref || (rc.ref_pt.is_some() && rc.frozen_center.is_some()))
                    && self.recompute_rx[0].is_none()
                    && zt.frames_total > 30;
                if ready {
                    crate::diag::log_line(
                        "zoomtest",
                        &format!("reference ready (orbit_len={} partial={}) — settling", rc.orbit_len, rc.partial),
                    );
                    advance(&mut zt, Phase::Quiet);
                } else if in_phase > WAIT_REF_S {
                    self.zoomtest_abort(zt, "boot never completed: no start reference");
                }
            }
            Phase::Quiet => {
                if in_phase > QUIET_S {
                    zt.start_l2 = self.viewport.log2_magnification();
                    zt.prev_l2 = zt.start_l2;
                    zt.last_frame = None;
                    zt.driving = true; // the virtual key goes down THIS frame
                    advance(&mut zt, Phase::Glide);
                }
            }
            Phase::Glide => {
                let l2 = self.viewport.log2_magnification();
                // The interval that ends now belongs to the frame just presented — the one whose
                // params `prev_real` describes and whose zoom step is l2 − prev_l2.
                if let Some(prev) = zt.last_frame {
                    let dt_ms = now.duration_since(prev).as_secs_f64() * 1000.0;
                    let rc = &self.ref_cache[0];
                    let gap_oct = if rc.frozen_center.is_some() { (l2 - rc.frozen_l2).max(0.0) } else { 0.0 };
                    let v = self.pointer.zoom_vel;
                    let vel_frac = if v > 1e-6 { self.paced_zoom_vel() / v } else { 1.0 };
                    zt.records.push(Frame {
                        t: zt.phase_t0.elapsed().as_secs_f64(),
                        dt_ms,
                        l2,
                        dl2: l2 - zt.prev_l2,
                        real: self.perf.prev_real[0],
                        gap_oct,
                        lag: rc.last_depth_lag,
                        orbit_id: rc.orbit_id,
                        look: self.perf.lookahead_installs,
                        vel_frac,
                        res: self.perf.motion_res,
                        mode: self.perf.last_mode,
                        gui_dt_ms: self.perf.last_dt_ms,
                    });
                }
                zt.last_frame = Some(now);
                zt.prev_l2 = l2;
                if l2 - zt.start_l2 >= zt.octaves || in_phase > GLIDE_MAX_S {
                    zt.driving = false;
                    self.zoomtest_report(zt); // exits
                }
            }
        }

        self.harness.zoomtest = Some(zt);
        ctx.request_repaint();
    }

    fn zoomtest_abort(&mut self, zt: ZoomTest, msg: &str) -> ! {
        zt.done.store(true, std::sync::atomic::Ordering::Relaxed);
        crate::diag::log_line("zoomtest", &format!("ABORT: {msg}"));
        eprintln!("--zoomtest: ABORT: {msg}");
        crate::exit(2);
    }

    /// Summarise, write the JSON, print the table, exit. 2 (never a pass) when the glide produced
    /// too few frames to mean anything.
    fn zoomtest_report(&mut self, zt: ZoomTest) -> ! {
        zt.done.store(true, std::sync::atomic::Ordering::Relaxed);
        let f = &zt.records;
        let n = f.len();
        if n < MIN_FRAMES {
            eprintln!("--zoomtest: VACUOUS: only {n} glide frames (need >= {MIN_FRAMES}) — nothing measured");
            crate::exit(2);
        }
        let secs = f.last().map(|r| r.t).unwrap_or(0.0) - f.first().map(|r| r.t).unwrap_or(0.0);
        let oct = f.last().unwrap().l2 - zt.start_l2;
        let mut dts: Vec<f64> = f.iter().map(|r| r.dt_ms).collect();
        dts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let dt_mean = dts.iter().sum::<f64>() / n as f64;
        let (dt_p50, dt_p95, dt_p99, dt_max) = (pct(&dts, 0.5), pct(&dts, 0.95), pct(&dts, 0.99), *dts.last().unwrap());
        let gt = |ms: f64| f.iter().filter(|r| r.dt_ms > ms).count();
        let (gt33, gt50, gt100) = (gt(33.0), gt(50.0), gt(100.0));
        let longest = f.iter().max_by(|a, b| a.dt_ms.partial_cmp(&b.dt_ms).unwrap()).unwrap();
        // Real-refresh cadence: wall time from one real frame's present to the next.
        let mut gaps: Vec<f64> = Vec::new();
        let mut acc = 0.0;
        let mut seen = false;
        for r in f {
            if r.real {
                if seen {
                    gaps.push(acc);
                }
                seen = true;
                acc = 0.0;
            }
            acc += r.dt_ms;
        }
        gaps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let reals = f.iter().filter(|r| r.real).count();
        let gap_mean = if gaps.is_empty() { 0.0 } else { gaps.iter().sum::<f64>() / gaps.len() as f64 };
        let (gap_p95, gap_max) = (pct(&gaps, 0.95), gaps.last().copied().unwrap_or(0.0));
        let mut steps: Vec<f64> = f.iter().map(|r| r.dl2.max(0.0)).collect();
        steps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let (step_p95, step_max) = (pct(&steps, 0.95), steps.last().copied().unwrap_or(0.0));
        let gap_oct_max = f.iter().map(|r| r.gap_oct).fold(0.0, f64::max);
        let lag_max = f.iter().map(|r| r.lag).fold(0.0, f64::max);
        let installs = f.last().unwrap().orbit_id.saturating_sub(f.first().unwrap().orbit_id);
        let look = f.last().unwrap().look.saturating_sub(f.first().unwrap().look);
        let paced_pct = f.iter().filter(|r| r.vel_frac < 0.999).count() as f64 / n as f64 * 100.0;
        let hold_pct = f.iter().filter(|r| !r.real).count() as f64 / n as f64 * 100.0;
        let fps = n as f64 / secs.max(1e-9);
        let oct_s = oct / secs.max(1e-9);

        eprintln!();
        eprintln!(
            "--zoomtest: {oct:.1} octaves in {secs:.1} s ({oct_s:.2} oct/s, {fps:.1} fps) · {n} frames · zoom rate {:.2}x · prefetch {}",
            zt.rate,
            if crate::render::no_prefetch() { "off" } else { "on" }
        );
        eprintln!(
            "  frame interval ms: mean {dt_mean:.1}  p50 {dt_p50:.1}  p95 {dt_p95:.1}  p99 {dt_p99:.1}  max {dt_max:.1}   >33ms {gt33}  >50ms {gt50}  >100ms {gt100}"
        );
        eprintln!(
            "  longest stall {:.0} ms at t={:.1} s (1e{:.1}x) · zoom step per frame p95 {step_p95:.4} oct  max {step_max:.3} oct",
            longest.dt_ms,
            longest.t,
            longest.l2 / std::f64::consts::LOG2_10
        );
        eprintln!(
            "  real refreshes {reals} ({:.1}/s), held frames {hold_pct:.0}% · real interval ms: mean {gap_mean:.0}  p95 {gap_p95:.0}  max {gap_max:.0} · held frame max {gap_oct_max:.2} oct ({:.2}x)",
            reals as f64 / secs.max(1e-9),
            gap_oct_max.exp2()
        );
        eprintln!(
            "  reference installs {installs} (lookahead {look}) · depth lag max {lag_max:.2} · pacer throttled {paced_pct:.1}% of frames"
        );

        let mut rows = String::with_capacity(n * 160);
        for (i, r) in f.iter().enumerate() {
            if i > 0 {
                rows.push_str(",\n");
            }
            rows.push_str(&format!(
                "{{\"t\":{:.4},\"dt_ms\":{:.3},\"gui_dt_ms\":{:.3},\"l2\":{:.4},\"dl2\":{:.5},\"real\":{},\"gap_oct\":{:.4},\"lag\":{:.3},\"orbit_id\":{},\"look\":{},\"vel_frac\":{:.3},\"res\":{:.3},\"mode\":{}}}",
                r.t, r.dt_ms, r.gui_dt_ms, r.l2, r.dl2, r.real, r.gap_oct, r.lag, r.orbit_id, r.look, r.vel_frac, r.res, r.mode
            ));
        }
        let json = format!(
            "{{\n\"tool\": \"fractadyne --zoomtest\",\n\"version\": {},\n\"location\": {},\n\"zoom_rate\": {:.3},\n\"prefetch\": {},\n\"start_l2\": {:.4},\n\"octaves\": {oct:.4},\n\"secs\": {secs:.3},\n\"summary\": {{\"frames\":{n},\"fps\":{fps:.2},\"oct_s\":{oct_s:.4},\"dt_mean\":{dt_mean:.3},\"dt_p50\":{dt_p50:.3},\"dt_p95\":{dt_p95:.3},\"dt_p99\":{dt_p99:.3},\"dt_max\":{dt_max:.3},\"gt33\":{gt33},\"gt50\":{gt50},\"gt100\":{gt100},\"longest_ms\":{:.3},\"longest_t\":{:.3},\"longest_l2\":{:.3},\"reals\":{reals},\"hold_pct\":{hold_pct:.2},\"real_gap_mean\":{gap_mean:.3},\"real_gap_p95\":{gap_p95:.3},\"real_gap_max\":{gap_max:.3},\"step_p95\":{step_p95:.5},\"step_max\":{step_max:.5},\"gap_oct_max\":{gap_oct_max:.4},\"lag_max\":{lag_max:.3},\"installs\":{installs},\"lookahead\":{look},\"paced_pct\":{paced_pct:.2}}},\n\"frames\": [\n{rows}\n]\n}}\n",
            json_str(&crate::version_string()),
            json_str(&zt.location.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "corpus-07 @ 2^103.3".into())),
            zt.rate,
            !crate::render::no_prefetch(),
            zt.start_l2,
            longest.dt_ms,
            longest.t,
            longest.l2,
        );
        let secs_now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let out = self.profile.out.clone().unwrap_or_else(|| {
            std::path::PathBuf::from(format!("logs/zoomtest-{}.json", Self::file_stamp(secs_now)))
        });
        if let Some(dir) = out.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        match std::fs::write(&out, &json) {
            Ok(()) => eprintln!("  zoomtest log → {}", out.display()),
            Err(e) => eprintln!("  zoomtest log write failed: {e}"),
        }
        crate::diag::log_line(
            "zoomtest",
            &format!("report: {n} frames, dt mean {dt_mean:.1} p95 {dt_p95:.1} max {dt_max:.1} ms, >33ms {gt33}, installs {installs} (lookahead {look})"),
        );
        crate::exit(0);
    }
}
