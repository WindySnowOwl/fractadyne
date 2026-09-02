use super::*;

/// A deep centre carries far more information than an `f64` can hold. `pan_complex` exists
/// so the overview map can move such a view WITHOUT going through one: it adds the delta to
/// the existing coordinate at full precision. The obvious alternative - convert the map
/// position to `f64` and assign it as the new centre - looks equivalent and silently
/// truncates the centre to about 17 digits, which at depth is the whole coordinate.
///
/// Pinned as a round trip: pan out and back, and the original bits must return.
#[test]
fn a_round_trip_pan_returns_the_original_deep_centre() {
    let digits_x = "0.35634774601304382214593134944855658665333542382319826904819524052878";
    let digits_y = "0.65517219785957047867473526044384060240158237433104919183695119307267";
    let cx = crate::parse_bf_prec(digits_x, 512).expect("cx parses");
    let cy = crate::parse_bf_prec(digits_y, 512).expect("cy parses");

    let mut vp = Viewport::new(1920.0, 1080.0);
    vp.set_center_log2mag(cx.clone(), cy.clone(), 300.0); // ~1e90x

    vp.pan_complex(0.25, -0.125);
    vp.pan_complex(-0.25, 0.125);

    let p = vp.precision;
    let dx = crate::to_f64(&vp.center_x.sub(&cx, p, RM)).abs();
    let dy = crate::to_f64(&vp.center_y.sub(&cy, p, RM)).abs();
    assert!(dx < 1.0e-40, "x drifted by {dx:e} over a round-trip pan");
    assert!(dy < 1.0e-40, "y drifted by {dy:e} over a round-trip pan");

    // And the reason the threshold is meaningful: an f64 round trip of the same coordinate
    // loses everything below ~1e-17, so this test would fail by twenty-odd orders against
    // the assign-from-f64 approach rather than passing by luck.
    let via_f64 = BigFloat::from_f64(crate::to_f64(&cx), p);
    let lost = crate::to_f64(&via_f64.sub(&cx, p, RM)).abs();
    assert!(lost > 1.0e-25, "an f64 round trip lost only {lost:e} - threshold is not probative");
}

/// Checklist step 19, "the image follows the cursor 1:1 while dragging". The claim is exact:
/// a drag of N pixels must move the centre by N * units_per_pixel, at any depth. Checked
/// across 15 orders of magnitude because a scale factor that is subtly wrong is invisible at
/// one depth and obvious at another.
#[test]
fn pan_pixels_moves_exactly_one_pixel_per_pixel() {
    for log2mag in [0.0f64, 20.0, 60.0, 200.0] {
        let mut vp = Viewport::new(1920.0, 1080.0);
        vp.set_center_log2mag(
            crate::BigFloat::from_f64(-0.5, 64),
            crate::BigFloat::from_f64(0.0, 64),
            log2mag,
        );
        // Independent of pan_pixels' own arithmetic: take the point under a pixel, drag,
        // and ask where that point now sits. 1:1 means it moved by exactly the drag.
        // Comparing against `pixel_offset` instead would only test pan_pixels against its
        // own helper, and a wrong scale would agree with itself.
        let (px, py) = (700.0f64, 400.0);
        let (cx, cy) = vp.pixel_to_complex(px, py);

        vp.pan_pixels(120.0, -80.0);

        let (bx, by) = vp.complex_to_pixel(&cx, &cy);
        assert!(
            (bx - (px + 120.0)).abs() < 0.01 && (by - (py - 80.0)).abs() < 0.01,
            "at 2^{log2mag}: the point moved to ({bx:.3}, {by:.3}), wanted ({}, {})",
            px + 120.0,
            py - 80.0
        );
    }
}

/// Checklist step 20, "wheel zooms about the cursor". The invariant is that the complex point
/// under the cursor does not move: zoom about a pixel, and that pixel must still show the same
/// point. Uses the pixel round-trip rather than comparing coordinates, so it holds at depths
/// where the coordinate itself is far past f64.
#[test]
fn zoom_at_keeps_the_point_under_the_cursor_fixed() {
    for factor in [0.25f64, 0.5, 2.0, 4.0] {
        let mut vp = Viewport::new(1920.0, 1080.0);
        vp.set_center_log2mag(
            crate::BigFloat::from_f64(-0.5, 64),
            crate::BigFloat::from_f64(0.0, 64),
            40.0,
        );
        // Deliberately NOT the centre: zooming about the centre keeps everything fixed, so a
        // centred cursor would pass even if the anchoring were ignored entirely.
        let (px, py) = (1500.0f64, 300.0);
        let (cx, cy) = vp.pixel_to_complex(px, py);

        vp.zoom_at(px, py, factor);

        let (bx, by) = vp.complex_to_pixel(&cx, &cy);
        assert!(
            (bx - px).abs() < 0.01 && (by - py).abs() < 0.01,
            "factor {factor}: the anchored point moved to ({bx:.3}, {by:.3}), wanted ({px}, {py})"
        );
    }
}

