//! Unlimited zoom (design/lsystems.md §4.6, phase 3).
//!
//! The `f64` walk places every subtree relative to the view in pixels; at 1e13× a step of the curve
//! is a pixel of a picture 1e13 pixels across, and `f64`'s 53 bits no longer place it. Here the top
//! of the derivation — the subtrees larger than the switch size (2³² pixels by default), the few
//! that reach into the view at each depth — is walked in `BigFloat`, from tables built in
//! `BigFloat` ([`BigTables`]): each subtree's displacement, its net turn, its reach. When a visible
//! subtree is small enough, its root is turned into view pixels once and the `f64` walk
//! ([`walk_from`]) draws it. So a view at 1e100× costs what one at 1× does, and is exact to the
//! precision asked for.
//!
//! **Headings are whole numbers.** Every angle a system turns by — its division of the circle, a
//! decimal (`angle 25.7` is 257/3,600 of a circle), a `\a` or `/a`, its first heading, `|` — is a
//! rational fraction of a turn, so a heading is a whole count of their common unit, and a
//! subtree's turn is an integer sum. Its unit vector is computed from the count, never as a
//! product of others: a product of unit vectors carries its rounding into every level above it,
//! four times over for the Koch curve, and the first version's headings had collapsed to (0, 0) by
//! depth 415. What cannot be exact is a step factor taken from an `f64` (`@Q2`'s nearest double): a
//! system using one zooms deep with that double as the truth.

use super::system::{Angle, LSystem, Role, Tok};
use super::tables::{ColourFx, Tables, NONE};
use super::variant::Variants;
use super::walk::{walk_from, Drawn, Polygon, Segment, SubStart, View, WalkOptions, WalkStats};
use crate::bignum::{arg_bf, log2_abs, to_f64, RM};
use astro_float::{BigFloat, Consts};
use std::collections::HashMap;
use std::sync::Mutex;

type Cx = [BigFloat; 2];

fn cmul(a: &Cx, b: &Cx, p: usize) -> Cx {
    [
        a[0].mul(&b[0], p, RM).sub(&a[1].mul(&b[1], p, RM), p, RM),
        a[0].mul(&b[1], p, RM).add(&a[1].mul(&b[0], p, RM), p, RM),
    ]
}

fn conj(a: &Cx) -> Cx {
    [a[0].clone(), a[1].neg()]
}

fn cadd(a: &Cx, b: &Cx, p: usize) -> Cx {
    [a[0].add(&b[0], p, RM), a[1].add(&b[1], p, RM)]
}

fn cscale(a: &Cx, s: &BigFloat, p: usize) -> Cx {
    [a[0].mul(s, p, RM), a[1].mul(s, p, RM)]
}

fn czero(p: usize) -> Cx {
    [BigFloat::from_f64(0.0, p), BigFloat::from_f64(0.0, p)]
}

fn cone(p: usize) -> Cx {
    [BigFloat::from_f64(1.0, p), BigFloat::from_f64(0.0, p)]
}

fn cnormalise(a: &Cx, p: usize) -> Option<Cx> {
    let len = a[0].mul(&a[0], p, RM).add(&a[1].mul(&a[1], p, RM), p, RM).sqrt(p, RM);
    (!len.is_zero()).then(|| [a[0].div(&len, p, RM), a[1].div(&len, p, RM)])
}

/// `log₂ |a|` (−∞ for zero), at any magnitude.
fn norm_log2(a: &Cx, p: usize) -> f64 {
    if a[0].is_zero() && a[1].is_zero() {
        return f64::NEG_INFINITY;
    }
    let m = log2_abs(&a[0]).max(log2_abs(&a[1]));
    // Scaled by the larger component first, so the squares cannot leave any range.
    let ex = -(m.floor() as i32);
    let scale = |v: &BigFloat| {
        let mut v = v.clone();
        if let Some(e) = v.exponent() {
            v.set_exponent(e + ex);
        }
        v
    };
    let (sx, sy) = (scale(&a[0]), scale(&a[1]));
    0.5 * log2_abs(&sx.mul(&sx, p, RM).add(&sy.mul(&sy, p, RM), p, RM)) - f64::from(ex)
}

