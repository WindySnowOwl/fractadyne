use super::*;
fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

#[test]
fn a_single_dash_spelling_of_a_real_option_is_named_and_corrected() {
    // The field case: `-play tour.toml` silently launched the GUI and a validation run
    // measured the saved session instead of the tour.
    match first_bad_option(&s(&["exe", "-play", "t.toml"])) {
        Some(BadOption::SingleDash { given, correct }) => {
            assert_eq!(given, "-play");
            assert_eq!(correct, "--play");
        }
        other => panic!("expected SingleDash, got {other:?}"),
    }
}

#[test]
fn values_and_documented_shorthands_never_trip_it() {
    assert_eq!(first_bad_option(&s(&["exe", "--center", "-0.75", "0.0"])), None);
    assert_eq!(first_bad_option(&s(&["exe", "--zoom", "-1e-3"])), None);
    assert_eq!(first_bad_option(&s(&["exe", "--render", "-o", "out.png"])), None);
    assert_eq!(first_bad_option(&s(&["exe", "--reset-state", "-y"])), None);
    assert_eq!(first_bad_option(&s(&["exe", "-V"])), None);
}

/// A free-form value that begins with a minus sign is a value: an expression for the centre (Help's
/// own example), a formula, a parameter list. These were refused as unknown options.
#[test]
fn free_form_values_that_start_with_a_minus_are_values() {
    assert_eq!(first_bad_option(&s(&["exe", "--render", "--center", "-3/4", "-0.5 + 0.25*cos(pi/4)"])), None);
    assert_eq!(first_bad_option(&s(&["exe", "--render", "--formula", "-z^2 + c", "--formula-params", "-0.5,0;-1,2"])), None);
    assert_eq!(first_bad_option(&s(&["exe", "--julia-c", "-0.8", "-0.156", "--render"])), None);
    // Only that many values: an unknown option after them is still caught.
    assert_eq!(
        first_bad_option(&s(&["exe", "--formula", "z^2 + c", "-x"])),
        Some(BadOption::UnknownShort("-x".into()))
    );
}

#[test]
fn unknown_options_are_flagged_in_both_spellings() {
    assert_eq!(
        first_bad_option(&s(&["exe", "--nonsense"])),
        Some(BadOption::UnknownLong("--nonsense".into()))
    );
    assert_eq!(
        first_bad_option(&s(&["exe", "-x"])),
        Some(BadOption::UnknownShort("-x".into()))
    );
}

#[test]
fn log_dir_with_a_value_is_a_known_option() {
    // Derived from the help reference — this test is what notices if the help entry is
    // ever dropped (which would make the guard reject the flag).
    assert_eq!(
        first_bad_option(&s(&["exe", "--log-dir", "D:/share/logs", "--selftest"])),
        None
    );
}

#[test]
fn a_correct_command_line_passes() {
    assert_eq!(
        first_bad_option(&s(&["exe", "--set", "TDR_BUDGET_MS=500", "--play", "t.toml"])),
        None
    );
}
