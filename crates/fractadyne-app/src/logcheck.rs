//! `--logcheck [LOG]` — hold one session of a Fractadyne log to `validation/logcheck-rules.toml`
//! (design/live-render-robustness.md §6.5, W8).
//!
//! WHY. The live path's warnings — a blind frame budget, a pass in the lethal band, GPU timing that
//! stopped arriving — were written to the log and graded by nobody: a harness passed on its own
//! assertion while the log beside it said the budget had gone blind. And most harness verdicts went
//! to stderr only, so a run that was killed, wedged or watchdogged left a log that looked like any
//! other. Now every task logs `[fd-harness] begin <mode>` at startup and `[fd-exit] <mode> exit
//! <code>` at its end (`crate::exit`), and checks its own log there, where a broken rule turns a 0
//! into a 1. The same check runs OUT OF PROCESS over a log a dead run left behind — a verdict
//! computed only on the judged thread goes silent exactly when that thread wedges (P19) — and there
//! a task with no `[fd-exit]` is its own outcome, NO VERDICT, never a pass.
//!
//! It never re-decides a condition. Every rule counts lines the code already logs when ITS predicate
//! fired, so the predicate stays in one place (§6.5's second caution); the rules file only says how
//! many of them a session may hold.
//!
//! Exit codes: 0 PASS, 1 FAIL (a rule broken, a required verdict line missing, a crash), 2 VACUOUS
//! (nothing to judge, or the rules would not load), 3 NO VERDICT (a task that never reached its end).

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// The committed rules, compiled in: the check at a task's exit needs them with nothing to find on
/// disk, and a binary can never be judged by rules from another commit.
pub(crate) const BUILTIN_RULES: &str = include_str!("../../../validation/logcheck-rules.toml");

const SESSION_TAG: &str = "[fd-start] fractadyne ";
const BEGIN_TAG: &str = "[fd-harness] begin ";
const EXIT_TAG: &str = "[fd-exit] ";

/// This process logged its `[fd-harness] begin` line, so it is checked at exit.
static BEGUN: AtomicBool = AtomicBool::new(false);
/// `at_exit` has run (an exit can race a watchdog thread's).
static EXITED: AtomicBool = AtomicBool::new(false);

// ---------------------------------------------------------------------------------------------
// The rules file, read strictly
// ---------------------------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRules {
    schema: u32,
    rule: Vec<RawRule>,
    #[serde(default)]
    harness: Vec<Harness>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRule {
    name: String,
    token: String,
    #[serde(default)]
    also: Option<String>,
    #[serde(default)]
    max: Option<u64>,
    #[serde(default)]
    max_per_min: Option<f64>,
    why: String,
    #[serde(default)]
    instrument_ok: bool,
    #[serde(default)]
    mode: BTreeMap<String, RawModeBound>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawModeBound {
    #[serde(default)]
    max: Option<u64>,
    #[serde(default)]
    max_per_min: Option<f64>,
    why: String,
    #[serde(default)]
    known: bool,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Harness {
    pub(crate) mode: String,
    pub(crate) require: Vec<String>,
}

/// How many matching lines a session may hold.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Bound {
    Max(u64),
    PerMin(f64),
}

impl fmt::Display for Bound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bound::Max(m) => write!(f, "max {m}"),
            Bound::PerMin(r) => write!(f, "{r}/min"),
        }
    }
}

fn bound_of(max: Option<u64>, per_min: Option<f64>, what: &str) -> Result<Bound, String> {
    match (max, per_min) {
        (Some(m), None) => Ok(Bound::Max(m)),
        (None, Some(r)) if r.is_finite() && r > 0.0 => Ok(Bound::PerMin(r)),
        (None, Some(r)) => Err(format!("{what}: max_per_min must be a positive number, got {r}")),
        (None, None) => Err(format!("{what}: no bound (give max or max_per_min)")),
        (Some(_), Some(_)) => Err(format!("{what}: two bounds (give max OR max_per_min)")),
    }
}

pub(crate) struct ModeBound {
    pub(crate) bound: Bound,
    pub(crate) why: String,
    pub(crate) known: bool,
}

pub(crate) struct Rule {
    pub(crate) name: String,
    pub(crate) token: String,
    pub(crate) also: Option<String>,
    pub(crate) bound: Bound,
    pub(crate) why: String,
    pub(crate) instrument_ok: bool,
    pub(crate) mode: BTreeMap<String, ModeBound>,
}

