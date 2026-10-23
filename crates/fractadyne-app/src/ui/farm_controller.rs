//! Tools ▸ Render on farm…: render the loaded tour on several machines (design/remote-rendering.md
//! §11). The controller runs as a child process (`--farm-render … --ui-status`), like the Render
//! tour window's render: this window reads its status lines and sends it commands. Clients join
//! from File ▸ Render client… on their machines.
//!
//! The tour is captured when the window opens: a farm job can run for hours and must not end
//! because the tour player was closed. Closing the window leaves the job running; quitting the app
//! ends stdin, and the controller stops the job (it resumes from the output folder later).

use crate::farm::status::{ClientRow, ControllerCommand, ControllerPhase, ControllerStatus, Link, FLAG};
use crate::FractadyneApp;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub(crate) struct FarmControllerUi {
    pub(crate) open: bool,
    pub(crate) script: Option<PathBuf>,
    pub(crate) tour_name: String,
    pub(crate) total_s: f64,
    pub(crate) out: String,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) custom_size: bool,
    pub(crate) fps: f64,
    pub(crate) ss: u32,
    pub(crate) port: u16,
    pub(crate) min_clients: u32,
    pub(crate) local: bool,
    /// This machine's path to the shared drive (share mode), or "" for streaming only.
    pub(crate) share_root: String,
    /// Pass reference orbits between machines (design §8); off: each builds its own.
    pub(crate) sharing: bool,
    pub(crate) link: Option<Link<ControllerStatus>>,
    /// The last status (and output) of a controller that has ended — or the UI walk's sample.
    pub(crate) last: Option<ControllerStatus>,
    pub(crate) last_log: Vec<String>,
    pub(crate) error: Option<String>,
    key: Option<String>,
    show_key: bool,
    stop_sent: Option<Instant>,
    /// The UI walk's seeded window draws as a running job (Stop, Pause, Remove) with no process
    /// behind it. Drawing only: nothing else — the live view, the menu — takes it for a job.
    pub(crate) uitest_live: bool,
}

impl Default for FarmControllerUi {
    fn default() -> Self {
        Self {
            open: false,
            script: None,
            tour_name: String::new(),
            total_s: 0.0,
            out: String::new(),
            width: 1920,
            height: 1080,
            custom_size: false,
            fps: 30.0,
            ss: 1,
            port: fractadyne_farm::DEFAULT_PORT,
            min_clients: 1,
            local: true,
            share_root: String::new(),
            sharing: true,
            link: None,
            last: None,
            last_log: Vec::new(),
            error: None,
            key: None,
            show_key: false,
            stop_sent: None,
            uitest_live: false,
        }
    }
}

impl FarmControllerUi {
    pub(crate) fn active(&self) -> bool {
        self.link.as_ref().is_some_and(|l| l.running())
    }

    /// This machine's GPU is busy with the job (its own client, or measuring the anchors).
    pub(crate) fn rendering_here(&self) -> bool {
        self.active()
            && self.link.as_ref().and_then(|l| l.status.as_ref()).is_some_and(|s| s.phase == ControllerPhase::Preparing || (self.local && s.phase == ControllerPhase::Rendering))
    }

    fn status(&self) -> Option<&ControllerStatus> {
        self.link.as_ref().and_then(|l| l.status.as_ref()).or(self.last.as_ref())
    }

    /// The `--farm-render` line this window describes — shared by Render and Copy command.
    fn args(&self) -> Vec<String> {
        let mut a = vec![
            "--farm-render".to_string(),
            self.script.as_ref().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default(),
            "--out".to_string(),
            absolute(&self.out).to_string_lossy().into_owned(),
            "--size".to_string(),
            format!("{}x{}", self.width, self.height),
            "--fps".to_string(),
            format!("{}", self.fps),
            "--ss".to_string(),
            self.ss.to_string(),
            "--listen".to_string(),
            format!("0.0.0.0:{}", self.port),
            "--min-clients".to_string(),
            self.min_clients.to_string(),
        ];
        if self.local {
            a.push("--local".into());
        }
        if !self.share_root.trim().is_empty() {
            a.push("--share-root".into());
            a.push(self.share_root.trim().to_string());
        }
        if !self.sharing {
            a.push("--sharing".into());
            a.push("off".into());
        }
        a
    }

    fn reload_key(&mut self) {
        self.key = crate::farm::farm_dir().ok().and_then(|d| std::fs::read_to_string(d.join("farm-key.txt")).ok()).map(|k| k.trim().to_string());
    }
}

