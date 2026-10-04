//! Who renders which frames (design §5): a pure state machine. Events in, commands out, the clock
//! passed in — no I/O, no threads — so every rule below is a unit test, and "every frame exactly
//! once" is a property test over random joins, departures, failures and bad frames.
//!
//! - **Runs, not frames.** A client gets contiguous runs of frames, so the render process's
//!   reference pipelining, L-system tables and dissolves work as in a single render. The first run
//!   is short; later ones aim at `run_target` of work at the client's measured pace, and shrink
//!   toward the end so the last frames do not wait on one slow machine.
//! - **Placement.** A client continues where its last run ended. Otherwise it takes the largest
//!   pending gap: at its start when nobody is working toward it, at its middle when somebody is.
//! - **Two runs per client**: the next is assigned while the current renders, so a client never
//!   idles on the round trip.
//! - **Unreachable** (no heartbeat for `heartbeat_timeout`): its unfinished frames go back to the
//!   queue at once.
//! - **Stall** (the client's current run has finished no frame for the stall timeout, which is
//!   `max(stall_floor, stall_factor × median frame time)`): cancelled and re-queued. A fixed
//!   per-frame deadline is optional, because a deep frame can honestly take an hour.
//! - **Bad frames** (failed verification): re-queued away from the sender, and a strike against
//!   it. At `strikes_to_remove` the client is removed — by NAME, so reconnecting does not undo it;
//!   only the user re-admits.
//! - **Late results**: the first verified copy of a frame wins; later ones are discarded.

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};

pub type ClientId = u32;
pub type RunId = u64;

pub use crate::proto::CancelReason;

#[derive(Clone, Debug)]
pub struct Config {
    pub frames: u64,
    /// Frames in a client's first runs, before its pace is known.
    pub first_run: u64,
    /// Work a later run aims at, at the client's measured pace.
    pub run_target: Duration,
    pub min_run: u64,
    pub max_run: u64,
    pub runs_per_client: usize,
    pub heartbeat_timeout: Duration,
    pub stall_floor: Duration,
    pub stall_factor: f64,
    /// Optional hard deadline per frame.
    pub deadline: Option<Duration>,
    pub strikes_to_remove: u32,
    /// A frame that fails (or misses the deadline) on this many different clients is given up.
    pub max_failures_per_frame: u32,
    /// Two device losses or crashes within this window park a client as unstable.
    pub crash_window: Duration,
}

impl Config {
    /// The design's defaults for a job of `frames` frames.
    pub fn new(frames: u64) -> Self {
        Self {
            frames,
            first_run: 8,
            run_target: Duration::from_secs(240),
            min_run: 2,
            max_run: 64,
            runs_per_client: 2,
            heartbeat_timeout: Duration::from_secs(10),
            stall_floor: Duration::from_secs(300),
            stall_factor: 8.0,
            deadline: None,
            strikes_to_remove: 2,
            max_failures_per_frame: 2,
            crash_window: Duration::from_secs(600),
        }
    }
}

