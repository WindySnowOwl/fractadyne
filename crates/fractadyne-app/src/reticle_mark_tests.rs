//! What the reticle's crosshair must be, pinned as pixel classifications.
//!
//! The mark is stamped into the reticle image ([`crate::reticle_mark`]) rather than painted over
//! it, so that each of its pixels can take an ink chosen from the fractal underneath. These tests
//! hold the two properties that made it worth moving: the exact point stays UNPAINTED, and every
//! core pixel is fully surrounded by outline pixels, which is what guarantees contrast on content
//! of any colour — including the amber-on-amber that hid the original four accent strokes.

use crate::{reticle_mark, ReticleMark};

/// The radius of a real reticle (`RETICLE_PX / 2 - 1`), so the geometry under test is the shipped
/// geometry rather than a convenient number.
const R: f32 = crate::RETICLE_PX as f32 * 0.5 - 1.0;

#[test]
fn the_exact_point_is_never_painted() {
    // ⭐The reason the arms hold off the centre at all: the reticle exists to show what is AT the
    // cursor, and a crosshair that covers it answers a different question.
    assert_eq!(reticle_mark(0.0, 0.0, R), ReticleMark::None);
    for d in [0.0_f32, 0.5, 1.0, 1.5, 2.0] {
        assert_eq!(reticle_mark(d, 0.0, R), ReticleMark::None, "dx={d}");
        assert_eq!(reticle_mark(0.0, d, R), ReticleMark::None, "dy={d}");
    }
}

#[test]
fn the_ring_marks_the_point_and_the_arms_reach_out() {
    // The ring sits at radius 5 and is what actually identifies the pixel under test.
    assert_eq!(reticle_mark(5.0, 0.0, R), ReticleMark::Core);
    assert_eq!(reticle_mark(0.0, -5.0, R), ReticleMark::Core);
    // Arms: along each axis, from the gap out to near the rim, in all four directions.
    for along in [9.0_f32, 20.0, 60.0, R - 11.0] {
        for (dx, dy) in [(along, 0.0), (-along, 0.0), (0.0, along), (0.0, -along)] {
            assert_eq!(reticle_mark(dx, dy, R), ReticleMark::Core, "({dx},{dy})");
        }
    }
    // ...and they stop short of the rim, so the mark never collides with the rim stroke.
    assert_eq!(reticle_mark(R - 1.0, 0.0, R), ReticleMark::None);
}

#[test]
fn nothing_is_marked_away_from_the_axes() {
    // A diagonal pixel well off both axes and outside the ring is untouched image.
    for (dx, dy) in [(30.0_f32, 30.0_f32), (-40.0, 25.0), (60.0, -70.0)] {
        assert_eq!(reticle_mark(dx, dy, R), ReticleMark::None, "({dx},{dy})");
    }
}

#[test]
fn every_core_pixel_is_fenced_by_outline() {
    // ⭐**The contrast guarantee, stated as a property.** Core takes one ink and Outline the
    // opposite, so a core run that touched raw image on any side could vanish into content of that
    // ink's colour — which is exactly how the old fixed-colour crosshair disappeared. Walk the
    // whole reticle: no core pixel may be 4-adjacent to an unmarked pixel.
    let n = crate::RETICLE_PX as i32;
    let (mut core, mut outline) = (0usize, 0usize);
    for y in 0..n {
        for x in 0..n {
            let (dx, dy) = ((x - n / 2) as f32, (y - n / 2) as f32);
            match reticle_mark(dx, dy, R) {
                ReticleMark::Core => {
                    core += 1;
                    for (ox, oy) in [(1.0_f32, 0.0_f32), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
                        assert_ne!(
                            reticle_mark(dx + ox, dy + oy, R),
                            ReticleMark::None,
                            "core pixel ({dx},{dy}) touches bare image at ({ox},{oy})"
                        );
                    }
                }
                ReticleMark::Outline => outline += 1,
                ReticleMark::None => {}
            }
        }
    }
    // A sanity floor on both populations: a mark that classified almost nothing would pass every
    // assertion above by vacuum. (Four arms ~93 px long plus a ring, at ~2 px wide.)
    assert!(core > 400, "core pixels: {core}");
    assert!(outline > core / 2, "outline {outline} vs core {core}");
}
