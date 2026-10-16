use super::{quiet_build, LineLimiter};

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
/// comment lines (a doc example). `src/` holds no test file: the tests live under `tests/unit/`.
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
                files.push(p);
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

/// Which reference builds' crumbs may be rate-limited (W10, 2026-10-16), on builds from the logs
/// that measured it. The shallow zoom's builds, four to eight a second, are small; a deep live
/// view's and the build an out-of-memory abort died in (2026-08-08) always log.
#[test]
fn only_small_reference_builds_are_rate_limited() {
    assert!(quiet_build(12_315, 206), "a shallow lookahead build (len 285 of 12,315 at 206 bits)");
    assert!(quiet_build(21_181, 435), "the deepest small build in the 2026-10-16 session");
    assert!(!quiet_build(10_000_000, 380), "the build the 2026-08-08 abort died in");
    assert!(!quiet_build(2_158_192, 16_810), "a 2^16,600 live view's reference");
    assert!(!quiet_build(2_009_392, 200_064), "a 2^200,000 view's reference");
}
