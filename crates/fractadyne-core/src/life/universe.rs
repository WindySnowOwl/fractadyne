//! The Life universe as a sparse tile map (design/automata.md §4.2): the plane is cut into 64×64-cell
//! tiles and only the tiles that differ from the background are stored, so memory follows the live
//! area, not the extent (±2^62 cells). A torus and a bounded plane are the same map with every tile
//! stored. This is the CPU twin of the GPU stepper (phase 2); [`super::dense::Dense`] is the truth
//! both are tested against.

use super::rule::{Rule, CENTRE, E, N, NE, NW, S, SE, SW, W};
use std::collections::{HashMap, HashSet};

/// log₂ of a tile's side.
pub const TILE_BITS: u32 = 6;
/// A tile's side in cells.
pub const TILE: i64 = 1 << TILE_BITS;
const SIDE: usize = TILE as usize;
/// Cells in a tile.
pub const TILE_CELLS: usize = SIDE * SIDE;
const CELLS: usize = TILE_CELLS;
/// The padded window a tile is stepped from: the tile and a one-cell ring of its neighbours.
const PAD: usize = SIDE + 2;
/// The plane's extent: cell coordinates stay within ±`LIMIT`, so a tile's neighbours never overflow.
pub const LIMIT: i64 = 1 << 62;

/// The shape of a universe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Topology {
    /// Unbounded (±2^62 cells); only tiles that differ from the background are stored.
    Plane,
    /// A torus of `width × height` cells (each a multiple of 64), cells `0..width × 0..height`.
    Torus { width: u32, height: u32 },
    /// Dead outside the rectangle `x..x+width × y..y+height`, where nothing is ever born.
    Bounded { x: i64, y: i64, width: u32, height: u32 },
}

impl Topology {
    /// Refuses a torus whose sides are not positive multiples of 64, an empty bounded plane, and
    /// either one past the plane's extent.
    pub fn validate(&self) -> Result<(), String> {
        match *self {
            Topology::Plane => Ok(()),
            Topology::Torus { width, height } => {
                if width == 0 || height == 0 || i64::from(width) % TILE != 0 || i64::from(height) % TILE != 0 {
                    Err(format!("a torus's sides are multiples of {TILE}, not {width}×{height}"))
                } else {
                    Ok(())
                }
            }
            Topology::Bounded { x, y, width, height } => {
                let inside = |v: i64, len: u32| v > -LIMIT && v + i64::from(len) < LIMIT;
                if width == 0 || height == 0 {
                    Err("a bounded plane has a width and a height".into())
                } else if !inside(x, width) || !inside(y, height) {
                    Err("the bounded plane lies outside the universe's extent".into())
                } else {
                    Ok(())
                }
            }
        }
    }
}

type Tile = Box<[u8; CELLS]>;

fn blank(state: u8) -> Tile {
    Box::new([state; CELLS])
}

/// Floor division of a cell coordinate into its tile and the offset within it.
#[inline]
fn split(v: i64) -> (i64, usize) {
    (v >> TILE_BITS, (v & (TILE - 1)) as usize)
}

/// A Life-like universe: a rule, a topology, the cells, and the generation count.
#[derive(Clone)]
pub struct Universe {
    rule: Rule,
    topology: Topology,
    /// The state of every cell no tile stores (an unbounded plane's; 0 otherwise). Changes only
    /// under B0 rules ([`Rule::next_background`]).
    background: u8,
    generation: u64,
    tiles: HashMap<(i64, i64), Tile>,
}

impl Universe {
    pub fn new(rule: Rule, topology: Topology) -> Result<Universe, String> {
        topology.validate()?;
        let mut u = Universe { rule, topology, background: 0, generation: 0, tiles: HashMap::new() };
        for key in u.fixed_tiles() {
            u.tiles.insert(key, blank(0));
        }
        Ok(u)
    }