/// How a run stopped early, as the scheduler needs it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AbortKind {
    /// The client's user cancelled or paused.
    UserPaused,
    /// We cancelled it.
    Canceled,
    /// The render process lost its GPU or crashed.
    Crashed,
    /// The job asks more than the client's policy allows.
    Policy(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LeaveKind {
    /// No heartbeat for the timeout (set by the scheduler itself on a tick, or by the network).
    Unreachable,
    /// The client closed the connection or said goodbye.
    Left,
}

#[derive(Clone, Debug)]
pub enum Event {
    Joined { client: ClientId, name: String },
    Left { client: ClientId, why: LeaveKind },
    Heartbeat { client: ClientId, paused: bool, frame: Option<u64>, frame_ms: Option<u64> },
    /// A frame arrived and passed verification; the app holds it as `<frame>.part` until told.
    FrameVerified { client: ClientId, run: RunId, index: u64, render_ms: u64 },
    /// A frame arrived and FAILED verification (the app quarantined it).
    FrameBad { client: ClientId, run: RunId, index: u64, why: String },
    /// The client reported it could not produce a frame.
    FrameFailed { client: ClientId, run: RunId, index: u64, why: String },
    RunAborted { client: ClientId, run: RunId, why: AbortKind },
    /// Stop handing out work (in-flight runs finish).
    Pause,
    Resume,
    /// Cancel everything; the job ends with what it has.
    Stop,
    /// The output folder is (or is no longer) too full to take more frames.
    StorageLow(bool),
    /// The user re-admits a removed machine, by name.
    Readmit { name: String },
    /// The user removes a client.
    RemoveByUser { client: ClientId },
    Tick,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Assign { client: ClientId, run: RunId, start: u64, end: u64 },
    Cancel { client: ClientId, run: Option<RunId>, reason: CancelReason },
    /// Rename the verified `<frame>.part` into place: it is this frame's first copy.
    Accept { index: u64 },
    /// Delete it: the frame was already done.
    Discard { index: u64 },
    /// Close this client's connection, telling it why; for a removal, write a diagnostics bundle.
    Remove { client: ClientId, reason: String, strikes: u32 },
    /// Close this client's connection: it was declared unreachable. If it is in fact alive it
    /// reconnects, and starts clean as a new connection.
    Drop { client: ClientId, reason: String },
    /// A line for the event log and the panel.
    Note(String),
    /// Every frame is done or given up.
    Done { failed: Vec<u64> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum FrameState {
    Pending,
    Assigned { client: ClientId, run: RunId },
    Done,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientState {
    Active,
    /// Paused by its own user; resumes when its heartbeat says so.
    Paused,
    /// Two crashes within the crash window: no work until it reconnects.
    Unstable,
    /// The job asks more than its policy allows.
    PolicyRefused,
    Removed,
    Gone,
}

struct Client {
    name: String,
    state: ClientState,
    runs: VecDeque<RunId>,
    /// When the front run became current (its stall clock).
    front_since: Instant,
    last_heartbeat: Instant,
    cursor: Option<u64>,
    ewma_ms: Option<f64>,
    frames_done: u64,
    crashes: VecDeque<Instant>,
}

struct Run {
    client: ClientId,
    start: u64,
    end: u64,
    last_progress: Instant,
}

/// What the panel and `status.toml` show.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientView {
    pub id: ClientId,
    pub name: String,
    pub state: ClientState,
    pub runs: Vec<(RunId, u64, u64)>,
    pub frames_done: u64,
    pub strikes: u32,
    pub ewma_ms: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub frames: u64,
    pub pending: u64,
    pub assigned: u64,
    pub done: u64,
    pub failed: u64,
    pub paused: bool,
    pub storage_low: bool,
    pub finished: bool,
    pub clients: Vec<ClientView>,
}

pub struct Scheduler {
    cfg: Config,
    frames: Vec<FrameState>,
    /// Per frame: machine names it is not to go to again, and its failure count.
    excluded: HashMap<u64, Vec<String>>,
    failures: HashMap<u64, u32>,
    clients: HashMap<ClientId, Client>,
    runs: HashMap<RunId, Run>,
    next_run: RunId,
    strikes: HashMap<String, u32>,
    removed: HashSet<String>,
    paused: bool,
    stopped: bool,
    storage_low: bool,
    finished: bool,
    frame_ms: Vec<u64>,
    /// The time of the event being processed.
    now: Instant,
}

impl Scheduler {
    /// A job of `cfg.frames` frames, of which `done` are already on disk (a resume).
    pub fn new(cfg: Config, done: &[u64]) -> Self {
        let mut frames = vec![FrameState::Pending; cfg.frames as usize];
        for &d in done {
            if let Some(f) = frames.get_mut(d as usize) {
                *f = FrameState::Done;
            }
        }
        Self {
            cfg,
            frames,
            excluded: HashMap::new(),
            failures: HashMap::new(),
            clients: HashMap::new(),
            runs: HashMap::new(),
            next_run: 1,
            strikes: HashMap::new(),
            removed: HashSet::new(),
            paused: false,
            stopped: false,
            storage_low: false,
            finished: false,
            frame_ms: Vec::new(),
            now: Instant::now(),
        }
    }

    pub fn config(&self) -> &Config {
        &self.cfg
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// The client a frame is assigned to, if any.
    pub fn owner(&self, index: u64) -> Option<ClientId> {
        match self.frames.get(index as usize) {
            Some(FrameState::Assigned { client, .. }) => Some(*client),
            _ => None,
        }
    }

    /// Feed one event; get what to do.
    pub fn step(&mut self, now: Instant, ev: Event) -> Vec<Command> {
        self.now = now;
        let mut out = Vec::new();
        match ev {
            Event::Joined { client, name } => {
                if self.removed.contains(&name) {
                    out.push(Command::Remove {
                        client,
                        reason: format!("\"{name}\" was removed from this farm earlier; it rejoins only when re-admitted at the controller"),
                        strikes: self.strikes.get(&name).copied().unwrap_or(0),
                    });
                } else {
                    out.push(Command::Note(format!("{name} joined")));
                    self.clients.insert(
                        client,
                        Client {
                            name,
                            state: ClientState::Active,
                            runs: VecDeque::new(),
                            front_since: now,
                            last_heartbeat: now,
                            cursor: None,
                            ewma_ms: None,
                            frames_done: 0,
                            crashes: VecDeque::new(),
                        },
                    );
                }
            }
            Event::Left { client, why } => {
                if let Some(c) = self.clients.get(&client) {
                    let name = c.name.clone();
                    let n = self.drop_runs(client);
                    if let Some(c) = self.clients.get_mut(&client) {
                        c.state = ClientState::Gone;
                    }
                    let what = match why {
                        LeaveKind::Unreachable => "is unreachable",
                        LeaveKind::Left => "left",
                    };
                    out.push(Command::Note(format!("{name} {what}; {n} unfinished frame(s) back in the queue")));
                }
            }
            Event::Heartbeat { client, paused, frame, frame_ms } => {
                // A connection the scheduler has written off cannot come back to life by talking:
                // it is dropped, and returns as a new connection.
                if let Some(c) = self.clients.get_mut(&client).filter(|c| matches!(c.state, ClientState::Active | ClientState::Paused)) {
                    c.last_heartbeat = now;
                    if c.state == ClientState::Paused && !paused {
                        c.state = ClientState::Active;
                        out.push(Command::Note(format!("{} resumed", c.name)));
                    }
                    if let (Some(limit), Some(f), Some(ms)) = (self.cfg.deadline, frame, frame_ms) {
                        if ms as u128 > limit.as_millis() {
                            if let Some(&run) = c.runs.front() {
                                let name = c.name.clone();
                                out.push(Command::Cancel { client, run: Some(run), reason: CancelReason::Deadline });
                                self.fail_frame(f, &name, &mut out, "missed the hard deadline");
                                let n = self.drop_run(run);
                                out.push(Command::Note(format!("{name}: frame {f} passed the {}s deadline; run cancelled, {n} frame(s) re-queued", limit.as_secs())));
                            }
                        }
                    }
                }
            }
            Event::FrameVerified { client, run, index, render_ms } => {
                let Some(state) = self.frames.get(index as usize).cloned() else {
                    out.push(Command::Discard { index });
                    return out;
                };
                if state == FrameState::Done {
                    out.push(Command::Discard { index });
                    out.push(Command::Note(format!("frame {index}: a late copy arrived and was discarded")));
                } else {
                    self.frames[index as usize] = FrameState::Done;
                    out.push(Command::Accept { index });
                    self.frame_ms.push(render_ms);
                    if let Some(r) = self.runs.get_mut(&run) {
                        r.last_progress = now;
                    }
                    if let Some(c) = self.clients.get_mut(&client) {
                        c.frames_done += 1;
                        let ms = render_ms as f64;
                        c.ewma_ms = Some(c.ewma_ms.map_or(ms, |e| 0.7 * e + 0.3 * ms));
                    }
                    self.retire_finished_runs(now);
                }
            }
            Event::FrameBad { client, run, index, why } => {
                let name = self.clients.get(&client).map(|c| c.name.clone()).unwrap_or_default();
                if self.frames.get(index as usize) == Some(&FrameState::Assigned { client, run }) {
                    self.frames[index as usize] = FrameState::Pending;
                }
                self.excluded.entry(index).or_default().push(name.clone());
                let s = self.strikes.entry(name.clone()).or_insert(0);
                *s += 1;
                let strikes = *s;
                out.push(Command::Note(format!("{name}: frame {index} failed verification ({why}) — strike {strikes}")));
                if strikes >= self.cfg.strikes_to_remove {
                    self.remove(client, &mut out, format!("{strikes} frames failed verification"));
                }
            }
            Event::FrameFailed { client, run, index, why } => {
                let name = self.clients.get(&client).map(|c| c.name.clone()).unwrap_or_default();
                if self.frames.get(index as usize) == Some(&FrameState::Assigned { client, run }) {
                    self.frames[index as usize] = FrameState::Pending;
                }
                self.fail_frame(index, &name, &mut out, &why);
            }
            Event::RunAborted { client, run, why } => {
                let n = self.drop_run(run);
                let mut parked: Option<CancelReason> = None;
                if let Some(c) = self.clients.get_mut(&client).filter(|c| matches!(c.state, ClientState::Active | ClientState::Paused)) {
                    let name = c.name.clone();
                    match why {
                        AbortKind::UserPaused => {
                            c.state = ClientState::Paused;
                            parked = Some(CancelReason::Paused);
                            out.push(Command::Note(format!("{name} was paused by its user; {n} frame(s) re-queued")));
                        }
                        AbortKind::Canceled => {}
                        AbortKind::Crashed => {
                            c.crashes.push_back(now);
                            while c.crashes.front().is_some_and(|&t| now.duration_since(t) > self.cfg.crash_window) {
                                c.crashes.pop_front();
                            }
                            if c.crashes.len() >= 2 {
                                c.state = ClientState::Unstable;
                                parked = Some(CancelReason::Reassigned);
                                out.push(Command::Note(format!("{name}: two render crashes within {} min — parked as unstable", self.cfg.crash_window.as_secs() / 60)));
                            } else {
                                out.push(Command::Note(format!("{name}: render process crashed; {n} frame(s) re-queued")));
                            }
                        }
                        AbortKind::Policy(what) => {
                            c.state = ClientState::PolicyRefused;
                            parked = Some(CancelReason::Reassigned);
                            out.push(Command::Note(format!("{name} refused the job: {what}")));
                        }
                    }
                }
                // ⚠A client that stops taking work gives back EVERY run it holds, not just the one
                // that ended — its prefetched next run would otherwise sit assigned forever (the stall
                // rule watches only active clients), and the job would never finish.
                if let Some(reason) = parked {
                    if self.clients.get(&client).is_some_and(|c| !c.runs.is_empty()) {
                        out.push(Command::Cancel { client, run: None, reason });
                        self.drop_runs(client);
                    }
                }
            }
            Event::Pause => {
                self.paused = true;
                out.push(Command::Note("job paused: no new work is handed out".into()));
            }
            Event::Resume => {
                self.paused = false;
                out.push(Command::Note("job resumed".into()));
            }
            Event::Stop => {
                self.stopped = true;
                let ids: Vec<ClientId> = self.clients.keys().copied().collect();
                for id in ids {
                    if self.clients.get(&id).is_some_and(|c| !c.runs.is_empty()) {
                        out.push(Command::Cancel { client: id, run: None, reason: CancelReason::Stopped });
                        self.drop_runs(id);
                    }
                }
                out.push(Command::Note("job stopped".into()));
            }
            Event::StorageLow(low) => {
                if low != self.storage_low {
                    self.storage_low = low;
                    out.push(Command::Note(if low {
                        "waiting for space: the output folder is nearly full, no new work is handed out".into()
                    } else {
                        "space available again: handing out work".into()
                    }));
                }
            }
            Event::Readmit { name } => {
                if self.removed.remove(&name) {
                    self.strikes.remove(&name);
                    out.push(Command::Note(format!("{name} re-admitted")));
                }
            }
            Event::RemoveByUser { client } => self.remove(client, &mut out, "removed by the controller's user".into()),
            Event::Tick => self.tick(now, &mut out),
        }
        if !self.finished {
            self.assign(now, &mut out);
            self.check_done(&mut out);
        }
        out
    }

    fn tick(&mut self, now: Instant, out: &mut Vec<Command>) {
        // Unreachable clients.
        let lost: Vec<ClientId> = self
            .clients
            .iter()
            .filter(|(_, c)| matches!(c.state, ClientState::Active | ClientState::Paused) && now.duration_since(c.last_heartbeat) > self.cfg.heartbeat_timeout)
            .map(|(&id, _)| id)
            .collect();
        for id in lost {
            out.extend(self.step_inner_left(id));
        }
        // Stalled runs: only a client's FRONT run is rendering.
        let stall = self.stall_timeout();
        let stalled: Vec<(ClientId, RunId)> = self
            .clients
            .iter()
            .filter(|(_, c)| c.state == ClientState::Active)
            .filter_map(|(&id, c)| {
                let run = *c.runs.front()?;
                let r = self.runs.get(&run)?;
                let since = r.last_progress.max(c.front_since);
                (now.duration_since(since) > stall).then_some((id, run))
            })
            .collect();
        for (id, run) in stalled {
            let name = self.clients[&id].name.clone();
            let n = self.drop_run(run);
            out.push(Command::Cancel { client: id, run: Some(run), reason: CancelReason::Stalled });
            out.push(Command::Note(format!("{name}: no frame finished in {}s — run cancelled, {n} frame(s) re-queued", stall.as_secs())));
        }
    }

    fn step_inner_left(&mut self, id: ClientId) -> Vec<Command> {
        let name = self.clients.get(&id).map(|c| c.name.clone()).unwrap_or_default();
        let n = self.drop_runs(id);
        if let Some(c) = self.clients.get_mut(&id) {
            c.state = ClientState::Gone;
        }
        let secs = self.cfg.heartbeat_timeout.as_secs();
        vec![
            Command::Drop { client: id, reason: format!("no heartbeat for {secs}s") },
            Command::Note(format!("{name} is unreachable (no heartbeat for {secs}s); {n} unfinished frame(s) back in the queue")),
        ]
    }

    /// The stall timeout now: `max(floor, factor × median frame time seen on this job)`.
    pub fn stall_timeout(&self) -> Duration {
        if self.frame_ms.is_empty() {
            return self.cfg.stall_floor;
        }
        let mut v = self.frame_ms.clone();
        let mid = v.len() / 2;
        let (_, m, _) = v.select_nth_unstable(mid);
        self.cfg.stall_floor.max(Duration::from_millis((*m as f64 * self.cfg.stall_factor) as u64))
    }

    fn fail_frame(&mut self, index: u64, name: &str, out: &mut Vec<Command>, why: &str) {
        self.excluded.entry(index).or_default().push(name.to_string());
        let f = self.failures.entry(index).or_insert(0);
        *f += 1;
        if *f >= self.cfg.max_failures_per_frame {
            if let Some(s) = self.frames.get_mut(index as usize) {
                if *s != FrameState::Done {
                    *s = FrameState::Failed;
                }
            }
            out.push(Command::Note(format!("frame {index} failed on {f} machines ({why}) — given up; --resume renders it later", f = *f)));
        } else {
            out.push(Command::Note(format!("{name}: frame {index} failed ({why}) — re-queued for another machine")));
        }
    }

    fn remove(&mut self, client: ClientId, out: &mut Vec<Command>, reason: String) {
        let Some(c) = self.clients.get(&client) else { return };
        let name = c.name.clone();
        if !c.runs.is_empty() {
            out.push(Command::Cancel { client, run: None, reason: CancelReason::Removed });
        }
        let n = self.drop_runs(client);
        if let Some(c) = self.clients.get_mut(&client) {
            c.state = ClientState::Removed;
        }
        self.removed.insert(name.clone());
        let strikes = self.strikes.get(&name).copied().unwrap_or(0);
        out.push(Command::Remove { client, reason: reason.clone(), strikes });
        out.push(Command::Note(format!("{name} removed from the farm: {reason}; {n} frame(s) re-queued")));
    }

    /// Re-queue a run's unfinished frames and forget it. Returns how many were re-queued.
    fn drop_run(&mut self, run: RunId) -> u64 {
        let Some(r) = self.runs.remove(&run) else { return 0 };
        let mut n = 0;
        for i in r.start..r.end {
            if self.frames[i as usize] == (FrameState::Assigned { client: r.client, run }) {
                self.frames[i as usize] = FrameState::Pending;
                n += 1;
            }
        }
        let now = self.now;
        if let Some(c) = self.clients.get_mut(&r.client) {
            let was_front = c.runs.front() == Some(&run);
            c.runs.retain(|&x| x != run);
            c.cursor = None;
            if was_front {
                // Its next run is current from now: its stall clock starts here.
                c.front_since = now;
            }
        }
        n
    }

    fn drop_runs(&mut self, client: ClientId) -> u64 {
        let runs: Vec<RunId> = self.clients.get(&client).map(|c| c.runs.iter().copied().collect()).unwrap_or_default();
        runs.into_iter().map(|r| self.drop_run(r)).sum()
    }

    /// A run whose frames are all done (or no longer its own) is retired; the client's next run
    /// becomes current and its stall clock starts.
    fn retire_finished_runs(&mut self, now: Instant) {
        let finished: Vec<RunId> = self
            .runs
            .iter()
            .filter(|(&id, r)| (r.start..r.end).all(|i| self.frames[i as usize] != FrameState::Assigned { client: r.client, run: id }))
            .map(|(&id, _)| id)
            .collect();
        for id in finished {
            if let Some(r) = self.runs.remove(&id) {
                if let Some(c) = self.clients.get_mut(&r.client) {
                    let was_front = c.runs.front() == Some(&id);
                    c.runs.retain(|&x| x != id);
                    if was_front {
                        c.front_since = now;
                    }
                }
            }
        }
    }

    fn assignable(&self, i: u64, name: &str) -> bool {
        self.frames.get(i as usize) == Some(&FrameState::Pending) && !self.excluded.get(&i).is_some_and(|v| v.iter().any(|n| n == name))
    }

    fn assign(&mut self, now: Instant, out: &mut Vec<Command>) {
        if self.paused || self.stopped || self.storage_low {
            return;
        }
        let mut ids: Vec<ClientId> = self.clients.iter().filter(|(_, c)| c.state == ClientState::Active).map(|(&id, _)| id).collect();
        ids.sort_unstable();
        let active = ids.len().max(1) as u64;
        loop {
            let mut progressed = false;
            for &id in &ids {
                let c = &self.clients[&id];
                if c.runs.len() >= self.cfg.runs_per_client {
                    continue;
                }
                let name = c.name.clone();
                let Some(start) = self.pick_start(id, &name) else { continue };
                let pending = self.frames.iter().filter(|f| **f == FrameState::Pending).count() as u64;
                let c = &self.clients[&id];
                let mut len = match c.ewma_ms {
                    None => self.cfg.first_run,
                    Some(ms) => ((self.cfg.run_target.as_millis() as f64 / ms.max(1.0)) as u64).clamp(self.cfg.min_run, self.cfg.max_run),
                };
                // Toward the end, smaller runs so the last frames spread across machines.
                len = len.min((pending / (2 * active)).max(1));
                let mut end = start;
                while end < self.cfg.frames && end - start < len && self.assignable(end, &name) {
                    end += 1;
                }
                if end == start {
                    continue;
                }
                let run = self.next_run;
                self.next_run += 1;
                for i in start..end {
                    self.frames[i as usize] = FrameState::Assigned { client: id, run };
                }
                self.runs.insert(run, Run { client: id, start, end, last_progress: now });
                let c = self.clients.get_mut(&id).unwrap();
                if c.runs.is_empty() {
                    c.front_since = now;
                }
                c.runs.push_back(run);
                c.cursor = Some(end);
                out.push(Command::Assign { client: id, run, start, end });
                progressed = true;
            }
            if !progressed {
                break;
            }
        }
    }

    /// Where a client's next run starts: where its last one ended, else the largest pending gap it
    /// may take — at the gap's start when nobody is working toward it, at its middle when somebody is.
    fn pick_start(&self, id: ClientId, name: &str) -> Option<u64> {
        if let Some(c) = self.clients[&id].cursor {
            if self.assignable(c, name) {
                return Some(c);
            }
        }
        let mut best: Option<(u64, u64)> = None; // (start, len)
        let mut i = 0;
        while i < self.cfg.frames {
            if !self.assignable(i, name) {
                i += 1;
                continue;
            }
            let s = i;
            while i < self.cfg.frames && self.assignable(i, name) {
                i += 1;
            }
            if best.is_none_or(|(_, l)| i - s > l) {
                best = Some((s, i - s));
            }
        }
        let (s, l) = best?;
        let approached = s > 0 && matches!(self.frames[(s - 1) as usize], FrameState::Assigned { .. });
        Some(if approached && l >= 2 { s + l / 2 } else { s })
    }

    fn check_done(&mut self, out: &mut Vec<Command>) {
        let open = self.frames.iter().any(|f| matches!(f, FrameState::Pending | FrameState::Assigned { .. }));
        if !open || (self.stopped && self.runs.is_empty()) {
            self.finished = true;
            let failed: Vec<u64> = self.frames.iter().enumerate().filter(|(_, f)| **f != FrameState::Done).map(|(i, _)| i as u64).collect();
            out.push(Command::Done { failed });
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        let count = |p: fn(&FrameState) -> bool| self.frames.iter().filter(|f| p(f)).count() as u64;
        let mut clients: Vec<ClientView> = self
            .clients
            .iter()
            .map(|(&id, c)| ClientView {
                id,
                name: c.name.clone(),
                state: c.state,
                runs: c.runs.iter().filter_map(|r| self.runs.get(r).map(|x| (*r, x.start, x.end))).collect(),
                frames_done: c.frames_done,
                strikes: self.strikes.get(&c.name).copied().unwrap_or(0),
                ewma_ms: c.ewma_ms,
            })
            .collect();
        clients.sort_by_key(|c| c.id);
        Snapshot {
            frames: self.cfg.frames,
            pending: count(|f| *f == FrameState::Pending),
            assigned: count(|f| matches!(f, FrameState::Assigned { .. })),
            done: count(|f| *f == FrameState::Done),
            failed: count(|f| *f == FrameState::Failed),
            paused: self.paused,
            storage_low: self.storage_low,
            finished: self.finished,
            clients,
        }
    }
}

#[cfg(test)]
mod tests;
