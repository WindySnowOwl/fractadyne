use super::usable_sa_skip;

#[test]
fn a_skip_at_or_past_the_budget_is_refused_not_clamped() {
    // The 2026-08-25 field case: budget dropped under a cached reference's skip.
    assert_eq!(usable_sa_skip(266_796, 224_000), 0, "the black-frame case");
    assert_eq!(usable_sa_skip(224_000, 224_000), 0, "equal is already past: iter >= max_iter");
    // Anything that genuinely leaves iterations to run is passed through untouched.
    assert_eq!(usable_sa_skip(223_999, 224_000), 223_999);
    assert_eq!(usable_sa_skip(0, 224_000), 0);
    assert_eq!(usable_sa_skip(37_494, 205_343), 37_494, "an ordinary deep frame is unaffected");
}

#[test]
fn a_giant_skip_refused_by_the_throttle_forces_an_unbounded_from_zero_window() {
    // ⛔The 2026-09-11 device loss (crash-1789092955-0) at the period-3 bulb root — a PARABOLIC
    // exact point (beta.96's "Exact points" menu). The critical orbit never escapes, so the
    // reference pins at the ~7.45M device cap and SA computes a skip over the WHOLE orbit. The live
    // frame-cost throttle then lowered THIS dispatch's budget below that skip, and `usable_sa_skip`
    // REFUSED it (correct in isolation — a skip past the budget breaks the shader loop on entry and
    // renders the frame black). But a refused skip seeds the shader at iter 0, so it grinds
    // `[0, budget)` per pixel; at a parabolic point that is the 1.86e10-step, 33-second frame that
    // tripped the driver watchdog.
    //
    // `--deviceloss-repro` could NOT reproduce it: with a fixed `max_iter == ref_iter` the budget
    // stays ABOVE the skip, SA applies (`sa_skip=7452443`), and the render is trivial. Only the
    // throttle — by inverting budget and skip — turns a valid skip into a from-zero grind
    // (`sa_skip=0 (raw 7452443)` in the crash manifest).
    const RAW_SKIP: u32 = 7_452_443; // reference at the cap; the whole orbit is skippable
    const THROTTLED: u32 = 2_843_648; // the crash's per-dispatch shader_iter (manifest gpu_iter)

    // The inversion — and that the THROTTLE is its trigger: leave the budget at/above the skip (the
    // `--deviceloss-repro` case) and the same skip applies, skipping ~7.45M iterations rather than
    // grinding them.
    assert_eq!(usable_sa_skip(RAW_SKIP, THROTTLED), 0, "skip >= throttled budget ⇒ refused");
    assert_eq!(usable_sa_skip(RAW_SKIP, 10_000_000), RAW_SKIP, "budget above the skip ⇒ applied");
    assert!(THROTTLED < RAW_SKIP, "inversion condition: the throttle put the budget below the skip");

    // The lethal consequence a fix must remove: the refused skip leaves the shader running a
    // from-ZERO window equal to the FULL budget — millions of iterations issued as ONE dispatch,
    // not a bounded first window. (`--deviceloss-repro`: cost is ~linear in the iteration window,
    // and a from-zero window this large at field resolution is watchdog-crossing.) A fix must chunk
    // the from-zero fallback (or keep the budget >= the skip so SA stays applied); either way this
    // window must stop being issued whole, at which point this assertion is updated to the bound.
    let from_zero_window = if usable_sa_skip(RAW_SKIP, THROTTLED) == 0 { THROTTLED } else { 0 };
    assert!(
        from_zero_window > 1_000_000,
        "the refused-skip fallback issues a {from_zero_window}-iteration from-zero window in one \
         dispatch — the unbounded first dispatch that must be chunked"
    );
}
