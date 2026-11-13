//! The coloring method list (coloring plan B6, design §9.1): the panel's Method box and Color ▸
//! Method draw the same list, the methods under headings by what they color by
//! (`fractadyne_core::coloring::MethodGroup`). Each method's tooltip says what it shows and what
//! choosing it costs here: whether the switch recolors the view on screen or calculates it again,
//! and which speed-ups and outputs it gives up. A method the family cannot color by is grayed out
//! with the reason, as before.
//!
//! ⭐**Every cost is read off the rules the renderer follows, never written beside them.** The
//! switch's comes from `ColoringConfig::orbit_stats_of`, the statistics set the frame's iteration
//! key holds (the GPU iterates again exactly when that set changes); the notes come from
//! `ColorMethod::can`, the capability table (B4), and say what a method gives up only where the
//! family, the view and the settings would otherwise give it. The UI test's `method-switch-cost`
//! check holds the switch's cost against the GPU's own record of whether a pass ran.

use crate::{Capability, ColorMethod, ColoringLabel, FractadyneApp};
use fractadyne_core::coloring::{Lacks, MethodGroup};

/// What switching to a method does to the picture on screen now.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SwitchCost {
    /// It is the method in use.
    InUse,
    /// The frame on screen already holds what the method colors by: the switch recolors it.
    Recolors,
    /// The switch changes what the iteration accumulates, so the view is calculated again.
    Recalculates,
}

/// The speed-ups and outputs the current family, view and settings give a method that gives up
/// none of them (Smooth). A note says a method lacks one only where it would otherwise apply: on a
/// Julia view nothing skips iterations, so no method is slower there for the want of a skip.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct Offered {
    /// Between 1e4× and 1e28×, iterations are skipped: the live view's band BLA, or series
    /// approximation where that is off (and in exports, which render that band in df32).
    pub(crate) skip_shallow: bool,
    /// Past 1e28×, iterations are skipped: BLA, or series approximation where no BLA tree is built.
    pub(crate) skip_deep: bool,
    /// Exports run multi-reference glitch correction.
    pub(crate) glitch_correction: bool,
    /// A zoom video can be made of this view.
    pub(crate) zoom_video: bool,
}

/// One thing a method gives up, as its tooltip says it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum MethodNote {
    /// Past 1e4×, every step is calculated: no series approximation, no BLA.
    NoSkip,
    /// Between 1e4× and 1e28×, every step is calculated (BLA joins past 1e28×).
    NoSkipShallow,
    /// Exports skip glitch correction.
    NoGlitchCorrection,
    /// No zoom video can use it.
    NoZoomVideo,
}

impl MethodNote {
    /// Every note, in the order the tooltip lists them.
    pub(crate) const ALL: [MethodNote; 4] =
        [MethodNote::NoSkip, MethodNote::NoSkipShallow, MethodNote::NoGlitchCorrection, MethodNote::NoZoomVideo];

    /// The capabilities whose absence this note reports: it is said of a method that can do none of
    /// them (and only where the view would otherwise give them, [`method_notes`]). The normalized
    /// export is the one capability no note reports: the panel's "Normalize deep colors" grays
    /// itself out for every method but Smooth and says why, which covers every method it refuses.
    pub(crate) const fn reads(self) -> &'static [Capability] {
        match self {
            MethodNote::NoSkip => &[Capability::Series, Capability::Bla],
            MethodNote::NoSkipShallow => &[Capability::Series, Capability::LiveWalk],
            MethodNote::NoGlitchCorrection => &[Capability::GlitchCorrection],
            MethodNote::NoZoomVideo => &[Capability::Strips],
        }
    }
}

/// What `m` gives up where `o` says the family, view and settings would otherwise give it, in the
/// tooltip's order.
pub(crate) fn method_notes(m: ColorMethod, o: Offered) -> Vec<MethodNote> {
    MethodNote::ALL
        .into_iter()
        .filter(|n| n.reads().iter().all(|c| !m.can(*c)))
        .filter(|n| match n {
            // Neither skip at any depth (decomposition: its final angle has no BLA aggregate).
            MethodNote::NoSkip => o.skip_shallow || o.skip_deep,
            // No series, and the live view's band BLA refuses it, but BLA takes it past 1e28×:
            // only the band between is calculated step by step. Without BLA, NoSkip says it all.
            MethodNote::NoSkipShallow => m.can(Capability::Bla) && o.skip_shallow,
            MethodNote::NoGlitchCorrection => o.glitch_correction,
            MethodNote::NoZoomVideo => o.zoom_video,
        })
        .collect()
}

