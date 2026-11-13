//! Color ▸ Browse palettes…: every palette over the current view, side by side, filtered by tag,
//! with the tags of the palette in use — and, for one of the user's saved gradients, a way to give
//! it tags of their own.
//!
//! ⭐**The thumbnails are the view, recolored, not re-rendered.** `fractadyne_gpu::atlas` colors
//! the main view's iteration texture through each palette into one atlas, so a thumbnail shows what
//! choosing that palette would show, at any depth, for one small color pass each. It is redrawn
//! when the view's texture or its coloring changes (`view_writes`, `accum_color_sig`), paced while
//! a view refines, and not at all once it has settled.
//!
//! Resting the pointer on a thumbnail previews it on the view, exactly as the palette lists do
//! (`crate::palette_preview`): the browser reports itself as an open picker while it is shown.

use crate::palette_preview::PaletteRow;
use fractadyne_color::tags::{self, TagGroup, TAGS};
use fractadyne_gpu::atlas::{self, AtlasPalette, PaletteAtlas};

/// Thumbnail width on screen, in points.
const THUMB_W: f32 = 140.0;
/// Space between thumbnails, and the height of the name under one.
const GAP: f32 = 10.0;
const NAME_H: f32 = 18.0;
/// The filter rows: groups that share a row, short ones paired so the filters take five rows.
const FILTER_ROWS: [&[TagGroup]; 5] = [
    &[TagGroup::Colour],
    &[TagGroup::Temperature, TagGroup::Energy],
    &[TagGroup::Brightness, TagGroup::Shape],
    &[TagGroup::Theme],
    &[TagGroup::Purpose],
];
/// A thumbnail's width in the atlas, in texels: more than it is shown at, so egui's linear filter
/// averages the color pass's point samples instead of showing their aliasing.
const CELL_W: u32 = 320;

/// Cell size and columns for an atlas of `n` thumbnails of `cell` texels within a `max`-texel
/// texture. About as many columns as rows, so the atlas stays near square: eight fixed columns
/// made 190 presets a strip 24 cells tall, past an 8192-texel limit once the view is tall. The
/// cell shrinks, keeping its shape, only when even a square atlas would not fit.
pub(crate) fn atlas_layout(n: u32, cell: [u32; 2], max: u32) -> ([u32; 2], u32) {
    let n = n.max(1);
    let (w, h) = (cell[0].max(1) as f64, cell[1].max(1) as f64);
    let cols = ((n as f64 * h / w).sqrt().ceil() as u32).clamp(1, n);
    let rows = n.div_ceil(cols);
    let fit = (max as f64 / (cols as f64 * w)).min(max as f64 / (rows as f64 * h)).min(1.0);
    let shrink = |v: f64| ((v * fit).floor() as u32).max(1);
    ([shrink(w), shrink(h)], cols)
}
/// Shortest time between two redraws while the view keeps changing under the browser.
const REDRAW_PACE: std::time::Duration = std::time::Duration::from_millis(400);
/// The longest tag a user may type.
const MAX_TAG_CHARS: usize = 32;

