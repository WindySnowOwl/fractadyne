use super::*;

/// EXPERIMENT (run: `cargo test -p fractadyne-core escape_length_vs_precision -- --ignored
/// --nocapture`): at a Misiurewicz centre, how long does the computed reference orbit stay
/// true as a function of ARITHMETIC precision? The point is parsed at full precision (116
/// digits ≈ 385 bits); only the orbit arithmetic varies. A Misiurewicz orbit is repelling, so
/// per-iteration rounding error is amplified until the computed orbit spuriously escapes —
/// the hypothesis is that the grand-tour crossover's `len=626` reference (built at 207 bits),
/// e63's "natural escape at 256,753" and e72's 602,516 are ALL this one mechanism at
/// different precisions, and that the picker itself goes blind at low precision (no candidate
/// survives phase 1, so `best_reference` falls back to "longest escaper").
/// COMPANION EXPERIMENT (run: `cargo test -p fractadyne-core --release --lib
/// escape_length_of_rounded_centre -- --ignored --nocapture`): the ROUNDED-POINT axis of the
/// cliff. `Playback::sample` used to route a pinned-centre glide through
/// `lerp_bf(a, a, ease, p)` with `p` = the CURRENT interpolated depth's precision — which is
/// not the identity: it ROUNDS the centre to `p` bits, i.e. hands the reference machinery a
/// genuinely different point. This measures the true escape length (high-precision
/// arithmetic) of `lerp_bf(centre, centre, 0.5, k)` for the glide precisions the grand tour
/// actually produces. However good the picker, it cannot beat a wrong input point.
#[test]
#[ignore]
fn escape_length_of_rounded_centre() {
    let cxs = "-1.0109636384562213181006238475735192993836101418531854095957676926471683503366629508912671364125546238220995191834757e-1";
    let cys = "9.5628651080914147131604703998237075557983304380930462483482733212267499793490593467836270525491219946548323699651521e-1";
    let cx = crate::parse_bf(cxs).unwrap();
    let cy = crate::parse_bf(cys).unwrap();
    let max_iter = 700_000u32;
    let arith = 480usize; // far above every cliff measured on the arithmetic axis
    let zero = BigFloat::from_f64(0.0, arith);
    for k in [64usize, 78, 96, 128, 157, 206, 300] {
        let rx = crate::lerp_bf(&cx, &cx, 0.5, k);
        let ry = crate::lerp_bf(&cy, &cy, 0.5, k);
        let n = orbit_length_bf(&zero, &zero, &rx, &ry, 0, max_iter, arith);
        println!(
            "lerp'd at {k:4} bits -> true escape {n}{}",
            if n >= max_iter { "  (SURVIVED)" } else { "" }
        );
    }
}

#[test]
#[ignore]
fn escape_length_vs_precision() {
    let cxs = "-1.0109636384562213181006238475735192993836101418531854095957676926471683503366629508912671364125546238220995191834757e-1";
    let cys = "9.5628651080914147131604703998237075557983304380930462483482733212267499793490593467836270525491219946548323699651521e-1";
    let cx = crate::parse_bf(cxs).unwrap();
    let cy = crate::parse_bf(cys).unwrap();
    let max_iter = 700_000u32;
    for p in [128usize, 160, 180, 207, 240, 286, 340, 400, 480] {
        let zero = BigFloat::from_f64(0.0, p);
        let n = orbit_length_bf(&zero, &zero, &cx, &cy, 0, max_iter, p);
        println!("prec {p:4} bits -> escape length {n}{}", if n >= max_iter { "  (SURVIVED)" } else { "" });
    }
}

