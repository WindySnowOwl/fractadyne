//! Drawing a laid-out formula ([`super::layout::Laid`]) with egui.
//!
//! Glyphs are egui text in the math family, each placed by its galley's own baseline
//! (`Glyph::pos`, which epaint documents as the baseline), so the layout's baselines are exactly
//! where the ink goes. Rules are snapped to whole physical pixels: a fraction bar 1.1 px thick drawn
//! as is would straddle two pixel rows and come out grey and blurred.

use super::font::family;
use super::layout::{Item, Laid};

/// Draw `laid` with its top-left at `origin`.
pub(crate) fn paint(painter: &egui::Painter, origin: egui::Pos2, laid: &Laid, color: egui::Color32, weak: egui::Color32) {
    let ppp = painter.ctx().pixels_per_point();
    let snap = |v: f32| (v * ppp).round() / ppp;
    for it in &laid.items {
        match *it {
            Item::Glyph { ch, size, x, y } => {
                let galley = painter.layout_no_wrap(ch.to_string(), egui::FontId::new(size, family()), color);
                let Some(g) = galley.rows.first().and_then(|r| r.glyphs.first()) else { continue };
                let at = origin + egui::vec2(x - g.pos.x, y - g.pos.y);
                painter.galley(at, galley.clone(), color);
            }
            Item::Rule { x, y, w, h } => {
                let top = snap(origin.y + y);
                let thick = (h * ppp).round().max(1.0) / ppp;
                let r = egui::Rect::from_min_size(egui::pos2(origin.x + x, top), egui::vec2(w, thick));
                painter.rect_filled(r, 0.0, color);
            }
            Item::Slot { x, y, w, h } => {
                let r = egui::Rect::from_min_size(origin + egui::vec2(x, y), egui::vec2(w, h));
                let stroke = egui::Stroke::new(1.0_f32, weak);
                let (dash, gap) = (3.0_f32, 2.0_f32);
                for (a, b) in [
                    (r.left_top(), r.right_top()),
                    (r.right_top(), r.right_bottom()),
                    (r.right_bottom(), r.left_bottom()),
                    (r.left_bottom(), r.left_top()),
                ] {
                    painter.extend(egui::Shape::dashed_line(&[a, b], stroke, dash, gap));
                }
            }
        }
    }
}
