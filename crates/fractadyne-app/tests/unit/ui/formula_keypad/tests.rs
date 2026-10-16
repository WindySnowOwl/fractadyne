use super::*;
use fractadyne_core::ir::parse::parse;

/// Every key produces something the parser accepts: a name alone parses as a step, an operator
/// between two values, a function around a value. The keypad cannot advertise a name the
/// language lacks.
#[test]
fn every_key_is_the_parsers_vocabulary() {
    let mut keys = 0;
    for tab in Tab::ALL {
        for row in rows(tab) {
            for k in row.into_iter().flatten() {
                keys += 1;
                let src = match k.action {
                    // Assignment: after a name ("=" alone), or a whole statement head ("z = ",
                    // "t = "); a temporary needs a final z statement to be a step.
                    Insert(" = ") => "t = z\nz = t + c".to_string(),
                    Insert(s) if s.contains('=') => format!("{s}z\nz = z + c"),
                    Insert(s) if s.trim() == ";" => format!("z + c {s}comment"),
                    Insert("\n") | Insert(", ") => format!("t = z{}z = t + c", k_text(k.action)),
                    Insert(s) if s.trim_start().starts_with('^') => format!("z{s}{}", if s == "^" { "3" } else { "" }),
                    Insert(s) if matches!(s.trim(), "+" | "-" | "*" | "/") => format!("z{s}c"),
                    Insert(s) if s.chars().all(|ch| ch.is_ascii_digit() || ch == '.') => format!("z + {s}0"),
                    Insert("(") => "(z + c".to_string() + ")",
                    Insert(")") => "(z + c".to_string() + ")",
                    Insert(s) => format!("z + {s}"),
                    Wrap(open, close) => format!("{open}z{close} + c"),
                    Backspace | Left | Right => continue,
                };
                parse(&src).unwrap_or_else(|e| panic!("key {:?} gives {src:?}, which does not parse: {e}", k.label));
            }
        }
    }
    assert!(keys >= 60, "the keypad lost keys ({keys})");
}

fn k_text(a: Action) -> &'static str {
    match a {
        Insert(s) => s,
        _ => "",
    }
}

/// Every named function the parser knows has a key (the reverse direction).
#[test]
fn every_parser_function_has_a_key() {
    let labels: Vec<&str> = Tab::ALL.iter().flat_map(|&t| rows(t)).flatten().flatten().map(|k| k.label).collect();
    for name in ["exp", "log", "sqrt", "sin", "cos", "tan", "sinh", "cosh", "tanh", "sqr", "abs", "conj", "real",
        "imag", "cabs", "flip", "recip", "ident", "cotan", "cotanh", "p1", "p2", "p3", "p4", "p5", "pixel"]
    {
        assert!(labels.contains(&name), "no key for {name}");
    }
}

/// The glue a click runs, against a real (headless) egui context: the field's stored selection is
/// what a key acts on, and the cursor it leaves is stored back for the next key — so a sequence of
/// presses composes like typing.
#[test]
fn presses_use_and_update_the_fields_stored_cursor() {
    let ctx = egui::Context::default();
    let id = egui::Id::new("formula_source_test");
    let mut text = "z = z + c".to_string();
    // The user selected the second `z` (characters 4..5), then clicked `sin`.
    let mut state = egui::text_edit::TextEditState::default();
    state.cursor.set_char_range(Some(egui::text::CCursorRange::two(
        egui::text::CCursor::new(4),
        egui::text::CCursor::new(5),
    )));
    state.store(&ctx, id);
    press(&ctx, id, &mut text, Wrap("sin(", ")"));
    assert_eq!(text, "z = sin(z) + c");
    // Then `^2` lands after the wrapped call, where the cursor was left…
    press(&ctx, id, &mut text, Insert("^2"));
    assert_eq!(text, "z = sin(z)^2 + c");
    // …and `del` removes it again, one character at a time.
    press(&ctx, id, &mut text, Backspace);
    press(&ctx, id, &mut text, Backspace);
    assert_eq!(text, "z = sin(z) + c");
    // With no stored state (the field never touched), a key appends at the end.
    let fresh = egui::Id::new("formula_source_fresh");
    let mut t = "z^2".to_string();
    press(&ctx, fresh, &mut t, Insert(" + c"));
    assert_eq!(t, "z^2 + c");
}

#[test]
fn keys_edit_at_the_cursor_and_wrap_a_selection() {
    // Insert at the cursor, cursor after.
    assert_eq!(apply("z + c", (1, 1), Insert("^2")), ("z^2 + c".into(), (3, 3)));
    // Insert replaces a selection (either order).
    assert_eq!(apply("z + c", (4, 5), Insert("p1")), ("z + p1".into(), (6, 6)));
    assert_eq!(apply("z + c", (5, 4), Insert("p1")), ("z + p1".into(), (6, 6)));
    // A function wraps the selection…
    assert_eq!(apply("z^2 + c", (0, 3), Wrap("sin(", ")")), ("sin(z^2) + c".into(), (8, 8)));
    // …or, with none, leaves the cursor between its parentheses.
    assert_eq!(apply("z = ", (4, 4), Wrap("cos(", ")")), ("z = cos()".into(), (8, 8)));
    // Backspace: the selection, else the character before the cursor, else nothing.
    assert_eq!(apply("z + c", (2, 5), Backspace), ("z ".into(), (2, 2)));
    assert_eq!(apply("z + c", (5, 5), Backspace), ("z + ".into(), (4, 4)));
    assert_eq!(apply("z + c", (0, 0), Backspace), ("z + c".into(), (0, 0)));
    // Arrows collapse a selection to its side, else move one character, within bounds.
    assert_eq!(apply("abc", (1, 1), Left).1, (0, 0));
    assert_eq!(apply("abc", (0, 0), Left).1, (0, 0));
    assert_eq!(apply("abc", (3, 3), Right).1, (3, 3));
    assert_eq!(apply("abc", (0, 2), Right).1, (2, 2));
    assert_eq!(apply("abc", (0, 2), Left).1, (0, 0));
    // Character (not byte) positions: multi-byte text before the cursor.
    assert_eq!(apply("π·z", (3, 3), Insert("^2")), ("π·z^2".into(), (5, 5)));
    // A cursor past the end (stale state) lands at the end.
    assert_eq!(apply("z", (9, 9), Insert("+c")), ("z+c".into(), (3, 3)));
}
