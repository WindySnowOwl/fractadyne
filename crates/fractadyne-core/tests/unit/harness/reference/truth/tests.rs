use super::*;

/// A deterministic generator (no dev-dependency).
fn lcg(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((*state >> 11) as f64) / ((1u64 << 53) as f64)
}

const P: usize = 320;

fn bfv(v: f64) -> BigFloat {
    BigFloat::from_f64(v, P)
}

/// `(x + iy)·2^k` with `x, y` arbitrary-precision: the exact value a [`Cdx`] should hold.
fn exact(x: &BigFloat, y: &BigFloat, k: i64) -> (BigFloat, BigFloat) {
    let s = FloatExp::from_f64(1.0).mul_pow2(k as f64).to_bf(P);
    (x.mul(&s, P, RM), y.mul(&s, P, RM))
}

/// log2 of the relative error of `got` against `want` (both complex), the larger part's scale.
fn rel_err_log2(got: &Cdx, want: (&BigFloat, &BigFloat)) -> f64 {
    let g = Cdx::from_bf(want.0, want.1);
    let d = got.add(Cdx { re: g.re.neg(), im: g.im.neg(), e: g.e });
    if d.is_zero() {
        return f64::NEG_INFINITY;
    }
    let mag = |c: &Cdx| c.e as f64 + 0.5 * (c.re.hi * c.re.hi + c.im.hi * c.im.hi).log2();
    mag(&d) - mag(&g)
}

#[test]
fn double_double_complex_arithmetic_holds_100_bits() {
    let mut s = 7u64;
    let mut worst = f64::NEG_INFINITY;
    let mut n = 0;
    for _ in 0..2000 {
        // Operands with random 100-bit-plus mantissas and exponents spread far past f64's range.
        let mk = |s: &mut u64| {
            let x = bfv(lcg(s) * 2.0 - 1.0).add(&bfv((lcg(s) * 2.0 - 1.0) * 1e-17), P, RM);
            let y = bfv(lcg(s) * 2.0 - 1.0).add(&bfv((lcg(s) * 2.0 - 1.0) * 1e-17), P, RM);
            let k = ((lcg(s) - 0.5) * 4000.0) as i64;
            (x, y, k)
        };
        let (ax, ay, ak) = mk(&mut s);
        let (bx, by, bk) = mk(&mut s);
        let (ax, ay) = exact(&ax, &ay, ak);
        let (bx, by) = exact(&bx, &by, bk);
        let (a, b) = (Cdx::from_bf(&ax, &ay), Cdx::from_bf(&bx, &by));
        // a·b
        let px = ax.mul(&bx, P, RM).sub(&ay.mul(&by, P, RM), P, RM);
        let py = ax.mul(&by, P, RM).add(&ay.mul(&bx, P, RM), P, RM);
        worst = worst.max(rel_err_log2(&a.mul(b), (&px, &py)));
        // a²
        let qx = ax.mul(&ax, P, RM).sub(&ay.mul(&ay, P, RM), P, RM);
        let qy = ax.mul(&ay, P, RM).mul(&bfv(2.0), P, RM);
        worst = worst.max(rel_err_log2(&a.sqr(), (&qx, &qy)));
        // a + b, at exponents close enough to interact (a scaled to b's magnitude)
        let (cx, cy) = exact(&ax, &ay, bk - ak);
        let c = Cdx::from_bf(&cx, &cy);
        let sx = cx.add(&bx, P, RM);
        let sy = cy.add(&by, P, RM);
        if !(sx.is_zero() && sy.is_zero()) {
            let e = rel_err_log2(&c.add(b), (&sx, &sy));
            // A sum that cancels is held to the operands' scale, not its own.
            let opnd = c.e.max(b.e) as f64;
            let sum = Cdx::from_bf(&sx, &sy);
            worst = worst.max(e + (sum.e as f64 - opnd).min(0.0));
        }
        n += 3;
    }
    assert!(n >= 6000);
    assert!(worst < -100.0, "worst relative error 2^{worst:.1}");
}

