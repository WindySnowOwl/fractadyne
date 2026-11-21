//! Color ▸ Custom coloring…: the coloring hook in use (design/coloring-plan.md B10), where it came
//! from, why its file's last save does not build, its parameters as sliders, and the file actions:
//! open a file (watched: a save recolors the view), start from an example, save the hook in use as a
//! file (to edit one that came with a view or an example), turn it off. `COLORING-HOOKS.md` is the
//! guide to writing one.
//!
//! The state is the session's [`crate::coloring_hook::HookState`]; this window only shows and sets it.

use super::panels::slider_double_clicked;
use fractadyne_gpu::color_hook::{EXAMPLES, MAX_PARAMS};

/// The width of the parameters' names: a name longer than this is shortened (in full when hovered).
/// ⚠A truncating label in a grid shrinks its column to nothing ("Line …", the first screenshots).
const NAME_W: f32 = 170.0;

/// The guide on GitHub, for a build without `COLORING-HOOKS.md` beside it (a development build).
const GUIDE_URL: &str = "https://github.com/WindySnowOwl/fractadyne/blob/main/COLORING-HOOKS.md";

/// The window's own state: whether it is open, and the last file action's outcome.
#[derive(Default)]
pub(crate) struct ColorHookDialog {
    pub(crate) open: bool,
    /// "Saved to …" or why it could not be: shown until the next action.
    message: Option<String>,
}

