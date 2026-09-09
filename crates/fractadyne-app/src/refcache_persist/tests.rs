//! The store's own rules — what it writes, what it finds, what it evicts — against a scratch
//! directory. ⚠The store is process-global, so these serialize on one lock and each starts from an
//! empty directory of its own. (Whether a stored orbit RENDERS the same as a fresh one is the
//! `orbit-cache` selftest's job; it needs a GPU.)

use super::*;
use crate::render::orbit_blob::tests::{blob, blob_at, key, point, sample_orbit};
use fractadyne_core::parse_bf_prec;

static SERIAL: Mutex<()> = Mutex::new(());

struct Scratch {
    dir: PathBuf,
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl Scratch {
    fn new(name: &str) -> Self {
        let guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!("fractadyne-orbit-store-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        set_dir_override(Some(dir.clone()));
        set_enabled(true);
        set_budget_bytes(1 << 30);
        Self { dir, _guard: guard }
    }
    fn files(&self) -> usize {
        std::fs::read_dir(&self.dir)
            .unwrap()
            .flatten()
            .filter(|f| f.path().extension().is_some_and(|x| x == ENTRY_EXT))
            .count()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        set_enabled(false);
        set_dir_override(None);
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn accept_all(_: &[BigFloat; 2], _: usize) -> Option<f64> {
    Some(0.0)
}

/// A second identity: the same key at a different point.
fn other_point(prec: usize) -> [BigFloat; 2] {
    [parse_bf_prec("-1.25", prec).unwrap(), parse_bf_prec("0.0", prec).unwrap()]
}

#[test]
fn offer_then_find_then_load_round_trips() {
    let s = Scratch::new("round-trip");
    let written = offer(blob(&sample_orbit(64), 512, 1000, key())).unwrap().expect("written");
    assert!(written.starts_with(&s.dir));
    assert_eq!(s.files(), 1);
    let hit = find(&Query { key: key(), fits: &accept_all }).expect("found");
    assert_eq!((hit.orbit_len, hit.prec, hit.candidates), (64, 512, 1));
    let d = load(&hit.path).expect("verifies");
    assert_eq!(d.reuse.prefix.len(), 64);
    assert_eq!(usage().entries, 1);
    // A different identity is invisible to this key.
    assert!(find(&Query { key: OrbitKey { formula_id: 1, ..key() }, fits: &accept_all }).is_none());
    // And an entry the caller's test refuses is not a hit, however well it matches.
    assert!(find(&Query { key: key(), fits: &|_, _| None }).is_none());
}

#[test]
fn a_longer_orbit_replaces_a_shorter_one_and_a_shorter_never_does() {
    let s = Scratch::new("replace");
    assert!(offer(blob(&sample_orbit(64), 512, 1000, key())).unwrap().is_some());
    assert!(offer(blob(&sample_orbit(128), 512, 1000, key())).unwrap().is_some(), "longer replaces");
    assert_eq!(s.files(), 1, "the same identity is ONE file");
    assert!(offer(blob(&sample_orbit(32), 512, 1000, key())).unwrap().is_none(), "shorter declined");
    assert!(offer(blob(&sample_orbit(128), 512, 1000, key())).unwrap().is_none(), "equal declined");
    let hit = find(&Query { key: key(), fits: &accept_all }).unwrap();
    assert_eq!(hit.orbit_len, 128);
}

#[test]
fn wanted_holds_only_a_new_identity_to_the_build_time_threshold() {
    let _s = Scratch::new("wanted");
    let id = crate::render::orbit_blob::key_id(&key(), 512, &point(512));
    let min = crate::tunables::ORBIT_CACHE_MIN_BUILD_MS;
    assert!(!wanted(id, 64, min * 0.5), "a quick new orbit is not worth a file");
    assert!(wanted(id, 64, min), "a slow new orbit is");
    offer(blob(&sample_orbit(64), 512, 1000, key())).unwrap().unwrap();
    assert!(wanted(id, 96, 0.0), "a materially longer build of an existing entry replaces it, however quick");
    assert!(!wanted(id, 65, 0.0), "a one-sample creep is not worth rewriting the file");
    assert!(!wanted(id, 64, min * 10.0), "the same length gains nothing, however slow");
    set_enabled(false);
    assert!(!wanted(id, 96, min), "nothing is wanted while the cache is off");
}

/// ⭐The live path grows a capped reference by ONE sample per rebuild (measured at e60205:
/// 256,001 → 256,002, 0.6 s after the entry it would have replaced). That creep must not rewrite
/// 4 MB each time; a real extension must.
#[test]
fn a_replacement_must_be_materially_longer() {
    assert!(!worth_replacing(256_001, 256_002), "the live cap's one-sample creep");
    assert!(!worth_replacing(256_001, 258_000), "under the 1/64 margin");
    assert!(worth_replacing(256_001, 2_008_193), "the live cap extended to a 2M ask");
    assert!(!worth_replacing(64, 65));
    assert!(worth_replacing(64, 66));
    assert!(!worth_replacing(64, 64));
    assert!(worth_replacing(0, 1));
    // And the store applies it: a creep is declined on disk too.
    let s = Scratch::new("creep");
    offer(blob(&sample_orbit(64), 512, 1000, key())).unwrap().unwrap();
    assert!(offer(blob(&sample_orbit(65), 512, 1000, key())).unwrap().is_none());
    assert_eq!(find(&Query { key: key(), fits: &accept_all }).unwrap().orbit_len, 64);
    assert_eq!(s.files(), 1);
}

/// ⭐⭐**Cost-aware eviction, not LRU.** The most recent entry is the CHEAPEST here, and it is the
/// one that goes — under LRU the expensive orbit written first would have been evicted instead.
#[test]
fn eviction_drops_the_cheapest_orbit_not_the_oldest() {
    let s = Scratch::new("evict");
    // Expensive: high precision (cost = len × prec²), written FIRST (the LRU victim).
    let dear = blob(&sample_orbit(64), 4096, 1000, key());
    // Cheap: low precision, same length, at another point, written LAST.
    let cheap = blob_at(&sample_orbit(64), 256, 1000, key(), other_point(256));
    let (dear_len, cheap_len) = (dear.len() as u64, cheap.len() as u64);
    offer(dear).unwrap().expect("written");
    // Budget: both do not fit.
    set_budget_bytes(dear_len + cheap_len - 1);
    // The cheap one would not survive its own eviction pass, so it is refused up front and the
    // expensive one is untouched.
    assert!(offer(cheap.clone()).unwrap().is_none(), "a cheaper orbit must not displace a dearer one");
    assert_eq!(s.files(), 1);
    assert_eq!(find(&Query { key: key(), fits: &accept_all }).unwrap().prec, 4096);
    // With room for both, both are kept; shrinking the budget afterwards evicts the cheap one.
    set_budget_bytes(dear_len + cheap_len);
    assert!(offer(cheap).unwrap().is_some());
    assert_eq!(s.files(), 2);
    set_budget_bytes(dear_len + cheap_len - 1);
    assert_eq!(s.files(), 1, "the budget change evicts at once");
    assert_eq!(usage().entries, 1);
    assert_eq!(find(&Query { key: key(), fits: &accept_all }).unwrap().prec, 4096, "the dear one survives");
}

/// The reverse: a DEARER orbit arriving at a full cache evicts the cheap ones to make room.
#[test]
fn a_dearer_orbit_evicts_cheaper_ones_to_fit() {
    let s = Scratch::new("displace");
    let cheap = blob_at(&sample_orbit(64), 256, 1000, key(), other_point(256));
    let dear = blob(&sample_orbit(64), 4096, 1000, key());
    let (cheap_len, dear_len) = (cheap.len() as u64, dear.len() as u64);
    offer(cheap).unwrap().expect("written");
    set_budget_bytes(cheap_len.max(dear_len) + 16);
    assert!(offer(dear).unwrap().is_some(), "the dear orbit is written");
    assert_eq!(s.files(), 1, "…and the cheap one made way for it");
    assert_eq!(find(&Query { key: key(), fits: &accept_all }).unwrap().prec, 4096);
}

#[test]
fn find_prefers_the_longest_orbit_then_the_closest_point() {
    let _s = Scratch::new("prefer");
    offer(blob(&sample_orbit(64), 512, 1000, key())).unwrap().unwrap();
    offer(blob_at(&sample_orbit(256), 512, 1000, key(), other_point(512))).unwrap().unwrap();
    // Both admissible: the longer wins even though it is (by this test's reckoning) farther.
    let far_is_long = |p: &[BigFloat; 2], _: usize| -> Option<f64> {
        Some(if p[0].mantissa_digits() == other_point(512)[0].mantissa_digits() { 0.5 } else { 0.1 })
    };
    let hit = find(&Query { key: key(), fits: &far_is_long }).unwrap();
    assert_eq!((hit.orbit_len, hit.candidates), (256, 2));
    assert!((hit.drift - 0.5).abs() < 1e-12);
    // Equal lengths: the closer one.
    offer(blob(&sample_orbit(256), 512, 1000, key())).unwrap().unwrap();
    let hit = find(&Query { key: key(), fits: &far_is_long }).unwrap();
    assert!((hit.drift - 0.1).abs() < 1e-12, "ties go to the closest point");
}

/// ⛔An entry that stops verifying is refused, deleted, and forgotten — never repaired.
#[test]
fn load_removes_an_entry_that_no_longer_verifies() {
    let s = Scratch::new("corrupt");
    let path = offer(blob(&sample_orbit(64), 512, 1000, key())).unwrap().unwrap();
    let mut b = std::fs::read(&path).unwrap();
    let last = b.len() - 20; // inside the orbit samples
    b[last] ^= 0x01;
    std::fs::write(&path, &b).unwrap();
    let hit = find(&Query { key: key(), fits: &accept_all }).expect("still indexed");
    assert!(load(&hit.path).is_none(), "a corrupt entry must not decode");
    assert!(!path.exists(), "…and must be deleted");
    assert_eq!(s.files(), 0);
    assert!(find(&Query { key: key(), fits: &accept_all }).is_none(), "…and forgotten");
}

#[test]
fn a_scan_indexes_ours_deletes_our_unreadable_and_leaves_foreign_files_alone() {
    let s = Scratch::new("scan");
    let good = blob(&sample_orbit(64), 512, 1000, key());
    std::fs::write(s.dir.join("aaaa.orbit"), &good).unwrap();
    // Ours by magic, header intact, but truncated in the BODY the scan never reads: the header's
    // recorded file length is what catches it, and it can never be used.
    std::fs::write(s.dir.join("bbbb.orbit"), &good[..good.len() / 2]).unwrap();
    // Not ours at all.
    std::fs::write(s.dir.join("notes.orbit"), b"someone else's file").unwrap();
    std::fs::write(s.dir.join("readme.txt"), b"hello").unwrap();
    set_dir_override(Some(s.dir.clone())); // drop the index so the next call scans
    assert_eq!(usage().entries, 1);
    assert!(s.dir.join("aaaa.orbit").exists());
    assert!(!s.dir.join("bbbb.orbit").exists(), "our unreadable file is removed");
    assert!(s.dir.join("notes.orbit").exists(), "a foreign file is left alone");
    assert!(s.dir.join("readme.txt").exists());
}

#[test]
fn clear_removes_every_entry_and_stray_temp_files() {
    let s = Scratch::new("clear");
    offer(blob(&sample_orbit(64), 512, 1000, key())).unwrap().unwrap();
    offer(blob_at(&sample_orbit(64), 512, 1000, key(), other_point(512))).unwrap().unwrap();
    std::fs::write(s.dir.join("dead.orbit.tmp"), b"interrupted").unwrap();
    assert_eq!(clear().unwrap(), 2);
    assert_eq!(s.files(), 0);
    assert!(!s.dir.join("dead.orbit.tmp").exists());
    assert_eq!(usage().entries, 0);
    assert!(find(&Query { key: key(), fits: &accept_all }).is_none());
}

#[test]
fn nothing_is_written_or_found_while_the_cache_is_off() {
    let s = Scratch::new("off");
    set_enabled(false);
    assert!(offer(blob(&sample_orbit(64), 512, 1000, key())).unwrap().is_none());
    assert_eq!(s.files(), 0);
    set_enabled(true);
    offer(blob(&sample_orbit(64), 512, 1000, key())).unwrap().unwrap();
    set_enabled(false);
    assert!(find(&Query { key: key(), fits: &accept_all }).is_none(), "off means off, even with entries on disk");
}

#[test]
fn byte_counts_read_like_a_browser_shows_them() {
    assert_eq!(fmt_bytes(512), "512 B");
    assert_eq!(fmt_bytes(2048), "2 KB");
    assert_eq!(fmt_bytes(4_096_000), "3.9 MB");
    assert_eq!(fmt_bytes(1 << 30), "1.00 GB");
}
