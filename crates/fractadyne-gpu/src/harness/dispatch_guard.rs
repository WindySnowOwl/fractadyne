//! ⭐**THE SELF-TEST'S SLOW-GPU GUARD**: a cap on what one pass may cost, which `--selftest`
//! installs and nothing else does.
//!
//! The self-test's controls are single passes on purpose (a chunked render is held bit for bit to
//! one dispatch of the same work), and its views were sized on cards like the RTX 3080. On a
//! GeForce GT 710 the third of them, 220x220 px at 50,000 iterations in one dispatch, ran past
//! Windows' 2-second GPU timeout (2026-11-01); the reset took the user's desktop and their open
//! window with it, and heavier passes follow it in the suite.
//!
//! So every iterate pass in `export.rs` asks [`admit`] before it is submitted, with its NOMINAL
//! steps (pixels x iteration window: an upper bound on the steps it can run). An installed guard
//! prices them at the card's measured worst cost per step and refuses a pass priced over its limit
//! with [`GpuError::TooSlowForThisGpu`], before anything reaches the GPU. The self-test records a
//! check whose pass was refused as SKIPPED. Nominal is a bound, not a forecast: a refused pass
//! might have finished in time, never the reverse, which is the side a guard against a driver reset
//! has to err on.
//!
//! Not installed (the app and every other harness), [`admit`] is one relaxed load.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, RwLock};

use crate::GpuError;

/// What one nominal step of a pass costs on this card, ms, by the pass's `(mode, formula)` (the
/// `IterUniforms` words).
pub type CostFn = dyn Fn(u32, u32) -> f64 + Send + Sync;

/// An installed guard.
pub struct Guard {
    /// The card's price of one nominal step.
    pub cost_ms: Box<CostFn>,
    /// The most one pass may be priced at, ms.
    pub limit_ms: f64,
    /// Refuse a pass priced over `limit_ms`. Off, the guard only prices passes ([`take_peak_ms`]),
    /// which is how a fast card's run records what the suite would ask of a slow one.
    pub enforce: bool,
}

/// A pass the guard refused.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Refusal {
    pub est_ms: f64,
    pub limit_ms: f64,
    pub steps: u64,
    pub mode: u32,
}

static INSTALLED: AtomicBool = AtomicBool::new(false);
static GUARD: RwLock<Option<Guard>> = RwLock::new(None);
static REFUSED: AtomicU64 = AtomicU64::new(0);
static LAST: Mutex<Option<Refusal>> = Mutex::new(None);
/// The dearest pass priced since the last [`take_peak_ms`], as `f64` bits.
static PEAK: AtomicU64 = AtomicU64::new(0);

/// Install `g` for every pass from now on, on every thread.
pub fn install(g: Guard) {
    *GUARD.write().unwrap_or_else(|e| e.into_inner()) = Some(g);
    INSTALLED.store(true, Ordering::Release);
}

/// Remove the guard: every pass is admitted again.
pub fn remove() {
    INSTALLED.store(false, Ordering::Release);
    *GUARD.write().unwrap_or_else(|e| e.into_inner()) = None;
}

/// Whether a pass priced at `est_ms` goes ahead. A price that is not a number refuses: a guard
/// that cannot price a pass has not shown it is safe.
pub fn admits(est_ms: f64, limit_ms: f64, enforce: bool) -> bool {
    !enforce || est_ms <= limit_ms
}

/// Ask before submitting a pass of `steps` nominal steps (pixels x iteration window) in `mode`
/// with `formula`.
pub(crate) fn admit(mode: u32, formula: u32, steps: u64) -> Result<(), GpuError> {
    if !INSTALLED.load(Ordering::Acquire) {
        return Ok(());
    }
    let held = GUARD.read().unwrap_or_else(|e| e.into_inner());
    let Some(g) = held.as_ref() else {
        return Ok(());
    };
    let est_ms = steps as f64 * (g.cost_ms)(mode, formula);
    let mut seen = PEAK.load(Ordering::Relaxed);
    while est_ms > f64::from_bits(seen) {
        match PEAK.compare_exchange_weak(seen, est_ms.to_bits(), Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(now) => seen = now,
        }
    }
    if admits(est_ms, g.limit_ms, g.enforce) {
        return Ok(());
    }
    REFUSED.fetch_add(1, Ordering::AcqRel);
    *LAST.lock().unwrap_or_else(|e| e.into_inner()) = Some(Refusal { est_ms, limit_ms: g.limit_ms, steps, mode });
    Err(GpuError::TooSlowForThisGpu { est_ms, limit_ms: g.limit_ms })
}

/// How many passes the guard has refused in this process.
pub fn refusals() -> u64 {
    REFUSED.load(Ordering::Acquire)
}

/// The latest refusal, if any.
pub fn last_refusal() -> Option<Refusal> {
    *LAST.lock().unwrap_or_else(|e| e.into_inner())
}

/// The dearest pass priced since the last call, ms (0 when none was), and start over.
pub fn take_peak_ms() -> f64 {
    f64::from_bits(PEAK.swap(0, Ordering::Relaxed))
}

#[cfg(test)]
#[path = "../../tests/unit/harness/dispatch_guard/tests.rs"]
mod tests;
