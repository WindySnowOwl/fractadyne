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
    /// log10 magnification of the view being soaked.
    decades: f64,
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
            // reference orbit, which is the regime the checklist row is about.
            self.uitest_set_live(ctx, s.decades);
            eprintln!(
                "[soak] {:.0}s at 1e{:.0}x — liveness window {}s (watchdog thread), \
                 growth limit {GROWTH_LIMIT_MB} MB",
                s.duration.as_secs_f64(),
                s.decades,
                WINDOW.as_secs()
            );
        }

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
