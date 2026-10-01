use super::*;
use fractadyne_core::ir::parse::parse;

/// Feed keys: characters as typed; `{Left}`, `{S-Right}` (with Shift), `{Bksp}`, `{Del}`,
/// `{Enter}`, `{Home}`, `{End}`, `{Tab}`, `{Up}`, `{Down}`, `{All}`, `{Undo}`, `{Redo}`.
fn keys(e: &mut Editor, ks: &str) {
    let mut rest = ks;
    while let Some(ch) = rest.chars().next() {
        if ch != '{' {
            e.type_char(ch);
            rest = &rest[ch.len_utf8()..];
            continue;
        }
        let end = rest.find('}').expect("a closing brace");
        let k = &rest[1..end];
        rest = &rest[end + 1..];
        match k {
            "Left" => e.step(Dir::Left, false),
            "Right" => e.step(Dir::Right, false),
            "S-Left" => e.step(Dir::Left, true),
            "S-Right" => e.step(Dir::Right, true),
            "Bksp" => {
                e.backspace();
            }
            "Del" => {
                e.delete();
            }
            "Enter" => {
                e.enter();
            }
            "Home" => e.home_end(false, false),
            "End" => e.home_end(true, false),
            "Tab" => {
                e.tab(false);
            }
            "Up" | "Down" => match e.vertical(k == "Up") {
                Vertical::To(c) => e.set_caret(c, false),
                Vertical::Nearest(c) => e.set_caret(Caret { pos: 0, ..c }, false),
                Vertical::Stay => {}
            },
            "All" => e.select_all(),
            "Undo" => {
                e.undo();
            }
            "Redo" => {
                e.redo();
            }
            other => panic!("unknown key {other}"),
        }
    }
}

fn typed_into(src: &str, ks: &str) -> String {
    let mut e = Editor::new(src);
    keys(&mut e, ks);
    let out = e.doc.source();
    assert_eq!(out, e.synced, "the printed text is what the dialog gets");
    out
}

fn typed(ks: &str) -> String {
    typed_into("", ks)
}

/// Typing builds the structures the design's table says (§4.9), and prints the text a user of the
/// text field would have typed.
#[test]
fn typing_builds_the_structures() {
    let cases = [
        ("z=z^2+c", "z = z^2 + c"),
        // `+` at the end of an exponent steps out of it; `/` takes the term before it.
        ("z^2+c/z", "z^2 + c/z"),
        ("a+b/c", "a + b/c"),
        ("-b/c", "-b/c"),
        ("a*b/c", "a*b/c"),
        ("a+-b/c", "a + -b/c"),
        ("/1{Tab}2", "1/2"),
        // An exponent: its sign kept when typed first; an exponent typed in an exponent nests,
        // one typed after an exponent raises the power.
        ("z^-1", "z^-1"),
        ("z^2^3", "z^2^3"),
        ("z^2{Right}^3", "(z^2)^3"),
        ("^2", "2"),
        // Parentheses, calls, bars, complex constants.
        ("(z+1)^2", "(z + 1)^2"),
        ("sin(z)+c", "sin(z) + c"),
        ("2sin(z)", "2*sin(z)"),
        ("|z|+c", "|z| + c"),
        ("(0.5,-0.25)*z", "(0.5, -0.25)*z"),
        // Names and numbers: letters run together into a name, a product of two needs its `*`.
        ("2z", "2*z"),
        ("zc", "zc"),
        ("z*c", "z*c"),
        ("2*z", "2*z"),
        ("1e-5*z", "1e-5*z"),
        ("p1", "p1"),
        // Statements: `,` on the line, Enter on a new one.
        ("t=sqr(z),z=t+c", "t = sqr(z), z = t + c"),
        ("t=sqr(z){Enter}z=t+c", "t = sqr(z)\nz = t + c"),
        // A `)` with no `(` groups what is before it (after the `=`).
        ("z=z+1)^2", "z = (z + 1)^2"),
    ];
    for (ks, want) in cases {
        assert_eq!(typed(ks), want, "keys {ks:?}");
    }
}

