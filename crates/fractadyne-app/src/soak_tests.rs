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
