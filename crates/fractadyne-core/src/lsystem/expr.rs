//! Arithmetic and logic on a parametric module's parameters (ABOP §1.10.1): numbers, the
//! production's formal parameters and the system's `define`d constants, combined with `+ - * / %`,
//! `^` (power), comparisons (`< <= > >= == !=`, and ABOP's `=` for equality), logic (`!`, `&` /
//! `&&`, `|` / `||`) and functions. A comparison or logical expression is 1 for true and 0 for
//! false; a condition holds when its value is not 0. Trigonometric functions take and give
//! degrees, as the turtle's angles are.

/// A parsed expression; variables are indices into the values it is evaluated with.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(f64),
    Var(u16),
    Neg(Box<Expr>),
    Not(Box<Expr>),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    Call(Func, Vec<Expr>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Func {
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Atan2,
    Sqrt,
    Exp,
    Ln,
    Log10,
    Abs,
    Floor,
    Ceil,
    Trunc,
    Min,
    Max,
    Pow,
    Sign,
}

impl Func {
    /// The function called `name`, and how many arguments it takes.
    fn named(name: &str) -> Option<(Func, usize)> {
        Some(match name {
            "sin" => (Func::Sin, 1),
            "cos" => (Func::Cos, 1),
            "tan" => (Func::Tan, 1),
            "asin" => (Func::Asin, 1),
            "acos" => (Func::Acos, 1),
            "atan" => (Func::Atan, 1),
            "atan2" => (Func::Atan2, 2),
            "sqrt" => (Func::Sqrt, 1),
            "exp" => (Func::Exp, 1),
            "log" | "ln" => (Func::Ln, 1),
            "log10" => (Func::Log10, 1),
            "abs" => (Func::Abs, 1),
            "floor" => (Func::Floor, 1),
            "ceil" => (Func::Ceil, 1),
            "trunc" => (Func::Trunc, 1),
            "min" => (Func::Min, 2),
            "max" => (Func::Max, 2),
            "pow" => (Func::Pow, 2),
            "sign" => (Func::Sign, 1),
            _ => return None,
        })
    }
}

/// Why an expression was refused: what, and its byte offset in the text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExprError {
    pub at: usize,
    pub message: String,
}

fn err<T>(at: usize, message: impl Into<String>) -> Result<T, ExprError> {
    Err(ExprError { at, message: message.into() })
}

/// How deeply an expression may nest (parentheses, unary operators, calls).
const MAX_NESTING: usize = 64;

/// Parses `text`, its names resolved against the formal parameters `vars` (by position) and the
/// constants `consts`.
pub fn parse(text: &str, vars: &[String], consts: &[(String, f64)]) -> Result<Expr, ExprError> {
    let mut p = Parser { s: text.as_bytes(), i: 0, vars, consts, nesting: 0 };
    let e = p.or()?;
    p.space();
    if p.i < p.s.len() {
        return err(p.i, format!("unexpected '{}'", p.s[p.i] as char));
    }
    Ok(e)
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    vars: &'a [String],
    consts: &'a [(String, f64)],
    nesting: usize,
}

