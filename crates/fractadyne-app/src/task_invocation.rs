use super::is_task_invocation;

#[test]
fn a_bare_interactive_launch_is_not_a_task() {
    assert!(!is_task_invocation::<&str>(&[]));
    assert!(!is_task_invocation(&["--fast"]));
    // A view handed over on the command line is still someone sitting in front of the app.
    assert!(!is_task_invocation(&["--center", "-0.75", "0.0"]));
}

#[test]
fn every_harness_and_offline_job_is_a_task() {
    // These all drive the real windowed app or run long unattended, and none of them may
    // resurrect itself on a lost device.
    for flag in [
        "--selftest", "--livetest", "--uitest", "--juliadive", "--torture", "--render",
        "--render-tour", "--bench-matrix", "--gputest", "--resizetest", "--motiontest",
        "--shot", "--soak",
    ] {
        assert!(is_task_invocation(&[flag]), "{flag} must count as a task invocation");
        // ...including when it is not the first argument.
        assert!(is_task_invocation(&["--size", "480x270", flag]), "{flag} missed mid-argv");
    }
}
