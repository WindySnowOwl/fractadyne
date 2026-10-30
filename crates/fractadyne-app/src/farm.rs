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
#[path = "harness/farm/farmtest.rs"]
pub(crate) mod farmtest;
pub(crate) mod local;
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
    if args.iter().any(|a| a == "--farmtest-strips") {
        return Some(farmtest::run_strips(args));
    }
    // A tour's frames (`--farm-render TOUR`), or a zoom video's strips (`--render-zoom-video F …
    // --farm`, `design/zoom-video.md` §8.5).
    let zoom_farm = args.iter().any(|a| a == "--render-zoom-video") && args.iter().any(|a| a == "--farm");
    if args.iter().any(|a| a == "--farm-render") || zoom_farm {
        return Some(controller::run(args));
    }
    if args.iter().any(|a| a == "--render-client") {
        return Some(client::run(args));
    }
    // `--render-tour … --gpus`: a farm on this machine, or (one card) the ordinary render below.
    local::run(args)
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
        // Linux: HOSTNAME is a shell variable, not exported to a program the shell starts, so a
        // farm on the test machine called itself "this machine" (2026-10-04). The kernel's own.
        .or_else(|| {
            ["/proc/sys/kernel/hostname", "/etc/hostname"]
                .iter()
                .find_map(|p| std::fs::read_to_string(p).ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| "this machine".into())
}

/// The build identity both ends compare (design §9): version and commit; build sequence numbers are
/// per machine and deliberately not compared.
pub(crate) fn build_identity() -> (&'static str, &'static str) {
    (crate::sysinfo::APP_VERSION, crate::sysinfo::BUILD_GIT)
}

/// Two build stamps name one commit: equal, or the same suffix (`-dirty`, `-archive`) and one sha a
/// prefix of the other, at least 7 digits. A stamp's length is not the commit: builds before the stamp was
/// fixed at 7 abbreviated to whatever was unique in their own repository ("29988ec" on Windows,
/// "29988ecf" in a Linux clone of the same commit), and an exact comparison refused the pair.
pub(crate) fn same_commit(a: &str, b: &str) -> bool {
    fn split(s: &str) -> (&str, &str) {
        s.split_once('-').unwrap_or((s, ""))
    }
    if a == b {
        return true;
    }
    let ((ha, ra), (hb, rb)) = (split(a), split(b));
    ra == rb && ha.len() >= 7 && hb.len() >= 7 && (ha.starts_with(hb) || hb.starts_with(ha))
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
    /// Share mode (design §7): a client with a shared folder writes its frames there
    /// (`names::share_dir`) instead of streaming them.
    #[serde(default)]
    pub(crate) share: bool,
    /// Shared computation (design §8): renders use the orbit cache, and the farm passes cached
    /// reference orbits between machines. Off: every machine builds its own.
    #[serde(default)]
    pub(crate) sharing: bool,
    /// A zoom video's strips instead of a tour's frames (`design/zoom-video.md` §8.5): unit K is
    /// strip K of this dive, `width`/`height` a strip's size, `script` and `anchors` empty.
    #[serde(default)]
    pub(crate) strips: Option<StripJob>,
}

/// The dive a strip job renders: what `--render-zoom-video … --farm` was given, as typed fields
/// (a client builds its render's command line from them — never from argument text off the wire).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct StripJob {
    /// The dive's manifest (`expmap.json`) as the controller's plan wrote it. A client writes it
    /// into its render folder first, so a client whose own plan differs is refused by the renderer
    /// (`write_or_check_manifest`) instead of rendering another dive.
    pub(crate) manifest: String,
    /// The dive's centre, as the viewport's text.
    pub(crate) center_re: String,
    pub(crate) center_im: String,
    /// Where it ends (`--zoom-log2`) and starts (`--expmap-start`), log2 magnification.
    pub(crate) zoom_log2: f64,
    pub(crate) start_log2: f64,
    /// The video's frame size (`--size`): the strips are sized for it.
    pub(crate) frame_width: u32,
    pub(crate) frame_height: u32,
    /// `--expmap-oversample`.
    pub(crate) oversample: f64,
    /// `--expmap-cap fixed` (else rising).
    pub(crate) cap_fixed: bool,
    /// `--iter N`, when given.
    pub(crate) iter: Option<u32>,
    /// `--bla` / `--no-bla`.
    pub(crate) bla: bool,
}

