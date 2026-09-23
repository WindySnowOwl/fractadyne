//! `--recordtest [FRAMES]` — the W1 gate: does the frame record actually RECORD?
//! (design/live-render-robustness.md §6.6.1 and §7.1.)
//!
//! ⭐⭐**An instrument nobody checks is the failure this whole workstream is about.** The record is
//! credited with diagnosing the next field device loss; if it silently stopped writing — a sink
//! that failed to open, a view whose row is never emitted, a field nobody fills — the first anyone
//! would learn of it is a crash that arrives with nothing attached. So it is checked here, on the
//! real eframe loop, against things the harness counts FOR ITSELF rather than against the
//! record's own account of itself.
//!
//! ⚠The design names this `--selftest record`, but `--selftest` renders offline inside one
//! `update()` call and never drives a live frame, so the check lives in its own harness.
//!
//! RED (exit 1), each on its own:
//!   * **count** — every frame the harness drove has exactly one view-0 record: no gap, no
//!     duplicate, and the sequence numbers are contiguous;
//!   * **fields** — a required field at its unset sentinel (a built view with no `present`, no
//!     resolution, a reading with no source or verdict, a counter reading describing a render from
//!     the future);
//!   * **round trip** — `frames.bin`, read back through a separate handle, is not exactly the ring;
//!   * **abort** — a child process that records and then `abort()`s (`0xc0000409` on Windows, the
//!     death class the panic hook never sees) leaves a `frames.bin` that is short, torn, or
//!     disordered;
//!   * a crash report appears during the run.
//!
//! VACUOUS (exit 2 — never 0, and never the same as a failure): the run recorded no dispatch, no
//! judged reading or no counter reading, or ran under `--set` overrides. A gate that passes
//! having exercised nothing has told you nothing.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::diag::frame_record::{self, present, read_src, verdict, FrameRecord};

/// Frames driven when no count is given: enough for a reference to build, the budget to take
/// readings, and a jump to land and settle.
pub(crate) const DEFAULT_FRAMES: u64 = 240;
/// The child records this many frames and then aborts.
const CHILD_ABORT_AFTER: u64 = 60;
/// How long to wait for the child (window creation + GPU init + its frames) before calling it hung.
const CHILD_TIMEOUT: Duration = Duration::from_secs(180);
/// Depth of the live view (log10 magnification) — on the perturbation path, so a reference is
/// built and readings flow — and the jump taken halfway through.
const DEPTH: f64 = 12.0;
const JUMP_TO: f64 = 13.5;

pub(crate) struct RecordTest {
    frames: u64,
    /// Child mode: abort after this many frames (`--recordtest-abort-after N`).
    abort_after: Option<u64>,
    start_frame: Option<u64>,
    seen: u64,
    jumped_at: Option<u64>,
    child: Option<(Child, PathBuf)>,
    crashes_at_start: Vec<String>,
    started: Instant,
}

impl RecordTest {
    pub(crate) fn new(frames: u64, abort_after: Option<u64>) -> Self {
        Self {
            frames: frames.max(60),
            abort_after,
            start_frame: None,
            seen: 0,
            jumped_at: None,
            child: None,
            crashes_at_start: crate::diag::crash_report_names(),
            started: Instant::now(),
        }
    }
}

/// Launch the abort child: this binary, hermetic (its own config and log dir, so its `frames.bin`
/// is its own), recording until it aborts. stdout/stderr go to files in its dir, never to a pipe
/// nobody drains (a child blocked on a full pipe would read as "hung").
fn spawn_abort_child() -> Result<(Child, PathBuf), String> {
    let dir = std::env::temp_dir().join(format!("fd-recordtest-{:016x}", frame_record::session_id()));
    let cfg = dir.join("config");
    std::fs::create_dir_all(&cfg).map_err(|e| format!("cannot create {}: {e}", cfg.display()))?;
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let out = std::fs::File::create(dir.join("child.out")).map_err(|e| e.to_string())?;
    let err = std::fs::File::create(dir.join("child.err")).map_err(|e| e.to_string())?;
    let child = Command::new(exe)
        .args(["--recordtest", "100000", "--recordtest-abort-after", &CHILD_ABORT_AFTER.to_string()])
        .env("FRACTADYNE_CONFIG_DIR", &cfg)
        .env_remove("FRACTADYNE_LOG_DIR")
        .env("FRACTADYNE_NO_SOUND", "1")
        .stdout(Stdio::from(out))
        .stderr(Stdio::from(err))
        .spawn()
        .map_err(|e| format!("spawn: {e}"))?;
    Ok((child, dir))
}

