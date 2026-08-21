//! Host system facts + version/time helpers (no external dependencies).
//!
//! Used by the benchmark report and the validation/profiling logs: CPU brand (CPUID),
//! physical cores + L2/L3 cache and dedicated VRAM (Windows APIs), process working set,
//! plus the build-version string and a chrono-free UTC formatter.

/// Semantic version (from Cargo) + an auto-incrementing per-build counter (build.rs).
pub(crate) const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const BUILD_SEQ: &str = env!("FRACT_BUILD");

/// Display string, e.g. `0.1.0 (build 42)`.
pub(crate) fn version_string() -> String {
    format!("{APP_VERSION} (build {BUILD_SEQ})")
}

/// Unix seconds → "YYYY-MM-DD HH:MM:SS UTC" (civil-date algorithm; no chrono dependency).
pub(crate) fn utc_string(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let r = (secs % 86_400) as i64;
    let (hh, mm, ss) = (r / 3600, (r % 3600) / 60, r % 60);
    let z = days + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}:{ss:02} UTC")
}

/// Current wall-clock time as "YYYY-MM-DD HH:MM:SS UTC" (for report/run timestamps).
pub(crate) fn now_utc_string() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    utc_string(secs)
}

/// Current + peak process working-set bytes (for the benchmark RAM metric).
#[cfg(windows)]
pub(crate) fn process_memory() -> (u64, u64) {
    #[repr(C)]
    struct Pmc {
        cb: u32,
        page_fault_count: u32,
        peak_working_set: usize,
        working_set: usize,
        quota_peak_paged: usize,
        quota_paged: usize,
        quota_peak_nonpaged: usize,
        quota_nonpaged: usize,
        pagefile: usize,
        peak_pagefile: usize,
    }
    extern "system" {
        fn GetCurrentProcess() -> isize;
        fn K32GetProcessMemoryInfo(h: isize, c: *mut Pmc, cb: u32) -> i32;
    }
    // SAFETY: `Pmc` is `#[repr(C)]` matching PROCESS_MEMORY_COUNTERS; we zero it and set `cb` to its
    // own size before the call, as the Win32 API requires. `GetCurrentProcess()` returns a pseudo-
    // handle that needs no closing. On failure (non-zero not returned) we ignore `pmc` and return 0s.
    unsafe {
        let mut pmc: Pmc = std::mem::zeroed();
        pmc.cb = std::mem::size_of::<Pmc>() as u32;
        if K32GetProcessMemoryInfo(GetCurrentProcess(), &mut pmc, pmc.cb) != 0 {
            (pmc.working_set as u64, pmc.peak_working_set as u64)
        } else {
            (0, 0)
        }
    }
}
#[cfg(target_os = "linux")]
pub(crate) fn process_memory() -> (u64, u64) {
    // `/proc/self/status`: `VmRSS` (resident now) and `VmHWM` (peak resident), both in kB.
    let Ok(st) = std::fs::read_to_string("/proc/self/status") else {
        return (0, 0);
    };
    let grab = |key: &str| -> u64 {
        st.lines()
            .find(|l| l.starts_with(key))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse::<u64>().ok())
            .map(|kb| kb * 1024)
            .unwrap_or(0)
    };
    (grab("VmRSS:"), grab("VmHWM:"))
}
#[cfg(not(any(windows, target_os = "linux")))]
pub(crate) fn process_memory() -> (u64, u64) {
    (0, 0)
}

/// Bytes free on the volume holding `path` (for the caller's user quota), or `None` when it can't
/// be determined — an unwritable/nonexistent path included, so the caller must treat `None` as
/// "unknown", never as "full".
#[cfg(windows)]
pub(crate) fn free_disk_bytes(path: &std::path::Path) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;
    extern "system" {
        fn GetDiskFreeSpaceExW(
            dir: *const u16,
            free_to_caller: *mut u64,
            total: *mut u64,
            total_free: *mut u64,
        ) -> i32;
    }
    // The path may not exist yet (the render creates its output folder), so walk up to the first
    // ancestor that does — the volume is the same either way, and that is all we are asking about.
    let mut probe = if path.as_os_str().is_empty() {
        std::path::Path::new(".")
    } else {
        path
    };
    while !probe.exists() {
        probe = probe.parent()?;
    }
    let wide: Vec<u16> = probe.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let (mut free, mut total, mut total_free) = (0u64, 0u64, 0u64);
    // SAFETY: `wide` is a NUL-terminated UTF-16 path that outlives the call, and the three outputs
    // are valid `u64` slots. A zero return means failure, in which case nothing is read.
    let ok = unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut free, &mut total, &mut total_free) };
    (ok != 0).then_some(free)
}