#[derive(Default)]
pub(crate) struct PaletteBrowser {
    pub(crate) open: bool,
    /// Tag ids every palette shown must carry.
    pub(crate) filter: Vec<String>,
    atlas: Option<PaletteAtlas>,
    /// What the atlas was drawn for (`AtlasKey`), and when.
    drawn_for: Option<AtlasKey>,
    drawn_at: Option<std::time::Instant>,
    /// The tag being typed for a saved gradient.
    new_tag: String,
    /// The measured tags of each saved gradient, and the library they were measured for.
    saved_measured: Option<(u64, Vec<Vec<&'static str>>)>,
    /// Frames the window has been open: in its first, the grid scrolls to the palette in use.
    open_frames: u32,
}

/// Everything a drawn atlas depends on.
#[derive(Clone, Copy, PartialEq, Eq)]
struct AtlasKey {
    writes: u64,
    colouring: u64,
    library: u64,
    cell: [u32; 2],
}

/// One palette in the browser: its row (what the preview and the lists call it), its name, its
/// tags, and whether the user can change them.
struct Entry {
    row: PaletteRow,
    name: String,
    tags: Vec<String>,
}

/// A tag's name in the interface language; a user's own tag is shown as they typed it.
pub(crate) fn tag_label(id: &str) -> String {
    match id {
        "reds" => tr!(tag_reds),
        "oranges" => tr!(tag_oranges),
        "yellows" => tr!(tag_yellows),
        "greens" => tr!(tag_greens),
        "blues" => tr!(tag_blues),
        "purples" => tr!(tag_purples),
        "pinks" => tr!(tag_pinks),
        "browns" => tr!(tag_browns),
        "neutrals" => tr!(tag_neutrals),
        "warm" => tr!(tag_warm),
        "cool" => tr!(tag_cool),
        "calm" => tr!(tag_calm),
        "lively" => tr!(tag_lively),
        "frantic" => tr!(tag_frantic),
        "dark" => tr!(tag_dark),
        "light" => tr!(tag_light),
        "pastel" => tr!(tag_pastel),
        "vivid" => tr!(tag_vivid),
        "smooth" => tr!(tag_smooth),
        "banded" => tr!(tag_banded),
        "single-hue" => tr!(tag_single_hue),
        "rainbow" => tr!(tag_rainbow),
        "holidays" => tr!(tag_holidays),
        "seasons" => tr!(tag_seasons),
        "nature" => tr!(tag_nature),
        "retro" => tr!(tag_retro),
        "art" => tr!(tag_art),
        "materials" => tr!(tag_materials),
        "science" => tr!(tag_science),
        "food" => tr!(tag_food),
        "minerals" => tr!(tag_minerals),
        "plants" => tr!(tag_plants),
        "animals" => tr!(tag_animals),
        "biomes" => tr!(tag_biomes),
        "places" => tr!(tag_places),
        "analytical" => tr!(tag_analytical),
        "aesthetic" => tr!(tag_aesthetic),
        own => own.to_string(),
    }
}

fn group_label(g: TagGroup) -> String {
    match g {
        TagGroup::Colour => tr!(tag_group_color),
        TagGroup::Temperature => tr!(tag_group_temperature),
        TagGroup::Energy => tr!(tag_group_energy),
        TagGroup::Brightness => tr!(tag_group_brightness),
        TagGroup::Shape => tr!(tag_group_shape),
        TagGroup::Theme => tr!(tag_group_theme),
        TagGroup::Purpose => tr!(tag_group_purpose),
    }
}

/// What a typed tag is stored as: a vocabulary tag when it names one (by id or by its name in the
/// interface language, case ignored), else the user's own words, trimmed, single-spaced, and cut to
/// [`MAX_TAG_CHARS`]. `None` for nothing.
pub(crate) fn normalize_user_tag(typed: &str) -> Option<String> {
    let words: Vec<&str> = typed.split_whitespace().collect();
    let s: String = words.join(" ").chars().filter(|c| !c.is_control()).take(MAX_TAG_CHARS).collect();
    if s.is_empty() {
        return None;
    }
    let lower = s.to_lowercase();
    TAGS.iter()
        .find(|d| d.id == lower || tag_label(d.id).to_lowercase() == lower)
        .map(|d| d.id.to_string())
        .or(Some(s))
}

/// Does `tags` hold every one of `filter`?
pub(crate) fn passes(tags: &[String], filter: &[String]) -> bool {
    filter.iter().all(|f| tags.iter().any(|t| t.eq_ignore_ascii_case(f)))
}

impl crate::FractadyneApp {
    /// A signature of the saved gradient library: names, segments and tags.
    fn library_sig(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for g in &self.saved_gradients {
            g.name.hash(&mut h);
            format!("{:?}", g.segment).hash(&mut h);
            g.tags.hash(&mut h);
        }
        h.finish()
    }

