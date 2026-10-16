//! The self-test's and the UI test's ways into the live reference pipeline: the checks that
//! hold a reused, cached or installed reference to a fresh build, and the shims that let a
//! harness build, install or seed one. Attached to `render` (`#[path]` in render.rs) because they
//! need its private machinery (`pick_and_build`, `install_recompute`, the orbit cache); moved
//! from render.rs.

use super::*;

/// A FRESH pick + build of `inp` (no reuse, no disk cache), with or without the overlap of
/// [`pick_and_build`] — the selftest's A/B: the two must agree byte for byte.
pub(crate) fn fresh_build_for_selftest(inp: RecomputeInputs, overlap: bool) -> RecomputeResult {
    pick_and_build(&inp, overlap)
}

/// `--uitest`'s seed for the Reference cache window: build one orbit and write it to the store
/// SYNCHRONOUSLY, bypassing the build-time threshold, so the window photographs a populated
/// layout rather than chrome around no data. Returns whether an entry was written.
pub(crate) fn seed_orbit_cache_for_uitest(inp: RecomputeInputs) -> bool {
    let key = orbit_key_for(&inp);
    let res = recompute_worker(inp);
    let Some(tail) = &res.orbit_tail else { return false };
    let key = orbit_blob::OrbitKey { backend: tail.backend, ..key };
    orbit_blob::encode(&res, key)
        .and_then(|b| crate::refcache_persist::offer(b).ok().flatten())
        .is_some()
}

impl FractadyneApp {
    /// Install a reference the way the live render loop does — for `--selftest`, which has no
    /// render loop and therefore nothing resident to borrow.
    ///
    /// ⭐Without this the thumbnail check could only ever see the REFUSED branch, and the path the
    /// real app takes at depth would stay untested — which is how the freeze shipped.
    pub(crate) fn install_recompute_for_selftest(&mut self, vi: usize, res: RecomputeResult) {
        self.install_recompute(vi, res);
    }

    /// ⭐⭐**Does a REUSED reference render the same image as a freshly picked one?**
    ///
    /// `try_reuse_reference`'s doc asserts it does — *"perturbation is invariant to which valid
    /// in-view reference is used, so a reused reference renders the same image as a fresh one"* —
    /// and, two sentences later, that *"the render isn't perfectly invariant there"* at extreme
    /// depth. Both cannot be true, and nothing measured which. This does.
    ///
    /// ⚠⚠**Nothing else can.** The F3 corpus and the goldens run headless `--render`, where there
    /// is no live view and `reuse` is always `None`, so they cannot see this path at all — 38/38
    /// maxD 0 would stay green however wrong reuse got. A check that cannot go red is not a gate.
    ///
    /// Returns `(pass, human-readable result)`.
    pub(crate) fn selfcheck_reference_reuse(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        vp: &Viewport,
        size: u32,
        short_iter: u32,
    ) -> (bool, String) {
        self.selfcheck_reuse_identity(device, queue, vp, size, short_iter, &|s, _| {
            let tail = s.orbit_tail.clone().ok_or_else(|| "the short orbit carries no tail".to_string())?;
            Ok(ReuseRef { point: s.rp.clone(), prefix: s.orbit.clone(), tail, prec: s.prec })
        })
    }

