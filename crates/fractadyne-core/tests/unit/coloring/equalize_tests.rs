//! The equalizing remap (coloring plan C6): the curves' inverses, the held quantile function, its
//! CDF, the table over a fitted window, and the property it exists for: each color takes about the
//! same share of the picture.

use super::*;

/// A deterministic heavy-tailed sample like a deep view's escapes: most pixels just above the
/// floor, a tail far above it (a SplitMix-style hash, so the test needs no crate).
fn sample(n: usize, floor: f64) -> Vec<f32> {
    (0..n as u64)
        .map(|i| {
            let mut z = i.wrapping_add(0x9E37_79B9_7F4A_7C15);
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            let u = ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64;
            // A Pareto-like tail: floor + 40 / (1 − u)^0.8.
            (floor + 40.0 / (1.0 - u * 0.999).powf(0.8)) as f32
        })
        .collect()
}

#[test]
fn every_curves_inverse_undoes_it() {
    for t in Transfer::ALL {
        for d in [0.0, 0.25, 1.0, 7.5, 1.0e3, 1.0e5] {
            let back = t.inverse(t.apply(d));
            assert!((back - d).abs() <= 1.0e-9 * d.max(1.0), "{t:?} at {d}: {back}");
        }
    }
}

#[test]
fn the_quantiles_of_sorted_values_run_from_the_lowest_to_the_highest() {
    let mut v = sample(10_000, 277.0);
    v.sort_by(f32::total_cmp);
    let q = quantiles_of_sorted(&v).unwrap();
    assert_eq!(q[0], v[0]);
    assert_eq!(q[EQ_QUANTILES], v[v.len() - 1]);
    assert!(q.windows(2).all(|w| w[0] <= w[1]), "never decreasing");
    // The median knot is the median.
    let m = q[EQ_QUANTILES / 2] as f64;
    let exact = (v[4999] as f64 + v[5000] as f64) * 0.5;
    assert!((m - exact).abs() <= 1.0e-3 * exact, "{m} against {exact}");
    assert!(quantiles_of_sorted(&[]).is_none());
}

#[test]
fn the_cdf_inverts_the_quantile_function() {
    let mut v = sample(20_000, 1000.0);
    v.sort_by(f32::total_cmp);
    let q = quantiles_of_sorted(&v).unwrap();
    for i in 0..=EQ_QUANTILES {
        let p = cdf_at(&q, q[i] as f64);
        // At a knot, the share is the knot's (the top of a run of equal knots, if any).
        assert!((p - i as f64 / EQ_QUANTILES as f64).abs() <= 1.5 / EQ_QUANTILES as f64, "knot {i}: {p}");
    }
    assert_eq!(cdf_at(&q, v[0] as f64 - 1.0), 0.0);
    assert_eq!(cdf_at(&q, v[v.len() - 1] as f64 + 1.0), 1.0);
    // Against the empirical CDF at values between the knots: within a knot's share.
    for &x in &[1010.0, 1050.0, 1100.0, 1300.0, 2000.0, 5000.0] {
        let exact = v.partition_point(|&y| (y as f64) <= x) as f64 / v.len() as f64;
        assert!((cdf_at(&q, x) - exact).abs() <= 1.0 / EQ_QUANTILES as f64, "{x}: {} against {exact}", cdf_at(&q, x));
    }
}

#[test]
fn a_run_of_equal_values_counts_whole() {
    // Half the pixels at exactly one value: below it the share is 0, at it the whole half.
    let mut v: Vec<f32> = std::iter::repeat_n(500.0, 1000).chain((0..1000).map(|i| 600.0 + i as f32)).collect();
    v.sort_by(f32::total_cmp);
    let q = quantiles_of_sorted(&v).unwrap();
    assert_eq!(cdf_at(&q, 499.9), 0.0);
    assert!((cdf_at(&q, 500.0) - 0.5).abs() <= 1.0 / EQ_QUANTILES as f64, "{}", cdf_at(&q, 500.0));
}

#[test]
fn the_table_starts_at_zero_ends_at_one_and_never_falls() {
    let mut v = sample(20_000, 277.0);
    v.sort_by(f32::total_cmp);
    let q = quantiles_of_sorted(&v).unwrap();
    let top = q[EQ_QUANTILES * 98 / 100] as f64;
    for t in Transfer::ALL {
        let k = eq_knots(&q, v[0] as f64, top, t);
        assert!(k[0].abs() < 1.0e-6, "{t:?}: R(0) = {}", k[0]);
        assert_eq!(k[EQ_KNOTS], 1.0);
        assert!(k.windows(2).all(|w| w[0] <= w[1]), "{t:?} falls");
    }
    // A window with nothing measured under it: the identity.
    let k = eq_knots(&q, 0.0, 100.0, Transfer::Linear);
    assert!((eq_remap(&k, 0.37) - 0.37).abs() < 1.0e-6);
}

