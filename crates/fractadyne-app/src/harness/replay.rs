//! `--replay FILE`: recompute every decision a `FRACTADYNE_RECORD_DECIDE` recording holds and
//! compare it with the recorded one (design/live-render-robustness.md §5.4, W5); also the
//! fixture coverage `cargo test` and `--selftest` hold the recordings to. Moved from
//! `render/decide.rs`, whose decisions it replays; the recorder stays there.

use crate::render::decide::*;

/// What a replay found.
#[derive(Debug, Default)]
pub(crate) struct ReplayReport {
    /// The recording's header line, if it had one.
    pub(crate) header: Option<serde_json::Value>,
    /// Decisions recomputed, and how many of them were of a MOVING frame (the ones with anything to
    /// decide: a settled frame's moving scale is 1 by construction).
    pub(crate) decisions: u64,
    pub(crate) moving: u64,
    /// Decisions whose recomputation differed from the record, and the first few, field by field.
    pub(crate) mismatches: u64,
    pub(crate) first: Vec<String>,
    /// Lines that could not be read back (an unknown stage, a value JSON cannot carry, a torn line).
    pub(crate) unreadable: u64,
    /// The recording is of another schema: nothing was compared.
    pub(crate) wrong_schema: Option<u64>,
    /// Its regime context lines, and the soak predicate's verdict over them (§6.7): whether the run
    /// ENTERED the escaped-reference regime, and the soak's own line saying how.
    pub(crate) regime_lines: u64,
    pub(crate) regime_entered: bool,
    pub(crate) regime_line: Option<String>,
    /// The adapters its header lines name (a file may hold several recordings, a header each).
    pub(crate) adapters: Vec<String>,
}

impl ReplayReport {
    /// Every decision recomputed exactly, and enough of them to mean something.
    pub(crate) fn passed(&self, min_decisions: u64) -> bool {
        self.wrong_schema.is_none()
            && self.mismatches == 0
            && self.unreadable == 0
            && self.decisions >= min_decisions
            && self.moving > 0
    }
}