/// A heading's name in the language in use.
pub(crate) fn group_label(g: MethodGroup) -> String {
    match g {
        MethodGroup::EscapeCount => tr!(method_group_escape),
        MethodGroup::Distance => tr!(method_group_distance),
        MethodGroup::OrbitStatistics => tr!(method_group_statistics),
        MethodGroup::Angle => tr!(method_group_angle),
    }
}

/// What a method shows, one sentence.
fn method_description(m: ColorMethod) -> String {
    match m {
        ColorMethod::Smooth => tr!(method_desc_smooth),
        ColorMethod::Stripe => tr!(method_desc_stripe),
        ColorMethod::TriangleIneq => tr!(method_desc_triangle),
        ColorMethod::OrbitTrap => tr!(method_desc_trap),
        ColorMethod::Distance => tr!(method_desc_distance),
        ColorMethod::Decomposition => tr!(method_desc_decomposition),
    }
}

fn note_text(n: MethodNote) -> String {
    match n {
        MethodNote::NoSkip => tr!(method_note_no_skip),
        MethodNote::NoSkipShallow => tr!(method_note_no_skip_shallow),
        MethodNote::NoGlitchCorrection => tr!(method_note_no_glitch_correction),
        MethodNote::NoZoomVideo => tr!(method_note_no_zoom_video),
    }
}

impl FractadyneApp {
    /// The views on screen, each with whether it is a Julia view: what a method switch reaches.
    pub(crate) fn method_views(&self) -> Vec<(f64, bool)> {
        if self.dual {
            vec![(self.viewport.magnification(), false), (self.julia_viewport.magnification(), true)]
        } else {
            vec![(self.viewport.magnification(), self.julia_mode)]
        }
    }

    /// What switching to `to` does to the frames on screen ([`SwitchCost`]).
    ///
    /// ⭐Asked under each view's CURRENT mode, which is exact: a view's live mode depends on the
    /// method only through `band_bla`, which turns on only for a frame that accumulates nothing.
    /// Two methods that accumulate the same set therefore share the mode too; and where the current
    /// frame accumulates nothing but `to` would, `to`'s set is not empty under either mode.
    /// What choosing `to` for the inside does to the frames on screen (coloring plan B5): the
    /// method list's rule, for the statistic the inside reads.
    pub(crate) fn inside_switch_cost(&self, to: crate::InteriorMode) -> SwitchCost {
        let m = self.coloring.color_method;
        if to == self.coloring.interior_of(m) {
            return SwitchCost::InUse;
        }
        let same = self.method_views().into_iter().all(|(mag, julia)| {
            let mode = self.live_render_mode(self.fractal, julia, mag);
            self.coloring.orbit_stats_with(m, mode, to) == self.coloring.orbit_stats(mode)
        });
        if same {
            SwitchCost::Recolors
        } else {
            SwitchCost::Recalculates
        }
    }

    pub(crate) fn switch_cost(&self, to: ColorMethod) -> SwitchCost {
        if to == self.coloring.color_method {
            return SwitchCost::InUse;
        }
        let same = self.method_views().into_iter().all(|(mag, julia)| {
            let mode = self.live_render_mode(self.fractal, julia, mag);
            self.coloring.orbit_stats_of(to, mode) == self.coloring.orbit_stats(mode)
        });
        if same {
            SwitchCost::Recolors
        } else {
            SwitchCost::Recalculates
        }
    }

