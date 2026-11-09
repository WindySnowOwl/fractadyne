//! The crash dialog's gate and text, and the marker that keeps a panic from being reported twice.

use super::*;

#[test]
fn only_a_persons_own_launch_gets_the_dialog() {
    // A person: no flags, a file to open, the two flags a person passes.
    for args in [
        &["fractadyne"][..],
        &["fractadyne", "C:\\views\\spiral.fdn"],
        &["fractadyne", "--adapter", "1"],
        &["fractadyne", "--language=de", "view.fdn"],
    ] {
        assert!(persons_flags_only(args), "{args:?} is a person's launch");
    }
    // Anything driven by a script, a harness or a task: a modal dialog would hang it.
    for args in [
        &["fractadyne", "--render", "--out", "x.png"][..],
        &["fractadyne", "--uitest", "dir"],
        &["fractadyne", "--selftest"],
        &["fractadyne", "--adapter", "1", "--shot", "view.fdn"],
        &["fractadyne", "--farm-client"],
    ] {
        assert!(!persons_flags_only(args), "{args:?} is not a person's launch");
    }
}

#[test]
fn the_reason_is_short_enough_to_read() {
    let msg = "wgpu error: Validation Error\n\nCaused by:\n  In Device::create_bind_group_layout\n    Too many bindings\n\nmore";
    assert_eq!(
        crash_reason(msg),
        "wgpu error: Validation Error\nCaused by:\nIn Device::create_bind_group_layout\nToo many bindings"
    );
    let long = "x".repeat(1000);
    let r = crash_reason(&long);
    assert_eq!(r.chars().count(), 400);
    assert!(r.ends_with('…'));
}

#[test]
fn a_panicked_session_names_its_report_in_its_marker() {
    assert_eq!(marker_panicked("x\nstarted now\npid 12\npanicked crash-1794180755-0.txt\n"), Some("crash-1794180755-0.txt"));
    assert_eq!(marker_panicked("x\nstarted now\npid 12\n"), None, "a hard kill: no panic line");
    assert_eq!(marker_panicked("panicked \n"), None, "an empty name is not a report");
}

#[test]
fn the_dialog_text_carries_every_placeholder() {
    // The panic hook fills these in; a translation that dropped one would hide what happened, the
    // error, what to try or the report's path. The English text is the template every translation
    // follows.
    let t = crate::ui::crash_dialog_text();
    for p in ["{what}", "{reason}", "{tips}", "{path}"] {
        assert!(t.body.contains(p), "{p} missing: {}", t.body);
    }
    assert!(t.body.contains(crate::ISSUES_URL), "{}", t.body);
    assert!(!t.title.is_empty());
    // Paragraphs: a system message box is plain text, and one block of it is hard to read.
    assert!(t.body.contains("

"), "{:?}", t.body);
    // Yes and No answer it: the text has to end on the question.
    assert!(t.body.trim_end().ends_with('?'), "{}", t.body);
}
