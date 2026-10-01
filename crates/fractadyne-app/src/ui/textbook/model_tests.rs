use super::*;
use fractadyne_core::ir::parse::parse;

fn row_of(src: &str) -> Row {
    match &read(src)[0].0 {
        Line::Read { stmts, .. } => stmts[0].row.clone(),
        other => panic!("{src:?}: {other:?}"),
    }
}

// Rows are built from parts: a name or number is a run of characters, the rest one atom each.
fn w(s: &str) -> Row {
    chars(s)
}
fn n(s: &str) -> Row {
    chars(s)
}
fn op(o: Op) -> Row {
    vec![Atom::Op(o)]
}
fn frac(num: Row, den: Row) -> Row {
    vec![Atom::Frac { num, den }]
}
fn sup(x: Row) -> Row {
    vec![Atom::Sup(x)]
}
fn group(r: Row) -> Row {
    vec![Atom::Group(r)]
}
fn cat(parts: Vec<Row>) -> Row {
    parts.concat()
}

/// The text read into rows: fractions and exponents take their operands, the parentheses that only
/// grouped an operand go, the user's own parentheses stay.
#[test]
fn reading_makes_fractions_exponents_and_keeps_the_users_parentheses() {
    assert_eq!(
        row_of("z = z^2 + c/(z + 1)"),
        cat(vec![
            w("z"),
            op(Op::Eq),
            w("z"),
            sup(n("2")),
            op(Op::Plus),
            frac(w("c"), cat(vec![w("z"), op(Op::Plus), n("1")])),
        ])
    );
    // (x*a)/b and x*(a/b) are different rows, as they are different texts. The `*` between two
    // names stays (they would run together into one); before parentheses it is implied.
    assert_eq!(row_of("x*a/b"), frac(cat(vec![w("x"), op(Op::Times), w("a")]), w("b")));
    assert_eq!(row_of("x*(a/b)"), cat(vec![w("x"), group(frac(w("a"), w("b")))]));
    assert_eq!(row_of("2*z"), cat(vec![n("2"), w("z")]));
    assert_eq!(row_of("z*2"), cat(vec![w("z"), op(Op::Times), n("2")]));
    // The base's parentheses are the user's (they show); the exponent's only group (they go).
    assert_eq!(
        row_of("(z + 1)^(p1 - 1)"),
        cat(vec![group(cat(vec![w("z"), op(Op::Plus), n("1")])), sup(cat(vec![w("p1"), op(Op::Minus), n("1")]))])
    );
    assert_eq!(row_of("SIN(z)"), vec![Atom::Func { name: "SIN".into(), arg: w("z") }]);
    assert_eq!(row_of("(0.5, -0.25)"), vec![Atom::Complex { re: n("0.5"), im: cat(vec![op(Op::Minus), n("0.25")]) }]);
    // A number's exponent sign is one of its characters.
    assert_eq!(row_of("1e-5"), n("1e-5"));
}

/// A run of characters reads as the lexer reads the same text.
#[test]
fn runs_read_as_the_lexer_reads_them() {
    let kinds = |s: &str| {
        let row = chars(s);
        run_tokens(&row, 0).0.iter().map(|t| (t.kind, token_text(&row, *t))).collect::<Vec<_>>()
    };
    let (num, name, sign) = (Kind::Number, Kind::Name, Kind::Sign);
    assert_eq!(kinds("p1"), [(name, "p1".into())]);
    assert_eq!(kinds("2z"), [(num, "2".into()), (name, "z".into())]);
    assert_eq!(kinds("1e-5"), [(num, "1e-5".into())]);
    assert_eq!(kinds("2e"), [(num, "2".into()), (name, "e".into())]);
    assert_eq!(kinds("1e-"), [(num, "1".into()), (name, "e".into()), (sign, "-".into())]);
    assert_eq!(kinds("2e5z"), [(num, "2e5".into()), (name, "z".into())]);
    assert!(exponent_sign_at(&chars("1e-"), 2));
    assert!(!exponent_sign_at(&chars("ze-"), 2));
    assert!(!exponent_sign_at(&chars("1e5e-"), 4));
}

