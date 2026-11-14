//! ⭐THE START-UP SPEED PROBE (`STARTUP_PROBE`, 2026-11-10): on a card the calibration table has no
//! entry for, the first frame times how long one nominal step takes there, in each precision mode,
//! before anything is sized from a guess.
//!
//! **Why.** Two numbers in the live view are guesses until the card is measured, and both were
//! guessed on fast discrete cards. On Intel HD Graphics 4400 (Eris, OpenGL, `--zoomtest 3` at 2^103,
//! 2026-11-10) both were badly wrong:
//!
//! - The OPENING budget (`TDR_BOOTSTRAP_STEPS`, 4e8 nominal steps, `pricing::bootstrap_steps` case 3).
//!   The first frame's 1.9e8 floatexp steps took **1354 ms** against a 2 s watchdog; the emergency
//!   retreat caught it after the fact. A view asking the whole 4e8 would have taken ~2.8 s.
//! - The DISPATCH CEILING (`calibration`): an unlisted card gets the table default, twice the RX
//!   6800 XT's cost per step. Eris's direct mode measured ~1000× the RTX 3080's, so a dispatch the
//!   default allowed could run ~100 s there.
//!
//! **What it does.** One small ALL-INTERIOR view per mode (direct, df32 perturbation, floatexp), BLA,
//! the series skip and the renormalized step off, the exact-repeat check off (`render_iter_probe`):
//! every pixel runs every step, the worst case the ceiling is defined at (`calibration`'s module
//! header). The passes go through the live renderer's own pipelines, so nothing compiles (a compile
//! of the iterate shader is 12–26 s on that card). Each mode ramps from a pass far too small to
//! matter on any card up to one of [`PASS_TARGET_MS`], at most ×16 in iterations a step. Then:
//!
//! - the mode's measured rate opens its budget (`Meter::record_opening_rate`, the opening only:
//!   the first reading of a real frame replaces it), and
//! - direct and df32's costs lower the ceiling where they are slower than the table's
//!   (`calibration::slower_of`).
//!
//! **What it cannot do: raise anything.** A mode that measures faster than every number it could
//! change stops early ([`Finding::Fast`]) and records nothing, so a fast card behaves exactly as
//! before, and a card WITH a table entry is not probed at all (`STARTUP_PROBE=2` probes it as an
//! experiment, its own entry the base). That is also how this squares with MEM-16, "startup
//! calibration is for capability discovery and a safe starting point, never the cost model"
//! (design/live-render-robustness.md §5.2): the worst case is a bound for safety, and the learned
//! budget never reads it.
//!
//! **Saved, then re-checked** ([`saved`]): a reading is kept per card, driver, build and power
//! state. A later launch in the same state uses it at once and measures the first frame's mode
//! again LATER, at the iterations it was saved at, once the picture is up and the view is still:
//! on a thread of its own, or on OpenGL on the UI thread ([`recheck`]); a reading that no longer
//! holds is measured again there and replaced. On Eris that takes the check off every launch but the first in a
//! build and power state.

mod recheck;
mod saved;

pub(crate) use recheck::{Recheck, SpeedNotice, NOTICE_FRAMES};

use super::{assemble_ref_fields, pre_switch_probe_representative, recompute_worker, RecomputeInputs};
use crate::calibration::Costs;
use crate::power_state::PowerState;
use crate::{FractadyneApp, FractalKind, RenderMode};
use fractadyne_core::Viewport;
use std::time::Instant;

