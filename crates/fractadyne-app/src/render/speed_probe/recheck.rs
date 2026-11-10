//! ⭐THE SAVED READING'S RE-CHECK, AFTER LAUNCH (user, 2026-11-10: "check in the background on a
//! separate thread so it doesn't delay launch").
//!
//! A launch that finds a saved speed reading for its card and power state ([`saved`]) puts it to
//! use at once, and this measures one mode of it again later: the mode the first frame renders in,
//! at the iterations it was saved at. It starts once a frame of the view has been measured, the
//! view is still, and nobody has touched the mouse or keyboard for [`STILL_INPUT`]
//! ([`may_start`]); or [`WAIT_MAX`] after that first frame regardless. A reading that holds
//! ([`saved::still_holds`]) stands. One that does not is measured again in every mode, and the
//! result applied on the UI thread: each slow mode's opening, the ceiling lowered where the card
//! measured slower (`calibration::lower_to`; never raised, so a card that now reads faster keeps
//! the saved ceiling until the next launch, which uses the new reading), and the file rewritten.
//!
//! **Where it runs.** On Vulkan, DirectX 12 and Metal, on a thread of its own
//! (`fd-speed-recheck`), beside the window. ⛔**On OpenGL, on the UI thread.** wgpu's OpenGL
//! backend holds one context behind a lock for every call, and a thread that waits more than ONE
//! SECOND for it panics ("Could not lock adapter context. This is most-likely a deadlock.",
//! wgpu-hal 24 `gles::wgl::AdapterContext::lock`, `CONTEXT_LOCK_TIMEOUT_SECS`). Intel HD Graphics
//! 4400's driver compiles a shader at its first draw inside that lock, and the probe draws through
//! the whole-frame iterate pipeline, which the live view's chunked frames need not draw: ~4.4 s
//! for floatexp on Eris. Run on a thread, dev.12 (2026-11-10) died 1.2 s into the re-check, the
//! window's thread timed out on the lock. On the UI thread the window stops for that long, once,
//! which is why the re-check waits for a still view, and its time is discounted from the frame
//! interval it falls in like the start-up probe's (`Perf::startup_probe_ms`), or it would read as a
//! multi-second frame the GPU timestamps cannot see (`FRAME BUDGET IS BLIND`).
//!
//! **The notice** ([`SpeedNotice`], user, 2026-11-10: "keep the check and show a dialog letting the
//! user know what's happening"). Where the re-check stops the window, a small window says what is
//! happening FIRST: it is put up, the re-check waits [`NOTICE_FRAMES`] frames so that it is on the
//! screen (egui sizes a new window on its first frame, invisibly), and it stays there, drawn, while
//! the window cannot respond. Then it shows the result for [`NOTICE_HOLD`] and closes. Off OpenGL
//! the re-check stops nothing and shows nothing.

use super::{lowered_costs, mode_name, plan_request, ramp, remeasure_at, saved, taken_now, Finding, Plan, MODES};
use crate::power_state::PowerState;
use crate::FractadyneApp;
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// How long after the first measured frame the re-check waits for a still view before it runs
/// anyway: a view kept moving (a long glide, the autopilot) still gets its reading confirmed.
pub(crate) const WAIT_MAX: Duration = Duration::from_secs(20);
/// No mouse or keyboard input for this long counts as the view being left alone.
pub(crate) const STILL_INPUT: Duration = Duration::from_secs(1);
/// Frames after the reading was put to use that stand for "the picture is up" where no frame has
/// been measured yet (a card whose frames are not priced in the view's mode).
pub(crate) const SHOWN_FRAMES: u64 = 120;
/// Frames the notice is up before the re-check stops the window: egui lays a new window out on its
/// first frame without drawing it, and the next frames present it.
pub(crate) const NOTICE_FRAMES: u64 = 3;
/// How long the notice shows the re-check's result before it closes itself.
pub(crate) const NOTICE_HOLD: Duration = Duration::from_secs(4);

/// The notice the re-check shows where it stops the window (the module header).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SpeedNotice {
    /// About to run, and running: the window cannot respond.
    Checking,
    /// Done at `at`: the saved reading `held`, or every mode was measured again.
    Done { held: bool, at: Instant },
}

