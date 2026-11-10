//! `--adapter` and `--list-adapters`: which graphics card the app renders on.
//!
//! Without `--adapter` wgpu chooses (the high-performance adapter), exactly as before. With it, the
//! app takes the adapter the spec names from the SAME enumeration egui-wgpu makes —
//! `enumerate_adapters` over [`backends`] — so a number `--list-adapters` prints is the adapter this
//! takes. A spec is that number (1-based), or part of the adapter's name, case-insensitive; where
//! several adapters match, the Vulkan one is taken, then the first. A render farm's client uses it
//! to run one session per card (design/remote-rendering.md §12, Phase 4).

use eframe::wgpu;

pub(crate) const FLAG: &str = "--adapter";
pub(crate) const LIST_FLAG: &str = "--list-adapters";

/// The graphics APIs the app runs on: Vulkan and GL, unless `WGPU_BACKEND` names others, and GL
/// alone where this machine's Vulkan loader is too old to list cards through, or Vulkan could not
/// open the window here before ([`vulkan_left_out`]).
/// One definition for the window's instance and for the listing, so their enumerations agree;
/// decided once a run.
pub(crate) fn backends() -> wgpu::Backends {
    static CHOSEN: std::sync::OnceLock<wgpu::Backends> = std::sync::OnceLock::new();
    *CHOSEN.get_or_init(|| {
        wgpu::Backends::from_env().unwrap_or_else(|| {
            let mut b = wgpu::Backends::VULKAN | wgpu::Backends::GL;
            if vulkan_left_out().is_some() {
                b.remove(wgpu::Backends::VULKAN);
            }
            b
        })
    })
}

/// ⭐Why this run leaves Vulkan out of [`backends`], or `None` (the usual case).
///
/// **A Vulkan 1.0 loader.** Venus (2026-11-10: a 2012 MacBook Pro's GeForce GT 650M under Boot
/// Camp, NVIDIA 382.05) has the 2016 loader `vulkan-1.dll` 1.0.26, and Fractadyne crashed natively
/// listing cards through it, so it could not start at all: once reading 0xffff… inside the loader,
/// once calling a null pointer from wgpu's `expose_adapter`, both right after wgpu enabled
/// VK_KHR_get_physical_device_properties2 (which the driver offers and, as I read it, that loader
/// predates and cannot route). OpenGL and DirectX 12 list the card fine there. A native crash
/// cannot be caught, so the check comes before any Vulkan call: the loader's exports are read, and
/// one without `vkEnumerateInstanceVersion` (Vulkan 1.1, loaders from 2018 on) is left out. That is
/// the very test wgpu makes for the instance version it logs ("Instance version: 0x400000" = 1.0).
///
/// **Vulkan could not open the window here before** ([`fallback`]): the same build, with the same
/// loader, failed at start-up on Vulkan and started again on OpenGL; this run starts there.
///
/// `WGPU_BACKEND` set = nothing is left out (and the loader is not read): `WGPU_BACKEND=vulkan`
/// tries Vulkan anyway. The loader check is Windows only.
pub(crate) fn vulkan_left_out() -> Option<&'static str> {
    static WHY: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    WHY.get_or_init(|| {
        if wgpu::Backends::from_env().is_some() {
            return None;
        }
        loader::this_machine().and_then(|l| l.too_old()).or_else(|| {
            fallback::remembered().map(|f| {
                format!(
                    "Vulkan could not open the window on {} {} ({}), so this run uses OpenGL. A new \
                     Fractadyne, a new Vulkan loader, or deleting {} in the settings folder tries Vulkan \
                     again; WGPU_BACKEND=vulkan tries it anyway.",
                    f.adapter,
                    if f.when.is_empty() { "at an earlier launch".to_string() } else { format!("at {}", f.when) },
                    f.reason,
                    fallback::FILE
                )
            })
        })
    })
    .as_deref()
}

/// The log's line about the Vulkan loader: why it is left out, or which one this machine has.
pub(crate) fn log_loader() {
    if let Some(why) = vulkan_left_out() {
        crate::diag::log_line("gpu", &format!("Vulkan is left out: {why}"));
    } else if let Some(l) = loader::this_machine() {
        crate::diag::log_line("gpu", &format!("Vulkan loader: {}", l.line()));
    }
}

