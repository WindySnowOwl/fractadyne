use super::*;

/// Row 18's texels, filled by `f` from each set's inputs.
fn row_of(f: impl Fn([f32; 2], [f32; 2]) -> (f32, f32)) -> Vec<f32> {
    let mut px = vec![0f32; (GT_W * 19 * 4) as usize];
    for ix in 0..GT_W {
        let s = ix + 100003 * 18;
        for k in 0..2u32 {
            let (z, d) = gt_normal_case(s * 2 + k);
            let (x, y) = f(z, d);
            let base = ((18 * GT_W + ix) * 4 + 2 * k) as usize;
            px[base] = x;
            px[base + 1] = y;
        }
    }
    px
}

fn graded(px: &[f32]) -> OpCheck {
    let texel = |ix: u32, op: u32| -> [f32; 4] {
        let base = ((op * GT_W + ix) * 4) as usize;
        [px[base], px[base + 1], px[base + 2], px[base + 3]]
    };
    normal_row(GT_W, &texel)
}

/// The shader's `slope_normal` before 2026-10-30, in f32 as the GPU runs it.
fn unscaled(z: [f32; 2], d: [f32; 2]) -> (f32, f32) {
    let ur = z[0] * d[0] + z[1] * d[1];
    let ui = z[1] * d[0] - z[0] * d[1];
    let m = (ur * ur + ui * ui).sqrt();
    if m > 1.0e-30 { (ur / m, ui / m) } else { (0.0, 0.0) }
}

/// A row that only ever met small derivatives could not fail the overflow it is there for: the
/// sets must reach well past 2^64 and stay well under it, a good share of each.
#[test]
fn the_normal_row_spans_the_overflow() {
    let (mut over, mut under) = (0, 0);
    for t in 0..GT_W * 2 {
        let (z, d) = gt_normal_case(t + 2 * 100003 * 18);
        assert!(z.iter().chain(d.iter()).all(|v| v.is_normal()), "set {t}: {z:?} {d:?}");
        let zd = (z[0] as f64).hypot(z[1] as f64) * (d[0] as f64).hypot(d[1] as f64);
        if zd >= 2f64.powi(66) {
            over += 1;
        } else if zd < 2f64.powi(60) {
            under += 1;
        }
    }
    assert!(over >= 150 && under >= 150, "{over} sets over 2^66, {under} under 2^60, of {}", GT_W * 2);
}

/// The grading passes the exact answer and fails the unscaled formula, on the overflowing sets
/// only: under 2^64 its normals are the f64 ones within the tolerance.
#[test]
fn the_normal_row_fails_the_unscaled_formula() {
    let exact = graded(&row_of(|z, d| {
        let (x, y) = normal_oracle(z, d);
        (x as f32, y as f32)
    }));
    assert_eq!(exact.fails, 0, "{}", exact.detail);
    let old = graded(&row_of(unscaled));
    let overflowing = (0..GT_W * 2)
        .filter(|&t| {
            let (z, d) = gt_normal_case(t + 2 * 100003 * 18);
            let (x, y) = unscaled(z, d);
            x == 0.0 && y == 0.0 || x.is_nan() || y.is_nan()
        })
        .count() as u32;
    assert!(overflowing > 100, "only {overflowing} sets overflow");
    assert_eq!(old.fails, overflowing, "{}", old.detail);
    assert!(old.detail.starts_with(&format!("{overflowing} of {} sets: first ix=", GT_W * 2)), "{}", old.detail);
}
