use super::*;

#[test]
fn selftest_verdict_parses() {
    assert_eq!(
        parse_selftest_verdict("checks 113/113, goldens 17/17 — OK"),
        Some((113, 113, 17, 17))
    );
    // The failing shape from a non-reference GPU (an AMD RX 6800 XT, 2026-08-14).
    assert_eq!(
        parse_selftest_verdict("checks 101/113, goldens 0/17 — FAILURES PRESENT"),
        Some((101, 113, 0, 17))
    );
    // Tolerates the verdict word being absent or changed.
    assert_eq!(
        parse_selftest_verdict("checks 1/2, goldens 3/4"),
        Some((1, 2, 3, 4))
    );
    assert_eq!(parse_selftest_verdict("report → validation/report.md"), None);
    assert_eq!(parse_selftest_verdict("checks 113"), None);
}

#[test]
fn uitest_verdict_parses() {
    assert_eq!(
        parse_uitest_verdict("=== --uitest complete: 25 steps, 25 pass / 0 warn / 0 fail ==="),
        Some((25, 0, 0))
    );
    assert_eq!(
        parse_uitest_verdict("=== --uitest complete: 25 steps, 24 pass / 1 warn / 0 fail ==="),
        Some((24, 1, 0))
    );
    assert_eq!(parse_uitest_verdict("something else entirely"), None);
}

#[test]
fn progress_lines_recognised() {
    assert!(is_progress_line("[selftest    5528ms] PASS direct-1e2 — ok"));
    assert!(is_progress_line("  [uitest] step 3/25 help"));
    assert!(!is_progress_line("checks 113/113, goldens 17/17 — OK"));
}
