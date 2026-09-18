//! The held-frame magnification floor, pinned against the two regressions it sits between.
//!
//! A held frame may be magnified while a fresh reference builds, and the bound on that is NOT one
//! number: a continuous dive has to keep the true scale (a clamped scale translates a moving frame
//! too far and it visibly slides), while an instantaneous jump has to be clamped (100× leaves a
//! flat patch covering the panel, black if the patch was dark). Both failures have been shipped
//! before, in opposite directions, so both bounds are held here.

use crate::render::hold_scale_floor;
use crate::tunables::HELD_JUMP_MAX_OCT;

#[test]
fn a_jump_may_not_magnify_past_the_tunable() {
    let floor = hold_scale_floor(true);
    // Magnification is 1/scale, so the floor IS the reciprocal of the allowed magnification.
    assert!(
        (floor - (-HELD_JUMP_MAX_OCT as f32).exp2()).abs() < 1.0e-9,
        "jump floor {floor} does not match HELD_JUMP_MAX_OCT"
    );
    let max_mag = 1.0 / floor;
    assert!(
        (max_mag - 8.0).abs() < 1.0e-3,
        "3 octaves should cap the held frame at 8x, got {max_mag}x"
    );
}

#[test]
fn the_100x_click_that_produced_the_black_frame_is_now_clamped() {
    // The measured case: a 100× click-to-zoom is 6.64 octaves, which magnified the held frame 100×
    // and left ~11 px of a 1084 px panel — the flat field reported as "the screen goes black".
    let true_scale = (-6.64_f32).exp2();
    let clamped = true_scale.max(hold_scale_floor(true));
    assert!(clamped > true_scale, "the jump case must clamp 6.64 octaves");
    // Content remaining on a 1084 px panel, in pixels of the frozen frame.
    let px_before = 1084.0 * true_scale;
    let px_after = 1084.0 * clamped;
    assert!(px_before < 12.0, "sanity: the unclamped patch was ~11 px, got {px_before}");
    assert!(
        px_after > 100.0,
        "the clamped patch must carry real content, got {px_after} px"
    );
}

#[test]
fn a_dive_keeps_its_true_scale() {
    // ⛔The opposite regression: flooring a MOVING frame slides it. A dive that has fallen 13
    // octaves behind (the old 1e-4 floor's breaking point) must pass through untouched.
    let floor = hold_scale_floor(false);
    for oct in [0.5_f32, 3.0, 6.64, 13.0, 30.0] {
        let true_scale = (-oct).exp2();
        assert_eq!(
            true_scale.max(floor),
            true_scale,
            "a dive at {oct} octaves behind must not be clamped"
        );
    }
    // ...but still inside f32's range, which is what the 2^-40 bound is for.
    assert!(floor > 0.0 && floor < (-39.0_f32).exp2());
}

#[test]
fn the_jump_floor_is_the_looser_of_the_two() {
    // Stated as an ordering so a future edit cannot swap the two branches unnoticed.
    assert!(
        hold_scale_floor(true) > hold_scale_floor(false),
        "the jump case is the CLAMPED one"
    );
}
