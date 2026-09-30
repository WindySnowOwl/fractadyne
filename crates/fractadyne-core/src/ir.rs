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

/// Elementary functions. `f64` only: their bignum forms need their own bit-identity contract across
/// backends before a reference orbit may depend on them (design phase 5).
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
    /// Complex division, `a·conj(b) / |b|²`. Not exact-ring: `f64` only.
    Div(Val, Val),
    /// `a^b = exp(b·log a)`, and `0^b = 0`. `f64` only.
    Pow(Val, Val),
    /// An elementary function. `f64` only.
    Func(Func, Val),
}

impl Op {
    /// The operands this instruction reads.
    pub fn operands(&self) -> impl Iterator<Item = Val> {
        let (a, b) = match *self {
            Op::Z | Op::C | Op::ZPrev | Op::Param(_) | Op::Const(..) => (None, None),
            Op::Add(a, b) | Op::Sub(a, b) | Op::Mul(a, b) | Op::Div(a, b) | Op::Pow(a, b) => {
                (Some(a), Some(b))
            }
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
            | Op::Func(_, a) => (Some(a), None),
        };
        a.into_iter().chain(b)
    }

    /// Whether every field this crate iterates in evaluates it exactly the same way: the ring
    /// operations plus the exact ones (sign, abs, parts). Division and the elementary functions are
    /// not — see [`Func`].
    pub fn is_ring(&self) -> bool {
        !matches!(self, Op::Div(..) | Op::Pow(..) | Op::Func(..))
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
    /// A bignum evaluation of a program with non-ring operations.
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
                "division and elementary functions have no bignum form yet (f64 only)"
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
        Ok(Program { insts, out })
    }

    pub fn insts(&self) -> &[Op] {
        &self.insts
    }

    pub fn out(&self) -> Val {
        self.out
    }

    pub fn reads_zprev(&self) -> bool {
        self.insts.iter().any(|op| matches!(op, Op::ZPrev))
    }

