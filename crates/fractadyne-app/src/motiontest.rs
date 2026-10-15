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
//! - **A2 (anti-freeze)**: `adopt_complete >= 1` during the motion window, counting frames from the
//!   second GPU (`live_adopted_total`, `motion_worker`) with the pins they replace — real detail must keep
//!   streaming; a gate that "fixes" A1 by holding forever (§10 option B, the field-reported
//!   ever-larger-blocks bug) fails here.
//! - **A3 (display honesty)**: `dirty_shown == 0` — no frame displayed the live texture while it
//!   diverged from the frozen bookkeeping during interaction.
//! - **A4 (walk restarts)**: walks restarted with part of their progress thrown away
//!   (`Perf::walk_restarts`, the frame record's `walk_restarted`). The iteration-starvation shape: a
//!   controller that re-chose the resolution every few frames restarted the walk every few frames,
//!   and 36% of a dive went blank while every per-pass metric improved. RED past
//!   [`MAX_DIVE_RESTARTS`] in the DIVE (design/live-render-robustness.md §7.4; 3–4 measured on the
//!   RTX 3080, 2026-10-13). The Home glide is reported, not gated: at 1M iterations its pins are
//!   outrun by the glide (~6 frames each, 30–36 per glide) — abandons, not a re-chosen walk.
//!   Explanatory: the screen is the verdict on a motion fix, never this count alone.
//! - **A5 (a jump keeps the jump floor, U39)**: first, at depth, a 100× click-to-zoom from the
//!   settled view (the real actuator, at the panel's centre), watched for [`CLICK_S`] and then
//!   clicked back out by the same factor before the motion begins. Until a frame at the new view is
//!   latched the display holds the old frame 6.64 octaves magnified, and that is what the jump floor
//!   bounds (`HELD_JUMP_MAX_OCT`: the 2026-09-18 "goes black" report was that frame unbounded, an
//!   11-pixel patch). RED if any held frame in the click window took the DIVE floor past 8× — the
//!   jump read as motion. Reported as not reached when no frame was held across the click.
//! - **A6 (no clamp in motion, U39)**: no frame of the dive or the Home glide showed its held frame
//!   at the jump floor's clamped scale (`Perf::hold_bound_moving`). Before U39 every moving frame
//!   that was not a Space glide took the jump floor, the Home glide included, and it bound on 7
//!   frames of one baseline run of three.
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
pub(crate) const CX: &str = "-1.178853950372678747911373866849720956148855";
pub(crate) const CY: &str = "0.1853420232408490265512092752061929308714979";
pub(crate) const LOG2_MAG: f64 = 103.3;
/// Explicit ask (auto-iter off). Big enough that a motion frame's budget-bounded pass covers only
/// a fraction of it (the chunked regime, at any panel size), small enough that the cold bignum
/// reference build stays in seconds, not minutes.
const ASK: u32 = 1_000_000;

/// Wall-clock phase budgets. The reference build is the only genuinely open-ended wait.
const WAIT_REF_S: f64 = 180.0;
const DIVE_S: f64 = 6.0;
const HOME_S: f64 = 90.0;
const SETTLE_S: f64 = 5.0;
/// How long A5 watches after the click.
const CLICK_S: f64 = 4.0;
/// Whole-run hard deadline, enforced by a watchdog THREAD (the `--autodive` lesson: every in-frame
/// check needs frames to still be delivered, which is exactly not the case that needs bounding).
const DEADLINE_S: f64 = 360.0;
/// A run whose motion phases produced fewer interacting chunk-eligible frames than this tested
/// nothing and fails regardless of the assertion counters.
const MIN_MOTION_FRAMES: u64 = 100;
/// A4's line for the dive phase (design §7.4: "more than 12 restarts per dive").
const MAX_DIVE_RESTARTS: u64 = 12;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Jump,
    WaitRef,
    Dive,
    Home,
    Settle,
    Click,
}

