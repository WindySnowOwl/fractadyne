use super::*;

/// Checklist step 87, "every Help section renders and is non-empty". The failure it guards is
/// specific and silent: the section list and the bodies used to be a list plus a `match` with
/// a catch-all arm, so a section added to the list and not to the match rendered the ABOUT
/// text under its own heading — wrong content, no error, and only visible to someone who
/// clicked that exact entry and knew what should have been there.
///
/// Readability of the prose stays human; that every entry has its own body does not.
#[test]
fn help_sections_all_render() {
    assert_eq!(
        SECTION_NAMES.len(),
        SECTION_BODIES.len(),
        "a Help section has no body, or a body has no entry in the contents list"
    );
    for (i, n) in SECTION_NAMES.iter().enumerate() {
        assert!(!n.trim().is_empty(), "section {i} has no name");
    }
    // Names are unique — two entries with the same label are indistinguishable in the list.
    let mut sorted: Vec<&str> = SECTION_NAMES.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), SECTION_NAMES.len(), "two Help sections share a name");
    // And no two entries share a BODY, which is the shape the old catch-all produced.
    for i in 0..SECTION_BODIES.len() {
        for j in (i + 1)..SECTION_BODIES.len() {
            assert!(
                !std::ptr::fn_addr_eq(SECTION_BODIES[i], SECTION_BODIES[j]),
                "{:?} and {:?} render the same body",
                SECTION_NAMES[i],
                SECTION_NAMES[j]
            );
        }
    }
    // About is the last entry, which the window relies on when it clamps an out-of-range
    // index — a stale `help_section` from an older session must land somewhere sensible.
    assert_eq!(*SECTION_NAMES.last().unwrap(), "About");
}

/// The Go-to dialog opens Help at the expression reference BY NAME; a rename that missed
/// the constant would silently open Overview instead.
#[test]
fn section_index_finds_every_named_section() {
    let i = section_index(EXPRESSIONS_SECTION);
    assert_eq!(SECTION_NAMES[i], EXPRESSIONS_SECTION);
    assert!(std::ptr::fn_addr_eq(SECTION_BODIES[i], help_expressions as fn(&mut egui::Ui)));
    let i = section_index(ACKNOWLEDGMENTS_SECTION);
    assert!(std::ptr::fn_addr_eq(SECTION_BODIES[i], help_acknowledgments as fn(&mut egui::Ui)));
    assert_eq!(section_index("no such section"), 0, "an unknown name falls back to Overview");
}
