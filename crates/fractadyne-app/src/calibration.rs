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
//! Multibrot 5, aux coloring) at an iteration count so high that ONE pixel's chain costs more than
//! the target. The resolution shrink and settle tiles leave `16·16·iter` unbounded, and so does
//! this; only the chunked path's iteration windows reach it.
//!
//! Compiled in, keyed by an adapter slug; an adapter with no entry gets the table's conservative
//! default. Not a `--set` override (§5.2: a per-card value as an override would make every gate on
//! that card non-stock); `DISPATCH_CEILING=0` only switches it off for a before/after measurement.

use std::sync::atomic::{AtomicU64, Ordering};
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
    /// The per-pass fixed GPU cost, ms (design §5.2): this adapter's `pass_fixed_ms`, else the
    /// table default's. `None` only when neither is in the table.
    pub(crate) pass_fixed_ms: Option<f64>,
    /// `pass_fixed_ms` was measured on THIS adapter (its own entry has one), so `--chunk-sweep` can
    /// hold a new measurement to it.
    pub(crate) pass_fixed_calibrated: bool,
    /// The adapter has an entry of its own (it was measured), rather than the default's.
    pub(crate) own_entry: bool,
}

const TABLE: &str = include_str!("../../../validation/calibration/ceilings.toml");

struct Active {
    cal: Calibration,
    factors: Vec<Factor>,
    /// The adapter the calibration was resolved for, as `init` was given it.
    adapter: String,
}

static ACTIVE: OnceLock<Active> = OnceLock::new();

/// Costs measured AFTER the calibration was resolved (the speed check's background re-check,
/// `render::speed_probe::recheck`), as f64 bits, 0 = none: [`costs_in_force`] takes the slower of
/// these and the resolved ones, so a later measurement can only lower a ceiling, like the first.
static LATER_DIRECT: AtomicU64 = AtomicU64::new(0);
static LATER_DF32: AtomicU64 = AtomicU64::new(0);

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
    /// The per-pass fixed cost, ms, from `--chunk-sweep`'s fit (absent = the default's).
    #[serde(default)]
    pass_fixed_ms: Option<f64>,
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
            pass_fixed_ms: positive(self.pass_fixed_ms),
            pass_fixed_calibrated: false,
            own_entry: false,
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

/// A measured value, or `None` for a missing, non-finite or non-positive one.
fn positive(v: Option<f64>) -> Option<f64> {
    v.filter(|v| v.is_finite() && *v > 0.0)
}

/// Look `adapter` up in a parsed table.
pub(crate) fn lookup(t: &Table, adapter: &str) -> Calibration {
    let key = slug(adapter);
    match t.adapter.iter().find(|e| e.slug == key) {
        Some(e) => {
            let mut c = e.calibration(format!("calibrated for {key}"));
            c.own_entry = true;
            c.pass_fixed_calibrated = c.pass_fixed_ms.is_some();
            c.pass_fixed_ms = c.pass_fixed_ms.or(positive(t.default.pass_fixed_ms));
            c
        }
        None => t.default.calibration(format!("default (no entry for {key})")),
    }
}

/// The formula's cost relative to Mandelbrot in `mode`, clamped at ≥ 1 (see the module header). A
/// formula without a row counts as Mandelbrot.
fn factor_of(rows: &[Factor], formula: FractalKind, mode: RenderMode) -> f64 {
    if formula == FractalKind::Custom {
        // No row can describe a formula the user wrote: its estimate (`custom::cost_factor`).
        return custom_factor();
    }
    let v = rows.iter().find(|f| f.name == formula.name()).map_or(1.0, |f| {
        if mode.is_direct() { f.direct } else { f.df32_pert }
    });
    if v.is_finite() { v.max(1.0) } else { 1.0 }
}

/// The active custom formula's cost factor, as `f64` bits. Process-wide like the table itself: one
/// custom formula is active at a time, and an export of it runs with the same one.
static CUSTOM_FACTOR: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Set when a custom formula is applied (`fractadyne_gpu::custom::CustomShader::cost_factor`).
pub(crate) fn set_custom_factor(v: f64) {
    CUSTOM_FACTOR.store(v.to_bits(), std::sync::atomic::Ordering::Relaxed);
}

