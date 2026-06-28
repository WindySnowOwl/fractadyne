//! High-resolution export (DESIGN.md §12).
//!
//! Encoders for the GPU-rendered image: 8-bit sRGB **PNG** and 32-bit float linear
//! **OpenEXR**. The GPU produces a linear RGBA `f32` buffer (row-major, 4 floats per
//! pixel); these helpers encode it to disk. (Tiled/streamed rendering and embedded
//! metadata come later; today the GPU renders the whole frame at once.)

use std::path::Path;

/// Linear → sRGB transfer (per channel, input/clamped to [0, 1]).
fn srgb_encode(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// tEXt keyword under which the reloadable view state is stored.
pub const META_KEYWORD: &str = "Fractadyne";

/// Write an 8-bit sRGB PNG from a linear RGBA `f32` buffer (`width*height*4` floats).
/// `metadata`, if present, is embedded as a `tEXt` chunk (reloadable view state).
pub fn write_png(
    path: &Path,
    width: u32,
    height: u32,
    rgba: &[f32],
    metadata: Option<&str>,
) -> Result<(), String> {
    let expected = width as usize * height as usize * 4;
    if rgba.len() < expected {
        return Err(format!("buffer too small: {} < {expected}", rgba.len()));
    }
    let mut bytes = Vec::with_capacity(expected);
    for px in rgba[..expected].chunks_exact(4) {
        bytes.push((srgb_encode(px[0]) * 255.0 + 0.5) as u8);
        bytes.push((srgb_encode(px[1]) * 255.0 + 0.5) as u8);
        bytes.push((srgb_encode(px[2]) * 255.0 + 0.5) as u8);
        bytes.push((px[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
    }
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let w = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    if let Some(meta) = metadata {
        encoder
            .add_text_chunk(META_KEYWORD.to_string(), meta.to_string())
            .map_err(|e| e.to_string())?;
    }
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(&bytes).map_err(|e| e.to_string())?;
    Ok(())
}

/// Decode a PNG and box-downsample it to a thumbnail (≤ `max` px on the long edge).
/// Returns `(width, height, rgba8)`. Currently PNG only (EXR thumbnails: future).
pub fn read_thumbnail(path: &Path, max: u32) -> Option<(u32, u32, Vec<u8>)> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => thumbnail_png(path, max),
        Some("exr") => thumbnail_exr(path, max),
        _ => None,
    }
}

fn thumbnail_png(path: &Path, max: u32) -> Option<(u32, u32, Vec<u8>)> {
    let file = std::fs::File::open(path).ok()?;
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let (w, h) = (info.width, info.height);
    if w == 0 || h == 0 {
        return None;
    }
    let ch = match info.color_type {
        png::ColorType::Rgba => 4usize,
        png::ColorType::Rgb => 3,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Grayscale => 1,
        _ => return None,
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
    Some((tw as u32, th as u32, out))
}

/// Decode an OpenEXR (linear f32) and box-downsample it to an sRGB thumbnail.
fn thumbnail_exr(path: &Path, max: u32) -> Option<(u32, u32, Vec<u8>)> {
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
    )
    .ok()?;
    let (w, h, data) = image.layer_data.channel_data.pixels;
    if w == 0 || h == 0 {
        return None;
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
    Some((tw as u32, th as u32, out))
}

/// Read the embedded Fractadyne view-state metadata from a PNG, if present.
pub fn read_png_metadata(path: &Path) -> Option<String> {
    let file = std::fs::File::open(path).ok()?;
    let reader = png::Decoder::new(std::io::BufReader::new(file))
        .read_info()
        .ok()?;
    reader
        .info()
        .uncompressed_latin1_text
        .iter()
        .find(|c| c.keyword == META_KEYWORD)
        .map(|c| c.text.clone())
}

/// Write a 32-bit float linear OpenEXR from a linear RGBA `f32` buffer. `metadata`,
/// if present, is stored as a custom `Fractadyne` image attribute (reloadable view).
pub fn write_exr(
    path: &Path,
    width: u32,
    height: u32,
    rgba: &[f32],
    metadata: Option<&str>,
) -> Result<(), String> {
    use exr::prelude::*;
    let expected = width as usize * height as usize * 4;
    if rgba.len() < expected {
        return Err(format!("buffer too small: {} < {expected}", rgba.len()));
    }
    let w = width as usize;
    let channels = SpecificChannels::rgba(|pos: Vec2<usize>| {
        let i = (pos.1 * w + pos.0) * 4;
        (rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3])
    });
    let mut image = Image::from_channels((width as usize, height as usize), channels);
    if let Some(meta) = metadata {
        image
            .attributes
            .other
            .insert(Text::from(META_KEYWORD), AttributeValue::Text(Text::from(meta)));
    }
    image.write().to_file(path).map_err(|e| e.to_string())
}

/// Read the embedded Fractadyne view-state metadata from an OpenEXR, if present.
pub fn read_exr_metadata(path: &Path) -> Option<String> {
    use exr::prelude::*;
    let meta = exr::meta::MetaData::read_from_file(path, false).ok()?;
    let key = Text::from(META_KEYWORD);
    for h in &meta.headers {
        for other in [&h.shared_attributes.other, &h.own_attributes.other] {
            if let Some(AttributeValue::Text(t)) = other.get(&key) {
                return Some(t.to_string());
            }
        }
    }
    None
}