/// Pixels on a side of every probe pass. 512² = 262,144 px, the RTX 3080's occupancy knee, so the
/// pass is not an under-filled one on a big card (a 16k-px pass costs up to 2.6× more per step
/// there, `calibration`); on Intel HD Graphics 4400, one floatexp step a pixel is ~1.8 ms.
pub(crate) const SIDE: u32 = 512;
/// The first pass's iterations: ~29 ms of floatexp on Intel HD Graphics 4400, the slowest card
/// measured, and a fraction of a millisecond on a discrete one.
pub(crate) const ITERS_START: u32 = 16;
/// The most iterations a probe pass asks, and the length of the probe's references. A card exactly
/// at the lowest threshold that matters (direct's table default, 1.92e-8 ms/step) reaches
/// [`PASS_TARGET_MS`] here (~21 ms); anything faster reads as [`Finding::Fast`] on the way. (It was
/// 65,536: on Intel HD Graphics 4400 the floatexp reference alone was ~0.5 s of a 7.3 s probe.)
pub(crate) const ITERS_MAX: u32 = 4096;
/// A pass this long, by the GPU's own clock, is timed well enough to price: the per-pass fixed cost
/// (~0.2 ms) is a few percent of it, and errs slow, the safe side.
pub(crate) const PASS_TARGET_MS: f64 = 12.0;
/// The ramp's guard against a GPU clock that reads low: the next pass, scaled from this one's WALL
/// time (submission to completion), stays under this. A wall carries the submission's latency, so
/// it over-predicts and the ramp only goes slower; where it cannot even double, the ramp stops.
pub(crate) const PASS_WALL_MAX_MS: f64 = 50.0;
/// Timed passes per ramp step (after one untimed warm-up, `render_iter_probe`); the slowest counts.
pub(crate) const REPS: u32 = 2;

/// The modes probed, in order.
const MODES: [RenderMode; 3] = [RenderMode::Direct, RenderMode::Df32Pert, RenderMode::Floatexp];

/// The probe's view: inside the main cardioid (multiplier |λ| ≈ 0.25), so every pixel of a small
/// view is interior and bounded, and with the exact-repeat check off runs every step it is given.
/// The same point as `validation/calibration/interior-*.fdn`.
const CENTER: (&str, &str) = ("-0.1", "0.1");

/// One mode's view depth, log2 of the units per pixel: direct shallow, df32 perturbation well inside
/// its exponent range, floatexp past it (δ ≈ 2^-190 cannot be held in f32, so the df32 tail phase,
/// which would price df32's arithmetic instead of floatexp's, never engages).
fn upp_log2(mode: RenderMode) -> i32 {
    match mode {
        RenderMode::Direct => -24,
        RenderMode::Df32Pert => -60,
        RenderMode::Floatexp => -200,
    }
}

/// What one mode's probe found.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Finding {
    /// The slowest timed pass at `iters` took `ms` on the GPU ([`gpu_ms`]; `wall_ms` submission to
    /// completion), at least [`PASS_TARGET_MS`], or the ramp could go no further: one nominal step
    /// costs `ms_per_step` here, at worst.
    Measured { iters: u32, ms: f64, wall_ms: f64, ms_per_step: f64 },
    /// At `iters` the pass took `ms` on the GPU, so a step costs at most `ms_per_step` (the pass's
    /// fixed cost is in it): below every cost that would change an opening or a ceiling. Nothing is
    /// recorded.
    Fast { iters: u32, ms: f64, ms_per_step: f64 },
}

impl Finding {
    /// The cost per step it found: measured, or the upper bound a fast pass gave.
    pub(crate) fn ms_per_step(&self) -> f64 {
        match *self {
            Finding::Measured { ms_per_step, .. } | Finding::Fast { ms_per_step, .. } => ms_per_step,
        }
    }
}

/// Tell the export's tile sizing what a step costs on this card (`fractadyne_gpu::set_card_step_ms`):
/// direct and df32 at the dispatch ceiling's costs (`calibration::costs_in_force`, the table's or
/// lower where this card measured slower), floatexp at `floatexp` (the speed check's reading;
/// `None` leaves it unknown, as on a card with a table entry, which is not checked).
pub(crate) fn tell_export_card_costs(floatexp: Option<Finding>) {
    if let Some(c) = crate::calibration::costs_in_force() {
        fractadyne_gpu::set_card_step_ms(RenderMode::Direct.to_u32(), c.direct);
        fractadyne_gpu::set_card_step_ms(RenderMode::Df32Pert.to_u32(), c.df32_pert);
    }
    if let Some(f) = floatexp {
        fractadyne_gpu::set_card_step_ms(RenderMode::Floatexp.to_u32(), f.ms_per_step());
    }
}

/// The ramp's next move after a timed pass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Step {
    Done(Finding),
    /// Run the next pass at this many iterations.
    Next(u32),
}