fn absolute(p: &str) -> PathBuf {
    let p = std::path::Path::new(p);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(p)
    }
}

fn quote(s: &str) -> String {
    if s.contains(' ') {
        format!("\"{s}\"")
    } else {
        s.to_string()
    }
}

/// A job's time, in whole seconds: "42 s", "38m 10s", "2h 05m".
fn duration(s: f64) -> String {
    let s = s.max(0.0).round() as u64;
    match s {
        0..60 => format!("{s} s"),
        60..3600 => format!("{}m {:02}s", s / 60, s % 60),
        _ => format!("{}h {:02}m", s / 3600, s % 3600 / 60),
    }
}

/// Free space as a person reads it: "412 GB", "51.7 GB", "830 MB".
fn space(b: u64) -> String {
    let gb = b as f64 / 1e9;
    if gb >= 100.0 {
        format!("{gb:.0} GB")
    } else if gb >= 1.0 {
        format!("{gb:.1} GB")
    } else {
        format!("{:.0} MB", b as f64 / 1e6)
    }
}

/// The phase in words, and whether it is a good (green), working (accent) or bad (red) state.
fn phase_words(st: &ControllerStatus) -> (String, i8) {
    match st.phase {
        ControllerPhase::Waiting => (
            tr!(farm_ctl_phase_waiting, connected = st.clients.iter().filter(|c| !c.removed && c.state != "gone").count(), needed = st.min_clients),
            0,
        ),
        ControllerPhase::Preparing => (tr!(farm_ctl_phase_preparing), 0),
        ControllerPhase::Rendering => (tr!(farm_ctl_phase_rendering), 1),
        ControllerPhase::Paused => (tr!(farm_ctl_phase_paused), 0),
        ControllerPhase::Finished if st.exit_code == Some(0) => (tr!(farm_ctl_phase_complete), 1),
        ControllerPhase::Finished if st.exit_code == Some(4) => (tr!(farm_ctl_phase_stopped), 0),
        ControllerPhase::Finished => (tr!(farm_ctl_phase_ended), -1),
    }
}

/// The words after the phase (`ControllerStatus::detail`) in the language in use. The controller
/// process writes them in English (it never loads a language, and its status line is data between
/// two processes), so the window translates the ones it knows — `farm::controller`'s
/// `ui_status` — where it draws them, and shows any other text, such as the error a controller
/// ended on, as it came.
fn detail_words(d: &str) -> String {
    match d {
        "Measuring the normalize anchors on this machine, once for every client" => return tr!(farm_ctl_detail_preparing),
        "Rendering" => return tr!(farm_ctl_phase_rendering),
        "Waiting for space in the output folder" => return tr!(farm_ctl_detail_no_space),
        "Paused: no new work is handed out" => return tr!(farm_ctl_detail_paused),
        "Complete" => return tr!(farm_ctl_phase_complete),
        "Finished with frames not rendered — run again on the same folder to resume" => return tr!(farm_ctl_detail_not_rendered),
        "Stopped — run again on the same folder to resume" => return tr!(farm_ctl_detail_stopped),
        "Finishing" => return tr!(farm_ctl_detail_finishing),
        _ => {}
    }
    if let Some(c) = d.strip_prefix("Ended (exit ").and_then(|r| r.strip_suffix(')')) {
        return tr!(farm_ctl_detail_ended, code = c);
    }
    let waiting = d.strip_prefix("Waiting for render clients: ").and_then(|r| r.strip_suffix(" connected")).and_then(|r| r.split_once(" of "));
    if let Some((Ok(have), Ok(need))) = waiting.map(|(a, b)| (a.parse::<u64>(), b.parse::<u64>())) {
        return tr!(farm_ctl_detail_waiting, connected = have, needed = need);
    }
    d.to_string()
}