/// ⭐VULKAN THAT COULD NOT OPEN THE WINDOW, remembered (user, 2026-11-10: "look into supporting a
/// fallback mode if a gpu can't be successfully initialized", then "yes" to this first part).
///
/// On Venus (GeForce GT 650M, NVIDIA 382.05, a Vulkan 1.4 loader from the LunarG SDK) Vulkan lists
/// the card, then the window cannot open: wgpu 24 always builds its indirect-validation compute
/// pipeline at device creation, and that driver loses the device on it ("Parent device is lost",
/// 20261110-143726-kgso), while OpenGL runs there. So when the window cannot open on a Vulkan
/// adapter that this run chose by itself (no `WGPU_BACKEND`), the failure is written to [`FILE`] in
/// the settings folder, and a session the user started (not a task, which must fail visibly) starts
/// again at once on OpenGL, where its window would have opened ([`start_again`]), saying so once
/// ([`RELAUNCH_ENV`]). A later launch of the same version
/// with the same Vulkan loader reads the file and leaves Vulkan out from the start
/// ([`super::vulkan_left_out`]); a new version or a new loader tries Vulkan again, and deleting the
/// file tries it now.
pub(crate) mod fallback {
    use serde::{Deserialize, Serialize};

    pub(crate) const FILE: &str = "vulkan-failed.toml";
    /// Set on the session started again on OpenGL: the failure's reason, for its one notice.
    pub(crate) const RELAUNCH_ENV: &str = "FRACTADYNE_VULKAN_FELL_BACK";
    /// A file larger than this is not read (one record is well under 1 KB).
    const MAX_BYTES: u64 = 16 * 1024;

    /// One Vulkan failure to open the window.
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub(crate) struct Failure {
        /// The build that failed (`sysinfo::version_string`).
        pub app: String,
        /// The Vulkan loader then (`loader_text`): a new one tries Vulkan again.
        pub loader: String,
        pub adapter: String,
        pub reason: String,
        /// When, local time, for a reader of the file.
        #[serde(default)]
        pub when: String,
    }

    /// This machine's Vulkan loader as the record names it: its path and version, or "none".
    pub(crate) fn loader_text() -> String {
        super::loader::this_machine().map_or_else(|| "none".to_string(), |l| l.line())
    }

    /// Whether `f` is about this build and this loader.
    pub(crate) fn applies(f: &Failure, app: &str, loader: &str) -> bool {
        f.app == app && f.loader == loader
    }

    pub(crate) fn parse(text: &str) -> Option<Failure> {
        let f: Failure = toml::from_str(text).ok()?;
        // Read as untrusted: bounded fields, or nothing.
        let ok = [&f.app, &f.loader, &f.adapter, &f.reason, &f.when].iter().all(|s| s.len() <= 1024);
        ok.then_some(f)
    }

    pub(crate) fn render(f: &Failure) -> Result<String, String> {
        let body = toml::to_string(f).map_err(|e| e.to_string())?;
        Ok(format!(
            "# Vulkan could not open Fractadyne's window here, so it runs on OpenGL. Written by the app;\n\
             # delete this file to try Vulkan again (a new version or a new Vulkan loader also does).\n\n{body}"
        ))
    }

