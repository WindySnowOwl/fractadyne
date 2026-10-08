//! A SECOND GPU for the live view (`design/multi-gpu-live.md`): a headless device on another
//! adapter — or a TWIN device on the window's own adapter, the byte-identity rig, which buys no
//! speed but runs every line of this path — with its own thread, rendering whole jobs through the
//! device-agnostic export renderer. The window's device presents everything; only whole products
//! come back. Losing the worker never ends the process: it marks itself dead and the live view
//! carries on with one GPU.
//!
//! First use (L2): progressive supersampling on settle. A job is one jittered sample of the
//! settled frame (`params_to_request_exact`); the result is that sample's colour as floats, for
//! `MandelbrotParams::accum_external` — the window's device rounds it to 8 bits itself, as it does
//! its own samples (see `fractadyne_gpu::AccumSample`).

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering::Relaxed, Ordering::SeqCst};
use std::sync::{mpsc, Arc};

use eframe::wgpu;

/// One job: render `req` (a whole frame) for view `view`'s accumulation run `run`, sample `index`.
pub(crate) struct Job {
    pub(crate) view: usize,
    pub(crate) run: u64,
    pub(crate) index: u32,
    pub(crate) req: fractadyne_gpu::ExportRequest,
}

/// A finished (or failed, or cancelled) [`Job`]. Every accepted job answers exactly once, so the
/// caller can always tell its sample index was not rendered.
pub(crate) struct Done {
    pub(crate) view: usize,
    pub(crate) run: u64,
    pub(crate) index: u32,
    pub(crate) sample: Option<Arc<fractadyne_gpu::AccumSample>>,
    pub(crate) ms: f64,
    pub(crate) err: Option<String>,
}

pub(crate) struct Worker {
    tx: mpsc::Sender<Job>,
    rx: mpsc::Receiver<Done>,
    /// The job being cancelled, as [`job_key`]; with `cancel`, the flag the render polls.
    cancelled: Arc<AtomicU64>,
    cancel: Arc<AtomicBool>,
    alive: Arc<AtomicBool>,
    /// The worker's adapter, as the log names it.
    pub(crate) name: String,
}

impl Worker {
    /// Open the worker device and start its thread. `spec`: `same` = the window's adapter again (a
    /// twin device), else a number or name as `--adapter` takes them (`gpu_choice::pick`).
    pub(crate) fn spawn(spec: &str, window: &wgpu::AdapterInfo) -> Result<Worker, String> {
        Self::spawn_on(spec, window, crate::gpu_choice::backends())
    }

    /// [`Self::spawn`] over `backends` only. The setting passes Vulkan alone: its number comes from
    /// a `--list-adapters` listing (`gpu_choice::cards_in_listing`, Vulkan cards only), and wgpu
    /// lists Vulkan adapters before OpenGL ones, so the number names the same card, while the app
    /// itself never enumerates OpenGL beside its own device (the farm client's rule).
    pub(crate) fn spawn_on(spec: &str, window: &wgpu::AdapterInfo, backends: wgpu::Backends) -> Result<Worker, String> {
        let Headless { device, queue, name, alive } = open_headless(spec, window, backends, "fractadyne.worker")?;
        // Test hook: `FRACTADYNE_WORKER_LOSE_AFTER=n` — the worker answers `n` jobs, then behaves as
        // a lost device (the path a real loss takes from there: answers fail, `alive` reads false).
        let lose_after = match std::env::var("FRACTADYNE_WORKER_LOSE_AFTER") {
            Err(_) => None,
            Ok(v) => match v.trim().parse::<u32>() {
                Ok(n) => Some(n),
                Err(_) => {
                    crate::diag::log_line("worker", &format!("FRACTADYNE_WORKER_LOSE_AFTER='{v}' is not a count — ignored"));
                    None
                }
            },
        };
        let (tx, jobs) = mpsc::channel::<Job>();
        let (done, rx) = mpsc::channel::<Done>();
        let cancel = Arc::new(AtomicBool::new(false));
        let cancelled = Arc::new(AtomicU64::new(u64::MAX));
        {
            let (cancel, cancelled, alive, name) = (cancel.clone(), cancelled.clone(), alive.clone(), name.clone());
            std::thread::Builder::new()
                .name("fd-gpu-worker".into())
                .spawn(move || {
                    let mut answered = 0u32;
                    for job in jobs {
                        if lose_after == Some(answered) && alive.swap(false, Relaxed) {
                            crate::diag::log_line(
                                "worker",
                                &format!("the worker GPU ({name}) is lost on purpose after {answered} jobs (FRACTADYNE_WORKER_LOSE_AFTER)"),
                            );
                        }
                        answered = answered.saturating_add(1);
                        // Clear the flag BEFORE reading the key (`cancel` writes them the other way
                        // round), so a cancel aimed at this job is never lost: either the key is
                        // seen here, or the flag is set after this store and the render sees it.
                        cancel.store(false, SeqCst);
                        let t = std::time::Instant::now();
                        let (sample, err) = if !alive.load(Relaxed) {
                            (None, Some("the worker GPU is gone".to_string()))
                        } else if cancelled.load(SeqCst) == job_key(job.view, job.run) {
                            (None, None) // cancelled while it waited in the queue
                        } else {
                            let progress = AtomicU32::new(0);
                            match fractadyne_gpu::render_export(&device, &queue, &job.req, &progress, &cancel) {
                                Ok(r) => (Some(Arc::new(sample_of(r))), None),
                                Err(fractadyne_gpu::GpuError::Canceled) => (None, None),
                                Err(e) => (None, Some(format!("{e}"))),
                            }
                        };
                        let ms = t.elapsed().as_secs_f64() * 1000.0;
                        let out = Done { view: job.view, run: job.run, index: job.index, sample, ms, err };
                        if done.send(out).is_err() {
                            break; // the app has gone
                        }
                    }
                })
                .map_err(|e| format!("the worker thread would not start: {e}"))?;
        }
        crate::diag::log_line("worker", &format!("worker GPU ready: {name} (spec '{spec}')"));
        Ok(Worker { tx, rx, cancelled, cancel, alive, name })
    }

