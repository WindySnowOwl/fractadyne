//! Start-up's shader compile, with the window showing.
//!
//! eframe shows the window only after its first frame (eframe 0.31: built hidden, made visible in
//! `post_rendering`), and the renderer's pipelines used to be built before that frame, in the
//! creation callback (`FractadyneApp::attach_gpu`). With the driver's shader cache cold, the first
//! start after an update of Fractadyne or of the driver, that was 10-16 s on the RTX 3080 and the
//! RX 6800 XT with nothing on the screen or the taskbar (2026-11-24): long enough to think the
//! program had not started. Now a worker builds them while `update` draws a screen that says what
//! is happening. On OpenGL, whose context a worker cannot hold that long (the thread waiting for it
//! panics: `fractadyne_gpu::compile_custom_async`), the screen is shown first and the window's
//! thread builds them after it.

use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use eframe::egui;
use eframe::egui_wgpu::RenderState;
use fractadyne_gpu::{FusionVerdict, GpuError, PreparedRenderer};

use crate::diag;

/// How long the first frame waits for the worker before it shows the screen: a warm start builds
/// in 0.1-0.7 s, and a screen that flashes up for a moment would say nothing.
const FIRST_WAIT: Duration = Duration::from_millis(400);

/// On the window's thread (OpenGL), how many frames of the screen, and for how long, before the
/// build blocks it. ⛔Not the first two: on OpenGL the frames just after the window shows are
/// presented at a stale size, and the one the build froze on screen showed only the screen's last
/// lines, shifted to the top over a black half (RTX 3080, 2026-11-24, for the whole 155 s compile).
const GL_FRAMES: u32 = 8;
const GL_SETTLE: Duration = Duration::from_millis(800);

/// What the build hands back: the product-fusion probe's verdict and the renderer.
type Built = (Result<FusionVerdict, GpuError>, PreparedRenderer);

/// Where start-up's renderer is.
pub(crate) enum StartupGpu {
    /// Installed (also the state before `attach_gpu`, when there is no window yet).
    Ready,
    /// A worker builds it; `waited` once the first frame has waited [`FIRST_WAIT`] for it.
    Building { rx: Receiver<Built>, since: Instant, waited: bool },
    /// OpenGL (or no worker thread): the window's thread builds it once the screen has been drawn
    /// [`GL_FRAMES`] times over [`GL_SETTLE`] (`frames` so far).
    OnThisThread { since: Instant, frames: u32 },
}

impl StartupGpu {
    /// Start building the renderer for `rs`'s device: on a worker, or (OpenGL) on the window's
    /// thread after the screen is up.
    pub(crate) fn start(rs: &RenderState) -> Self {
        let since = Instant::now();
        if rs.adapter.get_info().backend == eframe::wgpu::Backend::Gl {
            return Self::OnThisThread { since, frames: 0 };
        }
        let (device, queue, format) = (rs.device.clone(), rs.queue.clone(), rs.target_format);
        let (tx, rx) = std::sync::mpsc::channel();
        let spawned = std::thread::Builder::new().name("fd-startup-shaders".into()).spawn(move || {
            let _ = tx.send(build(&device, &queue, format));
        });
        match spawned {
            Ok(_) => Self::Building { rx, since, waited: false },
            Err(e) => {
                diag::log_line("start", &format!("no thread for the shader compile ({e}): the window's thread builds it"));
                Self::OnThisThread { since, frames: 0 }
            }
        }
    }

    fn since(&self) -> Option<Instant> {
        match self {
            Self::Ready => None,
            Self::Building { since, .. } | Self::OnThisThread { since, .. } => Some(*since),
        }
    }
}

/// The product-fusion probe, then the renderer's pipelines: the probe decides whether the iterate
/// pipelines get the WGSL product barrier, so it runs first. ⭐Start-up's phases as breadcrumbs: a
/// driver that dies compiling a shader leaves a crash report whose `activity` says which (Eris,
/// 2026-11-09, died 7 s into an OpenGL start-up).
fn build(device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue, format: eframe::wgpu::TextureFormat) -> Built {
    diag::breadcrumb("start-up: the product-fusion probe (one pipeline)".into());
    let fusion = fractadyne_gpu::detect_product_fusion(device, queue);
    diag::breadcrumb("start-up: building the renderer's pipelines".into());
    (fusion, fractadyne_gpu::prepare_renderer(device, queue, format))
}