    /// The failure saved for this build and this loader, if any (read once a run).
    pub(crate) fn remembered() -> Option<&'static Failure> {
        static READ: std::sync::OnceLock<Option<Failure>> = std::sync::OnceLock::new();
        READ.get_or_init(|| {
            let path = fractadyne_state::config_dir()?.join(FILE);
            if !std::fs::metadata(&path).is_ok_and(|m| m.len() <= MAX_BYTES) {
                return None;
            }
            let f = parse(&std::fs::read_to_string(&path).ok()?)?;
            applies(&f, &crate::sysinfo::version_string(), &loader_text()).then_some(f)
        })
        .as_ref()
    }

    /// Write `f` to the settings folder, whole or not at all.
    fn record(f: &Failure) -> Result<(), String> {
        let dir = fractadyne_state::config_dir().ok_or("no settings folder")?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join(FILE);
        let part = dir.join(format!("{FILE}.part"));
        std::fs::write(&part, render(f)?).and_then(|()| std::fs::rename(&part, &path)).map_err(|e| {
            let _ = std::fs::remove_file(&part);
            e.to_string()
        })
    }

    /// The window could not open (eframe's error, `reason`): when that was on a Vulkan adapter this
    /// run chose by itself, remember it and, outside a task, start this program again on OpenGL.
    /// `true` = started again (the caller shows no fatal dialog: the new session explains).
    pub(crate) fn after_window_failure(reason: &str) -> bool {
        if eframe::wgpu::Backends::from_env().is_some() || !super::backends().contains(eframe::wgpu::Backends::VULKAN) {
            return false;
        }
        let Some(gpu) = crate::diag::gpu_seen().filter(|g| g.api == "Vulkan") else { return false };
        let f = Failure {
            app: crate::sysinfo::version_string(),
            loader: loader_text(),
            adapter: gpu.name.clone(),
            reason: reason.chars().take(400).collect(),
            when: crate::sysinfo::local_now()
                .map_or_else(String::new, |(y, mo, d, h, mi, ..)| format!("{y}-{mo:02}-{d:02} {h:02}:{mi:02}")),
        };
        let saved = match record(&f) {
            Ok(()) => format!("saved to {FILE}"),
            Err(e) => format!("not saved ({e})"),
        };
        if crate::launched_as_task() {
            crate::diag::log_line(
                "gpu",
                &format!("Vulkan could not open the window on {}; {saved}: the next run starts on OpenGL (a task is not started again)", gpu.name),
            );
            return false;
        }
        match start_again(&[("WGPU_BACKEND", "gl"), (RELAUNCH_ENV, &f.reason)]) {
            Ok(pid) => {
                crate::diag::log_line(
                    "gpu",
                    &format!("Vulkan could not open the window on {}; {saved}; started again on OpenGL (process {pid})", gpu.name),
                );
                true
            }
            Err(e) => {
                crate::diag::log_line(
                    "gpu",
                    &format!("Vulkan could not open the window on {}; {saved}; could not start again on OpenGL: {e}", gpu.name),
                );
                false
            }
        }
    }

    /// In the session started again on OpenGL ([`RELAUNCH_ENV`]): the failure's reason, once.
    pub(crate) fn fell_back() -> Option<String> {
        std::env::var(RELAUNCH_ENV).ok().filter(|r| !r.is_empty()).map(|r| r.chars().take(400).collect())
    }

    /// Start this program again as it was started, with `vars` set; the new process's id.
    ///
    /// On Windows the new process gets this one's own command line and start-up information, so
    /// its window opens where this one's would have: on the monitor Explorer or the Start menu
    /// named, as a shortcut's "Run: Maximized" asks, at a test's chosen spot. A process started
    /// with `std::process::Command` gets none of that, and its window opens on the main monitor.
    fn start_again(vars: &[(&str, &str)]) -> Result<u32, String> {
        let exe = std::env::current_exe().map_err(|e| format!("cannot find this executable: {e}"))?;
        #[cfg(windows)]
        {
            same_start::spawn(&exe, vars)
        }
        #[cfg(not(windows))]
        {
            let mut cmd = std::process::Command::new(exe);
            cmd.args(std::env::args_os().skip(1));
            for (k, v) in vars {
                cmd.env(k, v);
            }
            cmd.spawn().map(|c| c.id()).map_err(|e| e.to_string())
        }
    }

    /// [`start_again`] on Windows: `CreateProcessW` with this process's own command line and
    /// `STARTUPINFO`.
    #[cfg(windows)]
    pub(crate) mod same_start {
        use std::ffi::{c_void, OsStr, OsString};
        use std::os::windows::ffi::OsStrExt;

        #[repr(C)]
        struct StartupInfo {
            cb: u32,
            reserved: *mut u16,
            desktop: *mut u16,
            title: *mut u16,
            x: u32,
            y: u32,
            x_size: u32,
            y_size: u32,
            x_chars: u32,
            y_chars: u32,
            fill: u32,
            flags: u32,
            show_window: u16,
            reserved2_len: u16,
            reserved2: *mut u8,
            std_input: isize,
            std_output: isize,
            std_error: isize,
        }

        #[repr(C)]
        struct ProcessInformation {
            process: isize,
            thread: isize,
            process_id: u32,
            thread_id: u32,
        }

        const STARTF_USESTDHANDLES: u32 = 0x100;
        const CREATE_UNICODE_ENVIRONMENT: u32 = 0x400;

        unsafe extern "system" {
            fn GetStartupInfoW(si: *mut StartupInfo);
            fn GetCommandLineW() -> *const u16;
            #[allow(clippy::too_many_arguments)]
            fn CreateProcessW(
                application: *const u16,
                command_line: *mut u16,
                process_attributes: *const c_void,
                thread_attributes: *const c_void,
                inherit_handles: i32,
                creation_flags: u32,
                environment: *const c_void,
                current_directory: *const u16,
                startup_info: *const StartupInfo,
                process_information: *mut ProcessInformation,
            ) -> i32;
            // Handles as `isize`, as the app's other declarations of these have them.
            fn CloseHandle(h: isize) -> i32;
        }

        pub(crate) fn spawn(exe: &std::path::Path, vars: &[(&str, &str)]) -> Result<u32, String> {
            let application: Vec<u16> = exe.as_os_str().encode_wide().chain([0]).collect();
            // This process's command line as it was given: the same arguments, quoted as they were.
            // SAFETY: the system's own string for this process, ended by a 0, valid while it runs.
            let mut command_line: Vec<u16> = unsafe {
                let p = GetCommandLineW();
                let mut n = 0;
                while *p.add(n) != 0 {
                    n += 1;
                }
                std::slice::from_raw_parts(p, n).to_vec()
            };
            command_line.push(0);
            let environment = block(std::env::vars_os(), vars);
            // SAFETY: plain data, filled by the system; the strings it points to are this
            // process's own, valid while it runs (`CreateProcessW` copies them).
            let mut si: StartupInfo = unsafe { std::mem::zeroed() };
            unsafe { GetStartupInfoW(&mut si) };
            si.cb = std::mem::size_of::<StartupInfo>() as u32;
            // Not passed on: the C runtime's private block, and this process's standard handles,
            // which are not inherited and so would name nothing in the new one. (Without that
            // flag the handle fields can carry the monitor or a hotkey, which are kept.)
            si.reserved = std::ptr::null_mut();
            si.reserved2 = std::ptr::null_mut();
            si.reserved2_len = 0;
            if si.flags & STARTF_USESTDHANDLES != 0 {
                si.flags &= !STARTF_USESTDHANDLES;
                si.std_input = 0;
                si.std_output = 0;
                si.std_error = 0;
            }
            // SAFETY: plain data, filled by the system.
            let mut pi: ProcessInformation = unsafe { std::mem::zeroed() };
            // SAFETY: every string is ended by a 0 and outlives the call; the environment block
            // is UTF-16 as the flag says.
            let ok = unsafe {
                CreateProcessW(
                    application.as_ptr(),
                    command_line.as_mut_ptr(),
                    std::ptr::null(),
                    std::ptr::null(),
                    0,
                    CREATE_UNICODE_ENVIRONMENT,
                    environment.as_ptr().cast(),
                    std::ptr::null(),
                    &si,
                    &mut pi,
                )
            };
            if ok == 0 {
                return Err(std::io::Error::last_os_error().to_string());
            }
            // SAFETY: the handles `CreateProcessW` just returned, closed once.
            unsafe {
                CloseHandle(pi.thread);
                CloseHandle(pi.process);
            }
            Ok(pi.process_id)
        }

        /// An environment block for `CreateProcessW`: `base` with `vars` set (a name in `base`
        /// that differs from one of them only in case is replaced), as `NAME=value` strings each
        /// ended by a 0, sorted by name ignoring case as the system asks, then a final 0.
        pub(crate) fn block(base: impl IntoIterator<Item = (OsString, OsString)>, vars: &[(&str, &str)]) -> Vec<u16> {
            let set = |name: &OsStr| vars.iter().any(|(k, _)| name.eq_ignore_ascii_case(k));
            let mut all: Vec<(OsString, OsString)> = base.into_iter().filter(|(k, _)| !set(k)).collect();
            all.extend(vars.iter().map(|(k, v)| (OsString::from(k), OsString::from(v))));
            all.sort_by_cached_key(|(k, _)| k.to_string_lossy().to_uppercase());
            let mut out = Vec::new();
            for (k, v) in &all {
                out.extend(k.encode_wide());
                out.push(u16::from(b'='));
                out.extend(v.encode_wide());
                out.push(0);
            }
            out.push(0);
            out
        }
    }
}

