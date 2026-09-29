use super::*;
use crate::tunables::{
    LIVE_MIN_SCALE, LIVE_PRICE_FRESH_FRAMES, LIVE_PRICE_MAX_OCT, LIVE_PRICE_STALE_OCT, LIVE_PROBE_MIN_SCALE,
    LIVE_REFRESH_MS, LIVE_REPROBE_MS, LIVE_SPLIT_MAX,
};
use crate::{PriceView, RefreshPrice};

const DF32: u32 = RenderMode::Df32Pert as u32;
const DIRECT: u32 = RenderMode::Direct as u32;
const FE: u32 = RenderMode::Floatexp as u32;

/// A view of `mode`, `nav`, `l2`, at a fixed 1 Mpx (so prices here are the frame's own size).
fn at(mode: u32, nav: u64, l2: f64) -> PriceView {
    PriceView { mode, nav, l2, px: 1_000_000 }
}

fn price(ns: f64, frame: u64, a: PriceView) -> Option<RefreshPrice> {
    Some(RefreshPrice { ns, frame, at: a })
}

/// The ns per nominal step that prices `steps` at `ms`.
fn ns_for(ms: f64, steps: u64) -> f64 {
    ms * 1.0e6 / steps as f64
}

/// A frame at `now` asking `steps`, at frame 101, no probe allowed — so with no price the answer
/// is `No` — unfitted, under "prefer detail", with a complete frame to hold, on an adapter with no
/// known occupancy knee.
fn ask(now: PriceView, steps: u64) -> LiveAsk {
    LiveAsk {
        now,
        steps,
        frame: 101,
        probe_after: u64::MAX,
        probe_cap: u64::MAX,
        fit: 1.0,
        prefer_detail: true,
        can_split: true,
        knee_px: 0,
    }
}

fn verdict(p: Option<RefreshPrice>, a: LiveAsk) -> LiveRefresh {
    live_refresh_verdict(p, &a).0
}

#[test]
fn a_fresh_cheap_price_for_this_view_renders_live_every_frame() {
    // 0.2e-3 ns per nominal step × 1.6e10 nominal = 3.2 ms: one pass, live.
    let v = at(DF32, 3, 40.0);
    let (r, pred, scale) = live_refresh_verdict(price(0.2e-3, 100, v), &ask(v, 16_000_000_000));
    assert_eq!(r, LiveRefresh::Priced);
    assert!((pred - 3.2).abs() < 1e-9, "{pred}");
    assert_eq!(scale, 1.0);
}

#[test]
fn a_dearer_refresh_renders_in_split_passes_of_at_most_the_share() {
    let v = at(DF32, 3, 40.0);
    let steps = 1_000_000_000;
    // (Off the exact multiples: a price rounds, and 2.0× the share may read as 2.0000000000000004.)
    for (ms, k) in [(1.01, 2), (1.5, 2), (1.99, 2), (2.5, 3), (7.9, 8)] {
        let p = price(ns_for(ms * LIVE_REFRESH_MS, steps), 100, v);
        assert_eq!(verdict(p, ask(v, steps)), LiveRefresh::Split(k), "{ms}× the share");
    }
    // Past LIVE_SPLIT_MAX passes: the hold and the pinned walk, and no probe second-guessing a
    // fresh measurement that says so.
    let past = LIVE_SPLIT_MAX as f64 * LIVE_REFRESH_MS * 1.01;
    let p = price(ns_for(past, steps), 100, v);
    assert_eq!(verdict(p, LiveAsk { probe_after: 0, ..ask(v, steps) }), LiveRefresh::No);
    // With nothing complete to hold while the sets compose, no split.
    let p = price(ns_for(2.0 * LIVE_REFRESH_MS, steps), 100, v);
    assert_eq!(verdict(p, LiveAsk { can_split: false, ..ask(v, steps) }), LiveRefresh::No);
}