    /// The tiles a torus or a bounded plane always stores (none for the plane).
    fn fixed_tiles(&self) -> Vec<(i64, i64)> {
        let (tx0, ty0, tx1, ty1) = match self.topology {
            Topology::Plane => return Vec::new(),
            Topology::Torus { width, height } => (0, 0, i64::from(width) / TILE - 1, i64::from(height) / TILE - 1),
            Topology::Bounded { x, y, width, height } => {
                (split(x).0, split(y).0, split(x + i64::from(width) - 1).0, split(y + i64::from(height) - 1).0)
            }
        };
        (ty0..=ty1).flat_map(|ty| (tx0..=tx1).map(move |tx| (tx, ty))).collect()
    }

    pub fn rule(&self) -> &Rule {
        &self.rule
    }

    pub fn topology(&self) -> Topology {
        self.topology
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn set_generation(&mut self, generation: u64) {
        self.generation = generation;
    }

    /// The state of every cell no tile stores.
    pub fn background(&self) -> u8 {
        self.background
    }

    /// Start an unbounded plane's background in `state` (binary rules only: a B0/S8 rule's natural
    /// background is alive). Refused for the other topologies, whose outside is always dead.
    pub fn set_background(&mut self, state: u8) -> bool {
        if self.topology != Topology::Plane || self.rule.states() != 2 || state > 1 {
            return false;
        }
        if state != self.background {
            // Stored tiles keep their cells; a tile that now matches the background is dropped.
            self.background = state;
            self.tiles.retain(|_, t| t.iter().any(|&c| c != state));
        }
        true
    }

    /// Stored tiles: the memory a universe uses is this × 4 KiB.
    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    /// The stored tiles: `(tile x, tile y)` (cell coordinates ÷ 64, rounded down) and the tile's
    /// 4,096 states, row by row. What the GPU stepper uploads.
    pub fn tiles(&self) -> impl Iterator<Item = ((i64, i64), &[u8; TILE_CELLS])> + '_ {
        self.tiles.iter().map(|(&k, t)| (k, &**t))
    }

    /// Replace one tile's cells (what the GPU stepper downloads). A plane drops a tile that equals
    /// the background; a torus or bounded plane ignores a tile outside its area, and a bounded
    /// plane's cells outside its rectangle stay dead.
    pub fn put_tile(&mut self, key: (i64, i64), cells: &[u8; TILE_CELLS]) {
        match self.topology {
            Topology::Plane => {
                if cells.iter().all(|&c| c == self.background) {
                    self.tiles.remove(&key);
                } else {
                    self.tiles.insert(key, Box::new(*cells));
                }
            }
            Topology::Torus { .. } | Topology::Bounded { .. } => {
                let Some(t) = self.tiles.get_mut(&key) else { return };
                **t = *cells;
                if let Topology::Bounded { x, y, width, height } = self.topology {
                    mask_outside(key, t, x, y, width, height);
                }
            }
        }
    }

    /// The cell `(x, y)` in universe coordinates: a torus wraps, a bounded plane's outside is dead.
    /// `None` when it lies outside the universe (beyond the plane's extent, or outside a bounded
    /// plane's rectangle).
    fn locate(&self, x: i64, y: i64) -> Option<(i64, i64)> {
        match self.topology {
            Topology::Plane => (x.abs() < LIMIT && y.abs() < LIMIT).then_some((x, y)),
            Topology::Torus { width, height } => Some((x.rem_euclid(width.into()), y.rem_euclid(height.into()))),
            Topology::Bounded { x: bx, y: by, width, height } => {
                let inside = x >= bx && x < bx + i64::from(width) && y >= by && y < by + i64::from(height);
                inside.then_some((x, y))
            }
        }
    }

    pub fn get(&self, x: i64, y: i64) -> u8 {
        let Some((x, y)) = self.locate(x, y) else { return 0 };
        let ((tx, ox), (ty, oy)) = (split(x), split(y));
        self.tiles.get(&(tx, ty)).map_or(self.background, |t| t[oy * SIDE + ox])
    }

