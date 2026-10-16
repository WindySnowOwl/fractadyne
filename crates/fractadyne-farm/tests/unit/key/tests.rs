use super::*;

#[test]
fn a_key_round_trips_through_its_text_whatever_the_spacing_and_case() {
    let k = FarmKey::generate().expect("rng");
    let text = k.to_text();
    assert!(text.starts_with("fdn1-"), "{text}");
    // 34 bytes = 272 bits = 55 letters, eleven groups of five.
    assert_eq!(text.trim_start_matches("fdn1-").split('-').count(), 11, "{text}");
    assert_eq!(FarmKey::from_text(&text).expect("own text"), k);
    let mangled = format!("  {} \r\n", text.to_ascii_uppercase().replace('-', " - "));
    assert_eq!(FarmKey::from_text(&mangled).expect("spacing and case ignored"), k);
}

/// ⭐A single wrong letter is reported as a typo — never accepted as a different key, which would
/// surface much later as an unexplained refusal by the controller.
#[test]
fn every_single_letter_typo_is_caught() {
    let k = FarmKey::from_bytes([7u8; 32]);
    let text = k.to_text();
    let body: Vec<char> = text["fdn1-".len()..].chars().collect();
    let mut caught = 0;
    let mut tried = 0;
    for i in 0..body.len() {
        if body[i] == '-' {
            continue;
        }
        let replacement = if body[i] == 'a' { 'b' } else { 'a' };
        let mut b = body.clone();
        b[i] = replacement;
        let typo = format!("fdn1-{}", b.iter().collect::<String>());
        tried += 1;
        match FarmKey::from_text(&typo) {
            Err(e) => {
                caught += 1;
                assert!(e.contains("typo") || e.contains("length"), "{e}");
            }
            Ok(other) => assert_ne!(other, k, "a typo decoded to the same key"),
        }
    }
    // A 16-bit checksum misses about 1 in 65,536 changes; over 55 positions we expect none.
    assert_eq!(caught, tried, "{} of {tried} typos slipped through", tried - caught);
}

#[test]
fn malformed_keys_say_what_is_wrong() {
    assert!(FarmKey::from_text("abc").unwrap_err().contains("fdn1-"));
    assert!(FarmKey::from_text("fdn1-0000").unwrap_err().contains("letters"));
    assert!(FarmKey::from_text("fdn1-aaaaa").unwrap_err().contains("length"));
}

#[test]
fn the_key_never_appears_in_debug_output() {
    let k = FarmKey::from_bytes([0xab; 32]);
    assert_eq!(format!("{k:?}"), "FarmKey(…)");
    let id = Identity::generate().expect("keypair");
    let d = format!("{id:?}");
    assert!(!d.contains(&crate::hex(id.private())), "{d}");
}

#[test]
fn an_identity_survives_its_file_and_is_made_once() {
    let dir = std::env::temp_dir().join(format!("fractadyne_farm_key_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("farm").join("identity.toml");
    let a = Identity::load_or_create(&path).expect("created");
    let b = Identity::load_or_create(&path).expect("loaded");
    assert_eq!(a.public(), b.public(), "a second load made a new identity");
    assert_eq!(a.private(), b.private());
    assert_eq!(a.fingerprint().len(), 19);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn base32_round_trips_every_length() {
    for n in 0..40 {
        let v: Vec<u8> = (0..n).map(|i| (i * 37 + 11) as u8).collect();
        assert_eq!(unbase32(&base32(&v)).expect("decodes"), (v, true), "length {n}");
    }
}
