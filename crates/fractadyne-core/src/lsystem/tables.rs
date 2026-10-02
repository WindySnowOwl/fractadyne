//! What a subtree of the derivation does, without expanding it (design/lsystems.md §4.3–4.4).
//!
//! For every symbol X with a production and every depth d, an [`Entry`]: the turtle's net motion,
//! turn, mirroring, step factor and colour change over X's d-fold rewrite ([`Effect`]), how far from
//! its start it draws (`r`), and how many segments it draws (`n`). Each row is a recurrence over the
//! productions on the row below, so building the tables to depth d costs d × the productions' length,
//! and the walk can step over a subtree of 4^60 segments in one addition.
//!
//! The tables also fix how the picture sits in the world at each order ([`Tables::step`]): the step
//! shrinks (and, for a curve such as the dragon, turns) as the order rises so the picture stays put,
//! and the order can follow the zoom ([`Tables::auto_order`]).

use super::system::{LSystem, Role, Tok, MAX_ORDER};
use std::f64::consts::{PI, TAU};

/// A turtle operation, resolved against the system's angle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Op {
    Sym(u8),
    /// Turn left by `k` steps of the division.
    Turn(i32),
    /// Turn left by radians: an angle that is not a division of the circle, or a turn that is not a
    /// whole number of its steps.
    Free(f64),
    Reverse,
    Push,
    Pop,
    Scale(f64),
    SetColour(i32),
    AddColour(i32),
    PolyStart,
    PolyEnd,
    Vertex,
}

/// The change a subtree makes to the colour index: set it (or not), then add.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColourFx {
    pub set: Option<i32>,
    pub add: i32,
}

impl ColourFx {
    pub fn apply(self, c: i32) -> i32 {
        self.set.unwrap_or(c).wrapping_add(self.add)
    }

    /// This change, then `b`.
    pub fn then(self, b: ColourFx) -> ColourFx {
        if b.set.is_some() {
            b
        } else {
            ColourFx { set: self.set, add: self.add.wrapping_add(b.add) }
        }
    }
}

/// What running a word does to the turtle, in the frame it started in: heading 0 (+x), not
/// reversed, step 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Effect {
    /// Where the turtle ends up.
    pub d: [f64; 2],
    /// The net turn in steps of the division (0 without one), in `0..n`.
    pub turns: i64,
    /// The net turn in radians not counted by `turns`, in `[0, τ)`.
    pub free: f64,
    /// Whether left and right end up swapped.
    pub flip: bool,
    /// The net step factor.
    pub scale: f64,
    pub colour: ColourFx,
}

impl Effect {
    pub const IDENTITY: Effect =
        Effect { d: [0.0, 0.0], turns: 0, free: 0.0, flip: false, scale: 1.0, colour: ColourFx { set: None, add: 0 } };
}

/// A subtree, summarised.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entry {
    pub fx: Effect,
    /// How far from its start the subtree draws — lines and polygons — in its own steps (−1: it
    /// draws nothing).
    pub r: f64,
    /// How far from its start the turtle goes at all, drawing or not (≥ 0): inside a polygon every
    /// position is a vertex, so this is the reach a subtree has there.
    pub ra: f64,
    /// How many segments (lines) it draws.
    pub n: f64,
    /// The work of drawing it whole inside a polygon: every step a vertex, and its own polygons'
    /// start vertices and `.` vertices besides.
    pub wp: f64,
    /// The work of drawing it whole: lines and polygon vertices.
    pub w: f64,
}

impl Entry {
    const NOTHING: Entry = Entry { fx: Effect::IDENTITY, r: -1.0, ra: 0.0, n: 0.0, wp: 0.0, w: 0.0 };
}

/// No production (a symbol's id).
pub(crate) const NONE: u16 = u16::MAX;

/// Past this, a table value is too large for `f64` arithmetic on it to stay meaningful; the tables
/// stop one depth short of it.
const LIMIT: f64 = 1e250;

/// Turn counts kept exact without a division: `−DIRS..=DIRS` steps come from a table.
const DIRS: i64 = 256;