/// A number as the command line writes one (`-1.19e-2`): what a centre may be.
fn plain_number(s: &str) -> bool {
    !s.is_empty() && s.len() <= 100_000 && s.chars().all(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '+' | '-')) && s.chars().any(|c| c.is_ascii_digit())
}

impl StripJob {
    /// The strip job's own checks (beside [`Bundle::check`]'s): a centre that is a number, depths,
    /// a frame size and oversampling the app accepts, an iteration count the policy allows, and a
    /// manifest that is a manifest.
    fn check(&self, policy: &fractadyne_farm::proto::Policy) -> Result<(), String> {
        if !plain_number(&self.center_re) || !plain_number(&self.center_im) {
            return Err("the dive's centre is not a number".into());
        }
        if !(self.zoom_log2.is_finite() && self.start_log2.is_finite() && self.start_log2 >= 0.0 && self.zoom_log2 > self.start_log2 && self.zoom_log2 <= 1e7) {
            return Err("the dive's depths are out of range".into());
        }
        if !(16..=16384).contains(&self.frame_width) || !(16..=16384).contains(&self.frame_height) {
            return Err(format!("{}×{} video frames are out of range", self.frame_width, self.frame_height));
        }
        if !(self.oversample.is_finite() && (1.0..=8.0).contains(&self.oversample)) {
            return Err("the strips' oversampling is out of range".into());
        }
        if self.iter.is_some_and(|n| n == 0 || n > policy.max_iter) {
            return Err(format!("an iteration count of {} is more than this machine allows ({})", self.iter.unwrap_or(0), policy.max_iter));
        }
        if self.manifest.len() > 1 << 20 || serde_json::from_str::<serde_json::Value>(&self.manifest).is_err() {
            return Err("the dive's manifest is not one".into());
        }
        Ok(())
    }

    /// The render of strips `start..end` (half-open, as `Assign` gives them) into `out`:
    /// `--render-expmap OUT --expmap-strips A-B` and the dive's flags. The caller adds what every
    /// farm render gets (`--set ORBIT_LEN_CAP`, `--orbit-cache`, `--adapter`).
    pub(crate) fn child_args(&self, out: &Path, start: u64, end: u64) -> Vec<String> {
        let mut a: Vec<String> = vec![
            "--render-expmap".into(),
            out.to_string_lossy().into_owned(),
            "--expmap-strips".into(),
            format!("{start}-{}", end.saturating_sub(1).max(start)),
            "--farm-child".into(),
        ];
        a.extend(self.dive_args());
        a
    }

    /// The controller's plan of the dive (`--expmap-plan`): the manifest written into `out` (or
    /// the one there checked), nothing rendered.
    pub(crate) fn plan_args(&self, out: &Path) -> Vec<String> {
        let mut a: Vec<String> = vec!["--render-expmap".into(), out.to_string_lossy().into_owned(), "--expmap-plan".into()];
        a.extend(self.dive_args());
        a
    }

    /// The dive's own flags.
    fn dive_args(&self) -> Vec<String> {
        let mut a: Vec<String> = vec![
            "--center".into(),
            self.center_re.clone(),
            self.center_im.clone(),
            "--zoom-log2".into(),
            format!("{}", self.zoom_log2),
            "--size".into(),
            format!("{}x{}", self.frame_width, self.frame_height),
            "--expmap-oversample".into(),
            format!("{}", self.oversample),
            if self.bla { "--bla" } else { "--no-bla" }.into(),
        ];
        if self.start_log2 != 0.0 {
            a.extend(["--expmap-start".into(), format!("{}", self.start_log2)]);
        }
        if self.cap_fixed {
            a.extend(["--expmap-cap".into(), "fixed".into()]);
        }
        if let Some(n) = self.iter {
            a.extend(["--iter".into(), n.to_string()]);
        }
        a
    }
}

