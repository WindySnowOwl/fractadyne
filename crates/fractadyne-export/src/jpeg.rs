//! JPEG export: the renderer's display-space buffer as a baseline JPEG, the view's metadata
//! carried inside it so the file reopens the view as a PNG does, and the reader for both.
//!
//! **Lossy, so set for fractals.** JPEG keeps a picture small by dropping what the eye misses
//! least, and fractal detail (filaments a pixel wide, in a different color from their neighbors)
//! is the worst case. Measured on three 1920x1080 exports (the home view, the seahorse at 20x and
//! at 1e6x, 2026-11-23): halving the color resolution (4:2:0, most encoders' default below high
//! quality) capped what quality could buy at the busiest pixels (the 1e6x view's filaments 32
//! levels off at quality 75, still 25 at 95) where full color resolution kept improving (23 to 6),
//! and at quality 90 full color beat 4:2:0 at 95 on every measure, 1.15 MB against 1.0. So every
//! JPEG here is 4:4:4, with optimized Huffman tables (smaller, still baseline), and 90 is the
//! quality an export starts at: 0.1 to 1.2 MB against the PNG's 2.2 to 4.3, about 33 to 45 dB.
//!
//! **No dither.** A PNG is dithered to hide the 8-bit steps in smooth ramps; JPEG's own
//! quantization is far coarser than that dither, which it would spend bytes keeping as noise. The
//! colors are rounded ([`crate::to_srgb8`]); alpha is dropped (exports are opaque).
//!
//! **The view inside.** Application segment 15 (APP15), which nothing else uses: the identifier
//! [`META_ID`], a segment's number and the count, then a share of the metadata's UTF-8 text, as many
//! segments as it takes (a deep view's coordinates run to tens of thousands of digits, past one
//! segment's 65,533 bytes). Readers that do not know it skip it, as every JPEG reader skips an
//! unknown application segment.

use crate::{ExportError, DEFAULT_IMAGE_LIMITS};
use std::path::Path;

/// The quality an export starts at (the IJG scale, 1 to 100): see the module note.
pub const JPEG_QUALITY_DEFAULT: u8 = 90;

/// The largest side a JPEG can have (its header holds 16 bits).
pub const JPEG_MAX_SIDE: u32 = 65_535;

/// The application segment the metadata travels in.
const META_APP: u8 = 15;
const META_MARKER: u8 = 0xE0 + META_APP;
/// What begins each metadata segment.
const META_ID: &[u8] = b"Fractadyne\0";
/// A segment's payload at most: 65,533 bytes less the identifier and the two counters.
const META_CHUNK: usize = 65_533 - 11 - 2;

/// Write the renderer's display-space (sRGB) RGBA `f32` buffer as a JPEG at `quality` (1 to 100,
/// clamped), with `metadata` inside (see the module note), atomically (a reader never sees half a
/// file). A side past [`JPEG_MAX_SIDE`] is refused, as is metadata past 255 segments (16 MB).
pub fn write_jpeg(path: &Path, width: u32, height: u32, rgba: &[f32], metadata: Option<&str>, quality: u8) -> Result<(), ExportError> {
    let expected = width as usize * height as usize * 4;
    if rgba.len() < expected {
        return Err(ExportError::SizeMismatch { expected, got: rgba.len() });
    }
    for (what, v) in [("JPEG width", width), ("JPEG height", height)] {
        if v > JPEG_MAX_SIDE {
            return Err(ExportError::TooLarge { what, value: v as u64, limit: JPEG_MAX_SIDE as u64 });
        }
    }
    let segments = match metadata {
        Some(m) => meta_segments(m.as_bytes())?,
        None => Vec::new(),
    };
    let rgb: Vec<u8> = crate::to_srgb8(&rgba[..expected]).chunks_exact(4).flat_map(|p| [p[0], p[1], p[2]]).collect();
    crate::write_atomic(path, |w| {
        let mut encoder = jpeg_encoder::Encoder::new(w, quality.clamp(1, 100));
        encoder.set_sampling_factor(jpeg_encoder::SamplingFactor::R_4_4_4);
        encoder.set_optimized_huffman_tables(true);
        for s in segments {
            encoder.add_app_segment(META_APP, s).map_err(jpeg_error)?;
        }
        encoder.encode(&rgb, width as u16, height as u16, jpeg_encoder::ColorType::Rgb).map_err(jpeg_error)
    })
}

fn jpeg_error(e: impl std::fmt::Display) -> ExportError {
    ExportError::Jpeg(e.to_string())
}

/// The metadata's segments: identifier, number (from 1), count, its share of the text.
fn meta_segments(text: &[u8]) -> Result<Vec<Vec<u8>>, ExportError> {
    let chunks: Vec<&[u8]> = if text.is_empty() { vec![&[][..]] } else { text.chunks(META_CHUNK).collect() };
    if chunks.len() > 255 {
        return Err(ExportError::TooLarge { what: "JPEG metadata bytes", value: text.len() as u64, limit: (255 * META_CHUNK) as u64 });
    }
    let n = chunks.len() as u8;
    Ok(chunks
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let mut s = Vec::with_capacity(META_ID.len() + 2 + c.len());
            s.extend_from_slice(META_ID);
            s.push(i as u8 + 1);
            s.push(n);
            s.extend_from_slice(c);
            s
        })
        .collect())
}

