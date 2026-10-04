//! Crash & hang visibility + unified tracing (design/diagnostics.md phases D1/D4).
//!
//! Everything here is built around one observation from the failure catalog: for a
//! GUI-subsystem launch on Windows stderr goes nowhere, so a panic, a device loss, or a
//! wedged frame loop is indistinguishable from "the app closed" unless the app writes its
//! own record. The pieces:
//!
//! - **Log file** — every diagnostic line is teed to `<config>/logs/fractadyne.log`
//!   (rotated once past ~5 MB). Disable with `FRACTADYNE_LOG=0`.
//! - **Console gate** — the same lines reach stderr only when [`console_on`]. ON whenever there is
//!   any command-line argument (so every headless mode, harness and validation script keeps its
//!   output), OFF for a bare GUI launch, and overridable by `--console` / `--no-console`,
//!   `FRACTADYNE_CONSOLE`, or the checkbox in the Diagnostics window. ⚠**The file is written
//!   either way** — the gate decides who is *told*, never what is *recorded* — and a panic always
//!   prints.
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
//! - **Trace categories** — `FRACTADYNE_TRACE=req,ref,gpu,tile,idle` selects categories
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
/// The last few frame-budget DECISIONS, for the crash report ([`budget_note`]).
///
/// ⭐⭐**A device loss is decided by what the budget controller was told, and until 2026-09-21 a
/// crash report could not say.** The RX 6800 XT loss of that date ran twenty frames of 200–1027 ms
/// with the learned budget frozen at 1.515e11, and the always-on lethal-band line never fired —
/// so the controller was never handed a slow reading at all. Which of "no reading arrived" and
/// "a reading arrived saying the pass was fast" was true could not be told from the log, because
/// the readings are only traced under `FRACTADYNE_TRACE=gpu` and nobody runs a fourteen-minute
/// session with tracing on waiting for a crash that may not come. This ring is always on, costs
/// one short string per accepted or discarded reading, and is dumped into the report.
static BUDGET_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
/// How many budget decisions the ring keeps. Enough to cover the seconds before a loss (both
/// recorded losses show three or four fatal frames after the first warning sign) without turning
/// the report into a log file.
const BUDGET_LOG_CAP: usize = 24;
/// `fractadyne.log` rotates past this size into `.1` … `.3` — checked WHILE RUNNING as well as at
/// startup. The single-slot, startup-only rotation let one long session write 20 MB into one file
/// (measured on the development machine's own log), and overwrite its own head the next launch.
const LOG_ROTATE_BYTES: u64 = 5_000_000;
const LOG_SLOTS: u32 = 4;
/// Bytes appended since the log's size was last checked — checked every ~256 KB, not every line.
static LOG_SINCE_CHECK: AtomicU64 = AtomicU64::new(0);
/// This startup rotated the previous session's log into `.1`, so that is where the unclean-exit
/// report must look for the dead session's last lines. ⚠It used to read the NEW, empty file: a
/// dead session with a log past 5 MB was reported with no last lines at all.
static ROTATED_AT_START: AtomicBool = AtomicBool::new(false);
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
    // ⚠**First**, before anything can call `log_line` — the start banner below is the very output
    // this decides the fate of.
    set_console(console_default(
        args,
        std::env::var("FRACTADYNE_CONSOLE").ok().as_deref(),
        std::env::var("FRACTADYNE_TRACE").ok().as_deref(),
    ));

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
        // Past ~5 MB the old log shifts into .1 (and .1 → .2 → .3).
        if std::fs::metadata(&path).map(|m| m.len() > LOG_ROTATE_BYTES).unwrap_or(false) {
            rotate_log(&dir);
            ROTATED_AT_START.store(true, Ordering::Relaxed);
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
    install_console_ctrl_handler();
    // The watchdog is NOT started here: the pre-GUI CLI modes (--crosscheck-f3,
    // --validate-deep, …) do minutes of legitimate silent bignum work and would trip it.
    // `FractadyneApp::new` starts it for every update()-driven mode (GUI and CLI renders).
}

/// Shift `fractadyne.log` → `.1` → `.2` → `.3`, dropping the oldest. (`rename` replaces an
/// existing target on every platform std supports, Windows included.)
fn rotate_log(dir: &std::path::Path) {
    for n in (1..LOG_SLOTS).rev() {
        let from = if n == 1 {
            dir.join("fractadyne.log")
        } else {
            dir.join(format!("fractadyne.log.{}", n - 1))
        };
        let _ = std::fs::rename(from, dir.join(format!("fractadyne.log.{n}")));
    }
}

/// Flush the log file and the frame record's files to the DISK, not just the OS cache — for the
/// moments before a possible machine hang (called on every slow frame). A process crash leaves the
/// OS cache to write them; a machine hang does not, and on 2026-09-27 PLUTO's log, `frames.bin` and
/// `frames.jsonl` came back as zeros after one. Never panics; errors are ignored (best effort).
pub(crate) fn sync_to_disk() {
    let path = LOG_FILE.lock().ok().and_then(|g| g.clone());
    if let Some(p) = path {
        if let Ok(f) = std::fs::OpenOptions::new().append(true).open(&p) {
            let _ = f.sync_data();
        }
    }
    frame_record::sync_to_disk();
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
        // Mid-run rotation, under the same lock as every write, so no line lands in a file being
        // renamed. The next write reopens `fractadyne.log` fresh.
        let since = LOG_SINCE_CHECK.fetch_add(text.len() as u64 + 16, Ordering::Relaxed);
        if since > 256 * 1024 {
            LOG_SINCE_CHECK.store(0, Ordering::Relaxed);
            if std::fs::metadata(path).map(|m| m.len() > LOG_ROTATE_BYTES).unwrap_or(false) {
                if let Some(dir) = path.parent() {
                    rotate_log(dir);
                }
            }
        }
    }
}

