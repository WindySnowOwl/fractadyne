//! File ▸ Render client…: lend this machine's GPU to a render farm (design/remote-rendering.md §10).
//!
//! The window starts `--render-client … --ui-status` as a child process and reads its status
//! lines (`farm::status`), as the Render tour window does with its render: the network and the
//! frame renders stay out of this process, and a lost GPU takes down a farm frame, never the app.
//! Closing the window leaves the client running (reopen it from the File menu); quitting the app
//! ends stdin, and the client leaves the farm.

use crate::farm::status::{ClientCommand, ClientPhase, ClientStatus, Link, FLAG};
use crate::FractadyneApp;
use serde::{Deserialize, Serialize};

/// What the window remembers between sessions, in `<config>/farm/client.toml` — never the key,
/// which lives in its own file (`client-key.txt`) the way the controller's does.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub(crate) struct ClientSettings {
    pub(crate) controller: String,
    pub(crate) name: String,
    pub(crate) max_width: u32,
    pub(crate) max_height: u32,
    pub(crate) max_ss: u32,
    pub(crate) max_iter: u32,
    /// This machine's path to the shared drive (share mode), or "" to stream every frame.
    pub(crate) share_root: String,
    /// The graphics card(s) to render on: "" for the system's choice, "all", or `--list-adapters`
    /// numbers ("2", "1,3").
    pub(crate) adapters: String,
    /// Take work only once nobody has used this machine for `idle_minutes` (Windows).
    pub(crate) when_idle: bool,
    pub(crate) idle_minutes: u32,
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            controller: String::new(),
            name: crate::farm::machine_name(&[]),
            max_width: 16384,
            max_height: 16384,
            max_ss: 8,
            max_iter: 10_000_000,
            share_root: String::new(),
            adapters: String::new(),
            when_idle: false,
            idle_minutes: 5,
        }
    }
}

#[derive(Default)]
pub(crate) struct FarmClientUi {
    pub(crate) open: bool,
    loaded: bool,
    pub(crate) settings: ClientSettings,
    /// The farm key as typed (shown masked).
    pub(crate) key: String,
    show_key: bool,
    pub(crate) link: Option<Link<ClientStatus>>,
    /// The last status of a client that has ended — or, for the UI walk, a sample.
    pub(crate) last: Option<ClientStatus>,
    pub(crate) error: Option<String>,
    thumb: Option<(u64, egui::TextureHandle)>,
    /// The UI walk's seeded window draws as a connected one (Disconnect, Pause) with no process
    /// behind it. Drawing only: nothing else — the live view, the menu — takes it for a client.
    pub(crate) uitest_live: bool,
    /// This machine's graphics cards (number, name), read by `--list-adapters` in a child process
    /// when the window first opens; `None` until it answers.
    cards: Option<Vec<(usize, String)>>,
    cards_rx: Option<std::sync::mpsc::Receiver<Vec<(usize, String)>>>,
    /// The last statuses of an ended client that had several sessions (one per graphics card).
    last_all: Vec<ClientStatus>,
    /// The UI walk's sample of a client with one session per graphics card.
    pub(crate) uitest_sessions: Vec<ClientStatus>,
    /// Controllers that answered the last Find; `None` before one ran.
    found: Option<Vec<fractadyne_farm::discovery::Found>>,
    found_rx: Option<std::sync::mpsc::Receiver<Vec<fractadyne_farm::discovery::Found>>>,
}

impl FarmClientUi {
    /// The client is running (connected, connecting or retrying).
    pub(crate) fn active(&self) -> bool {
        self.link.as_ref().is_some_and(|l| l.running())
    }

    /// Rendering farm frames on this machine right now (on any of its cards): the live view
    /// stands aside.
    pub(crate) fn rendering(&self) -> bool {
        self.active() && self.link.as_ref().is_some_and(|l| l.all.values().any(|s| s.phase == ClientPhase::Rendering))
    }

    /// What the window shows: the status — one session's, or several merged — and, when the
    /// client runs one session per graphics card, each session's.
    fn view(&self) -> (Option<ClientStatus>, Vec<ClientStatus>) {
        let sessions: Vec<ClientStatus> = match &self.link {
            Some(l) => l.all.values().cloned().collect(),
            None if !self.uitest_sessions.is_empty() => self.uitest_sessions.clone(),
            None => self.last_all.clone(),
        };
        if sessions.len() > 1 {
            return (merged(&sessions), sessions);
        }
        (self.link.as_ref().and_then(|l| l.status.clone()).or_else(|| self.last.clone()), Vec::new())
    }
}

