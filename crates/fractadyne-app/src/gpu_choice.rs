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
/// alone where this machine's Vulkan loader is too old to list cards through ([`vulkan_left_out`]).
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
/// `WGPU_BACKEND` set = nothing is left out (and the loader is not read): `WGPU_BACKEND=vulkan`
/// tries Vulkan anyway. Windows only; elsewhere `None`.
pub(crate) fn vulkan_left_out() -> Option<&'static str> {
    static WHY: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    WHY.get_or_init(|| {
        if wgpu::Backends::from_env().is_some() {
            return None;
        }
        loader::this_machine().and_then(|l| l.too_old())
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
