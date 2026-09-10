use super::relaunch_decision;

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
