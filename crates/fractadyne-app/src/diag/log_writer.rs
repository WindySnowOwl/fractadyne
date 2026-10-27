//! `fractadyne.log`'s own thread: [`super::log_line`] stamps a line, hands it over and returns.
//!
//! ⭐⭐**A log line must never wait for the disk.** Until 2026-10-27 every line was written by the
//! thread that logged it (most often the UI thread), which opened the file, appended and closed it.
//! On PLUTO (RTX 3070, 2026-10-26, the first run of a freshly unpacked build) one such write took
//! 12.0 s: the line's stderr copy is stamped +16.512 s and its file copy +28.514 s, and the window
//! froze for all of it (frame 504's body: 12,269 ms). The frame record, written to the same disk
//! through a handle it keeps open, took 79 µs that frame. The watchdog saw the hang on time (its
//! STALL row at +27.0 s), and its own log line then queued behind the stuck write. Across 15 PLUTO
//! runs of that day a log write usually took 1-2 ms, and five runs had one of 250-1,265 ms.
//!
//! So this thread keeps the file open, writes what arrives in batches, rotates it, and forces it to
//! the disk when asked ([`LogWriter::sync_within`]); a caller never touches the file. It is BOUNDED:
//! a writer [`QUEUE_LINES`] behind drops new lines, and the first line that gets through again says
//! how many, so a stuck disk costs a counted hole in the log, never a frozen window or memory without
//! limit. A write that took [`SLOW_WRITE_MS`] or more is reported when it lands, so the next stall
//! names itself.
//!
//! ⚠A reader in this process sees a line only once the thread has written it, which is normally
//! within a millisecond. A reader that needs every line logged so far (a harness checking its own
//! log, a clean exit, the panic hook) asks first: [`LogWriter::flush_within`].

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Lines the writer may fall behind by before new ones are dropped: at the busiest measured rate
/// (`FRACTADYNE_TRACE=tile`, ~90 lines a second on PLUTO) about three minutes of a stuck disk, and
/// a few MB of memory at most.
pub(crate) const QUEUE_LINES: usize = 16_384;
/// A write slower than this is reported in the log once it lands. Normal is 1-2 ms (above).
pub(crate) const SLOW_WRITE_MS: u64 = 1_000;
/// Bytes gathered into one write when lines arrive faster than the disk takes them.
const BATCH_BYTES: usize = 256 * 1024;
/// The writer thread's name (the panic hook tells its own panic apart by it).
pub(crate) const THREAD: &str = "fd-log";

/// Where the writer's bytes go: the log file ([`FileSink`]), or a test's stand-in.
pub(crate) trait Sink: Send + 'static {
    /// Append whole lines.
    fn append(&mut self, bytes: &[u8]) -> std::io::Result<()>;
    /// Force what was appended to the disk.
    fn sync(&mut self) -> std::io::Result<()>;
}

/// How a writer runs. [`Config::log_file`] is the app's; a test shortens what it needs to.
pub(crate) struct Config {
    pub(crate) queue_lines: usize,
    pub(crate) slow_write: Duration,
    /// The `[+12.345s]` stamp put on each line as it is handed over.
    pub(crate) stamp: fn() -> String,
    /// Run on the writer thread after each sync: the frame record's files are synced with the log.
    pub(crate) after_sync: fn(),
}

impl Config {
    pub(crate) fn log_file() -> Self {
        Config {
            queue_lines: QUEUE_LINES,
            slow_write: Duration::from_millis(SLOW_WRITE_MS),
            stamp: super::stamp,
            after_sync: super::frame_record::sync_to_disk,
        }
    }
}

enum Msg {
    Line(String),
    /// Answer once everything sent before this has been written (and, with `sync`, synced).
    Flush { sync: bool, ack: SyncSender<()> },
}

/// What became of a line handed to [`LogWriter::send`].
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Sent {
    Queued,
    /// The writer was a full queue behind; counted, and the next line that gets through says so.
    Dropped,
    /// The writer thread has stopped. The stamped line comes back for the caller to write itself.
    Gone(String),
}

/// What became of a [`LogWriter::sync_within`].
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Synced {
    Done,
    /// Asked, and not answered within the limit; the writer still does it.
    NotYet,
    /// An earlier sync has not finished, so this one neither asked nor waited.
    StillPending,
    /// The writer is a full queue behind (stuck): nothing was asked, nothing waited for.
    Behind,
    /// The writer thread has stopped.
    Gone,
}

