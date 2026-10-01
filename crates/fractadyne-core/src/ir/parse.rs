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
//! Fractint's formula sections too (design phase 3):
//!
//! ```text
//! z = 0.5:                    an init section: run once per pixel, before the first step
//!   z = c*z*(1 - z)           the loop
//!   |z| <= 100                a final COMPARISON is the bailout: iterate while it holds
//! ```
//!
//! Variables persist from one step to the next, as in Fractint: a name read before its statement
//! has run in this step is its value from the step before (from the init section, or 0) — when a
//! statement assigns it somewhere; otherwise it is a mistake, and an error. Comparisons (`< <= > >=
//! == !=`) compare REAL parts and give 1 or 0; `&&` and `||` evaluate both sides; `if (…) … elseif
//! (…) … else … endif` blocks compute both branches and keep one. Not supported, and reported:
//! `fn1`…`fn4` (the `.frm` reader puts Fractint's default functions in their place), the screen
//! variables, random numbers and `lastsqr`.

use super::syntax::{BinOp, Comment, Expr, ExprKind, Logic, Name, Span, Statement, Syntax};
use super::{Builder, Cmp, Formula, Func, Op, Val};

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
    // Every name a statement assigns: one read before its statement has run carries its value over
    // from the step before (Fractint's variables persist), rather than being a mistake.
    let mut assigned: Vec<String> = Vec::new();
    for w in tokens.windows(2) {
        if let (Tok::Ident(n), Tok::Assign) = (&w[0].0, &w[1].0) {
            if !assigned.contains(n) {
                assigned.push(n.clone());
            }
        }
    }
    let has_init = tokens.iter().any(|t| t.0 == Tok::Colon);
    let mut p = Parser {
        src,
        tokens,
        at: 0,
        b: Builder::new(),
        vars: Vec::new(),
        consts: Vec::new(),
        lower,
        assigned,
        state: Vec::new(),
        in_init: has_init,
    };
    let ir_err = |e: super::IrError| ParseError { line: 1, col: 1, message: e.to_string() };
    let mut statements = Vec::new();
    // The init section: a program of its own, run once from z = z₀.
    let mut init = None;
    if has_init {
        let z = p.leaf(Op::Z);
        p.vars.push(("z".to_string(), z));
        p.statements(&mut statements)?;
        p.expect(&Tok::Colon)?;
        let out = p.var("z").expect("z is always bound");
        init = Some((std::mem::take(&mut p.b), out, std::mem::take(&mut p.vars)));
        p.consts.clear();
        p.in_init = false;
    }
    let z = p.leaf(Op::Z);
    p.vars.push(("z".to_string(), z));
    let block = p.statements(&mut statements)?;
    if p.peek() == &Tok::Colon {
        return Err(p.err("a second ':' — the init section ends at the first".into()));
    }
    if p.peek() != &Tok::End {
        return Err(p.err(format!("{} without an `if`", p.peek().describe())));
    }
    let tree = Syntax { statements, comments };
    if !lower {
        return Ok((None, tree));
    }
    // A final comparison is the bailout (Fractint's last loop statement); any other final bare
    // expression is the new z. With a bailout, a loop that leaves z alone is a step too (the test
    // alone decides, on the pixel or on the other variables).
    let cond = block.last_bare.filter(|v| matches!(p.b.insts[v.index()], Op::Cmp(..) | Op::And(..) | Op::Or(..)));
    let out = match block.last_bare {
        Some(v) if cond.is_none() => v,
        _ if block.assigned_z || cond.is_some() => p.var("z").expect("z is always bound"),
        _ => return Err(ParseError { line: 1, col: 1, message: "no step: assign `z` or end with an expression".into() }),
    };
    // Each carried variable the loop sets: its value at the end of the step.
    let sets = |vals: &[(String, Val)], state: &[String]| -> Vec<(u16, Val)> {
        state
            .iter()
            .enumerate()
            .filter_map(|(i, name)| vals.iter().find(|(n, _)| n == name).map(|(_, v)| (i as u16, *v)))
            .collect()
    };
    let mut prog = std::mem::take(&mut p.b).finish(out).map_err(ir_err)?;
    prog = prog.with_vars(sets(&p.vars, &p.state)).map_err(ir_err)?;
    if let Some(c) = cond {
        prog = prog.with_cond(c).map_err(ir_err)?;
    }
    // Folding leaves the folded parts' instructions behind; drop everything the outputs do not read.
    let formula = Formula::single(prog.without_dead_code());
    if !has_init && p.state.is_empty() && cond.is_none() {
        return Ok((Some(formula), tree));
    }
    // A predefined name the loop assigns, and reads before it does, starts at its predefined value:
    // the init section (one that leaves z alone, if there is none) gives it.
    let starts: Vec<String> = p.state.iter().filter(|n| predefined(n)).cloned().collect();
    let init = match init {
        None if !starts.is_empty() => {
            let mut b = Builder::new();
            let z = b.push(Op::Z);
            Some((b, z, Vec::new()))
        }
        other => other,
    };
    let init = match init {
        Some((mut b, out, mut vals)) => {
            std::mem::swap(&mut p.b, &mut b);
            for name in &starts {
                if !vals.iter().any(|(n, _)| n == name) {
                    let v = p.predefined_value(name).expect("a predefined name");
                    vals.push((name.clone(), v));
                }
            }
            std::mem::swap(&mut p.b, &mut b);
            let prog = b.finish(out).map_err(ir_err)?.with_vars(sets(&vals, &p.state)).map_err(ir_err)?;
            Some(prog.without_dead_code())
        }
        None => None,
    };
    let n = u16::try_from(p.state.len()).map_err(|_| ParseError { line: 1, col: 1, message: "too many variables".into() })?;
    Ok((Some(formula.with_sections(n, init).map_err(ir_err)?), tree))
}

