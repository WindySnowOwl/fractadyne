//! Files that hold the user's work: the session, the bookmarks, the gradient and formula libraries.
//!
//! Three rules every one of them keeps, so a damaged or unfamiliar file is never the reason the
//! user loses what was in it:
//! - ⭐**Writes are atomic** ([`write_atomic`]): a temp file beside the target, flushed to disk, then
//!   renamed over it. A crash, a full disk or a power cut mid-write leaves the old file, never half
//!   of the new one. `std::fs::write` truncates first, so a failure part-way through it was a lost
//!   library (2026-11-16 review).
//! - ⭐**A file that cannot be read completely is copied aside before anything can replace it**
//!   ([`keep_copy`]). Reading a damaged file as "nothing", then saving, was how one bad value cost a
//!   user every setting, or one bad entry every bookmark.
//! - ⭐**What can be read is kept** ([`text_of`], [`salvage_entries`]): bytes that are not UTF-8 are
//!   read with replacement characters, and an entry that does not read is dropped alone, not with
//!   the file.

use serde::de::DeserializeOwned;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Write `bytes` to `path` so that the file is always either what it was or all of `bytes`: a temp
/// file beside it (named for this process, so two running copies of the app never write the same
/// temp file), flushed to disk, then renamed over it. Creates the parent directory.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let tmp = path.with_file_name(format!("{name}.tmp-{}", std::process::id()));
    let result = (|| {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        // ⚠Without this the rename can reach the disk before the data does, and a power cut then
        // leaves a file of the right name full of nothing.
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// Keep a copy of `path` beside it as `<stem>.<tag>.<ext>` (or `-2`, `-3`… when that name holds
/// something else), so a save cannot destroy what this build could not read. An identical copy
/// already kept is reused rather than made again, so a file that stays damaged across launches
/// (a harness never saves) does not fill the folder. Returns the copy's path.
pub fn keep_copy(path: &Path, tag: &str) -> std::io::Result<PathBuf> {
    let bytes = std::fs::read(path)?;
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let ext = path.extension().map(|s| format!(".{}", s.to_string_lossy())).unwrap_or_default();
    for k in 1..1000 {
        let name = if k == 1 { format!("{stem}.{tag}{ext}") } else { format!("{stem}.{tag}-{k}{ext}") };
        let p = path.with_file_name(name);
        match std::fs::read(&p) {
            Ok(b) if b == bytes => return Ok(p),
            Ok(_) => continue,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                write_atomic(&p, &bytes)?;
                return Ok(p);
            }
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::other("no free name for the copy"))
}

/// The text of a file's bytes: UTF-8, or — when it is not, as after an edit in a program that
/// saves in the system's code page — the same with each byte that does not read replaced by U+FFFD,
/// and `true` to say the file has to be kept as it was.
pub fn text_of(bytes: &[u8]) -> (String, bool) {
    match std::str::from_utf8(bytes) {
        Ok(t) => (t.to_owned(), false),
        Err(_) => (String::from_utf8_lossy(bytes).into_owned(), true),
    }
}

/// The entries of the array of tables `key` (`[[bookmark]]`…) in `text` that read as `T`, and how
/// many did not. For a file that does not deserialize whole: one entry with a missing field or a
/// value of the wrong type is dropped alone. `None` when `text` is not a TOML document at all.
pub fn salvage_entries<T: DeserializeOwned>(text: &str, key: &str) -> Option<(Vec<T>, usize)> {
    let table: toml::Table = text.parse().ok()?;
    Some(match table.get(key) {
        None => (Vec::new(), 0),
        Some(toml::Value::Array(items)) => {
            let mut kept = Vec::with_capacity(items.len());
            let mut dropped = 0;
            for item in items {
                match item.clone().try_into::<T>() {
                    Ok(t) => kept.push(t),
                    Err(_) => dropped += 1,
                }
            }
            (kept, dropped)
        }
        Some(_) => (Vec::new(), 1),
    })
}

/// What a list file holds on disk when a save is about to replace it ([`read_entries_on_disk`]).
#[derive(Debug, PartialEq)]
pub enum OnDisk<T> {
    /// No file: nothing to merge with.
    Missing,
    /// The entries that read (an entry that does not is left out).
    Entries(Vec<T>),
    /// A file that is not TOML at all: it says nothing about what another copy of the app did, so
    /// a merge must treat it as unchanged rather than as "everything was deleted".
    NotReadable,
}

/// The entries of the array of tables `key` in the list file at `path`, for the merge a save does
/// before it replaces the file ([`merge3`]). An error reading the file (it is locked, say) is
/// returned, and the save should not go ahead: it cannot know what it would destroy.
pub fn read_entries_on_disk<T: DeserializeOwned>(path: &Path, key: &str) -> std::io::Result<OnDisk<T>> {
    match std::fs::read(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(OnDisk::Missing),
        Err(e) => Err(e),
        Ok(bytes) => {
            let (text, _) = text_of(&bytes);
            Ok(match salvage_entries(&text, key) {
                Some((entries, _)) => OnDisk::Entries(entries),
                None => OnDisk::NotReadable,
            })
        }
    }
}

/// A three-way merge of a list file, for two running copies of the app saving the same file: `base`
/// is what this copy last read or wrote, `ours` what it holds now, `theirs` what is on disk now.
/// The result is `ours`, plus what the other copy added (in `theirs`, not in `base`), minus what it
/// removed (in `base`, not in `theirs`) — so neither copy's additions or deletions are lost to the
/// other's save. Entries are compared whole: an edit is a removal and an addition.
pub fn merge3<T: Clone + PartialEq>(base: &[T], ours: &[T], theirs: &[T]) -> Vec<T> {
    let removed_there = |e: &T| base.contains(e) && !theirs.contains(e);
    let mut out: Vec<T> = ours.iter().filter(|e| !removed_there(e)).cloned().collect();
    for e in theirs {
        if !base.contains(e) && !out.contains(e) {
            out.push(e.clone());
        }
    }
    out
}

#[cfg(test)]
#[path = "../tests/unit/durable/tests.rs"]
mod tests;
