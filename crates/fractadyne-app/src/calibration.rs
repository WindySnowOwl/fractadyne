//! PER-ADAPTER CALIBRATION: the FIXED per-dispatch ceiling (design/live-render-robustness.md §5.1).
//!
//! The frame budget is LEARNED, and a learned number can be stale: on 2026-09-27 a budget learned
//! at 1.515e11 and carried to the crash view sized whole-frame dispatches of 1.0–2.0 s on the RX
//! 6800 XT and hung the machine (the dead-man, beta.132, catches the SECOND such frame; nothing
//! learned can bound the FIRST). The ceiling is the term nothing learned can raise: an absolute cap
//! on one dispatch's nominal steps (`px × ss² × iteration window`), sized so that a ceiling-sized
//! dispatch at this adapter's WORST measured per-step cost takes [`TARGET_MS`].
//!
//! Three measured terms, all in `validation/calibration/ceilings.toml`:
//!
//! - **The card's worst per-step cost, per arithmetic mode.** Measured at an ALL-INTERIOR view
//!   (`validation/calibration/interior-*.fdn`): every pixel walks every step, so nominal == real
//!   and nothing escapes early. On the RTX 3080 that view costs MORE per step than the
//!   escaped-reference storm the ceiling was first sized from (2.76e-8 against 2.07e-8 ms/step).
//! - **The formula's cost relative to Mandelbrot**, per mode (Multibrot 5 is 1.77× in direct,
//!   1.48× in df32 perturbation). Clamped at ≥ 1: a cheap formula never lifts the ceiling above the
//!   Mandelbrot calibration, because the ceiling is for safety and a lower one costs only the
//!   per-dispatch overhead of splitting the work.
//! - **The occupancy knee.** Below ~262k px (RTX 3080) the GPU is not saturated, and a dispatch
//!   costs its iteration WINDOW rather than `px × window` — per nominal step up to 2.6× the
//!   saturated rate at 16k px. The ceiling therefore scales by `min(1, px / knee)`, which bounds
//!   `max(px, knee) × window` instead.
//!
//! FLOATEXP has no ceiling here: BLA skips most of its nominal steps, so a step cap low enough to
//! be safe with BLA skipping nothing would choke every ordinary deep frame. It keeps the dead-man
//! as its backstop (stated, not hidden: `ceiling_for(Floatexp, ..)` is `None`).
//!
//! ⚠What this cannot bound: a frame that cannot be split on the iteration axis (formulas past
//! Multibrot 5, aux colouring) at an iteration count so high that ONE pixel's chain costs more than
//! the target. The resolution shrink and settle tiles leave `16·16·iter` unbounded, and so does
//! this; only the chunked path's iteration windows reach it.
//!
//! Compiled in, keyed by an adapter slug; an adapter with no entry gets the table's conservative
//! default. Not a `--set` override (§5.2: a per-card value as an override would make every gate on
//! that card non-stock); `DISPATCH_CEILING=0` only switches it off for a before/after measurement.

use std::sync::OnceLock;

use crate::fractal::FractalKind;
use crate::RenderMode;

/// What one ceiling-sized dispatch costs at the calibrated worst case. The budget's own target
/// (`TDR_BUDGET_MS`), kept as a separate constant so a `--set` experiment on the budget cannot move
/// the ceiling.
pub(crate) const TARGET_MS: f64 = 400.0;

/// Worst measured cost of one nominal step, per mode, in ms. `0` = unmeasured (no ceiling).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Costs {
    pub(crate) df32_pert: f64,
    pub(crate) direct: f64,
}

/// What the running adapter got, and where it came from (for the log).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Calibration {
    pub(crate) costs: Costs,
    pub(crate) knee_px: u64,
    pub(crate) source: String,
}

const TABLE: &str = include_str!("../../../validation/calibration/ceilings.toml");

struct Active {
    cal: Calibration,
    factors: Vec<Factor>,
}

static ACTIVE: OnceLock<Active> = OnceLock::new();

#[derive(serde::Deserialize)]
pub(crate) struct Table {
    default: Card,
    #[serde(default)]
    adapter: Vec<Card>,
    #[serde(default)]
    formula: Vec<Factor>,
}

#[derive(serde::Deserialize)]
struct Card {
    #[serde(default)]
    slug: String,
    direct_ms_per_step: f64,
    df32_pert_ms_per_step: f64,
    knee_px: u64,
}

#[derive(serde::Deserialize, Clone)]
struct Factor {
    name: String,
    direct: f64,
    df32_pert: f64,
}