/// Before any custom formula is applied, 3× Mandelbrot — above every built-in's measured factor.
fn custom_factor() -> f64 {
    let v = f64::from_bits(CUSTOM_FACTOR.load(std::sync::atomic::Ordering::Relaxed));
    if v.is_finite() && v >= 1.0 { v } else { 3.0 }
}

/// What the table gives `adapter` (its own entry, or the default), without resolving it for the
/// run: the start-up speed probe reads it to know whether the card was measured and what its
/// measurement would have to beat. `None` when the table does not parse.
pub(crate) fn table_entry(adapter: &str) -> Option<Calibration> {
    parse(TABLE).ok().map(|t| lookup(&t, adapter))
}

/// Each of `table`'s per-step costs, raised to `measured`'s where the card measured SLOWER (a
/// higher cost is a lower ceiling), and whether any was. A measured term that is 0 or not finite
/// says nothing and leaves the table's. Never lowers a cost, so it can never raise a ceiling: the
/// start-up probe's reading is a bound for safety, not the cost model (MEM-16;
/// design/live-render-robustness.md §5.2).
pub(crate) fn slower_of(table: Costs, measured: Costs) -> (Costs, bool) {
    let pick = |t: f64, m: f64| if m.is_finite() && m > t { (m, true) } else { (t, false) };
    let (df32_pert, a) = pick(table.df32_pert, measured.df32_pert);
    let (direct, b) = pick(table.direct, measured.direct);
    (Costs { df32_pert, direct }, a || b)
}

/// Resolve the running adapter's calibration, once. Returns it for the log. A table that does not
/// parse is a build defect (it is compiled in and tested), so it degrades to NO ceiling rather than
/// a guess, and says so.
pub(crate) fn init(adapter: &str) -> &'static Calibration {
    init_measured(adapter, None)
}

/// [`init`], with the start-up speed probe's measurement of this card (`render::speed_probe`): the
/// table's costs raised to the measured ones where the card measured slower ([`slower_of`]), so its
/// ceiling only comes down. `None` = nothing measured. Once resolved, a later call returns what was
/// resolved and ignores what it is given.
pub(crate) fn init_measured(adapter: &str, measured: Option<Costs>) -> &'static Calibration {
    &ACTIVE
        .get_or_init(|| match parse(TABLE) {
            Ok(t) => {
                let mut cal = lookup(&t, adapter);
                if let Some(m) = measured {
                    let (costs, slower) = slower_of(cal.costs, m);
                    if slower {
                        cal.costs = costs;
                        cal.source = format!("{}, lowered to this card's start-up measurement", cal.source);
                    }
                }
                Active { cal, factors: t.formula, adapter: adapter.to_string() }
            }
            Err(e) => Active {
                cal: Calibration {
                    costs: Costs::default(),
                    knee_px: 0,
                    source: format!("none (calibration table unreadable: {e})"),
                    pass_fixed_ms: None,
                    pass_fixed_calibrated: false,
                    own_entry: false,
                },
                factors: Vec::new(),
                adapter: adapter.to_string(),
            },
        })
        .cal
}

/// Lower the ceiling to `measured` where the card measured slower than the costs in force
/// ([`slower_of`]: never raises one). For a measurement taken after [`init_measured`] resolved the
/// calibration (the speed check's background re-check); `false` = nothing came down, or nothing is
/// resolved yet.
pub(crate) fn lower_to(measured: Costs) -> bool {
    let Some(now) = costs_in_force() else { return false };
    let (c, slower) = slower_of(now, measured);
    if slower {
        LATER_DIRECT.store(c.direct.to_bits(), Ordering::Relaxed);
        LATER_DF32.store(c.df32_pert.to_bits(), Ordering::Relaxed);
    }
    slower
}

