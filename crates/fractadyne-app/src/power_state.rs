//! The machine's power state, as far as it sets how fast the graphics card runs: the key the
//! start-up speed check's saved readings are filed under (`render::speed_probe`, user 2026-11-10:
//! "check cpu clock frequency (assumed to correlate with power budget) and note that and recheck
//! if different").
//!
//! On a laptop one card is several cards: plugged in or on battery, with battery saver, under the
//! power plan and Windows' power mode slider, and hot or cool. A reading taken in one state says
//! little about another, so each is measured once and filed under what is read here:
//!
//! - **plugged in or on battery**, and **battery saver** (`GetSystemPowerStatus`);
//! - the **power plan** (`PowerGetActiveScheme`) and the **power mode** slider, Windows 10 1709 and
//!   later (`PowerGetEffectiveOverlayScheme`, looked up by name: absent on older Windows);
//! - the **CPU clock limit**, the processors' summed `MhzLimit` over their summed rated clock
//!   (`CallNtPowerInformation(ProcessorInformation)`). Windows lowers it when it caps the
//!   processors (thermal or platform limits). The CURRENT clock is no key: it moves with every
//!   idle and boost, so nearly every launch would measure again. The GPU's own clock has no
//!   standard reading; on a laptop it follows the same power state.
//!
//! Linux reads the same where sysfs has it (mains `online`, `cpufreq` limits, ACPI
//! `platform_profile`); elsewhere everything reads as unknown, which is one state.

use serde::{Deserialize, Serialize};

/// How far the CPU clock limit may move, in percentage points, and the state still be the one a
/// reading was filed under. A limit that wobbles a few points with temperature is not a new state;
/// a laptop capped to 60% is.
pub(crate) const CPU_LIMIT_SLACK_PCT: u32 = 10;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct PowerState {
    /// Plugged in: `Some(true)`; on battery: `Some(false)`; unknown (no reading): `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ac: Option<bool>,
    /// Battery saver is on.
    #[serde(default)]
    pub saver: bool,
    /// The active power plan, a GUID in lowercase ("" = unread).
    #[serde(default)]
    pub plan: String,
    /// The power mode in force (an overlay scheme GUID; "" = none or unread).
    #[serde(default)]
    pub mode: String,
    /// The processors' clock limit as a percentage of their rated clock (0 = unread).
    #[serde(default)]
    pub cpu_limit_pct: u32,
}

impl PowerState {
    /// This machine's, now.
    pub(crate) fn read() -> PowerState {
        read_now()
    }

    /// Whether a reading filed under `saved` holds in this state: everything equal, the CPU clock
    /// limit within [`CPU_LIMIT_SLACK_PCT`].
    pub(crate) fn matches(&self, saved: &PowerState) -> bool {
        self.ac == saved.ac
            && self.saver == saved.saver
            && self.plan == saved.plan
            && self.mode == saved.mode
            && self.cpu_limit_pct.abs_diff(saved.cpu_limit_pct) <= CPU_LIMIT_SLACK_PCT
    }

    /// The log's account: `plugged in, battery saver off, plan Balanced, power mode Best
    /// performance, CPU clock limit 100%`.
    pub(crate) fn line(&self) -> String {
        let ac = match self.ac {
            Some(true) => "plugged in",
            Some(false) => "on battery",
            None => "power source unknown",
        };
        let plan = if self.plan.is_empty() { "plan unread".to_string() } else { format!("plan {}", plan_name(&self.plan)) };
        let mode = if self.mode.is_empty() { String::new() } else { format!(", power mode {}", mode_name(&self.mode)) };
        let cpu = if self.cpu_limit_pct == 0 { "CPU clock limit unread".to_string() } else { format!("CPU clock limit {}%", self.cpu_limit_pct) };
        format!("{ac}, battery saver {}, {plan}{mode}, {cpu}", if self.saver { "on" } else { "off" })
    }
}

