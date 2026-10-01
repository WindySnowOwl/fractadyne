//! The textbook editor's model of a formula (design/formula-textbook-editor.md §4.3–4.5): rows of
//! atoms in which standard precedence applies, as in the text, with the structures that need two
//! dimensions — fractions, exponents, parentheses, functions — holding rows of their own.
//!
//! - [`read`]: a source into lines of statement rows, from the core's syntax tree. Lines are read
//!   one at a time (a statement never spans one: a line break always ends it), so a line that does
//!   not read does not hide the others.
//! - [`print_row`]: a row back to text, with the parentheses the grammar needs and no others.
//! - [`math_row`]: a row as the layout's math list, in truthful textbook notation (§4.5).

use super::layout::{Class, Node};
use fractadyne_core::ir::parse::syntax;
use fractadyne_core::ir::syntax::{BinOp, Expr, ExprKind, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Op {
    Plus,
    Minus,
    Times,
    Eq,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Atom {
    /// A name as written: `z`, `p1`, `pixel`, `tmp`.
    Word(String),
    /// A number as written: `0.25`, `1e-5`.
    Number(String),
    Op(Op),
    Frac { num: Row, den: Row },
    /// An exponent on the atom before it.
    Sup(Row),
    /// Parentheses the user wrote (not the ones a fraction or an exponent groups by itself).
    Group(Row),
    /// A named function; the name as written.
    Func { name: String, arg: Row },
    /// `|x|`, the squared modulus.
    Bars(Row),
    Complex { re: Row, im: Row },
    /// An empty place still to fill (made by editing).
    #[allow(dead_code)]
    Slot,
}

pub(crate) type Row = Vec<Atom>;

/// A statement: its row, and where its text is (bytes of the whole source).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Stmt {
    pub(crate) row: Row,
    pub(crate) span: Span,
}

/// One line of the source.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Line {
    /// Its statements (perhaps none) and its comment.
    Read { stmts: Vec<Stmt>, comment: Option<String> },
    /// A line that does not read: its text and why.
    Unread { text: String, error: String },
}

/// The source's non-blank lines, with each one's byte range.
pub(crate) fn read(src: &str) -> Vec<(Line, Span)> {
    let mut out = Vec::new();
    let mut start = 0;
    for raw in src.split('\n') {
        let span = Span { start, end: start + raw.len() };
        start = span.end + 1;
        if raw.trim().is_empty() {
            continue;
        }
        let line = match syntax(raw) {
            Ok(s) => Line::Read {
                stmts: s
                    .statements
                    .iter()
                    .map(|st| Stmt {
                        row: statement_row(raw, st),
                        span: Span { start: span.start + st.span.start, end: span.start + st.span.end },
                    })
                    .collect(),
                comment: s.comments.first().map(|c| c.text.trim().to_string()),
            },
            Err(e) => Line::Unread { text: raw.trim_end_matches('\r').to_string(), error: e.message },
        };
        out.push((line, span));
    }
    out
}

fn statement_row(src: &str, st: &fractadyne_core::ir::syntax::Statement) -> Row {
    let mut row = Vec::new();
    if let Some(t) = &st.target {
        row.push(Atom::Word(src[t.span.start..t.span.end].to_string()));
        row.push(Atom::Op(Op::Eq));
    }
    row.extend(flatten(src, &st.body));
    row
}

/// An expression as a row. Fractions and exponents take their operands as rows of their own, and
/// parentheses that only grouped an operand for them are dropped (the structure groups it).
pub(crate) fn flatten(src: &str, e: &Expr) -> Row {
    let text = |s: Span| src[s.start..s.end].to_string();
    let unwrap = |e: &Expr| match &e.kind {
        ExprKind::Group(inner) => flatten(src, inner),
        _ => flatten(src, e),
    };
    match &e.kind {
        ExprKind::Num(_) => vec![Atom::Number(text(e.span))],
        ExprKind::Name(_) => vec![Atom::Word(text(e.span))],
        ExprKind::Call { arg, .. } => {
            let name = text(e.span).split('(').next().unwrap_or("").trim().to_string();
            vec![Atom::Func { name, arg: flatten(src, arg) }]
        }
        ExprKind::Group(inner) => vec![Atom::Group(flatten(src, inner))],
        ExprKind::Complex(a, b) => vec![Atom::Complex { re: flatten(src, a), im: flatten(src, b) }],
        ExprKind::Bars(inner) => vec![Atom::Bars(flatten(src, inner))],
        ExprKind::Neg(inner) => [vec![Atom::Op(Op::Minus)], flatten(src, inner)].concat(),
        ExprKind::Pos(inner) => [vec![Atom::Op(Op::Plus)], flatten(src, inner)].concat(),
        ExprKind::Bin(BinOp::Div, l, r) => vec![Atom::Frac { num: unwrap(l), den: unwrap(r) }],
        ExprKind::Bin(op, l, r) => {
            let op = match op {
                BinOp::Add => Op::Plus,
                BinOp::Sub => Op::Minus,
                _ => Op::Times,
            };
            [flatten(src, l), vec![Atom::Op(op)], flatten(src, r)].concat()
        }
        ExprKind::Pow(b, x) => [flatten(src, b), vec![Atom::Sup(unwrap(x))]].concat(),
    }
}