/// A client row's state (`ClientRow::state`) in the language in use: the controller process
/// writes it in English (see `detail_words`), and this window compares it as it came ("gone",
/// "removing…"), so it is translated only here, for the eye. A state this does not know is shown
/// as it came.
fn state_words(r: &ClientRow) -> String {
    let s = r.state.as_str();
    match s {
        "removed by you" => return tr!(farm_ctl_state_removed),
        "removing — collecting its diagnostics" => return tr!(farm_ctl_state_removing),
        "gone" => return tr!(farm_ctl_state_gone),
        "leaving" => return tr!(farm_ctl_state_leaving),
        "checking (self-check)" => return tr!(farm_ctl_state_checking),
        "paused by its user" => return tr!(farm_ctl_state_paused),
        "parked: crashed twice — rejoins when it reconnects" => return tr!(farm_ctl_state_parked),
        "refused this job (its own limits)" => return tr!(farm_ctl_state_refused),
        "starting a run" => return tr!(farm_ctl_state_starting),
        "idle" => return tr!(farm_ctl_state_idle),
        "admitted — waiting for the job" => return tr!(farm_ctl_state_admitted),
        _ => {}
    }
    // "removed: 2 bad frames": the count is the row's strikes.
    if s.starts_with("removed: ") && r.strikes > 0 {
        return tr!(farm_ctl_state_removed_bad, count = r.strikes);
    }
    // "rendering 7310 (9/16)": the frame, and its place in the run.
    let rendering = s
        .strip_prefix("rendering ")
        .and_then(|t| t.strip_suffix(')'))
        .and_then(|t| t.split_once(" ("))
        .and_then(|(f, t)| t.split_once('/').map(|(i, n)| (f, i, n)));
    if let Some((frame, index, count)) = rendering {
        return tr!(farm_ctl_state_rendering, frame = frame, index = index, count = count);
    }
    r.state.clone()
}

/// A client row's GPU column: its class letter, and why.
fn gpu_cell(r: &ClientRow) -> (String, String) {
    let letter = r.gpu_class.clone().unwrap_or_else(|| "—".into());
    let why = match r.probe_px {
        Some(0) => tr!(farm_ctl_probe_same),
        Some(px) => tr!(farm_ctl_probe_differs, count = px),
        None => tr!(farm_ctl_probe_unknown),
    };
    let adapter = if r.adapter.is_empty() { String::new() } else { format!("\n{}", r.adapter) };
    let driver = if r.driver.is_empty() { String::new() } else { format!("\n{}", tr!(farm_ctl_gpu_driver, driver = &r.driver)) };
    let note = r.gpu_note.as_ref().map_or(String::new(), |n| format!("\n⚠ {}", tr!(farm_ctl_gpu_note, note = n)));
    let letter = if r.gpu_note.is_some() { format!("{letter} ⚠") } else { letter };
    (letter, format!("{why}{adapter}{driver}{note}"))
}