/// This machine's Vulkan loader, as the app would load it (see [`vulkan_left_out`]).
pub(crate) mod loader {
    /// What [`read`] found.
    #[derive(Clone, Debug, PartialEq)]
    pub(crate) struct Loader {
        /// The file the system loaded (the same search wgpu's `vulkan-1.dll` load makes).
        pub path: String,
        /// Its file version (major, minor, patch, build), where the file states one.
        pub version: Option<[u16; 4]>,
        /// It exports `vkEnumerateInstanceVersion`: a Vulkan 1.1 loader or later.
        pub api_1_1: bool,
    }

    /// [`read`], once a run.
    pub(crate) fn this_machine() -> Option<&'static Loader> {
        static READ: std::sync::OnceLock<Option<Loader>> = std::sync::OnceLock::new();
        READ.get_or_init(read).as_ref()
    }

    impl Loader {
        /// The log's account of it: `C:\…\vulkan-1.dll 1.3.290.0 (Vulkan 1.1 or later)`.
        pub(crate) fn line(&self) -> String {
            let v = self.version.map_or_else(|| "no version stated".to_string(), |v| format!("{}.{}.{}.{}", v[0], v[1], v[2], v[3]));
            format!("{} {v} ({})", self.path, if self.api_1_1 { "Vulkan 1.1 or later" } else { "Vulkan 1.0" })
        }

        /// Why Vulkan is left out with this loader, or `None` to keep it.
        pub(crate) fn too_old(&self) -> Option<String> {
            if self.api_1_1 {
                return None;
            }
            let version = self.version.map_or_else(|| "no version stated".to_string(), |v| format!("version {}.{}.{}.{}", v[0], v[1], v[2], v[3]));
            Some(format!(
                "this machine's Vulkan loader ({}, {version}) is a Vulkan 1.0 loader, from before 2018, and \
                 listing graphics cards through one crashed the app (a 2016 loader, 1.0.26, on a GeForce GT 650M), \
                 so this run uses OpenGL. A newer graphics driver brings a newer loader; WGPU_BACKEND=vulkan \
                 tries Vulkan anyway.",
                self.path
            ))
        }
    }

    /// The loader `vulkan-1.dll` the system finds (`LoadLibraryW`, the search wgpu's own load makes),
    /// read for its exports and file version, then released. `None` = no loader (no Vulkan driver;
    /// wgpu then finds no Vulkan cards by itself, which is safe) or not Windows.
    #[cfg(windows)]
    pub(crate) fn read() -> Option<Loader> {
        use core::ffi::c_void;
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn LoadLibraryW(name: *const u16) -> *mut c_void;
            fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
            fn GetModuleFileNameW(module: *mut c_void, name: *mut u16, size: u32) -> u32;
            fn FreeLibrary(handle: *mut c_void) -> i32;
        }
        let name: Vec<u16> = "vulkan-1.dll".encode_utf16().chain([0]).collect();
        // SAFETY: a NUL-terminated name. Loading the loader runs only its DllMain, which wgpu's own
        // load of it runs moments later anyway; no Vulkan function is called.
        let h = unsafe { LoadLibraryW(name.as_ptr()) };
        if h.is_null() {
            return None;
        }
        // SAFETY: a live module handle and a NUL-terminated export name; the result is only tested.
        let api_1_1 = !unsafe { GetProcAddress(h, c"vkEnumerateInstanceVersion".as_ptr().cast()) }.is_null();
        let mut buf = vec![0u16; 1024];
        // SAFETY: the buffer's length in u16s is passed; the call writes at most that many.
        let n = unsafe { GetModuleFileNameW(h, buf.as_mut_ptr(), buf.len() as u32) } as usize;
        let path = String::from_utf16_lossy(&buf[..n.min(buf.len())]);
        // SAFETY: the handle this function loaded, released once.
        unsafe { FreeLibrary(h) };
        let version = if n > 0 { file_version(&buf[..n]) } else { None };
        Some(Loader { path: if path.is_empty() { "vulkan-1.dll".into() } else { path }, version, api_1_1 })
    }

    #[cfg(not(windows))]
    pub(crate) fn read() -> Option<Loader> {
        None
    }

    /// A file's version (`VS_FIXEDFILEINFO.dwFileVersionMS/LS`), from the path `wide` (no NUL).
    #[cfg(windows)]
    fn file_version(wide: &[u16]) -> Option<[u16; 4]> {
        use core::ffi::c_void;
        #[link(name = "version")]
        unsafe extern "system" {
            fn GetFileVersionInfoSizeW(file: *const u16, handle: *mut u32) -> u32;
            fn GetFileVersionInfoW(file: *const u16, handle: u32, len: u32, data: *mut c_void) -> i32;
            fn VerQueryValueW(block: *const c_void, sub: *const u16, buf: *mut *mut c_void, len: *mut u32) -> i32;
        }
        let file: Vec<u16> = wide.iter().copied().chain([0]).collect();
        let mut ignored = 0u32;
        // SAFETY: a NUL-terminated path and an out-parameter.
        let size = unsafe { GetFileVersionInfoSizeW(file.as_ptr(), &mut ignored) };
        if size == 0 {
            return None;
        }
        let mut block = vec![0u8; size as usize];
        // SAFETY: the block's length is passed; the call writes at most that many bytes.
        if unsafe { GetFileVersionInfoW(file.as_ptr(), 0, size, block.as_mut_ptr().cast()) } == 0 {
            return None;
        }
        let root: Vec<u16> = "\\".encode_utf16().chain([0]).collect();
        let (mut p, mut len) = (std::ptr::null_mut::<c_void>(), 0u32);
        // SAFETY: the block GetFileVersionInfoW filled; the result points inside it.
        if unsafe { VerQueryValueW(block.as_ptr().cast(), root.as_ptr(), &mut p, &mut len) } == 0 || p.is_null() || len < 52 {
            return None;
        }
        // VS_FIXEDFILEINFO: signature, struct version, then the file version's MS and LS words.
        // SAFETY: `len` (≥ 52, the structure's size) bytes at `p`, inside `block`.
        let word = |k: usize| unsafe { std::ptr::read_unaligned((p as *const u32).add(k)) };
        if word(0) != 0xFEEF_04BD {
            return None;
        }
        let (ms, ls) = (word(2), word(3));
        Some([(ms >> 16) as u16, ms as u16, (ls >> 16) as u16, ls as u16])
    }
}

