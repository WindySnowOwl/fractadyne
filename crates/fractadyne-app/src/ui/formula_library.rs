//! The formula library window (Fractal ▸ Formula library…), modelled on the bookmarks window: the
//! custom formulas saved by name, each applied, edited in the Custom formula dialog, exported or
//! deleted from its row, and Import / Export for whole files (`formula_library.rs` has the file,
//! the merge rules and the storage) — and, on a second tab, the collection that comes with the app,
//! each entry applied at its starting view, opened in the dialog, or copied into the library.

use crate::formula_library::{self as lib, SavedFormula, StartView};
use crate::FractadyneApp;
use fractadyne_core::ir::parse::parse;

/// Which list the window shows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Shelf {
    Mine,
    Collection,
}

#[derive(Default)]
pub(crate) struct FormulaLibraryWindow {
    pub(crate) open: bool,
    /// The list shown. Until one is picked: the user's formulas, or the collection while there are
    /// none (a first look at the library finds something to apply).
    pub(crate) shelf: Option<Shelf>,
    /// The name box beside "Add current formula".
    name: String,
    /// The entry whose Delete was pressed once: its row asks before anything is lost.
    confirm_delete: Option<String>,
    /// Only the entries whose name or text holds this (in any case) are listed.
    filter: String,
    /// Whether each source reads, by source: an imported `.frm` file brings thousands of entries,
    /// too many to parse every frame.
    reads: std::collections::HashMap<String, Result<(), String>>,
}

impl FormulaLibraryWindow {
    /// The list's filter, as if typed (the UI test's filtered screen).
    pub(crate) fn set_filter(&mut self, text: &str) {
        self.filter = text.to_string();
    }
}

/// The text size of a typeset formula in the list, in points (the dialog's is 18).
const LIBRARY_PT: f32 = 14.0;

/// The most rows the list draws (each is laid out every frame); the filter finds the rest.
const ROWS_MAX: usize = 100;

/// One line of text, cut with an ellipsis at the width the row is given.
fn clipped(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) -> egui::Response {
    ui.add(egui::Label::new(text).truncate())
}

/// A row's formula: typeset while the formula dialog is in Textbook mode (the user's choice of
/// view) and it reads, else its text on one line; its parameters; why it does not read (`reads`).
fn formula_body(ui: &mut egui::Ui, f: &SavedFormula, reads: &Result<(), String>, textbook: bool, danger: egui::Color32) {
    let typeset =
        (textbook && reads.is_ok()).then(|| crate::ui::textbook::editor::typeset(ui, &f.source, LIBRARY_PT)).flatten();
    typeset
        .unwrap_or_else(|| clipped(ui, egui::RichText::new(f.one_line()).monospace().small()))
        .on_hover_text(egui::RichText::new(&f.source).monospace());
    if !f.params.is_empty() {
        let ps: Vec<String> =
            f.params.iter().enumerate().map(|(k, [re, im])| format!("p{} = {re}, {im}", k + 1)).collect();
        clipped(ui, egui::RichText::new(ps.join("   ")).weak().small());
    }
    if let Err(e) = reads {
        let why = format!("Doesn't read in this version: {e}");
        clipped(ui, egui::RichText::new(&why).small().color(danger)).on_hover_text(why);
    }
}

/// A name made safe to suggest as a file name: anything a path could use is replaced.
fn file_stem(name: &str) -> String {
    let stem: String =
        name.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' }).collect();
    let stem = stem.trim();
    if stem.is_empty() { "formula".to_string() } else { stem.to_string() }
}

impl FractadyneApp {
    /// Write the library; a failure loses durable work, so it is a toast, not a log line.
    fn save_formula_library(&mut self) -> bool {
        match lib::save(&self.saved_formulas) {
            Ok(()) => true,
            Err(e) => {
                self.pending_toast = Some(format!("Couldn't save the formula library: {e}"));
                false
            }
        }
    }

    /// Save `entry` under its name, replacing an entry of that name, and say which it did.
    pub(crate) fn save_to_formula_library(&mut self, entry: SavedFormula) {
        let Some(replaced) = lib::upsert(&mut self.saved_formulas, entry.clone()) else {
            self.pending_toast = Some("Nothing to save: the formula is empty.".to_string());
            return;
        };
        // The name as stored (tidied; a blank one named after the formula).
        let name = entry.tidy().map(|e| e.name).unwrap_or_default();
        if self.save_formula_library() {
            self.pending_toast = Some(if replaced {
                format!("Updated \"{name}\" in the formula library.")
            } else {
                format!("Saved \"{name}\" to the formula library.")
            });
        }
        self.formula_dialog.save_name = name;
    }