#[test]
fn a_dip_far_below_f64_keeps_its_bits() {
    // 1.2345...·2^-1656, the depth of the 2026-11-03 reference's first dip.
    let m = bfv(1.234_567_890_123_456_7).add(&bfv(1.0e-18), P, RM);
    let (x, y) = exact(&m, &bfv(-0.75), -1656);
    let c = Cdx::from_bf(&x, &y);
    assert_eq!(c.e, -1656);
    assert!(rel_err_log2(&c, (&x, &y)) < -100.0);
    // and survives arithmetic at that depth: c·c is at 2^-3312
    let qx = x.mul(&x, P, RM).sub(&y.mul(&y, P, RM), P, RM);
    let qy = x.mul(&y, P, RM).mul(&bfv(2.0), P, RM);
    assert!(rel_err_log2(&c.sqr(), (&qx, &qy)) < -100.0);
}

// The period-128 minibrot of bench ladder scene 40 (its nucleus), as in nucleus_pick's tests.
const NRE: &str = "-7.145519235131990488163335631423597550599502471617637408317e-1";
const NIM: &str = "3.540207347648327221719984333721934753758406620526068439482e-1";

#[test]
fn full_step_perturbation_matches_plain_iteration_at_every_pixel() {
    // The minibrot a tenth of the view (1e11.96x, 16:9), its nucleus 0.6 half-widths right of the
    // center, at the set's cap of 30 periods: a grid over the whole view, every pixel compared.
    let p = 192;
    let h = FloatExp::from_f64(4.0).mul_pow2(-11.962 * std::f64::consts::LOG2_10);
    let w = h.mul_f64(16.0 / 9.0);
    let n = [parse_bf_prec(NRE, p).unwrap(), parse_bf_prec(NIM, p).unwrap()];
    let cx = n[0].sub(&w.mul_f64(0.3).to_bf(p), p, RM);
    let cy = n[1].clone();
    let max_iter = 3840;
    let r = truth_reference(&cx, &cy, max_iter, p);
    let (gw, gh) = (40usize, 24usize);
    let (mut compared, mut escaped, mut interior, mut untrusted) = (0, 0, 0, 0);
    for gy in 0..gh {
        for gx in 0..gw {
            let px = cx.add(&w.mul_f64((gx as f64 + 0.5) / gw as f64 - 0.5).to_bf(p), p, RM);
            let py = cy.add(&h.mul_f64(0.5 - (gy as f64 + 0.5) / gh as f64).to_bf(p), p, RM);
            let dc = Cdx::from_bf(&px.sub(&r.cx, p, RM), &py.sub(&r.cy, p, RM));
            let want = truth_naive(&px, &py, max_iter, p);
            match truth_pixel(&r, dc, max_iter) {
                Some(got) => {
                    assert_eq!(
                        (got.n2, got.n256, got.n4096),
                        (want.n2, want.n256, want.n4096),
                        "pixel ({gx},{gy})"
                    );
                    if want.n2 >= 0 {
                        assert!((got.smooth256 - want.smooth256).abs() < 1e-6, "pixel ({gx},{gy}) smooth");
                        escaped += 1;
                    } else {
                        interior += 1;
                    }
                }
                None => untrusted += 1,
            }
            compared += 1;
        }
    }
    assert_eq!(compared, gw * gh);
    assert!(escaped > 600, "escaped {escaped}, interior {interior}");
    assert!(untrusted * 20 < compared, "{untrusted} of {compared} pixels handed to plain iteration");
    // The grid misses the minibrot's interior (a tenth of the view); its nucleus is interior. Its
    // orbit returns to 0 each period, so the perturbation loses everything to cancellation there
    // and says so; the plain iteration answers instead.
    let dc = Cdx::from_bf(&n[0].sub(&r.cx, p, RM), &n[1].sub(&r.cy, p, RM));
    assert_eq!(truth_pixel(&r, dc, max_iter), None);
    assert_eq!(truth_at(&r, &n[0], &n[1], max_iter, p), (TruthDwell::INTERIOR, true));
}

