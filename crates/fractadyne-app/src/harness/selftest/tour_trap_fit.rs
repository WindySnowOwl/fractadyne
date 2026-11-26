//! The self-test `tour-trap-fit` (user, 2026-11-26: a tour of a view with "Always fit range" and the
//! inside's fit came out pale, its palettes at the fixed rate, where the screen spread them over the
//! values in view). `FractadyneApp::fit_trap_to_frame` draws a tour frame once more for the trap's
//! values and colors it with the windows the live view would take from them.
//!
//! Checked on one view with both fits (a sine wave outside, the inside's own sine wave): the frame is
//! fitted under either fit and not without; the windows are the frame's own (its histogram, at the
//! live view's quantiles); they reach the request as the live map reads them; the picture changes by
//! it; nothing is carried from one frame to the next; a rendered tour's frame is the fitted one;
//! and a live view's windows (another place) neither color the frame nor are changed by it.

use super::{frame, SelfCheck};
use crate::{ColorMethod, FractalKind, InteriorMode, TrapPlace, TrapType};
use fractadyne_core::Viewport;

/// The frames' size.
const W: u32 = 320;
const H: u32 = 180;

/// One tour frame: its request at the fixed rate, the same fitted, and what was measured. Boxed,
/// and the renders below reduced to their pixels and windows in functions of their own: the
/// checks run inside the self-test's deep stack and render a whole tour from there, and this
/// function holding export results (their counters inline, some 23 KB each) overflowed the main
/// thread's stack.
struct Frame {
    fixed: Box<fractadyne_gpu::ExportRequest>,
    fitted: Box<fractadyne_gpu::ExportRequest>,
    fit: Option<crate::render::TrapFrameFit>,
}

type Window = Option<(f32, f32)>;

/// A render's picture (RGBA8) and the windows the live view would take from its own histograms.
struct Shot {
    px: Vec<u8>,
    windows: (Window, Window),
}

#[inline(never)]
fn shot(device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue, req: &fractadyne_gpu::ExportRequest) -> Option<Shot> {
    use fractadyne_gpu::{CTR_TRAP_HIST, CTR_TRAP_HIST_IN, TRAP_HIST_BUCKETS};
    let (progress, cancel) = (std::sync::atomic::AtomicU32::new(0), std::sync::atomic::AtomicBool::new(false));
    let r = fractadyne_gpu::render_export(device, queue, req, &progress, &cancel).ok()?;
    let hist = |at: usize| -> Vec<u32> { r.counters[at..at + TRAP_HIST_BUCKETS].iter().map(|&n| n.min(u64::from(u32::MAX)) as u32).collect() };
    Some(Shot {
        px: fractadyne_export::to_srgb8_dithered(&r.pixels, r.width),
        windows: (
            fractadyne_gpu::trap_window(&hist(CTR_TRAP_HIST), fractadyne_gpu::TRAP_WINDOW_QUANTILES),
            fractadyne_gpu::trap_window(&hist(CTR_TRAP_HIST_IN), fractadyne_gpu::INSIDE_WINDOW_QUANTILES),
        ),
    })
}

/// A request's colors, the fields a fit sets.
fn colors(r: &fractadyne_gpu::ExportRequest) -> (f32, f32, u32, f32, bool, f32, f32) {
    (r.cycle, r.offset, r.norm_mode, r.norm_lo, r.equalize.is_some(), r.interior.cycle, r.interior.offset)
}

