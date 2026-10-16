use super::*;

#[test]
fn env_var_names_are_derived_and_normalized() {
    assert_eq!(env_var_for("nvidia-smi"), "FRACTADYNE_NVIDIA_SMI");
    assert_eq!(env_var_for("ffmpeg"), "FRACTADYNE_FFMPEG");
    assert_eq!(env_var_for("python"), "FRACTADYNE_PYTHON");
}

struct Tmp(PathBuf);
impl Tmp {
    fn new(tag: &str) -> Self {
        let d = std::env::temp_dir()
            .join(format!("fractadyne_execresolve_{}_{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("temp dir");
        Self(d)
    }
}
impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn search_finds_an_exact_name_in_a_path_dir() {
    let tmp = Tmp::new("found");
    let f = tmp.0.join("tool");
    std::fs::write(&f, b"#!/bin/sh\n").unwrap();
    let path = std::env::join_paths([&tmp.0]).unwrap();
    assert_eq!(search_dirs("tool", &path, None).as_deref(), Some(f.as_path()));
    // A name that isn't there resolves to nothing (never a cwd guess).
    assert_eq!(search_dirs("absent", &path, None), None);
}

#[test]
fn empty_path_entries_meaning_cwd_are_skipped() {
    let tmp = Tmp::new("nocwd");
    let real = tmp.0.join("tool");
    std::fs::write(&real, b"x").unwrap();
    // Build "<empty>:<tmp>" so the first entry is the cwd sentinel; it must be skipped, and the
    // real dir still found — proving the search never treats an empty entry as the cwd.
    let sep = if cfg!(windows) { ";" } else { ":" };
    let raw = format!("{sep}{}", tmp.0.display());
    let path = std::ffi::OsString::from(raw);
    assert_eq!(search_dirs("tool", &path, None).as_deref(), Some(real.as_path()));
}

#[test]
fn first_matching_dir_wins() {
    let a = Tmp::new("a");
    let b = Tmp::new("b");
    std::fs::write(a.0.join("tool"), b"x").unwrap();
    std::fs::write(b.0.join("tool"), b"x").unwrap();
    let path = std::env::join_paths([&a.0, &b.0]).unwrap();
    assert_eq!(search_dirs("tool", &path, None).as_deref(), Some(a.0.join("tool").as_path()));
}
