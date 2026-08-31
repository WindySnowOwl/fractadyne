//! `find_misiurewicz` must say WHY it failed, not just that it did.
//!
//! The distinction is not academic. A user at a 2.77e89× dendrite was told "no point converged
//! near the view — navigate closer" when the solver had in fact converged in two Newton steps
//! onto a genuine Misiurewicz point 4.0e-77 away: a real feature, twelve decades COARSER than the
//! view, reachable by zooming OUT. Collapsing that into `None` gave advice pointing the wrong way.
//!
//! The location below is that user's, kept verbatim so the case stays reproducible.

use fractadyne_core::MisiurewiczMiss;

const CX: &str = "2.336541879936817878966215410838761608133707885474547567506859621515201675599324293839217233261500414218604921872973e-2";
const CY: &str = "8.2741173753632652070456275296875144057156928752279993918651650950463440957323066581594346138419917471379151772617142e-1";

fn center(p: usize) -> [fractadyne_core::BigFloat; 2] {
    [
        fractadyne_core::parse_bf_prec(CX, p).expect("cx parses"),
        fractadyne_core::parse_bf_prec(CY, p).expect("cy parses"),
    ]
}

/// The detector works at this depth — it is the SOLVE that used to look like the failure.
#[test]
fn the_detector_finds_a_pair_at_e89() {
    let p = 362;
    let c = center(p);
    let got = fractadyne_core::detect_misiurewicz(&c[0], &c[1], 0, 20_000, 1_024, p);
    let (k, per) = got.expect("no (k,p) detected at the reported location");
    // Not pinned to the exact pair — a better detector may legitimately find another — but a deep
    // dendrite must give a LARGE preperiod and a small period, which is the shape of the thing.
    assert!(k > 100, "preperiod {k} is too small to describe a feature at 1e89");
    assert!(per > 0 && per < 64, "period {per} is not a plausible cycle length here");
}

/// A real point outside the view is reported as TooFar WITH ITS DISTANCE — the number that says
/// which way to go — and the same point is accepted once the view is wide enough to contain it.
#[test]
fn a_distant_point_is_reported_as_far_not_as_absent() {
    let p = 362;
    let c = center(p);
    let (k, per) = fractadyne_core::detect_misiurewicz(&c[0], &c[1], 0, 20_000, 1_024, p)
        .expect("detect");

    // At the view's own magnification the point is far outside it.
    match fractadyne_core::find_misiurewicz(&c, k, per, 2.7685285297383285e89, 0) {
        Err(MisiurewiczMiss::TooFar { view_widths }) => {
            assert!(
                view_widths > 1_000.0,
                "reported TooFar but only {view_widths} view-widths — that is not the case this \
                 test is about"
            );
        }
        other => panic!("expected TooFar at 2.77e89, got {other:?}"),
    }

    // ...and it is a REAL point: widen the view and the same (k,p) solves. Without this the
    // assertion above would also pass for a solver that had wandered off to nowhere.
    let ok = fractadyne_core::find_misiurewicz(&c, k, per, 1.0e60, 0)
        .expect("the same point must solve when the view is wide enough to hold it");
    assert_eq!((ok.preperiod, ok.period), (k, per));
}

/// The distance reported is REAL INFORMATION about the pair asked for, not a constant.
///
/// ⚠This test began life asserting that the user's hand-typed (5,332) would be rejected as a
/// non-fitting pair. It is not: (5,332) is a perfectly good Misiurewicz pair whose nearest
/// instance simply lies 1.9e56 view-widths from this view. The premise was wrong, not the code —
/// so what it pins now is that two different pairs produce two different distances, which is what
/// makes the number worth showing the user.
#[test]
fn the_reported_distance_is_specific_to_the_pair() {
    let p = 362;
    let c = center(p);
    let mag = 2.7685285297383285e89;
    let far = |k, per| match fractadyne_core::find_misiurewicz(&c, k, per, mag, 0) {
        Err(MisiurewiczMiss::TooFar { view_widths }) => view_widths,
        other => panic!("expected TooFar for ({k},{per}), got {other:?}"),
    };
    let detected = far(437, 3);
    let typed = far(5, 332);
    assert!(detected > 1.0 && typed > 1.0, "{detected} / {typed}");
    // Different pairs, different answers — a report that always said the same thing would carry
    // no information and would still satisfy the two assertions above.
    assert!(
        (detected / typed - 1.0).abs() > 0.5,
        "both pairs reported the same distance ({detected} vs {typed}) — the number is not          telling us anything about the pair"
    );
}

/// Zero preperiod or period is a request error, not a search failure.
#[test]
fn a_zero_pair_is_a_bad_request() {
    let c = center(64);
    assert_eq!(
        fractadyne_core::find_misiurewicz(&c, 0, 3, 1.0e6, 0),
        Err(MisiurewiczMiss::BadRequest)
    );
    assert_eq!(
        fractadyne_core::find_misiurewicz(&c, 3, 0, 1.0e6, 0),
        Err(MisiurewiczMiss::BadRequest)
    );
}

/// ⭐⭐The end of the reported story: detecting at the VIEW'S SCALE finds a pair that actually
/// solves inside the view, where ranking by closest near-return does not.
///
/// This is the difference between the finder being unusable at depth and working:
///   closest separation → (437,3) → TooFar by 3.7e12 view-widths
///   scale-aware        → (901,1) → a point inside the view
#[test]
fn detecting_at_the_view_scale_finds_a_pair_that_solves_here() {
    let p = 362;
    let c = center(p);
    let mag = 2.7685285297383285e89_f64;
    let span_log2 = (3.0f64 / mag).log2();

    // The old ranking: a real point, but nowhere near this view.
    let (k0, p0) = fractadyne_core::detect_misiurewicz(&c[0], &c[1], 0, 20_000, 1_024, p)
        .expect("closest-separation detect");
    assert!(
        matches!(
            fractadyne_core::find_misiurewicz(&c, k0, p0, mag, 0),
            Err(MisiurewiczMiss::TooFar { .. })
        ),
        "the closest-separation pair ({k0},{p0}) was expected to land outside the view"
    );

    // Scale-aware: a pair whose feature is the size of the view, and it solves HERE.
    let (k1, p1) =
        fractadyne_core::detect_misiurewicz_at_scale(&c[0], &c[1], 0, 20_000, 1_024, p, Some(span_log2))
            .expect("scale-aware detect");
    let found = fractadyne_core::find_misiurewicz(&c, k1, p1, mag, 0)
        .expect("the scale-aware pair must solve within the view");
    assert_eq!((found.preperiod, found.period), (k1, p1));
    // ...and it is a DIFFERENT answer from the old one, or the scale test is doing nothing.
    assert_ne!((k0, p0), (k1, p1), "scale-aware selection returned the same pair");
}

/// The scale parameter must be a LOG. A linear width underflows to zero past ~1e308x, which would
/// silently switch the scale test off at exactly the depths it exists for — so `None` and a
/// non-finite value both fall back to the historical ranking rather than half-working.
#[test]
fn a_missing_or_broken_scale_falls_back_cleanly() {
    let p = 362;
    let c = center(p);
    let plain = fractadyne_core::detect_misiurewicz(&c[0], &c[1], 0, 20_000, 1_024, p);
    for bad in [None, Some(f64::NAN), Some(f64::NEG_INFINITY)] {
        let got =
            fractadyne_core::detect_misiurewicz_at_scale(&c[0], &c[1], 0, 20_000, 1_024, p, bad);
        assert_eq!(got, plain, "scale {bad:?} should fall back to the plain ranking");
    }
}