    /// Every palette the browser lists, built-in first, then the user's saved gradients.
    fn browser_entries(&mut self) -> Vec<Entry> {
        let sig = self.library_sig();
        if self.palette_browser.saved_measured.as_ref().map(|(s, _)| *s) != Some(sig) {
            let measured = self
                .saved_gradients
                .iter()
                .map(|g| tags::computed_tags(&crate::segments_to_gradient(&g.name, &g.segment)))
                .collect();
            self.palette_browser.saved_measured = Some((sig, measured));
        }
        let measured = &self.palette_browser.saved_measured.as_ref().map(|(_, m)| m.clone()).unwrap_or_default();
        let mut out: Vec<Entry> = fractadyne_color::PRESETS
            .iter()
            .enumerate()
            .map(|(i, p)| Entry {
                row: PaletteRow::Preset(i),
                name: p.name.to_string(),
                tags: tags::preset_tags(i).iter().map(|t| t.to_string()).collect(),
            })
            .collect();
        for (j, g) in self.saved_gradients.iter().enumerate() {
            let mut t: Vec<String> = measured.get(j).map(|m| m.iter().map(|s| s.to_string()).collect()).unwrap_or_default();
            for own in &g.tags {
                if !t.iter().any(|x| x.eq_ignore_ascii_case(own)) {
                    t.push(own.clone());
                }
            }
            out.push(Entry { row: PaletteRow::Saved(j), name: g.name.clone(), tags: t });
        }
        out
    }

    /// The baked table of every listed palette, in the browser's order.
    fn browser_luts(&self) -> Vec<AtlasPalette> {
        let lut = |g: fractadyne_color::segment::Gradient| {
            let l = g.bake(fractadyne_color::segment::LUT_SIZE);
            AtlasPalette { lut: l.entries, smooth: l.smooth }
        };
        fractadyne_color::PRESETS
            .iter()
            .map(|p| lut(fractadyne_color::segment::Gradient::from_stops(p.name, p.stops)))
            .chain(self.saved_gradients.iter().map(|g| lut(crate::segments_to_gradient(&g.name, &g.segment))))
            .collect()
    }

    /// Can the thumbnails show this view? Not a Game of Life or an L-system, which draw through
    /// their own renderers: their palettes are shown as strips.
    fn browser_can_thumbnail(&self) -> bool {
        !matches!(self.fractal, crate::FractalKind::Life | crate::FractalKind::LSystem)
    }

    /// Redraw the atlas when what it shows has changed, at most every [`REDRAW_PACE`].
    fn refresh_palette_atlas(&mut self, ctx: &egui::Context) {
        let Some(rs) = self.render_state.clone() else { return };
        if !self.browser_can_thumbnail() {
            if let Some(a) = self.palette_browser.atlas.take() {
                atlas::free_palette_atlas(&rs, a);
            }
            self.palette_browser.drawn_for = None;
            return;
        }
        let Some(writes) = atlas::view_writes(&rs, 0) else { return };
        let aspect = (self.viewport.height_px / self.viewport.width_px.max(1.0)).clamp(0.2, 5.0);
        let cell = [CELL_W, ((CELL_W as f64) * aspect).round().max(8.0) as u32];
        let key = AtlasKey { writes, colouring: self.accum_color_sig(0), library: self.library_sig(), cell };
        if self.palette_browser.drawn_for == Some(key) {
            return;
        }
        if let Some(at) = self.palette_browser.drawn_at {
            let since = at.elapsed();
            if since < REDRAW_PACE && self.palette_browser.atlas.is_some() {
                ctx.request_repaint_after(REDRAW_PACE - since);
                return;
            }
        }
        let luts = self.browser_luts();
        let (cell, cols) = atlas_layout(luts.len() as u32, cell, rs.device.limits().max_texture_dimension_2d);
        let old = self.palette_browser.atlas.take();
        match atlas::render_palette_atlas(&rs, 0, &luts, cell, cols, old) {
            Ok(a) => {
                self.palette_browser.atlas = Some(a);
                self.palette_browser.drawn_for = Some(key);
            }
            Err(e) => {
                crate::diag::log_line("palette", &format!("browser thumbnails not drawn: {e:?}"));
                self.palette_browser.drawn_for = None;
            }
        }
        self.palette_browser.drawn_at = Some(std::time::Instant::now());
    }

