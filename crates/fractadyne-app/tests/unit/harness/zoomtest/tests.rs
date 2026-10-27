use super::{parse_click, parse_taps};

/// A factor alone clicks the centre; a factor and a point clicks that point.
#[test]
fn click_parses() {
    assert_eq!(parse_click("100"), Some((100.0, None)));
    assert_eq!(parse_click("100, 0.3569159890518, 0.0834467673788"), Some((100.0, Some((0.3569159890518, 0.0834467673788)))));
    assert_eq!(parse_click("2.5,-0.75,0"), Some((2.5, Some((-0.75, 0.0)))));
}

#[test]
fn bad_clicks_are_refused() {
    for bad in ["", "1", "0.5", "-100", "inf", "100,0.3", "100,0.3,0.1,9", "100,x,0.1", "100,0.3,nan", "a"] {
        assert_eq!(parse_click(bad), None, "{bad:?} must not parse");
    }
}

/// The field's tap pattern parses; so does the older env var's form with spaces.
#[test]
fn taps_parse() {
    assert_eq!(parse_taps("30,0.25,1.0"), Some((30, 0.25, 1.0)));
    assert_eq!(parse_taps(" 6 , 0.5 , 0 "), Some((6, 0.5, 0.0)));
}

/// Anything a run could silently misread is refused: a field request that asked for taps must not
/// run one continuous glide instead.
#[test]
fn bad_taps_are_refused() {
    for bad in ["", "30", "30,0.25", "30,0.25,1.0,9", "0,0.25,1.0", "30,0,1.0", "30,-1,1.0", "30,0.25,-1", "30,inf,1", "a,b,c"] {
        assert_eq!(parse_taps(bad), None, "{bad:?} must not parse");
    }
}
