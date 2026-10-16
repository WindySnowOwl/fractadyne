use super::*;

fn moving() -> MovingInputs {
    MovingInputs {
        px: 1442 * 1102,
        gpu_iter: 10_000,
        budget: 4_000_000_000,
        interacting: true,
        pin_frame: false,
        view: 0,
        is_pert: true,
        panel_px: 1442 * 1102,
        prefer_detail: false,
        tour_playing: false,
        can_chunk: true,
        zoom_oct_s: 2.5,
        frame_ms: 16.7,
        last_dt_ms: 16.7,
        prev_real: true,
        motion_rate: 2.0e7,
        min_motion_res: 0.3,
        pin_frames_last: 0,
        pin_res_last: 0.0,
        motion_res: 0.6,
        motion_res_measured: false,
        visible_res: 1.0,
        motion_pass_steps_last: 0,
        norm_hi: 0.0,
        esc_top: 0.0,
        esc_hist: None,
        tdr_bootstrap_steps: crate::tunables::cost().tdr_bootstrap_steps,
        motion_need_source: 0,
        motion_need_quantile: 1.0,
        pass_fixed_ms: crate::tunables::PASS_FIXED_MS_DEFAULT,
    }
}

/// §7.5's gate, on the CPU alone (so CI runs it): every recording under `validation/replays/`
/// replays to the decisions it recorded, field by field. Anti-vacuity: the directory must hold a
/// recording, and each must carry at least `REPLAY_MIN_DECISIONS` decisions, some of a moving
/// frame. A deliberate change to a lifted decision turns this red, and the fixtures are then
/// re-recorded (`FRACTADYNE_RECORD_DECIDE`, see `validation/replays/README.md`).
#[test]
fn every_recorded_session_replays_to_its_decisions() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../validation/replays");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .expect("validation/replays exists")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "jsonl"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "VACUOUS: no recording in {}", dir.display());
    let texts: Vec<String> = files.iter().map(|f| std::fs::read_to_string(f).expect("readable fixture")).collect();
    // §7.5's coverage term (`fixture_coverage`): every branch of every lifted decision replayed.
    for c in fixture_coverage(texts.iter().map(String::as_str)) {
        assert!(c.hits > 0, "VACUOUS: no replayed {} decision where {}", c.stage, c.term);
    }
    // §7.5's regime term: a fixture ENTERED the §6.7 escaped-reference regime — the soak's own
    // predicate over the recordings' regime lines — so the set does not grade the controller only
    // where it is easy (`escaped-ref.jsonl`, the 2026-09-21 crash view under REF_ESCAPE_AT).
    assert!(
        texts.iter().any(|t| replay_lines(t).regime_entered),
        "VACUOUS: no fixture entered the §6.7 escaped-reference regime"
    );
    // …and the field term: the set's recordings come from more than one card (a field capture),
    // so a hidden input that differs between cards turns a replay red here, on the CPU.
    let mut adapters: Vec<String> = texts.iter().flat_map(|t| replay_lines(t).adapters).collect();
    adapters.sort();
    adapters.dedup();
    assert!(adapters.len() >= 2, "VACUOUS: the fixtures name fewer than two adapters: {adapters:?}");
    for (f, text) in files.iter().zip(&texts) {
        let rep = replay_lines(text);
        assert!(
            rep.passed(REPLAY_MIN_DECISIONS),
            "{}: {} decisions ({} moving), {} differ, {} unreadable, schema {:?}\n{}",
            f.display(),
            rep.decisions,
            rep.moving,
            rep.mismatches,
            rep.unreadable,
            rep.wrong_schema,
            rep.first.join("\n")
        );
    }
}

/// The replay's premise, checked where it can break silently: a lifted decision reads ONLY its
/// inputs. A global it reaches — a tunable read in place, the adapter's calibration, a clock —
/// is an input the recording does not carry, so the replay recomputes it in another process
/// (another card, CI with none) and differs, or worse, agrees only on the machine that recorded.
/// The first `--uitest` recording found one: `decide_moving` reached
/// `calibration::pass_fixed_ms()` through `motion_pass_steps`. Every `decide_*` body here and
/// every free function it reaches (transitively, by name, across the render sources and
/// `main.rs`) is scanned for the ways a global is read.
#[test]
fn a_lifted_decision_reads_no_global() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = vec![root.join("render.rs"), root.join("main.rs")];
    for e in std::fs::read_dir(root.join("render")).expect("render/").flatten() {
        if e.path().extension().is_some_and(|x| x == "rs") {
            sources.push(e.path());
        }
    }
    // Free functions (column 0) by name → body, comments stripped.
    let strip = |l: &str| l.split("//").next().unwrap_or("").to_string();
    let mut fns: std::collections::HashMap<String, String> = Default::default();
    for p in &sources {
        let text = std::fs::read_to_string(p).expect("source").replace("\r\n", "\n");
        let lines: Vec<&str> = text.lines().collect();
        let mut k = 0;
        while k < lines.len() {
            let l = lines[k];
            let head = ["pub(crate) fn ", "pub(super) fn ", "pub fn ", "fn "]
                .iter()
                .find_map(|p| l.strip_prefix(p));
            if let Some(name) = head.and_then(|h| h.split(['(', '<']).next()) {
                let start = k;
                while k < lines.len() && lines[k] != "}" {
                    k += 1;
                }
                let body: Vec<String> = lines[start..k.min(lines.len())].iter().map(|l| strip(l)).collect();
                fns.entry(name.to_string()).or_insert_with(|| body.join("\n"));
            }
            k += 1;
        }
    }
    let decide: Vec<String> = fns.keys().filter(|n| n.starts_with("decide_")).cloned().collect();
    assert!(decide.len() >= 3, "VACUOUS: found the lifted decisions {decide:?}");
    let forbidden = [
        "cost()",
        "calibration::",
        "overridden(",
        "app_micros(",
        "Instant::now",
        "SystemTime",
        ".load(",
        "env::var",
        "trace_on(",
    ];
    for d in &decide {
        let mut seen = std::collections::BTreeSet::new();
        let mut todo = vec![d.clone()];
        while let Some(name) = todo.pop() {
            if !seen.insert(name.clone()) {
                continue;
            }
            let body = &fns[&name];
            for bad in forbidden {
                assert!(
                    !body.contains(bad),
                    "{d} reaches `{name}`, which reads a global (`{bad}`): make it an input"
                );
            }
            // Every identifier called here that is a free function of the scanned sources.
            let bytes = body.as_bytes();
            let mut s = 0;
            for (j, &b) in bytes.iter().enumerate() {
                if !(b.is_ascii_alphanumeric() || b == b'_') {
                    if b == b'(' && j > s {
                        let id = &body[s..j];
                        if fns.contains_key(id) && !seen.contains(id) {
                            todo.push(id.to_string());
                        }
                    }
                    s = j + 1;
                }
            }
        }
        assert!(seen.len() > 1 || d == "decide_present", "VACUOUS: {d} reached no helper");
    }
}

