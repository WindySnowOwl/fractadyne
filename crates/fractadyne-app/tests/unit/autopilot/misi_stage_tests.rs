//! The staged Misiurewicz solve (`MisiSolve`): its depth schedule, and how its progress reads.

use super::*;

const FIRST: f64 = crate::tunables::MISI_FIRST_STAGE_OCT_DEFAULT;

#[test]
fn the_stages_start_past_the_view_and_double_to_the_target() {
    // The user's dive: from 2^1198.7 to the 1e5000x limit.
    let st = misi_stages(1198.68, 16_609.64, FIRST);
    assert_eq!(st[0], 1198.68 + FIRST);
    assert_eq!(*st.last().unwrap(), 16_609.64);
    for w in st.windows(2) {
        assert!(w[1] > w[0], "not deeper: {st:?}");
        assert!(w[1] <= 2.0 * w[0] + 1e-9 || w[1] == 16_609.64, "more than doubled: {st:?}");
    }
    assert!(st.len() <= 6, "{} stages: {st:?}", st.len());
}

#[test]
fn a_target_near_the_view_is_one_stage() {
    assert_eq!(misi_stages(100.0, 200.0, FIRST), vec![200.0]);
    assert_eq!(misi_stages(100.0, 100.0 + FIRST, FIRST), vec![100.0 + FIRST]);
}

#[test]
fn a_shallow_view_still_grows_each_stage_by_the_first_step() {
    // Doubling 2^10 is no progress worth a stage: each stage is at least FIRST deeper.
    let st = misi_stages(10.0, 2_000.0, FIRST);
    assert_eq!(st[0], 10.0 + FIRST);
    for w in st.windows(2) {
        assert!(w[1] - w[0] >= FIRST - 1e-9 || w[1] == 2_000.0, "{st:?}");
    }
}

#[test]
fn progress_reads_as_a_percentage_of_the_phase() {
    use fractadyne_core::SolveProgress as P;
    assert_eq!(misi_percent(None), 0);
    assert_eq!(misi_percent(Some(P::Detect { done: 500, of: 2_000 })), 25);
    assert_eq!(misi_percent(Some(P::Newton { step: 3, done: 130_000, of: 260_000, last_l2: None, tol_l2: -100.0 })), 50);
    assert_eq!(misi_percent(Some(P::Verify { done: 260_000, of: 260_000 })), 100);
    assert_eq!(misi_percent(Some(P::Detect { done: 5, of: 0 })), 100, "an empty phase is done, not divided by zero");
    assert_eq!(misi_step(Some(P::Newton { step: 3, done: 0, of: 1, last_l2: None, tol_l2: 0.0 })), 3);
    assert_eq!(misi_step(Some(P::Detect { done: 0, of: 1 })), 1);
}

#[test]
fn a_depth_reads_as_a_power_of_ten() {
    assert_eq!(misi_depth(16_609.64), "1e5000×");
    assert_eq!(misi_depth(1_328.77), "1e400×");
}

#[test]
fn dropping_the_solve_cancels_its_worker() {
    let (_tx, rx) = std::sync::mpsc::channel();
    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let solve = MisiSolve {
        rx,
        started: std::time::Instant::now(),
        toast_t: 0.0,
        cancel: cancel.clone(),
        progress: Default::default(),
        waiting: None,
    };
    assert!(!cancel.load(std::sync::atomic::Ordering::Relaxed));
    drop(solve);
    assert!(cancel.load(std::sync::atomic::Ordering::Relaxed), "a dropped solve left its worker running");
}

#[test]
fn a_small_first_stage_is_honoured() {
    let st = misi_stages(1000.0, 5000.0, 16.0);
    assert_eq!(st[0], 1016.0);
    assert_eq!(st[1], 2032.0);
    assert_eq!(*st.last().unwrap(), 5000.0);
}
