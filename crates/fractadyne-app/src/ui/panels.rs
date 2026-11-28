//! The right-hand control panel (REFACTOR-PLAN Phase 3, intra-crate UI split). An
//! `impl FractadyneApp` block moved verbatim from `main.rs`.
use crate::*;

/// Width of the control panel's label column. One value for every row, so the controls share a
/// left edge instead of starting wherever the preceding label happened to end.
const PANEL_LABEL_W: f32 = 104.0;

/// One labeled control row: label on the LEFT, control to its right.
///
/// egui's own idiom appends the label - `Slider::text` and `ComboBox::from_label` both put it
/// AFTER the widget - which reads as a caption and leaves a column of ragged text down the right
/// of the panel. Label-first is the desktop convention for a settings panel, and a fixed-width
/// label column aligns the controls with each other as a side effect.
///
/// Checkboxes deliberately do NOT use this: a trailing label is the convention for a checkbox,
/// and the panel's seventeen already read correctly.
impl crate::FractadyneApp {
    /// The trap's shape (the orbit trap method's, or the inside's when the trap colors it, B5) and
    /// where it sits (F4): drawn with whichever uses it.
    fn trap_shape_row(&mut self, ui: &mut egui::Ui) {
        let before = self.coloring.trap_type;
        labelled(ui, &tr!(panel_trap_shape), |ui| {
            egui::ComboBox::from_id_salt("panel_trap_shape")
                .selected_text(self.coloring.trap_type.label())
                .show_ui(ui, |ui| {
                    for t in TrapType::ALL {
                        ui.selectable_value(&mut self.coloring.trap_type, t, t.label());
                    }
                });
        });
        // The outline picked with none loaded asks for a drawing (F4 part 4).
        if self.coloring.trap_type != before {
            let ctx = ui.ctx().clone();
            self.coloring.trap_type = self.trap_outline_picked(&ctx, self.coloring.trap_type, before);
        }
        if self.coloring.trap_type == TrapType::Outline {
            self.trap_outline_row(ui);
        }
        trap_place_rows(ui, self.coloring.trap_type, &mut self.coloring.trap_place, &mut self.coloring.trap_phase_anim);
        self.trap_on_cycle_button(ui, false);
        self.trap_tail_rows(ui);
    }

    /// The Smooth method's band texture (coloring plan E1): which statistic moves the band edges,
    /// and by how much while one does.
    fn band_texture_rows(&mut self, ui: &mut egui::Ui) {
        use fractadyne_core::coloring::BandTexture;
        let label = |t: BandTexture| match t {
            BandTexture::None => tr!(panel_band_texture_none),
            BandTexture::Stripe => tr!(color_method_stripe),
            BandTexture::Triangle => tr!(color_method_triangle),
        };
        labelled(ui, &tr!(panel_band_texture), |ui| {
            egui::ComboBox::from_id_salt("panel_band_texture")
                .selected_text(label(self.coloring.band_texture))
                .show_ui(ui, |ui| {
                    for t in BandTexture::ALL {
                        ui.selectable_value(&mut self.coloring.band_texture, t, label(t));
                    }
                })
                .response
                .on_hover_text(tr!(panel_band_texture_hover));
        });
        if self.coloring.band_texture != BandTexture::None {
            let r = labelled(ui, &tr!(panel_band_texture_amount), |ui| {
                ui.add(egui::Slider::new(&mut self.coloring.band_texture_amount, crate::slider_range::BAND_TEXTURE_AMOUNT))
            });
            if slider_double_clicked(&r) {
                self.coloring.band_texture_amount = fractadyne_core::coloring::BAND_TEXTURE_AMOUNT_DEFAULT;
            }
            r.on_hover_text(format!("{}\n\n{}", tr!(panel_band_texture_amount_hover), tr!(panel_double_click_reset)));
            // E2: only near the set.
            let r = labelled(ui, &tr!(panel_band_texture_near), |ui| {
                ui.add(
                    egui::Slider::new(&mut self.coloring.band_texture_near, crate::slider_range::BAND_TEXTURE_NEAR)
                        .suffix(" px"),
                )
            });
            if slider_double_clicked(&r) {
                self.coloring.band_texture_near = 0.0;
            }
            r.on_hover_text(format!("{}\n\n{}", tr!(panel_band_texture_near_hover), tr!(panel_double_click_reset)));
        }
    }

    /// The Smooth method's statistic layer (coloring plan E3): which statistic's gray lies over the
    /// colors, and while one does, how it merges, its contrast and its opacity.
    fn stat_layer_rows(&mut self, ui: &mut egui::Ui) {
        use fractadyne_core::coloring::{BandTexture, BlendMode};
        let label = |t: BandTexture| match t {
            BandTexture::None => tr!(panel_band_texture_none),
            BandTexture::Stripe => tr!(color_method_stripe),
            BandTexture::Triangle => tr!(color_method_triangle),
        };
        let blend = |b: BlendMode| match b {
            BlendMode::HardLight => tr!(panel_stat_layer_blend_hard_light),
            BlendMode::SoftLight => tr!(panel_stat_layer_blend_soft_light),
            BlendMode::Overlay => tr!(panel_stat_layer_blend_overlay),
            BlendMode::Multiply => tr!(panel_stat_layer_blend_multiply),
            BlendMode::Screen => tr!(panel_stat_layer_blend_screen),
        };
        labelled(ui, &tr!(panel_stat_layer), |ui| {
            egui::ComboBox::from_id_salt("panel_stat_layer")
                .selected_text(label(self.coloring.stat_layer))
                .show_ui(ui, |ui| {
                    for t in BandTexture::ALL {
                        ui.selectable_value(&mut self.coloring.stat_layer, t, label(t));
                    }
                })
                .response
                .on_hover_text(tr!(panel_stat_layer_hover));
        });
        if self.coloring.stat_layer == BandTexture::None {
            return;
        }
        labelled(ui, &tr!(panel_stat_layer_blend), |ui| {
            egui::ComboBox::from_id_salt("panel_stat_layer_blend")
                .selected_text(blend(self.coloring.stat_layer_blend))
                .show_ui(ui, |ui| {
                    for b in BlendMode::ALL {
                        ui.selectable_value(&mut self.coloring.stat_layer_blend, b, blend(b));
                    }
                })
                .response
                .on_hover_text(tr!(panel_stat_layer_blend_hover));
        });
        let r = labelled(ui, &tr!(panel_stat_layer_contrast), |ui| {
            ui.add(egui::Slider::new(&mut self.coloring.stat_layer_contrast, crate::slider_range::STAT_LAYER_CONTRAST))
        });
        if slider_double_clicked(&r) {
            self.coloring.stat_layer_contrast = fractadyne_core::coloring::LAYER_CONTRAST_DEFAULT;
        }
        r.on_hover_text(format!("{}\n\n{}", tr!(panel_stat_layer_contrast_hover), tr!(panel_double_click_reset)));
        let r = labelled(ui, &tr!(panel_stat_layer_opacity), |ui| {
            ui.add(egui::Slider::new(&mut self.coloring.stat_layer_opacity, crate::slider_range::STAT_LAYER_OPACITY))
        });
        if slider_double_clicked(&r) {
            self.coloring.stat_layer_opacity = 1.0;
        }
        r.on_hover_text(format!("{}\n\n{}", tr!(panel_stat_layer_opacity_hover), tr!(panel_double_click_reset)));
    }

    /// The Smooth method's statistic bump (coloring plan E4): which statistic is read as a height,
    /// and while one is, its height.
    fn stat_bump_rows(&mut self, ui: &mut egui::Ui) {
        use fractadyne_core::coloring::BandTexture;
        let label = |t: BandTexture| match t {
            BandTexture::None => tr!(panel_band_texture_none),
            BandTexture::Stripe => tr!(color_method_stripe),
            BandTexture::Triangle => tr!(color_method_triangle),
        };
        labelled(ui, &tr!(panel_stat_bump), |ui| {
            egui::ComboBox::from_id_salt("panel_stat_bump")
                .selected_text(label(self.coloring.stat_bump))
                .show_ui(ui, |ui| {
                    for t in BandTexture::ALL {
                        ui.selectable_value(&mut self.coloring.stat_bump, t, label(t));
                    }
                })
                .response
                .on_hover_text(tr!(panel_stat_bump_hover));
        });
        if self.coloring.stat_bump == BandTexture::None {
            return;
        }
        let r = labelled(ui, &tr!(panel_stat_bump_height), |ui| {
            ui.add(egui::Slider::new(&mut self.coloring.stat_bump_height, crate::slider_range::STAT_BUMP_HEIGHT).suffix(" px"))
        });
        if slider_double_clicked(&r) {
            self.coloring.stat_bump_height = fractadyne_core::coloring::BUMP_HEIGHT_DEFAULT;
        }
        r.on_hover_text(format!("{}\n\n{}", tr!(panel_stat_bump_height_hover), tr!(panel_double_click_reset)));
    }

    /// The Smooth method's gloss (coloring plan E5): which statistic sets how strong the relief
    /// light's highlight is, and while one is, how much. Grayed out, saying why, while no highlight
    /// is drawn (the relief light, its slope from the distance and a highlight above 0).
    fn stat_gloss_rows(&mut self, ui: &mut egui::Ui) {
        use fractadyne_core::coloring::BandTexture;
        let label = |t: BandTexture| match t {
            BandTexture::None => tr!(panel_band_texture_none),
            BandTexture::Stripe => tr!(color_method_stripe),
            BandTexture::Triangle => tr!(color_method_triangle),
        };
        let e = &self.effects;
        let drawn = self.fractal.caps().distance_estimate && e.light && e.light_steep && e.light_highlight > 0.0;
        ui.add_enabled_ui(drawn, |ui| {
            labelled(ui, &tr!(panel_stat_gloss), |ui| {
                egui::ComboBox::from_id_salt("panel_stat_gloss")
                    .selected_text(label(self.coloring.stat_gloss))
                    .show_ui(ui, |ui| {
                        for t in BandTexture::ALL {
                            ui.selectable_value(&mut self.coloring.stat_gloss, t, label(t));
                        }
                    })
                    .response
                    .on_hover_text(tr!(panel_stat_gloss_hover));
            })
        })
        .response
        .on_disabled_hover_text(tr!(panel_stat_gloss_off));
        if self.coloring.stat_gloss == BandTexture::None {
            return;
        }
        let r = ui
            .add_enabled_ui(drawn, |ui| {
                labelled(ui, &tr!(panel_stat_gloss_amount), |ui| {
                    ui.add(egui::Slider::new(&mut self.coloring.stat_gloss_amount, crate::slider_range::STAT_GLOSS_AMOUNT))
                })
            })
            .inner;
        if slider_double_clicked(&r) {
            self.coloring.stat_gloss_amount = fractadyne_core::coloring::GLOSS_AMOUNT_DEFAULT;
        }
        r.on_hover_text(format!("{}\n\n{}", tr!(panel_stat_gloss_amount_hover), tr!(panel_double_click_reset)))
            .on_disabled_hover_text(tr!(panel_stat_gloss_off));
    }