/// What a run of statements left: its last bare expression (unless something followed it), and
/// whether it assigned `z`.
#[derive(Default)]
struct Block {
    last_bare: Option<Val>,
    assigned_z: bool,
}

/// The block keywords, which end a run of statements.
fn ends_block(t: &Tok) -> bool {
    matches!(t, Tok::End | Tok::Colon) || matches!(t, Tok::Ident(k) if matches!(k.as_str(), "elseif" | "else" | "endif"))
}

/// `1/k` is exact: `k` is a normal power of two.
fn exact_reciprocal(k: f64) -> bool {
    k.is_normal() && k.to_bits() & ((1u64 << 52) - 1) == 0 && (1.0 / k).is_normal()
}

/// The end of the error for a name nothing assigns (one place, as [`unassigned_name`] reads it).
const UNASSIGNED: &str = "` is used before it is assigned";

/// The name an error says nothing assigns — the `.frm` reader sets it to 0, as Fractint has it.
pub fn unassigned_name(e: &ParseError) -> Option<&str> {
    e.message.strip_prefix('`')?.strip_suffix(UNASSIGNED)
}

/// Whether a statement may assign `name`. Fractint's predefined variables — `pixel`, `p1`…`p5`,
/// `pi`, `e` — may be: each holds its predefined value until it is. `c`, this language's own name
/// for the pixel, may not (a `.frm` file's `c` is renamed as it is read).
fn assignable(name: &str) -> bool {
    !matches!(name, "c" | "if" | "elseif" | "else" | "endif") && function(name).is_none()
}

/// Whether `name` has a value before anything assigns it (see [`assignable`]): with Fractint's
/// `maxit` (the iteration cap) and `ismand` (1: the formula's own Mandelbrot form, as Fractint
/// draws it until its Julia toggle).
fn predefined(name: &str) -> bool {
    matches!(name, "c" | "pixel" | "pi" | "e" | "maxit" | "ismand") || param_index(name).is_some()
}