/// The ramp, after a pass of `px` pixels at `iters` iterations whose slowest timing was `gpu_ms`
/// by the GPU's clock ([`gpu_ms`]) and `wall_ms` submission to completion. `fast_below` is the cost
/// per step under which nothing would change ([`fast_below`]).
///
/// The next pass aims at twice [`PASS_TARGET_MS`] of GPU time from this one's cost per iteration,
/// which carries the pass's fixed cost and so over-estimates; at least 2× the iterations, at most
/// 16×, and no more than [`PASS_WALL_MAX_MS`] scaled from this pass's wall. Where that wall bound
/// cannot double the pass, the ramp stops and prices what it has. `None` = a timing is unusable.
pub(crate) fn step(px: u64, iters: u32, gpu_ms: f64, wall_ms: f64, fast_below: f64) -> Option<Step> {
    if !(gpu_ms.is_finite() && gpu_ms > 0.0 && wall_ms.is_finite() && wall_ms > 0.0) || px == 0 || iters == 0 {
        return None;
    }
    let ms_per_step = gpu_ms / (px as f64 * f64::from(iters));
    let measured = Step::Done(Finding::Measured { iters, ms: gpu_ms, wall_ms, ms_per_step });
    if gpu_ms >= PASS_TARGET_MS {
        return Some(measured);
    }
    if ms_per_step < fast_below {
        return Some(Step::Done(Finding::Fast { iters, ms: gpu_ms, ms_per_step }));
    }
    if iters >= ITERS_MAX {
        return Some(measured);
    }
    let i = f64::from(iters);
    let wall_cap = (i * PASS_WALL_MAX_MS / wall_ms).floor();
    if wall_cap < i * 2.0 {
        return Some(measured);
    }
    let aim = i * 2.0 * PASS_TARGET_MS / gpu_ms;
    Some(Step::Next(aim.clamp(i * 2.0, i * 16.0).min(wall_cap).min(f64::from(ITERS_MAX)) as u32))
}

/// The cost per step below which a mode's measurement changes nothing: its opening is already the
/// whole `bootstrap_steps` (`rate × bootstrap_ms ≥ bootstrap_steps`), and its ceiling's table cost
/// is not exceeded (`table_cost`, 0 = no ceiling in that mode).
pub(crate) fn fast_below(table_cost: f64, bootstrap_ms: f64, bootstrap_steps: u64) -> f64 {
    let opening = if bootstrap_steps > 0 { bootstrap_ms / bootstrap_steps as f64 } else { 0.0 };
    if table_cost > 0.0 && table_cost.is_finite() { opening.min(table_cost) } else { opening }
}

/// A timed pass's GPU time: its timestamp, where the card gave one that fits inside the pass's wall
/// (submission to completion, plus 0.5 ms for the clocks' resolution), else the wall.
///
/// ⚠Not the slower of the two, which this was first: the wall carries the submission's latency,
/// and PLUTO's RTX 3070 (2026-11-10, dev.8) waited ~12 ms per submission, so a pass of ~1 ms of
/// GPU work read 13.9 ms and the card measured ~20× slower than it is (direct 9.7e-8 ms/step; the
/// RTX 3080 here is 4.9e-9), lowering its ceiling 3–5× for nothing. A watchdog times the GPU's
/// work, which is what the timestamp measures. A timestamp that reads LOW cannot make a long pass
/// (the ramp's wall bound, [`PASS_WALL_MAX_MS`]); at worst it leaves a card on the defaults, as
/// before the probe.
pub(crate) fn gpu_ms([ts, wall]: [f64; 2]) -> f64 {
    if ts.is_finite() && ts > 0.0 && ts <= wall + 0.5 { ts } else { wall }
}

/// The table's cost for `mode` (0 for floatexp, which has no ceiling).
fn table_cost(costs: Costs, mode: RenderMode) -> f64 {
    match mode {
        RenderMode::Direct => costs.direct,
        RenderMode::Df32Pert => costs.df32_pert,
        RenderMode::Floatexp => 0.0,
    }
}

fn mode_name(mode: RenderMode) -> &'static str {
    match mode {
        RenderMode::Direct => "direct",
        RenderMode::Df32Pert => "df32",
        RenderMode::Floatexp => "floatexp",
    }
}

/// The local time, `2026-11-10 14:03`, for a saved reading's reader ("" where it cannot be read).
fn taken_now() -> String {
    crate::sysinfo::local_now().map_or_else(String::new, |(y, mo, d, h, mi, ..)| format!("{y}-{mo:02}-{d:02} {h:02}:{mi:02}"))
}

