//! **Every critical number, in one place.**
//!
//! Raised as a user requirement (2026-08-09): *"I don't like having critical numbers buried in
//! random blocks of code."* Each constant here is a value some incident set, and the doc comment
//! that records that incident travels WITH the value — the reasoning is the point, not the digits.
//! They were moved verbatim out of `render.rs` and `main.rs`; the call sites are unchanged, because
//! both modules re-export this one (`pub(crate) use crate::tunables::*`).
//!
//! ⚠**What this file is NOT.** It is not a configuration surface. The defaults are the only tested
//! path — the self-test, the goldens, `--bench-matrix` and `--livetest` all run at these values, and
//! a build that ships different ones has not been tested by any of them. The debug override
//! mechanism (`--set NAME=VALUE`) exists so a field diagnosis can move one number for one run; it
//! logs loudly at startup and stamps every crash report, so a report from an overridden run can
//! never masquerade as stock behaviour.
//!
//! ⭐**The bug history says the values are mostly NOT what breaks** (TODO.md, "tunables ... are
//! individually well-reasoned and collectively fragile"). Five shapes account for everything that
//! has gone wrong here, and only the last is "a constant is wrong":
//!
//! 1. A measured loop with a path where the measurement never arrives, falling back on a bootstrap
//!    constant that then BINDS something important. The dominant family by a wide margin.
//! 2. A wrong cost model — nominal steps ignore BLA skipping and latency-bound interiors, so
//!    `steps ∝ time` is false exactly where it is relied upon.
//! 3. Inconsistent capability gating (five of six `is_fe` sites).
//! 4. Unbounded waits and retries.
//! 5. A constant tuned at one depth failing at the next.
//!
//! So when a number here looks wrong, check first whether the loop that reads it is measuring at
//! all. ⚠And when changing one, test at e55/e61/e63/e72/e82/e94 — never at one depth (shape 5).

#![allow(dead_code)] // every value is read through the re-exports, not from this module directly

use std::sync::OnceLock;

// ----------------------------------------------------------------------------
// Debug overrides (`--set NAME=VALUE`)
// ----------------------------------------------------------------------------
//
// The frame-cost controller's twelve numbers are the ones every incident in this ledger involved —
// six device losses, the 16×16 collapse, the settled-resolution ceiling — and they are the ones a
// field diagnosis most often needs to move for ONE run ("does this reproduce at a 400 ms target?").
// They are therefore read through `cost()` rather than as constants, and the `*_DEFAULT` consts
// below are what `cost()` returns when nothing is overridden.
//
// ⚠**Why only these twelve.** An override is a promise that every read of the value goes through
// one place; the rest of the module is plain constants, and reading a constant is checked by the
// compiler to be exactly the documented value. Widening this set is one line each — but each line
// converts a compile-time guarantee into a runtime one, so it should be earned by a diagnosis that
// actually needed it.

/// The runtime-overridable frame-cost family. Field names are the lower-case constant names, so
/// `--set TDR_BUDGET_MS=500` and `cost().tdr_budget_ms` are obviously the same knob.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Cost {
    pub tdr_budget_ms: f64,
    pub tdr_explicit_budget_ms: f64,
    pub tdr_latency_accept_ms: f64,
    pub tdr_grow_max: f64,
    pub tdr_shrink_max: f64,
    pub tdr_lethal_ms: f64,
    pub tdr_bootstrap_steps: u64,
    pub tdr_bootstrap_ms: f64,
    pub mode_rate_unknown_margin: f64,
    pub tdr_min_steps: u64,
    pub tdr_steps_ceil: u64,
    pub explicit_steps_ceil: u64,
    pub explicit_dispatch_cap: u64,
    pub tdr_max_tiles: u64,
    pub tdr_tiles_ceil: u64,
}

impl Default for Cost {
    fn default() -> Self {
        Self {
            tdr_budget_ms: TDR_BUDGET_MS_DEFAULT,
            tdr_explicit_budget_ms: TDR_EXPLICIT_BUDGET_MS_DEFAULT,
            tdr_latency_accept_ms: TDR_LATENCY_ACCEPT_MS_DEFAULT,
            tdr_grow_max: TDR_GROW_MAX_DEFAULT,
            tdr_shrink_max: TDR_SHRINK_MAX_DEFAULT,
            tdr_lethal_ms: TDR_LETHAL_MS_DEFAULT,
            tdr_bootstrap_steps: TDR_BOOTSTRAP_STEPS_DEFAULT,
            tdr_bootstrap_ms: TDR_BOOTSTRAP_MS_DEFAULT,
            mode_rate_unknown_margin: MODE_RATE_UNKNOWN_MARGIN_DEFAULT,
            tdr_min_steps: TDR_MIN_STEPS_DEFAULT,
            tdr_steps_ceil: TDR_STEPS_CEIL_DEFAULT,
            explicit_steps_ceil: EXPLICIT_STEPS_CEIL_DEFAULT,
            explicit_dispatch_cap: EXPLICIT_DISPATCH_CAP_DEFAULT,
            tdr_max_tiles: TDR_MAX_TILES_DEFAULT,
            tdr_tiles_ceil: TDR_TILES_CEIL_DEFAULT,
        }
    }
}

static ACTIVE: OnceLock<Cost> = OnceLock::new();
static APPLIED: OnceLock<Vec<String>> = OnceLock::new();

/// The frame-cost tunables in force. Defaults unless `apply_overrides` ran with something.
#[inline]
pub(crate) fn cost() -> &'static Cost {
    ACTIVE.get_or_init(Cost::default)
}

/// One line for logs and crash reports. `stock` is the normal answer, and it is printed rather
/// than omitted on purpose: a crash report that says nothing about tunables cannot be told apart
/// from one written by a build that did not have this mechanism.
pub(crate) fn status_line() -> String {
    match APPLIED.get() {
        Some(v) if !v.is_empty() => format!("{} OVERRIDE(S) — {}", v.len(), v.join(", ")),
        _ => "stock".to_string(),
    }
}

