//! Crash & hang visibility + unified tracing (design/diagnostics.md phases D1/D4).
//!
//! Everything here is built around one observation from the failure catalog: for a
//! GUI-subsystem launch on Windows stderr goes nowhere, so a panic, a device loss, or a
//! wedged frame loop is indistinguishable from "the app closed" unless the app writes its
//! own record. The pieces:
//!
//! - **Log file** — every diagnostic line is teed to `<config>/logs/fractadyne.log`
//!   (rotated once past ~5 MB). Disable with `FRACTADYNE_LOG=0`.
//! - **Breadcrumb** — a global "current activity" cell, written at phase transitions
//!   (reference build, export tile, glitch pass, tour frame). Costs one mutex store per
//!   transition; read by the panic hook and the watchdog so a dead or hung process names
//!   what it was doing.
//! - **Panic hook** — writes `<config>/logs/crash-<stamp>.txt` with the panic message,
//!   backtrace, breadcrumb, last render manifest, and version, then falls through to the
//!   default hook. Installed for every mode (GUI and CLI) from `main()`.
//! - **Watchdog** — a thread that logs `possible hang` (with the breadcrumb) when nothing
//!   has stamped liveness for >10 s. The GUI stamps every `update()`; long CLI phases stamp
//!   via breadcrumbs and the export progress pump.
//! - **Trace categories** — `FRACTADYNE_TRACE=req,ref,gpu,tile` selects categories
//!   (`1`/empty = all); each line is stamped `[+12.345s]` and teed to the log file.
//!
//! The wgpu error/device-lost callbacks (installed in `FractadyneApp::new`) also report
//! through [`log_line`], so a device loss lands in the same file as everything else.

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Process start, for the `[+12.345s]` stamps. Set once by [`init`].
static START: OnceLock<Instant> = OnceLock::new();
/// `<config>/logs` once resolved (None = file logging unavailable/disabled).
static LOG_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
/// Serializes file appends; holds the resolved log-file path.
static LOG_FILE: Mutex<Option<PathBuf>> = Mutex::new(None);
/// The "current activity" cell: what the process is doing right now.
static BREADCRUMB: Mutex<String> = Mutex::new(String::new());
/// Last render manifest ([`set_manifest`]) — the request a crash was working on.
static MANIFEST: Mutex<String> = Mutex::new(String::new());
/// Liveness stamp (ms since START), fed by `update()`, breadcrumbs, and progress pumps.
static ALIVE_MS: AtomicU64 = AtomicU64::new(0);
/// True once the watchdog thread is running (so tests/multiple inits don't double-spawn).
static WATCHDOG_ON: AtomicBool = AtomicBool::new(false);
/// Monotonic suffix for crash-report filenames (collision-proofs same-second panics).
static CRASH_SEQ: AtomicU64 = AtomicU64::new(0);
/// Set once an allocation failure has been reported — the reporting path itself allocates, so a
/// second failure inside it must fall straight through to the runtime's abort instead of
/// recursing.
static OOM_REPORTED: AtomicBool = AtomicBool::new(false);
/// Emergency allocation held from startup and released on the FIRST allocation failure, so the
/// report (which formats strings and captures a backtrace) has room to run in a process that has
/// just been told there is none. Stored as a `usize` because the pointer crosses a static.
static OOM_RESERVE: AtomicU64 = AtomicU64::new(0);
/// Size of that reserve. Big enough for a backtrace capture and the report string; small enough
/// that holding it costs nothing worth measuring.
const OOM_RESERVE_BYTES: usize = 8 << 20;

/// Process working set / peak, for a breadcrumb. Cheap (one Win32 call), so it can sit on phase
/// transitions — but NOT on anything per-frame.
pub(crate) fn memory_summary() -> String {
    memory_line()
}

/// Process working set / peak, formatted for a report line. `(0, 0)` off Windows → "unavailable".
fn memory_line() -> String {
    match crate::sysinfo::process_memory() {
        (0, 0) => "unavailable".to_string(),
        (ws, peak) => format!("rss {} MB, peak {} MB", ws >> 20, peak >> 20),
    }
}