// The printer: the editor (P3) prints the rows it edits; until then the tests use it.

/// A primary: what the grammar's `atom` reads, so it can be an operand without parentheses.
#[cfg_attr(not(test), allow(dead_code))]
fn primary(a: &Atom) -> bool {
    matches!(a, Atom::Word(_) | Atom::Number(_) | Atom::Group(_) | Atom::Func { .. } | Atom::Bars(_) | Atom::Complex { .. })
}

/// Unary signs, then one primary, then at most one exponent: what the grammar's `unary` reads —
/// an exponent, or a denominator, needs no parentheses round it.
#[cfg_attr(not(test), allow(dead_code))]
fn unary_level(row: &[Atom]) -> bool {
    let rest = &row[row.iter().take_while(|a| matches!(a, Atom::Op(Op::Plus | Op::Minus))).count()..];
    match rest {
        [p] => primary(p),
        [p, Atom::Sup(_)] => primary(p),
        _ => false,
    }
}

/// No `+`, `−` or `=` between operands: a numerator needs no parentheses round it.
#[cfg_attr(not(test), allow(dead_code))]
fn term_level(row: &[Atom]) -> bool {
    row.iter().enumerate().all(|(i, a)| match a {
        Atom::Op(Op::Eq) => false,
        Atom::Op(Op::Plus | Op::Minus) => is_unary(row, i),
        _ => true,
    })
}

/// A `+`/`−` with no operand on its left: at the start, or after another operator.
#[cfg_attr(not(test), allow(dead_code))]
fn is_unary(row: &[Atom], i: usize) -> bool {
    i == 0 || matches!(row[i - 1], Atom::Op(_))
}

/// A row as text, with the parentheses the grammar needs to read it back as this row and no others.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn print_row(row: &[Atom]) -> String {
    let mut s = String::new();
    for (i, a) in row.iter().enumerate() {
        let as_base = matches!(row.get(i + 1), Some(Atom::Sup(_)));
        let after_times_or_sign = i > 0
            && match row[i - 1] {
                Atom::Op(Op::Times) => true,
                Atom::Op(Op::Plus | Op::Minus) => is_unary(row, i - 1),
                _ => false,
            };
        let printed = print_atom(row, i);
        let wrap = match a {
            // `x*a/b` reads (x*a)/b, `-a/b` reads (-a)/b: a fraction there is its own operand.
            Atom::Frac { .. } => as_base || after_times_or_sign,
            _ => as_base && !primary(a),
        };
        if wrap {
            s.push('(');
            s.push_str(&printed);
            s.push(')');
        } else {
            s.push_str(&printed);
        }
    }
    s
}

#[cfg_attr(not(test), allow(dead_code))]
fn print_atom(row: &[Atom], i: usize) -> String {
    let wrap_if = |cond: bool, r: &[Atom]| if cond { format!("({})", print_row(r)) } else { print_row(r) };
    match &row[i] {
        Atom::Word(w) | Atom::Number(w) => w.clone(),
        Atom::Op(Op::Plus) if is_unary(row, i) => "+".into(),
        Atom::Op(Op::Minus) if is_unary(row, i) => "-".into(),
        Atom::Op(Op::Plus) => " + ".into(),
        Atom::Op(Op::Minus) => " - ".into(),
        Atom::Op(Op::Times) => "*".into(),
        Atom::Op(Op::Eq) => " = ".into(),
        Atom::Frac { num, den } => {
            format!("{}/{}", wrap_if(!term_level(num) || num.is_empty(), num), wrap_if(!unary_level(den), den))
        }
        Atom::Sup(x) => format!("^{}", wrap_if(!unary_level(x), x)),
        Atom::Group(r) => format!("({})", print_row(r)),
        Atom::Func { name, arg } => format!("{name}({})", print_row(arg)),
        Atom::Bars(r) => format!("|{}|", print_row(r)),
        Atom::Complex { re, im } => format!("({}, {})", print_row(re), print_row(im)),
        Atom::Slot => String::new(),
    }
}

// ---- Truthful textbook notation (§4.5) ----

fn paren(body: Vec<Node>) -> Node {
    Node::Fenced { open: '(', close: ')', body }
}

/// A name: lower-cased (the language ignores case), italic, its trailing digits a subscript (𝑝₁).
fn word(w: &str) -> Node {
    let w = w.to_ascii_lowercase();
    match w.as_str() {
        "pi" => Node::glyphs("\u{1D70B}", Class::Ord),
        _ => {
            let letters = w.trim_end_matches(|c: char| c.is_ascii_digit());
            let digits = &w[letters.len()..];
            if letters.is_empty() || digits.is_empty() {
                Node::var(&w)
            } else {
                Node::Scripts { base: Box::new(Node::var(letters)), sup: None, sub: Some(vec![Node::num(digits)]) }
            }
        }
    }
}

