//! Export & view-state I/O: render-to-file (PNG/EXR, foreground + background worker),
//! the reloadable view-metadata blob (embedded in exports, also used by bookmarks/.fdn),
//! and Open-view. (The gallery browser stays with its UI in main.rs.)

use crate::{
    separate_paths, stitch_side_by_side, version_string, ExportFormat, ExportJob, FractadyneApp,
    FractalKind,
};

/// Version of the reloadable view-metadata format (the `format_version=` field shared by
/// exports, `.fdn` locations and bookmarks). Bump ONLY on a breaking change to an existing
/// field's meaning or units — purely additive new keys don't need it (the allow-list reader
/// ignores unknown keys and defaults missing ones, so old and new builds interoperate).
/// A file whose `format_version` exceeds this is from a newer build: we still load the
/// fields we recognise, but warn the user that newer settings/semantics may not apply.
pub(crate) const VIEW_FORMAT_VERSION: u32 = 1;

/// Largest zoom depth (octaves = log2 of magnification) accepted from an untrusted view
/// file. Past this the bignum working precision (∝ octaves) would balloon into a memory
/// DoS, so a hostile/garbage `upp_log2` is clamped here. ~10× the deepest validated zoom
/// (`--validate-deep` reaches 1e1000000× ≈ 3.3e6 octaves), so no real location is affected.
const MAX_LOAD_OCTAVES: f64 = 3.4e7;

/// Upper bound on `max_iter` accepted from an untrusted view file (an absurd value would
/// make an export grind for hours / exhaust the iteration budget). Well above any real use.
const MAX_LOAD_ITER: u32 = 10_000_000;

/// Report from restoring view metadata, so callers can surface anything noteworthy
/// (a newer file format, clamped values, unrecognized fields) instead of loading silently.
#[derive(Default)]
pub(crate) struct ViewLoad {
    /// `Some(v)` if the file's `format_version` exceeds this build's (loaded best-effort).
    pub(crate) newer: Option<u32>,
    /// Human-readable names of fields whose value was out of range and clamped/rejected.
    pub(crate) clamped: Vec<&'static str>,
    /// Unrecognized keys present in the file (a typo, or a newer format's new fields).
    pub(crate) unknown: Vec<String>,
}

impl ViewLoad {
    /// A short summary of anything noteworthy, or `None` when the load was fully clean.
    pub(crate) fn note(&self) -> Option<String> {
        let mut parts: Vec<String> = Vec::new();
        if let Some(v) = self.newer {
            parts.push(format!(
                "saved by a newer Fractadyne (format v{v}); some settings may not apply — consider updating"
            ));
        }
        if !self.clamped.is_empty() {
            parts.push(format!("clamped out-of-range {}", self.clamped.join(", ")));
        }
        if !self.unknown.is_empty() {
            parts.push(format!("ignored unknown field(s): {}", self.unknown.join(", ")));
        }
        (!parts.is_empty()).then(|| parts.join("; "))
    }
}

/// Keys the view-metadata reader understands; anything else in a file is reported as unknown.
const KNOWN_VIEW_KEYS: &[&str] = &[
    "app", "version", "format_version", "saved_unix", "saved", "notes", "fractal", "julia",
    "julia_c_re", "julia_c_im", "center_x", "center_y", "upp", "upp_log2", "zoom", "max_iter",
    "auto_iter", "palette", "cycle", "offset", "aa",
];