/// Reserve the emergency block. Called once from [`init`], before anything large runs.
fn arm_oom_reserve() {
    use std::alloc::{GlobalAlloc, Layout, System};
    let Ok(layout) = Layout::from_size_align(OOM_RESERVE_BYTES, 16) else {
        return;
    };
    // SAFETY: a plain sized allocation from the system allocator; the pointer is only ever
    // handed back to `System.dealloc` with this same layout, in `release_oom_reserve`.
    let p = unsafe { System.alloc(layout) };
    if !p.is_null() {
        OOM_RESERVE.store(p as usize as u64, Ordering::SeqCst);
    }
}

/// Give the reserve back to the allocator so the report below can allocate.
fn release_oom_reserve() {
    use std::alloc::{GlobalAlloc, Layout, System};
    let p = OOM_RESERVE.swap(0, Ordering::SeqCst);
    if p == 0 {
        return;
    }
    let Ok(layout) = Layout::from_size_align(OOM_RESERVE_BYTES, 16) else {
        return;
    };
    // SAFETY: `p` came from `System.alloc` with this exact layout in `arm_oom_reserve`, and the
    // swap above guarantees only one caller ever frees it.
    unsafe { System.dealloc(p as usize as *mut u8, layout) };
}

/// An allocation just returned null; the runtime is about to `abort()`. Leave a record first.
/// Called from the global allocator ([`crate::alloc::ReportingAlloc`]) — see that module for why
/// this is the only place it can be done on stable Rust.
pub(crate) fn on_alloc_fail(bytes: usize) {
    if OOM_REPORTED.swap(true, Ordering::SeqCst) {
        return; // already reporting (or re-entered from inside the report) — let it abort
    }
    release_oom_reserve();
    let msg = format!(
        "out of memory: allocation of {bytes} bytes failed ({})",
        memory_line()
    );
    write_crash_report_at(&msg, "<allocator>");
    log_line("oom", &msg);
}

/// Seconds since process start (0.0 before [`init`]).
pub(crate) fn elapsed_s() -> f64 {
    START.get().map(|t| t.elapsed().as_secs_f64()).unwrap_or(0.0)
}

fn stamp() -> String {
    format!("[+{:9.3}s]", elapsed_s())
}

/// One-time setup: start clock, open the log file (with rotation), install the panic
/// hook, start the watchdog. Called first thing in `main()`; cheap and infallible —
/// any file-system failure just disables file logging.
pub(crate) fn init(args: &[String]) {
    let _ = START.set(Instant::now());
    alive();

    // FRACTADYNE_LOG=0 disables the file (stderr behavior is unchanged either way).
    let file_log_on = std::env::var("FRACTADYNE_LOG").map_or(true, |v| v != "0");
    // `--log-dir DIR` / FRACTADYNE_LOG_DIR redirect the LOGS ONLY (log file, crash reports,
    // perf.jsonl, session.running) — the session and settings stay in the config dir. Built for
    // pointing validation-run logs at a network share without also making the run hermetic the
    // way FRACTADYNE_CONFIG_DIR does. The flag wins over the variable.
    let (over, over_src) = log_dir_override(
        args,
        std::env::var("FRACTADYNE_LOG_DIR").ok().as_deref(),
    );
    let mut start_notes: Vec<String> = Vec::new();
    let dir = if file_log_on {
        match over {
            // An explicitly requested dir that cannot be created must not SILENTLY become "no
            // file logging" — a validation run that quietly logs nowhere is the harness lesson
            // all over again. Fall back to the default location and say so in it.
            Some(d) => {
                if std::fs::create_dir_all(&d).is_ok() {
                    start_notes.push(format!("logs directed to {} ({over_src})", d.display()));
                    Some(d)
                } else {
                    start_notes.push(format!(
                        "log dir {} ({over_src}) is not writable — using the config dir instead",
                        d.display()
                    ));
                    fractadyne_state::config_dir().map(|d| d.join("logs"))
                }
            }
            None => fractadyne_state::config_dir().map(|d| d.join("logs")),
        }
    } else {
        None
    };
    let dir = dir.filter(|d| std::fs::create_dir_all(d).is_ok());
    let _ = LOG_DIR.set(dir.clone());
    if let Some(dir) = dir {
        let path = dir.join("fractadyne.log");
        // Single-slot rotation: past ~5 MB the old log becomes fractadyne.log.1.
        if std::fs::metadata(&path).map(|m| m.len() > 5_000_000).unwrap_or(false) {
            let _ = std::fs::rename(&path, dir.join("fractadyne.log.1"));
        }
        *LOG_FILE.lock().unwrap() = Some(path);
        for n in &start_notes {
            log_line("start", n);
        }
    }
    // BEFORE the start line, so the "last log lines" it quotes belong to the dead session.
    report_unclean_previous_session();
    arm_oom_reserve();
    log_line(
        "start",
        &format!(
            "fractadyne {} — {} — args: {}",
            crate::sysinfo::version_string(),
            crate::sysinfo::now_utc_string(),
            args.iter().skip(1).cloned().collect::<Vec<_>>().join(" "),
        ),
    );
    // What this BUILD contains, which is a compile-time fact and the only backend statement that
    // can honestly be made before anything has iterated. Which backend actually *ran* is a
    // separate question, answered by `backend_status_line()` in the crash report and `--selftest`.
    log_line(
        "start",
        &format!("bignum backends compiled in: {}", fractadyne_core::built_in_backends()),
    );

    install_panic_hook();
    // The watchdog is NOT started here: the pre-GUI CLI modes (--crosscheck-f3,
    // --validate-deep, …) do minutes of legitimate silent bignum work and would trip it.
    // `FractadyneApp::new` starts it for every update()-driven mode (GUI and CLI renders).
}

