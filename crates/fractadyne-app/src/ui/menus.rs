//! Top menu bar / toolbar and the bottom status bar (REFACTOR-PLAN Phase 3, intra-crate UI split).
//! `impl FractadyneApp` blocks moved verbatim from `main.rs`.
use crate::*;

impl FractadyneApp {
    /// Top menu bar + toolbar (File / Fractal / View / Tools / Bookmarks / Locations / Help)
    /// plus the icon toolbar. Takes the `gpu` handle for the quick-export toolbar action.
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
                        if ui.button("📂  Open view or location…").clicked() {
                            self.open_view(ctx);
                            ui.close_menu();
                        }
                        if ui.button("🖼  Gallery…").clicked() {
                            self.gallery.open = true;
                            self.scan_gallery();
                            ui.close_menu();
                        }
                        if ui.button("💾  Export image…").clicked() {
                            self.export.open = true;
                            ui.close_menu();
                        }
                        if ui
                            .button("📷  Snapshot  (Ctrl+S)")
                            .on_hover_text("Quick-export to the last-used folder, no dialog")
                            .clicked()
                        {
                            if let Some((dev, q)) = gpu {
                                self.quick_export(ctx, dev.clone(), q.clone());
                            }
                            ui.close_menu();
                        }
                        if ui
                            .button("🔗  Share location…")
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
                        if ui
                            .button("♻  Reset application state…")
                            .on_hover_text(
                                "Delete all saved state (session, bookmarks, thumbnails) and \
                                 start fresh. Asks for confirmation first.",
                            )
                            .clicked()
                        {
                            self.dialogs.reset_confirm_open = true;
                            ui.close_menu();
                        }
                        if ui.button("✖  Quit").clicked() {
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
                        });
                    });
                    ui.menu_button("View", |ui| {
                        let now = ui.ctx().input(|i| i.time);
                        if ui
                            .button("🏠  Home view")
                            .on_hover_text("Zoom out to the full home view (animated)")
                            .clicked()
                        {
                            self.zoom_home(now);
                            ui.close_menu();
                        }
                        if ui
                            .button("🔄  Reset view")
                            .on_hover_text("Reset to the fractal's default view (instant)")
                            .clicked()
                        {
                            self.reset_view();
                            ui.close_menu();
                        }
                        ui.add_enabled_ui(self.nav.undo.len() > 1, |ui| {
                            if ui.button("Undo view  (Backspace)").clicked() {
                                self.undo_view();
                                ui.close_menu();
                            }
                        });
                        ui.add_enabled_ui(!self.nav.redo.is_empty(), |ui| {
                            if ui.button("Redo view  (Shift+Backspace)").clicked() {
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
                        ui.checkbox(&mut self.watermark, "Fd watermark")
                            .on_hover_text(
                                "Draw a discreet \"Fd\" brand mark in the lower-right corner of the \
                                 live view and exported images. On by default.",
                            );
                        if ui
                            .checkbox(&mut self.dual, "Dual view (Mandelbrot ↔ Julia)")
                            .clicked()
                            && self.dual
                        {
                            self.julia_viewport.reset();
                            self.julia_viewport.center_x =
                                fractadyne_core::BigFloat::from_f64(0.0, 64);
                            self.julia_viewport.center_y =
                                fractadyne_core::BigFloat::from_f64(0.0, 64);
                        }
                        ui.checkbox(&mut self.perf.enabled, "Performance panel");
                        ui.checkbox(&mut self.anim.show_orbits, "Show orbits")
                            .on_hover_text(
                                "Draw the iteration path of the point under the cursor.",
                            );
                        ui.add_enabled_ui(self.anim.show_orbits, |ui| {
                            ui.checkbox(&mut self.anim.orbit_normalize, "    Normalize (fit to view)")
                                .on_hover_text(
                                    "Fit the orbit to the whole view so it stays well-framed \
                                     at any zoom (instead of mapped through the viewport, \
                                     where it flies off-screen when deep).",
                                );
                            ui.checkbox(&mut self.anim.orbit_anim, "    Animate (racing dot)")
                                .on_hover_text(
                                    "Send a color-cycling dot racing out along the orbit.",
                                );
                            ui.add_enabled(
                                self.anim.orbit_anim,
                                egui::Slider::new(&mut self.anim.orbit_anim_speed, 1.0..=40.0)
                                    .text("Orbit speed")
                                    .suffix("/s"),
                            );
                        });
                        ui.separator();
                        ui.menu_button("Settings", |ui| {
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
                    });
                    ui.menu_button("Tools", |ui| {
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
                        if ui.button("Play script…").clicked() {
                            self.load_script();
                            ui.close_menu();
                        }
                        if ui
                            .button("Script to current view…")
                            .on_hover_text(
                                "Create a tour script that zooms from the full view down to the \
                                 current view — add a caption and set the dive duration.",
                            )
                            .clicked()
                        {
                            self.open_script_export();
                            ui.close_menu();
                        }
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
                                let ctx = ui.ctx().clone();
                                self.toggle_autopilot(&ctx);
                                ui.close_menu();
                            }
                        });
                        if self.bench_report.is_some()
                            && ui.button("Show last benchmark").clicked()
                        {
                            self.dialogs.bench_open = true;
                            ui.close_menu();
                        }
                        ui.add_enabled_ui(self.playback.is_some(), |ui| {
                            if ui.button("Stop playback").clicked() {
                                self.stop_playback();
                                ui.close_menu();
                            }
                        });
                    });
                    ui.menu_button("Bookmarks", |ui| {
                        if ui.button("★  Add current view").clicked() {
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
                                    self.bookmark_thumb_texture(ctx, &id)
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
                    });
                    ui.menu_button("Locations", |ui| {
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
                        });
                        ui.separator();
                        for (name, cx, cy, mag) in FAMOUS {
                            if ui.button(*name).clicked() {
                                self.goto_location(cx, cy, *mag, name, &ctx);
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        if ui
                            .button("🎲  Random location")
                            .on_hover_text("Jump to a random detail-rich boundary point")
                            .clicked()
                        {
                            self.random_location(&ctx);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui
                            .button("Import .kfr…")
                            .on_hover_text("Load a Kalles Fraktaler location file")
                            .clicked()
                        {
                            self.import_kfr(&ctx);
                            ui.close_menu();
                        }
                    });
                    ui.menu_button("Help", |ui| {
                        if ui.button("Help & reference…  (F1)").clicked() {
                            self.dialogs.help_open = true;
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
                if ui.button("📂").on_hover_text("Open a view or location (PNG / EXR, .fdn, .kfr)").clicked() {
                    self.open_view(ctx);
                }
                if ui.button("🖼").on_hover_text("Gallery").clicked() {
                    self.gallery.open = true;
                    self.scan_gallery();
                }
                if ui.button("💾").on_hover_text("Export image…").clicked() {
                    self.export.open = true;
                }
                if ui
                    .button("📷")
                    .on_hover_text("Snapshot — quick export to the last folder (Ctrl+S)")
                    .clicked()
                {
                    if let Some((dev, q)) = gpu {
                        self.quick_export(ctx, dev.clone(), q.clone());
                    }
                }
                ui.separator();
                // ── Navigation / location: zoom, reset/home, bookmark ─────────────────
                if ui.button("🔍+").on_hover_text("Zoom in").clicked() {
                    self.zoom_center(0.5);
                }
                if ui.button("🔍−").on_hover_text("Zoom out").clicked() {
                    self.zoom_center(2.0);
                }
                // Click-to-zoom tool (single view): arm left-click = dive into the point,
                // right-click = back out; drag still pans. Factor set in Settings ▸ Navigation.
                ui.add_enabled_ui(!self.dual, |ui| {
                    if ui
                        .selectable_label(self.click_zoom, "🎯")
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
                        .selectable_label(running, "🛸")
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
                if ui.button("🔄").on_hover_text("Reset view (instant)").clicked() {
                    self.reset_view();
                }
                if ui
                    .button("🏠")
                    .on_hover_text("Zoom out to the home view (animated)")
                    .clicked()
                {
                    let now = ctx.input(|i| i.time);
                    self.zoom_home(now);
                }
                if ui
                    .button("★")
                    .on_hover_text("Bookmark this view")
                    .clicked()
                {
                    self.add_bookmark("");
                }
                ui.separator();
                // ── Appearance / display ──────────────────────────────────────────────
                if ui
                    .button("🎨")
                    .on_hover_text(format!(
                        "Next palette ({})",
                        fractadyne_color::PRESETS[self.coloring.palette_idx].name
                    ))
                    .clicked()
                {
                    self.coloring.palette_idx = (self.coloring.palette_idx + 1) % fractadyne_color::PRESETS.len();
                }
                if ui
                    .selectable_label(self.perf.enabled, "📊")
                    .on_hover_text("Performance panel")
                    .clicked()
                {
                    self.perf.enabled = !self.perf.enabled;
                }
                if ui
                    .selectable_label(self.fullscreen, "🖥")
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
    /// 1. reference refused AT the GPU buffer cap (2e95×: nothing deeper is computable),
    /// 2. reference clamped below the iteration budget after a declined extension (e21000),
    /// 3. pixels exhausting a budget the app cannot raise further, with escapes pressing the
    ///    ceiling (starvation — as opposed to ordinary in-set pixels, which always cap).
    #[allow(clippy::too_many_arguments)] // a pure classifier over independent state — a struct would just rename the args
    pub(crate) fn limit_status(
        partial: bool,
        orbit_len: u32,
        refused: u32,
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
        if refused >= dev_cap {
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
        if partial && refused > 0 && clamp_px < eff_iter {
            return Some((
                "⚠ ref clamped",
                format!(
                    "Pixels are limited to {} iterations (budget {}): the reference orbit does \
                     not escape within the length the app can safely use live — a {}-sample \
                     extension was built and declined (it would freeze the GPU present). The \
                     view may under-resolve; an export can push further.",
                    commas(&clamp_px.to_string()),
                    commas(&eff_iter.to_string()),
                    commas(&refused.to_string())
                ),
                false,
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

    pub(crate) fn draw_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let l2 = self.viewport.log2_magnification();
                ui.monospace(format!(
                    "center {}, {}",
                    fmt_coord_deep(&self.viewport.center_x, l2),
                    fmt_coord_deep(&self.viewport.center_y, l2),
                ));
                ui.separator();
                match self.pointer.pointer_complex {
                    Some((mx, my)) => {
                        ui.monospace(format!("cursor {}, {}", fmt_coord(mx), fmt_coord(my)))
                    }
                    None => ui.monospace("cursor —"),
                };
                ui.separator();
                if self.dual {
                    ui.monospace(format!(
                        "zoom  M {}×   J {}×",
                        fmt_zoom_log2(self.viewport.log2_magnification()),
                        fmt_zoom_log2(self.julia_viewport.log2_magnification()),
                    ));
                } else {
                    ui.monospace(format!("zoom {}×", fmt_zoom_log2(self.viewport.log2_magnification())));
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
                ui.monospace(format!("iter {}", commas(&eff_iter.to_string())));
                // Rendering-limit diagnostics: when a cap is genuinely binding, say so where the
                // user is already looking, instead of leaving a black/flat view unexplained (the
                // Misiurewicz-spar reports arrived as mystery screenshots precisely because the
                // app knew it was clamped and said nothing).
                let vc = &self.ref_cache[0];
                if let Some((label, detail, severe)) = Self::limit_status(
                    vc.partial,
                    vc.orbit_len,
                    vc.ref_ext_refused,
                    crate::render::orbit_len_cap(),
                    eff_iter,
                    self.perf.capped_frac[0],
                    self.perf.budget_measured[0],
                    self.perf.budget_maxed[0],
                    self.perf.norm_range[0].map(|(_, mx)| mx as f64),
                    self.perf.iter_plateau[0],
                    self.perf.iter_exhausted[0],
                ) {
                    ui.separator();
                    let color = if severe {
                        egui::Color32::from_rgb(0xE0, 0x6C, 0x60)
                    } else {
                        egui::Color32::from_rgb(0xE0, 0xA0, 0x30)
                    };
                    ui.colored_label(color, egui::RichText::new(label).monospace())
                        .on_hover_text(detail);
                }
                // Playback indicator. A spinner, because the honest failure mode here is not
                // knowing whether anything is happening: a deep hold can legitimately sit on one
                // frame for many seconds, and the pacer deliberately STOPS the tour clock while
                // the renderer catches up — so a frozen percentage is expected behaviour that
                // looks exactly like a hang. The spinner animates regardless (it proves the UI
                // thread is alive), the clock reads mm:ss / mm:ss, and a held clock says why.
                if let Some(pb) = &self.playback {
                    ui.separator();
                    ui.add(egui::Spinner::new().size(12.0));
                    let pct = if pb.total > 0.0 {
                        (pb.cur_t / pb.total * 100.0).clamp(0.0, 100.0)
                    } else {
                        100.0
                    };
                    let mmss = |t: f64| {
                        let t = t.max(0.0) as u64;
                        format!("{}:{:02}", t / 60, t % 60)
                    };
                    let tag = if pb.bench.is_some() { "benchmark" } else { "script" };
                    ui.monospace(format!(
                        "{} {tag} {} / {} ({pct:.0}%)",
                        pb.name,
                        mmss(pb.cur_t),
                        mmss(pb.total),
                    ));
                    if pb.paced_hold > 0.5 {
                        ui.colored_label(
                            egui::Color32::from_rgb(0xE0, 0xA0, 0x30),
                            egui::RichText::new("waiting for detail").monospace(),
                        )
                        .on_hover_text(
                            "The tour clock is paused while the renderer resolves this view \
                             (reference build / iteration budget climbing). Playback resumes by \
                             itself; see [playback] pace in the script.",
                        );
                    }
                }
            });
        });
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
        // 2e95× spar: refusal AT the device cap → the red depth-limit wall.
        let s = FractadyneApp::limit_status(true, 256_001, DEV + 1, DEV, MAX, None, 0, false, None, false, false)
            .expect("device-cap refusal must warn");
        assert!(s.0.contains("depth limit") && s.2, "want severe depth limit, got {s:?}");
        // e21000: extension declined below the device cap, pixels clamped under the budget.
        let s = FractadyneApp::limit_status(true, 256_001, 508_193, DEV, 500_000, None, 0, false, None, false, false)
            .expect("declined extension must warn");
        assert!(s.0.contains("ref clamped") && !s.2, "want amber ref clamp, got {s:?}");
        // The 6.5e94× black screen: probe climbed to the full appetite, frame stayed all-capped.
        let s = FractadyneApp::limit_status(false, 3_631_055, 0, DEV, MAX, Some(1.0), MAX, true, None, true, true)
            .expect("exhausted must warn");
        assert!(s.0.contains("iter exhausted"), "got {s:?}");
        // Starved at the app maximum: most pixels capped AND escapes pressing the ceiling.
        let s = FractadyneApp::limit_status(
            false, 900_000, 0, DEV, MAX, Some(0.60), MAX, true, Some(MAX as f64 * 0.97), false, false,
        )
        .expect("starved-at-ceiling must warn");
        assert!(s.0.contains("iter capped") && s.1.contains("already at the maximum"), "got {s:?}");

        // Quiet: an ordinary dive's motion partial (nothing refused)…
        assert!(FractadyneApp::limit_status(true, 256_001, 0, DEV, 500_000, None, 0, false, None, false, false).is_none());
        // …the user's 6.9e94× view — a minibrot in frame caps its in-set core (measured 3 px of
        // 2304 = 0.13%) while escapes finish ~1.1M under a 10M budget: resolved, must stay quiet…
        assert!(FractadyneApp::limit_status(
            false, 2_848_721, 0, DEV, MAX, Some(0.0013), MAX, true, Some(1_120_000.0), false, false
        )
        .is_none());
        // …a BIG minibrot (30% of frame in-set) with escapes far below the ceiling: still quiet —
        // in-set pixels always exhaust the budget and raising cannot help them…
        assert!(FractadyneApp::limit_status(
            false, 2_848_721, 0, DEV, MAX, Some(0.30), MAX, true, Some(1_500_000.0), false, false
        )
        .is_none());
        // …mid-climb capping (the app can still raise the budget itself — transient)…
        assert!(FractadyneApp::limit_status(
            false, 900_000, 0, DEV, 1_000_000, Some(0.5), 82_640 * 6, false, Some(490_000.0), false, false
        )
        .is_none());
        // …and a latched plateau that did NOT exhaust the appetite (partial-frame interior).
        assert!(FractadyneApp::limit_status(
            false, 900_000, 0, DEV, MAX, Some(0.9), MAX, true, Some(MAX as f64), true, false
        )
        .is_none());
    }
}