/// Windows' own plans by name, any other by its GUID.
fn plan_name(guid: &str) -> String {
    match guid {
        "381b4222-f694-41f0-9685-ff5bb260df2e" => "Balanced".into(),
        "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c" => "High performance".into(),
        "a1841308-3541-4fab-bc81-f71556f20b4a" => "Power saver".into(),
        "e9a42b02-d5df-448d-aa00-03f14749eb61" => "Ultimate Performance".into(),
        g => g.to_string(),
    }
}

/// The power mode slider's positions by name (Windows 11's labels), any other by its GUID; Linux's
/// `platform_profile` values are stored as they read.
fn mode_name(guid: &str) -> String {
    match guid {
        "961cc777-2547-4f9d-8174-7d86181b8a7a" => "Best power efficiency".into(),
        "00000000-0000-0000-0000-000000000000" => "Balanced".into(),
        "3af9b8d9-7c97-431d-ad78-34a8bfea439f" => "Better performance".into(),
        "ded574b5-45a0-4f42-8737-46345c09c238" => "Best performance".into(),
        g => g.to_string(),
    }
}

/// The summed clock limit over the summed rated clock, as a whole percentage (0 = no reading).
pub(crate) fn limit_pct(limit_mhz: u64, max_mhz: u64) -> u32 {
    if max_mhz == 0 || limit_mhz == 0 {
        return 0;
    }
    ((limit_mhz as f64 * 100.0 / max_mhz as f64).round() as u64).min(1000) as u32
}

#[cfg(windows)]
fn read_now() -> PowerState {
    use core::ffi::c_void;
    #[repr(C)]
    #[derive(Default)]
    struct SystemPowerStatus {
        ac_line_status: u8,
        battery_flag: u8,
        battery_life_percent: u8,
        system_status_flag: u8,
        battery_life_time: u32,
        battery_full_life_time: u32,
    }
    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct Guid {
        d1: u32,
        d2: u16,
        d3: u16,
        d4: [u8; 8],
    }
    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct ProcessorPowerInformation {
        number: u32,
        max_mhz: u32,
        current_mhz: u32,
        mhz_limit: u32,
        max_idle_state: u32,
        current_idle_state: u32,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetSystemPowerStatus(status: *mut SystemPowerStatus) -> i32;
        fn LocalFree(mem: *mut c_void) -> *mut c_void;
        fn LoadLibraryW(name: *const u16) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
        fn FreeLibrary(handle: *mut c_void) -> i32;
    }
    #[link(name = "powrprof")]
    unsafe extern "system" {
        fn PowerGetActiveScheme(root: *mut c_void, scheme: *mut *mut Guid) -> u32;
        fn CallNtPowerInformation(level: i32, input: *const c_void, in_len: u32, output: *mut c_void, out_len: u32) -> i32;
    }
    type OverlayFn = unsafe extern "system" fn(*mut Guid) -> u32;
    let text = |g: &Guid| {
        format!(
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            g.d1, g.d2, g.d3, g.d4[0], g.d4[1], g.d4[2], g.d4[3], g.d4[4], g.d4[5], g.d4[6], g.d4[7]
        )
    };
    let mut s = PowerState::default();

    let mut st = SystemPowerStatus::default();
    // SAFETY: an out-parameter of the documented layout.
    if unsafe { GetSystemPowerStatus(&mut st) } != 0 {
        s.ac = match st.ac_line_status {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        };
        s.saver = st.system_status_flag == 1;
    }

    let mut scheme: *mut Guid = std::ptr::null_mut();
    // SAFETY: an out-parameter the call fills with memory it allocates, freed below with LocalFree
    // as documented.
    if unsafe { PowerGetActiveScheme(std::ptr::null_mut(), &mut scheme) } == 0 && !scheme.is_null() {
        // SAFETY: the GUID the call returned, read once, then freed.
        s.plan = text(unsafe { &*scheme });
        unsafe { LocalFree(scheme.cast()) };
    }

    // The power mode: present from Windows 10 1709, so looked up rather than linked.
    let dll: Vec<u16> = "powrprof.dll".encode_utf16().chain([0]).collect();
    // SAFETY: a NUL-terminated name; powrprof is already loaded (linked above), so this only counts
    // a reference, released below.
    let h = unsafe { LoadLibraryW(dll.as_ptr()) };
    if !h.is_null() {
        // SAFETY: a live module and a NUL-terminated export name.
        let f = unsafe { GetProcAddress(h, c"PowerGetEffectiveOverlayScheme".as_ptr().cast()) };
        if !f.is_null() {
            // SAFETY: the export's documented signature, `DWORD (GUID*)`.
            let f: OverlayFn = unsafe { std::mem::transmute::<*mut c_void, OverlayFn>(f) };
            let mut g = Guid::default();
            // SAFETY: an out-parameter GUID.
            if unsafe { f(&mut g) } == 0 {
                s.mode = text(&g);
            }
        }
        // SAFETY: the reference taken above.
        unsafe { FreeLibrary(h) };
    }

    // One record per logical processor; the call needs room for all of them, so the buffer is
    // generous and the records it filled are the ones with a rated clock.
    const PROCESSOR_INFORMATION: i32 = 11;
    let n = std::thread::available_parallelism().map_or(64, |n| n.get()).clamp(64, 1024) * 4;
    let mut buf = vec![ProcessorPowerInformation::default(); n];
    let bytes = (n * std::mem::size_of::<ProcessorPowerInformation>()) as u32;
    // SAFETY: the buffer's byte length is passed; the call writes at most that many.
    if unsafe { CallNtPowerInformation(PROCESSOR_INFORMATION, std::ptr::null(), 0, buf.as_mut_ptr().cast(), bytes) } == 0 {
        let filled = buf.iter().filter(|p| p.max_mhz > 0);
        let (limit, max) = filled.fold((0u64, 0u64), |(l, m), p| (l + u64::from(p.mhz_limit), m + u64::from(p.max_mhz)));
        s.cpu_limit_pct = limit_pct(limit, max);
    }
    s
}

