//! Export & view-state I/O: render-to-file (PNG/EXR, foreground + background worker),
//! the reloadable view-metadata blob (embedded in exports, also used by bookmarks/.fdn),
//! and Open-view. (The gallery browser stays with its UI in main.rs.)

use crate::{
    separate_paths, stitch_side_by_side, version_string, ExportFormat, ExportJob, FractadyneApp,
    FractalKind,
};

/// A deep single-view export whose bignum reference orbit is being built off the main thread (it can
/// take minutes at extreme depth). Held while `rx` is pending; when the reference lands the request
/// is assembled (reusing it — no rebuild) and the render dispatched to a worker. Keeps the UI
/// responsive instead of freezing the whole app during the reference build.
pub(crate) struct ExportPrep {
    /// The deep MAP-view reference, building off-thread.
    pub rx: std::sync::mpsc::Receiver<crate::render::RecomputeResult>,
    pub map_vp: fractadyne_core::Viewport,
    /// Single-view Julia flag (the dual map is always Mandelbrot, so `false` there).
    pub julia_mode: bool,
    /// `Some` ⇒ dual export: the Julia panel to render + combine (usually shallow → builds instantly).
    pub julia_vp: Option<fractadyne_core::Viewport>,
    pub dual_mode: crate::DualExport,
    pub path: std::path::PathBuf,
}

/// Pre-rasterized "Fd" brand mark for stamping into exports: premultiplied **linear** RGBA at a
/// high resolution (downscaled per export). Built once on the main thread from the egui font atlas
/// (same glyphs as the live-view watermark), then blended by the export worker — which has no egui
/// context — via [`stamp_watermark`].
#[derive(Clone)]
pub(crate) struct WmOverlay {
    pub w: usize,
    pub h: usize,
    pub px: Vec<[f32; 4]>, // premultiplied, linear
}

/// Rasterize the "Fd" mark (F in brand text, d in amber) from the egui font atlas into a
/// premultiplied-linear overlay with a soft dark halo (so it stays legible on any background).
pub(crate) fn build_watermark_overlay(ctx: &egui::Context) -> Option<WmOverlay> {
    let ppp = ctx.pixels_per_point();
    let pts = 40.0_f32; // layout size in points; rasterized at pts*ppp texels, downscaled per export
    let galley =
        ctx.fonts(|f| f.layout_job(crate::theme::brand_mark_job(pts, crate::theme::BRAND_TEXT, crate::theme::BRAND_ACCENT)));
    let pad = (pts * ppp * 0.20).ceil() as i32;
    let gw = (galley.size().x * ppp).ceil() as i32;
    let gh = (galley.size().y * ppp).ceil() as i32;
    let w = (gw + 2 * pad).max(1) as usize;
    let h = (gh + 2 * pad).max(1) as usize;
    // Straight-alpha linear glyph layer: rgb = linear color, a = coverage.
    let mut glyph = vec![[0.0f32; 4]; w * h];
    ctx.fonts(|f| {
        let atlas = f.image();
        let aw = atlas.size[0];
        for row in &galley.rows {
            for g in &row.glyphs {
                let uv = g.uv_rect;
                if uv.max[0] <= uv.min[0] || uv.max[1] <= uv.min[1] {
                    continue;
                }
                let lin = egui::Rgba::from(
                    galley.job.sections.get(g.section_index as usize)
                        .map(|s| s.format.color).unwrap_or(egui::Color32::WHITE),
                );
                let ox = ((g.pos.x + uv.offset.x) * ppp).round() as i32 + pad;
                let oy = ((g.pos.y + uv.offset.y) * ppp).round() as i32 + pad;
                for ty in uv.min[1]..uv.max[1] {
                    for tx in uv.min[0]..uv.max[0] {
                        let cov = atlas.pixels[ty as usize * aw + tx as usize];
                        if cov <= 0.0 {
                            continue;
                        }
                        let dx = ox + (tx - uv.min[0]) as i32;
                        let dy = oy + (ty - uv.min[1]) as i32;
                        if dx < 0 || dy < 0 || dx >= w as i32 || dy >= h as i32 {
                            continue;
                        }
                        let p = &mut glyph[dy as usize * w + dx as usize];
                        if cov > p[3] {
                            *p = [lin.r(), lin.g(), lin.b(), cov];
                        }
                    }
                }
            }
        }
    });
    // Soft halo = box-blurred glyph coverage; composite the glyph over a translucent black halo.
    let radius = (pts * ppp * 0.09).round().max(1.0) as i32;
    let halo = blur_coverage(&glyph, w, h, radius);
    let mut px = vec![[0.0f32; 4]; w * h];
    for i in 0..w * h {
        let ga = glyph[i][3];
        let ha = (halo[i] * 0.60).min(1.0); // dark halo strength
        // glyph (premult) over black halo (premult rgb = 0): rgb = glyph.rgb*ga; a = ga + ha(1-ga)
        px[i] = [glyph[i][0] * ga, glyph[i][1] * ga, glyph[i][2] * ga, ga + ha * (1.0 - ga)];
    }
    Some(WmOverlay { w, h, px })
}

