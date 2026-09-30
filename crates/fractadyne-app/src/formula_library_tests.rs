use super::*;

fn entry(name: &str, source: &str, params: &[(&str, &str)]) -> SavedFormula {
    SavedFormula {
        name: name.into(),
        source: source.into(),
        params: params.iter().map(|(re, im)| [re.to_string(), im.to_string()]).collect(),
    }
}

/// A directory of its own under the temp dir, removed when dropped.
struct Scratch(PathBuf);
impl Scratch {
    fn new(tag: &str) -> Scratch {
        let dir = std::env::temp_dir().join(format!("fd-formula-lib-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn tidy_repairs_line_breaks_and_names_rather_than_rejecting() {
    let e = entry("  Two\nlines  ", "t = sqr(z)\r\nz = t + c\rz = z\u{0}", &[(" 0.5 ", "0")]).tidy().unwrap();
    assert_eq!(e.source, "t = sqr(z)\nz = t + c\nz = z", "CR LF and a lone CR both become \\n; NUL dropped");
    assert_eq!(e.name, "Two lines");
    assert_eq!(e.params, vec![["0.5".to_string(), "0".to_string()]]);
    // A blank name is taken from the first statement that is not a comment.
    let e = entry("", "; my cubic\nz = z^3 + c", &[]).tidy().unwrap();
    assert_eq!(e.name, "z = z^3 + c");
    // A source that no longer parses is KEPT (it may be a newer build's); an empty one is not.
    assert!(entry("x", "z = z^^2", &[]).tidy().is_some());
    assert!(entry("x", " \r\n\t", &[]).tidy().is_none());
    assert!(entry("x", &"z".repeat(SOURCE_MAX + 1), &[]).tidy().is_none());
    // At most MAX_PARAMS parameters.
    let many: Vec<(&str, &str)> = vec![("1", "0"); MAX_PARAMS + 3];
    assert_eq!(entry("x", "z = z^2 + c", &many).tidy().unwrap().params.len(), MAX_PARAMS);
}

#[test]
fn the_same_formula_is_the_same_source_and_values_whatever_the_name() {
    let a = entry("A", "z = z^2 + p1*z + c", &[("0.25", "-0.1")]);
    assert!(a.same_formula(&entry("B", "z = z^2 + p1*z + c", &[("0.250", "-1e-1")])), "numbers compare as values");
    assert!(!a.same_formula(&entry("A", "z = z^2 + p1*z + c", &[("0.25", "0.1")])));
    assert!(!a.same_formula(&entry("A", "z = z^2 + p1*z - c", &[("0.25", "-0.1")])));
    // A parameter missing on one side reads as 0.
    assert!(entry("A", "z = z^2 + c", &[]).same_formula(&entry("A", "z = z^2 + c", &[("0", "0")])));
}

#[test]
fn saving_under_a_name_replaces_that_entry_and_keeps_the_list_sorted() {
    let mut list = Vec::new();
    assert_eq!(upsert(&mut list, entry("sine", "z = sin(z) + c", &[])), Some(false));
    assert_eq!(upsert(&mut list, entry("Cubic", "z = z^3 + c", &[])), Some(false));
    assert_eq!(upsert(&mut list, entry("cubic", "z = z^3 - c", &[])), Some(false), "names are case-sensitive keys");
    assert_eq!(upsert(&mut list, entry("Cubic", "z = z^3 - p1*z + c", &[("0.5", "0")])), Some(true));
    let names: Vec<&str> = list.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["Cubic", "cubic", "sine"]);
    assert_eq!(list[0].source, "z = z^3 - p1*z + c", "updated in place, not appended");
    assert_eq!(upsert(&mut list, entry("empty", "  ", &[])), None);
    assert_eq!(list.len(), 3);
}

#[test]
fn an_import_never_overwrites_the_library() {
    let mut list = vec![entry("Cubic", "z = z^3 + c", &[]), entry("Sine", "z = sin(z) + c", &[])];
    let incoming = vec![
        entry("My sine", "z = sin(z) + c", &[]),     // already held, under another name
        entry("Cubic", "z = z^3 - p1*z + c", &[("0.5", "0")]), // a different formula, name taken
        entry("Cubic", "z = z^3 - z + c", &[]),      // and again: (2) is taken by then
        entry("Burning", "z = (abs(real(z)) + flip(abs(imag(z))))^2 + c", &[]),
        entry("Blank", "", &[]),
        entry("Burning again", "z = (abs(real(z)) + flip(abs(imag(z))))^2 + c", &[]), // same file, twice
    ];
    let report = merge(&mut list, incoming);
    assert_eq!(
        report,
        MergeReport {
            added: 3,
            duplicates: 2,
            renamed: vec![("Cubic".into(), "Cubic (2)".into()), ("Cubic".into(), "Cubic (3)".into())],
            skipped: 1,
        }
    );
    assert_eq!(list.iter().find(|f| f.name == "Cubic").unwrap().source, "z = z^3 + c", "the user's own is untouched");
    let names: Vec<&str> = list.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["Burning", "Cubic", "Cubic (2)", "Cubic (3)", "Sine"]);
    assert_eq!(
        report.sentence("shared.toml"),
        "Imported 3 formulas from \"shared.toml\": 2 formulas were already in the library; \
         2 renamed where the name was taken; 1 entry was empty or too long, skipped."
    );
    let again = merge(&mut list, vec![entry("Cubic", "z = z^3 + c", &[])]);
    assert_eq!(again.sentence("x.toml"), "Nothing new in \"x.toml\": 1 formula was already in the library.");
    let one = merge(&mut list, vec![entry("Sine", "z = sin(z)*c", &[])]);
    assert_eq!(one.sentence("y.toml"), "Imported 1 formula from \"y.toml\": \"Sine\" was taken, so it is \"Sine (2)\".");
}

#[test]
fn a_formula_file_round_trips_every_character() {
    let list = vec![
        entry("Hybrid \"square\"", "t = sqr(z)\nz = t + p1*conj(t) + c ; a comment", &[("0.25", "0")]),
        entry("Two params", "z = z^p1 + p2", &[("2.2", "0.3"), ("-0.1", "1e-3")]),
        entry("Ünïcode ∑", "z = exp(z) + c", &[]),
    ];
    let text = file_text(&list);
    assert!(text.starts_with("# Fractadyne custom formulas."));
    assert!(text.contains("format = \"fractadyne-formulas\""));
    assert_eq!(parse_file(&text).unwrap(), list);
    // A hand-written file needs neither the header nor the version.
    let hand = "[[formula]]\nname = \"Mine\"\nsource = '''\nz = z^2\nz = z + c'''\n";
    assert_eq!(parse_file(hand).unwrap(), vec![entry("Mine", "z = z^2\nz = z + c", &[])]);
}

#[test]
fn a_file_that_is_not_a_formula_file_says_so() {
    assert!(parse_file("this is = not toml [").unwrap_err().starts_with("not a formula file ("));
    assert_eq!(
        parse_file("format = \"fractadyne-gradients\"\n[[formula]]\nname = \"x\"\nsource = \"z\"\n").unwrap_err(),
        "a \"fractadyne-gradients\" file, not a formula file"
    );
    assert!(parse_file("title = \"a session\"\n").unwrap_err().starts_with("no formulas in it"));
    // A newer file's unknown fields are ignored: it imports what this build reads.
    let newer = "format = \"fractadyne-formulas\"\nversion = 9\n[[formula]]\nname = \"x\"\nsource = \"z = z^2 + c\"\nauthor = \"someone\"\n";
    assert_eq!(parse_file(newer).unwrap(), vec![entry("x", "z = z^2 + c", &[])]);
}

#[test]
fn the_library_saves_and_loads_through_the_disk() {
    let dir = Scratch::new("round-trip");
    let path = dir.0.join("formulas.toml");
    assert_eq!(load_from(&path), (Vec::new(), None), "no file yet: an empty library, nothing to report");
    let list = vec![entry("A", "z = z^2 + c", &[]), entry("B", "z = z^3 + p1", &[("0.5", "0")])];
    save_to(&path, &list).unwrap();
    assert!(!path.with_extension("toml.tmp").exists(), "the temp file was renamed over the library");
    assert_eq!(load_from(&path), (list.clone(), None));
    // Saving again replaces the file whole.
    save_to(&path, &list[..1]).unwrap();
    assert_eq!(load_from(&path).0, list[..1]);
}

#[test]
fn an_unreadable_library_is_moved_aside_never_overwritten() {
    let dir = Scratch::new("unreadable");
    let path = dir.0.join("formulas.toml");
    let garbage = "[[formula]]\nname = \"half a formula\nsource = ";
    std::fs::write(&path, garbage).unwrap();
    let (list, note) = load_from(&path);
    assert!(list.is_empty());
    let note = note.expect("the user is told");
    assert!(note.contains("formulas.unreadable.toml"), "{note}");
    assert!(!path.exists(), "moved, so the next save cannot replace it");
    assert_eq!(std::fs::read_to_string(dir.0.join("formulas.unreadable.toml")).unwrap(), garbage);
    // A second bad file does not replace the first one set aside.
    std::fs::write(&path, "also bad [").unwrap();
    let note = load_from(&path).1.unwrap();
    assert!(note.contains("formulas.unreadable-2.toml"), "{note}");
    assert_eq!(std::fs::read_to_string(dir.0.join("formulas.unreadable.toml")).unwrap(), garbage);
}

#[test]
fn a_row_shows_the_source_on_one_line() {
    assert_eq!(entry("x", "t = sqr(z)\n\n  z = t + c  ", &[]).one_line(), "t = sqr(z), z = t + c");
}
