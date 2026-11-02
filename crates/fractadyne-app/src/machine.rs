//! ⭐**THE MACHINE PROFILE** (2026-11-01): what a report needs to know about the computer, gathered
//! in one go so a tester is asked once, not once per question.
//!
//! The GT 710 report of 2026-11-01 needed a second round to learn how fast the card was, and a
//! device-loss report has always needed one to learn whether Windows reset the driver. So the
//! self-test report, the issue report and the diagnostics bundle carry the same profile: the OS
//! version, the CPU (clock, the instructions the bignum code uses, how fast it builds a reference
//! orbit), the memory, the power plan, every graphics adapter (IDs, type, driver and its date,
//! memory, the limits and features the renderer depends on), the display, Windows' GPU timeout and
//! scheduling settings, and the display-driver resets in the event log.
//!
//! ⛔Nothing here names the user or the computer: event descriptions go through
//! [`crate::diag::redact_home`] and lose the computer's name, and no line carries either.

use std::fmt::Write as _;

/// One display-driver or GPU-reset event from the Windows event log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DriverEvent {
    /// As the log has it, UTC (`2026-10-26T14:31:12.7180000Z`).
    pub(crate) time: String,
    pub(crate) source: String,
    pub(crate) id: u32,
    pub(crate) level: String,
    /// The description's first line (empty when the log has none).
    pub(crate) what: String,
}

impl DriverEvent {
    /// What the event means, where it is one we know.
    pub(crate) fn meaning(&self) -> &'static str {
        match (self.source.as_str(), self.id) {
            (_, 4101) => "the display driver stopped responding and was reset (a TDR)",
            ("nvlddmkm", 153) => "the NVIDIA driver reset the GPU (a TDR)",
            ("nvlddmkm", 13 | 14) => "an NVIDIA graphics exception",
            ("Microsoft-Windows-Kernel-Power", 41) => "Windows restarted without shutting down cleanly",
            ("Windows Error Reporting", _) if self.what.contains("LiveKernelEvent") => {
                "a GPU reset report (LiveKernelEvent)"
            }
            ("Windows Error Reporting" | "Application Error" | "Application Hang", _) => "an application crash or hang",
            _ => "",
        }
    }
}

/// Parse `wevtutil qe … /f:text` output: blocks headed `Event[N]`, with `Source:`, `Date:`,
/// `Event ID:`, `Level:` and `Description:` (whose text is on the following lines). The `Computer:`
/// line is never kept.
pub(crate) fn parse_wevtutil_text(text: &str) -> Vec<DriverEvent> {
    let mut out = Vec::new();
    let mut cur: Option<DriverEvent> = None;
    let mut in_description = false;
    for raw in text.lines() {
        // wevtutil ends some fields with a NUL (`Level: Error` + NUL on Windows 11 26300): keep no
        // control character, or a report carrying the line reads as binary.
        let cleaned: String = raw.chars().filter(|c| !c.is_control() || *c == '\t').collect();
        let line = cleaned.trim_end();
        if line.starts_with("Event[") {
            if let Some(e) = cur.take() {
                out.push(e);
            }
            cur = Some(DriverEvent { time: String::new(), source: String::new(), id: 0, level: String::new(), what: String::new() });
            in_description = false;
            continue;
        }
        let Some(e) = cur.as_mut() else { continue };
        let t = line.trim();
        if in_description {
            if e.what.is_empty() && !t.is_empty() && t != "N/A" {
                e.what = t.to_string();
            }
            continue;
        }
        let field = |name: &str| t.strip_prefix(name).map(|v| v.trim().to_string());
        if let Some(v) = field("Source:") {
            e.source = v;
        } else if let Some(v) = field("Date:") {
            e.time = v;
        } else if let Some(v) = field("Event ID:") {
            e.id = v.parse().unwrap_or(0);
        } else if let Some(v) = field("Level:") {
            e.level = v;
        } else if let Some(v) = field("Description:") {
            in_description = true;
            if !v.is_empty() && v != "N/A" {
                e.what = v;
            }
        }
    }
    if let Some(e) = cur {
        out.push(e);
    }
    out
}