/// Whether the re-check may start: `shown_for` since the first frame was measured (`None`: not
/// yet), the view `still` (settled, nothing in flight), and `input_idle` since the last mouse or
/// keyboard input (`None` where that cannot be read, which does not hold it back).
pub(crate) fn may_start(shown_for: Option<Duration>, still: bool, input_idle: Option<Duration>) -> bool {
    match shown_for {
        None => false,
        Some(d) if d >= WAIT_MAX => true,
        Some(_) => still && input_idle.is_none_or(|i| i >= STILL_INPUT),
    }
}

/// The saved reading the launch put to use, and what its re-check needs.
pub(crate) struct Saved {
    pub(crate) card: saved::Card,
    pub(crate) power: PowerState,
    /// When it was taken ("earlier" where the file does not say).
    pub(crate) taken: String,
    pub(crate) findings: [Finding; 3],
    /// The mode re-checked (an index into [`MODES`]): the first frame's.
    pub(crate) k: usize,
    /// Each mode's cost per step under which nothing would change (`fast_below`).
    pub(crate) below: [f64; 3],
    /// The cost per step above which a mode's opening comes down (`speed_probe_opens`).
    pub(crate) opening_floor: f64,
    /// The live renderer's iterate pipelines, one per mode (`live_iter_pipeline`).
    pub(crate) live: [Option<fractadyne_gpu::LiveIterPipeline>; 3],
    /// The device is on OpenGL: the re-check runs on the UI thread (the module header).
    pub(crate) gl: bool,
    /// The frame the reading was put to use on.
    pub(crate) adopted_frame: u64,
}

/// A saved reading waiting for its re-check, or being re-checked (`FractadyneApp::speed_recheck`).
pub(crate) struct Recheck {
    saved: Saved,
    /// When a frame of the view was first measured, or [`SHOWN_FRAMES`] passed.
    shown_at: Option<Instant>,
    running: Option<Running>,
    /// The frame the notice went up on (OpenGL), and whether the view was still then: the re-check
    /// runs [`NOTICE_FRAMES`] later.
    announced: Option<(u64, bool)>,
}

struct Running {
    rx: mpsc::Receiver<Outcome>,
    started: Instant,
}

/// What the re-check found.
struct Outcome {
    /// The re-checked mode's cost per step at the saved iterations, and how long that took, ms.
    fresh: Result<f64, String>,
    fresh_ms: f64,
    /// Every mode measured again, with how long each took, where the reading did not hold.
    again: Option<[(Result<Finding, String>, String); 3]>,
}

impl Recheck {
    pub(crate) fn new(saved: Saved) -> Recheck {
        Recheck { saved, shown_at: None, running: None, announced: None }
    }
}

/// What the re-check needs from [`Saved`], to move to its thread: the mode re-checked, the saved
/// findings, each mode's `fast_below`, the pipelines.
type Job = (usize, [Finding; 3], [f64; 3], [Option<fractadyne_gpu::LiveIterPipeline>; 3]);

/// `m`'s request, its reference built on first use.
fn request<'a>(
    reqs: &'a mut [Option<(fractadyne_gpu::ExportRequest, u32)>; 3],
    plans: &mut [Option<Plan>; 3],
    m: usize,
) -> &'a mut (fractadyne_gpu::ExportRequest, u32) {
    reqs[m].get_or_insert_with(|| plan_request(plans[m].take().expect("one plan a mode")))
}

/// The re-check, then, where the reading does not hold, every mode measured again.
fn run(dev: &eframe::wgpu::Device, q: &eframe::wgpu::Queue, (k, findings, below, live): Job, plans: [Plan; 3]) -> Outcome {
    const NONE: &str = "the renderer is not installed";
    let mut plans = plans.map(Some);
    let mut reqs = [None, None, None];
    let t = Instant::now();
    let fresh = match &live[k] {
        Some(l) => remeasure_at(dev, q, l, request(&mut reqs, &mut plans, k), findings[k]),
        None => Err(NONE.to_string()),
    };
    let fresh_ms = t.elapsed().as_secs_f64() * 1000.0;
    let holds = matches!(fresh, Ok(f) if saved::still_holds(findings[k], f, below[k]));
    let again = (!holds).then(|| {
        std::array::from_fn(|m| {
            let tm = Instant::now();
            let found = match &live[m] {
                Some(l) => ramp(dev, q, l, request(&mut reqs, &mut plans, m), below[m]),
                None => Err(NONE.to_string()),
            };
            (found, format!("{:.0} ms", tm.elapsed().as_secs_f64() * 1000.0))
        })
    });
    Outcome { fresh, fresh_ms, again }
}