/// One adapter, as `--list-adapters` prints it.
pub(crate) struct Row {
    pub name: String,
    pub backend: wgpu::Backend,
    pub kind: wgpu::DeviceType,
    pub driver: String,
}

impl Row {
    fn of(info: &wgpu::AdapterInfo) -> Row {
        Row { name: info.name.clone(), backend: info.backend, kind: info.device_type, driver: crate::farm::driver_text(&info.driver, &info.driver_info) }
    }

    /// A card that renders on its own silicon (not a software rasterizer).
    pub(crate) fn is_hardware(&self) -> bool {
        matches!(self.kind, wgpu::DeviceType::DiscreteGpu | wgpu::DeviceType::IntegratedGpu)
    }
}

/// Every adapter this machine offers on [`backends`], in wgpu's order.
pub(crate) fn list() -> Vec<Row> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: backends(), ..Default::default() });
    instance.enumerate_adapters(backends()).iter().map(|a| Row::of(&a.get_info())).collect()
}

fn kind_text(k: wgpu::DeviceType) -> &'static str {
    match k {
        wgpu::DeviceType::DiscreteGpu => "discrete",
        wgpu::DeviceType::IntegratedGpu => "integrated",
        wgpu::DeviceType::VirtualGpu => "virtual",
        wgpu::DeviceType::Cpu => "software",
        wgpu::DeviceType::Other => "other",
    }
}

