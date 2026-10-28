//! `--render-expmap DIR` (`design/zoom-video.md`, phase E0): a straight dive into the view,
//! rendered as EXPONENTIAL-MAP STRIPS (one octave of radius around the dive's centre each), with a
//! manifest. The raw material of a zoom video: E1 writes what zoomasm reads from them, E2 assembles
//! frames in the app.
//!
//! A strip renders as an ordinary raw export ([`fractadyne_gpu::render_iter_tiled`]) of a view
//! sized to the strip's disk, with the shader placing texels on the strip's polar grid
//! ([`fractadyne_gpu::StripMap`]). Sizing the VIEW to the disk is what makes the rest follow: the
//! reference pick, the series radius and the BLA radius all read the view's span, and its half
//! diagonal is the strip's outer radius. Each strip is written atomically, so a strip present in
//! DIR is complete, and a rerun into the same DIR (the manifest must agree) renders only the rest.
//!
//! Supported in E0: Mandelbrot and Multibrot 3-8 (the formulas whose fraction and distance
//! estimate the strip output computes), not Julia, not custom formulas.

use fractadyne_core::{recommended_iterations, ExpMap, Viewport};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// The manifest's file name, in the strips' directory.
pub(crate) const MANIFEST: &str = "expmap.json";

/// The iteration cap over a dive (the user's decision, `design/zoom-video.md` §9.4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum CapMode {
    /// Each strip what its depth wants, never less than the strip before. The default.
    #[default]
    Rising,
    /// Every strip the deepest strip's cap: no step in the cap anywhere in the video.
    Fixed,
}

/// What the strips are written as (`--expmap-format`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum StripFormat {
    /// Fractadyne's own strips: conformal rows with an overlap, the renderer's raw channels.
    #[default]
    Fractadyne,
    /// Kalles Fraktaler's EXR layout, which zoomasm assembles (`expmap_kf.rs`).
    Zoomasm,
}

impl StripFormat {
    pub(crate) fn name(self) -> &'static str {
        match self {
            StripFormat::Fractadyne => "fractadyne",
            StripFormat::Zoomasm => "zoomasm",
        }
    }
}

/// The `--expmap-*` options as given.
#[derive(Clone, Debug)]
pub(crate) struct ExpmapCli {
    pub(crate) start_log2mag: f64,
    pub(crate) oversample: f64,
    pub(crate) cap: CapMode,
    /// `--expmap-strips A-B`, inclusive.
    pub(crate) strips: Option<(u64, u64)>,
    /// `--iter N` was given: N is the cap (fixed) or the base the caps grow from (rising).
    pub(crate) iter_given: bool,
    pub(crate) format: StripFormat,
}

impl Default for ExpmapCli {
    fn default() -> Self {
        ExpmapCli {
            start_log2mag: 0.0,
            oversample: 1.0,
            cap: CapMode::Rising,
            strips: None,
            iter_given: false,
            format: StripFormat::Fractadyne,
        }
    }
}

impl ExpmapCli {
    /// Read the options from the command line; an unreadable one is fatal, never a default.
    pub(crate) fn from_args(args: &[String]) -> Self {
        let val = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1));
        let mut o = ExpmapCli { iter_given: args.iter().any(|a| a == "--iter"), ..Default::default() };
        if let Some(s) = val("--expmap-start") {
            o.start_log2mag = crate::arg_parse("--expmap-start", s, "a log2 magnification");
        }
        if let Some(s) = val("--expmap-oversample") {
            let q: f64 = crate::arg_parse("--expmap-oversample", s, "a number of texels a pixel");
            if !(q >= 1.0 && q <= 8.0) {
                eprintln!("fractadyne: --expmap-oversample: {q} is outside 1 to 8.");
                crate::exit(2)
            }
            o.oversample = q;
        }
        if let Some(s) = val("--expmap-cap") {
            o.cap = match s.as_str() {
                "rising" => CapMode::Rising,
                "fixed" => CapMode::Fixed,
                _ => {
                    eprintln!("fractadyne: --expmap-cap: \"{s}\" is neither rising nor fixed.");
                    crate::exit(2)
                }
            };
        }
        if let Some(s) = val("--expmap-format") {
            o.format = match s.as_str() {
                "fractadyne" => StripFormat::Fractadyne,
                "zoomasm" => StripFormat::Zoomasm,
                _ => {
                    eprintln!("fractadyne: --expmap-format: \"{s}\" is neither fractadyne nor zoomasm.");
                    crate::exit(2)
                }
            };
        }
        if let Some(s) = val("--expmap-strips") {
            o.strips = Some(match parse_strip_range(s) {
                Some(r) => r,
                None => {
                    eprintln!("fractadyne: --expmap-strips: cannot read \"{s}\" as A-B (strip numbers, A <= B).");
                    crate::exit(2)
                }
            });
        }
        o
    }
}

