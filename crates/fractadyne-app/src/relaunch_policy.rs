use super::relaunch_decision;

/// The device-lost handler writes the crashing view as a `.fdn` beside the crash report (the
/// manifest omits the coordinates, which is what made the 2026-09-10 field loss unreproducible).
/// This pins that the formatted view is actually a LOADABLE location — `center_re`/`center_im` and
/// the iteration count survive, and the app's own location gate accepts it — so a future device
/// loss becomes a reproducible case rather than a lost one.
#[test]
fn crash_view_fdn_is_a_loadable_location() {
    *super::CRASH_VIEW.lock().unwrap() = Some(super::CrashView {
        fractal: super::FractalKind::Mandelbrot,
        custom: None,
        julia: false,
        julia_c: (0.0, 0.0),
        cx: fractadyne_core::parse_bf("-0.7436438870371588707780645434936425750476").unwrap(),
        cy: fractadyne_core::parse_bf("0.1318259042053122928210973548747672652630").unwrap(),
        upp_log2: -55.0,
        log2mag: 54.0,
        max_iter: 10_000_000,
        auto_iter: false,
    });
    let fdn = super::crash_view_fdn().expect("a view was stashed");
    assert!(super::location_text_verdict(&fdn).is_ok(), "not accepted as a location:\n{fdn}");
    assert!(fdn.contains("max_iter=10000000"), "iteration count missing:\n{fdn}");
    // The centre round-trips (notation-independent — `to_decimal_string` writes scientific form for
    // |x| < 1, e.g. `-7.436…e-1`), which is what makes the crashing spot recoverable.
    let cre = fdn.lines().find_map(|l| l.strip_prefix("center_re=")).expect("center_re present");
    let v = fractadyne_core::parse_bf(cre).expect("center_re parses");
    assert!(
        (fractadyne_core::to_f64(&v) - (-0.7436438870371589)).abs() < 1e-12,
        "centre did not round-trip: {cre}"
    );
    assert!(!fdn.contains("formula="), "a built-in view carries no formula:\n{fdn}");

    // A Custom view carries its formula (same test: `CRASH_VIEW` is one global, and a second test
    // writing it in parallel would race this one).
    let custom = crate::custom_formula::CustomFormula::compile("t = sqr(z)\nz = t + p1*z + c", &[(0.5, -0.25)])
        .expect("compiles");
    *super::CRASH_VIEW.lock().unwrap() = Some(super::CrashView {
        fractal: super::FractalKind::Custom,
        custom: Some(std::sync::Arc::new(custom)),
        julia: false,
        julia_c: (0.0, 0.0),
        cx: fractadyne_core::parse_bf("-0.25").unwrap(),
        cy: fractadyne_core::parse_bf("0").unwrap(),
        upp_log2: -8.0,
        log2mag: 0.5,
        max_iter: 500,
        auto_iter: false,
    });
    let fdn = super::crash_view_fdn().expect("a view was stashed");
    assert!(super::location_text_verdict(&fdn).is_ok(), "not accepted as a location:\n{fdn}");
    assert!(fdn.contains("fractal=Custom\n"), "{fdn}");
    assert!(fdn.contains("formula=t = sqr(z)\\nz = t + p1*z + c\n"), "the formula, one escaped line:\n{fdn}");
    assert!(fdn.contains("formula_params=0.5,-0.25\n"), "{fdn}");
}

#[test]
fn a_first_loss_recovers_at_any_uptime() {
    // THE FIELD CASE (2026-08-18): a deep view + Home glide lost the device at 50.4s. The old
    // `elapsed_s() > 60` guard refused to restart, so a loss the app is designed to recover
    // from was experienced as a hard crash. 50.4s must restart.
    assert_eq!(relaunch_decision(0, 50.4), Some(1));
    // And the guard must not have simply moved: an immediate first loss still recovers once,
    // because one relaunch is cheap and the generation cap is what bounds a loop.
    assert_eq!(relaunch_decision(0, 0.2), Some(1));
    assert_eq!(relaunch_decision(0, 3600.0), Some(1));
}

#[test]
fn a_relaunch_that_did_not_help_stops() {
    // Restarted, then died again before it genuinely recovered: restarting is not working, so stop
    // rather than spin — "twice in a row, do not auto-restart again". ⚠The window is 600s, raised
    // from 15s after the 2026-09-10 field loop: each relaunch survived ~30-200s rebuilding the same
    // lethal reference (watchdog hangs of 100-160s) before dying, clearing 15s every time, so the
    // guard kept restarting into the identical crash.
    assert_eq!(relaunch_decision(1, 2.0), None);
    assert_eq!(relaunch_decision(1, 200.0), None); // the field loop's rebuild-and-recrash window
    assert_eq!(relaunch_decision(1, 599.9), None);
    assert_eq!(relaunch_decision(2, 14.9), None);
    // But a generation that ran a genuinely healthy stretch before a later (unrelated) loss gets
    // another go — transient recovery, not a loop.
    assert_eq!(relaunch_decision(1, 600.0), Some(2));
    assert_eq!(relaunch_decision(2, 3600.0), Some(3));
}

#[test]
fn the_generation_cap_terminates_any_loop() {
    // However healthy each generation looks, the chain is bounded — no restart loop can
    // outlive the cap even with long uptimes between losses.
    assert_eq!(relaunch_decision(3, 10_000.0), None);
    assert_eq!(relaunch_decision(9, 10_000.0), None);
    let mut gen = 0;
    let mut hops = 0;
    while let Some(next) = relaunch_decision(gen, 1_000.0) {
        gen = next;
        hops += 1;
        assert!(hops <= 8, "relaunch chain did not terminate");
    }
    assert_eq!(hops, 3, "the chain must stop after exactly MAX_GENERATIONS hops");
}

