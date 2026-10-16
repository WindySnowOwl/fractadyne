//! Headless CLI modes, run before the GUI starts: --find-minibrot, --compare,
//! --crosscheck-f3, and --validate-deep. `run_headless` returns true when it handled a
//! mode (so `main` exits Ok); the validation modes set a process exit code directly.

use crate::{version_string, FractalKind};

/// Windows- and Unix-style ways to explicitly ask for help.
const HELP_TOKENS: &[&str] = &["--help", "-h", "-?", "/?", "/h", "/help", "help"];

/// The pixels-per-point every `--farm-child` render runs at, whatever the machine's display scale,
/// so its burned-in watermark is rasterized identically on every machine of a farm. 2, not 1: the
/// mark is drawn at 2.6% of the frame height and its raster is about 1.6 × 40 × this many pixels
/// tall, so 2 keeps it a DOWNSCALE (sharp) up to 8K frames, where 1 would be enlarged past ~4K.
pub(crate) const FARM_PIXELS_PER_POINT: f32 = 2.0;

/// The set of known long options (`--xxx`), harvested from the shared CLI reference so it can never
/// drift from what `--help` documents (scans both the flag column and descriptions, so options only
/// mentioned in a parenthetical — e.g. `--zoom-f3`, `--er` — are still recognized).
fn known_long_flags() -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    for entry in crate::help::CLI_REFERENCE {
        let (a, b) = match entry {
            crate::help::CliRef::Flag(f, d) => (*f, *d),
            _ => continue,
        };
        for field in [a, b] {
            for tok in field.split(|c: char| c.is_whitespace() || matches!(c, ',' | '(' | ')' | '[' | ']')) {
                if tok.starts_with("--") && tok.len() > 2 {
                    set.insert(tok.to_string());
                }
            }
        }
    }
    set
}

/// The single-dash tokens that are legitimately options (everything else long-form takes two
/// dashes). `-o`/`-y` are documented shorthands, the rest are help/version conventions; negative
/// numbers (`-0.5` for `--center`) are excluded by the numeric parse in the classifier.
const SHORT_FLAGS: &[&str] = &["-h", "-?", "-V", "-o", "-y"];

/// Options whose values are free-form text that may begin with a minus sign and still not read as
/// one number — an expression (`--center -3/4 0`), a formula (`--formula "-z^2 + c"`), a parameter
/// list (`--formula-params -0.5,0`) — with how many values follow. Those values are not options.
const VALUE_FLAGS: &[(&str, usize)] = &[("--center", 2), ("--julia-c", 2), ("--formula", 1), ("--formula-params", 1)];

/// What is wrong with a command line, if anything. Pure, so the rules are pinned by test —
/// a mis-typed option must never silently launch the GUI (field case 2026-08-20: `-play tour`
/// booted the saved session instead of the tour, and a validation run measured the wrong thing).
#[derive(Debug, PartialEq, Eq)]
enum BadOption {
    /// `--xyz` that matches nothing in the CLI reference.
    UnknownLong(String),
    /// A single-dash spelling of a real long option: `-play` for `--play`.
    SingleDash { given: String, correct: String },
    /// A single-dash token that is neither a known short flag, a number, nor a long option.
    UnknownShort(String),
}

fn first_bad_option(args: &[String]) -> Option<BadOption> {
    let known = known_long_flags();
    let mut values = 0usize;
    for s in args.iter().skip(1) {
        let a = s.as_str();
        if values > 0 {
            values -= 1;
            continue;
        }
        if let Some(&(_, n)) = VALUE_FLAGS.iter().find(|(f, _)| *f == a) {
            values = n;
            continue;
        }
        if a.starts_with("--") && a.len() > 2 {
            if !known.contains(a) {
                return Some(BadOption::UnknownLong(a.to_string()));
            }
        } else if a.starts_with('-') && a.len() > 1 && !a.starts_with("--") {
            if SHORT_FLAGS.contains(&a) || a.parse::<f64>().is_ok() {
                continue; // documented shorthand, or a negative numeric value
            }
            let long = format!("-{a}"); // "-play" -> "--play"
            if known.contains(long.as_str()) {
                return Some(BadOption::SingleDash { given: a.to_string(), correct: long });
            }
            return Some(BadOption::UnknownShort(a.to_string()));
        }
    }
    None
}

#[cfg(test)]
#[path = "../tests/unit/cli/bad_options.rs"]
mod bad_options;

/// A harness flag that names a tour FILE but was given none. `--livetest` / `--divetest` take the
/// tour as the FOLLOWING argument (`--livetest tours/grand-tour.toml`). Given none, the value parses
/// as `None`, so `launched_for_a_task` is never set — and the app then opens an ORDINARY GUI window
/// instead of running the harness: the welcome dialog appears, the harness never starts, and on an
/// occluded window the frame loop stalls on present for tens of seconds. That is exactly how a bare
/// `--livetest` wedged a gate for minutes on 2026-09-10, masquerading as a livetest bug. Caught here
/// so it fails loudly with the right invocation instead of a silent no-op GUI.
pub(crate) fn missing_tour_flag(args: &[String]) -> Option<&'static str> {
    for &flag in &["--livetest", "--divetest"] {
        if let Some(pos) = args.iter().position(|a| a == flag) {
            // Satisfied only by a following non-flag token (the tour path).
            let has_file = args.get(pos + 1).is_some_and(|a| !a.starts_with('-'));
            if !has_file {
                return Some(flag);
            }
        }
    }
    None
}

#[cfg(test)]
#[path = "../tests/unit/cli/missing_tour_flag_tests.rs"]
mod missing_tour_flag_tests;