/// Why a flush or sync could not be asked for.
enum Unasked {
    Behind,
    Gone,
}

struct Shared {
    dropped: AtomicU64,
    syncing: AtomicBool,
}

pub(crate) struct LogWriter {
    /// The queue, under the lock every line is stamped in, so the file's order is the stamps' order.
    /// `None` once the thread has stopped.
    tx: Mutex<Option<SyncSender<Msg>>>,
    shared: Arc<Shared>,
    stamp: fn() -> String,
    queue: usize,
}

impl LogWriter {
    /// Start the thread. `None` if it could not be started.
    pub(crate) fn start(sink: impl Sink, cfg: Config) -> Option<LogWriter> {
        let queue = cfg.queue_lines.max(1);
        let (tx, rx) = std::sync::mpsc::sync_channel(queue);
        let shared = Arc::new(Shared { dropped: AtomicU64::new(0), syncing: AtomicBool::new(false) });
        let theirs = shared.clone();
        let (slow, stamp, after_sync) = (cfg.slow_write, cfg.stamp, cfg.after_sync);
        std::thread::Builder::new()
            .name(THREAD.into())
            .spawn(move || run(sink, rx, theirs, slow, stamp, after_sync))
            .ok()?;
        Some(LogWriter { tx: Mutex::new(Some(tx)), shared, stamp, queue })
    }

    /// Stamp `text` and hand it over. Never waits for the disk.
    pub(crate) fn send(&self, text: &str) -> Sent {
        let mut tx = self.tx.lock().unwrap_or_else(|p| p.into_inner());
        let Some(t) = tx.as_ref() else { return Sent::Gone(format!("{} {text}\n", (self.stamp)())) };
        self.mark_hole(t);
        let line = format!("{} {text}\n", (self.stamp)());
        match t.try_send(Msg::Line(line)) {
            Ok(()) => Sent::Queued,
            Err(TrySendError::Full(_)) => {
                self.shared.dropped.fetch_add(1, Ordering::Relaxed);
                Sent::Dropped
            }
            Err(TrySendError::Disconnected(m)) => {
                *tx = None;
                match m {
                    Msg::Line(line) => Sent::Gone(line),
                    Msg::Flush { .. } => Sent::Gone(String::new()),
                }
            }
        }
    }

    /// Wait up to `limit` for every line handed over so far to reach the OS, room in a full queue
    /// included. `false` if the writer is gone, or did not answer in time.
    pub(crate) fn flush_within(&self, limit: Duration) -> bool {
        let deadline = Instant::now() + limit;
        match self.ask(false, deadline) {
            Ok(ack) => ack.recv_timeout(deadline.saturating_duration_since(Instant::now())).is_ok(),
            Err(_) => false,
        }
    }

    /// Ask for the log, and then the frame record's files, to be forced to the disk, and wait up to
    /// `limit` for it. While an earlier sync is unfinished this neither asks nor waits, and a full
    /// queue (a writer that is stuck) is not waited on either: a disk that has stopped answering
    /// costs a caller one `limit` in all, not one per call.
    pub(crate) fn sync_within(&self, limit: Duration) -> Synced {
        if self.shared.syncing.swap(true, Ordering::AcqRel) {
            return Synced::StillPending;
        }
        match self.ask(true, Instant::now()) {
            Ok(ack) => {
                if ack.recv_timeout(limit).is_ok() {
                    Synced::Done
                } else {
                    Synced::NotYet
                }
            }
            Err(why) => {
                self.shared.syncing.store(false, Ordering::Release);
                match why {
                    Unasked::Behind => Synced::Behind,
                    Unasked::Gone => Synced::Gone,
                }
            }
        }
    }

