//! The walk (design/lsystems.md §4.5): depth first through the derivation tree, never building the
//! word. Before descending into a subtree it asks the [`Tables`]: a subtree whose reach misses the
//! view is stepped over in one addition, and one smaller than a pixel is drawn as its chord. So the
//! segments a view gets are bounded by its pixels, whatever the order.

use super::system::Role;
use super::tables::{norm, unit, Op, Tables, NONE};
use std::f64::consts::TAU;

/// A view of the world: what the walk culls to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    /// The view's centre, world units.
    pub centre: [f64; 2],
    /// World units per pixel.
    pub upp: f64,
    /// The view's size, pixels.
    pub size: [f64; 2],
    /// How far outside the view a segment still counts, pixels (half the line width, and the
    /// filter's reach).
    pub margin: f64,
}

impl View {
    /// A view that sees everything, in world units (centre 0, a pixel per unit): for a whole
    /// picture.
    pub const EVERYTHING: View = View { centre: [0.0, 0.0], upp: 1.0, size: [f64::INFINITY; 2], margin: 0.0 };

    /// Whether a disc at `p` (pixels from the centre) of radius `r` meets the view.
    pub(crate) fn meets(&self, p: [f64; 2], r: f64) -> bool {
        let dx = (p[0].abs() - 0.5 * self.size[0]).max(0.0);
        let dy = (p[1].abs() - 0.5 * self.size[1]).max(0.0);
        dx * dx + dy * dy <= r * r
    }
}

/// One segment to draw.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    /// The ends, pixels from the view's centre (y up).
    pub a: [f64; 2],
    pub b: [f64; 2],
    /// Where along the curve it is: the index of the first segment it stands for…
    pub index: f64,
    /// …and how many it stands for (1; or a whole subtree under a pixel, drawn as its chord).
    pub span: f64,
    /// How many brackets it is inside.
    pub depth: u16,
    /// The direction the turtle faced, in turns, `[0, 1)`.
    pub heading: f64,
    /// The colour index.
    pub colour: i32,
}

/// What to walk.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WalkOptions {
    pub order: u32,
    /// A subtree reaching less than this many pixels is drawn as its chord (0: never).
    pub lod_px: f64,
    /// Stop after this many segments.
    pub budget: u64,
}

/// What a walk did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WalkStats {
    pub segments: u64,
    /// Filled polygons drawn, and their vertices (the budget counts these with the segments).
    pub polygons: u64,
    pub vertices: u64,
    /// Subtrees looked at (descended into, stepped over or drawn as a chord).
    pub nodes: u64,
    /// Whether it stopped at the budget, with segments left to draw.
    pub stopped: bool,
}

/// A filled polygon (`{ … }`): its vertices in the view's pixels (y up, from its centre), and
/// what colours it, as of where it started.
#[derive(Clone, Debug, PartialEq)]
pub struct Polygon {
    pub pts: Vec<[f64; 2]>,
    /// The number of lines drawn before it (its place along the curve).
    pub index: f64,
    pub depth: u16,
    pub heading: f64,
    pub colour: i32,
}

/// What a walk draws.
#[derive(Clone, Copy, Debug)]
pub enum Drawn<'a> {
    Segment(&'a Segment),
    Polygon(&'a Polygon),
}

#[derive(Clone, Copy)]
struct State {
    pos: [f64; 2],
    turns: i64,
    free: f64,
    /// `unit(free)`, kept with it.
    fr: [f64; 2],
    flip: bool,
    /// The step, in pixels.
    scale: f64,
    colour: i32,
    depth: u16,
}

const AXIOM: u16 = NONE;

#[derive(Clone, Copy)]
struct Frame {
    /// The word (an index into the tables' words, or `AXIOM` / `ONE`).
    rule: u16,
    i: u32,
    /// The depth of this word's symbols (how many more times each rewrites).
    child_depth: u32,
    /// The variant of the node the word was rewritten from (the `ONE` word's: its symbol's own).
    v: u32,
}

/// The single-symbol word a [`walk_from`] starts with.
const ONE: u16 = NONE - 1;

/// Where a [`walk_from`] starts: one symbol rewritten `depth` times, with the turtle as given (in
/// the view's pixels, y up, from its centre). The deep walk hands each subtree small enough for
/// `f64` to one of these.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SubStart {
    pub sym: u8,
    pub depth: u32,
    /// Its variant (0 for a deterministic system).
    pub variant: u32,
    pub pos: [f64; 2],
    /// The turtle's heading, radians.
    pub heading: f64,
    pub flip: bool,
    /// The step, pixels.
    pub scale: f64,
    pub colour: i32,
    /// How many brackets the subtree sits inside.
    pub brackets: u16,
    /// The index along the curve of its first segment.
    pub index: f64,
}

