//! ⭐**WHAT A FATAL ERROR TELLS THE PERSON AT THE WINDOW, BEYOND THE ERROR ITSELF** (user,
//! 2026-11-09: "Ensure error messages are helpful and indicate where the failure happened and some
//! things to check or try"). The laptop Eris's crash said "a native crash: access violation
//! (0xc0000005) reading address 0xb1, in the graphics driver: ig75icd64.dll+0x733c8a": exact, and
//! of no use to someone who has never heard of ig75icd64.dll. Now the crash dialog and the console
//! also say, in plain words:
//! - what happened (the graphics driver crashed while it compiled Fractadyne's shaders),
//! - when (at start-up, preparing the graphics shaders; the shader; what the program was doing),
//! - on which graphics card, through which API and driver,
//! - and what to check or try, picked for that failure (a newer driver from the card's maker, the
//!   computer's other graphics card, the computer itself rather than Remote Desktop, starting again,
//!   a newer Fractadyne).
//!
//! Everything here is plain data and string replacement: it runs while the program dies (the panic
//! hook, the native crash's helper thread), so it takes no lock and calls no translation. The
//! phrases are translated ahead of time ([`Phrases`], built by `ui::crash_dialog_text`).

/// How far the program had got: set at start-up's steps (`diag::set_phase`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub(crate) enum Phase {
    /// Before start-up's first step (a crash in argument handling or the session's loading).
    Unknown = 0,
    /// eframe is opening the window.
    OpeningWindow = 1,
    /// The graphics device is being created, with the window's own (egui's) pipelines.
    StartingDevice = 2,
    /// The renderer's pipelines are being built: the driver compiles Fractadyne's shaders.
    PreparingShaders = 3,
    /// Start-up is over.
    Running = 4,
}

impl Phase {
    pub(crate) fn from_u8(v: u8) -> Phase {
        match v {
            1 => Phase::OpeningWindow,
            2 => Phase::StartingDevice,
            3 => Phase::PreparingShaders,
            4 => Phase::Running,
            _ => Phase::Unknown,
        }
    }

    fn at_start_up(self) -> bool {
        matches!(self, Phase::OpeningWindow | Phase::StartingDevice | Phase::PreparingShaders)
    }
}

/// What failed.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Failure {
    /// A Rust panic with this message: an error Fractadyne's own code stopped on.
    Panic(String),
    /// A crash that is not a panic (an access violation, say), in this module (a file name), if
    /// the address was in one.
    Native { module: Option<String> },
    /// The window could not open (eframe's error).
    WindowOpen,
    /// The window opened without a graphics device.
    NoDevice,
    /// The graphics driver offers less than Fractadyne needs.
    Unsupported,
}

/// The graphics card the window's device was opened on, as the dialog names it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct Gpu {
    pub(crate) name: String,
    /// The graphics API, as people know it: "OpenGL", "Vulkan", "Direct3D 12".
    pub(crate) api: &'static str,
    /// The driver's version as the API reports it ("4.3.0 - Build 20.19.15.4531"); may be empty.
    pub(crate) driver: String,
    /// Who makes the card, when known: the driver to install comes from them.
    pub(crate) vendor: Option<&'static str>,
}

/// The graphics API's name as people know it.
pub(crate) fn api_name(backend: eframe::wgpu::Backend) -> &'static str {
    use eframe::wgpu::Backend;
    match backend {
        Backend::Vulkan => "Vulkan",
        Backend::Metal => "Metal",
        Backend::Dx12 => "Direct3D 12",
        Backend::Gl => "OpenGL",
        Backend::BrowserWebGpu => "WebGPU",
        Backend::Empty => "no graphics API",
    }
}

/// The card's maker, from its PCI vendor id or, failing that (an OpenGL adapter may report none),
/// its name.
pub(crate) fn vendor_name(vendor_id: u32, card: &str) -> Option<&'static str> {
    match vendor_id {
        0x8086 => return Some("Intel"),
        0x10DE => return Some("NVIDIA"),
        0x1002 | 0x1022 => return Some("AMD"),
        0x5143 => return Some("Qualcomm"),
        _ => {}
    }
    let c = card.to_ascii_lowercase();
    if c.contains("intel") {
        Some("Intel")
    } else if c.contains("nvidia") || c.contains("geforce") || c.contains("quadro") {
        Some("NVIDIA")
    } else if c.contains("amd") || c.contains("radeon") {
        Some("AMD")
    } else if c.contains("qualcomm") || c.contains("adreno") {
        Some("Qualcomm")
    } else {
        None
    }
}

