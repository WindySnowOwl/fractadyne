//! The formula **intermediate representation** (design/custom-formulas.md §4.1).
//!
//! A formula's step is a [`Program`]: instructions in SSA form, each computing one complex value
//! from earlier ones — a linearised expression DAG, so a shared subexpression (the `store` of the
//! hybrid opcode model, a Fractint temporary) is computed once. Every front end lowers to it (the
//! hybrid opcode model via [`lower_opcodes`] today; a Fractint `.frm` subset and an author-supplied
//! perturbed step later), and every numeric form is generated from it instead of being written by
//! hand per formula:
//!
//! - [`orbit_points`] / [`step_f64`] — the `f64` interpreter (orbit overlay, shallow CPU oracle);
//! - [`reference_orbit`] — the bignum interpreter, in astro-float or (when built) MPFR, for the
//!   programs whose operations are all exact-ring ones ([`Program::bignum_evaluable`]).
//!
//! # Bit-identity with the hand-written steps
//!
//! All ten built-in steps are expressible here ([`builtin_step`]), and the nine that iterate with an
//! escape test reproduce their hand-written orbits **bit for bit** in `f64` and in bignum; Newton's
//! step does too, in `f64` (it has no bignum path). The tests in `ir/tests.rs` are the gate. Three
//! rules carry it:
//!
//! 1. **The same operation order.** `Mul` is `ax·bx − ay·by, ax·by + ay·bx` and `Sqr` is
//!    `x·x − y·y, 2·(x·y)` — the core's own `cmul` / `csqr` — and [`Op::PowI`] squares and
//!    multiplies from the most significant bit, which is exactly the Multibrot 3/4/5 chains.
//! 2. **Negation is a sign flag, never an operation.** The interpreter carries a sign with every
//!    scalar and folds it into the next add or multiply, so `conj(z)² + c` evaluates its imaginary
//!    part as `cy − 2xy`, Tricorn's hand-written form. Negation and conjugation therefore cost
//!    nothing and allocate nothing in bignum. (Measured with the fold removed: astro-float's
//!    `(−a) + b` has the same bits as `b − a` on every gate case, so identity does not depend on
//!    the fold. The fold means it never has to.)
//! 3. **Constants are exact.** An `f64` constant is exact in every field; it is converted once per
//!    orbit ([`IrField::konst`]), the way the Phoenix step converts its `0.5`.

use crate::backend::RefBackend;
use crate::fractal::Field;
use crate::reference::{pack_sample, split_df64, OrbitTail};
use astro_float::BigFloat;

/// A value in a [`Program`]: the index of the instruction that computes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Val(u32);

impl Val {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Elementary functions. Every one also evaluates in bignum (a custom formula's reference orbit),
/// where the two backends agree only to within their rounding — unlike the ring operations, which
/// they reproduce bit for bit. That is harmless for a custom formula, whose orbits are never cached
/// or shared, and it is why a built-in never uses them. `log` and `sqrt` are the principal branches,
/// cut along the negative real axis, and every field puts a point ON the cut (a zero imaginary
/// part of either sign) on its upper side: `Log(−1) = iπ`, `sqrt(−1) = i`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Func {
    Exp,
    Log,
    Sqrt,
    Sin,
    Cos,
    Tan,
    Sinh,
    Cosh,
    Tanh,
    /// Internal, in perturbed steps only (no formula-language name): `sin` of a SMALL argument,
    /// accurate RELATIVE to it. A GPU's `sin` is accurate only in absolute terms (~5e-7), which is
    /// all of a 1e-10 offset.
    SinSmall,
    /// Internal: `sinh` of a small argument, relative accuracy (as [`Func::SinSmall`]).
    SinhSmall,
    /// Internal: `exp(a) − 1`, relative accuracy for a small `a` (where `exp(a) − 1` cancels).
    Expm1,
}

/// One instruction. Operands are earlier instructions' values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    /// The current iterate.
    Z,
    /// The pixel (parameter plane) or the Julia constant.
    C,
    /// The previous iterate, zero before the first step (Phoenix-type formulas).
    ZPrev,
    /// A formula parameter, by index into the values supplied at evaluation.
    Param(u16),
    /// A complex constant.
    Const(f64, f64),
    Add(Val, Val),
    Sub(Val, Val),
    Mul(Val, Val),
    Sqr(Val),
    /// An integer power `n ≥ 1`: square and multiply from the most significant bit.
    PowI(Val, u32),
    /// Both parts times a real constant.
    Scale(Val, f64),
    Neg(Val),
    Conj(Val),
    /// `|Re| + i·Im`.
    AbsRe(Val),
    /// `Re + i·|Im|`.
    AbsIm(Val),
    /// `Re + 0i`.
    Re(Val),
    /// `Im + 0i`.
    Im(Val),
    /// `|z|² + 0i` — Fractint's `|z|`.
    Norm(Val),
    /// Complex division, `a·conj(b) / |b|²`. Not exact-ring: `f64` and bignum, each rounding it
    /// its own way (see [`Func`]).
    Div(Val, Val),
    /// `a^b = exp(b·Log a)`, and `0^b = 0`. `f64` and bignum.
    Pow(Val, Val),
    /// An elementary function, in `f64` and bignum (see [`Func`]).
    Func(Func, Val),
    /// The perturbation of the iterate, δz — in a PERTURBED program ([`perturb`]), where `Z` and
    /// `C` are the reference's values.
    Delta,
    /// The perturbation of c, δc (zero in Julia mode).
    DeltaC,
    /// `(diffabs(Re b, Re p), Im p)`: the perturbation of `AbsRe` at reference `b` with
    /// perturbation `p` — `|Re b + Re p| − |Re b|` without cancellation. `f64` only (it compares).
    DiffAbsRe(Val, Val),
    /// `(Re p, diffabs(Im b, Im p))`, likewise for `AbsIm`.
    DiffAbsIm(Val, Val),
    /// `tanh(b + p) − tanh(b)`: the perturbation of `tanh` at reference `b` with perturbation `p`,
    /// without cancellation and without overflow (a product form below [`TANH_DIFF_SPLIT`], the
    /// plain difference above it). `f64` only (it branches).
    DiffTanh(Val, Val),
    /// `tan(b + p) − tan(b)`, likewise.
    DiffTan(Val, Val),
    /// `Log(b + p) − Log(b)` for the PRINCIPAL logarithm: `log1p(p/b)` (accurate relative to a
    /// small `p`), plus `2πi` where `b` and `b + p` sit on opposite sides of the branch cut on the
    /// negative real axis — see [`log_diff`]. `f64` only (it branches).
    DiffLog(Val, Val),
    /// `sqrt(b + p) − sqrt(b)`, principal branch: `p / (sqrt(b + p) + sqrt(b))`, or the plain
    /// difference where the two roots point apart (across the cut) — see [`sqrt_diff`].
    DiffSqrt(Val, Val),
    /// `(b + p)^k − b^k` with the principal power `exp(k·Log a)` and `0^k = 0`, for an exponent `k`
    /// with no perturbation — see [`pow_diff`].
    DiffPow(Val, Val, Val),
    /// Persistent variable `i` of a `.frm`-style formula ([`Formula::vars`]): its value from the
    /// step before (or the init section's, or zero). `f64` only, like everything below.
    Var(u16),
    /// `(1, 0)` where the comparison holds between the REAL parts, else `(0, 0)` — Fractint's.
    Cmp(Cmp, Val, Val),
    /// Fractint's `&&`: `(1, 0)` where both real parts are non-zero. Both sides are evaluated.
    And(Val, Val),
    /// Fractint's `||`.
    Or(Val, Val),
    /// `a` where the real part of `cond` is non-zero, else `b`: an `if` block, its branches both
    /// computed (an untaken branch's NaN goes nowhere).
    Select(Val, Val, Val),
    /// The render's iteration cap, `(max_iter, 0)` — Fractint's `maxit`.
    MaxIter,
    /// Both parts rounded to integers (Fractint's `floor`, `ceil`, `trunc`, `round`).
    Round(Round, Val),
}

/// How [`Op::Round`] rounds each part.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Round {
    Floor,
    Ceil,
    /// Toward zero.
    Trunc,
    /// Fractint's `round`: `floor(x + 0.5)` (halves up, not away from zero).
    Nearest,
}

impl Round {
    pub fn apply(self, x: f64) -> f64 {
        match self {
            Round::Floor => x.floor(),
            Round::Ceil => x.ceil(),
            Round::Trunc => x.trunc(),
            Round::Nearest => (x + 0.5).floor(),
        }
    }
}

/// A comparison of real parts ([`Op::Cmp`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cmp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

impl Cmp {
    pub fn holds(self, a: f64, b: f64) -> bool {
        match self {
            Cmp::Lt => a < b,
            Cmp::Le => a <= b,
            Cmp::Gt => a > b,
            Cmp::Ge => a >= b,
            Cmp::Eq => a == b,
            Cmp::Ne => a != b,
        }
    }

    /// The operator as written (and in WGSL).
    pub fn symbol(self) -> &'static str {
        match self {
            Cmp::Lt => "<",
            Cmp::Le => "<=",
            Cmp::Gt => ">",
            Cmp::Ge => ">=",
            Cmp::Eq => "==",
            Cmp::Ne => "!=",
        }
    }
}

impl Op {
    /// The operands this instruction reads.
    pub fn operands(&self) -> impl Iterator<Item = Val> {
        let (a, b, c) = match *self {
            Op::Z
            | Op::C
            | Op::ZPrev
            | Op::Param(_)
            | Op::Const(..)
            | Op::Delta
            | Op::DeltaC
            | Op::Var(_)
            | Op::MaxIter => (None, None, None),
            Op::Select(c, a, b) => (Some(c), Some(a), Some(b)),
            Op::Round(_, a) => (Some(a), None, None),
            Op::Cmp(_, a, b) | Op::And(a, b) | Op::Or(a, b) => (Some(a), Some(b), None),
            Op::Add(a, b)
            | Op::Sub(a, b)
            | Op::Mul(a, b)
            | Op::Div(a, b)
            | Op::Pow(a, b)
            | Op::DiffAbsRe(a, b)
            | Op::DiffAbsIm(a, b)
            | Op::DiffTanh(a, b)
            | Op::DiffTan(a, b)
            | Op::DiffLog(a, b)
            | Op::DiffSqrt(a, b) => (Some(a), Some(b), None),
            Op::DiffPow(a, b, k) => (Some(a), Some(b), Some(k)),
            Op::Sqr(a)
            | Op::PowI(a, _)
            | Op::Scale(a, _)
            | Op::Neg(a)
            | Op::Conj(a)
            | Op::AbsRe(a)
            | Op::AbsIm(a)
            | Op::Re(a)
            | Op::Im(a)
            | Op::Norm(a)
            | Op::Func(_, a) => (Some(a), None, None),
        };
        a.into_iter().chain(b).chain(c)
    }

