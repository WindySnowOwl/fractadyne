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
        m.detail = tr!(farm_client_rendering_on_cards, rendering = rendering, total = sessions.len());
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
/// `gpu_choice::cards_in_listing`); none when it cannot run. The child logs under `logs/<log>`.
pub(crate) fn list_cards(log: &str) -> Vec<(usize, String)> {
    let Ok(exe) = std::env::current_exe() else { return Vec::new() };
    let mut cmd = std::process::Command::new(exe);
    cmd.arg(crate::gpu_choice::LIST_FLAG).stdin(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    if let Some(d) = crate::diag::logs_dir() {
        cmd.env("FRACTADYNE_LOG_DIR", d.join(log));
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
        "" => tr!(farm_client_gpu_system),
        "all" => cards.map_or_else(|| tr!(farm_client_gpu_every), |c| tr!(farm_client_gpu_every_n, count = c.len())),
        v => cards.and_then(|c| c.iter().find(|(n, _)| n.to_string() == v)).map_or_else(|| tr!(farm_client_gpu_adapter, number = v), |(n, name)| format!("{n} · {name}")),
    };
    egui::ComboBox::from_id_salt("farm_client_gpu")
        .width(220.0)
        .selected_text(text)
        .show_ui(ui, |ui| {
            ui.selectable_value(value, String::new(), tr!(farm_client_gpu_system));
            match cards {
                None => {
                    ui.label(egui::RichText::new(tr!(farm_client_gpu_finding)).weak());
                }
                Some(cs) => {
                    for (n, name) in cs {
                        ui.selectable_value(value, n.to_string(), format!("{n} · {name}"));
                    }
                    if cs.len() >= 2 {
                        ui.selectable_value(value, "all".to_string(), tr!(farm_client_gpu_every_each, count = cs.len()));
                    }
                }
            }
        })
        .response
        .on_hover_text(tr!(farm_client_gpu_hover));
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
        _ => return Err(tr!(farm_client_bad_address)),
    }
    fractadyne_farm::key::FarmKey::from_text(key.trim()).map_err(|e| tr!(farm_client_bad_key, why = e.to_string()))?;
    fractadyne_farm::names::check_display_name(s.name.trim()).map_err(|e| tr!(farm_client_bad_name, why = e.to_string()))?;
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
        ClientPhase::Connecting => (tr!(farm_client_phase_connecting), 0),
        ClientPhase::Checking => (tr!(farm_client_phase_checking), 0),
        ClientPhase::Idle => (tr!(farm_client_phase_idle), 1),
        ClientPhase::Rendering => (tr!(farm_client_phase_rendering), 1),
        ClientPhase::Paused if st.in_use && !st.paused => (tr!(farm_client_phase_in_use), 0),
        ClientPhase::Paused => (tr!(farm_client_phase_paused), 0),
        ClientPhase::Retrying => (st.retry_in_s.map_or_else(|| tr!(farm_client_phase_retrying), |s| tr!(farm_client_phase_retrying_in, seconds = s)), -1),
        ClientPhase::Ended if st.exit_code == Some(0) => (tr!(farm_client_phase_disconnected), 0),
        ClientPhase::Ended => (tr!(farm_client_phase_ended), -1),
    }
}

