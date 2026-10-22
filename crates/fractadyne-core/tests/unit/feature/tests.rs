use super::*;
use crate::{formula, parse_bf_prec, reference_orbit, to_f64, BigFloat};

const NEVER: &dyn Fn() -> bool = &|| false;

fn c(re: f64, im: f64) -> CFloatExp {
    CFloatExp { re: FloatExp::from_f64(re), im: FloatExp::from_f64(im) }
}

fn at(n: &NucleusNear) -> (f64, f64) {
    (n.dc.re.to_f64(), n.dc.im.to_f64())
}

fn reference(re: &str, im: &str, max_iter: u32) -> (Vec<[f32; 4]>, u32) {
    let p = 192;
    let zero = BigFloat::from_f64(0.0, p);
    let cx = parse_bf_prec(re, p).unwrap();
    let cy = parse_bf_prec(im, p).unwrap();
    reference_orbit(&zero, &zero, &cx, &cy, formula::MANDELBROT, max_iter, p)
}

// Nuclei of z² + c, to f64.
const P3_REAL: (f64, f64) = (-1.754_877_666_246_692_7, 0.0);
const P3_BULB: (f64, f64) = (-0.122_561_166_876_653_62, 0.744_861_766_619_744_2);

#[test]
fn finds_shallow_nuclei_without_a_reference() {
    // No reference: `dc` is `c` itself (the orbit of C = 0 is all zeros).
    let n = find_nucleus_near(&[], 0, c(-1.76, 0.005), FloatExp::from_f64(0.03), 1000, NEVER).expect("period 3");
    assert_eq!(n.period, 3);
    let (x, y) = at(&n);
    assert!((x - P3_REAL.0).abs() < 1e-12 && y.abs() < 1e-12, "{x} {y}");

    let n = find_nucleus_near(&[], 0, c(-0.1, 0.72), FloatExp::from_f64(0.06), 1000, NEVER).expect("the bulb");
    assert_eq!(n.period, 3);
    let (x, y) = at(&n);
    assert!((x - P3_BULB.0).abs() < 1e-12 && (y - P3_BULB.1).abs() < 1e-12, "{x} {y}");

    let n = find_nucleus_near(&[], 0, c(0.05, 0.02), FloatExp::from_f64(0.1), 1000, NEVER).expect("the cardioid");
    assert_eq!(n.period, 1);
    assert!(at(&n).0.abs() < 1e-12 && at(&n).1.abs() < 1e-12);

    let n = find_nucleus_near(&[], 0, c(-1.02, 0.01), FloatExp::from_f64(0.05), 1000, NEVER).expect("the period-2 disc");
    assert_eq!(n.period, 2);
    assert!((at(&n).0 + 1.0).abs() < 1e-12);
}

#[test]
fn nothing_is_found_where_there_is_nothing_near() {
    // Outside the set, far from everything: the orbit escapes before any period.
    assert!(find_nucleus_near(&[], 0, c(0.5, 0.5), FloatExp::from_f64(1e-3), 1000, NEVER).is_none());
    // A nucleus outside the disc is not kept: Newton from beside the period-2 disc lands on -1,
    // 0.04 away, beyond a 0.01 radius.
    assert!(find_nucleus_near(&[], 0, c(-1.04, 0.0), FloatExp::from_f64(0.01), 1000, NEVER)
        .is_none_or(|n| (at(&n).0 + 1.04).abs() <= 0.01));
    // No radius, no periods: nothing to search.
    assert!(find_nucleus_near(&[], 0, c(-1.0, 0.0), FloatExp::ZERO, 1000, NEVER).is_none());
    assert!(find_nucleus_near(&[], 0, c(-1.0, 0.0), FloatExp::from_f64(0.1), 0, NEVER).is_none());
}

#[test]
fn finds_a_nucleus_relative_to_a_reference() {
    // A reference inside the period-3 minibrot (it never escapes): dc is relative to C.
    let (orbit, len) = reference("-1.755", "0", 2000);
    assert!(len >= 1990, "a reference that never escapes: {len}");
    let n = find_nucleus_near(&orbit, len, c(-1.76 + 1.755, 0.005), FloatExp::from_f64(0.03), 2000, NEVER)
        .expect("period 3");
    assert_eq!(n.period, 3);
    assert!((at(&n).0 - (P3_REAL.0 + 1.755)).abs() < 1e-12, "{:?}", at(&n));
}

#[test]
fn rebasing_carries_a_search_past_an_escaped_reference() {
    // C = 0.3 escapes after a few dozen iterations; the searches near 0 and -1 run long past its
    // end on the rebased orbit.
    let (orbit, len) = reference("0.3", "0", 2000);
    assert!(len < 100, "the reference escaped early: {len}");
    let n = find_nucleus_near(&orbit, len, c(-0.3, 0.02), FloatExp::from_f64(0.1), 2000, NEVER).expect("the cardioid");
    assert_eq!(n.period, 1);
    assert!((at(&n).0 + 0.3).abs() < 1e-12 && at(&n).1.abs() < 1e-12, "{:?}", at(&n));
    let n = find_nucleus_near(&orbit, len, c(-1.3, 0.01), FloatExp::from_f64(0.05), 2000, NEVER).expect("period 2");
    assert_eq!(n.period, 2);
    assert!((at(&n).0 + 1.3).abs() < 1e-12, "{:?}", at(&n));
}

