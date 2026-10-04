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
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

/// Test instrument (`--farmtest`): corrupt the first N frames this client sends, AFTER computing
/// the digest it announces — so the controller's verification must catch them. Listed in
/// `tunables::INSTRUMENTS`, so a client running with it reports itself as not stock.
pub(crate) const CORRUPT_INSTRUMENT: &str = "FRACTADYNE_FARM_CORRUPT_FRAMES";

/// Test instrument (`--farmtest`): with `--when-idle`, someone sits down at this machine as its
/// first frame starts and stays N seconds — so the harness can prove the frame goes back to the
/// farm and work resumes once the machine is idle again. Listed in `tunables::INSTRUMENTS`.
pub(crate) const IN_USE_INSTRUMENT: &str = "FRACTADYNE_FARM_IN_USE_FOR";

// What one session of several (`--adapters`) prints says which: "[GPU 2] frame 14 sent".
thread_local! {
    static TAG: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

macro_rules! say {
    ($($t:tt)*) => {
        say_line(format!($($t)*))
    };
}

fn say_line(s: String) {
    TAG.with(|t| match t.borrow().as_str() {
        "" => println!("{s}"),
        tag => println!("[{tag}] {s}"),
    })
}

struct Cfg {
    addr: String,
    key: Arc<FarmKey>,
    id: Arc<Identity>,
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
    /// `--adapter` / `--adapters`: the graphics card this session's renders use — its
    /// `--list-adapters` number (what they are given) and its name — or wgpu's choice.
    adapter: Option<(String, String)>,
    /// `--when-idle`: take work only once nobody has used this machine for this long.
    when_idle: Option<Duration>,
    /// Which session of several this is (`--adapters`), from 1.
    slot: Option<u32>,
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

/// What the sessions of one client share (one per graphics card with `--adapters`): the window's
/// commands — one stdin — and the pause, cancel and leave they ask for. Pause and cancel from the
/// window persist across reconnects, like the files.
struct Control {
    on: bool,
    cmds: Mutex<Option<mpsc::Receiver<Option<String>>>>,
    paused: AtomicBool,
    /// Bumped by each "cancel the frame in progress" (the window's, or a CANCEL file): every
    /// session stops its frame once per bump.
    cancels: AtomicU64,
    leave: AtomicBool,
}

impl Control {
    fn new(on: bool) -> Arc<Control> {
        Arc::new(Control {
            on,
            cmds: Mutex::new(on.then(status::commands)),
            paused: AtomicBool::new(false),
            cancels: AtomicU64::new(0),
            leave: AtomicBool::new(false),
        })
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
                    self.cancels.fetch_add(1, Ordering::Relaxed);
                    self.paused.store(true, Ordering::Relaxed);
                }
                Some(Some(ClientCommand::Leave)) | None => self.leave.store(true, Ordering::Relaxed),
                Some(None) => eprintln!("fractadyne: render client: unknown command {:?}", c.unwrap_or_default()),
            }
        }
    }
}

/// The `--ui-status` side of one session: the status the app's window reads — a line per session,
/// told apart by `slot` — and the shared [`Control`]. Inert without the flag.
struct Ui {
    on: bool,
    st: Mutex<ClientStatus>,
    ctl: Arc<Control>,
    /// The last `Control::cancels` this session acted on.
    cancels_seen: AtomicU64,
}

