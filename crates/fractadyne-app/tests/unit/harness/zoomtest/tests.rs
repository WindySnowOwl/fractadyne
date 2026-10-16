use super::parse_taps;

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
