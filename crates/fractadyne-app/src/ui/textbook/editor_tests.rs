// The widget driven through a real (headless) egui context, as the text field's tests drive it.

use super::*;
use egui::{Event, Key, Modifiers, PointerButton};

fn id() -> egui::Id {
    egui::Id::new("textbook_editor_test")
}

fn key(k: Key) -> Event {
    key_with(k, Modifiers::NONE)
}

fn key_with(k: Key, modifiers: Modifiers) -> Event {
    Event::Key { key: k, physical_key: None, pressed: true, repeat: false, modifiers }
}

fn button(pos: egui::Pos2, pressed: bool) -> Event {
    Event::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE }
}

/// The editor in a panel, a button after it (so Tab has somewhere to move focus to).
struct Rig {
    ctx: egui::Context,
    ed: Editor,
    src: String,
    out: Outcome,
}

impl Rig {
    /// The editor holding `src`, focused.
    fn new(src: &str) -> Rig {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        // Fonts take effect from the next frame: one without the editor first.
        let _ = ctx.run(egui::RawInput::default(), |_| {});
        let mut rig = Rig { ctx, ed: Editor::default(), src: src.into(), out: Outcome::default() };
        rig.frame(vec![]);
        rig.ctx.memory_mut(|m| m.request_focus(id()));
        rig.frame(vec![]);
        rig.frame(vec![]);
        assert!(rig.focused(), "the editor has focus");
        rig
    }

    fn frame(&mut self, events: Vec<Event>) {
        let input = egui::RawInput {
            events,
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0))),
            ..Default::default()
        };
        let (ed, src) = (&mut self.ed, &mut self.src);
        let mut out = Outcome::default();
        let _ = self.ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                out = show(ui, id(), ed, src, 96.0, None);
                let _ = ui.button("Apply");
            });
        });
        self.out = out;
    }

    fn focused(&self) -> bool {
        self.ctx.memory(|m| m.has_focus(id()))
    }

    /// Where place `c` is on the screen, by the layout the widget drew.
    fn screen_pos(&self, c: &Caret) -> egui::Pos2 {
        let rect = self.ctx.read_response(id()).expect("the editor was drawn").rect;
        let ctx = Ctx { size_pt: SIZE_PT, ppp: self.ctx.pixels_per_point() };
        let frame = lay_out(&self.ed, &ctx, self.focused());
        let a = frame.anchor(c).expect("the place is drawn");
        rect.min + egui::vec2(10.0, 8.0) + egui::vec2(a.x, a.y - a.above / 3.0)
    }

    fn click(&mut self, at: egui::Pos2) {
        self.frame(vec![Event::PointerMoved(at)]);
        self.frame(vec![button(at, true)]);
        self.frame(vec![button(at, false)]);
    }
}

/// Typing goes into the formula, and Tab, Esc and the arrows stay with the editor.
#[test]
fn typing_edits_and_the_keys_stay_in_the_editor() {
    let mut r = Rig::new("");
    r.frame(vec![Event::Text("z=z^2+c/z".into())]);
    assert_eq!(r.src, "z = z^2 + c/z");
    assert!(r.out.changed);
    for k in [Key::Tab, Key::Escape, Key::ArrowLeft, Key::ArrowUp, Key::ArrowDown, Key::ArrowRight] {
        r.frame(vec![key(k)]);
        assert!(r.focused(), "{k:?} moved the focus away");
    }
    // ⌫ at the denominator's start selects the fraction, then takes it apart; Ctrl+Z puts it back.
    r.frame(vec![key(Key::End), key(Key::ArrowLeft), key(Key::ArrowLeft)]);
    r.frame(vec![key(Key::Backspace), key(Key::Backspace)]);
    assert_eq!(r.src, "z = z^2 + cz");
    r.frame(vec![key_with(Key::Z, Modifiers::COMMAND)]);
    assert_eq!(r.src, "z = z^2 + c/z");
    // Paste reads the text.
    r.frame(vec![key(Key::End), Event::Paste(" - 1/z".into())]);
    assert_eq!(r.src, "z = z^2 + c/z - 1/z");
}

