use super::*;

fn detect(re: &str, im: &str, prec: usize, max_iter: u32, max_period: u32) -> Option<(u32, u32)> {
    let cx = crate::parse_bf_prec(re, prec).expect("centre parses");
    let cy = crate::parse_bf_prec(im, prec).expect("centre parses");
    detect_misiurewicz(&cx, &cy, 0, max_iter, max_period, prec)
}

/// Two Misiurewicz points whose orbits can be checked by hand, so a wrong answer here is not a
/// matter of interpretation.
///
/// c = -2:  0 -> -2 -> 2 -> 2 -> ...      lands on the fixed point 2 after 2 steps => (2, 1)
/// c =  i:  0 ->  i -> -1+i -> -i -> -1+i lands on a 2-cycle after 2 steps        => (2, 2)
#[test]
fn the_two_textbook_points_are_identified() {
    assert_eq!(detect("-2.0", "0.0", 128, 64, 32), Some((2, 1)), "antenna tip");
    assert_eq!(detect("0.0", "1.0", 128, 64, 32), Some((2, 2)), "c = i");
}

/// The spiral this was written for: the centre of a 1.6e39x view whose structure is organised
/// around a Misiurewicz point. Independently identified as (49, 3) with mpmath, then Newton
/// refined to 522 digits and RENDERED at 1e500x, where it shows the same spiral rather than
/// noise — which is the check that the pair is right, not merely plausible.
///
/// The separation at this centre is 6.6e-39, so this doubles as the regression pin for the
/// f64 pre-filter: an implementation that ranked candidates in f64 would see every one of them
/// as exactly zero and could return any pair at all.
#[test]
fn a_deep_spiral_centre_resolves_to_its_misiurewicz_pair() {
    let got = detect(
        "-0.088792613303098660153845052309701500569653558627720875436411309366560178250182",
        "0.654809144755247929391298652387765097829565958367438788263142461373782142678715",
        256,
        600,
        64,
    );
    assert_eq!(got, Some((49, 3)), "expected the (49,3) pair the render confirmed");
}

/// An escaping orbit is not pre-periodic, and a point well inside the main cardioid has no
/// near-return to find. Both must decline rather than invent a pair.
#[test]
fn points_without_a_misiurewicz_pair_are_declined() {
    assert_eq!(detect("2.0", "2.0", 128, 64, 32), None, "escapes immediately");
}

/// ⭐The reported pre-period must be the CANONICAL (minimal) one. Once an orbit is pre-periodic
/// at k it is pre-periodic at every k′ > k, all of which Newton-solve to the same point — so
/// before the reduction the k shown to the user was an accident of how many candidates the
/// pre-filter admitted (field measurement at a 283,353× spiral: (95,1) reported for a point
/// whose canonical pre-period is 16, and the reported k rose 33 → 95 → 159 as the pre-filter
/// threshold was loosened 4× → 16× → 64×).
///
/// Seed: a hair inside the antenna tip (−2 + 1e−20 — canonically (2, 1)) with a ~1e−18 view
/// span. The scale-aware ranking rightly prefers a LATE index of the fixed-point tail (the
/// cycle derivative that matches the view span grows along the tail — that is how it finds the
/// feature organizing the view), so without the reduction it reports the entry ~30 steps late.
/// The reduction must walk it back to the true entry, holding the period fixed.
#[test]
fn the_reported_preperiod_is_the_canonical_minimal_one() {
    let p = 192;
    let cx = crate::parse_bf_prec("-1.99999999999999999999", p).expect("centre parses");
    let cy = crate::parse_bf_prec("0.0", p).expect("centre parses");
    let span_log2 = 1.0e-18f64.log2();
    let got = detect_misiurewicz_at_scale(&cx, &cy, 0, 200, 32, p, Some(span_log2));
    assert_eq!(got, Some((2, 1)), "canonical antenna-tip pair, not an inflated tail index");
}
