//! The reference-orbit cache on disk — `<config_dir>/orbits/<key>.orbit`, one entry per orbit.
//!
//! ⭐⭐**The problem it solves.** Returning to a location already visited at extreme depth costs
//! 30-60+ minutes on the author's machine, all of it rebuilding a reference orbit that already
//! existed once. The orbit the GPU consumes is 16 bytes per iteration — 4 MB at the live cap — so
//! it is kept, keyed on the reference POINT, and handed back to the recompute path as the same
//! [`ReuseRef`](crate::render::ReuseRef) the live deep-dive reuse already uses. Everything from
//! that seam on is existing, gated machinery.
//!
//! ⭐**Lookup is by NEIGHBOURHOOD, and it happens BEFORE the reference pick.** An entry serves any
//! view its point is inside of, at any precision up to the one it was built at: come back, then
//! zoom somewhere nearby, no rebuild. The design first put the lookup AFTER `pick_reference`
//! (key on the picked point); that was wrong by the numbers already in the tree — at 2.37e4000×
//! the candidate scoring cost 113.7 s against 32.8 s for the orbit itself — so a cache that paid
//! the pick to find its key would leave most of the saving on the table. The admissibility test is
//! the recompute path's own (`reuse_drift` in `render.rs`), passed in as a closure, so this module
//! can never accept an entry that path would refuse.
//!
//! ⭐**Eviction is by BUILD COST, not recency.** A dive writes a stream of cheap entries; under LRU
//! they would evict the hour-long orbit visited last week before any of themselves. The value of
//! an entry is the time it saves, and `orbit_len × precision²` is that time up to a machine
//! constant, so the cheapest orbit goes first and recency only breaks ties.
//!
//! **What is NOT stored**: the series approximation and the BLA — derived, cheap next to the orbit
//! (measured at 2.37e4000×: BLA 0.4 s of a 405 s build; SA is skipped whenever BLA builds), and
//! colouring-dependent. See `render::orbit_blob`.
//!
//! **History.** Until beta.77 this module kept ONE reference — the last view's, keyed on the exact
//! centre and zoom, as decimal strings, without a tail — so a restored session rendered at once.
//! That is now the trivial case of a cache hit: the restored view's cold start finds its own orbit
//! here. The old `last_reference.bin` is removed on startup.
//!
//! **Safety.** Every entry is refused unless it verifies (`orbit_blob`): a damaged orbit renders a
//! plausible picture of the wrong place, so nothing here repairs. Writes are temp-then-rename, so
//! a crash mid-write cannot leave a half entry. Unreadable files that carry our magic are deleted
//! (they can never be used); anything else in the directory is left alone.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use crate::render::orbit_blob::{self, DecodedOrbit, Header, OrbitKey};
use fractadyne_core::BigFloat;

/// Directory under the config dir.
pub(crate) const DIR_NAME: &str = "orbits";
/// Entry file extension.
pub(crate) const ENTRY_EXT: &str = "orbit";
/// The one-slot file this module wrote before beta.78; deleted on startup.
const LEGACY_FILE: &str = "last_reference.bin";

/// One indexed entry: what its header says, where it is, and how big it is.
pub(crate) struct Entry {
    pub(crate) path: PathBuf,
    pub(crate) header: Header,
    pub(crate) bytes: u64,
    /// Last hit (or write) — the file's modification time, touched on every hit.
    pub(crate) last_used: SystemTime,
}

impl Entry {
    fn cost(&self) -> f64 {
        entry_cost(&self.header)
    }
    fn id(&self) -> u64 {
        self.header.key_id()
    }
}

/// What a lookup asks for: the view identity, and the recompute path's own admissibility test —
/// given a candidate's point and build precision, the drift in spans if it may be reused.
pub(crate) struct Query<'a> {
    pub(crate) key: OrbitKey,
    pub(crate) fits: &'a dyn Fn(&[BigFloat; 2], usize) -> Option<f64>,
}

/// A lookup's verdict, for the trace line and the gate.
pub(crate) struct Hit {
    pub(crate) path: PathBuf,
    pub(crate) orbit_len: u32,
    pub(crate) prec: usize,
    pub(crate) drift: f64,
    pub(crate) candidates: usize,
}

