//! The textbook editor's model of a formula (design/formula-textbook-editor.md §4.3–4.5): rows of
//! atoms in which standard precedence applies, as in the text, with the structures that need two
//! dimensions — fractions, exponents, parentheses, functions — holding rows of their own.
//!
//! Names and numbers are runs of single characters, as MathQuill keeps them, so every place a caret
//! can stand is a boundary between atoms. A run reads as the lexer reads the same text
//! ([`run_tokens`]): `p1` is one name, `2z` a number and a name — two factors, so a product, which
//! the printer writes out (`2*z`).
//!
//! - [`read`]: a source into lines of statement rows, from the core's syntax tree. Lines are read
//!   one at a time (a statement never spans one: a line break always ends it), so a line that does
//!   not read does not hide the others.
//! - [`print_row`]: a row back to text, with the parentheses the grammar needs and no others.
//! - [`normalize`]: a row an edit left back in the grammar's form.
//! - [`emit`]: a row as the layout's math list, in truthful textbook notation (§4.5), with the
//!   caret's places marked.

use super::layout::{Class, Node};
use fractadyne_core::ir::parse::{is_function, syntax};
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
    /// A letter, digit, `.` or `_` of a name or a number — or the sign of a number's exponent
    /// (`1e-5`): one caret step each.
    Char(char),
    Op(Op),
    Frac { num: Row, den: Row },
    /// An exponent on what comes before it.
    Sup(Row),
    /// Parentheses the user wrote (not the ones a fraction or an exponent groups by itself).
    Group(Row),
    /// A named function; the name as written.
    Func { name: String, arg: Row },
    /// `|x|`, the squared modulus.
    Bars(Row),
    Complex { re: Row, im: Row },
}

pub(crate) type Row = Vec<Atom>;

impl Atom {
    /// How many rows it holds.
    pub(crate) fn arity(&self) -> u8 {
        match self {
            Atom::Char(_) | Atom::Op(_) => 0,
            Atom::Frac { .. } | Atom::Complex { .. } => 2,
            _ => 1,
        }
    }

    /// Its rows in caret order: a numerator before its denominator.
    pub(crate) fn child(&self, k: u8) -> Option<&Row> {
        match (self, k) {
            (Atom::Frac { num: r, .. } | Atom::Complex { re: r, .. }, 0) => Some(r),
            (Atom::Frac { den: r, .. } | Atom::Complex { im: r, .. }, 1) => Some(r),
            (Atom::Sup(r) | Atom::Group(r) | Atom::Bars(r) | Atom::Func { arg: r, .. }, 0) => Some(r),
            _ => None,
        }
    }

    pub(crate) fn child_mut(&mut self, k: u8) -> Option<&mut Row> {
        match (self, k) {
            (Atom::Frac { num: r, .. } | Atom::Complex { re: r, .. }, 0) => Some(r),
            (Atom::Frac { den: r, .. } | Atom::Complex { im: r, .. }, 1) => Some(r),
            (Atom::Sup(r) | Atom::Group(r) | Atom::Bars(r) | Atom::Func { arg: r, .. }, 0) => Some(r),
            _ => None,
        }
    }
}

/// The atoms of `text`, one per character.
pub(crate) fn chars(text: &str) -> Row {
    text.chars().map(Atom::Char).collect()
}

/// A place in a statement: the line and the statement on it, the path from the statement's row
/// down — (atom index, which of its rows) — and the position between atoms of that row.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Caret {
    pub(crate) line: usize,
    pub(crate) stmt: usize,
    pub(crate) path: Vec<(usize, u8)>,
    pub(crate) pos: usize,
}

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

/// The source's non-blank lines, with each one's byte range (the editor reads a line at a time,
/// through `edit::Doc`; the tests read whole sources).
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn read(src: &str) -> Vec<(Line, Span)> {
    let mut out = Vec::new();
    let mut start = 0;
    for raw in src.split('\n') {
        let span = Span { start, end: start + raw.len() };
        start = span.end + 1;
        if raw.trim().is_empty() {
            continue;
        }
        out.push((read_line(raw, span.start), span));
    }
    out
}

