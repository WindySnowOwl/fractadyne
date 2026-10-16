use super::redact_path;

#[test]
fn the_home_path_is_redacted_in_every_spelling_it_appears_in() {
    let home = r"C:\Users\alice";
    let text = "logs directed to C:\\Users\\alice\\AppData\\x (--log-dir)\n\
                    config c:/users/ALICE/config.toml\n\
                    json {\"p\":\"C:\\\\Users\\\\alice\\\\f.fdn\"}\n\
                    bare C:\\Users\\alice";
    let out = redact_path(text, home);
    assert!(!out.to_ascii_lowercase().contains("alice"), "{out}");
    assert!(out.contains(r"logs directed to ~\AppData\x"), "{out}");
    assert!(out.contains("config ~/config.toml"), "{out}");
    assert!(out.contains(r#"{"p":"~\\f.fdn"}"#), "{out}");
    assert!(out.ends_with("bare ~"), "{out}");
}

#[test]
fn it_does_not_match_inside_a_longer_name_or_redact_a_trivial_home() {
    assert_eq!(redact_path(r"C:\Users\robin\x", r"C:\Users\rob"), r"C:\Users\robin\x");
    assert_eq!(redact_path(r"C:\Users\rob\x", r"C:\Users\rob"), r"~\x");
    assert_eq!(redact_path(r"C:\a C:\b", r"C:\"), r"C:\a C:\b", "a 3-char home is left alone");
    assert_eq!(redact_path("/home/bob/.config", "/home/bob/"), "~/.config", "trailing separator");
    // Non-ASCII account names survive the ASCII-only case folding without breaking a char.
    assert_eq!(redact_path(r"C:\Users\José\f", r"C:\Users\José"), r"~\f");
}
