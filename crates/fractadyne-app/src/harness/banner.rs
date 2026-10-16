//! The LIVE TEST banner a harness-driven window wears: whether it is on, and its painter.
//! Moved from `main.rs` (the policy) and `ui/menus.rs` (the painter).

use eframe::egui;

/// Whether to paint the LIVE TEST banner: on by default for a windowed harness run
/// (`launched_for_a_task`), with `--no-test-banner` overriding off and `--test-banner` overriding
/// on. `force_off` wins over `force_on`, so a script that sets both gets the safe (hidden) result.
pub(crate) fn test_banner_on(launched_for_a_task: bool, force_on: bool, force_off: bool) -> bool {
    if force_off {
        false
    } else if force_on {
        true
    } else {
        launched_for_a_task
    }
}

impl crate::FractadyneApp {
    /// Paint the LIVE TEST banner across the top of a harness-driven window. Drawn on a foreground
    /// LAYER PAINTER, not an `Area` — the painter draws on top and senses no input, so it neither
    /// blocks `--uitest`'s synthetic clicks nor reflows the layout. It is egui, outside the fractal
    /// render target, so nothing the goldens / F3 corpus / `--livetest` compare can see it; it does
    /// appear in `--uitest` screenshots (which capture the whole frame), correctly labelling that
    /// bundle as a test — `--no-test-banner` removes it there when a pristine capture is wanted.
    pub(crate) fn draw_test_banner(&self, ctx: &egui::Context) {
        let Some(mode) = self.test_banner.as_deref() else {
            return;
        };
        let screen = ctx.screen_rect();
        let pr = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("live_test_banner"),
        ));
        let galley = pr.layout_no_wrap(
            format!("⚠ LIVE TEST · {mode} — automated window, not an interactive session"),
            egui::FontId::proportional(14.0),
            egui::Color32::WHITE,
        );
        let pad = egui::vec2(14.0, 7.0);
        let rect = egui::Rect::from_center_size(
            egui::pos2(screen.center().x, screen.min.y + 46.0),
            galley.size() + pad * 2.0,
        );
        pr.rect_filled(rect, 6.0, egui::Color32::from_rgb(0xC0, 0x2A, 0x2A));
        pr.rect_stroke(
            rect,
            6.0,
            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0x3a, 0x06, 0x06)),
            egui::StrokeKind::Inside,
        );
        pr.galley(rect.min + pad, galley, egui::Color32::PLACEHOLDER);
    }
}
