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
# A held shot (frames 6–10) that the picture arrives in by dissolving: frames 6–9 blend with frame
# 5, rendered in the same process, so a run starting inside the dissolve would render them as a
# hard cut and the identity check against the reference would fail. (A transition is clamped to
# its keyframe's hold: without the hold, the dissolve is silently zero-length.)
hold = 1.5
transition = "dissolve"
transition_secs = 1.0

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
    /// Piped only for a process run under `--ui-status`.
    stdin: Option<std::process::ChildStdin>,
}

impl Proc {
    fn spawn(name: &'static str, args: &[String], cfg: &Path, env: &[(&str, &str)]) -> Result<Self, String> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(cfg).map_err(|e| e.to_string())?;
        let mut cmd = std::process::Command::new(exe);
        let piped = args.iter().any(|a| a == super::status::FLAG);
        cmd.args(args)
            .env("FRACTADYNE_CONFIG_DIR", cfg)
            .env("FRACTADYNE_NO_SOUND", "1")
            .env_remove(super::client::CORRUPT_INSTRUMENT)
            .stdin(if piped { std::process::Stdio::piped() } else { std::process::Stdio::null() })
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
        let stdin = child.stdin.take();
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
        Ok(Self { name, child, lines: rx, log: Vec::new(), stdin })
    }

    /// Send a `--ui-status` command.
    fn send(&mut self, cmd: &str) {
        use std::io::Write;
        if let Some(s) = self.stdin.as_mut() {
            let _ = writeln!(s, "{cmd}");
            let _ = s.flush();
        }
    }

