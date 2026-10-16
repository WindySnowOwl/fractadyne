use super::*;

fn t0() -> Instant {
    Instant::now()
}

fn secs(base: Instant, s: u64) -> Instant {
    base + Duration::from_secs(s)
}

fn assigns(cmds: &[Command]) -> Vec<(ClientId, RunId, u64, u64)> {
    cmds.iter()
        .filter_map(|c| match c {
            Command::Assign { client, run, start, end } => Some((*client, *run, *start, *end)),
            _ => None,
        })
        .collect()
}

fn verify(s: &mut Scheduler, now: Instant, client: ClientId, run: RunId, range: std::ops::Range<u64>) -> Vec<Command> {
    let mut out = Vec::new();
    for i in range {
        out.extend(s.step(now, Event::FrameVerified { client, run, index: i, render_ms: 1000 }));
    }
    out
}

#[test]
fn the_first_client_starts_at_frame_zero_and_the_second_mid_gap() {
    let b = t0();
    let mut s = Scheduler::new(Config::new(100), &[]);
    let a = assigns(&s.step(b, Event::Joined { client: 1, name: "A".into() }));
    assert_eq!(a, vec![(1, 1, 0, 8), (1, 2, 8, 16)], "first run plus the prefetched second");
    let bb = assigns(&s.step(b, Event::Joined { client: 2, name: "B".into() }));
    // Largest gap [16, 100), approached by A: B starts in its middle.
    assert_eq!(bb[0].2, 16 + 84 / 2);
    assert_eq!(bb.len(), 2);
}

#[test]
fn a_finished_run_is_followed_by_the_next_at_the_measured_pace() {
    let b = t0();
    let mut cfg = Config::new(1000);
    cfg.run_target = Duration::from_secs(10);
    let mut s = Scheduler::new(cfg, &[]);
    s.step(b, Event::Joined { client: 1, name: "A".into() });
    let out = verify(&mut s, secs(b, 8), 1, 1, 0..8);
    // 1 s a frame, 10 s target → runs of 10, continuing from where run 2 ends (16).
    assert_eq!(assigns(&out), vec![(1, 3, 16, 26)]);
}

#[test]
fn a_frame_is_accepted_once_and_a_late_copy_discarded() {
    let b = t0();
    let mut s = Scheduler::new(Config::new(20), &[]);
    s.step(b, Event::Joined { client: 1, name: "A".into() });
    let first = s.step(b, Event::FrameVerified { client: 1, run: 1, index: 3, render_ms: 10 });
    assert!(first.contains(&Command::Accept { index: 3 }));
    let again = s.step(b, Event::FrameVerified { client: 1, run: 1, index: 3, render_ms: 10 });
    assert!(again.contains(&Command::Discard { index: 3 }));
    assert!(!again.contains(&Command::Accept { index: 3 }));
}

#[test]
fn an_unreachable_client_loses_its_frames_to_the_others() {
    let b = t0();
    let mut s = Scheduler::new(Config::new(40), &[]);
    s.step(b, Event::Joined { client: 1, name: "A".into() });
    s.step(b, Event::Joined { client: 2, name: "B".into() });
    // B heartbeats; A goes silent past the 10 s timeout.
    s.step(secs(b, 9), Event::Heartbeat { client: 2, paused: false, frame: None, frame_ms: None });
    let out = s.step(secs(b, 11), Event::Tick);
    assert!(out.iter().any(|c| matches!(c, Command::Note(n) if n.contains("A is unreachable"))), "{out:?}");
    let snap = s.snapshot();
    assert_eq!(snap.clients.iter().find(|c| c.id == 1).unwrap().state, ClientState::Gone);
    // B finishes its two runs, then picks up A's old frames.
    let runs_b: Vec<_> = snap.clients.iter().find(|c| c.id == 2).unwrap().runs.clone();
    let mut later = Vec::new();
    for (run, st, en) in runs_b {
        later.extend(verify(&mut s, secs(b, 12), 2, run, st..en));
    }
    assert!(assigns(&later).iter().any(|a| a.0 == 2 && a.2 < 16), "A's frames were never reassigned: {later:?}");
}

