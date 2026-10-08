//! `--render-tour TOUR --gpus all` (or `--gpus N,N,…`): one tour on every graphics card of THIS
//! machine (design/multi-gpu.md Phase 1). It is a render farm on loopback — a controller
//! (`--farm-render`) and one client with a session per card (`--adapters`), both child processes of
//! this one — that otherwise behaves like a single-GPU `--render-tour`: the same frame files in the
//! same folder, the same `frame K/N` progress lines (the Render tour window reads them), the
//! folder's `render-status.txt`, and the mp4. Both children read their stdin from this process
//! (`status::Link`), so however it ends — finished, stopped from the window, killed — the controller
//! stops and the client leaves.
//!
//! Refused rather than ignored: what a farm job does not have (`--segment`, `--segments`,
//! `--frames`, `--order`, `--dry-run`, the normalize-anchor flags), and the overrides a single
//! render takes from its command line but a farm takes from this machine's settings (`--watermark`,
//! `--show-location`, `--bla`, `--set`). Different card models draw a few pixels differently
//! (design/multi-gpu.md §2.6); the farm keeps a held shot and a dissolve on one card. The children log as guests of this
//! process (`logs/guests/…`); the client keeps its identity and job folders in
//! `<config>/farm/local-gpus/client/`.

use super::status::{ClientPhase, ClientStatus, ControllerPhase, ControllerStatus, Link};
use super::{build_identity, farm_dir, is_dirty, load_key, machine_name, value};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The flag.
pub(crate) const FLAG: &str = "--gpus";

/// How long every card's session may take to join (probe render and self-check) before the render
/// gives up rather than wait for a card that will never come.
const JOIN_WAIT: Duration = Duration::from_secs(300);

/// Options a `--gpus` render refuses, with why.
const REFUSED: [(&str, &str); 16] = [
    ("--segment", "a farm renders the whole tour"),
    ("--segments", "a farm renders the whole tour"),
    ("--segment-index", "a farm renders the whole tour"),
    ("--frames", "a farm renders the whole tour"),
    ("--order", "a farm hands out its own runs of frames"),
    ("--dry-run", "a farm has no dry run"),
    ("--norm-anchors", "a farm measures the normalize ranges itself"),
    ("--dump-norm-anchors", "a farm measures the normalize ranges itself"),
    ("--farm-child", "it is for a farm's own render processes"),
    ("--watermark", "a farm renders with this machine's saved settings; set it in the app"),
    ("--no-watermark", "a farm renders with this machine's saved settings; set it in the app"),
    ("--show-location", "a farm renders with this machine's saved settings; set it in the app"),
    ("--hud", "a farm renders with this machine's saved settings; set it in the app"),
    ("--bla", "a farm renders with this machine's saved settings; set it in the app"),
    ("--no-bla", "a farm renders with this machine's saved settings; set it in the app"),
    ("--set", "a farm needs stock tunables"),
];

/// What `--gpus` asked for.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Gpus {
    /// Every graphics card (Vulkan, real hardware: what `--adapters all` takes).
    All,
    /// These `--list-adapters` numbers, one session each (a number may repeat: two sessions on one
    /// card, which is how a one-card machine tests the whole path).
    List(Vec<usize>),
}

pub(crate) fn parse_gpus(spec: &str) -> Result<Gpus, String> {
    let s = spec.trim();
    if s.eq_ignore_ascii_case("all") {
        return Ok(Gpus::All);
    }
    let list: Result<Vec<usize>, _> = s.split(',').map(|p| p.trim().parse::<usize>()).collect();
    match list {
        Ok(v) if !v.is_empty() && v.iter().all(|&n| (1..100).contains(&n)) => Ok(Gpus::List(v)),
        _ => Err(format!("--gpus takes all, or card numbers from --list-adapters separated by commas (got \"{spec}\")")),
    }
}

/// Run it, if `--gpus` is on the command line of a `--render-tour`. `None`: not asked for, or only
/// one card, so the ordinary single-GPU render goes ahead. `Some(code)` otherwise.
pub(crate) fn run(args: &[String]) -> Option<i32> {
    if !args.iter().any(|a| a == FLAG) || !args.iter().any(|a| a == "--render-tour") {
        return None;
    }
    let spec = value(args, FLAG)?;
    match run_inner(args, spec) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("fractadyne: {e}");
            Some(2)
        }
    }
}