    /// Set a cell; `false` (and nothing changes) when it lies outside the universe or `state` is not
    /// one of the rule's.
    pub fn set(&mut self, x: i64, y: i64, state: u8) -> bool {
        if u16::from(state) >= self.rule.states() {
            return false;
        }
        let Some((x, y)) = self.locate(x, y) else { return false };
        let ((tx, ox), (ty, oy)) = (split(x), split(y));
        let bg = self.background;
        match self.tiles.get_mut(&(tx, ty)) {
            Some(t) => {
                t[oy * SIDE + ox] = state;
                if self.topology == Topology::Plane && state == bg && t.iter().all(|&c| c == bg) {
                    self.tiles.remove(&(tx, ty));
                }
            }
            None if state == bg => {}
            None => {
                let mut t = blank(bg);
                t[oy * SIDE + ox] = state;
                self.tiles.insert((tx, ty), t);
            }
        }
        true
    }

    /// Every cell that differs from the background, as `(x, y, state)`, in row order.
    pub fn cells(&self) -> Vec<(i64, i64, u8)> {
        let bg = self.background;
        let mut out = Vec::new();
        for (&(tx, ty), t) in &self.tiles {
            for (i, &c) in t.iter().enumerate() {
                if c != bg {
                    out.push((tx * TILE + (i % SIDE) as i64, ty * TILE + (i / SIDE) as i64, c));
                }
            }
        }
        out.sort_unstable_by_key(|&(x, y, _)| (y, x));
        out
    }

    /// The number of cells that differ from the background: the live cells of a binary rule on a
    /// dead background; for Generations, the dying cells too.
    pub fn population(&self) -> u64 {
        let bg = self.background;
        self.tiles.values().map(|t| t.iter().filter(|&&c| c != bg).count() as u64).sum()
    }

    /// The smallest rectangle `(x0, y0, x1, y1)` (inclusive) holding every cell that differs from
    /// the background; `None` when there are none.
    pub fn bounding_box(&self) -> Option<(i64, i64, i64, i64)> {
        let bg = self.background;
        let mut bb: Option<(i64, i64, i64, i64)> = None;
        for (&(tx, ty), t) in &self.tiles {
            for (i, &c) in t.iter().enumerate() {
                if c != bg {
                    let (x, y) = (tx * TILE + (i % SIDE) as i64, ty * TILE + (i / SIDE) as i64);
                    bb = Some(match bb {
                        None => (x, y, x, y),
                        Some((a, b, c, d)) => (a.min(x), b.min(y), c.max(x), d.max(y)),
                    });
                }
            }
        }
        bb
    }

    /// Clear every cell (the background returns to dead) and the generation count.
    pub fn clear(&mut self) {
        self.tiles.clear();
        self.background = 0;
        self.generation = 0;
        for key in self.fixed_tiles() {
            self.tiles.insert(key, blank(0));
        }
    }

    /// Fill the rectangle `x..x+w × y..y+h` at random: each cell alive with probability `density`,
    /// from `seed` (the same seed gives the same soup on every machine).
    pub fn random_fill(&mut self, x: i64, y: i64, w: u32, h: u32, density: f64, seed: u64) {
        let mut s = seed ^ 0x9E37_79B9_7F4A_7C15;
        let threshold = (density.clamp(0.0, 1.0) * (1u64 << 53) as f64) as u64;
        for j in 0..i64::from(h) {
            for i in 0..i64::from(w) {
                // splitmix64
                s = s.wrapping_add(0x9E37_79B9_7F4A_7C15);
                let mut z = s;
                z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                z ^= z >> 31;
                let alive = (z >> 11) < threshold;
                self.set(x + i, y + j, alive as u8);
            }
        }
    }

    /// The neighbour of tile `key` at offset `(dx, dy)`, if the universe has one there: a torus
    /// wraps; a plane's and a bounded plane's missing tiles read as the background.
    fn neighbour(&self, (tx, ty): (i64, i64), dx: i64, dy: i64) -> Option<&Tile> {
        let key = match self.topology {
            Topology::Torus { width, height } => {
                ((tx + dx).rem_euclid(i64::from(width) / TILE), (ty + dy).rem_euclid(i64::from(height) / TILE))
            }
            _ => (tx + dx, ty + dy),
        };
        self.tiles.get(&key)
    }