#[test]
fn the_remap_goes_on_at_slope_one_past_its_ends_and_its_slope_is_its_derivative() {
    let mut v = sample(20_000, 277.0);
    v.sort_by(f32::total_cmp);
    let q = quantiles_of_sorted(&v).unwrap();
    let k = eq_knots(&q, v[0] as f64, q[EQ_QUANTILES * 98 / 100] as f64, Transfer::Log);
    assert!((eq_remap(&k, 1.5) - 1.5).abs() < 1.0e-9);
    assert!((eq_remap(&k, -0.25) - (-0.25)).abs() < 1.0e-6);
    assert_eq!((eq_slope(&k, 1.5), eq_slope(&k, -0.25)), (1.0, 1.0));
    // Inside intervals (a knot's central difference would straddle two slopes).
    for u in [0.01, 0.1, 0.33, 0.5, 0.9].map(|u: f64| u + 0.37 / EQ_KNOTS as f64) {
        let h = 1.0e-4 / EQ_KNOTS as f64;
        let fd = (eq_remap(&k, u + h) - eq_remap(&k, u - h)) / (2.0 * h);
        assert!((eq_slope(&k, u) - fd).abs() <= 1.0e-3 * fd.abs().max(1.0), "at {u}: {} against {fd}", eq_slope(&k, u));
    }
}

#[test]
fn a_palette_laid_on_the_remap_gives_each_color_the_same_area() {
    // The property histogram coloring exists for: with the whole share, the palette position of
    // the picture's pixels is spread evenly over each sweep; the fitted map alone (share 0) piles
    // them up near the floor. Measured as the largest of 16 bins' shares of fract(position) against
    // the even 1/16.
    let mut v = sample(40_000, 277.0);
    v.sort_by(f32::total_cmp);
    let q = quantiles_of_sorted(&v).unwrap();
    let (lo, top) = (v[0] as f64, q[EQ_QUANTILES * 98 / 100] as f64);
    // Whole sweeps: with 2.12 the first 0.12 of the palette is passed three times and the rest
    // twice, an unevenness (0.088 for the fullest of 16) that is the sweeps', not the remap's.
    let sweeps = 2.0;
    let worst = |share: f64, t: Transfer| {
        let k = eq_knots(&q, lo, top, t);
        let mut bins = [0usize; 16];
        let mut n = 0;
        for &x in v.iter().filter(|&&x| (x as f64) <= top) {
            let u = match t {
                Transfer::Linear => (x as f64 - lo) / (top - lo),
                t => t.apply(x as f64 - lo) / t.apply(top - lo),
            };
            let pos = equalized(sweeps * u, share, sweeps, &k);
            bins[((pos.rem_euclid(1.0)) * 16.0).floor().min(15.0) as usize] += 1;
            n += 1;
        }
        bins.iter().map(|&b| b as f64 / n as f64).fold(0.0, f64::max)
    };
    for t in [Transfer::Linear, Transfer::Sqrt, Transfer::Log] {
        let (fitted, even) = (worst(0.0, t), worst(1.0, t));
        assert!(even < 1.25 / 16.0, "{t:?}: equalized, the fullest color holds {even}");
        // The control: the fitted map alone crowds the colors near the floor, except under the log,
        // which already suits this tail (its fullest color held 0.067).
        if t != Transfer::Log {
            assert!(fitted > 1.5 / 16.0, "{t:?}: the control, the fitted map alone, holds only {fitted}");
        }
    }
}

#[test]
fn share_zero_is_the_fitted_map_exactly() {
    let mut v = sample(5_000, 50.0);
    v.sort_by(f32::total_cmp);
    let q = quantiles_of_sorted(&v).unwrap();
    let k = eq_knots(&q, v[0] as f64, q[EQ_QUANTILES] as f64, Transfer::Sqrt);
    for x in [-0.3, 0.0, 0.7, 1.9, 2.12, 5.0] {
        assert_eq!(equalized(x, 0.0, 2.12, &k), x);
    }
}
