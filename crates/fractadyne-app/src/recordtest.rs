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
//!   * **heartbeat** — the harness wedges its own UI thread for [`WEDGE`] and the watchdog thread
//!     writes no STALL row of at least 10 s naming the last frame recorded;
//!   * **blind tripwire** — [`BLIND_FRAMES`] frames made slow on the wall while the GPU work stays
//!     short do not trip `⚠FRAME BUDGET IS BLIND` for view 0, in the log and in the record;
//!   * **frames.jsonl** — no header for this session, a row with the wrong keys, fewer summary
//!     rows than elapsed seconds, summaries counting frames that were never recorded, or no event
//!     row at all (the wedge guarantees a slow frame and a stall); its writer thread not answering
//!     a flush, or dropping a row;
//!   * **cost** — the record's p99 per-frame cost exceeds 0.5% of a 60 Hz frame (with the logs on
//!     a network share an excess is VACUOUS instead: that measures the share, not the record);
//!   * **log budget** — any log category ran above 10 lines/s over a 5 s window (design W10);
//!   * a crash report appears during the run.
//!
//! VACUOUS (exit 2 — never 0, and never the same as a failure): the run recorded no dispatch, no
//! judged reading or no counter reading, or ran under `--set` overrides. A gate that passes
//! having exercised nothing has told you nothing.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::diag::frame_record::{self, kind, present, read_src, verdict, FrameRecord};

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
/// A deliberate wedge of the UI thread, three quarters through: long enough for the watchdog
/// (which checks every 2 s and warns past 10 s) to write its STALL row. The heartbeat is the one
/// writer that is not the UI thread, and it is gated here by making the UI thread actually stop.
const WEDGE: Duration = Duration::from_secs(13);
/// The budget-blind shape in miniature, a quarter of the way through: this many consecutive frames
/// made slow on the WALL (a sleep past the 400 ms target, with a repaint asked for) while the GPU
/// work itself stays short, so no reading the controller judges slow can arrive. The tripwire must
/// fire for view 0 — the live wiring the pure `budget_blind` tests cannot see (finding U15).
const BLIND_FRAMES: u64 = 10;
const BLIND_SLEEP: Duration = Duration::from_millis(450);

pub(crate) struct RecordTest {
    frames: u64,
    /// Child mode: abort after this many frames (`--recordtest-abort-after N`).
    abort_after: Option<u64>,
    start_frame: Option<u64>,
    seen: u64,
    jumped_at: Option<u64>,
    wedged_at: Option<u64>,
    blind_from: Option<u64>,
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
            wedged_at: None,
            blind_from: None,
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