#[test]
fn the_radii_are_crossed_in_order() {
    // Outside the set the three counts climb with the radius; inside all three are -1.
    let p = 128;
    let out = truth_naive(&bfv(0.26), &bfv(0.0), 10_000, p); // escapes radius 2 at 30
    assert!(out.n2 > 0 && out.n2 < out.n256 && out.n256 < out.n4096, "{out:?}");
    // Fractadyne's smooth value: n + 1 - log2(log2|z|), and log2(log2|z|) is in (3, 4] past 256.
    assert!(out.smooth256 >= (out.n256 - 3) as f64 && out.smooth256 < (out.n256 - 2) as f64, "{out:?}");
    let inside = truth_naive(&bfv(-0.1), &bfv(0.1), 5_000, p);
    assert_eq!(inside, TruthDwell::INTERIOR);
}

/// The tip ladder's view (scripts/tip_ladder.py) as a `gw` x `gh` grid: center -2 + (0.4937 +
/// 0.0711i)·height, height 4·2^-level; pixel (x, y)'s point as the shader maps it.
fn tip_point(level: i64, x: usize, y: usize, gw: usize, gh: usize, p: usize) -> (BigFloat, BigFloat) {
    let height = FloatExp::from_f64(4.0).mul_pow2(-(level as f64));
    let (w, h) = (gw as f64, gh as f64);
    let cx = BigFloat::from_f64(-2.0, p).add(&height.mul_f64(0.4937).to_bf(p), p, RM);
    let cy = height.mul_f64(0.0711).to_bf(p);
    (
        cx.add(&height.mul_f64((x as f64 + 0.5) / h - 0.5 * w / h).to_bf(p), p, RM),
        cy.add(&height.mul_f64(0.5 - (y as f64 + 0.5) / h).to_bf(p), p, RM),
    )
}

#[test]
fn zooming_4x_about_minus_2_adds_one_to_every_count() {
    // c = -2 is a Misiurewicz point: its orbit lands on the fixed point z = 2, multiplier 4, so the
    // set near it is self-similar with factor 4 and each 4x zoom about it adds exactly one
    // iteration to every count (to within 4^-k; 400-bit mpmath agrees to 6 decimals from L = 40 to
    // 102). An oracle that shares nothing with this engine.
    // The fault it caught (2026-11-04): BOTH routes, the perturbation and the plain iteration
    // that checks it, finished the orbit in f64 from radius 2 on. Near -2 an orbit hovers just
    // outside |z| = 2 for as many steps as the zoom is deep (every point left of -2 passes radius 2
    // at the first step), and f64 cannot follow it: smooth counts off by up to 0.24 and whole
    // counts flipped at L = 42, and the check, sharing the fault, agreed.
    // Radius 2 itself is not judged: those first-step escapes are the same at every depth.
    let (gw, gh) = (32usize, 18usize);
    for &level in &[40i64, 400] {
        let p = level as usize + 96;
        let iter = 2 * level as u32 + 1000;
        let r_a = {
            let (x, y) = tip_point(level, gw / 2, gh / 2, gw, gh, p);
            truth_reference(&x, &y, iter, p)
        };
        let r_b = {
            let (x, y) = tip_point(level + 2, gw / 2, gh / 2, gw, gh, p);
            truth_reference(&x, &y, iter + 4, p)
        };
        let mut judged = 0;
        for y in 0..gh {
            for x in 0..gw {
                let (ax, ay) = tip_point(level, x, y, gw, gh, p);
                let (bx, by) = tip_point(level + 2, x, y, gw, gh, p);
                let a = truth_at(&r_a, &ax, &ay, iter, p).0;
                let b = truth_at(&r_b, &bx, &by, iter + 4, p).0;
                assert!(a.n256 > 0 && b.n256 > 0, "L={level} ({x},{y}): {a:?} {b:?}");
                let d = b.smooth256 - a.smooth256 - 1.0;
                assert!(d.abs() < 1e-6, "L={level} ({x},{y}): smooth {} then {} (off by {d:.3e})", a.smooth256, b.smooth256);
                // A whole count may flip only where the smooth count sits on a whole number.
                let f = a.smooth256 - a.smooth256.round();
                if f.abs() > 1e-6 {
                    assert_eq!(b.n256, a.n256 + 1, "L={level} ({x},{y}) radius 256");
                }
                assert_eq!(b.n4096, a.n4096 + 1, "L={level} ({x},{y}) radius 4096");
                judged += 1;
            }
        }
        assert_eq!(judged, gw * gh);
    }
    // And absolutely: the pixel nearest the antenna at the grid's right edge, L = 40, against
    // 500-bit mpmath (n 31, smooth 28.3358140733629), by both routes.
    let p = 136;
    let (x, y) = tip_point(40, 31, 10, gw, gh, p);
    let r = {
        let (cx, cy) = tip_point(40, gw / 2, gh / 2, gw, gh, p);
        truth_reference(&cx, &cy, 1080, p)
    };
    for (route, d) in [("perturbation", truth_at(&r, &x, &y, 1080, p).0), ("plain", truth_naive(&x, &y, 1080, p))] {
        assert_eq!(d.n256, 31, "{route}: {d:?}");
        assert!((d.smooth256 - 28.335_814_073_362_9).abs() < 1e-9, "{route}: {d:?}");
    }
}

