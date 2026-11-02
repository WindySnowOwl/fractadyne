//! ⭐**THE SELF-TEST ON A SLOW CARD** (2026-11-01): measure the card first, and skip what it cannot
//! run in one pass instead of resetting its driver.
//!
//! A GeForce GT 710 ran Help > Diagnostics > self-test and lost its device in the third check:
//! "home 1x, 50k iter", whose control is one 220x220 dispatch of 50,000 iterations, ran past
//! Windows' 2-second GPU timeout. The app paces its own work to the card (learned budgets, the
//! per-card ceiling); the self-test's controls are single passes ON PURPOSE (a chunked render is
//! held bit for bit to one), sized on cards like the RTX 3080, and heavier ones follow.
//!
//! So before the checks, [`FractadyneApp::measure_direct_step`] times one all-interior direct
//! frame, starting at 2 iterations and growing until a pass is long enough to time: every pixel
//! walks every step, so the cost per nominal step is the card's WORST in direct mode. A card with
//! no calibration entry of its own that is slower than the one the suite is sized for gets the GPU
//! crate's dispatch guard
//! (`fractadyne_gpu::dispatch_guard`): every pass is priced at nominal steps x that cost x the
//! mode's and formula's factors, and one priced over [`LIMIT_MS`] is refused before it is
//! submitted. The self-test records its check as SKIPPED. On a faster card the guard only PRICES
//! passes, so a release card's run records what the suite asks of a slow one without changing it.
//!
//! A card with its own calibration entry (a release card) is never throttled: it has run the whole
//! suite in a release battery, and the probe misreads some cards (the RX 6800 XT, 3.75x its
//! calibrated cost; see [`Policy::new`]). Under a `,gate` simulation a skip there is a FAILURE: the
//! gate must not shrink quietly ([`skips_fail`]).
//!
//! `FRACTADYNE_SELFTEST_SLOW_GPU=<ms per direct step>` simulates a card that slow, for trying the
//! skip path on a fast one: a PREVIEW of that card's run, whose skips are not counted as failures.
//! `…=<ms>,gate` judges the run as this card's instead, which is how the release-card rule is seen
//! to go red.

use crate::FractadyneApp;

/// The most one pass may be priced at, ms: a quarter of Windows' default 2-second GPU timeout
/// (`TdrDelay`), for a price that is already an upper bound.
pub(crate) const LIMIT_MS: f64 = 500.0;

/// The slowest card the suite is sized for, ms per direct step at an all-interior view: the
/// calibration table's default card (`validation/calibration/ceilings.toml`, twice the RX 6800 XT's
/// cost), which the app's own defaults are sized for too. A card measured slower gets the guard.
pub(crate) const SIZED_FOR_MS_PER_STEP: f64 = 1.92e-8;

/// What a nominal step costs in each render mode relative to direct mode, at its worst (nothing
/// skipped): df32 perturbation 5.2 (RTX 3080 2.76e-8 / 5.35e-9; the RX 6800 XT's table entry has the
/// same ratio), floatexp 15 (`topic-gpu-saturation`: floatexp is ~15x a direct iteration). An
/// unknown mode is priced as floatexp.
pub(crate) fn mode_ratio(mode: u32) -> f64 {
    match mode {
        1 => 1.0,
        0 => 5.2,
        _ => 15.0,
    }
}

/// The frame [`FractadyneApp::measure_direct_step`] times: past both measured occupancy knees
/// (262,144 px on the RTX 3080, 524,288 on the RX 6800 XT), so a fast card is timed busy.
const MEASURE_W: u32 = 1024;
const MEASURE_H: u32 = 768;
/// A pass this long is long enough to time (its fixed cost is a fraction of a millisecond).
const MEASURE_ENOUGH_MS: f64 = 25.0;
/// The iteration counts tried, in order: the first is cheap on any card that runs at all (1.6M
/// steps; ~7 ms on the GT 710, whose 50,000-iteration control took over 2 s).
const MEASURE_ITERS: [u32; 5] = [2, 16, 128, 1024, 8192];

/// One measurement of the card.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Measured {
    /// The worst cost of one nominal step in direct mode, ms.
    pub(crate) ms_per_step: f64,
    /// The pass it was read from: iterations, and its slowest wall time, ms.
    pub(crate) iters: u32,
    pub(crate) pass_ms: f64,
}