    /// Whether every field this crate iterates in evaluates it exactly the same way: the ring
    /// operations plus the exact ones (sign, abs, parts). Division and the elementary functions are
    /// not — see [`Func`].
    pub fn is_ring(&self) -> bool {
        !matches!(
            self,
            Op::Div(..)
                | Op::Pow(..)
                | Op::Func(..)
                | Op::DiffAbsRe(..)
                | Op::DiffAbsIm(..)
                | Op::DiffTanh(..)
                | Op::DiffTan(..)
                | Op::DiffLog(..)
                | Op::DiffSqrt(..)
                | Op::DiffPow(..)
                | Op::Var(_)
                | Op::Cmp(..)
                | Op::And(..)
                | Op::Or(..)
                | Op::Select(..)
                | Op::MaxIter
                | Op::Round(..)
        )
    }

    /// Whether the bignum fields evaluate it at all: the ring, division, powers, and the functions
    /// with a bignum form (see [`Func`]).
    pub fn is_bignum(&self) -> bool {
        self.is_ring()
            || matches!(
                self,
                Op::Div(..)
                    | Op::Pow(..)
                    | Op::Func(
                        Func::Exp
                            | Func::Log
                            | Func::Sqrt
                            | Func::Sin
                            | Func::Cos
                            | Func::Tan
                            | Func::Sinh
                            | Func::Cosh
                            | Func::Tanh,
                        _
                    )
            )
    }
}

/// Why a program or formula was rejected, or cannot be evaluated where it was asked to be.
#[derive(Clone, Debug, PartialEq)]
pub enum IrError {
    /// A program needs at least one instruction, a formula at least one phase.
    Empty,
    /// Instruction `inst` reads `operand`, which is not an earlier instruction.
    BadOperand { inst: usize, operand: usize },
    /// The output is not an instruction of the program.
    BadOutput(usize),
    /// `PowI` with exponent 0 (write the constant 1 instead).
    ZeroPower { inst: usize },
    /// A constant or scale factor is not finite.
    NonFinite { inst: usize },
    /// A `mul` in an opcode program before any `store`.
    MulBeforeStore { opcode: usize },
    /// The program reads parameter `index`; only `supplied` values were given.
    MissingParam { index: u16, supplied: usize },
    /// A bignum evaluation of a program with an operation bignum has no form of (see
    /// [`Op::is_bignum`]).
    NotBignum,
}

impl std::fmt::Display for IrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IrError::Empty => write!(f, "empty program or formula"),
            IrError::BadOperand { inst, operand } => {
                write!(f, "instruction {inst} reads {operand}, which is not an earlier instruction")
            }
            IrError::BadOutput(v) => write!(f, "output {v} is not an instruction of the program"),
            IrError::ZeroPower { inst } => write!(f, "instruction {inst}: integer power 0"),
            IrError::NonFinite { inst } => write!(f, "instruction {inst}: constant is not finite"),
            IrError::MulBeforeStore { opcode } => {
                write!(f, "opcode {opcode}: `mul` before any `store`")
            }
            IrError::MissingParam { index, supplied } => {
                write!(f, "reads parameter {index}, but {supplied} were supplied")
            }
            IrError::NotBignum => write!(
                f,
                "a perturbed step's own operations have no bignum form (f64 only)"
            ),
        }
    }
}

impl std::error::Error for IrError {}

/// One step `z → z'` in SSA form. Construct with [`Builder`] or [`Program::new`], which validate.
#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    insts: Vec<Op>,
    out: Val,
    /// The persistent variables the step sets: `(variable, its new value)` ([`Formula::vars`]).
    vars: Vec<(u16, Val)>,
    /// The formula's own bailout: iterate while this value's real part is not 0 — Fractint's last
    /// loop statement, evaluated with the step's values. `None`: until `|z|` passes the escape
    /// radius.
    cond: Option<Val>,
}

impl Program {
    /// Validate and wrap: every operand reads an EARLIER instruction, the output exists, integer
    /// powers are positive and constants finite.
    pub fn new(insts: Vec<Op>, out: Val) -> Result<Self, IrError> {
        if insts.is_empty() {
            return Err(IrError::Empty);
        }
        for (i, op) in insts.iter().enumerate() {
            if let Some(v) = op.operands().find(|v| v.index() >= i) {
                return Err(IrError::BadOperand { inst: i, operand: v.index() });
            }
            match *op {
                Op::PowI(_, 0) => return Err(IrError::ZeroPower { inst: i }),
                Op::Const(re, im) if !(re.is_finite() && im.is_finite()) => {
                    return Err(IrError::NonFinite { inst: i })
                }
                Op::Scale(_, k) if !k.is_finite() => return Err(IrError::NonFinite { inst: i }),
                _ => {}
            }
        }
        if out.index() >= insts.len() {
            return Err(IrError::BadOutput(out.index()));
        }
        Ok(Program { insts, out, vars: Vec::new(), cond: None })
    }

    /// The same program also setting persistent variables: `(variable, value)`, each value an
    /// instruction of the program.
    pub fn with_vars(mut self, vars: Vec<(u16, Val)>) -> Result<Self, IrError> {
        if let Some(&(_, v)) = vars.iter().find(|(_, v)| v.index() >= self.insts.len()) {
            return Err(IrError::BadOutput(v.index()));
        }
        self.vars = vars;
        Ok(self)
    }

    /// The same program with its own bailout condition (see [`Program::cond`]).
    pub fn with_cond(mut self, cond: Val) -> Result<Self, IrError> {
        if cond.index() >= self.insts.len() {
            return Err(IrError::BadOutput(cond.index()));
        }
        self.cond = Some(cond);
        Ok(self)
    }

    /// The bailout condition: iterate while its real part is not 0.
    pub fn cond(&self) -> Option<Val> {
        self.cond
    }

    pub fn insts(&self) -> &[Op] {
        &self.insts
    }

    pub fn out(&self) -> Val {
        self.out
    }

    /// The persistent variables this step sets, with their new values.
    pub fn vars(&self) -> &[(u16, Val)] {
        &self.vars
    }

    pub fn reads_zprev(&self) -> bool {
        self.insts.iter().any(|op| matches!(op, Op::ZPrev))
    }

    /// Whether the bignum interpreter can run it (see [`Op::is_bignum`]).
    pub fn bignum_evaluable(&self) -> bool {
        self.insts.iter().all(Op::is_bignum)
    }

    /// The same program without the instructions its output does not depend on, renumbered in
    /// order. Evaluation is unchanged (every surviving instruction computes what it did).
    pub fn without_dead_code(&self) -> Program {
        let mut live = vec![false; self.insts.len()];
        live[self.out.index()] = true;
        for &(_, v) in &self.vars {
            live[v.index()] = true;
        }
        if let Some(v) = self.cond {
            live[v.index()] = true;
        }
        for i in (0..self.insts.len()).rev() {
            if live[i] {
                for v in self.insts[i].operands() {
                    live[v.index()] = true;
                }
            }
        }
        let mut map = vec![0u32; self.insts.len()];
        let mut insts = Vec::with_capacity(self.insts.len());
        let remap = |v: Val, map: &[u32]| Val(map[v.index()]);
        for (i, op) in self.insts.iter().enumerate() {
            if !live[i] {
                continue;
            }
            map[i] = insts.len() as u32;
            insts.push(match *op {
                Op::Add(a, b) => Op::Add(remap(a, &map), remap(b, &map)),
                Op::Sub(a, b) => Op::Sub(remap(a, &map), remap(b, &map)),
                Op::Mul(a, b) => Op::Mul(remap(a, &map), remap(b, &map)),
                Op::Div(a, b) => Op::Div(remap(a, &map), remap(b, &map)),
                Op::Pow(a, b) => Op::Pow(remap(a, &map), remap(b, &map)),
                Op::Sqr(a) => Op::Sqr(remap(a, &map)),
                Op::PowI(a, n) => Op::PowI(remap(a, &map), n),
                Op::Scale(a, k) => Op::Scale(remap(a, &map), k),
                Op::Neg(a) => Op::Neg(remap(a, &map)),
                Op::Conj(a) => Op::Conj(remap(a, &map)),
                Op::AbsRe(a) => Op::AbsRe(remap(a, &map)),
                Op::AbsIm(a) => Op::AbsIm(remap(a, &map)),
                Op::Re(a) => Op::Re(remap(a, &map)),
                Op::Im(a) => Op::Im(remap(a, &map)),
                Op::Norm(a) => Op::Norm(remap(a, &map)),
                Op::Func(f, a) => Op::Func(f, remap(a, &map)),
                Op::DiffAbsRe(a, b) => Op::DiffAbsRe(remap(a, &map), remap(b, &map)),
                Op::DiffAbsIm(a, b) => Op::DiffAbsIm(remap(a, &map), remap(b, &map)),
                Op::DiffTanh(a, b) => Op::DiffTanh(remap(a, &map), remap(b, &map)),
                Op::DiffTan(a, b) => Op::DiffTan(remap(a, &map), remap(b, &map)),
                Op::DiffLog(a, b) => Op::DiffLog(remap(a, &map), remap(b, &map)),
                Op::DiffSqrt(a, b) => Op::DiffSqrt(remap(a, &map), remap(b, &map)),
                Op::DiffPow(a, b, k) => Op::DiffPow(remap(a, &map), remap(b, &map), remap(k, &map)),
                Op::Cmp(cmp, a, b) => Op::Cmp(cmp, remap(a, &map), remap(b, &map)),
                Op::And(a, b) => Op::And(remap(a, &map), remap(b, &map)),
                Op::Or(a, b) => Op::Or(remap(a, &map), remap(b, &map)),
                Op::Select(c, a, b) => Op::Select(remap(c, &map), remap(a, &map), remap(b, &map)),
                Op::Round(r, a) => Op::Round(r, remap(a, &map)),
                leaf @ (Op::Z
                | Op::C
                | Op::ZPrev
                | Op::Param(_)
                | Op::Const(..)
                | Op::Delta
                | Op::DeltaC
                | Op::Var(_)
                | Op::MaxIter) => leaf,
            });
        }
        let vars = self.vars.iter().map(|&(k, v)| (k, Val(map[v.index()]))).collect();
        let cond = self.cond.map(|v| Val(map[v.index()]));
        Program { insts, out: Val(map[self.out.index()]), vars, cond }
    }

