//! Branding & theme: the Fractadyne color identity (amber accent), the dark egui visuals,
//! the two-color wordmark, the procedural window icon, and a couple of small painting helpers.

use eframe::egui;

/// Brand accent (amber #E0A030) + logotype text color (#E6E7EA). The wordmark is
/// "Fracta" (text) + "dyne" (amber).
pub(crate) const BRAND_ACCENT: egui::Color32 = egui::Color32::from_rgb(0xE0, 0xA0, 0x30);
pub(crate) const BRAND_TEXT: egui::Color32 = egui::Color32::from_rgb(0xE6, 0xE7, 0xEA);

/// Linear RGBA interpolation between two colors (`t` in 0..1).
pub(crate) fn lerp_color(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    egui::Color32::from_rgba_unmultiplied(
        mix(a.r(), b.r()),
        mix(a.g(), b.g()),
        mix(a.b(), b.b()),
        mix(a.a(), b.a()),
    )
}

/// A compact toggle button drawn as two side-by-side rectangles (the dual-view
/// icon). Painted directly so it renders identically regardless of font glyphs.
pub(crate) fn dual_toggle_button(ui: &mut egui::Ui, selected: bool) -> egui::Response {
    let size = egui::vec2(30.0, ui.spacing().interact_size.y);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let v = ui.style().interact_selectable(&resp, selected);
    let p = ui.painter();
    p.rect_filled(rect, egui::CornerRadius::same(2), v.bg_fill);
    let inner = rect.shrink2(egui::vec2(8.0, 4.0));
    let gap = 3.0;
    let half_w = ((inner.width() - gap) * 0.5).max(1.0);
    let left = egui::Rect::from_min_size(inner.min, egui::vec2(half_w, inner.height()));
    let right = egui::Rect::from_min_size(
        egui::pos2(inner.min.x + half_w + gap, inner.min.y),
        egui::vec2(half_w, inner.height()),
    );
    let stroke = egui::Stroke::new(1.4, v.fg_stroke.color);
    p.rect_stroke(left, egui::CornerRadius::same(1), stroke, egui::StrokeKind::Inside);
    p.rect_stroke(right, egui::CornerRadius::same(1), stroke, egui::StrokeKind::Inside);
    resp
}

/// Apply the Fractadyne dark theme (charcoal panels + amber accents, per the design).
pub(crate) fn apply_brand_theme(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    let panel = egui::Color32::from_rgb(0x1A, 0x1B, 0x1E);
    v.panel_fill = panel;
    v.window_fill = panel;
    v.extreme_bg_color = egui::Color32::from_rgb(0x10, 0x10, 0x15);
    v.faint_bg_color = egui::Color32::from_rgb(0x23, 0x24, 0x28);
    v.hyperlink_color = BRAND_ACCENT;
    v.selection.bg_fill = egui::Color32::from_rgb(0x4A, 0x38, 0x14);
    v.selection.stroke = egui::Stroke::new(1.0, BRAND_ACCENT);
    v.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(0x23, 0x24, 0x28);
    v.widgets.inactive.bg_fill = egui::Color32::from_rgb(0x2C, 0x2E, 0x33);
    v.widgets.hovered.bg_fill = egui::Color32::from_rgb(0x3A, 0x3D, 0x44);
    v.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(0x3A, 0x3D, 0x44);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, BRAND_ACCENT);
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, BRAND_ACCENT);
    v.widgets.active.bg_fill = egui::Color32::from_rgb(0x45, 0x49, 0x52);
    v.widgets.active.bg_stroke = egui::Stroke::new(1.0, BRAND_ACCENT);
    v.widgets.open.bg_fill = egui::Color32::from_rgb(0x2C, 0x2E, 0x33);
    ctx.set_visuals(v);
}

/// The two-color "Fractadyne" logotype (Fracta + amber dyne), for the top bar.
pub(crate) fn brand_wordmark(ui: &mut egui::Ui) {
    let font = egui::FontId::proportional(15.0);
    let mut job = egui::text::LayoutJob::default();
    job.append(
        "Fracta",
        0.0,
        egui::TextFormat { font_id: font.clone(), color: BRAND_TEXT, ..Default::default() },
    );
    job.append(
        "dyne",
        0.0,
        egui::TextFormat { font_id: font, color: BRAND_ACCENT, ..Default::default() },
    );
    ui.add_space(2.0);
    ui.label(job);
}

/// Procedural window icon: concentric amber rings on a dark disc (transparent corners).
pub(crate) fn brand_icon() -> egui::IconData {
    let n: u32 = 64;
    let mut rgba = vec![0u8; (n * n * 4) as usize];
    let center = (n as f32 - 1.0) * 0.5;
    let acc = [0xE0_u8, 0xA0, 0x30];
    let acc2 = [0xFF_u8, 0xCB, 0x6B];
    let bg = [0x10_u8, 0x10, 0x15];
    for y in 0..n {
        for x in 0..n {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let d = (dx * dx + dy * dy).sqrt() / (n as f32 * 0.5);
            let i = ((y * n + x) * 4) as usize;
            if d > 1.0 {
                rgba[i + 3] = 0; // transparent outside the disc
                continue;
            }
            // concentric rings blending the two accents over the dark disc
            let ring = 0.5 + 0.5 * (d * 22.0).cos();
            let t = (ring * (1.0 - d)).clamp(0.0, 1.0);
            let blend = 0.5 + 0.5 * (d * 6.0).cos();
            let a0 = [
                (acc[0] as f32 * blend + acc2[0] as f32 * (1.0 - blend)),
                (acc[1] as f32 * blend + acc2[1] as f32 * (1.0 - blend)),
                (acc[2] as f32 * blend + acc2[2] as f32 * (1.0 - blend)),
            ];
            for k in 0..3 {
                rgba[i + k] = (bg[k] as f32 + (a0[k] - bg[k] as f32) * t) as u8;
            }
            rgba[i + 3] = 255;
        }
    }
    egui::IconData { rgba, width: n, height: n }
}
