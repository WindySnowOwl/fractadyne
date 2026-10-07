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
        let backends = crate::gpu_choice::backends();
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
                label: Some("fractadyne.worker"),
                required_features: features,
                required_limits: limits,
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ))
        .map_err(|e| format!("the worker device on {} would not open: {e}", info.name))?;
        let name = format!("{} · {:?}", info.name, info.backend);
        let alive = Arc::new(AtomicBool::new(true));
        // ⭐Its loss is ITS loss: unlike the window's device (main.rs), nothing here exits.
        {
            let (alive, name) = (alive.clone(), name.clone());
            device.set_device_lost_callback(move |reason, msg| {
                alive.store(false, Relaxed);
                crate::diag::log_line(
                    "worker",
                    &format!("the worker GPU ({name}) was lost ({reason:?}): {msg} — the live view carries on with one GPU"),
                );
            });
        }
        {
            let (alive, name) = (alive.clone(), name.clone());
            device.on_uncaptured_error(Box::new(move |e| {
                alive.store(false, Relaxed);
                crate::diag::log_line("worker", &format!("the worker GPU ({name}) failed: {e} — it is dropped"));
            }));
        }
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

/// A job's identity for [`Worker::cancel`]: its run, and its view in the low bit (views are 0 and 1).
fn job_key(view: usize, run: u64) -> u64 {
    (run << 1) | (view as u64 & 1)
}

/// An export's colour (`fs_color`'s output, RGBA floats) as a supersampling sample.
pub(crate) fn sample_of(r: fractadyne_gpu::ExportResult) -> fractadyne_gpu::AccumSample {
    fractadyne_gpu::AccumSample { width: r.width, height: r.height, rgba: r.pixels }
}
