use super::{dpi_changed, DpiState};

fn st(ppp: f32, w: f32, h: f32) -> DpiState {
    DpiState { ppp, logical: [w, h] }
}

/// A settled window must not emit a line every frame — float jitter in the last bits of
/// `pixels_per_point`, and sub-point wobble as panels settle, are not events.
#[test]
fn jitter_is_not_a_change() {
    let a = st(1.5, 1280.0, 800.0);
    assert!(!dpi_changed(a, a));
    assert!(!dpi_changed(a, st(1.5 + 1e-6, 1280.2, 799.8)));
}

/// The event this channel exists for: a monitor change moves the scale factor by a whole step.
#[test]
fn a_scale_factor_step_is_a_change() {
    for ppp in [1.0, 1.25, 1.75, 2.0] {
        assert!(dpi_changed(st(1.5, 1280.0, 800.0), st(ppp, 1280.0, 800.0)), "ppp {ppp}");
    }
}

/// …and so is a resize at an unchanged scale factor, which is the half that matters here: the
/// field report is a window that GROWS, and it would be invisible if only ppp were watched.
#[test]
fn a_resize_at_the_same_scale_is_a_change() {
    assert!(dpi_changed(st(1.5, 1280.0, 800.0), st(1.5, 1400.0, 800.0)));
    assert!(dpi_changed(st(1.5, 1280.0, 800.0), st(1.5, 1280.0, 900.0)));
}

/// Physical size is what the report describes growing. A drag between monitors is SUPPOSED to
/// hold the logical size and change the physical one, so the trace must show both to tell a
/// correct DPI transition from a runaway one.
#[test]
fn physical_size_tracks_the_scale_factor() {
    assert_eq!(st(1.0, 1280.0, 800.0).physical(), [1280.0, 800.0]);
    assert_eq!(st(1.5, 1280.0, 800.0).physical(), [1920.0, 1200.0]);
    // The healthy transition: logical held, physical scaled by exactly the new factor.
    let (before, after) = (st(1.5, 1280.0, 800.0), st(1.0, 1280.0, 800.0));
    assert!(dpi_changed(before, after));
    assert_eq!(after.physical(), [1280.0, 800.0]);
}