/// `log₂(2^a + 2^b)`.
fn log2_add(a: f64, b: f64) -> f64 {
    let (hi, lo) = if a >= b { (a, b) } else { (b, a) };
    if lo == f64::NEG_INFINITY {
        return hi;
    }
    hi + (1.0 + (lo - hi).exp2()).log2()
}

/// `2^x` as a `BigFloat`, for any `x`.
fn pow2(x: f64, p: usize) -> BigFloat {
    let e = x.floor();
    let mut v = BigFloat::from_f64((x - e).exp2(), p);
    if let Some(ve) = v.exponent() {
        v.set_exponent(ve + e as i32);
    }
    v
}

/// A decimal written as an `f64` (a step factor), as the decimal: `0.6`, not the double nearest.
fn decimal(v: f64, p: usize) -> BigFloat {
    crate::parse_bf_prec(&format!("{v}"), p).unwrap_or_else(|| BigFloat::from_f64(v, p))
}

fn gcd(a: i128, b: i128) -> i128 {
    if b == 0 {
        a.abs()
    } else {
        gcd(b, a % b)
    }
}

/// `v` as the decimal fraction it was written as: `(num, den)`, `den` a power of ten up to 10¹⁸.
fn decimal_ratio(v: f64) -> Option<(i128, i128)> {
    let s = format!("{v:e}");
    let (mant, exp) = s.split_once('e')?;
    let exp: i32 = exp.parse().ok()?;
    let neg = mant.starts_with('-');
    let mant = mant.trim_start_matches('-');
    let (int, frac) = mant.split_once('.').unwrap_or((mant, ""));
    let digits: i128 = format!("{int}{frac}").parse().ok()?;
    // value = digits · 10^(exp − len(frac))
    let shift = exp - frac.len() as i32;
    let (num, den) = if shift >= 0 {
        (digits.checked_mul(10i128.checked_pow(shift as u32)?)?, 1)
    } else {
        (digits, 10i128.checked_pow((-shift) as u32)?)
    };
    let g = gcd(num, den).max(1);
    Some((if neg { -num / g } else { num / g }, den / g))
}

/// The system's angles as whole counts of one unit: `1/modulus` of a turn.
struct Units {
    modulus: i128,
    /// One `+`.
    step: i128,
    /// `|`.
    around: i128,
}

impl Units {
    /// Every angle `sys` turns by, as a fraction of a turn over one common denominator.
    fn of(sys: &LSystem) -> Option<Units> {
        // (numerator, denominator) of a fraction of a turn.
        let turn_of_degrees = |d: f64| decimal_ratio(d).map(|(n, q)| (n, q * 360));
        let step = match sys.angle {
            Angle::Division(n) => (1, i128::from(n)),
            Angle::Degrees(d) => match sys.angle.division() {
                Some(n) => (1, i128::from(n)),
                None => turn_of_degrees(d)?,
            },
        };
        let around = match sys.angle.division() {
            Some(n) => (i128::from(n / 2), i128::from(n)),
            None => (1, 2),
        };
        let mut fractions = vec![step, around, turn_of_degrees(sys.heading)?];
        for w in sys.words() {
            for t in w {
                if let Tok::TurnBy(a) = t {
                    fractions.push(turn_of_degrees(*a)?);
                }
            }
        }
        let mut m: i128 = 1;
        for &(_, q) in &fractions {
            m = m.checked_mul(q / gcd(m, q).max(1))?;
            if m > 1 << 100 {
                return None;
            }
        }
        let count = |(n, q): (i128, i128)| (n * (m / q)).rem_euclid(m);
        Some(Units { modulus: m, step: count(step), around: count(around) })
    }

    fn degrees(&self, d: f64) -> i128 {
        let (n, q) = decimal_ratio(d).expect("checked in Units::of");
        (n * (self.modulus / (q * 360))).rem_euclid(self.modulus)
    }
}

/// A subtree's effect (as [`super::Effect`]), in `BigFloat`, its turn a whole count of units.
#[derive(Clone, Debug)]
struct BigEffect {
    d: Cx,
    turns: i128,
    flip: bool,
    scale: BigFloat,
    colour: ColourFx,
}

