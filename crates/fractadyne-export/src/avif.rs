//! AVIF export: the renderer's display-space buffer as an AVIF (AV1 in a HEIF container), the
//! view's metadata carried inside it so the file reopens the view as a PNG does, and the reader for
//! that metadata.
//!
//! **Lossy, and the best of the lossy formats for fractals.** Measured with libavif before adding it
//! (2026-11-23, three 1920x1080 exports): at full color resolution (4:4:4) an AVIF matched our JPG
//! at quality 90 in 40 to 60% of its size, and past it beat that JPG on every measure while still
//! smaller; at half color resolution (4:2:0) it stopped improving at the busiest pixels, as a 4:2:0
//! JPG or a WebP does. `ravif` (on `rav1e`, Rust with x86 assembly, which NASM assembles at build
//! time) always encodes 4:4:4. Still slower than libavif (0.1 to 0.5 s a 1080p frame): see
//! [`AVIF_SPEED`]. Undithered, as a JPG is.
//!
//! **The view inside.** A TIFF block whose one tag, ImageDescription, holds the metadata's UTF-8
//! text, stored as the file's Exif item (`avif-serialize` frames it); an Exif item has no 64 KB
//! limit, so a deep view's coordinates fit whole. [`read_avif_metadata`] walks the container's
//! boxes to that item and reads it, never the picture.
//!
//! **No decoder.** No pure-Rust AV1 decoder is mature enough to ship, so an AVIF has no gallery
//! thumbnail (`crate::read_thumbnail` says the format is unsupported) and is never read back here.

use crate::ExportError;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// The quality an export starts at (`ravif`'s scale, 1 to 100): where it matched our JPG at 90
/// (the seahorse at 20x and at 1e6x, 1920x1080: 42.1 and 34.7 dB against 41.5 and 33.4, the
/// busiest pixels 10.2 and 11.0 levels off against 10.4 and 11.3) in 52 to 65% of its size.
pub const AVIF_QUALITY_DEFAULT: u8 = 85;

/// `rav1e`'s speed (1 slowest to 10 fastest): a 1920x1080 frame in 0.4 to 0.8 s (its assembly
/// makes it 20 to 30% faster than without, the same file); 8 made the same file, 4 a slightly better
/// one in twice the time, 10 a worse one.
pub const AVIF_SPEED: u8 = 6;

/// The largest side an AVIF export may have (AV1 holds 16 bits a side).
pub const AVIF_MAX_SIDE: u32 = 65_535;

/// The TIFF tag the metadata is in: ImageDescription.
const TAG_DESCRIPTION: u16 = 0x010E;
/// The largest Exif item read back.
const EXIF_MAX: u64 = 64 << 20;

/// Write the renderer's display-space (sRGB) RGBA `f32` buffer as an AVIF at `quality` (1 to 100,
/// clamped), with `metadata` inside (see the module note), atomically. A side past
/// [`AVIF_MAX_SIDE`] is refused.
pub fn write_avif(path: &Path, width: u32, height: u32, rgba: &[f32], metadata: Option<&str>, quality: u8) -> Result<(), ExportError> {
    let expected = width as usize * height as usize * 4;
    if rgba.len() < expected {
        return Err(ExportError::SizeMismatch { expected, got: rgba.len() });
    }
    for (what, v) in [("AVIF width", width), ("AVIF height", height)] {
        if v > AVIF_MAX_SIDE {
            return Err(ExportError::TooLarge { what, value: v as u64, limit: AVIF_MAX_SIDE as u64 });
        }
    }
    let rgb: Vec<ravif::RGB8> = crate::to_srgb8(&rgba[..expected]).chunks_exact(4).map(|p| ravif::RGB8::new(p[0], p[1], p[2])).collect();
    let mut encoder = ravif::Encoder::new().with_quality(quality.clamp(1, 100) as f32).with_speed(AVIF_SPEED);
    if let Some(m) = metadata {
        encoder = encoder.with_exif(tiff_with_description(m.as_bytes()));
    }
    let encoded = encoder.encode_rgb(ravif::Img::new(&rgb, width as usize, height as usize)).map_err(avif_error)?;
    crate::write_atomic(path, |w| Ok(w.write_all(&encoded.avif_file)?))
}

fn avif_error(e: impl std::fmt::Display) -> ExportError {
    ExportError::Avif(e.to_string())
}

/// A little-endian TIFF block whose IFD0 holds one ImageDescription (ASCII type, the text and its
/// terminating NUL).
fn tiff_with_description(text: &[u8]) -> Vec<u8> {
    let count = text.len() as u32 + 1;
    let mut t = Vec::with_capacity(26 + text.len() + 1);
    t.extend_from_slice(b"II\x2a\0");
    t.extend_from_slice(&8u32.to_le_bytes()); // IFD0 at 8
    t.extend_from_slice(&1u16.to_le_bytes()); // one entry
    t.extend_from_slice(&TAG_DESCRIPTION.to_le_bytes());
    t.extend_from_slice(&2u16.to_le_bytes()); // ASCII
    t.extend_from_slice(&count.to_le_bytes());
    t.extend_from_slice(&26u32.to_le_bytes()); // the text right after the IFD
    t.extend_from_slice(&0u32.to_le_bytes()); // no next IFD
    t.extend_from_slice(text);
    t.push(0);
    t
}