/// Dispatch the headless CLI modes. Returns true if one ran (caller should exit).
pub(crate) fn run_headless(args: &[String]) -> bool {
    // A tour-harness flag with no tour file would otherwise fall through to a bare GUI launch (see
    // `missing_tour_flag`). Refuse it before any window opens.
    if let Some(flag) = missing_tour_flag(args) {
        eprintln!(
            "fractadyne: {flag} needs a tour file — e.g. `{flag} tours/grand-tour.toml`.\n\
             (Bare {flag} would open an ordinary window instead of running the harness.)"
        );
        crate::exit(2);
    }
    // Explicit help request (--help / -h / -? / /? / /h / /help / help) → reference to stdout, exit 0.
    if args.iter().skip(1).any(|a| HELP_TOKENS.contains(&a.as_str())) {
        print!("{}", crate::help::cli_help_text());
        return true;
    }
    // `--version` / `-V`: the first thing anyone types at a new binary. One line, exit 0 —
    // conventional shape (`fractadyne 0.2.40-beta.50 (build 1319)`), greppable by scripts.
    if args.iter().skip(1).any(|a| a == "--version" || a == "-V") {
        println!("fractadyne {}", version_string());
        return true;
    }
    // `--torture`: the escalating failure-hunting suite (design/torture-suite.md). Handled here,
    // headless and before any GPU work, because the SUPERVISOR needs no device of its own — it
    // launches each rung as a child process precisely so that a rung which loses the device or
    // wedges cannot take the runner down with it.
    if args.iter().any(|a| a == "--torture") {
        crate::exit(crate::harness::torture::run(args));
    }
    // The render farm (design/remote-rendering.md): the controller, a client, and their harness.
    // Headless for the same reason as `--torture` — they render nothing themselves; every frame
    // comes from a `--render-tour --farm-child` process they start.
    if let Some(code) = crate::farm::run_headless(args) {
        crate::exit(code);
    }
    // `--logcheck [LOG]`: hold a log (default: this app's own) to validation/logcheck-rules.toml.
    // Headless and deviceless, so it can judge the log a killed or wedged run left behind.
    if args.iter().any(|a| a == "--logcheck") {
        crate::exit(crate::harness::logcheck::run_cli(args));
    }
    // Print the tour-script schema reference (Markdown) and exit — used to (re)generate TOURS.md.
    if args.iter().any(|a| a == "--dump-tour-schema") {
        print!("{}", crate::scripting::tour_schema_markdown());
        return true;
    }
    // Print the frame record's encoding (JSON) and exit — used to (re)generate
    // validation/frame-schema.json, which scripts/framelog.py reads.
    if args.iter().any(|a| a == "--dump-frame-schema") {
        print!("{}", crate::diag::frame_record::schema_json());
        return true;
    }
    // Recompute every decision in a FRACTADYNE_RECORD_DECIDE recording and diff it against the
    // recorded one (design/live-render-robustness.md §5.4, W5). CPU only; exit 0 = identical.
    if let Some(pos) = args.iter().position(|a| a == "--replay") {
        match args.get(pos + 1).filter(|a| !a.starts_with('-')) {
            Some(path) => crate::render::decide::run_replay(path),
            None => {
                eprintln!("--replay: name the recording (a FRACTADYNE_RECORD_DECIDE .jsonl)");
                crate::exit(2);
            }
        }
    }
    // Check GitHub for a newer release, print the result, and exit (validates the in-app update
    // check headlessly; handy for automation). Optional track: `--check-updates beta` or
    // `--check-updates=beta`; with none named it follows the build's own track — beta for a
    // pre-release build, stable otherwise, the same rule a fresh install applies. Set
    // FRACTADYNE_FAKE_VERSION to pretend the running build is an older/newer version — the only
    // way to exercise the "update available" branch while the dev build is ahead of the latest
    // release (it moves this default too, since the pretend version decides the track).
    if let Some(pos) = args.iter().position(|a| a == "--check-updates" || a.starts_with("--check-updates=")) {
        let track_arg = args[pos]
            .split_once('=')
            .map(|(_, v)| v.to_string())
            .or_else(|| args.get(pos + 1).filter(|a| !a.starts_with('-')).cloned())
            .unwrap_or_default();
        let cur = crate::update::running_version();
        let track = if track_arg.is_empty() {
            crate::update::default_track_for(&cur)
        } else {
            crate::update::UpdateTrack::from_str(&track_arg)
        };
        println!("Checking {} track (running {cur})…", track.as_str());
        match crate::update::check(track, &cur) {
            crate::update::UpdateStatus::Available { version, url, prerelease } => {
                let channel = crate::update::channel_word(prerelease);
                println!("Update available: {version} ({channel})\n{url}");
            }
            crate::update::UpdateStatus::UpToDate => {
                println!("Up to date (no newer {} release than {cur}).", track.as_str());
            }
            crate::update::UpdateStatus::Error(e) => {
                eprintln!("Update check failed: {e}");
                crate::exit(1);
            }
        }
        return true;
    }
    // --gputest: verify the shader's extended-precision primitives against CPU oracles, on every
    // backend this machine offers. Headless (the test renders offscreen), so it belongs here
    // rather than in the app: no window flashes, it works over SSH, and it reaches backends the
    // windowed path cannot create a surface for. Exit 1 if any backend's arithmetic is unsound.
    // Optional `--out FILE` also writes the verbatim report (the Diagnostics dialog passes it so
    // the result can be opened and attached to a bug report); stdout is unchanged either way.
    if args.iter().any(|a| a == "--gputest") {
        let out = args
            .iter()
            .position(|a| a == "--out")
            .and_then(|i| args.get(i + 1))
            .map(std::path::PathBuf::from);
        let fails = crate::harness::gputest::run_gputest_sweep(out.as_deref());
        crate::exit(if fails > 0 { 1 } else { 0 });
    }
    // --discover: the render-farm controllers answering on this network (and this machine).
    if args.iter().any(|a| a == "--discover") {
        match fractadyne_farm::discovery::discover(std::time::Duration::from_millis(1500)) {
            Ok(found) if found.is_empty() => println!(
                "No render-farm controller answered. A controller answers while its farm is listening; a firewall at the controller must let UDP {} in.",
                fractadyne_farm::discovery::DISCOVERY_PORT
            ),
            Ok(found) => {
                for f in &found {
                    println!("{}  \"{}\" · {} {} · identity {}", f.address(), f.beacon.name, f.beacon.app_version, f.beacon.git, f.beacon.identity);
                }
            }
            Err(e) => {
                eprintln!("fractadyne: --discover: {e}");
                crate::exit(1);
            }
        }
        crate::exit(0);
    }
    // --list-adapters: the graphics adapters `--adapter` chooses among, numbered as it counts them.
    if args.iter().any(|a| a == crate::gpu_choice::LIST_FLAG) {
        print!("{}", crate::gpu_choice::listing(&crate::gpu_choice::list()));
        crate::exit(0);
    }

    // A bad option must never silently launch the GUI — report it and exit non-zero (like a
    // conventional CLI). The single-dash case gets a targeted one-liner instead of the full
    // reference: the correction IS the usage, and a page of help scrolling past is how an
    // error line goes unread.
    match first_bad_option(args) {
        Some(BadOption::SingleDash { given, correct }) => {
            eprintln!(
                "fractadyne: options take two dashes: '{given}' is not accepted — use '{correct}'"
            );
            eprintln!("fractadyne: run with --help for the full reference");
            crate::exit(2);
        }
        Some(BadOption::UnknownLong(bad)) | Some(BadOption::UnknownShort(bad)) => {
            eprintln!("fractadyne: unrecognized option '{bad}'\n");
            eprint!("{}", crate::help::cli_help_text());
            // Repeated after the reference on purpose: with a page of help above it, the
            // top line has already scrolled away.
            eprintln!("\nfractadyne: unrecognized option '{bad}' — see the reference above");
            crate::exit(2);
        }
        None => {}
    }
    // A value-taking option with its value missing is fatal, never a silent default — the same
    // family as the single-dash guard: `--log-dir` swallowing the next option, or silently
    // logging to the default location, is how a validation run's logs end up not where the
    // operator is watching.
    if let Some(i) = args.iter().position(|a| a == "--log-dir") {
        if !args.get(i + 1).is_some_and(|v| !v.starts_with('-')) {
            eprintln!("fractadyne: --log-dir needs a directory path, e.g. --log-dir D:\\logs");
            crate::exit(2);
        }
    }

    // --reset-state [-y|--yes]: permanently delete all persisted application state (session,
    // bookmarks, thumbnails), then exit. Warns and asks for confirmation on the terminal first
    // (type `reset`); -y/--yes skips the prompt for scripting. No console (double-click launch) ⇒
    // the read yields EOF ⇒ treated as "not confirmed", so nothing is deleted.
    if args.iter().any(|a| a == "--reset-state") {
        use std::io::Write;
        let assume_yes = args.iter().any(|a| a == "-y" || a == "--yes");
        eprintln!("This will PERMANENTLY delete all Fractadyne application state:");
        eprintln!("  {}", fractadyne_state::state_location_display());
        eprintln!("  (saved session, bookmarks, and bookmark thumbnails)");
        let confirmed = if assume_yes {
            true
        } else {
            eprint!("Type 'reset' to confirm (anything else cancels): ");
            let _ = std::io::stderr().flush();
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).is_ok() && line.trim().eq_ignore_ascii_case("reset")
        };
        if !confirmed {
            eprintln!("Cancelled — nothing was deleted.");
            return true;
        }
        match fractadyne_state::reset_all() {
            Ok(true) => println!("Application state reset."),
            Ok(false) => println!("No application state to reset (nothing was stored)."),
            Err(e) => {
                eprintln!("Reset failed: {e}");
                crate::exit(1);
            }
        }
        return true;
    }

    // --refdiag --center X Y --zoom-log2 L [--iter N]: sample reference orbit lengths across the
    // view. Answers whether long/interior references exist (multi-ref can help) or all escape early
    // (rebasing is inherent). Prints the distribution.
    if args.iter().any(|a| a == "--refdiag") {
        return crate::harness::refdiag::run(args);
    }

    // --pickcheck [FILE] [--size WxH] [--iter N]: the reference-pick redesign's acceptance
    // harness (design/pick-redesign.md). Runs BOTH phase-2 scoring engines — the original
    // bignum deep walk and the floatexp perturbation scorer — on identical inputs and asserts
    // they elect the SAME reference point. Bare = the built-in depth ladder (committed corpus /
    // validation locations, e17 → e4000; run from the repo root); FILE = one `.fdn` location.
    // Exit 0 = every rung ran and matched; 1 = a winner differed; 2 = a rung was skipped
    // (a skip is "not tested", never a pass).
    if args.iter().any(|a| a == "--pickcheck") {
        return crate::harness::pickcheck::run(args);
    }

    if args.iter().any(|a| a == "--find-minibrot") {
        let val = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1));
        let two = |name: &str| {
            args.iter()
                .position(|a| a == name)
                .and_then(|i| Some((args.get(i + 1)?, args.get(i + 2)?)))
        };
        let formula = match val("--fractal") {
            None => 0,
            Some(name) => match FractalKind::from_name(name) {
                Some(k) => k.formula_id(),
                None => {
                    eprintln!("fractadyne: --fractal: unknown family \"{name}\".");
                    crate::exit(2)
                }
            },
        };
        let center = match two("--center") {
            Some((x, y)) => {
                let (cx, cy) = crate::arg_center("--center", x, y);
                [cx, cy]
            }
            None => [
                fractadyne_core::BigFloat::from_f64(-0.5, 64),
                fractadyne_core::BigFloat::from_f64(0.0, 64),
            ],
        };
        // Same parser as `--render`, and for the same reason: a bare `f64::parse` with
        // `unwrap_or(1.0)` swallowed "1.0e23.9" (a fractional exponent is legal notation and is
        // what a zoom ladder produces) and searched at 1x instead, printing a confident nucleus
        // for the wrong place. The magnification stays a LOG from here on — `find_nucleus`
        // takes log2 since the f64 ceiling fell, so depths past 1e308x are ordinary inputs.
        // An unreadable value is still fatal, never a silent 1x.
        let mag_l2 = match val("--zoom") {
            None => 0.0,
            Some(z) => match crate::parse_zoom_to_log2(z) {
                Some(l2) if l2.is_finite() => l2,
                _ => {
                    eprintln!(
                        "fractadyne: --find-minibrot: cannot read \"{z}\" as a magnification."
                    );
                    crate::exit(2);
                }
            },
        };
        let search = match val("--period-search").map(String::as_str) {
            None | Some("closest") => fractadyne_core::PeriodSearch::ClosestApproach,
            Some("ball") => fractadyne_core::PeriodSearch::Ball,
            Some(other) => {
                eprintln!("fractadyne: --period-search: \"{other}\" is not one of: closest, ball.");
                crate::exit(2);
            }
        };
        let max_period = match val("--max-period") {
            None => 100_000,
            Some(v) => match v.parse::<u32>() {
                Ok(n) if n > 0 => n,
                _ => {
                    eprintln!("fractadyne: --max-period: cannot read \"{v}\" as a period.");
                    crate::exit(2);
                }
            },
        };
        match fractadyne_core::find_nucleus_by(&center, mag_l2, formula, max_period, search) {
            Some(n) => {
                println!(
                    "period {}\ncenter_x {}\ncenter_y {}",
                    n.period,
                    fractadyne_core::to_decimal_string(&n.cx),
                    fractadyne_core::to_decimal_string(&n.cy),
                );
                // Atom size / orientation and the depth a Newton-Raphson zoom would frame it at
                // — the same numbers the in-app "Find minibrot center" jump uses.
                let prec = fractadyne_core::precision_for_octaves(mag_l2.max(0.0).ceil() as u64);
                if let Some(a) =
                    fractadyne_core::nucleus_size(&n.cx, &n.cy, n.period, formula, prec)
                {
                    let zoom_l2 = crate::FractadyneApp::atom_frame_log2mag(a.log2_size);
                    println!(
                        "size_log2 {:.6}\nsize {:.6e}\norientation_deg {:.3}\nzoom_log2 {:.6}",
                        a.log2_size,
                        a.log2_size.exp2(),
                        a.orientation.to_degrees(),
                        zoom_l2,
                    );
                    // The center re-solved at the destination's precision — what the in-app jump
                    // actually navigates to. `find_nucleus` alone is only view-accurate.
                    let deep = fractadyne_core::precision_for_octaves(zoom_l2.max(0.0) as u64) + 64;
                    if let Some((rx, ry)) =
                        fractadyne_core::refine_nucleus(&n.cx, &n.cy, n.period, formula, deep)
                    {
                        let res =
                            fractadyne_core::nucleus_residual_log2(&rx, &ry, n.period, formula, deep)
                                .unwrap_or(f64::NAN);
                        println!(
                            "refined_x {}\nrefined_y {}\nresidual_log2 {:.1}",
                            fractadyne_core::to_decimal_string(&rx),
                            fractadyne_core::to_decimal_string(&ry),
                            res,
                        );
                    }
                }
            }
            None => println!("no minibrot center found"),
        }
        return true;
    }

    // Headless A/B comparison (no GPU): diff two renders / exported iteration files.
    //   --compare A B [--out heatmap.png]
    if let Some(i) = args.iter().position(|a| a == "--compare") {
        return crate::harness::compare::run(args, i);
    }

    // Cross-renderer validation against **Fraktaler-3** (no GPU): compare F3's raw
    // integer escape counts (EXR channel "N", UINT) against our *independent* CPU
    // arbitrary-precision dwell oracle at the identical complex coordinate of every
    // pixel. Two fully independent engines (F3's GPU perturbation vs our bignum CPU)
    // agreeing on exact integer iteration counts is the strongest external check.
    //
    //   --crosscheck-f3 raw.exr --center X Y --zoom-f3 Z [--iter K] [--er 256]
    //
    // Render the F3 side with (in a .f3.toml batch): render.save_exr = true,
    // render.exr_channels = ["N0"], image.subframes = 1, transform.exponential_map = false.
    if let Some(i) = args.iter().position(|a| a == "--crosscheck-f3") {
        return crate::harness::crosscheck_f3::run(args, i);
    }

    // Arbitrary-precision cost benchmark. CPU-only and GPU-free by construction, so it runs on a
    // CI box: the reference-orbit build is where a deep frame's time actually goes (the blessed
    // bench-matrix baseline puts it at 66% of `deep-interior-1e148`), and it is pure bignum.
    //
    // ⚠Two rules this table obeys, both learned the hard way:
    //   * It prints the backend that ACTUALLY ran, from `observed_backends()`. A benchmark whose
    //     configuration and behaviour can disagree measures an unknown build.
    //   * It asserts each row's orbit stayed BOUNDED and says so. An escaped orbit spends its
    //     iterations on infinities, which are fast and meaningless — the first draft of the
    //     backend-comparison probe "won" a row exactly that way before the check existed.
    //   --bench-bignum [--iters N]
    if args.iter().any(|a| a == "--bench-bignum") {
        return crate::harness::bench_bignum::run(args);
    }

    // Extreme-depth validation battery (no GPU, no external data): exercises the
    // arbitrary-precision arithmetic core at magnifications far beyond f64 range
    // (1e1000 … 1e1000000), via precision-doubling self-consistency + coordinate round-trip.
    // A per-pixel dwell oracle is infeasible this deep; these single-point checks are not.
    //   --validate-deep [--out report.md]
    if args.iter().any(|a| a == "--validate-deep") {
        return crate::harness::validate_deep::run(args);
    }
    false
}