fn param_index(name: &str) -> Option<u16> {
    let n: u16 = name.strip_prefix('p')?.parse().ok()?;
    // `then`, not `then_some`: `p0` would underflow evaluating the argument.
    (1..=MAX_PARAMS as u16).contains(&n).then(|| n - 1)
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
    /// Fractint's `cosxx`, the conjugate of the cosine (its cos before version 16).
    Cosxx,
    /// The inverse functions, from their logarithm forms (principal values).
    Asin,
    Acos,
    Atan,
    Asinh,
    Acosh,
    Atanh,
    /// `floor`, `ceil`, `trunc`, `round`, part by part.
    Round(super::Round),
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
        "cosxx" => Named::Cosxx,
        "asin" => Named::Asin,
        "acos" => Named::Acos,
        "atan" => Named::Atan,
        "asinh" => Named::Asinh,
        "acosh" => Named::Acosh,
        "atanh" => Named::Atanh,
        "floor" => Named::Round(super::Round::Floor),
        "ceil" => Named::Round(super::Round::Ceil),
        "trunc" => Named::Round(super::Round::Trunc),
        "round" => Named::Round(super::Round::Nearest),
        _ => return None,
    })
}

/// Whether `name` (in any case, as the lexer reads names) is one of the language's functions.
pub fn is_function(name: &str) -> bool {
    function(&name.to_ascii_lowercase()).is_some()
}

/// Functions other notations write differently: (their name, the language's, what it is). `ln` is
/// how ISO 80000-2, LaTeX and most textbooks write the natural logarithm, which the language (as
/// Fractint) calls `log`; cot, coth, Re and Im are how the formula editor's Textbook mode shows
/// cotan, cotanh, real and imag. A call so written does not read: the language keeps one name per
/// function. Its error says what to write instead, and the editor offers to rewrite it. Only CALLS:
/// a variable may still be called `ln`.
pub const OTHER_SPELLINGS: [(&str, &str, &str); 5] = [
    ("ln", "log", "the natural logarithm"),
    ("cot", "cotan", "the cotangent"),
    ("coth", "cotanh", "the hyperbolic cotangent"),
    ("re", "real", "the real part"),
    ("im", "imag", "the imaginary part"),
];

/// The language's name for function `name` as another notation writes it (`ln` → `log`).
pub fn our_spelling(name: &str) -> Option<&'static str> {
    let name = name.to_ascii_lowercase();
    OTHER_SPELLINGS.iter().find(|(other, _, _)| *other == name).map(|&(_, ours, _)| ours)
}

