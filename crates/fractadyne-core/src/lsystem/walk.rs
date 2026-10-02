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
    fn meets(&self, p: [f64; 2], r: f64) -> bool {
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
    /// Subtrees looked at (descended into, stepped over or drawn as a chord).
    pub nodes: u64,
    /// Whether it stopped at the budget, with segments left to draw.
    pub stopped: bool,
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
    rule: u16,
    i: u32,
    /// The depth of this word's symbols (how many more times each rewrites).
    child_depth: u32,
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

/// Walks `t` at `opts.order` over `view`, handing each segment to `sink` in curve order.
pub fn walk(t: &Tables, view: &View, opts: &WalkOptions, sink: &mut dyn FnMut(&Segment)) -> WalkStats {
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
    run(t, view, opts, s, Frame { rule: AXIOM, i: 0, child_depth: order }, 0, 0.0, sink)
}

/// Walks one subtree, from a turtle already placed (see [`SubStart`]).
pub fn walk_from(t: &Tables, view: &View, opts: &WalkOptions, start: &SubStart, sink: &mut dyn FnMut(&Segment)) -> WalkStats {
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
    run(t, view, opts, s, Frame { rule: ONE, i: 0, child_depth: depth }, start.sym, start.index, sink)
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
    sink: &mut dyn FnMut(&Segment),
) -> WalkStats {
    let mut stats = WalkStats::default();
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
        f.i += 1;
        let d = f.child_depth;
        match op {
            Op::Sym(c) => {
                let id = t.id[c as usize];
                if id != NONE && d > 0 {
                    let e = t.entry(c, d);
                    stats.nodes += 1;
                    if e.r >= 0.0 {
                        let r_px = s.scale * e.r;
                        let seen = view.meets(s.pos, r_px + view.margin);
                        if seen && r_px > opts.lod_px {
                            frames.push(Frame { rule: id, i: 0, child_depth: d - 1 });
                            continue;
                        }
                        if seen {
                            let a = s.pos;
                            let (h, colour, depth) = (heading(&s), s.colour, s.depth);
                            apply(t, &mut s, &e.fx);
                            if stats.segments >= opts.budget {
                                stats.stopped = true;
                                return stats;
                            }
                            sink(&Segment { a, b: s.pos, index, span: e.n, depth, heading: h, colour });
                            stats.segments += 1;
                            index += e.n;
                            continue;
                        }
                    }
                    apply(t, &mut s, &e.fx);
                    index += e.n;
                } else {
                    match t.roles[c as usize] {
                        Role::None => {}
                        role => {
                            let dir = t.dir(s.turns, s.fr);
                            let v = [dir[0] * s.scale, dir[1] * s.scale];
                            let a = s.pos;
                            s.pos = [a[0] + v[0], a[1] + v[1]];
                            if role == Role::Draw {
                                let mid = [a[0] + 0.5 * v[0], a[1] + 0.5 * v[1]];
                                if view.meets(mid, 0.5 * s.scale + view.margin) {
                                    if stats.segments >= opts.budget {
                                        stats.stopped = true;
                                        return stats;
                                    }
                                    sink(&Segment {
                                        a,
                                        b: s.pos,
                                        index,
                                        span: 1.0,
                                        depth: s.depth,
                                        heading: heading(&s),
                                        colour: s.colour,
                                    });
                                    stats.segments += 1;
                                }
                                index += 1.0;
                            }
                        }
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
    walk(t, &View::EVERYTHING, &opts, &mut |s| {
        for p in [s.a, s.b] {
            b = [b[0].min(p[0]), b[1].min(p[1]), b[2].max(p[0]), b[3].max(p[1])];
        }
    });
    (b[0] <= b[2]).then_some(b)
}

/// The order to frame a whole picture at: the highest in phase whose segment count is at most
/// `budget`.
pub fn framing_order(t: &Tables, budget: f64) -> u32 {
    t.in_phase(raw_framing_order(t, budget))
}

fn raw_framing_order(t: &Tables, budget: f64) -> u32 {
    (0..=t.max_depth).take_while(|&n| t.axiom_entry(n).n <= budget).last().unwrap_or(0)
}

/// A growing picture's period and box area (see [`Tables::period`], [`Tables::box_area`]), from
/// its boxes at the highest order of at most 20,000 segments and the two below. Period 2 when the
/// box differs from the order below by more than 2% of its size, and by over three times what it
/// differs from the order two below (the Sierpinski arrowhead mirrors outright; Paul Bourke's weed
/// sways 8% from side to side).
pub(crate) fn shape(t: &Tables) -> (u32, f64) {
    if !t.grows() {
        return (1, 0.0);
    }
    let m = raw_framing_order(t, 20_000.0);
    let Some(a) = bounds(t, m, 1 << 20) else { return (1, 0.0) };
    let area = (a[2] - a[0]) * (a[3] - a[1]);
    if m < 3 {
        return (1, area);
    }
    let (Some(b), Some(c)) = (bounds(t, m - 1, 1 << 20), bounds(t, m - 2, 1 << 20)) else {
        return (1, area);
    };
    let size = (a[2] - a[0]).max(a[3] - a[1]);
    let diff = |x: [f64; 4], y: [f64; 4]| (0..4).map(|k| (x[k] - y[k]).abs()).fold(0.0, f64::max);
    let period = if diff(a, b) > 0.02 * size && diff(a, b) > 3.0 * diff(a, c) { 2 } else { 1 };
    (period, area)
}

#[cfg(test)]
mod tests;
