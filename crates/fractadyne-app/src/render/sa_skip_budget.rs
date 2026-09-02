use super::usable_sa_skip;

#[test]
fn a_skip_at_or_past_the_budget_is_refused_not_clamped() {
    // The 2026-08-25 field case: budget dropped under a cached reference's skip.
    assert_eq!(usable_sa_skip(266_796, 224_000), 0, "the black-frame case");
    assert_eq!(usable_sa_skip(224_000, 224_000), 0, "equal is already past: iter >= max_iter");
    // Anything that genuinely leaves iterations to run is passed through untouched.
    assert_eq!(usable_sa_skip(223_999, 224_000), 223_999);
    assert_eq!(usable_sa_skip(0, 224_000), 0);
    assert_eq!(usable_sa_skip(37_494, 205_343), 37_494, "an ordinary deep frame is unaffected");
}