/// Fractint features outside this subset, named so the error says what is missing.
fn unsupported(name: &str) -> Option<&'static str> {
    Some(match name {
        "fn1" | "fn2" | "fn3" | "fn4" => {
            "the fn1…fn4 function slots are not supported — write a function (sin, sqr, …) in their place"
        }
        "whitesq" | "scrnpix" | "scrnmax" | "center" | "magxmag" | "rotskew" => {
            "screen and view variables are not supported"
        }
        "rand" | "srand" => "random numbers are not supported",
        "lastsqr" => "`lastsqr` is not supported",
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
    /// `< <= > >= == !=`.
    Cmp(Cmp),
    AndAnd,
    OrOr,
    /// The end of the init section.
    Colon,
    /// A lone `&` or `!`: recognised only to be refused.
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
            Tok::Cmp(c) => format!("'{}'", c.symbol()),
            Tok::AndAnd => "'&&'".into(),
            Tok::OrOr => "'||'".into(),
            Tok::Colon => "':'".into(),
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
                Tok::OrOr
            }
            b'|' => Tok::Bar,
            b'&' if bytes.get(i + 1) == Some(&b'&') => {
                i += 1;
                Tok::AndAnd
            }
            b'&' => Tok::Unsupported("&"),
            b'<' | b'>' | b'=' | b'!' if bytes.get(i + 1) == Some(&b'=') => {
                let cmp = match ch {
                    b'<' => Cmp::Le,
                    b'>' => Cmp::Ge,
                    b'=' => Cmp::Eq,
                    _ => Cmp::Ne,
                };
                i += 1;
                Tok::Cmp(cmp)
            }
            b'<' => Tok::Cmp(Cmp::Lt),
            b'>' => Tok::Cmp(Cmp::Gt),
            b'!' => Tok::Unsupported("!"),
            b'=' => Tok::Assign,
            b':' => Tok::Colon,
            b'{' | b'}' => {
                return Err(err(
                    i,
                    "braces belong around a formula in a .frm file — here, write its statements only".into(),
                ))
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
    /// Every name a statement assigns (see [`run`]).
    assigned: Vec<String>,
    /// The variables carried from one step to the next, by `Op::Var` index.
    state: Vec<String>,
    /// Reading the init section, where nothing is carried yet.
    in_init: bool,
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
            Some(i) if matches!(op, Op::Z | Op::C | Op::ZPrev | Op::Param(_) | Op::Var(_)) => Val(i as u32),
            _ => self.b.push(op),
        }
    }
    /// `name`'s value carried over from the step before (from the init section, or 0).
    fn carried(&mut self, name: &str) -> Val {
        let i = match self.state.iter().position(|n| n == name) {
            Some(i) => i,
            None => {
                self.state.push(name.to_string());
                self.state.len() - 1
            }
        };
        self.leaf(Op::Var(i as u16))
    }

    /// Statements up to a block keyword, the init section's `:` or the end; each recorded in `tree`.
    fn statements(&mut self, tree: &mut Vec<Statement>) -> Result<Block, ParseError> {
        let mut block = Block::default();
        loop {
            while self.eat(&Tok::Sep) {}
            if ends_block(self.peek()) {
                return Ok(block);
            }
            let stmt_at = self.at;
            // An assignment inside a statement (`a = b = pixel`) may set z too.
            let z_before = self.var("z");
            let assignment = matches!((self.peek(), self.peek_at(1)), (Tok::Ident(_), Tok::Assign));
            if !assignment && matches!(self.peek(), Tok::Ident(k) if k == "if") {
                self.at += 1;
                self.branches(tree)?;
                match self.peek() {
                    Tok::Ident(k) if k == "endif" => self.at += 1,
                    other => {
                        let found = other.describe();
                        return Err(self.err(format!("expected `endif` to close the `if`, found {found}")));
                    }
                }
                block.last_bare = None;
            } else if let (Tok::Ident(name), Tok::Assign) = (self.peek().clone(), self.peek_at(1).clone()) {
                self.at += 2;
                if !assignable(&name) {
                    return Err(self.err_at(stmt_at, format!("`{name}` cannot be assigned")));
                }
                let (v, body) = self.assign_or_expr()?;
                if name == "z" {
                    block.assigned_z = true;
                }
                let target = Name { name: name.clone(), span: self.token_span(stmt_at) };
                self.bind(name, v);
                block.last_bare = None;
                tree.push(Statement { span: self.span_from(stmt_at), target: Some(target), body });
            } else {
                let (v, body) = self.expr()?;
                block.last_bare = Some(v);
                tree.push(Statement { span: body.span, target: None, body });
            }
            block.assigned_z |= self.var("z") != z_before;
            match self.peek() {
                Tok::Sep | Tok::End | Tok::Colon => {}
                Tok::Unsupported(what) => {
                    return Err(self.err(format!("'{what}' is not an operator (Fractint's are && and ||)")))
                }
                _ => return Err(self.err(format!("expected ',' or a new line, found {}", self.peek().describe()))),
            }
        }
    }

    /// After `if` or `elseif`: `(condition)`, its statements, then `elseif …`, `else …` or nothing,
    /// up to (not including) the `endif`. Both branches are computed; each variable either sets
    /// takes the branch's value by the condition — a variable one branch leaves alone keeps its
    /// value from before the block (or from the step before).
    fn branches(&mut self, tree: &mut Vec<Statement>) -> Result<(), ParseError> {
        // The condition opens with a parenthesis, and runs on as an expression: Fractint reads
        // `if (|z| > b) || (t > n)`.
        if self.peek() != &Tok::LParen {
            return Err(self.err(format!("expected '(', found {}", self.peek().describe())));
        }
        let (cond, _) = self.expr()?;
        let before = self.vars.clone();
        self.statements(tree)?;
        let then_vars = std::mem::replace(&mut self.vars, before.clone());
        match self.peek().clone() {
            Tok::Ident(k) if k == "elseif" => {
                self.at += 1;
                self.branches(tree)?;
            }
            Tok::Ident(k) if k == "else" => {
                self.at += 1;
                self.statements(tree)?;
            }
            _ => {}
        }
        let else_vars = std::mem::replace(&mut self.vars, before.clone());
        let mut names: Vec<String> = Vec::new();
        for (n, _) in then_vars.iter().chain(&else_vars) {
            if !names.contains(n) {
                names.push(n.clone());
            }
        }
        let find = |vals: &[(String, Val)], n: &str| vals.iter().find(|(m, _)| m == n).map(|(_, v)| *v);
        for name in names {
            let t = find(&then_vars, &name).or(find(&before, &name));
            let e = find(&else_vars, &name).or(find(&before, &name));
            let t = match t {
                Some(v) => v,
                None => self.carried(&name),
            };
            let e = match e {
                Some(v) => v,
                None => self.carried(&name),
            };
            let v = if t == e { t } else { self.b.push(Op::Select(cond, t, e)) };
            self.bind(name, v);
        }
        Ok(())
    }

    /// `name` now holds `v`.
    fn bind(&mut self, name: String, v: Val) {
        match self.vars.iter_mut().find(|(n, _)| *n == name) {
            Some(slot) => slot.1 = v,
            None => self.vars.push((name, v)),
        }
    }

    /// A predefined name's value (see [`predefined`]).
    fn predefined_value(&mut self, name: &str) -> Option<Val> {
        Some(match name {
            "c" | "pixel" => self.leaf(Op::C),
            "pi" => self.konst(std::f64::consts::PI, 0.0),
            "e" => self.konst(std::f64::consts::E, 0.0),
            "maxit" => self.leaf(Op::MaxIter),
            "ismand" => self.konst(1.0, 0.0),
            _ => self.leaf(Op::Param(param_index(name)?)),
        })
    }
    fn konst(&mut self, re: f64, im: f64) -> Val {
        let v = self.b.push(Op::Const(re, im));
        self.consts.push((v, (re, im)));
        v
    }
    /// `(re, im)`, whose parts are real constants; `at` is the token an error points to.
    fn complex_constant(&mut self, at: usize, re: Val, im: Val) -> Result<Val, ParseError> {
        let real = |k: Option<(f64, f64)>| k.filter(|k| k.1 == 0.0).map(|k| k.0);
        let parts = match (real(self.const_of(re)), real(self.const_of(im))) {
            (Some(re), Some(im)) => (re, im),
            _ if self.lower => return Err(self.err_at(at, "a complex constant `(re, im)` takes two real numbers".into())),
            // Only the tree is wanted: a formula still being typed has one.
            _ => (0.0, 0.0),
        };
        Ok(self.konst(parts.0, parts.1))
    }
    fn const_of(&self, v: Val) -> Option<(f64, f64)> {
        self.consts.iter().find(|(c, _)| *c == v).map(|(_, k)| *k)
    }

    /// An assignment used as a value, or an expression. Fractint's assignment is an expression
    /// too, written where its value is the whole of what is read: chained (`a = b = pixel`) or
    /// opening parentheses (`(z = sin(z))*k`, `if ((d = |w|) < r)`). Elsewhere — `p1^z = 2` — an
    /// `=` stays the error it was.
    fn assign_or_expr(&mut self) -> Result<(Val, Expr), ParseError> {
        let Tok::Ident(name) = self.peek().clone() else { return self.expr() };
        if self.peek_at(1) != &Tok::Assign {
            return self.expr();
        }
        let at = self.at;
        if !assignable(&name) {
            return Err(self.err_at(at, format!("`{name}` cannot be assigned")));
        }
        self.at += 2;
        let (v, body) = self.assign_or_expr()?;
        let target = Name { name: name.clone(), span: self.token_span(at) };
        self.bind(name, v);
        Ok((v, self.node(at, ExprKind::Assign(target, Box::new(body)))))
    }

    /// An expression: `||` binds loosest, then `&&`, then the comparisons, then the arithmetic.
    fn expr(&mut self) -> Result<(Val, Expr), ParseError> {
        let first = self.at;
        let (mut acc, mut tree) = self.conjunction()?;
        while self.eat(&Tok::OrOr) {
            let (rhs, rhs_tree) = self.conjunction()?;
            acc = self.b.push(Op::Or(acc, rhs));
            tree = self.node(first, ExprKind::Logic(Logic::Or, Box::new(tree), Box::new(rhs_tree)));
        }
        Ok((acc, tree))
    }

    fn conjunction(&mut self) -> Result<(Val, Expr), ParseError> {
        let first = self.at;
        let (mut acc, mut tree) = self.relation()?;
        while self.eat(&Tok::AndAnd) {
            let (rhs, rhs_tree) = self.relation()?;
            acc = self.b.push(Op::And(acc, rhs));
            tree = self.node(first, ExprKind::Logic(Logic::And, Box::new(tree), Box::new(rhs_tree)));
        }
        Ok((acc, tree))
    }

    /// One comparison at most (`a < b < c` is not Fractint's either).
    fn relation(&mut self) -> Result<(Val, Expr), ParseError> {
        let first = self.at;
        let (lhs, lhs_tree) = self.sum()?;
        let Tok::Cmp(cmp) = *self.peek() else { return Ok((lhs, lhs_tree)) };
        self.at += 1;
        let (rhs, rhs_tree) = self.sum()?;
        let v = self.b.push(Op::Cmp(cmp, lhs, rhs));
        Ok((v, self.node(first, ExprKind::Cmp(cmp, Box::new(lhs_tree), Box::new(rhs_tree)))))
    }

    fn sum(&mut self) -> Result<(Val, Expr), ParseError> {
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
                let (first, first_tree) = self.assign_or_expr()?;
                if self.eat(&Tok::Comma) {
                    let (second, second_tree) = self.expr()?;
                    let v = self.complex_constant(tok_at, first, second)?;
                    self.expect(&Tok::RParen)?;
                    let tree = self.node(tok_at, ExprKind::Complex(Box::new(first_tree), Box::new(second_tree)));
                    return Ok((v, tree));
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
                    let open = self.at - 1;
                    let (mut a, mut arg) = self.assign_or_expr()?;
                    // `sin(1, 2)`: the call's parentheses are a complex constant's too (Fractint's).
                    if self.eat(&Tok::Comma) {
                        let (im, im_tree) = self.expr()?;
                        a = self.complex_constant(open, a, im)?;
                        self.expect(&Tok::RParen)?;
                        arg = self.node(open, ExprKind::Complex(Box::new(arg), Box::new(im_tree)));
                    } else {
                        self.expect(&Tok::RParen)?;
                    }
                    let tree = self.node(tok_at, ExprKind::Call { func: name, arg: Box::new(arg) });
                    return Ok((self.apply(f, a), tree));
                }
                if self.peek() == &Tok::LParen {
                    let message = match OTHER_SPELLINGS.iter().find(|(other, _, _)| *other == name) {
                        Some((_, ours, what)) => format!("unknown function `{name}`: {what} is written `{ours}` here"),
                        None => format!("unknown function `{name}`"),
                    };
                    return Err(self.err_at(tok_at, message));
                }
                let tree = self.node(tok_at, ExprKind::Name(name.clone()));
                if let Some(v) = self.var(&name) {
                    return Ok((v, tree));
                }
                // A predefined name the loop assigns is carried like any variable; before the loop
                // (and where nothing assigns it) it has its predefined value.
                if predefined(&name) && (self.in_init || !self.assigned.contains(&name)) {
                    return Ok((self.predefined_value(&name).expect("a predefined name"), tree));
                }
                match () {
                    // Assigned by a statement still to come (or in the loop, read in the init
                    // section): its value from the step before.
                    _ if self.assigned.contains(&name) => Ok((self.carried(&name), tree)),
                    _ if self.lower => Err(self.err_at(tok_at, format!("`{name}{UNASSIGNED}"))),
                    // Only the tree is wanted: stand in with `z`, which is always bound.
                    _ => Ok((self.leaf(Op::Z), tree)),
                }
            }
            Tok::Unsupported(what) => Err(self.err(format!("'{what}' is not an operator (Fractint's are && and ||)"))),
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
            Named::Cosxx => {
                let c = self.b.push(Op::Func(Func::Cos, a));
                self.b.push(Op::Conj(c))
            }
            // asin a = −i·log(i·a + √(1 − a²))
            Named::Asin => {
                let r = self.sqrt_one_minus_sq(a);
                let i = self.konst(0.0, 1.0);
                let ia = self.b.push(Op::Mul(i, a));
                let s = self.b.push(Op::Add(ia, r));
                self.minus_i_log(s)
            }
            // acos a = −i·log(a + i·√(1 − a²))
            Named::Acos => {
                let r = self.sqrt_one_minus_sq(a);
                let i = self.konst(0.0, 1.0);
                let ir = self.b.push(Op::Mul(i, r));
                let s = self.b.push(Op::Add(a, ir));
                self.minus_i_log(s)
            }
            // atan a = (i/2)·(log(1 − i·a) − log(1 + i·a))
            Named::Atan => {
                let (one, i) = (self.konst(1.0, 0.0), self.konst(0.0, 1.0));
                let ia = self.b.push(Op::Mul(i, a));
                let (m, p) = (self.b.push(Op::Sub(one, ia)), self.b.push(Op::Add(one, ia)));
                let (lm, lp) = (self.b.push(Op::Func(Func::Log, m)), self.b.push(Op::Func(Func::Log, p)));
                let d = self.b.push(Op::Sub(lm, lp));
                let half_i = self.konst(0.0, 0.5);
                self.b.push(Op::Mul(half_i, d))
            }
            // asinh a = log(a + √(a² + 1))
            Named::Asinh => {
                let one = self.konst(1.0, 0.0);
                let a2 = self.b.push(Op::Sqr(a));
                let s = self.b.push(Op::Add(a2, one));
                let r = self.b.push(Op::Func(Func::Sqrt, s));
                let t = self.b.push(Op::Add(a, r));
                self.b.push(Op::Func(Func::Log, t))
            }
            // acosh a = log(a + √(a + 1)·√(a − 1))
            Named::Acosh => {
                let one = self.konst(1.0, 0.0);
                let (p, m) = (self.b.push(Op::Add(a, one)), self.b.push(Op::Sub(a, one)));
                let (rp, rm) = (self.b.push(Op::Func(Func::Sqrt, p)), self.b.push(Op::Func(Func::Sqrt, m)));
                let r = self.b.push(Op::Mul(rp, rm));
                let t = self.b.push(Op::Add(a, r));
                self.b.push(Op::Func(Func::Log, t))
            }
            // atanh a = (log(1 + a) − log(1 − a))/2
            Named::Atanh => {
                let one = self.konst(1.0, 0.0);
                let (p, m) = (self.b.push(Op::Add(one, a)), self.b.push(Op::Sub(one, a)));
                let (lp, lm) = (self.b.push(Op::Func(Func::Log, p)), self.b.push(Op::Func(Func::Log, m)));
                let d = self.b.push(Op::Sub(lp, lm));
                self.b.push(Op::Scale(d, 0.5))
            }
            Named::Round(r) => self.b.push(Op::Round(r, a)),
        }
    }

    /// `√(1 − a²)`.
    fn sqrt_one_minus_sq(&mut self, a: Val) -> Val {
        let one = self.konst(1.0, 0.0);
        let a2 = self.b.push(Op::Sqr(a));
        let d = self.b.push(Op::Sub(one, a2));
        self.b.push(Op::Func(Func::Sqrt, d))
    }

    /// `−i·log(s)`.
    fn minus_i_log(&mut self, s: Val) -> Val {
        let l = self.b.push(Op::Func(Func::Log, s));
        let mi = self.konst(0.0, -1.0);
        self.b.push(Op::Mul(mi, l))
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod snapshot;

#[cfg(test)]
mod syntax_tests;
