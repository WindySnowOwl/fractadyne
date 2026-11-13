//! The method sheet's cells that need a calculation of their own (coloring plan B7, step 4).
//!
//! ⭐**Each is an export of the view on screen in the cell's coloring**, at the sheet's cell size,
//! on a DEVICE OF ITS OWN on the window's card, on a thread of its own: the Export dialog's route
//! (`export::spawn_export_worker`), for its reason: two threads submitting to the window's device
//! is where wgpu has had hazards. One cell at a time; the view changing or the sheet closing stops
//! the one under way.
//!
//! ⭐**The request borrows the live view's resident reference** (`RefSource::Live`, as the reticle
//! and the autopilot's probe do), so a cell costs no bignum. ⛔It is never asked for unless that
//! reference serves the view (`live_reference_serves`): `Live` otherwise builds one, synchronously,
//! on the UI thread. Nor while the live view is still calculating, which has the card first.
//!
//! ⭐**Priced before it is paid for**: a probe at an eighth of the cell's size per side, its GPU
//! time scaled by the pixels ([`estimate`]). Within [`AUTO_BUDGET_S`] a cell is calculated by
//! itself; past it, it shows the estimate and a Render button. A statistic the live frame does not
//! hold may also lose the live tree's skipping (`aux_lanes_stale`), which is why the live frame's
//! own cost cannot price it.

use super::method_sheet::{CellSource, SheetCell};
use fractadyne_core::coloring::MethodGroup;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

/// A cell estimated to take longer than this, seconds, is calculated only when asked (its Render
/// button).
pub(crate) const AUTO_BUDGET_S: f64 = 2.0;
/// The probe's size against the cell's, per side.
pub(crate) const PROBE_DIV: u32 = 8;
/// A probe still running after this long is stopped, and its estimate is a floor.
pub(crate) const PROBE_LIMIT: Duration = Duration::from_secs(2);
/// How long what the cells show must hold still before one is calculated.
const SETTLE: Duration = Duration::from_millis(500);
/// How often the sheet looks for a finished cell while one is under way.
const POLL: Duration = Duration::from_millis(100);

/// A probe, to price a cell, or the cell itself.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum JobKind {
    Probe,
    Full,
}

/// What a probe measured.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Probe {
    /// The GPU time of its passes (timestamps), ms; 0 where the card has no timer.
    pub(crate) gpu_ms: f64,
    /// From the job's start to its answer, ms.
    pub(crate) wall_ms: f64,
    /// Its pixels.
    pub(crate) px: u64,
    /// Stopped at [`PROBE_LIMIT`]: `wall_ms` is how long it ran.
    pub(crate) stopped: bool,
}

/// How long a cell should take.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Estimate {
    pub(crate) secs: f64,
    /// A floor (the probe was stopped), not an estimate.
    pub(crate) at_least: bool,
}

/// A cell of `cell_px` pixels priced from its probe: the probe's GPU time scaled by the pixels,
/// plus `fixed_ms`, the least a probe this session spent outside its passes (setup, the
/// reference's upload, the read-back, the submissions' latency: none of it grows with the
/// pixels). With no GPU timer, the probe's whole time scaled. A stopped probe gives a floor.
pub(crate) fn estimate(p: Probe, cell_px: u64, fixed_ms: f64) -> Estimate {
    let ratio = cell_px as f64 / p.px.max(1) as f64;
    if p.stopped {
        return Estimate { secs: p.wall_ms * ratio / 1000.0, at_least: true };
    }
    let ms = if p.gpu_ms > 0.0 { p.gpu_ms * ratio + fixed_ms.max(0.0) } else { p.wall_ms * ratio };
    Estimate { secs: ms / 1000.0, at_least: false }
}

/// What the scheduler needs to know of a cell.
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub(crate) struct CellStatus {
    pub(crate) estimate: Option<Estimate>,
    pub(crate) rendered: bool,
    pub(crate) failed: bool,
    /// Its Render button was pressed.
    pub(crate) asked: bool,
}