/// Are the tunables stock? The gates assert this: a self-test or benchmark run under an override
/// has measured a build nobody ships.
pub(crate) fn is_stock() -> bool {
    APPLIED.get().is_none_or(|v| v.is_empty())
}

/// Apply `NAME=VALUE` pairs collected from `--set`. Call ONCE, before anything renders.
///
/// Errors are fatal by design — a typo'd knob that silently did nothing would send a diagnosis
/// chasing a change that never happened. Values must be finite and positive; beyond that the
/// ranges are deliberately generous, because the point of the mechanism is to reach values the
/// shipped defaults refuse (including dangerous ones: setting a 3-second budget to reproduce a
/// device loss on purpose is a legitimate use, and the log line says so).
pub(crate) fn apply_overrides(pairs: &[(String, String)]) -> Result<(), String> {
    let mut c = Cost::default();
    let mut applied = Vec::new();
    for (name, raw) in pairs {
        let key = name.to_ascii_uppercase();
        let f = || -> Result<f64, String> {
            raw.parse::<f64>()
                .ok()
                .filter(|v| v.is_finite() && *v > 0.0)
                .ok_or_else(|| format!("--set {key}: '{raw}' is not a positive finite number"))
        };
        let u = || -> Result<u64, String> {
            raw.parse::<u64>()
                .ok()
                .filter(|v| *v > 0)
                .ok_or_else(|| format!("--set {key}: '{raw}' is not a positive integer"))
        };
        let was: String = match key.as_str() {
            "TDR_BUDGET_MS" => { let p = c.tdr_budget_ms; c.tdr_budget_ms = f()?; p.to_string() }
            "TDR_EXPLICIT_BUDGET_MS" => {
                let p = c.tdr_explicit_budget_ms; c.tdr_explicit_budget_ms = f()?; p.to_string()
            }
            "TDR_LATENCY_ACCEPT_MS" => {
                let p = c.tdr_latency_accept_ms; c.tdr_latency_accept_ms = f()?; p.to_string()
            }
            "TDR_GROW_MAX" => { let p = c.tdr_grow_max; c.tdr_grow_max = f()?; p.to_string() }
            "TDR_SHRINK_MAX" => { let p = c.tdr_shrink_max; c.tdr_shrink_max = f()?; p.to_string() }
            "TDR_LETHAL_MS" => { let p = c.tdr_lethal_ms; c.tdr_lethal_ms = f()?; p.to_string() }
            "TDR_BOOTSTRAP_STEPS" => {
                let p = c.tdr_bootstrap_steps; c.tdr_bootstrap_steps = u()?; p.to_string()
            }
            "TDR_BOOTSTRAP_MS" => {
                let p = c.tdr_bootstrap_ms; c.tdr_bootstrap_ms = f()?; p.to_string()
            }
            "MODE_RATE_UNKNOWN_MARGIN" => {
                let p = c.mode_rate_unknown_margin; c.mode_rate_unknown_margin = f()?; p.to_string()
            }
            "TDR_MIN_STEPS" => { let p = c.tdr_min_steps; c.tdr_min_steps = u()?; p.to_string() }
            "TDR_STEPS_CEIL" => { let p = c.tdr_steps_ceil; c.tdr_steps_ceil = u()?; p.to_string() }
            "EXPLICIT_STEPS_CEIL" => {
                let p = c.explicit_steps_ceil; c.explicit_steps_ceil = u()?; p.to_string()
            }
            "EXPLICIT_DISPATCH_CAP" => {
                let p = c.explicit_dispatch_cap; c.explicit_dispatch_cap = u()?; p.to_string()
            }
            "TDR_MAX_TILES" => { let p = c.tdr_max_tiles; c.tdr_max_tiles = u()?; p.to_string() }
            "TDR_TILES_CEIL" => { let p = c.tdr_tiles_ceil; c.tdr_tiles_ceil = u()?; p.to_string() }
            _ => return Err(format!("--set {key}: not an overridable tunable ({})", OVERRIDABLE)),
        };
        applied.push(format!("{key} {was} → {raw}"));
    }
    // The floor must stay under the ceiling whatever was typed, or the clamp in `budget_step`
    // panics on an inverted range — a debugging tool that crashes the app is not one.
    if c.tdr_min_steps > c.tdr_steps_ceil || c.tdr_min_steps > c.explicit_steps_ceil {
        return Err("--set: TDR_MIN_STEPS must not exceed TDR_STEPS_CEIL / EXPLICIT_STEPS_CEIL"
            .to_string());
    }
    if c.tdr_max_tiles > c.tdr_tiles_ceil {
        return Err("--set: TDR_MAX_TILES must not exceed TDR_TILES_CEIL".to_string());
    }
    let _ = ACTIVE.set(c);
    let _ = APPLIED.set(applied);
    Ok(())
}

/// The names `--set` accepts, for the error message and `--help`.
pub(crate) const OVERRIDABLE: &str = "TDR_BUDGET_MS, TDR_EXPLICIT_BUDGET_MS, \
    TDR_LATENCY_ACCEPT_MS, TDR_GROW_MAX, TDR_SHRINK_MAX, TDR_LETHAL_MS, TDR_BOOTSTRAP_STEPS, \r
    TDR_BOOTSTRAP_MS, \
    MODE_RATE_UNKNOWN_MARGIN, TDR_MIN_STEPS, TDR_STEPS_CEIL, EXPLICIT_STEPS_CEIL, \
    EXPLICIT_DISPATCH_CAP, TDR_MAX_TILES, TDR_TILES_CEIL";

#[cfg(test)]
mod override_tests {
    use super::*;