impl FractadyneApp {
    /// Reloadable view-state metadata embedded in exports. The center is stored as
    /// full-precision decimal so deep-zoom positions round-trip exactly.
    pub(crate) fn view_metadata(&self) -> String {
        let (jcx, jcy) = self.julia_c;
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        // Latin-1 / single-line safe notes (PNG tEXt), max 120 chars.
        let notes: String = self
            .export_notes
            .chars()
            .filter(|c| !c.is_control() && (*c as u32) <= 0xFF)
            .take(120)
            .collect();
        format!(
            "app=Fractadyne\nversion={}\nformat_version={}\nsaved_unix={}\nsaved={}\n\
             notes={}\nfractal={}\njulia={}\njulia_c_re={:.17e}\njulia_c_im={:.17e}\n\
             center_x={}\ncenter_y={}\nupp={:.17e}\nupp_log2={:.17e}\nzoom={}\nmax_iter={}\nauto_iter={}\n\
             palette={}\ncycle={}\noffset={}\naa={}\n",
            version_string(),
            VIEW_FORMAT_VERSION,
            secs,
            Self::utc_date_string(secs),
            notes,
            self.fractal.name(),
            self.julia_mode as u32,
            jcx,
            jcy,
            fractadyne_core::to_decimal_string(&self.viewport.center_x),
            fractadyne_core::to_decimal_string(&self.viewport.center_y),
            self.viewport.units_per_pixel.to_f64(),
            // Extended-range scale (log2 of units_per_pixel) so deep (>1e308×) views reload
            // exactly; `upp` above is the saturating f64 (back-compat + human-readable).
            self.viewport.units_per_pixel.log2(),
            self.viewport.magnification(),
            self.max_iter,
            self.auto_iter as u32,
            self.palette_idx,
            self.cycle,
            self.offset,
            self.aa,
        )
    }

    /// Restore the view from view-state metadata (exported image, `.fdn`, or bookmark).
    /// Untrusted input: every field is allow-listed, parsed leniently, and clamped to a
    /// safe range; unknown keys are ignored and missing keys keep their current value.
    /// Returns whether the file's `format_version` is within this build's range so callers
    /// can warn on a forward-incompatible (newer) file.
    pub(crate) fn load_view_metadata(&mut self, meta: &str) -> ViewLoad {
        let get = |key: &str| -> Option<String> {
            meta.lines().find_map(|l| {
                l.split_once('=')
                    .filter(|(k, _)| k.trim() == key)
                    .map(|(_, v)| v.trim().to_string())
            })
        };
        // A file with no `format_version` predates the field but is format-1 compatible.
        let file_ver = get("format_version")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(VIEW_FORMAT_VERSION);
        let mut report = ViewLoad::default();
        if let Some(f) = get("fractal").and_then(|s| FractalKind::from_name(&s)) {
            self.fractal = f;
        }
        self.julia_mode =
            get("julia").map(|s| s == "1").unwrap_or(false) && self.fractal.supports_julia();
        if let (Some(re), Some(im)) = (
            get("julia_c_re").and_then(|s| s.parse().ok()),
            get("julia_c_im").and_then(|s| s.parse().ok()),
        ) {
            self.julia_c = (re, im);
        }
        if let Some(cx) = get("center_x").and_then(|s| fractadyne_core::parse_bf(&s)) {
            self.viewport.center_x = cx;
        }
        if let Some(cy) = get("center_y").and_then(|s| fractadyne_core::parse_bf(&s)) {
            self.viewport.center_y = cy;
        }
        // Prefer the extended-range `upp_log2` (exact past 1e308×); fall back to the f64
        // `upp` for images saved before it existed. Clamp the depth so a hostile value
        // can't blow up the bignum working precision (memory DoS).
        if let Some(raw) = get("upp_log2").and_then(|s| s.parse::<f64>().ok()) {
            let l = raw.clamp(-MAX_LOAD_OCTAVES, MAX_LOAD_OCTAVES);
            if !raw.is_finite() || l != raw {
                report.clamped.push("zoom depth");
            }
            self.viewport.units_per_pixel = fractadyne_core::FloatExp::from_f64(1.0).mul_pow2(l);
        } else if let Some(raw) = get("upp").and_then(|s| s.parse::<f64>().ok()) {
            if raw.is_finite() && raw > 0.0 {
                let l = raw.log2().clamp(-MAX_LOAD_OCTAVES, MAX_LOAD_OCTAVES);
                if l != raw.log2() {
                    report.clamped.push("zoom depth");
                }
                self.viewport.units_per_pixel = fractadyne_core::FloatExp::from_f64(1.0).mul_pow2(l);
            } else {
                report.clamped.push("zoom depth");
            }
        }
        if let Some(mi) = get("max_iter").and_then(|s| s.parse::<u32>().ok()) {
            let c = mi.clamp(1, MAX_LOAD_ITER);
            if c != mi {
                report.clamped.push("max_iter");
            }
            self.max_iter = c;
        }
        if let Some(ai) = get("auto_iter") {
            self.auto_iter = ai == "1";
        }
        if let Some(p) = get("palette").and_then(|s| s.parse::<usize>().ok()) {
            if p < fractadyne_color::PRESETS.len() {
                self.palette_idx = p;
            } else {
                report.clamped.push("palette");
            }
        }
        if let Some(c) = get("cycle").and_then(|s| s.parse::<f32>().ok()) {
            if c.is_finite() {
                let v = c.clamp(0.0, 1.0e6);
                if v != c {
                    report.clamped.push("cycle");
                }
                self.cycle = v;
            } else {
                report.clamped.push("cycle");
            }
        }
        if let Some(o) = get("offset").and_then(|s| s.parse::<f32>().ok()) {
            if o.is_finite() {
                let v = o.clamp(-1.0e6, 1.0e6);
                if v != o {
                    report.clamped.push("offset");
                }
                self.offset = v;
            } else {
                report.clamped.push("offset");
            }
        }
        if let Some(a) = get("aa").and_then(|s| s.parse::<u32>().ok()) {
            let c = a.clamp(1, 16);
            if c != a {
                report.clamped.push("anti-aliasing");
            }
            self.aa = c;
        }
        if let Some(n) = get("notes") {
            self.export_notes = n;
        }
        // Match the viewport's working precision to the restored zoom; drop caches.
        self.viewport.precision = fractadyne_core::precision_for_octaves(
            self.viewport.log2_magnification().max(0.0).ceil() as u64,
        );
        self.invalidate_refs();
        self.zoom_vel = 0.0;
        self.record_nav();
        // Report any keys we didn't recognize (cap the list so a junk file can't flood it).
        for line in meta.lines() {
            if let Some((k, _)) = line.split_once('=') {
                let k = k.trim();
                if !k.is_empty()
                    && !KNOWN_VIEW_KEYS.contains(&k)
                    && !report.unknown.iter().any(|u| u == k)
                {
                    report.unknown.push(k.to_string());
                    if report.unknown.len() >= 8 {
                        break;
                    }
                }
            }
        }
        report.newer = (file_ver > VIEW_FORMAT_VERSION).then_some(file_ver);
        report
    }

