//! The Textbook editor (design/formula-textbook-editor.md §4.8): the formula typeset, with a caret
//! and a selection in it; keys, clicks and the clipboard become [`super::edit`] commands. The source
//! stays the truth: every edit prints the document back into it, and a source changed elsewhere
//! (Text mode, an example, the library) is read again.

use super::edit::{Dir, Editor, Shown, Vertical};
use super::layout::{self, Anchor, Ctx, Laid};
use super::model::{self, Caret, Marker};
use super::paint;

/// The text size of the typeset formula, in points.
pub(crate) const SIZE_PT: f32 = 18.0;

/// What a frame of the editor did.
#[derive(Debug, Default)]
pub(crate) struct Outcome {
    /// The text changed (the source already holds the new text).
    pub(crate) changed: bool,
    /// A line that does not read was clicked: edit it as text, from this byte of the source.
    pub(crate) to_text: Option<usize>,
}

/// A row as shown, beside its typeset math.
struct RowInfo {
    /// Its line in the document (and the source).
    line: usize,
    comment: Option<String>,
    /// A line that does not read: its text, why, and where it starts in the source.
    unread: Option<(String, String, usize)>,
    /// A statement with nothing in it (a comment alone on its line): its comment starts at the left.
    empty: bool,
}

/// One layout of the document: the typeset rows and where every caret place went.
struct Frame {
    laid: Laid,
    /// Each mark id's place.
    places: Vec<Caret>,
    rows: Vec<RowInfo>,
}

fn lay_out(ed: &Editor, ctx: &Ctx, focused: bool) -> Frame {
    // While focused, the name at the caret (and at the selection's other end) shows as typed.
    let raw = if focused { std::iter::once(ed.caret.clone()).chain(ed.anchor.clone()).collect() } else { Vec::new() };
    let mut m = Marker::new(raw);
    let (mut nodes, mut rows) = (Vec::new(), Vec::new());
    for s in ed.doc.shown() {
        match s {
            Shown::Stmt { line, stmt, row, comment } => {
                m.statement(line, stmt);
                nodes.push(model::emit(row, &mut Vec::new(), &mut m, true));
                rows.push(RowInfo { line, comment: comment.map(str::to_string), unread: None, empty: row.is_empty() });
            }
            Shown::Unread { line, text, error } => {
                nodes.push(Vec::new());
                let at = ed.doc.line_start(line);
                rows.push(RowInfo { line, comment: None, unread: Some((text.to_string(), error.to_string(), at)), empty: true });
            }
        }
    }
    Frame { laid: layout::rows(&nodes, ctx), places: m.places, rows }
}

impl Frame {
    fn place(&self, a: &Anchor) -> &Caret {
        &self.places[a.id as usize]
    }

    /// The anchors of the row `row` names (its `pos` aside).
    fn in_row<'a>(&'a self, row: &'a Caret) -> impl Iterator<Item = &'a Anchor> + 'a {
        self.laid.anchors.iter().filter(move |a| {
            let p = self.place(a);
            (p.line, p.stmt, &p.path) == (row.line, row.stmt, &row.path)
        })
    }

    /// Where a place is drawn: its own anchor, else (a place inside a name shown as π or 𝑝₁) the
    /// nearest of its row's.
    fn anchor(&self, c: &Caret) -> Option<Anchor> {
        let own = self.places.iter().position(|p| p == c).and_then(|id| self.laid.anchors.iter().find(|a| a.id as usize == id));
        own.or_else(|| self.in_row(c).min_by_key(|a| self.place(a).pos.abs_diff(c.pos))).copied()
    }

    /// The place nearest a point (from the formula's top left): one in a row whose extent holds the
    /// point — the innermost row winning a near tie — else in the nearest row.
    fn hit(&self, p: egui::Pos2) -> Option<Caret> {
        self.laid
            .anchors
            .iter()
            .map(|a| {
                let dy = ((a.y - a.above) - p.y).max(p.y - (a.y + a.below)).max(0.0);
                ((p.x - a.x).abs() + 3.0 * dy + 0.15 * (a.above + a.below), a)
            })
            .min_by(|x, y| x.0.total_cmp(&y.0))
            .map(|(_, a)| self.place(a).clone())
    }

    /// The place in the row `row` names nearest to `x`.
    fn nearest_in(&self, row: &Caret, x: f32) -> Option<Caret> {
        self.in_row(row).min_by(|a, b| (a.x - x).abs().total_cmp(&(b.x - x).abs())).map(|a| self.place(a).clone())
    }

    /// The line that does not read at height `y`, if one is there.
    fn unread_at(&self, y: f32) -> Option<&(String, String, usize)> {
        self.rows
            .iter()
            .zip(&self.laid.rows)
            .find(|(r, p)| r.unread.is_some() && y >= p.top - 4.0 && y <= p.bottom + 4.0)
            .and_then(|(r, _)| r.unread.as_ref())
    }
}

