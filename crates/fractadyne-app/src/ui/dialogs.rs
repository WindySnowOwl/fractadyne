//! Modal dialog surfaces — `impl FractadyneApp` blocks moved verbatim out of `main.rs`
//! (REFACTOR-PLAN Phase 3, intra-crate UI split). Each `draw_*_dialog(ctx)` is a no-op
//! unless its `*_open` flag is set; the event loop calls them as a flat sequence.

use crate::*;

impl FractadyneApp {
    /// "Go to location" dialog — jump to a pasted center/zoom, or copy the current one to share.
    pub(crate) fn draw_goto_dialog(&mut self, ctx: &egui::Context) {
        if !self.goto.open {
            return;
        }
        let mut open = self.goto.open;
        let mut go = false;
        let mut copy = false;
        egui::Window::new("Go to location")
            .open(&mut open)
            .resizable(false)
            .default_width(420.0)
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("Center X").weak().small());
                ui.add(egui::TextEdit::singleline(&mut self.goto.x).desired_width(f32::INFINITY));
                ui.label(egui::RichText::new("Center Y").weak().small());
                ui.add(egui::TextEdit::singleline(&mut self.goto.y).desired_width(f32::INFINITY));
                ui.label(egui::RichText::new("Zoom (magnification)").weak().small());
                ui.add(egui::TextEdit::singleline(&mut self.goto.zoom).desired_width(220.0));
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
                    egui::RichText::new("Paste a center/zoom from someone else, or Copy to share.")
                        .weak()
                        .small(),
                );
            });
        if copy {
            ctx.copy_text(format!(
                "center_x={}\ncenter_y={}\nzoom={}",
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
                        .stroke(egui::Stroke::new(1.0, accent))
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
                    if ui.button("★ Add current view").clicked() {
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
                                    if ui.button("🗑").on_hover_text("Delete").clicked() {
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
            let meta = self.bookmarks[i].meta.clone();
            self.load_view_metadata(&meta);
        }
        if let Some(i) = delete {
            // Remove the thumbnail file + cached texture, then the bookmark.
            let id = self.bookmarks[i].thumb.clone();
            if !id.is_empty() {
                if let Some(p) = Self::bookmark_thumb_path(&id) {
                    let _ = std::fs::remove_file(p);
                }
                self.thumb_cache.remove(&id);
            }
            self.bookmarks.remove(i);
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
                ui.radio_value(&mut self.bench_cfg.standard, false, "Current settings")
                    .on_hover_text("Play the fixed deep-zoom tour into the live view using your current resolution and settings.");
                ui.radio_value(&mut self.bench_cfg.standard, true, "Standardized")
                    .on_hover_text("Pin resolution + all render settings so the score is comparable across machines.");
                ui.add_enabled_ui(self.bench_cfg.standard, |ui| {
                    ui.separator();
                    ui.label("Resolution");
                    egui::ComboBox::from_id_salt("bench_res")
                        .selected_text(self.bench_cfg.res.label())
                        .show_ui(ui, |ui| {
                            for r in BenchRes::ALL {
                                ui.selectable_value(&mut self.bench_cfg.res, r, r.label());
                            }
                        });
                    ui.add_space(4.0);
                    ui.label("Depth");
                    egui::ComboBox::from_id_salt("bench_depth")
                        .selected_text(self.bench_cfg.depth.label())
                        .show_ui(ui, |ui| {
                            for d in BenchDepth::ALL {
                                ui.selectable_value(&mut self.bench_cfg.depth, d, d.label());
                            }
                        })
                        .response
                        .on_hover_text("Ultra dives past f64's limit (1e28×), stressing the perturbation / series-approx / BLA deep-zoom path much harder.");
                    ui.add_space(4.0);
                    ui.checkbox(&mut self.bench_cfg.burnin, "Burn-in (repeat)")
                        .on_hover_text("Run the benchmark repeatedly to reveal stability and thermal throttling.");
                    ui.add_enabled_ui(self.bench_cfg.burnin, |ui| {
                        ui.add(egui::Slider::new(&mut self.bench_cfg.passes, 2..=200).text("passes"));
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
                if self.bench_cfg.standard {
                    let (w, h) = self.bench_cfg.res.dims();
                    ui.add_space(2.0);
                    ui.weak(format!(
                        "Renders offscreen at {w}×{h}, {}× SS, Mandelbrot/smooth, 60-frame dive to 1e{:.0}×.",
                        scripting::STD_AA,
                        self.bench_cfg.depth.zoom_log10(),
                    ));
                }
            });
        self.dialogs.bench_dialog_open = open;
        if run_now {
            self.dialogs.bench_dialog_open = false;
            if self.bench_cfg.standard {
                let passes = if self.bench_cfg.burnin { self.bench_cfg.passes } else { 1 };
                let run = self.begin_standard_bench(self.bench_cfg.res, passes, self.bench_cfg.depth);
                self.std_bench = Some(run);
                ctx.request_repaint();
            } else {
                self.start_benchmark();
            }
        }
    }

    /// Standardized-benchmark progress window (advances one dive-frame per event-loop tick).
    pub(crate) fn draw_bench_progress_dialog(&mut self, ctx: &egui::Context) {
        let Some((label, done, total, last_fps, (fdone, ftotal))) =
            self.std_bench.as_ref().map(|r| {
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
            if let Some(run) = self.std_bench.take() {
                let report = (!run.pass_fps.is_empty()).then(|| self.format_std_bench(&run));
                let snap = run.take_snapshot();
                self.restore_from_bench(snap);
                if let Some(r) = report {
                    self.bench_report = Some(r);
                    self.dialogs.bench_open = true;
                }
            }
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
        let mut bench_save: Option<std::io::Result<()>> = None;
        egui::Window::new("Benchmark results")
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                if let Some(r) = self.bench_report.clone() {
                    ui.monospace(&r);
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("Copy").clicked() {
                            ui.ctx().copy_text(r.clone());
                        }
                        if ui.button("Save…").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Text", &["txt"])
                                .set_file_name("fractadyne_benchmark.txt")
                                .save_file()
                            {
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
                egui::ComboBox::from_label("Width (px)")
                    .selected_text(format!("{}", self.export.width))
                    .show_ui(ui, |ui| {
                        for w in [1280u32, 1920, 2560, 3840, 5120, 7680] {
                            ui.selectable_value(&mut self.export.width, w, format!("{w}"));
                        }
                    });
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