/// Append one line to the log file (no-op when file logging is off). Never panics.
fn file_line(text: &str) {
    let guard = match LOG_FILE.lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    if let Some(path) = guard.as_ref() {
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(f, "{} {}", stamp(), text);
        }
    }
}

/// A diagnostic event: stderr + log file. `cat` becomes the `[fd-cat]` prefix.
///
/// The stderr write is non-panicking (`writeln!` on a locked handle, error ignored): a
/// broken pipe — e.g. `fractadyne … 2>&1 | head` closing early — must NOT panic here. This
/// runs inside the panic hook, where an `eprintln!` that panics on EPIPE would trigger a
/// double-panic abort and lose the crash report entirely (the exact automation scenario D1
/// targets).
pub(crate) fn log_line(cat: &str, msg: &str) {
    let _ = writeln!(std::io::stderr(), "[fd-{cat}] {} {msg}", stamp());
    file_line(&format!("[fd-{cat}] {msg}"));
}

/// Trace category set parsed from FRACTADYNE_TRACE: `None` = tracing off,
/// `Some(vec![])` = all categories, `Some(cats)` = only those.
fn trace_cats() -> Option<&'static Vec<String>> {
    static CATS: OnceLock<Option<Vec<String>>> = OnceLock::new();
    CATS.get_or_init(|| match std::env::var("FRACTADYNE_TRACE") {
        Err(_) => None,
        Ok(v) if v == "0" => None,
        Ok(v) if v.is_empty() || v == "1" => Some(Vec::new()),
        Ok(v) => Some(v.split(',').map(|s| s.trim().to_ascii_lowercase()).collect()),
    })
    .as_ref()
}

/// Is trace category `cat` enabled? (`FRACTADYNE_TRACE=1` enables all.)
pub(crate) fn trace_on(cat: &str) -> bool {
    match trace_cats() {
        None => false,
        Some(cats) => cats.is_empty() || cats.iter().any(|c| c == cat),
    }
}

/// A trace event: printed (stderr + file) only when its category is enabled.
/// Prefer `if diag::trace_on("x") { diag::trace("x", format!(..)) }` at call sites so the
/// format cost is only paid when tracing.
pub(crate) fn trace(cat: &str, msg: String) {
    if trace_on(cat) {
        log_line(cat, &msg);
    }
}

/// `FRACTADYNE_PERF=1` enables the JSONL perf log (D3.2).
pub(crate) fn perf_on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("FRACTADYNE_PERF").is_ok_and(|v| v != "0"))
}

