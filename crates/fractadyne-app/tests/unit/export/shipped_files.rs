//! Every location file in the repository must still load cleanly under the current reader.
//!
//! ⭐⭐**Why this is a test and not a one-off review.** The `.fdn` format grew markers, comments,
//! a checksum, positional diagnostics and a thumbnail in one release. Every file written before
//! that — 43 in this repo, one of which SHIPS in the release archive, plus the view metadata baked
//! into the golden PNGs that also ship — was written by an older writer. "I checked them once" is
//! worth exactly as much as the next change to the reader; this fails the moment one stops loading.
//!
//! ⚠**Backward compatibility is the claim under test, in both directions.** These files have no
//! markers and no checksum, and that must stay SILENT — absence is the norm for anything written
//! before the field existed, and a reader that grumbled about it would make its own warnings
//! worthless.

use crate::export::{inspect_view_text, ChecksumState};
use std::path::{Path, PathBuf};

/// The repository root, from the crate this test is compiled in.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/fractadyne-app -> repo root")
        .to_path_buf()
}

fn find(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        // `target` is build output and `local`/`archive` are gitignored scratch — neither ships,
        // and both can contain deliberately broken files from debugging sessions.
        if p.is_dir() && !matches!(name, "target" | ".git" | "local" | "archive") {
            find(&p, ext, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some(ext) {
            out.push(p);
        }
    }
}

/// ⛔**The non-vacuity guard.** A test that walks a directory and finds nothing passes loudly. The
/// count is a floor, not an exact number, so adding a corpus location does not fail it.
const MIN_FDN: usize = 40;

#[test]
fn every_view_file_in_the_repo_loads_clean() {
    let root = repo_root();
    let mut files = Vec::new();
    find(&root, "fdn", &mut files);
    files.sort();
    assert!(
        files.len() >= MIN_FDN,
        "found only {} .fdn files under {} — the walk is not finding the corpus",
        files.len(),
        root.display()
    );

    let mut bad: Vec<String> = Vec::new();
    for p in &files {
        let rel = p.strip_prefix(&root).unwrap_or(p).display().to_string();
        let Ok(text) = std::fs::read_to_string(p) else {
            bad.push(format!("{rel}: not readable as UTF-8"));
            continue;
        };
        let (report, fields) = inspect_view_text(&text);
        if let Some(note) = report.note() {
            bad.push(format!("{rel}: {note}"));
        }
        // A location that parses but carries no coordinates is not a location.
        for key in ["center_re", "center_im"] {
            if !fields.iter().any(|(k, _, _, _)| k == key) {
                bad.push(format!("{rel}: has no {key}"));
            }
        }
    }
    assert!(bad.is_empty(), "{} of {} view files complain:\n  {}", bad.len(), files.len(), bad.join("\n  "));
}

/// ⚠**Old files carry no checksum, and that must be SILENT.** `Absent` is a distinct state from
/// `Mismatch` precisely so that every file written before the field existed keeps loading without
/// a warning; if these ever reported `Mismatch`, the digest would be wrong rather than the files.
#[test]
fn pre_checksum_files_report_absent_not_mismatch() {
    let root = repo_root();
    let mut files = Vec::new();
    find(&root, "fdn", &mut files);
    let mut mismatched = Vec::new();
    let mut absent = 0usize;
    for p in &files {
        let Ok(text) = std::fs::read_to_string(p) else { continue };
        match crate::export::view_checksum_state(&text) {
            ChecksumState::Absent => absent += 1,
            ChecksumState::Match => {}
            ChecksumState::Mismatch { found, computed } => mismatched.push(format!(
                "{}: found {found}, computed {computed}",
                p.strip_prefix(&root).unwrap_or(p).display()
            )),
        }
    }
    assert!(mismatched.is_empty(), "checksum mismatches:\n  {}", mismatched.join("\n  "));
    assert!(absent > 0, "the guard: none of the repo's files predate the checksum, so this proves nothing");
}

/// ⭐**The one `.fdn` that actually ships**, named explicitly rather than caught by the sweep — a
/// file in the release archive is the one a new user meets first, and it is worth a test that says
/// so by name rather than one that would still pass if it were deleted.
#[test]
fn the_shipped_sample_location_loads_clean() {
    let p = repo_root().join("scripts").join("deep-sample.fdn");
    let text = std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("scripts/deep-sample.fdn ships in the release archive: {e}"));
    let (report, fields) = inspect_view_text(&text);
    assert_eq!(report.note(), None, "the shipped sample complains on load");
    assert!(fields.iter().any(|(k, _, _, _)| k == "center_re"));
    assert!(!report.truncated);
    assert!(report.missing.is_empty());
}