impl Rule {
    fn matches(&self, line: &str) -> bool {
        line.contains(&self.token) && self.also.as_ref().is_none_or(|a| line.contains(a.as_str()))
    }
}

pub(crate) struct Rules {
    pub(crate) rules: Vec<Rule>,
    pub(crate) harness: Vec<Harness>,
}

/// Every mode a rule may name: each task's name, and `gui` for a session that ran none.
fn known_modes() -> BTreeSet<&'static str> {
    crate::task_mode_names().chain(std::iter::once("gui")).collect()
}

impl Rules {
    /// Parse and validate. Anything it does not understand is an ERROR, never a skipped rule.
    pub(crate) fn parse(text: &str) -> Result<Rules, String> {
        let raw: RawRules = toml::from_str(text).map_err(|e| e.to_string())?;
        if raw.schema != 1 {
            return Err(format!("schema {} (this build reads schema 1)", raw.schema));
        }
        if raw.rule.is_empty() {
            return Err("no [[rule]]".into());
        }
        let modes = known_modes();
        let mut names = BTreeSet::new();
        let mut rules = Vec::with_capacity(raw.rule.len());
        for r in raw.rule {
            let at = format!("rule '{}'", r.name);
            if r.name.trim().is_empty() {
                return Err("a rule with no name".into());
            }
            if !names.insert(r.name.clone()) {
                return Err(format!("{at}: the name is used twice"));
            }
            if r.token.trim().is_empty() || r.also.as_ref().is_some_and(|a| a.trim().is_empty()) {
                return Err(format!("{at}: an empty token"));
            }
            let bound = bound_of(r.max, r.max_per_min, &at)?;
            let mut mode = BTreeMap::new();
            for (m, mb) in r.mode {
                if !modes.contains(m.as_str()) {
                    return Err(format!("{at}: unknown mode '{m}' (a task name such as 'soak', or 'gui')"));
                }
                let b = bound_of(mb.max, mb.max_per_min, &format!("{at} mode.{m}"))?;
                if std::mem::discriminant(&b) != std::mem::discriminant(&bound) {
                    return Err(format!("{at} mode.{m}: a {b} bound on a rule bounded by {bound}"));
                }
                mode.insert(m, ModeBound { bound: b, why: mb.why, known: mb.known });
            }
            rules.push(Rule {
                name: r.name,
                token: r.token,
                also: r.also,
                bound,
                why: r.why,
                instrument_ok: r.instrument_ok,
                mode,
            });
        }
        let mut seen = BTreeSet::new();
        for h in &raw.harness {
            if !modes.contains(h.mode.as_str()) || h.mode == "gui" {
                return Err(format!("[[harness]] '{}': not a task name", h.mode));
            }
            if !seen.insert(h.mode.clone()) {
                return Err(format!("[[harness]] '{}': listed twice", h.mode));
            }
            if h.require.is_empty() || h.require.iter().any(|t| t.trim().is_empty()) {
                return Err(format!("[[harness]] '{}': require needs non-empty tokens", h.mode));
            }
        }
        Ok(Rules { rules, harness: raw.harness })
    }
}

// ---------------------------------------------------------------------------------------------
// Sessions
// ---------------------------------------------------------------------------------------------

/// One process's lines, from its `[fd-start] fractadyne … args:` banner to the next banner.
pub(crate) struct Session<'a> {
    pub(crate) args: &'a str,
    pub(crate) lines: Vec<&'a str>,
}

