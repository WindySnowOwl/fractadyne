use super::*;
use crate::i18n::{Arg, Punctuation, id};

/// A small German file in the shape of the real ones (the language data, two messages).
const GERMAN: &str = "### tag: de\n### revision: 7\n\n\
language-name = Deutsch\n\
language-english-name = German\n\
language-script = Latn\n\
number-decimal = ,\n\
number-group = .\n\
welcome-title = Willkommen bei Fractadyne\n\
language-dropped =\n    { $count ->\n        [one] eine Meldung zu prüfen\n       *[other] { $count } Meldungen zu prüfen\n    }\n";

#[test]
fn a_language_file_translates_what_it_has_and_english_answers_the_rest() {
    let c = Catalog::from_text(GERMAN, "de").unwrap();
    assert_eq!(c.tag, "de");
    assert_eq!(c.name, "Deutsch");
    assert_eq!(c.english_name, "German");
    assert_eq!(c.revision, 7);
    assert_eq!(c.number, Punctuation { decimal: ',', group: '.' });
    assert_eq!(c.text(id::welcome_title), "Willkommen bei Fractadyne");
    // Not in the file: English.
    assert_eq!(c.text(id::welcome_get_started), "Get started");
    assert_eq!(c.translated, 2);
    assert!(c.dropped.is_empty());
    assert_eq!(c.parse_errors, 0);
}

#[test]
fn a_message_that_fails_a_check_falls_back_alone() {
    let text = format!(
        "{GERMAN}welcome-intro = Siehe https://example.com\n\
         welcome-get-started = Los {{ $path }}\n\
         welcome-open = {}\n\
         welcome-theme = Thema\n",
        "x".repeat(200)
    );
    let c = Catalog::from_text(&text, "de").unwrap();
    let why = |m: &str| c.dropped.iter().find(|d| d.id == m).map(|d| d.why);
    assert_eq!(why("welcome-intro"), Some("it contains a web address"));
    assert_eq!(why("welcome-get-started"), Some("it uses an argument the English message does not have"));
    assert_eq!(why("welcome-open"), Some("it is more than four times the length of the English"));
    // Each falls back to English; the good one is used.
    assert_eq!(c.text(id::welcome_get_started), "Get started");
    assert_eq!(c.text(id::welcome_theme), "Thema");
}

#[test]
fn an_unreadable_entry_does_not_lose_the_rest_of_the_file() {
    let text = format!("{GERMAN}welcome-theme = {{ broken\nwelcome-open = Öffnen\n");
    let c = Catalog::from_text(&text, "de").unwrap();
    assert!(c.parse_errors > 0);
    assert_eq!(c.text(id::welcome_open), "Öffnen");
    assert_eq!(c.text(id::welcome_theme), "Theme");
}

#[test]
fn numbers_in_a_message_take_the_languages_punctuation() {
    let c = Catalog::from_text(GERMAN, "de").unwrap();
    // The punctuation is the language IN USE's (a process-wide setting), so check the catalog's own
    // choice of plural form here and the punctuation through `number_with` (tests/unit/i18n).
    assert_eq!(c.format(id::language_dropped, &[("count", Arg::Num(1.0))]), "eine Meldung zu prüfen");
    assert!(c.format(id::language_dropped, &[("count", Arg::Num(3.0))]).ends_with("Meldungen zu prüfen"));
}

#[test]
fn a_file_without_its_own_name_is_known_by_its_tag_never_as_english() {
    let c = Catalog::from_text("welcome-title = Bienvenue\n", "fr").unwrap();
    assert_eq!(c.name, "fr");
    assert_eq!(c.english_name, "fr");
    assert_eq!(c.number, Punctuation::default());
}

#[test]
fn a_file_too_large_or_with_a_bad_tag_is_refused() {
    assert!(Catalog::from_text(&"#".repeat(MAX_FILE_BYTES + 1), "de").is_err());
    assert!(Catalog::from_text(GERMAN, "not a tag!").is_err());
}