        if t.abort_after.is_none() && t.seen >= t.frames / 4 {
            let from = *t.blind_from.get_or_insert(this_frame);
            if this_frame < from + BLIND_FRAMES {
                std::thread::sleep(BLIND_SLEEP);
            }
        }
        if t.jumped_at.is_none() && t.seen >= t.frames / 2 {
            self.uitest_set_live(ctx, JUMP_TO);
            t.jumped_at = Some(this_frame);
        }
        if t.abort_after.is_none() && t.wedged_at.is_none() && t.seen >= t.frames * 3 / 4 {
            t.wedged_at = Some(this_frame);
            eprintln!("[recordtest] wedging the UI thread for {}s at frame {this_frame} (the watchdog must record it)", WEDGE.as_secs());
            std::thread::sleep(WEDGE);
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
        let v0: Vec<&FrameRecord> = recs
            .iter()
            .filter(|r| r.kind == kind::FRAME && r.view == 0 && r.frame >= f0 && r.frame < f1)
            .collect();
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
            if !matches!(r.kind, kind::FRAME | kind::STALL) {
                bad.push("kind unset");
            }
            if r.kind == kind::FRAME && r.view == 0 && r.frame >= f0 && r.frame < f1 && r.plan_calls == 0 {
                bad.push("plan_calls=0 on a driven frame (view 0 not built)");
            }
            if r.kind == kind::STALL && r.stall_ms == 0 {
                bad.push("a stall row with no duration");
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

        // ---- heartbeat: the deliberate wedge must have produced a STALL row from the watchdog.
        let stalls: Vec<&FrameRecord> = recs.iter().filter(|r| r.kind == kind::STALL).collect();
        match (t.wedged_at, stalls.iter().max_by_key(|r| r.stall_ms)) {
            (None, _) => fails.push("the wedge never ran, so the heartbeat was not tested".into()),
            (Some(w), None) => fails.push(format!(
                "the UI thread was wedged {}s at frame {w} and the watchdog wrote no stall row",
                WEDGE.as_secs()
            )),
            (Some(w), Some(s)) => {
                if s.stall_ms < 10_000 {
                    fails.push(format!("the longest stall row says {} ms for a {}s wedge", s.stall_ms, WEDGE.as_secs()));
                } else if s.frame + 1 != w {
                    fails.push(format!(
                        "the stall row names frame {} as the last recorded, but the wedge began before frame {w}'s emit",
                        s.frame
                    ));
                } else {
                    notes.push(format!(
                        "heartbeat: {} stall row(s) for the {}s wedge, longest {} ms, naming frame {} as the last recorded",
                        stalls.len(),
                        WEDGE.as_secs(),
                        s.stall_ms,
                        s.frame
                    ));
                }
            }
        }

        // ---- blind tripwire: the slow-wall phase must have tripped it, for view 0, in the log AND
        // the record — the wiring (repaint discriminator, reset rule, per-view attribution) that
        // the pure predicate's tests cannot see.
        match t.blind_from {
            None => fails.push("the blind phase never ran".into()),
            Some(b) => {
                let warned = v0.iter().find(|r| r.frame >= b && r.blind_warned);
                let peak = v0.iter().filter(|r| r.frame >= b).map(|r| r.blind_slow_frames).max().unwrap_or(0);
                let logged = crate::diag::logs_dir()
                    .and_then(|d| std::fs::read_to_string(d.join("fractadyne.log")).ok())
                    .is_some_and(|l| l.contains("⚠FRAME BUDGET IS BLIND: view=0"));
                match (warned, logged) {
                    (Some(w), true) => notes.push(format!(
                        "blind tripwire: {BLIND_FRAMES} wall-slow frames from frame {b}; fired at frame {} \
                         (count peaked at {peak}), logged for view 0",
                        w.frame
                    )),
                    (w, l) => fails.push(format!(
                        "{BLIND_FRAMES} wall-slow frames from frame {b} with fast GPU work: the tripwire \
                         {} in the record and {} in the log (count peaked at {peak}, threshold {})",
                        if w.is_some() { "fired" } else { "did NOT fire" },
                        if l { "fired" } else { "did NOT fire" },
                        crate::render::BUDGET_BLIND_FRAMES
                    )),
                }
            }
        }

        // ---- frames.jsonl: header for this session; every row exactly its keys; summaries add up.
        // Written by its own thread, so first wait for it to land everything queued so far.
        if !frame_record::flush_jsonl() {
            fails.push("the frames.jsonl writer did not answer a flush within 5 s".into());
        }
        if frame_record::jsonl_dropped() > 0 {
            fails.push(format!(
                "frames.jsonl dropped {} row(s): its writer thread fell a whole queue behind",
                frame_record::jsonl_dropped()
            ));
        }
        match frame_record::jsonl_file().map(|p| std::fs::read_to_string(&p).map_err(|e| (p, e))) {
            None => fails.push("no frames.jsonl path (logging off?)".into()),
            Some(Err((p, e))) => fails.push(format!("{} unreadable: {e}", p.display())),
            Some(Ok(text)) => {
                let frame_keys: std::collections::BTreeSet<&str> = FrameRecord::FIELDS.iter().map(|(n, _)| *n).collect();
                let sum_keys: std::collections::BTreeSet<&str> = frame_record::SUMMARY_KEYS.iter().copied().collect();
                let (mut header_ok, mut events, mut summaries, mut summed, mut bad_rows) = (false, 0u64, 0u64, 0u64, 0u64);
                let mut last_summary_t: Option<u64> = None;
                for (i, line) in text.lines().enumerate() {
                    let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
                        bad_rows += 1;
                        continue;
                    };
                    let Some(o) = v.as_object() else {
                        bad_rows += 1;
                        continue;
                    };
                    let keys: std::collections::BTreeSet<&str> = o.keys().map(String::as_str).collect();
                    match o.get("kind") {
                        Some(k) if k == "header" => {
                            header_ok = i == 0 && o.get("session").and_then(|s| s.as_u64()) == Some(frame_record::session_id());
                        }
                        Some(k) if k == "summary" => {
                            summaries += 1;
                            summed += o.get("frames").and_then(|f| f.as_u64()).unwrap_or(0);
                            last_summary_t = o.get("t_ms").and_then(|t| t.as_u64());
                            bad_rows += (keys != sum_keys) as u64;
                        }
                        _ => {
                            events += 1;
                            bad_rows += (keys != frame_keys) as u64;
                        }
                    }
                }
                if !header_ok {
                    fails.push("frames.jsonl does not open with this session's header".into());
                }
                if bad_rows > 0 {
                    fails.push(format!("frames.jsonl has {bad_rows} row(s) that do not parse or carry the wrong keys"));
                }
                // A summary is written when a record arrives past its second's end, and covers
                // every record before that one — so a 13 s gap is ONE summary, not thirteen. The
                // exact invariant: the summaries' view-0 frames are the view-0 frames recorded
                // before the last summary's timestamp.
                let whole_session = recs.first().map(|r| r.seq) == Some(0);
                match last_summary_t {
                    None => fails.push("frames.jsonl has no summary row".into()),
                    Some(_) if !whole_session => {
                        notes.push("the ring no longer holds the whole session; summary totals not cross-checked".into())
                    }
                    Some(tl) => {
                        let before = recs.iter().filter(|r| r.kind == kind::FRAME && r.view == 0 && r.t_ms < tl).count() as u64;
                        if summed != before {
                            fails.push(format!(
                                "frames.jsonl summaries count {summed} view-0 frames; {before} were recorded before the last summary"
                            ));
                        }
                    }
                }
                notes.push(format!(
                    "frames.jsonl: {} bytes, {summaries} summary rows covering {summed} frames, {events} event rows",
                    text.len()
                ));
                if events == 0 {
                    fails.push("frames.jsonl carries no event row, though the wedge made a slow frame and a stall".into());
                }
            }
        }

        // ---- log budget (design W10): no category may flood the log. `[fd-accum] begin` once ran
        // at ~31 lines/s and made up ~65% of a crashing session's log, burying everything else.
        // (Accumulation is off under harnesses, so that particular flood is pinned by the
        // `LineLimiter` unit test; this guards against the next one.)
        const LOG_RATE_LIMIT: f64 = 10.0; // lines per second, over any 5 s window
        if let Some(text) = crate::diag::logs_dir().and_then(|d| std::fs::read_to_string(d.join("fractadyne.log")).ok()) {
            let lines: Vec<&str> = text.lines().collect();
            // This session only: from its own start banner.
            let from = lines
                .iter()
                .rposition(|l| l.contains("[fd-start]") && l.contains("args:") && l.contains("--recordtest"))
                .unwrap_or(0);
            let mut per: std::collections::BTreeMap<&str, Vec<f64>> = Default::default();
            for l in &lines[from..] {
                let Some((secs, tail)) = l.strip_prefix("[+").and_then(|r| r.split_once("s] ")) else { continue };
                let Ok(t) = secs.trim().parse::<f64>() else { continue };
                let cat = if let Some(c) = tail.strip_prefix("[fd-") {
                    c.split(']').next()
                } else if tail.starts_with("[crumb]") {
                    Some("crumb")
                } else {
                    None
                };
                if let Some(c) = cat {
                    per.entry(c).or_default().push(t);
                }
            }
            let mut worst: Option<(&str, f64)> = None;
            for (cat, ts) in &per {
                // Densest 5 s window (the stamps are in order).
                let mut j = 0;
                let mut best = 0usize;
                for i in 0..ts.len() {
                    while ts[i] - ts[j] > 5.0 {
                        j += 1;
                    }
                    best = best.max(i - j + 1);
                }
                let rate = best as f64 / 5.0;
                if worst.is_none_or(|(_, r)| rate > r) {
                    worst = Some((cat, rate));
                }
                if rate > LOG_RATE_LIMIT {
                    fails.push(format!("[fd-{cat}] ran at {rate:.1} lines/s over 5 s (limit {LOG_RATE_LIMIT})"));
                }
            }
            if let Some((cat, rate)) = worst {
                notes.push(format!(
                    "log budget: {} categories this session, densest [{cat}] at {rate:.1} lines/s over 5 s (limit {LOG_RATE_LIMIT})",
                    per.len()
                ));
            }
        } else {
            fails.push("this session's fractadyne.log could not be read".into());
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
        //
        // ⚠**Where the logs are is part of the measurement.** `frames.bin` is written on the UI
        // thread every frame, so on a network share each write pays the share. The RX 6800 XT's
        // beta.113 battery ran with its logs on `\\vger\share` and failed here at 4.2% — a round
        // trip per `frames.jsonl` flush (that file now has its own thread). A cost within the limit
        // on a share is a pass (a local disk is cheaper); an excess there cannot say what a user's
        // local disk costs, so it is VACUOUS, not a failure.
        const REFERENCE_FRAME_MS: f64 = 1000.0 / 60.0;
        let logs = crate::diag::logs_dir();
        let logs_remote = logs.as_deref().is_some_and(crate::diag::is_network_path);
        let mut cost_unjudged: Option<String> = None;
        if let Some(d) = &logs {
            notes.push(format!(
                "logs: {} ({})",
                crate::diag::redact_home(&d.display().to_string()),
                if logs_remote { "a NETWORK share" } else { "local" }
            ));
        }
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
                let why = format!("the record's p99 costs {:.2}% of a 60 Hz frame (limit 0.5%)", share * 100.0);
                if logs_remote {
                    cost_unjudged = Some(format!(
                        "{why}, but the logs are on a network share, so that is the share's cost — \
                         rerun with FRACTADYNE_CONFIG_DIR on a local disk"
                    ));
                } else {
                    fails.push(why);
                }
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
        vacuous.extend(cost_unjudged);

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