    /// Open a previously-exported PNG/EXR and restore its view (via a native dialog).
    pub(crate) fn open_view(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("Fractadyne image", &["png", "exr"])
            .set_directory(Self::pictures_dir())
            .pick_file();
        let Some(path) = path else { return };
        let is_exr = path
            .extension()
            .map(|e| e.eq_ignore_ascii_case("exr"))
            .unwrap_or(false);
        let meta = if is_exr {
            fractadyne_export::read_exr_metadata(&path)
        } else {
            fractadyne_export::read_png_metadata(&path)
        };
        match meta {
            Some(m) => {
                let report = self.load_view_metadata(&m);
                self.export_status = Some(match report.note() {
                    None => format!("Loaded view from {}", path.display()),
                    Some(n) => format!("Loaded view from {} — {n}", path.display()),
                });
            }
            None => {
                self.export_status =
                    Some("That file has no embedded Fractadyne view metadata.".to_string());
            }
        }
    }


    pub(crate) fn export_ext(&self) -> &'static str {
        match self.export_format {
            ExportFormat::Png => "png",
            ExportFormat::Exr => "exr",
        }
    }

    /// Default timestamped export filename for the current fractal.
    pub(crate) fn export_default_name(&self) -> String {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        format!(
            "fractadyne_{}_{}.{}",
            self.fractal.name().replace(' ', ""),
            Self::file_stamp(stamp),
            self.export_ext(),
        )
    }

    /// Start a background export, prompting for a path (modal Save dialog).
    pub(crate) fn start_export(&mut self, device: eframe::wgpu::Device, queue: eframe::wgpu::Queue) {
        if self.export_task.is_some() {
            return;
        }
        let ext = self.export_ext();
        let start_dir = self
            .export_last_dir
            .clone()
            .filter(|d| d.is_dir())
            .unwrap_or_else(Self::pictures_dir);
        let path = rfd::FileDialog::new()
            .set_directory(start_dir)
            .set_file_name(self.export_default_name())
            .add_filter(ext.to_uppercase(), &[ext])
            .save_file();
        let Some(path) = path else {
            self.export_status = Some("Export canceled.".to_string());
            return;
        };
        self.start_export_to(device, queue, path);
    }

    /// Quick export (hotkey): no dialog — save to the last-used folder with an auto name.
    pub(crate) fn quick_export(&mut self, device: eframe::wgpu::Device, queue: eframe::wgpu::Queue) {
        if self.export_task.is_some() {
            return;
        }
        let dir = self
            .export_last_dir
            .clone()
            .filter(|d| d.is_dir())
            .unwrap_or_else(Self::pictures_dir);
        let path = dir.join(self.export_default_name());
        self.start_export_to(device, queue, path);
    }

    /// Synchronously render the current view and write it to `path` (used by the
    /// headless `--render` CLI mode). Blocks until done; returns a status message.
    pub(crate) fn render_to_file(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        path: &std::path::Path,
    ) -> Result<String, String> {
        use std::sync::atomic::AtomicBool;
        use std::sync::atomic::AtomicU32;
        let progress = AtomicU32::new(0);
        let cancel = AtomicBool::new(false);
        let meta = self.view_metadata();
        let fmt = self.export_format;
        let write = |p: &std::path::Path, w: u32, h: u32, px: &[f32]| match fmt {
            ExportFormat::Png => fractadyne_export::write_png(p, w, h, px, Some(&meta)),
            ExportFormat::Exr => fractadyne_export::write_exr(p, w, h, px, Some(&meta)),
        };
        let render = |req: &fractadyne_gpu::ExportRequest| {
            fractadyne_gpu::render_export(device, queue, req, &progress, &cancel)
        };
        match self.build_export_job() {
            ExportJob::Single(req) => {
                // Multi-reference glitch correction when enabled (falls back to a normal render
                // for aux coloring methods or sizes past the single-texture limit).
                let corrected = self.glitch_correct.then(|| {
                    self.render_export_corrected(
                        device, queue, &self.viewport, self.julia_mode, req.width, req.height,
                    )
                }).flatten();
                let r = match corrected {
                    Some(res) => res,
                    None => render(&req)?,
                };
                write(path, r.width, r.height, &r.pixels)?;
                Ok(format!("Saved {}×{} → {}", r.width, r.height, path.display()))
            }
            ExportJob::SideBySide(a, b) => {
                let (ra, rb) = (render(&a)?, render(&b)?);
                let (w, h, px) = stitch_side_by_side(&ra, &rb);
                write(path, w, h, &px)?;
                Ok(format!("Saved {w}×{h} → {}", path.display()))
            }
            ExportJob::Separate(a, b) => {
                let (pmap, pjul) = separate_paths(path);
                let ra = render(&a)?;
                write(&pmap, ra.width, ra.height, &ra.pixels)?;
                let rb = render(&b)?;
                write(&pjul, rb.width, rb.height, &rb.pixels)?;
                Ok(format!("Saved 2 files → {}", pmap.display()))
            }
        }
    }

    /// Render the **raw iteration texture** for the current view and write it as an EXR
    /// (`--render-iter`): four 32-bit float channels — R = smooth iteration (negative ⇒
    /// in-set/interior), G/B = slope normal (x, y), A = log₂(distance estimate in pixels).
    /// Lets a reviewer diff iteration data directly, removing coloring as a confound.
    /// Single-tile, clamped to the GPU's max texture dimension.
    pub(crate) fn render_iter_to_file(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        path: &std::path::Path,
    ) -> Result<String, String> {
        let req = self.current_export_request_for(&self.viewport, self.julia_mode);
        let r = fractadyne_gpu::render_iter(device, queue, &req)?;
        let meta = format!(
            "{}\n# iteration-data EXR: R=smooth_iter (<0 = interior), G=normal.x, \
             B=normal.y, A=log2(distance_estimate_px)",
            self.view_metadata()
        );
        fractadyne_export::write_exr(path, r.width, r.height, &r.pixels, Some(&meta))?;
        Ok(format!("Saved iteration EXR {}×{} → {}", r.width, r.height, path.display()))
    }


    /// Render the current job on a worker thread and write to `path` (or, for dual
    /// "separate", to `path` + a sibling). The UI stays responsive; result via channel.
    pub(crate) fn start_export_to(
        &mut self,
        device: eframe::wgpu::Device,
        queue: eframe::wgpu::Queue,
        path: std::path::PathBuf,
    ) {
        if self.export_task.is_some() {
            return;
        }
        if let Some(parent) = path.parent() {
            self.export_last_dir = Some(parent.to_path_buf());
        }
        let job = self.build_export_job();
        // Glitch-corrected single-view export runs synchronously — it re-renders per reference, so
        // it doesn't fit the tiled worker's progress model. Opt-in; falls back to the threaded
        // path for dual layouts, aux coloring methods, or sizes past the single-texture limit.
        if self.glitch_correct {
            if let ExportJob::Single(req) = &job {
                if let Some(res) = self.render_export_corrected(
                    &device, &queue, &self.viewport, self.julia_mode, req.width, req.height,
                ) {
                    let meta = self.view_metadata();
                    let (w, h) = (res.width, res.height);
                    let wr = match self.export_format {
                        ExportFormat::Png => fractadyne_export::write_png(&path, w, h, &res.pixels, Some(&meta)),
                        ExportFormat::Exr => fractadyne_export::write_exr(&path, w, h, &res.pixels, Some(&meta)),
                    };
                    self.export_status = Some(match wr {
                        Ok(_) => format!("Saved {w}×{h} (glitch-corrected) → {}", path.display()),
                        Err(e) => format!("Export failed: {e}"),
                    });
                    return;
                }
            }
        }
        let meta = self.view_metadata();
        let format = self.export_format;
        use std::sync::atomic::Ordering::Relaxed;
        self.export_progress.store(0, Relaxed);
        self.export_cancel.store(false, Relaxed);
        let progress = self.export_progress.clone();
        let cancel = self.export_cancel.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.export_task = Some(rx);
        self.export_status = Some("Rendering…".to_string());
        std::thread::spawn(move || {
            let render = |req: &fractadyne_gpu::ExportRequest| {
                fractadyne_gpu::render_export(&device, &queue, req, &progress, &cancel)
            };
            let write = |p: &std::path::Path, w: u32, h: u32, px: &[f32]| match format {
                ExportFormat::Png => fractadyne_export::write_png(p, w, h, px, Some(&meta)),
                ExportFormat::Exr => fractadyne_export::write_exr(p, w, h, px, Some(&meta)),
            };
            let msg = (|| -> Result<String, String> {
                match job {
                    ExportJob::Single(req) => {
                        let r = render(&req)?;
                        progress.store(2000, Relaxed);
                        write(&path, r.width, r.height, &r.pixels)?;
                        Ok(format!("Saved {}×{} → {}", r.width, r.height, path.display()))
                    }
                    ExportJob::SideBySide(a, b) => {
                        let ra = render(&a)?;
                        let rb = render(&b)?;
                        progress.store(2000, Relaxed);
                        let (w, h, px) = stitch_side_by_side(&ra, &rb);
                        write(&path, w, h, &px)?;
                        Ok(format!("Saved {w}×{h} → {}", path.display()))
                    }
                    ExportJob::Separate(a, b) => {
                        let (pmap, pjul) = separate_paths(&path);
                        let ra = render(&a)?;
                        write(&pmap, ra.width, ra.height, &ra.pixels)?;
                        let rb = render(&b)?;
                        progress.store(2000, Relaxed);
                        write(&pjul, rb.width, rb.height, &rb.pixels)?;
                        Ok(format!("Saved 2 files → {}", pmap.display()))
                    }
                }
            })();
            let _ = tx.send(match msg {
                Ok(m) => m,
                Err(e) if e == "canceled" => "Export canceled.".to_string(),
                Err(e) => format!("Export failed: {e}"),
            });
        });
    }
}
