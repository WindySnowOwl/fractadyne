//! The "Custom formula" dialog (design/custom-formulas.md §4.8, first cut): a text field for the
//! step in Fractint-style expressions, the parameters it reads, a syntax check as you type, and
//! Apply. Apply generates the formula's shader, compiles its pipelines off the render thread
//! (~1.4 s on the RTX 3080) while the view keeps rendering, and then shows it; the view is kept when a custom
//! formula is already showing, so a formula can be refined in place.

use crate::custom_formula::CustomFormula;
use crate::FractadyneApp;
use fractadyne_core::ir::parse::{parse, MAX_PARAMS};

/// Starting points, each a family the built-ins do not have. `(label, source, parameters)`.
const EXAMPLES: &[(&str, &str, &[(f64, f64)])] = &[
    ("z² + c", "z = z^2 + c", &[]),
    ("Cubic with a parameter", "z = z^3 - p1*z + c", &[(0.5, 0.0)]),
    ("Hybrid square", "t = sqr(z)\nz = t + p1*conj(t) + c", &[(0.25, 0.0)]),
    ("Perpendicular Burning Ship", "z = (real(z) - flip(abs(imag(z))))^2 + c", &[]),
    ("Sine", "z = sin(z) + c", &[]),
    ("Exponential", "z = exp(z) + c", &[]),
];

pub(crate) struct FormulaDialog {
    pub(crate) open: bool,
    pub(crate) source: String,
    /// `p1`…`p5` as typed, (re, im).
    pub(crate) params: [(String, String); MAX_PARAMS],
    /// Why the last Apply did not take (a parse error names its line and column).
    pub(crate) error: Option<String>,
    /// The keypad's open tab.
    pub(crate) tab: crate::ui::formula_keypad::Tab,
    /// The text field's completion list.
    pub(crate) completion: crate::ui::formula_editor::Completion,
    /// The textbook layout's type specimen window (opened by the uitest only).
    pub(crate) specimen: bool,
    /// Show the formula typeset (Textbook) instead of as text; persisted with the session.
    pub(crate) textbook: bool,
    /// The Textbook editor's document, caret and history (it follows `source`).
    pub(crate) editor: crate::ui::textbook::edit::Editor,
    /// An applied formula whose pipelines are compiling off the render thread. The view keeps
    /// showing what it shows until they are ready, then switches (`poll_formula_compile`).
    pub(crate) pending: Option<PendingFormula>,
    /// The name the text is saved under in the formula library; follows the entry last loaded or
    /// saved, so saving again updates it.
    pub(crate) save_name: String,
}

/// A formula waiting for its pipelines (`fractadyne_gpu::compile_custom_async`).
pub(crate) struct PendingFormula {
    formula: CustomFormula,
    rx: std::sync::mpsc::Receiver<fractadyne_gpu::PreparedCustom>,
    started: std::time::Instant,
}

impl Default for FormulaDialog {
    fn default() -> Self {
        FormulaDialog {
            open: false,
            source: "z = z^2 + c".into(),
            params: std::array::from_fn(|_| ("0".to_string(), "0".to_string())),
            error: None,
            tab: Default::default(),
            completion: Default::default(),
            specimen: false,
            textbook: false,
            editor: Default::default(),
            pending: None,
            save_name: String::new(),
        }
    }
}

impl FormulaDialog {
    fn set_params(&mut self, params: &[(f64, f64)]) {
        for (i, slot) in self.params.iter_mut().enumerate() {
            let (re, im) = params.get(i).copied().unwrap_or((0.0, 0.0));
            *slot = (re.to_string(), im.to_string());
        }
    }

    /// Put a library entry in the dialog: its text, its parameters as typed (the rest 0), its name.
    pub(crate) fn load_entry(&mut self, e: &crate::formula_library::SavedFormula) {
        self.source = e.source.clone();
        for (i, slot) in self.params.iter_mut().enumerate() {
            *slot = match e.params.get(i) {
                Some([re, im]) => (re.clone(), im.clone()),
                None => ("0".to_string(), "0".to_string()),
            };
        }
        self.save_name = e.name.clone();
        self.error = None;
    }