impl crate::FractadyneApp {
    /// One frame of the record test.
    pub(crate) fn recordtest_frame(&mut self, ctx: &egui::Context) {
        ctx.request_repaint(); // no input arrives; keep the loop turning
        let Some(mut t) = self.harness.recordtest.take() else { return };
        // ⚠This hook runs BEFORE `update` increments `frame_idx`, and a record carries the
        // incremented value — the index the slow-frame line and `build_params` see. So the record
        // THIS update will emit is `frame_idx + 1`. (Found by this harness's own first run: it
        // reported one frame missing, and the record, decoded, was complete from 1 to 239.)
        let this_frame = self.perf.frame_idx + 1;
        if t.start_frame.is_none() {
            self.uitest_set_live(ctx, DEPTH);
            t.start_frame = Some(this_frame);
            if t.abort_after.is_none() {
                match spawn_abort_child() {
                    Ok(c) => t.child = Some(c),
                    Err(e) => eprintln!("[recordtest] could not launch the abort child: {e}"),
                }
            }
            eprintln!(
                "[recordtest] {} frames at 1e{DEPTH}x, a jump to 1e{JUMP_TO}x halfway{}",
                t.frames,
                if t.abort_after.is_some() { " (CHILD: will abort)" } else { "" }
            );
        }
        t.seen += 1;

        // Child: record, then die the way the panic hook never sees. The last frame it RECORDED
        // (this update's is never emitted) is written down first, so the parent knows exactly
        // which records to expect: every frame from 1 — a process's first record — to that one.
        if let Some(n) = t.abort_after {
            if t.seen > n {
                let last = this_frame - 1;
                if let Some(dir) = crate::diag::logs_dir() {
                    let _ = std::fs::write(dir.join("recordtest-abort.txt"), last.to_string());
                }
                eprintln!("[recordtest] child aborting after recording frame {last}");
                std::process::abort();
            }
        }

        if t.jumped_at.is_none() && t.seen >= t.frames / 2 {
            self.uitest_set_live(ctx, JUMP_TO);
            t.jumped_at = Some(this_frame);
        }

        if t.seen >= t.frames && t.abort_after.is_none() {
            let code = self.recordtest_finish(&mut t, this_frame);
            crate::exit(code);
        }
        self.harness.recordtest = Some(t);
    }

