//! The self-test `count-base` (coloring plan D2): each escape count stored less a base, so that past
//! 2^21 the f32 texel keeps the fraction the count would lose (`fractadyne_gpu::CountBase`), and the
//! palette map rewritten about the base on the CPU (`count_rebased_map`).
//!
//! At a view of each render path (direct, df32 perturbation, floatexp with BLA, and the 9.3e78
//! minibrot's edge at 1e83x, counts of a third of a million to a million): a base below the frame's
//! lowest count colors as the frame without one, under the fixed rate and under a log map about the
//! frame's floor; its lanes hold each count less the base, as precisely as before or more; a base
//! above the lowest count is caught (`CTR_BELOW_BASE`). And a view whose budget reaches 2^21 but
//! whose counts stay low takes no base: the frame the render chooses for itself is the frame without
//! one, to the bit.

use super::SelfCheck;
use crate::FractalKind;
use fractadyne_core::Viewport;
use fractadyne_gpu::CountBase;

/// The frames' size.
const W: u32 = 160;
const H: u32 = 100;

/// What a check reads of a render: its pixels (the colors, or the lanes' main plane), the base it
/// was stored at, and the samples that escaped below it. Each render is reduced to this in a
/// function of its own, and its request boxed: an `ExportResult` holds its counters inline (some
/// 23 KB), and the checks run inside the self-test's deep stack, where a group holding a dozen of
/// them overflowed the main thread's stack (2026-11-27; see `tour_trap_fit`).
struct Shot {
    px: Vec<f32>,
    base: u32,
    below: u64,
}

/// The colored export of `req` at `base`.
#[inline(never)]
fn export(device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue, req: &fractadyne_gpu::ExportRequest, base: CountBase) -> Option<Shot> {
    let (progress, cancel) = (std::sync::atomic::AtomicU32::new(0), std::sync::atomic::AtomicBool::new(false));
    let q = Box::new(fractadyne_gpu::ExportRequest { count_base: base, ..req.clone() });
    match fractadyne_gpu::render_export(device, queue, &q, &progress, &cancel) {
        Ok(r) => Some(Shot { base: r.count_base, below: r.counters[fractadyne_gpu::CTR_BELOW_BASE], px: r.pixels }),
        Err(e) => {
            eprintln!("[selftest] GPU ERROR (render_export): {e}");
            None
        }
    }
}

/// The lanes of `req` at `base` (the main plane: count, normal, distance).
#[inline(never)]
fn lanes(device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue, req: &fractadyne_gpu::ExportRequest, base: CountBase) -> Option<Shot> {
    let q = Box::new(fractadyne_gpu::ExportRequest { count_base: base, ..req.clone() });
    let r = fractadyne_gpu::render_lanes(device, queue, &q, crate::tunables::CORRECT_WORK_BUDGET).ok()?;
    Some(Shot { base: r.count_base, below: r.counters[fractadyne_gpu::CTR_BELOW_BASE], px: r.pixels })
}

/// The largest channel difference of two colored frames, in levels.
fn worst(a: &Shot, b: &Shot) -> Option<f32> {
    (a.px.len() == b.px.len()).then(|| a.px.iter().zip(&b.px).enumerate().filter(|(i, _)| i % 4 != 3).map(|(_, (x, y))| (x - y).abs() * 255.0).fold(0.0f32, f32::max))
}

/// The lowest escaped count of a main plane.
fn lowest(px: &[f32]) -> Option<f32> {
    px.chunks_exact(4).map(|t| t[0]).filter(|v| *v >= 0.0).fold(None, |m: Option<f32>, v| Some(m.map_or(v, |m| m.min(v))))
}

/// The lanes at a base against the lanes at 0: (the base the lanes report; the largest difference,
/// in steps of the old texel, of each count less the base plus the base from the count; pixels
/// escaped in one and not the other; distinct fractions at 0, at the base).
fn held(plain: &Shot, based: &Shot) -> Option<(u32, f64, usize, usize, usize)> {
    if plain.px.len() != based.px.len() {
        return None;
    }
    let (mut off, mut moved, mut f0, mut fb) = (0.0f64, 0usize, std::collections::HashSet::new(), std::collections::HashSet::new());
    for (t0, tb) in plain.px.chunks_exact(4).zip(based.px.chunks_exact(4)) {
        if (t0[0] >= 0.0) != (tb[0] >= 0.0) {
            moved += 1;
            continue;
        }
        if t0[0] < 0.0 {
            continue;
        }
        let ulp = f64::from(f32::from_bits(t0[0].to_bits() + 1) - t0[0]);
        off = off.max((f64::from(tb[0]) + f64::from(based.base) - f64::from(t0[0])).abs() / ulp);
        f0.insert((f64::from(t0[0]).fract() * 1.0e6) as i64);
        fb.insert((f64::from(tb[0]).fract() * 1.0e6) as i64);
    }
    Some((based.base, off, moved, f0.len(), fb.len()))
}