/// The listing, one adapter a line: `N  name · API · kind · driver`.
pub(crate) fn listing(rows: &[Row]) -> String {
    if rows.is_empty() {
        return "No graphics adapter found (Vulkan or OpenGL).\n".into();
    }
    let mut s = String::new();
    for (i, r) in rows.iter().enumerate() {
        s.push_str(&format!("{:>2}  {} · {:?} · {} · {}\n", i + 1, r.name, r.backend, kind_text(r.kind), if r.driver.is_empty() { "driver not reported" } else { &r.driver }));
    }
    s
}

/// The graphics cards in a `--list-adapters` listing — the Vulkan entries of real cards (what
/// `--adapters all` takes) — as (number, name). How the Render client window learns them: from a
/// child process, so the app itself never enumerates OpenGL beside its own device.
pub(crate) fn cards_in_listing(text: &str) -> Vec<(usize, String)> {
    text.lines()
        .filter_map(|l| {
            let (n, rest) = l.trim_start().split_once("  ")?;
            let mut parts = rest.split(" · ");
            let (name, api, kind) = (parts.next()?, parts.next()?, parts.next()?);
            if api != "Vulkan" || !matches!(kind, "discrete" | "integrated") {
                return None;
            }
            Some((n.trim().parse().ok()?, name.to_string()))
        })
        .collect()
}