/// What the self-test does about the card's speed.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Policy {
    /// The price of one direct step, ms (measured, or the simulated value).
    pub(crate) ms_per_step: f64,
    /// Refuse passes priced over [`LIMIT_MS`].
    pub(crate) enforce: bool,
    /// The speed is `FRACTADYNE_SELFTEST_SLOW_GPU`'s, not this card's.
    pub(crate) simulated: bool,
    /// A simulation that previews a slower card's run: its skips do not count against this card.
    /// (`…,gate` simulates the speed but judges the run as this card's.)
    pub(crate) preview: bool,
    /// The card has its own calibration entry.
    pub(crate) own_entry: bool,
}

impl Policy {
    /// The policy for a card whose direct step costs `ms_per_step`.
    ///
    /// ⚠A card with its own calibration entry is never throttled (a simulation still is): it has
    /// run the whole suite in a release battery, and the probe does not measure every card the
    /// way its calibration did. On the RX 6800 XT it read 3.6e-8 ms per step, steady over four
    /// runs, 3.75x the card's calibrated direct cost and 1.9x the sizing, so beta.23's guard refused
    /// 177 passes there (priced 0.5 to 105 s) that beta.22 had run without a reset: 451/548 checks
    /// where beta.22 passed 545/546 (PLUTO battery 20261101-212811-wn12). The guard is for cards
    /// nobody has measured, like the GT 710.
    pub(crate) fn new(ms_per_step: f64, simulated: bool, own_entry: bool) -> Self {
        // A cost that is not a positive number cannot show an unknown card is fast enough.
        let unknown_and_slow = !own_entry && !(ms_per_step > 0.0 && ms_per_step <= SIZED_FOR_MS_PER_STEP);
        let enforce = simulated || unknown_and_slow;
        Self { ms_per_step, enforce, simulated, preview: simulated, own_entry }
    }

    /// The price of one nominal step of a pass in `mode` with GPU formula id `formula`, ms.
    pub(crate) fn step_ms(&self, mode: u32, formula: u32) -> f64 {
        self.ms_per_step
            * mode_ratio(mode)
            * crate::calibration::factor_for_id(formula, crate::RenderMode::from_u32(mode))
    }

    /// Install the guard for the rest of the run.
    pub(crate) fn install(&self) {
        let p = self.clone();
        fractadyne_gpu::dispatch_guard::install(fractadyne_gpu::dispatch_guard::Guard {
            cost_ms: Box::new(move |mode, formula| p.step_ms(mode, formula)),
            limit_ms: LIMIT_MS,
            enforce: self.enforce,
        });
    }

    /// The report's line about it.
    pub(crate) fn describe(&self, measured: Option<&Measured>) -> String {
        let speed = match (self.simulated, measured) {
            (true, _) => format!(
                "SIMULATED at {:.3e} ms per direct step (FRACTADYNE_SELFTEST_SLOW_GPU), {:.1}x the slowest card the suite is sized for — {}",
                self.ms_per_step,
                self.ms_per_step / SIZED_FOR_MS_PER_STEP,
                if self.preview { "a preview, not a verdict for this card" } else { "judged as this card (gate)" }
            ),
            (false, Some(m)) => format!(
                "{:.3e} ms per direct step (an all-interior {MEASURE_W}x{MEASURE_H} pass of {} iterations took {:.1} ms), {:.2}x the slowest card the suite is sized for",
                m.ms_per_step,
                m.iters,
                m.pass_ms,
                m.ms_per_step / SIZED_FOR_MS_PER_STEP
            ),
            (false, None) => "NOT MEASURED".to_string(),
        };
        let guard = if self.enforce {
            format!(
                "passes priced over {LIMIT_MS:.0} ms are not run and their checks are SKIPPED{}",
                if skips_fail(self.own_entry, self.preview) {
                    " — counted as FAILURES on this card, which has its own calibration entry"
                } else {
                    ""
                }
            )
        } else if self.own_entry && !(self.ms_per_step > 0.0 && self.ms_per_step <= SIZED_FOR_MS_PER_STEP) {
            "every pass runs: this card has its own calibration entry (passes are priced, not refused)".to_string()
        } else {
            "every pass runs (passes are priced, not refused)".to_string()
        };
        format!("{speed}; {guard}")
    }
}

/// Whether a skipped check counts as a failure: on a card with its own calibration entry, a release
/// card, a check that did not run is a gate that did not run. A preview of a slower card's run is
/// not this one's verdict.
pub(crate) fn skips_fail(own_entry: bool, preview: bool) -> bool {
    own_entry && !preview
}