#[derive(Clone, Debug)]
struct BigEntry {
    fx: BigEffect,
    /// `log₂` of the reach, in the subtree's own steps (−∞: it draws nothing).
    reach: f64,
    /// `log₂` of how far the turtle goes at all (see `Entry::ra`).
    reach_all: f64,
    n: f64,
}

#[derive(Clone, Debug)]
enum BigOp {
    Sym(u8),
    /// Turn left by this many units (right when reversed).
    Turn(i128),
    Reverse,
    Push,
    Pop,
    Scale(BigFloat),
    SetColour(i32),
    AddColour(i32),
    PolyStart,
    PolyEnd,
    Vertex,
}

/// A system's tables in `BigFloat`, at a precision, to a depth.
pub struct BigTables {
    pub prec: usize,
    pub depth: u32,
    modulus: i128,
    axiom: Vec<BigOp>,
    /// Every production's word, and which one a node of each rewriting symbol and variant takes
    /// (as [`Tables`]).
    rules: Vec<Vec<BigOp>>,
    id: [u16; 256],
    variants: Variants,
    choice: Vec<u16>,
    roles: [Role; 256],
    constant: Vec<BigEntry>,
    /// `rows[d][id · k + v]` for `d` in `1..=depth`.
    rows: Vec<Vec<BigEntry>>,
    /// The system's first heading, in units.
    heading: i128,
    /// The measure's direction at the `f64` tables' deepest row (see [`BigTables::new`]).
    along: Cx,
    /// Unit vectors by heading count, as asked for.
    rots: Mutex<(Consts, HashMap<i128, Cx>)>,
}

impl BigTables {
    /// `sys`'s tables at `prec` bits, to `depth` (the order the view needs) — and to the depth the
    /// picture's orientation is taken from ([`Tables::along_depth`]), whose direction is computed
    /// here at this precision: so a 90° curve stays exactly on its lattice, and the deep and `f64`
    /// walks agree on where everything is. `None` when the system's angles have no common unit
    /// that fits (a decimal of more than 18 places).
    pub fn new(sys: &LSystem, t: &Tables, prec: usize, depth: u32) -> Option<BigTables> {
        let mut bt = Self::raw(sys, prec, depth.max(t.along_depth))?;
        if let (true, Some(c)) = (t.grows(), t.measure_sym) {
            if let Some(a) = cnormalise(&bt.entry(c, t.along_depth, t.measure_variant).fx.d, bt.prec) {
                bt.along = a;
            }
        }
        Some(bt)
    }

    /// The tables alone, to `depth`, with no orientation.
    fn raw(sys: &LSystem, prec: usize, depth: u32) -> Option<BigTables> {
        let p = prec.max(64);
        let u = Units::of(sys)?;
        let compile = |w: &[Tok]| -> Vec<BigOp> {
            w.iter()
                .map(|&tok| match tok {
                    Tok::Sym(c) => BigOp::Sym(c),
                    Tok::Turn(k) => BigOp::Turn(i128::from(k) * u.step),
                    Tok::Around => BigOp::Turn(u.around),
                    Tok::TurnBy(a) => BigOp::Turn(u.degrees(a)),
                    Tok::Reverse => BigOp::Reverse,
                    Tok::Push => BigOp::Push,
                    Tok::Pop => BigOp::Pop,
                    Tok::Scale(f) => BigOp::Scale(decimal(f, p)),
                    Tok::SetColour(c) => BigOp::SetColour(c),
                    Tok::AddColour(c) => BigOp::AddColour(c),
                    Tok::PolyStart => BigOp::PolyStart,
                    Tok::PolyEnd => BigOp::PolyEnd,
                    Tok::Vertex => BigOp::Vertex,
                })
                .collect()
        };
        let variants = Variants::of(sys);
        let k = variants.k as usize;
        let mut id = [NONE; 256];
        let mut rules = Vec::new();
        let mut choice = Vec::new();
        let mut syms = 0usize;
        for c in 0..=255u8 {
            let ps = sys.productions(c);
            if ps.is_empty() {
                continue;
            }
            id[c as usize] = syms as u16;
            syms += 1;
            let first = rules.len() as u16;
            rules.extend(ps.iter().map(|pr| compile(&pr.word)));
            let weights: Vec<f64> = ps.iter().map(|pr| pr.weight).collect();
            choice.extend((0..k as u32).map(|v| first + variants.choose(c, v, &weights) as u16));
        }
        let mut roles = [Role::None; 256];
        let identity = BigEffect { d: czero(p), turns: 0, flip: false, scale: BigFloat::from_f64(1.0, p), colour: ColourFx::default() };
        let constant: Vec<BigEntry> = (0..256)
            .map(|c| {
                roles[c] = sys.roles[c];
                let step = BigEffect { d: cone(p), ..identity.clone() };
                match sys.roles[c] {
                    Role::Draw => BigEntry { fx: step, reach: 0.0, reach_all: 0.0, n: 1.0 },
                    Role::Move => BigEntry { fx: step, reach: f64::NEG_INFINITY, reach_all: 0.0, n: 0.0 },
                    Role::None => BigEntry { fx: identity.clone(), reach: f64::NEG_INFINITY, reach_all: f64::NEG_INFINITY, n: 0.0 },
                }
            })
            .collect();
        let mut bt = BigTables {
            prec: p,
            depth: 0,
            modulus: u.modulus,
            axiom: compile(&sys.axiom),
            rules,
            id,
            variants,
            choice,
            roles,
            constant,
            rows: vec![Vec::new()],
            heading: u.degrees(sys.heading),
            along: cone(p),
            rots: Mutex::new((Consts::new().expect("astro-float constants"), HashMap::new())),
        };
        for d in 1..=depth {
            let row: Vec<BigEntry> =
                (0..syms * k).map(|i| bt.fold(&bt.rules[bt.choice[i] as usize], d - 1, (i % k) as u32)).collect();
            bt.rows.push(row);
            bt.depth = d;
        }
        Some(bt)
    }

