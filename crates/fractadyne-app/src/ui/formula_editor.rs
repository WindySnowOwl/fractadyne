//! The Custom formula dialog's text field: parentheses coloured by nesting depth (the pair at the
//! cursor highlighted, an unmatched one in the error colour), comments dimmed, and completion of
//! the language's names as you type.
//!
//! The scanning follows the parser's lexer (`core::ir::parse`): `;` starts a comment that runs to
//! the end of the line, identifiers are ASCII letters, digits and `_` (not starting with a digit)
//! and case-insensitive. The names offered are the keypad's — which a test holds to the parser's
//! vocabulary — and the variables the formula assigns.

use crate::ui::formula_keypad::{self, Action, Tab};
use egui::text::{CCursor, CCursorRange, LayoutJob, TextFormat};

/// A completion list opens once this many characters of a name are typed: one letter matches too
/// much (`c` is the pixel and the start of seven functions), and every name the language has is
/// at least two letters long but `z`, `c` and `e`, which need no completing.
const MIN_PREFIX: usize = 2;
/// The most names the list shows at once.
const MAX_SHOWN: usize = 10;

/// A parenthesis outside comments: its BYTE offset, its nesting depth (0 = outermost) and the byte
/// offset of its partner, `None` if it has none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Paren {
    pub(crate) at: usize,
    pub(crate) depth: usize,
    pub(crate) partner: Option<usize>,
}

/// Every parenthesis outside comments, in order, paired as the parser pairs them.
pub(crate) fn parens(src: &str) -> Vec<Paren> {
    let mut out: Vec<Paren> = Vec::new();
    let mut open: Vec<usize> = Vec::new(); // indices into `out`
    let mut in_comment = false;
    for (at, ch) in src.char_indices() {
        match ch {
            '\n' => in_comment = false,
            _ if in_comment => {}
            ';' => in_comment = true,
            '(' => {
                out.push(Paren { at, depth: open.len(), partner: None });
                open.push(out.len() - 1);
            }
            ')' => match open.pop() {
                Some(k) => {
                    out[k].partner = Some(at);
                    out.push(Paren { at, depth: open.len(), partner: Some(out[k].at) });
                }
                None => out.push(Paren { at, depth: 0, partner: None }),
            },
            _ => {}
        }
    }
    out
}

/// Byte ranges of the comments (`;` up to, not including, the line break).
pub(crate) fn comments(src: &str) -> Vec<std::ops::Range<usize>> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for (at, ch) in src.char_indices() {
        match (ch, start) {
            (';', None) => start = Some(at),
            ('\n', Some(s)) => {
                out.push(s..at);
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        out.push(s..src.len());
    }
    out
}

/// How the field draws text.
#[derive(Clone)]
pub(crate) struct Look {
    pub(crate) font: egui::FontId,
    pub(crate) text: egui::Color32,
    pub(crate) comment: egui::Color32,
    pub(crate) depth: [egui::Color32; 4],
    pub(crate) unmatched: egui::Color32,
    /// Behind the pair at the cursor (drawn in `text`).
    pub(crate) pair_bg: egui::Color32,
}

impl Look {
    pub(crate) fn of(ui: &egui::Ui) -> Look {
        let v = ui.visuals();
        Look {
            font: egui::TextStyle::Monospace.resolve(ui.style()),
            text: v.text_color(),
            comment: v.weak_text_color(),
            depth: crate::theme::paren_colors(ui.ctx()),
            unmatched: crate::theme::danger_color(ui.ctx()),
            pair_bg: v.widgets.hovered.bg_fill,
        }
    }
}

/// The byte offset of character `index` (the end, past the last character).
fn byte_at(src: &str, index: usize) -> usize {
    src.char_indices().nth(index).map_or(src.len(), |(b, _)| b)
}

/// The pair the cursor (a CHARACTER index) is at: the parenthesis just before it, else the one just
/// after it — as most editors choose — and its partner.
pub(crate) fn pair_at(src: &str, ps: &[Paren], cursor: usize) -> Option<(usize, usize)> {
    let at = byte_at(src, cursor);
    let before = src[..at].chars().next_back().map(|c| at - c.len_utf8());
    let found = |b: usize| ps.iter().find(|p| p.at == b);
    let p = before.and_then(found).or_else(|| found(at))?;
    Some((p.at, p.partner?))
}

/// The field's text as a layout: comments dimmed, each parenthesis in its depth's colour, the pair
/// at `cursor` (a character index) highlighted, an unmatched one in the error colour.
pub(crate) fn layout_job(src: &str, cursor: Option<usize>, look: &Look) -> LayoutJob {
    let ps = parens(src);
    let pair = cursor.and_then(|c| pair_at(src, &ps, c));
    let in_comment = comments(src);
    let plain = TextFormat { font_id: look.font.clone(), color: look.text, ..Default::default() };
    let format_of = |at: usize| -> TextFormat {
        if in_comment.iter().any(|r| r.contains(&at)) {
            return TextFormat { color: look.comment, italics: true, ..plain.clone() };
        }
        match ps.iter().find(|p| p.at == at) {
            None => plain.clone(),
            Some(_) if pair.is_some_and(|(a, b)| a == at || b == at) => {
                TextFormat { color: look.text, background: look.pair_bg, ..plain.clone() }
            }
            Some(p) if p.partner.is_none() => TextFormat { color: look.unmatched, ..plain.clone() },
            Some(p) => TextFormat { color: look.depth[p.depth % look.depth.len()], ..plain.clone() },
        }
    };
    // Runs of characters that share a format, as sections over `src`.
    let mut job = LayoutJob { text: src.to_string(), ..Default::default() };
    let mut run: Option<(usize, TextFormat)> = None;
    for (at, _) in src.char_indices() {
        let f = format_of(at);
        match &run {
            Some((_, g)) if *g == f => {}
            _ => {
                if let Some((start, g)) = run.take() {
                    job.sections.push(egui::text::LayoutSection { leading_space: 0.0, byte_range: start..at, format: g });
                }
                run = Some((at, f));
            }
        }
    }
    if let Some((start, g)) = run {
        job.sections.push(egui::text::LayoutSection { leading_space: 0.0, byte_range: start..src.len(), format: g });
    }
    if job.sections.is_empty() {
        // An empty text still needs a format: the cursor's height comes from it.
        job.sections.push(egui::text::LayoutSection { leading_space: 0.0, byte_range: 0..0, format: plain });
    }
    job
}

/// A name the field can complete.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Candidate {
    pub(crate) name: String,
    /// What replaces the typed prefix: the name, and `(` after a function.
    pub(crate) insert: String,
    pub(crate) hint: String,
}