#[test]
fn a_stalled_run_is_cancelled_and_requeued() {
    let b = t0();
    let mut cfg = Config::new(30);
    cfg.stall_floor = Duration::from_secs(300);
    let mut s = Scheduler::new(cfg, &[]);
    s.step(b, Event::Joined { client: 1, name: "A".into() });
    for t in (0..=300).step_by(5) {
        s.step(secs(b, t), Event::Heartbeat { client: 1, paused: false, frame: Some(0), frame_ms: Some(t * 1000) });
    }
    assert!(!s.step(secs(b, 300), Event::Tick).iter().any(|c| matches!(c, Command::Cancel { .. })), "cancelled at the timeout itself");
    s.step(secs(b, 301), Event::Heartbeat { client: 1, paused: false, frame: Some(0), frame_ms: Some(301_000) });
    let out = s.step(secs(b, 302), Event::Tick);
    assert!(out.contains(&Command::Cancel { client: 1, run: Some(1), reason: CancelReason::Stalled }), "{out:?}");
}

/// A long frame is not a stall once frames have shown how long they take.
#[test]
fn the_stall_timeout_grows_with_the_jobs_frame_time() {
    let b = t0();
    let mut s = Scheduler::new(Config::new(30), &[]);
    s.step(b, Event::Joined { client: 1, name: "A".into() });
    s.step(b, Event::FrameVerified { client: 1, run: 1, index: 0, render_ms: 120_000 });
    assert_eq!(s.stall_timeout(), Duration::from_secs(960), "8 × the 2-minute median");
}

/// ⭐A bad frame goes to ANOTHER machine; the second strike removes the sender, which stays removed
/// across a reconnect until the user re-admits it.
#[test]
fn bad_frames_are_reallocated_and_two_strikes_remove_the_sender() {
    let b = t0();
    let mut s = Scheduler::new(Config::new(16), &[]);
    s.step(b, Event::Joined { client: 1, name: "A".into() });
    s.step(b, Event::Joined { client: 2, name: "B".into() });
    let out = s.step(b, Event::FrameBad { client: 1, run: 1, index: 2, why: "digest".into() });
    assert!(out.iter().any(|c| matches!(c, Command::Note(n) if n.contains("strike 1"))));
    // Frame 2 is no longer A's, and never goes back to it.
    assert_ne!(s.owner(2), Some(1), "frame 2 stayed with its bad sender");
    let out = s.step(b, Event::FrameBad { client: 1, run: 1, index: 3, why: "size".into() });
    assert!(out.iter().any(|c| matches!(c, Command::Remove { client: 1, strikes: 2, .. })), "{out:?}");
    assert!(out.iter().any(|c| matches!(c, Command::Cancel { client: 1, run: None, reason: CancelReason::Removed })));
    // Reconnecting under the same name does not undo it…
    let out = s.step(b, Event::Joined { client: 3, name: "A".into() });
    assert!(out.iter().any(|c| matches!(c, Command::Remove { client: 3, .. })), "a removed machine rejoined by reconnecting");
    // …re-admission does.
    s.step(b, Event::Readmit { name: "A".into() });
    let out = s.step(b, Event::Joined { client: 4, name: "A".into() });
    assert!(!out.iter().any(|c| matches!(c, Command::Remove { .. })));
    // B finishes its runs and then picks up frames 2 and 3; A (re-admitted, as client 4) must not.
    for _ in 0..10 {
        let runs: Vec<_> = s.snapshot().clients.into_iter().filter(|c| c.name == "B").flat_map(|c| c.runs).collect();
        for (run, st, en) in runs {
            verify(&mut s, b, 2, run, st..en);
        }
        for f in [2u64, 3] {
            assert_ne!(s.owner(f), Some(4), "frame {f} went back to the machine that sent it bad");
        }
    }
    assert!(s.is_finished() || s.snapshot().pending + s.snapshot().assigned > 0);
}

