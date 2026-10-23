//! Top menu bar / toolbar and the bottom status bar (REFACTOR-PLAN Phase 3, intra-crate UI split).
//! `impl FractadyneApp` blocks moved verbatim from `main.rs`.
use crate::*;

/// A titled group of choices inside a menu: a dimmed, smaller heading with its options **indented
/// underneath it**.
///
/// ⭐⭐**A heading set in the same font, weight and colour as the rows below it is not a heading —
/// it is the first row.** File ▸ Settings had four of them ("Frame-rate cap", "UI scale", "Theme",
/// "Updates") sitting flush with their own options, so the only thing grouping "60 FPS" with
/// "Frame-rate cap" rather than with "80%" was the separator lines, and the eye had to reconstruct
/// the structure from gaps (user, 2026-09-06).
///
/// Two cues rather than one, because either alone is weak: the heading is **dimmed and small** so
/// it reads as a label rather than something clickable, and the options are **indented** so the
/// grouping is spatial and survives a theme where the dimming is subtle. ⚠The heading is
/// deliberately quieter than its options — in a menu the options are what you came to click.
///
/// ⚠`indent`'s vertical rule is turned off: at four sections it is four lines of chrome in a small
/// popup, and the indent already carries the grouping.
fn menu_section<R>(
    ui: &mut egui::Ui,
    title: &str,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    ui.label(egui::RichText::new(title).small().weak());
    ui.visuals_mut().indent_has_left_vline = false;
    ui.spacing_mut().indent = 14.0;
    ui.indent(title, add).inner
}

impl FractadyneApp {
    /// One row of the Color ▸ Palette submenu: a swatch of the gradient, then its name. Returns
    /// whether the row was clicked — either half of it.
    ///
    /// ⭐**A gradient row previews itself.** The rule the `.ugr` picker and the editor's Saved
    /// picker both arrived at independently: a name like "blatte10" says nothing about what the
    /// gradient looks like, and a user's library is full of names only they chose. The presets draw
    /// the same swatch so the submenu does not read as two different kinds of list.
    fn palette_menu_row(
        ui: &mut egui::Ui,
        g: &fractadyne_color::segment::Gradient,
        name: &str,
        selected: bool,
    ) -> bool {
        let mut hit = false;
        ui.horizontal(|ui| {
            let (rect, resp) =
                ui.allocate_exact_size(egui::vec2(48.0, 12.0), egui::Sense::click());
            crate::paint_gradient(
                &ui.painter_at(rect),
                rect,
                &g.bake(fractadyne_color::segment::LUT_SIZE),
            );
            hit = resp.clicked() || ui.selectable_label(selected, name).clicked();
        });
        hit
    }

