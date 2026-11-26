//! A custom orbit-trap outline (coloring plan F4, part 4): a drawing's path flattened to straight
//! segments in the trap's unit frame, with the exact distance to it.
//!
//! The trap-shape study (2026-11-13) measured a 2048² distance texture of a heart against the exact
//! distance: 9% of the seahorse view came out off by more than 0.05 of a color cycle, because an
//! orbit's minimum distance reads the field where it is sharpest. So the distance here is the exact
//! one, to the nearest segment, found through a coarse grid: each cell keeps the segments that can
//! be the nearest one for some point in it (the ones within the cell center's nearest distance plus
//! the cell's diameter), so a point tests only its cell's list. The same lists, in the same order,
//! go to the GPU ([`Outline::gpu_data`]), so the shader's f32 distance and this f64 one (the BLA
//! aggregates' and the CPU judges') come from the same segments.
//!
//! The drawing is stored as text ([`Outline::text`]): its polylines, centered on their bounding box
//! and scaled so the farthest point is 1 from that center, each coordinate at five decimals. A view
//! file keeps that text, so a view opens the same without the drawing it came from, and the outline
//! in memory is always the one parsed back from it ([`Outline::from_paths`] goes through the text).
//! That text arrives from files and is UNTRUSTED: [`Outline::from_text`] bounds its length, its
//! segment count and every coordinate.

use std::fmt::Write as _;

/// Segments an outline may have. A drawing that flattens to more is refused (the importer coarsens
/// its flattening first).
pub const MAX_SEGMENTS: usize = 4096;
/// Bytes of outline text [`Outline::from_text`] reads (4096 segments at five decimals take ~80 KB).
pub const MAX_TEXT: usize = 256 * 1024;
/// Cells per side of the grid.
pub const GRID: usize = 64;
/// The grid covers [-DOMAIN, DOMAIN]² of the unit frame: the outline lies within the unit circle,
/// and an orbit point stays inside |z| < 2 until it escapes.
pub const DOMAIN: f64 = 2.0;
/// Segments a cell's list keeps at most: past that, the ones nearest the cell's center. Only a
/// drawing with hundreds of segments at one distance from a cell (rings around one center) reaches
/// it; the distance there is then from those segments, the same on the CPU and the GPU.
pub const MAX_CELL: usize = 192;

const CELL: f64 = 2.0 * DOMAIN / GRID as f64;

/// Why an outline could not be made, in words for the user.
#[derive(Debug, Clone, PartialEq)]
pub enum OutlineError {
    /// No segment of any length.
    Empty,
    /// More segments than [`MAX_SEGMENTS`].
    TooManySegments(usize),
    /// Text longer than [`MAX_TEXT`].
    TooLong(usize),
    /// The text is not an outline: what was wrong, and where.
    Malformed(String),
}

impl std::fmt::Display for OutlineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "the drawing has no lines"),
            Self::TooManySegments(n) => write!(f, "the drawing has {n} line segments; at most {MAX_SEGMENTS}"),
            Self::TooLong(n) => write!(f, "the outline is {n} bytes; at most {MAX_TEXT}"),
            Self::Malformed(why) => write!(f, "not an outline: {why}"),
        }
    }
}

/// A drawing's outline in the trap's unit frame, with its distance grid.
#[derive(Clone, Debug)]
pub struct Outline {
    /// [x0, y0, x1, y1], none of zero length.
    segs: Vec<[f64; 4]>,
    /// Per cell, row by row from the bottom (y), each row left to right (x): start and count in `idx`.
    cells: Vec<(u32, u32)>,
    /// The cells' segment lists, each nearest-first from the cell's center.
    idx: Vec<u32>,
    /// The canonical text (what a view stores).
    text: String,
}

/// Where each part of [`Outline::gpu_data`] starts, in vec4s from its first.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GpuLayout {
    /// The cells, two per vec4: [start, count, start, count].
    pub cells: u32,
    /// The segment indices, four per vec4.
    pub idx: u32,
    /// The segments, one per vec4: [x0, y0, x1, y1].
    pub segs: u32,
    /// vec4s in all.
    pub len: u32,
}