/// The driver's version, from wgpu's two fields ("NVIDIA" and "566.36"; "" and "4.3.0 - Build
/// 20.19.15.4531" on OpenGL).
pub(crate) fn driver_text(driver: &str, driver_info: &str) -> String {
    [driver.trim(), driver_info.trim()].iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(" ")
}

/// Everything known about the failure when it is explained.
pub(crate) struct Situation<'a> {
    pub(crate) failure: &'a Failure,
    pub(crate) phase: Phase,
    /// The pipeline compiles in flight (`fractadyne_gpu::pipeline_compile::compiling_now`).
    pub(crate) compiling: &'a [String],
    pub(crate) gpu: Option<&'a Gpu>,
    /// What the program was doing (the breadcrumb), if known.
    pub(crate) activity: Option<&'a str>,
    /// Whether this is a Remote Desktop session (often without graphics acceleration).
    pub(crate) remote_session: bool,
}

/// The one-sentence account of what happened.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Summary {
    /// The graphics driver crashed while it compiled Fractadyne's shaders.
    DriverWhileCompiling,
    /// The graphics driver crashed.
    Driver,
    /// The graphics card stopped responding (a device loss).
    DeviceLost,
    /// A crash in another module.
    InModule,
    /// A crash in no known module.
    Native,
    /// An error in Fractadyne's own code.
    Bug,
    WindowOpen,
    NoDevice,
    Unsupported,
}

/// Something to check or try.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Tip {
    /// Remote Desktop often has no graphics acceleration: start Fractadyne at the computer.
    RemoteDesktop,
    /// Install the newest graphics driver from the card's maker.
    UpdateDriver,
    /// Try the computer's other graphics card (`--list-adapters`, `--adapter N`).
    OtherCard,
    /// Start Fractadyne again.
    StartAgain,
    /// A newer Fractadyne may have fixed it.
    NewerFractadyne,
}

/// Whether a panic's message is a lost graphics device (the driver reset the card).
pub(crate) fn device_lost(msg: &str) -> bool {
    !crate::diag::device_loss_hint(msg).is_empty()
}

fn driver_module(f: &Failure) -> bool {
    matches!(f, Failure::Native { module: Some(m) } if super::native_crash::graphics_module(m))
}

pub(crate) fn summary(s: &Situation) -> Summary {
    match s.failure {
        Failure::Native { .. } if driver_module(s.failure) => {
            if !s.compiling.is_empty() || s.phase == Phase::PreparingShaders {
                Summary::DriverWhileCompiling
            } else {
                Summary::Driver
            }
        }
        Failure::Native { module: Some(_) } => Summary::InModule,
        Failure::Native { module: None } => Summary::Native,
        Failure::Panic(msg) if device_lost(msg) => Summary::DeviceLost,
        Failure::Panic(_) => Summary::Bug,
        Failure::WindowOpen => Summary::WindowOpen,
        Failure::NoDevice => Summary::NoDevice,
        Failure::Unsupported => Summary::Unsupported,
    }
}

/// What to check or try, most likely first.
pub(crate) fn tips(s: &Situation) -> Vec<Tip> {
    let sum = summary(s);
    // The graphics side: a driver crash, a lost card, no window or device, or anything while the
    // device or its shaders were being set up.
    let graphics = matches!(
        sum,
        Summary::DriverWhileCompiling | Summary::Driver | Summary::DeviceLost | Summary::WindowOpen | Summary::NoDevice | Summary::Unsupported
    ) || s.phase.at_start_up()
        || !s.compiling.is_empty();
    let mut t = Vec::new();
    if graphics {
        if s.remote_session {
            t.push(Tip::RemoteDesktop);
        }
        t.push(Tip::UpdateDriver);
        t.push(Tip::OtherCard);
    }
    // Mid-session, it may not happen twice; at start-up it will, every time.
    if s.phase == Phase::Running && sum != Summary::Unsupported {
        t.push(Tip::StartAgain);
    }
    if sum != Summary::Unsupported {
        t.push(Tip::NewerFractadyne);
    }
    t
}

