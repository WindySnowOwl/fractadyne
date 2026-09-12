use super::*;

#[test]
fn openings_are_bounded_by_the_serial_floor_and_tighten_on_worse_evidence() {
    let mut p = ChunkPricer::new();
    assert_eq!(p.open(4_000_000), 400_000, "1M it/s floor x 400 ms target");
    p.observe(400_000, 800.0); // twice as slow as assumed
    assert_eq!(p.open(4_000_000), 200_000);
    p.observe(400_000, 8.0); // a cheap chunk must never re-widen the opening
    assert_eq!(p.open(4_000_000), 200_000);
    assert_eq!(p.open(50_000), 50_000, "never past the ask");
}

#[test]
fn windows_halve_hot_double_cheap_and_hold_the_band() {
    let p = ChunkPricer::new();
    assert_eq!(p.next(400_000, 800.0, 4_000_000), 200_000);
    assert_eq!(p.next(400_000, 20.0, 4_000_000), 800_000);
    assert_eq!(p.next(400_000, 250.0, 4_000_000), 400_000);
    // A fresh pricer (worst rate = the 1M it/s floor) has not seen a hot location, so the soft
    // floor still holds: halving 20k lands under 16,384 and is floored back up.
    assert_eq!(p.next(20_000, 5000.0, 4_000_000), CHUNK_MIN_ITERS, "floor holds in the normal regime");
    assert_eq!(p.next(3_000_000, 20.0, 4_000_000), 4_000_000, "ask caps growth");
    assert_eq!(p.next(400_000, f64::NAN, 4_000_000), 400_000);
}

/// The device-loss-2026-09-12 regression: at a deep interior view a single 16,384-iter window
/// (the old hard floor) cost ~1,750 ms — 4.4× the 400 ms target and past the watchdog — and the
/// floor pinned it there every pass. Once the pricer has OBSERVED that serial rate, the floor must
/// yield below 16,384 so the window lands near the hot budget instead. `run_tile` always calls
/// `observe` before `open`/`next`, so the worst rate is current when the floor is computed.
#[test]
fn a_hot_interior_shrinks_the_window_below_the_soft_floor() {
    let mut p = ChunkPricer::new();
    // The field measurement: 16,384 iters took ~1,750 ms.
    p.observe(16_384, 1750.0);
    let rate = 1750.0 / 16_384.0; // ms per iter, now the worst-seen high-water mark

    // The opening window now targets the hot budget, not the lethal floor.
    let w = p.open(5_223_168);
    assert!(w < CHUNK_MIN_ITERS, "opening window {w} did not drop below the soft floor");
    assert!(w >= CHUNK_ABS_MIN, "opening window {w} broke the hard floor");
    let predicted_ms = w as f64 * rate;
    assert!(
        predicted_ms <= CHUNK_HOT_MS * 1.05,
        "predicted chunk wall {predicted_ms:.0} ms is not within the {CHUNK_HOT_MS} ms budget"
    );

    // And a hot chunk at that window now shrinks instead of being floored back to 16,384.
    let n = p.next(w, predicted_ms.max(CHUNK_HOT_MS + 1.0), 5_223_168);
    assert!(n < CHUNK_MIN_ITERS, "hot next() window {n} was floored back into the lethal band");
    assert!(n >= CHUNK_ABS_MIN);
}

/// The hard floor is never breached, however pathological the rate — so the pass count stays
/// bounded even where nothing but a resolution cut could keep a dispatch under the watchdog.
#[test]
fn the_hard_floor_bounds_the_window_at_any_rate() {
    let mut p = ChunkPricer::new();
    p.observe(1, 1_000_000.0); // absurd: 1 ms per iter, 1000× the field case
    assert_eq!(p.open(5_000_000), CHUNK_ABS_MIN, "opening ignored the hard floor");
    assert_eq!(p.next(CHUNK_ABS_MIN, 9_999.0, 5_000_000), CHUNK_ABS_MIN, "next() broke the hard floor");
}