    /// What the family, the views on screen and the settings give a method that gives up nothing.
    pub(crate) fn offered(&self) -> Offered {
        let caps = self.fractal.caps();
        let views = self.method_views();
        let any_mandelbrot_plane = views.iter().any(|(_, julia)| !julia);
        // Series approximation seeds the parameter plane's perturbation only (`do_sa` asks !julia).
        let series = caps.series_approximation && self.render_cfg.series_approx && any_mandelbrot_plane;
        let band = views.iter().any(|(_, julia)| self.band_bla_if(self.fractal, *julia, false));
        let bla = caps.bla && self.render_cfg.use_bla && any_mandelbrot_plane && self.fractal != crate::fractal::FractalKind::Custom;
        Offered {
            skip_shallow: band || series,
            skip_deep: bla || series,
            glitch_correction: caps.export_glitch_correction && any_mandelbrot_plane,
            zoom_video: !self.julia_mode && !self.dual && crate::ui::zoom_video::zoom_video_family(self.fractal),
        }
    }

    /// The method list, for the panel's Method box and Color ▸ Method. Returns whether a method was
    /// chosen (the menu closes then).
    pub(crate) fn method_list(&mut self, ui: &mut egui::Ui) -> bool {
        let caps = self.fractal.caps();
        let family = self.fractal.display_name();
        let offered = self.offered();
        let forced = self.dialogs.uitest_method_tooltip;
        let mut chosen = None;
        for (gi, g) in MethodGroup::ALL.into_iter().enumerate() {
            if gi > 0 {
                ui.separator();
            }
            ui.label(egui::RichText::new(group_label(g)).weak());
            for m in g.methods() {
                let lacks = m.lacks(caps);
                let r = ui.add_enabled(lacks.is_none(), egui::SelectableLabel::new(self.coloring.color_method == m, m.label()));
                if r.clicked() {
                    chosen = Some(m);
                }
                let tip = |ui: &mut egui::Ui| self.method_tooltip(ui, m, offered);
                if let Some(l) = lacks {
                    r.on_disabled_hover_text(match l {
                        Lacks::DistanceEstimate => tr!(coloring_needs_derivative, family = family.clone()),
                        Lacks::OrbitStatistics => tr!(coloring_needs_statistics, family = family.clone()),
                    });
                } else if forced == Some(m) {
                    r.show_tooltip_ui(tip);
                } else {
                    r.on_hover_ui(tip);
                }
            }
        }
        if let Some(m) = chosen {
            self.coloring.color_method = m;
        }
        chosen.is_some()
    }

    /// A method's tooltip: what it shows, what switching to it does now, what it gives up here.
    pub(crate) fn method_tooltip(&self, ui: &mut egui::Ui, m: ColorMethod, offered: Offered) {
        ui.set_max_width(340.0);
        ui.label(method_description(m));
        match self.switch_cost(m) {
            SwitchCost::InUse => {}
            SwitchCost::Recolors => {
                ui.label(egui::RichText::new(tr!(method_cost_recolors)).small());
            }
            SwitchCost::Recalculates => {
                ui.label(egui::RichText::new(tr!(method_cost_recalculates)).small());
            }
        }
        for n in method_notes(m, offered) {
            ui.label(egui::RichText::new(note_text(n)).small().weak());
        }
    }