    /// Top menu bar + toolbar (File / Fractal / View / Tools / Bookmarks / Locations / Help)
    /// plus the icon toolbar. Takes the `gpu` handle for the quick-export toolbar action.
    /// The bookmarks menu: add-current first, then Manage, then the recent bookmarks with
    /// thumbnails.
    ///
    /// Shared by Navigate ▸ Bookmarks and the toolbar star button. One function rather
    /// than two copies on purpose: they are the same menu, and a second copy is a second
    /// thing to forget when bookmarks gain a field.
    pub(crate) fn bookmarks_menu(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if ui.button(format!("{}  {}", crate::icons::BOOKMARK, tr!(menu_bookmarks_add))).clicked() {
            self.add_bookmark("");
            ui.close_menu();
        }
        if ui.button(tr!(menu_bookmarks_manage)).clicked() {
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
                self.bookmark_jump(i); // restores the view + arms heal-on-jump for a missing thumb
                ui.close_menu();
            }
        }
    }

    /// The "Show" group: which of the three pictures the selected formula is drawn as.
    ///
    /// Drawn identically in the Fractal menu and inside the formula dropdown, so the two cannot
    /// disagree — and so that someone who opens the picker LOOKING for Julia finds it there,
    /// beside the formula it belongs to, rather than as a checkbox somewhere else.
    ///
    /// ⭐The first option carries the FORMULA'S OWN NAME ("Mandelbrot set", "Burning Ship set").
    /// That is the whole teaching device: a new user sees the thing they came for, named, and
    /// sees the Julia set offered as its sibling rather than as a mode — and the third option
    /// shows them the relation instead of describing it.
    /// The family picker's list, drawn identically in the Fractal menu and the toolbar dropdown:
    /// each family that is not a power family on its own row, then one row per power family
    /// ("Burning Ship  3 4 5"), so fifteen families cost five rows. Returns the family picked.
    fn family_list(&self, ui: &mut egui::Ui) -> Option<FractalKind> {
        let mut picked = None;
        for kind in FractalKind::ALL.into_iter().filter(|k| k.power_family().is_none()) {
            if ui.selectable_label(self.fractal == kind, kind.name()).on_hover_text(kind.menu_hint()).clicked() {
                picked = Some(kind);
            }
        }
        ui.separator();
        ui.label(egui::RichText::new(tr!(menu_family_powers)).weak().small());
        egui::Grid::new(ui.next_auto_id()).num_columns(2).spacing([10.0, 2.0]).show(ui, |ui| {
            for (label, kinds) in FractalKind::POWER_GROUPS {
                let showing = kinds.contains(&self.fractal);
                ui.label(if showing { egui::RichText::new(label).strong() } else { egui::RichText::new(label) });
                ui.horizontal(|ui| {
                    for k in kinds {
                        let d = k.power_family().expect("a power family");
                        if ui.selectable_label(self.fractal == k, d.to_string()).on_hover_text(k.menu_hint()).clicked() {
                            picked = Some(k);
                        }
                    }
                });
                ui.end_row();
            }
        });
        // The automata: not escape-time formulas, but pictures of the same app (design/automata.md).
        ui.separator();
        ui.label(egui::RichText::new(tr!(menu_family_automata)).weak().small());
        for kind in FractalKind::AUTOMATA {
            if ui.selectable_label(self.fractal == kind, kind.name()).on_hover_text(kind.menu_hint()).clicked() {
                picked = Some(kind);
            }
        }
        // L-systems: turtle drawings from a grammar (design/lsystems.md).
        ui.separator();
        ui.label(egui::RichText::new(tr!(menu_family_lsystems)).weak().small());
        for kind in FractalKind::LSYSTEMS {
            if ui.selectable_label(self.fractal == kind, kind.name()).on_hover_text(kind.menu_hint()).clicked() {
                picked = Some(kind);
            }
        }
        picked
    }

    pub(crate) fn show_mode_group(&mut self, ui: &mut egui::Ui) {
        let can_julia = self.fractal.supports_julia();
        let name = self.fractal.name();
        let cur = self.show_mode();
        let mut pick: Option<crate::ShowMode> = None;

        ui.label(egui::RichText::new(tr!(menu_show)).weak().small());
        if ui
            .radio(cur == crate::ShowMode::Set, tr!(menu_show_set, name = name))
            .on_hover_text(tr!(menu_show_set_hover, name = name))
            .clicked()
        {
            pick = Some(crate::ShowMode::Set);
        }
        // Newton has no free parameter, so it has no Julia and no dual view. The options stay
        // VISIBLE and disabled rather than vanishing: a picker whose contents change shape as you
        // move through it is harder to learn than one where something is greyed with a reason.
        ui.add_enabled_ui(can_julia, |ui| {
            if ui
                .radio(cur == crate::ShowMode::Julia, tr!(menu_show_julia))
                .on_hover_text(if can_julia {
                    tr!(menu_show_julia_hover, name = name)
                } else {
                    tr!(menu_show_julia_none, name = name)
                })
                .clicked()
            {
                pick = Some(crate::ShowMode::Julia);
            }
            if ui
                .radio(cur == crate::ShowMode::Both, tr!(menu_show_both))
                .on_hover_text(if can_julia {
                    tr!(menu_show_both_hover, name = name)
                } else {
                    tr!(menu_show_both_none, name = name)
                })
                .clicked()
            {
                pick = Some(crate::ShowMode::Both);
            }
        });

        if let Some(m) = pick {
            self.set_show_mode(m);
            // A radio is a decision, unlike the checkboxes this replaced: close the menu (or the
            // dropdown) on it, the way picking a formula from the list above already does.
            ui.close_menu();
        }
    }

    pub(crate) fn draw_menu_bar(&mut self, ctx: &egui::Context) {
        let top = egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                    // The id egui keys this row's `menu_button` open-state (`BarState`) under —
                    // recorded so `close_menu_bar` can dismiss a hanging menu on view navigation.
                    self.menu_bar_id = Some(ui.id());
                    brand_wordmark(ui);
                    ui.separator();
                    ui.menu_button(tr!(menu_file), |ui| {
                        if ui.button(format!("{}  {}", crate::icons::OPEN, tr!(menu_file_open_view))).clicked() {
                            self.open_view(ctx);
                            ui.close_menu();
                        }
                        if ui.button(format!("{}  {}", crate::icons::GALLERY, tr!(menu_file_gallery))).clicked() {
                            self.gallery.open = true;
                            self.scan_gallery();
                            ui.close_menu();
                        }
                        if ui.button(format!("{}  {}", crate::icons::SAVE, tr!(menu_file_export))).clicked() {
                            self.export.open = true;
                            ui.close_menu();
                        }
                        if ui
                            .button(format!("{}  {}  (Ctrl+S)", crate::icons::SNAPSHOT, tr!(menu_file_snapshot)))
                            .on_hover_text(self.snapshot_hover_text())
                            .clicked()
                        {
                            self.snapshot(ctx);
                            ui.close_menu();
                        }
                        ui.separator();
                        // Life patterns (design/automata.md): opening one shows it as a universe.
                        if ui
                            .button(format!("{}  {}", crate::icons::IMPORT, tr!(menu_file_open_life)))
                            .on_hover_text(tr!(menu_file_open_life_hover))
                            .clicked()
                        {
                            self.life_open_file();
                            ui.close_menu();
                        }
                        if ui
                            .button(format!("{}  {}", crate::icons::PASTE, tr!(menu_file_life_text)))
                            .on_hover_text(tr!(menu_file_life_text_hover))
                            .clicked()
                        {
                            self.life.text_open = true;
                            ui.close_menu();
                        }
                        if self.fractal == FractalKind::Life
                            && ui
                                .button(format!("{}  {}", crate::icons::SAVE, tr!(menu_file_save_life)))
                                .on_hover_text(tr!(menu_file_save_life_hover))
                                .clicked()
                        {
                            self.life_save_file();
                            ui.close_menu();
                        }
                        ui.separator();
                        // L-systems (design/lsystems.md): a Fractint .l file opens directly.
                        if ui
                            .button(format!("{}  {}", crate::icons::IMPORT, tr!(menu_file_open_lsystem)))
                            .on_hover_text(tr!(menu_file_open_lsystem_hover))
                            .clicked()
                        {
                            self.lsystem_open_file();
                            ui.close_menu();
                        }
                        if self.fractal == FractalKind::LSystem
                            && ui
                                .button(format!("{}  {}", crate::icons::SAVE, tr!(menu_file_save_lsystem)))
                                .on_hover_text(tr!(menu_file_save_lsystem_hover))
                                .clicked()
                        {
                            self.lsystem_save_file();
                            ui.close_menu();
                        }
                        if self.fractal == FractalKind::LSystem
                            && ui
                                .button(format!("{}  {}", crate::icons::SAVE, tr!(menu_file_export_svg)))
                                .on_hover_text(tr!(menu_file_export_svg_hover))
                                .clicked()
                        {
                            self.lsystem_export_svg();
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui
                            .button(if self.farm_client.active() {
                                format!("{}  ({})", tr!(menu_file_render_client), tr!(menu_state_connected))
                            } else {
                                tr!(menu_file_render_client)
                            })
                            .on_hover_text(tr!(menu_file_render_client_hover))
                            .clicked()
                        {
                            self.open_farm_client();
                            ui.close_menu();
                        }
                        ui.separator();
                        // Settings live under File — the conventional home users reach for first
                        // (File → Preferences/Settings); they sat under View until 2026-08-13,
                        // where only display TOGGLES belong.
                        ui.menu_button(format!("{}  {}", crate::icons::SETTINGS, tr!(menu_file_settings)), |ui| {
                            menu_section(ui, &tr!(menu_settings_fps), |ui| {
                                for (label, val) in [
                                    (tr!(menu_settings_fps_uncapped), None),
                                    (tr!(menu_settings_fps_n, fps = 30), Some(30.0)),
                                    (tr!(menu_settings_fps_n, fps = 60), Some(60.0)),
                                    (tr!(menu_settings_fps_n, fps = 120), Some(120.0)),
                                ] {
                                    if ui.selectable_label(self.fps_cap == val, label).clicked() {
                                        self.fps_cap = val;
                                    }
                                }
                            });
                            ui.separator();
                            menu_section(ui, &tr!(menu_settings_ui_scale), |ui| {
                                for (pct, val) in [(80, 0.8_f32), (90, 0.9), (100, 1.0), (110, 1.1), (125, 1.25), (150, 1.5)] {
                                    let label = tr!(common_percent, value = pct);
                                    if ui
                                        .selectable_label((self.ui_scale - val).abs() < 0.01, label)
                                        .clicked()
                                    {
                                        self.ui_scale = val;
                                    }
                                }
                            });
                            ui.separator();
                            menu_section(ui, &tr!(menu_settings_theme), |ui| {
                                for m in [ThemeMode::Dark, ThemeMode::Light] {
                                    if ui.selectable_label(self.theme == m, m.label()).clicked() {
                                        self.theme = m;
                                        apply_theme(ui.ctx(), m);
                                    }
                                }
                            });
                            ui.separator();
                            menu_section(ui, &tr!(menu_settings_updates), |ui| {
                                ui.horizontal(|ui| {
                                    for t in crate::update::UpdateTrack::ALL {
                                        ui.selectable_value(&mut self.update_track, t, t.label())
                                            .on_hover_text(match t {
                                                crate::update::UpdateTrack::Stable => tr!(menu_settings_updates_stable_hover),
                                                crate::update::UpdateTrack::Beta => tr!(menu_settings_updates_beta_hover),
                                            });
                                    }
                                });
                                ui.checkbox(
                                    &mut self.update_check_on_launch,
                                    tr!(menu_settings_updates_on_launch),
                                )
                                .on_hover_text(tr!(menu_settings_updates_on_launch_hover));
                            });
                            ui.separator();
                            // What the camera button does. The first press asks; this is where the
                            // answer lives afterwards, so "don't ask again" is never a trap.
                            menu_section(ui, &format!("{} (Ctrl+S)", tr!(menu_settings_snapshot)), |ui| {
                                ui.horizontal(|ui| {
                                    for m in crate::SnapshotMode::ALL {
                                        ui.selectable_value(&mut self.snapshot_mode, m, m.label())
                                            .on_hover_text(match m {
                                                crate::SnapshotMode::Ask => tr!(menu_settings_snapshot_ask_hover),
                                                crate::SnapshotMode::Screen => tr!(menu_settings_snapshot_screen_hover),
                                                crate::SnapshotMode::Render => tr!(menu_settings_snapshot_render_hover),
                                            });
                                    }
                                });
                            });
                            ui.separator();
                            // Opens another surface, so no verb of its own and no icon (§8.1).
                            if ui
                                .button(tr!(menu_settings_refcache))
                                .on_hover_text(tr!(menu_settings_refcache_hover))
                                .clicked()
                            {
                                self.dialogs.orbit_cache_open = true;
                                ui.close_menu();
                            }
                        });
                        ui.separator();
                        if ui
                            .button(format!("{}  {}", crate::icons::RESET_APP, tr!(menu_file_reset)))
                            .on_hover_text(tr!(menu_file_reset_hover))
                            .clicked()
                        {
                            self.dialogs.reset_confirm_open = true;
                            ui.close_menu();
                        }
                        if ui.button(format!("{}  {}", crate::icons::QUIT, tr!(menu_file_quit))).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.menu_button(tr!(menu_fractal), |ui| {
                        // This list and the toolbar's dropdown are deliberately both here:
                        // the dropdown is the fast way to switch once you know what you
                        // want, and this is where you find that out. Hence the hints -
                        // without them the two really would be the same control twice.
                        if let Some(kind) = self.family_list(ui) {
                            self.set_fractal(kind);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui
                            .selectable_label(self.fractal == FractalKind::Custom, tr!(menu_fractal_custom))
                            .on_hover_text(FractalKind::Custom.menu_hint())
                            .clicked()
                        {
                            self.open_formula_dialog();
                            ui.close_menu();
                        }
                        if ui
                            .button(tr!(menu_fractal_library))
                            .on_hover_text(tr!(menu_fractal_library_hover))
                            .clicked()
                        {
                            self.formula_library.open = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        // Was two checkboxes ("Julia mode", "Dual view") of which only three
                        // combinations were legal — the fourth was suppressed by grey. One
                        // three-way choice says the same thing in one row less, and can be worded
                        // to explain what a Julia set IS. Same widget as the dropdown's.
                        self.show_mode_group(ui);
                    });
                    ui.menu_button(tr!(menu_view), |ui| {
                        let now = ui.ctx().input(|i| i.time);
                        // "Home view" vs "Reset view" were near-synonyms distinguishable only by
                        // tooltip (2026-08-13 UI review) — the labels now say what each does.
                        if ui
                            .button(format!("{}  {}", crate::icons::HOME, tr!(menu_view_zoom_home)))
                            .on_hover_text(tr!(menu_view_zoom_home_hover))
                            .clicked()
                        {
                            self.zoom_home(now);
                            ui.close_menu();
                        }
                        if ui
                            .button(format!("{}  {}", crate::icons::RESET_VIEW, tr!(menu_view_reset)))
                            .on_hover_text(tr!(menu_view_reset_hover))
                            .clicked()
                        {
                            self.reset_view();
                            ui.close_menu();
                        }
                        ui.add_enabled_ui(self.nav.undo.len() > 1, |ui| {
                            if ui.button(format!("{}  (Ctrl+Z / Backspace)", tr!(menu_view_undo))).clicked() {
                                self.undo_view();
                                ui.close_menu();
                            }
                        });
                        ui.add_enabled_ui(!self.nav.redo.is_empty(), |ui| {
                            if ui.button(format!("{}  (Ctrl+Y / Shift+Backspace)", tr!(menu_view_redo))).clicked() {
                                self.redo_view();
                                ui.close_menu();
                            }
                        });
                        ui.separator();
                        ui.checkbox(&mut self.dialogs.right_panel_open, tr!(menu_view_control_panel))
                            .on_hover_text(tr!(menu_view_control_panel_hover));
                        ui.checkbox(&mut self.dialogs.minimap, tr!(menu_view_minimap))
                            .on_hover_text(tr!(menu_view_minimap_hover));
                        ui.checkbox(&mut self.perf.enabled, tr!(menu_view_perf_panel));
                        ui.checkbox(&mut self.render_cfg.show_timestamp, tr!(menu_view_timestamp))
                            .on_hover_text(tr!(menu_view_timestamp_hover));
                        ui.checkbox(&mut self.render_cfg.show_zoom_target, tr!(menu_view_zoom_target))
                            .on_hover_text(tr!(menu_view_zoom_target_hover));
                        ui.menu_button(tr!(menu_view_live_panel), |ui| {
                            use crate::blinken::Size;
                            for (size, label, tip) in [
                                (Size::Off, tr!(menu_view_live_off), tr!(menu_view_live_off_hover)),
                                (Size::Strip, tr!(menu_view_live_strip), tr!(menu_view_live_strip_hover)),
                                (Size::Panel, tr!(menu_view_live_panel_size), tr!(menu_view_live_panel_size_hover)),
                                (Size::Full, tr!(menu_view_live_full), tr!(menu_view_live_full_hover)),
                            ] {
                                if ui.radio(self.blinken.size == size, label).on_hover_text(tip).clicked() {
                                    self.blinken.size = size;
                                }
                            }
                            ui.separator();
                            ui.add(egui::Slider::new(&mut self.blinken.opacity, 0.3..=1.0).text(tr!(menu_view_live_opacity)));
                        })
                        .response
                        .on_hover_text(tr!(menu_view_live_panel_hover));
                        ui.checkbox(&mut self.watermark, tr!(menu_view_watermark))
                            .on_hover_text(tr!(menu_view_watermark_hover));
                        // Dual view moved to Fractal (it selects WHAT renders, like Julia mode);
                        // the orbit overlay's toggle lives in Tools, with Normalize beside it
                        // and the animation controls in the control panel's Overlays section
                        // (2026-08-13 UI review: View mixed four unrelated concerns).
                    });
                    // Coloring is where users spend the most time, and until 2026-08-13 the menu
                    // bar said nothing about it (UI review). This mirrors the control panel's
                    // essentials; the sliders (cycle/offset/animation) stay in the panel, where
                    // continuous adjustment belongs.
                    ui.menu_button(tr!(menu_color), |ui| {
                        ui.menu_button(tr!(menu_color_method), |ui| {
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
                        ui.menu_button(tr!(menu_color_palette), |ui| {
                            let is_preset = !self.coloring.use_custom_palette
                                && !self.coloring.use_duotone
                                && !self.coloring.use_binary;
                            let live_saved = self.live_saved_gradient();
                            for (i, p) in fractadyne_color::PRESETS.iter().enumerate() {
                                let g = fractadyne_color::segment::Gradient::from_stops(
                                    p.name, p.stops,
                                );
                                if Self::palette_menu_row(
                                    ui,
                                    &g,
                                    p.name,
                                    is_preset && self.coloring.palette_idx == i,
                                ) {
                                    self.coloring.palette_idx = i;
                                    self.coloring.use_custom_palette = false;
                                    self.coloring.use_duotone = false;
                                    self.coloring.use_binary = false;
                                    ui.close_menu();
                                }
                            }
                            // ⭐**The user's own gradients belong beside the built-in ones.** A
                            // gradient saved in the editor used to be reachable only from inside
                            // the editor, so choosing one meant opening a modal to pick a palette
                            // while the four presets sat one click away in this menu. Saving it was
                            // already the act of saying "I want this again".
                            if !self.saved_gradients.is_empty() {
                                ui.separator();
                                let mut apply = None;
                                // ⚠A library has no size limit, so the list scrolls. egui clips a
                                // menu that does not fit the screen rather than scrolling it, and
                                // the entries it clips are simply gone.
                                egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                                    for (i, sg) in self.saved_gradients.iter().enumerate() {
                                        let g = crate::segments_to_gradient(&sg.name, &sg.segment);
                                        if Self::palette_menu_row(
                                            ui,
                                            &g,
                                            &sg.name,
                                            live_saved == Some(i),
                                        ) {
                                            apply = Some(i);
                                        }
                                    }
                                });
                                if let Some(i) = apply {
                                    self.apply_saved_gradient(i);
                                    ui.close_menu();
                                }
                            }
                            ui.separator();
                            // ⚠Ticked only when the custom palette is NOT one of the saved entries
                            // above — otherwise picking a saved gradient would put a check mark on
                            // two rows, since a saved gradient IS the custom palette. "Custom" here
                            // means the unsaved one.
                            if ui
                                .selectable_label(self.coloring.use_custom_palette && live_saved.is_none(), format!("{} {}", tr!(menu_color_custom), crate::icons::EDIT))
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
                                .selectable_label(self.coloring.use_duotone, tr!(menu_color_duotone))
                                .clicked()
                            {
                                self.coloring.use_duotone = true;
                                self.coloring.use_custom_palette = false;
                                self.coloring.use_binary = false;
                                ui.close_menu();
                            }
                            if ui
                                .selectable_label(self.coloring.use_binary, tr!(menu_color_binary))
                                .on_hover_text(tr!(menu_color_binary_hover))
                                .clicked()
                            {
                                self.coloring.use_binary = true;
                                self.coloring.use_custom_palette = false;
                                self.coloring.use_duotone = false;
                                ui.close_menu();
                            }
                        });
                        if ui.button(tr!(menu_color_edit_gradient)).clicked() {
                            if self.coloring.custom_palette.is_empty() {
                                self.coloring.custom_palette =
                                    self.preset_as_stops(self.coloring.palette_idx);
                            }
                            self.coloring.use_custom_palette = true;
                            self.coloring.palette_editor_open = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.checkbox(&mut self.coloring.normalize_live, tr!(menu_color_normalize))
                            .on_hover_text(tr!(menu_color_normalize_hover));
                        self.fit_range_checkbox(ui);
                        self.log_scale_checkbox(ui);
                        ui.label(
                            egui::RichText::new(tr!(menu_color_more))
                            .weak()
                            .small(),
                        );
                    });
                    ui.menu_button(tr!(menu_tools), |ui| {
                        let ctx = ui.ctx().clone();
                        // The mathematical tools lead: they are what distinguishes this app from
                        // every other deep-zoom explorer, and until 2026-08-13 they were filed
                        // under Locations / buried in a dialog tooltip (UI review).
                        ui.add_enabled_ui(self.fractal.caps().nucleus_finder, |ui| {
                            if ui
                                .button(format!("{}  (M)", tr!(menu_tools_find_minibrot)))
                                .on_hover_text(tr!(menu_tools_find_minibrot_hover))
                                .clicked()
                            {
                                self.find_minibrot(&ctx);
                                ui.close_menu();
                            }
                            if ui
                                .button(tr!(menu_tools_solver))
                                .on_hover_text(tr!(menu_tools_solver_hover))
                                .clicked()
                            {
                                self.open_goto();
                                ui.close_menu();
                            }
                        });
                        ui.add_enabled_ui(!self.dual, |ui| {
                            let label = if self.autopilot.active { tr!(menu_tools_autopilot_stop) } else { tr!(menu_tools_autopilot) };
                            if ui
                                .button(format!("{label}  (A)"))
                                .on_hover_text(tr!(menu_tools_autopilot_hover))
                                .clicked()
                            {
                                self.toggle_autopilot(&ctx);
                                ui.close_menu();
                            }
                            ui.horizontal(|ui| {
                                use crate::autopilot::AutopilotPriority as P;
                                ui.add_space(12.0);
                                ui.label(tr!(menu_tools_priority));
                                ui.radio_value(&mut self.autopilot.priority, P::Speed, tr!(menu_tools_priority_speed))
                                    .on_hover_text(tr!(menu_tools_priority_speed_hover));
                                ui.radio_value(&mut self.autopilot.priority, P::Quality, tr!(menu_tools_priority_quality))
                                    .on_hover_text(tr!(menu_tools_priority_quality_hover));
                            });
                            ui.horizontal(|ui| {
                                use crate::autopilot::AutopilotTarget as T;
                                ui.add_space(12.0);
                                ui.label(tr!(menu_tools_target));
                                ui.radio_value(&mut self.autopilot.target, T::Detail, tr!(menu_tools_target_detail))
                                    .on_hover_text(tr!(menu_tools_target_detail_hover));
                                ui.radio_value(&mut self.autopilot.target, T::Misiurewicz, tr!(menu_tools_target_misiurewicz))
                                    .on_hover_text(tr!(menu_tools_target_misiurewicz_hover));
                            });
                        });
                        ui.checkbox(&mut self.anim.show_orbits, tr!(menu_tools_orbits))
                            .on_hover_text(tr!(menu_tools_orbits_hover));
                        // Normalize sits here, unlike the rest of the overlay options, because
                        // it is a MODE rather than a continuous adjustment - without it the
                        // orbit flies off-screen at depth, so it is the one setting you reach
                        // for while looking at the picture. The animation toggle and its speed
                        // slider stay in the panel; a slider in a dropdown is what the
                        // 2026-08-13 review objected to.
                        ui.add_enabled_ui(self.anim.show_orbits, |ui| {
                            ui.indent("tools_orbit_opts", |ui| {
                                ui.checkbox(
                                    &mut self.anim.orbit_normalize,
                                    tr!(menu_tools_orbits_normalize),
                                )
                                .on_hover_text(tr!(menu_tools_orbits_normalize_hover));
                            });
                        });
                        ui.separator();
                        if ui
                            .button(tr!(menu_tools_play_tour))
                            .on_hover_text(tr!(menu_tools_play_tour_hover))
                            .clicked()
                        {
                            self.load_script();
                            ui.close_menu();
                        }
                        if ui
                            .button(tr!(menu_tools_tour_from_view))
                            .on_hover_text(tr!(menu_tools_tour_from_view_hover))
                            .clicked()
                        {
                            self.open_script_export();
                            ui.close_menu();
                        }
                        // Enabled while a render runs too: it is how to get back to its progress.
                        ui.add_enabled_ui(self.playback.is_some() || self.tour_render.child.is_some(), |ui| {
                            if ui
                                .button(if self.tour_render.child.is_some() {
                                    format!("{}  ({})", tr!(menu_tools_render_tour), tr!(menu_state_rendering))
                                } else {
                                    tr!(menu_tools_render_tour)
                                })
                                .on_hover_text(tr!(menu_tools_render_tour_hover))
                                .clicked()
                            {
                                self.open_tour_render();
                                ui.close_menu();
                            }
                        });
                        ui.add_enabled_ui(self.playback.is_some(), |ui| {
                            // Same thing the player's ✖ does — the transport's own ⏹ only rewinds.
                            if ui
                                .button(tr!(menu_tools_close_player))
                                .on_hover_text(tr!(menu_tools_close_player_hover))
                                .clicked()
                            {
                                self.stop_playback();
                                ui.close_menu();
                            }
                        });
                        // Enabled while a job runs even with no tour loaded: it is how to reach it.
                        ui.add_enabled_ui(self.playback.is_some() || self.farm_controller.active(), |ui| {
                            if ui
                                .button(if self.farm_controller.active() {
                                    format!("{}  ({})", tr!(menu_tools_render_farm), tr!(menu_state_running))
                                } else {
                                    tr!(menu_tools_render_farm)
                                })
                                .on_hover_text(tr!(menu_tools_render_farm_hover))
                                .clicked()
                            {
                                self.open_farm_controller();
                                ui.close_menu();
                            }
                        });
                        ui.separator();
                        if ui
                            .button(tr!(menu_tools_benchmark))
                            .on_hover_text(tr!(menu_tools_benchmark_hover))
                            .clicked()
                        {
                            self.dialogs.bench_dialog_open = true;
                            ui.close_menu();
                        }
                        if self.bench.report.is_some()
                            && ui.button(tr!(menu_tools_last_benchmark)).clicked()
                        {
                            self.dialogs.bench_open = true;
                            ui.close_menu();
                        }
                    });
                    // "Where can I go?" is ONE question: the canonical places (famous points,
                    // random, .kfr import) and the personal ones (bookmarks) were split across
                    // two menus until 2026-08-13 (UI review). Merged; bookmarks are the submenu.
                    ui.menu_button(tr!(menu_navigate), |ui| {
                        let ctx = ui.ctx().clone();
                        if ui
                            .button(tr!(menu_navigate_goto))
                            .on_hover_text(tr!(menu_navigate_goto_hover))
                            .clicked()
                        {
                            self.open_goto();
                            ui.close_menu();
                        }
                        if ui
                            .button(tr!(menu_navigate_misiurewicz))
                            .on_hover_text(tr!(menu_navigate_misiurewicz_hover))
                            .clicked()
                        {
                            self.open_misiurewicz_explorer();
                            ui.close_menu();
                        }
                        ui.separator();
                        for (name, cx, cy, mag) in FAMOUS {
                            if ui.button(*name).clicked() {
                                self.goto_location(cx, cy, *mag, name, &ctx);
                                ui.close_menu();
                            }
                        }
                        ui.menu_button(tr!(menu_navigate_exact), |ui| {
                            ui.label(
                                egui::RichText::new(tr!(menu_navigate_exact_note))
                                .weak()
                                .small(),
                            );
                            ui.separator();
                            for (name, re, im, mag) in crate::EXPRESSION_POI {
                                if ui.button(*name).clicked() {
                                    self.goto_expression(re, im, *mag, name, &ctx);
                                    ui.close_menu();
                                }
                            }
                        });
                        ui.separator();
                        if ui
                            .button(format!("{}  {}", crate::icons::RANDOM, tr!(menu_navigate_random)))
                            .on_hover_text(tr!(menu_navigate_random_hover))
                            .clicked()
                        {
                            self.random_location(&ctx);
                            ui.close_menu();
                        }
                        if ui
                            .button(format!("{}  {}", crate::icons::IMPORT, tr!(menu_navigate_import_kfr)))
                            .on_hover_text(tr!(menu_navigate_import_kfr_hover))
                            .clicked()
                        {
                            self.import_kfr(&ctx);
                            ui.close_menu();
                        }
                        // Sharing a location is a PLACES concern (moved from File 2026-08-13):
                        // it shares where you are, like the .kfr import shares where someone was.
                        if ui
                            .button(format!("{}  {}", crate::icons::SHARE, tr!(menu_navigate_share)))
                            .on_hover_text(tr!(menu_navigate_share_hover))
                            .clicked()
                        {
                            self.open_share();
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.menu_button(format!("{}  {}", crate::icons::BOOKMARK, tr!(menu_bookmarks)), |ui| {
                            self.bookmarks_menu(ui, &ctx);
                        });
                    });
                    ui.menu_button(tr!(menu_help), |ui| {
                        if ui.button(format!("{}  (F1)", tr!(menu_help_reference))).clicked() {
                            self.dialogs.help_open = true;
                            ui.close_menu();
                        }
                        if ui
                            .button(tr!(menu_help_welcome))
                            .on_hover_text(tr!(menu_help_welcome_hover))
                            .clicked()
                        {
                            self.dialogs.welcome_open = true;
                            ui.close_menu();
                        }
                        if ui
                            .button(tr!(menu_help_diagnostics))
                            .on_hover_text(tr!(menu_help_diagnostics_hover))
                            .clicked()
                        {
                            self.diagnostics.open = true;
                            ui.close_menu();
                        }
                        if ui
                            .button(tr!(menu_help_report))
                            .on_hover_text(tr!(menu_help_report_hover))
                            .clicked()
                        {
                            self.report.open = true;
                            ui.close_menu();
                        }
                        // Named for the BENEFIT, not the implementation: nobody is looking for
                        // "MPFR". The dialog itself explains what it actually is.
                        if ui
                            .button(tr!(menu_help_accelerated))
                            .on_hover_text(tr!(menu_help_accelerated_hover))
                            .clicked()
                        {
                            self.dialogs.accelerated_open = true;
                            ui.close_menu();
                        }
                        if ui
                            .button(tr!(menu_help_check_updates))
                            .on_hover_text(tr!(menu_help_check_updates_hover, track = self.update_track.label()))
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
                                        egui::RichText::new(format!("\u{2B06} {}", tr!(menu_help_update_available, version = version.as_str(), channel = channel)))
                                            .color(egui::Color32::from_rgb(0x5C, 0xC0, 0x6C)),
                                    )
                                    .clicked()
                                {
                                    // F-07: never open update metadata verbatim — require https +
                                    // github.com, else fall back to the canonical releases page.
                                    ctx.open_url(egui::OpenUrl::new_tab(
                                        crate::update::safe_release_url(&url),
                                    ));
                                    ui.close_menu();
                                }
                            }
                            Some(crate::update::UpdateStatus::UpToDate) => {
                                ui.label(egui::RichText::new(tr!(menu_help_up_to_date)).weak().small());
                            }
                            _ => {}
                        }
                        ui.separator();
                        ui.label(format!("Fractadyne v{}", version_string()));
                        ui.label(egui::RichText::new(tr!(menu_help_tagline)).weak().small());
                        ui.separator();
                        ui.label(egui::RichText::new(tr!(menu_help_license)).weak().small());
                        ui.label("MIT OR Apache-2.0");
                        ui.label(
                            egui::RichText::new(tr!(menu_help_copyright, year = "2026", holder = "Rithea Hong"))
                                .weak()
                                .small(),
                        );
                        ui.hyperlink_to(
                            format!("{} \u{2197}", tr!(menu_help_source)),
                            "https://github.com/WindySnowOwl/fractadyne",
                        );
                    });

                ui.separator();

                // Fractal picker (the name, as a dropdown).
                let prev = self.fractal;
                let mut sel = self.fractal;
                // egui caps a ComboBox popup at `spacing.combo_height` (200 px by default) and puts
                // a ScrollArea inside it. Ten formulas overflow that by a few pixels, so the list
                // showed a scrollbar while every entry was already visible.
                //
                // Derived from the row metrics rather than set to a constant: a hardcoded height
                // would be wrong again the moment a formula is added, the UI scale moves, or a text
                // size changes — and it would fail by silently reintroducing the scrollbar.
                // Over-estimating is FREE — the popup's ScrollArea shrinks to fit its content, so a
                // too-large bound simply means no scrollbar. Under-estimating is what puts one
                // back, so the margin is deliberate rather than sloppy. (A row is a `SelectableLabel`:
                // button text plus padding, ~17 px at the default 12.5 px button font, which is why
                // ten of them land within a few pixels of the 200 px cap.)
                //
                // ⚠The popup now also carries the "Show" group and, since the power families
                // (design/power-families.md), a grid of them under a heading — taller than 400 px,
                // and 400 is a SECOND cap that `height` cannot lift: egui lays a new popup's area
                // out within `spacing.default_area_size` (600×400) on its first frame, the area
                // remembers that size for the session, and the ScrollArea inside never grows past
                // it. (The uitest's `fractal-dropdown` showed the list cut off at the "Show"
                // heading with any `height`, the window's own included.) So the bound is the room
                // below the button, and the area default is raised to it for this one call — the
                // style is the context's, so it is put back straight after.
                let popup_h = (ui.ctx().screen_rect().bottom()
                    - ui.next_widget_position().y
                    - 2.0 * ui.spacing().interact_size.y)
                    .max(ui.spacing().combo_height);
                let area_default = ui.ctx().style().spacing.default_area_size;
                ui.ctx().style_mut(|s| s.spacing.default_area_size.y = popup_h);
                let mut open_formula = false;
                if std::mem::take(&mut self.dialogs.open_fractal_dropdown) {
                    // The combo box's own id rule (egui 0.31): `from_id_salt` wraps the salt in an
                    // `Id`, the button is the parent's persistent id of THAT, and the popup sits
                    // beside it ("popup"). (Of the bare string it is another id: nothing opened.)
                    let popup = ui.make_persistent_id(egui::Id::new("fractal_dropdown")).with("popup");
                    ui.memory_mut(|m| m.open_popup(popup));
                }
                // A combo box is a nested `Ui`, which the wrapping row places as one unit and cannot
                // move to the next line: where it no longer fits it is clipped at the window's edge
                // instead (the pseudo-language's longer menu titles did that at 1020 px, as a
                // translation's can). So the row is ended by hand when the room left is too small.
                let combo_w = ui.fonts(|f| {
                    f.layout_no_wrap(self.fractal.name().to_owned(), egui::TextStyle::Button.resolve(ui.style()), egui::Color32::WHITE).size().x
                }) + ui.spacing().icon_width
                    + 3.0 * ui.spacing().button_padding.x
                    + ui.spacing().item_spacing.x;
                if ui.available_size_before_wrap().x < combo_w {
                    ui.end_row();
                }
                egui::ComboBox::from_id_salt("fractal_dropdown")
                    .height(popup_h)
                    .selected_text(self.fractal.name())
                    .show_ui(ui, |ui| {
                        if let Some(k) = self.family_list(ui) {
                            sel = k;
                        }
                        ui.separator();
                        if ui
                            .selectable_label(self.fractal == FractalKind::Custom, tr!(menu_fractal_custom))
                            .on_hover_text(FractalKind::Custom.menu_hint())
                            .clicked()
                        {
                            open_formula = true;
                        }
                        // The picker is where someone goes looking for "Julia", so this is where
                        // it has to be — next to the formula it belongs to, not in a menu they
                        // have not thought to open.
                        ui.separator();
                        self.show_mode_group(ui);
                    });
                ui.ctx().style_mut(|s| s.spacing.default_area_size = area_default);
                if sel != prev {
                    self.set_fractal(sel);
                }
                if open_formula {
                    self.open_formula_dialog();
                }
                ui.separator();
                // The two toolbar toggles are the FAST path for someone who already knows what a
                // Julia set is; the picker's "Show" group is where someone finds that out. Both
                // go through `set_show_mode`, so the three surfaces cannot disagree about what
                // state a click lands in — clicking Julia while dual is on now leaves dual
                // rather than being greyed out, which is what the button appears to promise.
                // ⚠⚠**`add_enabled` on the widget, NOT `add_enabled_ui` around it.** The whole top
                // bar is one `horizontal_wrapped`, but `add_enabled_ui` builds a nested `Ui` that
                // the parent places as a single unit — so at a width where this button no longer
                // fits, it is CLIPPED at the window edge instead of wrapping to the next line
                // (user-reported, 2026-09-07). `add_enabled` adds one widget to the row that is
                // doing the wrapping, so it wraps like everything beside it.
                // A Life universe has no Julia set and no dual view; its own controls take their
                // slot — beside the picker, as what the picture is doing, and apart from the tour
                // ▶ further along, which plays something else.
                if self.fractal == FractalKind::Life {
                    self.life_toolbar(ui);
                } else if self.fractal == FractalKind::LSystem {
                    self.lsystem_toolbar(ui);
                } else {
                if ui
                    .add_enabled(
                        self.fractal.supports_julia(),
                        egui::SelectableLabel::new(self.julia_mode, tr!(menu_toolbar_julia)),
                    )
                    .on_hover_text(tr!(menu_toolbar_julia_hover))
                    .clicked()
                {
                    let m = if self.julia_mode { crate::ShowMode::Set } else { crate::ShowMode::Julia };
                    self.set_show_mode(m);
                }
                // Dual pairs a formula with its Julia set, so it's only meaningful where a Julia
                // exists (Newton has no free parameter → no Julia → no dual).
                if ui
                    .add_enabled(
                        self.fractal.supports_julia(),
                        egui::SelectableLabel::new(self.dual, crate::icons::DUAL),
                    )
                    .on_hover_text(tr!(menu_toolbar_dual_hover))
                    .clicked()
                {
                    let m = if self.dual { crate::ShowMode::Set } else { crate::ShowMode::Both };
                    self.set_show_mode(m);
                }
                }
                ui.separator();
                // ── File / I-O: open & browse, then save ──────────────────────────────
                if ui.button(crate::icons::OPEN).on_hover_text(tr!(menu_toolbar_open_hover)).clicked() {
                    self.open_view(ctx);
                }
                if ui.button(crate::icons::GALLERY).on_hover_text(tr!(menu_toolbar_gallery_hover)).clicked() {
                    self.gallery.open = true;
                    self.scan_gallery();
                }
                if ui.button(crate::icons::SAVE).on_hover_text(tr!(menu_toolbar_export_hover)).clicked() {
                    self.export.open = true;
                }
                if ui.button(crate::icons::SNAPSHOT).on_hover_text(self.snapshot_hover_text()).clicked() {
                    self.snapshot(ctx);
                }
                // ⭐Share location sits with the FILE actions, not with navigation, even though
                // its menu entry lives under Navigate (a location is a PLACES concern there). On
                // the toolbar the neighbouring verbs are open / browse / save, and this is the
                // fourth of those — it writes and reads a `.fdn`.
                if ui
                    .button(crate::icons::SHARE)
                    .on_hover_text(tr!(menu_toolbar_share_hover))
                    .clicked()
                {
                    self.open_share();
                }
                ui.separator();
                // ── Navigation / location: zoom, reset/home, bookmark ─────────────────
                if ui.button(crate::icons::ZOOM_IN).on_hover_text(tr!(menu_toolbar_zoom_in_hover)).clicked() {
                    self.zoom_center(crate::TOOLBAR_ZOOM_IN_FACTOR);
                }
                if ui.button(crate::icons::ZOOM_OUT).on_hover_text(tr!(menu_toolbar_zoom_out_hover)).clicked() {
                    self.zoom_center(crate::TOOLBAR_ZOOM_OUT_FACTOR);
                }
                // Click-to-zoom tool (single view): arm left-click = dive into the point,
                // right-click = back out; drag still pans. Factor set in Settings ▸ Navigation.
                if ui
                    .add(egui::SelectableLabel::new(self.click_zoom, crate::icons::CLICK_ZOOM))
                    .on_hover_text({
                        let factor = format!("{:.0}", self.render_cfg.click_zoom_factor);
                        let base = tr!(menu_toolbar_click_zoom_hover, factor = factor);
                        if self.dual { format!("{base} {}", tr!(menu_toolbar_click_zoom_dual)) } else { base }
                    })
                    .clicked()
                {
                    self.click_zoom = !self.click_zoom;
                }
                // Auto-zoom (autopilot): highlighted while running; click to start/stop. Single view only.
                let running = self.autopilot.active;
                if ui
                    .add_enabled(
                        !self.dual,
                        egui::SelectableLabel::new(running, crate::icons::AUTOPILOT),
                    )
                    .on_hover_text(if running { tr!(menu_toolbar_autopilot_running_hover) } else { tr!(menu_toolbar_autopilot_hover) })
                    .clicked()
                {
                    self.toggle_autopilot(ctx);
                }
                if ui.button(crate::icons::RESET_VIEW).on_hover_text(tr!(menu_toolbar_reset_hover)).clicked() {
                    self.reset_view();
                }
                if ui
                    .button(crate::icons::HOME)
                    .on_hover_text(tr!(menu_toolbar_home_hover))
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
                .on_hover_text(tr!(menu_toolbar_bookmarks_hover));
                ui.separator();
                // ── Appearance / display ──────────────────────────────────────────────
                if ui
                    .button(crate::icons::PALETTE)
                    .on_hover_text(tr!(menu_toolbar_next_palette_hover, name = fractadyne_color::PRESETS[self.coloring.palette_idx].name))
                    .clicked()
                {
                    self.coloring.palette_idx = (self.coloring.palette_idx + 1) % fractadyne_color::PRESETS.len();
                }
                if ui
                    .selectable_label(self.perf.enabled, crate::icons::PERF)
                    .on_hover_text(tr!(menu_view_perf_panel))
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
                    Some(p) => tr!(menu_toolbar_play_last_hover, file = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()),
                    None => tr!(menu_toolbar_play_hover),
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
                        .on_hover_text(tr!(menu_toolbar_speed_hover));
                }
                if ui
                    .selectable_label(self.fullscreen, crate::icons::FULLSCREEN)
                    .on_hover_text(tr!(menu_toolbar_fullscreen_hover))
                    .clicked()
                {
                    self.fullscreen = !self.fullscreen;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
                }
                // The language menu ends the bar, in every mode: nobody is stuck in a language they
                // cannot read (design/multilingual.md §8). No mode may hide it.
                ui.separator();
                self.draw_language_menu(ui);
            });
        });
        self.perf.layout.top_bar = Some(top.response.rect);
    }

    /// The DIRECT path's precision wall, for the status-bar diagnostic (design/custom-formulas.md
    /// §5.1). Current GPU compilers fold the error-free transforms double-single arithmetic relies
    /// on (`--gputest`), so a pixel's coordinate is single precision: once one f32 step of it spans
    /// more than a pixel, neighbouring pixels compute the same point and the image breaks into
    /// blocks. Measured per component, at the pixel coordinate's own magnitude — the orbit passing
    /// |z| ≈ 1 does not merge pixels (at 0.286+0.012i the blocks were 1.98 px across, the real
    /// part's step, and 1.00 down) — and on a field view (−1.64+0.36i, 549,309×) as 17×4-px
    /// blocks, one f32 step each way. `cx`/`cy` are the view centre, `half_w`/`half_h` its
    /// half-extent (the step is taken at the largest magnitude in view), `upp` the pixel size.
    /// Only the direct path has this wall; perturbation iterates small offsets.
    pub(crate) fn direct_precision_status(
        direct: bool,
        (cx, cy): (f64, f64),
        (half_w, half_h): (f64, f64),
        upp: f64,
        mag: f64,
    ) -> Option<(&'static str, String, bool)> {
        if !direct || !(upp > 0.0) || !upp.is_finite() {
            return None;
        }
        // One f32 step at |v|: the gap to the next representable value.
        let f32_step = |v: f64| {
            let a = v.abs() as f32;
            if !a.is_finite() {
                return f64::INFINITY;
            }
            (f32::from_bits(a.to_bits() + 1) - a) as f64
        };
        let step = f32_step(cx.abs() + half_w).max(f32_step(cy.abs() + half_h));
        if step <= upp {
            return None;
        }
        let blocks = step / upp;
        Some((
            "⚠ depth limit",
            format!(
                "This view renders directly on the GPU in single precision, where one step of the \
                 coordinates is {} pixels wide here: neighbouring pixels compute the same point, \
                 so the image breaks into blocks. At this location it is sharp to about {}×. This \
                 formula has no deep-zoom (perturbation) path yet, so zooming further shows no \
                 more detail.",
                commas(&format!("{blocks:.0}")),
                commas(&format!("{:.0}", mag / blocks))
            ),
            true,
        ))
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
        // Ambient minibrot period: the last solved period, but only while the view still equals the
        // one it was solved at (`view_key`). Resolved here, before the panel closure borrows `self`,
        // so the bar just paints the answer. `None` while nothing is solved or the view has moved.
        let period_now = self
            .feature_period
            .as_ref()
            .filter(|(_, key)| key == &self.view_key())
            .map(|(p, _)| *p);
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
                // Every readout goes through here, so this is also where they are recorded for
                // `--uitest` — what was drawn, not what a check thinks would have been.
                let mut drawn: Vec<String> = Vec::new();
                let mut mono = |ui: &mut egui::Ui, t: String| {
                    drawn.push(t.clone());
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
                    // A universe's cursor is a CELL: column, row (rows grow downward).
                    Some((mx, my)) if self.fractal == FractalKind::Life => {
                        (format!("{}", mx.floor() as i64), format!("{}", (-my).floor() as i64))
                    }
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
                let life_bar = matches!(self.fractal, FractalKind::Life | FractalKind::LSystem);
                if self.fractal == FractalKind::LSystem {
                    // A drawing's readouts: the zoom, then the order and what was drawn (segments
                    // and filled shapes), in the same three slots at fixed widths.
                    let drawn = self.lsystem.last_walk().map(|w| w.segments + w.polygons);
                    let (order, segments) = crate::lsystem_view::status_readouts(self.lsystem.order_at(&self.viewport), drawn);
                    mono(ui, crate::zoom_readout(false, self.viewport.log2_magnification(), self.julia_viewport.log2_magnification()));
                    ui.separator();
                    mono(ui, order);
                    ui.separator();
                    mono(ui, segments);
                } else if life_bar {
                    // A universe's readouts, in the same three slots and at their own fixed
                    // widths: the scale, the generation, the population.
                    let (scale, generation, population) = crate::life_view::status_readouts(
                        self.viewport.units_per_pixel.to_f64(),
                        self.life.generation(),
                        self.life.population(),
                    );
                    mono(ui, scale);
                    ui.separator();
                    mono(ui, generation);
                    ui.separator();
                    mono(ui, population);
                } else {
                mono(ui, crate::zoom_readout(
                    self.dual,
                    self.viewport.log2_magnification(),
                    self.julia_viewport.log2_magnification(),
                ));
                }
                if !life_bar {
                    ui.separator();
                }
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
                if !life_bar {
                    mono(ui, crate::iter_readout(eff_iter));
                }
                // Ambient minibrot period (§3.1): the finder result stays on screen instead of
                // fading with its toast. RESERVED WIDTH, and drawn TRANSPARENT when no period
                // applies (none solved yet, or the view has moved off the feature), so the slot's
                // presence never reflows the bar — the device the diagnostic slot below also uses.
                // `mono`'s last call is the iter readout above, so `drawn.push` here is free of it.
                if !life_bar {
                ui.separator();
                {
                    let ptext = crate::period_readout(period_now);
                    let pcolor = if period_now.is_some() {
                        ui.visuals().text_color()
                    } else {
                        egui::Color32::TRANSPARENT
                    };
                    drawn.push(ptext.clone());
                    let r = ui.add(
                        egui::Label::new(egui::RichText::new(ptext).monospace().color(pcolor))
                            .wrap_mode(egui::TextWrapMode::Extend),
                    );
                    if period_now.is_some() {
                        r.on_hover_text(
                            "Period of the minibrot you jumped to (press M, or Go to location ▸ Go \
                             to feature). It stays while the view sits on the feature and clears \
                             when you move away.",
                        );
                    }
                }
                } // not a Life view
                // Rendering-limit diagnostics: when a cap is genuinely binding, say so where the
                // user is already looking, instead of leaving a black/flat view unexplained (the
                // Misiurewicz-spar reports arrived as mystery screenshots precisely because the
                // app knew it was clamped and said nothing).
                let vc = &self.ref_cache[0];
                let vp = &self.viewport;
                let mag = vp.magnification();
                let upp = vp.units_per_pixel.to_f64();
                let direct = self.render_mode(self.fractal, self.julia_mode, mag).is_direct();
                // (A Life view has no escape-time limits: the slot stays transparent.)
                let limit = if life_bar { None } else { Self::direct_precision_status(
                    direct,
                    vp.center_f64(),
                    (upp * vp.width_px * 0.5, upp * vp.height_px * 0.5),
                    upp,
                    mag,
                )
                .or_else(|| {
                    Self::limit_status(
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
                    )
                }) };
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
                if !life_bar {
                    ui.separator();
                }
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
                // The diagnostic slot is recorded too, so `--uitest` sees the same reserved
                // width the reflow fix depends on rather than only the readouts above it.
                // (A Life view never shows a diagnostic, so it reserves no slot for one.)
                if !life_bar {
                    drawn.push(text.clone());
                    let r = ui.add(
                        egui::Label::new(egui::RichText::new(text).monospace().color(color))
                            .wrap_mode(egui::TextWrapMode::Extend),
                    );
                    if let Some(detail) = detail {
                        r.on_hover_text(detail);
                    }
                }
                drawn
            })
            .inner
        });
        // Expose the panel's height so the UI-test harness can detect the status bar wrapping to a
        // second line (or wavering between one and two at a fixed width — a repaint-storm smell),
        // its rect for the layout invariants, and the readouts themselves.
        self.perf.status_bar_h = resp.response.rect.height();
        self.perf.layout.status_bar = Some(resp.response.rect);
        self.perf.layout.window = Some(ctx.screen_rect());
        self.perf.status_bar_texts = resp.inner;
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
#[path = "../../tests/unit/ui/menus/tests.rs"]
mod tests;
