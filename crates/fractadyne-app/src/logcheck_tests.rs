use super::{check, split_sessions, Bound, Outcome, Rules, BUILTIN_RULES};

fn rules() -> Rules {
    Rules::parse(BUILTIN_RULES).expect("the committed rules load")
}

/// A session: its banner, then `body` (one line per entry).
fn log(args: &str, body: &[&str]) -> String {
    let mut s = format!("[+    0.001s] [fd-start] fractadyne 0.2.41-beta.120 (build 1, gabc) — 2026-09-26 — args: {args}\n");
    for l in body {
        s.push_str(l);
        s.push('\n');
    }
    s
}

fn judge(text: &str) -> super::Report {
    let r = rules();
    let sessions = split_sessions(text);
    check(sessions.last().expect("a session"), &r)
}

const BEGIN_SOAK: &str = "[+    0.010s] [fd-harness] begin soak pid 7 — tunables: stock";
const EXIT_0: &str = "[+   90.000s] [fd-exit] soak exit 0";
const SOAK_VERDICTS: [&str; 3] = [
    "[+   89.000s] [fd-verdict] soak-regime: NOT ENTERED — …",
    "[+   89.000s] [fd-verdict] soak-stall: no slow frame …",
    "[+   89.000s] [fd-verdict] soak-liveness: PASS (frames advanced in every window, memory held)",
];

fn finished_soak(extra: &[&str]) -> String {
    let mut body = vec![BEGIN_SOAK];
    body.extend_from_slice(extra);
    body.extend_from_slice(&SOAK_VERDICTS);
    body.push(EXIT_0);
    log("--soak 90 --soak-depth session", &body)
}

#[test]
fn the_committed_rules_load() {
    let r = rules();
    assert!(r.rules.len() >= 10, "{} rules", r.rules.len());
    assert!(r.harness.iter().any(|h| h.mode == "soak"));
}

/// §6.5's first caution: a rule must match a line the code can actually emit. Every token (and
/// `also`, and every required verdict token) must appear in this crate's source — with a leading
/// `[fd-<category>] ` checked as the category literal, since `log_line` composes that prefix.
#[test]
fn every_token_is_something_the_code_logs() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut src = String::new();
    let mut stack = vec![src_dir];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "rs")
                && !p.file_name().unwrap().to_string_lossy().contains("tests")
            {
                src.push_str(&std::fs::read_to_string(&p).unwrap());
            }
        }
    }
    let present = |tok: &str| -> Result<(), String> {
        let (cat, rest) = match tok.strip_prefix("[fd-").and_then(|t| t.split_once("] ")) {
            Some((cat, rest)) => (Some(cat), rest),
            None => match tok.strip_prefix("[fd-").and_then(|t| t.strip_suffix(']')) {
                Some(cat) => (Some(cat), ""),
                None => (None, tok),
            },
        };
        if let Some(cat) = cat {
            let as_log_line = format!("log_line(\"{cat}\"");
            let as_literal = format!("[fd-{cat}]");
            if !src.contains(&as_log_line) && !src.contains(&as_literal) && !src.contains(&format!("\"{cat}\"")) {
                return Err(format!("no [fd-{cat}] category in the source"));
            }
        }
        if !rest.is_empty() && !src.contains(rest) {
            return Err(format!("'{rest}' appears nowhere in the source"));
        }
        Ok(())
    };
    // The control: a check that cannot fail is not a check.
    assert!(present("xyzzy never logged by anything").is_err());
    assert!(present("[fd-xyzzy] begin").is_err());
    let r = rules();
    for rule in &r.rules {
        present(&rule.token).unwrap_or_else(|e| panic!("rule {}: token {e}", rule.name));
        if let Some(a) = &rule.also {
            present(a).unwrap_or_else(|e| panic!("rule {}: also {e}", rule.name));
        }
    }
    for h in &r.harness {
        for tok in &h.require {
            present(tok).unwrap_or_else(|e| panic!("harness {}: {e}", h.mode));
        }
    }
}

/// Read STRICTLY: each of these must be a load error, never a quietly skipped rule.
#[test]
fn a_rules_file_it_does_not_understand_does_not_load() {
    let rule = |body: &str| format!("schema = 1\n[[rule]]\nname = \"r\"\ntoken = \"x\"\nwhy = \"w\"\n{body}\n");
    let cases: [(&str, String); 10] = [
        ("unknown top-level key", format!("{}\nextra = 1", rule("max = 0"))),
        ("unknown rule key", rule("max = 0\nmaks = 1")),
        ("no bound", rule("")),
        ("two bounds", rule("max = 0\nmax_per_min = 1.0")),
        ("unknown mode", rule("max = 0\nmode.nosuchharness = { max = 1, why = \"w\" }")),
        ("mode bound of the other kind", rule("max = 0\nmode.soak = { max_per_min = 1.0, why = \"w\" }")),
        ("unknown mode key", rule("max = 0\nmode.soak = { max = 1, why = \"w\", knwon = true }")),
        ("a later schema", rule("max = 0").replace("schema = 1", "schema = 2")),
        ("a non-positive rate", rule("max_per_min = 0.0")),
        (
            "a duplicate name",
            format!("{}[[rule]]\nname = \"r\"\ntoken = \"y\"\nwhy = \"w\"\nmax = 0\n", rule("max = 0")),
        ),
    ];
    for (why, text) in cases {
        assert!(Rules::parse(&text).is_err(), "{why} must not load:\n{text}");
    }
    assert!(Rules::parse(&rule("max = 0")).is_ok(), "the control must load");
}