/// What a probe pass in one mode needs from the app, taken on the UI thread
/// ([`FractadyneApp::speed_probe_plan`]): the view, and its reference's inputs (none in direct
/// mode). The reference itself is built where the passes run ([`plan_request`]).
pub(crate) struct Plan {
    mode: RenderMode,
    vp: Viewport,
    inputs: Option<RecomputeInputs>,
}

/// One timed probe pass through `live` (warm-up, then [`REPS`] timed: `render_iter_probe`).
fn timed_pass(
    dev: &eframe::wgpu::Device,
    q: &eframe::wgpu::Queue,
    req: &fractadyne_gpu::ExportRequest,
    live: &fractadyne_gpu::LiveIterPipeline,
) -> Result<(fractadyne_gpu::ExportResult, Vec<[f64; 2]>), String> {
    fractadyne_gpu::render_iter_probe(dev, q, req, REPS, live).map_err(|e| e.to_string())
}

/// One mode's ramp (see the module header) through `live`, from a request [`plan_request`] built
/// (with the most iterations its reference supports).
fn ramp(
    dev: &eframe::wgpu::Device,
    q: &eframe::wgpu::Queue,
    live: &fractadyne_gpu::LiveIterPipeline,
    (req, iters_cap): &mut (fractadyne_gpu::ExportRequest, u32),
    fast_below: f64,
) -> Result<Finding, String> {
    let iters_cap = *iters_cap;
    let px = u64::from(SIDE) * u64::from(SIDE);
    let mut iters = ITERS_START.min(iters_cap);
    loop {
        req.max_iter = iters;
        let (r, reps) = timed_pass(dev, q, req, live)?;
        let (running, ok) = pre_switch_probe_representative(&r.pixels, SIDE, SIDE);
        if !ok {
            return Err(format!("{:.1}% of pixels ran every step at {iters} (need 98%)", running * 100.0));
        }
        let gpu = reps.iter().map(|&t| gpu_ms(t)).fold(f64::NAN, f64::max);
        let wall = reps.iter().map(|&[_, w]| w).fold(f64::NAN, f64::max);
        match step(px, iters, gpu, wall, fast_below) {
            None => return Err(format!("unusable times {reps:?} ms at {iters}")),
            Some(Step::Done(f)) => return Ok(f),
            Some(Step::Next(n)) if n > iters && n <= iters_cap => iters = n,
            Some(Step::Next(_)) => {
                // The reference stopped short (`partial`): price what was timed.
                let ms_per_step = gpu / (px as f64 * f64::from(iters));
                return Ok(Finding::Measured { iters, ms: gpu, wall_ms: wall, ms_per_step });
            }
        }
    }
}

/// A saved finding's mode measured again at the iterations it was found at: one probe pass, its
/// cost per step by the GPU's clock ([`gpu_ms`]).
fn remeasure_at(
    dev: &eframe::wgpu::Device,
    q: &eframe::wgpu::Queue,
    live: &fractadyne_gpu::LiveIterPipeline,
    (req, iters_cap): &mut (fractadyne_gpu::ExportRequest, u32),
    saved: Finding,
) -> Result<f64, String> {
    let iters = match saved {
        Finding::Measured { iters, .. } | Finding::Fast { iters, .. } => iters,
    };
    if iters > *iters_cap {
        return Err(format!("the reference reaches {iters_cap} iterations, not the saved {iters}"));
    }
    req.max_iter = iters;
    let (r, reps) = timed_pass(dev, q, req, live)?;
    let (running, ok) = pre_switch_probe_representative(&r.pixels, SIDE, SIDE);
    if !ok {
        return Err(format!("{:.1}% of pixels ran every step at {iters} (need 98%)", running * 100.0));
    }
    let gpu = reps.iter().map(|&t| gpu_ms(t)).fold(f64::NAN, f64::max);
    let px = f64::from(SIDE) * f64::from(SIDE);
    if !(gpu.is_finite() && gpu > 0.0) {
        return Err(format!("unusable times {reps:?} ms at {iters}"));
    }
    Ok(gpu / (px * f64::from(iters)))
}

