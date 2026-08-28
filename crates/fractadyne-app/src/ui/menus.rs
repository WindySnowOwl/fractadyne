//! Top menu bar / toolbar and the bottom status bar (REFACTOR-PLAN Phase 3, intra-crate UI split).
//! `impl FractadyneApp` blocks moved verbatim from `main.rs`.
use crate::*;

impl FractadyneApp {
    /// Top menu bar + toolbar (File / Fractal / View / Tools / Bookmarks / Locations / Help)
    /// plus the icon toolbar. Takes the `gpu` handle for the quick-export toolbar action.
    /// The bookmarks menu: add-current first, then Manage, then the recent bookmarks with
    /// thumbnails.
    ///
    /// Shared by Navigate ▸ Bookmarks and the toolbar star button. One function rather
    /// than two copies on purpose: they are the same menu, and a second copy is a second
    /// thing to forget when bookmarks gain a field.
    pub(crate) fn bookmarks_menu(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if ui.button(format!("{}  Add current view", crate::icons::BOOKMARK)).clicked() {
            self.add_bookmark("");
            ui.close_menu();
        }
        if ui.button("Manage…").clicked() {
            self.dialogs.bookmarks_open = true;
            ui.close_menu();
        }
        if !self.bookmarks.is_empty() {
            ui.separator();
            // Recent bookmarks (most recent first), each with its thumbnail so you
            // can pick by look without opening Manage. Preload the textures (mutable)
            // into owned handles first, then draw (borrow-free in the loop).
            let recent: Vec<usize> = (0..self.bookmarks.len()).rev().take(12).collect();
            let thumbs: Vec<Option<egui::TextureHandle>> = recent
                .iter()
                .map(|&i| {
                    let id = self.bookmarks[i].thumb.clone();
                    self.bookmark_thumb_texture(&ctx, &id)
                })
                .collect();
            let mut jump: Option<usize> = None;
            for (slot, &i) in recent.iter().enumerate() {
                let name = self.bookmarks[i].name.as_str();
                let clicked = if let Some(tex) = &thumbs[slot] {
                    let sz = tex.size_vec2();
                    let w = 64.0_f32;
                    ui.add(egui::Button::image_and_text(
                        egui::Image::new(egui::load::SizedTexture::new(
                            tex.id(),
                            egui::vec2(w, w * sz.y / sz.x.max(1.0)),
                        )),
                        name,
                    ))
                    .clicked()
                } else {
                    ui.button(name).clicked() // older bookmark without a thumbnail
                };
                if clicked {
                    jump = Some(i);
                }
            }
            if let Some(i) = jump {
                let meta = self.bookmarks[i].meta.clone();
                self.load_view_metadata(&meta);
                ui.close_menu();
            }
        }
    }