/// The distance from (px, py) to the segment [ax, ay, bx, by]. The shader's `outline_seg_dist` is
/// this, in f32.
fn seg_dist(px: f64, py: f64, s: &[f64; 4]) -> f64 {
    let (abx, aby) = (s[2] - s[0], s[3] - s[1]);
    let (apx, apy) = (px - s[0], py - s[1]);
    let t = ((apx * abx + apy * aby) / (abx * abx + aby * aby)).clamp(0.0, 1.0);
    (apx - abx * t).hypot(apy - aby * t)
}

/// The cell a point of the closed domain falls in: (column, row).
fn cell_of(x: f64, y: f64) -> (usize, usize) {
    let c = |v: f64| (((v + DOMAIN) / CELL).floor().max(0.0) as usize).min(GRID - 1);
    (c(x), c(y))
}

impl Outline {
    /// The outline of `paths`, each a polyline (a closed one repeats its first point last), in any
    /// frame with y up: centered on their bounding box, scaled so the farthest point is 1 from that
    /// center, rounded to the text's five decimals and parsed back, so the result is exactly what a
    /// view will store.
    pub fn from_paths(paths: &[Vec<[f64; 2]>]) -> Result<Self, OutlineError> {
        let pts = || paths.iter().flatten().filter(|p| p[0].is_finite() && p[1].is_finite());
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for p in pts() {
            (x0, y0, x1, y1) = (x0.min(p[0]), y0.min(p[1]), x1.max(p[0]), y1.max(p[1]));
        }
        if x0 > x1 {
            return Err(OutlineError::Empty);
        }
        let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
        let r = pts().map(|p| (p[0] - cx).hypot(p[1] - cy)).fold(0.0, f64::max);
        if r <= 0.0 {
            return Err(OutlineError::Empty);
        }
        let mut text = String::new();
        for path in paths {
            let mut first = true;
            for p in path.iter().filter(|p| p[0].is_finite() && p[1].is_finite()) {
                let (x, y) = ((p[0] - cx) / r, (p[1] - cy) / r);
                let _ = write!(text, "{}{} {}", if first { if text.is_empty() { "M " } else { " M " } } else { " L " }, fmt5(x), fmt5(y));
                first = false;
            }
        }
        Self::from_text(&text)
    }

    /// Parse outline text: polylines, each `M x y` then `L x y`..., numbers at most 1 from the
    /// origin (the unit frame), separated by spaces or commas. UNTRUSTED input: bounded in length,
    /// segments and value.
    pub fn from_text(text: &str) -> Result<Self, OutlineError> {
        if text.len() > MAX_TEXT {
            return Err(OutlineError::TooLong(text.len()));
        }
        let mut segs: Vec<[f64; 4]> = Vec::new();
        let mut last: Option<[f64; 2]> = None;
        let mut toks = text.split(|c: char| c.is_whitespace() || c == ',').filter(|t| !t.is_empty());
        let num = |t: Option<&str>, at: usize| -> Result<f64, OutlineError> {
            let t = t.ok_or_else(|| OutlineError::Malformed(format!("a coordinate is missing after segment {at}")))?;
            let v: f64 = t.parse().map_err(|_| OutlineError::Malformed(format!("{t:?} is not a number")))?;
            if !v.is_finite() || v.abs() > 1.0001 {
                return Err(OutlineError::Malformed(format!("{t} is outside the unit frame")));
            }
            Ok(v)
        };
        while let Some(t) = toks.next() {
            let p = [num(toks.next(), segs.len())?, num(toks.next(), segs.len())?];
            match t {
                "M" => last = Some(p),
                "L" => {
                    let a = last.ok_or_else(|| OutlineError::Malformed("a line before the first M".into()))?;
                    if a != p {
                        if segs.len() == MAX_SEGMENTS {
                            return Err(OutlineError::TooManySegments(MAX_SEGMENTS + 1));
                        }
                        segs.push([a[0], a[1], p[0], p[1]]);
                    }
                    last = Some(p);
                }
                _ => return Err(OutlineError::Malformed(format!("{t:?} is neither M nor L"))),
            }
        }
        if segs.is_empty() {
            return Err(OutlineError::Empty);
        }
        let (cells, idx) = build_grid(&segs);
        Ok(Self { segs, cells, idx, text: text.to_string() })
    }

    /// The canonical text, as a view stores it.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Line segments.
    pub fn segment_count(&self) -> usize {
        self.segs.len()
    }

    /// The longest cell list (at most [`MAX_CELL`]): the most segments a distance tests.
    pub fn longest_cell(&self) -> usize {
        self.cells.iter().map(|c| c.1 as usize).max().unwrap_or(0)
    }