/// A drag that left the selection's statement stays in it, at the end it left by.
fn within_anchor_statement(ed: &Editor, c: Caret) -> Caret {
    let a = ed.anchor.as_ref().unwrap_or(&ed.caret);
    if (a.line, a.stmt) == (c.line, c.stmt) {
        return c;
    }
    let after = (c.line, c.stmt) > (a.line, a.stmt);
    let len = ed.doc.row(&Caret { line: a.line, stmt: a.stmt, ..Default::default() }).map_or(0, |r| r.len());
    Caret { line: a.line, stmt: a.stmt, path: Vec::new(), pos: if after { len } else { 0 } }
}

/// Apply this frame's keys and clipboard events. Returns (text changed, caret moved).
fn keys(ui: &egui::Ui, ed: &mut Editor, frame: &Frame) -> (bool, bool) {
    use egui::{Event, Key};
    let events = ui.input(|i| i.events.clone());
    let (mut changed, mut moved) = (false, false);
    for ev in events {
        match ev {
            Event::Text(t) => {
                for ch in t.chars() {
                    changed |= ed.type_char(ch);
                }
                moved = true;
            }
            Event::Copy => {
                if let Some(t) = ed.copy() {
                    ui.ctx().copy_text(t);
                }
            }
            Event::Cut => {
                if let Some(t) = ed.cut() {
                    ui.ctx().copy_text(t);
                    changed = true;
                }
            }
            Event::Paste(t) => {
                changed |= ed.paste(&t);
                moved = true;
            }
            Event::Key { key, pressed: true, modifiers: mods, .. } => {
                moved = true;
                match key {
                    Key::ArrowLeft => ed.step(Dir::Left, mods.shift),
                    Key::ArrowRight => ed.step(Dir::Right, mods.shift),
                    Key::ArrowUp | Key::ArrowDown => {
                        // Positions on screen are this frame's: an edit earlier in the same frame
                        // moved them, so ↑/↓ after one waits for the next.
                        if changed {
                            continue;
                        }
                        let x = frame.anchor(&ed.caret).map_or(0.0, |a| a.x);
                        match ed.vertical(key == Key::ArrowUp) {
                            Vertical::To(c) => ed.set_caret(c, false),
                            Vertical::Nearest(row) => {
                                if let Some(c) = frame.nearest_in(&row, x) {
                                    ed.set_caret(c, false);
                                }
                            }
                            Vertical::Stay => {}
                        }
                    }
                    Key::Home => ed.home_end(false, mods.shift),
                    Key::End => ed.home_end(true, mods.shift),
                    Key::Backspace => changed |= ed.backspace(),
                    Key::Delete => changed |= ed.delete(),
                    Key::Enter => changed |= ed.enter(),
                    Key::Tab => {
                        ed.tab(mods.shift);
                    }
                    Key::Escape => ed.anchor = None,
                    Key::A if mods.command => ed.select_all(),
                    Key::Z if mods.command && mods.shift => changed |= ed.redo(),
                    Key::Z if mods.command => changed |= ed.undo(),
                    Key::Y if mods.command => changed |= ed.redo(),
                    _ => moved = false,
                }
            }
            _ => {}
        }
    }
    (changed, moved)
}

