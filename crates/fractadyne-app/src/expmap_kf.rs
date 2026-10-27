//! Kalles Fraktaler's EXR strips, which zoomasm assembles into zoom videos (`--expmap-format
//! zoomasm`; `design/zoom-video.md` §5).
//!
//! Every convention here was MEASURED from KF 2.15.5.2's own EXR output — flat images and
//! exponential maps at powers 2, 3 and 8, against an independent double-precision orbit, 2026-10-27
//! — never read from its source (AGPL, design §2):
//!
//! - **Geometry:** texel `(x, y)` at `θ = 2πx/W` anticlockwise from the positive real axis and
//!   `r = top·2^(−y/H)`: one octave a strip, the outer edge on row 0, each texel sampled at its
//!   corner ([`fractadyne_core::StripRows::Octave`]). Zoom-in order is ours; KF's own sequence
//!   counts the other way, which is what zoomasm's Reverse box is for.
//! - **`N`** (uint32) = `n + IterationsBias`, `n` the first iteration (`z₁ = c`) with `|zₙ| > ER`,
//!   `ER = 10^(4/p)` (100 at power 2, 21.5 at 3, 3.16 at 8); `0xFFFFFFFF` never escaped, 0 not
//!   computed. `N0`/`N1` (low and high words) replace it when a biased count passes `0xFFFFFFFE`.
//! - **`NF`** = `1 − log_p(ln|zₙ| / ln ER)`, in `[0, 1)`.
//! - **`T`** = `fract(p·arg zₙ / 2π)`.
//! - **`DEX`, `DEY`** = `|zₙ|·ln|zₙ| / |dzₙ/dc|` over a pixel, along `(Re u, −Im u)`, `u` the unit
//!   `zₙ/(dzₙ/dc)`: the plane's vector with the imaginary axis pointing DOWN, as in KF's default
//!   images (`ImagPointsUp: 0`). In a strip KF's pixel is `r·2/hypot(W, H)` — its flat image's
//!   pixel scaled by `r/top` — not the strip's own texel.
//! - **Interior:** `N = 0xFFFFFFFF` and every other channel 0.
//! - **Header:** `Iterations` (the cap) and `IterationsBias`, i32, in the layer's header.
//!
//! The renderer escapes at `|z| > 256`, past `ER` at every power it renders strips for, and
//! `ER^p = 10^4 > 256`: its escape iterate is KF's or the one after. The smooth count carries over
//! exactly and says which; `T` steps back through `zₙ₋₁^p = zₙ − c` when it is the one after.

use fractadyne_core::ExpMap;
use fractadyne_export::{ExrChannel, ExrChannelData};
use std::f64::consts::{LOG2_10, TAU};

/// KF's default `IterationsBias`.
pub(crate) const KF_ITERATIONS_BIAS: i32 = 1024;

/// The uint32 `N` that means "never escaped" (and, in `N0` and `N1` both, the same).
const KF_INTERIOR: u32 = u32::MAX;

/// `log2` of KF's escape radius at power `p`: `ER = 10^(4/p)`.
pub(crate) fn kf_escape_log2(p: f64) -> f64 {
    4.0 / p * LOG2_10
}

/// One escaped texel's count, fraction and phase in KF's convention (unbiased count), from the
/// renderer's: its escape count `n` (`|z| > 256`), fraction `nf = 1 − log_p(log2|z|)` and final
/// angle `t = atan2(z)/2π + ½`, at power `p`; `c` is the texel's point (for the step back).
///
/// The renderer's final `z` is `zₙ = |z|·e^{i·angle}`, `log2|z| = p^(1 − nf)`. KF escaped at `n − 1`
/// exactly when `|zₙ₋₁| > ER`, and `zₙ₋₁^p = zₙ − c` gives both its size and `p·arg zₙ₋₁`: the step
/// back is exact, where carrying the smooth count across would be off by the `+ c` (2e-5 of an
/// iteration at low counts). Otherwise KF read `zₙ` itself.
pub(crate) fn kf_count(n: u32, nf: f32, t: f32, p: f64, c: (f64, f64)) -> (u64, f32, f32) {
    let er_log2 = kf_escape_log2(p);
    let z_log2 = p.powf(1.0 - nf as f64);
    let angle = TAU * (t as f64 - 0.5);
    let mag = 2f64.powf(z_log2);
    let (wr, wi) = (mag * angle.cos() - c.0, mag * angle.sin() - c.1);
    let prev_log2 = wr.hypot(wi).log2() / p;
    let (whole, at_log2, turns) = if n > 1 && prev_log2 > er_log2 {
        (n - 1, prev_log2, wi.atan2(wr) / TAU) // p·arg z_{n−1} = arg(z_n − c)
    } else {
        (n, z_log2, p * angle / TAU)
    };
    // NF = 1 − log_p(ln|z|/ln ER), the logs' base cancelling; in [0, 1) up to rounding.
    let frac = (1.0 - (at_log2 / er_log2).ln() / p.ln()) as f32;
    let frac = if frac >= 1.0 { 1.0 - f32::EPSILON / 2.0 } else { frac.max(0.0) };
    let mut phase = turns.rem_euclid(1.0) as f32;
    if phase >= 1.0 {
        phase = 0.0;
    }
    (whole as u64, frac, phase)
}