impl crate::FractadyneApp {
    /// Once a frame, first thing, until start-up's renderer is installed: take it when it is built
    /// (and finish start-up), else draw the start-up screen. `true` when the frame goes on as usual.
    pub(crate) fn startup_gpu_ready(&mut self, ctx: &egui::Context, frame: &eframe::Frame) -> bool {
        let Some(rs) = frame.wgpu_render_state() else { return true };
        let since = match self.startup_gpu.since() {
            None => return true,
            Some(s) => s,
        };
        // Time this thread spends here (the first frame's wait; on OpenGL the build) is not GPU work:
        // discounted from the next frame's interval like the speed probe's, or the frame budget reads
        // a 195 s OpenGL build as one frame it never priced (PLUTO's 3070 on OpenGL, 2026-11-24).
        let t = Instant::now();
        let built = match &mut self.startup_gpu {
            StartupGpu::Ready => None,
            StartupGpu::Building { rx, waited, .. } => {
                let wait = if *waited { Duration::ZERO } else { FIRST_WAIT };
                *waited = true;
                match rx.recv_timeout(wait) {
                    Ok(b) => Some(b),
                    Err(RecvTimeoutError::Timeout) => None,
                    // Its panic wrote a crash report; this thread's ends the program, with the dialog.
                    Err(RecvTimeoutError::Disconnected) => {
                        panic!("start-up: the thread building the renderer's pipelines ended without them")
                    }
                }
            }
            StartupGpu::OnThisThread { frames, .. } => {
                if *frames >= GL_FRAMES && since.elapsed() >= GL_SETTLE {
                    Some(build(&rs.device, &rs.queue, rs.target_format))
                } else {
                    *frames += 1;
                    None
                }
            }
        };
        match built {
            Some(b) => {
                // Only this frame's: the screen's frames return before any interval is measured.
                self.perf.startup_probe_ms += t.elapsed().as_secs_f64() * 1000.0;
                self.finish_startup_gpu(rs, b, since);
                true
            }
            None => {
                self.draw_startup_screen(ctx, since);
                // The spinner and the seconds; on OpenGL, the frames that lead up to the build, at once.
                if matches!(self.startup_gpu, StartupGpu::OnThisThread { .. }) {
                    ctx.request_repaint();
                } else {
                    diag::repaint_after(ctx, Duration::from_millis(100));
                }
                false
            }
        }
    }

    /// Start-up's last steps, which need the renderer: install it, start the trap shapes' own
    /// pipelines, and mark start-up done.
    fn finish_startup_gpu(&mut self, rs: &RenderState, (fusion, prepared): Built, since: Instant) {
        fractadyne_gpu::install_prepared_renderer(rs, prepared);
        self.startup_gpu = StartupGpu::Ready;
        // The trap shapes' own pipelines, compiled on a worker while the window starts (seconds cold).
        fractadyne_gpu::warm_trap_shapes(rs);
        crate::log_product_fusion("main", &fusion);
        // A driver may have put its own crash handler in place of ours as it loaded.
        diag::native_crash::reinstall("the graphics driver's start-up");
        diag::log_line(
            "start",
            &format!("the renderer's pipelines were ready {} ms after the window opened", since.elapsed().as_millis()),
        );
        diag::breadcrumb("start-up: the renderer is ready".into());
        diag::set_phase(diag::advice::Phase::Running);
        crate::gpu_choice::fallback::start_up_done(diag::advice::api_name(rs.adapter.get_info().backend));
    }

    /// What the window shows while start-up compiles the shaders.
    fn draw_startup_screen(&self, ctx: &egui::Context, since: Instant) {
        let on_this_thread = matches!(self.startup_gpu, StartupGpu::OnThisThread { .. });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space((ui.available_height() * 0.36).max(0.0));
                ui.add(egui::Spinner::new().size(28.0));
                ui.add_space(14.0);
                ui.heading(tr!(startup_shaders_title));
                ui.add_space(8.0);
                let width = ui.available_width().min(520.0);
                ui.allocate_ui_with_layout(egui::vec2(width, 0.0), egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(tr!(startup_shaders_body));
                    if on_this_thread {
                        ui.label(tr!(startup_shaders_gl_note));
                    }
                });
                // The seconds, where they can count: on OpenGL the build freezes the window at "0 s".
                if !on_this_thread {
                    ui.add_space(8.0);
                    ui.weak(tr!(startup_shaders_elapsed, seconds = since.elapsed().as_secs()));
                }
            });
        });
    }
}
