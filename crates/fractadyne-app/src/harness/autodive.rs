//! `--autodive`: the autopilot's dive, driven unpaced from the command line to hammer the
//! frame-cost controller, and judged from the frame record. Moved from `autopilot.rs`.

use eframe::egui;

/// CLI `--autodive`: drive the autopilot dive from the command line so the frame-cost controller can
/// be hammered UNPACED, and report whether the dangerous regime was actually reached.
///
/// This exists because the device-loss class could not be reproduced automatically. Every scripted
/// attempt went through `--play`, and a tour clock DILATES on a slow frame, so pressure never
/// accumulates: measured on an RTX 3080, `repro-e28-crossover` peaks at ~195 ms of measured iterate
/// against a 900 ms lethal band, and twelve arms across two nights produced zero lethal readings
/// while a human diving by hand produced 21 in one evening.
///
/// The autopilot is what a hand-driven dive *is* — it runs in the normal update loop with no clock,
/// and because it re-targets the detail-richest region it dives INTO structure, which is exactly
/// where per-step cost explodes. So this is a thin wrapper over `toggle_autopilot`, not a new
/// harness. Ruled out first, so nobody re-treads it: `--frametest` drives `build_params` and
/// reference recompute but never prices a frame (one `fd-gpu` line per run, "no reading"), and
/// `--livetest` carries its own copy of the controller.
pub(crate) struct AutoDive {
    target_log10: f64,
    timeout_s: f64,
    /// Explicit iteration count, or `None` for auto-iter.
    ///
    /// ⚠**Defaults to EXPLICIT, and that is the whole difference between a harness that reproduces
    /// and one that cannot.** The first version forced `auto_iter = true`, reasoning that both field
    /// losses ran with auto-iter on. That was wrong in a way that made the harness useless: the LIVE
    /// path deliberately caps auto-iter lower than the export appetite for responsiveness, so every
    /// frame was cheap by construction. Measured on the RX 6800 XT: 900 s of diving reached 1e112x
    /// with a peak measured iterate of **19.5 ms** against a 900 ms lethal band — lower even than the
    /// paced tour it was built to replace.
    ///
    /// The crash manifest says what the regime actually needs:
    /// `iter=250000 (gpu_iter=250000, eff=250000, boost=1.00)` with the budget ceiling at `6.000e10`
    /// — the EXPLICIT ceiling. An explicit high count with auto-iter off is what makes a deep frame
    /// expensive enough to reach the band, so that is the default here.
    iter: Option<u32>,
    t0: std::time::Instant,
    /// Frames to let the GPU and first reference come up before diving.
    warmup: u32,
    armed: bool,
    /// Last sampled measurement, to notice when a NEW one lands.
    ///
    /// ⚠Paired with the step count, because ms ALONE undercounts. Field run 2026-08-18 on the
    /// RX 6800 XT logged four LETHAL-BAND frames and this harness reported three: two consecutive
    /// readings were both exactly 1155 ms, and a value-only comparison cannot tell a repeat from a
    /// stale sample. Two identical (ms, steps) pairs back to back remain indistinguishable, so the
    /// figure is reported as DISTINCT readings rather than claimed as a total.
    last_ms: f64,
    last_steps: u64,
    readings: u32,
    lethal: u32,
    peak_ms: f64,
    deepest_log10: f64,
    /// Has the explicit iteration count been applied yet? See `EXPLICIT_FROM_LOG10`.
    iter_applied: bool,
    /// Remaining dive→Home cycles. **THE ZOOM-HOME GLIDE IS THE STRESS TEST**, and it took three
    /// failed harness designs to see it.
    ///
    /// A monotonic dive cannot stress the frame-cost controller: measured on the RX 6800 XT, diving
    /// to 1e150x produced a peak measured iterate of 36 ms against a 400 ms target, with frames
    /// CPU-bound (body 200-639 ms against 1-2 ms of wait). The controller was STARVED, not stressed —
    /// and a controller that only ever sees 36 ms frames never grows into danger. Smooth progressions
    /// let it adapt continuously, which is it working correctly.
    ///
    /// Home is the opposite: one continuous sweep from extreme depth to 1x, crossing every mode
    /// boundary while the budget is still calibrated for the deep regime. That is exactly the shape of
    /// both field losses — `mode switch to 2: budget 8.31e10 -> bootstrap 2.45e8` followed by a frame
    /// ALREADY SIZED by the old regime going out at 1.146e11 steps (405x over budget, 3040 ms), and
    /// `bla_skip` collapsing 4,457,481 -> 9,343 so per-step cost jumped ~100x under a stale budget.
    /// It is also literally the button that killed the device on 2026-08-18.
    home_left: u32,
    /// A Home glide is in flight (peaks during it are tracked separately — see `peak_home_ms`).
    homing: bool,
    homes_done: u32,
    /// Peak measured iterate observed DURING a Home glide, so the report can say whether Home is
    /// harder than the dive rather than leaving it to inference.
    peak_home_ms: f64,
    /// Cleared by the normal exit paths so the watchdog thread stands down. See `new`.
    done: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// CURRENT depth, distinct from `deepest_log10`, which is a high-water mark.
    ///
    /// ⚠Using the high-water mark for the explicit-count switch broke multi-cycle runs: after a Home
    /// glide the view is back at 1x but `deepest_log10` still reads 30, so the switch fired instantly
    /// and the re-dive started at 1x with an explicit 250k — the all-interior case where the
    /// autopilot's target search returns None and the dive dies on the spot. Measured: 2 cycles
    /// requested, 1 completed, then a timeout doing nothing.
    current_log10: f64,
    /// W8: the frame record, read over the WHOLE run (`frame_record::since`), and what it says.
    rec_next: u64,
    tally: DiveTally,
}