/// `A-B` (or a single `A`) as an inclusive strip range.
pub(crate) fn parse_strip_range(s: &str) -> Option<(u64, u64)> {
    let (a, b) = match s.split_once('-') {
        Some((a, b)) => (a.trim().parse().ok()?, b.trim().parse().ok()?),
        None => {
            let a = s.trim().parse().ok()?;
            (a, a)
        }
    };
    (a <= b).then_some((a, b))
}

/// The depth a strip's cap is chosen for: the frame whose corner is the strip's INNER edge, the
/// deepest frame that still samples it at a pixel or more (`log2` magnification).
pub(crate) fn strip_depth(map: &ExpMap, k: u64, width_px: f64, height_px: f64) -> f64 {
    map.frame_log2mag(k, width_px, height_px) + 1.0
}

/// Every strip's iteration cap. `base` is `--iter`'s N when it was given, otherwise the session's
/// count. Rising: the app's depth rule (`recommended_iterations`: the base plus ~220 an octave) at
/// each strip's depth, never below the strip before. Fixed: N when `--iter` was given, otherwise
/// the deepest strip's rising cap, for every strip.
pub(crate) fn strip_caps(map: &ExpMap, strips: u64, width_px: f64, height_px: f64, base: u32, mode: CapMode, iter_given: bool) -> Vec<u32> {
    let mut caps = Vec::with_capacity(strips as usize);
    let mut floor = 0u32;
    for k in 0..strips {
        let c = recommended_iterations(base, strip_depth(map, k, width_px, height_px)).max(floor);
        floor = c;
        caps.push(c);
    }
    if mode == CapMode::Fixed {
        let all = if iter_given { base } else { caps.last().copied().unwrap_or(base) };
        caps.iter_mut().for_each(|c| *c = all);
    }
    caps
}

/// One rendered strip: the raw targets, the request, and whether it reused the dive's reference.
pub(crate) struct StripRender {
    pub(crate) result: fractadyne_gpu::ExportResult,
    pub(crate) req: fractadyne_gpu::ExportRequest,
    pub(crate) reused: bool,
}

/// Strip `k`'s file name.
pub(crate) fn strip_file(k: u64) -> String {
    format!("strip_{k:06}.exr")
}

/// The view strip `k` renders as: the dive's centre, `W × rows` texels, scaled so the rectangle's
/// half diagonal is the strip's outer radius. The renderer's view-derived radii (the reference
/// pick's span, the series and BLA radii) then cover the strip's disk; the texels themselves are
/// placed by the strip map, not by this rectangle's grid.
pub(crate) fn strip_view(map: &ExpMap, k: u64, cx: &fractadyne_core::BigFloat, cy: &fractadyne_core::BigFloat) -> Viewport {
    let (w, rows) = (map.width as f64, map.rows as f64);
    let mut vp = Viewport::new(w, rows);
    // upp = 2R / hypot(W, rows) and upp = (REFERENCE_HEIGHT / rows) · 2^-l2.
    let upp_log2 = map.outer_log2(k) + 1.0 - w.hypot(rows).log2();
    let l2 = (Viewport::REFERENCE_HEIGHT / rows).log2() - upp_log2;
    vp.set_center_log2mag(cx.clone(), cy.clone(), l2);
    vp
}

