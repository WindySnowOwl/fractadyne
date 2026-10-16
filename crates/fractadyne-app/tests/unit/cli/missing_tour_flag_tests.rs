//! `missing_tour_flag` — the guard that turns a bare `--livetest` (a silent no-op GUI that once
//! wedged a gate behind the welcome dialog) into a loud error.
use super::missing_tour_flag;

fn args(v: &[&str]) -> Vec<String> {
    std::iter::once("fractadyne")
        .chain(v.iter().copied())
        .map(String::from)
        .collect()
}

#[test]
fn bare_flag_is_caught() {
    assert_eq!(missing_tour_flag(&args(&["--livetest"])), Some("--livetest"));
    assert_eq!(missing_tour_flag(&args(&["--divetest"])), Some("--divetest"));
    // Trailing flag, no file between: still missing.
    assert_eq!(
        missing_tour_flag(&args(&["--livetest", "--size", "480x270"])),
        Some("--livetest"),
        "a following flag is not a tour file"
    );
    // At the very end of argv.
    assert_eq!(
        missing_tour_flag(&args(&["--size", "480x270", "--livetest"])),
        Some("--livetest")
    );
}

#[test]
fn a_flag_with_a_tour_file_passes() {
    assert_eq!(missing_tour_flag(&args(&["--livetest", "tours/grand-tour.toml"])), None);
    assert_eq!(
        missing_tour_flag(&args(&["--livetest", "tours/grand-tour.toml", "--size", "480x270"])),
        None
    );
    assert_eq!(missing_tour_flag(&args(&["--divetest", "tours/x.toml"])), None);
}

#[test]
fn unrelated_invocations_are_untouched() {
    assert_eq!(missing_tour_flag(&args(&["--selftest"])), None);
    assert_eq!(missing_tour_flag(&args(&["--render", "-o", "out.png"])), None);
    assert_eq!(missing_tour_flag(&args(&[])), None);
}
