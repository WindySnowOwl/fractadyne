use super::thumb_crop;

/// The regression exactly: an unset (all-zero) rect against a real window shot must fall
/// back to the WHOLE frame -- the pre-beta.3 clamp turned it into a 1x1 crop of the
/// window's top-left chrome pixel, which is the invisible gray dot of the field report.
#[test]
fn a_zero_rect_captures_the_whole_shot_not_one_pixel() {
    assert_eq!(thumb_crop([0, 0, 0, 0], 1920, 1080), (0, 0, 1920, 1080));
}

#[test]
fn a_normal_rect_is_passed_through() {
    assert_eq!(thumb_crop([0, 40, 1920, 1040], 1920, 1080), (0, 40, 1920, 1040));
}

#[test]
fn a_rect_hanging_past_the_shot_is_clamped_inside_it() {
    // e.g. a shot taken at a smaller window than the rect was measured at.
    assert_eq!(thumb_crop([0, 40, 1920, 1040], 1280, 720), (0, 40, 1280, 680));
    // Origin beyond the shot entirely: clamped to the last pixel, 1x1 -- the honest
    // answer for a truly disjoint rect (not reachable from the panel's own geometry).
    assert_eq!(thumb_crop([5000, 5000, 100, 100], 1280, 720), (1279, 719, 1, 1));
}

#[test]
fn a_degenerate_shot_never_panics() {
    assert_eq!(thumb_crop([0, 0, 100, 100], 0, 0), (0, 0, 1, 1));
}