    /// One more than the highest parameter index read (0 if none).
    pub fn param_count(&self) -> usize {
        self.insts
            .iter()
            .filter_map(|op| match op {
                Op::Param(i) => Some(*i as usize + 1),
                _ => None,
            })
            .max()
            .unwrap_or(0)
    }

    /// The step's degree in `z` as `|z| → ∞` — the `d` in `|z'| ≈ |z|^d` that smooth colouring
    /// divides by (`z^5 + c` → 5, `|x| + i|y|` squared → 2, Phoenix → 2). `None` where no power law
    /// holds: exponentials, trigonometry, logarithms, a non-constant exponent.
    pub fn escape_degree(&self) -> Option<f64> {
        let mut deg: Vec<Option<f64>> = Vec::with_capacity(self.insts.len());
        for op in &self.insts {
            let g = |v: Val| deg[v.index()];
            let d = match *op {
                Op::Z | Op::ZPrev => Some(1.0),
                Op::C | Op::Param(_) | Op::Const(..) => Some(0.0),
                Op::Add(a, b) | Op::Sub(a, b) => g(a).zip(g(b)).map(|(a, b)| a.max(b)),
                Op::Mul(a, b) => g(a).zip(g(b)).map(|(a, b)| a + b),
                Op::Div(a, b) => g(a).zip(g(b)).map(|(a, b)| a - b),
                Op::Sqr(a) | Op::Norm(a) => g(a).map(|a| 2.0 * a),
                Op::PowI(a, n) => g(a).map(|a| n as f64 * a),
                Op::Scale(a, _)
                | Op::Neg(a)
                | Op::Conj(a)
                | Op::AbsRe(a)
                | Op::AbsIm(a)
                | Op::Re(a)
                | Op::Im(a) => g(a),
                Op::Pow(a, b) => match self.insts[b.index()] {
                    Op::Const(k, 0.0) => g(a).map(|a| k * a),
                    _ => None,
                },
                Op::Func(Func::Sqrt, a) => g(a).map(|a| 0.5 * a),
                Op::Func(..) => None,
                // Perturbed programs: δz is of the iterate's degree, δc of c's.
                Op::Delta => Some(1.0),
                Op::DeltaC => Some(0.0),
                Op::DiffAbsRe(_, p) | Op::DiffAbsIm(_, p) => g(p),
                Op::DiffTanh(..) | Op::DiffTan(..) | Op::DiffLog(..) | Op::DiffSqrt(..) | Op::DiffPow(..) => None,
                // A comparison is 0 or 1; a branch is as large as its larger side; a variable
                // carried from step to step follows no law the step alone shows.
                Op::Cmp(..) | Op::And(..) | Op::Or(..) | Op::MaxIter => Some(0.0),
                Op::Select(_, a, b) => g(a).zip(g(b)).map(|(a, b)| a.max(b)),
                Op::Var(_) => None,
                // Rounding moves a value by less than 1: its size follows the operand's.
                Op::Round(_, a) => g(a),
            };
            deg.push(d);
        }
        deg[self.out.index()]
    }
}

/// Appends instructions and hands back their values.
#[derive(Default)]
pub struct Builder {
    insts: Vec<Op>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, op: Op) -> Val {
        self.insts.push(op);
        Val(self.insts.len() as u32 - 1)
    }

    pub fn finish(self, out: Val) -> Result<Program, IrError> {
        Program::new(self.insts, out)
    }
}

/// A formula: one program per phase, run in turn (iteration `n` runs phase `n mod len`) — the
/// hybrid model's interleaved lines. An ordinary formula has one phase.
///
/// A Fractint-style (`.frm`) formula may also have persistent variables, kept from one step to the
/// next ([`Op::Var`]); an init section, run once per pixel before the first step; and a bailout
/// condition of its own ([`Program::cond`]). Such a formula evaluates in `f64` only and renders on
/// the direct path (no reference orbit, no chunked passes).
#[derive(Clone, Debug, PartialEq)]
pub struct Formula {
    phases: Vec<Program>,
    /// How many persistent variables there are (`Op::Var(i)`, `i < vars`).
    vars: u16,
    /// Run once per pixel, from `z = z₀` (0, or the pixel in Julia mode) and every variable 0: its
    /// output is the starting `z`, and it sets the variables in its `vars`.
    init: Option<Program>,
}

impl Formula {
    pub fn new(phases: Vec<Program>) -> Result<Self, IrError> {
        if phases.is_empty() {
            return Err(IrError::Empty);
        }
        Ok(Formula { phases, vars: 0, init: None })
    }

    pub fn single(step: Program) -> Self {
        Formula { phases: vec![step], vars: 0, init: None }
    }

    /// The same formula with Fractint's sections: `vars` persistent variables and an init section.
    /// Every `Op::Var` read or set must be below `vars`.
    pub fn with_sections(mut self, vars: u16, init: Option<Program>) -> Result<Self, IrError> {
        let all = self.phases.iter().chain(&init);
        for p in all {
            let bad = p.insts.iter().find_map(|op| match op {
                Op::Var(i) if *i >= vars => Some(*i),
                _ => None,
            });
            if let Some(i) = bad.or_else(|| p.vars.iter().map(|&(i, _)| i).find(|&i| i >= vars)) {
                return Err(IrError::BadOutput(i as usize));
            }
        }
        self.vars = vars;
        self.init = init;
        Ok(self)
    }

    pub fn phases(&self) -> &[Program] {
        &self.phases
    }

    pub fn vars(&self) -> u16 {
        self.vars
    }

    pub fn init(&self) -> Option<&Program> {
        self.init.as_ref()
    }

    /// Whether a phase has a bailout condition of its own.
    pub fn has_bailout(&self) -> bool {
        self.phases.iter().any(|p| p.cond.is_some())
    }

    /// Whether it has any of Fractint's sections: persistent variables, an init or a bailout.
    pub fn has_sections(&self) -> bool {
        self.vars > 0 || self.init.is_some() || self.has_bailout()
    }

    pub fn reads_zprev(&self) -> bool {
        self.phases.iter().any(Program::reads_zprev)
    }

    pub fn bignum_evaluable(&self) -> bool {
        !self.has_sections() && self.phases.iter().all(Program::bignum_evaluable)
    }

    pub fn param_count(&self) -> usize {
        self.phases.iter().chain(&self.init).map(Program::param_count).max().unwrap_or(0)
    }

    /// Program `k` of [`Machine`]'s numbering: the phases, then the init section.
    fn program(&self, k: usize) -> &Program {
        match self.phases.get(k) {
            Some(p) => p,
            None => self.init.as_ref().expect("init numbered only if present"),
        }
    }

    /// The per-iteration escape degree: the geometric mean of the phases' (a hybrid alternating
    /// `z²` and `z³` grows as `|z|^√6` per iteration). `None` if any phase has none. Phases of one
    /// degree give it exactly: `exp(ln 7)` is 6.999999999999999, which a rule taking effect from
    /// degree 7 (the reference's escape test) must not read.
    pub fn escape_degree(&self) -> Option<f64> {
        let degrees = self.phases.iter().map(|p| p.escape_degree()).collect::<Option<Vec<f64>>>()?;
        if degrees.iter().all(|d| *d == degrees[0]) {
            return Some(degrees[0]);
        }
        let log_sum: f64 = degrees.iter().map(|d| d.ln()).sum();
        Some((log_sum / degrees.len() as f64).exp())
    }
}

// ---------------------------------------------------------------------------------------------
// The hybrid opcode model
// ---------------------------------------------------------------------------------------------

/// One operation of the hybrid opcode model (the model Kalles Fraktaler's hybrid designer and
/// Fraktaler-3's formula window share). This is the model, not a file format: reading `.kfr`
/// `HybridFormula` strings and `.f3.toml` `[[formula]]` blocks into it is design phase 4.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Opcode {
    /// Remember the current value for a later [`Opcode::Mul`].
    Store,
    /// `z = z²`.
    Sqr,
    /// `z = z × stored`.
    Mul,
    /// `Re z = |Re z|`.
    AbsX,
    /// `Im z = |Im z|`.
    AbsY,
    /// `Re z = −Re z`.
    NegX,
    /// `Im z = −Im z`.
    NegY,
    /// Rotate by this many degrees.
    Rot(f64),
    /// `z = z + c`.
    AddC,
}

/// Lower an opcode program to a [`Program`]. Nothing is implicit: a program that should add `c`
/// ends with [`Opcode::AddC`].
pub fn lower_opcodes(ops: &[Opcode]) -> Result<Program, IrError> {
    let mut b = Builder::new();
    let mut cur = b.push(Op::Z);
    let mut stored = None;
    for (i, op) in ops.iter().enumerate() {
        cur = match *op {
            Opcode::Store => {
                stored = Some(cur);
                cur
            }
            Opcode::Sqr => b.push(Op::Sqr(cur)),
            Opcode::Mul => {
                let s = stored.ok_or(IrError::MulBeforeStore { opcode: i })?;
                b.push(Op::Mul(cur, s))
            }
            Opcode::AbsX => b.push(Op::AbsRe(cur)),
            Opcode::AbsY => b.push(Op::AbsIm(cur)),
            // −conj(z) = −x + iy: the real part negated, with no arithmetic (a sign flag each).
            Opcode::NegX => {
                let t = b.push(Op::Conj(cur));
                b.push(Op::Neg(t))
            }
            Opcode::NegY => b.push(Op::Conj(cur)),
            Opcode::Rot(deg) => {
                let (s, c) = rotation(deg);
                let r = b.push(Op::Const(c, s));
                b.push(Op::Mul(cur, r))
            }
            Opcode::AddC => {
                let c = b.push(Op::C);
                b.push(Op::Add(cur, c))
            }
        };
    }
    b.finish(cur)
}