#[test]
fn a_recorded_regime_is_judged_by_the_soaks_predicate() {
    // A storm frame: a short COMPLETE reference (655 against a 4,627 ask), BLA off, and a counter
    // reading with rebases and no BLA skips. Recorded as a regime line, it enters the regime.
    let storm = RegimeFacts {
        frame: 358,
        t_ms: 22_600,
        view: 0,
        has_ref: true,
        ref_partial: false,
        ref_len: 655,
        gpu_iter: 4_627,
        bla_on: false,
        ctr_new: true,
        ctr_rebase: 11_492_892,
        ctr_bla_skip: 0,
        dispatched: true,
        chunked: true,
        fe_budget: 1_779_000_000,
    };
    let line = |f: RegimeFacts| serde_json::json!({"kind": "regime", "facts": f}).to_string();
    let rep = replay_lines(&line(storm));
    assert_eq!((rep.regime_lines, rep.regime_entered, rep.unreadable), (1, true, 0));
    assert!(rep.regime_line.as_deref().is_some_and(|l| l.starts_with("ENTERED")), "{:?}", rep.regime_line);
    // The controls: the shape without a storm reading, and a reference long enough for its ask.
    assert!(!replay_lines(&line(RegimeFacts { ctr_new: false, ..storm })).regime_entered);
    assert!(!replay_lines(&line(RegimeFacts { ref_len: 2_000, ..storm })).regime_entered);
    assert!(!replay_lines(&line(RegimeFacts { bla_on: true, ..storm })).regime_entered);
    // And it is judged by the soak's predicate itself, on the record it came from.
    let mut acc = crate::harness::soak::RegimeAcc::default();
    acc.add(&storm.record());
    assert!(acc.report().entered);
    assert_eq!(RegimeFacts::of(&storm.record()), storm, "the facts round-trip through the record");
}

#[test]
fn a_replay_catches_a_changed_decision() {
    // The control: the gate above can go red. A record whose decision is not what the inputs
    // decide is a mismatch, named by field.
    let i = MovingInputs { norm_hi: 5_000.0, motion_pass_steps_last: 200_000_000, ..moving() };
    let mut d = decide_moving(&i);
    let good = serde_json::json!({"kind": "decision", "frame": 7, "view": 0, "stage": "moving",
        "inputs": i, "decision": d}).to_string();
    let ok = replay_lines(&good);
    assert_eq!((ok.decisions, ok.mismatches, ok.moving), (1, 0, 1));
    d.visible_res *= 0.5;
    let bad = serde_json::json!({"kind": "decision", "frame": 7, "view": 0, "stage": "moving",
        "inputs": i, "decision": d}).to_string();
    let rep = replay_lines(&bad);
    assert_eq!(rep.mismatches, 1);
    assert!(rep.first[0].contains("visible_res"), "{}", rep.first[0]);
    // Another schema is refused, not compared.
    let other = format!("{{\"kind\":\"header\",\"schema\":{}}}\n{good}", DECIDE_SCHEMA + 1);
    assert!(replay_lines(&other).wrong_schema.is_some());
    // Too little to judge is never a pass.
    assert!(!ok.passed(REPLAY_MIN_DECISIONS));
}

#[test]
fn the_same_inputs_give_the_same_decision() {
    let i = moving();
    assert_eq!(decide_moving(&i), decide_moving(&i));
}

#[test]
fn a_settled_frame_is_native_and_touches_no_state() {
    let i = MovingInputs { interacting: false, ..moving() };
    let d = decide_moving(&i);
    assert_eq!((d.motion_scale, d.res_scale), (1.0, 1.0));
    assert_eq!((d.motion_res, d.motion_res_measured, d.visible_res), (i.motion_res, false, i.visible_res));
    assert_eq!(d.vis_sizing, [0.0; 3]);
    assert!(d.rate_cap.is_none() && !d.aimd_stepped);
}

#[test]
fn a_pin_frame_keeps_its_captured_geometry() {
    let d = decide_moving(&MovingInputs { pin_frame: true, ..moving() });
    assert_eq!((d.motion_scale, d.res_scale), (1.0, 1.0));
    assert!(!d.aimd_stepped, "a pin's passes do not step the AIMD");
}

#[test]
fn the_aimd_steps_only_after_a_real_frame() {
    let stepped = decide_moving(&moving());
    assert!(stepped.aimd_stepped && stepped.motion_res_measured);
    let held = decide_moving(&MovingInputs { prev_real: false, ..moving() });
    assert!(!held.aimd_stepped && !held.motion_res_measured);
    assert_eq!(held.motion_res, 0.6);
}

#[test]
fn the_ladder_sizes_a_moving_frame_to_reach_its_escapes() {
    // A pass budget that cannot reach a 5,000-iteration top over the whole panel drops the rung.
    let i = MovingInputs { norm_hi: 5_000.0, motion_pass_steps_last: 200_000_000, ..moving() };
    let d = decide_moving(&i);
    assert!(d.visible_res < 1.0, "rung {}", d.visible_res);
    assert!(d.res_scale <= d.visible_res);
    assert_eq!(d.vis_sizing[0], 5_000.0);
    // The complete walk's top is read only under MOTION_NEED_SOURCE=1.
    let other = decide_moving(&MovingInputs { esc_top: 50.0, ..i.clone() });
    assert_eq!(other, d, "source 0 ignores esc_top");
    let walk = decide_moving(&MovingInputs { esc_top: 50.0, motion_need_source: 1, ..i });
    assert_eq!(walk.vis_sizing[0], 50.0);
}