/// The event-log queries: the field agent's (`scripts/field-agent.ps1`, action `events`), which
/// have told every device loss apart from a crash since 2026-09. `(log, XPath filter)`.
fn event_queries(days: u32) -> Vec<(&'static str, String)> {
    let within = format!("TimeCreated[timediff(@SystemTime) <= {}]", days as u64 * 86_400_000);
    let system = |cond: &str| ("System", format!("*[System[({cond}) and {within}]]"));
    vec![
        system("EventID=4101"),
        system("Provider[@Name='Display']"),
        system("Provider[@Name='nvlddmkm']"),
        system("Provider[@Name='amdkmdag']"),
        system("Provider[@Name='amdwddmg']"),
        system("Provider[@Name='Microsoft-Windows-Kernel-Power'] and EventID=41"),
        ("Application", format!("*[System[Provider[@Name='Windows Error Reporting'] and {within}]]")),
        ("Application", format!("*[System[(Provider[@Name='Application Error'] or Provider[@Name='Application Hang']) and {within}]]")),
    ]
}

/// The display-driver resets, GPU reset reports, unclean restarts and this app's crashes in the
/// last `days` days, newest first, at most 40. Application events are kept only when they are a
/// GPU reset (LiveKernelEvent) or Fractadyne's own. Windows only.
pub(crate) fn driver_events(days: u32) -> Result<Vec<DriverEvent>, String> {
    if !cfg!(windows) {
        return Err("the event log is read on Windows only".to_string());
    }
    let mut all = Vec::new();
    for (log, q) in event_queries(days) {
        let mut cmd = std::process::Command::new("wevtutil");
        cmd.args(["qe", log, &format!("/q:{q}"), "/f:text", "/c:40", "/rd:true"]);
        no_console(&mut cmd);
        let out = cmd.output().map_err(|e| format!("wevtutil did not run: {e}"))?;
        // A provider this machine does not have answers with an error and no events: skip it.
        if !out.status.success() {
            continue;
        }
        let text = String::from_utf8_lossy(&out.stdout);
        for mut e in parse_wevtutil_text(&text) {
            if log == "Application" && !(e.what.contains("LiveKernelEvent") || e.what.to_ascii_lowercase().contains("fractadyne")) {
                continue;
            }
            e.what = scrub(&e.what);
            if !all.contains(&e) {
                all.push(e);
            }
        }
    }
    all.sort_by(|a, b| b.time.cmp(&a.time));
    all.truncate(40);
    Ok(all)
}

/// Take the user's home path and the computer's name out of a line from the event log.
fn scrub(s: &str) -> String {
    let s = crate::diag::redact_home(s);
    match std::env::var("COMPUTERNAME").ok().filter(|n| n.len() >= 3) {
        Some(name) => replace_ci(&s, &name, "<computer>"),
        None => s,
    }
}

/// Replace `needle` in `s`, ASCII-case-insensitively.
pub(crate) fn replace_ci(s: &str, needle: &str, with: &str) -> String {
    if needle.is_empty() {
        return s.to_string();
    }
    let lower = s.to_ascii_lowercase();
    let n = needle.to_ascii_lowercase();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while let Some(off) = lower[i..].find(&n) {
        out.push_str(&s[i..i + off]);
        out.push_str(with);
        i += off + n.len();
    }
    out.push_str(&s[i..]);
    out
}

/// The events as report lines: a count, then one line each.
pub(crate) fn events_block(events: &Result<Vec<DriverEvent>, String>, days: u32) -> String {
    match events {
        Err(e) => format!("Driver resets ({days} days): not read ({e})\n"),
        Ok(v) if v.is_empty() => format!("Driver resets ({days} days): none in the event log\n"),
        Ok(v) => {
            let mut s = format!("Driver resets ({days} days): {} event(s) in the event log, newest first\n", v.len());
            for e in v {
                let meaning = e.meaning();
                let _ = writeln!(
                    s,
                    "  {} {} {} {}{}{}",
                    e.time,
                    e.source,
                    e.id,
                    e.level,
                    if meaning.is_empty() { String::new() } else { format!(" - {meaning}") },
                    if e.what.is_empty() { String::new() } else { format!(" | {}", truncate(&e.what, 160)) }
                );
            }
            s
        }
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(n).collect::<String>())
    }
}

/// Spawn without a console window flashing up (a GUI app on Windows).
fn no_console(cmd: &mut std::process::Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    let _ = cmd;
}