#[test]
fn sessions_split_on_the_banner_and_name_their_harness() {
    let text = format!(
        "{}{}{}",
        log("--selftest", &["[+ 1.0s] [fd-harness] begin selftest pid 1 — tunables: stock"]),
        log("", &["[+ 1.0s] [fd-render] a GUI line"]),
        // An older log: no begin line, so the name comes from args — the recordtest abort child's.
        log("--recordtest 100000 --recordtest-abort-after 60", &["[+ 1.0s] [fd-render] x"]),
    );
    let s = split_sessions(&text);
    assert_eq!(s.len(), 3);
    assert_eq!(s[0].mode(), Some("selftest"));
    assert_eq!(s[1].mode(), None);
    assert_eq!(s[2].mode(), Some("recordtest"));
}

#[test]
fn a_clean_finished_harness_passes() {
    let r = judge(&finished_soak(&[]));
    assert_eq!(r.outcome, Outcome::Pass, "{:?}", r.lines);
    assert!(r.lines.iter().any(|l| l == "exit code 0"), "{:?}", r.lines);
}

#[test]
fn a_count_over_its_bound_fails() {
    let r = judge(&finished_soak(&["[+ 5.0s] [fd-render] ⚠FRAME BUDGET IS BLIND: view=0 12 frames in a row"]));
    assert_eq!(r.outcome, Outcome::Fail);
    assert!(r.lines.iter().any(|l| l.starts_with("FAIL budget-blind: 1 line(s), over max 0")), "{:?}", r.lines);
}

/// A rate is the most lines in any ONE minute: a burst fails however long the session, and the
/// same count spread out passes.
#[test]
fn a_rate_is_judged_on_its_worst_minute() {
    let starve = |t: f64| format!("[+ {t:.1}s] [fd-wgpu] no GPU iterate timing after 30 frames (TIMESTAMP_QUERY=true)");
    let session = |times: &[f64]| {
        let lines: Vec<String> = times.iter().map(|t| starve(*t)).collect();
        let mut body: Vec<&str> = lines.iter().map(String::as_str).collect();
        body.push("[+ 900.0s] [fd-render] the session goes on");
        log("", &body)
    };
    let six = judge(&session(&[500.0, 505.0, 510.0, 515.0, 520.0, 525.0]));
    assert_eq!(six.outcome, Outcome::Pass, "six in a minute is the bound: {:?}", six.lines);
    let burst = judge(&session(&[500.0, 505.0, 510.0, 515.0, 520.0, 525.0, 530.0]));
    assert_eq!(burst.outcome, Outcome::Fail, "{:?}", burst.lines);
    assert!(
        burst.lines.iter().any(|l| l.starts_with("FAIL timing-starved: 7 line(s), at most 7 in one minute, over 6/min")),
        "{:?}",
        burst.lines
    );
    let spread = judge(&session(&[100.0, 120.0, 140.0, 160.0, 180.0, 200.0, 220.0]));
    assert_eq!(spread.outcome, Outcome::Pass, "one every 20 s: {:?}", spread.lines);
}

/// The designed trip passes, and says nothing of KNOWN: it is an allowance, not a debt.
#[test]
fn a_mode_allowance_passes_its_harness_only() {
    let blind = "[+ 5.0s] [fd-render] ⚠FRAME BUDGET IS BLIND: view=0";
    let text = log(
        "--recordtest",
        &[
            "[+ 0.0s] [fd-harness] begin recordtest pid 1 — tunables: stock",
            blind,
            "[+ 9.0s] [fd-verdict] recordtest: PASS — …",
            "[+ 9.0s] [fd-exit] recordtest exit 0",
        ],
    );
    let r = judge(&text);
    assert_eq!(r.outcome, Outcome::Pass, "{:?}", r.lines);
    assert!(!r.lines.iter().any(|l| l.starts_with("KNOWN")), "{:?}", r.lines);
    assert_eq!(judge(&finished_soak(&[blind])).outcome, Outcome::Fail, "only recordtest has the allowance");
}