/// A settled view at 2^200 (zoom cap 2,000 + 200 × 256 = 53,200) under auto-iterations, with
/// the full 10M appetite, nothing climbed yet.
fn climb() -> IterInputs {
    IterInputs {
        interacting: false,
        auto_iter: true,
        eff_iter: crate::MAX_ITER_LIMIT,
        log2mag: 200.0,
        iter_cap: crate::MAX_ITER_LIMIT,
        reading: None,
        ref_len: 0,
        ref_escaped: false,
        recompute_in_flight: false,
        state: IterState {
            boost: 1.0,
            probe: None,
            plateau: false,
            stall: 0,
            stall_base: 1.0,
            exhausted: false,
            repicked: false,
            verdict: None,
            capped_frac: None,
            budget_measured: 0,
            budget_maxed: false,
        },
    }
}

/// A MAXITER reading: this capped fraction, measured by a pass armed at `armed` iterations.
fn capped(frac: f32, armed: u32) -> Option<u64> {
    Some(((frac.to_bits() as u64) << 32) | armed as u64)
}

/// The next frame of the same settled view, its reading measured at the budget the last one planned.
fn next(i: &IterInputs, d: &IterDecision, frac: f32) -> IterInputs {
    IterInputs { reading: capped(frac, d.planned), state: d.state, ..i.clone() }
}

#[test]
fn a_raise_that_helps_climbs_and_a_resolved_reading_is_the_verdict() {
    let i = IterInputs { reading: capped(0.5, 53_200), ..climb() };
    let d = decide_iter(&i);
    assert_eq!(d.budget_now, 53_200);
    assert_eq!(d.events, vec![IterEvent::Raised { frac: 0.5, from: 1.0, to: 1.6, flat: None }]);
    assert_eq!((d.state.probe, d.state.capped_frac, d.state.budget_measured), (Some((1.0, 0.5)), Some(0.5), 53_200));
    assert_eq!(d.state.verdict, None, "a raise is no verdict");
    assert_eq!((d.planned, d.gpu_iter), (85_120, 85_120));
    // Resolved at the raised budget: the boost stays, and that budget is the verdict.
    let d2 = decide_iter(&next(&i, &d, 0.0001));
    assert!(d2.events.is_empty());
    assert_eq!(d2.state.boost, 1.6);
    assert_eq!(d2.state.verdict, Some(85_120));
}

#[test]
fn three_fruitless_raises_on_a_partly_capped_frame_are_interior() {
    let mut i = IterInputs { reading: capped(0.5, 53_200), ..climb() };
    let mut d = decide_iter(&i);
    let mut steps = vec![];
    for _ in 0..3 {
        i = next(&i, &d, 0.5);
        d = decide_iter(&i);
        steps.extend(d.events.iter().copied());
    }
    let flats: Vec<_> = steps
        .iter()
        .map(|e| match e {
            IterEvent::Raised { flat, .. } => *flat,
            _ => None,
        })
        .collect();
    assert_eq!(flats, vec![Some(1), Some(2), None]);
    assert_eq!(steps[2], IterEvent::Plateau { to: 1.0 });
    assert!(d.state.plateau);
    assert_eq!(d.state.boost, 1.0, "back to where the fruitless run began");
    // A latched plateau stops the climb: the same reading again changes nothing.
    let after = decide_iter(&next(&i, &d, 0.5));
    assert!(after.events.is_empty());
    assert_eq!(after.state.boost, 1.0);
}

#[test]
fn a_fully_capped_frame_keeps_climbing_through_flat_steps() {
    // 100% capped looks the same at every budget short of the need, so flat steps there prove
    // nothing (the 6.5e94× view needed ~121×): no plateau, the climb goes on in 2.5× steps.
    let mut i = IterInputs { reading: capped(1.0, 53_200), ..climb() };
    let mut d = decide_iter(&i);
    for _ in 0..4 {
        i = next(&i, &d, 1.0);
        d = decide_iter(&i);
    }
    assert!(!d.state.plateau);
    assert_eq!(d.state.boost, 2.5f64.powi(5));
}

#[test]
fn motion_ends_a_fruitless_run_where_it_began() {
    let i = IterInputs {
        interacting: true,
        reading: capped(0.5, 218_009),
        state: IterState {
            boost: 4.096,
            probe: Some((2.56, 0.5)),
            stall: 2,
            stall_base: 1.0,
            verdict: Some(53_200),
            capped_frac: Some(0.5),
            ..climb().state
        },
        ..climb()
    };
    let d = decide_iter(&i);
    assert_eq!(d.events, vec![IterEvent::FlatRunEnded { stall: 2, from: 4.096, to: 1.0 }]);
    assert_eq!((d.state.boost, d.state.probe, d.state.stall), (1.0, None, 0));
    assert_eq!((d.state.verdict, d.state.capped_frac), (None, None), "a moving frame judges nothing");
    // A run that had not stalled keeps its boost through motion.
    let kept = decide_iter(&IterInputs { state: IterState { stall: 0, ..i.state }, ..i.clone() });
    assert_eq!(kept.state.boost, 4.096);
}

#[test]
fn the_seed_needs_an_orbit_that_escaped() {
    let i = IterInputs { ref_len: 20 * 53_200, ref_escaped: true, ..climb() };
    let d = decide_iter(&i);
    assert_eq!(d.events, vec![IterEvent::Seeded { from: 1.0, to: 20.0, ref_len: 20 * 53_200 }]);
    assert_eq!((d.budget_now, d.planned), (20 * 53_200, 20 * 53_200));
    // An orbit that stopped at the length we asked for measures the ask, not the location.
    assert!(decide_iter(&IterInputs { ref_escaped: false, ..i.clone() }).events.is_empty());
    // Nor past a latched plateau, nor while moving, nor under an explicit count.
    let plateau = IterState { plateau: true, ..i.state };
    assert!(decide_iter(&IterInputs { state: plateau, ..i.clone() }).events.is_empty());
    assert!(decide_iter(&IterInputs { interacting: true, ..i.clone() }).events.is_empty());
    let explicit = decide_iter(&IterInputs { auto_iter: false, eff_iter: 70_000, ..i });
    assert!(explicit.events.is_empty());
    assert_eq!(explicit.planned, 70_000, "an explicit count is honoured verbatim");
}

#[test]
fn a_reading_measured_at_another_budget_is_discarded() {
    // The readback lands frames late: a pre-raise "still capped" must not read as "the raise
    // did not help".
    let i = IterInputs { reading: capped(1.0, 33_250), ..climb() };
    let d = decide_iter(&i);
    assert!(d.reached_ask && d.stale);
    assert!(d.events.is_empty());
    assert_eq!((d.state.capped_frac, d.state.verdict), (None, None));
}

