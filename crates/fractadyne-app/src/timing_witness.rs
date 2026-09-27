//! THE TIMING WITNESS: every GPU timestamp reading the frame budget prices is held against a bound
//! the CPU can vouch for.
//!
//! The 2026-09-27 pass clock found that on the RX 6800 XT (Windows, Vulkan) the GPU's timestamps do
//! not keep time with the CPU's: against each pass's frame they step ±150–550 ms dozens of times a
//! run, and a 2.16 s step lands with nothing in flight (the RTX 3080 tracks to 0.3%, no step over
//! 27 ms). A reading can therefore be wrong in either direction, and the budget is priced from them.
//!
//! The witness is the CPU-side WINDOW of the timed pass: from the moment the GPU side ARMED the
//! timer (in `prepare`, after the pass was recorded and before the frame was submitted) to the
//! moment the GPU's completion callback for that frame's submission ran (after the work finished;
//! a late poll can only lengthen it). The pass ran inside that window, so a reading LONGER than it
//! is impossible. Just before arming, the GPU side polls once, so the previous frame's completion
//! is stamped if it has happened: when it precedes the arming, the queue was empty and the window
//! is this frame's own GPU work plus callback latency — a reading far SHORTER than it is suspect.
//!
//! Measured (beta.128, ~2,500 readings on both cards): no reading was too short; 2 of 1,505 on the
//! RX 6800 XT were impossible, both a walk's empty tail. So from beta.129 the budget refuses an
//! impossible reading outright, a suspect-short one may only shrink it, and the GPU side no longer
//! times an empty tail (`apply_iterate_measurement`; `fractadyne-gpu`'s `arm_ts`).

use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::sync::Arc;

/// Frames remembered. A reading lands 2–3 frames after its pass; 64 leaves room for stalls.
pub(crate) const RING: usize = 64;

/// A reading longer than its window by more than this is IMPOSSIBLE (timestamp granularity and the
/// microsecond stamps).
pub(crate) const IMPOSSIBLE_TOLERANCE_MS: f64 = 0.5;

/// An empty-queue reading that leaves more of its window than this unexplained may be too SHORT.
/// Measured on the tap rung (2026-09-27, ~2,500 readings): the window exceeds a truthful reading by
/// 13–18 ms at the median (callback latency, about a frame) and never by more than 58 ms (RX 6800 XT)
/// or 48 ms (RTX 3080). A false positive only withholds one growth step.
pub(crate) const SHORT_SLACK_MS: f64 = 100.0;

/// How often the running tally is logged.
const SUMMARY_EVERY_US: u64 = 30_000_000;

/// What the witness says about one reading.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Verdict {
    /// Arming to the frame's completion callback, ms: an upper bound on the pass's GPU time.
    pub(crate) window_ms: f64,
    /// The previous frame's work had completed before the timer was armed.
    pub(crate) queue_empty: bool,
}

impl Verdict {
    /// The reading could not have happened: longer than the window it ran inside.
    pub(crate) fn impossible(&self, reading_ms: f64) -> bool {
        reading_ms > self.window_ms + IMPOSSIBLE_TOLERANCE_MS
    }
}

#[derive(Default)]
struct Tally {
    n: u64,
    empty: u64,
    impossible: u64,
    worst_over_ms: f64,
    /// window − reading for empty-queue readings (the overhead a truthful reading leaves).
    slack: Vec<f64>,
    since_us: u64,
}

pub(crate) struct Witness {
    /// (frame, µs) when each frame's completion callback ran, written from the callback.
    done: Arc<Vec<[AtomicU64; 2]>>,
    tally: Tally,
}

impl Default for Witness {
    fn default() -> Self {
        Self {
            done: Arc::new((0..RING).map(|_| [AtomicU64::new(u64::MAX), AtomicU64::new(0)]).collect()),
            tally: Tally::default(),
        }
    }
}

impl Witness {
    /// Register the completion callback for frame `frame`, whose work has been submitted: call on
    /// the FOLLOWING frame's update (eframe submits after `update` returns).
    pub(crate) fn arm_done(&self, q: &wgpu::Queue, frame: u64, now: fn() -> u64) {
        let done = self.done.clone();
        q.on_submitted_work_done(move || {
            let slot = &done[(frame % RING as u64) as usize];
            slot[1].store(now(), Relaxed);
            slot[0].store(frame, Relaxed);
        });
    }

    /// Stamp a completion directly (tests).
    #[cfg(test)]
    pub(crate) fn stamp_done(&self, frame: u64, us: u64) {
        let slot = &self.done[(frame % RING as u64) as usize];
        slot[1].store(us, Relaxed);
        slot[0].store(frame, Relaxed);
    }

    fn done_us(&self, frame: u64) -> Option<u64> {
        let slot = &self.done[(frame % RING as u64) as usize];
        (slot[0].load(Relaxed) == frame).then(|| slot[1].load(Relaxed))
    }

    /// The witness for a reading of a pass recorded on `frame` whose timer was armed at `armed_us`,
    /// or `None` when that frame's completion has not been seen (or has left the ring).
    pub(crate) fn judge(&self, frame: u64, armed_us: u64) -> Option<Verdict> {
        if armed_us == 0 {
            return None;
        }
        let done = self.done_us(frame)?;
        let queue_empty = frame > 0 && self.done_us(frame - 1).is_some_and(|d| d <= armed_us);
        Some(Verdict { window_ms: done.saturating_sub(armed_us) as f64 / 1000.0, queue_empty })
    }

    /// Count one witnessed reading; every [`SUMMARY_EVERY_US`] return the tally as a log line.
    pub(crate) fn tally(&mut self, reading_ms: f64, v: &Verdict, now_us: u64) -> Option<String> {
        let t = &mut self.tally;
        if t.n == 0 {
            t.since_us = now_us;
        }
        t.n += 1;
        if v.impossible(reading_ms) {
            t.impossible += 1;
            t.worst_over_ms = t.worst_over_ms.max(reading_ms - v.window_ms);
        }
        if v.queue_empty {
            t.empty += 1;
            if t.slack.len() < 8192 {
                t.slack.push(v.window_ms - reading_ms);
            }
        }
        if now_us.saturating_sub(t.since_us) < SUMMARY_EVERY_US {
            return None;
        }
        let line = summary(t);
        *t = Tally::default();
        Some(line)
    }
}

fn summary(t: &Tally) -> String {
    let mut s = t.slack.clone();
    s.sort_by(f64::total_cmp);
    let q = |p: f64| s.get(((s.len() as f64 - 1.0) * p).round() as usize).copied().unwrap_or(0.0);
    format!(
        "timing witness: {} GPU reading(s) held against their pass's completion window, {} with the \
         queue empty; IMPOSSIBLE (longer than the window) {}{}; empty-queue window minus reading ms: \
         p50 {:.1} p95 {:.1} min {:.1} max {:.1}",
        t.n,
        t.empty,
        t.impossible,
        if t.impossible > 0 { format!(", worst {:.1} ms over", t.worst_over_ms) } else { String::new() },
        q(0.5),
        q(0.95),
        s.first().copied().unwrap_or(0.0),
        s.last().copied().unwrap_or(0.0),
    )
}

#[cfg(test)]
#[path = "timing_witness_tests.rs"]
mod tests;
