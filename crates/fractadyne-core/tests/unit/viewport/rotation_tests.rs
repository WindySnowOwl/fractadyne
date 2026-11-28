//! The view's rotation (design/view-rotation.md): one mapping between the screen and the plane,
//! `R(-θ)` on a screen offset with +y up, positive turning the picture counterclockwise on the
//! screen; exact at quarter turns; and at 0 every mapping gives the numbers it gave before rotation
//! existed, to the bit.

use super::*;
use crate::reference::sub_f64;

const W: f64 = 1600.0;
const H: f64 = 1000.0;
const ANGLES: [f64; 8] = [0.0, 12.5, 30.0, 45.0, 90.0, 180.0, -135.0, 270.0];

/// A view a little away from home, so offsets are not trivially round numbers.
fn view(deg: f64) -> Viewport {
    let mut vp = Viewport::new(W, H);
    vp.set_center_mag(bf(-0.743_643_887, 128), bf(0.131_825_904, 128), 2.0e3);
    vp.rotation = deg;
    vp
}

/// The plane offset of `(cx, cy)` from the view's center, in pixels.
fn plane_px(vp: &Viewport, c: &(BigFloat, BigFloat)) -> (f64, f64) {
    let upp = vp.units_per_pixel.to_f64();
    let p = vp.precision;
    (sub_f64(&c.0, &vp.center_x, p) / upp, sub_f64(&c.1, &vp.center_y, p) / upp)
}

#[test]
fn quarter_turns_are_exact() {
    assert_eq!(rotation_cos_sin(0.0), (1.0, 0.0));
    assert_eq!(rotation_cos_sin(90.0), (0.0, 1.0));
    assert_eq!(rotation_cos_sin(-90.0), (0.0, -1.0));
    assert_eq!(rotation_cos_sin(180.0), (-1.0, 0.0));
    assert_eq!(rotation_cos_sin(-180.0), (-1.0, 0.0));
    assert_eq!(rotation_cos_sin(450.0), (0.0, 1.0));
    let (c, s) = rotation_cos_sin(30.0);
    assert!((c - 3f64.sqrt() / 2.0).abs() < 1e-15 && (s - 0.5).abs() < 1e-15);
}

/// Positive turns the picture counterclockwise: at +90° the plane's +Re (east of the center) is up
/// on the screen, its +Im (north) to the left.
#[test]
fn positive_turns_the_picture_counterclockwise() {
    let vp = view(90.0);
    let upp = vp.units_per_pixel.to_f64();
    let p = vp.precision;
    let east = (vp.center_x.add(&bf(100.0 * upp, p), p, RM), vp.center_y.clone());
    let north = (vp.center_x.clone(), vp.center_y.add(&bf(100.0 * upp, p), p, RM));
    let (ex, ey) = vp.complex_to_pixel(&east.0, &east.1);
    let (nx, ny) = vp.complex_to_pixel(&north.0, &north.1);
    assert!((ex - W / 2.0).abs() < 1e-6 && (ey - (H / 2.0 - 100.0)).abs() < 1e-6, "east at ({ex}, {ey})");
    assert!((nx - (W / 2.0 - 100.0)).abs() < 1e-6 && (ny - H / 2.0).abs() < 1e-6, "north at ({nx}, {ny})");
    // The screen helpers agree, and invert each other.
    assert_eq!(vp.screen_to_plane(0.0, -100.0), (100.0, 0.0));
    assert_eq!(vp.plane_to_screen(100.0, 0.0), (0.0, -100.0));
}

/// A pixel to the plane and back lands on the same pixel, at any angle.
#[test]
fn a_pixel_round_trips() {
    for deg in ANGLES {
        let vp = view(deg);
        for (px, py) in [(0.0, 0.0), (410.0, 735.0), (W, H), (W * 0.5, H * 0.5), (1599.5, 3.25)] {
            let c = vp.pixel_to_complex(px, py);
            let (bx, by) = vp.complex_to_pixel(&c.0, &c.1);
            assert!((bx - px).abs() < 1e-6 && (by - py).abs() < 1e-6, "{deg}°: ({px}, {py}) came back ({bx}, {by})");
            let (fx, fy) = vp.complex_at_pixel_f64(px, py);
            let (gx, gy) = (to_f64(&c.0), to_f64(&c.1));
            assert!((fx - gx).abs() < 1e-12 && (fy - gy).abs() < 1e-12, "{deg}°: the f64 mapping parts at ({px}, {py})");
        }
    }
}

/// The point under the cursor stays under it through a zoom, and a drag carries the picture with
/// the pointer, at any angle.
#[test]
fn zoom_keeps_the_cursor_point_and_a_drag_follows_the_pointer() {
    for deg in ANGLES {
        let mut vp = view(deg);
        let (px, py) = (410.0, 735.0);
        let under = vp.pixel_to_complex(px, py);
        vp.zoom_at(px, py, 1.0 / 7.0);
        let (bx, by) = vp.complex_to_pixel(&under.0, &under.1);
        assert!((bx - px).abs() < 1e-6 && (by - py).abs() < 1e-6, "{deg}°: zoom moved the cursor's point to ({bx}, {by})");
        let under = vp.pixel_to_complex(px, py);
        vp.pan_pixels(37.0, -12.5);
        let (bx, by) = vp.complex_to_pixel(&under.0, &under.1);
        assert!((bx - (px + 37.0)).abs() < 1e-6 && (by - (py - 12.5)).abs() < 1e-6, "{deg}°: the drag put the point at ({bx}, {by})");
    }
}

/// Unturned, every mapping gives the old formulas' numbers to the bit: center ± offset per axis,
/// +y down on the screen.
#[test]
fn unturned_maps_as_before_to_the_bit() {
    let vp = view(0.0);
    let p = vp.precision;
    let off = |d: f64| vp.units_per_pixel.mul_f64(d).to_bf(p);
    let (px, py) = (410.0, 735.0);
    let (cx, cy) = vp.pixel_to_complex(px, py);
    let ox = off(px - W * 0.5);
    let oy = off(py - H * 0.5);
    assert_eq!(cx, vp.center_x.add(&ox, p, RM));
    assert_eq!(cy, vp.center_y.sub(&oy, p, RM));
    let (fx, fy) = vp.complex_at_pixel_f64(px, py);
    let (c0, c1) = vp.center_f64();
    let upp = vp.units_per_pixel.to_f64();
    assert_eq!((fx.to_bits(), fy.to_bits()), ((c0 + (px - W * 0.5) * upp).to_bits(), (c1 - (py - H * 0.5) * upp).to_bits()));
    let mut a = vp.clone();
    a.pan_pixels(37.0, -12.5);
    assert_eq!(a.center_x, vp.center_x.sub(&off(37.0), p, RM));
    assert_eq!(a.center_y, vp.center_y.add(&off(-12.5), p, RM));
    // A turned view's center is the same point however the picture is turned.
    let d = plane_px(&view(45.0), &view(0.0).pixel_to_complex(W * 0.5, H * 0.5));
    assert_eq!(d, (0.0, 0.0));
}