/// `(sin, cos)` of `deg` degrees, exact at multiples of 90° (where `f64` trigonometry leaves
/// a 1e-16 residue that would make a quarter turn slightly non-orthogonal).
fn rotation(deg: f64) -> (f64, f64) {
    let q = deg / 90.0;
    if q == q.round() {
        match (q as i64).rem_euclid(4) {
            0 => (0.0, 1.0),
            1 => (1.0, 0.0),
            2 => (0.0, -1.0),
            _ => (-1.0, 0.0),
        }
    } else {
        deg.to_radians().sin_cos()
    }
}

/// The eight built-in families that are opcode programs (design §3, A3), or `None`.
pub fn builtin_opcodes(formula: u32) -> Option<&'static [Opcode]> {
    use crate::formula as f;
    use Opcode::*;
    Some(match formula {
        f::MANDELBROT => &[Sqr, AddC],
        f::MULTIBROT3 => &[Store, Sqr, Mul, AddC],
        f::MULTIBROT4 => &[Sqr, Sqr, AddC],
        f::MULTIBROT5 => &[Store, Sqr, Sqr, Mul, AddC],
        f::TRICORN => &[NegY, Sqr, AddC],
        f::BURNING_SHIP => &[AbsX, AbsY, Sqr, AddC],
        f::CELTIC => &[Sqr, AbsX, AddC],
        f::BUFFALO => &[Sqr, AbsX, AbsY, AddC],
        _ => return None,
    })
}

/// A built-in family's step as a program: the eight opcode families lowered, Phoenix
/// (`z² + c − 0.5·z_prev`) and Newton (`z − (z³ − 1)/(3z²)`) written directly. Only the STEP:
/// Newton's convergence test is not part of a program.
pub fn builtin_step(formula: u32) -> Option<Program> {
    if let Some(ops) = builtin_opcodes(formula) {
        return lower_opcodes(ops).ok();
    }
    let mut b = Builder::new();
    let z = b.push(Op::Z);
    let out = match formula {
        crate::formula::PHOENIX => {
            let s = b.push(Op::Sqr(z));
            let c = b.push(Op::C);
            let t = b.push(Op::Add(s, c));
            let zp = b.push(Op::ZPrev);
            let h = b.push(Op::Scale(zp, 0.5));
            b.push(Op::Sub(t, h))
        }
        crate::formula::NEWTON => {
            let z2 = b.push(Op::Sqr(z));
            let z3 = b.push(Op::Mul(z2, z));
            let one = b.push(Op::Const(1.0, 0.0));
            let f = b.push(Op::Sub(z3, one));
            let d = b.push(Op::Scale(z2, 3.0));
            let q = b.push(Op::Div(f, d));
            b.push(Op::Sub(z, q))
        }
        // The power families: the fold, the power, the fold — as the parser reads `abs(z)^d + c`,
        // `conj(z)^d + c`, `z^d + c` and the Celtic and Buffalo folds of `z^d`.
        f => {
            use crate::formula::Shape;
            let (shape, d) = crate::formula::family(f)?;
            let base = match shape {
                Shape::BurningShip => {
                    let r = b.push(Op::AbsRe(z));
                    b.push(Op::AbsIm(r))
                }
                Shape::Tricorn => b.push(Op::Conj(z)),
                _ => z,
            };
            let p = b.push(Op::PowI(base, d));
            let w = match shape {
                Shape::Celtic => b.push(Op::AbsRe(p)),
                Shape::Buffalo => {
                    let r = b.push(Op::AbsRe(p));
                    b.push(Op::AbsIm(r))
                }
                _ => p,
            };
            let c = b.push(Op::C);
            b.push(Op::Add(w, c))
        }
    };
    b.finish(out).ok()
}

// ---------------------------------------------------------------------------------------------
// The interpreter
// ---------------------------------------------------------------------------------------------

/// A number field the interpreter runs in: [`Field`]'s arithmetic plus what a program needs
/// around it.
pub(crate) trait IrField: Field {
    /// An exact constant (an `f64` is exact in every field here).
    fn konst(v: f64, ctx: Self::Ctx) -> Self;
    /// Exact negation (a sign flip).
    fn fneg(self) -> Self;
    /// A non-ring operation on materialised operands (up to three: [`Op::DiffPow`]), or `None`
    /// where this field has none.
    fn elementary(
        _op: &Op,
        _a: &(Self, Self),
        _b: Option<&(Self, Self)>,
        _c: Option<&(Self, Self)>,
        _ctx: Self::Ctx,
    ) -> Option<(Self, Self)> {
        None
    }
    /// Whether a real value is non-zero (a `.frm` condition), where this field decides conditions
    /// at all (`f64` only).
    fn truthy(_v: &Self) -> Option<bool> {
        None
    }
}

/// What a bignum field needs beyond the ring for [`Op::is_bignum`]: division, the real functions,
/// π and a sign test, at the working precision.
pub(crate) trait Transcendental: IrField {
    fn fdiv(&self, o: &Self, ctx: Self::Ctx) -> Self;
    fn fexp(&self, ctx: Self::Ctx) -> Self;
    fn fsin(&self, ctx: Self::Ctx) -> Self;
    fn fcos(&self, ctx: Self::Ctx) -> Self;
    fn fsinh(&self, ctx: Self::Ctx) -> Self;
    fn fcosh(&self, ctx: Self::Ctx) -> Self;
    fn fsqrt(&self, ctx: Self::Ctx) -> Self;
    fn fln(&self, ctx: Self::Ctx) -> Self;
    fn fatan(&self, ctx: Self::Ctx) -> Self;
    fn fpi(ctx: Self::Ctx) -> Self;
    /// −1, 0 or +1 (a zero of either sign is 0).
    fn fsign(&self) -> i8;
}

/// The complex forms, from the real functions — the same identities as the `f64` interpreter's
/// ([`cfunc`], [`cdiv`], [`csqrt`], [`clog`], [`cpow`]), except tan and tanh as plain quotients (a
/// bignum does not overflow).
fn complex_elementary<F: Transcendental>(op: &Op, a: &(F, F), b: Option<&(F, F)>, p: F::Ctx) -> Option<(F, F)> {
    let (x, y) = a;
    let div = |a: &(F, F), b: &(F, F)| {
        let dd = b.0.fmul(&b.0, p).fadd(&b.1.fmul(&b.1, p), p);
        let re = a.0.fmul(&b.0, p).fadd(&a.1.fmul(&b.1, p), p);
        let im = a.1.fmul(&b.0, p).fsub(&a.0.fmul(&b.1, p), p);
        (re.fdiv(&dd, p), im.fdiv(&dd, p))
    };
    let exp = |a: &(F, F)| {
        let r = a.0.fexp(p);
        (r.fmul(&a.1.fcos(p), p), r.fmul(&a.1.fsin(p), p))
    };
    // The principal logarithm: (½·ln(x² + y²), arg), arg ∈ (−π, π] with a zero imaginary part
    // on the upper side (see [`Func`]). arg from atan: atan(y/x) off the imaginary axis, ±π/2 on it.
    let log = |a: &(F, F)| {
        let (x, y) = a;
        let half = F::konst(0.5, p);
        let re = x.fmul(x, p).fadd(&y.fmul(y, p), p).fln(p).fmul(&half, p);
        let arg = match (x.fsign(), y.fsign()) {
            (0, 0) => F::konst(0.0, p),
            (0, s) => {
                let h = F::fpi(p).fmul(&half, p);
                if s > 0 { h } else { h.fneg() }
            }
            (sx, sy) => {
                let t = y.fdiv(x, p).fatan(p);
                match (sx > 0, sy >= 0) {
                    (true, _) => t,
                    (false, true) => t.fadd(&F::fpi(p), p),
                    (false, false) => t.fsub(&F::fpi(p), p),
                }
            }
        };
        (re, arg)
    };
    let sin = || (x.fsin(p).fmul(&y.fcosh(p), p), x.fcos(p).fmul(&y.fsinh(p), p));
    let cos = || (x.fcos(p).fmul(&y.fcosh(p), p), x.fsin(p).fmul(&y.fsinh(p), p).fneg());
    let sinh = || (x.fsinh(p).fmul(&y.fcos(p), p), x.fcosh(p).fmul(&y.fsin(p), p));
    let cosh = || (x.fcosh(p).fmul(&y.fcos(p), p), x.fsinh(p).fmul(&y.fsin(p), p));
    Some(match *op {
        Op::Div(..) => div(a, b?),
        Op::Pow(..) => {
            let k = b?;
            if x.fsign() == 0 && y.fsign() == 0 {
                (F::konst(0.0, p), F::konst(0.0, p))
            } else {
                let l = log(a);
                let e = (k.0.fmul(&l.0, p).fsub(&k.1.fmul(&l.1, p), p), k.0.fmul(&l.1, p).fadd(&k.1.fmul(&l.0, p), p));
                exp(&e)
            }
        }
        Op::Func(Func::Exp, _) => exp(a),
        Op::Func(Func::Log, _) => log(a),
        // The principal square root, as [`csqrt`]: t = sqrt((|x| + |z|)/2), then (t, y/2t) for
        // x ≥ 0, else (|y|/2t, ±t) — the sign of y, a zero counting as positive.
        Op::Func(Func::Sqrt, _) => {
            if x.fsign() == 0 && y.fsign() == 0 {
                (F::konst(0.0, p), F::konst(0.0, p))
            } else {
                let m = x.fmul(x, p).fadd(&y.fmul(y, p), p).fsqrt(p);
                let t = x.fabs().fadd(&m, p).fmul(&F::konst(0.5, p), p).fsqrt(p);
                let two_t = t.fdouble();
                if x.fsign() >= 0 {
                    (t, y.fdiv(&two_t, p))
                } else if y.fsign() >= 0 {
                    (y.fabs().fdiv(&two_t, p), t)
                } else {
                    (y.fabs().fdiv(&two_t, p), t.fneg())
                }
            }
        }
        Op::Func(Func::Sin, _) => sin(),
        Op::Func(Func::Cos, _) => cos(),
        Op::Func(Func::Tan, _) => div(&sin(), &cos()),
        Op::Func(Func::Sinh, _) => sinh(),
        Op::Func(Func::Cosh, _) => cosh(),
        Op::Func(Func::Tanh, _) => div(&sinh(), &cosh()),
        _ => return None,
    })
}