/// The line under the phase (`ClientStatus::detail`) in the language in use. The client process
/// writes it in English — the same words are its log and its `--render-client` output, and that
/// process never loads a language — so the window translates the lines it knows where it draws
/// them (the words are `farm::client`'s), and shows any other line, such as a reason from the
/// controller or the network, as it came.
fn detail_words(d: &str) -> String {
    let between = |pre: &str, post: &str| d.strip_prefix(pre).and_then(|r| r.strip_suffix(post));
    match d {
        "Disconnected" => return tr!(farm_client_phase_disconnected),
        "The job is over" => return tr!(farm_client_detail_job_over),
        "Paused — the controller has been told" => return tr!(farm_client_detail_paused),
        "Pausing — finishing the frame in progress" => return tr!(farm_client_detail_pausing),
        "Cancelled the frame in progress; paused" => return tr!(farm_client_detail_cancelled),
        _ => {}
    }
    if let Some(a) = between("Connecting to ", "…") {
        return tr!(farm_client_detail_connecting, address = a);
    }
    if let Some(n) = between("Connected to \"", "\" — running the self-check") {
        return tr!(farm_client_detail_checking, controller = n);
    }
    if let Some(n) = between("Connected to \"", "\" — waiting for its user to approve this machine") {
        return tr!(farm_client_detail_approval, controller = n);
    }
    if let Some(n) = between("Connected to \"", "\" — idle") {
        return tr!(farm_client_detail_idle, controller = n);
    }
    if let Some(n) = between("Job \"", "\" — receiving its bundle") {
        return tr!(farm_client_detail_bundle, job = n);
    }
    if let Some(n) = between("Job \"", "\" — waiting for frames to render") {
        return tr!(farm_client_detail_job_waiting, job = n);
    }
    if let Some((a, b)) = d.strip_prefix("Rendering frames ").and_then(|r| r.split_once('–')) {
        return tr!(farm_client_detail_rendering, first = a, last = b);
    }
    if let Some(t) = d.strip_prefix("Waiting — someone is using this machine; work resumes once nobody has for ") {
        return tr!(farm_client_detail_in_use, time = t);
    }
    if let Some(r) = d.strip_prefix("The self-check failed — ") {
        return tr!(farm_client_detail_check_failed, why = r);
    }
    if let Some(r) = d.strip_prefix("Refused by the controller: ") {
        return tr!(farm_client_detail_refused, why = r);
    }
    if let Some(r) = d.strip_prefix("Refused this job: ") {
        return tr!(farm_client_detail_job_refused, why = r);
    }
    if let Some(r) = d.strip_prefix("The controller closed the connection: ") {
        return tr!(farm_client_detail_closed, why = r);
    }
    if let Some(r) = d.strip_prefix("The controller ended the connection: ") {
        return tr!(farm_client_detail_ended, why = r);
    }
    d.to_string()
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
                ui.label(egui::RichText::new(tr!(farm_client_finding)).weak().small());
            });
        }
        match found {
            Some([]) if !finding => {
                ui.label(
                    egui::RichText::new(tr!(farm_client_found_none, port = fractadyne_farm::discovery::DISCOVERY_PORT.to_string()))
                    .weak()
                    .small(),
                );
            }
            Some(list) => {
                if !list.is_empty() {
                    ui.label(egui::RichText::new(tr!(farm_client_found_pick)).weak().small());
                }
                let (ver, git) = crate::farm::build_identity();
                for f in list {
                    let same = f.beacon.app_version == ver && f.beacon.git == git;
                    let text = format!("\"{}\" · {} · {} {}", f.beacon.name, f.address(), f.beacon.app_version, f.beacon.git);
                    let r = ui.add(egui::SelectableLabel::new(f.address() == current, egui::RichText::new(text).small()));
                    let r = r.on_hover_text(tr!(farm_client_found_hover, identity = &f.beacon.identity));
                    if r.clicked() {
                        pick = Some(f.address());
                    }
                    if !same {
                        let warning = tr!(farm_client_found_other_build, version = ver, git = git);
                        ui.label(egui::RichText::new(format!("⚠ {warning}")).color(ui.visuals().warn_fg_color).small());
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
    // Frame numbers go in as text: they name frames (and their files), they are not amounts.
    let mut parts = vec![tr!(farm_client_run_frames, first = a.to_string(), last = (b - 1).to_string())];
    if let Some(f) = st.frame {
        parts.push(tr!(farm_client_run_frame, frame = f.to_string(), index = f.saturating_sub(a) + 1, count = b - a));
        if let Some(ms) = st.frame_ms {
            parts.push(tr!(farm_client_run_frame_time, time = secs(ms as f64)));
        }
    }
    parts.join(" · ")
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
                let _ = tx.send(list_cards("farm-client"));
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
            c.error = Some(tr!(farm_client_no_config));
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
            Err(e) => c.error = Some(tr!(farm_client_start_failed, why = e.to_string())),
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
        egui::Window::new(tr!(farm_client_title))
            .id(egui::Id::new("fractadyne.farm-client"))
            .open(&mut open)
            .resizable(false)
            .default_width(440.0)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let max_h = (ctx.screen_rect().height() - 160.0).max(200.0);
                egui::ScrollArea::vertical().max_height(max_h).auto_shrink([false, true]).show(ui, |ui| {
                    ui.set_max_width(420.0);
                    ui.label(egui::RichText::new(tr!(farm_client_intro)).weak().small());
                    ui.add_space(4.0);
                    let c = &mut self.farm_client;
                    ui.add_enabled_ui(!active, |ui| {
                        egui::Grid::new("farm_client_grid").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                            ui.label(tr!(farm_client_controller));
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut c.settings.controller).hint_text("192.168.1.20:46733").desired_width(220.0));
                                let finding = c.found_rx.is_some();
                                if ui.add_enabled(!finding, egui::Button::new(tr!(farm_client_find))).on_hover_text(tr!(farm_client_find_hover)).clicked() {
                                    c.find();
                                }
                            });
                            ui.end_row();
                            if !active {
                                if let Some(pick) = found_list(ui, c.found_rx.is_some(), c.found.as_deref(), c.settings.controller.trim()) {
                                    c.settings.controller = pick;
                                }
                            }
                            ui.label(tr!(farm_client_key));
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut c.key).password(!c.show_key).hint_text("fdn1-…").desired_width(220.0));
                                ui.checkbox(&mut c.show_key, tr!(farm_client_show_key));
                            });
                            ui.end_row();
                            ui.label(tr!(farm_client_machine));
                            ui.add(egui::TextEdit::singleline(&mut c.settings.name).desired_width(220.0)).on_hover_text(tr!(farm_client_machine_hover));
                            ui.end_row();
                            ui.label(tr!(farm_client_share));
                            let hint = tr!(farm_client_share_hint);
                            let (w, fits) = crate::theme::hinted_width(ui, &hint, 220.0);
                            let field = egui::TextEdit::singleline(&mut c.settings.share_root).desired_width(w);
                            if fits {
                                ui.add(field.hint_text(hint)).on_hover_text(tr!(farm_client_share_hover));
                            } else {
                                ui.add(field).on_hover_text(format!("{hint}\n\n{}", tr!(farm_client_share_hover)));
                            }
                            ui.end_row();
                            ui.label(tr!(farm_client_gpu));
                            gpu_combo(ui, &mut c.settings.adapters, c.cards.as_deref());
                            ui.end_row();
                            if cfg!(windows) {
                                ui.label(tr!(farm_client_when));
                                ui.horizontal(|ui| {
                                    let help = tr!(farm_client_when_idle_hover);
                                    ui.checkbox(&mut c.settings.when_idle, tr!(farm_client_when_idle)).on_hover_text(help.as_str());
                                    ui.add_enabled(c.settings.when_idle, egui::DragValue::new(&mut c.settings.idle_minutes).range(1..=1440).suffix(" min")).on_hover_text(help);
                                });
                                ui.end_row();
                            }
                        });
                        egui::CollapsingHeader::new(tr!(farm_client_limits)).id_salt("farm_client_limits").show(ui, |ui| {
                            ui.label(egui::RichText::new(tr!(farm_client_limits_note)).weak().small());
                            egui::Grid::new("farm_client_limits_grid").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                                ui.label(tr!(farm_client_limit_size));
                                ui.horizontal(|ui| {
                                    ui.add(egui::DragValue::new(&mut c.settings.max_width).range(16..=16384));
                                    ui.label("×");
                                    ui.add(egui::DragValue::new(&mut c.settings.max_height).range(16..=16384));
                                });
                                ui.end_row();
                                ui.label(tr!(farm_client_limit_ss));
                                ui.add(egui::DragValue::new(&mut c.settings.max_ss).range(1..=8).suffix("×"));
                                ui.end_row();
                                ui.label(tr!(farm_client_limit_iter));
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
                            ui.label(egui::RichText::new(detail_words(&st.detail)).small());
                        }
                        egui::Grid::new("farm_client_status").num_columns(2).spacing([8.0, 3.0]).show(ui, |ui| {
                            let row = |ui: &mut egui::Ui, k: &str, v: String| {
                                ui.label(egui::RichText::new(k).weak());
                                ui.add(egui::Label::new(v).truncate());
                                ui.end_row();
                            };
                            if !st.controller_name.is_empty() {
                                row(
                                    ui,
                                    &tr!(farm_client_controller),
                                    tr!(farm_client_row_controller_value, name = &st.controller_name, address = &st.controller, identity = &st.controller_fingerprint),
                                );
                            } else if !st.controller.is_empty() {
                                row(ui, &tr!(farm_client_controller), st.controller.clone());
                            }
                            if !st.identity.is_empty() {
                                row(ui, &tr!(farm_client_machine), tr!(farm_client_row_machine_value, name = &st.name, identity = &st.identity));
                            }
                            if let Some(g) = st.gpu.as_ref().filter(|_| sessions.is_empty()) {
                                row(ui, &tr!(farm_client_row_gpu), g.clone());
                            }
                            if let Some(j) = &st.job {
                                row(ui, &tr!(farm_client_row_job), match &st.job_detail {
                                    Some(d) => format!("{j} — {d}"),
                                    None => j.clone(),
                                });
                            }
                            if let Some((a, b)) = st.run {
                                row(ui, &tr!(farm_client_row_run), run_words(st, a, b));
                            }
                            // One session per graphics card: what each is doing.
                            let no_card = tr!(farm_client_session_no_card);
                            for s in &sessions {
                                // "NVIDIA GeForce RTX 4090 — frame 123 of 120–135"; the rest on hover, the counts on Sent.
                                let card = s.gpu.as_deref().and_then(|g| g.split(" · ").next()).unwrap_or(no_card.as_str());
                                let (what, more) = match s.run {
                                    Some((a, b)) if s.phase == ClientPhase::Rendering => (
                                        tr!(farm_client_session_frame, frame = s.frame.unwrap_or(a).to_string(), first = a.to_string(), last = (b - 1).to_string()),
                                        run_words(s, a, b),
                                    ),
                                    _ => (phase_words(s).0, detail_words(&s.detail)),
                                };
                                ui.label(egui::RichText::new(tr!(farm_client_row_gpu_n, slot = s.slot.unwrap_or(0).to_string())).weak());
                                ui.add(egui::Label::new(format!("{card} — {what}")).truncate()).on_hover_text(format!("{}
{more}", s.gpu.as_deref().unwrap_or(card)));
                                ui.end_row();
                            }
                            if st.frames_done > 0 {
                                // With one session per card, each card's share: "1,234 frames (812 + 422)".
                                let time: String = st.mean_ms.map_or("—".into(), secs);
                                let sent = if sessions.is_empty() {
                                    tr!(farm_client_sent, count = st.frames_done, time = time)
                                } else {
                                    let split = sessions.iter().map(|s| crate::i18n::count(s.frames_done)).collect::<Vec<_>>().join(" + ");
                                    tr!(farm_client_sent_split, count = st.frames_done, split = split, time = time)
                                };
                                row(ui, &tr!(farm_client_row_sent), sent);
                            }
                        });
                        if let Some(n) = &st.gpu_note {
                            ui.label(
                                egui::RichText::new(format!("⚠ {}", tr!(farm_client_gpu_note, note = n)))
                                    .color(ui.visuals().warn_fg_color)
                                    .small(),
                            );
                        }
                        if !st.self_check.is_empty() {
                            egui::CollapsingHeader::new(tr!(farm_client_self_check)).id_salt("farm_client_check").show(ui, |ui| {
                                for l in &st.self_check {
                                    ui.label(egui::RichText::new(l).monospace().small());
                                }
                            });
                        }
                        if let Some((_, tex)) = &c.thumb {
                            let sz = tex.size_vec2();
                            let scale = (240.0 / sz.x.max(1.0)).min(1.0);
                            ui.add(egui::Image::new(tex).fit_to_exact_size(sz * scale)).on_hover_text(tr!(farm_client_thumb_hover));
                        }
                        if active && matches!(st.phase, ClientPhase::Idle | ClientPhase::Rendering | ClientPhase::Paused) {
                            ui.horizontal(|ui| {
                                if st.paused {
                                    if ui.button(format!("{}  {}", crate::icons::PLAY, tr!(farm_client_resume))).on_hover_text(tr!(farm_client_resume_hover)).clicked() {
                                        command = Some(ClientCommand::Resume);
                                    }
                                } else if ui
                                    .button(format!("{}  {}", crate::icons::PAUSE, tr!(farm_client_pause)))
                                    .on_hover_text(tr!(farm_client_pause_hover))
                                    .clicked()
                                {
                                    command = Some(ClientCommand::Pause);
                                }
                                if st.phase == ClientPhase::Rendering
                                    && ui
                                        .button(format!("{}  {}", crate::icons::STOP, tr!(farm_client_cancel_frame)))
                                        .on_hover_text(tr!(farm_client_cancel_frame_hover))
                                        .clicked()
                                {
                                    command = Some(ClientCommand::CancelFrame);
                                }
                            });
                        }
                        if active && st.phase == ClientPhase::Rendering {
                            ui.label(egui::RichText::new(tr!(farm_client_live_paused)).weak().small());
                        }
                    }
                    if let Some(e) = &c.error {
                        ui.label(egui::RichText::new(e).color(crate::theme::danger_color(ui.ctx())));
                    }
                });
                ui.separator();
                crate::theme::action_row(ui, |ui| {
                    if crate::theme::cancel_button(ui, &tr!(farm_client_close))
                        .on_hover_text(if active { tr!(farm_client_close_active_hover) } else { tr!(farm_client_close_hover) })
                        .clicked()
                    {
                        close = true;
                    }
                    if active {
                        if ui.button(tr!(farm_client_disconnect)).on_hover_text(tr!(farm_client_disconnect_hover)).clicked() {
                            disconnect = true;
                        }
                    } else if crate::theme::confirm_button(ui, &tr!(farm_client_connect)).on_hover_text(tr!(farm_client_connect_hover)).clicked() {
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
}

/// `--uitest`'s seed for this window: harness code, filed under `src/harness/`.
#[path = "../harness/ui/farm_client/uitest_seed.rs"]
mod uitest_seed;

#[cfg(test)]
#[path = "../../tests/unit/ui/farm_client/tests.rs"]
mod tests;