    /// The trap's tail (F4 part 5), the method's trap and the inside's alike: Tail only, and its
    /// length while it is on. Under the trap's shape, wherever that is drawn.
    fn trap_tail_rows(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.coloring.trap_tail, tr!(panel_trap_tail)).on_hover_text(tr!(panel_trap_tail_hover));
        if self.coloring.trap_tail {
            let r = labelled(ui, &tr!(panel_tail_length), |ui| {
                ui.add(
                    egui::Slider::new(&mut self.coloring.trap_tail_len, crate::slider_range::TRAP_TAIL_PANEL)
                        .logarithmic(true)
                        .suffix(format!(" {}", tr!(panel_tail_length_unit)))
                        .custom_formatter(|n, _| crate::grouped_count(n))
                        .custom_parser(crate::parse_grouped_number),
                )
            });
            if slider_double_clicked(&r) {
                self.coloring.trap_tail_len = fractadyne_state::ColoringSpec::default().trap_tail_len;
            }
            r.on_hover_text(format!("{}\n\n{}", tr!(panel_trap_tail_length_hover), tr!(panel_double_click_reset)));
        }
    }

    /// The trap method's inside, when the trap colors it: a trap of its own, or the method's (the
    /// default). Its own has the same placement rows as the method's.
    fn inside_trap_rows(&mut self, ui: &mut egui::Ui) {
        let same = tr!(panel_inside_trap_same);
        let before = self.coloring.interior_trap;
        labelled(ui, &tr!(panel_inside_trap), |ui| {
            egui::ComboBox::from_id_salt("panel_inside_trap")
                .selected_text(self.coloring.interior_trap.map_or(same.clone(), |t| t.label()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.coloring.interior_trap, None, same.as_str());
                    for t in TrapType::ALL {
                        ui.selectable_value(&mut self.coloring.interior_trap, Some(t), t.label());
                    }
                })
                .response
                .on_hover_text(tr!(panel_inside_trap_hover));
            if self.dialogs.uitest_inside_trap_scroll {
                ui.scroll_to_cursor(Some(egui::Align::Min));
            }
        });
        if self.coloring.interior_trap != before && self.coloring.interior_trap == Some(TrapType::Outline) {
            let ctx = ui.ctx().clone();
            if self.trap_outline_picked(&ctx, TrapType::Outline, TrapType::Point) != TrapType::Outline {
                self.coloring.interior_trap = before;
            }
        }
        if let Some(shape) = self.coloring.interior_trap {
            if shape == TrapType::Outline {
                self.trap_outline_row(ui);
            }
            trap_place_rows(ui, shape, &mut self.coloring.interior_trap_place, &mut self.coloring.trap_phase_anim);
            self.trap_on_cycle_button(ui, true);
        }
    }

    /// Place on the cycle (F4 part 2): onto the attracting cycle at the view's center, the next point
    /// each press; `inside`, the inside's own trap.
    fn trap_on_cycle_button(&mut self, ui: &mut egui::Ui, inside: bool) {
        if ui.button(tr!(panel_trap_on_cycle)).on_hover_text(tr!(panel_trap_on_cycle_hover)).clicked() {
            let placed = self.place_trap_on_cycle_for(inside);
            let msg = self.trap_cycle_message(placed);
            let ctx = ui.ctx().clone();
            self.set_toast(msg, &ctx);
        }
    }

    /// The panel's Brightness, Contrast and Saturation rows (`fractadyne_color::adjust`).
    fn palette_adjust_rows(&mut self, ui: &mut egui::Ui) {
        use fractadyne_color::adjust::PaletteAdjust;
        let neutral = PaletteAdjust::NEUTRAL;
        let adj = &mut self.coloring.palette_adjust;
        let rows: [(String, String, &mut f32, std::ops::RangeInclusive<f32>, f32); 3] = [
            (tr!(panel_palette_brightness), tr!(panel_palette_brightness_hover), &mut adj.brightness, crate::slider_range::PALETTE_BRIGHTNESS, neutral.brightness),
            (tr!(panel_palette_contrast), tr!(panel_palette_contrast_hover), &mut adj.contrast, crate::slider_range::PALETTE_CONTRAST, neutral.contrast),
            (tr!(panel_palette_saturation), tr!(panel_palette_saturation_hover), &mut adj.saturation, crate::slider_range::PALETTE_SATURATION, neutral.saturation),
        ];
        for (label, hover, value, range, reset) in rows {
            let r = labelled(ui, &label, |ui| ui.add(egui::Slider::new(value, range)));
            if slider_double_clicked(&r) {
                *value = reset;
            }
            r.on_hover_text(hover);
        }
    }
}

/// A trap's placement rows (F4): its center, and the turn only for a shape that turns and the size
/// only for one that has a size (`TrapSpec::turns`, `sized`), so no row does nothing. Double-click
/// puts a slider back.
fn trap_place_rows(ui: &mut egui::Ui, shape: TrapType, place: &mut TrapPlace, phase_anim: &mut bool) {
    let reset = TrapPlace::DEFAULT;
    let spec = shape.spec();
    let center_hover = format!("{}\n\n{}", tr!(panel_trap_center_hover), tr!(panel_double_click_reset));
    let [cx, cy] = &mut place.center;
    for (label, value, d) in [
        (tr!(panel_trap_center_x), cx, reset.center[0]),
        (tr!(panel_trap_center_y), cy, reset.center[1]),
    ] {
        let r = labelled(ui, &label, |ui| ui.add(egui::Slider::new(value, crate::slider_range::TRAP_CENTER).fixed_decimals(3)));
        if slider_double_clicked(&r) {
            *value = d;
        }
        r.on_hover_text(&center_hover);
    }
    if spec.turns {
        let r = labelled(ui, &tr!(panel_trap_rotation), |ui| {
            ui.add(egui::Slider::new(&mut place.rotation, crate::slider_range::TRAP_ROTATION).suffix("°").fixed_decimals(1))
        });
        if slider_double_clicked(&r) {
            place.rotation = reset.rotation;
        }
        r.on_hover_text(format!("{}\n\n{}", tr!(panel_trap_rotation_hover), tr!(panel_double_click_reset)));
    }
    if spec.sized {
        let r = labelled(ui, &tr!(panel_trap_size), |ui| {
            ui.add(egui::Slider::new(&mut place.size, crate::slider_range::TRAP_SIZE).logarithmic(true))
        });
        if slider_double_clicked(&r) {
            place.size = reset.size;
        }
        r.on_hover_text(format!("{}\n\n{}", tr!(panel_trap_size_hover), tr!(panel_double_click_reset)));
    }
    // The square wave's harmonics (user 2026-11-25).
    if spec.harmonic {
        let r = labelled(ui, &tr!(panel_trap_harmonics), |ui| {
            ui.add(egui::Slider::new(&mut place.harmonics, crate::slider_range::TRAP_HARMONICS))
        });
        if slider_double_clicked(&r) {
            place.harmonics = reset.harmonics;
        }
        r.on_hover_text(format!("{}\n\n{}", tr!(panel_trap_harmonics_hover), tr!(panel_double_click_reset)));
    }
    // The sine wave's overlaid harmonics (user 2026-11-25).
    if spec.overlaid {
        let r = labelled(ui, &tr!(panel_trap_harmonics), |ui| {
            ui.add(egui::Slider::new(&mut place.sine_harmonics, crate::slider_range::TRAP_HARMONICS))
        });
        if slider_double_clicked(&r) {
            place.sine_harmonics = reset.sine_harmonics;
        }
        r.on_hover_text(format!("{}\n\n{}", tr!(panel_trap_sine_harmonics_hover), tr!(panel_double_click_reset)));
    }
    // A wave's phase, and its animation at the palette's Speed (user 2026-11-25).
    if spec.phased {
        let r = labelled(ui, &tr!(panel_trap_phase), |ui| {
            ui.add(egui::Slider::new(&mut place.phase, crate::slider_range::TRAP_PHASE).suffix("°").fixed_decimals(1))
        });
        if slider_double_clicked(&r) {
            place.phase = reset.phase;
        }
        r.on_hover_text(format!("{}\n\n{}", tr!(panel_trap_phase_hover), tr!(panel_double_click_reset)));
        labelled(ui, "", |ui| {
            ui.checkbox(phase_anim, tr!(panel_trap_phase_animate)).on_hover_text(tr!(panel_trap_phase_animate_hover))
        });
    }
}

/// A palette map curve's name (C2).
pub(crate) fn transfer_label(t: fractadyne_core::coloring::Transfer) -> String {
    match t {
        fractadyne_core::coloring::Transfer::Linear => tr!(transfer_linear),
        fractadyne_core::coloring::Transfer::Sqrt => tr!(transfer_sqrt),
        fractadyne_core::coloring::Transfer::Cbrt => tr!(transfer_cbrt),
        fractadyne_core::coloring::Transfer::Root4 => tr!(transfer_root4),
        fractadyne_core::coloring::Transfer::Log => tr!(transfer_log),
        fractadyne_core::coloring::Transfer::LogLog => tr!(transfer_loglog),
        fractadyne_core::coloring::Transfer::Square => tr!(transfer_square),
    }
}