thread_local! {
    /// astro-float's cache of π, e and friends for its transcendental functions, built once per
    /// thread (a reference build runs on a worker; each keeps its own).
    static ASTRO_CONSTS: std::cell::RefCell<astro_float::Consts> =
        std::cell::RefCell::new(astro_float::Consts::new().expect("astro-float constant cache"));
}

impl Transcendental for BigFloat {
    fn fdiv(&self, o: &BigFloat, p: usize) -> BigFloat {
        self.div(o, p, crate::RM)
    }
    fn fexp(&self, p: usize) -> BigFloat {
        ASTRO_CONSTS.with(|c| self.exp(p, crate::RM, &mut c.borrow_mut()))
    }
    fn fsin(&self, p: usize) -> BigFloat {
        ASTRO_CONSTS.with(|c| self.sin(p, crate::RM, &mut c.borrow_mut()))
    }
    fn fcos(&self, p: usize) -> BigFloat {
        ASTRO_CONSTS.with(|c| self.cos(p, crate::RM, &mut c.borrow_mut()))
    }
    fn fsinh(&self, p: usize) -> BigFloat {
        ASTRO_CONSTS.with(|c| self.sinh(p, crate::RM, &mut c.borrow_mut()))
    }
    fn fcosh(&self, p: usize) -> BigFloat {
        ASTRO_CONSTS.with(|c| self.cosh(p, crate::RM, &mut c.borrow_mut()))
    }
    fn fsqrt(&self, p: usize) -> BigFloat {
        self.sqrt(p, crate::RM)
    }
    fn fln(&self, p: usize) -> BigFloat {
        ASTRO_CONSTS.with(|c| self.ln(p, crate::RM, &mut c.borrow_mut()))
    }
    fn fatan(&self, p: usize) -> BigFloat {
        ASTRO_CONSTS.with(|c| self.atan(p, crate::RM, &mut c.borrow_mut()))
    }
    fn fpi(p: usize) -> BigFloat {
        ASTRO_CONSTS.with(|c| c.borrow_mut().pi(p, crate::RM))
    }
    fn fsign(&self) -> i8 {
        if self.is_zero() {
            0
        } else if self.is_negative() {
            -1
        } else {
            1
        }
    }
}

#[cfg(feature = "rug")]
impl Transcendental for rug::Float {
    fn fdiv(&self, o: &rug::Float, p: u32) -> rug::Float {
        rug::Float::with_val_round(p, self / o, rug::float::Round::Zero).0
    }
    fn fexp(&self, p: u32) -> rug::Float {
        rug::Float::with_val_round(p, self.exp_ref(), rug::float::Round::Zero).0
    }
    fn fsin(&self, p: u32) -> rug::Float {
        rug::Float::with_val_round(p, self.sin_ref(), rug::float::Round::Zero).0
    }
    fn fcos(&self, p: u32) -> rug::Float {
        rug::Float::with_val_round(p, self.cos_ref(), rug::float::Round::Zero).0
    }
    fn fsinh(&self, p: u32) -> rug::Float {
        rug::Float::with_val_round(p, self.sinh_ref(), rug::float::Round::Zero).0
    }
    fn fcosh(&self, p: u32) -> rug::Float {
        rug::Float::with_val_round(p, self.cosh_ref(), rug::float::Round::Zero).0
    }
    fn fsqrt(&self, p: u32) -> rug::Float {
        rug::Float::with_val_round(p, self.sqrt_ref(), rug::float::Round::Zero).0
    }
    fn fln(&self, p: u32) -> rug::Float {
        rug::Float::with_val_round(p, self.ln_ref(), rug::float::Round::Zero).0
    }
    fn fatan(&self, p: u32) -> rug::Float {
        rug::Float::with_val_round(p, self.atan_ref(), rug::float::Round::Zero).0
    }
    fn fpi(p: u32) -> rug::Float {
        rug::Float::with_val_round(p, rug::float::Constant::Pi, rug::float::Round::Zero).0
    }
    fn fsign(&self) -> i8 {
        if self.is_zero() {
            0
        } else if self.is_sign_negative() {
            -1
        } else {
            1
        }
    }
}

impl IrField for f64 {
    fn konst(v: f64, _: ()) -> f64 {
        v
    }
    fn fneg(self) -> f64 {
        -self
    }
    fn elementary(op: &Op, a: &(f64, f64), b: Option<&(f64, f64)>, c: Option<&(f64, f64)>, _: ()) -> Option<(f64, f64)> {
        Some(match *op {
            Op::Div(..) => cdiv(*a, *b?),
            Op::Pow(..) => cpow(*a, *b?),
            Op::Func(f, _) => cfunc(f, *a),
            Op::DiffLog(..) => log_diff(*a, *b?),
            Op::DiffSqrt(..) => sqrt_diff(*a, *b?),
            Op::DiffPow(..) => pow_diff(*a, *b?, *c?),
            Op::DiffAbsRe(..) => {
                let p = *b?;
                (diffabs(a.0, p.0), p.1)
            }
            Op::DiffAbsIm(..) => {
                let p = *b?;
                (p.0, diffabs(a.1, p.1))
            }
            Op::DiffTanh(..) => tanh_diff(*a, *b?),
            // tan a = −i·tanh(i·a)
            Op::DiffTan(..) => {
                let p = *b?;
                let d = tanh_diff((-a.1, a.0), (-p.1, p.0));
                (d.1, -d.0)
            }
            // Fractint's conditions: real parts only, 1 or 0.
            Op::Cmp(cmp, ..) => (f64::from(u8::from(cmp.holds(a.0, b?.0))), 0.0),
            Op::And(..) => (f64::from(u8::from(a.0 != 0.0 && b?.0 != 0.0)), 0.0),
            Op::Or(..) => (f64::from(u8::from(a.0 != 0.0 || b?.0 != 0.0)), 0.0),
            Op::Round(r, _) => (r.apply(a.0), r.apply(a.1)),
            _ => return None,
        })
    }
    fn truthy(v: &f64) -> Option<bool> {
        Some(*v != 0.0)
    }
}

/// `sech a = 2·e^−s / (1 + e^−2s)` with `s = ±a`, `Re s ≥ 0`: `e^−s` is at most 1 in size, so
/// nothing overflows, and it tends to 0 (relative accuracy intact) where `cosh a` overflows.
/// The shader's `cf_sech`.
fn csech(a: (f64, f64)) -> (f64, f64) {
    let (x, y) = if a.0 < 0.0 { (-a.0, -a.1) } else { a };
    let r = (-x).exp();
    let e = (r * y.cos(), -r * y.sin());
    let e2 = cmul64(e, e);
    cdiv((2.0 * e.0, 2.0 * e.1), (1.0 + e2.0, e2.1))
}

/// Below this `|Re p|`, [`tanh_diff`] takes the product form; above it, the plain difference.
/// `sinh p` stays finite in `f32` to ~89.
pub const TANH_DIFF_SPLIT: f64 = 40.0;

/// `tanh(b + p) − tanh(b)` as the perturbed `tanh` needs it, the shader's `cf_tanh_diff` branch for
/// branch. For a small `p` (a deep pixel's offset) it is `sinh(p)·sech(b)·sech(b + p)`: accurate
/// RELATIVE to itself, where the difference of the two `tanh`s would cancel (they round to the
/// same ±1 once the real parts pass ~19) — and with `sech` overflow-free, where
/// `sinh(p) / (cosh(b)·cosh(b + p))` gave `f32` inf/inf. For a large `p` (an escaping orbit),
/// `sinh(p)` itself overflows and meets a `sech` of 0 — so there the plain difference, which
/// cannot cancel much when the arguments are that far apart.
pub(crate) fn tanh_diff(b: (f64, f64), p: (f64, f64)) -> (f64, f64) {
    let w = (b.0 + p.0, b.1 + p.1);
    if p.0.abs() >= TANH_DIFF_SPLIT {
        let (t, u) = (cfunc(Func::Tanh, w), cfunc(Func::Tanh, b));
        return (t.0 - u.0, t.1 - u.1);
    }
    cmul64(cmul64(cfunc(Func::SinhSmall, p), csech(b)), csech(w))
}

/// Below this `|p/b|`, [`log_diff`] and [`pow_diff`] take their relative-accuracy forms; above it,
/// the plain difference, which cannot cancel much when `b + p` is that far from `b`.
pub const LOG_DIFF_SPLIT: f64 = 0.5;

/// `log(1 + u)` accurate RELATIVE to a small `u`: `(½·ln_1p(2·Re u + |u|²), atan2(Im u, 1 + Re u))`.
fn clog1p(u: (f64, f64)) -> (f64, f64) {
    (0.5 * (2.0 * u.0 + u.0 * u.0 + u.1 * u.1).ln_1p(), u.1.atan2(1.0 + u.0))
}

/// Whether `b` and `w` sit on opposite sides of the principal branch cut (the negative real axis)
/// — `+1` from below to above, `−1` from above to below, `0` otherwise. For `w` near `b` (the
/// relative-accuracy forms only): the crossing then needs both in the left half-plane, and the
/// signs of the imaginary parts decide it, a zero counting as above (see [`Func`]).
fn cut_crossing(b: (f64, f64), w: (f64, f64)) -> i8 {
    if b.0 >= 0.0 || w.0 >= 0.0 {
        return 0;
    }
    match (b.1 >= 0.0, w.1 >= 0.0) {
        (true, false) => -1,
        (false, true) => 1,
        _ => 0,
    }
}

/// `Log(b + p) − Log(b)` as the perturbed `log` needs it, the shader's `cf_log_diff` branch for
/// branch. For a small `p/b`, `log1p(p/b)` — accurate relative to itself, where two logs of nearly
/// equal values would cancel — plus `2πi·n` for a crossing of the branch cut, which `log1p` cannot
/// see: from above to below `Arg` drops by 2π (`n = −1`), from below to above it rises (`n = +1`).
/// Else the plain difference.
pub(crate) fn log_diff(b: (f64, f64), p: (f64, f64)) -> (f64, f64) {
    let w = (b.0 + p.0, b.1 + p.1);
    let u = cdiv(p, b);
    if !(u.0.hypot(u.1) < LOG_DIFF_SPLIT) {
        let (lw, lb) = (clog(w), clog(b));
        return (lw.0 - lb.0, lw.1 - lb.1);
    }
    let l = clog1p(u);
    (l.0, l.1 + std::f64::consts::TAU * cut_crossing(b, w) as f64)
}