/// Lines are read one at a time: a line that does not read is kept as text with its reason, and
/// does not hide the others. Spans are the whole source's.
#[test]
fn a_line_that_does_not_read_does_not_hide_the_others() {
    let src = "; Mandelbrot\nt = sqr(z), w = t\r\nz = t + (\n\nz = z + c ; done";
    let lines = read(src);
    assert_eq!(lines.len(), 4, "the blank line is skipped");
    assert!(matches!(&lines[0].0, Line::Read { stmts, comment: Some(c) } if stmts.is_empty() && c == "Mandelbrot"));
    let Line::Read { stmts, .. } = &lines[1].0 else { panic!() };
    assert_eq!(stmts.len(), 2);
    assert_eq!(&src[stmts[0].span.start..stmts[0].span.end], "t = sqr(z)");
    assert_eq!(&src[stmts[1].span.start..stmts[1].span.end], "w = t");
    assert!(matches!(&lines[2].0, Line::Unread { text, error } if text == "z = t + (" && error.contains("expected")));
    let Line::Read { stmts, comment } = &lines[3].0 else { panic!() };
    assert_eq!((&src[stmts[0].span.start..stmts[0].span.end], comment.as_deref()), ("z = z + c", Some("done")));
}

/// Printing a hand-built row (as editing makes them) puts the parentheses the grammar needs, and
/// writes out the products the row leaves implied.
#[test]
fn printing_adds_the_parentheses_the_grammar_needs_and_no_others() {
    let cases: Vec<(Row, &str)> = vec![
        (cat(vec![w("x"), op(Op::Times), frac(w("a"), w("b"))]), "x*(a/b)"),
        (cat(vec![w("x"), frac(w("a"), w("b"))]), "x*(a/b)"),
        (cat(vec![op(Op::Minus), frac(w("a"), w("b"))]), "-(a/b)"),
        (cat(vec![w("a"), op(Op::Minus), frac(w("b"), w("c"))]), "a - b/c"),
        (cat(vec![frac(n("1"), w("z")), sup(n("2"))]), "(1/z)^2"),
        (frac(cat(vec![w("a"), op(Op::Plus), w("b")]), cat(vec![w("c"), op(Op::Times), w("d")])), "(a + b)/(c*d)"),
        (frac(cat(vec![w("a"), op(Op::Times), w("b")]), cat(vec![op(Op::Minus), w("c")])), "a*b/-c"),
        (frac(w("a"), frac(w("b"), w("c"))), "a/(b/c)"),
        (frac(frac(w("a"), w("b")), w("c")), "a/b/c"),
        (cat(vec![w("z"), sup(cat(vec![w("p1"), op(Op::Plus), n("1")]))]), "z^(p1 + 1)"),
        (cat(vec![w("z"), sup(cat(vec![op(Op::Minus), n("1")]))]), "z^-1"),
        (frac(cat(vec![w("z"), sup(n("2"))]), cat(vec![w("z"), sup(n("3"))])), "z^2/z^3"),
        (cat(vec![w("t"), op(Op::Eq), w("z"), op(Op::Times), w("c")]), "t = z*c"),
        // Implied products, written out.
        (cat(vec![n("2"), w("z")]), "2*z"),
        (cat(vec![w("z"), group(cat(vec![w("z"), op(Op::Plus), n("1")]))]), "z*(z + 1)"),
        (cat(vec![w("z"), sup(n("2")), w("c")]), "z^2*c"),
        (cat(vec![n("2"), sup(n("3")), n("4")]), "2^3*4"),
        // `1e - 5` is 1·e − 5; `1e-5` one number.
        (cat(vec![n("1e"), op(Op::Minus), n("5")]), "1*e - 5"),
        (n("1e-5"), "1e-5"),
    ];
    for (row, want) in cases {
        assert_eq!(print_row(&row), want, "{row:?}");
    }
}

