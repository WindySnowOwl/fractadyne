use super::*;
fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

#[test]
fn the_flag_wins_over_the_variable() {
    let (d, src) = log_dir_override(&s(&["exe", "--log-dir", "D:/share"]), Some("E:/env"));
    assert_eq!(d, Some(PathBuf::from("D:/share")));
    assert_eq!(src, "--log-dir");
}

#[test]
fn the_variable_applies_when_the_flag_is_absent() {
    let (d, src) = log_dir_override(&s(&["exe", "--selftest"]), Some("//vger/share/logs"));
    assert_eq!(d, Some(PathBuf::from("//vger/share/logs")));
    assert_eq!(src, "FRACTADYNE_LOG_DIR");
    assert_eq!(log_dir_override(&s(&["exe"]), Some("")).0, None); // empty var = unset
}

#[test]
fn a_missing_value_yields_no_override_rather_than_a_guess() {
    // The CLI guard exits fatally on these; the resolver must not invent a directory
    // (or silently fall back to the env var) for the lines it logs before that exit.
    assert_eq!(log_dir_override(&s(&["exe", "--log-dir"]), Some("E:/env")).0, None);
    assert_eq!(
        log_dir_override(&s(&["exe", "--log-dir", "--play"]), None).0,
        None
    );
}

#[test]
fn no_override_means_the_default_location() {
    assert_eq!(log_dir_override(&s(&["exe", "--selftest"]), None).0, None);
}