/// The per-step costs the ceiling is sized from: the resolved ones, raised where a later
/// measurement read the card slower ([`lower_to`]). `None` before [`init_measured`].
pub(crate) fn costs_in_force() -> Option<Costs> {
    let a = ACTIVE.get()?;
    let later = |v: &AtomicU64| f64::from_bits(v.load(Ordering::Relaxed));
    Some(slower_of(a.cal.costs, Costs { direct: later(&LATER_DIRECT), df32_pert: later(&LATER_DF32) }).0)
}

/// The adapter this run's calibration was resolved for (`None` before `init`, e.g. a headless
/// harness): the name a decision recording's header carries, so a fixture set can say which cards
/// its recordings come from.
pub(crate) fn adapter() -> Option<&'static str> {
    ACTIVE.get().map(|a| a.adapter.as_str())
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
    let costs = costs_in_force()?;
    let cost = if mode.is_direct() {
        costs.direct
    } else if mode == RenderMode::Df32Pert {
        costs.df32_pert
    } else {
        return None;
    };
    ceiling(cost, factor_of(&a.factors, formula, mode), a.cal.knee_px, px)
}

/// ⭐THE CALIBRATED OPENING (2026-11-18): the most an UNPRICED dispatch may carry so that it costs
/// at most `share × TARGET_MS` (`TDR_BOOTSTRAP_MS`, 40 ms) at the adapter's worst measured per-step
/// cost — the bound the live-refresh probe already sizes its unpriced passes by (`render.rs`, "An
/// unpriced probe fits a `TDR_BOOTSTRAP_MS` share of the ceiling at the probe's own size").
///
/// `cap` is the ceiling at the frame's own pixels and `ceiling_at` the ceiling at any other count.
/// The share of `cap` is converted to pixels at `gpu_iter` and the ceiling re-taken THERE: below the
/// occupancy knee a dispatch costs its iteration window over the knee's pixels, so a small opening
/// frame gets the knee-reduced share, never the saturated one.
pub(crate) fn opening(cap: u64, share: f64, gpu_iter: u32, ceiling_at: impl Fn(u64) -> Option<u64>) -> Option<u64> {
    if !(share > 0.0) || !share.is_finite() {
        return None;
    }
    let share = share.min(1.0);
    let px = (cap as f64 * share / gpu_iter.max(1) as f64) as u64;
    ceiling_at(px.max(1)).map(|c| (c as f64 * share) as u64)
}

/// [`opening`] for `formula` in `mode` at `px` pixels and `gpu_iter` iterations on the running
/// adapter. `None` where the calibration cannot vouch for one: an adapter WITHOUT its own entry (the
/// table default is not a measurement of this card, and a card slower than it would turn 40 ms into
/// more — there the rate-derived guess stays the only opening), floatexp (no ceiling: BLA decides
/// its cost), before the adapter is known, or with `DISPATCH_CEILING=0`.
pub(crate) fn opening_for(mode: RenderMode, formula: FractalKind, px: u64, gpu_iter: u32) -> Option<u64> {
    if !own_entry() {
        return None;
    }
    let share = crate::tunables::cost().tdr_bootstrap_ms / TARGET_MS;
    let cap = ceiling_for(mode, formula, px)?;
    opening(cap, share, gpu_iter, |p| ceiling_for(mode, formula, p))
}

/// ⭐THE CROSS-MODE OPENING (2026-11-18): the opening of a switch between two calibrated modes, from
/// the rate (`prev_rate`, nominal steps per ms) this view measured in the mode it is LEAVING, on the
/// same pixels at the same iteration ask a frame ago. The worst-case per-step costs the two ceilings
/// are sized from give the ratio between the modes (`ceil_prev / ceil_new`: df32 costs 5.2× direct on
/// the RTX 3080), so `prev_rate × ceil_new / ceil_prev` is the new mode's rate at no better than the
/// calibrated ratio, and `bootstrap_ms` of it is the opening — never past `ceil_new`, which bounds it
/// at its 400 ms worst case whatever the content. 0 when there is no rate.
///
/// Why: the maintainer saw the df32 entry flash (2026-11-18, b6921): the calibrated opening is 40 ms
/// at the WORST cost, ~4% of a 1500×1635 panel at ~13.6k iterations, and the view climbed through
/// several sizes to the ceiling within ~0.36 s — after a direct picture at full size. Direct's last
/// pass there (21:06 session) took 20 ms for 5.03e10 steps: ×0.194, ×40 ms ≈ 1.9e10, past the
/// ceiling (1.449e10), so the first df32 frame opens at the size the mode then runs at.
pub(crate) fn cross_opening(prev_rate: f64, ceil_prev: u64, ceil_new: u64, bootstrap_ms: f64) -> u64 {
    if !(prev_rate > 0.0 && prev_rate.is_finite()) || ceil_prev == 0 || !(bootstrap_ms > 0.0) {
        return 0;
    }
    let rate_new = prev_rate * ceil_new as f64 / ceil_prev as f64;
    ((rate_new * bootstrap_ms) as u64).min(ceil_new)
}

