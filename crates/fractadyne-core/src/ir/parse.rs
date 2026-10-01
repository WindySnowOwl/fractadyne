//! The expression front end: a formula's step written as Fractint-style statements
//! (design/custom-formulas.md §4.2 item 2 — the expression core of the `.frm` reader, brought
//! forward so a custom formula can be typed and saved).
//!
//! ```text
//! z = z^3 - p1*z + c          one statement: the new z
//! t = sqr(z), z = t*t + c     temporaries; statements separated by ',' or a new line
//! z^2 + c                     a bare expression is the new z
//! ```
//!
//! Semantics follow Fractint where it has them: names are case-insensitive; `;` starts a comment;
//! statements run in order, so `z` read after `z = …` is the new value; `|x|` is the SQUARED modulus
//! (Fractint's `|z|`), `cabs(x)` the modulus; `abs` takes both parts' absolute values; `(re, im)` is a
//! complex constant; `p1`…`p5` are parameters; `pixel` is a synonym of `c`. `^` binds tighter than
//! unary minus (`-z^2` is `-(z^2)`) and associates to the right.
//!
//! Not yet (the rest of design phase 3): `init:`/`bailout:` sections, `if`, comparisons, `fn1`…`fn4`.
//! They are reported as errors, never mis-read.

use super::syntax::{BinOp, Comment, Expr, ExprKind, Name, Span, Statement, Syntax};
use super::{Builder, Formula, Func, Op, Val};

/// Where and why a source failed to parse. `line` and `col` are 1-based, `col` in characters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}: {}", self.line, self.col, self.message)
    }
}

impl std::error::Error for ParseError {}

/// The number of parameters a source can name (`p1`…`p5`, as in Fractint).
pub const MAX_PARAMS: usize = 5;

/// Parse a step. The result has one phase; its parameters are `p1`…`p5` in order.
pub fn parse(src: &str) -> Result<Formula, ParseError> {
    run(src, true).map(|(f, _)| f.expect("a lowering run returns its formula"))
}

/// Read a source into its syntax tree, without evaluating it: what the formula editor's textbook
/// mode typesets. It is the same pass as [`parse`] with the IR's own checks left out — a name used
/// before it is assigned, a complex constant with non-constant parts, a formula with no step — so a
/// formula still being typed still has a tree. A source `parse` accepts always has one, and the
/// grammar's errors are the same errors in the same places.
pub fn syntax(src: &str) -> Result<Syntax, ParseError> {
    run(src, false).map(|(_, s)| s)
}

/// One pass over `src`: the syntax tree always, the IR as well when `lower`. ⚠The IR is built by
/// exactly the calls, in exactly the order, of the parser before the tree existed (the snapshot in
/// `parse/snapshot.rs` holds it to that), and with `lower` every error comes where it came before.
fn run(src: &str, lower: bool) -> Result<(Option<Formula>, Syntax), ParseError> {
    let (tokens, comments) = lex(src)?;
    let mut p = Parser { src, tokens, at: 0, b: Builder::new(), vars: Vec::new(), consts: Vec::new(), lower };
    let z = p.leaf(Op::Z);
    p.vars.push(("z".to_string(), z));
    let mut assigned_z = false;
    let mut last_bare = None;
    let mut statements = Vec::new();
    loop {
        while p.eat(&Tok::Sep) {}
        if p.peek() == &Tok::End {
            break;
        }
        let stmt_at = p.at;
        if let (Tok::Ident(name), Tok::Assign) = (p.peek().clone(), p.peek_at(1).clone()) {
            p.at += 2;
            if reserved(&name) {
                return Err(p.err_at(stmt_at, format!("`{name}` cannot be assigned")));
            }
            let (v, body) = p.expr()?;
            if name == "z" {
                assigned_z = true;
            }
            let target = Name { name: name.clone(), span: p.token_span(stmt_at) };
            match p.vars.iter_mut().find(|(n, _)| *n == name) {
                Some(slot) => slot.1 = v,
                None => p.vars.push((name, v)),
            }
            last_bare = None;
            statements.push(Statement { span: p.span_from(stmt_at), target: Some(target), body });
        } else {
            let (v, body) = p.expr()?;
            last_bare = Some(v);
            statements.push(Statement { span: body.span, target: None, body });
        }
        match p.peek() {
            Tok::Sep | Tok::End => {}
            _ => return Err(p.err(format!("expected ',' or a new line, found {}", p.peek().describe()))),
        }
    }
    let tree = Syntax { statements, comments };
    if !lower {
        return Ok((None, tree));
    }
    let out = match last_bare {
        Some(v) => v,
        None if assigned_z => p.var("z").expect("z is always bound"),
        None => return Err(ParseError { line: 1, col: 1, message: "no step: assign `z` or end with an expression".into() }),
    };
    let prog = p.b.finish(out).map_err(|e| ParseError { line: 1, col: 1, message: e.to_string() })?;
    // Folding leaves the folded parts' instructions behind; drop everything the output does not read.
    Ok((Some(Formula::single(prog.without_dead_code())), tree))
}