    /// The Custom formula dialog's text as a library entry named `name`: the parameters it reads, as
    /// typed.
    pub(crate) fn dialog_formula_entry(&self, name: &str) -> SavedFormula {
        let d = &self.formula_dialog;
        let used = parse(&d.source).map(|f| f.param_count()).unwrap_or(0);
        let mut entry = SavedFormula {
            name: name.to_string(),
            source: d.source.clone(),
            params: d.params[..used].iter().map(|(re, im)| [re.clone(), im.clone()]).collect(),
            ..Default::default()
        };
        // The view on screen goes with it only if it is this formula's: saving text that was never
        // applied must not pin it to a view of some other formula.
        if self.current_formula_entry("").is_some_and(|now| now.same_formula(&entry)) {
            entry.view = Some(self.current_start_view());
        }
        entry
    }

    /// The APPLIED custom formula as a library entry named `name`, with the view on screen, if a
    /// custom formula is showing.
    fn current_formula_entry(&self, name: &str) -> Option<SavedFormula> {
        let c = self.custom.as_ref().filter(|_| self.fractal == crate::FractalKind::Custom)?;
        Some(SavedFormula {
            name: name.to_string(),
            source: c.source.clone(),
            params: c.params[..c.params_used()].iter().map(|(re, im)| [re.to_string(), im.to_string()]).collect(),
            view: Some(self.current_start_view()),
            ..Default::default()
        })
    }

    /// The view on screen as a starting view: the centre to its full precision, the magnification,
    /// the iteration count, Julia mode and its constant.
    pub(crate) fn current_start_view(&self) -> StartView {
        let v = &self.viewport;
        StartView {
            center: [fractadyne_core::to_decimal_string(&v.center_x), fractadyne_core::to_decimal_string(&v.center_y)],
            zoom: lib::zoom_text(v.log2_magnification()),
            iterations: Some(self.render_cfg.max_iter),
            julia: self.julia_mode.then(|| [self.julia_c.0.to_string(), self.julia_c.1.to_string()]),
        }
    }

    /// Go to a starting view (after its formula is applied): centre, magnification, iterations,
    /// Julia mode. The centre is read at the precision its depth needs.
    pub(crate) fn apply_start_view(&mut self, v: &StartView) {
        let Some(l2) = v.log2_zoom() else { return };
        let target = fractadyne_core::precision_for_octaves(l2.max(0.0).ceil() as u64) + 64;
        let (Some(cx), Some(cy)) = (
            fractadyne_core::parse_bf_prec(v.center[0].trim(), target),
            fractadyne_core::parse_bf_prec(v.center[1].trim(), target),
        ) else {
            return;
        };
        self.julia_mode = v.julia.is_some() && self.fractal.supports_julia();
        if let Some(c) = v.julia_c() {
            self.julia_c = c;
        }
        self.viewport.set_center_log2mag(cx, cy, l2);
        if let Some(n) = v.iterations {
            self.render_cfg.max_iter = n;
        }
        self.center_expr = None;
        self.pointer.zoom_vel = 0.0;
        self.invalidate_refs();
        self.record_nav();
    }

    /// Which entry the view is showing, if any: compared by formula, not by name.
    pub(crate) fn live_library_formula(&self) -> Option<usize> {
        let now = self.current_formula_entry("")?;
        self.saved_formulas.iter().position(|f| f.same_formula(&now))
    }

    /// Open an entry (of the library or the collection) in the Custom formula dialog, not applied:
    /// to change it, or save a variant. Apply there still goes to its view while the text is its.
    fn edit_formula_entry(&mut self, e: &SavedFormula) {
        self.formula_dialog.load_entry(e);
        self.formula_dialog.open = true;
    }

    /// Show an entry — as the dialog's Apply would, with the dialog's text following it — at its
    /// starting view if it has one.
    fn apply_formula_entry(&mut self, e: &SavedFormula) {
        self.formula_dialog.load_entry(e);
        let used = parse(&e.source).map(|f| f.param_count()).unwrap_or(0);
        let compiled = self
            .formula_dialog
            .parsed_params(used)
            .and_then(|p| crate::custom_formula::CustomFormula::compile(&e.source, &p));
        match compiled {
            Ok(c) => self.apply_custom_formula_async(c, e.view.clone()),
            Err(why) => {
                self.formula_dialog.error = Some(why.clone());
                self.pending_toast = Some(format!("\"{}\" can't be applied: {why}", e.name));
            }
        }
    }