/// Which of `rows` (name, API) the spec names: a 1-based number, or part of a name. A number of
/// three digits or more is a model number, part of a name — `--adapter 6800` means the RX 6800 —
/// while a smaller one is only ever a position: `--adapter 0` is an error, never "any name with a
/// 0 in it".
pub(crate) fn pick(spec: &str, rows: &[(String, wgpu::Backend)]) -> Result<usize, String> {
    let spec = spec.trim();
    if spec.is_empty() {
        return Err("--adapter needs a number from --list-adapters, or part of the adapter's name".into());
    }
    let number = spec.parse::<usize>().ok();
    if let Some(n) = number.filter(|&n| n < 100) {
        return if (1..=rows.len()).contains(&n) {
            Ok(n - 1)
        } else {
            Err(format!("--adapter {n}: this machine has {} adapter(s), numbered from 1 (see --list-adapters)", rows.len()))
        };
    }
    let want = spec.to_lowercase();
    let hits: Vec<usize> = (0..rows.len()).filter(|&i| rows[i].0.to_lowercase().contains(&want)).collect();
    hits.iter().copied().find(|&i| rows[i].1 == wgpu::Backend::Vulkan).or(hits.first().copied()).ok_or_else(|| match number {
        Some(_) => format!("--adapter {spec}: this machine has {} adapter(s), numbered from 1, and no adapter's name contains {spec} (see --list-adapters)", rows.len()),
        None => format!("--adapter \"{spec}\": no adapter's name contains that (see --list-adapters)"),
    })
}

/// The spec `--adapter` gives, if any. A flag without a value is an error, not "any adapter".
pub(crate) fn spec(args: &[String]) -> Result<Option<String>, String> {
    match args.iter().position(|a| a == FLAG) {
        None => Ok(None),
        Some(i) => match args.get(i + 1) {
            Some(v) if !v.starts_with("--") => Ok(Some(v.clone())),
            _ => Err("--adapter needs a number from --list-adapters, or part of the adapter's name".into()),
        },
    }
}

/// The window's adapter selector for `spec`: the adapter it names, provided it can draw to the
/// window. Checked before the window opens (see [`check`]), so this failing is a surface problem.
pub(crate) fn selector(spec: String) -> eframe::egui_wgpu::NativeAdapterSelectorMethod {
    std::sync::Arc::new(move |adapters: &[wgpu::Adapter], surface: Option<&wgpu::Surface<'_>>| {
        let rows: Vec<(String, wgpu::Backend)> = adapters.iter().map(|a| a.get_info()).map(|i| (i.name, i.backend)).collect();
        let i = pick(&spec, &rows)?;
        let a = &adapters[i];
        if let Some(s) = surface {
            if !a.is_surface_supported(s) {
                return Err(format!("--adapter {spec}: {} ({:?}) cannot draw to this window", rows[i].0, rows[i].1));
            }
        }
        Ok(a.clone())
    })
}

/// Resolve `spec` against this machine's adapters before anything opens, so a wrong one is an
/// error with the list, not a window that fails to start. Returns the adapter's line.
pub(crate) fn check(spec: &str) -> Result<String, String> {
    let rows = list();
    let pairs: Vec<(String, wgpu::Backend)> = rows.iter().map(|r| (r.name.clone(), r.backend)).collect();
    match pick(spec, &pairs) {
        Ok(i) => Ok(format!("{} · {:?}", rows[i].name, rows[i].backend)),
        Err(e) => Err(format!("{e}\nThis machine's adapters:\n{}", listing(&rows))),
    }
}

#[cfg(test)]
#[path = "../tests/unit/gpu_choice/gpu_choice_tests.rs"]
mod gpu_choice_tests;