/// The most segments per square step of the picture's bounding box the order follows the zoom to
/// ([`Tables::density`]). Measured over the library: the plane-filling curves sit at about 1 at
/// every order (Hilbert, Peano, Moore, the quadratic Gosper 1.00; Gosper 0.75; the terdragon
/// 1.23; Cross 1.72), the others below; a curve that overlaps itself climbs with every order
/// (Tiles 1.6, 3.1, 12, 46 at orders 1, 3, 7, 11; ABOP's plant c doubles an order) — past this it
/// is a solid blob, and more order is cost with nothing to show.
pub const MAX_DENSITY: f64 = 3.0;

/// A system's tables, to [`Tables::max_depth`].
#[derive(Clone, Debug)]
pub struct Tables {
    /// The division of the circle the turns count in (`None`: turns are radians).
    pub(crate) division: Option<u32>,
    /// One turn step, radians.
    pub(crate) delta: f64,
    /// Unit vectors: with a division, step k at `dirs[k]`; without, step k at `dirs[k + DIRS]`.
    dirs: Vec<[f64; 2]>,
    pub(crate) axiom: Vec<Op>,
    pub(crate) rules: Vec<Vec<Op>>,
    pub(crate) id: [u16; 256],
    pub(crate) roles: [Role; 256],
    /// What each symbol does when read, not rewritten.
    constant: [Entry; 256],
    /// `rows[d][id]` for `d` in `1..=max_depth` (`rows[0]` is empty).
    rows: Vec<Vec<Entry>>,
    /// The deepest table row.
    pub max_depth: u32,
    /// The first heading, radians.
    pub(crate) heading: f64,
    /// The picture's size measure at each depth up to [`Tables::along_depth`] (see
    /// [`Tables::step`]), as a vector…
    measure: Vec<[f64; 2]>,
    /// …the displacement of this symbol (`None`: the axiom's reach measures it instead).
    pub(crate) measure_sym: Option<u8>,
    /// Bits of a table entry's precision lost per level of depth: a subtree's displacement is a
    /// sum of its children's that cancel (the Sierpinski triangle's `F` advances 2 steps with 5
    /// steps of children), so rounding grows by the ratio a level. 0 when the arithmetic is exact
    /// (every turn a quarter turn, no step factors). The tables are good to `2^(d·loss − 52)` at
    /// depth `d`: past the depth where that matters, the deep walk takes over.
    pub loss: f64,
    /// The depth the picture's orientation and growth are taken from — moderate, so its value is
    /// reliable — and the direction of the measure there, exact (computed in `BigFloat`).
    pub along_depth: u32,
    along: [f64; 2],
    /// The measure's turn per order, radians (the dragon's is 45°): the step past `along_depth`.
    turn_per_order: f64,
    /// How much the picture grows per order (1: it does not).
    pub growth: f64,
    /// 2 for a picture that mirrors from one order to the next (the Sierpinski arrowhead lies on
    /// alternate sides of its base): its order steps by two, keeping the deepest row's phase.
    pub period: u32,
    /// The area of the picture's bounding box at each order up to one cheap to walk whole, world
    /// units (a later order takes the last); empty for a picture that does not grow. Each order's
    /// own: a plant whose leaves keep their size while its stem doubles is wide at order 3 and a
    /// line at order 13, and measured by the line it was capped at order 0 ([`Tables::density`]).
    pub box_areas: Vec<f64>,
    /// The longer side of the last of those boxes, world units (0 for a picture that does not
    /// grow): how large the picture is.
    pub box_size: f64,
}

fn mul(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] * b[0] - a[1] * b[1], a[0] * b[1] + a[1] * b[0]]
}

fn div(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    let m = b[0] * b[0] + b[1] * b[1];
    [(a[0] * b[0] + a[1] * b[1]) / m, (a[1] * b[0] - a[0] * b[1]) / m]
}

pub(crate) fn norm(a: [f64; 2]) -> f64 {
    a[0].hypot(a[1])
}

fn wrap(a: f64) -> f64 {
    let w = a.rem_euclid(TAU);
    if w >= TAU {
        0.0
    } else {
        w
    }
}