/// The next job: a cell the user asked for first; then a price for every cell, before any is paid
/// for (probes cost a sixty-fourth of their cells); then, in the sheet's order, the cells whose
/// estimate is within `budget_s`. `None`: nothing left to do by itself.
pub(crate) fn next_job(cells: &[CellStatus], budget_s: f64) -> Option<(usize, JobKind)> {
    let open = |c: &CellStatus| !c.rendered && !c.failed;
    if let Some(i) = cells.iter().position(|c| open(c) && c.asked) {
        return Some((i, JobKind::Full));
    }
    if let Some(i) = cells.iter().position(|c| open(c) && c.estimate.is_none()) {
        return Some((i, JobKind::Probe));
    }
    cells
        .iter()
        .position(|c| open(c) && c.estimate.is_some_and(|e| !e.at_least && e.secs <= budget_s))
        .map(|i| (i, JobKind::Full))
}

/// A time as `m:ss`, or `h:mm:ss` from an hour, rounded up (an estimate never reads `0:00`).
pub(crate) fn fmt_eta(secs: f64) -> String {
    let s = (secs.max(0.0).ceil() as u64).max(1);
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

/// The probe's size for a cell of `cell` pixels.
pub(crate) fn probe_size(cell: [u32; 2]) -> [u32; 2] {
    [(cell[0] / PROBE_DIV).max(8), (cell[1] / PROBE_DIV).max(8)]
}

/// What a cell that needs its own calculation shows.
pub(crate) enum CellView {
    Picture(egui::TextureId),
    /// Its calculation is under way, for this long.
    Calculating(Duration),
    /// Not yet priced.
    Estimating,
    /// Priced; `over` the budget, so it waits for its Render button.
    Priced { estimate: Estimate, over: bool },
    /// It could not be calculated here.
    Failed,
}

#[derive(Default)]
struct CellState {
    probe: Option<Probe>,
    tex: Option<egui::TextureHandle>,
    /// Its sRGB texels and size, for the UI test.
    rgba: Option<(Vec<u8>, [u32; 2])>,
    failed: bool,
    asked: bool,
}

struct InFlight {
    cell: SheetCell,
    kind: JobKind,
    key: u64,
    px: u64,
    cancel: Arc<AtomicBool>,
    started: Instant,
    /// Stopped at [`PROBE_LIMIT`] (not by a change of view).
    stopped: bool,
}

struct Job {
    req: Box<fractadyne_gpu::ExportRequest>,
    cancel: Arc<AtomicBool>,
}

enum JobError {
    Canceled,
    /// The sheet's device would not open: nothing can be calculated.
    NoDevice(String),
    Failed(String),
}

struct Rendered {
    size: [u32; 2],
    rgba: Vec<u8>,
    gpu_ms: f64,
}

struct Done {
    result: Result<Rendered, JobError>,
    wall_ms: f64,
}

struct Worker {
    tx: mpsc::Sender<Job>,
    rx: mpsc::Receiver<Done>,
}

/// The sheet's calculated cells, and the thread that calculates them.
#[derive(Default)]
pub(crate) struct SheetRender {
    worker: Option<Worker>,
    /// What the cells hold was calculated for ([`crate::FractadyneApp::sheet_render_key`]), and
    /// since when that has held.
    key: Option<u64>,
    since: Option<Instant>,
    /// The plan's cells that need their own calculation, in its order.
    cells: Vec<(SheetCell, CellState)>,
    in_flight: Option<InFlight>,
    /// The least a probe spent outside its passes this session, ms ([`estimate`]).
    fixed_ms: Option<f64>,
    /// Why no cell can be calculated here, once known.
    broken: Option<String>,
}

impl SheetRender {
    /// Stop: the job under way, the thread (its device closes with it), every calculated cell.
    pub(crate) fn stop(&mut self) {
        if let Some(f) = &self.in_flight {
            f.cancel.store(true, Relaxed);
        }
        *self = Self { fixed_ms: self.fixed_ms, ..Self::default() };
    }

    fn state(&self, c: SheetCell) -> Option<&CellState> {
        self.cells.iter().find(|(x, _)| *x == c).map(|(_, s)| s)
    }

    fn status(&self, s: &CellState, cell_px: u64) -> CellStatus {
        CellStatus {
            estimate: s.probe.map(|p| estimate(p, cell_px, self.fixed_ms.unwrap_or(0.0))),
            rendered: s.tex.is_some(),
            failed: s.failed,
            asked: s.asked,
        }
    }

    /// What cell `c` shows, for a cell of `cell_px` pixels against `budget_s`.
    pub(crate) fn view(&self, c: SheetCell, cell_px: u64, budget_s: f64) -> CellView {
        if let Some(f) = self.in_flight.as_ref().filter(|f| f.cell == c && f.kind == JobKind::Full && Some(f.key) == self.key) {
            return CellView::Calculating(f.started.elapsed());
        }
        let Some(s) = self.state(c) else { return CellView::Estimating };
        if let Some(t) = &s.tex {
            return CellView::Picture(t.id());
        }
        if s.failed || self.broken.is_some() {
            return CellView::Failed;
        }
        match self.status(s, cell_px).estimate {
            Some(e) => CellView::Priced { estimate: e, over: e.at_least || e.secs > budget_s },
            None => CellView::Estimating,
        }
    }

    /// Calculate cell `c` next, whatever its estimate (its Render button).
    pub(crate) fn ask(&mut self, c: SheetCell) {
        if let Some((_, s)) = self.cells.iter_mut().find(|(x, _)| *x == c) {
            s.asked = true;
        }
    }

    /// Whether the cell under way is `c`, asked for (a Stop button stops it).
    pub(crate) fn asked_in_flight(&self, c: SheetCell) -> bool {
        self.in_flight.as_ref().is_some_and(|f| f.cell == c && f.kind == JobKind::Full)
            && self.state(c).is_some_and(|s| s.asked)
    }

    /// Stop cell `c`'s calculation, and un-ask it.
    pub(crate) fn stop_cell(&mut self, c: SheetCell) {
        if let Some(f) = self.in_flight.as_ref().filter(|f| f.cell == c) {
            f.cancel.store(true, Relaxed);
        }
        if let Some((_, s)) = self.cells.iter_mut().find(|(x, _)| *x == c) {
            s.asked = false;
        }
    }

    /// Cell `c`'s texels and size, when calculated (the UI test).
    pub(crate) fn rgba(&self, c: SheetCell) -> Option<&(Vec<u8>, [u32; 2])> {
        self.state(c)?.rgba.as_ref()
    }

    /// Whether a job is under way, or one would start by itself (the UI test waits for neither).
    pub(crate) fn busy(&self, cell_px: u64, budget_s: f64) -> bool {
        if self.in_flight.is_some() {
            return true;
        }
        if self.broken.is_some() {
            return false;
        }
        let statuses: Vec<CellStatus> = self.cells.iter().map(|(_, s)| self.status(s, cell_px)).collect();
        next_job(&statuses, budget_s).is_some()
    }

    /// Whether every one of the cells has a picture.
    pub(crate) fn all_rendered(&self) -> bool {
        self.cells.iter().all(|(_, s)| s.tex.is_some())
    }

    /// How many cells have a picture, and how many are priced.
    pub(crate) fn counts(&self) -> (usize, usize) {
        (self.cells.iter().filter(|(_, s)| s.tex.is_some()).count(), self.cells.iter().filter(|(_, s)| s.probe.is_some()).count())
    }

    /// A new key: the job under way stops (its answer is dropped when it comes), and what was
    /// calculated for the old one goes.
    fn rekey(&mut self, key: u64, now: Instant) {
        if let Some(f) = &self.in_flight {
            f.cancel.store(true, Relaxed);
        }
        self.key = Some(key);
        self.since = Some(now);
        for (_, s) in &mut self.cells {
            *s = CellState { asked: false, ..CellState::default() };
        }
    }

    /// Keep the cells the plan still wants, in its order; new ones start empty.
    fn sync_cells(&mut self, wanted: &[SheetCell]) {
        if self.cells.len() == wanted.len() && self.cells.iter().zip(wanted).all(|((a, _), b)| a == b) {
            return;
        }
        let mut old = std::mem::take(&mut self.cells);
        for c in wanted {
            let s = old.iter().position(|(x, _)| x == c).map(|i| old.swap_remove(i).1).unwrap_or_default();
            self.cells.push((*c, s));
        }
    }

    /// Take a finished job's answer.
    fn collect(&mut self, ctx: &egui::Context) {
        let done = match self.worker.as_ref().map(|w| w.rx.try_recv()) {
            Some(Ok(d)) => d,
            // A thread that is gone with a job under way will never answer.
            Some(Err(mpsc::TryRecvError::Disconnected)) if self.in_flight.is_some() => {
                self.in_flight = None;
                self.broken = Some("the sheet's thread stopped".into());
                return;
            }
            _ => return,
        };
        let Some(f) = self.in_flight.take() else { return };
        if Some(f.key) != self.key {
            return; // calculated for a view that is gone
        }
        let label = cell_label(f.cell);
        let fixed = self.fixed_ms;
        let Some((_, s)) = self.cells.iter_mut().find(|(x, _)| *x == f.cell) else { return };
        match (f.kind, done.result) {
            (JobKind::Probe, Ok(r)) => {
                let p = Probe { gpu_ms: r.gpu_ms, wall_ms: done.wall_ms, px: f.px, stopped: false };
                s.probe = Some(p);
                if r.gpu_ms > 0.0 {
                    let outside = (done.wall_ms - r.gpu_ms).max(0.0);
                    self.fixed_ms = Some(fixed.map_or(outside, |m| m.min(outside)));
                }
                crate::diag::log_line(
                    "method-sheet",
                    &format!("{label}: probe {}x{} gpu {:.1} ms, wall {:.1} ms", r.size[0], r.size[1], r.gpu_ms, done.wall_ms),
                );
            }
            (JobKind::Probe, Err(JobError::Canceled)) if f.stopped => {
                s.probe = Some(Probe { gpu_ms: 0.0, wall_ms: done.wall_ms, px: f.px, stopped: true });
                crate::diag::log_line("method-sheet", &format!("{label}: probe stopped at {:.0} ms", done.wall_ms));
            }
            (JobKind::Full, Ok(r)) => {
                let img = egui::ColorImage::from_rgba_unmultiplied([r.size[0] as usize, r.size[1] as usize], &r.rgba);
                s.tex = Some(ctx.load_texture(format!("fractadyne.method-sheet.{label}"), img, egui::TextureOptions::LINEAR));
                s.rgba = Some((r.rgba, r.size));
                crate::diag::log_line(
                    "method-sheet",
                    &format!("{label}: calculated {}x{} gpu {:.1} ms, wall {:.1} ms", r.size[0], r.size[1], r.gpu_ms, done.wall_ms),
                );
            }
            (_, Err(JobError::Canceled)) => {}
            (_, Err(JobError::NoDevice(e))) => {
                crate::diag::log_line("method-sheet", &format!("no cell can be calculated: {e}"));
                self.broken = Some(e);
            }
            (_, Err(JobError::Failed(e))) => {
                s.failed = true;
                crate::diag::log_line("method-sheet", &format!("{label}: not calculated: {e}"));
            }
        }
    }

    fn send(&mut self, card: &eframe::wgpu::AdapterInfo, ctx: &egui::Context, job: InFlight, req: fractadyne_gpu::ExportRequest) {
        let worker = self.worker.get_or_insert_with(|| spawn_worker(card.clone(), ctx.clone()));
        if worker.tx.send(Job { req: Box::new(req), cancel: job.cancel.clone() }).is_err() {
            self.broken = Some("the sheet's thread did not start".into());
            return;
        }
        self.in_flight = Some(job);
    }
}

/// A cell, as the log names it.
fn cell_label(c: SheetCell) -> String {
    match (c.trap, c.stripe_freq) {
        (Some(t), _) => format!("{:?}/{:?}", c.method, t),
        (_, Some(f)) => format!("{:?}/{f}", c.method),
        _ => format!("{:?}", c.method),
    }
}

/// The thread, and on it the device it opens on the window's card (`gpu_worker::open_headless`,
/// as the Export dialog's renders do). It answers each job in turn, and ends when the sheet drops
/// its end of the channel.
fn spawn_worker(card: eframe::wgpu::AdapterInfo, ctx: egui::Context) -> Worker {
    let (tx, jobs) = mpsc::channel::<Job>();
    let (done_tx, rx) = mpsc::channel::<Done>();
    let spawned = std::thread::Builder::new().name("fd-method-sheet".into()).spawn(move || {
        let own = crate::gpu_worker::open_headless("same", &card, crate::gpu_choice::backends(), "fractadyne.method-sheet");
        for job in jobs {
            let t0 = Instant::now();
            let result = match &own {
                Err(e) => Err(JobError::NoDevice(e.clone())),
                Ok(_) if job.cancel.load(Relaxed) => Err(JobError::Canceled),
                Ok(h) => {
                    let progress = AtomicU32::new(0);
                    match fractadyne_gpu::render_export(&h.device, &h.queue, &job.req, &progress, &job.cancel) {
                        Ok(r) => Ok(Rendered {
                            size: [r.width, r.height],
                            rgba: fractadyne_export::to_srgb8(&r.pixels),
                            gpu_ms: r.iterate_ms + r.color_ms,
                        }),
                        Err(fractadyne_gpu::GpuError::Canceled) => Err(JobError::Canceled),
                        Err(e) => Err(JobError::Failed(e.to_string())),
                    }
                }
            };
            if done_tx.send(Done { result, wall_ms: t0.elapsed().as_secs_f64() * 1000.0 }).is_err() {
                break;
            }
            ctx.request_repaint();
        }
    });
    if let Err(e) = spawned {
        crate::diag::log_line("method-sheet", &format!("the sheet's thread did not start: {e}"));
    }
    Worker { tx, rx }
}

impl crate::FractadyneApp {
    /// The size, in texels, of one cell of the method sheet (its atlas's and a calculated one's).
    pub(crate) fn sheet_cell_size(&self) -> [u32; 2] {
        let aspect = (self.viewport.height_px / self.viewport.width_px.max(1.0)).clamp(0.2, 5.0);
        [super::method_sheet::CELL_W, ((super::method_sheet::CELL_W as f64) * aspect).round().max(8.0) as u32]
    }

    /// What the sheet's calculated cells depend on: the view, its iterations, the reference they
    /// borrow, the coloring but for what a cell sets for itself (the method, the trap's shape, the
    /// stripe density), and the size. ⭐Not the method the view is colored with: a hover preview
    /// changes that and none of these pictures.
    pub(crate) fn sheet_render_key(&self, size: [u32; 2]) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        let julia = self.julia_mode && !self.dual;
        fractadyne_core::to_decimal_string(&self.viewport.center_x).hash(&mut h);
        fractadyne_core::to_decimal_string(&self.viewport.center_y).hash(&mut h);
        self.viewport.units_per_pixel.log2().to_bits().hash(&mut h);
        self.fractal.name().hash(&mut h);
        self.custom_formula_metadata().hash(&mut h);
        julia.hash(&mut h);
        self.dual.hash(&mut h);
        self.julia_c.0.to_bits().hash(&mut h);
        self.julia_c.1.to_bits().hash(&mut h);
        self.render_cfg.max_iter.hash(&mut h);
        self.render_cfg.auto_iter.hash(&mut h);
        self.export_eff_iter(&self.viewport, julia).hash(&mut h);
        self.ref_cache[0].orbit_id.hash(&mut h);
        let mut spec = self.coloring_spec();
        spec.color_method.clear();
        spec.trap_type.clear();
        spec.stripe_freq = 0.0;
        format!("{spec:?}").hash(&mut h);
        self.coloring.palette_rev.hash(&mut h);
        self.effects.de_phase.to_bits().hash(&mut h);
        size.hash(&mut h);
        h.finish()
    }

    /// The view on screen at `size` pixels: the same center, the same vertical span.
    pub(crate) fn sheet_cell_viewport(&self, size: [u32; 2]) -> fractadyne_core::Viewport {
        let mut vp = self.viewport.clone();
        vp.set_size(size[0] as f64, size[1] as f64);
        vp.units_per_pixel = self.viewport.units_per_pixel * fractadyne_core::FloatExp::from_f64(self.viewport.height_px / size[1].max(1) as f64);
        vp
    }

    /// The export request for cell `c` at `size`: the request an export of the view in the cell's
    /// coloring makes, the builder reading the cell's method, shape and density as the coloring's
    /// (so what it accumulates, and whether the live tree's skipping still serves it, follow the
    /// cell), borrowing the live reference. The cell's own palette rate and no range mapping, as
    /// the sheet's recolored cells.
    fn sheet_cell_request(&mut self, c: SheetCell, size: [u32; 2]) -> fractadyne_gpu::ExportRequest {
        let julia = self.julia_mode && !self.dual;
        let vp = self.sheet_cell_viewport(size);
        let saved = (self.coloring.color_method, self.coloring.trap_type, self.coloring.stripe_freq);
        self.coloring.color_method = c.method;
        if let Some(t) = c.trap {
            self.coloring.trap_type = t;
        }
        if let Some(f) = c.stripe_freq {
            self.coloring.stripe_freq = f;
        }
        let mut req = self.autopilot_probe_request(&vp, julia);
        (self.coloring.color_method, self.coloring.trap_type, self.coloring.stripe_freq) = saved;
        req.width = size[0];
        req.height = size[1];
        req.ss = 1;
        req.cycle = self.color_cycle_for(c.method);
        req.offset = self.coloring.offset;
        req.norm_mode = 0;
        req.norm_lo = 0.0;
        req
    }

    /// The budget a cell must fit to be calculated by itself.
    pub(crate) fn sheet_budget_s(&self) -> f64 {
        self.harness.uitest_sheet_budget.unwrap_or(AUTO_BUDGET_S)
    }

    /// Each frame the sheet is open: take a finished cell, and start the next one when the view
    /// has held still, the live view is at rest and its reference serves.
    pub(crate) fn sheet_render_step(&mut self, ctx: &egui::Context, plan: &[(MethodGroup, Vec<(SheetCell, CellSource)>)]) {
        let Some(card) = self.render_state.as_ref().map(|rs| rs.adapter.get_info()) else { return };
        let size = self.sheet_cell_size();
        let key = self.sheet_render_key(size);
        let now = Instant::now();
        let wanted: Vec<SheetCell> =
            plan.iter().flat_map(|(_, cs)| cs.iter()).filter(|(_, s)| *s == CellSource::Render).map(|(c, _)| *c).collect();
        let budget = self.sheet_budget_s();
        let cell_px = size[0] as u64 * size[1] as u64;
        let hold = self.harness.uitest_sheet_hold;
        {
            let r = &mut self.method_sheet.render;
            if r.key != Some(key) {
                r.rekey(key, now);
            }
            r.sync_cells(&wanted);
            if !hold {
                r.collect(ctx);
            }
            if let Some(f) = r.in_flight.as_mut() {
                if f.kind == JobKind::Probe && !f.stopped && f.started.elapsed() >= PROBE_LIMIT {
                    f.stopped = true;
                    f.cancel.store(true, Relaxed);
                }
            }
        }
        let julia = self.julia_mode && !self.dual;
        let r = &self.method_sheet.render;
        let still = r.since.is_some_and(|t| now.duration_since(t) >= SETTLE);
        let at_rest = self.recompute_rx[0].is_none() && !self.perf.tile_pending[0] && !self.perf.chunk_pending[0];
        if r.in_flight.is_none() && r.broken.is_none() && still && at_rest {
            let statuses: Vec<CellStatus> = r.cells.iter().map(|(_, s)| r.status(s, cell_px)).collect();
            if let Some((i, kind)) = next_job(&statuses, budget) {
                let cell = r.cells[i].0;
                let jsize = if kind == JobKind::Probe { probe_size(size) } else { size };
                if self.live_reference_serves(&self.sheet_cell_viewport(jsize), julia) {
                    let req = self.sheet_cell_request(cell, jsize);
                    let job = InFlight {
                        cell,
                        kind,
                        key,
                        px: jsize[0] as u64 * jsize[1] as u64,
                        cancel: Arc::new(AtomicBool::new(false)),
                        started: now,
                        stopped: false,
                    };
                    self.method_sheet.render.send(&card, ctx, job, req);
                }
            }
        }
        if self.method_sheet.render.busy(cell_px, budget) {
            ctx.request_repaint_after(POLL);
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/ui/sheet_render/tests.rs"]
mod tests;
