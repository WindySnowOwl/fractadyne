//! `--soak SECONDS` — leave the app sitting at a deep view and assert it is STILL ALIVE.
//!
//! Checklist step 103 ("leave the app running idle at a deep view for five minutes: no creeping
//! memory growth, no watchdog restart, view still correct").
//!
//! ⭐⭐**A soak that greps only for crashes passes a hung app.** That is why this exists as a
//! harness rather than as "run it and look at the log": the failure mode at depth is not a panic,
//! it is a process that stops producing frames while remaining perfectly running.
//!
//! ⚠⚠**And the liveness judge must NOT live on the thread it is judging.** The first version of
//! this counted frames inside the per-frame hook and failed a window that saw none — which cannot
//! happen, because a window with no frames is a window where the hook never ran. It would have
//! reported nothing at all on the exact failure it was written for: a check that cannot go red is
//! not a gate. The counter is therefore bumped from the UI thread and read by a WATCHDOG THREAD,
//! which is still scheduled when the UI thread is wedged.
//!
//! Three things fail the run, each on its own:
//!   * **frames advance** — every [`WINDOW`] must contain at least one new frame (watchdog thread);
//!   * **memory holds** — resident set may not grow past [`GROWTH_LIMIT_MB`] over the soak;
//!   * **no crash reports appear** — a census difference, the same rule `--uitest` uses.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// How often liveness is judged. Long enough that a slow deep frame is not a stall; short enough
/// that a wedge is caught well inside a five-minute soak.
const WINDOW: Duration = Duration::from_secs(20);

/// Resident-set growth allowed across the whole soak. Generous on purpose: the row is about a
/// LEAK, not the few megabytes a settled view legitimately moves between tile sizes.
const GROWTH_LIMIT_MB: u64 = 256;

/// Frames drawn since launch, bumped by the UI thread and read by the watchdog.
static FRAMES: AtomicU64 = AtomicU64::new(0);
/// Set when the soak has finished, so the watchdog stops judging a process on its way out.
static DONE: AtomicBool = AtomicBool::new(false);

pub(crate) struct Soak {
    started: Instant,
    duration: Duration,
    /// Resident set at the first sample, and the largest seen since.
    rss_first_mb: u64,
    rss_peak_mb: u64,
    next_sample: Instant,
    samples: u32,
    crashes_at_start: Vec<String>,
    /// log10 magnification of the view being soaked; NaN = the session's own view
    /// (`--soak-depth session`).
    decades: f64,
    /// W9's regime predicate over every record of the run, and the next record `seq` to fold.
    regime: RegimeAcc,
    regime_next: u64,
}

impl Soak {
    pub(crate) fn new(secs: f64, decades: f64) -> Self {
        let now = Instant::now();
        let duration = Duration::from_secs_f64(secs.max(1.0));
        spawn_watchdog(duration);
        Self {
            started: now,
            duration,
            rss_first_mb: 0,
            rss_peak_mb: 0,
            next_sample: now + WINDOW,
            samples: 0,
            crashes_at_start: crate::diag::crash_report_names(),
            decades,
            regime: RegimeAcc::default(),
            regime_next: 0,
        }
    }
}

/// The liveness judge. Lives on its own thread precisely so that a wedged UI thread — the thing
/// being tested for — cannot also silence the test. Kills the process on a stall, because a
/// harness that hangs when it detects a hang is no better than the hang.
fn spawn_watchdog(duration: Duration) {
    std::thread::spawn(move || {
        let started = Instant::now();
        let mut last = FRAMES.load(Ordering::Relaxed);
        loop {
            std::thread::sleep(WINDOW);
            if DONE.load(Ordering::Relaxed) {
                return;
            }
            let now = FRAMES.load(Ordering::Relaxed);
            if now == last {
                eprintln!(
                    "\n=== --soak FAILED after {:.0}s ===",
                    started.elapsed().as_secs_f64()
                );
                eprintln!(
                    "soak-liveness: FAIL — no frame drawn in {}s (frame counter stuck at {now})",
                    WINDOW.as_secs()
                );
                // Not `crate::exit`: that path wants the UI thread, which is the thread that has
                // stopped responding. Leave the unclean-exit marker armed — the session really
                // did die, and the next launch should say so.
                std::process::exit(1);
            }
            last = now;
            if started.elapsed() > duration + WINDOW * 2 {
                // The UI thread should have finished the soak by now; do not linger.
                return;
            }
        }
    });
}