    /// Queue `job`; `false` when the worker is gone (the caller renders that sample itself).
    pub(crate) fn submit(&self, job: Job) -> bool {
        self.alive() && self.tx.send(job).is_ok()
    }

    /// Abandon view `view`'s job of run `run`, queued or rendering (its [`Done`] still arrives,
    /// without a sample).
    pub(crate) fn cancel(&self, view: usize, run: u64) {
        self.cancelled.store(job_key(view, run), SeqCst);
        self.cancel.store(true, SeqCst);
    }

    pub(crate) fn try_recv(&self) -> Option<Done> {
        self.rx.try_recv().ok()
    }

    pub(crate) fn alive(&self) -> bool {
        self.alive.load(Relaxed)
    }
}

/// A headless graphics device on another adapter (or a twin on the window's own: `same`), whose
/// loss is ITS loss: `alive` turns false and a line is logged, nothing exits. Shared by the live
/// view's worker (`Worker::spawn_on`) and a still export split across cards
/// (`fractadyne_gpu::render_export_multi`).
pub(crate) struct Headless {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    /// The adapter, as the log names it.
    pub(crate) name: String,
    pub(crate) alive: Arc<AtomicBool>,
}

/// Open a [`Headless`] device on the adapter `spec` names (as `--adapter` takes it, see
/// `gpu_choice::pick`; `same` = the window's), with the window device's limits, so the same
/// renders fit. `label` names the device in driver diagnostics.
pub(crate) fn open_headless(spec: &str, window: &wgpu::AdapterInfo, backends: wgpu::Backends, label: &str) -> Result<Headless, String> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends, ..Default::default() });
    let adapters = instance.enumerate_adapters(backends);
    let adapter = if spec == "same" {
        adapters.into_iter().find(|a| {
            let i = a.get_info();
            i.name == window.name && i.backend == window.backend
        })
    } else {
        let rows: Vec<(String, wgpu::Backend)> =
            adapters.iter().map(|a| (a.get_info().name, a.get_info().backend)).collect();
        let k = crate::gpu_choice::pick(spec, &rows)?;
        adapters.into_iter().nth(k)
    }
    .ok_or_else(|| format!("no adapter matches '{spec}'"))?;
    let info = adapter.get_info();
    // The window device's requests (main.rs), so the same jobs fit: up to 1 GiB of reference,
    // the adapter's full texture size, and 64 attachment bytes for the chunked iterate.
    let al = adapter.limits();
    let base = wgpu::Limits::default();
    let want: u32 = 1 << 30;
    let limits = wgpu::Limits {
        max_texture_dimension_2d: al.max_texture_dimension_2d.max(base.max_texture_dimension_2d),
        max_storage_buffer_binding_size: al
            .max_storage_buffer_binding_size
            .min(want)
            .max(base.max_storage_buffer_binding_size),
        max_buffer_size: al.max_buffer_size.min(want as u64).max(base.max_buffer_size),
        max_color_attachment_bytes_per_sample: al
            .max_color_attachment_bytes_per_sample
            .min(64)
            .max(base.max_color_attachment_bytes_per_sample),
        ..base
    };
    let mut features = wgpu::Features::empty();
    if adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
        features |= wgpu::Features::TIMESTAMP_QUERY;
    }
    let (device, queue) = crate::gputest::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some(label),
            required_features: features,
            required_limits: limits,
            memory_hints: wgpu::MemoryHints::default(),
        },
        None,
    ))
    .map_err(|e| format!("a device on {} would not open: {e}", info.name))?;
    let name = format!("{} · {:?}", info.name, info.backend);
    let alive = Arc::new(AtomicBool::new(true));
    // ⭐Its loss is ITS loss: unlike the window's device (main.rs), nothing here exits.
    {
        let (alive, name) = (alive.clone(), name.clone());
        device.set_device_lost_callback(move |reason, msg| {
            alive.store(false, Relaxed);
            crate::diag::log_line(
                "worker",
                &format!("the second GPU ({name}) was lost ({reason:?}): {msg} — it is dropped, and the work carries on without it"),
            );
        });
    }
    {
        let (alive, name) = (alive.clone(), name.clone());
        device.on_uncaptured_error(Box::new(move |e| {
            alive.store(false, Relaxed);
            crate::diag::log_line("worker", &format!("the second GPU ({name}) failed: {e} — it is dropped"));
        }));
    }
    Ok(Headless { device, queue, name, alive })
}