/// The log's words for one mode's finding (`took`: how long it took, or `saved`), and whether it
/// opens the mode's budget (`opens`, [`speed_probe_opens`]).
fn finding_part(mode: RenderMode, found: &Result<Finding, String>, took: &str, opens: bool) -> String {
    match found {
        Ok(Finding::Measured { iters, ms, wall_ms, ms_per_step }) => format!(
            "{} {ms_per_step:.3e} ms/step ({SIDE}² × {iters}: {ms:.1} ms GPU, {wall_ms:.1} wall; {took}){}",
            mode_name(mode),
            if opens {
                format!(", opens at {:.2e} steps", super::bootstrap_steps(1.0 / ms_per_step, 0.0) as f64)
            } else {
                String::new()
            }
        ),
        Ok(Finding::Fast { iters, ms, ms_per_step }) => format!(
            "{} under {ms_per_step:.2e} ms/step ({SIDE}² × {iters}: {ms:.2} ms GPU; {took}), faster than the defaults",
            mode_name(mode)
        ),
        Err(e) => format!("{} not measured ({e}; {took})", mode_name(mode)),
    }
}

/// A measured finding's cost per step where it opens its mode's budget: only where that comes down
/// from the default (`opening_floor`, [`fast_below`] with no ceiling term). A faster reading would
/// mark the mode measured and change nothing else, e.g. stop the pre-switch probe.
fn speed_probe_opens(found: &Result<Finding, String>, opening_floor: f64) -> Option<f64> {
    match found {
        Ok(Finding::Measured { ms_per_step, .. }) if *ms_per_step > opening_floor => Some(*ms_per_step),
        _ => None,
    }
}

/// The costs a set of findings lowers the ceiling to (0 = leave the table's): direct's and df32's
/// measured costs; floatexp has no ceiling.
fn lowered_costs(found: &[Result<Finding, String>; 3]) -> Costs {
    let mut lowered = Costs::default();
    for (mode, f) in MODES.into_iter().zip(found) {
        if let Ok(Finding::Measured { ms_per_step, .. }) = f {
            match mode {
                RenderMode::Direct => lowered.direct = *ms_per_step,
                RenderMode::Df32Pert => lowered.df32_pert = *ms_per_step,
                RenderMode::Floatexp => {}
            }
        }
    }
    lowered
}

/// The probe's request for `plan`'s mode at the cardioid point, and the most iterations its
/// reference supports: the reference is built here, on the caller's thread. Mandelbrot whatever the
/// session shows, plain smooth coloring, nothing that skips steps: BLA, the series skip and the
/// renormalized step are off.
fn plan_request(plan: Plan) -> (fractadyne_gpu::ExportRequest, u32) {
    let Plan { mode, vp, inputs } = plan;
    let scale = vp.gpu_scale();
    let mut iters_cap = ITERS_MAX;
    let fields = match inputs {
        None => super::RefFields::default(),
        Some(inputs) => {
            let res = recompute_worker(inputs);
            if res.partial {
                iters_cap = res.orbit_len.saturating_sub(1).max(1);
            }
            assemble_ref_fields(&vp, vp.precision, scale.delta_exp, res)
        }
    };
    let (cx, cy) = vp.center_f64();
    let (cxh, cyh) = (cx as f32, cy as f32);
    let none = fractadyne_core::SeriesSkip::NONE;
    let req = fractadyne_gpu::ExportRequest {
        width: SIDE,
        height: SIDE,
        ss: 1,
        span_mantissa: scale.span_mantissa,
        center: [cxh, cyh, (cx - f64::from(cxh)) as f32, (cy - f64::from(cyh)) as f32],
        ref_offset: fields.ref_offset,
        delta_exp: scale.delta_exp,
        sa_skip: 0,
        glitch_on: 0,
        vignette: Default::default(),
        sa_a: none.a,
        sa_a_exp: none.a_exp,
        sa_b: none.b,
        sa_b_exp: none.b_exp,
        sa_c: none.c,
        sa_c_exp: none.c_exp,
        rn: fractadyne_gpu::Renorm::default(),
        rn_bla: std::sync::Arc::new(Vec::new()),
        map: fractadyne_gpu::StripMap::default(),
        julia_c: [0.0; 4],
        orbit: fields.orbit,
        orbit_len: fields.orbit_len,
        bla: std::sync::Arc::new(Vec::new()),
        bla_on: 0,
        max_iter: ITERS_START,
        mode: mode.to_u32(),
        formula: FractalKind::Mandelbrot.formula_id(),
        custom: None,
        julia: 0,
        cycle: 1.0,
        offset: 0.0,
        norm_mode: 0,
        norm_lo: 0.0,
        palette_fade: fractadyne_gpu::PaletteFade::default(),
        aa_palette: fractadyne_gpu::PaletteAa::Off,
        aa_sample: 0,
        lut: std::sync::Arc::new(Vec::new()),
        lut_smooth: false,
        light: 0,
        light_angle: 0.0,
        light_height: 0.0,
        de_on: 0,
        de_strength: 0.0,
        de_width: 0.0,
        de_phase: 0.0,
        color_method: crate::ColorMethod::Smooth.gpu_id(),
        orbit_stats: crate::StatSet::NONE,
        stripe_freq: 0.0,
        stripe_tail: false,
        stripe_tail_len: 0,
        trap_type: crate::TrapType::Point.gpu_id(),
        aa_filter: 1,
        interior_col: [0.0; 4],
        interior: Default::default(),
        work_budget: None,
        pass_ms: None,
        jitter: [0.0, 0.0],
        tile_px_max: None,
    };
    (req, iters_cap)
}