/// The view metadata a JPEG export carries (`Ok(None)` for a JPEG without any: another program's,
/// or one written without it). Reads the segments before the image data only, never the picture.
/// Damaged metadata (a segment missing, numbered past its count, counts that disagree, or text
/// that is not UTF-8) is an error, never a part of a view.
pub fn read_jpeg_metadata(path: &Path) -> Result<Option<String>, ExportError> {
    use std::io::Read;
    crate::check_file_bytes(path)?;
    let mut r = std::io::BufReader::new(std::fs::File::open(path)?);
    let mut two = [0u8; 2];
    r.read_exact(&mut two)?;
    if two != [0xFF, 0xD8] {
        return Err(jpeg_error("not a JPEG file (no start-of-image marker)"));
    }
    let mut parts: Vec<Option<Vec<u8>>> = Vec::new();
    loop {
        // A marker: 0xFF, any fill bytes of 0xFF, then its code.
        r.read_exact(&mut two[..1])?;
        if two[0] != 0xFF {
            return Err(jpeg_error("a segment does not start with a marker"));
        }
        let mut code = 0xFFu8;
        while code == 0xFF {
            r.read_exact(&mut two[..1])?;
            code = two[0];
        }
        // The image data, or the end: no more segments of ours.
        if code == 0xDA || code == 0xD9 {
            break;
        }
        // Markers without a length (the restart markers, TEM).
        if (0xD0..=0xD7).contains(&code) || code == 0x01 {
            continue;
        }
        r.read_exact(&mut two)?;
        let len = u16::from_be_bytes(two) as usize;
        if len < 2 {
            return Err(jpeg_error("a segment shorter than its own length"));
        }
        let mut body = vec![0u8; len - 2];
        r.read_exact(&mut body)?;
        if code != META_MARKER || !body.starts_with(META_ID) || body.len() < META_ID.len() + 2 {
            continue;
        }
        let (i, n) = (body[META_ID.len()] as usize, body[META_ID.len() + 1] as usize);
        if n == 0 || i == 0 || i > n || (!parts.is_empty() && parts.len() != n) {
            return Err(jpeg_error("its view metadata is damaged (segments numbered out of their count)"));
        }
        if parts.is_empty() {
            parts = vec![None; n];
        }
        parts[i - 1] = Some(body[META_ID.len() + 2..].to_vec());
    }
    if parts.is_empty() {
        return Ok(None);
    }
    let mut text = Vec::new();
    for p in parts {
        text.extend(p.ok_or_else(|| jpeg_error("its view metadata is damaged (a segment is missing)"))?);
    }
    String::from_utf8(text).map(Some).map_err(|_| jpeg_error("its view metadata is not text"))
}

/// Decode a JPEG at full resolution to `(width, height, rgb8)`, its size checked against the image
/// limits from its header before the picture is allocated (as a PNG's is, F-01).
pub fn read_jpeg_rgb8(path: &Path) -> Result<(u32, u32, Vec<u8>), ExportError> {
    crate::check_file_bytes(path)?;
    let bytes = std::fs::read(path)?;
    decode_jpeg_rgb8(&bytes)
}

fn decode_jpeg_rgb8(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), ExportError> {
    use zune_core::colorspace::ColorSpace;
    use zune_core::options::DecoderOptions;
    let opts = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGB);
    let mut d = zune_jpeg::JpegDecoder::new_with_options(zune_core::bytestream::ZCursor::new(bytes), opts);
    d.decode_headers().map_err(jpeg_error)?;
    let info = d.info().ok_or_else(|| jpeg_error("no image header"))?;
    let (w, h) = (info.width as u32, info.height as u32);
    if w == 0 || h == 0 {
        return Err(ExportError::EmptyImage);
    }
    DEFAULT_IMAGE_LIMITS.check_dims(w as u64, h as u64)?;
    let px = d.decode().map_err(jpeg_error)?;
    if px.len() < w as usize * h as usize * 3 {
        return Err(ExportError::SizeMismatch { expected: w as usize * h as usize * 3, got: px.len() });
    }
    Ok((w, h, px))
}

/// A JPEG's thumbnail (≤ `max` px on the long edge), as `crate::read_thumbnail` makes a PNG's.
pub(crate) fn thumbnail_jpeg(path: &Path, max: u32) -> Result<(u32, u32, Vec<u8>), ExportError> {
    let (w, h, px) = read_jpeg_rgb8(path)?;
    Ok(crate::box_thumbnail(&px, w, h, 3, max))
}

#[cfg(test)]
#[path = "../tests/unit/jpeg/jpeg_tests.rs"]
mod jpeg_tests;
