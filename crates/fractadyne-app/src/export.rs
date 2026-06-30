//! Export & view-state I/O: render-to-file (PNG/EXR, foreground + background worker),
//! the reloadable view-metadata blob (embedded in exports, also used by bookmarks/.fdn),
//! and Open-view. (The gallery browser stays with its UI in main.rs.)

use crate::{
    separate_paths, stitch_side_by_side, version_string, ExportFormat, ExportJob, FractadyneApp,
    FractalKind,
};

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
            "app=Fractadyne\nversion={}\nformat_version=1\nsaved_unix={}\nsaved={}\n\
             notes={}\nfractal={}\njulia={}\njulia_c_re={:.17e}\njulia_c_im={:.17e}\n\
             center_x={}\ncenter_y={}\nupp={:.17e}\nupp_log2={:.17e}\nzoom={}\nmax_iter={}\nauto_iter={}\n\
             palette={}\ncycle={}\noffset={}\naa={}\n",
            version_string(),
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

    /// Restore the view from metadata read out of an exported PNG.
    pub(crate) fn load_view_metadata(&mut self, meta: &str) {
        let get = |key: &str| -> Option<String> {
            meta.lines().find_map(|l| {
                l.split_once('=')
                    .filter(|(k, _)| k.trim() == key)
                    .map(|(_, v)| v.trim().to_string())
            })
        };
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
        // `upp` for images saved before it existed.
        if let Some(l) = get("upp_log2").and_then(|s| s.parse::<f64>().ok()).filter(|l| l.is_finite()) {
            self.viewport.units_per_pixel = fractadyne_core::FloatExp::from_f64(1.0).mul_pow2(l);
        } else if let Some(upp) = get("upp").and_then(|s| s.parse::<f64>().ok()) {
            self.viewport.units_per_pixel = fractadyne_core::FloatExp::from_f64(upp);
        }
        if let Some(mi) = get("max_iter").and_then(|s| s.parse().ok()) {
            self.max_iter = mi;
        }
        if let Some(ai) = get("auto_iter") {
            self.auto_iter = ai == "1";
        }
        if let Some(p) = get("palette").and_then(|s| s.parse::<usize>().ok()) {
            if p < fractadyne_color::PRESETS.len() {
                self.palette_idx = p;
            }
        }
        if let Some(c) = get("cycle").and_then(|s| s.parse().ok()) {
            self.cycle = c;
        }
        if let Some(o) = get("offset").and_then(|s| s.parse().ok()) {
            self.offset = o;
        }
        if let Some(a) = get("aa").and_then(|s| s.parse().ok()) {
            self.aa = a;
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
                self.load_view_metadata(&m);
                self.export_status = Some(format!("Loaded view from {}", path.display()));
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
                let r = render(&req)?;
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
