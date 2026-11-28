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

/// The live frames' size and budget (stage 2), as the twin self-test's.
const LW: u32 = 192;
const LH: u32 = 120;
const LITER: u32 = 20_000;

/// What a live check reads of a run of passes on a fresh [`fractadyne_gpu::LiveTwin`]: the view's
/// G-buffer (its counts and the base they are stored at) and the palette words it colors them with.
struct Live {
    g: fractadyne_gpu::GBuffer,
    words: Option<(f32, f32, f32)>,
}

#[inline(never)]
fn live_run(device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue, passes: &[fractadyne_gpu::MandelbrotParams]) -> Result<Live, String> {
    let mut t = fractadyne_gpu::LiveTwin::new(device, queue);
    for p in passes {
        t.frame(device, queue, p).map_err(|e| e.to_string())?;
    }
    let id = passes.first().map_or(0, |p| p.view_id);
    let g = t.gbuffer(device, queue, id, false).map_err(|e| e.to_string())?;
    Ok(Live { g, words: t.color_words(id) })
}

/// [`held`] for two G-buffers.
fn held_g(plain: &fractadyne_gpu::GBuffer, based: &fractadyne_gpu::GBuffer) -> Option<(u32, f64, usize, usize, usize)> {
    held(&Shot { px: plain.iter.clone(), base: plain.count_base, below: 0 }, &Shot { px: based.iter.clone(), base: based.count_base, below: 0 })
}

/// The palette words a frame of `p` colors a texture stored at `base` with, `(offset, lo)`: its
/// map's words rewritten about the base ([`fractadyne_gpu::count_rebased_map`]).
fn words_at(p: &fractadyne_gpu::MandelbrotParams, base: u32) -> (f32, f32) {
    let eq = p.equalize.as_ref().is_some_and(|e| e.share > 0.0 && e.sweeps > 0.0);
    fractadyne_gpu::count_rebased_map(p.norm_mode, eq, p.cycle, p.offset, p.norm_lo, p.wrap, base)
}

impl crate::FractadyneApp {
    /// The live view's own params for corpus 07 at 1.3e30x (floatexp), Smooth, as a whole frame at
    /// its full size; built on a fresh frame state and a cold reference cache, and the session's put
    /// back whole after (as the twin self-test builds them: `build_params` mutates far more frame
    /// state than a list of fields would catch).
    #[inline(never)]
    fn cb_live_params(&mut self) -> Option<Box<fractadyne_gpu::MandelbrotParams>> {
        const CRX: &str = "-1.178853950372678747911373866849720956148855";
        const CRY: &str = "0.1853420232408490265512092752061929308714979";
        let saved_vp = self.viewport.clone();
        let saved = (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.coloring.color_method);
        let fresh = crate::Perf { ts_supported: self.perf.ts_supported, wall_fallback: self.perf.wall_fallback, actuators: self.perf.actuators, ..Default::default() };
        let saved_perf = std::mem::replace(&mut self.perf, fresh);
        let saved_cache = self.ref_cache.clone();
        let saved_rx = std::mem::take(&mut self.recompute_rx);
        self.ref_cache[0] = Default::default();
        self.render_cfg.max_iter = LITER;
        self.render_cfg.auto_iter = false;
        self.coloring.color_method = crate::ColorMethod::Smooth;
        self.viewport.set_size(LW as f64, LH as f64);
        self.viewport.set_center_log2mag(fractadyne_core::parse_bf(CRX).unwrap(), fractadyne_core::parse_bf(CRY).unwrap(), 100.0);
        let live = |app: &mut Self| {
            app.perf.frame_idx += 1;
            let center_bf = [app.viewport.center_x.clone(), app.viewport.center_y.clone()];
            let center = app.viewport.center_f64();
            let span = app.viewport.complex_span_fe();
            let mag = app.viewport.magnification();
            let l2 = app.viewport.log2_magnification();
            app.build_params(center_bf, center, span, mag, l2, app.fractal, false, LITER, false, 1, [LW, LH], 0, None)
        };
        let mut warm = 0;
        while self.ref_cache[0].ref_pt.is_none() && warm < 400 {
            let _ = live(self);
            std::thread::sleep(std::time::Duration::from_millis(10));
            warm += 1;
        }
        let have_ref = self.ref_cache[0].ref_pt.is_some();
        let pr = live(self);
        self.viewport = saved_vp;
        (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.coloring.color_method) = saved;
        self.perf = saved_perf;
        self.ref_cache = saved_cache;
        self.recompute_rx = saved_rx;
        if !have_ref {
            return None;
        }
        let mut p = Box::new(pr.headless());
        p.tile = None;
        p.chunk_range = None;
        p.chunk_idx = 0;
        p.split = [1, 0];
        p.ss = 1;
        p.jitter = [0.0, 0.0];
        p.resolution = [LW, LH];
        p.count_base = 0;
        Some(p)
    }

