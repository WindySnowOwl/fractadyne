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
        ControllerPhase::Waiting => (format!("Waiting for render clients ({} of {} connected)", st.clients.iter().filter(|c| !c.removed && c.state != "gone").count(), st.min_clients), 0),
        ControllerPhase::Preparing => ("Preparing the job".into(), 0),
        ControllerPhase::Rendering => ("Rendering".into(), 1),
        ControllerPhase::Paused => ("Paused".into(), 0),
        ControllerPhase::Finished if st.exit_code == Some(0) => ("Complete".into(), 1),
        ControllerPhase::Finished if st.exit_code == Some(4) => ("Stopped".into(), 0),
        ControllerPhase::Finished => ("Ended".into(), -1),
    }
}

/// A client row's GPU column: its class letter, and why.
fn gpu_cell(r: &ClientRow) -> (String, String) {
    let letter = r.gpu_class.clone().unwrap_or_else(|| "—".into());
    let why = match r.probe_px {
        Some(0) => "Its probe render is identical to this machine's.".to_string(),
        Some(px) => format!("Its probe render differs from this machine's in {} pixels: its frames will differ slightly from this machine's.", crate::grouped_count(px as f64)),
        None => "Its probe render has not been compared.".to_string(),
    };
    let adapter = if r.adapter.is_empty() { String::new() } else { format!("\n{}", r.adapter) };
    let driver = if r.driver.is_empty() { String::new() } else { format!("\ndriver {}", r.driver) };
    let note = r.gpu_note.as_ref().map_or(String::new(), |n| format!("\n⚠ {n} from this machine's"));
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
                self.farm_controller.error = log.iter().rev().find(|x| x.starts_with("! ")).map(|x| x.trim_start_matches("! ").to_string()).or(Some(format!("The controller ended (exit {exit:?}).")));
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
            f.error = Some("No tour is loaded.".into());
            return;
        }
        if let Err(e) = std::fs::create_dir_all(absolute(&f.out)) {
            f.error = Some(format!("The output folder {}: {e}", f.out));
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
            Err(e) => f.error = Some(format!("Could not start the farm: {e}")),
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
        egui::Window::new("Render on farm")
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
                            ui.label("Output folder");
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(&mut f.out).desired_width(330.0));
                                    if ui.button("Browse…").clicked() {
                                        browse = true;
                                    }
                                });
                                ui.label(egui::RichText::new("Frames, and the job's state in farm/: run the same job again to resume it.").weak().small());
                            });
                            ui.end_row();

                            ui.label("Size");
                            ui.horizontal(|ui| {
                                let (w, h) = (f.width, f.height);
                                let cur = crate::STANDARD_SIZES.iter().find(|(_, pw, ph)| *pw == w && *ph == h).map(|(l, _, _)| *l);
                                let custom = f.custom_size || cur.is_none();
                                egui::ComboBox::from_id_salt("farm_ctl_size")
                                    .width(210.0)
                                    .height(460.0)
                                    .selected_text(if custom { format!("Custom — {w}×{h}") } else { cur.unwrap_or_default().to_string() })
                                    .show_ui(ui, |ui| {
                                        if ui.selectable_label(custom, "Custom…").clicked() {
                                            f.custom_size = true;
                                        }
                                        ui.separator();
                                        for (label, pw, ph) in crate::STANDARD_SIZES {
                                            if ui.selectable_label(!custom && cur == Some(*label), *label).clicked() {
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

                            ui.label("Frames / second");
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut f.fps).range(0.01..=240.0).speed(0.25));
                                ui.label("   Supersampling");
                                egui::ComboBox::from_id_salt("farm_ctl_ss").selected_text(format!("{}×", f.ss)).width(60.0).show_ui(ui, |ui| {
                                    for n in [1u32, 2, 3, 4] {
                                        ui.selectable_value(&mut f.ss, n, format!("{n}×"));
                                    }
                                });
                            });
                            ui.end_row();

                            ui.label("Listen on port");
                            ui.add(egui::DragValue::new(&mut f.port).range(1024..=65535)).on_hover_text("Clients connect to this port. Allow it through the firewall for your local network.");
                            ui.end_row();

                            ui.label("Shared drive");
                            ui.add(egui::TextEdit::singleline(&mut f.share_root).hint_text("optional — this machine's path to it").desired_width(330.0)).on_hover_text(
                                "Share mode: clients that have the same drive write their frames to it instead of sending them over the connection, and this machine checks every one there. Clients without it send theirs.",
                            );
                            ui.end_row();

                            ui.label("Start with");
                            ui.horizontal(|ui| {
                                let unit = if f.min_clients == 1 { " client" } else { " clients" };
                                ui.add(egui::DragValue::new(&mut f.min_clients).range(1..=64).suffix(unit))
                                    .on_hover_text("The job starts once this many machines have connected and passed their self-check; others can join later.");
                                ui.checkbox(&mut f.local, "Also render on this machine").on_hover_text("This machine counts as one client.");
                            });
                            ui.end_row();
                        });
                    });

                    // The key and the address: what a client needs.
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.label("Farm key");
                        match &f.key {
                            Some(k) => {
                                let shown = if f.show_key { k.clone() } else { format!("{}…", k.chars().take(9).collect::<String>()) };
                                ui.label(egui::RichText::new(shown).monospace());
                                ui.checkbox(&mut f.show_key, "Show");
                                if ui.button(format!("{}  Copy key", crate::icons::COPY)).on_hover_text("Paste it into each client's Render client window").clicked() {
                                    ui.ctx().copy_text(k.clone());
                                }
                            }
                            None => {
                                ui.label(egui::RichText::new("none yet").weak());
                                if ui.button("Make a key").on_hover_text("Clients need it to join; it is also made when you press Render").clicked() {
                                    make_key = true;
                                }
                            }
                        }
                    });
                    let addr = f.status().filter(|_| active).map(|s| s.listen.clone()).or_else(|| lan.map(|ip| format!("{ip}:{}", f.port)));
                    ui.label(
                        egui::RichText::new(match addr {
                            Some(a) => format!("On each client: File ▸ Render client…, controller {a}, and the farm key."),
                            None => format!("On each client: File ▸ Render client…, this machine's address and port {}, and the farm key.", f.port),
                        })
                        .weak()
                        .small(),
                    );

                    let frames = ((f.total_s * f.fps).round() as i64 + 1).max(1);
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(format!("{} frames · {:.0}s of tour · {}×{} ss{}", crate::grouped_count(frames as f64), f.total_s, f.width, f.height, f.ss)).monospace());

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
                                ui.add(egui::Label::new(egui::RichText::new(format!("— {}", st.detail)).small()).truncate());
                            }
                        });
                        if st.frames > 0 && st.phase != ControllerPhase::Waiting {
                            let frac = st.done as f32 / st.frames as f32;
                            let mut text = format!("{} / {} frames", crate::grouped_count(st.done as f64), crate::grouped_count(st.frames as f64));
                            if st.frames_per_s > 0.0 && active {
                                text += &format!(" · {:.2} frames/s", st.frames_per_s);
                            }
                            if let (Some(eta), true) = (st.eta_s, active) {
                                text += &format!(" · about {} left", duration(eta));
                            }
                            ui.add(egui::ProgressBar::new(frac.clamp(0.0, 1.0)).desired_height(14.0).text(egui::RichText::new(text).monospace().small()));
                            strip(ui, &st.strip);
                            let mut facts = vec![format!("in {:.0} KB/s · out {:.0} KB/s", st.kb_in_per_s, st.kb_out_per_s)];
                            if let Some(b) = st.free_bytes {
                                facts.push(format!("{} free", space(b)));
                            }
                            facts.push(format!("{} elapsed", duration(st.elapsed_s)));
                            if st.failed > 0 {
                                facts.push(format!("{} not rendered", st.failed));
                            }
                            ui.label(egui::RichText::new(facts.join(" · ")).weak().small());
                        }
                        if st.storage_low {
                            ui.label(egui::RichText::new("⚠ The output folder is nearly full: no new work is handed out until there is room.").color(ui.visuals().warn_fg_color));
                        }
                        // GPUs that differ from this machine's: model, graphics API or driver, and
                        // whether their probes differed (the GPU classes).
                        let differing: Vec<&ClientRow> = st.clients.iter().filter(|r| r.gpu_note.is_some()).collect();
                        if !differing.is_empty() || st.gpu_classes > 1 {
                            let warn = ui.visuals().warn_fg_color;
                            ui.label(
                                egui::RichText::new(format!(
                                    "⚠ GPUs differ in this farm{}. Their frames differ slightly, which can show in a held shot.",
                                    if st.gpu_classes > 1 { format!(" ({} GPU classes: their probe frames differ)", st.gpu_classes) } else { String::new() }
                                ))
                                .color(warn)
                                .small(),
                            );
                            if let Some(g) = &st.this_gpu {
                                ui.label(egui::RichText::new(format!("    This machine: {g}")).weak().small());
                            }
                            for r in differing {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(format!(
                                            "    {}: {} — {}{}",
                                            r.name,
                                            r.gpu_note.as_deref().unwrap_or_default(),
                                            r.adapter,
                                            if r.driver.is_empty() { String::new() } else { format!(", driver {}", r.driver) }
                                        ))
                                        .color(warn)
                                        .small(),
                                    )
                                    .truncate(),
                                );
                            }
                        }
                        if !st.clients.is_empty() {
                            ui.add_space(4.0);
                            egui::Grid::new("farm_ctl_clients").num_columns(8).striped(true).spacing([10.0, 3.0]).show(ui, |ui| {
                                for h in ["Machine", "State", "GPU", "Frames", "Per frame", "Link", "Heard", ""] {
                                    ui.label(egui::RichText::new(h).weak().small());
                                }
                                ui.end_row();
                                for r in &st.clients {
                                    // Fixed widths: a truncating label in a grid shrinks its column
                                    // to nothing ("WOR…", "rend…" — seen in the UI walk).
                                    cell(ui, 150.0, egui::RichText::new(&r.name)).on_hover_text(format!("{}{}", r.addr, if r.strikes > 0 { format!("\n{} frame(s) failed verification", r.strikes) } else { String::new() }));
                                    let state_colour = if r.removed { crate::theme::danger_color(ui.ctx()) } else { ui.visuals().text_color() };
                                    cell(ui, 180.0, egui::RichText::new(&r.state).color(state_colour)).on_hover_text(&r.state);
                                    let (g, why) = gpu_cell(r);
                                    ui.label(g).on_hover_text(why);
                                    ui.label(crate::grouped_count(r.frames_done as f64));
                                    ui.label(r.ms_per_frame.map_or("—".into(), |ms| format!("{:.1} s", ms / 1000.0)));
                                    ui.label(if r.share { "shared drive".to_string() } else { r.link_mbps.map_or("—".into(), |m| format!("{m:.0} Mb/s")) });
                                    ui.label(r.heartbeat_age_s.map_or("—".into(), |a| format!("{a:.0} s ago")));
                                    if active {
                                        if r.removed {
                                            if ui.small_button("Re-admit").on_hover_text("Let this machine join again when it reconnects").clicked() {
                                                command = Some(ControllerCommand::Readmit(r.name.clone()));
                                            }
                                        } else if r.state != "gone" && !r.state.starts_with("removing") && ui.small_button("Remove").on_hover_text("Take this machine out of the job; its frames go to the others").clicked() {
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
                            egui::CollapsingHeader::new("Events").id_salt("farm_ctl_events").default_open(true).show(ui, |ui| {
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
                                    if ui.button(format!("{}  Resume", crate::icons::PLAY)).clicked() {
                                        command = Some(ControllerCommand::Resume);
                                    }
                                } else if ui.button(format!("{}  Pause", crate::icons::PAUSE)).on_hover_text("Hand out no new work; frames in progress finish").clicked() {
                                    command = Some(ControllerCommand::Pause);
                                }
                            });
                        }
                        if active && f.rendering_here() {
                            ui.label(egui::RichText::new("Live view paused while this machine renders for the farm.").weak().small());
                        }
                    }
                    if let Some(e) = &f.error {
                        ui.label(egui::RichText::new(e).color(crate::theme::danger_color(ui.ctx())));
                    }
                });
                ui.separator();
                let f = &self.farm_controller;
                crate::theme::action_row(ui, |ui| {
                    if crate::theme::cancel_button(ui, "Close")
                        .on_hover_text(if active { "Close the window; the job keeps running (reopen it from the Tools menu)" } else { "Close the window" })
                        .clicked()
                    {
                        close = true;
                    }
                    if ui.button(format!("{}  Copy command", crate::icons::COPY)).on_hover_text("Copy the equivalent command line").clicked() {
                        copy_cmd = true;
                    }
                    if active {
                        let label = if f.stop_sent.is_some_and(|t| t.elapsed() > Duration::from_secs(10)) { "Stop now" } else { "Stop" };
                        if ui.button(format!("{}  {label}", crate::icons::STOP)).on_hover_text("End the job with the frames it has; running the same job again resumes it").clicked() {
                            stop = true;
                        }
                    } else if crate::theme::confirm_button(ui, "Render").on_hover_text("Start listening for clients and render the tour on them").clicked() {
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

    /// The UI walk's view of this window: a job mid-render with three machines, one of them of
    /// another GPU class and one removed, without starting anything.
    pub(crate) fn uitest_seed_farm_controller(&mut self) {
        let f = &mut self.farm_controller;
        f.script = Some(PathBuf::from("tours/grand-tour.toml"));
        f.tour_name = "Grand tour".into();
        f.total_s = 331.0;
        f.out = "C:/Users/robin/Videos/grand-tour-farm".into();
        (f.width, f.height, f.fps, f.ss) = (1920, 1080, 30.0, 2);
        f.key = Some("fdn1-abcde-fghij-klmno-pqrst-uvwxy-z2345-67abc-defgh-ijklm-nopqr-stuvw".into());
        f.error = None;
        f.uitest_live = true;
        let strip: String = (0..400).map(|i| if i < 230 { 'd' } else if i % 37 == 0 { 'a' } else if i == 260 { 'x' } else { '.' }).collect();
        let row = |id: u32, name: &str, state: &str, class: &str, px: u64, frames: u64, ms: f64| ClientRow {
            id,
            name: name.into(),
            addr: format!("192.168.1.{}:5{id}012", 20 + id),
            state: state.into(),
            adapter: if class == "A" { "NVIDIA GeForce RTX 3080 · Vulkan".into() } else { "AMD Radeon RX 6800 XT · Vulkan".into() },
            driver: if class == "A" { "NVIDIA 581.42".into() } else { "AMD proprietary driver 25.9.1".into() },
            gpu_note: (class != "A").then(|| "a different GPU".to_string()),
            link_mbps: Some(940.0),
            share: false,
            frames_done: frames,
            ms_per_frame: Some(ms),
            strikes: 0,
            heartbeat_age_s: Some(1.0),
            kb_in: 0,
            probe_px: Some(px),
            gpu_class: Some(class.into()),
            removed: false,
        };
        let mut studio = row(2, "STUDIO-PC", "rendering 7310 (9/16)", "A", 0, 2380, 1330.0);
        studio.driver = "NVIDIA 576.02".into();
        studio.gpu_note = Some("the same GPU on a different driver".into());
        studio.share = true;
        let mut gone = row(4, "OLD-LAPTOP", "removed: 2 bad frames", "A", 0, 3, 9100.0);
        gone.removed = true;
        gone.strikes = 2;
        f.last = Some(ControllerStatus {
            phase: ControllerPhase::Rendering,
            detail: "Rendering".into(),
            name: "WORKSTATION".into(),
            listen: "192.168.1.20:46733".into(),
            this_gpu: Some("NVIDIA GeForce RTX 3080 · Vulkan, driver NVIDIA 581.42".into()),
            port: 46733,
            min_clients: 2,
            frames: 9931,
            done: 5712,
            assigned: 48,
            pending: 4170,
            failed: 1,
            frames_per_s: 1.84,
            kb_in_per_s: 2310.0,
            kb_out_per_s: 14.0,
            eta_s: Some(2290.0),
            free_bytes: Some(412 << 30),
            elapsed_s: 3105.0,
            strip,
            clients: vec![
                row(1, "WORKSTATION (local)", "rendering 5731 (4/16)", "A", 0, 2604, 1210.0),
                studio,
                row(3, "PLUTO", "rendering 9022 (2/12)", "B", 812, 725, 2620.0),
                gone,
            ],
            gpu_classes: 2,
            ..Default::default()
        });
        f.last_log = vec![
            "PLUTO (192.168.1.23:50089, b7a8-683c-a818-791f) connected — checking".into(),
            "PLUTO: probe differs from this machine's in 812 of 36864 pixels — another GPU class; its frames will differ slightly from this machine's (design §9)".into(),
            "PLUTO admitted (link 906 Mb/s)".into(),
            "⚠⚠ REMOVED OLD-LAPTOP: 2 frames failed verification — its diagnostics are being collected".into(),
            "  5712/9931 frames · 1.84 frames/s · in 2310 KB/s · out 14 KB/s · 3 machine(s) · about 38m10s left".into(),
        ];
        f.open = true;
    }
}

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
    resp.on_hover_text("Every frame of the job, left to right: done, being rendered, still to do, given up.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_phase_has_words_and_a_stop_says_so() {
        for p in [ControllerPhase::Waiting, ControllerPhase::Preparing, ControllerPhase::Rendering, ControllerPhase::Paused, ControllerPhase::Finished] {
            assert!(!phase_words(&ControllerStatus { phase: p, ..Default::default() }).0.is_empty());
        }
        let stopped = ControllerStatus { phase: ControllerPhase::Finished, exit_code: Some(4), ..Default::default() };
        assert_eq!(phase_words(&stopped).0, "Stopped");
        let done = ControllerStatus { phase: ControllerPhase::Finished, exit_code: Some(0), ..Default::default() };
        assert_eq!(phase_words(&done), ("Complete".to_string(), 1));
    }

    #[test]
    fn times_and_space_read_as_a_person_would_say_them() {
        assert_eq!(duration(42.4), "42 s");
        assert_eq!(duration(2290.0), "38m 10s");
        assert_eq!(duration(7500.0), "2h 05m");
        assert_eq!(duration(-3.0), "0 s");
        assert_eq!(space(412 << 30), "442 GB");
        assert_eq!(space(51_700_000_000), "51.7 GB");
        assert_eq!(space(830_000_000), "830 MB");
    }

    #[test]
    fn the_copied_command_is_the_one_that_runs() {
        let f = FarmControllerUi { script: Some(PathBuf::from("C:/tours/grand tour.toml")), out: "C:/frames".into(), min_clients: 3, local: true, ..Default::default() };
        let a = f.args();
        assert_eq!(a[0], "--farm-render");
        assert!(a.windows(2).any(|w| w == ["--listen", "0.0.0.0:46733"]));
        assert!(a.windows(2).any(|w| w == ["--min-clients", "3"]));
        assert_eq!(a.last().map(String::as_str), Some("--local"));
        assert!(!a.iter().any(|x| x == FLAG), "the copied command is for a terminal: no --ui-status");
        assert_eq!(quote("C:/tours/grand tour.toml"), "\"C:/tours/grand tour.toml\"");
    }
}