#[test]
fn all_capped_at_the_full_appetite_repicks_once_then_reverts() {
    // The appetite (53,200) binds at boost 2, and every pixel is still capped there.
    let i = IterInputs {
        eff_iter: 53_200,
        reading: capped(1.0, 53_200),
        state: IterState { boost: 2.0, ..climb().state },
        ..climb()
    };
    let d = decide_iter(&i);
    assert!(d.repick && d.state.repicked);
    assert_eq!(d.events, vec![IterEvent::Repick { frac: 1.0, budget: 53_200 }]);
    assert_eq!(d.state.boost, 2.0, "the boost stays at the appetite while the re-pick builds");
    // Still in flight: wait, decide nothing.
    let wait = decide_iter(&IterInputs { recompute_in_flight: true, ..next(&i, &d, 1.0) });
    assert!(!wait.repick && wait.events.is_empty());
    assert_eq!(wait.state.boost, 2.0);
    // Landed and still all-capped: interior; the cheap budget paints the same flat frame.
    let done = decide_iter(&next(&i, &wait, 1.0));
    assert_eq!(done.events, vec![IterEvent::Exhausted { frac: 1.0, budget: 53_200, from: 2.0 }]);
    assert!(done.state.plateau && done.state.exhausted && !done.repick);
    assert_eq!(done.state.boost, 1.0);
}

#[test]
fn a_recorded_climb_round_trips() {
    let i = IterInputs { reading: capped(0.37, 53_200), ref_len: 9_999, ..climb() };
    let d = decide_iter(&i);
    let line = serde_json::json!({"kind": "decision", "frame": 1, "view": 0, "stage": "iter",
        "inputs": i, "decision": d}).to_string();
    let rep = replay_lines(&line);
    assert_eq!((rep.decisions, rep.mismatches, rep.unreadable), (1, 0, 0));
}

/// A settled view whose budget has not converged, nothing else dispatching, the last real
/// dispatch 10 frames ago, the bootstrap budget (4e8) in force, and a frame of 8e8 nominal steps.
/// A settled prefer-detail view whose chunked walk composes this frame, nothing held yet.
fn composing() -> PresentInputs {
    PresentInputs {
        direct: false,
        chunk_over: true,
        chunk_range: Some([0, 4_096]),
        probe_fired: false,
        tile_work: false,
        grid_composing: false,
        res_scale: 1.0,
        interacting: false,
        offscreen: false,
        will_reproject: false,
        key_changed: false,
        gpu_iter: 50_000,
        ss: 1,
        spx: 1_000_000,
        prefer_detail: true,
        tour_playing: false,
        aa: 1,
        hold_active: false,
        pin_active: false,
        chunk_dirty: false,
        live_verified: true,
        aa_unmeasured: false,
        frame: 100,
    }
}

#[test]
fn a_settled_compose_snapshots_a_verified_texture() {
    let d = decide_present(&composing());
    assert!(d.display_hold && !d.pin_gate);
    assert!(d.hold_copy);
    assert_eq!(d.hold_verified, Some(true));
}

#[test]
fn a_settled_compose_never_snapshots_an_unverified_texture() {
    // U42: it took the snapshot of whatever the texture held and served it for the whole
    // compose. Now it takes none — the display shows the compose live — and says nothing new
    // about the hold (the frame that dropped the last one already called it unverified).
    let d = decide_present(&PresentInputs { live_verified: false, ..composing() });
    assert!(d.display_hold && !d.pin_gate, "still gated: the GPU shows live for want of a snapshot");
    assert!(!d.hold_copy);
    assert_eq!(d.hold_verified, None);
    assert!(d.hold_active, "and not retaken mid-compose, over the composing texture");
    // …and a compose already under way keeps the snapshot it took, verified or not.
    let d = decide_present(&PresentInputs { live_verified: false, hold_active: true, ..composing() });
    assert!(!d.hold_copy && d.display_hold);
}

fn probe() -> ProbeInputs {
    ProbeInputs {
        interacting: false,
        offscreen: false,
        will_reproject: false,
        tour_playing: false,
        key_changed: false,
        tile: false,
        chunk_range: None,
        budget_ok: false,
        frame: 100,
        dispatch_frame: 90,
        spx: 1000 * 800,
        ss: 1,
        gpu_iter: 1000,
        fe_budget: 0,
        bootstrap_steps: 400_000_000,
        explicit_dispatch_cap: crate::tunables::EXPLICIT_DISPATCH_CAP_DEFAULT,
        tdr_min_steps: crate::tunables::TDR_MIN_STEPS_DEFAULT,
        tdr_steps_ceil: crate::tunables::TDR_STEPS_CEIL_DEFAULT,
    }
}

#[test]
fn the_probe_re_measures_an_unconverged_idle_view() {
    let d = decide_probe(&probe());
    assert!(d.fired);
    assert_eq!(d.steps, 800_000_000, "the forced pass's nominal steps, for its measurement");
    // Anything else dispatching this frame, any motion, or a converged budget: no probe.
    for (what, i) in [
        ("moving", ProbeInputs { interacting: true, ..probe() }),
        ("a changed key", ProbeInputs { key_changed: true, ..probe() }),
        ("a settle tile", ProbeInputs { tile: true, ..probe() }),
        ("a chunk range", ProbeInputs { chunk_range: Some([0, 256]), ..probe() }),
        ("a converged budget", ProbeInputs { budget_ok: true, ..probe() }),
        ("a reprojection", ProbeInputs { will_reproject: true, ..probe() }),
        ("offscreen", ProbeInputs { offscreen: true, ..probe() }),
        ("a playing tour", ProbeInputs { tour_playing: true, ..probe() }),
    ] {
        assert_eq!(decide_probe(&i), ProbeDecision { fired: false, steps: 0 }, "{what}");
    }
}

#[test]
fn the_probe_is_paced_three_frames_apart() {
    assert!(!decide_probe(&ProbeInputs { dispatch_frame: 98, ..probe() }).fired);
    assert!(decide_probe(&ProbeInputs { dispatch_frame: 97, ..probe() }).fired);
}

