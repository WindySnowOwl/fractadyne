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
    assert_eq!(p.next(20_000, 5000.0, 4_000_000), CHUNK_MIN_ITERS, "floor holds");
    assert_eq!(p.next(3_000_000, 20.0, 4_000_000), 4_000_000, "ask caps growth");
    assert_eq!(p.next(400_000, f64::NAN, 4_000_000), 400_000);
}
