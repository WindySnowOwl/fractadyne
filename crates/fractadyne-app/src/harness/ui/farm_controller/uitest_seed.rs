//! `--uitest`'s seed for the farm controller window: a job mid-render on three machines - the
//! state the walk photographs, without a farm.
//! Attached to `ui::farm_controller` (`#[path]`): it fills the window's private fields. Moved
//! from `ui/farm_controller.rs`.

use super::*;

impl FractadyneApp {
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
        // The same GPU on another driver is another class (`Controller::gpu_class`), probe or not.
        let mut studio = row(2, "STUDIO-PC", "rendering 7310 (9/16)", "B", 0, 2380, 1330.0);
        studio.adapter = "NVIDIA GeForce RTX 3080 · Vulkan".into();
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
            refs_fresh: 41,
            refs_cache: 4890,
            refs_reused: 781,
            orbits_shared: 6,
            kb_in_per_s: 2310.0,
            kb_out_per_s: 14.0,
            eta_s: Some(2290.0),
            free_bytes: Some(412 << 30),
            elapsed_s: 3105.0,
            strip,
            clients: vec![
                row(1, "WORKSTATION (local)", "rendering 5731 (4/16)", "A", 0, 2604, 1210.0),
                studio,
                row(3, "PLUTO", "rendering 9022 (2/12)", "C", 812, 725, 2620.0),
                gone,
            ],
            gpu_classes: 3,
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
