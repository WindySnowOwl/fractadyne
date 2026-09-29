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
        split: 0,
        split_next: 0,
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
        policy: RefreshPolicy::Full,
        // The historical fixtures predate content verification: a finished walk with a reading
        // in hand, so the verdicts they pin are unchanged.
        detail: Some(true),
        reading_final: true,
        converged: false,
        past_range_top: false,
    }
}

#[test]
fn a_finished_walk_adopts_on_a_reading_with_detail_and_waits_for_one_without() {
    let mut i = inputs();
    i.cursor = 1_000_000;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Adopt);
    // No reading of this pin yet: the tail keeps arming readbacks; the pin waits.
    i.detail = None;
    i.reading_final = false;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Continue);
    // A blank reading of an EARLIER pass is not the answer either…
    i.detail = Some(false);
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Continue);
    // …the completing pass's blank reading is: nothing escaped at this ask.
    i.reading_final = true;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Stop(PinStop::Blank));
}

#[test]
fn nothing_but_the_settle_edge_or_age_takes_a_finished_walk_away_while_it_waits() {
    let mut i = inputs();
    i.cursor = 1_000_000;
    i.detail = None;
    i.reading_final = false;
    // An install, a drift past the abandon threshold, a pan, a resize: all wait.
    i.orbit_id = 99;
    i.drift_oct = 5.0;
    i.pan_spans = 3.0;
    i.panel = [1, 1];
    i.caller_reproject = true;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Continue);
    i.interacting = false;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Stop(PinStop::Settled));
    i.interacting = true;
    i.frame_idx = 1_000 + crate::tunables::PIN_MAX_FRAMES + 1;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Stop(PinStop::Age));
}

#[test]
fn the_converged_policy_adopts_a_picture_that_stopped_changing_and_the_full_policy_does_not() {
    let mut i = inputs();
    i.cursor = 6_000; // far short of the 1,000,000 ask
    i.detail = Some(true);
    i.converged = true;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Continue, "Full: every pixel decided first");
    i.policy = RefreshPolicy::Converged;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::AdoptConverged);
    // The known escape range is the other door in…
    i.converged = false;
    i.past_range_top = true;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::AdoptConverged);
    // …and neither opens without a reading that shows detail.
    i.detail = Some(false);
    i.converged = true;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Continue);
    i.detail = None;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::Continue);
}

#[test]
fn a_converged_adoption_still_outranks_the_abandons_but_not_a_blank_reading() {
    let mut i = inputs();
    i.cursor = 6_000;
    i.policy = RefreshPolicy::Converged;
    i.detail = Some(true);
    i.converged = true;
    i.drift_oct = 5.0;
    i.orbit_id = 99;
    assert_eq!(pin_verdict(&pin(), &i), PinVerdict::AdoptConverged);
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

fn split_pin(passes: u32, next: u32) -> PinnedRefresh {
    PinnedRefresh { split: passes, split_next: next, ..pin() }
}

#[test]
fn a_split_pin_adopts_when_every_set_has_run_whatever_else_holds() {
    // Complete by construction — no reading to wait for (none of this pin's content has come
    // back), no cursor (the walk's progress slot is irrelevant to it) — and, like a finished walk,
    // it outranks every abandon: the texture is whole.
    let mut i = inputs();
    i.detail = None;
    i.reading_final = false;
    i.cursor = 0;
    i.interacting = false;
    i.drift_oct = 5.0;
    i.orbit_id = 99;
    i.frame_idx = 1_000 + crate::tunables::PIN_MAX_FRAMES + 1;
    assert_eq!(pin_verdict(&split_pin(3, 3), &i), PinVerdict::Adopt);
    // One set short is not complete.
    assert_eq!(pin_verdict(&split_pin(3, 2), &inputs()), PinVerdict::Continue);
}

#[test]
fn a_split_pin_keeps_the_walk_abandons_but_not_the_orbit_one() {
    // A split pass keeps no state for the next: a reference installed between passes only changes
    // which valid orbit the later sets use, so it continues where a walk would stop.
    let mut i = inputs();
    i.orbit_id = 8;
    i.orbit_len = 100;
    assert_eq!(pin_verdict(&split_pin(4, 1), &i), PinVerdict::Continue);
    let cases: &[(&dyn Fn(&mut PinInputs), PinStop)] = &[
        (&|i| i.interacting = false, PinStop::Settled),
        (&|i| i.panel = [961, 540], PinStop::Panel),
        (&|i| i.caller_reproject = true, PinStop::CallerReproject),
        (&|i| i.drift_oct = 2.1, PinStop::Drift),
        (&|i| i.pan_spans = 1.6, PinStop::Pan),
        (&|i| i.frame_idx = 1_000 + crate::tunables::PIN_MAX_FRAMES + 1, PinStop::Age),
    ];
    for (mutate, want) in cases {
        let mut i = inputs();
        mutate(&mut i);
        assert_eq!(pin_verdict(&split_pin(4, 1), &i), PinVerdict::Stop(*want), "expected {want:?}");
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
