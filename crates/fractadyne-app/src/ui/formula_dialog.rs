//! The "Custom formula" dialog (design/custom-formulas.md §4.8, first cut): a text field for the
//! step in Fractint-style expressions, the parameters it reads, a syntax check as you type, and
//! Apply. Apply compiles the formula into its shader (~0.1 s) and shows it; the view is kept when a
//! custom formula is already showing, so a formula can be refined in place.

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
}

impl Default for FormulaDialog {
    fn default() -> Self {
        FormulaDialog {
            open: false,
            source: "z = z^2 + c".into(),
            params: std::array::from_fn(|_| ("0".to_string(), "0".to_string())),
            error: None,
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
        self.set_fractal(crate::FractalKind::Custom);
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
                ui.add(
                    egui::TextEdit::multiline(&mut self.formula_dialog.source)
                        .font(egui::TextStyle::Monospace)
                        .desired_rows(4)
                        .desired_width(f32::INFINITY)
                        .hint_text("z = z^2 + c"),
                );
                ui.label(
                    egui::RichText::new(
                        "z the iterate, c the pixel, p1…p5 parameters; + − * / ^, |z| (squared \
                         modulus), sqr abs conj real imag cabs flip recip exp log sqrt sin cos tan \
                         sinh cosh tanh. Statements separated by a new line or a comma; ; starts a comment.",
                    )
                    .weak()
                    .small(),
                );
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
                if let Some(c) = self.custom.as_ref().filter(|_| self.fractal == crate::FractalKind::Custom) {
                    ui.label(egui::RichText::new(c.depth_note()).weak().small());
                }
                if let Some(e) = &self.formula_dialog.error {
                    ui.colored_label(egui::Color32::from_rgb(0xE0, 0x6C, 0x60), e);
                }
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    apply = ui
                        .add_enabled(check.is_ok(), egui::Button::new("Apply"))
                        .on_hover_text("Compile the formula and show it")
                        .clicked();
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
                    self.apply_custom_formula(c);
                }
                Err(e) => self.formula_dialog.error = Some(e),
            }
        }
        self.formula_dialog.open = open;
    }
}
