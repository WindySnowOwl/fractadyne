//! `--uitest`'s seeds for the farm client window: a client mid-job, after Find, and on a two-card
//! machine - the states the walk photographs, without a farm.
//! Attached to `ui::farm_client` (`#[path]`): it fills the window's private fields. Moved
//! from `ui/farm_client.rs`.

use super::*;

impl FractadyneApp {
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
