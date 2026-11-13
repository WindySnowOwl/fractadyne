use super::*;

/// The capabilities no note reports, each left on purpose to another control (see
/// [`MethodNote::reads`]): the normalized export, which the panel's "Normalize deep colors" box
/// explains.
const NOT_NOTED: [Capability; 1] = [Capability::NormalizedExport];

/// Every capability of the B4 table is either told by a note or left, on purpose, to another
/// control ([`NOT_NOTED`]): a capability added to the table must be decided here, or this fails.
#[test]
fn every_capability_is_noted_or_told_elsewhere() {
    for c in Capability::ALL {
        let noted = MethodNote::ALL.iter().any(|n| n.reads().contains(&c));
        let elsewhere = NOT_NOTED.contains(&c);
        assert!(noted != elsewhere, "{c:?}: noted {noted}, left to another control {elsewhere}");
    }
}

/// Where everything is offered (a Mandelbrot view with the skips on), the notes are the ones the
/// renderer's established rules give: the count and the distance give up nothing; stripe, triangle
/// and the trap are calculated step by step between 1e4x and 1e28x (no series, no band BLA) and
/// take BLA past it; decomposition takes neither at any depth (its angle has no BLA aggregate);
/// every statistics method is refused by glitch correction and by zoom videos' strips.
#[test]
fn the_notes_are_the_established_ones() {
    use MethodNote::*;
    let all = Offered { skip_shallow: true, skip_deep: true, glitch_correction: true, zoom_video: true };
    let notes = |m: ColorMethod| method_notes(m, all);
    assert_eq!(notes(ColorMethod::Smooth), []);
    assert_eq!(notes(ColorMethod::Distance), []);
    for m in [ColorMethod::Stripe, ColorMethod::TriangleIneq, ColorMethod::OrbitTrap] {
        assert_eq!(notes(m), [NoSkipShallow, NoGlitchCorrection, NoZoomVideo], "{m:?}");
    }
    assert_eq!(notes(ColorMethod::Decomposition), [NoSkip, NoGlitchCorrection, NoZoomVideo]);
}

/// A note says a method gives something up only where the view would otherwise give it: on a
/// Julia view nothing skips iterations, Mandelbrot exports run no glitch correction, and no zoom
/// video can be made of a Burning Ship.
#[test]
fn a_note_only_where_the_view_would_give_it() {
    for m in ColorMethod::ALL {
        assert_eq!(method_notes(m, Offered::default()), [], "{m:?} with nothing offered");
    }
    let deep_only = Offered { skip_deep: true, ..Offered::default() };
    assert_eq!(method_notes(ColorMethod::Stripe, deep_only), [], "stripe takes BLA where only that is offered");
    assert_eq!(method_notes(ColorMethod::Decomposition, deep_only), [MethodNote::NoSkip]);
    let videos = Offered { zoom_video: true, ..Offered::default() };
    assert_eq!(method_notes(ColorMethod::OrbitTrap, videos), [MethodNote::NoZoomVideo]);
    assert_eq!(method_notes(ColorMethod::Distance, videos), []);
}