/// Resolve a log-directory override from the command line and environment: `--log-dir DIR`
/// wins, `FRACTADYNE_LOG_DIR` is the fallback, and no override means the default
/// `<config>/logs`. Pure so the precedence and the malformed-flag case are pinned by test.
/// A `--log-dir` whose value is missing (end of line, or the next token is another option)
/// yields no override here — the CLI guard exits fatally on it, and this resolver must not
/// guess a directory in the meantime.
pub(crate) fn log_dir_override(
    args: &[String],
    env: Option<&str>,
) -> (Option<PathBuf>, &'static str) {
    if let Some(i) = args.iter().position(|a| a == "--log-dir") {
        if let Some(v) = args.get(i + 1) {
            if !v.starts_with('-') {
                return (Some(PathBuf::from(v)), "--log-dir");
            }
        }
        return (None, "--log-dir");
    }
    match env.filter(|v| !v.is_empty()) {
        Some(v) => (Some(PathBuf::from(v)), "FRACTADYNE_LOG_DIR"),
        None => (None, ""),
    }
}

#[cfg(test)]
mod log_dir {
    use super::*;
    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn the_flag_wins_over_the_variable() {
        let (d, src) = log_dir_override(&s(&["exe", "--log-dir", "D:/share"]), Some("E:/env"));
        assert_eq!(d, Some(PathBuf::from("D:/share")));
        assert_eq!(src, "--log-dir");
    }

    #[test]
    fn the_variable_applies_when_the_flag_is_absent() {
        let (d, src) = log_dir_override(&s(&["exe", "--selftest"]), Some("//vger/share/logs"));
        assert_eq!(d, Some(PathBuf::from("//vger/share/logs")));
        assert_eq!(src, "FRACTADYNE_LOG_DIR");
        assert_eq!(log_dir_override(&s(&["exe"]), Some("")).0, None); // empty var = unset
    }

    #[test]
    fn a_missing_value_yields_no_override_rather_than_a_guess() {
        // The CLI guard exits fatally on these; the resolver must not invent a directory
        // (or silently fall back to the env var) for the lines it logs before that exit.
        assert_eq!(log_dir_override(&s(&["exe", "--log-dir"]), Some("E:/env")).0, None);
        assert_eq!(
            log_dir_override(&s(&["exe", "--log-dir", "--play"]), None).0,
            None
        );
    }

    #[test]
    fn no_override_means_the_default_location() {
        assert_eq!(log_dir_override(&s(&["exe", "--selftest"]), None).0, None);
    }
}

/// The resolved logs directory (`<config>/logs`), or `None` if file logging is off/unavailable.
/// Used by the issue reporter to pull the log + crash reports.
pub(crate) fn logs_dir() -> Option<PathBuf> {
    LOG_DIR.get().and_then(|o| o.clone())
}

/// The tail of the current log file (up to `max_bytes`, trimmed to a line boundary), for issue
/// reports. `None` if logging is off or the file can't be read.
pub(crate) fn recent_log(max_bytes: usize) -> Option<String> {
    let data = std::fs::read(logs_dir()?.join("fractadyne.log")).ok()?;
    let start = data.len().saturating_sub(max_bytes);
    let s = String::from_utf8_lossy(&data[start..]).into_owned();
    // If truncated mid-file, drop the partial first line.
    Some(if start > 0 {
        s.split_once('\n').map(|(_, rest)| rest.to_string()).unwrap_or(s)
    } else {
        s
    })
}

/// The newest `crash-*.txt` report (filename, contents), if any exists.
pub(crate) fn latest_crash() -> Option<(String, String)> {
    let dir = logs_dir()?;
    let mut best: Option<(SystemTime, PathBuf)> = None;
    for entry in std::fs::read_dir(&dir).ok()?.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("crash-") && name.ends_with(".txt") {
            if let Ok(t) = entry.metadata().and_then(|m| m.modified()) {
                if best.as_ref().is_none_or(|(bt, _)| t > *bt) {
                    best = Some((t, entry.path()));
                }
            }
        }
    }
    let (_, path) = best?;
    let body = std::fs::read_to_string(&path).ok()?;
    Some((path.file_name()?.to_string_lossy().into_owned(), body))
}

