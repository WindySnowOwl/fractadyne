//! `arg_center` reads a `--center` coordinate at the precision of the view it is for.
//!
//! It used the parser's natural precision for the digits given (~64 bits for a short decimal)
//! at any depth. The π points (scripts/pi_points.py, 2026-11-03) found it: `0.25000000000001` at
//! 2^71.8 sat about 40 pixels from itself, and the cusp's escape count moved by 5.
use super::arg_center;
use fractadyne_core::{bf_sub, parse_real_expr, BigFloat};

/// log2 of how far `got` lies from the exact decimal `exact` (−∞ when it is exact).
fn off_log2(got: &BigFloat, exact: &str) -> f64 {
    let want = parse_real_expr(exact, 20_000).expect("the exact value parses");
    let d = bf_sub(got, &want, 20_000);
    if d.is_zero() {
        return f64::NEG_INFINITY;
    }
    f64::from(d.exponent().expect("a non-zero difference has an exponent"))
}

#[test]
fn a_short_decimal_is_read_at_the_views_precision() {
    // A pixel of a 1080-row view at 2^l is 4·2^-l/1080 ≈ 2^-(l+8). Hold the coordinate to 2^-40 of
    // that: 2^-(l+48).
    for &(text, l) in &[("0.25000000000001", 71.8), ("-0.1", 3000.0), ("0.7071", 200.0)] {
        let (cx, cy) = arg_center("--center", text, "0", l);
        let off = off_log2(&cx, text);
        assert!(off < -(l + 48.0), "{text} at 2^{l}: off by 2^{off}, wants below 2^{}", -(l + 48.0));
        assert!(cy.is_zero(), "the imaginary 0 stays 0");
        // The CONTROL: the same text read for a 1x view keeps its natural precision and lands
        // measurably further off - so this test can see the fault it guards.
        let (c1, _) = arg_center("--center", text, "0", 0.0);
        assert!(off_log2(&c1, text) > -(l + 48.0), "{text}: the 1x control is as precise, so the test is blind");
    }
}

#[test]
fn a_long_decimal_keeps_every_digit_it_was_given() {
    // A centre pasted with more digits than the zoom needs is not rounded down to the zoom.
    let text = format!("-1.{}", "7".repeat(400)); // ~1,330 bits of digits
    let (cx, _) = arg_center("--center", &text, "0", 10.0);
    assert!(off_log2(&cx, &text) < -1300.0);
}