/// A7 is vacuous under this many GPU readings checked (a run measures ~400 on the RTX 3080).
const A7_MIN_READINGS: u64 = 50;

/// Counter snapshot taken when the motion window opens, so the verdict reads deltas over exactly
/// the driven phases (boot-time settling is not part of the experiment).
#[derive(Clone, Copy, Default)]
struct Base {
    partial: u64,
    complete: u64,
    worker: u64,
    motion_frames: u64,
    dirty: u64,
    restarts: u64,
    hold_moving: u64,
    /// Pinned walks that completed blank (`Perf::blank_walks_total`).
    blank: u64,
}

pub(crate) struct MotionTest {
    phase: Phase,
    t0: Instant,
    phase_t0: Instant,
    frames: u64,
    base: Base,
    /// `Perf::walk_restarts[0]` when the Home glide began (A4 splits the dive from the glide).
    restarts_at_home: u64,
    /// The motion phases' failures, carried past the click to the one verdict.
    fails: Vec<String>,
    /// `Perf::hold_floor_frames[0]` when the click went in (A5 reads the click window's deltas).
    click_base: [u64; 5],
    /// A5's click has been made (the second `WaitRef` leads to the dive).
    clicked: bool,
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
            restarts_at_home: 0,
            fails: Vec::new(),
            click_base: [0; 5],
            clicked: false,
            done,
        }
    }
}