// Extending a cached (truncated) orbit must be byte-identical to a from-scratch build to the same
// length — this is what lets a deep dive reuse the prior orbit instead of recomputing every step.
#[test]
fn extend_orbit_is_byte_identical_to_fresh() {
    let p = 220usize;
    let zero = BigFloat::from_f64(0.0, p);
    let cases: [(&str, &str, u32, u32); 3] = [
        // Seahorse boundary point: survives thousands of iters → the extend path actually runs.
        ("-0.7436438870371587047521915061147707", "0.131825904205311970493132056385139", 1500, 6000),
        ("-0.7436438870371587047521915061147707", "0.131825904205311970493132056385139", 64, 5000),
        // Fast-escaping exterior point: coarse already escaped → extend is a no-op, still matches.
        ("1.0", "0.5", 64, 5000),
    ];
    for (cxs, cys, coarse, full) in cases {
        let cx = crate::parse_bf(cxs).unwrap();
        let cy = crate::parse_bf(cys).unwrap();
        let (fresh, fresh_len) = reference_orbit(&zero, &zero, &cx, &cy, 0, full, p);
        let (pre, _pl, tail) = reference_orbit_t(&zero, &zero, &cx, &cy, 0, coarse, p);
        let (ext, ext_len, _t) = extend_reference_orbit(&pre, &tail, &cx, &cy, 0, full, p);
        assert_eq!(ext_len, fresh_len, "len mismatch @({cxs},{cys}) coarse {coarse} full {full}");
        assert_eq!(ext, fresh, "extended != fresh @({cxs},{cys}) coarse {coarse} full {full}");
    }
}

const FREQ: f64 = 6.0; // stripe angular frequency
const POWER: f64 = 2.0; // Mandelbrot TIA power

#[inline]
fn mag(x: f64, y: f64) -> f64 {
    (x * x + y * y).sqrt()
}
#[inline]
fn stripe_term(x: f64, y: f64) -> f64 {
    0.5 + 0.5 * (FREQ * y.atan2(x)).sin()
}
#[inline]
fn tia_term(prev: f64, cur: f64, cmag: f64) -> f64 {
    let m = prev.max(1e-12).powf(POWER);
    let lower = (m - cmag).abs();
    let upper = m + cmag;
    ((cur - lower) / (upper - lower).max(1e-9)).clamp(0.0, 1.0)
}
#[inline]
fn zval(orbit: &[[f32; 4]], k: u32) -> (f64, f64) {
    let z = orbit[k as usize];
    (z[0] as f64 + z[2] as f64, z[1] as f64 + z[3] as f64)
}

/// Brute-force a node's aggregate over its landing range Z_{start+1..=start+span} — the ground
/// truth the precomputed `BlaNode.agg_*` (seeded + merged in `build_bla_mandel`) must match. The
/// `k >= 2` guard mirrors the global TIA `n>=1` skip of the very first landing iterate z_1.
fn brute_node_agg(orbit: &[[f32; 4]], start: u32, span: u32, cmag: f64) -> (f64, f64, f64) {
    let (mut trap, mut ss, mut ts) = (1e30f64, 0.0f64, 0.0f64);
    for k in (start + 1)..=(start + span) {
        let (zx, zy) = zval(orbit, k);
        trap = trap.min(mag(zx, zy)); // point trap
        ss += stripe_term(zx, zy);
        if k >= 2 {
            let (px, py) = zval(orbit, k - 1);
            ts += tia_term(mag(px, py), mag(zx, zy), cmag);
        }
    }
    (trap, ts, ss)
}

/// Running aux state mirroring the shader's `aux_step` (point trap, stripe, power-2 TIA).
#[derive(Clone, Copy)]
struct Aux {
    trap: f64,
    ssum: f64,
    tsum: f64,
    n: f64,
    prev_abs: f64,
}
impl Aux {
    fn init(z0: (f64, f64)) -> Self {
        Aux { trap: 1e30, ssum: 0.0, tsum: 0.0, n: 0.0, prev_abs: mag(z0.0, z0.1) }
    }
    /// One *actual* post-step iterate z (exact path, and BLA full steps).
    fn push_exact(&mut self, z: (f64, f64), cmag: f64) {
        let cur = mag(z.0, z.1);
        self.trap = self.trap.min(cur);
        self.ssum += stripe_term(z.0, z.1);
        if self.n >= 1.0 {
            self.tsum += tia_term(self.prev_abs, cur, cmag);
        }
        self.prev_abs = cur;
        self.n += 1.0;
    }
    /// Fold a precomputed per-node aggregate on a skip (m → nm) — the actual GPU behavior: add
    /// the node's Σ/min, advance the count by its span, and restore the exit `prev_abs` to the
    /// *actual* landing |z_nm| (exact seam, since the shader carries δz there). The TIA entry
    /// seam (reference |Z_m| as the first `prev`) is already baked into `node.agg_tia`.
    fn fold_node(&mut self, node: &BlaNode, landing_abs: f64) {
        self.trap = self.trap.min(node.agg_trap);
        self.ssum += node.agg_stripe;
        self.tsum += node.agg_tia;
        self.n += node.span as f64;
        self.prev_abs = landing_abs;
    }
    fn stripe_avg(&self) -> f64 {
        self.ssum / self.n.max(1.0)
    }
    fn tia_avg(&self) -> f64 {
        self.tsum / (self.n - 1.0).max(1.0)
    }
}