/// `1/k` is exact: `k` is a normal power of two.
fn exact_reciprocal(k: f64) -> bool {
    k.is_normal() && k.to_bits() & ((1u64 << 52) - 1) == 0 && (1.0 / k).is_normal()
}

fn reserved(name: &str) -> bool {
    matches!(name, "c" | "pixel" | "pi" | "e") || param_index(name).is_some() || function(name).is_some()
}

fn param_index(name: &str) -> Option<u16> {
    let n: u16 = name.strip_prefix('p')?.parse().ok()?;
    (1..=MAX_PARAMS as u16).contains(&n).then_some(n - 1)
}

/// A named function: an elementary one, or one lowered to other operations.
#[derive(Clone, Copy)]
enum Named {
    Func(Func),
    Sqr,
    Abs,
    Conj,
    Real,
    Imag,
    Cabs,
    Flip,
    Recip,
    Ident,
    Cotan,
    Cotanh,
}

fn function(name: &str) -> Option<Named> {
    Some(match name {
        "exp" => Named::Func(Func::Exp),
        "log" => Named::Func(Func::Log),
        "sqrt" => Named::Func(Func::Sqrt),
        "sin" => Named::Func(Func::Sin),
        "cos" => Named::Func(Func::Cos),
        "tan" => Named::Func(Func::Tan),
        "sinh" => Named::Func(Func::Sinh),
        "cosh" => Named::Func(Func::Cosh),
        "tanh" => Named::Func(Func::Tanh),
        "sqr" => Named::Sqr,
        "abs" => Named::Abs,
        "conj" => Named::Conj,
        "real" => Named::Real,
        "imag" => Named::Imag,
        "cabs" => Named::Cabs,
        "flip" => Named::Flip,
        "recip" => Named::Recip,
        "ident" => Named::Ident,
        "cotan" => Named::Cotan,
        "cotanh" => Named::Cotanh,
        _ => return None,
    })
}

/// Whether `name` (in any case, as the lexer reads names) is one of the language's functions.
pub fn is_function(name: &str) -> bool {
    function(&name.to_ascii_lowercase()).is_some()
}

/// Fractint features outside this subset, named so the error says what is missing.
fn unsupported(name: &str) -> Option<&'static str> {
    Some(match name {
        "fn1" | "fn2" | "fn3" | "fn4" => "the fn1…fn4 function slots are not supported yet",
        "if" | "elseif" | "else" | "endif" => "`if` blocks are not supported yet",
        "whitesq" | "scrnpix" | "scrnmax" | "maxit" | "ismand" | "center" | "magxmag" | "rotskew" => {
            "screen and view variables are not supported"
        }
        "rand" | "srand" => "random numbers are not supported",
        "lastsqr" => "`lastsqr` is not supported",
        "cosxx" | "asin" | "acos" | "atan" | "asinh" | "acosh" | "atanh" | "floor" | "ceil" | "trunc" | "round" => {
            "this function is not supported yet"
        }
        _ => return None,
    })
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
    Comma,
    Bar,
    Assign,
    /// A comparison or logical operator: recognised only to be refused.
    Unsupported(&'static str),
    /// A statement separator: a new line (a `,` outside parentheses becomes one too).
    Sep,
    End,
}