    pub(crate) fn draw_menu_bar(
        &mut self,
        ctx: &egui::Context,
        gpu: &Option<(eframe::wgpu::Device, eframe::wgpu::Queue)>,
    ) {
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                    // The id egui keys this row's `menu_button` open-state (`BarState`) under —
                    // recorded so `close_menu_bar` can dismiss a hanging menu on view navigation.
                    self.menu_bar_id = Some(ui.id());
                    brand_wordmark(ui);
                    ui.separator();
                    ui.menu_button("File", |ui| {
                        if ui.button(format!("{}  Open view or location…", crate::icons::OPEN)).clicked() {
                            self.open_view(ctx);
                            ui.close_menu();
                        }
                        if ui.button(format!("{}  Gallery…", crate::icons::GALLERY)).clicked() {
                            self.gallery.open = true;
                            self.scan_gallery();
                            ui.close_menu();
                        }
                        if ui.button(format!("{}  Export image…", crate::icons::SAVE)).clicked() {
                            self.export.open = true;
                            ui.close_menu();
                        }
                        if ui
                            .button(format!("{}  Snapshot  (Ctrl+S)", crate::icons::SNAPSHOT))
                            .on_hover_text("Quick-export to the last-used folder, no dialog")
                            .clicked()
                        {
                            if let Some((dev, q)) = gpu {
                                self.quick_export(ctx, dev.clone(), q.clone());
                            }
                            ui.close_menu();
                        }
                        ui.separator();
                        // Settings live under File — the conventional home users reach for first
                        // (File → Preferences/Settings); they sat under View until 2026-08-13,
                        // where only display TOGGLES belong.
                        ui.menu_button(format!("{}  Settings", crate::icons::SETTINGS), |ui| {
                            ui.label("Frame-rate cap");
                            for (label, val) in [
                                ("Uncapped", None),
                                ("30 FPS", Some(30.0)),
                                ("60 FPS", Some(60.0)),
                                ("120 FPS", Some(120.0)),
                            ] {
                                if ui.selectable_label(self.fps_cap == val, label).clicked() {
                                    self.fps_cap = val;
                                }
                            }
                            ui.separator();
                            ui.label("UI scale (font size)");
                            for (label, val) in [
                                ("80%", 0.8_f32),
                                ("90%", 0.9),
                                ("100%", 1.0),
                                ("110%", 1.1),
                                ("125%", 1.25),
                                ("150%", 1.5),
                            ] {
                                if ui
                                    .selectable_label((self.ui_scale - val).abs() < 0.01, label)
                                    .clicked()
                                {
                                    self.ui_scale = val;
                                }
                            }
                            ui.separator();
                            ui.label("Theme");
                            for m in [ThemeMode::Dark, ThemeMode::Light] {
                                if ui.selectable_label(self.theme == m, m.label()).clicked() {
                                    self.theme = m;
                                    apply_theme(ui.ctx(), m);
                                }
                            }
                            ui.separator();
                            ui.label("Updates");
                            ui.horizontal(|ui| {
                                for t in crate::update::UpdateTrack::ALL {
                                    ui.selectable_value(&mut self.update_track, t, t.label())
                                        .on_hover_text(match t {
                                            crate::update::UpdateTrack::Stable => "Latest stable release",
                                            crate::update::UpdateTrack::Beta => "Latest build including pre-releases",
                                        });
                                }
                            });
                            ui.checkbox(
                                &mut self.update_check_on_launch,
                                "Check for updates on launch",
                            )
                            .on_hover_text("Otherwise, check manually via Help → Check for updates.");
                        });
                        ui.separator();
                        if ui
                            .button(format!("{}  Reset application state…", crate::icons::RESET_APP))
                            .on_hover_text(
                                "Delete all saved state (session, bookmarks, thumbnails) and \
                                 start fresh. Asks for confirmation first.",
                            )
                            .clicked()
                        {
                            self.dialogs.reset_confirm_open = true;
                            ui.close_menu();
                        }
                        if ui.button(format!("{}  Quit", crate::icons::QUIT)).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.menu_button("Fractal", |ui| {
                        for kind in FractalKind::ALL {
                            if ui
                                .selectable_label(self.fractal == kind, kind.name())
                                .clicked()
                            {
                                self.set_fractal(kind);
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        let can_julia = self.fractal.supports_julia();
                        ui.add_enabled_ui(can_julia, |ui| {
                            if ui
                                .checkbox(&mut self.julia_mode, "Julia mode")
                                .on_hover_text(
                                    "Show the Julia set of this formula for the current c.",
                                )
                                .changed()
                            {
                                self.invalidate_refs();
                            }
                            // Beside Julia mode, not in View: both select WHAT renders.
                            if ui
                                .checkbox(&mut self.dual, "Dual view (Mandelbrot ↔ Julia)")
                                .on_hover_text(
                                    "Split the window: the parameter set on the left, the Julia \
                                     set of the cursor's c on the right.",
                                )
                                .clicked()
                                && self.dual
                            {
                                self.julia_viewport.reset();
                                self.julia_viewport.center_x =
                                    fractadyne_core::BigFloat::from_f64(0.0, 64);
                                self.julia_viewport.center_y =
                                    fractadyne_core::BigFloat::from_f64(0.0, 64);
                            }
                        });
                    });
                    ui.menu_button("View", |ui| {
                        let now = ui.ctx().input(|i| i.time);
                        // "Home view" vs "Reset view" were near-synonyms distinguishable only by
                        // tooltip (2026-08-13 UI review) — the labels now say what each does.
                        if ui
                            .button(format!("{}  Zoom out to full view", crate::icons::HOME))
                            .on_hover_text("Zoom out to the full home view (animated)")
                            .clicked()
                        {
                            self.zoom_home(now);
                            ui.close_menu();
                        }
                        if ui
                            .button(format!("{}  Reset to default view", crate::icons::RESET_VIEW))
                            .on_hover_text("Reset to the fractal's default view (instant)")
                            .clicked()
                        {
                            self.reset_view();
                            ui.close_menu();
                        }
                        ui.add_enabled_ui(self.nav.undo.len() > 1, |ui| {
                            if ui.button("Undo view  (Ctrl+Z / Backspace)").clicked() {
                                self.undo_view();
                                ui.close_menu();
                            }
                        });
                        ui.add_enabled_ui(!self.nav.redo.is_empty(), |ui| {
                            if ui.button("Redo view  (Ctrl+Y / Shift+Backspace)").clicked() {
                                self.redo_view();
                                ui.close_menu();
                            }
                        });
                        ui.separator();
                        ui.checkbox(&mut self.dialogs.right_panel_open, "Control panel")
                            .on_hover_text("Show/hide the right-hand control panel.");
                        ui.checkbox(&mut self.dialogs.minimap, "Minimap overview")
                            .on_hover_text(
                                "Show a small home-view overview with a \"you are here\" \
                                 marker and the zoom depth. Click it to jump to a region.",
                            );
                        ui.checkbox(&mut self.perf.enabled, "Performance panel");
                        ui.checkbox(&mut self.watermark, "Show \"Fd\" watermark")
                            .on_hover_text(
                                "Draw a discreet \"Fd\" brand mark in the lower-right corner of the \
                                 live view and exported images. On by default.",
                            );
                        // Dual view moved to Fractal (it selects WHAT renders, like Julia mode);
                        // the orbit overlay's toggle lives in Tools with its options in the
                        // control panel's Overlays section (2026-08-13 UI review: View mixed four
                        // unrelated concerns).
                    });
                    // Coloring is where users spend the most time, and until 2026-08-13 the menu
                    // bar said nothing about it (UI review). This mirrors the control panel's
                    // essentials; the sliders (cycle/offset/animation) stay in the panel, where
                    // continuous adjustment belongs.
                    ui.menu_button("Color", |ui| {
                        ui.menu_button("Method", |ui| {
                            for m in crate::ColorMethod::ALL {
                                if ui
                                    .selectable_label(self.coloring.color_method == m, m.label())
                                    .clicked()
                                {
                                    self.coloring.color_method = m;
                                    ui.close_menu();
                                }
                            }
                        });
                        ui.menu_button("Palette", |ui| {
                            let is_preset = !self.coloring.use_custom_palette
                                && !self.coloring.use_duotone
                                && !self.coloring.use_binary;
                            for (i, p) in fractadyne_color::PRESETS.iter().enumerate() {
                                if ui
                                    .selectable_label(
                                        is_preset && self.coloring.palette_idx == i,
                                        p.name,
                                    )
                                    .clicked()
                                {
                                    self.coloring.palette_idx = i;
                                    self.coloring.use_custom_palette = false;
                                    self.coloring.use_duotone = false;
                                    self.coloring.use_binary = false;
                                    ui.close_menu();
                                }
                            }
                            ui.separator();
                            if ui
                                .selectable_label(self.coloring.use_custom_palette, format!("Custom {}", crate::icons::EDIT))
                                .clicked()
                            {
                                if self.coloring.custom_palette.is_empty() {
                                    self.coloring.custom_palette =
                                        self.preset_as_stops(self.coloring.palette_idx);
                                }
                                self.coloring.use_custom_palette = true;
                                self.coloring.use_duotone = false;
                                self.coloring.use_binary = false;
                                ui.close_menu();
                            }
                            if ui
                                .selectable_label(self.coloring.use_duotone, "Duotone")
                                .clicked()
                            {
                                self.coloring.use_duotone = true;
                                self.coloring.use_custom_palette = false;
                                self.coloring.use_binary = false;
                                ui.close_menu();
                            }
                            if ui
                                .selectable_label(self.coloring.use_binary, "Binary (set)")
                                .on_hover_text("Flat two-color: in-set vs out-of-set, no gradient.")
                                .clicked()
                            {
                                self.coloring.use_binary = true;
                                self.coloring.use_custom_palette = false;
                                self.coloring.use_duotone = false;
                                ui.close_menu();
                            }
                        });
                        if ui.button("Edit gradient…").clicked() {
                            if self.coloring.custom_palette.is_empty() {
                                self.coloring.custom_palette =
                                    self.preset_as_stops(self.coloring.palette_idx);
                            }
                            self.coloring.use_custom_palette = true;
                            self.coloring.palette_editor_open = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.checkbox(&mut self.coloring.normalize_live, "Normalize deep colors")
                            .on_hover_text(
                                "Remap the palette to the view's measured escape range at extreme \
                                 depth, so dense fields read as structure instead of speckle. \
                                 Smooth method only; ordinary views are unaffected.",
                            );
                        ui.label(
                            egui::RichText::new(
                                "Cycle, offset, animation and effects: Controls panel ▸ Coloring",
                            )
                            .weak()
                            .small(),
                        );
                    });
                    ui.menu_button("Tools", |ui| {
                        let ctx = ui.ctx().clone();
                        // The mathematical tools lead: they are what distinguishes this app from
                        // every other deep-zoom explorer, and until 2026-08-13 they were filed
                        // under Locations / buried in a dialog tooltip (UI review).
                        ui.add_enabled_ui(matches!(self.fractal.formula_id(), 0..=3), |ui| {
                            if ui
                                .button("Find minibrot + zoom to it  (M)")
                                .on_hover_text(
                                    "Newton-snap the view center to the nearby minibrot's exact \
                                     center, report its period, and zoom to the minibrot's own \
                                     scale — often many orders of magnitude in one step. Already \
                                     deeper than it? Only the center moves.",
                                )
                                .clicked()
                            {
                                self.find_minibrot(&ctx);
                                ui.close_menu();
                            }
                            if ui
                                .button("Newton / Misiurewicz solver…")
                                .on_hover_text(
                                    "Solve for the exact minibrot center (period + size) or the \
                                     Misiurewicz point nearest the view — Newton's method at full \
                                     precision, with the repelling multiplier λ. Opens the \
                                     location dialog, which hosts the solvers.",
                                )
                                .clicked()
                            {
                                self.open_goto();
                                ui.close_menu();
                            }
                        });
                        ui.add_enabled_ui(!self.dual, |ui| {
                            let label = if self.autopilot.active {
                                "Stop autopilot  (A)"
                            } else {
                                "Auto-zoom (autopilot)  (A)"
                            };
                            if ui
                                .button(label)
                                .on_hover_text(
                                    "Hands-free continuous deep zoom: dives toward the \
                                     detail-richest region, re-steering as it goes. Any \
                                     navigation input stops it.",
                                )
                                .clicked()
                            {
                                self.toggle_autopilot(&ctx);
                                ui.close_menu();
                            }
                        });
                        ui.checkbox(&mut self.anim.show_orbits, "Orbit overlay")
                            .on_hover_text(
                                "Draw the iteration path of the point under the cursor. \
                                 Fit/animation options: Controls panel ▸ Overlays.",
                            );
                        ui.separator();
                        if ui
                            .button("Play tour…")
                            .on_hover_text(
                                "Pick a tour script to play. The toolbar ▶ replays the last one \
                                 with a single click.",
                            )
                            .clicked()
                        {
                            self.load_script();
                            ui.close_menu();
                        }
                        if ui
                            .button("Tour from current view…")
                            .on_hover_text(
                                "Create a tour that zooms from the full view down to the current \
                                 view — add a caption and set the dive duration. Saved as a tour \
                                 script (.toml) you can edit, play, or render.",
                            )
                            .clicked()
                        {
                            self.open_script_export();
                            ui.close_menu();
                        }
                        ui.add_enabled_ui(self.playback.is_some(), |ui| {
                            if ui
                                .button("Render tour…")
                                .on_hover_text(
                                    "Render the loaded tour to a PNG frame sequence (and \
                                     optionally an mp4) in a separate process.",
                                )
                                .clicked()
                            {
                                self.open_tour_render();
                                ui.close_menu();
                            }
                            // Same thing the player's ✖ does — the transport's own ⏹ only rewinds.
                            if ui
                                .button("Close tour player")
                                .on_hover_text(
                                    "Stop the tour, close its player, and restore your own \
                                     iteration and coloring settings",
                                )
                                .clicked()
                            {
                                self.stop_playback();
                                ui.close_menu();
                            }
                        });
                        ui.separator();
                        if ui
                            .button("Benchmark…")
                            .on_hover_text(
                                "Measure rendering speed — current settings, or a standardized \
                                 run (fixed resolution + settings) comparable across machines. \
                                 Burn-in repeats it to check stability / thermal throttling.",
                            )
                            .clicked()
                        {
                            self.dialogs.bench_dialog_open = true;
                            ui.close_menu();
                        }
                        if self.bench_report.is_some()
                            && ui.button("Show last benchmark").clicked()
                        {
                            self.dialogs.bench_open = true;
                            ui.close_menu();
                        }
                    });
                    // "Where can I go?" is ONE question: the canonical places (famous points,
                    // random, .kfr import) and the personal ones (bookmarks) were split across
                    // two menus until 2026-08-13 (UI review). Merged; bookmarks are the submenu.
                    ui.menu_button("Navigate", |ui| {
                        let ctx = ui.ctx().clone();
                        if ui
                            .button("Go to location…")
                            .on_hover_text(
                                "Enter a center/zoom, jump to a well-known point, or Newton-solve \
                                 a Misiurewicz / minibrot feature near the view.",
                            )
                            .clicked()
                        {
                            self.open_goto();
                            ui.close_menu();
                        }
                        ui.separator();
                        for (name, cx, cy, mag) in FAMOUS {
                            if ui.button(*name).clicked() {
                                self.goto_location(cx, cy, *mag, name, &ctx);
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        if ui
                            .button(format!("{}  Random location", crate::icons::RANDOM))
                            .on_hover_text("Jump to a random detail-rich boundary point")
                            .clicked()
                        {
                            self.random_location(&ctx);
                            ui.close_menu();
                        }
                        if ui
                            .button("Import .kfr…")
                            .on_hover_text("Load a Kalles Fraktaler location file")
                            .clicked()
                        {
                            self.import_kfr(&ctx);
                            ui.close_menu();
                        }
                        // Sharing a location is a PLACES concern (moved from File 2026-08-13):
                        // it shares where you are, like the .kfr import shares where someone was.
                        if ui
                            .button(format!("{}  Share location…", crate::icons::SHARE))
                            .on_hover_text(
                                "Copy / paste / save / load a self-contained location \
                                 (.fdn): fractal, full-precision center, zoom, coloring.",
                            )
                            .clicked()
                        {
                            self.open_share();
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.menu_button(format!("{}  Bookmarks", crate::icons::BOOKMARK), |ui| {
                            self.bookmarks_menu(ui, &ctx);
                        });
                    });
                    ui.menu_button("Help", |ui| {
                        if ui.button("Help & reference…  (F1)").clicked() {
                            self.dialogs.help_open = true;
                            ui.close_menu();
                        }
                        if ui
                            .button("Welcome / quick start…")
                            .on_hover_text("The first-run overview: the essential controls and a few landmarks.")
                            .clicked()
                        {
                            self.dialogs.welcome_open = true;
                            ui.close_menu();
                        }
                        if ui
                            .button("Diagnostics…")
                            .on_hover_text(
                                "Check that Fractadyne works correctly on your hardware, and \
                                 attach the result to an issue report",
                            )
                            .clicked()
                        {
                            self.diagnostics.open = true;
                            ui.close_menu();
                        }
                        if ui
                            .button("Report an issue…")
                            .on_hover_text(
                                "File a GitHub issue (or email) with a crash report, log, and \
                                 location attached",
                            )
                            .clicked()
                        {
                            self.report.open = true;
                            ui.close_menu();
                        }
                        // Named for the BENEFIT, not the implementation: nobody is looking for
                        // "MPFR". The dialog itself explains what it actually is.
                        if ui
                            .button("Faster deep zoom\u{2026}")
                            .on_hover_text(
                                "An optional build that computes deep-zoom reference orbits \
                                 2.5-6.4x faster. Identical images; your settings carry over.",
                            )
                            .clicked()
                        {
                            self.dialogs.accelerated_open = true;
                            ui.close_menu();
                        }
                        if ui
                            .button("Check for updates")
                            .on_hover_text(format!(
                                "Check GitHub for a newer {} build",
                                self.update_track.label()
                            ))
                            .clicked()
                        {
                            self.start_update_check(true);
                            ui.close_menu();
                        }
                        match &self.update_status {
                            Some(crate::update::UpdateStatus::Available { version, url, prerelease }) => {
                                let url = url.clone();
                                let channel = crate::update::channel_word(*prerelease);
                                if ui
                                    .button(
                                        egui::RichText::new(format!("\u{2B06} {version} ({channel}) available — download"))
                                            .color(egui::Color32::from_rgb(0x5C, 0xC0, 0x6C)),
                                    )
                                    .clicked()
                                {
                                    ctx.open_url(egui::OpenUrl::new_tab(url));
                                    ui.close_menu();
                                }
                            }
                            Some(crate::update::UpdateStatus::UpToDate) => {
                                ui.label(egui::RichText::new("Up to date").weak().small());
                            }
                            _ => {}
                        }
                        ui.separator();
                        ui.label(format!("Fractadyne v{}", version_string()));
                        ui.label(egui::RichText::new("Native fractal explorer").weak().small());
                        ui.separator();
                        ui.label(egui::RichText::new("License").weak().small());
                        ui.label("MIT OR Apache-2.0");
                        ui.label(
                            egui::RichText::new("© 2026 Rithea Hong. Dual-licensed; use \
                                 under either license at your option.")
                                .weak()
                                .small(),
                        );
                        ui.hyperlink_to(
                            "Source \u{2197}",
                            "https://github.com/WindySnowOwl/fractadyne",
                        );
                    });

                ui.separator();

                // Fractal picker (the name, as a dropdown).
                let prev = self.fractal;
                let mut sel = self.fractal;
                egui::ComboBox::from_id_salt("fractal_dropdown")
                    .selected_text(self.fractal.name())
                    .show_ui(ui, |ui| {
                        for k in FractalKind::ALL {
                            ui.selectable_value(&mut sel, k, k.name());
                        }
                    });
                if sel != prev {
                    self.set_fractal(sel);
                }
                ui.separator();
                ui.add_enabled_ui(self.fractal.supports_julia() && !self.dual, |ui| {
                    if ui
                        .selectable_label(self.julia_mode, "Julia")
                        .on_hover_text("Show the Julia set of this formula")
                        .clicked()
                    {
                        self.julia_mode = !self.julia_mode;
                        self.invalidate_refs();
                    }
                });
                // Dual pairs a formula with its Julia set, so it's only meaningful where a Julia
                // exists (Newton has no free parameter → no Julia → no dual).
                ui.add_enabled_ui(self.fractal.supports_julia(), |ui| {
                    if dual_toggle_button(ui, self.dual)
                        .on_hover_text("Dual linked view (Mandelbrot ↔ Julia)")
                        .clicked()
                    {
                        self.toggle_dual();
                    }
                });
                ui.separator();
                // ── File / I-O: open & browse, then save ──────────────────────────────
                if ui.button(crate::icons::OPEN).on_hover_text("Open a view or location (PNG / EXR, .fdn, .kfr)").clicked() {
                    self.open_view(ctx);
                }
                if ui.button(crate::icons::GALLERY).on_hover_text("Gallery").clicked() {
                    self.gallery.open = true;
                    self.scan_gallery();
                }
                if ui.button(crate::icons::SAVE).on_hover_text("Export image…").clicked() {
                    self.export.open = true;
                }
                if ui
                    .button(crate::icons::SNAPSHOT)
                    .on_hover_text("Snapshot — quick export to the last folder (Ctrl+S)")
                    .clicked()
                {
                    if let Some((dev, q)) = gpu {
                        self.quick_export(ctx, dev.clone(), q.clone());
                    }
                }
                ui.separator();
                // ── Navigation / location: zoom, reset/home, bookmark ─────────────────
                if ui.button(crate::icons::ZOOM_IN).on_hover_text("Zoom in").clicked() {
                    self.zoom_center(0.5);
                }
                if ui.button(crate::icons::ZOOM_OUT).on_hover_text("Zoom out").clicked() {
                    self.zoom_center(2.0);
                }
                // Click-to-zoom tool (single view): arm left-click = dive into the point,
                // right-click = back out; drag still pans. Factor set in Settings ▸ Navigation.
                ui.add_enabled_ui(!self.dual, |ui| {
                    if ui
                        .selectable_label(self.click_zoom, crate::icons::CLICK_ZOOM)
                        .on_hover_text(format!(
                            "Click-to-zoom ({:.0}×): left-click dives into the point, \
                             right-click backs out (drag still pans). Factor in Settings ▸ Navigation.",
                            self.render_cfg.click_zoom_factor
                        ))
                        .clicked()
                    {
                        self.click_zoom = !self.click_zoom;
                    }
                });
                // Auto-zoom (autopilot): highlighted while running; click to start/stop. Single view only.
                ui.add_enabled_ui(!self.dual, |ui| {
                    let running = self.autopilot.active;
                    if ui
                        .selectable_label(running, crate::icons::AUTOPILOT)
                        .on_hover_text(if running {
                            "Auto-zoom is running — click to stop"
                        } else {
                            "Auto-zoom: dive toward detail (A)"
                        })
                        .clicked()
                    {
                        self.toggle_autopilot(ctx);
                    }
                });
                if ui.button(crate::icons::RESET_VIEW).on_hover_text("Reset view (instant)").clicked() {
                    self.reset_view();
                }
                if ui
                    .button(crate::icons::HOME)
                    .on_hover_text("Zoom out to the home view (animated)")
                    .clicked()
                {
                    let now = ctx.input(|i| i.time);
                    self.zoom_home(now);
                }
                // A DROPDOWN, not a one-shot add: the button used to save silently on click,
                // which gave no way to reach a saved view from the toolbar and no feedback that
                // anything had happened. Same menu as Navigate > Bookmarks, via one helper.
                ui.menu_button(crate::icons::BOOKMARK, |ui| {
                    self.bookmarks_menu(ui, ctx);
                })
                .response
                .on_hover_text("Bookmarks: save this view, or jump to a saved one");
                ui.separator();
                // ── Appearance / display ──────────────────────────────────────────────
                if ui
                    .button(crate::icons::PALETTE)
                    .on_hover_text(format!(
                        "Next palette ({})",
                        fractadyne_color::PRESETS[self.coloring.palette_idx].name
                    ))
                    .clicked()
                {
                    self.coloring.palette_idx = (self.coloring.palette_idx + 1) % fractadyne_color::PRESETS.len();
                }
                if ui
                    .selectable_label(self.perf.enabled, crate::icons::PERF)
                    .on_hover_text("Performance panel")
                    .clicked()
                {
                    self.perf.enabled = !self.perf.enabled;
                }
                ui.separator();
                // ── Tours: play a script, then a speed control while one runs ─────────────
                // The tours are the most demo-able thing in the app and nothing on screen used to
                // suggest they exist. ▶ replays the last script in one click (falling back to the
                // picker); the hover names it.
                let play_hover = match self.last_script.as_ref() {
                    Some(p) => format!(
                        "Play tour — {} (click) · replays the last tour; pick another via Tools ▸ Play tour…",
                        p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
                    ),
                    None => "Play a tour… (Tools ▸ Play tour)".to_string(),
                };
                if ui
                    .add_enabled(!self.tour_playing(), egui::Button::new(crate::icons::PLAY))
                    .on_hover_text(play_hover)
                    .clicked()
                {
                    self.play_last_or_pick_script();
                }
                // Speed picker: only meaningful while a tour plays. A direct picker (not the
                // transport's cycle button) so the whole set is one click away. The transport's
                // own `1×` button still exists and stays in sync (both read/write `pb.speed`).
                if let Some(pb) = self.playback.as_mut() {
                    egui::ComboBox::from_id_salt("toolbar_tour_speed")
                        .selected_text(format!("{}×", crate::fmt_speed(pb.speed)))
                        .width(52.0)
                        .show_ui(ui, |ui| {
                            for &sp in &[0.25f64, 0.5, 1.0, 2.0, 4.0, 8.0] {
                                ui.selectable_value(&mut pb.speed, sp, format!("{}×", crate::fmt_speed(sp)));
                            }
                        })
                        .response
                        .on_hover_text("Playback speed");
                }
                if ui
                    .selectable_label(self.fullscreen, crate::icons::FULLSCREEN)
                    .on_hover_text("Toggle fullscreen")
                    .clicked()
                {
                    self.fullscreen = !self.fullscreen;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
                }
            });
        });
    }

    /// Bottom status bar — center coordinate, cursor, zoom, effective iteration count, and the
    /// live script / benchmark playback progress.
    /// Classify which rendering limit (if any) is binding, for the status-bar diagnostic.
    /// Returns `(label, hover detail, severe)` — severe = red (a hard wall), else amber.
    ///
    /// Deliberately quiet during ordinary dives: a motion-capped partial reference (≤
    /// `LIVE_REF_CAP`, nothing refused) is transient machinery, not a limit the user needs to
    /// hear about. The conditions, in priority order, are the three walls actually measured on
    /// the Misiurewicz spar family:
    /// 1. the reference is still PARTIAL at the GPU buffer cap (2e95×: nothing deeper is
    ///    computable) — since v0.2.40-beta.49/50 the freeze guard INSTALLS this partial at a floor
    ///    budget rather than refusing it, so the wall is read off the installed length,
    /// 2. pixels exhausting a budget the app cannot raise further, with escapes pressing the
    ///    ceiling (starvation — as opposed to ordinary in-set pixels, which always cap).
    #[allow(clippy::too_many_arguments)] // a pure classifier over independent state — a struct would just rename the args
    pub(crate) fn limit_status(
        partial: bool,
        orbit_len: u32,
        dev_cap: u32,
        eff_iter: u32,
        capped: Option<f64>,
        budget_measured: u32,
        budget_maxed: bool,
        esc_max: Option<f64>,
        plateau: bool,
        exhausted: bool,
    ) -> Option<(&'static str, String, bool)> {
        let clamp_px = orbit_len.saturating_sub(1);
        // The reference is still partial (never escaped) at the GPU buffer cap: pixels are limited
        // to that length and nothing deeper is computable at this view. Guard v2 installs this
        // partial (at a floor budget) rather than refusing it, so the wall shows as `partial` with
        // `orbit_len` at the device cap — no refusal record involved.
        if partial && orbit_len >= dev_cap {
            return Some((
                "⚠ depth limit",
                format!(
                    "The reference orbit reached the GPU buffer cap ({} samples) without \
                     escaping, so pixels here are limited to ~{} iterations and finer detail \
                     cannot be computed at this view — live or exported. This is the current \
                     depth limit for this location family.",
                    commas(&dev_cap.to_string()),
                    commas(&clamp_px.to_string())
                ),
                true,
            ));
        }
        // Pixels exhausting the budget only means STARVATION if raising would help. In-set
        // (interior) pixels always exhaust it — a view containing a minibrot is legitimately a few
        // percent "capped" and warning there would cry wolf on the most ordinary deep view. The
        // discriminator is where the ESCAPED pixels finish: under starvation escapes press right
        // against the ceiling, whereas around a genuine minibrot they finish far below it (the
        // 2e82× view escaped at 833k–1.12M under a 10M budget). A fully-capped frame has no
        // escaped pixels to measure and is starved by definition.
        // The probe climbed to the full appetite and the frame stayed all-capped. Report it: this
        // is the state that produced a black screen with no explanation (6.5e94×, iterations at
        // 10M). Deliberately does NOT claim which cause — from the escape data the two are
        // indistinguishable, and saying "raise iterations" when they are already at the maximum
        // would be worse than saying nothing.
        if exhausted {
            return Some((
                "⚠ iter exhausted",
                format!(
                    "Every pixel still ran out of iterations at the largest budget this view can \
                     be given ({}). Either the view is inside the set here (normal at depth), or \
                     it needs more iterations than the app can currently provide — the image is \
                     the same either way, so the budget was returned to a cheap one.",
                    commas(&eff_iter.to_string())
                ),
                false,
            ));
        }
        if !plateau && budget_maxed {
            if let Some(frac) = capped {
                let pressing = frac > 0.98
                    || esc_max.is_some_and(|mx| mx >= 0.9 * budget_measured.max(1) as f64);
                if frac > 0.02 && pressing {
                    let hint = if eff_iter >= crate::MAX_ITER_LIMIT {
                        "Iterations is already at the maximum, so this view needs more than the \
                         app can currently give."
                    } else {
                        "Raise Iterations (Rendering panel) to resolve it."
                    };
                    return Some((
                        "⚠ iter capped",
                        format!(
                            "{:.0}% of pixels ran out of iterations at the {} budget, and the \
                             escaping ones are finishing right at that ceiling — the view is \
                             under-resolved. {hint}",
                            frac * 100.0,
                            commas(&budget_measured.to_string())
                        ),
                        false,
                    ));
                }
            }
        }
        None
    }

    /// Bottom status bar: view readouts only. Anything CLICKABLE belongs elsewhere — this bar is
    /// sized by its content, and the centre coordinates gain and lose digits as the view moves, so
    /// a control placed here slides horizontally under the cursor. The playback transport lives in
    /// `draw_playback_transport` for exactly that reason.
    pub(crate) fn draw_status_bar(&mut self, ctx: &egui::Context) {
        let resp = egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            // WRAPPED, not a single row: at depth the centre coordinates alone can be most of the
            // width, and a plain `horizontal` silently CLIPS whatever doesn't fit — the limit
            // diagnostics on the right would vanish on a narrow window, which is precisely when a
            // user needs to be told why the view is black. Wrapping costs a second line only when
            // it is actually needed.
            ui.horizontal_wrapped(|ui| {
                // ⚠Every readout is ATOMIC. `TextWrapMode::Extend` stops egui breaking a label at
                // the PADDING SPACES inside a reserved slot: `iter {:>w$}` is one string of
                // `iter`, spaces, then the count, and the default word-wrap happily splits it, so
                // `iter` sat on line 1 with `9,845` on line 2.
                //
                // That is the beta.149 reflow bug wearing a different hat, not a cosmetic one. The
                // split point moves with the DIGIT COUNT of a value that grows as the view settles,
                // so the bar's height changes while a deep view resolves - it shrinks the canvas, the
                // renderer reads that as a resize and restarts, and the numbers change again. The
                // width reservation exists to stop exactly that, and it is itself made of the spaces
                // egui was wrapping at. With `Extend` a readout that does not fit moves WHOLE onto
                // the next line, which is what `horizontal_wrapped` was chosen for.
                let mono = |ui: &mut egui::Ui, t: String| {
                    ui.add(
                        egui::Label::new(egui::RichText::new(t).monospace())
                            .wrap_mode(egui::TextWrapMode::Extend),
                    )
                };
                let l2 = self.viewport.log2_magnification();
                mono(ui, format!(
                    "center {}, {}",
                    fmt_coord_deep(&self.viewport.center_x, l2),
                    fmt_coord_deep(&self.viewport.center_y, l2),
                ));
                ui.separator();
                // ⭐The cursor readout is reserved at a CONSTANT width. It only appears when the
                // pointer is over the view, and this bar is content-sized and WRAPPING — so on a
                // narrow window, moving the mouse onto the canvas popped a wide `cursor x, y` in,
                // wrapped the bar to a second line, grew its height, shrank the canvas, and forced
                // a full re-render on every mouse-move (a wide window, which never wraps, never
                // did). Both `fmt_coord` values are right-padded to a fixed field so neither the
                // presence of the readout nor the pointer's position changes where the bar breaks:
                // a narrow window now wraps identically whether the mouse is on the view or off it,
                // so hovering no longer reflows the layout. `—` fills the field when the pointer is
                // elsewhere. (21 chars holds a grouped 15-dp coord: sign + up-to-1 int digit + `.`
                // + 15 fractional + 2 group spaces ≈ 20.)
                let (cx_s, cy_s) = match self.pointer.pointer_complex {
                    Some((mx, my)) => (fmt_coord(mx), fmt_coord(my)),
                    None => ("—".to_string(), "—".to_string()),
                };
                mono(ui, format!("cursor {cx_s:>21}, {cy_s:>21}"));
                ui.separator();
                // ⭐⭐**RESERVED WIDTH, for the same reason the cursor readout above is.** These
                // two fields CHANGE WIDTH as the view changes — `fmt_zoom_log2` grows an exponent,
                // and the iteration count below grows digits — and an unreserved bar that gets
                // one glyph wider on a window where it just fits wraps to two lines. That resizes
                // the central panel, which `central.rs`'s resize-detector treats as an
                // INTERACTION: view generation bumped, settle grid torn down, re-render, counters
                // move, width changes again. The beta.70/71 saga fixed exactly this loop for the
                // limit LABEL and left these two numeric fields unreserved.
                //
                // ⛔It is a live field report (2026-08-25): a deep dual view sat black while
                // `iter 1,395,703` wrapped the bar onto a second line. ⚠And beta.148 made it
                // easier to hit, not harder — once the adaptive probe could see a starved frame,
                // the iteration count started MOVING every settle, so the width changed
                // constantly.
                //
                // Right-aligned into a fixed slot: monospace, so equal char counts are equal
                // pixels by construction (`zoom_slot_width` is pinned by a test).
                let zw = crate::zoom_slot_width();
                if self.dual {
                    mono(ui, format!(
                        "zoom  M {:>zw$}×   J {:>zw$}×",
                        fmt_zoom_log2(self.viewport.log2_magnification()),
                        fmt_zoom_log2(self.julia_viewport.log2_magnification()),
                    ));
                } else {
                    mono(ui, format!(
                        "zoom {:>zw$}×",
                        fmt_zoom_log2(self.viewport.log2_magnification())
                    ));
                }
                ui.separator();
                // Show the count actually rendered last frame (coarse while moving, full when
                // settled) — matches the Performance panel's "eff iter".
                let eff_iter = if self.perf.last_eff_iter > 0 {
                    self.perf.last_eff_iter
                } else {
                    let want_iter = if self.render_cfg.auto_iter {
                        self.viewport.recommended_max_iter(self.render_cfg.max_iter)
                    } else {
                        self.render_cfg.max_iter
                    };
                    want_iter.min(zoom_iter_cap(self.viewport.log2_magnification()).max(256))
                };
                // Widest is `MAX_ITER_LIMIT` grouped — see the reservation note on `zoom` above.
                mono(ui, format!(
                    "iter {:>w$}",
                    commas(&eff_iter.to_string()),
                    w = crate::iter_slot_width()
                ));
                // Rendering-limit diagnostics: when a cap is genuinely binding, say so where the
                // user is already looking, instead of leaving a black/flat view unexplained (the
                // Misiurewicz-spar reports arrived as mystery screenshots precisely because the
                // app knew it was clamped and said nothing).
                let vc = &self.ref_cache[0];
                let limit = Self::limit_status(
                    vc.partial,
                    vc.orbit_len,
                    crate::render::orbit_len_cap(),
                    eff_iter,
                    self.perf.capped_frac[0],
                    self.perf.budget_measured[0],
                    self.perf.budget_maxed[0],
                    self.perf.norm_range[0].map(|(_, mx)| mx as f64),
                    self.perf.iter_plateau[0],
                    self.perf.iter_exhausted[0],
                );
                // ⭐The diagnostic slot is ALWAYS the SAME WIDGET with the SAME METRICS — like the
                // cursor readout above, but stricter. The label comes and goes with live counters;
                // on a width where the bar just fits, its arrival wrapped the bar to two lines,
                // which resized the central panel, which the resize-detector treated as an
                // INTERACTION — bumping the view generation, tearing down the settle grid, and
                // re-rendering forever (whose counters then moved the label again). A reserved
                // empty `allocate_ui_with_layout` was NOT enough: an empty allocation's height in
                // a wrapped layout differs from a rendered label's (measured 34.0↔41.3 px bar
                // oscillation). So both states render the identical monospace label — the text
                // padded to the widest variant's length when a diagnostic binds, and the widest
                // variant drawn fully TRANSPARENT when none does. Same glyph count, same ⚠, same
                // row metrics: the bar's layout is invariant by construction.
                const SLOT: &str = "⚠ iter exhausted"; // the widest label variant
                ui.separator();
                let (text, color, detail) = match limit {
                    Some((label, detail, severe)) => (
                        format!("{label:<width$}", width = SLOT.chars().count()),
                        if severe {
                            egui::Color32::from_rgb(0xE0, 0x6C, 0x60)
                        } else {
                            egui::Color32::from_rgb(0xE0, 0xA0, 0x30)
                        },
                        Some(detail),
                    ),
                    None => (SLOT.to_string(), egui::Color32::TRANSPARENT, None),
                };
                // Extend, not wrap: a Label in a wrapped layout otherwise breaks its own text at
                // the panel edge — the ⚠ stranded at the end of one line, "iter capped" on the
                // next. Extend makes the label move (or overflow) as ONE unit, in both states.
                let r = ui.add(
                    egui::Label::new(egui::RichText::new(text).monospace().color(color))
                        .wrap_mode(egui::TextWrapMode::Extend),
                );
                if let Some(detail) = detail {
                    r.on_hover_text(detail);
                }
            });
        });
        // Expose the panel's height so the UI-test harness can detect the status bar wrapping to a
        // second line (or wavering between one and two at a fixed width — a repaint-storm smell).
        self.perf.status_bar_h = resp.response.rect.height();
    }

    /// Floating playback transport — scrubber, restart / back / pause / stop / forward, speed,
    /// render, loop, close — anchored over the top-centre of the VIEW while a script is loaded.
    ///
    /// It lives here rather than in the status bar because that bar is sized by its content: the
    /// centre coordinates alone gain and lose digits as the view moves, so anything to their right
    /// slides horizontally under the cursor while you are trying to click it. A fixed anchor over
    /// the view is the only stable position — and it is where a video player puts a transport.
    ///
    /// **Nothing in here may change width while it is on screen**, for the same reason. Three
    /// things did: the elapsed clock (`9:59` → `10:00`), the speed label (`1×` → `0.5×`), and the
    /// "waiting for detail" notice appearing mid-hold — which moved every button to its right just
    /// as the renderer got busy and you reached for one. All three are now width-stable: the clock
    /// is padded to the total's width, the speed label to its widest form, and the notice is always
    /// laid out and merely painted transparent when it does not apply.
    pub(crate) fn draw_playback_transport(&mut self, ctx: &egui::Context) {
        let Some(pb) = &self.playback else { return };
        let (name, cur_t, wall_t, total, paused, speed, looping, is_bench, held, finished) = (
            pb.name.clone(),
            pb.cur_t,
            pb.wall_t,
            pb.total,
            pb.paused,
            pb.speed,
            pb.loop_,
            pb.bench.is_some(),
            pb.paced_hold > 0.5,
            pb.finished,
        );
        // Transport intents, applied after the closure: the widgets borrow `self` immutably and
        // `stop_playback` needs it mutably.
        let (mut seek, mut toggle_pause, mut stop, mut cycle_speed, mut toggle_loop) =
            (None::<f64>, false, false, false, false);
        let (mut open_render, mut close, mut pick_script) = (false, false, false);
        let has_source = self.playback.as_ref().is_some_and(|p| p.source.is_some());
        // `available_rect` is what the panels left over — the fractal view, below the menu bar and
        // inside the right panel — so the transport centres on the VIEW, not on the window.
        let view = ctx.available_rect();
        egui::Area::new(egui::Id::new("playback_transport"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(view.center().x, view.top() + 10.0))
            .pivot(egui::Align2::CENTER_TOP)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .fill(egui::Color32::from_black_alpha(190))
                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_white_alpha(40)))
                    .corner_radius(6)
                    .show(ui, |ui| {
                        // NOTHING IN HERE MAY BE SIZED FROM THE AVAILABLE WIDTH. Inside an `Area`
                        // that width is unbounded, so anything derived from it (a scrub bar told to
                        // "fill the row", a right-aligned sub-layout) blows the player up to the
                        // width of the screen; egui then clamps the result against the screen edge
                        // and the whole thing slides sideways with its right-hand buttons off-view.
                        // Every part is therefore intrinsically sized AND width-stable — the name is
                        // elided to a fixed field, the clock and speed are padded, the scrub bar is
                        // a constant, and the renderer notice is always laid out — so the player is
                        // content-sized and that content never changes width.
                        let row = ui.horizontal(|ui| {
                            // The spinner animates only while the clock actually moves, so a
                            // paused tour looks paused. It keeps spinning while the PACER holds the
                            // clock, because the renderer is working then — that is the state that
                            // was mistaken for a hang.
                            // Fixed slot: the three states are a stopped glyph, a paused glyph and
                            // an animated spinner, and they do not share a natural width — laid out
                            // freely, the whole row would shift a pixel or two every time playback
                            // paused. Everything on this row is sized this way for that reason.
                            const GLYPH: [f32; 2] = [18.0, 18.0];
                            if finished {
                                ui.add_sized(GLYPH, egui::Label::new(egui::RichText::new(crate::icons::STOP).size(13.0)))
                                    .on_hover_text(format!("Finished — scrub back in, or {} to close", crate::icons::CLOSE));
                            } else if paused {
                                ui.add_sized(GLYPH, egui::Label::new(egui::RichText::new(crate::icons::PAUSE).size(13.0)));
                            } else {
                                ui.add_sized(GLYPH, egui::Spinner::new().size(12.0));
                            }
                            let mmss = |t: f64| {
                                let t = t.max(0.0) as u64;
                                format!("{}:{:02}", t / 60, t % 60)
                            };
                            // Pad elapsed to the total's width: `9:59 / 12:00` and `10:00 / 12:00`
                            // must occupy the same space, or the whole transport jumps left once a
                            // tour passes ten minutes.
                            let tot = mmss(total);
                            let tag = if is_bench { "benchmark" } else { "script" };
                            const NAME_MAX: usize = 16;
                            let short = if name.chars().count() > NAME_MAX {
                                let s: String = name.chars().take(NAME_MAX - 1).collect();
                                format!("{s}…")
                            } else {
                                name.clone()
                            };
                            ui.label(
                                egui::RichText::new(format!(
                                    "{short:<NAME_MAX$} {:>w$} / {tot}",
                                    mmss(cur_t),
                                    w = tot.len()
                                ))
                                .monospace(),
                            )
                            .on_hover_text(format!("Playing {tag} \"{name}\""));
                            ui.separator();
                            // Media glyphs, NOT monospace: they live in egui's bundled emoji font,
                            // and forcing the monospace family drops them to tofu boxes.
                            // Every button gets the SAME fixed size. `▶` and `⏸` are not the same
                            // width in that font, so a play/pause toggle would otherwise nudge each
                            // button after it — the same complaint as the notice, one glyph wide.
                            const BTN: [f32; 2] = [26.0, 20.0];
                            let btn = |ui: &mut egui::Ui, glyph: &str, tip: &str| -> bool {
                                ui.add_sized(
                                    BTN,
                                    egui::Button::new(egui::RichText::new(glyph).size(14.0)).small(),
                                )
                                .on_hover_text(tip)
                                .clicked()
                            };
                            if btn(ui, crate::icons::SKIP_BACK, "Restart from the beginning") {
                                seek = Some(0.0);
                            }
                            if btn(ui, crate::icons::REWIND, "Back 10 seconds") {
                                seek = Some((cur_t - 10.0).max(0.0));
                            }
                            if btn(ui, if paused { crate::icons::PLAY } else { crate::icons::PAUSE },
                                   if finished { "Play again from the beginning" }
                                   else if paused { "Resume" } else { "Pause" }) {
                                toggle_pause = true;
                            }
                            // Stop = rewind and park, as on any media player. It no longer tears
                            // the player down — that is ✖ — so the tour stays scrubable.
                            if btn(ui, crate::icons::STOP, "Stop and rewind to the start") {
                                stop = true;
                            }
                            if btn(ui, crate::icons::FORWARD, "Forward 10 seconds") {
                                seek = Some((cur_t + 10.0).min(total));
                            }
                            // Speed cycles rather than opening a menu: one control, one glance,
                            // and the label doubles as the readout.
                            if ui
                                .add_sized(
                                    [40.0, BTN[1]],
                                    egui::Button::new(
                                        // Right-padded to `0.5×`, the widest of the four, so
                                        // cycling the speed cannot resize its own button either.
                                        egui::RichText::new(format!("{:>3}×", crate::fmt_speed(speed)))
                                            .monospace(),
                                    )
                                    .small(),
                                )
                                .on_hover_text("Playback speed (click to cycle 0.5× / 1× / 2× / 4×)")
                                .clicked()
                            {
                                cycle_speed = true;
                            }
                            // Render: the tour on screen is a preview of a frame sequence, so
                            // the transport is where you would look for the button that makes one.
                            // Disabled for a tour with no file (the built-in benchmark), since the
                            // renderer takes a script PATH.
                            ui.add_enabled_ui(has_source, |ui| {
                                if ui
                                    .add_sized(
                                        BTN,
                                        egui::Button::new(egui::RichText::new(crate::icons::TOUR).size(14.0))
                                            .small(),
                                    )
                                    .on_hover_text(if has_source {
                                        "Render this script to a frame sequence…"
                                    } else {
                                        "The built-in benchmark has no script file to render"
                                    })
                                    .clicked()
                                {
                                    open_render = true;
                                }
                            });
                            // Change the script without leaving the player: same picker as
                            // Tools → Play script, starting in the current script's folder.
                            if btn(ui, crate::icons::OPEN, "Play a different script…") {
                                pick_script = true;
                            }
                            let loop_fill = if looping {
                                crate::theme::BRAND_ACCENT.gamma_multiply(0.35)
                            } else {
                                ui.visuals().widgets.inactive.bg_fill
                            };
                            if ui
                                .add_sized(
                                    BTN,
                                    egui::Button::new(egui::RichText::new(crate::icons::LOOP).size(14.0))
                                        .small()
                                        .fill(loop_fill),
                                )
                                .on_hover_text("Repeat the tour when it reaches the end")
                                .clicked()
                            {
                                toggle_loop = true;
                            }
                            // Separated from the transport buttons so it is never the one you hit by
                            // accident. Placed inline, NOT in a right-aligned sub-layout: a
                            // right-to-left layout claims the remaining available width, which
                            // inside an `Area` is unbounded — that pushed the button off-screen.
                            ui.separator();
                            if btn(ui, crate::icons::CLOSE, "Close the player and restore your own view settings") {
                                close = true;
                            }
                        });
                        // ---- scrub bar + renderer notice ----
                        // The notice shares this row rather than the button row: it is the widest
                        // thing here, and on the button row it would push the buttons around. It is
                        // ALWAYS laid out and merely painted transparent when it does not apply, so
                        // the player keeps one width whether or not the renderer is behind — the
                        // point being that it used to appear exactly when the view got heavy,
                        // shifting every button along just as you reached for one.
                        ui.horizontal(|ui| {
                            let notice = ui.colored_label(
                                if held {
                                    egui::Color32::from_rgb(0xE0, 0xA0, 0x30)
                                } else {
                                    egui::Color32::TRANSPARENT
                                },
                                egui::RichText::new("waiting for detail").monospace(),
                            );
                            if held {
                                notice.clone().on_hover_text(
                                    "The tour clock is paused while the renderer resolves this \
                                     view (reference build / iteration budget climbing). Playback \
                                     resumes by itself; see [playback] pace in the script.",
                                );
                            }
                            // Fill out the button row's width. Safe to measure BECAUSE the row
                            // above cannot contain this slider — sizing the slider from the width
                            // of a row it is itself inside is the feedback loop that made the
                            // player grow to the width of the screen.
                            let spacing = ui.spacing().item_spacing.x;
                            ui.spacing_mut().slider_width =
                                (row.response.rect.width() - notice.rect.width() - 2.0 * spacing)
                                    .clamp(120.0, 640.0);
                            let mut scrub = cur_t;
                            let drift = wall_t - cur_t;
                            let hover = if drift > 0.05 {
                                let d = drift.max(0.0) as u64;
                                format!(
                                    "Drag to scrub through the tour\nScript clock is {}:{:02} behind \
                                     real time (the pacer slowed the tour to let the renderer keep \
                                     up). The upper tick marks where wall-clock playback would be.",
                                    d / 60,
                                    d % 60
                                )
                            } else {
                                "Drag to scrub through the tour".to_string()
                            };
                            let sr = ui
                                .add(
                                    egui::Slider::new(&mut scrub, 0.0..=total.max(0.001))
                                        .show_value(false),
                                )
                                .on_hover_text(hover);
                            if sr.changed() {
                                seek = Some(scrub);
                            }
                            // Wall-clock DRIFT indicator (user request): a split tick. The slider's
                            // own handle already marks the actual script clock (cur_t, the lower
                            // half); this paints an UPPER tick at where a wall-clock-locked playback
                            // would be (wall_t ≥ cur_t, so always at or right of the handle). When
                            // they coincide the split closes; as the pacer holds the tour back the
                            // upper tick pulls ahead, with a faint bracket spanning the gap. Drawn
                            // only past a visible threshold so a sub-frame drift doesn't shimmer.
                            if drift > 0.05 && total > 0.0 {
                                let rect = sr.rect;
                                let r = (rect.height() * 0.5).min(8.0); // ≈ handle radius inset
                                let x0 = rect.left() + r;
                                let x1 = rect.right() - r;
                                let map = |t: f64| {
                                    x0 + ((t / total) as f32).clamp(0.0, 1.0) * (x1 - x0)
                                };
                                let (cx, gx) = (map(cur_t), map(wall_t));
                                let top = rect.top() + 1.0;
                                let mid = rect.center().y;
                                let amber = egui::Color32::from_rgb(0xE0, 0xA0, 0x30);
                                let faint = egui::Color32::from_rgba_unmultiplied(0xE0, 0xA0, 0x30, 90);
                                let p = ui.painter();
                                // Upper tick at the wall-clock ghost.
                                p.line_segment(
                                    [egui::pos2(gx, top), egui::pos2(gx, mid)],
                                    egui::Stroke::new(2.0_f32, amber),
                                );
                                // Bracket from the playhead across to the ghost.
                                p.line_segment(
                                    [egui::pos2(cx, top), egui::pos2(gx, top)],
                                    egui::Stroke::new(1.0_f32, faint),
                                );
                            }
                        });
                    });
            });

