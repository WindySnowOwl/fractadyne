use super::*;

#[test]
fn gpus_takes_all_or_card_numbers() {
    assert_eq!(parse_gpus("all"), Ok(Gpus::All));
    assert_eq!(parse_gpus(" ALL "), Ok(Gpus::All));
    assert_eq!(parse_gpus("1,2"), Ok(Gpus::List(vec![1, 2])));
    // The same card twice: two sessions on one card, the one-card machine's test of the path.
    assert_eq!(parse_gpus("1, 1"), Ok(Gpus::List(vec![1, 1])));
    for bad in ["", "0", "1,", "two", "1;2", "6800"] {
        assert!(parse_gpus(bad).is_err(), "{bad:?}");
    }
}

#[test]
fn every_refused_option_says_why() {
    for (f, why) in REFUSED {
        assert!(f.starts_with("--") && !why.is_empty(), "{f}");
    }
}