/// One strip's channels, from the raw main target (`smooth, nx, ny, de_log2`) and the strip aux
/// target (`iter >> 12, iter & 4095, fraction, final angle in turns`).
///
/// `N`: the escape count; `u32::MAX` never escaped (capped); 0 glitched or not computed.
/// `NF`: the smooth count's fraction `1 − nu` (the smooth count is `N + NF`).
/// `T`: the final z's angle, in turns, `[0, 1)`.
/// `DE`: log2 of the distance estimate in texels of this strip (each texel's own size).
/// `NX`, `NY`: the unit slope normal, in the plane's frame (real right, imaginary up).
pub(crate) fn strip_channels(main: &[f32], aux: &[f32]) -> Vec<fractadyne_export::ExrChannel> {
    use fractadyne_export::{ExrChannel, ExrChannelData};
    let n = main.len() / 4;
    let (mut nn, mut nf, mut t, mut de, mut nx, mut ny) =
        (vec![0u32; n], vec![0f32; n], vec![0f32; n], vec![0f32; n], vec![0f32; n], vec![0f32; n]);
    for i in 0..n {
        let s = main[i * 4];
        de[i] = main[i * 4 + 3];
        nx[i] = main[i * 4 + 1];
        ny[i] = main[i * 4 + 2];
        if s >= 0.0 {
            nn[i] = (aux[i * 4] as u32) << 12 | aux[i * 4 + 1] as u32;
            nf[i] = aux[i * 4 + 2];
            t[i] = aux[i * 4 + 3];
        } else if s > -1.5 {
            nn[i] = u32::MAX; // interior: the cap was reached
        } // else glitched (-2): N stays 0
    }
    vec![
        ExrChannel { name: "N", data: ExrChannelData::U32(nn) },
        ExrChannel { name: "NF", data: ExrChannelData::F32(nf) },
        ExrChannel { name: "T", data: ExrChannelData::F32(t) },
        ExrChannel { name: "DE", data: ExrChannelData::F32(de) },
        ExrChannel { name: "NX", data: ExrChannelData::F32(nx) },
        ExrChannel { name: "NY", data: ExrChannelData::F32(ny) },
    ]
}

