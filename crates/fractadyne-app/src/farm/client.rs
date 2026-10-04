//! `--render-client HOST:PORT`: a machine that renders frames for a controller (design §10).
//!
//! It dials the controller (it opens no port), proves it holds the farm key, pins the controller's
//! identity, runs the handshake self-check, and then renders the runs it is assigned — each as a
//! `--render-tour … --farm-child` child in the job's own configuration folder, built from the job's
//! settings, never this machine's session. Every frame the child reports is re-read, checked against
//! the digest the child printed, and streamed to the controller, which verifies it again.
//!
//! Controls: the app's Render client window (through `--ui-status`, `status.rs`), or headless,
//! create `<config>/farm/PAUSE` to finish the current frame and stop taking work (delete it to
//! resume); create `CANCEL` to stop at once (it also pauses). A deliberate `Bye` from the controller
//! — for instance a removal — ends the client; a connection that merely drops is redialled with
//! back-off.

use super::status::{self, ClientCommand, ClientPhase, ClientStatus};
use super::*;
use fractadyne_farm::channel::{self, Pin, PinStore};
use fractadyne_farm::proto::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

/// Test instrument (`--farmtest`): corrupt the first N frames this client sends, AFTER computing
/// the digest it announces — so the controller's verification must catch them. Listed in
/// `tunables::INSTRUMENTS`, so a client running with it reports itself as not stock.
pub(crate) const CORRUPT_INSTRUMENT: &str = "FRACTADYNE_FARM_CORRUPT_FRAMES";

struct Cfg {
    addr: String,
    key: FarmKey,
    id: Identity,
    name: String,
    policy: Policy,
    allow_dirty: bool,
    /// `--one-job`: exit once the controller closes a job — how a test machine's field agent runs
    /// a client, so its run ends with the job instead of at its timeout.
    one_job: bool,
    /// `--share-root`: this machine's path to the shared drive, for share mode.
    share_root: Option<PathBuf>,
    work: PathBuf,
    control: PathBuf,
}

enum Out {
    Msg(Msg),
    Blob(u64, Vec<u8>),
}

enum Ev {
    Net(Incoming),
    NetDown(String),
    Child { run: u64, line: ChildLine },
    ChildClosed { run: u64 },
    Tick,
}

enum Ended {
    Exit(i32),
    Retry { connected: bool, why: String },
}

/// The `--ui-status` side: the status the app's window reads, and the commands it sends. Inert
/// without the flag. Pause and cancel from the window persist across reconnects, like the files.
struct Ui {
    on: bool,
    st: Mutex<ClientStatus>,
    cmds: Mutex<Option<mpsc::Receiver<Option<String>>>>,
    paused: AtomicBool,
    cancel: AtomicBool,
    leave: AtomicBool,
}

impl Ui {
    fn new(on: bool) -> Arc<Ui> {
        let ui = Arc::new(Ui {
            on,
            st: Mutex::new(ClientStatus::default()),
            cmds: Mutex::new(on.then(status::commands)),
            paused: AtomicBool::new(false),
            cancel: AtomicBool::new(false),
            leave: AtomicBool::new(false),
        });
        if on {
            let weak = Arc::downgrade(&ui);
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(1));
                let Some(ui) = weak.upgrade() else { return };
                ui.print();
            });
        }
        ui
    }

    /// Change the status; a change of phase is printed at once.
    fn update(&self, f: impl FnOnce(&mut ClientStatus)) {
        let mut s = self.st.lock().unwrap_or_else(|e| e.into_inner());
        let before = s.phase;
        f(&mut s);
        if self.on && s.phase != before {
            println!("{}", status::line(&*s));
        }
    }

    fn print(&self) {
        if self.on {
            println!("{}", status::line(&*self.st.lock().unwrap_or_else(|e| e.into_inner())));
        }
    }

    /// Apply the commands that arrived; the end of stdin (the window is gone) reads as Leave.
    fn poll(&self) {
        let guard = self.cmds.lock().unwrap_or_else(|e| e.into_inner());
        let Some(rx) = guard.as_ref() else { return };
        for c in rx.try_iter() {
            match c.as_deref().map(ClientCommand::parse) {
                Some(Some(ClientCommand::Pause)) => self.paused.store(true, Ordering::Relaxed),
                Some(Some(ClientCommand::Resume)) => self.paused.store(false, Ordering::Relaxed),
                Some(Some(ClientCommand::CancelFrame)) => {
                    self.cancel.store(true, Ordering::Relaxed);
                    self.paused.store(true, Ordering::Relaxed);
                }
                Some(Some(ClientCommand::Leave)) | None => self.leave.store(true, Ordering::Relaxed),
                Some(None) => eprintln!("fractadyne: render client: unknown command {:?}", c.unwrap_or_default()),
            }
        }
    }

    fn leaving(&self) -> bool {
        self.leave.load(Ordering::Relaxed)
    }

    /// The process is about to exit with `code`.
    fn end(&self, code: i32, why: &str) {
        self.update(|s| {
            s.phase = ClientPhase::Ended;
            s.exit_code = Some(code);
            if !why.is_empty() {
                s.detail = why.to_string();
            }
        });
        self.print();
    }
}

