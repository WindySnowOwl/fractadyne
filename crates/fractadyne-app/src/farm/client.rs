//! `--render-client HOST:PORT`: a machine that renders frames for a controller (design §10).
//!
//! It dials the controller (it opens no port), proves it holds the farm key, pins the controller's
//! identity, runs the handshake self-check, and then renders the runs it is assigned — each as a
//! `--render-tour … --farm-child` child in the job's own configuration folder, built from the job's
//! settings, never this machine's session. Every frame the child reports is re-read, checked against
//! the digest the child printed, and streamed to the controller, which verifies it again.
//!
//! Headless controls until the Phase 2 dialog: create `<config>/farm/PAUSE` to finish the current
//! frame and stop taking work (delete it to resume); create `CANCEL` to stop at once (it also
//! pauses). A deliberate `Bye` from the controller — for instance a removal — ends the client; a
//! connection that merely drops is redialled with back-off.

use super::*;
use fractadyne_farm::channel::{self, Pin, PinStore};
use fractadyne_farm::proto::*;
use std::collections::{HashMap, HashSet, VecDeque};
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
    let cfg = Cfg { addr, key, name, policy, allow_dirty, one_job, work: dir.join("jobs"), control: dir.clone(), id };
    println!("Render client \"{}\" (identity {}) — controller {}", cfg.name, cfg.id.fingerprint(), cfg.addr);
    println!("  pause: create {}  ·  cancel the frame in progress: create {}", cfg.control.join("PAUSE").display(), cfg.control.join("CANCEL").display());
    let mut backoff = Duration::from_secs(2);
    loop {
        match connect_once(&cfg, &mut pins) {
            Ended::Exit(code) => return code,
            Ended::Retry { connected, why } => {
                if connected {
                    backoff = Duration::from_secs(2);
                }
                println!("{why} — retrying in {}s", backoff.as_secs());
                crate::diag::log_line("farm", &format!("client: {why}"));
                std::thread::sleep(backoff);
                backoff = (backoff * 2).min(Duration::from_secs(60));
            }
        }
    }
}

fn connect_once(cfg: &Cfg, pins: &mut PinStore) -> Ended {
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
    let link_bytes = loop {
        let wait = deadline.saturating_duration_since(Instant::now()).max(Duration::from_millis(1));
        match ev_rx.recv_timeout(wait) {
            Ok(Ev::Net(Incoming::Control(Msg::HelloAck(a)))) => match a.verdict {
                // (Authenticated: only a holder of the farm key can produce this message.)
                _ if matches!(pin, Pin::Changed { .. }) => {
                    let Pin::Changed { pinned } = &pin else { unreachable!() };
                    eprintln!(
                        "fractadyne: the controller at {} changed identity (pinned {pinned}, now {fp}). If that is expected — it was reinstalled — delete its line from {} and start again.",
                        cfg.addr,
                        cfg.control.join("known-controllers.toml").display()
                    );
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
                    break a.link_sample_bytes;
                }
                Verdict::WaitingForApproval => println!("Connected to \"{}\" — waiting for its user to approve this machine", a.name),
                Verdict::Refused(r) => {
                    eprintln!("fractadyne: the controller refused this machine: {r}");
                    return Ended::Exit(3);
                }
            },
            Ok(Ev::Net(Incoming::Control(Msg::Bye(b)))) => {
                eprintln!("fractadyne: the controller closed the connection: {}", b.reason);
                return Ended::Exit(3);
            }
            Ok(Ev::NetDown(e)) => return retry(format!("the controller closed the connection during the handshake ({e}) — most often a different farm key")),
            Ok(Ev::Tick) | Ok(Ev::Net(Incoming::Control(Msg::Keepalive))) => {}
            Ok(_) => return retry("the controller sent something other than its verdict".into()),
            Err(_) if Instant::now() < deadline => {}
            Err(_) => return retry("the controller gave no verdict in time".into()),
        }
    };
    let (check, sample, gpu_cap, handshake) = self_check(cfg, link_bytes);
    let failed_hard = check.items.iter().find(|i| i.hard && !i.ok).map(|i| format!("{}: {}", i.name, i.detail));
    let _ = out_tx.send(Out::Msg(Msg::SelfCheck(check)));
    if let Some(blob) = sample {
        let _ = out_tx.send(Out::Blob(1, blob));
    }
    if let Some(f) = failed_hard {
        eprintln!("fractadyne: the self-check failed: {f}");
    }
    let mut s = Session {
        cfg,
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
        gpu_cap,
        corrupt_left: crate::tunables::instrument(CORRUPT_INSTRUMENT),
    };
    let ended = s.run(&ev_rx);
    s.stop_child(None);
    ended
}