impl crate::FractadyneApp {
    /// Render strip `k` of `map` around `(cx, cy)` at `cap` iterations: the raw main target and
    /// the strip aux target (`ExportResult::aux`), with the request it was rendered from. The one
    /// way a strip is rendered — `--render-expmap` and the selftest's `expmap` group both call it.
    /// With `dive_ref` ([`Self::dive_reference`]) the strip REUSES the dive's one orbit and only its
    /// series and BLA are worked out for this strip's radius; without, it picks and builds its own.
    /// Leaves `render_cfg.max_iter`/`auto_iter` and `export.width` changed: the caller restores
    /// them.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_strip(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        map: &ExpMap,
        k: u64,
        cx: &fractadyne_core::BigFloat,
        cy: &fractadyne_core::BigFloat,
        cap: u32,
        dive_ref: Option<&crate::render::ReuseRef>,
    ) -> Result<StripRender, String> {
        let vp = strip_view(map, k, cx, cy);
        // The cap is explicit per strip: auto-iterations would read the strip view's own depth
        // rule, which is not the dive's.
        self.render_cfg.max_iter = cap;
        self.render_cfg.auto_iter = false;
        self.export.width = map.width;
        let budget = crate::render::IterBudget { max_iter: cap, auto_iter: false };
        let mut reused = false;
        let mut req = match (dive_ref, self.export_reference_inputs_for(&vp, false, budget)) {
            (Some(dr), Some(inputs)) => {
                let res = crate::render::build_reference_watching("fd-ref-expmap", inputs.with_reuse(dr.clone()), None);
                reused = res.was_reused();
                self.current_export_request_with_ref(&vp, false, Some(res))
            }
            // The direct path (no reference at all), or no dive reference: build as an export does.
            _ => self.current_export_request_for(&vp, false),
        };
        req.width = map.width;
        req.height = map.rows;
        req.ss = 1;
        let (m, e) = map.outer_scale(k);
        // The shader's texel centre is at +0.5; the layout says where in the texel to sample.
        req.map = fractadyne_gpu::StripMap::strip(map.row_step(), map.sample_offset() - 0.5, m, e, req.delta_exp);
        let r = fractadyne_gpu::render_iter_tiled(device, queue, &req, crate::tunables::CORRECT_WORK_BUDGET, None, None, None)
            .map_err(|e| format!("strip {k}: {e}"))?;
        if r.aux.len() != r.pixels.len() {
            return Err(format!("strip {k}: the renderer returned no strip data (aux {} of {})", r.aux.len(), r.pixels.len()));
        }
        // A strip's count, fraction and angle are written by the chunked path's resolve only (the
        // shader's `strip_aux`); a tile that took the single pass would carry none of them.
        if r.tiles_total == 0 || r.tiles_chunked != r.tiles_total {
            return Err(format!(
                "strip {k}: {} of {} tiles took the chunked path this GPU needs for a strip",
                r.tiles_chunked, r.tiles_total
            ));
        }
        Ok(StripRender { result: r, req, reused })
    }

    /// The dive's ONE reference: built once for its deepest strip `k`, at that strip's cap and
    /// precision, which every shallower strip's view accepts (its precision is lower and the point
    /// is at its centre). `None` when that strip renders on the direct path, which needs none.
    pub(crate) fn dive_reference(
        &mut self,
        map: &ExpMap,
        k: u64,
        cx: &fractadyne_core::BigFloat,
        cy: &fractadyne_core::BigFloat,
        cap: u32,
    ) -> Option<crate::render::ReuseRef> {
        let vp = strip_view(map, k, cx, cy);
        self.render_cfg.max_iter = cap;
        self.render_cfg.auto_iter = false;
        let inputs = self.export_reference_inputs_for(&vp, false, crate::render::IterBudget { max_iter: cap, auto_iter: false })?;
        crate::render::build_reference_watching("fd-ref-expmap", inputs, None).reuse_ref()
    }

    /// `--render-expmap DIR`. Returns the summary line, or why it could not render.
    pub(crate) fn render_expmap_to_dir(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        dir: &Path,
    ) -> Result<String, String> {
        use crate::fractal::FractalKind as F;
        if self.julia_mode {
            return Err("--render-expmap does not render Julia sets yet (zoom-video phase E0)".into());
        }
        if !matches!(
            self.fractal,
            F::Mandelbrot | F::Multibrot3 | F::Multibrot4 | F::Multibrot5 | F::Multibrot6 | F::Multibrot7 | F::Multibrot8
        ) {
            return Err(format!(
                "--render-expmap renders Mandelbrot and Multibrot 3-8 only for now, not {} (zoom-video phase E0)",
                self.fractal.name()
            ));
        }
        let opts = self.render_cli.expmap_opts.clone();
        let (fw, fh) = (self.viewport.width_px, self.viewport.height_px);
        let end = self.viewport.log2_magnification();
        if end <= opts.start_log2mag {
            return Err(format!(
                "the view (2^{end:.2}) must be deeper than where the dive starts (--expmap-start {})",
                opts.start_log2mag
            ));
        }
        let map = match opts.format {
            StripFormat::Fractadyne => ExpMap::for_frames(fw, fh, opts.oversample, opts.start_log2mag),
            StripFormat::Zoomasm => ExpMap::for_frames(fw, fh, opts.oversample, opts.start_log2mag).with_octave_rows(),
        };
        let power = match self.fractal {
            F::Multibrot3 => 3.0,
            F::Multibrot4 => 4.0,
            F::Multibrot5 => 5.0,
            F::Multibrot6 => 6.0,
            F::Multibrot7 => 7.0,
            F::Multibrot8 => 8.0,
            _ => 2.0,
        };
        let n = map.strips_to(end, fh);
        let base = self.render_cfg.max_iter;
        let caps = strip_caps(&map, n, fw, fh, base, opts.cap, opts.iter_given);
        let (from, to) = opts.strips.unwrap_or((0, n - 1));
        if from >= n {
            return Err(format!("--expmap-strips starts at {from}, but this dive has {n} strips (0 to {})", n - 1));
        }
        let to = to.min(n - 1);
        std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
        let (cx, cy) = (self.viewport.center_x.clone(), self.viewport.center_y.clone());
        let dive = serde_json::json!({
            "fractal": self.fractal.name(),
            "center_re": fractadyne_core::to_decimal_string(&cx),
            "center_im": fractadyne_core::to_decimal_string(&cy),
            "frame": [fw, fh],
            "format": opts.format.name(),
            "oversample": opts.oversample,
            "start_log2mag": opts.start_log2mag,
            "end_log2mag": end,
            "width": map.width,
            "rows": map.rows,
            "r0_log2": map.r0_log2,
            "strips": n,
            "cap_mode": match opts.cap { CapMode::Rising => "rising", CapMode::Fixed => "fixed" },
            "caps": caps,
        });
        write_or_check_manifest(dir, &map, opts.format, &dive)?;
        crate::diag::log_line(
            "expmap",
            &format!(
                "dive {} → 2^{end:.2} in {n} {} strips of {}x{} (r0 2^{:.3}, {:.2} rows an octave); strips {from}-{to}; caps {} {}..{}",
                opts.start_log2mag, opts.format.name(), map.width, map.rows, map.r0_log2, map.rows_per_octave(),
                match opts.cap { CapMode::Rising => "rising", CapMode::Fixed => "fixed" },
                caps[from as usize], caps[to as usize],
            ),
        );

        let progress = Arc::new(AtomicU32::new(0));
        let _pump = crate::diag::progress_pump("expmap", progress.clone());
        let saved = (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.export.width);
        let result = (|| -> Result<(u64, u64), String> {
            let (mut rendered, mut kept) = (0u64, 0u64);
            // ⭐ONE reference for the whole dive, built for its deepest strip (that strip's cap and
            // precision serve every shallower one); each strip then works out only its own series
            // and BLA. A strip that cannot take it (the direct path, or a refusal) builds its own.
            let todo: Vec<u64> = (from..=to).filter(|&k| !dir.join(strip_file(k)).exists()).collect();
            let dive_ref = match todo.last() {
                Some(&deepest) => {
                    let t = std::time::Instant::now();
                    let r = self.dive_reference(&map, deepest, &cx, &cy, caps[deepest as usize]);
                    crate::diag::log_line(
                        "expmap",
                        &match &r {
                            Some(d) => format!(
                                "one reference for the dive, from strip {deepest}: {} samples at {} bits in {:.0} ms",
                                d.prefix.len(),
                                d.prec,
                                t.elapsed().as_secs_f64() * 1000.0
                            ),
                            None => "no dive reference: the deepest strip renders directly".into(),
                        },
                    );
                    r
                }
                None => None,
            };
            for k in from..=to {
                let path = dir.join(strip_file(k));
                if path.exists() {
                    kept += 1;
                    continue;
                }
                let t0 = std::time::Instant::now();
                let s = self.render_strip(device, queue, &map, k, &cx, &cy, caps[k as usize], dive_ref.as_ref())?;
                let (r, req) = (&s.result, &s.req);
                let meta = format!(
                    "expmap_strip={k}\nouter_log2={}\nmax_iter={}\nmode={}\nwidth={}\nrows={}\nformat={}\n",
                    map.outer_log2(k), req.max_iter, req.mode, map.width, map.rows, opts.format.name(),
                );
                let written = match opts.format {
                    StripFormat::Fractadyne => {
                        fractadyne_export::write_exr_channels(&path, map.width, map.rows, strip_channels(&r.pixels, &r.aux), Some(&meta))
                    }
                    StripFormat::Zoomasm => {
                        let centre = (fractadyne_core::to_f64(&cx), fractadyne_core::to_f64(&cy));
                        let s = crate::expmap_kf::KfStrip { map: &map, k, power, centre };
                        let attrs = [
                            ("Iterations", req.max_iter.min(i32::MAX as u32) as i32),
                            ("IterationsBias", crate::expmap_kf::KF_ITERATIONS_BIAS),
                        ];
                        let ch = crate::expmap_kf::kf_channels(&r.pixels, &r.aux, &s);
                        fractadyne_export::write_exr_channels_with(&path, map.width, map.rows, ch, Some(&meta), &attrs)
                    }
                };
                written.map_err(|e| format!("strip {k}: cannot write {}: {e}", path.display()))?;
                let escaped = r.pixels.chunks_exact(4).filter(|p| p[0] >= 0.0).count();
                crate::diag::log_line(
                    "expmap",
                    &format!(
                        "strip {k}/{n}: 2^{:.2}, cap {}, mode {}, {}, {:.1}% escaped, {:.0} ms",
                        map.frame_log2mag(k, fw, fh), req.max_iter, req.mode,
                        if req.mode == 1 { "direct" } else if s.reused { "the dive's reference" } else { "its own reference" },
                        100.0 * escaped as f64 / (r.pixels.len() / 4).max(1) as f64,
                        t0.elapsed().as_secs_f64() * 1000.0,
                    ),
                );
                rendered += 1;
                progress.store((((k - from + 1) * 1000) / (to - from + 1)) as u32, Ordering::Relaxed);
            }
            Ok((rendered, kept))
        })();
        (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.export.width) = saved;
        let (rendered, kept) = result?;
        Ok(format!(
            "{rendered} strip(s) rendered{} in {} ({n} in the dive, {}x{} each)",
            if kept > 0 { format!(", {kept} already there") } else { String::new() },
            dir.display(),
            map.width,
            map.rows,
        ))
    }
}

