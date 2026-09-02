//! The byte-level writers, round-tripped through their own readers.
//!
//! These had no direct coverage: the goldens exercise the PNG path end to end and incidentally,
//! and nothing at all exercised EXR or the metadata chunks. That is the wrong place to be thin,
//! because a defect here **eats user work silently** — an export that writes the wrong pixels,
//! drops the embedded view state, or double-encodes the transfer curve produces a file that
//! looks plausible and is wrong, and the user finds out when they try to reload it.
//!
//! What each test pins is a property some part of the app depends on, not just "it runs".
use super::*;

/// Throwaway directory, per test, in the OS temp dir (repo convention — no dev-dependency).
struct Tmp(std::path::PathBuf);
impl Tmp {
    fn new(tag: &str) -> Self {
        let d = std::env::temp_dir()
            .join(format!("fractadyne_export_test_{}_{tag}", std::process::id()));
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

/// A small image with a shallow ramp (the dither's domain), saturated corners, and a
/// non-uniform alpha — so a channel swap, a row-stride error or an alpha transform all show.
fn sample(w: u32, h: u32) -> Vec<f32> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let t = (y * w + x) as f32 / (w * h) as f32;
            v.extend_from_slice(&[t, 0.5 * t, 1.0 - t, if x == 0 { 0.5 } else { 1.0 }]);
        }
    }
    v
}

/// ⭐The identity every golden comparison rests on: what `write_png` stores is EXACTLY
/// `to_srgb8_dithered` of the buffer. If these ever diverge, every golden fails for a reason
/// that has nothing to do with rendering — which is a far more confusing failure than a wrong
/// picture, and the comment on `write_png` promises it explicitly.
#[test]
fn png_bytes_are_exactly_the_dithered_conversion() {
    let (w, h) = (16u32, 9u32);
    let src = sample(w, h);
    let t = Tmp::new("png_identity");
    let p = t.path("a.png");
    write_png(&p, w, h, &src, None).expect("write");
    let (rw, rh, got) = read_png_rgba8(&p).expect("read");
    assert_eq!((rw, rh), (w, h));
    assert_eq!(got, to_srgb8_dithered(&src, w), "PNG bytes drifted from the shared conversion");
}

/// `write_png_rgba8` exists precisely because `write_png` would re-apply a transfer curve to
/// pixels that are already in display space (the egui framebuffer capture case). Bytes in must
/// be bytes out — a double-encode is the bug this function was added to prevent.
#[test]
fn png_rgba8_writes_the_bytes_it_was_given() {
    let (w, h) = (4u32, 2u32);
    let src: Vec<u8> = (0..(w * h * 4) as u8).map(|i| i.wrapping_mul(7)).collect();
    let t = Tmp::new("png_rgba8");
    let p = t.path("b.png");
    write_png_rgba8(&p, w, h, &src, None).expect("write");
    let (_, _, got) = read_png_rgba8(&p).expect("read");
    assert_eq!(got, src, "8-bit path must not re-encode");
}

/// The embedded view state is what makes an exported image reloadable. It travels as a PNG
/// `tEXt` chunk, so it must survive verbatim — including the newlines and `=` of the TOML-ish
/// payload the app actually stores.
#[test]
fn png_metadata_roundtrips_verbatim() {
    let (w, h) = (3u32, 3u32);
    let meta = "center_x = \"-0.743643887037151\"\ncenter_y = \"0.131825904205330\"\n\
                zoom = \"1e30\"\nmax_iter = 4000000\n";
    let t = Tmp::new("png_meta");
    let p = t.path("c.png");
    write_png(&p, w, h, &sample(w, h), Some(meta)).expect("write");
    assert_eq!(read_png_metadata(&p).expect("read meta").as_deref(), Some(meta));
    // …and absence is None, not an empty string: the caller distinguishes "no view state" from
    // "a view state that happens to be blank".
    let q = t.path("d.png");
    write_png(&q, w, h, &sample(w, h), None).expect("write");
    assert_eq!(read_png_metadata(&q).expect("read meta"), None);
}

/// EXR is the LINEAR master: RGB is converted out of display space on write, alpha is not.
/// (An alpha that went through the transfer curve would make every composite subtly wrong.)
#[test]
fn exr_is_linear_rgb_with_untouched_alpha() {
    let (w, h) = (5u32, 4u32);
    let src = sample(w, h);
    let t = Tmp::new("exr_linear");
    let p = t.path("e.exr");
    write_exr(&p, w, h, &src, None).expect("write");
    let (rw, rh, got) = read_exr_rgba_f32(&p).expect("read");
    assert_eq!((rw, rh), (w, h));
    for i in 0..(w * h) as usize {
        for c in 0..3 {
            let want = srgb_to_linear(src[i * 4 + c]);
            assert!(
                (got[i * 4 + c] - want).abs() < 1e-6,
                "px {i} ch {c}: {} vs {want}",
                got[i * 4 + c]
            );
        }
        assert!(
            (got[i * 4 + 3] - src[i * 4 + 3]).abs() < 1e-6,
            "alpha must pass through untransformed"
        );
    }
}