/// A known, filed condition passes inside its bound, and SAYS so; past its bound it fails.
#[test]
fn a_known_condition_is_reported_and_still_bounded() {
    let lethal = "[+ 5.0s] [fd-render] ⚠IN-FLIGHT PASS IN THE LETHAL BAND: view=0 size=1 band=9 acc=950ms";
    let livetest = |n: usize| {
        let mut body = vec!["[+ 0.0s] [fd-harness] begin livetest pid 1 — tunables: stock"];
        body.extend(std::iter::repeat_n(lethal, n));
        body.push("[+ 300.0s] [fd-exit] livetest exit 0");
        log("--livetest tours/x.toml", &body)
    };
    let r = judge(&livetest(9));
    assert_eq!(r.outcome, Outcome::Pass, "{:?}", r.lines);
    assert!(r.lines.iter().any(|l| l.starts_with("KNOWN lethal-band-in-flight: 9 line(s)")), "{:?}", r.lines);
    assert!(r.summary().contains("KNOWN"), "{}", r.summary());
    // The design's red-check: a TDR_LETHAL_MS=1 run sheds dozens (46 under --livetest).
    assert_eq!(judge(&livetest(46)).outcome, Outcome::Fail);
}

#[test]
fn an_armed_instrument_excuses_only_the_rules_that_say_so() {
    let text = log(
        "--soak 90 --soak-depth session",
        &[
            "[+ 0.0s] [fd-harness] begin soak pid 1 — tunables: INSTRUMENT FRACTADYNE_REF_ESCAPE_AT=655",
            "[+ 5.0s] [fd-render] ⚠LETHAL-BAND FRAME: view=0 gpu=950ms",
            SOAK_VERDICTS[0],
            SOAK_VERDICTS[1],
            SOAK_VERDICTS[2],
            EXIT_0,
        ],
    );
    let r = judge(&text);
    assert_eq!(r.outcome, Outcome::Pass, "{:?}", r.lines);
    assert!(r.lines.iter().any(|l| l.starts_with("INSTRUMENT lethal-band-frame")), "{:?}", r.lines);
    // A panic is not excused by an instrument.
    let crashed = text.replace(EXIT_0, "[+ 6.0s] [fd-panic] wgpu error: Validation Error");
    assert_eq!(judge(&crashed).outcome, Outcome::Fail);
}

/// A run with no `[fd-exit]` never reached its end: NO VERDICT, not a pass — unless it crashed,
/// which is a failure.
#[test]
fn a_run_that_never_finished_has_no_verdict() {
    let killed = log("--soak 90", &[BEGIN_SOAK, "[+ 40.0s] [fd-render] still going"]);
    let r = judge(&killed);
    assert_eq!(r.outcome, Outcome::NoVerdict, "{:?}", r.lines);
    assert_eq!(r.outcome.code(), 3);
    let lost = log("--soak 90", &[BEGIN_SOAK, "[+ 40.0s] [fd-wgpu] DEVICE LOST (Unknown): Device is lost"]);
    assert_eq!(judge(&lost).outcome, Outcome::Fail);
}

#[test]
fn a_finished_harness_without_its_verdict_line_fails() {
    let text = log("--soak 90", &[BEGIN_SOAK, SOAK_VERDICTS[0], SOAK_VERDICTS[2], EXIT_0]);
    let r = judge(&text);
    assert_eq!(r.outcome, Outcome::Fail, "{:?}", r.lines);
    assert!(r.lines.iter().any(|l| l.contains("no verdict line '[fd-verdict] soak-stall:'")), "{:?}", r.lines);
}

/// A log from before the begin/exit lines existed is judged on its rules, and says what it could
/// not judge.
#[test]
fn a_legacy_log_is_judged_on_its_rules_only() {
    let r = judge(&log("--soak 90", &["[+ 40.0s] [fd-render] slow frame 12"]));
    assert_eq!(r.outcome, Outcome::Pass, "{:?}", r.lines);
    assert!(r.lines.iter().any(|l| l.starts_with("legacy log")), "{:?}", r.lines);
}

/// The 2026-09-21 RX 6800 XT loss, in the shape its log had: an interactive session.
#[test]
fn the_field_loss_fails_on_what_went_wrong() {
    let mut body = vec![];
    let starve = "[+ 810.0s] [fd-wgpu] no GPU iterate timing after 30 frames (TIMESTAMP_QUERY=true)";
    body.extend(std::iter::repeat_n(starve, 58));
    body.push("[+ 834.1s] [fd-render] ⚠IN-FLIGHT PASS IN THE LETHAL BAND: view=0 size=40960 band=9 acc=1037ms");
    body.push("[+ 860.4s] [fd-wgpu] DEVICE LOST (Unknown): Device is lost");
    let r = judge(&log("", &body));
    assert_eq!(r.outcome, Outcome::Fail);
    for rule in ["timing-starved", "lethal-band-in-flight", "device-lost"] {
        assert!(r.lines.iter().any(|l| l.starts_with(&format!("FAIL {rule}"))), "{rule}: {:?}", r.lines);
    }
    assert_eq!(r.mode, "gui");
}

#[test]
fn bounds_print_as_the_file_writes_them() {
    assert_eq!(Bound::Max(0).to_string(), "max 0");
    assert_eq!(Bound::PerMin(6.0).to_string(), "6/min");
}