/// The ImageDescription of a TIFF block, or `None` when it has none.
fn tiff_description(t: &[u8]) -> Result<Option<Vec<u8>>, ExportError> {
    let bad = || avif_error("its Exif data is damaged");
    let le = match t.get(..4) {
        Some(b"II\x2a\0") => true,
        Some(b"MM\0\x2a") => false,
        _ => return Err(bad()),
    };
    let u16_at = |i: usize| t.get(i..i + 2).map(|b| if le { u16::from_le_bytes([b[0], b[1]]) } else { u16::from_be_bytes([b[0], b[1]]) });
    let u32_at = |i: usize| t.get(i..i + 4).map(|b| if le { u32::from_le_bytes([b[0], b[1], b[2], b[3]]) } else { u32::from_be_bytes([b[0], b[1], b[2], b[3]]) });
    let ifd = u32_at(4).ok_or_else(bad)? as usize;
    let n = u16_at(ifd).ok_or_else(bad)? as usize;
    for e in 0..n {
        let at = ifd + 2 + e * 12;
        let (tag, typ, count) = (u16_at(at).ok_or_else(bad)?, u16_at(at + 2).ok_or_else(bad)?, u32_at(at + 4).ok_or_else(bad)? as usize);
        if tag != TAG_DESCRIPTION || typ != 2 {
            continue;
        }
        let start = if count <= 4 { at + 8 } else { u32_at(at + 8).ok_or_else(bad)? as usize };
        let mut text = t.get(start..start.checked_add(count).ok_or_else(bad)?).ok_or_else(bad)?.to_vec();
        while text.last() == Some(&0) {
            text.pop();
        }
        return Ok(Some(text));
    }
    Ok(None)
}

/// One ISO-BMFF box's header: its type, where its body starts and how long it is.
fn box_header<R: Read + Seek>(r: &mut R, end: u64) -> Result<Option<([u8; 4], u64, u64)>, ExportError> {
    let at = r.stream_position()?;
    if at + 8 > end {
        return Ok(None);
    }
    let mut h = [0u8; 8];
    r.read_exact(&mut h)?;
    let typ = [h[4], h[5], h[6], h[7]];
    let (body, len) = match u32::from_be_bytes([h[0], h[1], h[2], h[3]]) {
        0 => (at + 8, end - (at + 8)),
        1 => {
            let mut l = [0u8; 8];
            r.read_exact(&mut l)?;
            let size = u64::from_be_bytes(l);
            (at + 16, size.checked_sub(16).ok_or_else(|| avif_error("a box shorter than its header"))?)
        }
        size => (at + 8, (size as u64).checked_sub(8).ok_or_else(|| avif_error("a box shorter than its header"))?),
    };
    if body + len > end {
        return Err(avif_error("a box runs past its container"));
    }
    Ok(Some((typ, body, len)))
}

/// Read `n` bytes (up to 8) big-endian as a number.
fn be(b: &[u8]) -> u64 {
    b.iter().fold(0u64, |v, &x| (v << 8) | x as u64)
}