/// A click puts the caret where it lands: in a denominator, the next key goes there.
#[test]
fn a_click_puts_the_caret_in_a_denominator() {
    let mut r = Rig::new("z = c/(z + 1)");
    let den = Caret { path: vec![(2, 1)], pos: 3, ..Default::default() };
    let at = r.screen_pos(&den);
    r.click(at);
    assert_eq!(r.ed.caret, den, "the click at {at:?}");
    r.frame(vec![Event::Text("2".into())]);
    assert_eq!(r.src, "z = c/(z + 12)");
}

/// A drag selects; typing replaces the selection.
#[test]
fn a_drag_selects() {
    let mut r = Rig::new("z = z^2 + c");
    let from = r.screen_pos(&Caret { pos: 2, ..Default::default() });
    let to = r.screen_pos(&Caret { pos: 6, ..Default::default() });
    r.frame(vec![Event::PointerMoved(from)]);
    r.frame(vec![button(from, true)]);
    for k in 1..=4 {
        r.frame(vec![Event::PointerMoved(from + (to - from) * (k as f32 / 4.0))]);
    }
    r.frame(vec![button(to, false)]);
    let s = r.ed.selection().expect("a selection");
    assert_eq!((s.at.pos, s.end), (2, 6));
    r.frame(vec![Event::Text("w".into())]);
    assert_eq!(r.src, "z = w");
}

/// The selection is painted behind the formula, and the caret drawn, while the editor has focus —
/// also when the state was set up directly, as the uitest's screens set it.
#[test]
fn the_selection_and_the_caret_are_drawn() {
    let mut r = Rig::new("z = t + c");
    r.frame(vec![Event::Text("/2".into()), key(Key::ArrowRight)]);
    r.frame(vec![key_with(Key::ArrowLeft, Modifiers::SHIFT)]);
    assert!(r.ed.selection().is_some(), "the fraction is selected");
    let mut ed = Editor::new("t = sqr(z) ; square\nz = t + c");
    ed.place_at(30);
    for ch in "/(1+p1".chars() {
        ed.type_char(ch);
    }
    ed.step(Dir::Right, false);
    ed.step(Dir::Right, false);
    for _ in 0..2 {
        ed.step(Dir::Left, true);
    }
    assert!(ed.selection().is_some());
    r.src = ed.synced.clone();
    r.ed = ed;
    r.frame(vec![]);
    assert!(r.ed.selection().is_some(), "still selected after a frame");
    let fill = r.ctx.style().visuals.selection.bg_fill;
    let caret = r.ctx.style().visuals.text_cursor.stroke.color;
    let shapes = |r: &mut Rig, window_focused: bool| {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0))),
            focused: window_focused,
            ..Default::default()
        };
        let (ed, src) = (&mut r.ed, &mut r.src);
        let full = r.ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show(ui, id(), ed, src, 96.0, None);
            });
        });
        fn flat(s: &egui::Shape, out: &mut Vec<egui::Shape>) {
            match s {
                egui::Shape::Vec(v) => v.iter().for_each(|s| flat(s, out)),
                s => out.push(s.clone()),
            }
        }
        let mut all = Vec::new();
        full.shapes.iter().for_each(|c| flat(&c.shape, &mut all));
        let selected = all.iter().any(|s| matches!(s, egui::Shape::Rect(r) if r.fill == fill));
        let caret = all.iter().any(|s| matches!(s, egui::Shape::LineSegment { stroke, .. } if stroke.color == caret));
        (selected, caret)
    };
    assert_eq!(shapes(&mut r, true), (true, true));
    // The window without the system's focus (the uitest's): the editor keeps its focus and its
    // selection; only the caret hides, as the text field's does.
    assert_eq!(shapes(&mut r, false), (true, false));
    assert!(r.focused());
}

/// A line that does not read is edited as text: a click on it says where it starts.
#[test]
fn a_click_on_a_line_that_does_not_read_goes_to_text() {
    let mut r = Rig::new("z = z^2 + c\nz = (");
    let rect = r.ctx.read_response(id()).expect("drawn").rect;
    let ctx = Ctx { size_pt: SIZE_PT, ppp: r.ctx.pixels_per_point() };
    let frame = lay_out(&r.ed, &ctx, true);
    let row = frame.laid.rows[1];
    let at = rect.min + egui::vec2(10.0 + 12.0, 8.0 + row.baseline - 4.0);
    r.frame(vec![Event::PointerMoved(at)]);
    r.frame(vec![button(at, true)]);
    assert_eq!(r.out.to_text, Some("z = z^2 + c\n".len()));
}