impl Card {
    fn calibration(&self, source: String) -> Calibration {
        let c = |v: f64| if v.is_finite() && v > 0.0 { v } else { 0.0 };
        Calibration {
            costs: Costs { df32_pert: c(self.df32_pert_ms_per_step), direct: c(self.direct_ms_per_step) },
            knee_px: self.knee_px,
            source,
        }
    }
}

/// The adapter's name as a table key: lowercase, every run of non-alphanumerics one `-`.
/// "AMD Radeon RX 6800 XT" → "amd-radeon-rx-6800-xt".
pub(crate) fn slug(adapter: &str) -> String {
    let mut s = String::with_capacity(adapter.len());
    for c in adapter.chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c.to_ascii_lowercase());
        } else if !s.ends_with('-') {
            s.push('-');
        }
    }
    s.trim_matches('-').to_string()
}

pub(crate) fn parse(table: &str) -> Result<Table, String> {
    toml::from_str::<Table>(table).map_err(|e| e.to_string())
}

/// Look `adapter` up in a parsed table.
pub(crate) fn lookup(t: &Table, adapter: &str) -> Calibration {
    let key = slug(adapter);
    match t.adapter.iter().find(|e| e.slug == key) {
        Some(e) => e.calibration(format!("calibrated for {key}")),
        None => t.default.calibration(format!("default (no entry for {key})")),
    }
}

/// The formula's cost relative to Mandelbrot in `mode`, clamped at ≥ 1 (see the module header). A
/// formula without a row counts as Mandelbrot; a test keeps every formula in the table.
#[cfg(test)]
pub(crate) fn factor(t: &Table, formula: FractalKind, mode: RenderMode) -> f64 {
    factor_of(&t.formula, formula, mode)
}

fn factor_of(rows: &[Factor], formula: FractalKind, mode: RenderMode) -> f64 {
    let v = rows.iter().find(|f| f.name == formula.name()).map_or(1.0, |f| {
        if mode.is_direct() { f.direct } else { f.df32_pert }
    });
    if v.is_finite() { v.max(1.0) } else { 1.0 }
}

/// Resolve the running adapter's calibration, once. Returns it for the log. A table that does not
/// parse is a build defect (it is compiled in and tested), so it degrades to NO ceiling rather than
/// a guess, and says so.
pub(crate) fn init(adapter: &str) -> &'static Calibration {
    &ACTIVE
        .get_or_init(|| match parse(TABLE) {
            Ok(t) => Active { cal: lookup(&t, adapter), factors: t.formula },
            Err(e) => Active {
                cal: Calibration {
                    costs: Costs::default(),
                    knee_px: 0,
                    source: format!("none (calibration table unreadable: {e})"),
                },
                factors: Vec::new(),
            },
        })
        .cal
}

/// The nominal-step ceiling a worst per-step `cost` (ms) supports at `px` dispatch pixels:
/// `TARGET_MS / (cost × factor)`, scaled by `min(1, px / knee)`. `None` when `cost` is unmeasured.
pub(crate) fn ceiling(cost: f64, factor: f64, knee_px: u64, px: u64) -> Option<u64> {
    if !(cost > 0.0) || !cost.is_finite() {
        return None;
    }
    let sat = if knee_px == 0 { 1.0 } else { (px as f64 / knee_px as f64).min(1.0) };
    let steps = TARGET_MS / (cost * factor.max(1.0)) * sat;
    Some((steps as u64).max(1))
}

/// The ceiling for a dispatch of `formula` in `mode` over `px` pixels. `None` where a step ceiling
/// cannot bound the cost (floatexp), before the adapter is known, or with `DISPATCH_CEILING=0`.
pub(crate) fn ceiling_for(mode: RenderMode, formula: FractalKind, px: u64) -> Option<u64> {
    if crate::tunables::cost().dispatch_ceiling != 1 {
        return None;
    }
    let a = ACTIVE.get()?;
    let cost = if mode.is_direct() {
        a.cal.costs.direct
    } else if mode == RenderMode::Df32Pert {
        a.cal.costs.df32_pert
    } else {
        return None;
    };
    ceiling(cost, factor_of(&a.factors, formula, mode), a.cal.knee_px, px)
}

/// The running adapter's occupancy knee, px (0 = unknown, or no knee term).
pub(crate) fn knee_px() -> u64 {
    ACTIVE.get().map_or(0, |a| a.cal.knee_px)
}

/// Apply the ceiling to a dispatch budget. Never raises it.
pub(crate) fn capped(budget: u64, ceiling: Option<u64>) -> u64 {
    ceiling.map_or(budget, |c| budget.min(c))
}

#[cfg(test)]
#[path = "calibration_tests.rs"]
mod tests;