    /// The unit vector of a heading count — from the count, exactly where it is a whole number
    /// of quarter turns.
    fn rot(&self, turns: i128) -> Cx {
        let p = self.prec;
        let k = turns.rem_euclid(self.modulus);
        let mut guard = self.rots.lock().expect("rotation cache");
        let (cc, map) = &mut *guard;
        if let Some(r) = map.get(&k) {
            return r.clone();
        }
        let r = if (4 * k) % self.modulus == 0 {
            let (c, s) = [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)][(4 * k / self.modulus) as usize];
            [BigFloat::from_f64(c, p), BigFloat::from_f64(s, p)]
        } else {
            // 2πk/m, with k and m reduced so the integers stay small enough to be exact.
            let g = gcd(k, self.modulus).max(1);
            let parse = |v: i128, cc: &mut Consts| BigFloat::parse(&v.to_string(), astro_float::Radix::Dec, p, RM, cc);
            let num = parse(2 * (k / g), cc);
            let den = parse(self.modulus / g, cc);
            let a = cc.pi(p, RM).mul(&num, p, RM).div(&den, p, RM);
            [a.cos(p, RM, cc), a.sin(p, RM, cc)]
        };
        map.insert(k, r.clone());
        r
    }

    /// How many segments the axiom draws at `order` (≤ `depth`): what position along the curve
    /// is a fraction of.
    pub fn axiom_count(&self, order: u32) -> f64 {
        let (d, root) = (order.min(self.depth), self.variants.root);
        self.axiom
            .iter()
            .enumerate()
            .map(|(j, op)| if let BigOp::Sym(c) = op { self.entry(*c, d, self.variants.child(root, j)).n } else { 0.0 })
            .sum()
    }

    fn entry(&self, c: u8, d: u32, v: u32) -> &BigEntry {
        let id = self.id[c as usize];
        if id == NONE || d == 0 {
            &self.constant[c as usize]
        } else {
            &self.rows[d as usize][id as usize * self.variants.k as usize + v as usize]
        }
    }

    /// The word a node of rewriting symbol `id` and variant `v` rewrites by.
    fn word_of(&self, id: u16, v: u32) -> u16 {
        self.choice[id as usize * self.variants.k as usize + v as usize]
    }

    /// `a`, then `b` in the frame `a` leaves the turtle in.
    fn then(&self, a: &BigEffect, b: &BigEffect) -> BigEffect {
        let p = self.prec;
        let v = if a.flip { conj(&b.d) } else { b.d.clone() };
        let w = cmul(&self.rot(a.turns), &v, p);
        BigEffect {
            d: cadd(&a.d, &cscale(&w, &a.scale, p), p),
            turns: (a.turns + if a.flip { -b.turns } else { b.turns }).rem_euclid(self.modulus),
            flip: a.flip ^ b.flip,
            scale: a.scale.mul(&b.scale, p, RM),
            colour: a.colour.then(b.colour),
        }
    }

    fn fold(&self, ops: &[BigOp], d: u32, v: u32) -> BigEntry {
        let p = self.prec;
        let mut st = BigEffect { d: czero(p), turns: 0, flip: false, scale: BigFloat::from_f64(1.0, p), colour: ColourFx::default() };
        let mut stack: Vec<BigEffect> = Vec::new();
        let (mut reach, mut reach_all, mut n) = (f64::NEG_INFINITY, f64::NEG_INFINITY, 0.0f64);
        // Inside `{ }`: every position a vertex, no lines (as `Tables::fold`).
        let mut poly = false;
        for (j, op) in ops.iter().enumerate() {
            match op {
                BigOp::Sym(c) => {
                    let e = self.entry(*c, d, self.variants.child(v, j));
                    let from = norm_log2(&st.d, p);
                    let scale = log2_abs(&st.scale);
                    if e.reach_all > f64::NEG_INFINITY {
                        reach_all = reach_all.max(log2_add(from, scale + e.reach_all));
                    }
                    if poly {
                        if e.reach_all > f64::NEG_INFINITY {
                            reach = reach.max(log2_add(from, scale + e.reach_all));
                        }
                    } else if e.reach > f64::NEG_INFINITY {
                        reach = reach.max(log2_add(from, scale + e.reach));
                        n += e.n;
                    }
                    st = self.then(&st, &e.fx);
                }
                BigOp::PolyStart => {
                    poly = true;
                    reach = reach.max(norm_log2(&st.d, p));
                }
                BigOp::PolyEnd => poly = false,
                BigOp::Vertex => {
                    if poly {
                        reach = reach.max(norm_log2(&st.d, p));
                    }
                }
                BigOp::Turn(k) => st.turns = (st.turns + if st.flip { -k } else { *k }).rem_euclid(self.modulus),
                BigOp::Reverse => st.flip = !st.flip,
                BigOp::Push => stack.push(st.clone()),
                BigOp::Pop => st = stack.pop().expect("words are bracket-balanced"),
                BigOp::Scale(f) => st.scale = st.scale.mul(f, p, RM),
                BigOp::SetColour(c) => st.colour = st.colour.then(ColourFx { set: Some(*c), add: 0 }),
                BigOp::AddColour(c) => st.colour = st.colour.then(ColourFx { set: None, add: *c }),
            }
        }
        BigEntry { fx: st, reach, reach_all, n }
    }

    /// The world step at `order` (≤ `depth`), as the `f64` [`Tables::step`] defines it: the system's
    /// heading times the deepest row's measure direction over the measure at `order` — or the
    /// heading alone, for a picture that does not grow.
    fn step(&self, t: &Tables, order: u32) -> Cx {
        let p = self.prec;
        let heading = self.rot(self.heading);
        if !t.grows() {
            return heading;
        }
        let base = cmul(&heading, &self.along, p);
        let m = match t.measure_sym {
            Some(c) => self.entry(c, order, t.measure_variant).fx.d.clone(),
            None => cone(p),
        };
        if m[0].is_zero() && m[1].is_zero() {
            // As `Tables::step`'s growth-law branch: only at orders far shallower than a deep view.
            return cscale(&base, &pow2(t.step_log2(order), p), p);
        }
        // base / m = base · conj(m) / |m|²
        let m2 = m[0].mul(&m[0], p, RM).add(&m[1].mul(&m[1], p, RM), p, RM);
        let q = cmul(&base, &conj(&m), p);
        [q[0].div(&m2, p, RM), q[1].div(&m2, p, RM)]
    }
}

