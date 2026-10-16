//! Tests for the console-verbosity rule.
//!
//! ⚠⚠**Three gates parse the `[fd-*]` banner off stderr** — `crosscheck_backends.py` reads
//! "backends compiled in" to prove it is testing the build it thinks it is, `generate_corpus.py`
//! reads "session:" to prove the staged session loaded rather than silently falling back to
//! defaults, and `gpu-validate.ps1` folds stderr into every step log. All three invoke fractadyne
//! WITH arguments, which is why the default keys off "are there any arguments" rather than a list
//! of headless modes: a list would drop a mode the day one was added, and the failure would look
//! like a gate that had quietly stopped checking.

use super::console_default;

fn args(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

/// ⭐⭐**The rule the user asked for**: a bare launch says nothing.
#[test]
fn a_bare_launch_is_quiet_and_any_argument_makes_it_talk() {
    assert!(!console_default(&args(&["fractadyne"]), None, None), "a bare GUI launch must be quiet");
    // Every shape of argument, because the rule is "any", not "a recognised one".
    for a in [
        vec!["fractadyne", "--selftest"],
        vec!["fractadyne", "--render", "-o", "x.png"],
        vec!["fractadyne", "--pickcheck", "nope.fdn"],
        vec!["fractadyne", "--version"],
        vec!["fractadyne", "session.fdn"],
    ] {
        assert!(console_default(&args(&a), None, None), "{a:?} should print");
    }
}

/// The two invocations the validation gates actually make, named so a future change to the rule
/// has to look them in the eye.
#[test]
fn the_stderr_parsing_gates_keep_their_output() {
    // crosscheck_backends.py — reads "backends compiled in" from this.
    assert!(console_default(&args(&["fractadyne", "--pickcheck", "definitely-not-a-file.fdn"]), None, None));
    // generate_corpus.py — reads "session:" from a headless render.
    assert!(console_default(&args(&["fractadyne", "--render", "--out", "a.png"]), None, None));
}

/// Precedence: the flag beats the variable, and the variable beats the default.
#[test]
fn the_explicit_forms_outrank_the_default_in_both_directions() {
    // Force ON for a bare launch.
    assert!(console_default(&args(&["fractadyne", "--console"]), None, None));
    assert!(console_default(&args(&["fractadyne"]), Some("1"), None));
    // Force OFF for an invocation that would otherwise print.
    assert!(!console_default(&args(&["fractadyne", "--selftest", "--no-console"]), None, None));
    assert!(!console_default(&args(&["fractadyne", "--selftest"]), Some("0"), None));
    // The flag wins over a variable that disagrees, in both directions.
    assert!(!console_default(&args(&["fractadyne", "--no-console"]), Some("1"), None));
    assert!(console_default(&args(&["fractadyne", "--console"]), Some("0"), None));
}

/// ⭐⭐**A trace whose output goes nowhere is not a trace.** Someone who sets FRACTADYNE_TRACE has
/// asked, in the plainest possible terms, to be told things; staying quiet because they happened to
/// launch without other arguments would look like the tracing was broken. ⚠But an explicit "off"
/// still wins — they may be reading the log file.
#[test]
fn setting_a_trace_turns_the_console_on_by_itself() {
    assert!(console_default(&args(&["fractadyne"]), None, Some("gpu")));
    assert!(console_default(&args(&["fractadyne"]), None, Some("1")));
    // Matching `trace_cats`: "0" is off, and then the bare-launch default applies again.
    assert!(!console_default(&args(&["fractadyne"]), None, Some("0")));
    // Explicit off still wins over a live trace.
    assert!(!console_default(&args(&["fractadyne"]), Some("0"), Some("gpu")));
    assert!(!console_default(&args(&["fractadyne", "--no-console"]), None, Some("gpu")));
}