/// One status for a client with several sessions: the busiest session's phase and words, the
/// frames all of them sent, the thumbnail of the session that sent the most, and the machine's
/// own name (without " · GPU k").
pub(crate) fn merged(sessions: &[ClientStatus]) -> Option<ClientStatus> {
    let rank = |p: ClientPhase| match p {
        ClientPhase::Rendering => 6,
        ClientPhase::Checking => 5,
        ClientPhase::Connecting => 4,
        ClientPhase::Idle => 3,
        ClientPhase::Paused => 2,
        ClientPhase::Retrying => 1,
        ClientPhase::Ended => 0,
    };
    let top = sessions.iter().max_by_key(|s| rank(s.phase))?;
    let mut m = top.clone();
    m.name = top.name.rsplit_once(" · GPU ").map_or_else(|| top.name.clone(), |(n, _)| n.to_string());
    m.frames_done = sessions.iter().map(|s| s.frames_done).sum();
    let ms: f64 = sessions.iter().filter_map(|s| s.mean_ms.map(|x| x * s.frames_done as f64)).sum();
    m.mean_ms = (m.frames_done > 0).then(|| ms / m.frames_done as f64);
    m.last_frame = sessions.iter().filter(|s| s.last_frame.is_some()).max_by_key(|s| s.last_frame_seq).and_then(|s| s.last_frame.clone());
    m.last_frame_seq = sessions.iter().map(|s| s.last_frame_seq).sum();
    m.in_use = sessions.iter().any(|s| s.in_use);
    // The machine's words, not one card's run.
    let rendering = sessions.iter().filter(|s| s.phase == ClientPhase::Rendering).count();
    if rendering > 0 {
        m.detail = format!("Rendering on {rendering} of {} graphics cards", sessions.len());
    }
    if sessions.iter().all(|s| s.phase == ClientPhase::Ended) {
        m.exit_code = sessions.iter().filter_map(|s| s.exit_code).find(|&c| c != 0).or(Some(0));
    }
    (m.run, m.frame, m.frame_ms, m.gpu, m.slot) = (None, None, None, None, None);
    Some(m)
}

impl FarmClientUi {
    /// Ask the network which controllers listen, off the UI thread.
    fn find(&mut self) {
        if self.found_rx.is_some() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        self.found_rx = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send(fractadyne_farm::discovery::discover(std::time::Duration::from_millis(1200)).unwrap_or_default());
        });
    }
}

