//! Choosing the interface's language (design/multilingual.md §8): the language menu at the end of
//! the top bar, which no mode hides; the Languages window behind it; and the question asked once
//! when the system's language changes.

use crate::FractadyneApp;
use crate::i18n::catalog::Catalog;
use crate::i18n::langs::{self, Installed};
use crate::i18n::{self, Arg, DecimalMark, Source, id, pseudo};
use std::sync::Arc;

/// The language in use, the setting, and what the language menu lists.
pub(crate) struct LanguageState {
    /// The language in use (a tag).
    pub(crate) tag: String,
    /// The setting saved in `session.toml`: the language in use unless this run was told another
    /// (`--language`, `FRACTADYNE_LANG`, or a harness, which runs in English).
    pub(crate) saved: String,
    /// The system language found at the last launch (`SessionState::language_system_seen`).
    pub(crate) system_seen: String,
    /// The decimal mark the user prefers (`SessionState::decimal_mark`), over the language's own.
    pub(crate) decimal_mark: DecimalMark,
    /// The installed languages, as discovered at launch and on Reload.
    pub(crate) installed: Vec<Installed>,
    pub(crate) window_open: bool,
    /// Each installed language's coverage and messages set aside, read when the window opens.
    coverage: Option<Vec<(u32, usize)>>,
    /// The question asked once when the system's language changed: the system language as
    /// reported, the installed language that matches it, and that language's catalog (the question
    /// is asked in it as well).
    ask: Option<(String, String, Arc<Catalog>)>,
    /// A notice for the first frame: a language that could not be loaded.
    pub(crate) notice: Option<String>,
}

impl LanguageState {
    /// The language for this launch. `saved`, `system_seen` and `decimal` are the session's; `run`
    /// a language for this run only. On the first run (`saved` empty) the system's languages decide;
    /// on a later launch whose system language has changed to one the app has, the question is
    /// queued.
    pub(crate) fn start(saved: &str, system_seen: &str, decimal: &str, run: Option<String>) -> LanguageState {
        let installed = langs::discover();
        let system = langs::system_languages();
        let system_top = system.first().cloned().unwrap_or_default();
        let matched = langs::best_match(&system, &installed);
        let mut st = LanguageState {
            tag: "en".into(),
            saved: saved.to_owned(),
            system_seen: system_seen.to_owned(),
            decimal_mark: DecimalMark::from_key(decimal),
            installed,
            window_open: false,
            coverage: None,
            ask: None,
            notice: None,
        };
        if saved.is_empty() {
            st.saved = matched.clone().unwrap_or_else(|| "en".into());
            st.system_seen = system_top.clone();
        } else if !system_top.is_empty() && system_top != system_seen {
            match matched.as_deref().filter(|m| *m != saved) {
                Some(m) => {
                    if let Some(c) = st.installed.iter().find(|i| i.tag == m).and_then(|i| langs::load(i).ok()) {
                        st.ask = Some((system_top.clone(), m.to_owned(), Arc::new(c)));
                    }
                }
                // Nothing to ask (the app lacks it, or it is the language in use): just record it.
                None => st.system_seen = system_top.clone(),
            }
        }
        let want = run.unwrap_or_else(|| st.saved.clone());
        st.activate(&want);
        i18n::set_decimal_mark(st.decimal_mark);
        st
    }

    /// The user chose the decimal mark `m`: numbers take it from the next frame on, and it is kept.
    pub(crate) fn choose_decimal_mark(&mut self, m: DecimalMark) {
        self.decimal_mark = m;
        i18n::set_decimal_mark(m);
    }