#[test]
fn the_probe_fires_only_for_a_reading_the_pricing_rule_keeps() {
    // The boundary is `PRICE_REPRESENTATIVE_FRAC` of the budget, read from the constant (the
    // rule that discards a reading and the rule that spends a dispatch on one cannot split).
    let at = (400_000_000f64 * super::super::PRICE_REPRESENTATIVE_FRAC).ceil() as u64;
    let spx = at / 1000;
    assert_eq!(spx * 1000, at, "the boundary lands on a whole pixel count");
    assert!(decide_probe(&ProbeInputs { spx, ..probe() }).fired);
    assert!(!decide_probe(&ProbeInputs { spx: spx - 1, ..probe() }).fired);
}

#[test]
fn the_probe_stops_at_the_explicit_dispatch_ceiling() {
    // A measured budget at the ceiling: the next ×1.5 step was the lethal one (beta.48).
    let cap = crate::tunables::EXPLICIT_DISPATCH_CAP_DEFAULT;
    let big = ProbeInputs { fe_budget: cap, spx: cap / 1000, ..probe() };
    assert!(!decide_probe(&big).fired);
    assert!(decide_probe(&ProbeInputs { fe_budget: cap - 1, ..big.clone() }).fired);
    // The budget is clamped to the tunable bounds first, and those are inputs.
    assert!(!decide_probe(&ProbeInputs { fe_budget: cap - 1, tdr_min_steps: cap, ..big }).fired);
}

/// A settled chunk walk on a 1000×800 panel at ss 1, a measured budget, stock tunables, and one
/// 1,024-iteration pass in band 3 in flight for 50 ms.
fn walk() -> WalkPriceInputs {
    let c = crate::tunables::cost();
    WalkPriceInputs {
        interacting: false,
        pin_frame: false,
        auto_iter: true,
        spx: 1000 * 800,
        ss: 1,
        tdr_steps: 4_000_000_000,
        gpu_iter: 100_000,
        budget_measured: true,
        worst_rate: Some(2.0e7),
        motion_rate: 2.0e7,
        frame_rate: 0.0,
        last_dt_ms: 16.7,
        frame_period_ms: 16.7,
        present_throttle: 0,
        frame: 500,
        pin_pass_gpu_ms: -1.0,
        pin_pass_frame: 0,
        pin_inflight: 0,
        pin_done_at: 0,
        pin_dispatch_at: 0,
        late_readings: Vec::new(),
        pin_inflight_max: 1,
        no_timestamps: false,
        ledger: WalkLedger {
            inflight: Some((1024, 3, 50.0, false)),
            bands: [0; crate::tunables::CHUNK_BANDS],
            pass_dt: 0.0,
            motion_step_last: 0,
            late: [None; LATE_MAX],
        },
        tdr_budget_ms: c.tdr_budget_ms,
        tdr_explicit_budget_ms: c.tdr_explicit_budget_ms,
        tdr_lethal_ms: c.tdr_lethal_ms,
        tdr_bootstrap_steps: c.tdr_bootstrap_steps,
        admission: c.admission,
        pass_fixed_ms: crate::tunables::PASS_FIXED_MS_DEFAULT,
    }
}

#[test]
fn a_quick_frame_prices_the_pass_in_flight() {
    // With one pass in flight, a quick present proves the queue drained: the pass is priced at
    // the wall it accumulated (50 + this frame's 16.7 ms), its band earns a licence, and the next
    // dispatch is released.
    let d = decide_walk_price(&walk());
    assert_eq!(d.ledger.inflight, None);
    assert!((d.ledger.pass_dt - 66.7).abs() < 1e-9, "{}", d.ledger.pass_dt);
    assert_eq!(d.ledger.bands[3], 2048, "well under half the target: the fast lane doubles the licence");
    assert!(d.events.is_empty() && d.retreat.is_none());
}

/// The RTX 3080 trace at 1x on the (49,3) spiral (2026-10-14): a 545-iteration pin opening
/// dispatched at frame 498, its completion seen at 500 (35 ms of wall at 17.5 ms frames, so the
/// stand-in reads 0.0), its reading (12.5 ms) landing at 501.
fn released_pin_pass() -> WalkPriceInputs {
    WalkPriceInputs {
        interacting: true,
        pin_frame: true,
        frame: 500,
        frame_period_ms: 17.5,
        pin_pass_frame: 498,
        pin_inflight: 0,
        pin_dispatch_at: 1_000_000,
        pin_done_at: 1_035_000,
        ledger: WalkLedger { inflight: Some((545, 0, 6.7, false)), ..walk().ledger },
        ..walk()
    }
}

#[test]
fn a_pin_pass_released_before_its_reading_is_priced_by_that_reading() {
    let d = decide_walk_price(&released_pin_pass());
    assert_eq!(d.ledger.inflight, None, "released at its completion");
    assert_eq!(d.ledger.bands[0], 0, "not priced by a stand-in floored to 0.0");
    assert_eq!(d.ledger.late[0], Some((545, 0, 498, 0.0)));
    assert!(matches!(d.events[..], [WalkPriceEvent::PinDeferred { size: 545, .. }]), "{:?}", d.events);
    // The reading lands: 12.5 ms against the 20 ms target holds the licence near the pass
    // (×1.25), where the stand-in took the ×16 lane to 8,720.
    let next = WalkPriceInputs { frame: 501, late_readings: vec![(498, 12.5)], ledger: d.ledger, ..released_pin_pass() };
    let e = decide_walk_price(&WalkPriceInputs { ledger: WalkLedger { inflight: None, ..next.ledger }, ..next });
    assert_eq!(e.ledger.bands[0], 545 + 545 / 4);
    assert_eq!(e.ledger.late, [None; LATE_MAX]);
    assert!((e.ledger.pass_dt - 12.5).abs() < 1e-9);
}

#[test]
fn without_timestamps_the_stand_in_is_the_price_at_once() {
    let d = decide_walk_price(&WalkPriceInputs { no_timestamps: true, ..released_pin_pass() });
    assert_eq!(d.ledger.late, [None; LATE_MAX]);
    assert_eq!(d.ledger.bands[0], 545 * 16, "the completion stand-in is all such a device has");
}

