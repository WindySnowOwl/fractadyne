//! Auto-incrementing build counter. Each time this crate is recompiled (any source
//! change re-runs the build script by default), a persistent counter is bumped and
//! exposed as the `FRACT_BUILD` env var, so the app reports a monotonically
//! increasing build number on top of the semantic version.
//!
//! The counter lives at the workspace root (`.build_seq`) — shared across the debug
//! and release profiles (whose `OUT_DIR`s differ), and outside this crate's package
//! directory so writing it never re-triggers a rebuild.
use std::{env, fs, path::PathBuf};

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let path: PathBuf = if manifest.is_empty() {
        PathBuf::from(env::var("OUT_DIR").unwrap_or_default()).join("build_seq.txt")
    } else {
        // crates/fractadyne-app -> workspace root
        PathBuf::from(&manifest).join("..").join("..").join(".build_seq")
    };
    let n: u64 = fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
        + 1;
    let _ = fs::write(&path, n.to_string());
    println!("cargo:rustc-env=FRACT_BUILD={n}");
}