    /// Import a formula file into the library — ours, or Fractint's `.frm` (the entries that read
    /// in this version). Nothing already there is replaced (see [`lib::merge`]); the toast says what
    /// was added, skipped and renamed, and for a `.frm` how many did not read and why, most often.
    fn import_formula_file(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Formula files", &["toml", "frm"])
            .add_filter("Fractadyne formulas", &["toml"])
            .add_filter("Fractint formulas", &["frm"])
            .add_filter("All files", &["*"])
            .set_directory(self.dialog_dir_default())
            .pick_file()
        else {
            return;
        };
        self.remember_dir(&path);
        let file = path.file_name().map_or_else(String::new, |f| f.to_string_lossy().into_owned());
        let read = match std::fs::metadata(&path) {
            Ok(m) if m.len() <= lib::FILE_MAX => std::fs::read(&path).map_err(|e| e.to_string()),
            Ok(_) => Err("far too large to be a formula file".to_string()),
            Err(e) => Err(e.to_string()),
        };
        let frm_named = path.extension().is_some_and(|x| x.eq_ignore_ascii_case("frm"));
        let read = read.and_then(|bytes| {
            // Ours by its name, or by reading as ours; else a `.frm`, if anything in it reads.
            let ours = if frm_named {
                None
            } else {
                Some(String::from_utf8(bytes.clone()).map_err(|_| "not a text file".to_string()).and_then(|t| lib::parse_file(&t)))
            };
            match ours {
                Some(Ok(list)) => Ok((list, None)),
                ours => {
                    let frm = lib::from_frm(&bytes, &file);
                    match ours {
                        Some(Err(why)) if frm.formulas.is_empty() => Err(why),
                        _ if frm.formulas.is_empty() && frm.unread.is_empty() => Err("no formulas in it".to_string()),
                        _ => {
                            let unread = frm.sentence();
                            crate::diag::log_line("formula", &format!("frm import of {file}: unread {:?}", frm.unread));
                            Ok((frm.formulas, unread))
                        }
                    }
                }
            }
        });
        match read {
            Ok((incoming, unread)) => {
                let report = lib::merge(&mut self.saved_formulas, incoming);
                crate::diag::log_line("formula", &format!("formula import from {}: {report:?}", path.display()));
                if report.added == 0 || self.save_formula_library() {
                    let mut s = report.sentence(&file);
                    if let Some(u) = unread {
                        s.push(' ');
                        s.push_str(&u);
                    }
                    self.pending_toast = Some(s);
                }
            }
            Err(why) => self.pending_toast = Some(format!("Couldn't import \"{file}\": {why}.")),
        }
    }

    /// Export `list` (the whole library, or one entry) as a formula file.
    fn export_formula_file(&mut self, list: Vec<SavedFormula>, suggested: &str) {
        if list.is_empty() {
            return;
        }
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Fractadyne formulas", &["toml"])
            .set_file_name(format!("{}.toml", file_stem(suggested)))
            .set_directory(self.dialog_dir_default())
            .save_file()
        else {
            return;
        };
        self.remember_dir(&path);
        let file = path.file_name().map_or_else(String::new, |f| f.to_string_lossy().into_owned());
        self.pending_toast = Some(match std::fs::write(&path, lib::file_text(&list)) {
            Ok(()) if list.len() == 1 => format!("Exported \"{}\" to \"{file}\".", list[0].name),
            Ok(()) => format!("Exported {} formulas to \"{file}\".", list.len()),
            Err(e) => format!("Couldn't write \"{file}\": {e}"),
        });
    }

