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

/// Cap on the settle tail (`FRACTADYNE_ZOOMTEST_SETTLE`): the on-settle supersampling at a deep
/// view is 24 samples of a full re-render each, seconds apiece at depth.
const SETTLE_MAX_S: f64 = 120.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Jump,
    WaitRef,
    Quiet,
    Glide,
    /// `FRACTADYNE_ZOOMTEST_SETTLE=<png>`: after the glide, release the key, let the view settle
    /// and the on-settle supersampling converge, capture the window to `<png>`, then report. The
    /// field report this exists for (2026-09-16): "when zooming, it still sometimes finishes with
    /// something that looks like a transparent overlay of another location/zoom level" — a
    /// converged average that folded a sample from a different view. The capture is the evidence;
    /// the tile trace's `accum … FOLD` lines say which sample.
    Settle,
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
    /// complete frame by 2^gap). This holds on `real` frames too: a PINNED refresh re-iterates
    /// every frame into the G-buffer but the display keeps serving the hold snapshot of the last
    /// complete frame until the pin adopts. Before design/live-zoom-smoothing.md P-B every
    /// reference install abandoned the pin, so at 4.0× (an install every ~0.2 s) the 1e28
    /// hand-over frame was shown at 24–53× (4.5–5.7 oct, measured 2026-09-16); the rate-aware
    /// refresh sizing bounds this at `HELD_MAX_OCT`.
    gap_oct: f64,
    lag: f64,
    orbit_id: u64,
    look: u64,
    /// Share of the selected zoom speed the pacer let through this frame.
    vel_frac: f64,
    res: f64,
    /// The iterate resolution actually dispatched last (`perf.last_res_v[0]`, pixels) — the
    /// rate-aware refresh cap shows up here, not in `res` (the AIMD scale it is a cap on).
    res_px: [u32; 2],
    /// Observed zoom speed the sizing used, octaves/s (`perf.zoom_oct_s`).
    oct_s: f64,
    /// Pinned refreshes ADOPTED so far (`perf.adopt_complete[0]`): the count that proves the
    /// held frame is being replaced, not merely re-iterated into a G-buffer nobody sees.
    adopts: u64,
    /// LIVE AUTO-NORMALIZATION state, for the 2026-09-17 reports ("sometimes doesn't normalize",
    /// "the colours bounce around on zoom", "it may go to a flat colour panel"). The palette
    /// mapping is decided from the escape RANGE and an aliasing predicate on the local GRADIENT,
    /// and the gradient is measured between adjacent texels of the RENDER — so it scales with the
    /// dispatch resolution. `norm_on` flipping frame to frame IS the bounce; `norm_lo`/`norm_hi`
    /// jumping is the range chasing a reading from another view.
    norm_lo: f32,
    norm_hi: f32,
    norm_grad: f32,
    norm_on: bool,
    /// 0 = linear map, 1 = log map, 2 = not engaged (the raw palette).
    norm_map: u8,
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
    /// `FRACTADYNE_ZOOMTEST_SETTLE=<png>`: where the settled, converged window goes (see
    /// `Phase::Settle`). Accumulation — off under every task invocation — is admitted for this
    /// run when set (`FractadyneApp::zoomtest_settling`).
    settle: Option<std::path::PathBuf>,
    /// A window capture has been requested and its reply is due next frame.
    settle_pending: bool,
    /// 0 = the one-sample capture (`<png>-s0.png`) is still owed; 1 = the converged capture.
    settle_stage: u8,
    /// `FRACTADYNE_ZOOMTEST_TAPS=<n>,<octaves>,<pause_s>`: instead of one continuous glide, `n`
    /// short glides of `<octaves>` each with a `<pause_s>` rest between them — the wheel-tap
    /// pattern in the field log behind the ghost report (accumulation begins at every rest and
    /// is interrupted by the next tap). `taps_left` counts down; `tap_start_l2` is where the
    /// current tap began; `pause_until` is the rest's end.
    taps: Option<(u32, f64, f64)>,
    taps_left: u32,
    tap_start_l2: f64,
    pause_until: Option<Instant>,
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
            settle: std::env::var_os("FRACTADYNE_ZOOMTEST_SETTLE")
                .filter(|v| !v.is_empty())
                .map(std::path::PathBuf::from),
            settle_pending: false,
            settle_stage: 0,
            taps: std::env::var("FRACTADYNE_ZOOMTEST_TAPS").ok().and_then(|s| {
                let mut it = s.split(',');
                let n = it.next()?.trim().parse::<u32>().ok()?;
                let oct = it.next()?.trim().parse::<f64>().ok()?;
                let pause = it.next()?.trim().parse::<f64>().ok()?;
                (n > 0 && oct > 0.0 && pause >= 0.0).then_some((n, oct, pause))
            }),
            taps_left: 0,
            tap_start_l2: 0.0,
            pause_until: None,
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