impl FractadyneApp {
    /// Open the farm window for the loaded tour (seeded from its `[render]` block), or as it is
    /// while a job runs.
    pub(crate) fn open_farm_controller(&mut self) {
        let f = &mut self.farm_controller;
        f.reload_key();
        if f.active() {
            f.open = true;
            return;
        }
        let Some(pb) = &self.playback else { return };
        let Some(script) = pb.source.clone() else { return };
        let r = &pb.render;
        (f.width, f.height) = match (r.width, r.height) {
            (Some(w), Some(h)) => (w, h),
            (Some(w), None) => (w, (w * 9 / 16).max(16)),
            _ => (1920, 1080),
        };
        f.custom_size = false;
        f.fps = r.fps.unwrap_or(30.0);
        f.ss = r.ss.unwrap_or(1);
        f.tour_name = pb.name.clone();
        f.total_s = pb.total;
        if f.script.as_ref() != Some(&script) || f.out.is_empty() {
            // Beside the script, named for it: a farm job's frames should be easy to find.
            let stem = script.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "tour".into());
            f.out = r.out.clone().unwrap_or_else(|| script.parent().map(|d| d.join(format!("{stem}-farm"))).unwrap_or_else(|| PathBuf::from(format!("{stem}-farm")))).display().to_string();
        }
        f.script = Some(script);
        f.error = None;
        f.open = true;
    }

    pub(crate) fn poll_farm_controller(&mut self, ctx: &egui::Context) {
        let Some(l) = self.farm_controller.link.as_mut() else { return };
        if l.poll() {
            ctx.request_repaint();
        }
        if !l.running() {
            let exit = l.exit.flatten();
            let mut st = l.status.clone().unwrap_or_default();
            st.phase = ControllerPhase::Finished;
            st.exit_code = st.exit_code.or(exit);
            let log: Vec<String> = l.log.iter().cloned().collect();
            if !matches!(exit, Some(0) | Some(3) | Some(4)) && self.farm_controller.error.is_none() {
                self.farm_controller.error = log.iter().rev().find(|x| x.starts_with("! ")).map(|x| x.trim_start_matches("! ").to_string()).or(Some(tr!(farm_ctl_ended_exit, code = format!("{exit:?}"))));
            }
            self.farm_controller.link = None;
            self.farm_controller.last = Some(st);
            self.farm_controller.last_log = log;
            self.farm_controller.stop_sent = None;
            self.farm_controller.reload_key();
            if self.render_cfg.finish_sound && self.harness.uitest.is_none() {
                crate::tone::play_finish_sound(false);
            }
        } else {
            ctx.request_repaint_after(Duration::from_millis(250));
        }
    }

    fn start_farm_controller(&mut self) {
        let f = &mut self.farm_controller;
        f.error = None;
        if f.script.is_none() {
            f.error = Some(tr!(farm_ctl_no_tour));
            return;
        }
        if let Err(e) = std::fs::create_dir_all(absolute(&f.out)) {
            f.error = Some(tr!(farm_ctl_out_error, folder = &f.out, why = e.to_string()));
            return;
        }
        let mut args = f.args();
        args.push(FLAG.into());
        // Its own log folder: it must not write the session's frame record (`frames.bin`).
        let mut env = Vec::new();
        if let Some(d) = crate::diag::logs_dir() {
            env.push(("FRACTADYNE_LOG_DIR", d.join("farm-controller").into_os_string()));
        }
        match Link::spawn(&args, &env) {
            Ok(l) => {
                crate::diag::breadcrumb(format!("render farm → {}", f.out));
                f.link = Some(l);
                f.last = None;
                f.last_log.clear();
            }
            Err(e) => f.error = Some(tr!(farm_ctl_start_failed, why = e.to_string())),
        }
    }

    pub(crate) fn draw_farm_controller_window(&mut self, ctx: &egui::Context) {
        if !self.farm_controller.open {
            return;
        }
        let active = self.farm_controller.active() || self.farm_controller.uitest_live;
        let mut open = true;
        let (mut go, mut stop, mut close, mut copy_cmd, mut browse, mut make_key) = (false, false, false, false, false, false);
        let mut command: Option<ControllerCommand> = None;
        let lan = crate::farm::lan_address();
        egui::Window::new(tr!(farm_ctl_title))
            .id(egui::Id::new("fractadyne.farm-controller"))
            .open(&mut open)
            .resizable(false)
            .default_width(620.0)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let max_h = (ctx.screen_rect().height() - 160.0).max(240.0);
                egui::ScrollArea::vertical().max_height(max_h).auto_shrink([false, true]).show(ui, |ui| {
                    ui.set_max_width(600.0);
                    let f = &mut self.farm_controller;
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(format!("{}  ·  {}", f.tour_name, f.script.as_ref().map(|p| p.display().to_string()).unwrap_or_default()))
                                .weak()
                                .small(),
                        )
                        .truncate(),
                    );
                    ui.add_space(4.0);
                    ui.add_enabled_ui(!active, |ui| {
                        egui::Grid::new("farm_ctl_grid").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                            ui.label(tr!(farm_ctl_out));
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(&mut f.out).desired_width(330.0));
                                    if ui.button(tr!(farm_ctl_browse)).clicked() {
                                        browse = true;
                                    }
                                });
                                ui.label(egui::RichText::new(tr!(farm_ctl_out_note)).weak().small());
                            });
                            ui.end_row();

                            ui.label(tr!(farm_ctl_size));
                            ui.horizontal(|ui| {
                                let (w, h) = (f.width, f.height);
                                let cur = crate::STANDARD_SIZES.iter().find(|(_, pw, ph)| *pw == w && *ph == h).map(|(l, _, _)| *l);
                                let custom = f.custom_size || cur.is_none();
                                egui::ComboBox::from_id_salt("farm_ctl_size")
                                    .width(210.0)
                                    .height(460.0)
                                    .selected_text(if custom {
                                        tr!(farm_ctl_size_custom_value, width = w.to_string(), height = h.to_string())
                                    } else {
                                        crate::size_preset_label(cur.unwrap_or_default())
                                    })
                                    .show_ui(ui, |ui| {
                                        if ui.selectable_label(custom, tr!(farm_ctl_size_custom)).clicked() {
                                            f.custom_size = true;
                                        }
                                        ui.separator();
                                        for (label, pw, ph) in crate::STANDARD_SIZES {
                                            if ui.selectable_label(!custom && cur == Some(*label), crate::size_preset_label(label)).clicked() {
                                                (f.width, f.height, f.custom_size) = (*pw, *ph, false);
                                            }
                                        }
                                    });
                                if custom {
                                    ui.add(egui::DragValue::new(&mut f.width).range(16..=16384));
                                    ui.label("×");
                                    ui.add(egui::DragValue::new(&mut f.height).range(16..=16384));
                                }
                            });
                            ui.end_row();

                            ui.label(tr!(farm_ctl_fps));
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut f.fps).range(0.01..=240.0).speed(0.25));
                                ui.label(format!("   {}", tr!(farm_ctl_ss)));
                                egui::ComboBox::from_id_salt("farm_ctl_ss").selected_text(format!("{}×", f.ss)).width(60.0).show_ui(ui, |ui| {
                                    for n in [1u32, 2, 3, 4] {
                                        ui.selectable_value(&mut f.ss, n, format!("{n}×"));
                                    }
                                });
                            });
                            ui.end_row();

                            ui.label(tr!(farm_ctl_port));
                            ui.add(egui::DragValue::new(&mut f.port).range(1024..=65535)).on_hover_text(tr!(farm_ctl_port_hover));
                            ui.end_row();

                            ui.label(tr!(farm_ctl_share));
                            ui.add(egui::TextEdit::singleline(&mut f.share_root).hint_text(tr!(farm_ctl_share_hint)).desired_width(330.0))
                                .on_hover_text(tr!(farm_ctl_share_hover));
                            ui.end_row();

                            ui.label(tr!(farm_ctl_start_with));
                            ui.horizontal(|ui| {
                                let unit = format!(" {}", tr!(farm_ctl_start_clients, count = f.min_clients));
                                ui.add(egui::DragValue::new(&mut f.min_clients).range(1..=64).suffix(unit))
                                    .on_hover_text(tr!(farm_ctl_start_clients_hover));
                                ui.checkbox(&mut f.local, tr!(farm_ctl_local)).on_hover_text(tr!(farm_ctl_local_hover));
                                ui.checkbox(&mut f.sharing, tr!(farm_ctl_sharing)).on_hover_text(tr!(farm_ctl_sharing_hover));
                            });
                            ui.end_row();
                        });
                    });

                    // The key and the address: what a client needs.
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.label(tr!(farm_ctl_key));
                        match &f.key {
                            Some(k) => {
                                let shown = if f.show_key { k.clone() } else { format!("{}…", k.chars().take(9).collect::<String>()) };
                                ui.label(egui::RichText::new(shown).monospace());
                                ui.checkbox(&mut f.show_key, tr!(farm_ctl_show_key));
                                if ui.button(format!("{}  {}", crate::icons::COPY, tr!(farm_ctl_copy_key))).on_hover_text(tr!(farm_ctl_copy_key_hover)).clicked() {
                                    ui.ctx().copy_text(k.clone());
                                }
                            }
                            None => {
                                ui.label(egui::RichText::new(tr!(farm_ctl_no_key)).weak());
                                if ui.button(tr!(farm_ctl_make_key)).on_hover_text(tr!(farm_ctl_make_key_hover)).clicked() {
                                    make_key = true;
                                }
                            }
                        }
                    });
                    let addr = f.status().filter(|_| active).map(|s| s.listen.clone()).or_else(|| lan.map(|ip| format!("{ip}:{}", f.port)));
                    ui.label(
                        egui::RichText::new(match addr {
                            Some(a) => tr!(farm_ctl_steps, address = a),
                            None => tr!(farm_ctl_steps_port, port = f.port.to_string()),
                        })
                        .weak()
                        .small(),
                    );

                    let frames = ((f.total_s * f.fps).round() as i64 + 1).max(1);
                    ui.add_space(4.0);
                    let summary = tr!(
                        farm_ctl_summary,
                        count = frames,
                        seconds = crate::i18n::decimal(f.total_s, 0),
                        width = f.width.to_string(),
                        height = f.height.to_string(),
                        ss = f.ss.to_string()
                    );
                    ui.label(egui::RichText::new(summary).monospace());

                    let st = f.link.as_ref().and_then(|l| l.status.as_ref()).or(f.last.as_ref()).cloned();
                    if let Some(st) = st {
                        ui.add_space(4.0);
                        ui.separator();
                        let (words, tone) = phase_words(&st);
                        let colour = match tone {
                            1 => crate::theme::ok_color(ui.ctx()),
                            -1 => crate::theme::danger_color(ui.ctx()),
                            _ => crate::theme::ui_accent(ui.ctx()),
                        };
                        ui.horizontal(|ui| {
                            if active && matches!(st.phase, ControllerPhase::Waiting | ControllerPhase::Preparing | ControllerPhase::Rendering) {
                                ui.add(egui::Spinner::new().size(12.0));
                            }
                            ui.label(egui::RichText::new(words).strong().color(colour));
                            if !st.detail.is_empty() && !matches!(st.phase, ControllerPhase::Rendering | ControllerPhase::Waiting) {
                                ui.add(egui::Label::new(egui::RichText::new(format!("— {}", detail_words(&st.detail))).small()).truncate());
                            }
                        });
                        if st.frames > 0 && st.phase != ControllerPhase::Waiting {
                            let frac = st.done as f32 / st.frames as f32;
                            let mut text = tr!(farm_ctl_progress, done = st.done, total = st.frames);
                            if st.frames_per_s > 0.0 && active {
                                text += &format!(" · {}", tr!(farm_ctl_progress_rate, rate = crate::i18n::decimal(st.frames_per_s, 2)));
                            }
                            if let (Some(eta), true) = (st.eta_s, active) {
                                text += &format!(" · {}", tr!(farm_ctl_progress_eta, time = duration(eta)));
                            }
                            ui.add(egui::ProgressBar::new(frac.clamp(0.0, 1.0)).desired_height(14.0).text(egui::RichText::new(text).monospace().small()));
                            strip(ui, &st.strip);
                            let mut facts = vec![tr!(farm_ctl_io, received = crate::i18n::decimal(st.kb_in_per_s, 0), sent = crate::i18n::decimal(st.kb_out_per_s, 0))];
                            if let Some(b) = st.free_bytes {
                                facts.push(tr!(farm_ctl_free, space = space(b)));
                            }
                            facts.push(tr!(farm_ctl_elapsed, time = duration(st.elapsed_s)));
                            if st.failed > 0 {
                                facts.push(tr!(farm_ctl_failed, count = st.failed));
                            }
                            let refs = st.refs_fresh + st.refs_cache + st.refs_reused;
                            if refs > 0 || st.orbits_shared > 0 {
                                facts.push(tr!(farm_ctl_refs, built = st.refs_fresh, cached = st.refs_cache, reused = st.refs_reused, shared = st.orbits_shared));
                            }
                            ui.label(egui::RichText::new(facts.join(" · ")).weak().small());
                        }
                        if st.storage_low {
                            ui.label(egui::RichText::new(format!("⚠ {}", tr!(farm_ctl_storage_low))).color(ui.visuals().warn_fg_color));
                        }
                        // GPUs that differ from this machine's: model, graphics API or driver, and
                        // whether their probes differed (the GPU classes).
                        let differing: Vec<&ClientRow> = st.clients.iter().filter(|r| r.gpu_note.is_some()).collect();
                        if !differing.is_empty() || st.gpu_classes > 1 {
                            let warn = ui.visuals().warn_fg_color;
                            let differ = if st.gpu_classes > 1 { tr!(farm_ctl_gpus_differ_classes, count = st.gpu_classes) } else { tr!(farm_ctl_gpus_differ) };
                            ui.label(egui::RichText::new(format!("⚠ {differ}")).color(warn).small());
                            if let Some(g) = &st.this_gpu {
                                ui.label(egui::RichText::new(format!("    {}", tr!(farm_ctl_this_gpu, gpu = g))).weak().small());
                            }
                            for r in differing {
                                let note = r.gpu_note.as_deref().unwrap_or_default();
                                let line = if r.driver.is_empty() {
                                    tr!(farm_ctl_gpu_row, name = &r.name, note = note, adapter = &r.adapter)
                                } else {
                                    tr!(farm_ctl_gpu_row_driver, name = &r.name, note = note, adapter = &r.adapter, driver = &r.driver)
                                };
                                ui.add(egui::Label::new(egui::RichText::new(format!("    {line}")).color(warn).small()).truncate());
                            }
                        }
                        if !st.clients.is_empty() {
                            ui.add_space(4.0);
                            egui::Grid::new("farm_ctl_clients").num_columns(8).striped(true).spacing([10.0, 3.0]).show(ui, |ui| {
                                let heads = [
                                    tr!(farm_ctl_col_machine),
                                    tr!(farm_ctl_col_state),
                                    tr!(farm_ctl_col_gpu),
                                    tr!(farm_ctl_col_frames),
                                    tr!(farm_ctl_col_per_frame),
                                    tr!(farm_ctl_col_link),
                                    tr!(farm_ctl_col_heard),
                                    String::new(),
                                ];
                                for h in heads {
                                    ui.label(egui::RichText::new(h).weak().small());
                                }
                                ui.end_row();
                                for r in &st.clients {
                                    // Fixed widths: a truncating label in a grid shrinks its column
                                    // to nothing ("WOR…", "rend…" — seen in the UI walk).
                                    let strikes = if r.strikes > 0 { format!("\n{}", tr!(farm_ctl_strikes, count = r.strikes)) } else { String::new() };
                                    cell(ui, 150.0, egui::RichText::new(&r.name)).on_hover_text(format!("{}{strikes}", r.addr));
                                    let state_colour = if r.removed { crate::theme::danger_color(ui.ctx()) } else { ui.visuals().text_color() };
                                    let state = state_words(r);
                                    cell(ui, 180.0, egui::RichText::new(&state).color(state_colour)).on_hover_text(state);
                                    let (g, why) = gpu_cell(r);
                                    ui.label(g).on_hover_text(why);
                                    ui.label(crate::i18n::count(r.frames_done));
                                    ui.label(r.ms_per_frame.map_or("—".into(), |ms| format!("{:.1} s", ms / 1000.0)));
                                    ui.label(if r.share { tr!(farm_ctl_link_share) } else { r.link_mbps.map_or("—".into(), |m| format!("{m:.0} Mb/s")) });
                                    ui.label(r.heartbeat_age_s.map_or("—".into(), |a| tr!(farm_ctl_heard_ago, seconds = crate::i18n::decimal(a, 0))));
                                    if active {
                                        if r.removed {
                                            if ui.small_button(tr!(farm_ctl_readmit)).on_hover_text(tr!(farm_ctl_readmit_hover)).clicked() {
                                                command = Some(ControllerCommand::Readmit(r.name.clone()));
                                            }
                                        } else if r.state != "gone"
                                            && !r.state.starts_with("removing")
                                            && ui.small_button(tr!(farm_ctl_remove)).on_hover_text(tr!(farm_ctl_remove_hover)).clicked()
                                        {
                                            command = Some(ControllerCommand::Remove(r.id));
                                        }
                                    } else {
                                        ui.label("");
                                    }
                                    ui.end_row();
                                }
                            });
                        }
                        let log: Vec<String> = match &f.link {
                            Some(l) => l.log.iter().cloned().collect(),
                            None => f.last_log.clone(),
                        };
                        if !log.is_empty() {
                            egui::CollapsingHeader::new(tr!(farm_ctl_events)).id_salt("farm_ctl_events").default_open(true).show(ui, |ui| {
                                egui::ScrollArea::vertical().id_salt("farm_ctl_log").max_height(140.0).stick_to_bottom(true).show(ui, |ui| {
                                    for l in log.iter().rev().take(200).rev() {
                                        let warn = l.starts_with("! ") || l.contains('⚠');
                                        let t = egui::RichText::new(l.trim_start_matches("! ")).monospace().small();
                                        ui.add(egui::Label::new(if warn { t.color(ui.visuals().warn_fg_color) } else { t }).truncate());
                                    }
                                });
                            });
                        }
                        if active && matches!(st.phase, ControllerPhase::Rendering | ControllerPhase::Paused | ControllerPhase::Waiting) {
                            ui.horizontal(|ui| {
                                let paused = st.phase == ControllerPhase::Paused && !st.storage_low;
                                if paused {
                                    if ui.button(format!("{}  {}", crate::icons::PLAY, tr!(farm_ctl_resume))).clicked() {
                                        command = Some(ControllerCommand::Resume);
                                    }
                                } else if ui.button(format!("{}  {}", crate::icons::PAUSE, tr!(farm_ctl_pause))).on_hover_text(tr!(farm_ctl_pause_hover)).clicked() {
                                    command = Some(ControllerCommand::Pause);
                                }
                            });
                        }
                        if active && f.rendering_here() {
                            ui.label(egui::RichText::new(tr!(farm_ctl_live_paused)).weak().small());
                        }
                    }
                    if let Some(e) = &f.error {
                        ui.label(egui::RichText::new(e).color(crate::theme::danger_color(ui.ctx())));
                    }
                });
                ui.separator();
                let f = &self.farm_controller;
                crate::theme::action_row(ui, |ui| {
                    if crate::theme::cancel_button(ui, &tr!(farm_ctl_close))
                        .on_hover_text(if active { tr!(farm_ctl_close_active_hover) } else { tr!(farm_ctl_close_hover) })
                        .clicked()
                    {
                        close = true;
                    }
                    if ui.button(format!("{}  {}", crate::icons::COPY, tr!(farm_ctl_copy_command))).on_hover_text(tr!(farm_ctl_copy_command_hover)).clicked() {
                        copy_cmd = true;
                    }
                    if active {
                        let label = if f.stop_sent.is_some_and(|t| t.elapsed() > Duration::from_secs(10)) { tr!(farm_ctl_stop_now) } else { tr!(farm_ctl_stop) };
                        if ui.button(format!("{}  {label}", crate::icons::STOP)).on_hover_text(tr!(farm_ctl_stop_hover)).clicked() {
                            stop = true;
                        }
                    } else if crate::theme::confirm_button(ui, &tr!(farm_ctl_render)).on_hover_text(tr!(farm_ctl_render_hover)).clicked() {
                        go = true;
                    }
                });
            });
        self.farm_controller.open = open && !close;
        if browse {
            let seed = Some(absolute(&self.farm_controller.out)).filter(|p| p.is_dir()).unwrap_or_else(|| self.dialog_dir_default());
            if let Some(dir) = rfd::FileDialog::new().set_directory(seed).pick_folder() {
                self.remember_dir(&dir);
                self.farm_controller.out = dir.display().to_string();
            }
        }
        if make_key {
            if let Err(e) = crate::farm::load_key(&[], true) {
                self.farm_controller.error = Some(e);
            }
            self.farm_controller.reload_key();
        }
        if copy_cmd {
            ctx.copy_text(format!("fractadyne {}", self.farm_controller.args().iter().map(|a| quote(a)).collect::<Vec<_>>().join(" ")));
        }
        if let Some(c) = command {
            if let Some(l) = self.farm_controller.link.as_mut() {
                l.send(&c.text());
            }
            ctx.request_repaint();
        }
        if stop {
            let f = &mut self.farm_controller;
            match f.stop_sent {
                Some(t) if t.elapsed() > Duration::from_secs(10) => {
                    if let Some(l) = f.link.as_mut() {
                        l.kill();
                    }
                }
                Some(_) => {}
                None => {
                    if let Some(l) = f.link.as_mut() {
                        l.send(&ControllerCommand::Stop.text());
                    }
                    f.stop_sent = Some(Instant::now());
                }
            }
        }
        if go {
            self.start_farm_controller();
        }
    }
}

