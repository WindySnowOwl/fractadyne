use super::{parse, syntax};
use crate::ir::syntax::{BinOp, Expr, ExprKind, Span};

fn body(src: &str) -> Expr {
    syntax(src).unwrap_or_else(|e| panic!("{src:?}: {e}")).statements.last().unwrap().body.clone()
}

fn text<'a>(src: &'a str, s: Span) -> &'a str {
    &src[s.start..s.end]
}

/// The tree has the grammar's precedence and keeps what the IR drops: parentheses as written,
/// unary plus, spellings by span.
#[test]
fn the_tree_follows_the_grammar_and_keeps_what_the_ir_drops() {
    let src = "z = -z^2 + c/(z + 1)*p1";
    let s = syntax(src).unwrap();
    let st = &s.statements[0];
    assert_eq!(st.target.as_ref().map(|t| (t.name.as_str(), text(src, t.span))), Some(("z", "z")));
    assert_eq!(text(src, st.span), src);
    // (-(z^2)) + ((c / (z + 1)) * p1)
    let ExprKind::Bin(BinOp::Add, l, r) = &st.body.kind else { panic!("{:?}", st.body) };
    let ExprKind::Neg(pow) = &l.kind else { panic!("{l:?}") };
    assert!(matches!(&pow.kind, ExprKind::Pow(..)), "-z^2 is -(z^2)");
    assert_eq!(text(src, l.span), "-z^2");
    let ExprKind::Bin(BinOp::Mul, div, p1) = &r.kind else { panic!("{r:?}") };
    let ExprKind::Bin(BinOp::Div, _, group) = &div.kind else { panic!("{div:?}") };
    assert!(matches!(&group.kind, ExprKind::Group(_)), "parentheses as written");
    assert_eq!(text(src, group.span), "(z + 1)");
    assert_eq!(p1.kind, ExprKind::Name("p1".into()));
    // Right-associative powers; unary plus kept; complex constants, bars, calls; names lower-cased
    // with their spelling at the span.
    let b = body("Z^2^3");
    let ExprKind::Pow(base, exp) = &b.kind else { panic!() };
    assert_eq!(base.kind, ExprKind::Name("z".into()));
    assert!(matches!(&exp.kind, ExprKind::Pow(..)));
    assert!(matches!(body("+z").kind, ExprKind::Pos(_)));
    assert!(matches!(body("(0.5, -0.25)").kind, ExprKind::Complex(..)));
    assert!(matches!(body("|z|").kind, ExprKind::Bars(_)));
    let src = "SQR(z) + 1.50";
    let b = body(src);
    let ExprKind::Bin(_, call, num) = &b.kind else { panic!() };
    assert!(matches!(&call.kind, ExprKind::Call { func, .. } if func == "sqr"));
    assert_eq!(text(src, call.span), "SQR(z)");
    assert_eq!((num.kind.clone(), text(src, num.span)), (ExprKind::Num(1.5), "1.50"));
}

/// Statements, comments (kept, with their places), and the checks only evaluation makes left out:
/// a formula still being typed has a tree.
#[test]
fn comments_are_kept_and_a_half_typed_formula_has_a_tree() {
    let src = "; a (comment)\nt = sqr(z) ; square\r\nz = t + c";
    let s = syntax(src).unwrap();
    assert_eq!(s.statements.len(), 2);
    assert_eq!(s.comments.len(), 2);
    assert_eq!(s.comments[0].text, " a (comment)");
    assert_eq!(text(src, s.comments[1].span), "; square\r");
    assert_eq!(s.comments[1].text, " square", "the CR of a CR LF is not the comment's");
    // Used before it is assigned, a complex constant of variables, no step: parse refuses, the tree is there.
    for src in ["z = co", "z = (z, 1)", "t = z"] {
        assert!(parse(src).is_err(), "{src}");
        assert!(syntax(src).is_ok(), "{src}");
    }
    // A grammar error is an error for both, the same one.
    for src in ["z = sin(z", "z = z +", "c = 3", "z = fn1(z)", "z = )"] {
        assert_eq!(syntax(src).err(), parse(src).err(), "{src}");
    }
}

/// An error `parse` makes and `syntax` does not: one of the checks only evaluation makes.
fn evaluation_error(e: &super::ParseError) -> bool {
    let m = &e.message;
    m.contains("is used before it is assigned") || m.contains("complex constant") || m.contains("no step")
        || (e.line, e.col) == (1, 1)
}

/// Over the whole snapshot corpus: whatever `parse` accepts has a tree. Where `syntax` refuses,
/// `parse` refuses too — with the same error, or with an evaluation error it met EARLIER in the
/// text (it stops at the first error of either kind; `a2 ^= z` is "`a2` is used before it is
/// assigned" to it, "expected a value, found '='" to `syntax`). Where only `parse` refuses, it is an
/// evaluation error. And every node's span lies inside its parent's, in order.
#[test]
fn syntax_and_parse_agree_on_the_whole_corpus() {
    let (mut both_ok, mut only_parse_refuses, mut earlier) = (0, 0, 0);
    for src in super::snapshot::corpus() {
        match (parse(&src), syntax(&src)) {
            (Ok(_), Ok(_)) => both_ok += 1,
            (Ok(_), Err(e)) => panic!("{src:?}: parse accepts, syntax refuses: {e}"),
            (Err(p), Err(s)) if p == s => {}
            (Err(p), Err(s)) => {
                earlier += 1;
                assert!(evaluation_error(&p) && (p.line, p.col) < (s.line, s.col), "{src:?}: {p} / {s}");
            }
            (Err(p), Ok(_)) => {
                only_parse_refuses += 1;
                assert!(evaluation_error(&p), "{src:?}: {p}");
            }
        }
        if let Ok(s) = syntax(&src) {
            for st in &s.statements {
                assert!(st.body.span.start >= st.span.start && st.body.span.end <= st.span.end, "{src:?}");
                spans_nest(&src, &st.body);
            }
        }
    }
    assert!(
        both_ok > 800 && only_parse_refuses > 20,
        "{both_ok} both, {only_parse_refuses} only parse refuses, {earlier} an earlier evaluation error"
    );
}

fn spans_nest(src: &str, e: &Expr) {
    let kids: Vec<&Expr> = match &e.kind {
        ExprKind::Num(_) | ExprKind::Name(_) => vec![],
        ExprKind::Call { arg, .. } => vec![arg],
        ExprKind::Group(a) | ExprKind::Bars(a) | ExprKind::Neg(a) | ExprKind::Pos(a) | ExprKind::Assign(_, a) => vec![a],
        ExprKind::Complex(a, b)
        | ExprKind::Bin(_, a, b)
        | ExprKind::Pow(a, b)
        | ExprKind::Cmp(_, a, b)
        | ExprKind::Logic(_, a, b) => vec![a, b],
    };
    let mut at = e.span.start;
    for k in kids {
        assert!(k.span.start >= at && k.span.end <= e.span.end, "{src:?}: {:?} in {:?}", k.span, e.span);
        at = k.span.end;
        spans_nest(src, k);
    }
    match &e.kind {
        ExprKind::Group(_) | ExprKind::Complex(..) => {
            assert!(text(src, e.span).starts_with('(') && text(src, e.span).ends_with(')'), "{src:?}")
        }
        ExprKind::Bars(_) => assert!(text(src, e.span).starts_with('|') && text(src, e.span).ends_with('|')),
        ExprKind::Name(n) => assert_eq!(&text(src, e.span).to_ascii_lowercase(), n),
        _ => {}
    }
}