    /// Verdict + exit code: 0 pass, 1 fail, 2 vacuous.
    fn recordtest_finish(&self, t: &mut RecordTest, this_frame: u64) -> i32 {
        let f0 = t.start_frame.unwrap_or(1);
        // This update's record is emitted at the END of this `update`, so the frames the harness
        // can hold the record to are [f0, f1).
        let f1 = this_frame;
        let recs = frame_record::snapshot();
        let mut fails: Vec<String> = Vec::new();
        let mut notes: Vec<String> = Vec::new();

        // ---- count: one view-0 record per driven frame; contiguous sequence numbers.
        if recs.windows(2).any(|w| w[1].seq != w[0].seq + 1) {
            fails.push("the ring's sequence numbers are not contiguous".into());
        }
        let v0: Vec<&FrameRecord> = recs.iter().filter(|r| r.view == 0 && r.frame >= f0 && r.frame < f1).collect();
        let expected = f1.saturating_sub(f0);
        let mut frames: Vec<u64> = v0.iter().map(|r| r.frame).collect();
        frames.sort_unstable();
        let dups = frames.windows(2).filter(|w| w[0] == w[1]).count();
        frames.dedup();
        let missing = expected.saturating_sub(frames.len() as u64);
        if expected as usize > frame_record::RING_LEN {
            notes.push(format!("{expected} frames exceed the {}-record ring; only the newest are held", frame_record::RING_LEN));
        } else if missing > 0 || dups > 0 {
            fails.push(format!(
                "view 0 has {} records for {expected} frames [{f0},{f1}): {missing} missing, {dups} duplicated",
                v0.len()
            ));
        }

        // ---- fields: nothing required left at its unset sentinel.
        let mut field_fails = 0usize;
        let mut first_bad: Option<String> = None;
        for r in &recs {
            let mut bad: Vec<&str> = Vec::new();
            if r.schema != frame_record::SCHEMA {
                bad.push("schema");
            }
            if r.view == 0 && r.frame >= f0 && r.frame < f1 && r.plan_calls == 0 {
                bad.push("plan_calls=0 on a driven frame (view 0 not built)");
            }
            if r.plan_calls > 0 {
                if !matches!(r.present, present::LIVE | present::REPROJECT | present::HOLD) {
                    bad.push("present unset");
                }
                if r.res_w == 0 || r.res_h == 0 {
                    bad.push("resolution unset");
                }
                if r.ss == 0 {
                    bad.push("ss unset");
                }
            }
            if r.read_n > 0 {
                if !matches!(r.read_src, read_src::GPU | read_src::WALL) {
                    bad.push("read_src unset");
                }
                if !matches!(r.read_verdict, verdict::DISCARDED | verdict::MOVED | verdict::UNCHANGED) {
                    bad.push("read_verdict unset");
                }
            }
            if r.ctr_new {
                if r.ctr_px == 0 {
                    bad.push("counter reading with no pixels");
                }
                if r.ctr_tag > r.frame {
                    bad.push("counter reading describes a FUTURE render");
                }
            }
            if !bad.is_empty() {
                field_fails += 1;
                first_bad.get_or_insert_with(|| format!("f={} v{}: {}", r.frame, r.view, bad.join(", ")));
            }
        }
        if recs.windows(2).any(|w| w[1].t_ms < w[0].t_ms) {
            fails.push("t_ms runs backwards".into());
        }
        if field_fails > 0 {
            fails.push(format!("{field_fails} record(s) with a required field unset — first: {}", first_bad.unwrap_or_default()));
        }

        // ---- round trip: frames.bin, read back independently, must be exactly the ring.
        match crate::diag::logs_dir().map(|d| d.join("frames.bin")) {
            None => fails.push("no logs directory, so no frames.bin (is FRACTADYNE_LOG=0 set?)".into()),
            Some(p) => match frame_record::read_bin(&p) {
                Err(e) => fails.push(format!("frames.bin unreadable: {e}")),
                Ok(None) => fails.push("frames.bin has the wrong magic or schema".into()),
                Ok(Some(b)) => {
                    if b.session != frame_record::session_id() {
                        fails.push("frames.bin belongs to another session".into());
                    } else if b.torn > 0 {
                        fails.push(format!("frames.bin has {} torn slot(s) in a live process", b.torn));
                    } else if b.records != recs {
                        let diff = b.records.iter().zip(&recs).position(|(a, b)| a != b);
                        fails.push(format!(
                            "frames.bin ({} records) is not the ring ({}); first difference at index {:?}",
                            b.records.len(),
                            recs.len(),
                            diff
                        ));
                    }
                    if !b.header.contains("\"adapter\"") {
                        fails.push("frames.bin carries no session header (adapter line never set)".into());
                    }
                }
            },
        }

        // ---- abort: the child's frames.bin after a process abort.
        match t.child.take() {
            None => fails.push("the abort child was never launched".into()),
            Some((mut child, dir)) => {
                let deadline = Instant::now() + CHILD_TIMEOUT;
                let status = loop {
                    match child.try_wait() {
                        Ok(Some(s)) => break Some(s),
                        Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(200)),
                        _ => {
                            let _ = child.kill();
                            break None;
                        }
                    }
                };
                let logs = dir.join("config").join("logs");
                let abort_at = std::fs::read_to_string(logs.join("recordtest-abort.txt"))
                    .ok()
                    .and_then(|s| s.trim().parse::<u64>().ok());
                match (status, abort_at) {
                    (None, _) => fails.push(format!("the abort child did not finish in {}s", CHILD_TIMEOUT.as_secs())),
                    (Some(s), _) if s.success() => fails.push("the abort child exited cleanly instead of aborting".into()),
                    (Some(_), None) => fails.push(format!("the abort child died without reaching its abort (see {})", dir.display())),
                    (Some(s), Some(at)) => match frame_record::read_bin(&logs.join("frames.bin")) {
                        Ok(Some(b)) => {
                            let got: Vec<u64> = b.records.iter().filter(|r| r.view == 0).map(|r| r.frame).collect();
                            let want: Vec<u64> = (1..=at).collect();
                            if b.torn > 0 {
                                fails.push(format!("the aborted child's frames.bin has {} torn slot(s)", b.torn));
                            }
                            if got != want {
                                fails.push(format!(
                                    "the aborted child's frames.bin holds {} view-0 records ({:?}..{:?}), expected exactly frames 1..={at}",
                                    got.len(),
                                    got.first(),
                                    got.last()
                                ));
                            } else {
                                notes.push(format!(
                                    "abort child: died with {s} at frame {at}; its frames.bin held all {at} records, untorn"
                                ));
                            }
                        }
                        Ok(None) => fails.push("the aborted child's frames.bin has the wrong magic or schema".into()),
                        Err(e) => fails.push(format!("the aborted child left no readable frames.bin: {e}")),
                    },
                }
                if fails.is_empty() {
                    let _ = std::fs::remove_dir_all(&dir);
                }
            }
        }

