//! The formula language's syntax tree: what [`super::parse::syntax`] reads from a source, before
//! any of it is evaluated (design/formula-textbook-editor.md §4.2).
//!
//! The parser builds this tree in the same pass, and by the same code, as the IR — so the two can
//! never disagree about precedence or grouping. It keeps what the IR drops: parentheses as written,
//! unary plus, every node's place in the source, and the comments. The formula editor's textbook
//! mode typesets and edits from it.

/// A byte range of the source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// A formula: its statements in order, and its comments.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Syntax {
    pub statements: Vec<Statement>,
    pub comments: Vec<Comment>,
}

/// `target = body`, or a bare `body` (the new `z`).
#[derive(Clone, Debug, PartialEq)]
pub struct Statement {
    pub target: Option<Name>,
    pub body: Expr,
    pub span: Span,
}

/// A name as read: lower-cased (names are case-insensitive), with where it was written.
#[derive(Clone, Debug, PartialEq)]
pub struct Name {
    pub name: String,
    pub span: Span,
}

/// `; text` to the end of its line; `text` is what follows the `;`.
#[derive(Clone, Debug, PartialEq)]
pub struct Comment {
    pub text: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    /// A number; its spelling is the source at the span.
    Num(f64),
    /// A variable, parameter or constant name, lower-cased: `z`, `c`, `pixel`, `p1`, `pi`, `t`.
    Name(String),
    /// A named function applied to its argument; `func` lower-cased.
    Call { func: String, arg: Box<Expr> },
    /// Parentheses as written.
    Group(Box<Expr>),
    /// `(re, im)`.
    Complex(Box<Expr>, Box<Expr>),
    /// `|x|`, the squared modulus.
    Bars(Box<Expr>),
    Neg(Box<Expr>),
    Pos(Box<Expr>),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    /// `base ^ exponent`.
    Pow(Box<Expr>, Box<Expr>),
    /// A comparison of real parts (Fractint's): `|z| <= 4`.
    Cmp(super::Cmp, Box<Expr>, Box<Expr>),
    /// `a && b`, `a || b`.
    Logic(Logic, Box<Expr>, Box<Expr>),
    /// An assignment used as a value (Fractint's): the `b = pixel` of `a = b = pixel`.
    Assign(Name, Box<Expr>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Logic {
    And,
    Or,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}
