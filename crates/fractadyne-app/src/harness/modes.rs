//! The harness modes the command line turns on: their state (parsed in `FractadyneApp::new_state`),
//! the per-frame hooks `update()` runs for the in-loop harnesses, and the branches of the mode
//! ladder that run a suite and exit. Moved from `cli.rs`.

use super::profile;

/// `--selftest` mode state, parsed from the EXPANDED argv (see `selftest_filter`'s
/// old note: raw `std::env::args()` would bypass `@response-file` expansion).
pub(crate) struct SelftestCli {
    /// CLI `--selftest`: run the GPU validation suite, print a report, and exit.
    pub(crate) run: bool,
    pub(crate) done: bool,
    /// `--selftest-filter <substr>`, `--selftest-list`, `--bless` — parsed here from the
    /// EXPANDED args so they honor `@response-file` / `--args-file` expansion. `run_selftest`
    /// must read these, NOT `std::env::args()` (raw args bypass the expansion `main()` did).
    pub(crate) filter: Option<String>,
    pub(crate) list: bool,
    pub(crate) bless: bool,
    /// `--bless-new-adapter`: let `--bless` on a card other than the one
    /// `validation/golden/BLESSED-GPU.txt` names overwrite the REFERENCE set, moving the release
    /// gate's reference adapter (W8). Without it, such a bless writes that card's own set.
    pub(crate) bless_new_adapter: bool,
}

/// `--profile` mode state (the profiling harness: benchmark regions -> logs/).
pub(crate) struct ProfileCli {
    /// CLI `--profile`: run the profiling harness (benchmark regions), log to `logs/`, exit.
    pub(crate) run: bool,
    pub(crate) done: bool,
    pub(crate) reps: u32,
    pub(crate) regions: Option<String>,
    pub(crate) out: Option<std::path::PathBuf>,
}

/// `--frametest` mode state (frame-timing / stutter harness dive shape).
pub(crate) struct FrametestCli {
    /// CLI `--frametest`: run the frame-timing / stutter harness (deep-zoom dive), log, exit.
    pub(crate) run: bool,
    /// `--frametest --center X Y` (full-precision decimals; default seahorse).
    pub(crate) center: Option<(String, String)>,
    pub(crate) steps: u32,
    pub(crate) hold: u32,
    pub(crate) dive: f64,
}

