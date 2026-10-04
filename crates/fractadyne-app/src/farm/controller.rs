//! `--farm-render TOUR --out DIR`: the controller (design §11).
//!
//! It listens (clients dial in), admits a client only after the farm-key handshake, the version
//! gate and the client's self-check, waits for `--min-clients`, then measures what must be measured
//! once for every machine (the normalize anchors, at the smallest reference-orbit cap among them),
//! and drives the scheduler until every frame is verified or given up.
//!
//! Every frame is received into a private incoming file and checked — SHA-256 against what the
//! client announced, PNG structure, and the job's dimensions — before the scheduler hears of it. A
//! frame that fails is moved to `farm/bad/` under its sender's name and counts a strike; at
//! `--strikes` the sender is removed, its diagnostics fetched into `farm/diag/`, and it is told why.
//! A verified frame is renamed into place only when the scheduler accepts it as the first copy.
//!
//! One thread per connection reads (and receives frames), one writes; the main loop owns the
//! scheduler and the job state, so no lock guards either.

use super::*;
use fractadyne_farm::channel::{self, Pin, PinStore, RateLimiter, RecvError};
use fractadyne_farm::manifest::{DoneRecord, JobIdentity, Manifest};
use fractadyne_farm::proto::*;
use fractadyne_farm::sched::{self, ClientId, Command, Scheduler};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

/// Bytes the link-speed sample asks each client for at the handshake.
const LINK_SAMPLE: u64 = 1 << 20;

enum COut {
    Msg(Msg),
    Blob(u64, Arc<Vec<u8>>),
    Close,
}

enum CEv {
    Hello { conn: ClientId, hello: Hello, fingerprint: String, addr: String, out: mpsc::Sender<COut>, io: Arc<Io> },
    Msg { conn: ClientId, msg: Msg },
    LinkSample { conn: ClientId, bytes: u64, secs: f64 },
    FrameIn { conn: ClientId, run: u64, index: u64, render_ms: u64, tmp: PathBuf, bytes: u64, sha256: String },
    FrameBad { conn: ClientId, run: u64, index: u64, why: String, tmp: Option<PathBuf> },
    Closed { conn: ClientId, why: String },
    Note(String),
}

/// Bytes moved on one connection, for the link metrics.
#[derive(Default)]
struct Io {
    bytes_in: AtomicU64,
    bytes_out: AtomicU64,
}

struct Conn {
    name: String,
    addr: String,
    out: mpsc::Sender<COut>,
    state: ConnState,
    gpu_cap: Option<u64>,
    adapter: String,
    job_sent: bool,
    last_hb: Option<Heartbeat>,
    link_mbps: Option<f64>,
    self_check: String,
    diag: Option<(String, u32, Instant)>,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum ConnState {
    /// Hello accepted; waiting for the self-check (and its link sample).
    Checking,
    /// Passed; part of the farm.
    Admitted,
    /// Leaving: removed (diagnostics pending) or refused.
    Closing,
}

struct Job {
    manifest: Manifest,
    sched: Scheduler,
    bundle: Arc<Vec<u8>>,
    bundle_sha: String,
    job_id: String,
    orbit_len_cap: Option<u64>,
    frame_bytes_ewma: Option<f64>,
}

pub(crate) fn run(args: &[String]) -> i32 {
    match run_inner(args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("fractadyne: farm: {e}");
            2
        }
    }
}

fn run_inner(args: &[String]) -> Result<i32, String> {
    let tour = PathBuf::from(value(args, "--farm-render").ok_or("--farm-render needs a tour script")?);
    let out = PathBuf::from(value(args, "--out").or_else(|| value(args, "-o")).ok_or("--farm-render needs --out DIR for the frames")?);
    let listen = value(args, "--listen").map(str::to_string).unwrap_or_else(|| format!("0.0.0.0:{}", fractadyne_farm::DEFAULT_PORT));
    let min_clients: usize = number(args, "--min-clients").unwrap_or(1usize).max(1);
    let allow_dirty = args.iter().any(|a| a == "--farm-allow-dirty");
    let name = machine_name(args);
    let (ver, git) = build_identity();
    if is_dirty(git) && !allow_dirty {
        return Err(format!("this build ({git}) has uncommitted changes, and a farm refuses one — start with --farm-allow-dirty only to develop the farm itself"));
    }

    // The tour, resolved exactly as `--render-tour` resolves it.
    let script_text = std::fs::read_to_string(&tour).map_err(|e| format!("{}: {e}", tour.display()))?;
    let pb = crate::scripting::parse_tour_text(&script_text).map_err(|e| format!("{}: {e}", tour.display()))?;
    let (sw, sh) = value(args, "--size").map(|s| crate::arg_size("--size", s)).unwrap_or((None, None));
    let cli = crate::scripting::TourRenderConfig {
        fps: number(args, "--fps"),
        width: sw,
        height: number(args, "--height").or(sh),
        ss: number(args, "--ss"),
        prefix: value(args, "--prefix").map(str::to_string),
        out: Some(out.clone()),
        ..Default::default()
    };
    let r = cli.resolve(&pb.render, &tour);
    fractadyne_farm::names::check_file_part("frame prefix", &r.prefix)?;
    let frames = crate::scripting::tour_frame_count(pb.total, r.fps);

    // This machine's render settings: what every client's render processes will use.
    let (session, _) = fractadyne_state::load_with_status();
    let settings = RenderSettings::from_session(&session);
    settings.validate().map_err(|e| format!("this machine's render settings: {e}"))?;

    let (key, key_path, created) = load_key(args, true)?;
    let me = Arc::new(identity()?);
    let mut pins = PinStore::load(&farm_dir()?.join("known-clients.toml"))?;

    let listener = std::net::TcpListener::bind(&listen).map_err(|e| format!("cannot listen on {listen}: {e}"))?;
    let local_addr = listener.local_addr().map_err(|e| e.to_string())?;
    println!("Render farm controller \"{name}\" (identity {}) — {ver} {git}", me.fingerprint());
    println!("Tour \"{}\": {frames} frames at {}×{} ss{} {} fps → {}", pb.name, r.width, r.height, r.ss, r.fps, out.display());
    if created {
        println!("Generated a farm key — give it to each render client (saved in {}):\n  {}", key_path.display(), key.to_text());
    } else {
        println!("Farm key: {}", key_path.display());
    }
    println!("listening on {local_addr}");
    println!("  on each render client:  fractadyne --render-client <this machine's address>:{} --farm-key-file <its copy of the key>", local_addr.port());
    std::fs::create_dir_all(out.join("farm").join("incoming")).map_err(|e| format!("{}: {e}", out.display()))?;

    let (ev_tx, ev_rx) = mpsc::channel::<CEv>();
    let key = Arc::new(key);
    spawn_listener(listener, key.clone(), me.clone(), ev_tx.clone(), (r.width, r.height), out.join("farm").join("incoming"));

    let mut local_child = None;
    if args.iter().any(|a| a == "--local") {
        let mut a: Vec<String> = vec!["--render-client".into(), format!("127.0.0.1:{}", local_addr.port()), "--farm-key-file".into(), key_path.to_string_lossy().into_owned(), "--name".into(), format!("{name} (local)")];
        if allow_dirty {
            a.push("--farm-allow-dirty".into());
        }
        let mut c = spawn_child(&a, &out.join("farm").join("local-client"))?;
        let tx = ev_tx.clone();
        pump_child(&mut c, move |l| { let _ = tx.send(CEv::Note(format!("[local] {l}"))); }, Arc::new(std::sync::Mutex::new(VecDeque::new())), 50);
        local_child = Some(c);
        println!("Started a local render client on this machine");
    }

    let mut ctl = Controller {
        args: args.to_vec(),
        name,
        tour,
        script_text,
        tour_name: pb.name.clone(),
        normalize: pb.render.normalize,
        res: (r.width, r.height, r.fps, r.ss, r.prefix.clone(), frames),
        out,
        settings,
        allow_dirty,
        min_clients,
        conns: HashMap::new(),
        io: HashMap::new(),
        job: None,
        pins: &mut pins,
        started: Instant::now(),
        last_status: Instant::now(),
        last_metrics: Instant::now(),
        last_keepalive: Instant::now(),
        last_tick: Instant::now(),
        metrics_prev: (0, 0, 0, Instant::now()),
        storage_low: false,
    };
    let code = ctl.main_loop(&ev_rx);
    if let Some(mut c) = local_child {
        let _ = c.kill();
    }
    Ok(code)
}