impl crate::FractadyneApp {
    /// One frame of the soak. Jumps to the deep view on the first call, then just watches.
    pub(crate) fn soak_frame(&mut self, ctx: &egui::Context) {
        ctx.request_repaint(); // idle means no input events; keep the loop turning
        let Some(mut s) = self.harness.soak.take() else { return };
        let n = FRAMES.fetch_add(1, Ordering::Relaxed);
        if n == 0 {
            // The view under soak: deep enough to be on the perturbation path with a real
            // reference orbit, which is the regime the checklist row is about — or, with
            // `--soak-depth session`, whatever view the session opened at.
            let at = if s.decades.is_nan() {
                "the session's view".to_string()
            } else {
                self.uitest_set_live(ctx, s.decades);
                format!("1e{:.0}x", s.decades)
            };
            eprintln!(
                "[soak] {:.0}s at {at} — liveness window {}s (watchdog thread), growth limit \
                 {GROWTH_LIMIT_MB} MB; tunables {}",
                s.duration.as_secs_f64(),
                WINDOW.as_secs(),
                crate::tunables::status_line()
            );
        }

        // Fold every record since the last frame: one or two a frame, so the ring (4,096) can
        // only get ahead of this if a single frame took thousands of records, which it cannot.
        let (recs, next, lost) = crate::diag::frame_record::since(s.regime_next);
        for r in &recs {
            s.regime.add(r);
        }
        s.regime_next = next;
        s.regime.lost += lost;

        let now = Instant::now();
        if now >= s.next_sample {
            let rss = crate::sysinfo::process_memory().0 / (1024 * 1024);
            if s.rss_first_mb == 0 {
                s.rss_first_mb = rss;
            }
            s.rss_peak_mb = s.rss_peak_mb.max(rss);
            s.samples += 1;
            eprintln!(
                "[soak] +{:5.0}s  frames {:7}  rss {rss} MB",
                now.duration_since(s.started).as_secs_f64(),
                FRAMES.load(Ordering::Relaxed),
            );
            s.next_sample = now + WINDOW;
        }

        if now.duration_since(s.started) >= s.duration {
            DONE.store(true, Ordering::Relaxed);
            let code = self.soak_finish(&s);
            crate::exit(code);
        }
        self.harness.soak = Some(s);
    }

    /// Verdict + exit code. Every failure is named; a pass says what it measured.
    fn soak_finish(&self, s: &Soak) -> i32 {
        let elapsed = s.started.elapsed().as_secs_f64();
        let frames = FRAMES.load(Ordering::Relaxed);
        let growth = s.rss_peak_mb.saturating_sub(s.rss_first_mb);
        let new_crashes: Vec<String> = crate::diag::crash_report_names()
            .into_iter()
            .filter(|n| !s.crashes_at_start.contains(n))
            .collect();
        let mut fails: Vec<String> = Vec::new();
        // A soak too short to have been judged proves nothing, and must not report success.
        if s.samples == 0 {
            fails.push(format!(
                "no liveness window completed in {elapsed:.0}s — run for at least {}s",
                WINDOW.as_secs() * 2
            ));
        }
        if growth > GROWTH_LIMIT_MB {
            fails.push(format!("resident set grew {growth} MB (limit {GROWTH_LIMIT_MB})"));
        }
        if !new_crashes.is_empty() {
            fails.push(format!("crash report(s) appeared: {}", new_crashes.join(", ")));
        }
        eprintln!(
            "\n=== --soak complete: {elapsed:.0}s, {frames} frames, {} samples, rss {} -> {} MB ===",
            s.samples, s.rss_first_mb, s.rss_peak_mb
        );
        // With the escape instrument armed this soak IS W9's escaped-reference rung, and a
        // survival means nothing unless the regime was entered.
        let regime = s.regime.report();
        eprintln!("soak-regime: {}", regime.line);
        eprintln!(
            "soak-regime-coverage: {} record(s) read over the whole run, {} lost to the ring",
            s.regime.folded, s.regime.lost
        );
        let escape_armed = crate::Perf::ref_escape_at() > 0;
        if fails.is_empty() && escape_armed && !regime.entered {
            eprintln!(
                "soak-liveness: VACUOUS (exit 2) — FRACTADYNE_REF_ESCAPE_AT is armed but the regime was never \
                 entered, so surviving it proves nothing"
            );
            return 2;
        }
        if fails.is_empty() {
            eprintln!("soak-liveness: PASS (frames advanced in every window, memory held)");
            0
        } else {
            for f in &fails {
                eprintln!("soak-liveness: FAIL — {f}");
            }
            1
        }
    }
}

