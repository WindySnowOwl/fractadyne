//! The render farm, app side (design/remote-rendering.md): the headless `--farm-render` controller
//! and `--render-client` client, and the `--farmtest` harness that runs both on one machine.
//!
//! The pure half — channel, protocol, scheduler, job state — is the `fractadyne-farm` crate. This
//! half owns sockets, threads, files and child processes. Neither side renders in-process: every
//! frame comes from a `--render-tour … --farm-child` child, the tested path, so a lost GPU takes
//! down one run, never the client or the controller (the reasoning `ui/tour_render.rs` gives for the
//! Render dialog).
//!
//! Both modes run before any window or GPU device exists (`cli::run_headless`).

pub(crate) mod client;
pub(crate) mod controller;
pub(crate) mod farmtest;
pub(crate) mod status;

use fractadyne_farm::key::{FarmKey, Identity};
use fractadyne_farm::settings::RenderSettings;
use serde::{Deserialize, Serialize};
use std::io::BufRead;
use std::path::{Path, PathBuf};

/// Dispatch a farm mode, if the command line names one. Returns the exit code.
pub(crate) fn run_headless(args: &[String]) -> Option<i32> {
    if args.iter().any(|a| a == "--farmtest") {
        return Some(farmtest::run(args));
    }
    if args.iter().any(|a| a == "--farm-render") {
        return Some(controller::run(args));
    }
    if args.iter().any(|a| a == "--render-client") {
        return Some(client::run(args));
    }
    None
}

/// `<config>/farm/`: identity, keys, pins, and the client's job folders.
pub(crate) fn farm_dir() -> Result<PathBuf, String> {
    fractadyne_state::config_dir().map(|d| d.join("farm")).ok_or_else(|| "no configuration folder on this system".to_string())
}

/// This install's identity, made on first use.
pub(crate) fn identity() -> Result<Identity, String> {
    Identity::load_or_create(&farm_dir()?.join("identity.toml"))
}

/// The value after `flag`, if present; fatal when the flag is present without one.
pub(crate) fn value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    let i = args.iter().position(|a| a == flag)?;
    match args.get(i + 1) {
        Some(v) if !v.starts_with("--") => Some(v.as_str()),
        _ => {
            eprintln!("fractadyne: {flag} needs a value.");
            crate::exit(2)
        }
    }
}

/// A numeric flag value, fatal when unreadable.
pub(crate) fn number<T: std::str::FromStr>(args: &[String], flag: &str) -> Option<T> {
    value(args, flag).map(|s| match s.parse::<T>() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("fractadyne: {flag}: cannot read \"{s}\" as a number.");
            crate::exit(2)
        }
    })
}

/// The farm key: from `--farm-key-file F`, else `<config>/farm/farm-key.txt`. `create`: make and
/// save one when there is none (the controller); a client without a key cannot join.
pub(crate) fn load_key(args: &[String], create: bool) -> Result<(FarmKey, PathBuf, bool), String> {
    let path = match value(args, "--farm-key-file") {
        Some(p) => PathBuf::from(p),
        None => farm_dir()?.join("farm-key.txt"),
    };
    match std::fs::read_to_string(&path) {
        Ok(text) => FarmKey::from_text(&text).map(|k| (k, path.clone(), false)).map_err(|e| format!("{}: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound && create => {
            let k = FarmKey::generate()?;
            if let Some(d) = path.parent() {
                std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
            }
            let part = fractadyne_export::partial_path(&path);
            std::fs::write(&part, format!("{}\n", k.to_text()))
                .and_then(|()| std::fs::rename(&part, &path))
                .map_err(|e| format!("{}: {e}", path.display()))?;
            Ok((k, path, true))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(format!(
            "no farm key at {} — copy the key from the controller into that file, or pass --farm-key-file",
            path.display()
        )),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// This machine's display name: `--name`, else the computer name.
pub(crate) fn machine_name(args: &[String]) -> String {
    value(args, "--name")
        .map(str::to_string)
        .or_else(|| std::env::var("COMPUTERNAME").ok())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| "this machine".into())
}

/// The build identity both ends compare (design §9): version and commit; build sequence numbers are
/// per machine and deliberately not compared.
pub(crate) fn build_identity() -> (&'static str, &'static str) {
    (crate::sysinfo::APP_VERSION, crate::sysinfo::BUILD_GIT)
}

pub(crate) fn is_dirty(git: &str) -> bool {
    git.ends_with("-dirty")
}

/// Everything a client needs to render its share of a job, sent once per job as a blob.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Bundle {
    pub(crate) name: String,
    pub(crate) script: String,
    pub(crate) settings: RenderSettings,
    /// The normalize anchors, measured once by the controller (`--norm-anchors`).
    pub(crate) anchors: Option<String>,
    /// The job's reference-orbit length cap: the smallest among its machines (`--set ORBIT_LEN_CAP`).
    pub(crate) orbit_len_cap: Option<u64>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) fps: f64,
    pub(crate) ss: u32,
    pub(crate) prefix: String,
    pub(crate) frames: u64,
}

impl Bundle {
    /// Checks a client runs on arrival, against its own policy. `Err` = the job is refused.
    pub(crate) fn check(&self, policy: &fractadyne_farm::proto::Policy, gpu_cap: Option<u64>) -> Result<(), String> {
        fractadyne_farm::names::check_file_part("frame prefix", &self.prefix)?;
        if self.script.len() > 1 << 20 || self.anchors.as_ref().is_some_and(|a| a.len() > 1 << 20) {
            return Err("the job's script or anchors are too large".into());
        }
        if self.width == 0 || self.height == 0 || self.width > policy.max_width || self.height > policy.max_height {
            return Err(format!("{}×{} frames are larger than this machine allows ({}×{})", self.width, self.height, policy.max_width, policy.max_height));
        }
        if self.ss == 0 || self.ss > policy.max_ss {
            return Err(format!("supersampling {} is more than this machine allows ({})", self.ss, policy.max_ss));
        }
        if !(self.fps.is_finite() && self.fps > 0.0 && self.fps <= 1000.0) || self.frames == 0 || self.frames > fractadyne_farm::proto::MAX_JOB_FRAMES {
            return Err("the job's frame rate or frame count is out of range".into());
        }
        self.settings.validate()?;
        if self.settings.max_iter > policy.max_iter {
            return Err(format!("an iteration base of {} is more than this machine allows ({})", self.settings.max_iter, policy.max_iter));
        }
        if let (Some(cap), Some(mine)) = (self.orbit_len_cap, gpu_cap) {
            if cap > mine {
                return Err(format!("this GPU holds {mine} reference samples; the job uses {cap}"));
            }
        }
        Ok(())
    }
}

/// A line a `--farm-child` render prints.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ChildLine {
    Done { index: u64, bytes: u64, sha256: String, ms: u64 },
    Failed { index: u64, reason: String },
    Other(String),
}

