//! The memory timeline: once a minute, the process's memory and every graphics device's live GPU
//! objects, as a `[fd-mem]` log line and a ring the crash report prints (`memory history`).
//!
//! ⭐Why (2026-11-21): two sessions (2026-11-20 22:23 and 2026-11-21 07:12) ran the machine out of
//! memory after 22 and 25 minutes, at a 33 and a 31 GB peak working set, with nothing in the log to
//! say when the memory grew or what held it, and the crash report's working set at the end read
//! 2 MB (Windows had trimmed it). Committed memory is what runs a machine out, so it is read beside
//! the working set; the devices' counts (wgpu's `counters` feature) say whether the growth is GPU
//! objects the app keeps alive.
//!
//! A device is counted while it is registered: the window's for the whole run, the second device's
//! while its thread runs ([`DeviceGuard`]). The ring never holds a device, so it keeps none alive.

use eframe::wgpu;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;

/// How often a reading is taken.
pub(crate) const EVERY: Duration = Duration::from_secs(60);
/// Readings kept for the crash report: four hours at one a minute.
pub(crate) const KEEP: usize = 240;
/// The readings a crash report prints: the session's first few, then the latest.
pub(crate) const REPORT_FIRST: usize = 3;
pub(crate) const REPORT_LAST: usize = 40;

/// A graphics device's live objects and their memory, from wgpu's internal counters (all 0 in a
/// build without the `counters` feature).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuCounts {
    pub(crate) buffers: isize,
    pub(crate) textures: isize,
    pub(crate) views: isize,
    pub(crate) bind_groups: isize,
    pub(crate) buffer_bytes: isize,
    pub(crate) texture_bytes: isize,
    pub(crate) allocations: isize,
}

impl GpuCounts {
    pub(crate) fn of(device: &wgpu::Device) -> Self {
        let c = device.get_internal_counters().hal;
        GpuCounts {
            buffers: c.buffers.read(),
            textures: c.textures.read(),
            views: c.texture_views.read(),
            bind_groups: c.bind_groups.read(),
            buffer_bytes: c.buffer_memory.read(),
            texture_bytes: c.texture_memory.read(),
            allocations: c.memory_allocations.read(),
        }
    }
}

/// One reading.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Reading {
    /// Seconds since the process started.
    pub(crate) t_s: f64,
    pub(crate) working_set: u64,
    pub(crate) peak_working_set: u64,
    /// Committed (private) memory; 0 where the platform does not say.
    pub(crate) commit: u64,
    pub(crate) peak_commit: u64,
    /// The system's free memory.
    pub(crate) system_free: Option<u64>,
    /// Each registered device, by its name.
    pub(crate) gpus: Vec<(String, GpuCounts)>,
}

/// A size as it reads best: GB from one gigabyte, MB below (82 buffers once read "0.00 GB").
fn gb(bytes: u64) -> String {
    if bytes >= 1 << 30 {
        format!("{:.2} GB", bytes as f64 / (1u64 << 30) as f64)
    } else {
        format!("{} MB", bytes >> 20)
    }
}

/// A reading as its log line (and its line in the crash report).
pub(crate) fn line(r: &Reading) -> String {
    let mut s = format!("t={:.0}s", r.t_s);
    if r.commit > 0 {
        s.push_str(&format!(" commit {} (peak {}),", gb(r.commit), gb(r.peak_commit)));
    }
    s.push_str(&format!(" working set {} (peak {})", gb(r.working_set), gb(r.peak_working_set)));
    if let Some(f) = r.system_free {
        s.push_str(&format!(", system free {}", gb(f)));
    }
    for (name, g) in &r.gpus {
        s.push_str(&format!(
            "; {name}: {} buffers {}, {} textures {}, {} views, {} bind groups, {} allocations",
            g.buffers,
            gb(g.buffer_bytes.max(0) as u64),
            g.textures,
            gb(g.texture_bytes.max(0) as u64),
            g.views,
            g.bind_groups,
            g.allocations
        ));
    }
    s
}