/// Checklist step 65, "iteration count starts tracking zoom depth again". Depth-adaptive means
/// monotonic: deeper must never ask for FEWER iterations than shallower.
#[test]
fn recommended_max_iter_never_decreases_with_depth() {
    let mut prev = 0u32;
    for log2mag in [0.0f64, 10.0, 30.0, 60.0, 120.0, 300.0] {
        let mut vp = Viewport::new(1920.0, 1080.0);
        vp.set_center_log2mag(
            crate::BigFloat::from_f64(-0.5, 64),
            crate::BigFloat::from_f64(0.0, 64),
            log2mag,
        );
        let n = vp.recommended_max_iter(1000);
        assert!(n >= prev, "2^{log2mag} asked for {n} after {prev} at the shallower depth");
        prev = n;
    }
    // And it must actually CLIMB, not merely fail to fall - a constant would satisfy
    // monotonicity while making "auto-scale" do nothing.
    let mut deep = Viewport::new(1920.0, 1080.0);
    deep.set_center_log2mag(
        crate::BigFloat::from_f64(-0.5, 64),
        crate::BigFloat::from_f64(0.0, 64),
        300.0,
    );
    let mut shallow = Viewport::new(1920.0, 1080.0);
    shallow.set_center_log2mag(
        crate::BigFloat::from_f64(-0.5, 64),
        crate::BigFloat::from_f64(0.0, 64),
        0.0,
    );
    assert!(
        deep.recommended_max_iter(1000) > shallow.recommended_max_iter(1000) * 2,
        "auto-scale should ask for far more at 2^300 than at 1x"
    );
}

/// `zoom_by` must leave the centre exactly where it was, at any depth - it is the
/// gesture for "deeper from here", so any drift is the bug.
#[test]
fn zoom_by_holds_the_centre_and_scales_the_span() {
    let digits = "0.35634774601304382214593134944855658665333542382319826904819524052878";
    let cx = crate::parse_bf_prec(digits, 512).expect("parses");
    let cy = crate::parse_bf_prec(digits, 512).expect("parses");
    let mut vp = Viewport::new(1920.0, 1080.0);
    vp.set_center_log2mag(cx.clone(), cy.clone(), 300.0);
    let before = vp.log2_magnification();

    vp.zoom_by(0.5); // factor < 1 zooms IN
    assert!(
        (vp.log2_magnification() - (before + 1.0)).abs() < 1.0e-9,
        "halving units-per-pixel should add one octave: {} -> {}",
        before,
        vp.log2_magnification()
    );
    let p = vp.precision;
    let dx = crate::to_f64(&vp.center_x.sub(&cx, p, RM)).abs();
    let dy = crate::to_f64(&vp.center_y.sub(&cy, p, RM)).abs();
    assert!(dx < 1.0e-40 && dy < 1.0e-40, "centre drifted: {dx:e}, {dy:e}");
}

/// Checklist step 22, "the Home button ends at the standard home view". The animation is
/// `home_lerp` driven from the starting magnification down to 0, so what this pins is the
/// LANDING: at `new_logmag == 0` the view must be exactly home — 1x, on the requested centre —
/// from any starting depth, with nothing left of the deep coordinate it flew out of.
///
/// Checked from several depths because the centre glide is a fraction of the remaining
/// magnification (`1 - e^-logmag`), so an error in it shrinks with depth and would be
/// invisible if only one shallow start were tried.
#[test]
fn home_view_is_the_default() {
    for (log2mag, home) in [(20.0f64, (-0.5f64, 0.0f64)), (120.0, (-0.5, 0.0)), (400.0, (0.0, 0.0))] {
        let mut vp = Viewport::new(1920.0, 1080.0);
        let digits = "0.35634774601304382214593134944855658665333542382319826904819524052878";
        let cx = crate::parse_bf_prec(digits, 512).expect("parses");
        let cy = crate::parse_bf_prec(digits, 512).expect("parses");
        vp.set_center_log2mag(cx, cy, log2mag);
        let start = (vp.center_x.clone(), vp.center_y.clone());

        // Mid-flight it must be somewhere else entirely - otherwise "lands at home" would
        // also be satisfied by an animation that never moved.
        vp.home_lerp(home, &start, log2mag * 0.5 * std::f64::consts::LN_2);
        assert!(
            vp.log2_magnification() > 1.0,
            "2^{log2mag}: mid-flight is already home"
        );

        vp.home_lerp(home, &start, 0.0);
        let (x, y) = vp.center_f64();
        assert!(
            (x - home.0).abs() < 1.0e-12 && (y - home.1).abs() < 1.0e-12,
            "2^{log2mag}: landed at ({x}, {y}), wanted {home:?}"
        );
        assert!(
            (vp.magnification() - 1.0).abs() < 1.0e-9,
            "2^{log2mag}: landed at {}x, wanted 1x",
            vp.magnification()
        );
    }
}

