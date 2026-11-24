//! The JPEG writer and its readers, round-tripped: the picture within its quality, the view's
//! metadata exactly (a deep view's across many segments), and every way the file can say less than
//! it should refused rather than half-read.
use super::*;

/// Throwaway directory, per test, in the OS temp dir (repo convention).
struct Tmp(std::path::PathBuf);
impl Tmp {
    fn new(tag: &str) -> Self {
        let d = std::env::temp_dir().join(format!("fractadyne_jpeg_test_{}_{tag}", std::process::id()));
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

/// A picture with what JPEG finds hard and easy: a smooth two-color ramp across the left half, and
/// a fine pattern of one-pixel lines in two different hues across the right (a fractal's filaments).
fn picture(w: u32, h: u32) -> Vec<f32> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let (r, g, b) = if x < w / 2 {
                let t = x as f32 / (w / 2) as f32;
                (0.1 + 0.7 * t, 0.2 + 0.3 * t, 0.6 - 0.4 * t)
            } else if (x + y) % 3 == 0 {
                (0.9, 0.3, 0.1)
            } else {
                (0.1, 0.4, 0.8)
            };
            v.extend_from_slice(&[r, g, b, 1.0]);
        }
    }
    v
}

/// The mean and largest difference (8-bit levels) between a decoded JPEG and the picture's rounded
/// colors, over one half of the picture.
fn error(rgb: &[u8], src: &[f32], w: u32, right_half: bool) -> (f64, u8) {
    let want = crate::to_srgb8(src);
    let (mut sum, mut n, mut worst) = (0.0f64, 0usize, 0u8);
    for (i, (p, q)) in rgb.chunks_exact(3).zip(want.chunks_exact(4)).enumerate() {
        if ((i as u32 % w) >= w / 2) != right_half {
            continue;
        }
        for k in 0..3 {
            let d = p[k].abs_diff(q[k]);
            sum += d as f64;
            worst = worst.max(d);
            n += 1;
        }
    }
    (sum / n.max(1) as f64, worst)
}

#[test]
fn a_jpeg_is_the_picture_within_its_quality() {
    let t = Tmp::new("quality");
    let (w, h) = (192u32, 96u32);
    let px = picture(w, h);
    let mut last = (f64::MAX, 0u64);
    for q in [30u8, 60, 90, 98] {
        let p = t.path(&format!("q{q}.jpg"));
        write_jpeg(&p, w, h, &px, None, q).expect("written");
        let (rw, rh, rgb) = read_jpeg_rgb8(&p).expect("read back");
        assert_eq!((rw, rh), (w, h));
        let (ramp, _) = error(&rgb, &px, w, false);
        let (lines, worst) = error(&rgb, &px, w, true);
        let size = std::fs::metadata(&p).unwrap().len();
        // Higher quality: closer, and bigger.
        assert!(lines < last.0, "quality {q}: the lines {lines:.2} levels off, not closer than {:.2}", last.0);
        assert!(size > last.1, "quality {q}: {size} bytes, not more than {}", last.1);
        last = (lines, size);
        if q >= 90 {
            // Full color resolution: one-pixel lines in two hues keep their colors (4:2:0 halves
            // them: the measured 25 levels at the busiest pixels of a real export).
            assert!(lines <= 6.0, "quality {q}: the lines {lines:.2} levels off on average (max {worst})");
            assert!(ramp <= 1.5, "quality {q}: the ramp {ramp:.2} levels off on average");
        }
    }
}

#[test]
fn the_view_metadata_round_trips_exactly_even_past_one_segment() {
    let t = Tmp::new("meta");
    let (w, h) = (16u32, 8u32);
    let px = picture(w, h);
    // A small view, and a deep one whose coordinates run past several segments (and through
    // characters outside ASCII).
    let small = "app=Fractadyne\nzoom=2e3\nnotes=ruta ÄÖ ✓\n".to_string();
    let deep = format!("app=Fractadyne\ncenter_re=-1.{}\nnotes=☃\n", "7".repeat(200_000));
    for (name, text) in [("small", &small), ("deep", &deep)] {
        let p = t.path(&format!("{name}.jpg"));
        write_jpeg(&p, w, h, &px, Some(text), 90).expect("written");
        assert_eq!(read_jpeg_metadata(&p).expect("read").as_deref(), Some(text.as_str()), "{name}");
        // The picture is still a picture after them.
        assert_eq!(read_jpeg_rgb8(&p).expect("decoded").0, w, "{name}");
    }
    // Written without metadata: none, not an error.
    let p = t.path("none.jpg");
    write_jpeg(&p, w, h, &px, None, 90).unwrap();
    assert_eq!(read_jpeg_metadata(&p).unwrap(), None);
}

#[test]
fn damaged_metadata_is_refused_not_half_read() {
    let t = Tmp::new("damaged");
    let (w, h) = (16u32, 8u32);
    let text = format!("app=Fractadyne\nx={}\n", "1".repeat(150_000));
    let p = t.path("good.jpg");
    write_jpeg(&p, w, h, &picture(w, h), Some(&text), 90).unwrap();
    let good = std::fs::read(&p).unwrap();
    // The metadata segments, by where they start.
    let starts: Vec<usize> = good.windows(2 + 2 + META_ID.len()).enumerate().filter(|(_, s)| s[0] == 0xFF && s[1] == META_MARKER && &s[4..] == META_ID).map(|(i, _)| i).collect();
    assert_eq!(starts.len(), 3, "150,000 bytes take three segments");
    let seg_len = |at: usize| 2 + u16::from_be_bytes([good[at + 2], good[at + 3]]) as usize;
    // A segment cut out.
    let mut missing = good.clone();
    missing.drain(starts[1]..starts[1] + seg_len(starts[1]));
    // A segment's number past the count.
    let mut renumbered = good.clone();
    renumbered[starts[2] + 4 + META_ID.len()] = 9;
    // Counts that disagree.
    let mut recounted = good.clone();
    recounted[starts[0] + 4 + META_ID.len() + 1] = 4;
    for (name, bytes) in [("missing", missing), ("renumbered", renumbered), ("recounted", recounted)] {
        let q = t.path(&format!("{name}.jpg"));
        std::fs::write(&q, &bytes).unwrap();
        assert!(read_jpeg_metadata(&q).is_err(), "{name} was accepted");
    }
    // Not a JPEG at all.
    let q = t.path("not.jpg");
    std::fs::write(&q, b"\x89PNG\r\n\x1a\n").unwrap();
    assert!(read_jpeg_metadata(&q).is_err());
}

#[test]
fn a_side_past_the_formats_limit_is_refused() {
    let t = Tmp::new("limit");
    let p = t.path("wide.jpg");
    // The buffer is not looked at: the side is refused first.
    let e = write_jpeg(&p, JPEG_MAX_SIDE + 1, 1, &vec![0.0; (JPEG_MAX_SIDE as usize + 1) * 4], None, 90).unwrap_err();
    assert!(e.to_string().contains("JPEG width"), "{e}");
    assert!(!p.exists(), "nothing written");
}

#[test]
fn a_jpeg_has_a_thumbnail() {
    let t = Tmp::new("thumb");
    let (w, h) = (300u32, 150u32);
    let p = t.path("pic.jpeg");
    write_jpeg(&p, w, h, &picture(w, h), None, 90).unwrap();
    let (tw, th, rgba) = crate::read_thumbnail(&p, 100).expect("a thumbnail");
    assert_eq!((tw, th), (100, 50));
    assert_eq!(rgba.len(), (tw * th * 4) as usize);
    assert!(rgba.chunks_exact(4).all(|p| p[3] == 255), "opaque");
}