#[test]
fn a_reading_that_never_comes_leaves_the_stand_in_standing() {
    let d = decide_walk_price(&released_pin_pass());
    let wait = crate::tunables::PIN_PRICE_WAIT_FRAMES;
    let quiet = |frame| WalkPriceInputs {
        frame,
        ledger: WalkLedger { inflight: None, ..d.ledger },
        ..released_pin_pass()
    };
    assert_eq!(decide_walk_price(&quiet(498 + wait - 1)).ledger.late, d.ledger.late, "still waiting");
    let e = decide_walk_price(&quiet(498 + wait));
    assert_eq!(e.ledger.late, [None; LATE_MAX]);
    assert_eq!(e.ledger.bands[0], 545 * 16);
}

#[test]
fn at_two_in_flight_every_pass_is_priced_by_its_own_reading() {
    // The RTX 3080 motiontest at two in flight priced 26 pin passes across 70 pins: each new
    // pass rewrote the in-flight record of the one before it. Frame 499: the pass sent at 498 is
    // still running (one completion owed), and the next goes out now.
    let i = WalkPriceInputs { frame: 499, pin_inflight: 1, pin_inflight_max: 2, ..released_pin_pass() };
    let d = decide_walk_price(&i);
    assert_eq!(d.ledger.inflight, None, "parked, not rewritten");
    assert_eq!(d.ledger.late[0], Some((545, 0, 498, -1.0)));
    // Its own reading lands two frames later and prices it.
    let e = decide_walk_price(&WalkPriceInputs { frame: 501, late_readings: vec![(498, 12.5)], ledger: d.ledger, ..i.clone() });
    assert_eq!(e.ledger.late, [None; LATE_MAX]);
    assert_eq!(e.ledger.bands[0], 545 + 545 / 4);
    // One that never gets a reading, and has no stand-in, goes unpriced after twice the wait.
    let wait = crate::tunables::PIN_PRICE_WAIT_FRAMES;
    let quiet = WalkPriceInputs { frame: 498 + 2 * wait, ledger: d.ledger, ..i.clone() };
    let f = decide_walk_price(&quiet);
    assert_eq!(f.ledger.late, [None; LATE_MAX]);
    assert_eq!(f.ledger.bands[0], 0);
    assert!(f.events.iter().any(|e| matches!(e, WalkPriceEvent::PinUnpriced { size: 545, .. })));
    // At one in flight the same frame holds the pass unpriced, as before.
    let g = decide_walk_price(&WalkPriceInputs { pin_inflight_max: 1, ..i.clone() });
    assert_eq!(g.ledger.inflight.map(|p| p.0), Some(545));
    assert_eq!(g.ledger.late, [None; LATE_MAX]);
}

#[test]
fn the_frame_rate_bounds_a_whole_moving_frame_but_not_a_pin() {
    // The RX 6800 XT's frame: 905×519 against a 13,761-iteration ask, motion_rate inflated by pin
    // readings (1.7e10 steps/ms) against whole moving frames' 6.2e7 (counted at the knee).
    let moving = WalkPriceInputs {
        interacting: true,
        spx: 905 * 519,
        gpu_iter: 13_761,
        tdr_steps: 6_000_000_000,
        motion_rate: 1.7e10,
        // A growth limiter far above the ask, so only the frame rate can bound the pass.
        ledger: WalkLedger { inflight: None, motion_step_last: 10_000, ..walk().ledger },
        ..walk()
    };
    let free = decide_walk_price(&moving);
    let bound = decide_walk_price(&WalkPriceInputs { frame_rate: 6.2e7, ..moving.clone() });
    assert!(bound.budget_step < free.budget_step, "{} vs {}", bound.budget_step, free.budget_step);
    assert!(bound.budget_step < 13_761, "the first pass stops short of the ask: a walk");
    // The pass size the ladder reads is unchanged.
    assert_eq!(bound.pass_steps, free.pass_steps);
    // A pin is untouched, and so is a card with no frame rate.
    let pin = WalkPriceInputs { pin_frame: true, ..moving.clone() };
    assert_eq!(
        decide_walk_price(&WalkPriceInputs { frame_rate: 6.2e7, ..pin.clone() }).budget_step,
        decide_walk_price(&pin).budget_step
    );
}

#[test]
fn a_slow_frame_holds_the_pass_unpriced() {
    // A frame past CHUNK_DRAIN_DT_MS proves nothing (the queue may still hold the pass): keep
    // accumulating, release nothing.
    let d = decide_walk_price(&WalkPriceInputs { last_dt_ms: 150.0, ..walk() });
    assert_eq!(d.ledger.inflight, Some((1024, 3, 200.0, false)));
    assert_eq!(d.ledger.bands, [0; crate::tunables::CHUNK_BANDS]);
}

#[test]
fn a_pass_in_the_lethal_band_sheds_the_licences_once() {
    let i = walk();
    let lethal = WalkPriceInputs {
        last_dt_ms: 150.0,
        ledger: WalkLedger {
            inflight: Some((1024, 3, i.tdr_lethal_ms, false)),
            bands: [4096; crate::tunables::CHUNK_BANDS],
            ..i.ledger
        },
        ..i
    };
    let d = decide_walk_price(&lethal);
    assert!(matches!(d.events[..], [WalkPriceEvent::LethalShed { size: 1024, band: 3, .. }]), "{:?}", d.events);
    assert!(d.retreat.is_some());
    assert_eq!(d.ledger.bands, [0; crate::tunables::CHUNK_BANDS], "every licence back to the floor");
    // Shed, not released: the pass stays in flight, latched, and the next frame sheds nothing more.
    assert!(matches!(d.ledger.inflight, Some((1024, 3, _, true))));
    let next = decide_walk_price(&WalkPriceInputs { ledger: d.ledger, ..lethal });
    assert!(next.events.is_empty() && next.retreat.is_none());
}

#[test]
fn motion_abandons_the_pass_in_flight_and_grows_its_step_gently() {
    let d = decide_walk_price(&WalkPriceInputs { interacting: true, ..walk() });
    assert_eq!(d.ledger.inflight, None, "a moving frame restarts the walk; the record goes too");
    assert_eq!(d.ledger.motion_step_last, d.budget_step);
    assert!(d.budget_step <= crate::tunables::MOTION_STEP_OPEN, "the opening step: {}", d.budget_step);
    // Never more than double the previous moving step (nor below the floor).
    let grown = decide_walk_price(&WalkPriceInputs {
        interacting: true,
        ledger: WalkLedger { motion_step_last: 300, ..walk().ledger },
        ..walk()
    });
    assert!((grown.floor..=600).contains(&grown.budget_step), "{}", grown.budget_step);
}