impl Bundle {
    /// What the job's units are.
    pub(crate) fn kind(&self) -> fractadyne_farm::names::UnitKind {
        if self.strips.is_some() {
            fractadyne_farm::names::UnitKind::Strip
        } else {
            fractadyne_farm::names::UnitKind::Frame
        }
    }

    /// The file unit `index` is written under.
    pub(crate) fn unit_file_name(&self, index: u64) -> String {
        fractadyne_farm::names::unit_file_name(self.kind(), &self.prefix, index)
    }

    /// Checks a client runs on arrival, against its own policy. `Err` = the job is refused.
    pub(crate) fn check(&self, policy: &fractadyne_farm::proto::Policy, gpu_cap: Option<u64>) -> Result<(), String> {
        fractadyne_farm::names::check_file_part("frame prefix", &self.prefix)?;
        if let Some(s) = &self.strips {
            s.check(policy)?;
            if self.prefix != "strip" || !self.script.is_empty() || self.anchors.is_some() {
                return Err("a zoom video's strip job carries a script, anchors or another file prefix".into());
            }
        }
        if self.script.len() > 1 << 20 || self.anchors.as_ref().is_some_and(|a| a.len() > 1 << 20) {
            return Err("the job's script or anchors are too large".into());
        }
        // A strip is long and thin (a 1080p one at 3x is 20763×2294): its texels against the
        // policy's frame, not its sides.
        let too_big = match &self.strips {
            Some(_) => self.width as u64 * self.height as u64 > policy.max_width as u64 * policy.max_height as u64,
            None => self.width > policy.max_width || self.height > policy.max_height,
        };
        if self.width == 0 || self.height == 0 || too_big {
            let what = if self.strips.is_some() { "strips" } else { "frames" };
            return Err(format!("{}×{} {what} are larger than this machine allows ({}×{})", self.width, self.height, policy.max_width, policy.max_height));
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
    /// `reference`: where the frame's reference came from (`fresh`, `cache`, `reused`, `none`).
    Done { index: u64, bytes: u64, sha256: String, ms: u64, reference: Option<String> },
    Failed { index: u64, reason: String },
    /// Its reference build has advanced ([`reference_work_lines`]); `steps` is the process's count.
    ReferenceWork { steps: u64 },
    Other(String),
}

/// How often a farm render says its reference build has advanced.
const REFERENCE_WORK_EVERY: std::time::Duration = std::time::Duration::from_secs(5);

/// While a farm child renders: a `reference-work steps=N` line on stdout for every
/// [`REFERENCE_WORK_EVERY`] in which its reference build advanced
/// ([`fractadyne_core::reference_steps`], counted as a build runs), for its client to pass on
/// ([`fractadyne_farm::proto::Heartbeat::reference_work_ms`]). A deep reference finishes no frame
/// for many minutes, and the controller's stall rule took that for a hang: at 9.98e60205× every
/// client's 7–8 minute build was cancelled at 300 s and started again (2026-10-29). Only the
/// reference build counts, not a GPU pass: frame times are what the stall rule already measures,
/// and a frame that loops on passes forever must still look stalled. Stops when dropped.
pub(crate) struct ReferenceWorkLines {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

pub(crate) fn reference_work_lines() -> ReferenceWorkLines {
    use std::sync::atomic::Ordering::Relaxed;
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop2 = stop.clone();
    let thread = std::thread::Builder::new()
        .name("fd-ref-work".into())
        .spawn(move || {
            let mut last = fractadyne_core::reference_steps();
            let mut next = std::time::Instant::now() + REFERENCE_WORK_EVERY;
            // Checks `stop` every 100 ms, so dropping the guard never waits long.
            while !stop2.load(Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if std::time::Instant::now() < next {
                    continue;
                }
                next = std::time::Instant::now() + REFERENCE_WORK_EVERY;
                let n = fractadyne_core::reference_steps();
                if n != last {
                    last = n;
                    println!("reference-work steps={n}");
                }
            }
        })
        .ok();
    ReferenceWorkLines { stop, thread }
}

impl Drop for ReferenceWorkLines {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
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
                    let reference = field("ref=").filter(|r| ["fresh", "cache", "reused", "none"].contains(&r.as_str()));
                    return ChildLine::Done { index, bytes, sha256: h, ms, reference };
                }
            }
        }
    } else if l.starts_with("frame-failed ") {
        if let Some(Ok(index)) = field("index=").map(|i| i.parse()) {
            let reason = l.split_once("reason=").map(|(_, r)| r.trim_matches('"').to_string()).unwrap_or_default();
            return ChildLine::Failed { index, reason };
        }
    } else if l.starts_with("reference-work ") {
        if let Some(Ok(steps)) = field("steps=").map(|s| s.parse()) {
            return ChildLine::ReferenceWork { steps };
        }
    }
    ChildLine::Other(l.to_string())
}

