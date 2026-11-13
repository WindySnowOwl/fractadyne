//! Color ▸ Preview methods…: the current view in every coloring method, side by side (coloring
//! plan B7, design §9.2), as the palette browser shows it in every palette.
//!
//! ⭐**A cell whose data the frame on screen already holds is the view recolored, not re-rendered**
//! (`fractadyne_gpu::atlas::render_atlas` with the cell's method): Smooth and Distance color the
//! main lane, which every frame has; a statistic's methods color the aux lane, which holds it when
//! the frame accumulated it with the same parameters (the trap's shape, the stripe density). Every
//! other cell needs a calculation of its own. [`sheet_plan`] decides which is which; it is a pure
//! function of what the frame accumulated, with its own tests.
//!
//! The cells are the methods under the method list's headings (`MethodGroup`), with the parameter
//! variants that change the picture most: the three trap shapes, and the stripe density in use
//! beside 3 and 6. A method the family cannot color by (`ColorMethod::lacks`) has no cell: it would
//! be one flat color.

use crate::{ColorMethod, ColoringLabel, Stat, StatSet, TrapType};
use fractadyne_core::coloring::MethodGroup;
use fractadyne_gpu::atlas::{self, AtlasCell, AtlasPalette, CellColoring, PaletteAtlas};

/// Thumbnail width on screen, in points, and a cell's width in texels (more, so egui's linear
/// filter averages the point samples), recolored or calculated.
const THUMB_W: f32 = 220.0;
pub(crate) const CELL_W: u32 = 440;
/// Space between thumbnails, and the height of the two text lines under one (name, variant).
const GAP: f32 = 10.0;
const LABEL_H: f32 = 36.0;
/// Shortest time between two redraws while the view keeps changing under the sheet.
const REDRAW_PACE: std::time::Duration = std::time::Duration::from_millis(400);

/// The stripe densities the sheet shows besides the one in use (design §9.2).
pub(crate) const STRIPE_VARIANTS: [f32; 2] = [3.0, 6.0];

/// One cell of the sheet: a method, with the parameter that changes what its statistic holds.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct SheetCell {
    pub(crate) method: ColorMethod,
    /// The trap's shape, for the trap's cells.
    pub(crate) trap: Option<TrapType>,
    /// The stripe density, for the stripe's cells.
    pub(crate) stripe_freq: Option<f32>,
}

/// How a cell gets its picture.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum CellSource {
    /// The frame on screen holds what it colors by: a color pass over it.
    Recolor,
    /// It needs a calculation of its own.
    Render,
}

/// What the frame on screen accumulated, and the parameters the statistics were taken with.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct FrameStats {
    pub(crate) set: StatSet,
    pub(crate) stripe_freq: f32,
    pub(crate) trap: TrapType,
}

/// Whether `cell` can be drawn from a frame that accumulated `frame`.
pub(crate) fn cell_source(cell: SheetCell, frame: FrameStats) -> CellSource {
    let Some(stat) = cell.method.reads() else {
        return CellSource::Recolor; // the main lane: every frame has it
    };
    let same = match stat {
        Stat::Stripe => cell.stripe_freq.is_some_and(|f| (f - frame.stripe_freq).abs() <= f32::EPSILON * f.abs().max(1.0)),
        Stat::Trap => cell.trap == Some(frame.trap),
        Stat::Triangle | Stat::Angle => true,
    };
    if frame.set.has(stat) && same {
        CellSource::Recolor
    } else {
        CellSource::Render
    }
}

/// The sheet's cells for a family with `caps`, a row per heading (empty headings left out), each
/// with how it gets its picture from a frame that accumulated `frame`.
pub(crate) fn sheet_plan(caps: fractadyne_core::formula::FormulaCaps, frame: FrameStats) -> Vec<(MethodGroup, Vec<(SheetCell, CellSource)>)> {
    let mut rows = Vec::new();
    for g in MethodGroup::ALL {
        let mut cells = Vec::new();
        for m in g.methods().filter(|m| m.offered_for(caps)) {
            let plain = SheetCell { method: m, trap: None, stripe_freq: None };
            match m.reads() {
                Some(Stat::Trap) => {
                    cells.extend(TrapType::ALL.into_iter().map(|t| SheetCell { trap: Some(t), ..plain }));
                }
                Some(Stat::Stripe) => {
                    let mut freqs = vec![frame.stripe_freq];
                    for v in STRIPE_VARIANTS {
                        if !freqs.iter().any(|f| (f - v).abs() <= 1.0e-4) {
                            freqs.push(v);
                        }
                    }
                    freqs.sort_by(f32::total_cmp);
                    cells.extend(freqs.into_iter().map(|f| SheetCell { stripe_freq: Some(f), ..plain }));
                }
                _ => cells.push(plain),
            }
        }
        if !cells.is_empty() {
            rows.push((g, cells.into_iter().map(|c| (c, cell_source(c, frame))).collect()));
        }
    }
    rows
}

