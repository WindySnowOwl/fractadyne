use super::elide_middle;

#[test]
fn elide_keeps_both_ends_and_respects_the_budget() {
    assert_eq!(elide_middle("short", 46), "short", "under budget is untouched");
    let p = r"C:\Users\someone\AppData\Roaming\Fractadyne\Fractadyne\config";
    let e = elide_middle(p, 46);
    assert_eq!(e.chars().count(), 46, "must land exactly on the budget");
    assert!(e.starts_with(r"C:\Users"), "the drive must survive: {e}");
    assert!(e.ends_with("config"), "the leaf must survive: {e}");
}

#[test]
fn elide_is_char_safe_on_non_ascii_paths() {
    // A non-ASCII user directory is ordinary, and slicing by byte would panic on it.
    let p = "/home/josé-münchen/Ω-fractadyne/config/that/goes/on/and/on/for/a/while";
    let e = elide_middle(p, 30);
    assert_eq!(e.chars().count(), 30);
    assert!(e.contains('…'));
}
