use super::*;
use crate::RenderMode;

/// A frame is classified by the mode it renders in, whatever depth chose it: perturbation for both
/// perturbation modes, floatexp's moving share for floatexp wherever it runs (past
/// `PERT_FE_THRESHOLD`, and in the df32 band under `BAND_FLOATEXP`).
#[test]
fn frame_cost_follows_the_mode() {
    for (mode, pert) in [(RenderMode::Direct, false), (RenderMode::Df32Pert, true), (RenderMode::Floatexp, true)] {
        assert_eq!(frame_cost(1_000, mode, false).is_pert, pert, "{mode:?}");
    }
}

/// A garbage magnification must pick the SAFEST mode, never the most expensive one.
#[test]
fn a_nan_magnification_falls_back_to_direct_but_an_infinite_one_goes_deepest() {
    // THE FIELD CASE (2026-08-18, build 1678): a corrupted session gave a NaN zoom, and because
    // every comparison against NaN is false the selector fell through to Floatexp — the most
    // expensive arithmetic at maximum depth. Logged as
    // `arithmetic mode none → 2 at frame 1 (mag 2^NaN)`. With a 250k iteration ask that is the
    // un-chunkable ~1 s/frame regime, so the app opened to a black screen, "iter capped", a
    // laggy desktop, and a device loss waiting to happen.
    assert_eq!(RenderMode::select(true, false, f64::NAN), RenderMode::Direct);
    assert_eq!(RenderMode::select(true, true, f64::NAN), RenderMode::Direct);
    // A NEGATIVE magnification is garbage too, and the ordinary `< direct_below` compare
    // already handles it — no guard needed.
    assert_eq!(RenderMode::select(true, false, f64::NEG_INFINITY), RenderMode::Direct);

    // ⭐**`+∞` IS A REAL VIEW, NOT GARBAGE** — and the version of this test written alongside
    // the NaN guard asserted the opposite, which is how the defect survived four days.
    // `Viewport::magnification()` saturates past ~1e308×, so `+∞` is exactly what every
    // genuinely extreme location reports. Demoting it to Direct drops perturbation entirely
    // and renders a BLANK frame (the 4.6e1105× bench scene, "144× faster than Fraktaler-3",
    // was an empty image). Floatexp is the mode that exists for "deeper than f64 can say".
    assert_eq!(RenderMode::select(true, false, f64::INFINITY), RenderMode::Floatexp);
    assert_eq!(RenderMode::select(true, true, f64::INFINITY), RenderMode::Floatexp);
    // ...unless the formula has no perturbation at all, which still wins over everything.
    assert_eq!(RenderMode::select(false, false, f64::INFINITY), RenderMode::Direct);

    // And the ordinary partition is untouched by the guard.
    assert_eq!(RenderMode::select(true, false, 1.0), RenderMode::Direct);
    assert_eq!(RenderMode::select(true, false, 1.0e10), RenderMode::Df32Pert);
    assert_eq!(RenderMode::select(true, false, 1.0e30), RenderMode::Floatexp);
    // The saturation boundary itself: both sides of ~1e308 must choose the same mode, or the
    // f64's limit becomes a visible seam in the image at a depth the user can reach.
    assert_eq!(RenderMode::select(true, false, 1.0e308), RenderMode::Floatexp);
    assert_eq!(RenderMode::select(true, false, 1.0e308 * 10.0), RenderMode::Floatexp);
    // A formula without perturbation support is always the direct path, however deep.
    assert!(RenderMode::select(false, false, PERT_FE_THRESHOLD * 10.0).is_direct());
}

/// Moving frames are throttled by mode; settled frames get the full budget regardless.
#[test]
fn moving_budget_is_throttled_by_mode() {
    let wb = 6_000u64;
    assert_eq!(frame_cost(wb, RenderMode::Direct, true).budget, wb, "shallow moving: full");
    assert_eq!(frame_cost(wb, RenderMode::Df32Pert, true).budget, wb / 4, "df32 moving: quarter");
    assert_eq!(frame_cost(wb, RenderMode::Floatexp, true).budget, wb / 6, "floatexp moving: sixth");
    for mode in [RenderMode::Direct, RenderMode::Df32Pert, RenderMode::Floatexp] {
        assert_eq!(frame_cost(wb, mode, false).budget, wb * 6, "settled frames keep the full budget in every mode");
    }
}

/// Saturating multiply: a pathological budget must not wrap into a tiny one.
#[test]
fn settled_budget_saturates() {
    assert_eq!(frame_cost(u64::MAX, RenderMode::Direct, false).budget, u64::MAX);
}