/// Usage for the controls.
#[derive(Clone, Copy, Default)]
pub(crate) struct Usage {
    pub(crate) bytes: u64,
    pub(crate) entries: usize,
}

struct Store {
    enabled: bool,
    /// Where the entries live when not the config dir (`--selftest` points this at a scratch dir).
    dir_override: Option<PathBuf>,
    budget: u64,
    /// `None` until the directory has been scanned.
    entries: Option<Vec<Entry>>,
}

static STORE: Mutex<Store> = Mutex::new(Store {
    enabled: false,
    dir_override: None,
    budget: crate::tunables::ORBIT_CACHE_DEFAULT_MB as u64 * 1024 * 1024,
    entries: None,
});

/// Outstanding background writers, so a caller that must observe its own write can wait for it.
static WRITERS: Mutex<Vec<std::thread::JoinHandle<()>>> = Mutex::new(Vec::new());

fn lock() -> std::sync::MutexGuard<'static, Store> {
    STORE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Turn the cache on or off for this process. OFF for harnesses and offline jobs (a cached orbit
/// makes a timed run look faster than the code is, and a gate must build what it measures); ON for
/// a session someone sits in front of.
pub(crate) fn set_enabled(on: bool) {
    lock().enabled = on;
}

pub(crate) fn enabled() -> bool {
    lock().enabled
}

/// Size budget in bytes; the store evicts down to it on every write and on every change.
pub(crate) fn set_budget_bytes(b: u64) {
    let mut s = lock();
    s.budget = b;
    if let Some(dir) = s.dir() {
        s.ensure_scanned(&dir);
        s.evict_to_fit();
    }
}

/// The directory entries live in, or `None` with no config dir.
pub(crate) fn dir() -> Option<PathBuf> {
    lock().dir()
}

/// Point the store at another directory (the selftest's scratch), or back at the config dir with
/// `None`. Drops the index either way.
pub(crate) fn set_dir_override(dir: Option<PathBuf>) {
    let mut s = lock();
    s.dir_override = dir;
    s.entries = None;
}

/// Bytes on disk and entry count. Scans the directory on first call.
pub(crate) fn usage() -> Usage {
    let mut s = lock();
    let Some(dir) = s.dir() else { return Usage::default() };
    s.ensure_scanned(&dir);
    s.usage()
}

/// The indexed entries, cheapest first (the order eviction would take them), for the controls.
pub(crate) fn entries_summary() -> Vec<(u32, usize, u64, SystemTime)> {
    let mut s = lock();
    let Some(dir) = s.dir() else { return Vec::new() };
    s.ensure_scanned(&dir);
    let mut v: Vec<&Entry> = s.entries.as_deref().unwrap_or(&[]).iter().collect();
    v.sort_by(|a, b| a.cost().total_cmp(&b.cost()).then(a.last_used.cmp(&b.last_used)));
    v.iter().map(|e| (e.header.orbit_len, e.header.prec, e.bytes, e.last_used)).collect()
}

/// Delete every entry. Returns how many were removed. ⭐Costs time, never data: everything here
/// can be rebuilt by visiting the location again.
pub(crate) fn clear() -> std::io::Result<usize> {
    let mut s = lock();
    let Some(dir) = s.dir() else { return Ok(0) };
    s.ensure_scanned(&dir);
    let mut n = 0;
    if let Some(entries) = s.entries.take() {
        for e in entries {
            if std::fs::remove_file(&e.path).is_ok() {
                n += 1;
            }
        }
    }
    // Stray temp files from an interrupted write go too.
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for f in rd.flatten() {
            if f.path().extension().is_some_and(|x| x == "tmp") {
                let _ = std::fs::remove_file(f.path());
            }
        }
    }
    s.entries = Some(Vec::new());
    crate::diag::log_line("cache", &format!("orbit cache cleared: {n} entries removed from {}", dir.display()));
    Ok(n)
}