/// The formula as an editable typeset document, at least `min_height` points tall. `source` is the
/// dialog's text: read when it changed elsewhere, written on every edit. `error_line`: the source
/// line (from 1) the syntax check stops at, underlined in the error colour.
pub(crate) fn show(
    ui: &mut egui::Ui,
    id: egui::Id,
    ed: &mut Editor,
    source: &mut String,
    min_height: f32,
    error_line: Option<usize>,
) -> Outcome {
    let mut out = Outcome::default();
    ed.sync(source);
    let ctx = Ctx { size_pt: SIZE_PT, ppp: ui.ctx().pixels_per_point() };
    let focused = ui.memory(|m| m.has_focus(id));
    let mut frame = lay_out(ed, &ctx, focused);
    let v = ui.visuals().clone();
    let note_font = egui::FontId::proportional(SIZE_PT * 0.7);
    let mono = egui::FontId::monospace(SIZE_PT * 0.75);
    let margin = egui::vec2(10.0, 8.0);
    let text_w = |t: &str, f: &egui::FontId| ui.fonts(|fs| fs.layout_no_wrap(t.to_string(), f.clone(), v.text_color()).size().x);
    // Room for the comments and the lines that do not read beside the typeset rows.
    let width = frame
        .rows
        .iter()
        .zip(&frame.laid.rows)
        .map(|(r, p)| {
            let c = r.comment.as_deref().map_or(0.0, |c| p.right + 16.0 + text_w(&format!("; {c}"), &note_font));
            let u = r.unread.as_ref().map_or(0.0, |(t, _, _)| text_w(t, &mono));
            c.max(u)
        })
        .fold(frame.laid.size.x, f32::max);
    let size = egui::vec2(width + SIZE_PT, frame.laid.size.y) + 2.0 * margin;
    let stroke = if focused { v.selection.stroke } else { v.widgets.inactive.bg_stroke };
    egui::Frame::new().fill(v.extreme_bg_color).stroke(stroke).corner_radius(v.widgets.inactive.corner_radius).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        egui::ScrollArea::horizontal().id_salt(id.with("scroll")).show(ui, |ui| {
            let (_, rect) = ui.allocate_space(egui::vec2(size.x.max(ui.available_width()), size.y.max(min_height)));
            let response = ui.interact(rect, id, egui::Sense::click_and_drag()).on_hover_cursor(egui::CursorIcon::Text);
            let origin = rect.min + margin;
            let now = ui.input(|i| i.time);
            let touched = id.with("touched");
            let mut moved = false;

            // The pointer: a press puts the caret (Shift extends the selection), a drag selects, a
            // double click selects the name or number under it.
            let press = ui.input(|i| i.pointer.primary_pressed()) && response.contains_pointer();
            if let Some(p) = response.interact_pointer_pos().or(response.hover_pos()) {
                let local = (p - origin).to_pos2();
                if press {
                    response.request_focus();
                    if let Some((_, _, at)) = frame.unread_at(local.y) {
                        out.to_text = Some(*at);
                    } else if let Some(c) = frame.hit(local) {
                        let shift = ui.input(|i| i.modifiers.shift);
                        ed.set_caret(c, shift);
                        moved = true;
                    }
                } else if response.dragged() && out.to_text.is_none() {
                    if let Some(c) = frame.hit(local) {
                        let c = within_anchor_statement(ed, c);
                        if c != ed.caret {
                            ed.set_caret(c, true);
                            moved = true;
                        }
                    }
                }
            }
            if response.double_clicked() {
                ed.select_word();
                moved = true;
            }

            // ⚠Not `response.has_focus()`: since egui 0.29 that is false whenever the WINDOW lacks
            // the system's focus, so an editor that kept its focus dropped its selection (measured
            // in the uitest's window: memory focused, response not, every frame). Keys only arrive
            // in a focused window anyway; the caret alone hides with it, as the text field's does.
            let has_focus = ui.memory(|m| m.has_focus(id));
            if has_focus {
                // ⚠Keep Tab, Esc and the arrows here next frame: egui acts on them at the START of a
                // frame, before any widget sees the key (Tab moves focus on, Esc drops it).
                ui.memory_mut(|m| {
                    m.set_focus_lock_filter(id, egui::EventFilter { tab: true, horizontal_arrows: true, vertical_arrows: true, escape: true })
                });
                let (changed, by_key) = keys(ui, ed, &frame);
                moved |= by_key;
                if changed {
                    *source = ed.synced.clone();
                    out.changed = true;
                }
            }
            if moved || out.changed || has_focus != focused {
                frame = lay_out(ed, &ctx, has_focus);
                ui.ctx().request_repaint();
            }
            if moved || out.changed {
                ui.data_mut(|d| d.insert_temp(touched, now));
            }

            let painter = ui.painter_at(rect);
            // The selection, behind the formula.
            if let Some(s) = ed.selection().filter(|_| has_focus) {
                let ends = (frame.anchor(&s.at), frame.anchor(&Caret { pos: s.end, ..s.at.clone() }));
                if let (Some(a), Some(b)) = ends {
                    let r = egui::Rect::from_min_max(
                        origin + egui::vec2(a.x.min(b.x), (a.y - a.above).min(b.y - b.above)),
                        origin + egui::vec2(a.x.max(b.x), (a.y + a.below).max(b.y + b.below)),
                    );
                    painter.rect_filled(r, 2.0, v.selection.bg_fill);
                }
            }
            paint::paint(&painter, origin, &frame.laid, v.text_color(), v.weak_text_color());
            for (r, p) in frame.rows.iter().zip(&frame.laid.rows) {
                let baseline = origin.y + p.baseline;
                // The statement the syntax check stops at (a red line of text marks itself).
                if error_line == Some(r.line + 1) && r.unread.is_none() && !r.empty {
                    let y = origin.y + p.bottom + 2.0;
                    let stroke = egui::Stroke::new(1.5_f32, crate::theme::danger_color(ui.ctx()));
                    painter.line_segment([egui::pos2(origin.x + p.left, y), egui::pos2(origin.x + p.right, y)], stroke);
                }
                if let Some(c) = &r.comment {
                    let g = painter.layout_no_wrap(format!("; {c}"), note_font.clone(), v.weak_text_color());
                    let dy = g.rows.first().and_then(|row| row.glyphs.first()).map_or(0.0, |gl| gl.pos.y);
                    let x = origin.x + if r.empty { 0.0 } else { p.right + 16.0 };
                    painter.galley(egui::pos2(x, baseline - dy), g, v.weak_text_color());
                }
                if let Some((t, _, _)) = &r.unread {
                    let color = crate::theme::danger_color(ui.ctx());
                    let g = painter.layout_no_wrap(t.clone(), mono.clone(), color);
                    let dy = g.rows.first().and_then(|row| row.glyphs.first()).map_or(0.0, |gl| gl.pos.y);
                    painter.galley(egui::pos2(origin.x, baseline - dy), g, color);
                }
            }
            // The caret, blinking as egui's text cursor does.
            if has_focus && ui.input(|i| i.focused) {
                if let Some(a) = frame.anchor(&ed.caret) {
                    let ppp = ui.ctx().pixels_per_point();
                    let x = ((origin.x + a.x) * ppp).round() / ppp;
                    let r = egui::Rect::from_x_y_ranges(x..=x, origin.y + a.y - a.above..=origin.y + a.y + a.below);
                    let since = now - ui.data(|d| d.get_temp::<f64>(touched)).unwrap_or(now);
                    egui::text_selection::visuals::paint_text_cursor(ui, &painter, r, since);
                    if moved || out.changed {
                        ui.scroll_to_rect(r.expand2(egui::vec2(SIZE_PT, 0.0)), None);
                    }
                }
            }
            let hover = frame.unread_at((response.hover_pos().unwrap_or_default() - origin).y);
            let response = match hover {
                Some((_, why, _)) => response.on_hover_text(format!("Does not read: {why}. Click to edit it as text.")),
                None => response,
            };
            // For a screen reader, the formula is its text.
            let text = source.clone();
            response.widget_info(|| egui::WidgetInfo::text_edit(true, &text, &text));
        });
    });
    out
}

#[cfg(test)]
#[path = "editor_tests.rs"]
mod tests;
