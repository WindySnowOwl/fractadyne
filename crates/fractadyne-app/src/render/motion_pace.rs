use super::*;

const DT60: f64 = 1.0 / 60.0;

#[test]
fn a_pass_is_sized_from_the_measured_rate_to_the_target() {
    // 1e8 nominal steps/ms at a 10 ms target = 1e9 steps — not the 4e10 TDR budget.
    assert_eq!(motion_pass_steps(1.0e8, 10.0, 4.0e10 as u64, 4.0e8 as u64), 1.0e9 as u64);
    // The TDR budget stays the ceiling: a fast regime cannot size a dispatch past it.
    assert_eq!(motion_pass_steps(1.0e12, 10.0, 4.0e10 as u64, 4.0e8 as u64), 4.0e10 as u64);
}

#[test]
fn no_measurement_means_the_opening_guess_never_the_budget() {
    // The pin's first pass in a fresh mode used to run "at the budget's size" — after a jump
    // that is the 400 ms TDR target itself. With no rate it is the rate-derived bootstrap.
    assert_eq!(motion_pass_steps(0.0, 10.0, 4.0e10 as u64, 4.0e8 as u64), 4.0e8 as u64);
    assert_eq!(motion_pass_steps(f64::NAN, 10.0, 4.0e10 as u64, 4.0e8 as u64), 4.0e8 as u64);
    // The guess is itself bounded by the budget, and the result is never zero.
    assert_eq!(motion_pass_steps(0.0, 10.0, 1_000, 4.0e8 as u64), 1_000);
    assert_eq!(motion_pass_steps(0.0, 10.0, 0, 0), 1);
}

#[test]
fn the_pass_count_follows_the_zoom_rate() {
    // 4.0× on the slider = 2.67 oct/s: half an octave is 0.187 s = 11 frames at 60 Hz.
    let fast = refresh_passes_allowed(2.67, 0.5, 0.25, DT60);
    // 1.0× = 0.667 oct/s: the octave budget would allow 0.75 s, the time cap holds it to 0.25 s.
    let slow = refresh_passes_allowed(0.667, 0.5, 0.25, DT60);
    assert_eq!(fast, 11);
    assert_eq!(slow, 15);
    assert!(fast < slow, "a faster zoom must get FEWER passes, not more");
    // Below the time-cap knee every rate gets the same count — the slider's bottom is not
    // rewarded with multi-second pins.
    assert_eq!(refresh_passes_allowed(0.17, 0.5, 0.25, DT60), slow);
    // Not zooming (a pan, a settle edge) is the time cap too, and a garbage frame time is 60 Hz.
    assert_eq!(refresh_passes_allowed(0.0, 0.5, 0.25, DT60), slow);
    assert_eq!(refresh_passes_allowed(0.0, 0.5, 0.25, 0.0), slow);
    // Never zero, never past the pin's own age limit.
    assert_eq!(refresh_passes_allowed(1000.0, 0.5, 0.25, DT60), 1);
    assert_eq!(
        refresh_passes_allowed(1e-9, 0.5, 1e9, DT60),
        crate::tunables::PIN_MAX_FRAMES as u32
    );
}

#[test]
fn resolution_absorbs_what_the_passes_cannot_cover() {
    // 1.6 Mpx × 35k iterations = 5.6e10 nominal; 11 passes of 1.1e9 = 1.2e10 ⇒ sqrt(0.216).
    let cap = refresh_res_cap(1_100_000_000, 11, 35_000, 1_600_000, 0.30);
    assert!((cap - (1.21e10_f64 / 5.6e10).sqrt()).abs() < 1e-3, "got {cap}");
    // More passes (a slower zoom) ⇒ a sharper refresh; enough passes ⇒ native.
    assert!(refresh_res_cap(1_100_000_000, 15, 35_000, 1_600_000, 0.30) > cap);
    assert_eq!(refresh_res_cap(1_100_000_000, 60, 35_000, 1_600_000, 0.30), 1.0);
    // The user's sharpness floor holds however fast the zoom, and nothing exceeds native.
    assert_eq!(refresh_res_cap(1_000, 1, 1_000_000, 1_600_000, 0.30), 0.30);
    assert_eq!(refresh_res_cap(u64::MAX, 1, 1, 1, 0.30), 1.0);
    // Degenerate inputs are "no opinion" (no cap), never a zero-pixel frame.
    assert_eq!(refresh_res_cap(0, 0, 0, 0, f64::NAN), 1.0);
    assert_eq!(refresh_res_cap(1_000, 0, 1_000_000, 1_600_000, 0.30), 1.0);
    // A NaN floor is treated as no floor: the raw shortfall root comes through.
    let raw = refresh_res_cap(1_000, 1, 1_000_000, 1_600_000, f64::NAN);
    assert!((raw - (1_000.0_f64 / 1.6e12).sqrt()).abs() < 1e-12, "got {raw}");
}

