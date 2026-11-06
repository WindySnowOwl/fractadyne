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

// ---- The stages' worker loop (`run_misi_stages`) and the dive's gate (`misi_gate`), with a
// stand-in for Newton: the failure paths, which a real location does not reach on demand.

/// What a run of the stages sent, and the seeds each stage was solved from (as `f64` x).
fn run_with(
    stages: &[f64],
    mut answer: impl FnMut(u32) -> Result<(), fractadyne_core::MisiurewiczMiss>,
) -> (Vec<MisiMsg>, Vec<f64>, MisiProgress) {
    let bf = |x: f64| fractadyne_core::parse_bf_prec(&x.to_string(), 128).expect("parses");
    let center = [bf(-0.1), bf(0.65)];
    let (tx, rx) = std::sync::mpsc::channel();
    let progress = std::sync::Mutex::new(MisiProgress::default());
    let mut seeds = Vec::new();
    let mut stage = 0;
    run_misi_stages(center, (12, 3), 100.0, stages, &progress, &tx, std::time::Instant::now(), |seed, _scale| {
        stage += 1;
        let x = fractadyne_core::to_f64(&seed[0]);
        seeds.push(x);
        answer(stage).map(|()| fractadyne_core::Misiurewicz {
            preperiod: 12,
            period: 3,
            // Each stage moves the point a little, so the next stage's seed can be told apart.
            cx: bf(x + 1e-6),
            cy: seed[1].clone(),
        })
    });
    drop(tx);
    (rx.iter().collect(), seeds, progress.into_inner().unwrap())
}

#[test]
fn a_later_stage_that_fails_sends_the_stages_before_it_and_then_the_failure() {
    let stages = [356.0, 712.0, 1424.0];
    let (sent, seeds, progress) =
        run_with(&stages, |s| if s == 2 { Err(fractadyne_core::MisiurewiczMiss::NotConverged) } else { Ok(()) });
    assert_eq!(seeds.len(), 2, "a stage after the failure was solved");
    assert!((seeds[1] - (seeds[0] + 1e-6)).abs() < 1e-12, "stage 2 was not seeded by stage 1: {seeds:?}");
    assert_eq!(sent.len(), 2);
    match &sent[0] {
        MisiMsg::Stage { target, stage: 1, stages: 3, .. } => {
            assert_eq!(target.solved_l2, 356.0);
            assert!(!target.complete, "the first of three stages is not the last");
        }
        _ => panic!("first message is not stage 1"),
    }
    assert!(matches!(sent[1], MisiMsg::Failed { why: MisiMiss::NotSolved { preperiod: 12, period: 3 }, stage: 2 }));
    assert_eq!((progress.stage, progress.stage_l2), (2, 712.0), "progress names the stage that failed");
}

#[test]
fn a_first_stage_that_fails_sends_only_the_failure() {
    let (sent, _, _) = run_with(&[356.0, 712.0], |_| Err(fractadyne_core::MisiurewiczMiss::TooFar { log2_view_widths: 3.5 }));
    assert_eq!(sent.len(), 1);
    assert!(matches!(sent[0], MisiMsg::Failed { why: MisiMiss::OffScreen { preperiod: 12, period: 3, .. }, stage: 1 }));
}

#[test]
fn a_cancelled_stage_sends_nothing_more() {
    let (sent, seeds, _) =
        run_with(&[356.0, 712.0, 1424.0], |s| if s == 2 { Err(fractadyne_core::MisiurewiczMiss::Cancelled) } else { Ok(()) });
    assert_eq!(seeds.len(), 2);
    assert_eq!(sent.len(), 1, "a canceled solve reported more than its first stage");
    assert!(matches!(sent[0], MisiMsg::Stage { stage: 1, .. }));
}

#[test]
fn every_stage_solved_marks_only_the_last_complete() {
    let (sent, seeds, _) = run_with(&[356.0, 712.0, 1424.0], |_| Ok(()));
    assert_eq!(seeds.len(), 3);
    let complete: Vec<(u32, bool)> = sent
        .iter()
        .map(|m| match m {
            MisiMsg::Stage { target, stage, .. } => (*stage, target.complete),
            _ => panic!("a failure among solved stages"),
        })
        .collect();
    assert_eq!(complete, vec![(1, false), (2, false), (3, true)]);
}

#[test]
fn a_solve_the_dive_dropped_stops_after_the_stage_it_is_on() {
    let bf = |x: f64| fractadyne_core::parse_bf_prec(&x.to_string(), 128).expect("parses");
    let (tx, rx) = std::sync::mpsc::channel();
    drop(rx);
    let progress = std::sync::Mutex::new(MisiProgress::default());
    let mut solved = 0;
    run_misi_stages([bf(-0.1), bf(0.65)], (12, 3), 100.0, &[356.0, 712.0], &progress, &tx, std::time::Instant::now(), |seed, _| {
        solved += 1;
        Ok(fractadyne_core::Misiurewicz { preperiod: 12, period: 3, cx: seed[0].clone(), cy: seed[1].clone() })
    });
    assert_eq!(solved, 1, "the worker kept solving for a dive that had dropped it");
}

#[test]
fn the_dive_holds_for_a_running_refinement_and_stops_when_none_will_come() {
    // Short of the solved depth: dive on, whatever the refinement is doing.
    assert_eq!(misi_gate(300.0, 356.0, false, 16_609.6, true), MisiGate::Dive);
    assert_eq!(misi_gate(300.0, 356.0, false, 16_609.6, false), MisiGate::Dive);
    // At it, with the next stage running: hold.
    assert_eq!(misi_gate(356.0, 356.0, false, 16_609.6, true), MisiGate::Hold);
    // At it, with no stage running and the last one not reached: a later stage failed.
    assert_eq!(misi_gate(357.2, 356.0, false, 16_609.6, false), MisiGate::Stop(MisiStop::RefineFailed));
    // Solved to the limit: the dive's own limit stops it there.
    assert_eq!(misi_gate(16_609.6, 16_609.6, true, 16_609.6, false), MisiGate::Dive);
    // The limit lowered since: the same.
    assert_eq!(misi_gate(5_000.0, 16_609.6, true, 3_000.0, false), MisiGate::Dive);
    // Raised since: every stage solved, to the old limit; stop there, and say why.
    assert_eq!(misi_gate(1_330.0, 1_328.8, true, 16_609.6, false), MisiGate::Stop(MisiStop::LimitRaised));
}