impl Tables {
    /// Compiles `sys` and builds its tables as deep as `f64` allows (at most [`MAX_ORDER`]).
    pub fn new(sys: &LSystem) -> Tables {
        Self::with_depth(sys, MAX_ORDER)
    }

    /// As [`Tables::new`], to at most `depth`.
    pub fn with_depth(sys: &LSystem, depth: u32) -> Tables {
        let division = sys.angle.division();
        let delta = sys.angle.degrees().to_radians();
        // Exact where the value is exact (cos 90° is 0, not 6e-17; cos 60° is 0.5, not
        // 0.5000000000000001): the 90° and 60° curves then sum exactly, as the deep walk does.
        let snap = |v: [f64; 2]| v.map(|c| [0.0, 0.5, -0.5, 1.0, -1.0].into_iter().find(|&e| (c - e).abs() < 1e-15).unwrap_or(c));
        let dirs: Vec<[f64; 2]> = match division {
            Some(n) => (0..n as i64).map(|k| snap(unit(TAU * k as f64 / n as f64))).collect(),
            None => (-DIRS..=DIRS).map(|k| snap(unit(k as f64 * delta))).collect(),
        };
        let compile = |w: &[Tok]| -> Vec<Op> {
            w.iter()
                .map(|&t| match t {
                    Tok::Sym(c) => Op::Sym(c),
                    Tok::Turn(k) => match division {
                        Some(_) => Op::Turn(k as i32),
                        None => Op::Free(k as f64 * delta),
                    },
                    Tok::Around => match division {
                        Some(n) => Op::Turn((n / 2) as i32),
                        None => Op::Free(PI),
                    },
                    Tok::TurnBy(a) => {
                        let steps = division.map(|n| a * n as f64 / 360.0);
                        match steps {
                            Some(s) if (s - s.round()).abs() < 1e-9 && s.abs() < i32::MAX as f64 => {
                                Op::Turn(s.round() as i32)
                            }
                            _ => Op::Free(a.to_radians()),
                        }
                    }
                    Tok::Reverse => Op::Reverse,
                    Tok::Push => Op::Push,
                    Tok::Pop => Op::Pop,
                    Tok::Scale(f) => Op::Scale(f),
                    Tok::SetColour(c) => Op::SetColour(c),
                    Tok::AddColour(c) => Op::AddColour(c),
                    Tok::PolyStart => Op::PolyStart,
                    Tok::PolyEnd => Op::PolyEnd,
                    Tok::Vertex => Op::Vertex,
                })
                .collect()
        };
        let mut id = [NONE; 256];
        let mut rules = Vec::new();
        for c in 0..=255u8 {
            if let Some(w) = sys.rule(c) {
                id[c as usize] = rules.len() as u16;
                rules.push(compile(w));
            }
        }
        let mut roles = [Role::None; 256];
        let mut constant = [Entry::NOTHING; 256];
        for c in 0..256 {
            roles[c] = sys.roles[c];
            constant[c] = match sys.roles[c] {
                Role::Draw => Entry { fx: Effect { d: [1.0, 0.0], ..Effect::IDENTITY }, r: 1.0, ra: 1.0, n: 1.0, wp: 1.0, w: 1.0 },
                Role::Move => Entry { fx: Effect { d: [1.0, 0.0], ..Effect::IDENTITY }, r: -1.0, ra: 1.0, n: 0.0, wp: 1.0, w: 0.0 },
                Role::None => Entry::NOTHING,
            };
        }
        let mut t = Tables {
            division,
            delta,
            dirs,
            axiom: compile(&sys.axiom),
            rules,
            id,
            roles,
            constant,
            rows: vec![Vec::new()],
            max_depth: 0,
            heading: sys.heading.to_radians(),
            measure: Vec::new(),
            measure_sym: None,
            loss: 0.0,
            along_depth: 0,
            along: [1.0, 0.0],
            turn_per_order: 0.0,
            growth: 1.0,
            period: 1,
            box_areas: Vec::new(),
            box_size: 0.0,
        };
        for d in 1..=depth.min(MAX_ORDER) {
            let row: Vec<Entry> = (0..t.rules.len()).map(|k| t.fold(&t.rules[k], d - 1)).collect();
            let sane = row.iter().all(|e| {
                norm(e.fx.d) < LIMIT && e.ra < LIMIT && e.wp < LIMIT && e.fx.scale < LIMIT && e.fx.scale > 1.0 / LIMIT
            });
            if !sane {
                break;
            }
            t.rows.push(row);
            t.max_depth = d;
        }
        t.size_the_picture(sys);
        (t.period, t.box_areas, t.box_size) = super::walk::shape(&t);
        t
    }

