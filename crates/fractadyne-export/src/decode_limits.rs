//! F-01 decode limits: a crafted or corrupt image header must be REJECTED from its declared
//! dimensions, before any full-resolution allocation — never OOM the process.
//!
//! The audit's concrete case is a PNG whose IHDR claims 100000×100000 (≈40 GB decoded). The test
//! below builds exactly that (a valid-CRC header, an IDAT stub so the decoder reaches the header
//! check) and asserts the decoder returns `TooLarge`, not a multi-gigabyte allocation. The valid
//! round-trips guard the other direction: the caps must not touch a real export.
use super::*;

/// Throwaway directory, per test, in the OS temp dir (repo convention — no dev-dependency).
struct Tmp(std::path::PathBuf);
impl Tmp {
    fn new(tag: &str) -> Self {
        let d = std::env::temp_dir()
            .join(format!("fractadyne_export_limits_{}_{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("temp dir");
        Self(d)
    }
    fn path(&self, name: &str) -> std::path::PathBuf {
        self.0.join(name)
    }
}
impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Standard PNG/zlib CRC-32 (IEEE, reflected, poly 0xEDB8_8320) over a chunk's type+data — so the
/// crafted bytes are a *well-formed* PNG the decoder parses, proving OUR guard rejected it rather
/// than the decoder tripping over a corrupt CRC.
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in bytes {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

/// Append a PNG chunk (`len` big-endian, type, data, CRC over type+data) to `out`.
fn push_chunk(out: &mut Vec<u8>, ty: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(ty);
    out.extend_from_slice(data);
    let mut crc_input = Vec::with_capacity(4 + data.len());
    crc_input.extend_from_slice(ty);
    crc_input.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc_input).to_be_bytes());
}

/// A syntactically valid PNG whose IHDR declares `w`×`h` (8-bit RGBA), followed by an IDAT stub so
/// the decoder reaches — and returns — its header info. The IDAT data is never decoded: the size
/// check fires from the header first.
fn png_with_declared_dims(w: u32, h: u32) -> Vec<u8> {
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // depth 8, color type 6 (RGBA), no compression/filter/interlace
    push_chunk(&mut out, b"IHDR", &ihdr);
    push_chunk(&mut out, b"IDAT", &[0x78, 0x01]); // zlib header stub; not decoded
    push_chunk(&mut out, b"IEND", &[]);
    out
}

#[test]
fn png_header_claiming_absurd_dims_is_rejected_not_oomed() {
    // The audit's case: 100000×100000 RGBA8 ≈ 40 GB. Each axis is already past the per-dimension
    // cap, so it's refused on `width` from the header — no allocation attempted.
    let bytes = png_with_declared_dims(100_000, 100_000);
    match read_png_rgba8_bytes(&bytes) {
        Err(ExportError::TooLarge { what, .. }) => assert_eq!(what, "width"),
        other => panic!("expected TooLarge, got {other:?}"),
    }
}

#[test]
fn png_header_within_axis_caps_but_over_pixel_cap_is_rejected() {
    // 40000×40000 = 1.6 Gpix: each axis is under the 65535 dimension cap, so this exercises the
    // total-pixel cap specifically, driven through the real decoder from the header.
    let bytes = png_with_declared_dims(40_000, 40_000);
    match read_png_rgba8_bytes(&bytes) {
        Err(ExportError::TooLarge { what, .. }) => assert_eq!(what, "pixels"),
        other => panic!("expected TooLarge(pixels), got {other:?}"),
    }
}

#[test]
fn png_header_with_one_absurd_dimension_is_rejected() {
    // Width alone past the per-dimension cap (height modest), so the dimension check fires first.
    let bytes = png_with_declared_dims(200_000, 4);
    match read_png_rgba8_bytes(&bytes) {
        Err(ExportError::TooLarge { what, .. }) => assert_eq!(what, "width"),
        other => panic!("expected TooLarge(width), got {other:?}"),
    }
}

#[test]
fn valid_png_still_decodes() {
    let tmp = Tmp::new("png_ok");
    let p = tmp.path("ok.png");
    let (w, h) = (8u32, 6u32);
    let rgba8: Vec<u8> = (0..w * h * 4).map(|i| (i % 256) as u8).collect();
    write_png_rgba8(&p, w, h, &rgba8, None).expect("write");
    let (dw, dh, _) = read_png_rgba8(&p).expect("a real image within the caps must decode");
    assert_eq!((dw, dh), (w, h));
}

#[test]
fn valid_exr_still_decodes() {
    let tmp = Tmp::new("exr_ok");
    let p = tmp.path("ok.exr");
    let (w, h) = (8u32, 6u32);
    let rgba: Vec<f32> = (0..w * h * 4).map(|i| (i as f32) / 100.0).collect();
    write_exr(&p, w, h, &rgba, None).expect("write");
    let (dw, dh, _) = read_exr_rgba_f32(&p).expect("a real EXR within the caps must decode");
    assert_eq!((dw, dh), (w, h));
    // And the header pre-check on the same file must agree it's within limits.
    check_exr_dims(&p).expect("valid EXR dims pass the header pre-check");
}

#[test]
fn check_dims_bounds_each_axis_and_the_product() {
    let lim = ImageLimits::new();
    assert!(lim.check_dims(4096, 4096).is_ok(), "a 4K-class image is fine");
    assert!(matches!(
        lim.check_dims(lim.max_dim + 1, 1),
        Err(ExportError::TooLarge { what: "width", .. })
    ));
    assert!(matches!(
        lim.check_dims(1, lim.max_dim + 1),
        Err(ExportError::TooLarge { what: "height", .. })
    ));
    // Both dimensions legal individually, but their product is not (65535² > 512 Mpix).
    assert!(matches!(
        lim.check_dims(lim.max_dim, lim.max_dim),
        Err(ExportError::TooLarge { what: "pixels", .. })
    ));
}

#[test]
fn check_encoded_bytes_rejects_oversized_files() {
    let lim = ImageLimits::new();
    assert!(lim.check_encoded_bytes(lim.max_encoded_bytes).is_ok(), "exactly at the cap is fine");
    assert!(matches!(
        lim.check_encoded_bytes(lim.max_encoded_bytes + 1),
        Err(ExportError::TooLarge { what: "encoded bytes", .. })
    ));
}