/// One line (no line break in it) whose first byte is byte `at` of the source.
pub(crate) fn read_line(raw: &str, at: usize) -> Line {
    match syntax(raw) {
        Ok(s) => Line::Read {
            stmts: s
                .statements
                .iter()
                .map(|st| Stmt { row: statement_row(raw, st), span: Span { start: at + st.span.start, end: at + st.span.end } })
                .collect(),
            comment: s.comments.first().map(|c| c.text.trim().to_string()),
        },
        Err(e) => Line::Unread { text: raw.trim_end_matches('\r').to_string(), error: e.message },
    }
}

fn statement_row(src: &str, st: &fractadyne_core::ir::syntax::Statement) -> Row {
    let mut row = Vec::new();
    if let Some(t) = &st.target {
        row.extend(chars(&src[t.span.start..t.span.end]));
        row.push(Atom::Op(Op::Eq));
    }
    row.extend(flatten(src, &st.body));
    row
}

/// An expression as a row. Fractions and exponents take their operands as rows of their own, and
/// parentheses that only grouped an operand for them are dropped (the structure groups it). A `*`
/// the printer would write anyway (`2*z`, `z*(z + 1)`) is left implied, so no caret step is
/// invisible; one it would not (`z*c`, not the name `zc`) stays.
pub(crate) fn flatten(src: &str, e: &Expr) -> Row {
    let text = |s: Span| &src[s.start..s.end];
    let unwrap = |e: &Expr| match &e.kind {
        ExprKind::Group(inner) => flatten(src, inner),
        _ => flatten(src, e),
    };
    match &e.kind {
        ExprKind::Num(_) | ExprKind::Name(_) => chars(text(e.span)),
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
            let (l, r) = (flatten(src, l), flatten(src, r));
            if op == Op::Times && implied(&l, &r) {
                [l, r].concat()
            } else {
                [l, vec![Atom::Op(op)], r].concat()
            }
        }
        ExprKind::Pow(b, x) => [flatten(src, b), vec![Atom::Sup(unwrap(x))]].concat(),
    }
}

// ---- Reading runs of characters as the lexer does ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Number,
    Name,
    /// A `+` or `−` left in a run (an exponent's sign before its digits are typed).
    Sign,
}

/// A token of a run of characters: atoms `start..end` of its row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Token {
    pub(crate) kind: Kind,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

fn char_at(row: &[Atom], i: usize) -> Option<char> {
    match row.get(i) {
        Some(Atom::Char(c)) => Some(*c),
        _ => None,
    }
}

/// The tokens of the run of characters starting at `start`, read as the lexer reads the same text
/// (`fractadyne_core::ir::parse`), and where the run ends.
pub(crate) fn run_tokens(row: &[Atom], start: usize) -> (Vec<Token>, usize) {
    let ch = |i: usize| char_at(row, i);
    let mut end = start;
    while ch(end).is_some() {
        end += 1;
    }
    let digit = |i: usize| i < end && ch(i).is_some_and(|c| c.is_ascii_digit());
    let mut out = Vec::new();
    let mut i = start;
    while i < end {
        let c = ch(i).unwrap_or(' ');
        let s = i;
        let kind = if c.is_ascii_digit() || c == '.' {
            while i < end && ch(i).is_some_and(|c| c.is_ascii_digit() || c == '.') {
                i += 1;
            }
            if i < end && matches!(ch(i), Some('e' | 'E')) {
                let mut k = i + 1;
                if k < end && matches!(ch(k), Some('+' | '-')) {
                    k += 1;
                }
                if digit(k) {
                    while digit(k) {
                        k += 1;
                    }
                    i = k;
                }
            }
            Kind::Number
        } else if c.is_ascii_alphabetic() || c == '_' {
            while i < end && ch(i).is_some_and(|c| c.is_ascii_alphanumeric() || c == '_') {
                i += 1;
            }
            Kind::Name
        } else {
            i += 1;
            Kind::Sign
        };
        out.push(Token { kind, start: s, end: i });
    }
    (out, end)
}

fn token_text(row: &[Atom], t: Token) -> String {
    (t.start..t.end).filter_map(|i| char_at(row, i)).collect()
}

/// Where the run of characters holding atom `i` starts.
pub(crate) fn run_start(row: &[Atom], i: usize) -> usize {
    let mut s = i;
    while s > 0 && char_at(row, s - 1).is_some() {
        s -= 1;
    }
    s
}