    /// Fill `pad` with tile `key` and the one-cell ring around it; returns whether any of it
    /// differs from the background.
    fn fill_pad(&self, key: (i64, i64), pad: &mut [u8; PAD * PAD]) -> bool {
        let bg = self.background;
        pad.fill(bg);
        let mut differs = false;
        let last = SIDE - 1;
        for dy in -1..=1i64 {
            for dx in -1..=1i64 {
                let Some(t) = self.neighbour(key, dx, dy) else { continue };
                // The source rectangle in the neighbour, and where it lands in the pad.
                let (sx, w, px) = match dx {
                    -1 => (last, 1, 0),
                    0 => (0, SIDE, 1),
                    _ => (0, 1, PAD - 1),
                };
                let (sy, h, py) = match dy {
                    -1 => (last, 1, 0),
                    0 => (0, SIDE, 1),
                    _ => (0, 1, PAD - 1),
                };
                for r in 0..h {
                    let src = &t[(sy + r) * SIDE + sx..(sy + r) * SIDE + sx + w];
                    let dst = &mut pad[(py + r) * PAD + px..(py + r) * PAD + px + w];
                    dst.copy_from_slice(src);
                    differs |= src.iter().any(|&c| c != bg);
                }
            }
        }
        differs
    }

    /// Advance one generation.
    pub fn step(&mut self) {
        let plane = self.topology == Topology::Plane;
        let next_bg = if plane { self.rule.next_background(self.background) } else { 0 };
        // A plane steps its stored tiles and the ring of tiles around them; the others, every tile.
        let candidates: Vec<(i64, i64)> = if plane {
            let mut set = HashSet::with_capacity(self.tiles.len() * 4);
            for &(tx, ty) in self.tiles.keys() {
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        set.insert((tx + dx, ty + dy));
                    }
                }
            }
            set.into_iter().collect()
        } else {
            self.tiles.keys().copied().collect()
        };
        let mut next = HashMap::with_capacity(self.tiles.len());
        let mut pad = Box::new([0u8; PAD * PAD]);
        for key in candidates {
            let differs = self.fill_pad(key, &mut pad);
            if plane && !differs {
                // All background in and around it: the whole tile becomes the next background.
                continue;
            }
            let mut out = blank(0);
            step_tile(&self.rule, &pad, &mut out);
            if let Topology::Bounded { x, y, width, height } = self.topology {
                mask_outside(key, &mut out, x, y, width, height);
            }
            if !plane || out.iter().any(|&c| c != next_bg) {
                next.insert(key, out);
            }
        }
        self.tiles = next;
        self.background = next_bg;
        self.generation += 1;
    }

    /// Advance `n` generations.
    pub fn step_n(&mut self, n: u64) {
        for _ in 0..n {
            self.step();
        }
    }
}

/// One tile's next generation from its padded window.
fn step_tile(rule: &Rule, pad: &[u8; PAD * PAD], out: &mut [u8; CELLS]) {
    let alive = |c: u8| (c == 1) as usize;
    for y in 0..SIDE {
        let (r0, r1, r2) = (&pad[y * PAD..], &pad[(y + 1) * PAD..], &pad[(y + 2) * PAD..]);
        // The column entering on the right: its top, middle and bottom cells.
        let col = |c: usize| (alive(r0[c]) * NE) | (alive(r1[c]) * E) | (alive(r2[c]) * SE);
        // Shifting left by one moves NE→N→NW, E→C→W, SE→S→SW; the mask drops what leaves.
        let keep = NW | N | W | CENTRE | SW | S;
        let mut idx = ((col(0) << 1) & keep) | col(1);
        for x in 0..SIDE {
            idx = ((idx << 1) & keep) | col(x + 2);
            out[y * SIDE + x] = rule.next(r1[x + 1], idx);
        }
    }
}

/// Kill the cells of tile `key` that lie outside a bounded plane's rectangle.
fn mask_outside(key: (i64, i64), out: &mut [u8; CELLS], x: i64, y: i64, width: u32, height: u32) {
    let (x1, y1) = (x + i64::from(width), y + i64::from(height));
    for (i, c) in out.iter_mut().enumerate() {
        let (cx, cy) = (key.0 * TILE + (i % SIDE) as i64, key.1 * TILE + (i / SIDE) as i64);
        if cx < x || cx >= x1 || cy < y || cy >= y1 {
            *c = 0;
        }
    }
}

#[cfg(test)]
mod tests;
