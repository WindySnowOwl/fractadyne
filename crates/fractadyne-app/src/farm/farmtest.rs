//! `--farmtest [DIR]`: a whole render farm on one machine, with faults, against an independent
//! reference (design §12, Phase 1 gate).
//!
//! 1. A reference: the gate tour rendered by one plain `--render-tour --farm-child` process.
//! 2. A controller on a loopback port, waiting for three clients.
//! 3. Three clients: A sends its first two frames corrupted (the `FRACTADYNE_FARM_CORRUPT_FRAMES`
//!    instrument) and must be removed with a diagnostics bundle; B is killed — by its own process
//!    id — once it has sent a frame, and its frames must go back to the queue; C behaves.
//! 4. The verdict: the controller exits 0; the output holds exactly the tour's frames, each
//!    PIXEL-identical to the reference; `done.jsonl` records each once; A's bad frames are in
//!    `farm/bad/`, its bundle in `farm/diag/`, and the event log tells both stories.
//!
//! Exit 0 pass, 1 fail, 2 VACUOUS (a fault the test depends on never happened — B was killed before
//! sending anything, or A was never caught — so the run proved nothing about it).

use super::*;
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// 19 frames (6 s at 3 fps), normalized, three keyframes, a palette blend and a caption — small, so
/// the whole test runs in well under a minute.
const TOUR: &str = r#"format_version = 2
name = "Farm test"

[render]
size = "160x90"
fps = 3
ss = 1
normalize = true
max_iter = 2000
auto_iter = false

[[location]]
id = "target"
re = "-5.62202621523037212744969596262961926232336058642000859332104071064648040651980117009368022864076665266819518615342205563126413961786451e-1"
im = "6.42817149072775248899624656627830941472997397665282056405495715932366418738755172614822993656471501541311398325174287701850021449311247e-1"

[[keyframe]]
t = 0
re = "-0.5"
im = "0"
zoom = 1.0
max_iter = 2000
palette = "Ember"

[[keyframe]]
t = 2
location = "target"
zoom = 8.0
max_iter = 3000

[[keyframe]]
t = 6
location = "target"
zoom = "1e30"
max_iter = 20000
palette = "Nebula"

[[annotation]]
kind = "caption"
text = "farm test"
t = 0
secs = 0
pos = "bottom"
"#;

const FRAMES: u64 = 19;

struct Proc {
    name: &'static str,
    child: std::process::Child,
    lines: mpsc::Receiver<String>,
    log: Vec<String>,
}

impl Proc {
    fn spawn(name: &'static str, args: &[String], cfg: &Path, env: &[(&str, &str)]) -> Result<Self, String> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(cfg).map_err(|e| e.to_string())?;
        let mut cmd = std::process::Command::new(exe);
        cmd.args(args)
            .env("FRACTADYNE_CONFIG_DIR", cfg)
            .env("FRACTADYNE_NO_SOUND", "1")
            .env_remove(super::client::CORRUPT_INSTRUMENT)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        for (k, v) in env {
            cmd.env(k, v);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0800_0000);
        }
        let mut child = cmd.spawn().map_err(|e| format!("{name}: {e}"))?;
        let (tx, rx) = mpsc::channel();
        // Both streams into one channel, stderr marked, so the record shows everything it said.
        for (stream, mark) in [(child.stdout.take().map(|s| Box::new(s) as Box<dyn std::io::Read + Send>), ""), (child.stderr.take().map(|s| Box::new(s) as Box<dyn std::io::Read + Send>), "! ")] {
            if let Some(s) = stream {
                let tx = tx.clone();
                std::thread::spawn(move || {
                    for l in std::io::BufReader::new(s).lines().map_while(Result::ok) {
                        if tx.send(format!("{mark}{l}")).is_err() {
                            return;
                        }
                    }
                });
            }
        }
        Ok(Self { name, child, lines: rx, log: Vec::new() })
    }

    /// Drain what it has printed; `true` if any new line matched `pat`.
    fn drain(&mut self, pat: &str) -> bool {
        let mut hit = false;
        for l in self.lines.try_iter() {
            if l.contains(pat) {
                hit = true;
            }
            println!("    [{}] {}", self.name, l);
            self.log.push(l);
        }
        hit
    }

    fn running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}