/// Walks `t` at `opts.order` over `view`, handing each segment to `sink` in curve order (filled
/// polygons are left out: [`walk_all`] has them).
pub fn walk(t: &Tables, view: &View, opts: &WalkOptions, sink: &mut dyn FnMut(&Segment)) -> WalkStats {
    walk_all(t, view, opts, &mut |d| {
        if let Drawn::Segment(s) = d {
            sink(s)
        }
    })
}

/// [`walk`], with the filled polygons: each handed over when it closes.
pub fn walk_all(t: &Tables, view: &View, opts: &WalkOptions, sink: &mut dyn FnMut(Drawn)) -> WalkStats {
    let order = opts.order.min(t.max_depth);
    let u = t.step(order);
    let free = u[1].atan2(u[0]).rem_euclid(TAU);
    let s = State {
        pos: [-view.centre[0] / view.upp, -view.centre[1] / view.upp],
        turns: 0,
        free,
        fr: unit(free),
        flip: false,
        scale: norm(u) / view.upp,
        colour: 1,
        depth: 0,
    };
    run(t, view, opts, s, Frame { rule: AXIOM, i: 0, child_depth: order, v: t.variants.root }, 0, 0.0, &mut Vec::new(), sink)
}

/// Walks one subtree, from a turtle already placed (see [`SubStart`]). `open`: the polygons open
/// where the subtree starts — its steps add their vertices (and it leaves them open, since braces
/// close in the word that opens them).
pub fn walk_from(
    t: &Tables,
    view: &View,
    opts: &WalkOptions,
    start: &SubStart,
    open: &mut Vec<Polygon>,
    sink: &mut dyn FnMut(Drawn),
) -> WalkStats {
    let free = start.heading.rem_euclid(TAU);
    let s = State {
        pos: start.pos,
        turns: 0,
        free,
        fr: unit(free),
        flip: start.flip,
        scale: start.scale,
        colour: start.colour,
        depth: start.brackets,
    };
    let depth = start.depth.min(t.max_depth);
    let first = Frame { rule: ONE, i: 0, child_depth: depth, v: start.variant };
    run(t, view, opts, s, first, start.sym, start.index, open, sink)
}

/// Whether a polygon's box meets the view (with its margin).
fn polygon_seen(view: &View, pts: &[[f64; 2]]) -> bool {
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for p in pts {
        lo = [lo[0].min(p[0]), lo[1].min(p[1])];
        hi = [hi[0].max(p[0]), hi[1].max(p[1])];
    }
    let (hw, hh) = (0.5 * view.size[0] + view.margin, 0.5 * view.size[1] + view.margin);
    lo[0] <= hw && hi[0] >= -hw && lo[1] <= hh && hi[1] >= -hh
}