use crate::{diag, harness::profile, scripting};

// ==== CLI-launched mode state ====
// The flags parsed from argv that drive `update()`'s mode ladder, grouped out of
// `FractadyneApp` so the app struct reads as the interactive application. Field docs
// moved verbatim from the original `main.rs` declarations.

/// `--selftest` mode state, parsed from the EXPANDED argv (see `selftest_filter`'s
/// old note: raw `std::env::args()` would bypass `@response-file` expansion).
pub(crate) struct SelftestCli {
    /// CLI `--selftest`: run the GPU validation suite, print a report, and exit.
    pub(crate) run: bool,
    pub(crate) done: bool,
    /// `--selftest-filter <substr>`, `--selftest-list`, `--bless` — parsed here from the
    /// EXPANDED args so they honor `@response-file` / `--args-file` expansion. `run_selftest`
    /// must read these, NOT `std::env::args()` (raw args bypass the expansion `main()` did).
    pub(crate) filter: Option<String>,
    pub(crate) list: bool,
    pub(crate) bless: bool,
    /// `--bless-new-adapter`: let `--bless` on a card other than the one
    /// `validation/golden/BLESSED-GPU.txt` names overwrite the REFERENCE set, moving the release
    /// gate's reference adapter (W8). Without it, such a bless writes that card's own set.
    pub(crate) bless_new_adapter: bool,
}

