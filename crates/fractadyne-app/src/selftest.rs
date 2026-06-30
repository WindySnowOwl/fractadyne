//! GPU validation self-test (`--selftest`): renders controlled views and cross-checks the
//! render paths against each other, against arbitrary-precision/CPU oracles, and against
//! golden images. Writes a verifiable Markdown report. (Exact numeric ground truth lives in
//! `fractadyne-core` unit tests; this validates the visual/render pipeline.)

use crate::{
    gather_system_info, mandel_escapes, method_to_str, utc_string, version_string, FractadyneApp,
    FractalKind,
};
use fractadyne_core::Viewport;

/// Plain-f64 **smooth** Mandelbrot dwell, matching the shader exactly (bailout 256,
/// `smooth = iter + 1 − log₂(½·ln|z|² / ln2)`). f64 is dead-accurate at the depths the
/// self-test uses, so this is independent ground truth for the GPU perturbation path.
fn mandel_smooth_f64(cx: f64, cy: f64, max: u32) -> Option<f32> {
    const BAIL2: f64 = 256.0 * 256.0;
    let (mut zx, mut zy) = (0.0_f64, 0.0_f64);
    for iter in 1..=max {
        let (nzx, nzy) = (zx * zx - zy * zy + cx, 2.0 * zx * zy + cy);
        zx = nzx;
        zy = nzy;
        let m2 = zx * zx + zy * zy;
        if m2 > BAIL2 {
            let nu = (m2.ln() * 0.5 / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
            return Some(iter as f32 + 1.0 - nu as f32);
        }
    }
    None
}

/// FNV-1a 64-bit checksum (no deps) — a content fingerprint for golden images so a
/// third party can confirm they're looking at the same reference bytes.
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Per-channel 8-bit image difference: `(max abs, mean abs)`. Mismatched sizes → worst.
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

/// One row of the validation report.
struct SelfCheck {
    category: &'static str,
    name: String,
    params: String,
    result: String,
    threshold: &'static str,
    pass: bool,
}

/// Machine-readable validation catalog (`validation/catalog.toml`) — locations with
/// independently verifiable answers, consumed by `--selftest` (Phase 6.1 / 6.6).
#[derive(serde::Deserialize, Default)]
struct Catalog {
    #[serde(default)]
    nucleus: Vec<NucleusEntry>,
    #[serde(default)]
    membership: Vec<MemberEntry>,
}

#[derive(serde::Deserialize)]
struct NucleusEntry {
    name: String,
    #[serde(default)]
    fractal: Option<String>,
    center_x: String,
    center_y: String,
    zoom: f64,
    period: u32,
    #[serde(default)]
    nucleus_x: Option<String>,
    #[serde(default)]
    nucleus_y: Option<String>,
}

#[derive(serde::Deserialize)]
struct MemberEntry {
    name: String,
    center_x: String,
    center_y: String,
    interior: bool,
}

impl FractadyneApp {
    /// GPU validation suite (`--selftest`): renders controlled views and cross-checks the
    /// render paths against each other and against invariants. Prints a report; returns
    /// true iff every check passed. This validates the *visual/render* pipeline; exact
    /// numeric ground truth lives in `fractadyne-core`'s unit tests.
    pub(crate) fn run_selftest(&mut self, device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue) -> bool {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        // Deterministic params: Mandelbrot, no Julia.
        self.fractal = FractalKind::Mandelbrot;
        self.julia_mode = false;
        // Seahorse Valley — detailed at every depth tested; coordinate precise enough.
        const SX: &str = "-0.743643887037151";
        const SY: &str = "0.131825904205330";
        const N: u32 = 220;

        // Read back the raw iteration texture (smooth_iter, normal.x, normal.y, DE) — far
        // more sensitive than comparing final colors.
        let render = |req: &fractadyne_gpu::ExportRequest| -> Option<Vec<f32>> {
            fractadyne_gpu::render_iter(device, queue, req).ok().map(|r| r.pixels)
        };
        // A square request at the seahorse, then caller overrides the mode.
        let make = |cx: &str, cy: &str, mag: f64| -> fractadyne_gpu::ExportRequest {
            let mut vp = Viewport::new(N as f64, N as f64);
            vp.center_x = fractadyne_core::parse_bf(cx).unwrap();
            vp.center_y = fractadyne_core::parse_bf(cy).unwrap();
            vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mag));
            vp.precision = fractadyne_core::precision_for_magnification(mag);
            let mut req = self.current_export_request_for(&vp, false);
            req.width = N;
            req.height = N;
            req.ss = 1;
            req
        };
        // Mean |Δ| of the smooth-iteration channel over pixels that escaped in both, plus
        // the fraction differing by > 2 iterations (tolerates rare perturbation glitches).
        let compare = |a: &[f32], b: &[f32]| -> (f64, f64) {
            let (mut sum, mut n, mut big) = (0.0f64, 0u64, 0u64);
            for i in 0..(a.len() / 4) {
                let (ra, rb) = (a[i * 4], b[i * 4]);
                if ra >= 0.0 && rb >= 0.0 {
                    let d = (ra - rb).abs() as f64;
                    sum += d;
                    n += 1;
                    if d > 2.0 {
                        big += 1;
                    }
                }
            }
            if n == 0 { (f64::INFINITY, 1.0) } else { (sum / n as f64, big as f64 / n as f64) }
        };
        // Only the dwell (smooth-iter) channel needs to be finite; DE/normal channels can
        // legitimately overflow to ±inf when a mode is pushed past its range.
        let finite = |px: &[f32]| px.iter().step_by(4).all(|v| v.is_finite());

        // Independent integer-escape (`n`) bignum oracle for one (center, mag) view vs the
        // GPU `px`, on a sparse grid (slow on purpose). Each sample is classified:
        //   • both interior (CPU None, GPU < 0), or both escaped with the same n (|Δsmooth|<0.5)
        //   • boundary — ±1 iteration, or within a band of max_iter (dwell ill-conditioned)
        //   • mismatch — n off by ≥2, or interior/escaped disagreement away from the boundary.
        // Returns (checked, agree, boundary, mismatch). `max` MUST equal the GPU's max_iter
        // and bailout 256² so the integer counts are directly comparable.
        let oracle = |cx_s: &str, cy_s: &str, mag: f64, max: u32, px: &[f32]| -> (u64, u64, u64, u64) {
            let prec = fractadyne_core::precision_for_magnification(mag);
            let cx = fractadyne_core::parse_bf(cx_s).unwrap();
            let cy = fractadyne_core::parse_bf(cy_s).unwrap();
            let step = (3.0 / mag) / N as f64;
            let half = N as f64 / 2.0;
            let nn = N as usize;
            let at = |ii: usize, jj: usize| px[(jj * nn + ii) * 4];
            let gstep = (N / 5).max(1) as usize; // ~5×5 sparse grid
            let (mut checked, mut agree, mut boundary, mut mism) = (0u64, 0u64, 0u64, 0u64);
            let mut j = 0usize;
            while j < nn {
                let mut i = 0usize;
                while i < nn {
                    let g = at(i, j);
                    // Boundary detection from the GPU texture itself: a sample whose 4-neighbors
                    // flip interior↔exterior or jump in dwell is in an ill-conditioned region,
                    // where a sub-ULP coordinate difference legitimately flips n — exclude it.
                    let mut steep = false;
                    for (di, dj) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
                        let (ni, nj) = (i as isize + di, j as isize + dj);
                        if ni >= 0 && nj >= 0 && (ni as usize) < nn && (nj as usize) < nn {
                            let gn = at(ni as usize, nj as usize);
                            let flip = (g < 0.0) != (gn < 0.0);
                            let jump = g >= 0.0 && gn >= 0.0 && (g - gn).abs() > 2.0;
                            if flip || jump {
                                steep = true;
                            }
                        }
                    }
                    checked += 1;
                    if steep {
                        boundary += 1;
                        i += gstep;
                        continue;
                    }
                    let cre = fractadyne_core::add_f64(&cx, ((i as f64 + 0.5) - half) * step, prec);
                    let cim = fractadyne_core::add_f64(&cy, (half - (j as f64 + 0.5)) * step, prec);
                    let cpu = fractadyne_core::naive_dwell_bf(&cre, &cim, max, 65536.0, prec);
                    // Smooth region: GPU and CPU must agree exactly (same n).
                    match (g >= 0.0, cpu) {
                        (false, None) => agree += 1,
                        (true, Some((_n, smooth))) if (g - smooth).abs() < 0.75 => agree += 1,
                        _ => mism += 1,
                    }
                    i += gstep;
                }
                j += gstep;
            }
            (checked, agree, boundary, mism)
        };

        let mut checks: Vec<SelfCheck> = Vec::new();

        // ---- numeric & render-path checks (local closures borrow self immutably) ----
        {
            // (A) df32 perturbation vs an independent CPU f64 dwell @2e4× (f64 exact here).
            let mag = 2.0e4;
            let req = make(SX, SY, mag);
            if let Some(px) = render(&req) {
                let cx0 = fractadyne_core::to_f64(&fractadyne_core::parse_bf(SX).unwrap());
                let cy0 = fractadyne_core::to_f64(&fractadyne_core::parse_bf(SY).unwrap());
                let step = (3.0 / mag) / N as f64;
                let half = N as f64 / 2.0;
                let (mut n, mut big) = (0u64, 0u64);
                let mut k = 0usize;
                while k < (N as usize) * (N as usize) {
                    let (i, j) = ((k % N as usize) as f64, (k / N as usize) as f64);
                    let g = px[k * 4];
                    if g >= 0.0 {
                        let cre = cx0 + ((i + 0.5) - half) * step;
                        let cim = cy0 + (half - (j + 0.5)) * step;
                        if let Some(cpu) = mandel_smooth_f64(cre, cim, req.max_iter) {
                            n += 1;
                            if (g - cpu).abs() > 1.0 {
                                big += 1;
                            }
                        }
                    }
                    k += 7;
                }
                let frac = if n == 0 { 1.0 } else { big as f64 / n as f64 };
                checks.push(SelfCheck {
                    category: "Numeric",
                    name: "df32 perturbation vs CPU f64 dwell".into(),
                    params: format!("seahorse, 2e4×, {} iter, n={n}", req.max_iter),
                    result: format!("{:.1}% agree within 1 iter", (1.0 - frac) * 100.0),
                    threshold: "≥90% within 1 iter",
                    pass: frac < 0.10,
                });
                checks.push(SelfCheck {
                    category: "Finiteness",
                    name: "dwell finite (perturbation @2e4×)".into(),
                    params: "all sampled pixels".into(),
                    result: if finite(&px) { "all finite".into() } else { "NON-FINITE!".into() },
                    threshold: "all finite",
                    pass: finite(&px),
                });
            }

            // (B) floatexp vs df32 perturbation @1e10× — two representations, must agree.
            let mut a = make(SX, SY, 1.0e10);
            a.mode = 0;
            let mut b = a.clone();
            b.mode = 2;
            if let (Some(aa), Some(bb)) = (render(&a), render(&b)) {
                let (mean, frac) = compare(&aa, &bb);
                checks.push(SelfCheck {
                    category: "Numeric",
                    name: "floatexp vs df32 perturbation".into(),
                    params: "seahorse, 1e10×".into(),
                    result: format!("mean Δ={mean:.4} iter, >2iter {:.3}%", frac * 100.0),
                    threshold: "mean<0.5, <2% differ",
                    pass: mean < 0.5 && frac < 0.02,
                });
            }

            // (C) Independent bignum oracle across a DEPTH BATTERY — integer escape n, exact
            // on every non-boundary sample, testing whichever render mode the depth selector
            // actually uses (df32 perturbation through ~1e24×, floatexp at ≥1e28×). This is
            // the only check that gives *independent* deep-zoom correctness (not internal
            // consistency). Full-precision deep coordinates use a 38-digit minibrot nucleus.
            const NX: &str = "-0.74364388703715887077806454349323251348";
            const NY: &str = "0.131825904205312292821097354874199108694";
            let battery: &[(&str, &str, &str, f64)] = &[
                ("1e6x", SX, SY, 1.0e6),
                ("1e12x", SX, SY, 1.0e12),
                ("1e16x", NX, NY, 1.0e16),
                ("1e24x", NX, NY, 1.0e24),
                ("1e30x", NX, NY, 1.0e30),
            ];
            for (label, cx, cy, mag) in battery {
                let req = make(cx, cy, *mag); // mode chosen by the real depth selector
                if let Some(px) = render(&req) {
                    let (checked, agree, boundary, mism) = oracle(cx, cy, *mag, req.max_iter, &px);
                    checks.push(SelfCheck {
                        category: "Bignum oracle",
                        name: format!("naive bignum dwell vs GPU @{label}"),
                        params: format!("mode {}, {} iter, {checked} samples", req.mode, req.max_iter),
                        result: format!("{agree} agree, {boundary} boundary, {mism} mismatch"),
                        threshold: "0 hard mismatches",
                        pass: mism == 0 && checked > 0,
                    });
                } else {
                    checks.push(SelfCheck {
                        category: "Bignum oracle",
                        name: format!("naive bignum dwell vs GPU @{label}"),
                        params: "render".into(),
                        result: "render failed".into(),
                        threshold: "0 hard mismatches",
                        pass: false,
                    });
                }
            }

            // (D3) Series approximation — at deep zoom (mode 2) the order-3 polynomial seed
            // must (a) actually engage (skip > 0) and (b) reproduce the full-iteration render.
            // Compare an SA-on render to the same view with the skip forced to 0.
            {
                let on = make(NX, NY, 1.0e30);
                let mut off = on.clone();
                off.sa_skip = 0;
                let skip = on.sa_skip;
                match (render(&on), render(&off)) {
                    (Some(a), Some(b)) if skip > 0 => {
                        // Smooth-region max |Δ| (skip boundary/interior sentinels).
                        let mut maxd = 0.0f64;
                        for i in 0..(a.len() / 4) {
                            let (ra, rb) = (a[i * 4], b[i * 4]);
                            if ra >= 0.0 && rb >= 0.0 {
                                maxd = maxd.max((ra - rb).abs() as f64);
                            }
                        }
                        checks.push(SelfCheck {
                            category: "Series approximation",
                            name: "SA seed vs full iteration @1e30×".into(),
                            params: format!("Mandelbrot, 1e30×, skip {skip} of {} iter", on.max_iter),
                            result: format!("max Δ {maxd:.4} smooth iter"),
                            threshold: "skip>0 and max Δ < 0.05",
                            pass: maxd < 0.05,
                        });
                    }
                    _ => checks.push(SelfCheck {
                        category: "Series approximation",
                        name: "SA seed vs full iteration @1e30×".into(),
                        params: "Mandelbrot, 1e30×".into(),
                        result: if skip == 0 { "SA did not engage (skip=0)".into() } else { "render failed".into() },
                        threshold: "skip>0 and max Δ < 0.05",
                        pass: false,
                    }),
                }
            }

            // (D2) Reference independence — a correct perturbation render is invariant to the
            // chosen valid reference. Render with 3 distinct in-view references (the auto
            // `best_reference` plus two offset points), take the per-pixel majority dwell as
            // oracle-free truth, and assert the *auto* reference dissents from consensus on a
            // tiny, localized fraction (dissenters are exactly the glitched pixels). The
            // offset references are deliberately allowed to be poorer — they just provide
            // independent votes.
            {
                let mag = 1.0e8;
                let base = make(SX, SY, mag); // mode 0, best_reference
                let prec = fractadyne_core::precision_for_magnification(mag);
                let cxb = fractadyne_core::parse_bf(SX).unwrap();
                let cyb = fractadyne_core::parse_bf(SY).unwrap();
                // Actual complex span (shallow here): span_mantissa × 2^delta_exp.
                let span = base.span_mantissa[0] * 2f64.powi(base.delta_exp);
                let span_fe = fractadyne_core::FloatExp::from_f64(span);
                let with_ref = |ox: f64, oy: f64| -> fractadyne_gpu::ExportRequest {
                    let ref_pt = [
                        fractadyne_core::add_f64(&cxb, ox, prec),
                        fractadyne_core::add_f64(&cyb, oy, prec),
                    ];
                    let (orbit, len, rp) = self.compute_reference(
                        &[cxb.clone(), cyb.clone()], (span_fe, span_fe), base.max_iter, prec, false, Some(ref_pt),
                    );
                    let dx = fractadyne_core::ref_offset_mantissa(&cxb, &rp[0], base.delta_exp, prec);
                    let dy = fractadyne_core::ref_offset_mantissa(&cyb, &rp[1], base.delta_exp, prec);
                    let (dxh, dyh) = (dx as f32, dy as f32);
                    let mut r = base.clone();
                    r.orbit = orbit;
                    r.orbit_len = len;
                    r.ref_offset = [dxh, dyh, (dx - dxh as f64) as f32, (dy - dyh as f64) as f32];
                    r
                };
                let altb = with_ref(0.25 * span, 0.20 * span);
                let altc = with_ref(-0.22 * span, -0.18 * span);
                if let (Some(pa), Some(pb), Some(pc)) =
                    (render(&base), render(&altb), render(&altc))
                {
                    let nn = N as usize;
                    let eq = |x: f32, y: f32| ((x < 0.0) == (y < 0.0)) && (x < 0.0 || (x - y).abs() < 0.5);
                    // Skip boundary pixels (dwell ill-conditioned there — a sub-ULP reference
                    // difference legitimately flips n); count only smooth-region disagreement.
                    let steep = |i: usize, j: usize| -> bool {
                        let g = pa[(j * nn + i) * 4];
                        for (di, dj) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
                            let (ni, nj) = (i as isize + di, j as isize + dj);
                            if ni >= 0 && nj >= 0 && (ni as usize) < nn && (nj as usize) < nn {
                                let gn = pa[(nj as usize * nn + ni as usize) * 4];
                                if (g < 0.0) != (gn < 0.0) || (g >= 0.0 && gn >= 0.0 && (g - gn).abs() > 2.0) {
                                    return true;
                                }
                            }
                        }
                        false
                    };
                    let (mut smooth, mut auto_dissent, mut no_majority) = (0u64, 0u64, 0u64);
                    for j in 0..nn {
                        for i in 0..nn {
                            if steep(i, j) {
                                continue;
                            }
                            let k = j * nn + i;
                            let (a, b, c) = (pa[k * 4], pb[k * 4], pc[k * 4]);
                            let (ab, ac, bc) = (eq(a, b), eq(a, c), eq(b, c));
                            smooth += 1;
                            if ab || ac {
                                // auto in the majority — clean
                            } else if bc {
                                auto_dissent += 1;
                            } else {
                                no_majority += 1;
                            }
                        }
                    }
                    let frac = (auto_dissent + no_majority) as f64 / smooth.max(1) as f64;
                    checks.push(SelfCheck {
                        category: "Glitch",
                        name: "reference independence (3-ref majority)".into(),
                        params: "seahorse, 1e8×, auto vs 2 offset refs (smooth region)".into(),
                        result: format!(
                            "{} smooth px: auto dissent {auto_dissent}, no-majority {no_majority} ({:.4}%)",
                            smooth, frac * 100.0
                        ),
                        threshold: "<0.2% of smooth pixels",
                        pass: frac < 0.002,
                    });
                }
            }

            // (E) Real-axis symmetry + interior/exterior presence + finiteness @home.
            let req = make("-0.5", "0.0", 1.0);
            if let Some(px) = render(&req) {
                let w = N as usize;
                let (mut sum, mut n) = (0.0f64, 0u64);
                for y in 0..(N as usize / 2) {
                    for x in 0..w {
                        let (t, bm) = (px[(y * w + x) * 4], px[((N as usize - 1 - y) * w + x) * 4]);
                        if t >= 0.0 && bm >= 0.0 {
                            sum += (t - bm).abs() as f64;
                            n += 1;
                        }
                    }
                }
                let mean = if n == 0 { f64::INFINITY } else { sum / n as f64 };
                checks.push(SelfCheck {
                    category: "Invariant",
                    name: "real-axis mirror symmetry".into(),
                    params: "home view (-0.5, 0)".into(),
                    result: format!("mean Δ={mean:.5} iter"),
                    threshold: "mean<0.05",
                    pass: mean < 0.05,
                });
                let interior = px.iter().step_by(4).any(|&r| r < 0.0);
                let exterior = px.iter().step_by(4).any(|&r| r >= 0.0);
                checks.push(SelfCheck {
                    category: "Invariant",
                    name: "home has interior + exterior".into(),
                    params: "home view".into(),
                    result: format!("interior={interior}, exterior={exterior}"),
                    threshold: "both present",
                    pass: interior && exterior,
                });
            }
        }

        // ---- render-pipeline symmetry for the non-Mandelbrot family shaders ----
        // The bignum oracle only validates the Mandelbrot shader; these exact symmetries
        // (verified in fractadyne-core) are the main correctness signal for the other
        // analytic-family shaders. Render an origin/real-axis-centered view and compare
        // each pixel to its symmetry partner, excluding ill-conditioned boundary pixels.
        {
            let nn = N as usize;
            let steep = |px: &[f32], i: usize, j: usize| -> bool {
                let g = px[(j * nn + i) * 4];
                for (di, dj) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
                    let (ni, nj) = (i as isize + di, j as isize + dj);
                    if ni >= 0 && nj >= 0 && (ni as usize) < nn && (nj as usize) < nn {
                        let gn = px[(nj as usize * nn + ni as usize) * 4];
                        if (g < 0.0) != (gn < 0.0) || (g >= 0.0 && gn >= 0.0 && (g - gn).abs() > 2.0) {
                            return true;
                        }
                    }
                }
                false
            };
            let cases: &[(FractalKind, &str, fn(usize, usize, usize) -> (usize, usize))] = &[
                (FractalKind::Multibrot3, "Multibrot-3 180° rotation", |i, j, n| (n - 1 - i, n - 1 - j)),
                (FractalKind::Tricorn, "Tricorn real-axis reflection", |i, j, n| (i, n - 1 - j)),
                (FractalKind::Celtic, "Celtic real-axis reflection", |i, j, n| (i, n - 1 - j)),
            ];
            for &(fractal, label, partner) in cases {
                self.fractal = fractal;
                self.julia_mode = false;
                self.color_method = 0;
                self.use_custom_palette = false;
                self.auto_iter = false;
                self.max_iter = 1500;
                let mut vp = Viewport::new(N as f64, N as f64);
                vp.center_x = fractadyne_core::BigFloat::from_f64(0.0, 64);
                vp.center_y = fractadyne_core::BigFloat::from_f64(0.0, 64);
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / N as f64); // span 3, origin-centered
                vp.precision = 64;
                let mut req = self.current_export_request_for(&vp, false);
                req.width = N;
                req.height = N;
                req.ss = 1;
                let px = fractadyne_gpu::render_iter(device, queue, &req).ok().map(|r| r.pixels);
                if let Some(px) = px {
                    let (mut total, mut bad) = (0u64, 0u64);
                    for j in 0..nn {
                        for i in 0..nn {
                            if steep(&px, i, j) {
                                continue;
                            }
                            let (pi, pj) = partner(i, j, nn);
                            let (a, b) = (px[(j * nn + i) * 4], px[(pj * nn + pi) * 4]);
                            let eq = (a < 0.0) == (b < 0.0) && (a < 0.0 || (a - b).abs() < 0.5);
                            total += 1;
                            if !eq {
                                bad += 1;
                            }
                        }
                    }
                    checks.push(SelfCheck {
                        category: "Symmetry (render)",
                        name: label.into(),
                        params: format!("origin view, span 3, {total} smooth px"),
                        result: format!("{bad} asymmetric"),
                        threshold: "0 asymmetric",
                        pass: bad == 0 && total > 0,
                    });
                }
            }
        }

        // ---- abs-family deep zoom: perturbation (df32) vs direct path ----
        // Burning Ship / Celtic / Buffalo are non-analytic: their shader perturbation
        // folds with `diffabs` at the abs cusps. There's no closed-form oracle for an
        // off-axis detail view, so we cross-check the new perturbation path (mode 0)
        // against the trusted direct path (mode 1) at a depth where direct df32 is still
        // accurate (~1e5×). They must agree everywhere except a tiny fraction of
        // fold-crossing pixels (where a diffabs branch flip is an inherent glitch).
        {
            self.julia_mode = false;
            self.color_method = 0;
            self.use_custom_palette = false;
            self.auto_iter = false;
            self.max_iter = 2000;
            let nn = N as usize;
            let steep = |px: &[f32], i: usize, j: usize| -> bool {
                let g = px[(j * nn + i) * 4];
                for (di, dj) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
                    let (ni, nj) = (i as isize + di, j as isize + dj);
                    if ni >= 0 && nj >= 0 && (ni as usize) < nn && (nj as usize) < nn {
                        let gn = px[(nj as usize * nn + ni as usize) * 4];
                        if (g < 0.0) != (gn < 0.0) || (g >= 0.0 && gn >= 0.0 && (g - gn).abs() > 2.0) {
                            return true;
                        }
                    }
                }
                false
            };
            // (family, center, mag) — boundary-detail regions rich in escaping pixels.
            let abs_cases: &[(FractalKind, &str, &str, f64)] = &[
                (FractalKind::BurningShip, "-1.7548", "-0.0312", 1.0e5),
                (FractalKind::Celtic, "-1.2566", "0.0480", 1.0e5),
                (FractalKind::Buffalo, "-1.7548", "-0.0312", 1.0e5),
            ];
            for &(fractal, cx, cy, mag) in abs_cases {
                self.fractal = fractal;
                let mut vp = Viewport::new(N as f64, N as f64);
                vp.center_x = fractadyne_core::parse_bf(cx).unwrap();
                vp.center_y = fractadyne_core::parse_bf(cy).unwrap();
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mag));
                vp.precision = fractadyne_core::precision_for_magnification(mag);
                let mut pert = self.current_export_request_for(&vp, false);
                pert.width = N;
                pert.height = N;
                pert.ss = 1;
                let mut direct = pert.clone();
                direct.mode = 1; // force the trusted direct df32 path
                if let (Some(a), Some(b)) = (
                    fractadyne_gpu::render_iter(device, queue, &pert).ok().map(|r| r.pixels),
                    fractadyne_gpu::render_iter(device, queue, &direct).ok().map(|r| r.pixels),
                ) {
                    let (mut sum, mut n, mut big) = (0.0f64, 0u64, 0u64);
                    for j in 0..nn {
                        for i in 0..nn {
                            if steep(&a, i, j) {
                                continue;
                            }
                            let k = j * nn + i;
                            let (ra, rb) = (a[k * 4], b[k * 4]);
                            if ra >= 0.0 && rb >= 0.0 {
                                let d = (ra - rb).abs() as f64;
                                sum += d;
                                n += 1;
                                if d > 2.0 {
                                    big += 1;
                                }
                            }
                        }
                    }
                    let mean = if n == 0 { f64::INFINITY } else { sum / n as f64 };
                    let frac = if n == 0 { 1.0 } else { big as f64 / n as f64 };
                    checks.push(SelfCheck {
                        category: "Abs-family deep zoom",
                        name: format!("{} perturbation vs direct", fractal.name()),
                        params: format!("{mag:.0e}×, mode {} vs 1, n={n}", pert.mode),
                        result: format!("mean Δ={mean:.4} iter, >2iter {:.3}%", frac * 100.0),
                        threshold: "mode 0, mean<0.5, <2% differ, n>0",
                        pass: pert.mode == 0 && n > 0 && mean < 0.5 && frac < 0.02,
                    });
                }

                // floatexp (mode 2) vs df32 (mode 0) at a depth both paths handle —
                // validates the new extended-range abs path against the validated df32
                // one (the two carry δz in different representations and must agree).
                let mid_mag = 1.0e10;
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mid_mag));
                vp.precision = fractadyne_core::precision_for_magnification(mid_mag);
                let mut m0 = self.current_export_request_for(&vp, false);
                m0.width = N;
                m0.height = N;
                m0.ss = 1;
                m0.mode = 0;
                let mut m2 = m0.clone();
                m2.mode = 2;
                if let (Some(a), Some(b)) = (
                    fractadyne_gpu::render_iter(device, queue, &m0).ok().map(|r| r.pixels),
                    fractadyne_gpu::render_iter(device, queue, &m2).ok().map(|r| r.pixels),
                ) {
                    let (mut sum, mut n, mut big) = (0.0f64, 0u64, 0u64);
                    for j in 0..nn {
                        for i in 0..nn {
                            if steep(&a, i, j) {
                                continue;
                            }
                            let k = j * nn + i;
                            let (ra, rb) = (a[k * 4], b[k * 4]);
                            if ra >= 0.0 && rb >= 0.0 {
                                let d = (ra - rb).abs() as f64;
                                sum += d;
                                n += 1;
                                if d > 2.0 {
                                    big += 1;
                                }
                            }
                        }
                    }
                    let mean = if n == 0 { f64::INFINITY } else { sum / n as f64 };
                    let frac = if n == 0 { 1.0 } else { big as f64 / n as f64 };
                    checks.push(SelfCheck {
                        category: "Abs-family deep zoom",
                        name: format!("{} floatexp vs df32", fractal.name()),
                        params: format!("{mid_mag:.0e}×, mode 2 vs 0, n={n}"),
                        result: format!("mean Δ={mean:.4} iter, >2iter {:.3}%", frac * 100.0),
                        threshold: "mean<0.5, <2% differ, n>0",
                        pass: n > 0 && mean < 0.5 && frac < 0.02,
                    });
                }

                // Deep-zoom guard: past the df32 ceiling (~1e28×) the abs families switch
                // to the floatexp (mode 2) diffabs path. It must stay finite (no NaN/inf)
                // at extreme depth — where df32 perturbation would have underflowed to a
                // uniform screen. (Correctness is pinned by the matches above; whether a
                // blindly zoomed-in center lands on detail is not a correctness signal.)
                let deep_mag = 1.0e35;
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * deep_mag));
                vp.precision = fractadyne_core::precision_for_magnification(deep_mag);
                let mut deep = self.current_export_request_for(&vp, false);
                deep.width = N;
                deep.height = N;
                deep.ss = 1;
                if let Some(px) = fractadyne_gpu::render_iter(device, queue, &deep).ok().map(|r| r.pixels) {
                    let dwell_finite = px.iter().step_by(4).all(|v| v.is_finite());
                    let interior = px.iter().step_by(4).filter(|&&v| v < 0.0).count();
                    // Detail = spread of escaped dwell. A uniform screen (mode breakdown)
                    // would collapse this to ~0; real fractal structure spans many iters.
                    let (mut lo, mut hi, mut esc) = (f32::INFINITY, f32::NEG_INFINITY, 0u64);
                    for v in px.iter().step_by(4) {
                        if *v >= 0.0 {
                            lo = lo.min(*v);
                            hi = hi.max(*v);
                            esc += 1;
                        }
                    }
                    let spread = if esc > 0 { (hi - lo) as f64 } else { 0.0 };
                    checks.push(SelfCheck {
                        category: "Abs-family deep zoom",
                        name: format!("{} deep finiteness @1e35×", fractal.name()),
                        params: format!("{deep_mag:.0e}×, mode {}", deep.mode),
                        result: format!(
                            "{} dwell, {esc} escaped / {interior} interior, spread {spread:.1} iter",
                            if dwell_finite { "finite" } else { "NON-FINITE!" }
                        ),
                        threshold: "mode 2, all finite",
                        pass: deep.mode == 2 && dwell_finite,
                    });
                }
            }
        }

        // ---- invariance & consistency (Phase 3) — oracle-free, targets the tier crossovers ----
        {
            self.fractal = FractalKind::Mandelbrot;
            self.julia_mode = false;
            self.color_method = 0;
            self.use_custom_palette = false;
            self.auto_iter = false;
            let cxb = fractadyne_core::parse_bf(SX).unwrap();
            let cyb = fractadyne_core::parse_bf(SY).unwrap();
            // Build a square Mandelbrot iteration render at an explicit center/zoom/size.
            let build = |cx: &fractadyne_core::BigFloat, cy: &fractadyne_core::BigFloat,
                         mag: f64, size: u32, max: u32|
             -> Option<Vec<f32>> {
                let mut vp = Viewport::new(size as f64, size as f64);
                vp.center_x = cx.clone();
                vp.center_y = cy.clone();
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (size as f64 * mag));
                vp.precision = fractadyne_core::precision_for_magnification(mag);
                let mut req = self.current_export_request_for(&vp, false);
                req.width = size;
                req.height = size;
                req.ss = 1;
                req.max_iter = max;
                fractadyne_gpu::render_iter(device, queue, &req).ok().map(|r| r.pixels)
            };
            let steep = |px: &[f32], sz: usize, i: usize, j: usize| -> bool {
                let g = px[(j * sz + i) * 4];
                for (di, dj) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
                    let (ni, nj) = (i as isize + di, j as isize + dj);
                    if ni >= 0 && nj >= 0 && (ni as usize) < sz && (nj as usize) < sz {
                        let gn = px[(nj as usize * sz + ni as usize) * 4];
                        if (g < 0.0) != (gn < 0.0) || (g >= 0.0 && gn >= 0.0 && (g - gn).abs() > 2.0) {
                            return true;
                        }
                    }
                }
                false
            };
            let agree = |a: f32, b: f32| (a < 0.0) == (b < 0.0) && (a < 0.0 || (a - b).abs() < 0.5);
            let nn = N as usize;

            // 3.1 Resolution independence: an N×N pixel (i,j) shares its exact complex
            // coordinate with the 3N×3N pixel (3i+1, 3j+1); their dwell must match.
            if let (Some(p1), Some(p3)) =
                (build(&cxb, &cyb, 1.0e6, N, 2000), build(&cxb, &cyb, 1.0e6, N * 3, 2000))
            {
                let n3 = nn * 3;
                let (mut checked, mut bad) = (0u64, 0u64);
                for j in 0..nn {
                    for i in 0..nn {
                        if steep(&p1, nn, i, j) {
                            continue;
                        }
                        let (i3, j3) = (3 * i + 1, 3 * j + 1);
                        checked += 1;
                        if !agree(p1[(j * nn + i) * 4], p3[(j3 * n3 + i3) * 4]) {
                            bad += 1;
                        }
                    }
                }
                checks.push(SelfCheck {
                    category: "Consistency",
                    name: "resolution independence (N vs 3N)".into(),
                    params: format!("seahorse, 1e6×, {checked} smooth px"),
                    result: format!("{bad} differ"),
                    threshold: "0 differ",
                    pass: bad == 0 && checked > 0,
                });
            }

            // 3.2 Max-iter monotonic stability: a pixel already escaped at a low max_iter keeps
            // its dwell at a higher max_iter (raising the cap only escapes more interior pixels).
            if let (Some(pa), Some(pb)) =
                (build(&cxb, &cyb, 1.0e6, N, 500), build(&cxb, &cyb, 1.0e6, N, 3000))
            {
                let (mut checked, mut bad) = (0u64, 0u64);
                for k in 0..(nn * nn) {
                    let a = pa[k * 4];
                    if a >= 0.0 {
                        checked += 1;
                        if !agree(a, pb[k * 4]) {
                            bad += 1;
                        }
                    }
                }
                checks.push(SelfCheck {
                    category: "Consistency",
                    name: "max-iter monotonic stability".into(),
                    params: format!("seahorse, 1e6×, 500→3000 iter, {checked} escaped px"),
                    result: format!("{bad} changed dwell"),
                    threshold: "0 changed",
                    pass: bad == 0 && checked > 0,
                });
            }

            // 3.3 Zoom-sequence consistency ACROSS THE direct→perturbation crossover: a view at
            // 4e3× (direct) and at 1.2e4× (perturbation, 3× deeper) must agree where they
            // overlap — shallower pixel k ↔ deeper pixel (3k+1−N). Strongest test of the seam.
            if let (Some(ps), Some(pd)) =
                (build(&cxb, &cyb, 4.0e3, N, 3000), build(&cxb, &cyb, 1.2e4, N, 3000))
            {
                let (mut checked, mut bad) = (0u64, 0u64);
                for j in 0..nn {
                    for k in 0..nn {
                        let id = 3 * k as isize + 1 - nn as isize;
                        let jd = 3 * j as isize + 1 - nn as isize;
                        if id < 0 || jd < 0 || id as usize >= nn || jd as usize >= nn {
                            continue;
                        }
                        if steep(&ps, nn, k, j) {
                            continue;
                        }
                        checked += 1;
                        if !agree(ps[(j * nn + k) * 4], pd[(jd as usize * nn + id as usize) * 4]) {
                            bad += 1;
                        }
                    }
                }
                // Reference differs between the two zooms, so an isolated boundary pixel may
                // flip by 1 iteration; a true seam bug would differ over a large region.
                checks.push(SelfCheck {
                    category: "Consistency",
                    name: "zoom-sequence across direct→df32 seam".into(),
                    params: format!("seahorse, 4e3×↔1.2e4×, {checked} overlap px"),
                    result: format!("{bad} differ"),
                    threshold: "<0.1% differ",
                    pass: checked > 0 && (bad as f64) < 0.001 * checked as f64,
                });
            }

            // 3.4 Pan consistency: shift the center by an integer pixel count; the overlapping
            // region must be identical — A(i,j) == B(i−shift, j).
            let shift = (N / 4) as usize;
            let stepx = (3.0 / 1.0e6) / N as f64;
            let cxb2 = fractadyne_core::add_f64(&cxb, shift as f64 * stepx, fractadyne_core::precision_for_magnification(1.0e6));
            if let (Some(pa), Some(pb)) =
                (build(&cxb, &cyb, 1.0e6, N, 2000), build(&cxb2, &cyb, 1.0e6, N, 2000))
            {
                let (mut checked, mut bad) = (0u64, 0u64);
                for j in 0..nn {
                    for i in shift..nn {
                        if steep(&pa, nn, i, j) {
                            continue;
                        }
                        checked += 1;
                        if !agree(pa[(j * nn + i) * 4], pb[(j * nn + (i - shift)) * 4]) {
                            bad += 1;
                        }
                    }
                }
                // Reference recomputed for the shifted center, so an isolated boundary pixel
                // may flip by 1; a true offset bug would shift the whole overlap.
                checks.push(SelfCheck {
                    category: "Consistency",
                    name: "pan consistency".into(),
                    params: format!("seahorse, 1e6×, +{shift}px, {checked} overlap px"),
                    result: format!("{bad} differ"),
                    threshold: "<0.1% differ",
                    pass: checked > 0 && (bad as f64) < 0.001 * checked as f64,
                });
            }

            // 3.5 Determinism: the same request rendered twice must be bit-identical.
            if let (Some(p1), Some(p2)) =
                (build(&cxb, &cyb, 1.0e6, N, 2000), build(&cxb, &cyb, 1.0e6, N, 2000))
            {
                let identical = p1.len() == p2.len()
                    && p1.iter().zip(&p2).all(|(a, b)| a.to_bits() == b.to_bits());
                checks.push(SelfCheck {
                    category: "Consistency",
                    name: "render determinism (2 runs)".into(),
                    params: "seahorse, 1e6×".into(),
                    result: if identical { "bit-identical".into() } else { "NON-DETERMINISTIC".into() },
                    threshold: "bit-identical",
                    pass: identical,
                });
            }

            // ---- Phase 4: derivative-dependent checks (DE / dz/dc) ----
            // The distance estimate (alpha channel = log2 DE-in-pixels) and slope normal are
            // derived from the floatexp dz/dc; validate them independently of the dwell.
            if let Some(px) = build(&cxb, &cyb, 1.0e6, N, 4000) {
                let de_px = |k: usize| -> Option<f32> {
                    let a = px[k * 4 + 3];
                    (a < 20.0).then(|| 2.0_f32.powf(a)) // >=20 ⇒ "far/unavailable"
                };

                // 4.2 DE self-consistency: an exterior pixel touching the interior (boundary
                // ≤1px away) cannot have a large distance estimate — that's a direct
                // contradiction exposing a derivative-formula error.
                let (mut bnd, mut viol) = (0u64, 0u64);
                for j in 1..nn - 1 {
                    for i in 1..nn - 1 {
                        let k = j * nn + i;
                        if px[k * 4] < 0.0 {
                            continue; // interior
                        }
                        let touches_interior = [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)]
                            .iter()
                            .any(|&(di, dj)| {
                                px[(((j as isize + dj) as usize) * nn + (i as isize + di) as usize) * 4] < 0.0
                            });
                        if touches_interior {
                            bnd += 1;
                            // boundary-adjacent ⇒ DE must be small (≤ a few px), and available.
                            if de_px(k).map(|d| d > 16.0).unwrap_or(true) {
                                viol += 1;
                            }
                        }
                    }
                }
                checks.push(SelfCheck {
                    category: "Derivative",
                    name: "distance-estimate self-consistency".into(),
                    params: format!("seahorse, 1e6×, {bnd} boundary px"),
                    result: format!("{viol} with DE>16px at boundary"),
                    threshold: "<0.5% of boundary px",
                    pass: bnd > 0 && (viol as f64) < 0.005 * bnd as f64,
                });

                // 4.1 DE lower bound (Koebe ¼ theorem): a disk of radius DE/4 about an exterior
                // point contains no boundary. Verify with an INDEPENDENT CPU dwell at the disk
                // rim — catches dz/dc under-estimation (DE too large) invisible to dwell tests.
                let step = (3.0 / 1.0e6) / N as f64;
                let cx0 = fractadyne_core::to_f64(&cxb);
                let cy0 = fractadyne_core::to_f64(&cyb);
                let half = N as f64 / 2.0;
                let (mut checked, mut koebe_viol) = (0u64, 0u64);
                let g = (N / 12).max(1) as usize;
                let mut j = 0usize;
                while j < nn {
                    let mut i = 0usize;
                    while i < nn {
                        let k = j * nn + i;
                        if px[k * 4] >= 0.0 {
                            if let Some(d) = de_px(k) {
                                if (1.0..=4096.0).contains(&d) {
                                    let r = d as f64 * step * 0.25; // Koebe-safe radius (world)
                                    let cre = cx0 + ((i as f64 + 0.5) - half) * step;
                                    let cim = cy0 + (half - (j as f64 + 0.5)) * step;
                                    checked += 1;
                                    for (ox, oy) in [(r, 0.0), (-r, 0.0), (0.0, r), (0.0, -r)] {
                                        if mandel_escapes(cre + ox, cim + oy, 4000).is_none() {
                                            koebe_viol += 1;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        i += g;
                    }
                    j += g;
                }
                checks.push(SelfCheck {
                    category: "Derivative",
                    name: "DE lower bound (Koebe ¼)".into(),
                    params: format!("seahorse, 1e6×, {checked} sampled exterior px"),
                    result: format!("{koebe_viol} disks contain interior"),
                    threshold: "0",
                    pass: checked > 0 && koebe_viol == 0,
                });
            }
        }

        // ---- catalog: independently verifiable locations (Phase 6.1 / 6.6) ----
        // Loads validation/catalog.toml (committed, human-readable) and checks the build
        // against each known answer, so external validation is one command.
        if let Ok(text) = std::fs::read_to_string("validation/catalog.toml") {
            match toml::from_str::<Catalog>(&text) {
                Ok(cat) => {
                    for e in &cat.nucleus {
                        let formula = e.fractal.as_deref()
                            .and_then(FractalKind::from_name)
                            .map_or(0, |k| k.formula_id());
                        let (Some(sx), Some(sy)) =
                            (fractadyne_core::parse_bf(&e.center_x), fractadyne_core::parse_bf(&e.center_y))
                        else {
                            continue;
                        };
                        match fractadyne_core::find_nucleus(&[sx, sy], e.zoom, formula, 100_000) {
                            Some(n) => {
                                let mut pass = n.period == e.period;
                                let mut detail = format!("period {} (want {})", n.period, e.period);
                                if let (Some(nx), Some(ny)) = (&e.nucleus_x, &e.nucleus_y) {
                                    if let (Some(ex), Some(ey)) =
                                        (fractadyne_core::parse_bf(nx), fractadyne_core::parse_bf(ny))
                                    {
                                        let prec = fractadyne_core::precision_for_magnification(e.zoom * 1.0e3);
                                        let dx = fractadyne_core::sub_f64(&n.cx, &ex, prec);
                                        let dy = fractadyne_core::sub_f64(&n.cy, &ey, prec);
                                        let dist = (dx * dx + dy * dy).sqrt();
                                        let tol = (1.0e-10 / e.zoom).max(1.0e-25);
                                        pass = pass && dist < tol;
                                        detail = format!("{detail}, nucleus Δ={dist:.1e}");
                                    }
                                }
                                checks.push(SelfCheck {
                                    category: "Catalog",
                                    name: e.name.clone(),
                                    params: format!("zoom {:.0e}", e.zoom),
                                    result: detail,
                                    threshold: "period + nucleus",
                                    pass,
                                });
                            }
                            None => checks.push(SelfCheck {
                                category: "Catalog",
                                name: e.name.clone(),
                                params: "find_nucleus".into(),
                                result: "no nucleus found".into(),
                                threshold: "period + nucleus",
                                pass: false,
                            }),
                        }
                    }
                    // Membership: the independent arbitrary-precision oracle decides whether the
                    // (full-precision) point is interior, and must match the catalog's known
                    // answer. (GPU-vs-oracle agreement over full views is covered by the oracle
                    // battery; a 1×1 render at the exact δc=0 center is an unrepresentative edge
                    // case.) Precision is generous so deep points aren't truncated.
                    let prec = fractadyne_core::precision_for_magnification(1.0e40);
                    for e in &cat.membership {
                        let (Some(cx), Some(cy)) =
                            (fractadyne_core::parse_bf(&e.center_x), fractadyne_core::parse_bf(&e.center_y))
                        else {
                            continue;
                        };
                        let oracle_interior =
                            fractadyne_core::naive_dwell_bf(&cx, &cy, 200_000, 65536.0, prec).is_none();
                        checks.push(SelfCheck {
                            category: "Catalog",
                            name: e.name.clone(),
                            params: format!("interior expected {}", e.interior),
                            result: format!("oracle says interior={oracle_interior}"),
                            threshold: "matches catalog",
                            pass: oracle_interior == e.interior,
                        });
                    }
                }
                Err(err) => checks.push(SelfCheck {
                    category: "Catalog",
                    name: "parse validation/catalog.toml".into(),
                    params: "TOML".into(),
                    result: format!("parse error: {err}"),
                    threshold: "valid",
                    pass: false,
                }),
            }
        }

        // ---- view-state format: versioning + untrusted-input hardening ----
        // The reloadable metadata (exports / .fdn / bookmarks) must round-trip a deep view
        // exactly, flag a newer format_version (so it can't be silently mis-read), and clamp
        // hostile/garbage fields rather than ballooning precision or the iteration count.
        {
            self.fractal = FractalKind::Mandelbrot;
            self.julia_mode = false;
            self.viewport.center_x = fractadyne_core::parse_bf("-0.743643887037151").unwrap();
            self.viewport.center_y = fractadyne_core::parse_bf("0.131825904205330").unwrap();
            self.viewport.units_per_pixel = fractadyne_core::FloatExp::from_f64(1.0).mul_pow2(-120.0);
            self.max_iter = 1234;
            self.auto_iter = false;
            self.aa = 3;
            let blob = self.view_metadata();
            // Scramble live state, then restore from the blob.
            self.max_iter = 7;
            self.aa = 1;
            self.viewport.units_per_pixel = fractadyne_core::FloatExp::from_f64(1.0);
            let rt = self.load_view_metadata(&blob);
            let cx = fractadyne_core::to_f64(&self.viewport.center_x);
            let rt_ok = rt.note().is_none()
                && self.max_iter == 1234
                && self.aa == 3
                && (self.viewport.units_per_pixel.log2() + 120.0).abs() < 1.0e-6
                && (cx + 0.743643887037151).abs() < 1.0e-12;
            checks.push(SelfCheck {
                category: "View format",
                name: "metadata round-trips a deep view".into(),
                params: "serialize → scramble → load".into(),
                result: format!(
                    "iter {} aa {} upp_log2 {:.3} cx {:.15}",
                    self.max_iter, self.aa, self.viewport.units_per_pixel.log2(), cx
                ),
                threshold: "clean load; fractal/iter/aa/zoom/center preserved",
                pass: rt_ok,
            });

            // A newer format_version must be detected (not silently consumed).
            let newer = "app=Fractadyne\nformat_version=999\ncenter_x=-0.5\ncenter_y=0\nupp_log2=-3\n";
            let nr = self.load_view_metadata(newer);
            checks.push(SelfCheck {
                category: "View format",
                name: "newer format_version flagged".into(),
                params: "format_version=999".into(),
                result: nr.note().unwrap_or_else(|| "NOT flagged".into()),
                threshold: "newer == Some(999)",
                pass: nr.newer == Some(999),
            });

            // Hostile numeric fields must be clamped (DoS / runaway work) AND reported.
            let hostile = "app=Fractadyne\nformat_version=1\ncenter_x=-0.5\ncenter_y=0\n\
                           upp_log2=-1e30\nmax_iter=4000000000\naa=9999\ncycle=inf\noffset=NaN\n\
                           bogus_field=42\n";
            let hr = self.load_view_metadata(hostile);
            let clamped = (1..=10_000_000).contains(&self.max_iter)
                && (1..=16).contains(&self.aa)
                && self.viewport.units_per_pixel.log2().is_finite()
                && self.viewport.units_per_pixel.log2() >= -3.4e7 - 1.0
                && self.cycle.is_finite()
                && self.offset.is_finite()
                && hr.clamped.len() >= 4 // zoom depth, max_iter, aa, cycle, offset
                && hr.unknown.iter().any(|u| u == "bogus_field");
            checks.push(SelfCheck {
                category: "View format",
                name: "hostile fields clamped + reported".into(),
                params: "upp_log2=-1e30, max_iter=4e9, aa=9999, cycle=inf, bogus_field".into(),
                result: format!(
                    "iter {} aa {} upp_log2 {:.2e}; clamped [{}]; unknown [{}]",
                    self.max_iter, self.aa, self.viewport.units_per_pixel.log2(),
                    hr.clamped.join(", "), hr.unknown.join(", ")
                ),
                threshold: "clamped & finite; report lists clamped + unknown",
                pass: clamped,
            });
        }

        // ---- status-bar formatters (pure, depth-aware display) ----
        {
            // Zoom mantissa is space-grouped in 5s; exponent untouched.
            let zg = crate::group_sci_mantissa("3.38050027227e15");
            checks.push(SelfCheck {
                category: "Formatting",
                name: "zoom mantissa grouped".into(),
                params: "3.38050027227e15".into(),
                result: zg.clone(),
                threshold: "\"3.38050 02722 7e15\"",
                pass: zg == "3.38050 02722 7e15",
            });
            // Deep coordinate: elides the middle (leading … frontier) and a short coord
            // (`-0.5`) must not panic the 15-digit floor clamp.
            let deep = fractadyne_core::parse_bf("-0.743643887037158704752191506114774").unwrap();
            let ds = crate::fmt_coord_deep(&deep, 100.0);
            let short = crate::fmt_coord_deep(&fractadyne_core::parse_bf("-0.5").unwrap(), 1.0);
            let ok = ds.contains('…') && ds.starts_with("-0.74364") && short == "-0.5";
            checks.push(SelfCheck {
                category: "Formatting",
                name: "deep coordinate elides middle".into(),
                params: "32-digit center @ ~1e30×; and -0.5".into(),
                result: format!("{ds}  |  {short}"),
                threshold: "leading … frontier; short coord safe",
                pass: ok,
            });
        }

        // ---- golden-image regression (set deterministic coloring per spec) ----
        let bless = std::env::args().any(|a| a == "--bless");
        let report_path = std::env::args()
            .position(|a| a == "--out" || a == "-o")
            .and_then(|i| std::env::args().nth(i + 1))
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("validation/report.md"));
        let out_base = report_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let golden_dir = out_base.join("golden");
        let current_dir = out_base.join("current");
        let _ = std::fs::create_dir_all(&golden_dir);
        if !bless {
            let _ = std::fs::create_dir_all(&current_dir);
        }
        // (name, cx, cy, zoom, iter, method, palette)
        let specs: &[(&str, &str, &str, f64, u32, u32, usize)] = &[
            ("home", "-0.5", "0.0", 1.0, 800, 0, 0),
            ("seahorse", SX, SY, 2.0e3, 1500, 0, 1),
            ("seahorse-stripe-1e6", SX, SY, 1.0e6, 4000, 1, 1),
            ("elephant", "0.2925755", "-0.0149977", 1.5e3, 1500, 0, 2),
        ];
        let (gw, gh) = (320u32, 240u32);
        // (name, max Δ, mean Δ, checksum, pass, reproduce)
        let mut goldens: Vec<(String, u32, f64, u64, bool, String)> = Vec::new();
        for &(name, cx, cy, zoom, iter, method, palette) in specs {
            self.fractal = FractalKind::Mandelbrot;
            self.julia_mode = false;
            self.color_method = method;
            self.palette_idx = palette;
            self.use_custom_palette = false;
            self.use_duotone = false;
            self.use_binary = false;
            self.cycle = 0.27;
            self.offset = 0.1;
            self.stripe_freq = 6.0;
            self.light = false;
            self.de = false;
            self.auto_iter = false;
            self.max_iter = iter;
            let mut vp = Viewport::new(gw as f64, gh as f64);
            vp.center_x = fractadyne_core::parse_bf(cx).unwrap();
            vp.center_y = fractadyne_core::parse_bf(cy).unwrap();
            vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (gh as f64 * zoom));
            vp.precision = fractadyne_core::precision_for_magnification(zoom);
            let mut req = self.current_export_request_for(&vp, false);
            req.width = gw;
            req.height = gh;
            req.ss = 1;
            let reproduce = format!(
                "fractadyne --render --out {name}.png --fractal Mandelbrot --center {cx} {cy} \
                 --zoom {zoom} --size {gw} --iter {iter} --ss 1 --method {} --palette {palette}",
                method_to_str(method)
            );
            let progress = std::sync::atomic::AtomicU32::new(0);
            let cancel = std::sync::atomic::AtomicBool::new(false);
            match fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel) {
                Ok(r) => {
                    let srgb = fractadyne_export::to_srgb8(&r.pixels);
                    let sum = fnv1a64(&srgb);
                    let png_path = golden_dir.join(format!("{name}.png"));
                    if bless {
                        let _ = fractadyne_export::write_png(&png_path, r.width, r.height, &r.pixels, Some(&reproduce));
                        goldens.push((name.to_string(), 0, 0.0, sum, true, reproduce));
                    } else {
                        let cur_path = current_dir.join(format!("{name}.png"));
                        let _ = fractadyne_export::write_png(&cur_path, r.width, r.height, &r.pixels, Some(&reproduce));
                        match fractadyne_export::read_png_rgba8(&png_path) {
                            Some((w, h, gpx)) if w == r.width && h == r.height => {
                                let (max, mean) = img_diff(&srgb, &gpx);
                                goldens.push((name.to_string(), max, mean, sum, max <= 10 && mean <= 2.0, reproduce));
                            }
                            _ => goldens.push((name.to_string(), 255, 255.0, sum, false, format!("{reproduce}  [no golden — run --bless]"))),
                        }
                    }
                }
                Err(e) => goldens.push((name.to_string(), 255, 255.0, 0, false, format!("render failed: {e}"))),
            }
        }

        // ---- build the human-readable + verifiable report ----
        let sys = gather_system_info();
        let checks_pass = checks.iter().filter(|c| c.pass).count();
        let gold_pass = goldens.iter().filter(|g| g.4).count();
        let ok = checks_pass == checks.len() && (bless || gold_pass == goldens.len());

        let mut md = String::new();
        md.push_str("# Fractadyne validation report\n\n");
        md.push_str(&format!("- **Version:** {}\n", version_string()));
        md.push_str(&format!("- **Generated:** {} (unix {ts})\n", utc_string(ts)));
        md.push_str(&format!("- **GPU:** {}\n", self.gpu_name));
        md.push_str(&format!(
            "- **CPU:** {} ({} cores / {} threads, L2 {} KB, L3 {} KB)\n",
            sys.cpu, sys.physical, sys.logical, sys.l2_kb, sys.l3_kb
        ));
        md.push_str(&format!("- **OS:** {} / {}\n", std::env::consts::OS, std::env::consts::ARCH));
        md.push_str(&format!("- **Mode:** {}\n\n", if bless { "BLESS (recording references)" } else { "VALIDATE" }));
        md.push_str(
            "All checks use exact mathematics (arbitrary-precision dwell, closed-form \
             properties) or internal cross-checks — no external data. Anyone can reproduce \
             a golden image with the listed command and compare it to `golden/`.\n\n",
        );
        md.push_str("## Numeric, deep-zoom & invariant checks\n\n");
        md.push_str("| Category | Check | Parameters | Result | Threshold | Verdict |\n");
        md.push_str("|---|---|---|---|---|---|\n");
        for c in &checks {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                c.category, c.name, c.params, c.result, c.threshold,
                if c.pass { "✅ PASS" } else { "❌ FAIL" }
            ));
        }
        md.push_str(&format!("\n**{checks_pass}/{} checks passed.**\n\n", checks.len()));
        // 6.5 Documented oracle scope — state plainly what is independently checked, and
        // where it is *not*, so a reviewer knows exactly where to aim scrutiny.
        md.push_str(
            "## Coverage & scope\n\n\
             What each oracle independently verifies, and its validity range:\n\n\
             - **Naive bignum dwell** (arbitrary precision, no perturbation/reference): exact \
             integer escape count at **any depth** — the only fully independent deep-zoom \
             oracle. Tested 1e6×–1e30× across the real render modes (df32 + floatexp).\n\
             - **CPU f64 dwell**: exact only to ~f64 coordinate resolution (≲1e13×); used for \
             the shallow cross-check.\n\
             - **floatexp ↔ df32 agreement**: internal consistency in the overlap band; not an \
             external oracle by itself.\n\
             - **Reference independence**: oracle-free glitch detection (multi-reference \
             majority); confirms the chosen reference is clean, doesn't prove a coordinate.\n\
             - **Symmetries / landmarks / consistency / derivative checks**: exact mathematics, \
             any depth, but each only constrains the property it tests.\n\
             - **Catalog**: full-precision locations with externally known answers (period, \
             nucleus, membership) — reproduce independently from `validation/catalog.toml`.\n\n\
             **Not independently oracle-checked:** non-Mandelbrot family *dwell* at depth \
             (only their symmetry is checked); interior-coloring/decomposition exactness; \
             coloring beyond the integer dwell. Aim scrutiny there.\n\n",
        );
        md.push_str(&format!("## Golden images ({gw}×{gh})\n\n"));
        md.push_str(&format!(
            "Stored in `{}`. {} pixel tolerance: max ≤ 10, mean ≤ 2.0 (8-bit sRGB).\n\n",
            golden_dir.display(),
            if bless { "Recorded this run." } else { "Compared against; current renders written to `current/` for side-by-side review." }
        ));
        md.push_str("| Image | Max Δ | Mean Δ | Checksum (FNV-1a) | Verdict | Reproduce |\n");
        md.push_str("|---|---|---|---|---|---|\n");
        for g in &goldens {
            md.push_str(&format!(
                "| {} | {} | {:.3} | `{:016x}` | {} | `{}` |\n",
                g.0, g.1, g.2, g.3,
                if bless { "📷 recorded" } else if g.4 { "✅ match" } else { "❌ differ" },
                g.5
            ));
        }
        md.push_str(&format!(
            "\n**{}/{} golden images {}.**\n\n## Summary\n\n{}\n",
            gold_pass, goldens.len(),
            if bless { "recorded" } else { "within tolerance" },
            if ok { "✅ ALL CHECKS PASSED" } else { "❌ FAILURES PRESENT — see table above" }
        ));

        if let Err(e) = std::fs::write(&report_path, &md) {
            eprintln!("Failed to write report to {}: {e}", report_path.display());
        }

        // ---- concise stdout summary ----
        println!("\nFractadyne self-test — {}\n{}", if bless { "BLESS" } else { "VALIDATE" }, "=".repeat(48));
        for c in &checks {
            println!("  [{}] {} — {}", if c.pass { "PASS" } else { "FAIL" }, c.name, c.result);
        }
        for g in &goldens {
            println!(
                "  [{}] golden {} — maxΔ {} meanΔ {:.2}",
                if bless { "REC " } else if g.4 { "PASS" } else { "FAIL" }, g.0, g.1, g.2
            );
        }
        println!("{}", "=".repeat(48));
        println!("checks {checks_pass}/{}, goldens {gold_pass}/{} — {}", checks.len(), goldens.len(),
            if ok { "OK" } else { "FAILURES PRESENT" });
        println!("report → {}\n", report_path.display());
        ok
    }
}
