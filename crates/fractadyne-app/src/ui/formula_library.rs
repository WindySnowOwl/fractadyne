//! The formula library window (Fractal ▸ Formula library…), modelled on the bookmarks window: the
//! custom formulas saved by name, each applied, edited in the Custom formula dialog, exported or
//! deleted from its row, and Import / Export for whole files (`formula_library.rs` has the file,
//! the merge rules and the storage).

use crate::formula_library::{self as lib, SavedFormula};
use crate::FractadyneApp;
use fractadyne_core::ir::parse::parse;

#[derive(Default)]
pub(crate) struct FormulaLibraryWindow {
    pub(crate) open: bool,
    /// The name box beside "Add current formula".
    name: String,
    /// The entry whose Delete was pressed once: its row asks before anything is lost.
    confirm_delete: Option<String>,
}

/// One line of text, cut with an ellipsis at the width the row is given.
fn clipped(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) -> egui::Response {
    ui.add(egui::Label::new(text).truncate())
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
        SavedFormula {
            name: name.to_string(),
            source: d.source.clone(),
            params: d.params[..used].iter().map(|(re, im)| [re.clone(), im.clone()]).collect(),
        }
    }

    /// The APPLIED custom formula as a library entry named `name`, if there is one.
    fn current_formula_entry(&self, name: &str) -> Option<SavedFormula> {
        let c = self.custom.as_ref()?;
        Some(SavedFormula {
            name: name.to_string(),
            source: c.source.clone(),
            params: c.params[..c.params_used()].iter().map(|(re, im)| [re.to_string(), im.to_string()]).collect(),
        })
    }

    /// Which entry the view is showing, if any: compared by formula, not by name.
    pub(crate) fn live_library_formula(&self) -> Option<usize> {
        if self.fractal != crate::FractalKind::Custom {
            return None;
        }
        let now = self.current_formula_entry("")?;
        self.saved_formulas.iter().position(|f| f.same_formula(&now))
    }

    /// Open entry `i` in the Custom formula dialog, not applied: to change it, or save a variant.
    pub(crate) fn edit_library_formula(&mut self, i: usize) {
        let Some(e) = self.saved_formulas.get(i).cloned() else { return };
        self.formula_dialog.load_entry(&e);
        self.formula_dialog.open = true;
    }

    /// Show entry `i` — as the dialog's Apply would, with the dialog's text following it.
    pub(crate) fn apply_library_formula(&mut self, i: usize) {
        let Some(e) = self.saved_formulas.get(i).cloned() else { return };
        self.formula_dialog.load_entry(&e);
        let used = parse(&e.source).map(|f| f.param_count()).unwrap_or(0);
        let compiled = self
            .formula_dialog
            .parsed_params(used)
            .and_then(|p| crate::custom_formula::CustomFormula::compile(&e.source, &p));
        match compiled {
            Ok(c) => self.apply_custom_formula_async(c),
            Err(why) => {
                self.formula_dialog.error = Some(why.clone());
                self.pending_toast = Some(format!("\"{}\" can't be applied: {why}", e.name));
            }
        }
    }

    /// Import a formula file into the library. Nothing already there is replaced (see
    /// [`lib::merge`]); the toast says what was added, skipped and renamed.
    fn import_formula_file(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Fractadyne formulas", &["toml"])
            .add_filter("All files", &["*"])
            .set_directory(self.dialog_dir_default())
            .pick_file()
        else {
            return;
        };
        self.remember_dir(&path);
        let file = path.file_name().map_or_else(String::new, |f| f.to_string_lossy().into_owned());
        let read = match std::fs::metadata(&path) {
            Ok(m) if m.len() <= lib::FILE_MAX => std::fs::read_to_string(&path).map_err(|e| e.to_string()),
            Ok(_) => Err("far too large to be a formula file".to_string()),
            Err(e) => Err(e.to_string()),
        };
        match read.and_then(|text| lib::parse_file(&text)) {
            Ok(incoming) => {
                let report = lib::merge(&mut self.saved_formulas, incoming);
                crate::diag::log_line("formula", &format!("formula import from {}: {report:?}", path.display()));
                if report.added == 0 || self.save_formula_library() {
                    self.pending_toast = Some(report.sentence(&file));
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
        }
        let mut open = true;
        let mut close = false;
        let mut act: Option<Act> = None;
        let (mut add, mut import, mut export_all) = (false, false, false);
        let live = self.live_library_formula();
        let danger = crate::theme::danger_color(ctx);
        egui::Window::new("Formula library")
            .open(&mut open)
            .default_size([480.0, 460.0])
            .show(ctx, |ui| {
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
                ui.separator();
                if self.saved_formulas.is_empty() {
                    ui.label(
                        "No saved formulas yet. Save one from the Custom formula dialog, add the one \
                         the view shows, or import a file.",
                    );
                }
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, f) in self.saved_formulas.iter().enumerate() {
                        let reads = parse(&f.source);
                        ui.horizontal(|ui| {
                            // The marker first: a truncated name takes the rest of the row.
                            if live == Some(i) {
                                ui.label(egui::RichText::new("showing").small().color(ui.visuals().hyperlink_color));
                            }
                            clipped(ui, egui::RichText::new(&f.name).strong()).on_hover_text(&f.name);
                        });
                        clipped(ui, egui::RichText::new(f.one_line()).monospace().small())
                            .on_hover_text(egui::RichText::new(&f.source).monospace());
                        if !f.params.is_empty() {
                            let ps: Vec<String> = f
                                .params
                                .iter()
                                .enumerate()
                                .map(|(k, [re, im])| format!("p{} = {re}, {im}", k + 1))
                                .collect();
                            clipped(ui, egui::RichText::new(ps.join("   ")).weak().small());
                        }
                        if let Err(e) = &reads {
                            let why = format!("Doesn't read in this version: {e}");
                            clipped(ui, egui::RichText::new(&why).small().color(danger)).on_hover_text(why);
                        }
                        ui.horizontal(|ui| {
                            let apply = ui
                                .add_enabled_ui(reads.is_ok(), |ui| crate::theme::confirm_button(ui, "Apply"))
                                .inner
                                .on_hover_text("Show this formula");
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
        match act {
            Some(Act::Apply(i)) => self.apply_library_formula(i),
            Some(Act::Edit(i)) => self.edit_library_formula(i),
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
