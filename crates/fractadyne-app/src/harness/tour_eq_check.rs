//! The selftest's `tour-equalize` group (coloring plan C6b): a normalized tour's frames are
//! equalized by their keyframes' distributions, blended at each frame's time, and that is a property
//! of the TOUR — the same from an anchor file as measured, the same for a shard.
//!
//! A small tour (two keyframes, 7 frames at 160x90, Equalize colors at 100%) rendered through
//! `render_tour_to_dir` itself, several ways:
//! - measured on this machine (A);
//! - its anchors written with `--dump-norm-anchors`, then rendered from that file (B): the file holds
//!   both anchors' distributions (version 2), and B's frames are A's byte for byte;
//! - frames 2..5 alone, a farm's run (C): A's byte for byte;
//! - with Equalize colors at 0% (D, the CONTROL that it changed the picture).
//!
//! And the variable under test, directly: the middle frame (t = 1 s, halfway between the
//! keyframes) is rebuilt here through the frame loop's own setup (`enter_tour_frame`) and color
//! path, with the anchors' BLEND at its time (`norm_anchor_eq`): it must be A's frame byte for byte;
//! rebuilt with either keyframe's own distribution, or none, it must not be (else the comparison
//! cannot tell a blend from a nearest keyframe).

use crate::scripting::{anchor_file, norm_anchor_eq, norm_anchor_range, tour_frame_count, TourRenderConfig};
use fractadyne_core::Viewport;

/// The tour: Seahorse Valley from 2x to 200x in 2 s at 3 fps, normalized, its iterations fixed.
/// The view stays on the direct path (no reference orbit), so a frame rebuilt here is the loop's.
fn script(equalize: f32) -> String {
    format!(
        r#"format_version = 2
name = "Equalize tour check"

[render]
size = "160x90"
fps = 3
ss = 1
normalize = true
max_iter = 3000
auto_iter = false

[render.coloring]
equalize = {equalize:?}

[[keyframe]]
t = 0
re = "-0.743643887037151"
im = "0.131825904205330"
zoom = 2.0

[[keyframe]]
t = 2
re = "-0.743643887037151"
im = "0.131825904205330"
zoom = 200.0
"#
    )
}

const FRAMES: usize = 7;
/// Frame 3, t = 1 s: halfway between the keyframes' anchors (frames 0 and 6).
const MIDDLE: u64 = 3;
const W: u32 = 160;
const H: u32 = 90;

/// What the tour check found.
pub(crate) struct TourEqCheck {
    /// Frames A rendered.
    pub(crate) frames: usize,
    /// The anchor file: its version, and how many anchors carry a distribution.
    pub(crate) file_version: Option<i64>,
    pub(crate) file_distributions: usize,
    /// B's and C's frames identical to A's (of 7 and 3).
    pub(crate) from_file_same: usize,
    pub(crate) shard_same: usize,
    /// Bytes of the middle frame rebuilt with the blend that differ from A's (0 when it is A's).
    pub(crate) blend_bytes_apart: usize,
    /// Shares of the middle frame's pixels apart from A's when rebuilt with the first keyframe's
    /// distribution, the second's, and none.
    pub(crate) first_apart: f64,
    pub(crate) second_apart: f64,
    pub(crate) none_apart: f64,
    /// Mean difference (8-bit levels) of D's frames from A's.
    pub(crate) control_mean: f64,
}

impl TourEqCheck {
    pub(crate) fn pass(&self) -> bool {
        self.frames == FRAMES
            && self.file_version == Some(2)
            && self.file_distributions == 2
            && self.from_file_same == FRAMES
            && self.shard_same == 3
            && self.blend_bytes_apart == 0
            && self.first_apart >= 0.05
            && self.second_apart >= 0.05
            && self.none_apart >= 0.05
            && self.control_mean >= 2.0
    }
}

/// The frames a run wrote, by index.
fn frames_in(dir: &std::path::Path) -> Result<Vec<(u64, Vec<u8>)>, String> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let p = e.map_err(|e| e.to_string())?.path();
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        if let Some(i) = name.strip_prefix("frame_").and_then(|r| r.strip_suffix(".png")).and_then(|n| n.parse::<u64>().ok()) {
            let (_, _, px) = fractadyne_export::read_png_rgba8(&p).map_err(|e| format!("{}: {e}", p.display()))?;
            out.push((i, px));
        }
    }
    out.sort_by_key(|f| f.0);
    Ok(out)
}

/// The share of pixels whose color differs between two RGBA8 frames.
fn apart(a: &[u8], b: &[u8]) -> f64 {
    let n = a.len().min(b.len()) / 4;
    let d = a.chunks_exact(4).zip(b.chunks_exact(4)).filter(|(p, q)| p[..3] != q[..3]).count();
    d as f64 / n.max(1) as f64
}

