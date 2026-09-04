use super::{budget_step, probe_would_price, PRICE_REPRESENTATIVE_FRAC};

/// The measured idle case (2026-09-04, default home view): a native-resolution frame costing
/// 4.288e8 steps against the 9e8 bootstrap budget. It is 0.48 of the budget, so its reading
/// would be discarded — and manufacturing it is what dispatched the GPU every 3 frames forever.
#[test]
fn the_idle_home_view_probe_is_declined() {
    assert!(!probe_would_price(428_779_008, 900_000_000));
    // …and the discard it would have run into is the same rule, from the same constant.
    assert_eq!(budget_step(900_000_000, 428_779_008, 20.0, false), None);
}

/// The regime the probe EXISTS for: a view floored by a budget too small to render it. The frame
/// is budget-sized by construction there, so the guard must not block it — blocking this is the
/// "pixellated forever" deadlock the probe was written to break.
#[test]
fn a_budget_sized_floored_frame_still_probes() {
    let cur = 900_000_000u64;
    assert!(probe_would_price(cur, cur), "a frame AT budget must probe");
    assert!(probe_would_price((cur as f64 * 0.71) as u64, cur));
}

/// The threshold is one shared value, and the boundary belongs to the priced side.
#[test]
fn the_boundary_is_the_shared_threshold() {
    let cur = 1_000_000u64;
    let at = (cur as f64 * PRICE_REPRESENTATIVE_FRAC) as u64;
    assert!(probe_would_price(at, cur), "exactly at the threshold prices");
    assert!(!probe_would_price(at - 1, cur));
}

/// A zero budget must not divide-by-zero or lock the probe out (it is the pre-bootstrap state).
#[test]
fn a_zero_budget_always_probes() {
    assert!(probe_would_price(0, 0));
    assert!(probe_would_price(1, 0));
}

/// Naming the constant must not have moved the behaviour of the pricing rule: an UNDERSIZED but
/// SLOW dispatch is still kept (it is the strongest evidence per-step cost has collapsed), which
/// is the one-sided exception the discard test has always carried.
#[test]
fn an_undersized_but_slow_dispatch_is_still_read() {
    assert!(!probe_would_price(1_000, 900_000_000));
    assert!(budget_step(900_000_000, 1_000, 5_000.0, false).is_some());
}