/// The graphics cards `--list-adapters` names, read in a child process (see
/// `gpu_choice::cards_in_listing`); none when it cannot run.
fn list_cards() -> Vec<(usize, String)> {
    let Ok(exe) = std::env::current_exe() else { return Vec::new() };
    let mut cmd = std::process::Command::new(exe);
    cmd.arg(crate::gpu_choice::LIST_FLAG).stdin(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    if let Some(d) = crate::diag::logs_dir() {
        cmd.env("FRACTADYNE_LOG_DIR", d.join("farm-client"));
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    cmd.output().map(|o| crate::gpu_choice::cards_in_listing(&String::from_utf8_lossy(&o.stdout))).unwrap_or_default()
}

/// The graphics-card choice: the system's, one card, or every card (one session each).
fn gpu_combo(ui: &mut egui::Ui, value: &mut String, cards: Option<&[(usize, String)]>) {
    let current = value.trim().to_string();
    let text = match current.as_str() {
        "" => "The system's choice".to_string(),
        "all" => format!("Every graphics card{}", cards.map_or(String::new(), |c| format!(" ({})", c.len()))),
        v => cards.and_then(|c| c.iter().find(|(n, _)| n.to_string() == v)).map_or_else(|| format!("Adapter {v}"), |(n, name)| format!("{n} · {name}")),
    };
    egui::ComboBox::from_id_salt("farm_client_gpu")
        .width(220.0)
        .selected_text(text)
        .show_ui(ui, |ui| {
            ui.selectable_value(value, String::new(), "The system's choice");
            match cards {
                None => {
                    ui.label(egui::RichText::new("Finding this machine's graphics cards…").weak());
                }
                Some(cs) => {
                    for (n, name) in cs {
                        ui.selectable_value(value, n.to_string(), format!("{n} · {name}"));
                    }
                    if cs.len() >= 2 {
                        ui.selectable_value(value, "all".to_string(), format!("Every graphics card ({}) — one session each", cs.len()));
                    }
                }
            }
        })
        .response
        .on_hover_text("Which graphics card renders the farm's frames. With every card, each joins the farm as its own machine — \"NAME · GPU 1\", \"NAME · GPU 2\" — and Pause and Disconnect act on all of them.");
}

fn settings_path() -> Option<std::path::PathBuf> {
    crate::farm::farm_dir().ok().map(|d| d.join("client.toml"))
}

fn key_path() -> Option<std::path::PathBuf> {
    crate::farm::farm_dir().ok().map(|d| d.join("client-key.txt"))
}

/// Check what was typed before starting anything: `Err` says what to fix.
pub(crate) fn validate(s: &ClientSettings, key: &str) -> Result<(), String> {
    let c = s.controller.trim();
    match c.rsplit_once(':') {
        Some((host, port)) if !host.is_empty() && port.parse::<u16>().is_ok_and(|p| p > 0) => {}
        _ => return Err("Enter the controller's address as HOST:PORT, as its window shows it (for example 192.168.1.20:46733).".into()),
    }
    fractadyne_farm::key::FarmKey::from_text(key.trim()).map_err(|e| format!("The farm key: {e}"))?;
    fractadyne_farm::names::check_display_name(s.name.trim()).map_err(|e| format!("The name: {e}"))?;
    Ok(())
}

/// The command line the window starts its client with (`--farmtest` starts one the same way, so
/// a flag this gets wrong fails the harness).
pub(crate) fn client_args(s: &ClientSettings, key_file: &std::path::Path) -> Vec<String> {
    let mut a = vec![
        "--render-client".into(),
        s.controller.clone(),
        "--farm-key-file".into(),
        key_file.to_string_lossy().into_owned(),
        "--name".into(),
        s.name.clone(),
        "--max-size".into(),
        format!("{}x{}", s.max_width, s.max_height),
        "--max-ss".into(),
        s.max_ss.to_string(),
        "--max-iter".into(),
        s.max_iter.to_string(),
        FLAG.into(),
    ];
    if !s.share_root.trim().is_empty() {
        a.push("--share-root".into());
        a.push(s.share_root.trim().to_string());
    }
    if !s.adapters.trim().is_empty() {
        a.push("--adapters".into());
        a.push(s.adapters.trim().to_string());
    }
    if s.when_idle && cfg!(windows) {
        a.push("--when-idle".into());
        a.push(s.idle_minutes.max(1).to_string());
    }
    a
}

/// The phase in words, and whether it is a good (green), working (accent) or bad (red) state.
fn phase_words(st: &ClientStatus) -> (String, i8) {
    match st.phase {
        ClientPhase::Connecting => ("Connecting…".into(), 0),
        ClientPhase::Checking => ("Running the self-check…".into(), 0),
        ClientPhase::Idle => ("Connected — waiting for work".into(), 1),
        ClientPhase::Rendering => ("Rendering".into(), 1),
        ClientPhase::Paused if st.in_use && !st.paused => ("Waiting — someone is using this machine".into(), 0),
        ClientPhase::Paused => ("Paused".into(), 0),
        ClientPhase::Retrying => (format!("Controller unreachable — retrying{}", st.retry_in_s.map_or(String::new(), |s| format!(" in {s} s"))), -1),
        ClientPhase::Ended if st.exit_code == Some(0) => ("Disconnected".into(), 0),
        ClientPhase::Ended => ("Ended".into(), -1),
    }
}

/// Under the Controller field: what Find found, one line each to pick (the farm key is still
/// needed). Returns the address picked. Takes the two grid columns: a blank label, then the list.
fn found_list(ui: &mut egui::Ui, finding: bool, found: Option<&[fractadyne_farm::discovery::Found]>, current: &str) -> Option<String> {
    if !finding && found.is_none() {
        return None;
    }
    let mut pick = None;
    ui.label("");
    ui.vertical(|ui| {
        if finding {
            ui.horizontal(|ui| {
                ui.add(egui::Spinner::new().size(10.0));
                ui.label(egui::RichText::new("Looking for controllers on this network…").weak().small());
            });
        }
        match found {
            Some([]) if !finding => {
                ui.label(
                    egui::RichText::new(format!(
                        "No controller answered. One answers while its farm is listening; its firewall must let UDP {} in.",
                        fractadyne_farm::discovery::DISCOVERY_PORT
                    ))
                    .weak()
                    .small(),
                );
            }
            Some(list) => {
                if !list.is_empty() {
                    ui.label(egui::RichText::new("On this network — click one to use it:").weak().small());
                }
                let (ver, git) = crate::farm::build_identity();
                for f in list {
                    let same = f.beacon.app_version == ver && f.beacon.git == git;
                    let text = format!("\"{}\" · {} · {} {}", f.beacon.name, f.address(), f.beacon.app_version, f.beacon.git);
                    let r = ui.add(egui::SelectableLabel::new(f.address() == current, egui::RichText::new(text).small()));
                    let r = r.on_hover_text(format!("Identity {} — use this controller (its farm key is still needed)", f.beacon.identity));
                    if r.clicked() {
                        pick = Some(f.address());
                    }
                    if !same {
                        ui.label(egui::RichText::new(format!("⚠ A different build from this one ({ver} {git}): it would refuse this machine.")).color(ui.visuals().warn_fg_color).small());
                    }
                }
            }
            None => {}
        }
    });
    ui.end_row();
    pick
}

/// "frames 120–135 · frame 123 (4 of 16) · 2.1 s on this frame".
fn run_words(st: &ClientStatus, a: u64, b: u64) -> String {
    let at = st.frame.map_or(String::new(), |f| {
        format!(" · frame {f} ({} of {}){}", f.saturating_sub(a) + 1, b - a, st.frame_ms.map_or(String::new(), |ms| format!(" · {} on this frame", secs(ms as f64))))
    });
    format!("frames {a}–{}{at}", b - 1)
}

fn secs(ms: f64) -> String {
    if ms < 10_000.0 {
        format!("{:.1} s", ms / 1000.0)
    } else {
        FractadyneApp::fmt_export_duration(std::time::Duration::from_millis(ms as u64))
    }
}

impl FractadyneApp {
    pub(crate) fn open_farm_client(&mut self) {
        let c = &mut self.farm_client;
        if !c.loaded {
            c.loaded = true;
            if let Some(s) = settings_path().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|t| toml::from_str::<ClientSettings>(&t).ok()) {
                c.settings = s;
            }
            if let Some(k) = key_path().and_then(|p| std::fs::read_to_string(p).ok()) {
                c.key = k.trim().to_string();
            }
        }
        // No controller typed yet: look for one on the network.
        if c.settings.controller.trim().is_empty() && !c.active() {
            c.find();
        }
        // The cards, once, off the UI thread (a child process enumerates them).
        if c.cards.is_none() && c.cards_rx.is_none() {
            let (tx, rx) = std::sync::mpsc::channel();
            c.cards_rx = Some(rx);
            std::thread::spawn(move || {
                let _ = tx.send(list_cards());
            });
        }
        c.open = true;
    }

    /// Read the client's status; reap it when it ends. Every frame, so the File menu and the live
    /// view know about a client even with the window closed.
    pub(crate) fn poll_farm_client(&mut self, ctx: &egui::Context) {
        let Some(l) = self.farm_client.link.as_mut() else { return };
        if l.poll() {
            ctx.request_repaint();
        }
        if !l.running() {
            let st = l.status.clone();
            self.farm_client.last_all = if l.all.len() > 1 { l.all.values().cloned().collect() } else { Vec::new() };
            let exit = l.exit.flatten();
            let tail: Vec<String> = l.log.iter().rev().filter(|x| x.starts_with("! ")).take(1).cloned().collect();
            self.farm_client.link = None;
            self.farm_client.last = st.or_else(|| Some(ClientStatus { phase: ClientPhase::Ended, exit_code: exit, ..Default::default() }));
            if exit != Some(0) && self.farm_client.error.is_none() {
                self.farm_client.error = tail.first().map(|t| t.trim_start_matches("! ").to_string());
            }
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
    }

    fn connect_farm_client(&mut self) {
        let c = &mut self.farm_client;
        c.error = None;
        c.settings.controller = c.settings.controller.trim().to_string();
        c.settings.name = c.settings.name.trim().to_string();
        if let Err(e) = validate(&c.settings, &c.key) {
            c.error = Some(e);
            return;
        }
        // The key goes to a file the client reads (never onto a command line).
        let (Some(kp), Some(sp)) = (key_path(), settings_path()) else {
            c.error = Some("There is no configuration folder to keep the farm key in.".into());
            return;
        };
        let write = |p: &std::path::Path, text: &str| -> Result<(), String> {
            if let Some(d) = p.parent() {
                std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
            }
            let part = fractadyne_export::partial_path(p);
            std::fs::write(&part, text).and_then(|()| std::fs::rename(&part, p)).map_err(|e| format!("{}: {e}", p.display()))
        };
        if let Err(e) = write(&kp, &format!("{}\n", c.key.trim())).and_then(|()| write(&sp, &toml::to_string_pretty(&c.settings).unwrap_or_default())) {
            c.error = Some(e);
            return;
        }
        let s = &c.settings;
        let args = client_args(s, &kp);
        // Its own log folder: it must not write the session's frame record (`frames.bin`).
        let mut env = Vec::new();
        if let Some(d) = crate::diag::logs_dir() {
            env.push(("FRACTADYNE_LOG_DIR", d.join("farm-client").into_os_string()));
        }
        match Link::spawn(&args, &env) {
            Ok(l) => {
                crate::diag::breadcrumb(format!("render client → {}", s.controller));
                c.link = Some(l);
                c.last = None;
                c.thumb = None;
            }
            Err(e) => c.error = Some(format!("Could not start the render client: {e}")),
        }
    }

    pub(crate) fn draw_farm_client_window(&mut self, ctx: &egui::Context) {
        if !self.farm_client.open {
            return;
        }
        let active = self.farm_client.active() || self.farm_client.uitest_live;
        let mut open = true;
        let (mut connect, mut disconnect, mut close) = (false, false, false);
        let mut command: Option<ClientCommand> = None;
        if let Some(rx) = &self.farm_client.cards_rx {
            match rx.try_recv() {
                Ok(v) => {
                    self.farm_client.cards = Some(v);
                    self.farm_client.cards_rx = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => ctx.request_repaint_after(std::time::Duration::from_millis(200)),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => self.farm_client.cards_rx = None,
            }
        }
        if let Some(rx) = &self.farm_client.found_rx {
            match rx.try_recv() {
                Ok(v) => {
                    self.farm_client.found = Some(v);
                    self.farm_client.found_rx = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => ctx.request_repaint_after(std::time::Duration::from_millis(200)),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => self.farm_client.found_rx = None,
            }
        }
        let (view, sessions) = self.farm_client.view();
        // The thumbnail of the last frame sent, re-read when it changes.
        if let Some(st) = &view {
            let seq = st.last_frame_seq;
            if seq > 0 && self.farm_client.thumb.as_ref().is_none_or(|t| t.0 != seq) {
                if let Some(Ok((w, h, rgba))) = st.last_frame.as_deref().map(|p| fractadyne_export::read_thumbnail(std::path::Path::new(p), 240)) {
                    let img = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
                    self.farm_client.thumb = Some((seq, ctx.load_texture("farm-client-last-frame", img, egui::TextureOptions::LINEAR)));
                }
            }
        }
        egui::Window::new("Render client")
            .open(&mut open)
            .resizable(false)
            .default_width(440.0)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let max_h = (ctx.screen_rect().height() - 160.0).max(200.0);
                egui::ScrollArea::vertical().max_height(max_h).auto_shrink([false, true]).show(ui, |ui| {
                    ui.set_max_width(420.0);
                    ui.label(
                        egui::RichText::new(
                            "Render frames for a render farm. The controller — Tools ▸ Render on farm… on another \
                             machine — shows its address and farm key. This machine opens no port: it connects to the controller.",
                        )
                        .weak()
                        .small(),
                    );
                    ui.add_space(4.0);
                    let c = &mut self.farm_client;
                    ui.add_enabled_ui(!active, |ui| {
                        egui::Grid::new("farm_client_grid").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                            ui.label("Controller");
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut c.settings.controller).hint_text("192.168.1.20:46733").desired_width(220.0));
                                let finding = c.found_rx.is_some();
                                if ui.add_enabled(!finding, egui::Button::new("Find")).on_hover_text("Look for controllers on this network (a controller answers while its farm is listening)").clicked() {
                                    c.find();
                                }
                            });
                            ui.end_row();
                            if !active {
                                if let Some(pick) = found_list(ui, c.found_rx.is_some(), c.found.as_deref(), c.settings.controller.trim()) {
                                    c.settings.controller = pick;
                                }
                            }
                            ui.label("Farm key");
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut c.key).password(!c.show_key).hint_text("fdn1-…").desired_width(220.0));
                                ui.checkbox(&mut c.show_key, "Show");
                            });
                            ui.end_row();
                            ui.label("This machine");
                            ui.add(egui::TextEdit::singleline(&mut c.settings.name).desired_width(220.0)).on_hover_text("The name the controller shows for this machine.");
                            ui.end_row();
                            ui.label("Shared drive");
                            ui.add(egui::TextEdit::singleline(&mut c.settings.share_root).hint_text("optional — e.g. \\\\fileserver\\share").desired_width(220.0)).on_hover_text(
                                "This machine's path to the shared drive the controller uses (share mode): frames are written there instead of sent over the connection. Leave empty to send every frame.",
                            );
                            ui.end_row();
                            ui.label("Graphics card");
                            gpu_combo(ui, &mut c.settings.adapters, c.cards.as_deref());
                            ui.end_row();
                            if cfg!(windows) {
                                ui.label("When");
                                ui.horizontal(|ui| {
                                    let help = "Take work only once nobody has touched this machine's keyboard or mouse for this long, or while its screen is locked. When someone starts using it, the frame in progress stops at once and goes back to the farm.";
                                    ui.checkbox(&mut c.settings.when_idle, "Only when idle for").on_hover_text(help);
                                    ui.add_enabled(c.settings.when_idle, egui::DragValue::new(&mut c.settings.idle_minutes).range(1..=1440).suffix(" min")).on_hover_text(help);
                                });
                                ui.end_row();
                            }
                        });
                        egui::CollapsingHeader::new("Limits").id_salt("farm_client_limits").show(ui, |ui| {
                            ui.label(egui::RichText::new("A job asking for more than these is refused, never cut down.").weak().small());
                            egui::Grid::new("farm_client_limits_grid").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                                ui.label("Largest frame");
                                ui.horizontal(|ui| {
                                    ui.add(egui::DragValue::new(&mut c.settings.max_width).range(16..=16384));
                                    ui.label("×");
                                    ui.add(egui::DragValue::new(&mut c.settings.max_height).range(16..=16384));
                                });
                                ui.end_row();
                                ui.label("Most supersampling");
                                ui.add(egui::DragValue::new(&mut c.settings.max_ss).range(1..=8).suffix("×"));
                                ui.end_row();
                                ui.label("Most iterations");
                                ui.add(
                                    egui::DragValue::new(&mut c.settings.max_iter)
                                        .range(1..=u32::MAX)
                                        .custom_formatter(|v, _| crate::grouped_count(v))
                                        .custom_parser(crate::parse_grouped_number),
                                );
                                ui.end_row();
                            });
                        });
                    });

                    if let Some(st) = &view {
                        ui.add_space(4.0);
                        ui.separator();
                        let (words, tone) = phase_words(st);
                        let colour = match tone {
                            1 => crate::theme::ok_color(ui.ctx()),
                            -1 => crate::theme::danger_color(ui.ctx()),
                            _ => crate::theme::ui_accent(ui.ctx()),
                        };
                        ui.horizontal(|ui| {
                            if active && matches!(st.phase, ClientPhase::Connecting | ClientPhase::Checking | ClientPhase::Rendering) {
                                ui.add(egui::Spinner::new().size(12.0));
                            }
                            ui.label(egui::RichText::new(words).strong().color(colour));
                        });
                        if !st.detail.is_empty() {
                            ui.label(egui::RichText::new(&st.detail).small());
                        }
                        egui::Grid::new("farm_client_status").num_columns(2).spacing([8.0, 3.0]).show(ui, |ui| {
                            let row = |ui: &mut egui::Ui, k: &str, v: String| {
                                ui.label(egui::RichText::new(k).weak());
                                ui.add(egui::Label::new(v).truncate());
                                ui.end_row();
                            };
                            if !st.controller_name.is_empty() {
                                row(ui, "Controller", format!("\"{}\" · {} · identity {}", st.controller_name, st.controller, st.controller_fingerprint));
                            } else if !st.controller.is_empty() {
                                row(ui, "Controller", st.controller.clone());
                            }
                            if !st.identity.is_empty() {
                                row(ui, "This machine", format!("\"{}\" · identity {}", st.name, st.identity));
                            }
                            if let Some(g) = st.gpu.as_ref().filter(|_| sessions.is_empty()) {
                                row(ui, "GPU", g.clone());
                            }
                            if let Some(j) = &st.job {
                                row(ui, "Job", match &st.job_detail {
                                    Some(d) => format!("{j} — {d}"),
                                    None => j.clone(),
                                });
                            }
                            if let Some((a, b)) = st.run {
                                row(ui, "Run", run_words(st, a, b));
                            }
                            // One session per graphics card: what each is doing.
                            for s in &sessions {
                                // "NVIDIA GeForce RTX 4090 — frame 123 of 120–135"; the rest on hover, the counts on Sent.
                                let card = s.gpu.as_deref().and_then(|g| g.split(" · ").next()).unwrap_or("graphics card not known yet");
                                let (what, more) = match s.run {
                                    Some((a, b)) if s.phase == ClientPhase::Rendering => (format!("frame {} of {a}–{}", s.frame.unwrap_or(a), b - 1), run_words(s, a, b)),
                                    _ => (phase_words(s).0, s.detail.clone()),
                                };
                                ui.label(egui::RichText::new(format!("GPU {}", s.slot.unwrap_or(0))).weak());
                                ui.add(egui::Label::new(format!("{card} — {what}")).truncate()).on_hover_text(format!("{}
{more}", s.gpu.as_deref().unwrap_or(card)));
                                ui.end_row();
                            }
                            if st.frames_done > 0 {
                                // With one session per card, each card's share: "1,234 frames (812 + 422)".
                                let split = if sessions.is_empty() { String::new() } else { format!(" ({})", sessions.iter().map(|s| crate::grouped_count(s.frames_done as f64)).collect::<Vec<_>>().join(" + ")) };
                                row(ui, "Sent", format!("{} frame{}{split} · {} a frame", crate::grouped_count(st.frames_done as f64), if st.frames_done == 1 { "" } else { "s" }, st.mean_ms.map_or("—".into(), secs)));
                            }
                        });
                        if let Some(n) = &st.gpu_note {
                            ui.label(
                                egui::RichText::new(format!("⚠ This machine has {n}. Its frames will differ slightly from the controller's, which can show in a held shot."))
                                    .color(ui.visuals().warn_fg_color)
                                    .small(),
                            );
                        }
                        if !st.self_check.is_empty() {
                            egui::CollapsingHeader::new("Self-check").id_salt("farm_client_check").show(ui, |ui| {
                                for l in &st.self_check {
                                    ui.label(egui::RichText::new(l).monospace().small());
                                }
                            });
                        }
                        if let Some((_, tex)) = &c.thumb {
                            let sz = tex.size_vec2();
                            let scale = (240.0 / sz.x.max(1.0)).min(1.0);
                            ui.add(egui::Image::new(tex).fit_to_exact_size(sz * scale)).on_hover_text("The last frame this machine sent");
                        }
                        if active && matches!(st.phase, ClientPhase::Idle | ClientPhase::Rendering | ClientPhase::Paused) {
                            ui.horizontal(|ui| {
                                if st.paused {
                                    if ui.button(format!("{}  Resume", crate::icons::PLAY)).on_hover_text("Take work again").clicked() {
                                        command = Some(ClientCommand::Resume);
                                    }
                                } else if ui
                                    .button(format!("{}  Pause", crate::icons::PAUSE))
                                    .on_hover_text("Finish the frame in progress, then take no more work. The controller hands this machine's frames to the others.")
                                    .clicked()
                                {
                                    command = Some(ClientCommand::Pause);
                                }
                                if st.phase == ClientPhase::Rendering
                                    && ui
                                        .button(format!("{}  Cancel frame", crate::icons::STOP))
                                        .on_hover_text("Stop the frame in progress now and pause; the controller renders it elsewhere.")
                                        .clicked()
                                {
                                    command = Some(ClientCommand::CancelFrame);
                                }
                            });
                        }
                        if active && st.phase == ClientPhase::Rendering {
                            ui.label(egui::RichText::new("Live view paused while this machine renders farm frames.").weak().small());
                        }
                    }
                    if let Some(e) = &c.error {
                        ui.label(egui::RichText::new(e).color(crate::theme::danger_color(ui.ctx())));
                    }
                });
                ui.separator();
                crate::theme::action_row(ui, |ui| {
                    if crate::theme::cancel_button(ui, "Close")
                        .on_hover_text(if active { "Close the window; this machine stays in the farm (reopen it from the File menu)" } else { "Close the window" })
                        .clicked()
                    {
                        close = true;
                    }
                    if active {
                        if ui.button("Disconnect").on_hover_text("Leave the farm: the frame in progress is handed back").clicked() {
                            disconnect = true;
                        }
                    } else if crate::theme::confirm_button(ui, "Connect").on_hover_text("Connect to the controller and render frames for it").clicked() {
                        connect = true;
                    }
                });
            });
        self.farm_client.open = open && !close;
        if disconnect {
            command = Some(ClientCommand::Leave);
        }
        if let Some(cmd) = command {
            if let Some(l) = self.farm_client.link.as_mut() {
                l.send(cmd.text());
            }
            ctx.request_repaint();
        }
        if connect {
            self.connect_farm_client();
        }
    }

    /// The UI walk's view of this window: a client mid-job, without starting one.
    pub(crate) fn uitest_seed_farm_client(&mut self) {
        let c = &mut self.farm_client;
        c.loaded = true;
        c.settings.controller = "192.168.1.20:46733".into();
        c.settings.name = "STUDIO-PC".into();
        c.key = "fdn1-sample-sample-sample-sample-sample-sample-sample-sampl".into();
        c.error = None;
        c.uitest_live = true;
        c.cards = Some(vec![(1, "AMD Radeon RX 6800 XT".into())]);
        c.settings.adapters = String::new();
        c.settings.when_idle = true;
        c.settings.idle_minutes = 10;
        c.uitest_sessions.clear();
        c.last = Some(ClientStatus {
            phase: ClientPhase::Rendering,
            detail: "Rendering frames 120–135".into(),
            controller: "192.168.1.20:46733".into(),
            controller_name: "WORKSTATION".into(),
            controller_fingerprint: "39a8-eba6-e9ff-5883".into(),
            identity: "6940-a5ca-d697-eba5".into(),
            name: "STUDIO-PC".into(),
            job: Some("Grand tour".into()),
            job_detail: Some("1920×1080 ss2 at 30 fps, 9,931 frames".into()),
            run: Some((120, 136)),
            frame: Some(123),
            frame_ms: Some(2140),
            frames_done: 1234,
            mean_ms: Some(2380.0),
            gpu: Some("AMD Radeon RX 6800 XT · Vulkan, driver AMD proprietary driver 25.9.1".into()),
            controller_gpu: Some("NVIDIA GeForce RTX 3080 · Vulkan, driver NVIDIA 581.42".into()),
            gpu_note: Some("a different GPU from the controller's — this machine: AMD Radeon RX 6800 XT · Vulkan, driver AMD proprietary driver 25.9.1; the controller: NVIDIA GeForce RTX 3080 · Vulkan, driver NVIDIA 581.42".into()),
            self_check: vec!["ok   render: test frame in 707 ms on AMD Radeon RX 6800 XT · Vulkan".into(), "ok   storage: 51.7 GB free for frames in progress".into()],
            ..Default::default()
        });
        c.open = true;
    }

    /// The UI walk's view of the window before connecting, after Find answered: two controllers,
    /// one of another build.
    pub(crate) fn uitest_seed_farm_client_find(&mut self) {
        let c = &mut self.farm_client;
        c.loaded = true;
        c.settings = ClientSettings { name: "STUDIO-PC".into(), ..Default::default() };
        c.key = String::new();
        c.error = None;
        c.uitest_live = false;
        c.uitest_sessions.clear();
        c.last = None;
        c.cards = Some(vec![(1, "AMD Radeon RX 6800 XT".into())]);
        let (ver, git) = crate::farm::build_identity();
        let at = |ip: [u8; 4], name: &str, ver: &str, git: &str, id: &str| fractadyne_farm::discovery::Found {
            from: std::net::SocketAddr::from((ip, fractadyne_farm::discovery::DISCOVERY_PORT)),
            beacon: fractadyne_farm::discovery::Beacon { name: name.into(), port: 46733, app_version: ver.into(), git: git.into(), identity: id.into() },
        };
        c.found = Some(vec![at([192, 168, 1, 20], "WORKSTATION", ver, git, "39a8-eba6-e9ff-5883"), at([192, 168, 1, 31], "OLD-LAPTOP", "0.3.0-beta.17", "g09f924b", "71c2-0f3e-aa10-4b9d")]);
        c.open = true;
    }

    /// The UI walk's view of a two-card machine running one session per card.
    pub(crate) fn uitest_seed_farm_client_gpus(&mut self) {
        self.uitest_seed_farm_client();
        let c = &mut self.farm_client;
        c.cards = Some(vec![(1, "NVIDIA GeForce RTX 4090".into()), (2, "NVIDIA GeForce RTX 3080".into())]);
        c.settings.adapters = "all".into();
        let base = c.last.clone().unwrap_or_default();
        let session = |k: u32, gpu: &str, run: (u64, u64), frame: u64, done: u64| ClientStatus {
            slot: Some(k),
            name: format!("STUDIO-PC · GPU {k}"),
            gpu: Some(format!("{gpu} · Vulkan, driver NVIDIA 581.42")),
            gpu_note: None,
            run: Some(run),
            frame: Some(frame),
            frame_ms: Some(1900 + 400 * u64::from(k)),
            frames_done: done,
            detail: format!("Rendering frames {}–{}", run.0, run.1 - 1),
            ..base.clone()
        };
        c.uitest_sessions = vec![session(1, "NVIDIA GeForce RTX 4090", (120, 136), 123, 812), session(2, "NVIDIA GeForce RTX 3080", (200, 216), 207, 422)];
        c.last = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_is_typed_is_checked_before_anything_starts() {
        let key = fractadyne_farm::key::FarmKey::generate().expect("a key").to_text();
        let ok = ClientSettings { controller: "192.168.1.20:46733".into(), name: "PLUTO".into(), ..Default::default() };
        assert_eq!(validate(&ok, &key), Ok(()));
        assert_eq!(validate(&ClientSettings { controller: "studio-pc:46733".into(), ..ok.clone() }, &key), Ok(()), "a host name");
        for bad in ["", "192.168.1.20", "192.168.1.20:", ":46733", "192.168.1.20:0", "192.168.1.20:99999"] {
            assert!(validate(&ClientSettings { controller: bad.into(), ..ok.clone() }, &key).is_err(), "accepted controller {bad:?}");
        }
        assert!(validate(&ok, "fdn1-not-a-key").is_err());
        assert!(validate(&ClientSettings { name: "".into(), ..ok.clone() }, &key).is_err());
    }

    #[test]
    fn the_window_s_choices_reach_the_command_line() {
        let key = std::path::Path::new("k.txt");
        let plain = client_args(&ClientSettings::default(), key);
        assert!(!plain.iter().any(|a| a == "--adapters" || a == "--when-idle"), "{plain:?}");
        let s = ClientSettings { adapters: "all".into(), when_idle: true, idle_minutes: 10, ..Default::default() };
        let a = client_args(&s, key);
        let after = |f: &str| a.iter().position(|x| x == f).and_then(|i| a.get(i + 1)).cloned();
        assert_eq!(after("--adapters").as_deref(), Some("all"));
        assert_eq!(after("--when-idle").as_deref(), if cfg!(windows) { Some("10") } else { None });
    }

    #[test]
    fn several_sessions_read_as_one_machine() {
        let s = |k: u32, phase, done, mean| ClientStatus { slot: Some(k), name: format!("PLUTO · GPU {k}"), phase, frames_done: done, mean_ms: mean, last_frame_seq: done, last_frame: Some(format!("gpu{k}.png")), ..Default::default() };
        let m = merged(&[s(1, ClientPhase::Idle, 3, Some(1000.0)), s(2, ClientPhase::Rendering, 1, Some(2000.0))]).expect("merged");
        assert_eq!(m.phase, ClientPhase::Rendering, "the busiest session's phase");
        assert_eq!(m.detail, "Rendering on 1 of 2 graphics cards", "the machine's words, not one card's run");
        assert_eq!(m.name, "PLUTO");
        assert_eq!(m.frames_done, 4);
        assert_eq!(m.mean_ms, Some(1250.0), "weighted by frames");
        assert_eq!(m.last_frame.as_deref(), Some("gpu1.png"), "the thumbnail of the session that sent the most");
        assert_eq!(m.slot, None);
        let ended = |k, code| ClientStatus { slot: Some(k), phase: ClientPhase::Ended, exit_code: Some(code), ..Default::default() };
        assert_eq!(merged(&[ended(1, 0), ended(2, 3)]).and_then(|m| m.exit_code), Some(3), "one session refused: the client did not end well");
        assert!(merged(&[]).is_none());
    }

    #[test]
    fn every_phase_has_words() {
        for p in [ClientPhase::Connecting, ClientPhase::Checking, ClientPhase::Idle, ClientPhase::Rendering, ClientPhase::Paused, ClientPhase::Retrying, ClientPhase::Ended] {
            assert!(!phase_words(&ClientStatus { phase: p, ..Default::default() }).0.is_empty());
        }
        assert!(phase_words(&ClientStatus { phase: ClientPhase::Retrying, retry_in_s: Some(8), ..Default::default() }).0.contains("in 8 s"));
    }
}