pub(crate) fn run(args: &[String]) -> i32 {
    let fail = |e: String| {
        eprintln!("fractadyne: render client: {e}");
        2
    };
    let Some(addr) = value(args, "--render-client").map(str::to_string) else {
        return fail("--render-client needs the controller's address, e.g. --render-client 192.168.1.20:46733".into());
    };
    let key = match load_key(args, false) {
        Ok((k, _, _)) => k,
        Err(e) => return fail(e),
    };
    let name = machine_name(args);
    if let Err(e) = fractadyne_farm::names::check_display_name(&name) {
        return fail(e);
    }
    let (max_width, max_height) = match value(args, "--max-size").map(crate::parse_size) {
        Some((Some(w), Some(h))) => (w, h),
        Some(_) => return fail("--max-size expects WIDTHxHEIGHT".into()),
        None => (16384, 16384),
    };
    let policy = Policy {
        max_width,
        max_height,
        max_ss: number(args, "--max-ss").unwrap_or(8u32).clamp(1, 8),
        max_iter: number(args, "--max-iter").unwrap_or(10_000_000u32).max(1),
    };
    let allow_dirty = args.iter().any(|a| a == "--farm-allow-dirty");
    let (_, git) = build_identity();
    if is_dirty(git) && !allow_dirty {
        return fail(format!("this build ({git}) has uncommitted changes, and a farm refuses one — start with --farm-allow-dirty only to develop the farm itself"));
    }
    let dir = match farm_dir() {
        Ok(d) => d,
        Err(e) => return fail(e),
    };
    let id = match identity() {
        Ok(i) => i,
        Err(e) => return fail(e),
    };
    let mut pins = match PinStore::load(&dir.join("known-controllers.toml")) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    let one_job = args.iter().any(|a| a == "--one-job");
    let share_root = value(args, "--share-root").map(PathBuf::from);
    let cfg = Cfg { addr, key, name, policy, allow_dirty, one_job, share_root, work: dir.join("jobs"), control: dir.clone(), id };
    let ui = Ui::new(args.iter().any(|a| a == status::FLAG));
    ui.update(|s| {
        s.name = cfg.name.clone();
        s.identity = cfg.id.fingerprint();
        s.controller = cfg.addr.clone();
        s.detail = format!("Connecting to {}…", cfg.addr);
    });
    println!("Render client \"{}\" (identity {}) — controller {}", cfg.name, cfg.id.fingerprint(), cfg.addr);
    println!("  pause: create {}  ·  cancel the frame in progress: create {}", cfg.control.join("PAUSE").display(), cfg.control.join("CANCEL").display());
    let mut backoff = Duration::from_secs(2);
    loop {
        match connect_once(&cfg, &mut pins, &ui) {
            Ended::Exit(code) => {
                ui.end(code, "");
                return code;
            }
            Ended::Retry { connected, why } => {
                if connected {
                    backoff = Duration::from_secs(2);
                }
                println!("{why} — retrying in {}s", backoff.as_secs());
                crate::diag::log_line("farm", &format!("client: {why}"));
                ui.update(|s| {
                    s.phase = ClientPhase::Retrying;
                    s.detail = why.clone();
                    s.retry_in_s = Some(backoff.as_secs());
                    s.run = None;
                    s.frame = None;
                });
                // In short steps, so a Leave from the window ends the wait.
                let until = Instant::now() + backoff;
                while Instant::now() < until {
                    ui.poll();
                    if ui.leaving() {
                        ui.end(0, "Disconnected");
                        return 0;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                ui.update(|s| {
                    s.phase = ClientPhase::Connecting;
                    s.retry_in_s = None;
                    s.detail = format!("Connecting to {}…", cfg.addr);
                });
                backoff = (backoff * 2).min(Duration::from_secs(60));
            }
        }
    }
}

fn connect_once(cfg: &Cfg, pins: &mut PinStore, ui: &Arc<Ui>) -> Ended {
    use std::net::ToSocketAddrs;
    let retry = |why: String| Ended::Retry { connected: false, why };
    let Some(sa) = cfg.addr.to_socket_addrs().ok().and_then(|mut a| a.next()) else {
        return retry(format!("cannot resolve {}", cfg.addr));
    };
    let sock = match std::net::TcpStream::connect_timeout(&sa, Duration::from_secs(5)) {
        Ok(s) => s,
        Err(e) => return retry(format!("cannot reach the controller at {}: {e}", cfg.addr)),
    };
    let sess = match channel::initiate(sock, &cfg.key, &cfg.id) {
        Ok(s) => s,
        Err(e) => return retry(e),
    };
    // ⛔The pin is judged only once the controller has PROVEN it holds the farm key — by sending
    // the first authenticated message (its verdict, below). With `psk3` the client learns the
    // controller's static key in handshake message 2, before any such proof: pinning (or refusing)
    // here let a party with no key at all get its own identity pinned at the controller's address,
    // after which the real controller was refused as "changed identity". Found by a wrong-key test.
    let fp = fractadyne_farm::key::fingerprint(&sess.remote_static);
    let pin = pins.check(&cfg.addr, &fp);
    let (mut reader, writer) = match sess.split(Some(Duration::from_secs(30)), Duration::from_secs(30)) {
        Ok(p) => p,
        Err(e) => return retry(e),
    };
    let (ev_tx, ev_rx) = mpsc::channel::<Ev>();
    let (out_tx, out_rx) = mpsc::channel::<Out>();
    {
        let tx = ev_tx.clone();
        std::thread::spawn(move || loop {
            match reader.recv() {
                Ok(m) => {
                    if tx.send(Ev::Net(m)).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    let _ = tx.send(Ev::NetDown(e.to_string()));
                    break;
                }
            }
        });
    }
    {
        let tx = ev_tx.clone();
        std::thread::spawn(move || {
            let mut w = writer;
            for o in out_rx {
                let r = match o {
                    Out::Msg(m) => w.send(&m),
                    Out::Blob(id, b) => w.send_blob(id, &b),
                };
                if let Err(e) = r {
                    let _ = tx.send(Ev::NetDown(e));
                    break;
                }
            }
            w.shutdown();
        });
    }
    {
        let tx = ev_tx.clone();
        std::thread::spawn(move || {
            while tx.send(Ev::Tick).is_ok() {
                std::thread::sleep(Duration::from_millis(500));
            }
        });
    }
    let (ver, git) = build_identity();
    let hello = Hello {
        protocol: fractadyne_farm::PROTOCOL_VERSION,
        app_version: ver.into(),
        git: git.into(),
        allow_dirty: cfg.allow_dirty,
        name: cfg.name.clone(),
        tunables: crate::tunables::status_line(),
        policy: cfg.policy.clone(),
        clock_unix_ms: unix_ms(),
    };
    let _ = out_tx.send(Out::Msg(Msg::Hello(hello)));
    // The verdict.
    let deadline = Instant::now() + Duration::from_secs(20);
    let (link_bytes, controller_gpu) = loop {
        let wait = deadline.saturating_duration_since(Instant::now()).max(Duration::from_millis(1));
        match ev_rx.recv_timeout(wait) {
            Ok(Ev::Net(Incoming::Control(Msg::HelloAck(a)))) => match a.verdict {
                // (Authenticated: only a holder of the farm key can produce this message.)
                _ if matches!(pin, Pin::Changed { .. }) => {
                    let Pin::Changed { pinned } = &pin else { unreachable!() };
                    let why = format!(
                        "The controller at {} changed identity (pinned {pinned}, now {fp}). If that is expected — it was reinstalled — delete its line from {} and connect again.",
                        cfg.addr,
                        cfg.control.join("known-controllers.toml").display()
                    );
                    eprintln!("fractadyne: {why}");
                    ui.end(2, &why);
                    return Ended::Exit(2);
                }
                Verdict::Admitted => {
                    if pin == Pin::New {
                        match pins.pin(&cfg.addr, &fp) {
                            Ok(()) => println!("Pinned the controller's identity {fp}"),
                            Err(e) => eprintln!("fractadyne: could not save the controller's identity: {e}"),
                        }
                    }
                    println!("Connected to \"{}\" ({fp}) — running the self-check", a.name);
                    ui.update(|s| {
                        s.phase = ClientPhase::Checking;
                        s.controller_name = a.name.clone();
                        s.controller_fingerprint = fp.clone();
                        s.detail = format!("Connected to \"{}\" — running the self-check", a.name);
                    });
                    break (a.link_sample_bytes, a.gpu.clone());
                }
                Verdict::WaitingForApproval => {
                    println!("Connected to \"{}\" — waiting for its user to approve this machine", a.name);
                    ui.update(|s| s.detail = format!("Connected to \"{}\" — waiting for its user to approve this machine", a.name));
                }
                Verdict::Refused(r) => {
                    eprintln!("fractadyne: the controller refused this machine: {r}");
                    ui.end(3, &format!("Refused by the controller: {r}"));
                    return Ended::Exit(3);
                }
            },
            Ok(Ev::Net(Incoming::Control(Msg::Bye(b)))) => {
                eprintln!("fractadyne: the controller closed the connection: {}", b.reason);
                ui.end(3, &format!("The controller closed the connection: {}", b.reason));
                return Ended::Exit(3);
            }
            Ok(Ev::NetDown(e)) => return retry(format!("the controller closed the connection during the handshake ({e}) — most often a different farm key")),
            Ok(Ev::Tick) | Ok(Ev::Net(Incoming::Control(Msg::Keepalive))) => {}
            Ok(_) => return retry("the controller sent something other than its verdict".into()),
            Err(_) if Instant::now() < deadline => {}
            Err(_) => return retry("the controller gave no verdict in time".into()),
        }
    };
    let (check, sample, probe, gpu_cap, handshake) = self_check(cfg, link_bytes);
    // Say so when this GPU differs from the controller's: its frames will not match the controller's
    // pixel for pixel (design §9).
    let gpu_note = match (&controller_gpu, &check.gpu) {
        (Some(c), Some(m)) => gpu_difference(c, m).map(|d| format!("{} from the controller's — this machine: {}; the controller: {}", d.words(), gpu_text(m), gpu_text(c))),
        _ => None,
    };
    if let Some(n) = &gpu_note {
        println!("⚠ This machine has {n}. Its frames will differ slightly from the controller's.");
    }
    let (mine, theirs) = (check.gpu.as_ref().map(gpu_text), controller_gpu.as_ref().map(gpu_text));
    ui.update(|s| {
        s.gpu = mine;
        s.controller_gpu = theirs;
        s.gpu_note = gpu_note.clone();
    });
    let failed_hard = check.items.iter().find(|i| i.hard && !i.ok).map(|i| format!("{}: {}", i.name, i.detail));
    let _ = out_tx.send(Out::Msg(Msg::SelfCheck(check)));
    if let Some(blob) = sample {
        let _ = out_tx.send(Out::Blob(1, blob));
    }
    if let Some(blob) = probe {
        let _ = out_tx.send(Out::Blob(2, blob));
    }
    if let Some(f) = &failed_hard {
        eprintln!("fractadyne: the self-check failed: {f}");
    }
    ui.update(|s| {
        s.self_check = handshake.lines().map(str::to_string).collect();
        s.phase = ClientPhase::Idle;
        s.detail = match &failed_hard {
            Some(f) => format!("The self-check failed — {f}"),
            None => format!("Connected to \"{}\" — idle", s.controller_name),
        };
    });
    let mut s = Session {
        cfg,
        ui: ui.clone(),
        out: out_tx,
        ev_tx,
        next_blob: 2,
        jobs: HashMap::new(),
        pending_bundle: None,
        queue: VecDeque::new(),
        running: None,
        child_tail: Arc::new(Mutex::new(VecDeque::new())),
        heartbeats: VecDeque::new(),
        handshake,
        last_heartbeat: Instant::now() - Duration::from_secs(10),
        paused: false,
        frames_done: 0,
        ms_total: 0,
        gpu_cap,
        corrupt_left: crate::tunables::instrument(CORRUPT_INSTRUMENT),
    };
    let ended = s.run(&ev_rx);
    s.stop_child(None);
    ended
}

/// The handshake self-check (design §9.1): the probe render (device, render path, child launch,
/// adapter, orbit cap — and the image the controller compares), free space, and the link sample the
/// controller asked for. Returns the check, the link sample's bytes, the probe's PNG, the orbit cap
/// and the record kept for diagnostics.
#[allow(clippy::type_complexity)]
fn self_check(cfg: &Cfg, link_bytes: u64) -> (SelfCheck, Option<Vec<u8>>, Option<Vec<u8>>, Option<u64>, String) {
    let mut items = Vec::new();
    let t0 = Instant::now();
    let result = render_probe(&cfg.work.join("self-check"));
    let (gpu, cap) = match &result {
        Ok((_, stderr)) => {
            let f = gpu_facts(stderr);
            items.push(CheckItem {
                name: "render".into(),
                ok: true,
                hard: true,
                detail: format!(
                    "test frame in {} ms on {}{}",
                    t0.elapsed().as_millis(),
                    f.adapter.as_deref().unwrap_or("an unnamed adapter"),
                    f.driver.as_ref().map_or(String::new(), |d| format!(", driver {d}"))
                ),
            });
            (f.adapter.map(|a| GpuInfo { adapter: a, driver: f.driver.unwrap_or_default(), orbit_len_cap: f.orbit_len_cap.unwrap_or(0) }), f.orbit_len_cap)
        }
        Err(e) => {
            items.push(CheckItem { name: "render".into(), ok: false, hard: true, detail: tail_of(e, 900) });
            (None, None)
        }
    };
    let free = crate::sysinfo::free_disk_bytes(&cfg.work);
    items.push(CheckItem {
        name: "storage".into(),
        ok: free.is_none_or(|f| f > 1 << 30),
        hard: false,
        detail: free.map_or("free space unknown".into(), |f| format!("{:.1} GB free for frames in progress", f as f64 / 1e9)),
    });
    let sample = (link_bytes > 0).then(|| {
        let mut b = vec![0u8; link_bytes.min(MAX_LINK_SAMPLE_BYTES) as usize];
        let _ = ring::rand::SecureRandom::fill(&ring::rand::SystemRandom::new(), &mut b);
        b
    });
    let announce = sample.as_ref().map(|b| BlobAnnounce { id: 1, len: b.len() as u64, sha256: fractadyne_farm::sha256_hex(b) });
    let probe = result.ok().map(|(png, _)| png).filter(|p| p.len() as u64 <= MAX_PROBE_BYTES);
    let probe_announce = probe.as_ref().map(|b| BlobAnnounce { id: 2, len: b.len() as u64, sha256: fractadyne_farm::sha256_hex(b) });
    let record = items.iter().map(|i| format!("{} {}: {}", if i.ok { "ok  " } else { "FAIL" }, i.name, i.detail)).collect::<Vec<_>>().join("\n");
    for i in &items {
        println!("  self-check {} {}: {}", if i.ok { "ok  " } else { "FAIL" }, i.name, i.detail);
    }
    (SelfCheck { items, gpu, free_bytes: free, link_sample: announce, probe: probe_announce }, sample, probe, cap, record)
}

fn tail_of(s: &str, n: usize) -> String {
    fractadyne_farm::proto::tail(s, n).replace(|c: char| c.is_control() && c != '\n', " ")
}

struct Job {
    dir: PathBuf,
    bundle: Bundle,
    refusal: Option<String>,
    /// Where its frames are written: the job's local folder, or its folder on the shared drive.
    frames: PathBuf,
    share: bool,
}

struct Running {
    assign: Assign,
    child: std::process::Child,
    reported: HashSet<u64>,
    last_done: Option<u64>,
    frame_started: Instant,
    stop: Option<AbortReason>,
    pause_after_frame: bool,
    /// Cut here (stealing): stop once the frame before it is done.
    stop_from: Option<u64>,
}

struct Session<'a> {
    cfg: &'a Cfg,
    ui: Arc<Ui>,
    out: mpsc::Sender<Out>,
    ev_tx: mpsc::Sender<Ev>,
    next_blob: u64,
    jobs: HashMap<String, Job>,
    pending_bundle: Option<(JobOpen, BlobSink<Vec<u8>>)>,
    queue: VecDeque<Assign>,
    running: Option<Running>,
    child_tail: Arc<Mutex<VecDeque<String>>>,
    heartbeats: VecDeque<String>,
    handshake: String,
    last_heartbeat: Instant,
    paused: bool,
    frames_done: u64,
    /// Render milliseconds of the frames sent this session, for the mean.
    ms_total: u64,
    gpu_cap: Option<u64>,
    corrupt_left: u32,
}

impl Session<'_> {
    fn send(&self, m: Msg) {
        let _ = self.out.send(Out::Msg(m));
    }

    fn run(&mut self, ev: &mpsc::Receiver<Ev>) -> Ended {
        loop {
            let Ok(e) = ev.recv() else {
                return Ended::Retry { connected: true, why: "the connection ended".into() };
            };
            match e {
                Ev::Net(Incoming::Control(m)) => {
                    if let Some(end) = self.on_msg(m) {
                        return end;
                    }
                }
                Ev::Net(Incoming::Chunk { id, offset, data }) => {
                    if let Err(e) = self.on_chunk(id, offset, &data) {
                        return Ended::Retry { connected: true, why: format!("protocol error from the controller: {e}") };
                    }
                }
                Ev::NetDown(e) => return Ended::Retry { connected: true, why: format!("lost the controller ({e})") },
                Ev::Child { run, line } => self.on_child_line(run, line),
                Ev::ChildClosed { run } => self.on_child_closed(run),
                Ev::Tick => {
                    if let Some(end) = self.on_tick() {
                        return end;
                    }
                }
            }
        }
    }

    fn on_msg(&mut self, m: Msg) -> Option<Ended> {
        match m {
            Msg::JobOpen(j) => {
                if j.bundle.len == 0 {
                    return Some(Ended::Retry { connected: true, why: "an empty job bundle".into() });
                }
                let sink = BlobSink::new(j.bundle.clone(), Vec::new());
                println!("Job \"{}\" ({}) — receiving its bundle", j.name, j.job_id);
                self.ui.update(|s| {
                    s.job = Some(j.name.clone());
                    s.detail = format!("Job \"{}\" — receiving its bundle", j.name);
                });
                self.pending_bundle = Some((j, sink));
            }
            Msg::Assign(a) => {
                let Some(job) = self.jobs.get(&a.job_id) else {
                    return Some(Ended::Retry { connected: true, why: format!("work for unknown job {}", a.job_id) });
                };
                if let Some(why) = &job.refusal {
                    self.send(Msg::RunAborted(RunAborted { job_id: a.job_id.clone(), run_id: a.run_id, done_up_to: None, reason: AbortReason::Policy(why.clone()) }));
                } else if self.paused {
                    self.send(Msg::RunAborted(RunAborted { job_id: a.job_id.clone(), run_id: a.run_id, done_up_to: None, reason: AbortReason::Paused }));
                } else {
                    self.queue.push_back(a);
                    self.maybe_start();
                }
            }
            Msg::Cancel(c) if c.from.is_some() => {
                // A cut: the frames from `from` on went to another client.
                let (from, run) = (c.from.unwrap_or(0), c.run_id.unwrap_or(0));
                for q in self.queue.iter_mut().filter(|q| q.job_id == c.job_id && q.run_id == run) {
                    q.end = q.end.min(from);
                }
                self.queue.retain(|q| q.start < q.end);
                if let Some(r) = self.running.as_mut().filter(|r| r.assign.job_id == c.job_id && r.assign.run_id == run) {
                    let at = r.last_done.map_or(r.assign.start, |d| d + 1);
                    println!("  run {run} cut at frame {from}: another machine renders the rest");
                    r.stop_from = Some(from);
                    if at >= from {
                        self.stop_child(Some(AbortReason::Canceled));
                    }
                }
            }
            Msg::Cancel(c) => {
                self.queue.retain(|q| !(q.job_id == c.job_id && c.run_id.is_none_or(|r| r == q.run_id)));
                let hit = self.running.as_ref().is_some_and(|r| r.assign.job_id == c.job_id && c.run_id.is_none_or(|x| x == r.assign.run_id));
                if hit {
                    println!("  run cancelled by the controller ({:?})", c.reason);
                    self.stop_child(Some(AbortReason::Canceled));
                }
            }
            Msg::JobClose(c) => {
                self.queue.retain(|q| q.job_id != c.job_id);
                if self.running.as_ref().is_some_and(|r| r.assign.job_id == c.job_id) {
                    self.stop_child(Some(AbortReason::Canceled));
                }
                // The frames go; the render processes' logs stay (a field run collects them, and
                // they are small).
                if let Some(j) = self.jobs.remove(&c.job_id) {
                    let _ = std::fs::remove_dir_all(j.dir.join("frames"));
                    // Its folder on the shared drive: the frames were moved out by the controller;
                    // the render's status file is ours to remove, and the folders if empty.
                    if j.share {
                        let _ = std::fs::remove_file(j.frames.join("render-status.txt"));
                        let _ = std::fs::remove_dir(&j.frames);
                        if let Some(parent) = j.frames.parent() {
                            let _ = std::fs::remove_dir(parent);
                        }
                    }
                }
                println!("Job {} closed by the controller", c.job_id);
                self.ui.update(|s| {
                    s.detail = "The job is over".into();
                    s.run = None;
                    s.frame = None;
                    if s.phase == ClientPhase::Rendering {
                        s.phase = ClientPhase::Idle;
                    }
                });
                if self.cfg.one_job {
                    println!("--one-job: the job is over; exiting");
                    return Some(Ended::Exit(0));
                }
            }
            Msg::DiagRequest(d) => {
                let mut items = Vec::new();
                for it in d.items {
                    let text = match it {
                        DiagItem::ChildLog => self.child_tail.lock().unwrap_or_else(|e| e.into_inner()).iter().cloned().collect::<Vec<_>>().join("\n"),
                        DiagItem::Handshake => self.handshake.clone(),
                        DiagItem::Heartbeats => self.heartbeats.iter().cloned().collect::<Vec<_>>().join("\n"),
                    };
                    // Control characters escape to six bytes in JSON; keep well under the cap.
                    items.push((it, tail_of(&text, MAX_DIAG_ITEM / 2)));
                }
                self.send(Msg::DiagReport(DiagReport { items }));
            }
            Msg::Bye(b) => {
                eprintln!("fractadyne: the controller ended the connection: {}", b.reason);
                self.ui.end(3, &format!("The controller ended the connection: {}", b.reason));
                return Some(Ended::Exit(3));
            }
            Msg::Keepalive => {}
            other => {
                return Some(Ended::Retry { connected: true, why: format!("the controller sent a {} message, which only a client sends", kind_name(&other)) });
            }
        }
        None
    }

    fn on_chunk(&mut self, id: u64, offset: u64, data: &[u8]) -> Result<(), String> {
        let Some((j, sink)) = self.pending_bundle.as_mut() else {
            return Err(format!("a chunk of unannounced blob {id}"));
        };
        if j.bundle.id != id {
            return Err(format!("a chunk of blob {id}, expected {}", j.bundle.id));
        }
        if sink.push(offset, data).map_err(|e| e.to_string())? == BlobProgress::More {
            return Ok(());
        }
        let (j, sink) = self.pending_bundle.take().ok_or("no bundle")?;
        let bundle: Bundle = serde_json::from_slice(&sink.into_inner()).map_err(|e| format!("job bundle: {e}"))?;
        let refusal = bundle.check(&self.cfg.policy, self.gpu_cap).err();
        if let Some(r) = &refusal {
            println!("  job refused by this machine's policy: {r}");
        }
        let dir = self.cfg.work.join(&j.job_id);
        let prepare = || -> Result<(), String> {
            std::fs::create_dir_all(dir.join("frames")).map_err(|e| e.to_string())?;
            std::fs::write(dir.join("script.toml"), &bundle.script).map_err(|e| e.to_string())?;
            if let Some(a) = &bundle.anchors {
                std::fs::write(dir.join("anchors.toml"), a).map_err(|e| e.to_string())?;
            }
            let cfg = dir.join("cfg");
            let _ = std::fs::remove_dir_all(&cfg);
            write_session(&cfg, &bundle.settings)
        };
        let refusal = match (refusal, prepare()) {
            (Some(r), _) => Some(r),
            (None, Err(e)) => Some(format!("could not prepare the job folder: {e}")),
            (None, Ok(())) => None,
        };
        // Share mode, when the job offers it and this machine has the shared drive: proved
        // writable here before a frame is rendered into it, else this machine streams.
        let mut frames = dir.join("frames");
        let mut share = false;
        if let (true, Some(root)) = (bundle.share, &self.cfg.share_root) {
            let try_share = || -> Result<PathBuf, String> {
                let d = fractadyne_farm::names::share_dir(root, &j.job_id, &self.cfg.name)?;
                std::fs::create_dir_all(&d).map_err(|e| format!("{}: {e}", d.display()))?;
                let probe = d.join(".write-test");
                std::fs::write(&probe, b"ok").map_err(|e| format!("{}: {e}", d.display()))?;
                let _ = std::fs::remove_file(&probe);
                Ok(d)
            };
            match try_share() {
                Ok(d) => {
                    println!("  frames go to the shared drive: {}", d.display());
                    frames = d;
                    share = true;
                }
                Err(e) => println!("  the shared drive is not writable from here ({e}); streaming the frames instead"),
            }
        }
        let detail = format!("{}×{} ss{} at {} fps, {} frames", bundle.width, bundle.height, bundle.ss, bundle.fps, bundle.frames);
        println!("  job \"{}\": {detail}", bundle.name);
        self.ui.update(|s| {
            s.job_detail = Some(detail);
            s.detail = match &refusal {
                Some(r) => format!("Refused this job: {r}"),
                None => format!("Job \"{}\" — waiting for frames to render", bundle.name),
            };
        });
        self.jobs.insert(j.job_id.clone(), Job { dir, bundle, refusal, frames, share });
        Ok(())
    }

    fn maybe_start(&mut self) {
        if self.running.is_some() || self.paused {
            return;
        }
        let Some(a) = self.queue.pop_front() else { return };
        let Some(job) = self.jobs.get(&a.job_id) else { return };
        let b = &job.bundle;
        let mut args: Vec<String> = vec![
            "--render-tour".into(),
            job.dir.join("script.toml").to_string_lossy().into_owned(),
            "--out".into(),
            job.frames.to_string_lossy().into_owned(),
            "--frames".into(),
            format!("{}..{}", a.start, a.end),
            "--size".into(),
            format!("{}x{}", b.width, b.height),
            "--fps".into(),
            format!("{}", b.fps),
            "--ss".into(),
            b.ss.to_string(),
            "--prefix".into(),
            b.prefix.clone(),
            "--farm-child".into(),
            "-y".into(),
        ];
        if b.anchors.is_some() {
            args.push("--norm-anchors".into());
            args.push(job.dir.join("anchors.toml").to_string_lossy().into_owned());
        }
        if let Some(cap) = b.orbit_len_cap {
            args.push("--set".into());
            args.push(format!("ORBIT_LEN_CAP={cap}"));
        }
        self.child_tail.lock().unwrap_or_else(|e| e.into_inner()).clear();
        match spawn_child(&args, &job.dir.join("cfg")) {
            Ok(mut child) => {
                let run = a.run_id;
                let tx = self.ev_tx.clone();
                let tx2 = self.ev_tx.clone();
                // stdout: every line, then a "closed" event once the pipe ends — so every
                // frame-done line is handled before the run is judged finished.
                if let Some(out) = child.stdout.take() {
                    std::thread::spawn(move || {
                        for line in std::io::BufReader::new(out).lines().map_while(Result::ok) {
                            if tx.send(Ev::Child { run, line: parse_child_line(&line) }).is_err() {
                                return;
                            }
                        }
                        let _ = tx2.send(Ev::ChildClosed { run });
                    });
                }
                pump_child(&mut child, |_| {}, self.child_tail.clone(), 400);
                println!("  rendering frames {}..{} (run {})", a.start, a.end, a.run_id);
                self.ui.update(|s| {
                    s.phase = ClientPhase::Rendering;
                    s.run = Some((a.start, a.end));
                    s.frame = Some(a.start);
                    s.detail = format!("Rendering frames {}–{}", a.start, a.end - 1);
                });
                self.running = Some(Running { assign: a, child, reported: HashSet::new(), last_done: None, frame_started: Instant::now(), stop: None, pause_after_frame: false, stop_from: None });
            }
            Err(e) => {
                eprintln!("fractadyne: {e}");
                self.send(Msg::RunAborted(RunAborted { job_id: a.job_id, run_id: a.run_id, done_up_to: None, reason: AbortReason::ChildCrash(-1) }));
            }
        }
    }

    /// Kill the running child (if any); `why` is reported when its stdout closes.
    fn stop_child(&mut self, why: Option<AbortReason>) {
        if let Some(r) = self.running.as_mut() {
            if r.stop.is_none() {
                r.stop = why.or(Some(AbortReason::Canceled));
            }
            let _ = r.child.kill();
        }
    }

    fn on_child_line(&mut self, run: u64, line: ChildLine) {
        let Some(r) = self.running.as_mut().filter(|r| r.assign.run_id == run) else { return };
        match line {
            ChildLine::Done { index, bytes, sha256, ms } if self.jobs.get(&r.assign.job_id).is_some_and(|j| j.share) => {
                // Share mode: the frame is in this machine's folder on the shared drive, its digest
                // taken by the render as it read the file back. Announce it; the controller reads
                // and checks it there.
                let job = &self.jobs[&r.assign.job_id];
                let path = job.frames.join(fractadyne_farm::names::frame_file_name(&job.bundle.prefix, index));
                if self.corrupt_left > 0 {
                    self.corrupt_left -= 1;
                    if let Ok(mut data) = std::fs::read(&path) {
                        let mid = data.len() / 2;
                        data[mid] ^= 0x55;
                        let _ = std::fs::write(&path, &data);
                        eprintln!("  (instrument {CORRUPT_INSTRUMENT}: frame {index} left corrupted on purpose)");
                    }
                }
                let id = self.next_blob;
                self.next_blob += 1;
                let _ = self.out.send(Out::Msg(Msg::FrameDone(FrameDone { job_id: r.assign.job_id.clone(), run_id: run, index, render_ms: ms, blob: BlobAnnounce { id, len: bytes, sha256 }, on_share: true })));
                r.reported.insert(index);
                r.last_done = Some(r.last_done.map_or(index, |d| d.max(index)));
                r.frame_started = Instant::now();
                self.frames_done += 1;
                self.ms_total += ms;
                println!("  frame {index} on the shared drive ({} KB, {ms} ms)", bytes / 1024);
                let (done, mean) = (self.frames_done, self.ms_total as f64 / self.frames_done as f64);
                let end = r.assign.end;
                self.ui.update(|s| {
                    s.frames_done = done;
                    s.mean_ms = Some(mean);
                    s.frame = (index + 1 < end).then_some(index + 1);
                    s.last_frame = Some(path.to_string_lossy().into_owned());
                    s.last_frame_seq += 1;
                });
                if r.pause_after_frame {
                    r.stop = Some(AbortReason::Paused);
                    let _ = r.child.kill();
                } else if r.stop_from.is_some_and(|f| index + 1 >= f) {
                    r.stop = Some(AbortReason::Canceled);
                    let _ = r.child.kill();
                }
            }
            ChildLine::Done { index, bytes, sha256, ms } => {
                let job = &self.jobs[&r.assign.job_id];
                let path = job.frames.join(fractadyne_farm::names::frame_file_name(&job.bundle.prefix, index));
                match std::fs::read(&path) {
                    Ok(mut data) if data.len() as u64 == bytes && fractadyne_farm::sha256_hex(&data) == sha256 => {
                        let id = self.next_blob;
                        self.next_blob += 1;
                        let announce = BlobAnnounce { id, len: data.len() as u64, sha256 };
                        if self.corrupt_left > 0 {
                            self.corrupt_left -= 1;
                            let last = data.len() / 2;
                            data[last] ^= 0x55;
                            eprintln!("  (instrument {CORRUPT_INSTRUMENT}: frame {index} sent corrupted on purpose)");
                        }
                        let _ = self.out.send(Out::Msg(Msg::FrameDone(FrameDone { job_id: r.assign.job_id.clone(), run_id: run, index, render_ms: ms, blob: announce, on_share: false })));
                        let _ = self.out.send(Out::Blob(id, data));
                        // Kept as the window's thumbnail of the latest frame (the next replaces it).
                        let last = self.cfg.control.join("last-frame.png");
                        if std::fs::rename(&path, &last).is_err() {
                            let _ = std::fs::remove_file(&path);
                        }
                        r.reported.insert(index);
                        r.last_done = Some(r.last_done.map_or(index, |d| d.max(index)));
                        r.frame_started = Instant::now();
                        self.frames_done += 1;
                        self.ms_total += ms;
                        println!("  frame {index} sent ({} KB, {ms} ms)", bytes / 1024);
                        let (done, mean) = (self.frames_done, self.ms_total as f64 / self.frames_done as f64);
                        let end = r.assign.end;
                        self.ui.update(|s| {
                            s.frames_done = done;
                            s.mean_ms = Some(mean);
                            s.frame = (index + 1 < end).then_some(index + 1);
                            s.last_frame = Some(last.to_string_lossy().into_owned());
                            s.last_frame_seq += 1;
                        });
                        if r.pause_after_frame {
                            r.stop = Some(AbortReason::Paused);
                            let _ = r.child.kill();
                        } else if r.stop_from.is_some_and(|f| index + 1 >= f) {
                            r.stop = Some(AbortReason::Canceled);
                            let _ = r.child.kill();
                        }
                    }
                    other => {
                        let why = match other {
                            Ok(_) => "the file on disk does not match what the render reported".to_string(),
                            Err(e) => e.to_string(),
                        };
                        let _ = self.out.send(Out::Msg(Msg::FrameFailed(FrameFailed { job_id: r.assign.job_id.clone(), run_id: run, index, class: FailClass::ReadBack, message: tail_of(&why, 500) })));
                    }
                }
            }
            ChildLine::Failed { index, reason } => {
                let _ = self.out.send(Out::Msg(Msg::FrameFailed(FrameFailed { job_id: r.assign.job_id.clone(), run_id: run, index, class: FailClass::Other, message: tail_of(&reason, 500) })));
            }
            ChildLine::Other(_) => {}
        }
    }

    fn on_child_closed(&mut self, run: u64) {
        let Some(mut r) = self.running.take_if(|r| r.assign.run_id == run) else { return };
        let code = r.child.wait().ok().and_then(|s| s.code());
        let end = r.stop_from.map_or(r.assign.end, |f| f.min(r.assign.end));
        let complete = (r.assign.start..end).all(|i| r.reported.contains(&i));
        let reason = match (r.stop.take(), code) {
            (Some(why), _) => Some(why),
            (None, Some(0)) if complete => None,
            (None, Some(c)) => Some(AbortReason::ChildCrash(c)),
            (None, None) => Some(AbortReason::ChildCrash(-1)),
        };
        if let Some(why) = reason {
            if !matches!(why, AbortReason::Canceled) || !complete {
                println!("  run {} stopped: {why:?}", run);
            }
            self.send(Msg::RunAborted(RunAborted { job_id: r.assign.job_id.clone(), run_id: run, done_up_to: r.last_done, reason: why }));
        }
        let paused = self.paused;
        self.ui.update(|s| {
            s.run = None;
            s.frame = None;
            s.phase = if paused { ClientPhase::Paused } else { ClientPhase::Idle };
            s.detail = if paused { "Paused — the controller has been told".into() } else { format!("Connected to \"{}\" — idle", s.controller_name) };
        });
        self.maybe_start();
    }

    fn on_tick(&mut self) -> Option<Ended> {
        // The window's commands, and the headless control files.
        self.ui.poll();
        if self.ui.leaving() {
            println!("Disconnecting (its user left)");
            self.stop_child(Some(AbortReason::UserCancel));
            self.send(Msg::Bye(Bye { reason: "its user disconnected".into() }));
            std::thread::sleep(Duration::from_millis(300)); // let the goodbye go out
            self.ui.end(0, "Disconnected");
            return Some(Ended::Exit(0));
        }
        let pause = self.cfg.control.join("PAUSE").exists() || self.ui.paused.load(Ordering::Relaxed);
        let cancel = self.cfg.control.join("CANCEL");
        let cancel_by_ui = self.ui.cancel.swap(false, Ordering::Relaxed);
        if cancel.exists() || cancel_by_ui {
            if cancel.exists() {
                let _ = std::fs::remove_file(&cancel);
                let _ = std::fs::write(self.cfg.control.join("PAUSE"), b"created by CANCEL; delete to resume\n");
            }
            if self.running.is_some() {
                println!("Cancelled the frame in progress; paused");
                self.stop_child(Some(AbortReason::UserCancel));
            }
            self.paused = true;
            self.ui.update(|s| {
                s.paused = true;
                s.phase = ClientPhase::Paused;
                s.detail = "Cancelled the frame in progress; paused".into();
            });
        } else if pause != self.paused {
            self.paused = pause;
            if pause {
                println!("Paused: finishing the current frame, then taking no work");
                let running = self.running.is_some();
                if let Some(r) = self.running.as_mut() {
                    r.pause_after_frame = true;
                }
                self.ui.update(|s| {
                    s.paused = true;
                    if !running {
                        s.phase = ClientPhase::Paused;
                    }
                    s.detail = if running { "Pausing — finishing the frame in progress".into() } else { "Paused — the controller has been told".into() };
                });
            } else {
                println!("Resumed");
                self.ui.update(|s| {
                    s.paused = false;
                    if s.phase == ClientPhase::Paused {
                        s.phase = ClientPhase::Idle;
                    }
                    s.detail = format!("Connected to \"{}\" — idle", s.controller_name);
                });
                self.maybe_start();
            }
        }
        if let Some(r) = &self.running {
            let ms = r.frame_started.elapsed().as_millis() as u64;
            self.ui.update(|s| s.frame_ms = Some(ms));
        }
        if self.last_heartbeat.elapsed() >= Duration::from_secs(2) {
            self.last_heartbeat = Instant::now();
            let hb = Heartbeat {
                activity: if self.paused {
                    ClientActivity::Paused
                } else if self.running.is_some() {
                    ClientActivity::Rendering
                } else {
                    ClientActivity::Idle
                },
                job_id: self.running.as_ref().map(|r| r.assign.job_id.clone()),
                run_id: self.running.as_ref().map(|r| r.assign.run_id),
                frame: self.running.as_ref().map(|r| r.last_done.map_or(r.assign.start, |d| d + 1)),
                frame_ms: self.running.as_ref().map(|r| r.frame_started.elapsed().as_millis() as u64),
                frames_done: self.frames_done,
                free_bytes: crate::sysinfo::free_disk_bytes(&self.cfg.work),
            };
            self.heartbeats.push_back(serde_json::to_string(&hb).unwrap_or_default());
            while self.heartbeats.len() > 60 {
                self.heartbeats.pop_front();
            }
            self.send(Msg::Heartbeat(hb));
        }
        None
    }
}
