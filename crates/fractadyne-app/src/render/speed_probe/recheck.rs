//! ⭐THE SAVED READING'S RE-CHECK, IN THE BACKGROUND (user, 2026-11-10: "check in the background
//! on a separate thread so it doesn't delay launch").
//!
//! A launch that finds a saved speed reading for its card and power state ([`saved`]) puts it to
//! use at once, and this measures one mode of it again later, on a thread of its own
//! (`fd-speed-recheck`): the mode the first frame renders in, at the iterations it was saved at. It
//! starts once a frame of the view has been measured, the view is still, and nobody has touched the
//! mouse or keyboard for [`STILL_INPUT`] ([`may_start`]); or [`WAIT_MAX`] after that first frame
//! regardless. A reading that holds ([`saved::still_holds`]) stands. One that does not is measured
//! again in every mode on the same thread, and the result applied on the UI thread: each slow
//! mode's opening, the ceiling lowered where the card measured slower (`calibration::lower_to`;
//! never raised, so a card that now reads faster keeps the saved ceiling until the next launch,
//! which uses the new reading), and the file rewritten.
//!
//! ⚠On OpenGL it cannot run BESIDE the window. wgpu's OpenGL backend holds one context behind a
//! lock for every call (wgpu-hal `gles::wgl::AdapterContext::lock`), so the thread's passes and
//! the window's take turns; and Intel HD Graphics 4400's driver compiles a shader at its FIRST
//! DRAW, inside that lock. The probe draws through the whole-frame iterate pipeline, which the live
//! view's chunked frames need not draw, so that compile is the re-check's own: ~4.4 s for floatexp
//! on Eris (dev.11, 2026-11-10), against a first picture ~33 s after launch. There the window
//! stops for that long, once, which is why the re-check waits for a still view; and the time it
//! holds the GPU is taken out of the frame interval it falls in ([`BusyClock`], OpenGL only), or it
//! would read as a multi-second frame the GPU timestamps cannot see (`FRAME BUDGET IS BLIND`).

use super::{lowered_costs, mode_name, plan_request, ramp, remeasure_at, saved, taken_now, Finding, Plan, MODES};
use crate::power_state::PowerState;
use crate::FractadyneApp;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

/// How long after the first measured frame the re-check waits for a still view before it runs
/// anyway: a view kept moving (a long glide, the autopilot) still gets its reading confirmed.
pub(crate) const WAIT_MAX: Duration = Duration::from_secs(20);
/// No mouse or keyboard input for this long counts as the view being left alone.
pub(crate) const STILL_INPUT: Duration = Duration::from_secs(1);
/// Frames after the reading was put to use that stand for "the picture is up" where no frame has
/// been measured yet (a card whose frames are not priced in the view's mode).
pub(crate) const SHOWN_FRAMES: u64 = 120;

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

/// The time the re-check's thread holds the GPU, so the frame interval it falls in can leave it
/// out (`Perf::recheck_clock`, OpenGL only: see the module header).
#[derive(Default)]
pub(crate) struct BusyClock {
    /// Since when the thread has held the GPU, if it does now, and the ms of the spans that ended.
    state: Mutex<(Option<Instant>, f64)>,
    /// The thread holds the GPU no more (set after its last span).
    finished: AtomicBool,
}

/// A span of [`BusyClock`]: ends when dropped.
pub(crate) struct Held<'a>(&'a BusyClock);

impl BusyClock {
    fn state(&self) -> std::sync::MutexGuard<'_, (Option<Instant>, f64)> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Start a span, now.
    pub(crate) fn hold(&self) -> Held<'_> {
        self.state().0 = Some(Instant::now());
        Held(self)
    }

    /// All the time held, ms, up to `now`, the span under way included.
    pub(crate) fn total_ms(&self, now: Instant) -> f64 {
        let s = self.state();
        s.1 + s.0.map_or(0.0, |t| now.saturating_duration_since(t).as_secs_f64() * 1000.0)
    }

    pub(crate) fn finish(&self) {
        self.finished.store(true, Ordering::Release);
    }

    pub(crate) fn finished(&self) -> bool {
        self.finished.load(Ordering::Acquire)
    }
}

impl Drop for Held<'_> {
    fn drop(&mut self) {
        let mut s = self.0.state();
        if let Some(t) = s.0.take() {
            s.1 += t.elapsed().as_secs_f64() * 1000.0;
        }
    }
}

/// The ms `slot`'s clock held the GPU since the last call (the clock, and what was taken from it
/// so far): the part of a frame interval the re-check stopped the window for. Empties the slot once
/// the thread has finished and all of its time is taken.
pub(crate) fn overlap_ms(slot: &mut Option<(Arc<BusyClock>, f64)>, now: Instant) -> f64 {
    let Some((clock, taken)) = slot else { return 0.0 };
    let finished = clock.finished();
    let total = clock.total_ms(now);
    let d = (total - *taken).max(0.0);
    *taken = taken.max(total);
    if finished {
        *slot = None;
    }
    d
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
    /// The device is on OpenGL, where the thread's passes stop the window's (the module header).
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
}

struct Running {
    rx: mpsc::Receiver<Outcome>,
    started: Instant,
}

/// What the thread found.
struct Outcome {
    /// The re-checked mode's cost per step at the saved iterations, and how long that took, ms.
    fresh: Result<f64, String>,
    fresh_ms: f64,
    /// Every mode measured again, with how long each took, where the reading did not hold.
    again: Option<[(Result<Finding, String>, String); 3]>,
}

