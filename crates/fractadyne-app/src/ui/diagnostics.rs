//! Help → Diagnostics…: run the user-safe validation tests from the UI.
//!
//! The audience is **issue reporters and people testing on hardware we don't own** — not the
//! developer. Three consequences shape this module:
//!
//! 1. **No CLI gate.** The people who benefit most are exactly the ones who will never pass a
//!    flag, so the dialog is always in the Help menu. That is also what makes cross-GPU
//!    validation crowdsourceable instead of limited to cards we can buy.
//! 2. **Only the tests that mean something without context**: the self-test (does the maths hold
//!    on this GPU?), the UI test (does it draw and lay out correctly?), and the GPU arithmetic
//!    check (do the shader's extended-precision transforms survive this compiler?). The dev
//!    harnesses — `--livetest`, `--bench-matrix`, `--divetest`, `--juliadive` — stay CLI-only on
//!    purpose: a button for those produces confused bug reports, not information.
//!    `scripts/gpu-validate.*` is the power-user path and runs the full battery.
//!
//!    The GPU arithmetic check is deliberately **informational**, not pass/fail. On every NVIDIA
//!    stack tested it "fails" — the compiler folds the error-free transforms — and that failure is
//!    precisely the finding the project wants reported, not a fault in the user's machine. A green
//!    tick would be wrong and a red cross would scare the most common hardware into silence, so it
//!    reports "result captured" and asks for the report either way. It is the one-click form of the
//!    df32 corroboration request the announcement makes.
//! 3. **Results attach to an issue report**, upgrading "Report an issue…" from *here is my crash
//!    log* to *here is my crash log plus a machine-validated test result*.
//!
//! Like the render-script dialog, tests run as a **child process** (`current_exe --selftest`).
//! The reason is sharper here than there: these tests deliberately push the GPU, and a device
//! loss during one must kill the test, never the session the user is about to file a report from.
//!
//! ## Stream handling (learned empirically — don't "simplify" it)
//!
//! The self-test writes its **per-check lines to stderr** (it logs through `env_logger`) and its
//! **final verdict to stdout**. Reading only stdout gives a dialog that sits silent for fifteen
//! seconds and then prints an answer; reading only stderr never sees the verdict. So both streams
//! are pumped, and lines are classified by content rather than by which pipe they arrived on.

use crate::FractadyneApp;
use std::path::PathBuf;

/// Which test a run is executing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum DiagTest {
    SelfTest,
    UiTest,
    GpuTest,
}

impl DiagTest {
    /// The three tests, in the order they appear in the dialog.
    pub(crate) const ALL: [DiagTest; 3] = [DiagTest::SelfTest, DiagTest::UiTest, DiagTest::GpuTest];

    pub(crate) fn label(self) -> &'static str {
        match self {
            DiagTest::SelfTest => "Self-test",
            DiagTest::UiTest => "UI test",
            DiagTest::GpuTest => "GPU arithmetic check",
        }
    }

    /// Button text. Spelled out rather than derived from `label()` — lowercasing that gave
    /// "Run ui test", which reads as a typo.
    pub(crate) fn button(self) -> &'static str {
        match self {
            DiagTest::SelfTest => "Run self-test",
            DiagTest::UiTest => "Run UI test",
            DiagTest::GpuTest => "Run GPU arithmetic check",
        }
    }

    /// One sentence a non-developer can act on.
    pub(crate) fn blurb(self) -> &'static str {
        match self {
            DiagTest::SelfTest => {
                "Checks the maths and rendering on your GPU against known-correct results. \
                 Takes about 15 seconds."
            }
            DiagTest::UiTest => {
                "Walks the interface and the live view, capturing screenshots at several zoom \
                 depths. Takes a minute or two and opens windows while it runs."
            }
            DiagTest::GpuTest => {
                "Checks the shader's double-float arithmetic against known-correct values on every \
                 graphics backend your machine offers. Takes a few seconds, opens no window. If it \
                 reports a failure, that is a genuine finding we would like to see — please attach \
                 it to a report."
            }
        }
    }

    /// An informational test has no right answer to grade — it captures a result to send back
    /// rather than passing or failing. The GPU arithmetic check is one: on NVIDIA it "fails" by
    /// design (the compiler folds the transforms), and that is the datum, not a fault, so the
    /// dialog must not paint it red. See the module docs.
    pub(crate) fn is_informational(self) -> bool {
        matches!(self, DiagTest::GpuTest)
    }
}

/// A line from a running test child, tagged by stream. Both streams matter (see module docs), so
/// the tag is used only to keep a failure message from being buried, never to decide what to show.
pub(crate) enum DiagLine {
    Out(String),
    Err(String),
}