/// Normalisation puts an edited row back in the grammar's form and keeps the caret in place.
#[test]
fn normalising_keeps_the_grammar_and_the_caret() {
    let at = |path: Vec<(usize, u8)>, pos: usize| Caret { path, pos, ..Default::default() };
    let norm = |mut row: Row, c: Caret| {
        let mut c = c;
        normalize(&mut row, &mut Vec::new(), &mut [&mut c]);
        (row, c)
    };
    // A `-` typed as a character is an operator, unless it is an exponent's sign.
    let (row, _) = norm(cat(vec![w("z"), vec![Atom::Char('-')], w("c")]), at(vec![], 2));
    assert_eq!(row, cat(vec![w("z"), op(Op::Minus), w("c")]));
    let (row, _) = norm(n("1e-5"), at(vec![], 3));
    assert_eq!(row, n("1e-5"));
    // A `*` the printer writes anyway goes; the caret after it stays after the `2`.
    let (row, c) = norm(cat(vec![n("2"), op(Op::Times), w("z")]), at(vec![], 2));
    assert_eq!((row, c.pos), (cat(vec![n("2"), w("z")]), 1));
    let (row, _) = norm(cat(vec![w("z"), op(Op::Times), w("c")]), at(vec![], 2));
    assert_eq!(row, cat(vec![w("z"), op(Op::Times), w("c")]), "z*c would run together");
    // A function's name before parentheses calls it; a caret in them stays in them.
    let (row, c) = norm(cat(vec![n("2"), w("sin"), group(w("z"))]), at(vec![(4, 0)], 1));
    assert_eq!(row, cat(vec![n("2"), vec![Atom::Func { name: "sin".into(), arg: w("z") }]]));
    assert_eq!(c, at(vec![(1, 0)], 1));
    let (row, _) = norm(cat(vec![w("sin"), op(Op::Times), group(w("z"))]), at(vec![], 0));
    assert_eq!(row.len(), 5, "an explicit `sin*(z)` is not a call: {row:?}");
    // An exponent on an exponent raises the power: the caret in the first goes into the group.
    let (row, c) = norm(cat(vec![w("ab"), sup(n("2")), sup(n("3"))]), at(vec![(2, 0)], 1));
    assert_eq!(row, cat(vec![group(cat(vec![w("ab"), sup(n("2"))])), sup(n("3"))]));
    assert_eq!(c, at(vec![(0, 0), (2, 0)], 1));
    assert_eq!(print_row(&row), "(ab^2)^3");
}

/// Every place in a row gets one mark, and the name the caret touches shows as typed.
#[test]
fn every_caret_place_is_marked_once() {
    use super::super::layout::Node;
    fn ids(ns: &[Node], out: &mut Vec<u32>) {
        for n in ns {
            match n {
                Node::Mark(id) | Node::Slot(Some(id)) => out.push(*id),
                Node::Glyphs { anchors, .. } => out.extend(anchors.iter().map(|a| a.1)),
                Node::Frac { num, den } => {
                    ids(num, out);
                    ids(den, out);
                }
                Node::Scripts { base, sup, sub } => {
                    ids(std::slice::from_ref(&**base), out);
                    ids(sup.as_deref().unwrap_or(&[]), out);
                    ids(sub.as_deref().unwrap_or(&[]), out);
                }
                Node::Fenced { body, .. } | Node::Radical(body) | Node::Overline(body) | Node::Row(body) => ids(body, out),
                Node::Slot(None) => {}
            }
        }
    }
    // Every place of every row: the caret on each in turn, so every name shows as typed somewhere.
    for src in ["z = z^2 + c/(z + 1)", "t = sqr(z - p1)", "w = 2*z*c^-1 + |z| - (0.5, -0.25)", "x = sin(2*pi*z)"] {
        let row = row_of(src);
        let mut all = Vec::new();
        fn places(row: &[Atom], path: &mut Vec<(usize, u8)>, out: &mut Vec<Caret>) {
            for pos in 0..=row.len() {
                out.push(Caret { path: path.clone(), pos, ..Default::default() });
            }
            for (i, a) in row.iter().enumerate() {
                for k in 0..a.arity() {
                    path.push((i, k));
                    places(a.child(k).unwrap(), path, out);
                    path.pop();
                }
            }
        }
        places(&row, &mut Vec::new(), &mut all);
        for caret in &all {
            let mut m = Marker::new(vec![caret.clone()]);
            let nodes = emit(&row, &mut Vec::new(), &mut m, true);
            let mut got = Vec::new();
            ids(&nodes, &mut got);
            got.sort();
            assert!(got.windows(2).all(|w| w[0] != w[1]), "{src}: a mark twice with the caret at {caret:?}");
            assert_eq!(got.len(), m.places.len(), "{src}: marks made but not placed");
            assert!(m.places.contains(caret), "{src}: the caret's own place {caret:?} is not marked");
        }
    }
}