impl Recheck {
    pub(crate) fn new(saved: Saved) -> Recheck {
        Recheck { saved, shown_at: None, running: None }
    }
}

/// `m`'s request, its reference built on first use.
fn request<'a>(
    reqs: &'a mut [Option<(fractadyne_gpu::ExportRequest, u32)>; 3],
    plans: &mut [Option<Plan>; 3],
    m: usize,
) -> &'a mut (fractadyne_gpu::ExportRequest, u32) {
    reqs[m].get_or_insert_with(|| plan_request(plans[m].take().expect("one plan a mode")))
}

/// The thread: the re-check, then, where the reading does not hold, every mode measured again.
fn run(
    dev: &eframe::wgpu::Device,
    q: &eframe::wgpu::Queue,
    s: (usize, [Finding; 3], [f64; 3], [Option<fractadyne_gpu::LiveIterPipeline>; 3]),
    plans: [Plan; 3],
    clock: &BusyClock,
) -> Outcome {
    let (k, findings, below, live) = s;
    let mut plans = plans.map(Some);
    let mut reqs = [None, None, None];
    const NONE: &str = "the renderer is not installed";
    let t = Instant::now();
    let fresh = match &live[k] {
        Some(l) => remeasure_at(dev, q, l, request(&mut reqs, &mut plans, k), findings[k], Some(clock)),
        None => Err(NONE.to_string()),
    };
    let fresh_ms = t.elapsed().as_secs_f64() * 1000.0;
    let holds = matches!(fresh, Ok(f) if saved::still_holds(findings[k], f, below[k]));
    let again = (!holds).then(|| {
        std::array::from_fn(|m| {
            let tm = Instant::now();
            let found = match &live[m] {
                Some(l) => ramp(dev, q, l, request(&mut reqs, &mut plans, m), below[m], Some(clock)),
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
    pub(crate) fn speed_recheck_step(&mut self, dev: &eframe::wgpu::Device, q: &eframe::wgpu::Queue) {
        let Some(r) = &self.speed_recheck else { return };
        match &r.running {
            Some(run) => {
                let out = match run.rx.try_recv() {
                    Ok(out) => Some(out),
                    Err(mpsc::TryRecvError::Empty) => return,
                    Err(mpsc::TryRecvError::Disconnected) => None,
                };
                let r = self.speed_recheck.take().expect("checked above");
                match out {
                    Some(out) => self.speed_recheck_landed(r, out),
                    None => {
                        self.perf.recheck_clock = None;
                        crate::diag::log_line("wgpu", "speed re-check: its thread ended without a reading; the saved reading stays in use");
                    }
                }
            }
            None => {
                let measured = self.perf.worst_rate_steps_per_ms(0).is_some();
                let shown = measured || self.perf.frame_idx >= r.saved.adopted_frame + SHOWN_FRAMES;
                let still = self.view_idle(0) && (!self.dual || self.view_idle(1));
                let shown_at = match r.shown_at {
                    Some(t) => Some(t),
                    None if shown => Some(Instant::now()),
                    None => None,
                };
                let go = may_start(shown_at.map(|t| t.elapsed()), still, crate::sysinfo::user_idle());
                let plans = go.then(|| MODES.map(|m| self.speed_probe_plan(m)));
                let r = self.speed_recheck.as_mut().expect("checked above");
                r.shown_at = shown_at;
                if let Some(plans) = plans {
                    let s = &r.saved;
                    let job = (s.k, s.findings, s.below, s.live.clone());
                    let clock = Arc::new(BusyClock::default());
                    if s.gl {
                        self.perf.recheck_clock = Some((clock.clone(), 0.0));
                    }
                    let waited = shown_at.map_or(0.0, |t| t.elapsed().as_secs_f64());
                    crate::diag::breadcrumb(format!(
                        "speed re-check: {} on its own thread ({}; {waited:.1} s after the first frame)",
                        mode_name(MODES[s.k]),
                        if still { "the view is still" } else { "waited long enough" }
                    ));
                    let (tx, rx) = mpsc::channel();
                    let (dev, q) = (dev.clone(), q.clone());
                    super::super::spawn_named("fd-speed-recheck", move || {
                        let _busy = crate::blinken::registry::busy(crate::blinken::registry::Role::Probe);
                        let out = run(&dev, &q, job, plans, &clock);
                        clock.finish();
                        let _ = tx.send(out);
                    });
                    r.running = Some(Running { rx, started: Instant::now() });
                }
            }
        }
    }

    /// The re-check's outcome, on the UI thread: logged, and where the reading did not hold, the
    /// new measurement applied and filed (the module header).
    fn speed_recheck_landed(&mut self, r: Recheck, out: Outcome) {
        let Recheck { saved: s, running, .. } = r;
        let took_s = running.map_or(0.0, |r| r.started.elapsed().as_secs_f64());
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
            crate::diag::log_line(
                "wgpu",
                &format!("speed re-check (in the background, {took_s:.1} s on its own thread): {fresh}; the saved reading from {} holds", s.taken),
            );
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
                "speed re-check (in the background, {took_s:.1} s on its own thread): {fresh}; the saved reading from {} \
                 no longer holds, so every mode was measured again there: {}; {ceiling}; {filed}",
                s.taken,
                parts.join("; ")
            ),
        );
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/render/speed_probe/recheck/tests.rs"]
mod tests;
