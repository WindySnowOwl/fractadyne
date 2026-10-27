//! The log's writer thread (`diag::log_writer`): a line is handed over without waiting for the disk,
//! and none is lost or reordered on the way, except by a counted, marked drop.

use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// How long a stuck test disk stays stuck when the test never frees it: long enough that a caller
/// waiting for it is unmistakable, short enough that a regression fails instead of hanging.
const STUCK_FOR: Duration = Duration::from_secs(3);

/// A stand-in for the disk: keeps what is written, and can be stuck (every write and sync waits
/// until [`Disk::free`]) or slow.
#[derive(Clone)]
struct Disk {
    bytes: Arc<Mutex<Vec<u8>>>,
    free: Arc<(Mutex<bool>, Condvar)>,
    delay: Duration,
    syncs: Arc<AtomicU64>,
    /// Writes begun: the writer thread is inside one (and, on a stuck disk, stays there).
    entered: Arc<AtomicU64>,
}

impl Disk {
    fn new() -> Self {
        Disk {
            bytes: Arc::new(Mutex::new(Vec::new())),
            free: Arc::new((Mutex::new(true), Condvar::new())),
            delay: Duration::ZERO,
            syncs: Arc::new(AtomicU64::new(0)),
            entered: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Hand `w` a line and wait until the writer is inside the write, stuck: from then on nothing
    /// leaves the queue, so a test can fill it exactly.
    fn hold(&self, w: &LogWriter) {
        assert_eq!(w.send("line 0"), Sent::Queued);
        let t = Instant::now();
        while self.entered.load(Ordering::SeqCst) == 0 {
            assert!(t.elapsed() < Duration::from_secs(5), "the writer never began a write");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// Send `line 1`, `line 2`, ... until one is dropped (the queue is full); the numbers queued.
    fn fill(&self, w: &LogWriter) -> Vec<u32> {
        let mut queued = vec![0];
        for i in 1..1000 {
            match w.send(&format!("line {i}")) {
                Sent::Queued => queued.push(i),
                Sent::Dropped => return queued,
                Sent::Gone(_) => panic!("the writer stopped"),
            }
        }
        panic!("the queue never filled behind a stuck disk");
    }

    fn stuck() -> Self {
        let d = Disk::new();
        *d.free.0.lock().unwrap() = false;
        d
    }

    fn slow(delay: Duration) -> Self {
        Disk { delay, ..Disk::new() }
    }

    fn free(&self) {
        *self.free.0.lock().unwrap() = true;
        self.free.1.notify_all();
    }

    fn wait(&self) {
        let (m, cv) = &*self.free;
        let g = m.lock().unwrap();
        let _ = cv.wait_timeout_while(g, STUCK_FOR, |free| !*free).unwrap();
    }

    fn text(&self) -> String {
        String::from_utf8(self.bytes.lock().unwrap().clone()).unwrap()
    }
}

impl Sink for Disk {
    fn append(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.entered.fetch_add(1, Ordering::SeqCst);
        self.wait();
        std::thread::sleep(self.delay);
        self.bytes.lock().unwrap().extend_from_slice(bytes);
        Ok(())
    }

    fn sync(&mut self) -> std::io::Result<()> {
        self.wait();
        self.syncs.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

/// A stamp that only goes up, so the file's order can be checked against the order of stamping.
static TICK: AtomicU64 = AtomicU64::new(0);
fn tick() -> String {
    format!("[{:010}]", TICK.fetch_add(1, Ordering::SeqCst))
}

fn nothing() {}

fn cfg(queue: usize) -> Config {
    Config { queue_lines: queue, slow_write: Duration::from_secs(60), stamp: tick, after_sync: nothing }
}

/// The numbers of the `line N` lines, in file order.
fn numbers(text: &str) -> Vec<u32> {
    text.lines().filter_map(|l| l.split_once("] line ").and_then(|(_, n)| n.parse().ok())).collect()
}

#[test]
fn a_line_is_handed_over_while_the_disk_is_stuck() {
    let disk = Disk::stuck();
    let w = LogWriter::start(disk.clone(), cfg(64)).unwrap();
    let t = Instant::now();
    for i in 0..50 {
        assert_eq!(w.send(&format!("line {i}")), Sent::Queued);
    }
    let took = t.elapsed();
    assert!(
        took < Duration::from_millis(500),
        "50 lines took {took:?} to hand over while the disk was stuck: logging is waiting for the disk again"
    );
    disk.free();
    assert!(w.flush_within(Duration::from_secs(5)));
    assert_eq!(numbers(&disk.text()), (0..50).collect::<Vec<_>>(), "every line, in order");
}

#[test]
fn a_full_queue_drops_lines_and_marks_the_hole() {
    let disk = Disk::stuck();
    let w = LogWriter::start(disk.clone(), cfg(4)).unwrap();
    disk.hold(&w);
    let queued = disk.fill(&w);
    assert_eq!(queued.len(), 5, "the line being written and a 4-line queue");
    let mut dropped = 1;
    for i in 0..9 {
        assert_eq!(w.send(&format!("dropped {i}")), Sent::Dropped);
        dropped += 1;
    }
    disk.free();
    assert!(w.flush_within(Duration::from_secs(5)));
    // The line after the hole carries the note, ahead of it.
    assert_eq!(w.send("line 99"), Sent::Queued);
    assert!(w.flush_within(Duration::from_secs(5)));
    let text = disk.text();
    let mut want = queued.clone();
    want.push(99);
    assert_eq!(numbers(&text), want, "what was queued arrives, in order");
    let note = format!("[fd-log] {dropped} line(s) dropped here");
    let at = text.find(&note).unwrap_or_else(|| panic!("no `{note}` in:\n{text}"));
    let last_queued = text.find(&format!("] line {}\n", queued.last().unwrap())).unwrap();
    assert!(last_queued < at && at < text.find("] line 99").unwrap(), "the note is AT the hole:\n{text}");
}

#[test]
fn a_hole_at_the_end_is_marked_by_the_flush() {
    let disk = Disk::stuck();
    let w = LogWriter::start(disk.clone(), cfg(2)).unwrap();
    disk.hold(&w);
    disk.fill(&w);
    assert_eq!(w.send("dropped too"), Sent::Dropped);
    disk.free();
    assert!(w.flush_within(Duration::from_secs(5)));
    let text = disk.text();
    assert!(text.contains("[fd-log] 2 line(s) dropped here"), "{text}");
    assert!(text.trim_end().ends_with("dropped here: the log's writer fell 2 lines behind"), "the note ends the log:\n{text}");
}

#[test]
fn a_slow_write_is_reported_once_it_lands_in_stamp_order() {
    let disk = Disk::slow(Duration::from_millis(150));
    let w = LogWriter::start(disk.clone(), Config { slow_write: Duration::from_millis(50), ..cfg(16) }).unwrap();
    disk.hold(&w);
    for i in 1..=5 {
        assert_eq!(w.send(&format!("line {i}")), Sent::Queued);
    }
    assert!(w.flush_within(Duration::from_secs(5)));
    let text = disk.text();
    assert!(text.contains("[fd-log] this write of the log took"), "{text}");
    assert_eq!(numbers(&text), (0..=5).collect::<Vec<_>>());
    let stamps: Vec<u64> = text.lines().map(|l| l[1..11].parse().unwrap()).collect();
    assert!(stamps.windows(2).all(|p| p[0] < p[1]), "the note broke the file's stamp order:\n{text}");

    let quick = Disk::new();
    let w = LogWriter::start(quick.clone(), Config { slow_write: Duration::from_millis(50), ..cfg(16) }).unwrap();
    w.send("line 0");
    assert!(w.flush_within(Duration::from_secs(5)));
    assert!(!quick.text().contains("[fd-log]"), "a quick write is not reported: {}", quick.text());
}

#[test]
fn a_flush_gives_up_at_its_limit() {
    let disk = Disk::stuck();
    let w = LogWriter::start(disk.clone(), cfg(16)).unwrap();
    w.send("line 0");
    let t = Instant::now();
    assert!(!w.flush_within(Duration::from_millis(100)), "answered while the disk was stuck");
    assert!(t.elapsed() < Duration::from_secs(1), "waited {:?} for a 100 ms limit", t.elapsed());
    disk.free();
    assert!(w.flush_within(Duration::from_secs(5)));
    assert_eq!(numbers(&disk.text()), vec![0]);
}

static AFTER_SYNC: AtomicU64 = AtomicU64::new(0);
fn count_after_sync() {
    AFTER_SYNC.fetch_add(1, Ordering::SeqCst);
}

#[test]
fn a_stuck_disk_costs_one_sync_wait_not_one_per_call() {
    let disk = Disk::stuck();
    let w = LogWriter::start(disk.clone(), Config { after_sync: count_after_sync, ..cfg(16) }).unwrap();
    w.send("line 0");
    let t = Instant::now();
    assert_eq!(w.sync_within(Duration::from_millis(100)), Synced::NotYet);
    assert!(t.elapsed() < Duration::from_secs(1));
    let t = Instant::now();
    assert_eq!(w.sync_within(Duration::from_millis(100)), Synced::StillPending, "asked again while the first was unfinished");
    assert!(t.elapsed() < Duration::from_millis(50), "waited {:?} although a sync was already pending", t.elapsed());
    disk.free();
    assert!(w.flush_within(Duration::from_secs(5)));
    assert_eq!(disk.syncs.load(Ordering::SeqCst), 1, "the pending call must not have queued a second sync");
    assert_eq!(AFTER_SYNC.load(Ordering::SeqCst), 1, "the frame record is synced with the log");
    assert_eq!(w.sync_within(Duration::from_secs(5)), Synced::Done, "a sync that finished frees the next");
    assert_eq!(disk.syncs.load(Ordering::SeqCst), 2);
}

#[test]
fn a_sync_behind_a_full_queue_does_not_wait() {
    let disk = Disk::stuck();
    let w = LogWriter::start(disk.clone(), cfg(2)).unwrap();
    disk.hold(&w);
    disk.fill(&w);
    let t = Instant::now();
    assert_eq!(w.sync_within(Duration::from_millis(500)), Synced::Behind);
    assert!(t.elapsed() < Duration::from_millis(50), "waited {:?} on a writer that is stuck", t.elapsed());
    disk.free();
    assert!(w.flush_within(Duration::from_secs(5)), "a flush waits for room as the queue drains");
    assert_eq!(w.sync_within(Duration::from_secs(5)), Synced::Done, "being behind left no sync pending");
}

#[test]
fn lines_from_many_threads_land_in_the_order_they_were_stamped() {
    let disk = Disk::new();
    let w = Arc::new(LogWriter::start(disk.clone(), cfg(4096)).unwrap());
    let threads: Vec<_> = (0..4)
        .map(|k| {
            let w = w.clone();
            std::thread::spawn(move || {
                for i in 0..200 {
                    assert_eq!(w.send(&format!("line {}", k * 1000 + i)), Sent::Queued);
                }
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
    assert!(w.flush_within(Duration::from_secs(5)));
    let text = disk.text();
    let stamps: Vec<u64> = text.lines().map(|l| l[1..11].parse().unwrap()).collect();
    assert_eq!(stamps.len(), 800);
    assert!(stamps.windows(2).all(|p| p[0] < p[1]), "a line landed ahead of one stamped before it");
}

/// A disk that fails the way a dead writer thread does: the first write panics.
struct Broken;
impl Sink for Broken {
    fn append(&mut self, _: &[u8]) -> std::io::Result<()> {
        panic!("test: the log's writer thread dies here");
    }
    fn sync(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn a_stopped_writer_hands_the_line_back() {
    let w = LogWriter::start(Broken, cfg(16)).unwrap();
    let _ = w.send("line 0");
    let t = Instant::now();
    loop {
        match w.send("line 1") {
            Sent::Gone(line) => {
                assert!(line.ends_with("] line 1\n"), "the stamped line comes back whole: {line:?}");
                break;
            }
            _ => assert!(t.elapsed() < Duration::from_secs(5), "the writer died and nothing noticed"),
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(matches!(w.send("line 2"), Sent::Gone(_)), "and keeps coming back");
    assert!(!w.flush_within(Duration::from_millis(100)));
}

#[test]
fn the_file_keeps_every_line_across_a_rotation() {
    let dir = std::env::temp_dir().join(format!("fd-log-writer-test-{}-{}", std::process::id(), TICK.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("fractadyne.log");
    let w = LogWriter::start(FileSink::new(path.clone(), 2_000), cfg(64)).unwrap();
    for i in 0..60 {
        w.send(&format!("line {i} {}", "x".repeat(20)));
        assert!(w.flush_within(Duration::from_secs(5)));
    }
    let rotated = std::fs::read_to_string(dir.join("fractadyne.log.1")).expect("rotated past 2,000 bytes");
    let current = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(!dir.join("fractadyne.log.2").exists(), "rotated more than once for ~3,000 bytes");
    let nums = |t: &str| -> Vec<u32> {
        t.lines().filter_map(|l| l.split_once("] line ").and_then(|(_, r)| r.split(' ').next()?.parse().ok())).collect()
    };
    let mut all = nums(&rotated);
    assert!(!all.is_empty() && !current.is_empty(), "both sides of the rotation hold lines");
    all.extend(nums(&current));
    assert_eq!(all, (0..60).collect::<Vec<_>>(), "no line lost or reordered by the rotation");
    drop(w);
    let _ = std::fs::remove_dir_all(&dir);
}
