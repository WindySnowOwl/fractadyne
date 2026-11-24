//! The GIF writer and its readers: the palette size asked for, the picture within what that many
//! colors allow (closer with more), dithering that spreads a ramp's steps, the nearest-color search
//! exact, and the view's metadata exactly.
use super::*;

/// Throwaway directory, per test, in the OS temp dir (repo convention).
struct Tmp(std::path::PathBuf);
impl Tmp {
    fn new(tag: &str) -> Self {
        let d = std::env::temp_dir().join(format!("fractadyne_gif_test_{}_{tag}", std::process::id()));
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

/// A smooth ramp over the whole width, a different hue in each band of rows.
fn picture(w: u32, h: u32) -> Vec<f32> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let t = x as f32 / (w - 1) as f32;
            let (r, g, b) = match (y * 3) / h {
                0 => (t, 0.3 * t, 0.1),
                1 => (0.2, t, 1.0 - t),
                _ => (1.0 - 0.5 * t, 0.8 * t, t),
            };
            v.extend_from_slice(&[r, g, b, 1.0]);
        }
    }
    v
}

/// The mean difference (8-bit levels) from the picture's rounded colors.
fn error(rgba: &[u8], src: &[f32]) -> f64 {
    let want = crate::to_srgb8(src);
    let mut sum = 0.0;
    for (p, q) in rgba.chunks_exact(4).zip(want.chunks_exact(4)) {
        sum += (0..3).map(|k| p[k].abs_diff(q[k]) as f64).sum::<f64>();
    }
    sum / (3 * rgba.len() / 4) as f64
}

#[test]
fn a_gif_has_the_colors_asked_for_and_more_get_closer() {
    let t = Tmp::new("colors");
    let (w, h) = (240u32, 90u32);
    let px = picture(w, h);
    let mut last = f64::MAX;
    for n in [2u16, 4, 16, 64, 256] {
        let p = t.path(&format!("n{n}.gif"));
        write_gif(&p, w, h, &px, None, n, false).expect("written");
        let (rw, rh, rgba) = read_gif_rgba8(&p).expect("read back");
        assert_eq!((rw, rh), (w, h));
        let distinct: std::collections::HashSet<[u8; 3]> = rgba.chunks_exact(4).map(|c| [c[0], c[1], c[2]]).collect();
        assert!(distinct.len() <= n as usize, "{n} colors asked, {} drawn", distinct.len());
        let e = error(&rgba, &px);
        assert!(e < last, "{n} colors: {e:.2} levels off, not closer than {last:.2}");
        last = e;
    }
    assert!(last <= 2.0, "256 colors: {last:.2} levels off on average");
}

#[test]
fn dithering_spreads_a_ramps_steps_into_its_pixels() {
    // With few colors, undithered, a ramp is a staircase: long runs of one color. Dithered, the
    // average over a short run of pixels follows the ramp; the control is the undithered one.
    let t = Tmp::new("dither");
    let (w, h) = (256u32, 30u32);
    let px = picture(w, h);
    let mut run_err = [0.0f64; 2];
    for (i, dither) in [false, true].into_iter().enumerate() {
        let p = t.path(&format!("d{i}.gif"));
        write_gif(&p, w, h, &px, None, 8, dither).unwrap();
        let (_, _, rgba) = read_gif_rgba8(&p).unwrap();
        let want = crate::to_srgb8(&px);
        // 8x8 block means against the picture's.
        let (mut s, mut n) = (0.0f64, 0usize);
        for by in (0..h as usize - 8).step_by(8) {
            for bx in (0..w as usize - 8).step_by(8) {
                for k in 0..3 {
                    let (mut a, mut b) = (0.0f64, 0.0f64);
                    for y in by..by + 8 {
                        for x in bx..bx + 8 {
                            a += rgba[(y * w as usize + x) * 4 + k] as f64;
                            b += want[(y * w as usize + x) * 4 + k] as f64;
                        }
                    }
                    s += (a - b).abs() / 64.0;
                    n += 1;
                }
            }
        }
        run_err[i] = s / n as f64;
    }
    assert!(run_err[1] < run_err[0] * 0.6, "dithered blocks {:.2} levels off, undithered {:.2}", run_err[1], run_err[0]);
}

#[test]
fn the_nearest_color_search_is_exact() {
    let mut seed = 0x2545_f491_4f6c_dd1du64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % 256) as f32
    };
    for n in [2usize, 7, 64, 256] {
        let pal: Vec<[f32; 3]> = (0..n).map(|_| [next(), next(), next()]).collect();
        let near = Nearest::new(&pal);
        for _ in 0..2000 {
            let c = [next(), next(), next()];
            let d2 = |p: &[f32; 3]| (p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2);
            let best = pal.iter().map(d2).fold(f32::INFINITY, f32::min);
            assert_eq!(d2(&pal[near.index(c)]), best, "{n} colors at {c:?}");
        }
    }
}

#[test]
fn the_view_metadata_round_trips_exactly() {
    let t = Tmp::new("meta");
    let (w, h) = (24u32, 12u32);
    let px = picture(w, h);
    let deep = format!("app=Fractadyne\ncenter_re=-1.{}\nnotes=ÄÖ ✓\n", "3".repeat(120_000));
    for (name, text) in [("small", "app=Fractadyne\nzoom=8\n".to_string()), ("deep", deep)] {
        let p = t.path(&format!("{name}.gif"));
        write_gif(&p, w, h, &px, Some(&text), 16, true).expect("written");
        assert_eq!(read_gif_metadata(&p).expect("read").as_deref(), Some(text.as_str()), "{name}");
        assert_eq!(read_gif_rgba8(&p).expect("decoded").0, w, "{name}: still a picture");
    }
    let p = t.path("none.gif");
    write_gif(&p, w, h, &px, None, 16, true).unwrap();
    assert_eq!(read_gif_metadata(&p).unwrap(), None);
    let q = t.path("not.gif");
    std::fs::write(&q, b"\x89PNG\r\n\x1a\n.....").unwrap();
    assert!(read_gif_metadata(&q).is_err());
}

#[test]
fn the_palette_size_is_clamped_and_a_side_past_the_limit_refused() {
    let t = Tmp::new("limits");
    let (w, h) = (20u32, 10u32);
    let px = picture(w, h);
    let p = t.path("one.gif");
    write_gif(&p, w, h, &px, None, 1, false).unwrap();
    let (_, _, rgba) = read_gif_rgba8(&p).unwrap();
    let distinct: std::collections::HashSet<[u8; 3]> = rgba.chunks_exact(4).map(|c| [c[0], c[1], c[2]]).collect();
    assert!(distinct.len() <= 2, "1 color asked: clamped to 2, {} drawn", distinct.len());
    let e = write_gif(&t.path("wide.gif"), GIF_MAX_SIDE + 1, 1, &vec![0.0; (GIF_MAX_SIDE as usize + 1) * 4], None, 16, false).unwrap_err();
    assert!(e.to_string().contains("GIF width"), "{e}");
    let (tw, th, thumb) = crate::read_thumbnail(&p, 10).unwrap();
    assert_eq!((tw, th, thumb.len()), (10, 5, 200));
}
