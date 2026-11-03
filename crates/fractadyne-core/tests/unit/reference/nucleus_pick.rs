use super::*;

// The period-128 minibrot of bench ladder scene 40 (its nucleus), viewed at 1e12.962x, where the
// minibrot is a third of the view's height (validation/minibrot-offsets/nuclei.csv).
const NRE: &str = "-7.145519235131990488163335631423597550599502471617637408317e-1";
const NIM: &str = "3.540207347648327221719984333721934753758406620526068439482e-1";
const PERIOD: u32 = 128;
const MAG_LOG10: f64 = 12.962;
const CAP: u32 = 30 * PERIOD;
const P: usize = 128;

/// The view's span `[width, height]` at 16:9 (the height is 4 / magnification).
fn span() -> [FloatExp; 2] {
    let h = FloatExp::from_f64(4.0).mul_pow2(-MAG_LOG10 * std::f64::consts::LOG2_10);
    [h.mul_f64(16.0 / 9.0), h]
}

fn nucleus() -> [BigFloat; 2] {
    [parse_bf_prec(NRE, P).unwrap(), parse_bf_prec(NIM, P).unwrap()]
}

/// A view centred so the nucleus sits `fx` half-widths right of the centre and `fy` half-heights up.
fn view(fx: f64, fy: f64) -> [BigFloat; 2] {
    let [w, h] = span();
    let n = nucleus();
    [
        n[0].sub(&w.mul_f64(0.5 * fx).to_bf(P), P, RM),
        n[1].sub(&h.mul_f64(0.5 * fy).to_bf(P), P, RM),
    ]
}

fn log2_span() -> f64 {
    let [w, h] = span();
    w.log2().max(h.log2())
}

/// The walk of a view's centre records the nucleus it passes: a hint at the minibrot's period
/// whose Newton step points at the nucleus, to first order.
#[test]
fn the_walk_records_a_hint_at_the_nucleus_it_passes() {
    let c = view(0.6, 0.0);
    let (_, period, hints) = orbit_length_periodic(&c[0], &c[1], formula::MANDELBROT, CAP, P, log2_span(), None);
    assert!(period.is_none(), "the centre is not the nucleus, so its orbit does not close");
    let h = hints.iter().find(|h| h.period == PERIOD).expect("a hint at the minibrot's period");
    // c - step should be the nucleus: compare the step with c - nucleus.
    let n = nucleus();
    let want = CFloatExp { re: bf_to_floatexp(&c[0].sub(&n[0], P, RM)), im: bf_to_floatexp(&c[1].sub(&n[1], P, RM)) };
    let err = (h.step - want).abs().log2() - want.abs().log2();
    assert!(err < -8.0, "the hint's step is the nucleus offset to first order (relative error 2^{err:.1})");
}

/// The nucleus 0.6 half-widths right of a centre that escapes: the nucleus pick takes it, a
/// periodic reference, where the plain pick takes a survivor.
#[test]
fn the_nucleus_pick_takes_the_nucleus_when_the_centre_escapes() {
    let c = view(0.6, 0.0);
    let zero = bf(0.0, P + REF_RESCUE_EXTRA_BITS);
    let centre_len = orbit_length_bf(&zero, &zero, &c[0], &c[1], formula::MANDELBROT, CAP, P + REF_RESCUE_EXTRA_BITS, false);
    assert!(centre_len < CAP, "fixture: the centre must escape (got {centre_len} of {CAP})");

    let (plain, plain_diag) = best_reference_diag(&c, span(), formula::MANDELBROT, false, [0.0, 0.0], CAP, P);
    assert_eq!(plain_diag.nucleus, None, "the plain pick never takes a nucleus");

    let (got, diag) = best_reference_diag_nucleus(&c, span(), formula::MANDELBROT, false, [0.0, 0.0], CAP, P, None);
    assert_eq!(diag.nucleus, Some(PERIOD), "the nucleus pick takes the period-{PERIOD} nucleus");
    assert!(diag.winner_len >= CAP);
    // It is the nucleus (refined far below the view's span) ...
    let n = nucleus();
    let off = CFloatExp { re: bf_to_floatexp(&got[0].sub(&n[0], P, RM)), im: bf_to_floatexp(&got[1].sub(&n[1], P, RM)) };
    assert!(off.abs().log2() < span()[1].log2() - 40.0, "the pick is the nucleus (off by 2^{:.1})", off.abs().log2());
    // ... not what the plain pick took ...
    assert!(plain[0].cmp(&got[0]).is_some_and(|o| o != 0) || plain[1].cmp(&got[1]).is_some_and(|o| o != 0));
    // ... and its orbit closes, so a build stops at one period.
    let (_, period, _) = orbit_length_periodic(&got[0], &got[1], formula::MANDELBROT, CAP, P + REF_RESCUE_EXTRA_BITS, log2_span(), None);
    assert_eq!(period.map(|p| p.period), Some(PERIOD));
}