/// Exact perturbation iteration (no skips), accumulating aux at every step.
fn run_exact(
    orbit: &[[f32; 4]],
    dc: (f64, f64),
    bailout2: f64,
    max_iter: u32,
    cmag: f64,
) -> (Option<f64>, Aux) {
    let dc_c = CFloatExp { re: FloatExp::from_f64(dc.0), im: FloatExp::from_f64(dc.1) };
    let mut dz = CFloatExp { re: FloatExp::ZERO, im: FloatExp::ZERO };
    let mut m: u32 = 0;
    let nstep = orbit.len().saturating_sub(1) as u32;
    let mut aux = Aux::init(bla_full_z(orbit, 0, &dz));
    loop {
        if m >= max_iter || m >= nstep {
            return (None, aux);
        }
        let (zrx, zry) = zval(orbit, m);
        let two_z = CFloatExp {
            re: FloatExp::from_f64(2.0 * zrx),
            im: FloatExp::from_f64(2.0 * zry),
        };
        dz = two_z * dz + dz * dz + dc_c;
        m += 1;
        let (zx, zy) = bla_full_z(orbit, m, &dz);
        aux.push_exact((zx, zy), cmag);
        let mag2 = zx * zx + zy * zy;
        if mag2 > bailout2 {
            let nu = (mag2.ln() * 0.5 / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
            return (Some(m as f64 + 1.0 - nu), aux);
        }
    }
}

/// BLA-skipping iteration mirroring `bla_iterate`, folding the reference aggregate on skips.
/// Returns (escape, aux, skips, full_steps).
fn run_folded(
    orbit: &[[f32; 4]],
    levels: &[Vec<BlaNode>],
    dc: (f64, f64),
    bailout2: f64,
    max_iter: u32,
    cmag: f64,
) -> (Option<f64>, Aux, u32, u32) {
    let dc_c = CFloatExp { re: FloatExp::from_f64(dc.0), im: FloatExp::from_f64(dc.1) };
    let mut dz = CFloatExp { re: FloatExp::ZERO, im: FloatExp::ZERO };
    let mut m: u32 = 0;
    let nstep = orbit.len().saturating_sub(1) as u32;
    let mut aux = Aux::init(bla_full_z(orbit, 0, &dz));
    let (mut skips, mut fulls) = (0u32, 0u32);
    loop {
        if m >= max_iter || m >= nstep {
            return (None, aux, skips, fulls);
        }
        let dzmag = dz.abs();
        let mut applied = false;
        for l in (0..levels.len()).rev() {
            let step = 1u32 << l;
            if (m & (step - 1)) != 0 {
                continue;
            }
            let Some(&node) = levels[l].get((m >> l) as usize) else { continue };
            if m + node.span > nstep || !dzmag.lt(node.r) {
                continue;
            }
            let ndz = node.a * dz + node.b * dc_c;
            let nm = m + node.span;
            let (zx, zy) = bla_full_z(orbit, nm, &ndz);
            if zx * zx + zy * zy > bailout2 {
                continue;
            }
            aux.fold_node(&node, mag(zx, zy));
            dz = ndz;
            m = nm;
            applied = true;
            skips += 1;
            break;
        }
        if applied {
            continue;
        }
        let (zrx, zry) = zval(orbit, m);
        let two_z = CFloatExp {
            re: FloatExp::from_f64(2.0 * zrx),
            im: FloatExp::from_f64(2.0 * zry),
        };
        dz = two_z * dz + dz * dz + dc_c;
        m += 1;
        fulls += 1;
        let (zx, zy) = bla_full_z(orbit, m, &dz);
        aux.push_exact((zx, zy), cmag);
        let mag2 = zx * zx + zy * zy;
        if mag2 > bailout2 {
            let nu = (mag2.ln() * 0.5 / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
            return (Some(m as f64 + 1.0 - nu), aux, skips, fulls);
        }
    }
}

/// Sweep one view; print per-method fold error + the view's min |Z| (the stripe stressor).
/// Returns the trap max error (the oracle-bug canary) so the caller can assert on it.
fn analyze(label: &str, cx_str: &str, cy_str: &str, octaves: u64, dc_ext: f64) -> f64 {
    let p = crate::precision_for_octaves(octaves);
    let cx = crate::parse_bf(cx_str).unwrap();
    let cy = crate::parse_bf(cy_str).unwrap();
    let z0 = BigFloat::from_f64(0.0, p);
    let max_iter = 6000u32;
    let (orbit, len) = reference_orbit(&z0, &z0, &cx, &cy, formula::MANDELBROT, max_iter, p);
    let cmag = mag(to_f64(&cx), to_f64(&cy));
    let bailout2 = 1.0e10f64;
    let dc_max = FloatExp::from_f64(dc_ext * 1.5);
    let aux_p = AuxAggParams { trap_type: 0, stripe_freq: FREQ, cmag, power: POWER };
    let levels = build_bla_mandel(&orbit, dc_max, 1.0e-6, aux_p);

    // Phase-1 precompute check: every node's seeded+merged aggregate must equal a brute-force
    // over its landing range (catches a wrong seed index, a broken merge, or a mis-placed guard).
    for (l, level) in levels.iter().enumerate() {
        for (j, node) in level.iter().enumerate() {
            let start = (j as u32) << l;
            let (bt, bx, bs) = brute_node_agg(&orbit, start, node.span, cmag);
            let close = |a: f64, b: f64| (a - b).abs() <= 1e-9 * a.abs().max(1.0) + 1e-12;
            assert!(close(bt, node.agg_trap), "{label}: l{l} j{j} trap {bt} vs {}", node.agg_trap);
            assert!(close(bx, node.agg_tia), "{label}: l{l} j{j} tia {bx} vs {}", node.agg_tia);
            assert!(close(bs, node.agg_stripe), "{label}: l{l} j{j} stripe {bs} vs {}", node.agg_stripe);
        }
    }

    // Min |Z| along the reference (how ill-conditioned stripe's arg gets here). Skip Z_0 = 0
    // (the trivial Mandelbrot critical point) — aux accumulates z_{k≥1}, not z_0.
    let min_z = orbit
        .iter()
        .take(len as usize)
        .skip(1)
        .map(|z| mag(z[0] as f64 + z[2] as f64, z[1] as f64 + z[3] as f64))
        .fold(f64::INFINITY, f64::min);

    let grid = 48u32;
    let (mut tmax, mut tsum) = (0.0f64, 0.0f64);
    let (mut smax, mut ssum) = (0.0f64, 0.0f64);
    let (mut xmax, mut xsum) = (0.0f64, 0.0f64);
    let (mut cnt, mut esc_mismatch, mut tot_skips, mut tot_full) = (0u32, 0u32, 0u64, 0u64);
    for i in 0..grid {
        for j in 0..grid {
            let dcx = -dc_ext + 2.0 * dc_ext * (i as f64) / ((grid - 1) as f64);
            let dcy = -dc_ext + 2.0 * dc_ext * (j as f64) / ((grid - 1) as f64);
            let (ee, ea) = run_exact(&orbit, (dcx, dcy), bailout2, max_iter, cmag);
            let (fe, fa, sk, fl) = run_folded(&orbit, &levels, (dcx, dcy), bailout2, max_iter, cmag);
            let (Some(ei), Some(fi)) = (ee, fe) else { continue };
            cnt += 1;
            tot_skips += sk as u64;
            tot_full += fl as u64;
            if (ei - fi).abs() > 0.5 {
                esc_mismatch += 1;
            }
            tmax = tmax.max((ea.trap - fa.trap).abs());
            tsum += (ea.trap - fa.trap).abs();
            smax = smax.max((ea.stripe_avg() - fa.stripe_avg()).abs());
            ssum += (ea.stripe_avg() - fa.stripe_avg()).abs();
            xmax = xmax.max((ea.tia_avg() - fa.tia_avg()).abs());
            xsum += (ea.tia_avg() - fa.tia_avg()).abs();
        }
    }
    let c = cnt.max(1) as f64;
    eprintln!("\n=== {label}: ref len {len}, min|Z| {min_z:.3e}, {cnt} escaping px, {esc_mismatch} escape mismatch ===");
    eprintln!("  BLA: {tot_skips} skips / {tot_full} full steps");
    eprintln!("  trap   : max {:.3e}  mean {:.3e}", tmax, tsum / c);
    eprintln!("  stripe : max {:.3e}  mean {:.3e}   (0..1)", smax, ssum / c);
    eprintln!("  TIA    : max {:.3e}  mean {:.3e}   (0..1)", xmax, xsum / c);
    assert!(tot_skips > 0, "{label}: BLA never skipped — oracle exercised no folds");
    tmax
}

#[test]
fn aux_bla_fold_error() {
    // The seahorse valley is a structured boundary point whose orbit spirals near 0 (low |Z|),
    // so it already exercises stripe's ill-conditioned regime. Two pixel extents (shallower =
    // larger δc = looser BLA) bracket how the fold error scales with the perturbation size.
    let mut trap_worst = 0.0f64;
    let sx = "-0.7436438870371587047521915061147707";
    let sy = "0.131825904205311970493132056385139";
    trap_worst = trap_worst.max(analyze("seahorse δc~2e-12", sx, sy, 40, 2.0e-12));
    trap_worst = trap_worst.max(analyze("seahorse δc~2e-6", sx, sy, 40, 2.0e-6));
    // Canary across all views: trap's fold is a min over ~identical values → tiny error;
    // a large value signals an oracle bug (indexing / seam), not a method verdict.
    assert!(
        trap_worst < 0.05,
        "trap fold error {trap_worst:.3e} too large — oracle bug, not a method verdict"
    );
}

// The Misiurewicz finder must Newton-snap from a nearby seed onto the exact pre-periodic point.
#[test]
fn misiurewicz_solver_snaps_to_known_points() {
    // c = -2 is Misiurewicz (2,1): Z_1=-2, Z_2=2, Z_3=2 ⇒ Z_3=Z_2.
    let seed = [crate::parse_bf("-1.98").unwrap(), crate::parse_bf("0.01").unwrap()];
    let m = find_misiurewicz(&seed, 2, 1, SolveScale::here(0.0), 0).expect("M(2,1) near c=-2");
    assert!(
        (to_f64(&m.cx) + 2.0).abs() < 1e-12 && to_f64(&m.cy).abs() < 1e-12,
        "M(2,1) should be c=-2, got ({}, {})",
        to_f64(&m.cx),
        to_f64(&m.cy)
    );

    // The verified three-spar Misiurewicz (4,1) (see validation/misiurewicz-4-1.fdn).
    let seed = [crate::parse_bf("-0.10110").unwrap(), crate::parse_bf("0.95629").unwrap()];
    let m = find_misiurewicz(&seed, 4, 1, SolveScale::here(1.0e3f64.log2()), 0).expect("M(4,1) three-spar");
    assert!(
        (to_f64(&m.cx) - (-0.10109636384562216)).abs() < 1e-12
            && (to_f64(&m.cy) - 0.9562865108091415).abs() < 1e-12,
        "M(4,1) off: ({}, {})",
        to_f64(&m.cx),
        to_f64(&m.cy)
    );
}