    /// Make `row` the palette in use, as choosing it from a list does.
    fn browser_choose(&mut self, row: PaletteRow) {
        match row {
            PaletteRow::Preset(i) => {
                self.coloring.palette_idx = i;
                self.coloring.use_custom_palette = false;
                self.coloring.use_duotone = false;
                self.coloring.use_binary = false;
            }
            PaletteRow::Saved(j) => self.apply_saved_gradient(j),
        }
    }

    /// The palette in use as a browser row, if it is one of them.
    fn browser_chosen(&self) -> Option<PaletteRow> {
        if let Some(j) = self.live_saved_gradient() {
            return Some(PaletteRow::Saved(j));
        }
        let preset = !self.coloring.use_custom_palette && !self.coloring.use_duotone && !self.coloring.use_binary;
        preset.then_some(PaletteRow::Preset(self.coloring.palette_idx))
    }

    pub(crate) fn draw_palette_browser(&mut self, ctx: &egui::Context) {
        if !self.palette_browser.open {
            if let (Some(a), Some(rs)) = (self.palette_browser.atlas.take(), self.render_state.clone()) {
                atlas::free_palette_atlas(&rs, a);
            }
            self.palette_browser.drawn_for = None;
            self.palette_browser.open_frames = 0;
            return;
        }
        self.refresh_palette_atlas(ctx);
        let entries = self.browser_entries();
        let chosen = self.browser_chosen();
        let aspect = (self.viewport.height_px / self.viewport.width_px.max(1.0)).clamp(0.2, 5.0) as f32;
        let thumb = egui::vec2(THUMB_W, (THUMB_W * aspect).round());
        let mut open = true;
        let mut close = false;
        let mut pick: Option<PaletteRow> = None;
        egui::Window::new(tr!(palette_browser_title))
            .id(egui::Id::new("fractadyne.palette-browser"))
            .open(&mut open)
            .default_size([6.0 * (THUMB_W + GAP) + 30.0, 720.0])
            .show(ctx, |ui| {
                // A picker, for the hover preview: reported every frame the window draws.
                self.coloring.palette_preview.picker_open();
                self.browser_filters(ui, &entries);
                ui.separator();
                let shown: Vec<usize> =
                    (0..entries.len()).filter(|k| passes(&entries[*k].tags, &self.palette_browser.filter)).collect();
                // The details panel and, under it, the Close row.
                let details_h = 156.0;
                egui::ScrollArea::vertical()
                    .id_salt("palette-browser-grid")
                    .max_height((ui.available_height() - details_h).max(160.0))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if shown.is_empty() {
                            ui.label(egui::RichText::new(tr!(palette_browser_none)).weak());
                        }
                        if self.palette_browser.atlas.is_none() {
                            ui.label(egui::RichText::new(tr!(palette_browser_no_view)).weak().small());
                        }
                        // A grid of fixed cells, so every row lines up whatever its names are.
                        let cols = (((ui.available_width() + GAP) / (thumb.x + GAP)).floor() as usize).max(1);
                        for row in shown.chunks(cols) {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = GAP;
                                for k in row {
                                    if let Some(r) = self.browser_tile(ui, &entries[*k], *k, thumb, chosen) {
                                        pick = Some(r);
                                    }
                                }
                            });
                            ui.add_space(GAP);
                        }
                    });
                ui.separator();
                self.browser_details(ui, &entries, chosen);
                // A named way out, as every dialog has (the title bar's × alone is easy to miss).
                ui.separator();
                crate::theme::action_row(ui, |ui| {
                    if crate::theme::cancel_button(ui, &tr!(dlg_close)).clicked() {
                        close = true;
                    }
                });
            });
        if let Some(row) = pick {
            self.browser_choose(row);
        }
        self.palette_browser.open = open && !close;
        self.palette_browser.open_frames = self.palette_browser.open_frames.saturating_add(1);
    }

    /// The filter rows: the tag groups in five rows, then the user's own tags on the last.
    fn browser_filters(&mut self, ui: &mut egui::Ui, entries: &[Entry]) {
        let mut toggle: Option<String> = None;
        let mut own: Vec<String> = entries
            .iter()
            .flat_map(|e| e.tags.iter())
            .filter(|t| tags::tag_def(t).is_none())
            .cloned()
            .collect();
        own.sort_by_key(|t| t.to_lowercase());
        own.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
        let filter = self.palette_browser.filter.clone();
        let is_on = |t: &str| filter.iter().any(|f| f.eq_ignore_ascii_case(t));
        let group_head = |ui: &mut egui::Ui, text: String, first: bool| {
            let label = egui::Label::new(egui::RichText::new(text).weak()).truncate();
            if first {
                ui.add_sized([92.0, 18.0], label);
            } else {
                ui.add(label);
            }
        };
        for (r, groups) in FILTER_ROWS.iter().enumerate() {
            ui.horizontal_wrapped(|ui| {
                for (n, g) in groups.iter().enumerate() {
                    if n > 0 {
                        ui.add_space(12.0);
                    }
                    group_head(ui, group_label(*g), n == 0);
                    for d in TAGS.iter().filter(|d| d.group == *g) {
                        if ui.selectable_label(is_on(d.id), tag_label(d.id)).clicked() {
                            toggle = Some(d.id.to_string());
                        }
                    }
                }
                if r == FILTER_ROWS.len() - 1 {
                    if !own.is_empty() {
                        ui.add_space(12.0);
                        group_head(ui, tr!(palette_browser_yours), false);
                        for t in &own {
                            if ui.selectable_label(is_on(t), t).clicked() {
                                toggle = Some(t.clone());
                            }
                        }
                    }
                    ui.add_space(12.0);
                    if ui.add_enabled(!filter.is_empty(), egui::Button::new(tr!(palette_browser_clear))).clicked() {
                        self.palette_browser.filter.clear();
                    }
                }
            });
        }
        if let Some(t) = toggle {
            let f = &mut self.palette_browser.filter;
            match f.iter().position(|x| x.eq_ignore_ascii_case(&t)) {
                Some(i) => {
                    f.remove(i);
                }
                None => f.push(t),
            }
        }
    }

    /// One thumbnail and its name, in a fixed cell. Returns the row when it was clicked.
    fn browser_tile(
        &mut self,
        ui: &mut egui::Ui,
        e: &Entry,
        k: usize,
        size: egui::Vec2,
        chosen: Option<PaletteRow>,
    ) -> Option<PaletteRow> {
        let (cell, resp) = ui.allocate_exact_size(egui::vec2(size.x, size.y + NAME_H), egui::Sense::click());
        let img = egui::Rect::from_min_size(cell.min, size);
        let painter = ui.painter_at(cell.expand(4.0));
        match &self.palette_browser.atlas {
            Some(a) if k < a.count => {
                painter.image(a.id, img, a.uv(k), egui::Color32::WHITE);
            }
            _ => {
                let g = match e.row {
                    PaletteRow::Preset(i) => {
                        let p = &fractadyne_color::PRESETS[i];
                        fractadyne_color::segment::Gradient::from_stops(p.name, p.stops)
                    }
                    PaletteRow::Saved(j) => {
                        let s = &self.saved_gradients[j];
                        crate::segments_to_gradient(&s.name, &s.segment)
                    }
                };
                crate::paint_gradient(&painter, img, &g.bake(fractadyne_color::segment::LUT_SIZE));
            }
        }
        let is_chosen = chosen == Some(e.row);
        // As the lists do: opening on the palette in use, wherever it is in the grid.
        if is_chosen && self.palette_browser.open_frames < 2 {
            ui.scroll_to_rect(cell, Some(egui::Align::Center));
        }
        let accent = ui.visuals().selection.stroke.color;
        if is_chosen {
            painter.rect_stroke(img.expand(2.0), egui::CornerRadius::same(3), egui::Stroke::new(2.5_f32, accent), egui::StrokeKind::Outside);
        } else if resp.hovered() {
            painter.rect_stroke(img.expand(1.0), egui::CornerRadius::same(2), ui.visuals().widgets.hovered.fg_stroke, egui::StrokeKind::Outside);
        }
        let colour = if is_chosen { accent } else { ui.visuals().text_color() };
        let name_rect = egui::Rect::from_min_max(egui::pos2(cell.min.x, img.max.y), cell.max);
        ui.painter_at(name_rect).text(
            name_rect.center(),
            egui::Align2::CENTER_CENTER,
            &e.name,
            egui::TextStyle::Body.resolve(ui.style()),
            colour,
        );
        self.coloring.palette_preview.row(e.row, cell, resp.hovered());
        let tags: Vec<String> = e.tags.iter().map(|t| tag_label(t)).collect();
        let clicked = resp.clicked();
        resp.on_hover_text(format!("{}\n{}", e.name, tags.join(" · ")));
        clicked.then_some(e.row)
    }

    /// The tags of the palette in use; for a saved gradient, its own tags to remove and a way to add.
    fn browser_details(&mut self, ui: &mut egui::Ui, entries: &[Entry], chosen: Option<PaletteRow>) {
        let Some(row) = chosen else { return };
        let Some(e) = entries.iter().find(|e| e.row == row) else { return };
        ui.label(egui::RichText::new(tr!(palette_browser_tags_of, name = e.name.clone())).strong());
        let own: Vec<String> = match row {
            PaletteRow::Saved(j) => self.saved_gradients.get(j).map(|g| g.tags.clone()).unwrap_or_default(),
            PaletteRow::Preset(_) => Vec::new(),
        };
        let mut remove: Option<usize> = None;
        let mut add: Option<String> = None;
        ui.horizontal_wrapped(|ui| {
            for t in &e.tags {
                match own.iter().position(|o| o.eq_ignore_ascii_case(t)) {
                    Some(i) => {
                        let b = ui.button(format!("{} {}", tag_label(t), crate::icons::CLOSE)).on_hover_text(tr!(palette_browser_remove_tag));
                        if b.clicked() {
                            remove = Some(i);
                        }
                    }
                    None => {
                        ui.label(egui::RichText::new(tag_label(t)).weak());
                    }
                }
            }
        });
        match row {
            PaletteRow::Preset(_) => {
                ui.label(egui::RichText::new(tr!(palette_browser_tags_builtin)).weak().small());
            }
            PaletteRow::Saved(_) => {
                ui.horizontal_wrapped(|ui| {
                    let field = ui.add(
                        egui::TextEdit::singleline(&mut self.palette_browser.new_tag)
                            .hint_text(tr!(palette_browser_tag_hint))
                            .desired_width(160.0),
                    );
                    let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if ui.button(tr!(palette_browser_add_tag)).clicked() || enter {
                        add = normalize_user_tag(&self.palette_browser.new_tag);
                        self.palette_browser.new_tag.clear();
                    }
                    // The theme and purpose tags it does not have yet, one click each.
                    for d in TAGS.iter().filter(|d| !d.group.is_measured()) {
                        if !e.tags.iter().any(|t| t == d.id) && ui.small_button(format!("{} {}", crate::icons::ADD, tag_label(d.id))).clicked() {
                            add = Some(d.id.to_string());
                        }
                    }
                });
            }
        }
        if let PaletteRow::Saved(j) = row {
            let mut changed = false;
            if let Some(g) = self.saved_gradients.get_mut(j) {
                if let Some(i) = remove {
                    g.tags.remove(i);
                    changed = true;
                }
                if let Some(t) = add {
                    if !g.tags.iter().any(|x| x.eq_ignore_ascii_case(&t)) {
                        g.tags.push(t);
                        changed = true;
                    }
                }
            }
            if changed {
                self.save_gradient_library();
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/ui/palette_browser/tests.rs"]
mod tests;