/// The regression the scenario above found by accident: a client that crashes twice must give back
/// its PREFETCHED run too, or those frames are never rendered and the job never ends.
#[test]
fn a_parked_client_gives_back_every_run_and_the_job_still_finishes() {
    let b = t0();
    let mut s = Scheduler::new(Config::new(40), &[]);
    s.step(b, Event::Joined { client: 1, name: "A".into() });
    s.step(b, Event::Joined { client: 2, name: "B".into() });
    s.step(secs(b, 1), Event::RunAborted { client: 1, run: 1, why: AbortKind::Crashed });
    let front = s.snapshot().clients.iter().find(|c| c.id == 1).unwrap().runs[0].0;
    let out = s.step(secs(b, 2), Event::RunAborted { client: 1, run: front, why: AbortKind::Crashed });
    assert!(out.contains(&Command::Cancel { client: 1, run: None, reason: CancelReason::Reassigned }), "{out:?}");
    assert!(s.snapshot().clients.iter().find(|c| c.id == 1).unwrap().runs.is_empty());
    // B alone finishes the whole job.
    for _ in 0..200 {
        if s.is_finished() {
            break;
        }
        let runs: Vec<_> = s.snapshot().clients.into_iter().filter(|c| c.id == 2).flat_map(|c| c.runs).collect();
        for (run, st, en) in runs {
            verify(&mut s, secs(b, 3), 2, run, st..en);
        }
    }
    assert!(s.is_finished(), "{:?}", s.snapshot());
}

#[test]
fn a_user_pause_requeues_and_waits_for_the_client() {
    let b = t0();
    let mut s = Scheduler::new(Config::new(50), &[]);
    s.step(b, Event::Joined { client: 1, name: "A".into() });
    s.step(b, Event::FrameVerified { client: 1, run: 1, index: 0, render_ms: 5 });
    let out = s.step(b, Event::RunAborted { client: 1, run: 1, why: AbortKind::UserPaused });
    assert!(assigns(&out).is_empty(), "work went to a paused client");
    assert_eq!(s.snapshot().clients[0].state, ClientState::Paused);
    assert!(assigns(&s.step(b, Event::Heartbeat { client: 1, paused: true, frame: None, frame_ms: None })).is_empty());
    // Its user resumes: the next heartbeat says so, and work flows again.
    let out = s.step(b, Event::Heartbeat { client: 1, paused: false, frame: None, frame_ms: None });
    assert_eq!(s.snapshot().clients[0].state, ClientState::Active);
    assert!(!assigns(&out).is_empty(), "a resumed client got no work");
}

#[test]
fn a_parked_client_is_given_work_again_after_a_quiet_crash_window() {
    let b = t0();
    let mut s = Scheduler::new(Config::new(40), &[]);
    let a = assigns(&s.step(b, Event::Joined { client: 1, name: "A".into() }));
    s.step(secs(b, 1), Event::RunAborted { client: 1, run: a[0].1, why: AbortKind::Crashed });
    let again = assigns(&s.step(secs(b, 2), Event::Tick));
    let run = again.first().map(|x| x.1).or(a.get(1).map(|x| x.1)).expect("a run to crash");
    s.step(secs(b, 3), Event::RunAborted { client: 1, run, why: AbortKind::Crashed });
    assert_eq!(s.snapshot().clients[0].state, ClientState::Unstable);
    // Within the window: still parked. After it, without another crash: working again.
    s.step(secs(b, 300), Event::Heartbeat { client: 1, paused: false, frame: None, frame_ms: None });
    assert!(assigns(&s.step(secs(b, 300), Event::Tick)).is_empty());
    s.step(secs(b, 700), Event::Heartbeat { client: 1, paused: false, frame: None, frame_ms: None });
    let back = s.step(secs(b, 700), Event::Tick);
    assert_eq!(s.snapshot().clients[0].state, ClientState::Active, "{back:?}");
    assert!(!assigns(&back).is_empty(), "{back:?}");
}

#[test]
fn two_crashes_in_ten_minutes_park_a_client() {
    let b = t0();
    let mut s = Scheduler::new(Config::new(100), &[]);
    s.step(b, Event::Joined { client: 1, name: "A".into() });
    s.step(secs(b, 10), Event::RunAborted { client: 1, run: 1, why: AbortKind::Crashed });
    assert_eq!(s.snapshot().clients[0].state, ClientState::Active, "one crash parks nobody");
    let run = s.snapshot().clients[0].runs[0].0;
    s.step(secs(b, 70), Event::RunAborted { client: 1, run, why: AbortKind::Crashed });
    assert_eq!(s.snapshot().clients[0].state, ClientState::Unstable);
    assert!(s.snapshot().clients[0].runs.is_empty());
}