/// The OS, by name, version and build.
#[cfg(windows)]
pub(crate) fn os_line() -> String {
    let key = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion";
    let build = win::reg_sz(key, "CurrentBuild").unwrap_or_default();
    let ubr = win::reg_dword(key, "UBR");
    let display = win::reg_sz(key, "DisplayVersion").unwrap_or_default();
    let edition = win::reg_sz(key, "EditionID").unwrap_or_default();
    windows_name(&build, ubr, &display, &edition)
}

/// The OS, by name, version and kernel.
#[cfg(target_os = "linux")]
pub(crate) fn os_line() -> String {
    let pretty = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|t| t.lines().find_map(|l| l.strip_prefix("PRETTY_NAME=").map(|v| v.trim_matches('"').to_string())))
        .unwrap_or_else(|| "Linux".to_string());
    let kernel = std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default();
    format!("{pretty}, kernel {}", kernel.trim())
}

/// The OS.
#[cfg(not(any(windows, target_os = "linux")))]
pub(crate) fn os_line() -> String {
    format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH)
}

/// "Windows 11 Pro 24H2 (build 26100.2894)" from the registry's values. Windows 11 still reports
/// itself as Windows 10 in `ProductName`, so the name comes from the build (22000 and later is 11).
pub(crate) fn windows_name(build: &str, ubr: Option<u32>, display: &str, edition: &str) -> String {
    let n: u32 = build.trim().parse().unwrap_or(0);
    let name = if n >= 22_000 { "Windows 11" } else if n > 0 { "Windows 10" } else { "Windows" };
    let edition = match edition.trim() {
        "" => String::new(),
        "Professional" => " Pro".to_string(),
        "Core" => " Home".to_string(),
        e => format!(" {e}"),
    };
    let display = if display.trim().is_empty() { String::new() } else { format!(" {}", display.trim()) };
    let build = match (n, ubr) {
        (0, _) => String::new(),
        (n, Some(u)) => format!(" (build {n}.{u})"),
        (n, None) => format!(" (build {n})"),
    };
    format!("{name}{edition}{display}{build}, {}", std::env::consts::ARCH)
}

/// The CPU: brand, cores, caches, clock, and the instructions the bignum code can use.
pub(crate) fn cpu_line(si: &crate::sysinfo::SysInfo) -> String {
    let brand = if si.cpu.is_empty() { "unknown CPU".to_string() } else { si.cpu.clone() };
    let mut s = format!("{brand} ({} cores / {} threads", si.physical, si.logical);
    if si.l3_kb > 0 {
        let _ = write!(s, ", L3 {} MB", si.l3_kb / 1024);
    }
    s.push(')');
    if let Some(mhz) = cpu_mhz() {
        let _ = write!(s, ", {mhz} MHz");
    }
    let feats = cpu_features();
    if !feats.is_empty() {
        let _ = write!(s, ", {}", feats.join(" "));
    }
    s
}

/// The rated clock: the registry's `~MHz`.
#[cfg(windows)]
fn cpu_mhz() -> Option<u32> {
    win::reg_dword("HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0", "~MHz").filter(|m| *m > 0)
}

/// The highest clock the kernel allows.
#[cfg(target_os = "linux")]
fn cpu_mhz() -> Option<u32> {
    std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq")
        .ok()
        .and_then(|t| t.trim().parse::<u32>().ok())
        .map(|khz| khz / 1000)
}

#[cfg(not(any(windows, target_os = "linux")))]
fn cpu_mhz() -> Option<u32> {
    None
}

/// The vector and multiply instructions present, as the bignum backends care about them.
#[cfg(target_arch = "x86_64")]
fn cpu_features() -> Vec<&'static str> {
    let mut v = Vec::new();
    if std::is_x86_feature_detected!("avx2") {
        v.push("AVX2");
    }
    if std::is_x86_feature_detected!("fma") {
        v.push("FMA");
    }
    if std::is_x86_feature_detected!("bmi2") {
        v.push("BMI2");
    }
    if std::is_x86_feature_detected!("adx") {
        v.push("ADX");
    }
    if std::is_x86_feature_detected!("avx512f") {
        v.push("AVX-512");
    }
    v
}

#[cfg(not(target_arch = "x86_64"))]
fn cpu_features() -> Vec<&'static str> {
    Vec::new()
}

/// Memory: installed and free now.
pub(crate) fn memory_line() -> String {
    let gb = |b: u64| b as f64 / (1u64 << 30) as f64;
    match (crate::sysinfo::total_memory(), crate::sysinfo::available_memory()) {
        (Some(t), Some(a)) => format!("{:.1} GB, {:.1} GB free", gb(t), gb(a)),
        (Some(t), None) => format!("{:.1} GB", gb(t)),
        _ => "unknown".to_string(),
    }
}

