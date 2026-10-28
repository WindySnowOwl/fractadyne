//! `--assemble DIR` (`design/zoom-video.md` §6, phase E2): a zoom video, or single frames of one,
//! assembled from the exponential-map strips `--render-expmap DIR` rendered, on the GPU
//! ([`fractadyne_gpu::Assembler`]) and coloured with the app's own colouring.
//!
//! A frame at magnification `2^L` takes, for each of its samples, the nearest texel of the strip
//! its radius falls in, and the app's colour pass colours the result as it colours a render. The
//! strips are read from DIR a few ahead of the frames that need them, and a window of them stays on
//! the GPU; the frames go straight to ffmpeg ([`crate::video_pipe`]) or, with `--assemble-at`, to
//! PNGs.
//!
//! Fractadyne's own strips only (the zoomasm format is for zoomasm), and the colourings that read
//! no per-orbit statistics: smooth, distance, relief and glow, not stripe, TIA or trap.

use crate::expmap_render::{manifest_convention, strip_file, StripFormat, MANIFEST};
use fractadyne_core::{expmap_rows, ExpMap, StripRows, Viewport};
use fractadyne_gpu::{Assembler, FramePlacement, StripShape};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

/// The `--assemble-*` options as given.
#[derive(Clone, Debug, Default)]
pub(crate) struct AssembleCli {
    /// `--assemble-at L[,L...]`: single frames at these log2 magnifications, as PNGs.
    pub(crate) at: Vec<f64>,
    /// `--assemble-fps N` (default 30).
    pub(crate) fps: Option<f64>,
    /// `--assemble-speed X`: octaves of zoom a second (default 1).
    pub(crate) speed: Option<f64>,
    /// `--assemble-ss N`: samples a pixel along each axis (default: the strips' oversampling).
    pub(crate) ss: Option<u32>,
}

impl AssembleCli {
    /// Read the options from the command line; an unreadable one is fatal, never a default.
    pub(crate) fn from_args(args: &[String]) -> Self {
        let val = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1));
        let mut o = AssembleCli::default();
        if let Some(s) = val("--assemble-at") {
            o.at = s
                .split(',')
                .map(|p| crate::arg_parse::<f64>("--assemble-at", p.trim(), "log2 magnifications, comma-separated"))
                .collect();
        }
        if let Some(s) = val("--assemble-fps") {
            let f: f64 = crate::arg_parse("--assemble-fps", s, "frames a second");
            if !(f > 0.0 && f <= 1000.0) {
                eprintln!("fractadyne: --assemble-fps: {f} is outside 0 to 1000.");
                crate::exit(2)
            }
            o.fps = Some(f);
        }
        if let Some(s) = val("--assemble-speed") {
            let v: f64 = crate::arg_parse("--assemble-speed", s, "octaves a second");
            if !(v > 0.0 && v <= 1000.0) {
                eprintln!("fractadyne: --assemble-speed: {v} is outside 0 to 1000.");
                crate::exit(2)
            }
            o.speed = Some(v);
        }
        if let Some(s) = val("--assemble-ss") {
            let q: u32 = crate::arg_parse("--assemble-ss", s, "samples a pixel along each axis");
            if !(1..=8).contains(&q) {
                eprintln!("fractadyne: --assemble-ss: {q} is outside 1 to 8.");
                crate::exit(2)
            }
            o.ss = Some(q);
        }
        o
    }
}

/// A dive's strips, as its manifest describes them.
#[derive(Clone, Debug)]
pub(crate) struct Dive {
    pub(crate) dir: PathBuf,
    pub(crate) map: ExpMap,
    pub(crate) strips: u64,
    /// The frames the strips were rendered for, in pixels.
    pub(crate) frame: (f64, f64),
    pub(crate) oversample: f64,
    pub(crate) start_log2mag: f64,
    pub(crate) end_log2mag: f64,
    pub(crate) fractal: String,
}

