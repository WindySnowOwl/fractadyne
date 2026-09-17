//! Provenance of the pixels in a view's iteration texture — the guard that catches a ghost the
//! frame flags and the viewport comparison both miss.
//!
//! The app already refuses to fold a reprojected, held, pinned or mid-grid frame
//! (`accum_fold_clean`), and it already catches a fold whose VIEWPORT differs from sample 0's
//! (`⚠FOLD AT ANOTHER VIEW`). A field session on 2026-09-17 showed a ghost with that tripwire
//! silent through 41 accumulation runs, which leaves exactly one place for the stale pixels to be:
//! the texture itself, carried across a view change by a resize that seeded from the old content.

use super::{content_after_frame, seed_allowed};

const A: u64 = 0x1111_2222_3333_4444; // one view
const B: u64 = 0x5555_6666_7777_8888; // another

#[test]
fn a_resize_may_seed_from_this_view_at_another_size() {
    // The case seeding exists for: the motion-resolution controller changed the size mid-settle,
    // so refining in place beats refining against black.
    assert!(seed_allowed(true, true, Some(A), false, A));
}

#[test]
fn a_resize_may_not_seed_another_view_forward() {
    // The defect. Seeding here copies another location under the tiles that follow, and whatever
    // they do not cover stays on screen and becomes sample 0 of the average at full weight.
    assert!(!seed_allowed(true, true, Some(B), false, A));
}

#[test]
fn a_resize_may_not_seed_content_that_is_already_foreign() {
    // Right view, but something older is still under it — carrying that forward carries the ghost.
    assert!(!seed_allowed(true, true, Some(A), true, A));
}

#[test]
fn there_is_nothing_to_seed_from_before_the_first_frame() {
    assert!(!seed_allowed(false, true, Some(A), false, A));
    assert!(!seed_allowed(true, true, None, false, A));
}

#[test]
fn a_full_frame_pass_makes_the_texture_wholly_its_own_view() {
    // Every pixel is rewritten, so whatever was underneath is gone — including a foreign carry.
    assert_eq!(content_after_frame(Some(B), true, A, true, false), (Some(A), false));
    assert_eq!(content_after_frame(None, false, A, true, false), (Some(A), false));
}

#[test]
fn a_tile_over_another_views_pixels_leaves_them_foreign() {
    // The tile covers its rect and no more; the rest is still view B.
    assert_eq!(content_after_frame(Some(B), false, A, false, false), (Some(A), true));
}

#[test]
fn a_tile_on_a_blank_texture_is_not_foreign() {
    // Blank is the absence of pixels, not another view's pixels. Conflating the two would turn
    // every deep tiled settle into a permanent refusal.
    assert_eq!(content_after_frame(None, false, A, false, false), (Some(A), false));
}

#[test]
fn a_tile_over_this_views_own_pixels_stays_clean() {
    // The ordinary tiled settle: the frame builds rect by rect and never stops being view A.
    assert_eq!(content_after_frame(Some(A), false, A, false, false), (Some(A), false));
}

#[test]
fn a_tile_does_not_launder_content_that_was_already_foreign() {
    assert_eq!(content_after_frame(Some(A), true, A, false, false), (Some(A), true));
}

#[test]
fn a_reprojection_is_foreign_however_it_is_framed() {
    // The viewport is current; the pixels are the previous view's, warped. That is precisely the
    // combination the viewport tripwire cannot see, so it must be recorded here.
    assert_eq!(content_after_frame(Some(A), false, A, true, true), (Some(A), true));
    assert_eq!(content_after_frame(Some(B), false, A, false, true), (Some(A), true));
}

#[test]
fn a_clean_texture_survives_a_settle_and_a_resize_at_one_view() {
    // The sequence a normal deep settle walks: full frame, then tiles, then a resolution change
    // that is allowed to seed, then more tiles — and it must still be foldable at the end.
    let (mut s, mut f) = content_after_frame(None, false, A, true, false);
    for _ in 0..4 {
        let next = content_after_frame(s, f, A, false, false);
        s = next.0;
        f = next.1;
    }
    assert!(seed_allowed(true, true, s, f, A), "a same-view resize must still be seedable");
    assert_eq!((s, f), (Some(A), false));
}

#[test]
fn a_view_change_cannot_be_laundered_by_the_settle_that_follows_it() {
    // The ghost's whole path, end to end: view A is on screen, the view changes to B, and the
    // resize is REFUSED, so the texture is cleared rather than carrying A forward. Tiles of B then
    // build on blank, and the result is clean. Without the refusal the pair below stays foreign.
    let (s, f) = content_after_frame(None, false, A, true, false);
    assert!(!seed_allowed(true, true, s, f, B), "B must not seed from A");
    // Cleared: `resize` resets both, and blank has nothing to bleed through. The tiles of B that
    // follow build on blank, so they stay clean and the view can still accumulate once the app's
    // grid drains — the point of separating "foreign" from "unfinished".
    let (mut s, mut f) = (None, false);
    for _ in 0..3 {
        let next = content_after_frame(s, f, B, false, false);
        s = next.0;
        f = next.1;
    }
    assert_eq!((s, f), (Some(B), false), "a cleared texture must not read as foreign");
}

/// ⚠The regression this design nearly shipped: if a cleared texture read as foreign, every deep
/// view — which settles by tiles, never by one full-frame pass — would refuse to accumulate, and
/// the despeckle feature would be silently off exactly where it matters most.
#[test]
fn a_deep_view_that_only_ever_settles_by_tiles_can_still_accumulate() {
    let (mut s, mut f) = (None, false);
    for _ in 0..16 {
        let next = content_after_frame(s, f, A, false, false);
        s = next.0;
        f = next.1;
    }
    assert_eq!((s, f), (Some(A), false));
}
