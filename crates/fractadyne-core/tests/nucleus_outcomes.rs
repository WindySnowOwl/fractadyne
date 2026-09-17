//! `find_nucleus` past the f64 ceiling — the conversion `find_misiurewicz` got (2026-09-02).
//!
//! The finder took a LINEAR `f64` magnification, so it stopped at ~1e305×: `span = 3/mag`
//! underflowed, the Newton tolerance and the runaway rejection silently degenerated, and
//! `reduce_period`'s linear `tol2` hit exactly `0.0` — every genuine nucleus then failed the
//! period verification and the finder answered `None`. Half the Go-to dialog reached e60000
//! (the Misiurewicz half) and half did not.

use fractadyne_core as fc;

/// A magnification `f64` cannot express (`2^6644 ≈ 1e2000`), where every linear quantity the
/// old finder computed reads `0.0` or `inf`.
const DEEP_L2: f64 = 6_644.0;

/// ⭐The pattern the backlog item prescribes: assert the DEPTH WAS REACHED, not that a solve
/// returned — a centre short of the requested precision looks exactly like a good one. The
/// oracle is the Newton-step magnitude at the target precision (first-order distance to the
/// true root): the deep solve must sit within its own 2^-6644 view, while the shallow answer —
/// the best the old ceiling could produce — is thousands of view-widths outside it.
///
/// Doubles as the `reduce_period` pin the item asks for: at this depth the old linear `tol2`
/// was exactly `0.0`, so a nucleus could not verify at all — the REDUCED period 3 coming back
/// is the fixed behaviour, not an incidental detail.
#[test]
fn a_nucleus_solve_reaches_a_depth_f64_cannot_express() {
    // Stage 1 — shallow: the period-3 island's nucleus from a 1e6× view beside it.
    let seed = [fc::BigFloat::from_f64(-0.12256, 128), fc::BigFloat::from_f64(0.74486, 128)];
    let shallow = fc::find_nucleus(&seed, 1.0e6f64.log2(), 0, 64)
        .expect("the period-3 island nucleus is found at 1e6x");
    assert_eq!(shallow.period, 3, "the island beside (-0.12256, 0.74486) has period 3");

    // Stage 2 — deepen the known nucleus to target precision (`refine_nucleus`, the same
    // two-stage flow `--find-minibrot` uses), then run the CONVERTED finder at the deep view
    // seeded there. The refine in between is not a dodge, it is the contract: `find_nucleus`
    // finds the minibrot near a VIEW (seed within ~8 view-widths — a user at e2000 can see
    // the minibrot they ask about), and a shallow-accurate centre is trillions of deep
    // view-widths off — the runaway rejection SHOULD fire on it (and, converted to log2,
    // now actually does; the old linear test compared 0.0 > 0.0 and let anything through).
    let p = fc::precision_for_octaves(DEEP_L2 as u64);
    let (rx, ry) = fc::refine_nucleus(&shallow.cx, &shallow.cy, 3, 0, p)
        .expect("refine the shallow nucleus to deep precision");
    let deep = fc::find_nucleus(&[rx, ry], DEEP_L2, 0, 64)
        .expect("the finder must not decline past the old ~1e305x ceiling");
    assert_eq!(deep.period, 3, "the reduced (fundamental) period survives at depth");

    let res_deep = fc::nucleus_residual_log2(&deep.cx, &deep.cy, 3, 0, p)
        .expect("residual at the deep centre");
    let res_shallow = fc::nucleus_residual_log2(&shallow.cx, &shallow.cy, 3, 0, p)
        .expect("residual at the shallow centre");
    assert!(
        res_deep < -6_000.0,
        "deep solve must be accurate AT THE TARGET SCALE (Newton step 2^{res_deep:.0}; \
         the view is 2^-6644)"
    );
    assert!(
        res_shallow > -400.0,
        "the shallow answer is only view-accurate (~2^-40s); reading 2^{res_shallow:.0} means \
         this test's oracle is broken, not that the old ceiling was fine"
    );
    assert!(
        res_shallow - res_deep > 5_000.0,
        "the deep solve must add real digits over the shallow one \
         (2^{res_deep:.0} vs 2^{res_shallow:.0})"
    );
}