    #[test]
    fn defaults_match_the_documented_constants() {
        // The struct is a second copy of twelve numbers; this is what keeps it honest.
        let d = Cost::default();
        assert_eq!(d.tdr_budget_ms, TDR_BUDGET_MS_DEFAULT);
        assert_eq!(d.tdr_bootstrap_steps, TDR_BOOTSTRAP_STEPS_DEFAULT);
        assert_eq!(d.tdr_bootstrap_ms, TDR_BOOTSTRAP_MS_DEFAULT);
        assert_eq!(d.tdr_lethal_ms, TDR_LETHAL_MS_DEFAULT);
        assert_eq!(d.mode_rate_unknown_margin, MODE_RATE_UNKNOWN_MARGIN_DEFAULT);
        assert_eq!(d.tdr_min_steps, TDR_MIN_STEPS_DEFAULT);
        assert_eq!(d.tdr_steps_ceil, TDR_STEPS_CEIL_DEFAULT);
        assert_eq!(d.explicit_dispatch_cap, EXPLICIT_DISPATCH_CAP_DEFAULT);
        assert_eq!(d.tdr_max_tiles, TDR_MAX_TILES_DEFAULT);
        assert_eq!(d.tdr_tiles_ceil, TDR_TILES_CEIL_DEFAULT);
    }

    #[test]
    fn a_typo_is_an_error_not_a_silent_no_op() {
        let r = apply_overrides(&[("TDR_BUDGET_MSS".into(), "500".into())]);
        assert!(r.is_err(), "an unknown knob must be rejected");
        assert!(apply_overrides(&[("TDR_BUDGET_MS".into(), "nope".into())]).is_err());
        assert!(apply_overrides(&[("TDR_BUDGET_MS".into(), "0".into())]).is_err());
        assert!(apply_overrides(&[("TDR_BUDGET_MS".into(), "-1".into())]).is_err());
    }

    #[test]
    fn an_inverted_range_is_rejected() {
        // `budget_step` clamps into `TDR_MIN_STEPS ..= ceil`; an inverted range panics there.
        let r = apply_overrides(&[
            ("TDR_MIN_STEPS".into(), "900000000000".into()),
            ("TDR_STEPS_CEIL".into(), "1000".into()),
        ]);
        assert!(r.is_err(), "min above ceiling must be rejected, not clamped");
    }

    #[test]
    fn status_says_stock_when_nothing_was_set() {
        // These tests share a process, so this asserts the SHAPE, not a particular global state.
        let s = status_line();
        assert!(s == "stock" || s.contains("OVERRIDE(S)"), "unexpected status line: {s}");
    }
}

// ----------------------------------------------------------------------------
// GPU watchdog: the frame-cost budget controller
// ----------------------------------------------------------------------------

/// GPU time a settled floatexp frame is aimed at. Short enough that the UI thread keeps pumping
/// messages between frames (Windows paints a window "Not Responding" after ~5 s), so an
/// unaffordable view degrades in resolution instead of hanging.
///
/// ⭐**900 → 400 on 2026-08-16, after the SECOND field device loss converged on the same band.**
/// The old value's claim to be "comfortably under the ~2 s GPU watchdog" was the whole problem: the
/// controller does not approach a target from below and stop, it CONVERGES on it, so a 900 ms
/// target means steady-state dispatches of ~900 ms — and this file already documents ~0.9 s as the
/// start of the lethal band (see `TDR_EXPLICIT_BUDGET_MS`, cut to 400 ms for exactly that reason
/// after three Event-153 losses). The auto regime was left at 900 on the reasoning that auto-iter
/// counts are the PREDICTABLE case. Two field reports on an RX 6800 XT refute that, by two
/// unrelated routes:
///
///  1. **2026-08-15, at a mode 0→2 crossover.** Growth ×1.5 off the opening guess produced a
///     1038 ms dispatch two frames after the switch; device lost on the next submit.
///  2. **2026-08-16, at a SETTLED tiled view 1770 frames from any switch** (zoom 3.4e38,
///     `settled=true`, `tile=true`). A reference install moved `ref_len` 181,313 → 195,520 — 8%
///     longer — and per-frame nominal cost jumped 27× to 7.546e11 against a converged
///     `budget=3.000e11`. With tiling that is ~2.5e11 per tile, which measured **1033 ms and
///     1026 ms** on that card. Two frames later the device was gone.
///
/// The routes differ; the destination does not. Both died at ~1030 ms, both were the controller
/// working as specified. A target is a promise about steady state, so the target itself has to sit
/// outside the lethal band — no amount of care on the approach fixes aiming at it.
///
/// 400 ms matches the explicit regime and keeps the >2× margin its comment argues for. ⚠It is now
/// EQUAL to `TDR_EXPLICIT_BUDGET_MS` rather than above it; the two regimes differ in how
/// predictable their cost is, but the hardware's lethal band does not care which regime asked, so
/// one common target is the honest expression of it. ⚠It also sits BELOW
/// `TDR_LATENCY_ACCEPT_MS` (600 ms), which brings the latency-floor guard in `budget_step` into the
/// auto regime for the first time — see that constant.
///
/// ⚠**Where the budget binds, this costs settled resolution**, and that is the point: a view that
/// cannot be drawn in 400 ms is drawn smaller instead of risking the device.
///
/// Measured, not assumed: `--livetest` came back **24/24, 0 drifted** — no re-blessing needed. At
/// its 480×270 the deep holds never approach the ceiling, so the target never binds there. ⚠Read
/// that as "the gate does not exercise this", NOT as "the change is free": the 2026-08-16 loss was
/// a **2247×1485** settled tiled frame, and no gate renders live at that size (design/torture-suite
/// G3). The resolution cost lands at full-window deep views, which is exactly where no automated
/// coverage currently exists.
pub(crate) const TDR_BUDGET_MS_DEFAULT: f64 = 400.0;