/// What a finished run concluded.
#[derive(Clone)]
pub(crate) struct DiagVerdict {
    pub(crate) test: DiagTest,
    /// The headline the test itself printed, verbatim — e.g. `checks 113/113, goldens 17/17 — OK`.
    pub(crate) headline: String,
    pub(crate) ok: bool,
    /// Report file or screenshot folder, when the run produced one.
    pub(crate) artifact: Option<PathBuf>,
}

/// Dialog + running-child state.
#[derive(Default)]
pub(crate) struct DiagnosticsUi {
    pub(crate) open: bool,
    /// The test currently running, if any.
    pub(crate) running: Option<DiagTest>,
    /// Latest interesting line, shown live.
    pub(crate) progress: String,
    /// Checks/steps observed so far. The totals aren't known until the end, so this drives a
    /// count rather than a bar — an honest "47 checks done" beats a bar against a guessed total.
    pub(crate) seen: u32,
    /// First error line — kept because later lines are usually consequences of it.
    pub(crate) error: Option<String>,
    /// Where this run's artifact will land.
    pub(crate) artifact: Option<PathBuf>,
    /// Verdict of the most recent finished run (offered to the issue report).
    pub(crate) last: Option<DiagVerdict>,
    pub(crate) child: Option<std::process::Child>,
    pub(crate) rx: Option<std::sync::mpsc::Receiver<DiagLine>>,
}

/// Parse the self-test's final line: `checks 113/113, goldens 17/17 — OK` (or
/// `— FAILURES PRESENT`). Returns `(checks_passed, checks_total, goldens_passed, goldens_total)`.
///
/// Deliberately tolerant about what follows the counts: the trailing verdict word has changed
/// before, and a parser that insists on it would silently stop recognising the line.
pub(crate) fn parse_selftest_verdict(line: &str) -> Option<(u32, u32, u32, u32)> {
    let l = line.trim();
    let rest = l.strip_prefix("checks ")?;
    let (checks, rest) = rest.split_once(',')?;
    let (cp, ct) = checks.trim().split_once('/')?;
    let goldens = rest.trim().strip_prefix("goldens ")?;
    // Stop at the first non-count character so "17/17 — OK" and "17/17" both parse.
    let g: String = goldens
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '/')
        .collect();
    let (gp, gt) = g.split_once('/')?;
    Some((
        cp.trim().parse().ok()?,
        ct.trim().parse().ok()?,
        gp.trim().parse().ok()?,
        gt.trim().parse().ok()?,
    ))
}

/// Parse the UI test's final line:
/// `=== --uitest complete: 25 steps, 25 pass / 0 warn / 0 fail ===`.
/// Returns `(pass, warn, fail)`.
pub(crate) fn parse_uitest_verdict(line: &str) -> Option<(u32, u32, u32)> {
    let l = line.trim();
    if !l.contains("--uitest complete") {
        return None;
    }
    let after = l.split_once("steps,")?.1;
    let nums: Vec<u32> = after
        .split_whitespace()
        .filter_map(|t| t.parse::<u32>().ok())
        .collect();
    if nums.len() < 3 {
        return None;
    }
    Some((nums[0], nums[1], nums[2]))
}

/// Parse the GPU arithmetic sweep's final line into `(backends_tested, backends_failed)`:
/// `"3 backend(s) tested, all sound."` → `(3, 0)`;
/// `"2 of 3 backend(s) FAILED. …"` → `(3, 2)`;
/// `"No usable backend found — nothing tested."` → `(0, 0)`.
///
/// Both quantities matter to the headline, and "failed" is not a fault here (see
/// [`DiagTest::is_informational`]) — it is what the df32 request is asking people to report.
pub(crate) fn parse_gputest_verdict(line: &str) -> Option<(u32, u32)> {
    let l = line.trim();
    if let Some(rest) = l.strip_suffix("backend(s) tested, all sound.") {
        return Some((rest.trim().parse().ok()?, 0));
    }
    if let Some(idx) = l.find(" of ") {
        if l[idx..].contains("backend(s) FAILED") {
            let failed: u32 = l[..idx].trim().parse().ok()?;
            let ran: String = l[idx + 4..].chars().take_while(|c| c.is_ascii_digit()).collect();
            return Some((ran.parse().ok()?, failed));
        }
    }
    if l.starts_with("No usable backend found") {
        return Some((0, 0));
    }
    None
}

