//! `--reusetest`: headless staleness harness for the reuse-first zoom (design/xaos-reuse.md,
//! Stage 0). For each view it renders a from-scratch reference, then for a series of Δ-octave
//! zoom-ins renders the true (from-scratch) zoomed frame AND the *reprojection* the live path
//! would show for that frame — the exact color-pass affine magnify of the reference — and reports
//! how far the reprojection has drifted from the truth as a function of Δ, in two metrics:
//!
//! - **raw smooth-iter** (`meanΔiter`, `%>2iter`): a conservative per-pixel escape-time proxy.
//! - **perceptual sRGB** (`sRGBmean`, `sRGBmax`, 0–255): what the user actually SEES — both the
//!   reprojected and the fresh iteration buffers are run through the real color pass
//!   (`color_iter_buffer`) with the view's coloring, then diffed in sRGB. This is the metric that
//!   sets `REFRESH_OCTAVES` and the from-scratch golden the Stage-2 during-motion refine is gated on.
//!
//! Touches no live-loop state (isolated `render_iter`/`color_iter_buffer` frames), so it is safe
//! and deterministic.

use crate::{ColorMethod, FractadyneApp, FractalKind};
use fractadyne_core::{parse_bf, precision_for_magnification, FloatExp, Viewport};

struct ReuseView {
    name: &'static str,
    cx: &'static str,
    cy: &'static str,
    mag: f64,
}

/// Max + mean absolute per-channel difference of two sRGB8 buffers (0–255), like the goldens.
fn img_diff(a: &[u8], b: &[u8]) -> (u32, f64) {
    if a.len() != b.len() || a.is_empty() {
        return (255, 255.0);
    }
    let (mut max, mut sum) = (0u32, 0u64);
    for (&x, &y) in a.iter().zip(b) {
        let d = (x as i32 - y as i32).unsigned_abs();
        if d > max {
            max = d;
        }
        sum += d as u64;
    }
    (max, sum as f64 / a.len() as f64)
}

