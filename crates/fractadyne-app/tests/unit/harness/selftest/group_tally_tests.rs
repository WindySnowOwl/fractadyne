use super::{add_tally, incomplete_groups, GroupTally};

fn t(tag: &str, refused: u64, reported: u64, skipped: u64) -> GroupTally {
    GroupTally { tag: tag.to_string(), refused, reported, skipped }
}

/// A tag that opens again (`iter-chunk` gates three blocks; every golden opens `goldens`) adds to
/// its own row, which keeps its first-seen place.
#[test]
fn a_group_that_opens_again_adds_to_its_row() {
    let mut g = Vec::new();
    add_tally(&mut g, t("iter-chunk", 3, 10, 2));
    add_tally(&mut g, t("live-split", 0, 4, 0));
    add_tally(&mut g, t("iter-chunk", 1, 5, 1));
    assert_eq!(g, vec![t("iter-chunk", 4, 15, 3), t("live-split", 0, 4, 0)]);
}

/// Incomplete = the guard refused a pass in it, whether or not a check reported the refusal (a
/// check that reports only on success leaves no row). The goldens report a row each, whatever.
#[test]
fn a_group_is_incomplete_when_a_pass_of_it_was_refused() {
    let g = vec![
        t("numeric", 5, 0, 0),
        t("symmetry", 0, 9, 0),
        t("abs-family", 2, 7, 2),
        t("goldens", 31, 0, 0),
    ];
    assert_eq!(incomplete_groups(&g), vec![t("numeric", 5, 0, 0), t("abs-family", 2, 7, 2)]);
}
