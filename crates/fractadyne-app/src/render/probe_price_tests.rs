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

/// The RX 6800 XT's tap-zoom (2026-09-27): a 4.56e7 budget and moving-frame passes of 1.2e6-2.2e7
/// steps, each discarded alone. Pooled, they reach 0.7x the budget and price as one reading - which
/// `budget_step` then accepts - and the pool empties.
#[test]
fn discarded_readings_pool_until_representative() {
    use super::{budget_step, pool_step, ReadingPool};
    let cur = 45_600_000u64;
    let passes = [(1_930_000u64, 0.15), (5_600_000, 0.24), (22_400_000, 0.31), (4_530_000, 1.10)];
    for (s, ms) in passes {
        assert!(budget_step(cur, s, ms, false).is_none(), "each alone is discarded");
    }
    let mut pool = ReadingPool::default();
    let mut emitted = None;
    for (i, (s, ms)) in passes.iter().cycle().take(12).enumerate() {
        if let Some(out) = pool_step(&mut pool, *s, *ms, cur, 100 + i as u64, u64::MAX) {
            emitted = Some(out);
            break;
        }
    }
    let (steps, ms, n) = emitted.expect("the pool becomes representative");
    assert!(steps as f64 >= 0.7 * cur as f64, "{steps}");
    assert!(n >= 2, "{n}");
    assert_eq!(pool, ReadingPool::default(), "emptied once priced");
    assert!(budget_step(cur, steps, ms, false).is_some(), "the pooled reading is priceable");
}

/// A pool never mixes regimes: one older than POOL_MAX_FRAMES, or begun before the view's last
/// mode switch, starts over with the new reading.
#[test]
fn a_stale_or_pre_switch_pool_starts_over() {
    use super::{pool_step, ReadingPool, POOL_MAX_FRAMES};
    let cur = 1_000_000_000u64;
    let mut pool = ReadingPool::default();
    assert!(pool_step(&mut pool, 10_000_000, 1.0, cur, 100, u64::MAX).is_none());
    assert_eq!(pool.n, 1);
    // Too old: restarts with only the new reading.
    assert!(pool_step(&mut pool, 10_000_000, 1.0, cur, 100 + POOL_MAX_FRAMES + 1, u64::MAX).is_none());
    assert_eq!((pool.n, pool.steps, pool.since_frame), (1, 10_000_000, 100 + POOL_MAX_FRAMES + 1));
    // A mode switch after the pool began: restarts too.
    let mut pool = ReadingPool { steps: 5_000_000, ms: 0.5, n: 3, since_frame: 200 };
    assert!(pool_step(&mut pool, 1_000_000, 0.1, cur, 210, 205).is_none());
    assert_eq!((pool.n, pool.since_frame), (1, 210));
    // A garbage reading is not pooled.
    let before = pool;
    assert!(pool_step(&mut pool, 1_000, f64::NAN, cur, 211, 205).is_none());
    assert!(pool_step(&mut pool, 0, 1.0, cur, 211, 205).is_none());
    assert_eq!(pool, before);
}
