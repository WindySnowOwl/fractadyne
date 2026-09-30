//! Perturbed steps derived from a formula's IR (design/custom-formulas.md §4.4).
//!
//! Deep zoom iterates a pixel as `z = Z + δz`, with `Z` a reference orbit computed once in high
//! precision and `δz` small enough for low precision — but only if `δz' = f(Z+δz, C+δc) − f(Z, C)`
//! is computed WITHOUT forming `f(Z+δz)` and subtracting (the difference of two nearly equal large
//! numbers keeps none of δz's digits). This module rewrites a step into one that computes `δz'`
//! directly, by a fixed rule table over the IR — every rule cancellation-free:
//!
//! | value | reference `B` | perturbation `P` |
//! |---|---|---|
//! | `z`, `c` | `Z`, `C` | `δz`, `δc` |
//! | constant, parameter | itself | 0 |
//! | `a ± b` | `B(a) ± B(b)` | `P(a) ± P(b)` |
//! | `a·b` | `B(a)·B(b)` | `P(a)·W(b) + B(a)·P(b)`, with `W = B + P` |
//! | `sqr a` | `B(a)²` | `(2·B(a) + P(a))·P(a)` |
//! | `a^n` (integer) | — | the square-and-multiply chain, rule by rule |
//! | `k·a`, `−a`, `conj a`, `re a`, `im a` | likewise | likewise |
//! | `\|Re a\|` | `\|Re B(a)\|` | `diffabs(Re B(a), Re P(a))` (`DiffAbsRe`) |
//! | `\|a\|²` | `\|B(a)\|²` | `Re((2·B(a) + P(a))·conj P(a))` |
//!
//! The result is an ordinary IR [`Program`] over the inputs `Z`, `C` (the reference's values),
//! [`Op::Delta`] and [`Op::DeltaC`], so every interpreter and the WGSL generator run it unchanged.
//!
//! Not yet: division, powers with a non-integer exponent, the elementary functions (their rules
//! need the functions of the reference at high precision, design phase 5), and the previous iterate
//! (a second perturbation to carry).

use super::{Builder, Formula, Op, Program, Val};

/// Why a formula has no perturbed step (yet).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotPerturbable(pub &'static str);

impl std::fmt::Display for NotPerturbable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} has no deep-zoom form yet", self.0)
    }
}

impl std::error::Error for NotPerturbable {}

/// The perturbed step of every phase.
pub fn perturbed_formula(formula: &Formula) -> Result<Formula, NotPerturbable> {
    let phases = formula.phases().iter().map(perturbed).collect::<Result<Vec<_>, _>>()?;
    Ok(Formula::new(phases).expect("one phase per phase, and there was at least one"))
}

/// Whether [`perturbed_formula`] succeeds.
pub fn perturbable(formula: &Formula) -> bool {
    formula.phases().iter().all(|p| perturbed(p).is_ok())
}

/// A value of the step, as the rewritten program sees it: its reference `B`, and its perturbation
/// `P` (`None` = known zero, so a constant costs nothing).
#[derive(Clone, Copy)]
struct Pair {
    b: Val,
    p: Option<Val>,
}

struct Rewriter {
    out: Builder,
    zero: Option<Val>,
}

impl Rewriter {
    fn push(&mut self, op: Op) -> Val {
        self.out.push(op)
    }
    /// `W = B + P`.
    fn full(&mut self, x: Pair) -> Val {
        match x.p {
            Some(p) => self.push(Op::Add(x.b, p)),
            None => x.b,
        }
    }
    fn add(&mut self, a: Option<Val>, b: Option<Val>, sub: bool) -> Option<Val> {
        match (a, b) {
            (None, None) => None,
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(if sub { self.push(Op::Neg(b)) } else { b }),
            (Some(a), Some(b)) => Some(self.push(if sub { Op::Sub(a, b) } else { Op::Add(a, b) })),
        }
    }
    fn mul(&mut self, x: Pair, y: Pair) -> Pair {
        let b = self.push(Op::Mul(x.b, y.b));
        // P(x·y) = P(x)·W(y) + B(x)·P(y)
        let left = x.p.map(|px| {
            let wy = self.full(y);
            self.push(Op::Mul(px, wy))
        });
        let right = y.p.map(|py| self.push(Op::Mul(x.b, py)));
        let p = self.add(left, right, false);
        Pair { b, p }
    }
    fn sqr(&mut self, x: Pair) -> Pair {
        let b = self.push(Op::Sqr(x.b));
        // P(x²) = (2·B(x) + P(x))·P(x)
        let p = x.p.map(|px| {
            let two_b = self.push(Op::Scale(x.b, 2.0));
            let s = self.push(Op::Add(two_b, px));
            self.push(Op::Mul(s, px))
        });
        Pair { b, p }
    }
    fn zero(&mut self) -> Val {
        if let Some(z) = self.zero {
            return z;
        }
        let z = self.push(Op::Const(0.0, 0.0));
        self.zero = Some(z);
        z
    }
}

