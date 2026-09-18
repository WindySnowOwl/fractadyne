use super::*;

fn pin() -> PinnedRefresh {
    let bf = |v: f64| fractadyne_core::BigFloat::from_f64(v, 64);
    PinnedRefresh {
        center_bf: [bf(-1.0), bf(0.2)],
        center: (-1.0, 0.2),
        span: (
            fractadyne_core::FloatExp::from_f64(1.0),
            fractadyne_core::FloatExp::from_f64(1.0),
        ),
        magnification: 1.0e31,
        log2mag: 103.0,
        upp_l2: -110.0,
        eff_iter: 1_000_000,
        gpu_iter: 1_000_000,
        resolution: [480, 270],
        panel: [960, 540],
        ss: 1,
        orbit_id: 7,
        orbit_len: 868,
        ref_pt: None,
        started_frame: 1_000,
    }
}

fn inputs() -> PinInputs {
    PinInputs {
        interacting: true,
        caller_reproject: false,
        drift_oct: 0.3,
        pan_spans: 0.0,
        orbit_id: 7,
        orbit_len: 868,
        same_point: false,
        panel: [960, 540],
        frame_idx: 1_010,
        cursor: 400_000,
    }
}

#[test]
fn a_same_point_extension_keeps_the_pin() {
    // The lookahead re-installs the SAME reference point, longer (or merely with fresh tables),
    // every ~0.2 s at 4.0×: the stored per-pixel state resumes against an identical prefix.
    let mut i = inputs();
    i.same_point = true;
    i.orbit_id = 8;
    i.orbit_len = 900;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Continue, "extension");
    i.orbit_len = 868;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Continue, "same length, new id");
    // A SHORTER same-point orbit is a collapse / re-pick: its prefix is not the pinned one.
    i.orbit_len = 800;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Stop(PinStop::Orbit), "shorter");
    // Any other point is another orbit, however long.
    i.same_point = false;
    i.orbit_len = 900;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Stop(PinStop::Orbit), "other point");
    // And a same-point install never outranks completion or the settle edge.
    i.same_point = true;
    i.cursor = 1_000_000;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Adopt);
}

#[test]
fn a_mid_flight_pin_continues() {
    assert_eq!(pin_verdict(&pin(), &inputs()), PinVerdict::Continue);
}

#[test]
fn adoption_requires_the_full_ask_and_nothing_else() {
    // Complete → adopt, even at the settle edge, past the drift threshold, or old: the work
    // is done and the texture is whole — discarding it buys nothing.
    let mut i = inputs();
    i.cursor = 1_000_000;
    i.interacting = false;
    i.drift_oct = 5.0;
    i.frame_idx = 10_000;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Adopt);
    // One iteration short is not complete — a partial refresh can never become the held
    // frame (the §9 regression, requirement 3 of §10).
    i.cursor = 999_999;
    i.interacting = true;
    i.drift_oct = 0.0;
    i.frame_idx = 1_010;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Continue);
}

#[test]
fn every_abandon_reason_fires_and_is_ordered_after_adopt() {
    let cases: &[(&dyn Fn(&mut PinInputs), PinStop)] = &[
        (&|i| i.interacting = false, PinStop::Settled),
        (&|i| i.orbit_id = 8, PinStop::Orbit),
        (&|i| i.orbit_len = 900, PinStop::Orbit),
        (&|i| i.panel = [961, 540], PinStop::Panel),
        (&|i| i.caller_reproject = true, PinStop::CallerReproject),
        (&|i| i.drift_oct = 2.1, PinStop::Drift),
        (&|i| i.pan_spans = 1.6, PinStop::Pan),
        (&|i| i.frame_idx = 1_000 + crate::tunables::PIN_MAX_FRAMES + 1, PinStop::Age),
    ];
    for (mutate, want) in cases {
        let mut i = inputs();
        mutate(&mut i);
        assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Stop(*want), "expected {want:?}");
        // The same violation with a COMPLETE cursor still adopts.
        i.cursor = 1_000_000;
        assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Adopt, "adopt outranks {want:?}");
    }
}

#[test]
fn the_thresholds_are_boundaries_not_bands() {
    // Exactly AT a threshold continues; strictly past it stops — a pin must not flap on a
    // value that sits on the line for several frames.
    let mut i = inputs();
    i.drift_oct = crate::tunables::PIN_ABANDON_OCTAVES;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Continue);
    i.drift_oct = 0.0;
    i.pan_spans = crate::tunables::PIN_ABANDON_SPANS;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Continue);
    i.pan_spans = 0.0;
    i.frame_idx = 1_000 + crate::tunables::PIN_MAX_FRAMES;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Continue);
}
