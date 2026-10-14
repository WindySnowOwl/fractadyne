//! `--deviceloss-repro`'s criterion (design §7.3 gate 3).

use super::*;

fn d(chunked: bool, tiled: bool, window: u32, px: u64) -> LiveDispatch {
    LiveDispatch { chunked, tiled, window, px }
}

#[test]
fn the_model_is_linear_in_window_and_area_when_the_exponents_are_one() {
    let m = CostModel { a: 1.0, b: 1.0, winter: (0.01f64).ln(), safe_px: 1000.0 };
    // 0.01 ms per iteration at the safe area: 256 iterations = 2.56 ms; twice the area doubles it.
    assert!((m.predict_ms(256.0, 1000.0) - 2.56).abs() < 1e-9);
    assert!((m.predict_ms(256.0, 2000.0) - 5.12).abs() < 1e-9);
}

#[test]
fn an_unchunked_lethal_dispatch_is_red_and_a_chunked_one_is_not() {
    let arms = [("stale", d(false, false, 1_000_000, 2_000_000), 5_000.0)];
    assert_eq!(repro_criterion(&arms, 900.0).len(), 1);
    // The same predicted cost on a CHUNKED frame: its passes are the window actuator's business.
    let arms = [("stale", d(true, false, 1_000_000, 2_000_000), 5_000.0)];
    assert!(repro_criterion(&arms, 900.0).is_empty());
    // An unchunked frame under the band passes.
    let arms = [("fresh", d(false, true, 4_627, 50_000), 120.0)];
    assert!(repro_criterion(&arms, 900.0).is_empty());
}
