//! The naive way, as the check on [`super::walk`] (design/lsystems.md §8): build the word, then run
//! a turtle over it. It shares none of the walk's arithmetic — headings are radians summed as the
//! turtle reads, not table steps; positions are summed step by step, never through a subtree's
//! effect — so the two agreeing says something.

use super::system::{LSystem, Role, Tok};
use super::variant::Variants;
use super::walk::{Polygon, Segment};
use std::f64::consts::{PI, TAU};

/// The word after `order` rewrites, or `None` past `limit` tokens. A stochastic system's symbols
/// carry their variants through the rewrites ([`super::variant`]): each rewrites by the alternative
/// its symbol and variant choose, and passes its children theirs — rewriting the word a whole
/// generation at a time, where the walk descends a subtree at a time.
pub fn expand(sys: &LSystem, order: u32, limit: usize) -> Option<Vec<Tok>> {
    let vs = Variants::of(sys);
    let weights: Vec<Vec<f64>> = (0..=255u8).map(|c| sys.productions(c).iter().map(|p| p.weight).collect()).collect();
    let mut w: Vec<(Tok, u32)> = sys.axiom.iter().enumerate().map(|(j, &t)| (t, vs.child(vs.root, j))).collect();
    for _ in 0..order {
        let mut next = Vec::with_capacity(w.len() * 2);
        for &(t, v) in &w {
            match t {
                Tok::Sym(c) if !sys.productions(c).is_empty() => {
                    let p = &sys.productions(c)[vs.choose(c, v, &weights[c as usize])];
                    next.extend(p.word.iter().enumerate().map(|(j, &u)| (u, vs.child(v, j))));
                }
                _ => next.push((t, v)),
            }
            if next.len() > limit {
                return None;
            }
        }
        w = next;
    }
    Some(w.into_iter().map(|(t, _)| t).collect())
}

/// The segments a turtle draws reading `word`, starting at `start` with first step `step` (both
/// in the walk's pixels), numbered as the walk numbers them.
pub fn draw(sys: &LSystem, word: &[Tok], start: [f64; 2], step: [f64; 2]) -> Vec<Segment> {
    run(sys, word, start, step).segments
}

/// What a turtle run produced.
pub struct Run {
    pub segments: Vec<Segment>,
    /// The filled polygons (`{ … }`), in the order they close.
    pub polygons: Vec<Polygon>,
    /// Where the turtle ended.
    pub end: [f64; 2],
}

/// As [`draw`], with where the turtle ends.
pub fn run(sys: &LSystem, word: &[Tok], start: [f64; 2], step: [f64; 2]) -> Run {
    #[derive(Clone, Copy)]
    struct S {
        pos: [f64; 2],
        angle: f64,
        len: f64,
        flip: bool,
        colour: i32,
        depth: u16,
    }
    let delta = sys.angle.degrees().to_radians();
    let around = match sys.angle.division() {
        Some(n) => (n / 2) as f64 * TAU / n as f64,
        None => PI,
    };
    let mut s = S { pos: start, angle: step[1].atan2(step[0]), len: step[0].hypot(step[1]), flip: false, colour: 1, depth: 0 };
    let mut stack = Vec::new();
    let mut out = Vec::new();
    let mut polygons = Vec::new();
    // The polygons open, innermost last (a leaf's `{ }` may sit inside an outline's).
    let mut open: Vec<Polygon> = Vec::new();
    let mut index = 0.0;
    let sign = |s: &S| if s.flip { -1.0 } else { 1.0 };
    for &t in word {
        // Kept in [0, τ): an angle summed without bound loses the digits its sine needs.
        s.angle = s.angle.rem_euclid(TAU);
        match t {
            Tok::Sym(c) => {
                let role = sys.roles[c as usize];
                if role == Role::None {
                    continue;
                }
                let a = s.pos;
                s.pos = [a[0] + s.len * s.angle.cos(), a[1] + s.len * s.angle.sin()];
                // Inside a polygon a step is a vertex, drawn or not, and no line.
                if let Some(p) = open.last_mut() {
                    p.pts.push(s.pos);
                    continue;
                }
                if role == Role::Draw {
                    out.push(Segment {
                        a,
                        b: s.pos,
                        index,
                        span: 1.0,
                        depth: s.depth,
                        heading: (s.angle / TAU).rem_euclid(1.0),
                        colour: s.colour,
                    });
                    index += 1.0;
                }
            }
            Tok::Turn(k) => s.angle += sign(&s) * k as f64 * delta,
            Tok::Around => s.angle += sign(&s) * around,
            Tok::TurnBy(a) => s.angle += sign(&s) * a.to_radians(),
            Tok::Reverse => s.flip = !s.flip,
            Tok::Push => {
                stack.push(s);
                s.depth += 1;
            }
            Tok::Pop => s = stack.pop().expect("words are bracket-balanced"),
            Tok::Scale(f) => s.len *= f,
            Tok::SetColour(c) => s.colour = c,
            Tok::AddColour(c) => s.colour = s.colour.wrapping_add(c),
            Tok::PolyStart => open.push(Polygon {
                pts: vec![s.pos],
                index,
                depth: s.depth,
                heading: (s.angle / TAU).rem_euclid(1.0),
                colour: s.colour,
            }),
            Tok::Vertex => {
                if let Some(p) = open.last_mut() {
                    p.pts.push(s.pos);
                }
            }
            Tok::PolyEnd => {
                if let Some(p) = open.pop() {
                    polygons.push(p);
                }
            }
        }
    }
    Run { segments: out, polygons, end: s.pos }
}