/// The measure symbol's displacement at a depth `d` and the two below, summarised exactly (for
/// [`Tables`]: its orientation, growth and turn per order).
pub(crate) struct ExactMeasure {
    /// The direction at `d`, a unit vector.
    pub along: [f64; 2],
    /// `log₂` of its length at `d`, `d − 1`, `d − 2`.
    pub log2_norm: [f64; 3],
    /// Its turn from `d − 1` to `d`, radians.
    pub turn: f64,
}

pub(crate) fn exact_measure(sys: &LSystem, sym: u8, variant: u32, d: u32, prec: usize) -> Option<ExactMeasure> {
    let bt = BigTables::raw(sys, prec, d)?;
    let p = bt.prec;
    let m = |k: u32| bt.entry(sym, k, variant).fx.d.clone();
    let (a, b, c) = (m(d), m(d - 1), m(d - 2));
    let along = cnormalise(&a, p)?;
    // a / b = a · conj(b) / |b|²: only its direction matters.
    let r = cmul(&a, &conj(&b), p);
    Some(ExactMeasure {
        along: [to_f64(&along[0]), to_f64(&along[1])],
        log2_norm: [norm_log2(&a, p), norm_log2(&b, p), norm_log2(&c, p)],
        turn: if r[0].is_zero() && r[1].is_zero() { 0.0 } else { arg_bf(&r[0], &r[1], p) },
    })
}

