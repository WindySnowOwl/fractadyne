use super::norm_map_is_log;

#[test]
fn a_skewed_range_takes_the_log_map_and_a_narrow_high_band_does_not() {
    // Every range here was MEASURED off a real view via FRACTADYNE_TRACE=gpu.
    // 9.8e27x deep field — narrow, high: the case linear normalization was built for.
    assert!(!norm_map_is_log(45075.0, 63736.0, false), "deep field must stay linear");
    // 19.88x with a 350,000 iteration budget — the "cities at night" report.
    assert!(norm_map_is_log(1.0, 7925.0, false), "heavy-tailed range must take the log map");
    // Home with a 250,000 ceiling: skewed but well under the ratio, and it does not alias
    // anyway (the caller's phase test refuses to engage there at all).
    assert!(!norm_map_is_log(1.0, 232.0, false));
    // The checkbox always wins.
    assert!(norm_map_is_log(45075.0, 63736.0, true));
    // A zero/tiny floor must not make every range look infinitely skewed.
    assert!(!norm_map_is_log(0.0, 900.0, false));
    assert!(norm_map_is_log(0.0, 1001.0, false));
}