/// Rate-limits one always-on log line that marks a TRANSITION of a predicate that can flap: the
/// first occurrence prints, repeats within `gap_ms` are held back and counted, and the next line
/// that prints says how many were. A flapping predicate must stay VISIBLE — the count says so, and
/// the frame record counts every transition — but it must not drown the log: `[fd-accum] begin`
/// once made up ~65% of a crashing session's 1.06 MB, restarting ~31 times a second.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct LineLimiter {
    last_ms: Option<u64>,
    held: u32,
}

impl LineLimiter {
    /// `Some(held_back_since_last_print)` when this occurrence should print; `None` when held.
    pub(crate) fn admit(&mut self, now_ms: u64, gap_ms: u64) -> Option<u32> {
        match self.last_ms {
            Some(t) if now_ms.saturating_sub(t) < gap_ms => {
                self.held = self.held.saturating_add(1);
                None
            }
            _ => {
                self.last_ms = Some(now_ms);
                Some(std::mem::take(&mut self.held))
            }
        }
    }
}

/// Milliseconds since process start, for [`LineLimiter`].
pub(crate) fn elapsed_ms() -> u64 {
    (elapsed_s() * 1000.0) as u64
}

/// A diagnostic event: stderr + log file. `cat` becomes the `[fd-cat]` prefix.
///
/// The stderr write is non-panicking (`writeln!` on a locked handle, error ignored): a
/// broken pipe — e.g. `fractadyne … 2>&1 | head` closing early — must NOT panic here. This
/// runs inside the panic hook, where an `eprintln!` that panics on EPIPE would trigger a
/// double-panic abort and lose the crash report entirely (the exact automation scenario D1
/// targets).
/// The log file this process writes, once `init` has chosen it.
pub(crate) fn log_path() -> Option<PathBuf> {
    LOG_FILE.lock().ok().and_then(|g| g.clone())
}

/// A harness's verdict line: to stderr, where its operator reads it, and to the log as
/// `[fd-verdict] <line>`, where `--logcheck` requires it (`validation/logcheck-rules.toml`
/// `[[harness]]`) — a verdict only on stderr cannot tell a run that stopped from one that passed.
pub(crate) fn verdict(line: &str) {
    eprintln!("{line}");
    file_line(&format!("[fd-verdict] {line}"));
}

pub(crate) fn log_line(cat: &str, msg: &str) {
    // ⭐**The FILE always gets the line; only the console is gated.** That is what makes quiet-by-
    // default safe: Help ▸ recent log, the crash report's tail and a bug reporter's attachment are
    // all unchanged, so nothing is lost — it is just not shouted at someone who did not ask.
    if console_on() || cat == "panic" {
        let _ = writeln!(std::io::stderr(), "[fd-{cat}] {} {msg}", stamp());
    }
    file_line(&format!("[fd-{cat}] {msg}"));
}

/// Whether `[fd-*]` diagnostics reach stderr. See [`console_default`] for how it is chosen.
///
/// ⚠An `AtomicBool` rather than a `OnceLock` because the Diagnostics window can turn it on and off
/// while the app runs — the third of the three ways the user asked for.
static CONSOLE: AtomicBool = AtomicBool::new(true);

pub(crate) fn console_on() -> bool {
    CONSOLE.load(Ordering::Relaxed)
}

pub(crate) fn set_console(on: bool) {
    CONSOLE.store(on, Ordering::Relaxed);
}