/// The active power plan's name (Windows; `powercfg /getactivescheme`).
pub(crate) fn power_plan() -> Option<String> {
    if !cfg!(windows) {
        return None;
    }
    let mut cmd = std::process::Command::new("powercfg");
    cmd.arg("/getactivescheme");
    no_console(&mut cmd);
    let out = cmd.output().ok()?;
    parse_powercfg(&String::from_utf8_lossy(&out.stdout))
}

/// The name in parentheses at the end of `Power Scheme GUID: … (Balanced)`.
pub(crate) fn parse_powercfg(text: &str) -> Option<String> {
    let line = text.lines().find(|l| l.contains("GUID"))?;
    let open = line.rfind('(')?;
    let close = line.rfind(')')?;
    (close > open + 1).then(|| line[open + 1..close].trim().to_string())
}

/// Windows' GPU timeout (TDR) and hardware-accelerated scheduling settings: they decide whether a
/// long pass resets the driver, and how fast.
#[cfg(windows)]
pub(crate) fn gpu_settings_line() -> Option<String> {
    let key = "SYSTEM\\CurrentControlSet\\Control\\GraphicsDrivers";
    Some(tdr_summary(
        win::reg_dword(key, "TdrDelay"),
        win::reg_dword(key, "TdrDdiDelay"),
        win::reg_dword(key, "TdrLevel"),
        win::reg_dword(key, "HwSchMode"),
    ))
}

/// No such settings off Windows.
#[cfg(not(windows))]
pub(crate) fn gpu_settings_line() -> Option<String> {
    None
}

/// The TDR and scheduling registry values as a sentence; an absent value is Windows' default.
pub(crate) fn tdr_summary(delay: Option<u32>, ddi: Option<u32>, level: Option<u32>, hags: Option<u32>) -> String {
    let set = |v: Option<u32>, default: u32, unit: &str| match v {
        Some(x) if x != default => format!("{x}{unit} (changed from {default}{unit})"),
        Some(x) => format!("{x}{unit}"),
        None => format!("{default}{unit} (default)"),
    };
    let level_word = match level.unwrap_or(3) {
        0 => "detection OFF",
        1 => "bug check",
        2 => "recover VGA",
        3 => "recover",
        _ => "unknown",
    };
    let hags = match hags {
        Some(2) => "on",
        Some(1) => "off",
        Some(_) => "unknown",
        None => "not set",
    };
    format!(
        "TdrDelay {}, TdrDdiDelay {}, TdrLevel {} = {level_word}; hardware-accelerated GPU scheduling {hags}",
        set(delay, 2, " s"),
        set(ddi, 5, " s"),
        set(level, 3, "")
    )
}

/// The graphics adapters: the one in use in full (IDs, type, backend, driver, memory, limits,
/// features), then every other one the app could see, one line each.
pub(crate) fn adapter_lines(rs: &eframe::egui_wgpu::RenderState) -> Vec<String> {
    let mut v = Vec::new();
    let info = rs.adapter.get_info();
    let mut head = format!(
        "{} [{:04x}:{:04x}] {:?}, {:?}, driver {} {}",
        info.name, info.vendor, info.device, info.device_type, info.backend, info.driver, info.driver_info
    );
    let vram = crate::sysinfo::gpu_vram_bytes(Some(&info.name));
    if vram > 0 {
        let _ = write!(head, ", {} MB", vram >> 20);
    }
    v.push(format!("GPU in use: {}", head.trim_end()));
    #[cfg(windows)]
    {
        if let Some((provider, version, date)) = win::display_driver(&info.name) {
            v.push(format!("  driver package: {provider} {version}, dated {date}"));
        }
    }
    let l = rs.device.limits();
    v.push(format!(
        "  limits: texture 2D {} px, buffer {} MB, storage binding {} MB, workgroup {} invocations, bind groups {}",
        l.max_texture_dimension_2d,
        l.max_buffer_size >> 20,
        l.max_storage_buffer_binding_size >> 20,
        l.max_compute_invocations_per_workgroup,
        l.max_bind_groups
    ));
    v.push(format!("  features: {:?}", rs.adapter.features()));
    for a in rs.available_adapters.iter() {
        let i = a.get_info();
        if i.name == info.name && i.backend == info.backend && i.device == info.device {
            continue;
        }
        v.push(format!(
            "Other GPU: {} [{:04x}:{:04x}] {:?}, {:?}, driver {} {}",
            i.name, i.vendor, i.device, i.device_type, i.backend, i.driver, i.driver_info
        ));
    }
    v
}