    /// The highest order at most `n` in the deepest row's phase (see [`Tables::period`]).
    pub fn in_phase(&self, n: u32) -> u32 {
        let n = n.min(self.max_depth);
        let off = (self.max_depth - n) % self.period;
        if off <= n {
            n - off
        } else {
            n
        }
    }

    /// The entry for symbol `c` rewritten `d` times.
    pub fn entry(&self, c: u8, d: u32) -> Entry {
        let id = self.id[c as usize];
        if id == NONE || d == 0 {
            self.constant[c as usize]
        } else {
            self.rows[d as usize][id as usize]
        }
    }

    /// The axiom rewritten `order` times.
    pub fn axiom_entry(&self, order: u32) -> Entry {
        self.fold(&self.axiom, order)
    }

    /// The unit vector of a heading.
    pub(crate) fn dir(&self, turns: i64, free: [f64; 2]) -> [f64; 2] {
        let base = match self.division {
            Some(_) => self.dirs[turns as usize],
            None if turns.abs() <= DIRS => self.dirs[(turns + DIRS) as usize],
            None => unit(turns as f64 * self.delta),
        };
        mul(base, free)
    }

    /// `turns + k` in the division (or plain, without one).
    pub(crate) fn add_turns(&self, turns: i64, k: i64) -> i64 {
        match self.division {
            Some(n) => (turns + k).rem_euclid(n as i64),
            None => turns.saturating_add(k),
        }
    }

    /// `a`, then `b` in the frame `a` leaves the turtle in.
    pub fn then(&self, a: &Effect, b: &Effect) -> Effect {
        let v = if a.flip { [b.d[0], -b.d[1]] } else { b.d };
        let w = mul(self.dir(a.turns, unit(a.free)), v);
        let sign = if a.flip { -1 } else { 1 };
        let mut turns = self.add_turns(a.turns, sign * b.turns);
        let mut free = a.free + sign as f64 * b.free;
        // Without a division a turn count can grow without bound (`X → X+X+` doubles it); past the
        // exact range it becomes radians.
        if self.division.is_none() && turns.abs() > 1 << 40 {
            free += turns as f64 * self.delta;
            turns = 0;
        }
        Effect {
            d: [a.d[0] + a.scale * w[0], a.d[1] + a.scale * w[1]],
            turns,
            free: wrap(free),
            flip: a.flip ^ b.flip,
            scale: a.scale * b.scale,
            colour: a.colour.then(b.colour),
        }
    }

