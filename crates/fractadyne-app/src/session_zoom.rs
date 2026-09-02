use super::restored_units_per_pixel;

#[test]
fn a_corrupted_zoom_is_refused() {
    // The field case: whatever produced it, a non-finite zoom must not reach the viewport.
    assert!(restored_units_per_pixel(f64::NAN, -9).is_none());
    assert!(restored_units_per_pixel(f64::INFINITY, -9).is_none());
    assert!(restored_units_per_pixel(f64::NEG_INFINITY, -9).is_none());
    // Zero or negative pixel size is not a view, it is a division waiting to happen.
    assert!(restored_units_per_pixel(0.0, -9).is_none());
    assert!(restored_units_per_pixel(-1.44, -9).is_none());
    // An exponent orders beyond any real depth is corruption.
    assert!(restored_units_per_pixel(1.44, i32::MIN).is_none());
    assert!(restored_units_per_pixel(1.44, i32::MAX).is_none());
}

#[test]
fn real_sessions_still_load() {
    // The actual value on disk after this incident was resolved (zoom ~1.06x).
    assert!(restored_units_per_pixel(1.449069213429062, -9).is_some());
    // And depths across the range the app genuinely reaches, including past the df32 crossover
    // and out to the documented e2100 wall — none of these may be mistaken for corruption.
    for e in [0, -30, -100, -1000, -3700, -7000] {
        assert!(restored_units_per_pixel(1.5, e).is_some(), "legitimate depth 2^{e} refused");
    }
}