impl<'a> Session<'a> {
    /// The `[fd-harness] begin` line, which only a build from beta.120 on writes.
    fn begin(&self) -> Option<&'a str> {
        self.lines.iter().copied().find(|l| l.contains(BEGIN_TAG))
    }

    /// The task's name: from its begin line, else (an older log) from the `args:` of its banner.
    pub(crate) fn mode(&self) -> Option<&'a str> {
        match self.begin() {
            Some(l) => {
                let rest = &l[l.find(BEGIN_TAG).unwrap() + BEGIN_TAG.len()..];
                crate::task_mode_names().find(|m| rest.split_whitespace().next() == Some(*m))
            }
            None => {
                let argv: Vec<&str> = self.args.split_whitespace().collect();
                crate::task_mode_of(&argv)
            }
        }
    }

    /// The code from its `[fd-exit] <mode> exit <code>` line.
    pub(crate) fn exit_code(&self) -> Option<i32> {
        self.lines.iter().rev().find_map(|l| {
            let rest = &l[l.find(EXIT_TAG)? + EXIT_TAG.len()..];
            rest.rsplit_once(" exit ").and_then(|(_, c)| c.trim().parse().ok())
        })
    }

    /// A diagnostic instrument was armed: its begin line carries the tunables status.
    fn instrumented(&self) -> bool {
        self.begin().is_some_and(|l| l.contains("INSTRUMENT"))
    }

    /// The time of each line that matches `rule` — its own `[+   12.345s]` stamp, or the last one
    /// before it (a panic's message runs on over unstamped lines).
    fn stamps_of(&self, rule: &Rule) -> Vec<f64> {
        let mut t = 0.0;
        let mut out = Vec::new();
        for l in &self.lines {
            if let Some(s) = stamp_s(l) {
                t = s;
            }
            if rule.matches(l) {
                out.push(t);
            }
        }
        out
    }
}

/// The most of `times` (ascending) that fall in any one window of `window_s` seconds. A rate is
/// judged this way, not as an average over the session: the 2026-09-21 field session flipped its
/// GPU timing 58 times in ~35 s, which over the 860 s session averages to 4 a minute.
fn max_in_window(times: &[f64], window_s: f64) -> usize {
    let mut best = 0;
    let mut lo = 0;
    for hi in 0..times.len() {
        while times[hi] - times[lo] >= window_s {
            lo += 1;
        }
        best = best.max(hi - lo + 1);
    }
    best
}

fn stamp_s(line: &str) -> Option<f64> {
    let rest = line.strip_prefix("[+")?;
    let (num, _) = rest.split_once("s]")?;
    num.trim().parse().ok()
}

/// Split a log into sessions. Lines before the first banner belong to no session and are dropped.
pub(crate) fn split_sessions(text: &str) -> Vec<Session<'_>> {
    let mut out: Vec<Session> = Vec::new();
    for line in text.lines() {
        if line.contains(SESSION_TAG) {
            if let Some((_, args)) = line.split_once(" args:") {
                out.push(Session { args: args.trim(), lines: Vec::new() });
                continue;
            }
        }
        if let Some(s) = out.last_mut() {
            s.lines.push(line);
        }
    }
    out
}

/// A log and the slots it rotated into, oldest first, back to the one holding a session banner —
/// a long run's start can have rotated out of `fractadyne.log` (`diag::rotate_log`).
fn read_log(path: &Path, follow_rotation: bool) -> std::io::Result<String> {
    let mut text = String::from_utf8_lossy(&std::fs::read(path)?).into_owned();
    if follow_rotation {
        let dir = path.parent().unwrap_or(Path::new("."));
        for n in 1..=3 {
            if text.contains(SESSION_TAG) {
                break;
            }
            match std::fs::read(dir.join(format!("fractadyne.log.{n}"))) {
                Ok(b) => text = String::from_utf8_lossy(&b).into_owned() + &text,
                Err(_) => break,
            }
        }
    }
    Ok(text)
}

// ---------------------------------------------------------------------------------------------
// The check
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    Pass,
    Fail,
    Vacuous,
    NoVerdict,
}

impl Outcome {
    pub(crate) fn code(self) -> i32 {
        match self {
            Outcome::Pass => 0,
            Outcome::Fail => 1,
            Outcome::Vacuous => 2,
            Outcome::NoVerdict => 3,
        }
    }

    /// Worse first: a failure outranks a missing verdict, which outranks having nothing to judge.
    fn severity(self) -> u8 {
        match self {
            Outcome::Fail => 3,
            Outcome::NoVerdict => 2,
            Outcome::Vacuous => 1,
            Outcome::Pass => 0,
        }
    }

    fn word(self) -> &'static str {
        match self {
            Outcome::Pass => "PASS",
            Outcome::Fail => "FAIL",
            Outcome::Vacuous => "VACUOUS",
            Outcome::NoVerdict => "NO VERDICT",
        }
    }
}

