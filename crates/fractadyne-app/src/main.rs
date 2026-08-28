//! Fractadyne — native fractal explorer (the desktop binary).
//!
//! # Where things live (crate layering)
//!
//! The workspace separates the logical layers into crates (so the numerics and GPU code are
//! reusable and testable independently of the UI):
//!
//! - `fractadyne-core`  — numerics: arbitrary-precision [`Viewport`], reference orbits,
//!   perturbation/`FloatExp` scale, series approximation, minibrot finder, parsers.
//! - `fractadyne-gpu`   — wgpu pipelines + the WGSL shader (`mandelbrot.wgsl`); the live
//!   paint callback and the offscreen `render_export` / `render_iter`.
//! - `fractadyne-color` — palette presets / gradient sampling.
//! - `fractadyne-state` — session persistence (TOML).
//! - `fractadyne-export`— PNG / OpenEXR encode + decode.
//! - `fractadyne-app`   — *this crate*: the window, input, egui UI, and the glue that
//!   drives the above. The `fractadyne` binary.
//!
//! # This crate's modules
//!
//! Extracted modules (cohesive units lifted out to keep `main.rs` navigable):
//! - [`cli`]      — headless CLI modes (`--find-minibrot`, `--compare`, `--crosscheck-f3`,
//!   `--validate-deep`), so `fn main` is just dispatch + window setup.
//! - [`render`]   — the render-request builders: reference orbit, series-skip, mode select,
//!   `MandelbrotParams`/`ExportRequest` assembly (the performance-critical bridge to the GPU).
//! - [`export`]   — render-to-file (PNG/EXR), the reloadable view-metadata blob, Open-view.
//! - [`autopilot`]— the hands-free auto-zoom dive.
//! - [`fractal`]  — the [`FractalKind`] domain enum (families, formula ids, descriptions).
//! - [`help`]     — the in-app Help window content.
//! - [`theme`]    — branding colors, dark visuals, wordmark, window icon.
//! - [`sysinfo`]  — version string, UTC formatter, CPU/VRAM/RAM probes.
//! - [`selftest`] — the `--selftest` GPU validation suite (render-path cross-checks + goldens).
//! - [`profile`]  — the `--profile` development profiling harness (benchmark regions → logs).
//!
//! `main.rs` itself now holds: small shared types/helpers (`Perf`, `RandomPalette`,
//! formatting, tunables); `fn main` (CLI dispatch + window setup); and [`FractadyneApp`] —
//! all app state plus the remaining behavior (UI panels/dialogs/toolbar/menus, bookmarks,
//! goto/locations, gallery, scripting, minimap, coloring) and the `eframe::App` `update`.
//!
//! Modularization is ongoing — the bulk left is the UI (the large `update` method and the
//! dialog/panel renderers); navigation, scripting, and coloring are also candidates. Moving
//! items between modules in one crate has no runtime cost (no added indirection; inlining is
//! unaffected) — purely an
//! organization/readability change.

use eframe::egui;
use fractadyne_core::Viewport;
use fractadyne_gpu::{add_mandelbrot, install_renderer};

mod alloc;
mod autopilot;
mod cli;
mod diag;
mod error;
mod export;
mod bench_matrix;
mod fractal;
mod chunksweep;
mod gputest;
mod help;
mod icons;
#[cfg(test)]
mod icons_coverage;
mod livetest;
mod motiontest;
mod profile;
mod refcache_persist;
mod render;
mod reusetest;
mod update;
mod scripting;
mod selftest;
mod sysinfo;
mod theme;
mod torture;
mod tone;
mod tunables;
mod ui;
mod uitest;

// ⭐The tunables live in ONE module now (a user requirement — see `tunables.rs`), and are
// re-exported from here and from `render` so that every existing `crate::NAME` /
// `crate::render::NAME` call site is unchanged. The move is therefore provably behaviour-neutral:
// the compiler resolves the same names to the same values, and nothing outside `tunables.rs` had
// to be edited to make it so.
pub(crate) use tunables::*;
pub(crate) use fractal::FractalKind;
pub(crate) use scripting::{BenchDepth, BenchRes, Playback, StdBench};
pub(crate) use sysinfo::*;
pub(crate) use theme::*;
use help::*;
use std::time::Instant;

/// An allocation failure aborts without running the panic hook, so the app's crash reporting is
/// blind to it — this wrapper reports the null return first. See `alloc.rs`.
#[global_allocator]
static GLOBAL: alloc::ReportingAlloc = alloc::ReportingAlloc;

/// Lightweight per-frame performance/diagnostic tracking, shown in an overlay.
/// On by default for now; toggle via the View menu or the `--no-perf` CLI flag.
struct Perf {
    enabled: bool,
    /// Height (px) of the bottom status bar as of the last frame — instrumentation for `--uitest`,
    /// which watches it wrap to a second line (or waver between one and two) across window widths.
    status_bar_h: f32,
    last_frame: Option<Instant>,
    /// Smoothed wall-clock interval between frames (ms) → FPS.
    frame_ms: f64,
    /// Smoothed CPU time spent in `update()` (ms).
    cpu_ms: f64,
    /// Duration of the most recent reference-orbit recompute (ms).
    recompute_ms: f64,
    /// Total reference recomputes since launch.
    recompute_total: u64,
    /// Recomputes counted in the current 1 s window, and the last computed rate.
    rate_count: u32,
    rate_t0: Option<Instant>,
    recompute_per_s: f32,
    /// Reference builds SPAWNED in the current 1 s window, and the last computed rate. Distinct
    /// from `recompute_per_s`, which counts INSTALLS: a build whose result is discarded (a
    /// lookahead slot the dive never reaches, a refused extension) costs full CPU and shows up
    /// nowhere else. That blind spot hid a prefetch spin running ~400 builds a second for six
    /// minutes — the perf panel read a calm `recompute/s 2` throughout. See
    /// `playback_ref_prefetch`.
    build_count: u32,
    builds_per_s: f32,
    /// Of those, the ones the script-playback LOOKAHEAD spawned — the counter its rate backstop
    /// reads, so a runaway queue stops itself instead of merely being reported.
    prefetch_count: u32,
    /// Latched once per session: a build rate this high is a bug, not a workload (the reactive
    /// path plus a full lookahead queue is single digits per second).
    build_storm_warned: bool,
    /// Diagnostics from the last view-0 (Mandelbrot) build.
    last_mode: u32,
    last_eff_iter: u32,
    last_precision: usize,
    last_orbit_len: u32,
    last_sa_skip: u32,
    /// SA skip of the last re-iterating dispatch, PER VIEW — stamped into the LETHAL-BAND line.
    /// Separate from `last_sa_skip` above, which is view-0-only status-bar state: a dual-view
    /// death would otherwise be reported with the Mandelbrot view's skip. Safe to read two frames
    /// late (a GPU timestamp comes back that far behind the dispatch it prices) because the skip
    /// travels with the REFERENCE and so holds still across those frames. The chunk WINDOW does
    /// not — it advances every frame — which is why it is deliberately not recorded here.
    last_sa_skip_v: [u32; 2],
    /// Render resolution of the last DISPATCHING frame, per view, for the same line. Added
    /// 2026-08-24: the 2026-08-22 field loss could not be checked against the odd-render-height
    /// defect (a ~20x per-pass cost fixed in beta.137) because the only resolution anywhere in
    /// that log was the one-off manifest — the always-logged lethal line carried none. It does
    /// now, so the next capture can answer it.
    /// ⚠Same up-to-two-frame staleness as `last_sa_skip_v`, and for the same reason. Resolution
    /// moves only when the budget resizes it, so it holds still far better than the chunk window.
    last_res_v: [[u32; 2]; 2],
    /// Monotonic frame counter, for resolving adaptive-AA probes (below).
    frame_idx: u64,
    /// Armed adaptive-AA wall-clock probe per view: `(ss rendered, frame armed, max frame-interval
    /// ms observed since)`. Armed by `build_params` when a settle-ramp stage renders a new ss;
    /// resolved in `update` two frames later (`desired_maximum_frame_latency = 1` means a heavy GPU
    /// frame back-pressures the NEXT acquire, so its cost lands in the following interval).
    /// The 4th element is the frame's NOMINAL step count (`spx·ss²·gpu_iter`), used to derive
    /// `fe_rate_spm` when the probe resolves. Only armed on frames that actually re-iterate, so the
    /// step count always corresponds to real GPU work.
    aa_probe: [Option<(u32, u64, f64, u64)>; 2],
    /// Last resolved stage cost per view: `(ss, ms)`. Lets the TDR cap extend past its static
    /// no-BLA worst case where the MEASURED cost shows BLA is effective. Cleared on interaction,
    /// so a measurement can never carry across views.
    aa_measured: [Option<(u32, f64)>; 2],
    /// Per-view floatexp frame budget in nominal steps (`spx·ss²·gpu_iter`); 0 ⇒ not yet measured.
    ///
    /// The budget used to be a hard constant calibrated on a view where BLA skipped ~200×, so nominal
    /// steps wildly overstated real work. On a deep INTERIOR minibrot BLA cannot skip — every pixel
    /// runs the full iteration count — and nominal == actual, making that constant ~15× too generous:
    /// the load frame ran ~2 s in ONE dispatch and Windows reset the GPU.
    ///
    /// So close a loop on the only thing that matters — how long a frame actually takes. A resolved
    /// probe reports the frame's step count and its measured wall cost, and the budget is retargeted
    /// to `steps × TDR_BUDGET_MS / measured_ms`. Growth is capped per probe; shrink is unrestricted,
    /// because both the GPU watchdog and the UI thread are unforgiving.
    ///
    /// It is set by `calibrate_fe_rate`, which times the real iterate offscreen at two tiny sizes and
    /// takes the slope. Do NOT try to derive it from frame intervals: a wall-clock frame gap is
    /// dominated by repaint scheduling (~420 ms here regardless of frame size), so a loop closed on it
    /// carries no signal and decays the view to a postage stamp. Nothing here is tuned to one GPU — a
    /// slower part measures a lower rate and gets a smaller budget for the same wall-clock target.
    fe_budget: [u64; 2],
    /// Live iterate GPU time (ms, `f64::to_bits`; 0 = nothing new) published by the paint callback.
    iterate_ms: [std::sync::Arc<std::sync::atomic::AtomicU64>; 2],
    /// Nominal step count of the pass `iterate_ms` timed, published by the GPU in the same breath.
    /// The app must NOT pair a late reading with its own "last dispatch" slot: the readback lands
    /// 2-3 frames after the pass, and a tiled settle dispatches every frame, so that slot has
    /// already moved on (usually to a small clamped-edge tile) and every reading gets discarded as
    /// undersized. Measured at an explicit 10,000,000 iterations: five readings, then the budget
    /// froze at 9.0e8 and the view sat at 34x27 forever.
    iterate_steps: [std::sync::Arc<std::sync::atomic::AtomicU64>; 2],
    /// Nominal steps of the last frame that actually re-iterated — the cost that `iterate_ms` prices.
    fe_steps_last: [u64; 2],
    /// Last measured live iterate GPU ms per view — a copy of the swapped `iterate_ms`
    /// reading kept for the perf HUD (D3.5); the atomic itself is consumed by the controller.
    last_iterate_ms: [f64; 2],
    /// True once a view's budget measurement has converged near the target — the gate for
    /// starting a tiled settle (see the controller in `update`).
    fe_budget_ok: [bool; 2],
    /// Frame index at which a live iterate TIMING last arrived for this view. Compared against
    /// `fe_iter_frame` to detect MEASUREMENT STARVATION — a real iterate ran and nothing ever came
    /// back to price it. That is this codebase's single most repeated bug shape (beta.40's `is_fe`
    /// sink gate, beta.41's unwritten `fe_steps_last`, and a device with no `TIMESTAMP_QUERY` are
    /// the same failure): a measured loop silently falls back on a bootstrap constant, and the
    /// constant then BINDS something it was never meant to size.
    ts_reading_frame: [u64; 2],
    /// Frame index at which this view last submitted a REAL iterate dispatch — a changed iterate
    /// key OR an advancing settle tile. Distinct from `fe_iter_frame`, which is stamped on the key
    /// change alone: under a running grid the key is deliberately stable and only the tile rect
    /// moves, so `fe_iter_frame` freezes for the whole settle even though every frame dispatches.
    /// Both the starvation detector and the wall-clock reading need "did this frame actually run
    /// the pass", and pairing either with the key-change stamp costs the entire settle's worth of
    /// measurements. (Kept separate rather than widening `fe_iter_frame`, whose other reader — the
    /// other view's busy check in `next_settle_tile` — would starve the second panel's grid if it
    /// were stamped every frame.)
    fe_dispatch_frame: [u64; 2],
    /// Sticky, global: measurement never arrived, so frames are priced by WALL CLOCK instead.
    /// Deliberately tripped by observation rather than by the `TIMESTAMP_QUERY` capability bit —
    /// the capability is only one of the ways a reading fails to show up, and the other ways are
    /// bugs, which is exactly when a fallback is worth having. Global because every cause is
    /// (device features, a starved sink); either view may trip it.
    wall_fallback: bool,
    /// Full-size dispatches (>= 0.7x the learned budget at dispatch) whose completion the GPU
    /// has not yet confirmed; the motion-jam gate (`MOTION_UNPRICED_MAX`) compares this count.
    /// Retirement is `Queue::on_submitted_work_done` — a real, ordered completion callback per
    /// counted dispatch — because both reading-based credits proved unsound:
    /// clear-to-zero on any reading (crash-1787275348-0, RX 6800 XT) let stale small readings
    /// re-admit three ceiling-sized monsters at a time — FIFO proof covers work submitted
    /// BEFORE the priced dispatch, not after — and retire-by-matching-the-reading never fired
    /// at all, because the sink publishes the MEASURED executed count, not the nominal stamp
    /// (caught on the dev 3080: the backlog pinned at cap, every motion frame clamped to
    /// bootstrap, `--motiontest` failed A2 and `--autodive` never reached the lethal regime —
    /// a full-throttle harness suddenly measuring 36.8 ms peaks is a gate stuck closed).
    full_inflight: [u32; 2],
    /// Completion registrations owed: incremented when a full-size dispatch is counted, drained
    /// in `update` on the FOLLOWING frame — eframe submits a frame's work after `update`
    /// returns, so registering immediately would arm the callback against the PREVIOUS frame's
    /// queue tail and retire the new dispatch before it ran.
    full_reg_pending: [u8; 2],
    /// Fired completion callbacks since last drain (one per registration, in order).
    full_done: [std::sync::Arc<std::sync::atomic::AtomicU32>; 2],
    /// Throttle for the motion-jam log line: episodes recur every few frames while a heavy dive
    /// outruns its readings, and a line per episode buried a real session under ~1,300 of them.
    jam_log_at: Option<std::time::Instant>,
    /// Process working set / peak, polled at ~1 Hz for the perf panel (one Win32 call each —
    /// the sysinfo doc says "NOT on anything per-frame", so this caches).
    mem_rss: u64,
    mem_peak: u64,
    mem_total: Option<u64>,
    mem_poll: Option<std::time::Instant>,
    /// Duration of the PREVIOUS frame's `update` body (ms) — see the slow-frame attribution log.
    prev_body_ms: f64,
    /// In-flight wall-clock cost probe per view: `(steps, frame it was armed on, max interval
    /// seen since)`. See `wall_clock_budget_tick` — a dispatch's cost lands one to two intervals
    /// after the frame that submitted it, so it cannot be priced in place.
    wall_probe: [Option<(u64, u64, f64)>; 2],
    /// Whether the device granted `TIMESTAMP_QUERY` at all. Diagnostics only — nothing sizes a
    /// frame from it — but a crash report that names it turns "renders blurry on my laptop" into
    /// one line of fact.
    ts_supported: bool,
    /// The `RenderMode` each view's budget was measured under (`u32::MAX` = never). A mode switch
    /// changes the cost of a nominal step by several times — floatexp is much dearer than df32 —
    /// so carrying the old budget across one sizes the first frame in the new mode off a
    /// measurement of the old. Same class as `install_recompute`'s cost-discontinuity derate, and
    /// handled the same way: drop the budget and let the controller re-climb.
    budget_mode: [u32; 2],
    /// `frame_idx` of the last arithmetic-MODE switch for this view (`u64::MAX` = none yet).
    /// Surfaced in the live crash manifest as `since_mode_switch`, because the df32 → floatexp
    /// crossover is where this app's longest-running crash class lands: the grand tour glides
    /// linearly in log-zoom from 1e10 to 1e30 and so crosses `PERT_FE_THRESHOLD` (1e28) at
    /// t = 178.2 s — "exactly 2:58", which is where the field reports put it, and all five
    /// recorded manifests interpolate to 1–5 s past it. Knowing how many frames separate the
    /// crossover from the death is the difference between a coincidence and a cause, and it
    /// costs one `u64` to answer.
    mode_switch_frame: [u64; 2],
    /// ⭐Measured nominal steps per millisecond, per view and per arithmetic mode (0.0 = never
    /// measured). This is what makes the opening guess hardware-derived instead of assumed; see
    /// `render::bootstrap_steps` and `tunables::TDR_BOOTSTRAP_MS`.
    ///
    /// ⚠**Latches the MINIMUM, never an average.** Within one mode the per-step rate swings by two
    /// orders of magnitude depending on whether a BLA tree is live — the same mode-2 view measured
    /// ~5.7e5 steps/ms with `bla_skip=0` right after a crossover and is worth a budget 3.4× larger
    /// once BLA is skipping (2026-08-15 field report). An average, or a last-value, would let a
    /// BLA-live reading size the opening dispatch of a no-BLA re-entry, which is exactly the frame
    /// that lost the device. The pessimistic latch costs at most a few ×1.5 climb frames; the
    /// optimistic one costs the device, and the asymmetry is the whole design.
    mode_rate: [[f64; Self::MODE_RATE_SLOTS]; 2],
    /// Milliseconds of DELIBERATE `fps_cap` sleep at the end of the previous frame.
    ///
    /// ⚠Subtracted from the next frame interval before anything prices cost off it. The interval is
    /// frame-start to frame-start and the cap sleep sits inside it by construction, so with a 30 fps
    /// cap every frame read ~33 ms no matter how cheap it actually was. Both consumers treat that
    /// number as a cost signal: the AIMD motion-resolution controller shrinks above 24 ms, so it
    /// ratcheted to the 0.30 floor within about five real frames and stayed there for the whole
    /// session — deep motion permanently at 30% linear resolution on hardware with headroom to
    /// spare — and the no-timestamp wall-clock budget fallback under-grew for the same reason.
    /// Sleeping is not work.
    cap_sleep_ms: f64,
    /// ⭐DIAGNOSTIC INSTRUMENT (`FRACTADYNE_BLA_DROP_FRAMES=N`, default off). Frames of BLA
    /// suppression remaining for this view after an arithmetic-mode switch.
    ///
    /// Exists because the device-loss regime could not be reached on purpose. Every recorded loss
    /// has `bla_skip=0` in the frames right after a mode 0→2 crossover: the reference was replaced
    /// across the switch (a hand-zoom rebuilds it every ~1.5–2 s as auto-iter raises the ask), so
    /// no BLA tree existed for it yet and nominal steps WERE real cost. A scripted tour holds one
    /// reference through the crossover and its tree is ready, so it lands in the safe variant of
    /// the same journey — measured on the RX 6800 XT, `bla_skip=5,590,609` on all four arms of the
    /// A/B, identical to the digit, with and without prefetch.
    ///
    /// Reproducing that by tuning tour geometry is a timing lottery. This makes it a switch.
    ///
    /// Stored as the frame index to suppress UNTIL, not a countdown: `build_params` can run more
    /// than once per frame (tiled settles do), and a decrement there would burn the window early
    /// and silently shorten the very thing being measured.
    bla_suppress_until: [u64; 2],
    /// Tiled-settle progress per view (see `render::TileGrid`); `None` = no grid armed or running.
    tile_state: [Option<render::TileGrid>; 2],
    /// Iteration-range tiling. `chunk_ok`: the device granted the 48-byte color-attachment limit
    /// the three-target resumable pass needs (probed once). `chunk_fe_ok`: it also granted the 64
    /// bytes MODE 2 needs, because floatexp state does not fit in three targets — kept separate so
    /// a device that can do one and not the other loses only the one. `chunk_cursor`: the next
    /// iteration to resume from per view (0 = fresh). `chunk_idx`: pass counter (ping-pong
    /// parity). `chunk_sig`: the view identity the cursor belongs to — any change restarts the
    /// progression. `chunk_pending`: a progression is mid-flight (drives repaints, like
    /// `tile_pending`).
    chunk_ok: bool,
    chunk_fe_ok: bool,
    chunk_cursor: [u32; 2],
    chunk_idx: [u32; 2],
    chunk_sig: [(u64, u32, [u32; 2], u32); 2],
    chunk_pending: [bool; 2],
    /// Motion-presentation observability (design/mode2-chunking.md §11, asserted by
    /// `--motiontest`). Counted only DURING INTERACTION at a chunk-eligible perturbation view
    /// that already had frozen content: `adopt_partial` = frozen-texture latches that adopted a
    /// texture whose chunk progression was mid-flight (the §9 regression — under-iterated content
    /// becomes the held frame and is reprojected as if it were real detail); `adopt_complete` =
    /// latches/adoptions of complete content (during motion, this is streamed detail — its floor
    /// is the anti-freeze assertion). `chunk_motion_frames` counts the frames those assertions
    /// cover (anti-vacuity: a run with none tested nothing). `dirty_shown` counts frames that
    /// DISPLAYED the live texture while it diverged from the frozen bookkeeping during
    /// interaction — must stay 0 once the pinned refresh lands.
    adopt_partial: [u64; 2],
    adopt_complete: [u64; 2],
    chunk_motion_frames: [u64; 2],
    dirty_shown: [u64; 2],
    /// A pinned refresh in flight per view (option C, design/mode2-chunking.md §10-§11): the
    /// chunked refresh renders across frames at this captured view while the display keeps
    /// reprojecting the previous complete frozen texture; adopted only on completion.
    pin: [Option<crate::render::PinnedRefresh>; 2],
    /// The live iteration texture diverged from the frozen bookkeeping (a pin dispatched into it
    /// and has not adopted): while interacting, the display must serve the hold snapshot, never
    /// the texture. Cleared by adoption, by any real (non-pin) latch, and at the settle edge.
    chunk_dirty: [bool; 2],
    /// The last chunk range actually DISPATCHED per view — what a backpressure-paced frame
    /// re-emits verbatim so the GPU's (key, tile, chunk, probe) triple stays unchanged and no
    /// pass runs (the zero-work hold, chunking's version of the zero-area tile). `None` = no
    /// dispatch since the last progression reset.
    chunk_last_range: [Option<[u32; 2]>; 2],
    /// Wall price of the last settled chunk pass: the frame interval measured one frame AFTER
    /// the dispatch (the dispatch frame's own dt predates its submission). Feeds
    /// `render::chunk_step_factor`, which sizes the next pass from what the last one actually
    /// cost — the signal that keeps working when a saturated queue starves the GPU timestamps.
    /// 0.0 = no pass measured since the progression (re)started.
    chunk_pass_dt: [f64; 2],
    /// PRICE-SERIALIZED WALKING (design/mode2-chunking.md §12): the one pass a settled walk may
    /// have in flight — (size, cursor band, accumulated wall ms). Priced and released when a
    /// quick frame proves the queue drained; only then may the next pass launch.
    /// `(size, band, accumulated_wall_ms, already_shed)`. The last flag is the wall-clock
    /// retreat's latch: a pass whose accumulation crosses the lethal band sheds the ledger ONCE,
    /// without releasing the next dispatch. See the `acc >= lethal` branch in `render.rs`.
    chunk_inflight: [Option<(u32, u8, f64, bool)>; 2],
    /// The regional license ledger: per cursor band, the largest pass size whose wall price came
    /// in AT or under the target (quartered on a cliff price; see `render::chunk_band_update`).
    /// Survives same-sig restarts — re-crossing the wrap-storm band with amnesia was the kill —
    /// and clears when the sig changes.
    chunk_bands: [[u32; crate::tunables::CHUNK_BANDS]; 2],
    /// Budget-climb probe (see `MandelbrotParams::probe_nonce`): bumped on settled frames while
    /// the budget is unconverged so the GPU re-measures — breaks the resolution-floor deadlock
    /// where budget growth is too small to re-key the frame and the climb freezes.
    probe_nonce: [u32; 2],
    /// Present-gating ("prefer detail" stage B): the view's display is currently serving the hold
    /// snapshot while a composite builds invisibly. Engaged when compose work starts on a settled
    /// prefer-detail view; dropped — revealing the completed frame whole — when nothing composes.
    hold_active: [bool; 2],
    /// The hold snapshot's display transform `(uv_off_x, uv_off_y, uv_scale)`, captured ONCE at
    /// snapshot time from the frozen-frame bookkeeping (the snapshot holds the PRE-motion frame,
    /// but `frozen_center` is overwritten by the very first compose iterate — so the transform
    /// must be pinned here or later gate frames would mis-place the hold at identity).
    hold_uv: [[f32; 3]; 2],
    /// True while a view's settle grid has tiles left — holds the AA ramp and keeps repaints coming.
    tile_pending: [bool; 2],
    /// `frame_idx` of the last frame that spent its tile: one budget-sized tile per submission, so
    /// two deep views can't pair their dispatches past the watchdog.
    tile_turn: u64,
    /// `frame_idx` of each view's last interacting frame / last re-iterating floatexp dispatch.
    /// A settle tile HOLDS while the OTHER view is on either marker: a budget-sized tile must not
    /// share a submission with the other view's budget-sized frame (two ~TDR_BUDGET_MS dispatches
    /// pair up toward the watchdog), and a grid must not hog ~1 fps frames while the user is
    /// actively working in the other panel.
    interact_frame: [u64; 2],
    fe_iter_frame: [u64; 2],
    /// Bumped (to `frame_idx`) by any motion/reproject/autopilot frame; part of the tile-grid key,
    /// standing in for the view position (the f64 center is degenerate at depth).
    view_gen: [u64; 2],
    /// TDR step budget in force on the last frame that actually re-iterated, per view. A reproject
    /// frame re-samples that frame's frozen texture, so it must reproduce its exact resolution or the
    /// color-pass aspect-fit `fit = out_res / frozen_screen_dim` drifts off 1 (the v0.1.44/46
    /// magnify/shrink bugs). Since the budget is now adaptive, reproject frames reuse the stored value
    /// rather than recomputing from a moving one.
    frozen_budget: [u64; 2],
    /// Iterate-key `(ss, resolution, orbit_id)` submitted last frame per view — change detection
    /// for probe arming (a probe is only valid on a frame that actually re-iterates).
    aa_last_key: [(u32, [u32; 2], u64); 2],
    /// Raw (un-smoothed) last frame interval in ms — the motion-res controller needs the actual
    /// spike a real re-iterate frame caused, which the EMA `frame_ms` smooths away.
    last_dt_ms: f64,
    /// Whether each view's LAST built frame really re-iterated (vs reprojected a held frame) —
    /// stamped at the end of `build_params`; the motion-res controller adapts only on the
    /// interval that follows a real frame.
    prev_real: [bool; 2],
    /// Live MAXITER-counter sink per view (GPU → app, see `MandelbrotParams::maxiter_count`):
    /// written `count + 1` a couple frames after a full-frame iterate; drained with `swap(0)`.
    maxiter_sink: [std::sync::Arc<std::sync::atomic::AtomicU64>; 2],
    /// ADAPTIVE LIVE ITERATION BUDGET per view: multiplier on `zoom_iter_cap` for SETTLED frames.
    /// Near Misiurewicz spars the local escape counts exceed the live cap's 256/octave slope, so
    /// the settled view shows smooth "capped blobs" where an export shows dendrites; the probe
    /// controller below raises this while raising measurably reduces the capped-pixel fraction.
    iter_boost: [f64; 2],
    /// Last `(boost, capped fraction)` measurement — the probe's "did the raise help?" baseline.
    iter_probe: [Option<(f64, f64)>; 2],
    /// True once raising has been shown not to help (true interior in view — e.g. inside a
    /// minibrot): stop probing, revert to where the fruitless run began. Cleared with the view.
    iter_plateau: [bool; 2],
    /// Last settled measurement of the capped-pixel fraction, the budget it was measured at, and
    /// whether the app could still raise that budget itself. Fed by every valid settled reading
    /// (not just the probe's), so the status-bar limit diagnostic works at a manually-set budget
    /// where the probe never runs. `None` while moving.
    capped_frac: [Option<f64>; 2],
    budget_measured: [u32; 2],
    budget_maxed: [bool; 2],
    /// The probe climbed to the full appetite and the frame was STILL essentially all-capped —
    /// deep interior, or a view needing more than the app can give. Either way nothing more can
    /// be done here, so the boost reverts (same flat image, far cheaper) and the status bar says
    /// so rather than leaving a black screen unexplained.
    iter_exhausted: [bool; 2],
    /// Consecutive raises that bought nothing, and the boost the run started from.
    ///
    /// A single unhelpful raise does NOT mean "interior". Measured at the 3.3e61× three-spar: the
    /// capped fraction sits at exactly 100% through ×1.6 AND ×2.6, then collapses to 12% at ×4.1.
    /// Latching after one flat step reverted straight back to a black screen, so the probe now
    /// has to see several flat steps in a row before concluding there is nothing to find.
    iter_stall: [u8; 2],
    iter_stall_base: [f64; 2],
    /// Escape-range sink per view (GPU → app: packed `(min_bits << 32) | max_bits` f32 bits of the
    /// frame's escaped smooth-iter range; drained with `swap(u64::MAX)`).
    norm_sink: [std::sync::Arc<std::sync::atomic::AtomicU64>; 2],
    /// Live sink for the escape field's LOCAL GRADIENT — `(Σ|Δ|×16) << 32 | (samples + 1)`. The
    /// auto-normalization predicate keys on this rather than on the escape RANGE: aliasing is a
    /// local property (how far the palette moves between neighbouring pixels), not a global one.
    grad_sink: [std::sync::Arc<std::sync::atomic::AtomicU64>; 2],
    /// Live sink for `(rebase + 1) << 32 | (bla_skip + 1)` per view — how effective BLA actually
    /// is on the frames the user is looking at. Until beta.48 only the offline paths could see
    /// this, and a reference whose BLA skips nothing cost ~1 s a frame unnoticed.
    work_sink: [std::sync::Arc<std::sync::atomic::AtomicU64>; 2],
    /// EMA-smoothed escaped smooth-iter range per view — the live auto-normalization input.
    norm_range: [Option<(f32, f32)>; 2],
    /// EMA-smoothed MEAN |Δ smooth-iter| between neighbouring escaped pixels, per view.
    norm_grad: [Option<f32>; 2],
    /// This view's frames are currently chunk-governed (`chunk_over`), stamped every
    /// `build_params`. The norm drain keys on THIS, not on the cursor: escape readings lag their
    /// dispatches by 2-3 frames, so a completed walk's LAST band lands with the cursor already at
    /// the ask — classified by cursor it read as a whole-frame range and dragged the palette
    /// window onto one band (field report round two: "flashed correct coloring briefly, then
    /// flat"). Classified by regime it accumulates, and same-view stragglers folding into the
    /// next walk's accumulator are harmless.
    chunk_governed: [bool; 2],
    /// Adaptive deep-motion resolution scale (AIMD), driven by `frame_ms` in `build_params`. The
    /// WORK_BUDGET `res_scale` sizes moving frames from the *no-BLA-skip* cost and over-shrinks them
    /// where the BLA skips; this measured scale grows toward native while frames stay near vsync and
    /// backs off when they run long. Only deep perturbation motion reads it.
    motion_res: f64,
}

impl Perf {
    /// Arithmetic modes are `RenderMode::to_u32()` ∈ {0, 1, 2}; `u32::MAX` means "none yet" and is
    /// simply out of range, so `slot()` rejects it along with anything a future mode adds.
    const MODE_RATE_SLOTS: usize = 3;

    fn slot(mode: u32) -> Option<usize> {
        ((mode as usize) < Self::MODE_RATE_SLOTS).then_some(mode as usize)
    }

    /// The WORST rate (steps per ms) measured for this view's current mode, if any has been.
    /// `mode_rate` is kept as a running MINIMUM, so this is already the pessimistic extreme rather
    /// than an average — which is what a safety floor has to be sized from.
    pub(crate) fn worst_rate_steps_per_ms(&self, v: usize) -> Option<f64> {
        let r = Self::slot(self.budget_mode[v]).map_or(0.0, |s| self.mode_rate[v][s]);
        (r.is_finite() && r > 0.0).then_some(r)
    }

    /// Fold one priced dispatch into this view's per-mode rate, keeping the PESSIMISTIC extreme.
    /// See the `mode_rate` field comment for why this is a min and not an average.
    fn record_mode_rate(&mut self, v: usize, ms: f64, steps: u64) {
        let (Some(s), true) = (Self::slot(self.budget_mode[v]), ms > 0.0 && steps > 0) else {
            return;
        };
        // ⚠A measurement is DEFERRED — `wall_probe_step` holds a reading for two frames, and the
        // timestamp readback lags similarly — so `budget_mode` at record time is not necessarily
        // the mode the priced dispatch ran in. Straight after a crossover that misattributes the
        // OLD mode's rate to the new one, and the old mode is the fast one (~152× at 0→2 in the
        // field report), which is precisely the optimistic direction this whole mechanism exists to
        // avoid. The ceiling clamp in `bootstrap_steps` means such a poisoned rate can never be
        // worse than the historical constant, but it would silently defeat the fix in the one
        // window that motivated it. Skipping the frames that straddle the switch is enough: the
        // dispatch behind a reading taken 3+ frames later is unambiguously in the current mode.
        let since = self.frame_idx.saturating_sub(self.mode_switch_frame[v]);
        if self.mode_switch_frame[v] != u64::MAX && since < 3 {
            return;
        }
        let rate = steps as f64 / ms;
        if !rate.is_finite() || rate <= 0.0 {
            return;
        }
        let cur = self.mode_rate[v][s];
        self.mode_rate[v][s] = if cur > 0.0 { cur.min(rate) } else { rate };
    }

    /// The opening guess for this view's CURRENT mode, derived from what has actually been measured
    /// on this device. `rate_other` is the most pessimistic rate seen in any other mode, which is
    /// all we have to go on the first time a dive crosses into a mode.
    /// How many frames of BLA suppression `FRACTADYNE_BLA_DROP_FRAMES` asks for after a mode
    /// switch. Zero (the default, and any unparseable value) disables the instrument entirely.
    /// Read once: this is consulted per frame and an env lookup per frame is a silly cost.
    pub(crate) fn bla_drop_frames() -> u32 {
        use std::sync::OnceLock;
        static N: OnceLock<u32> = OnceLock::new();
        *N.get_or_init(|| {
            std::env::var("FRACTADYNE_BLA_DROP_FRAMES")
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0)
        })
    }

    fn bootstrap_steps(&self, v: usize) -> u64 {
        let this = Self::slot(self.budget_mode[v]).map_or(0.0, |s| self.mode_rate[v][s]);
        let other = self.mode_rate[v]
            .iter()
            .enumerate()
            .filter(|(s, _)| Self::slot(self.budget_mode[v]) != Some(*s))
            .map(|(_, r)| *r)
            .filter(|r| *r > 0.0)
            .fold(0.0_f64, |acc, r| if acc == 0.0 { r } else { acc.min(r) });
        render::bootstrap_steps(this, other)
    }
}

impl Default for Perf {
    fn default() -> Self {
        Self {
            enabled: true,
            status_bar_h: 0.0,
            last_frame: None,
            frame_ms: 0.0,
            cpu_ms: 0.0,
            recompute_ms: 0.0,
            recompute_total: 0,
            rate_count: 0,
            build_count: 0,
            builds_per_s: 0.0,
            prefetch_count: 0,
            build_storm_warned: false,
            rate_t0: None,
            recompute_per_s: 0.0,
            last_mode: 0,
            last_eff_iter: 0,
            last_precision: 0,
            last_orbit_len: 0,
            last_sa_skip: 0,
            last_sa_skip_v: [0; 2],
            last_res_v: [[0; 2]; 2],
            frame_idx: 0,
            aa_probe: [None, None],
            aa_measured: [None, None],
            aa_last_key: [(1, [0, 0], 0), (1, [0, 0], 0)],
            // Unknown until measured — the first floatexp frame uses the hardware-agnostic bootstrap.
            fe_budget: [0, 0],
            iterate_ms: [Default::default(), Default::default()],
            iterate_steps: [Default::default(), Default::default()],
            fe_steps_last: [0, 0],
            last_iterate_ms: [0.0, 0.0],
            fe_budget_ok: [false, false],
            ts_reading_frame: [0, 0],
            fe_dispatch_frame: [0, 0],
            wall_fallback: false,
            full_inflight: [0, 0],
            full_reg_pending: [0, 0],
            full_done: [
                std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
                std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
            ],
            jam_log_at: None,
            mem_rss: 0,
            mem_peak: 0,
            mem_total: None,
            mem_poll: None,
            prev_body_ms: 0.0,
            wall_probe: [None, None],
            ts_supported: false,
            budget_mode: [u32::MAX, u32::MAX],
            mode_switch_frame: [u64::MAX, u64::MAX],
            mode_rate: [[0.0; Self::MODE_RATE_SLOTS]; 2],
            cap_sleep_ms: 0.0,
            bla_suppress_until: [0, 0],
            tile_state: [None, None],
            tile_pending: [false, false],
            chunk_ok: false,
            chunk_fe_ok: false,
            chunk_cursor: [0, 0],
            chunk_idx: [0, 0],
            chunk_sig: [(0, 0, [0, 0], 0), (0, 0, [0, 0], 0)],
            chunk_pending: [false, false],
            adopt_partial: [0, 0],
            adopt_complete: [0, 0],
            chunk_motion_frames: [0, 0],
            dirty_shown: [0, 0],
            pin: [None, None],
            chunk_dirty: [false, false],
            chunk_last_range: [None, None],
            chunk_pass_dt: [0.0, 0.0],
            chunk_inflight: [None, None],
            chunk_bands: [[0; crate::tunables::CHUNK_BANDS], [0; crate::tunables::CHUNK_BANDS]],
            probe_nonce: [0, 0],
            hold_active: [false, false],
            hold_uv: [[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
            tile_turn: u64::MAX,
            interact_frame: [0, 0],
            fe_iter_frame: [0, 0],
            view_gen: [0, 0],
            frozen_budget: [0, 0],
            last_dt_ms: 0.0,
            prev_real: [false, false],
            maxiter_sink: [
                std::sync::Arc::new(std::sync::atomic::AtomicU64::new(u64::MAX)),
                std::sync::Arc::new(std::sync::atomic::AtomicU64::new(u64::MAX)),
            ],
            iter_boost: [1.0, 1.0],
            iter_probe: [None, None],
            iter_plateau: [false, false],
            capped_frac: [None, None],
            budget_measured: [0, 0],
            budget_maxed: [false, false],
            iter_exhausted: [false, false],
            iter_stall: [0, 0],
            iter_stall_base: [1.0, 1.0],
            norm_sink: [
                std::sync::Arc::new(std::sync::atomic::AtomicU64::new(u64::MAX)),
                std::sync::Arc::new(std::sync::atomic::AtomicU64::new(u64::MAX)),
            ],
            grad_sink: [
                std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
                std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
            ],
            work_sink: [
                std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
                std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
            ],
            norm_range: [None, None],
            norm_grad: [None, None],
            chunk_governed: [false, false],
            motion_res: 0.6,
        }
    }
}

fn ema(prev: f64, sample: f64) -> f64 {
    if prev <= 0.0 {
        sample
    } else {
        prev * 0.9 + sample * 0.1
    }
}

/// State for the "Render script…" dialog and the render it launches.
///
/// The render runs as a CHILD PROCESS (`--render-tour`), not on a thread: a tour render mutates
/// app state per frame — viewport, fractal, iteration budget — so moving it to a worker would mean
/// extracting all of that first. A separate process also gets its own GPU device, which matters
/// here specifically: a deep tour render is the heaviest thing this app does, and the failure mode
/// on record is losing the device. In a child, that kills the render and leaves the editor alive.
pub(crate) struct TourRenderUi {
    pub(crate) open: bool,
    pub(crate) out: String,
    pub(crate) prefix: String,
    pub(crate) width: u32,
    pub(crate) height: u32,
    /// Size dropdown is on "Custom…" — see `ExportState::custom_size` for why it is sticky.
    pub(crate) custom_size: bool,
    pub(crate) fps: f64,
    pub(crate) ss: u32,
    /// 0 = the whole tour; otherwise 1-based into the script's `[[segment]]` chapters.
    pub(crate) segment: usize,
    pub(crate) mp4: bool,
    pub(crate) overwrite: bool,
    /// Keep frames already on disk and render only what's missing (`--resume`).
    pub(crate) resume: bool,
    /// Render order: progressive (keyframes first, then bisect the largest gaps — preview the
    /// whole tour early) vs the default sequential. Maps to `--order progressive`.
    pub(crate) progressive: bool,
    /// Latest line from the child (its "frame N/M …" progress), and the finished-run summary.
    pub(crate) progress: String,
    /// `(done, planned)` parsed from the latest progress line — drives the progress BAR; the raw
    /// line stays underneath for the elapsed/left/fps detail. `None` until the first frame line
    /// (or when a line doesn't parse — the bar simply holds its last state).
    pub(crate) progress_frames: Option<(u64, u64)>,
    /// First error line the child wrote to stderr — the reason a render stopped.
    pub(crate) error: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) child: Option<std::process::Child>,
    pub(crate) rx: Option<std::sync::mpsc::Receiver<crate::ui::tour_render::RenderLine>>,
}

impl Default for TourRenderUi {
    fn default() -> Self {
        Self {
            open: false,
            out: "frames".to_string(),
            prefix: String::new(),
            width: 1920,
            height: 1080,
            custom_size: false,
            fps: 30.0,
            ss: 2,
            segment: 0,
            mp4: false,
            overwrite: true,
            resume: false,
            progressive: false,
            progress: String::new(),
            progress_frames: None,
            error: None,
            status: None,
            child: None,
            rx: None,
        }
    }
}

/// Parse a `--size` value as either a bare width (`1920`) or `WIDTHxHEIGHT` (`5120x2160`,
/// case-insensitive `x`/`X`/`×`). Returns `(width, height)` where each is `Some` when present and
/// parseable. This lets callers accept both forms; a bare width leaves the height to `--height` or
/// an aspect-ratio default.
/// The separators `parse_size` splits on. Shared so a caller can ask "was a height SUPPLIED?"
/// without re-listing the characters and drifting from the parser.
pub(crate) fn size_has_separator(s: &str) -> bool {
    s.contains(|c: char| c == 'x' || c == 'X' || c == '×')
}

pub(crate) fn parse_size(s: &str) -> (Option<u32>, Option<u32>) {
    let sep = |c: char| c == 'x' || c == 'X' || c == '×';
    match s.split_once(sep) {
        Some((w, h)) => (w.trim().parse().ok(), h.trim().parse().ok()),
        None => (s.trim().parse().ok(), None),
    }
}

/// Tokenize an args (response) file: whitespace-separated tokens, honoring `"…"` / `'…'` quoting so
/// values with spaces survive, with `#` starting a comment to end of line (outside quotes). One
/// token per argument — the same as typing them on the command line.
fn tokenize_args_file(text: &str) -> Vec<String> {
    let mut toks = Vec::new();
    for line in text.lines() {
        let mut cur = String::new();
        let mut in_tok = false;
        let mut quote: Option<char> = None;
        for c in line.chars() {
            match quote {
                Some(q) => {
                    if c == q {
                        quote = None;
                    } else {
                        cur.push(c);
                    }
                }
                None if c == '"' || c == '\'' => {
                    quote = Some(c);
                    in_tok = true;
                }
                None if c == '#' => break, // comment to end of line
                None if c.is_whitespace() => {
                    if in_tok {
                        toks.push(std::mem::take(&mut cur));
                        in_tok = false;
                    }
                }
                None => {
                    cur.push(c);
                    in_tok = true;
                }
            }
        }
        if in_tok {
            toks.push(cur);
        }
    }
    toks
}

/// Expand `@FILE` response-file arguments and `--args-file FILE` in `raw`, recursively (bounded), so
/// an entire command line can be kept in a text file. Each referenced file's tokens are spliced in
/// place; every other argument passes through untouched. `#` comments and quoting are supported.
/// A missing/unreadable file is a hard error (the `@`/`--args-file` sigil is an explicit request).
fn expand_arg_files(raw: &[String]) -> Result<Vec<String>, String> {
    fn go(args: &[String], out: &mut Vec<String>, depth: u32) -> Result<(), String> {
        if depth > 16 {
            return Err("--args-file nesting too deep (cycle?)".to_string());
        }
        let mut i = 0;
        while i < args.len() {
            let a = &args[i];
            let path = if let Some(p) = a.strip_prefix('@') {
                Some(p.to_string())
            } else if a == "--args-file" || a == "--args" {
                i += 1;
                Some(args.get(i).ok_or("--args-file needs a file path")?.clone())
            } else {
                None
            };
            match path {
                Some(p) => {
                    let text = std::fs::read_to_string(&p).map_err(|e| format!("args file '{p}': {e}"))?;
                    go(&tokenize_args_file(&text), out, depth + 1)?;
                }
                None => out.push(a.clone()),
            }
            i += 1;
        }
        Ok(())
    }
    let mut out = Vec::new();
    go(raw, &mut out, 0)?;
    Ok(out)
}

/// HSV (all 0..1) → RGB (0..1). For synthesizing vivid random palette stops.
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let h6 = (h.fract() * 6.0).clamp(0.0, 6.0);
    let i = h6.floor() as i32;
    let f = h6 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    match i % 6 {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

/// Randomized, continuously-morphing palette. Holds two gradient keyframes (`from`,
/// `to`) of fixed 6 stops and blends between them; on reaching `to` it becomes the
/// new `from` and a fresh random `to` is synthesized. Stops are HSV-random with the
/// endpoints equal so the gradient still cycles seamlessly.
struct RandomPalette {
    rng: u32,
    from: [[f32; 4]; fractadyne_color::MAX_STOPS],
    to: [[f32; 4]; fractadyne_color::MAX_STOPS],
    t: f32,
}

const RAND_STOPS: usize = 6;

impl RandomPalette {
    fn new(seed: u32) -> Self {
        let mut s = RandomPalette {
            rng: seed | 1,
            from: [[0.0; 4]; fractadyne_color::MAX_STOPS],
            to: [[0.0; 4]; fractadyne_color::MAX_STOPS],
            t: 0.0,
        };
        s.from = s.gen_stops();
        s.to = s.gen_stops();
        s
    }
    fn next_u32(&mut self) -> u32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x
    }
    fn next_f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }
    /// Generate a *harmonious* gradient: a single base hue with a gentle analogous
    /// excursion (so colors flow through neighbouring hues, not a clashing rainbow), a
    /// smooth dark→bright→dark brightness arc for contrast, and matching dark endpoints
    /// for seamless cycling. The hue/brightness follow a `sin(πt)` arc so the first and
    /// last stops coincide (no seam) while the middle carries the color and light.
    fn gen_stops(&mut self) -> [[f32; 4]; fractadyne_color::MAX_STOPS] {
        let h0 = self.next_f32(); // base hue
        let hue_span = 0.05 + 0.16 * self.next_f32(); // gentle analogous shift (~18°–75°)
        let dir = if self.next_f32() < 0.5 { -1.0 } else { 1.0 };
        let sat = 0.55 + 0.30 * self.next_f32(); // moderate, constant saturation
        let v_lo = 0.16 + 0.12 * self.next_f32(); // dim (not black) endpoints
        let v_hi = 0.85 + 0.15 * self.next_f32(); // bright middle
        let mut out = [[0.0f32; 4]; fractadyne_color::MAX_STOPS];
        for (i, slot) in out.iter_mut().enumerate().take(RAND_STOPS) {
            let t = i as f32 / (RAND_STOPS - 1) as f32; // 0..1
            let arc = (std::f32::consts::PI * t).sin(); // 0 at ends → 1 at middle
            let h = h0 + dir * hue_span * arc;
            let v = v_lo + (v_hi - v_lo) * arc;
            let c = hsv_to_rgb(h, sat, v);
            *slot = [c[0], c[1], c[2], t];
        }
        for i in RAND_STOPS..fractadyne_color::MAX_STOPS {
            out[i] = out[RAND_STOPS - 1];
        }
        out
    }
    /// Advance the blend; `speed` is gradient-changes per second.
    fn advance(&mut self, dt: f32, speed: f32) {
        self.t += dt * speed;
        while self.t >= 1.0 {
            self.t -= 1.0;
            self.from = self.to;
            self.to = self.gen_stops();
        }
    }
    /// Snap to a brand-new pair of gradients.
    fn reshuffle(&mut self) {
        self.from = self.gen_stops();
        self.to = self.gen_stops();
        self.t = 0.0;
    }
    /// Current blended stops for GPU upload.
    fn current(&self) -> ([[f32; 4]; fractadyne_color::MAX_STOPS], u32) {
        let mut out = self.from;
        // Per-channel lerp of two parallel stop arrays (from/to) into out; explicit indices read
        // clearer here than a double `enumerate`, so the range-loop lint is allowed locally.
        #[allow(clippy::needless_range_loop)]
        for i in 0..RAND_STOPS {
            for k in 0..3 {
                out[i][k] = self.from[i][k] + (self.to[i][k] - self.from[i][k]) * self.t;
            }
        }
        for i in RAND_STOPS..fractadyne_color::MAX_STOPS {
            out[i] = out[RAND_STOPS - 1];
        }
        (out, RAND_STOPS as u32)
    }
}

// `lerp_color` moved to `theme.rs` (re-exported below).

const EASE_TAU: f64 = 0.15; // ease-in/out time constant (seconds)

/// Dual-view divider bounds: the fraction of the width the LEFT (Mandelbrot) panel may take.
/// Neither panel may collapse — a zero-width panel still costs a render and shows nothing. The
/// drag handle, the session restore, and the scripted `dual_split` keyframe field all clamp here,
/// so a script cannot reach a state the viewer cannot drag back out of.
pub(crate) const DUAL_SPLIT_MIN: f32 = 0.15;
pub(crate) const DUAL_SPLIT_MAX: f32 = 0.85;

/// Anti-alias supersampling for progressive-settle stage `frame`, ramping 1→2→4→… up to `target`.
/// A settled view refines from an instant coarse frame to full AA over a few frames, rather than
/// blocking on one expensive full-AA frame. `frame` is capped so the shift can't overflow.
fn aa_ramp(frame: u32, target: u32) -> u32 {
    (1u32 << frame.min(5)).min(target.max(1))
}




/// Cache key for the interactive orbit (recompute only when these change).
#[derive(Clone, PartialEq)]
struct OrbitKey {
    px: f64,
    py: f64,
    cx: f64,
    cy: f64,
    upp: f64,
    julia: bool,
    formula: u32,
    jcx: f64,
    jcy: f64,
}
struct OrbitCacheEntry {
    key: OrbitKey,
    pts: Vec<(f64, f64)>,
}


// Version / system-info / time helpers moved to `sysinfo.rs` (re-exported below).

// ---- Fractadyne branding (matches design/Fractadyne.dc.html) ----
// Branding (BRAND_ACCENT/BRAND_TEXT, apply_brand_theme, brand_wordmark, brand_icon) moved
// to `theme.rs` (re-exported below).

// ---- coloring-method / orbit-trap enums (Phase 4: typed dispatch) ----
/// Coloring method — a typed replacement for the former bare `u32`. Discriminants match the GPU
/// `color_method` ids in `mandelbrot.wgsl` `fs_color` (serialized via [`ColorMethod::to_u32`]); the
/// `key` is the stable string persisted in the session / `.fdn`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum ColorMethod {
    #[default]
    Smooth = 0,
    Stripe = 1,
    TriangleIneq = 2,
    OrbitTrap = 3,
    Distance = 4,
    Decomposition = 5,
}
impl ColorMethod {
    pub(crate) const ALL: [ColorMethod; 6] = [
        ColorMethod::Smooth,
        ColorMethod::Stripe,
        ColorMethod::TriangleIneq,
        ColorMethod::OrbitTrap,
        ColorMethod::Distance,
        ColorMethod::Decomposition,
    ];
    /// GPU dispatch id (matches the shader).
    pub(crate) fn to_u32(self) -> u32 {
        self as u32
    }
    /// Stable persisted key.
    pub(crate) fn key(self) -> &'static str {
        match self {
            ColorMethod::Smooth => "smooth",
            ColorMethod::Stripe => "stripe",
            ColorMethod::TriangleIneq => "triangle",
            ColorMethod::OrbitTrap => "trap",
            ColorMethod::Distance => "distance",
            ColorMethod::Decomposition => "decomposition",
        }
    }
    /// Human label for the UI combo box.
    pub(crate) fn label(self) -> &'static str {
        match self {
            ColorMethod::Smooth => "Smooth iteration",
            ColorMethod::Stripe => "Stripe average",
            ColorMethod::TriangleIneq => "Triangle inequality",
            ColorMethod::OrbitTrap => "Orbit trap",
            ColorMethod::Distance => "Distance estimate",
            ColorMethod::Decomposition => "Decomposition",
        }
    }
    pub(crate) fn from_key(s: &str) -> ColorMethod {
        ColorMethod::ALL.into_iter().find(|m| m.key() == s).unwrap_or_default()
    }
    pub(crate) fn from_u32(v: u32) -> ColorMethod {
        ColorMethod::ALL.into_iter().find(|m| m.to_u32() == v).unwrap_or_default()
    }
    /// Methods that accumulate the aux statistics texture (stripe / triangle-ineq / decomposition).
    pub(crate) fn needs_aux(self) -> bool {
        matches!(
            self,
            ColorMethod::Stripe | ColorMethod::TriangleIneq | ColorMethod::Decomposition
        )
    }

    /// Methods that CANNOT skip iterations — BLA and series approximation must stay off. Two reasons
    /// a method blocks skipping: (1) it accumulates a **running per-iteration** statistic (stripe /
    /// triangle-inequality average, orbit-trap min) that skipped iterations would silently drop; or
    /// (2) it is a **discontinuous function of the exact final escape point** — decomposition tiles
    /// the plane into angular cells, so SA/BLA's small trajectory approximation shifts every cell
    /// edge (measured ~15% of pixels change with SA enabled), so it is NOT skip-safe despite reading
    /// only the final z. This set adds orbit-trap versus [`Self::needs_aux`] — the deep-zoom bug
    /// where trap was absent and thus wrongly kept BLA/SA on, silently dropping skipped-run minima.
    pub(crate) fn blocks_iter_skip(self) -> bool {
        matches!(
            self,
            ColorMethod::Stripe
                | ColorMethod::TriangleIneq
                | ColorMethod::OrbitTrap
                | ColorMethod::Decomposition
        )
    }
}

/// Orbit-trap shape (only meaningful for [`ColorMethod::OrbitTrap`]). Discriminants match the shader.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum TrapType {
    #[default]
    Point = 0,
    Cross = 1,
    Circle = 2,
}
impl TrapType {
    pub(crate) const ALL: [TrapType; 3] = [TrapType::Point, TrapType::Cross, TrapType::Circle];
    pub(crate) fn to_u32(self) -> u32 {
        self as u32
    }
    pub(crate) fn key(self) -> &'static str {
        match self {
            TrapType::Point => "point",
            TrapType::Cross => "cross",
            TrapType::Circle => "circle",
        }
    }
    pub(crate) fn label(self) -> &'static str {
        match self {
            TrapType::Point => "Point",
            TrapType::Cross => "Cross",
            TrapType::Circle => "Circle",
        }
    }
    pub(crate) fn from_key(s: &str) -> TrapType {
        TrapType::ALL.into_iter().find(|t| t.key() == s).unwrap_or_default()
    }
}

/// The numeric representation the renderer uses, chosen by depth (matches the shader's `mode`).
/// Serialized to `u32` only when written into the GPU uniforms / export request.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum RenderMode {
    /// df32 perturbation of a reference orbit (`1e4× … PERT_FE_THRESHOLD`).
    Df32Pert = 0,
    /// Direct df32 from z₀, no reference (shallow `< 1e4×`, or a non-perturbation formula).
    Direct = 1,
    /// Extended-range floatexp perturbation (`≥ PERT_FE_THRESHOLD`).
    Floatexp = 2,
}

/// A session's saved zoom, validated. `None` means "unusable — open at the default view instead".
///
/// The session restore was an ENTRY POINT WITHOUT A GUARD, the same shape as the tour `zoom` string
/// that once sized a usize::MAX-bit allocation while the `.fdn` reader validated properly. Here
/// `FloatExp::new(mantissa, exp)` was fed straight from the file, so a corrupted session handed the
/// app a NaN zoom (field case 2026-08-18, build 1678: black screen, "iter capped", laggy desktop).
/// `RenderMode::select` then compounded it by mapping NaN to the most expensive mode; that is fixed
/// separately, and this stops the bad value entering at all.
///
/// Rejecting only the zoom, not the centre: a garbage magnification with a good centre still opens
/// somewhere recognisable, and throwing away a 40-digit centre the user may not be able to retype is
/// the more destructive failure.
pub(crate) fn restored_units_per_pixel(mantissa: f64, exp: i32) -> Option<fractadyne_core::FloatExp> {
    if !mantissa.is_finite() || mantissa <= 0.0 {
        return None;
    }
    // Our deepest real views sit near a binary exponent of -7000 (about 1e2100x). Orders past that
    // are corruption, not ambition.
    if exp.unsigned_abs() > 1_000_000 {
        return None;
    }
    let upp = fractadyne_core::FloatExp::new(mantissa, exp);
    // The decisive test, and the one the field case would have failed: the magnification this
    // implies has to be a real number.
    let mut probe = Viewport::new(1280.0, 720.0);
    probe.units_per_pixel = upp;
    if !probe.log2_magnification().is_finite() {
        return None;
    }
    Some(upp)
}

#[cfg(test)]
mod session_zoom {
    use super::restored_units_per_pixel;

    #[test]
    fn a_corrupted_zoom_is_refused() {
        // The field case: whatever produced it, a non-finite zoom must not reach the viewport.
        assert!(restored_units_per_pixel(f64::NAN, -9).is_none());
        assert!(restored_units_per_pixel(f64::INFINITY, -9).is_none());
        assert!(restored_units_per_pixel(f64::NEG_INFINITY, -9).is_none());
        // Zero or negative pixel size is not a view, it is a division waiting to happen.
        assert!(restored_units_per_pixel(0.0, -9).is_none());
        assert!(restored_units_per_pixel(-1.44, -9).is_none());
        // An exponent orders beyond any real depth is corruption.
        assert!(restored_units_per_pixel(1.44, i32::MIN).is_none());
        assert!(restored_units_per_pixel(1.44, i32::MAX).is_none());
    }

    #[test]
    fn real_sessions_still_load() {
        // The actual value on disk after this incident was resolved (zoom ~1.06x).
        assert!(restored_units_per_pixel(1.449069213429062, -9).is_some());
        // And depths across the range the app genuinely reaches, including past the df32 crossover
        // and out to the documented e2100 wall — none of these may be mistaken for corruption.
        for e in [0, -30, -100, -1000, -3700, -7000] {
            assert!(restored_units_per_pixel(1.5, e).is_some(), "legitimate depth 2^{e} refused");
        }
    }
}

/// Whether this process was launched to run a HARNESS or an offline job rather than to be sat in
/// front of. Used only to decide that a lost device must NOT relaunch.
///
/// A harness that resurrects itself is worse than one that dies. `--selftest` and `--livetest` drive
/// the real windowed app (which is why the welcome dialog once blocked them), so before this the
/// device-lost handler would spawn a FRESH PROCESS RE-RUNNING THE SAME TASK FLAGS: a `--torture` rung
/// would orphan a GUI window its supervisor knows nothing about, and a `--livetest` could end up with
/// two concurrent gates writing the same log. The guard was previously `elapsed_s() > 60`, which hid
/// this for short runs by accident; it needs to be explicit.
///
/// Deliberately BROADER than `launched_for_a_task` (which exists for the welcome dialog): listing an
/// extra flag here only ever means "do not relaunch", which is the safe direction. ⚠Add new harness
/// and offline-job flags here.
pub(crate) fn is_task_invocation<S: AsRef<str>>(args: &[S]) -> bool {
    const TASK_FLAGS: &[&str] = &[
        "--selftest", "--livetest", "--divetest", "--uitest", "--juliadive", "--play-tour",
        "--bench-matrix", "--benchmark", "--profile", "--reusetest", "--resizetest", "--frametest",
        "--render", "--render-tour", "--torture", "--gputest", "--oomtest", "--refdiag",
        "--find-minibrot", "--check-updates", "--crosscheck-f3", "--autodive", "--motiontest",
        "--chunk-sweep", "--bench-bignum",
    ];
    args.iter().any(|a| TASK_FLAGS.contains(&a.as_ref()))
}

#[cfg(test)]
mod task_invocation {
    use super::is_task_invocation;

    #[test]
    fn a_bare_interactive_launch_is_not_a_task() {
        assert!(!is_task_invocation::<&str>(&[]));
        assert!(!is_task_invocation(&["--fast"]));
        // A view handed over on the command line is still someone sitting in front of the app.
        assert!(!is_task_invocation(&["--center", "-0.75", "0.0"]));
    }

    #[test]
    fn every_harness_and_offline_job_is_a_task() {
        // These all drive the real windowed app or run long unattended, and none of them may
        // resurrect itself on a lost device.
        for flag in [
            "--selftest", "--livetest", "--uitest", "--juliadive", "--torture", "--render",
            "--render-tour", "--bench-matrix", "--gputest", "--resizetest", "--motiontest",
        ] {
            assert!(is_task_invocation(&[flag]), "{flag} must count as a task invocation");
            // ...including when it is not the first argument.
            assert!(is_task_invocation(&["--size", "480x270", flag]), "{flag} missed mid-argv");
        }
    }
}

/// Whether a lost device may relaunch, and as which generation. `None` means stop.
///
/// Split out as a pure function so the policy is testable: it decides whether the user sees a
/// recovery or a crash, and the previous version of it (`elapsed_s() > 60`) shipped untested and
/// was wrong in the field. See `relaunch_after_device_loss` for the reasoning.
pub(crate) fn relaunch_decision(generation: u32, elapsed_s: f64) -> Option<u32> {
    const MAX_GENERATIONS: u32 = 3;
    if generation >= MAX_GENERATIONS {
        return None;
    }
    // A restarted generation that dies again almost immediately was not helped by restarting.
    if generation > 0 && elapsed_s < 15.0 {
        return None;
    }
    Some(generation + 1)
}

#[cfg(test)]
mod relaunch_policy {
    use super::relaunch_decision;

    #[test]
    fn a_first_loss_recovers_at_any_uptime() {
        // THE FIELD CASE (2026-08-18): a deep view + Home glide lost the device at 50.4s. The old
        // `elapsed_s() > 60` guard refused to restart, so a loss the app is designed to recover
        // from was experienced as a hard crash. 50.4s must restart.
        assert_eq!(relaunch_decision(0, 50.4), Some(1));
        // And the guard must not have simply moved: an immediate first loss still recovers once,
        // because one relaunch is cheap and the generation cap is what bounds a loop.
        assert_eq!(relaunch_decision(0, 0.2), Some(1));
        assert_eq!(relaunch_decision(0, 3600.0), Some(1));
    }

    #[test]
    fn a_relaunch_that_did_not_help_stops() {
        // Restarted, then died again within 15s: restarting is not working, so stop rather than
        // spin. This is the case the original uptime guard was really aiming at.
        assert_eq!(relaunch_decision(1, 2.0), None);
        assert_eq!(relaunch_decision(2, 14.9), None);
        // But a restarted generation that ran a while before dying gets another go.
        assert_eq!(relaunch_decision(1, 15.0), Some(2));
        assert_eq!(relaunch_decision(2, 600.0), Some(3));
    }

    #[test]
    fn the_generation_cap_terminates_any_loop() {
        // However healthy each generation looks, the chain is bounded — no restart loop can
        // outlive the cap even with long uptimes between losses.
        assert_eq!(relaunch_decision(3, 10_000.0), None);
        assert_eq!(relaunch_decision(9, 10_000.0), None);
        let mut gen = 0;
        let mut hops = 0;
        while let Some(next) = relaunch_decision(gen, 1_000.0) {
            gen = next;
            hops += 1;
            assert!(hops <= 8, "relaunch chain did not terminate");
        }
        assert_eq!(hops, 3, "the chain must stop after exactly MAX_GENERATIONS hops");
    }
}

impl RenderMode {
    /// Pick the representation for a view: direct when shallow or the formula has no perturbation,
    /// then df32, switching to floatexp past `PERT_FE_THRESHOLD`. The one place this is decided.
    /// `julia` selects the much lower direct→perturbation crossover (see `PERT_JULIA_THRESHOLD`).
    pub(crate) fn select(supports_perturbation: bool, julia: bool, mag: f64) -> RenderMode {
        // ⚠**NaN ONLY — never `!is_finite()`.** A NaN zoom is garbage input and must pick the
        // safest mode: every comparison against NaN is false, so the chain below falls through to
        // `Floatexp`, and a corrupted session (2026-08-18, build 1678) silently selected the most
        // expensive arithmetic at maximum depth — black screen, "iter capped", laggy desktop, a
        // device loss waiting to happen (`arithmetic mode none → 2 at frame 1 (mag 2^NaN)`).
        // Direct is right for garbage: no reference orbit, cheapest, and it renders *something*
        // rather than wedging the GPU.
        //
        // ⭐But `+∞` IS NOT GARBAGE, and catching it here was a REGRESSION (`c59bda0`, 2026-08-17
        // → fixed beta.124). `Viewport::magnification()` returns an f64 and therefore SATURATES
        // to `+∞` past ~1e308×, so this guard silently demoted every genuinely extreme view to
        // non-perturbation Direct — no reference, no BLA — and rendered a BLANK IMAGE. It reached
        // the bench kit: the 4.6e1105× corpus scene "finished" in 1.8 s against Fraktaler-3's
        // 258 s, and the 144× win was measuring an empty frame. Nothing caught it for four days
        // because the F3 corpus gate (the only thing that renders past 1e308) last ran green on
        // 2026-08-14, three days BEFORE the guard landed — and the unit test written with it
        // asserted `INFINITY → Direct`, enshrining the defect. An infinite magnification means
        // "deeper than an f64 can say", which is precisely what `Floatexp` exists for, so it must
        // fall through to the bottom of the chain.
        if mag.is_nan() {
            return RenderMode::Direct;
        }
        let direct_below = if julia { PERT_JULIA_THRESHOLD } else { 1.0e4 };
        if !supports_perturbation || mag < direct_below {
            RenderMode::Direct
        } else if mag < PERT_FE_THRESHOLD {
            RenderMode::Df32Pert
        } else {
            RenderMode::Floatexp
        }
    }
    pub(crate) fn to_u32(self) -> u32 {
        self as u32
    }
    pub(crate) fn from_u32(v: u32) -> RenderMode {
        match v {
            1 => RenderMode::Direct,
            2 => RenderMode::Floatexp,
            _ => RenderMode::Df32Pert,
        }
    }
    /// Direct path — no reference orbit, never glitches.
    pub(crate) fn is_direct(self) -> bool {
        matches!(self, RenderMode::Direct)
    }
    /// Extended-range floatexp path (the deep, ~5×-costlier mode-2).
    pub(crate) fn is_floatexp(self) -> bool {
        matches!(self, RenderMode::Floatexp)
    }
}

/// Fixed export aspect ratios (key, width ÷ height). "window" (not listed) matches the live view.
///
/// The keys are EXACT ratios, not marketing names, because [`aspect_key_for`] has to reproduce a
/// preset's stated height to within half a pixel. "21:9" is the trap: no ultrawide display is
/// actually 21:9 (2.333) — 2560×1080 is 64:27 (2.370) and 3440×1440 is 43:18 (2.389) — so a single
/// "21:9" entry would render both at the wrong height. They get one key each, and the friendly
/// names live in [`STANDARD_SIZES`] where users actually pick a size.
const EXPORT_ASPECTS: [(&str, f64); 12] = [
    ("16:9", 16.0 / 9.0),
    ("16:10", 16.0 / 10.0),
    ("3:2", 3.0 / 2.0),
    ("4:3", 4.0 / 3.0),
    ("1:1", 1.0),
    ("2:3", 2.0 / 3.0),
    ("9:16", 9.0 / 16.0),
    ("2:1", 2.0),
    ("64:27", 64.0 / 27.0),   // 21:9 ultrawide — 2560×1080, 3840×1620
    ("43:18", 43.0 / 18.0),   // 21:9 ultrawide — 3440×1440 (UWQHD)
    ("32:9", 32.0 / 9.0),     // super ultrawide — 5120×1440
    ("256:135", 256.0 / 135.0), // DCI cinema — 2048×1080, 4096×2160
];

/// Standard output sizes offered by the export and tour-render dialogs: (label, width, height).
///
/// Every entry's ratio matches an [`EXPORT_ASPECTS`] key, because the image dialog stores a width
/// plus an aspect and derives the height — a preset whose ratio it cannot express would silently
/// render at a different size than its own label claims. [`aspect_key_for`] is what enforces that,
/// and the `export-sizes` selftest fails if a row is ever added that breaks it.
pub(crate) const STANDARD_SIZES: &[(&str, u32, u32)] = &[
    ("HD 720p — 1280×720", 1280, 720),
    ("Full HD 1080p — 1920×1080", 1920, 1080),
    ("WUXGA — 1920×1200", 1920, 1200),
    ("QHD 1440p — 2560×1440", 2560, 1440),
    ("WQXGA — 2560×1600", 2560, 1600),
    ("4K UHD — 3840×2160", 3840, 2160),
    ("5K — 5120×2880", 5120, 2880),
    ("8K UHD — 7680×4320", 7680, 4320),
    ("UXGA 4:3 — 1600×1200", 1600, 1200),
    ("3:2 — 1920×1280", 1920, 1280),
    ("Square — 2048×2048", 2048, 2048),
    ("Portrait 9:16 — 1080×1920", 1080, 1920),
    ("UW-UXGA 21:9 — 2560×1080", 2560, 1080),
    ("UWQHD 21:9 — 3440×1440", 3440, 1440),
    ("UW4K 21:9 — 3840×1620", 3840, 1620),
    ("Super ultrawide 32:9 — 5120×1440", 5120, 1440),
    ("DCI 2K — 2048×1080", 2048, 1080),
    ("DCI 4K — 4096×2160", 4096, 2160),
];

/// The [`EXPORT_ASPECTS`] key matching `w × h`, or `None` when no listed ratio reproduces it.
/// The tolerance is half a pixel of height at this width: that is exactly the condition for the
/// dialog's `round()`-ed height to come back as `h`, so a `Some` here is a promise the preset
/// renders at its stated size. It is looser than exact equality on purpose: 1366×768 is not 16:9
/// (1.7786 vs 1.7778) yet 1366/(16/9) rounds to exactly 768, so it IS reproducible and rejecting
/// it would be wrong. A true mismatch — 3440×1440, whose 21:9 no key expresses — still fails.
pub(crate) fn aspect_key_for(w: u32, h: u32) -> Option<&'static str> {
    let (w, h) = (w as f64, h.max(1) as f64);
    EXPORT_ASPECTS
        .iter()
        .find(|(_, r)| ((w / r) - h).abs() < 0.5)
        .map(|(k, _)| *k)
}


/// Curated famous Mandelbrot locations: (name, center_x, center_y, magnification).
/// Coordinates are full-precision strings so deep entries land exactly.
const FAMOUS: &[(&str, &str, &str, f64)] = &[
    ("Seahorse Valley", "-0.74364388703", "0.13182590421", 2.667e3),
    ("Elephant Valley", "0.2925755", "-0.0149977", 2.0e3),
    ("Triple Spiral", "-0.088643135", "0.654461185", 1.6e3),
    ("Double Spiral", "-0.7470837", "0.1080358", 4.0e3),
    ("Spiral Galaxy", "-0.7269", "0.1889", 2.667e3),
    ("Mini Mandelbrot", "-1.7687788", "0.0017388", 8.0e3),
    ("Deep Seahorse", "-0.743643887037151", "0.131825904205330", 1.333e7),
];

/// Curated well-known Misiurewicz points of interest: (name, center_re, center_im, magnification).
/// Every center is an exact pre-periodic point (verified by Newton solve); the name carries its
/// `(preperiod, period)`. Selecting one jumps there — and the parameterized finder re-derives any
/// such point near the current view to arbitrary precision. Mandelbrot only.
const MISIUREWICZ_POI: &[(&str, &str, &str, f64)] = &[
    ("Antenna tip — Misiurewicz (2,1)", "-2.0", "0.0", 8.0),
    ("Upper boundary c=i — Misiurewicz (2,2)", "0.0", "1.0", 8.0),
    (
        "Three-spar — Misiurewicz (4,1)",
        "-0.101096363845622161025785445739",
        "0.95628651080914150077109605773",
        1.0e5,
    ),
    (
        "Elephant spiral — Misiurewicz (7,1)",
        "0.424512719050039642442472214172",
        "0.207530228166745302506073482244",
        1.0e4,
    ),
    (
        "North antenna — Misiurewicz (4,3)",
        "-0.173006716092090164776138289468",
        "1.06275228084924256023519761268",
        1.0e4,
    ),
];

/// Zoom-appropriate iteration cap. A very high manual iteration count over-resolves the
/// boundary's sub-pixel "dust" into per-pixel noise (and starves the render budget); this
/// caps the count at a generous, zoom-scaled value so normal auto-iteration is never
/// limited but an inflated base is. Used for both the live view and exports so they match.
/// Zoom-appropriate iteration cap from the zoom **octaves** (`log2(magnification)`), taken
/// directly so it stays finite past 1e308× where `magnification()` saturates to `∞`.
pub(crate) fn zoom_iter_cap(octaves: f64) -> u32 {
    let o = octaves.max(0.0);
    (ZOOM_ITER_BASE + o * ZOOM_ITER_PER_OCTAVE).min(u32::MAX as f64) as u32
}








// Self-test helpers + run_selftest moved to selftest.rs.

/// Plain-f64 Mandelbrot escape test: `Some(iter)` if it escapes within `max`, else
/// `None` (treated as interior). Used by the random-location boundary search.
fn mandel_escapes(cx: f64, cy: f64, max: u32) -> Option<u32> {
    let (mut zx, mut zy) = (0.0_f64, 0.0_f64);
    for i in 0..max {
        let (x2, y2) = (zx * zx, zy * zy);
        if x2 + y2 > 4.0 {
            return Some(i);
        }
        zy = 2.0 * zx * zy + cy;
        zx = x2 - y2 + cx;
    }
    None
}

/// Sample packed gradient stops (`[r, g, b, pos]`, ascending) at `t∈0..1` — mirrors the
/// shader's `palette()` — and gamma-encode to a display `Color32`.
fn sample_stops(stops: &[[f32; 4]; fractadyne_color::MAX_STOPS], n: u32, t: f32) -> egui::Color32 {
    let t = t.fract();
    let mut col = [stops[0][0], stops[0][1], stops[0][2]];
    let n = n.max(1) as usize;
    for i in 0..n.saturating_sub(1) {
        let (a, b) = (stops[i], stops[i + 1]);
        if t >= a[3] && t <= b[3] {
            let f = (t - a[3]) / (b[3] - a[3]).max(1.0e-6);
            col = [
                a[0] + (b[0] - a[0]) * f,
                a[1] + (b[1] - a[1]) * f,
                a[2] + (b[2] - a[2]) * f,
            ];
            break;
        }
    }
    let g = |c: f32| (c.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0 + 0.5) as u8;
    egui::Color32::from_rgb(g(col[0]), g(col[1]), g(col[2]))
}

// ---------------- Help content -------------------------------------------------
// In-app Help content lives in help.rs (the help_* section fns; imported via use help::*).

// ---- minimap overview ----
/// Fixed complex region the minimap thumbnail covers (center + half-extents), so the
/// "you are here" marker projects consistently regardless of the screen aspect.
const MINIMAP_CX: f64 = -0.5;
const MINIMAP_CY: f64 = 0.0;
const MINIMAP_HX: f64 = 1.6;
const MINIMAP_HY: f64 = 1.2;
/// Thumbnail render resolution (display size is scaled down in the overlay).
const MINIMAP_TW: u32 = 240;
const MINIMAP_TH: u32 = 180;

/// "Zoom home" animation pacing: seconds per octave-ish of zoom-out, clamped so a
/// shallow view still glides and an extreme one doesn't take forever.
const HOME_SECONDS_PER_LOGMAG: f64 = 0.45;
const HOME_MIN_SECONDS: f64 = 1.5;
const HOME_MAX_SECONDS: f64 = 9.0;

/// An in-progress smooth zoom-out to the home view (started by the Home button).
struct HomeAnim {
    start_time: f64,
    duration: f64,
    /// Mandelbrot/main view: center when the animation began + its ln(magnification).
    m_start_center: (fractadyne_core::BigFloat, fractadyne_core::BigFloat),
    m_start_logmag: f64,
    /// Dual Julia view (only used when `dual`).
    j_start_center: (fractadyne_core::BigFloat, fractadyne_core::BigFloat),
    j_start_logmag: f64,
    dual: bool,
}

fn main() -> eframe::Result<()> {
    env_logger::init();
    // Expand `@response-file` args and `--args-file FILE` so the whole command line can live in a
    // text file (see `expand_arg_files`). Do it once, up front, so every consumer sees the result.
    let args = match expand_arg_files(&std::env::args().collect::<Vec<_>>()) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("fractadyne: {e}");
            crate::exit(2);
        }
    };
    // Crash/hang visibility (design/diagnostics.md D1): log file, panic hook, watchdog.
    // Before run_headless so even the pre-GUI CLI modes get crash reports.
    diag::init(&args);
    // ⭐ARBITRARY-PRECISION BACKEND (`--bignum auto|astro|rug`, or FRACTADYNE_BIGNUM).
    //
    // Chosen here — after logging exists, before ANY mode runs — so a headless `--render`, a
    // self-test and the GUI all iterate in the same arithmetic, and the choice lands in the log a
    // bug report will carry.
    //
    // ⚠The ENV VAR is the one to use for a batch or gate run. Harnesses (`--torture` per rung, the
    // corpus script per location) launch fractadyne as CHILD processes, so a flag on the parent
    // never reaches them — the same reason `FRACTADYNE_NO_SOUND` exists. An explicit flag outranks
    // the env var without unsetting it.
    //
    // ⚠A backend this build does not contain is FATAL, never a quiet fall back to astro-float: a
    // silent downgrade would let a benchmark report numbers for arithmetic it never ran.
    {
        let flag = args
            .iter()
            .position(|a| a == "--bignum")
            .and_then(|i| args.get(i + 1).map(String::as_str))
            .or_else(|| args.iter().find_map(|a| a.strip_prefix("--bignum=")));
        let env = std::env::var("FRACTADYNE_BIGNUM").ok();
        if let Some(spec) = flag.map(str::to_string).or(env) {
            match fractadyne_core::parse_backend_choice(&spec) {
                Ok(choice) => {
                    if let Err(e) = fractadyne_core::select_backend(choice) {
                        eprintln!("fractadyne: {e}");
                        crate::exit(2);
                    }
                    diag::log_line("start", &format!("bignum backend selected: {}", choice.name()));
                }
                Err(e) => {
                    eprintln!("fractadyne: --bignum: {e}");
                    crate::exit(2);
                }
            }
        }
    }
    // ⭐DEBUG TUNABLE OVERRIDES (`--set NAME=VALUE`, repeatable). Applied here — after logging
    // exists, before ANY mode runs — so a headless `--render`, a self-test and the GUI all get the
    // same values, and so the startup line lands in the log file that a bug report will carry.
    // A bad name or value is FATAL: a typo'd knob that silently did nothing would send a field
    // diagnosis chasing a change that never happened.
    {
        let mut pairs = Vec::new();
        let mut it = args.iter().skip(1);
        while let Some(a) = it.next() {
            let kv = if a == "--set" {
                it.next().map(String::as_str)
            } else {
                a.strip_prefix("--set=")
            };
            if let Some(kv) = kv {
                match kv.split_once('=') {
                    Some((k, v)) => pairs.push((k.to_string(), v.to_string())),
                    None => {
                        eprintln!("fractadyne: --set expects NAME=VALUE (got '{kv}')");
                        crate::exit(2);
                    }
                }
            }
        }
        if !pairs.is_empty() {
            if let Err(e) = tunables::apply_overrides(&pairs) {
                eprintln!("fractadyne: {e}");
                crate::exit(2);
            }
            // Loud, and in the log: every later reading in this process is off-stock.
            diag::log_line("start", &format!("⚠TUNABLES {}", tunables::status_line()));
            eprintln!(
                "fractadyne: ⚠tunable override(s) in effect — {}\n\
                 fractadyne: ⚠this is a DEBUGGING build state; the shipped defaults are the only \
                 tested path, and this run's results describe no released configuration.",
                tunables::status_line()
            );
        }
    }
    // `--oomtest`: force a real allocation failure, to prove the OOM path actually writes a crash
    // report. It cannot be verified any other way — an out-of-memory abort skips the panic hook,
    // which is the whole reason this machinery exists, and waiting for a genuine OOM means waiting
    // for the deep reference build to die again.
    if args.iter().any(|a| a == "--oomtest") {
        diag::breadcrumb("oomtest: requesting an impossible allocation".into());
        let n = usize::MAX / 2;
        // Deliberately unsatisfiable: the allocator returns null, `ReportingAlloc` reports, and
        // the runtime aborts on it exactly as a real exhaustion would.
        let v: Vec<u8> = Vec::with_capacity(n);
        std::hint::black_box(&v);
        crate::exit(0);
    }
    if cli::run_headless(&args) {
        return Ok(());
    }

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        // Bound frames-in-flight to 1 so a slow deep-zoom frame can't accumulate a growing present
        // queue — the swapchain backpressure that hung the UI thread on continuous df32 zoom-out.
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            desired_maximum_frame_latency: Some(1),
            // Request GPU timestamp queries when the adapter supports them, so the profiling
            // harness can time the iterate vs color passes in pure GPU time (see fractadyne_gpu::
            // timing). Replicates egui-wgpu's default device_descriptor, adding only the one feature
            // — the app degrades cleanly (CPU-timed columns only) on adapters that lack it.
            wgpu_setup: eframe::egui_wgpu::WgpuSetup::CreateNew(
                eframe::egui_wgpu::WgpuSetupCreateNew {
                    // PIN the backend set to the one this build is validated on. eframe's default
                    // is `PRIMARY | GL`, which includes DX12 — and since the app now compiles the
                    // DX12 backend in (for `--gputest`, see Cargo.toml), leaving the default would
                    // let a routine `cargo build` silently move every user onto a different shader
                    // compiler and a different driver path. Everything cost-related is calibrated
                    // against the current stack: the TDR step budgets, the dispatch caps, the
                    // blessed goldens and livetest baselines. Switching backends is a deliberate,
                    // re-measured decision, not a side effect. `WGPU_BACKEND` still overrides for
                    // experiments, and `--gputest` grades every compiled-in backend regardless.
                    instance_descriptor: eframe::wgpu::InstanceDescriptor {
                        backends: eframe::wgpu::Backends::from_env().unwrap_or(
                            eframe::wgpu::Backends::VULKAN | eframe::wgpu::Backends::GL,
                        ),
                        ..Default::default()
                    },
                    device_descriptor: std::sync::Arc::new(|adapter: &eframe::wgpu::Adapter| {
                        let base_limits =
                            if adapter.get_info().backend == eframe::wgpu::Backend::Gl {
                                eframe::wgpu::Limits::downlevel_webgl2_defaults()
                            } else {
                                eframe::wgpu::Limits::default()
                            };
                        let mut features = eframe::wgpu::Features::empty();
                        // `FRACTADYNE_NO_TIMESTAMPS=1` declines the feature even where the adapter
                        // offers it. Not a user setting — it is the only way to exercise the
                        // no-TIMESTAMP_QUERY path on a dev box whose GPU has it, and that path had
                        // a reproducible bug (budget stuck at bootstrap → ~1/3 resolution forever)
                        // that was invisible here for exactly that reason. Older Intel iGPUs, some
                        // Mesa/RADV/ANV combinations, and the GL backend all land on it for real.
                        let no_ts = std::env::var("FRACTADYNE_NO_TIMESTAMPS")
                            .is_ok_and(|v| v != "0" && !v.is_empty());
                        if !no_ts
                            && adapter
                                .features()
                                .contains(eframe::wgpu::Features::TIMESTAMP_QUERY)
                        {
                            features |= eframe::wgpu::Features::TIMESTAMP_QUERY;
                        }
                        // Reference-orbit headroom: the default 128 MB storage-binding limit caps
                        // the orbit+BLA buffer at ~928k samples — a wall the Misiurewicz spar
                        // family hits by ~1e82× (its reference needs >928k iterations to escape,
                        // so the view renders black there while shallower spar depths resolve).
                        // Ask the ADAPTER for up to 1 GiB (≈7.4M samples); a lesser adapter grants
                        // what it has, and `init_orbit_len_cap` sizes the cap from whatever was
                        // actually granted, so nothing here assumes a big GPU.
                        let want_binding: u32 = 1 << 30;
                        let adapter_limits = adapter.limits();
                        let binding = adapter_limits.max_storage_buffer_binding_size.min(want_binding);
                        let buffer = adapter_limits.max_buffer_size.min(want_binding as u64);
                        // Iteration-range tiling writes Rgba32Float state attachments per chunk
                        // pass: THREE for direct/mode-0 (48 bytes/sample; the wgpu default limit is
                        // 32), and FOUR for mode-2, whose floatexp state does not fit in three —
                        // δz mantissa (4) + δz exponent (1) + derivative mantissa (4) + derivative
                        // exponent (1) + status/iter (2) + ref_n (1) = 13 floats against 12, and the
                        // two exponents cannot share one f32 channel at our depths (at 1e1105 the
                        // binary exponent is ≈ −3670, so two fields need 13 bits each = 26 > the 24
                        // an f32 holds exactly). So ask for 64.
                        //
                        // ⚠MEASURED before choosing this: the RTX 3080 reports 128 AVAILABLE while we
                        // were requesting only 48, so the old ceiling was self-imposed, not hardware.
                        // A lesser adapter grants what it has and the affected chunk path gates itself
                        // off (`chunking_available` / `chunking_mode2_available`) — nothing here
                        // assumes a big GPU.
                        let attach_bytes = adapter_limits.max_color_attachment_bytes_per_sample.min(64);
                        eframe::wgpu::DeviceDescriptor {
                            label: Some("fractadyne device"),
                            required_features: features,
                            required_limits: eframe::wgpu::Limits {
                                max_texture_dimension_2d: 8192,
                                max_storage_buffer_binding_size: binding.max(base_limits.max_storage_buffer_binding_size),
                                max_buffer_size: buffer.max(base_limits.max_buffer_size),
                                max_color_attachment_bytes_per_sample: attach_bytes
                                    .max(base_limits.max_color_attachment_bytes_per_sample),
                                ..base_limits
                            },
                            memory_hints: eframe::wgpu::MemoryHints::default(),
                        }
                    }),
                    ..Default::default()
                },
            ),
            ..Default::default()
        },
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([640.0, 400.0])
            .with_title(format!("Fractadyne v{}", version_string()))
            .with_icon(brand_icon()),
        ..Default::default()
    };

    // Arm the unclean-exit marker around the event loop only (see `diag::begin_gui_session`).
    // Every deliberate exit goes through `crate::exit`, which disarms it, so a clean shutdown can
    // never be mistaken for a crash.
    diag::begin_gui_session();
    let r = eframe::run_native(
        "Fractadyne",
        native_options,
        Box::new(move |cc| Ok(Box::new(FractadyneApp::new(cc, &args)))),
    );
    diag::end_session();
    r
}

/// Terminate the process, disarming the unclean-exit marker first. EVERY `std::process::exit` in
/// this crate goes through here: `exit` runs no destructors and no hooks, so a marker left armed
/// by a deliberate quit would be reported as a crash on the next launch. One choke point is the
/// only way to be sure none was missed.
pub(crate) fn exit(code: i32) -> ! {
    diag::end_session();
    std::process::exit(code)
}

/// Direct download URL for the accelerated package matching `version`.
///
/// Version-MATCHED on purpose. The two builds share settings, saved session and locations, so a
/// user can move between them freely -- which is exactly why handing them a "latest" link would be
/// the one way to cause confusion here: they would be running two different feature sets against
/// one shared session without being told.
///
/// `version` is `sysinfo::version_string()`, which carries a "(build N)" suffix that is not part
/// of the tag. Pinned by test against the name `scripts/build-accelerated.ps1` actually produces:
/// if either side is renamed, the link dies silently, and a dead download link in a menu is worse
/// than no menu entry.
fn accelerated_asset_url(version: &str) -> String {
    let tag = format!("v{}", version.split_whitespace().next().unwrap_or(version));
    format!("https://github.com/WindySnowOwl/fractadyne/releases/download/{tag}/fractadyne-{tag}-windows-x64-accelerated.zip")
}

#[cfg(test)]
mod accelerated_link {
    use super::accelerated_asset_url;

    #[test]
    fn the_download_url_is_well_formed_and_version_matched() {
        let u = accelerated_asset_url("0.2.40-beta.156 (build 2076)");
        assert_eq!(
            u,
            "https://github.com/WindySnowOwl/fractadyne/releases/download/\
             v0.2.40-beta.156/fractadyne-v0.2.40-beta.156-windows-x64-accelerated.zip"
                .replace(' ', "")
        );
        // A URL containing a space is a dead link, and a `\` continuation inside a Rust string
        // literal is how one gets there. This is the assertion that catches it.
        assert!(!u.contains(' '), "URL contains a space: {u}");
        assert!(u.starts_with("https://"), "{u}");
        // Must match the artifact name `scripts/build-accelerated.ps1` builds.
        assert!(u.ends_with("-windows-x64-accelerated.zip"), "{u}");
    }

    #[test]
    fn a_bare_version_without_a_build_suffix_also_works() {
        let u = accelerated_asset_url("0.3.0");
        assert!(u.contains("/v0.3.0/"), "{u}");
        assert!(!u.contains(' '), "{u}");
    }
}

/// Parse the value of an option that WAS SUPPLIED, or exit saying so.
///
/// Replaces the `val("--x").and_then(|s| s.parse::<T>().ok())` idiom, which conflates two things
/// that are not alike: the option is ABSENT (fall back to the default — correct, and still what
/// happens, because the caller only reaches here inside `Some`), and the option is PRESENT with a
/// value we cannot read (fall back to the default — a bug, because the program then does real,
/// expensive, wrong work and exits 0). `--zoom` shipped in the second state long enough for the
/// benchmark kit to measure a whole-set frame against a deep one and report a ratio; the only
/// reason it was ever noticed is that somebody looked at the picture. Every option that steers a
/// render or a comparison goes through here now, so an unreadable value is a message, not a
/// plausible-looking result.
fn arg_parse<T: std::str::FromStr>(name: &str, s: &str, expect: &str) -> T {
    match s.parse::<T>() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("fractadyne: {name}: cannot read \"{s}\" as {expect}.");
            crate::exit(2)
        }
    }
}

/// `--size` as `WIDTH` or `WIDTHxHEIGHT`, fatal when supplied and unreadable. A silent fallback
/// here renders at the WRONG RESOLUTION and says nothing — the exact shape of the benchmark-kit
/// defect where one renderer was handed 1280x720 while every other lane ran 1920x1080.
fn arg_size(name: &str, s: &str) -> (Option<u32>, Option<u32>) {
    let parsed = parse_size(s);
    // A bare width is legitimate, but ONLY when there was no separator for a height to have
    // failed on. `parse_size` parses the halves independently and returns (Some(w), None) for
    // BOTH "1920" and "1920x108O", so accepting the second drops a height that WAS supplied,
    // without a word: the tour then renders at the script's height, and the single-image lane
    // at whatever aspect the restored session happens to have, which is not even reproducible.
    // That is this entire class again, inside the fix for it.
    let readable = match parsed {
        (Some(_), Some(_)) => true,
        (Some(_), None) => !size_has_separator(s),
        _ => false,
    };
    if !readable {
        eprintln!("fractadyne: {name}: cannot read \"{s}\" as WIDTH or WIDTHxHEIGHT.");
        crate::exit(2)
    }
    parsed
}

/// A full-precision coordinate pair, fatal when either half is unreadable. `parse_bf` returns
/// `None` on anything it cannot read and the `?` collapsed the PAIR, so one stray character in a
/// pasted y ordinate threw the whole location away and rendered the fractal default centre at the
/// requested depth — solid interior, full cost, exit 0.
fn arg_center(name: &str, xs: &str, ys: &str) -> (fractadyne_core::BigFloat, fractadyne_core::BigFloat) {
    let one = |t: &str| match fractadyne_core::parse_bf(t) {
        Some(v) => v,
        None => {
            eprintln!("fractadyne: {name}: cannot read \"{t}\" as a decimal coordinate.");
            crate::exit(2)
        }
    };
    (one(xs), one(ys))
}

/// Group an integer string with commas every 3 digits (handles a leading `-`).
fn commas(s: &str) -> String {
    let neg = s.starts_with('-');
    let digits = s.trim_start_matches('-');
    let len = digits.len();
    let mut out = String::with_capacity(len + len / 3 + 1);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if neg {
        format!("-{out}")
    } else {
        out
    }
}

/// Space-group the fractional digits of a scientific-notation string's mantissa (in 5s),
/// matching the coordinate readout: `3.38050027227e15` → `3.38050 02722 7e15`. The exponent
/// is left intact. For display only — `parse_zoom_to_log2` strips spaces, so it round-trips.
fn group_sci_mantissa(s: &str) -> String {
    let (mant, exp) = match s.split_once(['e', 'E']) {
        Some((m, e)) => (m, Some(e)),
        None => (s, None),
    };
    let grouped = match mant.split_once('.') {
        Some((int_part, frac)) => {
            let mut g = String::with_capacity(frac.len() + frac.len() / 5);
            for (i, c) in frac.chars().enumerate() {
                if i > 0 && i % 5 == 0 {
                    g.push(' ');
                }
                g.push(c);
            }
            format!("{int_part}.{g}")
        }
        None => mant.to_string(),
    };
    match exp {
        Some(e) => format!("{grouped}e{e}"),
        None => grouped,
    }
}

/// Magnification with comma-grouped integer part + 2 decimals, e.g. `1,805,359.12`.
fn fmt_zoom(mag: f64) -> String {
    if mag > 1.0e12 {
        // Deep zoom: scientific notation, 12 significant digits (a 30-digit integer is
        // unreadable). `{:.11e}` → e.g. `3.38050027227e15`, with the mantissa space-grouped.
        group_sci_mantissa(&format!("{mag:.11e}"))
    } else if mag >= 1000.0 {
        // Large integer magnification: comma-grouped, no decimals (the `.00` is clutter).
        commas(&format!("{mag:.0}"))
    } else {
        // Small zoom: up to 2 decimals, trailing zeros trimmed (e.g. `1.5`, `256`, `2.37`).
        let s = format!("{mag:.2}");
        let s = s.trim_end_matches('0').trim_end_matches('.');
        s.to_string()
    }
}

/// Format magnification from `log2(magnification)` — stays correct past `f64`'s 1e308×
/// (where `magnification()` saturates to `∞`), formatting `2^log2mag` via base-10.
/// Playback-speed label for the transport button: `0.5`, `1`, `2`, `4` — no trailing `.0`, since
/// the button is a readout as much as a control and "1.0x" reads like a measurement.
pub(crate) fn fmt_speed(speed: f64) -> String {
    if (speed - speed.round()).abs() < 1.0e-6 {
        format!("{}", speed.round() as i64)
    } else {
        format!("{speed}")
    }
}

/// Character slot the status bar reserves for one `fmt_zoom_log2` value.
///
/// ⭐⭐**A STATUS-BAR FIELD THAT CHANGES WIDTH IS A REFLOW BUG WAITING FOR ITS WINDOW SIZE.** One
/// glyph more on a window where the bar just fits wraps it to two lines; that resizes the central
/// panel; `central.rs`'s resize-detector reads any panel resize as an INTERACTION, bumps the view
/// generation, tears down the settle grid and re-renders — whose counters then change the width
/// again. The beta.70/71 saga fixed that loop for the limit LABEL; the numeric fields were left
/// unreserved and reproduced it in the field on 2026-08-25 (`iter 1,395,703` wrapping a deep dual
/// view's bar while the pane sat black).
///
/// ⚠Pinned by `zoom_slot_fits_every_magnification`, which sweeps the whole supported range rather
/// than trusting this number: the widest output is not obvious, since the formatter switches
/// between a grouped decimal and a grouped scientific form at 1020 octaves.
pub(crate) fn zoom_slot_width() -> usize {
    21
}

/// Character slot for the status bar's grouped iteration count — `MAX_ITER_LIMIT` is the widest
/// value it can ever show. Same reflow reasoning as [`zoom_slot_width`].
pub(crate) fn iter_slot_width() -> usize {
    commas(&MAX_ITER_LIMIT.to_string()).chars().count()
}

#[cfg(test)]
mod status_bar_slots {
    use super::{commas, fmt_zoom_log2, iter_slot_width, zoom_slot_width, MAX_ITER_LIMIT};

    #[test]
    fn zoom_slot_fits_every_magnification() {
        // Sweep octaves across the whole supported range, densely around the 1020-octave switch
        // between the decimal and scientific forms, and past e21000 (the deepest view this app
        // has rendered). A slot that is one char short reintroduces the wrap loop.
        let mut worst = (0usize, 0.0f64, String::new());
        let mut l2 = 0.0f64;
        while l2 <= 72_000.0 {
            let w = fmt_zoom_log2(l2).chars().count();
            if w > worst.0 {
                worst = (w, l2, fmt_zoom_log2(l2));
            }
            l2 += if (1015.0..1025.0).contains(&l2) { 0.05 } else { 0.37 };
        }
        assert!(
            worst.0 <= zoom_slot_width(),
            "zoom slot {} too narrow: {} chars at log2mag {} ({:?})",
            zoom_slot_width(), worst.0, worst.1, worst.2
        );
    }

    #[test]
    fn iter_slot_fits_the_iteration_ceiling() {
        assert_eq!(iter_slot_width(), commas(&MAX_ITER_LIMIT.to_string()).chars().count());
        // Every value the bar can show must fit the slot it reserves.
        for v in [1u32, 999, 1_000, 35_733, 224_000, 1_395_703, MAX_ITER_LIMIT] {
            assert!(
                commas(&v.to_string()).chars().count() <= iter_slot_width(),
                "{v} does not fit the reserved iteration slot"
            );
        }
    }
}

pub(crate) fn fmt_zoom_log2(log2mag: f64) -> String {
    if log2mag <= 1020.0 {
        fmt_zoom(2f64.powf(log2mag.max(0.0)))
    } else {
        let log10 = log2mag * std::f64::consts::LOG10_2;
        let e = log10.floor();
        let m = 10f64.powf(log10 - e);
        group_sci_mantissa(&format!("{m:.2}e{e:.0}"))
    }
}

/// Magnification as a plain scientific string (no grouping), parseable by
/// [`parse_zoom_to_log2`] and valid past f64 range — used to pre-fill the go-to field and to write
/// the `.fdn` / bookmark `zoom=` field (`magnification()` saturates to `inf` past ~1e308×).
pub(crate) fn fmt_zoom_field(log2mag: f64) -> String {
    if log2mag <= 1020.0 {
        format!("{:e}", 2f64.powf(log2mag.max(0.0)))
    } else {
        let log10 = log2mag * std::f64::consts::LOG10_2;
        let e = log10.floor();
        let m = 10f64.powf(log10 - e);
        format!("{m:.6}e{e:.0}")
    }
}

/// Parse a magnification string (plain or scientific, e.g. `256`, `1.5e400`) into
/// `log2(magnification)`, reading the base-10 exponent directly so values far past f64
/// range still work. Grouping (`,` `_` spaces) is ignored. `None` on garbage / non-positive.
fn parse_zoom_to_log2(s: &str) -> Option<f64> {
    let t: String = s.chars().filter(|c| !matches!(c, ',' | '_' | ' ' | '\t')).collect();
    if t.is_empty() {
        return None;
    }
    let (mant, exp) = match t.split_once(['e', 'E']) {
        Some((m, x)) => (m, x.parse::<f64>().ok()?),
        None => (t.as_str(), 0.0),
    };
    let m: f64 = mant.parse().ok()?;
    if !(m.is_finite() && m > 0.0 && exp.is_finite()) {
        return None;
    }
    Some(m.log2() + exp * std::f64::consts::LOG2_10)
}

/// Coordinate with fractional digits grouped in 5s by spaces, e.g.
/// `-0.64939 71837 00000`.
fn fmt_coord(v: f64) -> String {
    let sign = if v.is_sign_negative() { "-" } else { "+" };
    let s = format!("{:.15}", v.abs());
    match s.split_once('.') {
        Some((int_part, frac)) => {
            let mut g = String::with_capacity(frac.len() + frac.len() / 5);
            for (i, c) in frac.chars().enumerate() {
                if i > 0 && i % 5 == 0 {
                    g.push(' ');
                }
                g.push(c);
            }
            format!("{sign}{int_part}.{g}")
        }
        None => format!("{sign}{s}"),
    }
}

/// Decompose a decimal/scientific number string into positional parts (sign, integer
/// digits, fractional digits) — `astro_float`'s `Display` is scientific (`7.43e-1`,
/// `5.e-1`), so shift the point by the exponent to recover plain `0.743…` form.
fn decimal_parts(s: &str) -> (&'static str, String, String) {
    let (sign, body) = match s.strip_prefix('-') {
        Some(r) => ("-", r),
        None => ("+", s.trim_start_matches('+')),
    };
    let (mant, exp) = match body.split_once(['e', 'E']) {
        Some((m, e)) => (m, e.trim().parse::<i32>().unwrap_or(0)),
        None => (body, 0),
    };
    let (mant_int, mant_frac) = match mant.split_once('.') {
        Some((i, f)) => (i, f),
        None => (mant, ""),
    };
    let digits: String = format!("{mant_int}{mant_frac}");
    // Digits to the left of the point: the mantissa's integer width, shifted by the exponent.
    let point = mant_int.len() as i32 + exp;
    let n = digits.len() as i32;
    if point <= 0 {
        (sign, "0".to_string(), format!("{}{digits}", "0".repeat((-point) as usize)))
    } else if point >= n {
        (sign, format!("{digits}{}", "0".repeat((point - n) as usize)), String::new())
    } else {
        let p = point as usize;
        (sign, digits[..p].to_string(), digits[p..].to_string())
    }
}

/// Format an arbitrary-precision coordinate for the status bar, showing enough significant
/// fractional digits for the current zoom (an `f64` readout freezes at ~15 digits, so deep
/// pans look static). Past a width threshold the middle is elided — `leading … frontier` —
/// so the deepest, *changing* digits stay on screen, e.g. `-0.74364 38870 … 06114 7740`.
pub(crate) fn fmt_coord_deep(v: &fractadyne_core::BigFloat, log2mag: f64) -> String {
    // Group a digit run in 5s for readability.
    let group = |digits: &str| -> String {
        let mut g = String::with_capacity(digits.len() + digits.len() / 5);
        for (i, c) in digits.chars().enumerate() {
            if i > 0 && i % 5 == 0 {
                g.push(' ');
            }
            g.push(c);
        }
        g
    };
    let (sign, int_part, frac) = decimal_parts(&fractadyne_core::to_decimal_string(v));
    if frac.is_empty() {
        return format!("{sign}{int_part}");
    }
    // Significant fractional digits worth showing ≈ decimal octaves of zoom + guard, capped
    // by what's actually stored. Floor of 15 keeps shallow views looking like `fmt_coord`
    // (but never asks for more digits than exist — avoids a min>max clamp on short coords).
    let want = (log2mag * std::f64::consts::LOG10_2).max(0.0).ceil() as usize + 6;
    let lo = 15.min(frac.len());
    let d = want.clamp(lo, frac.len());
    const HEAD: usize = 10; // leading digits kept
    const TAIL: usize = 10; // frontier digits kept
    if d <= HEAD + TAIL + 5 {
        format!("{sign}{int_part}.{}", group(&frac[..d]))
    } else {
        format!(
            "{sign}{int_part}.{} … {}",
            group(&frac[..HEAD]),
            group(&frac[d - TAIL..d])
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExportFormat {
    Png,
    Exr,
}

/// How the dual view is exported.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DualExport {
    /// Both panels stitched into one image (Mandelbrot | Julia).
    SideBySide,
    /// Two files (`…_map` and `…_julia`).
    Separate,
    /// Just the main/left panel.
    ActiveOnly,
}

// System-facts helpers (process_memory, SysInfo, gather_system_info, CPU/VRAM probes)
// moved to sysinfo.rs (re-exported below).
// ---- Scripting: keyframe camera tours (also drives the benchmark) ----

// Scripting/benchmark types + helpers moved to scripting.rs.

/// Palette animation mode (continuously shifts the color offset).
#[derive(Clone, Copy, PartialEq, Eq)]
enum PaletteAnim {
    Off,
    Forward,
    Reverse,
    PingPong,
    Random,
}

impl PaletteAnim {
    pub(crate) const ALL: [PaletteAnim; 5] = [
        PaletteAnim::Off,
        PaletteAnim::Forward,
        PaletteAnim::Reverse,
        PaletteAnim::PingPong,
        PaletteAnim::Random,
    ];
    fn name(self) -> &'static str {
        match self {
            PaletteAnim::Off => "Off",
            PaletteAnim::Forward => "Forward",
            PaletteAnim::Reverse => "Reverse",
            PaletteAnim::PingPong => "Ping-pong",
            PaletteAnim::Random => "Random gradients",
        }
    }
    pub(crate) fn key(self) -> &'static str {
        match self {
            PaletteAnim::Off => "off",
            PaletteAnim::Forward => "forward",
            PaletteAnim::Reverse => "reverse",
            PaletteAnim::PingPong => "pingpong",
            PaletteAnim::Random => "random",
        }
    }
    pub(crate) fn from_key(s: &str) -> PaletteAnim {
        match s {
            "forward" => PaletteAnim::Forward,
            "reverse" => PaletteAnim::Reverse,
            "pingpong" => PaletteAnim::PingPong,
            "random" => PaletteAnim::Random,
            _ => PaletteAnim::Off,
        }
    }
}

/// A render+write job handed to the background export worker.
pub(crate) enum ExportJob {
    Single(fractadyne_gpu::ExportRequest),
    SideBySide(fractadyne_gpu::ExportRequest, fractadyne_gpu::ExportRequest),
    Separate(fractadyne_gpu::ExportRequest, fractadyne_gpu::ExportRequest),
}

/// Stitch two rendered images horizontally (left | right) into one RGBA buffer.
pub(crate) fn stitch_side_by_side(
    a: &fractadyne_gpu::ExportResult,
    b: &fractadyne_gpu::ExportResult,
) -> (u32, u32, Vec<f32>) {
    let h = a.height.max(b.height);
    let w = a.width + b.width;
    let mut px = vec![0.0f32; (w as usize) * (h as usize) * 4];
    for i in (3..px.len()).step_by(4) {
        px[i] = 1.0; // opaque black background (for any height mismatch)
    }
    let blit = |dst: &mut [f32], src: &[f32], sw: u32, sh: u32, x0: u32| {
        let row = sw as usize * 4;
        for y in 0..sh as usize {
            let s = y * row;
            let d = (y * w as usize + x0 as usize) * 4;
            dst[d..d + row].copy_from_slice(&src[s..s + row]);
        }
    };
    blit(&mut px, &a.pixels, a.width, a.height, 0);
    blit(&mut px, &b.pixels, b.width, b.height, a.width);
    (w, h, px)
}

/// Derive `…_map` / `…_julia` sibling paths for a "separate" dual export.
pub(crate) fn separate_paths(path: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("export");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
    let dir = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    (
        dir.join(format!("{stem}_map.{ext}")),
        dir.join(format!("{stem}_julia.{ext}")),
    )
}

/// Read one `key=value` line out of an embedded metadata blob.
fn meta_get(meta: &str, key: &str) -> String {
    meta.lines()
        .find_map(|l| {
            l.split_once('=')
                .filter(|(k, _)| k.trim() == key)
                .map(|(_, v)| v.trim().to_string())
        })
        .unwrap_or_default()
}

/// Per-view cached perturbation reference orbit (arbitrary precision).
struct RefCache {
    ref_pt: Option<[fractadyne_core::BigFloat; 2]>,
    orbit: std::sync::Arc<Vec<[f32; 4]>>,
    orbit_len: u32,
    /// Bumped whenever `orbit` changes (tells the GPU to re-upload).
    orbit_id: u64,
    /// Precision / iteration count the cached orbit was computed at.
    orbit_prec: usize,
    orbit_iter: u32,
    /// True while the cached orbit is a TRUNCATED (coarse) reference from a progressive cold start —
    /// the render caps iterations to its length so it never rebases past it. Cleared when the full
    /// reference installs.
    partial: bool,
    /// Full-precision running state at the end of the cached orbit (`Z`, `Z_prev`, escaped flag), so a
    /// deeper rebuild at the SAME reference point can EXTEND this orbit instead of recomputing every
    /// step — the deep-dive reuse win (the bignum orbit build dominates a deep frame). `None` when no
    /// extendable orbit is cached (cold start, escaped/complete orbit, or a persisted reference).
    orbit_tail: Option<fractadyne_core::OrbitTail>,
    /// When the orbit was last recomputed (throttles refresh during interaction).
    last_recompute: Option<Instant>,
    /// Cached series-approximation skip + coefficients for this reference, and the
    /// `(orbit_id, eff_iter)` it was computed for (recompute only when that changes — the
    /// bignum coefficient iteration is as costly as the reference orbit itself).
    sa: fractadyne_core::SeriesSkip,
    sa_key: (u64, u32),
    /// Cached BLA tree (GPU-packed) for this reference, the `orbit_id` it was built for, and the
    /// `log2` of the worst-case `|δc|` (`dc_max`) it was built with. Rebuilt only when the orbit
    /// changes or the view zooms out enough that a larger `dc_max` is needed — so BLA doesn't pay a
    /// per-frame tree rebuild (panning within a reference reuses it). Empty = none/not built.
    bla: std::sync::Arc<Vec<[f32; 4]>>,
    bla_id: u64,
    bla_dc_max_log2: f64,
    /// Stripe-average frequency the cached BLA tree's `agg_stripe` lane was built with. Stripe's
    /// aggregate is frequency-specific, so the tree is rebuilt when the live frequency drifts from
    /// this (only while the stripe method is active). `NEG_INFINITY` = never built / unknown.
    bla_stripe_freq: f64,
    /// Trap type the cached BLA tree's `agg_trap` lane was built with. The trap aggregate is
    /// trap-type-specific, so the tree is rebuilt when the live trap type changes (only while the
    /// orbit-trap method is active). `u32::MAX` = never built / unknown.
    bla_trap_type: u32,
    /// View (center + log2 magnification) the current iteration texture was rendered at. A freeze
    /// uses this to zoom-reproject the frozen frame — scaling/panning it to follow the dive until a
    /// fresh reference lands (see `build_params`). `None` until the first real render.
    frozen_center: Option<[fractadyne_core::BigFloat; 2]>,
    frozen_l2: f64,
    /// When the frozen frame was rendered — ages the reuse-hold so a SLOW dive still refreshes
    /// real detail on a time floor (`REFRESH_MAX_SECS`), not just every `REFRESH_OCTAVES` of zoom.
    frozen_at: Option<Instant>,
    /// log2(units-per-pixel) of the frozen frame — the resize-invariant ZOOM signal the time
    /// floor gates on (`log2mag` follows the window height, so a resize drifts it without any
    /// zoom; re-iterating an un-zoomed held frame on every resize tick caused the "squashed
    /// resize" judder).
    frozen_upp_l2: f64,
    /// Octaves the view has zoomed IN past the cached BLA's validity (recomputed each frame in
    /// `build_params`; 0 when not in the deep floatexp regime). This is the "reference pipeline is
    /// behind the dive" signal — script playback reads it to DILATE the tour clock (slow the dive)
    /// instead of zooming into an ever-staler reprojection (the deep-dive monocolor blur).
    last_depth_lag: f64,
}

impl Default for RefCache {
    fn default() -> Self {
        Self {
            ref_pt: None,
            orbit: std::sync::Arc::new(Vec::new()),
            orbit_len: 0,
            orbit_id: 0,
            orbit_prec: 0,
            orbit_iter: 0,
            partial: false,
            orbit_tail: None,
            last_recompute: None,
            sa: fractadyne_core::SeriesSkip::NONE,
            sa_key: (u64::MAX, u32::MAX),
            bla: std::sync::Arc::new(Vec::new()),
            bla_id: u64::MAX,
            bla_dc_max_log2: f64::NEG_INFINITY,
            bla_stripe_freq: f64::NEG_INFINITY,
            bla_trap_type: u32::MAX,
            frozen_center: None,
            frozen_l2: 0.0,
            frozen_at: None,
            frozen_upp_l2: 0.0,
            last_depth_lag: 0.0,
        }
    }
}

/// A saved view (bookmark). `meta` is the same key=value view-metadata blob used by
/// exports, restorable via `load_view_metadata`.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Bookmark {
    name: String,
    meta: String,
    /// Thumbnail id — a small PNG preview lives at `bookmark_thumbs/<thumb>.png`. Empty when
    /// none (older bookmarks, or the render hasn't happened yet).
    #[serde(default)]
    thumb: String,
}

/// TOML wrapper for the bookmarks file (`[[bookmark]]` array).
#[derive(Default, serde::Serialize, serde::Deserialize)]
struct BookmarkFile {
    #[serde(default)]
    bookmark: Vec<Bookmark>,
}

/// A navigation history entry (location only) for undo/redo.
#[derive(Clone)]
struct ViewSnapshot {
    cx: fractadyne_core::BigFloat,
    cy: fractadyne_core::BigFloat,
    upp: fractadyne_core::FloatExp,
    prec: usize,
}

/// One exported image discovered by the gallery browser.
struct GalleryEntry {
    path: std::path::PathBuf,
    meta: String,
    fractal: String,
    zoom: String,
    saved: String,
    notes: String,
    app_version: String,
    saved_unix: u64,
    thumb: Option<egui::TextureHandle>,
    thumb_tried: bool,
}

// `FractalInfo` / `FractalKind` moved to `fractal.rs` (re-exported at the top of this file).

/// An in-progress **zoom box** (Shift+drag): rubber-band rectangle from `start` to `end`
/// (egui points) that, on release, zooms so the box fills the panel. `is_julia` tags which
/// panel it belongs to (dual view).
struct ZoomBox {
    start: egui::Pos2,
    end: egui::Pos2,
    is_julia: bool,
}

/// Constrain a free drag (`start`→`end`) to the panel's aspect ratio, anchored at `start` and
/// clamped inside `rect`, so the resulting box zooms to fill without distortion.
fn aspect_zoom_box(start: egui::Pos2, end: egui::Pos2, rect: egui::Rect) -> egui::Rect {
    let aspect = (rect.width() / rect.height().max(1.0)).max(1e-3);
    let (dx, dy) = ((end.x - start.x).abs(), (end.y - start.y).abs());
    let sx = if end.x >= start.x { 1.0 } else { -1.0 };
    let sy = if end.y >= start.y { 1.0 } else { -1.0 };
    // Enclose the drag, then clamp to what fits inside `rect` from `start` (keeping aspect).
    let maxw = if sx > 0.0 { rect.max.x - start.x } else { start.x - rect.min.x };
    let maxh = if sy > 0.0 { rect.max.y - start.y } else { start.y - rect.min.y };
    let w = dx.max(dy * aspect).min(maxw.max(0.0)).min(maxh.max(0.0) * aspect);
    let corner = egui::pos2(start.x + sx * w, start.y + sy * (w / aspect));
    egui::Rect::from_two_pos(start, corner)
}

/// State of the "Go to location" dialog (transient — not persisted). One of the field groups
/// (from the completed refactor) that break up the flat `FractadyneApp` struct.
/// Which parameterized feature the Go-to dialog's finder solves for (Mandelbrot only).
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum FeatureKind {
    /// Pre-periodic branch/spiral center — Newton on `Z_{k+p}=Z_k` (preperiod `k`, period `p`).
    #[default]
    Misiurewicz,
    /// Nearest minibrot nucleus — `Z_n=0`, period auto-detected (same as the M-key snap).
    Minibrot,
}

#[derive(Default)]
struct GotoDialog {
    open: bool,
    x: String,
    y: String,
    zoom: String,
    msg: Option<String>,
    /// Feature-finder inputs (parameterized "go to feature").
    feat_kind: FeatureKind,
    feat_k: String,
    feat_p: String,
}

/// State of the "Share location" (`.fdn`) dialog (transient).
#[derive(Default)]
struct ShareDialog {
    open: bool,
    text: String,
    msg: Option<String>,
}

/// Benchmark-configuration dialog state (transient).
struct BenchConfig {
    standard: bool,
    res: BenchRes,
    depth: BenchDepth,
    burnin: bool,
    passes: u32,
}
impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            standard: true,
            res: BenchRes::P1080,
            depth: BenchDepth::Standard,
            burnin: false,
            passes: 10,
        }
    }
}

/// Gallery-browser dialog state (transient): open flag, scanned folder, and its entries.
#[derive(Default)]
struct GalleryState {
    open: bool,
    dir: std::path::PathBuf,
    entries: Vec<GalleryEntry>,
}

/// Navigation history for undo/redo of view changes (transient).
#[derive(Default)]
struct NavHistory {
    undo: Vec<ViewSnapshot>,
    redo: Vec<ViewSnapshot>,
    was_interacting: bool,
}

/// Export-dialog state: the dialog's options, the in-flight background export, and the persisted
/// target folder. Grouped from the former flat `export_*` fields (Phase 2a).
struct ExportState {
    open: bool,
    width: u32,
    ss: u32,
    format: ExportFormat,
    dual_mode: DualExport,
    /// Aspect ratio: "window" (match the live view) or a fixed key ("16:9", "1:1", …).
    aspect: String,
    /// The size dropdown is on "Custom…", so the width/aspect fields are shown for direct editing.
    /// Sticky rather than derived: a custom size that happens to equal a preset would otherwise
    /// snap the dialog back to preset mode mid-edit and hide the field being typed into.
    custom_size: bool,
    notes: String,
    status: Option<String>,
    /// In-flight background export; receives the final status message when done.
    task: Option<std::sync::mpsc::Receiver<String>>,
    /// Deep single-view export whose reference orbit is building off-thread (before the render).
    prep: Option<crate::export::ExportPrep>,
    /// Progress in permille (0–1000) and a cooperative cancel flag.
    progress: std::sync::Arc<std::sync::atomic::AtomicU32>,
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// Last directory an export was saved to (persisted; defaults the Save dialog).
    last_dir: Option<std::path::PathBuf>,
    /// When the in-flight export began (deep exports: at the reference build). Drives the live
    /// "elapsed" readout and the final total-time report.
    started: Option<std::time::Instant>,
}

/// Relief-lighting + distance-estimate-glow effect settings (mostly persisted; `de_phase` is
/// runtime animation state). Grouped from the former flat `light_*` / `de_*` fields (Phase 2a).
struct EffectsConfig {
    /// Distance-estimate relief lighting (slope shading from the derivative).
    light: bool,
    light_angle: f32,  // radians
    light_height: f32, // relief strength (smaller = sharper)
    light_anim: bool,  // rotate the light direction over time
    /// Distance-estimate glow (contour bands near the boundary), + animation.
    de: bool,
    de_strength: f32,
    de_width: f32,
    de_anim: bool,
    de_phase: f32, // runtime (animated)
}

/// Auto-zoom autopilot state (transient except `dive_log2`, which is persisted). Grouped from the
/// former flat `autopilot*` fields (Phase 2a).
struct AutopilotState {
    /// Continuously diving toward the detail-richest region.
    active: bool,
    /// Screen-fraction pivot (0..1) currently zooming about; eased toward `goal` each frame.
    target: (f64, f64),
    /// Latest *evaluated* target (every `AUTOPILOT_EVAL_INTERVAL`); the goal `target` chases.
    goal: (f64, f64),
    /// App-time of the last target re-evaluation.
    eval_t: f64,
    /// True during the deep *stepped* dive (past the smooth regime) — see autopilot.rs / render.rs.
    stepping: bool,
    /// Dive limit as log2(magnification); persisted.
    dive_log2: f64,
}

/// Transient pointer / zoom / pan interaction state (not persisted): the in-progress zoom-box and
/// box-zoom gestures, pan reprojection, eased continuous-zoom, cursor tracking, and the per-view
/// settle-quality timers.
#[derive(Default)]
struct PointerState {
    /// In-progress Shift+drag zoom box (rubber-band → zoom to fill); `None` when not dragging.
    zoom_box: Option<ZoomBox>,
    /// Box-zoom (right-drag) start position in screen points; `None` when idle.
    box_start: Option<egui::Pos2>,
    /// Pan reprojection: accumulated device-pixel drag offset since the current pan began, and
    /// which view (0 = main/left, 1 = julia) is being panned. While a pan is pending the frozen
    /// iteration texture is translated by `pan_px` instead of re-rendered; cleared on settle.
    pan_px: egui::Vec2,
    pan_view: Option<u32>,
    /// Eased continuous-zoom velocity (log-rate per second; + = in, - = out, 0 = idle).
    zoom_vel: f64,
    /// Last cursor position over the canvas, for cursor-anchored continuous zoom.
    last_cursor: Option<egui::Pos2>,
    /// Complex coordinate under the cursor (for the status bar); `None` when off-canvas.
    pointer_complex: Option<(f64, f64)>,
    /// App-time of the last interaction, **per view** (`[0]` = main/Mandelbrot, `[1]` = Julia); each
    /// view stays at the coarse "moving" quality until `SETTLE_DELAY` after its own last change. Kept
    /// per-view so the dual view's Julia `c` cursor-drag doesn't force the Mandelbrot panel to
    /// re-render at low resolution.
    settle_t: [f64; 2],
    /// Progressive-AA settle stage, per view. On settle the anti-aliasing ramps 1×→2×→4×→… up to the
    /// chosen level over consecutive frames (each schedules the next); reset to 0 while interacting.
    settle_frame: [u32; 2],
    /// Per-view cold-start spinner debounce (see `draw_recompute_spinner`). `spin_since` = app-time
    /// the current contiguous placeholder-build streak began; `spin_last` = the last frame that view
    /// had such a build in flight (a >50 ms gap re-arms the streak). Together they impose a show-delay
    /// so a build too quick to notice never flashes the spinner. Not persisted; `[0.0; 2]` = idle.
    spin_since: [f64; 2],
    spin_last: [f64; 2],
}

/// How the fractal is *computed* (not colored): iteration budget + auto-scale, the perturbation
/// accelerators, continuous-zoom speed, the live work-budget, and anti-aliasing. All persisted —
/// round-trips 1:1 through `SessionState`. Grouped from the former flat compute fields (Phase 2a).
struct RenderConfig {
    /// Iteration budget ceiling.
    max_iter: u32,
    /// Auto-scale iteration count with zoom depth (else use `max_iter` as-is).
    auto_iter: bool,
    /// Series approximation (iteration-skipping) for deep Mandelbrot renders. Default on.
    series_approx: bool,
    /// Multi-reference glitch correction for exports (multi-pass, glitch-free). Default off.
    glitch_correct: bool,
    /// BLA (bilinear approximation): skips iterations throughout the orbit for deep floatexp
    /// Mandelbrot renders. Off by default while it's validated; enable to accelerate.
    use_bla: bool,
    /// Continuous-zoom speed multiplier (1.0 = default `ZOOM_RATE`).
    zoom_rate: f32,
    /// Magnification applied per click-to-zoom click (the `click_zoom` tool). 2–100×.
    click_zoom_factor: f32,
    /// Live-render work-budget multiplier (× `WORK_BUDGET`); persisted. Higher = crisper live deep
    /// zoom (fuller resolution) at lower FPS / less GPU-watchdog margin. Exports are unaffected.
    work_budget_scale: f64,
    /// Floor on the adaptive motion resolution (0.30–1.0): the lowest fraction of native resolution
    /// a moving/frozen deep-zoom frame may shrink to. Higher caps blockiness during a continuous
    /// zoom (sharper) at the cost of frame rate. Default 0.30 (matches the old fixed floor).
    min_motion_res: f32,
    /// Prefer detail over motion smoothness while zooming: motion frames REPROJECT the last fully
    /// detailed frame (geometrically tracked — the existing freeze path) instead of re-iterating
    /// at reduced quality; the view re-renders in full when motion pauses. KF-style stepping.
    /// Default off (today's coarse-but-live motion). Stage A of the "prefer detail" mode: the
    /// settle composite is still visible as it lands; present-gating (back-buffer swap) is the
    /// follow-up stage.
    prefer_detail: bool,
    /// Supersampling / anti-alias factor (1 = off, 2 = 2×2, 3 = 3×3).
    aa: u32,
    /// Play a sound when a render/export finishes (FRACTINT played a distinct tone; user
    /// request 2026-08-16). Fires on GUI export completion, tour-render completion, and CLI
    /// `--render` — never in harnesses, which don't reach those sites (and the uitest walk is
    /// explicitly excluded where it could).
    finish_sound: bool,
}

/// How a pixel is *colored* (not animated): the active palette (preset index / custom gradient /
/// duotone / binary), the cycle-density & offset sliders, the two-color lo/hi endpoints, and the
/// coloring method + its per-method params. `palette_editor_open` / `palette_rev` are the editor's
/// transient UI + cache-invalidation companions; the rest persist via `SessionState`. (Phase 2a.)
struct ColoringConfig {
    /// Selected palette index into `fractadyne_color::PRESETS`.
    palette_idx: usize,
    /// Color cycle density slider (0..1; mapped to a shader multiplier).
    cycle: f32,
    /// Palette offset slider (0..1).
    offset: f32,
    /// Custom gradient (editor): stops as `[pos, r, g, b]` (linear RGB). When `use_custom_palette`
    /// is set, this overrides the preset selection.
    custom_palette: Vec<[f32; 4]>,
    use_custom_palette: bool,
    /// Palette editor window open (transient UI).
    palette_editor_open: bool,
    /// Gradient editor's "Paste…" section: expanded, the pasted text, and the last result
    /// message. Session-transient by design — a half-typed import is not worth persisting.
    paste_open: bool,
    paste_text: String,
    paste_msg: Option<String>,
    /// Bumps on every gradient/duotone edit so caches (e.g. the minimap thumbnail) refresh (transient).
    palette_rev: u32,
    /// Two-color palette modes sharing the `lo`/`hi` colors (linear RGB), overriding preset/custom:
    /// **duotone** maps the value to a smooth `lo → hi → lo` ramp; **binary** paints a flat `hi`
    /// exterior with a flat `lo` interior (just in-set vs out-of-set).
    use_duotone: bool,
    use_binary: bool,
    duotone_lo: [f32; 3],
    duotone_hi: [f32; 3],
    /// Coloring method (smooth / stripe / triangle-ineq / orbit-trap / distance / decomposition).
    color_method: ColorMethod,
    stripe_freq: f32,
    trap_type: TrapType,
    /// Auto-normalize the palette cycle to the frame's escape-value range on export (`--normalize`).
    /// At extreme depth the smooth-iter counts are ~1e5–1e6 and a fixed `cycle` aliases a correct
    /// escape field into speckle; normalize maps the range to `cycle`-many palette sweeps. Export
    /// path only (transient; not persisted). See `render_export_normalized`.
    normalize: bool,
    /// LIVE auto-normalization: when a settled deep frame's escaped smooth-iter RANGE (read back
    /// from the GPU escape-range counters) is huge, remap the palette so the range spans
    /// `0.5 + cycle·6` sweeps — the live analogue of `--normalize`, killing the "noise pools"
    /// a fixed cycle makes of dense 1e5-scale escape fields. Only active for the Smooth method
    /// and only past a range threshold, so ordinary views keep classic coloring. Persisted.
    normalize_live: bool,
    /// Log-scaled palette mapping. Applies wherever normalization is active (live or
    /// `--normalize`): escape values crowd towards the high end at depth, so a linear map spends
    /// most of the palette on a thin shell. Persisted.
    log_palette: bool,
}

/// Time-varying visual overlays advanced per-frame: the orbit racing-dot (enable / normalize /
/// speed + live phase & hue), `show_orbits` and the tour-scripted `tour_orbit` that feed the same
/// overlay, and palette cycling (mode / speed / ping-pong dir) + the `RandomPalette` morphing
/// generator. Distinct from the static [`ColoringConfig`]. (Phase 2a.)
struct AnimationState {
    /// Draw the iteration orbit of the point under the cursor.
    show_orbits: bool,
    /// A tour-scripted orbit point (complex) to draw when there's no cursor (during playback);
    /// `None` = use the cursor. Set each frame by `advance_playback` (transient).
    tour_orbit: Option<(f64, f64)>,
    /// Fit the orbit into a fixed inset (good view at any zoom) instead of overlaying it on the
    /// fractal through the viewport.
    orbit_normalize: bool,
    /// Animate a dot racing out along the orbit, with a cycling color.
    orbit_anim: bool,
    /// Racing-dot speed (iterates per second along the path).
    orbit_anim_speed: f32,
    /// Position along the orbit path (segment units) and the dot's hue (0..1) — transient.
    orbit_phase: f32,
    orbit_hue: f32,
    /// Palette animation mode + speed (offset cycles/sec), and the ping-pong direction.
    palette_anim: PaletteAnim,
    palette_anim_speed: f32,
    anim_dir: f32,
    /// State for the randomized morphing-gradient palette mode (transient, seeded generator).
    random_palette: RandomPalette,
}

/// Scattered window/panel open-flags plus two small chrome selections — pure UI-visibility state the
/// egui layer reads. Only `right_panel_open` & `minimap` persist; the rest are transient. (Phase 2a.)
/// Where the issue reporter sends to (Help → Report an issue…). The project's own support mailbox
/// as of 2026-08-15, confirmed live before this was changed — a dead support address is worse than
/// none, which is why the TODO entry for it waited on the mailbox actually existing.
pub(crate) const REPORT_EMAIL: &str = "feedback@fractadyne.org";

/// GitHub issue tracker — the PRIMARY reporting channel (public, searchable, and other users can
/// confirm/subscribe); email stays as the private fallback.
pub(crate) const ISSUES_URL: &str = "https://github.com/WindySnowOwl/fractadyne/issues";

/// Minimal percent-encoding for a `mailto:` subject/body component (keeps RFC 3986 unreserved).
pub(crate) fn mailto_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// What kind of issue is being reported (drives the report header + email subject).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum IssueKind {
    Crash,
    Rendering,
    Performance,
    Ui,
    Feature,
    Other,
}
impl IssueKind {
    pub(crate) const ALL: [IssueKind; 6] = [
        IssueKind::Crash,
        IssueKind::Rendering,
        IssueKind::Performance,
        IssueKind::Ui,
        IssueKind::Feature,
        IssueKind::Other,
    ];
    pub(crate) fn label(self) -> &'static str {
        match self {
            IssueKind::Crash => "Application crash / freeze",
            IssueKind::Rendering => "Incorrect rendering",
            IssueKind::Performance => "Performance issue",
            IssueKind::Ui => "UI / usability issue",
            IssueKind::Feature => "Feature request",
            IssueKind::Other => "Other",
        }
    }
}

/// Optional severity for triage (`Unspecified` = omitted from the report).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Severity {
    Unspecified,
    Low,
    Medium,
    High,
    Blocking,
}
impl Severity {
    pub(crate) const ALL: [Severity; 5] = [
        Severity::Unspecified,
        Severity::Low,
        Severity::Medium,
        Severity::High,
        Severity::Blocking,
    ];
    pub(crate) fn label(self) -> &'static str {
        match self {
            Severity::Unspecified => "—",
            Severity::Low => "Low",
            Severity::Medium => "Medium",
            Severity::High => "High",
            Severity::Blocking => "Blocking",
        }
    }
}

/// How reproducible the issue is (`Unspecified` = omitted from the report).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Repro {
    Unspecified,
    Always,
    Sometimes,
    Once,
    Cannot,
}
impl Repro {
    pub(crate) const ALL: [Repro; 5] =
        [Repro::Unspecified, Repro::Always, Repro::Sometimes, Repro::Once, Repro::Cannot];
    pub(crate) fn label(self) -> &'static str {
        match self {
            Repro::Unspecified => "—",
            Repro::Always => "Always",
            Repro::Sometimes => "Sometimes",
            Repro::Once => "Happened once",
            Repro::Cannot => "Can't reproduce",
        }
    }
}

/// State for the "Report an issue" dialog: the classification, description, which artifacts to
/// include, and the last status line. The report is assembled on demand from these + live
/// diagnostics ([`FractadyneApp::build_report`]); nothing is sent until the user acts.
struct ReportState {
    open: bool,
    kind: IssueKind,
    severity: Severity,
    repro: Repro,
    description: String,
    include_sysinfo: bool,
    include_location: bool,
    include_log: bool,
    include_crash: bool,
    /// Include the most recent Diagnostics test result. Off unless a test has actually been run
    /// and the user chose to attach it — an issue that claims a test result it doesn't have is
    /// worse than one that claims nothing.
    include_test: bool,
    msg: Option<String>,
}
impl Default for ReportState {
    fn default() -> Self {
        // Include everything by default (crash auto-drops when there's none); system info is opt-out.
        Self {
            open: false,
            kind: IssueKind::Other,
            severity: Severity::Unspecified,
            repro: Repro::Unspecified,
            description: String::new(),
            include_sysinfo: true,
            include_location: true,
            include_log: true,
            include_crash: true,
            include_test: false,
            msg: None,
        }
    }
}

/// (`palette_editor_open` stays in [`ColoringConfig`]; the minimap *cache* — `minimap_tex` /
/// `minimap_key` — stays flat; only the minimap enable *toggle* joins here.)
#[derive(Default)]
struct DialogState {
    /// Benchmark *results* window open.
    bench_open: bool,
    /// Benchmark *config* dialog (mode / resolution / burn-in) open.
    bench_dialog_open: bool,
    /// Bookmarks browser open.
    bookmarks_open: bool,
    /// "Reset application state" confirmation dialog open.
    reset_confirm_open: bool,
    /// First-run welcome overlay open (shown once on a fresh install; re-openable from Help).
    welcome_open: bool,
    /// Post-crash "send a report?" prompt. Opened at startup when the previous session ended
    /// unclean and the user has not opted out.
    crash_prompt_open: bool,
    /// "Accelerated build" dialog open (Help menu).
    accelerated_open: bool,
    /// Keyboard/help overlay window open.
    help_open: bool,
    /// Selected Help section index.
    help_section: usize,
    /// Whether the right-hand control panel is shown (persisted).
    right_panel_open: bool,
    /// Minimap overview enabled (persisted).
    minimap: bool,
    /// "Script to current view" export dialog open, plus its inputs (a notation caption and the
    /// dive duration in seconds — defaulted from the zoom depth when the dialog opens).
    script_export_open: bool,
    script_export_note: String,
    script_export_secs: f64,
    /// A generic titled message dialog: `Some((title, body))` while shown. Used for one-off
    /// notices (e.g. a script that fails to load) that deserve a dialog rather than a fleeting
    /// toast, without borrowing an unrelated window's title.
    notice: Option<(String, String)>,
}

struct FractadyneApp {
    viewport: Viewport,
    /// Which fractal is being rendered (single-view mode).
    fractal: FractalKind,
    /// Julia constant `c` (complex). In dual view it's driven by the Mandelbrot cursor.
    julia_c: (f64, f64),
    /// Single-view Julia mode: show the Julia set of the current formula for `julia_c`.
    julia_mode: bool,
    /// Click-to-zoom tool (single view): when on, a left-click dives in by
    /// `render_cfg.click_zoom_factor` recentered on the point, right-click backs out. Off by
    /// default; drag still pans and Shift/right-drag still box-zoom (see `click_zoom_at`).
    click_zoom: bool,
    /// Dual view: if `Some`, the Julia `c` is pinned to this Mandelbrot point (a marker
    /// is drawn there) instead of following the cursor. Click to pin, click it to release.
    julia_pin: Option<(f64, f64)>,
    /// Dual linked view: Mandelbrot (left) ↔ Julia (right).
    dual: bool,
    /// Dual-view split position (fraction of width) — the draggable separator between panels.
    dual_split: f32,
    /// Borderless fullscreen state (toolbar toggle).
    fullscreen: bool,
    /// `Ui::id` of the menu-bar row (captured each frame in `draw_menu_bar`) — the key egui stores
    /// the bar's open-menu state under. Lets `close_menu_bar` dismiss a still-open menu when the
    /// user starts navigating the fractal view (wheel-zoom doesn't count as a "click elsewhere",
    /// so egui would otherwise leave the dropdown hanging over the canvas).
    menu_bar_id: Option<egui::Id>,
    /// Viewport for the Julia panel in dual view.
    julia_viewport: Viewport,
    /// Pointer / zoom / pan interaction state (zoom box, pan reprojection, eased zoom, settle timers).
    pointer: PointerState,
    /// Active smooth "zoom out to home" animation (Home button); `None` when idle.
    home_anim: Option<HomeAnim>,
    /// Auto-zoom autopilot state.
    autopilot: AutopilotState,
    /// Per-frame visual animation: orbit racing-dot, tour-scripted orbit point, palette cycling,
    /// and the randomized morphing-gradient generator.
    anim: AnimationState,
    /// Cache for the interactive orbit overlay (avoids recomputing the bignum orbit
    /// every frame when the cursor/view haven't moved).
    orbit_cache: std::cell::RefCell<Option<OrbitCacheEntry>>,
    /// Active scripted camera tour / benchmark (None when idle).
    playback: Option<Playback>,
    /// The viewer's own iteration budget + coloring, saved while a tour overrides them and
    /// restored when it ends (a script's settings are the script's, not the session's).
    playback_restore: Option<crate::scripting::PlaybackRestore>,
    /// Path of the last tour script played (persisted): the toolbar ▶ button and the menu default
    /// to it, so replaying the same tour — the common case — is one click, not a file dialog.
    last_script: Option<std::path::PathBuf>,
    /// Last directory any open/save dialog landed in — the shared fallback so the next dialog
    /// opens where the user last browsed (persisted). Category memories (`export.last_dir`, the
    /// script's own parent) take precedence when set; this catches everything else.
    last_dir: Option<std::path::PathBuf>,
    /// "Render script…" dialog + the child process doing the work (see `TourRenderUi`).
    tour_render: TourRenderUi,
    /// Help → Diagnostics…: run the user-safe tests from the UI (see `ui::diagnostics`).
    diagnostics: crate::ui::diagnostics::DiagnosticsUi,
    /// Last benchmark report text + whether its window is open.
    bench_report: Option<String>,
    bench_cfg: BenchConfig,
    /// In-flight standardized benchmark, advanced one pass per frame from `update()`.
    std_bench: Option<StdBench>,
    /// CLI `--benchmark-std [--res RES] [--burnin N]`: run headless, save, quit.
    auto_stdbench: bool,
    auto_stdbench_done: bool,
    std_res: BenchRes,
    std_passes: u32,
    std_depth: BenchDepth,
    /// GPU backend ("Dx12"/"Vulkan"/"Gl") — names the shader-compiler stack for `--gputest`.
    gpu_backend: String,
    /// GPU adapter name (for benchmark reports).
    gpu_name: String,
    /// Host system facts (CPU / cores / cache / VRAM) for benchmark reports.
    sysinfo: SysInfo,
    /// CLI auto-benchmark: run on startup, save to this path, then quit.
    auto_benchmark: bool,
    auto_benchmark_out: Option<std::path::PathBuf>,
    auto_benchmark_done: bool,
    /// CLI render-and-exit: render one image to `auto_render_out`, then quit.
    auto_render: bool,
    auto_render_out: Option<std::path::PathBuf>,
    auto_render_done: bool,
    /// `--render-iter`: write the raw iteration texture as EXR instead of a colored image.
    render_iter_mode: bool,
    /// CLI `--render-tour FILE`: render a keyframe tour to a PNG frame sequence, then quit.
    render_tour: Option<std::path::PathBuf>,
    render_tour_done: bool,
    /// The tour-render flags as GIVEN — each `None` means "the script's `[render]` block decides"
    /// (and a built-in default if it's silent too). Resolving here would erase the difference
    /// between a flag the user passed and a default we made up.
    tour_cli: crate::scripting::TourRenderConfig,
    /// CLI `--selftest`: run the GPU validation suite, print a report, and exit.
    selftest: bool,
    selftest_done: bool,
    /// `--selftest-filter <substr>`, `--selftest-list`, `--bless` — parsed here from the
    /// EXPANDED args so they honor `@response-file` / `--args-file` expansion. `run_selftest`
    /// must read these, NOT `std::env::args()` (raw args bypass the expansion `main()` did).
    selftest_filter: Option<String>,
    selftest_list: bool,
    selftest_bless: bool,
    /// CLI `--profile`: run the profiling harness (benchmark regions), log to `logs/`, exit.
    profile: bool,
    profile_done: bool,
    profile_reps: u32,
    profile_regions: Option<String>,
    profile_out: Option<std::path::PathBuf>,
    /// CLI `--bench-matrix [--bless] [--reps N]`: run the path-coverage perf + regression suite,
    /// compare against (or bless) the baseline, and exit.
    bench_matrix: bool,
    bench_matrix_done: bool,
    /// CLI `--reusetest`: measure reuse-first-zoom reprojection staleness vs Δ-octaves, exit.
    reusetest: bool,
    reusetest_done: bool,
    /// CLI `--resizetest`: headless window-resize aspect-invariant regression harness, exit.
    resizetest: bool,
    /// CLI `--divetest FILE`: headless live-dive performance harness (real-time tour windows at
    /// increasing depths through the ACTUAL playback machinery), report + JSON, exit.
    divetest: Option<std::path::PathBuf>,
    /// CLI `--livetest FILE`: headless live-OUTPUT harness — plays a tour through the live
    /// pipeline and validates the frames it shows against offline renders of the same views.
    livetest: Option<std::path::PathBuf>,
    /// CLI `--uitest [DIR]`: scripted walk through every UI screen + the live-render bands,
    /// screenshotting each and writing a review bundle (see `mod uitest`), then exit.
    uitest: Option<uitest::UiTest>,
    /// CLI `--juliadive [DIR]`: dev harness — dual view, continuous in-app Julia zoom to ~1400×
    /// with periodic screenshots (see `uitest::JuliaDive`). Reproduces the dual-view Julia motion
    /// path deterministically (synthetic OS input proved unreliable for wheel/focus).
    juliadive: Option<uitest::JuliaDive>,
    /// CLI `--autodive [LOG10]`: unpaced autopilot dive that hammers the frame-cost controller and
    /// reports whether the lethal regime was reached. See `autopilot::AutoDive`.
    autodive: Option<autopilot::AutoDive>,
    /// CLI `--chunk-sweep [ITERS]`: measure the UNCENSORED per-window wall cost of a chunked
    /// iterate at the current view. The one measurement that separates the two branches of the
    /// 2026-08-22 device loss — see `mod chunksweep`.
    chunk_sweep: Option<chunksweep::ChunkSweep>,
    /// CLI `--motiontest`: the motion-presentation gate (design/mode2-chunking.md §11) — the
    /// in-loop harness that can see what `--livetest`'s settled checkpoints cannot: what a
    /// chunked view ADOPTS as its frozen texture while the camera is moving.
    motiontest: Option<motiontest::MotionTest>,
    /// CLI `--play FILE`: start the GUI with this tour already playing in the LIVE view. The only
    /// way to exercise the on-screen playback path (present, watchdog budget, tiled settle) from a
    /// command line — a headless harness cannot reach it.
    play_tour: Option<std::path::PathBuf>,
    play_tour_done: bool,
    /// `--quick` with `--livetest`: skip the offline oracle (context/black metrics only).
    livetest_quick: bool,
    /// CLI `--frametest`: run the frame-timing / stutter harness (deep-zoom dive), log, exit.
    frametest: bool,
    /// `--frametest --center X Y` (full-precision decimals; default seahorse).
    frametest_center: Option<(String, String)>,
    frametest_steps: u32,
    frametest_hold: u32,
    frametest_dive: f64,
    /// True only while `draw_central` builds the LIVE view's params: `build_params` may start a
    /// tiled settle only then. The other callers (profiling and benchmark harnesses) time
    /// single-dispatch renders, and a silently tiled frame would corrupt their numbers.
    allow_tiled_settle: bool,
    /// Per-render setup timings (reference / series-skip), recorded by
    /// `current_export_request_for` via a `Cell` and read by the profiler.
    prof: std::cell::Cell<profile::ProfSetup>,
    /// Frame-rate cap (FPS); `None` = uncapped.
    fps_cap: Option<f64>,
    /// Export dialog + in-flight background export state.
    export: ExportState,
    /// Gallery browser state.
    gallery: GalleryState,
    /// Bookmarks (saved views), persisted to the config dir; + window/input state.
    bookmarks: Vec<Bookmark>,
    bookmark_name: String,
    /// A just-added bookmark whose thumbnail still needs rendering (deferred to `update`,
    /// where the GPU is available and the current view still matches the bookmark).
    pending_thumb: Option<usize>,
    /// A bookmark-thumbnail SCREENSHOT is in flight (the reply lands next frame as
    /// `egui::Event::Screenshot`); holds the bookmark index it belongs to.
    thumb_shot: Option<usize>,
    /// The central fractal panel's rect in PHYSICAL pixels ([x, y, w, h]), stored each frame by
    /// the central draw — what the bookmark thumbnail crops out of the window screenshot.
    central_rect_px: [u32; 4],
    /// Decoded bookmark-thumbnail textures, keyed by thumb id (lazy-loaded for the dialog).
    thumb_cache: std::collections::HashMap<String, egui::TextureHandle>,
    /// Navigation history (location undo/redo) + settle-edge tracking.
    nav: NavHistory,
    /// "Go to location" dialog state.
    goto: GotoDialog,
    /// Share-location (`.fdn`) dialog: open flag, editable location text, and an error line.
    share: ShareDialog,
    /// Transient status toast (message, time set) — e.g. minibrot-finder result.
    toast: Option<(String, f64)>,
    /// A toast queued from a context without an `egui::Context` (e.g. a bookmark auto-save
    /// failure in `save_bookmarks`); drained into `toast` early in `update()`.
    pending_toast: Option<String>,
    /// Window/panel open-flags + the right-panel & minimap toggles (see [`DialogState`]).
    dialogs: DialogState,
    /// "Report an issue" dialog state (Help → Report an issue…).
    report: ReportState,
    /// Set after a reset: stop autosaving so we don't recreate the just-deleted state file.
    suppress_autosave: bool,
    /// A one-shot warning to show as a toast on the first frame (e.g. a newer-version session).
    pending_state_warning: Option<String>,
    /// Compute knobs (iteration budget, perturbation accelerators, zoom speed, work-budget, AA).
    render_cfg: RenderConfig,
    /// Draw the discreet "Fd" brand mark in the lower-right of the live view and exports. On by
    /// default; toggleable.
    watermark: bool,
    /// Burn a zoom/coordinate HUD (top-left) into rendered frames. CLI `--show-location`, or a tour
    /// script's `show_location`. Off by default; only applied on the ctx-bearing render paths
    /// (`--render`, `--render-tour`).
    show_location: bool,
    /// Pre-rasterized "Fd" mark for stamping into exports (built lazily from the font atlas on the
    /// main thread; the export worker has no egui context). `None` until first built.
    watermark_overlay: Option<export::WmOverlay>,
    /// UI scale (egui zoom factor): scales the interface fonts + widgets. 1.0 = default.
    ui_scale: f32,
    /// Active UI theme (dark / light); persisted. Applied via `theme::apply_theme`.
    theme: ThemeMode,
    /// Update-check track (Stable / Beta) + whether to check on launch; both persisted.
    update_track: update::UpdateTrack,
    update_check_on_launch: bool,
    /// `max_color_attachment_bytes_per_sample`: (granted, adapter-available).
    ///
    /// ⚠These differ and the difference matters. We REQUEST `min(adapter, 48)` — 48 = the three
    /// Rgba32Float chunk-state targets the iteration-range path writes today — so `device.limits()`
    /// reports 48 by construction and says nothing about the hardware. A fourth attachment (mode-2
    /// chunking) needs 64, and only the ADAPTER limit can answer whether that is available.
    attach_bytes_per_sample: (u32, u32),
    /// Draw the "Fd" brand mark (live view + exports). Persisted; opt-out offered on first run.
    show_watermark: bool,
    /// "Don't ask again" for the post-crash report prompt. Persisted.
    crash_prompt_disabled: bool,
    /// In-flight update check (worker → UI) and its last result, and whether the launch check has
    /// fired this session. Not persisted.
    update_rx: Option<std::sync::mpsc::Receiver<update::UpdateStatus>>,
    update_status: Option<update::UpdateStatus>,
    update_launch_checked: bool,
    /// Whether the in-flight check was user-initiated (toast every outcome) vs the silent launch
    /// check (toast only when an update is found).
    update_manual: bool,
    /// Whether the "Update available" prompt (with the GitHub download link) is showing. Opened
    /// when a check finds a newer build; dismissable. Not persisted.
    update_prompt_open: bool,
    /// Minimap overview cache: home-view thumbnail + the key (formula, palette, method) it was
    /// rendered for (re-render on change). The enable *toggle* lives in [`DialogState`].
    minimap_tex: Option<egui::TextureHandle>,
    minimap_key: Option<(u32, usize, u32, u32)>,
    /// Static color mapping: palette selection, custom/duotone/binary, cycle/offset, method + params.
    coloring: ColoringConfig,
    /// Performance/diagnostic tracking + overlay.
    perf: Perf,
    /// Relief lighting + distance-estimate glow effect settings.
    effects: EffectsConfig,
    /// Per-view perturbation reference cache (index 0 = main/left, 1 = dual Julia).
    /// Separate caches let both dual panels use perturbation without thrashing.
    ref_cache: [RefCache; 2],
    /// `orbit_id` of view 0's reference last written to `last_reference.bin` (persistence de-dup, so
    /// autosave doesn't re-serialize an unchanged reference). `None` until a reference is saved.
    last_saved_ref_id: Option<u64>,
    /// `(orbit_id, first-seen time)` of a full reference awaiting its debounced persist — see
    /// `autosave`. Reset whenever the reference changes, so we write once it's been stable ~1 s.
    ref_save_pending: Option<(u64, f64)>,
    /// In-flight off-thread reference recompute per view (`None` = idle). Keeps the deep-zoom
    /// bignum recompute off the render thread: the frame keeps using the cached reference until the
    /// worker's result arrives (see `build_params`).
    recompute_rx: [Option<std::sync::mpsc::Receiver<crate::render::RecomputeResult>>; 2],
    /// Script-playback reference LOOKAHEAD queue (view 0): a tour knows its future camera path, so
    /// while the current reference serves the view, workers build the ones the dive is ABOUT to
    /// need (slots spaced ~2 octaves apart along the script). Finished results are held until the
    /// dive reaches each one's depth-validity window, then installed directly (see
    /// `playback_ref_prefetch`). Purely additive — a missed window is dropped and the reactive
    /// rebuild path covers it. Cleared on tour start/end and `invalidate_refs`.
    ref_prefetch: Vec<crate::render::RefPrefetchSlot>,
    /// Dedicated builds (≤2: one ready + one building) for upcoming HOLD keyframes' references
    /// at each hold's own explicit ask and destination precision, started DURING the glide. The
    /// ordinary lookahead deliberately builds with the short motion cap (`LIVE_REF_CAP`), so a
    /// deep hold's 8–90 s reference extension could not even START until the camera arrived and
    /// `interacting` dropped — the hold then spent its whole window clamped at the previous ask
    /// (livetest e82: 20.6 s stale; e94: 100% capped-black at the stale 2M orbit). See
    /// `playback_hold_prefetch`.
    hold_prefetch: Vec<crate::render::HoldPrefetch>,
    /// Last snapshot used for change detection (debounced auto-save).
    last_state: fractadyne_state::SessionState,
    /// App-time (s) of the last change while unsaved; `None` when clean.
    dirty_since: Option<f64>,
}

impl FractadyneApp {
    fn new(cc: &eframe::CreationContext<'_>, args: &[String]) -> Self {
        // A missing wgpu render state means the GPU backend failed to initialize. Report it
        // plainly and exit cleanly rather than surfacing a Rust panic + backtrace to the user.
        let render_state = match cc.wgpu_render_state.as_ref() {
            Some(rs) => rs,
            None => {
                eprintln!(
                    "Fractadyne requires the wgpu GPU backend (eframe Renderer::Wgpu), which failed \
                     to initialize. Check that your GPU drivers support Vulkan/DX12/Metal."
                );
                crate::exit(1);
            }
        };
        install_renderer(render_state);
        // Record the GPU's storage-buffer binding limit as the reference-orbit length ceiling, so
        // the off-thread live recompute can bound a deep-interior orbit to what the buffer holds
        // (a `.fdn` with auto-iter off + a very high max_iter at an interior view would otherwise
        // build an oversized orbit+BLA and panic in the live bind — the export path already guards
        // this with `OrbitTooLarge`). One-time; the limit is a device constant.
        render::init_orbit_len_cap(render_state.device.limits().max_storage_buffer_binding_size);
        // D1.3: route GPU faults through the diag log. Uncaptured errors keep wgpu's
        // fail-fast semantics (log first, then panic so the hook writes a crash report) —
        // EXCEPT device loss, which gets a graceful auto-restart instead of a hard crash.
        //
        // A lost device (Windows TDR — our own oversized dispatch, a driver reset, sleep/resume,
        // another app wedging the GPU) is unrecoverable in-place under eframe, but it is NOT an
        // app bug the user should experience as a crash: the session file already persists the
        // exact view, so a fresh process resumes almost seamlessly. Both crash reports on this
        // machine (2026-08-02 shallow view, 2026-08-06 deep spar) are this exact class. The
        // panic hook still writes the crash report first (durable artifact), then we relaunch —
        // bounded by the generation guard in `relaunch_after_device_loss` so a loss that recurs
        // cannot restart-loop forever.
        // ONE place decides whether a lost device may relaunch, because the two paths below drifted
        // apart in exactly the way that matters: both used `elapsed_s() > 60`, and that conflates
        // "early in this process's life" with "restart loop". FIELD CASE (2026-08-18, build 1675):
        // a deep view + Home glide lost the device at 50.4 s of uptime -- an ordinary user action,
        // not a boot problem -- and the guard suppressed the relaunch, so a loss the app is designed
        // to recover from was experienced as a hard crash.
        //
        // The honest guard counts GENERATIONS instead of reading the clock. The child already
        // carries FRACTADYNE_RESTARTED_AFTER_GPU_LOSS, so it becomes a counter: a genuine
        // relaunch loop still terminates (bounded at 3), while a first loss at any uptime recovers.
        // The clock keeps one narrow job it is actually good for -- if a RESTARTED generation dies
        // again within 15 s, restarting did not help and spinning is pointless.
        fn relaunch_after_device_loss() {
            let gen: u32 = std::env::var("FRACTADYNE_RESTARTED_AFTER_GPU_LOSS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            let argv: Vec<String> = std::env::args().skip(1).collect();
            if crate::is_task_invocation(&argv) {
                // Fail visibly instead. A harness or offline job that respawns itself hides the
                // failure it exists to surface, and can leave a window nobody is supervising.
                diag::log_line(
                    "wgpu",
                    "device lost during a task invocation — not relaunching (harnesses must fail loudly)",
                );
                return;
            }
            let Some(next) = crate::relaunch_decision(gen, diag::elapsed_s()) else {
                diag::log_line(
                    "wgpu",
                    &format!(
                        "device lost {:.1}s into generation {gen} — not restarting (relaunch policy)",
                        diag::elapsed_s()
                    ),
                );
                return;
            };
            if let Ok(exe) = std::env::current_exe() {
                diag::log_line("wgpu", &format!("device lost — restarting (generation {next})"));
                let _ = std::process::Command::new(exe)
                    .args(std::env::args().skip(1))
                    .env("FRACTADYNE_RESTARTED_AFTER_GPU_LOSS", next.to_string())
                    .spawn();
            }
        }
        render_state.device.on_uncaptured_error(Box::new(|e| {
            diag::log_line("wgpu", &format!("uncaptured error: {e}"));
            let msg = e.to_string();
            if msg.contains("device is lost") || msg.contains("Device is lost") || msg.contains("DeviceLost") {
                diag::write_crash_report(&format!("wgpu device lost: {msg}"));
                relaunch_after_device_loss();
                crate::exit(2);
            }
            panic!("wgpu uncaptured error: {e}");
        }));
        // The device-lost CALLBACK must also restart — not just the uncaptured-error path above.
        // Measured (e21000 experiment, 2026-08-07): after a TDR the main thread can BLOCK inside
        // a wgpu wait on the dead device, so no error ever surfaces, no panic fires, and the app
        // hangs forever with the watchdog barking — the historical "present wedge", finally
        // explained. This callback runs on another thread and is then the ONLY recovery path:
        // write the crash report and relaunch (same generation guard as above). `Destroyed` is the
        // clean-teardown reason — never restart on it.
        render_state.device.set_device_lost_callback(|reason, msg| {
            diag::log_line("wgpu", &format!("DEVICE LOST ({reason:?}): {msg}"));
            if !matches!(reason, eframe::wgpu::DeviceLostReason::Destroyed) {
                diag::write_crash_report(&format!("wgpu device lost ({reason:?}): {msg}"));
                relaunch_after_device_loss();
                crate::exit(2);
            }
        });
        // D1.4: hang watchdog for every update()-driven mode. update() stamps liveness each
        // frame; long blocking phases stamp via breadcrumbs and progress pumps, so a stale
        // stamp really does mean "the app went silent" — the log then names the phase.
        diag::start_watchdog();
        install_fonts(&cc.egui_ctx); // brand typefaces (theme-independent); visuals applied below
        let gpu_name = render_state.adapter.get_info().name;
        // Which BACKEND (and so which shader-compiler stack: DX12 = FXC/DXC-compiled HLSL,
        // Vulkan = SPIR-V) built the running pipelines — a `--gputest` verdict or wrong-render
        // report is uninterpretable without it. Kept separate from `gpu_name`, which baseline
        // `same_gpu` comparisons match on verbatim.
        let gpu_backend = format!("{:?}", render_state.adapter.get_info().backend);

        // CLI modes (headless, for automation / debugging):
        //   --benchmark [--out PATH]                    run the built-in benchmark, save, quit
        //   --render --out IMG [view options]           render one image, save, quit
        // Render view options: --fractal NAME, --center X Y, --zoom MAG, --size W,
        //   --ss N, --iter N, --julia, --julia-c RE IM, --palette IDX.
        // `args` already has any `@response-file` / `--args-file` expanded (see `main`).
        let out_path = args
            .iter()
            .position(|a| a == "--out" || a == "-o")
            .and_then(|i| args.get(i + 1))
            .map(std::path::PathBuf::from);
        let auto_benchmark = args.iter().any(|a| a == "--benchmark" || a == "--bench");
        // Standardized (pinned-settings) benchmark: --benchmark-std [--res RES] [--burnin N].
        // --burnin/--res on their own also imply it.
        let render_iter_mode = args.iter().any(|a| a == "--render-iter");
        let auto_render = args.iter().any(|a| a == "--render") || render_iter_mode;
        let selftest = args.iter().any(|a| a == "--selftest" || a == "--selftest-list");
        let profile = args.iter().any(|a| a == "--profile");
        let bench_matrix = args.iter().any(|a| a == "--bench-matrix");
        let reusetest = args.iter().any(|a| a == "--reusetest");
        let resizetest = args.iter().any(|a| a == "--resizetest");
        let frametest = args.iter().any(|a| a == "--frametest");
        let val = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1));
        // Selftest sub-flags from the EXPANDED args (so @response-file works — see field docs).
        let selftest_filter = val("--selftest-filter").map(|s| s.to_ascii_lowercase());
        let selftest_list = args.iter().any(|a| a == "--selftest-list");
        let selftest_bless = args.iter().any(|a| a == "--bless");
        // Standardized benchmark: --benchmark-std, or implied by --res / --burnin.
        let std_res = val("--res").and_then(|s| BenchRes::from_token(s)).unwrap_or(BenchRes::P1080);
        // --burnin may carry a pass count (`--burnin 20`) or stand alone (defaults to 10).
        let burnin_flag = args.iter().any(|a| a == "--burnin");
        let std_passes = if burnin_flag {
            val("--burnin").and_then(|s| s.parse::<u32>().ok()).unwrap_or(10).clamp(1, 500)
        } else {
            1
        };
        // Dive depth: --depth <standard|ultra|all> (or the shorthand --ultra). Also implies a
        // standardized run.
        let ultra_flag = args.iter().any(|a| a == "--ultra");
        let std_depth = val("--depth")
            .and_then(|s| BenchDepth::from_token(s))
            .unwrap_or(if ultra_flag { BenchDepth::Ultra } else { BenchDepth::Standard });
        let auto_stdbench = args.iter().any(|a| a == "--benchmark-std")
            || burnin_flag
            || ultra_flag
            || args.iter().any(|a| a == "--res" || a == "--depth");
        // --render-tour FILE [--fps N] [--size W] [--height H] [--ss N] [--out DIR] [--mp4 [PATH]]
        let render_tour = val("--render-tour").map(std::path::PathBuf::from);
        // Each flag stays OPTIONAL here: unset means the script's [render] block decides
        // (see TourRenderConfig::resolve), so a tour renders as authored with no flags at all.
        // --size accepts a bare width (`1920`) or `WIDTHxHEIGHT` (`5120x2160`); explicit --height
        // overrides the height from --size.
        // Unreadable is FATAL, absent is not — the same rule `--order` below already states.
        // A tour is the long job: silently rendering four hours of frames at the script's size
        // because the `--size` you passed was mistyped is the costliest version of this defect.
        let (size_w, size_h) = val("--size").map(|s| arg_size("--size", s)).unwrap_or((None, None));
        let tour_cli = scripting::TourRenderConfig {
            fps: val("--fps").map(|s| {
                let f = arg_parse::<f64>("--fps", s, "a number");
                if f > 0.0 {
                    f
                } else {
                    eprintln!("fractadyne: --fps must be greater than 0 (got \"{s}\").");
                    crate::exit(2)
                }
            }),
            width: size_w,
            height: val("--height")
                .map(|s| arg_parse::<u32>("--height", s, "a whole number"))
                .or(size_h),
            ss: val("--ss").map(|s| arg_parse::<u32>("--ss", s, "a whole number")),
            out: out_path.clone(),
            prefix: val("--prefix").cloned(),
            // --mp4 [PATH]: presence enables ffmpeg encoding after the render. A following
            // non-flag token is the output path; otherwise `<out-dir>/<prefix>.mp4`.
            mp4: args.iter().position(|a| a == "--mp4").map(|i| {
                args.get(i + 1).filter(|s| !s.starts_with('-')).map(std::path::PathBuf::from)
            }),
            // --segment NAME: render only that chapter, keeping the global frame numbering.
            segment: val("--segment").cloned(),
            // --segments N --segment-index K: shard the timeline for multi-machine rendering.
            // Sharding is the case where a silent default corrupts a DISTRIBUTED job: an
            // unreadable --segments used to mean "no sharding", so one node quietly rendered the
            // WHOLE timeline, and an unreadable --segment-index meant shard 0, so two nodes
            // rendered the same frames and the rest were never rendered at all.
            segments: val("--segments").map(|s| arg_parse::<u32>("--segments", s, "a whole number")),
            segment_index: val("--segment-index")
                .map(|s| arg_parse::<u32>("--segment-index", s, "a whole number")),
            // --dry-run: print the resolved frame plan and exit without rendering.
            dry_run: args.iter().any(|a| a == "--dry-run"),
            // --order sequential|progressive: which ORDER frames render in (indices unchanged).
            // A typo must not silently fall back to sequential — a farm script would then lose
            // the preview it asked for without a trace.
            progressive: match val("--order").map(|s| s.to_ascii_lowercase()).as_deref() {
                None | Some("sequential") => false,
                Some("progressive") => true,
                Some(other) => {
                    eprintln!(
                        "--order must be `sequential` or `progressive` (got \"{other}\")"
                    );
                    crate::exit(2);
                }
            },
            // --overwrite / -y: replace existing frames without prompting.
            overwrite: args.iter().any(|a| a == "--overwrite" || a == "-y"),
            // --resume: keep already-rendered frames and render only the missing ones (restart).
            resume: args.iter().any(|a| a == "--resume"),
        };
        let profile_reps = val("--reps").and_then(|s| s.parse().ok()).unwrap_or(5u32);
        let profile_regions = val("--regions").cloned();
        let divetest = val("--divetest").map(std::path::PathBuf::from);
        let livetest = val("--livetest").map(std::path::PathBuf::from);
        // --uitest [DIR]: presence enables the UI walk; a following non-flag token is the output
        // base directory (else the share/logs default). Built lazily so a normal launch pays nothing.
        let uitest = if args.iter().any(|a| a == "--uitest") {
            let out = val("--uitest").filter(|s| !s.starts_with('-')).map(std::path::PathBuf::from);
            Some(uitest::UiTest::new(out))
        } else {
            None
        };
        let autodive = if args.iter().any(|a| a == "--autodive") {
            let target = val("--autodive")
                .and_then(|v| v.parse::<f64>().ok())
                .filter(|v| v.is_finite() && *v > 1.0)
                .unwrap_or(320.0);
            let secs = val("--autodive-timeout")
                .and_then(|v| v.parse::<f64>().ok())
                .filter(|v| v.is_finite() && *v > 0.0)
                .unwrap_or(900.0);
            // Default 250_000 = the field crash manifest's explicit count. `--autodive-iter 0`
            // selects auto-iter instead, which is kept only so the two can be COMPARED: it is what
            // the first version forced, and it could not reach the regime.
            let iter = match val("--autodive-iter").and_then(|v| v.parse::<u32>().ok()) {
                Some(0) => None,
                Some(n) => Some(n),
                None => Some(250_000),
            };
            // Default 1 Home cycle: the zoom-home glide is the part that stresses the controller
            // (and the button that lost the device on 2026-08-18). `--autodive-home 0` dives only.
            let homes = val("--autodive-home")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(1)
                .min(20);
            Some(autopilot::AutoDive::new(target, secs, iter, homes))
        } else {
            None
        };
        // --motiontest: the motion-presentation gate (design/mode2-chunking.md §11) — asserts a
        // partial chunk progression is never adopted as the frozen texture during motion, and that
        // complete refreshes keep streaming. Self-contained: it jumps its own deep view.
        let motiontest = if args.iter().any(|a| a == "--motiontest") {
            Some(motiontest::MotionTest::new())
        } else {
            None
        };
        let juliadive = if args.iter().any(|a| a == "--juliadive") {
            let out =
                val("--juliadive").filter(|s| !s.starts_with('-')).map(std::path::PathBuf::from);
            Some(uitest::JuliaDive::new(out))
        } else {
            None
        };
        let chunk_sweep = chunksweep::ChunkSweep::from_args(args);
        let play_tour = val("--play").map(std::path::PathBuf::from);
        // Was the app launched to DO something specific, rather than to be explored? If so, no
        // first-run onboarding: a modal is either something nobody will ever click (a headless
        // harness) or something standing in front of the very thing the launch asked for.
        //
        // ⚠`--play` belongs here even though it is a perfectly ordinary windowed run, and leaving
        // it out was a real bug: the 2026-08-15 Radeon repro played its tour with the welcome
        // dialog sitting on top of it. That one did not block the tour — `--play` drives the live
        // view, so the dispatches underneath carried on and the measurement survived — but it
        // obscured the view being investigated, and the same omission against a harness that DOES
        // wait for a clean state is the hang this predicate exists to prevent. The question to ask
        // of a new flag is "did the user ask for a specific outcome?", not "is it headless?".
        let launched_for_a_task = livetest.is_some()
            || divetest.is_some()
            || uitest.is_some()
            || juliadive.is_some()
            || autodive.is_some()
            || motiontest.is_some()
            || chunk_sweep.is_some()
            || play_tour.is_some()
            || selftest
            || bench_matrix
            || profile
            || reusetest
            || resizetest
            || frametest
            || auto_render
            || auto_benchmark;
        let livetest_quick = args.iter().any(|a| a == "--quick");
        let frametest_steps = val("--steps").and_then(|s| s.parse().ok()).unwrap_or(40u32);
        let frametest_hold = val("--hold").and_then(|s| s.parse().ok()).unwrap_or(4u32);
        let frametest_dive = val("--dive").and_then(|s| s.parse::<f64>().ok()).unwrap_or(30.0);
        // --frametest --center X Y: dive at a custom point (default: the seahorse). Lets the
        // harness measure per-frame live-path costs along a REAL deep-dive line (a 34-digit
        // seahorse is precision-noise past ~1e34×, whose escaped-early references behave nothing
        // like a genuine filament dive's).
        let frametest_center = args
            .iter()
            .position(|a| a == "--center")
            .and_then(|i| match (args.get(i + 1), args.get(i + 2)) {
                (Some(x), Some(y)) => Some((x.clone(), y.clone())),
                _ => None,
            });
        let auto_benchmark_out = out_path.clone();
        let auto_render_out = out_path.clone();

        // Restore the last session (or defaults). The center comes from the
        // full-precision decimal strings when present (deep-zoom locations survive
        // restart); older session files without them fall back to the f64 fields.
        let (s, state_load) = fractadyne_state::load_with_status();
        // ⭐WHICH session this run is using, on the record. Every setting with no CLI flag —
        // coloring method, DE/lighting, series approximation, glitch correction — comes from that
        // file, so "which file, and did it actually load" is the first question behind any "why
        // does this render look different" report. It is also what makes a staged session
        // VERIFIABLE: `parse_with_status` falls back to defaults on an unparseable file, so a
        // harness that stages one (the F3 corpus generator) would otherwise render with defaults
        // and never know. `FRACTADYNE_CONFIG_DIR` relocates the file this names.
        crate::diag::log_line(
            "start",
            &format!(
                "session: {} — {}",
                fractadyne_state::state_location_display(),
                match &state_load {
                    fractadyne_state::StateLoad::Ok => "loaded".to_string(),
                    fractadyne_state::StateLoad::Fresh => "none (defaults)".to_string(),
                    fractadyne_state::StateLoad::Unreadable =>
                        "UNREADABLE, ignored (defaults)".to_string(),
                    fractadyne_state::StateLoad::Newer(v) => format!("loaded, newer format v{v}"),
                }
            ),
        );
        // Surface a warning (once the UI is up) if the session file was written by a newer build
        // than this one can fully account for.
        let pending_state_warning = match state_load {
            fractadyne_state::StateLoad::Newer(v) => Some(format!(
                "This session was saved by a newer Fractadyne (state v{v}; this build handles v{}). \
                 Some settings may not apply, and saving will rewrite it in this build's format.",
                fractadyne_state::STATE_FORMAT_VERSION
            )),
            _ => None,
        };
        let theme = ThemeMode::from_key(&s.theme);
        apply_theme(&cc.egui_ctx, theme);
        let mut viewport = Viewport::new(1280.0, 720.0);
        viewport.center_x = fractadyne_core::parse_bf(&s.center_x_str)
            .unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(s.center_x, 64));
        viewport.center_y = fractadyne_core::parse_bf(&s.center_y_str)
            .unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(s.center_y, 64));
        viewport.units_per_pixel = match restored_units_per_pixel(
            s.units_per_pixel,
            s.units_per_pixel_e,
        ) {
            Some(upp) => upp,
            None => {
                diag::log_line(
                    "start",
                    &format!(
                        "session zoom is UNUSABLE (mantissa={}, exp={}) — opening at the default view, keeping the saved centre",
                        s.units_per_pixel, s.units_per_pixel_e
                    ),
                );
                Viewport::new(1280.0, 720.0).units_per_pixel
            }
        };
        viewport.precision =
            fractadyne_core::precision_for_octaves(viewport.log2_magnification().max(0.0).ceil() as u64);
        // Restore the saved fractal family (so the view you left is fully recreated).
        let fractal = FractalKind::from_name(&s.fractal).unwrap_or(FractalKind::Mandelbrot);

        let mut app = Self {
            viewport,
            fractal,
            julia_c: (s.julia_c_re, s.julia_c_im),
            julia_mode: s.julia_mode && fractal.supports_julia(),
            click_zoom: s.click_zoom,
            julia_pin: None,
            dual: s.dual && fractal.supports_julia(),
            dual_split: s.dual_split.clamp(DUAL_SPLIT_MIN, DUAL_SPLIT_MAX),
            fullscreen: false,
            menu_bar_id: None,
            julia_viewport: {
                let mut v = Viewport::new(800.0, 800.0);
                v.center_x = fractadyne_core::BigFloat::from_f64(0.0, 64);
                v.center_y = fractadyne_core::BigFloat::from_f64(0.0, 64);
                v
            },
            pointer: PointerState::default(),
            home_anim: None,
            autopilot: AutopilotState {
                active: false,
                target: (0.5, 0.5),
                goal: (0.5, 0.5),
                eval_t: 0.0,
                stepping: false,
                dive_log2: s.autopilot_dive_log2,
            },
            anim: AnimationState {
                show_orbits: s.show_orbits,
                tour_orbit: None,
                orbit_normalize: s.orbit_normalize,
                orbit_anim: s.orbit_anim,
                orbit_anim_speed: s.orbit_anim_speed,
                orbit_phase: 0.0,
                orbit_hue: 0.0,
                palette_anim: PaletteAnim::from_key(&s.palette_anim),
                palette_anim_speed: s.palette_anim_speed,
                anim_dir: 1.0,
                random_palette: RandomPalette::new(0x9E37_79B9 ^ BUILD_SEQ.len() as u32),
            },
            orbit_cache: std::cell::RefCell::new(None),
            playback: None,
            last_script: s.last_script.clone().map(std::path::PathBuf::from),
            last_dir: s.last_dir.clone().map(std::path::PathBuf::from),
            playback_restore: None,
            tour_render: TourRenderUi::default(),
            diagnostics: Default::default(),
            bench_report: None,
            dialogs: DialogState {
                bench_open: false,
                bench_dialog_open: false,
                bookmarks_open: false,
                reset_confirm_open: false,
                // ⚠NEVER in a headless harness. The welcome modal is gated on a SESSION field, so
                // any harness given a fresh `FRACTADYNE_CONFIG_DIR` — which is exactly what a
                // hermetic run is — boots on "first run" and puts up a dialog nobody will ever
                // click. Measured 2026-08-15: `--livetest` under a wiped config reached the home
                // view in 0.85 s and then sat behind this modal, rendering frames but never
                // starting the tour. `--uitest` had already worked around it locally (it drives the
                // dialog on purpose); every other harness silently hung instead.
                //
                // The consequence was worse than one stuck run: it makes hermeticity and
                // automation mutually exclusive, so a torture rung with its own config dir would
                // block on EVERY live rung. Suppressing it for harness modes is what lets the
                // gates be reproducible.
                welcome_open: !s.welcome_seen && !launched_for_a_task,
                // Never in front of a harness: a modal would block --uitest/--livetest exactly the
                // way the welcome dialog once did.
                crash_prompt_open: crate::diag::previous_session_unclean()
                    && !s.crash_prompt_disabled
                    && !launched_for_a_task,
                accelerated_open: false,
                help_open: false,
                help_section: 0,
                right_panel_open: s.right_panel_open,
                minimap: s.minimap,
                script_export_open: false,
                script_export_note: String::new(),
                script_export_secs: 30.0,
                notice: None,
            },
            bench_cfg: BenchConfig::default(),
            std_bench: None,
            auto_stdbench,
            auto_stdbench_done: false,
            std_res,
            std_passes,
            std_depth,
            sysinfo: gather_system_info(Some(&gpu_name)),
            gpu_name,
            gpu_backend,
            report: ReportState::default(),
            auto_benchmark,
            auto_benchmark_out,
            auto_benchmark_done: false,
            auto_render,
            auto_render_out,
            auto_render_done: false,
            render_iter_mode,
            render_tour,
            render_tour_done: false,
            tour_cli,
            selftest,
            selftest_done: false,
            selftest_filter,
            selftest_list,
            selftest_bless,
            profile,
            profile_done: false,
            profile_reps,
            profile_regions,
            profile_out: out_path.clone(),
            bench_matrix,
            bench_matrix_done: false,
            reusetest,
            reusetest_done: false,
            resizetest,
            divetest,
            livetest,
            uitest,
            juliadive,
            chunk_sweep,
            autodive,
            motiontest,
            livetest_quick,
            play_tour,
            play_tour_done: false,
            frametest,
            frametest_center,
            frametest_steps,
            frametest_hold,
            frametest_dive,
            allow_tiled_settle: false,
            prof: std::cell::Cell::new(profile::ProfSetup::default()),
            fps_cap: (s.fps_cap > 0.0).then_some(s.fps_cap), // 0 = uncapped
            export: ExportState {
                open: false,
                width: s.export_width,
                ss: s.export_ss,
                format: if s.export_format == "exr" {
                    ExportFormat::Exr
                } else {
                    ExportFormat::Png
                },
                dual_mode: match s.export_dual_mode.as_str() {
                    "separate" => DualExport::Separate,
                    "active" => DualExport::ActiveOnly,
                    _ => DualExport::SideBySide,
                },
                aspect: s.export_aspect.clone(),
                // Start in Custom mode only if the restored size isn't one of the presets, so a
                // session saved at 1920×1080 reopens showing "Full HD 1080p".
                custom_size: false,
                notes: String::new(),
                status: None,
                task: None,
                prep: None,
                progress: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
                cancel: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                last_dir: s.export_dir.clone().map(std::path::PathBuf::from),
                started: None,
            },
            gallery: GalleryState { dir: Self::pictures_dir(), ..Default::default() },
            bookmarks: Self::load_bookmarks(),
            pending_thumb: None,
            thumb_shot: None,
            central_rect_px: [0, 0, 0, 0],
            thumb_cache: std::collections::HashMap::new(),
            bookmark_name: String::new(),
            nav: NavHistory::default(),
            goto: GotoDialog::default(),
            share: ShareDialog::default(),
            toast: None,
            // Booted by the device-loss handler's relaunch? Tell the user why the window blinked
            // (the session file restored their exact view; without this the restart is a mystery).
            pending_toast: (std::env::var_os("FRACTADYNE_RESTARTED_AFTER_GPU_LOSS").is_some())
                .then(|| {
                    "Recovered from a graphics device reset — your view was restored.".to_string()
                }),
            suppress_autosave: false,
            pending_state_warning,
            minimap_tex: None,
            minimap_key: None,
            coloring: ColoringConfig {
                palette_idx: s.palette_idx,
                cycle: s.cycle,
                offset: s.offset,
                custom_palette: s.custom_palette.clone(),
                use_custom_palette: s.use_custom_palette,
                palette_editor_open: false,
                paste_open: false,
                paste_text: String::new(),
                paste_msg: None,
                palette_rev: 0,
                use_duotone: s.use_duotone,
                use_binary: s.use_binary,
                duotone_lo: s.duotone_lo,
                duotone_hi: s.duotone_hi,
                color_method: ColorMethod::from_key(&s.color_method),
                stripe_freq: s.stripe_freq,
                trap_type: TrapType::from_key(&s.trap_type),
                normalize: args.iter().any(|a| a == "--normalize"),
                normalize_live: s.normalize_live,
                log_palette: s.log_palette || args.iter().any(|a| a == "--log-palette"),
            },
            perf: Perf {
                // Default on; `--no-perf` disables, `--perf` forces on.
                enabled: !std::env::args().any(|a| a == "--no-perf"),
                ..Perf::default()
            },
            render_cfg: RenderConfig {
                max_iter: s.max_iter,
                auto_iter: s.auto_iter,
                series_approx: s.series_approx,
                glitch_correct: s.glitch_correct,
                use_bla: s.use_bla,
                zoom_rate: s.zoom_rate,
                click_zoom_factor: s.click_zoom_factor.clamp(1.5, 1000.0),
                work_budget_scale: s.work_budget_scale.clamp(0.25, 8.0),
                min_motion_res: s.min_motion_res.clamp(0.30, 1.0),
                prefer_detail: s.prefer_detail,
                finish_sound: s.finish_sound,
                aa: s.aa,
            },
            effects: EffectsConfig {
                light: s.light,
                light_angle: s.light_angle,
                light_height: s.light_height,
                light_anim: s.light_anim,
                de: s.de,
                de_strength: s.de_strength,
                de_width: s.de_width,
                de_anim: s.de_anim,
                de_phase: 0.0,
            },
            watermark: s.watermark,
            show_location: s.show_location
                || args.iter().any(|a| a == "--show-location" || a == "--hud"),
            watermark_overlay: None,
            ui_scale: s.ui_scale.clamp(0.6, 2.5),
            theme,
            update_track: update::UpdateTrack::from_str(&s.update_track),
            update_check_on_launch: s.update_check_on_launch,
            attach_bytes_per_sample: (0, 0), // (granted, available) — filled on the first frame
            show_watermark: s.show_watermark,
            crash_prompt_disabled: s.crash_prompt_disabled,
            update_rx: None,
            update_status: None,
            update_launch_checked: false,
            update_manual: false,
            update_prompt_open: false,
            ref_cache: [RefCache::default(), RefCache::default()],
            last_saved_ref_id: None,
            ref_save_pending: None,
            recompute_rx: [None, None],
            ref_prefetch: Vec::new(),
            hold_prefetch: Vec::new(),
            last_state: s,
            dirty_since: None,
        };
        // `--bla` / `--no-bla` force BLA on/off for any headless mode (profiling / benchmark /
        // render), so it can be compared without a session file. Applied unconditionally (not just
        // the `--render` path) since `--profile`/`--benchmark` don't call `apply_cli_render`;
        // `--no-bla` wins if both are given.
        if args.iter().any(|a| a == "--bla") {
            app.render_cfg.use_bla = true;
        }
        if args.iter().any(|a| a == "--no-bla") {
            app.render_cfg.use_bla = false;
        }
        if args.iter().any(|a| a == "--no-watermark") {
            app.watermark = false;
        }
        if args.iter().any(|a| a == "--watermark") {
            app.watermark = true;
        }
        // Sound: `--no-sound` silences the render-finished tone (and `--sound` overrides
        // `FRACTADYNE_NO_SOUND` back on). Parsed before anything can finish a render.
        if args.iter().any(|a| a == "--no-sound") {
            crate::tone::set_muted(true);
        }
        if args.iter().any(|a| a == "--sound") {
            crate::tone::set_muted(false); // outranks FRACTADYNE_NO_SOUND without unsetting it
        }
        if args.iter().any(|a| a == "--glitch") {
            app.render_cfg.glitch_correct = true;
        }
        if args.iter().any(|a| a == "--no-glitch") {
            app.render_cfg.glitch_correct = false;
        }
        if app.auto_benchmark {
            app.start_benchmark();
        }
        if app.auto_render {
            app.apply_cli_render(args);
        }
        // `--import-kfr FILE`: load a Kalles Fraktaler location at startup (and before any
        // `--render`), so it works both live and headless.
        if let Some(p) = args
            .iter()
            .position(|a| a == "--import-kfr")
            .and_then(|i| args.get(i + 1))
        {
            match app.load_kfr_file(std::path::Path::new(p)) {
                Ok(m) => println!("{m}"),
                Err(e) => eprintln!("--import-kfr: {e}"),
            }
        }
        // `--import-imagina FILE`: same, for an Imagina TEXT location. A second independent
        // implementation to cross-check the validation corpus against — we otherwise compare only
        // to Fraktaler-3 (and to our own arbitrary-precision CPU oracle).
        if let Some(p) = args
            .iter()
            .position(|a| a == "--import-imagina")
            .and_then(|i| args.get(i + 1))
        {
            match app.load_imagina_file(std::path::Path::new(p)) {
                Ok(m) => println!("{m}"),
                Err(e) => eprintln!("--import-imagina: {e}"),
            }
        }
        // Resume the deep-zoom reference saved from last session so the restored view renders
        // immediately instead of rebuilding the (up to ~10 s) bignum orbit+SA+BLA. Best-effort: only
        // when the snapshot's view-key exactly matches the view we just restored (centre + zoom
        // exponent + formula + Julia); any mismatch/corruption falls through to the normal rebuild.
        if let Some(saved) = refcache_persist::load() {
            let e = app.viewport.units_per_pixel.e;
            let prec = app.viewport.precision;
            // The saved reference is valid iff we restored the SAME view: same zoom exponent + formula
            // + Julia, and the centre is sub-pixel-identical. The centre is compared NUMERICALLY, not
            // by decimal string — astro-float's to_string()/parse round-trip wobbles the trailing
            // digits, so string equality spuriously fails. `ref_offset_mantissa(c, saved, e, prec)` is
            // (c − saved)·2^-e ≈ the pixel offset; < 4 px means the same centre. Any mismatch (or a
            // parse failure) falls through to the normal rebuild.
            let center_ok = |s: &str, c: &fractadyne_core::BigFloat| {
                fractadyne_core::parse_bf(s)
                    .map(|b| fractadyne_core::ref_offset_mantissa(c, &b, e, prec).abs() < 4.0)
                    .unwrap_or(false)
            };
            let same_view = saved.upp_e == e
                && saved.formula_id == app.fractal.formula_id()
                && saved.julia == app.julia_mode
                && (!saved.julia
                    || ((saved.julia_c.0 - app.julia_c.0).abs() < 1e-12
                        && (saved.julia_c.1 - app.julia_c.1).abs() < 1e-12))
                && center_ok(&saved.center_x_str, &app.viewport.center_x)
                && center_ok(&saved.center_y_str, &app.viewport.center_y);
            if same_view && app.install_saved_ref(0, saved) {
                app.last_saved_ref_id = Some(app.ref_cache[0].orbit_id);
            }
        }
        app.nav.undo.push(app.snapshot_view()); // baseline for navigation undo
        app
    }

    /// Configure the view from `--render` CLI options (fractal / center / zoom / size
    /// / iterations / julia / palette). The actual render happens on the first frame.
    fn apply_cli_render(&mut self, args: &[String]) {
        let val = |name: &str| -> Option<&String> {
            args.iter().position(|a| a == name).and_then(|i| args.get(i + 1))
        };
        let two = |name: &str| -> Option<(&String, &String)> {
            args.iter()
                .position(|a| a == name)
                .and_then(|i| Some((args.get(i + 1)?, args.get(i + 2)?)))
        };
        if let Some(name) = val("--fractal") {
            match FractalKind::from_name(name) {
                Some(k) => self.fractal = k,
                None => {
                    eprintln!("fractadyne: --fractal: unknown family \"{name}\".");
                    crate::exit(2)
                }
            }
        }
        self.julia_mode = self.fractal.supports_julia() && args.iter().any(|a| a == "--julia");
        if let Some((re, im)) = two("--julia-c") {
            self.julia_c = (
                arg_parse::<f64>("--julia-c", re, "a number"),
                arg_parse::<f64>("--julia-c", im, "a number"),
            );
        }
        // --size: a bare width (`1920`) or `WIDTHxHEIGHT` (`5120x2160`). Set the viewport dimensions
        // *before* the center/zoom below — magnification is defined relative to the viewport height —
        // and drive the export width. A bare width keeps the current aspect.
        let (size_w, size_h) = val("--size").map(|s| arg_size("--size", s)).unwrap_or((None, None));
        if let Some(w) = size_w {
            let w = w.clamp(16, 16384);
            self.export.width = w;
            let h = size_h.map(|h| h.clamp(16, 16384)).unwrap_or_else(|| {
                ((w as f64) * self.viewport.height_px / self.viewport.width_px.max(1.0))
                    .round()
                    .clamp(16.0, 16384.0) as u32
            });
            self.viewport.width_px = w as f64;
            self.viewport.height_px = h as f64;
        }
        // Center: explicit (full precision) or the fractal's default.
        let center = two("--center").map(|(xs, ys)| arg_center("--center", xs, ys));
        let (cx, cy) = center.unwrap_or_else(|| {
            let (dx, dy) = self.fractal.default_center();
            (
                fractadyne_core::BigFloat::from_f64(dx, 64),
                fractadyne_core::BigFloat::from_f64(dy, 64),
            )
        });
        // `--zoom-log2 L` sets magnification = 2^L directly. `--zoom` takes a magnification
        // string and goes through `parse_zoom_to_log2` — the SAME parser the go-to field uses,
        // which accepts grouped digits, a FRACTIONAL exponent ("1.0e23.9") and magnitudes past
        // f64 range. It used to be a bare `s.parse::<f64>().ok().unwrap_or(1.0)`, and both of
        // that line’s failure modes were silent: "1.0e23.9" does not parse as f64, so the run
        // fell back to 1× and rendered the WHOLE SET with exit status 0 (this is what the
        // benchmark kit’s zoom-sequence lane was measuring against), while "4.6e1105" parses
        // to +inf, which is the blank-frame class fixed in beta.125. A value we cannot read is
        // now FATAL: the alternative is a render that looks perfectly fine and is of somewhere
        // else entirely.
        if let Some(l) = val("--zoom-log2").map(|s| arg_parse::<f64>("--zoom-log2", s, "a number")) {
            self.viewport.set_center_log2mag(cx, cy, l);
        } else if let Some(z) = val("--zoom") {
            match parse_zoom_to_log2(z) {
                Some(l2) => self.viewport.set_center_log2mag(cx, cy, l2),
                None => {
                    eprintln!(
                        "fractadyne: --zoom: cannot read \"{z}\" as a magnification. Write \
                         it as a positive number, optionally in scientific notation — 7.9e23, \
                         1e1105, or 1.0e23.9 for a fractional exponent."
                    );
                    crate::exit(2);
                }
            }
        } else {
            self.viewport.set_center_mag(cx, cy, 1.0);
        }
        if let Some(ss) = val("--ss").map(|s| arg_parse::<u32>("--ss", s, "a whole number")) {
            self.export.ss = ss.clamp(1, 8);
        }
        if let Some(it) = val("--iter").map(|s| arg_parse::<u32>("--iter", s, "a whole number")) {
            // An explicit CLI count is honored (the validation corpus renders fixed-iteration
            // frames up to the millions); the ceiling only guards against absurd typos — the real
            // practical bound is the reference-build time, which is the caller's choice to spend.
            self.render_cfg.max_iter = it.clamp(16, 100_000_000);
            self.render_cfg.auto_iter = false;
        }
        if let Some(p) = val("--palette").map(|s| arg_parse::<usize>("--palette", s, "a whole number")) {
            self.coloring.palette_idx = p.min(fractadyne_color::PRESETS.len() - 1);
            // A preset overrides any persisted binary/duotone/custom palette.
            self.coloring.use_binary = false;
            self.coloring.use_duotone = false;
            self.coloring.use_custom_palette = false;
        }
        if args.iter().any(|a| a == "--binary") {
            self.coloring.use_binary = true;
            self.coloring.use_duotone = false;
            self.coloring.use_custom_palette = false;
        }
        if args.iter().any(|a| a == "--light") {
            self.effects.light = true;
        }
        if let Some(a) =
            val("--light-angle").map(|s| arg_parse::<f32>("--light-angle", s, "a number"))
        {
            self.effects.light_angle = a;
        }
        if args.iter().any(|a| a == "--de") {
            self.effects.de = true;
        }
        // `from_key` ends in `unwrap_or_default()`, so an unknown key silently colours Smooth.
        // Round-trip the key to tell "you asked for smooth" from "we could not read that".
        if let Some(m) = val("--method") {
            let picked = ColorMethod::from_key(m);
            if picked.key() != m.as_str() {
                eprintln!("fractadyne: --method: unknown colouring \"{m}\".");
                crate::exit(2)
            }
            self.coloring.color_method = picked;
        }
        if let Some(f) =
            val("--stripe-freq").map(|s| arg_parse::<f32>("--stripe-freq", s, "a number"))
        {
            self.coloring.stripe_freq = f.clamp(1.0, 24.0);
        }
        if let Some(t) = val("--trap") {
            let picked = TrapType::from_key(t);
            if picked.key() != t.as_str() {
                eprintln!("fractadyne: --trap: unknown orbit trap \"{t}\".");
                crate::exit(2)
            }
            self.coloring.trap_type = picked;
        }
        // Output format from the file extension.
        if let Some(out) = &self.auto_render_out {
            if out.extension().and_then(|e| e.to_str()) == Some("exr") {
                self.export.format = ExportFormat::Exr;
            } else {
                self.export.format = ExportFormat::Png;
            }
        }
    }

    /// Snapshot current state and save it ~1 s after the last change (or on close).
    fn autosave(&mut self, ctx: &egui::Context) {
        // After a state reset the user chose not to persist this session — don't recreate the
        // file we just deleted (on the idle timer or on close).
        if self.suppress_autosave {
            return;
        }
        let cur = fractadyne_state::SessionState {
            state_version: fractadyne_state::STATE_FORMAT_VERSION,
            center_x: fractadyne_core::to_f64(&self.viewport.center_x),
            center_y: fractadyne_core::to_f64(&self.viewport.center_y),
            center_x_str: fractadyne_core::to_decimal_string(&self.viewport.center_x),
            center_y_str: fractadyne_core::to_decimal_string(&self.viewport.center_y),
            // ⚠NEVER PERSIST A NON-FINITE ZOOM, and shout when we nearly did. This is the other half
            // of `restored_units_per_pixel`: that guard stops a poisoned session from being LOADED,
            // this one stops it being WRITTEN, so a single bad frame cannot outlive the process.
            //
            // It is also the diagnostic. The 2026-08-18 field case (a NaN zoom that opened to a
            // black screen and selected the most expensive arithmetic mode) is unreproduced, and its
            // poisoned session was overwritten before it could be read — so the origin is still
            // unknown. `FloatExp::norm` passes a non-finite mantissa straight through, so once a NaN
            // exists anywhere it flows silently through every FloatExp operation and the first place
            // anyone notices is a black screen next launch. This logs at the moment of creation
            // instead, which is what will actually identify the source.
            units_per_pixel: if self.viewport.units_per_pixel.m.is_finite()
                && self.viewport.units_per_pixel.m != 0.0
            {
                self.viewport.units_per_pixel.m
            } else {
                diag::log_line(
                    "start",
                    &format!(
                        "REFUSING to persist a non-finite zoom (mantissa={}, exp={}) — saving the default view instead. This is a live bug: the viewport held an unusable magnification.",
                        self.viewport.units_per_pixel.m, self.viewport.units_per_pixel.e
                    ),
                );
                Viewport::new(1280.0, 720.0).units_per_pixel.m
            },
            units_per_pixel_e: if self.viewport.units_per_pixel.m.is_finite()
                && self.viewport.units_per_pixel.m != 0.0
            {
                self.viewport.units_per_pixel.e
            } else {
                Viewport::new(1280.0, 720.0).units_per_pixel.e
            },
            max_iter: self.render_cfg.max_iter,
            auto_iter: self.render_cfg.auto_iter,
            palette_idx: self.coloring.palette_idx,
            cycle: self.coloring.cycle,
            offset: self.coloring.offset,
            normalize_live: self.coloring.normalize_live,
            log_palette: self.coloring.log_palette,
            zoom_rate: self.render_cfg.zoom_rate,
            click_zoom: self.click_zoom,
            click_zoom_factor: self.render_cfg.click_zoom_factor,
            autopilot_dive_log2: self.autopilot.dive_log2,
            work_budget_scale: self.render_cfg.work_budget_scale,
            min_motion_res: self.render_cfg.min_motion_res,
            prefer_detail: self.render_cfg.prefer_detail,
            finish_sound: self.render_cfg.finish_sound,
            aa: self.render_cfg.aa,
            fps_cap: self.fps_cap.unwrap_or(0.0), // None (uncapped) → 0, so it round-trips
            export_width: self.export.width,
            export_ss: self.export.ss,
            export_format: match self.export.format {
                ExportFormat::Png => "png".to_string(),
                ExportFormat::Exr => "exr".to_string(),
            },
            export_dir: self
                .export.last_dir
                .as_ref()
                .map(|p| p.display().to_string()),
            last_dir: self.last_dir.as_ref().map(|p| p.display().to_string()),
            last_script: self.last_script.as_ref().map(|p| p.display().to_string()),
            welcome_seen: !self.dialogs.welcome_open,
            export_dual_mode: match self.export.dual_mode {
                DualExport::SideBySide => "side".to_string(),
                DualExport::Separate => "separate".to_string(),
                DualExport::ActiveOnly => "active".to_string(),
            },
            export_aspect: self.export.aspect.clone(),
            show_location: self.show_location,
            palette_anim: self.anim.palette_anim.key().to_string(),
            palette_anim_speed: self.anim.palette_anim_speed,
            light: self.effects.light,
            light_angle: self.effects.light_angle,
            light_height: self.effects.light_height,
            light_anim: self.effects.light_anim,
            de: self.effects.de,
            de_strength: self.effects.de_strength,
            de_width: self.effects.de_width,
            de_anim: self.effects.de_anim,
            color_method: self.coloring.color_method.key().to_string(),
            stripe_freq: self.coloring.stripe_freq,
            trap_type: self.coloring.trap_type.key().to_string(),
            minimap: self.dialogs.minimap,
            custom_palette: self.coloring.custom_palette.clone(),
            use_custom_palette: self.coloring.use_custom_palette,
            use_duotone: self.coloring.use_duotone,
            use_binary: self.coloring.use_binary,
            duotone_lo: self.coloring.duotone_lo,
            duotone_hi: self.coloring.duotone_hi,
            right_panel_open: self.dialogs.right_panel_open,
            fractal: self.fractal.name().to_string(),
            julia_mode: self.julia_mode,
            julia_c_re: self.julia_c.0,
            julia_c_im: self.julia_c.1,
            dual: self.dual,
            dual_split: self.dual_split,
            series_approx: self.render_cfg.series_approx,
            glitch_correct: self.render_cfg.glitch_correct,
            use_bla: self.render_cfg.use_bla,
            watermark: self.watermark,
            ui_scale: self.ui_scale,
            theme: self.theme.key().to_string(),
            update_track: self.update_track.as_str().to_string(),
            update_check_on_launch: self.update_check_on_launch,
            show_watermark: self.show_watermark,
            crash_prompt_disabled: self.crash_prompt_disabled,
            show_orbits: self.anim.show_orbits,
            orbit_normalize: self.anim.orbit_normalize,
            orbit_anim: self.anim.orbit_anim,
            orbit_anim_speed: self.anim.orbit_anim_speed,
        };
        let now = ctx.input(|i| i.time);
        if cur != self.last_state {
            self.last_state = cur;
            // Mark dirty on the FIRST change only — don't keep pushing the timer
            // forward on every frame, or a continuously-changing field (e.g. the
            // animated palette offset) would prevent the 1 s idle save from ever
            // firing (it would only save on close). This way it saves ~every 1 s.
            self.dirty_since.get_or_insert(now);
        }
        let closing = ctx.input(|i| i.viewport().close_requested());
        if let Some(t) = self.dirty_since {
            if closing || now - t > 1.0 {
                fractadyne_state::save(&self.last_state);
                self.dirty_since = None;
            }
        }
        // Persist the deep-zoom reference on its OWN debounce (not tied to the session-dirty flag —
        // the key case is loading a deep view and never touching it, where the session never goes
        // dirty). Save ~1 s after view 0's full reference stops changing (its `orbit_id` stable), so a
        // dive that rebuilds references every frame writes once on settle rather than churning 5 MB.
        let vc0 = &self.ref_cache[0];
        let unsaved = (!vc0.partial && vc0.ref_pt.is_some() && Some(vc0.orbit_id) != self.last_saved_ref_id)
            .then_some(vc0.orbit_id);
        match (unsaved, self.ref_save_pending) {
            (Some(id), Some((pid, _))) if pid == id => {} // still debouncing the same reference
            (Some(id), _) => self.ref_save_pending = Some((id, now)), // new unsaved ref → (re)start timer
            (None, _) => self.ref_save_pending = None,                // nothing to save
        }
        if let Some((_, t)) = self.ref_save_pending {
            if now - t > 1.0 || closing {
                self.save_reference_snapshot();
                self.ref_save_pending = None;
            }
        }
    }

    /// Serialize view 0's full reference to `last_reference.bin` if it changed since the last save.
    /// Off-thread: the ~4–5 MB serialize+write must not hitch the frame or the close. Skips shallow
    /// views (no reference) and coarse/in-progress references (`build_saved_ref` returns `None`).
    fn save_reference_snapshot(&mut self) {
        let Some(snapshot) = self.build_saved_ref(0) else {
            return;
        };
        let id = self.ref_cache[0].orbit_id;
        if self.last_saved_ref_id == Some(id) {
            return; // this exact reference is already on disk
        }
        self.last_saved_ref_id = Some(id);
        std::thread::spawn(move || {
            let _ = refcache_persist::save(&snapshot);
        });
    }

    /// Switch fractal type, resetting to that fractal's default view.
    fn set_fractal(&mut self, kind: FractalKind) {
        if self.fractal == kind {
            return;
        }
        self.fractal = kind;
        if !kind.supports_julia() {
            self.julia_mode = false;
            self.dual = false; // no Julia counterpart → dual view is meaningless
        }
        let (cx, cy) = kind.default_center();
        self.viewport.reset();
        self.viewport.center_x = fractadyne_core::BigFloat::from_f64(cx, 64);
        self.viewport.center_y = fractadyne_core::BigFloat::from_f64(cy, 64);
        self.pointer.zoom_vel = 0.0;
        self.invalidate_refs(); // dynamics changed → drop the cached reference orbits
    }

    /// Drop both per-view reference caches (call when the formula/mode/center changes
    /// such that the cached references no longer apply).
    fn invalidate_refs(&mut self) {
        self.ref_cache[0].ref_pt = None;
        self.ref_cache[1].ref_pt = None;
        // A new view's reference may escape where the old one didn't — retry extensions there.
        // Drop any in-flight recompute — its result is for the old fractal/mode and must not
        // install (would render the wrong formula until the next recompute).
        self.recompute_rx = [None, None];
        // Same for the playback lookahead: a prefetched reference for the old fractal/params
        // must never install after a change.
        self.ref_prefetch.clear();
        self.hold_prefetch.clear();
        // A new view's iteration needs are unrelated — restart the adaptive budget probe.
        self.perf.iter_boost = [1.0, 1.0];
        self.perf.iter_probe = [None, None];
        self.perf.iter_plateau = [false, false];
        self.perf.iter_stall = [0, 0];
        self.perf.iter_stall_base = [1.0, 1.0];
        self.perf.capped_frac = [None, None];
        self.perf.iter_exhausted = [false, false];
        self.perf.norm_range = [None, None];
    }

    /// Request the next animation frame. Frame pacing (the cap) is enforced at the end
    /// of `update`; this just keeps the animation loop alive.
    fn schedule_repaint(&self, ctx: &egui::Context) {
        ctx.request_repaint();
    }

    // compute_reference / series_skip_for / current_export_request_for moved to render.rs.

    /// Build the export job for the current state (single view, or dual per the chosen
    /// layout).
    /// Export image height (px) for the current width + aspect setting. "window" matches the live
    /// view's pixel aspect (same as the render); a fixed key uses that ratio. Uses the pixel aspect,
    /// not `complex_span` (which saturates to 0 past ~1e308× → a bogus 1-px height).
    fn export_height(&self) -> u32 {
        let ratio = if self.export.aspect == "window" {
            (self.viewport.width_px / self.viewport.height_px.max(1.0)).max(1.0e-6)
        } else {
            EXPORT_ASPECTS
                .iter()
                .find(|(k, _)| *k == self.export.aspect)
                .map(|(_, r)| *r)
                .unwrap_or(self.viewport.width_px / self.viewport.height_px.max(1.0))
        };
        ((self.export.width as f64) / ratio).round().max(1.0) as u32
    }

    fn build_export_job(&self) -> ExportJob {
        // Apply the chosen aspect: override the request height (the render centers the extra/fewer
        // rows on the same center; width stays `export_width`). For "window" this equals the height
        // the request already derived, so it's a no-op.
        let h = self.export_height();
        let fit = |mut req: fractadyne_gpu::ExportRequest| {
            req.height = h;
            // Keep the per-texel step isotropic (the GPU derives step = span/resolution per axis):
            // set the vertical span to match the horizontal step × the chosen height, so the fractal
            // isn't stretched when the aspect differs from the window. No-op for "Match window".
            req.span_mantissa.y = req.span_mantissa.x * (h as f64 / req.width.max(1) as f64);
            req
        };
        if self.dual {
            let map = fit(self.current_export_request_for(&self.viewport, false));
            let jul = fit(self.current_export_request_for(&self.julia_viewport, true));
            match self.export.dual_mode {
                DualExport::SideBySide => ExportJob::SideBySide(map, jul),
                DualExport::Separate => ExportJob::Separate(map, jul),
                DualExport::ActiveOnly => ExportJob::Single(map),
            }
        } else {
            ExportJob::Single(fit(self.current_export_request_for(&self.viewport, self.julia_mode)))
        }
    }

    /// Default Pictures directory (fallback: current dir).
    fn pictures_dir() -> std::path::PathBuf {
        directories::UserDirs::new()
            .and_then(|u| u.picture_dir().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."))
    }

    /// Directory to open a file dialog in: the last one the user browsed to (if it still exists),
    /// else `fallback`. Gives every open/save dialog a shared, persisted memory. `fallback` is a
    /// closure so the per-category default (e.g. Pictures) is only computed on a fresh install.
    fn dialog_dir(&self, fallback: impl FnOnce() -> std::path::PathBuf) -> std::path::PathBuf {
        self.last_dir
            .as_ref()
            .filter(|d| d.is_dir())
            .cloned()
            .unwrap_or_else(fallback)
    }

    /// The process's current directory, or `.` if it can't be read — a neutral dialog fallback
    /// for non-image categories (scripts, locations) with no natural home like Pictures.
    fn cwd_dir() -> std::path::PathBuf {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    }

    /// [`Self::dialog_dir`] with the neutral current-directory fallback — for dialogs with no
    /// category-specific default.
    fn dialog_dir_default(&self) -> std::path::PathBuf {
        self.dialog_dir(Self::cwd_dir)
    }

    /// Record the directory a dialog just landed in, so the next one opens there. `picked` may be
    /// a chosen file (remember its parent) or a chosen folder (remember it directly); a save
    /// target need not exist yet, so a non-directory path is treated as a file.
    fn remember_dir(&mut self, picked: &std::path::Path) {
        let dir = if picked.is_dir() {
            Some(picked.to_path_buf())
        } else {
            picked.parent().map(|p| p.to_path_buf())
        };
        if let Some(d) = dir.filter(|d| !d.as_os_str().is_empty()) {
            self.last_dir = Some(d);
        }
    }

    /// Path to the bookmarks file in the config dir (honours `FRACTADYNE_CONFIG_DIR`).
    fn bookmarks_path() -> Option<std::path::PathBuf> {
        fractadyne_state::config_dir().map(|d| d.join("bookmarks.toml"))
    }

    /// Load saved bookmarks (empty list if none / unreadable).
    fn load_bookmarks() -> Vec<Bookmark> {
        Self::bookmarks_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|t| toml::from_str::<BookmarkFile>(&t).ok())
            .map(|f| f.bookmark)
            .unwrap_or_default()
    }

    /// Persist bookmarks. A write failure loses durable user data, so it's surfaced as a toast
    /// (queued in `pending_toast` since this runs from contexts without an `egui::Context`).
    fn save_bookmarks(&mut self) {
        let Some(path) = Self::bookmarks_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let file = BookmarkFile {
            bookmark: self.bookmarks.clone(),
        };
        let result = match toml::to_string_pretty(&file) {
            Ok(text) => std::fs::write(&path, text),
            Err(e) => {
                self.pending_toast = Some(format!("Couldn't serialize bookmarks: {e}"));
                return;
            }
        };
        if let Err(e) = result {
            self.pending_toast = Some(format!("Couldn't save bookmarks: {e}"));
        }
    }

    /// Add a bookmark of the current view (auto-names it if `name` is blank).
    fn add_bookmark(&mut self, name: &str) {
        let name = if name.trim().is_empty() {
            format!(
                "{} {}×",
                self.fractal.name(),
                fmt_zoom_log2(self.viewport.log2_magnification())
            )
        } else {
            name.trim().to_string()
        };
        self.bookmarks.push(Bookmark {
            name,
            meta: self.view_metadata(),
            thumb: String::new(),
        });
        // Render the preview later this frame (GPU available in `update`, view unchanged).
        self.pending_thumb = Some(self.bookmarks.len() - 1);
        self.save_bookmarks();
    }

    /// Directory holding bookmark thumbnail PNGs (honours `FRACTADYNE_CONFIG_DIR`).
    fn bookmark_thumbs_dir() -> Option<std::path::PathBuf> {
        fractadyne_state::config_dir().map(|d| d.join("bookmark_thumbs"))
    }

    fn bookmark_thumb_path(id: &str) -> Option<std::path::PathBuf> {
        Self::bookmark_thumbs_dir().map(|d| d.join(format!("{id}.png")))
    }

    /// Bookmark thumbnails come from a SCREENSHOT of the central panel, not a re-render: a
    /// preview should show exactly what the user bookmarked (WYSIWYG — palette, normalization,
    /// settle state and all), and it costs zero GPU iterate work. The old path re-rendered
    /// through `render_export` synchronously on the main thread — a full reference build plus
    /// seconds of storm-priced tiles racing the live walk, which lost the device
    /// (crash-1787194989). Two-phase: request the shot this frame, harvest it the next
    /// (`ViewportCommand::Screenshot` replies as `egui::Event::Screenshot` — the uitest idiom).
    fn process_pending_thumb(&mut self, ctx: &egui::Context) {
        // Harvest first: a reply may be waiting from last frame's request.
        if let Some(i) = self.thumb_shot {
            let shot = ctx.input(|inp| {
                inp.events.iter().find_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
            });
            if let Some(img) = shot {
                self.thumb_shot = None;
                if i < self.bookmarks.len() {
                    let (iw, ih) = (img.size[0] as u32, img.size[1] as u32);
                    // Crop the central panel (physical px), clamped to the shot's bounds.
                    let [cx, cy, cw, chh] = self.central_rect_px;
                    let x0 = cx.min(iw.saturating_sub(1));
                    let y0 = cy.min(ih.saturating_sub(1));
                    let cw = cw.min(iw - x0).max(1);
                    let chh = chh.min(ih - y0).max(1);
                    let mut rgba = Vec::with_capacity((cw * chh * 4) as usize);
                    for y in y0..y0 + chh {
                        let row = &img.pixels[(y * iw + x0) as usize..(y * iw + x0 + cw) as usize];
                        for p in row {
                            rgba.extend_from_slice(&p.to_array());
                        }
                    }
                    let (tw, th, tpx) = fractadyne_export::box_thumbnail_rgba8(cw, chh, &rgba, 160);
                    let id = format!(
                        "{}-{i}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_nanos())
                            .unwrap_or(0)
                    );
                    if let Some(path) = Self::bookmark_thumb_path(&id) {
                        if let Some(parent) = path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        if fractadyne_export::write_png_rgba8(&path, tw, th, &tpx, None).is_ok() {
                            self.bookmarks[i].thumb = id;
                            self.save_bookmarks();
                        }
                    }
                }
            }
            return; // one round-trip at a time; the reply arrives on a later frame
        }
        // Request: fire the screenshot for a freshly added bookmark.
        if let Some(i) = self.pending_thumb.take() {
            if i < self.bookmarks.len() {
                self.thumb_shot = Some(i);
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            }
        }
    }

    /// Fetch (lazily loading + caching) the texture for a bookmark thumbnail id.
    fn bookmark_thumb_texture(&mut self, ctx: &egui::Context, id: &str) -> Option<egui::TextureHandle> {
        if id.is_empty() {
            return None;
        }
        if let Some(tex) = self.thumb_cache.get(id) {
            return Some(tex.clone());
        }
        let path = Self::bookmark_thumb_path(id)?;
        let (w, h, rgba) = fractadyne_export::read_png_rgba8(&path).ok()?;
        let img = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
        let tex = ctx.load_texture(format!("bmthumb.{id}"), img, egui::TextureOptions::LINEAR);
        self.thumb_cache.insert(id.to_string(), tex.clone());
        Some(tex)
    }

    /// UTC civil date/time `(year, month, day, hour, min, sec)` from a Unix timestamp
    /// (Hinnant's civil-from-days algorithm).
    fn civil_utc(secs: u64) -> (i64, i64, i64, u64, u64, u64) {
        let days = (secs / 86400) as i64;
        let rem = secs % 86400;
        let (hh, mm, sss) = (rem / 3600, (rem % 3600) / 60, rem % 60);
        let z = days + 719468;
        let era = if z >= 0 { z } else { z - 146096 } / 146097;
        let doe = z - era * 146097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        (y, m, d, hh, mm, sss)
    }

    /// UTC `YYYY-MM-DD HH:MM:SS` from a Unix timestamp.
    fn utc_date_string(secs: u64) -> String {
        let (y, m, d, hh, mm, sss) = Self::civil_utc(secs);
        format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}:{sss:02} UTC")
    }

    /// Frame size for a headless harness: `--size W|WxH` (with `--height` overriding the height)
    /// when given, else the harness's own default. Shares the tour flags so every harness takes
    /// the same size options.
    fn tour_size_or(&self, dw: u32, dh: u32) -> (u32, u32) {
        let w = self.tour_cli.width.unwrap_or(dw).clamp(16, 16384);
        let h = self
            .tour_cli
            .height
            .unwrap_or(if self.tour_cli.width.is_some() { (w * 9 / 16).max(16) } else { dh })
            .clamp(16, 16384);
        (w, h)
    }

    /// Filename-safe `YYYYMMDD_HHMMSS` stamp (local-readable, sorts chronologically).
    fn file_stamp(secs: u64) -> String {
        let (y, m, d, hh, mm, sss) = Self::civil_utc(secs);
        format!("{y:04}{m:02}{d:02}_{hh:02}{mm:02}{sss:02}")
    }

    // view_metadata / load_view_metadata / open_view moved to export.rs.
    /// Scan the gallery folder for exported PNG/EXR files with Fractadyne metadata,
    /// newest first. Thumbnails load lazily afterward.
    fn scan_gallery(&mut self) {
        self.gallery.entries.clear();
        let Ok(rd) = std::fs::read_dir(&self.gallery.dir) else {
            return;
        };
        for path in rd.flatten().map(|e| e.path()) {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase())
                .unwrap_or_default();
            let meta = match ext.as_str() {
                "png" => fractadyne_export::read_png_metadata(&path).ok().flatten(),
                "exr" => fractadyne_export::read_exr_metadata(&path).ok().flatten(),
                _ => None,
            };
            let Some(m) = meta else { continue };
            if meta_get(&m, "app") != "Fractadyne" {
                continue;
            }
            let zoom = meta_get(&m, "zoom")
                .parse::<f64>()
                .map(|z| format!("{}×", fmt_zoom(z)))
                .unwrap_or_default();
            self.gallery.entries.push(GalleryEntry {
                fractal: meta_get(&m, "fractal"),
                zoom,
                saved: meta_get(&m, "saved"),
                notes: meta_get(&m, "notes"),
                app_version: format!("Fractadyne {}", meta_get(&m, "version")),
                saved_unix: meta_get(&m, "saved_unix").parse().unwrap_or(0),
                path,
                meta: m,
                thumb: None,
                thumb_tried: false,
            });
        }
        self.gallery.entries
            .sort_by_key(|e| std::cmp::Reverse(e.saved_unix));
    }

    // export_ext / start_export / quick_export / render_to_file(_iter) / start_export_to moved to export.rs.

    // build_params moved to render.rs.

    /// Palette-cycle scaling for the GPU. The bounded statistical methods (stripe /
    /// triangle-inequality / decomposition) produce a 0..1 value, so they want a few
    /// cycles across the palette; the unbounded ones (iteration / trap / distance) use
    /// the fine per-unit scaling.
    fn color_cycle(&self) -> f32 {
        if self.coloring.color_method.needs_aux() {
            0.5 + self.coloring.cycle * 4.0
        } else {
            0.004 + self.coloring.cycle * 0.06
        }
    }

    /// The live-render work budget (`WORK_BUDGET`) scaled by the user's `work_budget_scale`. Higher
    /// lets deep/large frames render at fuller resolution (crisper) before the color pass falls back
    /// to a box-filtered upscale — at the cost of frame-rate and GPU-watchdog margin. Export is
    /// unaffected (always full resolution).
    pub(crate) fn effective_work_budget(&self) -> u64 {
        ((WORK_BUDGET as f64) * self.render_cfg.work_budget_scale.clamp(0.25, 8.0)).max(1.0e9) as u64
    }

    /// Render one fractal panel: navigation (drag-pan, wheel-zoom) + draw. Returns
    /// the panel's response (so the caller can read hover for the dual-view link).
    fn nav_and_draw(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        rect: egui::Rect,
        ppp: f64,
        scroll: f64,
        is_julia: bool,
    ) -> egui::Response {
        let resp = ui.allocate_rect(rect, egui::Sense::click_and_drag());
        let shift = ctx.input(|i| i.modifiers.shift);

        // Zoom box (Shift+drag): rubber-band a rectangle; on release, zoom so it fills the
        // panel. Handled before borrowing `vp` (uses the separate `self.pointer.zoom_box` field).
        if resp.drag_started() && shift {
            if let Some(p) = resp.interact_pointer_pos() {
                self.pointer.zoom_box = Some(ZoomBox { start: p, end: p, is_julia });
            }
        }
        let mut apply_zoom: Option<(f64, f64, f64)> = None; // (box_cx_px, box_cy_px, factor)
        let mut zoom_boxing = false;
        if self.pointer.zoom_box.as_ref().is_some_and(|z| z.is_julia == is_julia) {
            zoom_boxing = true;
            if let Some(cur) = resp.interact_pointer_pos() {
                self.pointer.zoom_box.as_mut().unwrap().end = cur;
            }
            let zb = self.pointer.zoom_box.as_ref().unwrap();
            let boxr = aspect_zoom_box(zb.start, zb.end, rect);
            // Foreground layer so the box draws above the fractal paint callback.
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("fract_zoom_box"),
            ));
            painter.rect_filled(
                boxr,
                egui::CornerRadius::ZERO,
                egui::Color32::from_rgba_unmultiplied(0xE0, 0xA0, 0x30, 32),
            );
            painter.rect_stroke(
                boxr,
                egui::CornerRadius::ZERO,
                egui::Stroke::new(1.5_f32, BRAND_ACCENT),
                egui::StrokeKind::Inside,
            );
            if resp.drag_stopped() {
                // Ignore a tiny box (an accidental Shift-click).
                if boxr.width() > 6.0 && boxr.height() > 6.0 {
                    apply_zoom = Some((
                        (boxr.center().x - rect.min.x) as f64 * ppp,
                        (boxr.center().y - rect.min.y) as f64 * ppp,
                        (boxr.width() / rect.width()) as f64, // < 1 ⇒ zoom in
                    ));
                }
                self.pointer.zoom_box = None;
                zoom_boxing = false;
            }
        }

        let vp = if is_julia {
            &mut self.julia_viewport
        } else {
            &mut self.viewport
        };
        let (nw, nh) = (rect.width() as f64 * ppp, rect.height() as f64 * ppp);
        // A canvas size change (window maximize / restore / edge-drag, or a dual-split drag) counts
        // as interacting, so the resize renders coarse and settles to full AA once the size holds —
        // otherwise every resize step re-renders at full 8× AA and the resize stutters.
        let resized = (nw - vp.width_px).abs() > 0.5 || (nh - vp.height_px).abs() > 0.5;
        vp.set_size(nw, nh);
        if let Some((bcx, bcy, factor)) = apply_zoom {
            let (w, h) = (vp.width_px, vp.height_px);
            vp.pan_pixels(w * 0.5 - bcx, h * 0.5 - bcy); // box center → screen center
            vp.zoom_at(w * 0.5, h * 0.5, factor); // then zoom the box to fill
        }
        // Plain drag pans (unless we're dragging a zoom box).
        if !zoom_boxing && resp.dragged_by(egui::PointerButton::Primary) {
            let d = resp.drag_delta();
            vp.pan_pixels(d.x as f64 * ppp, d.y as f64 * ppp);
        }
        let hovering = resp.hover_pos().is_some();
        if scroll != 0.0 {
            if let Some(p) = resp.hover_pos() {
                let l = p - rect.min;
                let f = (-0.0015 * scroll).exp();
                vp.zoom_at(l.x as f64 * ppp, l.y as f64 * ppp, f);
            }
        }
        let now = ctx.input(|i| i.time);
        // Dismiss a still-open menu the moment the user starts navigating the view. egui closes
        // menus on a click elsewhere, but wheel-zoom never clicks, and a drag's press can land on
        // the canvas without registering as one — either way the dropdown would hang over the
        // fractal while it pans/zooms underneath. Deliberately excludes `resized` (a window resize
        // isn't the user leaving the menu) but includes a zoom-box drag (pointer engagement).
        // (Inline rather than a `&self` helper: `vp` mutably borrows a self field here, so only
        // disjoint field reads — `menu_bar_id` is one — are allowed, not whole-`self` calls.)
        if resp.dragged()
            || apply_zoom.is_some()
            || (scroll != 0.0 && hovering)
            || (self.pointer.zoom_vel.abs() > 1e-3 && hovering)
        {
            if let Some(id) = self.menu_bar_id {
                // egui keeps the bar's open-menu state in a `BarState` keyed by the bar row's
                // `Ui::id` (recorded in `draw_menu_bar`); storing a default clears it through
                // public API. Guarded so egui memory isn't dirtied when no menu is open.
                if egui::menu::BarState::load(ctx, id).is_some() {
                    egui::menu::BarState::default().store(ctx, id);
                }
            }
        }
        // Drawing a zoom box drags the pointer but does NOT move the view, so it must not
        // count as "active" (that would drop the render back to the coarse moving preview).
        // Only the actual zoom application (apply_zoom) counts.
        let active = resized
            || (resp.dragged() && !zoom_boxing)
            || apply_zoom.is_some()
            || (scroll != 0.0 && hovering)
            || (self.pointer.zoom_vel.abs() > 1e-3 && hovering);
        let view = is_julia as usize;
        if active {
            self.pointer.settle_t[view] = now;
        }
        let interacting = now - self.pointer.settle_t[view] < SETTLE_DELAY;

        let eff_iter = if self.render_cfg.auto_iter {
            // Full depth-appropriate pixel appetite; freeze safety comes from the reference-length
            // cap (LIVE_REF_CAP via build_params), not the pixel count (see draw_central).
            vp.recommended_max_iter(self.render_cfg.max_iter)
        } else {
            self.render_cfg.max_iter
        };
        let center_bf = [vp.center_x.clone(), vp.center_y.clone()];
        let center = vp.center_f64();
        let span = vp.complex_span_fe();
        let mag = vp.magnification();
        let log2mag = vp.log2_magnification();
        // Progressive settle AA (after the `vp` borrow ends): coarse (1×) while moving, then refine
        // 1×→2×→4×→… up to the chosen level over consecutive frames, so a heavy view never blocks on
        // one full-AA frame.
        let aa_target = if interacting || self.tour_playing() {
            // Scripted playback: no settle AA ramp — see the note in `ui/central.rs`. Its cost is
            // quadratic and, outside the floatexp path, nothing bounds it.
            self.pointer.settle_frame[view] = 0;
            1
        } else {
            let ss = aa_ramp(self.pointer.settle_frame[view], self.render_cfg.aa);
            // Hold the ramp while a tiled settle is mid-grid: advancing ss changes the iterate key,
            // which would restart the grid every few frames and the view would never finish
            // sharpening. The ramp resumes the frame after the grid completes.
            if ss < self.render_cfg.aa && !self.perf.tile_pending[view] {
                self.pointer.settle_frame[view] += 1;
                self.schedule_repaint(ctx); // render the next, sharper AA stage
            }
            ss
        };
        let res = [
            (rect.width() as f64 * ppp) as u32,
            (rect.height() as f64 * ppp) as u32,
        ];
        let view_id = if is_julia { 1 } else { 0 };
        // Pan reprojection: while dragging, keep the last detailed frame and just translate it
        // with the cursor; re-render (at settled quality) only once the drag stops and the view
        // settles. Without this the coarse moving preview shows no detail at deep zoom, so you
        // can't see what you're panning toward. `pan_px` accumulates the same device-pixel
        // deltas fed to `pan_pixels`, so the slide matches the eventual settled render exactly.
        if resp.drag_started_by(egui::PointerButton::Primary) && !zoom_boxing {
            self.pointer.pan_px = egui::Vec2::ZERO;
            self.pointer.pan_view = Some(view_id);
        }
        if self.pointer.pan_view == Some(view_id)
            && !zoom_boxing
            && resp.dragged_by(egui::PointerButton::Primary)
        {
            let d = resp.drag_delta();
            self.pointer.pan_px += egui::vec2(d.x * ppp as f32, d.y * ppp as f32);
        }
        // ⭐YIELD THE GPU WHILE A TOUR RENDER OWNS IT (user-reported, 2026-08-16).
        //
        // The "Render tour…" dialog runs the render as a CHILD PROCESS, and nothing used to stop
        // the live view re-iterating behind it — in fact the dialog calls `request_repaint()` every
        // frame to animate its progress readout, so the parent was being woken deliberately to
        // compete with the single heaviest GPU work this app does. Two costs, and the second is the
        // one that matters:
        //
        //  - the render is simply slower, sharing the device with a full-quality live frame;
        //  - the CHILD'S COST MEASUREMENTS ARE POLLUTED. Its frame-cost controller sizes dispatches
        //    from measured GPU time while an uncontrolled second consumer is on the same card, so
        //    it prices frames it never rendered alone. That is the same error as running a gate
        //    beside a compile (design/torture-suite.md §P8), and here it feeds a controller whose
        //    mispricing has already cost two devices.
        //
        // Reuses the existing reproject path rather than inventing a suspend: `Some([0.0, 0.0])`
        // is exactly what a resize does — hold the last completed frame and re-sample it, no
        // iterate dispatch. Checked FIRST so it also wins over pan/zoom interaction.
        let render_child_busy = self.tour_render.child.is_some();
        let reproject = if render_child_busy {
            Some([0.0, 0.0])
        } else if self.pointer.pan_view == Some(view_id) && interacting {
            self.schedule_repaint(ctx); // keep rendering until the view settles
            Some([
                self.pointer.pan_px.x / res[0].max(1) as f32,
                self.pointer.pan_px.y / res[1].max(1) as f32,
            ])
        } else if resized {
            // Window/panel resize: hold the last frame rather than stretching it to the new aspect
            // ratio. The color pass fit-centres the frozen frame at native scale (center stays
            // centred) and fills any newly revealed border with the average color, until the size
            // settles and it re-iterates. `pan_px` stays at whatever it was (uv offset 0 here).
            self.schedule_repaint(ctx);
            Some([0.0, 0.0])
        } else {
            if self.pointer.pan_view == Some(view_id) {
                self.pointer.pan_view = None; // settled → next frame does a full re-iterate
            }
            None
        };
        // Only the live view may start a tiled settle (the profiling/benchmark callers of
        // `build_params` time single dispatches).
        self.allow_tiled_settle = true;
        let params = self.build_params(
            center_bf,
            center,
            span,
            mag,
            log2mag,
            self.fractal,
            is_julia,
            eff_iter,
            interacting,
            aa_target,
            res,
            view_id,
            reproject,
        );
        self.allow_tiled_settle = false;
        // A settle grid (or a chunked iteration progression) in progress needs the next frame
        // promptly — each frame renders one tile / one iteration range.
        if self.perf.tile_pending[view_id as usize] || self.perf.chunk_pending[view_id as usize] {
            self.schedule_repaint(ctx);
        }
        add_mandelbrot(ui.painter(), rect, params);
        resp
    }


    /// Screen position (points) of a complex coordinate in the Mandelbrot viewport,
    /// within the given panel rect. Inverse of `complex_at_pixel_f64`.
    fn complex_screen_pos(&self, c: (f64, f64), rect: egui::Rect, ppp: f64) -> egui::Pos2 {
        let (cx, cy) = self.viewport.center_f64();
        let upp = self.viewport.units_per_pixel.to_f64();
        let px = (c.0 - cx) / upp + self.viewport.width_px * 0.5;
        let py = self.viewport.height_px * 0.5 - (c.1 - cy) / upp;
        egui::pos2(
            rect.min.x + (px / ppp) as f32,
            rect.min.y + (py / ppp) as f32,
        )
    }


    /// Performance diagnostics, rendered into a docked panel section (FPS, CPU/GPU
    /// split, reference-recompute cost, and current render state).
    fn perf_section(&self, ui: &mut egui::Ui) {
        let p = &self.perf;
        let fps = if p.frame_ms > 0.0 { 1000.0 / p.frame_ms } else { 0.0 };
        let gpu_idle = (p.frame_ms - p.cpu_ms).max(0.0);
        let mode = match p.last_mode {
            1 => "direct df32",
            2 => "perturb floatexp",
            _ => "perturb df32",
        };
        // State-aware activity line. The raw repaint cadence (1000/frame_ms) is what a user
        // reads as "FPS", but it only MEANS frames-per-second while frames genuinely render:
        // an idle settled view repaints on the ~1 Hz heartbeat (showing "1.0" while computing
        // nothing), and a long capped refinement shows tile cadence (each frame is 1/Nth of a
        // composite, not a completed frame). So say what the renderer is actually doing:
        //   refining k/N (~m:ss) — a settle grid / chunked progression is composing the frame
        //   building reference   — the bignum orbit is still building (spinner shows too)
        //   idle                 — settled and quiet; no number pretending otherwise
        //   NN.N                 — real frames per second (interaction, animation, motion)
        let mmss = |secs: f64| -> String {
            let s = secs.max(0.0) as u64;
            format!("{}:{:02}", s / 60, s % 60)
        };
        let animating = self.anim.palette_anim != PaletteAnim::Off
            || (self.anim.show_orbits && self.anim.orbit_anim);
        let grid = p.tile_state[0].as_ref().filter(|_| p.tile_pending[0]);
        let line = if let Some(g) = grid {
            match g.geo {
                Some((gres, _, side)) => {
                    let total = gres[0].div_ceil(side).max(1) * gres[1].div_ceil(side).max(1);
                    let done = g.next.min(total);
                    let eta = (total.saturating_sub(done)) as f64 * p.frame_ms / 1000.0;
                    format!("FPS        refining {done}/{total} (~{})", mmss(eta))
                }
                None => "FPS        refining (arming)".to_string(),
            }
        } else if p.chunk_pending[0] {
            let ask = p.chunk_sig[0].1.max(1);
            let pct = 100.0 * p.chunk_cursor[0].min(ask) as f64 / ask as f64;
            format!("FPS        refining {pct:4.1}%")
        } else if self.recompute_rx[0].is_some() {
            "FPS        building reference".to_string()
        } else if !animating && p.frame_idx.saturating_sub(p.fe_dispatch_frame[0]) > 8 {
            // Nothing dispatched for a while and nothing animating: the repaint cadence is just
            // the idle heartbeat — don't dress it up as a framerate.
            "FPS        idle".to_string()
        } else {
            format!("FPS        {fps:6.1}")
        };
        // Annunciated row: default color and no marker when healthy; amber " !" at a warning
        // and red " !!" in a danger band. The marker is trailing ASCII so a level change never
        // moves the label column, and the glyph+color pair stays legible without color vision.
        // ⚠Thresholds are read from the live tunables / device limits at draw time — never
        // mirrored constants (the profile.rs SETTLE_DELAY 0.35-vs-0.18 drift is the recorded
        // warning about restating a number the code already owns).
        let level_row = |ui: &mut egui::Ui, level: u8, text: String| -> egui::Response {
            match level {
                2 => ui.monospace(
                    egui::RichText::new(format!("{text} !!"))
                        .color(egui::Color32::from_rgb(235, 90, 70)),
                ),
                1 => ui.monospace(
                    egui::RichText::new(format!("{text} !"))
                        .color(egui::Color32::from_rgb(235, 170, 50)),
                ),
                _ => ui.monospace(text),
            }
        };
        ui.monospace(line);
        ui.monospace(format!("frame      {:6.2} ms", p.frame_ms));
        ui.monospace(format!("cpu        {:6.2} ms", p.cpu_ms));
        ui.monospace(format!("gpu/idle   {gpu_idle:6.2} ms"));
        // The cost controller's measurement source. Wall-clock FALLBACK is the danger state:
        // the budget is pricing frames without GPU timings, which is exactly how both fatal
        // 2026-08-19 sessions ran ~1 s frames with the budget still growing.
        let (timing_lvl, timing_txt) = if p.wall_fallback {
            (2u8, "wall clock (timestamps starved)")
        } else if p.ts_supported {
            (0, "GPU timestamps")
        } else {
            (1, "wall clock (no TIMESTAMP_QUERY)")
        };
        level_row(ui, timing_lvl, format!("timing     {timing_txt}")).on_hover_text(
            "How frame cost is measured. GPU timestamps are the precise source; the wall-clock              fallback engages when timings stop arriving, and a budget priced by wall clock has              repeatedly mis-sized deep frames — treat red here as 'the safety margins are              estimates right now'.",
        );
        ui.separator();
        ui.monospace(format!("mode       {mode}"));
        ui.monospace(format!("eff iter   {:>7}", p.last_eff_iter));
        ui.monospace(format!("precision  {:>5} bit", p.last_precision));
        {
            // The device's orbit ceiling (storage-binding limit): past it the live view cannot
            // resolve deeper — the documented ~1e95 depth wall. Announce the approach.
            let cap = crate::render::orbit_len_cap();
            let lvl = if cap != u32::MAX && p.last_orbit_len as u64 * 10 > cap as u64 * 9 {
                2
            } else if cap != u32::MAX && p.last_orbit_len as u64 * 10 > cap as u64 * 7 {
                1
            } else {
                0
            };
            level_row(ui, lvl, format!("orbit len  {:>7}", p.last_orbit_len)).on_hover_text(
                format!(
                    "Reference orbit samples. This GPU holds at most {} — approaching it is the                      practical depth wall: past it the live view cannot resolve deeper locations                      that need a longer orbit.",
                    if cap == u32::MAX { "(uncapped)".to_string() } else { cap.to_string() }
                ),
            );
        }
        if p.last_sa_skip > 0 {
            ui.monospace(format!("SA skip    {:>7}", p.last_sa_skip));
        }
        ui.monospace(format!("aa         {}x", self.render_cfg.aa));
        ui.monospace(format!("dual       {}", self.dual));
        ui.monospace(format!("zoom       {}×", fmt_zoom_log2(self.viewport.log2_magnification())));
        // Deep-zoom budget telemetry (D3.5): the measured live iterate + the adaptive step
        // budget it feeds. Only meaningful in floatexp mode (mode 2), where the budget runs.
        if p.last_mode == 2 && p.last_iterate_ms[0] > 0.0 {
            let gsps = p.fe_steps_last[0] as f64 / (p.last_iterate_ms[0] / 1000.0) / 1.0e9;
            ui.separator();
            // The one number every device loss this release cycle shared: measured iterate
            // time creeping toward the ~900 ms lethal band. Amber past the controller's own
            // target, red inside the band.
            let c = crate::tunables::cost();
            let target = if self.render_cfg.auto_iter {
                c.tdr_budget_ms
            } else {
                c.tdr_explicit_budget_ms
            };
            let it_lvl = if p.last_iterate_ms[0] >= c.tdr_lethal_ms {
                2
            } else if p.last_iterate_ms[0] >= target {
                1
            } else {
                0
            };
            level_row(ui, it_lvl, format!("iterate    {:6.1} ms (GPU)", p.last_iterate_ms[0]))
                .on_hover_text(format!(
                    "Measured GPU time of the last iterate dispatch. The controller aims at                      {target:.0} ms; sustained readings near {:.0} ms are the band where this                      hardware class has lost the device.",
                    c.tdr_lethal_ms
                ));
            ui.monospace(format!("steps/s    {gsps:6.2} G"));
            // Amber while the budget is still CLIMBING: an unconverged budget is priced from
            // sparse readings, and every mis-sized settled dispatch this cycle happened in
            // exactly this state.
            level_row(
                ui,
                if p.fe_budget_ok[0] { 0 } else { 1 },
                format!(
                    "budget     {:.2e}{}",
                    p.fe_budget[0] as f64,
                    if p.fe_budget_ok[0] { " ✔" } else { " (settling)" }
                ),
            );
        }

        ui.separator();
        // Process memory (polled ~1 Hz) — deep reference builds have peaked at 2.3 GB in the
        // field with nothing on screen saying so. Amber past 3/4 of system RAM.
        let rss_lvl = match self.perf.mem_total {
            Some(total) if total > 0 && self.perf.mem_rss.saturating_mul(4) > total.saturating_mul(3) => 1u8,
            _ => 0,
        };
        level_row(
            ui,
            rss_lvl,
            format!("rss        {:>5} MB (peak {})", p.mem_rss >> 20, p.mem_peak >> 20),
        );
        // Estimated GPU-resident bytes, assembled from the allocation sizes the app knows:
        // reference orbits + BLA trees (both views), the iteration G-buffer pair, the
        // chunk-state ping-pong while a walk holds one, and the present-gate hold snapshot.
        // An estimate — wgpu cannot portably report the driver's true figure — so it is
        // labelled as one and carries no annunciator.
        let gpu_est: u64 = {
            let mut b = 0u64;
            for vi in 0..2 {
                b += self.ref_cache[vi].orbit_len as u64 * 16;
                b += self.ref_cache[vi].bla.len() as u64 * 16;
                let (ss, res, _) = p.aa_last_key[vi];
                let tex =
                    (res[0] as u64 * ss.max(1) as u64) * (res[1] as u64 * ss.max(1) as u64) * 16;
                b += tex * 2; // iteration G-buffer pair (tex + aux)
                if p.chunk_pending[vi] || p.chunk_last_range[vi].is_some() {
                    let targets = if p.last_mode == 2 { 4 } else { 3 };
                    b += tex * 2 * targets; // state ping-pong, two sets
                }
                if p.hold_active[vi] {
                    b += tex * 2; // hold snapshot pair
                }
            }
            b
        };
        ui.monospace(format!("gpu est.   {:>5} MB", gpu_est >> 20)).on_hover_text(
            "Estimated GPU-resident memory: reference orbits, BLA trees, iteration textures,              the chunk-state ping-pong while a refinement runs, and the present-gate snapshot.              Assembled from known allocation sizes; the driver's true figure is not portably              readable.",
        );

        // Julia parameter the Julia / dual view renders, plus how much c-space the
        // whole Mandelbrot panel covers. When "c/panel" drops far below one Julia
        // pixel (≈ Julia span ÷ panel width), hovering still updates c but the Julia
        // looks static — expected at deep Mandelbrot zoom, not a freeze.
        if self.dual || self.julia_mode {
            ui.separator();
            let (jr, ji) = self.julia_c;
            ui.monospace(format!("julia c.re {jr:+.15}"));
            ui.monospace(format!("julia c.im {ji:+.15}"));
            if self.julia_pin.is_some() {
                ui.monospace("julia c    pinned");
            }
            let c_per_panel = self.viewport.width_px * self.viewport.units_per_pixel.to_f64();
            ui.monospace(format!("c/panel    {c_per_panel:.3e}"))
                .on_hover_text(
                    "Width of c-space spanned by the whole Mandelbrot panel. When this \
                     is far below one Julia pixel, hovering changes c but the Julia \
                     looks unchanged — expected at deep zoom, not a freeze.",
                );
        }

        ui.separator();
        ui.monospace(format!("ref recompute {:6.2} ms", p.recompute_ms));
        ui.monospace(format!("recompute/s   {:>4.0}", p.recompute_per_s));
        ui.monospace(format!("ref builds/s  {:>4.0}", p.builds_per_s));
        ui.monospace(format!("recompute tot {:>5}", p.recompute_total));
    }

    /// Snapshot the current location for navigation history.
    fn snapshot_view(&self) -> ViewSnapshot {
        ViewSnapshot {
            cx: self.viewport.center_x.clone(),
            cy: self.viewport.center_y.clone(),
            upp: self.viewport.units_per_pixel,
            prec: self.viewport.precision,
        }
    }

    /// Restore a navigation snapshot (location only).
    fn apply_snapshot(&mut self, s: &ViewSnapshot) {
        self.viewport.center_x = s.cx.clone();
        self.viewport.center_y = s.cy.clone();
        self.viewport.units_per_pixel = s.upp;
        self.viewport.precision = s.prec;
        self.pointer.zoom_vel = 0.0;
        self.invalidate_refs();
    }

    /// Record the current location onto the undo history (deduped vs. the top), and
    /// clear the redo stack. Called when the view settles and after discrete jumps.
    fn record_nav(&mut self) {
        let snap = self.snapshot_view();
        let dup = self.nav.undo.last().is_some_and(|t| {
            t.upp == snap.upp && t.cx == snap.cx && t.cy == snap.cy
        });
        if !dup {
            self.nav.undo.push(snap);
            if self.nav.undo.len() > 256 {
                self.nav.undo.remove(0);
            }
            self.nav.redo.clear();
        }
    }

    /// Step back / forward through visited locations.
    fn undo_view(&mut self) {
        if self.nav.undo.len() < 2 {
            return;
        }
        let cur = self.nav.undo.pop().unwrap();
        self.nav.redo.push(cur);
        let prev = self.nav.undo.last().unwrap().clone();
        self.apply_snapshot(&prev);
        self.nav.was_interacting = false;
    }
    fn redo_view(&mut self) {
        if let Some(s) = self.nav.redo.pop() {
            self.apply_snapshot(&s);
            self.nav.undo.push(s);
            self.nav.was_interacting = false;
        }
    }

    /// Open the go-to-location dialog, pre-filled with the current view.
    fn open_goto(&mut self) {
        self.goto.x = fractadyne_core::to_decimal_string(&self.viewport.center_x);
        self.goto.y = fractadyne_core::to_decimal_string(&self.viewport.center_y);
        self.goto.zoom = fmt_zoom_field(self.viewport.log2_magnification());
        self.goto.msg = None;
        self.goto.open = true;
    }

    /// Apply the go-to-location dialog: parse + validate, then jump (recording history).
    fn apply_goto(&mut self) {
        // Zoom first: it sets the precision the coordinates are parsed at, so an inexact
        // rational like 37/100 carries enough digits to be viewed at the requested depth.
        // Clamp to a sane octave bound so a pasted absurd zoom can't request runaway precision.
        let log2mag = parse_zoom_to_log2(&self.goto.zoom)
            .filter(|l| l.is_finite())
            .map(|l| l.clamp(0.0, 1.0e6));
        let prec = fractadyne_core::precision_for_octaves(log2mag.unwrap_or(0.0) as u64);
        // The real field also accepts a whole complex expression — `(37+16i)/100` fills both
        // coordinates at once. Both fields are rewritten to the resolved decimals so what was
        // applied is visible rather than implied.
        let (cx, cy) = match fractadyne_core::parse_complex_prec(self.goto.x.trim(), prec) {
            Some((re, im)) if !im.is_zero() => {
                self.goto.x = fractadyne_core::to_decimal_string(&re);
                self.goto.y = fractadyne_core::to_decimal_string(&im);
                (Some(re), Some(im))
            }
            _ => (
                fractadyne_core::parse_bf_prec(self.goto.x.trim(), prec),
                fractadyne_core::parse_bf_prec(self.goto.y.trim(), prec),
            ),
        };
        match (cx, cy, log2mag) {
            (Some(cx), Some(cy), Some(l)) => {
                self.viewport.set_center_log2mag(cx, cy, l);
                self.pointer.zoom_vel = 0.0;
                self.invalidate_refs();
                self.record_nav();
                self.goto.msg = None;
                self.goto.open = false;
            }
            _ => {
                self.goto.msg = Some("Invalid input — check the coordinates and zoom.".into());
            }
        }
    }

    /// Set a transient status toast (auto-fades after a few seconds).
    fn set_toast(&mut self, msg: impl Into<String>, ctx: &egui::Context) {
        self.toast = Some((msg.into(), ctx.input(|i| i.time)));
    }

    /// "Zoom to center": find the nearby minibrot's exact nucleus (Newton-Raphson in
    /// arbitrary precision) and snap the view center to it, keeping the current zoom.
    /// Reports the period. Holomorphic families only (Mandelbrot / Multibrot).
    /// Kick off an update check on a background thread (non-blocking). `manual` = user-initiated
    /// (toast every outcome); the launch check passes `false` (toast only when an update exists).
    /// No-op if a check is already running.
    fn start_update_check(&mut self, manual: bool) {
        if self.update_rx.is_some() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let track = self.update_track;
        let cur = update::running_version();
        std::thread::spawn(move || {
            let _ = tx.send(update::check(track, &cur));
        });
        self.update_rx = Some(rx);
        self.update_status = None;
        self.update_manual = manual;
    }

    /// Poll the in-flight update check; on completion keep the status (for the Help menu) and toast
    /// the outcome — a manual check reports all outcomes, the silent launch check only an update.
    fn poll_update_check(&mut self, ctx: &egui::Context) {
        let done = self.update_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(status) = done {
            self.update_rx = None;
            match &status {
                // Surface the download link directly via the update prompt (below), not a
                // dead-end toast that just points at the Help menu.
                update::UpdateStatus::Available { .. } => {
                    self.update_prompt_open = true;
                }
                update::UpdateStatus::UpToDate if self.update_manual => {
                    self.set_toast("You're on the latest version.", ctx);
                }
                update::UpdateStatus::Error(e) if self.update_manual => {
                    self.set_toast(format!("Update check failed: {e}"), ctx);
                }
                _ => {}
            }
            self.update_status = Some(status);
        }
    }

    /// The "Update available" prompt: shows the new version + a direct **Download from GitHub**
    /// link (opens the release page). Dismissable ("Remind me later"); the Help menu keeps the
    /// same link afterwards. No auto-install.
    fn draw_update_dialog(&mut self, ctx: &egui::Context) {
        if !self.update_prompt_open {
            return;
        }
        // Only meaningful while an update is actually pending.
        let Some(update::UpdateStatus::Available { version, url, prerelease }) = &self.update_status
        else {
            self.update_prompt_open = false;
            return;
        };
        let (version, url, prerelease) = (version.clone(), url.clone(), *prerelease);
        let channel = update::channel_word(prerelease);
        let track = self.update_track.label();
        let current = update::running_version();
        let green = egui::Color32::from_rgb(0x5C, 0xC0, 0x6C);
        let mut open = true;
        let (mut download, mut later) = (false, false);
        egui::Window::new("Update available")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .default_width(430.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("\u{2B06}").size(22.0).color(green));
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "Fractadyne {version} ({channel}) is available"
                            ))
                            .strong(),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "You're running {current}  ·  {track}"
                            ))
                            .weak()
                            .small(),
                        );
                    });
                });
                ui.add_space(10.0);
                ui.label("Download the latest release from GitHub:");
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("\u{2B07}  Download from GitHub")
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(green),
                        )
                        .clicked()
                    {
                        download = true;
                    }
                    if ui.button("Remind me later").clicked() {
                        later = true;
                    }
                });
                ui.add_space(6.0);
                ui.hyperlink_to(
                    egui::RichText::new(format!("{url} \u{2197}")).small(),
                    &url,
                );
            });
        if download {
            ctx.open_url(egui::OpenUrl::new_tab(url));
        }
        if download || later || !open {
            self.update_prompt_open = false;
        }
    }

    /// Newton-Raphson zoom framing: the minibrot's own width occupies this fraction of the view
    /// height after the jump.
    ///
    /// Chosen by measurement, not taste. The visible copy — cardioid, bulbs, and the embedded
    /// Julia decoration that reads as part of it — runs about 1.6× the bare size estimate at high
    /// period, so 0.25 puts roughly 40% of the frame on the minibrot and leaves the rest as
    /// context. It also makes the degenerate case exact: period 1 (the whole set, size 1) frames
    /// at magnification 1 — precisely the home view.
    const ATOM_FILL: f64 = 0.25;

    /// Depth that frames a minibrot of the given `log₂` size — the destination of a
    /// Newton-Raphson zoom.
    fn atom_frame_log2mag(log2_size: f64) -> f64 {
        Viewport::REFERENCE_HEIGHT.log2() + Self::ATOM_FILL.log2() - log2_size
    }

    fn find_minibrot(&mut self, ctx: &egui::Context) {
        let formula = self.fractal.formula_id();
        if !matches!(formula, 0..=3) {
            self.set_toast(
                "Minibrot finder needs a holomorphic family (Mandelbrot / Multibrot).",
                ctx,
            );
            return;
        }
        let mag = self.viewport.magnification();
        let center = [self.viewport.center_x.clone(), self.viewport.center_y.clone()];
        let max_period =
            self.viewport.recommended_max_iter(self.render_cfg.max_iter).clamp(1_000, 100_000);
        match fractadyne_core::find_nucleus(&center, mag, formula, max_period) {
            Some(n) => {
                let cur_l2 = self.viewport.log2_magnification();
                let (cx, cy, target) = self.newton_raphson_target(n.cx, n.cy, n.period, formula);
                match target.filter(|t| *t > cur_l2) {
                    Some(t) => {
                        self.viewport.set_center_log2mag(cx, cy, t);
                        self.finish_nav_jump();
                        self.set_toast(
                            format!(
                                "Zoomed to the period-{} minibrot — {}×",
                                n.period,
                                fmt_zoom_field(t)
                            ),
                            ctx,
                        );
                    }
                    // No size estimate (non-quadratic family), or the view is already deeper
                    // than the minibrot's own scale — keep the depth, just fix the center.
                    None => {
                        self.viewport.set_center_log2mag(cx, cy, cur_l2);
                        self.finish_nav_jump();
                        self.set_toast(
                            format!("Snapped to period-{} minibrot center", n.period),
                            ctx,
                        );
                    }
                }
            }
            None => self.set_toast("No minibrot center found near the view center.", ctx),
        }
    }

    /// Size up a located minibrot and refine its center to the precision that depth demands.
    ///
    /// Two passes, because the two quantities define each other: the size estimate needs an
    /// accurate center, and how accurate the center must be is set by the depth the size implies.
    /// The first pass sizes the atom from the (shallow) center Newton just produced, which is
    /// plenty to pick the destination depth; the second re-solves the center at that depth's
    /// precision and re-sizes from it. Returns the refined center and the framing depth (`None`
    /// when the family has no size estimate).
    fn newton_raphson_target(
        &self,
        cx0: fractadyne_core::BigFloat,
        cy0: fractadyne_core::BigFloat,
        period: u32,
        formula: u32,
    ) -> (fractadyne_core::BigFloat, fractadyne_core::BigFloat, Option<f64>) {
        // Ceiling on `period × precision-bits` for the synchronous deep refine. The refine runs a
        // few Newton steps of `period` bignum iterations each ON THE UI THREAD; past this budget
        // (roughly a second of blocking on a fast core) the jump is declined in favor of a plain
        // center snap, rather than freezing the window for a period-100k atom at thousands of
        // bits. (The right long-term fix is an off-thread refine with a spinner — TODO'd.)
        const NR_REFINE_MAX_BIT_ITERS: u64 = 400_000_000;
        let mut cx = cx0;
        let mut cy = cy0;
        let mut prec = self.viewport.precision;
        let mut target = None;
        for _ in 0..2 {
            let Some(atom) = fractadyne_core::nucleus_size(&cx, &cy, period, formula, prec) else {
                break;
            };
            let t = Self::atom_frame_log2mag(atom.log2_size);
            if !t.is_finite() || t <= 0.0 {
                break;
            }
            // Guard bits above the destination depth so the center is exact *within* the frame.
            let need = fractadyne_core::precision_for_octaves(t as u64) + 64;
            if need <= prec {
                target = Some(t); // already precise enough for this depth — jump is safe
                break;
            }
            if (period as u64).saturating_mul(need as u64) > NR_REFINE_MAX_BIT_ITERS {
                break; // refine too costly for a synchronous call — recenter only, keep the depth
            }
            prec = need;
            let Some((rx, ry)) = fractadyne_core::refine_nucleus(&cx, &cy, period, formula, prec)
            else {
                break; // no refined center → no jump: an unrefined center at the atom's own
                       // scale would land the view on empty space
            };
            cx = rx;
            cy = ry;
            target = Some(t); // center now refined for this depth — jump is safe
        }
        (cx, cy, target)
    }

    /// Shared tail of a navigation jump: stop any glide, drop cached references, push history.
    fn finish_nav_jump(&mut self) {
        self.pointer.zoom_vel = 0.0;
        self.invalidate_refs();
        self.record_nav();
    }

    /// Newton-solve a parameterized feature near the current view center and snap onto its exact
    /// center (arbitrary precision): a Misiurewicz `(k,p)` branch/spiral center, or the nearest
    /// minibrot nucleus. Seeded from where you're looking, so it finds the feature you're near.
    /// Mandelbrot only. Driven by the Go-to dialog's feature finder.
    fn goto_feature(&mut self, ctx: &egui::Context) {
        if self.fractal.formula_id() != 0 {
            self.goto.msg = Some("Feature finding is Mandelbrot-only.".into());
            return;
        }
        let mag = self.viewport.magnification();
        let cur_l2 = self.viewport.log2_magnification();
        let center = [self.viewport.center_x.clone(), self.viewport.center_y.clone()];
        // A Newton-Raphson zoom target, when one is derivable (quadratic families only).
        let mut zoom_to: Option<f64> = None;
        let (found, label) = match self.goto.feat_kind {
            FeatureKind::Minibrot => {
                let max_period = self
                    .viewport
                    .recommended_max_iter(self.render_cfg.max_iter)
                    .clamp(1_000, 100_000);
                match fractadyne_core::find_nucleus(&center, mag, 0, max_period) {
                    Some(n) => {
                        let period = n.period;
                        let (cx, cy, t) = self.newton_raphson_target(n.cx, n.cy, period, 0);
                        zoom_to = t.filter(|t| *t > cur_l2);
                        (Some((cx, cy)), format!("period-{period} minibrot"))
                    }
                    None => (None, String::new()),
                }
            }
            FeatureKind::Misiurewicz => {
                let k = self.goto.feat_k.trim().parse::<u32>().ok().filter(|&v| v > 0);
                let p = self.goto.feat_p.trim().parse::<u32>().ok().filter(|&v| v > 0);
                let (Some(k), Some(p)) = (k, p) else {
                    self.goto.msg = Some("Enter a preperiod and period (positive integers).".into());
                    return;
                };
                match fractadyne_core::find_misiurewicz(&center, k, p, mag, 0) {
                    Some(m) => {
                        // The multiplier λ of the cycle the point lands on: |λ| is the ZOOM
                        // PERIOD (the view repeats every log₂|λ| octaves) and arg λ the twist
                        // per repeat. The numbers that say what diving here will look like.
                        let lam = fractadyne_core::misiurewicz_multiplier(
                            &m.cx,
                            &m.cy,
                            m.preperiod,
                            m.period,
                            0,
                            self.viewport.precision,
                        );
                        let label = match lam {
                            Some(l) => format!(
                                "Misiurewicz ({},{}) — repeats every {:.2} octaves, twist {:.1}°",
                                m.preperiod,
                                m.period,
                                l.log2_abs,
                                l.arg.to_degrees()
                            ),
                            None => format!("Misiurewicz ({},{})", m.preperiod, m.period),
                        };
                        (Some((m.cx, m.cy)), label)
                    }
                    None => (None, format!("Misiurewicz ({k},{p})")),
                }
            }
        };
        match found {
            Some((cx, cy)) => {
                let l2 = zoom_to.unwrap_or(cur_l2);
                self.viewport.set_center_log2mag(cx, cy, l2);
                self.finish_nav_jump();
                self.goto.open = false;
                self.set_toast(
                    match zoom_to {
                        Some(t) => format!("Zoomed to the {label} — {}×", fmt_zoom_field(t)),
                        None => format!("Snapped to {label} center"),
                    },
                    ctx,
                );
            }
            None => {
                self.goto.msg = Some(match self.goto.feat_kind {
                    FeatureKind::Minibrot => {
                        "No minibrot center found near the view — zoom closer to one.".to_string()
                    }
                    FeatureKind::Misiurewicz => {
                        format!("No {label} point converged near the view — navigate closer, or try different k/p.")
                    }
                });
            }
        }
    }

    /// Render the static home-view thumbnail for the minimap (fixed complex region), as
    /// an egui image. Cheap (small, direct path); only called when the thumbnail key
    /// changes. Returns `None` if the GPU render fails.
    fn render_minimap_image(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
    ) -> Option<egui::ColorImage> {
        let mut vp = Viewport::new(MINIMAP_TW as f64, MINIMAP_TH as f64);
        vp.center_x = fractadyne_core::BigFloat::from_f64(MINIMAP_CX, 64);
        vp.center_y = fractadyne_core::BigFloat::from_f64(MINIMAP_CY, 64);
        vp.units_per_pixel = fractadyne_core::FloatExp::from_f64((2.0 * MINIMAP_HX) / MINIMAP_TW as f64);
        vp.precision = 64;
        let mut req = self.current_export_request_for(&vp, false);
        req.width = MINIMAP_TW;
        req.height = MINIMAP_TH;
        req.ss = 1;
        req.max_iter = req.max_iter.clamp(200, 600);
        let progress = std::sync::atomic::AtomicU32::new(0);
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let res = fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel).ok()?;
        // Linear RGBA f32 → sRGB u8 (approx gamma) for display.
        let n = (res.width * res.height) as usize;
        let mut pixels = Vec::with_capacity(n * 4);
        for i in 0..n {
            for k in 0..3 {
                let c = res.pixels[i * 4 + k].clamp(0.0, 1.0);
                pixels.push((c.powf(1.0 / 2.2) * 255.0 + 0.5) as u8);
            }
            pixels.push(255);
        }
        Some(egui::ColorImage::from_rgba_unmultiplied(
            [res.width as usize, res.height as usize],
            &pixels,
        ))
    }

    /// Refresh the cached minimap thumbnail if its key (formula / palette / method)
    /// changed. No-op when the minimap is hidden.
    fn update_minimap(
        &mut self,
        ctx: &egui::Context,
        gpu: &Option<(eframe::wgpu::Device, eframe::wgpu::Queue)>,
    ) {
        // Shown for single Mandelbrot-family views and in dual view (the left panel is the
        // Mandelbrot map); hidden only for a single Julia view, where a Mandelbrot overview
        // wouldn't correspond to the shown set.
        if !self.dialogs.minimap || (self.julia_mode && !self.dual) {
            return;
        }
        // Key includes the palette identity (preset index or a sentinel) and a revision so
        // the thumbnail refreshes when the gradient / duotone colors change.
        let duo_hash = self
            .coloring
            .duotone_lo
            .iter()
            .chain(&self.coloring.duotone_hi)
            .fold(0u32, |a, &c| a.wrapping_mul(16_777_619) ^ c.to_bits());
        let (pal_idx, pal_rev) = if self.coloring.use_binary {
            (usize::MAX - 2, duo_hash)
        } else if self.coloring.use_duotone {
            (usize::MAX - 1, duo_hash)
        } else if self.coloring.use_custom_palette {
            (usize::MAX, self.coloring.palette_rev)
        } else {
            (self.coloring.palette_idx, 0)
        };
        let key = (self.fractal.formula_id(), pal_idx, self.coloring.color_method.to_u32(), pal_rev);
        if self.minimap_key == Some(key) && self.minimap_tex.is_some() {
            return;
        }
        if let Some((dev, q)) = gpu {
            if let Some(img) = self.render_minimap_image(dev, q) {
                let tex = ctx.load_texture("fractadyne.minimap", img, egui::TextureOptions::LINEAR);
                self.minimap_tex = Some(tex);
                self.minimap_key = Some(key);
            }
        }
    }



    /// The custom-gradient editor window: live gradient preview, per-stop color + position
    /// controls, add/remove, and seed-from-preset. Edits bump `palette_rev`.
    fn palette_editor_window(&mut self, ctx: &egui::Context) {
        if !self.coloring.palette_editor_open {
            return;
        }
        let mut open = self.coloring.palette_editor_open;
        let mut changed = false;
        egui::Window::new("Gradient editor")
            .open(&mut open)
            .resizable(false)
            .default_width(340.0)
            .show(ctx, |ui| {
                // Live gradient preview bar.
                let (packed, n) = self.pack_custom();
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(ui.available_width(), 26.0), egui::Sense::hover());
                let pr = ui.painter_at(rect);
                let steps = rect.width().ceil().max(1.0) as usize;
                for s in 0..steps {
                    let t = s as f32 / steps as f32;
                    let x = rect.min.x + t * rect.width();
                    let col = sample_stops(&packed, n, t);
                    pr.line_segment(
                        [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                        egui::Stroke::new(1.5_f32, col),
                    );
                }
                pr.rect_stroke(
                    rect,
                    2.0,
                    egui::Stroke::new(1.0_f32, BRAND_ACCENT),
                    egui::StrokeKind::Inside,
                );
                ui.add_space(6.0);

                // Per-stop rows (color + position + remove).
                let mut remove: Option<usize> = None;
                let count = self.coloring.custom_palette.len();
                for i in 0..count {
                    ui.horizontal(|ui| {
                        let mut rgb = [
                            self.coloring.custom_palette[i][1],
                            self.coloring.custom_palette[i][2],
                            self.coloring.custom_palette[i][3],
                        ];
                        if ui.color_edit_button_rgb(&mut rgb).changed() {
                            self.coloring.custom_palette[i][1] = rgb[0];
                            self.coloring.custom_palette[i][2] = rgb[1];
                            self.coloring.custom_palette[i][3] = rgb[2];
                            changed = true;
                        }
                        let mut pos = self.coloring.custom_palette[i][0];
                        if ui
                            .add(egui::Slider::new(&mut pos, 0.0..=1.0).text("pos").fixed_decimals(3))
                            .changed()
                        {
                            self.coloring.custom_palette[i][0] = pos.clamp(0.0, 1.0);
                            changed = true;
                        }
                        if count > 2 && ui.button(crate::icons::CLOSE).on_hover_text("Remove stop").clicked() {
                            remove = Some(i);
                        }
                    });
                }
                if let Some(i) = remove {
                    self.coloring.custom_palette.remove(i);
                    changed = true;
                }

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if self.coloring.custom_palette.len() < fractadyne_color::MAX_STOPS
                        && ui.button(format!("{} Add stop", crate::icons::ADD)).clicked()
                    {
                        self.coloring.custom_palette.push([0.5, 1.0, 1.0, 1.0]);
                        changed = true;
                    }
                    ui.menu_button("Copy preset…", |ui| {
                        for (i, p) in fractadyne_color::PRESETS.iter().enumerate() {
                            if ui.button(p.name).clicked() {
                                self.coloring.custom_palette = self.preset_as_stops(i);
                                changed = true;
                                ui.close_menu();
                            }
                        }
                    });
                    ui.toggle_value(&mut self.coloring.paste_open, "Paste…")
                        .on_hover_text("Import a palette from hex colours or 0–255 RGB triples");
                });

                // Paste-a-palette. The cheapest possible bridge to the existing palette cultures:
                // no file format to agree on, no dialog, and it covers "I found a palette on the
                // web" as well as pasting the body of a Fractint/KF `.map`.
                if self.coloring.paste_open {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(
                            "Paste hex colours (#ff8800) or 0–255 triples (255 136 0), separated \
                             by commas, spaces or new lines:",
                        )
                        .weak()
                        .small(),
                    );
                    ui.add(
                        egui::TextEdit::multiline(&mut self.coloring.paste_text)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY)
                            .hint_text("#000000, #8b1a1a, #ff8800, #ffe6b3"),
                    );
                    ui.horizontal(|ui| {
                        if ui.button("Apply").clicked() {
                            match fractadyne_color::parse_palette_text(&self.coloring.paste_text) {
                                Ok(colors) => {
                                    let got = colors.len();
                                    let used = fractadyne_color::resample_colors(
                                        &colors,
                                        fractadyne_color::MAX_STOPS,
                                    );
                                    // Spread the imported colours evenly; a single colour becomes
                                    // one stop at 0 rather than dividing by zero.
                                    let n = used.len();
                                    self.coloring.custom_palette = used
                                        .iter()
                                        .enumerate()
                                        .map(|(i, c)| {
                                            let pos = if n > 1 {
                                                i as f32 / (n - 1) as f32
                                            } else {
                                                0.0
                                            };
                                            [pos, c[0], c[1], c[2]]
                                        })
                                        .collect();
                                    self.coloring.paste_msg = Some(if got > n {
                                        format!(
                                            "Imported {n} of {got} colours — the gradient carries \
                                             {} stops, sampled evenly across your list.",
                                            fractadyne_color::MAX_STOPS
                                        )
                                    } else {
                                        format!("Imported {n} colours.")
                                    });
                                    changed = true;
                                }
                                Err(e) => self.coloring.paste_msg = Some(format!("Couldn't read that: {e}")),
                            }
                        }
                        if ui.button("Clear").clicked() {
                            self.coloring.paste_text.clear();
                            self.coloring.paste_msg = None;
                        }
                    });
                    if let Some(m) = &self.coloring.paste_msg {
                        ui.label(egui::RichText::new(m).weak().small());
                    }
                }
                ui.label(
                    egui::RichText::new(format!(
                        "{}/{} stops · positions may overlap; they're sorted automatically.",
                        self.coloring.custom_palette.len(),
                        fractadyne_color::MAX_STOPS
                    ))
                    .weak()
                    .small(),
                );
            });
        if changed {
            self.coloring.palette_rev = self.coloring.palette_rev.wrapping_add(1);
            self.coloring.use_custom_palette = true;
        }
        self.coloring.palette_editor_open = open;
    }

    /// Jump to a Mandelbrot location (full-precision center strings + magnification),
    /// e.g. a famous-locations entry. Switches to Mandelbrot, single view.
    fn goto_location(&mut self, cx: &str, cy: &str, mag: f64, name: &str, ctx: &egui::Context) {
        let (Some(x), Some(y)) =
            (fractadyne_core::parse_bf(cx), fractadyne_core::parse_bf(cy))
        else {
            return;
        };
        self.fractal = FractalKind::Mandelbrot;
        self.julia_mode = false;
        self.viewport.set_center_mag(x, y, mag.max(1.0));
        self.viewport.precision = fractadyne_core::precision_for_magnification(mag);
        self.pointer.zoom_vel = 0.0;
        self.invalidate_refs();
        self.record_nav();
        self.set_toast(format!("{name} · {}×", fmt_zoom(mag)), ctx);
    }

    /// Jump to a random interesting location: find a point on the set boundary by
    /// bisecting between an interior anchor and a random exterior direction, then zoom in
    /// a random amount. Boundary points are always detail-rich.
    fn random_location(&mut self, ctx: &egui::Context) {
        let mut s = (ctx.input(|i| i.time).to_bits() ^ 0x9E37_79B9_7F4A_7C15) | 1;
        let mut rnd = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            (s >> 11) as f64 / ((1u64 << 53) as f64)
        };
        let theta = rnd() * std::f64::consts::TAU;
        // Interior anchor (inside the main cardioid) → exterior point along θ.
        let (mut ix, mut iy) = (-0.5_f64, 0.0_f64);
        let (mut ox, mut oy) = (ix + 3.0 * theta.cos(), iy + 3.0 * theta.sin());
        for _ in 0..64 {
            let (mx, my) = ((ix + ox) * 0.5, (iy + oy) * 0.5);
            if mandel_escapes(mx, my, 3000).is_some() {
                ox = mx;
                oy = my;
            } else {
                ix = mx;
                iy = my;
            }
        }
        let (cx, cy) = ((ix + ox) * 0.5, (iy + oy) * 0.5);
        let mag = 10f64.powf(2.0 + rnd() * 4.0); // 1e2 .. 1e6
        self.fractal = FractalKind::Mandelbrot;
        self.julia_mode = false;
        self.viewport.set_center_mag(
            fractadyne_core::BigFloat::from_f64(cx, 64),
            fractadyne_core::BigFloat::from_f64(cy, 64),
            mag,
        );
        self.viewport.precision = fractadyne_core::precision_for_magnification(mag);
        self.pointer.zoom_vel = 0.0;
        self.invalidate_refs();
        self.record_nav();
        self.set_toast(format!("Random location · {}×", fmt_zoom(mag)), ctx);
    }

    /// The Help window: a left-hand table of contents + a scrollable content pane.
    /// The "Faster deep zoom" dialog: what the accelerated build is, and where to get it.
    ///
    /// Two states, because the honest thing to show depends on which binary is running, and that
    /// is taken from `fractadyne_core::available_backends()` -- a compile-time fact of THIS binary
    /// rather than a flag or a setting that could disagree with it.
    fn accelerated_window(&mut self, ctx: &egui::Context) {
        if !self.dialogs.accelerated_open {
            return;
        }
        let have_it = fractadyne_core::available_backends().len() > 1;
        let ver = crate::sysinfo::version_string();
        let asset = accelerated_asset_url(&ver);
        let releases = "https://github.com/WindySnowOwl/fractadyne/releases";

        let mut open = self.dialogs.accelerated_open;
        egui::Window::new("Faster deep zoom")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(520.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;
                if have_it {
                    ui.heading("You are running the accelerated build");
                    ui.label(
                        "Deep-zoom reference orbits are being computed with MPFR/GMP, which is \
                         2.5-6.4x faster than the standard build at that step - the pause before \
                         a deep view starts resolving.",
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "Arithmetic in use: {}",
                            fractadyne_core::backend_status_line()
                        ))
                        .monospace()
                        .small(),
                    );
                } else {
                    ui.heading("An optional build is 2.5-6.4x faster at depth");
                    ui.label(
                        "Deep zoom spends much of its time computing reference orbits on the CPU \
                         - the pause before a deep view starts resolving. An optional build does \
                         that with the MPFR/GMP libraries instead, which is 2.5 to 6.4 times \
                         faster, and more so the deeper you go. Everything else is the same.",
                    );
                    ui.separator();
                    ui.label(
                        "The images are BYTE-IDENTICAL - it is the same mathematics computed by a \
                         faster library, checked across every fractal formula and the whole \
                         deep-zoom comparison corpus.",
                    );
                    ui.label(
                        "Your settings, saved session and locations are shared between the two, \
                         so you can switch freely and nothing needs importing.",
                    );
                    ui.label(
                        egui::RichText::new(
                            "It is a separate download because the libraries it uses cannot be \
                             built with the compiler the standard Windows build uses, and they \
                             carry a different licence (GNU LGPL v3) from Fractadyne's own.",
                        )
                        .small(),
                    );
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui
                            .button("Download for this version")
                            .on_hover_text(&asset)
                            .clicked()
                        {
                            ctx.open_url(egui::OpenUrl::new_tab(&asset));
                        }
                        if ui
                            .button("All releases")
                            .on_hover_text(
                                "If the direct link 404s, this version has no accelerated build \
                                 yet - pick the nearest one here.",
                            )
                            .clicked()
                        {
                            ctx.open_url(egui::OpenUrl::new_tab(releases));
                        }
                    });
                    ui.label(
                        egui::RichText::new(
                            "Extract it and run fractadyne.exe from that folder, keeping the .dll \
                             files beside it.",
                        )
                        .small(),
                    );
                }
            });
        self.dialogs.accelerated_open = open;
    }

    fn help_window(&mut self, ctx: &egui::Context) {
        if !self.dialogs.help_open {
            return;
        }
        const SECTIONS: [&str; 11] = [
            "Overview",
            "Navigation",
            "Coloring & options",
            "Fractals",
            "How it works",
            "Command line",
            "Shortcuts",
            "Recommended hardware",
            "Acknowledgments",
            "Licenses",
            "About",
        ];
        let mut open = self.dialogs.help_open;
        // Cap the size to the screen so the content ScrollArea scrolls (rather than the window
        // growing to fit) and the window can't be resized past the screen edge (which pushed
        // the title-bar close button off-screen).
        let max_h = (ctx.screen_rect().height() - 80.0).max(360.0);
        let max_w = (ctx.screen_rect().width() - 40.0).max(480.0);
        egui::Window::new("Fractadyne Help")
            .open(&mut open)
            .default_size([800.0, 560.0])
            .min_width(480.0) // keep room for the content beside the fixed-width contents list
            .min_height(300.0)
            .max_width(max_w)
            .max_height(max_h)
            .constrain(true) // keep the whole window on-screen
            .resizable(true)
            .show(ctx, |ui| {
                // Manual two-column split (fixed contents list + scrollable content pane).
                // Explicit widths make the content both wrap AND fill the window, so the
                // window's resize grip stays at the true bottom-right — nested SidePanel/
                // CentralPanel mis-reported the width and stranded the grip beside the list.
                let toc_w = 165.0_f32;
                let avail = ui.available_size();
                ui.horizontal_top(|ui| {
                    // Contents list (fixed width).
                    ui.allocate_ui_with_layout(
                        egui::vec2(toc_w, avail.y),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_min_width(toc_w);
                            ui.set_max_width(toc_w);
                            ui.add_space(4.0);
                            for (i, name) in SECTIONS.iter().enumerate() {
                                ui.selectable_value(&mut self.dialogs.help_section, i, *name);
                            }
                        },
                    );
                    ui.separator();
                    ui.add_space(8.0); // left inset so the separator doesn't touch the text
                    let content_w = ui.available_width().max(240.0);
                    ui.allocate_ui_with_layout(
                        egui::vec2(content_w, avail.y),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_min_width(content_w);
                            ui.set_max_width(content_w);
                            // Solid, only-when-needed scrollbar (not floating/hover-only).
                            ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .scroll_bar_visibility(
                                    egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                                )
                                .show(ui, |ui| {
                                    ui.set_max_width(content_w - 18.0); // leave room for the bar
                                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                                    match self.dialogs.help_section {
                                        0 => help_overview(ui),
                                        1 => help_navigation(ui),
                                        2 => help_options(ui),
                                        3 => help_fractals(ui),
                                        4 => help_methodology(ui),
                                        5 => help_command_line(ui),
                                        6 => help_shortcuts(ui),
                                        7 => help_hardware(ui),
                                        8 => help_acknowledgments(ui),
                                        9 => help_licenses(ui),
                                        _ => help_about(ui),
                                    }
                                });
                        },
                    );
                });
            });
        self.dialogs.help_open = open;
    }

    /// Load a Kalles Fraktaler `.kfr` location file and jump to it. Defensive: bounds the
    /// file size and delegates to the hardened `parse_kfr`. (KF's zoom and ours are both
    /// linear magnification from the home view — close enough that the location lands at
    /// essentially the right place and scale.)
    fn load_kfr_file(&mut self, path: &std::path::Path) -> Result<String, crate::error::AppError> {
        use crate::error::AppError;
        let meta = std::fs::metadata(path)?;
        if meta.len() > 4_000_000 {
            return Err(AppError::Message("file too large (not a .kfr location?)".into()));
        }
        let text = std::fs::read_to_string(path)?;
        let v = fractadyne_core::parse_kfr(&text)
            .ok_or_else(|| AppError::Parse("not a valid .kfr location (need Re / Im / Zoom)".into()))?;
        let zoom = v.zoom;
        self.fractal = FractalKind::Mandelbrot;
        self.julia_mode = false;
        if let Some(it) = v.iterations {
            self.render_cfg.max_iter = it.clamp(64, 50_000);
            self.render_cfg.auto_iter = false;
        }
        self.viewport.set_center_mag(v.cx, v.cy, zoom.max(1.0));
        self.viewport.precision = fractadyne_core::precision_for_magnification(zoom);
        self.pointer.zoom_vel = 0.0;
        self.invalidate_refs();
        self.record_nav();
        Ok(format!("Imported .kfr location @ {}×", fmt_zoom(zoom)))
    }

    /// Load an **Imagina TEXT location** (`--import-imagina`). Mirrors `load_kfr_file`.
    ///
    /// The BINARY `.im` form is refused by its magic rather than parsed: its payload needs `HRReal`'s
    /// layout and GMP `mpf` raw streams, which are not documented in the source available to read, and
    /// a guessed binary parser is worse than none — it imports a plausible wrong location silently.
    /// Telling the user to re-save as text is the honest outcome.
    fn load_imagina_file(&mut self, path: &std::path::Path) -> Result<String, crate::error::AppError> {
        use crate::error::AppError;
        let meta = std::fs::metadata(path)?;
        if meta.len() > 4_000_000 {
            return Err(AppError::Message("file too large (not an Imagina location?)".into()));
        }
        let bytes = std::fs::read(path)?;
        if bytes.starts_with(&fractadyne_core::IMAGINA_BINARY_MAGIC) {
            return Err(AppError::Message(
                "this is a BINARY Imagina .im file, which is not supported — re-save it from Imagina as a text location (File type: Imagina text) and import that"
                    .into(),
            ));
        }
        let text = String::from_utf8_lossy(&bytes);
        let v = fractadyne_core::parse_imagina_text(&text).ok_or_else(|| {
            AppError::Parse("not a valid Imagina text location (need Location Size / Re / Im)".into())
        })?;
        let zoom = v.zoom;
        self.fractal = FractalKind::Mandelbrot;
        self.julia_mode = false;
        if let Some(it) = v.iterations {
            self.render_cfg.max_iter = it.clamp(64, MAX_ITER_LIMIT);
            self.render_cfg.auto_iter = false;
        }
        self.viewport.set_center_mag(v.cx, v.cy, zoom.max(1.0));
        self.viewport.precision = fractadyne_core::precision_for_magnification(zoom);
        self.pointer.zoom_vel = 0.0;
        self.invalidate_refs();
        self.record_nav();
        Ok(format!("Imported Imagina location @ {}×", fmt_zoom(zoom)))
    }

    /// Open the Share-location dialog, pre-filled with the current view as `.fdn` text.
    fn open_share(&mut self) {
        self.share.text = self.view_metadata();
        self.share.msg = None;
        self.share.open = true;
    }

    /// Apply the Share dialog's text as a location (hardened: bounded, allow-list parse via
    /// `load_view_metadata`, every field validated/clamped — no paths or code).
    fn apply_share_text(&mut self, ctx: &egui::Context) {
        let t = self.share.text.trim();
        if t.is_empty() || t.len() > SHARE_MAX {
            self.share.msg = Some("Nothing to load (or text too large).".into());
            return;
        }
        // Must look like a Fractadyne location (has our app tag or a center field).
        if meta_get(t, "app") != "Fractadyne" && meta_get(t, "center_re").is_empty() {
            self.share.msg = Some("Not a Fractadyne location.".into());
            return;
        }
        let t = t.to_string();
        let report = self.load_view_metadata(&t); // performs the jump + records history
        let zoom = fmt_zoom_log2(self.viewport.log2_magnification());
        match report.note() {
            None => {
                self.set_toast(format!("Loaded location @ {zoom}×"), ctx);
                self.share.open = false;
            }
            // Keep the dialog open and surface the report rather than silently jumping.
            Some(n) => {
                self.share.msg = Some(format!("Loaded @ {zoom}× — {n}."));
            }
        }
    }

    /// Compact, PII-free system-info block for issue reports (version, OS, CPU, GPU, VRAM).
    fn system_info_block(&self) -> String {
        let si = &self.sysinfo;
        let cache = match (si.l2_kb, si.l3_kb) {
            (0, 0) => "—".to_string(),
            (l2, 0) => format!("L2 {l2} KB"),
            (l2, l3) => format!("L2 {l2} KB / L3 {l3} KB"),
        };
        let vram = if si.vram_mb > 0 { format!("{} MB", si.vram_mb) } else { "unknown".to_string() };
        format!(
            "Fractadyne v{}\n{}\nOS:   {} / {}\nCPU:  {} ({} physical / {} logical, {})\nGPU:  {} ({})\nVRAM: {}\nBignum: {} (built with: {})\n",
            version_string(),
            now_utc_string(),
            std::env::consts::OS,
            std::env::consts::ARCH,
            if si.cpu.is_empty() { "unknown" } else { si.cpu.as_str() },
            si.physical,
            si.logical,
            cache,
            self.gpu_name,
            self.gpu_backend,
            vram,
            // Both halves matter in a bug report: which backend this session actually iterated in,
            // and which ones the build could have used. A deep-zoom report without them cannot be
            // reproduced once more than one backend ships.
            fractadyne_core::backend_status_line(),
            fractadyne_core::built_in_backends(),
        )
    }

    /// Email subject line for the issue report — the issue type, so it triages in the inbox.
    fn report_subject(&self) -> String {
        format!("Fractadyne issue: {}", self.report.kind.label())
    }

    /// Assemble the full issue-report text from the dialog state + live diagnostics — exactly what
    /// the preview shows and what Copy/Save/Email use. Nothing is transmitted here.
    fn build_report(&self) -> String {
        let mut s = String::new();
        s.push_str("Fractadyne issue report\n");
        s.push_str(&format!("To: {REPORT_EMAIL}\n"));
        s.push_str(&format!("Type: {}\n", self.report.kind.label()));
        if self.report.severity != Severity::Unspecified {
            s.push_str(&format!("Severity: {}\n", self.report.severity.label()));
        }
        if self.report.repro != Repro::Unspecified {
            s.push_str(&format!("Reproducibility: {}\n", self.report.repro.label()));
        }
        s.push('\n');
        s.push_str("== Description ==\n");
        let d = self.report.description.trim();
        s.push_str(if d.is_empty() { "(none provided)" } else { d });
        s.push_str("\n\n");
        if self.report.include_sysinfo {
            s.push_str("== System ==\n");
            s.push_str(&self.system_info_block());
            s.push('\n');
        }
        if self.report.include_location {
            s.push_str("== Current location (.fdn) ==\n");
            s.push_str(&self.view_metadata());
            s.push('\n');
        }
        if self.report.include_crash {
            if let Some((name, body)) = crate::diag::latest_crash() {
                s.push_str(&format!("== Latest crash report ({name}) ==\n"));
                s.push_str(body.trim_end());
                s.push_str("\n\n");
            }
        }
        // Before the log, so a reader meets the machine-validated verdict before the raw tail.
        if self.report.include_test {
            if let Some(block) = self.test_result_block() {
                s.push_str("== Diagnostics test result ==\n");
                s.push_str(block.trim_end());
                s.push_str("\n\n");
            }
        }
        if self.report.include_log {
            if let Some(log) = crate::diag::recent_log(48 * 1024) {
                s.push_str("== Recent log (tail) ==\n");
                s.push_str(log.trim_end());
                s.push('\n');
            }
        }
        s
    }

    /// Save the Share dialog's text to a `.fdn` file.
    fn save_share_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Fractadyne location", &["fdn"])
            .set_directory(self.dialog_dir_default())
            .set_file_name("location.fdn")
            .save_file()
        {
            self.remember_dir(&path);
            match std::fs::write(&path, self.share.text.as_bytes()) {
                Ok(()) => self.share.msg = Some("Saved.".into()),
                Err(e) => self.share.msg = Some(format!("Save failed: {e}")),
            }
        }
    }

    /// Load a `.fdn` file into the Share dialog's text box (size-bounded; not auto-applied,
    /// so the user can review before jumping).
    fn load_share_file(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Fractadyne location", &["fdn"])
            .set_directory(self.dialog_dir_default())
            .pick_file()
        else {
            return;
        };
        self.remember_dir(&path);
        match std::fs::metadata(&path) {
            Ok(m) if (m.len() as usize) <= SHARE_MAX => match std::fs::read_to_string(&path) {
                Ok(t) => {
                    self.share.text = t;
                    self.share.msg = Some("Loaded into the box — review, then Apply.".into());
                }
                Err(e) => self.share.msg = Some(format!("Read failed: {e}")),
            },
            Ok(_) => self.share.msg = Some("File too large (not a .fdn location?).".into()),
            Err(e) => self.share.msg = Some(format!("Read failed: {e}")),
        }
    }

    /// File-dialog import of a Kalles Fraktaler `.kfr` location.
    fn import_kfr(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Kalles Fraktaler location", &["kfr"])
            .set_directory(self.dialog_dir_default())
            .pick_file()
        else {
            return;
        };
        self.remember_dir(&path);
        match self.load_kfr_file(&path) {
            Ok(m) => self.set_toast(m, ctx),
            Err(e) => self.set_toast(format!("Import failed: {e}"), ctx),
        }
    }

    /// Reset the current view to the fractal's default (both panels in dual view).
    fn reset_view(&mut self) {
        self.viewport.reset();
        let (cx, cy) = self.fractal.default_center();
        self.viewport.center_x = fractadyne_core::BigFloat::from_f64(cx, 64);
        self.viewport.center_y = fractadyne_core::BigFloat::from_f64(cy, 64);
        if self.dual {
            self.julia_viewport.reset();
            self.julia_viewport.center_x = fractadyne_core::BigFloat::from_f64(0.0, 64);
            self.julia_viewport.center_y = fractadyne_core::BigFloat::from_f64(0.0, 64);
        }
        self.pointer.zoom_vel = 0.0;
        self.invalidate_refs();
        self.record_nav();
    }

    /// Begin a smooth zoom-out back to the home view. If already at (or near) home,
    /// just snaps via `reset_view`. `now` is the current app time (`ctx.input.time`).
    fn zoom_home(&mut self, now: f64) {
        let m_logmag = self.viewport.magnification().max(1.0).ln();
        let j_logmag = if self.dual {
            self.julia_viewport.magnification().max(1.0).ln()
        } else {
            0.0
        };
        let deepest = m_logmag.max(j_logmag);
        if deepest < 0.02 {
            self.reset_view();
            return;
        }
        let duration =
            (deepest * HOME_SECONDS_PER_LOGMAG).clamp(HOME_MIN_SECONDS, HOME_MAX_SECONDS);
        self.home_anim = Some(HomeAnim {
            start_time: now,
            duration,
            m_start_center: (self.viewport.center_x.clone(), self.viewport.center_y.clone()),
            m_start_logmag: m_logmag,
            j_start_center: (
                self.julia_viewport.center_x.clone(),
                self.julia_viewport.center_y.clone(),
            ),
            j_start_logmag: j_logmag,
            dual: self.dual,
        });
        self.pointer.zoom_vel = 0.0;
    }

    /// Advance the active zoom-home animation by one frame. Returns true while it is
    /// still running (so the caller can keep requesting repaints).
    fn advance_home_anim(&mut self, ctx: &egui::Context) -> bool {
        let Some(anim) = self.home_anim.take() else {
            return false;
        };
        // Let the user grab control: any pan/zoom input cancels the glide in place.
        let interrupted = ctx.input(|i| {
            i.pointer.primary_down()
                || i.key_down(egui::Key::Space)
                || i.smooth_scroll_delta.y != 0.0
        });
        if interrupted {
            return false;
        }
        let now = ctx.input(|i| i.time);
        let u = ((now - anim.start_time) / anim.duration).clamp(0.0, 1.0);
        if u >= 1.0 {
            self.reset_view(); // exact home (center + zoom), invalidates references
            return false;
        }
        // Ease in/out (smoothstep) on the remaining log-magnification.
        let e = u * u * (3.0 - 2.0 * u);
        let remain = 1.0 - e;
        self.viewport
            .home_lerp(self.fractal.default_center(), &anim.m_start_center, anim.m_start_logmag * remain);
        if anim.dual {
            self.julia_viewport
                .home_lerp((0.0, 0.0), &anim.j_start_center, anim.j_start_logmag * remain);
        }
        // Treat the glide as interaction so AA stays off and references aren't
        // recomputed every frame (rebasing covers the motion; quality on settle).
        self.pointer.settle_t = [now; 2];
        self.home_anim = Some(anim);
        true
    }

    // Autopilot (toggle_autopilot / autopilot_step / autopilot_pick_target) moved to autopilot.rs.

    /// Stops uploaded to the GPU: the morphing random gradient when in Random mode,
    /// otherwise the selected preset.
    fn active_stops(&self) -> ([[f32; 4]; fractadyne_color::MAX_STOPS], u32) {
        if self.anim.palette_anim == PaletteAnim::Random {
            self.anim.random_palette.current()
        } else if self.coloring.use_binary {
            // Flat exterior: a single stop of the `hi` color (interior uses `lo`).
            let mut out = [[0.0f32; 4]; fractadyne_color::MAX_STOPS];
            out[0] = [self.coloring.duotone_hi[0], self.coloring.duotone_hi[1], self.coloring.duotone_hi[2], 0.0];
            (out, 1)
        } else if self.coloring.use_duotone {
            // Smooth two-color ramp lo → hi → lo (seamless under cycling).
            let (lo, hi) = (self.coloring.duotone_lo, self.coloring.duotone_hi);
            let mut out = [[0.0f32; 4]; fractadyne_color::MAX_STOPS];
            out[0] = [lo[0], lo[1], lo[2], 0.0];
            out[1] = [hi[0], hi[1], hi[2], 0.5];
            out[2] = [lo[0], lo[1], lo[2], 1.0];
            (out, 3)
        } else if self.coloring.use_custom_palette {
            self.pack_custom()
        } else {
            fractadyne_color::PRESETS[self.coloring.palette_idx].packed()
        }
    }

    /// In-set (interior) color for the GPU. Binary/duotone use the chosen `lo` color so the
    /// set reads as one solid color; otherwise the default near-black.
    fn interior_color(&self) -> [f32; 4] {
        if self.coloring.use_binary || self.coloring.use_duotone {
            [self.coloring.duotone_lo[0], self.coloring.duotone_lo[1], self.coloring.duotone_lo[2], 1.0]
        } else {
            [0.02, 0.02, 0.03, 1.0]
        }
    }

    /// Pack the custom gradient into the GPU stop format `[r, g, b, pos]` (sorted by
    /// position, count clamped to `MAX_STOPS`). Falls back to a preset if empty.
    fn pack_custom(&self) -> ([[f32; 4]; fractadyne_color::MAX_STOPS], u32) {
        if self.coloring.custom_palette.is_empty() {
            return fractadyne_color::PRESETS[self.coloring.palette_idx].packed();
        }
        let mut stops = self.coloring.custom_palette.clone();
        stops.sort_by(|a, b| a[0].total_cmp(&b[0]));
        let n = stops.len().clamp(1, fractadyne_color::MAX_STOPS);
        let mut out = [[0.0f32; 4]; fractadyne_color::MAX_STOPS];
        for (i, slot) in out.iter_mut().enumerate() {
            let s = stops[i.min(n - 1)];
            *slot = [s[1], s[2], s[3], s[0]];
        }
        (out, n as u32)
    }

    /// The given preset's stops as editable `[pos, r, g, b]` rows (to seed the editor).
    fn preset_as_stops(&self, idx: usize) -> Vec<[f32; 4]> {
        fractadyne_color::PRESETS[idx.min(fractadyne_color::PRESETS.len() - 1)]
            .stops
            .iter()
            .map(|(pos, c)| [*pos, c[0], c[1], c[2]])
            .collect()
    }

    /// Advance the palette animation for this frame (offset shift, or random morph).
    fn advance_palette_anim(&mut self, ctx: &egui::Context) {
        let dt = (ctx.input(|i| i.stable_dt) as f64).clamp(0.0, 0.1) as f32;
        // Distance-estimate glow cycling — flows the contour bands (independent of the
        // palette animation; shares the Speed slider). Phase is in cycles, period 1.
        if self.effects.de && self.effects.de_anim && self.anim.palette_anim_speed > 0.0 {
            self.effects.de_phase = (self.effects.de_phase + self.anim.palette_anim_speed * dt).rem_euclid(1.0);
            self.schedule_repaint(ctx);
        }
        // Rotate the relief light direction (cheap — it's a color-pass param).
        if self.effects.light && self.effects.light_anim && self.anim.palette_anim_speed > 0.0 {
            self.effects.light_angle = (self.effects.light_angle
                + self.anim.palette_anim_speed * dt * std::f32::consts::TAU)
                .rem_euclid(std::f32::consts::TAU);
            self.schedule_repaint(ctx);
        }
        if self.anim.palette_anim == PaletteAnim::Off || self.anim.palette_anim_speed <= 0.0 {
            return;
        }
        let step = self.anim.palette_anim_speed * dt;
        match self.anim.palette_anim {
            PaletteAnim::Forward => self.coloring.offset = (self.coloring.offset + step).fract(),
            PaletteAnim::Reverse => self.coloring.offset = (self.coloring.offset - step).rem_euclid(1.0),
            PaletteAnim::PingPong => {
                self.coloring.offset += self.anim.anim_dir * step;
                if self.coloring.offset >= 1.0 {
                    self.coloring.offset = 1.0;
                    self.anim.anim_dir = -1.0;
                } else if self.coloring.offset <= 0.0 {
                    self.coloring.offset = 0.0;
                    self.anim.anim_dir = 1.0;
                }
            }
            PaletteAnim::Random => self.anim.random_palette.advance(dt, self.anim.palette_anim_speed),
            PaletteAnim::Off => {}
        }
        self.schedule_repaint(ctx);
    }

    // start_benchmark / load_script / advance_playback / format_bench moved to scripting.rs.

    /// Advance the orbit racing-dot animation (position along the path + hue).
    fn advance_orbit_anim(&mut self, ctx: &egui::Context) {
        if !(self.anim.show_orbits && self.anim.orbit_anim) {
            return;
        }
        let dt = (ctx.input(|i| i.stable_dt) as f64).clamp(0.0, 0.1) as f32;
        self.anim.orbit_phase = (self.anim.orbit_phase + self.anim.orbit_anim_speed * dt) % 1.0e6;
        self.anim.orbit_hue = (self.anim.orbit_hue + 0.22 * dt).fract(); // ~4.5 s per color cycle
        self.schedule_repaint(ctx);
    }

    /// Zoom the main viewport about its center (factor < 1 zooms in).
    fn zoom_center(&mut self, factor: f64) {
        let (cx, cy) = (self.viewport.width_px * 0.5, self.viewport.height_px * 0.5);
        self.viewport.zoom_at(cx, cy, factor);
    }

    /// Click-to-zoom action (single view): recenter on a canvas point (`px`/`py` in device pixels)
    /// and dive in by `render_cfg.click_zoom_factor`, or back out by it when `out`. Reuses the
    /// box-zoom recenter idiom (pan the point to center, then scale via the bignum viewport, so it
    /// stays deep-zoom-correct). Records a nav step so each click is Backspace-undoable, and marks
    /// the view interacting so it settles to full quality afterward. `now` = `ctx.input(|i| i.time)`.
    fn click_zoom_at(&mut self, px: f64, py: f64, out: bool, now: f64) {
        let f = self.render_cfg.click_zoom_factor.max(1.01) as f64;
        let factor = if out { f } else { 1.0 / f }; // zoom_at: factor < 1 ⇒ zoom in
        let (w, h) = (self.viewport.width_px, self.viewport.height_px);
        self.viewport.pan_pixels(w * 0.5 - px, h * 0.5 - py);
        self.viewport.zoom_at(w * 0.5, h * 0.5, factor);
        self.pointer.zoom_vel = 0.0; // cancel any continuous-zoom glide so the jump lands clean
        self.pointer.settle_t[0] = now;
        self.record_nav();
    }

    /// Toggle the dual linked view, framing the Julia panel when turning it on.
    fn toggle_dual(&mut self) {
        if !self.fractal.supports_julia() {
            self.dual = false; // no Julia → no dual (guards any non-UI caller)
            return;
        }
        self.dual = !self.dual;
        if self.dual {
            self.julia_viewport.reset();
            self.julia_viewport.center_x = fractadyne_core::BigFloat::from_f64(0.0, 64);
            self.julia_viewport.center_y = fractadyne_core::BigFloat::from_f64(0.0, 64);
        }
        self.invalidate_refs();
    }

    /// `src` tag for a reading that came from a real GPU timestamp query. The wall-clock fallback
    /// uses a different tag, and the difference is load-bearing for `record_mode_rate` — see there.
    const SRC_GPU_ITERATE: &'static str = "gpu_iterate";

    /// Fold one measured iterate cost into a view's frame budget. Shared by BOTH measurement
    /// sources — the GPU timestamp readback and the wall-clock fallback — because the arithmetic
    /// must not differ between them: a fallback that walks the budget by different rules is a
    /// second controller to reason about, and this ledger already records what happens when two
    /// paths that should be identical drift apart (five `is_fe` sites, of which one was missed).
    ///
    /// `ms` is how long the dispatch of `steps` nominal steps took. Returns whether the budget
    /// moved. The caller owns the pairing: `steps` must be the count that `ms` actually priced.
    fn apply_iterate_measurement(&mut self, v: usize, ms: f64, steps: u64, src: &str) -> bool {
        // Zero means "nothing measured yet in this mode" — that is the one case the opening guess
        // is for. ⚠It used to be `.max(TDR_BOOTSTRAP_STEPS)`, which re-raised the budget to the
        // guess before EVERY step, so a converged-low budget was hoisted back up on each reading
        // and could never settle below `bootstrap × TDR_SHRINK_MAX`. That is the same
        // guess-as-a-floor bug as the clamp in `budget_step`, in a second place; see TDR_MIN_STEPS.
        // Record the rate BEFORE pricing: this reading describes the mode we are in now, and the
        // opening guess for the NEXT crossover is only as good as the worst rate we have kept.
        // Deliberately outside the `budget_step` early-return below — a reading that carries no
        // signal for the budget (an undersized dispatch) is still a true measurement of per-step
        // cost, and the pessimistic ones are exactly the ones worth keeping.
        //
        // ⚠GPU TIMESTAMPS ONLY. The wall-clock fallback prices a dispatch by `max_dt` — the frame
        // interval at which the queue finally stalls — which deliberately includes present/vsync
        // wait so the BUDGET errs toward shrinking. That conservatism is wrong for a RATE: it
        // understates steps-per-ms, and because `record_mode_rate` latches the minimum, one
        // wall-clock reading would cap this view's opening guess low for the rest of the session.
        // A device with no timestamps at all therefore records no rate and falls back to the
        // historical constant — the honest degradation, since we genuinely cannot measure per-step
        // cost there.
        if src == Self::SRC_GPU_ITERATE {
            self.perf.record_mode_rate(v, ms, steps);
        }
        let cur = render::budget_base(self.perf.fe_budget[v], self.perf.bootstrap_steps(v));
        // The arithmetic lives in `render::budget_step` as a pure function so the properties that
        // matter — a slow reading always shrinks, growth is bounded, the clamps hold — are pinned
        // by tests rather than by re-reading this block.
        let Some((next, ok)) = render::budget_step(cur, steps, ms, !self.render_cfg.auto_iter)
        else {
            if diag::trace_on("gpu") {
                diag::trace(
                    "gpu",
                    format!(
                        "view={v} {src}={ms:.1}ms IGNORED (steps={:.3e} < 0.7×budget)",
                        steps as f64
                    ),
                );
            }
            return false;
        };
        // Refuse to bank growth measured while this view's reference is being REBUILT — the frame
        // was priced against an orbit that is about to be replaced. See `budget_after_build_gate`.
        let building = self.recompute_rx[v].is_some();
        let (next, ok) = render::budget_after_build_gate(cur, next, ok, building);
        if building && next == cur {
            diag::trace(
                "gpu",
                format!("view={v} {src}={ms:.1}ms growth REFUSED — reference rebuild in flight"),
            );
        }
        // ⚠ALWAYS logged, not trace-gated: a reading in the watchdog band means this frame came
        // within roughly 2× of losing the device, and both 2026 field losses show only three or
        // four such frames between the first one and the end. If it happens in the field the log
        // must already say so — by the time anyone thinks to enable a trace, the run is over.
        if ms >= crate::tunables::cost().tdr_lethal_ms {
            // Shed this view's earned chunk-band licences along with the budget. The budget cut
            // already reaches the window (`budget_step.min(chunk_band_license(..))`), but only
            // while `budget_step` is the smaller term — and it floors at 256. The licence is the
            // other term, and nothing else here can lower it: the cliff rule that would runs only
            // on a pass the walk judged drained, which a saturated queue stops producing exactly
            // when this fires. Output-neutral (see `chunk_band_retreat`).
            render::chunk_band_retreat(&mut self.perf.chunk_bands[v.min(1)]);
            diag::log_line(
                "render",
                &format!(
                    "⚠LETHAL-BAND FRAME: view={v} {src}={ms:.0}ms (band ≥{:.0}ms) steps={:.3e} \
                     budget={:.3e} sa_skip={} res={}x{} — emergency retreat to {:.3e}",
                    crate::tunables::cost().tdr_lethal_ms,
                    steps as f64,
                    cur as f64,
                    self.perf.last_sa_skip_v[v.min(1)],
                    self.perf.last_res_v[v.min(1)][0],
                    self.perf.last_res_v[v.min(1)][1],
                    next as f64
                ),
            );
        }
        self.perf.last_iterate_ms[v] = ms;
        self.perf.fe_budget_ok[v] = ok;
        if diag::trace_on("gpu") {
            diag::trace(
                "gpu",
                format!(
                    "view={v} {src}={ms:.1}ms cur={:.3e} -> next={:.3e} ok={ok}",
                    cur as f64, next as f64
                ),
            );
        }
        let moved = next != self.perf.fe_budget[v];
        self.perf.fe_budget[v] = next;
        moved
    }

    /// WALL-CLOCK FALLBACK for the frame budget, used once `wall_fallback` trips (no GPU
    /// timestamps, or a sink that never delivers). Called from the frame-interval capture, which
    /// is the one point where the interval, `prev_real`, and `fe_steps_last` all describe the SAME
    /// frame — the controller runs earlier in `update` and would pair a time with the wrong step
    /// count, and `build_params` runs later and would pair it with the next frame's.
    ///
    /// Precedent: the AIMD motion-resolution controller has always sized the moving frame from the
    /// measured frame interval, needs no device feature, and cannot hang. This is the same signal
    /// applied to the settled budget. It reads HIGH — the interval also contains present, the CPU
    /// side, and the other view — so the budget settles a little low, which is the safe direction.
    /// A dispatch that fits inside a vsync cannot be distinguished from a much cheaper one, but
    /// that only means the climb keeps growing at `TDR_GROW_MAX` until a frame is genuinely slow,
    /// which is exactly the intended search.
    /// Live `(rebase, bla_skip)` from the last counter readback, or `(0, 0)` if none has landed.
    /// Both are stored biased by one so a genuine zero is distinguishable from "never published" —
    /// and a genuine zero is the interesting reading here, not a missing one.
    fn live_work_counters(&self, v: usize) -> (u64, u64) {
        let packed = self.perf.work_sink[v].load(std::sync::atomic::Ordering::SeqCst);
        if packed == 0 {
            return (0, 0);
        }
        ((packed >> 32).saturating_sub(1), (packed & 0xFFFF_FFFF).saturating_sub(1))
    }

    fn wall_clock_budget_tick(&mut self, dt_ms: f64) {
        if !self.perf.wall_fallback || !(dt_ms > 0.01) {
            return;
        }
        for v in 0..2 {
            // ⭐**A DISPATCH IS NOT PRICED BY THE INTERVAL IT WAS SUBMITTED IN.** Submission is
            // asynchronous: the frame that queues 2 s of GPU work returns from `update` in a
            // vsync-shaped ~17 ms, and the cost only surfaces when the queue backs up and stalls a
            // later acquire. With `desired_maximum_frame_latency: 1` that is the NEXT acquire, so
            // the cost shows up one interval late — which is exactly what `aa_probe` below already
            // does ("costed as the max frame interval over F+1..=F+2"). This function used to pair
            // THIS frame's step count with the interval that ended before its work did, and the
            // error is not small: at the beta.48 device loss it read **18.4 ms for a frame that
            // took 1070 ms**, grew the budget 4.0e8 → 6.0e8 on that, and the next dispatch measured
            // **2136 ms** and took the device out. A 58× underestimate, in the direction of growth.
            //
            // So arm a probe at dispatch and cost it as the MAX interval over the following two
            // frames. Max, not mean: one of those intervals is the stall, the others are whatever
            // the loop did while waiting, and a stall is the signal.
            //
            // Only arm on a frame that REALLY re-iterated. `fe_dispatch_frame` is stamped when the
            // pass is encoded and `frame_idx` has not advanced yet, so equality means "the frame
            // just drawn ran the pass and left its step count in `fe_steps_last`". Pricing a
            // reprojection or a key-unchanged frame the GPU deduped away would authorize the
            // budget off an interval that measured nothing.
            let dispatched = self.perf.fe_dispatch_frame[v] == self.perf.frame_idx;
            let (probe, priced) = render::wall_probe_step(
                self.perf.wall_probe[v],
                dispatched,
                self.perf.fe_steps_last[v],
                self.perf.frame_idx,
                dt_ms,
            );
            self.perf.wall_probe[v] = probe;
            if let Some((ms, steps)) = priced {
                // The wall fallback prices "the interval at which the queue finally STALLS" —
                // unlike a GPU timestamp it genuinely proves the whole queue drained, so
                // clear-to-zero is honest here (and it only runs once timestamps are starved).
                self.perf.full_inflight[v] = 0;
                self.perf.full_reg_pending[v] = 0;
                self.perf.full_done[v].store(0, std::sync::atomic::Ordering::Relaxed);
                self.apply_iterate_measurement(v, ms, steps, "wall_iterate");
            }
        }
    }
}

// Menu bar, toolbar, control panels, status bar, and the central fractal canvas. The modal
// dialogs moved to `ui/dialogs.rs`; the remaining draw_* surfaces will follow into
// `ui/{menus,panels,central}.rs` (REFACTOR-PLAN Phase 3).
impl FractadyneApp {
}

impl eframe::App for FractadyneApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let frame_start = Instant::now();
        diag::alive(); // heartbeat: a frame loop that stops arriving here is a hang (D1.4)
        // Surface a deferred startup warning (e.g. a session saved by a newer build) as a toast,
        // once, on the first frame where the UI exists.
        if let Some(msg) = self.pending_state_warning.take() {
            self.set_toast(msg, ctx);
        }
        // Surface a toast queued from a context without an `egui::Context` (e.g. a bookmark
        // auto-save failure in `save_bookmarks`).
        if let Some(msg) = self.pending_toast.take() {
            self.set_toast(msg, ctx);
        }
        // Rasterize the export watermark once from the font atlas (main thread — the export worker
        // has no egui context). Lazy so it uses the loaded fonts + final DPI.
        if self.watermark && self.watermark_overlay.is_none() {
            self.watermark_overlay =
                self.show_watermark.then(|| export::build_watermark_overlay(ctx)).flatten();
        }
        // Apply the UI scale preference (egui zoom factor) when it changes.
        if (ctx.zoom_factor() - self.ui_scale).abs() > 1.0e-4 {
            ctx.set_zoom_factor(self.ui_scale);
        }
        // GPU handles for offline export (cloned Arcs; cheap).
        let gpu = frame
            .wgpu_render_state()
            .map(|rs| (rs.device.clone(), rs.queue.clone()));
        // Motion-jam bookkeeping: retire completed full-size dispatches (the callbacks fired
        // since last frame), then arm registrations owed from LAST frame's dispatches — eframe
        // has submitted that work by now, so `on_submitted_work_done` covers it and nothing
        // newer. Order matters within the frame: drain before arming, so a callback can never
        // retire the dispatch whose registration it accompanies.
        if let Some((_, q)) = gpu.as_ref() {
            for v in 0..2 {
                let done = self.perf.full_done[v].swap(0, std::sync::atomic::Ordering::Relaxed);
                self.perf.full_inflight[v] = self.perf.full_inflight[v].saturating_sub(done);
                for _ in 0..std::mem::take(&mut self.perf.full_reg_pending[v]) {
                    let c = self.perf.full_done[v].clone();
                    q.on_submitted_work_done(move || {
                        c.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    });
                }
            }
        }
        // Adapter name for the --uitest report header (once is enough; cheap to read each frame).
        let gpu_name = frame.wgpu_render_state().map(|rs| rs.adapter.get_info().name);
        // The ADAPTER limit, not the device's: see `attach_bytes_per_sample`.
        let adapter_attach = frame
            .wgpu_render_state()
            .map(|rs| rs.adapter.limits().max_color_attachment_bytes_per_sample);
        // Capability probe, once. Recorded for the crash report and the perf HUD only — the
        // wall-clock fallback trips on observed starvation, not on this bit, so a device that
        // advertises the feature but never delivers a reading is covered too. Deliberately NOT
        // used to size anything: cost here is data-dependent, not hardware-dependent (the same
        // nominal step count measured 114 ms at one location and 1147 ms at another on the SAME
        // GPU), so a capability-derived cost model would be confidently wrong exactly where it
        // matters.
        if self.perf.frame_idx == 0 {
            if let Some((dev, _)) = &gpu {
                self.perf.ts_supported =
                    dev.features().contains(eframe::wgpu::Features::TIMESTAMP_QUERY);
                // Whether the live iteration-range (chunked) path can run — see `Perf::chunk_ok`.
                self.perf.chunk_ok = fractadyne_gpu::chunking_available(dev);
                self.perf.chunk_fe_ok = fractadyne_gpu::chunking_mode2_available(dev);
                self.attach_bytes_per_sample = (
                    dev.limits().max_color_attachment_bytes_per_sample,
                    adapter_attach.unwrap_or(0),
                );
                diag::log_line(
                    "wgpu",
                    // The BACKEND belongs in every log (and so every crash report): it names the
                    // shader compiler that built the running pipelines, and the arithmetic the
                    // deep paths depend on is a property of that compiler, not just the GPU
                    // (see `--gputest`). It is also the check that the pinned backend set in
                    // `native_options` is doing what it claims.
                    &format!(
                        "adapter: {} · {} · capability: TIMESTAMP_QUERY={} attach_bytes/sample={} granted of {} available (48 = 3 chunk-state targets, 64 = 4)",
                        self.gpu_name,
                        self.gpu_backend,
                        self.perf.ts_supported,
                        self.attach_bytes_per_sample.0,
                        self.attach_bytes_per_sample.1
                    ),
                );
            }
        }

        // Re-size the floatexp frame budget from the LIVE iterate's measured GPU time, published by the
        // paint callback a couple of frames after the pass it describes. Walk the budget toward the
        // size that measures near TDR_BUDGET_MS by the observed time RATIO — no cost model, because at
        // a deep interior view frame time is latency-bound below GPU occupancy and `steps ∝ time` is
        // simply false there (see `render::TDR_GROW_MAX`). Growth is capped so the next frame cannot
        // leap from the target into the watchdog; shrink may halve at once, since both the watchdog and
        // the UI thread are unforgiving. Nothing is tuned to a particular GPU — a slower part measures
        // a longer pass and settles at a smaller budget for the same target.
        for v in 0..2 {
            let bits = self.perf.iterate_ms[v].swap(0, std::sync::atomic::Ordering::SeqCst);
            let ms = f64::from_bits(bits);
            if bits == 0 || !(ms > 0.01) || self.perf.fe_steps_last[v] == 0 {
                if diag::trace_on("gpu") && v == 0 {
                    diag::trace(
                        "gpu",
                        format!(
                            "view={v} no reading (bits={}, ms={ms:.2}, steps={})",
                            bits != 0,
                            self.perf.fe_steps_last[v]
                        ),
                    );
                }
                // MEASUREMENT STARVATION. A real iterate ran at `fe_iter_frame` and no timing has
                // come back since; a healthy timestamp readback lands 2–3 frames later, so half a
                // second of silence means it is never coming. Give up on timestamps for the rest of
                // the session and price frames by wall clock instead (applied at the interval
                // capture below, where the timing and the step count it prices line up).
                //
                // Without this the budget sits on `TDR_BOOTSTRAP_STEPS` forever — a value chosen as
                // a safe FIRST dispatch on unknown hardware, never as a statement about what a view
                // can afford — and since beta.40 that floor drives the resolution shrink in every
                // mode: 1920×1080 at 2k iterations is 4.1e9 nominal steps against 4.0e8, so the
                // view renders at ~600×337 upscaled to the panel, permanently. `can_tile` used to
                // require a non-zero budget too, so the tiled settle could not recover it either.
                // Measured from the last READING, not the last dispatch. A settling view
                // dispatches a tile every frame, so "frames since the newest dispatch" is 0 or 1
                // forever and never trips however starved the loop is; what matters is how long
                // the unpriced work has been piling up. The first clause keeps an idle view (no
                // dispatches at all, so nothing to price) from tripping it.
                const TS_STARVE_FRAMES: u64 = 30;
                if !self.perf.wall_fallback
                    && render::measurement_starved(
                        self.perf.fe_dispatch_frame[v],
                        self.perf.ts_reading_frame[v],
                        self.perf.frame_idx,
                        TS_STARVE_FRAMES,
                    )
                {
                    self.perf.wall_fallback = true;
                    diag::log_line(
                        "wgpu",
                        &format!(
                            "no GPU iterate timing after {TS_STARVE_FRAMES} frames \
                             (TIMESTAMP_QUERY={}): pricing frames by wall clock",
                            self.perf.ts_supported
                        ),
                    );
                }
                continue;
            }
            self.perf.ts_reading_frame[v] = self.perf.frame_idx;
            // A real GPU timing arrived, so the fallback's premise is gone — UNLATCH it. It used to
            // be a one-way switch, so a single 30-frame gap (a settle, a long reference install, an
            // alt-tab) turned the wall clock into a permanent SECOND measurement source competing
            // with working timestamps, on hardware where `TIMESTAMP_QUERY=true`. That is how the
            // beta.48 device loss got its fatal reading: this machine had timestamps the whole time.
            if self.perf.wall_fallback {
                self.perf.wall_fallback = false;
                self.perf.wall_probe[v] = None;
                diag::log_line("wgpu", "GPU iterate timing resumed: wall-clock pricing off");
            }
            // The count that came back WITH the timing — see `Perf::iterate_steps`. Falls back to
            // the app-side slot only for a reading published before the paired sink existed.
            let steps = match self.perf.iterate_steps[v].swap(0, std::sync::atomic::Ordering::SeqCst)
            {
                0 => self.perf.fe_steps_last[v],
                n => n,
            };
            if self.apply_iterate_measurement(v, ms, steps, Self::SRC_GPU_ITERATE) {
                ctx.request_repaint();
            }
        }
        self.update_minimap(ctx, &gpu);

        // --uitest: advance the scripted UI/live walk. Runs each frame and does NOT exit early —
        // the flags it sets (which dialog is open, which view) must be drawn by the rest of this
        // update(); it screenshots and exits itself once the walk is done. Gated on GPU being up.
        if self.uitest.is_some() && gpu.is_some() {
            self.uitest_frame(ctx, gpu_name.as_deref());
        }
        // --juliadive: dev harness for the dual-view Julia motion path (same in-loop pattern).
        if self.juliadive.is_some() && gpu.is_some() {
            self.juliadive_frame(ctx);
        }
        // --autodive: unpaced controller hammer (same in-loop pattern). Must run AFTER the
        // measurement apply above, so the reading it samples is this frame's.
        if self.autodive.is_some() && gpu.is_some() {
            self.autodive_frame(ctx);
        }
        // --chunk-sweep: let the live view settle, then measure the chunked iterate's per-window
        // wall cost offscreen and exit. It borrows the resident reference, so it MUST run after
        // the reference pipeline has had frames to finish — hence the settle countdown inside.
        if self.chunk_sweep.is_some() {
            if let Some((dev, q)) = &gpu {
                let dev = dev.clone();
                let q = q.clone();
                if self.chunk_sweep_step(&dev, &q) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                } else {
                    ctx.request_repaint();
                }
            }
        }
        // --motiontest: the motion-presentation gate (same in-loop pattern). It sets this frame's
        // input state (zoom_vel / the Home glide), so it runs BEFORE the central draw below.
        if self.motiontest.is_some() && gpu.is_some() {
            self.motiontest_frame(ctx);
        }

        // Ctrl+S → quick export (no dialog) to the last folder.
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            if let Some((dev, q)) = &gpu {
                self.quick_export(ctx, dev.clone(), q.clone());
            }
        }

        // Esc: stop the autopilot / a playing tour first, otherwise leave fullscreen.
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.autopilot.active {
                self.autopilot.active = false;
                self.autopilot.stepping = false;
                self.pointer.zoom_vel = 0.0;
            } else if self.playback.is_some() {
                self.stop_playback();
            } else if self.fullscreen {
                self.fullscreen = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
        }

        // Navigation undo/redo, unless typing. Backspace / Shift+Backspace is the Kalles
        // Fraktaler convention; Ctrl+Z / Ctrl+Y (and Ctrl+Shift+Z) are what everyone tries
        // first (2026-08-13 UI review) — both work.
        if !ctx.wants_keyboard_input() {
            let (undo, redo) = ctx.input(|i| {
                let bs = i.key_pressed(egui::Key::Backspace);
                let z = i.modifiers.command && i.key_pressed(egui::Key::Z);
                (
                    (bs && !i.modifiers.shift) || (z && !i.modifiers.shift),
                    (bs && i.modifiers.shift)
                        || (z && i.modifiers.shift)
                        || (i.modifiers.command && i.key_pressed(egui::Key::Y)),
                )
            });
            if undo {
                self.undo_view();
            } else if redo {
                self.redo_view();
            }
            // M: find the nearby minibrot center (single view only).
            if ctx.input(|i| i.key_pressed(egui::Key::M) && !i.modifiers.any()) && !self.dual {
                self.find_minibrot(ctx);
            }
            // A: toggle the auto-zoom autopilot (single view only).
            if ctx.input(|i| i.key_pressed(egui::Key::A) && !i.modifiers.any()) && !self.dual {
                self.toggle_autopilot(ctx);
            }
            // F1 / ? : toggle the help overlay.
            if ctx.input(|i| {
                i.key_pressed(egui::Key::F1)
                    || (i.key_pressed(egui::Key::Questionmark))
                    || (i.modifiers.shift && i.key_pressed(egui::Key::Slash))
            }) {
                self.dialogs.help_open = !self.dialogs.help_open;
            }
        }

        // CLI self-test: run the GPU validation suite, print the report, and exit with a
        // status code (0 = all passed).
        if self.selftest && !self.selftest_done {
            if let Some((dev, q)) = &gpu {
                self.selftest_done = true;
                let ok = self.run_selftest(dev, q);
                crate::exit(if ok { 0 } else { 1 });
            }
        }

        // CLI profiling: render the benchmark regions, time the costly stages, log to `logs/`.
        if self.profile && !self.profile_done {
            if let Some((dev, q)) = &gpu {
                self.profile_done = true;
                let regions = match &self.profile_regions {
                    Some(path) => match profile::load_regions(std::path::Path::new(path)) {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("--regions {path}: {e}; using built-in regions");
                            profile::default_regions()
                        }
                    },
                    None => profile::default_regions(),
                };
                let secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let out = self.profile_out.clone().unwrap_or_else(|| {
                    std::path::PathBuf::from(format!("logs/profile-{}.json", Self::file_stamp(secs)))
                });
                let reps = self.profile_reps;
                self.run_profile(dev, q, &regions, reps, &out);
                crate::exit(0);
            }
        }

        // CLI path-matrix benchmark: exercise every rendering path, compare (or bless) the
        // baseline, and flag regressions. `--bless` records; `--reps N` sets timed reps.
        if self.bench_matrix && !self.bench_matrix_done {
            if let Some((dev, q)) = &gpu {
                self.bench_matrix_done = true;
                let bless = self.selftest_bless; // shared `--bless` flag
                let reps = self.profile_reps; // shared `--reps N` flag (default 5)
                let code = self.run_bench_matrix(dev, q, bless, reps, false);
                crate::exit(code);
            }
        }

        // CLI resize regression harness: scripted drag-resize through the real frame logic,
        // asserting every painted frame is aspect-correct (exit 0 = invariant held).
        if self.resizetest {
            if let Some((dev, q)) = &gpu {
                self.resizetest = false;
                self.run_resizetest(dev, q); // exits
            }
        }

        if self.reusetest && !self.reusetest_done {
            if let Some((dev, q)) = &gpu {
                self.reusetest_done = true;
                self.run_reusetest(dev, q);
                crate::exit(0);
            }
        }

        // CLI headless live-dive harness: real-time tour windows at increasing depths through the
        // actual playback machinery (pacer + lookahead + reuse-hold), stats per depth band, exit.
        if let Some(tour) = self.divetest.clone() {
            if let Some((dev, q)) = &gpu {
                let secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let out = self.profile_out.clone().unwrap_or_else(|| {
                    std::path::PathBuf::from(format!("logs/divetest-{}.json", Self::file_stamp(secs)))
                });
                self.run_divetest(dev, q, &tour, &out);
                crate::exit(0);
            }
        }

        // CLI headless live-OUTPUT harness: play a tour through the live pipeline and validate the
        // frames it puts on screen against offline renders of the same views, exit non-zero on a
        // failing checkpoint (so it can gate a release the way the selftest does).
        if let Some(tour) = self.livetest.clone() {
            if let Some((dev, q)) = &gpu {
                let (w, h) = self.tour_size_or(960, 540);
                let out = self
                    .tour_cli
                    .out
                    .clone()
                    .unwrap_or_else(|| std::path::PathBuf::from("logs"));
                let seg = self.tour_cli.segment.clone();
                let fails =
                    self.run_livetest(dev, q, &tour, seg.as_deref(), [w, h], &out, self.livetest_quick);
                crate::exit(if fails > 0 { 1 } else { 0 });
            }
        }

        if self.frametest {
            if let Some((dev, q)) = &gpu {
                let secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let out = self.profile_out.clone().unwrap_or_else(|| {
                    std::path::PathBuf::from(format!("logs/frametest-{}.json", Self::file_stamp(secs)))
                });
                let (steps, hold, dive) = (self.frametest_steps, self.frametest_hold, self.frametest_dive);
                self.run_frametest(dev, q, steps, hold, dive, 512, &out);
                crate::exit(0);
            }
        }

        // CLI standardized benchmark (`--benchmark-std [--res] [--burnin] [--depth]`): run all
        // passes synchronously, print + save the report, and quit.
        if self.auto_stdbench && !self.auto_stdbench_done {
            if let Some((dev, q)) = &gpu {
                self.auto_stdbench_done = true;
                let (res, passes, depth) = (self.std_res, self.std_passes, self.std_depth);
                println!(
                    "Fractadyne standardized benchmark — {} · {} × {passes} pass{}",
                    res.label(),
                    depth.label(),
                    if passes == 1 { "" } else { "es" }
                );
                let mut run = self.begin_standard_bench(res, passes, depth);
                // step_std_bench now advances one dive-frame per call; print only when a pass
                // finishes (passes_done ticks up), not on every frame.
                let mut reported = 0u32;
                loop {
                    let done = self.step_std_bench(&mut run, dev, q);
                    if run.passes_done > reported {
                        reported = run.passes_done;
                        println!(
                            "  pass {}/{}  {:.1} fps",
                            run.passes_done,
                            run.passes_total,
                            run.pass_fps.last().copied().unwrap_or(0.0)
                        );
                    }
                    if done {
                        break;
                    }
                }
                let report = self.format_std_bench(&run);
                println!("\n{report}");
                let out = self.auto_benchmark_out.clone().unwrap_or_else(|| {
                    std::path::PathBuf::from("fractadyne_benchmark.txt")
                });
                match std::fs::write(&out, &report) {
                    Ok(()) => println!("\nSaved benchmark → {}", out.display()),
                    Err(e) => eprintln!("Failed to save benchmark to {}: {e}", out.display()),
                }
                crate::exit(0);
            }
        }

        // GUI standardized benchmark: advance one pass per frame so the window stays responsive
        // (and cancellable) between passes.
        if let Some(mut run) = self.std_bench.take() {
            if let Some((dev, q)) = &gpu {
                let done = self.step_std_bench(&mut run, dev, q);
                if done {
                    self.bench_report = Some(self.format_std_bench(&run));
                    self.dialogs.bench_open = true;
                    let snap = run.take_snapshot();
                    self.restore_from_bench(snap);
                } else {
                    self.std_bench = Some(run);
                    ctx.request_repaint();
                }
            } else {
                self.std_bench = Some(run); // wait for the GPU handles
            }
        }

        // CLI render-and-exit: render one image offscreen (or the raw iteration EXR), save
        // it, and quit.
        if self.auto_render && !self.auto_render_done {
            if let Some((dev, q)) = &gpu {
                self.auto_render_done = true;
                if !self.watermark && !self.render_iter_mode {
                    println!("Note: Fd watermark is off (saved preference) — pass --watermark to include it.");
                }
                let t0 = std::time::Instant::now();
                let result = if self.render_iter_mode {
                    let out = self
                        .auto_render_out
                        .clone()
                        .unwrap_or_else(|| std::path::PathBuf::from("fractadyne_iter.exr"));
                    self.render_iter_to_file(dev, q, &out)
                } else {
                    let out = self
                        .auto_render_out
                        .clone()
                        .unwrap_or_else(|| std::path::PathBuf::from("fractadyne_render.png"));
                    self.render_to_file(ctx, dev, q, &out)
                };
                if self.render_cfg.finish_sound {
                    // Blocking on purpose: the process exits right after the message prints,
                    // which would cut a detached tune mid-note.
                    crate::tone::play_finish_sound(true);
                }
                match result {
                    Ok(m) => println!("{m}  (in {})", Self::fmt_export_duration(t0.elapsed())),
                    Err(e) => {
                        // Fail with a real exit code: the Close path always exits 0, which
                        // made scripted corpus renders unable to detect failures (D1/F8).
                        eprintln!("Render failed: {e}");
                        diag::log_line("render", &format!("FAILED: {e}"));
                        crate::exit(1);
                    }
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        // CLI render-tour: render the keyframe script to a PNG frame sequence, then quit.
        if let Some(script) = self.render_tour.clone() {
            if !self.render_tour_done {
                if let Some((dev, q)) = &gpu {
                    self.render_tour_done = true;
                    if !self.watermark {
                        println!("Note: Fd watermark is off (saved preference) — pass --watermark to include it.");
                    }
                    let cfg = self.tour_cli.clone();
                    match self.render_tour_to_dir(ctx, dev, q, &script, &cfg) {
                        Ok(m) => println!("{m}"),
                        Err(e) => {
                            eprintln!("Tour render failed: {e}");
                            diag::log_line("render", &format!("tour FAILED: {e}"));
                            crate::exit(1);
                        }
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }

        // CLI `--play FILE`: start the tour in the LIVE view (once). Unlike every other tour entry
        // point this one keeps the GUI running, so it exercises the on-screen path — present,
        // watchdog budget, tiled settle — which no headless harness can reach.
        if let Some(script) = self.play_tour.clone() {
            if !self.play_tour_done {
                self.play_tour_done = true;
                match scripting::parse_tour_file(&script) {
                    Ok(pb) => {
                        println!("Playing \"{}\" ({:.0}s) in the live view…", pb.name, pb.total);
                        self.playback_restore = Some(scripting::PlaybackRestore {
                            max_iter: self.render_cfg.max_iter,
                            auto_iter: self.render_cfg.auto_iter,
                            palette_idx: self.coloring.palette_idx,
                            use_custom_palette: self.coloring.use_custom_palette,
                            use_binary: self.coloring.use_binary,
                            use_duotone: self.coloring.use_duotone,
                            minimap: self.dialogs.minimap,
                            show_orbits: self.anim.show_orbits,
                            dual_split: self.dual_split,
                        });
                        self.playback = Some(pb);
                    }
                    Err(e) => {
                        eprintln!("Could not play {}: {e}", script.display());
                        crate::exit(2);
                    }
                }
            }
        }

        // Scripted camera tour / benchmark: drive the view before anything renders.
        if self.playback.is_some() && self.advance_playback(ctx) {
            self.schedule_repaint(ctx);
        }

        // CLI auto-benchmark: once the tour has finished and produced a report, print
        // + save it and quit.
        if self.auto_benchmark && !self.auto_benchmark_done && self.playback.is_none() {
            if let Some(r) = self.bench_report.clone() {
                println!("{r}");
                let path = self
                    .auto_benchmark_out
                    .clone()
                    .unwrap_or_else(|| std::path::PathBuf::from("fractadyne_benchmark.txt"));
                match std::fs::write(&path, &r) {
                    Ok(()) => println!("\nSaved benchmark → {}", path.display()),
                    Err(e) => eprintln!("Failed to save benchmark to {}: {e}", path.display()),
                }
                self.auto_benchmark_done = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        // Smooth "zoom home" glide (Home button) — advance before anything draws so
        // this frame reflects the new view; keep repainting until it finishes.
        if self.home_anim.is_some() && self.advance_home_anim(ctx) {
            self.schedule_repaint(ctx);
        }

        // Auto-zoom autopilot — dive toward detail (advance before drawing so this frame
        // reflects the new view).
        self.autopilot_step(ctx, &gpu);

        // Palette cycling animation (shifts the color offset over time).
        self.advance_palette_anim(ctx);
        // Orbit racing-dot animation.
        self.advance_orbit_anim(ctx);

        // Poll a background export for completion.
        if let Some(rx) = &self.export.task {
            match rx.try_recv() {
                Ok(msg) => {
                    self.export.status = Some(self.finish_export_status(msg));
                    self.export.task = None;
                    // The finish tone — success or failure, the user asked to be told the long
                    // wait is over. Not during the scripted UI walk, which exercises exports.
                    if self.render_cfg.finish_sound && self.uitest.is_none() {
                        crate::tone::play_finish_sound(false);
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => ctx.request_repaint(),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.export.task = None;
                    self.export.started = None;
                }
            }
        }
        // Poll a deep export whose reference is building off-thread. When it lands, assemble the
        // request (reusing the reference — no rebuild) and dispatch the render to a worker. Keeps the
        // UI alive during the (long) bignum reference build instead of freezing.
        if let Some(prep) = self.export.prep.take() {
            match prep.rx.try_recv() {
                Ok(res) => {
                    let (ew, eh, ess) = (self.export.width.max(1), self.export_height(), self.export.ss.max(1));
                    // Map view: reuse the just-built reference (no rebuild). Dual map is Mandelbrot.
                    let map_julia = prep.julia_vp.is_none() && prep.julia_mode;
                    let mut map_req = self.current_export_request_with_ref(&prep.map_vp, map_julia, Some(res));
                    map_req.width = ew;
                    map_req.height = eh;
                    map_req.ss = ess;
                    // Keep per-texel step isotropic for the chosen aspect (see build_export_job).
                    map_req.span_mantissa.y = map_req.span_mantissa.x * (eh as f64 / ew as f64);
                    let job = if let Some(jvp) = &prep.julia_vp {
                        // Dual: build the Julia panel now (usually shallow → instant) and combine.
                        let mut jul = self.current_export_request_for(jvp, true);
                        jul.width = ew;
                        jul.height = eh;
                        jul.ss = ess;
                        jul.span_mantissa.y = jul.span_mantissa.x * (eh as f64 / ew as f64);
                        match prep.dual_mode {
                            DualExport::SideBySide => ExportJob::SideBySide(map_req, jul),
                            DualExport::Separate => ExportJob::Separate(map_req, jul),
                            DualExport::ActiveOnly => ExportJob::Single(map_req),
                        }
                    } else {
                        ExportJob::Single(map_req)
                    };
                    let hud = self
                        .show_location
                        .then(|| crate::scripting::build_location_overlay(ctx, &prep.map_vp, eh))
                        .flatten();
                    if let Some((dev, q)) = &gpu {
                        self.spawn_export_worker(dev.clone(), q.clone(), job, prep.path, hud);
                    } else {
                        self.export.status = Some("GPU not available".to_string());
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.export.prep = Some(prep); // reference still building
                    ctx.request_repaint();
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.export.status = Some("Export failed: reference build aborted.".to_string());
                    self.export.started = None;
                }
            }
        }
        if let Some(prev) = self.perf.last_frame {
            // Discount the previous frame's deliberate cap sleep — it lies inside this interval and
            // is not work. See `Perf::cap_sleep_ms` for the mispricing that cost.
            let dt = (frame_start.duration_since(prev).as_secs_f64() * 1000.0
                - std::mem::take(&mut self.perf.cap_sleep_ms))
                .max(0.0);
            self.perf.last_dt_ms = dt; // the actual spike, with cap sleep removed
            self.perf.frame_ms = ema(self.perf.frame_ms, dt);
            // Same interval, second consumer: with no GPU timestamps this is the ONLY cost signal
            // the frame budget can have. Here `dt`, `fe_iter_frame`, and `fe_steps_last` all still
            // describe the frame just drawn — see `wall_clock_budget_tick`.
            self.wall_clock_budget_tick(dt);
            // Resolve adaptive-AA probes (see `Perf::aa_probe`): a stage armed on frame F is
            // costed as the max frame interval over F+1..=F+2 — with frame latency 1 a heavy GPU
            // frame stalls the NEXT acquire, so its cost shows up one interval late. A resolution
            // requests one repaint: the settle ramp's own repaint chain ends when its counter
            // exhausts, so without this a measurement that lands afterwards would sit unapplied
            // (the view would idle below the AA the measurement just authorized).
            for v in 0..2 {
                if let Some((ss, armed, mx, steps)) = self.perf.aa_probe[v] {
                    let mx = mx.max(dt);
                    if self.perf.frame_idx >= armed + 2 {
                        self.perf.aa_measured[v] = Some((ss, mx));
                        // The same interval retargets the floatexp frame budget: this frame cost `mx` at
                        // `steps`, so the budget that would cost TDR_BUDGET_MS is `steps × target / mx`.
                        // Growth is capped per probe so one anomalously quick interval can't authorize a
                        // frame that then overruns; shrink is immediate and uncapped. A frame cheaper
                        // than the interval floor simply grows the budget, which is what stops the loop
                        // from spiralling downward. See `Perf::fe_budget`.
                        // NOTE: this interval must NOT be used to price the GPU. `mx` is a wall-clock
                        // frame gap, and it is dominated by repaint scheduling (`request_repaint_after`,
                        // animation timers), not by the iterate: halving a frame's work leaves it at a
                        // near-constant ~420 ms, so any budget loop closed on it decays to nothing.
                        // The floatexp budget is calibrated against the real GPU instead — see
                        // `calibrate_fe_rate`. This probe still prices AA stages, as before.
                        let _ = steps;
                        self.perf.aa_probe[v] = None;
                        ctx.request_repaint();
                    } else {
                        self.perf.aa_probe[v] = Some((ss, armed, mx, steps));
                    }
                }
            }
        }
        self.perf.frame_idx += 1;
        self.perf.last_frame = Some(frame_start);

        if !self.auto_benchmark && !self.auto_stdbench && !self.auto_render && !self.selftest && !self.profile && !self.reusetest && self.render_tour.is_none() && self.std_bench.is_none() && !self.uitest_active() {
            self.autosave(ctx); // don't let a CLI run (or a transient benchmark override) overwrite the saved session
        }

        // Combined menu bar + action toolbar. `horizontal_wrapped` keeps them on one
        // line when the window is wide enough, and wraps the toolbar below otherwise.
        // (We place the menu buttons directly in the wrapped row rather than via
        // `menu::bar`, which would claim the full width and push the toolbar down.)
        self.draw_menu_bar(ctx, &gpu);

        self.draw_status_bar(ctx);

        // Playback transport, over the view. Drawn after the panels so `available_rect` is the
        // fractal area (below the menu bar, inside the right panel) — the transport anchors to
        // the VIEW, not the window.
        self.draw_playback_transport(ctx);
        self.poll_tour_render(ctx);
        self.draw_tour_render_dialog(ctx);

        // Right-hand control panel: fractal info, coloring, navigation, and the
        // optional performance section. Hidden entirely while in fullscreen.
        self.draw_right_panel(ctx);

        self.draw_central(ctx);

        self.draw_toast(ctx);
        // Update check: fire the silent launch check once (if enabled), then poll any in-flight one.
        if !self.update_launch_checked {
            self.update_launch_checked = true;
            if self.update_check_on_launch {
                self.start_update_check(false);
            }
        }
        self.poll_update_check(ctx);
        self.draw_update_dialog(ctx);
        self.draw_goto_dialog(ctx);
        self.draw_share_dialog(ctx);
        // Polled unconditionally (like the tour render): a test keeps running and stays reapable
        // even if the user closes the dialog while it works.
        self.poll_diagnostics(ctx);
        self.draw_diagnostics_dialog(ctx);
        self.draw_report_dialog(ctx);
        self.draw_reset_dialog(ctx);
        self.draw_script_export_dialog(ctx);
        self.draw_bookmarks_dialog(ctx);

        // Render a just-added bookmark's thumbnail (deferred here for GPU access; the current
        // view still matches the bookmark, since adding it didn't move the view).
        self.process_pending_thumb(ctx);
        // ~1 Hz process-memory poll for the perf panel (deliberately not per-frame).
        if self.perf.mem_poll.is_none_or(|t| t.elapsed().as_secs_f64() > 1.0) {
            self.perf.mem_poll = Some(std::time::Instant::now());
            let (ws, peak) = crate::sysinfo::process_memory();
            self.perf.mem_rss = ws;
            self.perf.mem_peak = peak;
            if self.perf.mem_total.is_none() {
                self.perf.mem_total = crate::sysinfo::total_memory();
            }
        }

        self.draw_bench_config_dialog(ctx);
        self.draw_bench_progress_dialog(ctx);
        self.draw_bench_results_dialog(ctx);
        self.draw_notice_dialog(ctx);

        self.draw_gallery_dialog(ctx);

        self.draw_export_dialog(ctx, &gpu);

        // ---- performance overlay + frame timing finalization ----
        let nowi = Instant::now();
        match self.perf.rate_t0 {
            Some(t0) if nowi.duration_since(t0).as_secs_f64() >= 1.0 => {
                self.perf.recompute_per_s = self.perf.rate_count as f32;
                self.perf.rate_count = 0;
                self.perf.builds_per_s = self.perf.build_count as f32;
                // A build storm is silent otherwise: the workers are off-thread, each one is
                // fast, and nothing they produce is installed. Say it once, in the log, with the
                // number — the 2026-08-07 playback crash was diagnosed only by counting
                // breadcrumbs in a 5 MB log after the fact.
                if self.perf.build_count >= BUILD_STORM_PER_S && !self.perf.build_storm_warned {
                    self.perf.build_storm_warned = true;
                    crate::diag::log_line(
                        "ref",
                        &format!(
                            "reference-build storm: {} builds in 1 s ({} lookahead, {} installed) \
                             — a build queue is rebuilding targets it never installs",
                            self.perf.build_count,
                            self.perf.prefetch_count,
                            self.perf.recompute_per_s as u32
                        ),
                    );
                }
                self.perf.build_count = 0;
                self.perf.prefetch_count = 0;
                self.perf.rate_t0 = Some(nowi);
            }
            None => self.perf.rate_t0 = Some(nowi),
            _ => {}
        }
        if self.perf.enabled {
            // Refresh the readouts without pegging the GPU. A bare `request_repaint()` here forces a
            // full re-render every frame even on a completely static view, so at deep zoom the
            // per-panel color/downsample pass runs continuously and the whole system goes laggy
            // (the app never idles — the overlay is on by default). ⭐**The refresh interval MUST
            // scale with the frame's cost.** A fixed ~4 Hz (250 ms) looked idle only because a
            // cheap frame is <10 ms; at a deep view whose COLOR pass alone is ~234 ms (measured on
            // the three-spar at 1e98× with the distance-estimate glow on), 4 Hz means a 234 ms GPU
            // frame every 250 ms — the GPU pegged ~94% to redraw a static picture, which reads as
            // "the app is always recalculating" and makes any interaction feel heavy. The metrics
            // are constant on a static view anyway, so back the refresh off to at most one frame
            // per ~4 frame-times: ~4 Hz when frames are cheap, ~1 Hz when a frame costs 234 ms —
            // capping the overlay's idle GPU duty near 25% instead of 94%. Interaction, settling,
            // panning and in-flight recomputes still request immediate repaints elsewhere, so this
            // floor never slows an ACTIVE frame; it only governs the static-view heartbeat.
            // ⚠Bounded 250 ms .. 1000 ms. `frame_ms` is an EMA of the frame-to-frame interval,
            // which now INCLUDES this idle wait — so scaling the interval by it is a positive
            // feedback loop (slower refresh → longer dt → larger EMA → slower refresh …). The
            // 1000 ms cap breaks the loop: the heartbeat settles at ~1 Hz on an expensive static
            // view (≈20–25% GPU duty at a 234 ms frame) and never drifts past it, while cheap
            // frames keep the full ~4 Hz.
            let refresh_ms = (self.perf.frame_ms * 4.0).clamp(250.0, 1000.0);
            ctx.request_repaint_after(std::time::Duration::from_millis(refresh_ms as u64));
        }
        // Keep repainting while an off-thread reference recompute is in flight, so its result is
        // polled and installed (and the view sharpens) as soon as it lands. Repaint immediately while
        // interacting so an active dive stays smooth; otherwise throttle to ~20 Hz — a progressive
        // build keeps a receiver open through the multi-second full-refine stage, and a bare
        // per-frame repaint there re-runs the un-cached color/downsample pass continuously and pegs
        // the GPU (the iterate pass is IterKey-cached, but the color pass is not). ~50 ms polling
        // installs the result within an imperceptible delay while letting the GPU idle between polls.
        if self.recompute_rx.iter().any(|r| r.is_some()) {
            let now = ctx.input(|i| i.time);
            let interacting = now - self.pointer.settle_t[0] < SETTLE_DELAY
                || now - self.pointer.settle_t[1] < SETTLE_DELAY;
            if interacting {
                ctx.request_repaint();
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
            }
        }
        let body_ms = frame_start.elapsed().as_secs_f64() * 1000.0;
        self.perf.cpu_ms = ema(self.perf.cpu_ms, body_ms);
        // ⭐SLOW-FRAME ATTRIBUTION. A long frame interval has two very different causes and the
        // interval alone cannot tell them apart: time spent INSIDE `update` (CPU work — reference
        // installs, UI, a blocking wait we own) versus time spent OUTSIDE it (eframe's acquire /
        // present, i.e. blocked on the GPU, or simply idle waiting for the next repaint request).
        // Guessing between them cost this project two wrong diagnoses of the same crash: a 1.018 s
        // frame cadence after the df32→floatexp switch reads equally well as "1 Hz idle timer" and
        // as "1 s GPU dispatches", and the two call for opposite fixes. `outside` is measured
        // against the PREVIOUS frame's body, since dt spans `frame_start(N-1) → frame_start(N)`.
        // ⚠While the BLA-suppression instrument is armed, log EVERY frame regardless of the 200 ms
        // threshold. Without this the window is unobservable and the instrument cannot be checked:
        // on 2026-08-16 the RX 6800 XT armed it correctly at frame 1861 for frames 1861–2061, and
        // the first frame that happened to exceed the slow threshold was frame 3229 — 1368 frames
        // PAST the window. Both the "it works" reading on the dev 3080 and the "it did nothing"
        // reading on the Radeon were taken from frames outside the window and meant nothing.
        // Bounded by N frames by construction, so this cannot flood a normal run.
        let instrumented = self.perf.frame_idx < self.perf.bla_suppress_until[0].max(self.perf.bla_suppress_until[1]);
        if body_ms > 200.0 || self.perf.last_dt_ms > 200.0 || instrumented {
            let outside = self.perf.last_dt_ms - self.perf.prev_body_ms;
            let rebase_bla = self.live_work_counters(0);
            diag::log_line(
                "render",
                &format!(
                    "slow frame {}: dt={:.0}ms = body {:.0}ms + outside(acquire/present/idle) \
                     {:.0}ms repaint_requested={} — mode={} steps={:.3e} budget={:.3e} \
                     rebase={} bla_skip={} ref_len={} partial={}",
                    self.perf.frame_idx,
                    self.perf.last_dt_ms,
                    self.perf.prev_body_ms,
                    outside.max(0.0),
                    // ⭐The discriminator between "blocked on the GPU" and "idle". If the app
                    // requested no repaint, eframe falls back to its ~1 Hz tick and the interval
                    // measures NOTHING about cost — 1018/1016/1017 ms three frames running is a
                    // timer, not GPU work, and the two want opposite fixes.
                    ctx.has_requested_repaint(),
                    self.perf.last_mode,
                    self.perf.fe_steps_last[0] as f64,
                    self.perf.fe_budget[0] as f64,
                    // BLA effectiveness on the very frame that was slow. `bla_skip` near zero
                    // against a large `rebase` says the reference is wrong for this iteration
                    // count, not that the frame is intrinsically expensive.
                    rebase_bla.0,
                    rebase_bla.1,
                    // Reference state closes the loop: bla_skip≈0 with a short NON-partial
                    // (escaped) reference = the unfit-reuse pathology, not an expensive view.
                    self.ref_cache[0].orbit_len,
                    self.ref_cache[0].partial,
                ),
            );
        }
        self.perf.prev_body_ms = body_ms;

        // Navigation history: record a location each time the single view settles after
        // a pan/zoom gesture (its own dedup avoids repeats). Discrete jumps record
        // explicitly. Skipped in dual view.
        let interacting_now = ctx.input(|i| i.time) - self.pointer.settle_t[0] < SETTLE_DELAY;
        if self.nav.was_interacting && !interacting_now && !self.dual {
            self.record_nav();
        }
        self.nav.was_interacting = interacting_now;

        // Frame-rate cap: pace the main thread so we don't render faster than the cap
        // (paired with vsync this snaps to a clean sub-rate, e.g. 60 on a 120 Hz panel).
        if let Some(cap) = self.fps_cap {
            if cap > 0.0 {
                let target = 1.0 / cap;
                let spent = frame_start.elapsed().as_secs_f64();
                if spent < target {
                    let nap = target - spent;
                    // Remembered so the next frame's interval can subtract it — see cap_sleep_ms.
                    self.perf.cap_sleep_ms = nap * 1000.0;
                    std::thread::sleep(std::time::Duration::from_secs_f64(nap));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Reference-length collapse trigger (crash-1786506241): a long orbit collapsing to a short
    // ESCAPED one must derate the budget; the smooth paths must never trip it. Pure-predicate
    // pin — the interactive wheel-jump that produces a collapse has no scripted repro.
    #[test]
    fn install_collapse_trigger() {
        use crate::render::install_collapse;
        // The crash: millions → 90, escaped. MUST fire.
        assert!(install_collapse(3_730_527, 90, false));
        // Escaped→escaped big shrink (fast interactive zoom-out): fires.
        assert!(install_collapse(20_000, 5_000, false));
        // Smooth zoom-out re-pick (~×0.85): must NOT fire (inside the ×1.5 margin).
        assert!(!install_collapse(5_000, 4_200, false));
        // Exactly at the boundary (new = old/1.5): not a collapse (strict inequality).
        assert!(!install_collapse(3_000, 2_000, false));
        // Shrinking PARTIAL: exempt (the pixel clamp scales cost down with it).
        assert!(!install_collapse(1_000_000, 90, true));
        // Growth of any size: never a collapse (the jump trigger owns that direction).
        assert!(!install_collapse(90, 3_730_527, false));
        // Cold start (no previous orbit): never.
        assert!(!install_collapse(0, 90, false));
    }

    // Multi-machine sharding correctness: for any (frames, segments), the N ranges must be
    // contiguous, disjoint, and cover [0, F) exactly — a missing or duplicated frame at a shard
    // boundary silently corrupts a video assembled from several machines' output.
    #[test]
    fn segment_ranges_tile_exactly() {
        use crate::scripting::segment_range;
        for &(frames, n) in &[
            (0u64, 1u64), (1, 1), (1, 4), (7, 3), (100, 7), (9931, 4), (9931, 16),
            (233, 233), (233, 500), (1_000_000, 13),
        ] {
            let mut expected_start = 0u64;
            for k in 0..n {
                let (s, e) = segment_range(frames, n, k);
                assert_eq!(s, expected_start, "F={frames} N={n} k={k}: gap or overlap at start");
                assert!(e >= s, "F={frames} N={n} k={k}: negative range");
                expected_start = e;
            }
            assert_eq!(expected_start, frames, "F={frames} N={n}: union does not cover [0, F)");
        }
    }

    // The go-to / metadata zoom string must round-trip through log2(magnification) at any
    // depth — including past f64's 1e308× range, where a plain f64 zoom would be ∞.
    #[test]
    fn zoom_field_log2_roundtrip() {
        for &log2mag in &[0.0_f64, 8.0, 49.83, 100.0, 1019.0, 1100.0, 5000.0, 1.0e5] {
            let s = fmt_zoom_field(log2mag);
            let back = parse_zoom_to_log2(&s).expect("parse failed");
            assert!((back - log2mag).abs() < 1e-3, "{log2mag} → {s} → {back}");
        }
        // Plain and grouped human input parses too.
        assert!((parse_zoom_to_log2("256").unwrap() - 8.0).abs() < 1e-9);
        assert!((parse_zoom_to_log2("1,024").unwrap() - 10.0).abs() < 1e-9);
        assert!(parse_zoom_to_log2("1e400").unwrap() > 1300.0); // past f64 range, no overflow
        // A FRACTIONAL exponent is legal and load-bearing: a zoom ladder places its rungs at
        // |lambda|^n, which is never a whole power of ten. `f64::from_str` rejects "1.0e23.9"
        // outright, and the CLI used to swallow that failure and render at 1x with exit 0 --
        // the benchmark kit measured a whole-set frame that way. Pin the value, not just
        // "parses": a ladder that lands one decade off looks entirely plausible.
        let l2 = parse_zoom_to_log2("1.0e23.900008").expect("fractional exponent rejected");
        assert!((l2 / std::f64::consts::LOG2_10 - 23.900008).abs() < 1e-9, "got {l2}");
        assert!((parse_zoom_to_log2("2.5e3.5").unwrap() / std::f64::consts::LOG2_10
            - (2.5_f64.log10() + 3.5)).abs() < 1e-9);
        // Garbage rejected, no panic.
        for g in ["", "abc", "-5", "0", "e", "1e", "nan", "inf"] {
            assert!(parse_zoom_to_log2(g).is_none(), "accepted {g:?}");
        }
    }

    // Phase 5.1: fuzz the view-metadata parser chain (untrusted: loaded from PNG tEXt
    // chunks / pasted). `meta_get` + the downstream numeric parsers must never panic and
    // must produce bounded output on arbitrary input.
    #[test]
    fn fuzz_metadata_parser_panic_free() {
        let mut s = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        let charset = b"=\n\r key value-+0.123eE\t\0[]\"";
        for _ in 0..20_000 {
            let len = (next() % 96) as usize;
            let mut buf = String::with_capacity(len);
            for _ in 0..len {
                buf.push(charset[(next() as usize) % charset.len()] as char);
            }
            for k in ["center_re", "center_im", "zoom", "fractal", "julia", "max_iter", "missing"] {
                let v = meta_get(&buf, k);
                assert!(v.len() <= buf.len(), "meta_get returned oversized value");
            }
            // The real downstream parsers applied to extracted values must not panic.
            let _ = fractadyne_core::parse_bf(&meta_get(&buf, "center_re"));
            let _ = fractadyne_core::parse_bf(&meta_get(&buf, "center_im"));
            let _ = meta_get(&buf, "zoom").parse::<f64>();
            let _ = meta_get(&buf, "max_iter").parse::<u32>();
            let _ = FractalKind::from_name(&meta_get(&buf, "fractal"));
        }
        // Adversarial explicit metadata blobs.
        for m in ["", "=", "\n\n\n", "center_re=", "=value", "zoom=NaN", "max_iter=-1",
                  "center_re=1e999999999", "fractal=\0\0\0", "a=b=c=d", "zoom=  inf  "] {
            let _ = fractadyne_core::parse_bf(&meta_get(m, "center_re"));
            let _ = meta_get(m, "zoom").parse::<f64>();
        }
    }
}