/// The same reloadable-view promise as PNG, on the EXR side (a custom image attribute), plus
/// the channel discovery the Fraktaler-3 cross-check reads through.
#[test]
fn exr_metadata_and_channels_roundtrip() {
    let (w, h) = (2u32, 2u32);
    let meta = "zoom = \"4.6e1105\"\n";
    let t = Tmp::new("exr_meta");
    let p = t.path("f.exr");
    write_exr(&p, w, h, &sample(w, h), Some(meta)).expect("write");
    assert_eq!(read_exr_metadata(&p).expect("read meta").as_deref(), Some(meta));
    let mut chans = list_exr_channels(&p).expect("channels");
    chans.sort();
    assert_eq!(chans, vec!["A", "B", "G", "R"]);
    // Reading one channel by name must agree with the same channel of the full decode.
    let (_, _, all) = read_exr_rgba_f32(&p).expect("read rgba");
    let (_, _, red) = read_exr_channel_f32(&p, "R").expect("read R");
    for i in 0..(w * h) as usize {
        assert!((red[i] - all[i * 4]).abs() < 1e-6);
    }
    assert!(
        matches!(read_exr_channel_f32(&p, "N"), Err(ExportError::ChannelNotFound(_))),
        "a missing channel must be a typed error, not a panic or an empty image"
    );
}

/// A short buffer is a caller bug, and the writers must refuse it rather than encode whatever
/// happens to be in memory — and must not leave a half-written file behind for `--resume` to
/// later mistake for a finished frame.
#[test]
fn a_short_buffer_is_refused_and_writes_nothing() {
    let (w, h) = (8u32, 8u32);
    let short = vec![0.5f32; (w * h * 4 - 1) as usize];
    let t = Tmp::new("short");
    for (name, r) in [
        ("g.png", write_png(&t.path("g.png"), w, h, &short, None)),
        ("h.exr", write_exr(&t.path("h.exr"), w, h, &short, None)),
    ] {
        assert!(matches!(r, Err(ExportError::SizeMismatch { .. })), "{name} accepted a short buffer");
        assert!(!t.path(name).exists(), "{name} was created despite the error");
    }
    let short8 = vec![0u8; (w * h * 4 - 1) as usize];
    assert!(matches!(
        write_png_rgba8(&t.path("i.png"), w, h, &short8, None),
        Err(ExportError::SizeMismatch { .. })
    ));
}

/// Decoders are fed files the app did not write (a user's `--compare` argument, a dropped
/// file). Garbage must be a typed error, never a panic.
#[test]
fn corrupt_input_is_an_error_not_a_panic() {
    let t = Tmp::new("corrupt");
    assert!(read_png_rgba8_bytes(b"not a png at all").is_err());
    assert!(read_png_rgba8_bytes(&[]).is_err());
    // Truncation — the shape a killed render leaves on disk.
    let (w, h) = (4u32, 4u32);
    let p = t.path("j.png");
    write_png(&p, w, h, &sample(w, h), None).expect("write");
    let full = std::fs::read(&p).expect("read bytes");
    // Cuts into the header, the image data, or the whole IEND chunk are all rejected.
    for cut in [8, 24, full.len() / 2, full.len() - 12] {
        assert!(
            read_png_rgba8_bytes(&full[..cut]).is_err(),
            "a PNG truncated to {cut} of {} bytes decoded successfully",
            full.len()
        );
    }
    // ⭐**One truncation the decoder DOES accept: losing the last byte** — IEND's final CRC
    // byte, which it never verifies. Measured, not assumed. It is a narrow window, but it is
    // real, and it is the shape an interrupted write leaves: the file is one byte short of
    // complete and decodes into a perfectly good image. This is why the tour renderer's
    // `--resume` vetting (`png_frame_size`) tests for the exact 12-byte `IEND` ITSELF rather
    // than trusting a successful decode — a frame like this would otherwise look finished
    // forever and get baked into the middle of a nine-thousand-frame sequence, which is the
    // disk-full failure at frame 1091 that put that check there.
    assert!(
        read_png_rgba8_bytes(&full[..full.len() - 1]).is_ok(),
        "behaviour changed: the decoder now rejects a PNG missing IEND's last CRC byte. That is \
         stricter, and welcome — but the resume vetter's own IEND check must stay, because it \
         is what makes the guarantee rather than the decoder's leniency happening to be narrow."
    );
    let q = t.path("k.exr");
    std::fs::write(&q, b"\x76\x2f\x31\x01garbage").expect("write");
    assert!(read_exr_rgba_f32(&q).is_err());
    assert!(read_exr_metadata(&q).is_err());
}
