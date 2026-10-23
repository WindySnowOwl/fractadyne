//! The Custom formula dialog's keypad: every name and operator the formula language accepts, on
//! buttons, grouped into tabs (in the manner of GeoGebra's input keyboard). A key inserts at the
//! text cursor; a function key wraps the selection, or leaves the cursor between its parentheses.
//!
//! The keys are the parser's vocabulary and nothing else — a test parses what every key produces,
//! so the keypad cannot advertise a name the language does not have.

/// What a key does to the text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    /// Replace the selection with this text; the cursor ends after it.
    Insert(&'static str),
    /// Put `before`/`after` around the selection (or, with none, insert both with the cursor
    /// between them).
    Wrap(&'static str, &'static str),
    Backspace,
    Left,
    Right,
}

pub(crate) struct Key {
    pub(crate) label: &'static str,
    pub(crate) action: Action,
    /// The key's tooltip in the language in use (empty: none). A function, as the tables are
    /// constants and the text is looked up where it is drawn.
    pub(crate) hint: fn() -> String,
}

const fn key(label: &'static str, action: Action, hint: fn() -> String) -> Key {
    Key { label, action, hint }
}

/// A key without a tooltip (a digit, a parenthesis).
fn none() -> String {
    String::new()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum Tab {
    #[default]
    Basic,
    Functions,
    Names,
}

impl Tab {
    pub(crate) const ALL: [Tab; 3] = [Tab::Basic, Tab::Functions, Tab::Names];
    pub(crate) fn label(self) -> String {
        match self {
            Tab::Basic => "123".to_string(),
            Tab::Functions => "f(z)".to_string(),
            Tab::Names => tr!(formula_keypad_tab_names),
        }
    }
    pub(crate) fn hint(self) -> String {
        match self {
            Tab::Basic => tr!(formula_keypad_tab_basic_hint),
            Tab::Functions => tr!(formula_keypad_tab_functions_hint),
            Tab::Names => tr!(formula_keypad_tab_names_hint),
        }
    }
}

use Action::*;

/// `123`: two blocks, as on a calculator — names and templates on the left, digits and operators
/// on the right. Rows are laid out left to right, `None` a gap between the blocks.
const BASIC: [[Option<Key>; 10]; 4] = [
    [
        Some(key("z", Insert("z"), || tr!(formula_key_z))),
        Some(key("c", Insert("c"), || tr!(formula_key_c))),
        Some(key("p1", Insert("p1"), || tr!(formula_key_p1_below))),
        Some(key("p2", Insert("p2"), || tr!(formula_key_param, n = 2))),
        None,
        Some(key("7", Insert("7"), none)),
        Some(key("8", Insert("8"), none)),
        Some(key("9", Insert("9"), none)),
        Some(key("×", Insert("*"), || tr!(formula_key_multiply))),
        Some(key("÷", Insert("/"), || tr!(formula_key_divide))),
    ],
    [
        Some(key("□²", Insert("^2"), || tr!(formula_key_square_after))),
        // Not "□ⁿ": the UI font has no superscript n (it drew a box).
        Some(key("□^", Insert("^"), || tr!(formula_key_power))),
        Some(key("√", Wrap("sqrt(", ")"), || tr!(formula_key_square_root))),
        Some(key("|□|", Wrap("|", "|"), || tr!(formula_key_squared_modulus))),
        None,
        Some(key("4", Insert("4"), none)),
        Some(key("5", Insert("5"), none)),
        Some(key("6", Insert("6"), none)),
        Some(key("+", Insert(" + "), || tr!(formula_key_add))),
        Some(key("−", Insert(" - "), || tr!(formula_key_subtract))),
    ],
    [
        Some(key("(", Insert("("), none)),
        Some(key(")", Insert(")"), none)),
        Some(key("=", Insert(" = "), || tr!(formula_key_assign))),
        Some(key("↵", Insert("\n"), || tr!(formula_key_next_statement_comma))),
        None,
        Some(key("1", Insert("1"), none)),
        Some(key("2", Insert("2"), none)),
        Some(key("3", Insert("3"), none)),
        // Not "⌫": the UI font has no erase glyph (it drew a box), and the icon font's DELETE is a bin.
        Some(key("del", Backspace, || tr!(formula_key_backspace))),
        None,
    ],
    [
        Some(key("π", Insert("pi"), || "π".to_string())),
        Some(key("e", Insert("e"), || tr!(formula_key_euler))),
        Some(key("(a, b)", Insert("(0.5, 0.5)"), || tr!(formula_key_complex_constant))),
        Some(key(",", Insert(", "), || tr!(formula_key_comma))),
        None,
        Some(key("0", Insert("0"), none)),
        Some(key(".", Insert("."), none)),
        Some(key("◀", Left, || tr!(formula_key_left))),
        Some(key("▶", Right, || tr!(formula_key_right))),
        None,
    ],
];

/// `f(z)`: every named function, by family.
const FUNCTIONS: [[Option<Key>; 5]; 4] = [
    [
        Some(key("sin", Wrap("sin(", ")"), || tr!(formula_key_sin))),
        Some(key("cos", Wrap("cos(", ")"), || tr!(formula_key_cos))),
        Some(key("tan", Wrap("tan(", ")"), || tr!(formula_key_tan))),
        Some(key("cotan", Wrap("cotan(", ")"), || tr!(formula_key_cotan))),
        Some(key("exp", Wrap("exp(", ")"), || tr!(formula_key_exp))),
    ],
    [
        Some(key("sinh", Wrap("sinh(", ")"), || tr!(formula_key_sinh))),
        Some(key("cosh", Wrap("cosh(", ")"), || tr!(formula_key_cosh))),
        Some(key("tanh", Wrap("tanh(", ")"), || tr!(formula_key_tanh))),
        Some(key("cotanh", Wrap("cotanh(", ")"), || tr!(formula_key_cotanh))),
        Some(key("log", Wrap("log(", ")"), || tr!(formula_key_log))),
    ],
    [
        Some(key("sqr", Wrap("sqr(", ")"), || tr!(formula_key_sqr))),
        Some(key("sqrt", Wrap("sqrt(", ")"), || tr!(formula_key_sqrt))),
        Some(key("recip", Wrap("recip(", ")"), || tr!(formula_key_recip))),
        Some(key("conj", Wrap("conj(", ")"), || tr!(formula_key_conj))),
        Some(key("flip", Wrap("flip(", ")"), || tr!(formula_key_flip))),
    ],
    [
        Some(key("abs", Wrap("abs(", ")"), || tr!(formula_key_abs))),
        Some(key("cabs", Wrap("cabs(", ")"), || tr!(formula_key_cabs))),
        Some(key("real", Wrap("real(", ")"), || tr!(formula_key_real))),
        Some(key("imag", Wrap("imag(", ")"), || tr!(formula_key_imag))),
        Some(key("ident", Wrap("ident(", ")"), || tr!(formula_key_ident))),
    ],
];

/// `z c p`: every variable, parameter and constant, and the statement helpers.
const NAMES: [[Option<Key>; 5]; 3] = [
    [
        Some(key("z", Insert("z"), || tr!(formula_key_z))),
        Some(key("c", Insert("c"), || tr!(formula_key_c))),
        Some(key("pixel", Insert("pixel"), || tr!(formula_key_pixel))),
        Some(key("π", Insert("pi"), || "π".to_string())),
        Some(key("e", Insert("e"), || tr!(formula_key_euler))),
    ],
    [
        Some(key("p1", Insert("p1"), || tr!(formula_key_param, n = 1))),
        Some(key("p2", Insert("p2"), || tr!(formula_key_param, n = 2))),
        Some(key("p3", Insert("p3"), || tr!(formula_key_param, n = 3))),
        Some(key("p4", Insert("p4"), || tr!(formula_key_param, n = 4))),
        Some(key("p5", Insert("p5"), || tr!(formula_key_param, n = 5))),
    ],
    [
        Some(key("z =", Insert("z = "), || tr!(formula_key_set_z))),
        Some(key("t =", Insert("t = "), || tr!(formula_key_temporary))),
        Some(key("; …", Insert(" ; "), || tr!(formula_key_comment))),
        Some(key("↵", Insert("\n"), || tr!(formula_key_next_statement))),
        Some(key("del", Backspace, || tr!(formula_key_backspace))),
    ],
];

/// Every key of `tab`, as rows (with gaps).
pub(crate) fn rows(tab: Tab) -> Vec<Vec<Option<&'static Key>>> {
    fn collect<const W: usize>(rows: &'static [[Option<Key>; W]]) -> Vec<Vec<Option<&'static Key>>> {
        rows.iter().map(|r| r.iter().map(Option::as_ref).collect()).collect()
    }
    match tab {
        Tab::Basic => collect(&BASIC),
        Tab::Functions => collect(&FUNCTIONS),
        Tab::Names => collect(&NAMES),
    }
}

/// Apply `action` to `text` whose selection is the CHARACTER range `sel` (either order). Returns
/// the new text and the new selection (empty = a cursor).
pub(crate) fn apply(text: &str, sel: (usize, usize), action: Action) -> (String, (usize, usize)) {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let (a, b) = (sel.0.min(sel.1).min(n), sel.0.max(sel.1).min(n));
    let before: String = chars[..a].iter().collect();
    let selected: String = chars[a..b].iter().collect();
    let after: String = chars[b..].iter().collect();
    let len = |s: &str| s.chars().count();
    match action {
        Insert(s) => (format!("{before}{s}{after}"), (a + len(s), a + len(s))),
        Wrap(open, close) => {
            let text = format!("{before}{open}{selected}{close}{after}");
            if a == b {
                (text, (a + len(open), a + len(open)))
            } else {
                let end = a + len(open) + len(&selected) + len(close);
                (text, (end, end))
            }
        }
        Backspace if a < b => (format!("{before}{after}"), (a, a)),
        Backspace if a > 0 => {
            let before: String = chars[..a - 1].iter().collect();
            (format!("{before}{after}"), (a - 1, a - 1))
        }
        Backspace => (text.to_string(), (0, 0)),
        Left => {
            let to = if a < b { a } else { a.saturating_sub(1) };
            (text.to_string(), (to, to))
        }
        Right => {
            let to = if a < b { b } else { (b + 1).min(n) };
            (text.to_string(), (to, to))
        }
    }
}

/// A key pressed for the text field `id`: apply it at the field's STORED cursor (egui keeps it
/// in the field's state across the click that took focus away), store the new cursor, and hand
/// focus back so typing continues where the key left off.
pub(crate) fn press(ctx: &egui::Context, id: egui::Id, text: &mut String, action: Action) {
    let mut state = egui::text_edit::TextEditState::load(ctx, id).unwrap_or_default();
    let end = text.chars().count();
    let sel = state.cursor.char_range().map(|r| (r.secondary.index, r.primary.index)).unwrap_or((end, end));
    let (new_text, (a, b)) = apply(text, sel, action);
    *text = new_text;
    state
        .cursor
        .set_char_range(Some(egui::text::CCursorRange::two(egui::text::CCursor::new(a), egui::text::CCursor::new(b))));
    state.store(ctx, id);
    ctx.memory_mut(|m| m.request_focus(id));
}

/// Draw the keypad's tabs and keys; returns the key pressed, if any.
pub(crate) fn show(ui: &mut egui::Ui, tab: &mut Tab) -> Option<Action> {
    let mut pressed = None;
    ui.horizontal(|ui| {
        for t in Tab::ALL {
            ui.selectable_value(tab, t, t.label()).on_hover_text(t.hint());
        }
    });
    let size = egui::vec2(46.0, 26.0);
    egui::Grid::new(("formula_keypad", *tab as u8)).spacing(egui::vec2(4.0, 4.0)).show(ui, |ui| {
        for row in rows(*tab) {
            for k in row {
                match k {
                    Some(k) => {
                        let text = egui::RichText::new(k.label).monospace();
                        let b = ui.add_sized(size, egui::Button::new(text));
                        let hint = (k.hint)();
                        let b = if hint.is_empty() { b } else { b.on_hover_text(hint) };
                        if b.clicked() {
                            pressed = Some(k.action);
                        }
                    }
                    None => {
                        ui.allocate_space(egui::vec2(size.x * 0.4, size.y));
                    }
                }
            }
            ui.end_row();
        }
    });
    pressed
}

#[cfg(test)]
#[path = "../../tests/unit/ui/formula_keypad/tests.rs"]
mod tests;