/// `--uitest`'s seed for this window: harness code, filed under `src/harness/`.
#[path = "../harness/ui/farm_controller/uitest_seed.rs"]
mod uitest_seed;

/// A table cell of a fixed width, its text cut with "…" when longer.
fn cell(ui: &mut egui::Ui, width: f32, text: egui::RichText) -> egui::Response {
    let h = ui.spacing().interact_size.y;
    ui.allocate_ui_with_layout(egui::vec2(width, h), egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.set_min_width(width);
        ui.add(egui::Label::new(text).truncate())
    })
    .inner
}

/// The frame strip: one cell per `Scheduler::strip` character, coloured by state.
fn strip(ui: &mut egui::Ui, cells: &str) {
    if cells.is_empty() {
        return;
    }
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width().min(600.0), 8.0), egui::Sense::hover());
    let p = ui.painter_at(rect);
    let n = cells.chars().count() as f32;
    let (done, active, bad, todo) = (crate::theme::ok_color(ui.ctx()), crate::theme::ui_accent(ui.ctx()), crate::theme::danger_color(ui.ctx()), ui.visuals().widgets.inactive.bg_fill);
    for (i, c) in cells.chars().enumerate() {
        let x0 = rect.left() + rect.width() * i as f32 / n;
        let x1 = rect.left() + rect.width() * (i + 1) as f32 / n;
        let col = match c {
            'd' => done,
            'a' => active,
            'x' => bad,
            _ => todo,
        };
        p.rect_filled(egui::Rect::from_min_max(egui::pos2(x0, rect.top()), egui::pos2(x1.max(x0 + 0.5), rect.bottom())), 0.0, col);
    }
    resp.on_hover_text(tr!(farm_ctl_strip_hover));
}

#[cfg(test)]
#[path = "../../tests/unit/ui/farm_controller/tests.rs"]
mod tests;