/// Scan the directory on a background thread (and drop the pre-beta.78 one-slot file), so the
/// first deep build does not pay for the index. Called once at startup when the cache is on.
pub(crate) fn warm() {
    std::thread::spawn(|| {
        if let Some(cfg) = fractadyne_state::config_dir() {
            let _ = std::fs::remove_file(cfg.join(LEGACY_FILE));
        }
        let mut s = lock();
        if let Some(dir) = s.dir() {
            s.ensure_scanned(&dir);
            let u = s.usage();
            crate::diag::log_line(
                "cache",
                &format!(
                    "orbit cache: {} entries, {:.1} MB of {:.0} MB in {}",
                    u.entries,
                    u.bytes as f64 / 1_048_576.0,
                    s.budget as f64 / 1_048_576.0,
                    dir.display()
                ),
            );
        }
    });
}

/// Would an orbit of this identity and length be written? The cheap pre-check the worker runs
/// BEFORE cloning and encoding 4 MB: a build under the threshold with no entry to replace, or one
/// no longer than what is already stored, is not worth the work.
pub(crate) fn wanted(id: u64, orbit_len: u32, build_ms: f64) -> bool {
    let mut s = lock();
    if !s.enabled {
        return false;
    }
    let Some(dir) = s.dir() else { return false };
    s.ensure_scanned(&dir);
    match s.entries.as_deref().and_then(|v| v.iter().find(|e| e.id() == id)) {
        Some(e) => e.header.orbit_len < orbit_len,
        None => build_ms >= crate::tunables::ORBIT_CACHE_MIN_BUILD_MS,
    }
}

/// Find the best usable entry for `q`: among the entries whose identity matches and whose point
/// the caller's test admits, the LONGEST orbit (least to extend), ties to the closest point.
pub(crate) fn find(q: &Query) -> Option<Hit> {
    let mut s = lock();
    if !s.enabled {
        return None;
    }
    let dir = s.dir()?;
    s.ensure_scanned(&dir);
    let entries = s.entries.as_deref()?;
    let mut best: Option<(&Entry, f64)> = None;
    let mut candidates = 0usize;
    for e in entries {
        if e.header.key != q.key {
            continue;
        }
        candidates += 1;
        let Some(drift) = (q.fits)(&e.header.point, e.header.prec) else { continue };
        let better = match best {
            None => true,
            Some((b, bd)) => {
                e.header.orbit_len > b.header.orbit_len
                    || (e.header.orbit_len == b.header.orbit_len && drift < bd)
            }
        };
        if better {
            best = Some((e, drift));
        }
    }
    best.map(|(e, drift)| Hit {
        path: e.path.clone(),
        orbit_len: e.header.orbit_len,
        prec: e.header.prec,
        drift,
        candidates,
    })
}

/// Read and verify an entry found by [`find`]. A file that no longer verifies is removed from disk
/// and from the index, and `None` is returned — the caller builds fresh, and the next lookup will
/// not see it again. A hit touches the file's modification time (its last-used stamp).
pub(crate) fn load(path: &Path) -> Option<DecodedOrbit> {
    match std::fs::read(path).ok().and_then(|b| orbit_blob::decode(&b)) {
        Some(d) => {
            let now = SystemTime::now();
            if let Ok(f) = std::fs::OpenOptions::new().write(true).open(path) {
                let _ = f.set_modified(now);
            }
            let mut s = lock();
            if let Some(e) = s.entries.as_deref_mut().and_then(|v| v.iter_mut().find(|e| e.path == path)) {
                e.last_used = now;
            }
            Some(d)
        }
        None => {
            crate::diag::log_line(
                "cache",
                &format!("orbit cache entry {} did not verify — removed", path.display()),
            );
            let _ = std::fs::remove_file(path);
            let mut s = lock();
            if let Some(v) = s.entries.as_mut() {
                v.retain(|e| e.path != path);
            }
            None
        }
    }
}