#[test]
fn cleaning_removes_controls_and_reordering_marks_and_keeps_language() {
    let s = "\u{FEFF}a\r\nb\rc\u{0007}d\u{202E}e\u{2066}f\u{2069}\u{200B}g\u{200D}h„i“–\u{00A0}\u{202F}j\tk";
    assert_eq!(clean_text(s), "a\nb\ncdef\u{200B}g\u{200D}h„i“–\u{00A0}\u{202F}j\tk");
}

#[test]
fn the_publish_stamp_is_read_from_the_files_top() {
    let st = Stamp::read("### fractadyne-language-file: 1\n### tag: zh-Hans\n### revision: 42\n\nlanguage-name = 简体中文\n### revision: 99\n");
    assert_eq!(st, Stamp { tag: Some("zh-Hans".into()), revision: 42 });
    assert_eq!(Stamp::read("welcome-title = x\n"), Stamp::default());
}

#[test]
fn the_pseudo_language_brackets_and_lengthens_every_message() {
    let c = Catalog::pseudo();
    let s = c.text(id::welcome_get_started);
    assert!(s.starts_with('[') && s.ends_with(']'), "{s}");
    assert!(s.chars().count() > "Get started".len() + 2);
    assert_eq!(c.coverage(), 100);
}

/// Every language file in the repository loads as the program loads it: nothing unreadable, no
/// message set aside, its own name and a script. The gate a translation's pull request passes (the
/// CLI's `--check-language` prints the same report for one file).
#[test]
fn every_shipped_language_loads_with_nothing_set_aside() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../i18n");
    for entry in std::fs::read_dir(&dir).expect("i18n/").flatten() {
        let tag = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path().join("fractadyne.ftl");
        if tag == "en" || !path.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("UTF-8");
        let c = Catalog::from_text(&text, &tag).unwrap_or_else(|e| panic!("{tag}: {e}"));
        assert_eq!(c.parse_errors, 0, "{tag}: entries that do not read");
        let dropped: Vec<String> = c.dropped.iter().map(|d| format!("{}: {}", d.id, d.why)).collect();
        assert!(dropped.is_empty(), "{tag}: set aside:
{}", dropped.join("
"));
        assert_ne!(c.name, tag, "{tag}: no language-name");
        assert!(["Latn", "Cyrl", "Jpan", "Hans"].contains(&c.script.as_str()), "{tag}: script {}", c.script);
        assert!(["draft", "reviewed"].contains(&c.status.as_str()), "{tag}: language-status {:?}", c.status);
        assert!(c.translated * 2 > interface_messages(), "{tag}: {} translated", c.translated);
    }
}

/// A space as the digit-group separator survives loading: it IS the separator (French's narrow
/// no-break space, Ukrainian's no-break space), not padding to trim away. And the status a draft
/// declares is read.
#[test]
fn a_space_group_separator_is_kept() {
    let fr = "language-name = Français\nlanguage-script = Latn\nnumber-decimal = ,\nnumber-group = { \"\\u202F\" }\nlanguage-status = draft\n";
    let c = Catalog::from_text(fr, "fr").unwrap();
    assert_eq!(c.number, Punctuation { decimal: ',', group: '\u{202F}' });
    assert_eq!(c.status, "draft");
    let uk = "language-name = Українська\nlanguage-script = Cyrl\nnumber-decimal = ,\nnumber-group = \u{00A0}\n";
    assert_eq!(Catalog::from_text(uk, "uk").unwrap().number.group, '\u{00A0}');
}

/// The pseudo-language accents what a message writes, not what it is given: a path or a size passed
/// in shows as it is, inside the brackets.
#[test]
fn the_pseudo_language_leaves_arguments_as_they_are() {
    let c = Catalog::pseudo();
    let s = c.format(id::export_saved, &[("size", Arg::from("1920×1080")), ("path", Arg::from("C:/pictures/a.png"))]);
    assert!(s.starts_with('[') && s.ends_with(']'), "{s}");
    assert!(s.contains("C:/pictures/a.png"), "the path was changed: {s}");
    assert!(s.contains('é') || s.contains('å'), "the message's own text is not accented: {s}");
}
