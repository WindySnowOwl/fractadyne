//! The graphics driver's shader cache: telling the user when it stops keeping compiled shaders.
//!
//! A working driver cache answers a pipeline this process has compiled before in milliseconds.
//! When it is full it does not, and Fractadyne's largest shaders compile for 5-10 s each, at
//! every export and every launch (2026-10-16: NVIDIA's Vulkan cache at its 1 GB cap made the
//! self-test 386 s instead of 164 s). `fractadyne_gpu::pipeline_compile` notices the first repeat
//! that compiled slowly; this says so once a session, in the log and, outside a harness, in a
//! notice that tells the user what to clear.

use crate::FractadyneApp;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// The window adapter's vendor, once known (0 = not yet), so a harness's exit can name NVIDIA's
/// folder too.
static VENDOR: AtomicU32 = AtomicU32::new(0);
static LOGGED: AtomicBool = AtomicBool::new(false);

/// What the log line and the notice say: the slow repeats, and NVIDIA's cache folder with its size.
type Finding = (fractadyne_gpu::pipeline_compile::SlowRepeat, Option<(PathBuf, u64)>);

/// Record the window adapter's vendor (`wgpu::AdapterInfo::vendor`) the first time it is known.
pub(crate) fn set_vendor(vendor: u32) {
    let _ = VENDOR.compare_exchange(0, vendor, Ordering::Relaxed, Ordering::Relaxed);
}

/// The first time the driver was seen not keeping shaders, log it (once a process) and return it.
/// Called each frame and from `crate::exit`, because a harness can do all its work inside one frame.
pub(crate) fn log_once() -> Option<Finding> {
    let r = fractadyne_gpu::pipeline_compile::driver_cache_not_keeping()?;
    if LOGGED.swap(true, Ordering::SeqCst) {
        return None;
    }
    let nvidia = if VENDOR.load(Ordering::Relaxed) == NVIDIA {
        nvidia_cache_dir().map(|d| {
            let n = dir_size(&d);
            (d, n)
        })
    } else {
        None
    };
    crate::diag::log_line(
        "gpu",
        &format!(
            "the graphics driver's shader cache is not keeping compiled shaders: {} compiled again slowly, the slowest {} in {:.1} s{}",
            r.count,
            r.pipeline,
            r.ms as f64 / 1000.0,
            nvidia.as_ref().map_or(String::new(), |(d, n)| format!(" (NVIDIA cache {}, {} MB)", d.display(), n >> 20)),
        ),
    );
    Some((r, nvidia))
}

/// PCI vendor ids, from `wgpu::AdapterInfo::vendor`.
pub(crate) const NVIDIA: u32 = 0x10DE;
pub(crate) const AMD: u32 = 0x1002;

/// NVIDIA's OpenGL and Vulkan shader cache (`%LOCALAPPDATA%\NVIDIA\GLCache`), if it exists.
pub(crate) fn nvidia_cache_dir() -> Option<PathBuf> {
    let dir = PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join("NVIDIA").join("GLCache");
    dir.is_dir().then_some(dir)
}

/// The bytes of the files under `dir`.
pub(crate) fn dir_size(dir: &Path) -> u64 {
    let mut total = 0;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            match e.metadata() {
                Ok(m) if m.is_dir() => stack.push(e.path()),
                Ok(m) => total += m.len(),
                Err(_) => {}
            }
        }
    }
    total
}

/// The notice's title and body. `nvidia`: the cache folder and its size, when the card is NVIDIA's
/// and the folder exists.
pub(crate) fn notice(
    r: &fractadyne_gpu::pipeline_compile::SlowRepeat,
    vendor: u32,
    nvidia: Option<(&Path, u64)>,
) -> (String, String) {
    let said = tr!(
        driver_cache_body,
        count = r.count,
        pipeline = r.pipeline.as_str(),
        seconds = crate::i18n::decimal(r.ms as f64 / 1000.0, 1)
    );
    let fix = match (vendor, nvidia) {
        (NVIDIA, Some((dir, bytes))) => tr!(
            driver_cache_fix_nvidia_folder,
            folder = dir.display().to_string(),
            size = (bytes >> 20).to_string()
        ),
        (NVIDIA, None) => tr!(driver_cache_fix_nvidia),
        (AMD, _) => tr!(driver_cache_fix_amd),
        _ => tr!(driver_cache_fix_other),
    };
    (tr!(driver_cache_title), format!("{said}\n\n{fix}"))
}

impl FractadyneApp {
    /// Once a session: when the driver's shader cache stopped keeping compiled shaders, log it and,
    /// outside a harness, show the notice (waiting while another notice is up).
    pub(crate) fn check_driver_shader_cache(&mut self) {
        if self.driver_cache_told {
            return;
        }
        if fractadyne_gpu::pipeline_compile::driver_cache_not_keeping().is_none() {
            return;
        }
        let task = crate::launched_as_task();
        if !task && self.dialogs.notice.is_some() {
            return;
        }
        self.driver_cache_told = true;
        if let Some((r, nvidia)) = log_once() {
            if !task {
                let vendor = VENDOR.load(Ordering::Relaxed);
                self.dialogs.notice = Some(notice(&r, vendor, nvidia.as_ref().map(|(d, n)| (d.as_path(), *n))));
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/driver_cache/tests.rs"]
mod tests;