/// `sqrt(b + p) − sqrt(b)` (principal) as the perturbed `sqrt` needs it, the shader's
/// `cf_sqrt_diff`: `p / (sqrt(w) + sqrt(b))` while the two roots point the same way (the sum cannot
/// cancel), else — across the branch cut, where `sqrt(w) ≈ −sqrt(b)` — the plain difference, which
/// then cannot cancel. `(w − b) = (√w − √b)(√w + √b)` holds for any pair of roots, so both are exact.
pub(crate) fn sqrt_diff(b: (f64, f64), p: (f64, f64)) -> (f64, f64) {
    let (sw, sb) = (csqrt((b.0 + p.0, b.1 + p.1)), csqrt(b));
    let sum = (sw.0 + sb.0, sw.1 + sb.1);
    let dif = (sw.0 - sb.0, sw.1 - sb.1);
    if sum.0 * sum.0 + sum.1 * sum.1 >= dif.0 * dif.0 + dif.1 * dif.1 {
        if sum == (0.0, 0.0) {
            return (0.0, 0.0);
        }
        cdiv(p, sum)
    } else {
        dif
    }
}

/// `(b + p)^k − b^k` (principal power, `0^k = 0`) as the perturbed power needs it, the shader's
/// `cf_pow_diff`: `b^k·expm1(k·(Log w − Log b))` for a small `p/b`, with [`log_diff`] carrying the
/// branch cut; else the plain difference. At `b = 0` — a reference at `Z₀ = 0`, as after every
/// rebase in the parameter plane — it is `w^k` itself, which the product form would reach as
/// `0·∞`.
pub(crate) fn pow_diff(b: (f64, f64), p: (f64, f64), k: (f64, f64)) -> (f64, f64) {
    let w = (b.0 + p.0, b.1 + p.1);
    if b == (0.0, 0.0) {
        return cpow(w, k);
    }
    let u = cdiv(p, b);
    if !(u.0.hypot(u.1) < LOG_DIFF_SPLIT) {
        let (pw, pb) = (cpow(w, k), cpow(b, k));
        return (pw.0 - pb.0, pw.1 - pb.1);
    }
    cmul64(cpow(b, k), cfunc(Func::Expm1, cmul64(k, log_diff(b, p))))
}

/// `|c + d| − |c|` without cancellation (Kalles Fraktaler's "diffabs"): exactly `±d` while `c` and
/// `c + d` share a sign, `±(2c + d)` across the fold. The shader's `df_diffabs`, branch for branch.
pub(crate) fn diffabs(c: f64, d: f64) -> f64 {
    let cd = c + d;
    if c >= 0.0 {
        if cd >= 0.0 {
            d
        } else {
            -(2.0 * c + d)
        }
    } else if cd > 0.0 {
        2.0 * c + d
    } else {
        -d
    }
}

impl IrField for BigFloat {
    fn konst(v: f64, p: usize) -> BigFloat {
        BigFloat::from_f64(v, p)
    }
    fn fneg(mut self) -> BigFloat {
        self.inv_sign();
        self
    }
    fn elementary(
        op: &Op,
        a: &(BigFloat, BigFloat),
        b: Option<&(BigFloat, BigFloat)>,
        _: Option<&(BigFloat, BigFloat)>,
        p: usize,
    ) -> Option<(BigFloat, BigFloat)> {
        complex_elementary(op, a, b, p)
    }
}

#[cfg(feature = "rug")]
impl IrField for rug::Float {
    fn konst(v: f64, ctx: u32) -> rug::Float {
        <rug::Float as RefBackend>::from_f64(v, ctx)
    }
    fn fneg(self) -> rug::Float {
        -self
    }
    fn elementary(
        op: &Op,
        a: &(rug::Float, rug::Float),
        b: Option<&(rug::Float, rug::Float)>,
        _: Option<&(rug::Float, rug::Float)>,
        p: u32,
    ) -> Option<(rug::Float, rug::Float)> {
        complex_elementary(op, a, b, p)
    }
}

/// `a / b` in the order Newton's hand-written step uses: `dd = bx² + by²`,
/// `((ax·bx + ay·by)/dd, (ay·bx − ax·by)/dd)`.
fn cdiv(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let dd = b.0 * b.0 + b.1 * b.1;
    ((a.0 * b.0 + a.1 * b.1) / dd, (a.1 * b.0 - a.0 * b.1) / dd)
}

fn cmul64(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

fn cexp(a: (f64, f64)) -> (f64, f64) {
    let r = a.0.exp();
    let (s, c) = a.1.sin_cos();
    (r * c, r * s)
}

/// The principal logarithm (branch cut on the negative real axis). `y + 0.0` turns a −0 into +0,
/// so the cut itself is on the upper side as in every field (see [`Func`]): IEEE's
/// `atan2(−0, −1)` is −π, the bignum fields' is π.
fn clog(a: (f64, f64)) -> (f64, f64) {
    (a.0.hypot(a.1).ln(), (a.1 + 0.0).atan2(a.0))
}

/// `0^b = 0` (Fractint's convention), else `exp(b·log a)`.
fn cpow(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    if a.0 == 0.0 && a.1 == 0.0 {
        return (0.0, 0.0);
    }
    cexp(cmul64(b, clog(a)))
}

/// The principal square root, without cancellation on either half-plane.
fn csqrt(a: (f64, f64)) -> (f64, f64) {
    let (x, y) = a;
    if x == 0.0 && y == 0.0 {
        return (0.0, y);
    }
    let t = ((x.abs() + x.hypot(y)) * 0.5).sqrt();
    if x >= 0.0 {
        (t, y / (2.0 * t))
    } else {
        // `y + 0.0`: a −0 on the cut takes the upper side's root, as in every field.
        (y.abs() / (2.0 * t), t.copysign(y + 0.0))
    }
}

fn cfunc(f: Func, a: (f64, f64)) -> (f64, f64) {
    let (x, y) = a;
    match f {
        Func::Exp => cexp(a),
        Func::Log => clog(a),
        Func::Sqrt => csqrt(a),
        Func::Sin => (x.sin() * y.cosh(), x.cos() * y.sinh()),
        Func::Cos => (x.cos() * y.cosh(), -(x.sin() * y.sinh())),
        // tan and tanh as double-angle forms divided through by cosh 2y (cosh 2x): sin/cos
        // (sinh/cosh) is inf/inf = NaN once the imaginary (real) part passes ~355, where the value
        // is ±i (±1); these tend cleanly to it. The same forms as the GPU's.
        Func::Tan => {
            let ch = (2.0 * y).cosh();
            let d = (2.0 * x).cos() / ch + 1.0;
            ((2.0 * x).sin() / ch / d, (2.0 * y).tanh() / d)
        }
        Func::Sinh => (x.sinh() * y.cos(), x.cosh() * y.sin()),
        Func::Cosh => (x.cosh() * y.cos(), x.sinh() * y.sin()),
        Func::Tanh => {
            let ch = (2.0 * x).cosh();
            let d = (2.0 * y).cos() / ch + 1.0;
            ((2.0 * x).tanh() / d, (2.0 * y).sin() / ch / d)
        }
        // `f64`'s own `sin` and `sinh` are already accurate relative to a small argument.
        Func::SinSmall => cfunc(Func::Sin, a),
        Func::SinhSmall => cfunc(Func::Sinh, a),
        // e^x·cos y − 1 = expm1(x)·cos y − 2·sin²(y/2), with no difference of near-equal terms.
        Func::Expm1 => {
            let s = (0.5 * y).sin();
            (x.exp_m1() * y.cos() - 2.0 * s * s, x.exp() * y.sin())
        }
    }
}

/// A scalar: a slot in the machine's pool, possibly negated.
#[derive(Clone, Copy, Debug)]
struct S {
    slot: u32,
    neg: bool,
}

impl S {
    fn pos(slot: usize) -> S {
        S { slot: slot as u32, neg: false }
    }
    fn flip(self) -> S {
        S { slot: self.slot, neg: !self.neg }
    }
}

/// A complex value: two scalars.
#[derive(Clone, Copy, Debug)]
struct Cs {
    re: S,
    im: S,
}

// The pool's fixed layout; per-program constants follow, then each step's temporaries.
const Z_RE: usize = 0;
const Z_IM: usize = 1;
const ZP_RE: usize = 2;
const ZP_IM: usize = 3;
const C_RE: usize = 4;
const C_IM: usize = 5;
const ZERO: usize = 6;
// A perturbed program's δz and δc (zero for an ordinary one, which never reads them).
const DZ_RE: usize = 7;
const DZ_IM: usize = 8;
const DC_RE: usize = 9;
const DC_IM: usize = 10;
// A `.frm`-style formula's persistent variables, two slots each, before the constants.
const VAR_BASE: usize = 11;

/// Runs a [`Formula`] in one field. The state (`z`, `z_prev`, `c`) and every constant live in the
/// fixed front of `pool`; a step appends its temporaries and the next step drops them.
struct Machine<'f, F: IrField> {
    formula: &'f Formula,
    pool: Vec<F>,
    fixed: usize,
    /// Per phase, per instruction: the pre-converted constant (`Const`, `Param`; `Scale`'s factor
    /// in `re`).
    leaves: Vec<Vec<Option<Cs>>>,
    vals: Vec<Cs>,
    /// The formula reads `z_prev`, so each step shifts `z` into it.
    shift: bool,
    /// The bailout condition the last step left, for a formula with one ([`Program::cond`]).
    go: Option<bool>,
    ctx: F::Ctx,
}

