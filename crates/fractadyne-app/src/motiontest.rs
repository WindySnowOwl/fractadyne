//! `--motiontest` — the motion-presentation gate for chunked deep views.
//!
//! WHY THIS EXISTS (design/mode2-chunking.md §10-§11): `--livetest` checkpoints measure SETTLED
//! results, so the mode-2 live-flip regression — a moving frame's partial chunk progression
//! becoming the frozen texture, reported from the field as "the interior regions look mostly like
//! noise" — passed that gate 24/24 while plainly visible on screen. `--autodive` reaches the
//! regime but asserts device survival, not presentation. This harness runs the real windowed app
//! (the `autodive_frame`/`uitest_frame` in-loop shape), drives the two motion shapes that matter
//! (a continuous wheel-style dive, then the Home glide that lost the device on 2026-08-18), and
//! asserts INVARIANTS over `Perf`'s adoption counters, not raw per-frame numbers (the gate-flap
//! doctrine: outcomes gate, context attributes):
//!
//! - **A1 (the regression)**: `adopt_partial == 0` — a partial chunk progression must never be
//!   adopted as the frozen texture while the user has real content on screen. RED on the held
//!   slice-3 flip by construction: every interacting refresh there latches `[0, step)`.
//! - **A2 (anti-freeze)**: `adopt_complete >= 1` during the motion window — real detail must keep
//!   streaming; a gate that "fixes" A1 by holding forever (§10 option B, the field-reported
//!   ever-larger-blocks bug) fails here.
//! - **A3 (display honesty)**: `dirty_shown == 0` — no frame displayed the live texture while it
//!   diverged from the frozen bookkeeping during interaction.
//! - **Anti-vacuity**: the run FAILS unless it produced interacting chunk-eligible frames — a
//!   harness that never reached the regime proves nothing (the `--autodive 22` lesson: it cited
//!   the mode-2 crash while measuring 0 mode-2 frames).
//!
//! Exit codes follow the torture `classify` contract: 0 = all assertions hold, 2 = an assertion
//! failed or the run never reached the regime (never a pass), 4 = the watchdog (frame loop stopped
//! delivering). Run it with a wiped `FRACTADYNE_CONFIG_DIR` like every other gate.

use std::time::Instant;

use crate::FractadyneApp;

/// Corpus location 07 (44 digits) — the same structure-rich deep center the `iter-chunk` selftest
/// rows render, chosen there because a "deep magnification" over a trivially-escaping reference is
/// not a deep test. At 2^103.3 ≈ 1.3e31× it is comfortably past the ~1e28× mode-2 floor.
const CX: &str = "-1.178853950372678747911373866849720956148855";
const CY: &str = "0.1853420232408490265512092752061929308714979";
const LOG2_MAG: f64 = 103.3;
/// Explicit ask (auto-iter off). Big enough that a motion frame's budget-bounded pass covers only
/// a fraction of it (the chunked regime, at any panel size), small enough that the cold bignum
/// reference build stays in seconds, not minutes.
const ASK: u32 = 1_000_000;

/// Wall-clock phase budgets. The reference build is the only genuinely open-ended wait.
const WAIT_REF_S: f64 = 180.0;
const DIVE_S: f64 = 6.0;
const HOME_S: f64 = 90.0;
const SETTLE_S: f64 = 5.0;
/// Whole-run hard deadline, enforced by a watchdog THREAD (the `--autodive` lesson: every in-frame
/// check needs frames to still be delivered, which is exactly not the case that needs bounding).
const DEADLINE_S: f64 = 360.0;
/// A run whose motion phases produced fewer interacting chunk-eligible frames than this tested
/// nothing and fails regardless of the assertion counters.
const MIN_MOTION_FRAMES: u64 = 100;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Jump,
    WaitRef,
    Dive,
    Home,
    Settle,
}

/// Counter snapshot taken when the motion window opens, so the verdict reads deltas over exactly
/// the driven phases (boot-time settling is not part of the experiment).
#[derive(Clone, Copy, Default)]
struct Base {
    partial: u64,
    complete: u64,
    motion_frames: u64,
    dirty: u64,
}

pub(crate) struct MotionTest {
    phase: Phase,
    t0: Instant,
    phase_t0: Instant,
    frames: u64,
    base: Base,
    /// Cleared by the verdict paths so the watchdog thread stands down.
    done: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl MotionTest {
    pub(crate) fn new() -> Self {
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
                    "WATCHDOG: no verdict within {DEADLINE_S:.0}s — the frame loop stopped delivering (deep reference build on the main thread, or a wedge). Nothing was measured."
                );
                crate::diag::log_line("motiontest", &msg);
                eprintln!();
                eprintln!("--motiontest: {msg}");
                crate::exit(4);
            });
        }
        Self {
            phase: Phase::Jump,
            t0: Instant::now(),
            phase_t0: Instant::now(),
            frames: 0,
            base: Base::default(),
            done,
        }
    }
}