/// A strip's place in the dive, for its KF channels.
pub(crate) struct KfStrip<'a> {
    pub(crate) map: &'a ExpMap,
    pub(crate) k: u64,
    /// The formula's power (2 for Mandelbrot, 3-8 for Multibrot 3-8).
    pub(crate) power: f64,
    /// The dive's centre: only for `T`'s step back past `c`, where an f64 is ample.
    pub(crate) centre: (f64, f64),
}

/// KF's `DEX`, `DEY` from the renderer's distance estimate (`de_log2`, log2 over the strip's radial
/// texel `r·row_step`) and unit normal (`(nx, ny)` = `z/(dz/dc)` in the plane, imaginary up): the
/// magnitude over KF's strip pixel `r·2/hypot(W, H)`, so `r` cancels.
pub(crate) fn kf_de(de_log2: f32, nx: f32, ny: f32, row_step: f64, width: u32, rows: u32) -> (f32, f32) {
    if !de_log2.is_finite() || de_log2 >= 1.0e29 || !(nx.is_finite() && ny.is_finite()) {
        return (0.0, 0.0);
    }
    let mag = 2f64.powf(de_log2 as f64) * row_step * (width as f64).hypot(rows as f64) / 2.0;
    ((mag * nx as f64) as f32, (-mag * ny as f64) as f32)
}

/// A strip's channels in KF's layout from the renderer's main target (`smooth, nx, ny, de_log2`)
/// and strip aux target (`iter >> 12, iter & 4095, fraction, angle`), as
/// [`crate::expmap_render::strip_channels`] reads them.
pub(crate) fn kf_channels(main: &[f32], aux: &[f32], s: &KfStrip) -> Vec<ExrChannel> {
    let (w, h) = (s.map.width as usize, s.map.rows as usize);
    let n = main.len() / 4;
    debug_assert_eq!(n, w * h);
    let o = s.map.sample_offset();
    let cols: Vec<(f64, f64)> = (0..w).map(|i| (s.map.step() * (i as f64 + o)).sin_cos()).map(|(sn, cs)| (cs, sn)).collect();
    let radii: Vec<f64> = (0..h).map(|j| 2f64.powf(s.map.texel_polar(s.k, 0.0, j as f64).0)).collect();
    let row_step = s.map.row_step();
    let mut count = vec![0u64; n];
    let (mut nf, mut t, mut dex, mut dey) = (vec![0f32; n], vec![0f32; n], vec![0f32; n], vec![0f32; n]);
    let mut wide = false;
    for idx in 0..n {
        let sm = main[idx * 4];
        if sm >= 0.0 {
            let it = (aux[idx * 4] as u32) << 12 | aux[idx * 4 + 1] as u32;
            let (cs, sn) = cols[idx % w];
            let r = radii[idx / w];
            let c = (s.centre.0 + r * cs, s.centre.1 + r * sn);
            let (whole, f, ph) = kf_count(it, aux[idx * 4 + 2], aux[idx * 4 + 3], s.power, c);
            let biased = (whole as i64 + KF_ITERATIONS_BIAS as i64).max(1) as u64;
            wide |= biased > 0xFFFF_FFFE;
            count[idx] = biased;
            nf[idx] = f;
            t[idx] = ph;
            (dex[idx], dey[idx]) = kf_de(main[idx * 4 + 3], main[idx * 4 + 1], main[idx * 4 + 2], row_step, s.map.width, s.map.rows);
        } else if sm > -1.5 {
            count[idx] = u64::MAX; // never escaped: the cap was reached
        } // else glitched (-2): N stays 0, as KF writes a texel it did not compute
    }
    let mut out = Vec::with_capacity(6);
    if wide {
        let lo = count.iter().map(|&c| if c == u64::MAX { KF_INTERIOR } else { c as u32 }).collect();
        let hi = count.iter().map(|&c| if c == u64::MAX { KF_INTERIOR } else { (c >> 32) as u32 }).collect();
        out.push(ExrChannel { name: "N0", data: ExrChannelData::U32(lo) });
        out.push(ExrChannel { name: "N1", data: ExrChannelData::U32(hi) });
    } else {
        let n32 = count.iter().map(|&c| if c == u64::MAX { KF_INTERIOR } else { c as u32 }).collect();
        out.push(ExrChannel { name: "N", data: ExrChannelData::U32(n32) });
    }
    out.push(ExrChannel { name: "NF", data: ExrChannelData::F32(nf) });
    out.push(ExrChannel { name: "T", data: ExrChannelData::F32(t) });
    out.push(ExrChannel { name: "DEX", data: ExrChannelData::F32(dex) });
    out.push(ExrChannel { name: "DEY", data: ExrChannelData::F32(dey) });
    out
}

#[cfg(test)]
#[path = "../tests/unit/expmap_kf/tests.rs"]
mod tests;
