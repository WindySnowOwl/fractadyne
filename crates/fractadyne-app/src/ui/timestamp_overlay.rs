//! The elapsed-time overlay: a large clock drawn over the view for live diagnosis.
//!
//! ⭐**It reads the SAME clock the log stamps every line with.** `diag::elapsed_s` is what produces
//! the `[+12.345s]` prefix on every log line, so a screen recording or a phone video of a problem
//! that only appears in motion can be lined up against the log to the frame — which is the entire
//! reason this exists (user request, 2026-09-20, chasing a pan that no still frame shows).
//!
//! The frame counter beside it is the other half of that: between two log lines with the same
//! stamp, `f=` is what tells them apart, and it is what the `tile`/`glide` traces key on.

use eframe::egui;

/// Height of the clock text, in points.
const CLOCK_PT: f32 = 44.0;
/// Height of the frame counter under it, in points.
const FRAME_PT: f32 = 18.0;
/// Inset from the top-left corner of the view, in points.
const INSET: f32 = 14.0;
/// Padding inside the backing plate.
const PAD: egui::Vec2 = egui::vec2(12.0, 7.0);

/// What the overlay says, formatted. Split out so the formatting is testable without a context —
/// the exact text is the contract with the log, not decoration.
pub(crate) fn overlay_text(elapsed_s: f64, frame: u64) -> (String, String) {
    // ⚠`{:.3}` and a leading `+`, to match `diag::stamp`'s `[+{:9.3}s]`. A reader lining a video
    // up against the log compares these two strings by eye; if the precision here were different
    // they would have to do arithmetic on every comparison.
    (format!("+{elapsed_s:.3}s"), format!("frame {frame}"))
}

/// Draw the overlay into `rect` (the view's own rectangle) on the decoration layer — above the
/// fractal, below every dialog, exactly where the tour captions live.
pub(crate) fn draw(ctx: &egui::Context, rect: egui::Rect, elapsed_s: f64, frame: u64) {
    let (clock, frames) = overlay_text(elapsed_s, frame);
    let painter = super::central::decor_painter(ctx);
    // White on a dark plate: the fractal underneath is any colour at all, and a diagnostic that is
    // unreadable over half the palettes is no diagnostic. The plate is translucent so it never
    // hides what is being diagnosed.
    let fg = egui::Color32::WHITE;
    let clock_galley =
        ctx.fonts(|f| f.layout_no_wrap(clock, egui::FontId::monospace(CLOCK_PT), fg));
    let frame_galley = ctx.fonts(|f| {
        f.layout_no_wrap(frames, egui::FontId::monospace(FRAME_PT), egui::Color32::from_white_alpha(190))
    });
    let w = clock_galley.size().x.max(frame_galley.size().x);
    let h = clock_galley.size().y + frame_galley.size().y;
    let origin = rect.min + egui::vec2(INSET, INSET);
    let plate = egui::Rect::from_min_size(origin - PAD, egui::vec2(w, h) + PAD * 2.0);
    painter.rect_filled(plate, 6.0, egui::Color32::from_black_alpha(150));
    painter.galley(origin, clock_galley.clone(), fg);
    painter.galley(origin + egui::vec2(0.0, clock_galley.size().y), frame_galley, fg);
}

#[cfg(test)]
mod tests;
