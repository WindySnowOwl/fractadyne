use super::{curve_span, norm_map_transfer, window_phase, NormShape};
use fractadyne_core::coloring::Transfer;

#[test]
fn a_skewed_range_takes_the_log_map_and_a_narrow_high_band_does_not() {
    let is_log = |mn: f32, mx: f32, t: Transfer| norm_map_transfer(mn, mx, t) == Transfer::Log;
    // Every range here was MEASURED off a real view via FRACTADYNE_TRACE=gpu.
    // 9.8e27x deep field — narrow, high: the case linear normalization was built for.
    assert!(!is_log(45075.0, 63736.0, Transfer::Linear), "deep field must stay linear");
    // 19.88x with a 350,000 iteration budget — the "cities at night" report.
    assert!(is_log(1.0, 7925.0, Transfer::Linear), "heavy-tailed range must take the log map");
    // Home with a 250,000 ceiling: skewed but well under the ratio, and it does not alias
    // anyway (the caller's phase test refuses to engage there at all).
    assert!(!is_log(1.0, 232.0, Transfer::Linear));
    // The user's choice always wins.
    assert!(is_log(45075.0, 63736.0, Transfer::Log));
    // A zero/tiny floor must not make every range look infinitely skewed.
    assert!(!is_log(0.0, 900.0, Transfer::Linear));
    assert!(is_log(0.0, 1001.0, Transfer::Linear));
}

/// A curve the user chose (C2) is the map's whatever the range: only Linear switches to log.
#[test]
fn a_chosen_curve_is_kept_whatever_the_range() {
    for t in Transfer::ALL.into_iter().filter(|t| *t != Transfer::Linear) {
        assert_eq!(norm_map_transfer(1.0, 7925.0, t), t);
        assert_eq!(norm_map_transfer(45075.0, 63736.0, t), t);
    }
}

/// ⭐Under every curve the window holds the same sweeps: its floor at phase 0, its top at the
/// sweeps, rising between; and log's span is the one the map always had, bit for bit.
#[test]
fn every_curve_spans_the_window_with_the_same_sweeps() {
    let w = (1000.0_f32, 5000.0_f32);
    for t in Transfer::ALL {
        let s = NormShape { sweeps: 2.5, transfer: t };
        assert!(window_phase(w, w.0, s).abs() < 1.0e-5, "{t:?} floor");
        assert!((window_phase(w, w.1, s) - 2.5).abs() < 1.0e-4, "{t:?} top: {}", window_phase(w, w.1, s));
        let mid = window_phase(w, 3000.0, s);
        assert!(mid > 0.0 && mid < 2.5, "{t:?} middle {mid}");
    }
    // Where the bands fall differs: the roots and logs put the middle value past half the sweeps
    // (wider bands far from the set, lower values), Square before it.
    let mid = |t| window_phase(w, 3000.0, NormShape { sweeps: 2.0, transfer: t });
    assert!(mid(Transfer::Sqrt) > 1.0 && mid(Transfer::Log) > mid(Transfer::Sqrt) && mid(Transfer::Square) < 1.0);
    assert_eq!(curve_span(4000.0, Transfer::Log).to_bits(), (4000.0_f32 + 1.0).ln().to_bits());
    assert!((curve_span(4000.0, Transfer::Sqrt) - 4000f32.sqrt()).abs() < 1.0e-3);
}