#[test]
fn low_storage_holds_new_work_until_space_returns() {
    let b = t0();
    let mut s = Scheduler::new(Config::new(100), &[]);
    s.step(b, Event::StorageLow(true));
    assert!(assigns(&s.step(b, Event::Joined { client: 1, name: "A".into() })).is_empty());
    assert!(!assigns(&s.step(b, Event::StorageLow(false))).is_empty());
}

#[test]
fn a_resumed_job_assigns_only_what_is_missing_and_finishes() {
    let b = t0();
    let done: Vec<u64> = (0..10).filter(|i| *i != 4 && *i != 7).collect();
    let mut s = Scheduler::new(Config::new(10), &done);
    let a = assigns(&s.step(b, Event::Joined { client: 1, name: "A".into() }));
    let covered: Vec<u64> = a.iter().flat_map(|x| x.2..x.3).collect();
    assert_eq!(covered, vec![4, 7]);
    let mut out = Vec::new();
    for (c, r, st, en) in a {
        out.extend(verify(&mut s, b, c, r, st..en));
    }
    assert!(out.contains(&Command::Done { failed: vec![] }), "{out:?}");
}

#[test]
fn a_frame_failing_on_two_machines_is_given_up_and_the_job_still_ends() {
    let b = t0();
    let mut s = Scheduler::new(Config::new(2), &[]);
    s.step(b, Event::Joined { client: 1, name: "A".into() });
    s.step(b, Event::Joined { client: 2, name: "B".into() });
    s.step(b, Event::FrameFailed { client: 1, run: 1, index: 0, why: "disk".into() });
    let snap = s.snapshot();
    let b_run = snap.clients.iter().find(|c| c.name == "B").and_then(|c| c.runs.iter().find(|r| r.1 == 0).copied());
    let run_b = b_run.map(|r| r.0).unwrap_or(0);
    let out = s.step(b, Event::FrameFailed { client: 2, run: run_b, index: 0, why: "disk".into() });
    assert!(out.iter().any(|c| matches!(c, Command::Note(n) if n.contains("given up"))), "{out:?}");
    let out = s.step(b, Event::FrameVerified { client: 1, run: 1, index: 1, render_ms: 1 });
    assert!(out.contains(&Command::Done { failed: vec![0] }), "{out:?}");
}

#[test]
fn a_frame_past_the_hard_deadline_cancels_its_run() {
    let b = t0();
    let mut cfg = Config::new(20);
    cfg.deadline = Some(Duration::from_secs(60));
    let mut s = Scheduler::new(cfg, &[]);
    s.step(b, Event::Joined { client: 1, name: "A".into() });
    let out = s.step(secs(b, 61), Event::Heartbeat { client: 1, paused: false, frame: Some(0), frame_ms: Some(61_000) });
    assert!(out.contains(&Command::Cancel { client: 1, run: Some(1), reason: CancelReason::Deadline }), "{out:?}");
}

#[test]
fn stop_cancels_everything_and_ends_the_job() {
    let b = t0();
    let mut s = Scheduler::new(Config::new(20), &[]);
    s.step(b, Event::Joined { client: 1, name: "A".into() });
    let out = s.step(b, Event::Stop);
    assert!(out.contains(&Command::Cancel { client: 1, run: None, reason: CancelReason::Stopped }));
    assert!(out.iter().any(|c| matches!(c, Command::Done { failed } if failed.len() == 20)), "{out:?}");
    assert!(s.is_finished());
}

/// ⭐⭐EVERY FRAME EXACTLY ONCE — over random joins, departures, silences, crashes, pauses, bad and
/// failed frames and late deliveries. A frame is never accepted twice, and the job always ends
/// with every frame accepted exactly once or given up.
#[test]
fn every_frame_exactly_once_under_random_chaos() {
    for seed in 1..=60u64 {
        chaos(seed);
    }
}