impl FractadyneApp {
    /// Run the probe if this card gets one (`STARTUP_PROBE`), record each slow mode's opening rate,
    /// log what it found, and return the costs the ceiling should be lowered to (0 = leave the
    /// table's). `None` = not probed. Called once, on the first frame that has a device, before the
    /// calibration is resolved (`calibration::init_measured`). With `STARTUP_PROBE=1` a saved
    /// reading for this card and power state is used at once and re-checked in the background
    /// ([`recheck`]); 2 always measures.
    pub(crate) fn startup_speed_probe(&mut self, rs: &eframe::egui_wgpu::RenderState) -> Option<Costs> {
        let setting = crate::tunables::cost().startup_probe;
        let table = crate::calibration::table_entry(&self.gpu_name)?;
        if setting == 0 || (setting == 1 && table.own_entry) || self.render_cli.run {
            return None;
        }
        let t0 = Instant::now();
        crate::diag::breadcrumb("start-up: the speed probe (one small pass per precision mode)".into());
        let c = crate::tunables::cost();
        let opening_floor = fast_below(0.0, c.tdr_bootstrap_ms, c.tdr_bootstrap_steps);
        let below = MODES.map(|mode| fast_below(table_cost(table.costs, mode), c.tdr_bootstrap_ms, c.tdr_bootstrap_steps));
        let info = rs.adapter.get_info();
        let card = saved::Card {
            app: crate::sysinfo::version_string(),
            name: info.name.clone(),
            api: format!("{:?}", info.backend),
            driver: crate::farm::driver_text(&info.driver, &info.driver_info),
        };
        let power = PowerState::read();
        let dir = if setting == 1 { fractadyne_state::config_dir() } else { None };
        let readings = dir.as_deref().map(saved::load).unwrap_or_default();
        let live = MODES.map(|m| fractadyne_gpu::live_iter_pipeline(rs, m.to_u32()));
        let mut how = String::new();
        // A saved reading for this card in this power state: used now, re-checked in the background.
        let saved_hit = saved::find(&readings, &card, &power).cloned();
        let found: [(Result<Finding, String>, String); 3] = match saved_hit {
            Some(r) => {
                let findings = r.findings().expect("find returns usable readings only");
                let mode = self.live_render_mode(self.fractal, false, self.viewport.magnification());
                let k = MODES.iter().position(|&m| m == mode).unwrap_or(0);
                let taken = if r.taken.is_empty() { "earlier".to_string() } else { r.taken.clone() };
                how = format!(
                    "saved {taken}, in use; {} is re-checked in the background once the picture is up and the view is still",
                    mode_name(MODES[k])
                );
                self.speed_recheck = Some(Recheck::new(recheck::Saved {
                    card: card.clone(),
                    power: power.clone(),
                    taken,
                    findings,
                    k,
                    below,
                    opening_floor,
                    live: live.clone(),
                    gl: info.backend == eframe::wgpu::Backend::Gl,
                    adopted_frame: self.perf.frame_idx,
                }));
                findings.map(|f| (Ok(f), String::from("saved")))
            }
            None => {
                let mut out = Vec::new();
                for (k, mode) in MODES.into_iter().enumerate() {
                    let tm = Instant::now();
                    let found = match &live[k] {
                        Some(l) => {
                            let mut req = plan_request(self.speed_probe_plan(mode));
                            ramp(&rs.device, &rs.queue, l, &mut req, below[k])
                        }
                        None => Err("the renderer is not installed".to_string()),
                    };
                    out.push((found, format!("{:.0} ms", tm.elapsed().as_secs_f64() * 1000.0)));
                }
                out.try_into().expect("one per mode")
            }
        };
        let parts = self.speed_probe_apply(&found, opening_floor);
        // Floatexp's cost for the export's tiles; direct's and df32's follow once the calibration
        // is resolved (`main.rs`, `tell_export_card_costs`).
        self.speed_probe_floatexp = found[2].0.as_ref().ok().copied();
        let from_saved = self.speed_recheck.is_some();
        // Every mode measured: filed for the next launch in this state. A mode that could not be
        // measured saves nothing, so the next launch tries again.
        if let (false, Some(d), [(Ok(d0), _), (Ok(f), _), (Ok(x), _)]) = (from_saved, &dir, &found) {
            let r = saved::Reading {
                card: card.clone(),
                power: power.clone(),
                taken: taken_now(),
                direct: saved::SavedFinding::of(*d0),
                df32: saved::SavedFinding::of(*f),
                floatexp: saved::SavedFinding::of(*x),
            };
            match saved::store(d, &saved::with(&readings, r)) {
                Ok(()) => how.push_str(&format!("saved to {}", saved::FILE)),
                Err(e) => how.push_str(&format!("not saved ({e})")),
            }
        }
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        // Its time lies inside the next frame interval, and is no frame's dispatch: discounted from
        // it like a pipeline compile (`Perf::startup_probe_ms`). Counted, it read as a 10 s frame
        // the GPU timestamps could not see (`FRAME BUDGET IS BLIND`, Eris dev.8, 2026-11-10).
        self.perf.startup_probe_ms = ms;
        crate::diag::log_line(
            "wgpu",
            &format!(
                "speed probe ({}): {} — {ms:.0} ms; {}{}",
                table.source,
                parts.join("; "),
                power.line(),
                if how.is_empty() { String::new() } else { format!("; {how}") }
            ),
        );
        Some(lowered_costs(&found.map(|(f, _)| f)))
    }

