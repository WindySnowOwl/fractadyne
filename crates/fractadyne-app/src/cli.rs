//! Headless CLI modes, run before the GUI starts: --find-minibrot, --compare,
//! --crosscheck-f3, and --validate-deep. `run_headless` returns true when it handled a
//! mode (so `main` exits Ok); the validation modes set a process exit code directly.

use crate::{version_string, FractalKind};

/// Windows- and Unix-style ways to explicitly ask for help.
const HELP_TOKENS: &[&str] = &["--help", "-h", "-?", "/?", "/h", "/help", "help"];

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
    for s in args.iter().skip(1) {
        let a = s.as_str();
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
mod bad_options;

/// Dispatch the headless CLI modes. Returns true if one ran (caller should exit).
pub(crate) fn run_headless(args: &[String]) -> bool {
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
        std::process::exit(crate::torture::run(args));
    }
    // Print the tour-script schema reference (Markdown) and exit — used to (re)generate TOURS.md.
    if args.iter().any(|a| a == "--dump-tour-schema") {
        print!("{}", crate::scripting::tour_schema_markdown());
        return true;
    }
    // Check GitHub for a newer release, print the result, and exit (validates the in-app update
    // check headlessly; handy for automation). Optional track: `--check-updates beta` or
    // `--check-updates=beta` (default stable). Set FRACTADYNE_FAKE_VERSION to pretend the running
    // build is an older/newer version — the only way to exercise the "update available" branch
    // while the dev build is ahead of the latest release.
    if let Some(pos) = args.iter().position(|a| a == "--check-updates" || a.starts_with("--check-updates=")) {
        let track_arg = args[pos]
            .split_once('=')
            .map(|(_, v)| v.to_string())
            .or_else(|| args.get(pos + 1).filter(|a| !a.starts_with('-')).cloned())
            .unwrap_or_default();
        let track = crate::update::UpdateTrack::from_str(&track_arg);
        let cur = crate::update::running_version();
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
    if args.iter().any(|a| a == "--gputest") {
        let fails = crate::gputest::run_gputest_sweep();
        crate::exit(if fails > 0 { 1 } else { 0 });
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
        use fractadyne_core as fc;
        let val = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1));
        let two = |name: &str| args.iter().position(|a| a == name).and_then(|i| Some((args.get(i + 1)?, args.get(i + 2)?)));
        let (cx, cy) = two("--center")
            .map(|(x, y)| crate::arg_center("--center", x, y))
            .unwrap_or((fc::BigFloat::from_f64(-0.5, 64), fc::BigFloat::from_f64(0.0, 64)));
        let log2mag = val("--zoom-log2")
            .map(|s| crate::arg_parse::<f64>("--zoom-log2", s, "a number"))
            .unwrap_or(133.0);
        let max_iter = val("--iter")
            .map(|s| crate::arg_parse::<u32>("--iter", s, "a whole number"))
            .unwrap_or(60_000);
        let p = fc::precision_for_octaves(log2mag.ceil().max(0.0) as u64);
        let mut vp = fc::Viewport::new(1000.0, 1000.0);
        vp.set_center_log2mag(cx, cy, log2mag);
        println!("refdiag @ 1e{:.1}x, precision {p} bits, max_iter {max_iter}", log2mag / std::f64::consts::LOG2_10);
        // Sample an 11x11 grid across the view; report each point's reference orbit length.
        let n = 11usize;
        let mut lens: Vec<u32> = Vec::new();
        let zero = fc::BigFloat::from_f64(0.0, p);
        for j in 0..n {
            for i in 0..n {
                let px = (i as f64 / (n as f64 - 1.0)) * 1000.0;
                let py = (j as f64 / (n as f64 - 1.0)) * 1000.0;
                let (rx, ry) = vp.pixel_to_complex(px, py);
                let (_orbit, len) = fc::reference_orbit(&zero, &zero, &rx, &ry, 0, max_iter, p);
                lens.push(len);
            }
        }
        lens.sort_unstable();
        let total = lens.len();
        let interior = lens.iter().filter(|&&l| l >= max_iter).count();
        let median = lens[total / 2];
        let maxl = *lens.last().unwrap();
        let minl = *lens.first().unwrap();
        // Histogram buckets by orbit length.
        let buckets = [1000u32, 4000, 8000, 16000, 32000, u32::MAX];
        let mut hist = [0usize; 6];
        for &l in &lens {
            for (bi, &b) in buckets.iter().enumerate() {
                if l < b { hist[bi] += 1; break; }
            }
        }
        println!("  {total} points: min={minl} median={median} max={maxl} interior(>=max_iter)={interior}");
        println!("  orbit-length histogram: <1k={} <4k={} <8k={} <16k={} <32k={} >=32k={}",
            hist[0], hist[1], hist[2], hist[3], hist[4], hist[5]);
        let best = fc::best_reference(&[
            {let (x,_)=vp.pixel_to_complex(500.0,500.0); x},
            {let (_,y)=vp.pixel_to_complex(500.0,500.0); y},
        ], [vp.complex_span_fe().0, vp.complex_span_fe().1], 0, false, [0.0,0.0], max_iter, p);
        let (_bo, blen) = fc::reference_orbit(&zero, &zero, &best[0], &best[1], 0, max_iter, p);
        println!("  best_reference orbit length = {blen}  (if << max_iter, pixels rebase past it)");
        return true;
    }

    // --pickcheck [FILE] [--size WxH] [--iter N]: the reference-pick redesign's acceptance
    // harness (design/pick-redesign.md). Runs BOTH phase-2 scoring engines — the original
    // bignum deep walk and the floatexp perturbation scorer — on identical inputs and asserts
    // they elect the SAME reference point. Bare = the built-in depth ladder (committed corpus /
    // validation locations, e17 → e4000; run from the repo root); FILE = one `.fdn` location.
    // Exit 0 = every rung ran and matched; 1 = a winner differed; 2 = a rung was skipped
    // (a skip is "not tested", never a pass).
    if args.iter().any(|a| a == "--pickcheck") {
        use fractadyne_core as fc;
        let val = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1));
        let (w, h) = val("--size")
            .map(|s| {
                s.split_once('x')
                    .and_then(|(a, b)| Some((a.parse::<u32>().ok()?, b.parse::<u32>().ok()?)))
                    .unwrap_or_else(|| {
                        eprintln!("--size wants WxH (e.g. 1280x720), got '{s}'");
                        crate::exit(1);
                    })
            })
            .unwrap_or((1280, 720)); // the corpus render geometry
        let iter_override = val("--iter").map(|s| crate::arg_parse::<u32>("--iter", s, "a whole number"));
        // One rung: a location file, or the built-in e89 dendrite (the misiurewicz_outcomes
        // fixture — kept verbatim from crates/fractadyne-core/tests/misiurewicz_outcomes.rs,
        // the one ladder depth with no committed .fdn).
        enum Rung {
            File(&'static str),
            Given(String),
            Builtin { name: &'static str, cx: &'static str, cy: &'static str, log2mag: f64, iter: u32 },
        }
        const E89_CX: &str = "2.336541879936817878966215410838761608133707885474547567506859621515201675599324293839217233261500414218604921872973e-2";
        const E89_CY: &str = "8.2741173753632652070456275296875144057156928752279993918651650950463440957323066581594346138419917471379151772617142e-1";
        let ladder: Vec<Rung> = match val("--pickcheck").filter(|s| !s.starts_with('-')) {
            Some(path) => vec![Rung::Given(path.to_string())],
            None => vec![
                Rung::File("validation/corpus/locations/05-dendrite-8e17.fdn"),
                Rung::File("validation/corpus/locations/06-seahorse-1e24.fdn"),
                Rung::File("validation/corpus/locations/08-deep-6.6e43.fdn"),
                Rung::Builtin {
                    name: "e89-dendrite (misiurewicz_outcomes)",
                    cx: E89_CX,
                    cy: E89_CY,
                    log2mag: 297.12071983391957, // 2.77e89×
                    iter: 100_000,
                },
                Rung::File("validation/corpus/locations/14-deep-1.2e148.fdn"),
                Rung::File("validation/corpus/locations/09-deep-6.1e500.fdn"),
                Rung::File("validation/corpus/locations/19-deep-1.3e726.fdn"),
                Rung::File("validation/corpus/locations/20-deep-1.2e1008.fdn"),
                Rung::File("validation/e4000-misiurewicz.fdn"),
            ],
        };
        let (mut matched, mut mismatched, mut skipped) = (0u32, 0u32, 0u32);
        println!("pickcheck: dual-run pick equivalence (walk vs perturb), {w}x{h}, {} rung(s)", ladder.len());
        for (ri, rung) in ladder.iter().enumerate() {
            // Resolve the rung to (name, cx, cy, log2mag, iter, formula, julia). A file that
            // cannot supply a usable input is a loud SKIP, never a silent default.
            let path: Option<&str> = match rung {
                Rung::File(p) => Some(p),
                Rung::Given(s) => Some(s.as_str()),
                Rung::Builtin { .. } => None,
            };
            let parsed: Result<(String, String, String, f64, u32, u32, bool), String> = match rung {
                Rung::Builtin { name, cx, cy, log2mag, iter } => Ok((
                    (*name).to_string(),
                    (*cx).to_string(),
                    (*cy).to_string(),
                    *log2mag,
                    iter_override.unwrap_or(*iter),
                    0,
                    false,
                )),
                _ => {
                    let p = path.unwrap();
                    (|| {
                        let text = std::fs::read_to_string(p).map_err(|e| format!("{p}: {e}"))?;
                        let get = |k: &str| crate::meta_get(&text, k);
                        let upp_log2 = match get("upp_log2").parse::<f64>() {
                            Ok(v) if v.is_finite() => v,
                            _ => return Err(format!("{p}: no readable upp_log2 (depth) field")),
                        };
                        let iter = match iter_override.or_else(|| get("max_iter").parse::<u32>().ok()) {
                            Some(i) if i > 0 => i,
                            _ => return Err(format!("{p}: no readable max_iter (pass --iter N)")),
                        };
                        let formula = match get("fractal") {
                            s if s.is_empty() => 0, // absent → Mandelbrot, as the app's loader defaults
                            s => FractalKind::from_name(&s)
                                .map(|k| k.formula_id())
                                .ok_or_else(|| format!("{p}: unknown fractal '{s}'"))?,
                        };
                        // log2(magnification) from the stored per-pixel scale, at THIS render
                        // height: mag = REFERENCE_HEIGHT / (height · upp) — mirrors Viewport.
                        let log2mag = 2.0 - (h as f64).log2() - upp_log2;
                        Ok((p.to_string(), get("center_re"), get("center_im"), log2mag, iter, formula, get("julia") == "1"))
                    })()
                }
            };
            let (name, cx_s, cy_s, log2mag, iter, formula, julia) = match parsed {
                Ok(t) => t,
                Err(why) => {
                    println!("  [{}] SKIP {why}", ri + 1);
                    skipped += 1;
                    continue;
                }
            };
            // Inputs exactly as build_params derives them for a pick at this view.
            let mut vp = fc::Viewport::new(w as f64, h as f64);
            let precision = fc::precision_for_octaves(log2mag.max(0.0).ceil() as u64);
            let (cx, cy) = match (fc::parse_bf_prec(&cx_s, precision), fc::parse_bf_prec(&cy_s, precision)) {
                (Some(x), Some(y)) => (x, y),
                _ => {
                    println!("  [{}] SKIP {name}: centre coordinates did not parse", ri + 1);
                    skipped += 1;
                    continue;
                }
            };
            vp.set_center_log2mag(cx.clone(), cy.clone(), log2mag);
            let gpu_iter = iter.min(crate::MAX_ITER_LIMIT).min(crate::zoom_iter_cap(log2mag).max(256));
            let span = vp.complex_span_fe();
            println!(
                "  [{}/{}] {name} @1e{:.1}x prec={precision}b iter={gpu_iter}",
                ri + 1,
                ladder.len(),
                log2mag / std::f64::consts::LOG2_10,
            );
            match fc::best_reference_dual(&[cx, cy], [span.0, span.1], formula, julia, [0.0, 0.0], gpu_iter, precision) {
                Err(why) => {
                    println!("      SKIP: {why}");
                    skipped += 1;
                }
                Ok(d) => {
                    println!(
                        "      walk    {:8.2}s winner_len={} survivors={} scored={}{}{}",
                        d.walk_secs,
                        d.walk_diag.winner_len,
                        d.walk_diag.survivors,
                        d.walk_diag.deep_scored,
                        d.walk_diag.rescued.map(|r| format!(" RESCUED={r}")).unwrap_or_default(),
                        if d.walk_diag.fallback_escaper { " FALLBACK-ESCAPER" } else { "" },
                    );
                    println!(
                        "      perturb {:8.2}s winner_len={} survivors={} scored={} rb={} fb={}{}{}",
                        d.perturb_secs,
                        d.perturb_diag.winner_len,
                        d.perturb_diag.survivors,
                        d.perturb_diag.deep_scored,
                        d.perturb_diag.perturb_rebases,
                        d.perturb_diag.perturb_fallbacks,
                        d.perturb_diag.rescued.map(|r| format!(" RESCUED={r}")).unwrap_or_default(),
                        if d.perturb_diag.fallback_escaper { " FALLBACK-ESCAPER" } else { "" },
                    );
                    if d.identical {
                        let speedup = if d.perturb_secs > 0.0 { d.walk_secs / d.perturb_secs } else { 0.0 };
                        println!("      MATCH ({speedup:.2}x)");
                        matched += 1;
                    } else {
                        let trunc = |b: &fc::BigFloat| {
                            let s = fc::to_decimal_string(b);
                            s.chars().take(64).collect::<String>()
                        };
                        println!("      MISMATCH:");
                        println!("        walk    ({}, {})", trunc(&d.walk[0]), trunc(&d.walk[1]));
                        println!("        perturb ({}, {})", trunc(&d.perturb[0]), trunc(&d.perturb[1]));
                        mismatched += 1;
                    }
                }
            }
        }
        println!("pickcheck: {matched} matched, {mismatched} mismatched, {skipped} skipped");
        if mismatched > 0 {
            crate::exit(1);
        } else if skipped > 0 || matched == 0 {
            crate::exit(2);
        }
        return true;
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
        // for the wrong place. `find_nucleus` is view-accurate and genuinely wants an f64
        // magnification, so the conversion back is honest -- but past f64 range it would
        // saturate to +inf, and a search there means nothing. Both cases are fatal.
        let mag = match val("--zoom") {
            None => 1.0,
            Some(z) => match crate::parse_zoom_to_log2(z) {
                Some(l2) if l2 < 1000.0 => l2.exp2(),
                other => {
                    if other.is_some() {
                        eprintln!(
                            "fractadyne: --find-minibrot: \"{z}\" is past the f64 magnification range \
                             this search works in; nucleus finding is view-accurate only."
                        );
                    } else {
                        eprintln!(
                            "fractadyne: --find-minibrot: cannot read \"{z}\" as a magnification."
                        );
                    }
                    crate::exit(2);
                }
            },
        };
        match fractadyne_core::find_nucleus(&center, mag, formula, 100_000) {
            Some(n) => {
                println!(
                    "period {}\ncenter_x {}\ncenter_y {}",
                    n.period,
                    fractadyne_core::to_decimal_string(&n.cx),
                    fractadyne_core::to_decimal_string(&n.cy),
                );
                // Atom size / orientation and the depth a Newton-Raphson zoom would frame it at
                // — the same numbers the in-app "Find minibrot center" jump uses.
                let prec = fractadyne_core::precision_for_magnification(mag);
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
        let (a, b) = (args.get(i + 1), args.get(i + 2));
        let (Some(a), Some(b)) = (a, b) else {
            eprintln!("--compare needs two file paths");
            return true;
        };
        let out = args
            .iter()
            .position(|x| x == "--out" || x == "-o")
            .and_then(|j| args.get(j + 1))
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("compare_heatmap.png"));
        let load = |p: &str| -> Option<(u32, u32, Vec<f32>)> {
            let path = std::path::Path::new(p);
            match path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref() {
                Some("exr") => fractadyne_export::read_exr_rgba_f32(path).ok(),
                Some("png") => fractadyne_export::read_png_rgba8(path)
                    .ok()
                    .map(|(w, h, bytes)| (w, h, bytes.iter().map(|&x| x as f32).collect())),
                _ => None,
            }
        };
        match (load(a), load(b)) {
            (Some((wa, ha, pa)), Some((wb, hb, pb))) if wa == wb && ha == hb => {
                let n = (wa as usize) * (ha as usize);
                // Channel 0 (smooth iteration for EXR, red for PNG) is the primary signal.
                // Channel 0 (smooth iteration / red) is always finite. The DE/normal
                // channels carry ±∞/1e30 sentinels for interior/unavailable, so the
                // all-channel stats skip non-finite and sentinel-scale diffs.
                let (mut max0, mut sum0, mut differ) = (0.0f64, 0.0f64, 0u64);
                let (mut maxall, mut sumall, mut nall) = (0.0f64, 0.0f64, 0u64);
                for k in 0..n {
                    let d0 = (pa[k * 4] - pb[k * 4]).abs() as f64;
                    max0 = max0.max(d0);
                    sum0 += d0;
                    if d0 > 1e-6 {
                        differ += 1;
                    }
                    for c in 0..4 {
                        let d = (pa[k * 4 + c] - pb[k * 4 + c]).abs() as f64;
                        if d.is_finite() && d < 1.0e20 {
                            maxall = maxall.max(d);
                            sumall += d;
                            nall += 1;
                        }
                    }
                }
                println!("Comparison: {a}  vs  {b}");
                println!("  size {wa}×{ha}");
                println!("  channel 0: max Δ {max0:.6}, mean Δ {:.6}, {differ}/{n} pixels differ", sum0 / n as f64);
                println!("  all channels (finite): max Δ {maxall:.6}, mean Δ {:.6}", sumall / nall.max(1) as f64);
                // Heatmap of |Δ channel 0|, normalized to the max (grayscale).
                let scale = if max0 > 0.0 { 1.0 / max0 as f32 } else { 0.0 };
                let mut heat = vec![0.0f32; n * 4];
                for k in 0..n {
                    let t = (pa[k * 4] - pb[k * 4]).abs() * scale;
                    heat[k * 4] = t;
                    heat[k * 4 + 1] = t;
                    heat[k * 4 + 2] = t;
                    heat[k * 4 + 3] = 1.0;
                }
                match fractadyne_export::write_png(&out, wa, ha, &heat, None) {
                    Ok(()) => println!("  heatmap → {}", out.display()),
                    Err(e) => eprintln!("  heatmap write failed: {e}"),
                }
            }
            (Some((wa, ha, _)), Some((wb, hb, _))) => {
                eprintln!("dimension mismatch: {wa}×{ha} vs {wb}×{hb}");
                return true;
            }
            _ => {
                eprintln!("failed to load one or both inputs (PNG/EXR only)");
                return true;
            }
        }
        return true;
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
        let Some(file) = args.get(i + 1) else {
            eprintln!("--crosscheck-f3 needs an EXR path");
            return true;
        };
        let val = |name: &str| args.iter().position(|a| a == name).and_then(|j| args.get(j + 1));
        let two = |name: &str| {
            args.iter()
                .position(|a| a == name)
                .and_then(|j| Some((args.get(j + 1)?, args.get(j + 2)?)))
        };
        // This mode is a CORRECTNESS ORACLE: it decides whether Fraktaler-3 and our independent
        // bignum dwell computation agree. A silently defaulted coordinate or magnification here
        // does not produce a wrong picture, it produces a wrong VERDICT — and one that reads as
        // "the other renderer disagrees with us" rather than "you mistyped a flag".
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
        let f3_zoom = val("--zoom-f3")
            .map(|s| {
                let z = crate::arg_parse::<f64>("--zoom-f3", s, "a number");
                // `f64::from_str` returns +inf for an overflowing literal instead of an error,
                // and the corpus's deepest locations are far past that. `spacing` would be 0 and
                // every sample would land on the same coordinate.
                if z.is_finite() && z > 0.0 {
                    z
                } else {
                    eprintln!(
                        "fractadyne: --zoom-f3: \"{s}\" is not a finite positive \
                         magnification (this comparison works in f64, so it cannot reach past \
                         ~1e308x)."
                    );
                    crate::exit(2)
                }
            })
            .unwrap_or(1.0);
        let max_iter = val("--iter")
            .map(|s| crate::arg_parse::<u32>("--iter", s, "a whole number"))
            .unwrap_or(10_000);
        let er = val("--er")
            .map(|s| crate::arg_parse::<f64>("--er", s, "a number"))
            .unwrap_or(256.0);
        let bailout2 = er * er;
        // Our magnification convention (height 3) vs F3's (height 4): mag = 0.75·zoom.
        let our_mag = 0.75 * f3_zoom;
        let prec = fractadyne_core::precision_for_magnification(our_mag).max(64);

        let Ok((w, h, nch)) = fractadyne_export::read_exr_channel_f32(std::path::Path::new(file), "N")
        else {
            eprintln!(
                "could not read EXR channel \"N\" from {file}. Channels present: {:?}\n\
                 (Fraktaler-3 batch must set render.exr_channels = [\"N0\"] and render.save_exr = true.)",
                fractadyne_export::list_exr_channels(std::path::Path::new(file)).unwrap_or_default()
            );
            return true;
        };
        let (wf, hf) = (w as f64, h as f64);
        // F3 pixel spacing; saved EXR is vertically flipped (vertical_flip defaults false
        // ⇒ save_exr flips), so saved row y maps to kernel row h-1-y.
        let spacing = 4.0 / f3_zoom / hf;
        let cx0 = &center[0];
        let cy0 = &center[1];
        // F3 interior sentinel: N0 = 0xFFFFFFFF (reads as ~4.29e9 in f32); exterior n = N0 - 1024.
        let is_interior_f3 = |v: f32| v > 2.0e9;
        let n_f3 = |v: f32| (v - 1024.0).round() as i64;

        eprintln!("Fraktaler-3 cross-check: {file}");
        eprintln!("  {w}×{h}, F3 zoom {f3_zoom:e} (our mag {our_mag:e}), iter {max_iter}, escape_radius {er}");
        eprintln!("  oracle: independent arbitrary-precision CPU dwell ({prec}-bit), bailout² {bailout2}");

        // F3 jitters every sample by a deterministic hash-based triangular sub-pixel offset
        // (anti-aliasing reconstruction, applied even at subframes=1). To compare integer
        // counts exactly we must sample our oracle at F3's *actual* point, not the pixel
        // centre — so replicate the kernel's jitter (hybrid.h: burtle_hash/triangle/wrap;
        // for subframe 0, dx == dy == triangle(burtle_hash(ix)/2³²)).
        let burtle_hash = |mut a: u32| -> u32 {
            a = a.wrapping_add(0x7ed5_5d16).wrapping_add(a << 12);
            a = (a ^ 0xc761_c23c) ^ (a >> 19);
            a = a.wrapping_add(0x1656_67b1).wrapping_add(a << 5);
            a = a.wrapping_add(0xd3a2_646c) ^ (a << 9);
            a = a.wrapping_add(0xfd70_46c5).wrapping_add(a << 3);
            a = (a ^ 0xb55a_4f09) ^ (a >> 16);
            a
        };
        let triangle = |h: f64| -> f64 {
            let orig = h * 2.0 - 1.0;
            let v = (orig / orig.abs().sqrt()).max(-1.0);
            v - if orig >= 0.0 { 1.0 } else { -1.0 }
        };
        let (wi, hi) = (w as i64, h as i64);

        // Pass 1: our oracle escape count at each pixel's exact (jittered) c (interior ⇒ -1).
        let oracle_n: Vec<i64> = (0..(w as usize * h as usize))
            .map(|k| {
                let (x, y) = ((k % w as usize) as i64, (k / w as usize) as i64);
                // Saved EXR is vertically flipped ⇒ kernel (i, j) = (x, h-1-y).
                let (ki, kj) = (x, hi - 1 - y);
                let ix = ((kj * wi + ki) & 0xffff_ffff) as u32;
                let jit = triangle(burtle_hash(ix) as f64 / 4_294_967_296.0);
                let ox = ((ki as f64 + 0.5 + jit) - wf / 2.0) * spacing;
                let oy = ((kj as f64 + 0.5 + jit) - hf / 2.0) * spacing;
                let cx = fractadyne_core::add_f64(cx0, ox, prec);
                let cy = fractadyne_core::add_f64(cy0, oy, prec);
                match fractadyne_core::naive_dwell_bf(&cx, &cy, max_iter, bailout2, prec) {
                    Some((n, _)) => n as i64,
                    None => -1,
                }
            })
            .collect();

        // Pass 2: compare, excluding ill-conditioned boundary pixels (a 4-neighbour
        // flips interior/exterior, or our oracle's count jumps by >2 — those pixels are
        // sub-pixel-sensitive and the two engines legitimately sample slightly differently).
        let idx = |x: usize, y: usize| y * w as usize + x;
        let (mut interior_ok, mut interior_tot) = (0u64, 0u64);
        let (mut exact, mut within1, mut smooth_tot, mut boundary, mut maxd) = (0u64, 0u64, 0u64, 0u64, 0i64);
        let (mut worst, mut worst_at) = (0i64, (0usize, 0usize));
        for y in 0..h as usize {
            for x in 0..w as usize {
                let k = idx(x, y);
                let o = oracle_n[k];
                let fi = is_interior_f3(nch[k]);
                let oi = o < 0;
                // Boundary detection via oracle neighbourhood.
                let mut steep = false;
                for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                    let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let no = oracle_n[idx(nx as usize, ny as usize)];
                    if (no < 0) != oi || (no >= 0 && o >= 0 && (no - o).abs() > 2) {
                        steep = true;
                    }
                }
                // Exclude ill-conditioned pixels from BOTH metrics. A pixel sitting on the
                // max-iteration cliff (a 4-neighbour flips interior/exterior) is `steep`, so
                // this also keeps near-cliff membership flips — legitimately ambiguous to the
                // last ULP — out of the membership stat.
                if steep {
                    boundary += 1;
                    continue;
                }
                if fi || oi {
                    interior_tot += 1;
                    if fi == oi {
                        interior_ok += 1;
                    }
                    continue;
                }
                smooth_tot += 1;
                let d = (n_f3(nch[k]) - o).abs();
                if d == 0 {
                    exact += 1;
                }
                if d <= 1 {
                    within1 += 1;
                }
                maxd = maxd.max(d);
                if d > worst {
                    worst = d;
                    worst_at = (x, y);
                }
            }
        }
        let pct = |a: u64, b: u64| if b == 0 { 100.0 } else { 100.0 * a as f64 / b as f64 };
        eprintln!(
            "  interior/exterior membership: {interior_ok}/{interior_tot} agree ({:.3}%)",
            pct(interior_ok, interior_tot)
        );
        eprintln!(
            "  smooth-region exterior counts: exact {exact}/{smooth_tot} ({:.3}%), |Δ|≤1 {within1}/{smooth_tot} ({:.3}%)",
            pct(exact, smooth_tot),
            pct(within1, smooth_tot)
        );
        eprintln!("  max |Δn| (non-boundary) {maxd} at pixel {worst_at:?}; boundary pixels excluded {boundary}");
        // PASS: membership ≥99.5%, and ≥99% of smooth-region exterior pixels match within 1.
        let pass = pct(interior_ok, interior_tot) >= 99.5 && pct(within1, smooth_tot) >= 99.0;
        println!("crosscheck-f3: {}", if pass { "PASS" } else { "FAIL" });
        crate::exit(if pass { 0 } else { 1 });
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
        use std::time::Instant;
        let val = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1));
        // ⚠ABSENT and SUPPLIED-BUT-UNREADABLE must not collapse into one default — that is the
        // silent-CLI-default class closed in `01add37`. `--iters 1O` (letter O) must be a message,
        // not a table of numbers for a run nobody asked for. `arg_parse` is the shared helper.
        let scale: f64 = match val("--iters") {
            None => 1.0,
            Some(s) => crate::arg_parse("--iters", s, "a positive number"),
        };
        if !scale.is_finite() || scale <= 0.0 {
            eprintln!("fractadyne: --iters must be positive and finite (got {scale}).");
            crate::exit(2);
        }
        // An INTERIOR point (well inside the main cardioid), irrational so every limb is
        // populated and the multiplies do real carry work — a dyadic centre like -0.5 would
        // produce sparse mantissas and understate the cost.
        //
        // ⚠Interior, not boundary, and that choice is load-bearing: an interior orbit cannot
        // escape at ANY precision, so every row does the full `iters` of real bignum work. A
        // boundary centre escaped at 64 bits (where truncation makes it a different point) and
        // spent the rest of the row on infinities — fast, meaningless, and it made the whole
        // command exit 1 permanently. The escape guard below stays as the tripwire; it was seen
        // firing on exactly that case before this centre was chosen.
        const CX: &str = "-0.287935866751537269847162833107461";
        const CY: &str = "0.007071067811865475244008443621048";

        let backends = fractadyne_core::available_backends();
        println!("Reference-orbit cost per arbitrary-precision backend (CPU only, no GPU)");
        println!("build contains: {}", fractadyne_core::built_in_backends());
        println!(
            "comparing: {}",
            backends.iter().map(|b| b.name()).collect::<Vec<_>>().join(" vs ")
        );
        println!();

        let mut header = format!("{:>8} {:>7} {:>10} {:>14}", "bits", "words", "iters", "bounded");
        for b in &backends {
            header.push_str(&format!(" {:>14}", format!("{} ns/it", b.name())));
        }
        if backends.len() > 1 {
            header.push_str(&format!(" {:>9} {:>16}", "speedup", "identical?"));
        }
        println!("{header}");

        let mut any_unbounded = false;
        let mut any_divergence = false;
        let mut worst: f64 = f64::MAX;
        let mut best: f64 = 0.0;
        for &bits in &[64usize, 128, 256, 576, 1088, 2112, 3776, 8256] {
            // Fewer iterations as the precision (and so the per-step cost) grows.
            let base = if bits <= 256 {
                60_000.0
            } else if bits <= 1088 {
                20_000.0
            } else if bits <= 3776 {
                4_000.0
            } else {
                1_500.0
            };
            let iters = ((base * scale) as u32).max(16);
            let (Some(cx), Some(cy)) =
                (fractadyne_core::parse_bf_prec(CX, bits), fractadyne_core::parse_bf_prec(CY, bits))
            else {
                eprintln!("fractadyne: could not parse the benchmark centre at {bits} bits");
                crate::exit(2);
            };
            let z0 = fractadyne_core::BigFloat::from_f64(0.0, bits);

            // Every backend runs the SAME work in ONE process, warm-up then median of three.
            // Run-to-run variance on these is real, and a comparison across two builds would be
            // measuring the builds as much as the backends.
            let mut ns: Vec<f64> = Vec::with_capacity(backends.len());
            let mut orbits: Vec<Vec<[f32; 4]>> = Vec::with_capacity(backends.len());
            let mut mag = 0.0f64;
            let mut len = 0u32;
            for &b in &backends {
                let mut ts = Vec::with_capacity(3);
                let mut last_orbit = Vec::new();
                for rep in 0..4 {
                    let t = Instant::now();
                    let (orbit, l) = fractadyne_core::reference_orbit_in(
                        b,
                        &z0,
                        &z0,
                        &cx,
                        &cy,
                        fractadyne_core::formula::MANDELBROT,
                        iters,
                        bits,
                    );
                    let dt = t.elapsed().as_secs_f64();
                    if rep > 0 {
                        ts.push(dt);
                    }
                    len = l;
                    let last = orbit.last().copied().unwrap_or([0.0; 4]);
                    let (x, y) = fractadyne_core::sample_xy(&last);
                    mag = (x * x + y * y).sqrt();
                    last_orbit = orbit;
                }
                ts.sort_by(|a, b| a.partial_cmp(b).unwrap());
                ns.push(ts[ts.len() / 2] * 1.0e9 / len.max(1) as f64);
                orbits.push(last_orbit);
            }

            // A speed ratio between backends that computed DIFFERENT orbits is meaningless, and
            // the ratio is the number people quote. Assert identity per row rather than trusting
            // the unit tests to have happened to cover this precision.
            let identical = orbits.windows(2).all(|w| {
                w[0].len() == w[1].len()
                    && w[0].iter().zip(&w[1]).all(|(a, b)| {
                        a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
                    })
            });
            any_divergence |= !identical;

            // `reference_orbit` stops early on escape, so a short orbit IS the escape signal.
            let bounded = len >= iters && mag.is_finite() && mag <= 1.0e6;
            any_unbounded |= !bounded;
            let flag = if bounded { format!("|z|={mag:.2}") } else { format!("ESCAPED@{len}") };

            let mut line =
                format!("{:>8} {:>7} {:>10} {:>14}", bits, bits.div_ceil(64), len, flag);
            for v in &ns {
                line.push_str(&format!(" {v:>14.1}"));
            }
            if backends.len() > 1 {
                let sp = ns[0] / ns[ns.len() - 1];
                worst = worst.min(sp);
                best = best.max(sp);
                line.push_str(&format!(
                    " {:>8.2}x {:>16}",
                    sp,
                    if identical { "BYTE-IDENTICAL" } else { "*** DIFFERS ***" }
                ));
            }
            println!("{line}");
            if !bounded {
                println!(
                    "         ^^ INVALID ROW - an escaped orbit times infinity arithmetic, not bignum"
                );
            }
        }

        if backends.len() > 1 && worst.is_finite() {
            println!();
            println!(
                "speedup range ({} over {}): {worst:.2}x .. {best:.2}x",
                backends[backends.len() - 1].name(),
                backends[0].name()
            );
        }

        // Printed AFTER the work, from what actually iterated. `MIXED` is the CORRECT answer for
        // this command and a failure for `--selftest`: a comparison must exercise every backend,
        // whereas a verdict must be attributable to exactly one.
        println!();
        println!("backends exercised: {}", fractadyne_core::backend_status_line());
        println!("tunables: {}", crate::tunables::status_line());
        if any_divergence {
            println!();
            println!("BACKENDS DISAGREED - the ratios above compare different computations");
            crate::exit(1);
        }
        if any_unbounded {
            println!("\nat least one row is INVALID (see above) — do not quote this table");
            crate::exit(1);
        }
        crate::exit(0);
    }

    // Extreme-depth validation battery (no GPU, no external data): exercises the
    // arbitrary-precision arithmetic core at magnifications far beyond f64 range
    // (1e1000 … 1e1000000), via precision-doubling self-consistency + coordinate round-trip.
    // A per-pixel dwell oracle is infeasible this deep; these single-point checks are not.
    //   --validate-deep [--out report.md]
    if args.iter().any(|a| a == "--validate-deep") {
        use std::time::Instant;
        let out = args
            .iter()
            .position(|x| x == "--out" || x == "-o")
            .and_then(|j| args.get(j + 1))
            .map(std::path::PathBuf::from);
        // (decimal exponent, iteration count) — fewer iters as precision grows (cost ∝ bits·k).
        let battery: &[(f64, u32)] = &[(1_000.0, 20_000), (10_000.0, 4_000), (100_000.0, 800), (1_000_000.0, 200)];
        let guard = 256usize;
        let mut rows: Vec<String> = Vec::new();
        let mut all_ok = true;
        println!("Extreme-depth precision self-consistency (arbitrary-precision arithmetic core)");
        println!(
            "{:>12} {:>11} {:>7} {:>7} {:>13} {:>13} {:>9}  result",
            "magnif.", "bits", "limbs", "k", "agree(bits)", "rt(bits)", "time(s)"
        );
        for (exp, k) in battery.iter().copied() {
            let octaves = (exp * std::f64::consts::LN_10 / std::f64::consts::LN_2).ceil() as u64;
            let p = fractadyne_core::precision_for_octaves(octaves);
            let t = Instant::now();
            let agree = fractadyne_core::deep_consistency_bits(p, guard, k);
            let rt = fractadyne_core::deep_roundtrip_bits(p);
            let secs = t.elapsed().as_secs_f64();
            // Sound p-bit arithmetic agrees to ≈ p − log₂(k); allow a generous margin.
            let pass = agree >= p as i64 - 128 && rt >= p as i64 - 256;
            all_ok &= pass;
            let verdict = if pass { "PASS" } else { "FAIL" };
            println!(
                "      1e{:<5.0} {:>11} {:>7} {:>7} {:>13} {:>13} {:>9.2}  {}",
                exp, p, p / 64, k, agree, rt, secs, verdict
            );
            rows.push(format!(
                "| 1e{:.0} | {} | {} | {} | {} | {} | {:.2} | {} |",
                exp, p, p / 64, k, agree, rt, secs, verdict
            ));
        }
        if let Some(path) = out {
            let mut md = String::new();
            md.push_str("# Extreme-depth precision validation\n\n");
            md.push_str(&format!("Fractadyne {}\n\n", version_string()));
            md.push_str(
                "Precision-doubling self-consistency of the arbitrary-precision arithmetic core, at \
                 magnifications beyond `f64` range. Iterate `z²+c` (full-mantissa interior point) at \
                 precision `p` and at `p+256`; `agree` = leading base-2 bits that match (sound ≈ \
                 `p − log₂(k)`). `rt` = bits preserved through a decimal `to_string → parse` \
                 round-trip. No GPU, no external data.\n\n",
            );
            md.push_str("| magnification | bits | limbs | k iters | agree (bits) | round-trip (bits) | time (s) | result |\n");
            md.push_str("|---|---|---|---|---|---|---|---|\n");
            for r in &rows {
                md.push_str(r);
                md.push('\n');
            }
            md.push_str(&format!("\n**Overall: {}**\n", if all_ok { "PASS" } else { "FAIL" }));
            md.push_str(
                "\n## Scope\n\nThis validates the *arithmetic and precision machinery* at extreme \
                 bit-width (the depth-critical numerics), not a full rendered image: a per-pixel \
                 arbitrary-precision dwell oracle is computationally infeasible this deep, and \
                 rendering a specific feature at these magnitudes would need a center coordinate of \
                 roughly that many decimal digits. (The live renderer's scale is an extended-range \
                 `FloatExp`, not `f64`, so there is no 1e308× scale wall — the practical limit is \
                 coordinate precision + iteration/compute cost.) Independent per-pixel cross-checks \
                 (`--selftest`, `--crosscheck-f3`) cover the renderable depth range.\n",
            );
            if let Err(e) = std::fs::write(&path, md) {
                eprintln!("report write failed: {e}");
            } else {
                println!("report → {}", path.display());
            }
        }
        println!("validate-deep: {}", if all_ok { "PASS" } else { "FAIL" });
        crate::exit(if all_ok { 0 } else { 1 });
    }
    false
}

