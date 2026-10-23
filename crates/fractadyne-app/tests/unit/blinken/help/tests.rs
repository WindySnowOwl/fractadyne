use super::*;
use crate::blinken::panel::{HELP_KEYS, LAMP_KEYS};

#[test]
fn every_instrument_lamp_and_readout_on_the_panel_has_a_card() {
    for key in HELP_KEYS.iter().chain(LAMP_KEYS.iter()) {
        assert!(card(key).is_some(), "no help card for {key}");
    }
}

#[test]
fn every_card_is_reachable_from_the_panel_and_its_key_is_unique() {
    for (i, (key, _)) in CARDS.iter().enumerate() {
        assert!(HELP_KEYS.contains(key) || LAMP_KEYS.contains(key), "card {key} is on nothing");
        assert!(!CARDS[..i].iter().any(|(k, _)| k == key), "card {key} twice");
    }
}

#[test]
fn a_card_with_a_trouble_list_says_what_healthy_looks_like_too() {
    for (key, build) in CARDS.iter() {
        let c = build();
        assert!(!c.title.is_empty() && !c.what.is_empty(), "{key}");
        if !c.bad.is_empty() {
            assert!(!c.good.is_empty(), "{key}: trouble without healthy");
        }
    }
}

#[test]
fn a_cards_title_is_shown_as_its_message_holds_it_in_capitals() {
    // The panel's legends are engraved in capitals, and the messages hold them so (an uppercased
    // translation would get Turkish i and German ß wrong): the English ones are already capitals.
    for (key, build) in CARDS.iter() {
        let t = build().title;
        assert_eq!(t, t.to_uppercase(), "{key}");
    }
    assert_eq!(card("lamp:refAhead").map(|c| c.title), Some("REF AHEAD".to_owned()));
}
