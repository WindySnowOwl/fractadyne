//! The right-hand control panel (REFACTOR-PLAN Phase 3, intra-crate UI split). An
//! `impl FractadyneApp` block moved verbatim from `main.rs`.
use crate::*;

impl FractadyneApp {
    /// Right-side control panel (Coloring + Navigation sections) plus the reopen handle shown
    /// when it's hidden. No-op in fullscreen.
    pub(crate) fn draw_right_panel(&mut self, ctx: &egui::Context) {
        if !self.fullscreen && self.dialogs.right_panel_open {
        egui::SidePanel::right("coloring_panel")
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    // ▶ points toward the right edge the panel collapses to (it's docked right).
                    if ui.small_button("\u{23F5}").on_hover_text("Hide control panel").clicked() {
                        self.dialogs.right_panel_open = false;
                    }
                    ui.label(egui::RichText::new("Controls").strong());
                });
                ui.separator();
                // Per-fractal info: formula, background, reference link.
                let info = self.fractal.info();
                egui::CollapsingHeader::new(self.fractal.name())
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.monospace(info.formula);
                        ui.add_space(4.0);
                        ui.label(info.about);
                        ui.add_space(4.0);
                        ui.hyperlink_to("Reference \u{2197}", info.reference);
                    });
                ui.separator();

                egui::CollapsingHeader::new("Coloring").default_open(true).show(ui, |ui| {
                egui::ComboBox::from_label("Method")
                    .selected_text(self.coloring.color_method.label())
                    .show_ui(ui, |ui| {
                        for m in ColorMethod::ALL {
                            ui.selectable_value(&mut self.coloring.color_method, m, m.label());
                        }
                    })
                    .response
                    .on_hover_text(
                        "How escape data maps to color. Stripe / triangle-inequality / \
                         orbit-trap / decomposition reveal orbit structure; distance \
                         shades by proximity to the boundary.",
                    );
                if self.coloring.color_method == ColorMethod::Stripe {
                    ui.add(
                        egui::Slider::new(&mut self.coloring.stripe_freq, 1.0..=24.0)
                            .text("Stripe density")
                            .logarithmic(true),
                    );
                }
                if self.coloring.color_method == ColorMethod::OrbitTrap {
                    egui::ComboBox::from_label("Trap shape")
                        .selected_text(self.coloring.trap_type.label())
                        .show_ui(ui, |ui| {
                            for t in TrapType::ALL {
                                ui.selectable_value(&mut self.coloring.trap_type, t, t.label());
                            }
                        });
                }
                let pal_name = if self.coloring.use_binary {
                    "Binary (set)"
                } else if self.coloring.use_duotone {
                    "Duotone"
                } else if self.coloring.use_custom_palette {
                    "Custom"
                } else {
                    fractadyne_color::PRESETS[self.coloring.palette_idx].name
                };
                egui::ComboBox::from_label("Palette")
                    .selected_text(pal_name)
                    .show_ui(ui, |ui| {
                        let is_preset = !self.coloring.use_custom_palette && !self.coloring.use_duotone && !self.coloring.use_binary;
                        for (i, p) in fractadyne_color::PRESETS.iter().enumerate() {
                            if ui.selectable_label(is_preset && self.coloring.palette_idx == i, p.name).clicked() {
                                self.coloring.palette_idx = i;
                                self.coloring.use_custom_palette = false;
                                self.coloring.use_duotone = false;
                                self.coloring.use_binary = false;
                            }
                        }
                        if ui.selectable_label(self.coloring.use_custom_palette, "Custom ✎").clicked() {
                            if self.coloring.custom_palette.is_empty() {
                                self.coloring.custom_palette = self.preset_as_stops(self.coloring.palette_idx);
                            }
                            self.coloring.use_custom_palette = true;
                            self.coloring.use_duotone = false;
                            self.coloring.use_binary = false;
                        }
                        if ui.selectable_label(self.coloring.use_duotone, "Duotone").clicked() {
                            self.coloring.use_duotone = true;
                            self.coloring.use_custom_palette = false;
                            self.coloring.use_binary = false;
                        }
                        if ui
                            .selectable_label(self.coloring.use_binary, "Binary (set)")
                            .on_hover_text("Flat two-color: in-set vs out-of-set, no gradient.")
                            .clicked()
                        {
                            self.coloring.use_binary = true;
                            self.coloring.use_custom_palette = false;
                            self.coloring.use_duotone = false;
                        }
                    });
                if self.coloring.use_duotone || self.coloring.use_binary {
                    // Two shared colors. (Binary: interior/exterior; duotone: shadow/highlight.)
                    let (lo_lbl, hi_lbl) = if self.coloring.use_binary {
                        ("In-set", "Out-of-set")
                    } else {
                        ("Shadow", "Highlight")
                    };
                    ui.horizontal(|ui| {
                        ui.color_edit_button_rgb(&mut self.coloring.duotone_lo);
                        ui.label(lo_lbl);
                        ui.color_edit_button_rgb(&mut self.coloring.duotone_hi);
                        ui.label(hi_lbl);
                    });
                } else if ui.button("Edit gradient…").clicked() {
                    if self.coloring.custom_palette.is_empty() {
                        self.coloring.custom_palette = self.preset_as_stops(self.coloring.palette_idx);
                    }
                    self.coloring.use_custom_palette = true;
                    self.coloring.palette_editor_open = true;
                }
                ui.add(egui::Slider::new(&mut self.coloring.cycle, 0.0..=1.0).text("Cycle"));
                ui.add(egui::Slider::new(&mut self.coloring.offset, 0.0..=1.0).text("Offset"));
                egui::ComboBox::from_label("Animate")
                    .selected_text(self.anim.palette_anim.name())
                    .show_ui(ui, |ui| {
                        for m in PaletteAnim::ALL {
                            ui.selectable_value(&mut self.anim.palette_anim, m, m.name());
                        }
                    });
                ui.add_enabled(
                    self.anim.palette_anim != PaletteAnim::Off,
                    egui::Slider::new(&mut self.anim.palette_anim_speed, 0.01..=2.0)
                        .text("Speed")
                        .suffix("/s")
                        .logarithmic(true),
                )
                .on_hover_text(
                    "Cycle speed: color-offset cycles/sec, or (Random) gradient \
                     changes/sec.",
                );
                if self.anim.palette_anim == PaletteAnim::Random && ui.button("Shuffle gradient").clicked()
                {
                    self.anim.random_palette.reshuffle();
                }
                ui.separator();
                ui.checkbox(&mut self.effects.light, "3D relief lighting")
                    .on_hover_text(
                        "Shade the surface using the distance-estimate slope — an \
                         embossed, lit look. (Holomorphic families: Mandelbrot / \
                         Multibrot.)",
                    );
                ui.add_enabled_ui(self.effects.light, |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.effects.light_angle, 0.0..=std::f32::consts::TAU)
                            .text("Light angle")
                            .suffix(" rad"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.effects.light_height, 0.2..=4.0)
                            .text("Relief")
                            .logarithmic(true),
                    )
                    .on_hover_text("Lower = sharper relief; higher = softer/flatter.");
                    ui.checkbox(&mut self.effects.light_anim, "Rotate light")
                        .on_hover_text("Spin the light direction over time (uses the Speed slider).");
                });
                ui.checkbox(&mut self.effects.de, "Distance glow")
                    .on_hover_text(
                        "Bright distance-estimate contour bands that densify into glowing \
                         filaments near the boundary. (Holomorphic families.)",
                    );
                ui.add_enabled_ui(self.effects.de, |ui| {
                    ui.add(egui::Slider::new(&mut self.effects.de_strength, 0.0..=1.0).text("Glow"));
                    ui.add(
                        egui::Slider::new(&mut self.effects.de_width, 0.15..=4.0)
                            .text("Band width")
                            .logarithmic(true),
                    )
                    .on_hover_text("Spacing of the distance contours (octaves per band).");
                    ui.checkbox(&mut self.effects.de_anim, "Animate glow")
                        .on_hover_text("Flow the glow bands over time (uses the Speed slider).");
                });
                ui.separator();
                ui.checkbox(&mut self.render_cfg.auto_iter, "Auto-scale iterations with zoom");
                let label = if self.render_cfg.auto_iter { "Iterations (base)" } else { "Iterations" };
                ui.add(
                    egui::Slider::new(&mut self.render_cfg.max_iter, 64..=500_000)
                        .logarithmic(true)
                        .text(label),
                )
                .on_hover_text(
                    "Base iteration count. With Auto-scale on, the effective count climbs \
                     with zoom depth. While you're moving, the preview caps iterations low \
                     (50,000) for responsiveness, then sharpens to the full count when the \
                     view settles — so deep edges look smooth during motion and resolve when \
                     you stop.",
                );
                // Detail note: the coarse count only applies while moving. If the view is
                // settled and still resolution-limited (huge window at extreme depth), an
                // export renders at full resolution — otherwise the settled preview already
                // matches it. Show the current effective (settled) count so the user knows
                // the true detail level once motion stops.
                let log2mag = self.viewport.log2_magnification();
                let want_iter = if self.render_cfg.auto_iter {
                    self.viewport.recommended_max_iter(self.render_cfg.max_iter)
                } else {
                    self.render_cfg.max_iter
                };
                let settled_iter = want_iter.min(500_000).min(zoom_iter_cap(log2mag).max(256));
                let px = (self.viewport.width_px * self.viewport.height_px).max(1.0) as u64;
                let res_limited = px.saturating_mul(settled_iter.max(1) as u64)
                    > self.effective_work_budget().saturating_mul(6);
                if res_limited {
                    let accent = theme::ui_accent(ui.ctx());
                    ui.label(
                        egui::RichText::new(format!(
                            "⚠ At this depth the settled preview renders {} iterations at \
                             reduced resolution to stay responsive. Export for full-resolution \
                             detail.",
                            commas(&settled_iter.to_string()),
                        ))
                        .small()
                        .color(accent),
                    );
                }
                ui.separator();
                egui::ComboBox::from_label("Anti-alias")
                    .selected_text(match self.render_cfg.aa {
                        1 => "Off",
                        2 => "2×",
                        3 => "3×",
                        4 => "4×",
                        _ => "8×",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.render_cfg.aa, 1, "Off");
                        ui.selectable_value(&mut self.render_cfg.aa, 2, "2×");
                        ui.selectable_value(&mut self.render_cfg.aa, 3, "3×");
                        ui.selectable_value(&mut self.render_cfg.aa, 4, "4×");
                        ui.selectable_value(&mut self.render_cfg.aa, 8, "8×");
                    })
                    .response
                    .on_hover_text(
                        "Supersampling for still images (applied when the view settles). \
                         Higher tames the fine exterior 'dust' at the cost of render time.",
                    );

                });
                egui::CollapsingHeader::new("Navigation").default_open(true).show(ui, |ui| {
                ui.add(
                    egui::Slider::new(&mut self.render_cfg.zoom_rate, 0.25..=4.0)
                        .text("Zoom speed")
                        .suffix("×")
                        .logarithmic(true),
                )
                .on_hover_text("Speed of hold-Space continuous zoom (1× ≈ 2× per 1.5 s).");

                // Auto-zoom (autopilot) dive limit, edited in decimal orders (1eN×) but stored as log2.
                let mut dive_log10 = self.autopilot.dive_log2 / std::f64::consts::LOG2_10;
                if ui
                    .add(
                        egui::Slider::new(&mut dive_log10, 30.0..=5000.0)
                            .text("Auto-zoom dive limit")
                            .logarithmic(true)
                            .custom_formatter(|n, _| format!("1e{n:.0}×")),
                    )
                    .on_hover_text(
                        "Depth where auto-zoom (A key) stops. Up to ~1e271× it glides smoothly; \
                         deeper, it switches to a choppy stepped dive to reach extreme depth quickly.",
                    )
                    .changed()
                {
                    self.autopilot.dive_log2 = dive_log10 * std::f64::consts::LOG2_10;
                }

                // Live-render work budget: detail-vs-speed for the deep-zoom preview.
                ui.add(
                    egui::Slider::new(&mut self.render_cfg.work_budget_scale, 0.25..=8.0)
                        .text("Live render budget")
                        .suffix("×")
                        .logarithmic(true),
                )
                .on_hover_text(
                    "Detail vs. speed for the live view at deep zoom. Higher renders at fuller \
                     resolution (crisper — less of the \"soft\" upscaled look) but lowers frame-rate, \
                     and very high values risk a brief GPU stall on the heaviest frames. Exports are \
                     always full resolution regardless of this.",
                );

                });
                // Performance section, docked at the bottom of this same panel
                // (toggle via the Perf button or the View menu).
                if self.perf.enabled {
                    egui::CollapsingHeader::new("Performance")
                        .default_open(true)
                        .show(ui, |ui| {
                            self.perf_section(ui);
                        });
                }
            });
        }

        // Reopen handle when the control panel is hidden (and not fullscreen).
        if !self.fullscreen && !self.dialogs.right_panel_open {
            egui::Area::new(egui::Id::new("panel_reopen"))
                .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 36.0))
                .show(ctx, |ui| {
                    if ui.button("\u{2630}").on_hover_text("Show control panel").clicked() {
                        self.dialogs.right_panel_open = true;
                    }
                });
        }
    }
}