/// `--profile` mode state (the profiling harness: benchmark regions -> logs/).
pub(crate) struct ProfileCli {
    /// CLI `--profile`: run the profiling harness (benchmark regions), log to `logs/`, exit.
    pub(crate) run: bool,
    pub(crate) done: bool,
    pub(crate) reps: u32,
    pub(crate) regions: Option<String>,
    pub(crate) out: Option<std::path::PathBuf>,
}

/// `--frametest` mode state (frame-timing / stutter harness dive shape).
pub(crate) struct FrametestCli {
    /// CLI `--frametest`: run the frame-timing / stutter harness (deep-zoom dive), log, exit.
    pub(crate) run: bool,
    /// `--frametest --center X Y` (full-precision decimals; default seahorse).
    pub(crate) center: Option<(String, String)>,
    pub(crate) steps: u32,
    pub(crate) hold: u32,
    pub(crate) dive: f64,
}

/// Benchmark state - the GUI standardized benchmark plus the `--benchmark` /
/// `--benchmark-std` CLI drivers share it.
pub(crate) struct BenchState {
    /// Last benchmark report text + whether its window is open.
    pub(crate) report: Option<String>,
    pub(crate) cfg: crate::BenchConfig,
    /// In-flight standardized benchmark, advanced one pass per frame from `update()`.
    pub(crate) std: Option<crate::StdBench>,
    /// CLI `--benchmark-std [--res RES] [--burnin N]`: run headless, save, quit.
    pub(crate) auto_std: bool,
    pub(crate) auto_std_done: bool,
    pub(crate) std_res: crate::BenchRes,
    pub(crate) std_passes: u32,
    pub(crate) std_depth: crate::BenchDepth,
    /// CLI auto-benchmark: run on startup, save to this path, then quit.
    pub(crate) auto: bool,
    pub(crate) auto_out: Option<std::path::PathBuf>,
    pub(crate) auto_done: bool,
}