impl FractadyneApp {
    /// Each frame, with the GPU handles: start the saved reading's re-check when it may
    /// ([`may_start`]), or take what it found.
    pub(crate) fn speed_recheck_step(&mut self, ctx: &egui::Context, dev: &eframe::wgpu::Device, q: &eframe::wgpu::Queue) {
        let Some(r) = &self.speed_recheck else { return };
        if let Some(run) = &r.running {
            let out = match run.rx.try_recv() {
                Ok(out) => Some(out),
                Err(mpsc::TryRecvError::Empty) => return,
                Err(mpsc::TryRecvError::Disconnected) => None,
            };
            let r = self.speed_recheck.take().expect("checked above");
            let took_s = r.running.as_ref().map_or(0.0, |r| r.started.elapsed().as_secs_f64());
            match out {
                Some(out) => self.speed_recheck_landed(r.saved, out, &format!("in the background, {took_s:.1} s on its own thread")),
                None => crate::diag::log_line("wgpu", "speed re-check: its thread ended without a reading; the saved reading stays in use"),
            }
            return;
        }
        let measured = self.perf.worst_rate_steps_per_ms(0).is_some();
        let shown = measured || self.perf.frame_idx >= r.saved.adopted_frame + SHOWN_FRAMES;
        let still = self.view_idle(0) && (!self.dual || self.view_idle(1));
        let shown_at = r.shown_at.or_else(|| shown.then(Instant::now));
        // Once the notice is up, the re-check runs whatever happens next: a notice left up
        // because someone moved the mouse would promise a pause that never comes.
        match r.announced {
            None if !may_start(shown_at.map(|t| t.elapsed()), still, crate::sysinfo::user_idle()) => {
                self.speed_recheck.as_mut().expect("checked above").shown_at = shown_at;
                return;
            }
            None if r.saved.gl => {
                let frame = self.perf.frame_idx;
                let r = self.speed_recheck.as_mut().expect("checked above");
                r.shown_at = shown_at;
                r.announced = Some((frame, still));
                self.dialogs.speed_notice = Some(SpeedNotice::Checking);
                ctx.request_repaint();
                return;
            }
            Some((at, _)) if self.perf.frame_idx < at + NOTICE_FRAMES => {
                ctx.request_repaint();
                return;
            }
            _ => {}
        }
        // Why it started: when the notice went up, where there was one.
        let still = r.announced.map_or(still, |(_, s)| s);
        let plans = MODES.map(|m| self.speed_probe_plan(m));
        let s = &r.saved;
        let job = (s.k, s.findings, s.below, s.live.clone());
        let waited = shown_at.map_or(0.0, |t| t.elapsed().as_secs_f64());
        let gl = s.gl;
        crate::diag::breadcrumb(format!(
            "speed re-check: {} {} ({}; {waited:.1} s after the first frame)",
            mode_name(MODES[s.k]),
            if gl { "on the UI thread (OpenGL)" } else { "on its own thread" },
            if still { "the view is still" } else { "waited long enough" }
        ));
        if gl {
            let t = Instant::now();
            let out = run(dev, q, job, plans);
            let ms = t.elapsed().as_secs_f64() * 1000.0;
            // In the next frame interval, and no frame's dispatch: discounted like the start-up probe.
            self.perf.startup_probe_ms += ms;
            self.dialogs.speed_notice = Some(SpeedNotice::Done { held: out.again.is_none(), at: Instant::now() });
            ctx.request_repaint();
            let r = self.speed_recheck.take().expect("checked above");
            self.speed_recheck_landed(r.saved, out, &format!("on the UI thread, which OpenGL needs, {ms:.0} ms"));
            return;
        }
        let (tx, rx) = mpsc::channel();
        let (dev, q) = (dev.clone(), q.clone());
        super::super::spawn_named("fd-speed-recheck", move || {
            let _busy = crate::blinken::registry::busy(crate::blinken::registry::Role::Probe);
            let _ = tx.send(run(&dev, &q, job, plans));
        });
        let r = self.speed_recheck.as_mut().expect("checked above");
        r.shown_at = shown_at;
        r.running = Some(Running { rx, started: Instant::now() });
    }

