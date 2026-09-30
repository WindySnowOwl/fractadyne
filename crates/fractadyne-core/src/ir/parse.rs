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
    let tokens = lex(src)?;
    let mut p = Parser { src, tokens, at: 0, b: Builder::new(), vars: Vec::new(), consts: Vec::new() };
    let z = p.leaf(Op::Z);
    p.vars.push(("z".to_string(), z));
    let mut assigned_z = false;
    let mut last_bare = None;
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
            let v = p.expr()?;
            if name == "z" {
                assigned_z = true;
            }
            match p.vars.iter_mut().find(|(n, _)| *n == name) {
                Some(slot) => slot.1 = v,
                None => p.vars.push((name, v)),
            }
            last_bare = None;
        } else {
            let v = p.expr()?;
            last_bare = Some(v);
        }
        match p.peek() {
            Tok::Sep | Tok::End => {}
            _ => return Err(p.err(format!("expected ',' or a new line, found {}", p.peek().describe()))),
        }
    }
    let out = match last_bare {
        Some(v) => v,
        None if assigned_z => p.var("z").expect("z is always bound"),
        None => return Err(ParseError { line: 1, col: 1, message: "no step: assign `z` or end with an expression".into() }),
    };
    let prog = p.b.finish(out).map_err(|e| ParseError { line: 1, col: 1, message: e.to_string() })?;
    // Folding leaves the folded parts' instructions behind; drop everything the output does not read.
    Ok(Formula::single(prog.without_dead_code()))
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

/// Tokens with their byte offsets. A ',' at parenthesis depth 0 is a statement separator; inside
/// parentheses it separates a complex constant's parts.
fn lex(src: &str) -> Result<Vec<(Tok, usize)>, ParseError> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
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
                out.push((Tok::Num(v), start));
                continue;
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let mut j = i;
                while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                    j += 1;
                }
                let name = src[i..j].to_ascii_lowercase();
                i = j;
                out.push((Tok::Ident(name), start));
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
        out.push((tok, start));
    }
    out.push((Tok::End, src.len()));
    Ok(out)
}

fn line_col(src: &str, at: usize) -> (usize, usize) {
    let before = &src[..at.min(src.len())];
    let line = before.matches('\n').count() + 1;
    let col = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
    (line, col)
}

struct Parser<'a> {
    src: &'a str,
    tokens: Vec<(Tok, usize)>,
    at: usize,
    b: Builder,
    vars: Vec<(String, Val)>,
    /// Values known to be constants, for folding `(re, im)` parts, `-2` and `2*3`.
    consts: Vec<(Val, (f64, f64))>,
}

impl Parser<'_> {
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

    fn expr(&mut self) -> Result<Val, ParseError> {
        let mut acc = self.term()?;
        loop {
            let add = if self.eat(&Tok::Plus) {
                true
            } else if self.eat(&Tok::Minus) {
                false
            } else {
                break;
            };
            let rhs = self.term()?;
            acc = match (self.const_of(acc), self.const_of(rhs)) {
                (Some(a), Some(b)) if add => self.konst(a.0 + b.0, a.1 + b.1),
                (Some(a), Some(b)) => self.konst(a.0 - b.0, a.1 - b.1),
                _ if add => self.b.push(Op::Add(acc, rhs)),
                _ => self.b.push(Op::Sub(acc, rhs)),
            };
        }
        Ok(acc)
    }

    fn term(&mut self) -> Result<Val, ParseError> {
        let mut acc = self.unary()?;
        loop {
            let mul = if self.eat(&Tok::Star) {
                true
            } else if self.eat(&Tok::Slash) {
                false
            } else {
                break;
            };
            let rhs = self.unary()?;
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
        Ok(acc)
    }

    fn unary(&mut self) -> Result<Val, ParseError> {
        if self.eat(&Tok::Minus) {
            let v = self.unary()?;
            return Ok(match self.const_of(v) {
                Some((re, im)) => self.konst(-re, -im),
                None => self.b.push(Op::Neg(v)),
            });
        }
        if self.eat(&Tok::Plus) {
            return self.unary();
        }
        self.power()
    }

    fn power(&mut self) -> Result<Val, ParseError> {
        let base = self.atom()?;
        if !self.eat(&Tok::Caret) {
            return Ok(base);
        }
        let exp = self.unary()?; // right-associative: z^2^3 = z^(2^3); z^-1 allowed
        let k = match self.const_of(exp) {
            Some((k, im)) if im == 0.0 && k.fract() == 0.0 && k.abs() <= 64.0 => k,
            _ => return Ok(self.b.push(Op::Pow(base, exp))),
        };
        let n = k.abs() as u32;
        if n == 0 {
            return Ok(self.konst(1.0, 0.0));
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
            return Ok(self.konst(r.0, r.1));
        }
        let p = if n == 1 { base } else { self.b.push(Op::PowI(base, n)) };
        Ok(if k < 0.0 {
            let one = self.konst(1.0, 0.0);
            self.b.push(Op::Div(one, p))
        } else {
            p
        })
    }

    fn atom(&mut self) -> Result<Val, ParseError> {
        let tok_at = self.at;
        match self.peek().clone() {
            Tok::Num(v) => {
                self.at += 1;
                Ok(self.konst(v, 0.0))
            }
            Tok::LParen => {
                self.at += 1;
                let first = self.expr()?;
                if self.eat(&Tok::Comma) {
                    let second = self.expr()?;
                    let real = |k: Option<(f64, f64)>| k.filter(|k| k.1 == 0.0).map(|k| k.0);
                    let (Some(re), Some(im)) = (real(self.const_of(first)), real(self.const_of(second))) else {
                        return Err(self.err_at(tok_at, "a complex constant `(re, im)` takes two real numbers".into()));
                    };
                    self.expect(&Tok::RParen)?;
                    return Ok(self.konst(re, im));
                }
                self.expect(&Tok::RParen)?;
                Ok(first)
            }
            Tok::Bar => {
                self.at += 1;
                let v = self.expr()?;
                self.expect(&Tok::Bar)?;
                Ok(self.b.push(Op::Norm(v)))
            }
            Tok::Ident(name) => {
                self.at += 1;
                if let Some(msg) = unsupported(&name) {
                    return Err(self.err_at(tok_at, format!("`{name}`: {msg}")));
                }
                if let Some(f) = function(&name) {
                    self.expect(&Tok::LParen)?;
                    let a = self.expr()?;
                    self.expect(&Tok::RParen)?;
                    return Ok(self.apply(f, a));
                }
                if self.peek() == &Tok::LParen {
                    return Err(self.err_at(tok_at, format!("unknown function `{name}`")));
                }
                match name.as_str() {
                    "c" | "pixel" => return Ok(self.leaf(Op::C)),
                    "pi" => return Ok(self.konst(std::f64::consts::PI, 0.0)),
                    "e" => return Ok(self.konst(std::f64::consts::E, 0.0)),
                    _ => {}
                }
                if let Some(i) = param_index(&name) {
                    return Ok(self.leaf(Op::Param(i)));
                }
                self.var(&name).ok_or_else(|| self.err_at(tok_at, format!("`{name}` is used before it is assigned")))
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