pub(crate) fn run(args: &[String]) -> i32 {
    match run_inner(args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("farmtest: {e}");
            1
        }
    }
}

fn run_inner(args: &[String]) -> Result<i32, String> {
    let base = match args.iter().position(|a| a == "--farmtest").and_then(|i| args.get(i + 1)).filter(|v| !v.starts_with("--")) {
        Some(d) => PathBuf::from(d),
        None => std::env::temp_dir().join(format!("fractadyne-farmtest-{}", std::process::id())),
    };
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).map_err(|e| format!("{}: {e}", base.display()))?;
    let tour = base.join("farm-test.toml");
    std::fs::write(&tour, TOUR).map_err(|e| e.to_string())?;
    let key_file = base.join("farm-key.txt");
    std::fs::write(&key_file, format!("{}\n", FarmKey::generate()?.to_text())).map_err(|e| e.to_string())?;
    let s = |v: &str| v.to_string();
    let p = |v: &Path| v.to_string_lossy().into_owned();
    println!("Fractadyne farmtest — {}", base.display());
    let t0 = Instant::now();

    // 1. The reference.
    println!("  reference render…");
    let refdir = base.join("reference");
    let mut r = Proc::spawn("reference", &[s("--render-tour"), p(&tour), s("--out"), p(&refdir), s("--farm-child"), s("-y")], &base.join("cfg-reference"), &[])?;
    let st = r.child.wait().map_err(|e| e.to_string())?;
    r.drain("");
    if !st.success() {
        return Err(format!("the reference render failed ({st})"));
    }

    // 2. The controller.
    let out = base.join("farm-out");
    let mut ctl = Proc::spawn(
        "controller",
        &[
            s("--farm-render"),
            p(&tour),
            s("--out"),
            p(&out),
            s("--listen"),
            s("127.0.0.1:0"),
            s("--min-clients"),
            s("3"),
            s("--farm-key-file"),
            p(&key_file),
            s("--first-run"),
            s("3"),
            s("--strikes"),
            s("2"),
            s("--farm-allow-dirty"),
        ],
        &base.join("cfg-controller"),
        &[],
    )?;
    let port = {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Ok(l) = ctl.lines.recv_timeout(Duration::from_millis(200)) {
                println!("    [controller] {l}");
                ctl.log.push(l.clone());
                if let Some(rest) = l.strip_prefix("listening on ") {
                    break rest.rsplit(':').next().and_then(|p| p.trim().parse::<u16>().ok()).ok_or("no port in the listening line")?;
                }
            }
            if !ctl.running() || Instant::now() > deadline {
                return Err("the controller never started listening".into());
            }
        }
    };
    let addr = format!("127.0.0.1:{port}");

    // 3. The clients.
    let client = |name: &'static str, env: &[(&str, &str)]| {
        Proc::spawn(
            name,
            &[s("--render-client"), addr.clone(), s("--farm-key-file"), p(&key_file), s("--name"), format!("farmtest-{name}"), s("--farm-allow-dirty")],
            &base.join(format!("cfg-{name}")),
            env,
        )
    };
    let mut a = client("A", &[(super::client::CORRUPT_INSTRUMENT, "2")])?;
    let mut b = client("B", &[])?;
    let mut c = client("C", &[])?;

    // 4. Run, killing B once it has sent a frame.
    let mut b_killed = false;
    let deadline = Instant::now() + Duration::from_secs(300);
    let code = loop {
        ctl.drain("");
        a.drain("");
        c.drain("");
        if b.drain("frame ") && !b_killed && b.log.iter().any(|l| l.contains(" sent (")) {
            println!("  → killing client B (pid {}) mid-run", b.child.id());
            let _ = b.child.kill();
            b_killed = true;
        }
        if let Ok(Some(st)) = ctl.child.try_wait() {
            std::thread::sleep(Duration::from_millis(300));
            ctl.drain("");
            break st.code().unwrap_or(-1);
        }
        if Instant::now() > deadline {
            let _ = ctl.child.kill();
            break -2;
        }
        std::thread::sleep(Duration::from_millis(100));
    };
    for pr in [&mut a, &mut b, &mut c] {
        if pr.running() {
            let _ = pr.child.kill();
        }
        let _ = pr.child.wait();
        pr.drain("");
    }

    // 5. The verdict.
    let mut fails: Vec<String> = Vec::new();
    let mut check = |ok: bool, what: String| {
        println!("  [{}] {what}", if ok { "PASS" } else { "FAIL" });
        if !ok {
            fails.push(what);
        }
    };
    check(code == 0, format!("the controller finished the job (exit {code})"));
    let frames: Vec<u64> = std::fs::read_dir(&out)
        .map(|rd| {
            rd.flatten()
                .filter_map(|e| {
                    let n = e.file_name().to_string_lossy().into_owned();
                    n.strip_prefix("farm-test_")?.strip_suffix(".png")?.parse().ok()
                })
                .collect()
        })
        .unwrap_or_default();
    let mut sorted = frames.clone();
    sorted.sort_unstable();
    check(sorted == (0..FRAMES).collect::<Vec<_>>(), format!("exactly frames 0..{FRAMES} were written ({} files)", frames.len()));
    let mut identical = 0;
    for i in 0..FRAMES {
        let name = fractadyne_farm::names::frame_file_name("farm-test", i);
        let (fa, fb) = (fractadyne_export::read_png_rgba8(&out.join(&name)), fractadyne_export::read_png_rgba8(&refdir.join(&name)));
        if let (Ok(x), Ok(y)) = (fa, fb) {
            if x == y {
                identical += 1;
            }
        }
    }
    check(identical == FRAMES, format!("{identical} of {FRAMES} frames are pixel-identical to the single-machine reference"));
    let done = std::fs::read_to_string(out.join("farm").join("done.jsonl")).unwrap_or_default();
    let mut idx: Vec<u64> = done.lines().filter_map(|l| serde_json::from_str::<fractadyne_farm::manifest::DoneRecord>(l).ok()).map(|d| d.index).collect();
    let records = idx.len();
    idx.sort_unstable();
    idx.dedup();
    check(records == FRAMES as usize && idx.len() == FRAMES as usize, format!("done.jsonl records each frame once ({records} records, {} distinct)", idx.len()));
    let bad: Vec<String> = std::fs::read_dir(out.join("farm").join("bad")).map(|rd| rd.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect()).unwrap_or_default();
    let bad_a = bad.iter().filter(|n| n.contains("farmtest-A")).count();
    check(bad_a >= 2, format!("client A's corrupted frames were caught and quarantined ({bad_a} in farm/bad)"));
    let diag_a = std::fs::read_dir(out.join("farm").join("diag"))
        .map(|rd| rd.flatten().any(|e| e.file_name().to_string_lossy().starts_with("farmtest-A") && e.path().join("identity.toml").exists()))
        .unwrap_or(false);
    check(diag_a, "client A's diagnostics bundle was written to farm/diag".into());
    let events = std::fs::read_to_string(out.join("farm").join("events.jsonl")).unwrap_or_default();
    check(events.contains("REMOVED farmtest-A"), "the event log records A's removal".into());
    check(
        b_killed && (events.contains("farmtest-B: connection closed") || events.contains("farmtest-B is unreachable")),
        "the event log records B's departure after it was killed".into(),
    );
    let a_corrupted = a.log.iter().any(|l| l.contains("sent corrupted on purpose"));
    println!("farmtest: {:.1}s", t0.elapsed().as_secs_f64());
    if !b_killed || !a_corrupted {
        println!(
            "farmtest: VACUOUS — {}",
            if !b_killed { "client B never sent a frame, so it was never killed mid-run" } else { "client A never sent a corrupted frame, so verification was never exercised" }
        );
        return Ok(2);
    }
    if fails.is_empty() {
        println!("farmtest: PASS");
        Ok(0)
    } else {
        println!("farmtest: FAIL ({} check(s))", fails.len());
        Ok(1)
    }
}
