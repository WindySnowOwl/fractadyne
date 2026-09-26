use super::regime_report;
use crate::diag::frame_record::{kind, FrameRecord};

/// The 2026-09-21 field frame: a 655-sample ESCAPED reference against a 4,627-iteration ask, BLA
/// off, rebasing with no BLA skips, dispatched in one piece.
fn field_frame() -> FrameRecord {
    FrameRecord {
        kind: kind::FRAME,
        has_ref: true,
        ref_partial: false,
        ref_len: 655,
        gpu_iter: 4627,
        bla_on: false,
        ctr_new: true,
        ctr_rebase: 33_000_000,
        ctr_bla_skip: 0,
        dispatched: true,
        chunked: false,
        fe_budget: 151_500_000_000,
        ..Default::default()
    }
}

#[test]
fn the_field_frame_is_the_regime() {
    let r = regime_report(&[field_frame()]);
    assert!(r.entered, "{}", r.line);
    assert!(r.line.starts_with("ENTERED"), "{}", r.line);
    assert!(r.line.contains("1 of 1 dispatches UN-chunked"), "{}", r.line);
}

/// Each term of the predicate, removed on its own, must take the frame out of the regime — a
/// predicate with a term that never matters is a gate that cannot go red on that term.
#[test]
fn every_term_of_the_predicate_is_load_bearing() {
    let cases: [(&str, FrameRecord); 5] = [
        ("partial reference", FrameRecord { ref_partial: true, ..field_frame() }),
        ("ask under 4x the reference", FrameRecord { gpu_iter: 655 * 4 - 1, ..field_frame() }),
        ("BLA on", FrameRecord { bla_on: true, ..field_frame() }),
        ("no reference", FrameRecord { has_ref: false, ..field_frame() }),
        ("a stall row", FrameRecord { kind: kind::STALL, ..field_frame() }),
    ];
    for (why, rec) in cases {
        assert!(!regime_report(&[rec]).entered, "{why} must not count as entering the regime");
    }
}

/// The shape without the storm (no counter reading, or one with BLA skips / no rebases) is
/// reported as SHAPE ONLY and does not count as entered.
#[test]
fn the_shape_without_a_counted_storm_is_not_entered() {
    for rec in [
        FrameRecord { ctr_new: false, ..field_frame() },
        FrameRecord { ctr_rebase: 0, ..field_frame() },
        FrameRecord { ctr_bla_skip: 5, ..field_frame() },
    ] {
        let r = regime_report(&[rec]);
        assert!(!r.entered, "{}", r.line);
        assert!(r.line.starts_with("SHAPE ONLY"), "{}", r.line);
    }
    let none = regime_report(&[]);
    assert!(!none.entered && none.line.starts_with("NOT ENTERED"), "{}", none.line);
}

/// The RX 6800 XT run of 2026-09-23: the storm in the first frames, then thousands of settled
/// frames on the same short reference with no counter readings and no dispatches. Folded over the
/// whole run it is ENTERED; read from only the last `RING_LEN` records — what the soak used to do —
/// the same run is SHAPE ONLY, which is the false "VACUOUS" that run printed.
#[test]
fn a_storm_early_in_a_long_run_still_counts() {
    use crate::diag::frame_record::RING_LEN;
    let storm = FrameRecord { frame: 13, t_ms: 1_899, ..field_frame() };
    let settled = FrameRecord { ctr_new: false, dispatched: false, ..field_frame() };
    let mut run = vec![storm];
    run.extend(std::iter::repeat_n(settled, RING_LEN + 4_000));

    let whole = regime_report(&run);
    assert!(whole.entered, "{}", whole.line);
    assert!(whole.line.contains("first storm reading at frame 13 (+1.9s)"), "{}", whole.line);

    let tail = regime_report(&run[run.len() - RING_LEN..]);
    assert!(!tail.entered && tail.line.starts_with("SHAPE ONLY"), "{}", tail.line);
}

/// Records the ring overwrote before the soak read them are named, never silently absent.
#[test]
fn records_lost_to_the_ring_are_named() {
    let mut acc = super::RegimeAcc::default();
    acc.add(&field_frame());
    acc.lost = 7;
    let r = acc.report();
    assert!(r.entered, "{}", r.line);
    assert!(r.line.contains("7 record(s) overwritten before they were read"), "{}", r.line);
}

