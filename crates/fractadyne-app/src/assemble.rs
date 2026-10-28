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
    /// `--keep-strips DIR` (with `--render-zoom-video`): write the strips there too.
    pub(crate) keep: Option<PathBuf>,
    /// `--assemble-crf N`: H.264 quality, 0 (lossless) to 51 (default 18, the tour export's).
    pub(crate) crf: Option<u32>,
    /// `--assemble-tour T`: the video follows tour T's timeline (its keyframes' zooms, times, holds
    /// and easing; every frame on the dive's centre).
    pub(crate) tour: Option<PathBuf>,
    /// `--assemble-rotate R`: the frame turns R turns an octave of zoom (the plane turns clockwise on
    /// screen when R is positive).
    pub(crate) rotate: Option<f64>,
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
        o.keep = val("--keep-strips").map(PathBuf::from);
        o.tour = val("--assemble-tour").map(PathBuf::from);
        if let Some(s) = val("--assemble-rotate") {
            let r: f64 = crate::arg_parse("--assemble-rotate", s, "turns an octave");
            if !r.is_finite() || r.abs() > 100.0 {
                eprintln!("fractadyne: --assemble-rotate: {r} is outside -100 to 100 turns an octave.");
                crate::exit(2)
            }
            o.rotate = Some(r);
        }
        if let Some(s) = val("--assemble-crf") {
            let q: u32 = crate::arg_parse("--assemble-crf", s, "an H.264 quality, 0 (lossless) to 51");
            if q > 51 {
                eprintln!("fractadyne: --assemble-crf: {q} is outside 0 to 51.");
                crate::exit(2)
            }
            o.crf = Some(q);
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
    /// The dive's centre, as the manifest writes it (decimal).
    pub(crate) centre: (String, String),
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
        centre: (
            d.get("center_re").and_then(|v| v.as_str()).ok_or_else(|| format!("{} has no center_re", path.display()))?.to_string(),
            d.get("center_im").and_then(|v| v.as_str()).ok_or_else(|| format!("{} has no center_im", path.display()))?.to_string(),
        ),
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

/// Texels a side of the blocks a strip's escape values are summarized in ([`StripRange`]).
pub(crate) const RANGE_BLOCK: usize = 8;

/// A strip's escape values in blocks of [`RANGE_BLOCK`]² texels: each block's least, greatest and
/// mean smooth count among its escaped texels, and how many escaped (`+∞`, `−∞`, 0, 0 when none
/// did). A frame's escape range, and where most of its picture escapes, follow to within a block,
/// from a sixteenth of the strip — what normalizing a video's palette reads ([`frame_range`]).
#[derive(Debug)]
pub(crate) struct StripRange {
    pub(crate) bw: usize,
    pub(crate) bh: usize,
    pub(crate) lo: Vec<f32>,
    pub(crate) hi: Vec<f32>,
    pub(crate) mean: Vec<f32>,
    pub(crate) n: Vec<u32>,
}

/// A `w × h` strip's main target summarized ([`StripRange`]).
pub(crate) fn strip_range(main: &[f32], w: usize, h: usize) -> StripRange {
    let (bw, bh) = (w.div_ceil(RANGE_BLOCK), h.div_ceil(RANGE_BLOCK));
    let (mut lo, mut hi) = (vec![f32::INFINITY; bw * bh], vec![f32::NEG_INFINITY; bw * bh]);
    let (mut sum, mut n) = (vec![0f64; bw * bh], vec![0u32; bw * bh]);
    for j in 0..h {
        let row = (j / RANGE_BLOCK) * bw;
        for i in 0..w {
            let v = main[(j * w + i) * 4];
            if v >= 0.0 {
                let b = row + i / RANGE_BLOCK;
                lo[b] = lo[b].min(v);
                hi[b] = hi[b].max(v);
                sum[b] += v as f64;
                n[b] += 1;
            }
        }
    }
    let mean = sum.iter().zip(&n).map(|(&s, &c)| if c > 0 { (s / c as f64) as f32 } else { 0.0 }).collect();
    StripRange { bw, bh, lo, hi, mean, n }
}

/// The palette window of the frame at magnification `2^l` (`fw × fh` pixels), as the live view takes
/// it: from the least smooth count in the frame to the value by which fraction `q` of its AREA had
/// escaped (`norm_palette_top`'s quantile: the slowest pixels, just under the cap wherever interior
/// is in view, would otherwise squash the picture into the bottom of the palette). Read from the
/// blocks of every strip the frame shows whose centre texel lies inside it, each weighted by its
/// escaped texels' area in the frame (a deep strip's texels are tiny there). `None` when nothing in
/// the frame escaped. `ranges` must hold its strips.
pub(crate) fn frame_range(
    map: &ExpMap,
    ranges: &HashMap<u64, Arc<StripRange>>,
    strips: u64,
    l: f64,
    fw: f64,
    fh: f64,
    q: f64,
) -> Result<Option<(f32, f32)>, String> {
    let place = placement(map, l, fh, 1, 0.0);
    let Some((first, last)) = fractadyne_gpu::strips_for_frame(&place, fw as u32, fh as u32, strips) else {
        return Ok(None);
    };
    let (o, rpo, step) = (map.sample_offset(), map.rows_per_octave(), map.step());
    let (hw, hh) = (fw / 2.0, fh / 2.0);
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    let mut blocks: Vec<(f32, f64)> = Vec::new();
    for k in first..=last {
        let r = ranges.get(&k).ok_or_else(|| format!("strip {k}'s escape range has not been read"))?;
        // A texel of row j lies 2^(D − k − (j + o)/rpo) pixels from the centre, D the placement's
        // depth (r0_log2 − log2 pixel), kept exact as the placement keeps it.
        let dk = (place.depth_whole - k as i64) as f64 + place.depth_frac;
        let centre = |b: usize, n: u32| ((b * RANGE_BLOCK + RANGE_BLOCK / 2).min(n as usize - 1)) as f64 + o;
        let cols: Vec<(f64, f64)> = (0..r.bw).map(|bx| (step * centre(bx, map.width)).sin_cos()).collect();
        for by in 0..r.bh {
            let rad = (dk - centre(by, map.rows) / rpo).exp2();
            for (bx, &(s, c)) in cols.iter().enumerate() {
                let b = by * r.bw + bx;
                if r.n[b] > 0 && (rad * c).abs() <= hw && (rad * s).abs() <= hh {
                    lo = lo.min(r.lo[b]);
                    hi = hi.max(r.hi[b]);
                    // A texel's area in the frame goes as the square of its radius (pixels).
                    blocks.push((r.mean[b], r.n[b] as f64 * rad * rad));
                }
            }
        }
    }
    if hi < lo {
        return Ok(None);
    }
    let top = weighted_quantile(&mut blocks, q).map_or(hi, |v| v.clamp(lo, hi));
    Ok(Some((lo, top)))
}

/// The value below which fraction `q` of the total weight lies (`(value, weight)` pairs, reordered).
pub(crate) fn weighted_quantile(v: &mut [(f32, f64)], q: f64) -> Option<f32> {
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    let total: f64 = v.iter().map(|p| p.1).sum();
    if !(total > 0.0) {
        return None;
    }
    let want = q.clamp(0.0, 1.0) * total;
    let mut acc = 0.0;
    for &(x, w) in v.iter() {
        acc += w;
        if acc >= want {
            return Some(x);
        }
    }
    v.last().map(|p| p.0)
}

/// How much of a video frame's palette map is the LOG form, for the window `[lo, top]`: all of it
/// when the user asked for log; otherwise a smooth step from none to all across a factor of two
/// either side of the window ratio where the live view switches (`NORM_LOG_AUTO_RATIO`), so a dive
/// whose window crosses that ratio dissolves from one form to the other instead of jumping.
pub(crate) fn log_weight(lo: f32, top: f32, log_palette: bool) -> f32 {
    if log_palette {
        return 1.0;
    }
    let x = (top / lo.max(1.0)).max(1.0e-30).log2() - crate::render::NORM_LOG_AUTO_RATIO.log2();
    let t = ((x + 1.0) / 2.0).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A video's palette ranges for `--normalize`: measured at every whole octave of the dive (and its
/// end) by [`frame_range`], and interpolated between in depth, so each frame's mapping is a
/// function of its depth alone — as a tour's time-keyed anchors are — never of the frames before it.
/// GEOMETRICALLY: a window grows by factors (11 to 54 over the first octave of a dive from 1×), and
/// interpolated linearly it moved 13% in the first frame of the octave and 3% in the last.
/// An anchor with nothing escaped takes its neighbour's range; with none on either side the last
/// range given carries on.
pub(crate) struct NormAnchors {
    start: f64,
    end: f64,
    at: HashMap<i64, Option<(f32, f32)>>,
    last: Option<(f32, f32)>,
}

impl NormAnchors {
    pub(crate) fn new(start: f64, end: f64) -> Self {
        NormAnchors { start, end, at: HashMap::new(), last: None }
    }

    /// The depth of anchor `n`: `n` octaves into the dive, the last one at its end.
    pub(crate) fn depth(&self, n: i64) -> f64 {
        (self.start + n as f64).min(self.end)
    }

    /// The anchors frame depth `l` lies between.
    pub(crate) fn around(&self, l: f64) -> (i64, i64) {
        let n = ((l - self.start).floor() as i64).max(0);
        (n, if self.depth(n + 1) > self.depth(n) { n + 1 } else { n })
    }

    /// The range at depth `l`, measuring the anchors around it with `measure` (a depth's range) the
    /// first time they are needed.
    pub(crate) fn range(
        &mut self,
        l: f64,
        mut measure: impl FnMut(f64) -> Result<Option<(f32, f32)>, String>,
    ) -> Result<Option<(f32, f32)>, String> {
        let (a, b) = self.around(l);
        for n in [a, b] {
            if !self.at.contains_key(&n) {
                let r = measure(self.depth(n))?;
                self.at.insert(n, r);
            }
        }
        let (la, lb) = (self.depth(a), self.depth(b));
        let t = if lb > la { ((l - la) / (lb - la)).clamp(0.0, 1.0) as f32 } else { 0.0 };
        // Exactly the anchor's window at an anchor; geometric between (linear if either end is not
        // positive, which only a window starting at 0 can be).
        let lerp = |p: f32, q: f32| match t {
            t if t <= 0.0 => p,
            t if t >= 1.0 => q,
            t if p > 0.0 && q > 0.0 => (p.ln() + t * (q.ln() - p.ln())).exp(),
            t => p + t * (q - p),
        };
        let r = match (self.at[&a], self.at[&b]) {
            (Some(x), Some(y)) => Some((lerp(x.0, y.0), lerp(x.1, y.1))),
            (Some(x), None) | (None, Some(x)) => Some(x),
            (None, None) => self.last,
        };
        if r.is_some() {
            self.last = r;
        }
        Ok(r)
    }
}

/// Strips read from a dive's folder on threads of their own, a few ahead of the frames that need
/// them: an 85 MB strip takes a good part of a second to decode, and a frame needs a new one only
/// every octave. Each strip's escape range ([`StripRange`]) is kept after its texels have gone to the
/// GPU, for `--normalize`.
pub(crate) struct StripReader {
    dive: Arc<Dive>,
    pending: HashMap<u64, JoinHandle<Result<(Vec<f32>, StripRange), String>>>,
    /// Read, not yet taken (a strip read early for an anchor's range).
    ready: HashMap<u64, Vec<f32>>,
    pub(crate) ranges: HashMap<u64, Arc<StripRange>>,
}

impl StripReader {
    pub(crate) fn new(dive: Arc<Dive>) -> Self {
        StripReader { dive, pending: HashMap::new(), ready: HashMap::new(), ranges: HashMap::new() }
    }

    /// Start reading strip `k` if it is in the dive and not already read or on its way.
    pub(crate) fn ask(&mut self, k: u64) {
        if k >= self.dive.strips || self.pending.contains_key(&k) || self.ready.contains_key(&k) {
            return;
        }
        let dive = self.dive.clone();
        let h = std::thread::Builder::new().name(format!("strip-read-{k}")).spawn(move || {
            let path = dive.dir.join(strip_file(k));
            if !path.exists() {
                return Err(format!(
                    "strip {k} is not in {} (render it: --render-expmap with --expmap-strips {k})",
                    dive.dir.display()
                ));
            }
            let main = strip_main(&path, &dive.map)?;
            let range = strip_range(&main, dive.map.width as usize, dive.map.rows as usize);
            Ok((main, range))
        });
        match h {
            Ok(h) => {
                self.pending.insert(k, h);
            }
            Err(e) => crate::diag::log_line("assemble", &format!("could not start reading strip {k}: {e}")),
        }
    }

    /// Finish reading strip `k` (starting now if nobody asked), into `ready` and `ranges`.
    fn read(&mut self, k: u64) -> Result<(), String> {
        if self.ready.contains_key(&k) {
            return Ok(());
        }
        self.ask(k);
        let h = self.pending.remove(&k).ok_or_else(|| format!("strip {k} could not be read"))?;
        let (main, range) = h.join().map_err(|_| format!("reading strip {k} panicked"))??;
        self.ready.insert(k, main);
        self.ranges.insert(k, Arc::new(range));
        Ok(())
    }

    /// Strip `k`'s texels, for the GPU.
    pub(crate) fn take(&mut self, k: u64) -> Result<Vec<f32>, String> {
        self.read(k)?;
        self.ready.remove(&k).ok_or_else(|| format!("strip {k} could not be read"))
    }

    /// Make sure strips `first..=last` have their ranges (reading any that have none yet).
    pub(crate) fn ensure_ranges(&mut self, first: u64, last: u64) -> Result<(), String> {
        for k in first..=last.min(self.dive.strips.saturating_sub(1)) {
            if !self.ranges.contains_key(&k) {
                self.read(k)?;
            }
        }
        Ok(())
    }

    /// Forget the ranges of strips shallower than `k` (a dive only goes deeper).
    pub(crate) fn forget_before(&mut self, k: u64) {
        self.ranges.retain(|&s, _| s >= k);
    }
}

/// Strips rendered as a video's frames need them (`--render-zoom-video`), in dive order, on the
/// dive's one reference, and written to a folder as well when one is given (`--keep-strips`, in
/// Fractadyne's own format, for `--assemble` to recolour later). A strip's texels wait in `ready`
/// until the window takes them; its escape range stays for `--normalize`.
pub(crate) struct StripRenderer {
    pub(crate) plan: crate::expmap_render::DivePlan,
    dive_ref: Option<crate::render::ReuseRef>,
    next: u64,
    ready: HashMap<u64, Vec<f32>>,
    pub(crate) ranges: HashMap<u64, Arc<StripRange>>,
    keep: Option<PathBuf>,
    /// Seconds spent rendering strips.
    pub(crate) render_s: f64,
}

/// Where a video's strips come from.
pub(crate) enum StripSource {
    /// A dive's folder (`--assemble DIR`).
    Folder(StripReader),
    /// Rendered on demand (`--render-zoom-video`).
    Render(Box<StripRenderer>),
}

impl StripSource {
    fn ranges(&self) -> &HashMap<u64, Arc<StripRange>> {
        match self {
            StripSource::Folder(r) => &r.ranges,
            StripSource::Render(r) => &r.ranges,
        }
    }

    fn forget_before(&mut self, k: u64) {
        match self {
            StripSource::Folder(r) => r.forget_before(k),
            StripSource::Render(r) => r.ranges.retain(|&s, _| s >= k),
        }
    }
}

impl crate::FractadyneApp {
    /// Render strips `src.next..=k` (those not rendered yet) into `src.ready`.
    fn render_strips_through(
        &mut self,
        src: &mut StripRenderer,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        k: u64,
    ) -> Result<(), String> {
        let last = k.min(src.plan.strips.saturating_sub(1));
        while src.next <= last {
            let j = src.next;
            let t = std::time::Instant::now();
            let p = &src.plan;
            let s = self.render_strip(device, queue, &p.map, j, &p.cx, &p.cy, p.caps[j as usize], src.dive_ref.as_ref())?;
            if let Some(dir) = &src.keep {
                crate::expmap_render::write_strip_file(
                    &dir.join(strip_file(j)),
                    &p.map,
                    j,
                    StripFormat::Fractadyne,
                    p.power,
                    (&p.cx, &p.cy),
                    &s.result,
                    &s.req,
                )?;
            }
            let secs = t.elapsed().as_secs_f64();
            src.render_s += secs;
            crate::diag::log_line(
                "assemble",
                &format!(
                    "strip {j}/{}: 2^{:.2}, cap {}, mode {}, {}, {:.0} ms",
                    p.strips,
                    p.map.frame_log2mag(j, p.frame.0, p.frame.1),
                    s.req.max_iter,
                    s.req.mode,
                    if s.req.mode == 1 { "direct" } else if s.reused { "the dive's reference" } else { "its own reference" },
                    secs * 1000.0
                ),
            );
            let main = s.result.pixels;
            src.ranges.insert(j, Arc::new(strip_range(&main, p.map.width as usize, p.map.rows as usize)));
            src.ready.insert(j, main);
            src.next += 1;
        }
        Ok(())
    }

    /// Strip `k`'s texels, for the GPU: read, or rendered (with any strips before it not yet rendered).
    fn take_strip(
        &mut self,
        src: &mut StripSource,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        k: u64,
    ) -> Result<Vec<f32>, String> {
        match src {
            StripSource::Folder(r) => r.take(k),
            StripSource::Render(r) => {
                self.render_strips_through(r, device, queue, k)?;
                r.ready.remove(&k).ok_or_else(|| format!("strip {k} was rendered and has left the window; a dive only goes deeper"))
            }
        }
    }

    /// Put the strips a frame at `place` reads into the assembler's window (asking a folder for the
    /// next `ahead` beyond them).
    fn fill_window(
        &mut self,
        asm: &mut Assembler,
        src: &mut StripSource,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        place: &FramePlacement,
        ahead: u64,
    ) -> Result<(), String> {
        let (first, last) = asm.strips_for(place).ok_or("the frame is shallower than the dive's first strip")?;
        if let StripSource::Folder(r) = src {
            for k in first..=last + ahead {
                if !asm.has_strip(k) {
                    r.ask(k);
                }
            }
        }
        for k in first..=last {
            if !asm.has_strip(k) {
                let main = self.take_strip(src, device, queue, k)?;
                asm.put_strip(queue, k, &main).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    /// The palette window of the frame at depth `l`, reading or rendering the strips it shows first.
    fn measured_range(
        &mut self,
        dive: &Dive,
        src: &mut StripSource,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        l: f64,
    ) -> Result<Option<(f32, f32)>, String> {
        let (fw, fh) = dive.frame;
        let place = placement(&dive.map, l, fh, 1, 0.0);
        if let Some((first, last)) = fractadyne_gpu::strips_for_frame(&place, fw as u32, fh as u32, dive.strips) {
            match src {
                StripSource::Folder(r) => r.ensure_ranges(first, last)?,
                StripSource::Render(r) => {
                    // Strips are rendered in order, so a range not rendered yet is one ahead.
                    self.render_strips_through(r, device, queue, last)?;
                }
            }
        }
        frame_range(&dive.map, src.ranges(), dive.strips, l, fw, fh, crate::tunables::cost().norm_top_quantile)
    }
}

impl crate::FractadyneApp {
    /// Map the palette onto a video frame's window `[lo, top]` in `req`: the live view's map
    /// (`norm_map_for`), with its linear-to-log switch made a dissolve ([`log_weight`]) through the
    /// colour pass's palette fade.
    pub(crate) fn apply_video_norm(&self, req: &mut fractadyne_gpu::ExportRequest, lo: f32, top: f32) {
        let w = log_weight(lo, top, self.coloring.log_palette);
        let (lin, log) = (self.norm_map_linear(lo, top), self.norm_map_log(lo, top));
        let main = if w >= 1.0 { &log } else { &lin };
        req.cycle = main.cycle;
        req.offset = main.offset;
        req.norm_mode = main.mode;
        req.norm_lo = main.lo;
        req.palette_fade = if w > 0.0 && w < 1.0 {
            fractadyne_gpu::PaletteFade { weight: w, cycle: log.cycle, offset: log.offset, mode: log.mode, lo: log.lo }
        } else {
            fractadyne_gpu::PaletteFade::default()
        };
    }

    /// The colouring an assembled frame gets: the session's (palette, method, cycle, effects), as an
    /// export of the dive's first frame would carry it. Built at 1×, where Mandelbrot renders
    /// directly, so no reference orbit is computed for what is only a colouring.
    pub(crate) fn assembly_colour_request(&self, dive: &Dive) -> fractadyne_gpu::ExportRequest {
        let vp = Viewport::new(dive.frame.0, dive.frame.1);
        self.current_export_request_for(&vp, false)
    }

    /// What every assembly sets up: the "Fd" mark, the samples a pixel, the colouring (refusing one a
    /// strip cannot give), and the GPU window for the dive's frames. `source` names where the strips
    /// come from, for the log.
    fn begin_assembly(
        &mut self,
        ctx: &egui::Context,
        device: &eframe::wgpu::Device,
        dive: &Dive,
        source: &str,
    ) -> Result<(Assembler, u32, fractadyne_gpu::ExportRequest), String> {
        // The "Fd" mark goes on every frame, as on a tour's (rasterized once, from the font atlas).
        if self.watermark && self.watermark_overlay.is_none() {
            self.watermark_overlay = self.show_watermark.then(|| crate::export::build_watermark_overlay(ctx)).flatten();
        }
        let ss = self.render_cli.assemble_opts.ss.unwrap_or_else(|| (dive.oversample.round() as u32).clamp(1, 8));
        let (fw, fh) = (dive.frame.0 as u32, dive.frame.1 as u32);
        let colour = self.assembly_colour_request(dive);
        if fractadyne_gpu::method_needs_aux(colour.color_method) {
            return Err("this colouring reads per-orbit statistics (stripe, TIA, trap or decomposition) that strips do not keep; \
                        assemble with smooth, distance, relief or glow"
                .into());
        }
        let layers = fractadyne_gpu::layers_for_frame(fw * ss, fh * ss);
        let asm = Assembler::new(device, shape_of(dive), [fw, fh], ss, layers).map_err(|e| e.to_string())?;
        crate::diag::log_line(
            "assemble",
            &format!(
                "{source}: a {} dive in {} strips of {}x{}, frames {fw}x{fh} at {ss}x{ss} samples, a window of {layers} strips ({:.0} MB)",
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
        Ok((asm, ss, colour))
    }

    /// The palette follows the escape range when asked (`--normalize`, or the session's "Normalize
    /// deep colors"): past ~1e50× a fixed cycle aliases the escape field into confetti.
    fn assembly_normalizes(&self) -> bool {
        self.coloring.normalize || self.coloring.normalize_live
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
        let (mut asm, ss, colour) = self.begin_assembly(ctx, device, &dive, &dir.display().to_string())?;
        let mut src = StripSource::Folder(StripReader::new(dive.clone()));
        let (fw, fh) = (dive.frame.0 as u32, dive.frame.1 as u32);

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
                let place = placement(&dive.map, l, dive.frame.1, ss, opts.rotate.unwrap_or(0.0) * (l - dive.start_log2mag));
                self.fill_window(&mut asm, &mut src, device, queue, &place, 0)?;
                let ext = if raw { "exr" } else { "png" };
                let path = if several { out.join(format!("frame_{l:.3}.{ext}")) } else { out.clone() };
                let meta = format!("app=Fractadyne\nassembled_from={}\nlog2mag={l}\nss={ss}\n", dir.display());
                if raw {
                    let main = asm.reprojected(device, queue, &place).map_err(|e| e.to_string())?;
                    fractadyne_export::write_exr_raw(&path, fw * ss, fh * ss, &main, Some(&meta))
                        .map_err(|e| format!("{}: {e}", path.display()))?;
                } else {
                    // A single frame normalizes to its own window.
                    let mut c = colour.clone();
                    if self.assembly_normalizes() {
                        if let Some((lo, top)) = self.measured_range(&dive, &mut src, device, queue, l)? {
                            self.apply_video_norm(&mut c, lo, top);
                        }
                    }
                    let mut rgba = asm.frame(device, queue, &place, &c).map_err(|e| e.to_string())?;
                    self.apply_watermark(&mut rgba, fw, fh);
                    fractadyne_export::write_png(&path, fw, fh, &rgba, Some(&meta)).map_err(|e| format!("{}: {e}", path.display()))?;
                }
            }
            return Ok(format!("{} frame(s) assembled from {} into {}", opts.at.len(), dir.display(), out.display()));
        }

        let out = self.render_cli.out.clone().ok_or("--assemble makes a video: give its file with --out (e.g. dive.mp4)")?;
        let summary = self.assemble_video(device, queue, &dive, &mut src, &mut asm, ss, &colour, &out)?;
        Ok(format!("{summary}, from the strips in {}", dir.display()))
    }

    /// `--render-zoom-video F`: the dive into the view, rendered as strips and assembled into a video
    /// in one run — a strip is rendered just before the first frame that shows it and leaves the GPU
    /// with the last, so the dive never needs its strips on disk (`--keep-strips DIR` writes them
    /// too, for `--assemble` to recolour later).
    pub(crate) fn render_zoom_video(
        &mut self,
        ctx: &egui::Context,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        out: &Path,
    ) -> Result<String, String> {
        // Options a one-step video cannot honour are refused, never silently ignored.
        let eo = &self.render_cli.expmap_opts;
        if eo.format != StripFormat::Fractadyne {
            return Err("--render-zoom-video assembles Fractadyne's own strips; --expmap-format zoomasm is for --render-expmap".into());
        }
        if eo.strips.is_some() {
            return Err("--render-zoom-video renders the whole dive; --expmap-strips is for --render-expmap".into());
        }
        if !self.render_cli.assemble_opts.at.is_empty() {
            return Err("--render-zoom-video makes a video; for single frames render the strips (--render-expmap DIR), then --assemble DIR --assemble-at".into());
        }
        let plan = self.plan_dive(StripFormat::Fractadyne, "--render-zoom-video")?;
        let keep = self.render_cli.assemble_opts.keep.clone();
        if let Some(dir) = &keep {
            std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
            crate::expmap_render::write_or_check_manifest(dir, &plan.map, StripFormat::Fractadyne, &plan.dive)?;
        }
        let dive = Dive {
            dir: keep.clone().unwrap_or_default(),
            map: plan.map,
            strips: plan.strips,
            frame: plan.frame,
            oversample: plan.oversample,
            start_log2mag: plan.start_log2mag,
            end_log2mag: plan.end_log2mag,
            fractal: self.fractal.name().to_string(),
            centre: (fractadyne_core::to_decimal_string(&plan.cx), fractadyne_core::to_decimal_string(&plan.cy)),
        };
        let (mut asm, ss, colour) = self.begin_assembly(ctx, device, &dive, "rendering")?;
        crate::diag::log_line(
            "assemble",
            &format!(
                "the dive {} → 2^{:.2} into the view, caps {}..{}{}",
                plan.start_log2mag,
                plan.end_log2mag,
                plan.caps.first().copied().unwrap_or(0),
                plan.caps.last().copied().unwrap_or(0),
                keep.as_ref().map(|d| format!(", strips kept in {}", d.display())).unwrap_or_default(),
            ),
        );
        let saved = (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.export.width);
        let result = (|| -> Result<String, String> {
            // ⭐ONE reference for the whole dive, built for its deepest strip, as `--render-expmap`.
            let deepest = plan.strips - 1;
            let t = std::time::Instant::now();
            let dive_ref = self.dive_reference(&plan.map, deepest, &plan.cx, &plan.cy, plan.caps[deepest as usize]);
            crate::diag::log_line(
                "assemble",
                &match &dive_ref {
                    Some(d) => format!(
                        "one reference for the dive, from strip {deepest}: {} samples at {} bits in {:.0} ms",
                        d.prefix.len(),
                        d.prec,
                        t.elapsed().as_secs_f64() * 1000.0
                    ),
                    None => "no dive reference: the deepest strip renders directly".into(),
                },
            );
            let mut src = StripSource::Render(Box::new(StripRenderer {
                plan,
                dive_ref,
                next: 0,
                ready: HashMap::new(),
                ranges: HashMap::new(),
                keep,
                render_s: 0.0,
            }));
            let summary = self.assemble_video(device, queue, &dive, &mut src, &mut asm, ss, &colour, out)?;
            let strips_s = match &src {
                StripSource::Render(r) => r.render_s,
                StripSource::Folder(_) => 0.0,
            };
            Ok(format!("{summary}; {} strips rendered in {:.1} s", dive.strips, strips_s))
        })();
        (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.export.width) = saved;
        result
    }

    /// The video's frames, as `(log2 magnification, turns)`, its frame rate, and a description for
    /// the log: the whole dive at a constant zoom speed (`--assemble-speed`), or tour T's timeline
    /// (`--assemble-tour`: its keyframes' zooms, times, holds and easing, at its own frame rate unless
    /// `--assemble-fps` says otherwise), turning `--assemble-rotate` turns an octave either way.
    /// A tour must stay on the dive's centre and inside its depths: a strip video cannot pan.
    pub(crate) fn video_frames(opts: &AssembleCli, dive: &Dive) -> Result<(Vec<(f64, f64)>, f64, String), String> {
        let spin = opts.rotate.unwrap_or(0.0);
        let turns = |l: f64| spin * (l - dive.start_log2mag);
        let Some(path) = &opts.tour else {
            let fps = opts.fps.unwrap_or(30.0);
            let speed = opts.speed.unwrap_or(1.0);
            let frames = dive_frames(dive.start_log2mag, dive.end_log2mag, speed, fps).into_iter().map(|l| (l, turns(l))).collect();
            return Ok((
                frames,
                fps,
                format!("{speed} octaves a second (2^{:.2} to 2^{:.2})", dive.start_log2mag, dive.end_log2mag),
            ));
        };
        let pb = crate::scripting::parse_tour_file(path)?;
        let fps = opts.fps.or(pb.render.fps).unwrap_or(30.0);
        let n = crate::scripting::tour_frame_count(pb.total, fps);
        let p = fractadyne_core::precision_for_octaves(dive.end_log2mag.max(0.0) as u64 + 64);
        let parse = |s: &str| fractadyne_core::parse_bf_prec(s, p).ok_or_else(|| format!("the dive's centre {s} is unreadable"));
        let (cx, cy) = (parse(&dive.centre.0)?, parse(&dive.centre.1)?);
        let mut frames = Vec::with_capacity(n as usize);
        for f in 0..n {
            let t = (f as f64 / fps).min(pb.total);
            let s = pb.sample(t);
            let l = s.logmag / std::f64::consts::LN_2;
            if l < dive.start_log2mag - 1e-6 || l > dive.end_log2mag + 1e-6 {
                return Err(format!(
                    "{} reaches 2^{l:.2} at {t:.2} s, outside the dive's 2^{:.2} to 2^{:.2}",
                    path.display(),
                    dive.start_log2mag,
                    dive.end_log2mag
                ));
            }
            // On the centre to within a thousandth of a pixel of the frame it is at.
            let pixel_log2 = (Viewport::REFERENCE_HEIGHT / dive.frame.1).log2() - l;
            let off = fractadyne_core::log2_abs(&fractadyne_core::bf_sub(&s.cx, &cx, p))
                .max(fractadyne_core::log2_abs(&fractadyne_core::bf_sub(&s.cy, &cy, p)));
            if off > pixel_log2 - 10.0 {
                return Err(format!(
                    "{} is off the dive's centre at {t:.2} s (by 2^{off:.1}; a pixel there is 2^{pixel_log2:.1}): a zoom video \
                     from strips dives into one point, so start the tour where its last pan ends",
                    path.display()
                ));
            }
            frames.push((l, turns(l)));
        }
        Ok((frames, fps, format!("the timeline of {} ({:.1} s)", path.display(), pb.total)))
    }

    /// The whole dive as a video at a constant zoom speed (`--assemble-speed`, `--assemble-fps`),
    /// piped to ffmpeg as each frame is made; the palette normalized over it when asked.
    #[allow(clippy::too_many_arguments)]
    fn assemble_video(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        dive: &Dive,
        src: &mut StripSource,
        asm: &mut Assembler,
        ss: u32,
        colour: &fractadyne_gpu::ExportRequest,
        out: &Path,
    ) -> Result<String, String> {
        let opts = self.render_cli.assemble_opts.clone();
        let (fw, fh) = (dive.frame.0 as u32, dive.frame.1 as u32);
        let normalize = self.assembly_normalizes();
        let mut anchors = NormAnchors::new(dive.start_log2mag, dive.end_log2mag);
        let (frames, fps, how) = Self::video_frames(&opts, dive)?;
        let pipe = crate::video_pipe::VideoPipe::start(out, fw, fh, fps, opts.crf.unwrap_or(18))?;
        crate::diag::log_line(
            "assemble",
            &format!("{} frames at {fps} fps, {how} → {} (ffmpeg: {})", frames.len(), out.display(), pipe.ffmpeg.display()),
        );
        let progress = Arc::new(AtomicU32::new(0));
        let _pump = crate::diag::progress_pump("assemble", progress.clone());
        let t0 = std::time::Instant::now();
        let mut gpu_s = 0.0f64;
        let mut range_seen: Option<(f32, f32)> = None;
        for (f, &(l, turns)) in frames.iter().enumerate() {
            let place = placement(&dive.map, l, dive.frame.1, ss, turns);
            self.fill_window(asm, src, device, queue, &place, 2)?;
            let mut c = colour.clone();
            if normalize {
                let log_palette = self.coloring.log_palette;
                let r = anchors.range(l, |la| {
                    let r = self.measured_range(dive, src, device, queue, la)?;
                    crate::diag::log_line(
                        "assemble",
                        &match r {
                            Some((lo, top)) => format!(
                                "palette anchor at 2^{la:.2}: window {lo:.1} to {top:.1} ({:.0}% log)",
                                100.0 * log_weight(lo, top, log_palette)
                            ),
                            None => format!("palette anchor at 2^{la:.2}: nothing escaped"),
                        },
                    );
                    Ok(r)
                })?;
                if let Some((lo, top)) = r {
                    self.apply_video_norm(&mut c, lo, top);
                    range_seen = Some(range_seen.map_or((lo, top), |(a, b)| (a.min(lo), b.max(top))));
                }
                if let Some((first, _)) = asm.strips_for(&place) {
                    src.forget_before(first.saturating_sub(2));
                }
            }
            let tf = std::time::Instant::now();
            let mut rgba = asm.frame(device, queue, &place, &c).map_err(|e| e.to_string())?;
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
        if let Some((lo, top)) = range_seen {
            crate::diag::log_line(
                "assemble",
                &format!("normalized: the palette followed windows from {lo:.1} to {top:.1} over the dive, {} anchors", anchors.at.len()),
            );
        }
        Ok(format!(
            "{written} frames into {} ({:.1} s of video in {:.1} s; the GPU {:.1} ms a frame){}",
            out.display(),
            written as f64 / fps,
            secs,
            1000.0 * gpu_s / written.max(1) as f64,
            if normalize { ", palette normalized" } else { "" },
        ))
    }
}

#[cfg(test)]
#[path = "../tests/unit/assemble/tests.rs"]
mod tests;