    /// Drain what it has printed; `true` if any new line matched `pat`.
    fn drain(&mut self, pat: &str) -> bool {
        let mut hit = false;
        for l in self.lines.try_iter() {
            if l.contains(pat) {
                hit = true;
            }
            // Status lines are kept for the verdict, not shown: one a second, and long.
            if !l.starts_with(super::status::PREFIX) {
                println!("    [{}] {}", self.name, l);
            }
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
            s(super::status::FLAG),
            // Share mode: client C has the same "shared drive" (a folder here) and writes there.
            s("--share-root"),
            p(&base.join("share")),
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
    // `ui`: started as the app's Render client window starts it (which also gives it the shared
    // drive, so its frames go through share mode).
    let client = |name: &'static str, env: &[(&str, &str)], ui: bool| {
        let mut args = if ui {
            // Exactly the command line the app's Render client window starts.
            let settings = crate::ui::farm_client::ClientSettings {
                controller: addr.clone(),
                name: format!("farmtest-{name}"),
                share_root: base.join("share").to_string_lossy().into_owned(),
                ..Default::default()
            };
            crate::ui::farm_client::client_args(&settings, &key_file)
        } else {
            vec![s("--render-client"), addr.clone(), s("--farm-key-file"), p(&key_file), s("--name"), format!("farmtest-{name}")]
        };
        args.push(s("--farm-allow-dirty"));
        Proc::spawn(name, &args, &base.join(format!("cfg-{name}")), env)
    };
    // A corrupts its first two frames — on the shared drive, after their digests were announced.
    let mut a = client("A", &[(super::client::CORRUPT_INSTRUMENT, "2")], true)?;
    let mut b = client("B", &[], false)?;
    // C as the app's Render client window runs it — its very command line: status lines out,
    // commands in.
    let mut c = client("C", &[], true)?;

    // 4. Run, killing B once it has sent a frame; pause the job from "the window" (stdin) as soon as
    // it renders, and resume it once the status says paused.
    let mut b_killed = false;
    let mut pause_step = 0; // 0 → pause sent (1) → paused seen, resume sent (2) → rendering again (3)
    let mut seen = 0;
    let deadline = Instant::now() + Duration::from_secs(300);
    let code = loop {
        ctl.drain("");
        for l in &ctl.log[seen..] {
            if let Some(st) = super::status::parse::<super::status::ControllerStatus>(l) {
                use super::status::ControllerPhase as P;
                match (pause_step, st.phase) {
                    (0, P::Rendering) => pause_step = 1,
                    (1, P::Paused) => pause_step = 2,
                    (2, P::Rendering | P::Finished) => pause_step = 3,
                    _ => {}
                }
            }
        }
        seen = ctl.log.len();
        match pause_step {
            1 if !ctl.log.iter().any(|l| l.contains("job paused")) => {
                println!("  → pausing the job through stdin");
                ctl.send("pause");
            }
            2 if !ctl.log.iter().any(|l| l.contains("job resumed")) => {
                println!("  → resuming it");
                ctl.send("resume");
            }
            _ => {}
        }
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
    // C leaves when told, as the window's Disconnect does — not by being killed.
    let leave_t = Instant::now();
    c.send("leave");
    let c_exit = loop {
        if let Ok(Some(st)) = c.child.try_wait() {
            break st.code();
        }
        if leave_t.elapsed() > Duration::from_secs(10) {
            break None;
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let leave_s = leave_t.elapsed().as_secs_f64();
    for pr in [&mut a, &mut b, &mut c] {
        if pr.running() {
            let _ = pr.child.kill();
        }
        let _ = pr.child.wait();
        std::thread::sleep(Duration::from_millis(100));
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
    // Every kept frame says where its reference came from (design §8's measure).
    let sources: Vec<Option<String>> = done.lines().filter_map(|l| serde_json::from_str::<fractadyne_farm::manifest::DoneRecord>(l).ok()).map(|d| d.reference).collect();
    let fresh = sources.iter().filter(|r| r.as_deref() == Some("fresh")).count();
    check(
        !sources.is_empty() && sources.iter().all(|r| r.is_some()) && fresh >= 1,
        format!("every frame says where its reference came from ({} of {}, {fresh} built fresh)", sources.iter().filter(|r| r.is_some()).count(), sources.len()),
    );
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
    // The window's side: status lines, commands, the probes.
    let statuses: Vec<super::status::ControllerStatus> = ctl.log.iter().filter_map(|l| super::status::parse(l)).collect();
    let last = statuses.last();
    check(
        statuses.len() >= 5 && last.is_some_and(|s| s.phase == super::status::ControllerPhase::Finished && s.exit_code == Some(0) && s.done == FRAMES),
        format!(
            "the controller's status lines parse ({}), the last says finished, exit {:?}, {} of {FRAMES} done",
            statuses.len(),
            last.and_then(|s| s.exit_code),
            last.map_or(0, |s| s.done)
        ),
    );
    check(pause_step == 3, format!("pause and resume through stdin took effect (step {pause_step} of 3)"));
    // (stdout only: each note is echoed to stderr as a `[fd-farm]` log line too)
    let identical_probes = ctl.log.iter().filter(|l| !l.starts_with("! ") && l.contains("probe identical to this machine's")).count();
    check(identical_probes == 3, format!("every client's probe matched this machine's ({identical_probes} of 3)"));
    let probes: Vec<String> = std::fs::read_dir(out.join("farm").join("probes")).map(|rd| rd.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect()).unwrap_or_default();
    check(probes.len() == 4 && probes.iter().any(|n| n == "this-machine.png"), format!("farm/probes holds this machine's probe and the clients' ({} files)", probes.len()));
    let classes: Vec<Option<String>> = statuses.iter().rev().find(|s| s.clients.len() >= 3).map(|s| s.clients.iter().map(|c| c.gpu_class.clone()).collect()).unwrap_or_default();
    check(classes.len() >= 3 && classes.iter().all(|c| c.as_deref() == Some("A")), format!("one GPU class, A, for every client ({classes:?})"));
    check(events.contains("probe identical"), "the event log kept the notes from before the job started (the probes)".into());
    // The GPU comparison READ something: this machine's GPU and every client's driver are known —
    // and, all on this one GPU, none was called different. (Without the reads, "no warning" would
    // pass by comparing nothing.)
    let with_rows = statuses.iter().rev().find(|s| s.clients.len() >= 3);
    let this_gpu = with_rows.and_then(|s| s.this_gpu.clone());
    let drivers: Vec<String> = with_rows.map(|s| s.clients.iter().map(|c| c.driver.clone()).collect()).unwrap_or_default();
    let called_different = ctl.log.iter().any(|l| l.contains("differ slightly from this machine's") && l.contains("has a") || l.contains("has the same GPU"));
    check(
        this_gpu.is_some() && drivers.len() >= 3 && drivers.iter().all(|d| !d.is_empty()) && !called_different && with_rows.is_some_and(|s| s.clients.iter().all(|c| c.gpu_note.is_none())),
        format!("this machine's GPU ({}) and every client's driver ({drivers:?}) were read, and none was called different", this_gpu.as_deref().unwrap_or("not read")),
    );
    // Share mode: C's frames came through the shared drive, and its folders there are gone.
    let c_shared = ["farmtest-A", "farmtest-C"].iter().all(|n| ctl.log.iter().any(|l| !l.starts_with("! ") && l.contains(&format!("{n} writes its frames to the shared drive"))));
    let leftovers: Vec<String> = std::fs::read_dir(base.join("share").join("fractadyne-farm"))
        .map(|rd| rd.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect())
        .unwrap_or_default();
    check(c_shared && leftovers.is_empty(), format!("clients A and C delivered through the shared drive (A's two bad copies caught there), and the job's folders there were removed (left: {leftovers:?})"));
    let cst: Vec<super::status::ClientStatus> = c.log.iter().filter_map(|l| super::status::parse(l)).collect();
    let rendered = cst.iter().any(|x| x.phase == super::status::ClientPhase::Rendering);
    let c_last = cst.last();
    check(
        c_exit == Some(0) && leave_s < 5.0 && rendered && c_last.is_some_and(|x| x.phase == super::status::ClientPhase::Ended && x.exit_code == Some(0) && x.frames_done > 0),
        format!(
            "client C (under --ui-status) reported rendering, left on \"leave\" in {leave_s:.1}s with exit {c_exit:?}, its last status {:?} after {} frame(s)",
            c_last.map(|x| x.phase),
            c_last.map_or(0, |x| x.frames_done)
        ),
    );
    // "sent corrupted" when streamed, "left corrupted" on the shared drive.
    let a_corrupted = a.log.iter().any(|l| l.contains("corrupted on purpose"));
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