fn run_inner(args: &[String], spec: &str) -> Result<Option<i32>, String> {
    if let Some((f, why)) = REFUSED.iter().find(|(f, _)| args.iter().any(|a| a == f)) {
        return Err(format!("{f} cannot be used with --gpus: {why}."));
    }
    let tour = PathBuf::from(value(args, "--render-tour").ok_or("--gpus needs --render-tour FILE")?);
    let (adapters, n) = match parse_gpus(spec)? {
        Gpus::All => {
            let cards = crate::ui::farm_client::list_cards("local-gpus");
            if cards.len() < 2 {
                say(&format!("--gpus all: this machine has {} graphics card(s) — rendering on one, as usual.", cards.len()));
                return Ok(None);
            }
            ("all".to_string(), cards.len())
        }
        Gpus::List(v) if v.len() < 2 => {
            say(&format!("--gpus {spec} names one graphics card — rendering on it, as usual."));
            return Ok(None);
        }
        Gpus::List(v) => (v.iter().map(|k| k.to_string()).collect::<Vec<_>>().join(","), v.len()),
    };

    // The tour exactly as `--render-tour` resolves it: the folder, the names, the mp4.
    let text = std::fs::read_to_string(&tour).map_err(|e| format!("{}: {e}", tour.display()))?;
    let pb = crate::scripting::parse_tour_text(&text).map_err(|e| format!("{}: {e}", tour.display()))?;
    let (sw, sh) = value(args, "--size").map(|s| crate::arg_size("--size", s)).unwrap_or((None, None));
    let cli = crate::scripting::TourRenderConfig {
        fps: super::number(args, "--fps"),
        width: sw,
        height: super::number(args, "--height").or(sh),
        ss: super::number(args, "--ss"),
        prefix: value(args, "--prefix").map(str::to_string),
        out: value(args, "--out").or_else(|| value(args, "-o")).map(PathBuf::from),
        mp4: args
            .iter()
            .position(|a| a == "--mp4")
            .map(|i| args.get(i + 1).filter(|s| !s.starts_with('-')).map(PathBuf::from)),
        ..Default::default()
    };
    let r = cli.resolve(&pb.render, &tour);
    let frames = crate::scripting::tour_frame_count(pb.total, r.fps);
    fractadyne_farm::names::check_file_part("frame prefix", &r.prefix)?;

    // Frames already in the folder: the single render's rule. The controller adopts every complete
    // frame it finds, so `--overwrite` must clear them (and the folder's farm state) first.
    let overwrite = args.iter().any(|a| a == "--overwrite" || a == "-y");
    let resume = args.iter().any(|a| a == "--resume");
    let existing: Vec<PathBuf> = (0..frames)
        .map(|i| r.out.join(fractadyne_farm::names::frame_file_name(&r.prefix, i)))
        .filter(|p| p.exists())
        .collect();
    if !existing.is_empty() && !overwrite && !resume {
        return Err(format!(
            "{} already exists; pass --overwrite (or -y) to replace the frames, --resume to keep them and render the rest, or use an empty --out directory",
            existing[0].display()
        ));
    }
    if overwrite {
        for p in &existing {
            std::fs::remove_file(p).map_err(|e| format!("{}: {e}", p.display()))?;
        }
        let state = r.out.join("farm");
        if state.exists() {
            std::fs::remove_dir_all(&state).map_err(|e| format!("{}: {e}", state.display()))?;
        }
    }
    std::fs::create_dir_all(&r.out).map_err(|e| format!("{}: {e}", r.out.display()))?;

    crate::scripting::write_render_status(&r.out, "running");
    let started = Instant::now();
    let result = render(args, &tour, &r, &adapters, n);
    match &result {
        Ok(()) => crate::scripting::write_render_status(&r.out, "complete"),
        Err(e) => crate::scripting::write_render_status(&r.out, &format!("failed: {e}")),
    }
    match result {
        Ok(()) => {
            say(&format!(
                "Rendered {frames} frame(s) in {} on {n} graphics cards → {}",
                crate::scripting::fmt_hms(started.elapsed().as_secs_f64()),
                r.out.display()
            ));
            say(&crate::scripting::assemble_mp4(&r.out, &r.prefix, r.fps, r.mp4.as_deref()));
            Ok(Some(0))
        }
        Err(e) => {
            eprintln!("fractadyne: render FAILED: {e}");
            Ok(Some(1))
        }
    }
}