    /// The panel's Inside rows (coloring plan B5, design §9.1): what colors the inside of the set,
    /// each choice saying what it shows and what choosing it costs, as the method list does; and,
    /// while the trap colors it, the interior palette.
    pub(crate) fn inside_rows(&mut self, ui: &mut egui::Ui) {
        use crate::InteriorMode;
        let inside = self.coloring.interior_of(self.coloring.color_method);
        super::panels::labelled(ui, &tr!(panel_inside), |ui| {
            egui::ComboBox::from_id_salt("panel_inside")
                .selected_text(inside_label(inside))
                .show_ui(ui, |ui| {
                    for m in InteriorMode::ALL {
                        let r = ui.selectable_label(inside == m, inside_label(m)).on_hover_ui(|ui| self.inside_tooltip(ui, m));
                        // A choice only when it changes something: the method's own inside stays the
                        // method's (`None`) until another is picked, so a later method keeps its own.
                        if r.clicked() && inside != m {
                            self.coloring.interior = Some(m);
                        }
                    }
                })
                .response
                .on_hover_text(tr!(panel_inside_hover));
            // A solid inside's color (C1), beside it. Duotone and Binary color the inside with
            // their dark color, which their own swatch sets.
            if inside == InteriorMode::Solid && !self.coloring.use_duotone && !self.coloring.use_binary {
                if let Some(c) = crate::color_swatch(ui, egui::Id::new("panel_inside_color"), self.coloring.interior_color, &tr!(panel_inside_color_hover)) {
                    self.coloring.interior_color = c;
                }
            }
        });
        if inside != InteriorMode::Trap {
            return;
        }
        let presets = fractadyne_color::PRESETS;
        let name = match self.coloring.interior_palette {
            Some(i) => presets[i.min(presets.len() - 1)].name.to_string(),
            None => tr!(panel_inside_palette_same),
        };
        let swatch = self.coloring.interior_palette.map(|i| {
            let p = &presets[i.min(presets.len() - 1)];
            fractadyne_color::segment::Gradient::from_stops(p.name, p.stops).bake(fractadyne_color::segment::LUT_SIZE)
        });
        super::panels::labelled(ui, &tr!(panel_inside_palette), |ui| {
            // ⚠**AS WIDE AS THE ROOM LEFT, the name shortened to fit**, never as wide as its text:
            // "Same as outside" widened the panel by 15 px, the canvas narrowed, and choosing the
            // inside's palette calculated the whole view again (`--uitest` inside-trap, 2026-11-13;
            // the Method box's lesson, B7). ⚠In a child of that width: a combo box truncates its text
            // at the width AVAILABLE to it (it overwrites a laid-out text's own wrapping), which in
            // the row ran to the panel's edge, and its padding then spilled past it.
            let sp = ui.spacing();
            let box_w = (ui.available_width() - sp.item_spacing.x).max(60.0);
            let inner_w = box_w - 2.0 * sp.button_padding.x;
            let combo = ui
                .allocate_ui(egui::vec2(inner_w, sp.interact_size.y), |ui| {
                    egui::ComboBox::from_id_salt("panel_inside_palette")
                        .width(inner_w)
                        .truncate()
                        // A chosen palette with its swatch, as the Palette box shows one; "Same as
                        // outside" as words alone, which want the room.
                        .selected_text(if self.coloring.interior_palette.is_some() {
                            crate::ui::menus::palette_box_text(ui, &name)
                        } else {
                            name.clone().into()
                        })
                        .height(430.0)
                        .show_ui(ui, |ui| {
                    if ui.selectable_label(self.coloring.interior_palette.is_none(), tr!(panel_inside_palette_same)).clicked() {
                        self.coloring.interior_palette = None;
                    }
                    for (i, p) in presets.iter().enumerate() {
                        Self::palette_preset_heading(ui, i);
                        let g = fractadyne_color::segment::Gradient::from_stops(p.name, p.stops);
                        let chosen = self.coloring.interior_palette == Some(i);
                        let row = Self::palette_menu_row(ui, &g, p.name, chosen);
                        if row.clicked {
                            self.coloring.interior_palette = Some(i);
                        }
                    }
                        })
                })
                .inner;
            if let Some(swatch) = &swatch {
                crate::ui::menus::paint_palette_box_swatch(ui, combo.response.rect, swatch);
            }
            combo.response.on_hover_text(tr!(panel_inside_palette_hover));
        });
    }

    /// An Inside choice's tooltip: what it shows, then what choosing it does to the frame on screen.
    fn inside_tooltip(&self, ui: &mut egui::Ui, m: crate::InteriorMode) {
        ui.set_max_width(340.0);
        ui.label(match m {
            crate::InteriorMode::Solid => tr!(inside_solid_desc),
            crate::InteriorMode::Trap => tr!(inside_trap_desc),
        });
        match self.inside_switch_cost(m) {
            SwitchCost::InUse => {}
            SwitchCost::Recolors => {
                ui.label(egui::RichText::new(tr!(method_cost_recolors)).small());
            }
            SwitchCost::Recalculates => {
                ui.label(egui::RichText::new(tr!(method_cost_recalculates)).small());
            }
        }
    }
}

/// An Inside choice's name in the panel.
pub(crate) fn inside_label(m: crate::InteriorMode) -> String {
    match m {
        crate::InteriorMode::Solid => tr!(inside_solid),
        crate::InteriorMode::Trap => tr!(inside_trap),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/ui/method_list/tests.rs"]
mod tests;
