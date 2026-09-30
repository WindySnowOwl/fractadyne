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
    /// An applied formula whose pipelines are compiling off the render thread. The view keeps
    /// showing what it shows until they are ready, then switches (`poll_formula_compile`).
    pub(crate) pending: Option<PendingFormula>,
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
            pending: None,
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

    /// The typed parameters, or which one is not a number.
    fn parsed_params(&self, used: usize) -> Result<Vec<(f64, f64)>, String> {
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
        if !self.formula_dialog.open {
            return;
        }
        let mut open = true;
        let mut apply = false;
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
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("The step, in Fractint-style expressions").weak().small());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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
                ui.add(
                    egui::TextEdit::multiline(&mut self.formula_dialog.source)
                        .id(text_id)
                        .font(egui::TextStyle::Monospace)
                        .desired_rows(4)
                        .desired_width(f32::INFINITY)
                        .hint_text("z = z^2 + c"),
                );
                ui.label(
                    egui::RichText::new(
                        "Type, or use the keypad — it holds every name the formula language knows. \
                         Statements are separated by a new line or a comma; ; starts a comment.",
                    )
                    .weak()
                    .small(),
                );
                if let Some(action) = crate::ui::formula_keypad::show(ui, &mut self.formula_dialog.tab) {
                    let ctx = ui.ctx().clone();
                    crate::ui::formula_keypad::press(&ctx, text_id, &mut self.formula_dialog.source, action);
                    self.formula_dialog.error = None;
                }
                match &check {
                    Ok(_) => {
                        // Plain words: the UI font has no check-mark glyph (it drew a box).
                        ui.label(egui::RichText::new("Reads correctly.").small().color(ui.visuals().hyperlink_color));
                    }
                    Err(e) => {
                        ui.colored_label(egui::Color32::from_rgb(0xE0, 0x6C, 0x60), e.to_string());
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
            });
        if let Some(i) = example {
            let (_, src, params) = EXAMPLES[i];
            self.formula_dialog.source = src.to_string();
            self.formula_dialog.set_params(params);
            self.formula_dialog.error = None;
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