/// The display the window is on, and the window.
pub(crate) fn display_line(ctx: &egui::Context) -> String {
    ctx.input(|i| {
        let vp = i.viewport();
        let ppp = vp.native_pixels_per_point.unwrap_or(i.pixels_per_point);
        let monitor = vp
            .monitor_size
            .map(|m| format!("monitor {:.0}x{:.0} pt", m.x, m.y))
            .unwrap_or_else(|| "monitor size unknown".to_string());
        let window = vp
            .inner_rect
            .map(|r| format!("window {:.0}x{:.0} pt", r.width(), r.height()))
            .unwrap_or_default();
        format!("{monitor} at {ppp:.2}x scale, {window}")
    })
}

/// How fast this CPU builds a reference orbit, one core, in the build's bignum backend: an interior
/// point (it never escapes, so every iteration runs) at a shallow and a deep precision. Reference
/// builds are the CPU pause before a deep view resolves; this lets a report's build times be read
/// against the machine. About a tenth of a second on a 2019 desktop CPU.
pub(crate) fn cpu_reference_speed() -> String {
    let mut parts = Vec::new();
    for (octaves, iters) in [(100u64, 20_000u32), (3_000, 2_000)] {
        let p = fractadyne_core::precision_for_octaves(octaves);
        let (Some(zero), Some(cx), Some(cy)) = (
            fractadyne_core::parse_bf_prec("0", p),
            fractadyne_core::parse_bf_prec("-0.1", p),
            fractadyne_core::parse_bf_prec("0.1", p),
        ) else {
            return "not measured (the point did not parse)".to_string();
        };
        let t = std::time::Instant::now();
        let (_orbit, len) = fractadyne_core::reference_orbit(&zero, &zero, &cx, &cy, 0, iters, p);
        let us = t.elapsed().as_secs_f64() * 1e6 / len.max(1) as f64;
        parts.push(format!("{us:.2} µs per iteration at {p} bits"));
    }
    format!("{} ({}, one core)", parts.join(", "), fractadyne_core::backend_status_line())
}

/// The days of event log a profile reads.
pub(crate) const EVENT_DAYS: u32 = 30;

/// The quick part of the profile (the registry, the GPU's own description): OS, CPU, memory, the
/// graphics adapters, Windows' GPU settings and the display. `rs` and `ctx` are optional: a harness
/// without a window has neither.
pub(crate) fn fast_text(
    si: &crate::sysinfo::SysInfo,
    rs: Option<&eframe::egui_wgpu::RenderState>,
    ctx: Option<&egui::Context>,
) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "OS: {}", os_line());
    let _ = writeln!(s, "CPU: {}", cpu_line(si));
    let _ = writeln!(s, "Memory: {}", memory_line());
    if let Some(rs) = rs {
        for l in adapter_lines(rs) {
            let _ = writeln!(s, "{l}");
        }
    }
    if let Some(g) = gpu_settings_line() {
        let _ = writeln!(s, "GPU settings: {g}");
    }
    if let Some(ctx) = ctx {
        let _ = writeln!(s, "Display: {}", display_line(ctx));
    }
    s
}

/// The slow part: the CPU's reference-orbit speed, the power plan and the event log (about half a
/// second on Windows, most of it the event log).
pub(crate) fn slow_text(event_days: u32) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "CPU reference speed: {}", cpu_reference_speed());
    if let Some(p) = power_plan() {
        let _ = writeln!(s, "Power plan: {p}");
    }
    s.push_str(&events_block(&driver_events(event_days), event_days));
    s
}

/// The whole profile, as report lines, gathered now (the self-test report, the bundle).
pub(crate) fn profile_text(
    si: &crate::sysinfo::SysInfo,
    rs: Option<&eframe::egui_wgpu::RenderState>,
    ctx: Option<&egui::Context>,
    event_days: u32,
) -> String {
    format!("{}{}", fast_text(si, rs, ctx), slow_text(event_days))
}