/// A run of one group's cells on one row of the sheet.
pub(crate) struct Segment<'a> {
    pub(crate) group: MethodGroup,
    /// The group's first run: it carries the heading.
    pub(crate) first: bool,
    pub(crate) cells: &'a [(SheetCell, CellSource)],
}

/// The plan laid out in rows of at most `cols` cells: a group goes on the row in progress when it
/// fits there, else starts a row, and a group longer than a row is cut into runs. So short groups
/// (the count, the distance) share a row instead of each leaving one mostly empty.
pub(crate) fn sheet_rows<'a>(plan: &'a [(MethodGroup, Vec<(SheetCell, CellSource)>)], cols: usize) -> Vec<Vec<Segment<'a>>> {
    let cols = cols.max(1);
    let mut rows: Vec<Vec<Segment<'a>>> = Vec::new();
    let mut used = 0;
    for (g, cells) in plan {
        if used > 0 && used + cells.len() > cols {
            used = 0;
        }
        for (i, run) in cells.chunks(cols).enumerate() {
            if used == 0 || i > 0 {
                rows.push(Vec::new());
                used = 0;
            }
            rows.last_mut().expect("a row was pushed").push(Segment { group: *g, first: i == 0, cells: run });
            used += run.len();
        }
    }
    rows
}

#[derive(Default)]
pub(crate) struct MethodSheet {
    pub(crate) open: bool,
    /// Resting the pointer on a recolor cell shows that method on the view (the palette pickers'
    /// rules, `crate::palette_preview`); a cell that needs its own calculation is not previewed, which
    /// would start one.
    pub(crate) preview: crate::palette_preview::HoverPreview<SheetCell>,
    /// The cells that need a calculation of their own, and the thread that calculates them
    /// (`ui::sheet_render`, step 4).
    pub(crate) render: crate::ui::sheet_render::SheetRender,
    atlas: Option<PaletteAtlas>,
    /// The recolor cells the atlas holds, in its order, and what it was drawn for, and when.
    atlas_cells: Vec<SheetCell>,
    drawn_for: Option<SheetKey>,
    drawn_at: Option<std::time::Instant>,
}

/// Everything a drawn atlas depends on.
#[derive(Clone, Copy, PartialEq, Eq)]
struct SheetKey {
    writes: u64,
    colouring: u64,
    cells: u64,
    cell: [u32; 2],
}

impl crate::FractadyneApp {
    /// What view 0's frame accumulated, and with which parameters.
    pub(crate) fn sheet_frame(&self) -> FrameStats {
        let mode = self.live_render_mode(self.fractal, self.julia_mode && !self.dual, self.viewport.magnification());
        FrameStats { set: self.coloring.orbit_stats(mode), stripe_freq: self.coloring.stripe_freq, trap: self.coloring.trap_type }
    }

    /// The method the view is colored with: the chosen one, or the sheet's previewed cell's while
    /// the pointer rests on it. ⭐Only a cell every view on screen can recolor (the dual view's Julia
    /// pane may sit in another band, holding other statistics): a preview never starts a
    /// calculation, since what a frame accumulates is its set plus what the shown method reads
    /// (`fractadyne_gpu::iter_stats`), and a recolor cell's statistic is already in the set.
    /// Exports, files and the session read the chosen method, never this.
    pub(crate) fn shown_method(&self) -> ColorMethod {
        let chosen = self.coloring.color_method;
        let Some(c) = self.method_sheet.preview.shown() else { return chosen };
        let all_recolor = self.method_views().into_iter().all(|(mag, julia)| {
            let mode = self.live_render_mode(self.fractal, julia, mag);
            let frame = FrameStats { set: self.coloring.orbit_stats(mode), stripe_freq: self.coloring.stripe_freq, trap: self.coloring.trap_type };
            cell_source(c, frame) == CellSource::Recolor
        });
        if all_recolor {
            c.method
        } else {
            chosen
        }
    }

