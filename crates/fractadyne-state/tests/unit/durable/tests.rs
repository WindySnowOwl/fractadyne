use super::*;

fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("fractadyne_durable_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The write lands whole, replaces what was there, creates the folder, and leaves no temp file.
#[test]
fn an_atomic_write_replaces_the_file_and_leaves_nothing_beside_it() {
    let d = temp_dir("atomic");
    let path = d.join("sub").join("bookmarks.toml");
    write_atomic(&path, b"first").unwrap();
    write_atomic(&path, b"second, longer").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"second, longer");
    let names: Vec<String> =
        std::fs::read_dir(d.join("sub")).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    assert_eq!(names, ["bookmarks.toml"], "no temp file left behind");
    let _ = std::fs::remove_dir_all(&d);
}

/// A write that fails (the target is a folder, so the rename cannot replace it) leaves the target
/// as it was and no temp file.
#[test]
fn a_failed_atomic_write_leaves_the_old_file_and_no_temp() {
    let d = temp_dir("atomic_fail");
    let target = d.join("gradients.toml");
    std::fs::create_dir_all(&target).unwrap();
    assert!(write_atomic(&target, b"x").is_err());
    assert!(target.is_dir());
    let n = std::fs::read_dir(&d).unwrap().count();
    assert_eq!(n, 1, "only the target remains");
    let _ = std::fs::remove_dir_all(&d);
}

/// A kept copy is the file byte for byte; the same file again reuses it; a different one takes the
/// next free name, so no kept copy is ever overwritten.
#[test]
fn a_kept_copy_is_exact_reused_and_never_overwritten() {
    let d = temp_dir("keep");
    let path = d.join("bookmarks.toml");
    std::fs::write(&path, b"one \xFF").unwrap();
    let a = keep_copy(&path, "unreadable").unwrap();
    assert_eq!(a.file_name().unwrap(), "bookmarks.unreadable.toml");
    assert_eq!(std::fs::read(&a).unwrap(), b"one \xFF");
    assert_eq!(keep_copy(&path, "unreadable").unwrap(), a, "an identical copy is reused");
    std::fs::write(&path, b"two").unwrap();
    let b = keep_copy(&path, "unreadable").unwrap();
    assert_eq!(b.file_name().unwrap(), "bookmarks.unreadable-2.toml");
    assert_eq!(std::fs::read(&a).unwrap(), b"one \xFF", "the first copy is untouched");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn text_that_is_not_utf8_is_read_with_replacements_and_flagged() {
    assert_eq!(text_of(b"caf\xC3\xA9"), ("café".to_string(), false));
    assert_eq!(text_of(b"caf\xE9"), ("caf\u{FFFD}".to_string(), true));
}

#[derive(serde::Deserialize, Debug, PartialEq)]
struct Entry {
    name: String,
    n: u8,
}

/// One entry with a missing field or a value out of its type's range is dropped alone.
#[test]
fn entries_that_do_not_read_are_dropped_alone() {
    let text = "[[e]]\nname = \"a\"\nn = 1\n\n[[e]]\nname = \"b\"\n\n[[e]]\nname = \"c\"\nn = 300\n\n[[e]]\nname = \"d\"\nn = 4\n";
    let (kept, dropped) = salvage_entries::<Entry>(text, "e").unwrap();
    assert_eq!(kept, [Entry { name: "a".into(), n: 1 }, Entry { name: "d".into(), n: 4 }]);
    assert_eq!(dropped, 2);
    assert!(salvage_entries::<Entry>("not [[ toml", "e").is_none());
    assert_eq!(salvage_entries::<Entry>("x = 1\n", "e").unwrap(), (Vec::new(), 0));
}

/// What a save finds on disk: nothing, the entries that read, or — for a file that is not TOML —
/// "no information", which a merge must not read as "everything was deleted".
#[test]
fn what_is_on_disk_is_read_for_the_merge() {
    let d = temp_dir("on_disk");
    let path = d.join("list.toml");
    assert_eq!(read_entries_on_disk::<Entry>(&path, "e").unwrap(), OnDisk::Missing);
    std::fs::write(&path, "[[e]]\nname = \"a\"\nn = 1\n\n[[e]]\nname = \"b\"\n").unwrap();
    assert_eq!(read_entries_on_disk::<Entry>(&path, "e").unwrap(), OnDisk::Entries(vec![Entry { name: "a".into(), n: 1 }]));
    std::fs::write(&path, b"\0\0 garbage [[[").unwrap();
    assert_eq!(read_entries_on_disk::<Entry>(&path, "e").unwrap(), OnDisk::NotReadable);
    let _ = std::fs::remove_dir_all(&d);
}

/// Two running copies: each one's additions and deletions survive the other's save.
#[test]
fn a_three_way_merge_keeps_both_copies_changes() {
    let base = ["a", "b", "c"];
    // This copy added "d" and deleted "b"; the other added "e" and deleted "c".
    let ours = ["a", "c", "d"];
    let theirs = ["a", "b", "e"];
    assert_eq!(merge3(&base, &ours, &theirs), ["a", "d", "e"]);
    // Nothing changed on disk: ours as it is.
    assert_eq!(merge3(&base, &ours, &base), ours);
    // Both added the same entry: once.
    assert_eq!(merge3(&["a"], &["a", "z"], &["a", "z"]), ["a", "z"]);
}