    pub(crate) fn draw_formula_library(&mut self, ctx: &egui::Context) {
        if !self.formula_library.open {
            return;
        }
        enum Act {
            Apply(usize),
            Edit(usize),
            Export(usize),
            AskDelete(String),
            Delete(usize),
            KeepIt,
            /// The collection's entry `i`.
            ApplyShipped(usize),
            EditShipped(usize),
            CopyShipped(usize),
        }
        let mut open = true;
        let mut close = false;
        let mut act: Option<Act> = None;
        let (mut add, mut import, mut export_all) = (false, false, false);
        let live = self.live_library_formula();
        let danger = crate::theme::danger_color(ctx);
        let textbook = self.formula_dialog.textbook;
        let shipped = lib::collection();
        let live_shipped = self.current_formula_entry("").and_then(|now| shipped.iter().position(|f| f.same_formula(&now)));
        let shelf = self.formula_library.shelf.unwrap_or(if self.saved_formulas.is_empty() {
            Shelf::Collection
        } else {
            Shelf::Mine
        });
        let mut picked = shelf;
        egui::Window::new("Formula library")
            .open(&mut open)
            .default_size([480.0, 460.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut picked, Shelf::Mine, format!("My formulas ({})", self.saved_formulas.len()))
                        .on_hover_text("The formulas you saved or imported");
                    ui.selectable_value(&mut picked, Shelf::Collection, format!("Collection ({})", shipped.len()))
                        .on_hover_text("Formulas that come with Fractadyne, each with a view to start from");
                });
                ui.separator();
                if shelf == Shelf::Collection {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let mut heading = "";
                        for (i, f) in shipped.iter().enumerate() {
                            if f.category != heading {
                                heading = &f.category;
                                ui.add_space(6.0);
                                ui.label(egui::RichText::new(heading).heading().small());
                            }
                            ui.horizontal(|ui| {
                                if live_shipped == Some(i) {
                                    ui.label(egui::RichText::new("showing").small().color(ui.visuals().hyperlink_color));
                                }
                                clipped(ui, egui::RichText::new(&f.name).strong()).on_hover_text(&f.name);
                            });
                            formula_body(ui, f, &Ok(()), textbook, danger);
                            ui.label(egui::RichText::new(&f.about).weak().small());
                            ui.horizontal(|ui| {
                                if crate::theme::confirm_button(ui, "Apply").on_hover_text("Show it, at its starting view").clicked() {
                                    act = Some(Act::ApplyShipped(i));
                                }
                                if ui
                                    .button(format!("{} Edit", crate::icons::EDIT))
                                    .on_hover_text("Open it in the Custom formula dialog, without applying it")
                                    .clicked()
                                {
                                    act = Some(Act::EditShipped(i));
                                }
                                if ui
                                    .button(format!("{} Copy to mine", crate::icons::ADD))
                                    .on_hover_text("Add it to your formulas, to keep or change")
                                    .clicked()
                                {
                                    act = Some(Act::CopyShipped(i));
                                }
                            });
                            ui.separator();
                        }
                    });
                    ui.separator();
                    crate::theme::action_row(ui, |ui| {
                        if crate::theme::cancel_button(ui, "Close").clicked() {
                            close = true;
                        }
                    });
                    return;
                }
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.formula_library.name)
                            .hint_text("name (optional)")
                            .desired_width(220.0),
                    );
                    add = ui
                        .add_enabled(
                            self.custom.is_some(),
                            egui::Button::new(format!("{} Add current formula", crate::icons::ADD)),
                        )
                        .on_hover_text("Save the formula the view shows")
                        .on_disabled_hover_text("Apply a custom formula first (Fractal > Custom formula…)")
                        .clicked();
                });
                ui.horizontal(|ui| {
                    import = ui
                        .button(format!("{} Import…", crate::icons::IMPORT))
                        .on_hover_text("Add the formulas in a formula file; nothing here is replaced")
                        .clicked();
                    export_all = ui
                        .add_enabled(
                            !self.saved_formulas.is_empty(),
                            egui::Button::new(format!("{} Export all…", crate::icons::SAVE)),
                        )
                        .on_hover_text("Write every formula here to one file, to share or keep")
                        .clicked();
                });
                ui.horizontal(|ui| {
                    ui.label("Filter");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.formula_library.filter)
                            .hint_text("filter by name or text")
                            .desired_width(220.0),
                    );
                });
                ui.separator();
                if self.saved_formulas.is_empty() {
                    ui.label(
                        "No saved formulas yet. Save one from the Custom formula dialog, add the one \
                         the view shows, or import a file (ours, or Fractint's .frm).",
                    );
                }
                let needle = self.formula_library.filter.trim().to_lowercase();
                let shown: Vec<usize> = (0..self.saved_formulas.len())
                    .filter(|&i| {
                        let f = &self.saved_formulas[i];
                        needle.is_empty() || f.name.to_lowercase().contains(&needle) || f.source.to_lowercase().contains(&needle)
                    })
                    .collect();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for &i in shown.iter().take(ROWS_MAX) {
                        let f = &self.saved_formulas[i];
                        let reads = self
                            .formula_library
                            .reads
                            .entry(f.source.clone())
                            .or_insert_with(|| parse(&f.source).map(|_| ()).map_err(|e| e.to_string()))
                            .clone();
                        ui.horizontal(|ui| {
                            // The marker first: a truncated name takes the rest of the row.
                            if live == Some(i) {
                                ui.label(egui::RichText::new("showing").small().color(ui.visuals().hyperlink_color));
                            }
                            clipped(ui, egui::RichText::new(&f.name).strong()).on_hover_text(&f.name);
                        });
                        formula_body(ui, f, &reads, textbook, danger);
                        if !f.about.is_empty() {
                            clipped(ui, egui::RichText::new(&f.about).weak().small()).on_hover_text(&f.about);
                        }
                        ui.horizontal(|ui| {
                            let apply = ui
                                .add_enabled_ui(reads.is_ok(), |ui| crate::theme::confirm_button(ui, "Apply"))
                                .inner
                                .on_hover_text(if f.view.is_some() {
                                    "Show this formula, at the view saved with it"
                                } else {
                                    "Show this formula"
                                });
                            if apply.clicked() {
                                act = Some(Act::Apply(i));
                            }
                            if ui
                                .button(format!("{} Edit", crate::icons::EDIT))
                                .on_hover_text("Open it in the Custom formula dialog, without applying it")
                                .clicked()
                            {
                                act = Some(Act::Edit(i));
                            }
                            if ui.button(crate::icons::SAVE).on_hover_text("Export this formula to a file").clicked() {
                                act = Some(Act::Export(i));
                            }
                            if self.formula_library.confirm_delete.as_deref() == Some(f.name.as_str()) {
                                ui.label(egui::RichText::new("Delete it?").color(danger));
                                if ui.button("Delete").clicked() {
                                    act = Some(Act::Delete(i));
                                }
                                if ui.button("Keep").clicked() {
                                    act = Some(Act::KeepIt);
                                }
                            } else if ui.button(crate::icons::DELETE).on_hover_text("Delete").clicked() {
                                act = Some(Act::AskDelete(f.name.clone()));
                            }
                        });
                        ui.separator();
                    }
                    if shown.len() > ROWS_MAX {
                        let more = crate::commas(&(shown.len() - ROWS_MAX).to_string());
                        ui.label(egui::RichText::new(format!("…and {more} more: type in the filter to find one.")).weak());
                    } else if shown.is_empty() && !self.saved_formulas.is_empty() {
                        ui.label(egui::RichText::new("None match the filter.").weak());
                    }
                });
                ui.separator();
                crate::theme::action_row(ui, |ui| {
                    if crate::theme::cancel_button(ui, "Close").clicked() {
                        close = true;
                    }
                });
            });
        if add {
            let name = self.formula_library.name.trim().to_string();
            if let Some(e) = self.current_formula_entry(&name) {
                self.save_to_formula_library(e);
                self.formula_library.name.clear();
            }
        }
        if import {
            self.import_formula_file();
        }
        if export_all {
            self.export_formula_file(self.saved_formulas.clone(), "fractadyne-formulas");
        }
        if picked != shelf {
            self.formula_library.shelf = Some(picked);
        }
        match act {
            Some(Act::Apply(i)) => {
                if let Some(e) = self.saved_formulas.get(i).cloned() {
                    self.apply_formula_entry(&e);
                }
            }
            Some(Act::Edit(i)) => {
                if let Some(e) = self.saved_formulas.get(i).cloned() {
                    self.edit_formula_entry(&e);
                }
            }
            Some(Act::ApplyShipped(i)) => {
                if let Some(e) = shipped.get(i) {
                    self.apply_formula_entry(e);
                }
            }
            Some(Act::EditShipped(i)) => {
                if let Some(e) = shipped.get(i) {
                    self.edit_formula_entry(e);
                }
            }
            Some(Act::CopyShipped(i)) => {
                if let Some(e) = shipped.get(i).cloned() {
                    let name = e.name.clone();
                    let report = lib::merge(&mut self.saved_formulas, vec![e]);
                    if report.added == 0 {
                        self.pending_toast = Some(format!("\"{name}\" is already in your formulas."));
                    } else if self.save_formula_library() {
                        let as_named = report.renamed.first().map_or(name.clone(), |(_, to)| to.clone());
                        self.pending_toast = Some(format!("Copied \"{name}\" to your formulas as \"{as_named}\"."));
                    }
                }
            }
            Some(Act::Export(i)) => {
                if let Some(e) = self.saved_formulas.get(i).cloned() {
                    let name = e.name.clone();
                    self.export_formula_file(vec![e], &name);
                }
            }
            Some(Act::AskDelete(name)) => self.formula_library.confirm_delete = Some(name),
            Some(Act::KeepIt) => self.formula_library.confirm_delete = None,
            Some(Act::Delete(i)) => {
                self.formula_library.confirm_delete = None;
                if i < self.saved_formulas.len() {
                    let gone = self.saved_formulas.remove(i);
                    if self.save_formula_library() {
                        self.pending_toast = Some(format!("Deleted \"{}\" from the formula library.", gone.name));
                    }
                }
            }
            None => {}
        }
        self.formula_library.open = open && !close;
        if !self.formula_library.open {
            self.formula_library.confirm_delete = None;
        }
    }
}