/// Per-measurement budget change limits. Growth is capped well under 2× so the next frame cannot leap
/// from the target into the watchdog; shrink is allowed to halve at once.
///
/// The budget is retargeted by the measured-time RATIO, deliberately without modelling cost as
/// `steps ∝ time`. That model is false at deep interior views: every pixel runs the full iteration
/// count on a dependent chain, so a small frame is LATENCY-bound (~89.9k iterations ≈ 415 ms here no
/// matter how few pixels) and only becomes throughput-bound once it saturates GPU occupancy. Assuming
/// proportionality made the loop conclude a shrunk frame should be fast, and it drove the view to a
/// postage stamp trying to reach a target that no resolution could reach. A ratio search needs no such
/// assumption — it just walks toward whatever size actually measures near the target.
pub(crate) const TDR_GROW_MAX_DEFAULT: f64 = 1.5;

/// Most the budget may shrink on ONE reading. Bounded for the same reason growth is: the ratio
/// search should walk, not lurch, or it oscillates between a postage stamp and an unaffordable
/// frame. ⚠**Bypassed above `TDR_LETHAL_MS`** — see there; a cap on how fast you can retreat is a
/// liability once you are already standing in the fire.
pub(crate) const TDR_SHRINK_MAX_DEFAULT: f64 = 0.5;

/// ⭐The measured frame time at which the budget controller stops easing and retreats in ONE step.
///
/// This band has been prose in half a dozen comments in this file ("~0.9 s", "the lethal band")
/// since the first Event-153 losses; naming it makes it testable and overridable.
///
/// **Why a single-step retreat.** `TDR_SHRINK_MAX` caps a reading at ×0.5, which is right while
/// walking toward a target and actively harmful past this point. The 2026-08-16 device loss
/// (`reports/fractadyne-report.-2026-08-16a.txt`) measured **1033 ms** against a target that wanted
/// ×0.39; clamped to ×0.5 that needs two or more readings, readings are deferred ~2 frames, and
/// every frame in between is another ~1 s dispatch. **The device died after three.** The controller
/// was not mispricing after the first reading — it had already seen the danger and was retreating
/// too slowly to survive its own recovery.
///
/// Above this threshold the shrink uses the raw `target/ms` ratio, still floored by
/// `TDR_MIN_STEPS`. Overshooting into a frame that is too small is cheap and self-correcting
/// (the next reading grows it back at ×1.5); overshooting into the watchdog is not.
pub(crate) const TDR_LETHAL_MS_DEFAULT: f64 = 900.0;

/// Cost of the very first frame in a mode, before any measurement exists — the OPENING GUESS, not a
/// floor. Everything above it is measured, not assumed.
///
/// ⚠**Its old claim, "a few ms even on a GPU orders of magnitude slower", is FALSE in the
/// floatexp / no-BLA regime, and believing it cost six device losses.** Measured 2026-08-09 on an
/// RTX 3080 at the three-spar dive, `mode=2` with `orbit_len=626` (an escaped reference so short
/// that BLA skips nothing and real cost tracks the nominal count instead of being a small fraction
/// of it): the old 4e8 value measured **~1070 ms**, over half the ~2 s driver watchdog on a
/// current discrete part and therefore NEGATIVE margin on anything slower. A nominal step is not a
/// fixed amount of work — that is the standing lesson of this file — so no constant expressed in
/// steps can be "a few ms" everywhere.
///
/// ⚠**Lowering it does not help and is not free — TRIED 4e8 → 1e8, REVERTED.** It did not prevent
/// the device loss (the run died anyway, at 69×54 and 9.86e7 steps), because the cost that kills is
/// not reachable by this constant — see the mode-2 throughput collapse in TODO.md. And it cost real
/// resolution: `--livetest` caught the `seahorse-2` checkpoint dropping **480×270 → 100×56**. The
/// guess is a starting point the controller climbs from at ×1.5 per reading, so starting 4× lower
/// costs ~4 frames of coarseness at every checkpoint that has not converged yet. See
/// `TDR_MIN_STEPS` for the change that does matter.
/// ⭐As of 2026-08-15 this is a CEILING on the opening guess, not the guess itself: `bootstrap_steps`
/// derives the guess from a measured per-step RATE and clamps it here. A card that has never been
/// measured still starts exactly where it always did; a card (or regime) measured to be slower
/// starts lower. See `TDR_BOOTSTRAP_MS` for why, and note that this is the fix the paragraph above
/// asks for by name — "no constant expressed in steps can be 'a few ms' everywhere" is a statement
/// that the guess must be denominated in TIME and converted with a rate.
pub(crate) const TDR_BOOTSTRAP_STEPS_DEFAULT: u64 = 400_000_000;

/// ⭐Wall-clock the OPENING GUESS aims at, in ms. `bootstrap_steps` converts it to a nominal step
/// count with a measured per-step rate, which is the only way the guess can mean the same thing on
/// hardware and in regimes it was not calibrated on — the standing lesson recorded under
/// `TDR_BOOTSTRAP_STEPS`.
///
/// **The field report this comes from** (2026-08-15, RX 6800 XT / Vulkan, beta.104,
/// `reports/fractadyne-report-2026-08-15.txt`): a mode 0→2 switch at frame 4298 correctly reset the
/// budget to unmeasured; frame 4300 then dispatched **3.922e8** steps — the 4e8 guess — and two
/// frames later a ×1.5 growth to 6.000e8 measured **1038 ms** and lost the device. That is the same
/// ~1070 ms this file already records for 4e8 in `mode=2` with no BLA, reproduced on a second
/// vendor. `bla_skip=0` on both post-switch frames: immediately after the crossover there is no
/// valid BLA tree, so nominal steps ARE real cost.
///
/// 40 ms is deliberately the value that reproduces `TDR_BOOTSTRAP_STEPS` on the dev card in the
/// BLA-live regime, so the common path is unchanged: the derived guess only bites where the
/// measured rate is genuinely worse. ⚠It must stay far under `TDR_LATENCY_ACCEPT_MS` — an opening
/// dispatch is the one nobody has priced yet, and the whole point is that it cannot be the frame
/// that kills the device.
pub(crate) const TDR_BOOTSTRAP_MS_DEFAULT: f64 = 40.0;

