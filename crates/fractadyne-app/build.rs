//! Auto-incrementing build counter. Each time this crate is recompiled (any source
//! change re-runs the build script by default), a persistent counter is bumped and
//! exposed as the `FRACT_BUILD` env var, so the app reports a monotonically
//! increasing build number on top of the semantic version.
//!
//! The counter lives at the workspace root (`.build_seq`) — shared across the debug
//! and release profiles (whose `OUT_DIR`s differ), and outside this crate's package
//! directory so writing it never re-triggers a rebuild.
//!
//! It also stamps the commit the binary was built from as `FRACT_GIT`: the short sha, with
//! `-dirty` when a tracked source differs from it, `-archive` when built from a `git archive`
//! tarball that carries `BUILD-COMMIT.txt` (written by `scripts/publish-share.ps1`; the tree cannot
//! be checked, so cleanliness is unverified), or `unknown` when neither is available. Two binaries
//! of one version were indistinguishable before this, and a field run once measured the previous
//! build without anyone being able to tell (design/live-render-robustness.md §6.1).
//!
//! ⚠Emitting ANY `rerun-if-changed` switches Cargo off its default of re-running when any file in
//! the package changes, so the default set is listed explicitly — otherwise the build counter would
//! silently stop moving on ordinary edits. The set is the whole of `WATCHED`, which is also the
//! pathspec the dirty check reads: the flag and the thing that refreshes it are one list, so it
//! cannot go stale in either direction.
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

/// Everything the binary is compiled from, relative to the workspace root: every crate (a change
/// to a dependency crate changes this binary as much as a change here), the manifests and lockfile,
/// Cargo's config, and the two trees the app embeds from outside `crates/` — `tours/*.toml`
/// (the selftest) and `THIRD-PARTY-NOTICES.md` (Help ▸ About). A new `include_str!` of a file
/// outside this list belongs in it.
const WATCHED: &[&str] = &[
    "crates",
    "Cargo.toml",
    "Cargo.lock",
    ".cargo",
    "tours",
    "THIRD-PARTY-NOTICES.md",
];

/// Written into the source tarball by `publish-share.ps1` (`git archive --add-file`), never
/// committed: line 1 the full sha, line 2 the short one.
const ARCHIVE_STAMP: &str = "BUILD-COMMIT.txt";

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let path: PathBuf = if manifest.is_empty() {
        PathBuf::from(env::var("OUT_DIR").unwrap_or_default()).join("build_seq.txt")
    } else {
        // crates/fractadyne-app -> workspace root
        PathBuf::from(&manifest)
            .join("..")
            .join("..")
            .join(".build_seq")
    };
    let n: u64 = fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
        + 1;
    let _ = fs::write(&path, n.to_string());
    println!("cargo:rustc-env=FRACT_BUILD={n}");

    let root = PathBuf::from(&manifest).join("..").join("..");
    // Never emit a path that does not exist: Cargo treats a missing rerun-if-changed file as
    // always stale, which would recompile this (large) crate on every build.
    let rerun = |p: &Path| {
        if p.exists() {
            println!("cargo:rerun-if-changed={}", p.display());
        }
    };
    for w in WATCHED {
        rerun(&root.join(w));
    }
    rerun(&root.join(ARCHIVE_STAMP));
    println!("cargo:rerun-if-env-changed=FRACTADYNE_GIT");

    let git = match git_identity(&root) {
        Some((ident, watch)) => {
            for p in &watch {
                rerun(p);
            }
            ident
        }
        None => archive_identity(&root).unwrap_or_else(|| "unknown".to_string()),
    };
    println!("cargo:rustc-env=FRACT_GIT={git}");
}

/// `(identity, files to watch)` from the work tree, or `None` outside one (or without git).
///
/// One `rev-parse` gives the top level, the git dir, the common dir and the short sha. The files
/// watched are the ones a commit, checkout, reset or stash actually rewrites: `HEAD` (checkout; a
/// detached HEAD holds the sha itself), `refs/heads` as a DIRECTORY (on a branch, a commit rewrites
/// the branch's loose ref, not `HEAD` — and after `git pack-refs` the loose file may not exist yet,
/// so watching the file itself would miss its re-creation), and `packed-refs`. `.git/index` is
/// deliberately NOT watched: `git add` rewrites it without changing what this check reports, and
/// every rerun recompiles this crate.
///
/// ⚠The top level must BE this workspace. `git -C` walks upward, so a source tarball unpacked
/// inside some other repository (a home directory kept under git, say) would otherwise be stamped
/// with THAT repository's commit — a confident wrong answer, which is worse than `unknown`.
fn git_identity(root: &Path) -> Option<(String, Vec<PathBuf>)> {
    let out = git()
        .arg("-C")
        .arg(root)
        .args([
            "rev-parse",
            "--show-toplevel",
            "--git-dir",
            "--git-common-dir",
            "--short",
            "HEAD",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut lines = text.lines().map(str::trim);
    let top = fs::canonicalize(lines.next()?).ok()?;
    if top != fs::canonicalize(root).ok()? {
        return None;
    }
    let git_dir = abs(root, lines.next()?);
    let common = abs(root, lines.next()?);
    let sha = lines.next()?;
    if sha.is_empty() || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    // `--no-optional-locks`: a plain `git status` may refresh and REWRITE the index as a side
    // effect, which is exactly the kind of write a build script must not cause. Untracked files are
    // excluded (the `git describe --dirty` meaning): logs and scratch output are not sources.
    let status = git()
        .arg("-C")
        .arg(root)
        .args([
            "--no-optional-locks",
            "status",
            "--porcelain",
            "--untracked-files=no",
            "--",
        ])
        .args(WATCHED)
        .output();
    // If cleanliness cannot be established, say dirty: a build must never claim to be a clean
    // commit it cannot vouch for, and `publish-share.ps1` refuses a dirty build.
    let dirty = match status {
        Ok(s) if s.status.success() => !s.stdout.iter().all(u8::is_ascii_whitespace),
        _ => true,
    };
    let ident = if dirty {
        format!("{sha}-dirty")
    } else {
        sha.to_string()
    };
    let watch = vec![
        git_dir.join("HEAD"),
        common.join("refs").join("heads"),
        common.join("packed-refs"),
    ];
    Some((ident, watch))
}

/// `git`, or the executable `FRACTADYNE_GIT` names — the app's own `FRACTADYNE_<NAME>` override
/// convention. ⚠Needed by `scripts/build-accelerated.ps1`, which compiles inside an MSYS2 login
/// shell whose PATH does not include Git for Windows: there `git` was not found and the MPFR
/// package the maintainer runs day to day was stamped `git unknown` (build 3513).
fn git() -> Command {
    Command::new(env::var_os("FRACTADYNE_GIT").unwrap_or_else(|| "git".into()))
}

/// The tarball case: `BUILD-COMMIT.txt` at the root, line 2 the short sha.
fn archive_identity(root: &Path) -> Option<String> {
    let text = fs::read_to_string(root.join(ARCHIVE_STAMP)).ok()?;
    let short = text.lines().nth(1)?.trim();
    (short.len() >= 7 && short.chars().all(|c| c.is_ascii_hexdigit()))
        .then(|| format!("{short}-archive"))
}

fn abs(root: &Path, p: &str) -> PathBuf {
    let p = PathBuf::from(p);
    if p.is_absolute() {
        p
    } else {
        root.join(p)
    }
}