/// A wrap's name (C2).
pub(crate) fn wrap_label(w: fractadyne_core::coloring::Wrap) -> String {
    match w {
        fractadyne_core::coloring::Wrap::Repeat => tr!(wrap_repeat),
        fractadyne_core::coloring::Wrap::Mirror => tr!(wrap_mirror),
        fractadyne_core::coloring::Wrap::Clamp => tr!(wrap_clamp),
    }
}

/// An edge shading curve's name (C3).
pub(crate) fn edge_curve_label(c: fractadyne_core::coloring::EdgeCurve) -> String {
    use fractadyne_core::coloring::EdgeCurve as E;
    match c {
        E::Linear => tr!(edge_curve_linear),
        E::Sqrt => tr!(edge_curve_sqrt),
        E::Square => tr!(edge_curve_square),
        E::Smooth => tr!(edge_curve_smooth),
        E::Hard => tr!(edge_curve_hard),
    }
}

/// Whether `r`, a slider's response, was double-clicked: on its value box, or on its handle or track.
/// ⚠egui's slider senses only DRAGS on the track (`Sense::drag()`, egui 0.31 `slider.rs`), so
/// `Response::double_clicked` saw a double-click on the value box alone and the handle did nothing
/// (user, 2026-11-20). A double-click of the button with the pointer over the slider counts too.
/// ⚠So does a TRIPLE: egui counts a third click within 0.6 s of the first, so a double-click soon
/// after a click on the slider (setting it, then resetting it) is a triple to egui, and was missed
/// (the uitest's `slider-double-click-reset`, 0.58 s after its control's click). A triple-click
/// means nothing else on a slider.
pub(crate) fn slider_double_clicked(r: &egui::Response) -> bool {
    let b = egui::PointerButton::Primary;
    r.double_clicked() || (r.hovered() && r.ctx.input(|i| i.pointer.button_double_clicked(b) || i.pointer.button_triple_clicked(b)))
}

/// The light's direction as a picture (user, 2026-11-22: "2.2 rad is hard for most people to
/// read"): a dial at the start of the Light angle row, a sun on its rim where the light comes from
/// and an arrow from it across the dial the way the light shines.
/// ⭐The angle turns counterclockwise from the right, up a quarter turn, ON SCREEN: measured
/// 2026-11-22 on the overview, the light at 0 left the right half at 0.84 of its unlit shade and
/// the left at 0.34; at π/2 the top half at 0.88 and the bottom at 0.31. A screen point's y grows
/// DOWN, hence the minus in [`dial_point`] and [`dial_angle_at`].
/// A press or a drag on it points the light at the pointer (on round, past the dial's edge); a
/// double-click puts `reset` back, as the row's slider does. `angle` and `reset` are in degrees,
/// as the row shows them (user, 2026-11-28); the dial's geometry works in radians.
pub(crate) fn light_dial(ui: &mut egui::Ui, angle: &mut f32, reset: f32) -> egui::Response {
    let side = ui.spacing().interact_size.y;
    let (rect, mut r) = ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::click_and_drag());
    let c = rect.center();
    if r.is_pointer_button_down_on() {
        if let Some(a) = r.interact_pointer_pos().and_then(|p| dial_angle_at(c, p)).map(f32::to_degrees) {
            if a != *angle {
                *angle = a;
                r.mark_changed();
            }
        }
    }
    // After the press above, which a double-click's second press is too: the reset wins.
    if slider_double_clicked(&r) && *angle != reset {
        *angle = reset;
        r.mark_changed();
    }
    if ui.is_rect_visible(rect) {
        let w = ui.style().interact(&r);
        let ring = if w.bg_stroke.width > 0.0 { w.bg_stroke } else { ui.visuals().widgets.noninteractive.bg_stroke };
        let painter = ui.painter();
        let rad = side * 0.5 - 0.5;
        painter.circle(c, rad, ui.visuals().extreme_bg_color, ring);
        let ink = egui::Stroke::new(1.5_f32, w.fg_stroke.color);
        let sun = dial_point(c, rad - 3.0, angle.to_radians());
        let tip = dial_point(c, rad - 1.5, angle.to_radians() + std::f32::consts::PI);
        painter.line_segment([sun, tip], ink);
        // The arrowhead: two strokes back from the tip, about 35 degrees either side of the shaft.
        let back = (sun - tip).normalized() * (side * 0.25);
        for turn in [-0.6f32, 0.6] {
            let (s, co) = turn.sin_cos();
            painter.line_segment([tip, tip + egui::vec2(back.x * co - back.y * s, back.x * s + back.y * co)], ink);
        }
        painter.circle(sun, 2.75, DIAL_SUN, egui::Stroke::new(1.0_f32, w.fg_stroke.color));
    }
    r
}

/// The sun on the light's dial: one warm yellow in either theme, ringed in the text's color.
const DIAL_SUN: egui::Color32 = egui::Color32::from_rgb(255, 200, 40);

/// The point `r` from `c` in the direction `angle`: radians counterclockwise from the right, on
/// screen, where a point's y grows down.
pub(crate) fn dial_point(c: egui::Pos2, r: f32, angle: f32) -> egui::Pos2 {
    c + egui::vec2(r * angle.cos(), -r * angle.sin())
}

/// The direction from `c` to `p` in [0, 2π], as [`dial_point`] turns it; `None` at `c` itself.
pub(crate) fn dial_angle_at(c: egui::Pos2, p: egui::Pos2) -> Option<f32> {
    let (dx, dy) = (p.x - c.x, c.y - p.y);
    (dx * dx + dy * dy > 1.0e-6).then(|| dy.atan2(dx).rem_euclid(std::f32::consts::TAU))
}

#[cfg(test)]
#[path = "../../tests/unit/ui/panels/light_dial_tests.rs"]
mod light_dial_tests;

#[cfg(test)]
#[path = "../../tests/unit/ui/panels/slider_reset_tests.rs"]
mod slider_reset_tests;