/// W9's regime-reached predicate (design §6.7), read from the frame record rather than from any
/// timing instrument — the lesson of U8/U16 is that a rung judged by the thing it stresses can pass
/// without having stressed it.
pub(crate) struct RegimeReport {
    /// At least one frame on a short COMPLETE reference (`ref_len` ≤ a quarter of the iteration
    /// ask) with BLA off, AND a counter reading from such a frame with rebases and no BLA skips.
    pub(crate) entered: bool,
    pub(crate) line: String,
}

/// The predicate's running totals, folded one record at a time so the verdict covers the WHOLE run.
/// ⛔It used to be read from `frame_record::snapshot()` at the end — the last `RING_LEN` (4,096)
/// records — and the rung's storm comes in its first seconds: on the RX 6800 XT (2026-09-23) frames
/// 12–19 ran on the 655-sample reference at 248M rebases a frame, and 8,200 frames later the verdict
/// said "SHAPE ONLY … VACUOUS". A check that runs at the END cannot see what rotated out of its window.
#[derive(Default)]
pub(crate) struct RegimeAcc {
    in_shape: u64,
    storm: u64,
    unchunked: u64,
    dispatched: u64,
    max_rebase: u64,
    /// `(min, max)` of `fe_budget` over the frames in shape.
    budget: Option<(u64, u64)>,
    /// `(ref_len, gpu_iter)` of the first frame in shape.
    first_shape: Option<(u32, u32)>,
    /// `(frame, t_ms)` of the first storm reading.
    first_storm: Option<(u64, u64)>,
    /// Records read, and records the ring overwrote before they could be — a verdict must not
    /// claim to have seen those.
    pub(crate) folded: u64,
    pub(crate) lost: u64,
}

impl RegimeAcc {
    pub(crate) fn add(&mut self, r: &crate::diag::frame_record::FrameRecord) {
        use crate::diag::frame_record::kind;
        self.folded += 1;
        let shaped = r.kind == kind::FRAME
            && r.has_ref
            && !r.ref_partial
            && r.ref_len > 0
            && r.gpu_iter as u64 >= 4 * r.ref_len as u64
            && !r.bla_on;
        if !shaped {
            return;
        }
        self.in_shape += 1;
        self.first_shape.get_or_insert((r.ref_len, r.gpu_iter));
        if r.ctr_new && r.ctr_rebase > 0 && r.ctr_bla_skip == 0 {
            self.storm += 1;
            self.max_rebase = self.max_rebase.max(r.ctr_rebase);
            self.first_storm.get_or_insert((r.frame, r.t_ms));
        }
        if r.dispatched {
            self.dispatched += 1;
            if !r.chunked {
                self.unchunked += 1;
            }
        }
        let (lo, hi) = self.budget.unwrap_or((u64::MAX, 0));
        self.budget = Some((lo.min(r.fe_budget), hi.max(r.fe_budget)));
    }

    pub(crate) fn report(&self) -> RegimeReport {
        let mut line = match self.first_shape {
            None => "NOT ENTERED — no frame on a short complete reference (ref_len <= ask/4) with BLA off".to_string(),
            Some((ref_len, ask)) => {
                let (bmin, bmax) = self.budget.unwrap_or((0, 0));
                format!(
                    "{} {} frame(s) on a short escaped reference (ref_len {ref_len} vs ask {ask}, BLA off); {} \
                     counter reading(s) with rebases and bla_skip=0 (max {} rebases); {} of {} dispatches \
                     UN-chunked; budget {:.3e}..{:.3e}",
                    if self.storm == 0 { "SHAPE ONLY:" } else { "ENTERED:" },
                    self.in_shape,
                    self.storm,
                    self.max_rebase,
                    self.unchunked,
                    self.dispatched,
                    bmin as f64,
                    bmax as f64
                )
            }
        };
        if let Some((frame, t_ms)) = self.first_storm {
            line.push_str(&format!("; first storm reading at frame {frame} (+{:.1}s)", t_ms as f64 / 1000.0));
        }
        if self.lost > 0 {
            line.push_str(&format!("; ⚠{} record(s) overwritten before they were read", self.lost));
        }
        RegimeReport { entered: self.storm > 0, line }
    }
}

pub(crate) fn regime_report(recs: &[crate::diag::frame_record::FrameRecord]) -> RegimeReport {
    let mut acc = RegimeAcc::default();
    for r in recs {
        acc.add(r);
    }
    acc.report()
}

#[cfg(test)]
#[path = "soak_tests.rs"]
mod tests;