#[test]
fn a_pin_pass_grows_in_proportion_to_how_cheap_it_priced() {
    // 0.1 ms against a 20 ms target: the model would allow ×100; the lane caps at ×16.
    assert_eq!(pin_fast_lane(0.1, 20.0), 16);
    // 1 ms ⇒ ×10 (predicted 10 ms = half the target), 2.5 ms ⇒ ×4, and never below ×4.
    assert_eq!(pin_fast_lane(1.0, 20.0), 10);
    assert_eq!(pin_fast_lane(2.5, 20.0), 4);
    assert_eq!(pin_fast_lane(9.0, 20.0), 4);
    // An unmeasurably cheap pass (zero, negative, NaN) is the strongest "cheap": ×16.
    assert_eq!(pin_fast_lane(0.0, 20.0), 16);
    assert_eq!(pin_fast_lane(-1.0, 20.0), 16);
    assert_eq!(pin_fast_lane(f64::NAN, 20.0), 16);
    // The grown pass is the ledger's business: a cheap price still multiplies the SIZE only
    // through `chunk_band_update_with_lane`, which applies the lane below half the target.
    let mut bands = [0u32; crate::tunables::CHUNK_BANDS];
    chunk_band_update_with_lane(&mut bands, 7, 256, 0.1, 20.0, pin_fast_lane(0.1, 20.0));
    assert_eq!(bands[7], 4096);
    chunk_band_update_with_lane(&mut bands, 7, 4096, 1.0, 20.0, pin_fast_lane(1.0, 20.0));
    assert_eq!(bands[7], 40_960);
}

#[test]
fn a_pin_inherits_a_cold_band_only_past_the_first_wrap_storm() {
    let mut bands = [0u32; crate::tunables::CHUNK_BANDS];
    bands[8] = 16_384; // [32k, 64k) priced; band 9 [64k, 128k) cold
    let orbit_len = 868;
    // Past 2 × orbit_len: the cold band opens at half its predecessor, not the floor.
    assert_eq!(pin_band_license(&bands, 9, 256, 65_536, orbit_len), 8_192);
    // The floor still holds when the predecessor is tiny.
    bands[8] = 300;
    assert_eq!(pin_band_license(&bands, 9, 256, 65_536, orbit_len), 256);
    // A band with its own licence uses it (and never below the floor).
    bands[9] = 100;
    assert_eq!(pin_band_license(&bands, 9, 256, 65_536, orbit_len), 256);
    bands[9] = 2_048;
    assert_eq!(pin_band_license(&bands, 9, 256, 65_536, orbit_len), 2_048);
    // Inside the storm's reach — a pass starting below 2 × orbit_len — the floor rule is
    // untouched, however rich the predecessor: band 3 [512, 1024) with orbit_len 868.
    let mut cold = [0u32; crate::tunables::CHUNK_BANDS];
    cold[2] = 65_536;
    assert_eq!(pin_band_license(&cold, 3, 256, 512, orbit_len), 256);
    // A long reference (100k) keeps every band below 200k at the floor.
    cold[8] = 16_384;
    assert_eq!(pin_band_license(&cold, 9, 256, 65_536, 100_000), 256);
    // No reference length known ⇒ no inheritance; band 0 has no predecessor.
    assert_eq!(pin_band_license(&cold, 9, 256, 65_536, 0), 256);
    assert_eq!(pin_band_license(&cold, 0, 256, 0, orbit_len), 256);
}

