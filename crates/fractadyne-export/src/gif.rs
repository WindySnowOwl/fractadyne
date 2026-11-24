//! GIF export: the renderer's display-space buffer reduced to a palette of 2 to 256 colors, the
//! view's metadata carried inside it so the file reopens the view as a PNG does, and the reader for
//! both.
//!
//! **The palette.** NeuQuant (`color_quant`, the GIF crate's own) on a sample of the picture, then
//! eight rounds of k-means on the same sample, which moved each color to the middle of the pixels
//! nearest it: 0.4 to 1.7 dB better at every size from 2 to 256 colors (the seahorse at 1e6x and at
//! 20x, 1920x1080, 2026-11-23). **Dithered by default** (Floyd–Steinberg): a GIF's few colors turn
//! a smooth ramp into flat bands (16 colors) or faint contour rings (256), which the error
//! diffusion hides while the filaments look the same either way; undithered, the picture is
//! posterized, which some want.
//!
//! **The view inside.** An application extension, `FRCTDYNE` `1.0`, its data the metadata's UTF-8
//! text in the format's own sub-blocks, before the image. A reader that does not know it skips it,
//! as every GIF reader skips an unknown application extension.

use crate::{ExportError, DEFAULT_IMAGE_LIMITS};
use std::path::Path;

/// The palette size an export starts at.
pub const GIF_COLORS_DEFAULT: u16 = 256;
/// The fewest and most colors a GIF palette holds here.
pub const GIF_COLORS_MIN: u16 = 2;
pub const GIF_COLORS_MAX: u16 = 256;
/// The largest side a GIF can have (its header holds 16 bits).
pub const GIF_MAX_SIDE: u32 = 65_535;

/// The application extension's identifier (8 bytes) and authentication code (3).
const APP_ID: &[u8; 11] = b"FRCTDYNE1.0";
/// Pixels the palette is learned from at most (an even stride over the picture past that): 250,000
/// learned a 1920x1080 picture's palette in 0.1 s (NeuQuant) and 0.5 s (k-means).
const SAMPLE_MAX: usize = 250_000;
/// Rounds of k-means after NeuQuant.
const KMEANS_ROUNDS: usize = 8;

/// Write the renderer's display-space (sRGB) RGBA `f32` buffer as a GIF of `colors` colors (2 to
/// 256, clamped), dithered or not, with `metadata` inside (see the module note), atomically. A side
/// past [`GIF_MAX_SIDE`] is refused.
pub fn write_gif(path: &Path, width: u32, height: u32, rgba: &[f32], metadata: Option<&str>, colors: u16, dither: bool) -> Result<(), ExportError> {
    let expected = width as usize * height as usize * 4;
    if rgba.len() < expected {
        return Err(ExportError::SizeMismatch { expected, got: rgba.len() });
    }
    for (what, v) in [("GIF width", width), ("GIF height", height)] {
        if v > GIF_MAX_SIDE {
            return Err(ExportError::TooLarge { what, value: v as u64, limit: GIF_MAX_SIDE as u64 });
        }
    }
    let rgb: Vec<u8> = crate::to_srgb8(&rgba[..expected]).chunks_exact(4).flat_map(|p| [p[0], p[1], p[2]]).collect();
    let colors = colors.clamp(GIF_COLORS_MIN, GIF_COLORS_MAX) as usize;
    let palette = learn_palette(&rgb, colors);
    let indices = map_to_palette(&palette, &rgb, width as usize, dither);
    let flat: Vec<u8> = palette.iter().flatten().copied().collect();
    crate::write_atomic(path, |w| {
        let mut e = ::gif::Encoder::new(w, width as u16, height as u16, &flat).map_err(gif_error)?;
        if let Some(m) = metadata {
            e.write_raw_extension(::gif::AnyExtension(0xFF), &[APP_ID, m.as_bytes()]).map_err(gif_error)?;
        }
        let frame = ::gif::Frame { width: width as u16, height: height as u16, buffer: std::borrow::Cow::Borrowed(&indices), ..Default::default() };
        e.write_frame(&frame).map_err(gif_error)?;
        e.into_inner().map_err(gif_error)?;
        Ok(())
    })
}

fn gif_error(e: impl std::fmt::Display) -> ExportError {
    ExportError::Gif(e.to_string())
}