impl crate::FractadyneApp {
    /// The tour check (see the module note). The caller sets the coloring it runs under (smooth,
    /// point-sampled) and restores the session's afterwards; this leaves the tour's in place.
    pub(crate) fn tour_equalize_check(&mut self, device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue) -> Result<TourEqCheck, String> {
        let dir = std::env::temp_dir().join(format!("fractadyne-selftest-tour-eq-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let result = self.tour_equalize_check_in(device, queue, &dir);
        let _ = std::fs::remove_dir_all(&dir);
        result
    }

    fn tour_equalize_check_in(&mut self, device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue, dir: &std::path::Path) -> Result<TourEqCheck, String> {
        let ctx = egui::Context::default();
        let text = script(1.0);
        let path = dir.join("tour.toml");
        std::fs::write(&path, &text).map_err(|e| e.to_string())?;
        let anchors = dir.join("anchors.toml");
        let run = |app: &mut Self, script: &std::path::Path, cfg: TourRenderConfig| app.render_tour_to_dir(&ctx, device, queue, script, &cfg).map(|_| ());
        // (The prefix said: by default it is the script file's name.)
        let into = |sub: &str| TourRenderConfig { out: Some(dir.join(sub)), prefix: Some("frame".into()), overwrite: true, ..Default::default() };
        // A: measured here.
        run(self, &path, into("a"))?;
        let a = frames_in(&dir.join("a"))?;
        // B: from the anchor file.
        run(self, &path, TourRenderConfig { dump_norm_anchors: Some(anchors.clone()), ..into("unused") })?;
        let file = std::fs::read_to_string(&anchors).map_err(|e| e.to_string())?;
        let file_version = file.lines().find_map(|l| l.strip_prefix("version = ")).and_then(|v| v.trim().parse().ok());
        let file_distributions = file.lines().filter(|l| l.starts_with("quantiles = ")).count();
        run(self, &path, TourRenderConfig { norm_anchors: Some(anchors.clone()), ..into("b") })?;
        let b = frames_in(&dir.join("b"))?;
        let from_file_same = a.iter().zip(&b).filter(|(x, y)| x.0 == y.0 && x.1 == y.1).count();
        // C: a farm's run of frames 2..5.
        run(self, &path, TourRenderConfig { frames: Some((2, 5)), ..into("c") })?;
        let c = frames_in(&dir.join("c"))?;
        let shard_same = c.iter().filter(|(i, px)| a.iter().any(|(j, q)| j == i && q == px)).count();
        // The middle frame rebuilt here, as the frame loop renders it, with the anchors the file holds.
        let pb = crate::scripting::parse_tour_file(&path)?;
        self.apply_tour_coloring(&pb.render);
        let fps = 3.0;
        let frames = tour_frame_count(pb.total, fps);
        let actx = anchor_file::AnchorContext {
            script_digest: anchor_file::script_digest(&text),
            fps,
            frames,
            base_iter: 3000,
            base_auto: false,
            total: pb.total,
        };
        let d = anchor_file::read(&anchors, &actx)?;
        let ranges: Vec<(f64, (f32, f32))> = d.anchors.iter().map(|a| (a.t, a.range)).collect();
        let eqs: Vec<(f64, fractadyne_core::coloring::Quantiles)> = d.anchors.iter().filter_map(|a| Some((a.t, a.eq?))).collect();
        if eqs.len() != 2 {
            return Err(format!("the anchor file holds {} distributions, not the 2 keyframes'", eqs.len()));
        }
        let t = anchor_file::frame_time(MIDDLE, fps, pb.total);
        let s = pb.sample(t);
        self.enter_tour_frame(&s, 3000, false);
        let mut vp = Viewport::new(W as f64, H as f64);
        vp.set_center_log2mag(s.cx.clone(), s.cy.clone(), s.logmag / std::f64::consts::LN_2);
        let r = norm_anchor_range(&ranges, t).ok_or("no range at the middle frame")?;
        let blend = norm_anchor_eq(&eqs, t);
        let rebuild = |app: &Self, eq: Option<fractadyne_core::coloring::Quantiles>| -> Result<Vec<u8>, String> {
            app.render_export_normalized(device, queue, &vp, app.julia_mode, W, H, 1, crate::render::NormRange::Fixed(r, eq), None, None, 2_000_000_000)
                .map(|(res, _)| fractadyne_export::to_srgb8_dithered(&res.pixels, W))
                .ok_or_else(|| "the middle frame did not render normalized".to_string())
        };
        let mid = a.iter().find(|f| f.0 == MIDDLE).map(|f| f.1.clone()).ok_or("frame 3 was not written")?;
        let with_blend = rebuild(self, blend)?;
        let blend_bytes_apart = with_blend.iter().zip(&mid).filter(|(x, y)| x != y).count() + with_blend.len().abs_diff(mid.len());
        let first_apart = apart(&rebuild(self, Some(eqs[0].1))?, &mid);
        let second_apart = apart(&rebuild(self, Some(eqs[1].1))?, &mid);
        let none_apart = apart(&rebuild(self, None)?, &mid);
        // D: Equalize colors off (the control).
        let off = dir.join("off.toml");
        std::fs::write(&off, script(0.0)).map_err(|e| e.to_string())?;
        run(self, &off, into("d"))?;
        let dd = frames_in(&dir.join("d"))?;
        let (mut sum, mut n) = (0.0f64, 0usize);
        for ((_, x), (_, y)) in a.iter().zip(&dd) {
            for (p, q) in x.chunks_exact(4).zip(y.chunks_exact(4)) {
                sum += (0..3).map(|k| (p[k] as f64 - q[k] as f64).abs()).sum::<f64>() / 3.0;
                n += 1;
            }
        }
        Ok(TourEqCheck {
            frames: a.len(),
            file_version,
            file_distributions,
            from_file_same,
            shard_same,
            blend_bytes_apart,
            first_apart,
            second_apart,
            none_apart,
            control_mean: sum / n.max(1) as f64,
        })
    }
}