/// The token of `row` that ends exactly at `end` (`None` if `end` is not after a character).
pub(crate) fn token_ending_at(row: &[Atom], end: usize) -> Option<(Token, String)> {
    if end == 0 || char_at(row, end - 1).is_none() {
        return None;
    }
    let (toks, _) = run_tokens(&row[..end], run_start(row, end - 1));
    toks.last().filter(|t| t.end == end).map(|&t| (t, token_text(row, t)))
}

/// A `+`/`-` at `i` would be the sign of a number's exponent: right after the `e` of a number
/// that has none yet (`1e-5`), as the lexer reads it.
pub(crate) fn exponent_sign_at(row: &[Atom], i: usize) -> bool {
    i >= 2
        && matches!(char_at(row, i - 1), Some('e' | 'E'))
        && token_ending_at(row, i - 1).is_some_and(|(t, text)| t.kind == Kind::Number && !text.contains(['e', 'E']))
}

// ---- The printer ----

/// A row as the parser reads it: whole names and numbers, every product written out.
#[derive(Clone, Debug, PartialEq)]
enum Item<'a> {
    Text(Kind, String),
    Op(Op),
    At(&'a Atom),
}

fn ends_factor(it: &Item) -> bool {
    !matches!(it, Item::Op(_))
}

fn starts_factor(it: &Item) -> bool {
    matches!(it, Item::Text(..)) || matches!(it, Item::At(a) if !matches!(a, Atom::Sup(_)))
}

/// Push a factor, with the `*` before it if it follows another.
fn push<'a>(out: &mut Vec<Item<'a>>, it: Item<'a>) {
    if starts_factor(&it) && out.last().is_some_and(ends_factor) {
        out.push(Item::Op(Op::Times));
    }
    out.push(it);
}

fn items(row: &[Atom]) -> Vec<Item<'_>> {
    let mut out: Vec<Item> = Vec::new();
    let mut i = 0;
    while i < row.len() {
        match &row[i] {
            Atom::Char(_) => {
                let (toks, end) = run_tokens(row, i);
                for t in toks {
                    match t.kind {
                        Kind::Sign if char_at(row, t.start) == Some('-') => out.push(Item::Op(Op::Minus)),
                        Kind::Sign => out.push(Item::Op(Op::Plus)),
                        k => push(&mut out, Item::Text(k, token_text(row, t))),
                    }
                }
                i = end;
            }
            Atom::Op(o) => {
                out.push(Item::Op(*o));
                i += 1;
            }
            a => {
                push(&mut out, Item::At(a));
                i += 1;
            }
        }
    }
    out
}

/// `l` then `r` read as `l*r` already: the `*` between them can be left implied. Not where the
/// two would run together (`z*c`, `2*3`), and not a function's name before parentheses, which
/// would call it.
fn implied(l: &[Atom], r: &[Atom]) -> bool {
    if l.is_empty() || r.is_empty() {
        return false;
    }
    let calls = matches!(r[0], Atom::Group(_) | Atom::Complex { .. })
        && token_ending_at(l, l.len()).is_some_and(|(t, name)| t.kind == Kind::Name && is_function(&name));
    let times = [Atom::Op(Op::Times)];
    !calls && items(&[l, r].concat()) == items(&[l, &times[..], r].concat())
}

/// A primary: what the grammar's `atom` reads, so it can be an operand without parentheses.
fn primary(it: &Item) -> bool {
    matches!(it, Item::Text(..) | Item::At(Atom::Group(_) | Atom::Func { .. } | Atom::Bars(_) | Atom::Complex { .. }))
}

/// A row the grammar reads as one primary (a name, a number, parentheses, a call, bars): it can be
/// raised without parentheses round it.
pub(crate) fn single_primary(row: &[Atom]) -> bool {
    matches!(&items(row)[..], [p] if primary(p))
}

/// Unary signs, then one primary, then at most one exponent: what the grammar's `unary` reads —
/// an exponent, or a denominator, needs no parentheses round it.
fn unary_level(row: &[Item]) -> bool {
    let rest = &row[row.iter().take_while(|a| matches!(a, Item::Op(Op::Plus | Op::Minus))).count()..];
    match rest {
        [p] => primary(p),
        [p, Item::At(Atom::Sup(_))] => primary(p),
        _ => false,
    }
}