    /// Use the language `tag` (or the installed one that best matches it). Falls back to English,
    /// with a notice, when its file cannot be loaded.
    fn activate(&mut self, tag: &str) -> bool {
        let found = if tag == pseudo::TAG {
            Some(Installed {
                tag: pseudo::TAG.into(),
                name: pseudo::NAME.into(),
                english_name: pseudo::ENGLISH_NAME.into(),
                revision: 0,
                source: Source::Builtin,
            })
        } else {
            let exact = self.installed.iter().find(|i| i.tag == tag).cloned();
            exact.or_else(|| {
                let m = langs::best_match(&[tag.to_owned()], &self.installed)?;
                self.installed.iter().find(|i| i.tag == m).cloned()
            })
        };
        let Some(i) = found else {
            i18n::set_active(i18n::english());
            self.tag = "en".into();
            return tag == "en";
        };
        match langs::load(&i) {
            Ok(c) => {
                i18n::set_active(Arc::new(c));
                self.tag = i.tag;
                true
            }
            Err(why) => {
                i18n::set_active(i18n::english());
                self.tag = "en".into();
                self.notice = Some(tr!(language_load_failed, name = i.name.as_str(), why = why));
                false
            }
        }
    }

    /// The user chose `tag` (the menu, the Languages window, the welcome window): use it and keep it.
    pub(crate) fn choose(&mut self, tag: &str) {
        if self.activate(tag) {
            self.saved = self.tag.clone();
        }
    }

    /// Read the language files again (after editing or adding one), keeping the language in use.
    pub(crate) fn reload(&mut self) {
        self.installed = langs::discover();
        self.coverage = None;
        let tag = self.tag.clone();
        self.activate(&tag);
    }

    /// The language in use's own name.
    pub(crate) fn current_name(&self) -> String {
        if self.tag == pseudo::TAG {
            return pseudo::NAME.into();
        }
        self.installed.iter().find(|i| i.tag == self.tag).map_or_else(|| self.tag.clone(), |i| i.name.clone())
    }

    fn coverage(&mut self) -> &[(u32, usize)] {
        if self.coverage.is_none() {
            let c = self.installed.iter().map(|i| langs::load(i).map_or((0, 0), |c| (c.coverage(), c.dropped.len()))).collect();
            self.coverage = Some(c);
        }
        self.coverage.as_deref().unwrap_or(&[])
    }
}

/// The language for this run only: `--language TAG`, else `FRACTADYNE_LANG`, else English for a
/// harness or offline job (their checks read English text), else none (the saved choice).
pub(crate) fn run_language(args: &[String]) -> Option<String> {
    let flag = args.windows(2).find(|w| w[0] == "--language").map(|w| w[1].clone());
    flag.or_else(|| std::env::var(langs::LANG_ENV).ok().filter(|v| !v.is_empty()))
        .or_else(|| crate::is_task_invocation(args).then(|| "en".to_owned()))
}

/// The decimal-mark choice: its heading, then one line per choice showing what a number looks like
/// with it (`1,234.5` / `1.234,5`), the language's own first. Returns the one clicked.
fn decimal_mark_choice(ui: &mut egui::Ui, current: DecimalMark) -> Option<DecimalMark> {
    let mut chosen = None;
    ui.label(egui::RichText::new(tr!(language_decimal_mark)).weak());
    for m in DecimalMark::ALL {
        let sample = i18n::decimal_with(1234.5, 1, false, i18n::punctuation_with(m));
        let label = match m {
            DecimalMark::Language => tr!(language_decimal_language, sample = sample),
            DecimalMark::Point => tr!(language_decimal_point, sample = sample),
            DecimalMark::Comma => tr!(language_decimal_comma, sample = sample),
        };
        if ui.selectable_label(current == m, label).on_hover_text(tr!(language_decimal_mark_hover)).clicked() {
            chosen = Some(m);
        }
    }
    chosen
}

/// How the Languages window names where a language comes from.
fn source_text(s: &Source) -> String {
    match s {
        Source::Builtin => tr!(language_source_builtin),
        Source::Package(_) => tr!(language_source_package),
        Source::User(_) => tr!(language_source_user),
        Source::Dev(_) => tr!(language_source_dev),
    }
}