/// The farm itself: start the controller, then the client once it listens; relay progress; end
/// both.
fn render(
    args: &[String],
    tour: &Path,
    r: &crate::scripting::ResolvedTourRender,
    adapters: &str,
    n: usize,
) -> Result<(), String> {
    // A key and a client of their own, kept between runs: the controller pins the client's name to
    // its identity (`known-clients.toml`), so a client made afresh each time would be refused the
    // second time. The user's own farm key is neither read nor made.
    let base = farm_dir()?.join("local-gpus");
    let key = base.join("farm-key.txt").to_string_lossy().into_owned();
    load_key(&["--farm-key-file".to_string(), key.clone()], true)?;
    let name = format!("{} (local)", machine_name(args));
    // One executable plays every part, so the build gate has nothing to compare; a farm refuses a
    // build with uncommitted changes unless BOTH ends say they allow one.
    let dirty = is_dirty(build_identity().1);
    // ⚠No FRACTADYNE_LOG_DIR for either child: their own render processes inherit it, and one the
    // client stops when the job closes then left its "still running" marker where the next run's
    // client reported it as a crash. Started beside this process, each logs as a guest
    // (`logs/guests/…`), and the client's render processes in their job folders, as in any farm.

    let mut ca: Vec<String> = vec![
        "--farm-render".into(),
        tour.to_string_lossy().into_owned(),
        "--out".into(),
        r.out.to_string_lossy().into_owned(),
        "--listen".into(),
        "127.0.0.1:0".into(),
        "--no-discovery".into(),
        "--min-clients".into(),
        n.to_string(),
        "--farm-key-file".into(),
        key.clone(),
        "--name".into(),
        name.clone(),
        "--size".into(),
        format!("{}x{}", r.width, r.height),
        "--fps".into(),
        r.fps.to_string(),
        "--ss".into(),
        r.ss.to_string(),
        "--prefix".into(),
        r.prefix.clone(),
        super::status::FLAG.into(),
    ];
    if let Some(v) = value(args, "--sharing") {
        ca.extend(["--sharing".into(), v.to_string()]);
    }
    for f in ["--prebuild", "--no-prebuild"] {
        if args.iter().any(|a| a == f) {
            ca.push(f.into());
        }
    }
    if dirty {
        ca.push("--farm-allow-dirty".into());
    }
    say(&format!("Rendering \"{}\" on {n} graphics cards ({adapters}) through a render farm on this machine…", tour.display()));
    let mut ctl = Link::<ControllerStatus>::spawn(&ca, &[])?;

    // The controller's port, from its first status line.
    let t = Instant::now();
    let port = loop {
        ctl.poll();
        relay(&mut ctl.log);
        if let Some(p) = ctl.status.as_ref().map(|s| s.port).filter(|&p| p != 0) {
            break p;
        }
        if !ctl.running() || t.elapsed() > Duration::from_secs(60) {
            ctl.kill();
            return Err(format!("the farm controller did not start: {}", tail(&ctl.log)));
        }
        std::thread::sleep(Duration::from_millis(100));
    };

    let mut la: Vec<String> = vec![
        "--render-client".into(),
        format!("127.0.0.1:{port}"),
        "--farm-key-file".into(),
        key,
        "--adapters".into(),
        adapters.into(),
        "--one-job".into(),
        "--name".into(),
        name,
        super::status::FLAG.into(),
    ];
    if dirty {
        la.push("--farm-allow-dirty".into());
    }
    let env = [("FRACTADYNE_CONFIG_DIR", base.join("client").into_os_string())];
    let mut cl = match Link::<ClientStatus>::spawn(&la, &env) {
        Ok(c) => c,
        Err(e) => {
            ctl.send("stop");
            return Err(format!("the render client did not start: {e}"));
        }
    };

    let begun = Instant::now();
    let mut last = (u64::MAX, Instant::now() - Duration::from_secs(10));
    let mut why_stopped: Option<String> = None;
    let mut client_gone: Option<Instant> = None;
    loop {
        ctl.poll();
        cl.poll();
        relay(&mut ctl.log);
        // The client's own lines are mostly its sessions' chatter; keep what explains a failure.
        cl.log.retain(|l| l.starts_with("! "));
        relay(&mut cl.log);
        if let Some(s) = &ctl.status {
            // `frame K/N` as a single render prints it: what the Render tour window's bar reads.
            if s.done != last.0 && (last.1.elapsed() >= Duration::from_secs(1) || s.done >= s.frames) && s.frames > 0 {
                let rate = s.frames_per_s;
                say(&format!(
                    "  frame {}/{}  ({} elapsed, {} left, {rate:.2} fps)",
                    s.done,
                    s.frames,
                    crate::scripting::fmt_hms(begun.elapsed().as_secs_f64()),
                    crate::scripting::fmt_hms(s.eta_s.unwrap_or(0.0)),
                ));
                last = (s.done, Instant::now());
            }
            if why_stopped.is_none() && s.phase == ControllerPhase::Waiting && begun.elapsed() > JOIN_WAIT {
                why_stopped = Some(format!(
                    "not every graphics card joined within {} s ({} of {n}): {}",
                    JOIN_WAIT.as_secs(),
                    s.clients.iter().filter(|c| !c.removed).count(),
                    sessions_text(&cl)
                ));
                ctl.send("stop");
            }
        }
        if !ctl.running() {
            break;
        }
        // Every session ended. `--one-job` sessions leave as the job closes, a moment before the
        // controller exits, so give it that moment; still running after it, the job lost its
        // renderers and nothing would render the rest.
        if !cl.running() {
            let gone = *client_gone.get_or_insert_with(Instant::now);
            if why_stopped.is_none() && gone.elapsed() > Duration::from_secs(15) {
                why_stopped = Some(format!("the render client ended (exit {:?}): {}", cl.exit.flatten(), sessions_text(&cl)));
                ctl.send("stop");
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    // The job is over; `--one-job` sessions leave on their own, and this makes sure.
    cl.send("leave");
    let t = Instant::now();
    while cl.running() && t.elapsed() < Duration::from_secs(20) {
        cl.poll();
        std::thread::sleep(Duration::from_millis(100));
    }
    if cl.running() {
        cl.kill();
    }
    relay(&mut ctl.log);
    match (why_stopped, ctl.exit.flatten()) {
        (Some(why), _) => Err(why),
        (None, Some(0)) => Ok(()),
        (None, Some(3)) => Err(format!(
            "some frames could not be rendered — see {} (--resume renders them again)",
            r.out.join("farm").join("events.jsonl").display()
        )),
        (None, Some(4)) => Err("stopped before the end (--resume continues it)".into()),
        (None, code) => Err(format!("the farm controller exited with {code:?}: {}", tail(&ctl.log))),
    }
}

/// Print, and drop, the lines a child said since the last call. To stdout only: each child keeps
/// its own log, and a child's line in THIS log would be judged as this process's (a client's report
/// of a crash failed this render's own log check).
fn relay(log: &mut std::collections::VecDeque<String>) {
    use std::io::Write;
    let mut out = std::io::stdout().lock();
    while let Some(l) = log.pop_front() {
        let _ = writeln!(out, "  [farm] {l}"); // a closed pipe must not end the render (`scripting::say`)
    }
}

/// The last few lines a child said, for an error message.
fn tail(log: &std::collections::VecDeque<String>) -> String {
    let v: Vec<&str> = log.iter().rev().take(4).map(String::as_str).collect();
    if v.is_empty() {
        "it said nothing".into()
    } else {
        v.into_iter().rev().collect::<Vec<_>>().join(" | ")
    }
}

/// Each card's session in a few words, for an error message.
fn sessions_text(cl: &Link<ClientStatus>) -> String {
    let words: Vec<String> = cl
        .all
        .values()
        .map(|s| {
            let phase = match s.phase {
                ClientPhase::Ended => "ended",
                ClientPhase::Checking => "checking",
                ClientPhase::Connecting => "connecting",
                ClientPhase::Retrying => "retrying",
                _ => "joined",
            };
            format!("{}: {phase} ({})", s.name, s.detail)
        })
        .collect();
    if words.is_empty() {
        "no session reported".into()
    } else {
        words.join("; ")
    }
}

fn say(msg: &str) {
    crate::scripting::say(msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpus_takes_all_or_card_numbers() {
        assert_eq!(parse_gpus("all"), Ok(Gpus::All));
        assert_eq!(parse_gpus(" ALL "), Ok(Gpus::All));
        assert_eq!(parse_gpus("1,2"), Ok(Gpus::List(vec![1, 2])));
        // The same card twice: two sessions on one card, the one-card machine's test of the path.
        assert_eq!(parse_gpus("1, 1"), Ok(Gpus::List(vec![1, 1])));
        for bad in ["", "0", "1,", "two", "1;2", "6800"] {
            assert!(parse_gpus(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn every_refused_option_says_why() {
        for (f, why) in REFUSED {
            assert!(f.starts_with("--") && !why.is_empty(), "{f}");
        }
    }
}
