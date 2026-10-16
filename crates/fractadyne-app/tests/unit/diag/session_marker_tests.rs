use super::*;

#[test]
fn a_folder_is_a_guests_only_while_another_session_runs_in_it() {
    let d = std::env::temp_dir().join(format!("fd-guest-test-{}", std::process::id()));
    std::fs::create_dir_all(&d).expect("temp dir");
    assert_eq!(guest_of(&d), None, "no marker");
    std::fs::write(d.join("session.running"), format!("x\nstarted now\npid {}\n", std::process::id())).expect("marker");
    assert_eq!(guest_of(&d), None, "this process's own marker");
    std::fs::write(d.join("session.running"), "x\nstarted then\npid 4294967290\n").expect("marker");
    assert_eq!(guest_of(&d), None, "a dead session's marker");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_marker_names_its_pid_and_an_old_one_has_none() {
    assert_eq!(marker_pid("fractadyne 0.3.0-beta.18 (build 1, gabc)\nstarted 2026-10-04\npid 4242\n"), Some(4242));
    assert_eq!(marker_pid("fractadyne 0.3.0-beta.17 (build 1, gabc)\nstarted 2026-10-03\n"), None);
    assert_eq!(marker_pid("pid x\n"), None);
}

#[test]
fn this_process_is_alive_and_a_finished_one_is_not() {
    // The test binary is fractadyne-<hash>: alive, and named like the app.
    assert!(fractadyne_alive(std::process::id()));
    // A process that has exited — and was never Fractadyne.
    #[cfg(windows)]
    let mut c = std::process::Command::new("cmd").args(["/C", "exit 0"]).spawn().expect("cmd runs");
    #[cfg(not(windows))]
    let mut c = std::process::Command::new("true").spawn().expect("true runs");
    let pid = c.id();
    c.wait().expect("it ends");
    assert!(!fractadyne_alive(pid));
    // A RUNNING process that is not Fractadyne: what a dead session's recycled pid looks like.
    #[cfg(windows)]
    {
        let mut other = std::process::Command::new("cmd").args(["/C", "ping -n 4 127.0.0.1 >nul"]).spawn().expect("cmd runs");
        assert!(!fractadyne_alive(other.id()), "a live cmd.exe passed for a Fractadyne session");
        let _ = other.kill();
        let _ = other.wait();
    }
}
