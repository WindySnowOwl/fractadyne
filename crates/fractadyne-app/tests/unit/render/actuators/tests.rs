use super::*;

#[test]
fn an_unprobed_device_offers_no_walk() {
    for mode in [RenderMode::Direct, RenderMode::Df32Pert, RenderMode::Floatexp] {
        assert!(!can_chunk(None, mode), "{mode:?}");
    }
}

#[test]
fn the_deep_walk_needs_the_fourth_target() {
    let three = Actuators::new(true, false, true, 0);
    assert!(three.can_chunk(RenderMode::Direct) && three.can_chunk(RenderMode::Df32Pert));
    assert!(!three.can_chunk(RenderMode::Floatexp));
    let four = Actuators::new(true, true, true, 0);
    assert!(four.can_chunk(RenderMode::Floatexp));
    // No walk at all: the mode-2 bit is meaningless without the three targets.
    let none = Actuators::new(false, true, true, 0);
    assert!(!none.can_chunk(RenderMode::Direct) && !none.can_chunk(RenderMode::Floatexp));
}

#[test]
fn the_verdict_names_a_late_probe_and_every_missing_actuator() {
    let late = Actuators::new(false, false, false, 3).verdict_line();
    assert!(late.contains("resumable walk NO"), "{late}");
    assert!(late.contains("GPU timing NO"), "{late}");
    assert!(late.contains("probed at frame 3"), "{late}");
    let full = Actuators::new(true, true, true, 0).verdict_line();
    assert!(!full.contains("NO") && !full.contains("probed at"), "{full}");
}