/// The version gate (design §9, decided 2026-10-03: exact commit, `-dirty` refused): why a client
/// may not join, or `None`. Pure, so every refusal is pinned by test. Build sequence numbers are per
/// machine and deliberately not compared. `--farm-allow-dirty` on BOTH ends admits a dirty build and
/// an instrumented one (the `--farmtest` faults) — never changed tunables, which change pictures.
pub(crate) fn admission_refusal(h: &Hello, ver: &str, git: &str, allow_dirty: bool) -> Option<String> {
    let dev = allow_dirty && h.allow_dirty;
    if h.protocol != fractadyne_farm::PROTOCOL_VERSION {
        Some(format!("this farm speaks protocol {}; this client speaks {}", fractadyne_farm::PROTOCOL_VERSION, h.protocol))
    } else if h.app_version != ver || h.git != git {
        Some(format!("version mismatch: this farm runs {ver} {git}; this client is {} {} — install the same build", h.app_version, h.git))
    } else if is_dirty(&h.git) && !dev {
        Some("a build with uncommitted changes (-dirty) cannot join a farm".into())
    } else if h.tunables != "stock" && !(dev && !h.tunables.contains("OVERRIDE")) {
        Some(format!("this client runs with changed tunables ({}) — a farm needs stock builds", h.tunables))
    } else {
        None
    }
}

fn spawn_listener(listener: std::net::TcpListener, key: Arc<FarmKey>, me: Arc<Identity>, ev: mpsc::Sender<CEv>, dims: (u32, u32), incoming: PathBuf) {
    std::thread::spawn(move || {
        // Counts FAILED handshakes only (see `RateLimiter`): shared with the connection threads,
        // which report a failure; this thread refuses an address while it cools down.
        let limiter = Arc::new(std::sync::Mutex::new(RateLimiter::default()));
        let mut next: ClientId = 1;
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            let ip = stream.peer_addr().map(|a| a.ip()).ok();
            if ip.is_some_and(|ip| !limiter.lock().unwrap_or_else(|e| e.into_inner()).allowed(ip, Instant::now())) {
                continue;
            }
            let id = next;
            next += 1;
            let (key, me, ev, incoming, limiter) = (key.clone(), me.clone(), ev.clone(), incoming.clone(), limiter.clone());
            std::thread::spawn(move || serve(id, stream, &key, &me, ev, dims, &incoming, &limiter));
        }
    });
}