    /// Queue a flush, retrying while the queue is full until `deadline` (once, if it has passed).
    fn ask(&self, sync: bool, deadline: Instant) -> Result<std::sync::mpsc::Receiver<()>, Unasked> {
        let (ack, answer) = std::sync::mpsc::sync_channel(1);
        let mut msg = Msg::Flush { sync, ack };
        loop {
            {
                let mut tx = self.tx.lock().unwrap_or_else(|p| p.into_inner());
                let Some(t) = tx.as_ref() else { return Err(Unasked::Gone) };
                self.mark_hole(t);
                match t.try_send(msg) {
                    Ok(()) => return Ok(answer),
                    Err(TrySendError::Full(m)) => msg = m,
                    Err(TrySendError::Disconnected(_)) => {
                        *tx = None;
                        return Err(Unasked::Gone);
                    }
                }
            }
            if Instant::now() >= deadline {
                return Err(Unasked::Behind);
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// Lines were dropped since the last line that got through: say how many, AT the hole, ahead of
    /// the line that found room again (or of a flush, so a hole at the end of a run is told too).
    fn mark_hole(&self, t: &SyncSender<Msg>) {
        let n = self.shared.dropped.swap(0, Ordering::Relaxed);
        if n == 0 {
            return;
        }
        let note = format!(
            "{} [fd-log] {n} line(s) dropped here: the log's writer fell {} lines behind\n",
            (self.stamp)(),
            self.queue
        );
        if t.try_send(Msg::Line(note)).is_err() {
            self.shared.dropped.fetch_add(n, Ordering::Relaxed);
        }
    }
}

/// The thread: gather what has arrived (up to [`BATCH_BYTES`]), write it in one go, sync if asked,
/// then answer. Its own notes go straight into the file.
fn run(
    mut sink: impl Sink,
    rx: Receiver<Msg>,
    shared: Arc<Shared>,
    slow: Duration,
    stamp: fn() -> String,
    after_sync: fn(),
) {
    let mut buf: Vec<u8> = Vec::with_capacity(BATCH_BYTES);
    let mut acks: Vec<SyncSender<()>> = Vec::new();
    while let Ok(first) = rx.recv() {
        let mut sync = false;
        let mut next = Some(first);
        while let Some(m) = next.take() {
            match m {
                Msg::Line(l) => buf.extend_from_slice(l.as_bytes()),
                Msg::Flush { sync: s, ack } => {
                    sync |= s;
                    acks.push(ack);
                }
            }
            if buf.len() < BATCH_BYTES {
                next = rx.try_recv().ok();
            }
        }
        let started = Instant::now();
        // The note on a slow write is stamped when the write BEGAN: the lines queued behind it were
        // stamped later, and the file stays in stamp order.
        let began = stamp();
        if !buf.is_empty() {
            let _ = sink.append(&buf);
            buf.clear();
        }
        if sync {
            let _ = sink.sync();
            after_sync();
            shared.syncing.store(false, Ordering::Release);
        }
        let took = started.elapsed();
        if took >= slow {
            let note = format!(
                "{began} [fd-log] this write of the log took {:.1} s: the disk, or a program holding the file, did not answer. Nothing waited for it; lines were queued meanwhile\n",
                took.as_secs_f64()
            );
            let _ = sink.append(note.as_bytes());
        }
        for a in acks.drain(..) {
            let _ = a.send(());
        }
    }
}

/// The log file, kept open: opening it for every line is what stalled (see the module docs).
/// Rotates into `.1` … `.3` past `rotate_bytes`, closed before the rename.
pub(crate) struct FileSink {
    path: PathBuf,
    file: Option<std::fs::File>,
    size: u64,
    rotate_bytes: u64,
}

impl FileSink {
    pub(crate) fn new(path: PathBuf, rotate_bytes: u64) -> Self {
        FileSink { path, file: None, size: 0, rotate_bytes }
    }

    fn file(&mut self) -> std::io::Result<&mut std::fs::File> {
        if self.file.is_none() {
            let f = std::fs::OpenOptions::new().create(true).append(true).open(&self.path)?;
            self.size = f.metadata().map(|m| m.len()).unwrap_or(0);
            self.file = Some(f);
        }
        self.file.as_mut().ok_or_else(|| std::io::Error::other("the log file is not open"))
    }
}

impl Sink for FileSink {
    fn append(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        let r = self.file()?.write_all(bytes);
        match r {
            Ok(()) => self.size += bytes.len() as u64,
            // Reopened for the next batch: a share that came back is written to again.
            Err(_) => self.file = None,
        }
        if self.size > self.rotate_bytes {
            self.file = None;
            if let Some(dir) = self.path.parent() {
                super::rotate_log(dir);
            }
            self.size = 0;
        }
        r
    }

    fn sync(&mut self) -> std::io::Result<()> {
        match self.file.as_ref() {
            Some(f) => f.sync_data(),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/diag/log_writer/tests.rs"]
mod tests;
