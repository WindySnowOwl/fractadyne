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
    // Restarted, then died again within 15s: restarting is not working, so stop rather than
    // spin. This is the case the original uptime guard was really aiming at.
    assert_eq!(relaunch_decision(1, 2.0), None);
    assert_eq!(relaunch_decision(2, 14.9), None);
    // But a restarted generation that ran a while before dying gets another go.
    assert_eq!(relaunch_decision(1, 15.0), Some(2));
    assert_eq!(relaunch_decision(2, 600.0), Some(3));
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