/// JSON has no NaN literal, and a NaN here is meaningful: the normalization has no reading yet.
fn json_f32(v: f32) -> String {
    if v.is_finite() { format!("{v:.2}") } else { "null".into() }
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

    /// A `--zoomtest` run with a settle tail (`FRACTADYNE_ZOOMTEST_SETTLE`): progressive on-settle
    /// supersampling is admitted for it, exactly as for `--shot`, since the converged average is
    /// what it captures.
    pub(crate) fn zoomtest_settling(&self) -> bool {
        self.harness.zoomtest.as_ref().is_some_and(|z| z.settle.is_some())
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
                // `FRACTADYNE_ZOOMTEST_PROFILE=user`: the reporting user's settings that shape
                // the refresh path (their 2026-09-16 session): native-resolution refreshes while
                // zooming, 2× AA, the dual view, a high motion-resolution floor, live
                // normalization. A ghost seen under those settings must be reproduced under them.
                if std::env::var("FRACTADYNE_ZOOMTEST_PROFILE").is_ok_and(|p| p == "user") {
                    self.render_cfg.prefer_detail = true;
                    self.render_cfg.aa = 2;
                    self.render_cfg.min_motion_res = 0.83;
                    self.coloring.normalize_live = true;
                    self.coloring.log_palette = true; // "Log color scale", as the report's screenshots show
                    self.dual = self.fractal.supports_julia();
                    self.dual_split = 0.588;
                    crate::diag::log_line(
                        "zoomtest",
                        "profile 'user': prefer_detail, aa=2, min_motion_res=0.83, normalize_live, dual (split 0.588)",
                    );
                }
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
                    if let Some((n, _, _)) = zt.taps {
                        zt.taps_left = n;
                        zt.tap_start_l2 = zt.start_l2;
                        crate::diag::log_line("zoomtest", &format!("tap 1 of {n} begins at 2^{:.2}", zt.start_l2));
                    }
                    advance(&mut zt, Phase::Glide);
                }
            }
            Phase::Glide => {
                let l2 = self.viewport.log2_magnification();
                // The interval that ends now belongs to the frame just presented — the one whose
                // params `prev_real` describes and whose zoom step is l2 − prev_l2.
                if let Some(prev) = zt.last_frame {
                    let dt_ms = now.duration_since(prev).as_secs_f64() * 1000.0;
                    // The mapping as the colour pass would take it this frame (None = the raw
                    // palette, i.e. "Normalize deep colors" is on but not engaged).
                    let nm = self.live_norm_cycle_offset(0);
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
                        res_px: self.perf.last_res_v[0],
                        oct_s: self.perf.zoom_oct_s,
                        adopts: self.perf.adopt_complete[0],
                        // The window the colour pass USES (`norm_shown`), not the fed target:
                        // the glide sits between them, and it is the shown one the eye sees.
                        norm_lo: self.perf.norm_shown[0].map_or(f32::NAN, |r| r.0),
                        norm_hi: self.perf.norm_shown[0].map_or(f32::NAN, |r| r.1),
                        norm_grad: self.perf.norm_grad[0].unwrap_or(f32::NAN),
                        norm_on: nm.is_some(),
                        norm_map: nm.map_or(2, |m| m.mode as u8),
                        mode: self.perf.last_mode,
                        gui_dt_ms: self.perf.last_dt_ms,
                    });
                }
                zt.last_frame = Some(now);
                zt.prev_l2 = l2;
                // The tap pattern: release after `<octaves>`, rest, press again. The rest is
                // long enough for the view to settle and the accumulation to BEGIN, and the
                // next tap interrupts it — the field log's shape (six begins in 15 s).
                if let Some((n, oct, pause)) = zt.taps {
                    if let Some(until) = zt.pause_until {
                        if now >= until {
                            zt.pause_until = None;
                            zt.taps_left = zt.taps_left.saturating_sub(1);
                            if zt.taps_left == 0 {
                                if zt.settle.is_some() {
                                    advance(&mut zt, Phase::Settle);
                                } else {
                                    self.zoomtest_report(zt); // exits
                                }
                            } else {
                                zt.tap_start_l2 = l2;
                                zt.driving = true;
                                crate::diag::log_line(
                                    "zoomtest",
                                    &format!("tap {} of {n} begins at 2^{l2:.2}", n + 1 - zt.taps_left),
                                );
                            }
                        }
                    } else if zt.driving && (l2 - zt.tap_start_l2 >= oct || in_phase > GLIDE_MAX_S) {
                        zt.driving = false; // the key comes up; the glide eases out during the rest
                        crate::diag::log_line(
                            "zoomtest",
                            &format!("tap {} of {n} released at 2^{l2:.2}, resting {pause:.1} s", n + 1 - zt.taps_left),
                        );
                        if zt.taps_left <= 1 && zt.settle.is_some() {
                            // The last release goes straight to the settle tail, so its
                            // one-sample capture can catch the average before it grows.
                            zt.taps_left = 0;
                            advance(&mut zt, Phase::Settle);
                        } else {
                            zt.pause_until = Some(now + std::time::Duration::from_secs_f64(pause));
                        }
                    }
                } else if l2 - zt.start_l2 >= zt.octaves || in_phase > GLIDE_MAX_S {
                    zt.driving = false; // the virtual key comes up THIS frame; the glide eases out
                    if zt.settle.is_some() {
                        advance(&mut zt, Phase::Settle);
                    } else {
                        self.zoomtest_report(zt); // exits
                    }
                }
            }
            Phase::Settle => {
                // Two captures of the SAME view: `<png>-s0.png` when the average holds exactly
                // one sample (the classic settled frame, presented through the accumulator),
                // and `<png>` once it has converged. Their difference is the de-speckle change
                // — high-frequency, cancelling under a blur — plus any ghost, which does not.
                // Capturing a control in a second run does not work: the tap sequence lands at
                // a slightly different depth each run and the whole image shifts.
                let no_accum = std::env::var_os("FRACTADYNE_NO_ACCUM").is_some();
                // Harvest a capture already in flight (the `--shot` idiom: the reply lands a
                // frame later as `Event::Screenshot`).
                if zt.settle_pending {
                    let image = ctx.input(|i| {
                        i.events.iter().find_map(|e| match e {
                            egui::Event::Screenshot { image, .. } => Some(image.clone()),
                            _ => None,
                        })
                    });
                    if let Some(image) = image {
                        let (w, h) = (image.size[0] as u32, image.size[1] as u32);
                        let mut bytes = Vec::with_capacity(image.pixels.len() * 4);
                        for px in &image.pixels {
                            bytes.extend_from_slice(&[px.r(), px.g(), px.b(), px.a()]);
                        }
                        let final_stage = zt.settle_stage == 1 || no_accum;
                        let out = {
                            let p = zt.settle.clone().unwrap();
                            if final_stage {
                                p
                            } else {
                                let mut s = p.with_extension("").into_os_string();
                                s.push("-s0.png");
                                std::path::PathBuf::from(s)
                            }
                        };
                        if let Some(dir) = out.parent() {
                            let _ = std::fs::create_dir_all(dir);
                        }
                        match fractadyne_export::write_png_rgba8(&out, w, h, &bytes, None) {
                            Ok(()) => {
                                let msg = format!(
                                    "settle capture → {} ({w}x{h}) at 2^{:.4}, accumulated {} sample(s)",
                                    out.display(),
                                    self.viewport.log2_magnification(),
                                    self.perf.accum_count[0]
                                );
                                crate::diag::log_line("zoomtest", &msg);
                                eprintln!("--zoomtest: {msg}");
                            }
                            Err(e) => eprintln!("--zoomtest: settle capture write failed: {e}"),
                        }
                        zt.settle_pending = false;
                        if final_stage {
                            self.zoomtest_report(zt); // exits
                        }
                        zt.settle_stage = 1;
                    }
                } else {
                    let count = self.perf.accum_count[0];
                    let active = self.perf.accum_active[0];
                    let converged = active && count >= crate::accum_target();
                    // With accumulation off (`FRACTADYNE_NO_ACCUM=1`) there is nothing to
                    // converge: capture once the settled render has been quiet for a while.
                    let quiet_single = no_accum
                        && in_phase > 6.0
                        && !self.perf.tile_pending[0]
                        && !self.perf.chunk_pending[0]
                        && self.recompute_rx[0].is_none();
                    // Stage 0 fires on the first folded sample this tail observes (ideally the
                    // one-sample average; the log line says how many it actually holds).
                    let due = match zt.settle_stage {
                        0 if !no_accum => active && count >= 1 && self.perf.accum_committed[0],
                        _ => converged || quiet_single,
                    };
                    if due || in_phase > SETTLE_MAX_S {
                        if !due {
                            eprintln!(
                                "--zoomtest: settle stage {} did not complete within {SETTLE_MAX_S:.0} s (accum active {active}, {count} sample(s)) — capturing anyway",
                                zt.settle_stage
                            );
                            zt.settle_stage = 1;
                        }
                        zt.settle_pending = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                    }
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
        let adopts = f.last().unwrap().adopts.saturating_sub(f.first().unwrap().adopts);
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
            "  reference installs {installs} (lookahead {look}) · pinned refreshes adopted {adopts} · depth lag max {lag_max:.2} · pacer throttled {paced_pct:.1}% of frames"
        );

        let mut rows = String::with_capacity(n * 160);
        for (i, r) in f.iter().enumerate() {
            if i > 0 {
                rows.push_str(",\n");
            }
            rows.push_str(&format!(
                "{{\"t\":{:.4},\"dt_ms\":{:.3},\"gui_dt_ms\":{:.3},\"l2\":{:.4},\"dl2\":{:.5},\"real\":{},\"gap_oct\":{:.4},\"lag\":{:.3},\"orbit_id\":{},\"look\":{},\"vel_frac\":{:.3},\"res\":{:.3},\"res_px\":[{},{}],\"oct_s\":{:.3},\"adopts\":{},\"norm_lo\":{},\"norm_hi\":{},\"norm_grad\":{},\"norm_on\":{},\"norm_map\":{},\"mode\":{}}}",
                r.t, r.dt_ms, r.gui_dt_ms, r.l2, r.dl2, r.real, r.gap_oct, r.lag, r.orbit_id, r.look, r.vel_frac, r.res, r.res_px[0], r.res_px[1], r.oct_s, r.adopts,
                json_f32(r.norm_lo), json_f32(r.norm_hi), json_f32(r.norm_grad), r.norm_on, r.norm_map, r.mode
            ));
        }
        let json = format!(
            "{{\n\"tool\": \"fractadyne --zoomtest\",\n\"version\": {},\n\"location\": {},\n\"zoom_rate\": {:.3},\n\"prefetch\": {},\n\"start_l2\": {:.4},\n\"octaves\": {oct:.4},\n\"secs\": {secs:.3},\n\"summary\": {{\"frames\":{n},\"fps\":{fps:.2},\"oct_s\":{oct_s:.4},\"dt_mean\":{dt_mean:.3},\"dt_p50\":{dt_p50:.3},\"dt_p95\":{dt_p95:.3},\"dt_p99\":{dt_p99:.3},\"dt_max\":{dt_max:.3},\"gt33\":{gt33},\"gt50\":{gt50},\"gt100\":{gt100},\"longest_ms\":{:.3},\"longest_t\":{:.3},\"longest_l2\":{:.3},\"reals\":{reals},\"hold_pct\":{hold_pct:.2},\"real_gap_mean\":{gap_mean:.3},\"real_gap_p95\":{gap_p95:.3},\"real_gap_max\":{gap_max:.3},\"step_p95\":{step_p95:.5},\"step_max\":{step_max:.5},\"gap_oct_max\":{gap_oct_max:.4},\"lag_max\":{lag_max:.3},\"installs\":{installs},\"lookahead\":{look},\"adopts\":{adopts},\"paced_pct\":{paced_pct:.2}}},\n\"frames\": [\n{rows}\n]\n}}\n",
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
