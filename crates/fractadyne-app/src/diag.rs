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
    let dir = if file_log_on {
        fractadyne_state::config_dir().map(|d| d.join("logs"))
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
    }
    log_line(
        "start",
        &format!(
            "fractadyne {} — {} — args: {}",
            crate::sysinfo::version_string(),
            crate::sysinfo::now_utc_string(),
            args.iter().skip(1).cloned().collect::<Vec<_>>().join(" "),
        ),
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
pub(crate) fn log_line(cat: &str, msg: &str) {
    eprintln!("[fd-{cat}] {} {msg}", stamp());
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
/// panic hook and watchdog. Also stamps liveness and (at debug volume) the log file.
pub(crate) fn breadcrumb(msg: String) {
    alive();
    file_line(&format!("[crumb] {msg}"));
    if let Ok(mut b) = BREADCRUMB.lock() {
        *b = msg;
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
        let report = format!(
            "fractadyne crash report\n\
             version : {}\n\
             time    : {}\n\
             uptime  : {:.1}s\n\
             panic   : {msg}\n\
             at      : {loc}\n\
             activity: {}\n\
             manifest: {}\n\
             thread  : {}\n\n\
             backtrace (debug symbols are disabled in this build; addresses only):\n{}\n",
            crate::sysinfo::version_string(),
            crate::sysinfo::now_utc_string(),
            elapsed_s(),
            current_breadcrumb(),
            MANIFEST.lock().map(|m| m.clone()).unwrap_or_default(),
            std::thread::current().name().unwrap_or("<unnamed>"),
            std::backtrace::Backtrace::force_capture(),
        );
        // The log line survives even if the crash file can't be written.
        log_line("panic", &format!("{msg} at {loc} — activity: {}", current_breadcrumb()));
        if let Some(Some(dir)) = LOG_DIR.get() {
            let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
            let path = dir.join(format!("crash-{secs}.txt"));
            if std::fs::write(&path, &report).is_ok() {
                eprintln!("[fd-panic] crash report written: {}", path.display());
            }
        }
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
            'outer: while !stop2.load(Ordering::Relaxed) {
                // ~2 s cadence, but check `stop` every 100 ms so Drop never stalls.
                for _ in 0..20 {
                    std::thread::sleep(Duration::from_millis(100));
                    if stop2.load(Ordering::Relaxed) {
                        break 'outer;
                    }
                }
                alive();
                let p = progress.load(Ordering::Relaxed).min(1000);
                eprint!("\r[fd-progress] {} {label} {:3}%", stamp(), p / 10);
                let _ = std::io::stderr().flush();
                printed = true;
            }
            if printed {
                eprintln!();
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