/// The name being typed at `cursor` (a character index): its start (a character index) and text.
/// `None` inside a comment, in a number (`1e5`), or in the middle of a word.
pub(crate) fn prefix_at(src: &str, cursor: usize) -> Option<(usize, String)> {
    let chars: Vec<char> = src.chars().collect();
    if cursor == 0 || cursor > chars.len() {
        return None;
    }
    let word = |c: char| c.is_ascii_alphanumeric() || c == '_';
    if chars.get(cursor).is_some_and(|&c| word(c)) {
        return None;
    }
    let mut start = cursor;
    while start > 0 && word(chars[start - 1]) {
        start -= 1;
    }
    if start == cursor || !(chars[start].is_ascii_alphabetic() || chars[start] == '_') {
        return None;
    }
    if start > 0 && chars[start - 1] == '.' {
        return None;
    }
    let line_start = chars[..start].iter().rposition(|&c| c == '\n').map_or(0, |i| i + 1);
    if chars[line_start..start].contains(&';') {
        return None;
    }
    Some((start, chars[start..cursor].iter().collect()))
}

/// The language's names, from the keypad: a function as `name(`, a name as itself.
fn vocabulary() -> Vec<Candidate> {
    let ident = |s: &str| {
        let mut cs = s.chars();
        cs.next().is_some_and(|c| c.is_ascii_alphabetic()) && cs.all(|c| c.is_ascii_alphanumeric() || c == '_')
    };
    let mut out: Vec<Candidate> = Vec::new();
    for tab in [Tab::Functions, Tab::Names, Tab::Basic] {
        for k in formula_keypad::rows(tab).into_iter().flatten().flatten() {
            let (name, insert) = match k.action {
                Action::Wrap(open, _) if open.ends_with('(') && ident(&open[..open.len() - 1]) => {
                    (open[..open.len() - 1].to_string(), open.to_string())
                }
                Action::Insert(s) if ident(s) => (s.to_string(), s.to_string()),
                _ => continue,
            };
            if !out.iter().any(|c| c.name == name) {
                let hint = if k.hint.is_empty() { k.label } else { k.hint };
                out.push(Candidate { name, insert, hint: hint.to_string() });
            }
        }
    }
    out
}