/// Render-and-exit CLI modes: `--render` (PNG / `--render-iter` EXR), `--render-tour`,
/// and `--play` (tour in the live view).
pub(crate) struct RenderCli {
    /// CLI render-and-exit: render one image to `out`, then quit.
    pub(crate) run: bool,
    pub(crate) out: Option<std::path::PathBuf>,
    pub(crate) done: bool,
    /// `--render-iter`: write the raw iteration texture as EXR instead of a colored image.
    pub(crate) iter_mode: bool,
    /// `--glitch-audit [N]` / `--tail-audit [N]`: instead of writing an image, check N of the pixels
    /// where two renders differ (and N controls) against the arbitrary-precision oracle
    /// (`glitchaudit`); `audit_kind` says which two.
    pub(crate) audit: Option<usize>,
    pub(crate) audit_kind: crate::harness::glitchaudit::AuditKind,
    /// CLI `--render-tour FILE`: render a keyframe tour to a PNG frame sequence, then quit.
    pub(crate) tour: Option<std::path::PathBuf>,
    pub(crate) tour_done: bool,
    /// The tour-render flags as GIVEN — each `None` means "the script's `[render]` block decides"
    /// (and a built-in default if it's silent too). Resolving here would erase the difference
    /// between a flag the user passed and a default we made up.
    pub(crate) tour_cfg: crate::scripting::TourRenderConfig,
    /// CLI `--play FILE`: start the GUI with this tour already playing in the LIVE view. The only
    /// way to exercise the on-screen playback path (present, watchdog budget, tiled settle) from a
    /// command line — a headless harness cannot reach it.
    pub(crate) play: Option<std::path::PathBuf>,
    pub(crate) play_done: bool,
}

/// One-shot dev-harness modes: each field is a CLI-requested mode that drives the app
/// from `update()`'s ladder (`cli_mode_ladder` / `harness_frame_hooks`) and exits or
/// finishes on its own.
pub(crate) struct HarnessModes {
    /// CLI `--uitest [DIR]`: scripted walk through every UI screen + the live-render bands,
    /// screenshotting each and writing a review bundle (see `mod uitest`), then exit.
    pub(crate) uitest: Option<crate::harness::uitest::UiTest>,
    /// `--uitest` scratch for the control-panel toggle step: the canvas and panel widths measured
    /// while the panel was still open, so the reflow after hiding it can be compared against what
    /// the layout actually did rather than against the panel's nominal width.
    pub(crate) uitest_central_w: Option<f32>,
    pub(crate) uitest_panel_w: Option<f32>,
    /// The shader key the `formula-apply-async` step applied, so its record can check that the
    /// view switched to exactly that formula.
    pub(crate) uitest_async_key: Option<u64>,
    /// Pointer events the `life-draw` step feeds the app, one a frame, at CELLS of the Life view
    /// (turned into positions as they are fed, so they land on the cell whatever the layout).
    pub(crate) uitest_pointer: std::collections::VecDeque<crate::UiPointer>,
    pub(crate) soak: Option<crate::harness::soak::Soak>,
    /// `--recordtest [FRAMES]`: the frame record's own gate — does it record every frame, fill
    /// every required field, and survive an abort? See `mod recordtest`.
    pub(crate) recordtest: Option<crate::harness::recordtest::RecordTest>,
    /// `--shot`: regenerate the published screenshot from a saved location, then exit.
    pub(crate) shot: Option<crate::harness::shot::Shot>,
    /// CLI `--juliadive [DIR]`: dev harness — dual view, continuous in-app Julia zoom to ~1400×
    /// with periodic screenshots (see `uitest::JuliaDive`). Reproduces the dual-view Julia motion
    /// path deterministically (synthetic OS input proved unreliable for wheel/focus).
    pub(crate) juliadive: Option<crate::harness::uitest::JuliaDive>,
    /// CLI `--dualsettle [DIR]`: dev harness — dual view, live-hover c motion then idle, timed
    /// settled screenshots (the 2026-09-04 blurry-Julia field report).
    pub(crate) dualsettle: Option<crate::harness::uitest::DualSettle>,
    /// CLI `--autodive [LOG10]`: unpaced autopilot dive that hammers the frame-cost controller and
    /// reports whether the lethal regime was reached. See `autopilot::AutoDive`.
    pub(crate) autodive: Option<crate::autopilot::AutoDive>,
    /// CLI `--chunk-sweep [ITERS]`: measure the UNCENSORED per-window wall cost of a chunked
    /// iterate at the current view. The one measurement that separates the two branches of the
    /// 2026-08-22 device loss — see `mod chunksweep`.
    pub(crate) chunk_sweep: Option<crate::harness::chunksweep::ChunkSweep>,
    /// CLI `--deviceloss-repro [ZOOM_LOG2] [REF_ITER]`: a SAFE reproduction of the deep-interior
    /// device loss — measures the honest floor-window submission cost at small areas and extrapolates
    /// to full resolution, predicting the lethal dispatch without running it. See `mod deviceloss_repro`.
    pub(crate) deviceloss_repro: Option<crate::harness::deviceloss_repro::DeviceLossRepro>,
    /// CLI `--motiontest`: the motion-presentation gate (design/mode2-chunking.md §11) — the
    /// in-loop harness that can see what `--livetest`'s settled checkpoints cannot: what a
    /// chunked view ADOPTS as its frozen texture while the camera is moving.
    pub(crate) motiontest: Option<crate::harness::motiontest::MotionTest>,
    /// CLI `--zoomtest [OCTAVES]`: on-screen update-latency harness for live zooms — the real
    /// window, a virtual Space key through the production glide, every presented frame's wall
    /// interval + what it showed, JSON + summary, exit. See `mod zoomtest`.
    pub(crate) zoomtest: Option<crate::harness::zoomtest::ZoomTest>,
    /// CLI `--divetest FILE`: headless live-dive performance harness (real-time tour windows at
    /// increasing depths through the ACTUAL playback machinery), report + JSON, exit.
    pub(crate) divetest: Option<std::path::PathBuf>,
    /// CLI `--livetest FILE`: headless live-OUTPUT harness — plays a tour through the live
    /// pipeline and validates the frames it shows against offline renders of the same views.
    pub(crate) livetest: Option<std::path::PathBuf>,
    /// `--quick` with `--livetest`: skip the offline oracle (context/black metrics only).
    pub(crate) livetest_quick: bool,
    /// CLI `--reusetest`: measure reuse-first-zoom reprojection staleness vs Δ-octaves, exit.
    pub(crate) reusetest: bool,
    pub(crate) reusetest_done: bool,
    /// CLI `--resizetest`: headless window-resize aspect-invariant regression harness, exit.
    pub(crate) resizetest: bool,
    /// CLI `--bench-matrix [--bless] [--reps N]`: run the path-coverage perf + regression suite,
    /// compare against (or bless) the baseline, and exit.
    pub(crate) bench_matrix: bool,
    pub(crate) bench_matrix_done: bool,
}