#[allow(clippy::too_many_arguments)]
fn run(
    t: &Tables,
    view: &View,
    opts: &WalkOptions,
    mut s: State,
    first: Frame,
    one: u8,
    mut index: f64,
    open: &mut Vec<Polygon>,
    sink: &mut dyn FnMut(Drawn),
) -> WalkStats {
    let mut stats = WalkStats::default();
    // How deep `open` was when this walk began: polygons below it belong to the caller.
    let base = open.len();
    let mut stack: Vec<State> = Vec::new();
    let mut frames = vec![first];
    let single = [Op::Sym(one)];
    let heading = |s: &State| -> f64 {
        let a = s.turns as f64 * t.delta + s.free;
        (a / TAU).rem_euclid(1.0)
    };
    while let Some(f) = frames.last_mut() {
        let ops: &[Op] = match f.rule {
            AXIOM => &t.axiom,
            ONE => &single,
            r => &t.rules[r as usize],
        };
        let Some(&op) = ops.get(f.i as usize) else {
            frames.pop();
            continue;
        };
        let j = f.i as usize;
        f.i += 1;
        let d = f.child_depth;
        // The variant of a symbol at place `j` (the `ONE` word's symbol has its own).
        let (fv, one_word) = (f.v, f.rule == ONE);
        match op {
            Op::Sym(c) => {
                let id = t.id[c as usize];
                let in_poly = !open.is_empty();
                if id != NONE && d > 0 {
                    let cv = if one_word { fv } else { t.variants.child(fv, j) };
                    let e = t.entry(c, d, cv);
                    stats.nodes += 1;
                    // Inside a polygon every position is a vertex: the subtree's reach is all of it.
                    let r = if in_poly { e.ra } else { e.r };
                    if r >= 0.0 {
                        let r_px = s.scale * r;
                        let seen = view.meets(s.pos, r_px + view.margin);
                        if seen && r_px > opts.lod_px {
                            frames.push(Frame { rule: t.word_of(id, cv), i: 0, child_depth: d - 1, v: cv });
                            continue;
                        }
                        if seen && !in_poly {
                            let a = s.pos;
                            let (h, colour, depth) = (heading(&s), s.colour, s.depth);
                            apply(t, &mut s, &e.fx);
                            if stats.segments + stats.vertices >= opts.budget {
                                stats.stopped = true;
                                return stats;
                            }
                            sink(Drawn::Segment(&Segment { a, b: s.pos, index, span: e.n, depth, heading: h, colour }));
                            stats.segments += 1;
                            index += e.n;
                            continue;
                        }
                    }
                    apply(t, &mut s, &e.fx);
                    if let Some(p) = open.last_mut() {
                        // Off the view or under a pixel, inside a polygon: its chord. (Off the
                        // view, the chord and the path it stands for lie in a disc that misses the
                        // view, so the fill inside the view is the same.)
                        p.pts.push(s.pos);
                    } else {
                        index += e.n;
                    }
                } else {
                    match t.roles[c as usize] {
                        Role::None => {}
                        role => {
                            let dir = t.dir(s.turns, s.fr);
                            let v = [dir[0] * s.scale, dir[1] * s.scale];
                            let a = s.pos;
                            s.pos = [a[0] + v[0], a[1] + v[1]];
                            if let Some(p) = open.last_mut() {
                                // Inside a polygon a step is a vertex, drawn or not, and no line.
                                p.pts.push(s.pos);
                            } else if role == Role::Draw {
                                let mid = [a[0] + 0.5 * v[0], a[1] + 0.5 * v[1]];
                                if view.meets(mid, 0.5 * s.scale + view.margin) {
                                    if stats.segments + stats.vertices >= opts.budget {
                                        stats.stopped = true;
                                        return stats;
                                    }
                                    sink(Drawn::Segment(&Segment {
                                        a,
                                        b: s.pos,
                                        index,
                                        span: 1.0,
                                        depth: s.depth,
                                        heading: heading(&s),
                                        colour: s.colour,
                                    }));
                                    stats.segments += 1;
                                }
                                index += 1.0;
                            }
                        }
                    }
                }
            }
            Op::PolyStart => open.push(Polygon { pts: vec![s.pos], index, depth: s.depth, heading: heading(&s), colour: s.colour }),
            Op::Vertex => {
                if let Some(p) = open.last_mut() {
                    p.pts.push(s.pos);
                }
            }
            Op::PolyEnd => {
                if open.len() > base {
                    let p = open.pop().expect("an open polygon");
                    if p.pts.len() >= 3 && polygon_seen(view, &p.pts) {
                        if stats.segments + stats.vertices >= opts.budget {
                            stats.stopped = true;
                            return stats;
                        }
                        stats.polygons += 1;
                        stats.vertices += p.pts.len() as u64;
                        sink(Drawn::Polygon(&p));
                    }
                }
            }
            Op::Turn(k) => {
                let k = if s.flip { -(k as i64) } else { k as i64 };
                s.turns = t.add_turns(s.turns, k);
            }
            Op::Free(a) => {
                s.free = (s.free + if s.flip { -a } else { a }).rem_euclid(TAU);
                s.fr = unit(s.free);
            }
            Op::Reverse => s.flip = !s.flip,
            Op::Push => {
                stack.push(s);
                s.depth = s.depth.saturating_add(1);
            }
            Op::Pop => s = stack.pop().expect("words are bracket-balanced"),
            Op::Scale(f) => s.scale *= f,
            Op::SetColour(c) => s.colour = c,
            Op::AddColour(c) => s.colour = s.colour.wrapping_add(c),
        }
    }
    stats
}