pub(crate) struct Report {
    pub(crate) outcome: Outcome,
    pub(crate) lines: Vec<String>,
    pub(crate) mode: String,
}

impl Report {
    pub(crate) fn summary(&self) -> String {
        let fails = self.lines.iter().filter(|l| l.starts_with("FAIL")).count();
        let known = self.lines.iter().filter(|l| l.starts_with("KNOWN")).count();
        let mut s = format!("{} ({})", self.outcome.word(), self.mode);
        if fails > 0 {
            s.push_str(&format!(" — {fails} failure(s)"));
        }
        if known > 0 {
            s.push_str(&format!(" — {known} KNOWN pre-existing condition(s), not a clean pass"));
        }
        s
    }
}

/// Hold one session to the rules.
pub(crate) fn check(s: &Session, rules: &Rules) -> Report {
    let mode = s.mode();
    let key = mode.unwrap_or("gui");
    let instrumented = s.instrumented();
    let mut lines = Vec::new();
    let mut fail = false;
    let mut no_verdict = false;

    for rule in &rules.rules {
        let times = s.stamps_of(rule);
        let n = times.len() as u64;
        if n == 0 {
            continue;
        }
        let in_a_minute = max_in_window(&times, 60.0) as f64;
        let over = |b: Bound| match b {
            Bound::Max(m) => n > m,
            Bound::PerMin(r) => in_a_minute > r,
        };
        let seen = match rule.bound {
            Bound::Max(_) => format!("{n} line(s)"),
            Bound::PerMin(_) => format!("{n} line(s), at most {in_a_minute} in one minute"),
        };
        let (bound, why, known) = match rule.mode.get(key) {
            Some(mb) => (mb.bound, mb.why.as_str(), mb.known),
            None => (rule.bound, rule.why.as_str(), false),
        };
        let name = &rule.name;
        if over(bound) {
            if rule.instrument_ok && instrumented {
                lines.push(format!("INSTRUMENT {name}: {seen} — allowed while an instrument is armed ({})", rule.why));
            } else {
                fail = true;
                lines.push(format!("FAIL {name}: {seen}, over {bound} — {}", rule.why));
            }
        } else if known && over(rule.bound) {
            lines.push(format!(
                "KNOWN {name}: {seen}, over the default {} but within {key}'s {bound} — {why}",
                rule.bound
            ));
        } else {
            lines.push(format!("ok {name}: {seen}, within {bound}"));
        }
    }

    if mode.is_some() {
        match (s.begin(), s.exit_code()) {
            (None, _) => lines.push(
                "legacy log: no [fd-harness] line (a build before beta.120), so whether the run reached its \
                 verdict is not judged"
                    .into(),
            ),
            (Some(_), None) => {
                no_verdict = true;
                lines.push(
                    "NO VERDICT: no [fd-exit] line — the run was killed, wedged or crashed before its end".into(),
                );
            }
            (Some(_), Some(c)) => {
                lines.push(format!("exit code {c}"));
                for h in rules.harness.iter().filter(|h| h.mode == key) {
                    for tok in &h.require {
                        if !s.lines.iter().any(|l| l.contains(tok.as_str())) {
                            fail = true;
                            lines.push(format!(
                                "FAIL no verdict line '{tok}': the {key} harness logs it on every run it finishes"
                            ));
                        }
                    }
                }
            }
        }
    }

    let outcome = if fail {
        Outcome::Fail
    } else if no_verdict {
        Outcome::NoVerdict
    } else if s.lines.is_empty() {
        Outcome::Vacuous
    } else {
        Outcome::Pass
    };
    Report { outcome, lines, mode: key.to_string() }
}

// ---------------------------------------------------------------------------------------------
// In process: the begin line and the check at exit
// ---------------------------------------------------------------------------------------------

/// `[fd-harness] begin <mode> pid <pid> — tunables: <status>`, logged by `main` for every task.
/// The pid is how `at_exit` finds its OWN session: `--torture` runs its rungs as child processes
/// that can write the same log, so the last session in it need not be this one.
pub(crate) fn harness_begin(mode: &str, tunables: &str) {
    crate::diag::log_line("harness", &format!("begin {mode} pid {} — tunables: {tunables}", std::process::id()));
    BEGUN.store(true, Ordering::Relaxed);
}

