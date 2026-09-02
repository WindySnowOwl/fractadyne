//! Help → Diagnostics…: run the user-safe validation tests from the UI.
//!
//! The audience is **issue reporters and people testing on hardware we don't own** — not the
//! developer. Three consequences shape this module:
//!
//! 1. **No CLI gate.** The people who benefit most are exactly the ones who will never pass a
//!    flag, so the dialog is always in the Help menu. That is also what makes cross-GPU
//!    validation crowdsourceable instead of limited to cards we can buy.
//! 2. **Only the two tests that mean something without context**: the self-test (does the maths
//!    hold on this GPU?) and the UI test (does it draw and lay out correctly?). The dev harnesses
//!    — `--livetest`, `--bench-matrix`, `--divetest`, `--juliadive` — stay CLI-only on purpose: a
//!    button for those produces confused bug reports, not information. `scripts/gpu-validate.*`
//!    is the power-user path and runs the full battery.
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
}

impl DiagTest {
    pub(crate) fn label(self) -> &'static str {
        match self {
            DiagTest::SelfTest => "Self-test",
            DiagTest::UiTest => "UI test",
        }
    }

    /// Button text. Spelled out rather than derived from `label()` — lowercasing that gave
    /// "Run ui test", which reads as a typo.
    pub(crate) fn button(self) -> &'static str {
        match self {
            DiagTest::SelfTest => "Run self-test",
            DiagTest::UiTest => "Run UI test",
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
        }
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

/// Is this a per-check/per-step progress line worth counting and showing?
pub(crate) fn is_progress_line(line: &str) -> bool {
    let l = line.trim_start();
    l.starts_with("[selftest") || l.starts_with("[uitest") || l.starts_with("=== step")
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
                // content, never by pipe (see module docs).
                if parse_selftest_verdict(&text).is_some() || parse_uitest_verdict(&text).is_some()
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
            let ok = match (
                parse_selftest_verdict(&headline),
                parse_uitest_verdict(&headline),
            ) {
                (Some((cp, ct, _, _)), _) => cp == ct,
                (_, Some((_, _, fail))) => fail == 0,
                _ => success,
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

                for test in [DiagTest::SelfTest, DiagTest::UiTest] {
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
                    let (colour, word) = if v.ok {
                        (egui::Color32::from_rgb(0x4c, 0xaf, 0x50), "passed")
                    } else {
                        (egui::Color32::from_rgb(0xe5, 0x73, 0x73), "reported problems")
                    };
                    ui.label(
                        egui::RichText::new(format!("{} {word}", v.test.label()))
                            .color(colour)
                            .strong(),
                    );
                    ui.label(egui::RichText::new(&v.headline).monospace().small());
                    if !v.ok {
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
            });

        self.diagnostics.open = open;
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
        let mut s = format!(
            "{}: {}\n{}\n",
            v.test.label(),
            if v.ok { "passed" } else { "reported problems" },
            v.headline
        );
        if let Some(p) = &v.artifact {
            s.push_str(&format!("Results: {}\n", p.display()));
        }
        Some(s)
    }
}