/// What the frame record says about a whole `--autodive` run (W8, design §7.8): the verdict reads
/// THIS — readings in the lethal band, the retreats they caused and what those reached, and the
/// wall-clock intervals that followed — not the last iterate time the harness happened to sample.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct DiveTally {
    /// Readings in the lethal band (`read_lethal`).
    pub(crate) lethal: u32,
    /// Emergency retreats that made the next dispatch smaller, and ones that reached nothing
    /// (`retreat` 1 and 2; `render::RetreatOutcome`).
    pub(crate) applied: u32,
    pub(crate) unreached: u32,
    /// Frames whose wall interval fell in the lethal band.
    pub(crate) wall_slow: u32,
    /// The longest run of lethal-band intervals after a retreat applied, and the current run.
    pub(crate) slow_run_after: u32,
    run: u32,
    /// Records the ring had overwritten before they were read (0: the whole run was judged).
    pub(crate) lost: u64,
}

/// Lethal-band intervals in a row after a retreat that still count as the retreat holding: the
/// dispatch already queued when the budget came down runs anyway. Measured on the RX 6800 XT (the
/// three beta.132 AFTER arms, 2026-09-27): one ~1,400 ms frame after each derate, then 92–105 ms.
pub(crate) const SLOW_AFTER_RETREAT_MAX: u32 = 2;

impl DiveTally {
    /// Fold one frame record of view 0.
    pub(crate) fn feed(&mut self, last_dt_ms: f64, retreat: u8, read_lethal: bool, lethal_ms: f64) {
        self.lethal += read_lethal as u32;
        match retreat {
            1 => self.applied += 1,
            2 => self.unreached += 1,
            _ => {}
        }
        if last_dt_ms >= lethal_ms {
            self.wall_slow += 1;
            if self.applied > 0 {
                self.run += 1;
                self.slow_run_after = self.slow_run_after.max(self.run);
            }
        } else {
            self.run = 0;
        }
    }
}

/// The verdict on a whole `--autodive` run (`crate::verdict`): VACUOUS when the regime was never
/// reached; RED when it was and a retreat reached nothing, none applied, or the lethal intervals
/// kept coming after one (the budget did not move); PASS only if the record covered every frame —
/// otherwise VACUOUS too, since what was lost to the ring is exactly what was not judged.
pub(crate) fn autodive_verdict(t: &DiveTally) -> (i32, String) {
    if t.lethal == 0 && t.applied + t.unreached == 0 {
        return (
            crate::harness::verdict::VACUOUS,
            "DID NOT REACH THE REGIME - no lethal reading, so nothing was tested".to_string(),
        );
    }
    let mut red = Vec::new();
    if t.unreached > 0 {
        red.push(format!("{} retreat(s) REACHED NOTHING", t.unreached));
    }
    if t.applied == 0 {
        red.push("no retreat applied".to_string());
    }
    if t.slow_run_after > SLOW_AFTER_RETREAT_MAX {
        red.push(format!(
            "{} lethal-band intervals in a row after a retreat (more than {SLOW_AFTER_RETREAT_MAX}: the budget did not move)",
            t.slow_run_after
        ));
    }
    if red.is_empty() && t.lost > 0 {
        return (
            crate::harness::verdict::VACUOUS,
            format!("{} frame record(s) LOST to the ring before they were read - the run was not judged whole", t.lost),
        );
    }
    if red.is_empty() {
        (
            crate::harness::verdict::PASS,
            format!(
                "REACHED THE REGIME AND THE RETREAT HELD: {} lethal reading(s), {} retreat(s) applied, {} lethal-band interval(s), at most {} in a row after a retreat",
                t.lethal, t.applied, t.wall_slow, t.slow_run_after
            ),
        )
    } else {
        (crate::harness::verdict::RED, format!("REACHED THE REGIME AND THE RETREAT DID NOT HOLD: {}", red.join("; ")))
    }
}