/// Called by `crate::exit` — every deliberate exit. For a task that logged its begin line: log
/// `[fd-exit]`, check this session's log, print the verdict, and return the exit code to use —
/// `code`, or 1 when `code` was 0 and the log broke a rule. Anything else returns `code` untouched.
pub(crate) fn at_exit(code: i32) -> i32 {
    let Some(mode) = crate::task_mode() else { return code };
    if !BEGUN.load(Ordering::Relaxed) || EXITED.swap(true, Ordering::SeqCst) {
        return code;
    }
    crate::diag::log_line("exit", &format!("{mode} exit {code}"));
    let Some(path) = crate::diag::log_path() else { return code };
    let rules = match Rules::parse(BUILTIN_RULES) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("logcheck: the built-in rules do not load ({e}); this run's log was not checked");
            return code;
        }
    };
    let text = match read_log(&path, true) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("logcheck: cannot read {}: {e}; this run's log was not checked", path.display());
            return code;
        }
    };
    let sessions = split_sessions(&text);
    let me = format!(" pid {} ", std::process::id());
    let own = sessions.iter().rev().find(|s| s.begin().is_some_and(|b| b.contains(&me)));
    let Some(session) = own else {
        eprintln!("logcheck: VACUOUS — this run's session (pid {}) is not in {}", std::process::id(), path.display());
        return code;
    };
    let report = check(session, &rules);
    for l in report.lines.iter().filter(|l| !l.starts_with("ok ")) {
        eprintln!("logcheck: {l}");
    }
    eprintln!("logcheck: {}", report.summary());
    crate::diag::log_line("logcheck", &report.summary());
    if code == 0 && report.outcome == Outcome::Fail {
        eprintln!("logcheck: exit code 0 becomes 1 — the run's own log broke a rule (validation/logcheck-rules.toml)");
        1
    } else {
        code
    }
}

// ---------------------------------------------------------------------------------------------
// Out of process: `--logcheck [LOG] [--logcheck-rules FILE] [--logcheck-all]`
// ---------------------------------------------------------------------------------------------

/// Returns the exit code: the worst outcome over the sessions checked.
pub(crate) fn run_cli(args: &[String]) -> i32 {
    let val = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .filter(|s| !s.starts_with("--"))
            .cloned()
    };
    let rules_text = match val("--logcheck-rules") {
        Some(p) => match std::fs::read_to_string(&p) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("fractadyne: --logcheck-rules: cannot read {p}: {e}");
                return 2;
            }
        },
        None => BUILTIN_RULES.to_string(),
    };
    let rules = match Rules::parse(&rules_text) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("fractadyne: --logcheck: the rules do not load: {e}");
            return 2;
        }
    };
    // With no LOG, the app's own log — whose last session is THIS invocation, skipped below.
    let (path, own_log): (PathBuf, bool) = match val("--logcheck") {
        Some(p) => (PathBuf::from(p), false),
        None => match crate::diag::log_path() {
            Some(p) => (p, true),
            None => {
                eprintln!("fractadyne: --logcheck: no log path (give one: --logcheck path\\to\\fractadyne.log)");
                return 2;
            }
        },
    };
    let text = match read_log(&path, own_log) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("fractadyne: --logcheck: cannot read {}: {e}", path.display());
            return 2;
        }
    };
    let sessions: Vec<Session> =
        split_sessions(&text).into_iter().filter(|s| !s.args.contains("--logcheck")).collect();
    println!("logcheck: {} — {} session(s)", path.display(), sessions.len());
    if sessions.is_empty() {
        println!("logcheck: VACUOUS — no session to judge");
        return Outcome::Vacuous.code();
    }
    let all = args.iter().any(|a| a == "--logcheck-all");
    let first = if all { 0 } else { sessions.len() - 1 };
    let mut worst = Outcome::Pass;
    for (i, s) in sessions.iter().enumerate().skip(first) {
        let r = check(s, &rules);
        println!("\nsession {} of {} — args: {}", i + 1, sessions.len(), if s.args.is_empty() { "(none)" } else { s.args });
        for l in &r.lines {
            println!("  {l}");
        }
        println!("  {}", r.summary());
        if r.outcome.severity() > worst.severity() {
            worst = r.outcome;
        }
    }
    worst.code()
}

#[cfg(test)]
#[path = "logcheck_tests.rs"]
mod tests;