struct SimClient {
    name: String,
    alive: bool,
    silent_until: Option<u64>,
    paused: bool,
    /// (run, next frame to render, end)
    queue: VecDeque<(RunId, u64, u64)>,
    steady: bool,
}

fn chaos(seed: u64) {
    let mut rng = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
    let mut rand = move |m: u64| {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng % m
    };
    let frames = 30 + rand(170);
    let mut cfg = Config::new(frames);
    cfg.first_run = 1 + rand(8);
    cfg.stall_floor = Duration::from_secs(40);
    // Dissolves (windows of frames no run may start at) and held shots.
    let mut no_start = vec![false; frames as usize];
    for _ in 0..rand(4) {
        let a = 1 + rand(frames - 1);
        for i in a..(a + 1 + rand(6)).min(frames) {
            no_start[i as usize] = true;
        }
    }
    cfg.no_start = no_start.clone();
    for _ in 0..rand(3) {
        let a = rand(frames);
        cfg.holds.push((a, (a + 2 + rand(20)).min(frames)));
    }
    let mut s = Scheduler::new(cfg, &[]);
    let base = t0();
    let mut t = 0u64;
    let mut accepted: HashMap<u64, u32> = HashMap::new();
    let mut failed: Vec<u64> = Vec::new();
    let mut clients: HashMap<ClientId, SimClient> = HashMap::new();
    let mut next_id: ClientId = 1;
    let names = ["steady", "flaky", "slow", "liar", "laptop"];
    let mut done = false;

    let apply = |cmds: Vec<Command>, clients: &mut HashMap<ClientId, SimClient>, accepted: &mut HashMap<u64, u32>, failed: &mut Vec<u64>, done: &mut bool| {
        for c in cmds {
            match c {
                Command::Assign { client, run, start, end } => {
                    assert!(!no_start[start as usize], "seed {seed}: a run starts at frame {start}, mid-dissolve");
                    if let Some(sc) = clients.get_mut(&client) {
                        sc.queue.push_back((run, start, end));
                    }
                }
                Command::Trim { client, run, from } => {
                    assert!(!no_start.get(from as usize).copied().unwrap_or(false), "seed {seed}: a run cut at {from}, mid-dissolve");
                    if let Some(sc) = clients.get_mut(&client) {
                        for q in sc.queue.iter_mut().filter(|q| q.0 == run) {
                            q.2 = q.2.min(from);
                        }
                        sc.queue.retain(|q| q.1 < q.2);
                    }
                }
                Command::Cancel { client, run, .. } => {
                    if let Some(sc) = clients.get_mut(&client) {
                        match run {
                            Some(r) => sc.queue.retain(|q| q.0 != r),
                            None => sc.queue.clear(),
                        }
                    }
                }
                Command::Remove { client, .. } | Command::Drop { client, .. } => {
                    if let Some(sc) = clients.get_mut(&client) {
                        sc.alive = false;
                        sc.queue.clear();
                    }
                }
                Command::Accept { index } => {
                    let n = accepted.entry(index).or_insert(0);
                    *n += 1;
                    assert_eq!(*n, 1, "seed {seed}: frame {index} accepted twice");
                }
                Command::Done { failed: f } => {
                    *failed = f;
                    *done = true;
                }
                Command::Discard { .. } | Command::Note(_) => {}
            }
        }
    };

    for step in 0..40_000 {
        if done {
            break;
        }
        // Keep the farm populated: one steady client always exists; others come and go.
        if !clients.values().any(|c| c.alive && c.steady) || (clients.values().filter(|c| c.alive).count() < 4 && rand(50) == 0) {
            let steady = !clients.values().any(|c| c.alive && c.steady);
            let name = if steady { "steady".to_string() } else { names[1 + rand(4) as usize].to_string() };
            let id = next_id;
            next_id += 1;
            clients.insert(id, SimClient { name: name.clone(), alive: true, silent_until: None, paused: false, queue: VecDeque::new(), steady });
            // Two GPU classes; the steady machine is always A.
            s.set_class(id, Some(if steady || rand(2) == 0 { "A".to_string() } else { "B".to_string() }));
            let out = s.step(secs(base, t), Event::Joined { client: id, name });
            apply(out, &mut clients, &mut accepted, &mut failed, &mut done);
        }
        let roll = rand(100);
        if roll < 70 {
            // A client renders the next frame of its current run.
            let ids: Vec<ClientId> = clients.iter().filter(|(_, c)| c.alive && !c.paused && !c.queue.is_empty()).map(|(&i, _)| i).collect();
            if ids.is_empty() {
                continue;
            }
            let id = ids[rand(ids.len() as u64) as usize];
            let sc = clients.get_mut(&id).unwrap();
            let front = sc.queue.front_mut().unwrap();
            let (run, f, end) = *front;
            front.1 += 1;
            if f + 1 >= end {
                sc.queue.pop_front();
            }
            let steady = sc.steady;
            let liar = sc.name == "liar";
            let r = rand(100);
            let ev = if !steady && (r < 4 || (liar && r < 30)) {
                Event::FrameBad { client: id, run, index: f, why: "sim".into() }
            } else if !steady && r < 7 {
                Event::FrameFailed { client: id, run, index: f, why: "sim".into() }
            } else {
                Event::FrameVerified { client: id, run, index: f, render_ms: 500 + rand(3000) }
            };
            let out = s.step(secs(base, t), ev);
            apply(out, &mut clients, &mut accepted, &mut failed, &mut done);
        } else if roll < 74 {
            // A non-steady client misbehaves.
            let ids: Vec<ClientId> = clients.iter().filter(|(_, c)| c.alive && !c.steady).map(|(&i, _)| i).collect();
            if ids.is_empty() {
                continue;
            }
            let id = ids[rand(ids.len() as u64) as usize];
            let what = rand(5);
            let sc = clients.get_mut(&id).unwrap();
            let ev = match what {
                0 => {
                    sc.alive = false;
                    Some(Event::Left { client: id, why: LeaveKind::Left })
                }
                1 => {
                    sc.silent_until = Some(t + 5 + rand(30)); // may or may not cross the timeout
                    None
                }
                2 => sc.queue.pop_front().map(|q| Event::RunAborted { client: id, run: q.0, why: AbortKind::Crashed }),
                3 => {
                    let front = sc.queue.front().map(|q| q.0);
                    sc.queue.clear();
                    front.map(|run| {
                        sc.paused = true;
                        Event::RunAborted { client: id, run, why: AbortKind::UserPaused }
                    })
                }
                _ => {
                    sc.paused = false;
                    Some(Event::Heartbeat { client: id, paused: false, frame: None, frame_ms: None })
                }
            };
            if let Some(ev) = ev {
                let out = s.step(secs(base, t), ev);
                apply(out, &mut clients, &mut accepted, &mut failed, &mut done);
            }
        } else {
            // Time passes; heartbeats from everyone not silent; a tick.
            t += 1 + rand(4);
            let ids: Vec<ClientId> = clients.iter().filter(|(_, c)| c.alive && c.silent_until.is_none_or(|u| t >= u)).map(|(&i, _)| i).collect();
            for id in ids {
                let paused = clients[&id].paused;
                let out = s.step(secs(base, t), Event::Heartbeat { client: id, paused, frame: None, frame_ms: None });
                apply(out, &mut clients, &mut accepted, &mut failed, &mut done);
            }
            let out = s.step(secs(base, t), Event::Tick);
            apply(out, &mut clients, &mut accepted, &mut failed, &mut done);
            // A client the scheduler gave up on as unreachable reconnects as a new connection.
            for c in clients.values_mut() {
                if c.silent_until.is_some_and(|u| t >= u) {
                    c.silent_until = None;
                }
            }
        }
        let _ = step;
    }
    assert!(done, "seed {seed}: the job never finished ({:?})", s.snapshot());
    for i in 0..frames {
        let n = accepted.get(&i).copied().unwrap_or(0);
        assert!(n == 1 || (n == 0 && failed.contains(&i)), "seed {seed}: frame {i} accepted {n} times, failed={}", failed.contains(&i));
    }
}

