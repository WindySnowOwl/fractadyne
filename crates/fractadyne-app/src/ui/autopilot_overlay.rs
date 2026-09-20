//! The auto-zoom target overlay: the region of the view the dive is zooming into, outlined, with
//! its corners joined to the corners of the screen — so where the camera is heading can be seen
//! before it gets there (View ▸ Show auto-zoom target; user request 2026-09-20, after the dive's
//! motion had become hard to read at extreme depth).
//!
//! ⭐**The box is the geometry, not the intent.** The dive zooms about the AIM, so the region that
//! will fill the screen after `ZOOM_TARGET_FACTOR`× more magnification is the box around the aim
//! that `target_rect` computes — that is literally what is being zoomed to. The GOAL (the point on
//! the fractal the steering is easing the aim toward) is marked separately; while the two differ
//! the dive is still turning, and the box shows where it would land if it stopped turning now.

use eframe::egui;

/// How much further the dive is shown zooming: the box is the region that fills the screen after
/// this much more magnification. 4× is two octaves — a couple of seconds at the default rate,
/// far enough ahead to read as "over there", close enough that the box is a useful size.
pub(crate) const ZOOM_TARGET_FACTOR: f64 = 4.0;

/// The screen-fraction rectangle that fills the view after zooming about `aim` (screen fractions)
/// by `factor`: `(min, max)`. Zooming about a point keeps that point where it is and scales every
/// distance from it, so the region that maps onto the whole screen is the screen shrunk toward
/// the aim — `min = aim·(1 − 1/f)`, `size = 1/f`. Always inside the screen for an aim inside it.
pub(crate) fn target_rect(aim: (f64, f64), factor: f64) -> ((f64, f64), (f64, f64)) {
    let f = if factor.is_finite() && factor > 1.0 { factor } else { 1.0 };
    let ax = if aim.0.is_finite() { aim.0.clamp(0.0, 1.0) } else { 0.5 };
    let ay = if aim.1.is_finite() { aim.1.clamp(0.0, 1.0) } else { 0.5 };
    let inv = 1.0 / f;
    let min = (ax * (1.0 - inv), ay * (1.0 - inv));
    (min, (min.0 + inv, min.1 + inv))
}

/// Draw the overlay into `rect` (the view's own rectangle) on the decoration layer — above the
/// fractal, below every dialog. Each line is drawn twice, a dark wide stroke under a light thin
/// one, so it reads over any palette.
pub(crate) fn draw(ctx: &egui::Context, rect: egui::Rect, aim: (f64, f64), goal: Option<(f64, f64)>) {
    let painter = super::central::decor_painter(ctx);
    let at = |p: (f64, f64)| egui::pos2(rect.min.x + p.0 as f32 * rect.width(), rect.min.y + p.1 as f32 * rect.height());
    let (bmin, bmax) = target_rect(aim, ZOOM_TARGET_FACTOR);
    let inner = [at(bmin), at((bmax.0, bmin.1)), at(bmax), at((bmin.0, bmax.1))];
    let outer = [rect.left_top(), rect.right_top(), rect.right_bottom(), rect.left_bottom()];
    let shadow = egui::Stroke::new(3.0_f32, egui::Color32::from_black_alpha(140));
    let line = egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 220));
    for stroke in [shadow, line] {
        for i in 0..4 {
            painter.line_segment([inner[i], outer[i]], stroke); // corner to corner
            painter.line_segment([inner[i], inner[(i + 1) % 4]], stroke); // the box
        }
    }
    // The goal: a small ring, so the point the steering is easing toward is visible while it
    // still differs from where the box sits.
    if let Some(g) = goal {
        let c = at(g);
        painter.circle_stroke(c, 7.0_f32, shadow);
        painter.circle_stroke(c, 7.0_f32, egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(90, 220, 255)));
    }
}

#[cfg(test)]
mod tests;