/// Write the manifest, or check that the one already in `dir` describes the same dive in the same
/// layout: a strip from another dive, or laid out another way, must never be taken for one of
/// this dive's.
fn write_or_check_manifest(dir: &Path, map: &ExpMap, format: StripFormat, dive: &serde_json::Value) -> Result<(), String> {
    let path: PathBuf = dir.join(MANIFEST);
    let (convention, channels) = manifest_convention(format);
    if let Ok(text) = std::fs::read_to_string(&path) {
        let old: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| format!("{} is not a manifest this app can read: {e}", path.display()))?;
        if old.get("dive") != Some(dive) || old.get("convention").and_then(|c| c.as_str()) != Some(convention) {
            return Err(format!(
                "{} holds the strips of another dive, or laid out another way (its {} differs); use an empty directory",
                dir.display(),
                MANIFEST
            ));
        }
        return Ok(());
    }
    let manifest = serde_json::json!({
        "format": match format { StripFormat::Fractadyne => "fractadyne-expmap", StripFormat::Zoomasm => "kf-expmap" },
        "version": 1,
        "app": crate::sysinfo::version_string(),
        "convention": convention,
        "rows_per_octave": map.rows_per_octave(),
        "channels": channels,
        "files": "strip_NNNNNN.exr, one per strip, NNNNNN = k",
        "dive": dive,
    });
    let text = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())? + "\n";
    std::fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