/// A number as written; scientific notation as a × 10ᵇ.
fn number(t: &str) -> Vec<Node> {
    match t.split_once(['e', 'E']) {
        Some((m, x)) => {
            let mut exp = Vec::new();
            let x = match x.strip_prefix('-') {
                Some(rest) => {
                    exp.push(Node::bin('\u{2212}'));
                    rest
                }
                None => x.trim_start_matches('+'),
            };
            exp.push(Node::num(x));
            vec![
                Node::num(m),
                Node::glyphs("\u{00D7}", Class::Bin),
                Node::Scripts { base: Box::new(Node::num("10")), sup: Some(exp), sub: None },
            ]
        }
        None => vec![Node::num(t)],
    }
}

/// An exponent on the last node: on a fraction, a radical or something already raised, the base
/// takes parentheses — (1/𝑧)², not 1/𝑧².
fn attach_sup(out: &mut Vec<Node>, sup: Vec<Node>) {
    let base = out.pop().unwrap_or(Node::Slot);
    let base = match base {
        Node::Scripts { base, sup: None, sub } => {
            out.push(Node::Scripts { base, sup: Some(sup), sub });
            return;
        }
        b @ (Node::Frac { .. } | Node::Radical(_) | Node::Scripts { .. }) => paren(vec![b]),
        b => b,
    };
    out.push(Node::Scripts { base: Box::new(base), sup: Some(sup), sub: None });
}

/// A function in its textbook notation — but only where that notation computes the same thing.
fn func(name: &str, arg: &Row) -> Vec<Node> {
    let body = math_row(arg);
    let operand = |body: Vec<Node>| match body.as_slice() {
        [n @ (Node::Glyphs { .. } | Node::Fenced { .. })] => n.clone(),
        _ => paren(body),
    };
    match name.to_ascii_lowercase().as_str() {
        "sqr" => vec![Node::Scripts { base: Box::new(operand(body)), sup: Some(vec![Node::num("2")]), sub: None }],
        "sqrt" => vec![Node::Radical(body)],
        "cabs" => vec![Node::Fenced { open: '|', close: '|', body }],
        "conj" => vec![Node::Overline(body)],
        "recip" => vec![Node::Frac { num: vec![Node::num("1")], den: body }],
        "real" => vec![Node::op("Re"), paren(body)],
        "imag" => vec![Node::op("Im"), paren(body)],
        "cotan" => vec![Node::op("cot"), paren(body)],
        "cotanh" => vec![Node::op("coth"), paren(body)],
        // exp stays exp: e^z is a power with a varying exponent, a different computation.
        n => vec![Node::op(n), paren(body)],
    }
}

/// A product shown by juxtaposition, except where that would misread: before a number (𝑧·2, not
/// the name 𝑧2), and between a number and a fraction (2·½, not the mixed number 2½).
fn needs_dot(left: Option<&Atom>, right: Option<&Atom>) -> bool {
    let num = |a: Option<&Atom>| matches!(a, Some(Atom::Number(_)));
    let frac = |a: Option<&Atom>| matches!(a, Some(Atom::Frac { .. }));
    num(right) || (num(left) && frac(right)) || (frac(left) && num(right))
}

/// A row as the layout's math list.
pub(crate) fn math_row(row: &[Atom]) -> Vec<Node> {
    let mut out: Vec<Node> = Vec::new();
    for (i, a) in row.iter().enumerate() {
        match a {
            Atom::Word(w) => out.push(word(w)),
            Atom::Number(t) => out.extend(number(t)),
            Atom::Op(Op::Plus) => out.push(Node::bin('+')),
            Atom::Op(Op::Minus) => out.push(Node::bin('\u{2212}')),
            Atom::Op(Op::Eq) => out.push(Node::rel('=')),
            Atom::Op(Op::Times) => {
                if needs_dot(i.checked_sub(1).and_then(|j| row.get(j)), row.get(i + 1)) {
                    out.push(Node::bin('\u{22C5}'));
                }
            }
            Atom::Frac { num, den } => out.push(Node::Frac { num: math_row(num), den: math_row(den) }),
            Atom::Sup(x) => attach_sup(&mut out, math_row(x)),
            Atom::Group(r) => out.push(paren(math_row(r))),
            Atom::Func { name, arg } => out.extend(func(name, arg)),
            Atom::Bars(r) => out.push(Node::Scripts {
                base: Box::new(Node::Fenced { open: '|', close: '|', body: math_row(r) }),
                sup: Some(vec![Node::num("2")]),
                sub: None,
            }),
            Atom::Complex { re, im } => {
                let mut body = math_row(re);
                let (sign, mag) = match im.split_first() {
                    Some((Atom::Op(Op::Minus), rest)) => ('\u{2212}', rest),
                    _ => ('+', im.as_slice()),
                };
                body.push(Node::bin(sign));
                body.extend(math_row(mag));
                body.push(Node::var("i"));
                out.push(paren(body));
            }
            Atom::Slot => out.push(Node::Slot),
        }
    }
    out
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