/// THE property (design §6): every statement printed back from its row reads as the same
/// computation — the IR, every constant's bits — over a corpus of formulas.
#[test]
fn printing_a_read_formula_back_computes_the_same() {
    let mut checked = 0;
    for src in corpus() {
        let Ok(want) = parse(&src) else { continue };
        let mut printed = String::new();
        let mut at = 0;
        for (line, _) in read(&src) {
            let Line::Read { stmts, .. } = line else { panic!("{src:?} parses but a line does not read") };
            for st in stmts {
                printed.push_str(&src[at..st.span.start]);
                printed.push_str(&print_row(&st.row));
                at = st.span.end;
            }
        }
        printed.push_str(&src[at..]);
        let got = parse(&printed).unwrap_or_else(|e| panic!("{src:?} printed as {printed:?}: {e}"));
        assert_eq!(got, want, "{src:?} printed as {printed:?}");
        checked += 1;
    }
    assert!(checked > 400, "{checked} formulas checked");
}

/// Formulas from the repository and a deterministic stream over the grammar.
pub(in super::super) fn corpus() -> Vec<String> {
    let mut out: Vec<String> = [
        "z^2 + c", "z = z^3 - p1*z + c", "t = sqr(z)\nz = t + p1*conj(t) + c", "z = (real(z) - flip(abs(imag(z))))^2 + c",
        "z = sin(z) + c", "z = exp(z) + c", "z = z^2 + c/(z + 2)", "z = z^2 + 0.3*log(z + 1) + c", "sqrt(z^4 + c)",
        "z^p1 + c", "z = (0.25, -0.1)*z + c", "z = z^2 + 0.25*z - (0, 1)*z/10 + c", "-z^2", "2^3^2", "z^-1", "z^2.5",
        "|z| + c", "cabs(z)", "recip(z) + 1/z", "x = 2, z = x*z/3*c", "z = -(z/2)", "z = +z*-c", "z = 1e-5*z + c",
        "z = ((z))", "z = z/(c/(z+1))", "z = (z/c)/z", "z = z*(1/c)", "z = 2*(3/4)*z", "z = (z^2)^3", "z = -z/-c",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let mut seed = 0x7e47_b00c_u64;
    let mut next = |k: u64| {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) % k
    };
    fn expr(next: &mut dyn FnMut(u64) -> u64, d: u32) -> String {
        let leaves = ["z", "c", "p1", "2", "0.5", "3", "pi", "1e-3"];
        if d == 0 || next(5) == 0 {
            return leaves[next(leaves.len() as u64) as usize].into();
        }
        match next(10) {
            0..=3 => {
                let o = ["+", "-", "*", "/"][next(4) as usize];
                format!("{}{o}{}", expr(next, d - 1), expr(next, d - 1))
            }
            4 => format!("({})", expr(next, d - 1)),
            5 => format!("-{}", expr(next, d - 1)),
            6 => format!("{}({})", ["sin", "sqr", "conj", "recip", "sqrt", "exp"][next(6) as usize], expr(next, d - 1)),
            7 => format!("|{}|", expr(next, d - 1)),
            _ => {
                let base = if next(2) == 0 { "z".to_string() } else { format!("({})", expr(next, d - 1)) };
                let exp = ["2", "3", "-1", "p1", "(z+1)", "0.5", "(1/2)"][next(7) as usize];
                format!("{base}^{exp}")
            }
        }
    }
    for _ in 0..600 {
        out.push(format!("z = {}", expr(&mut next, 4)));
    }
    out
}