pub(crate) fn labelled<R>(ui: &mut egui::Ui, label: &str, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
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
    /// The "Log color scale" checkbox, drawn identically in the Color menu and the Controls
    /// panel — one function so the two can never disagree about its state, its wording, or when
    /// it is live.
    ///
    /// ⭐**Disabled unless some normalization is active, because otherwise it does nothing at
    /// all.** `log_palette` is read only inside the normalized mapping: live, inside the
    /// `normalize_live` branch of `live_norm_cycle_offset`; offline, only by
    /// `render_export_normalized`. With both off it is a control that silently changes no pixel,
    /// which is worse than an absent one — the user concludes the feature is broken.
    ///
    /// ⚠The gate is `normalize_live || normalize`, not just the live flag: `normalize` is the
    /// EXPORT-side switch (`--normalize`), and someone who launched with it still needs this
    /// checkbox to steer their exports even with the live view un-normalized.
    /// "Always fit range" — the range-fit half of normalization, drawn identically in the panel and
    /// the Color menu so the two can never disagree, as the Scale ([`Self::scale_row`]).
    ///
    /// ⭐**Why this is a separate control and not a looser threshold.** "Normalize deep colors" is a
    /// Nyquist ALIASING guard (`mean|Δ smooth-iter| × cycle > 0.5`): on a smooth view it declines by
    /// design, because remapping a view that reads correctly only changes the picture for nothing.
    /// But people read "normalize" as "fit the palette to what is on screen", and reported it as
    /// broken when it sat there declining (2026-09-17, 2026-09-18). The threshold is calibrated
    /// against two regressions in OPPOSITE directions — flat-gray at shallow depth, and "cities at
    /// night" — so moving it to satisfy the request risks both. Asking the question outright costs
    /// one checkbox and leaves the guard alone.
    ///
    /// ⚠Indented under the line above, because it modifies it rather than standing on its own. Not
    /// grayed when normalization is off: this checkbox is itself a way to TURN normalization on.
    pub(crate) fn fit_range_checkbox(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(ui.spacing().indent * 0.5);
            ui.checkbox(&mut self.coloring.normalize_fit, tr!(panel_fit_range))
                .on_hover_text(tr!(panel_fit_range_hover));
        });
    }

    /// Whether the palette is fitted to a range, so that the Scale's curve applies (C2).
    fn scale_applies(&self) -> bool {
        // "Always fit range" makes a mapping on its own (for the orbit trap it is the only one), so
        // it enables this as the guard does.
        self.coloring.normalize_live || self.coloring.normalize || self.coloring.normalize_fit
    }

    /// Scale (coloring plan C2, what "Log color scale" was): the curve the fitted palette takes the
    /// values through. A list in the panel.
    pub(crate) fn scale_row(&mut self, ui: &mut egui::Ui) {
        let live = self.scale_applies();
        // INDENTED, because it is a sub-option of the checkboxes above it — the same idiom the
        // "Click to zoom" / "Factor" pair already uses in this panel. The two marks say different
        // things and both are wanted: graying says "you cannot use this right now", the indent
        // says "this belongs to the line above" — which is the half that tells you WHY, and it
        // says it before you go hunting for a tooltip.
        ui.add_enabled_ui(live, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(ui.spacing().indent * 0.5);
                ui.label(tr!(panel_scale));
                // As wide as the widest name, whichever is chosen, so the row never widens the panel.
                let font = egui::TextStyle::Button.resolve(ui.style());
                let widest = fractadyne_core::coloring::Transfer::ALL
                    .iter()
                    .map(|t| ui.fonts(|f| f.layout_no_wrap(transfer_label(*t), font.clone(), egui::Color32::WHITE).size().x))
                    .fold(0.0_f32, f32::max);
                let sp = ui.spacing();
                let box_w = widest + sp.icon_width + sp.icon_spacing + 2.0 * sp.button_padding.x;
                let r = egui::ComboBox::from_id_salt("panel_scale")
                    .width(box_w)
                    .selected_text(transfer_label(self.coloring.transfer))
                    .show_ui(ui, |ui| {
                        for t in fractadyne_core::coloring::Transfer::ALL {
                            ui.selectable_value(&mut self.coloring.transfer, t, transfer_label(t));
                        }
                    });
                r.response.on_hover_text(if live { tr!(panel_scale_hover) } else { tr!(panel_scale_off_hover) });
            });
        });
    }

    /// Equalize (coloring plan C6): how much histogram coloring is mixed into the fitted palette, a
    /// slider indented under the Scale, drawn the same in the panel and the Color menu. Grayed where
    /// it would change no pixel, with the reason: no normalization (the Scale's), a method other
    /// than Smooth and the orbit trap (it spreads the escape count, or the trap's values: C6c), or
    /// the orbit trap without "Always fit range" (the trap's only fit, `live_norm_window`).
    pub(crate) fn equalize_row(&mut self, ui: &mut egui::Ui) {
        let method = self.coloring.color_method;
        let trap = method == ColorMethod::OrbitTrap;
        let fitted = if trap { self.coloring.normalize_fit } else { self.scale_applies() };
        let live = (method == ColorMethod::Smooth || trap) && fitted;
        ui.add_enabled_ui(live, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(ui.spacing().indent * 0.5);
                ui.label(tr!(panel_equalize));
                let r = ui.add(
                    egui::Slider::new(&mut self.coloring.equalize, crate::slider_range::EQUALIZE)
                        .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                        .custom_parser(|t| t.trim().trim_end_matches('%').trim().parse::<f64>().ok().map(|v| v / 100.0)),
                );
                if slider_double_clicked(&r) {
                    self.coloring.equalize = fractadyne_state::ColoringSpec::default().equalize;
                }
                r.on_hover_text(format!("{}\n\n{}", tr!(panel_equalize_hover), tr!(panel_double_click_reset)))
                    .on_disabled_hover_text(match method {
                        ColorMethod::Smooth => tr!(panel_scale_off_hover),
                        ColorMethod::OrbitTrap => tr!(panel_equalize_trap_off_hover),
                        _ => tr!(panel_equalize_method_hover),
                    });
            });
        });
    }

    /// Scale in the Color menu: a submenu of choices (a list inside a menu would close it).
    pub(crate) fn scale_menu(&mut self, ui: &mut egui::Ui) {
        let live = self.scale_applies();
        ui.add_enabled_ui(live, |ui| {
            ui.menu_button(format!("{}: {}", tr!(panel_scale), transfer_label(self.coloring.transfer)), |ui| {
                for t in fractadyne_core::coloring::Transfer::ALL {
                    if ui.radio(self.coloring.transfer == t, transfer_label(t)).clicked() {
                        self.coloring.transfer = t;
                        ui.close_menu();
                    }
                }
            })
            .response
            .on_hover_text(if live { tr!(panel_scale_hover) } else { tr!(panel_scale_off_hover) });
        });
    }

    /// Right-side control panel (Coloring + Navigation sections) plus the reopen handle shown
    /// when it's hidden. No-op in fullscreen.
    pub(crate) fn draw_right_panel(&mut self, ctx: &egui::Context) {
        // Set again below where the Cycle slider and the light's dial draw this frame.
        self.perf.layout.cycle_slider = None;
        self.perf.layout.light_dial = None;
        if !self.fullscreen && self.dialogs.right_panel_open {
        let panel = egui::SidePanel::right("coloring_panel")
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    // ▶ points toward the right edge the panel collapses to (it's docked right).
                    if ui.small_button("\u{23F5}").on_hover_text(tr!(panel_hide_hover)).clicked() {
                        self.dialogs.right_panel_open = false;
                    }
                    ui.label(egui::RichText::new(tr!(panel_title)).strong());
                });
                ui.separator();
                // Scroll the sections when they don't fit the window height (header stays pinned).
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {

                if self.fractal == FractalKind::Life {
                    // The family's own name (`FractalKind::name`), as the heading always read.
                    egui::CollapsingHeader::new(FractalKind::Life.display_name())
                        .id_salt("fractadyne.panel.life")
                        .default_open(true)
                        .show(ui, |ui| self.life_panel(ui));
                }
                if self.fractal == FractalKind::LSystem {
                    egui::CollapsingHeader::new(FractalKind::LSystem.display_name())
                        .id_salt("fractadyne.panel.lsystem")
                        .default_open(true)
                        .show(ui, |ui| self.lsystem_panel(ui));
                }
                egui::CollapsingHeader::new(tr!(panel_section_navigate)).id_salt("fractadyne.panel.navigate").default_open(true).show(ui, |ui| {
                labelled(ui, &tr!(panel_zoom_speed), |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.render_cfg.zoom_rate, 0.25..=4.0)
                            .suffix("×")
                            .logarithmic(true),
                    )
                })
                .on_hover_text(tr!(panel_zoom_speed_hover));
                ui.checkbox(&mut self.render_cfg.snap_zoom, tr!(panel_snap_zoom))
                    .on_hover_text(tr!(panel_snap_zoom_hover));

                // Click-to-zoom tool: arm a click to dive into the point by a fixed factor. Off by
                // default; drag still pans and Shift/right-drag still box-zoom. Works in the dual
                // view too, on whichever panel the click lands in — where it takes the plain click
                // and Julia pinning moves to Ctrl+click for as long as the tool is armed.
                ui.checkbox(&mut self.click_zoom, tr!(panel_click_zoom))
                    .on_hover_text(tr!(panel_click_zoom_hover));
                ui.add_enabled_ui(self.click_zoom, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("    {}", tr!(panel_click_zoom_factor)));
                        for f in crate::CLICK_ZOOM_FACTORS {
                            ui.selectable_value(
                                &mut self.render_cfg.click_zoom_factor,
                                f,
                                format!("{f:.0}×"),
                            );
                        }
                    });
                    // Snap: the reason the tool is usually hard to aim is that a miss of d pixels
                    // becomes d × factor pixels once the scale shrinks, so at 50× a click that
                    // looked perfect is a third of a panel out. Solving for the nucleus takes the
                    // aim out of it.
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.checkbox(&mut self.click_zoom_snap, tr!(panel_click_zoom_snap))
                            .on_hover_text(tr!(panel_click_zoom_snap_hover));
                    });
                });


                });
                egui::CollapsingHeader::new(tr!(panel_section_coloring)).id_salt("fractadyne.panel.coloring").default_open(true).show(ui, |ui| {
                // ⭐A family without a derivative has no distance estimate
                // (`FormulaCaps::distance_estimate`), so Distance would color the whole view one
                // color: it is grayed out with the reason. A session that already holds it keeps it
                // and says why under the box, rather than switching the method on its own.
                // ⭐Likewise Newton records no orbit statistics, so stripe, triangle, the traps and
                // decomposition would color it one color: each method says what the family lacks.
                // ⭐The list is `method_list`'s, shared with Color ▸ Method (coloring plan B6): the
                // methods under headings, each tooltip saying what it shows and what choosing it
                // costs here.
                let caps = self.fractal.caps();
                labelled(ui, &tr!(panel_method), |ui| {
                    if std::mem::take(&mut self.dialogs.open_method_list) {
                        // The combo box's own id rule (see the fractal dropdown in menus.rs).
                        let popup = ui.make_persistent_id(egui::Id::new("panel_method")).with("popup");
                        ui.memory_mut(|m| m.open_popup(popup));
                    }
                    // ⚠**AS WIDE AS THE WIDEST METHOD, whichever is chosen.** A box that fits its
                    // text grew with "Distance estimate", and with the sheet's button beside it the
                    // row then outgrew the panel by 2 px: the panel widened, the canvas narrowed, and
                    // a switch the list calls a recolor calculated the whole view again (the uitest's
                    // method-switch-cost, 2026-11-13). Sized to the widest name, the row's width
                    // does not depend on the choice.
                    let font = egui::TextStyle::Button.resolve(ui.style());
                    let widest = ColorMethod::ALL
                        .iter()
                        .map(|m| ui.fonts(|f| f.layout_no_wrap(m.label(), font.clone(), egui::Color32::WHITE).size().x))
                        .fold(0.0_f32, f32::max);
                    let sp = ui.spacing();
                    let box_w = (widest + sp.icon_width + sp.icon_spacing + 2.0 * sp.button_padding.x).max(sp.combo_width);
                    let combo = egui::ComboBox::from_id_salt("panel_method")
                        .width(box_w)
                        .selected_text(self.coloring.color_method.label())
                        .height(480.0)
                        .show_ui(ui, |ui| {
                            self.method_list(ui);
                        });
                    if combo.inner.is_none() {
                        // The list is closed: a forced tooltip ends with it.
                        self.dialogs.uitest_method_tooltip = None;
                    }
                    combo.response.on_hover_text(tr!(panel_method_hover));
                    // The method sheet: this view in every method, side by side (B7).
                    if ui.small_button(crate::icons::GALLERY).on_hover_text(tr!(method_sheet_open_hover)).clicked() {
                        self.method_sheet.open = true;
                    }
                });
                if let Some(l) = self.coloring.color_method.lacks(caps) {
                    let note = match l {
                        fractadyne_core::coloring::Lacks::DistanceEstimate => {
                            tr!(panel_method_no_derivative, family = self.fractal.display_name())
                        }
                        fractadyne_core::coloring::Lacks::OrbitStatistics => {
                            tr!(panel_method_no_statistics, family = self.fractal.display_name())
                        }
                    };
                    ui.label(egui::RichText::new(note).small().color(ui.visuals().warn_fg_color));
                }
                // ⭐A coloring hook colors the view in place of the method (B10): said here, under
                // the method, so a method that seems to change little is explained, and a save of the
                // hook's file that does not build is seen without the window open. Truncated, so a
                // long file name never widens the panel (the method row's own rule above).
                if self.color_hook_for(self.fractal).is_some() {
                    let name = self
                        .coloring
                        .hook
                        .path
                        .as_deref()
                        .and_then(|p| p.file_name())
                        .map_or_else(|| tr!(color_hook_panel_unnamed), |s| s.to_string_lossy().into_owned());
                    let accent = ui.visuals().selection.stroke.color;
                    let line = egui::Label::new(egui::RichText::new(tr!(color_hook_panel, name = name)).small().color(accent))
                        .truncate()
                        .sense(egui::Sense::click());
                    if ui.add(line).on_hover_text(tr!(color_hook_panel_hover)).clicked() {
                        self.color_hook_dialog.open = true;
                    }
                    if self.coloring.hook.error.is_some() {
                        let warn = egui::RichText::new(tr!(color_hook_panel_error)).small().color(ui.visuals().warn_fg_color);
                        let hover = format!("{}\n\n{}", tr!(color_hook_error_kept), tr!(color_hook_panel_hover));
                        if ui.add(egui::Label::new(warn).truncate().sense(egui::Sense::click())).on_hover_text(hover).clicked() {
                            self.color_hook_dialog.open = true;
                        }
                    }
                }
                // The Smooth method's band texture (E1): a statistic moves the band edges along its
                // texture. A family with no orbit statistics has nothing to texture them with.
                if self.coloring.color_method == ColorMethod::Smooth && ColorMethod::Stripe.lacks(caps).is_none() {
                    self.band_texture_rows(ui);
                    // ...and its statistic layer (E3), bump (E4) and gloss (E5), on the same terms.
                    self.stat_layer_rows(ui);
                    self.stat_bump_rows(ui);
                    self.stat_gloss_rows(ui);
                }
                // The stripe average's density and tail: the stripe method's, or a stripe texture's,
                // layer's, bump's or gloss's.
                let stripe = fractadyne_core::coloring::BandTexture::Stripe;
                let stripe_used = self.coloring.color_method == ColorMethod::Stripe
                    || (self.coloring.color_method == ColorMethod::Smooth
                        && (self.coloring.band_texture == stripe
                            || self.coloring.stat_layer == stripe
                            || self.coloring.stat_bump == stripe
                            || self.coloring.stat_gloss == stripe));
                if stripe_used {
                    let r = labelled(ui, &tr!(panel_stripe_density), |ui| {
                        ui.add(
                            egui::Slider::new(&mut self.coloring.stripe_freq, crate::slider_range::STRIPE_FREQ)
                                .logarithmic(true),
                        )
                    });
                    if slider_double_clicked(&r) {
                        self.coloring.stripe_freq = fractadyne_state::ColoringSpec::default().stripe_freq;
                    }
                    r.on_hover_text(tr!(panel_double_click_reset));
                    ui.checkbox(&mut self.coloring.stripe_tail, tr!(panel_stripe_tail))
                        .on_hover_text(tr!(panel_stripe_tail_hover));
                    if self.coloring.stripe_tail {
                        let r = labelled(ui, &tr!(panel_tail_length), |ui| {
                            ui.add(
                                egui::Slider::new(&mut self.coloring.stripe_tail_len, 4..=1024)
                                    .logarithmic(true)
                                    .suffix(format!(" {}", tr!(panel_tail_length_unit)))
                                    .custom_formatter(|n, _| crate::grouped_count(n))
                                    .custom_parser(crate::parse_grouped_number),
                            )
                        });
                        if slider_double_clicked(&r) {
                            self.coloring.stripe_tail_len = fractadyne_state::ColoringSpec::default().stripe_tail_len;
                        }
                        r.on_hover_text(format!("{}\n\n{}", tr!(panel_tail_length_hover), tr!(panel_double_click_reset)));
                    }
                }
                // The trap's shape beside the method that uses it. When only the inside uses the
                // trap, it is with the inside's rows at the end of the section.
                if self.coloring.color_method == ColorMethod::OrbitTrap {
                    self.trap_shape_row(ui);
                }
                // The palette, and how it looks.
                ui.separator();
                // ⭐**The collapsed label NAMES a saved gradient rather than saying "Custom".** The
                // whole reason to save one is to refer to it later; a control that can only report
                // the slot it occupies makes the library invisible from here.
                let live_saved = self.live_saved_gradient();
                let pal_name: String = if self.coloring.use_binary {
                    tr!(menu_color_binary)
                } else if self.coloring.use_duotone {
                    tr!(menu_color_duotone)
                } else if let Some(i) = live_saved {
                    self.saved_gradients[i].name.clone()
                } else if self.coloring.use_custom_palette {
                    tr!(menu_color_custom)
                } else {
                    fractadyne_color::PRESETS[self.coloring.palette_idx].name.to_string()
                };
                // The chosen palette's swatch, not a hover preview's: the box names the choice.
                let pal_swatch = self.chosen_gradient().bake(fractadyne_color::segment::LUT_SIZE);
                labelled(ui, &tr!(panel_palette), |ui| {
                    if std::mem::take(&mut self.dialogs.open_palette_dropdown) {
                        // The combo box's own id rule (see the fractal dropdown in menus.rs).
                        let popup = ui.make_persistent_id(egui::Id::new("panel_palette")).with("popup");
                        ui.memory_mut(|m| m.open_popup(popup));
                        self.coloring.palette_preview.reopened();
                    }
                    let combo = egui::ComboBox::from_id_salt("panel_palette")
                        .selected_text(crate::ui::menus::palette_box_text(ui, &pal_name))
                        // Tall enough for a good run of the presets; it scrolls past that.
                        .height(430.0)
                        .show_ui(ui, |ui| {
                            let is_preset = !self.coloring.use_custom_palette && !self.coloring.use_duotone && !self.coloring.use_binary;
                            // The hover preview, as in Color ▸ Palette: reported from inside the
                            // open list, so closing it any way at all reverts the view.
                            use crate::palette_preview::PaletteRow;
                            self.coloring.palette_preview.picker_open();
                            // Each with its swatch, as Color ▸ Palette shows them: the same
                            // control on two surfaces (`palette_menu_row`).
                            for (i, p) in fractadyne_color::PRESETS.iter().enumerate() {
                                Self::palette_preset_heading(ui, i);
                                let g = fractadyne_color::segment::Gradient::from_stops(p.name, p.stops);
                                let chosen = is_preset && self.coloring.palette_idx == i;
                                let row = Self::palette_menu_row(ui, &g, p.name, chosen);
                                self.reveal_chosen_palette(ui, &row, chosen);
                                self.coloring.palette_preview.row(PaletteRow::Preset(i), row.rect, row.hovered);
                                if row.clicked {
                                    self.coloring.palette_idx = i;
                                    self.coloring.use_custom_palette = false;
                                    self.coloring.use_duotone = false;
                                    self.coloring.use_binary = false;
                                }
                            }
                            // The user's saved gradients, on the same footing as the presets — the
                            // Color ▸ Palette menu carries the identical list, and the two are the
                            // same control on two surfaces.
                            if !self.saved_gradients.is_empty() {
                                ui.separator();
                                let mut apply = None;
                                for (i, sg) in self.saved_gradients.iter().enumerate() {
                                    let g = crate::segments_to_gradient(&sg.name, &sg.segment);
                                    let chosen = live_saved == Some(i);
                                    let row = Self::palette_menu_row(ui, &g, &sg.name, chosen);
                                    self.reveal_chosen_palette(ui, &row, chosen);
                                    self.coloring.palette_preview.row(PaletteRow::Saved(i), row.rect, row.hovered);
                                    if row.clicked {
                                        apply = Some(i);
                                    }
                                }
                                if let Some(i) = apply {
                                    self.apply_saved_gradient(i);
                                }
                                ui.separator();
                            }
                            // ⚠Ticked only when the custom palette is not one of the saved entries
                            // above, or picking a saved gradient would check two rows at once.
                            if ui.selectable_label(self.coloring.use_custom_palette && live_saved.is_none(), format!("{} {}", tr!(menu_color_custom), crate::icons::EDIT)).clicked() {
                                if self.coloring.custom_palette.is_empty() {
                                    self.coloring.custom_palette = self.preset_as_stops(self.coloring.palette_idx);
                                }
                                self.coloring.use_custom_palette = true;
                                self.coloring.use_duotone = false;
                                self.coloring.use_binary = false;
                            }
                            if ui.selectable_label(self.coloring.use_duotone, tr!(menu_color_duotone)).clicked() {
                                self.coloring.use_duotone = true;
                                self.coloring.use_custom_palette = false;
                                self.coloring.use_binary = false;
                            }
                            if ui
                                .selectable_label(self.coloring.use_binary, tr!(menu_color_binary))
                                .on_hover_text(tr!(menu_color_binary_hover))
                                .clicked()
                            {
                                self.coloring.use_binary = true;
                                self.coloring.use_custom_palette = false;
                                self.coloring.use_duotone = false;
                            }
                        });
                    crate::ui::menus::paint_palette_box_swatch(ui, combo.response.rect, &pal_swatch);
                    // The palette browser: every palette over this view, side by side, by tag.
                    if ui.small_button(crate::icons::GALLERY).on_hover_text(tr!(menu_color_browse_hover)).clicked() {
                        self.palette_browser.open_for(crate::ui::palette_browser::BrowserTarget::Outside);
                    }
                });
                if self.coloring.use_duotone || self.coloring.use_binary {
                    // Two shared colors. (Binary: interior/exterior; duotone: shadow/highlight.)
                    let (lo_lbl, hi_lbl) = if self.coloring.use_binary {
                        (tr!(panel_in_set), tr!(panel_out_of_set))
                    } else {
                        (tr!(panel_shadow), tr!(panel_highlight))
                    };
                    // ⭐The hex sits under each swatch rather than beside it: the panel is 298 px
                    // and two swatches with their labels already fill the row, so an inline value
                    // would wrap. It is a readout here, not a field — the gradient editor is where
                    // a color gets typed, and duplicating an editable field in two places would
                    // duplicate the "which one wins" question with it.
                    // The app's swatch, which shows a display-space color as it renders (egui's took it
                    // for linear light and drew it lighter).
                    ui.horizontal(|ui| {
                        if let Some(c) = crate::color_swatch(ui, egui::Id::new("panel_duotone_lo"), self.coloring.duotone_lo, &lo_lbl) {
                            self.coloring.duotone_lo = c;
                        }
                        ui.label(lo_lbl.as_str());
                        if let Some(c) = crate::color_swatch(ui, egui::Id::new("panel_duotone_hi"), self.coloring.duotone_hi, &hi_lbl) {
                            self.coloring.duotone_hi = c;
                        }
                        ui.label(hi_lbl.as_str());
                    });
                    ui.horizontal(|ui| {
                        for c in [self.coloring.duotone_lo, self.coloring.duotone_hi] {
                            let [r, g, b] = crate::rgb_bytes(c);
                            ui.label(
                                egui::RichText::new(format!("{}  {r} {g} {b}", crate::hex_of(c)))
                                    .monospace()
                                    .weak()
                                    .small(),
                            );
                        }
                    });
                } else if ui.button(tr!(menu_color_edit_gradient)).clicked() {
                    // Only OPENS it: the editor starts from the gradient on screen, and takes its
                    // Cancel point first (`palette_editor_window`).
                    self.coloring.palette_editor_open = true;
                }
                // The palette's brightness, contrast and saturation (user request 2026-11-15): the
                // palette as it colors the view, its stops untouched; a recolor, never a pass. A
                // double-click puts one back (no reset button: the rows stay as wide as the others).
                self.palette_adjust_rows(ui);
                // The palette's anti-aliasing (coloring plan C8): a recolor, no pass.
                ui.checkbox(&mut self.coloring.palette_aa, tr!(panel_palette_aa)).on_hover_text(tr!(panel_palette_aa_hover));
                // Blend in OKLab (coloring plan C7): every palette baked with its blends in OKLab; a
                // recolor, no pass.
                ui.checkbox(&mut self.coloring.palette_oklab, tr!(panel_palette_oklab)).on_hover_text(tr!(panel_palette_oklab_hover));
                // Where a pixel's value lands on the palette: rate, divisor, offset, the fitted
                // range, and the animation that moves the offset.
                ui.separator();
                // Every slider here resets on a double-click, to a new session's value (user,
                // 2026-11-20); Color ▸ Reset coloring settings puts the section's numbers back at once.
                let d = fractadyne_state::ColoringSpec::default();
                let r = labelled(ui, &tr!(panel_cycle), |ui| {
                    ui.add(egui::Slider::new(&mut self.coloring.cycle, crate::slider_range::CYCLE))
                });
                self.perf.layout.cycle_slider = ui.clip_rect().contains_rect(r.rect).then_some(r.rect);
                if slider_double_clicked(&r) {
                    self.coloring.cycle = d.cycle;
                }
                r.on_hover_text(tr!(panel_double_click_reset));
                // "Divide iterations": under Cycle, because it scales what Cycle chose. ⭐Grayed out
                // where it would change no pixel (another method, or a range fit), with a tooltip
                // that says why — a live control that does nothing reads as a broken one.
                let applies = self.iter_div_applies();
                let per_palette = crate::iterations_per_palette(self.color_cycle()).map_or(0, |n| n.round() as u64);
                // ⚠The readout is the FIXED palette's. While "Normalize deep colors" is remapping
                // the view, the screen shows the fitted palette instead, so the tooltip says so
                // rather than state a rate the picture is not using.
                let mut hover = tr!(panel_iter_div_hover, count = per_palette);
                if (0..=self.dual as usize).any(|vi| self.live_norm_window(vi).is_some()) {
                    hover = format!("{hover}\n\n{}", tr!(panel_iter_div_normalized_note));
                }
                hover = format!("{hover}\n\n{}", tr!(panel_double_click_reset));
                let r = labelled(ui, &tr!(panel_iter_div), |ui| {
                    ui.add_enabled(
                        applies,
                        egui::Slider::new(&mut self.coloring.iter_div, 1..=fractadyne_state::ITER_DIV_MAX)
                            .logarithmic(true),
                    )
                });
                if slider_double_clicked(&r) {
                    self.coloring.iter_div = d.iter_div;
                }
                r.on_hover_text(hover)
                .on_disabled_hover_text(if self.coloring.color_method == crate::ColorMethod::Smooth {
                    tr!(panel_iter_div_fit_hover)
                } else {
                    tr!(panel_iter_div_method_hover)
                });
                let r = labelled(ui, &tr!(panel_offset), |ui| {
                    ui.add(egui::Slider::new(&mut self.coloring.offset, crate::slider_range::OFFSET))
                });
                if slider_double_clicked(&r) {
                    self.coloring.offset = d.offset;
                }
                r.on_hover_text(tr!(panel_double_click_reset));
                // How the palette continues past its end, and the bands a cycle is cut into (C2).
                labelled(ui, &tr!(panel_wrap), |ui| {
                    egui::ComboBox::from_id_salt("panel_wrap")
                        .selected_text(wrap_label(self.coloring.wrap))
                        .show_ui(ui, |ui| {
                            for w in fractadyne_core::coloring::Wrap::ALL {
                                ui.selectable_value(&mut self.coloring.wrap, w, wrap_label(w));
                            }
                        })
                        .response
                        .on_hover_text(tr!(panel_wrap_hover));
                });
                let r = labelled(ui, &tr!(panel_steps), |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.coloring.steps, 0..=fractadyne_core::coloring::MAX_STEPS)
                            .custom_formatter(|n, _| if n < 0.5 { tr!(panel_steps_off) } else { format!("{n:.0}") }),
                    )
                });
                if slider_double_clicked(&r) {
                    self.coloring.steps = d.steps;
                }
                r.on_hover_text(format!("{}\n\n{}", tr!(panel_steps_hover), tr!(panel_double_click_reset)));
                // ⭐Normalization FIRST, then the mapping it selects. The Scale is read only
                // inside the normalized path, so with normalization off it changed nothing — and
                // it sat ABOVE the switch that gives it meaning.
                ui.checkbox(&mut self.coloring.normalize_live, tr!(menu_color_normalize))
                    .on_hover_text(tr!(panel_normalize_hover));
                self.fit_range_checkbox(ui);
                self.scale_row(ui);
                self.equalize_row(ui);
                // C11: the Julia pane on the set's map. Only where there is a Julia pane, and grayed
                // where it would change no pixel (another method: each plane colors its own values).
                if self.dual {
                    ui.add_enabled(
                        self.coloring.color_method == ColorMethod::Smooth,
                        egui::Checkbox::new(&mut self.coloring.julia_linked, tr!(panel_julia_linked)),
                    )
                    .on_hover_text(tr!(panel_julia_linked_hover))
                    .on_disabled_hover_text(tr!(panel_julia_linked_method_hover));
                }
                labelled(ui, &tr!(panel_animate), |ui| {
                    egui::ComboBox::from_id_salt("panel_animate")
                        .selected_text(self.anim.palette_anim.name())
                        .show_ui(ui, |ui| {
                            for m in PaletteAnim::ALL {
                                ui.selectable_value(&mut self.anim.palette_anim, m, m.name());
                            }
                        });
                });
                let r = labelled(ui, &tr!(panel_anim_speed), |ui| {
                    ui.add_enabled(
                        self.anim.palette_anim != PaletteAnim::Off || self.anim_speed_in_use(),
                        egui::Slider::new(&mut self.anim.palette_anim_speed, 0.01..=2.0)
                            .suffix("/s")
                            .logarithmic(true),
                    )
                });
                if slider_double_clicked(&r) {
                    self.anim.palette_anim_speed = fractadyne_state::SessionState::default().palette_anim_speed;
                }
                r.on_hover_text(format!("{}\n\n{}", tr!(panel_anim_speed_hover), tr!(panel_double_click_reset)));
                if self.anim.palette_anim == PaletteAnim::Random && ui.button(tr!(panel_shuffle_gradient)).clicked()
                {
                    self.anim.random_palette.reshuffle();
                }
                // The inside: what colors it, the trap's shape when only the inside uses the trap,
                // its palette and fit. Last, so the main palette never moves when the inside
                // changes (user, 2026-11-15 and 2026-11-20).
                ui.separator();
                self.inside_rows(ui);
                if self.coloring.color_method != ColorMethod::OrbitTrap
                    && self.coloring.interior_of(self.coloring.color_method) == crate::InteriorMode::Trap
                {
                    self.trap_shape_row(ui);
                } else if self.coloring.color_method == ColorMethod::OrbitTrap
                    && self.coloring.interior_of(ColorMethod::OrbitTrap) == crate::InteriorMode::Trap
                {
                    // The trap method colors the inside by its trap: the same, or one of the inside's own.
                    self.inside_trap_rows(ui);
                }
                self.inside_palette_rows(ui);
                });
                // Iterations, supersampling, relief and glow are escape-time settings: a Life
                // universe has none of them.
                if self.fractal.is_escape_time() {
                egui::CollapsingHeader::new(tr!(panel_section_quality)).id_salt("fractadyne.panel.quality").default_open(true).show(ui, |ui| {
                ui.checkbox(&mut self.render_cfg.auto_iter, tr!(panel_auto_iter));
                let label = if self.render_cfg.auto_iter { tr!(panel_iterations_base) } else { tr!(panel_iterations) };
                labelled(ui, &label, |ui| {
                    ui.add(
                        egui::Slider::new(
                            &mut self.render_cfg.max_iter,
                            64..=crate::MAX_ITER_LIMIT,
                        )
                        .logarithmic(true)
                        // Comma-grouped (`10,000,000`) — a seven-digit count is easy to misread by
                        // a factor of ten, which is exactly the setting that can stall the app.
                        .custom_formatter(|n, _| crate::grouped_count(n))
                        .custom_parser(crate::parse_grouped_number),
                    )
                })
                .on_hover_text(tr!(panel_iterations_hover));
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
                            "⚠ {}",
                            tr!(panel_res_limited, count = settled_iter),
                        ))
                        .small()
                        .color(accent),
                    );
                }
                ui.separator();
                labelled(ui, &tr!(panel_anti_alias), |ui| {
                    egui::ComboBox::from_id_salt("panel_anti_alias")
                        .selected_text(match self.render_cfg.aa {
                            1 => tr!(panel_anti_alias_off),
                            2 => "2×".to_string(),
                            3 => "3×".to_string(),
                            4 => "4×".to_string(),
                            _ => "8×".to_string(),
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.render_cfg.aa, 1, tr!(panel_anti_alias_off));
                            ui.selectable_value(&mut self.render_cfg.aa, 2, "2×");
                            ui.selectable_value(&mut self.render_cfg.aa, 3, "3×");
                            ui.selectable_value(&mut self.render_cfg.aa, 4, "4×");
                            ui.selectable_value(&mut self.render_cfg.aa, 8, "8×");
                        })
                        .response
                        .on_hover_text(tr!(panel_anti_alias_hover));
                });

                });
                let effects = egui::CollapsingHeader::new(tr!(panel_section_effects))
                    .id_salt("fractadyne.panel.effects")
                    .default_open(false)
                    .open(self.dialogs.uitest_effects_open)
                    .show(ui, |ui| {
                // ⭐Relief lighting and the glow read the distance estimate's slope and value, which
                // a family without a derivative does not have: both are grayed out there with the
                // reason. A switch already on stays on (it comes back with a family that has one),
                // and the note says it does nothing here.
                let has_de = self.fractal.caps().distance_estimate;
                let no_de = tr!(coloring_needs_derivative, family = self.fractal.display_name());
                if !has_de && (self.effects.light || self.effects.de || self.effects.edge || self.effects.grid) {
                    ui.label(
                        egui::RichText::new(tr!(panel_effects_no_de, family = self.fractal.display_name()))
                            .small()
                            .color(ui.visuals().warn_fg_color),
                    );
                }
                ui.add_enabled(has_de, egui::Checkbox::new(&mut self.effects.light, tr!(panel_relief_lighting)))
                    .on_hover_text(tr!(panel_relief_lighting_hover))
                    .on_disabled_hover_text(&no_de);
                ui.add_enabled_ui(has_de && self.effects.light, |ui| {
                    let d = fractadyne_state::ColoringSpec::default();
                    // The dial first, the slider shortened by its width: the number stays in line
                    // with the rows below.
                    let (dial, r) = labelled(ui, &tr!(panel_light_angle), |ui| {
                        let dial = light_dial(ui, &mut self.effects.light_angle, d.light_angle);
                        let spacing = ui.spacing_mut();
                        spacing.slider_width = (spacing.slider_width - dial.rect.width() - spacing.item_spacing.x).max(40.0);
                        let r = ui.add(
                            egui::Slider::new(&mut self.effects.light_angle, crate::slider_range::LIGHT_ANGLE)
                                .suffix("°")
                                .fixed_decimals(1),
                        );
                        (dial, r)
                    });
                    self.perf.layout.light_dial = ui.clip_rect().contains_rect(dial.rect).then_some(dial.rect);
                    dial.on_hover_text(format!("{}\n\n{}", tr!(panel_light_angle_dial_hover), tr!(panel_double_click_reset)));
                    if slider_double_clicked(&r) {
                        self.effects.light_angle = d.light_angle;
                    }
                    r.on_hover_text(tr!(panel_double_click_reset));
                    let r = labelled(ui, &tr!(panel_relief), |ui| {
                        ui.add(
                            egui::Slider::new(&mut self.effects.light_height, crate::slider_range::LIGHT_HEIGHT)
                                .logarithmic(true),
                        )
                    });
                    if slider_double_clicked(&r) {
                        self.effects.light_height = d.light_height;
                    }
                    r.on_hover_text(format!("{}\n\n{}", tr!(panel_relief_hover), tr!(panel_double_click_reset)));
                    // The surface (coloring plan C5): its slope from the distance, and a highlight on
                    // it. Each row grayed out without the slope says why.
                    ui.checkbox(&mut self.effects.light_steep, tr!(panel_relief_steep))
                        .on_hover_text(tr!(panel_relief_steep_hover));
                    let steep = self.effects.light_steep;
                    let r = ui
                        .add_enabled_ui(steep, |ui| {
                            labelled(ui, &tr!(panel_relief_steep_px), |ui| {
                                ui.add(
                                    egui::Slider::new(&mut self.effects.light_steep_px, crate::slider_range::RELIEF_STEEP_PX)
                                        .logarithmic(true)
                                        .suffix(" px"),
                                )
                            })
                        })
                        .inner;
                    if slider_double_clicked(&r) {
                        self.effects.light_steep_px = d.light_steep_px;
                    }
                    r.on_hover_text(format!("{}\n\n{}", tr!(panel_relief_steep_px_hover), tr!(panel_double_click_reset)))
                        .on_disabled_hover_text(tr!(panel_relief_steep_off));
                    let r = ui
                        .add_enabled_ui(steep, |ui| {
                            labelled(ui, &tr!(panel_relief_highlight), |ui| {
                                ui.add(egui::Slider::new(&mut self.effects.light_highlight, crate::slider_range::RELIEF_HIGHLIGHT))
                            })
                        })
                        .inner;
                    if slider_double_clicked(&r) {
                        self.effects.light_highlight = d.light_highlight;
                    }
                    r.on_hover_text(format!("{}\n\n{}", tr!(panel_relief_highlight_hover), tr!(panel_double_click_reset)))
                        .on_disabled_hover_text(tr!(panel_relief_highlight_off));
                    let r = ui
                        .add_enabled_ui(steep, |ui| {
                            labelled(ui, &tr!(panel_relief_shininess), |ui| {
                                ui.add(
                                    egui::Slider::new(&mut self.effects.light_shininess, crate::slider_range::RELIEF_SHININESS)
                                        .logarithmic(true),
                                )
                            })
                        })
                        .inner;
                    if slider_double_clicked(&r) {
                        self.effects.light_shininess = d.light_shininess;
                    }
                    r.on_hover_text(format!("{}\n\n{}", tr!(panel_relief_shininess_hover), tr!(panel_double_click_reset)))
                        .on_disabled_hover_text(tr!(panel_relief_highlight_off));
                    ui.add_enabled_ui(steep, |ui| {
                        labelled(ui, &tr!(panel_relief_highlight_color), |ui| {
                            if let Some(c) = crate::color_swatch(
                                ui,
                                egui::Id::new("panel_relief_highlight_color"),
                                self.effects.light_highlight_color,
                                &tr!(panel_relief_highlight_color_hover),
                            ) {
                                self.effects.light_highlight_color = c;
                            }
                        })
                    })
                    .response
                    .on_disabled_hover_text(tr!(panel_relief_highlight_off));
                    ui.checkbox(&mut self.effects.light_anim, tr!(panel_rotate_light))
                        .on_hover_text(tr!(panel_rotate_light_hover));
                });
                ui.add_enabled(has_de, egui::Checkbox::new(&mut self.effects.de, tr!(panel_distance_glow)))
                    .on_hover_text(tr!(panel_distance_glow_hover))
                    .on_disabled_hover_text(&no_de);
                ui.add_enabled_ui(has_de && self.effects.de, |ui| {
                    let d = fractadyne_state::ColoringSpec::default();
                    let r = labelled(ui, &tr!(panel_glow), |ui| {
                        ui.add(egui::Slider::new(&mut self.effects.de_strength, crate::slider_range::GLOW_STRENGTH))
                    });
                    if slider_double_clicked(&r) {
                        self.effects.de_strength = d.de_strength;
                    }
                    r.on_hover_text(tr!(panel_double_click_reset));
                    let r = labelled(ui, &tr!(panel_band_width), |ui| {
                        ui.add(
                            egui::Slider::new(&mut self.effects.de_width, crate::slider_range::GLOW_WIDTH)
                                .logarithmic(true),
                        )
                    });
                    if slider_double_clicked(&r) {
                        self.effects.de_width = d.de_width;
                    }
                    r.on_hover_text(format!("{}\n\n{}", tr!(panel_band_width_hover), tr!(panel_double_click_reset)));
                    ui.checkbox(&mut self.effects.de_anim, tr!(panel_animate_glow))
                        .on_hover_text(tr!(panel_animate_glow_hover));
                });
                // Edge shading (coloring plan C3): it reads the distance estimate, as the glow does.
                ui.add_enabled(has_de, egui::Checkbox::new(&mut self.effects.edge, tr!(panel_edge_shading)))
                    .on_hover_text(tr!(panel_edge_shading_hover))
                    .on_disabled_hover_text(&no_de);
                ui.add_enabled_ui(has_de && self.effects.edge, |ui| {
                    let d = fractadyne_state::ColoringSpec::default();
                    let r = labelled(ui, &tr!(panel_edge_width), |ui| {
                        ui.add(
                            egui::Slider::new(&mut self.effects.edge_width, crate::slider_range::EDGE_WIDTH)
                                .logarithmic(true)
                                .suffix(" px"),
                        )
                    });
                    if slider_double_clicked(&r) {
                        self.effects.edge_width = d.edge_width;
                    }
                    r.on_hover_text(format!("{}\n\n{}", tr!(panel_edge_width_hover), tr!(panel_double_click_reset)));
                    let r = labelled(ui, &tr!(panel_edge_strength), |ui| {
                        ui.add(egui::Slider::new(&mut self.effects.edge_strength, crate::slider_range::EDGE_STRENGTH))
                    });
                    if slider_double_clicked(&r) {
                        self.effects.edge_strength = d.edge_strength;
                    }
                    r.on_hover_text(tr!(panel_double_click_reset));
                    labelled(ui, &tr!(panel_edge_curve), |ui| {
                        egui::ComboBox::from_id_salt("panel_edge_curve")
                            .selected_text(edge_curve_label(self.effects.edge_curve))
                            .show_ui(ui, |ui| {
                                for c in fractadyne_core::coloring::EdgeCurve::ALL {
                                    ui.selectable_value(&mut self.effects.edge_curve, c, edge_curve_label(c));
                                }
                            })
                            .response
                            .on_hover_text(tr!(panel_edge_curve_hover));
                    });
                    // The color, or the inside's: the set's edge smoothed by the distance.
                    labelled(ui, &tr!(panel_edge_color), |ui| {
                        let inside = self.effects.edge_inside;
                        ui.add_enabled_ui(!inside, |ui| {
                            if let Some(c) = crate::color_swatch(ui, egui::Id::new("panel_edge_color"), self.effects.edge_color, &tr!(panel_edge_color_hover)) {
                                self.effects.edge_color = c;
                            }
                        });
                        ui.checkbox(&mut self.effects.edge_inside, tr!(panel_edge_inside))
                            .on_hover_text(tr!(panel_edge_inside_hover));
                    });
                });
                // Equipotential lines (coloring plan C10): they read the distance estimate too.
                ui.add_enabled(has_de, egui::Checkbox::new(&mut self.effects.grid, tr!(panel_equipotentials)))
                    .on_hover_text(tr!(panel_equipotentials_hover))
                    .on_disabled_hover_text(&no_de);
                ui.add_enabled_ui(has_de && self.effects.grid, |ui| {
                    let d = fractadyne_state::ColoringSpec::default();
                    let r = labelled(ui, &tr!(panel_equipotential_spacing), |ui| {
                        ui.add(
                            egui::Slider::new(&mut self.effects.grid_spacing, crate::slider_range::GRID_SPACING)
                                .logarithmic(true)
                                .suffix(" px"),
                        )
                    });
                    if slider_double_clicked(&r) {
                        self.effects.grid_spacing = d.equipotential_spacing;
                    }
                    r.on_hover_text(format!("{}\n\n{}", tr!(panel_equipotential_spacing_hover), tr!(panel_double_click_reset)));
                    let r = labelled(ui, &tr!(panel_equipotential_width), |ui| {
                        ui.add(
                            egui::Slider::new(&mut self.effects.grid_width, crate::slider_range::GRID_WIDTH)
                                .logarithmic(true)
                                .suffix(" px"),
                        )
                    });
                    if slider_double_clicked(&r) {
                        self.effects.grid_width = d.equipotential_width;
                    }
                    r.on_hover_text(format!("{}\n\n{}", tr!(panel_equipotential_width_hover), tr!(panel_double_click_reset)));
                    let r = labelled(ui, &tr!(panel_equipotential_strength), |ui| {
                        ui.add(egui::Slider::new(&mut self.effects.grid_strength, crate::slider_range::GRID_STRENGTH))
                    });
                    if slider_double_clicked(&r) {
                        self.effects.grid_strength = d.equipotential_strength;
                    }
                    r.on_hover_text(tr!(panel_double_click_reset));
                    labelled(ui, &tr!(panel_equipotential_color), |ui| {
                        if let Some(c) = crate::color_swatch(ui, egui::Id::new("panel_equipotential_color"), self.effects.grid_color, &tr!(panel_equipotential_color_hover)) {
                            self.effects.grid_color = c;
                        }
                    });
                });
                });
                if self.dialogs.uitest_effects_scroll && self.dialogs.uitest_effects_open == Some(true) {
                    effects.header_response.scroll_to_me(Some(egui::Align::Min));
                }
                } // escape-time sections
                // The orbit overlay's options lived as an indented block inside the View MENU
                // until 2026-08-13 (UI review: a panel's worth of controls in a dropdown). The
                // toggle is mirrored in Tools ▸ Orbit overlay.
                egui::CollapsingHeader::new(tr!(panel_section_overlays)).id_salt("fractadyne.panel.overlays").default_open(false).show(ui, |ui| {
                ui.checkbox(&mut self.anim.show_orbits, tr!(menu_tools_orbits))
                    .on_hover_text(tr!(panel_orbits_hover));
                ui.add_enabled_ui(self.anim.show_orbits, |ui| {
                    ui.checkbox(&mut self.anim.orbit_normalize, tr!(menu_tools_orbits_normalize))
                        .on_hover_text(tr!(menu_tools_orbits_normalize_hover));
                    ui.checkbox(&mut self.anim.orbit_anim, tr!(panel_orbit_anim))
                        .on_hover_text(tr!(panel_orbit_anim_hover));
                    labelled(ui, &tr!(panel_orbit_speed), |ui| {
                        ui.add_enabled(
                            self.anim.orbit_anim,
                            egui::Slider::new(&mut self.anim.orbit_anim_speed, 1.0..=40.0)
                                .suffix("/s"),
                        )
                    });
                });
                ui.checkbox(&mut self.dialogs.minimap, tr!(menu_view_minimap))
                    .on_hover_text(tr!(panel_minimap_hover));
                });
                // ADVANCED, closed by default. Everything a first-time user should not meet
                // while looking for how to zoom: the accelerators (whose names - BLA, series
                // approximation, glitch correction - mean nothing without the theory) and the
                // performance-tuning sliders that used to sit inside Navigation. "Live render
                // budget" and "Min motion resolution" are arguably MORE obscure than BLA: BLA can
                // be ignored safely, whereas a mis-set budget changes behavior with no clue why.
                egui::CollapsingHeader::new(tr!(panel_section_advanced))
                    .id_salt("fractadyne.panel.advanced")
                    .default_open(false)
                    .open(self.dialogs.uitest_advanced_open)
                    .show(ui, |ui| {
                ui.label(egui::RichText::new(tr!(panel_accelerators)).weak().small());
                ui.checkbox(&mut self.render_cfg.use_bla, tr!(panel_bla))
                    .on_hover_text(tr!(panel_bla_hover));
                ui.checkbox(&mut self.render_cfg.series_approx, tr!(panel_series_approx))
                    .on_hover_text(tr!(panel_series_approx_hover));
                ui.checkbox(&mut self.render_cfg.glitch_correct, tr!(panel_glitch_correct))
                    .on_hover_text(tr!(panel_glitch_correct_hover));
                ui.separator();
                ui.label(egui::RichText::new(tr!(panel_performance_tuning)).weak().small());
                // Auto-zoom (autopilot) dive limit, edited in decimal orders (1eN×) but stored as log2.
                let mut dive_log10 = self.autopilot.dive_log2 / std::f64::consts::LOG2_10;
                if labelled(ui, &tr!(panel_dive_limit), |ui| {
                    ui.add(
                        egui::Slider::new(&mut dive_log10, 30.0..=5000.0)
                            .logarithmic(true)
                            .custom_formatter(|n, _| format!("1e{n:.0}×")),
                    )
                })
                    .on_hover_text(tr!(panel_dive_limit_hover))
                    .changed()
                {
                    self.autopilot.dive_log2 = dive_log10 * std::f64::consts::LOG2_10;
                }
                // What the auto-zoom puts first. Either way the screen never shows a frame that
                // has not been proven to hold a picture (design/verified-present.md).
                labelled(ui, &tr!(panel_autopilot_priority), |ui| {
                    use crate::autopilot::AutopilotPriority as P;
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.autopilot.priority, P::Speed, tr!(menu_tools_priority_speed))
                            .on_hover_text(tr!(panel_priority_speed_hover));
                        ui.selectable_value(&mut self.autopilot.priority, P::Quality, tr!(menu_tools_priority_quality))
                            .on_hover_text(tr!(panel_priority_quality_hover));
                    });
                });
                // What the auto-zoom aims at.
                labelled(ui, &tr!(panel_autopilot_target), |ui| {
                    use crate::autopilot::AutopilotTarget as T;
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.autopilot.target, T::Detail, tr!(menu_tools_target_detail))
                            .on_hover_text(tr!(panel_target_detail_hover));
                        ui.selectable_value(&mut self.autopilot.target, T::Misiurewicz, tr!(menu_tools_target_misiurewicz))
                            .on_hover_text(tr!(panel_target_misiurewicz_hover));
                    });
                });

                // Live-render work budget: detail-vs-speed for the deep-zoom preview.
                labelled(ui, &tr!(panel_work_budget), |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.render_cfg.work_budget_scale, 0.25..=8.0)
                            .suffix("×")
                            .logarithmic(true),
                    )
                })
                .on_hover_text(tr!(panel_work_budget_hover));

                // Motion sharpness floor: cap how pixelated a continuous deep zoom is allowed to get.
                labelled(ui, &tr!(panel_min_motion_res), |ui| {
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
                .on_hover_text(tr!(panel_min_motion_res_hover));

                // Another graphics card for the settle's supersampling (design/multi-gpu-live.md L2).
                self.second_gpu_row(ui);

                // KF-style stepping: hold the last detailed frame (geometrically tracked) through
                // motion instead of rendering coarse intermediate frames.
                ui.checkbox(&mut self.render_cfg.finish_sound, tr!(panel_finish_sound))
                    .on_hover_text(tr!(panel_finish_sound_hover));
                ui.checkbox(&mut self.render_cfg.prefer_detail, tr!(panel_prefer_detail))
                    .on_hover_text(tr!(panel_prefer_detail_hover));
                });
                if self.dialogs.uitest_advanced_open == Some(false) {
                    self.dialogs.uitest_advanced_open = None; // closed now; clicks rule it again
                }
                // Per-fractal info, LAST and closed by default: it is reference material, not a
                // control, and it was previously the panel's top section - the most valuable space in
                // the panel spent on something read once. Its header was also `self.fractal.name()`,
                // i.e. literally "Mandelbrot", which reads as a formula SELECTOR next to the toolbar
                // dropdown that actually is one.
                let info = self.fractal.info();
                let custom_src = (self.fractal == FractalKind::Custom)
                    .then(|| self.custom.as_ref().map(|c| c.source.clone()))
                    .flatten();
                egui::CollapsingHeader::new(tr!(panel_section_about, name = self.fractal.display_name()))
                    // Keyed by the family, as the header's text used to key it: each remembers its own state.
                    .id_salt(("fractadyne.panel.about", self.fractal.name()))
                    .default_open(false)
                    .show(ui, |ui| {
                        match &custom_src {
                            Some(src) => ui.monospace(src),
                            None => ui.monospace(self.fractal.formula()),
                        };
                        ui.add_space(4.0);
                        ui.label(self.fractal.about());
                        if !info.reference.is_empty() {
                            ui.add_space(4.0);
                            ui.hyperlink_to(format!("{} \u{2197}", tr!(panel_about_reference)), info.reference);
                        }
                    });
                // Performance section, docked at the bottom of this same panel
                // (toggle via the Perf button or the View menu).
                if self.perf.enabled {
                    egui::CollapsingHeader::new(tr!(panel_section_performance))
                        .id_salt("fractadyne.panel.performance")
                        .default_open(true)
                        .show(ui, |ui| {
                            self.perf_section(ui);
                        });
                }
                });
            });
            self.perf.layout.right_panel = Some(panel.response.rect);
        } else {
            self.perf.layout.right_panel = None;
        }

        // Reopen handle when the control panel is hidden (and not fullscreen).
        if !self.fullscreen && !self.dialogs.right_panel_open {
            egui::Area::new(egui::Id::new("panel_reopen"))
                .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 36.0))
                .show(ctx, |ui| {
                    if ui.button("\u{2630}").on_hover_text(tr!(panel_show_hover)).clicked() {
                        self.dialogs.right_panel_open = true;
                    }
                });
        }
    }
}