#[cfg(target_os = "linux")]
pub(crate) fn free_disk_bytes(path: &std::path::Path) -> Option<u64> {
    use std::os::unix::ffi::OsStrExt;
    // Same ancestor walk as the Windows arm: the target may not exist yet (renders create their
    // output folder), and the volume is the same for any existing ancestor.
    let mut probe = if path.as_os_str().is_empty() {
        std::path::Path::new(".")
    } else {
        path
    };
    while !probe.exists() {
        probe = probe.parent()?;
    }
    let c = std::ffi::CString::new(probe.as_os_str().as_bytes()).ok()?;
    // SAFETY: `c` is a NUL-terminated path that outlives the call; `st` is zeroed storage the
    // call fills. A non-zero return means failure, in which case nothing is read. `f_bavail`
    // (blocks available to an unprivileged caller) × `f_frsize` is the caller's usable quota —
    // the same semantic the Windows arm's `free_to_caller` reports.
    unsafe {
        let mut st: libc::statvfs = std::mem::zeroed();
        (libc::statvfs(c.as_ptr(), &mut st) == 0)
            .then(|| st.f_bavail as u64 * st.f_frsize as u64)
    }
}
#[cfg(not(any(windows, target_os = "linux")))]
pub(crate) fn free_disk_bytes(_path: &std::path::Path) -> Option<u64> {
    None
}

