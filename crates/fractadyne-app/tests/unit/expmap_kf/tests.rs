//! The KF strip format's conversions, against KF's conventions as measured from its own output
//! (`expmap_kf.rs`'s note): computed here straight from those definitions on real orbits, never
//! through the conversion under test.

use super::*;
use fractadyne_core::StripRows;

/// `z ← z^p + c` from 0 in f64 until `|z| > radius`: (the escape count, the escaped z).
fn escape(c: (f64, f64), p: u32, radius: f64, max: u32) -> Option<(u32, (f64, f64))> {
    let (mut zr, mut zi) = (0.0f64, 0.0f64);
    for n in 1..=max {
        let (mut ar, mut ai) = (1.0, 0.0);
        for _ in 0..p {
            (ar, ai) = (ar * zr - ai * zi, ar * zi + ai * zr);
        }
        (zr, zi) = (ar + c.0, ai + c.1);
        if zr.hypot(zi) > radius {
            return Some((n, (zr, zi)));
        }
    }
    None
}

#[test]
fn a_count_fraction_and_phase_come_out_as_kalles_fraktaler_writes_them() {
    let mut seed = 0x2545_f491_4f6c_dd1du64;
    let mut rnd = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed >> 11) as f64 / (1u64 << 53) as f64
    };
    for p in 2u32..=8 {
        let pf = p as f64;
        // KF's escape radius as measured (100 at power 2, 21.54 at 3, 3.162 at 8), written out here,
        // not taken from the code under test.
        let er = 10f64.powf(4.0 / pf);
        let (mut same, mut stepped_back, mut checked) = (0, 0, 0);
        for _ in 0..4000 {
            let c = (rnd() * 3.0 - 1.8, rnd() * 3.0 - 1.5);
            let (Some((n, z)), Some((n_kf, z_kf))) = (escape(c, p, 256.0, 5000), escape(c, p, er, 5000)) else {
                continue;
            };
            // The renderer's channels, as the shader forms them, in f32.
            let nf = (1.0 - z.0.hypot(z.1).log2().ln() / pf.ln()) as f32;
            let t = (z.1.atan2(z.0) / TAU + 0.5) as f32;
            // KF's, straight from its definitions.
            let want_nf = 1.0 - (z_kf.0.hypot(z_kf.1).ln() / er.ln()).ln() / pf.ln();
            let want_t = (pf * z_kf.1.atan2(z_kf.0) / TAU).rem_euclid(1.0);
            let (whole, frac, phase) = kf_count(n, nf, t, pf, c);
            checked += 1;
            if n_kf == n { same += 1 } else { stepped_back += 1 }
            // The continuous count, to f32's rounding (exactly: carrying the smooth count across
            // escape iterates instead is off by up to 2.4e-5, the `+ c`); the split into whole and
            // fraction too, except within f32's reach of a whole number, where either side of the
            // band edge is the same count.
            let dc = (whole as f64 + frac as f64) - (n_kf as f64 + want_nf);
            assert!(dc.abs() < 2e-6, "power {p}, c {c:?}: count {whole}+{frac} for {n_kf}+{want_nf}");
            if want_nf > 1e-5 && want_nf < 1.0 - 1e-5 {
                assert_eq!(whole, n_kf as u64, "power {p}, c {c:?}: escape count");
            }
            assert!((0.0..1.0).contains(&frac) && (0.0..1.0).contains(&phase));
            let dt = (phase as f64 - want_t + 0.5).rem_euclid(1.0) - 0.5;
            assert!(dt.abs() < 1e-6 * pf, "power {p}, c {c:?}: phase {phase} for {want_t} (n {n}, KF's n {n_kf})");
        }
        // Both ways the renderer's escape relates to KF's were exercised, many times each.
        assert!(checked > 1000 && same > 100 && stepped_back > 100, "power {p}: {checked} checked, {same} at the same iterate, {stepped_back} one after");
    }
}

#[test]
fn the_distance_estimate_is_the_planes_vector_with_imaginary_up_over_kfs_strip_pixel() {
    let (w, h) = (2304u32, 256u32);
    let row_step = std::f64::consts::LN_2 / h as f64;
    // 8 radial texels (r·row_step each) along (0.6, 0.8), imaginary up.
    let (dx, dy) = kf_de(3.0, 0.6, 0.8, row_step, w, h);
    // KF's pixel in a strip is r·2/hypot(W, H): 8·row_step/(2/hypot) of them.
    let mag = 8.0 * row_step * (w as f64).hypot(h as f64) / 2.0;
    assert!((dx as f64 - 0.6 * mag).abs() < 1e-4 * mag, "{dx}");
    // KF's ImagPointsUp: 1 keeps the plane's direction (its default, 0, negates DEY).
    assert!((dy as f64 - 0.8 * mag).abs() < 1e-4 * mag, "{dy}: imaginary points up");
    // No distance estimate (the renderer's sentinel, or not a number): KF's "missing", (0, 0).
    assert_eq!(kf_de(1.0e30, 0.6, 0.8, row_step, w, h), (0.0, 0.0));
    assert_eq!(kf_de(f32::NAN, 0.6, 0.8, row_step, w, h), (0.0, 0.0));
}