impl Tok {
    fn describe(&self) -> String {
        match self {
            Tok::Num(v) => format!("the number {v}"),
            Tok::Ident(s) => format!("`{s}`"),
            Tok::Plus => "'+'".into(),
            Tok::Minus => "'-'".into(),
            Tok::Star => "'*'".into(),
            Tok::Slash => "'/'".into(),
            Tok::Caret => "'^'".into(),
            Tok::LParen => "'('".into(),
            Tok::RParen => "')'".into(),
            Tok::Comma => "','".into(),
            Tok::Bar => "'|'".into(),
            Tok::Assign => "'='".into(),
            Tok::Unsupported(s) => format!("'{s}'"),
            Tok::Sep => "the end of the statement".into(),
            Tok::End => "the end of the formula".into(),
        }
    }
}

/// Tokens with their byte ranges, and the comments. A ',' at parenthesis depth 0 is a statement
/// separator; inside parentheses it separates a complex constant's parts.
#[allow(clippy::type_complexity)]
fn lex(src: &str) -> Result<(Vec<(Tok, usize, usize)>, Vec<Comment>), ParseError> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    let mut comments = Vec::new();
    let (mut i, mut depth) = (0usize, 0i32);
    let err = |at: usize, message: String| {
        let (line, col) = line_col(src, at);
        ParseError { line, col, message }
    };
    while i < bytes.len() {
        let ch = bytes[i];
        let start = i;
        let tok = match ch {
            b' ' | b'\t' | b'\r' => {
                i += 1;
                continue;
            }
            b'\n' => Tok::Sep,
            b';' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                let text = src[start + 1..i].trim_end_matches('\r').to_string();
                comments.push(Comment { text, span: Span { start, end: i } });
                continue;
            }
            b'0'..=b'9' | b'.' => {
                let mut j = i;
                while j < bytes.len() && (bytes[j].is_ascii_digit() || bytes[j] == b'.') {
                    j += 1;
                }
                if j < bytes.len() && (bytes[j] == b'e' || bytes[j] == b'E') {
                    let mut k = j + 1;
                    if k < bytes.len() && (bytes[k] == b'+' || bytes[k] == b'-') {
                        k += 1;
                    }
                    if k < bytes.len() && bytes[k].is_ascii_digit() {
                        while k < bytes.len() && bytes[k].is_ascii_digit() {
                            k += 1;
                        }
                        j = k;
                    }
                }
                let text = &src[i..j];
                let v: f64 = text.parse().map_err(|_| err(i, format!("`{text}` is not a number")))?;
                i = j;
                out.push((Tok::Num(v), start, i));
                continue;
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let mut j = i;
                while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                    j += 1;
                }
                let name = src[i..j].to_ascii_lowercase();
                i = j;
                out.push((Tok::Ident(name), start, i));
                continue;
            }
            b'+' => Tok::Plus,
            b'-' => Tok::Minus,
            b'*' => Tok::Star,
            b'/' => Tok::Slash,
            b'^' => Tok::Caret,
            b'(' => {
                depth += 1;
                Tok::LParen
            }
            b')' => {
                depth -= 1;
                Tok::RParen
            }
            b',' if depth <= 0 => Tok::Sep,
            b',' => Tok::Comma,
            b'|' if bytes.get(i + 1) == Some(&b'|') => {
                i += 1;
                Tok::Unsupported("||")
            }
            b'|' => Tok::Bar,
            b'=' if bytes.get(i + 1) == Some(&b'=') => {
                i += 1;
                Tok::Unsupported("==")
            }
            b'=' => Tok::Assign,
            b'<' | b'>' | b'!' | b'&' => {
                if bytes.get(i + 1) == Some(&b'=') || bytes.get(i + 1) == Some(&b'&') {
                    i += 1;
                }
                Tok::Unsupported("comparison")
            }
            b':' | b'{' | b'}' => {
                return Err(err(i, "sections (`init:`, `{ }`) are not supported yet — write the loop body only".into()))
            }
            _ => {
                let c = src[i..].chars().next().unwrap_or('?');
                return Err(err(i, format!("unexpected character '{c}'")));
            }
        };
        i += 1;
        out.push((tok, start, i));
    }
    out.push((Tok::End, src.len(), src.len()));
    Ok((out, comments))
}

fn line_col(src: &str, at: usize) -> (usize, usize) {
    let before = &src[..at.min(src.len())];
    let line = before.matches('\n').count() + 1;
    let col = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
    (line, col)
}