    /// The re-check's outcome, on the UI thread (`how`: where it ran and how long it took): logged,
    /// and where the reading did not hold, the new measurement applied and filed (the module header).
    fn speed_recheck_landed(&mut self, s: Saved, out: Outcome, how: &str) {
        let mode = MODES[s.k];
        let was = match s.findings[s.k] {
            Finding::Measured { ms_per_step, .. } => format!("{ms_per_step:.3e}"),
            Finding::Fast { ms_per_step, .. } => format!("under {ms_per_step:.2e}"),
        };
        let fresh = match &out.fresh {
            Ok(f) => format!("{} re-checked at {f:.3e} ms/step against the saved {was} ({:.0} ms)", mode_name(mode), out.fresh_ms),
            Err(e) => format!("{} could not be re-checked ({e}; {:.0} ms)", mode_name(mode), out.fresh_ms),
        };
        let Some(again) = out.again else {
            crate::diag::log_line("wgpu", &format!("speed re-check ({how}): {fresh}; the saved reading from {} holds", s.taken));
            return;
        };
        let parts = self.speed_probe_apply(&again, s.opening_floor);
        let found = again.map(|(f, _)| f);
        let ceiling = if crate::calibration::lower_to(lowered_costs(&found)) {
            "the dispatch ceiling lowered to this measurement"
        } else {
            "the dispatch ceiling unchanged"
        };
        let filed = match fractadyne_state::config_dir() {
            None => "not saved (no config folder)".to_string(),
            Some(d) => {
                let readings = saved::load(&d);
                let (next, what) = match &found {
                    [Ok(d0), Ok(f), Ok(x)] => (
                        saved::with(
                            &readings,
                            saved::Reading {
                                card: s.card.clone(),
                                power: s.power.clone(),
                                taken: taken_now(),
                                direct: saved::SavedFinding::of(*d0),
                                df32: saved::SavedFinding::of(*f),
                                floatexp: saved::SavedFinding::of(*x),
                            },
                        ),
                        format!("saved to {}", saved::FILE),
                    ),
                    // A mode not measured: the old reading goes, and the next launch measures.
                    _ => (saved::without(&readings, &s.card, &s.power), format!("the old reading removed from {}", saved::FILE)),
                };
                match saved::store(&d, &next) {
                    Ok(()) => what,
                    Err(e) => format!("not saved ({e})"),
                }
            }
        };
        crate::diag::log_line(
            "wgpu",
            &format!(
                "speed re-check ({how}): {fresh}; the saved reading from {} no longer holds, so every mode was \
                 measured again: {}; {ceiling}; {filed}",
                s.taken,
                parts.join("; ")
            ),
        );
    }
}

impl FractadyneApp {
    /// The speed check's notice ([`SpeedNotice`]), while there is one: what is happening, then the
    /// result, closed [`NOTICE_HOLD`] after it (or by its title bar's ✕). It only presents, so it
    /// has no Cancel (UI-DESIGN.md §8.2), and there would be nothing to cancel.
    pub(crate) fn draw_speed_notice(&mut self, ctx: &egui::Context) {
        let Some(notice) = self.dialogs.speed_notice else { return };
        if let SpeedNotice::Done { at, .. } = notice {
            let left = NOTICE_HOLD.saturating_sub(at.elapsed());
            if left.is_zero() {
                self.dialogs.speed_notice = None;
                return;
            }
            ctx.request_repaint_after(left);
        }
        let mut open = true;
        egui::Window::new(tr!(speed_check_title))
            .id(egui::Id::new("fractadyne.speed_notice"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut open)
            .show(ctx, |ui| {
                ui.set_max_width(380.0);
                ui.label(tr!(speed_check_running));
                if let SpeedNotice::Done { held, .. } = notice {
                    ui.add_space(6.0);
                    ui.strong(if held { tr!(speed_check_held) } else { tr!(speed_check_updated) });
                }
            });
        if !open {
            self.dialogs.speed_notice = None;
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/render/speed_probe/recheck/tests.rs"]
mod tests;