/// What a click in the window asked for, done after the window has drawn.
enum Action {
    Open,
    Example(&'static str),
    SaveAs,
    TurnOff,
    Guide,
}

/// The examples' labels, by their file names.
fn example_label(file: &str) -> String {
    match file {
        "smooth.wgsl" => tr!(color_hook_example_smooth),
        "fringe.wgsl" => tr!(color_hook_example_fringe),
        "transfer-curves.wgsl" => tr!(color_hook_example_transfer),
        "darken-near-set.wgsl" => tr!(color_hook_example_darken),
        other => other.to_string(),
    }
}

/// A path's file name, for a line that should not show the user's folders.
fn file_name(p: &std::path::Path) -> String {
    p.file_name().map_or_else(|| p.display().to_string(), |s| s.to_string_lossy().into_owned())
}

impl crate::FractadyneApp {
    pub(crate) fn draw_color_hook_dialog(&mut self, ctx: &egui::Context) {
        if !self.color_hook_dialog.open {
            return;
        }
        let (mut open, mut close) = (true, false);
        let mut action: Option<Action> = None;
        egui::Window::new(tr!(color_hook_title))
            .id(egui::Id::new("fractadyne.color-hook"))
            .open(&mut open)
            .default_width(480.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_max_width(480.0);
                ui.label(egui::RichText::new(tr!(color_hook_intro)).weak());
                ui.add_space(6.0);
                let hook = &self.coloring.hook;
                let status = match (&hook.active, &hook.path) {
                    (None, _) => tr!(color_hook_off),
                    (Some(_), Some(p)) => tr!(color_hook_from_file, file = file_name(p)),
                    (Some(_), None) => tr!(color_hook_from_view),
                };
                ui.label(status);
                if let Some(e) = &hook.error {
                    ui.label(egui::RichText::new(tr!(color_hook_error, error = e.clone())).color(ui.visuals().error_fg_color));
                    if hook.active.is_some() {
                        ui.label(egui::RichText::new(tr!(color_hook_error_kept)).small().weak());
                    }
                }
                // The parameters the hook uses or names, each on its own slider; a double-click
                // gives back the hook's default.
                if let Some(active) = hook.active.clone() {
                    let shown: Vec<usize> = (0..MAX_PARAMS).filter(|&i| active.used[i] || active.named(i)).collect();
                    if !shown.is_empty() {
                        ui.add_space(6.0);
                        ui.label(egui::RichText::new(tr!(color_hook_params)).strong());
                        egui::Grid::new("color-hook-params").num_columns(2).spacing([12.0, 4.0]).show(ui, |ui| {
                            for i in shown {
                                let p = &active.params[i];
                                // The name is the hook's own (its author's words), as written.
                                let row_h = ui.spacing().interact_size.y;
                                ui.allocate_ui_with_layout(egui::vec2(NAME_W, row_h), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                    ui.set_min_width(NAME_W);
                                    ui.add(egui::Label::new(&p.name).truncate()).on_hover_text(&p.name);
                                });
                                let v = &mut self.coloring.hook.params[i];
                                let r = ui.add(egui::Slider::new(v, p.min..=p.max));
                                if slider_double_clicked(&r) {
                                    *v = p.default;
                                }
                                r.on_hover_text(tr!(panel_double_click_reset));
                                ui.end_row();
                            }
                        });
                    }
                }
                ui.separator();
                let active = self.coloring.hook.active.is_some();
                ui.horizontal_wrapped(|ui| {
                    if ui.button(tr!(color_hook_load)).on_hover_text(tr!(color_hook_load_hover)).clicked() {
                        action = Some(Action::Open);
                    }
                    ui.menu_button(tr!(color_hook_examples), |ui| {
                        for (file, text) in EXAMPLES {
                            if ui.button(example_label(file)).clicked() {
                                action = Some(Action::Example(text));
                                ui.close_menu();
                            }
                        }
                    })
                    .response
                    .on_hover_text(tr!(color_hook_examples_hover));
                    if ui.add_enabled(active, egui::Button::new(tr!(color_hook_save_as))).on_hover_text(tr!(color_hook_save_as_hover)).clicked() {
                        action = Some(Action::SaveAs);
                    }
                    if ui.add_enabled(active, egui::Button::new(tr!(color_hook_turn_off))).on_hover_text(tr!(color_hook_turn_off_hover)).clicked() {
                        action = Some(Action::TurnOff);
                    }
                    if ui.button(tr!(color_hook_guide)).on_hover_text(tr!(color_hook_guide_hover)).clicked() {
                        action = Some(Action::Guide);
                    }
                });
                if let Some(m) = &self.color_hook_dialog.message {
                    ui.label(egui::RichText::new(m).small().weak());
                }
                ui.separator();
                crate::theme::action_row(ui, |ui| {
                    if crate::theme::cancel_button(ui, &tr!(dlg_close)).clicked() {
                        close = true;
                    }
                });
            });
        match action {
            Some(Action::Open) => self.color_hook_open_file(),
            Some(Action::Example(text)) => {
                self.color_hook_dialog.message = None;
                if self.coloring.hook.set_text(text, None, None).is_ok() {
                    crate::diag::log_line("hook", "an example coloring hook is in use");
                }
            }
            Some(Action::SaveAs) => self.color_hook_save_as(),
            Some(Action::TurnOff) => {
                self.color_hook_dialog.message = None;
                self.coloring.hook.turn_off();
            }
            Some(Action::Guide) => self.open_color_hook_guide(ctx),
            None => {}
        }
        self.color_hook_dialog.open = open && !close;
    }

    /// Open…: a `.wgsl` file, loaded and watched. One that does not build changes nothing but the
    /// error the window shows.
    fn color_hook_open_file(&mut self) {
        let Some(path) = crate::diag::modal(|| rfd::FileDialog::new()
            .add_filter(tr!(color_hook_filter), &["wgsl", "txt"])
            .set_directory(self.dialog_dir_default())
            .pick_file())
        else {
            return;
        };
        self.remember_dir(&path);
        self.color_hook_dialog.message = None;
        match self.coloring.hook.load_file(&path) {
            Ok(()) => crate::diag::log_line("hook", &format!("the coloring hook {} is in use, its file watched", file_name(&path))),
            Err(e) => crate::diag::log_line("hook", &format!("the coloring hook {} does not build: {e}", file_name(&path))),
        }
    }

    /// Save as…: the hook in use written to a file, which is then the one watched (an example's or a
    /// view's hook has no file until it is saved), its parameters kept.
    fn color_hook_save_as(&mut self) {
        let Some(hook) = self.coloring.hook.active.clone() else { return };
        let name = self.coloring.hook.path.as_deref().map_or_else(|| "coloring-hook.wgsl".to_string(), file_name);
        let Some(path) = crate::diag::modal(|| rfd::FileDialog::new()
            .add_filter(tr!(color_hook_filter), &["wgsl"])
            .set_directory(self.dialog_dir_default())
            .set_file_name(name)
            .save_file())
        else {
            return;
        };
        self.remember_dir(&path);
        let params = self.coloring.hook.params;
        self.color_hook_dialog.message = Some(match fractadyne_state::durable::write_atomic(&path, hook.text.as_bytes()) {
            Ok(()) => {
                // The same text, now with a file to watch.
                let _ = self.coloring.hook.set_text(&hook.text, Some(&params), Some(path.clone()));
                tr!(color_hook_saved, path = path.display().to_string())
            }
            Err(e) => tr!(color_hook_save_failed, error = e.to_string()),
        });
    }

    /// The guide: `COLORING-HOOKS.md` beside the program, else on GitHub. Never during the scripted
    /// UI walk, which must not open windows of other programs.
    fn open_color_hook_guide(&self, ctx: &egui::Context) {
        if self.harness.uitest.is_some() {
            return;
        }
        let local = std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("COLORING-HOOKS.md"))).filter(|p| p.is_file());
        let url = match local {
            Some(p) => format!("file:///{}", p.display().to_string().replace('\\', "/")),
            None => GUIDE_URL.to_string(),
        };
        ctx.open_url(egui::OpenUrl::new_tab(url));
    }
}