#[test]
fn an_unmeasured_walk_opens_at_any_floor() {
    // No measurement yet: the floor is 1, so the bootstrap bound sizes the opening pass at any
    // panel size (the 256 floor is earned by a measured budget).
    assert_eq!(decide_walk_price(&WalkPriceInputs { budget_measured: false, ..walk() }).floor, 1);
    assert_eq!(decide_walk_price(&walk()).floor, 256);
}

/// A settled view mid-walk (cursor 3,000 of 20,000) whose signature is already this frame's, with an
/// SA-seeded complete reference of 50,000 samples.
fn walk_id() -> WalkIdInputs {
    let mut i = WalkIdInputs {
        interacting: false,
        pin_frame: false,
        fe_walk: true,
        center: [-0.75, 0.1],
        magnification: 1.0e30,
        pos_sig: 0x1234_5678,
        settings_hash: 0xabcd,
        series_seed: 0x5eed,
        jitter_bits: [0, 0],
        gpu_iter: 20_000,
        resolution: super::super::Resolution::panel([1000, 800]),
        ss: 1,
        pass_steps: 4_000_000_000,
        has_ref: true,
        orbit_len: 50_000,
        sa_skip: 5_000,
        partial: false,
        chunk_charge: 1,
        state: WalkState {
            sig: super::super::WalkSig::NONE,
            walk_view: super::super::WalkSig::NONE,
            epoch: 7,
            running: None,
            tail_retries: 0,
            cursor: 3_000,
            idx: 4,
            last_range: Some([2_000, 3_000]),
            walk_end: 20_000,
            restarts: 0,
            restart_pending: false,
            count_walk: 0,
            motion_pass_steps_last: 0,
            motion_frames: 0,
        },
    };
    // Already this view's walk: its signatures are the ones this frame makes.
    let d = decide_walk_id(&WalkIdInputs { pin_frame: true, ..i.clone() });
    i.state.sig = d.sig;
    i.state.walk_view = decide_walk_id(&WalkIdInputs { pin_frame: true, jitter_bits: [0, 0], ..i.clone() }).sig;
    i
}

#[test]
fn the_same_view_resumes_its_walk() {
    let i = walk_id();
    let d = decide_walk_id(&i);
    assert!(!d.restarted && !d.clear_ledger && d.restart_lost.is_none());
    assert_eq!((d.cur, d.state.cursor, d.state.idx, d.state.epoch), (3_000, 3_000, 4, 7));
    assert_eq!(d.state.count_walk, 7, "a settled mode-2 walk counts its running pixels");
}

#[test]
fn every_input_the_signature_folds_restarts_the_walk() {
    // The composition, pinned: each input on its own moves the signature.
    let i = walk_id();
    let base = decide_walk_id(&i).sig;
    for (what, j) in [
        ("the centre's re", WalkIdInputs { center: [-0.7500001, 0.1], ..i.clone() }),
        ("the centre's im", WalkIdInputs { center: [-0.75, 0.1000001], ..i.clone() }),
        ("the magnification", WalkIdInputs { magnification: 1.1e30, ..i.clone() }),
        ("the reference length", WalkIdInputs { orbit_len: 50_001, ..i.clone() }),
        ("the exact position", WalkIdInputs { pos_sig: 0x1234_5679, ..i.clone() }),
        ("the settings", WalkIdInputs { settings_hash: 0xabce, ..i.clone() }),
        ("the series seed", WalkIdInputs { series_seed: 0x5eee, ..i.clone() }),
        ("the jitter", WalkIdInputs { jitter_bits: [1, 0], ..i.clone() }),
        ("the ask", WalkIdInputs { gpu_iter: 20_001, ..i.clone() }),
        ("the texture", WalkIdInputs { resolution: super::super::Resolution::panel([1000, 801]), ..i.clone() }),
        ("the ss", WalkIdInputs { ss: 2, ..i.clone() }),
    ] {
        let d = decide_walk_id(&j);
        assert_ne!(d.sig, base, "{what}");
        assert!(d.restarted && d.cur == 0, "{what}");
        assert_eq!(d.restart_lost, Some((3_000, 20_000, base)), "{what}: a part-way walk is counted lost");
        assert_eq!((d.state.restarts, d.state.epoch), (1, 8), "{what}");
    }
}

#[test]
fn a_new_supersampling_sample_keeps_the_band_ledger() {
    // The same view at a new jitter restarts the walk but keeps the ORBIT's prices; another view
    // clears them.
    let i = walk_id();
    let sample = decide_walk_id(&WalkIdInputs { jitter_bits: [0x3e9e_b852, 0xbe61_47ae], ..i.clone() });
    assert!(sample.restarted && !sample.clear_ledger);
    let other = decide_walk_id(&WalkIdInputs { magnification: 2.0e30, ..i });
    assert!(other.restarted && other.clear_ledger);
}

#[test]
fn motion_restarts_every_frame_and_a_pin_never_does() {
    let i = walk_id();
    let moving = decide_walk_id(&WalkIdInputs { interacting: true, ..i.clone() });
    assert!(moving.restarted && !moving.clear_ledger, "the ledger survives the pins of a glide");
    assert_eq!((moving.walk_floor, moving.walk_end), (0, 20_000), "motion walks [0, ask)");
    assert_eq!(moving.state.motion_pass_steps_last, 4_000_000_000);
    assert_eq!(moving.state.motion_frames, 1);
    // A pin owns its progression: a moved signature does not restart it.
    let pin = decide_walk_id(&WalkIdInputs { pin_frame: true, interacting: true, magnification: 2.0e30, ..i });
    assert!(!pin.restarted);
    assert_eq!(pin.cur, 3_000);
}

#[test]
fn a_settled_walk_covers_only_its_live_range() {
    let i = walk_id();
    let d = decide_walk_id(&i);
    let we = super::super::sa_skip_rescue(20_000, 5_000, 50_000);
    assert_eq!((d.walk_floor, d.walk_end), (super::super::usable_sa_skip(5_000, we), we));
    assert_eq!(d.state.walk_end, d.walk_end);
    // A partial reference caps the walk at the orbit it holds.
    let partial = decide_walk_id(&WalkIdInputs { partial: true, orbit_len: 12_000, ..i.clone() });
    assert!(partial.walk_end <= 11_999, "{}", partial.walk_end);
    // No reference yet: the whole ask from 0.
    let none = decide_walk_id(&WalkIdInputs { has_ref: false, ..i });
    assert_eq!((none.walk_floor, none.walk_end), (0, 20_000));
}