#[cfg(test)]
#[path = "../../tests/unit/harness/autodive/dive_verdict_tests.rs"]
mod dive_verdict_tests;

/// Depth at which `--autodive` switches from auto-iter to its explicit iteration count.
///
/// ⚠It CANNOT be applied from frame one, and finding that out cost a run. The autopilot picks its
/// next target by looking for detail variance; with an explicit 250,000 at shallow depth the view is
/// essentially all-interior, the evaluator returns `None`, and the autopilot deactivates on the spot
/// (`autopilot.rs`, the `None =>` arm). Measured: the dive "finished" in 13.9 s at 1e0.8x having gone
/// nowhere. So dive shallow on auto-iter, where targeting works, and switch to the expensive explicit
/// count once there is real structure to aim at — which is also the honest reproduction of the field
/// case, where the user zoomed deep first and the big count was set at depth.
const EXPLICIT_FROM_LOG10: f64 = 6.0;

impl AutoDive {
    pub(crate) fn new(target_log10: f64, timeout_s: f64, iter: Option<u32>, home_cycles: u32) -> Self {
        // ⚠**THE TIMEOUT NEEDS ITS OWN THREAD.** Every other check in this harness runs inside
        // `autodive_frame`, i.e. inside `update()` — so `--autodive-timeout` could only fire while
        // frames were still being DELIVERED, which is exactly not the case that needs bounding. A
        // deep reference build runs arbitrary-precision on the main thread (hundreds of bits at
        // 1e150), and the historical "present wedge" blocks the main thread inside a wgpu wait
        // forever; both look identical from outside — the app reports "idle" with a frozen zoom, and
        // nothing enforced the deadline. Reported from the field 2026-08-18 at 1e149.
        //
        // The watchdog is deliberately dumb: sleep, check a flag, and hard-exit if the deadline
        // passes. It gets a grace margin so that whenever frames ARE flowing the in-frame path wins
        // and prints the full summary; the watchdog only ever speaks when nothing else can.
        //
        // ⚠This is why `--torture live/home/glide-from-depth` was the trustworthy way to run it
        // before now: the supervisor bounds each rung as a CHILD PROCESS from outside. Fixing the
        // flag does not make that redundant, it makes the two agree.
        const WATCHDOG_GRACE_S: f64 = 10.0;
        let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        {
            let done = std::sync::Arc::clone(&done);
            let budget = timeout_s + WATCHDOG_GRACE_S;
            std::thread::spawn(move || {
                let start = std::time::Instant::now();
                while start.elapsed().as_secs_f64() < budget {
                    if done.load(std::sync::atomic::Ordering::Relaxed) {
                        return;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(250));
                }
                if done.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }
                // ⚠One line on purpose. A `\` line-continuation inside a Rust string literal does
                // NOT strip the following indentation in this CRLF repo, so the message reaches the
                // user with a long run of spaces in the middle. Three messages shipped that way today
                // before it was worth writing down; see CONTRIBUTING.
                let msg = format!(
                    "WATCHDOG: no completion {budget:.0}s after the {timeout_s:.0}s timeout — the frame loop stopped delivering (deep reference build on the main thread, or a wedge). Nothing was measured."
                );
                crate::diag::log_line("autodive", &msg);
                eprintln!();
                eprintln!("--autodive: {msg}");
                crate::exit(4);
            });
        }
        Self {
            done,
            target_log10,
            timeout_s,
            iter,
            t0: std::time::Instant::now(),
            // Frames to let the GPU and first reference come up. `FRACTADYNE_AUTODIVE_WARMUP`
            // lengthens it so a dive can start from a view that has fully settled (and finished
            // its on-settle supersampling) — the state a user's dive starts from when they look
            // at a view before pressing A, which a 45-frame warmup does not reproduce.
            warmup: std::env::var("FRACTADYNE_AUTODIVE_WARMUP")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(45),
            armed: false,
            last_ms: 0.0,
            last_steps: 0,
            readings: 0,
            lethal: 0,
            peak_ms: 0.0,
            deepest_log10: 0.0,
            iter_applied: false,
            home_left: home_cycles,
            homing: false,
            homes_done: 0,
            peak_home_ms: 0.0,
            current_log10: 0.0,
            rec_next: 0,
            tally: DiveTally::default(),
        }
    }
}