/// Write an encoded entry (temp-then-rename), index it, and evict down to the budget. Replaces an
/// existing entry of the same identity only if this one is LONGER. Returns the path written, or
/// `None` when it was not (cache off, no directory, or nothing gained).
pub(crate) fn offer(bytes: Vec<u8>) -> std::io::Result<Option<PathBuf>> {
    let (header, _) = orbit_blob::read_header(&bytes)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "not an orbit blob"))?;
    let mut s = lock();
    if !s.enabled {
        return Ok(None);
    }
    let Some(dir) = s.dir() else { return Ok(None) };
    s.ensure_scanned(&dir);
    let id = header.key_id();
    if let Some(e) = s.entries.as_deref().and_then(|v| v.iter().find(|e| e.id() == id)) {
        if e.header.orbit_len >= header.orbit_len {
            return Ok(None);
        }
    }
    let bytes_len = bytes.len() as u64;
    // ⭐Would it survive its own eviction pass? An entry cheaper than what the budget already
    // holds would be written and then evicted at once; refuse it up front instead. This is what
    // keeps a dive's stream of cheap orbits from ever displacing the hour-long one.
    if !s.survives(id, entry_cost(&header), bytes_len) {
        return Ok(None);
    }
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{id:016x}.{ENTRY_EXT}"));
    let tmp = dir.join(format!("{id:016x}.{ENTRY_EXT}.tmp"));
    std::fs::write(&tmp, &bytes)?;
    std::fs::rename(&tmp, &path)?;
    let entries = s.entries.get_or_insert_with(Vec::new);
    entries.retain(|e| e.id() != id);
    entries.push(Entry { path: path.clone(), header, bytes: bytes_len, last_used: SystemTime::now() });
    s.evict_to_fit();
    Ok(Some(path))
}

/// ⭐The eviction rank: proportional to the bignum work the orbit took, so the cheapest orbit is
/// the one evicted. `orbit_len` steps, each a multiply that scales with precision².
fn entry_cost(h: &Header) -> f64 {
    h.orbit_len as f64 * (h.prec as f64) * (h.prec as f64)
}

/// Encode and write on a detached thread, so the worker that built the orbit returns at once.
pub(crate) fn offer_async(res: crate::render::RecomputeResult, key: OrbitKey, origin: &'static str) {
    let h = std::thread::spawn(move || {
        let t = std::time::Instant::now();
        let Some(bytes) = orbit_blob::encode(&res, key) else { return };
        let n = bytes.len();
        let (len, prec) = orbit_blob::read_header(&bytes)
            .map(|(h, _)| (h.orbit_len, h.prec))
            .unwrap_or((0, 0));
        match offer(bytes) {
            Ok(Some(p)) => crate::diag::log_line(
                "cache",
                &format!(
                    "orbit cache WRITE [{origin}]: {} ({:.1} MB, len={len} prec={prec}) in {:.0} ms",
                    p.file_name().and_then(|f| f.to_str()).unwrap_or("?"),
                    n as f64 / 1_048_576.0,
                    t.elapsed().as_secs_f64() * 1000.0
                ),
            ),
            Ok(None) => {}
            Err(e) => crate::diag::log_line("cache", &format!("orbit cache write failed: {e}")),
        }
    });
    let mut w = WRITERS.lock().unwrap_or_else(|e| e.into_inner());
    w.retain(|h| !h.is_finished());
    w.push(h);
}

/// Wait for every background write started by [`offer_async`]. For a caller that must observe
/// its own writes — the selftest — and for a clean shutdown.
pub(crate) fn drain() {
    let handles: Vec<_> = std::mem::take(&mut *WRITERS.lock().unwrap_or_else(|e| e.into_inner()));
    for h in handles {
        let _ = h.join();
    }
}

impl Store {
    fn dir(&self) -> Option<PathBuf> {
        self.dir_override
            .clone()
            .or_else(|| fractadyne_state::config_dir().map(|d| d.join(DIR_NAME)))
    }

    fn usage(&self) -> Usage {
        let v = self.entries.as_deref().unwrap_or(&[]);
        Usage { bytes: v.iter().map(|e| e.bytes).sum(), entries: v.len() }
    }

    fn ensure_scanned(&mut self, dir: &Path) {
        if self.entries.is_some() {
            return;
        }
        self.entries = Some(scan(dir));
    }