/// The one-line headline shown for a finished GPU arithmetic run and carried into a report.
/// Synthesised rather than taken from the sweep's own last line, whose FAILED variant wraps mid
/// sentence and reads badly on its own.
pub(crate) fn gputest_headline(ran: u32, failed: u32) -> String {
    match (ran, failed) {
        (0, _) => "no usable graphics backend — nothing tested".to_string(),
        (n, 0) => format!("{n} backend(s) tested — all double-float transforms intact"),
        (n, f) => format!(
            "{n} backend(s) tested — {f} fold the error-free transforms (expected on NVIDIA; please attach this)"
        ),
    }
}

/// Is this a per-check/per-step progress line worth counting and showing? The GPU sweep has no
/// per-check markers, but it prints a `── <backend>` header per device, which serves the same
/// role — it lets the count advance and names the backend under test.
pub(crate) fn is_progress_line(line: &str) -> bool {
    let l = line.trim_start();
    l.starts_with("[selftest") || l.starts_with("[uitest") || l.starts_with("=== step")
        || l.starts_with("──")
}

#[cfg(test)]
mod tests;

impl FractadyneApp {
    /// Where this run's artifacts go: a `diagnostics/` folder beside the session, so a user can
    /// find them and a report can point at them. Falls back to the temp dir when there is no
    /// config dir (a portable/sandboxed run).
    fn diagnostics_dir(&self) -> PathBuf {
        let base = fractadyne_state::config_dir().unwrap_or_else(std::env::temp_dir);
        base.join("diagnostics")
    }

    /// Launch a test as a child process and start streaming its output.
    pub(crate) fn start_diagnostic(&mut self, test: DiagTest) {
        use std::io::{BufRead, BufReader};
        if self.diagnostics.running.is_some() {
            return; // one at a time; the buttons are disabled, this is belt-and-braces
        }
        let exe = match std::env::current_exe() {
            Ok(e) => e,
            Err(e) => {
                self.diagnostics.error = Some(format!("Cannot find the executable: {e}"));
                return;
            }
        };
        let dir = self.diagnostics_dir();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            self.diagnostics.error = Some(format!("Cannot create {}: {e}", dir.display()));
            return;
        }
        let stamp = crate::FractadyneApp::file_stamp(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        );
        let (args, artifact) = match test {
            DiagTest::SelfTest => {
                let out = dir.join(format!("selftest-{stamp}.md"));
                (
                    vec![
                        "--selftest".to_string(),
                        "--out".to_string(),
                        out.display().to_string(),
                    ],
                    Some(out),
                )
            }
            // `--uitest DIR` creates its own timestamped folder underneath; the exact name is in
            // the child's output, so the artifact starts as the parent and is refined on finish.
            DiagTest::UiTest => (
                vec!["--uitest".to_string(), dir.display().to_string()],
                Some(dir.clone()),
            ),
            // `--gputest --out FILE` writes the verbatim table (see gputest.rs): the streamed
            // copy is trimmed per line and loses the column alignment, so the on-disk report the
            // user opens and attaches comes from the child, not from captured stdout.
            DiagTest::GpuTest => {
                let out = dir.join(format!("gputest-{stamp}.txt"));
                (
                    vec![
                        "--gputest".to_string(),
                        "--out".to_string(),
                        out.display().to_string(),
                    ],
                    Some(out),
                )
            }
        };

        // A fresh run must not inherit the previous one's failure or counts.
        self.diagnostics.progress.clear();
        self.diagnostics.seen = 0;
        self.diagnostics.error = None;
        self.diagnostics.artifact = artifact;

