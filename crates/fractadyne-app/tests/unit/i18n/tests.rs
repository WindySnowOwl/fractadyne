use super::*;
use fluent_bundle::FluentResource;
use fluent_syntax::ast;

#[test]
fn digits_group_in_threes_with_the_languages_separator() {
    assert_eq!(group_digits("0", ','), "0");
    assert_eq!(group_digits("999", ','), "999");
    assert_eq!(group_digits("1000", ','), "1,000");
    assert_eq!(group_digits("1234567", '.'), "1.234.567");
    assert_eq!(group_digits("1234567", '\u{202F}'), "1\u{202F}234\u{202F}567");
}

#[test]
fn numbers_take_the_languages_decimal_mark() {
    let en = Punctuation::default();
    let de = Punctuation { decimal: ',', group: '.' };
    assert_eq!(number_with(1234567.0, en), "1,234,567");
    assert_eq!(number_with(1234567.0, de), "1.234.567");
    assert_eq!(number_with(-12345.5, en), "-12,345.5");
    assert_eq!(number_with(-12345.5, de), "-12.345,5");
    assert_eq!(number_with(0.25, de), "0,25");
    assert_eq!(number_with(f64::NAN, de), "NaN");
}

/// build.rs scans the English file line by line for the message IDs and their `$variables`; Fluent's
/// own parser must agree, message for message, or the generated constants point at the wrong text.
#[test]
fn the_generated_ids_are_the_english_files_messages_in_order() {
    let r = FluentResource::try_new(ENGLISH.to_owned()).unwrap_or_else(|(_, e)| panic!("the English file does not parse: {e:?}"));
    let mut names = Vec::new();
    let mut vars = Vec::new();
    for e in r.entries() {
        if let ast::Entry::Message(m) = e {
            names.push(m.id.name);
            let mut v = Vec::new();
            if let Some(p) = &m.value {
                catalog::pattern_vars(p, &mut v);
            }
            for a in &m.attributes {
                catalog::pattern_vars(&a.value, &mut v);
            }
            vars.push(v);
        }
    }
    assert_eq!(names, id::NAMES.to_vec());
    for (i, v) in vars.iter().enumerate() {
        let mut want = id::VARS[i].to_vec();
        let mut got = v.clone();
        want.sort();
        got.sort();
        assert_eq!(got, want, "variables of {}", id::NAMES[i]);
    }
}

/// Every English message formats with the arguments it declares: no error, no raw placeholder.
#[test]
fn every_english_message_formats() {
    let en = english();
    for (i, name) in id::NAMES.iter().enumerate() {
        let args: Vec<(&'static str, Arg)> = id::VARS[i].iter().map(|v| (*v, Arg::Num(2.0))).collect();
        let s = en.format(Msg(i as u16), &args);
        assert_ne!(s, *name, "{name} did not format");
        assert!(!s.is_empty(), "{name} is empty");
        assert!(!s.contains('{') && !s.contains('}'), "{name}: {s}");
    }
}

#[test]
fn english_describes_itself() {
    let en = english();
    assert_eq!(en.tag, "en");
    assert_eq!(en.name, "English");
    assert_eq!(en.script, "Latn");
    assert_eq!(en.number, Punctuation::default());
    assert_eq!(en.coverage(), 100);
    assert_eq!(en.text(id::welcome_title), "Welcome to Fractadyne");
}

#[test]
fn a_plural_message_chooses_its_form() {
    let en = english();
    assert_eq!(en.format(id::language_dropped, &[("count", Arg::Num(1.0))]), "one message needs checking");
    assert_eq!(en.format(id::language_dropped, &[("count", Arg::Num(1234.0))]), "1,234 messages need checking");
}

#[test]
fn a_missing_argument_shows_as_a_gap_never_as_a_bare_id() {
    // Formatting reports the missing argument; the message is still its words, with the gap shown.
    let s = english().format(id::language_coverage, &[]);
    assert_ne!(s, "language-coverage");
    assert!(s.ends_with('%'), "{s}");
}

/// A blank line inside a message is a paragraph break, as the tooltips' "\n\n" were.
#[test]
fn a_blank_line_in_a_message_is_a_paragraph_break() {
    let s = english().format(id::menu_show_set_hover, &[("name", Arg::from("Mandelbrot"))]);
    assert_eq!(s, "The parameter plane — the Mandelbrot set itself.\n\nEvery point of it is a different Julia set; this is the map of all of them.");
}