/// `FRACTADYNE_SELFTEST_SLOW_GPU`, read: the simulated cost per direct step (ms), and whether the
/// run is judged as this card's (`,gate`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Simulation {
    pub(crate) ms_per_step: f64,
    pub(crate) gate: bool,
}

/// Parse `<ms>` or `<ms>,gate`. Anything else is an error, never a silent default.
pub(crate) fn parse_simulation(v: &str) -> Result<Simulation, String> {
    let (num, gate) = match v.trim().split_once(',') {
        Some((n, "gate")) => (n, true),
        Some(_) => return Err(format!("FRACTADYNE_SELFTEST_SLOW_GPU='{v}': after the comma only 'gate' is understood")),
        None => (v.trim(), false),
    };
    match num.trim().parse::<f64>() {
        Ok(x) if x.is_finite() && x > 0.0 => Ok(Simulation { ms_per_step: x, gate }),
        _ => Err(format!("FRACTADYNE_SELFTEST_SLOW_GPU='{v}' is not a positive number of ms per step")),
    }
}

/// The simulation `FRACTADYNE_SELFTEST_SLOW_GPU` asks for, if it is set.
pub(crate) fn simulation() -> Result<Option<Simulation>, String> {
    match std::env::var("FRACTADYNE_SELFTEST_SLOW_GPU") {
        Err(_) => Ok(None),
        Ok(v) => parse_simulation(&v).map(Some),
    }
}

/// The cost per step one timed pass shows, from its walls (ms): the slowest of them, which is the
/// side a guard has to err on. `None` when no wall is a positive number.
pub(crate) fn step_cost(walls: &[f64], px: u64, iters: u32) -> Option<(f64, f64)> {
    let worst = walls.iter().copied().filter(|w| w.is_finite() && *w > 0.0).fold(None, |a: Option<f64>, w| {
        Some(a.map_or(w, |a| a.max(w)))
    })?;
    Some((worst / (px as f64 * iters.max(1) as f64), worst))
}

impl FractadyneApp {
    /// Time the card in direct mode at an all-interior view (see the module header). Runs before
    /// the guard is installed, so its own passes are never refused; it starts at 2 iterations so
    /// they are cheap on any card. `Err` says why there is no measurement.
    pub(crate) fn measure_direct_step(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
    ) -> Result<Measured, String> {
        let saved = self.fractal;
        self.fractal = crate::fractal::FractalKind::Mandelbrot;
        let mut vp = fractadyne_core::Viewport::new(MEASURE_W as f64, MEASURE_H as f64);
        // Deep inside the main cardioid (the calibration's own all-interior Mandelbrot point): a
        // 0.2-wide frame here has no escaping pixel.
        let (Some(cx), Some(cy)) = (fractadyne_core::parse_bf("-0.1"), fractadyne_core::parse_bf("0.1")) else {
            self.fractal = saved;
            return Err("the measuring view's centre does not parse".to_string());
        };
        vp.center_x = cx;
        vp.center_y = cy;
        vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(0.2 / MEASURE_W as f64);
        let mut req = self.current_export_request_for(&vp, false);
        self.fractal = saved;
        req.width = MEASURE_W;
        req.height = MEASURE_H;
        req.ss = 1;
        if req.mode != 1 {
            return Err(format!("the measuring view rendered in mode {}, not direct", req.mode));
        }
        let px = MEASURE_W as u64 * MEASURE_H as u64;
        let mut last = None;
        for iters in MEASURE_ITERS {
            req.max_iter = iters;
            let (r, timed) = fractadyne_gpu::render_iter_timed(device, queue, &req, 2)
                .map_err(|e| format!("the measuring pass failed: {e}"))?;
            let escaped = r.pixels.iter().step_by(4).filter(|v| **v >= 0.0).count();
            if escaped > 0 {
                return Err(format!("{escaped} pixels of the measuring view escaped; it must be all interior"));
            }
            let walls: Vec<f64> = timed.iter().map(|t| t[1]).collect();
            let Some((ms_per_step, pass_ms)) = step_cost(&walls, px, iters) else {
                return Err("the measuring pass reported no time".to_string());
            };
            last = Some(Measured { ms_per_step, iters, pass_ms });
            if pass_ms >= MEASURE_ENOUGH_MS {
                break;
            }
        }
        last.ok_or_else(|| "no measuring pass ran".to_string())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/harness/slow_gpu/tests.rs"]
mod tests;