#[test]
fn split_passes_are_the_share_rounded_up() {
    assert_eq!(split_passes(0.0), 1);
    assert_eq!(split_passes(LIVE_REFRESH_MS), 1);
    assert_eq!(split_passes(LIVE_REFRESH_MS * 1.0001), 2);
    assert_eq!(split_passes(LIVE_REFRESH_MS * 3.0), 3);
    for bad in [f64::NAN, f64::INFINITY, -1.0] {
        assert_eq!(split_passes(bad), 1, "{bad}");
    }
}

#[test]
fn without_prefer_detail_a_dearer_refresh_renders_every_frame_shrunk_to_the_share() {
    let v = at(DF32, 3, 40.0);
    let steps = 1_000_000_000;
    let p = price(ns_for(1.5 * LIVE_REFRESH_MS, steps), 100, v);
    let a = LiveAsk { prefer_detail: false, ..ask(v, steps) };
    // Live every frame: shrunk so its steps cost the share.
    let (r, _, scale) = live_refresh_verdict(p, &a);
    assert_eq!(r, LiveRefresh::Priced);
    assert!((scale * scale * 1.5 - 1.0).abs() < 1e-9, "{scale}");
    // …but never past the floor, counting the ceiling fit already taken: then it splits.
    let fitted = LiveAsk { fit: LIVE_MIN_SCALE / scale * 0.99, ..a };
    assert_eq!(live_refresh_verdict(p, &fitted).0, LiveRefresh::Split(2));
    // A refresh within the share is never shrunk.
    let cheap = price(ns_for(0.5 * LIVE_REFRESH_MS, steps), 100, v);
    assert_eq!(live_refresh_verdict(cheap, &a), (LiveRefresh::Priced, 0.5 * LIVE_REFRESH_MS, 1.0));
}

#[test]
fn a_price_is_only_about_its_own_mode_and_navigation_epoch() {
    let v = at(DF32, 3, 40.0);
    let p = price(0.2e-3, 100, v);
    let steps = 1_000_000_000;
    // Another render mode (either way across a switch) or another navigation epoch (a click-zoom,
    // a go-to): no price at all, so with no probe allowed, No.
    for now in [at(FE, 3, 40.0), at(DIRECT, 3, 40.0), at(DF32, 4, 40.0)] {
        assert_eq!(verdict(p, ask(now, steps)), LiveRefresh::No, "{now:?}");
    }
    // Past the stale window it prices nothing either.
    let far = at(DF32, 3, 40.0 + LIVE_PRICE_STALE_OCT + 0.1);
    assert_eq!(verdict(p, ask(far, steps)), LiveRefresh::No);
}

#[test]
fn a_stale_price_licenses_only_a_frame_at_the_share_or_a_spaced_reprobe() {
    // Stale by age or by depth, a price may still license a frame it prices within ONE displayed
    // frame's share — never a split on its own say-so, which needs a fresh one.
    let v = at(DF32, 3, 40.0);
    let steps = 1_000_000_000;
    let old = 100 + LIVE_PRICE_FRESH_FRAMES + 1;
    let deeper = at(DF32, 3, 40.0 + 2.0 * LIVE_PRICE_MAX_OCT);
    for (now, frame) in [(v, old), (deeper, 101)] {
        let cheap = price(ns_for(0.5 * LIVE_REFRESH_MS, steps), 100, v);
        let dear = price(ns_for(1.5 * LIVE_REFRESH_MS, steps), 100, v);
        let a = LiveAsk { frame, ..ask(now, steps) };
        assert_eq!(verdict(cheap, a), LiveRefresh::Priced, "{now:?} {frame}");
        // Dear and stale: the hold — until a probe's turn comes, which re-measures it at full
        // size in split passes (the cost moves with the reference, not only the depth)…
        assert_eq!(verdict(dear, a), LiveRefresh::No, "{now:?} {frame}");
        assert_eq!(verdict(dear, LiveAsk { probe_after: 0, ..a }), LiveRefresh::Reprobe(2), "{now:?} {frame}");
        // …in no more than LIVE_SPLIT_MAX of them…
        let dearer = price(ns_for(0.9 * LIVE_REPROBE_MS, steps), 100, v);
        assert_eq!(
            verdict(dearer, LiveAsk { probe_after: 0, ..a }),
            LiveRefresh::Reprobe(LIVE_SPLIT_MAX),
            "{now:?} {frame}"
        );
        // …or, past the reprobe bound, with a shrunk probe — and past LIVE_HOPE_X × the 36 ms this
        // frame could render live at (72 ms), nothing.
        let very = price(ns_for(1.4 * LIVE_REPROBE_MS, steps), 100, v);
        assert_eq!(verdict(very, LiveAsk { probe_after: 0, ..a }), LiveRefresh::Probe, "{now:?} {frame}");
        let hopeless = price(ns_for(1.6 * LIVE_REPROBE_MS, steps), 100, v);
        assert_eq!(verdict(hopeless, LiveAsk { probe_after: 0, ..a }), LiveRefresh::No, "{now:?} {frame}");
    }
    // Within the fresh windows the same dear price splits.
    let dear = price(ns_for(1.5 * LIVE_REFRESH_MS, steps), 100, v);
    assert_eq!(verdict(dear, ask(v, steps)), LiveRefresh::Split(2));
}