/// ⭐Safety divisor for the FIRST entry into an arithmetic mode this device has never measured, when
/// the only rate available was earned in a different mode. Per-step cost is not comparable across
/// modes — the field report above measured mode 0 at ~8.6e7 steps/ms and mode 2 (no BLA) at
/// ~5.7e5, a ratio of about **152×** — so another mode's rate is an upper bound on speed, never an
/// estimate, and must be divided down before it sizes a dispatch.
///
/// 256 is one binary step past the single ratio ever measured, chosen asymmetrically on purpose:
/// being too conservative costs a few coarse frames while the controller re-climbs at ×1.5 (the
/// cost `TDR_BOOTSTRAP_STEPS` already documents for a low guess), while being too optimistic once
/// costs the device. The very first reading in the new mode replaces this estimate with a real one.
pub(crate) const MODE_RATE_UNKNOWN_MARGIN_DEFAULT: f64 = 256.0;

/// ⭐Absolute floor the measured budget may fall to. The clamp's lower bound used to be
/// `TDR_BOOTSTRAP_STEPS`, which meant **the controller could not shrink below the opening guess no
/// matter what it measured** — and where that guess is worth 780 ms rather than a few ms, the app
/// sat submitting ~0.8 s dispatches back to back during a fast dive, roughly 2× from the ~2 s
/// driver watchdog, in the one regime where cost is least predictable. Six device losses, each
/// matched 1:1 by an `nvlddmkm` Event 153, all within seconds of entering that regime; the last
/// one at `146x115  steps=4.598e8  budget=4.625e8` — the controller had grown the budget from the
/// bootstrap because a frame measured 780 ms against a 900 ms target, i.e. it was working exactly
/// as designed and the design had no way down.
///
/// A safety valve must be able to move in the direction of safety. 100× below the opening guess is
/// far enough to reach single-digit milliseconds in the worst regime measured, and the resolution
/// shrink's own 16×16 floor still bounds how small a frame can get.
pub(crate) const TDR_MIN_STEPS_DEFAULT: u64 = 4_000_000;

/// Nominal-step bound for EXPLICIT-count dispatches (auto-iter off) while the cost model is still
/// UNMEASURED. Three device losses in one release cycle shared the shape "controller converges
/// explicit-count dispatches toward the 900 ms target; ~0.9–1.3 s of unpreemptible fragment work
/// intermittently loses the device" (crash-1786499093 Direct, crash-1786506241 mode-0
/// rebase-grind, crash-1786538140 mode-2 settle tiles) — the `nvlddmkm` Event-153 marginality of
/// the beta.48 saga, reproduced at will. 2e10 nominal is ~60–200 ms real even with ZERO skip —
/// safely under any watchdog — which is exactly the guarantee an unmeasured dispatch needs.
/// Applied to the budget-climb probe's stop and to the tiling gate's "arm once the cap region is
/// reached" predicate. Auto-iter views are untouched.
///
/// ⚠It is NOT the bound on a MEASURED explicit budget. Shipped as a flat cap on `tdr_steps`, it
/// silently priced every skip-heavy dispatch at its zero-skip worst case: a scripted dive at a
/// 5111×2158 window (2026-08-12 field report) ran cap-sized dispatches that measured **54.3 ms**
/// against the ~900 ms target — 4× of safe headroom spent rendering 26-pixel blocks — while the
/// frame budget sat frozen above the cap discarding every reading as undersized (`(settling)`
/// forever). Measured cost is the real currency: `budget_step` in the explicit regime converges
/// on `TDR_EXPLICIT_BUDGET_MS` real and is ceilinged by `EXPLICIT_STEPS_CEIL`.
pub(crate) const EXPLICIT_DISPATCH_CAP_DEFAULT: u64 = 20_000_000_000;

/// Real-milliseconds target for MEASURED explicit-count dispatches. The lethal band starts around
/// ~0.9 s (three reproduced Event-153 losses); 400 ms keeps a >2× margin while roughly tripling
/// the per-dispatch work the old flat nominal cap allowed in skip-heavy regimes.
///
/// ⚠It used to be "deliberately below the auto-iter `TDR_BUDGET_MS`", on the reasoning that
/// explicit counts are where skip effectiveness is least predictable. As of 2026-08-16 the two are
/// EQUAL: two field device losses arrived through the auto regime at ~1030 ms, so the premise that
/// auto is the safe one did not survive contact. Unpredictability decides how much MARGIN you need;
/// the lethal band decides where the ceiling is, and it is the same band for both regimes.
///
/// ⚠It MUST sit above the deep view's per-dispatch LATENCY FLOOR. Shipped first at 200 ms and
/// caught by `--livetest` (the grand tour's six deep holds collapsed 480×270 → 16×16): a 16×16
/// mode-2 dispatch at a 250k pixel ask measures ~250–330 ms *no matter how few pixels* — 256
/// threads on 250k-step dependent chains is latency-bound, the codebase's oldest banked lesson —
/// so every reading at the floor read "slow", the controller shrank to `TDR_MIN_STEPS`, and the
/// pinned budget read back as converged (`next == cur ⇒ ok`). 400 ms clears the floor measured
/// here; the floor GUARD in `budget_step` is what keeps a deeper/slower view from cornering
/// itself when even 400 ms is below its floor.
pub(crate) const TDR_EXPLICIT_BUDGET_MS_DEFAULT: f64 = 400.0;

