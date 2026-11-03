use super::*;

// An orbit dip below f64's normal range (2^-1022) is stored in the extended form with its true
// exponent (`pack_sample_or`). Before 2026-11-03 such a dip went through the truncated `f64` view:
// below 2^-1074 it was stored as a plain zero, and in the subnormal range its lanes came out ∞/NaN
// (`pack_sample`'s `2^-k` overflows past 2^1023). A reference at 1e999× passing a period-14,416
// minibrot dips to 2^-1656 every period, and the export rendered every pixel stuck to it.
//
// Near `c = 0` every iterate is `c` itself (`c² + c` rounds to `c` at these magnitudes), so an
// orbit there is a run of dips of a known value.

const P: usize = 256;

/// The extended form read back as the shader's `orbit_fe` reads it: `(m_re, m_im, exponent)`.
fn ext(s: [f32; 4]) -> Option<(f32, f32, i32)> {
    (s[0] == 0.0 && s[2].abs() >= 2.0).then(|| (s[2] - 4.0, s[3], s[1] as i32))
}

fn fe(m: f64, e: i32) -> BigFloat {
    FloatExp::new(m, e).to_bf(P)
}

/// Every sample `1..len` of the Mandelbrot orbit at `c`, as the builder packed it.
fn samples(cx: &BigFloat, cy: &BigFloat) -> Vec<[f32; 4]> {
    let zero = BigFloat::from_f64(0.0, P);
    let (o, len) = reference_orbit(&zero, &zero, cx, cy, 0, 12, P);
    assert_eq!(len, 13, "an orbit at c ≈ 0 runs the whole budget");
    o[1..len as usize].to_vec()
}

#[test]
fn a_dip_below_f64s_range_keeps_its_exponent() {
    // (c, the sample every iterate must pack to: [0, k, m_re + 4, m_im])
    let cases: [((f64, i32), (f64, i32), [f32; 4]); 5] = [
        // Below f64's range entirely: was stored as a plain zero sample.
        ((1.0, -1100), (0.0, 0), [0.0, -1100.0, 5.0, 0.0]),
        ((1.5, -1100), (-1.25, -1102), [0.0, -1100.0, 5.5, -0.3125]),
        // f64's subnormal range: was stored as ∞/NaN lanes.
        ((0.0, 0), (1.0, -1050), [0.0, -1050.0, 4.0, 1.0]),
        ((-1.75, -1060), (1.0, -1080), [0.0, -1060.0, 2.25, 1.0 / 1048576.0]),
        // The depth the bug was found at.
        ((1.0, -1656), (-1.5, -1657), [0.0, -1656.0, 5.0, -0.75]),
    ];
    let mut compared = 0;
    for ((xm, xe), (ym, ye), want) in cases {
        let cx = if xm == 0.0 { BigFloat::from_f64(0.0, P) } else { fe(xm, xe) };
        let cy = if ym == 0.0 { BigFloat::from_f64(0.0, P) } else { fe(ym, ye) };
        for (n, s) in samples(&cx, &cy).into_iter().enumerate() {
            assert!(s.iter().all(|v| v.is_finite()), "c=({xm}·2^{xe}, {ym}·2^{ye}) sample {}: {s:?}", n + 1);
            assert!(ext(s).is_some(), "c=({xm}·2^{xe}, {ym}·2^{ye}) sample {}: not extended-range: {s:?}", n + 1);
            assert_eq!(s.map(f32::to_bits), want.map(f32::to_bits), "c=({xm}·2^{xe}, {ym}·2^{ye}) sample {}", n + 1);
            compared += 1;
        }
    }
    assert_eq!(compared, 5 * 12);
}

/// The MPFR walk packs the same dips as the astro-float walk, bit for bit.
#[cfg(feature = "rug")]
#[test]
fn the_mpfr_walk_packs_a_deep_dip_the_same() {
    let mut compared = 0;
    for (cx, cy) in [(fe(1.5, -1100), fe(-1.25, -1102)), (BigFloat::from_f64(0.0, P), fe(1.0, -1050)), (fe(1.0, -1656), fe(-1.5, -1657))] {
        let zero = BigFloat::from_f64(0.0, P);
        let (a, alen) = reference_orbit_in(crate::BackendChoice::Astro, &zero, &zero, &cx, &cy, 0, 12, P);
        let (r, rlen) = reference_orbit_in(crate::BackendChoice::Rug, &zero, &zero, &cx, &cy, 0, 12, P);
        assert_eq!(alen, rlen);
        for n in 1..alen as usize {
            assert!(ext(a[n]).is_some(), "sample {n} not extended-range: {:?}", a[n]);
            assert_eq!(a[n].map(f32::to_bits), r[n].map(f32::to_bits), "sample {n}");
            compared += 1;
        }
    }
    assert_eq!(compared, 3 * 12);
}

/// Above 2^-1022 every sample is `pack_sample`'s of the truncated `f64` view, bit for bit: the
/// change reaches only the dips that view cannot hold.
#[test]
fn above_f64s_normal_range_a_sample_is_unchanged() {
    let p = 192;
    let mut compared = 0;
    let cases = [
        // Normal samples, an f64-range dip below EXT_SAMPLE_THRESHOLD, and the last normal octave.
        (BigFloat::from_f64(-0.1528, p), BigFloat::from_f64(1.0397, p)),
        (BigFloat::from_f64(-1.9, p), BigFloat::from_f64(0.0, p)),
        (fe(1.0, -1000), fe(1.25, -1001)),
        (fe(1.0, -1022), BigFloat::from_f64(0.0, p)),
    ];
    for (cx, cy) in cases {
        let zero = BigFloat::from_f64(0.0, p);
        let (o, len) = crate::with_orbit_schedule(false, || reference_orbit(&zero, &zero, &cx, &cy, 0, 300, p));
        let (mut zx, mut zy) = (zero.clone(), zero.clone());
        for n in 1..len as usize {
            (zx, zy) = step_bf(&zx, &zy, &cx, &cy, 0, p);
            let want = pack_sample(to_f64(&zx), to_f64(&zy));
            assert_eq!(o[n].map(f32::to_bits), want.map(f32::to_bits), "c=({}, {}) sample {n}", to_f64(&cx), to_f64(&cy));
            compared += 1;
        }
    }
    assert!(compared > 600, "too few samples compared ({compared})");
}
