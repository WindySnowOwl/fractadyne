//! Which cells a view shows (design/automata.md §3): the app's [`Viewport`] maps a cell `(i, j)` —
//! column `i`, row `j`, rows growing downward as pattern files read — to the unit square
//! `x ∈ [i, i+1)`, `−y ∈ [j, j+1)`. The centre is a `BigFloat`, so a view 2^60 cells from the origin
//! still knows which cell each pixel is in; this module turns it into a tile origin (exact `i64`)
//! plus small `f64` offsets the GPU can take.

use super::universe::TILE;
use crate::{FloatExp, Viewport};
use astro_float::{BigFloat, RoundingMode, Sign};

/// A view in cell terms.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellWindow {
    /// The tile (cell coordinates ÷ 64, rounded down) holding the view's top-left corner.
    pub tile_x0: i64,
    pub tile_y0: i64,
    /// The top-left corner's cell coordinates relative to that tile's top-left cell, in `[0, 64)`.
    pub origin: [f64; 2],
    /// Cells per pixel (`Viewport::units_per_pixel`).
    pub cells_per_px: f64,
}

/// `⌊x⌋` exactly, or `None` past ±2^62.
pub fn floor_i64(x: &BigFloat) -> Option<i64> {
    let (Some(e), Some(d)) = (x.exponent(), x.mantissa_digits()) else { return Some(0) };
    let Some(&msw) = d.last() else { return Some(0) };
    if msw == 0 {
        return Some(0);
    }
    let neg = matches!(x.sign(), Some(Sign::Neg));
    // value = 0.MSW… × 2^e: the integer part is the top `e` bits.
    let (mag, frac) = if e <= 0 {
        (0u64, true)
    } else if e > 62 {
        return None;
    } else {
        let e = e as u32;
        let mag = msw >> (64 - e);
        let below = msw << e != 0 || d[..d.len() - 1].iter().any(|&w| w != 0);
        (mag, below)
    };
    let mag = mag as i64;
    Some(if neg { -mag - frac as i64 } else { mag })
}

fn bf(v: f64, p: usize) -> BigFloat {
    BigFloat::from_f64(v, p)
}

/// The cells `vp` shows, or `None` when its corner lies past the plane's extent.
pub fn cell_window(vp: &Viewport) -> Option<CellWindow> {
    let p = vp.precision.max(128);
    let rm = RoundingMode::None;
    let half_w: FloatExp = vp.units_per_pixel.mul_f64(vp.width_px * 0.5);
    let half_h: FloatExp = vp.units_per_pixel.mul_f64(vp.height_px * 0.5);
    let left = vp.center_x.sub(&half_w.to_bf(p), p, rm);
    // Rows grow downward: the top edge is the largest y, and its row coordinate is −y.
    let top = vp.center_y.add(&half_h.to_bf(p), p, rm).neg();
    let corner = |v: &BigFloat| -> Option<(i64, f64)> {
        let i = floor_i64(v)?;
        // v − i, exactly: `i` in two parts that are each exact as f64 (`hi` has at most 42
        // significant bits, `lo` fewer than 20), so the fraction survives at any distance.
        let hi = (i >> 20) << 20;
        let frac = crate::to_f64(&v.sub(&bf(hi as f64, p), p, rm).sub(&bf((i - hi) as f64, p), p, rm));
        let tile = i.div_euclid(TILE);
        Some((tile, (i - tile * TILE) as f64 + frac))
    };
    let (tile_x0, ox) = corner(&left)?;
    let (tile_y0, oy) = corner(&top)?;
    Some(CellWindow { tile_x0, tile_y0, origin: [ox, oy], cells_per_px: vp.units_per_pixel.to_f64() })
}

#[cfg(test)]
mod tests;
