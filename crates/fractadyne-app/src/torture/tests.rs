use super::*;

#[test]
fn every_rung_id_is_unique_and_well_formed() {
    let mut seen = std::collections::HashSet::new();
    for r in LADDER {
        assert!(seen.insert(r.id), "duplicate rung id {}", r.id);
        let parts: Vec<&str> = r.id.split('/').collect();
        assert_eq!(parts.len(), 3, "{} must be lane/family/name", r.id);
        assert_eq!(parts[0], r.lane.as_str(), "{} lane prefix mismatch", r.id);
        assert!(!r.motivation.is_empty(), "{} needs a motivation", r.id);
        match &r.cmd {
            Cmd::SelfExe(a) => assert!(!a.is_empty(), "{} needs args", r.id),
            Cmd::External(p, _) => assert!(!p.is_empty(), "{} needs a program", r.id),
        }
    }
}

#[test]
fn every_prerequisite_exists_and_comes_earlier() {
    // A forward reference would silently never block, which is worse than a cycle: the suite
    // would look like it had dependency awareness while having none.
    for (i, r) in LADDER.iter().enumerate() {
        for need in r.requires {
            let at = LADDER.iter().position(|o| o.id == *need);
            let at = at.unwrap_or_else(|| panic!("{} requires unknown rung {need}", r.id));
            assert!(at < i, "{} requires {need}, which comes later in the ladder", r.id);
        }
    }
}

#[test]
fn the_ladder_actually_escalates_within_each_lane() {
    // A rung with prerequisites must come after them (checked elsewhere); here we assert the
    // ladder is not accidentally flat — each lane needs a dependency chain, or "highest rung
    // passed" is meaningless and a failure blocks nothing.
    for lane in [Lane::Offline, Lane::Tour, Lane::Live] {
        let chained = LADDER
            .iter()
            .filter(|r| r.lane == lane && !r.requires.is_empty())
            .count();
        assert!(chained >= 2, "{:?} lane has no escalation chain", lane);
    }
}

#[test]
fn the_crossover_is_covered_and_reaches_the_regime_that_matters() {
    // G1 in the design doc: PERT_FE_THRESHOLD had no test point anywhere in the project. The
    // CORRECTNESS half is now covered by --selftest (oracle entries at 9.3e27×/1.3e28× plus a
    // bracket check on the selector); this rung owns the half that still is not — HOLDING at
    // the crossover under a frame budget, which is the regime every device loss has landed in.
    // If someone deletes it, the most dangerous depth in the app goes unsampled under load.
    let x = LADDER
        .iter()
        .find(|r| r.id == "offline/depth/e28-crossover")
        .expect("the e28 crossover rung must exist — see design G1");
    match &x.cmd {
        Cmd::SelfExe(a) => assert!(
            a.windows(2).any(|w| w[0] == "--zoom" && w[1] == "1e28"),
            "the crossover rung must actually sit at 1e28"
        ),
        _ => panic!("crossover rung must run our own binary"),
    }
}

#[test]
fn an_absent_external_tool_is_a_skip_not_a_pass_and_not_a_failure() {
    // The corpus gate needs Python. If it is missing the run must say so out loud: counting it
    // as a pass would let a machine report a green suite while never checking deep arithmetic.
    let skip = Outcome::SkipUnsupported("python not found".into());
    assert!(!skip.is_pass(), "a skipped gate is not a pass");
    assert!(!skip.is_failure(), "a machine without python is not a product bug");
}

#[test]
fn selectors_match_by_exact_id_and_by_prefix() {
    assert_eq!(select(&[]).len(), LADDER.len(), "no selector means everything");
    let live = select(&["live".to_string()]);
    assert!(!live.is_empty());
    assert!(live.iter().all(|r| r.lane == Lane::Live));
    let one = select(&["live/tour/grand-full".to_string()]);
    assert_eq!(one.len(), 1);
    assert_eq!(one[0].id, "live/tour/grand-full");
    // A prefix must not match a longer sibling name by accident.
    assert!(select(&["live/tou".to_string()]).is_empty());
    assert!(select(&["nope".to_string()]).is_empty());
}

#[test]
fn selection_preserves_ladder_order() {
    let all = select(&[]);
    let ids: Vec<&str> = all.iter().map(|r| r.id).collect();
    let ladder_ids: Vec<&str> = LADDER.iter().map(|r| r.id).collect();
    assert_eq!(ids, ladder_ids);
}

#[test]
fn a_deadline_expiry_outranks_the_exit_code() {
    // A killed child also reports a non-zero/absent code; "timed out" must win, or every hang
    // is filed as a crash and the real signal is lost.
    assert_eq!(classify(true, None, ""), Outcome::FailDeadline);
    assert_eq!(classify(true, Some(0), ""), Outcome::FailDeadline);
}

#[test]
fn device_loss_is_recognised_even_though_it_exits_nonzero() {
    let log = "[fd-wgpu] DEVICE LOST (Unknown): Device is lost";
    assert_eq!(classify(false, Some(2), log), Outcome::FailDeviceLost);
    assert_eq!(classify(false, None, log), Outcome::FailDeviceLost);
}

#[test]
fn a_watchdog_hang_is_a_failure_not_a_pass() {
    // The banked lesson: a soak that greps only for crashes passes a hung app. Exit 0 with a
    // watchdog line in the log is exactly that shape.
    let log = "[fd-watch] possible hang: no activity for 71s";
    assert_eq!(classify(false, Some(0), log), Outcome::FailHang);
}

#[test]
fn clean_and_drifted_runs_are_told_apart() {
    assert_eq!(classify(false, Some(0), "checks 116/116 — OK"), Outcome::Pass);
    assert_eq!(classify(false, Some(2), "algorithmic drift"), Outcome::FailAssert);
    assert_eq!(classify(false, Some(101), "panicked"), Outcome::FailCrash);
}

#[test]
fn only_real_failures_block_dependents() {
    // Blocked must not propagate: if it did, one root cause would report as a chain of
    // apparently distinct failures and the "multiple failures per run" goal would be defeated.
    assert!(Outcome::FailDeviceLost.is_failure());
    assert!(Outcome::FailDeadline.is_failure());
    assert!(!Outcome::Blocked("x".into()).is_failure());
    assert!(!Outcome::Pass.is_failure());
    assert!(!Outcome::Blocked("x".into()).is_pass());
}