/// What a render child's log says about its GPU.
#[derive(Debug, PartialEq, Default)]
pub(crate) struct GpuFacts {
    /// "NVIDIA GeForce RTX 3080 · Vulkan".
    pub(crate) adapter: Option<String>,
    /// "NVIDIA 581.42".
    pub(crate) driver: Option<String>,
    pub(crate) orbit_len_cap: Option<u64>,
}

/// Read the adapter, driver and orbit-length cap lines of a render child's stderr.
pub(crate) fn gpu_facts(stderr: &str) -> GpuFacts {
    let after = |key: &str| stderr.lines().find_map(|l| l.split_once(key).map(|(_, r)| r.trim().to_string()));
    GpuFacts {
        adapter: after("adapter: ").map(|r| r.split(" · capability").next().unwrap_or(&r).trim().to_string()),
        driver: after("] driver: ").filter(|d| d != "unknown"),
        orbit_len_cap: after("reference-orbit length cap = ").and_then(|r| r.split_whitespace().next()?.parse().ok()),
    }
}

/// The driver as one phrase from wgpu's two fields (either may be empty): "NVIDIA 581.42".
pub(crate) fn driver_text(driver: &str, info: &str) -> String {
    let t = [driver.trim(), info.trim()].iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(" ");
    if t.is_empty() { "unknown".into() } else { t }
}

/// How one machine's GPU differs from another's, worst first.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum GpuDiff {
    /// Another GPU model.
    Model,
    /// The same GPU through another graphics API (another shader compiler).
    Backend,
    /// The same GPU and API on another driver version.
    Driver,
}

impl GpuDiff {
    pub(crate) fn words(self) -> &'static str {
        match self {
            GpuDiff::Model => "a different GPU",
            GpuDiff::Backend => "the same GPU through a different graphics API",
            GpuDiff::Driver => "the same GPU on a different driver",
        }
    }
}

/// `None` when `b` is the same GPU as `a`, through the same API, on the same driver. A driver
/// either side does not know is not called a mismatch.
pub(crate) fn gpu_difference(a: &fractadyne_farm::proto::GpuInfo, b: &fractadyne_farm::proto::GpuInfo) -> Option<GpuDiff> {
    let split = |s: &str| {
        let (m, api) = s.split_once(" · ").unwrap_or((s, ""));
        (m.trim().to_ascii_lowercase(), api.trim().to_ascii_lowercase())
    };
    let ((ma, aa), (mb, ab)) = (split(&a.adapter), split(&b.adapter));
    if ma != mb {
        Some(GpuDiff::Model)
    } else if aa != ab {
        Some(GpuDiff::Backend)
    } else if !a.driver.is_empty() && !b.driver.is_empty() && a.driver != b.driver {
        Some(GpuDiff::Driver)
    } else {
        None
    }
}