/// Append one JSON record to `<config>/logs/perf.jsonl` (no-op unless `FRACTADYNE_PERF=1`).
/// Caller supplies the JSON body; timestamp/version are added here. Regression tracking
/// across builds becomes greppable history instead of memory.
pub(crate) fn perf_jsonl(body_fields: &str) {
    if !perf_on() {
        return;
    }
    let Some(Some(dir)) = LOG_DIR.get() else { return };
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let line = format!(
        "{{\"ts\":{secs},\"version\":\"{}\",{body_fields}}}",
        crate::sysinfo::version_string(),
    );
    if let Ok(mut f) =
        std::fs::OpenOptions::new().create(true).append(true).open(dir.join("perf.jsonl"))
    {
        let _ = writeln!(f, "{line}");
    }
}

/// Stamp liveness (the watchdog resets its stall clock).
pub(crate) fn alive() {
    if let Some(t) = START.get() {
        ALIVE_MS.store(t.elapsed().as_millis() as u64, Ordering::Relaxed);
    }
}

/// Record what the process is doing right now. Written at phase transitions; read by the
/// panic hook and watchdog. Also stamps liveness and tees to the log file.
///
/// This is a single process-global slot, so when phases on different threads overlap (a GUI
/// export worker + a live recompute worker, say) the last writer wins and the panic hook /
/// watchdog may name a *concurrent* activity rather than the failing thread's own. It is
/// therefore tagged with the writing thread's name: the crash report also records the
/// panicking thread separately (`thread:`), so a reader can see when the breadcrumb came
/// from a different thread and treat it as context, not cause. Per-thread breadcrumbs (a
/// thread-local registry snapshotted in the hook) are the full fix — deferred as a larger,
/// deliberate change; the full-timeline `[crumb]` log lines below already disambiguate.
pub(crate) fn breadcrumb(msg: String) {
    alive();
    let thread = std::thread::current().name().unwrap_or("?").to_string();
    file_line(&format!("[crumb] ({thread}) {msg}"));
    if let Ok(mut b) = BREADCRUMB.lock() {
        *b = format!("{msg} [{thread}]");
    }
}

/// Current breadcrumb (empty string when none was set yet).
pub(crate) fn current_breadcrumb() -> String {
    BREADCRUMB.lock().map(|b| b.clone()).unwrap_or_default()
}

/// Record the effective render manifest (center/zoom/iter/mode). Kept for crash reports;
/// also the D4.2 anti-F8 record of what a render was *actually* asked to do.
pub(crate) fn set_manifest(msg: String) {
    if let Ok(mut m) = MANIFEST.lock() {
        *m = msg;
    }
}

/// Compose and persist a crash report (the durable artifact — written even if stderr is
/// broken). A process-wide counter in the name prevents two reports in the same wall-clock
/// second (realistic on a device loss: the uncaptured-error callback reports on one thread
/// while an export worker's next wgpu call panics on another) from clobbering each other via
/// same-second `crash-<secs>.txt` overwrite. `loc` is the code location when known ("" = none).
/// Used by the panic hook AND by the device-lost handler, which restarts instead of panicking
/// but must leave the same forensic trail.
pub(crate) fn write_crash_report(msg: &str) {
    write_crash_report_at(msg, "<device-lost handler>");
}

fn write_crash_report_at(msg: &str, loc: &str) {
    let report = format!(
        "fractadyne crash report\n\
         version : {}\n\
         time    : {}\n\
         uptime  : {:.1}s\n\
         panic   : {msg}\n\
         at      : {loc}\n\
         memory  : {}\n\
         activity: {}\n\
         manifest: {}\n\
         tunables: {}\n\
         bignum  : {}\n\
         thread  : {}\n\n\
         backtrace (debug symbols are disabled in this build; addresses only):\n{}\n",
        crate::sysinfo::version_string(),
        crate::sysinfo::now_utc_string(),
        elapsed_s(),
        memory_line(),
        current_breadcrumb(),
        MANIFEST.lock().map(|m| m.clone()).unwrap_or_default(),
        // Always printed, `stock` included: a report that says nothing about tunables cannot be
        // told apart from one written by a build that predates the override mechanism — and a
        // report from an overridden run must never be read as stock behaviour.
        crate::tunables::status_line(),
        // Same reasoning as the tunables line, and sourced the same way: from what actually ran.
        // A deep-zoom crash report whose arithmetic backend is unknown cannot be compared with
        // any other report, and `none` is itself informative (nothing had iterated yet).
        fractadyne_core::backend_status_line(),
        std::thread::current().name().unwrap_or("<unnamed>"),
        std::backtrace::Backtrace::force_capture(),
    );
    if let Some(Some(dir)) = LOG_DIR.get() {
        let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let n = CRASH_SEQ.fetch_add(1, Ordering::Relaxed);
        let path = dir.join(format!("crash-{secs}-{n}.txt"));
        if std::fs::write(&path, &report).is_ok() {
            let _ = writeln!(std::io::stderr(), "[fd-panic] crash report written: {}", path.display());
        }
    }
}

