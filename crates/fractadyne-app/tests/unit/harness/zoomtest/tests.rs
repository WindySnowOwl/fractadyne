use super::{parse_click, parse_taps, ClickAt};

/// A factor alone clicks the centre; a factor and a point clicks that point; `@` a share of the view.
#[test]
fn click_parses() {
    assert_eq!(parse_click("100"), Some((100.0, ClickAt::Centre)));
    assert_eq!(
        parse_click("100, 0.3569159890518, 0.0834467673788"),
        Some((100.0, ClickAt::Point(0.3569159890518, 0.0834467673788)))
    );
    assert_eq!(parse_click("2.5,-0.75,0"), Some((2.5, ClickAt::Point(-0.75, 0.0))));
    assert_eq!(parse_click("100@0.7,0.35"), Some((100.0, ClickAt::View(0.7, 0.35))));
    assert_eq!(parse_click("25@ 0 , 0.999"), Some((25.0, ClickAt::View(0.0, 0.999))));
}

#[test]
fn bad_clicks_are_refused() {
    for bad in [
        "", "1", "0.5", "-100", "inf", "100,0.3", "100,0.3,0.1,9", "100,x,0.1", "100,0.3,nan", "a",
        // A share of the view is inside it, and comes alone.
        "100@", "100@0.5", "100@1,0.5", "100@0.5,-0.1", "100@0.5,0.5,0.5", "100,0.3,0.1@0.5,0.5", "1@0.5,0.5",
    ] {
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
