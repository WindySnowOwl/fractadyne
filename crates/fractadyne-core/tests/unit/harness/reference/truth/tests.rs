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
    // centre, at the set's cap of 30 periods: a grid over the whole view, every pixel compared.
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
            match truth_pixel(&r, dc, (to_f64(&px), to_f64(&py)), max_iter) {
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
    assert_eq!(truth_pixel(&r, dc, (to_f64(&n[0]), to_f64(&n[1])), max_iter), None);
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