/// Recompute every recorded decision and compare it with the one recorded, field by field.
pub(crate) fn replay_lines(text: &str) -> ReplayReport {
    let mut rep = ReplayReport::default();
    let mut regime = crate::harness::soak::RegimeAcc::default();
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            rep.unreadable += 1;
            continue;
        };
        match v.get("kind").and_then(|k| k.as_str()) {
            Some("header") => {
                let schema = v.get("schema").and_then(|s| s.as_u64()).unwrap_or(0);
                if schema != DECIDE_SCHEMA as u64 {
                    rep.wrong_schema = Some(schema);
                    return rep;
                }
                if let Some(a) = v.get("adapter").and_then(|a| a.as_str()) {
                    if !rep.adapters.iter().any(|x| x == a) {
                        rep.adapters.push(a.to_string());
                    }
                }
                rep.header = Some(v);
            }
            Some("regime") => match serde_json::from_value::<RegimeFacts>(v.get("facts").cloned().unwrap_or_default()) {
                Ok(f) => {
                    rep.regime_lines += 1;
                    regime.add(&f.record());
                }
                Err(_) => rep.unreadable += 1,
            },
            Some("decision") => {
                let stage = v.get("stage").and_then(|s| s.as_str()).unwrap_or("");
                let recorded = v.get("decision").cloned().unwrap_or(serde_json::Value::Null);
                let recomputed = match stage {
                    "moving" => {
                        let Ok(inputs) = serde_json::from_value::<MovingInputs>(
                            v.get("inputs").cloned().unwrap_or(serde_json::Value::Null),
                        ) else {
                            rep.unreadable += 1;
                            continue;
                        };
                        if inputs.interacting {
                            rep.moving += 1;
                        }
                        serde_json::to_value(decide_moving(&inputs)).unwrap_or(serde_json::Value::Null)
                    }
                    "present" => {
                        let Ok(inputs) = serde_json::from_value::<PresentInputs>(
                            v.get("inputs").cloned().unwrap_or(serde_json::Value::Null),
                        ) else {
                            rep.unreadable += 1;
                            continue;
                        };
                        serde_json::to_value(decide_present(&inputs)).unwrap_or(serde_json::Value::Null)
                    }
                    "iter" => {
                        let Ok(inputs) = serde_json::from_value::<IterInputs>(
                            v.get("inputs").cloned().unwrap_or(serde_json::Value::Null),
                        ) else {
                            rep.unreadable += 1;
                            continue;
                        };
                        serde_json::to_value(decide_iter(&inputs)).unwrap_or(serde_json::Value::Null)
                    }
                    "probe" => {
                        let Ok(inputs) = serde_json::from_value::<ProbeInputs>(
                            v.get("inputs").cloned().unwrap_or(serde_json::Value::Null),
                        ) else {
                            rep.unreadable += 1;
                            continue;
                        };
                        serde_json::to_value(decide_probe(&inputs)).unwrap_or(serde_json::Value::Null)
                    }
                    "walk-dispatch" => {
                        let Ok(inputs) = serde_json::from_value::<WalkDispatchInputs>(
                            v.get("inputs").cloned().unwrap_or(serde_json::Value::Null),
                        ) else {
                            rep.unreadable += 1;
                            continue;
                        };
                        if inputs.interacting {
                            rep.moving += 1;
                        }
                        serde_json::to_value(decide_walk_dispatch(&inputs)).unwrap_or(serde_json::Value::Null)
                    }
                    "walk-id" => {
                        let Ok(inputs) = serde_json::from_value::<WalkIdInputs>(
                            v.get("inputs").cloned().unwrap_or(serde_json::Value::Null),
                        ) else {
                            rep.unreadable += 1;
                            continue;
                        };
                        if inputs.interacting {
                            rep.moving += 1;
                        }
                        serde_json::to_value(decide_walk_id(&inputs)).unwrap_or(serde_json::Value::Null)
                    }
                    "walk-price" => {
                        let Ok(inputs) = serde_json::from_value::<WalkPriceInputs>(
                            v.get("inputs").cloned().unwrap_or(serde_json::Value::Null),
                        ) else {
                            rep.unreadable += 1;
                            continue;
                        };
                        if inputs.interacting {
                            rep.moving += 1;
                        }
                        serde_json::to_value(decide_walk_price(&inputs)).unwrap_or(serde_json::Value::Null)
                    }
                    _ => {
                        rep.unreadable += 1;
                        continue;
                    }
                };
                rep.decisions += 1;
                if recomputed != recorded {
                    rep.mismatches += 1;
                    if rep.first.len() < 10 {
                        let fields: Vec<String> = match (recomputed.as_object(), recorded.as_object()) {
                            (Some(a), Some(b)) => a
                                .iter()
                                .filter(|(k, x)| b.get(*k) != Some(*x))
                                .map(|(k, x)| format!("{k}: recorded {} recomputed {x}", b.get(k).cloned().unwrap_or_default()))
                                .collect(),
                            _ => vec!["(not an object)".to_string()],
                        };
                        rep.first.push(format!(
                            "line {} frame {} view {} {stage}: {}",
                            n + 1,
                            v.get("frame").cloned().unwrap_or_default(),
                            v.get("view").cloned().unwrap_or_default(),
                            fields.join("; ")
                        ));
                    }
                }
            }
            _ => rep.unreadable += 1,
        }
    }
    if rep.regime_lines > 0 {
        let r = regime.report();
        rep.regime_entered = r.entered;
        rep.regime_line = Some(r.line);
    }
    rep
}

/// One coverage term of a fixture set (§7.5): a branch of a lifted decision, and how many of the set's
/// recorded decisions took it.
#[derive(Debug)]
pub(crate) struct Coverage {
    pub(crate) stage: &'static str,
    pub(crate) term: &'static str,
    pub(crate) hits: u64,
}