    /// The typed parameters, or which one is not a number.
    pub(crate) fn parsed_params(&self, used: usize) -> Result<Vec<(f64, f64)>, String> {
        self.params[..used]
            .iter()
            .enumerate()
            .map(|(i, (re, im))| {
                let num = |s: &str| s.trim().parse::<f64>().ok().filter(|v| v.is_finite());
                match (num(re), num(im)) {
                    (Some(re), Some(im)) => Ok((re, im)),
                    _ => Err(format!("p{} is not a pair of numbers", i + 1)),
                }
            })
            .collect()
    }
}

impl FractadyneApp {
    /// Open the dialog on the current custom formula (or the default one).
    pub(crate) fn open_formula_dialog(&mut self) {
        if let Some(c) = self.custom.clone() {
            self.formula_dialog.source = c.source.clone();
            self.formula_dialog.set_params(&c.params);
            // Named as its library entry, if it is one, so Save updates that entry.
            if let Some(i) = self.live_library_formula() {
                self.formula_dialog.save_name = self.saved_formulas[i].name.clone();
            }
        }
        self.formula_dialog.error = None;
        self.formula_dialog.open = true;
    }

    /// Make `c` the custom formula and show it. Switching INTO Custom resets to its home view like
    /// any family switch; re-applying while it shows keeps the view.
    pub(crate) fn apply_custom_formula(&mut self, c: CustomFormula) {
        crate::calibration::set_custom_factor(c.shader.cost_factor);
        crate::diag::log_line(
            "formula",
            &format!(
                "custom formula applied: {:?} (params {:?}, {:?} precision, cost factor {:.2})",
                c.source,
                &c.params[..c.params_used()],
                c.shader.precision,
                c.shader.cost_factor
            ),
        );
        self.custom = Some(std::sync::Arc::new(c));
        if self.fractal == crate::FractalKind::Custom {
            // `set_fractal` is a no-op for the family already shown, but the step changed: a
            // reference orbit (or one in flight) is the OLD formula's and must not be reused.
            self.invalidate_refs();
        }
        self.set_fractal(crate::FractalKind::Custom);
    }

    /// Apply `c` once its pipelines are compiled OFF the render thread, which otherwise builds them
    /// on the first frame that draws it (`prepare_custom_now`): ~1.4 s of frozen window on every
    /// Apply, and every parameter change is a new module. Until then the view keeps rendering what it shows. With
    /// no renderer to compile against, applies at once (the render thread compiles).
    pub(crate) fn apply_custom_formula_async(&mut self, c: CustomFormula) {
        let rx = self
            .render_state
            .as_ref()
            .and_then(|rs| fractadyne_gpu::compile_custom_async(rs, c.shader.clone()));
        match rx {
            // A newer Apply replaces an older one still compiling; that worker's result is dropped.
            Some(rx) => {
                self.formula_dialog.pending =
                    Some(PendingFormula { formula: c, rx, started: std::time::Instant::now() })
            }
            None => self.apply_custom_formula(c),
        }
    }