    /// The live view's count base (coloring plan D2, stage 2), on a fresh live renderer each: a whole
    /// pass stores its counts at the base it asks for and colors them about it; a walk keeps the base
    /// its first window took, whatever its later windows ask; a split pass keeps the texture's.
    pub(super) fn count_base_live_checks(&mut self, device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue) -> Vec<SelfCheck> {
        let mut checks = Vec::new();
        let Some(p0) = self.cb_live_params() else {
            checks.push(SelfCheck {
                category: "consistency",
                name: "a live frame stores its counts at the base it asks for".into(),
                params: "corpus 07 at 1.3e30x".into(),
                result: "no reference was built".into(),
                threshold: "a reference",
                pass: false,
            });
            return checks;
        };
        let plain = live_run(device, queue, std::slice::from_ref(&*p0));
        let low = plain.as_ref().ok().and_then(|l| l.g.escape_range()).map(|r| r.0);
        let base = low.map_or(1, |v| (v.floor() as u32).saturating_sub(64).max(1));
        let at = |b: u32| fractadyne_gpu::MandelbrotParams { count_base: b, ..(*p0).clone() };
        // A whole pass at the base.
        let one = live_run(device, queue, &[at(base)]);
        let (want_o, want_lo) = words_at(&p0, base);
        let whole = match (&plain, &one) {
            (Ok(a), Ok(b)) => Some((held_g(&a.g, &b.g), b.words, b.words == Some((want_o, want_lo, base as f32)))),
            _ => None,
        };
        checks.push(SelfCheck {
            category: "consistency",
            name: "a live frame stores its counts at the base it asks for, and colors them about it".into(),
            params: format!("corpus 07 at 1.3e30x, {LW}x{LH}, {LITER} iterations, one pass; the base {base} (the lowest count {low:?} less 64)"),
            result: format!("((the base stored; the largest difference in steps of the old texel; pixels moved; distinct fractions at 0, at the base); the color words; the map rewritten about the base): {whole:?}"),
            threshold: "the base stored; within 1 step; none moved; the words the map's about the base",
            pass: whole.is_some_and(|(h, _, w)| w && h.is_some_and(|(cb, off, moved, f0, fb)| cb == base && off <= 1.0 && moved == 0 && fb >= f0)),
        });
        // A walk: its first window at the base, the later ones asking for another.
        let mid = plain.as_ref().ok().map_or(LITER / 2, |l| {
            let mut c: Vec<f32> = l.g.iter.chunks(4).map(|t| t[0]).filter(|v| *v >= 0.0).collect();
            c.sort_by(|a, b| a.total_cmp(b));
            c.get(c.len() / 2).map_or(LITER / 2, |v| (*v as u32).clamp(1, LITER - 1))
        });
        let window = |s: u32, e: u32, k: u32, b: u32| fractadyne_gpu::MandelbrotParams { chunk_range: Some([s, e]), chunk_idx: k, count_base: b, ..(*p0).clone() };
        let steady = live_run(device, queue, &[window(0, mid, 0, base), window(mid, LITER, 1, base), window(LITER, LITER, 2, base)]);
        let asked = live_run(device, queue, &[window(0, mid, 0, base), window(mid, LITER, 1, base + 65_536), window(LITER, LITER, 2, base + 65_536)]);
        let walk = match (&steady, &asked) {
            (Ok(s), Ok(a)) => Some((
                a.g.count_base,
                s.g.iter.len() == a.g.iter.len() && s.g.iter.iter().zip(&a.g.iter).all(|(x, y)| x.to_bits() == y.to_bits()),
                a.words.map(|w| w.2),
            )),
            _ => None,
        };
        checks.push(SelfCheck {
            category: "consistency",
            name: "a live walk keeps the base its first window took".into(),
            params: format!("the same view walked in windows [0, {mid}), [{mid}, {LITER}) and the tail: the first at the base {base}, the later ones asking for {}", base + 65_536),
            result: format!("(the base stored; the walk the walk at a steady base, bit for bit; the colors' base): {walk:?}"),
            threshold: "the first window's base; identical; colored at it",
            pass: walk == Some((base, true, Some(base as f32))),
        });
        // A split pass over a frame stored at 0, asking for the base: the texture keeps 0.
        let split = |j: u32| fractadyne_gpu::MandelbrotParams { split: [2, j], count_base: base, ..(*p0).clone() };
        let kept = live_run(device, queue, &[(*p0).clone(), split(0), split(1)]);
        let split_r = match (&plain, &kept) {
            (Ok(a), Ok(k)) => Some((k.g.count_base, a.g.iter.iter().zip(&k.g.iter).all(|(x, y)| x.to_bits() == y.to_bits()), k.words.map(|w| w.2))),
            _ => None,
        };
        checks.push(SelfCheck {
            category: "consistency",
            name: "a live split pass keeps the texture's base".into(),
            params: format!("a whole pass at base 0, then the two halves of a split asking for {base}"),
            result: format!("(the base stored; the frame the whole pass's, bit for bit; the colors' base): {split_r:?}"),
            threshold: "0; identical; colored at 0",
            pass: split_r == Some((0, true, Some(0.0))),
        });
        checks
    }

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
        // The live view's (stage 2).
        checks.extend(self.count_base_live_checks(device, queue));
        (
            (self.fractal, self.julia_mode, self.coloring.color_method, self.coloring.interior, self.coloring.transfer),
            (self.coloring.band_texture, self.coloring.stat_layer, self.coloring.stat_bump, self.coloring.stat_gloss, self.coloring.use_custom_palette),
            (self.effects.light, self.effects.de),
            (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.render_cfg.series_approx, self.render_cfg.use_bla),
        ) = saved;
        checks
    }
}