    /// Would a candidate entry of this cost and size still be there after the eviction pass its
    /// own write would trigger? Everything cheaper than it (an existing entry of EQUAL cost is
    /// older, so it goes first) is evicted before it would be; if the budget still does not fit
    /// it after that, it would be evicted too.
    fn survives(&self, id: u64, cost: f64, bytes: u64) -> bool {
        let others = self.entries.as_deref().unwrap_or(&[]).iter().filter(|e| e.id() != id);
        let (mut total, mut cheaper) = (0u64, 0u64);
        for e in others {
            total += e.bytes;
            if e.cost() <= cost {
                cheaper += e.bytes;
            }
        }
        total - cheaper + bytes <= self.budget
    }

    /// Remove the cheapest entries (ties: least recently used first) until the total fits the
    /// budget.
    fn evict_to_fit(&mut self) {
        let budget = self.budget;
        let Some(entries) = self.entries.as_mut() else { return };
        let mut total: u64 = entries.iter().map(|e| e.bytes).sum();
        if total <= budget {
            return;
        }
        let mut order: Vec<usize> = (0..entries.len()).collect();
        order.sort_by(|&a, &b| {
            entries[a]
                .cost()
                .total_cmp(&entries[b].cost())
                .then(entries[a].last_used.cmp(&entries[b].last_used))
        });
        let mut doomed = Vec::new();
        for i in order {
            if total <= budget {
                break;
            }
            total -= entries[i].bytes;
            doomed.push(i);
        }
        doomed.sort_unstable_by(|a, b| b.cmp(a));
        for i in doomed {
            let e = entries.remove(i);
            let _ = std::fs::remove_file(&e.path);
            crate::diag::log_line(
                "cache",
                &format!(
                    "orbit cache EVICT: {} (len={} prec={}, {:.1} MB) — over the {:.0} MB budget",
                    e.path.file_name().and_then(|f| f.to_str()).unwrap_or("?"),
                    e.header.orbit_len,
                    e.header.prec,
                    e.bytes as f64 / 1_048_576.0,
                    budget as f64 / 1_048_576.0
                ),
            );
        }
    }
}

/// Read one entry's header from its file prefix.
fn index_entry(path: &Path) -> Option<Entry> {
    use std::io::Read;
    let meta = std::fs::metadata(path).ok()?;
    let mut f = std::fs::File::open(path).ok()?;
    let mut prelude = [0u8; 14];
    f.read_exact(&mut prelude).ok()?;
    let span = orbit_blob::header_span(&prelude)?;
    let mut buf = vec![0u8; span];
    buf[..14].copy_from_slice(&prelude);
    f.read_exact(&mut buf[14..]).ok()?;
    let (header, _) = orbit_blob::read_header(&buf)?;
    // ⭐The body is never read here; its length is the one thing about it that can be checked.
    if header.file_len != meta.len() {
        return None;
    }
    Some(Entry {
        path: path.to_path_buf(),
        header,
        bytes: meta.len(),
        last_used: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
    })
}

/// Index every `*.orbit` in `dir`. Files with our magic that do not verify are deleted; they can
/// never be used, and leaving them would count against the budget forever.
fn scan(dir: &Path) -> Vec<Entry> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else { return out };
    for f in rd.flatten() {
        let p = f.path();
        if p.extension().is_none_or(|x| x != ENTRY_EXT) {
            continue;
        }
        match index_entry(&p) {
            Some(e) => out.push(e),
            None => {
                let ours = std::fs::File::open(&p)
                    .ok()
                    .and_then(|mut f| {
                        use std::io::Read;
                        let mut m = [0u8; 8];
                        f.read_exact(&mut m).ok().map(|()| &m == b"FDNORBIT")
                    })
                    .unwrap_or(false);
                if ours {
                    let _ = std::fs::remove_file(&p);
                    crate::diag::log_line(
                        "cache",
                        &format!("orbit cache entry {} unreadable by this build — removed", p.display()),
                    );
                }
            }
        }
    }
    out
}

/// Human-readable byte count for the controls.
pub(crate) fn fmt_bytes(b: u64) -> String {
    const MB: f64 = 1_048_576.0;
    if b >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", b as f64 / (MB * 1024.0))
    } else if b >= 1024 * 1024 {
        format!("{:.1} MB", b as f64 / MB)
    } else if b >= 1024 {
        format!("{:.0} KB", b as f64 / 1024.0)
    } else {
        format!("{b} B")
    }
}

#[cfg(test)]
mod tests;