impl FractadyneApp {
    pub(crate) fn run_reusetest(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
    ) {
        const N: u32 = 512;
        // Fixed iteration count across every render so the comparison isolates the *spatial*
        // reprojection error, not auto-iter count differences between the base and zoomed views.
        const ITER: u32 = 4000;
        // Hermetic config (mirrors the selftest baseline): the harness must not depend on the
        // live session. `--reusetest` exits via process::exit, so no restore is needed.
        self.fractal = FractalKind::Mandelbrot;
        self.julia_mode = false;
        self.dual = false;
        self.render_cfg.auto_iter = false;
        self.render_cfg.max_iter = ITER;
        self.render_cfg.series_approx = true;
        self.render_cfg.use_bla = true;
        self.render_cfg.glitch_correct = false;
        self.coloring.color_method = ColorMethod::Smooth;

        let views = [
            ReuseView { name: "seahorse-1e6", cx: "-0.743643887037151", cy: "0.131825904205330", mag: 1.0e6 },
            ReuseView { name: "seahorse-1e12", cx: "-0.743643887037151", cy: "0.131825904205330", mag: 1.0e12 },
            // Precise boundary point (the golden "mandelbrot-1e6"): richly escaping, so the
            // escaped-pixel comparison is well-populated (unlike a coarse-precision centre).
            ReuseView {
                name: "boundary-1e6",
                cx: "-7.219621882920463979621343199249635039400777157391994056859e-1",
                cy: "2.406540627640154659873781066416545013133592385797331352286e-1",
                mag: 1.0e6,
            },
        ];
        let deltas = [0.1_f64, 0.25, 0.5, 0.75, 1.0];

        // render_iter for a (center, mag) view → (request, RGBA32F iter buffer). Channel 0 = smooth
        // iter (<0 = interior); the request carries the coloring params for `color_iter_buffer`.
        let render = |app: &Self,
                      cx: &str,
                      cy: &str,
                      mag: f64|
         -> Option<(fractadyne_gpu::ExportRequest, Vec<f32>)> {
            let mut vp = Viewport::new(N as f64, N as f64);
            vp.center_x = parse_bf(cx)?;
            vp.center_y = parse_bf(cy)?;
            vp.units_per_pixel = FloatExp::from_f64(3.0 / (N as f64 * mag));
            vp.precision = precision_for_magnification(mag);
            let mut req = app.current_export_request_for(&vp, false);
            req.width = N;
            req.height = N;
            req.ss = 1;
            let px = fractadyne_gpu::render_iter(device, queue, &req).ok()?.pixels;
            Some((req, px))
        };
        // Color an iteration buffer through the real color pass, returned as sRGB8 (what the user sees).
        let colored = |req: &fractadyne_gpu::ExportRequest, iter: &[f32]| -> Option<Vec<u8>> {
            fractadyne_gpu::color_iter_buffer(device, queue, req, iter)
                .ok()
                .map(|r| fractadyne_export::to_srgb8(&r.pixels))
        };

        println!(
            "reuse-first zoom staleness — reprojection vs from-scratch, {N}×{N}, iter {ITER} (fixed).\n\
             `reprojection` = the exact color-pass affine magnify (uv_scale = 2^-Δ about centre) of the\n\
             base frame — what the live view shows while holding through a Δ-octave dive.\n\
             raw = smooth-iter (conservative); sRGB = the colored image the user actually sees (0–255).\n"
        );
        println!(
            "  {:<16} {:>5}  {:>6}  {:>7}  {:>9}  {:>9}  {:>8}",
            "view", "Δoct", "%escap", "raw%>2", "sRGB-near", "sRGB-bilin", "bilin-Δ"
        );
        println!("  {}", "-".repeat(72));

        let nn = N as usize;
        for v in &views {
            let Some((_base_req, base)) = render(self, v.cx, v.cy, v.mag) else {
                println!("  {:<16}  (render failed)", v.name);
                continue;
            };
            for &d in &deltas {
                let magd = v.mag * 2f64.powf(d);
                let Some((fresh_req, fresh)) = render(self, v.cx, v.cy, magd) else { continue };
                // Build two reprojected iteration buffers — the zoomed-view pixel (x,y) samples the
                // base frame at the magnified coordinate (2^Δ magnify → uv_scale = 2^-Δ):
                //  - `reproj_n`  NEAREST (what the shader does today, `pix = i32(suv*screen_dim)`);
                //  - `reproj_b`  class-guarded BILINEAR (bilerp the escape channel only where all 4
                //    neighbours escaped — else nearest, to avoid averaging interior(-1) with exterior;
                //    normal/DE always bilerp). This is the candidate color-pass improvement, measured
                //    before touching the fragile live shader.
                let s = 2f64.powf(-d);
                let mut reproj_n = vec![0.0f32; nn * nn * 4];
                let mut reproj_b = vec![0.0f32; nn * nn * 4];
                let (mut sum, mut n, mut big) = (0.0f64, 0u64, 0u64);
                let samp = |bx: usize, by: usize, c: usize| base[(by * nn + bx) * 4 + c];
                for y in 0..nn {
                    for x in 0..nn {
                        let fpx = (((x as f64 + 0.5) / N as f64 - 0.5) * s + 0.5) * N as f64 - 0.5;
                        let fpy = (((y as f64 + 0.5) / N as f64 - 0.5) * s + 0.5) * N as f64 - 0.5;
                        let x0 = (fpx.floor() as i64).clamp(0, N as i64 - 1) as usize;
                        let y0 = (fpy.floor() as i64).clamp(0, N as i64 - 1) as usize;
                        let x1 = (x0 + 1).min(nn - 1);
                        let y1 = (y0 + 1).min(nn - 1);
                        let (fx, fy) = (fpx - fpx.floor(), fpy - fpy.floor());
                        // nearest (round)
                        let nx = if fx >= 0.5 { x1 } else { x0 };
                        let ny = if fy >= 0.5 { y1 } else { y0 };
                        let dst = (y * nn + x) * 4;
                        for c in 0..4 {
                            reproj_n[dst + c] = samp(nx, ny, c);
                        }
                        // bilinear (class-guarded on channel 0 = escape)
                        let bilerp = |c: usize| -> f32 {
                            let a = samp(x0, y0, c) as f64 * (1.0 - fx) + samp(x1, y0, c) as f64 * fx;
                            let b = samp(x0, y1, c) as f64 * (1.0 - fx) + samp(x1, y1, c) as f64 * fx;
                            (a * (1.0 - fy) + b * fy) as f32
                        };
                        let all_esc = samp(x0, y0, 0) >= 0.0 && samp(x1, y0, 0) >= 0.0
                            && samp(x0, y1, 0) >= 0.0 && samp(x1, y1, 0) >= 0.0;
                        reproj_b[dst] = if all_esc { bilerp(0) } else { samp(nx, ny, 0) };
                        reproj_b[dst + 1] = bilerp(1);
                        reproj_b[dst + 2] = bilerp(2);
                        reproj_b[dst + 3] = bilerp(3);
                        // raw escape-time drift (nearest), over pixels escaped in both.
                        let (rn, tr) = (reproj_n[dst], fresh[dst]);
                        if rn >= 0.0 && tr >= 0.0 {
                            let e = (rn - tr).abs() as f64;
                            sum += e;
                            n += 1;
                            if e > 2.0 {
                                big += 1;
                            }
                        }
                    }
                }
                let _ = sum;
                let pct = if n > 0 { 100.0 * big as f64 / n as f64 } else { 0.0 };
                let escap = 100.0 * n as f64 / (nn * nn) as f64;
                // Perceptual: color each reprojection with the zoomed-view coloring, diff vs the
                // from-scratch colored frame, in sRGB.
                let Some(cf) = colored(&fresh_req, &fresh) else { continue };
                let near = colored(&fresh_req, &reproj_n).map(|c| img_diff(&cf, &c).1).unwrap_or(0.0);
                let bilin = colored(&fresh_req, &reproj_b).map(|c| img_diff(&cf, &c).1).unwrap_or(0.0);
                let improve = if near > 0.0 { 100.0 * (near - bilin) / near } else { 0.0 };
                println!(
                    "  {:<16} {:>5.2}  {:>5.1}%  {:>6.1}%  {:>9.2}  {:>9.2}  {:>7.1}%",
                    v.name, d, escap, pct, near, bilin, improve
                );
            }
        }
        println!(
            "\nRead: sRGB-near/bilin are perceptual staleness (mean 0–255) of the held frame vs a real\n\
             render at that dive depth; bilin-Δ is how much class-guarded BILINEAR reprojection beats\n\
             today's NEAREST. If bilin-Δ is large, a bilinear color-pass reproject (fs_color, reproject\n\
             branch — goldens are reproject=0 so untouched) is worth the change; if small, the staleness\n\
             is dominated by missing sub-pixel detail (only the Stage-2 during-motion refine helps).\n\
             Either way this is measured BEFORE touching the fragile live shader (design/xaos-reuse.md)."
        );
    }
}