#[test]
fn agrees_with_the_bignum_finder_at_depth() {
    // Seahorse valley at 2^34, on the period-8007 nucleus the bignum finder names (its atom
    // domain holds the centre). Lower periods crowd it — 998 at 3.6e-16, 7962 at 1.3e-17 (each
    // a true nucleus; see the next test) — so a disc of 1e-24, which only it is inside, finds the
    // same 8007 as the bignum finder, at the same place.
    let p = 256;
    let rm = crate::RM;
    let cx = parse_bf_prec("-0.743643887037158704752191506114774", p).unwrap();
    let cy = parse_bf_prec("0.131825904205311970493132056385139", p).unwrap();
    let nb = crate::find_nucleus(&[cx.clone(), cy.clone()], 34.0, formula::MANDELBROT, 100_000).expect("a nucleus in view");
    assert_eq!(nb.period, 8007);
    let zero = BigFloat::from_f64(0.0, p);
    let (orbit, len) = reference_orbit(&zero, &zero, &cx, &cy, formula::MANDELBROT, 200_000, p);
    let nx = to_f64(&nb.cx.sub(&cx, p, rm));
    let ny = to_f64(&nb.cy.sub(&cy, p, rm));
    let radius = 1.0e-24;
    let n = find_nucleus_near(&orbit, len, c(nx + radius / 3.0, ny - radius / 3.0), FloatExp::from_f64(radius), 200_000, NEVER)
        .expect("the period-8007 nucleus");
    assert_eq!(n.period, nb.period);
    let (x, y) = at(&n);
    assert!((x - nx).abs() < radius * 1e-6 && (y - ny).abs() < radius * 1e-6, "{x:e} {y:e} vs {nx:e} {ny:e}");
}

/// How far `C + dc` is from the root of `z_period(c) = 0`, to first order (`|z / z'|`, a Newton
/// step), iterating in bignum: the independent check that a found point is a nucleus.
fn bignum_distance(cx: &BigFloat, cy: &BigFloat, dc: CFloatExp, period: u32, p: usize) -> f64 {
    let rm = crate::RM;
    let ncx = cx.add(&dc.re.to_bf(p), p, rm);
    let ncy = cy.add(&dc.im.to_bf(p), p, rm);
    let zero = BigFloat::from_f64(0.0, p);
    let one = BigFloat::from_f64(1.0, p);
    let two = BigFloat::from_f64(2.0, p);
    let (mut zx, mut zy) = (zero.clone(), zero.clone());
    let (mut dx, mut dy) = (zero.clone(), zero);
    for _ in 0..period {
        // z' ← 2 z z' + 1, then z ← z² + c
        let ndx = two.mul(&zx.mul(&dx, p, rm).sub(&zy.mul(&dy, p, rm), p, rm), p, rm).add(&one, p, rm);
        let ndy = two.mul(&zx.mul(&dy, p, rm).add(&zy.mul(&dx, p, rm), p, rm), p, rm);
        let x2 = zx.mul(&zx, p, rm);
        let y2 = zy.mul(&zy, p, rm);
        let xy = zx.mul(&zy, p, rm);
        zy = xy.add(&xy, p, rm).add(&ncy, p, rm);
        zx = x2.sub(&y2, p, rm).add(&ncx, p, rm);
        (dx, dy) = (ndx, ndy);
    }
    let z = to_f64(&zx).hypot(to_f64(&zy));
    let d = to_f64(&dx).hypot(to_f64(&dy));
    z / d
}

#[test]
fn the_lowest_period_in_the_disc_is_a_true_nucleus() {
    // The snap's disc at that view (1/24 of its height, 9.7e-12) holds the 998 and the 8007; the
    // search names the LOWER period, the bigger minibrot of the two. And a 1e-17 disc beside the
    // 8007 holds a 7962. Each is a nucleus: iterated in bignum from the found c, a Newton step to
    // the root of z_p is a millionth of its disc or less.
    let p = 256;
    let cx = parse_bf_prec("-0.743643887037158704752191506114774", p).unwrap();
    let cy = parse_bf_prec("0.131825904205311970493132056385139", p).unwrap();
    let zero = BigFloat::from_f64(0.0, p);
    let (orbit, len) = reference_orbit(&zero, &zero, &cx, &cy, formula::MANDELBROT, 200_000, p);
    let radius = 4.0 * 2f64.powf(-34.0) / 24.0;
    let n = find_nucleus_near(&orbit, len, c(0.0, 0.0), FloatExp::from_f64(radius), 200_000, NEVER).expect("a nucleus");
    assert_eq!(n.period, 998);
    let r = bignum_distance(&cx, &cy, n.dc, n.period, p);
    assert!(r < radius * 1e-6, "998: {r:e} from the root");
    let r17 = 1.0e-17;
    let n = find_nucleus_near(&orbit, len, c(r17 / 3.0, -r17 / 3.0), FloatExp::from_f64(r17), 200_000, NEVER).expect("a nucleus");
    assert_eq!(n.period, 7962);
    let r = bignum_distance(&cx, &cy, n.dc, n.period, p);
    assert!(r < r17 * 1e-6, "7962: {r:e} from the root");
    // The control: the point the search started from is no nucleus of that period by this test.
    let r0 = bignum_distance(&cx, &cy, c(r17 / 3.0, -r17 / 3.0), n.period, p);
    assert!(r0 > r17 * 0.1, "the start point read {r0:e} from a root");
}

#[test]
fn a_cancelled_search_finds_nothing() {
    // Long enough to reach the first cancellation check: a point deep in the cardioid never
    // escapes, and a radius too small for it to see its own nucleus.
    let cancel: &dyn Fn() -> bool = &|| true;
    assert!(find_nucleus_near(&[], 0, c(0.1, 0.1), FloatExp::from_f64(1e-30), 100_000, cancel).is_none());
}