/// Checklist step 23, "Reset to default view returns to the default view for the current
/// fractal". `reset_to` is what the menu item applies to each panel; from any depth it must
/// restore the whole framing - magnification, centre AND precision. A reset that left the
/// precision at its deep value would look right and carry hundreds of bits of working
/// precision through every subsequent frame.
#[test]
fn reset_view_is_the_default() {
    for (w, h) in [(1920.0f64, 1080.0f64), (640.0, 480.0), (300.0, 1000.0)] {
        let fresh = Viewport::new(w, h);
        let mut vp = Viewport::new(w, h);
        let digits = "0.35634774601304382214593134944855658665333542382319826904819524052878";
        vp.set_center_log2mag(
            crate::parse_bf_prec(digits, 512).expect("parses"),
            crate::parse_bf_prec(digits, 512).expect("parses"),
            300.0,
        );
        assert!(vp.precision > 64, "the deep view under test is not actually deep");

        // Each formula has its own default centre; the Mandelbrot's is the fresh viewport's.
        vp.reset_to(-0.5, 0.0);
        let (x, y) = vp.center_f64();
        assert!((x - -0.5).abs() < 1.0e-15 && y.abs() < 1.0e-15, "centre ({x}, {y})");
        assert_eq!(vp.precision, fresh.precision, "{w}x{h}: precision not reset");
        assert!(
            (vp.magnification() - 1.0).abs() < 1.0e-9,
            "{w}x{h}: {}x after reset",
            vp.magnification()
        );
        assert_eq!(
            vp.units_per_pixel.to_f64(),
            fresh.units_per_pixel.to_f64(),
            "{w}x{h}: framing differs from a fresh viewport of the same size"
        );

        // A non-default centre is honoured (Burning Ship and friends do not sit at -0.5).
        vp.reset_to(-1.75, 0.03);
        let (x, y) = vp.center_f64();
        assert!((x - -1.75).abs() < 1.0e-15 && (y - 0.03).abs() < 1.0e-15, "centre ({x}, {y})");
    }
}

/// Checklist steps 16-17, "a click zooms in on the clicked point by the selected Factor".
/// Two claims, both exact: the magnification multiplies by the factor, and the clicked point
/// becomes the view centre. Run at depth as well as at home because the recentre is a bignum
/// pan and the zoom is a `FloatExp` scale - a factor applied to the wrong one of the two
/// would still look plausible shallow.
///
/// ⚠This is the ACTION, not the gesture: that a left-click reaches it is a human check.
#[test]
fn click_zoom_applies_factor() {
    // The list the tool offers (`CLICK_ZOOM_FACTORS`), plus the right-click zoom-OUT of each.
    for f in [2.0f64, 4.0, 10.0, 50.0, 100.0] {
        for (label, factor, gain) in [("in", 1.0 / f, f), ("out", f, 1.0 / f)] {
            for log2mag in [0.0f64, 40.0, 200.0] {
                let mut vp = Viewport::new(1600.0, 900.0);
                vp.set_center_log2mag(
                    crate::BigFloat::from_f64(-0.5, 64),
                    crate::BigFloat::from_f64(0.0, 64),
                    log2mag,
                );
                // Deliberately off-centre: clicking the centre would pass even if the
                // recentre were skipped entirely.
                let (px, py) = (1180.0f64, 260.0);
                let (tx, ty) = vp.pixel_to_complex(px, py);
                let before = vp.log2_magnification();

                vp.recenter_and_zoom(px, py, factor);

                assert!(
                    (vp.log2_magnification() - (before + gain.log2())).abs() < 1.0e-9,
                    "{f}x {label} at 2^{log2mag}: magnification went 2^{before} -> 2^{}",
                    vp.log2_magnification()
                );
                // The clicked point is now the centre - checked in PIXELS, so it holds at
                // depths where the coordinate itself is far past f64.
                let (bx, by) = vp.complex_to_pixel(&tx, &ty);
                assert!(
                    (bx - 800.0).abs() < 0.01 && (by - 450.0).abs() < 0.01,
                    "{f}x {label} at 2^{log2mag}: clicked point sits at ({bx:.3}, {by:.3}),                          wanted the centre (800, 450)"
                );
            }
        }
    }
}

/// Direction and magnitude: the delta lands on the centre as given, in complex units.
#[test]
fn pan_complex_moves_the_centre_by_exactly_the_delta() {
    let mut vp = Viewport::new(1920.0, 1080.0);
    vp.set_center_mag(BigFloat::from_f64(-0.5, 64), BigFloat::from_f64(0.25, 64), 1.0);
    vp.pan_complex(0.125, -0.0625);
    let (x, y) = vp.center_f64();
    assert!((x - -0.375).abs() < 1.0e-12, "x = {x}");
    assert!((y - 0.1875).abs() < 1.0e-12, "y = {y}");
}