/// Backspace and Delete take characters, step into structures, and take a structure apart in two
/// presses (an empty one in one); at a statement's edge they join statements.
#[test]
fn erasing_takes_structures_apart() {
    let cases = [
        ("z+c{Bksp}", "z + "),
        // The first press at the denominator's start selects the fraction, the second takes it
        // apart: the numerator stays, the caret after it.
        ("a/b{Bksp}{Bksp}", "a/()"),
        ("a/b{Bksp}{Bksp}{Bksp}x", "ax"),
        ("z^{Bksp}", "z"),
        ("2sin({Bksp}", "2"),
        // Into a structure from outside, then through it; an emptied one goes.
        ("z^2{Right}{Bksp}{Bksp}{Bksp}", "z"),
        ("(z+1){Bksp}{Bksp}{Bksp}{Bksp}{Bksp}", ""),
        ("(z+1){Left}{Left}{Left}{Left}{Bksp}{Bksp}", "z + 1"),
        // A call taken apart keeps its argument's parentheses.
        ("sin(z){Left}{Left}{Bksp}{Bksp}", "(z)"),
        ("z^2{Home}{Right}{Del}{Right}{Del}{Del}", "z2"),
        ("z+c{Home}{Del}", "+c"),
        ("a{Enter}b{Home}{Bksp}", "ab"),
        ("a{Enter}b{Up}{End}{Del}", "ab"),
        ("a,b{Home}{Bksp}", "ab"),
    ];
    for (ks, want) in cases {
        assert_eq!(typed(ks), want, "keys {ks:?}");
    }
    // A comment would end up mid-line: no join.
    assert_eq!(typed_into("t = z ; square\nz = t", "{Down}{Home}{Bksp}"), "t = z ; square\nz = t");
    // A blank line between goes on its own.
    assert_eq!(typed_into("a = 1\n\nz = a", "{Down}{Home}{Bksp}"), "a = 1\nz = a");
}

/// Selections are ranges of one row: typing replaces them, `(` `^` `/` wrap them.
#[test]
fn selections_are_replaced_and_wrapped() {
    assert_eq!(typed("z+c{Home}{S-Right}{S-Right}{S-Right}^2"), "(z + c)^2");
    assert_eq!(typed("z+1{S-Left}{S-Left}{S-Left}/2"), "(z + 1)/2");
    assert_eq!(typed("z+c{S-Left}{S-Left}(") , "z*(+c)");
    assert_eq!(typed("z+c{All}w"), "w");
    // Selecting out of a structure takes it whole.
    assert_eq!(typed("a+b/c{S-Left}{S-Left}x"), "a + x");
    let mut e = Editor::new("");
    keys(&mut e, "z^2+c{Home}{S-Right}{S-Right}");
    assert_eq!(e.copy().as_deref(), Some("z^2"));
    assert_eq!(e.cut().as_deref(), Some("z^2"));
    assert_eq!(e.doc.source(), "+c");
}

/// Paste: text that reads goes in as its structures; text that does not is typed.
#[test]
fn pasting_reads_the_text() {
    let paste = |src: &str, ks: &str, text: &str| {
        let mut e = Editor::new(src);
        keys(&mut e, ks);
        e.paste(text);
        e.doc.source()
    };
    assert_eq!(paste("", "", "z^2 + c"), "z^2 + c");
    assert_eq!(paste("", "2*", "a/b"), "2*(a/b)");
    assert_eq!(paste("", "x+", "y/2"), "x + y/2");
    assert_eq!(paste("", "", "t = sqr(z), z = t + c"), "t = sqr(z), z = t + c");
    assert_eq!(paste("", "", "t = sqr(z)\r\nz = t + c"), "t = sqr(z)\nz = t + c");
    assert_eq!(paste("", "a", "/b"), "a/b", "a fragment that does not read is typed");
    // A sign after an operand is the operator, not part of what follows.
    assert_eq!(paste("", "c/z{Right}", " - 1/z"), "c/z - 1/z");
    assert_eq!(paste("", "c*", "-1/z"), "c*(-1/z)");
}

/// An empty box is known, so the dialog can say to fill it.
#[test]
fn an_empty_box_is_known() {
    let mut e = Editor::new("z = z + c");
    assert!(!e.has_empty_box());
    keys(&mut e, "^");
    assert!(e.has_empty_box());
    keys(&mut e, "2");
    assert!(!e.has_empty_box());
    // An empty statement of its own (a new line) is not a box.
    keys(&mut e, "{Enter}");
    assert!(!e.has_empty_box());
}

/// Undo takes back a whole typed name at once; redo puts it back.
#[test]
fn undo_takes_back_a_typed_name_at_once() {
    assert_eq!(typed("zc{Undo}"), "");
    assert_eq!(typed("zc{Undo}{Redo}"), "zc");
    assert_eq!(typed("z+c{Undo}"), "z + ");
    assert_eq!(typed("z+c{Undo}{Undo}{Undo}"), "");
}

/// The arrows walk in and out of structures in reading order; ↑/↓ cross a fraction.
#[test]
fn arrows_walk_through_structures() {
    // From the end, ← visits every place once, in reading order backwards; → goes back the same way.
    let mut e = Editor::new("z = c/(z + 1)^2 + sqr(|z|)");
    let walk = |e: &mut Editor, dir: Dir| {
        let mut seen = vec![e.caret.clone()];
        loop {
            e.step(dir, false);
            if seen.last() == Some(&e.caret) {
                return seen;
            }
            seen.push(e.caret.clone());
        }
    };
    let mut left = walk(&mut e, Dir::Left);
    left.reverse();
    assert_eq!(left, e.doc.places());
    assert_eq!(walk(&mut e, Dir::Right), e.doc.places());
    // ↑ from the denominator is the numerator.
    let mut e = Editor::new("z = c/(z + 1)");
    e.step(Dir::Left, false);
    assert_eq!(e.caret.path, vec![(2, 1)]);
    assert_eq!(e.vertical(true), Vertical::Nearest(Caret { path: vec![(2, 0)], ..Default::default() }));
    // Statements: ↓ and → cross to the next.
    let mut e = Editor::new("a = 1\nb = 2");
    e.home_end(true, false);
    assert!(matches!(e.vertical(false), Vertical::Nearest(Caret { line: 1, .. })));
    e.step(Dir::Right, false);
    assert_eq!((e.caret.line, e.caret.pos), (1, 0));
}