/// The variables `src` assigns (`t = …` at the start of a statement), lower-cased as the parser
/// reads them, other than `z` and the language's own names.
fn variables(src: &str, known: &[Candidate]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in src.lines() {
        let code = line.split(';').next().unwrap_or("");
        // A `,` outside parentheses starts a statement too.
        let mut depth = 0i32;
        let mut stmt_start = 0;
        let mut stmts = Vec::new();
        for (i, ch) in code.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                ',' if depth <= 0 => {
                    stmts.push(&code[stmt_start..i]);
                    stmt_start = i + 1;
                }
                _ => {}
            }
        }
        stmts.push(&code[stmt_start..]);
        for s in stmts {
            let s = s.trim_start();
            let end = s.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(s.len());
            let (name, rest) = s.split_at(end);
            let rest = rest.trim_start();
            if !name.is_empty()
                && name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                && rest.starts_with('=')
                && !rest.starts_with("==")
            {
                let name = name.to_ascii_lowercase();
                if name != "z" && !known.iter().any(|c| c.name == name) && !out.contains(&name) {
                    out.push(name);
                }
            }
        }
    }
    out
}

/// The names starting with `prefix` (ignoring case, as the parser does), alphabetically.
pub(crate) fn candidates(src: &str, prefix: &str) -> Vec<Candidate> {
    let p = prefix.to_ascii_lowercase();
    let known = vocabulary();
    let mut out: Vec<Candidate> = variables(src, &known)
        .into_iter()
        .map(|name| Candidate { insert: name.clone(), name, hint: "A variable this formula sets".into() })
        .chain(known)
        .filter(|c| c.name.starts_with(&p))
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// The list to show for `src` with the cursor at `cursor`: the prefix's start, the prefix and the
/// names — only when there is something to complete (more than the name already typed in full).
pub(crate) fn completion(src: &str, cursor: usize) -> Option<(usize, String, Vec<Candidate>)> {
    let (start, prefix) = prefix_at(src, cursor)?;
    if prefix.chars().count() < MIN_PREFIX {
        return None;
    }
    let cs = candidates(src, &prefix);
    let nothing_to_add = cs.len() == 1 && cs[0].insert.eq_ignore_ascii_case(&prefix);
    (!cs.is_empty() && !nothing_to_add).then_some((start, prefix, cs))
}

/// Replace the prefix (characters `start..cursor`) with `c`. Returns the new text and cursor. A
/// function already followed by `(` gets no second one; the cursor goes past the existing one.
pub(crate) fn complete(src: &str, start: usize, cursor: usize, c: &Candidate) -> (String, usize) {
    let chars: Vec<char> = src.chars().collect();
    let (start, cursor) = (start.min(chars.len()), cursor.min(chars.len()));
    let before: String = chars[..start].iter().collect();
    let after: String = chars[cursor..].iter().collect();
    let insert = match c.insert.strip_suffix('(') {
        Some(name) if after.starts_with('(') => {
            let text = format!("{before}{name}{after}");
            return (text, start + name.chars().count() + 1);
        }
        _ => c.insert.as_str(),
    };
    (format!("{before}{insert}{after}"), start + insert.chars().count())
}

/// The completion list's state between frames.
#[derive(Default)]
pub(crate) struct Completion {
    /// The highlighted entry.
    sel: usize,
    /// Esc closed the list for this prefix (its start and text); it stays closed until that changes.
    dismissed: Option<(usize, String)>,
}

/// The cursor (a character index) egui keeps for field `id`.
fn stored_cursor(ctx: &egui::Context, id: egui::Id) -> Option<usize> {
    egui::text_edit::TextEditState::load(ctx, id).and_then(|s| s.cursor.char_range()).map(|r| r.primary.index)
}

/// Put field `id`'s cursor at character `at`.
pub(crate) fn store_cursor(ctx: &egui::Context, id: egui::Id, at: usize) {
    let mut state = egui::text_edit::TextEditState::load(ctx, id).unwrap_or_default();
    state.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(at))));
    state.store(ctx, id);
}