/// One-shot dev-harness modes: each field is a CLI-requested mode that drives the app
/// from `update()`'s ladder (`cli_mode_ladder` / `harness_frame_hooks`) and exits or
/// finishes on its own.
pub(crate) struct HarnessModes {
    /// CLI `--uitest [DIR]`: scripted walk through every UI screen + the live-render bands,
    /// screenshotting each and writing a review bundle (see `mod uitest`), then exit.
    pub(crate) uitest: Option<crate::harness::uitest::UiTest>,
    /// `--uitest` scratch for the control-panel toggle step: the canvas and panel widths measured
    /// while the panel was still open, so the reflow after hiding it can be compared against what
    /// the layout actually did rather than against the panel's nominal width.
    pub(crate) uitest_central_w: Option<f32>,
    pub(crate) uitest_panel_w: Option<f32>,
    /// The shader key the `formula-apply-async` step applied, so its record can check that the
    /// view switched to exactly that formula.
    pub(crate) uitest_async_key: Option<u64>,
    /// Pointer events the `life-draw` step feeds the app, one a frame, at CELLS of the Life view
    /// (turned into positions as they are fed, so they land on the cell whatever the layout).
    pub(crate) uitest_pointer: std::collections::VecDeque<crate::UiPointer>,
    pub(crate) soak: Option<crate::harness::soak::Soak>,
    /// `--recordtest [FRAMES]`: the frame record's own gate — does it record every frame, fill
    /// every required field, and survive an abort? See `mod recordtest`.
    pub(crate) recordtest: Option<crate::harness::recordtest::RecordTest>,
    /// `--shot`: regenerate the published screenshot from a saved location, then exit.
    pub(crate) shot: Option<crate::harness::shot::Shot>,
    /// CLI `--juliadive [DIR]`: dev harness — dual view, continuous in-app Julia zoom to ~1400×
    /// with periodic screenshots (see `uitest::JuliaDive`). Reproduces the dual-view Julia motion
    /// path deterministically (synthetic OS input proved unreliable for wheel/focus).
    pub(crate) juliadive: Option<crate::harness::uitest::JuliaDive>,
    /// CLI `--dualsettle [DIR]`: dev harness — dual view, live-hover c motion then idle, timed
    /// settled screenshots (the 2026-09-04 blurry-Julia field report).
    pub(crate) dualsettle: Option<crate::harness::uitest::DualSettle>,
    /// CLI `--autodive [LOG10]`: unpaced autopilot dive that hammers the frame-cost controller and
    /// reports whether the lethal regime was reached. See `autopilot::AutoDive`.
    pub(crate) autodive: Option<crate::autopilot::AutoDive>,
    /// CLI `--chunk-sweep [ITERS]`: measure the UNCENSORED per-window wall cost of a chunked
    /// iterate at the current view. The one measurement that separates the two branches of the
    /// 2026-08-22 device loss — see `mod chunksweep`.
    pub(crate) chunk_sweep: Option<crate::harness::chunksweep::ChunkSweep>,
    /// CLI `--deviceloss-repro [ZOOM_LOG2] [REF_ITER]`: a SAFE reproduction of the deep-interior
    /// device loss — measures the honest floor-window submission cost at small areas and extrapolates
    /// to full resolution, predicting the lethal dispatch without running it. See `mod deviceloss_repro`.
    pub(crate) deviceloss_repro: Option<crate::harness::deviceloss_repro::DeviceLossRepro>,
    /// CLI `--motiontest`: the motion-presentation gate (design/mode2-chunking.md §11) — the
    /// in-loop harness that can see what `--livetest`'s settled checkpoints cannot: what a
    /// chunked view ADOPTS as its frozen texture while the camera is moving.
    pub(crate) motiontest: Option<crate::harness::motiontest::MotionTest>,
    /// CLI `--zoomtest [OCTAVES]`: on-screen update-latency harness for live zooms — the real
    /// window, a virtual Space key through the production glide, every presented frame's wall
    /// interval + what it showed, JSON + summary, exit. See `mod zoomtest`.
    pub(crate) zoomtest: Option<crate::harness::zoomtest::ZoomTest>,
    /// CLI `--divetest FILE`: headless live-dive performance harness (real-time tour windows at
    /// increasing depths through the ACTUAL playback machinery), report + JSON, exit.
    pub(crate) divetest: Option<std::path::PathBuf>,
    /// CLI `--livetest FILE`: headless live-OUTPUT harness — plays a tour through the live
    /// pipeline and validates the frames it shows against offline renders of the same views.
    pub(crate) livetest: Option<std::path::PathBuf>,
    /// `--quick` with `--livetest`: skip the offline oracle (context/black metrics only).
    pub(crate) livetest_quick: bool,
    /// CLI `--reusetest`: measure reuse-first-zoom reprojection staleness vs Δ-octaves, exit.
    pub(crate) reusetest: bool,
    pub(crate) reusetest_done: bool,
    /// CLI `--resizetest`: headless window-resize aspect-invariant regression harness, exit.
    pub(crate) resizetest: bool,
    /// CLI `--bench-matrix [--bless] [--reps N]`: run the path-coverage perf + regression suite,
    /// compare against (or bless) the baseline, and exit.
    pub(crate) bench_matrix: bool,
    pub(crate) bench_matrix_done: bool,
}