impl<'f, F: IrField> Machine<'f, F> {
    fn new(
        formula: &'f Formula,
        z: (F, F),
        zp: (F, F),
        c: (F, F),
        params: &[(f64, f64)],
        max_iter: f64,
        ctx: F::Ctx,
    ) -> Result<Self, IrError> {
        let zero = || F::konst(0.0, ctx);
        let mut pool = vec![z.0, z.1, zp.0, zp.1, c.0, c.1, zero(), zero(), zero(), zero(), zero()];
        for _ in 0..2 * formula.vars as usize {
            pool.push(zero());
        }
        let push = |v: f64, pool: &mut Vec<F>| {
            pool.push(F::konst(v, ctx));
            S::pos(pool.len() - 1)
        };
        // The phases, then the init section ([`Formula::program`]'s numbering; an absent one has
        // no leaves).
        let n = formula.phases.len();
        let programs = (0..n + 1).map(|k| formula.phases.get(k).or(formula.init.as_ref()));
        let mut leaves = Vec::with_capacity(n + 1);
        for prog in programs {
            let Some(prog) = prog else {
                leaves.push(Vec::new());
                continue;
            };
            let mut lv = Vec::with_capacity(prog.insts.len());
            for op in &prog.insts {
                lv.push(match *op {
                    Op::Const(re, im) => Some(Cs { re: push(re, &mut pool), im: push(im, &mut pool) }),
                    Op::Param(i) => {
                        let &(re, im) = params
                            .get(i as usize)
                            .ok_or(IrError::MissingParam { index: i, supplied: params.len() })?;
                        Some(Cs { re: push(re, &mut pool), im: push(im, &mut pool) })
                    }
                    Op::MaxIter => Some(Cs { re: push(max_iter, &mut pool), im: push(0.0, &mut pool) }),
                    Op::Scale(_, k) => {
                        let s = push(k, &mut pool);
                        Some(Cs { re: s, im: s })
                    }
                    _ => None,
                });
            }
            leaves.push(lv);
        }
        let fixed = pool.len();
        let shift = formula.reads_zprev();
        Ok(Machine { formula, pool, fixed, leaves, vals: Vec::new(), shift, go: None, ctx })
    }

    fn push(&mut self, v: F) -> S {
        self.pool.push(v);
        S::pos(self.pool.len() - 1)
    }

    fn at(&self, s: S) -> &F {
        &self.pool[s.slot as usize]
    }

    fn mul(&mut self, a: S, b: S) -> S {
        let v = self.at(a).fmul(self.at(b), self.ctx);
        S { neg: a.neg ^ b.neg, ..self.push(v) }
    }

    /// `a + b` with the signs folded in: `(−a) + b` is computed as `b − a`, never as a sum with a
    /// negated operand (module rule 2).
    fn add(&mut self, a: S, b: S) -> S {
        let (x, y) = (self.at(a), self.at(b));
        let (v, neg) = match (a.neg, b.neg) {
            (false, false) => (x.fadd(y, self.ctx), false),
            (true, true) => (x.fadd(y, self.ctx), true),
            (true, false) => (y.fsub(x, self.ctx), false),
            (false, true) => (x.fsub(y, self.ctx), false),
        };
        S { neg, ..self.push(v) }
    }

    fn sub(&mut self, a: S, b: S) -> S {
        self.add(a, b.flip())
    }

    fn double(&mut self, a: S) -> S {
        let v = self.at(a).fdouble();
        S { neg: a.neg, ..self.push(v) }
    }

    fn abs(&mut self, a: S) -> S {
        let v = self.at(a).fabs();
        self.push(v)
    }

    /// `ax·bx − ay·by, ax·by + ay·bx` — the core's `cmul`.
    fn cmul(&mut self, a: Cs, b: Cs) -> Cs {
        let (xx, yy) = (self.mul(a.re, b.re), self.mul(a.im, b.im));
        let re = self.sub(xx, yy);
        let (xy, yx) = (self.mul(a.re, b.im), self.mul(a.im, b.re));
        let im = self.add(xy, yx);
        Cs { re, im }
    }

    /// `x·x − y·y, 2·(x·y)` — the core's `csqr`.
    fn csqr(&mut self, a: Cs) -> Cs {
        let (xx, yy) = (self.mul(a.re, a.re), self.mul(a.im, a.im));
        let re = self.sub(xx, yy);
        let xy = self.mul(a.re, a.im);
        let im = self.double(xy);
        Cs { re, im }
    }

    fn materialize(&self, s: S) -> F {
        let v = self.at(s).clone();
        if s.neg {
            v.fneg()
        } else {
            v
        }
    }

    /// Evaluate program `k` ([`Formula::program`]: a phase, the init section or the bailout) on the
    /// current state; the result's scalars are in the pool.
    fn eval(&mut self, k: usize) -> Result<Cs, IrError> {
        let formula = self.formula;
        let prog = formula.program(k);
        self.pool.truncate(self.fixed);
        self.vals.clear();
        let zero = S::pos(ZERO);
        for (i, op) in prog.insts.iter().enumerate() {
            let g = |v: Val| self.vals[v.index()];
            let v = match *op {
                Op::Z => Cs { re: S::pos(Z_RE), im: S::pos(Z_IM) },
                Op::ZPrev => Cs { re: S::pos(ZP_RE), im: S::pos(ZP_IM) },
                Op::C => Cs { re: S::pos(C_RE), im: S::pos(C_IM) },
                Op::Delta => Cs { re: S::pos(DZ_RE), im: S::pos(DZ_IM) },
                Op::DeltaC => Cs { re: S::pos(DC_RE), im: S::pos(DC_IM) },
                Op::Var(j) => {
                    let s = VAR_BASE + 2 * j as usize;
                    Cs { re: S::pos(s), im: S::pos(s + 1) }
                }
                Op::Cmp(_, a, b) | Op::And(a, b) | Op::Or(a, b) => {
                    let (a, b) = (g(a), g(b));
                    let a = (self.materialize(a.re), self.materialize(a.im));
                    let b = (self.materialize(b.re), self.materialize(b.im));
                    let (re, im) = F::elementary(op, &a, Some(&b), None, self.ctx).ok_or(IrError::NotBignum)?;
                    Cs { re: self.push(re), im: self.push(im) }
                }
                Op::Select(c, a, b) => {
                    let (c, a, b) = (g(c), g(a), g(b));
                    let cond = self.materialize(c.re);
                    match F::truthy(&cond) {
                        Some(true) => a,
                        Some(false) => b,
                        None => return Err(IrError::NotBignum),
                    }
                }
                Op::Const(..) | Op::Param(_) | Op::MaxIter => self.leaves[k][i].expect("converted in new()"),
                Op::Add(a, b) => {
                    let (a, b) = (g(a), g(b));
                    Cs { re: self.add(a.re, b.re), im: self.add(a.im, b.im) }
                }
                Op::Sub(a, b) => {
                    let (a, b) = (g(a), g(b));
                    Cs { re: self.sub(a.re, b.re), im: self.sub(a.im, b.im) }
                }
                Op::Mul(a, b) => {
                    let (a, b) = (g(a), g(b));
                    self.cmul(a, b)
                }
                Op::Sqr(a) => {
                    let a = g(a);
                    self.csqr(a)
                }
                Op::PowI(a, n) => {
                    let a = g(a);
                    let mut r = a;
                    for bit in (0..31 - n.leading_zeros()).rev() {
                        r = self.csqr(r);
                        if (n >> bit) & 1 == 1 {
                            r = self.cmul(r, a);
                        }
                    }
                    r
                }
                Op::Scale(a, _) => {
                    let (a, f) = (g(a), self.leaves[k][i].expect("converted in new()").re);
                    Cs { re: self.mul(a.re, f), im: self.mul(a.im, f) }
                }
                Op::Neg(a) => {
                    let a = g(a);
                    Cs { re: a.re.flip(), im: a.im.flip() }
                }
                Op::Conj(a) => {
                    let a = g(a);
                    Cs { re: a.re, im: a.im.flip() }
                }
                Op::AbsRe(a) => {
                    let a = g(a);
                    Cs { re: self.abs(a.re), im: a.im }
                }
                Op::AbsIm(a) => {
                    let a = g(a);
                    Cs { re: a.re, im: self.abs(a.im) }
                }
                Op::Re(a) => Cs { re: g(a).re, im: zero },
                Op::Im(a) => Cs { re: g(a).im, im: zero },
                Op::Norm(a) => {
                    let a = g(a);
                    let (xx, yy) = (self.mul(a.re, a.re), self.mul(a.im, a.im));
                    Cs { re: self.add(xx, yy), im: zero }
                }
                Op::Div(a, b)
                | Op::Pow(a, b)
                | Op::DiffAbsRe(a, b)
                | Op::DiffAbsIm(a, b)
                | Op::DiffTanh(a, b)
                | Op::DiffTan(a, b)
                | Op::DiffLog(a, b)
                | Op::DiffSqrt(a, b) => {
                    let (a, b) = (g(a), g(b));
                    let a = (self.materialize(a.re), self.materialize(a.im));
                    let b = (self.materialize(b.re), self.materialize(b.im));
                    let (re, im) = F::elementary(op, &a, Some(&b), None, self.ctx).ok_or(IrError::NotBignum)?;
                    Cs { re: self.push(re), im: self.push(im) }
                }
                Op::DiffPow(a, b, k) => {
                    let (a, b, k) = (g(a), g(b), g(k));
                    let a = (self.materialize(a.re), self.materialize(a.im));
                    let b = (self.materialize(b.re), self.materialize(b.im));
                    let k = (self.materialize(k.re), self.materialize(k.im));
                    let (re, im) = F::elementary(op, &a, Some(&b), Some(&k), self.ctx).ok_or(IrError::NotBignum)?;
                    Cs { re: self.push(re), im: self.push(im) }
                }
                Op::Func(_, a) | Op::Round(_, a) => {
                    let a = g(a);
                    let a = (self.materialize(a.re), self.materialize(a.im));
                    let (re, im) = F::elementary(op, &a, None, None, self.ctx).ok_or(IrError::NotBignum)?;
                    Cs { re: self.push(re), im: self.push(im) }
                }
            };
            self.vals.push(v);
        }
        Ok(self.vals[prog.out.index()])
    }