// ---- W7 (design/live-render-robustness.md §5.7): the iteration setting a relaunch comes back with.

use crate::session_outcome::{quarantine, Ask, SessionEnd, SessionOutcome};

fn outcome(end: SessionEnd, at_log2mag: f64, max_iter: u32, auto_iter: bool, admitted: u64) -> SessionOutcome {
    SessionOutcome { end, at_log2mag, ask: Ask { max_iter, auto_iter }, admitted_budget: Some(admitted), mode_rate: None }
}

/// The 2026-09-10 field loop: auto-iterations from a base of 10,000,000 at a deep minibrot
/// (≈1e81×, 269 octaves) lost the device; the relaunch opened at Home with the same setting and
/// ground again — every interior pixel of the cardioid iterated to 10,000,000.
const FIELD_LOOP: (f64, u32, bool) = (269.0, 10_000_000, true);

#[test]
fn a_session_that_lost_its_device_does_not_get_its_heavy_setting_back() {
    let (l2, n, auto) = FIELD_LOOP;
    let q = quarantine(&outcome(SessionEnd::DeviceLost, l2, n, auto, 400_000_000), Ask::default_ask())
        .expect("the field loop's setting must be set aside");
    assert_eq!(q.restored, Ask::default_ask());
    assert!(q.restored.at(l2) < q.outcome.ask.at(l2));
}

/// §7.7's gate, RED criterion: given a recorded outcome whose PRICE exceeded the budget at the loss —
/// the frame its setting asked for cost more than its controller would let one dispatch carry — the
/// relaunch must not come back at that price. Price = the frame's pixels × what the setting asks of
/// a pixel at the depth where the session ended. (Not "below the budget": the default's own frame
/// at depth costs more than one dispatch too, which is what the walk splits; the load to keep from
/// coming back is the setting the session died at.)
#[test]
fn the_restored_setting_is_priced_below_the_one_the_session_died_at() {
    const PX: u64 = 1676 * 1295; // the RX 6800 XT field geometry
    let cases = [
        // (end, depth, base, auto, admitted steps a dispatch at the last frame)
        (SessionEnd::DeviceLost, FIELD_LOOP.0, FIELD_LOOP.1, FIELD_LOOP.2, 400_000_000u64),
        (SessionEnd::Unclean, 0.0, 10_000_000, false, 4_000_000_000), // an explicit grind at Home
        (SessionEnd::Unclean, 400.0, 2_000_000, false, 1_500_000_000), // deep, explicit
        (SessionEnd::DeviceLost, 100.0, 250_000, false, 400_000_000), // `--autodive`'s field count
    ];
    for (end, l2, n, auto, admitted) in cases {
        let o = outcome(end, l2, n, auto, admitted);
        let lost = PX * o.ask.at(l2) as u64;
        assert!(lost > admitted, "a case that does not exceed its budget tests nothing: {o:?}");
        let restored = match quarantine(&o, Ask::default_ask()) {
            Some(q) => q.restored,
            None => o.ask,
        };
        let price = PX * restored.at(l2) as u64;
        assert!(
            price < lost,
            "RED: the relaunch would come back with {restored:?}, priced {price:.3e} steps, \
             not below the {lost:.3e} the session died at ({o:?})",
            price = price as f64,
            lost = lost as f64
        );
    }
}

#[test]
fn a_clean_session_keeps_its_setting_however_heavy() {
    let (l2, n, auto) = FIELD_LOOP;
    assert_eq!(quarantine(&outcome(SessionEnd::Clean, l2, n, auto, 1), Ask::default_ask()), None);
}

#[test]
fn a_setting_no_heavier_than_the_default_is_never_set_aside() {
    // The 2026-09-21 RX 6800 XT loss: an explicit 4,627 at 1.76e6× — under what the default asks
    // there (256 + 20.7 octaves × 220 = 4,810). Its cause was the escaped reference, not the
    // setting, and setting it aside would only surprise.
    let crash = 1.76e6f64.log2();
    assert_eq!(quarantine(&outcome(SessionEnd::DeviceLost, crash, 4_627, false, 1), Ask::default_ask()), None);
    // The default itself, and a light explicit count.
    assert_eq!(quarantine(&outcome(SessionEnd::Unclean, 300.0, 256, true, 1), Ask::default_ask()), None);
    assert_eq!(quarantine(&outcome(SessionEnd::Unclean, 10.0, 1_000, false, 1), Ask::default_ask()), None);
    // A poisoned depth asks as at Home.
    assert_eq!(quarantine(&outcome(SessionEnd::Unclean, f64::NAN, 200, false, 1), Ask::default_ask()), None);
    assert!(quarantine(&outcome(SessionEnd::Unclean, f64::NAN, 50_000, false, 1), Ask::default_ask()).is_some());
}

#[test]
fn the_notice_names_both_settings() {
    let (l2, n, auto) = FIELD_LOOP;
    let q = quarantine(&outcome(SessionEnd::DeviceLost, l2, n, auto, 400_000_000), Ask::default_ask()).unwrap();
    let t = q.notice();
    assert!(t.contains("lost its graphics device"), "{t}");
    assert!(t.contains("automatic iterations from a base of 10,000,000"), "{t}");
    assert!(t.contains("automatic iterations from a base of 256"), "{t}");
    let line = q.log_line();
    assert!(line.contains("set aside") && line.contains("4.000e8"), "{line}");
}