#[test]
fn direct_double_double_matches_plain_iteration_beside_the_pinch_points() {
    // The pi points (scripts/pi_points.py): beside the neck c = -3/4 the count is pi/eps plus about
    // 1, beside the cusp c = 1/4 it is pi/sqrt(eps) minus about 2 (Boll 1991, Klebanoff 2001). The
    // direct route must agree with plain arbitrary-precision iteration exactly, and both with pi.
    let p = 160;
    let pi = std::f64::consts::PI;
    for (re, im, law) in [
        ("-0.75", "1e-3", pi / 1e-3),
        ("-0.75", "1e-4", pi / 1e-4),
        ("0.250001", "0", pi / 1e-3),
        ("0.25000001", "0", pi / 1e-4),
    ] {
        let (x, y) = (parse_bf_prec(re, p).unwrap(), parse_bf_prec(im, p).unwrap());
        let d = truth_direct(&x, &y, 100_000);
        let n = truth_naive(&x, &y, 100_000, p);
        assert_eq!((d.n2, d.n256, d.n4096), (n.n2, n.n256, n.n4096), "{re} + {im}i");
        assert!((d.smooth256 - n.smooth256).abs() < 1e-9, "{re} + {im}i: {d:?} {n:?}");
        assert!((d.n2 as f64 - law).abs() <= 3.0, "{re} + {im}i: {} against pi's {law:.1}", d.n2);
    }
}

#[test]
fn radius_2_is_decided_exactly_where_the_orbit_hovers_on_it() {
    // At the tip every point left of -2 has |c| > 2 by as little as the zoom is deep (1e-100 here),
    // so it is past radius 2 at the first step; the orbit then hovers on |z| = 2, where an f64 sees
    // 2.0. Taken in f64 the routes disagreed by one at radius 2 at 1e100x (2026-11-04).
    let (gw, gh) = (32usize, 18usize);
    let level = 332i64;
    let p = level as usize + 96;
    let iter = 2 * level as u32 + 1000;
    let r = {
        let (x, y) = tip_point(level, gw / 2, gh / 2, gw, gh, p);
        truth_reference(&x, &y, iter, p)
    };
    // The tip sits 0.4937 heights left of the center: grid column 6.6.
    let tip_col = gw as f64 / 2.0 - 0.5 - 0.4937 * gh as f64;
    for y in 0..gh {
        for x in 0..gw {
            let (px, py) = tip_point(level, x, y, gw, gh, p);
            let (a, plain) = truth_at(&r, &px, &py, iter, p);
            let n = truth_naive(&px, &py, iter, p);
            assert_eq!((a.n2, a.n256, a.n4096), (n.n2, n.n256, n.n4096), "({x},{y}) plain={plain}");
            if (x as f64) < tip_col {
                assert_eq!(a.n2, 1, "({x},{y}) is left of -2: |c| > 2");
            } else {
                assert!(a.n2 > 1, "({x},{y}) is right of -2: {a:?}");
            }
        }
    }
}