/// Parse one stdout line of a farm child (`scripting::frame_done_line`'s format).
pub(crate) fn parse_child_line(line: &str) -> ChildLine {
    let l = line.trim();
    let field = |k: &str| l.split_whitespace().find_map(|t| t.strip_prefix(k)).map(str::to_string);
    if let Some(rest) = l.strip_prefix("frame-done ") {
        let _ = rest;
        if let (Some(i), Some(b), Some(h), Some(m)) = (field("index="), field("bytes="), field("sha256="), field("ms=")) {
            if let (Ok(index), Ok(bytes), Ok(ms)) = (i.parse(), b.parse(), m.parse()) {
                if h.len() == 64 {
                    return ChildLine::Done { index, bytes, sha256: h, ms };
                }
            }
        }
    } else if l.starts_with("frame-failed ") {
        if let Some(Ok(index)) = field("index=").map(|i| i.parse()) {
            let reason = l.split_once("reason=").map(|(_, r)| r.trim_matches('"').to_string()).unwrap_or_default();
            return ChildLine::Failed { index, reason };
        }
    }
    ChildLine::Other(l.to_string())
}

/// What a render child's stderr says about its GPU: the adapter line and the orbit-length cap.
pub(crate) fn gpu_facts(stderr: &str) -> (Option<String>, Option<u64>) {
    let adapter = stderr.lines().find_map(|l| {
        let rest = l.split_once("adapter: ")?.1;
        Some(rest.split(" · capability").next().unwrap_or(rest).trim().to_string())
    });
    let cap = stderr.lines().find_map(|l| {
        let rest = l.split_once("reference-orbit length cap = ")?.1;
        rest.split_whitespace().next()?.parse().ok()
    });
    (adapter, cap)
}

/// Start a render child of this executable: `args`, with `config_dir` as its configuration folder,
/// stdout and stderr piped, no console window, no sound.
pub(crate) fn spawn_child(args: &[String], config_dir: &Path) -> Result<std::process::Child, String> {
    if args.is_empty() {
        return Err("refusing to start a render process with no arguments (that opens the app's window)".into());
    }
    let exe = std::env::current_exe().map_err(|e| format!("cannot find this executable: {e}"))?;
    std::fs::create_dir_all(config_dir).map_err(|e| format!("{}: {e}", config_dir.display()))?;
    let mut cmd = std::process::Command::new(exe);
    cmd.args(args)
        .env("FRACTADYNE_CONFIG_DIR", config_dir)
        .env("FRACTADYNE_NO_SOUND", "1")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.spawn().map_err(|e| format!("could not start a render process: {e}"))
}