/// Read `dir`'s manifest: a dive of Fractadyne's own strips, in the layout this app reads.
pub(crate) fn read_dive(dir: &Path) -> Result<Dive, String> {
    let path = dir.join(MANIFEST);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let m: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("{} is not a manifest: {e}", path.display()))?;
    let d = m.get("dive").ok_or_else(|| format!("{} has no dive", path.display()))?;
    let format = d.get("format").and_then(|v| v.as_str()).unwrap_or("fractadyne");
    if format != StripFormat::Fractadyne.name() {
        return Err(format!(
            "{} holds {format} strips, which zoomasm assembles; --assemble reads Fractadyne's own (--render-expmap without --expmap-format)",
            dir.display()
        ));
    }
    if m.get("convention").and_then(|c| c.as_str()) != Some(manifest_convention(StripFormat::Fractadyne).0) {
        return Err(format!("{}'s strips are laid out another way than this version of the app reads", dir.display()));
    }
    let num = |k: &str| d.get(k).and_then(|v| v.as_f64()).ok_or_else(|| format!("{} has no {k}", path.display()));
    let width = num("width")? as u32;
    let rows = num("rows")? as u32;
    if rows != expmap_rows(width) {
        return Err(format!("{}: {rows} rows is not a {width}-texel strip's", path.display()));
    }
    let frame = d.get("frame").and_then(|f| f.as_array()).filter(|f| f.len() == 2);
    let frame = frame.and_then(|f| Some((f[0].as_f64()?, f[1].as_f64()?))).ok_or_else(|| format!("{} has no frame", path.display()))?;
    Ok(Dive {
        dir: dir.to_path_buf(),
        map: ExpMap { width, rows, r0_log2: num("r0_log2")?, layout: StripRows::Conformal },
        strips: num("strips")? as u64,
        frame,
        oversample: num("oversample")?,
        start_log2mag: num("start_log2mag")?,
        end_log2mag: num("end_log2mag")?,
        fractal: d.get("fractal").and_then(|v| v.as_str()).unwrap_or("Mandelbrot").to_string(),
    })
}

