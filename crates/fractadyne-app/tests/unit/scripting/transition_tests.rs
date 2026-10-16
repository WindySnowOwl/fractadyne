use super::{parse_tour_text, Composite};

fn pb(body: &str) -> super::Playback {
    let text = format!("format_version = 2\nname = \"t\"\n{body}");
    parse_tour_text(&text).expect("script should parse")
}

#[test]
fn fade_in_rises_from_black_and_then_stops_compositing() {
    let p = pb("[[keyframe]]\nt = 0.0\nzoom = 1.0\nhold = 4.0\ntransition = \"fade\"\n\
                transition_secs = 1.0\n");
    assert_eq!(p.transition_at(0.0), Some(Composite::Scale(0.0)), "starts black");
    assert_eq!(p.transition_at(0.5), Some(Composite::Scale(0.5)), "half way");
    // Past the window there is nothing to do — every later frame must be untouched, not
    // multiplied by 1.0, so the common case costs no pass over the buffer at all.
    assert_eq!(p.transition_at(1.5), None);
}

#[test]
fn fade_out_sinks_over_the_end_of_its_hold() {
    let p = pb("[[keyframe]]\nt = 0.0\nzoom = 1.0\nhold = 5.0\nfade_out_secs = 2.0\n");
    assert_eq!(p.transition_at(1.0), None, "before the window: untouched");
    assert_eq!(p.transition_at(3.0), Some(Composite::Scale(1.0)), "window opens at full");
    assert_eq!(p.transition_at(4.0), Some(Composite::Scale(0.5)));
    assert_eq!(p.transition_at(5.0), Some(Composite::Scale(0.0)), "ends black");
}

#[test]
fn sinking_beats_rising_when_the_windows_overlap() {
    // A short hold with both a fade-in and a fade-out: the last frame of the tour is the one
    // moment a viewer is guaranteed to be watching, and it must go DARK. If rising won here the
    // tour would brighten as it ended.
    let p = pb("[[keyframe]]\nt = 0.0\nzoom = 1.0\nhold = 1.0\ntransition = \"fade\"\n\
                transition_secs = 1.0\nfade_out_secs = 1.0\n");
    assert_eq!(p.transition_at(1.0), Some(Composite::Scale(0.0)), "ends black, not bright");
}

#[test]
fn dissolve_reports_blend_and_is_detected_for_the_order_guard() {
    let p = pb("[[keyframe]]\nt = 0.0\nzoom = 1.0\nhold = 2.0\n\
                [[keyframe]]\nt = 2.0\nzoom = 100.0\nhold = 2.0\n\
                transition = \"dissolve\"\ntransition_secs = 1.0\n");
    assert!(p.has_dissolve(), "the renderer must know to demand sequential order");
    assert_eq!(p.transition_at(2.0), Some(Composite::Blend(0.0)), "starts on the old picture");
    assert_eq!(p.transition_at(2.5), Some(Composite::Blend(0.5)));
    assert_eq!(p.transition_at(3.5), None, "and is over");
    // The first keyframe asked for nothing, so nothing happens there.
    assert_eq!(p.transition_at(1.0), None);
}

#[test]
fn a_cut_and_an_absent_transition_are_both_free() {
    let p = pb("[[keyframe]]\nt = 0.0\nzoom = 1.0\nhold = 2.0\ntransition = \"cut\"\n\
                transition_secs = 1.0\n");
    assert_eq!(p.transition_at(0.0), None);
    assert_eq!(p.transition_at(0.5), None);
    assert!(!p.has_dissolve());
}

#[test]
fn a_transition_longer_than_its_hold_is_clamped_to_it() {
    // Otherwise the ramp is still climbing when the next keyframe arrives and the picture
    // jumps at partial brightness.
    let p = pb("[[keyframe]]\nt = 0.0\nzoom = 1.0\nhold = 0.5\ntransition = \"fade\"\n\
                transition_secs = 10.0\n");
    assert_eq!(p.transition_at(0.5), Some(Composite::Scale(1.0)), "reaches full within the hold");
    assert_eq!(p.transition_at(0.6), None);
}
