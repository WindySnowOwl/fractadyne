//! Optional per-pass **GPU timestamp** capture for the profiling harness.
//!
//! Every "GPU ms" the harnesses report today is CPU wall-clock around a blocking
//! `submit` + `poll(Wait)` + readback — so it folds in command encoding, queue submission, and
//! buffer readback, and (in the live path) vsync/present wait. That conflation is exactly what
//! mis-led an op-level optimization pass (it read the wrong column and called a real win
//! "perf-neutral"). GPU timestamp queries fix it: when capture is enabled (profiler-only) and the
//! device advertises [`wgpu::Features::TIMESTAMP_QUERY`], [`crate::render_export`] brackets its
//! **iterate** and **color** passes with GPU timestamps and accumulates the pure-GPU pass time
//! here — vsync- and CPU-overhead-independent, and split per pass so the deep-floatexp iterate cost
//! can finally be attributed on its own.
//!
//! The capture is a thread-local toggle (mirroring the app's existing `ProfSetup` `Cell`), so no
//! render entry point changes signature: the profiler wraps a render in [`capture`], and the
//! render path calls [`accumulate`].
//!
//! Since D3.1 the export paths take timestamps **unconditionally** (results also flow out via
//! `ExportResult.iterate_ms/color_ms`, which works across threads); this thread-local scope
//! remains only for `--profile`'s aggregated per-region view. `accumulate` outside a capture
//! scope writes an accumulator nobody reads — harmless, `capture` resets it on entry.

use std::cell::Cell;

/// Pure-GPU time (milliseconds) of the iterate and color passes, summed across the tiles/passes
/// run inside one [`capture`] scope. `captured` is false when the device lacks
/// `TIMESTAMP_QUERY` (the profiler then prints the CPU-timed columns only).
#[derive(Clone, Copy, Default)]
pub struct PassTimings {
    pub iterate_ms: f64,
    pub color_ms: f64,
    pub captured: bool,
}

thread_local! {
    static ACC: Cell<PassTimings> =
        const { Cell::new(PassTimings { iterate_ms: 0.0, color_ms: 0.0, captured: false }) };
}

/// Run `f` with GPU per-pass timestamp capture enabled, returning its result plus the accumulated
/// iterate/color GPU time. Not re-entrant (profiler-only, single-threaded) — a nested call resets
/// the accumulator.
pub fn capture<R>(f: impl FnOnce() -> R) -> (R, PassTimings) {
    ACC.with(|a| a.set(PassTimings::default()));
    let r = f();
    (r, ACC.with(|a| a.get()))
}

/// Add one pass pair's measured GPU time to the active capture accumulator.
pub(crate) fn accumulate(iterate_ms: f64, color_ms: f64) {
    ACC.with(|a| {
        let mut v = a.get();
        v.iterate_ms += iterate_ms;
        v.color_ms += color_ms;
        v.captured = true;
        a.set(v);
    });
}
