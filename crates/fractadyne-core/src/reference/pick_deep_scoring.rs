use super::*;

/// δc = 0 must replay the reference exactly: the δ-recurrence stays at zero, the escape
/// test sees the recorded samples themselves, and the returned length is the recorded
/// walk's own. This is the identity the duplicate-centre candidates in the 5×5 grid rely
/// on (offset (0,0) exists at every scale), so it is pinned per supported power.
#[test]
fn a_zero_delta_perturbation_replays_the_reference() {
    let p = 96;
    for (formula, cx, cy) in [
        (formula::MANDELBROT, -0.743643887037151, 0.131825904205330),
        (formula::MULTIBROT3, 0.219533102209776, 0.731777007365920),
        (formula::MULTIBROT5, 0.232076866967485, 0.773589556558284),
    ] {
        let zero = bf(0.0, p);
        let (len, orbit) = orbit_length_bf_recorded(
            &zero,
            &zero,
            &bf(cx, p),
            &bf(cy, p),
            formula,
            50_000,
            p,
        );
        assert!(len < 50_000, "fixture must escape (formula {formula}, got {len})");
        assert_eq!(orbit.len() as u32, len + 1, "one sample per step plus the start");
        let mut rebases = 0;
        let power = formula_power(formula).unwrap();
        let got = perturb_orbit_length(&orbit, CFloatExp::ZERO, power, 50_000, &mut rebases);
        assert_eq!(rebases, 0, "a replay never rebases (formula {formula})");
        assert_eq!(
            got,
            Some(len),
            "δc = 0 must reproduce the recorded length (formula {formula})"
        );
    }
}

/// The scorer against its oracle, directly: for a spread of candidates around a deep
/// boundary coordinate — on-filament (outliving the reference), off-filament (escaping
/// before AND after it), and the reference itself — every TRUSTED score
/// (`Some`) from `perturb_orbit_length` must equal the escape length `orbit_length_bf`
/// computes in full precision, and a DISTRUSTED score (`None`, the post-first-rebase trust
/// budget ran out) is the fallback signal the pick answers with a bignum walk. The spread
/// deliberately includes candidates far outliving the reference: the probes that sized the
/// trust window caught a real survivor mis-scored at 0.6× there, so this pins both sides —
/// trusted-means-exact AND drift-prone-means-flagged. Runs per supported power — the arm
/// the Mandelbrot-only `--pickcheck` ladder cannot reach for the multibrot families.
/// `min_trusted` / `min_flagged` pin the fixture's regime: the tight-spread Mandelbrot set
/// must trust everything (the deep-zoom shape the redesign exists for), and the wide-spread
/// multibrot sets must actually trip the trust budget (the drift-prone shape it defends
/// against) — otherwise the respective path went untested and the test cannot go red.
fn scorer_matches_oracle_at(
    formula: u32,
    cx_s: &str,
    cy_s: &str,
    min_trusted: u32,
    min_flagged: u32,
) {
    let p = 164;
    let power = formula_power(formula).unwrap();
    let cx = parse_bf_prec(cx_s, p).unwrap();
    let cy = parse_bf_prec(cy_s, p).unwrap();
    let span = FloatExp::from_f64(1.0e-30);
    let max_iter = 30_000u32;
    let zero = bf(0.0, p);
    let mut cands: Vec<[BigFloat; 2]> = Vec::new();
    for (fx, fy) in [
        (0.0, 0.0),
        (0.04, 0.0),
        (-0.12, 0.28),
        (0.5, -0.5),
        (-0.28, -0.04),
        (0.12, 0.12),
    ] {
        cands.push([
            cx.add(&span.mul_f64(fx).to_bf(p), p, RM),
            cy.add(&span.mul_f64(fy).to_bf(p), p, RM),
        ]);
    }
    let oracle: Vec<u32> = cands
        .iter()
        .map(|c| orbit_length_bf(&zero, &zero, &c[0], &c[1], formula, max_iter, p))
        .collect();
    // Reference = the shortest ESCAPING candidate, so others outlive it and the
    // reference-exhausted rebase + the trust budget are both exercised.
    let (ri, _) = oracle
        .iter()
        .enumerate()
        .filter(|(_, &l)| l < max_iter)
        .min_by_key(|(_, &l)| l)
        .expect("degenerate fixture: no candidate escapes — pick another coordinate");
    assert!(
        oracle.iter().any(|&l| l > oracle[ri]),
        "degenerate fixture: nothing outlives the reference — rebase path untested"
    );
    let (rlen, orbit) = orbit_length_bf_recorded(
        &zero, &zero, &cands[ri][0], &cands[ri][1], formula, max_iter, p,
    );
    assert_eq!(rlen, oracle[ri], "recording must not change the walk");
    let (mut trusted, mut flagged) = (0u32, 0u32);
    for (i, c) in cands.iter().enumerate() {
        let dc = CFloatExp {
            re: bf_to_floatexp(&c[0].sub(&cands[ri][0], p, RM)),
            im: bf_to_floatexp(&c[1].sub(&cands[ri][1], p, RM)),
        };
        let mut rebases = 0;
        match perturb_orbit_length(&orbit, dc, power, max_iter, &mut rebases) {
            Some(got) => {
                trusted += 1;
                assert_eq!(
                    got, oracle[i],
                    "candidate {i} (power {power}): trusted perturb={got} oracle={} rebases={rebases}",
                    oracle[i]
                );
            }
            None => flagged += 1,
        }
    }
    assert!(
        trusted >= min_trusted && flagged >= min_flagged,
        "fixture regime shifted (power {power}): trusted={trusted} (want ≥{min_trusted})              flagged={flagged} (want ≥{min_flagged})"
    );
}