impl Parser<'_> {
    fn space(&mut self) {
        while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }

    /// Takes `tok` if it is next (and, for `&` and `|`, its doubled form whole).
    fn eat(&mut self, tok: &str) -> bool {
        self.space();
        if self.s[self.i..].starts_with(tok.as_bytes()) {
            self.i += tok.len();
            true
        } else {
            false
        }
    }

    fn deeper(&mut self) -> Result<(), ExprError> {
        self.nesting += 1;
        if self.nesting > MAX_NESTING {
            return err(self.i, "the expression nests too deeply");
        }
        Ok(())
    }

    fn or(&mut self) -> Result<Expr, ExprError> {
        let mut a = self.and()?;
        while self.eat("||") || self.eat("|") {
            let b = self.and()?;
            a = Expr::Bin(BinOp::Or, Box::new(a), Box::new(b));
        }
        Ok(a)
    }

    fn and(&mut self) -> Result<Expr, ExprError> {
        let mut a = self.cmp()?;
        while self.eat("&&") || self.eat("&") {
            let b = self.cmp()?;
            a = Expr::Bin(BinOp::And, Box::new(a), Box::new(b));
        }
        Ok(a)
    }

    fn cmp(&mut self) -> Result<Expr, ExprError> {
        let mut a = self.add()?;
        loop {
            // Longest first: `<=` before `<`, `==` before `=`.
            let op = [("<=", BinOp::Le), (">=", BinOp::Ge), ("==", BinOp::Eq), ("!=", BinOp::Ne), ("<", BinOp::Lt), (">", BinOp::Gt), ("=", BinOp::Eq)]
                .into_iter()
                .find(|(t, _)| self.eat(t));
            let Some((_, op)) = op else { return Ok(a) };
            let b = self.add()?;
            a = Expr::Bin(op, Box::new(a), Box::new(b));
        }
    }

    fn add(&mut self) -> Result<Expr, ExprError> {
        let mut a = self.mul()?;
        loop {
            let op = if self.eat("+") {
                BinOp::Add
            } else if self.eat("-") {
                BinOp::Sub
            } else {
                return Ok(a);
            };
            let b = self.mul()?;
            a = Expr::Bin(op, Box::new(a), Box::new(b));
        }
    }

    fn mul(&mut self) -> Result<Expr, ExprError> {
        let mut a = self.unary()?;
        loop {
            let op = if self.eat("*") {
                BinOp::Mul
            } else if self.eat("/") {
                BinOp::Div
            } else if self.eat("%") {
                BinOp::Rem
            } else {
                return Ok(a);
            };
            let b = self.unary()?;
            a = Expr::Bin(op, Box::new(a), Box::new(b));
        }
    }

    fn unary(&mut self) -> Result<Expr, ExprError> {
        self.deeper()?;
        let e = if self.eat("-") {
            Expr::Neg(Box::new(self.unary()?))
        } else if self.eat("+") {
            self.unary()?
        } else if self.s[self.i..].starts_with(b"!") && !self.s[self.i..].starts_with(b"!=") && self.eat("!") {
            Expr::Not(Box::new(self.unary()?))
        } else {
            self.pow()?
        };
        self.nesting -= 1;
        Ok(e)
    }

    fn pow(&mut self) -> Result<Expr, ExprError> {
        let base = self.primary()?;
        if self.eat("^") {
            // Right-associative, and binding tighter than a minus before it: -2^2 is -4, 2^-1 is ½.
            let e = self.unary()?;
            return Ok(Expr::Bin(BinOp::Pow, Box::new(base), Box::new(e)));
        }
        Ok(base)
    }

    fn primary(&mut self) -> Result<Expr, ExprError> {
        self.space();
        let start = self.i;
        let Some(&c) = self.s.get(self.i) else { return err(self.i, "an expression ends too soon") };
        if c == b'(' {
            self.i += 1;
            self.deeper()?;
            let e = self.or()?;
            self.nesting -= 1;
            if !self.eat(")") {
                return err(self.i, "a '(' is never closed");
            }
            return Ok(e);
        }
        if c.is_ascii_digit() || c == b'.' {
            let mut j = self.i;
            while j < self.s.len() && (self.s[j].is_ascii_digit() || self.s[j] == b'.') {
                j += 1;
            }
            // An exponent: `1e-3`.
            if j < self.s.len() && matches!(self.s[j], b'e' | b'E') {
                let mut k = j + 1;
                if k < self.s.len() && matches!(self.s[k], b'+' | b'-') {
                    k += 1;
                }
                if k < self.s.len() && self.s[k].is_ascii_digit() {
                    while k < self.s.len() && self.s[k].is_ascii_digit() {
                        k += 1;
                    }
                    j = k;
                }
            }
            let text = std::str::from_utf8(&self.s[self.i..j]).unwrap_or("");
            let v: f64 = text.parse().map_err(|_| ExprError { at: start, message: format!("'{text}' is not a number") })?;
            self.i = j;
            return Ok(Expr::Num(v));
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            let mut j = self.i;
            while j < self.s.len() && (self.s[j].is_ascii_alphanumeric() || self.s[j] == b'_') {
                j += 1;
            }
            let name = std::str::from_utf8(&self.s[self.i..j]).unwrap_or("").to_string();
            self.i = j;
            self.space();
            if self.s.get(self.i) == Some(&b'(') {
                let Some((f, arity)) = Func::named(&name) else { return err(start, format!("no function '{name}'")) };
                self.i += 1;
                self.deeper()?;
                let mut args = Vec::new();
                if !self.eat(")") {
                    loop {
                        args.push(self.or()?);
                        if self.eat(")") {
                            break;
                        }
                        if !self.eat(",") {
                            return err(self.i, format!("'{name}(' needs ',' or ')'"));
                        }
                    }
                }
                self.nesting -= 1;
                if args.len() != arity {
                    return err(start, format!("'{name}' takes {arity} argument{}", if arity == 1 { "" } else { "s" }));
                }
                return Ok(Expr::Call(f, args));
            }
            // A formal parameter shadows a constant of the same name.
            if let Some(k) = self.vars.iter().position(|v| *v == name) {
                return Ok(Expr::Var(k as u16));
            }
            if let Some((_, v)) = self.consts.iter().rev().find(|(n, _)| *n == name) {
                return Ok(Expr::Num(*v));
            }
            return err(start, format!("'{name}' is not a parameter or a defined constant"));
        }
        err(start, format!("unexpected '{}'", c as char))
    }
}