        let child = std::process::Command::new(exe)
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();
        match child {
            Ok(mut c) => {
                let (tx, rx) = std::sync::mpsc::channel();
                self.diagnostics.rx = Some(rx);
                if let Some(out) = c.stdout.take() {
                    let tx = tx.clone();
                    std::thread::spawn(move || {
                        for line in BufReader::new(out).lines().map_while(Result::ok) {
                            let l = line.trim().to_string();
                            if !l.is_empty() && tx.send(DiagLine::Out(l)).is_err() {
                                break;
                            }
                        }
                    });
                }
                if let Some(err) = c.stderr.take() {
                    std::thread::spawn(move || {
                        for line in BufReader::new(err).lines().map_while(Result::ok) {
                            let l = line.trim().to_string();
                            if !l.is_empty() && tx.send(DiagLine::Err(l)).is_err() {
                                break;
                            }
                        }
                    });
                }
                self.diagnostics.child = Some(c);
                self.diagnostics.running = Some(test);
            }
            Err(e) => self.diagnostics.error = Some(format!("Could not start the test: {e}")),
        }
    }

    /// Drain the child's output and notice when it exits. Called every frame while a test runs.
    pub(crate) fn poll_diagnostics(&mut self, ctx: &egui::Context) {
        if self.diagnostics.running.is_none() {
            return;
        }
        let mut verdict_line: Option<String> = None;
        if let Some(rx) = &self.diagnostics.rx {
            while let Ok(line) = rx.try_recv() {
                let (text, from_err) = match line {
                    DiagLine::Out(l) => (l, false),
                    DiagLine::Err(l) => (l, true),
                };
                // Verdicts can arrive on either stream depending on the test — classify by
                // content, never by pipe (see module docs). The GPU sweep's own last line wraps
                // mid-sentence on failure, so it is replaced with a clean synthesised headline.
                if let Some((ran, failed)) = parse_gputest_verdict(&text) {
                    verdict_line = Some(gputest_headline(ran, failed));
                } else if parse_selftest_verdict(&text).is_some()
                    || parse_uitest_verdict(&text).is_some()
                {
                    verdict_line = Some(text.clone());
                }
                if is_progress_line(&text) {
                    self.diagnostics.seen += 1;
                    self.diagnostics.progress = text;
                } else if from_err
                    && self.diagnostics.error.is_none()
                    && (text.contains("panic") || text.contains("FAILED"))
                {
                    self.diagnostics.error = Some(text);
                }
            }
        }
        if let Some(v) = verdict_line {
            self.diagnostics.progress = v;
        }

        let finished = match self.diagnostics.child.as_mut() {
            Some(child) => match child.try_wait() {
                Ok(Some(st)) => Some(st.success()),
                Ok(None) => None,
                Err(_) => Some(false),
            },
            None => None,
        };
        if let Some(success) = finished {
            let test = self.diagnostics.running.take().unwrap_or(DiagTest::SelfTest);
            self.diagnostics.child = None;
            self.diagnostics.rx = None;
            let headline = if self.diagnostics.progress.is_empty() {
                "the test produced no summary line".to_string()
            } else {
                self.diagnostics.progress.clone()
            };
            // Trust the test's own verdict line over the exit code where we have one: the
            // self-test exits non-zero on golden mismatches, which on non-reference hardware are
            // expected rather than failures, and a red banner there teaches testers to ignore it.
            // An informational test is never graded — it captures a result to send back — so it is
            // always "ok" for the purpose of colour; the GPU sweep exits non-zero precisely when it
            // has found the thing worth reporting, which must not read as a fault.
            let ok = if test.is_informational() {
                true
            } else {
                match (
                    parse_selftest_verdict(&headline),
                    parse_uitest_verdict(&headline),
                ) {
                    (Some((cp, ct, _, _)), _) => cp == ct,
                    (_, Some((_, _, fail))) => fail == 0,
                    _ => success,
                }
            };
            self.diagnostics.last = Some(DiagVerdict {
                test,
                headline,
                ok,
                artifact: self.diagnostics.artifact.clone(),
            });
            ctx.request_repaint();
        } else {
            // Keep the UI ticking while the child works, otherwise the progress line only
            // advances when the mouse moves.
            ctx.request_repaint_after(std::time::Duration::from_millis(150));
        }
    }

    /// The Diagnostics window.
    pub(crate) fn draw_diagnostics_dialog(&mut self, ctx: &egui::Context) {
        if !self.diagnostics.open {
            return;
        }
        let mut open = self.diagnostics.open;
        let mut close = false;
        let running = self.diagnostics.running;
        let mut start: Option<DiagTest> = None;
        let mut open_artifact: Option<PathBuf> = None;
        let mut attach = false;

        egui::Window::new("Diagnostics")
            .open(&mut open)
            .resizable(true)
            .default_width(520.0)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(
                        "Run a test to check that Fractadyne works correctly on your hardware. \
                         Results are saved on your machine and can be attached to an issue \
                         report — nothing is sent anywhere on its own.",
                    )
                    .weak()
                    .small(),
                );
                ui.label(
                    egui::RichText::new(format!(
                        "Deep-zoom arithmetic: {}",
                        fractadyne_core::backend_status_line()
                    ))
                    .weak()
                    .small(),
                );
                ui.add_space(8.0);

                for test in DiagTest::ALL {
                    ui.horizontal(|ui| {
                        let busy = running.is_some();
                        let btn = ui.add_enabled(!busy, egui::Button::new(test.button()));
                        if btn.clicked() {
                            start = Some(test);
                        }
                        if running == Some(test) {
                            ui.spinner();
                        }
                    });
                    ui.label(egui::RichText::new(test.blurb()).weak().small());
                    ui.add_space(6.0);
                }

                if running.is_some() {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(format!("{} checks done", self.diagnostics.seen));
                    });
                    if !self.diagnostics.progress.is_empty() {
                        ui.label(
                            egui::RichText::new(&self.diagnostics.progress)
                                .weak()
                                .small(),
                        );
                    }
                }

                if let Some(v) = &self.diagnostics.last {
                    ui.separator();
                    // An informational test is neutral — never green/red — because its "failure"
                    // is the datum, not a fault (see DiagTest::is_informational).
                    let (colour, word) = if v.test.is_informational() {
                        (egui::Color32::from_rgb(0x9a, 0x9d, 0xa6), "result captured")
                    } else if v.ok {
                        (egui::Color32::from_rgb(0x4c, 0xaf, 0x50), "passed")
                    } else {
                        (egui::Color32::from_rgb(0xe5, 0x73, 0x73), "reported problems")
                    };
                    ui.label(
                        egui::RichText::new(format!("{} — {word}", v.test.label()))
                            .color(colour)
                            .strong(),
                    );
                    ui.label(egui::RichText::new(&v.headline).monospace().small());
                    if v.test.is_informational() {
                        ui.label(
                            egui::RichText::new(
                                "This one has no pass or fail: it records how your GPU computes, \
                                 and a reported failure is a real finding rather than a fault in \
                                 your machine. Either way, attaching it to a report is the most \
                                 useful thing you can do with it.",
                            )
                            .weak()
                            .small(),
                        );
                    } else if !v.ok {
                        ui.label(
                            egui::RichText::new(
                                "Some differences are expected on hardware other than the \
                                 reference GPU — image comparisons in particular. Attaching this \
                                 to an issue report is the most useful thing you can do with it.",
                            )
                            .weak()
                            .small(),
                        );
                    }
                    ui.horizontal(|ui| {
                        if let Some(p) = &v.artifact {
                            if p.exists() && ui.button("Open results").clicked() {
                                open_artifact = Some(p.clone());
                            }
                        }
                        if ui
                            .button("Attach to an issue report…")
                            .on_hover_text(
                                "Opens the report dialog with this result included, so the issue \
                                 carries a machine-validated test rather than just a description",
                            )
                            .clicked()
                        {
                            attach = true;
                        }
                    });
                }

                if let Some(e) = &self.diagnostics.error {
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(0xe5, 0x73, 0x73), e);
                }

                // ⭐The third of the three ways to reach the console output (flag, env var, here).
                // It lives in Diagnostics rather than File ▸ Settings because it is not a
                // preference about how the app looks — it is an instrument, and this is the window
                // someone chasing a problem already has open.
                ui.separator();
                let mut console = crate::diag::console_on();
                if ui
                    .checkbox(&mut console, "Print diagnostics to the console")
                    .on_hover_text(
                        "Show the [fd-…] lines on stderr from now on. Off by default when \
                         Fractadyne is launched without arguments. This changes nothing about what \
                         is RECORDED — the log file always gets every line — so leave it off unless \
                         you are watching a terminal. Persists for this session only; use the \
                         --console flag or FRACTADYNE_CONSOLE=1 to catch startup too.",
                    )
                    .changed()
                {
                    crate::diag::set_console(console);
                    // Announce the change through the very channel it controls, so turning it on
                    // produces immediate evidence that it worked rather than silence until the next
                    // event happens to fire.
                    crate::diag::log_line(
                        "console",
                        if console { "console output ON (from Diagnostics)" } else { "console output OFF (from Diagnostics)" },
                    );
                }

                ui.separator();
                crate::theme::action_row(ui, |ui| {
                    if crate::theme::cancel_button(ui, "Close").clicked() {
                        close = true;
                    }
                });
            });

        self.diagnostics.open = open && !close;
        if let Some(t) = start {
            self.start_diagnostic(t);
        }
        if let Some(p) = open_artifact {
            // `file://` through the same opener the rest of the app uses for links.
            let url = format!("file:///{}", p.display().to_string().replace('\\', "/"));
            ctx.open_url(egui::OpenUrl::new_tab(url));
        }
        if attach {
            self.report.include_test = true;
            self.report.open = true;
            self.diagnostics.open = false;
        }
    }

    /// The test-result block for an issue report, when one has been run and the user kept it.
    pub(crate) fn test_result_block(&self) -> Option<String> {
        let v = self.diagnostics.last.as_ref()?;
        let status = if v.test.is_informational() {
            "result captured"
        } else if v.ok {
            "passed"
        } else {
            "reported problems"
        };
        let mut s = format!("{}: {status}\n{}\n", v.test.label(), v.headline);
        if let Some(p) = &v.artifact {
            s.push_str(&format!("Results: {}\n", p.display()));
        }
        Some(s)
    }
}
