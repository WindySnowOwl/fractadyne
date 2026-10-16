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

#[test]
fn zooming_out_and_back_in_about_the_centre_keeps_a_deep_centre_exactly() {
    // The (49,3) hero point to 120 digits, at 2^300: out to 1x and back, in glide-sized steps.
    let x = crate::parse_bf_prec("-0.0887926133030986601538450523097015005697106891253792690956998351285032693708137349015830312735269435862157790706524", 512).unwrap();
    let y = crate::parse_bf_prec("0.654809144755247929391298652387765097829500298148873521264227891688595897967629224347883397141609121876708356441134", 512).unwrap();
    let mut vp = Viewport::new(1280.0, 800.0);
    vp.set_center_log2mag(x.clone(), y.clone(), 300.0);
    let step = 2f64.powf(0.05); // ~a frame of a 1x glide
    while vp.log2_magnification() > 0.0 {
        vp.zoom_by(step);
    }
    while vp.log2_magnification() < 300.0 {
        vp.zoom_by(1.0 / step);
    }
    let p = 512;
    let dx = vp.center_x.sub(&x, p, crate::bignum::RM);
    let dy = vp.center_y.sub(&y, p, crate::bignum::RM);
    // (Zero is exact; otherwise the exponent: |d| < 2^e.)
    let off = |d: &BigFloat| if d.is_zero() { i64::MIN } else { d.exponent().map_or(i64::MAX, |e| e as i64) };
    // Off by less than a hundredth of a pixel at 2^300 (a pixel there is ~2^-310): it was ~1e-20 off.
    assert!(off(&dx) < -320 && off(&dy) < -320, "drifted: 2^{} / 2^{}", off(&dx), off(&dy));
}