/// The handshake self-check (design §9.1): a one-frame render (device, render path, child launch,
/// adapter, orbit cap), free space, and the link sample the controller asked for.
fn self_check(cfg: &Cfg, link_bytes: u64) -> (SelfCheck, Option<Vec<u8>>, Option<u64>, String) {
    let mut items = Vec::new();
    let dir = cfg.work.join("self-check");
    let _ = std::fs::remove_dir_all(&dir);
    let tour = dir.join("self-check.toml");
    let t0 = Instant::now();
    let result = std::fs::create_dir_all(&dir)
        .map_err(|e| e.to_string())
        .and_then(|()| std::fs::write(&tour, SELF_CHECK_TOUR).map_err(|e| e.to_string()))
        .and_then(|()| {
            let args: Vec<String> = ["--render-tour", &tour.to_string_lossy(), "--out", &dir.join("frames").to_string_lossy(), "--frames", "0..1", "--farm-child", "-y"]
                .iter()
                .map(|s| s.to_string())
                .collect();
            let mut child = spawn_child(&args, &dir.join("cfg"))?;
            let tail = Arc::new(Mutex::new(VecDeque::new()));
            let (tx, rx) = mpsc::channel();
            pump_child(&mut child, move |l| { let _ = tx.send(l); }, tail.clone(), 400);
            let deadline = Instant::now() + Duration::from_secs(120);
            let status = loop {
                if let Some(st) = child.try_wait().map_err(|e| e.to_string())? {
                    break st;
                }
                if Instant::now() > deadline {
                    let _ = child.kill();
                    return Err("the test render did not finish within 120 s".into());
                }
                std::thread::sleep(Duration::from_millis(50));
            };
            std::thread::sleep(Duration::from_millis(100)); // let the pumps drain
            let done = rx.try_iter().any(|l| matches!(parse_child_line(&l), ChildLine::Done { .. }));
            let err: String = tail.lock().unwrap_or_else(|e| e.into_inner()).iter().cloned().collect::<Vec<_>>().join("\n");
            if status.success() && done {
                Ok(err)
            } else {
                Err(format!("the test render failed (exit {:?}): {}", status.code(), tail_of(&err, 300)))
            }
        });
    let (gpu, cap) = match &result {
        Ok(stderr) => {
            let (adapter, cap) = gpu_facts(stderr);
            items.push(CheckItem {
                name: "render".into(),
                ok: true,
                hard: true,
                detail: format!("test frame in {} ms on {}", t0.elapsed().as_millis(), adapter.as_deref().unwrap_or("an unnamed adapter")),
            });
            (adapter.map(|a| GpuInfo { adapter: a, orbit_len_cap: cap.unwrap_or(0) }), cap)
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
    let record = items.iter().map(|i| format!("{} {}: {}", if i.ok { "ok  " } else { "FAIL" }, i.name, i.detail)).collect::<Vec<_>>().join("\n");
    for i in &items {
        println!("  self-check {} {}: {}", if i.ok { "ok  " } else { "FAIL" }, i.name, i.detail);
    }
    (SelfCheck { items, gpu, free_bytes: free, link_sample: announce }, sample, cap, record)
}

fn tail_of(s: &str, n: usize) -> String {
    fractadyne_farm::proto::tail(s, n).replace(|c: char| c.is_control() && c != '\n', " ")
}

struct Job {
    dir: PathBuf,
    bundle: Bundle,
    refusal: Option<String>,
}

struct Running {
    assign: Assign,
    child: std::process::Child,
    reported: HashSet<u64>,
    last_done: Option<u64>,
    frame_started: Instant,
    stop: Option<AbortReason>,
    pause_after_frame: bool,
}

struct Session<'a> {
    cfg: &'a Cfg,
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
                Ev::Tick => self.on_tick(),
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
                }
                println!("Job {} closed by the controller", c.job_id);
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
        println!("  job \"{}\": {}×{} ss{} at {} fps, {} frames", bundle.name, bundle.width, bundle.height, bundle.ss, bundle.fps, bundle.frames);
        self.jobs.insert(j.job_id.clone(), Job { dir, bundle, refusal });
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
            job.dir.join("frames").to_string_lossy().into_owned(),
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
                self.running = Some(Running { assign: a, child, reported: HashSet::new(), last_done: None, frame_started: Instant::now(), stop: None, pause_after_frame: false });
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
            ChildLine::Done { index, bytes, sha256, ms } => {
                let job = &self.jobs[&r.assign.job_id];
                let path = job.dir.join("frames").join(fractadyne_farm::names::frame_file_name(&job.bundle.prefix, index));
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
                        let _ = self.out.send(Out::Msg(Msg::FrameDone(FrameDone { job_id: r.assign.job_id.clone(), run_id: run, index, render_ms: ms, blob: announce })));
                        let _ = self.out.send(Out::Blob(id, data));
                        let _ = std::fs::remove_file(&path);
                        r.reported.insert(index);
                        r.last_done = Some(r.last_done.map_or(index, |d| d.max(index)));
                        r.frame_started = Instant::now();
                        self.frames_done += 1;
                        println!("  frame {index} sent ({} KB, {ms} ms)", bytes / 1024);
                        if r.pause_after_frame {
                            r.stop = Some(AbortReason::Paused);
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
        let complete = (r.assign.start..r.assign.end).all(|i| r.reported.contains(&i));
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
        self.maybe_start();
    }

    fn on_tick(&mut self) {
        // The headless controls.
        let pause = self.cfg.control.join("PAUSE").exists();
        let cancel = self.cfg.control.join("CANCEL");
        if cancel.exists() {
            let _ = std::fs::remove_file(&cancel);
            let _ = std::fs::write(self.cfg.control.join("PAUSE"), b"created by CANCEL; delete to resume\n");
            if self.running.is_some() {
                println!("Cancelled the frame in progress; paused (delete PAUSE to resume)");
                self.stop_child(Some(AbortReason::UserCancel));
            }
            self.paused = true;
        } else if pause != self.paused {
            self.paused = pause;
            if pause {
                println!("Paused: finishing the current frame, then taking no work (delete PAUSE to resume)");
                if let Some(r) = self.running.as_mut() {
                    r.pause_after_frame = true;
                }
            } else {
                println!("Resumed");
                self.maybe_start();
            }
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
    }
}
