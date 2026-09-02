//! Modal dialog surfaces — `impl FractadyneApp` blocks moved verbatim out of `main.rs`
//! (REFACTOR-PLAN Phase 3, intra-crate UI split). Each `draw_*_dialog(ctx)` is a no-op
//! unless its `*_open` flag is set; the event loop calls them as a flat sequence.

use crate::*;

/// Depth-based default dive duration (seconds) for "Script to current view": a base plus ~1.5 s per
/// order of magnitude, so a deep dive lasts long enough to stay watchable (fast dives flash
/// single-color frames at extreme zoom). Rounded to whole seconds; clamped to a sane range.
fn default_dive_secs(log10mag: f64) -> f64 {
    (8.0 + 1.5 * log10mag.max(0.0)).round().clamp(8.0, 3600.0)
}

/// Middle-elide a path so BOTH ends survive. Left-truncation hides the drive, right-truncation
/// hides the leaf, and it is the two ends together that let someone recognise where a file went.
/// Operates on chars, not bytes, so a non-ASCII user directory cannot split a codepoint.
fn elide_middle(s: &str, max: usize) -> String {
    let n = s.chars().count();
    if n <= max || max < 5 {
        return s.to_string();
    }
    let keep = max - 1; // room for the ellipsis
    let head = keep.div_ceil(2);
    let tail = keep - head;
    let h: String = s.chars().take(head).collect();
    let t: String = s.chars().skip(n - tail).collect();
    format!("{h}…{t}")
}

/// Reveal a directory in the platform file manager. Best-effort and deliberately silent on
/// failure: this is a convenience next to a path that is already displayed in full, so the user
/// can always copy it by hand if the launch does not work.
fn open_in_file_manager(dir: &std::path::Path) -> std::io::Result<()> {
    #[cfg(windows)]
    let mut cmd = {
        let mut c = std::process::Command::new("explorer");
        c.arg(dir);
        c
    };
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = std::process::Command::new("open");
        c.arg(dir);
        c
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut cmd = {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(dir);
        c
    };
    cmd.spawn().map(|_| ())
}

#[cfg(test)]
mod welcome_tests;