/// `<logs>/session.running` — present only while the GUI event loop is running.
fn marker_path() -> Option<PathBuf> {
    LOG_DIR.get().cloned().flatten().map(|d| d.join("session.running"))
}

/// Arm the unclean-exit marker. Called immediately before the GUI event loop starts.
///
/// This is the backstop for the death classes nothing else can see. The panic hook covers
/// panics and the allocator wrapper covers OOM, but a `__fastfail` abort from anywhere else, an
/// access violation (`0xc0000005`, one of which is on record here unexplained), or an outright
/// kill all leave no trace at all — the process is simply gone and the log stops mid-sentence.
///
/// Armed around the GUI ONLY, and every deliberate exit routes through [`crate::exit`] which
/// disarms it, so a normal shutdown can never look like a crash. That matters more than
/// coverage: a false crash report would teach everyone to ignore real ones.
pub(crate) fn begin_gui_session() {
    if let Some(p) = marker_path() {
        let body = format!(
            "{}\nstarted {}\n",
            crate::sysinfo::version_string(),
            crate::sysinfo::now_utc_string()
        );
        let _ = std::fs::write(p, body);
    }
}

/// Disarm the marker. Idempotent; called from [`crate::exit`] and after the event loop returns.
pub(crate) fn end_session() {
    if let Some(p) = marker_path() {
        let _ = std::fs::remove_file(p);
    }
}

/// If the previous GUI session left its marker behind, it never shut down cleanly. Report it,
/// naming what it was doing — the log's last breadcrumb is the only evidence such a death leaves.
/// Deliberately worded as "no clean shutdown" rather than asserting a crash: a hard kill (Task
/// Manager, a `Stop-Process` from a test harness, a power loss) lands here too.
/// Did the PREVIOUS session end without a clean shutdown? Set during `init` by
/// `report_unclean_previous_session`, so the UI can offer to send the report it just wrote.
///
/// The `session.running` marker covers both shapes: a panic (whose own crash report was written by
/// the dying process) and a hard kill or device loss that never reached the panic hook. Either way
/// the marker survives, which is exactly the signal a user cares about — "it didn't come back
/// cleanly last time".
pub(crate) fn previous_session_unclean() -> bool {
    PREV_UNCLEAN.load(std::sync::atomic::Ordering::Relaxed)
}
static PREV_UNCLEAN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn report_unclean_previous_session() {
    let Some(p) = marker_path() else { return };
    let Ok(prev) = std::fs::read_to_string(&p) else { return };
    let _ = std::fs::remove_file(&p);
    let tail = LOG_DIR
        .get()
        .cloned()
        .flatten()
        .map(|d| d.join("fractadyne.log"))
        .and_then(|f| std::fs::read_to_string(f).ok())
        .map(|s| {
            // Last six lines in CHRONOLOGICAL order — `rev().take()` alone reads newest-first,
            // which is exactly backwards for following what the process was doing as it died.
            let mut last: Vec<&str> = s.lines().rev().take(6).collect();
            last.reverse();
            last.join("\n  ")
        })
        .unwrap_or_default();
    let msg = format!(
        "previous session ended without a clean shutdown (no panic, no crash report) — \
         {} | last log lines:\n  {}",
        prev.lines().collect::<Vec<_>>().join(", "),
        tail
    );
    log_line("unclean", &msg);
    write_crash_report_at(&msg, "<previous session>");
    PREV_UNCLEAN.store(true, std::sync::atomic::Ordering::Relaxed);
}

fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let msg = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "<non-string panic payload>".into());
        let loc = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "<unknown>".into());
        // Write the crash FILE first — see `write_crash_report_at`.
        write_crash_report_at(&msg, &loc);
        // Then the log line (non-panicking stderr; also teed to the log file).
        log_line("panic", &format!("{msg} at {loc} — activity: {}", current_breadcrumb()));
        default(info);
    }));
}

/// Watchdog: logs `possible hang` with the breadcrumb when nothing stamped liveness for
/// >10 s, then re-warns every 30 s while the stall persists. It cannot distinguish a hang
/// from a long uninstrumented compute — that ambiguity is the point: either way the log
/// names the phase that went silent. Started once from `FractadyneApp::new` (update()-driven
/// modes stamp liveness every frame; long phases stamp via breadcrumbs/progress pumps).
pub(crate) fn start_watchdog() {
    if WATCHDOG_ON.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::Builder::new()
        .name("fd-watchdog".into())
        .spawn(|| {
            const STALL_S: u64 = 10;
            const REWARN_S: u64 = 30;
            let mut last_warn_ms: u64 = 0;
            loop {
                std::thread::sleep(Duration::from_secs(2));
                let Some(t) = START.get() else { continue };
                let now_ms = t.elapsed().as_millis() as u64;
                let alive_ms = ALIVE_MS.load(Ordering::Relaxed);
                let stale_s = now_ms.saturating_sub(alive_ms) / 1000;
                if stale_s >= STALL_S && now_ms.saturating_sub(last_warn_ms) >= REWARN_S * 1000 {
                    last_warn_ms = now_ms;
                    log_line(
                        "watch",
                        &format!(
                            "possible hang: no activity for {stale_s}s — last activity: {}",
                            {
                                let b = current_breadcrumb();
                                if b.is_empty() { "<none recorded>".into() } else { b }
                            }
                        ),
                    );
                }
            }
        })
        .ok();
}

/// Spawn a CLI progress pump: prints `\r<label> N%` to stderr every ~2 s from a permille
/// progress atomic (the `render_export` contract), stamps liveness, and stops when the
/// returned guard is dropped. Prints nothing for renders that finish inside the first tick.
pub(crate) struct ProgressPump {
    stop: std::sync::Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

pub(crate) fn progress_pump(
    label: &str,
    progress: std::sync::Arc<std::sync::atomic::AtomicU32>,
) -> ProgressPump {
    let stop = std::sync::Arc::new(AtomicBool::new(false));
    let stop2 = stop.clone();
    let label = label.to_string();
    let thread = std::thread::Builder::new()
        .name("fd-progress".into())
        .spawn(move || {
            let mut printed = false;
            let mut last_p = u32::MAX;
            'outer: while !stop2.load(Ordering::Relaxed) {
                // ~2 s cadence, but check `stop` every 100 ms so Drop never stalls.
                for _ in 0..20 {
                    std::thread::sleep(Duration::from_millis(100));
                    if stop2.load(Ordering::Relaxed) {
                        break 'outer;
                    }
                }
                let p = progress.load(Ordering::Relaxed).min(1000);
                // Stamp liveness ONLY when a tile actually finished. Stamping every tick
                // (the original bug) made the watchdog blind to a wedged render: a hung
                // render_export froze `p` but liveness kept advancing, so `possible hang`
                // never fired. Now a frozen `p` lets the stall clock run out — the log then
                // shows this frozen line followed by the watchdog's warnings (the exact
                // "slow vs hung" signal DIAGNOSTICS.md tells the reader to look for). A slow
                // single-tile render also freezes `p`; the watchdog's warning there is the
                // documented, acceptable can't-tell-hang-from-long-compute ambiguity.
                if p != last_p {
                    alive();
                    last_p = p;
                    // Tee to the log file so a post-mortem sees progression, not just stderr.
                    file_line(&format!("[progress] {label} {}%", p / 10));
                }
                let _ = write!(std::io::stderr(), "\r[fd-progress] {} {label} {:3}%", stamp(), p / 10);
                let _ = std::io::stderr().flush();
                printed = true;
            }
            if printed {
                let _ = writeln!(std::io::stderr());
            }
        })
        .ok();
    ProgressPump { stop, thread }
}

impl Drop for ProgressPump {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
