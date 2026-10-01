use super::*;
use fractadyne_core::ir::parse::parse;

fn row_of(src: &str) -> Row {
    match &read(src)[0].0 {
        Line::Read { stmts, .. } => stmts[0].row.clone(),
        other => panic!("{src:?}: {other:?}"),
    }
}

fn w(s: &str) -> Atom {
    Atom::Word(s.into())
}
fn n(s: &str) -> Atom {
    Atom::Number(s.into())
}
fn op(o: Op) -> Atom {
    Atom::Op(o)
}

/// The text read into rows: fractions and exponents take their operands, the parentheses that only
/// grouped an operand go, the user's own parentheses stay.
#[test]
fn reading_makes_fractions_exponents_and_keeps_the_users_parentheses() {
    assert_eq!(
        row_of("z = z^2 + c/(z + 1)"),
        vec![
            w("z"),
            op(Op::Eq),
            w("z"),
            Atom::Sup(vec![n("2")]),
            op(Op::Plus),
            Atom::Frac { num: vec![w("c")], den: vec![w("z"), op(Op::Plus), n("1")] },
        ]
    );
    // (x*a)/b and x*(a/b) are different rows, as they are different texts.
    assert_eq!(row_of("x*a/b"), vec![Atom::Frac { num: vec![w("x"), op(Op::Times), w("a")], den: vec![w("b")] }]);
    assert_eq!(
        row_of("x*(a/b)"),
        vec![w("x"), op(Op::Times), Atom::Group(vec![Atom::Frac { num: vec![w("a")], den: vec![w("b")] }])]
    );
    // The base's parentheses are the user's (they show); the exponent's only group (they go).
    assert_eq!(
        row_of("(z + 1)^(p1 - 1)"),
        vec![
            Atom::Group(vec![w("z"), op(Op::Plus), n("1")]),
            Atom::Sup(vec![w("p1"), op(Op::Minus), n("1")]),
        ]
    );
    assert_eq!(row_of("SIN(z)"), vec![Atom::Func { name: "SIN".into(), arg: vec![w("z")] }]);
    assert_eq!(row_of("(0.5, -0.25)"), vec![Atom::Complex { re: vec![n("0.5")], im: vec![op(Op::Minus), n("0.25")] }]);
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

/// Printing a hand-built row (as editing will make them) puts the parentheses the grammar needs.
#[test]
fn printing_adds_the_parentheses_the_grammar_needs_and_no_others() {
    let frac = |num: Row, den: Row| Atom::Frac { num, den };
    let cases: Vec<(Row, &str)> = vec![
        (vec![w("x"), op(Op::Times), frac(vec![w("a")], vec![w("b")])], "x*(a/b)"),
        (vec![op(Op::Minus), frac(vec![w("a")], vec![w("b")])], "-(a/b)"),
        (vec![w("a"), op(Op::Minus), frac(vec![w("b")], vec![w("c")])], "a - b/c"),
        (vec![frac(vec![n("1")], vec![w("z")]), Atom::Sup(vec![n("2")])], "(1/z)^2"),
        (vec![frac(vec![w("a"), op(Op::Plus), w("b")], vec![w("c"), op(Op::Times), w("d")])], "(a + b)/(c*d)"),
        (vec![frac(vec![w("a"), op(Op::Times), w("b")], vec![op(Op::Minus), w("c")])], "a*b/-c"),
        (vec![frac(vec![w("a")], vec![frac(vec![w("b")], vec![w("c")])])], "a/(b/c)"),
        (vec![frac(vec![frac(vec![w("a")], vec![w("b")])], vec![w("c")])], "a/b/c"),
        (vec![w("z"), Atom::Sup(vec![w("p1"), op(Op::Plus), n("1")])], "z^(p1 + 1)"),
        (vec![w("z"), Atom::Sup(vec![op(Op::Minus), n("1")])], "z^-1"),
        (vec![frac(vec![w("z"), Atom::Sup(vec![n("2")])], vec![w("z"), Atom::Sup(vec![n("3")])])], "z^2/z^3"),
        (vec![w("t"), op(Op::Eq), w("z"), op(Op::Times), w("c")], "t = z*c"),
    ];
    for (row, want) in cases {
        assert_eq!(print_row(&row), want, "{row:?}");
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
fn corpus() -> Vec<String> {
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
    // An exponent on a fraction parenthesises it: (1/𝑧)², not 1/𝑧².
    let row = vec![Atom::Frac { num: vec![n("1")], den: vec![w("z")] }, Atom::Sup(vec![n("2")])];
    assert!(matches!(&math_row(&row)[..], [Node::Scripts { base, .. }] if matches!(**base, Node::Fenced { .. })));
    // Scientific notation as a × 10ᵇ; a complex constant as a ± b𝑖.
    assert_eq!(m("1e-5").len(), 3);
    let c = m("(0.25, -0.1)");
    let [Node::Fenced { body, .. }] = &c[..] else { panic!("{c:?}") };
    assert!(body.iter().any(|n| matches!(n, Node::Glyphs { text, .. } if text == "\u{2212}")));
}
