//! Each offscreen pass's GPU time as it is measured, for a live instrument watching a render on
//! its own thread: the live panel's worker gauge, which shows a second GPU's supersampling samples
//! as they render rather than one figure when each ends.
//!
//! A thread-local hook, so no render entry point changes signature: the caller wraps a render in
//! [`with`], and the passes report through [`report`]. Outside [`with`] a report goes nowhere.

use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};

/// Offscreen passes this process has finished, on any thread, whether or not a sink is listening.
///
/// ⭐**An export's sign of life between tiles.** Its progress moves in thousandths of the image, and
/// a deep export runs many thousands of tiles: at 4.37e21× (5120×4035, ss 4, 80,960 tiles) one 16 px
/// tile near the set's interior took 3.6–4.3 s as fourteen bounded passes, so a thousandth took
/// minutes to move and the watchdog logged `possible hang` 170 times over a healthy render that
/// finished (RTX 3070, 2026-10-25). Every pass reports after it has been read back, so an advancing
/// count is work finished, and a wedged dispatch freezes it exactly as it freezes the progress —
/// the headless progress pump reads it as it reads the reference build's step count.
pub static PASSES: AtomicU64 = AtomicU64::new(0);

/// [`PASSES`]' current value.
pub fn passes() -> u64 {
    PASSES.load(Ordering::Relaxed)
}

type Sink = Box<dyn FnMut(f64)>;

thread_local! {
    static SINK: RefCell<Option<Sink>> = const { RefCell::new(None) };
}

/// Runs `f` with `sink` taking the GPU time (ms) of every pass this thread's renders measure, then
/// restores whatever sink was there before (also if `f` panics). The sink is dropped on the way
/// out, so a sink that batches can flush in its `Drop`.
pub fn with<R>(sink: impl FnMut(f64) + 'static, f: impl FnOnce() -> R) -> R {
    struct Restore(Option<Sink>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let prev = self.0.take();
            // The sink being replaced is dropped after the borrow ends: its `Drop` may report.
            let ours = SINK.with(|s| std::mem::replace(&mut *s.borrow_mut(), prev));
            drop(ours);
        }
    }
    let prev = SINK.with(|s| s.borrow_mut().replace(Box::new(sink)));
    let _restore = Restore(prev);
    f()
}

/// One pass's GPU time, ms, to this thread's sink if it has one.
pub(crate) fn report(ms: f64) {
    // A finished pass whatever its reading: a pass that measured nothing still finished.
    PASSES.fetch_add(1, Ordering::Relaxed);
    if !(ms > 0.0 && ms.is_finite()) {
        return;
    }
    SINK.with(|s| {
        // A sink that reports from inside itself finds the cell borrowed: that report is dropped.
        if let Ok(mut g) = s.try_borrow_mut() {
            if let Some(f) = g.as_mut() {
                f(ms);
            }
        }
    });
}

#[cfg(test)]
#[path = "../tests/unit/pass_sink/tests.rs"]
mod tests;