/// `r` added to `ring`, which keeps the newest [`KEEP`].
pub(crate) fn push_into(ring: &mut VecDeque<Reading>, r: Reading) {
    ring.push_back(r);
    while ring.len() > KEEP {
        ring.pop_front();
    }
}

/// The crash report's `memory history` section for `readings` (oldest first): the first
/// [`REPORT_FIRST`] and the last [`REPORT_LAST`], with how many were left out between them.
pub(crate) fn report_section(readings: &[Reading]) -> String {
    if readings.is_empty() {
        return "memory history: no reading was taken (the first comes as the window opens)\n".to_string();
    }
    let mut s = format!("memory history: a reading every {} s, oldest first\n", EVERY.as_secs());
    let n = readings.len();
    let skip = n.saturating_sub(REPORT_FIRST + REPORT_LAST);
    for (i, r) in readings.iter().enumerate() {
        if skip > 0 && i == REPORT_FIRST {
            s.push_str(&format!("           … {skip} readings left out …\n"));
        }
        if skip > 0 && i >= REPORT_FIRST && i < REPORT_FIRST + skip {
            continue;
        }
        s.push_str(&format!("           {}\n", line(r)));
    }
    s
}

static DEVICES: Mutex<Vec<(String, wgpu::Device)>> = Mutex::new(Vec::new());
static RING: Mutex<VecDeque<Reading>> = Mutex::new(VecDeque::new());

/// Count `device` under `name` from now on (a device of the same name is replaced). For a device
/// that lives as long as the process; a shorter-lived one takes a [`DeviceGuard`].
pub(crate) fn register_device(name: &str, device: &wgpu::Device) {
    if let Ok(mut d) = DEVICES.lock() {
        d.retain(|(n, _)| n != name);
        d.push((name.to_string(), device.clone()));
    }
}

/// Counts a device while it is held; dropping it (the device's thread ending, a panic included)
/// stops counting it and lets the device go.
pub(crate) struct DeviceGuard(String);

impl DeviceGuard {
    pub(crate) fn register(name: &str, device: &wgpu::Device) -> Self {
        register_device(name, device);
        DeviceGuard(name.to_string())
    }
}

impl Drop for DeviceGuard {
    fn drop(&mut self) {
        if let Ok(mut d) = DEVICES.lock() {
            d.retain(|(n, _)| *n != self.0);
        }
    }
}

/// A reading now.
pub(crate) fn sample() -> Reading {
    let (working_set, peak_working_set, commit, peak_commit) = crate::sysinfo::process_memory_detail();
    let gpus = DEVICES.lock().map(|d| d.iter().map(|(n, dev)| (n.clone(), GpuCounts::of(dev))).collect()).unwrap_or_default();
    Reading {
        t_s: super::elapsed_s(),
        working_set,
        peak_working_set,
        commit,
        peak_commit,
        system_free: crate::sysinfo::available_memory(),
        gpus,
    }
}

/// The readings so far, oldest first. `try_lock`: the crash report asks from the panic hook, which
/// must never wait on the sampler (it holds the lock for a push only); a miss gives none.
pub(crate) fn history() -> Vec<Reading> {
    RING.try_lock().map(|r| r.iter().cloned().collect()).unwrap_or_default()
}

/// Start taking a reading every [`EVERY`] (the first at once), once per process.
pub(crate) fn start() {
    static STARTED: std::sync::Once = std::sync::Once::new();
    STARTED.call_once(|| {
        let spawned = std::thread::Builder::new().name("fd-mem".into()).spawn(|| loop {
            let r = sample();
            super::log_line("mem", &line(&r));
            if let Ok(mut ring) = RING.lock() {
                push_into(&mut ring, r);
            }
            std::thread::sleep(EVERY);
        });
        if let Err(e) = spawned {
            super::log_line("mem", &format!("the memory timeline could not start: {e}"));
        }
    });
}