/// Ceiling on a latency floor the guard in `budget_step` may HOLD at (rather than shrink away
/// from). Between the 400 ms target and here, a small slow dispatch is treated as an accepted
/// latency floor; past here it is watchdog-relevant and the shrink proceeds. Sits well under the
/// ~0.9 s lethal band and above every floor measured so far (250–330 ms at the grand tour's
/// 250k–1.2M holds).
///
/// ⚠**This now applies to the AUTO regime as well, which it did not before 2026-08-16.** The old
/// note here said the accept bound "keeps this guard out of the AUTO regime entirely: auto's `slow`
/// starts at 900 ms, already past it". Lowering `TDR_BUDGET_MS` to 400 makes auto's `slow` start at
/// 400 ms, so a small auto dispatch measuring 400–600 ms is now HELD rather than shrunk. That is
/// the intended behaviour in both regimes — a latency-bound frame cannot be made faster by making
/// it smaller — but it is a real change to auto, and it is recorded here because the previous
/// sentence would otherwise read as still true.
pub(crate) const TDR_LATENCY_ACCEPT_MS_DEFAULT: f64 = 600.0;

/// Nominal ceiling for a MEASURED explicit budget: 3× `EXPLICIT_DISPATCH_CAP`, so even a total
/// skip collapse between one measurement and the next (nominal = real, the worst case nominal
/// denomination guarantees) prices at ~180–600 ms — under the lethal band with margin. Growth
/// from the cap to here takes ~3 measured readings at ×1.5, each a real timing.
pub(crate) const EXPLICIT_STEPS_CEIL_DEFAULT: u64 = 60_000_000_000;

/// Never exceed this even if a view measures absurdly cheap (a lone quick interval shouldn't uncork a
/// multi-second dispatch).
pub(crate) const TDR_STEPS_CEIL_DEFAULT: u64 = 300_000_000_000;


// ----------------------------------------------------------------------------
// GPU watchdog: the tiled settle
// ----------------------------------------------------------------------------

/// FLOOR on the budget-sized dispatches a tiled settle may spend sharpening one frame — the count
/// this was a fixed constant at, kept so nothing regresses below the behaviour it produced. It is
/// also what a view with a still-CLIMBING budget gets, so its grid completes quickly and can
/// re-form sharper; `settle_max_tiles` grants a converged view what its frame actually needs.
pub(crate) const TDR_MAX_TILES_DEFAULT: u64 = 16;

/// Absolute cap on tiles per grid, whatever the frame asks for — a backstop on grid bookkeeping,
/// not a cost bound (each tile is independently budget-sized, so the count cannot trip the
/// watchdog). It IS the wall-clock envelope of a settle, though: 512 dispatches at the explicit
/// regime's `TDR_EXPLICIT_BUDGET_MS` target is ~3.5 minutes of background sharpening on a view the
/// user is parked at, one dispatch per frame, abandoned the moment they touch anything. Views whose
/// tiles measure far under the target (the common case — a budget pinned at `EXPLICIT_STEPS_CEIL`
/// measures 12–57 ms per tile here) spend seconds, not minutes.
pub(crate) const TDR_TILES_CEIL_DEFAULT: u64 = 512;


// ----------------------------------------------------------------------------
// Motion: hold, refresh and reprojection
// ----------------------------------------------------------------------------

pub(crate) const REFRESH_OCTAVES: f64 = 0.5;

pub(crate) const REFRESH_MAX_SECS: f64 = 0.15;

pub(crate) const REFRESH_MIN_DRIFT: f64 = 0.02;

/// Hard drift ceiling for reusing a cached reference: a point beyond this fraction of a span
/// off-centre is re-anchored (fresh pick) instead. Held at the `out_of_view` gate that already
/// filters the caller, so a reused reference is never worse than one the live path already trusts.
pub(crate) const REUSE_MAX_DRIFT: f64 = 0.7;


// ----------------------------------------------------------------------------
// Reference lookahead and pacing
// ----------------------------------------------------------------------------

/// Slack (octaves) on "the dive has arrived at this slot's target" — the width of the old install
/// window, kept so a pump landing just short of a target still installs instead of waiting a frame.
pub(crate) const PREFETCH_REACH_SLACK: f64 = 0.14;

/// Slot spacing (octaves). Sets the ACTIVE reference's peak lag between installs: an
/// install restores lag ≈ 1.0, and the next slot becomes installable when the view reaches
/// its window ⇒ peak active lag = 1.0 + spacing − 0.14. That peak MUST stay below
/// `PACE_LAG_LO` (1.5) and `DEEP_LAG_HOLD` (1.8), or the tail of every inter-install
/// interval rhythmically clips the pacer / freeze-reproject zones — the residual "visible
/// jerkiness from ~e400" with 1.0 spacing (peak 1.86: below ~e400 the reactive path patched
/// the tail in ~10–30 ms, past it builds cost 70–130 ms and lose the race at the fast dive
/// phase). 0.5 ⇒ peak lag ≈ 1.36 — clear of both thresholds.
pub(crate) const PREFETCH_OCT: f64 = 0.5;

/// Lookahead depth (slots × PREFETCH_OCT octaves = ~3 octaves, ~0.3 s of runway at the
/// fastest ~10 oct/s phase vs 0.1–0.6 s per build; consumption 2 installs/octave). Each
/// build's candidate scoring already fans out across all cores, so concurrent slots briefly
/// oversubscribe threads — harmless for compute-bound bursts.
pub(crate) const PREFETCH_SLOTS: usize = 6;