    /// Whether the bignum interpreter can run it (see [`Op::is_ring`]).
    pub fn bignum_evaluable(&self) -> bool {
        self.insts.iter().all(Op::is_ring)
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
#[derive(Clone, Debug, PartialEq)]
pub struct Formula {
    phases: Vec<Program>,
}

impl Formula {
    pub fn new(phases: Vec<Program>) -> Result<Self, IrError> {
        if phases.is_empty() {
            return Err(IrError::Empty);
        }
        Ok(Formula { phases })
    }

    pub fn single(step: Program) -> Self {
        Formula { phases: vec![step] }
    }

    pub fn phases(&self) -> &[Program] {
        &self.phases
    }

    pub fn reads_zprev(&self) -> bool {
        self.phases.iter().any(Program::reads_zprev)
    }

    pub fn bignum_evaluable(&self) -> bool {
        self.phases.iter().all(Program::bignum_evaluable)
    }

    pub fn param_count(&self) -> usize {
        self.phases.iter().map(Program::param_count).max().unwrap_or(0)
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
        _ => return None,
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
    /// A non-ring operation on materialised operands, or `None` where this field has none.
    fn elementary(_op: &Op, _a: &(Self, Self), _b: Option<&(Self, Self)>) -> Option<(Self, Self)> {
        None
    }
}

impl IrField for f64 {
    fn konst(v: f64, _: ()) -> f64 {
        v
    }
    fn fneg(self) -> f64 {
        -self
    }
    fn elementary(op: &Op, a: &(f64, f64), b: Option<&(f64, f64)>) -> Option<(f64, f64)> {
        Some(match *op {
            Op::Div(..) => cdiv(*a, *b?),
            Op::Pow(..) => cpow(*a, *b?),
            Op::Func(f, _) => cfunc(f, *a),
            _ => return None,
        })
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
}

#[cfg(feature = "rug")]
impl IrField for rug::Float {
    fn konst(v: f64, ctx: u32) -> rug::Float {
        <rug::Float as RefBackend>::from_f64(v, ctx)
    }
    fn fneg(self) -> rug::Float {
        -self
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

/// The principal logarithm (branch cut on the negative real axis).
fn clog(a: (f64, f64)) -> (f64, f64) {
    (a.0.hypot(a.1).ln(), a.1.atan2(a.0))
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
        (y.abs() / (2.0 * t), t.copysign(y))
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
        Func::Tan => cdiv(cfunc(Func::Sin, a), cfunc(Func::Cos, a)),
        Func::Sinh => (x.sinh() * y.cos(), x.cosh() * y.sin()),
        Func::Cosh => (x.cosh() * y.cos(), x.sinh() * y.sin()),
        Func::Tanh => cdiv(cfunc(Func::Sinh, a), cfunc(Func::Cosh, a)),
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
    ctx: F::Ctx,
}

impl<'f, F: IrField> Machine<'f, F> {
    fn new(
        formula: &'f Formula,
        z: (F, F),
        zp: (F, F),
        c: (F, F),
        params: &[(f64, f64)],
        ctx: F::Ctx,
    ) -> Result<Self, IrError> {
        let mut pool = vec![z.0, z.1, zp.0, zp.1, c.0, c.1, F::konst(0.0, ctx)];
        let push = |v: f64, pool: &mut Vec<F>| {
            pool.push(F::konst(v, ctx));
            S::pos(pool.len() - 1)
        };
        let mut leaves = Vec::with_capacity(formula.phases.len());
        for prog in &formula.phases {
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
        Ok(Machine { formula, pool, fixed, leaves, vals: Vec::new(), shift, ctx })
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

    /// Evaluate phase `k` on the current state; the result's scalars are in the pool.
    fn eval(&mut self, k: usize) -> Result<Cs, IrError> {
        let formula = self.formula;
        let prog = &formula.phases[k];
        self.pool.truncate(self.fixed);
        self.vals.clear();
        let zero = S::pos(ZERO);
        for (i, op) in prog.insts.iter().enumerate() {
            let g = |v: Val| self.vals[v.index()];
            let v = match *op {
                Op::Z => Cs { re: S::pos(Z_RE), im: S::pos(Z_IM) },
                Op::ZPrev => Cs { re: S::pos(ZP_RE), im: S::pos(ZP_IM) },
                Op::C => Cs { re: S::pos(C_RE), im: S::pos(C_IM) },
                Op::Const(..) | Op::Param(_) => self.leaves[k][i].expect("converted in new()"),
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
                Op::Div(a, b) | Op::Pow(a, b) => {
                    let (a, b) = (g(a), g(b));
                    let a = (self.materialize(a.re), self.materialize(a.im));
                    let b = (self.materialize(b.re), self.materialize(b.im));
                    let (re, im) = F::elementary(op, &a, Some(&b)).ok_or(IrError::NotBignum)?;
                    Cs { re: self.push(re), im: self.push(im) }
                }
                Op::Func(_, a) => {
                    let a = g(a);
                    let a = (self.materialize(a.re), self.materialize(a.im));
                    let (re, im) = F::elementary(op, &a, None).ok_or(IrError::NotBignum)?;
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
        let out = self.eval(n % self.formula.phases.len())?;
        self.commit(out);
        Ok(())
    }
}

fn check_params(formula: &Formula, params: &[(f64, f64)]) -> Result<(), IrError> {
    let need = formula.param_count();
    if params.len() < need {
        return Err(IrError::MissingParam { index: (need - 1) as u16, supplied: params.len() });
    }
    Ok(())
}

/// One `f64` step of `prog`: `z' = prog(z, c, z_prev)`.
pub fn step_f64(
    prog: &Program,
    z: (f64, f64),
    c: (f64, f64),
    zprev: (f64, f64),
    params: &[(f64, f64)],
) -> Result<(f64, f64), IrError> {
    let formula = Formula::single(prog.clone());
    check_params(&formula, params)?;
    let mut m = Machine::new(&formula, z, zprev, c, params, ())?;
    m.step(0)?;
    Ok((m.pool[Z_RE], m.pool[Z_IM]))
}

/// The `f64` orbit, with [`crate::orbit_points`]'s contract: `z₀` first, then each iterate until
/// `|z|² > bailout2` or `max_points` steps. `z_prev` starts at zero.
pub fn orbit_points(
    formula: &Formula,
    z0: (f64, f64),
    c: (f64, f64),
    params: &[(f64, f64)],
    max_points: usize,
    bailout2: f64,
) -> Result<Vec<(f64, f64)>, IrError> {
    check_params(formula, params)?;
    let mut m = Machine::new(formula, z0, (0.0, 0.0), c, params, ())?;
    let mut pts = Vec::with_capacity(max_points.min(1024) + 1);
    pts.push(z0);
    for n in 0..max_points {
        m.step(n)?;
        let (x, y) = (m.pool[Z_RE], m.pool[Z_IM]);
        pts.push((x, y));
        if x * x + y * y > bailout2 {
            break;
        }
    }
    Ok(pts)
}

/// The reference orbit's escape test on the truncated `f64` view — the literal
/// `crate::reference::run_orbit_gen` uses. The bit-identity test compares orbit LENGTHS, so a
/// drift between the two is caught there.
const REFERENCE_BAILOUT2: f64 = 1.0e12;

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
    let mut m = Machine::new(formula, z, zp, c, params, ctx)?;
    let mut escaped = false;
    for n in 0..max_iter as usize {
        m.step(n)?;
        let xv = m.pool[Z_RE].to_f64_trunc();
        let yv = m.pool[Z_IM].to_f64_trunc();
        out.push(pack_sample(xv, yv));
        if xv * xv + yv * yv > REFERENCE_BAILOUT2 {
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

#[cfg(test)]
mod tests;