/// Whether `pt` lies inside the view centred at `c` (the pick's own rectangle, `±span/2`).
fn inside(c: &[BigFloat; 2], pt: &[BigFloat; 2]) -> bool {
    let [w, h] = span();
    bf_to_floatexp(&pt[0].sub(&c[0], P, RM)).log2() <= w.log2() - 1.0
        && bf_to_floatexp(&pt[1].sub(&c[1], P, RM)).log2() <= h.log2() - 1.0
}

/// The period-128 nucleus just past the right edge (1.2 half-widths) is never taken. The minibrot
/// still reaches into that view, and its period-2 bulb's nucleus (period 256) lies just inside the
/// edge: that one may be taken, because it is in the view and its orbit closes. Whatever is taken
/// must be inside the view.
#[test]
fn a_nucleus_outside_the_view_is_never_taken() {
    let c = view(1.2, 0.0);
    let (got, diag) = best_reference_diag_nucleus(&c, span(), formula::MANDELBROT, false, [0.0, 0.0], CAP, P, None);
    assert_ne!(diag.nucleus, Some(PERIOD), "the out-of-view period-{PERIOD} nucleus is not the reference");
    if diag.nucleus.is_some() {
        assert!(inside(&c, &got), "a nucleus that is taken lies inside the view (took {:?})", diag.nucleus);
    }
}

/// Far from the minibrot (3 half-widths out on the diagonal) no nucleus is in the view: the nucleus
/// pick must be the plain pick exactly.
#[test]
fn with_no_nucleus_in_view_the_pick_is_the_plain_one() {
    let c = view(3.0, 3.0);
    let (plain, plain_diag) = best_reference_diag(&c, span(), formula::MANDELBROT, false, [0.0, 0.0], CAP, P);
    let (got, diag) = best_reference_diag_nucleus(&c, span(), formula::MANDELBROT, false, [0.0, 0.0], CAP, P, None);
    assert_eq!(diag.nucleus, None, "no nucleus is taken (took {:?})", diag.nucleus);
    assert!(
        plain[0].cmp(&got[0]).is_some_and(|o| o == 0) && plain[1].cmp(&got[1]).is_some_and(|o| o == 0),
        "the pick is the plain pick's"
    );
    assert_eq!(diag.winner_len, plain_diag.winner_len);
}

/// Recording hints changes nothing a build returns: the same walk with and without a view that
/// could hold the nucleus gives the same length and period.
#[test]
fn a_centred_view_still_closes_on_its_period() {
    let c = view(0.0, 0.0);
    let (len, period, _) = orbit_length_periodic(&c[0], &c[1], formula::MANDELBROT, CAP, P + REF_RESCUE_EXTRA_BITS, log2_span(), None);
    assert_eq!(period.map(|p| p.period), Some(PERIOD), "the nucleus-centred view closes at its period");
    assert_eq!(len, CAP, "a closing orbit scores the whole budget");
}

/// The repeats rule: a nucleus whose period is longer than the pick allows is not taken, and the
/// same hint is taken once the period fits. (The pick allows a quarter of the centre's own walk,
/// `NUCLEUS_MIN_REPEATS`.)
#[test]
fn a_nucleus_longer_than_the_pick_allows_is_not_taken() {
    let c = view(0.6, 0.0);
    let pd = P + REF_RESCUE_EXTRA_BITS;
    let (_, _, hints) = orbit_length_periodic(&c[0], &c[1], formula::MANDELBROT, CAP, pd, log2_span(), None);
    assert!(nucleus_in_view(&hints, &c, span(), formula::MANDELBROT, PERIOD - 1, pd, log2_span()).is_none());
    let got = nucleus_in_view(&hints, &c, span(), formula::MANDELBROT, PERIOD, pd, log2_span());
    assert_eq!(got.map(|(_, period)| period), Some(PERIOD));
}