/// Available PHYSICAL memory in bytes — memory that could be allocated right now without paging,
/// or `None` when it can't be determined (the caller must treat `None` as "unknown", never as
/// "plenty"). Used to gate the tour render's reference lookahead so it never builds a second
/// bignum reference that won't fit alongside the one already resident.
#[cfg(windows)]
pub(crate) fn available_memory() -> Option<u64> {
    #[repr(C)]
    struct MemoryStatusEx {
        length: u32,
        memory_load: u32,
        total_phys: u64,
        avail_phys: u64,
        total_page_file: u64,
        avail_page_file: u64,
        total_virtual: u64,
        avail_virtual: u64,
        avail_extended_virtual: u64,
    }
    extern "system" {
        fn GlobalMemoryStatusEx(buffer: *mut MemoryStatusEx) -> i32;
    }
    // SAFETY: `status` is zeroed storage with `length` set to its own size, as the API requires;
    // the call fills it and a non-zero return means the `avail_phys` field is valid.
    unsafe {
        let mut status: MemoryStatusEx = std::mem::zeroed();
        status.length = std::mem::size_of::<MemoryStatusEx>() as u32;
        (GlobalMemoryStatusEx(&mut status) != 0).then_some(status.avail_phys)
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn available_memory() -> Option<u64> {
    // `MemAvailable` (kernel's own estimate of allocatable-without-swapping, in kB) is the right
    // signal — better than MemFree, which omits reclaimable cache. Present since Linux 3.14.
    let text = std::fs::read_to_string("/proc/meminfo").ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("MemAvailable:") {
            let kb: u64 = rest.split_whitespace().next()?.parse().ok()?;
            return Some(kb * 1024);
        }
    }
    None
}

#[cfg(not(any(windows, target_os = "linux")))]
pub(crate) fn available_memory() -> Option<u64> {
    None
}

/// Total physical memory, so a "used" figure can be reported as a fraction rather than a bare
/// number that means nothing without the denominator.
#[cfg(windows)]
pub(crate) fn total_memory() -> Option<u64> {
    #[repr(C)]
    struct MemoryStatusEx {
        length: u32,
        memory_load: u32,
        total_phys: u64,
        avail_phys: u64,
        total_page_file: u64,
        avail_page_file: u64,
        total_virtual: u64,
        avail_virtual: u64,
        avail_extended_virtual: u64,
    }
    extern "system" {
        fn GlobalMemoryStatusEx(buffer: *mut MemoryStatusEx) -> i32;
    }
    // SAFETY: identical contract to `available_memory` above — zeroed storage, `length` set to its
    // own size, non-zero return means the fields are valid.
    unsafe {
        let mut status: MemoryStatusEx = std::mem::zeroed();
        status.length = std::mem::size_of::<MemoryStatusEx>() as u32;
        (GlobalMemoryStatusEx(&mut status) != 0).then_some(status.total_phys)
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn total_memory() -> Option<u64> {
    let text = std::fs::read_to_string("/proc/meminfo").ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            let kb: u64 = rest.split_whitespace().next()?.parse().ok()?;
            return Some(kb * 1024);
        }
    }
    None
}

#[cfg(not(any(windows, target_os = "linux")))]
pub(crate) fn total_memory() -> Option<u64> {
    None
}

/// System-wide CPU busy percentage, measured as a DELTA between successive calls.
///
/// ⚠Stateful by necessity: CPU time is a monotonic counter, so a single reading is meaningless —
/// the percentage only exists relative to a previous sample. The first call therefore returns
/// `None` (it establishes the baseline) rather than a fabricated 0%, which would read as "idle" at
/// exactly the moment a caller most wants the truth.
#[derive(Default)]
pub(crate) struct CpuSampler {
    prev: Option<(u64, u64)>, // (idle, total)
}

impl CpuSampler {
    /// Busy percent since the previous call, or `None` for the first call / on error.
    pub(crate) fn sample(&mut self) -> Option<f64> {
        let (idle, total) = read_cpu_times()?;
        let out = match self.prev {
            Some((pi, pt)) if total > pt => {
                let d_total = (total - pt) as f64;
                let d_idle = (idle.saturating_sub(pi)) as f64;
                Some(((d_total - d_idle) / d_total * 100.0).clamp(0.0, 100.0))
            }
            _ => None,
        };
        self.prev = Some((idle, total));
        out
    }
}

#[cfg(windows)]
fn read_cpu_times() -> Option<(u64, u64)> {
    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct FileTime {
        low: u32,
        high: u32,
    }
    extern "system" {
        fn GetSystemTimes(idle: *mut FileTime, kernel: *mut FileTime, user: *mut FileTime) -> i32;
    }
    fn v(f: FileTime) -> u64 {
        ((f.high as u64) << 32) | f.low as u64
    }
    // SAFETY: three out-params of the exact FILETIME layout the API writes; non-zero return means
    // all three were filled. Note Windows counts idle time INSIDE kernel time, so total is
    // kernel+user and idle is a subset of it — not an additional term.
    unsafe {
        let (mut i, mut k, mut u) = (FileTime::default(), FileTime::default(), FileTime::default());
        if GetSystemTimes(&mut i, &mut k, &mut u) == 0 {
            return None;
        }
        Some((v(i), v(k) + v(u)))
    }
}

#[cfg(target_os = "linux")]
fn read_cpu_times() -> Option<(u64, u64)> {
    // /proc/stat's first line: cpu user nice system idle iowait irq softirq steal ...
    let text = std::fs::read_to_string("/proc/stat").ok()?;
    let line = text.lines().next()?.strip_prefix("cpu ")?;
    let f: Vec<u64> = line
        .split_whitespace()
        .filter_map(|t| t.parse().ok())
        .collect();
    if f.len() < 5 {
        return None;
    }
    // idle + iowait are both "not doing work".
    let idle = f[3] + f.get(4).copied().unwrap_or(0);
    Some((idle, f.iter().sum()))
}

#[cfg(not(any(windows, target_os = "linux")))]
fn read_cpu_times() -> Option<(u64, u64)> {
    None
}

/// GPU utilisation percent and VRAM used/total bytes, via `nvidia-smi`.
///
/// ⚠Deliberately best-effort and vendor-limited: there is no portable GPU counter, and shelling out
/// is too slow to do at a high rate. Callers must sample this rarely and treat `None` as "unknown",
/// never as zero — an AMD box reporting 0% GPU would be a lie, and the RX 6800 XT is exactly the
/// machine whose numbers matter most right now.
pub(crate) fn gpu_usage() -> Option<(f64, u64, u64)> {
    let out = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=utilization.gpu,memory.used,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let row = text.lines().next()?;
    let mut f = row.split(',').map(|s| s.trim());
    let util: f64 = f.next()?.parse().ok()?;
    let used: u64 = f.next()?.parse().ok()?;
    let total: u64 = f.next()?.parse().ok()?;
    Some((util, used * 1024 * 1024, total * 1024 * 1024))
}

/// Host system facts shown in benchmark reports (gathered once at startup).
pub(crate) struct SysInfo {
    pub(crate) cpu: String,
    pub(crate) logical: usize,
    pub(crate) physical: usize,
    pub(crate) l2_kb: u64,
    pub(crate) l3_kb: u64,
    pub(crate) vram_mb: u64,
}

/// `active_gpu` is the wgpu adapter name when the caller has one — the Windows VRAM probe uses
/// it to pick the RIGHT display adapter out of the registry (see `gpu_vram_bytes`). `None` is
/// fine for harness contexts and falls back to the widest scan.
pub(crate) fn gather_system_info(active_gpu: Option<&str>) -> SysInfo {
    let logical = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(0);
    let (physical, l2_kb, l3_kb) = cpu_topology();
    SysInfo {
        cpu: cpu_brand(),
        logical,
        physical: if physical == 0 { logical } else { physical },
        l2_kb,
        l3_kb,
        vram_mb: gpu_vram_bytes(active_gpu) / (1024 * 1024),
    }
}

/// CPU brand string via the CPUID extended leaves (no dependencies).
#[cfg(target_arch = "x86_64")]
fn cpu_brand() -> String {
    use std::arch::x86_64::__cpuid;
    // `__cpuid` is part of the x86_64 baseline, so it is callable in safe code.
    if __cpuid(0x8000_0000).eax < 0x8000_0004 {
        return String::new();
    }
    let mut bytes = Vec::with_capacity(48);
    for leaf in [0x8000_0002u32, 0x8000_0003, 0x8000_0004] {
        let r = __cpuid(leaf);
        for v in [r.eax, r.ebx, r.ecx, r.edx] {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
    }
    String::from_utf8_lossy(&bytes)
        .trim_matches(|c: char| c == '\0' || c.is_whitespace())
        .to_string()
}
#[cfg(not(target_arch = "x86_64"))]
fn cpu_brand() -> String {
    String::new()
}

/// Physical core count + total L2/L3 cache (KB) via GetLogicalProcessorInformation.
#[cfg(windows)]
fn cpu_topology() -> (usize, u64, u64) {
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Slpi {
        processor_mask: usize,
        relationship: u32,
        _pad: u32,
        info: [u8; 16], // union; for RelationCache it holds CACHE_DESCRIPTOR
    }
    #[repr(C)]
    struct CacheDescriptor {
        level: u8,
        assoc: u8,
        line_size: u16,
        size: u32,
        ctype: u32,
    }
    extern "system" {
        fn GetLogicalProcessorInformation(buf: *mut Slpi, len: *mut u32) -> i32;
    }
    // SAFETY: first call passes a null buffer to query the required byte length into `len`; we then
    // allocate a `Vec<Slpi>` (`#[repr(C)]` matching SYSTEM_LOGICAL_PROCESSOR_INFORMATION) of exactly
    // `len/size_of::<Slpi>()` elements and pass its pointer + capacity, so the API cannot overrun it.
    unsafe {
        let mut len: u32 = 0;
        GetLogicalProcessorInformation(std::ptr::null_mut(), &mut len);
        let sz = std::mem::size_of::<Slpi>() as u32;
        if len == 0 || sz == 0 {
            return (0, 0, 0);
        }
        let count = (len / sz) as usize;
        let mut buf = vec![
            Slpi {
                processor_mask: 0,
                relationship: 0,
                _pad: 0,
                info: [0u8; 16],
            };
            count
        ];
        if GetLogicalProcessorInformation(buf.as_mut_ptr(), &mut len) == 0 {
            return (0, 0, 0);
        }
        let (mut physical, mut l2, mut l3) = (0usize, 0u64, 0u64);
        for e in &buf {
            match e.relationship {
                0 => physical += 1, // RelationProcessorCore
                2 => {
                    // RelationCache
                    let cd = &*(e.info.as_ptr() as *const CacheDescriptor);
                    match cd.level {
                        2 => l2 += cd.size as u64,
                        3 => l3 += cd.size as u64,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        (physical, l2 / 1024, l3 / 1024)
    }
}
#[cfg(target_os = "linux")]
fn cpu_topology() -> (usize, u64, u64) {
    // Physical cores: distinct (physical id, core id) pairs in /proc/cpuinfo — hyperthreads
    // share the pair, sockets differ in physical id.
    let physical = std::fs::read_to_string("/proc/cpuinfo")
        .map(|s| {
            let mut phys = 0u64;
            let mut set = std::collections::HashSet::new();
            for l in s.lines() {
                if let Some((k, v)) = l.split_once(':') {
                    match k.trim() {
                        "physical id" => phys = v.trim().parse().unwrap_or(0),
                        "core id" => {
                            set.insert((phys, v.trim().parse::<u64>().unwrap_or(u64::MAX)));
                        }
                        _ => {}
                    }
                }
            }
            set.len()
        })
        .unwrap_or(0);
    // L2/L3 totals: walk /sys/devices/system/cpu/cpu*/cache/index*, counting each cache ONCE —
    // a cache shared by several CPUs appears under each of them, so only the CPU that is first
    // in its `shared_cpu_list` counts it (the list's first entry is canonical).
    let (mut l2, mut l3) = (0u64, 0u64);
    let read = |p: &std::path::Path| std::fs::read_to_string(p).unwrap_or_default();
    if let Ok(rd) = std::fs::read_dir("/sys/devices/system/cpu") {
        for e in rd.flatten() {
            let name = e.file_name();
            let name = name.to_string_lossy();
            let Some(cpu_n) = name.strip_prefix("cpu").and_then(|n| n.parse::<u64>().ok()) else {
                continue;
            };
            let cache = e.path().join("cache");
            let Ok(idx) = std::fs::read_dir(&cache) else { continue };
            for c in idx.flatten() {
                if !c.file_name().to_string_lossy().starts_with("index") {
                    continue;
                }
                let dir = c.path();
                let first_sharer = read(&dir.join("shared_cpu_list"));
                let first: Option<u64> = first_sharer
                    .trim()
                    .split(&[',', '-'][..])
                    .next()
                    .and_then(|v| v.parse().ok());
                if first != Some(cpu_n) {
                    continue; // another CPU owns this (shared) cache's count
                }
                let size = read(&dir.join("size"));
                let size = size.trim();
                let kb = if let Some(m) = size.strip_suffix('M') {
                    m.parse::<u64>().unwrap_or(0) * 1024
                } else {
                    size.strip_suffix('K').unwrap_or(size).parse::<u64>().unwrap_or(0)
                };
                match read(&dir.join("level")).trim() {
                    "2" => l2 += kb,
                    "3" => l3 += kb,
                    _ => {}
                }
            }
        }
    }
    (physical, l2, l3)
}
#[cfg(not(any(windows, target_os = "linux")))]
fn cpu_topology() -> (usize, u64, u64) {
    (0, 0, 0)
}

/// The render-finished tone: FRACTINT's actual "normal completion" tune, read out of the DOS
/// source rather than guessed (user request 2026-08-16; sourcing done 2026-08-21). From
/// `general.asm` in the FRACTINT source (mirror: LegalizeAdulthood/fractint), verbatim:
///
/// ```text
/// buzzer0         dw      1047,100        ; "normal" completion
///                 dw      1109,100
///                 dw      1175,100
///                 dw      0,0
/// ```
///
/// Three rising 100 ms notes — C6, C#6, D6 — on the PC speaker. (`dos/sound.c`'s soundcard path
/// uses the same three frequencies, confirming the tune; "interrupted" was the descending
/// mirror 2093/1976/1857 and "error" a 40 Hz razzberry, neither used here.) `kernel32 Beep` is
/// the modern PC-speaker shim (synthesized through the default output device since Windows 7),
/// so this is the faithful reproduction. ⚠Beep BLOCKS for the note's duration.
///
/// `blocking`: the GUI passes false (the tune plays on its own thread; a 300 ms stall in
/// `update` would be a real hitch); the CLI `--render` path passes true — the process exits
/// right after the completion message, which would kill a detached tune mid-note, and 300 ms
/// added to a finished render is nothing.
#[cfg(windows)]
pub(crate) fn play_finish_sound(blocking: bool) {
    fn tune() {
        #[link(name = "kernel32")]
        extern "system" {
            fn Beep(freq: u32, ms: u32) -> i32;
        }
        for (freq, ms) in [(1047u32, 100u32), (1109, 100), (1175, 100)] {
            // SAFETY: no pointers; Beep plays synchronously and returns.
            unsafe {
                Beep(freq, ms);
            }
        }
    }
    if blocking {
        tune();
    } else {
        std::thread::spawn(tune);
    }
}
#[cfg(not(windows))]
pub(crate) fn play_finish_sound(_blocking: bool) {}

/// Dedicated VRAM (bytes) read from the display-adapter registry keys.
///
/// ⚠Every GPU ever installed leaves a class subkey behind, so on a swap-bench machine "max over
/// the first 8 subkeys" reported a STALE adapter: the RX 6800 XT field report said 8192 MB for a
/// 16 GB card (finding ① of the 2026-08-15 device-loss report) — a previously installed 8 GB
/// card's entry sat in 0000-0007 while the live card's entry was further down. The probe now
/// scans 64 subkeys, reads each entry's `DriverDesc`, and prefers entries matching the ACTIVE
/// wgpu adapter name; the widest max is only the fallback when no name is available or nothing
/// matches. Best-effort; returns 0 if unavailable.
#[cfg(windows)]
fn gpu_vram_bytes(active_gpu: Option<&str>) -> u64 {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "advapi32")]
    extern "system" {
        fn RegGetValueW(
            hkey: isize,
            subkey: *const u16,
            value: *const u16,
            flags: u32,
            ptype: *mut u32,
            pdata: *mut core::ffi::c_void,
            pcb: *mut u32,
        ) -> i32;
    }
    let hklm: isize = 0x8000_0002u32 as i32 as isize; // HKEY_LOCAL_MACHINE
    let wide = |s: &str| -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    };
    let value = wide("HardwareInformation.qwMemorySize");
    let desc_value = wide("DriverDesc");
    let want = active_gpu.map(|s| s.trim().to_ascii_lowercase()).filter(|s| !s.is_empty());
    let mut matched = 0u64;
    let mut any = 0u64;
    for i in 0..64 {
        let key = wide(&format!(
            "SYSTEM\\CurrentControlSet\\Control\\Class\\{{4d36e968-e325-11ce-bfc1-08002be10318}}\\{i:04}"
        ));
        let mut data = [0u8; 8];
        let mut cb = 8u32;
        // SAFETY: `key`/`value` are NUL-terminated UTF-16 buffers that outlive the call; `data` is a
        // fixed 8-byte buffer and `cb` is set to its size, which RegGetValueW respects (it writes at
        // most `cb` bytes and updates it). A non-zero `rc` means "not found" and we skip the buffer.
        let rc = unsafe {
            RegGetValueW(
                hklm,
                key.as_ptr(),
                value.as_ptr(),
                0x0000_ffff, // RRF_RT_ANY
                std::ptr::null_mut(),
                data.as_mut_ptr() as *mut core::ffi::c_void,
                &mut cb,
            )
        };
        if rc != 0 {
            continue;
        }
        let bytes = u64::from_le_bytes(data);
        any = any.max(bytes);
        if let Some(want) = want.as_deref() {
            let mut desc = [0u16; 256];
            let mut dcb = (desc.len() * 2) as u32;
            // SAFETY: same contract as above; `desc` is a fixed buffer and `dcb` its byte size.
            let rc = unsafe {
                RegGetValueW(
                    hklm,
                    key.as_ptr(),
                    desc_value.as_ptr(),
                    0x0000_0002, // RRF_RT_REG_SZ
                    std::ptr::null_mut(),
                    desc.as_mut_ptr() as *mut core::ffi::c_void,
                    &mut dcb,
                )
            };
            if rc == 0 {
                let n = desc.iter().position(|&c| c == 0).unwrap_or(desc.len());
                let d = String::from_utf16_lossy(&desc[..n]).trim().to_ascii_lowercase();
                // Containment either way: the driver and wgpu phrase the same card slightly
                // differently across vendors (suffixes like "(TM)" on one side).
                if !d.is_empty() && (d.contains(want) || want.contains(d.as_str())) {
                    matched = matched.max(bytes);
                }
            }
        }
    }
    if matched > 0 { matched } else { any }
}
#[cfg(target_os = "linux")]
fn gpu_vram_bytes(_active_gpu: Option<&str>) -> u64 {
    // amdgpu/i915 expose VRAM bytes directly in sysfs; take the largest card. (Stale-adapter
    // selection is a Windows-registry problem; sysfs lists only PRESENT cards, so the name is
    // unused here.)
    let mut best = 0u64;
    if let Ok(rd) = std::fs::read_dir("/sys/class/drm") {
        for e in rd.flatten() {
            let p = e.path().join("device/mem_info_vram_total");
            if let Ok(v) = std::fs::read_to_string(&p) {
                best = best.max(v.trim().parse::<u64>().unwrap_or(0));
            }
        }
    }
    if best > 0 {
        return best;
    }
    // NVIDIA has no sysfs equivalent — ask nvidia-smi (absent ⇒ 0; this is diagnostics-grade,
    // best-effort by design, and runs once at startup).
    std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=memory.total", "--format=csv,noheader,nounits"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.lines().filter_map(|l| l.trim().parse::<u64>().ok()).max())
        .map(|mib| mib * 1024 * 1024)
        .unwrap_or(0)
}
#[cfg(not(any(windows, target_os = "linux")))]
fn gpu_vram_bytes(_active_gpu: Option<&str>) -> u64 {
    0
}
