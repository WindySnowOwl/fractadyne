//! High-resolution export (DESIGN.md §12).
//!
//! Encoders for the GPU-rendered image: 8-bit sRGB **PNG** and 32-bit float linear
//! **OpenEXR**. The GPU produces a linear RGBA `f32` buffer (row-major, 4 floats per
//! pixel); these helpers encode it to disk. (Tiled/streamed rendering and embedded
//! metadata come later; today the GPU renders the whole frame at once.)

use std::path::Path;

/// Failure modes of the export encoders / decoders. The library-error variants (`Io`,
/// `PngDecode`, `PngEncode`, `Exr`) are `#[from]` sources so `?` threads them through and their
/// `Display` is transparent (a `{e}` status line reads exactly as before); the hand-written
/// variants capture distinctions the library types can't express and that a caller may want to
/// match on (channel-missing vs corrupt vs size-mismatch, instead of collapsing to `None`).
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    PngDecode(#[from] png::DecodingError),
    #[error(transparent)]
    PngEncode(#[from] png::EncodingError),
    #[error(transparent)]
    Exr(#[from] exr::error::Error),
    /// An EXR named-channel lookup found nothing (e.g. Fraktaler-3's `N` / `NF`).
    #[error("EXR channel {0:?} not found")]
    ChannelNotFound(String),
    /// A PNG color type the decoder can't map to RGBA8.
    #[error("unsupported PNG color type")]
    UnsupportedColorType,
    /// A file extension that isn't a format we can read (thumbnails).
    #[error("unsupported file format: {0}")]
    UnsupportedFormat(String),
    /// A decoded image had zero width or height.
    #[error("empty image")]
    EmptyImage,
    /// A buffer smaller than `width*height*4`, or decoded channel data that didn't match dims.
    #[error("buffer/size mismatch: expected {expected}, got {got}")]
    SizeMismatch { expected: usize, got: usize },
}

// Color-space note (why there's no linear→sRGB encode on the PNG path):
//
// The renderer is *display-referred*. `fs_color` writes palette colors (0..1) straight into a
// **non-sRGB** framebuffer — egui-wgpu deliberately selects `Bgra8Unorm`/`Rgba8Unorm` (see
// `preferred_framebuffer_format`) — so the bytes the GPU stores ARE the sRGB values the monitor
// shows: the live view is WYSIWYG. Palette interpolation and relief lighting therefore also
// happen in gamma space, by design (it matches what the user sees while exploring).
//
// So the export buffer already holds sRGB display values. The PNG must quantize them *directly*;
// applying a second linear→sRGB transfer (the old bug) lifts the shadows and desaturates the
// image relative to the live view. The EXR, a linear-convention container, gets the inverse
// (`srgb_to_linear`) so a linear-aware viewer reproduces the same appearance.

/// sRGB → linear transfer (per channel, input clamped to [0, 1]). Used for the EXR master.
fn srgb_to_linear(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// linear → sRGB transfer (per channel, input clamped to [0, 1]). Used to display the
/// (linear) EXR master as an 8-bit thumbnail.
fn srgb_encode(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Quantize a display-space (sRGB) channel value to 8-bit — a direct round, no transfer.
fn quantize8(c: f32) -> u8 {
    (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

/// Bayer 8×8 ordered-dither matrix, values 0..63 in the canonical recursive order.
#[rustfmt::skip]
const BAYER8: [u8; 64] = [
     0, 32,  8, 40,  2, 34, 10, 42,
    48, 16, 56, 24, 50, 18, 58, 26,
    12, 44,  4, 36, 14, 46,  6, 38,
    60, 28, 52, 20, 62, 30, 54, 22,
     3, 35, 11, 43,  1, 33,  9, 41,
    51, 19, 59, 27, 49, 17, 57, 25,
    15, 47,  7, 39, 13, 45,  5, 37,
    63, 31, 55, 23, 61, 29, 53, 21,
];

/// Quantize with an ordered-dither offset of up to ±½ LSB, chosen by pixel position.
///
/// Fractal exteriors are enormous, very smooth gradients — the worst case for 8-bit
/// quantization, and why banding is the complaint newcomers raise first. Rounding alone maps a
/// wide span of colour onto one byte value and leaves a visible contour where it steps; nudging
/// the rounding threshold by a position-dependent fraction of one level breaks that contour into
/// a fine pattern the eye integrates back into a smooth ramp.
///
/// **Ordered, not random, and this is load-bearing.** A random or error-diffused dither would
/// make every render differ from the last, breaking the golden images, the corpus renders, and
/// the frame-to-frame stability of a zoom video (static noise crawling over a moving image is
/// far worse than banding). Bayer is a pure function of `(x, y)`, so renders stay bit-identical
/// run to run while the pattern stays fixed to the image rather than swimming through it.
fn quantize8_dither(c: f32, x: usize, y: usize) -> u8 {
    // Bayer 0..63 → −0.5..+0.5 of one 8-bit level, centred so the mean offset is ~0 and overall
    // brightness is unchanged.
    let d = (BAYER8[(y % 8) * 8 + (x % 8)] as f32 + 0.5) / 64.0 - 0.5;
    let v = c.clamp(0.0, 1.0) * 255.0 + 0.5 + d;
    v.clamp(0.0, 255.0) as u8
}

/// Convert the renderer's display-space (sRGB) RGBA `f32` buffer to 8-bit RGBA bytes —
/// identical to what [`write_png`] stores. Exposed so callers (e.g. golden-image validation)
/// can compare a fresh render against a decoded PNG on the exact same footing. No transfer is
/// applied: the buffer already holds sRGB display values (see the color-space note above).
pub fn to_srgb8(rgba: &[f32]) -> Vec<u8> {
    let n = rgba.len() / 4;
    let mut out = Vec::with_capacity(n * 4);
    for px in rgba[..n * 4].chunks_exact(4) {
        out.push(quantize8(px[0]));
        out.push(quantize8(px[1]));
        out.push(quantize8(px[2]));
        out.push(quantize8(px[3]));
    }
    out
}

/// As [`to_srgb8`], but with ordered dithering applied to the colour channels — the conversion
/// every 8-bit deliverable goes through (see [`quantize8_dither`] for why banding matters here
/// and why the dither is ordered rather than random).
///
/// `width` is needed because the dither pattern is a function of pixel position; a caller with a
/// flat buffer and no geometry should use [`to_srgb8`] and accept the banding.
///
/// **Alpha is never dithered.** It is 1.0 almost everywhere in our output, and perturbing it
/// yields stray 254s — an image that looks fine but is no longer fully opaque, which then shows
/// up as speckle wherever it gets composited.
pub fn to_srgb8_dithered(rgba: &[f32], width: u32) -> Vec<u8> {
    let w = width.max(1) as usize;
    let n = rgba.len() / 4;
    let mut out = Vec::with_capacity(n * 4);
    for (i, px) in rgba[..n * 4].chunks_exact(4).enumerate() {
        let (x, y) = (i % w, i / w);
        out.push(quantize8_dither(px[0], x, y));
        out.push(quantize8_dither(px[1], x, y));
        out.push(quantize8_dither(px[2], x, y));
        out.push(quantize8(px[3]));
    }
    out
}

#[cfg(test)]
mod dither_tests;

/// Decode an EXR at full resolution to `(width, height, rgba_f32)` (row-major, 4 floats
/// per pixel). Used by the `--compare` tool to diff raw iteration data.
pub fn read_exr_rgba_f32(path: &Path) -> Result<(u32, u32, Vec<f32>), ExportError> {
    use exr::prelude::*;
    let image = read_first_rgba_layer_from_file(
        path,
        |size: Vec2<usize>, _| -> (usize, usize, Vec<f32>) {
            (size.0, size.1, vec![0.0f32; size.0 * size.1 * 4])
        },
        |buf: &mut (usize, usize, Vec<f32>), pos: Vec2<usize>, (r, g, b, a): (f32, f32, f32, f32)| {
            let w = buf.0;
            let i = (pos.1 * w + pos.0) * 4;
            buf.2[i] = r;
            buf.2[i + 1] = g;
            buf.2[i + 2] = b;
            buf.2[i + 3] = a;
        },
    )?;
    let (w, h, data) = image.layer_data.channel_data.pixels;
    if w == 0 || h == 0 {
        return Err(ExportError::EmptyImage);
    }
    Ok((w as u32, h as u32, data))
}

/// Decode a single named channel from an EXR to `(width, height, Vec<f32>)`, converting
/// UINT / F16 / F32 sample types to `f32` (row-major, one value per pixel).
///
/// Used for cross-renderer validation against **Fraktaler-3**, whose raw EXR stores the
/// integer escape count in a UINT channel named `"N"` (exterior `n + 1024`, interior
/// `0xFFFFFFFF`) and the smooth fraction in float channel `"NF"`.
pub fn read_exr_channel_f32(path: &Path, name: &str) -> Result<(u32, u32, Vec<f32>), ExportError> {
    use exr::prelude::*;
    let image = read()
        .no_deep_data()
        .largest_resolution_level()
        .all_channels()
        .first_valid_layer()
        .all_attributes()
        .from_file(path)?;
    let layer = &image.layer_data;
    let (w, h) = (layer.size.0, layer.size.1);
    if w == 0 || h == 0 {
        return Err(ExportError::EmptyImage);
    }
    let chan = layer
        .channel_data
        .list
        .iter()
        .find(|c| c.name.to_string() == name)
        .ok_or_else(|| ExportError::ChannelNotFound(name.to_string()))?;
    let data: Vec<f32> = match &chan.sample_data {
        FlatSamples::F16(v) => v.iter().map(|x| x.to_f32()).collect(),
        FlatSamples::F32(v) => v.clone(),
        FlatSamples::U32(v) => v.iter().map(|&x| x as f32).collect(),
    };
    if data.len() != w * h {
        return Err(ExportError::SizeMismatch { expected: w * h, got: data.len() });
    }
    Ok((w as u32, h as u32, data))
}

/// List the channel names present in an EXR's first valid layer (diagnostics / discovery).
pub fn list_exr_channels(path: &Path) -> Result<Vec<String>, ExportError> {
    use exr::prelude::*;
    let image = read()
        .no_deep_data()
        .largest_resolution_level()
        .all_channels()
        .first_valid_layer()
        .all_attributes()
        .from_file(path)?;
    Ok(image.layer_data.channel_data.list.iter().map(|c| c.name.to_string()).collect())
}

/// Decode a PNG at full resolution to `(width, height, rgba8)` (for golden-image diffs).
pub fn read_png_rgba8(path: &Path) -> Result<(u32, u32, Vec<u8>), ExportError> {
    let file = std::fs::File::open(path)?;
    decode_png_rgba8(std::io::BufReader::new(file))
}

/// Decode a PNG from an in-memory byte slice (e.g. an `include_bytes!` asset) to
/// `(width, height, rgba8)`.
pub fn read_png_rgba8_bytes(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), ExportError> {
    decode_png_rgba8(std::io::Cursor::new(bytes))
}

fn decode_png_rgba8<R: std::io::Read>(r: R) -> Result<(u32, u32, Vec<u8>), ExportError> {
    let mut decoder = png::Decoder::new(r);
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;
    let (w, h) = (info.width, info.height);
    let ch = match info.color_type {
        png::ColorType::Rgba => 4usize,
        png::ColorType::Rgb => 3,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Grayscale => 1,
        _ => return Err(ExportError::UnsupportedColorType),
    };
    let mut rgba = vec![0u8; (w as usize) * (h as usize) * 4];
    for (i, px) in buf.chunks_exact(ch).take((w * h) as usize).enumerate() {
        let (r, g, b, a) = match ch {
            4 => (px[0], px[1], px[2], px[3]),
            3 => (px[0], px[1], px[2], 255),
            2 => (px[0], px[0], px[0], px[1]),
            _ => (px[0], px[0], px[0], 255),
        };
        rgba[i * 4..i * 4 + 4].copy_from_slice(&[r, g, b, a]);
    }
    Ok((w, h, rgba))
}

/// tEXt keyword under which the reloadable view state is stored.
pub const META_KEYWORD: &str = "Fractadyne";

/// Write an 8-bit sRGB PNG from the renderer's display-space (sRGB) RGBA `f32` buffer
/// (`width*height*4` floats). The colors are quantized directly — no linear→sRGB transfer —
/// so the PNG matches the live view byte-for-byte (see the color-space note above).
/// `metadata`, if present, is embedded as a `tEXt` chunk (reloadable view state).
impl ExportError {
    /// The underlying `io::ErrorKind` when this failure is an I/O failure at any layer — the
    /// write-retry policy classifies on it. A PNG encode that died because the destination
    /// vanished mid-write surfaces as `PngEncode(IoError)`, and counts. `None` means the failure
    /// is not a destination problem (a real encode bug), and retrying cannot help.
    pub fn io_kind(&self) -> Option<std::io::ErrorKind> {
        match self {
            ExportError::Io(e) => Some(e.kind()),
            ExportError::PngEncode(png::EncodingError::IoError(e)) => Some(e.kind()),
            _ => None,
        }
    }
}

/// What to do about a failed frame write. See `write_retry_policy`.
#[derive(Debug, PartialEq, Eq)]
pub enum WriteVerdict {
    /// Wait this long, then try the same write again.
    RetryAfter(std::time::Duration),
    /// Report and abort — retrying cannot help (or the cap is spent).
    Fatal,
}

/// Classify a frame-write failure and say whether/when to retry (user request 2026-08-17: an
/// eight-hour 4K soak died at frame 6721 because the SMB host serving the destination was
/// rebooting; the share came back minutes later and nothing needed to be lost).
///
/// The classification IS the design, not a detail:
/// - A destination that VANISHED (`NotFound`, network kinds, timeouts — and every kind we cannot
///   name, because Windows maps SMB failures like `ERROR_BAD_NETPATH` to uncategorized) will very
///   likely come back: retry, backing off from seconds (a blip) to minutes (a rebooting host).
/// - A FULL disk, a permissions error, a read-only or invalid destination will not fix
///   themselves: fatal immediately, so the operator hears about it while it is fixable.
/// - The total wait is capped (`GIVE_UP`): after that, give up cleanly — the completed frames
///   are intact and `--resume` continues from the gap (verified 2026-08-17).
///
/// Deliberately RETRY-BIASED for unknown kinds: misclassifying an exotic fatal error costs a
/// bounded 30 minutes; misclassifying a network blip as fatal kills an overnight render.
pub fn write_retry_policy(
    kind: std::io::ErrorKind,
    total_waited: std::time::Duration,
    attempt: u32,
) -> WriteVerdict {
    use std::io::ErrorKind as K;
    const GIVE_UP: std::time::Duration = std::time::Duration::from_secs(30 * 60);
    match kind {
        K::StorageFull
        | K::QuotaExceeded
        | K::PermissionDenied
        | K::ReadOnlyFilesystem
        | K::InvalidFilename
        | K::InvalidInput
        | K::Unsupported => WriteVerdict::Fatal,
        _ if total_waited >= GIVE_UP => WriteVerdict::Fatal,
        _ => {
            // 1 s, 2 s, 5 s, 10 s, 30 s, 60 s, 2 min, 5 min, then 10 min forever (the cap above
            // bounds the sum): fast enough to catch a blip, patient enough for a reboot.
            const LADDER: [u64; 8] = [1, 2, 5, 10, 30, 60, 120, 300];
            let s = LADDER.get(attempt as usize).copied().unwrap_or(600);
            WriteVerdict::RetryAfter(std::time::Duration::from_secs(s))
        }
    }
}

#[cfg(test)]
mod write_retry;

pub fn write_png(
    path: &Path,
    width: u32,
    height: u32,
    rgba: &[f32],
    metadata: Option<&str>,
) -> Result<(), ExportError> {
    let expected = width as usize * height as usize * 4;
    if rgba.len() < expected {
        return Err(ExportError::SizeMismatch { expected, got: rgba.len() });
    }
    // Dithered: this is the 8-bit deliverable, and fractal exteriors are exactly the smooth-ramp
    // case where plain rounding leaves visible contours. `to_srgb8_dithered` is also what the
    // golden comparison uses, so a written PNG and a freshly converted buffer stay byte-identical
    // — a mismatch there would make every golden fail for a reason that has nothing to do with
    // rendering.
    let bytes = to_srgb8_dithered(&rgba[..expected], width);
    let file = std::fs::File::create(path)?;
    let w = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    if let Some(meta) = metadata {
        encoder.add_text_chunk(META_KEYWORD.to_string(), meta.to_string())?;
    }
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&bytes)?;
    Ok(())
}

/// Write already-sRGB 8-bit RGBA pixels straight to a PNG (no linear→sRGB conversion). Use this
/// for pixels that are ALREADY in display space — e.g. an egui `ColorImage` framebuffer capture,
/// where `write_png`'s `quantize8` (which applies the sRGB transfer curve) would double-encode.
pub fn write_png_rgba8(
    path: &Path,
    width: u32,
    height: u32,
    rgba8: &[u8],
    metadata: Option<&str>,
) -> Result<(), ExportError> {
    let expected = width as usize * height as usize * 4;
    if rgba8.len() < expected {
        return Err(ExportError::SizeMismatch { expected, got: rgba8.len() });
    }
    let file = std::fs::File::create(path)?;
    let w = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    if let Some(meta) = metadata {
        encoder.add_text_chunk(META_KEYWORD.to_string(), meta.to_string())?;
    }
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&rgba8[..expected])?;
    Ok(())
}

/// Decode a PNG and box-downsample it to a thumbnail (≤ `max` px on the long edge).
/// Returns `(width, height, rgba8)`. Currently PNG only (EXR thumbnails: future).
pub fn read_thumbnail(path: &Path, max: u32) -> Result<(u32, u32, Vec<u8>), ExportError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());
    match ext.as_deref() {
        Some("png") => thumbnail_png(path, max),
        Some("exr") => thumbnail_exr(path, max),
        other => Err(ExportError::UnsupportedFormat(other.unwrap_or("(none)").to_string())),
    }
}

fn thumbnail_png(path: &Path, max: u32) -> Result<(u32, u32, Vec<u8>), ExportError> {
    let file = std::fs::File::open(path)?;
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;
    let (w, h) = (info.width, info.height);
    if w == 0 || h == 0 {
        return Err(ExportError::EmptyImage);
    }
    let ch = match info.color_type {
        png::ColorType::Rgba => 4usize,
        png::ColorType::Rgb => 3,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Grayscale => 1,
        _ => return Err(ExportError::UnsupportedColorType),
    };
    let scale = (w.max(h).div_ceil(max)).max(1) as usize;
    let tw = (w as usize / scale).max(1);
    let th = (h as usize / scale).max(1);
    let (wu, hu) = (w as usize, h as usize);
    let mut out = vec![0u8; tw * th * 4];
    for ty in 0..th {
        for tx in 0..tw {
            // Average the scale×scale source block.
            let (mut rs, mut gs, mut bs, mut as_, mut n) = (0u32, 0u32, 0u32, 0u32, 0u32);
            for dy in 0..scale {
                let sy = ty * scale + dy;
                if sy >= hu {
                    break;
                }
                for dx in 0..scale {
                    let sx = tx * scale + dx;
                    if sx >= wu {
                        break;
                    }
                    let si = (sy * wu + sx) * ch;
                    let (r, g, b, a) = match ch {
                        4 => (buf[si], buf[si + 1], buf[si + 2], buf[si + 3]),
                        3 => (buf[si], buf[si + 1], buf[si + 2], 255),
                        2 => (buf[si], buf[si], buf[si], buf[si + 1]),
                        _ => (buf[si], buf[si], buf[si], 255),
                    };
                    rs += r as u32;
                    gs += g as u32;
                    bs += b as u32;
                    as_ += a as u32;
                    n += 1;
                }
            }
            let n = n.max(1);
            let di = (ty * tw + tx) * 4;
            out[di] = (rs / n) as u8;
            out[di + 1] = (gs / n) as u8;
            out[di + 2] = (bs / n) as u8;
            out[di + 3] = (as_ / n) as u8;
        }
    }
    Ok((tw as u32, th as u32, out))
}

/// Decode an OpenEXR (linear f32) and box-downsample it to an sRGB thumbnail.
fn thumbnail_exr(path: &Path, max: u32) -> Result<(u32, u32, Vec<u8>), ExportError> {
    use exr::prelude::*;
    let image = read_first_rgba_layer_from_file(
        path,
        |size: Vec2<usize>, _| -> (usize, usize, Vec<f32>) {
            (size.0, size.1, vec![0.0f32; size.0 * size.1 * 4])
        },
        |buf: &mut (usize, usize, Vec<f32>), pos: Vec2<usize>, (r, g, b, a): (f32, f32, f32, f32)| {
            let w = buf.0;
            let i = (pos.1 * w + pos.0) * 4;
            buf.2[i] = r;
            buf.2[i + 1] = g;
            buf.2[i + 2] = b;
            buf.2[i + 3] = a;
        },
    )?;
    let (w, h, data) = image.layer_data.channel_data.pixels;
    if w == 0 || h == 0 {
        return Err(ExportError::EmptyImage);
    }
    let scale = ((w.max(h) as u32).div_ceil(max)).max(1) as usize;
    let tw = (w / scale).max(1);
    let th = (h / scale).max(1);
    let mut out = vec![0u8; tw * th * 4];
    for ty in 0..th {
        for tx in 0..tw {
            // Average the scale×scale block in linear space, then encode to sRGB.
            let (mut rs, mut gs, mut bs, mut as_, mut n) = (0f32, 0f32, 0f32, 0f32, 0u32);
            for dy in 0..scale {
                let sy = ty * scale + dy;
                if sy >= h {
                    break;
                }
                for dx in 0..scale {
                    let sx = tx * scale + dx;
                    if sx >= w {
                        break;
                    }
                    let i = (sy * w + sx) * 4;
                    rs += data[i];
                    gs += data[i + 1];
                    bs += data[i + 2];
                    as_ += data[i + 3];
                    n += 1;
                }
            }
            let n = n.max(1) as f32;
            let di = (ty * tw + tx) * 4;
            out[di] = (srgb_encode(rs / n) * 255.0 + 0.5) as u8;
            out[di + 1] = (srgb_encode(gs / n) * 255.0 + 0.5) as u8;
            out[di + 2] = (srgb_encode(bs / n) * 255.0 + 0.5) as u8;
            out[di + 3] = ((as_ / n).clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        }
    }
    Ok((tw as u32, th as u32, out))
}

/// Read the embedded Fractadyne view-state metadata from a PNG, if present.
/// Box-downsample an in-memory RGBA8 image to at most `max` px on the long edge (the same
/// kernel `read_thumbnail` applies to files). Returns the input untouched when it already fits.
/// Used by the bookmark thumbnail, which snapshots the SCREEN instead of re-rendering — a
/// bookmark preview should show exactly what the user bookmarked, and a re-render at a deep
/// view costs a reference build plus seconds of GPU (crash-1787194989).
pub fn box_thumbnail_rgba8(w: u32, h: u32, rgba: &[u8], max: u32) -> (u32, u32, Vec<u8>) {
    let max = max.max(1);
    let long = w.max(h);
    if long <= max || w == 0 || h == 0 {
        return (w, h, rgba.to_vec());
    }
    let scale = long.div_ceil(max); // integer box size; output ≤ max on the long edge
    let ow = (w / scale).max(1);
    let oh = (h / scale).max(1);
    let mut out = Vec::with_capacity((ow * oh * 4) as usize);
    for oy in 0..oh {
        for ox in 0..ow {
            let (mut r, mut g, mut b, mut a, mut n) = (0u32, 0u32, 0u32, 0u32, 0u32);
            for sy in 0..scale {
                let y = oy * scale + sy;
                if y >= h {
                    break;
                }
                for sx in 0..scale {
                    let x = ox * scale + sx;
                    if x >= w {
                        break;
                    }
                    let i = ((y * w + x) * 4) as usize;
                    r += rgba[i] as u32;
                    g += rgba[i + 1] as u32;
                    b += rgba[i + 2] as u32;
                    a += rgba[i + 3] as u32;
                    n += 1;
                }
            }
            let n = n.max(1);
            out.extend_from_slice(&[(r / n) as u8, (g / n) as u8, (b / n) as u8, (a / n) as u8]);
        }
    }
    (ow, oh, out)
}

pub fn read_png_metadata(path: &Path) -> Result<Option<String>, ExportError> {
    let file = std::fs::File::open(path)?;
    let reader = png::Decoder::new(std::io::BufReader::new(file)).read_info()?;
    Ok(reader
        .info()
        .uncompressed_latin1_text
        .iter()
        .find(|c| c.keyword == META_KEYWORD)
        .map(|c| c.text.clone()))
}

/// Write a 32-bit float **linear** OpenEXR from the renderer's display-space (sRGB) RGBA `f32`
/// buffer. The color channels are converted sRGB→linear so the EXR is a proper linear master
/// that reproduces the live/PNG appearance in a linear-aware viewer (alpha is left as-is).
/// `metadata`, if present, is stored as a custom `Fractadyne` image attribute (reloadable view).
pub fn write_exr(
    path: &Path,
    width: u32,
    height: u32,
    rgba: &[f32],
    metadata: Option<&str>,
) -> Result<(), ExportError> {
    use exr::prelude::*;
    let expected = width as usize * height as usize * 4;
    if rgba.len() < expected {
        return Err(ExportError::SizeMismatch { expected, got: rgba.len() });
    }
    let w = width as usize;
    let channels = SpecificChannels::rgba(|pos: Vec2<usize>| {
        let i = (pos.1 * w + pos.0) * 4;
        (
            srgb_to_linear(rgba[i]),
            srgb_to_linear(rgba[i + 1]),
            srgb_to_linear(rgba[i + 2]),
            rgba[i + 3],
        )
    });
    let mut image = Image::from_channels((width as usize, height as usize), channels);
    if let Some(meta) = metadata {
        image
            .attributes
            .other
            .insert(Text::from(META_KEYWORD), AttributeValue::Text(Text::from(meta)));
    }
    image.write().to_file(path)?;
    Ok(())
}

/// Read the embedded Fractadyne view-state metadata from an OpenEXR, if present.
pub fn read_exr_metadata(path: &Path) -> Result<Option<String>, ExportError> {
    use exr::prelude::*;
    let meta = exr::meta::MetaData::read_from_file(path, false)?;
    let key = Text::from(META_KEYWORD);
    for h in &meta.headers {
        for other in [&h.shared_attributes.other, &h.own_attributes.other] {
            if let Some(AttributeValue::Text(t)) = other.get(&key) {
                return Ok(Some(t.to_string()));
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod writer_roundtrip_tests;