/// The picture's pixels the palette is learned from: all of them up to [`SAMPLE_MAX`], an even
/// stride past that.
fn sample(rgb: &[u8]) -> Vec<[u8; 3]> {
    let n = rgb.len() / 3;
    let stride = n.div_ceil(SAMPLE_MAX).max(1);
    rgb.chunks_exact(3).step_by(stride).map(|p| [p[0], p[1], p[2]]).collect()
}

/// `colors` colors for the picture (see the module note).
pub(crate) fn learn_palette(rgb: &[u8], colors: usize) -> Vec<[u8; 3]> {
    let s = sample(rgb);
    let rgba: Vec<u8> = s.iter().flat_map(|p| [p[0], p[1], p[2], 255]).collect();
    let nq = color_quant::NeuQuant::new(1, colors, &rgba);
    let mut pal: Vec<[f32; 3]> = nq.color_map_rgb().chunks_exact(3).map(|p| [p[0] as f32, p[1] as f32, p[2] as f32]).collect();
    let pts: Vec<[f32; 3]> = s.iter().map(|p| p.map(|v| v as f32)).collect();
    for _ in 0..KMEANS_ROUNDS {
        let near = Nearest::new(&pal);
        let mut sum = vec![[0f64; 4]; pal.len()];
        for c in &pts {
            let i = near.index(*c);
            for k in 0..3 {
                sum[i][k] += c[k] as f64;
            }
            sum[i][3] += 1.0;
        }
        // A color no pixel is nearest stays where it is.
        for (p, s) in pal.iter_mut().zip(&sum) {
            if s[3] > 0.0 {
                *p = [0, 1, 2].map(|k| (s[k] / s[3]) as f32);
            }
        }
    }
    pal.iter().map(|p| p.map(|v| v.round().clamp(0.0, 255.0) as u8)).collect()
}

/// Each pixel's palette index: the nearest color, or with `dither` the nearest to the pixel plus the
/// error its left and upper neighbors handed on (Floyd–Steinberg: 7/16 right, 3/16 below left, 5/16
/// below, 1/16 below right).
pub(crate) fn map_to_palette(palette: &[[u8; 3]], rgb: &[u8], width: usize, dither: bool) -> Vec<u8> {
    let pal: Vec<[f32; 3]> = palette.iter().map(|p| p.map(|v| v as f32)).collect();
    let near = Nearest::new(&pal);
    let w = width.max(1);
    let h = rgb.len() / 3 / w;
    let mut out = vec![0u8; w * h];
    if !dither {
        for (o, p) in out.iter_mut().zip(rgb.chunks_exact(3)) {
            *o = near.index([p[0] as f32, p[1] as f32, p[2] as f32]) as u8;
        }
        return out;
    }
    // Two rows of carried error: this one and the next.
    let (mut cur, mut next) = (vec![[0f32; 3]; w + 2], vec![[0f32; 3]; w + 2]);
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let c = [0, 1, 2].map(|k| (rgb[i * 3 + k] as f32 + cur[x + 1][k]).clamp(0.0, 255.0));
            let j = near.index(c);
            out[i] = j as u8;
            let e = [0, 1, 2].map(|k| c[k] - pal[j][k]);
            for k in 0..3 {
                cur[x + 2][k] += e[k] * (7.0 / 16.0);
                next[x][k] += e[k] * (3.0 / 16.0);
                next[x + 1][k] += e[k] * (5.0 / 16.0);
                next[x + 2][k] += e[k] * (1.0 / 16.0);
            }
        }
        std::mem::swap(&mut cur, &mut next);
        next.iter_mut().for_each(|e| *e = [0.0; 3]);
    }
    out
}

/// The nearest palette color, searched from the palette sorted by red outward in both directions
/// until red alone is farther than the best found (exact; far fewer distances than all of them).
struct Nearest {
    by_red: Vec<([f32; 3], usize)>,
}

impl Nearest {
    fn new(pal: &[[f32; 3]]) -> Self {
        let mut by_red: Vec<([f32; 3], usize)> = pal.iter().copied().zip(0..).collect();
        by_red.sort_by(|a, b| a.0[0].total_cmp(&b.0[0]));
        Nearest { by_red }
    }

    fn index(&self, c: [f32; 3]) -> usize {
        let d2 = |p: &[f32; 3]| (p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2);
        let at = self.by_red.partition_point(|e| e.0[0] < c[0]);
        let (mut best, mut best_i) = (f32::INFINITY, self.by_red.first().map_or(0, |e| e.1));
        // Upward from the first red at or above the pixel's, then downward from below it.
        for e in &self.by_red[at..] {
            if (e.0[0] - c[0]).powi(2) >= best {
                break;
            }
            let d = d2(&e.0);
            if d < best {
                (best, best_i) = (d, e.1);
            }
        }
        for e in self.by_red[..at].iter().rev() {
            if (e.0[0] - c[0]).powi(2) >= best {
                break;
            }
            let d = d2(&e.0);
            if d < best {
                (best, best_i) = (d, e.1);
            }
        }
        best_i
    }
}

