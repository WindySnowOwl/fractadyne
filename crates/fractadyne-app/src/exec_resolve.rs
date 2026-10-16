//! F-06: resolve external executables to a TRUSTED ABSOLUTE path rather than handing a bare
//! program name to `Command::new`.
//!
//! ⭐**Why.** `Command::new("nvidia-smi")` (or `ffmpeg`, `python`) lets the OS resolve the name —
//! and on Windows that resolution includes the **current directory** and every `PATH` entry. A
//! malicious `nvidia-smi.exe` dropped in the directory the app is launched from (a downloads
//! folder, a shared drive), or a poisoned `PATH` dir, would then run with the user's privileges.
//! `nvidia-smi` is the one that matters most: it runs **automatically at startup** for GPU
//! diagnostics, before the user does anything. `ffmpeg` (tour → mp4) and `python` (the `--torture`
//! corpus gate) are user-/gate-initiated.
//!
//! The fix is uniform: resolve to an absolute path ourselves, searching only (1) an explicit
//! `FRACTADYNE_<NAME>` override, (2) known trusted system locations, (3) `PATH` **excluding the
//! current directory**, then invoke that absolute path. The cwd exclusion is the point — everything
//! here iterates `PATH` dirs and never `.`.
//!
//! Honest scope (matches the desktop threat model): this closes the cwd-injection vector outright
//! and prefers trusted system dirs; a `PATH` fallback still trusts `PATH`, which already requires
//! an attacker to have local write access to a search directory.

use std::ffi::OsStr;
use std::path::PathBuf;

/// The env var that explicitly configures `name`'s executable path, e.g. `nvidia-smi` →
/// `FRACTADYNE_NVIDIA_SMI`, `ffmpeg` → `FRACTADYNE_FFMPEG`. Non-alphanumerics become `_`.
fn env_var_for(name: &str) -> String {
    let mut s = String::from("FRACTADYNE_");
    for ch in name.chars() {
        s.push(if ch.is_ascii_alphanumeric() { ch.to_ascii_uppercase() } else { '_' });
    }
    s
}

/// An explicitly-configured, existing path from `FRACTADYNE_<NAME>`, if set and pointing at a file.
fn from_env(name: &str) -> Option<PathBuf> {
    let p = PathBuf::from(std::env::var_os(env_var_for(name))?);
    p.is_file().then_some(p)
}

/// Pure PATH search (no env access): the first existing absolute path for `name` across
/// `path_var`'s entries, honoring `pathext` (Windows) — and **never** the current directory: empty
/// entries (which the shell treats as cwd) are skipped and `.` is never added.
fn search_dirs(name: &str, path_var: &OsStr, pathext: Option<&str>) -> Option<PathBuf> {
    for dir in std::env::split_paths(path_var) {
        if dir.as_os_str().is_empty() {
            continue; // an empty PATH entry means "current directory" — deliberately not searched
        }
        // Exact name first (covers a name that already carries an extension, and every Unix case).
        let direct = dir.join(name);
        if direct.is_file() {
            return Some(direct);
        }
        if let Some(exts) = pathext {
            for ext in exts.split(';').filter(|e| !e.is_empty()) {
                let cand = dir.join(format!("{name}{ext}"));
                if cand.is_file() {
                    return Some(cand);
                }
            }
        }
    }
    None
}

/// [`search_dirs`] over the process `PATH` (+ `PATHEXT` on Windows).
fn search_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let pathext = if cfg!(windows) {
        Some(std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.COM;.BAT;.CMD".to_string()))
    } else {
        None
    };
    search_dirs(name, &path, pathext.as_deref())
}

/// The platform's trusted absolute directories for `nvidia-smi`. The NVIDIA driver installs a copy
/// into `System32` (Windows) / `/usr/bin` (Linux); older Windows drivers used the `NVSMI` dir.
fn nvidia_trusted_dirs() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        let sysroot = std::env::var_os("SystemRoot")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
        vec![
            sysroot.join("System32"),
            PathBuf::from(r"C:\Program Files\NVIDIA Corporation\NVSMI"),
        ]
    }
    #[cfg(not(windows))]
    {
        vec![
            PathBuf::from("/usr/bin"),
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/bin"),
        ]
    }
}

/// Resolve `nvidia-smi` to an absolute path: explicit override → trusted system dir → `PATH`
/// (never cwd). `None` means "not found in a trusted location" — callers treat that as "GPU stats
/// unavailable", never as a reason to fall back to a bare-name spawn.
pub(crate) fn nvidia_smi() -> Option<PathBuf> {
    if let Some(p) = from_env("nvidia-smi") {
        return Some(p);
    }
    let leaf = if cfg!(windows) { "nvidia-smi.exe" } else { "nvidia-smi" };
    for dir in nvidia_trusted_dirs() {
        let cand = dir.join(leaf);
        if cand.is_file() {
            return Some(cand);
        }
    }
    search_path("nvidia-smi")
}

/// Resolve a general external tool (`ffmpeg`, `python`) to an absolute path: explicit override →
/// `PATH` (never cwd). There is no canonical install location for these, so there are no trusted
/// system dirs — but resolving the absolute path ourselves still removes the cwd-injection vector,
/// and the caller can surface the resolved path so a surprising location is visible.
pub(crate) fn external(name: &str) -> Option<PathBuf> {
    from_env(name).or_else(|| search_path(name))
}

#[cfg(test)]
#[path = "../tests/unit/exec_resolve/tests.rs"]
mod tests;
