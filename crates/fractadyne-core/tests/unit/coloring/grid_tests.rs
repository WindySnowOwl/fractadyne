use super::*;

/// A line's coverage is the box-filtered line: full on a line wider than the tap, nothing past half
/// the width and half the tap, and over any one line it adds up to the line's width, whatever the
/// tap (so a supersampled pixel's texels draw the line as wide as one texel a pixel does).
#[test]
fn a_lines_coverage_adds_up_to_its_width_at_every_tap() {
    assert_eq!(grid_cover(0.0, 1.5, 1.0), 1.0);
    assert_eq!(grid_cover(0.0, 1.5, 0.25), 1.0);
    assert_eq!(grid_cover(1.25, 1.5, 1.0), 0.0);
    assert_eq!(grid_cover(-1.25, 1.5, 1.0), 0.0);
    assert!((grid_cover(0.0, 0.5, 1.0) - 0.5).abs() < 1e-12, "a line narrower than the tap covers its share of it");
    for width in [0.5, 1.0, 1.5, 3.0, 8.0] {
        for tap in [0.125, 0.25, 0.5, 1.0, 2.0, 4.0] {
            let n = 40_000;
            let span = width + tap + 2.0;
            let dx = 2.0 * span / n as f64;
            let sum: f64 = (0..n).map(|i| grid_cover(-span + (i as f64 + 0.5) * dx, width, tap) * dx).sum();
            assert!((sum - width).abs() < 1e-3, "width {width}, tap {tap}: the line adds up to {sum}");
        }
    }
}

/// The level of detail: over a stretch of the field where a count is `s` output pixels across, the
/// lines cover between half and all of width / spacing of it (they are spacing to twice that apart),
/// at every s from a tenth of the spacing to ten thousand times it. A FIXED level (lines at whole
/// counts) does not: its coverage follows s (the control).
#[test]
fn the_lines_keep_their_spacing_at_every_scale() {
    let (spacing, width) = (16.0, 1.5);
    let ln2 = std::f64::consts::LN_2;
    let mut fixed = Vec::new();
    for k in -3..=13 {
        let s = spacing * 2f64.powf(k as f64 + 0.37);
        let d = s / ln2;
        // A run of the field `n` counts long from a count of 3 (low enough that μ's f32 step is far
        // under a pixel even at 85,000 px a count), sampled every tenth of a pixel.
        let n = (4000.0 / s).max(1.0);
        let samples = ((n * s) * 10.0) as usize;
        let at = |i: usize| 3.0 + i as f64 / (10.0 * s);
        let mean: f64 = (0..samples).map(|i| grid_weight(at(i), d, ln2, spacing, width, 1.0, 1.0)).sum::<f64>() / samples as f64;
        let share = mean / (width / spacing);
        assert!((0.45..=1.1).contains(&share), "s {s:.2} px a count: the lines cover {share:.3} of width / spacing");
        let whole: f64 = (0..samples)
            .map(|i| {
                let f = ((at(i) + 0.5).rem_euclid(1.0) - 0.5).abs();
                grid_cover(f * s, width, 1.0)
            })
            .sum::<f64>()
            / samples as f64;
        fixed.push(whole / (width / spacing));
    }
    let (lo, hi) = fixed.iter().fold((f64::MAX, 0.0f64), |(a, b), &x| (a.min(x), b.max(x)));
    assert!(lo < 0.1 && hi > 5.0, "the control: lines at whole counts cover {lo:.3} to {hi:.3} of width / spacing");
}

/// The fades: none within half a pixel of the set, none where μ's f32 step is a pixel or more, none
/// at no strength or without a distance's degree; the strength scales the line.
#[test]
fn the_lines_fade_near_the_set_and_where_the_count_is_coarse() {
    let ln2 = std::f64::consts::LN_2;
    // On a line (μ a whole count), 40 px from the set.
    assert_eq!(grid_weight(100.0, 40.0, ln2, 16.0, 1.5, 1.0, 1.0), 1.0);
    assert!((grid_weight(100.0, 40.0, ln2, 16.0, 1.5, 0.6, 1.0) - 0.6).abs() < 1e-12);
    assert_eq!(grid_weight(100.0, 0.4, ln2, 16.0, 1.5, 1.0, 1.0), 0.0, "within half a pixel of the set");
    assert_eq!(grid_weight(100.0, 40.0, 0.0, 16.0, 1.5, 1.0, 1.0), 0.0, "no distance estimate's degree");
    assert_eq!(grid_weight(100.0, 40.0, ln2, 16.0, 1.5, 0.0, 1.0), 0.0, "no strength");
    // A million counts, 40 px from the set: μ's step is 1/8 of a count, 3.5 px.
    assert_eq!(grid_weight(1048576.0, 40.0, ln2, 16.0, 1.5, 1.0, 1.0), 0.0, "a coarse count");
    // A thousand counts there: 0.003 px.
    assert_eq!(grid_weight(1024.0, 40.0, ln2, 16.0, 1.5, 1.0, 1.0), 1.0, "a fine count");
}