/// ⭐⭐**The period and the centre must describe the SAME atom.** `reduce_period` used to scan
/// every `n ≤ p_est` for the first one within tolerance of the solved point, but the only thing
/// that licenses the reduction is `Z_m = 0 ⟹ Z_{jm} = 0`, which runs one way — so only a DIVISOR
/// of the detected period can be the true period. Scanning everything asked "is any lower-period
/// nucleus nearby", and near a deep atom something always is: at the seahorse seed viewed at 1e6×
/// the detector correctly said 998, Newton converged on the 998 nucleus, and the reduction handed
/// back **973** — a real neighbouring minibrot's period attached to the 998 minibrot's centre. A
/// jump there lands ~500,000 atom-widths into empty space.
///
/// The tolerance is a thousandth of the VIEW span, so the failure got worse as the view widened
/// (39 at 1e1.8×, 468 at 1e3.9×, 858 at 1e4.8×). This test therefore sweeps the view rather than
/// pinning one magnification: the answer must not depend on how far out the user was standing.
/// `moved` is the oracle that would have caught it — re-refining a correct answer at high
/// precision barely moves it, while the 973 answer moved 1.8e-9 against an atom 3.4e-15 wide.
#[test]
fn the_reduced_period_and_the_centre_describe_the_same_atom() {
    // The classic deep seahorse-valley point. The minibrot here has period 998 and its atom is
    // 2^-50.5 wide — far below every view magnification swept, which is the whole point.
    let seed = [
        fc::parse_bf("-0.743643887037151").unwrap(),
        fc::parse_bf("0.131825904205330").unwrap(),
    ];
    for &l2 in &[6.0f64, 13.0, 16.0, 1.0e6f64.log2(), 22.0, 30.0, 40.0] {
        let n = fc::find_nucleus(&seed, l2, 0, 100_000)
            .unwrap_or_else(|| panic!("seahorse nucleus must be found at 2^{l2}"));
        assert_eq!(
            n.period, 998,
            "the seahorse minibrot has period 998; 2^{l2} reported {} — the reduction picked a \
             neighbouring atom's period",
            n.period
        );

        let atom = fc::nucleus_size(&n.cx, &n.cy, n.period, 0, 128).expect("atom size");
        assert!(
            (atom.log2_size + 50.5).abs() < 0.5,
            "the period-998 atom is 2^-50.5 wide; 2^{l2} sized it 2^{:.3}, so the centre and the \
             period do not describe the same atom",
            atom.log2_size
        );

        // The centre must be accurate far below the atom's own width, or the jump this feature
        // exists to make lands on empty space. Re-refining at the atom's precision is the oracle:
        // a correct centre barely moves.
        let p = fc::precision_for_octaves(atom.log2_size.abs().ceil() as u64) + 64;
        let (rx, ry) = fc::refine_nucleus(&n.cx, &n.cy, n.period, 0, p).expect("refine");
        let moved = (fc::sub_f64(&rx, &n.cx, p).powi(2) + fc::sub_f64(&ry, &n.cy, p).powi(2)).sqrt();
        assert!(
            moved < atom.log2_size.exp2() * 1.0e-3,
            "2^{l2}: refining moved the centre {moved:.2e}, which is {:.0} atom-widths — the \
             solve did not converge at the scale its own answer lives at",
            moved / atom.log2_size.exp2()
        );
    }
}

/// The conversion must not move shallow results: the same solve at 1e6× still lands on the
/// catalog-class nucleus with the fundamental period, and re-solving from the nucleus itself
/// converges in place (the runaway rejection — now in log2 — must not misfire on a perfect
/// seed).
#[test]
fn shallow_behaviour_is_unchanged_and_a_perfect_seed_converges_in_place() {
    let seed = [fc::BigFloat::from_f64(-0.12256, 128), fc::BigFloat::from_f64(0.74486, 128)];
    let n = fc::find_nucleus(&seed, 1.0e6f64.log2(), 0, 64).expect("nucleus at 1e6x");
    assert_eq!(n.period, 3);

    let again = fc::find_nucleus(&[n.cx.clone(), n.cy.clone()], 1.0e6f64.log2(), 0, 64)
        .expect("re-solving from the nucleus itself must not be rejected as a runaway");
    assert_eq!(again.period, 3);
}