        if open_render {
            self.open_tour_render();
        }
        if pick_script {
            // Same picker as Tools → Play script (opens in the current script's folder). A chosen
            // file replaces this playback; a cancelled picker leaves it exactly as it was.
            self.load_script();
            return;
        }
        if close {
            self.stop_playback();
            return;
        }
        // Apply the transport. A seek moves the clock only: the camera follows on the next tick
        // through the normal sampling path, so scrubbing needs no special case anywhere else.
        if let Some(pb) = self.playback.as_mut() {
            if stop {
                pb.cur_t = 0.0;
                pb.wall_t = 0.0; // ghost re-anchors: no drift at a fresh stop
                pb.paused = true;
                pb.finished = false;
            }
            if let Some(t) = seek {
                pb.cur_t = t.clamp(0.0, pb.total);
                pb.wall_t = pb.cur_t; // a manual seek jumps real-time "to here" — drift resets
                // Scrubbing back into a finished tour makes it playable again; scrubbing to the
                // very end leaves it finished, so it doesn't immediately re-fire the toast.
                pb.finished = pb.cur_t >= pb.total;
            }
            if toggle_pause {
                // Play on a finished tour means "watch it again" — the only sensible reading, and
                // it saves reaching for ⏮ first.
                if pb.finished {
                    pb.cur_t = 0.0;
                    pb.wall_t = 0.0;
                    pb.finished = false;
                    pb.paused = false;
                } else {
                    pb.paused = !pb.paused;
                }
            }
            if toggle_loop {
                pb.loop_ = !pb.loop_;
            }
            if cycle_speed {
                pb.speed = match pb.speed {
                    s if s < 0.75 => 1.0,
                    s if s < 1.5 => 2.0,
                    s if s < 3.0 => 4.0,
                    _ => 0.5,
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::FractadyneApp;

    /// The measured limit regimes classify correctly, and ordinary states stay quiet.
    /// Every case here is a real view measured this week, not an invented one.
    #[test]
    fn limit_status_matches_measured_regimes() {
        const DEV: u32 = 7_452_444;
        const MAX: u32 = crate::MAX_ITER_LIMIT;
        // 2e95× spar: the reference is still partial AT the device cap → the red depth-limit wall.
        let s = FractadyneApp::limit_status(true, DEV, DEV, MAX, None, 0, false, None, false, false)
            .expect("partial-at-device-cap must warn");
        assert!(s.0.contains("depth limit") && s.2, "want severe depth limit, got {s:?}");
        // A partial BELOW the device cap is an ordinary motion/clamp state, not the wall — quiet.
        assert!(FractadyneApp::limit_status(true, 256_001, DEV, 500_000, None, 0, false, None, false, false).is_none());
        // The 6.5e94× black screen: probe climbed to the full appetite, frame stayed all-capped.
        let s = FractadyneApp::limit_status(false, 3_631_055, DEV, MAX, Some(1.0), MAX, true, None, true, true)
            .expect("exhausted must warn");
        assert!(s.0.contains("iter exhausted"), "got {s:?}");
        // Starved at the app maximum: most pixels capped AND escapes pressing the ceiling.
        let s = FractadyneApp::limit_status(
            false, 900_000, DEV, MAX, Some(0.60), MAX, true, Some(MAX as f64 * 0.97), false, false,
        )
        .expect("starved-at-ceiling must warn");
        assert!(s.0.contains("iter capped") && s.1.contains("already at the maximum"), "got {s:?}");

        // Quiet: an ordinary dive's motion partial (nothing refused)…
        assert!(FractadyneApp::limit_status(true, 256_001, DEV, 500_000, None, 0, false, None, false, false).is_none());
        // …the user's 6.9e94× view — a minibrot in frame caps its in-set core (measured 3 px of
        // 2304 = 0.13%) while escapes finish ~1.1M under a 10M budget: resolved, must stay quiet…
        assert!(FractadyneApp::limit_status(
            false, 2_848_721, DEV, MAX, Some(0.0013), MAX, true, Some(1_120_000.0), false, false
        )
        .is_none());
        // …a BIG minibrot (30% of frame in-set) with escapes far below the ceiling: still quiet —
        // in-set pixels always exhaust the budget and raising cannot help them…
        assert!(FractadyneApp::limit_status(
            false, 2_848_721, DEV, MAX, Some(0.30), MAX, true, Some(1_500_000.0), false, false
        )
        .is_none());
        // …mid-climb capping (the app can still raise the budget itself — transient)…
        assert!(FractadyneApp::limit_status(
            false, 900_000, DEV, 1_000_000, Some(0.5), 82_640 * 6, false, Some(490_000.0), false, false
        )
        .is_none());
        // …and a latched plateau that did NOT exhaust the appetite (partial-frame interior).
        assert!(FractadyneApp::limit_status(
            false, 900_000, DEV, MAX, Some(0.9), MAX, true, Some(MAX as f64), true, false
        )
        .is_none());
    }
}
