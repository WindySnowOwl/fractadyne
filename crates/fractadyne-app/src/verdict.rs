//! What a gating harness's exit code says (W8, design/live-render-robustness.md §7.8).
//!
//! ⭐**"Did not run the experiment" is not "ran it and failed", and neither is a pass.** The
//! harnesses a release reads could tell the two apart and then exited 2 for both — `--motiontest`
//! computed a VACUOUS term and threw it away, `--autodive` called a run that never reached the
//! lethal regime an assertion failure (U8). A vacuous run then reads as a regression, or a
//! regression gets explained away as vacuous. One convention, for `--motiontest`, `--autodive`,
//! `--zoomtest`, `--soak` and `--recordtest`; the torture ladder's `classify` reads it.
//!
//! (1 is not part of it: a harness whose own log broke a `--logcheck` rule turns 0 into 1.)

/// The experiment ran, and every criterion held.
pub(crate) const PASS: i32 = 0;
/// The experiment ran, and what it tests failed.
pub(crate) const RED: i32 = 2;
/// The experiment did not run: the regime it tests was never reached, or too little was measured
/// to judge. Never a pass — and not a failure of the thing under test either.
pub(crate) const VACUOUS: i32 = 3;
/// The in-process watchdog: no verdict within the harness's deadline (the frame loop stopped).
pub(crate) const WATCHDOG: i32 = 4;

/// The code for a run with real failures (`red`) and vacuous terms (`vacuous`). A real failure
/// wins: a run that was partly vacuous and also failed something it did test has failed.
pub(crate) fn code(red: bool, vacuous: bool) -> i32 {
    if red {
        RED
    } else if vacuous {
        VACUOUS
    } else {
        PASS
    }
}

/// The word a harness prints for `code`.
pub(crate) fn word(code: i32) -> &'static str {
    match code {
        PASS => "PASS",
        RED => "FAIL",
        VACUOUS => "VACUOUS",
        WATCHDOG => "WATCHDOG",
        _ => "FAIL",
    }
}

#[cfg(test)]
#[path = "../tests/unit/verdict/tests.rs"]
mod tests;