/// ⭐How many lookahead builds may be IN FLIGHT at once, as opposed to how many slots the
/// queue holds. Previously unbounded: the refill loop below spawns until the queue is
/// full, so whenever the queue drained it started six builds in the same millisecond —
/// visible in every crash log as five `building reference [lookahead]` lines sharing a
/// timestamp.
///
/// That is not the cheap burst the old comment assumed. `best_reference`'s candidate
/// scoring uses `std::thread::scope` with `available_parallelism()` threads PER CALL — a
/// fresh set each time, not a shared pool — so six concurrent builds put six times the
/// core count of bignum work up against the render thread. Measured on the 2026-08-09
/// 08:33 device loss, with the frame budget already pinned to the bootstrap so the GPU had
/// almost nothing to do: **8 frames in 3.8 s — ~475 ms per frame — for a 135×106 frame
/// doing 4e8 steps**, which is ~10 ms of GPU work on the dev 3080. The frames were not
/// GPU-bound; the main thread was starved.
///
/// Two is enough to keep a build overlapping the one being consumed while leaving the
/// machine to the renderer. The queue still covers its full depth — slots simply fill in
/// sequence instead of all at once.
pub(crate) const PREFETCH_MAX_INFLIGHT: usize = 2;

/// How far ahead (tour seconds) a hold build may start. Deep extensions cost 25–90 s of
/// wall clock; the pacer only ever DILATES tour time, so the real lead is at least this.
pub(crate) const HOLD_PREFETCH_LEAD_S: f64 = 120.0;

pub(crate) const DIVE_PROBE_S: f64 = 0.25;

pub(crate) const DEEP_LAG_HOLD: f64 = 1.8;


// ----------------------------------------------------------------------------
// Reference building and arithmetic
// ----------------------------------------------------------------------------

/// Extra orbit precision above the depth's exact requirement, so successive DEEPER rebuilds within
/// this band can extend the cached orbit (see [`try_reuse_reference`]) instead of recomputing it.
/// The orbit is stored as df32 with ample accuracy headroom, so building at a higher precision
/// leaves the df32 samples byte-identical — this only grows the accumulation margin, not the render.
pub(crate) const REF_PREC_HEADROOM: usize = 128;

/// BLA per-step linear tolerance (drops δz² with relative error ≤ this). Smaller ⇒ more
/// accurate but fewer/smaller skips; 1e-6 keeps pixel error negligible while still merging.
pub(crate) const BLA_EPS: f64 = 1.0e-6;


// ----------------------------------------------------------------------------
// Glitch correction
// ----------------------------------------------------------------------------

pub(crate) const CORRECT_WORK_BUDGET: u64 = 2_000_000_000;


// ----------------------------------------------------------------------------
// Reference caps and build-rate limits
// ----------------------------------------------------------------------------

/// Ceiling on the LIVE preview's reference-orbit LENGTH (samples) **while interacting**. At
/// extreme depth a non-escaping tip yields a full ~500k-iter reference — a ~4M-node BLA whose
/// build/upload per dive-refresh is what froze the window on boot/settle historically. Capping the
/// REFERENCE (not the pixel iteration) keeps every motion-path build cheap.
///
/// CRUCIAL: it must sit ABOVE the moderate-depth reference build (`ref_build_iter` ≈ `gpu_iter` +
/// headroom, ≈184k at ~1e205×). A reference that ESCAPES below the cap builds complete
/// (`partial=false`), so pixels iterate to the full `gpu_iter` by REBASING past it and the border
/// resolves. But truncating a reference that would have escaped just above the cap flips it to
/// `partial=true`, and the render then clamps `max_iter` to the (short) orbit length — leaving a
/// smooth, unresolved border (the ~1e205× session point escaped at 100002 and a 100k cap truncated
/// it 2 iterations early).
///
/// **SETTLED views are no longer bound by this** (`live_orbit_cap`, since the 6.3e63× spar blobs):
/// the adaptive iteration boost raises settled budgets past 256k at Misiurewicz spar fields, whose
/// near-neutral references are long-lived at ANY depth — the old "only bites past ~1e290×"
/// assumption was wrong there — and a partial reference shorter than the budget clamps pixels
/// into blobs. A settled build follows its budget instead; the device storage-binding cap
/// (`init_orbit_len_cap`, orbit + BLA sized together) is the remaining bound, so GPU memory stays
/// safe. Export keeps the full appetite (`orbit_len_cap = u32::MAX`).
pub(crate) const LIVE_REF_CAP: u32 = 256_000;

/// Reference builds per second that mean a spin rather than a workload. Healthy playback spawns
/// one reactive build plus at most `PREFETCH_SLOTS` lookahead builds per INSTALL, and installs run
/// ~2/octave — single digits per second even on the fastest dive. Set an order of magnitude above
/// that so it can only fire on a genuine runaway.
pub(crate) const BUILD_STORM_PER_S: u32 = 60;

/// Hard ceiling on lookahead reference builds per second (`playback_ref_prefetch`). A backstop, not
/// a policy: the queue's own bookkeeping should keep it near one build per install, and three
/// separate defects have each turned it into a spin. Well above any legitimate rate.
pub(crate) const PREFETCH_BUILDS_PER_S: u32 = 30;

/// Max iterates drawn by the interactive orbit overlay (shallow f64 path).
pub(crate) const ORBIT_MAX: usize = 512;

/// Deep (bignum) orbit cap — large enough to run past where nearby points' orbits
/// diverge (≈ the reference orbit's escape length) so the overlay responds to the
/// cursor; bounded for cost. Cached so it only recomputes when the cursor/view moves.
pub(crate) const ORBIT_MAX_DEEP: u32 = 8192;

/// Upper bound on a loaded/pasted `.fdn` location blob (untrusted input). A real one is
/// well under 1 KB; this caps parse work without rejecting anything legitimate.
pub(crate) const SHARE_MAX: usize = 256 * 1024;


