//! A print whose reader has gone (`fractadyne --help | head`) ends the program quietly; every
//! other panic, a failed print included, is still a crash.

use super::*;

#[test]
fn a_print_to_a_closed_pipe_is_recognized() {
    // The messages std's print macros panic with, verbatim (the Windows one from the three crash
    // reports of 2026-11-01..03).
    let closed = if cfg!(windows) {
        [
            "failed printing to stdout: The pipe is being closed. (os error 232)",
            "failed printing to stderr: The pipe has been ended. (os error 109)",
        ]
    } else {
        ["failed printing to stdout: Broken pipe (os error 32)", "failed printing to stderr: Broken pipe (os error 32)"]
    };
    for m in closed {
        assert!(closed_pipe_panic(m), "{m}");
    }
}

#[test]
fn any_other_panic_is_still_a_crash() {
    for m in [
        // A print that failed for another reason: a full disk behind a redirect, an invalid handle.
        "failed printing to stdout: There is not enough space on the disk. (os error 112)",
        "failed printing to stdout: The handle is invalid. (os error 6)",
        // The code not at the end, or not a number.
        "failed printing to stdout: (os error 232) and then something",
        "failed printing to stdout: The pipe is being closed. (os error two)",
        // The words alone, from anywhere else.
        "index out of bounds: the len is 3 but the index is 232",
        "called `Result::unwrap()` on an `Err` value: The pipe is being closed. (os error 232)",
        "",
    ] {
        assert!(!closed_pipe_panic(m), "{m}");
    }
}

#[test]
fn the_exit_code_is_a_shells_sigpipe() {
    assert_eq!(CLOSED_PIPE_EXIT, 128 + 13);
}