#[test]
fn the_strip_shows_each_cells_worst_state_and_never_more_cells_than_frames() {
    let b = t0();
    let mut s = Scheduler::new(Config::new(10), &[0, 1, 2, 3]);
    assert_eq!(s.strip(10), "dddd......");
    assert_eq!(s.strip(100), "dddd......", "one cell per frame at most");
    assert_eq!(s.strip(5), "dd...", "two frames a cell");
    s.step(b, Event::Joined { client: 1, name: "A".into() });
    // The first run starts at the gap (frame 4); one cell per frame agrees with the counts.
    let snap = s.snapshot();
    let full = s.strip(10);
    assert!(full.starts_with("ddddaaaa"), "{full}");
    assert_eq!(full.matches('a').count() as u64, snap.assigned);
    assert_eq!(full.matches('.').count() as u64, snap.pending);
    assert_eq!(s.strip(3), "daa", "a cell with a done and an assigned frame shows assigned");
    assert_eq!(s.strip(0), "");
    assert_eq!(Scheduler::new(Config::new(0), &[]).strip(8), "");
}

#[test]
fn no_run_starts_inside_a_dissolve_and_a_requeued_dissolve_frame_leads_in() {
    let b = t0();
    let mut cfg = Config::new(40);
    // Frames 6..12 are mid-dissolve: each blends with frame 5, rendered in the same process.
    cfg.no_start = (0..40).map(|i| (6..12).contains(&i)).collect();
    let mut s = Scheduler::new(cfg, &[]);
    let a = assigns(&s.step(b, Event::Joined { client: 1, name: "A".into() }));
    assert_eq!(a[0], (1, 1, 0, 12), "the first run (8 frames) would end mid-dissolve: it runs on through it");
    assert_eq!(a[1].2, 12);

    // A short job, all one run; frame 9 comes back (a bad copy). The run that renders it again
    // starts at frame 5, the frame the dissolve rises through, and owns only frame 9.
    let mut cfg = Config::new(12);
    cfg.no_start = (0..12).map(|i| (6..12).contains(&i)).collect();
    let mut s = Scheduler::new(cfg, &[]);
    assert_eq!(assigns(&s.step(b, Event::Joined { client: 1, name: "A".into() })), vec![(1, 1, 0, 12)]);
    verify(&mut s, b, 1, 1, 0..9);
    s.step(b, Event::FrameBad { client: 1, run: 1, index: 9, why: "x".into() });
    let bb = assigns(&s.step(b, Event::Joined { client: 2, name: "B".into() }));
    assert_eq!(bb, vec![(2, 2, 5, 10)]);
    // Its copy of frame 5 is discarded without a "late copy" note; frame 9 is accepted.
    let five = s.step(b, Event::FrameVerified { client: 2, run: 2, index: 5, render_ms: 10 });
    assert!(five.contains(&Command::Discard { index: 5 }));
    assert!(!five.iter().any(|c| matches!(c, Command::Note(n) if n.contains("late copy"))), "{five:?}");
    assert!(s.step(b, Event::FrameVerified { client: 2, run: 2, index: 9, render_ms: 10 }).contains(&Command::Accept { index: 9 }));
}

