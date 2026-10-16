use super::*;

#[test]
fn a_starved_view_fails_unless_the_app_names_the_limit() {
    // Black on both sides at the ask, resolved at 4×: the U34 blind spot.
    assert_eq!(grade(true, 0.0, Some(35.0), 0.0, false), Grade::Starved);
    assert_eq!(grade(true, 0.0, Some(35.0), 0.0, true), Grade::Capped);
    assert!(Grade::Starved.fails() && !Grade::Capped.fails());
    // Short of the fail line it warns, reported or not.
    assert_eq!(grade(true, 0.0, Some(5.0), 0.0, true), Grade::Warn);
    assert_eq!(grade(true, 0.0, Some(5.0), 0.0, false), Grade::Warn);
    // The live path's own shortfall outranks starvation, and an explanation does not excuse it.
    assert_eq!(grade(true, 12.0, Some(50.0), 0.0, true), Grade::Fail);
    // Nothing judged without the oracle.
    assert_eq!(grade(false, 50.0, Some(50.0), 99.0, false), Grade::Unjudged);
}

#[test]
fn without_the_second_oracle_the_grade_is_the_old_verdict() {
    // The pre-W8 rule: ≥10 points FAIL, ≥3 points or ≥24 sRGB warn, else ok — the words a
    // blessed baseline records, so an old baseline still compares.
    for (excess, srgb, want) in [
        (10.0, 0.0, "FAIL"),
        (9.99, 0.0, "warn"),
        (3.0, 0.0, "warn"),
        (2.9, 23.9, "ok"),
        (0.0, 24.0, "warn"),
        (-5.0, 0.0, "ok"),
    ] {
        assert_eq!(grade(true, excess, None, srgb, false).word(), want, "excess {excess} sRGB {srgb}");
    }
    assert_eq!(grade(false, 0.0, None, 0.0, false).word(), "----");
}

#[test]
fn the_status_bar_labels_the_grade_reads_are_the_ones_it_shows() {
    // From `limit_status_matches_measured_regimes`' measured regimes: if a label is renamed,
    // `reports_the_limit` must follow it, or every honest report grades STARVED.
    const DEV: u32 = 7_452_444;
    const MAX: u32 = crate::MAX_ITER_LIMIT;
    let label = |s: Option<(&'static str, String, bool)>| s.map(|s| s.0);
    let depth = label(FractadyneApp::limit_status(true, DEV, DEV, MAX, None, 0, false, None, false, false));
    let capped = label(FractadyneApp::limit_status(
        false, 900_000, DEV, MAX, Some(0.60), MAX, true, Some(MAX as f64 * 0.97), false, false,
    ));
    let exhausted = label(FractadyneApp::limit_status(
        false, 3_631_055, DEV, MAX, Some(1.0), MAX, true, None, true, true,
    ));
    assert!(depth.is_some() && capped.is_some() && exhausted.is_some());
    assert!(reports_the_limit(depth), "{depth:?}");
    assert!(reports_the_limit(capped), "{capped:?}");
    // Exhausted concludes interior — the claim a 4× render can contradict — so it never excuses.
    assert!(!reports_the_limit(exhausted), "{exhausted:?}");
    assert!(!reports_the_limit(None));
}

#[test]
fn a_false_alarm_is_an_iter_capped_report_that_four_times_the_ask_does_not_bear_out() {
    let capped = Some("⚠ iter capped");
    // The grand tour's home view: 9.42% black at 2,000, 9.41% at 8,000.
    assert!(false_alarm(capped, 0.0942, Some(0.01)));
    // The 3.3e61 spar at an exact 60,000: 100% black, 0.1% at 4× — the report is right.
    assert!(!false_alarm(capped, 1.0, Some(99.9)));
    // No second oracle, no judgement; and only "iter capped" makes the claim.
    assert!(!false_alarm(capped, 0.0942, None));
    assert!(!false_alarm(Some("⚠ depth limit"), 0.0942, Some(0.01)));
    assert!(!false_alarm(None, 0.0942, Some(0.01)));
}

#[test]
fn the_escape_range_reads_escaped_pixels_only() {
    let px = |v: &[f32]| v.iter().flat_map(|&x| [x, 0.0, 0.0, 1.0]).collect::<Vec<f32>>();
    assert_eq!(escape_range(&px(&[-1.0, -1.0])), None);
    assert_eq!(escape_range(&px(&[-1.0, 40.5, 12.25, -1.0, 3000.0])), Some((12.25, 3000.0)));
    assert_eq!(interior_frac(&px(&[-1.0, 40.5, 12.25, -1.0])), 0.5);
}