/// The view metadata an AVIF export carries (`Ok(None)` for an AVIF without it: another program's,
/// or one written without it). Reads the container's item boxes and the Exif item only.
pub fn read_avif_metadata(path: &Path) -> Result<Option<String>, ExportError> {
    crate::check_file_bytes(path)?;
    let mut f = std::io::BufReader::new(std::fs::File::open(path)?);
    let end = f.seek(SeekFrom::End(0))?;
    f.seek(SeekFrom::Start(0))?;
    // ftyp first, of an AVIF brand.
    let Some((typ, body, len)) = box_header(&mut f, end)? else { return Err(avif_error("not an AVIF file")) };
    let mut ftyp = vec![0u8; len.min(256) as usize];
    f.read_exact(&mut ftyp)?;
    if &typ != b"ftyp" || !ftyp.chunks(4).any(|b| b == b"avif" || b == b"avis") {
        return Err(avif_error("not an AVIF file (no AVIF brand)"));
    }
    f.seek(SeekFrom::Start(body + len))?;
    // The meta box, whole (it is small: the item tables and properties).
    let meta = loop {
        let Some((typ, body, len)) = box_header(&mut f, end)? else { return Ok(None) };
        if &typ == b"meta" {
            if len > 16 << 20 {
                return Err(avif_error("an implausibly large meta box"));
            }
            let mut m = vec![0u8; len as usize];
            f.read_exact(&mut m)?;
            break m;
        }
        f.seek(SeekFrom::Start(body + len))?;
    };
    // Its children, after the full box's version and flags.
    let mut c = std::io::Cursor::new(&meta[..]);
    c.seek(SeekFrom::Start(4))?;
    let mend = meta.len() as u64;
    let (mut iinf, mut iloc, mut idat) = (None, None, None);
    while let Some((typ, body, len)) = box_header(&mut c, mend)? {
        let b = meta[body as usize..(body + len) as usize].to_vec();
        match &typ {
            b"iinf" => iinf = Some(b),
            b"iloc" => iloc = Some(b),
            b"idat" => idat = Some(b),
            _ => {}
        }
        c.seek(SeekFrom::Start(body + len))?;
    }
    let (Some(iinf), Some(iloc)) = (iinf, iloc) else { return Ok(None) };
    let Some(exif_id) = exif_item(&iinf)? else { return Ok(None) };
    let (method, extents) = item_extents(&iloc, exif_id)?.ok_or_else(|| avif_error("its Exif item has no location"))?;
    let mut data = Vec::new();
    for (off, len) in extents {
        if data.len() as u64 + len > EXIF_MAX {
            return Err(avif_error("an implausibly large Exif item"));
        }
        match method {
            0 => {
                f.seek(SeekFrom::Start(off))?;
                let mut b = vec![0u8; len as usize];
                f.read_exact(&mut b)?;
                data.extend(b);
            }
            1 => {
                let d = idat.as_ref().ok_or_else(|| avif_error("its Exif item is in an idat it does not have"))?;
                data.extend_from_slice(d.get(off as usize..(off + len) as usize).ok_or_else(|| avif_error("its Exif item runs past its idat"))?);
            }
            _ => return Err(avif_error("its Exif item is stored a way this build does not read")),
        }
    }
    // The HEIF Exif item: the offset to the TIFF header, then the block.
    let skip = data.get(..4).map(be).ok_or_else(|| avif_error("its Exif item is empty"))? as usize;
    let tiff = data.get(4 + skip..).ok_or_else(|| avif_error("its Exif item is damaged"))?;
    match tiff_description(tiff)? {
        Some(text) => String::from_utf8(text).map(Some).map_err(|_| avif_error("its view metadata is not text")),
        None => Ok(None),
    }
}

/// The ID of the item of type `Exif` in an `iinf` body, if any.
fn exif_item(iinf: &[u8]) -> Result<Option<u32>, ExportError> {
    let version = *iinf.first().ok_or_else(|| avif_error("an empty item table"))?;
    let mut c = std::io::Cursor::new(iinf);
    c.seek(SeekFrom::Start(if version == 0 { 6 } else { 8 }))?;
    let end = iinf.len() as u64;
    while let Some((typ, body, len)) = box_header(&mut c, end)? {
        if &typ == b"infe" {
            let e = &iinf[body as usize..(body + len) as usize];
            let v = *e.first().unwrap_or(&0);
            let (id, ty) = match v {
                2 => (e.get(4..6).map(be), e.get(8..12)),
                3 => (e.get(4..8).map(be), e.get(10..14)),
                _ => (None, None),
            };
            if let (Some(id), Some(b"Exif")) = (id, ty) {
                return Ok(Some(id as u32));
            }
        }
        c.seek(SeekFrom::Start(body + len))?;
    }
    Ok(None)
}

/// An item's construction method and extents `(offset, length)` from an `iloc` body.
#[allow(clippy::type_complexity)]
fn item_extents(iloc: &[u8], want: u32) -> Result<Option<(u8, Vec<(u64, u64)>)>, ExportError> {
    let bad = || avif_error("its item locations are damaged");
    let mut i = 0usize;
    let mut take = |n: usize| -> Result<u64, ExportError> {
        let b = iloc.get(i..i + n).ok_or_else(bad)?;
        i += n;
        Ok(be(b))
    };
    let version = take(4)? >> 24;
    let sizes = take(1)?;
    let (offset_size, length_size) = ((sizes >> 4) as usize, (sizes & 15) as usize);
    let sizes2 = take(1)?;
    let (base_size, index_size) = ((sizes2 >> 4) as usize, if version >= 1 { (sizes2 & 15) as usize } else { 0 });
    let count = take(if version < 2 { 2 } else { 4 })?;
    for _ in 0..count {
        let id = take(if version < 2 { 2 } else { 4 })? as u32;
        let method = if version >= 1 { (take(2)? & 15) as u8 } else { 0 };
        let _data_ref = take(2)?;
        let base = take(base_size)?;
        let n = take(2)?;
        let mut extents = Vec::new();
        for _ in 0..n {
            let _index = take(index_size)?;
            let off = take(offset_size)?;
            let len = take(length_size)?;
            extents.push((base + off, len));
        }
        if id == want {
            return Ok(Some((method, extents)));
        }
    }
    Ok(None)
}

#[cfg(test)]
#[path = "../tests/unit/avif/avif_tests.rs"]
mod avif_tests;