fn truth(b: bool) -> f64 {
    if b {
        1.0
    } else {
        0.0
    }
}

impl Expr {
    /// The value, with `vals` for the formal parameters.
    pub fn eval(&self, vals: &[f64]) -> f64 {
        match self {
            Expr::Num(v) => *v,
            Expr::Var(k) => vals.get(*k as usize).copied().unwrap_or(f64::NAN),
            Expr::Neg(a) => -a.eval(vals),
            Expr::Not(a) => truth(a.eval(vals) == 0.0),
            Expr::Bin(op, a, b) => {
                let x = a.eval(vals);
                // `&` and `|` look at their right side only when they must.
                match op {
                    BinOp::And => return truth(x != 0.0 && !x.is_nan() && b.eval(vals) != 0.0),
                    BinOp::Or => return truth((x != 0.0 && !x.is_nan()) || b.eval(vals) != 0.0),
                    _ => {}
                }
                let y = b.eval(vals);
                match op {
                    BinOp::Add => x + y,
                    BinOp::Sub => x - y,
                    BinOp::Mul => x * y,
                    BinOp::Div => x / y,
                    BinOp::Rem => x % y,
                    BinOp::Pow => x.powf(y),
                    BinOp::Lt => truth(x < y),
                    BinOp::Le => truth(x <= y),
                    BinOp::Gt => truth(x > y),
                    BinOp::Ge => truth(x >= y),
                    BinOp::Eq => truth(x == y),
                    BinOp::Ne => truth(x != y),
                    BinOp::And | BinOp::Or => unreachable!("handled above"),
                }
            }
            Expr::Call(f, args) => {
                let a = |k: usize| args[k].eval(vals);
                match f {
                    Func::Sin => a(0).to_radians().sin(),
                    Func::Cos => a(0).to_radians().cos(),
                    Func::Tan => a(0).to_radians().tan(),
                    Func::Asin => a(0).asin().to_degrees(),
                    Func::Acos => a(0).acos().to_degrees(),
                    Func::Atan => a(0).atan().to_degrees(),
                    Func::Atan2 => a(0).atan2(a(1)).to_degrees(),
                    Func::Sqrt => a(0).sqrt(),
                    Func::Exp => a(0).exp(),
                    Func::Ln => a(0).ln(),
                    Func::Log10 => a(0).log10(),
                    Func::Abs => a(0).abs(),
                    Func::Floor => a(0).floor(),
                    Func::Ceil => a(0).ceil(),
                    Func::Trunc => a(0).trunc(),
                    Func::Min => a(0).min(a(1)),
                    Func::Max => a(0).max(a(1)),
                    Func::Pow => a(0).powf(a(1)),
                    Func::Sign => {
                        let v = a(0);
                        if v > 0.0 {
                            1.0
                        } else if v < 0.0 {
                            -1.0
                        } else {
                            0.0
                        }
                    }
                }
            }
        }
    }

    /// Whether, as a condition, it holds (a value that is not 0, and is a number).
    pub fn holds(&self, vals: &[f64]) -> bool {
        let v = self.eval(vals);
        v != 0.0 && !v.is_nan()
    }
}

#[cfg(test)]
mod tests;