#[test]
fn a_price_from_a_much_smaller_frame_only_bounds_this_one() {
    // The mode switch, measured: a 0.4-scale probe priced the full frame at ~33 ms; full frames
    // then ran at ~7. Fresh, it is still only a bound, so it earns a full-size reprobe (in its turn)
    // rather than holding the glide on the pinned walk.
    let v = at(DF32, 3, 13.4);
    let probe_px = (v.px as f64 * 0.16) as u64;
    let steps = 8_700_000_000;
    let p = price(ns_for(33.0, steps), 100, PriceView { px: probe_px, ..v });
    assert_eq!(verdict(p, ask(v, steps)), LiveRefresh::No);
    assert_eq!(
        verdict(p, LiveAsk { probe_after: 0, ..ask(v, steps) }),
        LiveRefresh::Reprobe(split_passes(33.0).min(LIVE_SPLIT_MAX))
    );
    // The same price measured on a frame this size is the frame's own, and it splits.
    let own = price(ns_for(33.0, steps), 100, v);
    assert_eq!(verdict(own, ask(v, steps)), LiveRefresh::Split(split_passes(33.0)));
    // A frame within LIVE_PRICE_SIZE_MIN of this one's pixels counts as this size.
    let near = (v.px as f64 * crate::tunables::LIVE_PRICE_SIZE_MIN) as u64 + 1;
    let close = price(ns_for(33.0, steps), 100, PriceView { px: near, ..v });
    assert_eq!(verdict(close, ask(v, steps)), LiveRefresh::Split(split_passes(33.0)));
}

#[test]
fn with_no_price_a_probe_needs_its_turn() {
    let v = at(DF32, 0, 20.0);
    let a = LiveAsk { frame: 10, probe_after: 11, ..ask(v, 1_000_000) };
    assert_eq!(verdict(None, a), LiveRefresh::No);
    assert_eq!(verdict(None, LiveAsk { frame: 11, ..a }), LiveRefresh::Probe);
    // `u64::MAX` = never (no GPU timestamps to price a probe with).
    assert_eq!(verdict(None, LiveAsk { frame: u64::MAX - 1, probe_after: u64::MAX, ..a }), LiveRefresh::No);
    // A price for ANOTHER mode is no price for this one: the probe still runs — the mode switch.
    let direct = price(0.2e-3, 9, at(DIRECT, 0, 13.2));
    assert_eq!(verdict(direct, LiveAsk { frame: 11, ..a }), LiveRefresh::Probe);
}