/// Separable box blur of the coverage (alpha) channel — the halo footprint.
fn blur_coverage(src: &[[f32; 4]], w: usize, h: usize, r: i32) -> Vec<f32> {
    let mut a: Vec<f32> = src.iter().map(|p| p[3]).collect();
    let mut tmp = vec![0.0f32; w * h];
    let norm = 1.0 / (2 * r + 1) as f32;
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            for k in -r..=r {
                let xx = (x as i32 + k).clamp(0, w as i32 - 1) as usize;
                s += a[y * w + xx];
            }
            tmp[y * w + x] = s * norm;
        }
    }
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            for k in -r..=r {
                let yy = (y as i32 + k).clamp(0, h as i32 - 1) as usize;
                s += tmp[yy * w + x];
            }
            a[y * w + x] = s * norm;
        }
    }
    a
}

/// Alpha-blend the pre-rasterized mark into the lower-right of a **linear** RGBA export buffer.
/// Height scales to ~2.6% of the image (matching the live view); area-averaged downscale.
pub(crate) fn stamp_watermark(pixels: &mut [f32], w: u32, h: u32, ov: &WmOverlay) {
    if ov.w == 0 || ov.h == 0 || w == 0 || h == 0 {
        return;
    }
    let th = (h as f32 * 0.026).clamp(16.0, (h as f32) * 0.2);
    let scale = th / ov.h as f32; // < 1 (downscale)
    let tw = (ov.w as f32 * scale).round() as i32;
    let th = th.round() as i32;
    if tw <= 0 || th <= 0 {
        return;
    }
    let margin_x = (w as f32 * 0.012).round() as i32;
    let margin_y = (h as f32 * 0.012).round() as i32;
    let x0 = w as i32 - tw - margin_x;
    let y0 = h as i32 - th - margin_y;
    let inv = ov.w as f32 / tw as f32; // source texels per dest pixel
    for dy in 0..th {
        for dx in 0..tw {
            // Area-average the source footprint for this dest pixel.
            let sx0 = (dx as f32 * inv) as i32;
            let sx1 = (((dx + 1) as f32 * inv) as i32).max(sx0 + 1).min(ov.w as i32);
            let sy0 = (dy as f32 * inv) as i32;
            let sy1 = (((dy + 1) as f32 * inv) as i32).max(sy0 + 1).min(ov.h as i32);
            let (mut acc, mut n) = ([0.0f32; 4], 0.0f32);
            for sy in sy0..sy1 {
                for sx in sx0..sx1 {
                    let s = ov.px[sy as usize * ov.w + sx as usize];
                    acc[0] += s[0]; acc[1] += s[1]; acc[2] += s[2]; acc[3] += s[3];
                    n += 1.0;
                }
            }
            if n == 0.0 {
                continue;
            }
            let src = [acc[0] / n, acc[1] / n, acc[2] / n, acc[3] / n]; // premultiplied linear
            let (px_, py) = (x0 + dx, y0 + dy);
            if px_ < 0 || py < 0 || px_ >= w as i32 || py >= h as i32 {
                continue;
            }
            let idx = (py as usize * w as usize + px_ as usize) * 4;
            let ia = 1.0 - src[3];
            pixels[idx] = src[0] + pixels[idx] * ia;
            pixels[idx + 1] = src[1] + pixels[idx + 1] * ia;
            pixels[idx + 2] = src[2] + pixels[idx + 2] * ia;
            // leave alpha channel (export is opaque)
        }
    }
}

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
            .export.notes
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
            self.export.notes = n;
        }
        // Match the viewport's working precision to the restored zoom; drop caches.
        self.viewport.precision = fractadyne_core::precision_for_octaves(
            self.viewport.log2_magnification().max(0.0).ceil() as u64,
        );
        self.invalidate_refs();
        self.pointer.zoom_vel = 0.0;
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
            Ok(Some(m)) => {
                let report = self.load_view_metadata(&m);
                self.export.status = Some(match report.note() {
                    None => format!("Loaded view from {}", path.display()),
                    Some(n) => format!("Loaded view from {} — {n}", path.display()),
                });
            }
            Ok(None) => {
                self.export.status =
                    Some("That file has no embedded Fractadyne view metadata.".to_string());
            }
            Err(e) => {
                self.export.status = Some(format!("Couldn't read {}: {e}", path.display()));
            }
        }
    }


    pub(crate) fn export_ext(&self) -> &'static str {
        match self.export.format {
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
    pub(crate) fn start_export(&mut self, ctx: &egui::Context, device: eframe::wgpu::Device, queue: eframe::wgpu::Queue) {
        if self.export.task.is_some() || self.export.prep.is_some() {
            return;
        }
        let ext = self.export_ext();
        let start_dir = self
            .export.last_dir
            .clone()
            .filter(|d| d.is_dir())
            .unwrap_or_else(Self::pictures_dir);
        let path = rfd::FileDialog::new()
            .set_directory(start_dir)
            .set_file_name(self.export_default_name())
            .add_filter(ext.to_uppercase(), &[ext])
            .save_file();
        let Some(path) = path else {
            self.export.status = Some("Export canceled.".to_string());
            return;
        };
        self.start_export_to(ctx, device, queue, path);
    }

    /// Quick export (hotkey): no dialog — save to the last-used folder with an auto name.
    pub(crate) fn quick_export(&mut self, ctx: &egui::Context, device: eframe::wgpu::Device, queue: eframe::wgpu::Queue) {
        if self.export.task.is_some() || self.export.prep.is_some() {
            return;
        }
        let dir = self
            .export.last_dir
            .clone()
            .filter(|d| d.is_dir())
            .unwrap_or_else(Self::pictures_dir);
        let path = dir.join(self.export_default_name());
        self.start_export_to(ctx, device, queue, path);
    }

    /// Stamp the "Fd" mark into a linear RGBA image buffer if the watermark is enabled and built.
    pub(crate) fn apply_watermark(&self, pixels: &mut [f32], w: u32, h: u32) {
        if self.watermark {
            if let Some(ov) = &self.watermark_overlay {
                stamp_watermark(pixels, w, h, ov);
            }
        }
    }

    /// Render one export view — glitch-corrected when enabled and applicable, else the plain path.
    /// `vp` + `julia` identify the view (correction maps glitched pixels back to coordinates to seed
    /// fresh references). Correction is synchronous (multi-pass GPU + readback), so this must run on
    /// the thread owning the device; it falls back to the plain render for aux coloring / oversized
    /// views / when nothing is glitched.
    #[allow(clippy::too_many_arguments)] // REFACTOR-PLAN Phase 2/4: fold the request params into a struct
    fn render_export_view(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        vp: &fractadyne_core::Viewport,
        julia: bool,
        req: &fractadyne_gpu::ExportRequest,
        progress: &std::sync::atomic::AtomicU32,
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Result<fractadyne_gpu::ExportResult, fractadyne_gpu::GpuError> {
        if self.glitch_correct {
            if let Some(res) = self.render_export_corrected(device, queue, vp, julia, req.width, req.height) {
                return Ok(res);
            }
        }
        fractadyne_gpu::render_export(device, queue, req, progress, cancel)
    }

    /// Synchronously render the current view and write it to `path` (used by the
    /// headless `--render` CLI mode). Blocks until done; returns a status message.
    pub(crate) fn render_to_file(
        &self,
        ctx: &egui::Context,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        path: &std::path::Path,
    ) -> Result<String, crate::error::AppError> {
        use std::sync::atomic::AtomicBool;
        use std::sync::atomic::AtomicU32;
        let progress = AtomicU32::new(0);
        let cancel = AtomicBool::new(false);
        let meta = self.view_metadata();
        let fmt = self.export.format;
        let write = |p: &std::path::Path, w: u32, h: u32, mut px: Vec<f32>| {
            self.apply_watermark(&mut px, w, h);
            match fmt {
                ExportFormat::Png => fractadyne_export::write_png(p, w, h, &px, Some(&meta)),
                ExportFormat::Exr => fractadyne_export::write_exr(p, w, h, &px, Some(&meta)),
            }
        };
        // Each view is glitch-corrected when enabled + applicable (single and both dual panels).
        let view = |vp: &fractadyne_core::Viewport, julia: bool, req: &fractadyne_gpu::ExportRequest| {
            self.render_export_view(device, queue, vp, julia, req, &progress, &cancel)
        };
        match self.build_export_job() {
            ExportJob::Single(req) => {
                let mut r = view(&self.viewport, self.julia_mode, &req)?;
                if self.show_location {
                    crate::scripting::stamp_location(ctx, &mut r.pixels, r.width, r.height, &self.viewport);
                }
                write(path, r.width, r.height, r.pixels)?;
                Ok(format!("Saved {}×{} → {}", r.width, r.height, path.display()))
            }
            ExportJob::SideBySide(a, b) => {
                let ra = view(&self.viewport, false, &a)?;
                let rb = view(&self.julia_viewport, true, &b)?;
                let (w, h, px) = stitch_side_by_side(&ra, &rb);
                write(path, w, h, px)?;
                Ok(format!("Saved {w}×{h} → {}", path.display()))
            }
            ExportJob::Separate(a, b) => {
                let (pmap, pjul) = separate_paths(path);
                let ra = view(&self.viewport, false, &a)?;
                write(&pmap, ra.width, ra.height, ra.pixels)?;
                let rb = view(&self.julia_viewport, true, &b)?;
                write(&pjul, rb.width, rb.height, rb.pixels)?;
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
    ) -> Result<String, crate::error::AppError> {
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


    /// Fully render + write a glitch-corrected export synchronously (main thread), for every job
    /// layout. `Some(status)` = handled (success message, or a write-error message); `None` = a view
    /// can't be corrected (aux coloring / oversized), so the caller falls back to the plain threaded
    /// path. Correction re-renders per reference, so it can't use the tiled worker's progress model.
    fn export_corrected_sync(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        path: &std::path::Path,
        job: &ExportJob,
        hud: Option<&crate::scripting::HudOverlay>,
    ) -> Option<String> {
        let meta = self.view_metadata();
        let correct = |vp: &fractadyne_core::Viewport, julia: bool, req: &fractadyne_gpu::ExportRequest| {
            self.render_export_corrected(device, queue, vp, julia, req.width, req.height)
        };
        let write = |p: &std::path::Path,
                     w: u32,
                     h: u32,
                     mut px: Vec<f32>|
         -> Result<(), fractadyne_export::ExportError> {
            self.apply_watermark(&mut px, w, h);
            if let Some(ov) = hud {
                crate::scripting::blit_location_overlay(&mut px, w, h, ov);
            }
            match self.export.format {
                ExportFormat::Png => fractadyne_export::write_png(p, w, h, &px, Some(&meta)),
                ExportFormat::Exr => fractadyne_export::write_exr(p, w, h, &px, Some(&meta)),
            }
        };
        let status = |res: Result<(), fractadyne_export::ExportError>, ok: String| match res {
            Ok(_) => ok,
            Err(e) => format!("Export failed: {e}"),
        };
        match job {
            ExportJob::Single(req) => {
                let r = correct(&self.viewport, self.julia_mode, req)?;
                let (w, h) = (r.width, r.height);
                Some(status(write(path, w, h, r.pixels), format!("Saved {w}×{h} (glitch-corrected) → {}", path.display())))
            }
            ExportJob::SideBySide(a, b) => {
                let ra = correct(&self.viewport, false, a)?;
                let rb = correct(&self.julia_viewport, true, b)?;
                let (w, h, px) = stitch_side_by_side(&ra, &rb);
                Some(status(write(path, w, h, px), format!("Saved {w}×{h} (glitch-corrected) → {}", path.display())))
            }
            ExportJob::Separate(a, b) => {
                let ra = correct(&self.viewport, false, a)?;
                let rb = correct(&self.julia_viewport, true, b)?;
                let (pmap, pjul) = separate_paths(path);
                let w1 = write(&pmap, ra.width, ra.height, ra.pixels);
                let w2 = write(&pjul, rb.width, rb.height, rb.pixels);
                Some(status(w1.and(w2), format!("Saved 2 files (glitch-corrected) → {}", pmap.display())))
            }
        }
    }

    /// Render the current job on a worker thread and write to `path` (or, for dual
    /// "separate", to `path` + a sibling). The UI stays responsive; result via channel.
    pub(crate) fn start_export_to(
        &mut self,
        ctx: &egui::Context,
        device: eframe::wgpu::Device,
        queue: eframe::wgpu::Queue,
        path: std::path::PathBuf,
    ) {
        if self.export.task.is_some() || self.export.prep.is_some() {
            return;
        }
        // Start the export clock now — for a deep export this includes the (long) off-thread
        // reference build, which is part of the wait the user is timing.
        self.export.started = Some(std::time::Instant::now());
        if let Some(parent) = path.parent() {
            self.export.last_dir = Some(parent.to_path_buf());
        }
        // Deep export: build the (slow, bignum) MAP reference orbit OFF the main thread so the UI
        // stays responsive instead of freezing (at extreme depth the reference build alone is
        // minutes). The render dispatches once it lands — see the `export_prep` poll in `update()`.
        // Works for single AND dual (the dual Julia panel is usually shallow and builds instantly in
        // the poll). Glitch correction is skipped at this depth (its multi-pass re-render would
        // re-block the UI); shallower exports keep the synchronous path below (fast; correction applies).
        if self.viewport.magnification() >= crate::PERT_FE_THRESHOLD {
            let map_julia = !self.dual && self.julia_mode; // the dual map panel is Mandelbrot
            if let Some(rx) = self.spawn_export_reference(&self.viewport, map_julia) {
                self.export.prep = Some(ExportPrep {
                    rx,
                    map_vp: self.viewport.clone(),
                    julia_mode: self.julia_mode,
                    julia_vp: self.dual.then(|| self.julia_viewport.clone()),
                    dual_mode: self.export.dual_mode,
                    path,
                });
                self.export.status =
                    Some("Preparing deep export — building reference (this can take a while)…".to_string());
                return;
            }
        }
        let job = self.build_export_job();
        // Optional location HUD: rasterized here on the main thread (needs the egui font atlas), then
        // blitted by the sync/worker write paths (which have no context). Built from the map view;
        // it lands on the image's top-left — which for a side-by-side dual stitch is the map panel.
        let hud = self
            .show_location
            .then(|| crate::scripting::build_location_overlay(ctx, &self.viewport, self.export_height()))
            .flatten();
        // Glitch correction re-renders per reference (synchronous, main thread), so it runs here
        // rather than on the tiled worker. Handles single + dual layouts; falls back to the threaded
        // path for aux coloring methods or views past the ~32 MP / single-texture correction limit.
        if self.glitch_correct {
            if let Some(msg) = self.export_corrected_sync(&device, &queue, &path, &job, hud.as_ref()) {
                self.export.status = Some(self.finish_export_status(msg));
                return;
            }
        }
        self.spawn_export_worker(device, queue, job, path, hud);
    }

    /// Format an export duration compactly: `8.3s`, or `1m 04.0s` past a minute.
    pub(crate) fn fmt_export_duration(d: std::time::Duration) -> String {
        let s = d.as_secs_f64();
        if s < 60.0 {
            format!("{s:.1}s")
        } else {
            let m = (s / 60.0).floor() as u64;
            format!("{m}m {:04.1}s", s - m as f64 * 60.0)
        }
    }

    /// Finalize an export status line: on success append the total elapsed time; either way clear
    /// the timer. Cancel/failure messages pass through unchanged (no time — the run didn't finish).
    pub(crate) fn finish_export_status(&mut self, msg: String) -> String {
        match self.export.started.take() {
            Some(t) if msg.starts_with("Saved") => {
                format!("{msg}  (in {})", Self::fmt_export_duration(t.elapsed()))
            }
            _ => msg,
        }
    }

    /// Render an already-assembled export job (references built) on a background worker and write it
    /// (watermark + HUD applied). Shares the export status channel + progress/cancel. Used by the
    /// deep-export path once the reference has been built off the main thread (`export_prep`).
    pub(crate) fn spawn_export_worker(
        &mut self,
        device: eframe::wgpu::Device,
        queue: eframe::wgpu::Queue,
        job: ExportJob,
        path: std::path::PathBuf,
        hud: Option<crate::scripting::HudOverlay>,
    ) {
        use std::sync::atomic::Ordering::Relaxed;
        let meta = self.view_metadata();
        let format = self.export.format;
        self.export.progress.store(0, Relaxed);
        self.export.cancel.store(false, Relaxed);
        let progress = self.export.progress.clone();
        let cancel = self.export.cancel.clone();
        let wm = self.watermark.then(|| self.watermark_overlay.clone()).flatten();
        let (tx, rx) = std::sync::mpsc::channel();
        self.export.task = Some(rx);
        self.export.status = Some("Rendering…".to_string());
        std::thread::spawn(move || {
            let render = |req: &fractadyne_gpu::ExportRequest| {
                fractadyne_gpu::render_export(&device, &queue, req, &progress, &cancel)
            };
            let write = |p: &std::path::Path, w: u32, h: u32, mut px: Vec<f32>| {
                if let Some(ov) = &wm {
                    stamp_watermark(&mut px, w, h, ov);
                }
                if let Some(ov) = &hud {
                    crate::scripting::blit_location_overlay(&mut px, w, h, ov);
                }
                match format {
                    ExportFormat::Png => fractadyne_export::write_png(p, w, h, &px, Some(&meta)),
                    ExportFormat::Exr => fractadyne_export::write_exr(p, w, h, &px, Some(&meta)),
                }
            };
            let msg = (|| -> Result<String, crate::error::AppError> {
                match job {
                    ExportJob::Single(req) => {
                        let r = render(&req)?;
                        progress.store(2000, Relaxed);
                        let (rw, rh) = (r.width, r.height);
                        write(&path, rw, rh, r.pixels)?;
                        Ok(format!("Saved {}×{} → {}", rw, rh, path.display()))
                    }
                    ExportJob::SideBySide(a, b) => {
                        let ra = render(&a)?;
                        let rb = render(&b)?;
                        progress.store(2000, Relaxed);
                        let (w, h, px) = stitch_side_by_side(&ra, &rb);
                        write(&path, w, h, px)?;
                        Ok(format!("Saved {w}×{h} → {}", path.display()))
                    }
                    ExportJob::Separate(a, b) => {
                        let (pmap, pjul) = separate_paths(&path);
                        let ra = render(&a)?;
                        write(&pmap, ra.width, ra.height, ra.pixels)?;
                        let rb = render(&b)?;
                        progress.store(2000, Relaxed);
                        write(&pjul, rb.width, rb.height, rb.pixels)?;
                        Ok(format!("Saved 2 files → {}", pmap.display()))
                    }
                }
            })();
            let _ = tx.send(match msg {
                Ok(m) => m,
                Err(crate::error::AppError::Gpu(fractadyne_gpu::GpuError::Canceled)) => {
                    "Export canceled.".to_string()
                }
                Err(e) => format!("Export failed: {e}"),
            });
        });
    }
}
