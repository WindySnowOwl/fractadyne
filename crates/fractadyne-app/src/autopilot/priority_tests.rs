//! The autopilot's priority: the Quality pace (`quality_speed_cap`) and how the glide takes it.

use super::*;

const RATE: f64 = crate::ZOOM_RATE * 4.0; // the slider's fastest setting, nepers/s
const DT: f64 = 1.0 / 60.0;

#[test]
fn the_pace_covers_the_held_frame_bound_in_one_refresh_period() {
    // Refreshes land every 0.4 s: the held frame may magnify HELD_MAX_OCT before the next one,
    // so the pace is that many octaves per 0.4 s.
    let cap = quality_speed_cap(0.4, 0.0, RATE);
    let expect = crate::tunables::HELD_MAX_OCT * std::f64::consts::LN_2 / 0.4;
    assert!((cap - expect).abs() < 1e-12, "cap {cap} expected {expect}");
    assert!(cap < RATE, "0.4 s refreshes cannot sustain the 4× rate ({cap} vs {RATE})");
}

#[test]
fn a_dive_with_no_refresh_yet_keeps_the_rate() {
    assert_eq!(quality_speed_cap(0.0, 0.0, RATE), RATE);
    assert_eq!(quality_speed_cap(f64::NAN, 0.0, RATE), RATE);
    // Quick refreshes never push the pace ABOVE the rate either.
    assert_eq!(quality_speed_cap(0.01, 0.0, RATE), RATE);
}

#[test]
fn a_held_frame_already_past_the_bound_slows_the_dive_in_proportion() {
    let bound = crate::tunables::HELD_MAX_OCT;
    let at_bound = quality_speed_cap(0.4, bound, RATE);
    let twice = quality_speed_cap(0.4, 2.0 * bound, RATE);
    assert!((twice - at_bound / 2.0).abs() < 1e-12, "{twice} vs half of {at_bound}");
    // …and a frame inside the bound leaves the period's pace alone.
    assert_eq!(quality_speed_cap(0.4, bound * 0.5, RATE), at_bound);
}

#[test]
fn the_pace_never_stalls_the_dive() {
    // A refresh that takes a minute (or never lands) still leaves the floor: a stalled dive
    // cannot produce the refresh that would release it.
    let floor = RATE * crate::tunables::QUALITY_MIN_SPEED_FRAC;
    assert_eq!(quality_speed_cap(60.0, 0.0, RATE), floor);
    assert_eq!(quality_speed_cap(60.0, 40.0, RATE), floor);
    assert_eq!(quality_speed_cap(0.4, f64::INFINITY, RATE), floor);
}

#[test]
fn the_glide_eases_into_the_cap_rather_than_stepping() {
    // Up to speed at the full rate; then the cap halves it. The next frame moves a fraction of
    // the way (SPEED_TAU), never all of it.
    let p = (0.5, 0.5);
    let capped = glide_step(p, p, p, RATE, RATE, RATE * 0.5, 1.6, DT);
    assert!(capped.speed < RATE && capped.speed > RATE * 0.5, "eased: {}", capped.speed);
    let expect = RATE + (RATE * 0.5 - RATE) * (1.0 - (-DT / SPEED_TAU).exp());
    assert!((capped.speed - expect).abs() < 1e-12);
    // No cap: the same frame holds the rate.
    let free = glide_step(p, p, p, RATE, RATE, f64::INFINITY, 1.6, DT);
    assert!((free.speed - RATE).abs() < 1e-12);
}

#[test]
fn the_priority_round_trips_through_its_session_string() {
    for p in [AutopilotPriority::Speed, AutopilotPriority::Quality] {
        assert_eq!(AutopilotPriority::parse(p.as_str()), Some(p));
    }
    assert_eq!(AutopilotPriority::parse(" Quality "), Some(AutopilotPriority::Quality));
    assert_eq!(AutopilotPriority::parse("fastest"), None);
    assert_eq!(AutopilotPriority::default(), AutopilotPriority::Speed);
}