impl FractadyneApp {
    /// The language menu: the end of the top bar, in every mode. Its list is never translated —
    /// each language in its own name, its English name beside it — so it reads the same from any
    /// language, and the icon is the one thing on the bar that needs no reading.
    pub(crate) fn draw_language_menu(&mut self, ui: &mut egui::Ui) {
        let mut pick = None;
        let mut mark = None;
        let mut more = false;
        let current = self.language.current_name();
        ui.menu_button(format!("{}  {}", crate::icons::LANGUAGES, current), |ui| {
            for i in &self.language.installed {
                if ui.selectable_label(i.tag == self.language.tag, i.menu_label()).clicked() {
                    pick = Some(i.tag.clone());
                    ui.close_menu();
                }
            }
            ui.separator();
            if let Some(m) = decimal_mark_choice(ui, self.language.decimal_mark) {
                mark = Some(m);
            }
            ui.separator();
            if ui.button(tr!(language_more)).clicked() {
                more = true;
                ui.close_menu();
            }
        });
        if let Some(t) = pick {
            self.language.choose(&t);
        }
        if let Some(m) = mark {
            self.language.choose_decimal_mark(m);
        }
        if more {
            self.language.window_open = true;
        }
    }

    /// The Languages window: what is installed, how much of it is translated, where it is from;
    /// install a file, open the folder, read the files again.
    pub(crate) fn draw_language_window(&mut self, ctx: &egui::Context) {
        if !self.language.window_open {
            return;
        }
        let mut open = true;
        let mut pick = None;
        let mut mark = None;
        let (mut install, mut reload, mut folder) = (false, false, false);
        let rows: Vec<(Installed, (u32, usize))> = {
            let cov = self.language.coverage().to_vec();
            self.language.installed.iter().cloned().zip(cov).collect()
        };
        egui::Window::new(tr!(language_window_title))
            .id(egui::Id::new("fractadyne.languages"))
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .default_width(520.0)
            .show(ctx, |ui| {
                ui.add(egui::Label::new(tr!(language_window_intro)).wrap());
                ui.add_space(6.0);
                egui::Grid::new("fractadyne.languages.grid").striped(true).num_columns(4).show(ui, |ui| {
                    ui.label(egui::RichText::new(tr!(language_col_language)).weak());
                    ui.label(egui::RichText::new(tr!(language_col_translated)).weak());
                    ui.label(egui::RichText::new(tr!(language_col_from)).weak());
                    ui.end_row();
                    for (i, (pct, dropped)) in &rows {
                        let label = egui::RichText::new(i.menu_label());
                        ui.label(if i.tag == self.language.tag { label.strong() } else { label });
                        let cov = ui.label(tr!(language_coverage, percent = *pct));
                        if *dropped > 0 {
                            cov.on_hover_text(tr!(language_dropped, count = *dropped));
                        }
                        let from = ui.label(source_text(&i.source));
                        if let Some(p) = i.path() {
                            from.on_hover_text(p.display().to_string());
                        }
                        if i.tag != self.language.tag && ui.small_button(tr!(language_use)).clicked() {
                            pick = Some(i.tag.clone());
                        }
                        ui.end_row();
                    }
                });
                ui.add_space(8.0);
                if let Some(m) = decimal_mark_choice(ui, self.language.decimal_mark) {
                    mark = Some(m);
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    install = ui.button(tr!(language_install)).on_hover_text(tr!(language_install_hover)).clicked();
                    folder = ui.button(tr!(language_open_folder)).clicked();
                    reload = ui.button(tr!(language_reload)).on_hover_text(tr!(language_reload_hover)).clicked();
                });
            });
        self.language.window_open = open;
        if let Some(t) = pick {
            self.language.choose(&t);
        }
        if let Some(m) = mark {
            self.language.choose_decimal_mark(m);
        }
        if reload {
            self.language.reload();
        }
        if folder {
            if let Some(d) = langs::user_dir() {
                let _ = std::fs::create_dir_all(&d);
                let _ = super::dialogs::open_in_file_manager(&d);
            }
        }
        if install {
            if let Some(path) = rfd::FileDialog::new().add_filter(tr!(language_file_filter), &["ftl"]).pick_file() {
                let msg = self.install_language_file(&path);
                self.set_toast(msg, ctx);
            }
        }
    }

    /// Check `path` as a language file and copy it into `<config>/languages/`; the notice to show.
    fn install_language_file(&mut self, path: &std::path::Path) -> String {
        let file = path.file_name().map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned());
        let fail = |why: String| tr!(language_install_failed, file = file.as_str(), why = why);
        let text = match std::fs::read(path) {
            Ok(b) if b.len() <= i18n::catalog::MAX_FILE_BYTES => match String::from_utf8(b) {
                Ok(t) => t,
                Err(_) => return fail(tr!(language_install_not_utf8)),
            },
            Ok(_) => return fail(tr!(language_install_too_large)),
            Err(e) => return fail(e.to_string()),
        };
        let stamp = i18n::catalog::Stamp::read(&i18n::catalog::clean_text(&text));
        let hint = stamp.tag.or_else(|| {
            let stem = path.file_stem()?.to_string_lossy().into_owned();
            Some(stem.strip_prefix("fractadyne-").unwrap_or(&stem).to_owned())
        });
        let Some(tag) = hint.as_deref().and_then(langs::canonical_tag) else {
            return fail(tr!(language_install_no_tag));
        };
        if tag == "en" || tag == pseudo::TAG {
            return fail(tr!(language_install_english));
        }
        let c = match Catalog::from_text(&text, &tag) {
            Ok(c) => c,
            Err(why) => return fail(why),
        };
        if c.translated == 0 {
            return fail(tr!(language_install_empty));
        }
        let Some(dir) = langs::user_dir() else { return fail(tr!(language_install_no_folder)) };
        let dest = dir.join(format!("fractadyne-{tag}.ftl"));
        let tmp = dir.join(format!("fractadyne-{tag}.ftl.tmp"));
        let wrote = std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&tmp, &text)).and_then(|_| std::fs::rename(&tmp, &dest));
        if let Err(e) = wrote {
            return fail(e.to_string());
        }
        self.language.reload();
        tr!(language_installed, name = c.name.as_str(), file = file.as_str())
    }

    /// The question asked once when the system's language has changed to one the app has: in that
    /// language and in the one in use, so it reads in either.
    pub(crate) fn draw_language_prompt(&mut self, ctx: &egui::Context) {
        let Some((system, tag, cat)) = self.language.ask.clone() else { return };
        let new_name = cat.name.clone();
        let cur_name = self.language.current_name();
        let mut answer: Option<bool> = None;
        egui::Window::new(tr!(language_changed_title))
            .id(egui::Id::new("fractadyne.language.changed"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                let theirs = cat.format(id::language_changed_question, &[("new", Arg::from(new_name.as_str()))]);
                let ours = tr!(language_changed_question, new = new_name.as_str());
                ui.label(&theirs);
                if ours != theirs {
                    ui.label(egui::RichText::new(ours).weak());
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button(cat.format(id::language_changed_use, &[("name", Arg::from(new_name.as_str()))])).clicked() {
                        answer = Some(true);
                    }
                    if ui.button(tr!(language_changed_keep, name = cur_name.as_str())).clicked() {
                        answer = Some(false);
                    }
                });
            });
        if let Some(switch) = answer {
            if switch {
                self.language.choose(&tag);
            }
            self.language.system_seen = system;
            self.language.ask = None;
        }
    }

    /// The welcome window's language row: the same list as the language menu.
    pub(crate) fn welcome_language_row(&mut self, ui: &mut egui::Ui) {
        let mut pick = None;
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(tr!(welcome_language)).weak().small());
            egui::ComboBox::from_id_salt("fractadyne.welcome.language")
                .selected_text(format!("{}  {}", crate::icons::LANGUAGES, self.language.current_name()))
                .show_ui(ui, |ui| {
                    for i in &self.language.installed {
                        if ui.selectable_label(i.tag == self.language.tag, i.menu_label()).clicked() {
                            pick = Some(i.tag.clone());
                        }
                    }
                });
        });
        if let Some(t) = pick {
            self.language.choose(&t);
        }
    }
}