/// Client `id` renders every frame it is given, and whatever it is given next, until it gets
/// nothing more; the other clients render nothing. Every command along the way, in order.
fn drain(s: &mut Scheduler, b: Instant, id: ClientId, mut out: Vec<Command>) -> Vec<Command> {
    let mut log = out.clone();
    for _ in 0..1000 {
        let mine: Vec<(ClientId, RunId, u64, u64)> = assigns(&out).into_iter().filter(|a| a.0 == id).collect();
        if mine.is_empty() {
            break;
        }
        out = Vec::new();
        for (_, run, s0, e) in mine {
            out.extend(verify(s, b, id, run, s0..e));
        }
        log.extend(out.iter().cloned());
    }
    log
}

#[test]
fn an_idle_client_takes_a_queued_run_whole_before_cutting_a_running_one() {
    let b = t0();
    let mut s = Scheduler::new(Config::new(16), &[]);
    let a = assigns(&s.step(b, Event::Joined { client: 1, name: "A".into() }));
    assert_eq!(a.len(), 2, "A renders one run and holds the next: {a:?}");
    let queued = a[1];
    // B renders everything it can; A renders nothing. Once nothing is pending, B first takes A's
    // queued run whole (no work lost) — and only later cuts A's running one.
    let out = s.step(b, Event::Joined { client: 2, name: "B".into() });
    let log = drain(&mut s, b, 2, out);
    let cancel = log.iter().position(|c| *c == Command::Cancel { client: 1, run: Some(queued.1), reason: CancelReason::Reassigned });
    let trim = log.iter().position(|c| matches!(c, Command::Trim { client: 1, .. }));
    assert!(cancel.is_some(), "A's queued run was never handed over: {log:?}");
    assert!(trim.is_none_or(|t| t > cancel.unwrap()), "a running run was cut before the queued one moved: {log:?}");
    assert!(assigns(&log).iter().any(|x| x.0 == 2 && x.2 == queued.2), "B did not start where A's queued run did: {log:?}");
}