    /// ⭐**The overlapped pick + build is the sequential one, byte for byte** ([`pick_and_build`]).
    /// Builds `vp` fresh both ways and compares everything a render reads from the result: the
    /// point, the orbit samples, the series skip and its coefficients, the BLA table, the
    /// precisions and the tail state. Returns the overlap arm's `(spec_build, sa_from)` and a
    /// summary, so the caller can require the overlap to have ENGAGED: an arm that fell back to
    /// the sequential path would compare that path with itself and pass.
    pub(crate) fn selfcheck_ref_overlap(
        &self,
        vp: &Viewport,
        max_iter: u32,
    ) -> Result<(&'static str, &'static str, bool, String), String> {
        let budget = IterBudget { max_iter, auto_iter: false };
        let inputs = || self.export_reference_inputs_for(vp, false, budget).ok_or("not a perturbation view");
        let t = Instant::now();
        let a = fresh_build_for_selftest(inputs()?, false);
        let seq_ms = t.elapsed().as_secs_f64() * 1000.0;
        let t = Instant::now();
        let b = fresh_build_for_selftest(inputs()?, true);
        let ovl_ms = t.elapsed().as_secs_f64() * 1000.0;
        let bits = |v: &[[f32; 4]]| v.iter().flat_map(|q| q.map(f32::to_bits)).collect::<Vec<u32>>();
        let sa_bits = |s: &fractadyne_core::SeriesSkip| {
            (s.skip, s.a.map(f32::to_bits), s.a_exp, s.b.map(f32::to_bits), s.b_exp, s.c.map(f32::to_bits), s.c_exp)
        };
        let tail_escaped = |r: &RecomputeResult| r.orbit_tail.as_ref().map(|t| t.escaped);
        let differs = [
            ("point", !same_point(&a.rp, &b.rp)),
            ("orbit", a.orbit_len != b.orbit_len || bits(&a.orbit) != bits(&b.orbit)),
            ("series skip", sa_bits(&a.sa) != sa_bits(&b.sa)),
            ("BLA", bits(&a.bla) != bits(&b.bla) || a.bla_dc_max_log2.to_bits() != b.bla_dc_max_log2.to_bits()),
            ("precision", (a.prec, a.req_prec, a.iter) != (b.prec, b.req_prec, b.iter)),
            ("tail", (a.partial, tail_escaped(&a)) != (b.partial, tail_escaped(&b))),
        ];
        if let Some((what, _)) = differs.iter().find(|(_, d)| *d) {
            return Err(format!("the overlapped build differs from the sequential one in its {what}"));
        }
        Ok((
            b.spec_build,
            b.sa_from,
            b.pick_centre_from_build,
            format!(
                "identical; len={} sa_skip={} sa={}/{} centre build={} pick's centre walk={} ({seq_ms:.0} ms sequential, {ovl_ms:.0} ms overlapped)",
                b.orbit_len,
                b.sa.skip,
                if a.sa_from.is_empty() { "none" } else { a.sa_from },
                if b.sa_from.is_empty() { "none" } else { b.sa_from },
                b.spec_build,
                if b.pick_centre_from_build { "from the build" } else { "walked" },
            ),
        ))
    }

    /// ⭐⭐**Does an orbit that went through the DISK render the same image as a fresh pick?**
    ///
    /// The on-disk cache (`refcache_persist`) exists so an extreme location costs seconds instead
    /// of an hour to return to; its failure mode is a WRONG picture arrived at quickly, which
    /// nothing downstream would notice. The codec's own tests pin the bytes; this pins the claim
    /// that matters, with a GPU: a reference written to disk, found by the cache's own lookup,
    /// decoded and EXTENDED renders bit-identically to one built fresh.
    ///
    /// Two arms, both with non-vacuity guards:
    /// 1. **Through the store**: the short reference is encoded, written, looked up and loaded,
    ///    and the identity check runs on what came back (reuse must ENGAGE and GROW).
    /// 2. **Unaided**: a worker given NO reuse hint must find the entry on its own, because a
    ///    lookup that silently misses falls back to a fresh build that renders the same picture
    ///    — the one failure the identity arm cannot see.
    ///
    /// ⚠Runs against a scratch directory, with the cache forced ON for its duration and restored
    /// after, so it neither reads nor pollutes the real cache. Background writes are drained
    /// before the restore, so none can land in the real directory afterwards.
    pub(crate) fn selfcheck_orbit_cache(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        vp: &Viewport,
        size: u32,
        short_iter: u32,
    ) -> (bool, String) {
        use crate::refcache_persist as cache;
        let scratch = std::env::temp_dir().join("fractadyne-selftest-orbits");
        let _ = std::fs::remove_dir_all(&scratch);
        let was_on = cache::enabled();
        cache::set_dir_override(Some(scratch.clone()));
        cache::set_enabled(true);
        let verdict = self.selfcheck_orbit_cache_inner(device, queue, vp, size, short_iter);
        cache::drain();
        cache::set_enabled(was_on);
        cache::set_dir_override(None);
        let _ = std::fs::remove_dir_all(&scratch);
        verdict
    }

