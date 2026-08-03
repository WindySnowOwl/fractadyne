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
                        if ui.button("📂  Open view…").clicked() {
                            self.open_view();
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
                        if ui.button("Go to location…").clicked() {
                            self.open_goto();
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
                        ui.add_enabled_ui(matches!(self.fractal.formula_id(), 0..=3), |ui| {
                            if ui
                                .button("Find minibrot center  (M)")
                                .on_hover_text(
                                    "Newton-snap the view center to the nearby minibrot's \
                                     exact center and report its period.",
                                )
                                .clicked()
                            {
                                let ctx = ui.ctx().clone();
                                self.find_minibrot(&ctx);
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
                                let ctx = ui.ctx().clone();
                                self.toggle_autopilot(&ctx);
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
                        ui.checkbox(&mut self.render_cfg.series_approx, "Series approximation")
                            .on_hover_text(
                                "Speed up deep renders by seeding the perturbation from a \
                                 polynomial and skipping early iterations. Applies where BLA \
                                 isn't available (df32 depths, Multibrot, BLA off) — with BLA \
                                 active the same skip comes from the BLA tree, so the costly \
                                 series pass is skipped. Identical output; turn off to compare.",
                            );
                        ui.checkbox(&mut self.render_cfg.glitch_correct, "Glitch correction (export)")
                            .on_hover_text(
                                "Multi-reference glitch correction for exported images: detects \
                                 perturbation glitches and re-renders those pixels against extra \
                                 references until clean. On by default. Applies to exports up to \
                                 ~32 MP / the GPU texture limit (non-aux coloring); larger images \
                                 and the live view fall back to the plain path.",
                            );
                        ui.checkbox(&mut self.render_cfg.use_bla, "BLA acceleration (deep zoom)")
                            .on_hover_text(
                                "Bilinear approximation: skip iterations throughout the orbit at \
                                 extreme depth (floatexp Mandelbrot, ≥1e28×) — ~5× faster GPU \
                                 render, identical output (verified by the self-test). On by \
                                 default; turn off to compare or if you hit an artifact.",
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
                        if self.bench_report.is_some()
                            && ui.button("Show last benchmark").clicked()
                        {
                            self.dialogs.bench_open = true;
                            ui.close_menu();
                        }
                        ui.add_enabled_ui(self.playback.is_some(), |ui| {
                            if ui.button("Stop playback").clicked() {
                                self.playback = None;
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
                            // Click to jump (most recent first).
                            let mut jump: Option<usize> = None;
                            for (i, b) in self.bookmarks.iter().enumerate().rev().take(12) {
                                if ui.button(&b.name).clicked() {
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
                if ui.button("📂").on_hover_text("Open view…").clicked() {
                    self.open_view();
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
                if let Some(pb) = &self.playback {
                    let elapsed = pb.t0.map_or(0.0, |t0| ctx.input(|i| i.time) - t0);
                    let pct = if pb.total > 0.0 {
                        (elapsed / pb.total * 100.0).clamp(0.0, 100.0)
                    } else {
                        100.0
                    };
                    ui.separator();
                    let tag = if pb.bench.is_some() { "benchmark" } else { "script" };
                    ui.monospace(format!("▶ {} {tag} {pct:.0}%", pb.name));
                }
            });
        });
    }
}
