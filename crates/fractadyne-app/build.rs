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
//! And it turns the English interface file, `i18n/en/fractadyne.ftl`, into one constant per message
//! (`$OUT_DIR/i18n_ids.rs`, included by `src/i18n/mod.rs`), so code that asks for a message the file
//! does not have fails to compile (design/multilingual.md §7).
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
/// Cargo's config, and the trees the app embeds from outside `crates/` — `tours/*.toml` (the
/// selftest), `THIRD-PARTY-NOTICES.md` (Help ▸ About) and `i18n/` (the English interface file). A
/// new `include_str!` of a file outside this list belongs in it.
const WATCHED: &[&str] = &[
    "crates",
    "Cargo.toml",
    "Cargo.lock",
    ".cargo",
    "tours",
    "THIRD-PARTY-NOTICES.md",
    "i18n",
];

/// The English interface file, relative to the workspace root.
const ENGLISH_FTL: &str = "i18n/en/fractadyne.ftl";

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

    let ftl = fs::read_to_string(root.join(ENGLISH_FTL)).unwrap_or_else(|e| panic!("{ENGLISH_FTL}: {e}"));
    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("i18n_ids.rs");
    let code = i18n_ids(&ftl).unwrap_or_else(|e| panic!("{ENGLISH_FTL}: {e}"));
    // Written only when it changes, so an unrelated edit does not touch it.
    if fs::read_to_string(&out).ok().as_deref() != Some(code.as_str()) {
        fs::write(&out, code).expect("write i18n_ids.rs");
    }

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

/// The Rust source of `i18n_ids.rs` from the English Fluent file: `COUNT`, the IDs in file order
/// (`NAMES`), the variables each message uses (`VARS`), and one `Msg` constant per message named
/// after its ID (`menu-file-open` → `menu_file_open`).
///
/// A Fluent message starts at the beginning of a line with its identifier and `=`; its other lines
/// are indented (continuations, variants, attributes), terms start with `-` and comments with `#`.
/// So an entry is a line whose first character is a lowercase letter, and every `$name` until the
/// next entry is one of its variables. A unit test holds this scan to `fluent-syntax`'s own parse of
/// the same file.
fn i18n_ids(src: &str) -> Result<String, String> {
    let mut msgs: Vec<(String, Vec<String>)> = Vec::new();
    for (n, line) in src.lines().enumerate() {
        let first = line.chars().next();
        if first.is_some_and(|c| c.is_ascii_lowercase()) {
            let Some(eq) = line.find('=') else {
                return Err(format!("line {}: an entry without '='", n + 1));
            };
            let id = line[..eq].trim_end();
            if !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
                return Err(format!("line {}: message ID {id:?} is not lowercase kebab-case", n + 1));
            }
            if msgs.iter().any(|(m, _)| m == id) {
                return Err(format!("line {}: message {id:?} is defined twice", n + 1));
            }
            msgs.push((id.to_string(), Vec::new()));
        } else if first.is_some_and(|c| c == '-' || c == '#') {
            // A term or a comment ends the message before it.
            if first == Some('-') {
                msgs.push((String::new(), Vec::new()));
            }
            continue;
        }
        if let Some((_, vars)) = msgs.last_mut() {
            let b = line.as_bytes();
            let mut i = 0;
            while i < b.len() {
                if b[i] == b'$' && b.get(i + 1).is_some_and(|c| c.is_ascii_alphabetic()) {
                    let start = i + 1;
                    let mut j = start;
                    while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'_' || b[j] == b'-') {
                        j += 1;
                    }
                    let v = &line[start..j];
                    if !vars.iter().any(|x| x == v) {
                        vars.push(v.to_string());
                    }
                    i = j;
                } else {
                    i += 1;
                }
            }
        }
    }
    msgs.retain(|(id, _)| !id.is_empty());
    let mut code = String::from("// Generated by build.rs from i18n/en/fractadyne.ftl. Do not edit.\n\n");
    code += &format!("/// Messages in the English file.\npub(crate) const COUNT: usize = {};\n\n", msgs.len());
    code += "/// Each message's ID, by `Msg` index.\npub(crate) static NAMES: [&str; COUNT] = [\n";
    for (id, _) in &msgs {
        code += &format!("    {id:?},\n");
    }
    code += "];\n\n/// The variables each English message uses, by `Msg` index.\npub(crate) static VARS: [&[&str]; COUNT] = [\n";
    for (_, vars) in &msgs {
        code += &format!("    &{vars:?},\n");
    }
    code += "];\n\n";
    for (i, (id, _)) in msgs.iter().enumerate() {
        code += &format!("#[allow(non_upper_case_globals, dead_code)]\npub(crate) const {}: super::Msg = super::Msg({i});\n", id.replace('-', "_"));
    }
    Ok(code)
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
    // The FULL sha, cut to 7 here. `--short` abbreviates to whatever length is unique in THIS
    // repository, so one commit was stamped "29988ec" on Windows and "29988ecf" in the Linux test
    // machine's clone of a bundle - and a render farm, which admits only the same commit, refused it.
    let full = lines.next()?;
    if full.len() < 7 || !full.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let sha = &full[..7];
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