    /// A key for the outline: equal outlines, equal keys (FNV-1a over the text).
    pub fn key(&self) -> u64 {
        self.text.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3))
    }

    /// The distance from (x, y), in the unit frame, to the outline. Inside the grid's domain the
    /// exact distance to the nearest segment; outside it, the distance to the domain's edge plus the
    /// outline's from there (an upper bound, within the domain's size of the exact one, where the
    /// trap's minimum is rarely taken).
    pub fn dist(&self, x: f64, y: f64) -> f64 {
        let (qx, qy) = (x.clamp(-DOMAIN, DOMAIN), y.clamp(-DOMAIN, DOMAIN));
        let out = (x - qx).hypot(y - qy);
        let (cx, cy) = cell_of(qx, qy);
        let (start, count) = self.cells[cy * GRID + cx];
        let mut d = f64::MAX;
        for &k in &self.idx[start as usize..(start + count) as usize] {
            d = d.min(seg_dist(qx, qy, &self.segs[k as usize]));
        }
        out + d
    }

    /// The exact distance, testing every segment (for tests: the grid must agree with it).
    pub fn dist_brute(&self, x: f64, y: f64) -> f64 {
        self.segs.iter().map(|s| seg_dist(x, y, s)).fold(f64::MAX, f64::min)
    }

    /// What the GPU reads, as vec4s appended to its reference buffer, and where each part starts.
    pub fn gpu_data(&self) -> (Vec<[f32; 4]>, GpuLayout) {
        let mut v: Vec<[f32; 4]> = Vec::with_capacity(GRID * GRID / 2 + self.idx.len() / 4 + self.segs.len() + 2);
        for pair in self.cells.chunks(2) {
            v.push([pair[0].0 as f32, pair[0].1 as f32, pair[1].0 as f32, pair[1].1 as f32]);
        }
        let idx = v.len() as u32;
        for quad in self.idx.chunks(4) {
            let mut q = [0.0f32; 4];
            for (i, &k) in quad.iter().enumerate() {
                q[i] = k as f32;
            }
            v.push(q);
        }
        let segs = v.len() as u32;
        v.extend(self.segs.iter().map(|s| [s[0] as f32, s[1] as f32, s[2] as f32, s[3] as f32]));
        let len = v.len() as u32;
        (v, GpuLayout { cells: 0, idx, segs, len })
    }
}

/// A coordinate at five decimals, without a trailing "-0".
fn fmt5(v: f64) -> String {
    let s = format!("{v:.5}");
    let t = s.trim_end_matches('0').trim_end_matches('.');
    if t == "-0" || t.is_empty() { "0".to_string() } else { t.to_string() }
}

/// Each cell's candidate list: the segments within (the nearest one's distance from the cell's
/// center + the cell's diameter) of that center, nearest first, at most [`MAX_CELL`]. Exact: for a
/// point p in the cell, |p - c| <= r, so its nearest segment s* has d(c, s*) <= d(p, s*) + r <=
/// d(p, s_c) + r <= d_c + 2r, s_c being the center's nearest.
fn build_grid(segs: &[[f64; 4]]) -> (Vec<(u32, u32)>, Vec<u32>) {
    let r = CELL * std::f64::consts::FRAC_1_SQRT_2;
    let mut cells = Vec::with_capacity(GRID * GRID);
    let mut idx = Vec::new();
    let mut d: Vec<(f64, u32)> = Vec::with_capacity(segs.len());
    for row in 0..GRID {
        for col in 0..GRID {
            let (cx, cy) = (-DOMAIN + (col as f64 + 0.5) * CELL, -DOMAIN + (row as f64 + 0.5) * CELL);
            d.clear();
            d.extend(segs.iter().enumerate().map(|(k, s)| (seg_dist(cx, cy, s), k as u32)));
            let dc = d.iter().map(|e| e.0).fold(f64::MAX, f64::min);
            let reach = dc + 2.0 * r + 1e-12;
            d.retain(|e| e.0 <= reach);
            d.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            d.truncate(MAX_CELL);
            cells.push((idx.len() as u32, d.len() as u32));
            idx.extend(d.iter().map(|e| e.1));
        }
    }
    (cells, idx)
}

#[cfg(test)]
#[path = "../tests/unit/outline/tests.rs"]
mod tests;