    /// A word whose symbols are rewritten `d` times, summarised.
    fn fold(&self, ops: &[Op], d: u32) -> Entry {
        let mut st = Effect::IDENTITY;
        let mut stack: Vec<Effect> = Vec::new();
        let (mut r, mut ra, mut n, mut wp, mut w) = (-1.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
        // Inside `{ }` (braces close in the word that opens them).
        let mut poly = false;
        for &op in ops {
            match op {
                Op::Sym(c) => {
                    let e = self.entry(c, d);
                    let from = norm(st.d);
                    ra = ra.max(from + st.scale * e.ra);
                    wp += e.wp;
                    if poly {
                        // Every position a vertex; no lines.
                        r = r.max(from + st.scale * e.ra);
                        w += e.wp;
                    } else {
                        if e.r >= 0.0 {
                            r = r.max(from + st.scale * e.r);
                            n += e.n;
                        }
                        w += e.w;
                    }
                    st = self.then(&st, &e.fx);
                }
                Op::PolyStart => {
                    poly = true;
                    r = r.max(norm(st.d));
                    w += 1.0;
                    wp += 1.0;
                }
                Op::PolyEnd => poly = false,
                Op::Vertex => {
                    // A `.` adds a vertex to whichever polygon is open — this word's, or, when the
                    // word itself sits inside a polygon, that one.
                    wp += 1.0;
                    if poly {
                        r = r.max(norm(st.d));
                        w += 1.0;
                    }
                }
                Op::Turn(k) => {
                    let k = if st.flip { -(k as i64) } else { k as i64 };
                    st.turns = self.add_turns(st.turns, k);
                }
                Op::Free(a) => st.free = wrap(st.free + if st.flip { -a } else { a }),
                Op::Reverse => st.flip = !st.flip,
                Op::Push => stack.push(st),
                Op::Pop => st = stack.pop().expect("words are bracket-balanced"),
                Op::Scale(f) => st.scale *= f,
                Op::SetColour(c) => st.colour = st.colour.then(ColourFx { set: Some(c), add: 0 }),
                Op::AddColour(c) => st.colour = st.colour.then(ColourFx { set: None, add: c }),
            }
        }
        Entry { fx: st, r, ra, n, wp, w }
    }

    /// Picks the picture's size measure and growth (see [`Tables::step`]).
    fn size_the_picture(&mut self, sys: &LSystem) {
        let top = self.max_depth;
        // The symbols the axiom's derivation reaches.
        let mut reach = [false; 256];
        let mut todo: Vec<u8> = sys.axiom.iter().filter_map(|t| if let Tok::Sym(c) = t { Some(*c) } else { None }).collect();
        while let Some(c) = todo.pop() {
            if std::mem::replace(&mut reach[c as usize], true) {
                continue;
            }
            for t in sys.rule(c).unwrap_or(&[]) {
                if let Tok::Sym(x) = t {
                    todo.push(*x);
                }
            }
        }
        self.loss = self.precision_loss(sys, &reach);
        // A moderate depth, where the tables still hold ~22 good bits (and the BigFloat measure
        // below is cheap): the picture's measure, orientation and growth are taken there. The
        // deepest row is no place for them — at depth 415 the Sierpinski triangle's tables had
        // lost everything (its `F` measured vertical, its growth 3 for 2).
        let dm = if self.loss > 0.0 { ((30.0 / self.loss) as u32).clamp(8, 64) } else { 64 }.min(top);
        self.along_depth = dm;
        // The reached symbol that travels furthest there measures the picture: its displacement,
        // as a vector, carries the curve's growth AND its turn per order (the dragon turns 45° an
        // order). A picture whose symbols all return to their start (closed loops) is measured by
        // how far the axiom draws instead.
        let best = (0..256)
            .filter(|&c| reach[c] && self.id[c] != NONE)
            .map(|c| (c as u8, norm(self.entry(c as u8, dm).fx.d)))
            .filter(|&(_, m)| m > 0.0)
            .max_by(|a, b| a.1.total_cmp(&b.1));
        self.measure_sym = best.map(|b| b.0);
        self.measure = (0..=dm)
            .map(|d| match best {
                Some((c, _)) => self.entry(c, d).fx.d,
                None => [self.axiom_entry(d).r.max(0.0), 0.0],
            })
            .collect();
        // The measure there, exact: its direction (the orientation — the deep walk computes the
        // same direction at its own precision, so the two walks agree), its growth from two orders
        // apart (so a picture that alternates between two shapes measures its true rate), and its
        // turn per order.
        let exact = best.filter(|_| dm >= 2).and_then(|(c, _)| {
            let prec = 128 + (f64::from(dm) * self.loss).ceil() as usize;
            super::deep::exact_measure(sys, c, dm, prec)
        });
        let m = |d: u32| self.measure[d as usize];
        match exact {
            Some(e) => {
                self.along = e.along;
                self.growth = ((e.log2_norm[0] - e.log2_norm[2]) / 2.0).exp2();
                self.turn_per_order = e.turn;
            }
            None if dm >= 2 && norm(m(dm)) > 0.0 && norm(m(dm - 2)) > 0.0 => {
                let n = norm(m(dm));
                self.along = [m(dm)[0] / n, m(dm)[1] / n];
                self.growth = (n / norm(m(dm - 2))).sqrt();
                let r = div(m(dm), m(dm - 1));
                self.turn_per_order = r[1].atan2(r[0]);
            }
            None => self.growth = 1.0,
        }
        // Growth by a factor, or by a step a generation? A stem that lengthens by one segment an
        // order (Saupe's bush) measures 1.06 at depth 32 and 1.03 at 64 — falling towards 1 — where
        // a factor measures the same at both. Only a factor lets the order follow the zoom.
        let half = dm / 2;
        if half >= 2 && norm(m(half)) > 0.0 && norm(m(half - 2)) > 0.0 {
            let g_half = (norm(m(half)) / norm(m(half - 2))).sqrt();
            if (g_half - self.growth).abs() > 0.25 * (self.growth - 1.0).abs() + 1e-9 {
                self.growth = 1.0;
            }
        }
    }

    /// Bits lost per level (see [`Tables::loss`]): the most any production's children reach, over
    /// what it reaches, at a moderate depth — 0 when every turn is a quarter turn and there are no
    /// step factors (then the sums are of integers, exact in `f64`).
    fn precision_loss(&self, sys: &LSystem, reach: &[bool; 256]) -> f64 {
        let quarter = |deg: f64| (deg / 90.0 - (deg / 90.0).round()).abs() < 1e-12;
        let words = || std::iter::once(&sys.axiom).chain(sys.rules.iter().flatten());
        let exact = quarter(sys.angle.degrees())
            && words().flatten().all(|t| match t {
                Tok::TurnBy(a) => quarter(*a),
                Tok::Scale(_) => false,
                _ => true,
            });
        if exact || self.max_depth == 0 {
            return 0.0;
        }
        let d0 = self.max_depth.min(12);
        let mut worst = 0.0f64;
        for c in (0..256).filter(|&c| reach[c] && self.id[c] != NONE) {
            let r = self.entry(c as u8, d0).r;
            if r <= 0.0 {
                continue;
            }
            // The children's reach, each at the step factor it is drawn with.
            let (mut scale, mut stack, mut sum) = (1.0f64, Vec::new(), 0.0f64);
            for &op in &self.rules[self.id[c] as usize] {
                match op {
                    Op::Sym(y) => sum += self.entry(y, d0 - 1).r.max(0.0) * scale,
                    Op::Scale(f) => scale *= f,
                    Op::Push => stack.push(scale),
                    Op::Pop => scale = stack.pop().unwrap_or(scale),
                    _ => {}
                }
            }
            worst = worst.max((sum / r).log2());
        }
        worst.clamp(0.0, 8.0)
    }

    /// Whether the picture grows by a factor per order (so the order can follow the zoom). A
    /// picture that grows by a step per order (a stem that lengthens by one segment) changes its
    /// proportions as it grows, and is drawn at a fixed order instead.
    pub fn grows(&self) -> bool {
        self.growth > 1.01
    }

    /// The turtle's first step at `order`, as a world vector (length and direction).
    ///
    /// The step is the reciprocal of the picture's measure at that order, turned to the measure's
    /// direction at the deepest row: so the measured symbol spans the same world vector at every
    /// order, and the picture neither grows nor spins as the order rises — exactly, for a curve
    /// whose vertices persist from order to order (Koch); to within about a step for one whose do
    /// not (Hilbert). A picture that does not grow keeps a step of 1.
    pub fn step(&self, order: u32) -> [f64; 2] {
        let h = unit(self.heading);
        if !self.grows() {
            return h;
        }
        let dm = self.along_depth;
        let top = self.measure[dm as usize];
        if order > dm {
            // Past the moderate depth, the growth law (and the measure's turn per order): this is
            // for choosing an order and framing; the deep walk draws there, from exact tables.
            let k = f64::from(order - dm);
            let s = self.growth.powf(-k);
            let base = self.step(dm);
            let r = unit(-self.turn_per_order * k);
            return mul(base, [r[0] * s, r[1] * s]);
        }
        let m = self.measure[order as usize];
        // What the growth law says the measure is at this order: |m(dm)| / g^(dm − order).
        let expect = (norm(top).ln() - f64::from(dm - order) * self.growth.ln()).exp();
        if norm(m) >= 0.25 * expect {
            mul(h, div(self.along, m))
        } else {
            // The measured symbol draws nothing yet (the dragon's X at order 0), or next to
            // nothing: the growth law sizes the step instead.
            let s = 1.0 / expect;
            mul(h, [self.along[0] * s, self.along[1] * s])
        }
    }

    /// `log₂` of the step's length at `order`, at any order up to [`MAX_ORDER`].
    pub fn step_log2(&self, order: u32) -> f64 {
        if order <= self.along_depth || !self.grows() {
            return norm(self.step(order)).log2();
        }
        norm(self.step(self.along_depth)).log2() - f64::from(order - self.along_depth) * self.growth.log2()
    }

    /// How large (`log₂` pixels) a subtree the `f64` walk may place to ~2⁻¹² px: the deeper the
    /// subtree, the more of `f64`'s bits its tables have lost ([`Tables::loss`]). The app walks in
    /// `f64` while the whole picture is under this, and the deep walk hands subtrees under it to
    /// the `f64` walk.
    pub fn f64_reach_log2(&self) -> f64 {
        if !self.grows() {
            return 40.0;
        }
        (40.0 / (1.0 + self.loss / self.growth.log2())).clamp(8.0, 40.0)
    }

    /// [`Tables::auto_order`] for any zoom: `log2_px_per_unit` may be far past `f64` (a view at
    /// 1e1000×), and the order past the tables' depth (up to [`MAX_ORDER`]), where the deep walk
    /// draws it.
    pub fn auto_order_log2(&self, log2_px_per_unit: f64, step_px: f64) -> Option<u32> {
        if !self.grows() {
            return None;
        }
        let want = step_px.log2() - log2_px_per_unit;
        if self.step_log2(self.max_depth) <= want {
            return self.auto_order(log2_px_per_unit.exp2(), step_px);
        }
        // The growth law: each order shrinks the step by the growth factor.
        let past = ((self.step_log2(self.max_depth) - want) / self.growth.log2()).ceil() as u32;
        let n = self.max_depth.saturating_add(past).min(MAX_ORDER);
        let n = n - (n - self.max_depth) % self.period;
        // Past the tables, an overlapping curve was capped long before: the cap's order stands.
        let cap = self.auto_order(f64::MAX, step_px).unwrap_or(n);
        Some(if cap < self.max_depth { cap } else { n })
    }

    /// The order at which the step is at most `step_px` pixels, at `px_per_unit` pixels per world
    /// unit: the order that follows the zoom. `None` for a picture that does not grow.
    ///
    /// A curve that overlaps itself gets denser with every order (Tiles: seven segments to a
    /// factor of √5 in size): past [`MAX_DENSITY`], more order adds cost and no picture, so the
    /// order stops there — at any zoom.
    pub fn auto_order(&self, px_per_unit: f64, step_px: f64) -> Option<u32> {
        if !self.grows() {
            return None;
        }
        let mut n = (0..=self.max_depth)
            .filter(|&n| (self.max_depth - n).is_multiple_of(self.period))
            .find(|&n| norm(self.step(n)) * px_per_unit <= step_px)
            .unwrap_or(self.max_depth);
        while n >= self.period && self.density(n) > MAX_DENSITY {
            n -= self.period;
        }
        Some(n)
    }

    /// Segments per square step of the picture's bounding box at `order` (0 without a box).
    pub fn density(&self, order: u32) -> f64 {
        let area = self.box_areas.get(order as usize).or(self.box_areas.last()).copied().unwrap_or(0.0);
        if area <= 0.0 {
            return 0.0;
        }
        let s = norm(self.step(order));
        self.axiom_entry(order).n * s * s / area
    }
}

pub(crate) fn unit(a: f64) -> [f64; 2] {
    let (s, c) = a.sin_cos();
    [c, s]
}

#[cfg(test)]
mod tests;