    /// Record the opening rate of each mode `found` measured slow ([`speed_probe_opens`]), in both
    /// views, and return the log's words for each mode (`found`'s second field: how long it took).
    fn speed_probe_apply(&mut self, found: &[(Result<Finding, String>, String); 3], opening_floor: f64) -> Vec<String> {
        let mut parts = Vec::new();
        for (mode, (f, took)) in MODES.into_iter().zip(found) {
            let opens = speed_probe_opens(f, opening_floor);
            if let Some(ms_per_step) = opens {
                for v in 0..2 {
                    self.perf.record_opening_rate(v, mode.to_u32(), 1.0 / ms_per_step);
                }
            }
            parts.push(finding_part(mode, f, took, opens.is_some()));
        }
        parts
    }

    /// What a probe pass in `mode` needs from the app ([`Plan`]): cheap, for the UI thread; the
    /// reference is built by [`plan_request`], where the passes run.
    fn speed_probe_plan(&self, mode: RenderMode) -> Plan {
        let mut vp = Viewport::new(f64::from(SIDE), f64::from(SIDE));
        vp.center_x = fractadyne_core::parse_bf(CENTER.0).expect("a decimal literal parses");
        vp.center_y = fractadyne_core::parse_bf(CENTER.1).expect("a decimal literal parses");
        vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(2f64.powi(upp_log2(mode)));
        vp.precision = fractadyne_core::precision_for_magnification(vp.magnification());
        let inputs = (!mode.is_direct()).then(|| {
            let scale = vp.gpu_scale();
            let mut inputs = self.export_reference_inputs(
                &vp, false, mode, ITERS_MAX, vp.precision, scale.span_mantissa, scale.delta_exp,
            );
            inputs.origin = "speed-probe";
            inputs.formula = FractalKind::Mandelbrot.formula_id();
            inputs.custom = None;
            inputs.do_sa = false;
            inputs.bla_dc_max = None;
            inputs.nucleus = false;
            inputs
        });
        Plan { mode, vp, inputs }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/render/speed_probe/tests.rs"]
mod tests;
