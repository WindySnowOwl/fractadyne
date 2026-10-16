//! Settle-tile serialization: `settle_tile_waits` and the per-view completion record it reads
//! (`Perf::view_reg_pending`, `view_outstanding`, `view_last_done`).

use super::*;
use crate::tunables::{SETTLE_COMPLETION_TIMEOUT_US, SETTLE_INFLIGHT_MAX};

/// The RX 6800 XT budget trap's first tile.
const TILE: u64 = 8_062_084_800;

fn done(price_ms: u64, steps: u64) -> ViewDone {
    ViewDone { seq: 1, price_us: price_ms * 1000, steps }
}

#[test]
fn an_idle_view_never_holds_its_tile() {
    assert!(!settle_tile_waits(0, ViewDone::default(), TILE, 0));
    assert!(!settle_tile_waits(0, done(900, TILE), TILE, 0));
}

#[test]
fn two_outstanding_always_hold() {
    assert!(settle_tile_waits(SETTLE_INFLIGHT_MAX, done(5, TILE), TILE, 0));
    assert!(settle_tile_waits(SETTLE_INFLIGHT_MAX + 3, done(5, TILE), TILE, 0));
}

#[test]
fn a_cheap_tile_pipelines_behind_one_cheap_predecessor() {
    // The RTX 3080 stripe settle: same-size tiles completing in ~two frames.
    assert!(!settle_tile_waits(1, done(35, 4_000_000_000), 4_000_000_000, 20_000));
}

#[test]
fn a_tile_whose_predecessor_ran_slow_waits() {
    assert!(settle_tile_waits(1, done(443, TILE), 5_582_382_960, 20_000));
}

#[test]
fn a_fast_but_small_predecessor_is_no_evidence_for_a_big_tile() {
    // The trap: the grid's first tile followed a 1.7e6-step motion pass that completed quickly.
    assert!(settle_tile_waits(1, done(20, 1_747_584), TILE, 20_000));
}

#[test]
fn with_nothing_completed_yet_one_outstanding_holds() {
    assert!(settle_tile_waits(1, ViewDone::default(), TILE, 20_000));
}

#[test]
fn a_completion_that_never_comes_releases_after_the_timeout() {
    assert!(settle_tile_waits(SETTLE_INFLIGHT_MAX, done(900, TILE), TILE, SETTLE_COMPLETION_TIMEOUT_US));
    assert!(!settle_tile_waits(SETTLE_INFLIGHT_MAX, done(900, TILE), TILE, SETTLE_COMPLETION_TIMEOUT_US + 1));
}

#[test]
fn the_timeout_is_past_the_watchdog_and_any_tile_seen_in_the_field() {
    // Windows' TDR is 2 s; the RX 6800 XT ran settle tiles to ~0.8 s (budget trap, 2026-10-12).
    const { assert!(SETTLE_COMPLETION_TIMEOUT_US > 2_000_000) };
}

#[test]
fn outstanding_work_is_counted_from_registration_until_its_callback() {
    let mut p = crate::Perf::default();
    assert_eq!(p.view_outstanding(0), 0);
    // `update` registers each frame's owed completion under the next sequence…
    p.view_reg_seq[0] = 2;
    assert_eq!(p.view_outstanding(0), 2);
    assert_eq!(p.view_outstanding(1), 0, "the other view's queue is its own");
    // …and the callback records the one that completed (they fire in order).
    *p.view_done[0].lock().unwrap() = ViewDone { seq: 1, price_us: 30_000, steps: TILE };
    assert_eq!(p.view_outstanding(0), 1);
    assert_eq!(p.view_last_done(0).price_us, 30_000);
    p.view_done[0].lock().unwrap().seq = 2;
    assert_eq!(p.view_outstanding(0), 0);
}

#[test]
fn a_synchronous_harness_retires_everything_it_registered() {
    let mut p = crate::Perf::default();
    p.view_reg_seq[0] = 3;
    p.view_reg_pending[0] = true;
    p.retire_synchronous_dispatches(0);
    assert_eq!(p.view_outstanding(0), 0);
    assert!(!p.view_reg_pending[0]);
}