/// A slow frame (> 200 ms with a repaint requested) at budget `tdr`, at `t_ms`, on view 0.
fn slow(frame: u64, t_ms: u64, tdr: u64) -> FrameRecord {
    FrameRecord {
        kind: kind::FRAME,
        frame,
        t_ms,
        last_dt_ms: 400.0,
        repaint_requested: true,
        tdr_steps: tdr,
        ..Default::default()
    }
}

fn stall_of(recs: &[FrameRecord]) -> String {
    let mut acc = super::RegimeAcc::default();
    for r in recs {
        acc.add(r);
    }
    acc.stall_line()
}

/// The 2026-09-21 field loss: 20 slow frames over 32.8 s, the budget at 1.515e11 on every one.
#[test]
fn the_field_budget_stall_is_one_run() {
    let recs: Vec<FrameRecord> =
        (0..20).map(|i| slow(45_322 + i * 10, 825_900 + i * 1_726, 151_500_000_000)).collect();
    let line = stall_of(&recs);
    assert!(line.contains("20 slow frame(s) over 32.8 s from frame 45322 (+825.9s) at 1.515e11"), "{line}");
}

/// The still rung (RX 6800 XT, 2026-09-23/25): the budget came down at the first reading and again
/// at the next, so no run of slow frames at an unmoving budget is longer than four.
#[test]
fn a_budget_that_comes_down_ends_the_run() {
    let recs = [
        slow(14, 4_194, 151_500_000_000),
        slow(15, 5_223, 11_821_295_753),
        slow(16, 6_307, 11_821_295_753),
        slow(19, 6_805, 11_821_295_753),
        slow(20, 7_258, 2_064_775_407),
        slow(21, 7_657, 2_064_775_407),
        slow(22, 7_922, 2_064_775_407),
        slow(23, 8_230, 2_064_775_407),
    ];
    let line = stall_of(&recs);
    assert!(line.contains("4 slow frame(s) over 1.0 s from frame 20"), "{line}");
}

/// A budget that RISES between slow frames has not come down: the run continues.
#[test]
fn a_rising_budget_does_not_end_the_run() {
    let line = stall_of(&[slow(1, 1_000, 100), slow(2, 2_000, 200), slow(3, 3_000, 200)]);
    assert!(line.contains("3 slow frame(s) over 2.0 s"), "{line}");
}

/// eframe's ~1 Hz idle tick reads as a one-second frame; with no repaint requested it measures
/// nothing about cost and must not count (the discriminator behind two earlier wrong diagnoses).
#[test]
fn an_idle_tick_is_not_a_slow_frame() {
    let tick = FrameRecord { repaint_requested: false, ..slow(1, 1_000, 5) };
    let fast = FrameRecord { last_dt_ms: 150.0, ..slow(2, 2_000, 5) };
    let line = stall_of(&[tick, fast]);
    assert!(line.starts_with("no slow frame"), "{line}");
}

/// Slow frames more than ten seconds apart are separate episodes, however flat the budget.
#[test]
fn a_long_gap_splits_the_run() {
    let line = stall_of(&[slow(1, 1_000, 9), slow(2, 2_000, 9), slow(3, 12_001, 9)]);
    assert!(line.contains("2 slow frame(s) over 1.0 s from frame 1"), "{line}");
}

/// Timing readings that arrived INSIDE the run are counted — the field's question is whether the
/// controller was told and did not act, or was never told. One after the last slow frame is not.
#[test]
fn readings_inside_the_run_are_counted() {
    let reading = |frame, t_ms| FrameRecord { kind: kind::FRAME, frame, t_ms, read_n: 1, ..Default::default() };
    let recs = [slow(1, 1_000, 9), reading(2, 1_500), reading(3, 1_600), slow(4, 2_000, 9), reading(5, 2_500)];
    let line = stall_of(&recs);
    assert!(line.contains("2 slow frame(s)") && line.contains("2 GPU timing reading(s) inside it"), "{line}");
}

/// The two views are tracked apart: view 1's slow frames neither extend nor end view 0's run.
#[test]
fn the_views_are_tracked_apart() {
    let v1 = |frame, t_ms, tdr| FrameRecord { view: 1, ..slow(frame, t_ms, tdr) };
    let recs = [slow(1, 1_000, 9), v1(1, 1_000, 1), slow(2, 2_000, 9), v1(2, 2_000, 1), slow(3, 3_000, 9)];
    let line = stall_of(&recs);
    assert!(line.contains("3 slow frame(s) over 2.0 s") && line.contains("view 0"), "{line}");
}