    /// Switch to a pending formula whose pipelines have arrived: install them, then apply, in the
    /// same update — so the frame that first draws the formula finds them ready.
    pub(crate) fn poll_formula_compile(&mut self, ctx: &egui::Context) {
        let Some(p) = self.formula_dialog.pending.as_ref() else { return };
        match p.rx.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                ctx.request_repaint_after(std::time::Duration::from_millis(30));
            }
            got => {
                let p = self.formula_dialog.pending.take().expect("checked above");
                match (got, self.render_state.as_ref()) {
                    (Ok(prepared), Some(rs)) => {
                        crate::diag::log_line(
                            "formula",
                            &format!(
                                "custom formula pipelines compiled off the render thread in {:.0} ms",
                                p.started.elapsed().as_secs_f64() * 1000.0
                            ),
                        );
                        fractadyne_gpu::install_custom(rs, prepared);
                    }
                    // The worker died (or the renderer went away): the render thread compiles.
                    _ => crate::diag::log_line("formula", "off-thread compile failed; the render thread compiles"),
                }
                self.apply_custom_formula(p.formula);
            }
        }
    }

    pub(crate) fn draw_formula_dialog(&mut self, ctx: &egui::Context) {
        if self.formula_dialog.specimen {
            crate::ui::textbook::specimen_window(ctx, &mut self.formula_dialog.specimen);
        }
        if !self.formula_dialog.open {
            return;
        }
        let mut open = true;
        let mut apply = false;
        let (mut save, mut library) = (false, false);
        let mut example: Option<usize> = None;
        // The syntax check runs on every frame the dialog is open: parsing is microseconds, and a
        // message that describes the text on screen can never be stale.
        let check = parse(&self.formula_dialog.source);
        let used = check.as_ref().map(|f| f.param_count()).unwrap_or(0);
        egui::Window::new("Custom formula")
            .open(&mut open)
            .resizable(true)
            .default_width(460.0)
            .show(ctx, |ui| {
                let was_textbook = self.formula_dialog.textbook;
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("The step, in Fractint-style expressions").weak().small());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Laid out right to left: Textbook | Text appear as Text | Textbook.
                        ui.selectable_value(&mut self.formula_dialog.textbook, true, "Textbook")
                            .on_hover_text("Show the formula typeset, as a textbook would set it");
                        ui.selectable_value(&mut self.formula_dialog.textbook, false, "Text")
                            .on_hover_text("Edit the formula as text");
                        ui.separator();
                        egui::ComboBox::from_id_salt("formula_examples")
                            .selected_text("Examples")
                            .show_ui(ui, |ui| {
                                for (i, (label, src, _)) in EXAMPLES.iter().enumerate() {
                                    if ui.selectable_label(false, *label).on_hover_text(*src).clicked() {
                                        example = Some(i);
                                    }
                                }
                            });
                    });
                });
                let text_id = egui::Id::new("formula_source");
                let book_id = egui::Id::new("formula_textbook");
                // Text mode at source byte `at`, its field focused.
                let to_text = |ui: &egui::Ui, src: &str, at: usize| {
                    let ch = src[..at.min(src.len())].chars().count();
                    crate::ui::formula_editor::store_cursor(ui.ctx(), text_id, ch);
                    ui.ctx().memory_mut(|m| m.request_focus(text_id));
                };
                // The toggle moves the caret across: to the line the other mode's caret was on.
                if self.formula_dialog.textbook != was_textbook {
                    let src = &self.formula_dialog.source;
                    if self.formula_dialog.textbook {
                        let ch = crate::ui::formula_editor::stored_cursor(ui.ctx(), text_id).unwrap_or(0);
                        let byte = src.char_indices().nth(ch).map_or(src.len(), |(b, _)| b);
                        self.formula_dialog.editor.sync(src);
                        self.formula_dialog.editor.place_at(byte);
                        ui.ctx().memory_mut(|m| m.request_focus(book_id));
                    } else {
                        to_text(ui, src, self.formula_dialog.editor.caret_offset());
                    }
                }
                if self.formula_dialog.textbook {
                    let out = crate::ui::textbook::editor::show(
                        ui,
                        book_id,
                        &mut self.formula_dialog.editor,
                        &mut self.formula_dialog.source,
                        96.0,
                        check.as_ref().err().map(|e| e.line),
                    );
                    if out.changed {
                        self.formula_dialog.error = None;
                    }
                    // A line that does not read is edited as text.
                    if let Some(at) = out.to_text {
                        self.formula_dialog.textbook = false;
                        to_text(ui, &self.formula_dialog.source, at);
                    }
                } else {
                    crate::ui::formula_editor::source_field(
                        ui,
                        text_id,
                        &mut self.formula_dialog.source,
                        4,
                        "z = z^2 + c",
                        &mut self.formula_dialog.completion,
                    );
                }
                ui.label(
                    egui::RichText::new(if self.formula_dialog.textbook {
                        "Type as in the text: / makes a fraction of the term before it, ^ an exponent, \
                         ( parentheses. The arrows move through the formula; Tab goes to the next empty \
                         box. A line in red does not read: click it to edit it as text."
                    } else {
                        "Type, or use the keypad — it holds every name the formula language knows. \
                         Names complete as you type (Tab). Statements are separated by a new line or a \
                         comma; ; starts a comment."
                    })
                    .weak()
                    .small(),
                );
                if let Some(action) = crate::ui::formula_keypad::show(ui, &mut self.formula_dialog.tab) {
                    let ctx = ui.ctx().clone();
                    // Editing is textual for now: a key in the Textbook view goes to the text.
                    self.formula_dialog.textbook = false;
                    crate::ui::formula_keypad::press(&ctx, text_id, &mut self.formula_dialog.source, action);
                    self.formula_dialog.error = None;
                }
                match &check {
                    Ok(_) => {
                        // Plain words: the UI font has no check-mark glyph (it drew a box).
                        ui.label(egui::RichText::new("Reads correctly.").small().color(ui.visuals().hyperlink_color));
                    }
                    Err(e) => {
                        // Typeset, the text's columns are nowhere to be seen: the line is underlined,
                        // and an empty box (which prints as `()`) is named as what it is.
                        let msg = match self.formula_dialog.textbook {
                            true if self.formula_dialog.editor.has_empty_box() => "Fill the empty box.".to_string(),
                            true => format!("Line {}: {}", e.line, e.message),
                            false => e.to_string(),
                        };
                        ui.colored_label(egui::Color32::from_rgb(0xE0, 0x6C, 0x60), msg);
                    }
                }
                if used > 0 {
                    ui.add_space(4.0);
                    egui::Grid::new("formula_params").num_columns(3).show(ui, |ui| {
                        for (i, (re, im)) in self.formula_dialog.params[..used].iter_mut().enumerate() {
                            ui.label(format!("p{}", i + 1));
                            ui.add(egui::TextEdit::singleline(re).desired_width(140.0).hint_text("re"));
                            ui.add(egui::TextEdit::singleline(im).desired_width(140.0).hint_text("im"));
                            ui.end_row();
                        }
                    });
                }
                if let Ok(f) = &check {
                    ui.label(egui::RichText::new(crate::custom_formula::depth_note_for(f)).weak().small());
                }
                if let Some(e) = &self.formula_dialog.error {
                    ui.colored_label(egui::Color32::from_rgb(0xE0, 0x6C, 0x60), e);
                }
                ui.add_space(4.0);
                let compiling = self.formula_dialog.pending.is_some();
                ui.horizontal(|ui| {
                    apply = ui
                        .add_enabled(check.is_ok(), egui::Button::new("Apply"))
                        .on_hover_text("Compile the formula and show it")
                        .clicked();
                    // Always laid out, blank when idle, so the row does not reflow as it comes and
                    // goes.
                    let note = if compiling { "Compiling the formula for the GPU…" } else { "" };
                    ui.label(egui::RichText::new(note).weak().small());
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.formula_dialog.save_name)
                            .hint_text("name (optional)")
                            .desired_width(180.0),
                    );
                    // Saving under a name the library holds UPDATES that entry, and says so first.
                    let name = self.formula_dialog.save_name.trim();
                    let label = if !name.is_empty() && self.saved_formulas.iter().any(|f| f.name == name) {
                        "Update in library"
                    } else {
                        "Save to library"
                    };
                    save = ui
                        .add_enabled(check.is_ok(), egui::Button::new(format!("{} {label}", crate::icons::SAVE)))
                        .on_hover_text("Keep this formula and its parameters in the formula library")
                        .on_disabled_hover_text("Only a formula that reads correctly can be saved")
                        .clicked();
                    library = ui
                        .button("Library…")
                        .on_hover_text("The saved formulas: apply, edit, import and export them")
                        .clicked();
                });
            });
        if let Some(i) = example {
            let (label, src, params) = EXAMPLES[i];
            self.formula_dialog.source = src.to_string();
            self.formula_dialog.set_params(params);
            self.formula_dialog.error = None;
            // Not the name of the entry last loaded: saving an example must not update that entry.
            self.formula_dialog.save_name = label.to_string();
        }
        if save {
            let name = self.formula_dialog.save_name.trim().to_string();
            let entry = self.dialog_formula_entry(&name);
            self.save_to_formula_library(entry);
        }
        if library {
            self.formula_library.open = true;
        }
        if apply {
            let result = self
                .formula_dialog
                .parsed_params(used)
                .and_then(|p| CustomFormula::compile(&self.formula_dialog.source, &p));
            match result {
                Ok(c) => {
                    self.formula_dialog.error = None;
                    self.apply_custom_formula_async(c);
                }
                Err(e) => self.formula_dialog.error = Some(e),
            }
        }
        self.formula_dialog.open = open;
    }
}