/// "NVIDIA GeForce RTX 3080 · Vulkan, driver NVIDIA 581.42".
pub(crate) fn gpu_text(g: &fractadyne_farm::proto::GpuInfo) -> String {
    if g.driver.is_empty() {
        format!("{}, driver unknown", g.adapter)
    } else {
        format!("{}, driver {}", g.adapter, g.driver)
    }
}

/// Start a render child of this executable: `args`, with `config_dir` as its configuration folder,
/// stdout and stderr piped, no console window, no sound.
pub(crate) fn spawn_child(args: &[String], config_dir: &Path) -> Result<std::process::Child, String> {
    std::fs::create_dir_all(config_dir).map_err(|e| format!("{}: {e}", config_dir.display()))?;
    spawn_in(args, Some(config_dir))
}

/// [`spawn_child`] in THIS process's own configuration (its session, so its colouring): a
/// controller's own step, such as assembling a zoom video once its strips are in.
pub(crate) fn spawn_child_here(args: &[String]) -> Result<std::process::Child, String> {
    spawn_in(args, None)
}

fn spawn_in(args: &[String], config_dir: Option<&Path>) -> Result<std::process::Child, String> {
    if args.is_empty() {
        return Err("refusing to start a render process with no arguments (that opens the app's window)".into());
    }
    let exe = std::env::current_exe().map_err(|e| format!("cannot find this executable: {e}"))?;
    let mut cmd = std::process::Command::new(exe);
    if let Some(d) = config_dir {
        cmd.env("FRACTADYNE_CONFIG_DIR", d);
    }
    cmd.args(args)
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
pub(crate) fn render_probe(dir: &Path, adapter: Option<&str>) -> Result<(Vec<u8>, String), String> {
    use std::time::{Duration, Instant};
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let tour = dir.join("self-check.toml");
    std::fs::write(&tour, SELF_CHECK_TOUR).map_err(|e| e.to_string())?;
    let frames = dir.join("frames");
    let mut args: Vec<String> = ["--render-tour", &tour.to_string_lossy(), "--out", &frames.to_string_lossy(), "--frames", "0..1", "--farm-child", "-y"].iter().map(|s| s.to_string()).collect();
    if let Some(a) = adapter {
        args.push(crate::gpu_choice::FLAG.into());
        args.push(a.into());
    }
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

/// A reference orbit as the farm passes it on: its bytes verified (`orbit_blob::decode` checks the
/// whole file's digest) and the file name the orbit cache gives it, from its own header.
pub(crate) fn orbit_file_name(bytes: &[u8]) -> Option<(String, crate::render::orbit_blob::Header)> {
    let d = crate::render::orbit_blob::decode(bytes)?;
    Some((format!("{:016x}.{}", d.header.key_id(), crate::refcache_persist::ENTRY_EXT), d.header))
}

/// Move a file: a rename, or — across drives, where a rename cannot — a copy to `<dst>.part`, a
/// rename into place, and the source removed.
pub(crate) fn move_file(src: &Path, dst: &Path) -> std::io::Result<()> {
    if std::fs::rename(src, dst).is_ok() {
        return Ok(());
    }
    let part = fractadyne_export::partial_path(dst);
    std::fs::copy(src, &part)?;
    std::fs::rename(&part, dst)?;
    let _ = std::fs::remove_file(src);
    Ok(())
}

/// `--when-idle`: this machine counts as in use when someone touched it within `need` and its
/// screen is not locked — a locked machine is nobody's, whatever its input timer says. An idle time
/// that cannot be read counts as idle, as in the field agent's rule.
pub(crate) fn in_use(need: std::time::Duration, idle: Option<std::time::Duration>, locked: bool) -> bool {
    !locked && idle.is_some_and(|i| i < need)
}

/// Unix time in milliseconds.
pub(crate) fn unix_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

#[cfg(test)]
#[path = "../tests/unit/farm/farm_tests.rs"]
mod farm_tests;