#[test]
fn a_probe_its_cap_would_shrink_to_nothing_is_not_run() {
    // Right after a mode switch the learned budget is the 4e6 bootstrap: fitting a 1.6e9-step probe
    // under it renders it at 5 % of its size, which measures the pass's overhead, not the frame.
    let v = at(DF32, 0, 20.0);
    let a = LiveAsk { probe_after: 0, ..ask(v, 1_600_000_000) };
    assert_eq!(verdict(None, a), LiveRefresh::Probe);
    assert_eq!(verdict(None, LiveAsk { probe_cap: 4_000_000, ..a }), LiveRefresh::No);
    assert_eq!(verdict(None, LiveAsk { probe_cap: 0, ..a }), LiveRefresh::No);
    // At the floor it still probes.
    let at_floor = (1_600_000_000.0 * LIVE_PROBE_MIN_SCALE * LIVE_PROBE_MIN_SCALE * 1.001) as u64;
    assert_eq!(verdict(None, LiveAsk { probe_cap: at_floor, ..a }), LiveRefresh::Probe);
}

#[test]
fn nonsense_never_licenses_a_frame() {
    let v = at(DF32, 0, 20.0);
    for bad in [f64::NAN, f64::INFINITY, -1.0, 0.0] {
        assert_eq!(verdict(price(bad, 100, v), ask(v, 1)), LiveRefresh::No, "{bad}");
    }
}

#[test]
fn a_live_frame_fits_the_ceiling_but_never_shrinks_past_the_floor() {
    let f = LIVE_MIN_SCALE;
    assert_eq!(live_fit(100, 100, f), Some(1.0));
    assert_eq!(live_fit(50, 100, f), Some(1.0));
    // Over the cap: the linear scale that fits it (steps go as its square)…
    let cap = 1_000_000u64;
    let s = live_fit(cap * 5 / 4, cap, f).unwrap();
    assert!((s - 0.8f64.sqrt()).abs() < 1e-12, "{s}");
    // …down to the floor and no further.
    let edge = cap as f64 / (f * f);
    assert!(live_fit((edge * 0.999) as u64, cap, f).is_some());
    assert_eq!(live_fit((edge * 1.001) as u64, cap, f), None);
}

#[test]
fn a_split_pass_carries_its_share_of_the_frame_steps() {
    // The count a split pass sends to the GPU and the one the price ring records are ONE formula.
    let full = full_pass_steps(1_000_000, 1, 5_000, 1);
    assert_eq!(full, 5_000_000_000);
    assert_eq!(full_pass_steps(1_000_000, 1, 5_000, 0), full, "0 = unsplit");
    assert_eq!(full_pass_steps(1_000_000, 1, 5_000, 4), full / 4);
    assert_eq!(full_pass_steps(1_000_000, 2, 5_000, 1), full * 4, "ss² counts");
}

#[test]
fn a_probe_may_carry_what_the_walk_already_dispatches_but_only_at_or_over_the_knee() {
    // The RX 6800 XT (knee 524,288 px) just past the df32 switch of the PLUTO glide: 6,078
    // iterations. Under a pass bound of 2.63e9 even a knee-sized probe (3.19e9 steps' worth of
    // worst case) is over it, and a smaller one costs the same: none.
    let knee = 524_288u64;
    assert_eq!(probe_walk_bound(2_630_000_000, 6_078, knee), 0);
    // One reading later the walk's bound is 3.9e9: a probe of up to that many steps (641k px) runs.
    assert_eq!(probe_walk_bound(3_900_000_000, 6_078, knee), 3_900_000_000);
    // Exactly at the knee counts; no knee term (an unknown adapter's 0) bounds by the steps alone.
    assert_eq!(probe_walk_bound(knee * 6_078, 6_078, knee), knee * 6_078);
    assert_eq!(probe_walk_bound(1_000, 6_078, 0), 1_000);
    // A 0 iteration count is one iteration, not a free pass.
    assert_eq!(probe_walk_bound(knee - 1, 0, knee), 0);
}