    fn selfcheck_orbit_cache_inner(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        vp: &Viewport,
        size: u32,
        short_iter: u32,
    ) -> (bool, String) {
        use crate::refcache_persist as cache;
        let via = |s: &RecomputeResult, inp: &RecomputeInputs| -> Result<ReuseRef, String> {
            let tail = s.orbit_tail.as_ref().ok_or("the short orbit carries no tail")?;
            let key = orbit_blob::OrbitKey { backend: tail.backend, ..orbit_key_for(inp) };
            let bytes = orbit_blob::encode(s, key).ok_or("encode refused the short reference")?;
            let written = cache::offer(bytes).map_err(|e| format!("write failed: {e}"))?;
            if written.is_none() {
                return Err("the store declined the entry".into());
            }
            orbit_cache_lookup(inp, key).ok_or_else(|| "the lookup MISSED the entry just written".to_string())
        };
        let (pass, msg) = self.selfcheck_reuse_identity(device, queue, vp, size, short_iter, &via);
        if !pass {
            return (false, msg);
        }
        // Arm 2. The identity arm's extension was offered to the store on a background thread;
        // let it land so the unaided lookup sees the store as a later session would.
        cache::drain();
        let Some(inp) = self.export_reference_inputs_for(vp, false, IterBudget::current(self)) else {
            return (false, "could not rebuild the inputs for the unaided arm".into());
        };
        let t = Instant::now();
        let c = recompute_worker(inp);
        let unaided_ms = t.elapsed().as_secs_f64() * 1000.0;
        if !c.from_disk {
            return (
                false,
                format!("{msg}; but the worker did NOT consult the cache on its own (reused={})", c.reused),
            );
        }
        let u = cache::usage();
        (
            true,
            format!(
                "{msg}; unaided, the worker served len={} from disk in {unaided_ms:.0}ms ({} entries, {})",
                c.orbit_len,
                u.entries,
                cache::fmt_bytes(u.bytes)
            ),
        )
    }