impl FractadyneApp {
    /// One step per frame, called from `update()` after the measurement apply (same in-loop shape
    /// as `autodive_frame`). Sets input state BEFORE the central draw of the same `update()`, so
    /// the frame it shapes is the frame the counters describe.
    pub(crate) fn motiontest_frame(&mut self, ctx: &egui::Context) {
        let Some(mut mt) = self.harness.motiontest.take() else { return };
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
                    self.motiontest_verdict(mt, crate::verdict::RED, "internal: corpus center failed to parse");
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
                if ready && !mt.clicked {
                    // A5 first, at depth: a 100× click from the settled deep view, through the real
                    // actuator. (After the Home glide the view is at 1×, in direct mode, where
                    // nothing is ever held.)
                    mt.clicked = true;
                    mt.click_base = self.perf.hold_floor_frames[0];
                    self.render_cfg.click_zoom_factor = 100.0;
                    let (px, py) = (self.viewport.width_px * 0.5, self.viewport.height_px * 0.5);
                    self.click_zoom_at(px, py, false, now);
                    advance(&mut mt, Phase::Click);
                } else if ready {
                    mt.base = Base {
                        partial: self.perf.adopt_partial[0],
                        complete: self.perf.adopt_complete[0],
                        worker: self.perf.live_adopted_total[0],
                        motion_frames: self.perf.chunk_motion_frames[0],
                        dirty: self.perf.dirty_shown[0],
                        restarts: self.perf.walk_restarts[0],
                        hold_moving: self.perf.hold_bound_moving[0],
                        blank: self.perf.blank_walks_total[0],
                    };
                    let rc = &self.ref_cache[0];
                    crate::diag::log_line(
                        "motiontest",
                        &format!(
                            "reference ready (orbit_len={} actuators={:?}) — driving",
                            rc.orbit_len, self.perf.actuators
                        ),
                    );
                    advance(&mut mt, Phase::Dive);
                } else if in_phase > WAIT_REF_S {
                    // Nothing was driven: the experiment never started (W8 — not a failure of motion).
                    self.motiontest_verdict(mt, crate::verdict::VACUOUS, "boot never completed: no full reference");
                }
            }
            Phase::Click => {
                if in_phase > CLICK_S {
                    use crate::render::frame_plan::hold_floor;
                    let d = |code: u8| {
                        self.perf.hold_floor_frames[0][code as usize]
                            .wrapping_sub(mt.click_base[code as usize])
                    };
                    let (bound, deep, held) = (
                        d(hold_floor::JUMP_BOUND),
                        d(hold_floor::DIVE_DEEP),
                        d(hold_floor::DIVE) + d(hold_floor::JUMP) + d(hold_floor::JUMP_BOUND) + d(hold_floor::DIVE_DEEP),
                    );
                    let a5 = if deep > 0 {
                        "RED"
                    } else if bound > 0 {
                        "held at the jump floor"
                    } else {
                        "not reached (no frame held across the click)"
                    };
                    eprintln!(
                        "--motiontest: A5 100x click: {held} held frame(s), {bound} at the jump floor binding, {deep} at the dive floor past 8x — {a5}"
                    );
                    crate::diag::log_line(
                        "motiontest",
                        &format!("A5 click: held={held} jump-bound={bound} dive-deep={deep} ({a5})"),
                    );
                    if deep > 0 {
                        mt.fails.push(format!(
                            "A5: {deep} frame(s) after a 100x click held the old frame at the DIVE floor past 8x — the jump was read as motion (the 11-pixel patch)"
                        ));
                    }
                    // Back out by the same factor at the centre: the view the motion phases were
                    // calibrated at, without dropping the references (an undo would).
                    let (px, py) = (self.viewport.width_px * 0.5, self.viewport.height_px * 0.5);
                    self.click_zoom_at(px, py, true, now);
                    advance(&mut mt, Phase::WaitRef);
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
                    mt.restarts_at_home = self.perf.walk_restarts[0];
                    advance(&mut mt, Phase::Home);
                }
            }
            Phase::Home => {
                if self.home_anim.is_none() {
                    advance(&mut mt, Phase::Settle);
                } else if in_phase > HOME_S {
                    self.motiontest_verdict(mt, crate::verdict::RED, "home glide never finished");
                }
            }
            Phase::Settle => {
                if in_phase > SETTLE_S {
                    let b = mt.base;
                    let partial = self.perf.adopt_partial[0].wrapping_sub(b.partial);
                    let complete = self.perf.adopt_complete[0].wrapping_sub(b.complete);
                    let worker = self.perf.live_adopted_total[0].wrapping_sub(b.worker);
                    let motion =
                        self.perf.chunk_motion_frames[0].wrapping_sub(b.motion_frames);
                    let dirty = self.perf.dirty_shown[0].wrapping_sub(b.dirty);
                    let dive_restarts = mt.restarts_at_home.wrapping_sub(b.restarts);
                    let home_restarts = self.perf.walk_restarts[0].wrapping_sub(mt.restarts_at_home);
                    eprintln!();
                    eprintln!(
                        "--motiontest: frames={} motion-chunk-frames={} (actuators {:?})",
                        mt.frames, motion, self.perf.actuators
                    );
                    eprintln!(
                        "--motiontest: adopt partial={partial} complete={complete} worker={worker} dirty-shown={dirty} \
                         (session totals: converged={} blank-walks={})",
                        self.perf.adopt_converged[0], self.perf.blank_walks_total[0]
                    );
                    eprintln!(
                        "--motiontest: A4 walk restarts (progress thrown away): dive={dive_restarts} (RED past {MAX_DIVE_RESTARTS}) home={home_restarts} (reported)"
                    );
                    crate::diag::log_line(
                        "motiontest",
                        &format!("A4 walk restarts: dive={dive_restarts} home={home_restarts}"),
                    );
                    // A5's failure, if any, came from the click before the motion.
                    let mut fails = std::mem::take(&mut mt.fails);
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
                    if complete + worker == 0 {
                        fails.push(
                            "A2: no complete refresh was adopted during motion — detail stopped streaming (the option-B freeze shape)".into(),
                        );
                    }
                    if dirty != 0 {
                        fails.push(format!(
                            "A3: {dirty} frame(s) displayed the live texture while it diverged from the frozen bookkeeping"
                        ));
                    }
                    if dive_restarts > MAX_DIVE_RESTARTS {
                        fails.push(format!(
                            "A4: {dive_restarts} walks restarted with progress thrown away during the dive (more than {MAX_DIVE_RESTARTS}: the iteration-starvation shape)"
                        ));
                    }
                    // A7, provenance (W6, design/live-render-robustness.md §7.6 gate 1), over the whole
                    // session from its first frame — boot included, where the cold-start holds are.
                    let pt = self.perf.prov[0];
                    eprintln!(
                        "--motiontest: A7 provenance: {} GPU readings checked, {} named a frame with no pass; \
                         {} unverified holds served; {} climb readings judged at a cap; {} settled climb starvations",
                        pt.checked, pt.unran, pt.unverified_served, pt.judged_capped, pt.climb_starved
                    );
                    crate::diag::log_line(
                        "motiontest",
                        &format!(
                            "A7 provenance: checked={} unran={} unverified_served={} judged_capped={} climb_starved={}",
                            pt.checked, pt.unran, pt.unverified_served, pt.judged_capped, pt.climb_starved
                        ),
                    );
                    if pt.checked < A7_MIN_READINGS {
                        fails.push(format!(
                            "VACUOUS A7: only {} GPU readings checked (need >= {A7_MIN_READINGS})",
                            pt.checked
                        ));
                    }
                    if pt.unran + pt.unverified_served + pt.judged_capped + pt.climb_starved != 0 {
                        fails.push(format!(
                            "A7: provenance — {} reading(s) named a frame with no pass, {} unverified hold(s) served, \
                             {} climb reading(s) judged at a cap, {} settled climb starvation(s)",
                            pt.unran, pt.unverified_served, pt.judged_capped, pt.climb_starved
                        ));
                    }
                    // A8 (W8): a pinned walk that completes BLANK during the driven phases. This scene
                    // is deep filaments; a blank walk here is the content guard catching a picture the
                    // walk never reached — and three in a row end an autopilot dive (U57).
                    let blank = self.perf.blank_walks_total[0].wrapping_sub(b.blank);
                    eprintln!("--motiontest: A8 blank walks during the driven phases: {blank}");
                    if blank != 0 {
                        fails.push(format!(
                            "A8: {blank} pinned walk(s) completed BLANK during the driven phases (kept off screen by the content guard)"
                        ));
                    }
                    let hold_moving = self.perf.hold_bound_moving[0].wrapping_sub(b.hold_moving);
                    eprintln!("--motiontest: A6 moving frames shown at the jump floor's clamped scale: {hold_moving}");
                    if hold_moving != 0 {
                        fails.push(format!(
                            "A6: {hold_moving} moving frame(s) showed the held frame at the jump floor's clamped scale (U39: it slides against the view)"
                        ));
                    }
                    // W8: a VACUOUS term alone is not a failure of what this tests, and a real failure
                    // beside one is still a failure (`crate::verdict::code`).
                    let pass = fails.is_empty();
                    let code = crate::verdict::code(
                        fails.iter().any(|f| !f.starts_with("VACUOUS")),
                        fails.iter().any(|f| f.starts_with("VACUOUS")),
                    );
                    self.motiontest_verdict(
                        mt,
                        code,
                        &if pass {
                            "all motion-presentation invariants hold".to_string()
                        } else {
                            fails.join("; ")
                        },
                    );
                }
            }
        }

        self.harness.motiontest = Some(mt);
        ctx.request_repaint();
    }

    /// Exit with `code` (`crate::verdict`: 0 pass, 2 a criterion failed, 3 VACUOUS — the regime was
    /// never reached, so nothing was tested).
    fn motiontest_verdict(&mut self, mt: MotionTest, code: i32, msg: &str) -> ! {
        mt.done.store(true, std::sync::atomic::Ordering::Relaxed);
        let verdict = crate::verdict::word(code);
        crate::diag::log_line("motiontest", &format!("{verdict}: {msg}"));
        eprintln!("--motiontest: VERDICT {verdict}: {msg}");
        crate::exit(code);
    }
}