static FAST: std::sync::OnceLock<String> = std::sync::OnceLock::new();
static SLOW: std::sync::OnceLock<String> = std::sync::OnceLock::new();
static SLOW_STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Gather the profile for [`cached_text`]: the quick part now (once a session), the slow part on a
/// thread of its own. Cheap to call every frame after the first.
pub(crate) fn warm(si: &crate::sysinfo::SysInfo, rs: Option<&eframe::egui_wgpu::RenderState>, ctx: &egui::Context) {
    FAST.get_or_init(|| fast_text(si, rs, Some(ctx)));
    if !SLOW_STARTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        let ctx = ctx.clone();
        let _ = std::thread::Builder::new().name("fd-machine-profile".into()).spawn(move || {
            let _ = SLOW.set(slow_text(EVENT_DAYS));
            ctx.request_repaint();
        });
    }
}

/// The profile [`warm`] gathered, for a text rebuilt every frame (the issue report's preview):
/// `None` before `warm`, and a note in place of the slow part while it is still being gathered.
pub(crate) fn cached_text() -> Option<String> {
    let fast = FAST.get()?;
    Some(format!(
        "{fast}{}",
        SLOW.get().map(String::as_str).unwrap_or("(still gathering the CPU's reference speed, the power plan and the driver resets)\n")
    ))
}

/// Take the user's home path and the computer's name out of text that leaves the machine (the
/// diagnostics bundle's logs).
pub(crate) fn redact(s: &str) -> String {
    scrub(s)
}

#[cfg(windows)]
mod win {
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
    const HKLM: isize = 0x8000_0002u32 as i32 as isize;
    const RRF_RT_REG_SZ: u32 = 0x0000_0002;
    const RRF_RT_REG_DWORD: u32 = 0x0000_0010;

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    /// A REG_SZ value under HKLM.
    pub(super) fn reg_sz(key: &str, value: &str) -> Option<String> {
        let (k, v) = (wide(key), wide(value));
        let mut buf = vec![0u16; 512];
        let mut cb = (buf.len() * 2) as u32;
        // SAFETY: `k`/`v` are NUL-terminated UTF-16 buffers that outlive the call; `buf` is `cb` bytes
        // long and RegGetValueW writes at most `cb` bytes (updating it). A non-zero result means the
        // value is absent or too long, and the buffer is not read.
        let rc = unsafe {
            RegGetValueW(HKLM, k.as_ptr(), v.as_ptr(), RRF_RT_REG_SZ, std::ptr::null_mut(), buf.as_mut_ptr().cast(), &mut cb)
        };
        if rc != 0 {
            return None;
        }
        let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Some(String::from_utf16_lossy(&buf[..n]).trim().to_string())
    }

    /// A REG_DWORD value under HKLM.
    pub(super) fn reg_dword(key: &str, value: &str) -> Option<u32> {
        let (k, v) = (wide(key), wide(value));
        let mut data = 0u32;
        let mut cb = 4u32;
        // SAFETY: as `reg_sz`; `data` is 4 bytes and `cb` says so.
        let rc = unsafe {
            RegGetValueW(
                HKLM,
                k.as_ptr(),
                v.as_ptr(),
                RRF_RT_REG_DWORD,
                std::ptr::null_mut(),
                (&mut data as *mut u32).cast(),
                &mut cb,
            )
        };
        (rc == 0).then_some(data)
    }

    /// The display driver package for the adapter named `name`: its provider, version and date,
    /// from the display class's driver key whose description matches (as `gpu_vram_bytes` does).
    pub(super) fn display_driver(name: &str) -> Option<(String, String, String)> {
        let want = name.trim().to_ascii_lowercase();
        for i in 0..64 {
            let key = format!("SYSTEM\\CurrentControlSet\\Control\\Class\\{{4d36e968-e325-11ce-bfc1-08002be10318}}\\{i:04}");
            let Some(desc) = reg_sz(&key, "DriverDesc") else { continue };
            let d = desc.to_ascii_lowercase();
            if d.is_empty() || !(d.contains(&want) || want.contains(d.as_str())) {
                continue;
            }
            return Some((
                reg_sz(&key, "ProviderName").unwrap_or_default(),
                reg_sz(&key, "DriverVersion").unwrap_or_default(),
                reg_sz(&key, "DriverDate").unwrap_or_default(),
            ));
        }
        None
    }
}

#[cfg(test)]
#[path = "../tests/unit/machine/tests.rs"]
mod tests;