/// One connection: the handshake, then every message it sends, with frames received to disk and
/// verified here, off the main loop.
#[allow(clippy::too_many_arguments)]
fn serve(conn: ClientId, stream: std::net::TcpStream, key: &FarmKey, me: &Identity, ev: mpsc::Sender<CEv>, dims: (u32, u32), incoming: &Path, limiter: &std::sync::Mutex<RateLimiter>) {
    let addr = stream.peer_addr().map(|a| a.to_string()).unwrap_or_default();
    let ip = stream.peer_addr().map(|a| a.ip()).ok();
    let sess = match channel::respond(stream, key, me) {
        Ok(s) => s,
        Err(e) => {
            let cooling = ip.is_some_and(|ip| limiter.lock().unwrap_or_else(|e| e.into_inner()).failed(ip, Instant::now()));
            let _ = ev.send(CEv::Note(format!(
                "handshake from {addr} refused: {e}{}",
                if cooling { " — too many failures from that address; refusing it for 60 s" } else { "" }
            )));
            return;
        }
    };
    let fingerprint = fractadyne_farm::key::fingerprint(&sess.remote_static);
    // 30 s of silence from a client that heartbeats every 2 s means it is gone.
    let Ok((mut r, w)) = sess.split(Some(Duration::from_secs(30)), Duration::from_secs(30)) else { return };
    let io = Arc::new(Io::default());
    let (out_tx, out_rx) = mpsc::channel::<COut>();
    {
        let io = io.clone();
        std::thread::spawn(move || {
            let mut w = w;
            for o in out_rx {
                let res = match o {
                    COut::Msg(m) => w.send(&m),
                    COut::Blob(id, b) => w.send_blob(id, &b),
                    COut::Close => break,
                };
                match res {
                    Ok(n) => {
                        io.bytes_out.fetch_add(n, Ordering::Relaxed);
                    }
                    Err(_) => break,
                }
            }
            w.shutdown();
        });
    }
    let hello = match r.recv() {
        Ok(Incoming::Control(Msg::Hello(h))) => h,
        other => {
            let _ = ev.send(CEv::Note(format!("{addr}: the first message was not a Hello ({other:?})")));
            let _ = out_tx.send(COut::Close);
            return;
        }
    };
    if ev.send(CEv::Hello { conn, hello, fingerprint, addr, out: out_tx.clone(), io: io.clone() }).is_err() {
        return;
    }
    enum Pending {
        Frame { f: FrameDone, sink: BlobSink<std::io::BufWriter<std::fs::File>>, path: PathBuf },
        Sample { sink: BlobSink<std::io::Sink>, started: Instant },
    }
    let mut pending: HashMap<u64, Pending> = HashMap::new();
    let close = |why: String| {
        let _ = ev.send(CEv::Closed { conn, why });
        let _ = out_tx.send(COut::Close);
    };
    loop {
        match r.recv() {
            Ok(Incoming::Control(m)) => {
                io.bytes_in.fetch_add(64, Ordering::Relaxed);
                match m {
                    Msg::FrameDone(f) => {
                        if pending.len() >= 4 || pending.contains_key(&f.blob.id) {
                            break close("protocol error: too many frames in flight".into());
                        }
                        let path = incoming.join(format!("{conn}-{}.png.part", f.blob.id));
                        match std::fs::File::create(&path) {
                            Ok(file) => {
                                let sink = BlobSink::new(f.blob.clone(), std::io::BufWriter::new(file));
                                pending.insert(f.blob.id, Pending::Frame { f, sink, path });
                            }
                            Err(e) => break close(format!("cannot store an incoming frame: {e}")),
                        }
                    }
                    Msg::SelfCheck(s) => {
                        if let Some(b) = &s.link_sample {
                            pending.insert(b.id, Pending::Sample { sink: BlobSink::new(b.clone(), std::io::sink()), started: Instant::now() });
                        }
                        let _ = ev.send(CEv::Msg { conn, msg: Msg::SelfCheck(s) });
                    }
                    m @ (Msg::Heartbeat(_) | Msg::FrameFailed(_) | Msg::RunAborted(_) | Msg::DiagReport(_) | Msg::Bye(_)) => {
                        if ev.send(CEv::Msg { conn, msg: m }).is_err() {
                            break;
                        }
                    }
                    other => break close(format!("protocol error: a client sent {}", kind_name(&other))),
                }
            }
            Ok(Incoming::Chunk { id, offset, data }) => {
                io.bytes_in.fetch_add(data.len() as u64 + 19, Ordering::Relaxed);
                let Some(p) = pending.get_mut(&id) else {
                    break close(format!("protocol error: a chunk of unannounced blob {id}"));
                };
                let res = match p {
                    Pending::Frame { sink, .. } => sink.push(offset, &data),
                    Pending::Sample { sink, .. } => sink.push(offset, &data),
                };
                match (res, pending.remove(&id).expect("present")) {
                    (Ok(BlobProgress::More), p) => {
                        pending.insert(id, p);
                    }
                    (Ok(BlobProgress::Complete), Pending::Sample { sink, started }) => {
                        let _ = ev.send(CEv::LinkSample { conn, bytes: sink.received(), secs: started.elapsed().as_secs_f64() });
                    }
                    (Ok(BlobProgress::Complete), Pending::Frame { f, sink, path }) => {
                        drop(sink.into_inner());
                        // Structure and dimensions: a PNG of THIS job's size, complete to its IEND.
                        match crate::scripting::png_frame_size(&path) {
                            Some(d) if d == dims => {
                                let _ = ev.send(CEv::FrameIn { conn, run: f.run_id, index: f.index, render_ms: f.render_ms, tmp: path, bytes: f.blob.len, sha256: f.blob.sha256 });
                            }
                            got => {
                                let why = match got {
                                    Some((w, h)) => format!("a {w}×{h} image where the job is {}×{}", dims.0, dims.1),
                                    None => "not a complete PNG".into(),
                                };
                                let _ = ev.send(CEv::FrameBad { conn, run: f.run_id, index: f.index, why, tmp: Some(path) });
                            }
                        }
                    }
                    (Err(BlobError::Digest(why)), Pending::Frame { f, sink, path }) => {
                        drop(sink.into_inner());
                        let _ = ev.send(CEv::FrameBad { conn, run: f.run_id, index: f.index, why, tmp: Some(path) });
                    }
                    (Err(e), Pending::Frame { sink, path, .. }) => {
                        drop(sink.into_inner());
                        let _ = std::fs::remove_file(&path);
                        break close(format!("protocol error: {e}"));
                    }
                    (Err(e), Pending::Sample { .. }) => break close(format!("protocol error in the link sample: {e}")),
                }
            }
            Err(RecvError::TimedOut) => break close("no message for 30 s".into()),
            Err(e) => break close(e.to_string()),
        }
    }
    for (_, p) in pending {
        if let Pending::Frame { sink, path, .. } = p {
            drop(sink.into_inner());
            let _ = std::fs::remove_file(path);
        }
    }
}

