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

use super::{assemble_ref_fields, pre_switch_probe_representative, recompute_worker};
use crate::calibration::Costs;
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
/// The most iterations a probe pass asks (a fast card reads as [`Finding::Fast`] long before).
pub(crate) const ITERS_MAX: u32 = 1 << 16;
/// A pass this long is timed well enough to price: the per-pass fixed cost (~0.2 ms) and the wall
/// clock's submission latency are a few percent of it, and both err slow, the safe side.
pub(crate) const PASS_TARGET_MS: f64 = 12.0;
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
    /// The slowest timed pass at `iters` took `ms`, at least [`PASS_TARGET_MS`] (or at
    /// [`ITERS_MAX`]): one nominal step costs `ms_per_step` here, at worst.
    Measured { iters: u32, ms: f64, ms_per_step: f64 },
    /// At `iters` the pass took `ms`, so a step costs at most `ms_per_step` (the pass's fixed cost
    /// is in it): below every cost that would change an opening or a ceiling. Nothing is recorded.
    Fast { iters: u32, ms: f64, ms_per_step: f64 },
}

/// The ramp's next move after a timed pass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Step {
    Done(Finding),
    /// Run the next pass at this many iterations.
    Next(u32),
}

/// The ramp, after a pass of `px` pixels at `iters` iterations whose slowest timing was
/// `slowest_ms`. `fast_below` is the cost per step under which nothing would change
/// ([`fast_below`]). The next pass aims at twice [`PASS_TARGET_MS`] from this one's cost per
/// iteration, which carries the pass's fixed cost and so over-estimates: it lands at or under
/// ~2× the target, and never more than 16× the iterations or less than 2×. `None` = the timing is
/// unusable (zero, negative or not finite).
pub(crate) fn step(px: u64, iters: u32, slowest_ms: f64, fast_below: f64) -> Option<Step> {
    if !(slowest_ms.is_finite() && slowest_ms > 0.0) || px == 0 || iters == 0 {
        return None;
    }
    let ms_per_step = slowest_ms / (px as f64 * f64::from(iters));
    if slowest_ms >= PASS_TARGET_MS {
        return Some(Step::Done(Finding::Measured { iters, ms: slowest_ms, ms_per_step }));
    }
    if ms_per_step < fast_below {
        return Some(Step::Done(Finding::Fast { iters, ms: slowest_ms, ms_per_step }));
    }
    if iters >= ITERS_MAX {
        return Some(Step::Done(Finding::Measured { iters, ms: slowest_ms, ms_per_step }));
    }
    let lo = f64::from(iters) * 2.0;
    let hi = f64::from(iters) * 16.0;
    let aim = f64::from(iters) * 2.0 * PASS_TARGET_MS / slowest_ms;
    Some(Step::Next(aim.clamp(lo, hi).min(f64::from(ITERS_MAX)) as u32))
}

/// The cost per step below which a mode's measurement changes nothing: its opening is already the
/// whole `bootstrap_steps` (`rate × bootstrap_ms ≥ bootstrap_steps`), and its ceiling's table cost
/// is not exceeded (`table_cost`, 0 = no ceiling in that mode).
pub(crate) fn fast_below(table_cost: f64, bootstrap_ms: f64, bootstrap_steps: u64) -> f64 {
    let opening = if bootstrap_steps > 0 { bootstrap_ms / bootstrap_steps as f64 } else { 0.0 };
    if table_cost > 0.0 && table_cost.is_finite() { opening.min(table_cost) } else { opening }
}

