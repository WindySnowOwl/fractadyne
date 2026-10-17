//! What compiling pipelines costs: the account the app's watchdog and the self-test read.
//!
//! A pipeline creation is one blocking driver call that shows no progress. While the driver's own
//! shader cache answers it costs ~7 ms; when the cache misses, the driver compiles, and the big
//! iterate shaders take seconds. Measured on the RTX 3080 (2026-10-16) with NVIDIA's Vulkan cache at
//! its 1 GB cap: the gather pipeline ~7.5 s each of the 6 times the self-test creates it, a u-space BLA
//! iterate pipeline up to 9.5 s. The self-test went from 164 s to 400-450 s, and two such compiles
//! back to back with no other progress tripped the watchdog's 10 s `possible hang`. Its log check
//! failed a run that was compiling, not hung.
//!
//! So the watchdog reads [`compiles_done`]: a compile that finished is work, as a reference build's
//! steps are. It also reads [`compile_in_flight_ms`]: a compile still running is not a hang until
//! it has run far longer than any measured. [`compile_ms_total`] and [`compile_ms_max`] say what
//! compiling cost the process, which `--selftest` prints.
//!
//! ⚠Not done here: compiling each pipeline once per device rather than once per export (every
//! export creates its pipelines again; the self-test makes ~1,480 of ~18 shapes beside the custom
//! formulas). A `wgpu::PipelineCache` kept per device would do it, but wgpu 24 has no device
//! identity that is unique across instances. A device's id is per instance, and the app opens
//! each headless device on its own instance, so a cache looked up by `wgpu::Device` was handed to a
//! same-card twin whose instance has no such cache (wgpu-core panic, `PipelineCache[Id(0,1)] does
//! not exist`, 2026-10-16). It needs the cache passed in by whoever owns the device, or every
//! device on one instance.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

static EPOCH: OnceLock<Instant> = OnceLock::new();
static DONE: AtomicU64 = AtomicU64::new(0);
static IN_FLIGHT: AtomicU64 = AtomicU64::new(0);
/// The start, in ms since [`EPOCH`] plus one (0 = none), of the compile running since none was.
static OLDEST_START: AtomicU64 = AtomicU64::new(0);
static MS_TOTAL: AtomicU64 = AtomicU64::new(0);
static MS_MAX: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u64 {
    EPOCH.get_or_init(Instant::now).elapsed().as_millis() as u64
}

/// Run one pipeline creation `f`, accounted (see the module docs).
pub(crate) fn compiling<T>(f: impl FnOnce() -> T) -> T {
    let start = now_ms();
    let started = Started::new(start);
    let out = f();
    drop(started);
    let ms = now_ms().saturating_sub(start);
    MS_TOTAL.fetch_add(ms, Ordering::Relaxed);
    MS_MAX.fetch_max(ms, Ordering::Relaxed);
    DONE.fetch_add(1, Ordering::SeqCst);
    out
}

/// In flight from construction to drop, so a panicking compile does not stay "in flight".
struct Started;

impl Started {
    fn new(start_ms: u64) -> Self {
        if IN_FLIGHT.fetch_add(1, Ordering::SeqCst) == 0 {
            OLDEST_START.store(start_ms + 1, Ordering::SeqCst);
        }
        Started
    }
}

impl Drop for Started {
    fn drop(&mut self) {
        if IN_FLIGHT.fetch_sub(1, Ordering::SeqCst) == 1 {
            OLDEST_START.store(0, Ordering::SeqCst);
        }
    }
}

/// Pipeline creations finished in this process.
pub fn compiles_done() -> u64 {
    DONE.load(Ordering::SeqCst)
}

/// How long the compile now running has run, ms; `None` when none is. With several running at
/// once (a twin and this device), the age of the one started while none was.
pub fn compile_in_flight_ms() -> Option<u64> {
    if IN_FLIGHT.load(Ordering::SeqCst) == 0 {
        return None;
    }
    match OLDEST_START.load(Ordering::SeqCst) {
        0 => None,
        s => Some(now_ms().saturating_sub(s - 1)),
    }
}

/// Wall time spent creating pipelines in this process, ms.
pub fn compile_ms_total() -> u64 {
    MS_TOTAL.load(Ordering::Relaxed)
}

/// The longest single pipeline creation in this process, ms.
pub fn compile_ms_max() -> u64 {
    MS_MAX.load(Ordering::Relaxed)
}

#[cfg(test)]
#[path = "../tests/unit/pipeline_compile/tests.rs"]
mod tests;