/// No `+`, `−` or `=` between operands: a numerator needs no parentheses round it.
fn term_level(row: &[Item]) -> bool {
    row.iter().enumerate().all(|(i, a)| match a {
        Item::Op(Op::Eq) => false,
        Item::Op(Op::Plus | Op::Minus) => is_unary(row, i),
        _ => true,
    })
}

/// A `+`/`−` with no operand on its left: at the start, or after another operator.
fn is_unary(row: &[Item], i: usize) -> bool {
    i == 0 || matches!(row[i - 1], Item::Op(_))
}

/// A row as text, with the parentheses the grammar needs to read it back as this row and no others.
pub(crate) fn print_row(row: &[Atom]) -> String {
    print_items(&items(row))
}

fn print_items(row: &[Item]) -> String {
    let mut s = String::new();
    for (i, a) in row.iter().enumerate() {
        let as_base = matches!(row.get(i + 1), Some(Item::At(Atom::Sup(_))));
        let after_times_or_sign = i > 0
            && match row[i - 1] {
                Item::Op(Op::Times) => true,
                Item::Op(Op::Plus | Op::Minus) => is_unary(row, i - 1),
                _ => false,
            };
        let printed = print_item(row, i);
        let wrap = match a {
            // `x*a/b` reads (x*a)/b, `-a/b` reads (-a)/b: a fraction there is its own operand.
            Item::At(Atom::Frac { .. }) => as_base || after_times_or_sign,
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

fn print_item(row: &[Item], i: usize) -> String {
    let wrap_if = |cond: bool, r: &[Item]| if cond { format!("({})", print_items(r)) } else { print_items(r) };
    match &row[i] {
        Item::Text(_, t) => t.clone(),
        Item::Op(Op::Plus) if is_unary(row, i) => "+".into(),
        Item::Op(Op::Minus) if is_unary(row, i) => "-".into(),
        Item::Op(Op::Plus) => " + ".into(),
        Item::Op(Op::Minus) => " - ".into(),
        Item::Op(Op::Times) => "*".into(),
        Item::Op(Op::Eq) => " = ".into(),
        Item::At(a) => match a {
            Atom::Frac { num, den } => {
                let (num, den) = (items(num), items(den));
                format!("{}/{}", wrap_if(!term_level(&num) || num.is_empty(), &num), wrap_if(!unary_level(&den), &den))
            }
            Atom::Sup(x) => {
                let x = items(x);
                format!("^{}", wrap_if(!unary_level(&x), &x))
            }
            Atom::Group(r) => format!("({})", print_row(r)),
            Atom::Func { name, arg } => format!("{name}({})", print_row(arg)),
            Atom::Bars(r) => format!("|{}|", print_row(r)),
            Atom::Complex { re, im } => format!("({}, {})", print_row(re), print_row(im)),
            Atom::Char(_) | Atom::Op(_) => String::new(),
        },
    }
}

// ---- Normalisation: a row an edit left, back in the grammar's form ----

/// Where a caret is relative to one row: at a position in it, or inside one of its atoms.
enum Rel {
    At(usize),
    In(usize),
}

fn rel(c: &Caret, prefix: &[(usize, u8)]) -> Option<Rel> {
    if !c.path.starts_with(prefix) {
        return None;
    }
    Some(match c.path.get(prefix.len()) {
        None => Rel::At(c.pos),
        Some(&(i, _)) => Rel::In(i),
    })
}

/// Move every caret in the row at `prefix` with `f`, which maps an atom index or position of the
/// old row to the new one (and may send a caret into a new row below it).
fn remap(carets: &mut [&mut Caret], prefix: &[(usize, u8)], f: impl Fn(Rel) -> (Vec<(usize, u8)>, usize)) {
    let d = prefix.len();
    for c in carets.iter_mut() {
        match rel(c, prefix) {
            Some(Rel::At(p)) => {
                let (down, pos) = f(Rel::At(p));
                c.path.truncate(d);
                c.path.extend(down);
                c.pos = pos;
            }
            Some(Rel::In(i)) => {
                // `pos` is the atom's new index; `down` (if any) the rows it now sits in.
                let (down, j) = f(Rel::In(i));
                let rest: Vec<(usize, u8)> = c.path[d + 1..].to_vec();
                let k = c.path[d].1;
                c.path.truncate(d);
                c.path.extend(down);
                c.path.push((j, k));
                c.path.extend(rest);
            }
            None => {}
        }
    }
}

/// Put a row an edit left back in the grammar's form, keeping the carets where they were:
/// - a `+`/`-` typed as a character becomes an operator, unless it is a number's exponent sign;
/// - a `*` the printer would write anyway is left implied (no invisible caret steps);
/// - a function's name before parentheses becomes a call (`sin` + `(z)` → `sin(z)`);
/// - an exponent on an exponent raises the whole power: `(z²)³` (the text `z^2^3` is z^(2^3)).
pub(crate) fn normalize(row: &mut Row, prefix: &mut Vec<(usize, u8)>, carets: &mut [&mut Caret]) {
    for i in 0..row.len() {
        for k in 0..row[i].arity() {
            prefix.push((i, k));
            if let Some(r) = row[i].child_mut(k) {
                normalize(r, prefix, carets);
            }
            prefix.pop();
        }
    }
    // Signs.
    for i in 0..row.len() {
        if let Atom::Char(c @ ('+' | '-')) = row[i] {
            if !exponent_sign_at(row, i) {
                row[i] = Atom::Op(if c == '-' { Op::Minus } else { Op::Plus });
            }
        }
    }
    // Implied products.
    let mut i = 1;
    while i + 1 < row.len() {
        if row[i] == Atom::Op(Op::Times) {
            let l0 = if char_at(row, i - 1).is_some() { run_start(row, i - 1) } else { i - 1 };
            let r1 = if char_at(row, i + 1).is_some() { run_tokens(row, i + 1).1 } else { i + 2 };
            if implied(&row[l0..i], &row[i + 1..r1]) {
                row.remove(i);
                remap(carets, prefix, |r| match r {
                    Rel::At(p) => (vec![], if p > i { p - 1 } else { p }),
                    Rel::In(j) => (vec![], if j > i { j - 1 } else { j }),
                });
                continue;
            }
        }
        i += 1;
    }
    // Calls.
    let mut e = 1;
    while e < row.len() {
        let call = match row[e] {
            Atom::Group(_) => token_ending_at(row, e).filter(|(t, name)| t.kind == Kind::Name && is_function(name)),
            _ => None,
        };
        if let Some((t, name)) = call {
            let s = t.start;
            let Atom::Group(arg) = row.remove(e) else { unreachable!() };
            row.splice(s..e, [Atom::Func { name, arg }]);
            let n = e - s;
            remap(carets, prefix, |r| match r {
                Rel::At(p) if p <= s => (vec![], p),
                Rel::At(p) if p < e => (vec![], s),
                Rel::At(p) if p == e => (vec![(s, 0)], 0),
                Rel::At(p) => (vec![], p - n),
                Rel::In(j) if j < s => (vec![], j),
                Rel::In(j) if j == e => (vec![], s),
                Rel::In(j) => (vec![], j - n),
            });
            e = s + 1;
            continue;
        }
        e += 1;
    }
    // Powers of powers.
    let mut i = 1;
    while i < row.len() {
        if matches!(row[i], Atom::Sup(_)) && matches!(row[i - 1], Atom::Sup(_)) {
            let b = base_start(row, i - 1);
            let inner: Row = row.drain(b..i).collect();
            row.insert(b, Atom::Group(inner));
            let n = i - b; // atoms that went into the group
            remap(carets, prefix, |r| match r {
                Rel::At(p) if p <= b => (vec![], p),
                Rel::At(p) if p < i => (vec![(b, 0)], p - b),
                Rel::At(p) => (vec![], p - n + 1),
                Rel::In(j) if j < b => (vec![], j),
                Rel::In(j) if j < i => (vec![(b, 0)], j - b),
                Rel::In(j) => (vec![], j - n + 1),
            });
            i = b + 2;
            continue;
        }
        i += 1;
    }
}

/// Where the base of an exponent at `sup` starts: the name or number before it, or the structure
/// (nothing, if an operator or the row's start is before it).
pub(crate) fn base_start(row: &[Atom], sup: usize) -> usize {
    match sup.checked_sub(1).map(|j| &row[j]) {
        Some(Atom::Char(_)) => token_ending_at(row, sup).map_or(sup, |(t, _)| if t.kind == Kind::Sign { sup } else { t.start }),
        Some(Atom::Op(_)) | None => sup,
        Some(_) => sup - 1,
    }
}

// ---- Truthful textbook notation (§4.5), with the caret's places ----

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
/// takes parentheses — (1/𝑧)², not 1/𝑧². `mark` is the place between base and exponent: at the
/// base's end.
fn attach_sup(out: &mut Vec<Node>, sup: Vec<Node>, mark: Option<u32>) {
    let base = out.pop().unwrap_or(Node::Slot(None));
    let base = match base {
        // 𝑝₁²: the exponent over the subscript (the place before it is the name's end, which
        // shows as typed whenever the caret is there).
        Node::Scripts { base, sup: None, sub } => {
            out.push(Node::Scripts { base, sup: Some(sup), sub });
            return;
        }
        b @ (Node::Frac { .. } | Node::Radical(_) | Node::Scripts { .. }) => paren(vec![b]),
        b => b,
    };
    let base = match (base, mark) {
        (Node::Glyphs { text, class, mut anchors }, Some(id)) => {
            anchors.push((text.chars().count(), id));
            Node::Glyphs { text, class, anchors }
        }
        (b, Some(id)) => Node::Row(vec![b, Node::Mark(id)]),
        (b, None) => b,
    };
    out.push(Node::Scripts { base: Box::new(base), sup: Some(sup), sub: None });
}

/// The nodes besides marks.
fn real(nodes: &[Node]) -> impl Iterator<Item = &Node> {
    nodes.iter().filter(|n| !matches!(n, Node::Mark(_)))
}

/// A function in its textbook notation — but only where that notation computes the same thing.
fn func(name: &str, body: Vec<Node>) -> Vec<Node> {
    let operand = |body: Vec<Node>| {
        let single = {
            let mut r = real(&body);
            matches!((r.next(), r.next()), (Some(Node::Glyphs { .. } | Node::Fenced { .. }), None))
        };
        if single {
            Node::Row(body)
        } else {
            paren(body)
        }
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

/// Which marks to make and where they stand: each mark id's place, and the places whose name or
/// number shows as typed (the caret's and the selection's other end: 𝑝1 while it is being typed,
/// 𝑝₁ once the caret leaves it).
#[derive(Default)]
pub(crate) struct Marker {
    pub(crate) places: Vec<Caret>,
    on: bool,
    line: usize,
    stmt: usize,
    raw: Vec<Caret>,
}

impl Marker {
    pub(crate) fn new(raw: Vec<Caret>) -> Marker {
        Marker { on: true, raw, ..Default::default() }
    }

    /// The statement the next rows belong to.
    pub(crate) fn statement(&mut self, line: usize, stmt: usize) {
        (self.line, self.stmt) = (line, stmt);
    }

    fn mark(&mut self, path: &[(usize, u8)], pos: usize) -> Option<u32> {
        if !self.on {
            return None;
        }
        self.places.push(Caret { line: self.line, stmt: self.stmt, path: path.to_vec(), pos });
        Some(self.places.len() as u32 - 1)
    }

    fn here<'a>(&'a self, path: &'a [(usize, u8)]) -> impl Iterator<Item = &'a Caret> + 'a {
        self.raw.iter().filter(move |c| c.line == self.line && c.stmt == self.stmt && c.path.starts_with(path))
    }

    /// A caret touches atoms `start..end` of the row at `path`.
    fn touches(&self, path: &[(usize, u8)], start: usize, end: usize) -> bool {
        self.here(path).any(|c| c.path.len() == path.len() && (start..=end).contains(&c.pos))
    }

    /// A caret is inside atom `i` of the row at `path`.
    fn inside(&self, path: &[(usize, u8)], i: usize) -> bool {
        self.here(path).any(|c| c.path.get(path.len()).is_some_and(|&(j, _)| j == i))
    }
}

/// How a unit looks at its edges, for the products that need a dot to read right.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Look {
    Num,
    Name,
    Frac,
    Other,
}

/// What a row shows, unit by unit: a name or number, an operator, a structure.
#[derive(Clone, Copy)]
enum Unit {
    Tok(Token),
    Op(usize, Op),
    At(usize),
}

impl Unit {
    fn start(self) -> usize {
        match self {
            Unit::Tok(t) => t.start,
            Unit::Op(i, _) | Unit::At(i) => i,
        }
    }
}

fn units(row: &[Atom]) -> Vec<Unit> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < row.len() {
        match &row[i] {
            Atom::Char(_) => {
                let (toks, end) = run_tokens(row, i);
                out.extend(toks.into_iter().map(Unit::Tok));
                i = end;
            }
            Atom::Op(o) => {
                out.push(Unit::Op(i, *o));
                i += 1;
            }
            _ => {
                out.push(Unit::At(i));
                i += 1;
            }
        }
    }
    out
}

/// How unit `u` looks at its edges. `left`: the edge a following unit meets — where a name shown
/// with its digits as a subscript (𝑝₁) has ended unmistakably, unless it shows as typed (`raw`).
fn look(row: &[Atom], u: Unit, left: bool, raw: bool) -> Look {
    match u {
        Unit::Tok(t) if t.kind == Kind::Number => Look::Num,
        Unit::Tok(t) if t.kind == Kind::Name => {
            let text = token_text(row, t);
            let subscripted = text.ends_with(|c: char| c.is_ascii_digit()) && text.starts_with(|c: char| c.is_ascii_alphabetic());
            if text.eq_ignore_ascii_case("pi") || (left && subscripted && !raw) {
                Look::Other
            } else {
                Look::Name
            }
        }
        Unit::At(i) => match &row[i] {
            Atom::Frac { .. } => Look::Frac,
            Atom::Func { name, .. } if name.eq_ignore_ascii_case("recip") => Look::Frac,
            _ => Look::Other,
        },
        _ => Look::Other,
    }
}

fn is_factor_end(u: Unit) -> bool {
    !matches!(u, Unit::Op(..) | Unit::Tok(Token { kind: Kind::Sign, .. }))
}

fn is_factor_start(row: &[Atom], u: Unit) -> bool {
    match u {
        Unit::Tok(t) => t.kind != Kind::Sign,
        Unit::At(i) => !matches!(row[i], Atom::Sup(_)),
        Unit::Op(..) => false,
    }
}

/// A product shown by juxtaposition, except where that would misread: before a number (𝑧·2, not
/// the name 𝑧2), between a number and a fraction (2·½, not the mixed number 2½), between two names
/// (𝑧·𝑐, not the name 𝑧𝑐).
fn needs_dot(l: Look, r: Look) -> bool {
    r == Look::Num || (l, r) == (Look::Num, Look::Frac) || (l, r) == (Look::Frac, Look::Num) || (l, r) == (Look::Name, Look::Name)
}

/// A row as the layout's math list (no caret places).
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn math_row(row: &[Atom]) -> Vec<Node> {
    emit(row, &mut Vec::new(), &mut Marker::default(), false)
}

/// A row as the layout's math list, with a mark at every place the caret can stand. `top`: a
/// statement's own row, which shows nothing when it is empty unless the caret is in it (an empty
/// row inside a structure is a place still to fill: a slot).
pub(crate) fn emit(row: &[Atom], path: &mut Vec<(usize, u8)>, m: &mut Marker, top: bool) -> Vec<Node> {
    emit_from(row, 0, path, m, top)
}

fn emit_from(row: &[Atom], from: usize, path: &mut Vec<(usize, u8)>, m: &mut Marker, top: bool) -> Vec<Node> {
    let mut out = Vec::new();
    if row.len() <= from {
        let id = m.mark(path, from);
        if top && !m.touches(path, from, from) {
            out.extend(id.map(Node::Mark));
        } else {
            out.push(Node::Slot(id));
        }
        return out;
    }
    let all = units(row);
    let us: Vec<Unit> = all.into_iter().filter(|u| u.start() >= from).collect();
    for (k, &u) in us.iter().enumerate() {
        let prev = k.checked_sub(1).map(|j| us[j]);
        let next = us.get(k + 1).copied();
        let start = u.start();
        // The place before an exponent is at its base's end (`attach_sup`).
        if !matches!(row[start], Atom::Sup(_)) {
            out.extend(m.mark(path, start).map(Node::Mark));
        }
        // An implied product that needs its dot.
        let raw = |u: Unit| matches!(u, Unit::Tok(t) if m.touches(path, t.start, t.end));
        let dot = |l: Unit, r: Unit| needs_dot(look(row, l, true, raw(l)), look(row, r, false, raw(r)));
        if prev.is_some_and(|p| is_factor_end(p) && is_factor_start(row, u) && dot(p, u)) {
            out.push(Node::bin('\u{22C5}'));
        }
        match u {
            Unit::Tok(t) => {
                let text = token_text(row, t);
                let as_typed = m.touches(path, t.start, t.end);
                let mut ns = match t.kind {
                    Kind::Sign => vec![Node::bin(if text == "-" { '\u{2212}' } else { '+' })],
                    Kind::Name if as_typed => vec![Node::var(&text)],
                    Kind::Number if as_typed => vec![Node::num(&text)],
                    Kind::Name => vec![word(&text)],
                    Kind::Number => number(&text),
                };
                // A plain run, a glyph a character: places between its characters.
                if let [Node::Glyphs { text: shown, anchors, .. }] = &mut ns[..] {
                    if shown.chars().count() == t.end - t.start {
                        for p in t.start + 1..t.end {
                            anchors.extend(m.mark(path, p).map(|id| (p - t.start, id)));
                        }
                    }
                }
                // 1×10⁻⁵ raised is (1×10⁻⁵)², as the text's one number is.
                let raised = next.is_some_and(|n| matches!(row[n.start()], Atom::Sup(_)));
                if raised && ns.len() > 1 {
                    out.push(paren(ns));
                } else {
                    out.extend(ns);
                }
            }
            Unit::Op(_, Op::Plus) => out.push(Node::bin('+')),
            Unit::Op(_, Op::Minus) => out.push(Node::bin('\u{2212}')),
            Unit::Op(_, Op::Eq) => out.push(Node::rel('=')),
            Unit::Op(_, Op::Times) => {
                // Written, it shows where the product needs a dot — or has an operand missing.
                let (l, r) = (prev.filter(|&p| is_factor_end(p)), next.filter(|&n| is_factor_start(row, n)));
                if l.zip(r).is_none_or(|(l, r)| dot(l, r)) {
                    out.push(Node::bin('\u{22C5}'));
                }
            }
            Unit::At(i) => {
                let child = |k: u8, path: &mut Vec<(usize, u8)>, m: &mut Marker, from: usize| {
                    let r = row[i].child(k).map_or(&[][..], |r| &r[..]);
                    path.push((i, k));
                    let ns = emit_from(r, from, path, m, false);
                    path.pop();
                    ns
                };
                match &row[i] {
                    Atom::Sup(_) => {
                        let mark = m.mark(path, i);
                        let sup = child(0, path, m, 0);
                        if prev.is_some_and(is_factor_end) {
                            attach_sup(&mut out, sup, mark);
                        } else {
                            out.extend(mark.map(Node::Mark));
                            out.push(Node::Scripts { base: Box::new(Node::Slot(None)), sup: Some(sup), sub: None });
                        }
                    }
                    Atom::Frac { .. } => {
                        let num = child(0, path, m, 0);
                        let den = child(1, path, m, 0);
                        out.push(Node::Frac { num, den });
                    }
                    Atom::Group(_) => out.push(paren(child(0, path, m, 0))),
                    Atom::Func { name, .. } => {
                        let body = child(0, path, m, 0);
                        out.extend(func(name, body));
                    }
                    Atom::Bars(_) => out.push(Node::Scripts {
                        base: Box::new(Node::Fenced { open: '|', close: '|', body: child(0, path, m, 0) }),
                        sup: Some(vec![Node::num("2")]),
                        sub: None,
                    }),
                    Atom::Complex { im, .. } => {
                        let mut body = child(0, path, m, 0);
                        if m.inside(path, i) {
                            // Being edited: as written, its two parts and the comma.
                            body.push(Node::glyphs(",", Class::Punct));
                            body.extend(child(1, path, m, 0));
                        } else {
                            let negative = matches!(im.first(), Some(Atom::Op(Op::Minus)));
                            if negative {
                                path.push((i, 1));
                                body.extend(m.mark(path, 0).map(Node::Mark));
                                path.pop();
                            }
                            body.push(Node::bin(if negative { '\u{2212}' } else { '+' }));
                            body.extend(child(1, path, m, usize::from(negative)));
                            body.push(Node::var("i"));
                        }
                        out.push(paren(body));
                    }
                    Atom::Char(_) | Atom::Op(_) => {}
                }
            }
        }
    }
    out.extend(m.mark(path, row.len()).map(Node::Mark));
    out
}

#[cfg(test)]
#[path = "model_tests.rs"]
pub(super) mod tests;