struct Controller<'a> {
    args: Vec<String>,
    name: String,
    tour: PathBuf,
    script_text: String,
    tour_name: String,
    normalize: bool,
    res: (u32, u32, f64, u32, String, u64),
    out: PathBuf,
    settings: RenderSettings,
    allow_dirty: bool,
    min_clients: usize,
    conns: HashMap<ClientId, Conn>,
    /// Every connection's byte counters, kept after it closes so the job's totals only grow (a
    /// rate computed over live connections alone dropped to zero whenever a machine left).
    io: HashMap<ClientId, (String, Arc<Io>)>,
    job: Option<Job>,
    pins: &'a mut PinStore,
    started: Instant,
    last_status: Instant,
    last_metrics: Instant,
    last_keepalive: Instant,
    last_tick: Instant,
    metrics_prev: (u64, u64, u64, Instant),
    storage_low: bool,
}

impl Controller<'_> {
    fn note(&self, s: &str) {
        println!("{s}");
        crate::diag::log_line("farm", s);
        if let Some(j) = &self.job {
            let _ = j.manifest.record_event(unix_ms(), s);
        }
    }

    fn send(&self, conn: ClientId, m: Msg) {
        if let Some(c) = self.conns.get(&conn) {
            let _ = c.out.send(COut::Msg(m));
        }
    }

    fn close(&mut self, conn: ClientId, reason: Option<String>) {
        if let Some(c) = self.conns.get_mut(&conn) {
            if let Some(r) = reason {
                let _ = c.out.send(COut::Msg(Msg::Bye(Bye { reason: r })));
            }
            let _ = c.out.send(COut::Close);
            c.state = ConnState::Closing;
        }
    }

    fn main_loop(&mut self, ev: &mpsc::Receiver<CEv>) -> i32 {
        loop {
            match ev.recv_timeout(Duration::from_millis(250)) {
                Ok(e) => {
                    if let Some(code) = self.on_event(e) {
                        return code;
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => return 2,
            }
            if let Some(code) = self.periodic() {
                return code;
            }
        }
    }

    fn on_event(&mut self, e: CEv) -> Option<i32> {
        match e {
            CEv::Note(s) => self.note(&s),
            CEv::Hello { conn, hello, fingerprint, addr, out, io } => self.on_hello(conn, hello, fingerprint, addr, out, io),
            CEv::LinkSample { conn, bytes, secs } => {
                if let Some(c) = self.conns.get_mut(&conn) {
                    c.link_mbps = Some(bytes as f64 * 8.0 / secs.max(1e-6) / 1e6);
                }
                self.try_admit(conn);
            }
            CEv::Msg { conn, msg } => return self.on_msg(conn, msg),
            CEv::FrameIn { conn, run, index, render_ms, tmp, bytes, sha256 } => {
                let name = self.conns.get(&conn).map(|c| c.name.clone()).unwrap_or_default();
                let Some(j) = self.job.as_mut() else {
                    let _ = std::fs::remove_file(&tmp);
                    return None;
                };
                let cmds = j.sched.step(Instant::now(), sched::Event::FrameVerified { client: conn, run, index, render_ms });
                for c in &cmds {
                    match c {
                        Command::Accept { index: i } if *i == index => {
                            let dest = j.manifest.frame_path(index);
                            let moved = std::fs::rename(&tmp, &dest);
                            if let Err(e) = moved {
                                eprintln!("fractadyne: frame {index}: could not move it into place: {e}");
                            } else {
                                let _ = j.manifest.record_done(&DoneRecord { index, bytes, sha256: sha256.clone(), machine: name.clone(), ms: render_ms, at_unix: unix_ms() / 1000 });
                                j.frame_bytes_ewma = Some(j.frame_bytes_ewma.map_or(bytes as f64, |e| 0.8 * e + 0.2 * bytes as f64));
                            }
                        }
                        Command::Discard { index: i } if *i == index => {
                            let _ = std::fs::remove_file(&tmp);
                        }
                        _ => {}
                    }
                }
                return self.exec(cmds);
            }
            CEv::FrameBad { conn, run, index, why, tmp } => {
                let name = self.conns.get(&conn).map(|c| c.name.clone()).unwrap_or_default();
                if let Some(j) = self.job.as_mut() {
                    if let Some(t) = tmp {
                        let q = j.manifest.quarantine_path(index, &name);
                        if let Some(d) = q.parent() {
                            let _ = std::fs::create_dir_all(d);
                        }
                        if std::fs::rename(&t, &q).is_err() {
                            let _ = std::fs::remove_file(&t);
                        }
                    }
                    let cmds = j.sched.step(Instant::now(), sched::Event::FrameBad { client: conn, run, index, why });
                    return self.exec(cmds);
                }
            }
            CEv::Closed { conn, why } => {
                if let Some(c) = self.conns.remove(&conn) {
                    if c.state == ConnState::Admitted {
                        self.note(&format!("{}: connection closed ({why})", c.name));
                        if let Some(j) = self.job.as_mut() {
                            let cmds = j.sched.step(Instant::now(), sched::Event::Left { client: conn, why: sched::LeaveKind::Left });
                            return self.exec(cmds);
                        }
                    } else if let Some((reason, strikes, _)) = c.diag.clone() {
                        // Closed before its diagnostics arrived: write what we have.
                        self.write_diag(&c, &reason, strikes, None);
                    }
                }
            }
        }
        None
    }

    fn on_hello(&mut self, conn: ClientId, h: Hello, fp: String, addr: String, out: mpsc::Sender<COut>, io: Arc<Io>) {
        let (ver, git) = build_identity();
        let refusal = if let Some(r) = admission_refusal(&h, ver, git, self.allow_dirty) {
            Some(r)
        } else {
            match self.pins.check(&h.name, &fp) {
                Pin::Changed { pinned } => Some(format!("\"{}\" changed identity (pinned {pinned}, now {fp}); remove its line from known-clients.toml at the controller if it was reinstalled", h.name)),
                Pin::New => {
                    let _ = self.pins.pin(&h.name, &fp);
                    None
                }
                Pin::Known => None,
            }
        };
        let ack = |verdict| {
            Msg::HelloAck(HelloAck {
                protocol: fractadyne_farm::PROTOCOL_VERSION,
                app_version: ver.into(),
                git: git.into(),
                name: self.name.clone(),
                verdict,
                link_sample_bytes: LINK_SAMPLE,
            })
        };
        if h.tunables != "stock" && refusal.is_none() {
            self.note(&format!("⚠ {} runs instrumented ({}) — allowed only because both ends run with --farm-allow-dirty", h.name, h.tunables));
        }
        match refusal {
            Some(r) => {
                self.note(&format!("refused {} ({addr}): {r}", h.name));
                let _ = out.send(COut::Msg(ack(Verdict::Refused(r))));
                let _ = out.send(COut::Close);
            }
            None => {
                self.note(&format!("{} ({addr}, {fp}) connected — checking", h.name));
                self.io.insert(conn, (h.name.clone(), io.clone()));
                let _ = out.send(COut::Msg(ack(Verdict::Admitted)));
                self.conns.insert(
                    conn,
                    Conn {
                        name: h.name,
                        addr,
                        out,
                        state: ConnState::Checking,
                        gpu_cap: None,
                        adapter: String::new(),
                        job_sent: false,
                        last_hb: None,
                        link_mbps: None,
                        self_check: String::new(),
                        diag: None,
                    },
                );
            }
        }
    }

    fn on_msg(&mut self, conn: ClientId, m: Msg) -> Option<i32> {
        match m {
            Msg::SelfCheck(s) => {
                let c = self.conns.get_mut(&conn)?;
                c.self_check = s.items.iter().map(|i| format!("{} {}: {}", if i.ok { "ok  " } else { "FAIL" }, i.name, i.detail)).collect::<Vec<_>>().join("; ");
                c.gpu_cap = s.gpu.as_ref().map(|g| g.orbit_len_cap).filter(|v| *v > 0);
                c.adapter = s.gpu.as_ref().map(|g| g.adapter.clone()).unwrap_or_default();
                let name = c.name.clone();
                let summary = c.self_check.clone();
                if let Some(f) = s.items.iter().find(|i| i.hard && !i.ok) {
                    let why = format!("self-check failed — {}: {}", f.name, f.detail);
                    self.note(&format!("refused {name}: {why}"));
                    self.close(conn, Some(why));
                    return None;
                }
                self.note(&format!("{name} self-check: {summary}"));
                if s.link_sample.is_none() {
                    self.try_admit(conn);
                }
            }
            Msg::Heartbeat(h) => {
                if let Some(c) = self.conns.get_mut(&conn) {
                    c.last_hb = Some(h.clone());
                }
                if let Some(j) = self.job.as_mut() {
                    let cmds = j.sched.step(Instant::now(), sched::Event::Heartbeat { client: conn, paused: h.activity == ClientActivity::Paused, frame: h.frame, frame_ms: h.frame_ms });
                    return self.exec(cmds);
                }
            }
            Msg::FrameFailed(f) => {
                if let Some(j) = self.job.as_mut() {
                    let cmds = j.sched.step(Instant::now(), sched::Event::FrameFailed { client: conn, run: f.run_id, index: f.index, why: f.message });
                    return self.exec(cmds);
                }
            }
            Msg::RunAborted(r) => {
                let why = match r.reason {
                    AbortReason::UserCancel | AbortReason::Paused => sched::AbortKind::UserPaused,
                    AbortReason::Canceled => sched::AbortKind::Canceled,
                    AbortReason::DeviceLost | AbortReason::ChildCrash(_) => sched::AbortKind::Crashed,
                    AbortReason::Policy(w) => sched::AbortKind::Policy(w),
                };
                if let Some(j) = self.job.as_mut() {
                    let cmds = j.sched.step(Instant::now(), sched::Event::RunAborted { client: conn, run: r.run_id, why });
                    return self.exec(cmds);
                }
            }
            Msg::DiagReport(d) => {
                let pending = self.conns.get(&conn).and_then(|c| c.diag.clone());
                if let Some((reason, strikes, _)) = pending {
                    if let Some(c) = self.conns.get(&conn) {
                        self.write_diag(c, &reason, strikes, Some(&d));
                    }
                    if let Some(c) = self.conns.get_mut(&conn) {
                        c.diag = None;
                    }
                    self.close(conn, Some(format!("removed from the farm: {reason}")));
                }
            }
            Msg::Bye(b) => {
                let name = self.conns.get(&conn).map(|c| c.name.clone()).unwrap_or_default();
                self.note(&format!("{name} left: {}", b.reason));
            }
            _ => {}
        }
        None
    }

    fn try_admit(&mut self, conn: ClientId) {
        let Some(c) = self.conns.get(&conn) else { return };
        if c.state != ConnState::Checking {
            return;
        }
        let (name, cap, mbps) = (c.name.clone(), c.gpu_cap, c.link_mbps);
        if let (Some(j), Some(cap)) = (&self.job, cap) {
            let job_cap = j.orbit_len_cap;
            if job_cap.is_some_and(|jc| cap < jc) {
                let why = format!("this GPU holds {cap} reference samples; the running job uses {} — it can join the next job", job_cap.unwrap_or(0));
                self.note(&format!("refused {name}: {why}"));
                self.close(conn, Some(why));
                return;
            }
        }
        if let Some(c) = self.conns.get_mut(&conn) {
            c.state = ConnState::Admitted;
        }
        self.note(&format!("{name} admitted{}", mbps.map_or(String::new(), |m| format!(" (link {m:.0} Mb/s)"))));
        if let Some(j) = self.job.as_mut() {
            let cmds = j.sched.step(Instant::now(), sched::Event::Joined { client: conn, name });
            let _ = self.exec(cmds);
        }
    }

    /// Start the job once enough clients are admitted: the orbit cap, the anchors, the bundle, the
    /// manifest (resume), the scheduler.
    fn start_job(&mut self) -> Result<(), String> {
        let (w, h, fps, ss, prefix, frames) = self.res.clone();
        let admitted: Vec<ClientId> = self.conns.iter().filter(|(_, c)| c.state == ConnState::Admitted).map(|(&id, _)| id).collect();
        let cap = admitted.iter().filter_map(|id| self.conns[id].gpu_cap).min();
        if let Some(cap) = cap {
            let who = admitted.iter().find(|id| self.conns[*id].gpu_cap == Some(cap)).map(|id| self.conns[id].name.clone()).unwrap_or_default();
            self.note(&format!("reference-orbit cap for this job: {cap} samples (set by {who}, the smallest among the farm)"));
        }
        let cfg_dir = self.out.join("farm").join("controller-cfg");
        let _ = std::fs::remove_dir_all(&cfg_dir);
        write_session(&cfg_dir, &self.settings)?;
        let anchors = if self.normalize {
            let path = self.out.join("farm").join("anchors.toml");
            self.note("measuring the normalize anchors on this machine, once for every client…");
            let mut a: Vec<String> = vec![
                "--render-tour".into(),
                self.tour.to_string_lossy().into_owned(),
                "--dump-norm-anchors".into(),
                path.to_string_lossy().into_owned(),
                "--size".into(),
                format!("{w}x{h}"),
                "--fps".into(),
                format!("{fps}"),
                "--ss".into(),
                ss.to_string(),
                "--prefix".into(),
                prefix.clone(),
                "--farm-child".into(),
            ];
            if let Some(cap) = cap {
                a.push("--set".into());
                a.push(format!("ORBIT_LEN_CAP={cap}"));
            }
            let mut child = spawn_child(&a, &cfg_dir)?;
            let tail = Arc::new(std::sync::Mutex::new(VecDeque::new()));
            pump_child(&mut child, |_| {}, tail.clone(), 40);
            let st = child.wait().map_err(|e| e.to_string())?;
            if !st.success() {
                let t: Vec<String> = tail.lock().unwrap_or_else(|e| e.into_inner()).iter().cloned().collect();
                return Err(format!("measuring the normalize anchors failed: {}", t.join(" | ")));
            }
            Some(std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?)
        } else {
            None
        };
        let bundle = Bundle {
            name: self.tour_name.clone(),
            script: self.script_text.clone(),
            settings: self.settings.clone(),
            anchors: anchors.clone(),
            orbit_len_cap: cap,
            width: w,
            height: h,
            fps,
            ss,
            prefix: prefix.clone(),
            frames,
        };
        let bundle_bytes = serde_json::to_vec(&bundle).map_err(|e| e.to_string())?;
        let settings_key = format!("{}|cap={}", serde_json::to_string(&self.settings).map_err(|e| e.to_string())?, cap.map_or("-".into(), |c| c.to_string()));
        let (ver, git) = build_identity();
        let ident = JobIdentity::new(
            &self.tour_name,
            &self.tour.to_string_lossy(),
            &self.script_text,
            &settings_key,
            anchors.as_deref(),
            ver,
            git,
            (w, h),
            fps,
            ss,
            &prefix,
            frames,
            unix_ms() / 1000,
        );
        let job_id = ident.job_id.clone();
        let (manifest, resume) = Manifest::open(&self.out, ident, &|p: &Path| crate::scripting::png_frame_size(p) == Some((w, h)))?;
        if !resume.done.is_empty() || !resume.lost.is_empty() || resume.partials_removed > 0 {
            self.note(&format!(
                "resuming job {job_id}: {} of {frames} frames already done ({} adopted), {} to render again, {} unfinished write(s) removed",
                resume.done.len(),
                resume.adopted.len(),
                resume.lost.len(),
                resume.partials_removed
            ));
        }
        let mut cfg = sched::Config::new(frames);
        if let Some(s) = number::<u32>(&self.args, "--strikes") {
            cfg.strikes_to_remove = s.max(1);
        }
        if let Some(s) = number::<u64>(&self.args, "--stall") {
            cfg.stall_floor = Duration::from_secs(s.max(1));
        }
        cfg.deadline = number::<u64>(&self.args, "--deadline").map(Duration::from_secs);
        if let Some(n) = number::<u64>(&self.args, "--first-run") {
            cfg.first_run = n.max(1);
        }
        let mut sched = Scheduler::new(cfg, &resume.done);
        let bundle_sha = fractadyne_farm::sha256_hex(&bundle_bytes);
        let mut cmds = Vec::new();
        for id in &admitted {
            cmds.extend(sched.step(Instant::now(), sched::Event::Joined { client: *id, name: self.conns[id].name.clone() }));
        }
        self.job = Some(Job { manifest, sched, bundle: Arc::new(bundle_bytes), bundle_sha, job_id: job_id.clone(), orbit_len_cap: cap, frame_bytes_ewma: None });
        self.note(&format!("job {job_id} started with {} client(s)", admitted.len()));
        if self.exec(cmds).is_some() {
            // Already finished (a resume of a complete job); `periodic` reports it.
        }
        Ok(())
    }

    /// Carry out the scheduler's commands. `Some(code)` = the job is over.
    fn exec(&mut self, cmds: Vec<Command>) -> Option<i32> {
        for c in cmds {
            match c {
                Command::Assign { client, run, start, end } => {
                    let Some(j) = &self.job else { continue };
                    let job_id = j.job_id.clone();
                    let (bundle, sha) = (j.bundle.clone(), j.bundle_sha.clone());
                    if let Some(conn) = self.conns.get_mut(&client) {
                        if !conn.job_sent {
                            conn.job_sent = true;
                            let _ = conn.out.send(COut::Msg(Msg::JobOpen(JobOpen { job_id: job_id.clone(), name: self.tour_name.clone(), bundle: BlobAnnounce { id: 1, len: bundle.len() as u64, sha256: sha } })));
                            let _ = conn.out.send(COut::Blob(1, bundle));
                        }
                        let _ = conn.out.send(COut::Msg(Msg::Assign(Assign { job_id, run_id: run, start, end })));
                    }
                }
                Command::Cancel { client, run, reason } => {
                    if let Some(j) = &self.job {
                        self.send(client, Msg::Cancel(Cancel { job_id: j.job_id.clone(), run_id: run, reason }));
                    }
                }
                Command::Accept { .. } | Command::Discard { .. } => {}
                Command::Remove { client, reason, strikes } => {
                    if let Some(c) = self.conns.get_mut(&client) {
                        let name = c.name.clone();
                        c.diag = Some((reason.clone(), strikes, Instant::now()));
                        c.state = ConnState::Closing;
                        let _ = c.out.send(COut::Msg(Msg::DiagRequest(DiagRequest { items: vec![DiagItem::ChildLog, DiagItem::Handshake, DiagItem::Heartbeats] })));
                        // The notification: loud, in the console, the log and the event file.
                        self.note(&format!("⚠⚠ REMOVED {name}: {reason} — its diagnostics are being collected into {}", self.out.join("farm").join("diag").display()));
                        eprintln!("\x07"); // the terminal bell: a removal is worth a sound
                    }
                }
                Command::Drop { client, reason } => {
                    self.close(client, None);
                    let name = self.conns.get(&client).map(|c| c.name.clone()).unwrap_or_default();
                    crate::diag::log_line("farm", &format!("dropped {name}: {reason}"));
                }
                Command::Note(s) => self.note(&s),
                Command::Done { failed } => return Some(self.finish(&failed)),
            }
        }
        None
    }

    fn write_diag(&self, c: &Conn, reason: &str, strikes: u32, report: Option<&DiagReport>) {
        let Some(j) = &self.job else { return };
        let dir = j.manifest.diag_dir(&c.name, unix_ms() / 1000);
        let _ = std::fs::create_dir_all(&dir);
        let identity = format!(
            "name = {:?}\naddress = {:?}\nadapter = {:?}\norbit_len_cap = {}\nlink_mbps = {}\nreason = {:?}\nstrikes = {strikes}\nself_check = {:?}\n",
            c.name,
            c.addr,
            c.adapter,
            c.gpu_cap.unwrap_or(0),
            c.link_mbps.map_or("unknown".into(), |m| format!("{m:.1}")),
            reason,
            c.self_check
        );
        let _ = std::fs::write(dir.join("identity.toml"), identity);
        if let Some(r) = report {
            for (item, text) in &r.items {
                let file = match item {
                    DiagItem::ChildLog => "child-log.txt",
                    DiagItem::Handshake => "handshake.txt",
                    DiagItem::Heartbeats => "heartbeats.jsonl",
                };
                let _ = std::fs::write(dir.join(file), text);
            }
        } else {
            let _ = std::fs::write(dir.join("child-log.txt"), "(the client did not send its diagnostics before disconnecting)\n");
        }
        // Its events, and the frames it sent that failed verification.
        if let Ok(ev) = std::fs::read_to_string(j.manifest.farm_dir().join("events.jsonl")) {
            let mine: Vec<&str> = ev.lines().filter(|l| l.contains(&c.name)).collect();
            let _ = std::fs::write(dir.join("events.jsonl"), mine.join("\n"));
        }
        let bad = j.manifest.farm_dir().join("bad");
        if let Ok(rd) = std::fs::read_dir(&bad) {
            let safe: String = c.name.chars().map(|ch| if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' { ch } else { '_' }).collect();
            for e in rd.flatten() {
                let n = e.file_name().to_string_lossy().into_owned();
                if n.contains(&format!(".{safe}.png")) {
                    let _ = std::fs::copy(e.path(), dir.join(&n));
                }
            }
        }
        self.note(&format!("diagnostics for {} written to {}", c.name, dir.display()));
    }

    fn finish(&mut self, failed: &[u64]) -> i32 {
        let Some(j) = &self.job else { return 2 };
        let ids: Vec<ClientId> = self.conns.keys().copied().collect();
        for id in ids {
            self.send(id, Msg::JobClose(JobClose { job_id: j.job_id.clone() }));
        }
        let elapsed = self.started.elapsed().as_secs_f64();
        self.write_status();
        let snap = j.sched.snapshot();
        let msg = if failed.is_empty() {
            format!("Farm render complete: {} frames in {:.1}s → {}", snap.frames, elapsed, self.out.display())
        } else {
            format!(
                "Farm render finished with {} frame(s) not rendered ({:?}{}) — run again on the same folder to resume",
                failed.len(),
                &failed[..failed.len().min(10)],
                if failed.len() > 10 { ", …" } else { "" }
            )
        };
        self.note(&msg);
        // Let the JobClose messages go out before the process ends.
        std::thread::sleep(Duration::from_millis(300));
        if failed.is_empty() {
            0
        } else {
            3
        }
    }

    /// Ticks, keepalives, status, metrics, the storage monitor, diagnostics timeouts, job start.
    fn periodic(&mut self) -> Option<i32> {
        let now = Instant::now();
        if self.job.is_none() {
            let admitted = self.conns.values().filter(|c| c.state == ConnState::Admitted).count();
            if admitted >= self.min_clients {
                if let Err(e) = self.start_job() {
                    eprintln!("fractadyne: farm: {e}");
                    return Some(2);
                }
                if self.job.as_ref().is_some_and(|j| j.sched.is_finished()) {
                    return Some(self.finish(&[]));
                }
            }
        }
        if now.duration_since(self.last_keepalive) >= Duration::from_secs(5) {
            self.last_keepalive = now;
            for c in self.conns.values() {
                let _ = c.out.send(COut::Msg(Msg::Keepalive));
            }
        }
        // Removals whose diagnostics never came.
        let overdue: Vec<ClientId> = self.conns.iter().filter(|(_, c)| c.diag.as_ref().is_some_and(|d| now.duration_since(d.2) > Duration::from_secs(5))).map(|(&id, _)| id).collect();
        for id in overdue {
            let Some((reason, strikes, _)) = self.conns.get(&id).and_then(|c| c.diag.clone()) else { continue };
            if let Some(c) = self.conns.get(&id) {
                self.write_diag(c, &reason, strikes, None);
            }
            if let Some(c) = self.conns.get_mut(&id) {
                c.diag = None;
            }
            self.close(id, Some(format!("removed from the farm: {reason}")));
        }
        self.job.as_ref()?;
        if now.duration_since(self.last_tick) >= Duration::from_secs(1) {
            self.last_tick = now;
            let cmds = self.job.as_mut()?.sched.step(now, sched::Event::Tick);
            if let Some(code) = self.exec(cmds) {
                return Some(code);
            }
        }
        if now.duration_since(self.last_status) >= Duration::from_secs(2) {
            self.last_status = now;
            self.write_status();
        }
        if now.duration_since(self.last_metrics) >= Duration::from_secs(10) {
            self.last_metrics = now;
            if let Some(code) = self.metrics_and_storage(now) {
                return Some(code);
            }
        }
        None
    }

    fn write_status(&self) {
        let Some(j) = &self.job else { return };
        let s = j.sched.snapshot();
        let mut t = format!(
            "# Rewritten every 2 s by the farm controller.\njob = {:?}\nname = {:?}\nframes = {}\ndone = {}\nassigned = {}\npending = {}\nfailed = {}\npaused = {}\nwaiting_for_space = {}\nfinished = {}\nelapsed_s = {:.0}\n",
            j.job_id, self.tour_name, s.frames, s.done, s.assigned, s.pending, s.failed, s.paused, s.storage_low, s.finished, self.started.elapsed().as_secs_f64()
        );
        for c in &s.clients {
            let conn = self.conns.get(&c.id);
            t.push_str(&format!(
                "\n[[machine]]\nname = {:?}\nstate = \"{:?}\"\nframes_done = {}\nstrikes = {}\nms_per_frame = {}\nruns = {:?}\nadapter = {:?}\n",
                c.name,
                c.state,
                c.frames_done,
                c.strikes,
                c.ewma_ms.map_or(0.0, |m| m.round()),
                c.runs.iter().map(|r| format!("{}..{}", r.1, r.2)).collect::<Vec<_>>(),
                conn.map(|c| c.adapter.clone()).unwrap_or_default()
            ));
        }
        let _ = j.manifest.write_status(&t);
    }

    fn metrics_and_storage(&mut self, now: Instant) -> Option<i32> {
        let j = self.job.as_ref()?;
        let s = j.sched.snapshot();
        let (bin, bout): (u64, u64) = self.io.values().fold((0, 0), |(a, b), (_, io)| (a + io.bytes_in.load(Ordering::Relaxed), b + io.bytes_out.load(Ordering::Relaxed)));
        let (pdone, pin, pout, pt) = self.metrics_prev;
        let dt = now.duration_since(pt).as_secs_f64().max(1e-3);
        let fps = (s.done.saturating_sub(pdone)) as f64 / dt;
        let (kin, kout) = (bin.saturating_sub(pin) as f64 / 1024.0 / dt, bout.saturating_sub(pout) as f64 / 1024.0 / dt);
        self.metrics_prev = (s.done, bin, bout, now);
        let free = crate::sysinfo::free_disk_bytes(&self.out);
        let frame_bytes = j.frame_bytes_ewma.unwrap_or(0.0);
        let need = (s.pending + s.assigned) as f64 * frame_bytes;
        let remaining = s.frames - s.done - s.failed;
        let eta = if fps > 0.0 { remaining as f64 / fps } else { f64::NAN };
        let machines: Vec<serde_json::Value> = s
            .clients
            .iter()
            .map(|c| {
                let io = self.io.get(&c.id).map(|(_, x)| x.bytes_in.load(Ordering::Relaxed)).unwrap_or(0);
                serde_json::json!({"name": c.name, "state": format!("{:?}", c.state), "frames": c.frames_done, "ms_per_frame": c.ewma_ms, "strikes": c.strikes, "bytes_in": io})
            })
            .collect();
        let sample = serde_json::json!({
            "at_ms": unix_ms(), "done": s.done, "frames": s.frames, "frames_per_s": fps,
            "kb_in_per_s": kin, "kb_out_per_s": kout, "pending": s.pending, "assigned": s.assigned,
            "failed": s.failed, "free_bytes": free, "machines": machines,
        });
        let _ = j.manifest.record_metrics(&sample);
        println!(
            "  {}/{} frames · {fps:.2} frames/s · in {kin:.0} KB/s · out {kout:.0} KB/s · {} machine(s){}{}",
            s.done,
            s.frames,
            s.clients.iter().filter(|c| c.state == sched::ClientState::Active).count(),
            if eta.is_finite() { format!(" · about {} left", fmt_secs(eta)) } else { String::new() },
            free.map_or(String::new(), |f| format!(" · {:.1} GB free", f as f64 / 1e9))
        );
        // The storage monitor: warn early; stop handing out work before a write could fail.
        if let Some(f) = free {
            let f = f as f64;
            let low = frame_bytes > 0.0 && f < frame_bytes + 256.0 * 1024.0 * 1024.0;
            if f < (2.0 * need).max(2.0 * 1024.0 * 1024.0 * 1024.0) && !low {
                self.note(&format!("⚠ space is getting short: {:.1} GB free, about {:.1} GB still to write", f / 1e9, need / 1e9));
            }
            if low != self.storage_low {
                self.storage_low = low;
                let cmds = self.job.as_mut()?.sched.step(now, sched::Event::StorageLow(low));
                return self.exec(cmds);
            }
        }
        None
    }
}

fn fmt_secs(s: f64) -> String {
    let s = s.max(0.0) as u64;
    if s >= 3600 {
        format!("{}h{:02}m", s / 3600, s % 3600 / 60)
    } else if s >= 60 {
        format!("{}m{:02}s", s / 60, s % 60)
    } else {
        format!("{s}s")
    }
}

#[cfg(test)]
mod tests;