#[test]
fn an_idle_client_takes_the_second_half_of_the_largest_running_run() {
    let b = t0();
    let mut cfg = Config::new(16);
    cfg.runs_per_client = 1;
    let mut s = Scheduler::new(cfg, &[]);
    assert_eq!(assigns(&s.step(b, Event::Joined { client: 1, name: "A".into() })), vec![(1, 1, 0, 8)]);
    // A renders frame 0, then stalls on frame 1 (its heartbeat says so).
    verify(&mut s, b, 1, 1, 0..1);
    s.step(b, Event::Heartbeat { client: 1, paused: false, frame: Some(1), frame_ms: Some(10) });
    let out = s.step(b, Event::Joined { client: 2, name: "B".into() });
    let log = drain(&mut s, b, 2, out);
    // When B has nothing left, it takes the second half of what A has not started: frames 5..8.
    assert!(log.contains(&Command::Trim { client: 1, run: 1, from: 5 }), "{log:?}");
    assert!(assigns(&log).iter().any(|x| x.0 == 2 && x.2 == 5), "B did not start at the cut: {log:?}");
    // A's copy of a stolen frame, rendered before it heard, still counts once: first copy wins.
    let late = s.step(b, Event::FrameVerified { client: 1, run: 1, index: 6, render_ms: 10 });
    assert!(late.contains(&Command::Discard { index: 6 }) || late.contains(&Command::Accept { index: 6 }));
    // Too little left to be worth a cut: nothing is stolen.
    let mut cfg = Config::new(4);
    cfg.runs_per_client = 1;
    let mut s = Scheduler::new(cfg, &[]);
    s.step(b, Event::Joined { client: 1, name: "A".into() });
    let out = s.step(b, Event::Joined { client: 2, name: "B".into() });
    let log = drain(&mut s, b, 2, out);
    assert!(!log.iter().any(|c| matches!(c, Command::Trim { .. })), "{log:?}");
}

#[test]
fn a_held_shot_stays_with_one_gpu_class_while_one_is_there() {
    let b = t0();
    let mut cfg = Config::new(20);
    cfg.first_run = 4;
    cfg.runs_per_client = 1;
    cfg.holds = vec![(0, 20)];
    let mut s = Scheduler::new(cfg, &[]);
    s.set_class(1, Some("A".into()));
    assert_eq!(assigns(&s.step(b, Event::Joined { client: 1, name: "A1".into() })), vec![(1, 1, 0, 4)]);
    s.set_class(2, Some("B".into()));
    let out = s.step(b, Event::Joined { client: 2, name: "B1".into() });
    assert!(assigns(&out).is_empty() && !out.iter().any(|c| matches!(c, Command::Trim { .. } | Command::Cancel { .. })), "class B took part of A's held shot: {out:?}");
    // Another class-A machine may share it.
    s.set_class(3, Some("A".into()));
    assert!(!assigns(&s.step(b, Event::Joined { client: 3, name: "A2".into() })).is_empty());
    // With no class-A machine left, class B renders the rest rather than leave it undone.
    s.step(b, Event::Left { client: 1, why: LeaveKind::Left });
    let out = s.step(b, Event::Left { client: 3, why: LeaveKind::Left });
    assert!(!assigns(&out).is_empty(), "the job stalled waiting for a class that left: {out:?}");
}
