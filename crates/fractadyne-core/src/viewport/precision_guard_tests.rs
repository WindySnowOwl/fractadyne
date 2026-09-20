//! The centre's precision guard: how much arithmetic headroom a view carries beyond its depth.
//!
//! ⭐A deep view's centre is only useful if it can still address the PIXELS of that view: the
//! difference between neighbouring pixels is `units_per_pixel`, which at 1e150× is ~1e-152, and a
//! centre held to just enough bits for the magnification would round that away. `Viewport` sets
//! `precision_for_octaves(depth) = depth + 64` — about 19 decimal digits of guard past the last
//! bit the depth itself needs — and these pin it, because the guard is invisible until it is gone.

use super::*;
use crate::bignum::{precision_for_octaves, to_decimal_string};

/// Guard bits beyond the depth, as the viewport requests them.
const GUARD_BITS: usize = 64;

#[test]
fn the_precision_carries_at_least_the_guard_past_the_depth() {
    for l2 in [0.0, 10.0, 53.0, 100.0, 372.4, 481.7, 1000.0, 4000.0] {
        let mut vp = Viewport::new(1920.0, 1200.0);
        vp.set_center_log2mag(bf(-0.75, 64), bf(0.1, 64), l2);
        let need = l2.max(0.0).ceil() as usize;
        assert!(
            vp.precision >= need + GUARD_BITS,
            "2^{l2}: precision {} is not {need} + {GUARD_BITS}",
            vp.precision
        );
        // ≥ 5 decimal digits of guard is the floor anyone should rely on; 64 bits is ~19.
        assert!((vp.precision - need) as f64 * std::f64::consts::LOG10_2 >= 5.0);
    }
}

#[test]
fn a_deep_centre_can_still_address_one_pixel() {
    // The guard exists so a single-pixel pan at depth MOVES the centre, and by the pixel's worth.
    for l2 in [100.0, 372.4, 481.7, 1000.0] {
        let mut vp = Viewport::new(1920.0, 1200.0);
        vp.set_center_log2mag(bf(-0.75, 64), bf(0.1, 64), l2);
        let before = to_decimal_string(&vp.center_x);
        vp.pan_pixels(1.0, 0.0);
        let after = to_decimal_string(&vp.center_x);
        assert_ne!(before, after, "2^{l2}: a one-pixel pan left the centre unchanged");
        // And the move is one pixel: (Δcentre)/upp ≈ 1.
        let d = ref_offset_mantissa(&vp.center_x, &bf(-0.75, vp.precision), vp.units_per_pixel.e, vp.precision)
            / vp.units_per_pixel.m;
        assert!((d.abs() - 1.0).abs() < 1e-6, "2^{l2}: one pixel moved {d} pixels");
    }
}

#[test]
fn the_view_bounds_stay_exact_at_depth() {
    // The corners of the bounding box, read back through the inverse mapping, land on the corners.
    for l2 in [100.0, 481.7, 2000.0] {
        let mut vp = Viewport::new(1920.0, 1200.0);
        vp.set_center_log2mag(bf(0.25, 64), bf(-0.5, 64), l2);
        for (px, py) in [(0.0, 0.0), (vp.width_px, 0.0), (0.0, vp.height_px), (vp.width_px, vp.height_px)] {
            let (cx, cy) = vp.pixel_to_complex(px, py);
            let (bx, by) = vp.complex_to_pixel(&cx, &cy);
            assert!((bx - px).abs() < 1e-6 && (by - py).abs() < 1e-6, "2^{l2}: corner ({px},{py}) → ({bx},{by})");
        }
    }
}

#[test]
fn precision_for_octaves_is_monotone_and_guarded() {
    let mut prev = 0;
    for oct in [0u64, 1, 64, 1000, 100_000, 3_320_000] {
        let p = precision_for_octaves(oct);
        assert!(p >= oct as usize + GUARD_BITS);
        assert!(p >= prev);
        prev = p;
    }
}
