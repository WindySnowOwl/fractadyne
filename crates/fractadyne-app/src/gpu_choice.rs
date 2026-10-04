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

/// The graphics APIs the app runs on: Vulkan and GL, unless `WGPU_BACKEND` names others. One
/// definition for the window's instance and for the listing, so their enumerations agree.
pub(crate) fn backends() -> wgpu::Backends {
    wgpu::Backends::from_env().unwrap_or(wgpu::Backends::VULKAN | wgpu::Backends::GL)
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
mod gpu_choice_tests;
