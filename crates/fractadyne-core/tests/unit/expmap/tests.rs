//! The exponential map's geometry (`design/zoom-video.md` §3), against the published map computed
//! independently in high precision.

use super::*;
use crate::bignum::to_f64;
use astro_float::{BigFloat, Consts, RoundingMode};

const RM: RoundingMode = RoundingMode::ToEven;
const P: usize = 256;

/// `R₀·2^-k·exp(−2π(j + ½)/W)·(cos θ, sin θ)`, `θ = 2π(i + ½)/W`, with `R₀` the half diagonal of a
/// `w × h` frame at magnification `2^start`: all of it in 256-bit arithmetic, from the definitions.
fn published(w: f64, h: f64, start: i32, width: u32, k: u64, i: f64, j: f64) -> (BigFloat, BigFloat, i64) {
    let mut cc = Consts::new().unwrap();
    let bf = |x: f64| BigFloat::from_f64(x, P);
    let diag = bf(w).mul(&bf(w), P, RM).add(&bf(h).mul(&bf(h), P, RM), P, RM).sqrt(P, RM);
    // REFERENCE_HEIGHT · diag / (2h) · 2^-start
    let mut r0 = bf(Viewport::REFERENCE_HEIGHT).mul(&diag, P, RM).div(&bf(2.0 * h), P, RM);
    let e0 = r0.exponent().unwrap();
    r0.set_exponent(e0 - start);
    let two_pi_over_w = cc.pi(P, RM).mul(&bf(2.0), P, RM).div(&bf(width as f64), P, RM);
    let radial = two_pi_over_w.mul(&bf(j + 0.5), P, RM).neg().exp(P, RM, &mut cc);
    let theta = two_pi_over_w.mul(&bf(i + 0.5), P, RM);
    // 2^-k is kept apart as an exponent: at k = 50,000 the value itself is far below f64.
    let r = r0.mul(&radial, P, RM);
    (r.mul(&theta.cos(P, RM, &mut cc), P, RM), r.mul(&theta.sin(P, RM, &mut cc), P, RM), -(k as i64))
}

/// `(m·2^e)` of a `FloatExp` against a `BigFloat` times `2^shift`, as a relative error.
fn rel_err(got: FloatExp, want: &BigFloat, shift: i64) -> f64 {
    let mut w = want.clone();
    let e = w.exponent().unwrap() as i64;
    // Bring both to the same scale: divide out 2^(e + shift) from each.
    w.set_exponent(0);
    let want_m = to_f64(&w);
    let got_m = got.m * 2f64.powi((got.e as i64 - e - shift) as i32);
    ((got_m - want_m) / want_m).abs()
}

#[test]
fn a_texel_lands_where_the_published_map_puts_it() {
    let (w, h) = (1920.0, 1080.0);
    for start in [0, 7] {
        let m = ExpMap::for_frames(w, h, 1.0, start as f64);
        for &k in &[0u64, 1, 13, 1_000, 50_000] {
            for &(i, j) in &[(0.0, 0.0), (1234.0, 17.0), (m.width as f64 - 1.0, m.rows as f64 - 1.0), (3460.0, 382.25)] {
                let (re, im) = m.texel_offset(k, i, j);
                let (want_re, want_im, shift) = published(w, h, start, m.width, k, i, j);
                for (got, want, axis) in [(re, &want_re, "re"), (im, &want_im, "im")] {
                    // A component near zero (θ near a quarter turn) has no relative accuracy to ask
                    // for; judge it against the radius instead.
                    if to_f64(want).abs() < 1e-3 * 2f64.powf(m.outer_log2(0)) {
                        continue;
                    }
                    let err = rel_err(got, want, shift);
                    assert!(err < 1e-12, "start {start}, strip {k}, texel ({i}, {j}) {axis}: relative error {err:e}");
                }
            }
        }
    }
}

#[test]
fn a_strip_is_as_wide_as_the_circle_through_a_frames_corners() {
    let m = ExpMap::for_frames(1920.0, 1080.0, 1.0, 0.0);
    // π × 2202.9 = 6920.6: the design's 1080p figure, rounded up.
    assert_eq!(m.width, 6921);
    assert_eq!(m.rows, (6921.0 * LN_2 / TAU).ceil() as u32 + EXPMAP_OVERLAP_ROWS);
    // At the frame where strip k first appears, its outer row's texels are at most a pixel, and
    // barely less: the corner is sampled one texel a pixel.
    for k in [0u64, 5, 300] {
        let mag = m.frame_log2mag(k, 1920.0, 1080.0);
        let pixel_log2 = (Viewport::REFERENCE_HEIGHT / 1080.0).log2() - mag;
        let texel_log2 = m.texel_size_log2(k, -0.5);
        assert!(texel_log2 <= pixel_log2 + 1e-12 && texel_log2 > pixel_log2 - 0.01, "strip {k}: texel 2^{texel_log2}, pixel 2^{pixel_log2}");
        assert!((mag - k as f64).abs() < 1e-9, "strip {k} first appears {k} octaves into the dive, not {mag}");
    }
    // Oversampling widens the strip by its factor.
    assert_eq!(ExpMap::for_frames(1920.0, 1080.0, 2.0, 0.0).width, 13_842);
}

#[test]
fn neighbouring_strips_overlap_by_the_shared_rows_and_no_more() {
    for width in [1000u32, 6921, 13_842, 12_288] {
        let m = ExpMap { width, rows: expmap_rows(width), r0_log2: 1.0 };
        let octave_rows = m.rows_per_octave();
        // Strip k's last row reaches past strip k+1's first: the octave is covered with no gap.
        assert!(m.rows as f64 > octave_rows + (EXPMAP_OVERLAP_ROWS as f64) - 1.0 + 1e-9, "width {width}");
        assert!((m.rows as f64) <= octave_rows + EXPMAP_OVERLAP_ROWS as f64 + 1.0, "width {width}: more than one spare row");
        let (inner, _) = m.texel_polar(3, 0.0, m.rows as f64 - 0.5);
        assert!(inner < m.outer_log2(4), "width {width}: a gap between strips 3 and 4");
    }
}

#[test]
fn a_dive_has_strips_down_to_half_a_pixel_of_its_last_frame() {
    let (w, h) = (1920.0, 1080.0);
    let m = ExpMap::for_frames(w, h, 1.0, 0.0);
    // 100 octaves of dive, then log2(2202.9) = 11.1 more for the last frame's centre.
    let n = m.strips_to(100.0, h);
    assert_eq!(n, 112);
    let half_pixel_log2 = (Viewport::REFERENCE_HEIGHT / h).log2() - 100.0 - 1.0;
    assert!(m.outer_log2(n) <= half_pixel_log2, "the last strip stops short of the centre");
    assert!(m.outer_log2(n - 1) > half_pixel_log2, "a strip more than needed");
}

#[test]
fn the_outer_scale_is_exact_at_any_depth() {
    let m = ExpMap::for_frames(3840.0, 2160.0, 1.0, 2.5);
    for k in [0u64, 1, 2, 1_000, 1_000_000] {
        let (mantissa, e) = m.outer_scale(k);
        assert!((1.0..2.0).contains(&mantissa), "strip {k}: mantissa {mantissa}");
        let back = mantissa.log2() + e as f64;
        assert!((back - m.outer_log2(k)).abs() < 1e-9, "strip {k}: 2^{back} for 2^{}", m.outer_log2(k));
    }
}