/// Open every graphics card of this machine except the window's own (Vulkan, real hardware), for
/// an export split across them. A card that will not open is logged and left out. Enumerates
/// Vulkan only, so the app never enumerates OpenGL beside its own device; numbered as
/// [`open_headless`] picks them over the same enumeration.
pub(crate) fn open_other_cards(window: &wgpu::AdapterInfo) -> Vec<Headless> {
    let backends = crate::gpu_choice::backends() & wgpu::Backends::VULKAN;
    // Test hook: `FRACTADYNE_EXPORT_TWIN=1` — a second device on the window's own card, so a
    // one-card machine runs the Export dialog's split path end to end.
    if std::env::var_os("FRACTADYNE_EXPORT_TWIN").is_some() {
        return open_headless("same", window, backends, "fractadyne.export").map_or_else(
            |e| {
                crate::diag::log_line("render", &format!("export: FRACTADYNE_EXPORT_TWIN: {e}"));
                Vec::new()
            },
            |h| vec![h],
        );
    }
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends, ..Default::default() });
    let cards: Vec<(usize, String)> = instance
        .enumerate_adapters(backends)
        .iter()
        .map(|a| a.get_info())
        .enumerate()
        .filter(|(_, i)| matches!(i.device_type, wgpu::DeviceType::DiscreteGpu | wgpu::DeviceType::IntegratedGpu))
        .map(|(k, i)| (k + 1, i.name))
        .collect();
    drop(instance);
    let mut out = Vec::new();
    for k in export_extra_cards(&crate::farm::local::Gpus::All, &cards, &window.name) {
        match open_headless(&k.to_string(), window, backends, "fractadyne.export") {
            Ok(h) => out.push(h),
            Err(e) => crate::diag::log_line("render", &format!("export: graphics card {k} left out: {e}")),
        }
    }
    out
}

/// The cards a still export renders on BESIDE the window's own, for `--render … --gpus SPEC`
/// (`farm::local::parse_gpus`), as `--list-adapters` numbers for [`open_headless`]. `cards` are
/// this machine's Vulkan cards (number, name). The window's card always renders: `all` adds every
/// other card; a list adds what it names, the window's own card standing for the window the first
/// time it appears (a second mention opens a second device on it — the one-card machine's test of
/// the whole path). The window's card is found by name; with two of that name, the first.
pub(crate) fn export_extra_cards(gpus: &crate::farm::local::Gpus, cards: &[(usize, String)], window_name: &str) -> Vec<usize> {
    let mine = cards.iter().find(|(_, n)| n == window_name).map(|(k, _)| *k);
    match gpus {
        crate::farm::local::Gpus::All => cards.iter().map(|(k, _)| *k).filter(|&k| Some(k) != mine).collect(),
        crate::farm::local::Gpus::List(v) => {
            let mut v = v.clone();
            if let Some(i) = mine.and_then(|m| v.iter().position(|&k| k == m)) {
                v.remove(i);
            }
            v
        }
    }
}

/// Where the live view's second graphics card stands, for the setting's status line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WorkerState {
    /// No second card: the window's card renders every sample.
    Off,
    /// Its device is being opened, off the UI thread.
    Starting,
    /// Rendering samples; the adapter's name as the log gives it.
    Running(String),
    /// It would not open (the reason).
    Failed(String),
    /// It was lost or failed while in use; the view carries on with one card.
    Lost(String),
}

impl WorkerState {
    /// The setting's status line. `pinned`: `--worker-gpu` chose the card, not the setting.
    pub(crate) fn text(&self, pinned: bool) -> String {
        let s = match self {
            WorkerState::Off => "Off: this window's graphics card renders every sample.".to_string(),
            WorkerState::Starting => "Starting…".to_string(),
            WorkerState::Running(name) => format!("In use: {name}"),
            WorkerState::Failed(e) => format!("Could not start: {e}"),
            WorkerState::Lost(name) => {
                format!("Stopped: {name} was lost. One card from here; choose it again to retry.")
            }
        };
        if pinned {
            format!("{s} (set by --worker-gpu)")
        } else {
            s
        }
    }
}