#[test]
fn a_frame_that_stays_off_live_says_why() {
    let v = at(DF32, 3, 40.0);
    let steps = 1_000_000_000;
    // No price: no timestamps, the probe's turn not yet come, or a probe too small to run.
    assert_eq!(live_no_reason(None, &ask(v, steps)), LiveNo::NoTiming);
    let waits = LiveAsk { probe_after: 200, ..ask(v, steps) };
    assert_eq!(verdict(None, waits), LiveRefresh::No);
    assert_eq!(live_no_reason(None, &waits), LiveNo::ProbeWaits);
    let tiny = LiveAsk { probe_after: 0, probe_cap: steps / 100, ..ask(v, steps) };
    assert_eq!(verdict(None, tiny), LiveRefresh::No, "a 0.1-scale probe measures nothing");
    assert_eq!(live_no_reason(None, &tiny), LiveNo::ProbeTooSmall);
    // A price for another mode is no price.
    let other = price(ns_for(1.0, steps), 100, at(DIRECT, 3, 40.0));
    assert_eq!(live_no_reason(other, &waits), LiveNo::ProbeWaits);
    // Fresh and past the split limit, or only a bound too dear to act on.
    let dear = price(ns_for(LIVE_SPLIT_MAX as f64 * LIVE_REFRESH_MS * 1.5, steps), 100, v);
    assert_eq!(verdict(dear, ask(v, steps)), LiveRefresh::No);
    assert_eq!(live_no_reason(dear, &ask(v, steps)), LiveNo::MeasuredDear);
    let stale = price(ns_for(LIVE_REPROBE_MS * 1.5, steps), 100 - LIVE_PRICE_FRESH_FRAMES - 5, v);
    assert_eq!(verdict(stale, ask(v, steps)), LiveRefresh::No);
    assert_eq!(live_no_reason(stale, &ask(v, steps)), LiveNo::BoundTooDear);
}

#[test]
fn a_split_is_no_finer_than_brings_a_set_down_to_the_occupancy_knee() {
    // RX 6800 XT (knee 524,288) at 1280×735: halves already sit under the knee.
    assert_eq!(finest_split(940_800, 524_288), 2);
    // RTX 3080 (knee 262,144) at 1457×1102: sevenths; eighths cost what sevenths do.
    assert_eq!(finest_split(1457 * 1102, 262_144), 7);
    // A frame under the knee cannot split at all; a huge one stops at LIVE_SPLIT_MAX; no knee known,
    // LIVE_SPLIT_MAX.
    assert_eq!(finest_split(100_000, 262_144), 1);
    assert_eq!(finest_split(100_000_000, 262_144), LIVE_SPLIT_MAX);
    assert_eq!(finest_split(100_000, 0), LIVE_SPLIT_MAX);
    assert_eq!(finest_split(0, 262_144), 1);
    assert_eq!(finest_split(u64::MAX, 262_144), LIVE_SPLIT_MAX);
}

#[test]
fn a_split_set_is_priced_with_the_knee_counted() {
    use crate::tunables::LIVE_SPLIT_SET_MAX_MS;
    // The RTX 3080 just past the df32 switch (1457×1102, 27–32 ms frames): sevenths at the knee's
    // floor, 0.163 of the frame, ~5 ms a set — as 0.3.0-beta.3 drew them.
    let px = 1457 * 1102;
    assert_eq!(split_sets(30.0, px, 262_144), Some(7));
    assert_eq!(split_sets(36.0, px, 262_144), Some(7), "eighths cost what sevenths do");
    // The RX 6800 XT (1262×724, 15–27 ms frames): halves under its 524k knee cost ~0.57 of the
    // frame, over LIVE_SPLIT_SET_MAX_MS — the walk. A 9 ms frame halves at ~5 ms a set.
    assert_eq!(split_sets(15.0, 1262 * 724, 524_288), None);
    assert_eq!(split_sets(9.0, 1262 * 724, 524_288), Some(2));
    // With no knee known, the plain share rule: ⌈pred / share⌉ sets.
    assert_eq!(split_sets(2.5 * LIVE_REFRESH_MS, px, 0), Some(3));
    // No split needed, or past LIVE_SPLIT_MAX, or a frame under the knee: none.
    assert_eq!(split_sets(LIVE_REFRESH_MS, px, 262_144), None);
    assert_eq!(split_sets(LIVE_SPLIT_MAX as f64 * LIVE_REFRESH_MS * 1.01, px, 0), None);
    assert_eq!(split_sets(9.0, 200_000, 262_144), None);
    // Halves at a 0.75 floor: exactly at the set bound, in; a hair over it, out.
    assert_eq!(split_sets(2.0 * LIVE_REFRESH_MS, 400_000, 300_000), Some(2));
    assert!((2.0 * LIVE_REFRESH_MS * 0.75 - LIVE_SPLIT_SET_MAX_MS).abs() < 1e-12);
    assert_eq!(split_sets(2.0 * LIVE_REFRESH_MS, 400_000, 301_000), None);
}

