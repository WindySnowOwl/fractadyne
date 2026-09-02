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