/// What a manifest says of a format's layout (`convention`, checked when a folder is reused and
/// when one is assembled) and its channels.
pub(crate) fn manifest_convention(format: StripFormat) -> (&'static str, serde_json::Value) {
    match format {
        StripFormat::Fractadyne => (
            "texel (i, j) of strip k is at angle 2*pi*(i + 0.5)/width anticlockwise from the positive real axis and radius 2^(r0_log2 - k) * exp(-2*pi*(j + 0.5)/width) from the centre; row 0 is outermost; strip k's last rows overlap strip k+1's first",
            serde_json::json!({
                "N": "escape count (uint32); 4294967295 = never escaped (the cap was reached), 0 = glitched",
                "NF": "fraction of the smooth count, 1 - nu; the smooth count is N + NF",
                "T": "angle of the final z, in turns [0, 1)",
                "DE": "log2 of the distance estimate in texels of this strip",
                "NX": "slope normal, real part (the plane's frame)",
                "NY": "slope normal, imaginary part",
            }),
        ),
        StripFormat::Zoomasm => (
            "Kalles Fraktaler's exponential map with the imaginary axis up (its ImagPointsUp: 1), for zoomasm: texel (i, j) of strip k is sampled at angle 2*pi*i/width CLOCKWISE from the positive real axis and radius 2^(r0_log2 - k - j/rows) from the centre; row 0 is outermost; one octave a strip, no overlap; strip 0 is the outermost (zoom-in order: leave zoomasm's Reverse and Flip boxes unticked)",
            serde_json::json!({
                "N": "escape count + IterationsBias (uint32), escaping at |z| > 10^(4/power); 4294967295 = never escaped, 0 = not computed (N0 and N1 instead, low and high words, when a count passes 4294967294)",
                "NF": "1 - log_power(ln|z| / ln 10^(4/power)), in [0, 1); the smooth count is N - IterationsBias + NF",
                "T": "fract(power * arg z / 2pi)",
                "DEX": "distance estimate over KF's pixel (r * 2 / hypot(width, rows)), along the real axis",
                "DEY": "the same along the imaginary axis, pointing up",
            }),
        ),
    }
}

#[cfg(test)]
#[path = "../tests/unit/expmap_render/tests.rs"]
mod tests;