#[test]
fn a_refresh_whose_sets_the_knee_keeps_dear_takes_the_walk() {
    let v = at(DF32, 3, 40.0); // 1 Mpx
    let steps = 1_000_000_000;
    let p = price(ns_for(2.5 * LIVE_REFRESH_MS, steps), 100, v); // 11.25 ms: 3 passes
    // Thirds of 1 Mpx over a 300k knee cost their share, 3.75 ms: a split.
    let a = LiveAsk { knee_px: 300_000, ..ask(v, steps) };
    assert_eq!(verdict(p, a), LiveRefresh::Split(3));
    // Under a 700k knee only halves help, and a half costs 0.7 of the frame, 7.9 ms: the walk,
    // and the trace says why.
    let a = LiveAsk { knee_px: 700_000, ..ask(v, steps) };
    assert_eq!(verdict(p, a), LiveRefresh::No);
    assert_eq!(live_no_reason(p, &a), LiveNo::UnderKnee);
    // Past LIVE_SPLIT_MAX it is dear whatever the knee.
    let dear = price(ns_for(LIVE_SPLIT_MAX as f64 * LIVE_REFRESH_MS * 1.5, steps), 100, v);
    assert_eq!(live_no_reason(dear, &a), LiveNo::MeasuredDear);
}

#[test]
fn a_shrunk_frame_stays_at_or_over_the_knee() {
    let v = at(DF32, 3, 40.0); // 1 Mpx
    let steps = 1_000_000_000;
    // 1.5× the share: shrunk to √(1/1.5) = 0.816 per axis, 667k px.
    let p = price(ns_for(1.5 * LIVE_REFRESH_MS, steps), 100, v);
    let a = LiveAsk { prefer_detail: false, knee_px: 600_000, ..ask(v, steps) };
    let (r, _, scale) = live_refresh_verdict(p, &a);
    assert_eq!(r, LiveRefresh::Priced);
    assert!((scale - (1.0f64 / 1.5).sqrt()).abs() < 1e-9, "{scale}");
    // Under a 700k knee the shrunk frame would cost what one at the knee does; halves, priced
    // with the knee, fit (4.7 ms a set).
    let a = LiveAsk { knee_px: 700_000, ..a };
    assert_eq!(verdict(p, a), LiveRefresh::Split(2));
    // A frame under the knee neither shrinks nor splits usefully: the walk.
    let a = LiveAsk { knee_px: 1_200_000, ..a };
    assert_eq!(verdict(p, a), LiveRefresh::No);
    assert_eq!(live_no_reason(p, &a), LiveNo::UnderKnee);
}

#[test]
fn the_dearest_live_frame_is_the_largest_price_a_split_accepts() {
    // No knee in the way (or none known), and the RTX 3080's 1.6 Mpx frame: 8 × the share, 36 ms.
    assert_eq!(live_max_ms(1_000_000, 0), LIVE_SPLIT_MAX as f64 * LIVE_REFRESH_MS);
    assert_eq!(live_max_ms(1457 * 1102, 262_144), LIVE_SPLIT_MAX as f64 * LIVE_REFRESH_MS);
    // The RX 6800 XT at 1262×724: halves at the knee's floor, ~11.8 ms.
    let radeon = live_max_ms(1262 * 724, 524_288);
    assert!((radeon - 11.76).abs() < 0.01, "{radeon}");
    // Under the knee: only a frame within the share.
    assert_eq!(live_max_ms(200_000, 262_144), LIVE_REFRESH_MS);
    // It is exactly split_sets' edge.
    for (px, knee) in [(1262 * 724, 524_288), (1457 * 1102, 262_144), (400_000, 300_000)] {
        let m = live_max_ms(px, knee);
        assert!(split_sets(m * 0.999, px, knee).is_some(), "{px} {knee}");
        assert!(split_sets(m * 1.001, px, knee).is_none(), "{px} {knee}");
    }
}

