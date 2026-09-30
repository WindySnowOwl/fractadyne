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
    pub(crate) hint: &'static str,
}

const fn key(label: &'static str, action: Action, hint: &'static str) -> Key {
    Key { label, action, hint }
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
    pub(crate) fn label(self) -> &'static str {
        match self {
            Tab::Basic => "123",
            Tab::Functions => "f(z)",
            Tab::Names => "names",
        }
    }
    pub(crate) fn hint(self) -> &'static str {
        match self {
            Tab::Basic => "Numbers, operators and the most used names",
            Tab::Functions => "Functions",
            Tab::Names => "Every variable, parameter and constant, and statement syntax",
        }
    }
}

use Action::*;

/// `123`: two blocks, as on a calculator — names and templates on the left, digits and operators
/// on the right. Rows are laid out left to right, `None` a gap between the blocks.
const BASIC: [[Option<Key>; 10]; 4] = [
    [
        Some(key("z", Insert("z"), "The iterate")),
        Some(key("c", Insert("c"), "The pixel (the Julia constant in Julia mode)")),
        Some(key("p1", Insert("p1"), "Parameter 1 — set in the fields below")),
        Some(key("p2", Insert("p2"), "Parameter 2")),
        None,
        Some(key("7", Insert("7"), "")),
        Some(key("8", Insert("8"), "")),
        Some(key("9", Insert("9"), "")),
        Some(key("×", Insert("*"), "Multiply")),
        Some(key("÷", Insert("/"), "Divide")),
    ],
    [
        Some(key("□²", Insert("^2"), "Square (after a value: z^2)")),
        // Not "□ⁿ": the UI font has no superscript n (it drew a box).
        Some(key("□^", Insert("^"), "Power: an integer, real or complex exponent")),
        Some(key("√", Wrap("sqrt(", ")"), "Square root")),
        Some(key("|□|", Wrap("|", "|"), "SQUARED modulus |z|² — as in Fractint")),
        None,
        Some(key("4", Insert("4"), "")),
        Some(key("5", Insert("5"), "")),
        Some(key("6", Insert("6"), "")),
        Some(key("+", Insert(" + "), "Add")),
        Some(key("−", Insert(" - "), "Subtract (or negate)")),
    ],
    [
        Some(key("(", Insert("("), "")),
        Some(key(")", Insert(")"), "")),
        Some(key("=", Insert(" = "), "Assign: z = …, or a temporary t = …")),
        Some(key("↵", Insert("\n"), "Next statement (a comma works too)")),
        None,
        Some(key("1", Insert("1"), "")),
        Some(key("2", Insert("2"), "")),
        Some(key("3", Insert("3"), "")),
        // Not "⌫": the UI font has no erase glyph (it drew a box), and the icon font's DELETE is a bin.
        Some(key("del", Backspace, "Delete the character before the cursor (or the selection)")),
        None,
    ],
    [
        Some(key("π", Insert("pi"), "π")),
        Some(key("e", Insert("e"), "Euler's number")),
        Some(key("(a, b)", Insert("(0.5, 0.5)"), "A complex constant: (real, imaginary)")),
        Some(key(",", Insert(", "), "Separates statements, or a complex constant's parts")),
        None,
        Some(key("0", Insert("0"), "")),
        Some(key(".", Insert("."), "")),
        Some(key("◀", Left, "Cursor left")),
        Some(key("▶", Right, "Cursor right")),
        None,
    ],
];

/// `f(z)`: every named function, by family.
const FUNCTIONS: [[Option<Key>; 5]; 4] = [
    [
        Some(key("sin", Wrap("sin(", ")"), "Sine")),
        Some(key("cos", Wrap("cos(", ")"), "Cosine")),
        Some(key("tan", Wrap("tan(", ")"), "Tangent")),
        Some(key("cotan", Wrap("cotan(", ")"), "Cotangent")),
        Some(key("exp", Wrap("exp(", ")"), "Exponential eᶻ")),
    ],
    [
        Some(key("sinh", Wrap("sinh(", ")"), "Hyperbolic sine")),
        Some(key("cosh", Wrap("cosh(", ")"), "Hyperbolic cosine")),
        Some(key("tanh", Wrap("tanh(", ")"), "Hyperbolic tangent")),
        Some(key("cotanh", Wrap("cotanh(", ")"), "Hyperbolic cotangent")),
        Some(key("log", Wrap("log(", ")"), "Natural logarithm (principal branch)")),
    ],
    [
        Some(key("sqr", Wrap("sqr(", ")"), "Square: sqr(z) = z²")),
        Some(key("sqrt", Wrap("sqrt(", ")"), "Square root (principal)")),
        Some(key("recip", Wrap("recip(", ")"), "Reciprocal 1/z")),
        Some(key("conj", Wrap("conj(", ")"), "Complex conjugate")),
        Some(key("flip", Wrap("flip(", ")"), "Swap real and imaginary parts")),
    ],
    [
        Some(key("abs", Wrap("abs(", ")"), "Absolute value of BOTH parts: |x| + i|y|")),
        Some(key("cabs", Wrap("cabs(", ")"), "Modulus |z| (a real number)")),
        Some(key("real", Wrap("real(", ")"), "Real part")),
        Some(key("imag", Wrap("imag(", ")"), "Imaginary part, as a real number")),
        Some(key("ident", Wrap("ident(", ")"), "Identity (Fractint's fn placeholder)")),
    ],
];

/// `z c p`: every variable, parameter and constant, and the statement helpers.
const NAMES: [[Option<Key>; 5]; 3] = [
    [
        Some(key("z", Insert("z"), "The iterate")),
        Some(key("c", Insert("c"), "The pixel (the Julia constant in Julia mode)")),
        Some(key("pixel", Insert("pixel"), "The same as c (Fractint's name)")),
        Some(key("π", Insert("pi"), "π")),
        Some(key("e", Insert("e"), "Euler's number")),
    ],
    [
        Some(key("p1", Insert("p1"), "Parameter 1")),
        Some(key("p2", Insert("p2"), "Parameter 2")),
        Some(key("p3", Insert("p3"), "Parameter 3")),
        Some(key("p4", Insert("p4"), "Parameter 4")),
        Some(key("p5", Insert("p5"), "Parameter 5")),
    ],
    [
        Some(key("z =", Insert("z = "), "Start a statement setting z")),
        Some(key("t =", Insert("t = "), "A temporary: any new name works (t, w, a2, …)")),
        Some(key("; …", Insert(" ; "), "A comment, to the end of the line")),
        Some(key("↵", Insert("\n"), "Next statement")),
        Some(key("del", Backspace, "Delete the character before the cursor (or the selection)")),
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
                        let b = if k.hint.is_empty() { b } else { b.on_hover_text(k.hint) };
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
mod tests;