impl crate::FractadyneApp {
    /// Frame 0 of a one-frame tour of the checks' view, with the method's trap at 30 degrees, as
    /// RGBA8 from its PNG: rendered by the tour's own loop, the mark and the location stamp off.
    #[inline(never)]
    fn ttf_tour_frame(&mut self, device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue) -> Result<Vec<u8>, String> {
        let dir = std::env::temp_dir().join(format!("fractadyne-selftest-tour-trap-fit-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let script = format!(
            "format_version = 2\nname = \"Tour trap fit check\"\n\n[render]\nsize = \"{W}x{H}\"\nfps = 1\nss = 1\n\n\
             [[keyframe]]\nt = 0\nre = \"0.3182553503775252\"\nim = \"-0.0409738111817041\"\nzoom = 600.0\nmax_iter = 2000\ntrap_phase = 30.0\n\n\
             [[keyframe]]\nt = 1\n"
        );
        let path = dir.join("tour.toml");
        // ⚠Everything a tour render sets, put back after it (the list `tour-equalize` keeps): left
        // at the tour's 320 px, the export width made the later `appearance` group render its
        // frames at 320x180, where Binary's small inside vanished and its check failed (a full
        // self-test, 2026-11-26; that group alone passed).
        let saved = (self.watermark, self.show_location);
        let saved_tour = (
            self.fractal,
            self.julia_mode,
            self.dual,
            self.julia_c,
            self.viewport.clone(),
            self.render_cfg.max_iter,
            self.render_cfg.auto_iter,
            self.export.ss,
            self.export.width,
        );
        (self.watermark, self.show_location) = (false, false);
        let ctx = egui::Context::default();
        let cfg = crate::scripting::TourRenderConfig {
            out: Some(dir.join("frames")),
            prefix: Some("frame".into()),
            overwrite: true,
            ..Default::default()
        };
        let result = std::fs::write(&path, script)
            .map_err(|e| e.to_string())
            .and_then(|_| self.render_tour_to_dir(&ctx, device, queue, &path, &cfg).map(|_| ()))
            .and_then(|_| {
                fractadyne_export::read_png_rgba8(&dir.join("frames").join("frame_00000.png")).map(|(_, _, px)| px).map_err(|e| e.to_string())
            });
        (self.watermark, self.show_location) = saved;
        (
            self.fractal,
            self.julia_mode,
            self.dual,
            self.julia_c,
            self.viewport,
            self.render_cfg.max_iter,
            self.render_cfg.auto_iter,
            self.export.ss,
            self.export.width,
        ) = saved_tour;
        let _ = std::fs::remove_dir_all(&dir);
        result
    }

    /// The tour frame of `vp` with the method's trap at `phase`.
    #[inline(never)]
    fn ttf_frame(&mut self, device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue, vp: &Viewport, phase: f32) -> Frame {
        self.coloring.trap_place.phase = fractadyne_core::coloring::wrap_degrees(phase);
        let mut fixed = Box::new(self.current_export_request_for(vp, false));
        (fixed.width, fixed.height, fixed.ss) = (W, H, 1);
        let mut fitted = fixed.clone();
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let fit = self.fit_trap_to_frame(device, queue, &mut fitted, &cancel);
        Frame { fixed, fitted, fit }
    }

    pub(super) fn tour_trap_fit_checks(&mut self, device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue) -> Vec<SelfCheck> {
        let saved_spec = self.coloring_spec();
        let saved = (self.fractal, self.julia_mode, self.dual, self.render_cfg.max_iter, self.render_cfg.auto_iter, self.export.ss);
        let saved_windows = (self.perf.trap_pal, self.perf.trap_shown, self.perf.inside_pal, self.perf.inside_shown);
        let saved_eq = [self.perf.trap_eq_shown[0].take(), self.perf.trap_eq_shown[1].take()];
        self.fractal = FractalKind::Mandelbrot;
        (self.julia_mode, self.dual) = (false, false);
        (self.render_cfg.auto_iter, self.render_cfg.max_iter, self.export.ss) = (false, 2_000, 1);
        (self.coloring.use_custom_palette, self.coloring.use_duotone, self.coloring.use_binary) = (false, false, false);
        self.coloring.palette_idx = fractadyne_color::preset_by_id("ice").unwrap_or(0);
        self.coloring.color_method = ColorMethod::OrbitTrap;
        self.coloring.trap_type = TrapType::Sine;
        self.coloring.trap_place = TrapPlace { phase: 30.0, ..TrapPlace::DEFAULT };
        self.coloring.interior = Some(InteriorMode::Trap);
        self.coloring.interior_trap = Some(TrapType::Sine);
        self.coloring.interior_trap_place = TrapPlace { size: 0.5, phase: -45.0, ..TrapPlace::DEFAULT };
        self.coloring.interior_palette = fractadyne_color::preset_by_id("ember");
        (self.coloring.normalize_fit, self.coloring.interior_fit) = (true, true);
        self.coloring.transfer = fractadyne_core::coloring::Transfer::Linear;
        (self.coloring.equalize, self.coloring.cycle, self.coloring.offset) = (0.0, 0.41, 0.38);
        // No live view's windows: the requests' colors start at the fixed rate, the control.
        (self.perf.trap_pal, self.perf.trap_shown, self.perf.inside_pal, self.perf.inside_shown) = ([None; 2], [None; 2], [None; 2], [None; 2]);
        // By the main cardioid at 600x: a bulb's inside and the outside around it.
        let mut vp = Viewport::new(W as f64, H as f64);
        let at = |t: &str| fractadyne_core::parse_bf_prec(t, 128).expect("a center");
        vp.set_center_mag(at("0.3182553503775252"), at("-0.0409738111817041"), 600.0);
        let mut checks = Vec::new();
        let yn = |b: bool| if b { "yes" } else { "NO" };

        // Fitted under either fit, and not without.
        let both = self.trap_fit_wanted();
        self.coloring.normalize_fit = false;
        let inside_only = self.trap_fit_wanted();
        self.coloring.interior_fit = false;
        let neither = self.trap_fit_wanted();
        self.coloring.normalize_fit = true;
        let outside_only = self.trap_fit_wanted();
        self.coloring.interior_fit = true;
        checks.push(SelfCheck {
            category: "Script",
            name: "tour trap fit: a tour frame is fitted under either trap fit, and not without".into(),
            params: "Orbit trap, the inside by its own trap: Always fit range and the inside's fit, each and neither".into(),
            result: format!(
                "fitted under both: {}, the outside's alone: {}, the inside's alone: {}, neither: {}",
                yn(both),
                yn(outside_only),
                yn(inside_only),
                if neither { "FITTED" } else { "not fitted" }
            ),
            threshold: "fitted under each, not under neither",
            pass: both && outside_only && inside_only && !neither,
        });

        // The frame's own windows, into its request as the live map reads them; the picture changes.
        let a = self.ttf_frame(device, queue, &vp, 30.0);
        let (fitted_px, fixed_px) = (shot(device, queue, &a.fitted), shot(device, queue, &a.fixed));
        let (window, inside) = fitted_px.as_ref().map_or((None, None), |s| s.windows);
        let measured = a.fit.map(|f| (f.window, f.inside));
        checks.push(SelfCheck {
            category: "Script",
            name: "tour trap fit: the windows are the frame's own values', at the live view's quantiles".into(),
            params: format!("{W}x{H}, the fit's pre-pass against the frame's own histograms"),
            result: format!("measured {measured:?}; the frame's own {:?}", (window, inside)),
            threshold: "both measured, and equal to the frame's own",
            pass: window.is_some() && inside.is_some() && measured == Some((window, inside)),
        });
        let expected = match (window, inside) {
            (Some((lo, hi)), Some((ilo, ihi))) => {
                let m = self.norm_map_for(lo, hi);
                let (ic, io) = crate::render::inside_fit_map(ilo, ihi, self.coloring.offset);
                Some((m.cycle, m.offset, m.mode, m.lo, false, ic, io))
            }
            _ => None,
        };
        checks.push(SelfCheck {
            category: "Script",
            name: "tour trap fit: the frame is colored by the live view's map of its windows".into(),
            params: "the request's palette map and the inside's, against the live view's for the same windows".into(),
            result: format!("request {:?}; the live map {expected:?}", colors(&a.fitted)),
            threshold: "the same",
            pass: expected == Some(colors(&a.fitted)),
        });
        let moved = match (&fixed_px, &fitted_px) {
            (Some(f), Some(g)) => Some(frame::distance(&f.px, &g.px)),
            _ => None,
        };
        checks.push(SelfCheck {
            category: "Script",
            name: "tour trap fit: the fit changes the picture from the fixed rate".into(),
            params: format!("{W}x{H}, the fitted frame against the same frame at the fixed rate"),
            result: moved.map_or("did not render".into(), |d| format!("{d:.2} levels apart")),
            threshold: ">= 2 levels",
            pass: moved.is_some_and(|d| d >= 2.0),
        });

        // Nothing carried from frame to frame: the phase turned and back gives the first frame again.
        let _b = self.ttf_frame(device, queue, &vp, 120.0);
        let a2 = self.ttf_frame(device, queue, &vp, 30.0 + 360.0);
        let again = shot(device, queue, &a2.fitted);
        let same_px = match (&fitted_px, &again) {
            (Some(f), Some(g)) => Some(frame::distance(&f.px, &g.px)),
            _ => None,
        };
        checks.push(SelfCheck {
            category: "Script",
            name: "tour trap fit: a frame's colors are its own (after a whole turn, the first frame again)".into(),
            params: "the phase at 30, 120, then 390 degrees".into(),
            result: format!(
                "colors at 30 {:?}, at 390 {:?}; pictures {}",
                colors(&a.fitted),
                colors(&a2.fitted),
                same_px.map_or("did not render".into(), |d| format!("{d:.4} levels apart"))
            ),
            threshold: "the same colors, the same picture",
            pass: colors(&a.fitted) == colors(&a2.fitted) && same_px == Some(0.0),
        });

        // Through the tour's own frame loop (`render_tour_to_dir`): its frame is the fitted one, so
        // the loop asks for the fit (a check of the function alone would pass without it).
        let (tour_px, tour_err) = match self.ttf_tour_frame(device, queue) {
            Ok(px) => (Some(px), None),
            Err(e) => (None, Some(e)),
        };
        let (to_fitted, to_fixed) = match (&tour_px, &fitted_px, &fixed_px) {
            (Some(t), Some(f), Some(x)) => (Some(frame::distance(t, &f.px)), Some(frame::distance(t, &x.px))),
            _ => (None, None),
        };
        checks.push(SelfCheck {
            category: "Script",
            name: "tour trap fit: a rendered tour's frame is the fitted frame".into(),
            params: format!("a one-frame tour of the same view and phase at {W}x{H}, its PNG against the frame fitted here and at the fixed rate"),
            result: match (to_fitted, to_fixed) {
                (Some(a), Some(b)) => format!("{a:.3} levels from the fitted frame, {b:.2} from the fixed rate's"),
                _ => format!("did not render: {}", tour_err.unwrap_or_else(|| "a frame".into())),
            },
            threshold: "<= 0.5 levels from the fitted frame, >= 2 from the fixed rate's",
            pass: to_fitted.is_some_and(|d| d <= 0.5) && to_fixed.is_some_and(|d| d >= 2.0),
        });

        // A live view's windows, of another place, neither color the frame nor are changed by it.
        let elsewhere = (Some((1.0f32, 2.0f32)), Some((3.0f32, 4.0f32)));
        (self.perf.trap_pal[0], self.perf.trap_shown[0]) = (elsewhere.0, elsewhere.0);
        (self.perf.inside_pal[0], self.perf.inside_shown[0]) = (elsewhere.1, elsewhere.1);
        let c = self.ttf_frame(device, queue, &vp, 30.0);
        let kept = (self.perf.trap_pal[0], self.perf.trap_shown[0], self.perf.inside_pal[0], self.perf.inside_shown[0]);
        let untouched = kept == (elsewhere.0, elsewhere.0, elsewhere.1, elsewhere.1) && self.perf.trap_eq_shown[0].is_none();
        checks.push(SelfCheck {
            category: "Script",
            name: "tour trap fit: a live view's windows neither color a tour frame nor are changed by it".into(),
            params: "the live view's windows set to another place's, then a frame fitted".into(),
            result: format!("frame colors {:?} (its own: {:?}); the live view's after {kept:?}", colors(&c.fitted), colors(&a.fitted)),
            threshold: "the frame's own colors; the live view's windows as they were",
            pass: colors(&c.fitted) == colors(&a.fitted) && untouched,
        });

        (self.perf.trap_pal, self.perf.trap_shown, self.perf.inside_pal, self.perf.inside_shown) = saved_windows;
        let [e0, e1] = saved_eq;
        (self.perf.trap_eq_shown[0], self.perf.trap_eq_shown[1]) = (e0, e1);
        (self.fractal, self.julia_mode, self.dual, self.render_cfg.max_iter, self.render_cfg.auto_iter, self.export.ss) = saved;
        self.apply_coloring_spec(&saved_spec);
        checks
    }
}