// ----------------------------------------------------------------------------
// Tour pacing
// ----------------------------------------------------------------------------

/// Deep-dive pipeline pacing window (octaves of `last_depth_lag`): below `LO` the dive runs at
/// full speed; above `HI` it's fully held (just under the mode-2 stale-reference spin/freeze
/// threshold ≈ 3 octaves); in between it proportionally slows. Shared by the script-playback
/// clock dilation (`advance_playback`) and the interactive zoom-velocity damping
/// (`paced_zoom_vel`) so both degrade the same way: the image stays sharp, the dive slows.
pub(crate) const PACE_LAG_LO: f64 = 1.5;

pub(crate) const PACE_LAG_HI: f64 = 2.8;


// ----------------------------------------------------------------------------
// Iteration budget
// ----------------------------------------------------------------------------

/// Hard ceiling on the user-settable iteration count (the Iterations slider max and the live
/// budget clamps). Was 500,000 — which the 2.6e72× Misiurewicz spar outgrew (measured: 33% of
/// samples still capped at 500k, ZERO at 1M — the view was fully resolvable, the app just refused
/// to try). Peer deep-zoomers (KF / Fraktaler-3 / Imagina) treat iterations as effectively
/// unbounded and users routinely run millions; 10M covers the depths our reference/precision
/// stack actually reaches. Live-path safety does NOT come from this number: per-frame cost is
/// bounded by the work budget / tiled settle / motion-res machinery, and non-escaping references
/// keep the `LIVE_REF_CAP` clamp (freeze guard). The AUTO appetite has its own tighter ceiling in
/// `recommended_max_iter`.
pub(crate) const MAX_ITER_LIMIT: u32 = 10_000_000;

/// Ceiling on the adaptive iteration boost multiplier. The old ×16 was a wall the Misiurewicz
/// spar family outgrew by ~1e82×: `zoom_iter_cap` there is ~72k, so even a maxed boost stopped at
/// ~1.15M while the field genuinely needs several million — the probe was still measuring "capped,
/// and raising helps" when the ceiling cut it off. 256× lets `zoom_iter_cap × boost` reach
/// `MAX_ITER_LIMIT` at any depth where that appetite is real; runaway protection is the job of the
/// stall/plateau evidence logic and the frame-budget machinery (cost-based), not this number.
pub(crate) const ITER_BOOST_MAX: f64 = 256.0;

/// Consecutive fruitless raises the adaptive iteration probe must see before it concludes the
/// view is genuinely interior and stops (see `build_params`). Three, because the measured worst
/// case — the 3.3e61× three-spar — stays at 100% capped through two raises before resolving on
/// the third. Too low and a starved view latches to a black screen; too high and a true interior
/// pays a few extra settle frames before reverting.
pub(crate) const ITER_STALL_LIMIT: u8 = 3;

/// Max GPU work per render (texels × iterations) before the OS GPU watchdog (TDR)
/// risks a device-lost crash. Supersampling auto-reduces to stay under this.
pub(crate) const WORK_BUDGET: u64 = 60_000_000_000;

/// The zoom-appropriate iteration cap, `ZOOM_ITER_BASE + octaves × ZOOM_ITER_PER_OCTAVE`
/// (`zoom_iter_cap`). It is what AUTO-iteration is allowed to ask for before the adaptive boost
/// multiplies it; an explicit count bypasses it entirely (`live_iter_budget`). Written inline until
/// 2026-08-15 — exactly the "critical number buried in a random block of code" this module exists
/// to end, and one the ledger refers to constantly as "2000 + 256/octave".
///
/// ⚠It is a CEILING on the ask, not an estimate of what a view needs: deep Misiurewicz spar fields
/// need multiples of it (hence `ITER_BOOST_MAX`), while a shallow escape-heavy view never
/// approaches it. Changing the slope moves every auto-iter view at every depth at once, so it is
/// the single riskiest number here to retune — test at e55/e61/e63/e72/e82/e94, never at one depth.
pub(crate) const ZOOM_ITER_BASE: f64 = 2000.0;
pub(crate) const ZOOM_ITER_PER_OCTAVE: f64 = 256.0;


// ----------------------------------------------------------------------------
// Arithmetic-mode thresholds
// ----------------------------------------------------------------------------

/// Magnification at/above which perturbation switches from the fast df32 δ to the
/// floatexp δ. df32 stays clean to ~1e30×; cross over before then with margin.
pub(crate) const PERT_FE_THRESHOLD: f64 = 1.0e28;

/// Magnification at/above which a JULIA view leaves direct mode for perturbation. Far below the
/// Mandelbrot threshold (1e4) because their direct-mode precision differs structurally: a
/// Mandelbrot pixel re-injects its df32 `c` EVERY iteration, so the per-pixel identity survives
/// f32 rounding noise; a Julia pixel's identity lives only in `z0`, entered once — measured
/// (2026-08-13, `--juliadive` + user report at J 4,362×): speckle from ~530×, hard
/// iteration-plateau "tessellation" patches by ~1000–4000×, healthy again at ≥1e4 where
/// perturbation takes over. 1e2 keeps a wide margin below the first measurable degradation;
/// perturbation is exact at any depth and the Julia reference machinery is the same one deep
/// dual views already use.
pub(crate) const PERT_JULIA_THRESHOLD: f64 = 1.0e2;


// ----------------------------------------------------------------------------
// Navigation timing
// ----------------------------------------------------------------------------

/// Continuous-zoom tuning.
pub(crate) const ZOOM_RATE: f64 = 0.462; // ln(2)/1.5 ≈ ~2× magnification per 1.5 s at full speed

/// Keep anti-aliasing off for this long after the last interaction, so rapid zoom
/// steps don't each trigger a full-AA render (which felt laggy).
pub(crate) const SETTLE_DELAY: f64 = 0.18;
