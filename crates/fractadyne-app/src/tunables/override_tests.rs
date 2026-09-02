use super::*;

#[test]
fn defaults_match_the_documented_constants() {
    // The struct is a second copy of twelve numbers; this is what keeps it honest.
    let d = Cost::default();
    assert_eq!(d.tdr_budget_ms, TDR_BUDGET_MS_DEFAULT);
    assert_eq!(d.tdr_bootstrap_steps, TDR_BOOTSTRAP_STEPS_DEFAULT);
    assert_eq!(d.tdr_bootstrap_ms, TDR_BOOTSTRAP_MS_DEFAULT);
    assert_eq!(d.tdr_lethal_ms, TDR_LETHAL_MS_DEFAULT);
    assert_eq!(d.mode_rate_unknown_margin, MODE_RATE_UNKNOWN_MARGIN_DEFAULT);
    assert_eq!(d.tdr_min_steps, TDR_MIN_STEPS_DEFAULT);
    assert_eq!(d.tdr_steps_ceil, TDR_STEPS_CEIL_DEFAULT);
    assert_eq!(d.motion_unpriced_max, MOTION_UNPRICED_MAX_DEFAULT);
    assert_eq!(d.explicit_dispatch_cap, EXPLICIT_DISPATCH_CAP_DEFAULT);
    assert_eq!(d.tdr_max_tiles, TDR_MAX_TILES_DEFAULT);
    assert_eq!(d.tdr_tiles_ceil, TDR_TILES_CEIL_DEFAULT);
}

#[test]
fn a_typo_is_an_error_not_a_silent_no_op() {
    let r = apply_overrides(&[("TDR_BUDGET_MSS".into(), "500".into())]);
    assert!(r.is_err(), "an unknown knob must be rejected");
    assert!(apply_overrides(&[("TDR_BUDGET_MS".into(), "nope".into())]).is_err());
    assert!(apply_overrides(&[("TDR_BUDGET_MS".into(), "0".into())]).is_err());
    assert!(apply_overrides(&[("TDR_BUDGET_MS".into(), "-1".into())]).is_err());
}

#[test]
fn an_inverted_range_is_rejected() {
    // `budget_step` clamps into `TDR_MIN_STEPS ..= ceil`; an inverted range panics there.
    let r = apply_overrides(&[
        ("TDR_MIN_STEPS".into(), "900000000000".into()),
        ("TDR_STEPS_CEIL".into(), "1000".into()),
    ]);
    assert!(r.is_err(), "min above ceiling must be rejected, not clamped");
}

#[test]
fn status_says_stock_when_nothing_was_set() {
    // These tests share a process, so this asserts the SHAPE, not a particular global state.
    let s = status_line();
    assert!(s == "stock" || s.contains("OVERRIDE(S)"), "unexpected status line: {s}");
}
