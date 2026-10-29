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
        f.zoom = None;
        f.open = true;
    }

    /// The UI walk's view of a zoom video on the farm (`design/zoom-video.md` §8.5): a 1080p dive
    /// at 2x, its strips mid-render on two machines — or, `assembling`, every strip in and the video
    /// being made here.
    pub(crate) fn uitest_seed_farm_controller_zoom(&mut self, assembling: bool) {
        self.uitest_seed_farm_controller();
        let f = &mut self.farm_controller;
        f.zoom = Some(ZoomFarmJob {
            args: vec!["--render-zoom-video".into(), "C:/Users/robin/Videos/spiral-dive.mp4".into()],
            summary: "7,246 frames · 121s of video · 132 strips of 13842×1529".into(),
            video: "C:/Users/robin/Videos/spiral-dive.mp4".into(),
        });
        f.tour_name = "spiral-dive".into();
        let strip: String = (0..132).map(|i| if assembling || i < 61 { 'd' } else if i == 61 || i == 98 { 'a' } else { '.' }).collect();
        if let Some(st) = f.last.as_mut() {
            st.strips = true;
            st.frames = 132;
            st.done = if assembling { 132 } else { 61 };
            st.assigned = if assembling { 0 } else { 2 };
            st.pending = if assembling { 0 } else { 69 };
            st.failed = 0;
            st.frames_per_s = 1.0 / 38.0;
            st.eta_s = (!assembling).then_some(2640.0);
            st.strip = strip;
            st.refs_fresh = 2;
            st.refs_cache = 3;
            st.refs_reused = if assembling { 127 } else { 56 };
            st.orbits_shared = 1;
            st.gpu_classes = 2;
            st.clients.retain(|r| r.id == 1 || r.id == 3);
            for r in st.clients.iter_mut() {
                (r.frames_done, r.ms_per_frame) = if r.id == 1 { (40, Some(31_200.0)) } else { (21, Some(58_900.0)) };
                r.state = if assembling { "idle".into() } else if r.id == 1 { "rendering 61 (1/4)".into() } else { "rendering 98 (2/3)".into() };
            }
            if assembling {
                st.phase = ControllerPhase::Assembling;
                st.detail = "Every strip is in: assembling the video on this machine".into();
                st.assembling = Some("frame 2460/7246  (2m51s elapsed, 5m33s left, 14.4 fps, 2^41.03)".into());
            }
        }
        f.last_log = vec![
            "Zoom video \"spiral-dive\": 132 strips of 13842×1529 → C:/Users/robin/Videos/spiral-dive-strips, assembled into C:/Users/robin/Videos/spiral-dive.mp4 once every strip is in".into(),
            "PLUTO admitted (link 906 Mb/s)".into(),
            "PLUTO shared a reference orbit: 179040 iterations at 240 bits (2.9 MB)".into(),
        ];
        if assembling {
            f.last_log.push("every strip is in — assembling C:/Users/robin/Videos/spiral-dive.mp4 on this machine".into());
        }
    }
}
