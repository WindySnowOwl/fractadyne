//! The right-hand control panel (REFACTOR-PLAN Phase 3, intra-crate UI split). An
//! `impl FractadyneApp` block moved verbatim from `main.rs`.
use crate::*;

/// Width of the control panel's label column. One value for every row, so the controls share a
/// left edge instead of starting wherever the preceding label happened to end.
const PANEL_LABEL_W: f32 = 104.0;

/// One labelled control row: label on the LEFT, control to its right.
///
/// egui's own idiom appends the label - `Slider::text` and `ComboBox::from_label` both put it
/// AFTER the widget - which reads as a caption and leaves a column of ragged text down the right
/// of the panel. Label-first is the desktop convention for a settings panel, and a fixed-width
/// label column aligns the controls with each other as a side effect.
///
/// Checkboxes deliberately do NOT use this: a trailing label is the convention for a checkbox,
/// and the panel's seventeen already read correctly.
fn labelled<R>(ui: &mut egui::Ui, label: &str, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.horizontal(|ui| {
        // PAD to the column rather than laying the label out inside a fixed-size child: a
        // child `Ui` shrinks to its content, so the controls still began wherever each label
        // happened to end - label-first but just as ragged, which was half the point.
        // A label wider than the column simply pushes its control right; that is preferable
        // to truncating a name the user needs to read.
        let used = ui.label(label).rect.width();
        if used < PANEL_LABEL_W {
            ui.add_space(PANEL_LABEL_W - used);
        }
        add(ui)
    })
    .inner
}

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
                // Scroll the sections when they don't fit the window height (header stays pinned).
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {

                egui::CollapsingHeader::new("Navigate").default_open(true).show(ui, |ui| {
                labelled(ui, "Zoom speed", |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.render_cfg.zoom_rate, 0.25..=4.0)
                            .suffix("×")
                            .logarithmic(true),
                    )
                })
                .on_hover_text("Speed of hold-Space continuous zoom (1× ≈ 2× per 1.5 s).");

                // Click-to-zoom tool: arm a click to dive into the point by a fixed factor. Off by
                // default; drag still pans and Shift/right-drag still box-zoom. Single view only.
                ui.checkbox(&mut self.click_zoom, "Click to zoom")
                    .on_hover_text(
                        "When on, a left-click in the view dives in by the factor below \
                         (right-click backs out), recentered on the clicked point. Drag still pans; \
                         Shift+drag / right-drag still box-zoom. Backspace undoes a click. \
                         Single view only.",
                    );
                ui.add_enabled_ui(self.click_zoom, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("    Factor");
                        for f in [2.0_f32, 4.0, 10.0, 50.0, 100.0] {
                            ui.selectable_value(
                                &mut self.render_cfg.click_zoom_factor,
                                f,
                                format!("{f:.0}×"),
                            );
                        }
                    });
                });


                });
                egui::CollapsingHeader::new("Coloring").default_open(true).show(ui, |ui| {
                labelled(ui, "Method", |ui| {
                    egui::ComboBox::from_id_salt("panel_method")
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
                });
                if self.coloring.color_method == ColorMethod::Stripe {
                    labelled(ui, "Stripe density", |ui| {
                        ui.add(
                            egui::Slider::new(&mut self.coloring.stripe_freq, 1.0..=24.0)
                                .logarithmic(true),
                        )
                    });
                }
                if self.coloring.color_method == ColorMethod::OrbitTrap {
                    labelled(ui, "Trap shape", |ui| {
                        egui::ComboBox::from_id_salt("panel_trap_shape")
                            .selected_text(self.coloring.trap_type.label())
                            .show_ui(ui, |ui| {
                                for t in TrapType::ALL {
                                    ui.selectable_value(&mut self.coloring.trap_type, t, t.label());
                                }
                            });
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
                labelled(ui, "Palette", |ui| {
                    egui::ComboBox::from_id_salt("panel_palette")
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
                            if ui.selectable_label(self.coloring.use_custom_palette, format!("Custom {}", crate::icons::EDIT)).clicked() {
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
                labelled(ui, "Cycle", |ui| {
                    ui.add(egui::Slider::new(&mut self.coloring.cycle, 0.0..=1.0))
                });
                labelled(ui, "Offset", |ui| {
                    ui.add(egui::Slider::new(&mut self.coloring.offset, 0.0..=1.0))
                });
                ui.checkbox(&mut self.coloring.log_palette, "Log color scale")
                    .on_hover_text(
                        "Spread the palette by the logarithm of the escape value rather than \
                         linearly. Escape counts crowd towards the high end at depth, so a linear \
                         map spends most of the palette on a thin shell near the boundary and \
                         flattens everything else. Applies wherever normalization is active.",
                    );
                ui.checkbox(&mut self.coloring.normalize_live, "Normalize deep colors")
                    .on_hover_text(
                        "At extreme depth, escape counts span hundreds of thousands and a fixed \
                         Cycle wraps the palette thousands of times between neighboring pixels — a \
                         correct dense field reads as speckle noise. This remaps the palette to the \
                         view's measured escape range (Cycle then sets how many palette sweeps span \
                         it). Smooth method only; ordinary views are unaffected. Matches the \
                         --normalize export option.",
                    );
                labelled(ui, "Animate", |ui| {
                    egui::ComboBox::from_id_salt("panel_animate")
                        .selected_text(self.anim.palette_anim.name())
                        .show_ui(ui, |ui| {
                            for m in PaletteAnim::ALL {
                                ui.selectable_value(&mut self.anim.palette_anim, m, m.name());
                            }
                        });
                });
                labelled(ui, "Speed", |ui| {
                    ui.add_enabled(
                        self.anim.palette_anim != PaletteAnim::Off,
                        egui::Slider::new(&mut self.anim.palette_anim_speed, 0.01..=2.0)
                            .suffix("/s")
                            .logarithmic(true),
                    )
                })
                .on_hover_text(
                    "Cycle speed: color-offset cycles/sec, or (Random) gradient \
                     changes/sec.",
                );
                if self.anim.palette_anim == PaletteAnim::Random && ui.button("Shuffle gradient").clicked()
                {
                    self.anim.random_palette.reshuffle();
                }
                });
                egui::CollapsingHeader::new("Quality").default_open(true).show(ui, |ui| {
                ui.checkbox(&mut self.render_cfg.auto_iter, "Auto-scale iterations with zoom");
                let label = if self.render_cfg.auto_iter { "Iterations (base)" } else { "Iterations" };
                labelled(ui, label, |ui| {
                    ui.add(
                        egui::Slider::new(
                            &mut self.render_cfg.max_iter,
                            64..=crate::MAX_ITER_LIMIT,
                        )
                        .logarithmic(true),
                    )
                })
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
                let settled_iter =
                    want_iter.min(crate::MAX_ITER_LIMIT).min(zoom_iter_cap(log2mag).max(256));
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
                labelled(ui, "Anti-alias", |ui| {
                    egui::ComboBox::from_id_salt("panel_anti_alias")
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

                });
                egui::CollapsingHeader::new("Effects").default_open(false).show(ui, |ui| {
                ui.checkbox(&mut self.effects.light, "3D relief lighting")
                    .on_hover_text(
                        "Shade the surface using the distance-estimate slope — an \
                         embossed, lit look. (Holomorphic families: Mandelbrot / \
                         Multibrot.)",
                    );
                ui.add_enabled_ui(self.effects.light, |ui| {
                    labelled(ui, "Light angle", |ui| {
                        ui.add(
                            egui::Slider::new(&mut self.effects.light_angle, 0.0..=std::f32::consts::TAU)
                                .suffix(" rad"),
                        )
                    });
                    labelled(ui, "Relief", |ui| {
                        ui.add(
                            egui::Slider::new(&mut self.effects.light_height, 0.2..=4.0)
                                .logarithmic(true),
                        )
                    })
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
                    labelled(ui, "Glow", |ui| {
                        ui.add(egui::Slider::new(&mut self.effects.de_strength, 0.0..=1.0))
                    });
                    labelled(ui, "Band width", |ui| {
                        ui.add(
                            egui::Slider::new(&mut self.effects.de_width, 0.15..=4.0)
                                .logarithmic(true),
                        )
                    })
                    .on_hover_text("Spacing of the distance contours (octaves per band).");
                    ui.checkbox(&mut self.effects.de_anim, "Animate glow")
                        .on_hover_text("Flow the glow bands over time (uses the Speed slider).");
                });
                });
                // The orbit overlay's options lived as an indented block inside the View MENU
                // until 2026-08-13 (UI review: a panel's worth of controls in a dropdown). The
                // toggle is mirrored in Tools ▸ Orbit overlay.
                egui::CollapsingHeader::new("Overlays").default_open(false).show(ui, |ui| {
                ui.checkbox(&mut self.anim.show_orbits, "Orbit overlay")
                    .on_hover_text("Draw the iteration path of the point under the cursor.");
                ui.add_enabled_ui(self.anim.show_orbits, |ui| {
                    ui.checkbox(&mut self.anim.orbit_normalize, "Normalize (fit to view)")
                        .on_hover_text(
                            "Fit the orbit to the whole view so it stays well-framed at any zoom \
                             (instead of mapped through the viewport, where it flies off-screen \
                             when deep).",
                        );
                    ui.checkbox(&mut self.anim.orbit_anim, "Animate (racing dot)")
                        .on_hover_text("Send a color-cycling dot racing out along the orbit.");
                    labelled(ui, "Orbit speed", |ui| {
                        ui.add_enabled(
                            self.anim.orbit_anim,
                            egui::Slider::new(&mut self.anim.orbit_anim_speed, 1.0..=40.0)
                                .suffix("/s"),
                        )
                    });
                });
                ui.checkbox(&mut self.dialogs.minimap, "Minimap overview").on_hover_text(
                    "A small home-view overview with a \"you are here\" marker and the zoom \
                     depth. Click it to jump to a region.",
                );
                });
                // ADVANCED, closed by default. Everything a first-time user should not meet
                // while looking for how to zoom: the accelerators (whose names - BLA, series
                // approximation, glitch correction - mean nothing without the theory) and the
                // performance-tuning sliders that used to sit inside Navigation. "Live render
                // budget" and "Min motion resolution" are arguably MORE obscure than BLA: BLA can
                // be ignored safely, whereas a mis-set budget changes behaviour with no clue why.
                egui::CollapsingHeader::new("Advanced").default_open(false).show(ui, |ui| {
                ui.label(egui::RichText::new("Accelerators").weak().small());
                ui.checkbox(&mut self.render_cfg.use_bla, "BLA acceleration (deep zoom)")
                    .on_hover_text(
                        "Bilinear approximation: skip iterations throughout the orbit at extreme \
                         depth (floatexp Mandelbrot, ≥1e28×) — ~5× faster GPU render, identical \
                         output (verified by the self-test). On by default; turn off to compare \
                         or if you hit an artifact.",
                    );
                ui.checkbox(&mut self.render_cfg.series_approx, "Series approximation")
                    .on_hover_text(
                        "Seed the perturbation from a polynomial to skip early iterations, where \
                         BLA isn't active (df32 depths, Multibrot, BLA off). Identical output; \
                         turn off to compare.",
                    );
                ui.checkbox(&mut self.render_cfg.glitch_correct, "Glitch correction (export)")
                    .on_hover_text(
                        "Multi-reference glitch correction for exported images: detects \
                         perturbation glitches and re-renders those pixels against extra \
                         references until clean. On by default. Exports up to ~32 MP (non-aux \
                         coloring); larger images and the live view use the plain path.",
                    );
                ui.separator();
                ui.label(egui::RichText::new("Performance tuning").weak().small());
                // Auto-zoom (autopilot) dive limit, edited in decimal orders (1eN×) but stored as log2.
                let mut dive_log10 = self.autopilot.dive_log2 / std::f64::consts::LOG2_10;
                if labelled(ui, "Auto-zoom dive limit", |ui| {
                    ui.add(
                        egui::Slider::new(&mut dive_log10, 30.0..=5000.0)
                            .logarithmic(true)
                            .custom_formatter(|n, _| format!("1e{n:.0}×")),
                    )
                })
                    .on_hover_text(
                        "Depth where auto-zoom (A key) stops. Up to ~1e271× it glides smoothly; \
                         deeper, it switches to a choppy stepped dive to reach extreme depth quickly.",
                    )
                    .changed()
                {
                    self.autopilot.dive_log2 = dive_log10 * std::f64::consts::LOG2_10;
                }

                // Live-render work budget: detail-vs-speed for the deep-zoom preview.
                labelled(ui, "Live render budget", |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.render_cfg.work_budget_scale, 0.25..=8.0)
                            .suffix("×")
                            .logarithmic(true),
                    )
                })
                .on_hover_text(
                    "Detail vs. speed for the live view at deep zoom. Higher renders at fuller \
                     resolution (crisper — less of the \"soft\" upscaled look) but lowers frame-rate, \
                     and very high values risk a brief GPU stall on the heaviest frames. Exports are \
                     always full resolution regardless of this.",
                );

                // Motion sharpness floor: cap how pixelated a continuous deep zoom is allowed to get.
                labelled(ui, "Min motion resolution", |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.render_cfg.min_motion_res, 0.30..=1.0)
                            .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                            .custom_parser(|s| {
                                s.trim().trim_end_matches('%').parse::<f64>().ok().map(|v| {
                                    if v > 1.0 { v / 100.0 } else { v }
                                })
                            }),
                    )
                })
                .on_hover_text(
                    "The lowest resolution the live view may drop to while continuously zooming or \
                     panning at deep zoom. Higher stops a fast dive from getting blocky/pixelated \
                     (sharper) at the cost of frame-rate — at 100% deep-dive detail refreshes must \
                     render at full cost, which can make deep zooming visibly step/stutter. If a \
                     deep dive feels jerky, lower this. Default 30%; doesn't affect the settled \
                     image or exports.",
                );

                // KF-style stepping: hold the last detailed frame (geometrically tracked) through
                // motion instead of rendering coarse intermediate frames.
                ui.checkbox(&mut self.render_cfg.finish_sound, "Sound when a render finishes")
                    .on_hover_text(
                        "Play FRACTINT's completion tune when an export or tour render finishes: \
                         three rising notes (1047, 1109, 1175 Hz for 100 ms each), exactly as the \
                         DOS original encoded them — synthesized as a band-limited square and \
                         high-passed to sound like the little PC-speaker cone that played it.",
                    );
                ui.checkbox(&mut self.render_cfg.prefer_detail, "Prefer detail while zooming")
                    .on_hover_text(
                        "While zooming or panning, keep showing the last fully detailed frame — \
                         scaled and tracked to follow the motion — instead of re-rendering at \
                         reduced quality every frame. The view renders in full when the motion \
                         pauses. Off: motion renders live at reduced resolution (smoother, \
                         coarser). Shallow views (direct mode) always render live — they are \
                         cheap and sharp every frame either way.",
                    );
                });
                // Per-fractal info, LAST and closed by default: it is reference material, not a
                // control, and it was previously the panel's top section - the most valuable space in
                // the panel spent on something read once. Its header was also `self.fractal.name()`,
                // i.e. literally "Mandelbrot", which reads as a formula SELECTOR next to the toolbar
                // dropdown that actually is one.
                let info = self.fractal.info();
                egui::CollapsingHeader::new(format!("About {}", self.fractal.name()))
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.monospace(info.formula);
                        ui.add_space(4.0);
                        ui.label(info.about);
                        ui.add_space(4.0);
                        ui.hyperlink_to("Reference \u{2197}", info.reference);
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