impl crate::FractadyneApp {
    /// Per-frame advance hooks for the in-loop dev harnesses (`--shot`, `--uitest`,
    /// `--soak`, `--juliadive`, `--autodive`, `--chunk-sweep`, `--motiontest`).
    /// Order matters and is documented inline; moved verbatim from `update()`.
    pub(crate) fn harness_frame_hooks(
        &mut self,
        ctx: &eframe::egui::Context,
        gpu: &Option<(eframe::wgpu::Device, eframe::wgpu::Queue)>,
        gpu_name: &Option<String>,
    ) {
        // --uitest: advance the scripted UI/live walk. Runs each frame and does NOT exit early —
        // the flags it sets (which dialog is open, which view) must be drawn by the rest of this
        // update(); it screenshots and exits itself once the walk is done. Gated on GPU being up.
        // --shot: load, settle, capture, exit. Before --uitest so the two can never both
        // drive a frame.
        if let (true, Some(gpu)) = (self.harness.shot.is_some(), gpu) {
            self.shot_frame(ctx, gpu);
        }
        if self.harness.uitest.is_some() && gpu.is_some() {
            self.uitest_frame(ctx, gpu_name.as_deref());
        }
        // --soak: sit at a deep view and assert the app keeps producing frames.
        if self.harness.soak.is_some() && gpu.is_some() {
            self.soak_frame(ctx);
        }
        // --recordtest: drive live frames, then audit the frame record against its own count.
        if self.harness.recordtest.is_some() && gpu.is_some() {
            self.recordtest_frame(ctx);
        }
        // --dualsettle: dev harness for the dual-view Julia SETTLE path (same in-loop pattern).
        if self.harness.dualsettle.is_some() && gpu.is_some() {
            self.dualsettle_frame(ctx);
        }
        // --juliadive: dev harness for the dual-view Julia motion path (same in-loop pattern).
        if self.harness.juliadive.is_some() && gpu.is_some() {
            self.juliadive_frame(ctx);
        }
        // --autodive: unpaced controller hammer (same in-loop pattern). Must run AFTER the
        // measurement apply above, so the reading it samples is this frame's.
        if self.harness.autodive.is_some() && gpu.is_some() {
            self.autodive_frame(ctx);
        }
        // --chunk-sweep: let the live view settle, then measure the chunked iterate's per-window
        // wall cost offscreen and exit. It borrows the resident reference, so it MUST run after
        // the reference pipeline has had frames to finish — hence the settle countdown inside.
        if self.harness.chunk_sweep.is_some() {
            if let Some((dev, q)) = gpu {
                let dev = dev.clone();
                let q = q.clone();
                if self.chunk_sweep_step(&dev, &q) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                } else {
                    ctx.request_repaint();
                }
            }
        }
        // --deviceloss-repro: builds its own reference synchronously and measures the floor-window
        // cost, so it does NOT wait for the live pipeline — it runs on the first frame it gets a
        // device and closes. Same in-loop pattern as chunk-sweep.
        if self.harness.deviceloss_repro.is_some() {
            if let Some((dev, q)) = gpu {
                let dev = dev.clone();
                let q = q.clone();
                if self.deviceloss_repro_step(&dev, &q) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                } else {
                    ctx.request_repaint();
                }
            }
        }
        // --motiontest: the motion-presentation gate (same in-loop pattern). It sets this frame's
        // input state (zoom_vel / the Home glide), so it runs BEFORE the central draw below.
        if self.harness.motiontest.is_some() && gpu.is_some() {
            self.motiontest_frame(ctx);
        }
        // --zoomtest: the on-screen update-latency harness (same in-loop pattern). It holds the
        // virtual Space key the central draw and the lookahead pump read this frame, and stamps the
        // interval since the previous update — so it too runs BEFORE the central draw.
        if self.harness.zoomtest.is_some() && gpu.is_some() {
            self.zoomtest_frame(ctx);
        }
    }

    /// The harness branches of `cli_mode_ladder`, which runs them first: each runs its suite and
    /// exits, or does nothing.
    pub(crate) fn harness_ladder(&mut self, gpu: &Option<(eframe::wgpu::Device, eframe::wgpu::Queue)>) {
        // CLI self-test: run the GPU validation suite, print the report, and exit with a
        // status code (0 = all passed).
        if self.selftest.run && !self.selftest.done {
            if let Some((dev, q)) = gpu {
                self.selftest.done = true;
                let ok = self.run_selftest(dev, q);
                crate::exit(if ok { 0 } else { 1 });
            }
        }

        // CLI profiling: render the benchmark regions, time the costly stages, log to `logs/`.
        if self.profile.run && !self.profile.done {
            if let Some((dev, q)) = gpu {
                self.profile.done = true;
                let regions = match &self.profile.regions {
                    Some(path) => match profile::load_regions(std::path::Path::new(path)) {
                        Ok(r) => r,
                        // FATAL, not a fallback. This fell back to the built-ins and exited 0,
                        // so a regions file that failed to parse (measured: one cp1252 em-dash)
                        // profiled eight unrelated views and reported success — the exact trap the
                        // F3 harness taught us to check output files for. The flag is an explicit
                        // request; honouring a different one is worse than stopping.
                        Err(e) => {
                            eprintln!("fractadyne: --regions {path}: {e}");
                            crate::exit(2);
                        }
                    },
                    None => profile::default_regions(),
                };
                let secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let out = self.profile.out.clone().unwrap_or_else(|| {
                    std::path::PathBuf::from(format!("logs/profile-{}.json", Self::file_stamp(secs)))
                });
                let reps = self.profile.reps;
                self.run_profile(dev, q, &regions, reps, &out);
                crate::exit(0);
            }
        }

        // CLI path-matrix benchmark: exercise every rendering path, compare (or bless) the
        // baseline, and flag regressions. `--bless` records; `--reps N` sets timed reps.
        if self.harness.bench_matrix && !self.harness.bench_matrix_done {
            if let Some((dev, q)) = gpu {
                self.harness.bench_matrix_done = true;
                let bless = self.selftest.bless; // shared `--bless` flag
                let reps = self.profile.reps; // shared `--reps N` flag (default 5)
                let code = self.run_bench_matrix(dev, q, bless, reps, false);
                crate::exit(code);
            }
        }

        // CLI resize regression harness: scripted drag-resize through the real frame logic,
        // asserting every painted frame is aspect-correct (exit 0 = invariant held).
        if self.harness.resizetest {
            if let Some((dev, q)) = gpu {
                self.harness.resizetest = false;
                self.run_resizetest(dev, q); // exits
            }
        }

        if self.harness.reusetest && !self.harness.reusetest_done {
            if let Some((dev, q)) = gpu {
                self.harness.reusetest_done = true;
                self.run_reusetest(dev, q);
                crate::exit(0);
            }
        }

        // CLI headless live-dive harness: real-time tour windows at increasing depths through the
        // actual playback machinery (pacer + lookahead + reuse-hold), stats per depth band, exit.
        if let Some(tour) = self.harness.divetest.clone() {
            if let Some((dev, q)) = gpu {
                let secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let out = self.profile.out.clone().unwrap_or_else(|| {
                    std::path::PathBuf::from(format!("logs/divetest-{}.json", Self::file_stamp(secs)))
                });
                self.run_divetest(dev, q, &tour, &out);
                crate::exit(0);
            }
        }

        // CLI headless live-OUTPUT harness: play a tour through the live pipeline and validate the
        // frames it puts on screen against offline renders of the same views; exits by
        // `crate::verdict` (2 drift from the blessed baseline, 3 nothing judged).
        if let Some(tour) = self.harness.livetest.clone() {
            if let Some((dev, q)) = gpu {
                let (w, h) = self.tour_size_or(960, 540);
                let out = self
                    .render_cli.tour_cfg
                    .out
                    .clone()
                    .unwrap_or_else(|| std::path::PathBuf::from("logs"));
                let seg = self.render_cli.tour_cfg.segment.clone();
                let code =
                    self.run_livetest(dev, q, &tour, seg.as_deref(), [w, h], &out, self.harness.livetest_quick);
                crate::exit(code);
            }
        }

        if self.frametest.run {
            if let Some((dev, q)) = gpu {
                let secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let out = self.profile.out.clone().unwrap_or_else(|| {
                    std::path::PathBuf::from(format!("logs/frametest-{}.json", Self::file_stamp(secs)))
                });
                let (steps, hold, dive) = (self.frametest.steps, self.frametest.hold, self.frametest.dive);
                self.run_frametest(dev, q, steps, hold, dive, 512, &out);
                crate::exit(0);
            }
        }
    }
}
