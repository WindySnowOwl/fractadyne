//! Does a reference built at a higher precision store the same df32 samples? (design §5.8: the
//! tree said both "yes" and "no".) Measured 2026-10-17: it depends on the orbit.
//!
//! - A bounded orbit (a minibrot's): yes, byte-identical at the depth's need, a fresh build's +128
//!   bits and +256.
//! - An orbit on a repelling cycle (a Misiurewicz point's): only up to the iteration where the
//!   lower precision's rounding has grown into the sample. Every 128 bits kept the misiurewicz-4-1
//!   orbit on its cycle ~310 iterations longer (escapes at 1,416 / 1,721 / 2,033 samples). Each is
//!   a valid orbit to within its precision, so the picture is right either way, but not the same
//!   bytes.

use super::*;

fn fixture(name: &str) -> (String, String, f64) {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../validation").join(name);
    let fdn = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    let get = |key: &str| {
        fdn.lines()
            .find_map(|l| l.strip_prefix(&format!("{key}=")))
            .unwrap_or_else(|| panic!("{name}: no {key}"))
            .trim()
            .to_string()
    };
    (get("center_re"), get("center_im"), -get("upp_log2").parse::<f64>().unwrap())
}

fn build(cx: &str, cy: &str, iters: u32, prec: usize) -> (Vec<[f32; 4]>, u32) {
    let cx = crate::parse_bf_prec(cx, prec).expect("cx");
    let cy = crate::parse_bf_prec(cy, prec).expect("cy");
    let zero = BigFloat::from_f64(0.0, prec);
    reference_orbit(&zero, &zero, &cx, &cy, 0, iters, prec)
}

/// The samples of `a` and `b` that differ in any bit, over their common length.
fn differing(a: &[[f32; 4]], b: &[[f32; 4]]) -> usize {
    a.iter()
        .zip(b)
        .filter(|(x, y)| x.iter().zip(y.iter()).any(|(p, q)| p.to_bits() != q.to_bits()))
        .count()
}

#[test]
fn a_bounded_orbit_is_the_same_bytes_at_any_precision_a_repelling_one_is_not() {
    // A minibrot's orbit: the same samples at the need, a fresh build's precision and above it.
    let (cx, cy, depth) = fixture("minibrot-m-key-1.8e122.fdn");
    let need = crate::precision_for_octaves(depth.ceil() as u64);
    let fresh = build(&cx, &cy, 100_000, need + 128);
    for p in [need, need + 256] {
        let other = build(&cx, &cy, 100_000, p);
        assert_eq!(other.1, fresh.1, "{p} bits: the same length");
        assert_eq!(differing(&fresh.0, &other.0), 0, "{p} bits: the same bytes");
    }

    // A Misiurewicz point's: each 128 bits keeps it on its repelling cycle longer, and the samples
    // part where the lower precision's rounding has grown into them.
    let (cx, cy, depth) = fixture("misiurewicz-4-1.fdn");
    let need = crate::precision_for_octaves(depth.ceil() as u64);
    let lens: Vec<u32> = [need, need + 128, need + 256].iter().map(|&p| build(&cx, &cy, 100_000, p).1).collect();
    assert!(lens[0] < lens[1] && lens[1] < lens[2], "more precision, a longer orbit before it escapes: {lens:?}");
    let (lo, hi) = (build(&cx, &cy, 100_000, need + 128), build(&cx, &cy, 100_000, need + 256));
    let n = lo.1.min(hi.1) as usize;
    assert!(differing(&lo.0[..n], &hi.0[..n]) > 0, "a repelling orbit's tail depends on its precision");
    assert_eq!(differing(&lo.0[..1000], &hi.0[..1000]), 0, "its head does not");
}