/// A strip as the renderer's main target, `(smooth count, normal x, y, log2 DE in texels)` a
/// texel, from its file: the count `N + NF` (exact in f64, then rounded once to f32), `-1` where it
/// never escaped and `-2` where it was not computed, as an iterate pass marks them.
pub(crate) fn strip_main(path: &Path, map: &ExpMap) -> Result<Vec<f32>, String> {
    use fractadyne_export::ExrChannelData as D;
    let (w, h, channels) = fractadyne_export::read_exr_all_channels(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if (w, h) != (map.width, map.rows) {
        return Err(format!("{} is {w}x{h}, not the dive's {}x{}", path.display(), map.width, map.rows));
    }
    let mut n: Option<Vec<u32>> = None;
    let (mut nf, mut de, mut nx, mut ny) = (None, None, None, None);
    for (name, data) in channels {
        match (name.as_str(), data) {
            ("N", D::U32(v)) => n = Some(v),
            ("NF", D::F32(v)) => nf = Some(v),
            ("DE", D::F32(v)) => de = Some(v),
            ("NX", D::F32(v)) => nx = Some(v),
            ("NY", D::F32(v)) => ny = Some(v),
            _ => {}
        }
    }
    let missing = |c: &str| format!("{} has no {c} channel of the right type", path.display());
    let (n, nf, de, nx, ny) = (
        n.ok_or_else(|| missing("N"))?,
        nf.ok_or_else(|| missing("NF"))?,
        de.ok_or_else(|| missing("DE"))?,
        nx.ok_or_else(|| missing("NX"))?,
        ny.ok_or_else(|| missing("NY"))?,
    );
    let mut out = vec![0f32; n.len() * 4];
    for i in 0..n.len() {
        out[i * 4] = match n[i] {
            u32::MAX => -1.0,
            0 => -2.0,
            c => (c as f64 + nf[i] as f64) as f32,
        };
        out[i * 4 + 1] = nx[i];
        out[i * 4 + 2] = ny[i];
        out[i * 4 + 3] = de[i];
    }
    Ok(out)
}

/// Where the frame at magnification `2^l` sits among the strips, for frames `height_px` pixels
/// high at `ss×ss` samples a pixel: the samples are `REFERENCE_HEIGHT / (h·ss·2^l)` apart, so one
/// sample from the centre lies `r0_log2 − log2(spacing)` octaves inside strip 0's outer radius.
pub(crate) fn placement(map: &ExpMap, l: f64, height_px: f64, ss: u32, turns: f64) -> FramePlacement {
    let spacing_log2 = (Viewport::REFERENCE_HEIGHT / (height_px * ss as f64)).log2() - l;
    FramePlacement::new(map.r0_log2 - spacing_log2, turns)
}

/// The strip shape the GPU needs, for `dive`.
pub(crate) fn shape_of(dive: &Dive) -> StripShape {
    StripShape {
        width: dive.map.width,
        rows: dive.map.rows,
        rows_per_octave: dive.map.rows_per_octave(),
        sample_offset: dive.map.sample_offset(),
        row_step: dive.map.row_step(),
        strips: dive.strips,
    }
}

/// The frames of a straight dive from `start` to `end` (log2 magnifications) at `speed` octaves a
/// second and `fps` frames a second: the first at `start`, the last at `end`, evenly spaced in
/// `log2` magnification between (a constant zoom speed).
pub(crate) fn dive_frames(start: f64, end: f64, speed: f64, fps: f64) -> Vec<f64> {
    let n = (((end - start) / speed * fps).round() as u64).max(1) + 1;
    (0..n).map(|f| start + (end - start) * f as f64 / (n - 1) as f64).collect()
}

/// Strips read from a dive's folder on threads of their own, a few ahead of the frames that need
/// them: an 85 MB strip takes a good part of a second to decode, and a frame needs a new one only
/// every octave.
pub(crate) struct StripReader {
    dive: Arc<Dive>,
    pending: HashMap<u64, JoinHandle<Result<Vec<f32>, String>>>,
}

impl StripReader {
    pub(crate) fn new(dive: Arc<Dive>) -> Self {
        StripReader { dive, pending: HashMap::new() }
    }

    /// Start reading strip `k` if it is in the dive and not already on its way.
    pub(crate) fn ask(&mut self, k: u64) {
        if k >= self.dive.strips || self.pending.contains_key(&k) {
            return;
        }
        let dive = self.dive.clone();
        let h = std::thread::Builder::new()
            .name(format!("strip-read-{k}"))
            .spawn(move || {
                let path = dive.dir.join(strip_file(k));
                if !path.exists() {
                    return Err(format!(
                        "strip {k} is not in {} (render it: --render-expmap with --expmap-strips {k})",
                        dive.dir.display()
                    ));
                }
                strip_main(&path, &dive.map)
            });
        match h {
            Ok(h) => {
                self.pending.insert(k, h);
            }
            Err(e) => crate::diag::log_line("assemble", &format!("could not start reading strip {k}: {e}")),
        }
    }

    /// Strip `k`, reading it now if nobody asked for it.
    pub(crate) fn take(&mut self, k: u64) -> Result<Vec<f32>, String> {
        self.ask(k);
        let h = self.pending.remove(&k).ok_or_else(|| format!("strip {k} could not be read"))?;
        h.join().map_err(|_| format!("reading strip {k} panicked"))?
    }
}

/// Put the strips a frame at `place` reads into the assembler's window (and ask for the next
/// `ahead` beyond them).
pub(crate) fn fill_window(
    asm: &mut Assembler,
    queue: &eframe::wgpu::Queue,
    reader: &mut StripReader,
    place: &FramePlacement,
    ahead: u64,
) -> Result<(), String> {
    let (first, last) = asm.strips_for(place).ok_or("the frame is shallower than the dive's first strip")?;
    for k in first..=last + ahead {
        reader.ask(k);
    }
    for k in first..=last {
        if !asm.has_strip(k) {
            let main = reader.take(k)?;
            asm.put_strip(queue, k, &main).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

impl crate::FractadyneApp {
    /// The colouring an assembled frame gets: the session's (palette, method, cycle, effects), as an
    /// export of the dive's first frame would carry it. Built at 1×, where Mandelbrot renders
    /// directly, so no reference orbit is computed for what is only a colouring.
    pub(crate) fn assembly_colour_request(&self, dive: &Dive) -> fractadyne_gpu::ExportRequest {
        let vp = Viewport::new(dive.frame.0, dive.frame.1);
        self.current_export_request_for(&vp, false)
    }

    /// `--assemble DIR`. Returns the summary line, or why it could not assemble.
    pub(crate) fn assemble_from_dir(
        &mut self,
        ctx: &egui::Context,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        dir: &Path,
    ) -> Result<String, String> {
        let dive = Arc::new(read_dive(dir)?);
        let opts = self.render_cli.assemble_opts.clone();
        // The "Fd" mark goes on every frame, as on a tour's (rasterized once, from the font atlas).
        if self.watermark && self.watermark_overlay.is_none() {
            self.watermark_overlay = self.show_watermark.then(|| crate::export::build_watermark_overlay(ctx)).flatten();
        }
        let ss = opts.ss.unwrap_or_else(|| (dive.oversample.round() as u32).clamp(1, 8));
        let (fw, fh) = (dive.frame.0 as u32, dive.frame.1 as u32);
        let colour = self.assembly_colour_request(&dive);
        if fractadyne_gpu::method_needs_aux(colour.color_method) {
            return Err("this colouring reads per-orbit statistics (stripe, TIA, trap or decomposition) that strips do not keep; \
                        assemble with smooth, distance, relief or glow"
                .into());
        }
        let layers = fractadyne_gpu::layers_for_frame(fw * ss, fh * ss);
        let mut asm = Assembler::new(device, shape_of(&dive), [fw, fh], ss, layers).map_err(|e| e.to_string())?;
        let mut reader = StripReader::new(dive.clone());
        crate::diag::log_line(
            "assemble",
            &format!(
                "{}: a {} dive in {} strips of {}x{}, frames {fw}x{fh} at {ss}x{ss} samples, a window of {layers} strips ({:.0} MB)",
                dir.display(),
                dive.fractal,
                dive.strips,
                dive.map.width,
                dive.map.rows,
                layers as f64 * dive.map.width as f64 * dive.map.rows as f64 * 16.0 / 1.0e6,
            ),
        );
        if ss as f64 > dive.oversample + 1e-9 {
            crate::diag::log_line(
                "assemble",
                &format!(
                    "⚠ {ss} samples a pixel from strips oversampled {}x: near the frame's corners several samples share a texel \
                     (render the strips with --expmap-oversample {ss} for full supersampling)",
                    dive.oversample
                ),
            );
        }

        if !opts.at.is_empty() {
            // Single frames, as PNGs: into --out (a folder when there are several, or a file).
            let several = opts.at.len() > 1;
            let out = self.render_cli.out.clone().unwrap_or_else(|| PathBuf::from(if several { "assembled" } else { "assembled.png" }));
            if several {
                std::fs::create_dir_all(&out).map_err(|e| format!("cannot create {}: {e}", out.display()))?;
            }
            // With --render-iter, the frame's raw main target (its sample grid, before colouring) as an
            // EXR, as `--render-iter` writes a flat render's: the two compare sample for sample.
            let raw = self.render_cli.iter_mode;
            for &l in &opts.at {
                let place = placement(&dive.map, l, dive.frame.1, ss, 0.0);
                fill_window(&mut asm, queue, &mut reader, &place, 0)?;
                let ext = if raw { "exr" } else { "png" };
                let path = if several { out.join(format!("frame_{l:.3}.{ext}")) } else { out.clone() };
                let meta = format!("app=Fractadyne\nassembled_from={}\nlog2mag={l}\nss={ss}\n", dir.display());
                if raw {
                    let main = asm.reprojected(device, queue, &place).map_err(|e| e.to_string())?;
                    fractadyne_export::write_exr_raw(&path, fw * ss, fh * ss, &main, Some(&meta))
                        .map_err(|e| format!("{}: {e}", path.display()))?;
                } else {
                    let mut rgba = asm.frame(device, queue, &place, &colour).map_err(|e| e.to_string())?;
                    self.apply_watermark(&mut rgba, fw, fh);
                    fractadyne_export::write_png(&path, fw, fh, &rgba, Some(&meta)).map_err(|e| format!("{}: {e}", path.display()))?;
                }
            }
            return Ok(format!("{} frame(s) assembled from {} into {}", opts.at.len(), dir.display(), out.display()));
        }

        // A video: the whole dive, at a constant zoom speed.
        let out = self.render_cli.out.clone().ok_or("--assemble makes a video: give its file with --out (e.g. dive.mp4)")?;
        let fps = opts.fps.unwrap_or(30.0);
        let speed = opts.speed.unwrap_or(1.0);
        let frames = dive_frames(dive.start_log2mag, dive.end_log2mag, speed, fps);
        let pipe = crate::video_pipe::VideoPipe::start(&out, fw, fh, fps, 18)?;
        crate::diag::log_line(
            "assemble",
            &format!(
                "{} frames at {fps} fps, {speed} octaves a second (2^{:.2} to 2^{:.2}) → {} (ffmpeg: {})",
                frames.len(),
                dive.start_log2mag,
                dive.end_log2mag,
                out.display(),
                pipe.ffmpeg.display()
            ),
        );
        let progress = Arc::new(AtomicU32::new(0));
        let _pump = crate::diag::progress_pump("assemble", progress.clone());
        let t0 = std::time::Instant::now();
        let mut gpu_s = 0.0f64;
        for (f, &l) in frames.iter().enumerate() {
            let place = placement(&dive.map, l, dive.frame.1, ss, 0.0);
            fill_window(&mut asm, queue, &mut reader, &place, 2)?;
            let tf = std::time::Instant::now();
            let mut rgba = asm.frame(device, queue, &place, &colour).map_err(|e| e.to_string())?;
            gpu_s += tf.elapsed().as_secs_f64();
            self.apply_watermark(&mut rgba, fw, fh);
            pipe.push(rgba)?;
            progress.store(((f + 1) * 1000 / frames.len()) as u32, Ordering::Relaxed);
            if (f + 1) % 300 == 0 {
                crate::diag::log_line(
                    "assemble",
                    &format!("frame {}/{} (2^{l:.2}), {:.1} fps", f + 1, frames.len(), (f + 1) as f64 / t0.elapsed().as_secs_f64()),
                );
            }
        }
        let written = pipe.finish()?;
        let secs = t0.elapsed().as_secs_f64();
        Ok(format!(
            "{written} frames assembled from {} into {} ({:.1} s of video; {:.1} fps, the GPU {:.1} ms a frame)",
            dir.display(),
            out.display(),
            written as f64 / fps,
            written as f64 / secs,
            1000.0 * gpu_s / written.max(1) as f64,
        ))
    }
}

#[cfg(test)]
#[path = "../tests/unit/assemble/tests.rs"]
mod tests;