/// A timed pass's time: the slower of its GPU timestamp and its wall clock. The wall carries the
/// submission and the wait, so it errs slow, the safe side for a safety bound; and a card whose
/// timestamps describe another dispatch (the RX 6800 XT's, `calibration`) cannot make it fast.
pub(crate) fn pass_ms([ts, wall]: [f64; 2]) -> f64 {
    if ts.is_finite() && ts > 0.0 { ts.max(wall) } else { wall }
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

impl FractadyneApp {
    /// Run the probe if this card gets one (`STARTUP_PROBE`), record each slow mode's opening rate,
    /// log what it found, and return the costs the ceiling should be lowered to (0 = leave the
    /// table's). `None` = not probed. Called once, on the first frame that has a device, before the
    /// calibration is resolved (`calibration::init_measured`).
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
        let mut lowered = Costs::default();
        let mut parts = Vec::new();
        for mode in MODES {
            let below = fast_below(table_cost(table.costs, mode), c.tdr_bootstrap_ms, c.tdr_bootstrap_steps);
            match self.speed_probe_mode(rs, mode, below) {
                Ok(Finding::Measured { iters, ms, ms_per_step }) => {
                    // The opening, only where it would come down (a faster reading would mark the
                    // mode measured and change nothing else, e.g. stop the pre-switch probe).
                    let opens = ms_per_step > opening_floor;
                    if opens {
                        for v in 0..2 {
                            self.perf.record_opening_rate(v, mode.to_u32(), 1.0 / ms_per_step);
                        }
                    }
                    match mode {
                        RenderMode::Direct => lowered.direct = ms_per_step,
                        RenderMode::Df32Pert => lowered.df32_pert = ms_per_step,
                        RenderMode::Floatexp => {}
                    }
                    parts.push(format!(
                        "{} {ms_per_step:.3e} ms/step ({SIDE}² × {iters}: {ms:.1} ms){}",
                        mode_name(mode),
                        if opens {
                            format!(", opens at {:.2e} steps", super::bootstrap_steps(1.0 / ms_per_step, 0.0) as f64)
                        } else {
                            String::new()
                        }
                    ));
                }
                Ok(Finding::Fast { iters, ms, ms_per_step }) => parts.push(format!(
                    "{} under {ms_per_step:.2e} ms/step ({SIDE}² × {iters}: {ms:.2} ms), faster than the defaults",
                    mode_name(mode)
                )),
                Err(e) => parts.push(format!("{} not measured ({e})", mode_name(mode))),
            }
        }
        crate::diag::log_line(
            "wgpu",
            &format!(
                "speed probe ({}): {} — {:.0} ms",
                table.source,
                parts.join("; "),
                t0.elapsed().as_secs_f64() * 1000.0
            ),
        );
        Some(lowered)
    }

    /// One mode's ramp (see the module header).
    fn speed_probe_mode(
        &self,
        rs: &eframe::egui_wgpu::RenderState,
        mode: RenderMode,
        fast_below: f64,
    ) -> Result<Finding, String> {
        let live = fractadyne_gpu::live_iter_pipeline(rs, mode.to_u32())
            .ok_or_else(|| "the renderer is not installed".to_string())?;
        let (mut req, iters_cap) = self.speed_probe_request(mode);
        let px = u64::from(SIDE) * u64::from(SIDE);
        let mut iters = ITERS_START.min(iters_cap);
        loop {
            req.max_iter = iters;
            let (r, reps) = fractadyne_gpu::render_iter_probe(&rs.device, &rs.queue, &req, REPS, &live)
                .map_err(|e| e.to_string())?;
            let (running, ok) = pre_switch_probe_representative(&r.pixels, SIDE, SIDE);
            if !ok {
                return Err(format!("{:.1}% of pixels ran every step at {iters} (need 98%)", running * 100.0));
            }
            let slowest = reps.iter().map(|&t| pass_ms(t)).fold(f64::NAN, f64::max);
            match step(px, iters, slowest, fast_below) {
                None => return Err(format!("unusable times {reps:?} ms at {iters}")),
                Some(Step::Done(f)) => return Ok(f),
                Some(Step::Next(n)) if n > iters && n <= iters_cap => iters = n,
                Some(Step::Next(_)) => {
                    // The reference stopped short (`partial`): price what was timed.
                    let ms_per_step = slowest / (px as f64 * f64::from(iters));
                    return Ok(Finding::Measured { iters, ms: slowest, ms_per_step });
                }
            }
        }
    }

    /// The probe's request for `mode` at the cardioid point, and the most iterations its reference
    /// supports. Mandelbrot whatever the session shows, plain smooth coloring, nothing that skips
    /// steps: BLA, the series skip and the renormalized step are off.
    fn speed_probe_request(&self, mode: RenderMode) -> (fractadyne_gpu::ExportRequest, u32) {
        let mut vp = Viewport::new(f64::from(SIDE), f64::from(SIDE));
        vp.center_x = fractadyne_core::parse_bf(CENTER.0).expect("a decimal literal parses");
        vp.center_y = fractadyne_core::parse_bf(CENTER.1).expect("a decimal literal parses");
        vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(2f64.powi(upp_log2(mode)));
        vp.precision = fractadyne_core::precision_for_magnification(vp.magnification());
        let scale = vp.gpu_scale();
        let mut iters_cap = ITERS_MAX;
        let fields = if mode.is_direct() {
            super::RefFields::default()
        } else {
            let mut inputs = self.export_reference_inputs(
                &vp, false, mode, ITERS_MAX, vp.precision, scale.span_mantissa, scale.delta_exp,
            );
            inputs.origin = "speed-probe";
            inputs.formula = FractalKind::Mandelbrot.formula_id();
            inputs.custom = None;
            inputs.do_sa = false;
            inputs.bla_dc_max = None;
            inputs.nucleus = false;
            let res = recompute_worker(inputs);
            if res.partial {
                iters_cap = res.orbit_len.saturating_sub(1).max(1);
            }
            assemble_ref_fields(&vp, vp.precision, scale.delta_exp, res)
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
            aa_palette: false,
            lut: std::sync::Arc::new(Vec::new()),
            lut_smooth: false,
            light: 0,
            light_angle: 0.0,
            light_height: 0.0,
            de_on: 0,
            de_strength: 0.0,
            de_width: 0.0,
            de_phase: 0.0,
            color_method: crate::ColorMethod::Smooth.to_u32(),
            stripe_freq: 0.0,
            stripe_tail: false,
            stripe_tail_len: 0,
            trap_type: 0,
            aa_filter: 1,
            interior_col: [0.0; 4],
            work_budget: None,
            pass_ms: None,
            jitter: [0.0, 0.0],
            tile_px_max: None,
        };
        (req, iters_cap)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/render/speed_probe/tests.rs"]
mod tests;
