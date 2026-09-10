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