/// The notation (§4.5): truthful, and textbook where that is still the same computation.
#[test]
fn the_notation_is_truthful() {
    use super::super::layout::Node;
    let m = |src: &str| math_row(&row_of(src));
    // sqr is ², with parentheses round a compound operand; exp stays exp (e^z is another thing).
    assert!(matches!(&m("sqr(z)")[..], [Node::Scripts { sup: Some(_), .. }]));
    assert!(matches!(&m("sqr(z + c)")[..], [Node::Scripts { base, .. }] if matches!(**base, Node::Fenced { open: '(', .. })));
    assert!(matches!(&m("exp(z)")[..], [Node::Glyphs { text, .. }, Node::Fenced { .. }] if text == "exp"));
    // |z| is the SQUARED modulus: |𝑧|²; cabs is |𝑧|.
    assert!(matches!(&m("|z|")[..], [Node::Scripts { base, sup: Some(_), .. }] if matches!(**base, Node::Fenced { open: '|', .. })));
    assert!(matches!(&m("cabs(z)")[..], [Node::Fenced { open: '|', .. }]));
    assert!(matches!(&m("recip(z)")[..], [Node::Frac { .. }]));
    assert!(matches!(&m("conj(z)")[..], [Node::Overline(_)]));
    assert!(matches!(&m("sqrt(z)")[..], [Node::Radical(_)]));
    assert!(matches!(&m("cotan(z)")[..], [Node::Glyphs { text, .. }, _] if text == "cot"));
    assert!(matches!(&m("real(z)")[..], [Node::Glyphs { text, .. }, _] if text == "Re"));
    // Names: p₁, π, case ignored.
    assert!(matches!(&m("p1")[..], [Node::Scripts { sub: Some(_), sup: None, .. }]));
    assert!(matches!(&m("PI")[..], [Node::Glyphs { text, .. }] if text == "\u{1D70B}"));
    // Products: juxtaposition, a dot before a number and between a number and a fraction.
    let dots = |src: &str| m(src).iter().filter(|n| matches!(n, Node::Glyphs { text, .. } if text == "\u{22C5}")).count();
    assert_eq!(dots("2*z"), 0);
    assert_eq!(dots("z*2"), 1);
    assert_eq!(dots("2*3"), 1);
    assert_eq!(dots("p1*conj(t)"), 0);
    // Between two names, and before a sign: 𝑧·𝑐, not the name 𝑧𝑐; 𝑧·−𝑐, not 𝑧 − 𝑐. A name whose
    // digits are a subscript has ended plainly: 𝑝₁𝑧 (but 𝑧·𝑝₁, which could be the name 𝑧𝑝₁).
    assert_eq!(dots("z*c"), 1);
    assert_eq!(dots("p1*z"), 0);
    assert_eq!(dots("z*p1"), 1);
    assert_eq!(dots("z*-c"), 1);
    assert_eq!(dots("(z + 1)*2"), 1);
    // An exponent on a fraction parenthesises it: (1/𝑧)², not 1/𝑧².
    let row = cat(vec![frac(n("1"), w("z")), sup(n("2"))]);
    assert!(matches!(&math_row(&row)[..], [Node::Scripts { base, .. }] if matches!(**base, Node::Fenced { .. })));
    // A number in scientific form raised is raised whole: (1×10⁻⁵)².
    assert!(matches!(&m("1e-5^2")[..], [Node::Scripts { base, .. }] if matches!(&**base, Node::Fenced { body, .. } if body.len() == 3)));
    // Scientific notation as a × 10ᵇ; a complex constant as a ± b𝑖.
    assert_eq!(m("1e-5").len(), 3);
    let c = m("(0.25, -0.1)");
    let [Node::Fenced { body, .. }] = &c[..] else { panic!("{c:?}") };
    assert!(body.iter().any(|n| matches!(n, Node::Glyphs { text, .. } if text == "\u{2212}")));
}