impl FractadyneApp {
    /// One step per frame, called from `update()` after the measurement apply (same in-loop shape
    /// as `autodive_frame`). Sets input state BEFORE the central draw of the same `update()`, so
    /// the frame it shapes is the frame the counters describe.
    pub(crate) fn motiontest_frame(&mut self, ctx: &egui::Context) {
        let Some(mut mt) = self.motiontest.take() else { return };
        mt.frames += 1;
        let in_phase = mt.phase_t0.elapsed().as_secs_f64();
        let now = ctx.input(|i| i.time);

        let advance = |mt: &mut MotionTest, p: Phase| {
            crate::diag::log_line(
                "motiontest",
                &format!("phase {:?} -> {:?} at +{:.1}s", mt.phase, p, mt.t0.elapsed().as_secs_f64()),
            );
            mt.phase = p;
            mt.phase_t0 = Instant::now();
        };

        match mt.phase {
            Phase::Jump => {
                // The deep jump, in-process (no session staging to silently not load): corpus
                // loc 07 in mode 2, explicit ask, auto-iter off. `parse_bf_prec` floors the
                // precision — astro-float's own FromStr derives precision from digit count.
                let cx = fractadyne_core::parse_bf_prec(CX, 192);
                let cy = fractadyne_core::parse_bf_prec(CY, 192);
                let (Some(cx), Some(cy)) = (cx, cy) else {
                    self.motiontest_verdict(mt, false, "internal: corpus center failed to parse");
                };
                self.render_cfg.max_iter = ASK;
                self.render_cfg.auto_iter = false;
                self.viewport.set_center_log2mag(cx, cy, LOG2_MAG);
                self.pointer.zoom_vel = 0.0;
                self.invalidate_refs();
                advance(&mut mt, Phase::WaitRef);
            }
            Phase::WaitRef => {
                // The regime needs a full (non-progressive-coarse) reference installed, no build
                // in flight, and at least one real frame latched as frozen content — the pin (and
                // the counters) key on replacing REAL content, not a cold view's first frame.
                let rc = &self.ref_cache[0];
                let ready = rc.ref_pt.is_some()
                    && !rc.partial
                    && self.recompute_rx[0].is_none()
                    && rc.frozen_center.is_some()
                    && mt.frames > 30;
                if ready {
                    mt.base = Base {
                        partial: self.perf.adopt_partial[0],
                        complete: self.perf.adopt_complete[0],
                        motion_frames: self.perf.chunk_motion_frames[0],
                        dirty: self.perf.dirty_shown[0],
                    };
                    crate::diag::log_line(
                        "motiontest",
                        &format!(
                            "reference ready (orbit_len={} chunk_ok={} chunk_fe_ok={}) — driving",
                            rc.orbit_len, self.perf.chunk_ok, self.perf.chunk_fe_ok
                        ),
                    );
                    advance(&mut mt, Phase::Dive);
                } else if in_phase > WAIT_REF_S {
                    self.motiontest_verdict(mt, false, "boot never completed: no full reference");
                }
            }
            Phase::Dive => {
                if in_phase < DIVE_S {
                    // A sustained wheel-style dive: the `active` check reads `zoom_vel`, stamps
                    // `settle_t`, and the central draw applies the (pipeline-paced) zoom this
                    // same frame — the exact shape of a user holding the wheel.
                    self.pointer.zoom_vel = 2.5;
                } else {
                    self.pointer.zoom_vel = 0.0;
                    // Straight into the Home glide with no settle gap — the glide stamps
                    // `settle_t` every frame, so interaction is continuous across the seam.
                    self.zoom_home(now);
                    advance(&mut mt, Phase::Home);
                }
            }
            Phase::Home => {
                if self.home_anim.is_none() {
                    advance(&mut mt, Phase::Settle);
                } else if in_phase > HOME_S {
                    self.motiontest_verdict(mt, false, "home glide never finished");
                }
            }
            Phase::Settle => {
                if in_phase > SETTLE_S {
                    let b = mt.base;
                    let partial = self.perf.adopt_partial[0].wrapping_sub(b.partial);
                    let complete = self.perf.adopt_complete[0].wrapping_sub(b.complete);
                    let motion =
                        self.perf.chunk_motion_frames[0].wrapping_sub(b.motion_frames);
                    let dirty = self.perf.dirty_shown[0].wrapping_sub(b.dirty);
                    eprintln!();
                    eprintln!(
                        "--motiontest: frames={} motion-chunk-frames={} (chunk_ok={} chunk_fe_ok={})",
                        mt.frames, motion, self.perf.chunk_ok, self.perf.chunk_fe_ok
                    );
                    eprintln!(
                        "--motiontest: adopt partial={partial} complete={complete} dirty-shown={dirty}"
                    );
                    let mut fails: Vec<String> = Vec::new();
                    if motion < MIN_MOTION_FRAMES {
                        fails.push(format!(
                            "VACUOUS: only {motion} interacting chunk-eligible frames (need >= {MIN_MOTION_FRAMES}) — the regime was not reached"
                        ));
                    }
                    if partial != 0 {
                        fails.push(format!(
                            "A1: {partial} partial progression(s) adopted as the frozen texture during motion (the §9 noise regression)"
                        ));
                    }
                    if complete == 0 {
                        fails.push(
                            "A2: no complete refresh was adopted during motion — detail stopped streaming (the option-B freeze shape)".into(),
                        );
                    }
                    if dirty != 0 {
                        fails.push(format!(
                            "A3: {dirty} frame(s) displayed the live texture while it diverged from the frozen bookkeeping"
                        ));
                    }
                    let pass = fails.is_empty();
                    self.motiontest_verdict(
                        mt,
                        pass,
                        &if pass {
                            "all motion-presentation invariants hold".to_string()
                        } else {
                            fails.join("; ")
                        },
                    );
                }
            }
        }

        self.motiontest = Some(mt);
        ctx.request_repaint();
    }

    fn motiontest_verdict(&mut self, mt: MotionTest, pass: bool, msg: &str) -> ! {
        mt.done.store(true, std::sync::atomic::Ordering::Relaxed);
        let verdict = if pass { "PASS" } else { "FAIL" };
        crate::diag::log_line("motiontest", &format!("{verdict}: {msg}"));
        eprintln!("--motiontest: VERDICT {verdict}: {msg}");
        // 2, not 1, on failure: torture's `classify` maps 2 -> FailAssert ("ran fine, result
        // wrong"), and a run that never reached the regime must never read as a pass.
        crate::exit(if pass { 0 } else { 2 });
    }
}