/// The dialog's phrases in the language in use, with placeholders filled in at the crash:
/// `{module}`, `{phase}`, `{shader}`, `{activity}`, `{card}`, `{api}`, `{driver}`, `{vendor}`.
#[derive(Clone, Debug, Default)]
pub(crate) struct Phrases {
    pub(crate) summary_driver_compiling: String,
    pub(crate) summary_driver: String,
    pub(crate) summary_device_lost: String,
    pub(crate) summary_module: String,
    pub(crate) summary_native: String,
    pub(crate) summary_bug: String,
    pub(crate) summary_window: String,
    pub(crate) summary_no_device: String,
    pub(crate) summary_unsupported: String,
    pub(crate) phase_window: String,
    pub(crate) phase_device: String,
    pub(crate) phase_shaders: String,
    pub(crate) phase_running: String,
    /// "When: {phase}"
    pub(crate) when: String,
    /// "Shader being compiled: {shader}"
    pub(crate) shader: String,
    /// "Doing: {activity}"
    pub(crate) activity: String,
    /// "Graphics: {card} ({api}, driver {driver})"
    pub(crate) graphics: String,
    /// "Graphics: {card} ({api})"
    pub(crate) graphics_no_driver: String,
    pub(crate) tip_remote: String,
    /// With `{vendor}`.
    pub(crate) tip_driver_vendor: String,
    pub(crate) tip_driver: String,
    pub(crate) tip_other_card: String,
    pub(crate) tip_again: String,
    pub(crate) tip_newer: String,
}

/// The account, ready for the dialog's `{what}` and `{tips}`: the summary and the detail lines,
/// and the tips as a bulleted list.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct Account {
    pub(crate) what: String,
    pub(crate) tips: String,
}

pub(crate) fn compose(s: &Situation, p: &Phrases) -> Account {
    let module = match s.failure {
        Failure::Native { module: Some(m) } => m.as_str(),
        _ => "",
    };
    let card = s.gpu.map(|g| g.name.as_str()).unwrap_or("");
    let fill = |t: &str| {
        t.replace("{module}", module)
            .replace("{card}", card)
            .replace("{api}", s.gpu.map(|g| g.api).unwrap_or(""))
            .replace("{driver}", s.gpu.map(|g| g.driver.as_str()).unwrap_or(""))
            .replace("{vendor}", s.gpu.and_then(|g| g.vendor).unwrap_or(""))
    };
    let mut lines = vec![fill(match summary(s) {
        Summary::DriverWhileCompiling => &p.summary_driver_compiling,
        Summary::Driver => &p.summary_driver,
        Summary::DeviceLost => &p.summary_device_lost,
        Summary::InModule => &p.summary_module,
        Summary::Native => &p.summary_native,
        Summary::Bug => &p.summary_bug,
        Summary::WindowOpen => &p.summary_window,
        Summary::NoDevice => &p.summary_no_device,
        Summary::Unsupported => &p.summary_unsupported,
    })];
    let phase = match s.phase {
        Phase::Unknown => None,
        Phase::OpeningWindow => Some(&p.phase_window),
        Phase::StartingDevice => Some(&p.phase_device),
        Phase::PreparingShaders => Some(&p.phase_shaders),
        Phase::Running => Some(&p.phase_running),
    };
    if let Some(ph) = phase {
        lines.push(p.when.replace("{phase}", ph));
    }
    if !s.compiling.is_empty() {
        lines.push(p.shader.replace("{shader}", &s.compiling.join("; ")));
    }
    // What the program was doing, mid-session (a start-up step is already the phase).
    if let Some(a) = s.activity.filter(|a| !a.is_empty() && s.phase == Phase::Running) {
        lines.push(p.activity.replace("{activity}", a));
    }
    if let Some(g) = s.gpu {
        lines.push(fill(if g.driver.is_empty() { &p.graphics_no_driver } else { &p.graphics }));
    }
    let tips = tips(s)
        .into_iter()
        .map(|t| {
            let text = match t {
                Tip::RemoteDesktop => fill(&p.tip_remote),
                Tip::UpdateDriver if s.gpu.and_then(|g| g.vendor).is_some() => fill(&p.tip_driver_vendor),
                Tip::UpdateDriver => fill(&p.tip_driver),
                Tip::OtherCard => fill(&p.tip_other_card),
                Tip::StartAgain => fill(&p.tip_again),
                Tip::NewerFractadyne => fill(&p.tip_newer),
            };
            format!("• {text}")
        })
        .collect::<Vec<_>>()
        .join("\n");
    Account { what: lines.join("\n"), tips }
}