/// A line nobody edited prints back byte for byte; an edited one keeps its indentation and comment.
#[test]
fn unedited_lines_print_verbatim() {
    let src = "; Mandelbrot, with a twist\r\n  t = sqr(z)   ; square\r\n\r\nz = (\r\nz = t+c";
    let mut e = Editor::new(src);
    assert_eq!(e.doc.source(), src);
    // The caret starts at the end of the first statement; the second line's is edited.
    e.caret = Caret { line: 1, stmt: 0, path: Vec::new(), pos: e.doc.row(&Caret { line: 1, ..Default::default() }).unwrap().len() };
    keys(&mut e, "*2");
    assert_eq!(e.doc.source(), "; Mandelbrot, with a twist\r\n  t = sqr(z)*2   ; square\r\n\r\nz = (\r\nz = t+c");
    // A new line takes the source's line break.
    keys(&mut e, "{Enter}w");
    assert!(e.doc.source().contains("; square\r\nw\r\n"), "{:?}", e.doc.source());
    // Typing into a comment-only line puts the statement before the comment.
    assert_eq!(typed_into("; note\nz = c", "{Up}t"), "t ; note\nz = c");
    // A source with nothing that reads gets a line to type into, and a break before it once typed.
    assert_eq!(typed_into("z = (", ""), "z = (");
    assert_eq!(typed_into("z = (", "w"), "z = (\nw");
}

/// Over the corpus: read and printed back unedited, every source is itself; every statement
/// marked edited and printed from its rows, it computes the same.
#[test]
fn the_document_prints_back_the_source() {
    let mut checked = 0;
    for src in super::super::model::tests::corpus() {
        let mut doc = Doc::read(&src);
        assert_eq!(doc.source(), src);
        let Ok(want) = parse(&src) else { continue };
        for l in &mut doc.lines {
            if let Kind::Read { edited, .. } = &mut l.kind {
                *edited = true;
            }
        }
        let printed = doc.source();
        assert_eq!(parse(&printed).as_ref(), Ok(&want), "{src:?} printed as {printed:?}");
        checked += 1;
    }
    assert!(checked > 400, "{checked}");
}

/// Random keys never leave the caret or the selection outside the document, the dialog always gets
/// the printed text, and undoing everything gives back the source.
#[test]
fn random_keys_keep_the_document_whole() {
    let mut seed = 0x5eed_f00d_u64;
    let mut next = |k: u64| {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) % k
    };
    let chars: Vec<char> = "zc2.e+-*/^()|,=sin".chars().collect();
    let sources = ["z = z^2 + c", "t = sqr(z) ; s\nz = t/(c - 1) + |z|", "", "z = (0.5, -0.25)*z^-2 + 1e-3"];
    // Long runs; then short ones, inside the undo depth, that must undo back to the start.
    for (src, steps) in sources.iter().map(|s| (*s, 3000)).chain(sources.iter().map(|s| (*s, 120))) {
        let mut e = Editor::new(src);
        for step in 0..steps {
            match next(14) {
                0..=5 => {
                    e.type_char(chars[next(chars.len() as u64) as usize]);
                }
                6 => e.step(Dir::Left, next(3) == 0),
                7 => e.step(Dir::Right, next(3) == 0),
                8 => {
                    e.backspace();
                }
                9 => {
                    e.delete();
                }
                10 => match e.vertical(next(2) == 0) {
                    Vertical::To(c) | Vertical::Nearest(c) => e.set_caret(Caret { pos: 0, ..c }, false),
                    Vertical::Stay => {}
                },
                11 => {
                    if next(4) == 0 {
                        e.enter();
                    } else {
                        e.tab(next(2) == 0);
                    }
                }
                12 => {
                    e.undo();
                }
                _ => {
                    if let Some(t) = e.cut() {
                        e.paste(&t);
                    }
                }
            }
            let ok = |c: &Caret| e.doc.row(c).is_some_and(|r| c.pos <= r.len());
            assert!(ok(&e.caret), "{src:?} step {step}: caret {:?} outside {:?}", e.caret, e.doc.source());
            if let Some(a) = &e.anchor {
                assert!(ok(a), "{src:?} step {step}: anchor {a:?} outside");
            }
            assert_eq!(e.synced, e.doc.source(), "{src:?} step {step}");
        }
        if steps < UNDO_DEPTH {
            while e.undo() {}
            assert_eq!(e.doc.source(), src, "undoing everything");
        }
    }
}