        // ---- cost: the record's own overhead, measured, against the design's exit criterion
        // (under 0.5% of a frame). `rec_us` is the previous frame's emit, timed in process — a
        // direct measurement resolving microseconds, where an A/B of whole-frame times has a noise
        // floor of milliseconds and could only have said "not seen".
        //
        // ⚠Judged against a 60 Hz frame, NOT this run's own median. This harness draws an idle
        // view with nothing throttling it, so its frames run ~1.1 ms (~900 fps) — a rate no user
        // sees, against which the first version of this check read 1.5–2.3% and went red while
        // the absolute cost was 16 µs. The share of the run's own median is still printed. And the
        // p99, not the median, is what is held to the limit. The FIRST emit — which creates the
        // 2 MB `frames.bin` and allocates the ring, once per session — is reported on its own.
        const REFERENCE_FRAME_MS: f64 = 1000.0 / 60.0;
        let by_seq = |s: u64| recs.iter().find(|r| r.seq == s).map(|r| r.rec_us);
        let first_emit = by_seq(1);
        let mut cost: Vec<f32> = recs.iter().filter(|r| r.seq >= 2).map(|r| r.rec_us).filter(|u| *u > 0.0).collect();
        let mut dts: Vec<f64> = v0.iter().map(|r| r.last_dt_ms).filter(|d| *d > 0.0).collect();
        cost.sort_by(f32::total_cmp);
        dts.sort_by(f64::total_cmp);
        let pct = |v: &[f32], p: f64| v.get(((v.len() as f64 - 1.0) * p).round() as usize).copied().unwrap_or(0.0);
        if cost.is_empty() || dts.is_empty() {
            fails.push("no emit cost or frame interval was recorded".into());
        } else {
            let (p50, p99, max) = (pct(&cost, 0.5), pct(&cost, 0.99), *cost.last().unwrap());
            let median_dt = dts[dts.len() / 2];
            let share = p99 as f64 / 1000.0 / REFERENCE_FRAME_MS;
            notes.push(format!(
                "record cost per frame: p50 {p50:.1} µs, p99 {p99:.1} µs, max {max:.1} µs — p99 is {:.3}% of a 60 Hz frame \
                 ({:.2}% of this run's uncapped median of {median_dt:.2} ms); first emit (opens frames.bin) {:.0} µs",
                share * 100.0,
                p50 as f64 / 10.0 / median_dt,
                first_emit.unwrap_or(0.0)
            ));
            if share > 0.005 {
                fails.push(format!("the record's p99 costs {:.2}% of a 60 Hz frame (limit 0.5%)", share * 100.0));
            }
        }

        let new_crashes: Vec<String> = crate::diag::crash_report_names()
            .into_iter()
            .filter(|n| !t.crashes_at_start.contains(n))
            .collect();
        if !new_crashes.is_empty() {
            fails.push(format!("crash report(s) appeared: {}", new_crashes.join(", ")));
        }

        // ---- richness: a run that exercised nothing is VACUOUS, never a pass.
        let n_dispatch = v0.iter().filter(|r| r.dispatched).count();
        let n_read = recs.iter().filter(|r| r.read_n > 0).count();
        let n_ctr = recs.iter().filter(|r| r.ctr_new).count();
        let n_other = v0.iter().filter(|r| r.present != present::LIVE).count();
        let mut vacuous: Vec<String> = Vec::new();
        if n_dispatch == 0 {
            vacuous.push("no frame dispatched".into());
        }
        if n_read == 0 {
            vacuous.push("no reading was judged by the budget controller".into());
        }
        if n_ctr == 0 {
            vacuous.push("no counter reading arrived".into());
        }
        if !crate::tunables::is_stock() {
            vacuous.push(format!("tunables overridden ({})", crate::tunables::status_line()));
        }

        eprintln!(
            "\n=== --recordtest: {} frames in {:.1}s — {} records held, view 0: {} dispatching, {} not live; \
             {n_read} judged readings, {n_ctr} counter readings; jump at frame {:?} ===",
            expected,
            t.started.elapsed().as_secs_f64(),
            recs.len(),
            n_dispatch,
            n_other,
            t.jumped_at
        );
        for n in &notes {
            eprintln!("recordtest: {n}");
        }
        if !fails.is_empty() {
            for f in &fails {
                eprintln!("recordtest: FAIL — {f}");
            }
            eprintln!("recordtest: FAIL");
            1
        } else if !vacuous.is_empty() {
            for v in &vacuous {
                eprintln!("recordtest: VACUOUS — {v}");
            }
            eprintln!("recordtest: VACUOUS (exit 2) — this run cannot vouch for the record of the shipped build (see above)");
            2
        } else {
            eprintln!("recordtest: PASS — every driven frame recorded once, every required field set, frames.bin exact, and it survived an abort");
            0
        }
    }
}