/// Moves the turtle as a subtree with effect `e` would.
fn apply(t: &Tables, s: &mut State, e: &super::tables::Effect) {
    let v = if s.flip { [e.d[0], -e.d[1]] } else { e.d };
    let dir = t.dir(s.turns, s.fr);
    let w = [dir[0] * v[0] - dir[1] * v[1], dir[0] * v[1] + dir[1] * v[0]];
    s.pos = [s.pos[0] + s.scale * w[0], s.pos[1] + s.scale * w[1]];
    let sign = if s.flip { -1 } else { 1 };
    s.turns = t.add_turns(s.turns, sign * e.turns);
    if e.free != 0.0 {
        s.free = (s.free + sign as f64 * e.free).rem_euclid(TAU);
        s.fr = unit(s.free);
    }
    s.flip ^= e.flip;
    s.scale *= e.scale;
    s.colour = e.colour.apply(s.colour);
}

/// The picture's bounding box at `order`, world units `[x0, y0, x1, y1]` (`None`: it draws
/// nothing), from a walk of at most `budget` segments.
pub fn bounds(t: &Tables, order: u32, budget: u64) -> Option<[f64; 4]> {
    let mut b = [f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
    let opts = WalkOptions { order, lod_px: 0.0, budget };
    let mut add = |p: [f64; 2]| b = [b[0].min(p[0]), b[1].min(p[1]), b[2].max(p[0]), b[3].max(p[1])];
    walk_all(t, &View::EVERYTHING, &opts, &mut |d| match d {
        Drawn::Segment(s) => {
            add(s.a);
            add(s.b);
        }
        Drawn::Polygon(p) => p.pts.iter().for_each(|&q| add(q)),
    });
    (b[0] <= b[2]).then_some(b)
}

/// The order to frame a whole picture at: the highest in phase whose segment count is at most
/// `budget`.
pub fn framing_order(t: &Tables, budget: f64) -> u32 {
    t.in_phase(raw_framing_order(t, budget))
}

fn raw_framing_order(t: &Tables, budget: f64) -> u32 {
    // By the work — lines and polygon vertices — so a picture of filled shapes alone frames too.
    (0..=t.max_depth).take_while(|&n| t.axiom_entry(n).w <= budget).last().unwrap_or(0)
}

/// A growing picture's period, box areas and size (see [`Tables::period`], [`Tables::box_areas`],
/// [`Tables::box_size`]), from its boxes at the highest order of at most 20,000 segments and the
/// orders below — down to order 0 while the walks stay cheap (an order below the last walked takes
/// its box). Period 2 when the top box differs from the order below by more than 2% of its size,
/// and by over three times what it differs from the order two below (the Sierpinski arrowhead
/// mirrors outright; Paul Bourke's weed sways 8% from side to side).
pub(crate) fn shape(t: &Tables) -> (u32, Vec<f64>, f64) {
    if !t.grows() {
        return (1, Vec::new(), 0.0);
    }
    let m = raw_framing_order(t, 20_000.0) as usize;
    let mut boxes: Vec<Option<[f64; 4]>> = vec![None; m + 1];
    let (mut work, mut lowest) = (0.0, m);
    for n in (0..=m).rev() {
        if work > 100_000.0 && n + 2 < m {
            break;
        }
        work += t.axiom_entry(n as u32).w.max(1.0);
        boxes[n] = bounds(t, n as u32, 1 << 20);
        lowest = n;
    }
    let Some(a) = boxes[m] else { return (1, Vec::new(), 0.0) };
    let areas = (0..=m).map(|n| boxes[n.max(lowest)].map_or(0.0, |b| (b[2] - b[0]) * (b[3] - b[1]))).collect();
    let size = (a[2] - a[0]).max(a[3] - a[1]);
    let diff = |x: [f64; 4], y: [f64; 4]| (0..4).map(|k| (x[k] - y[k]).abs()).fold(0.0, f64::max);
    let period = match (m >= 3).then(|| (boxes[m - 1], boxes[m - 2])) {
        Some((Some(b), Some(c))) if diff(a, b) > 0.02 * size && diff(a, b) > 3.0 * diff(a, c) => 2,
        _ => 1,
    };
    (period, areas, size)
}

#[cfg(test)]
mod tests;
