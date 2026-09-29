use super::*;
use crate::tunables::{
    LIVE_MIN_SCALE, LIVE_PRICE_FRESH_FRAMES, LIVE_PRICE_MAX_OCT, LIVE_PRICE_STALE_OCT, LIVE_PROBE_MIN_SCALE,
    LIVE_REFRESH_MAX_MS, LIVE_REFRESH_MS,
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

/// A frame at `now` asking `steps`, at frame 101, the last live frame just before it, no probe
/// allowed — so with no price the answer is `No` — unfitted, under "prefer detail" (amortizing).
fn ask(now: PriceView, steps: u64) -> LiveAsk {
    LiveAsk {
        now,
        steps,
        frame: 101,
        since_live: 1,
        probe_after: u64::MAX,
        probe_cap: u64::MAX,
        fit: 1.0,
        amortize: true,
    }
}

fn verdict(p: Option<RefreshPrice>, a: LiveAsk) -> LiveRefresh {
    live_refresh_verdict(p, &a).0
}

#[test]
fn a_fresh_cheap_price_for_this_view_renders_live_every_frame() {
    // 0.2e-3 ns per nominal step × 1.6e10 nominal = 3.2 ms: live, even one frame after the last.
    let v = at(DF32, 3, 40.0);
    let (r, pred, scale) = live_refresh_verdict(price(0.2e-3, 100, v), &ask(v, 16_000_000_000));
    assert_eq!(r, LiveRefresh::Priced);
    assert!((pred - 3.2).abs() < 1e-9, "{pred}");
    assert_eq!(scale, 1.0);
}

#[test]
fn without_prefer_detail_a_dearer_refresh_renders_every_frame_shrunk_to_the_share() {
    let v = at(DF32, 3, 40.0);
    let steps = 1_000_000_000;
    let p = price(ns_for(1.5 * LIVE_REFRESH_MS, steps), 100, v);
    let a = LiveAsk { amortize: false, ..ask(v, steps) };
    // One frame after a live one, and live again: shrunk so its steps cost the share.
    let (r, _, scale) = live_refresh_verdict(p, &a);
    assert_eq!(r, LiveRefresh::Priced);
    assert!((scale * scale * 1.5 - 1.0).abs() < 1e-9, "{scale}");
    // …but never past the floor, counting the ceiling fit already taken: then it amortizes.
    let fitted = LiveAsk { fit: LIVE_MIN_SCALE / scale * 0.99, ..a };
    assert_eq!(live_refresh_verdict(p, &fitted).0, LiveRefresh::Wait);
    // A refresh within the share is never shrunk.
    let cheap = price(ns_for(0.5 * LIVE_REFRESH_MS, steps), 100, v);
    assert_eq!(live_refresh_verdict(cheap, &a), (LiveRefresh::Priced, 0.5 * LIVE_REFRESH_MS, 1.0));
}

#[test]
fn a_refresh_dearer_than_one_frame_renders_every_kth_frame_and_holds_between() {
    // Predicted 1.5 × LIVE_REFRESH_MS: one frame after a live one it waits (holds); two after, its
    // cost averages under the per-frame share and it renders.
    let v = at(DF32, 3, 40.0);
    let steps = 1_000_000_000;
    let p = price(ns_for(1.5 * LIVE_REFRESH_MS, steps), 100, v);
    assert_eq!(verdict(p, LiveAsk { since_live: 1, ..ask(v, steps) }), LiveRefresh::Wait);
    assert_eq!(verdict(p, LiveAsk { since_live: 2, ..ask(v, steps) }), LiveRefresh::Priced);
    // A long stretch without a live frame never waits.
    assert_eq!(verdict(p, LiveAsk { since_live: 1_000, ..ask(v, steps) }), LiveRefresh::Priced);
}

#[test]
fn past_the_single_frame_limit_it_holds_and_does_not_probe() {
    // Over LIVE_REFRESH_MAX_MS no amount of waiting licenses one pass that long — and a probe may
    // not second-guess a fresh measurement that says so.
    let v = at(DF32, 3, 40.0);
    let steps = 1_000_000_000;
    let p = price(ns_for(2.0 * LIVE_REFRESH_MAX_MS, steps), 100, v);
    let a = LiveAsk { since_live: 1_000, probe_after: 0, ..ask(v, steps) };
    assert_eq!(verdict(p, a), LiveRefresh::No);
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
fn a_stale_price_licenses_only_a_frame_at_the_strict_share() {
    // Stale by age or by depth, a price may still license a frame it prices within ONE displayed
    // frame's share — never the amortized Wait/Priced cadence, which needs a fresh one.
    let v = at(DF32, 3, 40.0);
    let steps = 1_000_000_000;
    let old = 100 + LIVE_PRICE_FRESH_FRAMES + 1;
    let deeper = at(DF32, 3, 40.0 + 2.0 * LIVE_PRICE_MAX_OCT);
    for (now, frame) in [(v, old), (deeper, 101)] {
        let cheap = price(ns_for(0.5 * LIVE_REFRESH_MS, steps), 100, v);
        let dear = price(ns_for(1.5 * LIVE_REFRESH_MS, steps), 100, v);
        let a = LiveAsk { frame, since_live: 1_000, ..ask(now, steps) };
        assert_eq!(verdict(cheap, a), LiveRefresh::Priced, "{now:?} {frame}");
        // Dear and stale: the hold — until a probe's turn comes, which re-measures it at full size
        // (the cost moves with the reference, not only the depth)…
        assert_eq!(verdict(dear, a), LiveRefresh::No, "{now:?} {frame}");
        assert_eq!(verdict(dear, LiveAsk { probe_after: 0, ..a }), LiveRefresh::Reprobe, "{now:?} {frame}");
        // …or, far past the reprobe bound, with a shrunk probe.
        let very = price(ns_for(2.0 * crate::tunables::LIVE_REPROBE_MS, steps), 100, v);
        assert_eq!(verdict(very, LiveAsk { probe_after: 0, ..a }), LiveRefresh::Probe, "{now:?} {frame}");
    }
    // Within the fresh windows the same dear price amortizes instead.
    let dear = price(ns_for(1.5 * LIVE_REFRESH_MS, steps), 100, v);
    assert_eq!(verdict(dear, LiveAsk { since_live: 2, ..ask(v, steps) }), LiveRefresh::Priced);
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
    assert_eq!(verdict(p, LiveAsk { probe_after: 0, ..ask(v, steps) }), LiveRefresh::Reprobe);
    // The same price measured on a frame this size is the frame's own: dear, so the hold.
    let own = price(ns_for(33.0, steps), 100, v);
    assert_eq!(verdict(own, LiveAsk { probe_after: 0, ..ask(v, steps) }), LiveRefresh::No);
    // A frame within LIVE_PRICE_SIZE_MIN of this one's pixels counts as this size.
    let near = (v.px as f64 * crate::tunables::LIVE_PRICE_SIZE_MIN) as u64 + 1;
    let close = price(ns_for(33.0, steps), 100, PriceView { px: near, ..v });
    assert_eq!(verdict(close, LiveAsk { probe_after: 0, ..ask(v, steps) }), LiveRefresh::No);
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