impl FractadyneApp {
    /// Post-crash prompt: offer to send a report when the PREVIOUS session ended without a clean
    /// shutdown, with a "don't ask again" opt-out.
    ///
    /// ⚠Asking, not sending. `diag` has already written the report to disk either way; this only
    /// offers to open the existing Report-an-issue dialog, which previews the full text before
    /// anything leaves the machine and carries the crash report as one selectable artifact. Nothing
    /// is transmitted from here.
    ///
    /// ⚠Suppressed for `launched_for_a_task` (see where `crash_prompt_open` is initialised): a modal
    /// in front of `--uitest` / `--livetest` would block them exactly the way the welcome dialog
    /// once did.
    pub(crate) fn draw_crash_prompt(&mut self, ctx: &egui::Context) {
        if !self.dialogs.crash_prompt_open {
            return;
        }
        let mut open = true;
        egui::Window::new("Fractadyne didn't shut down cleanly")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("The previous session ended unexpectedly. A report has been saved on this machine — would you like to send it?");
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("Nothing is sent until you review it: the next screen shows exactly what would be included, and system information is one checkbox to exclude.")
                        .weak()
                        .small(),
                );
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui
                        .button(egui::RichText::new("Report it…").strong())
                        .on_hover_text("Opens Report an issue, with the crash report attached.")
                        .clicked()
                    {
                        self.report.open = true;
                        self.dialogs.crash_prompt_open = false;
                    }
                    if ui.button("Not now").clicked() {
                        self.dialogs.crash_prompt_open = false;
                    }
                });
                ui.add_space(4.0);
                // Persisted, and phrased as the user's intent ("stop asking") rather than the
                // implementation ("suppress prompt").
                let mut dont_ask = self.crash_prompt_disabled;
                if ui
                    .checkbox(&mut dont_ask, "Don't ask again after a crash")
                    .on_hover_text("Reports are still saved locally, and Help ▸ Report an issue always works.")
                    .changed()
                {
                    self.crash_prompt_disabled = dont_ask;
                }
            });
        if !open {
            self.dialogs.crash_prompt_open = false;
        }
    }

    /// First-run welcome overlay: a short quick-start shown once on a fresh install (and
    /// re-openable from Help). Deep-zoom explorers are opaque to newcomers — this covers the
    /// first-two-minutes controls and offers a couple of one-click destinations, then gets out of
    /// the way. Deliberately NOT a tutorial or a "simple mode": nothing is hidden, and the full
    /// Help (F1) is one click away. Dismissing it persists `welcome_seen` so it never nags.
    pub(crate) fn draw_welcome_dialog(&mut self, ctx: &egui::Context) {
        if !self.dialogs.welcome_open {
            return;
        }
        let mut open = self.dialogs.welcome_open;
        let (mut dismiss, mut open_help) = (false, false);
        let mut goto: Option<(&str, &str, &str, f64)> = None;
        egui::Window::new("Welcome to Fractadyne")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .default_width(460.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(
                    "A deep-zoom explorer for the Mandelbrot set and its relatives. \
                     A few controls to get you moving:",
                );
                ui.add_space(6.0);
                let row = |ui: &mut egui::Ui, k: &str, d: &str| {
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [96.0, ui.spacing().interact_size.y],
                            egui::Label::new(egui::RichText::new(k).monospace().strong())
                                .halign(egui::Align::LEFT),
                        );
                        ui.label(d);
                    });
                };
                row(ui, "drag", "pan the view");
                row(ui, "scroll", "zoom in / out toward the cursor");
                row(ui, "hold Space", "smooth continuous zoom (Shift+Space out)");
                row(ui, &format!("{} then click", crate::icons::CLICK_ZOOM), "dive into the clicked point (toolbar)");
                row(ui, "M", "jump to the nearest minibrot at its own scale");
                ui.add_space(6.0);
                ui.label(egui::RichText::new("Jump to a landmark to start:").weak().small());
                ui.horizontal_wrapped(|ui| {
                    // A few shallow, striking destinations (see `FAMOUS`).
                    for entry in &[FAMOUS[0], FAMOUS[1], FAMOUS[6]] {
                        if ui.button(entry.0).clicked() {
                            goto = Some(*entry);
                        }
                    }
                });
                ui.add_space(8.0);
                ui.separator();
                // ── Setup strip ─────────────────────────────────────────────────────────────
                // Deliberately SHORT, and the rule that keeps it short: a first-run screen carries
                // only what is consequential, awkward to discover later, or a matter of consent.
                // Everything else belongs in File → Settings, which is one click away below — a
                // welcome screen that grows into a settings panel is a wall between the user and
                // the fractal, and the landmark buttons above are what actually get them moving.
                //
                // Theme and update track are NOT new here; they already live in File → Settings.
                // They are surfaced because first run is when people want them and least know
                // where to look. The other two rows are orientation, not settings.
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Theme").weak().small());
                    for m in [crate::theme::ThemeMode::Dark, crate::theme::ThemeMode::Light] {
                        if ui.selectable_label(self.theme == m, m.label()).clicked() {
                            self.theme = m;
                            crate::theme::apply_theme(ui.ctx(), m);
                        }
                    }
                    ui.add_space(12.0);
                    // The brand-mark opt-out belongs at first run, not in a menu: someone who is
                    // going to publish frames wants that choice BEFORE they render, not after
                    // finding it later. Applies to the live view and to exports.
                    ui.checkbox(&mut self.show_watermark, "Show \"Fd\" mark")
                        .on_hover_text("The small brand mark in the lower-right of the view. Off removes it from the live view and from exported images.");
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Updates").weak().small());
                    // ⚠The one CONSENT item on this screen: a network request the app makes on the
                    // user's behalf. That belongs at first run rather than buried in a menu.
                    ui.checkbox(&mut self.update_check_on_launch, "Check for updates")
                        .on_hover_text(
                            "Asks GitHub for the latest release when the app starts. \
                             Nothing is uploaded.",
                        );
                    if self.update_check_on_launch {
                        for t in crate::update::UpdateTrack::ALL {
                            ui.selectable_value(&mut self.update_track, t, t.label());
                        }
                    }
                });
                ui.add_space(4.0);
                // Where things go — answers "where did my screenshot end up?" before it is asked.
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Files").weak().small());
                    let dir = fractadyne_state::config_dir();
                    let shown = dir
                        .as_ref()
                        .map(|d| d.display().to_string())
                        .unwrap_or_else(|| "(no config directory)".to_string());
                    ui.label(
                        egui::RichText::new(elide_middle(&shown, 46)).monospace().small(),
                    )
                    .on_hover_text(&shown);
                    if let Some(d) = dir {
                        if ui.small_button("Open").clicked() {
                            let _ = open_in_file_manager(&d);
                        }
                    }
                });
                // The adapter, as text. Not a setting — but nearly every hard problem this project
                // has hit is hardware-specific, and a user who can read their GPU straight off this
                // screen writes a far better bug report. It also confirms which card was picked,
                // which is not obvious on a dual-GPU laptop.
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Rendering on").weak().small());
                    ui.label(egui::RichText::new(&self.gpu_name).small());
                });
                ui.add_space(8.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Open full help  (F1)").clicked() {
                        open_help = true;
                    }
                    // A pointer, not a button: Settings is a submenu, so a button here could not
                    // actually open it, and a control that cannot deliver is worse than none.
                    ui.label(
                        egui::RichText::new(format!("More in File → {} Settings", crate::icons::SETTINGS))
                            .weak()
                            .small(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(egui::Button::new(
                                egui::RichText::new("Get started").color(egui::Color32::WHITE),
                            ).fill(egui::Color32::from_rgb(0x3A, 0x7A, 0xB0)))
                            .clicked()
                        {
                            dismiss = true;
                        }
                    });
                });
            });
        // A landmark button both jumps AND dismisses — the user is off exploring.
        if let Some((name, cx, cy, mag)) = goto {
            self.goto_location(cx, cy, mag, name, ctx);
            dismiss = true;
        }
        if open_help {
            self.dialogs.help_open = true;
            dismiss = true;
        }
        // The window's own [x], "Get started", a landmark, or opening Help all dismiss it. Persist
        // happens via the session save (welcome_seen = !welcome_open).
        self.dialogs.welcome_open = open && !dismiss;
    }

    /// "Go to location" dialog — jump to a pasted center/zoom, or copy the current one to share.
    pub(crate) fn draw_goto_dialog(&mut self, ctx: &egui::Context) {
        if !self.goto.open {
            return;
        }
        let mut open = self.goto.open;
        let mut go = false;
        let mut copy = false;
        let mut poi: Option<usize> = None;
        let mut find_feat = false;
        let mut cancel_feat = false;
        egui::Window::new("Go to location")
            .open(&mut open)
            .resizable(false)
            .default_width(420.0)
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("Center — real (Re)").weak().small());
                let rx = ui.add(egui::TextEdit::singleline(&mut self.goto.x).desired_width(f32::INFINITY));
                ui.label(egui::RichText::new("Center — imaginary (Im)").weak().small());
                let ry = ui.add(egui::TextEdit::singleline(&mut self.goto.y).desired_width(f32::INFINITY));
                ui.label(egui::RichText::new("Zoom (magnification)").weak().small());
                let rz = ui.add(egui::TextEdit::singleline(&mut self.goto.zoom).desired_width(220.0));
                // Live warning for the flat-frame trap BEFORE the jump is made: the solver
                // reaches depths the renderer's fixed iteration count cannot resolve, and a
                // correct coordinate under a solid-colour frame reads as a bug. Auto-iteration
                // never warns — its budget follows the jump like a hand zoom.
                if let Some((have, typical)) = crate::parse_zoom_to_log2(&self.goto.zoom)
                    .and_then(|t| {
                        crate::deep_jump_iter_shortfall(
                            t,
                            self.render_cfg.max_iter,
                            self.render_cfg.auto_iter,
                        )
                    })
                {
                    ui.label(
                        egui::RichText::new(format!(
                            "⚠ Iterations is fixed at {have}; this depth typically needs                              ~{typical}. The view will render flat until it is raised — or                              enable auto-iterations (Rendering panel), which adapts by itself."
                        ))
                        .small()
                        .color(ui.visuals().warn_fg_color),
                    );
                }
                // Same rule as the k/p boxes below: a message describes the inputs that produced
                // it, so editing any of them retires it rather than leaving a stale verdict.
                if rx.changed() || ry.changed() || rz.changed() {
                    self.goto.msg = None;
                }
                if let Some(m) = &self.goto.msg {
                    ui.colored_label(egui::Color32::from_rgb(0xE0, 0x6C, 0x60), m);
                }
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    go = ui.button("Go").clicked();
                    if ui.button("Copy").on_hover_text("Copy this location to the clipboard").clicked() {
                        copy = true;
                    }
                    if ui.button("Use current").clicked() {
                        self.goto.x = fractadyne_core::to_decimal_string(&self.viewport.center_x);
                        self.goto.y = fractadyne_core::to_decimal_string(&self.viewport.center_y);
                        self.goto.zoom = fmt_zoom_field(self.viewport.log2_magnification());
                        self.goto.msg = None;
                    }
                });
                ui.label(
                    egui::RichText::new(
                        "Paste a center/zoom from someone else, or Copy to share. Coordinates \
                         accept exact fractions (-3/4) and the real field accepts a whole \
                         complex value ((37+16i)/100), which fills both.",
                    )
                    .weak()
                    .small(),
                );

                ui.separator();
                egui::CollapsingHeader::new("Go to feature (Mandelbrot)")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Well-known points:").weak().small());
                        egui::ComboBox::from_id_salt("goto_poi")
                            .selected_text("Jump to a point of interest…")
                            .width(260.0)
                            .show_ui(ui, |ui| {
                                for (i, (name, ..)) in crate::MISIUREWICZ_POI.iter().enumerate() {
                                    if ui.selectable_label(false, *name).clicked() {
                                        poi = Some(i);
                                    }
                                }
                            });
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new("Or solve for a feature near the current view:")
                                .weak()
                                .small(),
                        );
                        ui.horizontal(|ui| {
                            ui.selectable_value(
                                &mut self.goto.feat_kind,
                                crate::FeatureKind::Misiurewicz,
                                "Misiurewicz",
                            );
                            ui.selectable_value(
                                &mut self.goto.feat_kind,
                                crate::FeatureKind::Minibrot,
                                "Nearest minibrot",
                            );
                        });
                        if self.goto.feat_kind == crate::FeatureKind::Misiurewicz {
                            ui.horizontal(|ui| {
                                ui.label("Preperiod k");
                                let rk = ui.add(
                                    egui::TextEdit::singleline(&mut self.goto.feat_k)
                                        .hint_text("auto")
                                        .desired_width(48.0),
                                );
                                ui.add_space(8.0);
                                ui.label("Period p");
                                let rp = ui.add(
                                    egui::TextEdit::singleline(&mut self.goto.feat_p)
                                        .hint_text("auto")
                                        .desired_width(48.0),
                                );
                                // ⚠A message names the (k,p) it was about. Editing either field
                                // makes it describe inputs that are no longer there — and because
                                // a failed solve LEAVES its numbers in these boxes, clearing them
                                // back to "auto" left a red line still quoting the pair you had
                                // just deleted, reading as though blank fields had produced it.
                                if rk.changed() || rp.changed() {
                                    self.goto.msg = None;
                                }
                                // ⭐A way BACK to auto. A successful detect writes its numbers
                                // into these boxes so you can see what was found — which quietly
                                // makes "auto" a one-shot: every later press re-uses the pair
                                // from last time, and the only way back was to clear two boxes by
                                // hand and know that blank meant auto. Shown only when there is
                                // something to clear, so it is never a button that does nothing.
                                if !self.goto.feat_k.is_empty() || !self.goto.feat_p.is_empty() {
                                    ui.add_space(6.0);
                                    if ui
                                        .small_button("Auto")
                                        .on_hover_text(
                                            "Clear both boxes so the next search detects the                                              preperiod and period from the orbit at the view                                              centre.",
                                        )
                                        .clicked()
                                    {
                                        self.goto.feat_k.clear();
                                        self.goto.feat_p.clear();
                                        self.goto.msg = None;
                                    }
                                }
                            })
                            .response
                            .on_hover_text(
                                "Leave both blank to detect them from the orbit at the view \
                                 centre; the values found are filled in here. Nobody looking at a \
                                 spiral knows its preperiod, and having to supply one is what made \
                                 this solver unusable in practice.",
                            );
                        }
                        // While a solve runs: a spinner, its elapsed time, and no second button
                        // press. The work is off-thread now (see `goto_feature`), so the window
                        // keeps painting — which is the only reason a spinner here means anything.
                        if let Some(elapsed) = self.feature_solve.as_ref().map(|f| f.started.elapsed()) {
                            ui.horizontal(|ui| {
                                ui.add(egui::Spinner::new().size(16.0));
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Solving… {:.1}s  (arbitrary precision; deep views take \
                                         longer)",
                                        elapsed.as_secs_f64()
                                    ))
                                    .weak(),
                                );
                                // ⭐A VISIBLE way out. Closing the dialog already abandons the
                                // search (see `poll_feature_solve`), but nothing said so, and now
                                // that the solve reaches the depth asked for, the wait is real:
                                // measured on a 902-step orbit, 2,000 octaves is 0.07 s, 50,000 is
                                // 16 s, and the 192,672 behind a 1e58000× request is 159 s — and a
                                // longer (k, p) scales that again. Abandoning is enough: the
                                // worker owns no shared state, so dropping its channel lets it
                                // finish into nothing.
                                if ui
                                    .button("Cancel")
                                    .on_hover_text(
                                        "Stop waiting. The solve itself runs to completion in the \
                                         background and its answer is discarded — nothing is left \
                                         in a half-finished state.",
                                    )
                                    .clicked()
                                {
                                    cancel_feat = true;
                                }
                            });
                        } else {
                            find_feat = ui
                                .button("Find near view")
                                .on_hover_text(
                                    "Newton-solve from the current center onto the exact feature \
                                     (arbitrary precision). Navigate near the feature first.\n\n\
                                     Set the Zoom field above to the depth you WANT before solving: \
                                     the point is computed to the precision that depth needs, and \
                                     the view jumps straight there. That is one solve instead of a \
                                     long sequence of manual zoom steps.",
                                )
                                .clicked();
                        }
                    });
            });
        if let Some(i) = poi {
            let (name, cx, cy, mag) = crate::MISIUREWICZ_POI[i];
            self.goto_location(cx, cy, mag, name, ctx);
            self.goto.open = false;
        }
        if find_feat {
            self.goto_feature(ctx);
        }
        if cancel_feat {
            // Drop the receiver: the worker's `send` then fails and its result is discarded.
            self.feature_solve = None;
            self.goto.msg = Some("Solve cancelled.".into());
        }
        if copy {
            ctx.copy_text(format!(
                "center_re={}\ncenter_im={}\nzoom={}",
                self.goto.x, self.goto.y, self.goto.zoom
            ));
        }
        if go {
            self.apply_goto(); // clears goto_open on success
        }
        // Closed if the user hit the window's ✕ (open=false) or Go succeeded.
        self.goto.open = open && self.goto.open;
    }

    /// "Share location" (.fdn) dialog — copy/paste/apply/save/load a self-contained location.
    pub(crate) fn draw_share_dialog(&mut self, ctx: &egui::Context) {
        if !self.share.open {
            return;
        }
        let mut open = self.share.open;
        let (mut copy, mut apply, mut save, mut load) = (false, false, false, false);
        egui::Window::new("Share location")
            .open(&mut open)
            .resizable(true)
            .default_width(460.0)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(
                        "A self-contained location (fractal, full-precision center, zoom, \
                         coloring). Copy it to share, or paste/load one and Apply.",
                    )
                    .weak()
                    .small(),
                );
                ui.add_space(4.0);
                egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.share.text)
                            .desired_width(f32::INFINITY)
                            .desired_rows(10)
                            .code_editor(),
                    );
                });
                if let Some(m) = &self.share.msg {
                    ui.colored_label(egui::Color32::from_rgb(0xE0, 0xA0, 0x30), m);
                }
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    apply = ui.button("Apply").on_hover_text("Jump to the location in the box").clicked();
                    copy = ui.button("Copy").on_hover_text("Copy the text to the clipboard").clicked();
                    if ui.button("Use current").clicked() {
                        self.share.text = self.view_metadata();
                        self.share.msg = None;
                    }
                    save = ui.button("Save .fdn…").clicked();
                    load = ui.button("Load .fdn…").clicked();
                });
            });
        if copy {
            ctx.copy_text(self.share.text.clone());
            self.share.msg = Some("Copied to clipboard.".into());
        }
        if save {
            self.save_share_file();
        }
        if load {
            self.load_share_file();
        }
        if apply {
            self.apply_share_text(ctx); // clears share_open on success
        }
        self.share.open = open && self.share.open;
    }

    /// "Report an issue" dialog (Help → Report an issue…): a description + selectable artifacts
    /// (system info, current location, recent log, latest crash report), a full preview of exactly
    /// what will be sent, then GitHub issue (primary, [`crate::ISSUES_URL`]) / Copy / Save /
    /// Email to [`crate::REPORT_EMAIL`]. Nothing is
    /// transmitted until the user acts, and system info is one checkbox to exclude.
    pub(crate) fn draw_report_dialog(&mut self, ctx: &egui::Context) {
        if !self.report.open {
            return;
        }
        let mut open = self.report.open;
        let has_crash = crate::diag::latest_crash().is_some();
        let (mut copy, mut save, mut email, mut gmail, mut github) =
            (false, false, false, false, false);
        egui::Window::new("Report an issue")
            .open(&mut open)
            .resizable(true)
            .default_width(560.0)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(
                        "The best place to report is a GitHub issue — it's public and searchable, \
                         and you'll see when it's fixed. Email works too. Review everything below \
                         first: nothing leaves your machine until you act.",
                    )
                    .weak()
                    .small(),
                );
                ui.add_space(6.0);
                egui::Grid::new("report_classify")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Type:");
                        egui::ComboBox::from_id_salt("report_kind")
                            .selected_text(self.report.kind.label())
                            .show_ui(ui, |ui| {
                                for k in crate::IssueKind::ALL {
                                    ui.selectable_value(&mut self.report.kind, k, k.label());
                                }
                            });
                        ui.end_row();
                        ui.label("Severity:");
                        ui.horizontal(|ui| {
                            egui::ComboBox::from_id_salt("report_sev")
                                .selected_text(self.report.severity.label())
                                .show_ui(ui, |ui| {
                                    for s in crate::Severity::ALL {
                                        ui.selectable_value(&mut self.report.severity, s, s.label());
                                    }
                                });
                            ui.add_space(16.0);
                            ui.label("Reproducible:");
                            egui::ComboBox::from_id_salt("report_repro")
                                .selected_text(self.report.repro.label())
                                .show_ui(ui, |ui| {
                                    for r in crate::Repro::ALL {
                                        ui.selectable_value(&mut self.report.repro, r, r.label());
                                    }
                                });
                        });
                        ui.end_row();
                    });
                ui.add_space(6.0);
                ui.label("Describe what happened:");
                ui.add(
                    egui::TextEdit::multiline(&mut self.report.description)
                        .desired_width(f32::INFINITY)
                        .desired_rows(5)
                        .hint_text(
                            "What happened, the steps to reproduce it, and what you expected instead.",
                        ),
                );
                ui.add_space(6.0);
                ui.label(egui::RichText::new("Include:").weak().small());
                ui.checkbox(&mut self.report.include_sysinfo, "System info (version, OS, CPU, GPU, VRAM)");
                ui.checkbox(&mut self.report.include_location, "Current location (.fdn)");
                ui.checkbox(&mut self.report.include_log, "Recent log");
                ui.add_enabled_ui(has_crash, |ui| {
                    ui.checkbox(
                        &mut self.report.include_crash,
                        if has_crash { "Latest crash report" } else { "Latest crash report (none found)" },
                    );
                });
                // Only offered once a test has actually been run — an issue that claims a test
                // result it does not have is worse than one that claims nothing. Help →
                // Diagnostics… produces it, and its "Attach to an issue report…" ticks this.
                let has_test = self.diagnostics.last.is_some();
                ui.add_enabled_ui(has_test, |ui| {
                    ui.checkbox(
                        &mut self.report.include_test,
                        if has_test {
                            "Diagnostics test result"
                        } else {
                            "Diagnostics test result (run one from Help → Diagnostics…)"
                        },
                    )
                    .on_hover_text(
                        "A machine-validated pass/fail from your own hardware — far more useful \
                         to a maintainer than a description alone",
                    );
                });

                let report = self.build_report();
                ui.add_space(4.0);
                egui::CollapsingHeader::new(format!("Preview what will be sent  ({} KB)", report.len() / 1024))
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(260.0)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut report.as_str())
                                        .desired_width(f32::INFINITY)
                                        .font(egui::TextStyle::Monospace),
                                );
                            });
                    });

                if let Some(m) = &self.report.msg {
                    ui.colored_label(egui::Color32::from_rgb(0xE0, 0xA0, 0x30), m);
                }
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    github = ui
                        .button(egui::RichText::new("Open a GitHub issue").strong())
                        .on_hover_text(
                            "Open a new-issue page with the title filled in; the report is copied \
                             to your clipboard — paste it into the issue body (Ctrl+V).",
                        )
                        .clicked();
                    copy = ui
                        .button("Copy report")
                        .on_hover_text("Copy everything to the clipboard")
                        .clicked();
                    save = ui.button("Save report…").on_hover_text("Save as a .txt to attach").clicked();
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Or by email:").weak().small());
                    gmail = ui
                        .button("Compose in Gmail")
                        .on_hover_text(format!(
                            "Open a Gmail compose window to {} with the subject filled; paste the copied report",
                            crate::REPORT_EMAIL
                        ))
                        .clicked();
                    email = ui
                        .button("Email app…")
                        .on_hover_text("Open your default mail app (mailto) — needs an email handler configured")
                        .clicked();
                });
                ui.label(
                    egui::RichText::new(
                        "Tip: the report is copied to your clipboard on any of these — paste it into \
                         the issue or message body (Ctrl+V). Please also attach a screenshot or \
                         sample image if you can (drag it in); it helps a lot for rendering and UI \
                         problems.",
                    )
                    .weak()
                    .small(),
                );
            });
        if github {
            ctx.copy_text(self.build_report());
            // The report itself can't ride the URL (length limits) — it's on the clipboard; the
            // new-issue page opens with the title prefilled and a paste hint as the body.
            ctx.open_url(egui::OpenUrl::new_tab(crate::issue_new_url(&self.report_subject())));
            self.report.msg =
                Some("Opened GitHub — the report is on the clipboard; paste it into the issue.".into());
        }
        if copy {
            ctx.copy_text(self.build_report());
            self.report.msg = Some("Report copied to the clipboard.".into());
        }
        if save {
            if let Some(path) = rfd::FileDialog::new()
                .set_directory(self.dialog_dir_default())
                .set_file_name("fractadyne-report.txt")
                .add_filter("Text", &["txt"])
                .save_file()
            {
                self.remember_dir(&path);
                match std::fs::write(&path, self.build_report().as_bytes()) {
                    Ok(()) => self.report.msg = Some(format!("Saved to {}", path.display())),
                    Err(e) => self.report.msg = Some(format!("Save failed: {e}")),
                }
            }
        }
        if email {
            ctx.copy_text(self.build_report());
            ctx.open_url(egui::OpenUrl::same_tab(crate::issue_mailto_url(&self.report_subject())));
            self.report.msg =
                Some("Opened your email app — the report is on the clipboard; paste it in.".into());
        }
        if gmail {
            ctx.copy_text(self.build_report());
            // Gmail web compose — reliable for webmail users without an OS mailto handler. The body
            // can't hold the report (URL length), so it's on the clipboard to paste.
            let url = format!(
                "https://mail.google.com/mail/?view=cm&fs=1&to={}&su={}",
                crate::mailto_encode(crate::REPORT_EMAIL),
                crate::mailto_encode(&self.report_subject()),
            );
            ctx.open_url(egui::OpenUrl::new_tab(url));
            self.report.msg =
                Some("Opened Gmail compose — paste the copied report into the body (Ctrl+V).".into());
        }
        self.report.open = open && self.report.open;
    }

    /// "Reset application state" confirmation dialog — permanently deletes all saved data.
    pub(crate) fn draw_reset_dialog(&mut self, ctx: &egui::Context) {
        if !self.dialogs.reset_confirm_open {
            return;
        }
        let mut open = self.dialogs.reset_confirm_open;
        let (mut confirm, mut cancel) = (false, false);
        egui::Window::new("Reset application state")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .default_width(460.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.colored_label(
                    egui::Color32::from_rgb(0xE0, 0x6C, 0x60),
                    "⚠  This permanently deletes all saved Fractadyne data:",
                );
                ui.add_space(2.0);
                ui.label("• the saved session (current view, coloring, preferences)");
                ui.label("• all bookmarks and their thumbnails");
                ui.add_space(6.0);
                ui.label(egui::RichText::new("Location").weak().small());
                ui.label(
                    egui::RichText::new(fractadyne_state::state_location_display())
                        .monospace()
                        .small(),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(
                        "This can't be undone. The current session won't be re-saved on exit, \
                         so defaults load on the next launch.",
                    )
                    .weak()
                    .small(),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    cancel = ui.button("Cancel").clicked();
                    if ui
                        .add(egui::Button::new(
                            egui::RichText::new("Reset everything").color(egui::Color32::WHITE),
                        ).fill(egui::Color32::from_rgb(0xB0, 0x3A, 0x30)))
                        .clicked()
                    {
                        confirm = true;
                    }
                });
            });
        if confirm {
            match fractadyne_state::reset_all() {
                Ok(_) => {
                    // Don't recreate what we just deleted; reflect the cleared bookmarks in the UI.
                    self.suppress_autosave = true;
                    self.bookmarks.clear();
                    self.set_toast(
                        "Application state reset — defaults will load on the next launch.",
                        ctx,
                    );
                }
                Err(e) => self.set_toast(format!("Reset failed: {e}"), ctx),
            }
            self.dialogs.reset_confirm_open = false;
        } else if cancel {
            self.dialogs.reset_confirm_open = false;
        } else {
            self.dialogs.reset_confirm_open = open && self.dialogs.reset_confirm_open;
        }
    }

    /// Open the "Script to current view" dialog, seeding the dive duration from the current zoom
    /// depth (deeper ⇒ longer, so the dive isn't a blur) and clearing the notation.
    pub(crate) fn open_script_export(&mut self) {
        let log10mag = self.viewport.log2_magnification() / std::f64::consts::LOG2_10;
        self.dialogs.script_export_secs = default_dive_secs(log10mag);
        self.dialogs.script_export_note.clear();
        self.dialogs.script_export_open = true;
    }

    /// "Script to current view" dialog — build a tour that zooms from the full view down to the
    /// current view, with an optional caption and a chosen duration, and save it as a `.toml`.
    pub(crate) fn draw_script_export_dialog(&mut self, ctx: &egui::Context) {
        if !self.dialogs.script_export_open {
            return;
        }
        let log2mag = self.viewport.log2_magnification();
        let log10mag = log2mag / std::f64::consts::LOG2_10;
        let mut open = self.dialogs.script_export_open;
        let mut save = false;
        egui::Window::new("Tour from current view")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .default_width(440.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(
                        "Creates a tour that zooms from the full view down to where you are now.",
                    )
                    .weak(),
                );
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Destination").weak().small());
                    ui.label(
                        egui::RichText::new(format!(
                            "{} · {}×",
                            self.fractal.name(),
                            crate::fmt_zoom_field(log2mag)
                        ))
                        .monospace()
                        .small(),
                    );
                });
                ui.add_space(8.0);

                ui.label("Notation (optional caption shown during the dive)");
                ui.add(
                    egui::TextEdit::multiline(&mut self.dialogs.script_export_note)
                        .desired_rows(2)
                        .desired_width(f32::INFINITY)
                        .hint_text("e.g. Diving to the Misiurewicz three-spar…"),
                );
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Zoom duration");
                    ui.add(
                        egui::DragValue::new(&mut self.dialogs.script_export_secs)
                            .speed(0.5)
                            .range(1.0..=7200.0)
                            .suffix(" s"),
                    );
                    let rate = if self.dialogs.script_export_secs > 0.0 {
                        log10mag / self.dialogs.script_export_secs
                    } else {
                        0.0
                    };
                    ui.label(
                        egui::RichText::new(format!("≈ {rate:.1} decades/s"))
                            .weak()
                            .small(),
                    );
                    if ui
                        .small_button("Reset")
                        .on_hover_text("Depth-based default")
                        .clicked()
                    {
                        self.dialogs.script_export_secs = default_dive_secs(log10mag);
                    }
                });
                ui.label(
                    egui::RichText::new(
                        "Default scales with depth so the dive stays watchable. Faster dives can \
                         flash single-color frames at extreme zoom.",
                    )
                    .weak()
                    .small(),
                );
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Save tour…").clicked() {
                        save = true;
                    }
                    if ui.button("Cancel").clicked() {
                        self.dialogs.script_export_open = false;
                    }
                });
            });
        self.dialogs.script_export_open = open && self.dialogs.script_export_open;
        if save {
            self.save_dive_script();
        }
    }

    /// Build the tour TOML and write it via a save-file dialog.
    fn save_dive_script(&mut self) {
        let secs = self.dialogs.script_export_secs.clamp(1.0, 7200.0);
        let note = self.dialogs.script_export_note.trim().to_string();
        let toml = self.build_dive_script(&note, secs);
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Fractadyne script (TOML)", &["toml"])
            .set_directory(self.dialog_dir_default())
            .set_file_name("dive-to-view.toml")
            .save_file()
        {
            self.remember_dir(&path);
            let ctx_msg = match std::fs::write(&path, toml.as_bytes()) {
                Ok(()) => format!(
                    "Saved tour → {}. Play it via Tools → Play tour, or render with \
                     --render-tour.",
                    path.file_name().and_then(|n| n.to_str()).unwrap_or("script.toml")
                ),
                Err(e) => format!("Save failed: {e}"),
            };
            self.pending_toast = Some(ctx_msg);
            self.dialogs.script_export_open = false;
        }
    }

    /// Compose the tour script that dives from the full view to the current center/zoom.
    ///
    /// A deep dive must keep the center **fixed** on the target while zooming — panning the center
    /// during a deep zoom sends the intermediate frames to points that aren't on the target's path,
    /// which at extreme magnification are off the set entirely (a black / uniform screen). So the
    /// dive is three phases: show the whole set, **pan onto the target at low zoom** (mag 8), then
    /// zoom straight in with the center OMITTED so it's inherited (held fixed on the point). Shallow
    /// targets (≤ 8×) just zoom directly — panning-while-zooming is harmless that shallow.
    pub(crate) fn build_dive_script(&self, note: &str, secs: f64) -> String {
        let log2mag = self.viewport.log2_magnification();
        let log10mag = log2mag / std::f64::consts::LOG2_10;
        let cx = fractadyne_core::to_decimal_string(&self.viewport.center_x);
        let cy = fractadyne_core::to_decimal_string(&self.viewport.center_y);
        let (hx, hy) = self.fractal.default_center();
        let target_mag = self.viewport.magnification(); // saturates to ∞ past ~1e308×
        // Target zoom, always emitted as a QUOTED scientific string. A bare `{target_mag}` is a
        // trap: Rust's f64 Display never uses exponent form, so 1e87 prints as an 88-digit integer
        // literal — which TOML parses as an i64 and rejects ("too large to fit in the target type")
        // for any dive past ~1e19. `{:e}` gives the shortest round-tripping form; past f64's ~1e308
        // ceiling `target_mag` saturates to ∞ (Display "inf"), so fall back to the log10 magnitude.
        let target_zoom = if log10mag < 300.0 {
            format!("\"{target_mag:e}\"")
        } else {
            format!("\"1e{log10mag:.6}\"")
        };
        // The budget the destination actually wants, so the deep end isn't rendered flat while
        // the shallow frames still cost almost nothing (they interpolate up from 2000).
        let target_iter = self
            .viewport
            .recommended_max_iter(self.render_cfg.max_iter)
            .max(2_000);
        let deep = target_mag > 8.0;
        const SWOOP_SECS: f64 = 4.0;

        let mut s = String::new();
        s.push_str("# Fractadyne tour — generated by \"Tour from current view\".\n");
        s.push_str("# Zooms from the full view to a saved location. Play: Tools → Play tour;\n");
        s.push_str("# render: fractadyne --render-tour <this file> --out frames --mp4.\n");
        s.push_str("format_version = 2\n");
        s.push_str("name = \"Dive to view\"\n\n");
        s.push_str("[render]\n");
        s.push_str("size = \"1920x1080\"\n");
        s.push_str("fps = 30\n\n");
        // The destination, named once — keyframes reference it (and so can anything added later).
        s.push_str("[[location]]\n");
        s.push_str("id = \"target\"\n");
        s.push_str(&format!("re = \"{cx}\"\n"));
        s.push_str(&format!("im = \"{cy}\"\n\n"));

        // Phase 1 — the whole set (classic framing). `t` is absolute seconds throughout.
        s.push_str("[[keyframe]]            # full view\n");
        s.push_str("id = \"home\"\n");
        s.push_str("t = 0.0\n");
        s.push_str(&format!("fractal = \"{}\"\n", self.fractal.name()));
        if self.julia_mode {
            s.push_str("julia = true\n");
        }
        s.push_str(&format!("re = \"{hx}\"\n"));
        s.push_str(&format!("im = \"{hy}\"\n"));
        s.push_str("zoom = 1.0\n");
        s.push_str("max_iter = 2000\n");
        s.push_str("hold = 1.5\n\n");

        let total = if deep {
            // Phase 2 — pan onto the target while still zoomed out (so the pan never happens deep).
            s.push_str("[[keyframe]]            # recenter onto the target (still zoomed out)\n");
            s.push_str("id = \"recenter\"\n");
            s.push_str(&format!("t = {}\n", 1.5 + SWOOP_SECS));
            s.push_str("location = \"target\"\n");
            s.push_str("zoom = 8.0\n");
            s.push_str("max_iter = 3000\n");
            s.push_str("ease = \"smooth\"\n");
            s.push_str("hold = 0.5\n\n");
            // Phase 3 — the dive. Center inherited ⇒ held fixed on the point.
            s.push_str("[[keyframe]]            # dive in (center fixed — inherited from above)\n");
            s.push_str("id = \"dive\"\n");
            s.push_str(&format!("t = {}\n", 1.5 + SWOOP_SECS + 0.5 + secs));
            s.push_str(&format!("zoom = {target_zoom}\n"));
            s.push_str(&format!("max_iter = {target_iter}\n"));
            s.push_str("ease = \"out\"        # decelerate onto the point\n");
            s.push_str("hold = 2.0\n");
            1.5 + SWOOP_SECS + 0.5 + secs + 2.0
        } else {
            // Shallow: a single zoom straight to the target.
            s.push_str("[[keyframe]]            # zoom to the current view\n");
            s.push_str("id = \"dive\"\n");
            s.push_str(&format!("t = {}\n", 1.5 + secs));
            s.push_str("location = \"target\"\n");
            s.push_str(&format!("zoom = {target_zoom}\n"));
            s.push_str(&format!("max_iter = {target_iter}\n"));
            s.push_str("ease = \"out\"\n");
            s.push_str("hold = 2.0\n");
            1.5 + secs + 2.0
        };

        if !note.is_empty() {
            s.push('\n');
            s.push_str("[[annotation]]\n");
            s.push_str("kind = \"caption\"\n");
            // Escape backslashes and quotes so the TOML basic string round-trips.
            let esc = note.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
            s.push_str(&format!("text = \"{esc}\"\n"));
            s.push_str("t = 0.0\n");
            s.push_str(&format!("secs = {total:.1}\n"));
        }
        s
    }

    /// Transient status toast (fades out over ~4.5s), e.g. the minibrot-finder result.
    pub(crate) fn draw_toast(&mut self, ctx: &egui::Context) {
        let Some((msg, t0)) = self.toast.clone() else {
            return;
        };
        let age = ctx.input(|i| i.time) - t0;
        if age < 4.5 {
            let fade = ((4.5 - age) / 0.6).clamp(0.0, 1.0) as f32;
            egui::Area::new(egui::Id::new("fractadyne.toast"))
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 12.0))
                .interactable(false)
                .show(ctx, |ui| {
                    let fill = ui.visuals().extreme_bg_color.gamma_multiply(fade);
                    let accent = ui.visuals().hyperlink_color.gamma_multiply(fade);
                    let ink = ui.visuals().strong_text_color().gamma_multiply(fade);
                    egui::Frame::popup(ui.style())
                        .fill(fill)
                        .stroke(egui::Stroke::new(1.0_f32, accent))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(msg).color(ink));
                        });
                });
            ctx.request_repaint(); // keep fading
        } else {
            self.toast = None;
        }
    }

    /// Bookmarks manager — add the current view, jump to / delete saved locations (with thumbnails).
    pub(crate) fn draw_bookmarks_dialog(&mut self, ctx: &egui::Context) {
        if !self.dialogs.bookmarks_open {
            return;
        }
        let mut open = self.dialogs.bookmarks_open;
        let mut jump: Option<usize> = None;
        let mut delete: Option<usize> = None;
        let mut changed = false;
        // Pre-load thumbnail textures (mutable) before the immutable draw loop below.
        let thumb_ids: Vec<String> = self.bookmarks.iter().map(|b| b.thumb.clone()).collect();
        for id in &thumb_ids {
            let _ = self.bookmark_thumb_texture(ctx, id);
        }
        egui::Window::new("Bookmarks")
            .open(&mut open)
            .default_size([440.0, 480.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.bookmark_name)
                            .hint_text("name (optional)")
                            .desired_width(240.0),
                    );
                    if ui.button(format!("{} Add current view", crate::icons::BOOKMARK)).clicked() {
                        let name = self.bookmark_name.clone();
                        self.add_bookmark(&name);
                        self.bookmark_name.clear();
                    }
                });
                ui.separator();
                if self.bookmarks.is_empty() {
                    ui.label("No bookmarks yet. Add one from the current view.");
                }
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, b) in self.bookmarks.iter().enumerate() {
                        ui.horizontal(|ui| {
                            // Thumbnail preview (fixed 80px wide, aspect-preserving).
                            if let Some(tex) = self.thumb_cache.get(&b.thumb) {
                                let sz = tex.size_vec2();
                                let w = 80.0_f32;
                                ui.add(egui::Image::new(egui::load::SizedTexture::new(
                                    tex.id(),
                                    egui::vec2(w, w * sz.y / sz.x.max(1.0)),
                                )));
                            } else {
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(80.0, 56.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    rect,
                                    egui::CornerRadius::same(2),
                                    egui::Color32::from_black_alpha(80),
                                );
                            }
                            ui.vertical(|ui| {
                                ui.label(&b.name);
                                let zoom = meta_get(&b.meta, "zoom");
                                if !zoom.is_empty() {
                                    ui.label(egui::RichText::new(format!("{zoom}×")).weak().small());
                                }
                                ui.horizontal(|ui| {
                                    if ui.button("Go").clicked() {
                                        jump = Some(i);
                                    }
                                    if ui.button(crate::icons::DELETE).on_hover_text("Delete").clicked() {
                                        delete = Some(i);
                                    }
                                });
                            });
                        });
                        ui.separator();
                    }
                });
            });
        if let Some(i) = jump {
            self.bookmark_jump(i); // restores the view + arms heal-on-jump for a missing thumb
        }
        if let Some(i) = delete {
            // Remove the bookmark, then the thumbnail file + cached texture it named.
            if let Some(id) = crate::take_bookmark(&mut self.bookmarks, i) {
                if let Some(p) = Self::bookmark_thumb_path(&id) {
                    let _ = std::fs::remove_file(p);
                }
                self.thumb_cache.remove(&id);
            }
            changed = true;
        }
        if changed {
            self.save_bookmarks();
        }
        self.dialogs.bookmarks_open = open;
    }

    /// Benchmark configuration dialog — pick current-settings vs standardized (resolution/depth/
    /// burn-in) and start the run.
    pub(crate) fn draw_bench_config_dialog(&mut self, ctx: &egui::Context) {
        if !self.dialogs.bench_dialog_open {
            return;
        }
        let mut open = self.dialogs.bench_dialog_open;
        let mut run_now = false;
        egui::Window::new("Benchmark")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Mode");
                ui.radio_value(&mut self.bench.cfg.standard, false, "Current settings")
                    .on_hover_text("Play the fixed deep-zoom tour into the live view using your current resolution and settings.");
                ui.radio_value(&mut self.bench.cfg.standard, true, "Standardized")
                    .on_hover_text("Pin resolution + all render settings so the score is comparable across machines.");
                ui.add_enabled_ui(self.bench.cfg.standard, |ui| {
                    ui.separator();
                    ui.label("Resolution");
                    egui::ComboBox::from_id_salt("bench_res")
                        .selected_text(self.bench.cfg.res.label())
                        .show_ui(ui, |ui| {
                            for r in BenchRes::ALL {
                                ui.selectable_value(&mut self.bench.cfg.res, r, r.label());
                            }
                        });
                    ui.add_space(4.0);
                    ui.label("Depth");
                    egui::ComboBox::from_id_salt("bench_depth")
                        .selected_text(self.bench.cfg.depth.label())
                        .show_ui(ui, |ui| {
                            for d in BenchDepth::ALL {
                                ui.selectable_value(&mut self.bench.cfg.depth, d, d.label());
                            }
                        })
                        .response
                        .on_hover_text(
                            "Both dives cross into the deep floatexp/BLA path; Ultra \
                             (1e48×) spends far longer there than Standard (1e32×), and \
                             takes correspondingly longer to run.",
                        );
                    ui.add_space(4.0);
                    ui.checkbox(&mut self.bench.cfg.burnin, "Burn-in (repeat)")
                        .on_hover_text("Run the benchmark repeatedly to reveal stability and thermal throttling.");
                    ui.add_enabled_ui(self.bench.cfg.burnin, |ui| {
                        ui.add(egui::Slider::new(&mut self.bench.cfg.passes, 2..=200).text("passes"));
                    });
                });
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Run").clicked() {
                        run_now = true;
                    }
                    if ui.button("Cancel").clicked() {
                        self.dialogs.bench_dialog_open = false;
                    }
                });
                if self.bench.cfg.standard {
                    let (w, h) = self.bench.cfg.res.dims();
                    ui.add_space(2.0);
                    ui.weak(format!(
                        "Renders offscreen at {w}×{h}, {}× SS, Mandelbrot/smooth, 60-frame dive to 1e{:.0}×.",
                        scripting::STD_AA,
                        self.bench.cfg.depth.zoom_log10(),
                    ));
                }
            });
        self.dialogs.bench_dialog_open = open;
        if run_now {
            self.dialogs.bench_dialog_open = false;
            if self.bench.cfg.standard {
                let passes = if self.bench.cfg.burnin { self.bench.cfg.passes } else { 1 };
                let run = self.begin_standard_bench(self.bench.cfg.res, passes, self.bench.cfg.depth);
                self.bench.std = Some(run);
                ctx.request_repaint();
            } else {
                self.start_benchmark();
            }
        }
    }

    /// Standardized-benchmark progress window (advances one dive-frame per event-loop tick).
    pub(crate) fn draw_bench_progress_dialog(&mut self, ctx: &egui::Context) {
        let Some((label, done, total, last_fps, (fdone, ftotal))) =
            self.bench.std.as_ref().map(|r| {
                (r.res.label(), r.passes_done, r.passes_total, r.pass_fps.last().copied(), r.frame_progress())
            })
        else {
            return;
        };
        let mut cancel = false;
        egui::Window::new("Running benchmark…")
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(format!("Standardized · {label}"));
                });
                if total > 1 {
                    ui.label(format!("Burn-in pass {}/{}", (done + 1).min(total), total));
                } else {
                    ui.label("Rendering the fixed dive…");
                }
                // Per-pass frame progress so it's visibly advancing (not hung) even mid-pass.
                ui.add(
                    egui::ProgressBar::new(fdone as f32 / ftotal.max(1) as f32)
                        .text(format!("frame {fdone}/{ftotal}")),
                );
                if let Some(f) = last_fps {
                    ui.label(format!("last pass: {f:.1} fps"));
                }
                ui.add_space(4.0);
                if ui.button("Cancel").clicked() {
                    cancel = true;
                }
            });
        if cancel {
            if let Some(run) = self.bench.std.take() {
                let report = (!run.pass_fps.is_empty()).then(|| self.format_std_bench(&run));
                let snap = run.take_snapshot();
                self.restore_from_bench(snap);
                if let Some(r) = report {
                    self.bench.report = Some(r);
                    self.dialogs.bench_open = true;
                }
            }
        }
    }

    /// Generic titled-message dialog (`dialogs.notice`) — a one-off notice with a Copy button and
    /// a Close. Used for errors that warrant a dialog over a fleeting toast (e.g. a script that
    /// fails to load), so the message no longer has to borrow an unrelated window's title.
    pub(crate) fn draw_notice_dialog(&mut self, ctx: &egui::Context) {
        let Some((title, body)) = self.dialogs.notice.clone() else {
            return;
        };
        let mut open = true; // the window's own [x] close button
        let mut close_clicked = false; // the explicit Close button inside the body
        egui::Window::new(&title)
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.monospace(&body);
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Copy").clicked() {
                        ui.ctx().copy_text(body.clone());
                    }
                    if ui.button("Close").clicked() {
                        close_clicked = true;
                    }
                });
            });
        if !open || close_clicked {
            self.dialogs.notice = None;
        }
    }

    /// Benchmark results window — show the report, copy/save it, or reopen the config to run again.
    pub(crate) fn draw_bench_results_dialog(&mut self, ctx: &egui::Context) {
        if !self.dialogs.bench_open {
            return;
        }
        let mut open = self.dialogs.bench_open;
        let mut run_again = false;
        // Captured inside the egui closure (which borrows `self`), surfaced as a toast after it.
        // The saved directory is likewise captured out and folded into the shared dialog memory
        // once the closure's borrow of `self` ends.
        let mut bench_save: Option<std::io::Result<()>> = None;
        let mut bench_save_dir: Option<std::path::PathBuf> = None;
        let start_dir = self.dialog_dir_default();
        egui::Window::new("Benchmark results")
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                if let Some(r) = self.bench.report.clone() {
                    ui.monospace(&r);
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("Copy").clicked() {
                            ui.ctx().copy_text(r.clone());
                        }
                        if ui.button("Save…").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Text", &["txt"])
                                .set_directory(&start_dir)
                                .set_file_name("fractadyne_benchmark.txt")
                                .save_file()
                            {
                                bench_save_dir = path.parent().map(|p| p.to_path_buf());
                                bench_save = Some(std::fs::write(path, &r));
                            }
                        }
                        if ui.button("Run again…").clicked() {
                            run_again = true;
                        }
                    });
                } else {
                    ui.label("No benchmark has been run yet.");
                }
            });
        self.dialogs.bench_open = open;
        if let Some(d) = bench_save_dir {
            self.remember_dir(&d);
        }
        if let Some(res) = bench_save {
            self.set_toast(
                match res {
                    Ok(()) => "Benchmark saved.".to_string(),
                    Err(e) => format!("Save failed: {e}"),
                },
                ctx,
            );
        }
        // Close the results and reopen the benchmark tool to configure another run.
        if run_again {
            self.dialogs.bench_open = false;
            self.dialogs.bench_dialog_open = true;
        }
    }

    /// Gallery browser — scan a folder of exported PNG/EXR images and reopen any view (lazy thumbs).
    pub(crate) fn draw_gallery_dialog(&mut self, ctx: &egui::Context) {
        if !self.gallery.open {
            return;
        }
        let mut open = self.gallery.open;
        let mut to_open: Option<String> = None;
        let mut do_rescan = false;
        egui::Window::new("Gallery")
            .open(&mut open)
            .default_size([540.0, 620.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Folder…").clicked() {
                        if let Some(d) = rfd::FileDialog::new()
                            .set_directory(&self.gallery.dir)
                            .pick_folder()
                        {
                            self.remember_dir(&d);
                            self.gallery.dir = d;
                            do_rescan = true;
                        }
                    }
                    if ui.button("Refresh").clicked() {
                        do_rescan = true;
                    }
                    ui.label(
                        egui::RichText::new(self.gallery.dir.display().to_string())
                            .weak()
                            .small(),
                    );
                });
                ui.separator();
                if self.gallery.entries.is_empty() {
                    ui.label("No Fractadyne images in this folder.");
                }
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for entry in &self.gallery.entries {
                        ui.horizontal(|ui| {
                            match &entry.thumb {
                                Some(t) => {
                                    let s = t.size();
                                    ui.add(egui::Image::new(egui::load::SizedTexture::new(
                                        t.id(),
                                        egui::vec2(s[0] as f32, s[1] as f32),
                                    )));
                                }
                                None => {
                                    ui.add_sized(
                                        [160.0, 100.0],
                                        egui::Label::new(egui::RichText::new("…").weak()),
                                    );
                                }
                            }
                            ui.vertical(|ui| {
                                let title = if entry.fractal.is_empty() {
                                    "Fractadyne image"
                                } else {
                                    &entry.fractal
                                };
                                ui.strong(title);
                                ui.label(format!("zoom {}   {}", entry.zoom, entry.saved));
                                if !entry.notes.is_empty() {
                                    ui.label(format!("\u{201c}{}\u{201d}", entry.notes));
                                }
                                ui.label(egui::RichText::new(&entry.app_version).weak().small());
                                if ui.button("Open this view").clicked() {
                                    to_open = Some(entry.meta.clone());
                                }
                            });
                        });
                        ui.separator();
                    }
                });
            });
        self.gallery.open = open;
        if do_rescan {
            self.scan_gallery();
        }
        if let Some(meta) = to_open {
            self.load_view_metadata(&meta);
            self.export.status = Some("Loaded view from gallery.".to_string());
        }
        // Lazily decode one thumbnail per frame so scanning a folder never freezes.
        if let Some(entry) = self.gallery.entries.iter_mut().find(|e| !e.thumb_tried) {
            entry.thumb_tried = true;
            if let Ok((tw, th, rgba)) = fractadyne_export::read_thumbnail(&entry.path, 160) {
                let img = egui::ColorImage::from_rgba_unmultiplied([tw as usize, th as usize], &rgba);
                let name = format!("thumb:{}", entry.path.display());
                entry.thumb = Some(ctx.load_texture(name, img, egui::TextureOptions::LINEAR));
            }
            ctx.request_repaint();
        }
    }

    /// Export-image dialog — width/aspect/format/HUD/folder options, then Export (auto-named into
    /// the chosen folder) or Save as… `gpu` is needed to dispatch the render.
    pub(crate) fn draw_export_dialog(
        &mut self,
        ctx: &egui::Context,
        gpu: &Option<(eframe::wgpu::Device, eframe::wgpu::Queue)>,
    ) {
        if !self.export.open {
            return;
        }
        let mut open = self.export.open;
        let mut do_export = false;
        let mut do_export_as = false;
        egui::Window::new("Export image")
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                // Size presets. The dialog's underlying model is width + aspect (height is
                // derived), so a preset sets BOTH — `aspect_key_for` guarantees the listed
                // dimensions are reproducible in that model, which is why the table is shared with
                // the tour-render dialog rather than duplicated with different rounding.
                let cur_h = self.export_height();
                let cur_preset = crate::STANDARD_SIZES
                    .iter()
                    .find(|(_, w, h)| *w == self.export.width && *h == cur_h)
                    .map(|(l, _, _)| *l);
                let custom = self.export.custom_size || cur_preset.is_none();
                egui::ComboBox::from_label("Size")
                    .width(210.0)
                    // Tall enough for every preset plus Custom: egui's default popup height
                    // showed only the first ten, which hid the ultrawide/DCI entries AND put
                    // "Custom…" behind a scroll nobody would look for.
                    .height(460.0)
                    .selected_text(if custom {
                        format!("Custom — {}×{}", self.export.width, cur_h)
                    } else {
                        cur_preset.unwrap_or_default().to_string()
                    })
                    .show_ui(ui, |ui| {
                        // Custom FIRST, not last: the preset list is long enough to overflow the
                        // popup and scroll, and a "Custom…" below the fold is one nobody finds.
                        // Ordering it first is robust to the list growing; a taller popup is not.
                        if ui
                            .selectable_label(custom, "Custom…")
                            .on_hover_text("Set the width yourself; height follows the Aspect below.")
                            .clicked()
                        {
                            self.export.custom_size = true;
                        }
                        ui.separator();
                        for (label, w, h) in crate::STANDARD_SIZES {
                            let on = !custom && cur_preset == Some(*label);
                            if ui.selectable_label(on, *label).clicked() {
                                self.export.width = *w;
                                if let Some(k) = crate::aspect_key_for(*w, *h) {
                                    self.export.aspect = k.to_string();
                                }
                                self.export.custom_size = false;
                            }
                        }
                    });
                if custom {
                    ui.horizontal(|ui| {
                        ui.label("Width (px)");
                        ui.add(
                            egui::DragValue::new(&mut self.export.width).range(16..=32768).speed(8),
                        );
                        ui.label(format!("× {cur_h} (from Aspect)"));
                    });
                }
                egui::ComboBox::from_label("Supersampling")
                    .selected_text(format!("{}×", self.export.ss))
                    .show_ui(ui, |ui| {
                        for s in [1u32, 2, 3, 4] {
                            ui.selectable_value(&mut self.export.ss, s, format!("{s}×"));
                        }
                    });
                egui::ComboBox::from_label("Aspect")
                    .selected_text(if self.export.aspect == "window" {
                        "Match window".to_string()
                    } else {
                        self.export.aspect.clone()
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.export.aspect,
                            "window".to_string(),
                            "Match window",
                        );
                        for (k, _) in EXPORT_ASPECTS {
                            ui.selectable_value(&mut self.export.aspect, k.to_string(), k);
                        }
                    });
                ui.horizontal(|ui| {
                    ui.label("Format:");
                    ui.radio_value(&mut self.export.format, ExportFormat::Png, "PNG");
                    ui.radio_value(&mut self.export.format, ExportFormat::Exr, "OpenEXR");
                });
                ui.checkbox(&mut self.show_location, "Location HUD")
                    .on_hover_text(
                        "Burn a zoom-level + coordinate panel into the top-left of the exported \
                         image (scales with the output; shows the map/Mandelbrot view's zoom + \
                         center).",
                    );
                ui.checkbox(&mut self.render_cfg.glitch_correct, "Glitch correction")
                    .on_hover_text(
                        "Multi-reference correction of perturbation glitches. Automatically \
                         skipped for very deep (floatexp) single-view exports so the reference \
                         build + render run off-thread and the app stays responsive.",
                    );
                if self.viewport.magnification() >= PERT_FE_THRESHOLD {
                    ui.label(
                        egui::RichText::new(
                            "Deep export: the reference builds off-thread (UI stays live); \
                             glitch correction is skipped at this depth.",
                        )
                        .weak()
                        .small(),
                    );
                }
                if self.dual {
                    egui::ComboBox::from_label("Dual layout")
                        .selected_text(match self.export.dual_mode {
                            DualExport::SideBySide => "Side by side",
                            DualExport::Separate => "Separate files",
                            DualExport::ActiveOnly => "Map only",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.export.dual_mode,
                                DualExport::SideBySide,
                                "Side by side",
                            );
                            ui.selectable_value(
                                &mut self.export.dual_mode,
                                DualExport::Separate,
                                "Separate files",
                            );
                            ui.selectable_value(
                                &mut self.export.dual_mode,
                                DualExport::ActiveOnly,
                                "Map only",
                            );
                        });
                }
                ui.horizontal(|ui| {
                    ui.label("Notes:");
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.export.notes)
                            .char_limit(120)
                            .desired_width(220.0)
                            .hint_text("saved with the image (≤120 chars)"),
                    );
                    if resp.changed() && self.export.notes.chars().count() > 120 {
                        self.export.notes = self.export.notes.chars().take(120).collect();
                    }
                });
                ui.label(format!(
                    "Output: {} × {} px   ({} chars left)",
                    self.export.width,
                    self.export_height(),
                    120usize.saturating_sub(self.export.notes.chars().count()),
                ));
                ui.label(
                    egui::RichText::new(
                        "Rendered in tiles (no size cap) on a background thread. The \
                         file embeds the view + notes so it can be reopened via \
                         File ▸ Open view.",
                    )
                    .weak()
                    .small(),
                );
                // Target directory: a persistent folder that "Export" saves into (auto name).
                let target_dir = self
                    .export.last_dir
                    .clone()
                    .filter(|d| d.is_dir())
                    .unwrap_or_else(Self::pictures_dir);
                ui.horizontal(|ui| {
                    ui.label("Folder:");
                    if ui
                        .button("Choose…")
                        .on_hover_text("Pick the target directory for exports")
                        .clicked()
                    {
                        if let Some(d) = rfd::FileDialog::new().set_directory(&target_dir).pick_folder() {
                            self.remember_dir(&d);
                            self.export.last_dir = Some(d);
                        }
                    }
                });
                ui.label(
                    egui::RichText::new(target_dir.display().to_string())
                        .weak()
                        .small(),
                );
                ui.add_space(6.0);
                let busy = self.export.task.is_some() || self.export.prep.is_some();
                let elapsed = self
                    .export.started
                    .map(|t| Self::fmt_export_duration(t.elapsed()));
                if self.export.prep.is_some() {
                    // Deep export: the bignum reference is building off-thread (UI stays live).
                    ui.horizontal(|ui| {
                        ui.add(egui::Spinner::new());
                        ui.label("Preparing (building reference)…");
                    });
                } else if busy {
                    let p = self.export.progress.load(std::sync::atomic::Ordering::Relaxed);
                    if p >= 2000 {
                        // Rendering done; encoding/writing the file (not cancelable).
                        ui.horizontal(|ui| {
                            ui.add(egui::Spinner::new());
                            ui.label("Saving…");
                        });
                    } else {
                        ui.label("Rendering…");
                        ui.add(egui::ProgressBar::new(p as f32 / 1000.0).show_percentage());
                        if ui.button("Cancel").clicked() {
                            self.export.cancel
                                .store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                }
                // Live elapsed readout while an export is in flight (updated each frame; the
                // busy pollers above request repaints). The final total is folded into the
                // status line on completion.
                if busy {
                    if let Some(t) = &elapsed {
                        ui.label(egui::RichText::new(format!("Elapsed: {t}")).weak().small());
                    }
                } else {
                    ui.horizontal(|ui| {
                        if ui
                            .button("Export")
                            .on_hover_text("Render and save into the folder above (auto-named)")
                            .clicked()
                        {
                            do_export = true;
                        }
                        if ui
                            .button("Save as…")
                            .on_hover_text("Choose the file name and location")
                            .clicked()
                        {
                            do_export_as = true;
                        }
                    });
                }
                if let Some(s) = &self.export.status {
                    ui.add_space(6.0);
                    ui.label(s);
                }
            });
        self.export.open = open;
        if do_export {
            if let Some((dev, q)) = gpu {
                // Save straight into the chosen folder with an auto (timestamped) name.
                let dir = self
                    .export.last_dir
                    .clone()
                    .filter(|d| d.is_dir())
                    .unwrap_or_else(Self::pictures_dir);
                let path = dir.join(self.export_default_name());
                self.start_export_to(ctx, dev.clone(), q.clone(), path);
            } else {
                self.export.status = Some("GPU not available".to_string());
            }
        }
        if do_export_as {
            if let Some((dev, q)) = gpu {
                self.start_export(ctx, dev.clone(), q.clone());
            } else {
                self.export.status = Some("GPU not available".to_string());
            }
        }
    }
}