struct Parser<'a> {
    src: &'a str,
    /// Tokens with their byte ranges.
    tokens: Vec<(Tok, usize, usize)>,
    at: usize,
    b: Builder,
    vars: Vec<(String, Val)>,
    /// Values known to be constants, for folding `(re, im)` parts, `-2` and `2*3`.
    consts: Vec<(Val, (f64, f64))>,
    /// Building the IR (and so making its checks); without, only the syntax tree is wanted.
    lower: bool,
}

impl Parser<'_> {
    /// Token `i`'s byte range.
    fn token_span(&self, i: usize) -> Span {
        let (_, start, end) = &self.tokens[i.min(self.tokens.len() - 1)];
        Span { start: *start, end: *end }
    }
    /// From token `first` to the last token consumed.
    fn span_from(&self, first: usize) -> Span {
        let start = self.token_span(first).start;
        let end = if self.at > first { self.token_span(self.at - 1).end } else { start };
        Span { start, end }
    }
    fn node(&self, first: usize, kind: ExprKind) -> Expr {
        Expr { kind, span: self.span_from(first) }
    }
    fn peek(&self) -> &Tok {
        &self.tokens[self.at].0
    }
    fn peek_at(&self, k: usize) -> &Tok {
        &self.tokens[(self.at + k).min(self.tokens.len() - 1)].0
    }
    fn eat(&mut self, t: &Tok) -> bool {
        if self.peek() == t {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn err_at(&self, token: usize, message: String) -> ParseError {
        let (line, col) = line_col(self.src, self.tokens[token.min(self.tokens.len() - 1)].1);
        ParseError { line, col, message }
    }
    fn err(&self, message: String) -> ParseError {
        self.err_at(self.at, message)
    }
    fn expect(&mut self, t: &Tok) -> Result<(), ParseError> {
        if self.eat(t) {
            Ok(())
        } else {
            Err(self.err(format!("expected {}, found {}", t.describe(), self.peek().describe())))
        }
    }

    fn var(&self, name: &str) -> Option<Val> {
        self.vars.iter().find(|(n, _)| n == name).map(|(_, v)| *v)
    }
    /// `Z` and `C` once each, however often they are read.
    fn leaf(&mut self, op: Op) -> Val {
        let found = self.b.insts.iter().position(|o| *o == op);
        match found {
            Some(i) if matches!(op, Op::Z | Op::C | Op::ZPrev | Op::Param(_)) => Val(i as u32),
            _ => self.b.push(op),
        }
    }
    fn konst(&mut self, re: f64, im: f64) -> Val {
        let v = self.b.push(Op::Const(re, im));
        self.consts.push((v, (re, im)));
        v
    }
    fn const_of(&self, v: Val) -> Option<(f64, f64)> {
        self.consts.iter().find(|(c, _)| *c == v).map(|(_, k)| *k)
    }

    fn expr(&mut self) -> Result<(Val, Expr), ParseError> {
        let first = self.at;
        let (mut acc, mut tree) = self.term()?;
        loop {
            let add = if self.eat(&Tok::Plus) {
                true
            } else if self.eat(&Tok::Minus) {
                false
            } else {
                break;
            };
            let (rhs, rhs_tree) = self.term()?;
            acc = match (self.const_of(acc), self.const_of(rhs)) {
                (Some(a), Some(b)) if add => self.konst(a.0 + b.0, a.1 + b.1),
                (Some(a), Some(b)) => self.konst(a.0 - b.0, a.1 - b.1),
                _ if add => self.b.push(Op::Add(acc, rhs)),
                _ => self.b.push(Op::Sub(acc, rhs)),
            };
            let op = if add { BinOp::Add } else { BinOp::Sub };
            tree = self.node(first, ExprKind::Bin(op, Box::new(tree), Box::new(rhs_tree)));
        }
        Ok((acc, tree))
    }

    fn term(&mut self) -> Result<(Val, Expr), ParseError> {
        let first = self.at;
        let (mut acc, mut tree) = self.unary()?;
        loop {
            let mul = if self.eat(&Tok::Star) {
                true
            } else if self.eat(&Tok::Slash) {
                false
            } else {
                break;
            };
            let (rhs, rhs_tree) = self.unary()?;
            let op = if mul { BinOp::Mul } else { BinOp::Div };
            tree = self.node(first, ExprKind::Bin(op, Box::new(tree), Box::new(rhs_tree)));
            let (ka, kb) = (self.const_of(acc), self.const_of(rhs));
            let real = |k: Option<(f64, f64)>| k.filter(|k| k.1 == 0.0).map(|k| k.0);
            acc = if let (Some(a), Some(b)) = (ka, kb) {
                if mul {
                    self.konst(a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
                } else {
                    let d = b.0 * b.0 + b.1 * b.1;
                    self.konst((a.0 * b.0 + a.1 * b.1) / d, (a.1 * b.0 - a.0 * b.1) / d)
                }
            } else if let (true, Some(k)) = (mul, real(ka)) {
                // A real factor scales both parts: cheaper than a complex product, and the same
                // value (the product's cross terms are exact zeros).
                self.b.push(Op::Scale(rhs, k))
            } else if let (true, Some(k)) = (mul, real(kb)) {
                self.b.push(Op::Scale(acc, k))
            } else if let (false, Some(k)) = (mul, real(kb).filter(|k| exact_reciprocal(*k))) {
                // Dividing by a power of two is multiplying by its exact reciprocal.
                self.b.push(Op::Scale(acc, 1.0 / k))
            } else if mul {
                self.b.push(Op::Mul(acc, rhs))
            } else {
                self.b.push(Op::Div(acc, rhs))
            };
        }
        Ok((acc, tree))
    }

    fn unary(&mut self) -> Result<(Val, Expr), ParseError> {
        let first = self.at;
        if self.eat(&Tok::Minus) {
            let (v, inner) = self.unary()?;
            let v = match self.const_of(v) {
                Some((re, im)) => self.konst(-re, -im),
                None => self.b.push(Op::Neg(v)),
            };
            return Ok((v, self.node(first, ExprKind::Neg(Box::new(inner)))));
        }
        if self.eat(&Tok::Plus) {
            let (v, inner) = self.unary()?;
            return Ok((v, self.node(first, ExprKind::Pos(Box::new(inner)))));
        }
        self.power()
    }

    fn power(&mut self) -> Result<(Val, Expr), ParseError> {
        let first = self.at;
        let (base, base_tree) = self.atom()?;
        if !self.eat(&Tok::Caret) {
            return Ok((base, base_tree));
        }
        let (exp, exp_tree) = self.unary()?; // right-associative: z^2^3 = z^(2^3); z^-1 allowed
        let tree = self.node(first, ExprKind::Pow(Box::new(base_tree), Box::new(exp_tree)));
        Ok((self.power_value(base, exp), tree))
    }

    /// `base ^ exp`: an integer constant exponent as repeated products, constants folded.
    fn power_value(&mut self, base: Val, exp: Val) -> Val {
        let k = match self.const_of(exp) {
            Some((k, im)) if im == 0.0 && k.fract() == 0.0 && k.abs() <= 64.0 => k,
            _ => return self.b.push(Op::Pow(base, exp)),
        };
        let n = k.abs() as u32;
        if n == 0 {
            return self.konst(1.0, 0.0);
        }
        if let Some((re, im)) = self.const_of(base) {
            let (mut r, b) = ((1.0, 0.0), (re, im));
            for _ in 0..n {
                r = (r.0 * b.0 - r.1 * b.1, r.0 * b.1 + r.1 * b.0);
            }
            if k < 0.0 {
                let d = r.0 * r.0 + r.1 * r.1;
                r = (r.0 / d, -r.1 / d);
            }
            return self.konst(r.0, r.1);
        }
        let p = if n == 1 { base } else { self.b.push(Op::PowI(base, n)) };
        if k < 0.0 {
            let one = self.konst(1.0, 0.0);
            self.b.push(Op::Div(one, p))
        } else {
            p
        }
    }

    fn atom(&mut self) -> Result<(Val, Expr), ParseError> {
        let tok_at = self.at;
        match self.peek().clone() {
            Tok::Num(v) => {
                self.at += 1;
                Ok((self.konst(v, 0.0), self.node(tok_at, ExprKind::Num(v))))
            }
            Tok::LParen => {
                self.at += 1;
                let (first, first_tree) = self.expr()?;
                if self.eat(&Tok::Comma) {
                    let (second, second_tree) = self.expr()?;
                    let real = |k: Option<(f64, f64)>| k.filter(|k| k.1 == 0.0).map(|k| k.0);
                    let parts = match (real(self.const_of(first)), real(self.const_of(second))) {
                        (Some(re), Some(im)) => (re, im),
                        _ if self.lower => {
                            return Err(self.err_at(tok_at, "a complex constant `(re, im)` takes two real numbers".into()))
                        }
                        // Only the tree is wanted: a formula still being typed has one.
                        _ => (0.0, 0.0),
                    };
                    self.expect(&Tok::RParen)?;
                    let tree = self.node(tok_at, ExprKind::Complex(Box::new(first_tree), Box::new(second_tree)));
                    return Ok((self.konst(parts.0, parts.1), tree));
                }
                self.expect(&Tok::RParen)?;
                Ok((first, self.node(tok_at, ExprKind::Group(Box::new(first_tree)))))
            }
            Tok::Bar => {
                self.at += 1;
                let (v, inner) = self.expr()?;
                self.expect(&Tok::Bar)?;
                Ok((self.b.push(Op::Norm(v)), self.node(tok_at, ExprKind::Bars(Box::new(inner)))))
            }
            Tok::Ident(name) => {
                self.at += 1;
                if let Some(msg) = unsupported(&name) {
                    return Err(self.err_at(tok_at, format!("`{name}`: {msg}")));
                }
                if let Some(f) = function(&name) {
                    self.expect(&Tok::LParen)?;
                    let (a, arg) = self.expr()?;
                    self.expect(&Tok::RParen)?;
                    let tree = self.node(tok_at, ExprKind::Call { func: name, arg: Box::new(arg) });
                    return Ok((self.apply(f, a), tree));
                }
                if self.peek() == &Tok::LParen {
                    return Err(self.err_at(tok_at, format!("unknown function `{name}`")));
                }
                let tree = self.node(tok_at, ExprKind::Name(name.clone()));
                match name.as_str() {
                    "c" | "pixel" => return Ok((self.leaf(Op::C), tree)),
                    "pi" => return Ok((self.konst(std::f64::consts::PI, 0.0), tree)),
                    "e" => return Ok((self.konst(std::f64::consts::E, 0.0), tree)),
                    _ => {}
                }
                if let Some(i) = param_index(&name) {
                    return Ok((self.leaf(Op::Param(i)), tree));
                }
                match self.var(&name) {
                    Some(v) => Ok((v, tree)),
                    None if self.lower => Err(self.err_at(tok_at, format!("`{name}` is used before it is assigned"))),
                    // Only the tree is wanted: stand in with `z`, which is always bound.
                    None => Ok((self.leaf(Op::Z), tree)),
                }
            }
            Tok::Unsupported(what) => Err(self.err(format!("{what} operators are not supported yet (no `if` or bailout tests)"))),
            other => Err(self.err(format!("expected a value, found {}", other.describe()))),
        }
    }

    fn apply(&mut self, f: Named, a: Val) -> Val {
        match f {
            Named::Func(func) => self.b.push(Op::Func(func, a)),
            Named::Sqr => self.b.push(Op::Sqr(a)),
            Named::Abs => {
                let r = self.b.push(Op::AbsRe(a));
                self.b.push(Op::AbsIm(r))
            }
            Named::Conj => self.b.push(Op::Conj(a)),
            Named::Real => self.b.push(Op::Re(a)),
            Named::Imag => self.b.push(Op::Im(a)),
            Named::Cabs => {
                let n = self.b.push(Op::Norm(a));
                self.b.push(Op::Func(Func::Sqrt, n))
            }
            // flip(x + iy) = y + ix = i·conj(z)
            Named::Flip => {
                let cj = self.b.push(Op::Conj(a));
                let i = self.konst(0.0, 1.0);
                self.b.push(Op::Mul(i, cj))
            }
            Named::Recip => {
                let one = self.konst(1.0, 0.0);
                self.b.push(Op::Div(one, a))
            }
            Named::Ident => a,
            Named::Cotan => {
                let c = self.b.push(Op::Func(Func::Cos, a));
                let s = self.b.push(Op::Func(Func::Sin, a));
                self.b.push(Op::Div(c, s))
            }
            Named::Cotanh => {
                let c = self.b.push(Op::Func(Func::Cosh, a));
                let s = self.b.push(Op::Func(Func::Sinh, a));
                self.b.push(Op::Div(c, s))
            }
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod snapshot;

#[cfg(test)]
mod syntax_tests;