/// The view metadata a GIF export carries (`Ok(None)` for a GIF without it). Reads the blocks before
/// the first image only, never the picture.
pub fn read_gif_metadata(path: &Path) -> Result<Option<String>, ExportError> {
    use std::io::Read;
    crate::check_file_bytes(path)?;
    let mut r = std::io::BufReader::new(std::fs::File::open(path)?);
    let mut head = [0u8; 13];
    r.read_exact(&mut head)?;
    if &head[..3] != b"GIF" {
        return Err(gif_error("not a GIF file"));
    }
    // The global color table, when the screen descriptor says there is one.
    if head[10] & 0x80 != 0 {
        let mut skip = vec![0u8; 3 << ((head[10] & 7) + 1)];
        r.read_exact(&mut skip)?;
    }
    let mut byte = [0u8; 1];
    let mut sub_blocks = |r: &mut std::io::BufReader<std::fs::File>| -> Result<Vec<Vec<u8>>, ExportError> {
        let mut blocks = Vec::new();
        loop {
            r.read_exact(&mut byte)?;
            if byte[0] == 0 {
                return Ok(blocks);
            }
            let mut b = vec![0u8; byte[0] as usize];
            r.read_exact(&mut b)?;
            blocks.push(b);
        }
    };
    loop {
        let mut kind = [0u8; 1];
        r.read_exact(&mut kind)?;
        match kind[0] {
            // An extension: ours is an application extension whose first sub-block is our id.
            0x21 => {
                let mut label = [0u8; 1];
                r.read_exact(&mut label)?;
                let blocks = sub_blocks(&mut r)?;
                if label[0] == 0xFF && blocks.first().is_some_and(|b| b.as_slice() == APP_ID) {
                    let text: Vec<u8> = blocks[1..].concat();
                    return String::from_utf8(text).map(Some).map_err(|_| gif_error("its view metadata is not text"));
                }
            }
            // The first image, or the end: no metadata before it.
            0x2C | 0x3B => return Ok(None),
            _ => return Err(gif_error("a block of an unknown kind")),
        }
    }
}

/// Decode a GIF's first image at full size to `(width, height, rgba8)`, its size checked against
/// the image limits before the picture is allocated (as a PNG's is, F-01).
pub fn read_gif_rgba8(path: &Path) -> Result<(u32, u32, Vec<u8>), ExportError> {
    crate::check_file_bytes(path)?;
    let mut opts = ::gif::DecodeOptions::new();
    opts.set_color_output(::gif::ColorOutput::RGBA);
    // The decoder's own guard set to this policy's (an RGBA8 frame at the pixel limit), not its 50 MB
    // default, which would refuse an export this policy allows (as `png_bytes` does for PNG).
    if let Some(limit) = std::num::NonZeroU64::new(DEFAULT_IMAGE_LIMITS.max_pixels.saturating_mul(4)) {
        opts.set_memory_limit(::gif::MemoryLimit::Bytes(limit));
    }
    let mut d = opts.read_info(std::io::BufReader::new(std::fs::File::open(path)?)).map_err(gif_error)?;
    DEFAULT_IMAGE_LIMITS.check_dims(d.width() as u64, d.height() as u64)?;
    let f = d.read_next_frame().map_err(gif_error)?.ok_or_else(|| gif_error("no image in it"))?;
    let (w, h) = (f.width as u32, f.height as u32);
    if w == 0 || h == 0 {
        return Err(ExportError::EmptyImage);
    }
    Ok((w, h, f.buffer.to_vec()))
}

/// A GIF's thumbnail (≤ `max` px on the long edge), as `crate::read_thumbnail` makes a PNG's.
pub(crate) fn thumbnail_gif(path: &Path, max: u32) -> Result<(u32, u32, Vec<u8>), ExportError> {
    let (w, h, px) = read_gif_rgba8(path)?;
    Ok(crate::box_thumbnail(&px, w, h, 4, max))
}

#[cfg(test)]
#[path = "../tests/unit/gif/gif_tests.rs"]
mod gif_tests;
