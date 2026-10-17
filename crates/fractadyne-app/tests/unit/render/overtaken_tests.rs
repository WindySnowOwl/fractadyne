use super::*;

#[test]
fn a_reactive_build_the_lookahead_overtook_is_dropped() {
    // The (49,3) spiral at 2^277: the lookahead installed at lag 0.88, the reactive build lands at 1.86.
    assert!(reactive_build_overtaken(true, true, false, 0.88, 1.86));
}

#[test]
fn a_zoom_outs_shallower_build_still_lands() {
    // Zoomed out past the installed BLA (lag 0.6, under the 0.85 edge): the build for the view now
    // sits nearer 1, so it installs although it is older.
    assert!(!reactive_build_overtaken(true, true, false, 0.6, 1.05));
}

#[test]
fn only_an_older_build_of_the_same_point_carrying_nothing_more_is_dropped() {
    // Spawned after the installed reference: it is the newest request, it installs.
    assert!(!reactive_build_overtaken(false, true, false, 0.88, 1.86));
    // Another point (an out-of-view re-anchor).
    assert!(!reactive_build_overtaken(true, false, false, 0.88, 1.86));
    // A longer, more precise or complete orbit.
    assert!(!reactive_build_overtaken(true, true, true, 0.88, 1.86));
    // No BLA on either side (`bla_dc_max_log2` is -inf there): nothing to compare, it installs.
    assert!(!reactive_build_overtaken(true, true, false, f64::NEG_INFINITY, f64::NEG_INFINITY));
    assert!(!reactive_build_overtaken(true, true, false, f64::NAN, 1.86));
}