/// A view for the deep walk: its centre exact, its scale by `log₂`.
#[derive(Clone, Debug)]
pub struct DeepView {
    pub centre: [BigFloat; 2],
    /// `log₂` of world units a pixel.
    pub upp_log2: f64,
    pub size: [f64; 2],
    pub margin: f64,
}

/// The precision a deep walk at `order` of a picture about `2^extent_log2` world units across
/// needs at `upp_log2`: the view's own digits, the bits the tables lose down to that order (and to
/// the orientation's depth), and guard bits.
pub fn deep_precision(t: &Tables, order: u32, upp_log2: f64, extent_log2: f64) -> usize {
    let view = (extent_log2 - upp_log2).max(0.0).ceil() as usize;
    let lost = (f64::from(order.max(t.along_depth)) * t.loss).ceil() as usize;
    view + lost + 96
}

/// Whether a polygon's box (view pixels) meets the view.
fn polygon_meets(pts: &[[f64; 2]], view: &DeepView) -> bool {
    let (hw, hh) = (0.5 * view.size[0] + view.margin, 0.5 * view.size[1] + view.margin);
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for p in pts {
        lo = [lo[0].min(p[0]), lo[1].min(p[1])];
        hi = [hi[0].max(p[0]), hi[1].max(p[1])];
    }
    lo[0] <= hw && hi[0] >= -hw && lo[1] <= hh && hi[1] >= -hh
}

/// The size (pixels) under which the deep walk hands a subtree to the `f64` walk: see
/// [`Tables::f64_reach_log2`].
pub fn switch_px(t: &Tables) -> f64 {
    t.f64_reach_log2().min(32.0).exp2()
}

#[derive(Clone)]
struct DeepState {
    pos: Cx,
    turns: i128,
    flip: bool,
    scale: BigFloat,
    colour: i32,
    depth: u16,
}

#[derive(Clone, Copy)]
struct Frame {
    rule: u16,
    i: u32,
    child_depth: u32,
    /// The variant of the node the word was rewritten from.
    v: u32,
}

/// Walks `t` at `opts.order` over a deep `view`, through `bt` (built to at least that order), and
/// hands each segment to `sink` as the `f64` walk would — pixels from the view's centre, y up.
/// `switch_px`: subtrees reaching less than this many pixels are drawn by the `f64` walk.
pub fn deep_walk(
    t: &Tables,
    bt: &BigTables,
    view: &DeepView,
    opts: &WalkOptions,
    switch_px: f64,
    sink: &mut dyn FnMut(&Segment),
) -> WalkStats {
    deep_walk_all(t, bt, view, opts, switch_px, &mut |d| {
        if let Drawn::Segment(s) = d {
            sink(s)
        }
    })
}