/// The formula's text field. While a completion list is open, ↑/↓ choose, Tab takes the
/// highlighted name, Enter takes it too unless it is exactly what is typed (so Enter after a
/// complete name still starts a new line), Esc closes the list, and a click takes a name. The keys
/// are taken BEFORE the field sees them, or a multi-line field would move its cursor or insert a
/// line break as well.
pub(crate) fn source_field(
    ui: &mut egui::Ui,
    id: egui::Id,
    text: &mut String,
    rows: usize,
    hint: &str,
    st: &mut Completion,
) -> egui::Response {
    let ctx = ui.ctx().clone();
    let focused = ctx.memory(|m| m.has_focus(id));
    let open = |text: &str, cursor: Option<usize>, st: &Completion| {
        let (start, prefix, cs) = completion(text, cursor?)?;
        (st.dismissed.as_ref() != Some(&(start, prefix.clone()))).then_some((start, prefix, cs))
    };
    if let Some((start, prefix, cs)) = open(text, stored_cursor(&ctx, id), st).filter(|_| focused) {
        let shown = cs.len().min(MAX_SHOWN);
        st.sel = st.sel.min(shown - 1);
        let mut take = false;
        ui.input_mut(|i| {
            if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown) {
                st.sel = (st.sel + 1) % shown;
            }
            if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) {
                st.sel = (st.sel + shown - 1) % shown;
            }
            let adds = !cs[st.sel].insert.eq_ignore_ascii_case(&prefix);
            take = i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)
                || (adds && i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
            if !take && i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                st.dismissed = Some((start, prefix.clone()));
            }
        });
        if take {
            let cursor = stored_cursor(&ctx, id).unwrap_or(0);
            let (new, at) = complete(text, start, cursor, &cs[st.sel]);
            *text = new;
            store_cursor(&ctx, id, at);
            st.sel = 0;
        }
    }

    let look = Look::of(ui);
    let cursor = stored_cursor(&ctx, id);
    let mut layouter = |ui: &egui::Ui, s: &str, wrap: f32| {
        let mut job = layout_job(s, cursor, &look);
        job.wrap.max_width = wrap;
        ui.fonts(|f| f.layout_job(job))
    };
    let out = egui::TextEdit::multiline(text)
        .id(id)
        .font(egui::TextStyle::Monospace)
        .desired_rows(rows)
        .desired_width(f32::INFINITY)
        .hint_text(hint)
        .layouter(&mut layouter)
        .show(ui);
    // The output has no cursor once the field has lost focus — as it does in the frame of a press on
    // the list — so fall back on the stored one, or the list vanished under that press.
    let now = out.cursor_range.map(|r| r.primary.ccursor.index).or_else(|| stored_cursor(&ctx, id));
    // The highlight was laid out with the cursor as it stood BEFORE this frame's keys and clicks;
    // one more frame shows it where it is now.
    if now != cursor {
        ctx.request_repaint();
    }
    if out.response.changed() {
        st.sel = 0;
    }

    // ⚠Drawn while the field had focus at the START of the frame, not only while it still has it: a
    // press anywhere outside the field takes its focus in the same frame (egui's
    // `pointer_pressed_elsewhere`), so a list drawn only for a focused field vanished under the very
    // press that chose from it. For the same reason an entry is taken on the PRESS — by the release
    // the list is gone.
    let mut clicked: Option<(usize, usize, Candidate)> = None;
    if focused || out.response.has_focus() {
        if let Some((start, _prefix, cs)) = open(text, now, st) {
            // ⚠Keep Tab, Esc and the arrows in the field next frame. egui acts on them at the START of
            // a frame, before any widget sees the key: Tab moves focus to the next widget and Esc drops
            // it, unless the focused widget's lock filter claims them. The field sets its own filter
            // as it draws (Tab not locked), so this goes after it.
            ctx.memory_mut(|m| {
                m.set_focus_lock_filter(
                    id,
                    egui::EventFilter { tab: true, horizontal_arrows: true, vertical_arrows: true, escape: true },
                )
            });
            let at = now.unwrap_or(0);
            let caret = out.galley.pos_from_ccursor(CCursor::new(at)).translate(out.galley_pos.to_vec2());
            egui::Area::new(id.with("completion"))
                .order(egui::Order::Foreground)
                .fixed_pos(caret.left_bottom() + egui::vec2(0.0, 2.0))
                .show(&ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        for (k, c) in cs.iter().take(MAX_SHOWN).enumerate() {
                            let r = ui
                                .horizontal(|ui| {
                                    let name = egui::RichText::new(&c.insert).monospace();
                                    let r = ui.selectable_label(k == st.sel, name);
                                    ui.label(egui::RichText::new(&c.hint).weak().small());
                                    r
                                })
                                .inner;
                            if r.is_pointer_button_down_on() {
                                clicked = Some((start, at, c.clone()));
                            }
                        }
                        if cs.len() > MAX_SHOWN {
                            ui.label(egui::RichText::new(format!("… {} more", cs.len() - MAX_SHOWN)).weak().small());
                        }
                        ui.label(egui::RichText::new("Tab or Enter to complete, Esc to close").weak().small());
                    });
                });
        }
    }
    if let Some((start, at, c)) = clicked {
        let (new, cursor) = complete(text, start, at, &c);
        *text = new;
        store_cursor(&ctx, id, cursor);
        ctx.memory_mut(|m| m.request_focus(id));
        st.sel = 0;
    }
    out.response
}

#[cfg(test)]
#[path = "formula_editor_tests.rs"]
mod tests;
