//! The Textbook view of the formula (read-only for now): every statement typeset, aligned at its
//! `=`, its comment after it; a line that does not read shown as text in the error colour, with the
//! reason on hover. A click returns the clicked statement's place in the source, for the dialog to
//! put the text caret there.

use super::layout::{self, Ctx, Node};
use super::model::{self, Line};
use super::paint;

/// The text size of a typeset formula, in points.
pub(crate) const SIZE_PT: f32 = 18.0;

/// One typeset row and what goes with it.
struct ViewRow {
    nodes: Vec<Node>,
    /// The byte the row's text starts at, for a click.
    at: usize,
    comment: Option<String>,
    /// A line that does not read: its text and why.
    unread: Option<(String, String)>,
}

fn view_rows(src: &str) -> Vec<ViewRow> {
    let mut out = Vec::new();
    for (line, span) in model::read(src) {
        match line {
            Line::Read { stmts, comment } if stmts.is_empty() => {
                out.push(ViewRow { nodes: Vec::new(), at: span.start, comment, unread: None })
            }
            Line::Read { stmts, comment } => {
                let last = stmts.len() - 1;
                for (i, st) in stmts.into_iter().enumerate() {
                    out.push(ViewRow {
                        nodes: model::math_row(&st.row),
                        at: st.span.start,
                        comment: if i == last { comment.clone() } else { None },
                        unread: None,
                    });
                }
            }
            Line::Unread { text, error } => {
                out.push(ViewRow { nodes: Vec::new(), at: span.start, comment: None, unread: Some((text, error)) })
            }
        }
    }
    out
}

/// Show `src` typeset, at least `min_height` points tall. Returns the source byte of the statement
/// clicked, if one was.
pub(crate) fn show(ui: &mut egui::Ui, src: &str, min_height: f32) -> Option<usize> {
    let rows = view_rows(src);
    let ctx = Ctx { size_pt: SIZE_PT, ppp: ui.ctx().pixels_per_point() };
    let laid = layout::rows(&rows.iter().map(|r| r.nodes.clone()).collect::<Vec<_>>(), &ctx);
    let v = ui.visuals().clone();
    let note_font = egui::FontId::proportional(SIZE_PT * 0.7);
    let mono = egui::FontId::monospace(SIZE_PT * 0.75);
    let margin = egui::vec2(10.0, 8.0);
    // Room for the comments and unread lines beside the typeset ones.
    let extra = rows
        .iter()
        .zip(&laid.rows)
        .map(|(r, p)| {
            let text_w = |t: &str, f: &egui::FontId| ui.fonts(|fs| fs.layout_no_wrap(t.to_string(), f.clone(), v.text_color()).size().x);
            let c = r.comment.as_deref().map_or(0.0, |c| p.right + 16.0 + text_w(&format!("; {c}"), &note_font));
            let u = r.unread.as_ref().map_or(0.0, |(t, _)| text_w(t, &mono));
            c.max(u)
        })
        .fold(laid.size.x, f32::max);
    let size = egui::vec2(extra, laid.size.y) + 2.0 * margin;
    let mut clicked = None;
    egui::Frame::new()
        .fill(v.extreme_bg_color)
        .stroke(v.widgets.inactive.bg_stroke)
        .corner_radius(v.widgets.inactive.corner_radius)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            egui::ScrollArea::horizontal().id_salt("textbook_view").show(ui, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(size.x, size.y.max(min_height)), egui::Sense::click());
                let origin = rect.min + margin;
                let painter = ui.painter_at(rect);
                paint::paint(&painter, origin, &laid, v.text_color(), v.weak_text_color());
                for (r, p) in rows.iter().zip(&laid.rows) {
                    let baseline = origin.y + p.baseline;
                    if let Some(c) = &r.comment {
                        let g = painter.layout_no_wrap(format!("; {c}"), note_font.clone(), v.weak_text_color());
                        let dy = g.rows.first().and_then(|row| row.glyphs.first()).map_or(0.0, |gl| gl.pos.y);
                        let x = origin.x + if r.nodes.is_empty() { 0.0 } else { p.right + 16.0 };
                        painter.galley(egui::pos2(x, baseline - dy), g, v.weak_text_color());
                    }
                    if let Some((t, _)) = &r.unread {
                        let color = crate::theme::danger_color(ui.ctx());
                        let g = painter.layout_no_wrap(t.clone(), mono.clone(), color);
                        let dy = g.rows.first().and_then(|row| row.glyphs.first()).map_or(0.0, |gl| gl.pos.y);
                        painter.galley(egui::pos2(origin.x, baseline - dy), g, color);
                    }
                }
                let row_at = |pos: egui::Pos2| {
                    rows.iter().zip(&laid.rows).find(|(_, p)| pos.y <= origin.y + p.bottom + 4.0).or(rows.iter().zip(&laid.rows).last())
                };
                let response = match response.hover_pos().and_then(row_at) {
                    Some((ViewRow { unread: Some((_, why)), .. }, _)) => response.on_hover_text(format!("Does not read: {why}")),
                    _ => response.on_hover_text("Click to edit this line as text"),
                };
                if response.clicked() {
                    clicked = response.interact_pointer_pos().and_then(row_at).map(|(r, _)| r.at);
                }
            });
        });
    clicked
}
