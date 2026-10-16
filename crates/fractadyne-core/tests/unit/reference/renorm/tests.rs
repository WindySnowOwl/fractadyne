use super::*;

/// A reference orbit in f64 (shallow minibrots only), packed like the real one.
fn orbit_f64(cx: f64, cy: f64, n: usize) -> Vec<[f32; 4]> {
    let (mut x, mut y) = (0.0f64, 0.0f64);
    let mut out = vec![super::super::pack_sample(0.0, 0.0)];
    for _ in 0..n {
        let nx = x * x - y * y + cx;
        y = 2.0 * x * y + cy;
        x = nx;
        out.push(super::super::pack_sample(x, y));
    }
    out
}

fn cmul(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

fn cf(c: CFloatExp) -> (f64, f64) {
    (c.re.to_f64(), c.im.to_f64())
}

#[test]
fn period_two_matches_its_closed_form() {
    // c₀ = −1: Z = 0, −1, 0. F(z) = (z² − 1)² − 1 = z⁴ − 2z², so a = −2, b = dZ₂/dc = 2Z₁·1 + 1
    // = −1, RefC = a·Z₂ = 0, and in u = −2z the map is u² + c′ − u⁴/8: κ = −1/8.
    let orbit = orbit_f64(-1.0, 0.0, 2);
    let (mut a, mut d, mut e, mut s, mut rho) = (Xc::real(1.0), Xc::real(1.0), XC_ZERO, XC_ZERO, XC_ZERO);
    let z1 = sample_xc(&orbit[1]);
    let z2 = z1.scale2();
    let inv = z2.recip();
    s = s.add(d.mul(z1.recip()));
    rho = inv.mul(inv).scale2().add(rho.mul(inv));
    a = a.mul(z2);
    e = e.mul(z2).add(d.mul(d).scale2());
    d = d.mul(z2).add(Xc::real(1.0));
    let _ = (e, s);
    assert_eq!(cf(a.to_cfe()), (-2.0, 0.0));
    assert_eq!(cf(d.to_cfe()), (-1.0, 0.0));
    let kappa = rho.mul(a.scale2().recip());
    assert!((kappa.log2_abs() - (-3.0)).abs() < 1e-12, "{}", kappa.log2_abs());
    // ...and it is refused anyway: shorter than RENORM_MIN_LEN, and R² = 2^-24/(1/8) is far
    // below the set.
    assert!(candidate(2, sample_xc(&orbit[2]), a, d, e, s, rho).is_none());
}

/// The model against the iteration itself, at the ladder's period-953 minibrot (scene 56,
/// 1.3e53×): for pixels around the nucleus, k renormalized steps from u = 0 must land where
/// k·953 direct bignum iterations do, scaled by a — to within the model's error.
#[test]
fn renormalized_steps_follow_the_direct_iteration() {
    use crate::{parse_real_expr, to_f64, BigFloat};
    let prec = 400;
    let cx = parse_real_expr("-2.804105430550454669840777002898397927204098765083452471337241940737508121866324414807243345475023e-2", prec).unwrap();
    let cy = parse_real_expr("6.948927538996523858929943394989672880373767486737755673829879259356243644967862160620146096103277e-1", prec).unwrap();
    let zero = BigFloat::from_f64(0.0, prec);
    let (orbit, _, _) = super::super::reference_orbit_t(&zero, &zero, &cx, &cy, 0, 960, prec);
    // The view at 1.34e53×: radius 2/zoom ≈ 2^-176.
    let log2_view = (2.0f64 / 1.342765e53).log2();
    let step = renorm_step(&orbit).expect("the period is usable");
    assert_eq!(step.len, 953, "{step:?}");
    let (a, b, rc) = (cf(step.a), cf(step.b), cf(step.ref_c));
    let ab = cmul(a, b);
    let r = step.log2_r.exp2();
    let view = log2_view.exp2();
    let mut checked = 0;
    for (i, frac) in [0.002f64, 0.01, 0.05, 0.2, 0.6].iter().enumerate() {
        let th = 0.7 + i as f64;
        let dc = (view * frac * th.cos(), view * frac * th.sin());
        assert!(dc.0.hypot(dc.1).log2() <= step.log2_dc_max, "the test pixel is inside the model's |dc|");
        let cp = (rc.0 + ab.0 * dc.0 - ab.1 * dc.1, rc.1 + ab.0 * dc.1 + ab.1 * dc.0);
        let pcx = cx.add(&BigFloat::from_f64(dc.0, prec), prec, crate::bignum::RM);
        let pcy = cy.add(&BigFloat::from_f64(dc.1, prec), prec, crate::bignum::RM);
        let (mut zx, mut zy) = (zero.clone(), zero.clone());
        let mut u = (0.0f64, 0.0f64);
        for _k in 1..=6 {
            for _ in 0..953 {
                let (nx, ny) = super::super::step_bf(&zx, &zy, &pcx, &pcy, 0, prec);
                zx = nx;
                zy = ny;
            }
            u = (u.0 * u.0 - u.1 * u.1 + cp.0, 2.0 * u.0 * u.1 + cp.1);
            let um = u.0.hypot(u.1);
            if um > r || um > 1e6 {
                break;
            }
            let uz = cmul(a, (to_f64(&zx), to_f64(&zy)));
            let tol = 1e-6 * um.max(1.0);
            assert!((uz.0 - u.0).hypot(uz.1 - u.1) <= tol, "dc {dc:?}: a·z {uz:?} vs u {u:?}");
            checked += 1;
        }
    }
    assert!(checked >= 10, "the comparison ran ({checked})");
}

/// The u-space tree against the steps it replaces, at the ladder's period-15,248 minibrot
/// (953 × 16, 2.1e57×): the step is the parent's 953 and the u-reference repeats every 16 steps.
/// For a pixel inside a deep view, each level's node at `k = 2^l` (span `2^l`) must land where
/// `2^l` perturbed u-steps do, to within the tree's tolerance.
#[test]
fn the_u_space_tree_follows_the_u_steps_it_skips() {
    use crate::{parse_real_expr, BigFloat};
    let prec = 400;
    let cx = parse_real_expr("-2.804105430550454669840777002898397927204098765083451958014259277838710256889075333271267217529135e-2", prec).unwrap();
    let cy = parse_real_expr("6.948927538996523858929943394989672880373767486737755680968675269305405323393245987015207130043252e-1", prec).unwrap();
    let zero = BigFloat::from_f64(0.0, prec);
    let (orbit, _, _) = super::super::reference_orbit_t(&zero, &zero, &cx, &cy, 0, 15_248, prec);
    let step = renorm_step(&orbit).expect("the parent's period is usable");
    assert_eq!(step.len, 953, "{step:?}");
    let k = ((orbit.len() - 1) / 953) as u32;
    assert_eq!(k, 16);
    // A view as deep as the ladder's 1e68 scene (whose step is this same parent's): in u-space
    // 57's own view spans ~1e-4 of c′, where |B·δc′| outweighs every merged radius — a shallow
    // view, as a z-space one at 1e4× is — while 1e68's spans ~1e-15.
    let view = (4.0f64 / 2.0e68).log2();
    let eps = 1e-6;
    let tree = renorm_bla_gpu(&orbit, &step, FloatExp::from_f64(1.0).mul_pow2(view), eps);
    assert_eq!(tree.len(), renorm_bla_nodes(k) * 4);
    let decode = |m: [f32; 4], e: f32| -> (f64, f64) {
        let s = (e as f64).exp2();
        ((m[0] as f64 + m[1] as f64) * s, (m[2] as f64 + m[3] as f64) * s)
    };
    // The u-reference, as the shader forms it, and a pixel's δc′ a thousandth of the view out.
    let a = cf(step.a);
    let uref: Vec<(f64, f64)> = (0..=k as usize)
        .map(|j| {
            let (zx, zy) = super::super::sample_xy(&orbit[j * 953]);
            cmul(a, (zx, zy))
        })
        .collect();
    // Level 0's A is 2·U_k (node 0: U_0 = 0, so it can never apply).
    for (j, u) in uref.iter().enumerate().take(k as usize) {
        let (ax, ay) = decode(tree[j * 4], tree[j * 4 + 2][0]);
        assert!((ax - 2.0 * u.0).hypot(ay - 2.0 * u.1) <= 1e-6 * (1.0 + u.0.hypot(u.1)), "node {j}");
    }
    let ab = cmul(a, cf(step.b));
    let dc = 1e-3 * view.exp2();
    let dcp = cmul(ab, (dc * 0.6, dc * 0.8));
    let plain = |mut du: (f64, f64), from: usize, n: usize| {
        for i in from..from + n {
            let u = uref[i];
            let t = cmul((2.0 * u.0, 2.0 * u.1), du);
            let sq = cmul(du, du);
            du = (t.0 + sq.0 + dcp.0, t.1 + sq.1 + dcp.1);
        }
        du
    };
    let mut off = 0usize;
    let mut len = k as usize;
    let mut checked = 0;
    for l in 0.. {
        let span = 1usize << l;
        if span < k as usize && 1 < len {
            // The node at k = span (j = 1) covers steps span..2·span.
            let node = (off + 1) * 4;
            let du0 = plain((0.0, 0.0), 0, span);
            let (am, bm, ex, sp) = (tree[node], tree[node + 1], tree[node + 2], tree[node + 3]);
            assert_eq!(sp[0] as usize, span.min(k as usize - span), "level {l} span");
            let r = (ex[3] as f64) * (ex[2] as f64).exp2();
            if du0.0.hypot(du0.1) < r && sp[0] as usize == span {
                let (ax, ay) = decode(am, ex[0]);
                let (bx, by) = decode(bm, ex[1]);
                let lin = cmul((ax, ay), du0);
                let lin = (lin.0 + bx * dcp.0 - by * dcp.1, lin.1 + bx * dcp.1 + by * dcp.0);
                let want = plain(du0, span, span);
                let err = (lin.0 - want.0).hypot(lin.1 - want.1);
                assert!(err <= 1e-4 * want.0.hypot(want.1), "level {l}: skip {lin:?} vs steps {want:?}");
                checked += 1;
            }
        }
        if len <= 1 {
            break;
        }
        off += len;
        len = len.div_ceil(2);
    }
    assert!(checked >= 3, "the comparison ran at {checked} levels");
}

/// Probe (ignored; `cargo test --release -p fractadyne-core -- --ignored probe_u_space --nocapture`):
/// the shader's u-space walk, BLA and all, on the CPU at the ladder's 1e68 scene, printing what
/// the u-reference looks like and how far each skip goes.
#[test]
#[ignore]
fn probe_u_space_at_the_1e68_minibrot() {
    use crate::{parse_real_expr, BigFloat};
    let prec = 280;
    let cx = parse_real_expr("-2.8041054305504546698407770028983979272040987650834524316481736852848679363432650309792659781081363390252362581574126e-2", prec).unwrap();
    let cy = parse_real_expr("6.9489275389965238589299433949896728803737674867377557299604836990561657887294711299974639904251203515742450986635039e-1", prec).unwrap();
    let zero = BigFloat::from_f64(0.0, prec);
    let (orbit, _, _) = super::super::reference_orbit_t(&zero, &zero, &cx, &cy, 0, 951_094, prec);
    let step = renorm_step(&orbit).expect("step");
    let l = step.len as usize;
    let k = (orbit.len() - 1) / l;
    println!("step {} K {k} log2|a| {:.1} log2|ab| {:.1} R 2^{:.0}", step.len, step.a.abs().log2(), (step.a * step.b).abs().log2(), step.log2_r);
    let a = cf(step.a);
    let uref: Vec<(f64, f64)> = (0..=k).map(|j| cmul(a, super::super::sample_xy(&orbit[j * l]))).collect();
    let mut mags: Vec<(f64, usize)> = uref.iter().enumerate().map(|(j, u)| (u.0.hypot(u.1), j)).collect();
    let lp: f64 = mags.iter().skip(1).take(k - 1).map(|(m, _)| (2.0 * m).log2()).sum();
    mags.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
    println!("smallest |U_k|: {:?}", &mags[..12]);
    println!("log2 |prod 2U_k| over the cycle: {lp:.1}; median |U| {:.3}", mags[mags.len() / 2].0);
    let view = (4.0f64 / 2.142891e68).log2();
    let eps = 1e-6;
    let tree = renorm_bla_gpu(&orbit, &step, FloatExp::from_f64(2.5).mul_pow2(view), eps);
    let decode = |m: [f32; 4], e: f32| -> (f64, f64) {
        let s = (e as f64).exp2();
        ((m[0] as f64 + m[1] as f64) * s, (m[2] as f64 + m[3] as f64) * s)
    };
    let mut lv = vec![];
    let (mut off, mut len) = (0usize, k);
    loop {
        lv.push((off, len));
        if len <= 1 {
            break;
        }
        off += len;
        len = len.div_ceil(2);
    }
    let ab = cmul(a, cf(step.b));
    let r2 = (2.0 * step.log2_r).exp2();
    for frac in [0.05f64, 0.2, 0.45] {
        let dc = frac * view.exp2();
        let dcp = cmul(ab, (dc * 0.6, dc * 0.8));
        let (mut du, mut ddu, mut kr, mut kk) = ((0.0f64, 0.0f64), (0.0f64, 0.0f64), 0usize, 0usize);
        let (mut trips, mut skipped, mut rebases) = (0u64, 0u64, 0u64);
        let mut hist = [0u64; 12];
        let k_max = 28_532_820usize.div_ceil(l);
        let mut out = false;
        while kk < k_max && trips < 200_000 {
            trips += 1;
            let dum = du.0.hypot(du.1);
            let mut applied = false;
            for lvl in (0..lv.len()).rev() {
                let stepn = 1usize << lvl;
                if kr & (stepn - 1) != 0 {
                    continue;
                }
                let j = kr >> lvl;
                if j >= lv[lvl].1 {
                    continue;
                }
                let n = (lv[lvl].0 + j) * 4;
                let ex = tree[n + 2];
                let span = tree[n + 3][0] as usize;
                if span == 0 || kr + span > k || kk + span > k_max {
                    continue;
                }
                let r = ex[3] as f64 * (ex[2] as f64).exp2();
                if !(dum < r) {
                    continue;
                }
                let am = decode(tree[n], ex[0]);
                let bm = decode(tree[n + 1], ex[1]);
                let t = cmul(am, du);
                let t2 = cmul(bm, dcp);
                let ndu = (t.0 + t2.0, t.1 + t2.1);
                let nkr = kr + span;
                let u = (uref[nkr].0 + ndu.0, uref[nkr].1 + ndu.1);
                if u.0 * u.0 + u.1 * u.1 > r2 {
                    continue;
                }
                let dd = cmul(am, ddu);
                ddu = (dd.0 + bm.0, dd.1 + bm.1);
                du = ndu;
                kr = nkr;
                kk += span;
                hist[lvl.min(11)] += 1;
                skipped += span as u64;
                if u.0.hypot(u.1) < du.0.hypot(du.1) || (kr + 1) * l >= orbit.len() {
                    du = u;
                    kr = 0;
                    rebases += 1;
                }
                applied = true;
                break;
            }
            if applied {
                continue;
            }
            let uk = uref[kr];
            let t = cmul((2.0 * uk.0, 2.0 * uk.1), du);
            let sq = cmul(du, du);
            du = (t.0 + sq.0 + dcp.0, t.1 + sq.1 + dcp.1);
            kr += 1;
            kk += 1;
            let u = (uref[kr].0 + du.0, uref[kr].1 + du.1);
            if u.0.hypot(u.1) < du.0.hypot(du.1) || (kr + 1) * l >= orbit.len() {
                du = u;
                kr = 0;
                rebases += 1;
            }
            if u.0 * u.0 + u.1 * u.1 > r2 {
                out = true;
                break;
            }
        }
        println!(
            "pixel {frac}: u-steps {kk} trips {trips} ({:.1}/trip) skipped {skipped} rebases {rebases} out {out}; skips by level {hist:?}",
            kk as f64 / trips as f64
        );
    }
}

#[test]
fn a_trivial_step_is_never_offered() {
    // L = 1 is an empty product (κ = 0, every limit infinite): it passes the model tests, and
    // must still never be chosen. A wide view of the main cardioid's centre has nothing else.
    let orbit = orbit_f64(-0.1, 0.05, 200);
    assert!(renorm_step(&orbit).map_or(true, |st| st.len >= RENORM_MIN_LEN), "{:?}", renorm_step(&orbit));
}

#[test]
fn an_extended_dip_keeps_its_exponent() {
    let mut s = [0.0f32, -5000.0, 4.0 + 1.25, 0.0]; // 1.25·2^-5000 in the extended form
    s[1] = -5000.0;
    let x = sample_xc(&s);
    assert!((x.log2_abs() - (1.25f64.log2() - 5000.0)).abs() < 1e-9);
}
