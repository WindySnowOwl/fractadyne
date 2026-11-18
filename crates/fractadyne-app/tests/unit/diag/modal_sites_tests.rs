//! Every native file picker in the app runs inside `diag::modal`, so a new one cannot bring back the
//! false `possible hang` and the dead-man's budget reset (2026-11-18: a Save dialog left open 38 s).

use std::path::Path;

/// The calls that block the UI thread until the person chooses.
const BLOCKING: [&str; 5] = [".pick_file()", ".pick_files()", ".save_file()", ".pick_folder()", ".pick_folders()"];

fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).expect("src is readable").flatten() {
        let p = e.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn every_file_picker_runs_inside_diag_modal() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    let (mut seen, mut bare) = (0, Vec::new());
    for f in &files {
        let text = std::fs::read_to_string(f).expect("a source file reads");
        for call in BLOCKING {
            for (at, _) in text.match_indices(call) {
                seen += 1;
                // Written `diag::modal(|| … .pick_file())`: the call closes the closure's parenthesis.
                if !text[at + call.len()..].starts_with(')') {
                    let line = text[..at].matches('\n').count() + 1;
                    bare.push(format!("{}:{line}", f.strip_prefix(&src).unwrap_or(f).display()));
                }
            }
        }
    }
    // Not vacuous: the app has 29 of them (2026-11-18).
    assert!(seen >= 20, "found only {seen} file pickers: did the scan miss the sources?");
    assert!(bare.is_empty(), "file pickers outside diag::modal: {bare:?}");
}