    /// Make `out` the new `z` (and the old `z` the new `z_prev` if the formula reads it).
    /// Temporaries are moved out of the pool, not cloned — the highest slot first, so the other
    /// index stays valid.
    fn commit(&mut self, out: Cs) {
        let (a, b) = (out.re.slot as usize, out.im.slot as usize);
        let fixed = self.fixed;
        let (va, vb) = match (a >= fixed, b >= fixed) {
            (true, true) if a > b => {
                let va = self.pool.swap_remove(a);
                (va, self.pool.swap_remove(b))
            }
            (true, true) if a < b => {
                let vb = self.pool.swap_remove(b);
                (self.pool.swap_remove(a), vb)
            }
            (true, false) => {
                let vb = self.pool[b].clone();
                (self.pool.swap_remove(a), vb)
            }
            (false, true) => {
                let va = self.pool[a].clone();
                (va, self.pool.swap_remove(b))
            }
            _ => (self.pool[a].clone(), self.pool[b].clone()),
        };
        let va = if out.re.neg { va.fneg() } else { va };
        let vb = if out.im.neg { vb.fneg() } else { vb };
        let old_re = std::mem::replace(&mut self.pool[Z_RE], va);
        let old_im = std::mem::replace(&mut self.pool[Z_IM], vb);
        if self.shift {
            self.pool[ZP_RE] = old_re;
            self.pool[ZP_IM] = old_im;
        }
    }

    /// One iteration `n` (0-based): phase `n mod len`.
    fn step(&mut self, n: usize) -> Result<(), IrError> {
        self.run(n % self.formula.phases.len())
    }

    /// Evaluate program `k` and commit it: its bailout condition and persistent variables (read
    /// before anything moves, as `commit` moves temporaries), then `z`.
    fn run(&mut self, k: usize) -> Result<(), IrError> {
        let out = self.eval(k)?;
        if let Some(c) = self.formula.program(k).cond {
            let re = self.materialize(self.vals[c.index()].re);
            self.go = Some(F::truthy(&re).ok_or(IrError::NotBignum)?);
        }
        let sets: Vec<(usize, F, F)> = self
            .formula
            .program(k)
            .vars
            .iter()
            .map(|&(j, v)| {
                let cs = self.vals[v.index()];
                (VAR_BASE + 2 * j as usize, self.materialize(cs.re), self.materialize(cs.im))
            })
            .collect();
        self.commit(out);
        for (s, re, im) in sets {
            self.pool[s] = re;
            self.pool[s + 1] = im;
        }
        Ok(())
    }

    /// The init section, if the formula has one: from `z = z₀` and every variable 0, the starting
    /// `z` and variables. `z_prev` stays 0.
    fn init(&mut self) -> Result<(), IrError> {
        if self.formula.init.is_none() {
            return Ok(());
        }
        let shift = std::mem::replace(&mut self.shift, false);
        let r = self.run(self.formula.phases.len());
        self.shift = shift;
        r
    }

    /// Whether to go on iterating by the formula's bailout, as the last step decided it (`None`
    /// without one: the caller tests the escape radius).
    fn keep_going(&self) -> Option<bool> {
        self.go
    }
}

fn check_params(formula: &Formula, params: &[(f64, f64)]) -> Result<(), IrError> {
    let need = formula.param_count();
    if params.len() < need {
        return Err(IrError::MissingParam { index: (need - 1) as u16, supplied: params.len() });
    }
    Ok(())
}

/// One `f64` step of `prog`: `z' = prog(z, c, z_prev)`. A lone step has no iteration cap:
/// `maxit` reads 0.
pub fn step_f64(
    prog: &Program,
    z: (f64, f64),
    c: (f64, f64),
    zprev: (f64, f64),
    params: &[(f64, f64)],
) -> Result<(f64, f64), IrError> {
    let formula = Formula::single(prog.clone());
    check_params(&formula, params)?;
    let mut m = Machine::new(&formula, z, zprev, c, params, 0.0, ())?;
    m.step(0)?;
    Ok((m.pool[Z_RE], m.pool[Z_IM]))
}

/// One `f64` step of a PERTURBED program ([`perturb`]): `δz' = prog(Z, C, δz, δc)` with `Z`, `C`
/// the reference's iterate and constant.
pub fn step_perturbed_f64(
    prog: &Program,
    z: (f64, f64),
    c: (f64, f64),
    dz: (f64, f64),
    dc: (f64, f64),
    params: &[(f64, f64)],
) -> Result<(f64, f64), IrError> {
    let formula = Formula::single(prog.clone());
    check_params(&formula, params)?;
    let mut m = Machine::new(&formula, z, (0.0, 0.0), c, params, 0.0, ())?;
    m.pool[DZ_RE] = dz.0;
    m.pool[DZ_IM] = dz.1;
    m.pool[DC_RE] = dc.0;
    m.pool[DC_IM] = dc.1;
    m.step(0)?;
    Ok((m.pool[Z_RE], m.pool[Z_IM]))
}

/// The `f64` orbit, with [`crate::orbit_points`]'s contract: `z₀` first, then each iterate until
/// `|z|² > bailout2` or `max_points` steps. `z_prev` starts at zero; `maxit` is `max_points`.
pub fn orbit_points(
    formula: &Formula,
    z0: (f64, f64),
    c: (f64, f64),
    params: &[(f64, f64)],
    max_points: usize,
    bailout2: f64,
) -> Result<Vec<(f64, f64)>, IrError> {
    check_params(formula, params)?;
    let mut m = Machine::new(formula, z0, (0.0, 0.0), c, params, max_points as f64, ())?;
    m.init()?;
    let mut pts = Vec::with_capacity(max_points.min(1024) + 1);
    pts.push((m.pool[Z_RE], m.pool[Z_IM]));
    for n in 0..max_points {
        m.step(n)?;
        let (x, y) = (m.pool[Z_RE], m.pool[Z_IM]);
        pts.push((x, y));
        // A formula's own bailout replaces the escape radius (Fractint's: iterate while it holds).
        match m.keep_going() {
            Some(go) if !go => break,
            Some(_) => {}
            None if x * x + y * y > bailout2 => break,
            None => {}
        }
    }
    Ok(pts)
}


/// A reference orbit of `formula` in the session's bignum backend, with
/// [`crate::reference_orbit_t`]'s contract: the same samples (`Z₀` split exactly, then
/// [`pack_sample`] per step), the same escape test and the same [`OrbitTail`].
#[allow(clippy::too_many_arguments)]
pub fn reference_orbit(
    formula: &Formula,
    z0x: &BigFloat,
    z0y: &BigFloat,
    cx: &BigFloat,
    cy: &BigFloat,
    params: &[(f64, f64)],
    max_iter: u32,
    p: usize,
) -> Result<(Vec<[f32; 4]>, u32, OrbitTail), IrError> {
    reference_orbit_in(crate::backend::selected(), formula, z0x, z0y, cx, cy, params, max_iter, p)
}

/// [`reference_orbit`] in an explicitly named backend.
#[allow(clippy::too_many_arguments)]
pub fn reference_orbit_in(
    backend: crate::BackendChoice,
    formula: &Formula,
    z0x: &BigFloat,
    z0y: &BigFloat,
    cx: &BigFloat,
    cy: &BigFloat,
    params: &[(f64, f64)],
    max_iter: u32,
    p: usize,
) -> Result<(Vec<[f32; 4]>, u32, OrbitTail), IrError> {
    if !formula.bignum_evaluable() {
        return Err(IrError::NotBignum);
    }
    check_params(formula, params)?;
    let mut out = Vec::with_capacity(max_iter as usize + 1);
    let (xh, xl) = split_df64(crate::to_f64(z0x));
    let (yh, yl) = split_df64(crate::to_f64(z0y));
    out.push([xh, yh, xl, yl]); // Z_0
    let tail = match backend {
        crate::BackendChoice::Astro => {
            run_reference::<BigFloat>(&mut out, formula, z0x, z0y, cx, cy, params, max_iter, p)?
        }
        #[cfg(feature = "rug")]
        crate::BackendChoice::Rug => {
            run_reference::<rug::Float>(&mut out, formula, z0x, z0y, cx, cy, params, max_iter, p)?
        }
    };
    let len = out.len() as u32;
    Ok((out, len, tail))
}

#[allow(clippy::too_many_arguments)]
fn run_reference<B: RefBackend + IrField>(
    out: &mut Vec<[f32; 4]>,
    formula: &Formula,
    z0x: &BigFloat,
    z0y: &BigFloat,
    cx: &BigFloat,
    cy: &BigFloat,
    params: &[(f64, f64)],
    max_iter: u32,
    p: usize,
) -> Result<OrbitTail, IrError> {
    let ctx = B::ctx_for(p);
    let zero = BigFloat::from_f64(0.0, p);
    let z = (B::from_carrier(z0x, ctx), B::from_carrier(z0y, ctx));
    let zp = (B::from_carrier(&zero, ctx), B::from_carrier(&zero, ctx));
    let c = (B::from_carrier(cx, ctx), B::from_carrier(cy, ctx));
    let mut m = Machine::new(formula, z, zp, c, params, f64::from(max_iter), ctx)?;
    let mut escaped = false;
    // The reference orbit's escape test on the truncated `f64` view, by the rule
    // `crate::reference::run_orbit_gen` follows (lower from degree 7, where the next sample
    // would overflow f32). The bit-identity test compares orbit LENGTHS, so a drift between the
    // two is caught there.
    let escape2 = crate::reference::ref_escape2_of_degree(formula.escape_degree().unwrap_or(2.0));
    for n in 0..max_iter as usize {
        m.step(n)?;
        crate::reference::count_reference_step(n as u32 + 1);
        let xv = m.pool[Z_RE].to_f64_trunc();
        let yv = m.pool[Z_IM].to_f64_trunc();
        out.push(pack_sample(xv, yv));
        if xv * xv + yv * yv > escape2 {
            escaped = true;
            break;
        }
    }
    crate::backend::note_observed::<B>();
    Ok(OrbitTail {
        zx: m.pool[Z_RE].to_carrier(ctx),
        zy: m.pool[Z_IM].to_carrier(ctx),
        zpx: m.pool[ZP_RE].to_carrier(ctx),
        zpy: m.pool[ZP_IM].to_carrier(ctx),
        escaped,
        backend: B::BIT,
    })
}

/// Formulas written as Fractint-style expressions.
pub mod parse;

/// The syntax tree the parser reads a formula into (the textbook editor's view of it).
pub mod syntax;

/// Perturbed steps derived from a formula (deep zoom).
pub mod perturb;

/// Fractint's `.frm` formula files, read into the formula language.
pub mod frm;

#[cfg(test)]
mod tests;