#[test]
fn a_bound_the_adapter_cannot_render_live_is_not_re_measured() {
    // The RX 6800 XT at 1262×724 (knee 524k): a stale 35 ms price. With nothing over ~11.8 ms able
    // to go live there, neither a reprobe nor a probe — the walk, until the price lapses.
    let px = 1262 * 724;
    let v = PriceView { px, ..at(DF32, 3, 40.0) };
    let steps = 8_000_000_000;
    let a = LiveAsk { probe_after: 0, frame: 100 + LIVE_PRICE_FRESH_FRAMES + 1, knee_px: 524_288, ..ask(v, steps) };
    let stale = |ms: f64| price(ns_for(ms, steps), 100, v);
    assert_eq!(verdict(stale(35.0), a), LiveRefresh::No);
    assert_eq!(live_no_reason(stale(35.0), &a), LiveNo::BoundTooDear);
    // Within 4/3 of its reach, a reprobe in halves; within LIVE_HOPE_X of it (23.5 ms), a probe.
    assert_eq!(verdict(stale(15.0), a), LiveRefresh::Reprobe(2));
    assert_eq!(verdict(stale(20.0), a), LiveRefresh::Probe);
    assert_eq!(verdict(stale(25.0), a), LiveRefresh::No);
    // The same stale price on the RTX 3080's frame earns its reprobe as before.
    let v = PriceView { px: 1457 * 1102, ..v };
    let a = LiveAsk { now: v, knee_px: 262_144, ..a };
    assert_eq!(verdict(price(ns_for(35.0, steps), 100, v), a), LiveRefresh::Reprobe(7));
}

#[test]
fn a_view_proved_hopeless_is_not_probed_again_when_its_price_lapses() {
    // The RX 6800 XT at 1262×724 (live max ~11.8 ms, hope 23.5): a probe measured the glide's view
    // at 35 ms, and the zoom has since gone past LIVE_PRICE_STALE_OCT — no usable price.
    let px = 1262 * 724;
    let measured = PriceView { px, ..at(DF32, 3, 20.0) };
    let now = PriceView { l2: 20.0 + LIVE_PRICE_STALE_OCT + 0.5, ..measured };
    let steps = 8_000_000_000;
    let a = LiveAsk { probe_after: 0, knee_px: 524_288, ..ask(now, steps) };
    let p = |ms: f64, at: PriceView| price(ns_for(ms, steps), 100, at);
    let (r, pred, _) = live_refresh_verdict(p(35.0, measured), &a);
    assert_eq!(r, LiveRefresh::No);
    assert!((pred - 35.0).abs() < 1e-6, "{pred}");
    assert_eq!(live_no_reason(p(35.0, measured), &a), LiveNo::BackedOff);
    // A price that left hope (20 ms), another navigation epoch's, or another mode's: a probe.
    assert_eq!(verdict(p(20.0, measured), a), LiveRefresh::Probe);
    assert_eq!(verdict(p(35.0, PriceView { nav: 2, ..measured }), a), LiveRefresh::Probe);
    assert_eq!(verdict(p(35.0, PriceView { mode: DIRECT, ..measured }), a), LiveRefresh::Probe);
    // The RTX 3080's frame could render 36 ms live: its 55 ms view keeps its probes.
    let big = PriceView { px: 1457 * 1102, ..measured };
    let a3080 = LiveAsk { now: PriceView { px: big.px, ..now }, knee_px: 262_144, ..a };
    assert_eq!(verdict(p(55.0, big), a3080), LiveRefresh::Probe);
    assert_eq!(verdict(p(80.0, big), a3080), LiveRefresh::No);
}

