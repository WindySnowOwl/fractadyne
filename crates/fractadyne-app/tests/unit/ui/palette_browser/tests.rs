use super::*;

/// Every vocabulary tag has a message: one that fell through to `tag_label`'s last arm would show
/// its bare id ("single-hue") in every language.
#[test]
fn every_vocabulary_tag_has_a_name() {
    for d in fractadyne_color::tags::TAGS {
        let l = tag_label(d.id);
        assert!(!l.is_empty() && l != d.id, "{}: no message, shown as its id", d.id);
    }
}

/// A typed tag that names a vocabulary tag, by id or by its name, IS that tag, so filtering by
/// "Calm" finds a gradient the user tagged "calm"; anything else is kept as typed, tidied.
#[test]
fn a_typed_tag_becomes_a_vocabulary_tag_when_it_names_one() {
    assert_eq!(normalize_user_tag("  Calm "), Some("calm".into()));
    assert_eq!(normalize_user_tag("single-hue"), Some("single-hue".into()));
    assert_eq!(normalize_user_tag("Single hue"), Some("single-hue".into()));
    assert_eq!(normalize_user_tag("  my   favourites "), Some("my favourites".into()));
    assert_eq!(normalize_user_tag("   "), None);
    assert_eq!(normalize_user_tag(&"x".repeat(100)).map(|s| s.chars().count()), Some(MAX_TAG_CHARS));
}

/// A filter keeps a palette only when it carries EVERY chosen tag, case ignored.
#[test]
fn a_filter_keeps_only_palettes_with_every_tag() {
    let t = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert!(passes(&t(&["blues", "calm"]), &t(&["blues"])));
    assert!(passes(&t(&["blues", "calm"]), &t(&["blues", "calm"])));
    assert!(!passes(&t(&["blues"]), &t(&["blues", "calm"])));
    assert!(passes(&t(&["Favourites"]), &t(&["favourites"])));
    assert!(passes(&t(&["blues"]), &t(&[])), "no filter keeps everything");
}
