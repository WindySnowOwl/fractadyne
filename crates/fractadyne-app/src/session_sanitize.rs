//! The saved session, held to the ranges the app itself can set, before anything reads it.
//!
//! ⭐**The session file is input like any other.** A view file's every number is clamped or
//! refused (`load_view_metadata`), but the session restore took most of its numbers as written:
//! a hand edit, a damaged file or a value written by a bug came straight back on every launch.
//! Review 2026-11-16 (finding 9): `aa = 1000` would supersample a million samples per pixel on the
//! first frame, and a GPU-loss restart resets only the VIEW, so it would come back again; a NaN
//! Julia constant (finding 6), once loaded from a view, was saved and restored forever.
//!
//! Every value here is clamped to what the control that sets it allows (or the view reader, where
//! it is more generous), and a value that is not a finite number goes back to its default. What was
//! changed is named, for the log.

use fractadyne_state::SessionState;

/// Clamp `s` into the ranges the app can set, putting back to its default anything that is not a
/// finite number. Returns the settings it had to change.
pub(crate) fn sanitize(s: &mut SessionState) -> Vec<&'static str> {
    let d = SessionState::default();
    let mut fixed: Vec<&'static str> = Vec::new();

    fn real64(v: &mut f64, lo: f64, hi: f64, default: f64, name: &'static str, fixed: &mut Vec<&'static str>) {
        let c = if v.is_finite() { v.clamp(lo, hi) } else { default };
        if c.to_bits() != v.to_bits() {
            *v = c;
            fixed.push(name);
        }
    }
    fn real32(v: &mut f32, lo: f32, hi: f32, default: f32, name: &'static str, fixed: &mut Vec<&'static str>) {
        let c = if v.is_finite() { v.clamp(lo, hi) } else { default };
        if c.to_bits() != v.to_bits() {
            *v = c;
            fixed.push(name);
        }
    }
    fn count(v: &mut u32, lo: u32, hi: u32, name: &'static str, fixed: &mut Vec<&'static str>) {
        let c = (*v).clamp(lo, hi);
        if c != *v {
            *v = c;
            fixed.push(name);
        }
    }

    // The view: the Julia constant (a view's reader now refuses a non-finite one too).
    if !(s.julia_c_re.is_finite() && s.julia_c_im.is_finite()) {
        (s.julia_c_re, s.julia_c_im) = (d.julia_c_re, d.julia_c_im);
        fixed.push("julia_c");
    }
    real32(&mut s.dual_split, crate::DUAL_SPLIT_MIN, crate::DUAL_SPLIT_MAX, d.dual_split, "dual_split", &mut fixed);
    // The rotations: a number is put in (−180, 180] where it is read; anything else is unturned.
    for (v, name) in [(&mut s.rotation, "rotation"), (&mut s.julia_view_rotation, "julia_view_rotation")] {
        if !v.is_finite() {
            *v = 0.0;
            fixed.push(name);
        }
    }
    if let Some(l) = s.julia_view_upp_log2 {
        if !l.is_finite() || l.abs() > crate::export::MAX_LOAD_OCTAVES {
            s.julia_view_upp_log2 = None;
            fixed.push("julia_view_upp_log2");
        }
    }
    // Iterations: the slider's and an imported location's ceiling.
    count(&mut s.max_iter, 1, crate::MAX_ITER_LIMIT, "max_iter", &mut fixed);

    // The coloring, by the view reader's own table (its ranges and choices), and the two settings the
    // view reader holds outside it.
    crate::coloring_fields::sanitize(&mut s.coloring, &mut fixed);
    real32(&mut s.coloring.cycle, 0.0, 1.0e6, d.coloring.cycle, "cycle", &mut fixed);
    real32(&mut s.coloring.offset, -1.0e6, 1.0e6, d.coloring.offset, "offset", &mut fixed);
    count(&mut s.coloring.iter_div, 1, fractadyne_state::ITER_DIV_MAX, "iter_div", &mut fixed);

    // Rendering: supersampling as a view file allows it, the frame cap (0 = uncapped).
    count(&mut s.aa, 1, 16, "aa", &mut fixed);
    if !(s.fps_cap.is_finite() && s.fps_cap >= 0.0) {
        s.fps_cap = d.fps_cap;
        fixed.push("fps_cap");
    } else if s.fps_cap > 0.0 {
        real64(&mut s.fps_cap, 1.0, 1000.0, d.fps_cap, "fps_cap", &mut fixed);
    }
    real64(&mut s.work_budget_scale, 0.25, 8.0, d.work_budget_scale, "work_budget_scale", &mut fixed);
    real32(&mut s.min_motion_res, 0.30, 1.0, d.min_motion_res, "min_motion_res", &mut fixed);

    // Navigation: the sliders' ranges.
    real32(&mut s.zoom_rate, 0.25, 4.0, d.zoom_rate, "zoom_rate", &mut fixed);
    real32(&mut s.click_zoom_factor, 1.5, 1000.0, d.click_zoom_factor, "click_zoom_factor", &mut fixed);
    real64(&mut s.autopilot_dive_log2, 1.0, crate::export::MAX_LOAD_OCTAVES, d.autopilot_dive_log2, "autopilot_dive_log2", &mut fixed);

    // Animation speeds: the sliders' ranges.
    real32(&mut s.palette_anim_speed, 0.01, 2.0, d.palette_anim_speed, "palette_anim_speed", &mut fixed);
    real32(&mut s.orbit_anim_speed, 1.0, 40.0, d.orbit_anim_speed, "orbit_anim_speed", &mut fixed);

    // Export: the dialog's width range and the command line's supersampling range.
    count(&mut s.export_width, 16, 32_768, "export_width", &mut fixed);
    count(&mut s.export_ss, 1, 8, "export_ss", &mut fixed);

    // The interface.
    real32(&mut s.ui_scale, 0.6, 2.5, d.ui_scale, "ui_scale", &mut fixed);
    real32(&mut s.live_panel_opacity, 0.3, 1.0, d.live_panel_opacity, "live_panel_opacity", &mut fixed);

    fixed
}

#[cfg(test)]
#[path = "../tests/unit/session_sanitize/tests.rs"]
mod tests;