impl Ui {
    fn new(ctl: Arc<Control>, slot: Option<u32>) -> Arc<Ui> {
        let on = ctl.on;
        let ui = Arc::new(Ui {
            on,
            st: Mutex::new(ClientStatus { slot, ..ClientStatus::default() }),
            cancels_seen: AtomicU64::new(ctl.cancels.load(Ordering::Relaxed)),
            ctl,
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

    fn poll(&self) {
        self.ctl.poll();
    }

    fn leaving(&self) -> bool {
        self.ctl.leave.load(Ordering::Relaxed)
    }

    fn paused(&self) -> bool {
        self.ctl.paused.load(Ordering::Relaxed)
    }

    /// A cancel this session has not acted on yet.
    fn take_cancel(&self) -> bool {
        let now = self.ctl.cancels.load(Ordering::Relaxed);
        self.cancels_seen.swap(now, Ordering::Relaxed) != now
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
    let pins = match PinStore::load(&dir.join("known-controllers.toml")) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    let pins = Mutex::new(pins);
    let one_job = args.iter().any(|a| a == "--one-job");
    let share_root = value(args, "--share-root").map(PathBuf::from);
    let when_idle = match value(args, "--when-idle") {
        None => None,
        Some(v) => match v.trim().parse::<f64>() {
            Ok(m) if (0.0..=1440.0).contains(&m) => Some(Duration::from_secs_f64(m * 60.0)),
            _ => return fail(format!("--when-idle expects minutes, 0 to 1440 (got \"{v}\")")),
        },
    };
    if when_idle.is_some() && !cfg!(windows) {
        return fail("--when-idle needs Windows: on this system the app cannot tell when someone is using the machine".into());
    }
    let slots = match adapter_slots(args) {
        Ok(s) => s,
        Err(e) => return fail(e),
    };
    let multi = slots.len() > 1;
    let (key, id) = (Arc::new(key), Arc::new(id));
    let mut cfgs = Vec::new();
    for (i, adapter) in slots.into_iter().enumerate() {
        // One session per graphics card: each its own name at the controller (which pins and
        // schedules by name) and its own work folder; the controls are the machine's.
        let slot = multi.then_some(i as u32 + 1);
        let name = slot.map_or_else(|| name.clone(), |k| format!("{name} · GPU {k}"));
        if let Err(e) = fractadyne_farm::names::check_display_name(&name) {
            return fail(e);
        }
        let work = slot.map_or_else(|| dir.join("jobs"), |k| dir.join(format!("jobs-gpu{k}")));
        cfgs.push(Cfg { addr: addr.clone(), key: key.clone(), id: id.clone(), name, policy: policy.clone(), allow_dirty, one_job, share_root: share_root.clone(), work, control: dir.clone(), adapter, when_idle, slot });
    }
    let ctl = Control::new(args.iter().any(|a| a == status::FLAG));
    let names = cfgs.iter().map(|c| format!("\"{}\"", c.name)).collect::<Vec<_>>().join(", ");
    println!("Render client {names} (identity {}) — controller {addr}", id.fingerprint());
    for c in &cfgs {
        if let Some((n, words)) = &c.adapter {
            println!("  {}: graphics adapter {n}, {words}", c.name);
        }
    }
    if let Some(w) = when_idle {
        println!("  only when idle: takes work once nobody has used this machine for {}", minutes_text(w));
    }
    println!("  pause: create {}  ·  cancel the frame in progress: create {}", dir.join("PAUSE").display(), dir.join("CANCEL").display());
    if !multi {
        return cfgs.first().map_or(2, |cfg| serve(cfg, &pins, Ui::new(ctl, None)));
    }
    let codes: Vec<i32> = std::thread::scope(|sc| {
        let pins = &pins;
        let running: Vec<_> = cfgs
            .iter()
            .map(|cfg| {
                let ui = Ui::new(ctl.clone(), cfg.slot);
                sc.spawn(move || {
                    TAG.with(|t| *t.borrow_mut() = format!("GPU {}", cfg.slot.unwrap_or(0)));
                    serve(cfg, pins, ui)
                })
            })
            .collect();
        running.into_iter().map(|h| h.join().unwrap_or(1)).collect()
    });
    codes.into_iter().find(|&c| c != 0).unwrap_or(0)
}

/// One session: connect, serve the controller, redial with back-off when the connection drops;
/// the exit code once it ends (left, refused, removed, or `--one-job` done).
fn serve(cfg: &Cfg, pins: &Mutex<PinStore>, ui: Arc<Ui>) -> i32 {
    ui.update(|s| {
        s.name = cfg.name.clone();
        s.identity = cfg.id.fingerprint();
        s.controller = cfg.addr.clone();
        s.detail = format!("Connecting to {}…", cfg.addr);
    });
    let mut backoff = Duration::from_secs(2);
    loop {
        match connect_once(cfg, pins, &ui) {
            Ended::Exit(code) => {
                ui.end(code, "");
                return code;
            }
            Ended::Retry { connected, why } => {
                if connected {
                    backoff = Duration::from_secs(2);
                }
                say!("{why} — retrying in {}s", backoff.as_secs());
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

fn connect_once(cfg: &Cfg, pins: &Mutex<PinStore>, ui: &Arc<Ui>) -> Ended {
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
    let pin = pins.lock().unwrap_or_else(|e| e.into_inner()).check(&cfg.addr, &fp);
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
                        match pins.lock().unwrap_or_else(|e| e.into_inner()).pin(&cfg.addr, &fp) {
                            Ok(()) => say!("Pinned the controller's identity {fp}"),
                            Err(e) => eprintln!("fractadyne: could not save the controller's identity: {e}"),
                        }
                    }
                    say!("Connected to \"{}\" ({fp}) — running the self-check", a.name);
                    ui.update(|s| {
                        s.phase = ClientPhase::Checking;
                        s.controller_name = a.name.clone();
                        s.controller_fingerprint = fp.clone();
                        s.detail = format!("Connected to \"{}\" — running the self-check", a.name);
                    });
                    break (a.link_sample_bytes, a.gpu.clone());
                }
                Verdict::WaitingForApproval => {
                    say!("Connected to \"{}\" — waiting for its user to approve this machine", a.name);
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
        say!("⚠ This machine has {n}. Its frames will differ slightly from the controller's.");
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
        pending_orbits: HashMap::new(),
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
        in_use: false,
        first_frame_at: None,
        started: Instant::now(),
    };
    // Before any work can arrive: a machine in use takes none.
    s.idle_policy();
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
    let result = render_probe(&cfg.work.join("self-check"), cfg.adapter.as_ref().map(|(n, _)| n.as_str()));
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
        say!("  self-check {} {}: {}", if i.ok { "ok  " } else { "FAIL" }, i.name, i.detail);
    }
    (SelfCheck { items, gpu, free_bytes: free, link_sample: announce, probe: probe_announce }, sample, probe, cap, record)
}

/// `--when-idle`: is someone using this machine now? Under the test instrument
/// ([`IN_USE_INSTRUMENT`] = N), someone sits down as the session's first frame starts and stays N
/// seconds, and before that the machine was last touched as the session began.
fn in_use_now(need: Duration, started: Instant, first_frame_at: Option<Instant>) -> bool {
    let n = crate::tunables::instrument(IN_USE_INSTRUMENT);
    if n > 0 {
        let now = Instant::now();
        let stay = Duration::from_secs(u64::from(n));
        let last_input = match first_frame_at {
            Some(t) if now < t + stay => now,
            Some(t) => t + stay,
            None => started,
        };
        return in_use(need, Some(now - last_input), false);
    }
    in_use(need, crate::sysinfo::user_idle(), crate::sysinfo::screen_locked())
}

/// The window's phase and words while no frame renders: paused by this machine's user, waiting for
/// the machine to be idle (`--when-idle`, with how long it waits for), or idle.
fn waiting(paused: bool, in_use: Option<Duration>, controller: &str) -> (ClientPhase, String) {
    match (paused, in_use) {
        (true, _) => (ClientPhase::Paused, "Paused — the controller has been told".into()),
        (false, Some(need)) => (ClientPhase::Paused, format!("Waiting — someone is using this machine; work resumes once nobody has for {}", minutes_text(need))),
        (false, None) => (ClientPhase::Idle, format!("Connected to \"{controller}\" — idle")),
    }
}

fn minutes_text(d: Duration) -> String {
    match d.as_secs() {
        s if s < 120 => format!("{s} s"),
        s => format!("{} min", s / 60),
    }
}

/// The sessions `--adapter` / `--adapters` ask for: each its graphics card's `--list-adapters`
/// number (what its renders are given) and name, or one session on wgpu's choice without either.
/// `--adapters all` is every graphics card once: its Vulkan entry (the app does not render on GL).
fn adapter_slots(args: &[String]) -> Result<Vec<Option<(String, String)>>, String> {
    use crate::gpu_choice;
    let one = gpu_choice::spec(args)?;
    let many = value(args, "--adapters");
    if one.is_some() && many.is_some() {
        return Err("give --adapter (one graphics card) or --adapters (several), not both".into());
    }
    if one.is_none() && many.is_none() {
        return Ok(vec![None]);
    }
    let rows = gpu_choice::list();
    let pairs: Vec<(String, eframe::wgpu::Backend)> = rows.iter().map(|r| (r.name.clone(), r.backend)).collect();
    let listed = |e: String| format!("{e}\nThis machine's adapters:\n{}", gpu_choice::listing(&rows));
    let picks: Vec<usize> = match (one, many) {
        (Some(s), _) => vec![gpu_choice::pick(&s, &pairs).map_err(listed)?],
        (None, Some(l)) if l.trim().eq_ignore_ascii_case("all") => {
            rows.iter().enumerate().filter(|(_, r)| r.is_hardware() && r.backend == eframe::wgpu::Backend::Vulkan).map(|(i, _)| i).collect()
        }
        (None, Some(l)) => l.split(',').map(|s| gpu_choice::pick(s, &pairs)).collect::<Result<_, _>>().map_err(listed)?,
        (None, None) => Vec::new(),
    };
    if picks.is_empty() {
        return Err(listed("--adapters all: no graphics card found".into()));
    }
    Ok(picks.into_iter().map(|i| Some(((i + 1).to_string(), format!("{} · {:?}", rows[i].name, rows[i].backend)))).collect())
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
    /// Orbit-cache files (name → size) already offered to the controller or received from it.
    orbits: HashMap<String, u64>,
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
    /// Reference orbits on their way from the controller, by blob id.
    pending_orbits: HashMap<u64, (String, BlobSink<Vec<u8>>)>,
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
    /// `--when-idle`: someone is using this machine, so it takes no work.
    in_use: bool,
    /// When this session's first frame started (the in-use instrument keys on it).
    first_frame_at: Option<Instant>,
    started: Instant,
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
                say!("Job \"{}\" ({}) — receiving its bundle", j.name, j.job_id);
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
                } else if self.paused || self.in_use {
                    // Told, the scheduler parks this machine until its heartbeat says it is back.
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
                    say!("  run {run} cut at frame {from}: another machine renders the rest");
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
                    say!("  run cancelled by the controller ({:?})", c.reason);
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
                say!("Job {} closed by the controller", c.job_id);
                self.ui.update(|s| {
                    s.detail = "The job is over".into();
                    s.run = None;
                    s.frame = None;
                    if s.phase == ClientPhase::Rendering {
                        s.phase = ClientPhase::Idle;
                    }
                });
                if self.cfg.one_job {
                    say!("--one-job: the job is over; exiting");
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
            Msg::OrbitPush(o) => {
                if self.jobs.contains_key(&o.job_id) && !self.pending_orbits.contains_key(&o.blob.id) {
                    let sink = BlobSink::new(o.blob.clone(), Vec::new());
                    self.pending_orbits.insert(o.blob.id, (o.job_id, sink));
                }
            }
            other => {
                return Some(Ended::Retry { connected: true, why: format!("the controller sent a {} message, which only a client sends", kind_name(&other)) });
            }
        }
        None
    }

    fn on_chunk(&mut self, id: u64, offset: u64, data: &[u8]) -> Result<(), String> {
        if let Some((_, sink)) = self.pending_orbits.get_mut(&id) {
            if sink.push(offset, data).map_err(|e| e.to_string())? == BlobProgress::More {
                return Ok(());
            }
            if let Some((job_id, sink)) = self.pending_orbits.remove(&id) {
                self.orbit_arrived(&job_id, sink.into_inner());
            }
            return Ok(());
        }
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
            say!("  job refused by this machine's policy: {r}");
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
                    say!("  frames go to the shared drive: {}", d.display());
                    frames = d;
                    share = true;
                }
                Err(e) => say!("  the shared drive is not writable from here ({e}); streaming the frames instead"),
            }
        }
        let detail = format!("{}×{} ss{} at {} fps, {} frames", bundle.width, bundle.height, bundle.ss, bundle.fps, bundle.frames);
        say!("  job \"{}\": {detail}", bundle.name);
        self.ui.update(|s| {
            s.job_detail = Some(detail);
            s.detail = match &refusal {
                Some(r) => format!("Refused this job: {r}"),
                None => format!("Job \"{}\" — waiting for frames to render", bundle.name),
            };
        });
        self.jobs.insert(j.job_id.clone(), Job { dir, bundle, refusal, frames, share, orbits: HashMap::new() });
        Ok(())
    }

    fn maybe_start(&mut self) {
        if self.running.is_some() || self.paused || self.in_use {
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
        // Shared computation: the render keeps (and finds) reference orbits in the job's own
        // cache, which the farm's shared orbits are dropped into (`orbit_arrived`).
        if b.sharing {
            args.push("--orbit-cache".into());
        }
        if let Some((n, _)) = &self.cfg.adapter {
            args.push(crate::gpu_choice::FLAG.into());
            args.push(n.clone());
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
                say!("  rendering frames {}..{} (run {})", a.start, a.end, a.run_id);
                self.ui.update(|s| {
                    s.phase = ClientPhase::Rendering;
                    s.run = Some((a.start, a.end));
                    s.frame = Some(a.start);
                    s.detail = format!("Rendering frames {}–{}", a.start, a.end - 1);
                });
                self.first_frame_at.get_or_insert(Instant::now());
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
            ChildLine::Done { index, bytes, sha256, ms, reference } if self.jobs.get(&r.assign.job_id).is_some_and(|j| j.share) => {
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
                let _ = self.out.send(Out::Msg(Msg::FrameDone(FrameDone { job_id: r.assign.job_id.clone(), run_id: run, index, render_ms: ms, blob: BlobAnnounce { id, len: bytes, sha256 }, on_share: true, reference })));
                r.reported.insert(index);
                r.last_done = Some(r.last_done.map_or(index, |d| d.max(index)));
                r.frame_started = Instant::now();
                self.frames_done += 1;
                self.ms_total += ms;
                say!("  frame {index} on the shared drive ({} KB, {ms} ms)", bytes / 1024);
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
            ChildLine::Done { index, bytes, sha256, ms, reference } => {
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
                        let _ = self.out.send(Out::Msg(Msg::FrameDone(FrameDone { job_id: r.assign.job_id.clone(), run_id: run, index, render_ms: ms, blob: announce, on_share: false, reference })));
                        let _ = self.out.send(Out::Blob(id, data));
                        // Kept as the window's thumbnail of the latest frame (the next replaces it).
                        let last = self.cfg.control.join(self.cfg.slot.map_or_else(|| "last-frame.png".into(), |k| format!("last-frame-gpu{k}.png")));
                        if std::fs::rename(&path, &last).is_err() {
                            let _ = std::fs::remove_file(&path);
                        }
                        r.reported.insert(index);
                        r.last_done = Some(r.last_done.map_or(index, |d| d.max(index)));
                        r.frame_started = Instant::now();
                        self.frames_done += 1;
                        self.ms_total += ms;
                        say!("  frame {index} sent ({} KB, {ms} ms)", bytes / 1024);
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

    /// Offer the controller the reference orbits this job's renders cached that it has not seen.
    fn offer_orbits(&mut self, job_id: &str) {
        let Some(job) = self.jobs.get_mut(job_id) else { return };
        if !job.bundle.sharing {
            return;
        }
        let dir = job.dir.join("cfg").join(crate::refcache_persist::DIR_NAME);
        let Ok(rd) = std::fs::read_dir(&dir) else { return };
        let mut offers = Vec::new();
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            let Some(len) = e.metadata().ok().map(|m| m.len()) else { continue };
            if !name.ends_with(&format!(".{}", crate::refcache_persist::ENTRY_EXT)) || job.orbits.get(&name) == Some(&len) || len > MAX_ORBIT_BYTES {
                continue;
            }
            job.orbits.insert(name.clone(), len);
            if let Ok(bytes) = std::fs::read(e.path()) {
                if orbit_file_name(&bytes).is_some_and(|(n, _)| n == name) {
                    offers.push(bytes);
                }
            }
        }
        for bytes in offers {
            let id = self.next_blob;
            self.next_blob += 1;
            say!("  offering the farm a reference orbit this machine built ({} MB)", bytes.len() / 1_000_000);
            let blob = BlobAnnounce { id, len: bytes.len() as u64, sha256: fractadyne_farm::sha256_hex(&bytes) };
            let _ = self.out.send(Out::Msg(Msg::OrbitOffer(OrbitBlob { job_id: job_id.to_string(), blob })));
            let _ = self.out.send(Out::Blob(id, bytes));
        }
    }

    /// A reference orbit from the farm: verified, then put where this job's renders look for one
    /// (unless a longer one of the same identity is already there).
    fn orbit_arrived(&mut self, job_id: &str, bytes: Vec<u8>) {
        let Some(job) = self.jobs.get_mut(job_id) else { return };
        let Some((name, h)) = orbit_file_name(&bytes) else {
            say!("  a reference orbit from the controller did not verify; ignored");
            return;
        };
        let dir = job.dir.join("cfg").join(crate::refcache_persist::DIR_NAME);
        let dest = dir.join(&name);
        if std::fs::metadata(&dest).is_ok_and(|m| m.len() >= bytes.len() as u64) {
            return;
        }
        let tmp = dir.join(format!("{name}.tmp"));
        let ok = std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(&tmp, &bytes)).and_then(|()| std::fs::rename(&tmp, &dest));
        if ok.is_ok() {
            job.orbits.insert(name, bytes.len() as u64);
            say!("  received a reference orbit from the farm: {} iterations at {} bits ({} MB)", h.orbit_len, h.prec, bytes.len() / 1_000_000);
        }
    }

    fn on_child_closed(&mut self, run: u64) {
        let Some(mut r) = self.running.take_if(|r| r.assign.run_id == run) else { return };
        let code = r.child.wait().ok().and_then(|s| s.code());
        let end = r.stop_from.map_or(r.assign.end, |f| f.min(r.assign.end));
        let complete = (r.assign.start..end).all(|i| r.reported.contains(&i));
        // Judged by its FRAMES: a render that reported every one of them is complete, whatever
        // its exit code after (a log check, a teardown) — the controller verifies each frame
        // itself. Counting those as crashes parked a healthy machine as unstable (2026-10-04).
        let reason = match (r.stop.take(), code) {
            (Some(why), _) => Some(why),
            (None, Some(0)) if complete => None,
            (None, c) if complete => {
                say!("  run {run}: every frame arrived; the render then exited {c:?} — counted as complete");
                None
            }
            (None, Some(c)) => Some(AbortReason::ChildCrash(c)),
            (None, None) => Some(AbortReason::ChildCrash(-1)),
        };
        if let Some(why) = reason {
            if !matches!(why, AbortReason::Canceled) || !complete {
                say!("  run {} stopped: {why:?}", run);
            }
            self.send(Msg::RunAborted(RunAborted { job_id: r.assign.job_id.clone(), run_id: run, done_up_to: r.last_done, reason: why }));
        }
        self.offer_orbits(&r.assign.job_id.clone());
        let (paused, busy) = (self.paused, self.cfg.when_idle.filter(|_| self.in_use));
        self.ui.update(|s| {
            s.run = None;
            s.frame = None;
            (s.phase, s.detail) = waiting(paused, busy, &s.controller_name);
        });
        self.maybe_start();
    }

    /// `--when-idle`: someone using this machine stops the frame in progress — it goes back to the
    /// farm with no strike against this machine (`UserCancel`) — and no work is taken until nobody
    /// has used it for the set time. A locked screen counts as nobody.
    fn idle_policy(&mut self) {
        let Some(need) = self.cfg.when_idle else { return };
        let busy = in_use_now(need, self.started, self.first_frame_at);
        if busy == self.in_use {
            return;
        }
        self.in_use = busy;
        let running = self.running.is_some();
        if busy && running {
            say!("Someone is using this machine — stopped the frame in progress; it goes back to the farm");
            self.stop_child(Some(AbortReason::UserCancel));
        } else if busy {
            say!("Someone is using this machine — no work until nobody has used it for {}", minutes_text(need));
            // A run waiting here goes back too: told, the scheduler parks this machine and takes
            // back the rest of its runs.
            if let Some(a) = self.queue.pop_front() {
                self.send(Msg::RunAborted(RunAborted { job_id: a.job_id, run_id: a.run_id, done_up_to: None, reason: AbortReason::UserCancel }));
            }
        } else {
            say!("Nobody has used this machine for {} — taking work", minutes_text(need));
        }
        let paused = self.paused;
        self.ui.update(|s| {
            s.in_use = busy;
            // A stopped frame's run reports its own end (`on_child_closed`).
            if !running {
                (s.phase, s.detail) = waiting(paused, busy.then_some(need), &s.controller_name);
            }
        });
        if !busy {
            self.maybe_start();
        }
    }

    fn on_tick(&mut self) -> Option<Ended> {
        // The window's commands, and the headless control files.
        self.ui.poll();
        if self.ui.leaving() {
            say!("Disconnecting (its user left)");
            self.stop_child(Some(AbortReason::UserCancel));
            self.send(Msg::Bye(Bye { reason: "its user disconnected".into() }));
            std::thread::sleep(Duration::from_millis(300)); // let the goodbye go out
            self.ui.end(0, "Disconnected");
            return Some(Ended::Exit(0));
        }
        // A CANCEL file: the session that takes it tells them all (`Control::cancels`).
        let cancel = self.cfg.control.join("CANCEL");
        if cancel.exists() && std::fs::remove_file(&cancel).is_ok() {
            let _ = std::fs::write(self.cfg.control.join("PAUSE"), b"created by CANCEL; delete to resume\n");
            self.ui.ctl.cancels.fetch_add(1, Ordering::Relaxed);
        }
        let pause = self.cfg.control.join("PAUSE").exists() || self.ui.paused();
        if self.ui.take_cancel() {
            if self.running.is_some() {
                say!("Cancelled the frame in progress; paused");
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
                say!("Paused: finishing the current frame, then taking no work");
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
                say!("Resumed");
                let busy = self.cfg.when_idle.filter(|_| self.in_use);
                self.ui.update(|s| {
                    s.paused = false;
                    if s.phase == ClientPhase::Paused {
                        (s.phase, s.detail) = waiting(false, busy, &s.controller_name);
                    }
                });
                self.maybe_start();
            }
        }
        self.idle_policy();
        if let Some(r) = &self.running {
            let ms = r.frame_started.elapsed().as_millis() as u64;
            self.ui.update(|s| s.frame_ms = Some(ms));
        }
        if self.last_heartbeat.elapsed() >= Duration::from_secs(2) {
            self.last_heartbeat = Instant::now();
            let hb = Heartbeat {
                activity: if self.paused || self.in_use {
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