/// Read a child's two streams on threads: each stdout line goes to `on_line`; stderr is kept (its
/// last `keep` lines) in `tail`, which outlives the child for diagnostics.
pub(crate) fn pump_child(
    child: &mut std::process::Child,
    on_line: impl Fn(String) + Send + 'static,
    tail: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<String>>>,
    keep: usize,
) {
    if let Some(out) = child.stdout.take() {
        std::thread::spawn(move || {
            for line in std::io::BufReader::new(out).lines().map_while(Result::ok) {
                on_line(line);
            }
        });
    }
    if let Some(err) = child.stderr.take() {
        std::thread::spawn(move || {
            for line in std::io::BufReader::new(err).lines().map_while(Result::ok) {
                let mut t = tail.lock().unwrap_or_else(|e| e.into_inner());
                t.push_back(line);
                while t.len() > keep {
                    t.pop_front();
                }
            }
        });
    }
}

/// The one-frame tour a client renders as its self-check — and the PROBE (design §9): the
/// controller renders the same tour and counts differing pixels, which tells apart GPUs whose
/// pictures differ. Past 1e30× so the perturbation path runs, where two GPU models measurably
/// differ (an RX 6800 XT and an RTX 3080 on 2.7 % of a 640×360 frame of this view, 2026-10-04). It
/// also proves the GPU, the render path and the child launch together, and its log names the
/// adapter and the orbit-length cap.
pub(crate) const SELF_CHECK_TOUR: &str = r#"format_version = 2
name = "Render-farm probe"

[render]
size = "256x144"
fps = 1
max_iter = 20000
auto_iter = false

[[keyframe]]
t = 0
re = "-5.62202621523037212744969596262961926232336058642000859332104071064648040651980117009368022864076665266819518615342205563126413961786451e-1"
im = "6.42817149072775248899624656627830941472997397665282056405495715932366418738755172614822993656471501541311398325174287701850021449311247e-1"
zoom = "1e30"
"#;

/// Render the probe (`SELF_CHECK_TOUR`) in `dir` with a fresh configuration: its PNG bytes and the
/// render's stderr (which names the adapter and the orbit cap). Blocking, at most 120 s.
pub(crate) fn render_probe(dir: &Path) -> Result<(Vec<u8>, String), String> {
    use std::time::{Duration, Instant};
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let tour = dir.join("self-check.toml");
    std::fs::write(&tour, SELF_CHECK_TOUR).map_err(|e| e.to_string())?;
    let frames = dir.join("frames");
    let args: Vec<String> = ["--render-tour", &tour.to_string_lossy(), "--out", &frames.to_string_lossy(), "--frames", "0..1", "--farm-child", "-y"].iter().map(|s| s.to_string()).collect();
    let mut child = spawn_child(&args, &dir.join("cfg"))?;
    let tail = std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new()));
    let (tx, rx) = std::sync::mpsc::channel();
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
    if !(status.success() && done) {
        return Err(format!("the test render failed (exit {:?}): {}", status.code(), fractadyne_farm::proto::tail(&err, 300)));
    }
    let png = std::fs::read(frames.join(fractadyne_farm::names::frame_file_name("self-check", 0))).map_err(|e| format!("the test render's frame: {e}"))?;
    Ok((png, err))
}

/// A probe as compared: its size, its pixels' digest (machines whose digests match render this
/// view identically), and its RGBA.
pub(crate) struct Probe {
    pub(crate) w: u32,
    pub(crate) h: u32,
    pub(crate) digest: String,
    pub(crate) rgba: Vec<u8>,
}

impl Probe {
    pub(crate) fn decode(png: &[u8]) -> Result<Probe, String> {
        let (w, h, rgba) = fractadyne_export::read_png_rgba8_bytes(png).map_err(|e| format!("the probe is not a readable PNG: {e}"))?;
        Ok(Probe { w, h, digest: fractadyne_farm::sha256_hex(&rgba), rgba })
    }

    /// Pixels where `self` and `other` differ; `None` when their sizes do.
    pub(crate) fn differing_px(&self, other: &Probe) -> Option<u64> {
        ((self.w, self.h) == (other.w, other.h)).then(|| self.rgba.chunks_exact(4).zip(other.rgba.chunks_exact(4)).filter(|(a, b)| a != b).count() as u64)
    }
}

/// Write a fresh session for render children from `settings` into `dir/session.toml`.
pub(crate) fn write_session(dir: &Path, settings: &RenderSettings) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let text = toml::to_string_pretty(&settings.to_session()).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("session.toml"), text).map_err(|e| format!("{}: {e}", dir.display()))
}

/// This machine's address on the network it would reach the internet through: what to tell
/// clients. A UDP "connect" picks the route without sending anything.
pub(crate) fn lan_address() -> Option<std::net::IpAddr> {
    let s = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect("192.0.2.1:9").ok()?; // TEST-NET-1: routed like any public address, never answered
    s.local_addr().ok().map(|a| a.ip()).filter(|ip| !ip.is_unspecified() && !ip.is_loopback())
}

/// Unix time in milliseconds.
pub(crate) fn unix_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

#[cfg(test)]
mod farm_tests;