#[test]
fn a_moving_frame_is_sized_so_it_can_reach_an_escape() {
    // The user's 2^800 dive, from the log and the video of it (2026-09-20): a motion pass carried
    // ~1.35e9 steps over a 1469x1102 panel (a 833-iteration walk), and pixels there do not escape
    // until ~4471 — so at native the frame commits NOTHING and paints one flat colour, which is
    // 36% of what the video recorded.
    let px = 1469u64 * 1102;
    let pass = 833 * px; // the budget the frame was actually given
    let res = visible_res_target(pass, px, 4471.0);
    assert!(res < 0.50 && res > 0.30, "sized at {res:.3}, expected about 0.43");
    // At that scale the pass reaches the band — which is the whole point.
    let walked = pass as f64 / (px as f64 * res * res);
    assert!(walked >= 4471.0, "still blank: walks {walked:.0} of 4471");
    // ⭐Both of the user's settings forbid it, which is why this sits beneath them.
    assert!(res < 0.83, "min_motion_res 0.83 blocks it");
}

#[test]
fn the_visible_target_is_absolute_so_it_cannot_wind_up() {
    // ⛔The bug in the first attempt: a per-pass FEEDBACK step wound down to its floor (146x110
    // for 1760 frames at 2^800) because `budget_step` may only double per pass, so the cap moved
    // faster than the step could answer. An absolute target holds still while the step climbs.
    let px = 1469u64 * 1102;
    let pass = 833 * px;
    let first = visible_res_target(pass, px, 4471.0);
    for _ in 0..100 {
        assert_eq!(visible_res_target(pass, px, 4471.0), first, "the target moved on its own");
    }
    assert!(first > crate::render::VISIBLE_RES_MIN, "wound down to the floor");
}

#[test]
fn the_visible_target_never_guesses_and_never_asks_for_no_pixels() {
    let px = 1469u64 * 1102;
    // Budget already covers native ⇒ no opinion, never a gratuitous shrink.
    assert_eq!(visible_res_target(u64::MAX, px, 4471.0), 1.0);
    // Shallow views escape in a handful of iterations — native fits easily.
    assert_eq!(visible_res_target(833 * px, px, 40.0), 1.0);
    // Nothing measured yet (a fresh view, or the first frame) must not shrink anything.
    assert_eq!(visible_res_target(0, px, 4471.0), 1.0);
    assert_eq!(visible_res_target(833 * px, px, 0.0), 1.0);
    assert_eq!(visible_res_target(833 * px, px, f64::NAN), 1.0);
    assert_eq!(visible_res_target(833 * px, 0, 4471.0), 1.0);
    // However hopeless, it never asks for a zero-pixel frame.
    assert_eq!(visible_res_target(1, px, 1.0e9), crate::render::VISIBLE_RES_MIN);
}

#[test]
fn the_visible_scale_is_held_because_changing_it_restarts_the_walk() {
    // ⛔The lesson both earlier attempts cost: resolution is part of the walk's signature, so each
    // change discards the iteration cursor. They re-signed 128-216 times a dive against the
    // baseline's ONE, and the screen got worse (8% -> 36% blank) even as walks reached further.
    // Drops are immediate; a target drifting inside a rung must produce NO change at all.
    let mut held = 1.0;
    let mut changes = 0;
    for (i, t) in [0.44, 0.46, 0.43, 0.45, 0.44, 0.47, 0.42, 0.45].into_iter().enumerate() {
        let now = visible_res_hold(held, t);
        if now != held {
            changes += 1;
        }
        held = now;
        assert!(held <= t + 1e-9, "step {i}: held {held} cannot show anything at target {t}");
    }
    assert_eq!(changes, 1, "a hovering target re-signed the walk {changes} times");
    assert_eq!(held, 0.353_553_39, "expected the rung below 0.42");
    // Sharpening needs the target to clear the held rung by half again — not a nudge.
    assert_eq!(visible_res_hold(0.5, 0.60), 0.5, "sharpened on a nudge");
    assert_eq!(visible_res_hold(0.5, 0.80), 0.707_106_78);
    // A frame that cannot show anything drops at once, with no hysteresis in the way.
    assert_eq!(visible_res_hold(1.0, 0.20), 0.176_776_70);
    // Degenerate inputs never move it off native.
    assert_eq!(visible_res_hold(1.0, f64::NAN), 1.0);
    assert_eq!(visible_res_hold(f64::NAN, 1.0), 1.0);
}