/// Power 2, at the deep seahorse boundary coordinate the selftest goldens use.
#[test]
fn scorer_matches_oracle_mandelbrot() {
    scorer_matches_oracle_at(
        formula::MANDELBROT,
        "-7.219621882920463979621343199249635039400777157391994056859e-1",
        "2.406540627640154659873781066416545013133592385797331352286e-1",
        6, // tight spread: every score trusted and exact, rebases included
        0,
    );
}

/// Power 3 — the binomial arm the Mandelbrot ladder never runs.
#[test]
fn scorer_matches_oracle_multibrot3() {
    scorer_matches_oracle_at(
        formula::MULTIBROT3,
        "2.19533102209775940218788168856401426185991366731348781648e-1",
        "7.317770073659198278104833118192370226116695264984596408352e-1",
        1, // the reference itself, at least
        1, // wide spread: the trust budget must actually fire
    );
}

/// Power 5 — the widest binomial row.
#[test]
fn scorer_matches_oracle_multibrot5() {
    scorer_matches_oracle_at(
        formula::MULTIBROT5,
        "2.320768669674853369085651557338865001525750889159483426277e-1",
        "7.735895565582844849904484291320284693154748744446630197764e-1",
        1,
        1,
    );
}

/// The acceptance property end to end, on the tight-spread fixture: both phase-2 engines
/// elect the SAME point, the perturbation engine actually scored someone (an early-return
/// fixture could never go red), and the production auto-resolution takes the new engine at
/// this depth and elects the same point. The committed depth ladder (e17 → e4000) is
/// `--pickcheck`; this is its fast always-on rung.
#[test]
fn both_engines_elect_the_same_reference() {
    let p = 164;
    let cx = parse_bf_prec("-7.219621882920463979621343199249635039400777157391994056859e-1", p)
        .unwrap();
    let cy = parse_bf_prec("2.406540627640154659873781066416545013133592385797331352286e-1", p)
        .unwrap();
    let span_y = FloatExp::from_f64(1.0e-30);
    let span = [span_y.mul_f64(16.0 / 9.0), span_y];
    let dual =
        best_reference_dual(&[cx.clone(), cy.clone()], span, 0, false, [0.0, 0.0], 30_000, p)
            .expect("mandelbrot is perturb-eligible");
    assert!(
        dual.identical,
        "engines disagree: walk len={} vs perturb len={}",
        dual.walk_diag.winner_len, dual.perturb_diag.winner_len
    );
    assert!(dual.perturb_diag.deep_perturb, "perturb engine must actually have run");
    assert!(
        dual.perturb_diag.deep_scored > 0,
        "degenerate fixture: nothing beyond the first survivor was scored — pick another"
    );
    let (auto_point, auto_diag) =
        best_reference_diag(&[cx, cy], span, 0, false, [0.0, 0.0], 30_000, p);
    assert!(auto_diag.deep_perturb, "auto gate must engage at a 1e-30 span");
    let eq = |a: &BigFloat, b: &BigFloat| a.cmp(b).is_some_and(|o| o == 0);
    assert!(
        eq(&auto_point[0], &dual.perturb[0]) && eq(&auto_point[1], &dual.perturb[1]),
        "auto resolution must elect the perturb engine's point"
    );
}

/// The auto gate's other side: a shallow span keeps the walk engine, and so does a
/// non-polynomial formula at any depth. (Forcing can widen the span gate for the harness,
/// but never eligibility.)
#[test]
fn shallow_views_and_other_formulas_keep_the_walk() {
    let p = 104;
    let cx = bf(-0.743643887037151, p);
    let cy = bf(0.131825904205330, p);
    // Span 2^-40 — shallower than the 2^-44 auto floor.
    let shallow = FloatExp::new(1.0, -40);
    let (_, diag) = best_reference_diag(
        &[cx.clone(), cy.clone()],
        [shallow, shallow],
        0,
        false,
        [0.0, 0.0],
        8_000,
        p,
    );
    assert!(!diag.deep_perturb, "2^-40 span must keep the bignum walk");
    // Tricorn (non-holomorphic) at a deep span: eligibility, not the span, decides.
    let deep = FloatExp::new(1.0, -100);
    let (_, diag) = best_reference_diag(
        &[cx.clone(), cy.clone()],
        [deep, deep],
        formula::TRICORN,
        false,
        [0.0, 0.0],
        8_000,
        p,
    );
    assert!(!diag.deep_perturb, "tricorn must keep the bignum walk");
    assert!(
        best_reference_dual(&[cx, cy], [deep, deep], formula::TRICORN, false, [0.0, 0.0], 8_000, p)
            .is_err(),
        "the dual harness must refuse (SKIP), not vacuously pass, an ineligible formula"
    );
}
