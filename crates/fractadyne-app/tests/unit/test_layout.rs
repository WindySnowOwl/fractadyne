//! No crate keeps test code in `src/` (design/test-code-separation.md): every unit test lives
//! under the crate's `tests/unit/`, declared from the code it tests with
//! `#[cfg(test)] #[path = "…/tests/unit/…"] mod name;`. `scripts/check_test_layout.py` is the
//! whole check (canonical paths, orphans, `--selfcheck`); this is its core rule, run by every
//! `cargo test`, because the convention it replaces eroded between the runs of a check nobody
//! had to run.

use std::path::{Path, PathBuf};

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).expect("a source directory").flatten() {
        let p = e.path();
        if p.is_dir() {
            sources(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// The violations in one file's lines: a `#[test]`, or a `#[cfg(test)]` that is not a
/// declaration of a module whose file is under `tests/unit/`.
fn violations(lines: &[&str]) -> Vec<(usize, &'static str)> {
    let mut found = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        if *l == "#[test]" {
            found.push((i + 1, "a #[test]"));
        }
        if *l != "#[cfg(test)]" {
            continue;
        }
        // The declaration's other attributes and comments, then the item it declares.
        let mut path_into_tests = false;
        let mut item = "";
        for l in &lines[i + 1..] {
            if l.is_empty() || l.starts_with("//") {
                continue;
            }
            if l.starts_with("#[") {
                path_into_tests |= l.starts_with("#[path") && l.contains("tests/unit/");
                continue;
            }
            item = l;
            break;
        }
        let declares = item.split("mod ").nth(1).is_some_and(|r| r.ends_with(';'));
        if !declares {
            found.push((i + 1, "a test-only item, or an inline test module"));
        } else if !path_into_tests {
            found.push((i + 1, "a test module whose file is not under tests/unit/"));
        }
    }
    found
}

#[test]
fn no_crate_keeps_test_code_in_src() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    for c in std::fs::read_dir(&crates).expect("crates/").flatten() {
        let src = c.path().join("src");
        if src.is_dir() {
            sources(&src, &mut files);
        }
    }
    assert!(files.len() > 100, "VACUOUS: the scan found {} source files", files.len());
    let (mut declared, mut found) = (0, Vec::new());
    for f in &files {
        let text = std::fs::read_to_string(f).expect("readable source");
        let lines: Vec<&str> = text.lines().map(str::trim).collect();
        declared += lines.iter().filter(|l| **l == "#[cfg(test)]").count();
        for (n, what) in violations(&lines) {
            found.push(format!("{}:{n}: {what}", f.display()));
        }
    }
    assert!(declared > 150, "VACUOUS: the scan saw only {declared} test-module declarations");
    assert!(found.is_empty(), "test code in src/ (see CONTRIBUTING.md):\n{}", found.join("\n"));
}

/// The rule goes red for each shape it exists to stop, and passes the one declaration it allows.
#[test]
fn the_layout_rule_goes_red() {
    let allowed = ["#[cfg(test)]", "#[path = \"../tests/unit/a/tests.rs\"]", "mod tests;"];
    assert!(violations(&allowed).is_empty());
    let cases: [&[&str]; 4] = [
        &["#[test]", "fn t() {}"],
        &["#[cfg(test)]", "mod tests {", "}"],
        &["#[cfg(test)]", "fn helper() {}"],
        &["#[cfg(test)]", "mod beside;"],
    ];
    for lines in cases {
        assert!(!violations(lines).is_empty(), "{lines:?} must be a violation");
    }
}
