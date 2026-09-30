use super::*;

#[test]
fn a_source_round_trips_through_one_line() {
    for src in [
        "z^2 + c",
        "t = sqr(z)\nz = t*t + c ; comment",
        "z = z^3 - p1*z + c\r\nz = z + p2\r",
        "back\\slash and \\n literally",
        "",
    ] {
        let line = escape_line(src);
        assert!(!line.contains('\n') && !line.contains('\r'), "{line:?}");
        let normalised = src.replace("\r\n", "\n").replace('\r', "\n");
        assert_eq!(unescape_line(&line), normalised, "{src:?}");
    }
}

#[test]
fn parameters_round_trip_and_bad_ones_are_refused() {
    let f = CustomFormula::compile("z^2 + p1*z + p2", &[(0.25, -0.5), (1e-17, 3.0)]).unwrap();
    assert_eq!(f.params_used(), 2);
    let line = f.params_line();
    assert_eq!(parse_params_line(&line).unwrap(), vec![(0.25, -0.5), (1e-17, 3.0)]);
    assert_eq!(parse_params_line("").unwrap(), vec![]);
    assert_eq!(parse_params_line("1,2;x,3"), None);
    assert_eq!(parse_params_line("1,inf"), None);
    assert_eq!(parse_params_line("1"), None);
}

#[test]
fn compile_reports_parse_errors_with_their_place() {
    let err = CustomFormula::compile("z = z^2 +", &[]).err().unwrap();
    assert!(err.contains("line 1"), "{err}");
    let f = CustomFormula::compile("z^2 + c", &[]).unwrap();
    assert_eq!(f.params.len(), MAX_PARAMS);
    // A ring formula deep-zooms; one with a function or a division renders direct and says which
    // feature stops it — and the dialog's note (of the typed text) agrees with the shader the
    // renderer selects its mode by.
    for (src, deep, why) in [
        ("z^2 + c", true, ""),
        ("sin(z) + c", false, "an elementary function"),
        ("z^2 + 1/c", false, "division"),
        ("z^2.5 + c", false, "a non-integer power"),
    ] {
        let f = CustomFormula::compile(src, &[]).unwrap();
        let note = depth_note_for(&f.formula);
        assert_eq!(note.contains("Deep zoom by perturbation"), deep, "{src}: {note}");
        assert_eq!(f.shader.perturbation.is_ok(), deep, "{src}: the renderer disagrees with the note");
        assert!(deep || (note.contains("single-precision step of c") && note.contains(why)), "{src}: {note}");
    }
    // Different parameters, different module: the key tells two renders apart.
    let a = CustomFormula::compile("z^2 + p1", &[(0.1, 0.0)]).unwrap();
    let b = CustomFormula::compile("z^2 + p1", &[(0.2, 0.0)]).unwrap();
    assert_ne!(a.shader.key, b.shader.key);
}