/// The cards the setting offers: every card in `cards` (`gpu_choice::cards_in_listing`) except
/// the one drawing the window, `window_name`. A card is left out only when it is the ONLY one by
/// that name: with two identical cards neither can be told apart from the window's, so both stay
/// (choosing the window's own then opens a second device on it, which still works).
pub(crate) fn second_card_choices(cards: &[(usize, String)], window_name: &str) -> Vec<(usize, String)> {
    let same = cards.iter().filter(|(_, n)| n == window_name).count();
    cards.iter().filter(|(_, n)| !(same == 1 && n == window_name)).cloned().collect()
}

/// What the setting holds, cleaned: a card number from the listing, or empty (off). Anything else
/// (an edited session file) reads as off rather than as a name to search for.
pub(crate) fn setting_spec(raw: &str) -> String {
    let t = raw.trim();
    if t.parse::<usize>().is_ok_and(|n| (1..100).contains(&n)) {
        t.to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_setting_offers_every_card_but_the_window_s_own() {
        let cards = vec![(1, "NVIDIA GeForce RTX 3080".to_string()), (2, "AMD Radeon RX 6800 XT".to_string())];
        assert_eq!(second_card_choices(&cards, "NVIDIA GeForce RTX 3080"), vec![(2, "AMD Radeon RX 6800 XT".to_string())]);
        // One card: nothing to offer.
        assert!(second_card_choices(&cards[..1], "NVIDIA GeForce RTX 3080").is_empty());
        // Two identical cards: neither can be told apart from the window's, so both are offered.
        let twins = vec![(1, "NVIDIA GeForce RTX 3080".to_string()), (2, "NVIDIA GeForce RTX 3080".to_string())];
        assert_eq!(second_card_choices(&twins, "NVIDIA GeForce RTX 3080"), twins);
        // A window on a card the listing does not name (another API): every card is offered.
        assert_eq!(second_card_choices(&cards, "llvmpipe"), cards);
    }

    #[test]
    fn an_export_renders_on_the_window_s_card_and_the_ones_named_beside_it() {
        use crate::farm::local::Gpus;
        let cards = vec![(1, "AMD Radeon RX 6800 XT".to_string()), (2, "NVIDIA GeForce RTX 3070".to_string())];
        // The window on the 6800 XT: `all` adds the 3070; naming both adds the 3070 alone.
        assert_eq!(export_extra_cards(&Gpus::All, &cards, "AMD Radeon RX 6800 XT"), vec![2]);
        assert_eq!(export_extra_cards(&Gpus::List(vec![1, 2]), &cards, "AMD Radeon RX 6800 XT"), vec![2]);
        assert_eq!(export_extra_cards(&Gpus::List(vec![2]), &cards, "AMD Radeon RX 6800 XT"), vec![2]);
        // One card named twice: the window plus a second device on the same card.
        let one = vec![(1, "NVIDIA GeForce RTX 3080".to_string())];
        assert_eq!(export_extra_cards(&Gpus::List(vec![1, 1]), &one, "NVIDIA GeForce RTX 3080"), vec![1]);
        assert!(export_extra_cards(&Gpus::All, &one, "NVIDIA GeForce RTX 3080").is_empty());
    }

    #[test]
    fn the_setting_holds_a_card_number_or_nothing() {
        assert_eq!(setting_spec(""), "");
        assert_eq!(setting_spec(" 2 "), "2");
        assert_eq!(setting_spec("0"), "");
        assert_eq!(setting_spec("same"), "");
        assert_eq!(setting_spec("RTX"), "");
        assert_eq!(setting_spec("6800"), ""); // a model number is a name, not a position
    }

    #[test]
    fn the_status_line_says_where_the_card_stands() {
        assert!(WorkerState::Off.text(false).starts_with("Off"));
        assert_eq!(WorkerState::Running("X · Vulkan".into()).text(false), "In use: X · Vulkan");
        assert!(WorkerState::Lost("X".into()).text(false).contains("choose it again"));
        assert!(WorkerState::Running("X".into()).text(true).ends_with("(set by --worker-gpu)"));
    }
}

/// A job's identity for [`Worker::cancel`]: its run, and its view in the low bit (views are 0 and 1).
fn job_key(view: usize, run: u64) -> u64 {
    (run << 1) | (view as u64 & 1)
}

/// An export's colour (`fs_color`'s output, RGBA floats) as a supersampling sample.
pub(crate) fn sample_of(r: fractadyne_gpu::ExportResult) -> fractadyne_gpu::AccumSample {
    fractadyne_gpu::AccumSample { width: r.width, height: r.height, rgba: r.pixels }
}