/// Should `[fd-*]` diagnostics go to the console this run?
///
/// ⭐⭐**Quiet for a bare launch, verbose the moment there is an argument.** Someone who
/// double-clicks the app or starts it from a shortcut did not ask for a running commentary; someone
/// who typed `fractadyne --render …` is at a terminal and the output is the point.
///
/// ⚠⚠**"Any argument" rather than a list of CLI modes, deliberately.** Three gates parse this
/// banner off stderr — `validation/crosscheck_backends.py` reads *"backends compiled in"* to prove
/// it is running the build under test, `validation/corpus/generate_corpus.py` reads *"session:"* to
/// prove the staged session actually loaded, and `scripts/gpu-validate.ps1` folds stderr into every
/// step log — and all three invoke fractadyne WITH arguments. A hand-maintained list of headless
/// modes would silently drop one the day a mode was added, and the failure would look like a gate
/// that stopped checking rather than one that broke. There is no list to fall out of date.
///
/// Precedence, most explicit first:
/// 1. `--console` / `--no-console` on the command line.
/// 2. `FRACTADYNE_CONSOLE` (`0` off, anything else on).
/// 3. `FRACTADYNE_TRACE` set to anything live implies **on** — a trace whose output goes nowhere is
///    the [probe-whose-reading-is-discarded] failure, and it would look like the tracing is broken.
/// 4. Otherwise: on if there is any argument past the program name.
pub(crate) fn console_default(
    args: &[String],
    env_console: Option<&str>,
    env_trace: Option<&str>,
) -> bool {
    if args.iter().any(|a| a == "--no-console") {
        return false;
    }
    if args.iter().any(|a| a == "--console") {
        return true;
    }
    if let Some(v) = env_console {
        return v != "0";
    }
    // Matches `trace_cats`: unset, or "0", is off.
    if env_trace.is_some_and(|v| v != "0") {
        return true;
    }
    args.len() > 1
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
mod log_dir;

#[cfg(test)]
#[path = "diag/console.rs"]
mod console_tests;

#[cfg(test)]
#[path = "diag/device_loss_hint.rs"]
mod device_loss_hint_tests;

/// The always-on per-frame record (design/live-render-robustness.md W1).
pub(crate) mod frame_record;

/// Replace the user's home directory with `~` everywhere in `s` — for anything that leaves the
/// machine (the issue report). Issues are public, and on every desktop OS the home path contains
/// the account name.
pub(crate) fn redact_home(s: &str) -> String {
    match directories::BaseDirs::new() {
        Some(b) => redact_path(s, &b.home_dir().to_string_lossy()),
        None => s.to_string(),
    }
}

/// [`redact_home`] with the home path given — pure, so it is pinned by test. Matches the path with
/// either separator and with JSON-escaped backslashes, ASCII-case-insensitively (Windows paths are
/// case-insensitive and logs are not consistent about it). A path shorter than four characters is
/// left alone: a home of `/` or `C:\` would redact half the report.
pub(crate) fn redact_path(s: &str, home: &str) -> String {
    let home = home.trim_end_matches(['\\', '/']);
    if home.len() < 4 {
        return s.to_string();
    }
    let mut variants = vec![
        home.to_string(),
        home.replace('\\', "/"),
        home.replace('/', "\\"),
        home.replace('/', "\\").replace('\\', "\\\\"),
    ];
    // Longest first, so the JSON-escaped form is not half-replaced by the plain one.
    variants.sort_by_key(|v| std::cmp::Reverse(v.len()));
    variants.dedup();
    let mut out = s.to_string();
    for v in &variants {
        let needle = v.to_ascii_lowercase();
        let mut result = String::with_capacity(out.len());
        let hay = out.to_ascii_lowercase(); // byte-for-byte the same length as `out`
        let mut last = 0;
        for (i, _) in hay.match_indices(&needle) {
            if i < last {
                continue;
            }
            // The match must END where the path component ends: a home of `C:\Users\rob` is not
            // inside `C:\Users\robin`.
            let end = i + needle.len();
            let next = out[end..].chars().next();
            if next.is_some_and(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.')) {
                continue;
            }
            result.push_str(&out[last..i]);
            result.push('~');
            last = end;
        }
        result.push_str(&out[last..]);
        out = result;
    }
    out
}

/// The resolved logs directory (`<config>/logs`), or `None` if file logging is off/unavailable.
/// Used by the issue reporter to pull the log + crash reports.
pub(crate) fn logs_dir() -> Option<PathBuf> {
    LOG_DIR.get().and_then(|o| o.clone())
}

/// Is `p` on a network share? Best effort: `false` when it cannot tell.
///
/// ⭐Where the logs live is part of what a cost measurement measures: `frames.bin` is written on the
/// UI thread every frame, and the RX 6800 XT's beta.113 battery, run with its logs on a
/// network share, read the share's round trip as the frame record's cost.
pub(crate) fn is_network_path(p: &std::path::Path) -> bool {
    #[cfg(windows)]
    {
        let s = p.to_string_lossy();
        let s = s.replace('/', "\\");
        if let Some(rest) = s.strip_prefix(r"\\?\") {
            // A verbatim path: `\\?\UNC\host\share` is a share, `\\?\C:\` is a drive.
            return rest.get(..4).is_some_and(|u| u.eq_ignore_ascii_case("UNC\\")) || drive_is_remote(rest);
        }
        s.starts_with(r"\\") || drive_is_remote(&s)
    }
    #[cfg(not(windows))]
    {
        // The filesystem type of the longest mount point containing `p`.
        let Ok(p) = p.canonicalize() else { return false };
        let Ok(mounts) = std::fs::read_to_string("/proc/self/mounts") else { return false };
        mounts
            .lines()
            .filter_map(|l| {
                let mut f = l.split_whitespace();
                let (_, at, fstype) = (f.next()?, f.next()?, f.next()?);
                p.starts_with(at).then_some((at.len(), fstype))
            })
            .max_by_key(|(n, _)| *n)
            .is_some_and(|(_, t)| matches!(t, "nfs" | "nfs4" | "cifs" | "smb3" | "smbfs" | "9p" | "fuse.sshfs"))
    }
}

/// `C:…` on a drive Windows calls remote (a mapped share).
#[cfg(windows)]
fn drive_is_remote(s: &str) -> bool {
    const DRIVE_REMOTE: u32 = 4;
    unsafe extern "system" {
        fn GetDriveTypeW(root: *const u16) -> u32;
    }
    let b = s.as_bytes();
    if b.len() < 2 || b[1] != b':' || !b[0].is_ascii_alphabetic() {
        return false;
    }
    let root = [b[0] as u16, b':' as u16, b'\\' as u16, 0];
    // SAFETY: a NUL-terminated wide string that outlives the call.
    unsafe { GetDriveTypeW(root.as_ptr()) == DRIVE_REMOTE }
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

/// Every `crash-*.txt` report currently on disk, by filename.
///
/// A harness takes this before and after its run: a report that appeared DURING the walk is the
/// one that matters, and the difference is the only way to tell it from one a previous session
/// left behind. (Checklist steps 1 and 102 are exactly this question.)
pub(crate) fn crash_report_names() -> Vec<String> {
    let Some(dir) = logs_dir() else { return Vec::new() };
    let Ok(rd) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut out: Vec<String> = rd
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("crash-") && n.ends_with(".txt"))
        .collect();
    out.sort();
    out
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

/// The newest `crash-view-*.fdn` (filename, contents), if any exists — the exact location a crash
/// happened at, written beside the crash report by [`write_crash_report_at`]. Offered in the issue
/// reporter so a device loss arrives with the coordinates that reproduce it (the crash report's
/// manifest omits them, and after a device-loss relaunch the CURRENT location is Home, not the spot).
pub(crate) fn latest_crash_view() -> Option<(String, String)> {
    let dir = logs_dir()?;
    let mut best: Option<(SystemTime, PathBuf)> = None;
    for entry in std::fs::read_dir(&dir).ok()?.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("crash-view-") && name.ends_with(".fdn") {
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

/// Record one frame-budget decision for the crash report (see [`BUDGET_LOG`]). Cheap and
/// always on: a handful of these per second, bounded to [`BUDGET_LOG_CAP`].
///
/// ⚠Note what a caller must put in the line: the SOURCE of the reading, the measured ms, the
/// dispatch's nominal steps, the budget it was judged against, and the verdict. The verdict alone
/// is not enough — "discarded" and "accepted, no change" look identical afterwards, and telling
/// them apart is the whole question.
pub(crate) fn budget_note(line: String) {
    if let Ok(mut v) = BUDGET_LOG.lock() {
        if v.len() == BUDGET_LOG_CAP {
            v.remove(0);
        }
        v.push(format!("{:8.3}s {line}", elapsed_s()));
    }
}

/// The recorded decisions, oldest first, for the crash report. `None` when nothing was recorded
/// (a crash before any frame priced) so the report can say that rather than print an empty block.
pub(crate) fn budget_history() -> Option<String> {
    let v = BUDGET_LOG.lock().ok()?;
    if v.is_empty() {
        return None;
    }
    Some(v.iter().map(|l| format!("          {l}")).collect::<Vec<_>>().join("\n"))
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

/// The crash-report `hint:` line for a GPU device-loss crash, or `""` for any other crash. A device
/// loss can be a DRIVER bug — the 2026-09-11 parabolic-point loss was deterministic on NVIDIA
/// Vulkan 596.21 and gone on 616.92, no code change — but not always: the 2026-09-12 loss during a
/// 5K×ss4 export at 5.2M iterations happened ON 616.92 (nvlddmkm Event 153, no TDR recovery). So the
/// hint asks for a current driver AND for the report either way, rather than "update first" — a
/// loss on a current driver is exactly the capture issue #1 needs. A panic or OOM gets no such
/// line (it would be noise).
pub(crate) fn device_loss_hint(msg: &str) -> &'static str {
    let m = msg.to_ascii_lowercase();
    // Real device-loss crashes always carry the wgpu wrapper "device lost" (main.rs), but match the
    // vendor spellings too (DXGI is `DEVICE_REMOVED`, underscore) so the hint is robust to rewording.
    if m.contains("device lost")
        || m.contains("device is lost")
        || m.contains("devicelost")
        || m.contains("device removed")
        || m.contains("device_removed")
        || m.contains("deviceremoved")
    {
        "hint    : This is a GPU device loss. Some of these are graphics-driver bugs (one was \
         fixed by a driver update alone), so make sure your driver is current — but not all are, \
         and a loss on a current driver is exactly the capture we need. Either way, please attach \
         this report and the crash-view .fdn beside it to \
         https://github.com/WindySnowOwl/fractadyne/issues/1\n"
    } else {
        ""
    }
}

/// Build the crash report text. Separate from writing it so a test can assert the report actually
/// CARRIES what a reader is told to look for — the budget section exists because a field device
/// loss could not be diagnosed without it, and a section that silently stopped being emitted would
/// be discovered at the worst possible moment, namely the next device loss.
///
/// `frames` is the pre-rendered `frames:` section ([`frame_record::crash_section`]): the live
/// ring's for a crash in this process, the recovered `frames.bin`'s for a previous session's.
fn compose_crash_report(msg: &str, loc: &str, frames: &str) -> String {
    let gpu_hint = device_loss_hint(msg);
    format!(
        "fractadyne crash report\n\
         version : {}\n\
         session : {:016x}\n\
         time    : {}\n\
         uptime  : {:.1}s\n\
         panic   : {msg}\n\
         at      : {loc}\n\
         memory  : {}\n\
         activity: {}\n\
         manifest: {}\n\
         tunables: {}\n\
         bignum  : {}\n\
         thread  : {}\n\
         {}{frames}{}\n\
         backtrace (debug symbols are disabled in this build; addresses only):\n{}\n",
        crate::sysinfo::version_string(),
        frame_record::session_id(),
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
        // The frame-budget decisions leading up to this, oldest first. On a device loss this is
        // the section to read first: it says whether the controller was handed a slow reading and
        // ignored it, or was never handed one at all.
        budget_history().map_or_else(
            || "budget  : no frame-budget decision was recorded before the crash\n".to_string(),
            |h| format!("budget  : the last frame-budget decisions (oldest first)\n{h}\n"),
        ),
        gpu_hint,
        std::backtrace::Backtrace::force_capture(),
    )
}

fn write_crash_report_at(msg: &str, loc: &str) {
    write_crash_report_frames(msg, loc, None);
}

/// Write a crash report whose `frames:` section comes from `recovered` — a previous session's
/// `frames.bin` as `(its header, its records)` — or, when `None`, from this process's live ring.
/// Either way the WHOLE set goes to a companion `crash-<stamp>-frames.jsonl` beside the report
/// (the report itself prints only the last few): that file is the one that can cover an entire
/// slow episode, where the 24-entry decision ring above held about 1.4 s of a 33 s one.
fn write_crash_report_frames(msg: &str, loc: &str, recovered: Option<(String, Vec<frame_record::FrameRecord>)>) {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let n = CRASH_SEQ.fetch_add(1, Ordering::Relaxed);
    let (header, records) = recovered.unwrap_or_else(|| (frame_record::header(), frame_record::snapshot()));
    let dir = LOG_DIR.get().cloned().flatten();
    let companion = match &dir {
        Some(dir) if !records.is_empty() => {
            let name = format!("crash-{secs}-{n}-frames.jsonl");
            frame_record::write_jsonl(&dir.join(&name), &header, &records).ok().map(|_| name)
        }
        _ => None,
    };
    let report = compose_crash_report(msg, loc, &frame_record::crash_section(&records, companion.as_deref()));
    if let Some(dir) = dir {
        let path = dir.join(format!("crash-{secs}-{n}.txt"));
        if std::fs::write(&path, &report).is_ok() {
            let _ = writeln!(std::io::stderr(), "[fd-panic] crash report written: {}", path.display());
        }
        // Beside the report, the crashing VIEW as a loadable `.fdn` — the full-precision coordinates
        // the manifest line omits. This is what turns a field device loss into a reproducible case:
        // `fractadyne --deviceloss-repro --center <center_re> <center_im> --zoom-log2 <log2mag>`, or
        // just File ▸ Open on the file. Formatting happens HERE (once), not on the hot path.
        if let Some(fdn) = crate::crash_view_fdn() {
            let vpath = dir.join(format!("crash-view-{secs}-{n}.fdn"));
            if std::fs::write(&vpath, &fdn).is_ok() {
                let _ = writeln!(std::io::stderr(), "[fd-panic] crash view written: {}", vpath.display());
            }
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
/// This process armed the marker: only it may disarm it (see [`end_session`]).
static MARKER_ARMED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(crate) fn begin_gui_session() {
    if let Some(p) = marker_path() {
        // ⚠Not over a LIVE session's marker. A windowed task the GUI starts (`--render-tour` runs
        // in an event loop too) shares its log folder; arming here overwrote the running
        // session's marker with the task's own and then removed it at the task's exit (measured
        // 2026-10-04) — leaving the session unwatched for the rest of its life.
        if let Some(owner) = std::fs::read_to_string(&p).ok().as_deref().and_then(marker_pid).filter(|&pid| pid != std::process::id() && fractadyne_alive(pid)) {
            log_line("start", &format!("the session marker belongs to the running pid {owner}; this process does not arm it"));
            return;
        }
        MARKER_ARMED.store(true, Ordering::Relaxed);
        // The pid lets any other process that starts against this log folder while the session is
        // alive (a Render tour or farm child, a second instance, a script asking `--version`) see
        // the marker is live, not left behind — see `report_unclean_previous_session`.
        let body = format!(
            "{}\nstarted {}\npid {}\n",
            crate::sysinfo::version_string(),
            crate::sysinfo::now_utc_string(),
            std::process::id()
        );
        let _ = std::fs::write(p, body);
    }
}

/// The `pid N` line of a session marker (markers written before it carried one have none).
pub(crate) fn marker_pid(marker: &str) -> Option<u32> {
    marker.lines().find_map(|l| l.strip_prefix("pid ")?.trim().parse().ok())
}

/// Is `pid` a running Fractadyne process? The name check keeps a dead session's recycled pid,
/// now some other program, from passing for it. `false` where the platform cannot say.
pub(crate) fn fractadyne_alive(pid: u32) -> bool {
    #[cfg(windows)]
    {
        const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
        const STILL_ACTIVE: u32 = 259;
        unsafe extern "system" {
            fn OpenProcess(access: u32, inherit: i32, pid: u32) -> isize;
            fn GetExitCodeProcess(h: isize, code: *mut u32) -> i32;
            fn QueryFullProcessImageNameW(h: isize, flags: u32, name: *mut u16, size: *mut u32) -> i32;
            fn CloseHandle(h: isize) -> i32;
        }
        // SAFETY: plain Win32 calls on a handle we open and close here; the name buffer and its
        // length are ours and outlive the call.
        unsafe {
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if h == 0 {
                return false;
            }
            let mut code = 0u32;
            let running = GetExitCodeProcess(h, &mut code) != 0 && code == STILL_ACTIVE;
            let mut buf = [0u16; 1024];
            let mut len = buf.len() as u32;
            let named = QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut len) != 0
                && String::from_utf16_lossy(&buf[..len as usize])
                    .rsplit(['\\', '/'])
                    .next()
                    .is_some_and(|n| n.to_ascii_lowercase().starts_with("fractadyne"));
            CloseHandle(h);
            running && named
        }
    }
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string(format!("/proc/{pid}/comm")).is_ok_and(|c| c.trim().starts_with("fractadyne"))
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        let _ = pid;
        false
    }
}

/// Disarm the marker. Idempotent; called from [`crate::exit`] and after the event loop returns.
pub(crate) fn end_session() {
    // ⚠Only the process that ARMED it. Every process exits through here, and a child the GUI
    // started (a Render tour, a farm process) shares its log folder: disarming unconditionally,
    // that child deleted the running session's marker as it quit (measured 2026-10-04, after the
    // start-up check had already been taught to leave a live marker alone).
    if let Some(p) = marker_path().filter(|_| MARKER_ARMED.load(Ordering::Relaxed)) {
        let _ = std::fs::remove_file(p);
    }
    // `frames.jsonl` is written by its own thread, which `process::exit` does not wait for: give
    // the rows already queued a moment to land. Bounded — a dead share must not hang a quit.
    frame_record::flush_jsonl_within(std::time::Duration::from_millis(500));
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
    // ⚠A LIVE session's marker, not a dead one's: this process started against the log folder of
    // a GUI that is still running. Every child the GUI starts did exactly that — a Render tour
    // child (reproduced 2026-10-04) deleted the running session's marker, so its own later crash
    // could no longer be noticed, and filed a crash report for a session that was alive.
    if let Some(pid) = marker_pid(&prev).filter(|&pid| pid != std::process::id() && fractadyne_alive(pid)) {
        log_line("start", &format!("a Fractadyne session (pid {pid}) is running against this log folder; its marker is left alone"));
        return;
    }
    let _ = std::fs::remove_file(&p);
    // The dead session's log: rotated into `.1` if this startup rotated it (see ROTATED_AT_START).
    let dead_log = if ROTATED_AT_START.load(Ordering::Relaxed) { "fractadyne.log.1" } else { "fractadyne.log" };
    let tail = LOG_DIR
        .get()
        .cloned()
        .flatten()
        .map(|d| d.join(dead_log))
        .and_then(|f| std::fs::read_to_string(f).ok())
        .map(|s| {
            // Last six lines in CHRONOLOGICAL order — `rev().take()` alone reads newest-first,
            // which is exactly backwards for following what the process was doing as it died.
            let mut last: Vec<&str> = s.lines().rev().take(6).collect();
            last.reverse();
            last.join("\n  ")
        })
        .unwrap_or_default();
    // ⭐The dead session's own frame record, if its `frames.bin` survived. This is the death class
    // that file exists for: a `__fastfail` or an access violation never reaches the panic hook, so
    // until now all that was left of the session was the six log lines above. Read HERE, before
    // this session records a frame and truncates the file.
    let recovered = frame_record::previous_session_frames();
    let frames_note = match &recovered {
        Some(b) => format!(
            " | its frame record survived: session {:016x}, {} records{}",
            b.session,
            b.records.len(),
            if b.torn > 0 { format!(", {} torn slot(s) — the last write was interrupted", b.torn) } else { String::new() }
        ),
        None => " | no frame record survived".to_string(),
    };
    let msg = format!(
        "previous session ended without a clean shutdown (no panic, no crash report) — \
         {}{frames_note} | last log lines:\n  {}",
        prev.lines().collect::<Vec<_>>().join(", "),
        tail
    );
    log_line("unclean", &msg);
    write_crash_report_frames(&msg, "<previous session>", recovered.map(|b| (b.header, b.records)));
    PREV_UNCLEAN.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// Treat a console-initiated shutdown as a clean one.
///
/// ⭐⭐**Closing the terminal window is not a crash, and saying it was devalues the times it is.**
/// The GUI arms a `session.running` marker that only [`end_session`] disarms, so any death that
/// skips it is reported on the next launch — with a crash report file written for it. That backstop
/// is right for the deaths nothing else can see (a `__fastfail`, an access violation, a device
/// loss), but the console window hosting a console-subsystem GUI is something a user closes **on
/// purpose**, and it was landing in the same bucket (user-reported, 2026-09-06). A user who is told
/// "it crashed" every time they close a window stops reading the message that matters.
///
/// ⚠**Windows tells us first.** `CTRL_CLOSE_EVENT` arrives with a grace period (seconds) before the
/// process is terminated, which is ample for one `remove_file` — so this is a real distinction we
/// can draw, not a guess. ⚠**A hard kill still reports**, and must: `TerminateProcess` (Task
/// Manager, `Stop-Process -Force`, a harness cleaning up) delivers no event, so the marker survives
/// and the next launch says so. That is the honest split — *we were told* versus *we were shot*.
///
/// ⚠Returns FALSE from the handler on purpose: the cleanup is done, and the default handler should
/// go on terminating the process exactly as it did before. This changes what is RECORDED, not what
/// happens.
#[cfg(windows)]
fn install_console_ctrl_handler() {
    const CTRL_C_EVENT: u32 = 0;
    const CTRL_BREAK_EVENT: u32 = 1;
    const CTRL_CLOSE_EVENT: u32 = 2;
    const CTRL_LOGOFF_EVENT: u32 = 5;
    const CTRL_SHUTDOWN_EVENT: u32 = 6;

    unsafe extern "system" {
        fn SetConsoleCtrlHandler(
            handler: Option<unsafe extern "system" fn(u32) -> i32>,
            add: i32,
        ) -> i32;
    }

    unsafe extern "system" fn on_ctrl(event: u32) -> i32 {
        let what = match event {
            CTRL_C_EVENT => "Ctrl+C",
            CTRL_BREAK_EVENT => "Ctrl+Break",
            CTRL_CLOSE_EVENT => "console window closed",
            CTRL_LOGOFF_EVENT => "session logoff",
            CTRL_SHUTDOWN_EVENT => "system shutdown",
            _ => return 0,
        };
        // The log still tells the whole story — this is not pretending nothing happened, it is
        // recording what DID happen instead of guessing "crash".
        log_line("exit", &format!("{what} — shutting down (not a crash)"));
        end_session();
        0
    }

    // SAFETY: a plain Win32 registration of a `extern "system"` fn with no arguments of ours; the
    // handler runs on an OS-injected thread and only removes a file and appends a log line, both of
    // which take their own locks.
    unsafe {
        SetConsoleCtrlHandler(Some(on_ctrl), 1);
    }
}

#[cfg(not(windows))]
fn install_console_ctrl_handler() {}

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
        // Then let the `frames.jsonl` writer catch up (bounded: the panic may be ON that thread).
        frame_record::flush_jsonl_within(std::time::Duration::from_millis(250));
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
            // ⚠`None` until the first warning. It used to start at 0 — i.e. "warned at process
            // start" — so the re-warn spacing suppressed EVERY hang in a process's first 30 s:
            // found when `--recordtest` wedged its UI thread for 13 s at +6 s and the watchdog
            // said nothing at all.
            let mut last_warn_ms: Option<u64> = None;
            loop {
                std::thread::sleep(Duration::from_secs(2));
                let Some(t) = START.get() else { continue };
                let now_ms = t.elapsed().as_millis() as u64;
                let alive_ms = ALIVE_MS.load(Ordering::Relaxed);
                let stale_s = now_ms.saturating_sub(alive_ms) / 1000;
                if stale_s >= STALL_S
                    && last_warn_ms.is_none_or(|w| now_ms.saturating_sub(w) >= REWARN_S * 1000)
                {
                    last_warn_ms = Some(now_ms);
                    // Into the frame record too, from THIS thread: the record's only other writer
                    // is the UI thread, which is the one that has stopped. A wedge becomes a row
                    // with a length instead of a gap somebody has to notice.
                    frame_record::record_stall(now_ms.saturating_sub(alive_ms));
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

#[cfg(test)]
mod session_marker_tests {
    use super::*;

    #[test]
    fn a_marker_names_its_pid_and_an_old_one_has_none() {
        assert_eq!(marker_pid("fractadyne 0.3.0-beta.18 (build 1, gabc)\nstarted 2026-10-04\npid 4242\n"), Some(4242));
        assert_eq!(marker_pid("fractadyne 0.3.0-beta.17 (build 1, gabc)\nstarted 2026-10-03\n"), None);
        assert_eq!(marker_pid("pid x\n"), None);
    }

    #[test]
    fn this_process_is_alive_and_a_finished_one_is_not() {
        // The test binary is fractadyne-<hash>: alive, and named like the app.
        assert!(fractadyne_alive(std::process::id()));
        // A process that has exited — and was never Fractadyne.
        #[cfg(windows)]
        let mut c = std::process::Command::new("cmd").args(["/C", "exit 0"]).spawn().expect("cmd runs");
        #[cfg(not(windows))]
        let mut c = std::process::Command::new("true").spawn().expect("true runs");
        let pid = c.id();
        c.wait().expect("it ends");
        assert!(!fractadyne_alive(pid));
        // A RUNNING process that is not Fractadyne: what a dead session's recycled pid looks like.
        #[cfg(windows)]
        {
            let mut other = std::process::Command::new("cmd").args(["/C", "ping -n 4 127.0.0.1 >nul"]).spawn().expect("cmd runs");
            assert!(!fractadyne_alive(other.id()), "a live cmd.exe passed for a Fractadyne session");
            let _ = other.kill();
            let _ = other.wait();
        }
    }
}

#[cfg(test)]
mod log_hygiene_tests {
    use super::LineLimiter;

    /// A flapping transition prints once per gap, and the next line that prints carries the count.
    #[test]
    fn a_rate_limited_line_prints_once_per_gap_and_counts_what_it_held() {
        let mut l = LineLimiter::default();
        assert_eq!(l.admit(0, 5000), Some(0), "the first occurrence always prints");
        for t in (32..4999).step_by(32) {
            assert_eq!(l.admit(t, 5000), None, "held at {t} ms");
        }
        let held = (32..4999).step_by(32).count() as u32;
        assert_eq!(l.admit(5000, 5000), Some(held), "and the next one says how many");
        assert_eq!(l.admit(5001, 5000), None);
        assert_eq!(l.admit(12_000, 5000), Some(1));
    }

    /// Every category the app logs under must appear as `[fd-<cat>]` in DIAGNOSTICS.md's prefix
    /// table — a documentation check that can genuinely go red (design W10). The source is scanned
    /// for `log_line("…"`, `diag::trace("…"` and `trace_on("…"` with a literal category, ignoring
    /// comment lines (a doc example) and the test-only files.
    #[test]
    fn every_log_category_is_documented() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let docs = std::fs::read_to_string(root.join("../../DIAGNOSTICS.md")).expect("DIAGNOSTICS.md");
        let mut files = Vec::new();
        let mut stack = vec![root.join("src")];
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(&d).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    let name = p.file_name().unwrap().to_string_lossy().into_owned();
                    let test_only = name.ends_with("_tests.rs")
                        || ["console.rs", "device_loss_hint.rs", "log_dir.rs"].contains(&name.as_str())
                            && p.parent().is_some_and(|q| q.ends_with("diag"));
                    if !test_only {
                        files.push(p);
                    }
                }
            }
        }
        let mut cats = std::collections::BTreeSet::new();
        for f in &files {
            let text: String = std::fs::read_to_string(f)
                .unwrap()
                .lines()
                .filter(|l| !l.trim_start().starts_with("//"))
                .collect::<Vec<_>>()
                .join("\n");
            for call in ["log_line(", "diag::trace(", "trace_on("] {
                for (i, _) in text.match_indices(call) {
                    let rest = text[i + call.len()..].trim_start();
                    if let Some(lit) = rest.strip_prefix('"') {
                        if let Some(end) = lit.find('"') {
                            let cat = &lit[..end];
                            if !cat.is_empty() && cat.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                                cats.insert(cat.to_string());
                            }
                        }
                    }
                }
            }
        }
        assert!(cats.len() >= 20, "the scan found only {} categories — it has stopped seeing the calls: {cats:?}", cats.len());
        let missing: Vec<_> = cats.iter().filter(|c| !docs.contains(&format!("[fd-{c}]"))).collect();
        assert!(missing.is_empty(), "logged but not in DIAGNOSTICS.md's prefix table: {missing:?}");
    }
}

#[cfg(test)]
mod redact_tests {
    use super::redact_path;

    #[test]
    fn the_home_path_is_redacted_in_every_spelling_it_appears_in() {
        let home = r"C:\Users\alice";
        let text = "logs directed to C:\\Users\\alice\\AppData\\x (--log-dir)\n\
                    config c:/users/ALICE/config.toml\n\
                    json {\"p\":\"C:\\\\Users\\\\alice\\\\f.fdn\"}\n\
                    bare C:\\Users\\alice";
        let out = redact_path(text, home);
        assert!(!out.to_ascii_lowercase().contains("alice"), "{out}");
        assert!(out.contains(r"logs directed to ~\AppData\x"), "{out}");
        assert!(out.contains("config ~/config.toml"), "{out}");
        assert!(out.contains(r#"{"p":"~\\f.fdn"}"#), "{out}");
        assert!(out.ends_with("bare ~"), "{out}");
    }

    #[test]
    fn it_does_not_match_inside_a_longer_name_or_redact_a_trivial_home() {
        assert_eq!(redact_path(r"C:\Users\robin\x", r"C:\Users\rob"), r"C:\Users\robin\x");
        assert_eq!(redact_path(r"C:\Users\rob\x", r"C:\Users\rob"), r"~\x");
        assert_eq!(redact_path(r"C:\a C:\b", r"C:\"), r"C:\a C:\b", "a 3-char home is left alone");
        assert_eq!(redact_path("/home/bob/.config", "/home/bob/"), "~/.config", "trailing separator");
        // Non-ASCII account names survive the ASCII-only case folding without breaking a char.
        assert_eq!(redact_path(r"C:\Users\José\f", r"C:\Users\José"), r"~\f");
    }
}

#[cfg(test)]
mod network_path_tests {
    use super::is_network_path;
    use std::path::Path;

    /// The shapes a share takes on Windows, and the local ones it must not be confused with.
    #[cfg(windows)]
    #[test]
    fn a_unc_path_is_a_share_and_the_temp_dir_is_not() {
        assert!(is_network_path(Path::new(r"\\fileserver\share\Fractadyne\config\logs")));
        assert!(is_network_path(Path::new("//fileserver/share/x")));
        assert!(is_network_path(Path::new(r"\\?\UNC\fileserver\share\x")));
        let tmp = std::env::temp_dir();
        assert!(!is_network_path(&tmp), "{} read as a share", tmp.display());
        let verbatim = std::path::PathBuf::from(format!(r"\\?\{}", tmp.display()));
        assert!(!is_network_path(&verbatim), "a verbatim LOCAL path is not a share: {}", verbatim.display());
        assert!(!is_network_path(Path::new("relative\\logs")));
    }

    #[cfg(not(windows))]
    #[test]
    fn the_temp_dir_is_not_a_share() {
        assert!(!is_network_path(&std::env::temp_dir()));
        assert!(!is_network_path(Path::new("/definitely/not/a/path")));
    }
}

#[cfg(test)]
mod budget_log_tests {
    use super::*;

    /// Held for the WHOLE of each test that touches the global ring. ⚠The ring's own mutex does
    /// not serialise them — it is released between calls — and two of these tests raced: one
    /// clearing the ring and asserting it empty while the other filled it (seen as "decision 10…25"
    /// inside an "empty" report once the test mix changed the scheduling).
    static SERIAL: Mutex<()> = Mutex::new(());

    /// The ring must keep the MOST RECENT decisions and stay bounded: a crash report that
    /// carried the first 24 decisions of a fourteen-minute session would describe the healthy
    /// start and say nothing about the seconds that killed it.
    #[test]
    fn the_ring_keeps_the_last_decisions_and_is_bounded() {
        let _g = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
        if let Ok(mut v) = BUDGET_LOG.lock() {
            v.clear();
        }
        for i in 0..(BUDGET_LOG_CAP + 10) {
            budget_note(format!("decision {i}"));
        }
        let h = budget_history().expect("recorded decisions");
        assert_eq!(h.lines().count(), BUDGET_LOG_CAP);
        assert!(h.contains(&format!("decision {}", BUDGET_LOG_CAP + 9)), "keeps the newest");
        assert!(!h.contains("decision 0 "), "drops the oldest:\n{h}");
        if let Ok(mut v) = BUDGET_LOG.lock() {
            v.clear();
        }
    }

    /// The report must actually carry the budget section, and it must say WHICH of the two
    /// situations held. A device loss where the controller was handed a slow reading and chose to
    /// keep the budget is a different bug from one where it was handed nothing at all, and the
    /// 2026-09-21 RX 6800 XT loss could not be told apart without this.
    #[test]
    fn the_crash_report_carries_the_budget_decisions() {
        let _g = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
        if let Ok(mut v) = BUDGET_LOG.lock() {
            v.clear();
        }
        let empty = compose_crash_report("wgpu device lost (Unknown)", "<device-lost handler>", "");
        assert!(empty.contains("budget  : no frame-budget decision was recorded"), "{empty}");

        budget_note("v0 gpu_iterate=12.0ms steps=4.017e10 budget=1.515e11 DISCARDED".into());
        let filled = compose_crash_report("wgpu device lost (Unknown)", "<device-lost handler>", "");
        assert!(filled.contains("budget  : the last frame-budget decisions"), "{filled}");
        assert!(filled.contains("DISCARDED"), "the decision itself must survive:
{filled}");
        assert!(filled.contains("budget=1.515e11"), "and what it was judged against");
        if let Ok(mut v) = BUDGET_LOG.lock() {
            v.clear();
        }
    }

    /// The report must carry the `frames:` section it is handed, and name the session — the id is
    /// what joins a report to its `frames.bin` and its companion `-frames.jsonl`.
    #[test]
    fn the_crash_report_carries_the_frames_section_and_the_session() {
        let section = frame_record::crash_section(&[], None);
        let r = compose_crash_report("wgpu device lost (Unknown)", "<device-lost handler>", &section);
        assert!(r.contains("frames  : no frame was recorded before the crash"), "{r}");
        assert!(
            r.contains(&format!("session : {:016x}", frame_record::session_id())),
            "the session id joins the report to its frame files:\n{r}"
        );
        // The section sits after the budget block and before the backtrace.
        let (b, f, t) = (r.find("budget  :").unwrap(), r.find("frames  :").unwrap(), r.find("backtrace").unwrap());
        assert!(b < f && f < t, "{r}");
    }

    /// Nothing recorded must read as "nothing recorded", not as an empty section a reader could
    /// mistake for "the controller made no decisions because it was never asked".
    #[test]
    fn an_empty_ring_reports_nothing_rather_than_an_empty_block() {
        let _g = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
        if let Ok(mut v) = BUDGET_LOG.lock() {
            v.clear();
        }
        assert!(budget_history().is_none());
    }
}
