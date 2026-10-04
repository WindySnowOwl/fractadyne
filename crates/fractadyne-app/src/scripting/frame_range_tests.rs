//! `--frames A..B`: the render farm's unit of work. Every spelling either means exactly one set of
//! frames or is refused — never a silent "all" or "none".
use super::parse_frame_range;

#[test]
fn half_open_and_inclusive_spellings() {
    assert_eq!(parse_frame_range("0..10"), Ok((0, 10)));
    assert_eq!(parse_frame_range("120..136"), Ok((120, 136)));
    assert_eq!(parse_frame_range("5..=5"), Ok((5, 6)), "a single inclusive frame");
    assert_eq!(parse_frame_range("0..=9"), Ok((0, 10)), "inclusive end is converted to half-open");
    assert_eq!(parse_frame_range(" 3 .. 7 "), Ok((3, 7)), "spaces around the parts are allowed");
}

#[test]
fn empty_reversed_and_malformed_ranges_are_refused() {
    for bad in [
        "5..5",   // half-open with nothing in it
        "10..5",  // reversed
        "6..=5",  // reversed inclusive
        "5",      // not a range
        "..5",    // no start
        "5..",    // no end
        "-1..5",  // negative
        "a..b",   // not numbers
        "1.5..3", // not whole frames
        "0..=18446744073709551615", // inclusive end overflows the half-open form
        "",
    ] {
        assert!(parse_frame_range(bad).is_err(), "accepted --frames {bad:?}");
    }
}

#[test]
fn the_refusal_names_what_was_typed() {
    let e = parse_frame_range("10..5").unwrap_err();
    assert!(e.contains("10..5") && e.contains("no frames"), "{e}");
    let e = parse_frame_range("7").unwrap_err();
    assert!(e.contains("START..END"), "{e}");
}