#[test]
fn one_reading_cannot_make_the_view_much_cheaper_but_any_can_make_it_dearer() {
    use crate::tunables::LIVE_PRICE_DROP_MAX;
    let v = at(DF32, 3, 40.0);
    // The RX 6800 XT's 2-set reprobe: 14.63 ms for the first set, 0.26 for the second (4e9 steps).
    let old = price(ns_for(14.63, 4_038_662_375), 698, v);
    let bogus = ns_for(0.26, 4_038_662_375);
    let got = bounded_price_drop(old, bogus, v);
    assert!((got - old.unwrap().ns / LIVE_PRICE_DROP_MAX).abs() < 1e-15, "{got}");
    // Dearer, or a drop inside the bound: as read.
    let dear = old.unwrap().ns * 3.0;
    assert_eq!(bounded_price_drop(old, dear, v), dear);
    let near = old.unwrap().ns / 2.0;
    assert_eq!(bounded_price_drop(old, near, v), near);
    // A real 4.6× drop (a small probe's price, then the full frame) lands in two readings.
    let full = old.unwrap().ns / 4.6;
    let first = bounded_price_drop(old, full, v);
    assert!(first > full);
    assert_eq!(bounded_price_drop(price(first, 699, v), full, v), full);
    // Against no price, or another mode's, another epoch's, or one octaves away: as read.
    assert_eq!(bounded_price_drop(None, bogus, v), bogus);
    assert_eq!(bounded_price_drop(old, bogus, at(DIRECT, 3, 40.0)), bogus);
    assert_eq!(bounded_price_drop(old, bogus, at(DF32, 4, 40.0)), bogus);
    assert_eq!(bounded_price_drop(old, bogus, at(DF32, 3, 40.0 + LIVE_PRICE_STALE_OCT * 1.01)), bogus);
}

#[test]
fn a_reprobe_splits_no_finer_than_the_knee_and_one_pass_needs_no_hold() {
    // A price from a 0.4 Mpx probe only bounds a 1 Mpx frame: 11.25 ms, 3 passes.
    let v = at(DF32, 3, 40.0);
    let steps = 1_000_000_000;
    let p = price(ns_for(2.5 * LIVE_REFRESH_MS, steps), 100, PriceView { px: 400_000, ..v });
    let a = LiveAsk { probe_after: 0, ..ask(v, steps) };
    assert_eq!(verdict(p, a), LiveRefresh::Reprobe(3));
    assert_eq!(verdict(p, LiveAsk { knee_px: 400_000, ..a }), LiveRefresh::Reprobe(3));
    assert_eq!(verdict(p, LiveAsk { knee_px: 600_000, ..a }), LiveRefresh::Reprobe(2));
    assert_eq!(verdict(p, LiveAsk { can_split: false, ..a }), LiveRefresh::Probe);
    // A frame under the knee renders live only within the share, so its reprobe bound is 4/3 of
    // that (6 ms) and its hope twice it (9 ms): 11.25 ms earns nothing, 7.2 ms a probe, 5.4 ms one
    // pass, which needs no held frame.
    let one = LiveAsk { knee_px: 1_200_000, ..a };
    assert_eq!(verdict(p, one), LiveRefresh::No);
    let mid = price(ns_for(1.6 * LIVE_REFRESH_MS, steps), 100, PriceView { px: 400_000, ..v });
    assert_eq!(verdict(mid, one), LiveRefresh::Probe);
    let near = price(ns_for(1.2 * LIVE_REFRESH_MS, steps), 100, PriceView { px: 400_000, ..v });
    assert_eq!(verdict(near, one), LiveRefresh::Reprobe(1));
    assert_eq!(verdict(near, LiveAsk { can_split: false, ..one }), LiveRefresh::Reprobe(1));
}