/// Tally one recorded decision of a stage against that stage's terms (an unreadable one is the
/// replay's to count, not this).
fn tally<I: serde::de::DeserializeOwned, D: serde::de::DeserializeOwned>(
    v: &serde_json::Value,
    terms: &[(&'static str, fn(&I, &D) -> bool)],
    hits: &mut [u64],
) {
    let (Ok(i), Ok(d)) = (
        serde_json::from_value::<I>(v["inputs"].clone()),
        serde_json::from_value::<D>(v["decision"].clone()),
    ) else {
        return;
    };
    for (h, (_, f)) in hits.iter_mut().zip(terms) {
        *h += f(&i, &d) as u64;
    }
}

fn collect(out: &mut Vec<Coverage>, stage: &'static str, terms: impl Iterator<Item = &'static str>, hits: &[u64]) {
    out.extend(terms.zip(hits).map(|(term, &hits)| Coverage { stage, term, hits }));
}

/// A settled view the climb probe would consider: nothing else dispatching, the budget unconverged.
fn probe_idle(i: &ProbeInputs) -> bool {
    !i.interacting
        && !i.offscreen
        && !i.will_reproject
        && !i.tour_playing
        && !i.key_changed
        && !i.tile
        && i.chunk_range.is_none()
        && !i.budget_ok
}

/// §7.5's coverage term over a fixture set: for every branch of every lifted decision, how many of
/// the recordings' decisions took it. Across the set every one must be replayed at least once — a
/// fixture set satisfied by one regime grades the controller only where it is easy. The one place
/// the terms are stated: `cargo test` (`every_recorded_session_replays_to_its_decisions`) and
/// `--selftest replay` both read them here.
pub(crate) fn fixture_coverage<'a>(texts: impl IntoIterator<Item = &'a str>) -> Vec<Coverage> {
    // §7.5's coverage term: across the set, every branch of the decision is replayed at least
    // once — a fixture set satisfied by one regime grades the controller only where it is easy.
    let branches: [(&'static str, fn(&MovingInputs, &MovingDecision) -> bool); 9] = [
        ("settled", |i, _| !i.interacting),
        ("a pin frame", |i, _| i.pin_frame),
        ("prefer detail", |i, _| i.interacting && i.prefer_detail && !i.pin_frame),
        ("the rate cap measured", |_, d| d.rate_cap.is_some_and(|c| c.measured)),
        ("the rate cap's model", |_, d| d.rate_cap.is_some_and(|c| !c.measured)),
        ("the AIMD stepped", |_, d| d.aimd_stepped),
        ("the ladder's rung moved", |i, d| d.visible_res != i.visible_res),
        ("the ladder bound the scale", |_, d| d.res_scale < d.motion_scale),
        ("direct-mode motion", |i, _| i.interacting && !i.is_pert),
    ];
    let present: [(&'static str, fn(&PresentInputs, &PresentDecision) -> bool); 6] = [
        ("a settled compose held the display", |_, d| d.display_hold && !d.pin_gate),
        (
            "a settled compose refused an unverified snapshot",
            |i, d| d.display_hold && !d.pin_gate && !i.hold_active && !i.live_verified,
        ),
        ("a pin held the display", |_, d| d.pin_gate),
        ("a snapshot was taken", |_, d| d.hold_copy),
        ("the settled-cost probe armed", |_, d| d.aa_probe.is_some()),
        ("the colour-pass filter widened", |_, d| d.aa_filter > 1),
    ];
    let climb_terms: [(&'static str, fn(&IterInputs, &IterDecision) -> bool); 11] = [
        ("a settled reading was judged", |i, d| !i.interacting && d.reached_ask && !d.stale),
        ("motion reset the climb", |i, _| i.interacting),
        ("a stale reading was discarded", |_, d| d.stale),
        ("the boost was raised", |_, d| d.events.iter().any(|e| matches!(e, IterEvent::Raised { .. }))),
        ("the boost was seeded", |_, d| d.events.iter().any(|e| matches!(e, IterEvent::Seeded { .. }))),
        ("the verdict was reached", |i, d| d.state.verdict.is_some() && i.state.verdict.is_none()),
        ("an explicit count", |i, _| !i.auto_iter),
        ("the climb plateaued", |_, d| d.events.iter().any(|e| matches!(e, IterEvent::Plateau { .. }))),
        ("the appetite re-pick", |_, d| d.events.iter().any(|e| matches!(e, IterEvent::Repick { .. }))),
        ("the climb was exhausted", |_, d| d.events.iter().any(|e| matches!(e, IterEvent::Exhausted { .. }))),
        ("motion ended a flat run", |_, d| d.events.iter().any(|e| matches!(e, IterEvent::FlatRunEnded { .. }))),
    ];
    let probe_terms: [(&'static str, fn(&ProbeInputs, &ProbeDecision) -> bool); 3] = [
        ("the probe fired", |_, d| d.fired),
        ("an idle unconverged view waited for the pacing", |i, _| {
            probe_idle(i) && i.frame.saturating_sub(i.dispatch_frame) < 3
        }),
        ("an idle unconverged view the pricing rule or the ceiling held", |i, d| {
            probe_idle(i) && i.frame.saturating_sub(i.dispatch_frame) >= 3 && !d.fired
        }),
    ];
    let price_terms: [(&'static str, fn(&WalkPriceInputs, &WalkPriceDecision) -> bool); 10] = [
        ("a settled pass was priced", |i, d| {
            !i.interacting && !i.pin_frame && i.ledger.inflight.is_some() && d.ledger.inflight.is_none()
        }),
        ("a pass was held unpriced", |i, d| i.ledger.inflight.is_some() && d.ledger.inflight.is_some()),
        ("a pin pass was priced", |_, d| {
            d.events.iter().any(|e| matches!(e, WalkPriceEvent::PinPriced { .. }))
        }),
        ("the lethal band shed the licences", |_, d| {
            d.events.iter().any(|e| matches!(e, WalkPriceEvent::LethalShed { .. }))
        }),
        ("a pin pass was released before its reading, its price deferred", |_, d| {
            d.events.iter().any(|e| matches!(e, WalkPriceEvent::PinDeferred { .. }))
        }),
        ("a parked pin pass was priced by its own reading", |_, d| {
            d.events.iter().any(|e| matches!(e, WalkPriceEvent::PinPriced { acc, .. } if *acc == -1.0))
        }),
        ("a pin pass still in flight was parked as the next went out", |_, d| {
            d.events.iter().any(|e| matches!(e, WalkPriceEvent::PinDeferred { stand_in, .. } if *stand_in < 0.0))
        }),
        ("motion abandoned a pass", |i, _| i.interacting && !i.pin_frame && i.ledger.inflight.is_some()),
        ("the growth limiter bound the step", |i, d| {
            (i.interacting || i.pin_frame)
                && i.ledger.motion_step_last > 0
                && d.budget_step == (2 * i.ledger.motion_step_last).max(d.floor)
        }),
        ("an unmeasured walk", |i, _| !i.budget_measured),
    ];
    // (A partial reference capping a settled walk is unit-tested only: no recorded source settles on
    // one — `a_settled_walk_covers_only_its_live_range`.)
    let id_terms: [(&'static str, fn(&WalkIdInputs, &WalkIdDecision) -> bool); 6] = [
        ("a settled walk resumed", |i, d| !d.restarted && !i.pin_frame && !i.interacting),
        ("a restart threw a part-way walk away", |_, d| d.restart_lost.is_some()),
        ("a supersampling sample kept the band ledger", |i, d| {
            d.restarted && !d.clear_ledger && !i.interacting && !i.pin_frame
        }),
        ("another view cleared the band ledger", |_, d| d.clear_ledger),
        ("a pin frame kept its walk", |i, _| i.pin_frame),
        ("the SA-skip floor", |_, d| d.walk_floor > 0),
    ];
    // (The pin gate's wall-clock escape is unit-tested only: W3 measured none on either card —
    // `a_pin_pass_escapes_a_completion_that_never_comes`.)
    let dispatch_terms: [(&'static str, fn(&WalkDispatchInputs, &WalkDispatchDecision) -> bool); 8] = [
        ("a settled pass went out", |i, d| !i.interacting && !i.pin_frame && d.pass),
        ("one unpriced pass held the walk", |i, d| !i.pin_frame && !d.pass && i.cur < i.walk_end),
        ("a pin pass went out", |_, d| d.pin_dispatched),
        ("the pin gate held", |i, d| i.pin_frame && !d.pass && i.cur < i.walk_end),
        ("a moving frame previewed", |i, d| i.interacting && !i.pin_frame && d.pass),
        ("a pass was charged for its running pixels", |_, d| {
            d.events.iter().any(|e| matches!(e, DispatchEvent::Charge { .. }))
        }),
        ("a finished walk emitted its tail", |i, _| i.cur >= i.walk_end),
        ("the tail re-sent for its reading", |i, d| {
            i.cur >= i.walk_end && d.state.tail_retries > i.state.tail_retries
        }),
    ];
    let mut h_branches = [0u64; 9];
    let mut h_present = [0u64; 5];
    let mut h_climb_terms = [0u64; 11];
    let mut h_probe_terms = [0u64; 3];
    let mut h_price_terms = [0u64; 7];
    let mut h_id_terms = [0u64; 6];
    let mut h_dispatch_terms = [0u64; 8];
    for text in texts {
        for line in text.lines() {
            let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else { continue };
            if v["kind"] != "decision" {
                continue;
            }
            match v["stage"].as_str() {
                Some("moving") => tally(&v, &branches, &mut h_branches),
                Some("present") => tally(&v, &present, &mut h_present),
                Some("iter") => tally(&v, &climb_terms, &mut h_climb_terms),
                Some("probe") => tally(&v, &probe_terms, &mut h_probe_terms),
                Some("walk-price") => tally(&v, &price_terms, &mut h_price_terms),
                Some("walk-id") => tally(&v, &id_terms, &mut h_id_terms),
                Some("walk-dispatch") => tally(&v, &dispatch_terms, &mut h_dispatch_terms),
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    collect(&mut out, "moving", branches.iter().map(|(t, _)| *t), &h_branches);
    collect(&mut out, "present", present.iter().map(|(t, _)| *t), &h_present);
    collect(&mut out, "iter", climb_terms.iter().map(|(t, _)| *t), &h_climb_terms);
    collect(&mut out, "probe", probe_terms.iter().map(|(t, _)| *t), &h_probe_terms);
    collect(&mut out, "walk-price", price_terms.iter().map(|(t, _)| *t), &h_price_terms);
    collect(&mut out, "walk-id", id_terms.iter().map(|(t, _)| *t), &h_id_terms);
    collect(&mut out, "walk-dispatch", dispatch_terms.iter().map(|(t, _)| *t), &h_dispatch_terms);
    out
}

/// A replay fixture must hold at least this many decisions to mean anything (§7.5's anti-vacuity).
pub(crate) const REPLAY_MIN_DECISIONS: u64 = 200;

/// `--replay <file>`: recompute every decision in a recording and diff it against the recorded one.
/// Exit 0 = every decision recomputed exactly; 2 = a mismatch, an unreadable line, another schema,
/// or too little to judge (never a pass).
pub(crate) fn run_replay(path: &str) -> ! {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("--replay: cannot read {path}: {e}");
            crate::exit(2);
        }
    };
    let rep = replay_lines(&text);
    if let Some(h) = &rep.header {
        println!(
            "--replay: {path} - recorded by {} (tunables: {})",
            h.get("version").and_then(|x| x.as_str()).unwrap_or("?"),
            h.get("tunables").and_then(|x| x.as_str()).unwrap_or("?")
        );
    }
    if let Some(s) = rep.wrong_schema {
        println!("--replay: VACUOUS - the recording is schema {s}, this build reads {DECIDE_SCHEMA}");
        crate::exit(2);
    }
    println!(
        "--replay: {} decisions recomputed ({} of a moving frame), {} differ, {} unreadable",
        rep.decisions, rep.moving, rep.mismatches, rep.unreadable
    );
    for m in &rep.first {
        println!("  {m}");
    }
    if let Some(r) = &rep.regime_line {
        println!("--replay: regime ({} frame lines): {r}", rep.regime_lines);
    }
    let pass = rep.passed(REPLAY_MIN_DECISIONS);
    if !pass && rep.mismatches == 0 && rep.unreadable == 0 {
        println!(
            "--replay: VACUOUS - fewer than {REPLAY_MIN_DECISIONS} decisions, or none of a moving frame"
        );
    }
    println!("--replay: {}", if pass { "PASS - every decision recomputed exactly" } else { "FAIL" });
    crate::exit(if pass { 0 } else { 2 });
}