/// [`deep_walk`], with the filled polygons.
pub fn deep_walk_all(
    t: &Tables,
    bt: &BigTables,
    view: &DeepView,
    opts: &WalkOptions,
    switch_px: f64,
    sink: &mut dyn FnMut(Drawn),
) -> WalkStats {
    let p = bt.prec;
    let order = opts.order.min(bt.depth);
    let inv_upp = pow2(-view.upp_log2, p);
    let px_view = View { centre: [0.0, 0.0], upp: 1.0, size: view.size, margin: view.margin };
    let to_px = |pos: &Cx| -> [f64; 2] {
        [
            to_f64(&pos[0].sub(&view.centre[0], p, RM).mul(&inv_upp, p, RM)),
            to_f64(&pos[1].sub(&view.centre[1], p, RM).mul(&inv_upp, p, RM)),
        ]
    };
    let meets = |q: [f64; 2], r: f64| -> bool {
        let dx = (q[0].abs() - 0.5 * view.size[0]).max(0.0);
        let dy = (q[1].abs() - 0.5 * view.size[1]).max(0.0);
        dx * dx + dy * dy <= r * r
    };
    // The first step: its direction is the walk's base (not, in general, a whole count of units —
    // the dragon's picture is turned by its measure); every turn after it is a count.
    let u = bt.step(t, order);
    let Some(base) = cnormalise(&u, p) else { return WalkStats::default() };
    let len = u[0].mul(&u[0], p, RM).add(&u[1].mul(&u[1], p, RM), p, RM).sqrt(p, RM);
    let dir = |turns: i128| cmul(&base, &bt.rot(turns), p);
    let mut s = DeepState { pos: czero(p), turns: 0, flip: false, scale: len, colour: 1, depth: 0 };
    let switch_log2 = switch_px.log2();
    let mut stats = WalkStats::default();
    let mut index = 0.0f64;
    let mut stack: Vec<DeepState> = Vec::new();
    // The polygons open, innermost last, their vertices in view pixels.
    let mut open: Vec<Polygon> = Vec::new();
    let mut frames = vec![Frame { rule: NONE, i: 0, child_depth: order, v: bt.variants.root }];
    let advance = |s: &mut DeepState, e: &BigEffect| {
        let v = if s.flip { conj(&e.d) } else { e.d.clone() };
        let w = cmul(&dir(s.turns), &v, p);
        s.pos = cadd(&s.pos, &cscale(&w, &s.scale, p), p);
        s.turns = (s.turns + if s.flip { -e.turns } else { e.turns }).rem_euclid(bt.modulus);
        s.flip ^= e.flip;
        s.scale = s.scale.mul(&e.scale, p, RM);
        s.colour = e.colour.apply(s.colour);
    };
    let heading = |s: &DeepState| -> f64 {
        let h = dir(s.turns);
        arg_bf(&h[0], &h[1], 64)
    };
    while let Some(f) = frames.last_mut() {
        let ops = if f.rule == NONE { &bt.axiom } else { &bt.rules[f.rule as usize] };
        let Some(op) = ops.get(f.i as usize) else {
            frames.pop();
            continue;
        };
        let j = f.i as usize;
        f.i += 1;
        let d = f.child_depth;
        let fv = f.v;
        match op {
            BigOp::Sym(c) => {
                let c = *c;
                let id = bt.id[c as usize];
                let in_poly = !open.is_empty();
                if id != NONE && d > 0 {
                    let cv = bt.variants.child(fv, j);
                    let e = bt.entry(c, d, cv);
                    stats.nodes += 1;
                    // Inside a polygon every position is a vertex: the subtree's reach is all of it.
                    let reach = if in_poly { e.reach_all } else { e.reach };
                    if reach > f64::NEG_INFINITY {
                        let r_log2 = log2_abs(&s.scale) + reach - view.upp_log2;
                        let q = to_px(&s.pos);
                        // The disc test in f64 at 2⁷³ px rounds by hundreds of pixels; near a
                        // subtree's far end its reach is exactly the distance to the view, and the
                        // subtree holding the view was culled (the Koch curve's end, at 3⁴⁰). So
                        // the test allows for its own rounding: keeping a little more costs a little.
                        let r = r_log2.exp2();
                        let slack = 1e-9 * (r + q[0].abs() + q[1].abs());
                        let seen = r_log2 > 900.0 || meets(q, r + slack + view.margin);
                        if seen && r_log2 > switch_log2 {
                            frames.push(Frame { rule: bt.word_of(id, cv), i: 0, child_depth: d - 1, v: cv });
                            continue;
                        }
                        if seen {
                            // Small enough for f64: the f64 walk draws it from here.
                            let start = SubStart {
                                sym: c,
                                depth: d,
                                variant: cv,
                                pos: q,
                                heading: heading(&s),
                                flip: s.flip,
                                scale: to_f64(&s.scale.mul(&inv_upp, p, RM)),
                                colour: s.colour,
                                brackets: s.depth,
                                index,
                            };
                            let left = opts.budget.saturating_sub(stats.segments + stats.vertices);
                            let sub = walk_from(t, &px_view, &WalkOptions { order: d, lod_px: opts.lod_px, budget: left }, &start, &mut open, sink);
                            stats.segments += sub.segments;
                            stats.polygons += sub.polygons;
                            stats.vertices += sub.vertices;
                            stats.nodes += sub.nodes;
                            if sub.stopped {
                                stats.stopped = true;
                                return stats;
                            }
                            // The f64 walk added the subtree's vertices; the end is the exact one.
                            advance(&mut s, &e.fx);
                            if !in_poly {
                                index += e.n;
                            }
                            continue;
                        }
                    }
                    advance(&mut s, &e.fx);
                    if let Some(poly) = open.last_mut() {
                        // Off the view, inside a polygon: its chord (as the f64 walk).
                        poly.pts.push(to_px(&s.pos));
                    } else {
                        index += e.n;
                    }
                } else {
                    match bt.roles[c as usize] {
                        Role::None => {}
                        role => {
                            let a = s.pos.clone();
                            s.pos = cadd(&s.pos, &cscale(&dir(s.turns), &s.scale, p), p);
                            if let Some(poly) = open.last_mut() {
                                poly.pts.push(to_px(&s.pos));
                            } else if role == Role::Draw {
                                let (qa, qb) = (to_px(&a), to_px(&s.pos));
                                let mid = [0.5 * (qa[0] + qb[0]), 0.5 * (qa[1] + qb[1])];
                                let half = 0.5 * (qb[0] - qa[0]).hypot(qb[1] - qa[1]);
                                if half.is_finite() && meets(mid, half + view.margin) {
                                    if stats.segments + stats.vertices >= opts.budget {
                                        stats.stopped = true;
                                        return stats;
                                    }
                                    let h = (heading(&s) / std::f64::consts::TAU).rem_euclid(1.0);
                                    sink(Drawn::Segment(&Segment { a: qa, b: qb, index, span: 1.0, depth: s.depth, heading: h, colour: s.colour }));
                                    stats.segments += 1;
                                }
                                index += 1.0;
                            }
                        }
                    }
                }
            }
            BigOp::PolyStart => open.push(Polygon {
                pts: vec![to_px(&s.pos)],
                index,
                depth: s.depth,
                heading: (heading(&s) / std::f64::consts::TAU).rem_euclid(1.0),
                colour: s.colour,
            }),
            BigOp::Vertex => {
                if let Some(poly) = open.last_mut() {
                    poly.pts.push(to_px(&s.pos));
                }
            }
            BigOp::PolyEnd => {
                if let Some(poly) = open.pop() {
                    let finite = poly.pts.iter().all(|q| q[0].is_finite() && q[1].is_finite());
                    if poly.pts.len() >= 3 && finite && polygon_meets(&poly.pts, view) {
                        stats.polygons += 1;
                        stats.vertices += poly.pts.len() as u64;
                        sink(Drawn::Polygon(&poly));
                    }
                }
            }
            BigOp::Turn(k) => s.turns = (s.turns + if s.flip { -k } else { *k }).rem_euclid(bt.modulus),
            BigOp::Reverse => s.flip = !s.flip,
            BigOp::Push => {
                stack.push(s.clone());
                s.depth = s.depth.saturating_add(1);
            }
            BigOp::Pop => s = stack.pop().expect("words are bracket-balanced"),
            BigOp::Scale(f) => s.scale = s.scale.mul(f, p, RM),
            BigOp::SetColour(c) => s.colour = *c,
            BigOp::AddColour(c) => s.colour = s.colour.wrapping_add(*c),
        }
    }
    stats
}

#[cfg(test)]
mod tests;
