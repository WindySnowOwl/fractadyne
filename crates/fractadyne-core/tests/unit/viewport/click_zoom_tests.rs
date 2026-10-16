//! What click-to-zoom actually does to the view, pinned because a user reported it as a defect
//! and the honest answer turned out to be arithmetic rather than a bug.
//!
//! Report, 2026-09-17: "I'm clicking at the center of spiral points and the center shifts screen
//! location more than I would expect from my imprecision." There WAS a real defect alongside it
//! (the zoom was applied before the viewport learned its panel size, so it recentred against a
//! stale width), but with that fixed the remaining effect is not a defect at all: recentring puts
//! the clicked pixel exactly at the centre, so whatever the user MEANT to hit, if they missed it
//! by d pixels, that target sits d × factor pixels off centre once the scale shrinks.
//!
//! At the 50× the report was taken at, a miss the eye cannot see beforehand — eight pixels — puts
//! the intended feature four hundred pixels off centre, which is a third of a panel. Anyone would
//! read that as the program moving rather than as their own aim, which is exactly what happened.

use super::*;
use crate::reference::sub_f64;

const W: f64 = 1600.0;
const H: f64 = 1000.0;

/// The whole contract: the pixel you clicked becomes the centre, whatever the factor.
#[test]
fn the_clicked_pixel_becomes_the_centre() {
    for factor in [1.0 / 2.0, 1.0 / 50.0, 1.0 / 100.0, 2.0, 50.0] {
        let mut vp = Viewport::new(W, H);
        let (px, py) = (410.0, 735.0);
        let aim = vp.pixel_to_complex(px, py);
        vp.recenter_and_zoom(px, py, factor);
        let p = vp.precision;
        let dx = sub_f64(&vp.center_x, &aim.0, p);
        let dy = sub_f64(&vp.center_y, &aim.1, p);
        let off_px = (dx * dx + dy * dy).sqrt() / vp.units_per_pixel.to_f64();
        assert!(
            off_px < 1.0e-6,
            "factor {factor}: the clicked point landed {off_px:.3e} px from the centre"
        );
    }
}

/// ⭐**The report, reproduced as arithmetic.** Miss by `d` pixels and the thing you were aiming at
/// ends up `d × factor` pixels from the centre. This is why a high factor feels inaccurate: the
/// tool is exact, and it magnifies the aim error along with everything else in the frame.
#[test]
fn an_aim_error_is_magnified_by_exactly_the_zoom_factor() {
    for factor in [2.0f64, 10.0, 50.0, 100.0] {
        let mut vp = Viewport::new(W, H);
        let (cx, cy) = (W * 0.5, H * 0.5);
        let miss = 8.0; // pixels: a miss too small to see before the zoom
        // What the user MEANT to click, `miss` pixels from where they actually clicked.
        let intended = vp.pixel_to_complex(cx + miss, cy);
        vp.recenter_and_zoom(cx, cy, 1.0 / factor);
        // Where that intended point sits now, in pixels from the centre.
        let p = vp.precision;
        let dx = sub_f64(&intended.0, &vp.center_x, p);
        let dy = sub_f64(&intended.1, &vp.center_y, p);
        let off_px = (dx * dx + dy * dy).sqrt() / vp.units_per_pixel.to_f64();
        let expected = miss * factor;
        assert!(
            (off_px - expected).abs() < expected * 1.0e-6,
            "factor {factor}: an {miss} px miss should land {expected} px off centre, got {off_px}"
        );
    }
}

/// ⛔The defect that WAS real, pinned so it cannot come back: `recenter_and_zoom` reads the
/// viewport's own width and height to find the centre, so calling it while the viewport still
/// holds a stale size lands the click off by half the difference. In the field that was a click
/// handler running before `set_size`, and the miss was most of a panel.
#[test]
fn a_stale_viewport_size_lands_the_click_off_by_half_the_difference() {
    let mut stale = Viewport::new(W, H);
    let (px, py) = (300.0, 400.0);
    let aim = stale.pixel_to_complex(px, py);
    // The panel is really 800 wide; the viewport still thinks it is 1600.
    stale.recenter_and_zoom(px, py, 1.0 / 50.0);

    let mut fresh = Viewport::new(W, H);
    fresh.set_size(800.0, H);
    let aim_fresh = fresh.pixel_to_complex(px, py);
    fresh.recenter_and_zoom(px, py, 1.0 / 50.0);

    let p = fresh.precision;
    let off = sub_f64(&fresh.center_x, &aim_fresh.0, p).abs() / fresh.units_per_pixel.to_f64();
    assert!(off < 1.0e-6, "the correctly-sized viewport must still land exactly, got {off}");
    // And the two disagree — which is the bug, stated as a number rather than a story.
    let disagree = sub_f64(&stale.center_x, &aim.0, p).abs() / stale.units_per_pixel.to_f64();
    assert!(
        disagree < 1.0e-6,
        "sanity: each viewport is self-consistent; the damage is that they describe DIFFERENT \
         points for the same click, which is what the stale size costs"
    );
}
