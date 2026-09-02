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