impl crate::FractadyneApp {
    /// The export request for a view of the checks, at their size and one sample a pixel.
    #[inline(never)]
    fn cb_request(&mut self, re: &str, im: &str, mag: f64, iter: u32) -> Box<fractadyne_gpu::ExportRequest> {
        let at = |s: &str| fractadyne_core::parse_bf_prec(s, 512).expect("the view's coordinate");
        self.render_cfg.max_iter = iter;
        let mut vp = Viewport::new(W as f64, H as f64);
        vp.set_center_mag(at(re), at(im), mag);
        let mut r = Box::new(self.current_export_request_for(&vp, false));
        (r.width, r.height, r.ss) = (W, H, 1);
        r
    }

    /// The group's checks (see the module).
    pub(super) fn count_base_checks(&mut self, device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue) -> Vec<SelfCheck> {
        use fractadyne_core::coloring::BandTexture as B;
        let saved = (
            (self.fractal, self.julia_mode, self.coloring.color_method, self.coloring.interior, self.coloring.transfer),
            (self.coloring.band_texture, self.coloring.stat_layer, self.coloring.stat_bump, self.coloring.stat_gloss, self.coloring.use_custom_palette),
            (self.effects.light, self.effects.de),
            (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.render_cfg.series_approx, self.render_cfg.use_bla),
        );
        self.fractal = FractalKind::Mandelbrot;
        self.julia_mode = false;
        self.coloring.color_method = crate::ColorMethod::Smooth;
        self.coloring.interior = None;
        self.coloring.transfer = fractadyne_core::coloring::Transfer::Linear;
        (self.coloring.band_texture, self.coloring.stat_layer, self.coloring.stat_bump, self.coloring.stat_gloss) = (B::None, B::None, B::None, B::None);
        self.coloring.use_custom_palette = false;
        (self.effects.light, self.effects.de) = (false, false);
        (self.render_cfg.auto_iter, self.render_cfg.series_approx, self.render_cfg.use_bla) = (false, true, true);
        let mut checks = Vec::new();
        let sh = ("-0.7436438870371587047521915061147707", "0.131825904205311970493132056385139");
        let views: [(&str, &str, &str, f64, u32); 4] = [
            ("direct, the seahorse at 2e3x", sh.0, sh.1, 2e3, 2_000),
            ("df32, the seahorse at 1e6x", sh.0, sh.1, 1e6, 4_000),
            ("floatexp with BLA, corpus 07 at 1e30x", "-1.178853950372678747911373866849720956148855", "0.1853420232408490265512092752061929308714979", 1e30, 25_000),
            (
                "floatexp with BLA, the 9.3e78 minibrot's edge at 1e83x",
                "-0.095615882490732676063982994499490578147935343195074805633443741253353130261608559077156855936192455760390466637846658126512375332624285714286",
                "0.92407201125877521262558092669605306283394020346180079142271967031202190501107147692169317306941475085803150814402634231567774302992857142857",
                1e83,
                10_000_000,
            ),
        ];
        for (name, re, im, mag, iter) in views {
            let req = self.cb_request(re, im, mag, iter);
            let plain = lanes(device, queue, &req, CountBase::Fixed(0));
            let Some(low) = plain.as_ref().and_then(|l| lowest(&l.px)) else {
                checks.push(SelfCheck {
                    category: "consistency",
                    name: format!("a count base under the lowest count colors as before ({name})"),
                    params: format!("{W}x{H}, {iter} iterations"),
                    result: "no lanes, or no escaped pixel".into(),
                    threshold: "lanes with escaped pixels",
                    pass: false,
                });
                continue;
            };
            // A base 64 iterations under the lowest count (the colored frame's iterate can place a
            // pixel a few iterations off the lanes'), at least 1.
            let base = (low.floor() as u32).saturating_sub(64).max(1);
            let (e0, eb) = (export(device, queue, &req, CountBase::Fixed(0)), export(device, queue, &req, CountBase::Fixed(base)));
            // ...and under a log map about the frame's floor, whose floor the base moves.
            let log = Box::new(fractadyne_gpu::ExportRequest { norm_mode: 1, norm_lo: low, cycle: 0.3, ..(*req).clone() });
            let (l0, lb) = (export(device, queue, &log, CountBase::Fixed(0)), export(device, queue, &log, CountBase::Fixed(base)));
            let colors = match (&e0, &eb, &l0, &lb) {
                (Some(e0), Some(eb), Some(l0), Some(lb)) => Some((worst(e0, eb), worst(l0, lb), eb.base, eb.below + lb.below)),
                _ => None,
            };
            checks.push(SelfCheck {
                category: "consistency",
                name: format!("a count base under the lowest count colors as before ({name})"),
                params: format!("{W}x{H}, {iter} iterations, the base {base} (the lowest count {low:.2} less 64); the fixed rate, and a log map about the lowest count at cycle 0.3"),
                result: format!("(the largest channel difference in levels: fixed rate, log; the base the frame reports; samples below it): {colors:?}"),
                threshold: "at most 1 level in either map; the base reported; none below it",
                pass: colors.is_some_and(|(a, b, cb, n)| a.is_some_and(|a| a <= 1.0) && b.is_some_and(|b| b <= 1.0) && cb == base && n == 0),
            });
            // The lanes: each count less the base, to within the old texel's own step, and the
            // fraction kept at least as finely (at the deep edge, several times as finely).
            let based = lanes(device, queue, &req, CountBase::Fixed(base));
            let kept = plain.as_ref().zip(based.as_ref()).and_then(|(p, b)| held(p, b));
            let deep = low >= 65_536.0;
            checks.push(SelfCheck {
                category: "consistency",
                name: format!("lanes at a count base hold each count less the base ({name})"),
                params: format!("{W}x{H}, {iter} iterations, the base {base}: the count lane at the base plus the base against the lane at 0, pixel by pixel"),
                result: format!("(the base the lanes report; the largest difference in steps of the old texel; pixels escaped in one and not the other; distinct fractions at 0, at the base): {kept:?}"),
                threshold: if deep { "the base reported; within 1 step; none moved; 4 times the distinct fractions or more" } else { "the base reported; within 1 step; none moved; as many distinct fractions or more" },
                pass: kept.is_some_and(|(cb, off, moved, f0, fb)| cb == base && off <= 1.0 && moved == 0 && fb >= if deep { 4 * f0 } else { f0 }),
            });
            // A base above the lowest count: the frame says so.
            let above = low.floor() as u32 + 64;
            if let Some(over) = export(device, queue, &req, CountBase::Fixed(above)) {
                checks.push(SelfCheck {
                    category: "consistency",
                    name: format!("a count base above the lowest count is caught ({name})"),
                    params: format!("{W}x{H}, {iter} iterations, the base {above} (the lowest count plus 64)"),
                    result: format!("samples that escaped below it: {}", over.below),
                    threshold: "at least 1",
                    pass: over.below > 0,
                });
            }
        }
        // A budget that reaches 2^21 at a view whose counts stay in the thousands: the render
        // measures the frame and takes no base, the frame without one to the bit.
        let req = self.cb_request(views[2].1, views[2].2, views[2].3, fractadyne_gpu::COUNT_BASE_MIN);
        let same = match (export(device, queue, &req, CountBase::Auto), export(device, queue, &req, CountBase::Fixed(0))) {
            (Some(a), Some(z)) => Some((a.base, a.px.len() == z.px.len() && a.px.iter().zip(&z.px).all(|(x, y)| x.to_bits() == y.to_bits()))),
            _ => None,
        };
        checks.push(SelfCheck {
            category: "consistency",
            name: "a frame whose counts stay low takes no count base, to the bit".into(),
            params: format!("corpus 07 at 1e30x, {W}x{H}, {} iterations (2^21: the render measures the frame first)", fractadyne_gpu::COUNT_BASE_MIN),
            result: format!("(the base the render chose; every channel the same bits as at base 0): {same:?}"),
            threshold: "base 0, identical",
            pass: same == Some((0, true)),
        });
        (
            (self.fractal, self.julia_mode, self.coloring.color_method, self.coloring.interior, self.coloring.transfer),
            (self.coloring.band_texture, self.coloring.stat_layer, self.coloring.stat_bump, self.coloring.stat_gloss, self.coloring.use_custom_palette),
            (self.effects.light, self.effects.de),
            (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.render_cfg.series_approx, self.render_cfg.use_bla),
        ) = saved;
        checks
    }
}
