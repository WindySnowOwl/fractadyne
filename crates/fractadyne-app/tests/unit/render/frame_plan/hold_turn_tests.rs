//! A held frame shown under a turned view (design/view-rotation.md, R3). The color pass's
//! reprojection (`fs_color`, mirrored here: change one with the other) must put on each pixel of
//! the view the held frame's texel that shows the same plane point, as the two viewports' own
//! mappings say: the held frame's angle, depth and center all differing from the view's.

use super::{hold_transform, hold_turn, plane_spans_on_screen};
use fractadyne_core::{BigFloat, Viewport};

const W: f64 = 1600.0;
const H: f64 = 1000.0;

fn view(cx: f64, cy: f64, l2: f64, deg: f64) -> Viewport {
    let mut vp = Viewport::new(W, H);
    vp.set_center_log2mag(BigFloat::from_f64(cx, 128), BigFloat::from_f64(cy, 128), l2);
    vp.rotation = deg;
    vp
}

/// `fs_color`'s reprojection of the output pixel `(px, py)` (fit 1): the held frame's uv.
fn shader_uv(px: f64, py: f64, uv_off: [f32; 2], scale: f32, turn: [f32; 2]) -> (f64, f64) {
    let (mut sx, mut sy) = (px / W, py / H);
    if turn != [0.0, 0.0] {
        let (c, s) = (f64::from(turn[0]), f64::from(turn[1]));
        let (dx, dy) = ((sx - 0.5) * W, (sy - 0.5) * H);
        sx = (c * dx + s * dy) / W + 0.5;
        sy = (c * dy - s * dx) / H + 0.5;
    }
    let k = f64::from(scale);
    ((sx - 0.5) * k + 0.5 - f64::from(uv_off[0]), (sy - 0.5) * k + 0.5 - f64::from(uv_off[1]))
}

/// The app's transform (`hold_transform_for`): the view's center from the held frame's, in spans of
/// the view now along the plane's axes, turned into the held frame's screen axes; the turn between.
fn transform(held: &Viewport, now: &Viewport) -> ([f32; 3], [f32; 2]) {
    let (hc, nc) = (held.center_f64(), now.center_f64());
    let (sx, sy) = now.complex_span();
    let off = ((nc.0 - hc.0) / sx, (nc.1 - hc.1) / sy);
    let off = plane_spans_on_screen(off, held.rotation, H / W);
    let (t, _) = hold_transform(held.log2_magnification() - now.log2_magnification(), false, false, off);
    (t, hold_turn(held.rotation, now.rotation))
}

#[test]
fn a_held_frame_at_another_angle_shows_each_point_where_the_view_has_it() {
    let cases = [
        // (held angle, view angle, held depth, view depth, view center offset in pixels of the view)
        (0.0, 0.0, 10.0, 10.3, (40.0, -25.0)),
        (0.0, 30.0, 10.0, 10.0, (0.0, 0.0)),
        (0.0, 90.0, 10.0, 10.4, (12.0, 30.0)),
        (30.0, 0.0, 10.2, 10.0, (-50.0, 18.0)),
        (-75.0, 41.5, 10.0, 10.6, (33.0, 61.0)),
        (180.0, -90.0, 9.8, 10.0, (-7.0, -90.0)),
    ];
    for (hr, vr, hl, vl, (ox, oy)) in cases {
        let held = view(-0.743, 0.131, hl, hr);
        let mut now = view(-0.743, 0.131, vl, vr);
        now.pan_pixels(-ox, -oy); // the view's center moved by (ox, oy) of its pixels
        let ([ux, uy, scale], turn) = transform(&held, &now);
        assert_eq!(turn == [0.0, 0.0], hr == vr, "{hr}/{vr}: a turn exactly when the angles differ");
        for (px, py) in [(800.5, 500.5), (0.5, 0.5), (1599.5, 999.5), (210.5, 860.5), (1400.5, 120.5)] {
            let (gx, gy) = shader_uv(px, py, [ux, uy], scale, turn);
            let c = now.pixel_to_complex(px, py);
            let (hx, hy) = held.complex_to_pixel(&c.0, &c.1);
            let (ex, ey) = (gx * W - hx, gy * H - hy);
            assert!(ex.abs() < 2e-3 && ey.abs() < 2e-3, "held {hr} at 2^{hl}, view {vr} at 2^{vl}: pixel ({px}, {py}) sampled the held frame {ex:+.4}, {ey:+.4} px off");
        }
    }
}

/// The same angle, however written, is no turn; a quarter turn is exact.
#[test]
fn the_turn_is_none_for_one_angle_and_exact_at_a_quarter() {
    assert_eq!(hold_turn(30.0, 30.0), [0.0, 0.0]);
    assert_eq!(hold_turn(0.0, 90.0), [0.0, -1.0]);
    assert_eq!(hold_turn(90.0, 0.0), [0.0, 1.0]);
}