impl crate::FractadyneApp {
    /// Per-frame advance hooks for the in-loop dev harnesses (`--shot`, `--uitest`,
    /// `--soak`, `--juliadive`, `--autodive`, `--chunk-sweep`, `--motiontest`).
    /// Order matters and is documented inline; moved verbatim from `update()`.
    pub(crate) fn harness_frame_hooks(
        &mut self,
        ctx: &eframe::egui::Context,
        gpu: &Option<(eframe::wgpu::Device, eframe::wgpu::Queue)>,
        gpu_name: &Option<String>,
    ) {
        // --uitest: advance the scripted UI/live walk. Runs each frame and does NOT exit early —
        // the flags it sets (which dialog is open, which view) must be drawn by the rest of this
        // update(); it screenshots and exits itself once the walk is done. Gated on GPU being up.
        // --shot: load, settle, capture, exit. Before --uitest so the two can never both
        // drive a frame.
        if let (true, Some(gpu)) = (self.harness.shot.is_some(), gpu) {
            self.shot_frame(ctx, gpu);
        }
        if self.harness.uitest.is_some() && gpu.is_some() {
            self.uitest_frame(ctx, gpu_name.as_deref());
        }
        // --soak: sit at a deep view and assert the app keeps producing frames.
        if self.harness.soak.is_some() && gpu.is_some() {
            self.soak_frame(ctx);
        }
        // --recordtest: drive live frames, then audit the frame record against its own count.
        if self.harness.recordtest.is_some() && gpu.is_some() {
            self.recordtest_frame(ctx);
        }
        // --dualsettle: dev harness for the dual-view Julia SETTLE path (same in-loop pattern).
        if self.harness.dualsettle.is_some() && gpu.is_some() {
            self.dualsettle_frame(ctx);
        }
        // --juliadive: dev harness for the dual-view Julia motion path (same in-loop pattern).
        if self.harness.juliadive.is_some() && gpu.is_some() {
            self.juliadive_frame(ctx);
        }
        // --autodive: unpaced controller hammer (same in-loop pattern). Must run AFTER the
        // measurement apply above, so the reading it samples is this frame's.
        if self.harness.autodive.is_some() && gpu.is_some() {
            self.autodive_frame(ctx);
        }
        // --chunk-sweep: let the live view settle, then measure the chunked iterate's per-window
        // wall cost offscreen and exit. It borrows the resident reference, so it MUST run after
        // the reference pipeline has had frames to finish — hence the settle countdown inside.
        if self.harness.chunk_sweep.is_some() {
            if let Some((dev, q)) = gpu {
                let dev = dev.clone();
                let q = q.clone();
                if self.chunk_sweep_step(&dev, &q) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                } else {
                    ctx.request_repaint();
                }
            }
        }
        // --deviceloss-repro: builds its own reference synchronously and measures the floor-window
        // cost, so it does NOT wait for the live pipeline — it runs on the first frame it gets a
        // device and closes. Same in-loop pattern as chunk-sweep.
        if self.harness.deviceloss_repro.is_some() {
            if let Some((dev, q)) = gpu {
                let dev = dev.clone();
                let q = q.clone();
                if self.deviceloss_repro_step(&dev, &q) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                } else {
                    ctx.request_repaint();
                }
            }
        }
        // --motiontest: the motion-presentation gate (same in-loop pattern). It sets this frame's
        // input state (zoom_vel / the Home glide), so it runs BEFORE the central draw below.
        if self.harness.motiontest.is_some() && gpu.is_some() {
            self.motiontest_frame(ctx);
        }
        // --zoomtest: the on-screen update-latency harness (same in-loop pattern). It holds the
        // virtual Space key the central draw and the lookahead pump read this frame, and stamps the
        // interval since the previous update — so it too runs BEFORE the central draw.
        if self.harness.zoomtest.is_some() && gpu.is_some() {
            self.zoomtest_frame(ctx);
        }
    }

    /// The CLI-mode ladder: every `--flag` that runs a suite/render and exits (or, for
    /// the GUI standardized bench and `--play`, arms ongoing state). One branch per
    /// mode, in load-bearing order; moved verbatim from `update()`.
    pub(crate) fn cli_mode_ladder(
        &mut self,
        ctx: &eframe::egui::Context,
        gpu: &Option<(eframe::wgpu::Device, eframe::wgpu::Queue)>,
    ) {
        // CLI self-test: run the GPU validation suite, print the report, and exit with a
        // status code (0 = all passed).
        if self.selftest.run && !self.selftest.done {
            if let Some((dev, q)) = gpu {
                self.selftest.done = true;
                let ok = self.run_selftest(dev, q);
                crate::exit(if ok { 0 } else { 1 });
            }
        }

        // CLI profiling: render the benchmark regions, time the costly stages, log to `logs/`.
        if self.profile.run && !self.profile.done {
            if let Some((dev, q)) = gpu {
                self.profile.done = true;
                let regions = match &self.profile.regions {
                    Some(path) => match profile::load_regions(std::path::Path::new(path)) {
                        Ok(r) => r,
                        // FATAL, not a fallback. This fell back to the built-ins and exited 0,
                        // so a regions file that failed to parse (measured: one cp1252 em-dash)
                        // profiled eight unrelated views and reported success — the exact trap the
                        // F3 harness taught us to check output files for. The flag is an explicit
                        // request; honouring a different one is worse than stopping.
                        Err(e) => {
                            eprintln!("fractadyne: --regions {path}: {e}");
                            crate::exit(2);
                        }
                    },
                    None => profile::default_regions(),
                };
                let secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let out = self.profile.out.clone().unwrap_or_else(|| {
                    std::path::PathBuf::from(format!("logs/profile-{}.json", Self::file_stamp(secs)))
                });
                let reps = self.profile.reps;
                self.run_profile(dev, q, &regions, reps, &out);
                crate::exit(0);
            }
        }

        // CLI path-matrix benchmark: exercise every rendering path, compare (or bless) the
        // baseline, and flag regressions. `--bless` records; `--reps N` sets timed reps.
        if self.harness.bench_matrix && !self.harness.bench_matrix_done {
            if let Some((dev, q)) = gpu {
                self.harness.bench_matrix_done = true;
                let bless = self.selftest.bless; // shared `--bless` flag
                let reps = self.profile.reps; // shared `--reps N` flag (default 5)
                let code = self.run_bench_matrix(dev, q, bless, reps, false);
                crate::exit(code);
            }
        }

        // CLI resize regression harness: scripted drag-resize through the real frame logic,
        // asserting every painted frame is aspect-correct (exit 0 = invariant held).
        if self.harness.resizetest {
            if let Some((dev, q)) = gpu {
                self.harness.resizetest = false;
                self.run_resizetest(dev, q); // exits
            }
        }

        if self.harness.reusetest && !self.harness.reusetest_done {
            if let Some((dev, q)) = gpu {
                self.harness.reusetest_done = true;
                self.run_reusetest(dev, q);
                crate::exit(0);
            }
        }

        // CLI headless live-dive harness: real-time tour windows at increasing depths through the
        // actual playback machinery (pacer + lookahead + reuse-hold), stats per depth band, exit.
        if let Some(tour) = self.harness.divetest.clone() {
            if let Some((dev, q)) = gpu {
                let secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let out = self.profile.out.clone().unwrap_or_else(|| {
                    std::path::PathBuf::from(format!("logs/divetest-{}.json", Self::file_stamp(secs)))
                });
                self.run_divetest(dev, q, &tour, &out);
                crate::exit(0);
            }
        }

        // CLI headless live-OUTPUT harness: play a tour through the live pipeline and validate the
        // frames it puts on screen against offline renders of the same views; exits by
        // `crate::verdict` (2 drift from the blessed baseline, 3 nothing judged).
        if let Some(tour) = self.harness.livetest.clone() {
            if let Some((dev, q)) = gpu {
                let (w, h) = self.tour_size_or(960, 540);
                let out = self
                    .render_cli.tour_cfg
                    .out
                    .clone()
                    .unwrap_or_else(|| std::path::PathBuf::from("logs"));
                let seg = self.render_cli.tour_cfg.segment.clone();
                let code =
                    self.run_livetest(dev, q, &tour, seg.as_deref(), [w, h], &out, self.harness.livetest_quick);
                crate::exit(code);
            }
        }

        if self.frametest.run {
            if let Some((dev, q)) = gpu {
                let secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let out = self.profile.out.clone().unwrap_or_else(|| {
                    std::path::PathBuf::from(format!("logs/frametest-{}.json", Self::file_stamp(secs)))
                });
                let (steps, hold, dive) = (self.frametest.steps, self.frametest.hold, self.frametest.dive);
                self.run_frametest(dev, q, steps, hold, dive, 512, &out);
                crate::exit(0);
            }
        }

        // CLI standardized benchmark (`--benchmark-std [--res] [--burnin] [--depth]`): run all
        // passes synchronously, print + save the report, and quit.
        if self.bench.auto_std && !self.bench.auto_std_done {
            if let Some((dev, q)) = gpu {
                self.bench.auto_std_done = true;
                let (res, passes, depth) = (self.bench.std_res, self.bench.std_passes, self.bench.std_depth);
                println!(
                    "Fractadyne standardized benchmark — {} · {} × {passes} pass{}",
                    res.label(),
                    depth.label(),
                    if passes == 1 { "" } else { "es" }
                );
                let mut run = self.begin_standard_bench(res, passes, depth);
                // step_std_bench now advances one dive-frame per call; print only when a pass
                // finishes (passes_done ticks up), not on every frame.
                let mut reported = 0u32;
                loop {
                    let done = self.step_std_bench(&mut run, dev, q);
                    if run.passes_done > reported {
                        reported = run.passes_done;
                        println!(
                            "  pass {}/{}  {:.1} fps",
                            run.passes_done,
                            run.passes_total,
                            run.pass_fps.last().copied().unwrap_or(0.0)
                        );
                    }
                    if done {
                        break;
                    }
                }
                let report = self.format_std_bench(&run);
                println!("\n{report}");
                let out = self.bench.auto_out.clone().unwrap_or_else(|| {
                    std::path::PathBuf::from("fractadyne_benchmark.txt")
                });
                match std::fs::write(&out, &report) {
                    Ok(()) => println!("\nSaved benchmark → {}", out.display()),
                    Err(e) => eprintln!("Failed to save benchmark to {}: {e}", out.display()),
                }
                crate::exit(0);
            }
        }

        // GUI standardized benchmark: advance one pass per frame so the window stays responsive
        // (and cancellable) between passes.
        if let Some(mut run) = self.bench.std.take() {
            if let Some((dev, q)) = gpu {
                let done = self.step_std_bench(&mut run, dev, q);
                if done {
                    self.bench.report = Some(self.format_std_bench(&run));
                    self.dialogs.bench_open = true;
                    let snap = run.take_snapshot();
                    self.restore_from_bench(snap);
                } else {
                    self.bench.std = Some(run);
                    ctx.request_repaint();
                }
            } else {
                self.bench.std = Some(run); // wait for the GPU handles
            }
        }

        // CLI render-and-exit: render one image offscreen (or the raw iteration EXR), save
        // it, and quit.
        if self.render_cli.run && !self.render_cli.done {
            if let Some((dev, q)) = gpu {
                self.render_cli.done = true;
                if !self.watermark && !self.render_cli.iter_mode {
                    println!("Note: Fd watermark is off (saved preference) — pass --watermark to include it.");
                }
                let t0 = std::time::Instant::now();
                let result = if let Some(n) = self.render_cli.audit {
                    self.run_glitch_audit(dev, q, n, self.render_cli.audit_kind)
                } else if self.render_cli.iter_mode {
                    let out = self
                        .render_cli.out
                        .clone()
                        .unwrap_or_else(|| std::path::PathBuf::from("fractadyne_iter.exr"));
                    self.render_iter_to_file(dev, q, &out)
                } else {
                    let out = self
                        .render_cli.out
                        .clone()
                        .unwrap_or_else(|| std::path::PathBuf::from("fractadyne_render.png"));
                    self.render_to_file(ctx, dev, q, &out)
                };
                if self.render_cfg.finish_sound {
                    // Blocking on purpose: the process exits right after the message prints,
                    // which would cut a detached tune mid-note.
                    crate::tone::play_finish_sound(true);
                }
                // Synchronous offscreen work, not a live dispatch: the tripwire must not read this
                // frame's interval as a lethal-band GPU frame (`render::tripwire_dt`).
                self.perf.offscreen_ms += t0.elapsed().as_secs_f64() * 1000.0;
                match result {
                    Ok(m) => println!("{m}  (in {})", Self::fmt_export_duration(t0.elapsed())),
                    Err(e) => {
                        // Fail with a real exit code: the Close path always exits 0, which
                        // made scripted corpus renders unable to detect failures (D1/F8).
                        eprintln!("Render failed: {e}");
                        diag::log_line("render", &format!("FAILED: {e}"));
                        crate::exit(1);
                    }
                }
                // Exit HERE, not through `ViewportCommand::Close`: a close runs the rest of this
                // frame and one more, and those are LIVE frames — at 4.6e1105× they spawned a
                // 258k-iteration live reference build the exit then abandoned — before eframe's
                // teardown (0.2 s at 4K). Nothing on that path matters to a CLI render: autosave
                // is off for it (the gate at the end of `update`) and so is the orbit cache.
                crate::exit(0);
            }
        }

        // CLI render-tour: render the keyframe script to a PNG frame sequence, then quit.
        if let Some(script) = self.render_cli.tour.clone() {
            // `--farm-child`: the burned-in "Fd" watermark is rasterized at a fixed 40 points ×
            // the display's pixels-per-point, then scaled to the frame — so the same frame's mark
            // came out differently on a 1.5× laptop and a 1× workstation. (Captions, callouts and
            // the HUD do not: they lay out at `size / ppp` and so rasterize at the same pixel size
            // at any scale — measured, a 1.5× and a pinned render are identical there.) A farm pins
            // FARM_PIXELS_PER_POINT and rebuilds the mark at it.
            //
            // It cannot be set before the first frame — egui converts it to a zoom factor through
            // the NATIVE scale, unknown until a pass has run (measured: set at creation, the render
            // still saw 1.5) — and it applies from the NEXT pass, while the whole tour renders
            // inside one. So: set it, drop the mark built at the native scale, repaint, start then.
            //
            // ⚠That extra frame has a cost: eframe 0.31 shows the window once its FIRST frame is
            // painted (`post_rendering`), whatever `with_visible` asked for — an ordinary tour stays
            // hidden only because it renders inside frame one and never finishes it. So the window
            // is hidden again here; eframe applies this frame's viewport commands after showing it.
            if self.render_cli.tour_cfg.farm_child
                && !self.render_cli.tour_done
                && (ctx.pixels_per_point() - FARM_PIXELS_PER_POINT).abs() > 1e-6
            {
                ctx.set_pixels_per_point(FARM_PIXELS_PER_POINT);
                self.watermark_overlay = None;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                ctx.request_repaint();
                return;
            }
            if !self.render_cli.tour_done {
                if let Some((dev, q)) = gpu {
                    self.render_cli.tour_done = true;
                    if !self.watermark {
                        println!("Note: Fd watermark is off (saved preference) — pass --watermark to include it.");
                    }
                    let cfg = self.render_cli.tour_cfg.clone();
                    let t0 = std::time::Instant::now();
                    let result = self.render_tour_to_dir(ctx, dev, q, &script, &cfg);
                    // Synchronous offscreen work (see `render::tripwire_dt`).
                    self.perf.offscreen_ms += t0.elapsed().as_secs_f64() * 1000.0;
                    match result {
                        Ok(m) => println!("{m}"),
                        Err(e) => {
                            eprintln!("Tour render failed: {e}");
                            diag::log_line("render", &format!("tour FAILED: {e}"));
                            crate::exit(1);
                        }
                    }
                    if self.render_cli.tour_cfg.farm_child {
                        // `Close` is acted on in a LATER frame, and a hidden window gets none —
                        // measured: the farm child rendered every frame, then waited forever. Exit
                        // here, as `--render` does, once any orbit-cache write this run started is
                        // on disk (a farm child keeps its references for the next run).
                        crate::refcache_persist::drain();
                        crate::exit(0);
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }

        // CLI `--play FILE`: start the tour in the LIVE view (once). Unlike every other tour entry
        // point this one keeps the GUI running, so it exercises the on-screen path — present,
        // watchdog budget, tiled settle — which no headless harness can reach.
        if let Some(script) = self.render_cli.play.clone() {
            if !self.render_cli.play_done {
                self.render_cli.play_done = true;
                match scripting::parse_tour_file(&script) {
                    Ok(pb) => {
                        println!("Playing \"{}\" ({:.0}s) in the live view…", pb.name, pb.total);
                        self.playback_restore = Some(scripting::PlaybackRestore {
                            max_iter: self.render_cfg.max_iter,
                            auto_iter: self.render_cfg.auto_iter,
                            palette_idx: self.coloring.palette_idx,
                            use_custom_palette: self.coloring.use_custom_palette,
                            use_binary: self.coloring.use_binary,
                            use_duotone: self.coloring.use_duotone,
                            minimap: self.dialogs.minimap,
                            show_orbits: self.anim.show_orbits,
                            dual_split: self.dual_split,
                        });
                        self.playback = Some(pb);
                    }
                    Err(e) => {
                        eprintln!("Could not play {}: {e}", script.display());
                        crate::exit(2);
                    }
                }
            }
        }
    }
}
