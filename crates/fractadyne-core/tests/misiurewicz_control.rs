//! A long Misiurewicz detect or solve reports its progress and stops when asked (`SolveControl`),
//! and a solve done in STAGES, each seeded by the last, lands on the same point as one done at
//! the final depth from the start (the autopilot solves this way, so a deep dive starts in seconds
//! instead of minutes).
//!
//! The location is the 2.77e89× dendrite of `misiurewicz_outcomes.rs`, whose detected point solves
//! once the view is widened to 1e60×.

use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use fractadyne_core::{MisiurewiczMiss, SolveControl, SolveProgress, SolveScale};

const CX: &str = "2.336541879936817878966215410838761608133707885474547567506859621515201675599324293839217233261500414218604921872973e-2";
const CY: &str = "8.2741173753632652070456275296875144057156928752279993918651650950463440957323066581594346138419917471379151772617142e-1";

fn center(p: usize) -> [fractadyne_core::BigFloat; 2] {
    [
        fractadyne_core::parse_bf_prec(CX, p).expect("cx parses"),
        fractadyne_core::parse_bf_prec(CY, p).expect("cy parses"),
    ]
}

fn pair() -> (u32, u32) {
    let c = center(362);
    fractadyne_core::detect_misiurewicz(&c[0], &c[1], 0, 20_000, 1_024, 362).expect("a pair at the dendrite")
}

/// log2 of the view the seed is judged against (1e60×, wide enough to hold the point).
fn seed_l2() -> f64 {
    1.0e60f64.log2()
}

#[test]
fn a_cancelled_detect_or_solve_stops() {
    let stop = AtomicBool::new(true);
    let ctl = SolveControl { cancel: Some(&stop), progress: None };
    let c = center(362);
    assert_eq!(fractadyne_core::detect_misiurewicz_at_scale_with(&c[0], &c[1], 0, 20_000, 1_024, 362, None, &ctl), None);
    let (k, per) = pair();
    let scale = SolveScale { log2_seed: seed_l2(), log2_target: 600.0 };
    assert_eq!(
        fractadyne_core::find_misiurewicz_with(&c, k, per, scale, 0, &ctl).err(),
        Some(MisiurewiczMiss::Cancelled)
    );
}

#[test]
fn a_solve_reports_each_step_and_the_check() {
    let seen: Mutex<Vec<SolveProgress>> = Mutex::new(Vec::new());
    let note = |p: SolveProgress| seen.lock().unwrap().push(p);
    let ctl = SolveControl { cancel: None, progress: Some(&note) };
    let (k, per) = pair();
    let c = center(362);
    let scale = SolveScale { log2_seed: seed_l2(), log2_target: 600.0 };
    let m = fractadyne_core::find_misiurewicz_with(&c, k, per, scale, 0, &ctl).expect("solves");
    assert_eq!((m.preperiod, m.period), (k, per));
    let seen = seen.into_inner().unwrap();
    // Every step starts at iteration 0 and ends with the whole orbit and its correction.
    let total = (k + per) as u64;
    let ends: Vec<_> = seen
        .iter()
        .filter_map(|p| match *p {
            SolveProgress::Newton { step, done, of, last_l2, tol_l2 } if done == of => Some((step, of, last_l2, tol_l2)),
            _ => None,
        })
        .collect();
    assert!(!ends.is_empty(), "no step was reported finished: {seen:?}");
    for (i, &(step, of, last, _)) in ends.iter().enumerate() {
        assert_eq!(step, i as u32 + 1);
        assert_eq!(of, total);
        assert!(last.is_some());
    }
    // The last correction is below the tolerance (that is what converged means).
    let (_, _, last, tol) = *ends.last().unwrap();
    assert!(last.unwrap() < tol);
    assert!(seen.iter().any(|p| matches!(p, SolveProgress::Verify { .. })), "the check was not reported");
}

#[test]
fn a_staged_solve_lands_on_the_same_point_in_fewer_final_steps() {
    let (k, per) = pair();
    let c = center(362);
    let target = 1_200.0;
    let steps = |seed: &[fractadyne_core::BigFloat; 2], log2_target: f64| {
        let last = Mutex::new(0u32);
        let note = |p: SolveProgress| {
            if let SolveProgress::Newton { step, .. } = p {
                *last.lock().unwrap() = step;
            }
        };
        let ctl = SolveControl { cancel: None, progress: Some(&note) };
        let m = fractadyne_core::find_misiurewicz_with(seed, k, per, SolveScale { log2_seed: seed_l2(), log2_target }, 0, &ctl)
            .expect("solves");
        (m, last.into_inner().unwrap())
    };
    let (direct, direct_steps) = steps(&c, target);
    let (first, _) = steps(&c, 300.0);
    let (second, second_steps) = steps(&[first.cx.clone(), first.cy.clone()], 600.0);
    let (staged, staged_steps) = steps(&[second.cx.clone(), second.cy.clone()], target);
    // The same point, to within the final stage's tolerance (a billionth of a 2^-1200 view): their
    // difference, scaled up by 2^(1200 + 20), is still below one.
    let p = fractadyne_core::precision_for_octaves(target as u64);
    let shift = -(target as i32 + 20);
    let dx = fractadyne_core::ref_offset_mantissa(&staged.cx, &direct.cx, shift, p);
    let dy = fractadyne_core::ref_offset_mantissa(&staged.cy, &direct.cy, shift, p);
    assert!(dx.abs() < 1.0 && dy.abs() < 1.0, "staged and direct differ by ({dx}, {dy}) x 2^{shift}");
    // Seeded by the stage before, the final precision needs fewer Newton steps than from the view.
    assert!(
        staged_steps < direct_steps && second_steps < direct_steps,
        "steps at the final precision: staged {staged_steps}, direct {direct_steps} (second stage {second_steps})"
    );
}