    /// The shared body of the two reuse-identity checks: a fresh CONTROL build, a SHORT build,
    /// a reuse arm seeded from the short build by `via`, and a bit-for-bit comparison of the two
    /// renders — with the guards that keep the comparison from passing vacuously.
    fn selfcheck_reuse_identity(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        vp: &Viewport,
        size: u32,
        short_iter: u32,
        via: &dyn Fn(&RecomputeResult, &RecomputeInputs) -> Result<ReuseRef, String>,
    ) -> (bool, String) {
        use std::sync::atomic::{AtomicBool, AtomicU32};
        let Some(inp_a) = self.export_reference_inputs_for(vp, false, IterBudget::current(self)) else {
            return (false, "no reference for this view (direct path) — nothing to compare".into());
        };
        let a = recompute_worker(inp_a);
        // ⚠The control must be a genuine fresh pick, or "they match" says nothing.
        if a.reused {
            return (false, "the CONTROL arm reused a reference; it must be a fresh pick".into());
        }
        // ⚠⚠**The reuse arm must start SHORT, or the extension is never exercised.** Seeding it
        // from the full-length reference made `extend_reference_orbit` a no-op (measured: orbit
        // 813 → 813) and the check passed while proving nothing about extending. A live reference
        // is deliberately shorter than an export's ask — that asymmetry IS the case under test.
        let short = IterBudget { max_iter: short_iter, auto_iter: false };
        let Some(inp_s) = self.export_reference_inputs_for(vp, false, short) else {
            return (false, "could not build the short (live-like) reference inputs".into());
        };
        let sref = recompute_worker(inp_s);
        if sref.orbit_tail.as_ref().is_none_or(|t| t.escaped) {
            return (false, "the short orbit ESCAPED, so there is no tail to extend from".into());
        }
        let short_len = sref.orbit_len;
        let Some(mut inp_b) = self.export_reference_inputs_for(vp, false, IterBudget::current(self)) else {
            return (false, "could not rebuild the inputs for the reuse arm".into());
        };
        inp_b.reuse = match via(&sref, &inp_b) {
            Ok(r) => Some(r),
            Err(e) => return (false, e),
        };
        let b = recompute_worker(inp_b);
        // ⚠⚠**THE NON-VACUITY GUARD.** Reuse declines silently — wrong precision, drifted point,
        // no prefix — and falls back to a fresh build, in which case both arms are the same build
        // and agreeing proves nothing. This is the chunking lesson in a new costume: assert the
        // mechanism RAN before believing a null result.
        if !b.reused {
            return (false, "reuse DECLINED — the comparison would have been vacuous".into());
        }
        // ⚠And it must have GROWN: reuse that hands back the prefix untouched tests nothing.
        if b.orbit_len <= short_len {
            return (
                false,
                format!(
                    "reuse did not EXTEND (orbit {short_len} to {}) - short(iter={} len={} partial={}) fresh(iter={} len={} partial={})",
                    b.orbit_len, sref.iter, sref.orbit_len, sref.partial, a.iter, a.orbit_len, a.partial,
                ),
            );
        }
        // The measurement this whole exercise is for: what a GUI export would SAVE by extending
        // the live reference instead of picking and building a fresh one.
        let (fresh_ms, ext_ms) = (a.ref_ms, b.ref_ms);
        let mut req_a = self.build_export_request(vp, false, RefSource::ProbeOnly(Box::new(a)));
        let mut req_b = self.build_export_request(vp, false, RefSource::ProbeOnly(Box::new(b)));
        // ⚠`ProbeOnly` is taken unconditionally and leaves `req_max_iter` at `eff_iter`, so the two
        // requests differ in the REFERENCE and in nothing else — which is the whole point.
        for r in [&mut req_a, &mut req_b] {
            r.width = size;
            r.height = size;
            r.ss = 1;
        }
        let progress = AtomicU32::new(0);
        let cancel = AtomicBool::new(false);
        let ra = fractadyne_gpu::render_export(device, queue, &req_a, &progress, &cancel);
        let rb = fractadyne_gpu::render_export(device, queue, &req_b, &progress, &cancel);
        match (ra, rb) {
            (Ok(x), Ok(y)) if x.pixels.len() == y.pixels.len() => {
                let diffs = x
                    .pixels
                    .iter()
                    .zip(y.pixels.iter())
                    .filter(|(p, q)| p.to_bits() != q.to_bits())
                    .count();
                (
                    diffs == 0,
                    format!(
                        "extended {short_len} to {} (fresh {}); {diffs} of {} texels differ; reference {fresh_ms:.0}ms fresh vs {ext_ms:.0}ms extend",
                        req_b.orbit_len,
                        req_a.orbit_len,
                        x.pixels.len()
                    ),
                )
            }
            _ => (false, "render failed".into()),
        }
    }