#[cfg(target_os = "linux")]
fn read_now() -> PowerState {
    let read = |p: &std::path::Path| std::fs::read_to_string(p).ok().map(|t| t.trim().to_string());
    let mut s = PowerState::default();
    // Mains: a power supply of type "Mains" reports `online`.
    if let Ok(dir) = std::fs::read_dir("/sys/class/power_supply") {
        for e in dir.flatten().take(32) {
            let p = e.path();
            if read(p.join("type").as_path()).as_deref() == Some("Mains") {
                match read(p.join("online").as_path()).as_deref() {
                    Some("1") => s.ac = Some(true),
                    Some("0") if s.ac.is_none() => s.ac = Some(false),
                    _ => {}
                }
            }
        }
    }
    s.mode = read(std::path::Path::new("/sys/firmware/acpi/platform_profile")).unwrap_or_default().chars().take(32).collect();
    let (mut limit, mut max) = (0u64, 0u64);
    if let Ok(dir) = std::fs::read_dir("/sys/devices/system/cpu") {
        for e in dir.flatten().take(4096) {
            let p = e.path().join("cpufreq");
            let khz = |f: &str| read(p.join(f).as_path()).and_then(|t| t.parse::<u64>().ok());
            if let (Some(l), Some(m)) = (khz("scaling_max_freq"), khz("cpuinfo_max_freq")) {
                limit += l;
                max += m;
            }
        }
    }
    s.cpu_limit_pct = limit_pct(limit, max);
    s
}

#[cfg(not(any(windows, target_os = "linux")))]
fn read_now() -> PowerState {
    PowerState::default()
}

#[cfg(test)]
#[path = "../tests/unit/power_state/tests.rs"]
mod tests;