/// ⚠⚠**And it must survive the journey it exists for.** A sample location is something people copy
/// out of a file and paste somewhere; if that breaks it, the sample is teaching the wrong lesson.
#[test]
fn the_shipped_sample_survives_a_hostile_paste() {
    let p = repo_root().join("scripts").join("deep-sample.fdn");
    let text = std::fs::read_to_string(&p).expect("read");
    let cases: [(&str, fn(&str) -> String); 4] = [
        ("DOS endings", |s| s.replace('\n', "\r\n")),
        ("classic-Mac endings", |s| s.replace('\n', "\r")),
        ("a word processor's minus signs", |s| s.replace("=-", "=\u{2212}")),
        ("a stray byte-order mark", |s| format!("\u{FEFF}{s}")),
    ];
    for (name, mangle) in cases {
        let (report, fields) = inspect_view_text(&mangle(&text));
        assert!(report.problems.is_empty(), "{name}: {:?}", report.problems);
        assert!(report.missing.is_empty(), "{name}: missing {:?}", report.missing);
        let cx = fields
            .iter()
            .find(|(k, _, _, _)| k == "center_re")
            .map(|(_, v, _, _)| v.clone())
            .unwrap_or_default();
        assert!(
            fractadyne_core::parse_bf(&cx).is_some(),
            "{name}: center_re {cx:?} stopped being a number"
        );
    }
}

/// The `.kfr` locations in the corpus go through a parser that also gained the hygiene layer.
#[test]
fn every_kfr_in_the_repo_still_parses() {
    let root = repo_root();
    let mut files = Vec::new();
    find(&root, "kfr", &mut files);
    assert!(files.len() >= 20, "found only {} .kfr files — the walk is wrong", files.len());
    let mut bad = Vec::new();
    for p in &files {
        let Ok(text) = std::fs::read_to_string(p) else {
            bad.push(format!("{}: not UTF-8", p.display()));
            continue;
        };
        if fractadyne_core::parse_kfr(&text).is_none() {
            bad.push(p.strip_prefix(&root).unwrap_or(p).display().to_string());
        }
    }
    assert!(bad.is_empty(), "{} .kfr files stopped parsing:\n  {}", bad.len(), bad.join("\n  "));
}

/// The golden PNGs ship too, and their `tEXt` chunk uses the same `Fractadyne` keyword as an
/// exported view — but carries something else entirely: the **command line that reproduces the
/// image**. That is the right payload for a golden, and this test pins it, because the shared
/// keyword makes the two easy to confuse.
///
/// ⛔⭐⭐**`read_png_metadata` returning `Some` does NOT mean "this is a view."** The gallery
/// already knew that and filters on `app=Fractadyne`; `open_view` did not, and would have told
/// someone opening a golden that their file was "missing center_re, center_im" rather than that
/// it holds no view at all. Found by writing this test against the wrong assumption and reading
/// what the files actually say.
#[test]
fn shipped_golden_pngs_carry_a_repro_command_not_a_view() {
    let dir = repo_root().join("validation").join("golden");
    let mut pngs = Vec::new();
    find(&dir, "png", &mut pngs);
    pngs.sort();
    assert!(!pngs.is_empty(), "no golden PNGs found under {}", dir.display());

    let (mut commands, mut views, mut bad) = (0usize, 0usize, Vec::new());
    for p in &pngs {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        match fractadyne_export::read_png_metadata(p) {
            Ok(Some(m)) if crate::export::looks_like_a_view(&m) => {
                // If a golden ever DOES carry a view, it has to be a loadable one.
                views += 1;
                let (report, fields) = inspect_view_text(&m);
                if let Some(note) = report.note() {
                    bad.push(format!("{name}: {note}"));
                }
                if !fields.iter().any(|(k, _, _, _)| k == "center_re") {
                    bad.push(format!("{name}: no centre after alias folding"));
                }
            }
            Ok(Some(m)) => {
                if m.contains("--render") {
                    commands += 1;
                } else {
                    bad.push(format!("{name}: metadata is neither a view nor a repro command"));
                }
            }
            Ok(None) => {}
            Err(e) => bad.push(format!("{name}: unreadable — {e}")),
        }
    }
    assert!(bad.is_empty(), "{} goldens complain:
  {}", bad.len(), bad.join("
  "));
    assert!(
        commands + views > 0,
        "the guard: none of the {} goldens carried any metadata, so this proved nothing",
        pngs.len()
    );
}
/// ⛔**`scripts/deep-sample.fdn` is deliberately LEFT in its pre-v0.2.20 form.** It is the only
/// genuine legacy-format specimen we have, and it is what proves `LEGACY_VIEW_KEYS` works against
/// a real file rather than only against a string built in a test. Modernizing it would tidy away
/// the regression test for the compatibility path.
///
/// ⚠If it is ever regenerated, replace this with a synthetic legacy fixture — do not simply delete
/// the coverage.
#[test]
fn the_shipped_sample_is_the_legacy_format_specimen() {
    let text = std::fs::read_to_string(repo_root().join("scripts").join("deep-sample.fdn"))
        .expect("read");
    assert!(
        text.contains("center_x=") && text.contains("center_y="),
        "the sample no longer uses the pre-v0.2.20 spelling; the legacy path lost its real fixture"
    );
    assert!(!text.contains("center_re="), "it should not carry both spellings");
    let (report, fields) = inspect_view_text(&text);
    assert_eq!(report.note(), None);
    let cx = fields.iter().find(|(k, _, _, _)| k == "center_re").expect("aliased to center_re");
    assert!(cx.1.starts_with("2.885512010930598"), "the aliased value is the file's own: {}", cx.1);
}