/// The perturbed step of one program: a program computing `δz'` from `Z`, `C`, `δz`, `δc`.
pub fn perturbed(prog: &Program) -> Result<Program, NotPerturbable> {
    let mut r = Rewriter { out: Builder::new(), zero: None };
    let mut vals: Vec<Pair> = Vec::with_capacity(prog.insts().len());
    for op in prog.insts() {
        let v = |i: Val| vals[i.index()];
        let pair = match *op {
            Op::Z => {
                let b = r.push(Op::Z);
                let p = r.push(Op::Delta);
                Pair { b, p: Some(p) }
            }
            Op::C => {
                let b = r.push(Op::C);
                let p = r.push(Op::DeltaC);
                Pair { b, p: Some(p) }
            }
            Op::Const(..) | Op::Param(_) => Pair { b: r.push(*op), p: None },
            Op::Add(a, b) | Op::Sub(a, b) => {
                let (x, y) = (v(a), v(b));
                let sub = matches!(op, Op::Sub(..));
                let bb = r.push(if sub { Op::Sub(x.b, y.b) } else { Op::Add(x.b, y.b) });
                let p = r.add(x.p, y.p, sub);
                Pair { b: bb, p }
            }
            Op::Mul(a, b) => {
                let (x, y) = (v(a), v(b));
                r.mul(x, y)
            }
            Op::Sqr(a) => {
                let x = v(a);
                r.sqr(x)
            }
            Op::PowI(a, n) => {
                // The interpreters' own chain, each link by its rule.
                let base = v(a);
                let mut acc = base;
                for bit in (0..31 - n.leading_zeros()).rev() {
                    acc = r.sqr(acc);
                    if (n >> bit) & 1 == 1 {
                        acc = r.mul(acc, base);
                    }
                }
                acc
            }
            Op::Scale(a, k) => {
                let x = v(a);
                let b = r.push(Op::Scale(x.b, k));
                let p = x.p.map(|p| r.push(Op::Scale(p, k)));
                Pair { b, p }
            }
            Op::Neg(a) | Op::Conj(a) | Op::Re(a) | Op::Im(a) => {
                let x = v(a);
                let wrap = |t: Val| match *op {
                    Op::Neg(_) => Op::Neg(t),
                    Op::Conj(_) => Op::Conj(t),
                    Op::Re(_) => Op::Re(t),
                    _ => Op::Im(t),
                };
                let b = r.push(wrap(x.b));
                let p = x.p.map(|p| r.push(wrap(p)));
                Pair { b, p }
            }
            Op::AbsRe(a) | Op::AbsIm(a) => {
                let x = v(a);
                let re = matches!(op, Op::AbsRe(_));
                let b = r.push(if re { Op::AbsRe(x.b) } else { Op::AbsIm(x.b) });
                let p = x.p.map(|p| r.push(if re { Op::DiffAbsRe(x.b, p) } else { Op::DiffAbsIm(x.b, p) }));
                Pair { b, p }
            }
            Op::Norm(a) => {
                let x = v(a);
                let b = r.push(Op::Norm(x.b));
                // |W|² − |B|² = Re((2B + P)·conj P)
                let p = x.p.map(|p| {
                    let two_b = r.push(Op::Scale(x.b, 2.0));
                    let s = r.push(Op::Add(two_b, p));
                    let cp = r.push(Op::Conj(p));
                    let m = r.push(Op::Mul(s, cp));
                    r.push(Op::Re(m))
                });
                Pair { b, p }
            }
            Op::ZPrev => return Err(NotPerturbable("the previous iterate")),
            Op::Div(..) => return Err(NotPerturbable("division")),
            Op::Pow(..) => return Err(NotPerturbable("a non-integer power")),
            Op::Func(..) => return Err(NotPerturbable("an elementary function")),
            Op::Delta | Op::DeltaC | Op::DiffAbsRe(..) | Op::DiffAbsIm(..) => {
                return Err(NotPerturbable("an already perturbed program"))
            }
        };
        vals.push(pair);
    }
    let out = match vals[prog.out().index()].p {
        Some(p) => p,
        None => r.zero(),
    };
    Ok(r.out.finish(out).expect("rewritten in order").without_dead_code())
}

#[cfg(test)]
mod tests;
