use super::*;

/// A ramp so shallow that plain rounding collapses it into a handful of flat bands — the
/// fractal-exterior case. Dithering must break those bands up while preserving the mean.
fn shallow_ramp(w: usize, h: usize) -> Vec<f32> {
    let mut v = Vec::with_capacity(w * h * 4);
    // Row-invariant by construction: the ramp runs across x only, so every row is identical
    // and the row index is deliberately unused.
    for _y in 0..h {
        for x in 0..w {
            // Span ~4 eight-bit levels across the whole width: ~64 px per band undithered.
            let c = 0.25 + (x as f32 / w as f32) * (4.0 / 255.0);
            v.extend_from_slice(&[c, c, c, 1.0]);
        }
    }
    v
}

fn distinct_in_row(bytes: &[u8], w: usize, row: usize) -> usize {
    let mut seen = [false; 256];
    for x in 0..w {
        seen[bytes[(row * w + x) * 4] as usize] = true;
    }
    seen.iter().filter(|s| **s).count()
}

/// The banding metric the TODO asked for: how long a run of identical output values gets.
fn longest_run(bytes: &[u8], w: usize, row: usize) -> usize {
    let (mut best, mut cur) = (1usize, 1usize);
    for x in 1..w {
        if bytes[(row * w + x) * 4] == bytes[(row * w + x - 1) * 4] {
            cur += 1;
            best = best.max(cur);
        } else {
            cur = 1;
        }
    }
    best
}

#[test]
fn dither_breaks_up_bands() {
    let (w, h) = (256usize, 8usize);
    let src = shallow_ramp(w, h);
    let plain = to_srgb8(&src);
    let dithered = to_srgb8_dithered(&src, w as u32);

    // Undithered, the ramp is a few wide plateaus; dithered, the same row carries more
    // levels and no long flat stretch.
    let plain_run = longest_run(&plain, w, 0);
    let dith_run = longest_run(&dithered, w, 0);
    assert!(plain_run >= 32, "expected wide bands undithered, got {plain_run}");
    // Halved, not quartered: an ordered dither spreads its offsets over the 8x8 tile, so any
    // single ROW sees only a subset of them and a per-row metric understates the effect. The
    // 2D distinct-level count below is the fairer measure. (Measured here: 64 -> 26.)
    assert!(
        dith_run * 2 < plain_run,
        "dither should shorten the longest flat run (plain {plain_run}, dithered {dith_run})"
    );
    let distinct_2d = |b: &[u8]| {
        let mut seen = [false; 256];
        for px in b.chunks_exact(4) {
            seen[px[0] as usize] = true;
        }
        seen.iter().filter(|s| **s).count()
    };
    assert!(
        distinct_2d(&dithered) > distinct_2d(&plain),
        "dither should use more output levels ({} -> {})",
        distinct_2d(&plain),
        distinct_2d(&dithered)
    );
    assert!(
        distinct_in_row(&dithered, w, 0) >= distinct_in_row(&plain, w, 0),
        "a single row should not lose levels"
    );

    // Brightness is preserved: the offsets are centred, so the mean must not shift by more
    // than a fraction of one level.
    let mean = |b: &[u8]| b.iter().step_by(4).map(|v| *v as f64).sum::<f64>() / (w * h) as f64;
    assert!((mean(&plain) - mean(&dithered)).abs() < 0.6);
}

/// Deterministic and position-keyed: the same input always gives the same bytes (goldens and
/// the corpus depend on this), and alpha is never perturbed.
#[test]
fn dither_is_deterministic_and_leaves_alpha_alone() {
    let src = shallow_ramp(64, 4);
    assert_eq!(to_srgb8_dithered(&src, 64), to_srgb8_dithered(&src, 64));
    assert!(to_srgb8_dithered(&src, 64).iter().skip(3).step_by(4).all(|a| *a == 255));
}

/// Fully saturated values must not wrap or overshoot when the offset pushes them past an end.
#[test]
fn dither_clamps_at_the_extremes() {
    let src: Vec<f32> = [0.0f32, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0].to_vec();
    let out = to_srgb8_dithered(&src, 2);
    assert_eq!(&out[0..3], &[0, 0, 0]);
    assert_eq!(&out[4..7], &[255, 255, 255]);
}
