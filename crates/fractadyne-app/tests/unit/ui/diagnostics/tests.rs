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
    // The GPU sweep's per-backend header stands in for a progress marker.
    assert!(is_progress_line("── Vulkan · NVIDIA GeForce RTX 3080 · driver: 596.21"));
    assert!(!is_progress_line("checks 113/113, goldens 17/17 — OK"));
}

#[test]
fn gputest_verdict_parses() {
    assert_eq!(
        parse_gputest_verdict("3 backend(s) tested, all sound."),
        Some((3, 0))
    );
    // The FAILED line wraps mid-sentence, so only its first physical line reaches the parser.
    assert_eq!(
        parse_gputest_verdict("2 of 3 backend(s) FAILED. Include this whole report in a bug report — a"),
        Some((3, 2))
    );
    assert_eq!(
        parse_gputest_verdict("No usable backend found — nothing tested."),
        Some((0, 0))
    );
    // Not the verdict line: table rows, headers, and the self-test's own verdict must not match.
    assert_eq!(parse_gputest_verdict("── Vulkan · NVIDIA · driver: 596.21"), None);
    assert_eq!(parse_gputest_verdict("checks 113/113, goldens 17/17 — OK"), None);
}

#[test]
fn gputest_headline_reads_cleanly_and_never_alarms() {
    // The all-sound and nothing-tested cases.
    assert!(gputest_headline(3, 0).contains("all double-float transforms intact"));
    assert!(gputest_headline(0, 0).contains("nothing tested"));
    // The NVIDIA case: it must NOT use the words "fail"/"fault"/"error" that would read as the
    // user's machine being broken — the folding is the finding, and the headline says "expected".
    let nv = gputest_headline(3, 3);
    assert!(nv.contains("expected on NVIDIA"), "{nv}");
    assert!(nv.contains("attach"), "{nv}");
    // Reject words that read as the user's machine being broken. NOT "error" — the phrase
    // "error-free transforms" is the correct name for what folds, and banning that substring is
    // what this assertion got wrong the first time.
    for alarm in ["fault", "broken", "unsound", "failed"] {
        assert!(!nv.to_lowercase().contains(alarm), "headline alarms with {alarm:?}: {nv}");
    }
}

#[test]
fn the_gpu_check_is_the_only_informational_test() {
    assert!(DiagTest::GpuTest.is_informational());
    assert!(!DiagTest::SelfTest.is_informational());
    assert!(!DiagTest::UiTest.is_informational());
    // And it is present in the dialog's test list.
    assert!(DiagTest::ALL.contains(&DiagTest::GpuTest));
}