    /// Settle the sheet's hover preview for this frame (as `settle_palette_preview`).
    pub(crate) fn settle_method_preview(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        let s = self.method_sheet.preview.end_frame(now);
        if s.changed {
            ctx.request_repaint();
        }
        if let Some(t) = s.wake_in {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(t.max(0.0)));
        }
    }

    /// The cell that is the coloring in use.
    pub(crate) fn sheet_cell_in_use(&self) -> SheetCell {
        let m = self.coloring.color_method;
        SheetCell {
            method: m,
            trap: (m.reads() == Some(Stat::Trap)).then_some(self.coloring.trap_type),
            stripe_freq: (m.reads() == Some(Stat::Stripe)).then_some(self.coloring.stripe_freq),
        }
    }

    /// Can the cells show this view? Not a Game of Life or an L-system, which draw through their
    /// own renderers.
    fn sheet_can_draw(&self) -> bool {
        !matches!(self.fractal, crate::FractalKind::Life | crate::FractalKind::LSystem)
    }

    /// Redraw the atlas of recolor cells when what it shows has changed, at most every
    /// [`REDRAW_PACE`].
    fn refresh_method_atlas(&mut self, ctx: &egui::Context, plan: &[(MethodGroup, Vec<(SheetCell, CellSource)>)]) {
        let Some(rs) = self.render_state.clone() else { return };
        if !self.sheet_can_draw() {
            if let Some(a) = self.method_sheet.atlas.take() {
                atlas::free_palette_atlas(&rs, a);
            }
            self.method_sheet.drawn_for = None;
            return;
        }
        let Some(writes) = atlas::view_writes(&rs, 0) else { return };
        let recolor: Vec<SheetCell> =
            plan.iter().flat_map(|(_, cs)| cs.iter()).filter(|(_, s)| *s == CellSource::Recolor).map(|(c, _)| *c).collect();
        let cells_sig = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            for c in &recolor {
                c.method.key().hash(&mut h);
                c.trap.map(|t| t.key()).hash(&mut h);
                c.stripe_freq.map(f32::to_bits).hash(&mut h);
            }
            h.finish()
        };
        let cell = self.sheet_cell_size();
        let key = SheetKey { writes, colouring: self.accum_color_sig(0), cells: cells_sig, cell };
        if self.method_sheet.drawn_for == Some(key) {
            return;
        }
        if let Some(at) = self.method_sheet.drawn_at {
            let since = at.elapsed();
            if since < REDRAW_PACE && self.method_sheet.atlas.is_some() {
                ctx.request_repaint_after(REDRAW_PACE - since);
                return;
            }
        }
        let lut = self.active_gradient().bake(fractadyne_color::segment::LUT_SIZE);
        let palette = AtlasPalette { lut: lut.entries, smooth: lut.smooth };
        let in_use = self.sheet_cell_in_use();
        // The cell in use is the view exactly (its own rate, its normalization); every other cell
        // takes its method's rate and no range mapping.
        let cells: Vec<AtlasCell> = recolor
            .iter()
            .map(|c| AtlasCell {
                palette: &palette,
                coloring: (*c != in_use).then(|| CellColoring { method: c.method, cycle: self.color_cycle_for(c.method) }),
            })
            .collect();
        let (cell, cols) = crate::ui::palette_browser::atlas_layout(cells.len() as u32, cell, rs.device.limits().max_texture_dimension_2d);
        let old = self.method_sheet.atlas.take();
        match atlas::render_atlas(&rs, 0, &cells, cell, cols, old) {
            Ok(a) => {
                self.method_sheet.atlas = Some(a);
                self.method_sheet.atlas_cells = recolor;
                self.method_sheet.drawn_for = Some(key);
            }
            Err(e) => {
                crate::diag::log_line("method-sheet", &format!("cells not drawn: {e:?}"));
                self.method_sheet.atlas_cells.clear();
                self.method_sheet.drawn_for = None;
            }
        }
        self.method_sheet.drawn_at = Some(std::time::Instant::now());
    }

    /// Make `c` the coloring in use, as choosing its method (and its shape or density) does.
    fn sheet_choose(&mut self, c: SheetCell) {
        self.coloring.color_method = c.method;
        if let Some(t) = c.trap {
            self.coloring.trap_type = t;
        }
        if let Some(f) = c.stripe_freq {
            self.coloring.stripe_freq = f;
        }
    }

    pub(crate) fn draw_method_sheet(&mut self, ctx: &egui::Context) {
        if !self.method_sheet.open {
            if let (Some(a), Some(rs)) = (self.method_sheet.atlas.take(), self.render_state.clone()) {
                atlas::free_palette_atlas(&rs, a);
            }
            self.method_sheet.atlas_cells.clear();
            self.method_sheet.drawn_for = None;
            self.method_sheet.render.stop();
            return;
        }
        let plan = sheet_plan(self.fractal.caps(), self.sheet_frame());
        self.refresh_method_atlas(ctx, &plan);
        if self.sheet_can_draw() {
            self.sheet_render_step(ctx, &plan);
        } else {
            self.method_sheet.render.stop();
        }
        let in_use = self.sheet_cell_in_use();
        let aspect = (self.viewport.height_px / self.viewport.width_px.max(1.0)).clamp(0.2, 5.0) as f32;
        let thumb = egui::vec2(THUMB_W, (THUMB_W * aspect).round());
        let (mut open, mut close) = (true, false);
        let mut pick: Option<SheetCell> = None;
        egui::Window::new(tr!(method_sheet_title))
            .id(egui::Id::new("fractadyne.method-preview"))
            .open(&mut open)
            .default_size([4.0 * (THUMB_W + GAP) + 30.0, 720.0])
            .show(ctx, |ui| {
                // A picker, for the hover preview: reported every frame the window draws.
                self.method_sheet.preview.picker_open();
                if !self.sheet_can_draw() || self.method_sheet.atlas.is_none() {
                    ui.label(egui::RichText::new(tr!(method_sheet_no_view)).weak().small());
                }
                egui::ScrollArea::vertical()
                    .id_salt("method-sheet-grid")
                    .max_height((ui.available_height() - 40.0).max(160.0))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let cols = (((ui.available_width() + GAP) / (thumb.x + GAP)).floor() as usize).max(1);
                        for row in sheet_rows(&plan, cols) {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = GAP;
                                for seg in row {
                                    ui.vertical(|ui| {
                                        // A heading over a group's first cells; a group's next row
                                        // keeps the line, empty, so the cells stay aligned.
                                        let head = if seg.first { crate::ui::method_list::group_label(seg.group) } else { String::new() };
                                        ui.label(egui::RichText::new(head).weak());
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = GAP;
                                            for (c, src) in seg.cells {
                                                if self.sheet_tile(ui, *c, *src, thumb, *c == in_use) {
                                                    pick = Some(*c);
                                                }
                                            }
                                        });
                                    });
                                }
                            });
                            ui.add_space(GAP);
                        }
                    });
                ui.separator();
                crate::theme::action_row(ui, |ui| {
                    if crate::theme::cancel_button(ui, &tr!(dlg_close)).clicked() {
                        close = true;
                    }
                });
            });
        if let Some(c) = pick {
            self.sheet_choose(c);
        }
        self.method_sheet.open = open && !close;
    }

    /// One cell: the picture (or, for a cell that needs its own calculation, a note on the palette),
    /// the method's name and its variant. The cost is in the tooltip, the method list's. Returns
    /// whether it was clicked.
    fn sheet_tile(&mut self, ui: &mut egui::Ui, c: SheetCell, src: CellSource, size: egui::Vec2, in_use: bool) -> bool {
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(size.x, size.y + LABEL_H), egui::Sense::click());
        let img = egui::Rect::from_min_size(rect.min, size);
        let painter = ui.painter_at(rect.expand(4.0));
        let k = self.method_sheet.atlas_cells.iter().position(|x| *x == c);
        let full_uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        match (&self.method_sheet.atlas, k) {
            (Some(a), Some(k)) if src == CellSource::Recolor && k < a.count => {
                painter.image(a.id, img, a.uv(k), egui::Color32::WHITE);
            }
            _ if src == CellSource::Render => {
                use crate::ui::sheet_render::{fmt_eta, CellView};
                let size = self.sheet_cell_size();
                let view = self.method_sheet.render.view(c, size[0] as u64 * size[1] as u64, self.sheet_budget_s());
                // The status line, and the button that goes with it: Render past the budget, Cancel
                // for a cell calculating because it was asked for.
                let (status, button) = match view {
                    CellView::Picture(id) => {
                        painter.image(id, img, full_uv, egui::Color32::WHITE);
                        (None, None)
                    }
                    CellView::Calculating(t) => (
                        Some(tr!(method_sheet_calculating, elapsed = fmt_eta(t.as_secs_f64()))),
                        self.method_sheet.render.asked_in_flight(c).then(|| (tr!(dlg_cancel), String::new(), false)),
                    ),
                    CellView::Estimating => (Some(tr!(method_sheet_estimating)), None),
                    CellView::Priced { estimate, over } => {
                        let eta = fmt_eta(estimate.secs);
                        let line = if estimate.at_least { tr!(method_sheet_estimate_over, eta = eta) } else { tr!(method_sheet_estimate, eta = eta) };
                        (Some(line), over.then(|| (tr!(method_sheet_render), tr!(method_sheet_render_hover), true)))
                    }
                    CellView::Failed => (Some(tr!(method_sheet_render_failed)), None),
                };
                if let Some(status) = status {
                    let below = self.sheet_placeholder(ui, &painter, img, &status);
                    if let Some((label, hover, render)) = button {
                        let font = egui::TextStyle::Button.resolve(ui.style());
                        let w = ui.fonts(|f| f.layout_no_wrap(label.clone(), font, egui::Color32::WHITE).size().x) + 2.0 * ui.spacing().button_padding.x + 4.0;
                        let r = egui::Rect::from_center_size(egui::pos2(img.center().x, below + 14.0), egui::vec2(w.min(img.width() - 8.0), 22.0));
                        // ⚠In a child that takes no room from the row: `ui.put` moved the row's
                        // cursor back to the button's right edge, and the next cell drew over this
                        // one (`--uitest` sheet-cells-apart).
                        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(r));
                        let mut b = child.add_sized(r.size(), egui::Button::new(label));
                        if !hover.is_empty() {
                            b = b.on_hover_text(hover);
                        }
                        if b.clicked() {
                            if render {
                                self.method_sheet.render.ask(c);
                            } else {
                                self.method_sheet.render.stop_cell(c);
                            }
                        }
                    }
                }
            }
            _ => {
                self.sheet_placeholder(ui, &painter, img, "");
            }
        }
        self.method_sheet.preview.row(c, rect, resp.hovered() && src == CellSource::Recolor);
        let accent = ui.visuals().selection.stroke.color;
        if in_use {
            painter.rect_stroke(img.expand(2.0), egui::CornerRadius::same(3), egui::Stroke::new(2.5_f32, accent), egui::StrokeKind::Outside);
        } else if resp.hovered() {
            painter.rect_stroke(img.expand(1.0), egui::CornerRadius::same(2), ui.visuals().widgets.hovered.fg_stroke, egui::StrokeKind::Outside);
        }
        let colour = if in_use { accent } else { ui.visuals().text_color() };
        painter.text(
            egui::pos2(img.center().x, img.bottom() + 11.0),
            egui::Align2::CENTER_CENTER,
            c.method.label(),
            egui::TextStyle::Body.resolve(ui.style()),
            colour,
        );
        let variant = match (c.trap, c.stripe_freq) {
            (Some(t), _) => Some(t.label()),
            (_, Some(f)) => Some(tr!(method_sheet_density, density = format!("{f:.0}"))),
            _ => None,
        };
        if let Some(v) = variant {
            painter.text(
                egui::pos2(img.center().x, img.bottom() + 27.0),
                egui::Align2::CENTER_CENTER,
                v,
                egui::TextStyle::Small.resolve(ui.style()),
                ui.visuals().weak_text_color(),
            );
        }
        let offered = self.offered();
        resp.on_hover_ui(|ui| self.method_tooltip(ui, c.method, offered)).clicked()
    }

    /// A cell with no picture: the palette under a note that it needs its own calculation, and
    /// `status` under that. Returns the note's bottom edge, for a button below it.
    fn sheet_placeholder(&self, ui: &egui::Ui, painter: &egui::Painter, img: egui::Rect, status: &str) -> f32 {
        painter.rect_filled(img, egui::CornerRadius::same(2), ui.visuals().extreme_bg_color);
        let strip = egui::Rect::from_min_size(img.left_bottom() - egui::vec2(0.0, 10.0), egui::vec2(img.width(), 10.0));
        crate::paint_gradient(painter, strip, &self.active_gradient().bake(fractadyne_color::segment::LUT_SIZE));
        let text = if status.is_empty() { tr!(method_sheet_needs_render) } else { format!("{}\n{status}", tr!(method_sheet_needs_render)) };
        // Each line centered: a centered job lays its rows out around x = 0.
        let mut job = egui::text::LayoutJob::simple(text, egui::TextStyle::Small.resolve(ui.style()), ui.visuals().weak_text_color(), img.width() - 16.0);
        job.halign = egui::Align::Center;
        let note = painter.layout_job(job);
        let at = egui::pos2(img.center().x, img.center().y - note.size().y / 2.0 - 14.0);
        let bottom = at.y + note.size().y;
        painter.galley(at, note, ui.visuals().weak_text_color());
        bottom
    }
}

#[cfg(test)]
#[path = "../../tests/unit/ui/method_sheet/tests.rs"]
mod tests;