    /// Render the CURRENT viewport at `n`×`n` for a determinism probe, reference and all.
    ///
    /// ⭐**Borrows the installed reference** (`RefSource::Live`). The caller builds the orbit once
    /// and installs it; at ~200,000 bits and two million iterations a per-render build would double
    /// the slowest check in the suite in order to test something it is not asking about.
    pub(crate) fn selfcheck_deep_render(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        n: u32,
    ) -> Option<Vec<f32>> {
        use std::sync::atomic::{AtomicBool, AtomicU32};
        let mut req = self.build_export_request(&self.viewport, self.julia_mode, RefSource::Live);
        req.width = n;
        req.height = n;
        req.ss = 1;
        let (sx, sy) = crate::contain_span(
            req.span_mantissa.x,
            req.span_mantissa.y,
            n as f64,
            n as f64,
        );
        req.span_mantissa = fractadyne_core::SpanMantissa::new(sx, sy);
        let progress = AtomicU32::new(0);
        let cancel = AtomicBool::new(false);
        fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel)
            .ok()
            .map(|r| r.pixels)
    }

    /// Does the same view render the same picture on a WIDER canvas?
    ///
    /// ⭐⭐**Why this is the check an "the export doesn't match my screen" report needs.**
    /// Magnification is HEIGHT-anchored (`upp = REFERENCE_HEIGHT / height_px × 2^-log2mag`), so two
    /// renders sharing a height share `upp` exactly, every pixel column maps to a fixed complex
    /// coordinate relative to the centre, and **the narrow image must appear pixel-for-pixel inside
    /// the wide one**. A centring error, an off-by-one in the raster mapping, or a mirrored axis all
    /// break that identity; a mere aspect change does not.
    ///
    /// ⚠⚠**The viewport is what varies, not just the request.** `delta_exp` comes from
    /// `vp.gpu_scale()`, which reads `vp.width_px` — so overriding `req.width` alone would hold the
    /// GPU scale fixed and test a narrower thing than `--render --size WxH` actually does.
    ///
    /// ⭐Returns `(differing_texels, compared_texels, note)` rather than a verdict: the caller
    /// decides the threshold, because the honest threshold DIFFERS BY DEPTH — see the two call
    /// sites in `selftest.rs`.
    pub(crate) fn selfcheck_width_independence(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        vp: &Viewport,
        height: u32,
        narrow: u32,
        wide: u32,
    ) -> (usize, usize, String) {
        use std::sync::atomic::{AtomicBool, AtomicU32};
        if narrow >= wide || (wide - narrow) % 2 != 0 {
            return (usize::MAX, 0, "widths must differ by an even amount".into());
        }
        let build = |w: u32| {
            let mut v = vp.clone();
            v.width_px = w as f64;
            v.height_px = height as f64;
            let mut r = self.build_export_request(&v, false, RefSource::Fresh);
            r.width = w;
            r.height = height;
            r.ss = 1;
            r
        };
        let (req_n, req_w) = (build(narrow), build(wide));
        let progress = AtomicU32::new(0);
        let cancel = AtomicBool::new(false);
        let rn = fractadyne_gpu::render_export(device, queue, &req_n, &progress, &cancel);
        let rw = fractadyne_gpu::render_export(device, queue, &req_w, &progress, &cancel);
        let (Ok(a), Ok(b)) = (rn, rw) else {
            return (usize::MAX, 0, "render failed".into());
        };
        if a.width != narrow || b.width != wide || a.height != height || b.height != height {
            return (
                usize::MAX,
                0,
                format!(
                    "size not honoured: got {}x{} and {}x{}",
                    a.width, a.height, b.width, b.height
                ),
            );
        }
        // Column `i` of the narrow image is column `i + off` of the wide one: both map to
        // `cx + (i - w/2 + 0.5) * upp` with their own `w`.
        let off = ((wide - narrow) / 2) as usize;
        let (nw, ww) = (narrow as usize, wide as usize);
        let mut diffs = 0usize;
        let mut worst = 0.0f32;
        let mut worst_at = (0usize, 0usize);
        for y in 0..height as usize {
            for x in 0..nw {
                let ia = (y * nw + x) * 4;
                let ib = (y * ww + x + off) * 4;
                let mut d = 0.0f32;
                for c in 0..3 {
                    d = d.max((a.pixels[ia + c] - b.pixels[ib + c]).abs());
                }
                if d > 0.0 {
                    diffs += 1;
                    if d > worst {
                        worst = d;
                        worst_at = (y, x);
                    }
                }
            }
        }
        let compared = height as usize * nw;
        let note = if diffs == 0 {
            format!("{narrow}px inside {wide}px at h={height}: identical over {compared} texels")
        } else {
            format!(
                "{diffs} of {compared} texels differ (worst {:.3} at row {} col {})",
                worst, worst_at.0, worst_at.1
            )
        };
        (diffs, compared, note)
    }
}
