use super::*;

const DT60: f64 = 1.0 / 60.0;

#[test]
fn a_pass_is_sized_from_the_measured_rate_to_the_target() {
    // 1e8 nominal steps/ms at a 10 ms target = 1e9 steps — not the 4e10 TDR budget. (With no fixed
    // term; the shipped default sets 0.2 ms of every pass aside, leaving 9.8 ms of steps.)
    assert_eq!(motion_pass_steps_fixed(1.0e8, 10.0, 0.0, 4.0e10 as u64, 4.0e8 as u64), 1.0e9 as u64);
    assert_eq!(
        motion_pass_steps(1.0e8, 10.0, 4.0e10 as u64, 4.0e8 as u64),
        (1.0e8 * (10.0 - crate::tunables::PASS_FIXED_MS_DEFAULT)) as u64
    );
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

/// The user's 2^800 dive, from the log and the video of it (2026-09-20): a 1469×1102 panel, a
/// motion pass budget of ~1.35e9 steps (an 833-iteration walk at native), and an escape range
/// topping out near 4900 — so at native the frame commits NOTHING and paints one flat colour,
/// which is 36% of what the video recorded.
const PANEL: u64 = 1469 * 1102;
const PASS: u64 = 833 * PANEL;
const NEED: f64 = 4900.0;

#[test]
fn a_moving_frame_is_sized_so_it_can_show_the_picture() {
    let target = visible_res_target(PASS, PANEL, NEED);
    let held = visible_res_hold(1.0, target, visible_frame_reaches(PASS, PANEL, 1.0, NEED));
    assert_eq!(held, 0.25, "picked {held} for a target of {target:.3}");
    // At that rung the pass reaches the top of the range — with room to spare.
    assert!(visible_frame_reaches(PASS, PANEL, held, NEED));
    let walked = PASS as f64 / (PANEL as f64 * held * held);
    assert!(walked > NEED * 2.0, "walks {walked:.0}: no headroom against a budget cut");
    // ⭐Both of the user's settings forbid this scale, which is why it sits beneath them.
    assert!(held < 0.83, "min_motion_res 0.83 would have blocked it");
}

#[test]
fn the_rung_holds_through_the_budget_cuts_that_were_flipping_it() {
    // The 18 s capture at 2^800 saw the pass budget swing 1.35e9 → 5.5e8 (2.45×) whenever a
    // frame that reached the band cost all of it, and the rung followed every swing: 62 changes,
    // 11% of frames blank. A rung chosen with headroom survives the cut, and is not abandoned
    // for a target that has merely got more cautious.
    let held = 0.25;
    let cut = (PASS as f64 / 2.45) as u64;
    assert!(visible_frame_reaches(cut, PANEL, held, NEED), "the frame really is fine after the cut");
    let target_after_cut = visible_res_target(cut, PANEL, NEED);
    assert!(target_after_cut < held, "test premise: the cautious target now sits below the rung");
    assert_eq!(
        visible_res_hold(held, target_after_cut, visible_frame_reaches(cut, PANEL, held, NEED)),
        held,
        "the rung was abandoned on a cut it survives"
    );
    // The budget lifting back does not re-sign the walk either: nothing clears the raise margin.
    assert_eq!(visible_res_hold(held, visible_res_target(PASS, PANEL, NEED), true), held);
    // But a cut the frame genuinely cannot survive is answered at once.
    let deep_cut = PASS / 4;
    assert!(!visible_frame_reaches(deep_cut, PANEL, held, NEED));
    assert!(visible_res_hold(held, visible_res_target(deep_cut, PANEL, NEED), false) < held);
}

#[test]
fn the_visible_scale_is_a_short_sticky_ladder() {
    // ⛔Resolution is part of the walk's signature: every change discards the iteration cursor.
    // A target drifting within a rung must produce no change at all; a drop is immediate when the
    // frame cannot show anything; sharpening needs the target to clear the rung by half again.
    // Native cannot reach; every rung can. The target wobbles inside the 0.25 rung's band
    // (a raise needs 0.375), so after the one drop nothing may move.
    let mut held = 1.0;
    let mut changes = 0;
    for t in [0.30, 0.32, 0.28, 0.31, 0.29, 0.34, 0.27, 0.30] {
        let now = visible_res_hold(held, t, held < 1.0);
        changes += (now != held) as u32;
        held = now;
    }
    assert_eq!(changes, 1, "a hovering target re-signed the walk {changes} times");
    assert_eq!(held, 0.25);
    assert_eq!(visible_res_hold(0.5, 0.60, true), 0.5, "sharpened on a nudge");
    assert_eq!(visible_res_hold(0.5, 0.80, true), 0.707_106_78);
    assert_eq!(visible_res_hold(1.0, 0.20, false), 0.176_776_70);
    assert_eq!(visible_res_hold(1.0, f64::NAN, true), 1.0);
    assert_eq!(visible_res_hold(f64::NAN, 1.0, true), 1.0);
}

#[test]
fn the_visible_target_never_guesses_and_never_asks_for_no_pixels() {
    // Budget already covers native ⇒ no opinion, never a gratuitous shrink.
    assert_eq!(visible_res_target(u64::MAX, PANEL, NEED), 1.0);
    // Shallow views escape in a handful of iterations — native fits easily.
    assert_eq!(visible_res_target(PASS, PANEL, 40.0), 1.0);
    // Nothing measured yet (a fresh view, or the first frame) must not shrink anything.
    assert_eq!(visible_res_target(0, PANEL, NEED), 1.0);
    assert_eq!(visible_res_target(PASS, PANEL, 0.0), 1.0);
    assert_eq!(visible_res_target(PASS, PANEL, f64::NAN), 1.0);
    assert_eq!(visible_res_target(PASS, 0, NEED), 1.0);
    assert!(visible_frame_reaches(0, PANEL, 0.5, NEED), "nothing measured: nothing to abandon for");
    // However hopeless, it never asks for a zero-pixel frame.
    assert_eq!(visible_res_target(1, PANEL, 1.0e9), crate::render::VISIBLE_RES_MIN);
    // And an absolute target cannot wind up: the same inputs give the same answer, every time.
    let first = visible_res_target(PASS, PANEL, NEED);
    assert!((0..100).all(|_| visible_res_target(PASS, PANEL, NEED) == first));
}

#[test]
fn one_interval_cannot_slash_the_motion_rate() {
    // The pin-pass cuts measured at 2^800: 1.35e9 -> 1.5e8 (9x) on the evidence of ONE interval
    // the motion frame did not own, and the next motion frame painted a flat colour. A cut is
    // still a cut - it just cannot exceed MOTION_CUT_MAX in one event; repeated slow intervals
    // still ratchet it down.
    use crate::bounded_motion_cut;
    let cur = 1.35e8; // steps/ms
    assert_eq!(bounded_motion_cut(cur, cur / 9.0), Some(cur / crate::tunables::MOTION_CUT_MAX));
    let mut r = cur;
    for _ in 0..3 {
        r = bounded_motion_cut(r, cur / 9.0).unwrap();
    }
    assert!(r <= cur / 8.0, "three slow intervals must still be allowed to find a 9x slower regime");
    // A mild cut passes through untouched; a faster interval is no cut at all.
    assert_eq!(bounded_motion_cut(cur, cur * 0.7), Some(cur * 0.7));
    assert_eq!(bounded_motion_cut(cur, cur * 1.3), None);
    // Nothing measured yet: the interval IS the first measurement, unbounded.
    assert_eq!(bounded_motion_cut(0.0, 5.0e7), Some(5.0e7));
    // Garbage intervals leave the estimate alone.
    assert_eq!(bounded_motion_cut(cur, f64::NAN), None);
    assert_eq!(bounded_motion_cut(cur, 0.0), None);
}

/// `PASS_FIXED_MS` 0 is exactly the proportional sizing the app shipped with.
#[test]
fn no_fixed_term_is_the_proportional_sizing() {
    for rate in [1.0e5, 1.4e7, 1.0e8] {
        assert_eq!(
            motion_pass_steps_fixed(rate, 10.0, 0.0, 4.0e10 as u64, 4.0e8 as u64),
            (rate * 10.0) as u64
        );
    }
    assert_eq!(variable_pass_ms(1.14, 0.0), 1.14);
    assert_eq!(variable_pass_ms(1.14, -1.0), 1.14);
    assert_eq!(variable_pass_ms(1.14, f64::NAN), 1.14);
}

/// The RX 6800 XT case (2026-09-26): a 1.6e7-step pass in 1.14 ms with ~1 ms of it fixed. Priced
/// proportionally that is 1.4e7 steps/ms and a 10 ms pass of 1.4e8 steps. Unclamped, the variable
/// rate would be ~1.1e8 (8x); 88% of that reading is fixed, so the 4x clamp binds, and 9 ms at the
/// clamped rate is a pass about 3.6x larger.
#[test]
fn a_fixed_term_sizes_the_pass_from_the_variable_rate() {
    let (steps, ms, fixed) = (1.6e7, 1.14, 1.0);
    let proportional = steps / ms;
    let variable = steps / variable_pass_ms(ms, fixed);
    assert!((variable / proportional - 4.0).abs() < 1e-9, "clamped at 4x: {}", variable / proportional);
    let before = motion_pass_steps_fixed(proportional, 10.0, 0.0, u64::MAX, 1);
    let after = motion_pass_steps_fixed(variable, 10.0, fixed, u64::MAX, 1);
    assert!(after as f64 / before as f64 > 3.0, "{before} -> {after}");
    // A pass with less of its time fixed is not clamped: 3 ms with 1 ms fixed is a 1.5x rate.
    assert!((variable_pass_ms(3.0, 1.0) - 2.0).abs() < 1e-12);
}

/// Set too high, the fixed term can at most quadruple a rate and quarter a pass target: a value
/// that swallowed the whole reading must not size an unbounded pass, nor a zero one.
#[test]
fn a_fixed_term_set_too_high_is_bounded() {
    assert_eq!(variable_pass_ms(1.0, 5.0), 0.25);
    assert_eq!(variable_pass_ms(10.0, 9.9), 2.5);
    let p = motion_pass_steps_fixed(1.0e8, 10.0, 9.9, u64::MAX, 1);
    assert_eq!(p, 2.5e8 as u64);
    // ...and the TDR budget is still the ceiling.
    assert_eq!(motion_pass_steps_fixed(1.0e12, 10.0, 1.0, 4.0e10 as u64, 1), 4.0e10 as u64);
}

/// The quantile of a log2 escape histogram: bucket b spans [2^b, 2^(b+1)), log-linear inside it.
#[test]
fn an_escape_quantile_reads_the_log2_histogram() {
    let mut h = [0u32; 24];
    h[10] = 90; // 90 samples in [1024, 2048)
    h[13] = 10; // 10 stragglers in [8192, 16384)
    // 90% of the picture has escaped by the top of bucket 10.
    assert!((escape_quantile(&h, 0.9).unwrap() - 2048.0).abs() < 1e-9);
    // Half of bucket 10's samples: halfway through it in log2, i.e. 2^10.5.
    assert!((escape_quantile(&h, 0.45).unwrap() - 2f64.powf(10.5)).abs() < 1e-6);
    // The whole picture reaches the top of the stragglers' bucket.
    assert!((escape_quantile(&h, 1.0).unwrap() - 16384.0).abs() < 1e-9);
    assert_eq!(escape_quantile(&[0u32; 24], 0.9), None);
    assert_eq!(escape_quantile(&h, 0.0), None);
    assert_eq!(escape_quantile(&h, 1.5), None);
}

/// `MOTION_NEED_QUANTILE` at 1 (the default), no histogram or no range: the top of the range,
/// exactly the shipped sizing. Below 1: the quantile, never above the top.
#[test]
fn a_moving_frame_needs_the_top_unless_asked_for_less() {
    let mut h = [0u32; 24];
    h[10] = 90;
    h[13] = 10;
    let hi = 12_651.0;
    assert_eq!(motion_need(hi, Some(&h), 1.0), hi);
    assert_eq!(motion_need(hi, None, 0.9), hi);
    assert_eq!(motion_need(0.0, Some(&h), 0.9), 0.0);
    assert!((motion_need(hi, Some(&h), 0.9) - 2048.0).abs() < 1e-9);
    // A quantile above the measured top is capped at the top.
    assert_eq!(motion_need(1500.0, Some(&h), 0.9), 1500.0);
}