#[test]
fn interior_and_glitched_texels_are_marked_and_a_wide_count_splits_into_n0_and_n1() {
    let map = ExpMap { width: 3, rows: 1, r0_log2: 0.0, layout: StripRows::Octave };
    let strip = KfStrip { map: &map, k: 0, power: 2.0, centre: (-0.75, 0.1) };
    let iter: u32 = 123_456_789;
    // Escaped (|z| = 2^10 > 256 and > 100: KF read the same z, or the one before), capped, glitched.
    let nf = (1.0 - 10f64.log2()) as f32;
    let main = [
        iter as f32, 0.6, 0.8, 2.5, //
        -1.0, 0.0, 0.0, 1.0e30, //
        -2.0, 0.0, 0.0, 1.0e30,
    ];
    let aux = [
        (iter >> 12) as f32, (iter & 4095) as f32, nf, 0.75, //
        0.0, 0.0, 1.0e30, 0.0, //
        0.0, 0.0, 1.0e30, 0.0,
    ];
    let ch = kf_channels(&main, &aux, &strip);
    let names: Vec<_> = ch.iter().map(|c| c.name).collect();
    assert_eq!(names, ["N", "NF", "T", "DEX", "DEY"]);
    let u = |name: &str| match &ch.iter().find(|c| c.name == name).unwrap().data {
        ExrChannelData::U32(v) => v.clone(),
        _ => panic!("{name} is not uint32"),
    };
    let f = |name: &str| match &ch.iter().find(|c| c.name == name).unwrap().data {
        ExrChannelData::F32(v) => v.clone(),
        _ => panic!("{name} is not f32"),
    };
    let (whole, _, _) = kf_count(iter, nf, 0.75, 2.0, (0.0, 0.0));
    // Rendered columns 0, 1, 2 (anticlockwise) are written at 0, 2, 1 (clockwise): the capped
    // texel lands in the last column and the glitched one in the middle.
    assert_eq!(u("N"), vec![whole as u32 + KF_ITERATIONS_BIAS as u32, 0, u32::MAX]);
    for name in ["NF", "T", "DEX", "DEY"] {
        assert_eq!(&f(name)[1..], &[0.0, 0.0], "{name}: interior and glitched texels are 0, as KF writes them");
    }
    assert!(f("DEX")[0] > 0.0 && f("DEY")[0] > 0.0);

    // A count past 0xFFFFFFFE once biased: KF's N0 (low word) and N1 (high word).
    let big: u32 = u32::MAX - 10;
    let aux_big = [(big >> 12) as f32, (big & 4095) as f32, nf, 0.75, 0.0, 0.0, 1.0e30, 0.0, 0.0, 0.0, 1.0e30, 0.0];
    let ch = kf_channels(&main, &aux_big, &strip);
    let names: Vec<_> = ch.iter().map(|c| c.name).collect();
    assert_eq!(names, ["N0", "N1", "NF", "T", "DEX", "DEY"]);
    let get = |name: &str| match &ch.iter().find(|c| c.name == name).unwrap().data {
        ExrChannelData::U32(v) => v.clone(),
        _ => panic!(),
    };
    let (whole, _, _) = kf_count(big, nf, 0.75, 2.0, (0.0, 0.0));
    let biased = whole + KF_ITERATIONS_BIAS as u64;
    assert_eq!(get("N0"), vec![biased as u32, 0, u32::MAX]);
    assert_eq!(get("N1"), vec![(biased >> 32) as u32, 0, u32::MAX]);
}

/// zoomasm shows a KF strip's column `x` at angle `2πx/W` ANTICLOCKWISE on screen with the
/// imaginary axis down (KF's default), so for the dive to come out the right way up, as Fractadyne
/// shows it, column `x` must hold the plane's point at `−2πx/W`: CLOCKWISE (measured 2026-10-28 by
/// assembling strips in zoomasm 3.4 against Fractadyne's flat renders, and against KF's own
/// `ImagPointsUp: 1` strips). Each rendered texel carries its own column and row in its count, and
/// the angle it is written at is worked out here from the geometry, not from `kf_column`.
#[test]
fn each_texel_is_written_where_the_plane_comes_out_upright_in_zoomasm() {
    let (w, h) = (8usize, 2usize);
    let map = ExpMap { width: w as u32, rows: h as u32, r0_log2: 0.0, layout: StripRows::Octave };
    let strip = KfStrip { map: &map, k: 0, power: 2.0, centre: (-0.75, 0.1) };
    let tag = |i: usize, j: usize| 1000 + 100 * j as u32 + i as u32;
    let nf = (1.0 - 10f64.log2()) as f32; // |z| = 2^10: KF read the same z
    let (mut main, mut aux) = (Vec::new(), Vec::new());
    for j in 0..h {
        for i in 0..w {
            let it = tag(i, j);
            main.extend([it as f32, 0.6, 0.8, 2.5]);
            aux.extend([(it >> 12) as f32, (it & 4095) as f32, nf, 0.75]);
        }
    }
    let ch = kf_channels(&main, &aux, &strip);
    let ExrChannelData::U32(n) = &ch.iter().find(|c| c.name == "N").unwrap().data else { panic!("N is not uint32") };
    let step = TAU / w as f64;
    for j in 0..h {
        for x in 0..w {
            // The rendered column at the plane angle −2πx/W (renderer: column i at +2πi/W).
            let i = ((-(x as f64) * step).rem_euclid(TAU) / step).round() as usize % w;
            let (whole, _, _) = kf_count(tag(i, j), nf, 0.75, 2.0, (0.0, 0.0));
            assert_eq!(n[j * w + x], whole as u32 + KF_ITERATIONS_BIAS as u32, "column {x}, row {j}: wants the rendered texel ({i}, {j})");
        }
    }
    // Column 2 of 8 is a quarter turn clockwise: BELOW the centre in the plane, rendered at 270°.
    assert_eq!(n[2] as u64, kf_count(tag(6, 0), nf, 0.75, 2.0, (0.0, 0.0)).0 + KF_ITERATIONS_BIAS as u64);
}