/// A settled walk at cursor 3,000 of [0, 20,000), its last pass priced (nothing in flight), every
/// band still cold, a 1000×800 panel at ss 1.
fn dispatch() -> WalkDispatchInputs {
    WalkDispatchInputs {
        interacting: false,
        pin_frame: false,
        fe_walk: false,
        cur: 3_000,
        walk_floor: 0,
        walk_end: 20_000,
        floor: 256,
        pass_steps: 4_000_000_000,
        budget_step: 5_000,
        gpu_iter: 20_000,
        spx: 1000 * 800,
        ss: 1,
        frame: 500,
        now_us: 1_000_000,
        last_dt_ms: 16.7,
        worst_rate: Some(2.0e7),
        bands: [0; crate::tunables::CHUNK_BANDS],
        orbit_len: 50_000,
        epoch: 7,
        running: None,
        knee_px: 0,
        accum_active: false,
        walk_read_sig: 0,
        norm_sig_submit: 0,
        pin_inflight_max: 1,
        pin_timeout_scaled: 1,
        chunk_charge: 1,
        chunk_shuffle: 0,
        state: DispatchState {
            pin_inflight: 0,
            pin_reg_pending: 0,
            pin_dispatch_at: 0,
            pin_escapes: 0,
            pin_pass_steps: 0,
            pin_pass_frame: 0,
            pin_pass_gpu_ms: -1.0,
            last_range: Some([2_000, 3_000]),
            inflight: None,
            idx: 4,
            cursor: 3_000,
            pending: false,
            charged_px: 0,
            fe_steps_last: 0,
            tail_retries: 0,
            probe_nonce: 0,
            norm_complete_submit: true,
        },
    }
}

#[test]
fn a_settled_pass_goes_out_at_its_band_licence_and_waits_to_be_priced() {
    let d = decide_walk_dispatch(&dispatch());
    // A cold band licenses the floor: one floor-sized pass meets a cost cliff, not a budget-sized one.
    assert_eq!(d.chunk_range, Some([3_000, 3_256]));
    assert_eq!((d.chunk_idx, d.pass), (4, true));
    assert_eq!((d.state.cursor, d.state.idx, d.state.pending), (3_256, 5, true));
    assert_eq!(d.state.inflight, Some((256, 4, 0.0, false)), "in flight, unpriced, in band 4");
    assert_eq!(d.state.last_range, d.chunk_range);
    assert_eq!(d.state.fe_steps_last, 800_000 * 256, "the measurement pairs with the work done");
    assert!(!d.state.norm_complete_submit, "a part of the walk is not the picture");
}

#[test]
fn one_unpriced_pass_holds_the_settled_walk() {
    // Hold the last dispatched range: the GPU dedupes it and the drain frame prices the pass.
    let mut i = dispatch();
    i.state.inflight = Some((256, 4, 30.0, false));
    let d = decide_walk_dispatch(&i);
    assert_eq!((d.chunk_range, d.chunk_idx, d.pass), (Some([2_000, 3_000]), 4, false));
    assert_eq!((d.state.cursor, d.state.pending), (3_000, true));
}

#[test]
fn a_moving_frame_previews_without_advancing_the_walk() {
    let d = decide_walk_dispatch(&WalkDispatchInputs { interacting: true, ..dispatch() });
    assert!(d.pass && d.chunk_range.is_some_and(|[s, e]| s == 3_000 && e > s));
    assert_eq!((d.state.cursor, d.state.idx, d.state.inflight), (3_000, 4, None));
    assert!(d.state.norm_complete_submit, "a moving frame's one-pass preview IS what is displayed");
}

#[test]
fn a_pin_pass_escapes_a_completion_that_never_comes() {
    // One pin pass in the queue for 500 ms, registered, at a 16.7 ms frame: past the 170 ms gate.
    let mut i = WalkDispatchInputs { pin_frame: true, interacting: true, ..dispatch() };
    i.state.pin_inflight = 1;
    i.state.pin_dispatch_at = 500_000;
    let d = decide_walk_dispatch(&i);
    assert_eq!(
        d.events.first(),
        Some(&DispatchEvent::PinEscape { age_us: 500_000, limit_us: 170_000, registered: true, n: 1 })
    );
    assert!(d.pass && d.pin_dispatched, "released: the next pin pass goes out");
    assert_eq!((d.state.pin_inflight, d.state.pin_reg_pending, d.state.pin_escapes), (1, 1, 1));
    assert_eq!((d.state.pin_pass_frame, d.state.pin_pass_gpu_ms), (500, -1.0));
    assert_eq!(Some(d.state.cursor), d.chunk_range.map(|[_, e]| e), "a pin advances its walk");
    // Inside the gate it holds.
    i.state.pin_dispatch_at = 900_000;
    let held = decide_walk_dispatch(&i);
    assert!(!held.pass && held.events.is_empty() && !held.pin_dispatched);
}

#[test]
fn a_finished_walk_resends_its_tail_until_its_reading_lands() {
    let i = WalkDispatchInputs { cur: 20_000, walk_read_sig: 1, norm_sig_submit: 2, ..dispatch() };
    let d = decide_walk_dispatch(&i);
    assert_eq!((d.chunk_range, d.pass), (Some([20_000, 20_000]), false));
    assert_eq!((d.state.probe_nonce, d.state.tail_retries), (1, 1), "a fresh nonce re-sends the tail");
    assert!(d.state.norm_complete_submit);
    // Its reading has landed: nothing more to send.
    let landed = decide_walk_dispatch(&WalkDispatchInputs { walk_read_sig: 2, ..i });
    assert_eq!((landed.state.probe_nonce, landed.state.tail_retries), (0, 0));
}

#[test]
fn a_recorded_decision_round_trips() {
    let i = MovingInputs { esc_hist: Some(vec![0, 3, 7, 0]), ..moving() };
    let d = decide_moving(&i);
    let back_i: MovingInputs = serde_json::from_str(&serde_json::to_string(&i).unwrap()).unwrap();
    let back_d: MovingDecision = serde_json::from_str(&serde_json::to_string(&d).unwrap()).unwrap();
    assert_eq!(back_i, i);
    assert_eq!(back_d, d);
    assert_eq!(decide_moving(&back_i), d, "a replayed decision is the recorded one");
}
