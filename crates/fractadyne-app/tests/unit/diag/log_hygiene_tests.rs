use super::LineLimiter;

/// A flapping transition prints once per gap, and the next line that prints carries the count.
#[test]
fn a_rate_limited_line_prints_once_per_gap_and_counts_what_it_held() {
    let mut l = LineLimiter::default();
    assert_eq!(l.admit(0, 5000), Some(0), "the first occurrence always prints");
    for t in (32..4999).step_by(32) {
        assert_eq!(l.admit(t, 5000), None, "held at {t} ms");
    }
    let held = (32..4999).step_by(32).count() as u32;
    assert_eq!(l.admit(5000, 5000), Some(held), "and the next one says how many");
    assert_eq!(l.admit(5001, 5000), None);
    assert_eq!(l.admit(12_000, 5000), Some(1));
}

/// Every category the app logs under must appear as `[fd-<cat>]` in DIAGNOSTICS.md's prefix
/// table — a documentation check that can genuinely go red (design W10). The source is scanned
/// for `log_line("…"`, `diag::trace("…"` and `trace_on("…"` with a literal category, ignoring
/// comment lines (a doc example) and the test-only files.
#[test]
fn every_log_category_is_documented() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let docs = std::fs::read_to_string(root.join("../../DIAGNOSTICS.md")).expect("DIAGNOSTICS.md");
    let mut files = Vec::new();
    let mut stack = vec![root.join("src")];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") {
                let name = p.file_name().unwrap().to_string_lossy().into_owned();
                let test_only = name.ends_with("_tests.rs")
                    || ["console.rs", "device_loss_hint.rs", "log_dir.rs"].contains(&name.as_str())
                        && p.parent().is_some_and(|q| q.ends_with("diag"));
                if !test_only {
                    files.push(p);
                }
            }
        }
    }
    let mut cats = std::collections::BTreeSet::new();
    for f in &files {
        let text: String = std::fs::read_to_string(f)
            .unwrap()
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        for call in ["log_line(", "diag::trace(", "trace_on("] {
            for (i, _) in text.match_indices(call) {
                let rest = text[i + call.len()..].trim_start();
                if let Some(lit) = rest.strip_prefix('"') {
                    if let Some(end) = lit.find('"') {
                        let cat = &lit[..end];
                        if !cat.is_empty() && cat.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                            cats.insert(cat.to_string());
                        }
                    }
                }
            }
        }
    }
    assert!(cats.len() >= 20, "the scan found only {} categories — it has stopped seeing the calls: {cats:?}", cats.len());
    let missing: Vec<_> = cats.iter().filter(|c| !docs.contains(&format!("[fd-{c}]"))).collect();
    assert!(missing.is_empty(), "logged but not in DIAGNOSTICS.md's prefix table: {missing:?}");
}