impl crate::FractadyneApp {
    /// One frame of `--autodive`. Same in-loop shape as `uitest_frame` / `juliadive_frame`.
    pub(crate) fn autodive_frame(&mut self, ctx: &egui::Context) {
        const LOG2_10: f64 = 3.321_928_094_887_362;
        let Some(mut d) = self.harness.autodive.take() else { return };

        // Sample the controller's own measurement rather than scraping the log. `gpu_iterate=` in
        // the log only ever appears INSIDE the lethal message, so log-scraping is circular — it can
        // only report a number once the thing being detected has already happened.
        let ms = self.perf.last_iterate_ms[0];
        let steps = self.perf.fe_steps_last[0];
        if ms > 0.0 && ((ms - d.last_ms).abs() > f64::EPSILON || steps != d.last_steps) {
            d.readings += 1;
            if ms > d.peak_ms {
                d.peak_ms = ms;
            }
            if ms >= crate::tunables::cost().tdr_lethal_ms {
                d.lethal += 1;
            }
            if d.homing && ms > d.peak_home_ms {
                d.peak_home_ms = ms;
            }
            d.last_ms = ms;
            d.last_steps = steps;
        }
        // W8: the record of every frame since the last one read — the verdict's evidence.
        {
            let (recs, next, lost) = crate::diag::frame_record::since(d.rec_next);
            d.rec_next = next;
            d.tally.lost += lost;
            let lethal_ms = crate::tunables::cost().tdr_lethal_ms;
            for r in recs.iter().filter(|r| r.kind == crate::diag::frame_record::kind::FRAME && r.view == 0) {
                d.tally.feed(r.last_dt_ms, r.retreat, r.read_n > 0 && r.read_lethal, lethal_ms);
            }
        }
        let depth = self.viewport.log2_magnification() / LOG2_10;
        if depth.is_finite() {
            d.current_log10 = depth;
            if depth > d.deepest_log10 {
                d.deepest_log10 = depth;
            }
        }

        if !d.armed {
            if d.warmup > 0 {
                d.warmup -= 1;
            } else {
                // Field conditions, from the crash manifest: an EXPLICIT iteration count with
                // auto-iter OFF. See the note on `AutoDive::iter` for why auto-iter is the wrong
                // choice here despite the losses having run with it.
                // Always start on auto-iter so the autopilot's target evaluation has detail to
                // find; the explicit count lands at EXPLICIT_FROM_LOG10 (see that note).
                self.render_cfg.auto_iter = true;
                self.autopilot.dive_log2 = d.target_log10 * LOG2_10;
                self.toggle_autopilot(ctx);
                d.armed = true;
                crate::diag::log_line(
                    "autodive",
                    &format!(
                        "diving to 1e{:.0}x ({}, lethal band >= {:.0}ms)",
                        d.target_log10,
                        match d.iter {
                            Some(n) => format!("explicit iter={n}"),
                            None => "auto-iter".to_string(),
                        },
                        crate::tunables::cost().tdr_lethal_ms
                    ),
                );
            }
        }

        // Switch to the explicit count once the dive has structure to aim at.
        if d.armed && !d.iter_applied && d.current_log10 >= EXPLICIT_FROM_LOG10 {
            if let Some(n) = d.iter {
                self.render_cfg.max_iter = n.clamp(64, crate::MAX_ITER_LIMIT);
                self.render_cfg.auto_iter = false;
                crate::diag::log_line(
                    "autodive",
                    &format!("1e{:.1}x reached — switching to explicit iter={n}", d.current_log10),
                );
            }
            d.iter_applied = true;
        }

        // Test hook: freeze the frame loop on purpose so the watchdog above can be OBSERVED firing.
        // A safety net nobody has seen catch anything is a guess.
        if std::env::var("FRACTADYNE_AUTODIVE_FREEZE").is_ok() && d.armed {
            crate::diag::log_line("autodive", "FRACTADYNE_AUTODIVE_FREEZE — blocking the frame loop");
            loop {
                std::thread::sleep(std::time::Duration::from_secs(3600));
            }
        }

        let elapsed = d.t0.elapsed().as_secs_f64();
        let dive_done = d.armed && !self.autopilot.active;
        let timed_out = elapsed >= d.timeout_s;

        // ---- dive → Home cycling: the Home glide is the part that actually stresses the controller.
        if !timed_out && d.homing {
            if self.home_anim.is_none() {
                d.homes_done += 1;
                d.homing = false;
                crate::diag::log_line(
                    "autodive",
                    &format!(
                        "home glide {} complete (peak during home {:.1}ms)",
                        d.homes_done, d.peak_home_ms
                    ),
                );
                if d.home_left > 0 {
                    // Re-dive. Back to auto-iter first: at 1x an explicit 250k makes the view
                    // all-interior and the autopilot's target search returns None immediately (the
                    // 1e0.8x non-dive). The explicit count re-lands at EXPLICIT_FROM_LOG10.
                    self.render_cfg.auto_iter = true;
                    d.iter_applied = false;
                    self.autopilot.dive_log2 = d.target_log10 * LOG2_10;
                    self.toggle_autopilot(ctx);
                }
            }
            self.harness.autodive = Some(d);
            ctx.request_repaint();
            return;
        }
        if !timed_out && dive_done && d.home_left > 0 {
            d.home_left -= 1;
            d.homing = true;
            let now = ctx.input(|i| i.time);
            crate::diag::log_line(
                "autodive",
                &format!("1e{:.1}x reached — ZOOM HOME (the stress test)", d.current_log10),
            );
            self.zoom_home(now);
            self.harness.autodive = Some(d);
            ctx.request_repaint();
            return;
        }

        if dive_done || timed_out {
            // ⚠Distinguish "reached the target" from "the autopilot quit on us". The first version
            // called any inactive autopilot a "dive limit reached", which reported success at 1e0.8x
            // against a 1e45 target — a verdict that lied in the direction of looking fine.
            let reached = d.deepest_log10 >= d.target_log10 - 0.5;
            let _ = dive_done;
            let why = if timed_out {
                "timeout"
            } else if reached {
                "dive limit reached"
            } else {
                "AUTOPILOT STOPPED EARLY (no detail target, or interrupted) — dive did not finish"
            };
            // The verdict, from the frame record over the whole run (W8): a run that never reached the
            // regime is VACUOUS, never a pass — a green exit with zero lethal readings would mean the
            // experiment did not run, not that the code is safe — and one that reached it passes only
            // if every retreat held.
            let (code, verdict) = autodive_verdict(&d.tally);
            crate::diag::log_line(
                "autodive",
                &format!(
                    "{why} after {elapsed:.1}s | deepest 1e{:.1}x | {} reading(s), peak measured \
                     iterate {:.1}ms | {verdict}",
                    d.deepest_log10, d.readings, d.peak_ms
                ),
            );
            println!();
            println!("--autodive: {why} after {elapsed:.1}s");
            println!("  deepest depth      1e{:.1}x", d.deepest_log10);
            println!("  iteration ask      {}",
                match d.iter { Some(n) => format!("{n} (explicit)"), None => "auto".to_string() });
            println!("  controller readings {}", d.readings);
            println!("  peak measured iterate {:.1}ms (lethal band >= {:.0}ms)",
                d.peak_ms, crate::tunables::cost().tdr_lethal_ms);
            println!("  home glides         {} (peak during home {:.1}ms)",
                d.homes_done, d.peak_home_ms);
            println!("  lethal readings     {} (distinct, sampled) · {} in the frame record", d.lethal, d.tally.lethal);
            println!(
                "  retreats            {} applied, {} reached nothing · lethal-band intervals {} (at most {} in a row after a retreat){}",
                d.tally.applied,
                d.tally.unreached,
                d.tally.wall_slow,
                d.tally.slow_run_after,
                if d.tally.lost > 0 { format!(" · {} record(s) LOST to the ring", d.tally.lost) } else { String::new() }
            );
            println!("  {verdict}");
            println!("--autodive: VERDICT {}", crate::harness::verdict::word(code));
            d.done.store(true, std::sync::atomic::Ordering::Relaxed);
            crate::exit(code);
        }

        self.harness.autodive = Some(d);
        ctx.request_repaint();
    }
}
