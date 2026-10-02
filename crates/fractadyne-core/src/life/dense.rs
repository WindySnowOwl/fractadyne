//! The reference stepper: a plain array and the obvious loop, every cell reading its nine
//! neighbours one by one. Slow, and deliberately independent of the tile map — no tiles, no padded
//! windows, no sliding index — so a tiling bug cannot hide in both. The tile stepper is tested
//! against it here, and the GPU stepper (phase 2) against the tile stepper.

use super::rule::{Rule, CENTRE, E, N, NE, NW, S, SE, SW, W};
use super::universe::Topology;

/// A universe held as one array: for a plane, the box around every cell that differs from the
/// background, grown by one cell a side each generation (the light cone) and trimmed back.
#[derive(Clone)]
pub struct Dense {
    rule: Rule,
    topology: Topology,
    background: u8,
    /// The array's top-left cell and size.
    x0: i64,
    y0: i64,
    w: i64,
    h: i64,
    cells: Vec<u8>,
}

impl Dense {
    pub fn new(rule: Rule, topology: Topology) -> Dense {
        let (x0, y0, w, h) = match topology {
            Topology::Plane => (0, 0, 0, 0),
            Topology::Torus { width, height } => (0, 0, i64::from(width), i64::from(height)),
            Topology::Bounded { x, y, width, height } => (x, y, i64::from(width), i64::from(height)),
        };
        Dense { rule, topology, background: 0, x0, y0, w, h, cells: vec![0; (w * h) as usize] }
    }

    /// Start a plane's background in `state` (binary rules; see [`super::Universe::set_background`]).
    pub fn set_background(&mut self, state: u8) {
        assert_eq!(self.topology, Topology::Plane);
        let old = self.background;
        for c in &mut self.cells {
            if *c == old {
                *c = state;
            }
        }
        self.background = state;
    }

    pub fn background(&self) -> u8 {
        self.background
    }

    /// The cell `(x, y)`: a torus wraps; outside a bounded plane is dead; outside a plane's box is
    /// the background.
    pub fn get(&self, x: i64, y: i64) -> u8 {
        let (x, y) = match self.topology {
            Topology::Torus { .. } => (x.rem_euclid(self.w), y.rem_euclid(self.h)),
            _ => (x, y),
        };
        if x < self.x0 || y < self.y0 || x >= self.x0 + self.w || y >= self.y0 + self.h {
            return if self.topology == Topology::Plane { self.background } else { 0 };
        }
        self.cells[((y - self.y0) * self.w + (x - self.x0)) as usize]
    }

    pub fn set(&mut self, x: i64, y: i64, state: u8) {
        let (x, y) = match self.topology {
            Topology::Torus { .. } => (x.rem_euclid(self.w), y.rem_euclid(self.h)),
            _ => (x, y),
        };
        if self.topology == Topology::Plane {
            self.include(x, y);
        } else if x < self.x0 || y < self.y0 || x >= self.x0 + self.w || y >= self.y0 + self.h {
            return;
        }
        let i = ((y - self.y0) * self.w + (x - self.x0)) as usize;
        self.cells[i] = state;
    }

    /// Grow a plane's box to hold `(x, y)`.
    fn include(&mut self, x: i64, y: i64) {
        if self.w == 0 {
            (self.x0, self.y0, self.w, self.h) = (x, y, 1, 1);
            self.cells = vec![self.background];
            return;
        }
        let (nx0, ny0) = (self.x0.min(x), self.y0.min(y));
        let (nx1, ny1) = ((self.x0 + self.w).max(x + 1), (self.y0 + self.h).max(y + 1));
        if (nx0, ny0, nx1 - nx0, ny1 - ny0) != (self.x0, self.y0, self.w, self.h) {
            let mut grown = Dense { cells: Vec::new(), ..self.clone() };
            (grown.x0, grown.y0, grown.w, grown.h) = (nx0, ny0, nx1 - nx0, ny1 - ny0);
            grown.cells = (0..grown.h)
                .flat_map(|j| (0..grown.w).map(move |i| (i, j)))
                .map(|(i, j)| self.get(nx0 + i, ny0 + j))
                .collect();
            *self = grown;
        }
    }

    /// Advance one generation.
    pub fn step(&mut self) {
        let plane = self.topology == Topology::Plane;
        let next_bg = if plane { self.rule.next_background(self.background) } else { 0 };
        // A plane's next generation can differ from the background one cell further out.
        let grow = i64::from(plane);
        let (nx0, ny0, nw, nh) = (self.x0 - grow, self.y0 - grow, self.w + 2 * grow, self.h + 2 * grow);
        let mut next = vec![0u8; (nw * nh) as usize];
        for j in 0..nh {
            for i in 0..nw {
                let (x, y) = (nx0 + i, ny0 + j);
                let at = |dx: i64, dy: i64, bit: usize| if self.get(x + dx, y + dy) == 1 { bit } else { 0 };
                let idx = at(-1, -1, NW)
                    | at(0, -1, N)
                    | at(1, -1, NE)
                    | at(-1, 0, W)
                    | at(0, 0, CENTRE)
                    | at(1, 0, E)
                    | at(-1, 1, SW)
                    | at(0, 1, S)
                    | at(1, 1, SE);
                next[(j * nw + i) as usize] = self.rule.next(self.get(x, y), idx);
            }
        }
        (self.x0, self.y0, self.w, self.h, self.cells) = (nx0, ny0, nw, nh, next);
        self.background = next_bg;
        if plane {
            self.trim();
        }
    }

    /// Shrink a plane's box to the cells that differ from the background.
    fn trim(&mut self) {
        let bg = self.background;
        let mut bb: Option<(i64, i64, i64, i64)> = None;
        for j in 0..self.h {
            for i in 0..self.w {
                if self.cells[(j * self.w + i) as usize] != bg {
                    bb = Some(match bb {
                        None => (i, j, i, j),
                        Some((a, b, c, d)) => (a.min(i), b.min(j), c.max(i), d.max(j)),
                    });
                }
            }
        }
        let Some((i0, j0, i1, j1)) = bb else {
            (self.x0, self.y0, self.w, self.h, self.cells) = (0, 0, 0, 0, Vec::new());
            return;
        };
        let (w, h) = (i1 - i0 + 1, j1 - j0 + 1);
        let cells = (0..h)
            .flat_map(|j| (0..w).map(move |i| (i, j)))
            .map(|(i, j)| self.cells[((j0 + j) * self.w + i0 + i) as usize])
            .collect();
        (self.x0, self.y0, self.w, self.h, self.cells) = (self.x0 + i0, self.y0 + j0, w, h, cells);
    }

    /// Every cell that differs from the background, as `(x, y, state)`, in row order.
    pub fn cells(&self) -> Vec<(i64, i64, u8)> {
        let bg = self.background;
        let mut out = Vec::new();
        for j in 0..self.h {
            for i in 0..self.w {
                let c = self.cells[(j * self.w + i) as usize];
                if c != bg {
                    out.push((self.x0 + i, self.y0 + j, c));
                }
            }
        }
        out
    }
}