/// [`cross_opening`] for a switch from `prev` to `new` on the running adapter at `px` pixels: `None`
/// unless the adapter has its own entry and both modes have a ceiling (direct and df32; floatexp has
/// none, so a switch into or out of it keeps the old rules).
pub(crate) fn cross_opening_for(
    prev: RenderMode,
    new: RenderMode,
    formula: FractalKind,
    px: u64,
    prev_rate: f64,
) -> Option<u64> {
    if !own_entry() {
        return None;
    }
    let ceil_prev = ceiling_for(prev, formula, px)?;
    let ceil_new = ceiling_for(new, formula, px)?;
    Some(cross_opening(prev_rate, ceil_prev, ceil_new, crate::tunables::cost().tdr_bootstrap_ms))
}

/// The per-pass fixed GPU cost the pacing subtracts from a pass's time, ms (design §5.2;
/// `render::variable_pass_ms`, `render::motion_pass_steps`): `--set PASS_FIXED_MS` when given (the
/// A/B arm, and the way to try a value), else this adapter's calibrated `pass_fixed_ms`, else the
/// table default's, else `PASS_FIXED_MS_DEFAULT` (before the adapter is known).
///
/// A CALIBRATION, not a tunable (§5.2): a per-card value delivered as `--set` would make every gate
/// on that card non-stock. It does not depend on content (it is the cost of clearing and storing
/// the walk's state attachments and the resolve), which is what exempts it from MEM-16's rule
/// against calibrating the cost model at startup; `--chunk-sweep` at two views tests that.
pub(crate) fn pass_fixed_ms() -> f64 {
    if crate::tunables::overridden("PASS_FIXED_MS") {
        return crate::tunables::cost().pass_fixed_ms;
    }
    ACTIVE
        .get()
        .and_then(|a| a.cal.pass_fixed_ms)
        .unwrap_or(crate::tunables::PASS_FIXED_MS_DEFAULT)
}

/// This adapter's own measured per-pass fixed cost, ms, if the table has one (`--chunk-sweep`'s
/// gate holds a new measurement to it); `None` for a card on the default.
pub(crate) fn calibrated_pass_fixed_ms() -> Option<f64> {
    ACTIVE.get().filter(|a| a.cal.pass_fixed_calibrated).and_then(|a| a.cal.pass_fixed_ms)
}

/// Whether the running adapter has its own entry in the table: a card that was measured, which is
/// what the release gates run on (`false` before `init`).
pub(crate) fn own_entry() -> bool {
    ACTIVE.get().is_some_and(|a| a.cal.own_entry)
}

/// The cost factor of the formula with GPU id `formula` in `mode` (see [`factor_of`]): an id no
/// built-in has is the custom formula's. Before `init`, 3.0 for a built-in (above every measured
/// factor; the largest is 2.85), the custom estimate for the rest.
pub(crate) fn factor_for_id(formula: u32, mode: RenderMode) -> f64 {
    let kind = FractalKind::ALL.iter().copied().find(|k| k.formula_id() == formula).unwrap_or(FractalKind::Custom);
    match ACTIVE.get() {
        Some(a) => factor_of(&a.factors, kind, mode),
        None if kind == FractalKind::Custom => custom_factor(),
        None => 3.0,
    }
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
#[path = "../tests/unit/calibration/tests.rs"]
mod tests;
