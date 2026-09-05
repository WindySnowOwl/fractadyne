//! GPU validation self-test (`--selftest`): renders controlled views and cross-checks the
//! render paths against each other, against arbitrary-precision/CPU oracles, and against
//! golden images. Writes a verifiable Markdown report. (Exact numeric ground truth lives in
//! `fractadyne-core` unit tests; this validates the visual/render pipeline.)

use crate::{
    gather_system_info, mandel_escapes, utc_string, version_string, FractadyneApp,
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

/// Golden tolerances when running on the SAME GPU the goldens were blessed on: essentially
/// exact, with just enough room for driver-level noise.
const GOLDEN_MAX_STRICT: u32 = 10;
const GOLDEN_MEAN_STRICT: f64 = 2.0;

/// Golden tolerance when running on a DIFFERENT GPU from the one that blessed them.
///
/// Cross-vendor floating point legitimately disagrees: fma contraction, rounding, and — as the
/// 2026-08-14 measurements showed — whether the shader compiler preserves the df32 error-free
/// transforms at all. A pixel one iteration either side of an escape boundary lands somewhere
/// else entirely in a cycling palette, so `maxΔ` saturates at 255 on a perfectly healthy render
/// and is useless here; only the MEAN carries signal.
///
/// Calibrated against real hardware rather than guessed. An RX 6800 XT (whose Vulkan compiler
/// keeps the transforms, so its arithmetic differs from the reference 3080's by more than
/// rounding) produced meanΔ ≤ 0.1 on seven of the seventeen goldens, 0.5–0.9 on five, 2.8–4.6 on
/// three, and 19.15 / 16.51 on the two deep multibrots. This threshold sits above that worst
/// legitimate case while staying far below a structurally wrong render — an all-black or
/// misframed image scores 100+, so the check still catches real breakage.
///
/// This mode is INFORMATIONAL, never a release gate: the gate is exact-on-the-reference-GPU, and
/// the report always prints the numbers so a human can judge.
const GOLDEN_MEAN_CROSS_GPU: f64 = 24.0;

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


/// Frame predicates for the appearance checks (design/checklist-automation.md).
///
/// The manual checklist says things like "colouring visibly changes and produces a coherent image
/// (no all-black, all-white, or uniform flat frame)". These turn that into numbers.
///
/// ⚠A differential check ("A and B differ") is worthless on its own: it passes when one of them
/// is a blank frame, which is the failure it was written to catch. `coherent` must be asserted on
/// BOTH sides first, and every caller below does.
mod frame {
    /// Rec.709 luma of an RGBA8 buffer, one byte per pixel out.
    fn luma(px: &[u8]) -> Vec<f32> {
        px.chunks_exact(4)
            .map(|p| 0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32)
            .collect()
    }

    /// Tonal spread and how many 16-level buckets are occupied. A real fractal frame has both;
    /// all-black, all-white and uniform-flat frames have neither.
    pub(super) fn coherence(px: &[u8]) -> (f32, usize) {
        let l = luma(px);
        if l.is_empty() {
            return (0.0, 0);
        }
        let mean = l.iter().sum::<f32>() / l.len() as f32;
        let var = l.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / l.len() as f32;
        let mut buckets = [0u32; 16];
        for v in &l {
            buckets[((v / 16.0) as usize).min(15)] += 1;
        }
        // A bucket counts only if it holds ≥0.5% of the frame, so dithering noise in an otherwise
        // flat frame cannot fake tonal range.
        let floor = (l.len() as f32 * 0.005) as u32;
        (var.sqrt(), buckets.iter().filter(|&&c| c > floor).count())
    }

    /// Does this look like a rendered image rather than a flat fill?
    ///
    /// The bar is FLATNESS, which is what the checklist row asks for - "no all-black,
    /// all-white, or uniform flat frame" - not prettiness. A first attempt at stddev ≥ 6 with
    /// ≥ 3 buckets rejected orbit-trap and binary renders, which are legitimately low-contrast
    /// at a shallow view; tightening past the stated requirement would have turned a real
    /// check into a taste argument. `the flat-frame control is rejected` pins the other end,
    /// so this cannot be loosened into something that accepts everything.
    ///
    /// Occupied-bucket count is NOT part of the verdict. It was, and it failed the binary
    /// render: two-tone output puts nearly every pixel in one 16-level band while being far
    /// from the other, giving stddev 10.6 across a single bucket. Spread alone separates that
    /// cleanly from the flat control's 0.3 - a 30x margin - and does not punish an image for
    /// having few distinct tones, which is not what "flat" means.
    pub(super) fn coherent(px: &[u8]) -> bool {
        let (sd, b) = coherence(px);
        let _ = b; // reported as diagnostics; the verdict is spread alone, see above
        sd >= 1.0
    }

    /// Mean absolute per-channel difference. Same-size buffers only.
    pub(super) fn distance(a: &[u8], b: &[u8]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return f64::INFINITY;
        }
        let mut sum = 0u64;
        for (x, y) in a.iter().zip(b) {
            sum += (*x as i32 - *y as i32).unsigned_abs() as u64;
        }
        sum as f64 / a.len() as f64
    }

    /// Mean absolute luma step between horizontally adjacent pixels — the aliasing measure from
    /// the live-normalisation work. High means salt-and-pepper speckle; low means smooth bands.
    /// Doubles as an edge-energy measure for the anti-aliasing rows.
    pub(super) fn neighbour_step(px: &[u8], w: u32) -> f64 {
        let l = luma(px);
        let w = w as usize;
        if w < 2 || l.len() < w * 2 {
            return 0.0;
        }
        let (mut sum, mut n) = (0.0f64, 0u64);
        for row in l.chunks_exact(w) {
            for pair in row.windows(2) {
                sum += (pair[1] - pair[0]).abs() as f64;
                n += 1;
            }
        }
        if n == 0 { 0.0 } else { sum / n as f64 }
    }
}

/// One row of the validation report.
/// Stream each check result the moment it lands (design/diagnostics.md D2.3): a suite that
/// buffers everything to the end cannot name its slow or hung check — this one names it live,
/// with the elapsed time since the previous check (which includes this check's own setup).
fn push_check(checks: &mut Vec<SelfCheck>, last: &mut std::time::Instant, c: SelfCheck) {
    let ms = last.elapsed().as_millis();
    *last = std::time::Instant::now();
    eprintln!(
        "[selftest {:>7}ms] {} {} — {}",
        ms,
        if c.pass { "PASS" } else { "FAIL" },
        c.name,
        c.result
    );
    // "Last completed check" is what the watchdog/crash report names when the NEXT check
    // wedges — exactly how the 2-hour F10 hog was identified.
    crate::diag::breadcrumb(format!("selftest: after '{}'", c.name));
    checks.push(c);
}

/// Resolve a repo-relative data path (D2.6/F12): prefer the CWD (the normal repo-root
/// invocation), else walk up from the executable (target/release/… → repo root) looking
/// for the `validation/` tree. A suite run from another directory must not silently lose
/// whole check categories — callers still fail LOUDLY if the file is absent everywhere.
pub(crate) fn anchored(rel: &str) -> std::path::PathBuf {
    let cwd = std::path::PathBuf::from(rel);
    if cwd.exists() || std::path::Path::new("validation").exists() {
        return cwd;
    }
    if let Ok(exe) = std::env::current_exe() {
        for dir in exe.ancestors().skip(1) {
            if dir.join("validation").exists() {
                return dir.join(rel);
            }
        }
    }
    cwd
}

/// `render_iter` that PRINTS a GPU error instead of swallowing it (design/diagnostics.md
/// D2.5/F11): the suite's checks skip on `None`, so without this a device-level failure
/// silently shrank the check count instead of naming itself.
fn st_render_iter(
    device: &eframe::wgpu::Device,
    queue: &eframe::wgpu::Queue,
    req: &fractadyne_gpu::ExportRequest,
) -> Option<Vec<f32>> {
    match fractadyne_gpu::render_iter(device, queue, req) {
        Ok(r) => Some(r.pixels),
        Err(e) => {
            eprintln!("[selftest] GPU ERROR (render_iter): {e}");
            None
        }
    }
}

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
        // HERMETIC BASELINE (design/diagnostics.md D2.1): reset every config field the checks
        // read to documented values, so nothing leaks in from the live session. Three real
        // incidents came from ad-hoc per-check pinning: a stripe session gated SA off (v0.2.1,
        // 58/60), a staged session disabled series_approx (v0.2.6, 58/61), and a 500k-max_iter
        // session turned the "SA seed vs full iteration" SA-off arm and the CPU bignum oracles
        // into a 2+-hour suite once v0.2.5 stopped capping explicit counts. `--selftest` exits
        // via process::exit without saving the session, so no restore is needed.
        self.fractal = FractalKind::Mandelbrot;
        self.julia_mode = false;
        self.dual = false;
        self.render_cfg.auto_iter = true; // depth-scaled counts, as every check was designed for
        self.render_cfg.max_iter = 4000;
        self.render_cfg.series_approx = true;
        self.render_cfg.use_bla = true;
        self.render_cfg.glitch_correct = true; // exports set glitch_on:0; pinned for completeness
        self.coloring.color_method = crate::ColorMethod::Smooth;
        self.coloring.use_custom_palette = false;
        self.coloring.use_binary = false;
        self.coloring.use_duotone = false;
        self.effects.de = false;
        self.effects.light = false;
        // D2.2: echo the effective config so any residual leak is visible in the report
        // itself (the two hermeticity incidents were only diagnosable by guessing).
        let cfg_echo = format!(
            "fractal={:?} julia={} auto_iter={} max_iter={} sa={} bla={} glitch={} color={:?}",
            self.fractal,
            self.julia_mode,
            self.render_cfg.auto_iter,
            self.render_cfg.max_iter,
            self.render_cfg.series_approx,
            self.render_cfg.use_bla,
            self.render_cfg.glitch_correct,
            self.coloring.color_method,
        );
        eprintln!("[selftest] config: {cfg_echo}");

        // D2.7: `--selftest-filter <substr>` runs only the check groups (and goldens) whose
        // tag or name matches; `--selftest-list` prints the group tags and exits. Groups
        // share config state at their boundaries (F13), so a filtered verdict is for
        // ITERATION, not release gating — the summary says so when a filter is active.
        // The flags come from `new()` (the EXPANDED args), NOT std::env::args(), so
        // `@response-file` expansion is honored (raw args would silently drop them).
        let filter: Option<String> = self.selftest.filter.clone();
        const GROUPS: &[&str] = &[
            "numeric", "symmetry", "abs-family", "multibrot-sa", "bla", "aux-bla",
            "consistency", "counters", "iter-budget", "iter-chunk", "nr-zoom", "coords",
            "ref-pick", "script", "metadata",
            "display", "catalog", "goldens", "bench-matrix", "live-res", "appearance",
            "checklist",
        ];
        if self.selftest.list {
            println!("selftest groups (use with --selftest-filter <substr>):");
            for g in GROUPS {
                println!("  {g}");
            }
            crate::exit(0);
        }
        // (A filter that runs ZERO checks is rejected AFTER the suite — see the guard just
        // before the report is written. Doing it post-hoc matches on what actually ran, so
        // it can't drift from the group/golden name lists the way a pre-flight check would.)
        let want = |tag: &str| -> bool {
            filter.as_ref().is_none_or(|f| tag.to_ascii_lowercase().contains(f.as_str()))
        };
        if let Some(f) = &filter {
            eprintln!("[selftest] FILTERED RUN (--selftest-filter {f}): group state is shared — use full runs for verdicts");
        }
        // Seahorse Valley — detailed at every depth tested; coordinate precise enough.
        const SX: &str = "-0.743643887037151";
        const SY: &str = "0.131825904205330";
        const N: u32 = 220;

        // Read back the raw iteration texture (smooth_iter, normal.x, normal.y, DE) — far
        // more sensitive than comparing final colors. GPU errors are printed, not swallowed
        // (D2.5): a device-level failure must name itself, not shrink the check count.
        let render = |req: &fractadyne_gpu::ExportRequest| -> Option<Vec<f32>> {
            match fractadyne_gpu::render_iter(device, queue, req) {
                Ok(r) => Some(r.pixels),
                Err(e) => {
                    eprintln!("[selftest] GPU ERROR (render_iter): {e}");
                    None
                }
            }
        };
        // A square request at the seahorse, then caller overrides the mode. Takes the app
        // explicitly (no captured `self` borrow) so checks can flip `render_cfg` knobs — e.g.
        // `use_bla`, which since the SA⊂BLA gate also decides whether SA is computed — between calls.
        // ⚠`mag` here is NOT the view magnification. This sets units_per_pixel from a 3-unit span
        // while `Viewport::magnification()` measures against REFERENCE_HEIGHT = 4, so the view
        // this builds sits at 4/3 × `mag`. Harmless for checks that just want "deep" (the labels
        // below are nominal), but it silently defeats anything testing a THRESHOLD: a crossover
        // check written at 7.9e27 renders at 1.06e28 and lands on the far side of the 1e28 switch.
        // Scale by 3/4 when you need a specific magnification.
        let make = |app: &Self, cx: &str, cy: &str, mag: f64| -> fractadyne_gpu::ExportRequest {
            let mut vp = Viewport::new(N as f64, N as f64);
            vp.center_x = fractadyne_core::parse_bf(cx).unwrap();
            vp.center_y = fractadyne_core::parse_bf(cy).unwrap();
            vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mag));
            vp.precision = fractadyne_core::precision_for_magnification(mag);
            let mut req = app.current_export_request_for(&vp, false);
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
        let mut last_check_t = std::time::Instant::now();

        // ---- iteration-range tiling: the chunked resumable path must be BIT-IDENTICAL to the
        // single-pass fs_iterate for direct mode (it replicates the arithmetic and order exactly,
        // carrying full df32 state between bounded dispatches). An odd chunk size forces many
        // passes plus a partial final one; the home view mixes interior (full-count grind) and
        // escaped pixels, the seahorse is escape-heavy. This is what makes it safe to route a
        // watchdog-threatening direct frame through the chunked path: same picture, many short
        // dispatches. ----
        if want("iter-chunk") {
            let bit_exact = |a: &[f32], b: &[f32]| -> (usize, f32) {
                let mut diffs = 0usize;
                let mut maxd = 0.0f32;
                for (x, y) in a.iter().zip(b.iter()) {
                    if x.to_bits() != y.to_bits() {
                        diffs += 1;
                        maxd = maxd.max((x - y).abs());
                    }
                }
                (diffs, maxd)
            };
            // (view, mag, max_iter, chunk, truncate, expected mode, desc) — direct mode
            // (mag < 1e4), df32 perturbation (mag ≥ 1e4; mode 0 resumes δz + the floatexp
            // derivative + ref_n, rebasing across chunk boundaries) and floatexp perturbation
            // (mode 2, four state targets). The truncated-orbit cases force an end-of-orbit rebase
            // STORM — the 99-sample-reference grind regime of the 197k× spar, in miniature, and the
            // 250k-against-119,563 shape of the 2026-08-18 field device loss.
            //
            // ⚠The expected mode is CHECKED, not assumed. `make`'s `mag` is 3/4 of the view
            // magnification (a 3-unit span against REFERENCE_HEIGHT = 4), so a mode-2 case written
            // at 1e28 would render in mode 0 and this group would quietly become five more mode-0
            // checks — the same silent-downgrade shape as a harness that runs a config it isn't.
            //
            // The mode-2 rows split 21,000 iterations into 2, 3 and 7 passes over the same view, so
            // the boundaries land at different absolute iterations each time; with a rebase every
            // ~97 steps in the truncated row, boundaries land ON rebases across the grid rather than
            // by luck at one hand-picked iteration.
            // ⚠Mode 2 needs a coordinate with enough DIGITS, not just a big magnification. The
            // 15-digit seahorse above is garbage past ~1e15×: at 1e30× its reference escapes after
            // 3090 samples and SA seeds every pixel at 3088, so the pixels escape ~2 iterations
            // later and the "chunked" render agrees trivially — zero rebases, zero BLA skips, and
            // nothing of the chunk path exercised. Corpus loc 07 (44 digits) and the 38-digit
            // minibrot nucleus are the real deep points the numeric battery uses.
            const CRX: &str = "-1.178853950372678747911373866849720956148855";
            const CRY: &str = "0.1853420232408490265512092752061929308714979";
            const NX: &str = "-0.74364388703715887077806454349323251348";
            const NY: &str = "0.131825904205312292821097354874199108694";
            let cases: &[(&str, &str, f64, u32, u32, bool, u32, &str)] = &[
                ("-0.5", "0.0", 1.0, 2_000, 137, false, 1, "home 1×, 2000 iter, chunk 137"),
                (SX, SY, 2.0e3, 2_000, 137, false, 1, "seahorse 2e3×, 2000 iter, chunk 137"),
                ("-0.5", "0.0", 1.0, 50_000, 7_000, false, 1, "home 1×, 50k iter, chunk 7000"),
                (SX, SY, 2.0e4, 3_000, 517, false, 0, "mode0 seahorse 2e4×, 3000 iter, chunk 517"),
                (SX, SY, 2.0e4, 20_000, 700, true, 0, "mode0 97-sample ref (rebase storm), 20k iter, chunk 700"),
                (CRX, CRY, 1.0e30, 21_000, 10_500, false, 2, "mode2 corpus07 1.3e30×, 21k iter, 2 passes"),
                (CRX, CRY, 1.0e30, 21_000, 7_000, false, 2, "mode2 corpus07 1.3e30×, 21k iter, 3 passes"),
                (CRX, CRY, 1.0e30, 21_000, 3_000, false, 2, "mode2 corpus07 1.3e30×, 21k iter, 7 passes"),
                (NX, NY, 1.0e30, 21_000, 3_000, false, 2, "mode2 nucleus 1.3e30× (interior), 21k iter, 7 passes"),
                (CRX, CRY, 1.0e30, 21_000, 2_600, true, 2, "mode2 97-sample ref (orbit wraps), 21k iter, chunk 2600"),
            ];
            for (cx, cy, mag, max_iter, chunk, truncate, want_mode, desc) in cases {
                let mut req = make(self, cx, cy, *mag);
                req.max_iter = *max_iter;
                if *truncate {
                    // A deliberately useless reference: every pixel rebases at the orbit end
                    // every ~97 steps, in BOTH renders — the chunked path must reproduce the
                    // storm bit-for-bit across chunk boundaries.
                    let short: Vec<[f32; 4]> = req.orbit.iter().take(97).copied().collect();
                    req.orbit = std::sync::Arc::new(short);
                    req.orbit_len = 97;
                    req.sa_skip = 0;
                    req.bla = std::sync::Arc::new(Vec::new());
                    req.bla_on = 0;
                }
                let single = render(&req);
                let chunked = fractadyne_gpu::render_iter_chunked(device, queue, &req, *chunk)
                    .map_err(|e| eprintln!("[selftest] GPU ERROR (render_iter_chunked): {e}"))
                    .ok();
                let (pass, result) = if req.mode != *want_mode {
                    // Not "the arithmetic differs" — the case did not test what it is named after,
                    // and a bit-identity pass in the wrong mode is worse than a failure.
                    (false, format!("ran in mode {} not {want_mode}", req.mode))
                } else {
                    match (&single, &chunked) {
                        (Some(a), Some(r)) if a.len() == r.pixels.len() => {
                            let (diffs, maxd) = bit_exact(a, &r.pixels);
                            let bla = r.counters[fractadyne_gpu::CTR_BLA_SKIP];
                            let reb = r.counters[fractadyne_gpu::CTR_REBASE];
                            // ⚠Mode 2 is the only mode that traverses the BLA tree, and each chunk
                            // pass rebuilds its own table — so the untruncated mode-2 rows must SHOW
                            // skips, not merely agree. Bit-identity alone cannot certify this:
                            // if BLA silently switched off in BOTH renders they would still agree,
                            // and the chunked path would be running the beta.101 e100 pathology
                            // (0.04 Gsteps/s against 174 in the same frame) with a green gate.
                            let bla_ok = *want_mode != 2 || *truncate || bla > 0;
                            (
                                diffs == 0 && bla_ok,
                                format!(
                                    "mode {} — {diffs} texels differ (max Δ {maxd:.3e}), bla_skip {bla}, rebase {reb}",
                                    req.mode
                                ),
                            )
                        }
                        _ => (false, "render failed".into()),
                    }
                };
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "IterChunk",
                    name: "chunked render is bit-identical".into(),
                    params: (*desc).into(),
                    result,
                    threshold: "0 texels differ (mode 2: and BLA engaged)",
                    pass,
                });
            }
        }

        // (D4) The TILED chunked iterate — the per-tile windowed dispatch that fixed the 5K
        // export device loss (crash-1787292746). Same shaders as the battery above, but through
        // the tile loops' integration: scissored shared ping-pong state reused across tiles,
        // wall-priced windows (`ChunkPricer`), per-tile counter epochs, and `fs_resolve` into
        // each tile's G-buffer. `max_iter` is far above the 400k opening window and the 2e10
        // nominal tile bound, so every tile runs several windows and the frame runs 16 tiles —
        // both integrations must reproduce their single-dispatch control bit-for-bit.
        if want("iter-chunk") {
            let mag = 1.0e30;
            const CRX: &str = "-1.178853950372678747911373866849720956148855";
            const CRY: &str = "0.1853420232408490265512092752061929308714979";
            let mut vp = Viewport::new(N as f64, N as f64);
            vp.center_x = fractadyne_core::parse_bf(CRX).unwrap();
            vp.center_y = fractadyne_core::parse_bf(CRY).unwrap();
            vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mag));
            vp.precision = fractadyne_core::precision_for_magnification(mag);
            let saved_iter = self.render_cfg.max_iter;
            let saved_auto = self.render_cfg.auto_iter;
            let saved_method = self.coloring.color_method;
            self.render_cfg.max_iter = 4_000_000;
            self.render_cfg.auto_iter = false;
            // Aux methods are out of chunk scope by design; pin Smooth so the case exercises
            // the chunked path regardless of what the session left selected.
            self.coloring.color_method = crate::ColorMethod::Smooth;
            let mut req = self.current_export_request_for(&vp, false);
            req.width = N;
            req.height = N;
            req.ss = 1;
            self.render_cfg.max_iter = saved_iter;
            self.render_cfg.auto_iter = saved_auto;
            self.coloring.color_method = saved_method;
            let bit_exact = |a: &[f32], b: &[f32]| -> usize {
                a.iter().zip(b).filter(|(x, y)| x.to_bits() != y.to_bits()).count()
            };
            if req.mode != 2 {
                // The case did not test what it is named after (mode-2 fe chunking through the
                // tile loops) — a bit-identity pass in the wrong mode would be worse than a fail.
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "IterChunk",
                    name: "tiled chunked export is bit-identical".into(),
                    params: "corpus07 1e30x, 4M iter, 16 tiles".into(),
                    result: format!("ran in mode {} not 2", req.mode),
                    threshold: "mode 2",
                    pass: false,
                });
            } else {
                use std::sync::atomic::{AtomicBool, AtomicU32};
                let progress = AtomicU32::new(0);
                let cancel = AtomicBool::new(false);
                let a = fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel)
                    .map_err(|e| eprintln!("[selftest] GPU ERROR (render_export): {e}"))
                    .ok();
                let b =
                    fractadyne_gpu::render_export_unchunked(device, queue, &req, &progress, &cancel)
                        .map_err(|e| eprintln!("[selftest] GPU ERROR (render_export_unchunked): {e}"))
                        .ok();
                let (pass, result) = match (&a, &b) {
                    (Some(a), Some(b)) if a.pixels.len() == b.pixels.len() => {
                        let diffs = bit_exact(&a.pixels, &b.pixels);
                        (
                            diffs == 0 && a.max_dispatch_ms > 0.0,
                            format!(
                                "{diffs} texels differ; max dispatch {:.0}ms vs control {:.0}ms",
                                a.max_dispatch_ms, b.max_dispatch_ms
                            ),
                        )
                    }
                    _ => (false, "render failed".into()),
                };
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "IterChunk",
                    name: "tiled chunked export is bit-identical".into(),
                    params: "corpus07 1e30x, 4M iter, 16 tiles, colored".into(),
                    result,
                    threshold: "0 texels differ",
                    pass,
                });

                // Same claim for `render_iter_tiled` (the normalized export's pass 1): raw
                // iteration buffer against the trusted single-dispatch `render_iter`.
                let t = fractadyne_gpu::render_iter_tiled(device, queue, &req, 20_000_000_000, None, None)
                    .map_err(|e| eprintln!("[selftest] GPU ERROR (render_iter_tiled): {e}"))
                    .ok();
                let u = fractadyne_gpu::render_iter(device, queue, &req)
                    .map_err(|e| eprintln!("[selftest] GPU ERROR (render_iter): {e}"))
                    .ok();
                let (pass, result) = match (&t, &u) {
                    (Some(t), Some(u)) if t.pixels.len() == u.pixels.len() => {
                        let diffs = bit_exact(&t.pixels, &u.pixels);
                        (diffs == 0, format!("{diffs} texels differ"))
                    }
                    _ => (false, "render failed".into()),
                };
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "IterChunk",
                    name: "tiled chunked iter buffer is bit-identical".into(),
                    params: "corpus07 1e30x, 4M iter, 16 tiles, raw".into(),
                    result,
                    threshold: "0 texels differ",
                    pass,
                });
            }
        }

        // ⭐⭐ONE COMPILED ENTRY POINT PER RENDER — the gate the corpus red at `06-seahorse-1e24`
        // actually needed, and the one the case above cannot provide.
        //
        // The case above asserts chunked == unchunked bit-for-bit. That claim is TRUE in mode 2 and
        // FALSE in mode 0 on this backend, and not because of a logic bug: `fs_iterate` and
        // `fs_iterate_chunk` are separate entry points compiled independently, and NVIDIA's Windows
        // backend folds them differently (the df32-EFT family again). Measured at corpus 06, ss=2:
        // 279 pixels and 47 rebases of 30.8M apart — and 378/67 apart with the chunk path forced
        // into ONE window carrying no state at all, so the boundary is innocent and the PROGRAM is
        // the variable. Re-asserting bit-identity here would just be a knowingly-red case.
        //
        // What IS enforceable, and what the fix establishes, is that a single render never MIXES
        // the two: the chunker is built up front from `chunk_scope` alone, so the entry point is a
        // property of the REQUEST and not of the tile budget, the adaptive cap, or where the
        // expensive region happened to sit. Under the old lazy build, location 06 rendered
        // `chunks=0` on one tile and `chunks=2` on eight — one image, two programs — which is why
        // its pixels moved when `TILE_WORK_BUDGET` moved.
        //
        // ⚠The case asserts `mode == 0`, `tiles_total > 1` and `tiles_chunked > 0` BEFORE the
        // invariant, because every one of those has been a false control in this investigation: a
        // single-tile render cannot mix, a render that never chunks cannot mix, and mode 2 is the
        // combination that already worked.
        //
        // ⭐⭐THE PARAMETERS ARE CHOSEN SO THE OLD CODE FAILS DETERMINISTICALLY, not incidentally.
        // `ChunkPricer::new().open(n) == min(400_000, n)` is a constant — no timing input — so the
        // lazy trigger `pricer.open(max_iter) < max_iter` is FALSE on the first tile for any ask at
        // or below 400k, and the old build therefore left tile 0 on `fs_iterate` no matter how fast
        // the machine was. `max_iter` is pinned at exactly 400_000 and `ss` at 2 so the frame is
        // several tiles: the old build then either mixes (a later hot tile teaches the pricer down,
        // `tiles_chunked < tiles_total`) or never chunks at all (`tiles_chunked == 0`), and BOTH
        // are red here. ⚠A 4M ask would NOT discriminate — the opening is 400k < 4M, so even the
        // lazy build chunks from tile 0 and reports a clean 26/26. That configuration was tried
        // first and passed on both builds; it is exactly the kind of check that looks like a gate
        // and is not one.
        //
        // ⚠What this case does NOT do is reproduce the historical corpus red. That needed the
        // 1280x720 ss=2 corpus geometry, where tile 0 landed at 415 ms against the pricer's 400 ms
        // threshold — i.e. a TIMING-marginal reproduction, unfit for a gate. The corpus itself
        // covers that; this case covers the structural property on every run.
        if want("iter-chunk") {
            let mag = 1.0e24;
            const C6X: &str = "-0.7436438870371587047521915061147707";
            const C6Y: &str = "0.131825904205311970493132056385139";
            let mut vp = Viewport::new(N as f64, N as f64);
            vp.center_x = fractadyne_core::parse_bf(C6X).unwrap();
            vp.center_y = fractadyne_core::parse_bf(C6Y).unwrap();
            vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mag));
            vp.precision = fractadyne_core::precision_for_magnification(mag);
            let saved_iter = self.render_cfg.max_iter;
            let saved_auto = self.render_cfg.auto_iter;
            let saved_method = self.coloring.color_method;
            // Exactly the pricer's opening bound: `open(400_000) == 400_000`, which is NOT
            // `< max_iter`, so the lazy rule cannot fire on tile 0. See the note above.
            self.render_cfg.max_iter = 400_000;
            self.render_cfg.auto_iter = false;
            self.coloring.color_method = crate::ColorMethod::Smooth;
            let mut req = self.current_export_request_for(&vp, false);
            req.width = N;
            req.height = N;
            req.ss = 2; // ss=1 here is a single tile, and a single tile cannot mix
            self.render_cfg.max_iter = saved_iter;
            self.render_cfg.auto_iter = saved_auto;
            self.coloring.color_method = saved_method;

            use std::sync::atomic::{AtomicBool, AtomicU32};
            let progress = AtomicU32::new(0);
            let cancel = AtomicBool::new(false);
            let r = if req.mode == 0 {
                fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel)
                    .map_err(|e| eprintln!("[selftest] GPU ERROR (render_export mode 0): {e}"))
                    .ok()
            } else {
                None
            };
            let (pass, result) = match (&r, req.mode) {
                // Guard the VARIABLE UNDER TEST first: a mode drift would silently re-run the
                // mode-2 coverage the case above already has.
                (_, m) if m != 0 => (false, format!("ran in mode {m} not 0")),
                (None, _) => (false, "render failed".into()),
                (Some(r), _) => {
                    let (t, c) = (r.tiles_total, r.tiles_chunked);
                    if t <= 1 {
                        (false, format!("{t} tile — a single-tile render cannot mix"))
                    } else if c == 0 {
                        // In chunk scope every tile must be chunked. 0 means the chunker was not
                        // built up front — the lazy build, or a device outside chunk scope. Fail
                        // loudly either way rather than report a vacuous pass.
                        (false, format!("{t} tiles, 0 chunked — chunker not built up front"))
                    } else {
                        (c == t, format!("{c}/{t} tiles chunked"))
                    }
                }
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "IterChunk",
                name: "mode-0 render uses ONE entry point".into(),
                params: "corpus06 1e24x, 400k iter, ss2, multi-tile".into(),
                result,
                threshold: "all tiles chunked",
                pass,
            });
        }

        // ---- numeric & render-path checks (local closures borrow self immutably) ----
        if want("numeric") {
            // (A) df32 perturbation vs an independent CPU f64 dwell @2e4× (f64 exact here).
            let mag = 2.0e4;
            let req = make(self, SX, SY, mag);
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
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Numeric",
                    name: "df32 perturbation vs CPU f64 dwell".into(),
                    params: format!("seahorse, 2e4×, {} iter, n={n}", req.max_iter),
                    result: format!("{:.1}% agree within 1 iter", (1.0 - frac) * 100.0),
                    threshold: "≥90% within 1 iter",
                    pass: frac < 0.10,
                });
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Finiteness",
                    name: "dwell finite (perturbation @2e4×)".into(),
                    params: "all sampled pixels".into(),
                    result: if finite(&px) { "all finite".into() } else { "NON-FINITE!".into() },
                    threshold: "all finite",
                    pass: finite(&px),
                });
            }

            // (B) floatexp vs df32 perturbation @1e10× — two representations, must agree.
            let mut a = make(self, SX, SY, 1.0e10);
            a.mode = 0;
            let mut b = a.clone();
            b.mode = 2;
            if let (Some(aa), Some(bb)) = (render(&a), render(&b)) {
                let (mean, frac) = compare(&aa, &bb);
                push_check(&mut checks, &mut last_check_t, SelfCheck {
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
            // actually uses (df32 perturbation through 9.3e27×, floatexp at ≥1.3e28×). This is
            // the only check that gives *independent* deep-zoom correctness (not internal
            // consistency). Full-precision deep coordinates use a 38-digit minibrot nucleus.
            const NX: &str = "-0.74364388703715887077806454349323251348";
            const NY: &str = "0.131825904205312292821097354874199108694";
            // ⚠MEASURED: at these depths the nucleus above fills the frame with the minibrot's
            // INTERIOR — at 9.3e27× all 48400 pixels reach max_iter without escaping. The oracle
            // still agrees there, but only on "never escapes"; it never compares an escape COUNT,
            // and a dwell comparison on that view has no pixels to average (n = 0). The crossover
            // three entries added below therefore use a structure-rich center — validation corpus location
            // 07, 43 digits, which at the same depth escapes on every pixel (maxiter = 0) and
            // takes ~986k rebases, so the oracle checks real dwell values.
            const CRX: &str = "-1.178853950372678747911373866849720956148855";
            const CRY: &str = "0.1853420232408490265512092752061929308714979";
            let battery: &[(&str, &str, &str, f64)] = &[
                ("1e6x", SX, SY, 1.0e6),
                ("1e12x", SX, SY, 1.0e12),
                ("1e16x", NX, NY, 1.0e16),
                ("1e24x", NX, NY, 1.0e24),
                // The df32→floatexp crossover sits at 1e28×, and this battery used to step
                // 1e24 → 1e30, straight over it: nothing pinned mode 0 near its own ceiling,
                // where its δ limbs are most stressed, and nothing pinned mode 2 just after it
                // takes over. Both sides of the switch now carry an independent oracle, so a
                // regression in either representation — or in where the selector draws the line
                // — fails here rather than surviving to a user's zoom through the boundary.
                // These two are pre-scaled by 3/4 (see the note on `make`) so they land where
                // the labels say: 9.3e27× is the deepest mode 0 the selector will hand out.
                ("1.3e26x", CRX, CRY, 1.0e26),
                ("9.3e27x (mode 0 ceiling)", CRX, CRY, 7.0e27),
                ("1.3e28x (mode 2 floor)", CRX, CRY, 1.0e28),
                ("1e30x", NX, NY, 1.0e30),
            ];
            for (label, cx, cy, mag) in battery {
                let req = make(self, cx, cy, *mag); // mode chosen by the real depth selector
                if let Some(px) = render(&req) {
                    let (checked, agree, boundary, mism) = oracle(cx, cy, *mag, req.max_iter, &px);
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Bignum oracle",
                        name: format!("naive bignum dwell vs GPU @{label}"),
                        params: format!("mode {}, {} iter, {checked} samples", req.mode, req.max_iter),
                        result: format!("{agree} agree, {boundary} boundary, {mism} mismatch"),
                        threshold: "0 hard mismatches",
                        pass: mism == 0 && checked > 0,
                    });
                } else {
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Bignum oracle",
                        name: format!("naive bignum dwell vs GPU @{label}"),
                        params: "render".into(),
                        result: "render failed".into(),
                        threshold: "0 hard mismatches",
                        pass: false,
                    });
                }
            }

            // (C2) floatexp vs df32 AT THE TOP OF MODE 0's RANGE. Check (B) already compares the
            // two representations, but at 1e10× — eighteen decades below the ~1e28× crossover, so
            // it exercises df32 where nothing is close to a limit. This runs the same comparison
            // at 9.3e27×, the deepest point the depth selector still hands to mode 0, where δ is
            // ~1e-31 and the df32 limbs carry the least headroom they ever do. Mode 2 is the
            // reference: it is independently oracle-pinned at 1.3e28× and 1e30× just above.
            //
            // ⚠This does NOT validate df32's lo limbs on NVIDIA, where the error-free transforms
            // are compiler-folded and mode 0 is effectively f32 (see topic-gpu-arithmetic). It
            // validates the mode as shipped on this machine, which is the thing a user renders.
            {
                let mut a = make(self, CRX, CRY, 7.0e27); // 3/4-scaled → 9.3e27× actual
                let selector_mode = a.mode;
                a.mode = 0;
                let mut b = a.clone();
                b.mode = 2;
                // ⚠THE MEAN BOUND IS CROSS-GPU AWARE, and the reason matters more than the number.
                // On the blessed card this comparison is DEGENERATE: NVIDIA's shader compiler folds
                // mode 0's error-free transforms, so mode 0 is effectively f32 and the two paths
                // agree EXACTLY (measured mean Δ 0.0000). That agreement is not evidence of accuracy
                // — it is two similarly-degraded paths converging, and calibrating a tight bound on
                // it was calibrating on the degenerate case.
                //
                // On AMD (RX 6800 XT, which PRESERVES the transforms — `--gputest` shows df_add at
                // 3.35e-15) the two representations genuinely diverge at the extreme end of mode 0's
                // range: measured mean Δ 0.6564, 0.610% of pixels differing by >2 iterations. That is
                // precision, not a defect, and the independent arbiter says so — the bignum oracle
                // PASSES for BOTH modes at these exact depths (20 agree / 5 boundary / 0 mismatch at
                // 9.3e27× mode 0 and at 1.3e28× mode 2), and its per-sample tolerance is
                // |Δsmooth| < 0.75, which 0.66 sits inside.
                //
                // So the strict bound stays on the card it was calibrated on, exactly as the goldens
                // and bench-matrix already do. The >2-iteration FRACTION does NOT loosen: that is the
                // "no pixel is grossly wrong" gate and it held at 0.61% against a 2% allowance.
                // ⚠The real discriminator is EFT PRESERVATION, not vendor identity — an absent
                // BLESSED-GPU.txt therefore means STRICT, so a missing file can never silently
                // loosen a gate (the same safe direction the golden comparison takes).
                let cross_gpu = std::fs::read_to_string(
                    anchored("validation/golden").join("BLESSED-GPU.txt"),
                )
                .ok()
                .map(|g| g.trim().to_string())
                .is_some_and(|g| g != self.gpu_name.trim());
                let mean_cap = if cross_gpu { 1.0 } else { 0.5 };
                if let (Some(aa), Some(bb)) = (render(&a), render(&b)) {
                    let (mean, frac) = compare(&aa, &bb);
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Numeric",
                        name: "floatexp vs df32 at the df32 ceiling".into(),
                        params: format!(
                            "corpus loc 07, 9.3e27×, selector chose mode {selector_mode}{}",
                            if cross_gpu { " (cross-GPU: mean bound 1.0)" } else { "" }
                        ),
                        result: format!("mean Δ={mean:.4} iter, >2iter {:.3}%", frac * 100.0),
                        threshold: if cross_gpu {
                            "selector picks mode 0, mean<1.0 (cross-GPU), <2% differ"
                        } else {
                            "selector picks mode 0, mean<0.5, <2% differ"
                        },
                        pass: selector_mode == 0 && mean < mean_cap && frac < 0.02,
                    });
                }
            }

            // (C3) Pin the crossover itself. The two checks above are only meaningful if the
            // selector still routes 9.3e27× to df32 and has switched to floatexp by 1.3e28×;
            // if the threshold ever moves, they would silently start comparing mode 2 to mode 2
            // (vacuously identical) and the oracle entries would stop covering both sides.
            {
                // Report the ACTUAL magnifications, not the 3/4-scaled arguments — reading a
                // nominal "7e27" in a crossover check is what made this land on the wrong side
                // of the switch the first time.
                let magof = |m: f64| -> f64 {
                    let mut v = Viewport::new(N as f64, N as f64);
                    v.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * m));
                    v.magnification()
                };
                let below = make(self, CRX, CRY, 7.0e27).mode; // → 9.3e27×
                let above = make(self, CRX, CRY, 1.0e28).mode; // → 1.3e28×
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Numeric",
                    name: "df32→floatexp crossover brackets ~1e28×".into(),
                    params: format!(
                        "{:.2e}× vs {:.2e}×, threshold {:.0e}×",
                        magof(7.0e27),
                        magof(1.0e28),
                        crate::PERT_FE_THRESHOLD
                    ),
                    result: format!("mode {below} below, mode {above} above"),
                    threshold: "0 below, 2 above",
                    pass: below == 0 && above == 2,
                });
            }

            // (D3) Series approximation — at deep zoom (mode 2) the order-3 polynomial seed
            // must (a) actually engage (skip > 0) and (b) reproduce the full-iteration render.
            // Compare an SA-on render to the same view with the skip forced to 0. BLA is forced
            // OFF for the request build: since the SA⊂BLA gate, SA is only computed when no BLA
            // tree is built — this exercises the SA path exactly where it still runs (BLA off /
            // unavailable), rather than passing vacuously with skip 0.
            {
                let (saved_bla, saved_sa) = (self.render_cfg.use_bla, self.render_cfg.series_approx);
                let saved_method = self.coloring.color_method;
                self.render_cfg.use_bla = false;
                self.render_cfg.series_approx = true;
                // A blocking coloring method (stripe/TIA/trap/decomposition) gates SA off; the session
                // may have loaded one, so pin Smooth for the SA build (as the Multibrot SA checks do).
                self.coloring.color_method = crate::ColorMethod::Smooth;
                let on = make(self, NX, NY, 1.0e30);
                self.render_cfg.use_bla = saved_bla;
                self.render_cfg.series_approx = saved_sa;
                self.coloring.color_method = saved_method;
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
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "Series approximation",
                            name: "SA seed vs full iteration @1e30×".into(),
                            params: format!("Mandelbrot, 1e30×, skip {skip} of {} iter", on.max_iter),
                            result: format!("max Δ {maxd:.4} smooth iter"),
                            threshold: "skip>0 and max Δ < 0.05",
                            pass: maxd < 0.05,
                        });
                    }
                    _ => push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Series approximation",
                        name: "SA seed vs full iteration @1e30×".into(),
                        params: "Mandelbrot, 1e30×".into(),
                        result: if skip == 0 { "SA did not engage (skip=0)".into() } else { "render failed".into() },
                        threshold: "skip>0 and max Δ < 0.05",
                        pass: false,
                    }),
                }
            }

            // (D3g) The SA⊂BLA gate — when a BLA tree is built for a floatexp Mandelbrot view,
            // the request must carry NO series seed (SA's bignum coefficient pass is the dominant
            // deep build cost, ~9.4 s at 1e1105×, for a skip BLA already provides) and an engaged
            // BLA. Guards against silently re-paying the SA build wherever BLA is active.
            {
                let (saved_bla, saved_sa) = (self.render_cfg.use_bla, self.render_cfg.series_approx);
                self.render_cfg.use_bla = true;
                self.render_cfg.series_approx = true;
                let req = make(self, NX, NY, 1.0e30);
                self.render_cfg.use_bla = saved_bla;
                self.render_cfg.series_approx = saved_sa;
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Series approximation",
                    name: "SA gated off when BLA active @1e30×".into(),
                    params: format!("Mandelbrot mode {}, SA toggle on, BLA on", req.mode),
                    result: format!("sa_skip {}, bla_on {}", req.sa_skip, req.bla_on),
                    threshold: "sa_skip == 0 and bla_on == 1",
                    pass: req.sa_skip == 0 && req.bla_on == 1,
                });
            }

            // (D3b) Series approximation on the df32 path (mode 0) — same engage + fidelity
            // check at a depth the depth-selector renders with mode 0 (< 1e28×). The seed is
            // computed in floatexp then collapsed to the absolute df32 δ this path carries.
            {
                let (saved_sa, saved_method) = (self.render_cfg.series_approx, self.coloring.color_method);
                self.render_cfg.series_approx = true;
                // A blocking coloring method (stripe/TIA/trap/decomposition) gates SA off; pin Smooth.
                self.coloring.color_method = crate::ColorMethod::Smooth;
                let on = make(self, NX, NY, 1.0e20);
                self.render_cfg.series_approx = saved_sa;
                self.coloring.color_method = saved_method;
                let mut off = on.clone();
                off.sa_skip = 0;
                let (skip, mode) = (on.sa_skip, on.mode);
                match (render(&on), render(&off)) {
                    (Some(a), Some(b)) if skip > 0 && mode == 0 => {
                        let mut maxd = 0.0f64;
                        for i in 0..(a.len() / 4) {
                            let (ra, rb) = (a[i * 4], b[i * 4]);
                            if ra >= 0.0 && rb >= 0.0 {
                                maxd = maxd.max((ra - rb).abs() as f64);
                            }
                        }
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "Series approximation",
                            name: "SA seed vs full iteration @1e20× (mode 0)".into(),
                            params: format!("Mandelbrot, 1e20×, mode {mode}, skip {skip} of {} iter", on.max_iter),
                            result: format!("max Δ {maxd:.4} smooth iter"),
                            threshold: "mode 0, skip>0, max Δ < 0.05",
                            pass: maxd < 0.05,
                        });
                    }
                    _ => push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Series approximation",
                        name: "SA seed vs full iteration @1e20× (mode 0)".into(),
                        params: format!("Mandelbrot, 1e20×, mode {mode}"),
                        result: if skip == 0 { "SA did not engage (skip=0)".into() } else { "render failed / wrong mode".into() },
                        threshold: "mode 0, skip>0, max Δ < 0.05",
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
                let base = make(self, SX, SY, mag); // mode 0, best_reference
                let prec = fractadyne_core::precision_for_magnification(mag);
                let cxb = fractadyne_core::parse_bf(SX).unwrap();
                let cyb = fractadyne_core::parse_bf(SY).unwrap();
                // Actual complex span (shallow here): span_mantissa × 2^delta_exp.
                let span = base.span_mantissa.x * 2f64.powi(base.delta_exp);
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
                    let mut r = base.clone();
                    r.orbit = orbit;
                    r.orbit_len = len;
                    r.ref_offset = fractadyne_gpu::RefOffset::from_df32(dx, dy);
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
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
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

                // (D2b) GPU glitch DETECTION (Pauldelbrot, `glitch_on`). A far-offset reference
                // makes pixels satisfy |z|² < tol²·|Z|² → flagged with the -2 sentinel; the auto
                // reference flags far fewer. Detection responding to reference quality is the
                // prerequisite for multi-reference correction (phase 2 GPU port).
                let mut g_auto = base.clone();
                g_auto.glitch_on = 1;
                let mut g_bad = with_ref(0.45 * span, 0.35 * span);
                g_bad.glitch_on = 1;
                if let (Some(pa), Some(pb)) = (render(&g_auto), render(&g_bad)) {
                    let flagged = |px: &[f32]| px.iter().step_by(4).filter(|&&r| r < -1.5).count();
                    let (auto_gl, bad_gl) = (flagged(&pa), flagged(&pb));
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Glitch",
                        name: "glitch detection responds to reference quality".into(),
                        params: "seahorse, 1e8×, auto vs far-offset reference".into(),
                        result: format!("auto-ref flagged {auto_gl}, far-ref flagged {bad_gl}"),
                        threshold: "detection fires (>0) and far-offset flags ≥ auto",
                        pass: auto_gl > 0 && bad_gl >= auto_gl,
                    });
                }

                // (D2b2) Glitch detection SURVIVES CHUNKING (beta.124). The corrector's base pass
                // runs `glitch_on = 1` through `render_iter_tiled`, which since beta.124 splits
                // each tile's iterate into wall-priced iteration windows — so a glitched pixel
                // now settles as `ST_GLITCHED` mid-progression, is passed through every later
                // window, and is turned back into the -2 sentinel by `fs_resolve`. Compared
                // against the trusted single-dispatch `render_iter` on the SAME far-offset
                // reference, which flags plenty of pixels: bit-identity alone could pass
                // vacuously if detection silently stopped firing in BOTH, so the flagged count
                // is asserted non-zero too.
                {
                    let mut g = with_ref(0.45 * span, 0.35 * span);
                    g.glitch_on = 1;
                    let tiled = fractadyne_gpu::render_iter_tiled(device, queue, &g, 2_000_000_000, None, None)
                        .map_err(|e| eprintln!("[selftest] GPU ERROR (render_iter_tiled): {e}"))
                        .ok();
                    if let (Some(single), Some(t)) = (render(&g), &tiled) {
                        let flagged = |px: &[f32]| px.iter().step_by(4).filter(|&&r| r < -1.5).count();
                        let (gs, gt) = (flagged(&single), flagged(&t.pixels));
                        let diffs = single
                            .iter()
                            .zip(&t.pixels)
                            .filter(|(a, b)| a.to_bits() != b.to_bits())
                            .count();
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "Glitch",
                            name: "chunked glitch detection is bit-identical".into(),
                            params: "seahorse, 1e8×, far-offset ref, tiled+chunked vs single".into(),
                            result: format!("{diffs} texels differ; flagged single {gs}, chunked {gt}"),
                            threshold: "0 texels differ, and detection actually fired (>0)",
                            pass: diffs == 0 && gs > 0 && gt == gs,
                        });
                    }
                }

                // (D2b3) The scattered-GATHER pass is BIT-IDENTICAL to the full-frame pass.
                // This is the check the whole gather idea rests on: `fs_iterate_gather` renders a
                // tiny texture whose texel i takes its pixel coordinate from a list instead of from
                // the rasterizer, and shares the iteration kernel (`iterate_at`) verbatim with
                // `fs_iterate`. If that were even one ULP off, glitch correction would silently
                // start adopting different pixels than the renderer it is correcting. The sample is
                // deliberately SCATTERED (a coprime stride walks the whole frame, plus all four
                // corners and the last pixel) because a contiguous one would not exercise the
                // indirection at all, and the run asserts that the sample actually spans the
                // outcome classes — a comparison over 500 identical interior pixels would pass
                // vacuously. Repeated with a work budget small enough to force ~32 batches, which
                // is what exercises the batch loop's last-row padding and its scatter back.
                {
                    let mut g = with_ref(0.45 * span, 0.35 * span);
                    g.glitch_on = 1;
                    let full = fractadyne_gpu::render_iter_tiled(device, queue, &g, 2_000_000_000, None, None)
                        .map_err(|e| eprintln!("[selftest] GPU ERROR (render_iter_tiled): {e}"))
                        .ok();
                    let nn = N as usize;
                    let npx = nn * nn;
                    let mut idx: Vec<usize> = vec![0, nn - 1, npx - nn, npx - 1];
                    idx.extend((0..500).map(|k: usize| (k * 7919) % npx));
                    let coords: Vec<[u32; 2]> =
                        idx.iter().map(|&i| [(i % nn) as u32, (i / nn) as u32]).collect();
                    let gather = |budget| {
                        fractadyne_gpu::render_iter_gather(device, queue, &g, &coords, budget, None)
                            .map_err(|e| eprintln!("[selftest] GPU ERROR (render_iter_gather): {e}"))
                            .ok()
                    };
                    if let (Some(f), Some(one), Some(many)) = (&full, gather(2_000_000_000), gather(1)) {
                        let diff = |g: &fractadyne_gpu::GatherResult| {
                            idx.iter().enumerate().filter(|(k, &i)| {
                                (0..4).any(|c| {
                                    f.pixels[i * 4 + c].to_bits() != g.pixels[k * 4 + c].to_bits()
                                })
                            }).count()
                        };
                        let (d1, dn) = (diff(&one), diff(&many));
                        let class = |p: f32| if p < -1.5 { 0 } else if p < 0.0 { 1 } else { 2 };
                        let mut seen = [0usize; 3];
                        for &i in &idx {
                            seen[class(f.pixels[i * 4])] += 1;
                        }
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "Glitch",
                            name: "scattered-gather iterate is bit-identical".into(),
                            params: format!(
                                "seahorse, 1e8×, far-offset ref, {} scattered px, 1 batch vs {} batches",
                                idx.len(),
                                idx.len().div_ceil(16),
                            ),
                            result: format!(
                                "{d1} differ (1 batch), {dn} differ (batched); sample: {} glitched, {} interior, {} escaped",
                                seen[0], seen[1], seen[2]
                            ),
                            threshold: "0 texels differ either way, and the sample spans glitched + escaped",
                            pass: d1 == 0 && dn == 0 && seen[0] > 0 && seen[2] > 0,
                        });
                    }
                }

                // (D2c) End-to-end multi-reference CORRECTION. Starting from the auto reference
                // (which flags a few glitches here), the corrector drops in extra references and
                // must resolve every flagged pixel — residual glitches → 0.
                {
                    let mut vp = Viewport::new(N as f64, N as f64);
                    vp.center_x = fractadyne_core::parse_bf(SX).unwrap();
                    vp.center_y = fractadyne_core::parse_bf(SY).unwrap();
                    vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mag));
                    vp.precision = fractadyne_core::precision_for_magnification(mag);
                    if let Some(ci) = self.render_corrected_iter(
                        device, queue, &vp, false, N, N, 40, None,
                        crate::render::CorrectionBudget::UNBOUNDED,
                    ) {
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "Glitch",
                            name: "multi-reference correction resolves glitches".into(),
                            params: "seahorse, 1e8×, auto seed + correction".into(),
                            result: format!("{} references, {} residual glitches", ci.refs_used, ci.residual),
                            threshold: "0 residual glitches",
                            pass: ci.residual == 0,
                        });

                        // (D2e) The correction CUT is deterministic. The old wall-clock deadline
                        // cut the loop wherever machine load happened to put it — two runs of the
                        // same binary at e4000 differed by 3–101 bytes. Bounded in WORK, the same
                        // request must cut at the same pass every run: size the CPU budget to admit
                        // the front build plus exactly one correction pass, run twice, and require
                        // bit-identical buffers AND that the budget actually bound (fewer refs than
                        // the unbounded run above — a cut that never engages proves nothing).
                        let ask = self.export_eff_iter(&vp, false);
                        let price = crate::render::glitch_build_price(ask, vp.precision);
                        let bounded = crate::render::CorrectionBudget {
                            cpu_bits2: price.saturating_mul(5) / 2,
                            gpu_steps: u64::MAX,
                        };
                        let a = self.render_corrected_iter(
                            device, queue, &vp, false, N, N, 40, None, bounded,
                        );
                        let b = self.render_corrected_iter(
                            device, queue, &vp, false, N, N, 40, None, bounded,
                        );
                        let (pass, result) = match (&a, &b) {
                            (Some(x), Some(y)) => {
                                let identical = x.pixels.len() == y.pixels.len()
                                    && x.pixels
                                        .iter()
                                        .zip(&y.pixels)
                                        .all(|(p, q)| p.to_bits() == q.to_bits());
                                let bound = x.refs_used < ci.refs_used;
                                (
                                    identical && x.refs_used == y.refs_used && bound,
                                    format!(
                                        "run A {} refs, run B {} refs, identical {identical}, bound engaged {bound} (unbounded used {})",
                                        x.refs_used, y.refs_used, ci.refs_used
                                    ),
                                )
                            }
                            _ => (false, "a bounded run returned None".into()),
                        };
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "Glitch",
                            name: "work-boxed correction cuts deterministically".into(),
                            params: "seahorse, 1e8×, CPU budget = front + 1 pass, run twice".into(),
                            result,
                            threshold: "bit-identical buffers, same refs, budget engaged",
                            pass,
                        });
                    }

                    // (D2d) Corrected → colored export. The merged buffer colors into a finite,
                    // structured image (both interior and exterior present), and matches a normal
                    // export on the smooth region (correction only touches the rare glitched px).
                    // Pin Smooth so this check is session-independent (a blocking coloring method left
                    // by the session — e.g. stripe — otherwise makes a render return None and the
                    // whole check silently skip, dropping the total check count).
                    let d2d_method = self.coloring.color_method;
                    self.coloring.color_method = crate::ColorMethod::Smooth;
                    if let (Some(cor), Some(plain)) = (
                        self.render_export_corrected(
                            device, queue, &vp, false, N, N, None,
                            crate::render::CorrectionBudget::UNBOUNDED,
                        ),
                        render(&make(self, SX, SY, mag)),
                    ) {
                        let n = (N * N) as usize;
                        let finite = cor.pixels.iter().all(|v| v.is_finite());
                        // `plain` is the raw iteration buffer (r<0 = interior); the corrected image
                        // is colored RGBA. Compare structure: both should have interior + exterior.
                        let cor_dark = (0..n).any(|i| cor.pixels[i * 4] < 0.05);
                        let cor_bright = (0..n).any(|i| cor.pixels[i * 4] > 0.2);
                        let interior_plain = (0..n).filter(|&i| plain[i * 4] < 0.0).count();
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "Glitch",
                            name: "corrected buffer colors to a valid image".into(),
                            params: "seahorse, 1e8×, render_export_corrected".into(),
                            result: format!(
                                "finite {finite}, dark {cor_dark}, bright {cor_bright}, plain interior px {interior_plain}"
                            ),
                            threshold: "finite + structured (interior & exterior)",
                            pass: finite && cor_dark && cor_bright,
                        });
                    }
                    self.coloring.color_method = d2d_method;
                }
            }

            // (E) Real-axis symmetry + interior/exterior presence + finiteness @home.
            let req = make(self, "-0.5", "0.0", 1.0);
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
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Invariant",
                    name: "real-axis mirror symmetry".into(),
                    params: "home view (-0.5, 0)".into(),
                    result: format!("mean Δ={mean:.5} iter"),
                    threshold: "mean<0.05",
                    pass: mean < 0.05,
                });
                let interior = px.iter().step_by(4).any(|&r| r < 0.0);
                let exterior = px.iter().step_by(4).any(|&r| r >= 0.0);
                push_check(&mut checks, &mut last_check_t, SelfCheck {
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
        if want("symmetry") {
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
            // (pixel i, pixel j, size n) -> the symmetric pixel expected to match.
            type SymmetryMap = fn(usize, usize, usize) -> (usize, usize);
            let cases: &[(FractalKind, &str, SymmetryMap)] = &[
                (FractalKind::Multibrot3, "Multibrot-3 180° rotation", |i, j, n| (n - 1 - i, n - 1 - j)),
                (FractalKind::Tricorn, "Tricorn real-axis reflection", |i, j, n| (i, n - 1 - j)),
                (FractalKind::Celtic, "Celtic real-axis reflection", |i, j, n| (i, n - 1 - j)),
            ];
            for &(fractal, label, partner) in cases {
                self.fractal = fractal;
                self.julia_mode = false;
                self.coloring.color_method = crate::ColorMethod::Smooth;
                self.coloring.use_custom_palette = false;
                self.render_cfg.auto_iter = false;
                self.render_cfg.max_iter = 1500;
                let mut vp = Viewport::new(N as f64, N as f64);
                vp.center_x = fractadyne_core::BigFloat::from_f64(0.0, 64);
                vp.center_y = fractadyne_core::BigFloat::from_f64(0.0, 64);
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / N as f64); // span 3, origin-centered
                vp.precision = 64;
                let mut req = self.current_export_request_for(&vp, false);
                req.width = N;
                req.height = N;
                req.ss = 1;
                let px = st_render_iter(device, queue, &req);
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
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
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
        if want("abs-family") {
            self.julia_mode = false;
            self.coloring.color_method = crate::ColorMethod::Smooth;
            self.coloring.use_custom_palette = false;
            self.render_cfg.auto_iter = false;
            self.render_cfg.max_iter = 2000;
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
                    st_render_iter(device, queue, &pert),
                    st_render_iter(device, queue, &direct),
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
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
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
                    st_render_iter(device, queue, &m0),
                    st_render_iter(device, queue, &m2),
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
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
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
                if let Some(px) = st_render_iter(device, queue, &deep) {
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
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
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

            // ---- Phoenix deep zoom: two-term perturbation vs the trusted direct path ----
            // Phoenix (z' = z² + c − 0.5·z_{n-1}) carries a two-term δz recurrence with a rebased
            // previous term (rebase-to-0 works because the reference's z_{-1} = 0). Validate mode 0
            // (df32) and mode 2 (floatexp) against direct on the smooth region (steep/filament
            // pixels skipped) at 1e5× — deep enough to exercise δz rebasing, shallow enough that
            // direct df32 is still accurate. mode 2 is depth-independent, so it's checked here too.
            {
                self.fractal = FractalKind::Phoenix;
                let mag = 1.0e5;
                let mut vp = Viewport::new(N as f64, N as f64);
                vp.center_x = fractadyne_core::parse_bf("0.0").unwrap();
                vp.center_y = fractadyne_core::parse_bf("0.40").unwrap();
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mag));
                vp.precision = fractadyne_core::precision_for_magnification(mag);
                let mut base = self.current_export_request_for(&vp, false);
                base.width = N;
                base.height = N;
                base.ss = 1;
                let mut direct = base.clone();
                direct.mode = 1;
                let mut m0 = base.clone();
                m0.mode = 0;
                let mut m2 = base.clone();
                m2.mode = 2;
                let ren = |req: &fractadyne_gpu::ExportRequest| st_render_iter(device, queue, req);
                let cmp = |a: &[f32], b: &[f32]| -> (f64, f64, u64) {
                    let (mut sum, mut n, mut big) = (0.0f64, 0u64, 0u64);
                    for j in 0..nn {
                        for i in 0..nn {
                            if steep(a, i, j) {
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
                    (mean, frac, n)
                };
                if let (Some(d), Some(p0), Some(p2)) = (ren(&direct), ren(&m0), ren(&m2)) {
                    let (mean0, frac0, n0) = cmp(&p0, &d);
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Phoenix deep zoom",
                        name: "Phoenix perturbation vs direct".into(),
                        params: format!("1e5×, mode 0 vs 1, n={n0}"),
                        result: format!("mean Δ={mean0:.4} iter, >2iter {:.3}%", frac0 * 100.0),
                        threshold: "mean<0.5, <2% differ, n>0",
                        pass: n0 > 0 && mean0 < 0.5 && frac0 < 0.02,
                    });
                    let (mean2, frac2, n2) = cmp(&p2, &p0);
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Phoenix deep zoom",
                        name: "Phoenix floatexp vs df32".into(),
                        params: format!("1e5×, mode 2 vs 0, n={n2}"),
                        result: format!("mean Δ={mean2:.4} iter, >2iter {:.3}%", frac2 * 100.0),
                        threshold: "mean<0.5, <2% differ, n>0",
                        pass: n2 > 0 && mean2 < 0.5 && frac2 < 0.02,
                    });
                }
            }
        }

        // ---- series approximation engages for the Multibrot families ----
        // The order-3 coefficient recurrence for z^d is validated exactly in fractadyne-core;
        // here we confirm the app actually selects SA for these formulas (skip > 0) and the
        // GPU render is finite and bit-consistent with an SA-off render (the seed shader code
        // is formula-agnostic, already validated for Mandelbrot in modes 0 and 2).
        if want("multibrot-sa") {
            self.julia_mode = false;
            self.coloring.color_method = crate::ColorMethod::Smooth;
            self.coloring.use_custom_palette = false;
            self.render_cfg.auto_iter = false;
            self.render_cfg.max_iter = 4000;
            // Hermeticity (the v0.2.1 lesson, second occurrence): this check READS
            // `render_cfg.series_approx` via `current_export_request_for` — a session saved with
            // SA disabled (e.g. by tooling that stages the session file) made all three checks
            // report "SA did not engage" with no code defect present. Pin it like `color_method`.
            self.render_cfg.series_approx = true;
            for (fractal, cx, cy) in [
                (FractalKind::Multibrot3, "0.2", "0.1"),
                (FractalKind::Multibrot4, "0.2", "0.1"),
                (FractalKind::Multibrot5, "0.2", "0.1"),
            ] {
                self.fractal = fractal;
                let mut vp = Viewport::new(N as f64, N as f64);
                vp.center_x = fractadyne_core::parse_bf(cx).unwrap();
                vp.center_y = fractadyne_core::parse_bf(cy).unwrap();
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * 1.0e7));
                vp.precision = fractadyne_core::precision_for_magnification(1.0e7);
                let mut on = self.current_export_request_for(&vp, false);
                on.width = N;
                on.height = N;
                on.ss = 1;
                let mut off = on.clone();
                off.sa_skip = 0;
                let (skip, mode) = (on.sa_skip, on.mode);
                match (
                    st_render_iter(device, queue, &on),
                    st_render_iter(device, queue, &off),
                ) {
                    (Some(a), Some(b)) if skip > 0 && mode == 0 => {
                        let finite = a.iter().step_by(4).all(|v| v.is_finite());
                        let (mut mism, mut esc) = (0u64, 0u64);
                        for i in 0..(a.len() / 4) {
                            let (ra, rb) = (a[i * 4], b[i * 4]);
                            let (ia, ib) = (ra < 0.0, rb < 0.0);
                            if ia != ib {
                                mism += 1;
                            } else if !ia {
                                esc += 1;
                                if (ra - rb).abs() > 0.5 {
                                    mism += 1;
                                }
                            }
                        }
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "Series approximation",
                            name: format!("{} SA engages + matches SA-off @1e7×", fractal.name()),
                            params: format!("mode {mode}, skip {skip} of {} iter, {esc} escaped", on.max_iter),
                            result: format!("{mism} mismatch, {}", if finite { "finite" } else { "NON-FINITE!" }),
                            threshold: "skip>0, mode 0, finite, 0 mismatch",
                            pass: finite && mism == 0,
                        });
                    }
                    _ => push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Series approximation",
                        name: format!("{} SA engages + matches SA-off @1e7×", fractal.name()),
                        params: format!("mode {mode}, skip {skip}"),
                        result: if skip == 0 { "SA did not engage (skip=0)".into() } else { "render failed / wrong mode".into() },
                        threshold: "skip>0, mode 0, finite, 0 mismatch",
                        pass: false,
                    }),
                }
            }
        }

        // ---- BLA (bilinear approximation): GPU render must match the non-BLA render ----
        // Enable BLA on a deep floatexp (mode 2) Mandelbrot view and compare against the same
        // request with BLA off (SA also off, to isolate BLA). The multi-level skip + escape
        // revert must reproduce the full perturbation everywhere except rare boundary pixels.
        if want("bla") {
            self.fractal = FractalKind::Mandelbrot;
            self.julia_mode = false;
            self.coloring.color_method = crate::ColorMethod::Smooth;
            self.coloring.use_custom_palette = false;
            self.render_cfg.auto_iter = false;
            self.render_cfg.max_iter = 5000;
            self.render_cfg.series_approx = false; // isolate BLA
            self.render_cfg.use_bla = true;
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
            // Deep 38-digit minibrot nucleus (mode 2, BLA-eligible).
            const NX: &str = "-0.74364388703715887077806454349323251348";
            const NY: &str = "0.131825904205312292821097354874199108694";
            let mut vp = Viewport::new(N as f64, N as f64);
            vp.center_x = fractadyne_core::parse_bf(NX).unwrap();
            vp.center_y = fractadyne_core::parse_bf(NY).unwrap();
            vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * 1.0e30));
            vp.precision = fractadyne_core::precision_for_magnification(1.0e30);
            let mut on = self.current_export_request_for(&vp, false);
            on.width = N;
            on.height = N;
            on.ss = 1;
            let mut off = on.clone();
            off.bla_on = 0;
            let (bla_on, mode) = (on.bla_on, on.mode);
            match (
                st_render_iter(device, queue, &on),
                st_render_iter(device, queue, &off),
            ) {
                (Some(a), Some(b)) if bla_on == 1 && mode == 2 => {
                    // Compare all non-boundary pixels (b = non-BLA is the ground-truth mask):
                    // interior↔interior and escaped-with-|Δ|<0.5 agree; anything else mismatches.
                    let (mut mism, mut esc, mut interior) = (0u64, 0u64, 0u64);
                    for j in 0..nn {
                        for i in 0..nn {
                            if steep(&b, i, j) {
                                continue;
                            }
                            let k = j * nn + i;
                            let (ra, rb) = (a[k * 4], b[k * 4]);
                            match (ra < 0.0, rb < 0.0) {
                                (true, true) => interior += 1,
                                (false, false) => {
                                    esc += 1;
                                    if (ra - rb).abs() > 0.5 {
                                        mism += 1;
                                    }
                                }
                                _ => mism += 1,
                            }
                        }
                    }
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "BLA",
                        name: "BLA render == non-BLA @1e30×".into(),
                        params: format!("Mandelbrot mode 2, bla_on {bla_on}, {esc} escaped / {interior} interior"),
                        result: format!("{mism} mismatch"),
                        threshold: "bla engaged, 0 mismatch",
                        pass: mism == 0 && (esc + interior) > 0,
                    });
                }
                _ => push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "BLA",
                    name: "BLA render == non-BLA @1e30×".into(),
                    params: format!("bla_on {bla_on}, mode {mode}"),
                    result: if bla_on == 0 { "BLA did not engage".into() } else { "render failed / wrong mode".into() },
                    threshold: "bla engaged, mean<0.5, <2% differ, n>0",
                    pass: false,
                }),
            }
            // Escape-path coverage: the nucleus view above is all-interior, so it never exercises
            // BLA's escape-overshoot revert. A deep BOUNDARY view (many escapers) does — BLA on
            // must still match BLA off on every escaped pixel.
            {
                let mut vp = Viewport::new(N as f64, N as f64);
                vp.center_x = fractadyne_core::parse_bf(SX).unwrap();
                vp.center_y = fractadyne_core::parse_bf(SY).unwrap();
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * 1.0e30));
                vp.precision = fractadyne_core::precision_for_magnification(1.0e30);
                let mut on = self.current_export_request_for(&vp, false);
                on.width = N;
                on.height = N;
                on.ss = 1;
                let mut off = on.clone();
                off.bla_on = 0;
                let (bon, mode) = (on.bla_on, on.mode);
                if let (Some(a), Some(b)) = (
                    st_render_iter(device, queue, &on),
                    st_render_iter(device, queue, &off),
                ) {
                    let (mut mism, mut esc) = (0u64, 0u64);
                    for j in 0..nn {
                        for i in 0..nn {
                            if steep(&b, i, j) {
                                continue;
                            }
                            let k = j * nn + i;
                            let (ra, rb) = (a[k * 4], b[k * 4]);
                            match (ra < 0.0, rb < 0.0) {
                                (false, false) => {
                                    esc += 1;
                                    if (ra - rb).abs() > 0.5 {
                                        mism += 1;
                                    }
                                }
                                (true, true) => {}
                                _ => mism += 1,
                            }
                        }
                    }
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "BLA",
                        name: "BLA escape path == non-BLA @1e30× (boundary)".into(),
                        params: format!("seahorse boundary, mode {mode}, bla_on {bon}, {esc} escaped"),
                        result: format!("{mism} mismatch"),
                        threshold: "bla engaged, escapers>100, 0 mismatch",
                        pass: bon == 1 && mode == 2 && esc > 100 && mism == 0,
                    });
                }
            }
            self.render_cfg.use_bla = false;
            self.render_cfg.series_approx = true;
        }

        // ---- aux⇄BLA fold (Phase 2): aux coloring must match with BLA skipping on vs off ----
        // The Phase-2 shader folds each skipped run's aux aggregate. render_iter forces aux off, so
        // compare the COLORED render (render_export) with BLA on vs off for the BLA-folded methods —
        // point orbit-trap (default min-|z| aggregate), triangle-inequality (cmag/power), and stripe
        // average (Σ stripe terms). They must agree except at the rare BLA escape-boundary pixels the
        // smooth BLA test tolerates. Stripe is tested at a NON-DEFAULT frequency so the BLA aggregate
        // must have been built with the live frequency (not the old hardcoded 1.0) to match.
        if want("aux-bla") {
            self.fractal = FractalKind::Mandelbrot;
            self.julia_mode = false;
            self.coloring.color_method = crate::ColorMethod::Smooth; // Smooth so the BLA tree builds
            self.coloring.use_custom_palette = false;
            let saved_stripe_freq = self.coloring.stripe_freq;
            let saved_trap_type = self.coloring.trap_type;
            self.render_cfg.auto_iter = false;
            self.render_cfg.max_iter = 5000;
            self.render_cfg.series_approx = false; // isolate BLA
            self.render_cfg.use_bla = true;
            let nn = N as usize;
            let mut vp = Viewport::new(N as f64, N as f64);
            vp.center_x = fractadyne_core::parse_bf(SX).unwrap();
            vp.center_y = fractadyne_core::parse_bf(SY).unwrap();
            vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * 1.0e30));
            vp.precision = fractadyne_core::precision_for_magnification(1.0e30);
            let prog = std::sync::atomic::AtomicU32::new(0);
            let cancel = std::sync::atomic::AtomicBool::new(false);
            let rex = |req: &fractadyne_gpu::ExportRequest| {
                match fractadyne_gpu::render_export(device, queue, req, &prog, &cancel) {
                    Ok(r) => Some(r.pixels),
                    Err(e) => {
                        eprintln!("[selftest] GPU ERROR (render_export): {e}");
                        None
                    }
                }
            };
            // Each case sets the live aux params BEFORE building the request, so the BLA aggregate is
            // baked for exactly the method / trap-type / frequency the render then reads (all three
            // trap types exercised; stripe at a non-default 5.0). Rebuilt per case for that reason.
            for (m, tt, freq, label) in [
                (3u32, 0u32, 1.0f32, "orbit-trap-point"),
                (3u32, 1u32, 1.0, "orbit-trap-cross"),
                (3u32, 2u32, 1.0, "orbit-trap-circle"),
                (2u32, 0u32, 1.0, "triangle-ineq"),
                (1u32, 0u32, 5.0, "stripe"),
            ] {
                self.coloring.trap_type = crate::TrapType::ALL[tt as usize];
                self.coloring.stripe_freq = freq;
                let mut on = self.current_export_request_for(&vp, false);
                on.width = N;
                on.height = N;
                on.ss = 1;
                on.color_method = m;
                on.trap_type = tt;
                on.sa_skip = 0; // isolate BLA (no SA prefix yet)
                let (bla_on, mode) = (on.bla_on, on.mode);
                let mut off = on.clone();
                off.bla_on = 0;
                match (rex(&on), rex(&off)) {
                    (Some(a), Some(b)) if bla_on == 1 && mode == 2 && a.len() == b.len() => {
                        let (mut maxd, mut nd) = (0.0f32, 0u64);
                        for k in 0..a.len() {
                            let d = (a[k] - b[k]).abs();
                            maxd = maxd.max(d);
                            if d > 0.02 {
                                nd += 1;
                            }
                        }
                        let chans = (nn * nn * 4) as u64;
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "BLA",
                            name: format!("{label}: BLA-fold == non-BLA @1e30×"),
                            params: format!("bla_on {bla_on}, maxΔ {maxd:.4}"),
                            result: format!("{nd}/{chans} channels >2%"),
                            threshold: "bla engaged, maxΔ<0.1, <1% differ",
                            pass: maxd < 0.1 && nd < chans / 100,
                        });
                    }
                    _ => push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "BLA",
                        name: format!("{label}: BLA-fold == non-BLA @1e30×"),
                        params: format!("bla_on {bla_on}, mode {mode}"),
                        result: "render failed / BLA not engaged".into(),
                        threshold: "bla engaged",
                        pass: false,
                    }),
                }
            }
            self.render_cfg.use_bla = false;
            self.render_cfg.series_approx = true;
            self.coloring.color_method = crate::ColorMethod::Smooth;
            self.coloring.stripe_freq = saved_stripe_freq;
            self.coloring.trap_type = saved_trap_type;
        }

        // ---- invariance & consistency (Phase 3) — oracle-free, targets the tier crossovers ----
        if want("consistency") {
            self.fractal = FractalKind::Mandelbrot;
            self.julia_mode = false;
            self.coloring.color_method = crate::ColorMethod::Smooth;
            self.coloring.use_custom_palette = false;
            self.render_cfg.auto_iter = false;
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
                st_render_iter(device, queue, &req)
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
                push_check(&mut checks, &mut last_check_t, SelfCheck {
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
                push_check(&mut checks, &mut last_check_t, SelfCheck {
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
                push_check(&mut checks, &mut last_check_t, SelfCheck {
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
                push_check(&mut checks, &mut last_check_t, SelfCheck {
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
                push_check(&mut checks, &mut last_check_t, SelfCheck {
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
                push_check(&mut checks, &mut last_check_t, SelfCheck {
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
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Derivative",
                    name: "DE lower bound (Koebe ¼)".into(),
                    params: format!("seahorse, 1e6×, {checked} sampled exterior px"),
                    result: format!("{koebe_viol} disks contain interior"),
                    threshold: "0",
                    pass: checked > 0 && koebe_viol == 0,
                });
            }
        }

        // ---- GPU event counters: execution proof for the deep-zoom paths (D2.8/F4) ----
        // A silently-dead shader branch renders byte-identically (the WGSL NaN-marker
        // lesson from v0.2.6): these checks assert that the code paths the deep views
        // claim to exercise actually FIRED, via the D3.3 shader counters.
        if want("counters") {
            self.fractal = FractalKind::Mandelbrot;
            self.julia_mode = false;
            // Deterministic setup — proven necessary: as first written these checks PASSED
            // in the full suite but FAILED under --selftest-filter, because they leaned on
            // reference/config state leaked from earlier groups (F13 in the flesh). Pin the
            // reference length explicitly: auto_iter=false + max_iter=N builds the orbit to
            // exactly N, independent of what ran before.
            // (a) BLA skips at 1e30x: a 4000-iteration reference reaches the cap without
            // escaping (partial), so its BLA tree is KEPT (an escaped reference drops it),
            // and the render must take multi-step skips.
            self.render_cfg.use_bla = true;
            self.render_cfg.series_approx = true;
            self.render_cfg.auto_iter = false;
            self.render_cfg.max_iter = 4000;
            {
                let mut req = make(self, SX, SY, 1.0e30);
                req.mode = 2;
                match fractadyne_gpu::render_iter(device, queue, &req) {
                    Ok(r) => {
                        let c = &r.counters;
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "Counters",
                            name: "BLA skips fire @1e30× (execution proof)".into(),
                            params: format!("bla_on={} iter={}", req.bla_on, req.max_iter),
                            result: format!(
                                "bla_skip={} rebase={} maxiter_px={}",
                                c[fractadyne_gpu::CTR_BLA_SKIP],
                                c[fractadyne_gpu::CTR_REBASE],
                                c[fractadyne_gpu::CTR_MAXITER],
                            ),
                            threshold: "bla_on and bla_skip > 0",
                            pass: req.bla_on == 1 && c[fractadyne_gpu::CTR_BLA_SKIP] > 0,
                        });
                    }
                    Err(e) => {
                        eprintln!("[selftest] GPU ERROR (render_iter): {e}");
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "Counters",
                            name: "BLA skips fire @1e30× (execution proof)".into(),
                            params: String::new(),
                            result: format!("GPU error: {e}"),
                            threshold: "render succeeds",
                            pass: false,
                        });
                    }
                }
            }
            // (b) Extended-range orbit samples + rebases on a dip-carrying orbit — the
            // machinery of the v0.2.6 fix (validation corpus 14, ~1.2e148x). The reference
            // dips to ~1e-71 every ~4383 iterations, so a 5000-sample orbit CONTAINS a dip
            // (ext decodes must fire), and rendering 20000 iterations against it forces
            // ~4 end-of-orbit wraps per pixel — real rebases through the extended-range
            // compare (deterministic; dip-triggered |z|<|dz| rebases only occur at much
            // higher iteration counts where dz has grown to dip scale). SA and BLA are off
            // to isolate the plain perturbation recurrence.
            self.render_cfg.use_bla = false;
            self.render_cfg.series_approx = false;
            self.render_cfg.auto_iter = false;
            self.render_cfg.max_iter = 5000;
            {
                const C14X: &str = "-0.3158354656090698908113251908145989842764104941136552011217533774266655202463327904910559501703762081531934176786217990113494418705307973163264218287292234362119";
                const C14Y: &str = "0.6533553743954627788289923830392687875350977003260517837408108019649970888461393846103786781501651324966145060684808980380361143296058258024081840162818693511972";
                let mut req = make(self, C14X, C14Y, 1.0e148);
                req.mode = 2;
                req.max_iter = 20_000;
                match fractadyne_gpu::render_iter(device, queue, &req) {
                    Ok(r) => {
                        let c = &r.counters;
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "Counters",
                            name: "extended-range samples + rebases fire on a dip orbit @1.2e148×".into(),
                            params: format!("orbit_len={} render_iter=20000 sa=off bla=off", req.orbit_len),
                            result: format!(
                                "ext={} rebase={} maxiter_px={}",
                                c[fractadyne_gpu::CTR_EXT_SAMPLE],
                                c[fractadyne_gpu::CTR_REBASE],
                                c[fractadyne_gpu::CTR_MAXITER],
                            ),
                            threshold: "ext > 0 and rebase > 0",
                            pass: c[fractadyne_gpu::CTR_EXT_SAMPLE] > 0
                                && c[fractadyne_gpu::CTR_REBASE] > 0,
                        });
                    }
                    Err(e) => {
                        eprintln!("[selftest] GPU ERROR (render_iter): {e}");
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "Counters",
                            name: "extended-range samples + rebases fire on a dip orbit @1.2e148×".into(),
                            params: String::new(),
                            result: format!("GPU error: {e}"),
                            threshold: "render succeeds",
                            pass: false,
                        });
                    }
                }
            }
            self.render_cfg.use_bla = true;
            self.render_cfg.series_approx = true;
            self.render_cfg.auto_iter = true;
            self.render_cfg.max_iter = 4000;
        }

        // ---- catalog: independently verifiable locations (Phase 6.1 / 6.6) ----
        // Loads validation/catalog.toml (committed, human-readable) and checks the build
        // against each known answer, so external validation is one command. A missing file
        // is a FAILED check, not a silent skip (D2.6/F12): the check count must not vary
        // with the working directory.
        let catalog_path = anchored("validation/catalog.toml");
        // A read FAILURE (not just not-found) is also a FAILED check, not a silent skip
        // (D2.6/F12): on Windows a sharing violation / ACL denial makes exists() pass while
        // the read errors — the category must not vanish and let the suite report OK.
        let catalog_text = if want("catalog") {
            match std::fs::read_to_string(&catalog_path) {
                Ok(t) => Some(t),
                Err(e) => {
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Catalog",
                        name: "load validation/catalog.toml".into(),
                        params: format!("{}", catalog_path.display()),
                        result: format!(
                            "{e} (run from the repo root, or keep validation/ next to the exe tree)"
                        ),
                        threshold: "file present and readable",
                        pass: false,
                    });
                    None
                }
            }
        } else {
            None
        };
        if let Some(text) = catalog_text {
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
                        match fractadyne_core::find_nucleus(&[sx, sy], e.zoom.log2(), formula, 100_000) {
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
                                push_check(&mut checks, &mut last_check_t, SelfCheck {
                                    category: "Catalog",
                                    name: e.name.clone(),
                                    params: format!("zoom {:.0e}", e.zoom),
                                    result: detail,
                                    threshold: "period + nucleus",
                                    pass,
                                });
                            }
                            None => push_check(&mut checks, &mut last_check_t, SelfCheck {
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
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "Catalog",
                            name: e.name.clone(),
                            params: format!("interior expected {}", e.interior),
                            result: format!("oracle says interior={oracle_interior}"),
                            threshold: "matches catalog",
                            pass: oracle_interior == e.interior,
                        });
                    }
                }
                Err(err) => push_check(&mut checks, &mut last_check_t, SelfCheck {
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
        // ---- adaptive iteration budget: reach at a known-starved location ----
        // The 3.3e61× Misiurewicz three-spar renders ENTIRELY interior at the depth-scaled cap —
        // a black screen — and needs several times that budget before any pixel escapes. This
        // pins the two facts the live probe depends on: the base cap really is starved here (so
        // the check can't silently pass on an easy view), and the budget the probe can reach in
        // `ITER_STALL_LIMIT` raises really does resolve it. Lower the limit or the step and this
        // fails, which is the point: the controller reverted too early and the view went black.

        // ---- live settled RESOLUTION invariant ----
        // ⭐The one assertion that would have caught the whole beta.40/41/47 family in one line.
        // Each of those bugs ended the same way — a settled view pinned at a fraction of its panel
        // and upscaled — and none of them could be seen by a golden, because a golden renders
        // OFFLINE at a requested size and so has no panel to be a fraction of:
        //   · beta.40  the df32 path had no cost bound at all, then got one that over-shrank
        //   · beta.41  the budget could never bootstrap on a static view → 205×162 upscaled
        //   · beta.47  no `TIMESTAMP_QUERY` → budget stuck at the bootstrap → 504×396, forever
        //
        // The invariant, both halves: an UNMEASURED budget MAY bound the first dispatch, and must
        // NOT bind the settled resolution. So assert both directions — a suite that only checked
        // the second would pass if the bound were simply deleted.
        if want("live-res") {
            // The reported A2 geometry: a maximized panel at an explicit, ordinary iteration
            // count. 1445·1134·2000 ≈ 3.3e9 nominal steps against the 4.0e8 bootstrap, so a
            // single dispatch cannot hold it and the tiled settle has to.
            const PANEL: [u32; 2] = [1445, 1134];
            const ITER: u32 = 2000;
            let (saved_iter, saved_auto, saved_aa) = (
                self.render_cfg.max_iter,
                self.render_cfg.auto_iter,
                self.render_cfg.aa,
            );
            self.render_cfg.max_iter = ITER;
            self.render_cfg.auto_iter = false;
            self.render_cfg.aa = 1;
            self.viewport.set_size(PANEL[0] as f64, PANEL[1] as f64);
            self.viewport.set_center_log2mag(
                fractadyne_core::parse_bf(SX).unwrap(),
                fractadyne_core::parse_bf(SY).unwrap(),
                (1.0e9f64).log2(),
            );
            // A never-measured budget, which is the state a device without TIMESTAMP_QUERY is
            // stuck in permanently and every view is in for its first frames.
            self.perf.fe_budget = [0, 0];
            self.perf.fe_budget_ok = [false, false];
            self.perf.tile_state = [None, None];
            self.perf.view_gen = [0, 0];
            self.allow_tiled_settle = true;

            // The real loop advances `frame_idx` every frame, and the tiled settle depends on it:
            // `next_settle_tile`'s turn token (`tile_turn == frame_idx`) and its "is the other
            // view busy" guards all compare against it, so a harness that leaves it pinned gets
            // exactly ONE tile and then holds forever — which looks identical to the bug under
            // test. Start clear of the `frame_idx - interact_frame[other] <= 1` window too.
            self.perf.frame_idx = 100;
            let build = |app: &mut Self| -> (u32, u32, u32, u32) {
                app.perf.frame_idx += 1;
                let center_bf = [app.viewport.center_x.clone(), app.viewport.center_y.clone()];
                let center = app.viewport.center_f64();
                let span = app.viewport.complex_span_fe();
                let mag = app.viewport.magnification();
                let l2 = app.viewport.log2_magnification();
                let pr = app.build_params(
                    center_bf, center, span, mag, l2, app.fractal, false, ITER, false, 1, PANEL,
                    0, None,
                );
                // A chunked frame's dispatch runs only its iteration RANGE — that is the count
                // the budget bound applies to (the full ask is honoured across frames).
                let disp_iter = pr.chunk_range.map(|[s, e]| e.saturating_sub(s).max(1)).unwrap_or(ITER);
                (pr.resolution[0], pr.resolution[1], pr.ss, disp_iter)
            };

            // WARM UP until the reference orbit exists. `build_params` starts the bignum build
            // off-thread and installs it on a later call, and until it lands `will_reproject` is
            // true (no `ref_pt`), which forces `can_tile` off — a harness that skipped this would
            // measure the reproject path and report the very collapse it is meant to detect.
            let mut warm = 0;
            while self.ref_cache[0].ref_pt.is_none() && warm < 400 {
                let _ = build(self);
                std::thread::sleep(std::time::Duration::from_millis(10));
                warm += 1;
            }
            let have_ref = self.ref_cache[0].ref_pt.is_some();
            // Re-arm from a clean grid so the measurement below starts at the ARM frame.
            self.perf.tile_state = [None, None];
            self.perf.tile_pending = [false, false];

            // Frame 1 ARMS the grid: it is the coarse single-dispatch full frame, so it MUST be
            // bounded — this is the half that keeps an unknown GPU safe.
            let (arm_w, arm_h, arm_ss, arm_iter) = build(self);
            let arm_steps =
                (arm_w as u64) * (arm_h as u64) * (arm_ss as u64).pow(2) * arm_iter as u64;
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Live budget",
                name: "unmeasured budget bounds the FIRST dispatch".into(),
                params: format!("{}×{} panel, {ITER} iter, fe_budget=0", PANEL[0], PANEL[1]),
                result: format!("arm frame {arm_w}×{arm_h} ss{arm_ss} = {:.3e} steps", arm_steps as f64),
                threshold: "≤ crate::tunables::cost().tdr_bootstrap_steps",
                pass: arm_steps <= crate::tunables::cost().tdr_bootstrap_steps,
            });

            // Subsequent settled frames run the grid and must reach (near-)native resolution.
            // A handful of frames is plenty: the grid geometry is fixed on its first tile.
            // The grid needs one frame per tile; this geometry is ~15 tiles, so give it room.
            let mut best_w = arm_w;
            for _ in 0..40 {
                let (w, _, _, _) = build(self);
                best_w = best_w.max(w);
            }
            let frac = best_w as f64 / PANEL[0] as f64;
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Live budget",
                name: "unmeasured budget does NOT bind settled resolution".into(),
                params: format!("{}×{} panel, {ITER} iter, fe_budget=0", PANEL[0], PANEL[1]),
                result: format!("settled width {best_w}/{} ({:.0}% of panel)", PANEL[0], frac * 100.0),
                threshold: "≥90% of panel width",
                pass: have_ref && frac >= 0.90,
            });
            if !have_ref {
                eprintln!(
                    "[selftest] live-res: reference orbit never installed — the check above is                      reporting the reproject path, not the tiled settle."
                );
            }

            // ---- the suite must be measuring the SHIPPED numbers ----
            // `--set` can move any of the frame-cost tunables for a run, which is exactly what a
            // field diagnosis wants and exactly what a verdict must not be quoted from: every
            // threshold in this suite, every golden and every blessed baseline assumes the
            // defaults. A run with an override is measuring a build nobody ships, so say so here
            // rather than letting the summary read as a clean bill of health.
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Live budget",
                name: "tunables are stock (no --set overrides)".into(),
                params: "the suite's thresholds, goldens and baselines all assume the defaults"
                    .into(),
                result: crate::tunables::status_line(),
                threshold: "stock",
                pass: crate::tunables::is_stock(),
            });

            // ---- and the suite must say WHICH arithmetic backend produced these numbers ----
            // ⭐Every golden, every corpus render and every blessed baseline is the output of one
            // bignum backend. A pass quoted without naming it becomes unattributable the moment a
            // second backend exists. Sourced from `observed_backends`, which a *finished orbit*
            // sets — not a flag, an env var or a config field, any of which can disagree with what
            // actually ran. `MIXED` fails: one suite must not be half-credited to two backends.
            let backends = fractadyne_core::observed_backends();
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Live budget",
                name: "one bignum backend produced this run".into(),
                params: "goldens and baselines are the output of a single arithmetic backend"
                    .into(),
                result: fractadyne_core::backend_status_line(),
                threshold: "exactly one",
                pass: backends.len() == 1,
            });

            // ---- the tile ALLOWANCE must not bind the settled resolution either ----
            // ⭐The same invariant as above, at the count where it actually broke. The settled
            // resolution is `tdr_steps × settle_max_tiles ÷ iterations`, so the ALLOWANCE is a
            // resolution ceiling; while it was a two-valued switch on `budget >=
            // EXPLICIT_DISPATCH_CAP` it was also a per-step RATE test, and a deep view whose
            // converged budget landed just under 2e10 got sixteen dispatches forever — 85×49 out
            // of a 1920×1102 panel, the 2026-08-14 field report. The count above (2000) cannot see
            // it: sixteen tiles cover that panel at 2000 iterations with room to spare.
            //
            // The budget here is INJECTED rather than measured, and re-injected every frame,
            // because the property under test is exactly "a converged budget of this size must not
            // bind resolution" — a real measurement would make the test depend on the day's GPU
            // rate, which is the very thing that must not decide whether the view is sharp. 1.666e10
            // is the value the field frame reported (85×49 × 4,000,000 = one dispatch's worth).
            {
                const DEEP: [u32; 2] = [1920, 1102];
                const DEEP_ITER: u32 = 4_000_000;
                const FIELD_BUDGET: u64 = 16_660_000_000;
                self.render_cfg.max_iter = 2000; // cheap warm-up reference; raised below
                self.render_cfg.auto_iter = false;
                self.viewport.set_size(DEEP[0] as f64, DEEP[1] as f64);
                self.viewport.set_center_log2mag(
                    fractadyne_core::parse_bf(SX).unwrap(),
                    fractadyne_core::parse_bf(SY).unwrap(),
                    100.0, // 1.3e30× — floatexp (mode 2); chunk-eligible since the slice-3 flip
                );
                self.ref_cache[0].ref_pt = None;
                self.perf.tile_state = [None, None];
                self.perf.fe_budget = [0, 0];
                self.perf.fe_budget_ok = [false, false];
                let deep_build = |app: &mut Self, iter: u32| -> ([u32; 2], bool) {
                    app.perf.frame_idx += 1;
                    let center_bf = [app.viewport.center_x.clone(), app.viewport.center_y.clone()];
                    let center = app.viewport.center_f64();
                    let span = app.viewport.complex_span_fe();
                    let mag = app.viewport.magnification();
                    let l2 = app.viewport.log2_magnification();
                    let pr = app.build_params(
                        center_bf, center, span, mag, l2, app.fractal, false, iter, false, 1, DEEP,
                        0, None,
                    );
                    (pr.resolution, pr.display_hold)
                };
                // Warm up until the orbit exists — `will_reproject` forces `can_tile` off without
                // one, and a harness that skipped this would measure the reproject path.
                let mut warm = 0;
                while self.ref_cache[0].ref_pt.is_none() && warm < 400 {
                    let _ = deep_build(self, 2000);
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    warm += 1;
                }
                let deep_ref = self.ref_cache[0].ref_pt.is_some();
                self.render_cfg.max_iter = DEEP_ITER;
                self.perf.tile_state = [None, None];
                self.perf.tile_pending = [false, false];
                let mut deep_best = 0u32;
                for _ in 0..60 {
                    // Re-injected per frame: a reference install landing mid-loop derates the
                    // budget and clears `ok`, which is a different (real) behaviour and not what
                    // this check is about.
                    self.perf.fe_budget = [FIELD_BUDGET, FIELD_BUDGET];
                    self.perf.fe_budget_ok = [true, true];
                    deep_best = deep_best.max(deep_build(self, DEEP_ITER).0[0]);
                }
                let deep_frac = deep_best as f64 / DEEP[0] as f64;
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Live budget",
                    name: "tile allowance does NOT bind settled resolution".into(),
                    params: format!(
                        "{}×{} panel, {DEEP_ITER} iter @1.3e30×, converged budget {:.3e}",
                        DEEP[0], DEEP[1], FIELD_BUDGET as f64
                    ),
                    result: format!(
                        "settled width {deep_best}/{} ({:.0}% of panel)",
                        DEEP[0],
                        deep_frac * 100.0
                    ),
                    threshold: "≥90% of panel width",
                    pass: deep_ref && deep_frac >= 0.90,
                });
                if !deep_ref {
                    eprintln!(
                        "[selftest] live-res deep: reference orbit never installed — the check \
                         above is reporting the reproject path, not the tiled settle."
                    );
                }

                // ---- …and the finished composite must REVEAL ----
                // ⭐"It does the computation but doesn't update the image; it shows up as soon as I
                // resize slightly" (field report, 2026-08-15). Present-gating ("prefer detail")
                // serves a SNAPSHOT of the last complete frame while a grid composes underneath,
                // and drops the gate when nothing is composing — but `composing` counted
                // `tile.is_some()`, and `next_settle_tile` REPEATS the final rect forever once the
                // grid is done (deliberately: the GPU dedupes it, so a finished view costs
                // nothing). So the gate never dropped, the display kept serving the coarse ARM
                // frame, and the sharp composite sat in the texture unseen. A window nudge
                // "fixed" it because interaction breaks the gate and shows the texture directly.
                // The chunked path's own term (`e > s`) already excludes its completed tail; this
                // is the same care, missing on the tile path.
                //
                // The grid here is 540 tiles at one per frame, so give it room and then assert the
                // gate is DOWN — and that it was UP first, or a gate that never engages at all
                // would pass this vacuously.
                let saved_detail = self.render_cfg.prefer_detail;
                self.render_cfg.prefer_detail = true;
                self.perf.tile_state = [None, None];
                self.perf.tile_pending = [false, false];
                self.perf.hold_active = [false, false];
                let mut held_any = false;
                let mut held_last = true;
                let mut frames = 0;
                for i in 0..900 {
                    self.perf.fe_budget = [FIELD_BUDGET, FIELD_BUDGET];
                    self.perf.fe_budget_ok = [true, true];
                    let (_, hold) = deep_build(self, DEEP_ITER);
                    held_any |= hold;
                    held_last = hold;
                    frames = i + 1;
                    // Stop as soon as the grid has completed AND the gate has dropped; if it never
                    // drops, the loop runs out and the check fails with `held_last` still true.
                    if held_any && !hold && !self.perf.tile_pending[0] {
                        break;
                    }
                }
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Live budget",
                    name: "a completed tiled settle REVEALS (present gate drops)".into(),
                    params: format!(
                        "{}×{} panel, {DEEP_ITER} iter @1.3e30×, prefer detail on",
                        DEEP[0], DEEP[1]
                    ),
                    result: format!(
                        "gate engaged={held_any}, still holding after {frames} frames={held_last}"
                    ),
                    threshold: "engages, then drops once the grid completes",
                    pass: deep_ref && held_any && !held_last,
                });
                self.render_cfg.prefer_detail = saved_detail;

                // ---- LIVE CHUNK SIZING ACROSS A GROWING BAND LEDGER ----
                // Companion to the "allowance up, budget CLIMBING" check below, covering the axis
                // that one does not: a CONVERGED settled walk long enough for `chunk_band_license`
                // to ratchet. That ledger is what sized the 2026-08-22 field device loss
                // (crash-1787401025-0) — 1024 iterations from `bands[0]`, against a budget
                // authorising 24,457 — and its growth is a ×2 fast lane, so the invariant has to
                // hold not on one frame but along the whole ladder.
                //
                // ⚠`chunk_fe_ok` is FORCED, not probed. It is false on a device that granted only
                // 48 color-attachment bytes, and this view is mode 2 — so on such a machine the
                // mode-2 sizing arithmetic would never be reached and the check would pass without
                // testing it. `build_params` computes a dispatch rather than issuing one, so
                // forcing the capability gates the arithmetic without needing the hardware.
                //
                // THE INVARIANT: a settled chunked pass stays inside ONE dispatch budget
                // (`tdr_steps`), never the multi-tile allowance. Sizing a pass from the allowance
                // dispatched single passes worth sixteen budgets and lost a device
                // (crash-1787158916-0, 9.600e11-step passes against a 6.000e10 budget).
                //
                // ⚠ANTI-VACUITY: the run must actually have been chunk-governed, and the ladder
                // must actually have MOVED — a walk pinned at the 256 floor would satisfy the
                // budget bound trivially while testing none of the growth this exists to cover.
                let saved_chunk = (self.perf.chunk_ok, self.perf.chunk_fe_ok);
                let saved_method = self.coloring.color_method;
                self.perf.chunk_ok = true;
                self.perf.chunk_fe_ok = true;
                // Aux colorings are out of chunk scope by design; pin a non-aux method so the case
                // exercises the path regardless of what the loaded session left selected.
                self.coloring.color_method = crate::ColorMethod::Smooth;
                self.perf.chunk_sig = [(0, 0, [0, 0], 0); 2];
                self.perf.chunk_cursor = [0, 0];
                self.perf.chunk_bands = [[0; crate::tunables::CHUNK_BANDS], [0; crate::tunables::CHUNK_BANDS]];
                self.perf.chunk_inflight = [None, None];
                self.perf.chunk_pass_dt = [0.0, 0.0];
                self.perf.tile_state = [None, None];
                self.perf.tile_pending = [false, false];
                let mut governed_frames = 0u32;
                let mut worst_pass_steps = 0u64;
                let mut biggest_step = 0u32;
                for _ in 0..24 {
                    self.perf.fe_budget = [FIELD_BUDGET, FIELD_BUDGET];
                    self.perf.fe_budget_ok = [true, true];
                    let before = self.perf.chunk_cursor[0];
                    let _ = deep_build(self, DEEP_ITER);
                    if self.perf.chunk_governed[0] {
                        governed_frames += 1;
                        worst_pass_steps = worst_pass_steps.max(self.perf.fe_steps_last[0]);
                        biggest_step =
                            biggest_step.max(self.perf.chunk_cursor[0].saturating_sub(before));
                    }
                }
                let within_one_budget = worst_pass_steps <= FIELD_BUDGET;
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Live budget",
                    name: "a growing chunk band license never outgrows one dispatch budget".into(),
                    params: format!(
                        "{}×{} panel, {DEEP_ITER} iter @1.3e30×, converged budget {:.3e},                          24 settled frames, chunk_fe_ok forced",
                        DEEP[0], DEEP[1], FIELD_BUDGET as f64
                    ),
                    result: format!(
                        "{governed_frames}/24 frames chunk-governed, license grew to {biggest_step}                          iters, worst pass {:.3e} steps ({:.2}× budget)",
                        worst_pass_steps as f64,
                        worst_pass_steps as f64 / FIELD_BUDGET as f64
                    ),
                    threshold: "governed, ladder moved past the 256 floor, every pass ≤ one budget",
                    pass: deep_ref && governed_frames > 0 && biggest_step > 256 && within_one_budget,
                });
                if governed_frames == 0 || biggest_step <= 256 {
                    eprintln!(
                        "[selftest] live chunk sizing: governed={governed_frames}/24, largest                          window={biggest_step} — the check above tested less than it claims.                          Either `chunk_over` is unreachable here or the band ledger never left its                          floor; fix the setup rather than the threshold."
                    );
                }
                self.coloring.color_method = saved_method;
                self.perf.chunk_ok = saved_chunk.0;
                self.perf.chunk_fe_ok = saved_chunk.1;

                // ---- one chunked pass = ONE dispatch budget, even with the tile allowance up ----
                // ⭐Field device loss 2026-08-19 17:01 UTC (crash-1787158916-0, RTX 3080,
                // beta.106, 165.7 s uptime: a minibrot interior at an explicit 4,000,000). On a
                // SETTLED frame `tiling` is true, so `tdr_allowed = tdr_steps × max_tiles` — and
                // the chunk step was sized from that ALLOWANCE, submitting single passes worth
                // exactly SIXTEEN dispatch budgets (the log's ratios: 9.600e11 vs 6.000e10,
                // 5.070e11 vs 3.169e10, 2.224e11 vs 1.390e10 — 16.0× each; measured 1136 ms and
                // 912 ms lethal-band frames, and the emergency retreat could not help because the
                // NEXT pass was again 16× the retreated budget). The allowance exists for TILES —
                // many bounded dispatches per frame-equivalent; a chunk pass is ONE submission and
                // must be sized from ONE budget. `bla_skip` collapsing to 0 at the minibrot
                // interior made nominal cost real cost at exactly the wrong moment — the regime
                // iteration chunking exists for, met with a 16× dispatch.
                {
                    self.perf.chunk_sig[0] = (0, 0, [0, 0], 0);
                    self.perf.chunk_cursor = [0, 0];
                    self.perf.chunk_idx = [0, 0];
                    self.perf.tile_state = [None, None];
                    self.perf.tile_pending = [false, false];
                    // The bound under test: what ONE dispatch may cost on this frame — the
                    // injected converged budget through the same clamps `build_params` applies
                    // (explicit ask ⇒ the explicit ceiling).
                    let budget_now =
                        crate::render::budget_base(FIELD_BUDGET, self.perf.bootstrap_steps(0))
                            .min(crate::tunables::cost().explicit_steps_ceil);
                    // Several frames, worst pass: `tiling` (and with it the ×16 allowance) only
                    // engages once the settle grid has ARMED under a stable key, which takes a
                    // frame or two — the field session had been settled for minutes. A one-frame
                    // harness measures the pre-arm state and passes vacuously.
                    let mut disp: Option<u64> = None;
                    for _ in 0..8 {
                        self.perf.fe_budget = [FIELD_BUDGET, FIELD_BUDGET];
                        // ok=FALSE, deliberately: the field session's budget was CLIMBING (a
                        // reference-install derate plus a timestamp outage kept it unconverged),
                        // which pins the allowance at exactly TDR_MAX_TILES — the log's 16.0×
                        // ratios are this state's fingerprint. A CONVERGED allowance covers the
                        // whole need and un-chunks the frame (tiles bound it instead), so the
                        // climbing state is the only one where the chunk step can meet the
                        // allowance at all.
                        self.perf.fe_budget_ok = [false, false];
                        self.perf.frame_idx += 1;
                        let center_bf =
                            [self.viewport.center_x.clone(), self.viewport.center_y.clone()];
                        let center = self.viewport.center_f64();
                        let span = self.viewport.complex_span_fe();
                        let mag = self.viewport.magnification();
                        let l2 = self.viewport.log2_magnification();
                        let pr = self.build_params(
                            center_bf, center, span, mag, l2, self.fractal, false, DEEP_ITER,
                            false, 1, DEEP, 0, None,
                        );
                        let d = pr.chunk_range.map(|[s, e]| {
                            (pr.resolution[0] as u64)
                                * (pr.resolution[1] as u64)
                                * (pr.ss as u64).pow(2)
                                * (e.saturating_sub(s).max(1) as u64)
                        });
                        disp = disp.max(d);
                    }
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Live budget",
                        name: "a settled chunked pass stays inside ONE dispatch budget".into(),
                        params: format!(
                            "{}×{} panel, {DEEP_ITER} iter @1.3e30×, allowance up, budget {:.3e} CLIMBING",
                            DEEP[0], DEEP[1], budget_now as f64
                        ),
                        result: match disp {
                            Some(d) => format!(
                                "chunk pass = {:.3e} nominal ({:.2}× budget)",
                                d as f64,
                                d as f64 / budget_now as f64
                            ),
                            None => "frame did not chunk".into(),
                        },
                        threshold: "chunked, and ≤ 1× the single-dispatch budget",
                        pass: deep_ref && disp.is_some_and(|d| d <= budget_now),
                    });
                }
                self.viewport.set_size(PANEL[0] as f64, PANEL[1] as f64);
                self.perf.fe_budget = [0, 0];
                self.perf.fe_budget_ok = [false, false];
                self.perf.tile_state = [None, None];
            }

            // ✅A1's user-visible half, end to end: an EXPLICIT count must reach the shader
            // params verbatim. Direct mode at a shallow view so no reference/pixel-clamp can
            // confound the reading — the only thing between the Iterations box and the GPU here
            // is the budget formula this pins. (Before beta.53: 10,000,000 in, ~2,800 out.)
            self.render_cfg.max_iter = 10_000_000;
            self.viewport.set_center_log2mag(
                fractadyne_core::parse_bf("-0.5").unwrap(),
                fractadyne_core::parse_bf("0.0").unwrap(),
                (10.0f64).log2(),
            );
            let pr = {
                let center_bf = [self.viewport.center_x.clone(), self.viewport.center_y.clone()];
                let center = self.viewport.center_f64();
                let span = self.viewport.complex_span_fe();
                let mag = self.viewport.magnification();
                let l2 = self.viewport.log2_magnification();
                self.build_params(
                    center_bf, center, span, mag, l2, self.fractal, false, 10_000_000, false, 1,
                    PANEL, 0, None,
                )
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Live budget",
                name: "explicit iteration count honoured verbatim".into(),
                params: "auto off, 10,000,000 iterations, direct mode @10×".into(),
                result: format!("params.max_iter = {}", pr.max_iter),
                threshold: "== 10,000,000",
                pass: pr.max_iter == 10_000_000,
            });

            self.render_cfg.max_iter = saved_iter;
            self.render_cfg.auto_iter = saved_auto;
            self.render_cfg.aa = saved_aa;
            self.allow_tiled_settle = false;
            self.perf.tile_state = [None, None];
        }

        if want("iter-budget") {
            const SPAR_X: &str = "-1.0109636384562213181006238475735192993836101418531854095957676149333034794266e-1";
            const SPAR_Y: &str = "9.5628651080914147131604703998237075557983304380930462483482733394361292090816e-1";
            const SPAR_MAG: f64 = 3.2950838546818387e61;
            let log2mag = SPAR_MAG.log2();
            let base = crate::zoom_iter_cap(log2mag).max(256);
            // Where the probe can climb to within its stall allowance (step 2.5 while >90% capped).
            let mut boost = 1.0f64;
            for _ in 0..crate::ITER_STALL_LIMIT {
                boost = (boost * 2.5).min(16.0);
            }
            let reach = (((base as f64) * boost) as u32).min(crate::MAX_ITER_LIMIT);
            // Fraction of pixels with no escape value: the shader's interior/capped sentinel.
            let flat = |px: &[f32]| -> f64 {
                let n = px.len() / 4;
                if n == 0 {
                    return 1.0;
                }
                px.chunks_exact(4).filter(|c| c[0] < 0.0).count() as f64 / n as f64
            };
            // The budget has to be set BEFORE the request is built: the reference orbit is sized
            // with it, so raising `req.max_iter` afterwards leaves the orbit short and the render
            // starved for a completely different reason than the one under test.
            let (saved_iter, saved_auto) =
                (self.render_cfg.max_iter, self.render_cfg.auto_iter);
            self.render_cfg.auto_iter = false;
            let at_budget = |app: &mut Self, iter: u32| -> Option<f64> {
                app.render_cfg.max_iter = iter;
                let mut req = make(app, SPAR_X, SPAR_Y, SPAR_MAG);
                req.width = 96;
                req.height = 96;
                req.ss = 1;
                render(&req).map(|p| flat(&p))
            };
            let starved = at_budget(self, base);
            let resolved = at_budget(self, reach);
            self.render_cfg.max_iter = saved_iter;
            self.render_cfg.auto_iter = saved_auto;
            let pass = starved.is_some_and(|s| s > 0.99) && resolved.is_some_and(|r| r < 0.10);
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Iter-budget",
                name: "probe reach resolves a starved spar".into(),
                params: format!("3.3e61× three-spar, cap {base} → reach {reach}"),
                result: format!(
                    "flat {:.1}% at cap → {:.1}% at reach",
                    starved.unwrap_or(f64::NAN) * 100.0,
                    resolved.unwrap_or(f64::NAN) * 100.0
                ),
                threshold: ">99% flat at cap, <10% at reach",
                pass,
            });
        }

        // ---- Newton-Raphson zoom: atom size, framing depth, center refinement ----
        // Each check is self-validating: the size estimate is pinned by components whose width is
        // known exactly, and the center accuracy is measured against the atom it must land inside
        // — no stored reference coordinate is involved.
        if want("nr-zoom") {
            // Exact anchors. The whole set (period 1) has size 1 — the main cardioid spans
            // −0.75…0.25 — and frames at magnification 1, which IS the home view. The period-2
            // disk at c = −1 spans −1.25…−0.75, so size 1/2.
            let anchors: &[(&str, f64, f64, u32, f64)] =
                &[("whole set", 0.0, 0.0, 1, 0.0), ("period-2 disk", -1.0, 0.0, 2, -1.0)];
            let mut bad = Vec::new();
            for (name, cx, cy, period, want_l2) in anchors {
                let (bx, by) = (
                    fractadyne_core::BigFloat::from_f64(*cx, 256),
                    fractadyne_core::BigFloat::from_f64(*cy, 256),
                );
                match fractadyne_core::nucleus_size(&bx, &by, *period, 0, 256) {
                    Some(a) if (a.log2_size - want_l2).abs() < 1.0e-9 => {}
                    Some(a) => bad.push(format!("{name}: 2^{:.6} (want 2^{want_l2})", a.log2_size)),
                    None => bad.push(format!("{name}: no estimate")),
                }
            }
            // Home-view identity: framing the whole set must land exactly at magnification 1.
            let home = Self::atom_frame_log2mag(0.0);
            if home.abs() > 1.0e-9 {
                bad.push(format!("period-1 frames at 2^{home:.6}, want 2^0"));
            }
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "NR-zoom",
                name: "atom size vs exactly-known components".into(),
                params: "period 1, 2 + home-view identity".into(),
                result: if bad.is_empty() { "all exact".into() } else { bad.join("; ") },
                threshold: "exact to 1e-9",
                pass: bad.is_empty(),
            });

            // A real jump: a period-998 minibrot in deep Seahorse Valley. Its size sets a
            // destination ~9 orders of magnitude below the view that found it — the case the
            // whole feature exists for, and the case where a view-accurate center is not enough.
            let seed = [
                fractadyne_core::parse_bf(SX).unwrap(),
                fractadyne_core::parse_bf(SY).unwrap(),
            ];
            match fractadyne_core::find_nucleus(&seed, 1.0e6f64.log2(), 0, 100_000) {
                Some(n) => {
                    let size = fractadyne_core::nucleus_size(&n.cx, &n.cy, n.period, 0, 128)
                        .map(|a| a.log2_size);
                    let target = size.map(Self::atom_frame_log2mag);
                    let (residual, moved) = match target {
                        Some(t) => {
                            let prec =
                                fractadyne_core::precision_for_octaves(t.max(0.0) as u64) + 64;
                            match fractadyne_core::refine_nucleus(&n.cx, &n.cy, n.period, 0, prec) {
                                Some((rx, ry)) => (
                                    fractadyne_core::nucleus_residual_log2(
                                        &rx, &ry, n.period, 0, prec,
                                    ),
                                    (fractadyne_core::sub_f64(&rx, &n.cx, prec).powi(2)
                                        + fractadyne_core::sub_f64(&ry, &n.cy, prec).powi(2))
                                    .sqrt(),
                                ),
                                None => (None, f64::NAN),
                            }
                        }
                        None => (None, f64::NAN),
                    };
                    let sz = size.unwrap_or(f64::NAN);
                    // The center must be accurate far below the atom's own width (else the jump
                    // lands on empty space), and refinement must not wander off the atom.
                    let pass = n.period == 998
                        && (sz + 50.5).abs() < 0.5
                        && residual.is_some_and(|r| r < sz - 100.0)
                        && moved < sz.exp2() * 1.0e-3;
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "NR-zoom",
                        name: "deep minibrot: size, framing, center accuracy".into(),
                        params: format!("seahorse 1e6× → period {}", n.period),
                        result: format!(
                            "size 2^{sz:.3}, frame 2^{:.3}, center err 2^{:.0}, moved {moved:.1e}",
                            target.unwrap_or(f64::NAN),
                            residual.unwrap_or(f64::NAN)
                        ),
                        threshold: "period 998, size 2^-50.5, err < size/2^100",
                        pass,
                    });
                }
                None => push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "NR-zoom",
                    name: "deep minibrot: size, framing, center accuracy".into(),
                    params: "seahorse 1e6×".into(),
                    result: "no nucleus found".into(),
                    threshold: "period 998",
                    pass: false,
                }),
            }
        }

        // The two Misiurewicz points whose multiplier has a closed form. λ is the number that
        // says what a dive here looks like: |λ| is the zoom period, arg λ the twist per repeat.
        // c = −2 gives exactly 4, real — which is *why* the antenna tip repeats without
        // spiralling. c = i gives 4(1+i) over the {−1+i, −i} cycle: 45° of twist per period.
        if want("nr-zoom") {
            let cases: &[(&str, f64, f64, u32, u32, f64, f64)] = &[
                ("antenna tip c=-2", -2.0, 0.0, 2, 1, 2.0, 0.0),
                ("dendrite c=i", 0.0, 1.0, 2, 2, 2.5, 45.0),
            ];
            let mut bad = Vec::new();
            for (name, cx, cy, k, p, want_l2, want_deg) in cases {
                let (bx, by) = (
                    fractadyne_core::BigFloat::from_f64(*cx, 256),
                    fractadyne_core::BigFloat::from_f64(*cy, 256),
                );
                match fractadyne_core::misiurewicz_multiplier(&bx, &by, *k, *p, 0, 256) {
                    Some(l)
                        if (l.log2_abs - want_l2).abs() < 1.0e-9
                            && (l.arg.to_degrees() - want_deg).abs() < 1.0e-7 => {}
                    Some(l) => bad.push(format!(
                        "{name}: 2^{:.6} @{:.4}° (want 2^{want_l2} @{want_deg}°)",
                        l.log2_abs,
                        l.arg.to_degrees()
                    )),
                    None => bad.push(format!("{name}: no multiplier")),
                }
            }
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "NR-zoom",
                name: "Misiurewicz multiplier vs closed forms".into(),
                params: "c=-2 (lambda=4), c=i (lambda=4(1+i))".into(),
                result: if bad.is_empty() { "both exact".into() } else { bad.join("; ") },
                threshold: "exact to 1e-9",
                pass: bad.is_empty(),
            });
        }

        // ---- coordinate entry: exact rationals and complex values ----
        // The Go-to dialog's parser. Several mathematically significant landmarks are exactly
        // rational and NOT representable as terminating decimals, so accepting `p/q` is what
        // makes them enterable at all; the precision floor is what makes them usable at depth.
        if want("ref-pick") {
            // ⭐The 2:58 device-loss pick, exactly (design/reference-lifecycle.md L1). A lookahead
            // build at the grand tour's shallow-dive era scored candidates at 78 bits, where the
            // three-spar Misiurewicz centre's orbit numerically escapes in a few hundred
            // iterations (the precision CLIFF — see core test `escape_length_vs_precision`), so
            // phase 1 had no survivor and the longest-escaper fallback picked a 626-sample
            // reference. Reuse then pinned it into ~90× frame cost and a GPU device loss. The
            // cliff rescue must redo the selection at the build precision and return the CENTRE,
            // surviving the full ask.
            let cx = fractadyne_core::parse_bf(
                "-1.0109636384562213181006238475735192993836101418531854095957676926471683503366629508912671364125546238220995191834757e-1",
            )
            .expect("three-spar cx");
            let cy = fractadyne_core::parse_bf(
                "9.5628651080914147131604703998237075557983304380930462483482733212267499793490593467836270525491219946548323699651521e-1",
            )
            .expect("three-spar cy");
            let span = fractadyne_core::FloatExp::from_f64(2.4e-4); // mag ≈ 2^14, the prec-78 era
            let (pick, diag) = fractadyne_core::best_reference_diag(
                &[cx.clone(), cy.clone()],
                [span, span],
                0,     // Mandelbrot
                false, // not Julia
                [0.0, 0.0],
                13_607, // the observed lookahead ask at that era
                78,     // the observed scoring precision — deep inside the cliff
            );
            let picked_centre = pick[0] == cx && pick[1] == cy;
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "RefPick",
                name: "cliff rescue picks the surviving centre".into(),
                params: "three-spar @2^14, ask 13607, scored @78 bits".into(),
                result: format!(
                    "rescued={:?} survivors={} winner_len={} centre={picked_centre}",
                    diag.rescued, diag.survivors, diag.winner_len
                ),
                threshold: "rescued, centre picked, survives the ask",
                pass: diag.rescued.is_some() && picked_centre && diag.winner_len >= 13_607,
            });

            // The rescue must NOT fire on a healthy pick: same view scored at an adequate
            // precision picks the centre plainly (phase-1 survivor, no rescue) — this pins that
            // healthy picks stay byte-identical to the pre-rescue selection.
            let (pick2, diag2) = fractadyne_core::best_reference_diag(
                &[cx.clone(), cy.clone()],
                [span, span],
                0,
                false,
                [0.0, 0.0],
                13_607,
                286,
            );
            let picked_centre2 = pick2[0] == cx && pick2[1] == cy;
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "RefPick",
                name: "healthy pick unchanged (no rescue)".into(),
                params: "same view, scored @286 bits".into(),
                result: format!(
                    "rescued={:?} survivors={} winner_len={} centre={picked_centre2}",
                    diag2.rescued, diag2.survivors, diag2.winner_len
                ),
                threshold: "no rescue, centre picked",
                pass: diag2.rescued.is_none() && picked_centre2 && diag2.winner_len >= 13_607,
            });
        }
        if want("coords") {
            // Exact dyadic rationals — the parabolic valley entrances.
            let exact: &[(&str, f64)] =
                &[("-3/4", -0.75), ("1/4", 0.25), ("-5/4", -1.25), ("(1+i)*(1-i)/2", 1.0)];
            let mut bad = Vec::new();
            for (src, want_v) in exact {
                match fractadyne_core::parse_bf(src) {
                    Some(v) if fractadyne_core::to_f64(&v) == *want_v => {}
                    Some(v) => bad.push(format!("{src} → {}", fractadyne_core::to_f64(&v))),
                    None => bad.push(format!("{src} → rejected")),
                }
            }
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Coords",
                name: "exact rational entry".into(),
                params: format!("{} expressions", exact.len()),
                result: if bad.is_empty() { "all exact".into() } else { bad.join("; ") },
                threshold: "bit-exact",
                pass: bad.is_empty(),
            });

            // The Pythagorean boundary point (37+16i)/100 — exactly on ∂M, both coordinates
            // non-terminating in binary. Parsed at a 1e60×-class precision floor, it must agree
            // with its decimal form far past f64, or it would be unusable at the depth it's for.
            let prec = fractadyne_core::precision_for_octaves(200);
            let (rok, iok, agree) =
                match fractadyne_core::parse_complex_prec("(37+16i)/100", prec) {
                    Some((re, im)) => {
                        let dec = fractadyne_core::parse_bf_prec("0.37", prec);
                        let d = dec.map(|d| fractadyne_core::sub_f64(&re, &d, prec).abs());
                        (
                            (fractadyne_core::to_f64(&re) - 0.37).abs() < 1.0e-15,
                            (fractadyne_core::to_f64(&im) - 0.16).abs() < 1.0e-15,
                            d.unwrap_or(1.0),
                        )
                    }
                    None => (false, false, 1.0),
                };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Coords",
                name: "complex rational (37+16i)/100".into(),
                params: format!("{prec}-bit floor"),
                result: format!("re/im ok={rok}/{iok}, Δ vs decimal={agree:.1e}"),
                threshold: "both coords, Δ<1e-30",
                pass: rok && iok && agree < 1.0e-30,
            });

            // Malformed input must be REJECTED, never half-read into a wrong coordinate —
            // astro-float's own FromStr accepts "1 2" as 1, which is exactly the trap here.
            let malformed = ["1/0", "(37+16i/100", "1 2", "3/4x", "abc", "1e", "", "()"];
            let leaked: Vec<&str> = malformed
                .iter()
                .copied()
                .filter(|s| fractadyne_core::parse_bf(s).is_some())
                .collect();
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Coords",
                name: "malformed coordinates rejected".into(),
                params: format!("{} inputs", malformed.len()),
                result: if leaked.is_empty() {
                    "all rejected".into()
                } else {
                    format!("ACCEPTED: {leaked:?}")
                },
                threshold: "all rejected",
                pass: leaked.is_empty(),
            });

            // Functions, constants and powers (beta.17). Identities evaluated at the same
            // 1e60×-class floor must agree far past f64 — an expression typed at depth is
            // only useful if it carries the digits for that depth — and the refusal set
            // (branch-ambiguous powers, complex args to real functions, DoS-scale trig
            // arguments) must stay refused.
            let idents: &[(&str, &str)] = &[
                ("cos(pi/3)", "1/2"),
                ("sqrt(2)^2", "2"),
                ("root(-8,3)", "-2"),
                ("ln(e)", "1"),
                ("-1/2 + (1/4)*cos(pi/4)", "(sqrt(2)-4)/8"), // polar composition x0 + r·cos(θ)
            ];
            let mut bad = Vec::new();
            for (a, b) in idents {
                let d = match (
                    fractadyne_core::parse_bf_prec(a, prec),
                    fractadyne_core::parse_bf_prec(b, prec),
                ) {
                    (Some(x), Some(y)) => fractadyne_core::sub_f64(&x, &y, prec).abs(),
                    _ => 1.0,
                };
                if !(d < 1.0e-50) {
                    bad.push(format!("{a} vs {b}: Δ={d:.1e}"));
                }
            }
            let refuse = ["2^i", "sin(i)", "root(-4,2)", "bogus(1)", "sin(1e999)"];
            let leaked: Vec<&str> = refuse
                .iter()
                .copied()
                .filter(|s| fractadyne_core::parse_complex_prec(s, 64).is_some())
                .collect();
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Coords",
                name: "expression functions & constants".into(),
                params: format!("{} identities, {} refusals, {prec}-bit floor", idents.len(), refuse.len()),
                result: if bad.is_empty() && leaked.is_empty() {
                    "identities exact, all refused".into()
                } else {
                    format!("{}{}", bad.join("; "), if leaked.is_empty() { String::new() } else { format!(" ACCEPTED: {leaked:?}") })
                },
                threshold: "Δ<1e-50, all refused",
                pass: bad.is_empty() && leaked.is_empty(),
            });

            // Full-precision decimal round-trip must be untouched by the expression path.
            let deep = fractadyne_core::deep_roundtrip_bits(4096);
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Coords",
                name: "deep decimal round-trip intact".into(),
                params: "4096-bit coordinate".into(),
                result: format!("{deep} bits agree"),
                threshold: "≥4000 bits",
                pass: deep >= 4000,
            });
        }

        // ---- tour scripts (format v2) ----
        // The shipped tours are the app's demo reel AND its deep-render regression gauntlet, so a
        // script that no longer resolves is a shipped-content break, not a test-fixture break.
        // They're compiled in, so this checks the files in the repo rather than whatever happens
        // to sit next to an installed binary.
        if want("script") {
            const TOURS: &[(&str, &str)] = &[
                ("grand-tour", include_str!("../../../tours/grand-tour.toml")),
                ("deep-minibrot-dive", include_str!("../../../tours/deep-minibrot-dive.toml")),
                ("deep-spiral-dive", include_str!("../../../tours/deep-spiral-dive.toml")),
                ("dive-to-misiurewicz-4-1", include_str!("../../../tours/dive-to-misiurewicz-4-1.toml")),
                ("dive-to-view-3e1216", include_str!("../../../tours/dive-to-view-3e1216.toml")),
                ("julia-and-mandelbrot", include_str!("../../../tours/julia-and-mandelbrot.toml")),
            ];
            let mut bad = Vec::new();
            let mut total_s = 0.0;
            for (name, text) in TOURS {
                match crate::scripting::parse_tour_text(text) {
                    Ok(pb) if pb.total > 0.0 => total_s += pb.total,
                    Ok(_) => bad.push(format!("{name}: zero-length timeline")),
                    Err(e) => bad.push(format!("{name}: {}", e.lines().next().unwrap_or(&e))),
                }
            }
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Script",
                name: "shipped tours resolve".into(),
                params: format!("{} scripts", TOURS.len()),
                result: if bad.is_empty() {
                    format!("all resolve, {total_s:.0}s of tour")
                } else {
                    bad.join("; ")
                },
                threshold: "all resolve",
                pass: bad.is_empty(),
            });

            // Absolute timing + inheritance. The camera must SIT at keyframe 1 through its hold
            // and only then glide — cumulative `secs` timing got this right too, but only
            // absolute `t` keeps it right when a keyframe is inserted above.
            const TIMING: &str = "format_version = 2\n\
                 [[keyframe]]\nid = \"a\"\nt = 0\nre = \"-0.5\"\nim = \"0.0\"\nzoom = 1\n\
                 max_iter = 1000\nhold = 2\nease = \"linear\"\n\
                 [[keyframe]]\nid = \"b\"\nt = 6\nzoom = \"1e12\"\nmax_iter = 1000000\n\
                 ease = \"linear\"\n";
            let (mut when, mut budget) = ("script failed to resolve".to_string(), String::new());
            let mut timing_ok = false;
            if let Ok(pb) = crate::scripting::parse_tour_text(TIMING) {
                let l10 = |t: f64| pb.sample(t).logmag / std::f64::consts::LN_10;
                // t=2 is the end of the hold (still 1×); t=4 is the midpoint of the 2→6s glide.
                let (held, mid, end) = (l10(2.0), l10(4.0), l10(6.0));
                // Iteration budget interpolates geometrically: √(1e3 · 1e6) ≈ 31623 at the middle.
                let mid_iter = pb.sample(4.0).max_iter.unwrap_or(0);
                let end_iter = pb.sample(6.0).max_iter.unwrap_or(0);
                timing_ok = held.abs() < 1.0e-9
                    && (mid - 6.0).abs() < 1.0e-6
                    && (end - 12.0).abs() < 1.0e-9
                    && (mid_iter as f64 - 31_623.0).abs() < 50.0
                    && end_iter == 1_000_000
                    && (pb.total - 6.0).abs() < 1.0e-9;
                when = format!("hold@2s=1e{held:.1}, mid@4s=1e{mid:.3}, end=1e{end:.1}");
                budget = format!(", iter mid={mid_iter} end={end_iter}");
            }
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Script",
                name: "absolute times + geometric iteration ramp".into(),
                params: "hold 0–2s, glide 2–6s to 1e12×".into(),
                result: format!("{when}{budget}"),
                threshold: "still 1× at 2s, 1e6× at 4s, 31623 iters at 4s",
                pass: timing_ok,
            });

            // Export size presets must be REPRODUCIBLE in the image dialog's model, which stores a
            // width plus an aspect key and derives the height. A preset whose ratio no key
            // expresses would render at a different size than its own label claims — silently, and
            // only for that one entry. Check every row end to end: resolve its aspect, then redo
            // the dialog's own `width / ratio` rounding and demand the stated height back.
            let mut bad_sizes = Vec::new();
            for (label, w, h) in crate::STANDARD_SIZES {
                match crate::aspect_key_for(*w, *h) {
                    None => bad_sizes.push(format!("{label}: no aspect key")),
                    Some(k) => {
                        let ratio = crate::EXPORT_ASPECTS
                            .iter()
                            .find(|(kk, _)| kk == &k)
                            .map(|(_, r)| *r)
                            .unwrap_or(0.0);
                        let got = ((*w as f64) / ratio).round().max(1.0) as u32;
                        if got != *h {
                            bad_sizes.push(format!("{label}: {k} gives {got}, not {h}"));
                        }
                    }
                }
            }
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Script",
                name: "export size presets round-trip through the aspect model".into(),
                params: format!("{} presets", crate::STANDARD_SIZES.len()),
                result: if bad_sizes.is_empty() {
                    "all reproduce their stated height".into()
                } else {
                    bad_sizes.join("; ")
                },
                threshold: "every preset resolves to an aspect key that regenerates its height",
                pass: bad_sizes.is_empty(),
            });

            // Lookahead HOLD rule. `playback_ref_prefetch` builds references for depths the tour is
            // about to reach; a slot the dive has NOT yet reached must be HELD. Through beta.37 the
            // rule was read from the slot's BLA `dc_max` with the sign inverted, so an early slot
            // looked like a missed one and was dropped the moment its build landed — the queue then
            // rebuilt the same six targets every frame (~400 reference builds a second, measured in
            // a 230 s playback that lost the GPU device). Pin all three outcomes.
            let slots = [(100.5, true), (101.0, true), (101.5, false), (102.0, true)];
            let early = crate::render::prefetch_reached(100.0, &slots);
            let arrived = crate::render::prefetch_reached(100.5, &slots);
            // A dive that crossed three targets in one pump takes the DEEPEST ready one (index 3),
            // not the shallowest, and is not blocked by the still-building slot at 101.5.
            let leapt = crate::render::prefetch_reached(102.0, &slots);
            let hold_ok = early.is_none() && arrived == Some(0) && leapt == Some(3);
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Script",
                name: "lookahead holds slots the dive hasn't reached".into(),
                params: "queue at +0.5/+1.0/+1.5(building)/+2.0 octaves".into(),
                result: format!("at 100.0 → {early:?}, at 100.5 → {arrived:?}, at 102.0 → {leapt:?}"),
                threshold: "none held-back, then slot 0, then deepest ready (slot 3)",
                pass: hold_ok,
            });

            // Zoom strings past f64's ~1e308 ceiling — the reason `zoom` is a string at all.
            const DEEP: &str = "format_version = 2\n\
                 [[keyframe]]\nt = 0\nre = \"-0.5\"\nim = \"0.0\"\nzoom = \"3.0938e1216\"\n";
            let deep_l10 = crate::scripting::parse_tour_text(DEEP)
                .map(|pb| pb.sample(0.0).logmag / std::f64::consts::LN_10)
                .unwrap_or(0.0);
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Script",
                name: "deep zoom string survives f64 range".into(),
                params: "zoom = \"3.0938e1216\"".into(),
                result: format!("log10 mag = {deep_l10:.4}"),
                threshold: "1216.4904 ± 1e-3",
                pass: (deep_l10 - 1216.4904).abs() < 1.0e-3,
            });

            // Malformed scripts must be REJECTED with a diagnosis, never silently mis-played.
            // The v1 case is the sharp one: v1 and v2 share no timing keys, so a v1 file would
            // otherwise default every keyframe to t=0 at 1× and render as one still frame.
            let bad_scripts: &[(&str, &str)] = &[
                ("v1 script", "name = \"old\"\nformat_version = 1\n[[keyframe]]\nsecs = 0\nmag = 1\n"),
                ("no version", "[[keyframe]]\nt = 0\nzoom = 1\n"),
                ("no keyframes", "format_version = 2\n"),
                ("t goes backwards", "format_version = 2\n[[keyframe]]\nt = 5\nhold = 2\n[[keyframe]]\nt = 6\n"),
                ("missing t", "format_version = 2\n[[keyframe]]\nt = 0\n[[keyframe]]\nzoom = 2\n"),
                ("unknown location", "format_version = 2\n[[keyframe]]\nt = 0\nlocation = \"nope\"\n"),
                ("unknown annotation kind", "format_version = 2\n[[keyframe]]\nt = 0\n[[annotation]]\nkind = \"subtitle\"\ntext = \"x\"\n"),
                ("unanchored callout", "format_version = 2\n[[keyframe]]\nt = 0\n[[annotation]]\nkind = \"callout\"\ntext = \"x\"\n"),
                ("unparseable zoom", "format_version = 2\n[[keyframe]]\nt = 0\nzoom = \"deep\"\n"),
                ("unknown pace", "format_version = 2\n[playback]\npace = \"turbo\"\n[[keyframe]]\nt = 0\n"),
                ("half a coordinate", "format_version = 2\n[[keyframe]]\nt = 0\nre = \"-0.5\"\n"),
            ];
            let accepted: Vec<&str> = bad_scripts
                .iter()
                .filter(|(_, s)| crate::scripting::parse_tour_text(s).is_ok())
                .map(|(n, _)| *n)
                .collect();
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Script",
                name: "malformed scripts rejected".into(),
                params: format!("{} scripts", bad_scripts.len()),
                result: if accepted.is_empty() {
                    "all rejected".into()
                } else {
                    format!("ACCEPTED: {accepted:?}")
                },
                threshold: "all rejected",
                pass: accepted.is_empty(),
            });

            // Live-playback pacing: `settled` is what makes a deep tour show its destination
            // instead of walking past it, so the value has to survive the round trip.
            const PACE: &str = "format_version = 2\n[playback]\npace = \"settled\"\nsettle_timeout = 8\n\
                 [[keyframe]]\nt = 0\nre = \"-0.5\"\nim = \"0.0\"\nzoom = 1\nhold = 2\n\
                 [[keyframe]]\nt = 6\nzoom = 100\n";
            let pace_ok = crate::scripting::parse_tour_text(PACE)
                .map(|pb| {
                    // `settled` acts at holds, so the hold windows must be identifiable: inside
                    // the first keyframe's hold (0–2s) and at the final keyframe, but NOT during
                    // the 2–6s glide between them.
                    pb.pace == crate::scripting::Pace::Settled
                        && pb.holding_at(1.0).0
                        && !pb.holding_at(4.0).0
                        && pb.holding_at(6.0).0
                })
                .unwrap_or(false);
            let default_pace = crate::scripting::parse_tour_text(
                "format_version = 2\n[[keyframe]]\nt = 0\nzoom = 1\n",
            )
            .map(|pb| pb.pace == crate::scripting::Pace::Adaptive)
            .unwrap_or(false);
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Script",
                name: "playback pacing".into(),
                params: "pace = settled / default".into(),
                result: format!("settled parsed={pace_ok}, default adaptive={default_pace}"),
                threshold: "settled honored, default adaptive",
                pass: pace_ok && default_pace,
            });

            // Palettes: a keyframe naming one preset must apply that preset verbatim (a static
            // tour has to color exactly as picking the preset would), while a keyframe-to-keyframe
            // change cross-fades the two gradients — the one mechanism behind static palettes,
            // morphs, and cycling.
            const PAL: &str = "format_version = 2\n\
                 [[palette]]\nid = \"black-red\"\nstops = [{ at = 0.0, color = \"#000000\" }, \
                 { at = 1.0, color = \"#ff0000\" }]\n\
                 [[keyframe]]\nt = 0\nre = \"-0.5\"\nim = \"0.0\"\nzoom = 1\npalette = \"black-red\"\n\
                 hold = 1\nease = \"linear\"\n\
                 [[keyframe]]\nt = 3\npalette = \"Ember\"\nease = \"linear\"\n";
            let (pal_ok, pal_desc) = match crate::scripting::parse_tour_text(PAL) {
                Ok(pb) => {
                    use crate::scripting::PaletteApply as P;
                    let start = pb.sample(0.5).palette;
                    let mid = pb.sample(2.0).palette;
                    let end = pb.sample(3.0).palette;
                    let ember = fractadyne_color::PRESETS
                        .iter()
                        .position(|p| p.name.eq_ignore_ascii_case("Ember"))
                        .unwrap_or(0);
                    // Halfway through the morph, the green channel must sit strictly between the
                    // two sources at the same gradient position: the black→red ramp has none at
                    // all, Ember has plenty. Ember's value is interpolated here independently of
                    // the code under test.
                    let blended = match &mid {
                        Some(P::Stops(s)) if s.len() == fractadyne_color::MAX_STOPS => {
                            let probe = s[4]; // pos 4/7 ≈ 0.571
                            let (pos, g) = (probe[0], probe[2]);
                            let src = fractadyne_color::PRESETS[ember].stops;
                            let g_ember = src
                                .windows(2)
                                .find(|w| pos <= w[1].0)
                                .map(|w| {
                                    let f = ((pos - w[0].0) / (w[1].0 - w[0].0).max(1.0e-6)).clamp(0.0, 1.0);
                                    w[0].1[1] + (w[1].1[1] - w[0].1[1]) * f
                                })
                                .unwrap_or(0.0);
                            g > 0.05 && g < g_ember && (g - g_ember * 0.5).abs() < 0.02
                        }
                        _ => false,
                    };
                    let ok = matches!(&start, Some(P::Stops(s)) if s.len() == 2 && s[1][1] > 0.9)
                        && blended
                        && matches!(end, Some(P::Preset(i)) if i == ember);
                    (
                        ok,
                        format!(
                            "start={}, mid={}, end={}",
                            match &start { Some(P::Stops(s)) => format!("{} stops", s.len()), Some(P::Preset(i)) => format!("preset {i}"), None => "none".into() },
                            match &mid { Some(P::Stops(s)) => format!("{} blended stops", s.len()), Some(P::Preset(i)) => format!("preset {i}"), None => "none".into() },
                            match &end { Some(P::Stops(s)) => format!("{} stops", s.len()), Some(P::Preset(i)) => format!("preset {i}"), None => "none".into() },
                        ),
                    )
                }
                Err(e) => (false, e.lines().next().unwrap_or("error").to_string()),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Script",
                name: "palette definition + morph".into(),
                params: "custom stops → Ember over 2s".into(),
                result: pal_desc,
                threshold: "stops verbatim, blend at the midpoint, preset verbatim",
                pass: pal_ok,
            });

            // "Script to current view" writes a script the app then has to read back. A generator
            // emitting a shape the reader rejects is invisible until someone tries to play the
            // file, so generate one and resolve it here — at TWO depths, because the writer picks
            // the zoom's format by depth. The 2^-289 case sits in f64 range (~1e85×) and exercises
            // the finite-magnitude branch; a bare `{mag}` there prints an ~85-digit integer that
            // TOML rejects as an i64 overflow (the "zoom too large" bug). The 2^-1200 case is past
            // f64's ceiling and exercises the log10 string branch. Both must round-trip.
            for octaves in [289.0_f64, 1200.0] {
                let saved = (self.viewport.clone(), self.render_cfg.max_iter);
                self.viewport.center_x = fractadyne_core::parse_bf("-0.101096363845622131810062").unwrap();
                self.viewport.center_y = fractadyne_core::parse_bf("0.956286510809141471316047").unwrap();
                self.viewport.units_per_pixel = fractadyne_core::FloatExp::from_f64(1.0).mul_pow2(-octaves);
                self.viewport.precision = fractadyne_core::precision_for_octaves(octaves as u64);
                // The view's own depth is the target: `log2_magnification` folds in the viewport
                // size, so it is NOT simply the octaves in `units_per_pixel = 2^-octaves`.
                let want_l10 = self.viewport.log2_magnification() / std::f64::consts::LOG2_10;
                let text = self.build_dive_script("Zoom to a deep view", 60.0);
                let got = crate::scripting::parse_tour_text(&text)
                    .map(|pb| (pb.total, pb.sample(pb.total).logmag / std::f64::consts::LN_10));
                let (ok, desc) = match &got {
                    // 1.5s hold + 4s swoop + 0.5s hold + 60s dive + 2s hold = 68s.
                    Ok((total, l10)) => (
                        (total - 68.0).abs() < 0.01 && (l10 - want_l10).abs() < 0.01,
                        format!("{total:.1}s, ends 1e{l10:.1}× (view 1e{want_l10:.1}×)"),
                    ),
                    Err(e) => (false, e.lines().next().unwrap_or("error").to_string()),
                };
                (self.viewport, self.render_cfg.max_iter) = saved;
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Script",
                    name: "generated dive script round-trips".into(),
                    params: format!("\"Script to current view\" at 2^{octaves}× (1e{want_l10:.0}×)"),
                    result: desc,
                    threshold: "resolves, 68s, ends at the view's depth",
                    pass: ok,
                });
            }

            // A keyframe's centre is parsed at the DEEPEST depth the tour reaches, not its own.
            //
            // This only bites EXACT RATIONAL coordinates — a plain decimal literal is parsed from
            // its own digit count, so `1e8× keyframe, 119-digit centre` was never truncated (a
            // hypothesis this check was written to prove and promptly disproved). A rational is
            // different: it is EVALUATED, at whatever precision it is given, so `re = "-1/3"` on a
            // keyframe at 1e8× is worth ~19 digits unless the floor comes from the tour's deepest
            // view. The camera interpolates between keyframes and the lookahead builds references
            // for depths ahead of the current one, so a shallow keyframe's centre still has to
            // carry the digits its deep neighbours need.
            const RATIONAL_PREC: &str = concat!(
                "format_version = 2
",
                "[[keyframe]]
id = \"shallow\"
t = 0
re = \"-1/3\"
im = \"1/7\"
zoom = 8
",
                "[[keyframe]]
id = \"deep\"
t = 10
zoom = \"1e94\"
",
            );
            let prec = fractadyne_core::precision_for_octaves(400);
            let want = fractadyne_core::parse_bf_prec("-1/3", prec);
            let (drift, prec_ok) = match (crate::scripting::parse_tour_text(RATIONAL_PREC), want) {
                (Ok(pb), Some(w)) => {
                    // Sampled at the SHALLOW keyframe, which is where the naive parse loses digits.
                    let d = fractadyne_core::sub_f64(&pb.sample(0.0).cx, &w, prec).abs();
                    (d, d < 1.0e-110)
                }
                _ => (1.0, false),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Script",
                name: "shallow keyframe keeps deep-neighbour precision".into(),
                params: "rational centre on a 1e8× keyframe, tour reaches 1e94×".into(),
                result: format!("centre drift {drift:.2e}"),
                threshold: "< 1e-110 (the 1e94× view span is ~1e-95)",
                pass: prec_ok,
            });

            // `--segment` resolution: chapters close at the next chapter's start, and a name can
            // be given as an id, a unique prefix, or a 1-based index.
            // Asserted against the script's OWN total rather than a literal: the tour's timeline
            // is edited (the deep chapter was slowed once already), and a hardcoded duration turns
            // every such edit into a spurious failure that says nothing about segment lookup.
            let seg = crate::scripting::parse_tour_text(include_str!("../../../tours/grand-tour.toml"))
                .ok()
                .map(|pb| {
                    let by_id = pb.find_segment("gauntlet").map(|s| (s.start, s.end));
                    let by_prefix = pb.find_segment("land").map(|s| s.id.clone());
                    let by_index = pb.find_segment("1").map(|s| s.id.clone());
                    let missing = pb.find_segment("nope").is_err();
                    (by_id, by_prefix, by_index, missing, pb.total)
                });
            let (seg_ok, seg_desc) = match &seg {
                Some((Ok((start, end)), Ok(prefix), Ok(first), true, total)) => (
                    // The last chapter runs to the end of the tour; the others are ordered.
                    *start > 0.0 && *end == *total && prefix == "landmarks" && first == "whole-set",
                    format!("gauntlet {start}–{end}s of {total}s, prefix→{prefix}, #1→{first}"),
                ),
                _ => (false, "segment lookup failed".into()),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Script",
                name: "segment lookup".into(),
                params: "grand-tour chapters".into(),
                result: seg_desc,
                threshold: "id / prefix / index resolve, unknown errors",
                pass: seg_ok,
            });
        }

        // The reloadable metadata (exports / .fdn / bookmarks) must round-trip a deep view
        // exactly, flag a newer format_version (so it can't be silently mis-read), and clamp
        // hostile/garbage fields rather than ballooning precision or the iteration count.
        if want("metadata") {
            self.fractal = FractalKind::Mandelbrot;
            self.julia_mode = false;
            self.viewport.center_x = fractadyne_core::parse_bf("-0.743643887037151").unwrap();
            self.viewport.center_y = fractadyne_core::parse_bf("0.131825904205330").unwrap();
            self.viewport.units_per_pixel = fractadyne_core::FloatExp::from_f64(1.0).mul_pow2(-120.0);
            self.render_cfg.max_iter = 1234;
            self.render_cfg.auto_iter = false;
            self.render_cfg.aa = 3;
            let blob = self.view_metadata();
            // Scramble live state, then restore from the blob.
            self.render_cfg.max_iter = 7;
            self.render_cfg.aa = 1;
            self.viewport.units_per_pixel = fractadyne_core::FloatExp::from_f64(1.0);
            let rt = self.load_view_metadata(&blob);
            let cx = fractadyne_core::to_f64(&self.viewport.center_x);
            let rt_ok = rt.note().is_none()
                && self.render_cfg.max_iter == 1234
                && self.render_cfg.aa == 3
                && (self.viewport.units_per_pixel.log2() + 120.0).abs() < 1.0e-6
                && (cx + 0.743643887037151).abs() < 1.0e-12;
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "View format",
                name: "metadata round-trips a deep view".into(),
                params: "serialize → scramble → load".into(),
                result: format!(
                    "iter {} aa {} upp_log2 {:.3} cx {:.15}",
                    self.render_cfg.max_iter, self.render_cfg.aa, self.viewport.units_per_pixel.log2(), cx
                ),
                threshold: "clean load; fractal/iter/aa/zoom/center preserved",
                pass: rt_ok,
            });

            // A newer format_version must be detected (not silently consumed).
            let newer = "app=Fractadyne\nformat_version=999\ncenter_re=-0.5\ncenter_im=0\nupp_log2=-3\n";
            let nr = self.load_view_metadata(newer);
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "View format",
                name: "newer format_version flagged".into(),
                params: "format_version=999".into(),
                result: nr.note().unwrap_or_else(|| "NOT flagged".into()),
                threshold: "newer == Some(999)",
                pass: nr.newer == Some(999),
            });

            // Hostile numeric fields must be clamped (DoS / runaway work) AND reported.
            let hostile = "app=Fractadyne\nformat_version=1\ncenter_re=-0.5\ncenter_im=0\n\
                           upp_log2=-1e30\nmax_iter=4000000000\naa=9999\ncycle=inf\noffset=NaN\n\
                           bogus_field=42\n";
            let hr = self.load_view_metadata(hostile);
            let clamped = (1..=10_000_000).contains(&self.render_cfg.max_iter)
                && (1..=16).contains(&self.render_cfg.aa)
                && self.viewport.units_per_pixel.log2().is_finite()
                && self.viewport.units_per_pixel.log2() >= -3.4e7 - 1.0
                && self.coloring.cycle.is_finite()
                && self.coloring.offset.is_finite()
                && hr.clamped.len() >= 4 // zoom depth, max_iter, aa, cycle, offset
                && hr.unknown.iter().any(|u| u == "bogus_field");
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "View format",
                name: "hostile fields clamped + reported".into(),
                params: "upp_log2=-1e30, max_iter=4e9, aa=9999, cycle=inf, bogus_field".into(),
                result: format!(
                    "iter {} aa {} upp_log2 {:.2e}; clamped [{}]; unknown [{}]",
                    self.render_cfg.max_iter, self.render_cfg.aa, self.viewport.units_per_pixel.log2(),
                    hr.clamped.join(", "), hr.unknown.join(", ")
                ),
                threshold: "clamped & finite; report lists clamped + unknown",
                pass: clamped,
            });
        }

        // ---- status-bar formatters (pure, depth-aware display) ----
        if want("display") {
            // Zoom mantissa is space-grouped in 5s; exponent untouched.
            let zg = crate::group_sci_mantissa("3.38050027227e15");
            push_check(&mut checks, &mut last_check_t, SelfCheck {
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
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Formatting",
                name: "deep coordinate elides middle".into(),
                params: "32-digit center @ ~1e30×; and -0.5".into(),
                result: format!("{ds}  |  {short}"),
                threshold: "leading … frontier; short coord safe",
                pass: ok,
            });
        }

        // ---- appearance: the image actually CHANGES when a control changes ----
        // Enforces the "Coloring" block of the manual checklist (steps 48-57, 60-62): every colour
        // method and palette must produce a coherent image, and must not produce the SAME image as
        // its neighbours. A method silently falling back to another one looks perfectly fine in a
        // screenshot and is invisible to the goldens, which only ever render one method each.
        if want("appearance") {
            let (aw, ah) = (480u32, 270u32);
            // One view with interior, exterior and filament in frame, so every method has
            // something to colour. Shallow on purpose: this is about colouring, not depth.
            let render = |app: &mut Self, dev: &eframe::wgpu::Device, q: &eframe::wgpu::Queue|
             -> Option<Vec<u8>> {
                let mut vp = Viewport::new(aw as f64, ah as f64);
                vp.set_center_mag(
                    fractadyne_core::BigFloat::from_f64(-0.743_643_887_037_15, 64),
                    fractadyne_core::BigFloat::from_f64(0.131_825_904_205_31, 64),
                    2.0e3,
                );
                let req = app.current_export_request_for(&vp, false);
                let progress = std::sync::atomic::AtomicU32::new(0);
                let cancel = std::sync::atomic::AtomicBool::new(false);
                fractadyne_gpu::render_export(dev, q, &req, &progress, &cancel)
                    .ok()
                    .map(|r| fractadyne_export::to_srgb8_dithered(&r.pixels, r.width))
            };

            // Pin everything that is not the variable under test.
            self.fractal = crate::FractalKind::Mandelbrot;
            self.julia_mode = false;
            self.dual = false;
            self.render_cfg.auto_iter = false;
            self.render_cfg.max_iter = 2_000;
            self.render_cfg.aa = 1;
            self.coloring.use_custom_palette = false;
            self.coloring.use_binary = false;
            self.coloring.use_duotone = false;
            self.coloring.cycle = 0.27;
            self.coloring.offset = 0.1;
            self.anim.palette_anim = crate::PaletteAnim::Off;
            self.effects.light = false;
            self.effects.de = false;

            // --- colour methods, steps 48-53 ---
            let mut frames: Vec<(String, Vec<u8>)> = Vec::new();
            for m in crate::ColorMethod::ALL {
                self.coloring.color_method = m;
                match render(self, device, queue) {
                    Some(px) => frames.push((m.label().to_string(), px)),
                    None => push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "appearance",
                        name: format!("method renders — {}", m.label()),
                        params: format!("{aw}x{ah}"),
                        result: "render_export failed".into(),
                        threshold: "must render",
                        pass: false,
                    }),
                }
            }
            for (name, px) in &frames {
                let (sd, b) = frame::coherence(px);
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "appearance",
                    name: format!("method coherent — {name}"),
                    params: format!("{aw}x{ah}"),
                    result: format!("stddev {sd:.1}, {b} buckets"),
                    threshold: "stddev ≥ 6, ≥ 3 buckets",
                    pass: frame::coherent(px),
                });
            }
            // Every pair, not just neighbours: two methods collapsing onto each other is the
            // defect, and which two is not predictable.
            let mut worst = (f64::INFINITY, String::new());
            for i in 0..frames.len() {
                for j in (i + 1)..frames.len() {
                    let d = frame::distance(&frames[i].1, &frames[j].1);
                    if d < worst.0 {
                        worst = (d, format!("{} vs {}", frames[i].0, frames[j].0));
                    }
                }
            }
            if !frames.is_empty() {
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "appearance",
                    name: "colour methods are all different".into(),
                    params: format!("{} methods, {} pairs", frames.len(), frames.len() * (frames.len() - 1) / 2),
                    result: format!("closest pair {} at meanΔ {:.2}", worst.1, worst.0),
                    threshold: "meanΔ ≥ 1.0 for every pair",
                    pass: worst.0 >= 1.0,
                });
            }
            self.coloring.color_method = crate::ColorMethod::from_u32(0);

            // --- palettes, step 54 ---
            let mut pal: Vec<(String, Vec<u8>)> = Vec::new();
            for (i, name) in fractadyne_color::PRESETS.iter().enumerate() {
                self.coloring.palette_idx = i;
                if let Some(px) = render(self, device, queue) {
                    pal.push((name.name.to_string(), px));
                }
            }
            let mut pworst = (f64::INFINITY, String::new());
            let mut pcoherent = true;
            for i in 0..pal.len() {
                pcoherent &= frame::coherent(&pal[i].1);
                for j in (i + 1)..pal.len() {
                    let d = frame::distance(&pal[i].1, &pal[j].1);
                    if d < pworst.0 {
                        pworst = (d, format!("{} vs {}", pal[i].0, pal[j].0));
                    }
                }
            }
            if !pal.is_empty() {
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "appearance",
                    name: "palettes are all different and coherent".into(),
                    params: format!("{} palettes", pal.len()),
                    result: format!("closest pair {} at meanΔ {:.2}", pworst.1, pworst.0),
                    threshold: "meanΔ ≥ 1.0 for every pair, all coherent",
                    pass: pcoherent && pworst.0 >= 1.0,
                });
            }
            self.coloring.palette_idx = 0;

            // --- controls that must visibly do something, steps 55-57 and 60-62 ---
            // Each is a differential against the same baseline, with BOTH sides required coherent.
            // ⚠NOT exercised here: "Log color scale" and "Normalize deep colors" reach the
            // image only through the LIVE normalized mapping. `render_export` does not
            // normalize unless asked, so toggling either against this path reports a
            // byte-identical frame - measured, meanΔ 0.00 - and a check built on it would
            // be green and vacuous. Checklist steps 29, 30 and 57 need the live path; see
            // design/checklist-automation.md.
            self.coloring.log_palette = false;
            self.coloring.normalize_live = false;
            let base = render(self, device, queue);
            let mut toggles: Vec<(&str, Box<dyn Fn(&mut Self)>, Box<dyn Fn(&mut Self)>)> = Vec::new();
            toggles.push(("cycle slider",
                Box::new(|a: &mut Self| a.coloring.cycle = 0.8),
                Box::new(|a: &mut Self| a.coloring.cycle = 0.27)));
            toggles.push(("offset slider",
                Box::new(|a: &mut Self| a.coloring.offset = 0.6),
                Box::new(|a: &mut Self| a.coloring.offset = 0.1)));
            toggles.push(("binary (set)",
                Box::new(|a: &mut Self| a.coloring.use_binary = true),
                Box::new(|a: &mut Self| a.coloring.use_binary = false)));
            toggles.push(("duotone",
                Box::new(|a: &mut Self| a.coloring.use_duotone = true),
                Box::new(|a: &mut Self| a.coloring.use_duotone = false)));
            toggles.push(("3D relief lighting",
                Box::new(|a: &mut Self| a.effects.light = true),
                Box::new(|a: &mut Self| a.effects.light = false)));
            toggles.push(("distance glow",
                Box::new(|a: &mut Self| a.effects.de = true),
                Box::new(|a: &mut Self| a.effects.de = false)));

            if let Some(base) = base {
                let base_ok = frame::coherent(&base);
                for (name, on, off) in toggles {
                    on(self);
                    let got = render(self, device, queue);
                    off(self);
                    let (result, pass) = match got {
                        Some(px) => {
                            let d = frame::distance(&base, &px);
                            let ok = frame::coherent(&px);
                            let (sd, b) = frame::coherence(&px);
                            (
                                format!("meanΔ {d:.2} vs baseline; stddev {sd:.1}, {b} buckets, coherent: {ok}"),
                                base_ok && ok && d >= 1.0,
                            )
                        }
                        None => ("render_export failed".to_string(), false),
                    };
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "appearance",
                        name: format!("control changes the image — {name}"),
                        params: format!("{aw}x{ah}, both frames must be coherent"),
                        result,
                        threshold: "meanΔ ≥ 1.0",
                        pass,
                    });
                }
            }
            // --- negative control for the coherence predicate ---
            // Every check above leans on `coherent`. A predicate that accepted anything would
            // make all of them vacuous while reporting a clean run, so prove the other end:
            // one iteration escapes every pixel at once and must render a FLAT frame, and
            // `coherent` must reject it.
            let iter_was = self.render_cfg.max_iter;
            self.render_cfg.max_iter = 1;
            let flat = render(self, device, queue);
            self.render_cfg.max_iter = iter_was;
            if let Some(px) = flat {
                let (sd, b) = frame::coherence(&px);
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "appearance",
                    name: "the flat-frame control is rejected".into(),
                    params: "max_iter = 1, so every pixel escapes immediately".into(),
                    result: format!("stddev {sd:.1}, {b} buckets — coherent: {}", frame::coherent(&px)),
                    threshold: "must NOT be judged coherent",
                    pass: !frame::coherent(&px),
                });
            }

            // --- anti-aliasing, step 66 ---
            // Supersampling must visibly soften edges. Measured as the mean luma step between
            // horizontally adjacent pixels: aliased edges are hard jumps, a resolved edge is a
            // ramp. Both frames must be coherent, so this cannot pass by rendering nothing.
            //
            // ⚠The supersampling of an EXPORT comes from `export.ss`, not `render_cfg.aa` -
            // that one drives the live view. Written against `aa` first, this check reported
            // an identical edge measure at 1x and 2x, to two decimal places, because nothing
            // it set ever reached the renderer. An unchanged number is the shape of a check
            // that cannot fail, not of a feature that does nothing.
            let ss_was = self.export.ss;
            self.export.ss = 1;
            let aa1 = render(self, device, queue);
            self.export.ss = 2;
            let aa2 = render(self, device, queue);
            self.export.ss = ss_was;
            if let (Some(a), Some(b)) = (aa1, aa2) {
                let (e1, e2) = (frame::neighbour_step(&a, aw), frame::neighbour_step(&b, aw));
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "appearance",
                    name: "supersampling softens edges".into(),
                    params: format!("{aw}x{ah}, export ss 1x vs 2x"),
                    result: format!("edge step {e1:.2} -> {e2:.2}"),
                    threshold: "2x strictly lower, both coherent",
                    pass: frame::coherent(&a) && frame::coherent(&b) && e2 < e1,
                });
            }
        }


        // ---- manual-checklist rows the other groups do not reach ----
        //
        // The rows here are the ones whose whole content is "render this and look at it":
        // the depth ladder (25, 27, 28), Julia (44), random locations (69), the two colour
        // mappings that only exist on the normalized path (29, 30, 57, 58), the export rows
        // (77, 78, 80) and the rapid-switching soak (105). See
        // design/checklist-automation.md for which clause of each row this covers and which
        // stays a human judgement.
        if want("checklist") {
            let (cw, ch) = (320u32, 180u32);
            // Pin everything that is not the variable under test, and put the view back
            // afterwards — later groups (and the goldens) share this app instance.
            let saved_vp = self.viewport.clone();
            let saved_iter = self.render_cfg.max_iter;
            let saved_auto = self.render_cfg.auto_iter;
            let saved_fractal = self.fractal;
            let saved_julia = self.julia_mode;
            self.fractal = crate::FractalKind::Mandelbrot;
            self.julia_mode = false;
            self.dual = false;
            self.render_cfg.auto_iter = false;
            self.render_cfg.aa = 1;
            self.export.ss = 1;
            self.coloring.color_method = crate::ColorMethod::Smooth;
            self.coloring.palette_idx = 0;
            self.coloring.use_custom_palette = false;
            self.coloring.use_binary = false;
            self.coloring.use_duotone = false;
            self.coloring.log_palette = false;
            self.coloring.normalize_live = false;
            self.coloring.cycle = 0.27;
            self.coloring.offset = 0.1;
            self.anim.palette_anim = crate::PaletteAnim::Off;
            self.effects.light = false;
            self.effects.de = false;

            // Render the CURRENT app view at (w,h) through the ordinary export path, which
            // chunks its dispatches — a deep frame at a real iteration count is exactly the
            // unbounded-submission shape that loses the device, and a self-test must never
            // crash the GPU it is validating.
            let shoot = |app: &Self, dev: &eframe::wgpu::Device, q: &eframe::wgpu::Queue,
                         w: u32, h: u32| -> Option<(Vec<u8>, u32)> {
                // ⚠`julia` here is the request's OWN flag, not a panel index: a single-view
                // Julia render must ASK for one. Passing `false` renders the parameter plane
                // with Julia mode on and reports a frame identical to the Mandelbrot — which
                // is exactly what this check first measured (meanΔ 0.00).
                let mut req = app.current_export_request_for(&app.viewport, app.julia_mode);
                req.width = w;
                req.height = h;
                let orbit_len = req.orbit_len;
                let progress = std::sync::atomic::AtomicU32::new(0);
                let cancel = std::sync::atomic::AtomicBool::new(false);
                fractadyne_gpu::render_export(dev, q, &req, &progress, &cancel)
                    .ok()
                    .map(|r| (fractadyne_export::to_srgb8_dithered(&r.pixels, r.width), orbit_len))
            };
            // Jump the app's view to a full-precision location at `mag_log10`.
            let goto = |app: &mut Self, cx: &str, cy: &str, mag_log10: f64, iter: u32| {
                let log2mag = mag_log10 * std::f64::consts::LOG2_10;
                let prec = fractadyne_core::precision_for_octaves(log2mag.max(0.0).ceil() as u64);
                // ⚠A silent fallback here would be indistinguishable from the bug these checks
                // hunt: an unparseable centre lands on the whole set, which at 1e500× renders a
                // flat frame and reads as "deep zoom is broken". Panic instead — this is a
                // literal in this file, so a failure to parse is a typo, not an input.
                let x = fractadyne_core::parse_bf_prec(cx, prec)
                    .unwrap_or_else(|| panic!("selftest: centre {cx:?} does not parse"));
                let y = fractadyne_core::parse_bf_prec(cy, prec)
                    .unwrap_or_else(|| panic!("selftest: centre {cy:?} does not parse"));
                app.viewport.set_size(cw as f64, ch as f64);
                app.viewport.set_center_log2mag(x, y, log2mag);
                app.viewport.precision = prec;
                app.render_cfg.max_iter = iter;
            };

            // --- steps 25 and 27: the depth ladder ---
            // Coordinates and iteration counts are the F3 comparison corpus's own, so a rung
            // that goes black here is a location we have independently rendered correctly.
            // (name, cx, cy, log10 magnification, iterations)
            type Rung = (&'static str, &'static str, &'static str, f64, u32);
            const SEA_X: &str = "-0.7436438870371587047521915061147707";
            const SEA_Y: &str = "0.131825904205311970493132056385139";
            const LADDER: &[Rung] = &[
                ("1e0", "-0.75", "0.0", 0.125, 512),
                ("1.3e4", SEA_X, SEA_Y, 4.125, 1_500),
                ("1.3e6", SEA_X, SEA_Y, 6.125, 3_000),
                ("3.9e12", "-0.743643908041274519886726", "0.131825923574324509717824", 12.591, 60_000),
                ("1.1e18", "-0.71455191519512020059044918385", "0.35402073332318232065549730365", 18.036, 50_000),
                ("1.3e24", SEA_X, SEA_Y, 24.125, 20_000),
            ];
            let mut ladder_ok = true;
            let mut worst = (f32::INFINITY, "");
            let mut prev = (0usize, 0u32); // (precision, orbit_len)
            let mut first_prec = 0usize;
            let mut grows = true;
            let mut growth = String::new();
            for (name, cx, cy, mag, iter) in LADDER {
                goto(self, cx, cy, *mag, *iter);
                let prec = self.viewport.precision;
                match shoot(self, device, queue, cw, ch) {
                    Some((px, orbit_len)) => {
                        let (sd, _) = frame::coherence(&px);
                        if !frame::coherent(&px) {
                            ladder_ok = false;
                        }
                        if sd < worst.0 {
                            worst = (sd, name);
                        }
                        // Depth must cost something: the working precision has to grow as the
                        // ladder descends, or the deeper rungs are being rendered with the
                        // shallow view's machinery.
                        //
                        // ⚠Reference ORBIT LENGTH is reported but NOT gated. It is bounded by
                        // where the reference point escapes, which is a property of the
                        // location and not of the depth: measured across these rungs it goes
                        // 1501 → 3001 → 1558 → 619 → 20001, and it is perfectly healthy. The
                        // checklist row's "orbit length grows" is a claim about diving at ONE
                        // point, which a ladder of different points cannot test.
                        if prec < prev.0 {
                            grows = false;
                        }
                        if first_prec == 0 {
                            first_prec = prec;
                        }
                        // What CAN be required of every perturbed rung: a reference exists.
                        if *mag > 1.0 && orbit_len == 0 {
                            ladder_ok = false;
                        }
                        growth.push_str(&format!("{name}:{prec}b/{orbit_len} "));
                        prev = (prec, orbit_len);
                    }
                    None => {
                        ladder_ok = false;
                        growth.push_str(&format!("{name}:FAILED "));
                    }
                }
            }
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "checklist",
                name: "depth-ladder-coherent".into(),
                params: format!("{} rungs, 1x -> 1.3e24x, {cw}x{ch}", LADDER.len()),
                result: format!(
                    "weakest rung {} at stddev {:.1}; precision {first_prec}b -> {}b; {growth}",
                    worst.1, worst.0, prev.0
                ),
                threshold: "every rung coherent, perturbed rungs have a reference; precision more than doubles",
                // ⚠Non-decreasing is NOT enough: a CONSTANT precision satisfies it while
                // making depth cost nothing, and that mutant passed this check on its first
                // draft (every rung pinned at 64 bits, green). It has to actually climb.
                pass: ladder_ok && grows && prev.0 > first_prec * 2,
            });

            // --- step 28: past the f64 magnification range ---
            // 6.1e500 is corpus location 09. It matters specifically because `magnification()`
            // SATURATES to +inf past ~1e308x, and a guard written for NaN once demoted every
            // such view to Direct mode and rendered an empty frame (beta.125).
            // Corpus location 09 (6.1e500×, 150,000 iterations), verbatim — see the drift
            // guard below. ⚠These were first typed from memory rather than copied, and the
            // resulting frame was flat: a wrong deep centre is not a wrong picture, it is the
            // WHOLE SET, which at this depth is a blank field and reads exactly like the
            // saturation bug this row exists to catch.
            const X500: &str = "-8.351966078548609175704283083728201809956421539984007929099437008685832333266\
6026012321442424716476137516010235155803265588739473477613596416091464645795520269598424012720833\
7641161382449650762068504929672877197722390865649996670577215903692518919284922807301340923025946\
7459812564279863991009144218705795579205742155079434234517406000246525499747298743298423112048661\
8202330117277556383076138282583978997392314887381834712013461059227773552093199422831818832614215\
489147840039739096870634260502312035491160466210910542672e-2";
            const Y500: &str = "6.563392665142135764243544562479428717973237578041407051280494868053203874959\
4379415920265613034895936155571162042591359618401572538324489365585021937324229690811051813054355\
3032971465057843804726868054110073433374070365768499180999961785644891370747637496781349088901691\
9265370226945907099365482327646526518611942469695308223223586411313594133148334474017001142785407\
3921885047231710113229147644154379696549177162208675566004999502643881966677072076493891512329846\
424644392411879461499442500274655605273427804756526273386e-1";

            self.render_cfg.max_iter = 150_000;
            goto(self, X500, Y500, 500.91, 150_000);
            let extreme = shoot(self, device, queue, cw, ch);
            let (res, pass) = match &extreme {
                Some((px, orbit_len)) => {
                    let (sd, b) = frame::coherence(px);
                    (
                        format!("stddev {sd:.1}, {b} buckets, orbit_len {orbit_len}"),
                        frame::coherent(px) && *orbit_len > 0,
                    )
                }
                None => ("render_export failed".to_string(), false),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "checklist",
                name: "extreme-depth-coherent".into(),
                params: format!("6.1e500x, 150,000 iterations, {cw}x{ch}"),
                result: res,
                threshold: "coherent frame from a real reference orbit (not blank, not flat)",
                pass,
            });

            // --- steps 29, 30 and 57: the normalized colour mappings ---
            // These reach the image ONLY through the normalized path. `render_export` does not
            // normalize unless asked, so an A/B of the checkbox against it reports a
            // byte-identical frame (measured, meanD 0.00) — a check built that way is green
            // and vacuous. `render_export_normalized` is the mapping the checkbox selects.
            let normed = |app: &Self, dev: &eframe::wgpu::Device, q: &eframe::wgpu::Queue|
             -> Option<Vec<u8>> {
                app.render_export_normalized(dev, q, &app.viewport, false, cw, ch, 1, crate::render::NormRange::OwnFrame, None, u64::MAX)
                    .map(|(r, _)| fractadyne_export::to_srgb8_dithered(&r.pixels, r.width))
            };
            // A deep, dense field: the regime where an un-normalized palette aliases into
            // per-pixel confetti and normalizing is what makes the bands readable.
            goto(self, SEA_X, SEA_Y, 24.125, 20_000);
            let plain = shoot(self, device, queue, cw, ch).map(|(px, _)| px);
            let norm = normed(self, device, queue);
            let (res, pass) = match (&plain, &norm) {
                (Some(a), Some(b)) => {
                    let (sa, sb) = (frame::neighbour_step(a, cw), frame::neighbour_step(b, cw));
                    (
                        format!("neighbour step {sa:.2} -> {sb:.2}, meanD {:.2}", frame::distance(a, b)),
                        frame::coherent(a) && frame::coherent(b) && sb < sa,
                    )
                }
                _ => ("render failed".to_string(), false),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "checklist",
                name: "normalize-reduces-speckle".into(),
                params: format!("1.3e24x, 20,000 iterations, {cw}x{ch}, both frames must be coherent"),
                result: res,
                threshold: "normalized frame has the SMALLER neighbour step",
                pass,
            });

            // Log colour scale, on the same view and the same path.
            let lin = normed(self, device, queue);
            self.coloring.log_palette = true;
            let logd = normed(self, device, queue);
            self.coloring.log_palette = false;
            let (res, pass) = match (&lin, &logd) {
                (Some(a), Some(b)) => {
                    let d = frame::distance(a, b);
                    (format!("meanD {d:.2}"), frame::coherent(a) && frame::coherent(b) && d >= 1.0)
                }
                _ => ("render failed".to_string(), false),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "checklist",
                name: "log-scale-changes-the-image".into(),
                params: "normalized mapping, log off vs on".into(),
                result: res,
                threshold: "meanD >= 1.0, both coherent",
                pass,
            });

            // --- step 58: a gradient edit reaches the image ---
            // The dialog is a human check; what a machine can hold is that a changed STOP
            // changes the picture, and that the custom gradient is what is being sampled
            // rather than the preset silently continuing to win.
            goto(self, SEA_X, SEA_Y, 6.125, 3_000);
            let preset = shoot(self, device, queue, cw, ch).map(|(px, _)| px);
            self.coloring.custom_palette = vec![
                [0.0, 0.0, 0.0, 0.0],
                [0.35, 0.55, 0.02, 0.02],
                [0.7, 1.0, 0.55, 0.05],
                [1.0, 1.0, 1.0, 0.75],
            ];
            self.coloring.use_custom_palette = true;
            let edited = shoot(self, device, queue, cw, ch).map(|(px, _)| px);
            // Move ONE stop; everything else about the gradient is unchanged.
            self.coloring.custom_palette[1] = [0.35, 0.02, 0.10, 0.75];
            let moved = shoot(self, device, queue, cw, ch).map(|(px, _)| px);
            self.coloring.use_custom_palette = false;
            self.coloring.custom_palette.clear();
            let (res, pass) = match (&preset, &edited, &moved) {
                (Some(p), Some(e), Some(m)) => {
                    let (d1, d2) = (frame::distance(p, e), frame::distance(e, m));
                    (
                        format!("preset->custom meanD {d1:.2}, one stop moved meanD {d2:.2}"),
                        frame::coherent(e) && frame::coherent(m) && d1 >= 1.0 && d2 >= 1.0,
                    )
                }
                _ => ("render failed".to_string(), false),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "checklist",
                name: "gradient-edit-changes-the-image".into(),
                params: format!("{cw}x{ch}, custom gradient vs preset, then one stop moved"),
                result: res,
                threshold: "both meanD >= 1.0, both coherent",
                pass,
            });

            // --- palette import: the three traps that look PLAUSIBLE when they are wrong ---
            //
            // Each importer was checked end to end by hand when it shipped (beta.24-27), and a
            // manual check is not a gate: it does not run again. These put the same three
            // measurements in the release suite.
            //
            // ⭐⭐Every one is a CONTROL PAIR — the same file with ONE field changed — because the
            // naive form of each check cannot fail. "The .map render has few colours" is also true
            // of a smoothed import of a dark palette; "the .ugr render is red" is also true if red
            // and blue were swapped and the file happened to be red. Changing one field and
            // requiring the OPPOSITE answer is what makes them able to go red.
            //
            // The palette state is restored at the end of the block; later checks share this app.
            const IMPORT_MAP: &str = "0 0 0\n64 64 64\n128 128 128\n192 192 192\n252 252 252\n";
            // color=255 is 0x0000FF and color=16711680 is 0xFF0000. Ultra Fractal packs BGR, so
            // the first is RED and the second is BLUE; under an RGB reading they swap.
            const IMPORT_UGR: &str = "r {\ngradient:\ntitle=\"r\" index=0 color=255 index=399 color=255\n}\nb {\ngradient:\ntitle=\"b\" index=0 color=16711680 index=399 color=16711680\n}\n";
            // One segment, red at BOTH ends. In RGB that is flat red; swept round the hue wheel it
            // is the whole spectrum. The two files differ only in the final column.
            const IMPORT_GGR_RGB: &str = "GIMP Gradient\nName: rgb\n1\n0 0.5 1 1 0 0 1 1 0 0 1 0 0\n";
            const IMPORT_GGR_HSV: &str = "GIMP Gradient\nName: hsv\n1\n0 0.5 1 1 0 0 1 1 0 0 1 0 1\n";

            let distinct = |px: &[u8]| -> usize {
                let mut v: Vec<[u8; 3]> = px.chunks_exact(4).map(|p| [p[0], p[1], p[2]]).collect();
                v.sort_unstable();
                v.dedup();
                v.len()
            };

            goto(self, SEA_X, SEA_Y, 6.125, 3_000);
            self.coloring.use_custom_palette = true;

            // (1) A `.map` imported as BANDS must render ONLY the levels the file declares.
            let m = fractadyne_color::import::parse_map(IMPORT_MAP).expect("selftest .map fixture");
            let n = m.colors.len();
            self.coloring.custom_palette = m
                .colors
                .iter()
                .enumerate()
                .map(|(i, c)| [i as f32 / (n - 1) as f32, c[0], c[1], c[2]])
                .collect();
            self.coloring.custom_segments.clear();
            self.coloring.custom_palette_flat = true;
            let banded = shoot(self, device, queue, cw, ch).map(|(px, _)| px);
            self.coloring.custom_palette_flat = false; // the control: same colours, blended
            let smoothed = shoot(self, device, queue, cw, ch).map(|(px, _)| px);
            let (res, pass) = match (&banded, &smoothed) {
                (Some(b), Some(sm)) => {
                    // Declared levels, as bytes. The in-set colour is not grey, so filtering to
                    // r == g == b isolates the palette without needing to know the interior.
                    let want: Vec<u8> = m.colors.iter().map(|c| (c[0] * 255.0).round() as u8).collect();
                    let greys = |px: &[u8]| -> Vec<u8> {
                        px.chunks_exact(4)
                            .filter(|p| p[0] == p[1] && p[1] == p[2])
                            .map(|p| p[0])
                            .collect()
                    };
                    let bg = greys(b);
                    let stray = bg.iter().filter(|v| !want.contains(v)).count();
                    let band_levels = {
                        let mut v = bg.clone();
                        v.sort_unstable();
                        v.dedup();
                        v.len()
                    };
                    let smooth_levels = {
                        let mut v = greys(sm);
                        v.sort_unstable();
                        v.dedup();
                        v.len()
                    };
                    (
                        format!(
                            "banded: {band_levels} levels over {} grey px, {stray} off-palette; \
                             smoothed control: {smooth_levels} levels",
                            bg.len()
                        ),
                        // The bands must be EXACT, and the control must prove the exactness is the
                        // banding rather than a dark palette with few colours in it anyway.
                        !bg.is_empty()
                            && stray == 0
                            && band_levels <= want.len()
                            && smooth_levels > band_levels * 3
                            && frame::coherent(b),
                    )
                }
                _ => ("render failed".to_string(), false),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "checklist",
                name: "map-bands-are-exact".into(),
                params: format!("{cw}x{ch}, {n}-entry .map as bands, then the same file smoothed"),
                result: res,
                threshold: "zero off-palette pixels; the smoothed control has >3x the levels",
                pass,
            });

            // (2) `.ugr` packs colour BGR. Reading it as RGB swaps red and blue on every gradient
            // and still looks like a plausible palette — so the check renders BOTH and requires
            // them to come out opposite ways round.
            let ugr = fractadyne_color::import::parse_ugr(IMPORT_UGR).expect("selftest .ugr fixture");
            let mut shots = Vec::new();
            for g in &ugr {
                let st = g.to_gradient().to_stops();
                self.coloring.custom_palette =
                    st.into_iter().map(|(p, c)| [p, c[0], c[1], c[2]]).collect();
                self.coloring.custom_palette_flat = false;
                shots.push(shoot(self, device, queue, cw, ch).map(|(px, _)| px));
            }
            let (res, pass) = match (shots.first().and_then(|s| s.as_ref()), shots.get(1).and_then(|s| s.as_ref())) {
                (Some(red), Some(blue)) => {
                    // Mean red and blue over the exterior, as whole-frame channel means.
                    let chan = |px: &[u8], i: usize| -> f64 {
                        px.chunks_exact(4).map(|p| p[i] as f64).sum::<f64>()
                            / (px.len() / 4).max(1) as f64
                    };
                    let (rr, rb) = (chan(red, 0), chan(red, 2));
                    let (br, bb) = (chan(blue, 0), chan(blue, 2));
                    (
                        format!("color=255 -> r {rr:.1} b {rb:.1}; color=16711680 -> r {br:.1} b {bb:.1}"),
                        rr > rb * 2.0 && bb > br * 2.0 && frame::coherent(red),
                    )
                }
                _ => ("render failed".to_string(), false),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "checklist",
                name: "ugr-color-is-bgr".into(),
                params: "color=255 must render RED and color=16711680 BLUE (red is the LOW byte)".into(),
                result: res,
                threshold: "each render's own channel leads the other by 2x, both ways round",
                pass,
            });

            // (3) A `.ggr` segment carries its own colour SPACE, so identical endpoints swept round
            // the hue wheel are a whole spectrum while in RGB they are one flat colour. The two
            // fixtures differ in exactly one integer.
            let mut ggr_shots = Vec::new();
            for text in [IMPORT_GGR_RGB, IMPORT_GGR_HSV] {
                let g = fractadyne_color::import::parse_ggr(text).expect("selftest .ggr fixture");
                self.set_custom_segments(&g);
                ggr_shots.push(shoot(self, device, queue, cw, ch).map(|(px, _)| px));
            }
            let (res, pass) = match (&ggr_shots[0], &ggr_shots[1]) {
                (Some(rgb), Some(hsv)) => {
                    let (dr, dh) = (distinct(rgb), distinct(hsv));
                    (
                        format!("RGB space: {dr} distinct colours; HSV sweep: {dh}"),
                        dh > dr * 10 && frame::coherent(hsv),
                    )
                }
                _ => ("render failed".to_string(), false),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "checklist",
                name: "ggr-colour-space-is-per-segment".into(),
                params: "one segment, red at both ends, colouring column 0 vs 1".into(),
                result: res,
                threshold: "the hue sweep yields >10x the distinct colours of the RGB reading",
                pass,
            });

            self.coloring.custom_segments.clear();
            self.coloring.custom_palette.clear();
            self.coloring.custom_palette_flat = false;
            self.coloring.use_custom_palette = false;

            // --- step 44: Julia mode ---
            self.viewport.reset_to(0.0, 0.0);
            self.viewport.set_size(cw as f64, ch as f64);
            self.render_cfg.max_iter = 2_000;
            self.julia_c = (-0.743_643_887_037_15, 0.131_825_904_205_31);
            let mandel = shoot(self, device, queue, cw, ch).map(|(px, _)| px);
            self.julia_mode = true;
            let julia = shoot(self, device, queue, cw, ch).map(|(px, _)| px);
            self.julia_mode = false;
            let (res, pass) = match (&mandel, &julia) {
                (Some(m), Some(j)) => {
                    let (sd, b) = frame::coherence(j);
                    let d = frame::distance(m, j);
                    (
                        format!("stddev {sd:.1}, {b} buckets; meanD {d:.2} vs the parameter plane"),
                        frame::coherent(j) && d >= 1.0,
                    )
                }
                _ => ("render failed".to_string(), false),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "checklist",
                name: "julia-coherent".into(),
                params: "c = -0.743644 + 0.131826i, whole-plane framing".into(),
                result: res,
                threshold: "coherent, and not the same image as the Mandelbrot",
                pass,
            });

            // --- step 69: random locations ---
            // The picker bisects onto the boundary, so every jump should land in detail. A
            // seed that lands on a blank field is a real defect and a reproducible one — the
            // seed is in the report.
            let mut bad: Vec<String> = Vec::new();
            const SEEDS: [u64; 6] = [1, 7, 12345, 0x9E37_79B9, 0xDEAD_BEEF, u64::MAX / 3];
            for seed in SEEDS {
                let (cx, cy, mag) = crate::random_boundary_location(seed);
                self.viewport.set_size(cw as f64, ch as f64);
                self.viewport.set_center_mag(
                    fractadyne_core::BigFloat::from_f64(cx, 64),
                    fractadyne_core::BigFloat::from_f64(cy, 64),
                    mag,
                );
                self.viewport.precision = fractadyne_core::precision_for_magnification(mag);
                self.render_cfg.max_iter = 20_000;
                match shoot(self, device, queue, cw, ch).map(|(px, _)| px) {
                    Some(px) if frame::coherent(&px) => {}
                    Some(px) => {
                        let (sd, _) = frame::coherence(&px);
                        bad.push(format!("seed {seed} @{mag:.1e} flat (stddev {sd:.1})"));
                    }
                    None => bad.push(format!("seed {seed} failed to render")),
                }
            }
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "checklist",
                name: "random-locations-coherent".into(),
                params: format!("{} seeds, 1e2..1e6x, {cw}x{ch}", SEEDS.len()),
                result: if bad.is_empty() { "all landed in structure".into() } else { bad.join("; ") },
                threshold: "every random location renders a coherent (non-flat) frame",
                pass: bad.is_empty(),
            });

            // --- step 77: the snapshot on disk IS the view ---
            // "Not corrupt or truncated" and "matches what was on screen" are one question for
            // a file: do the bytes decode back to exactly the pixels that were rendered, and
            // does the metadata it carries name the same view?
            goto(self, SEA_X, SEA_Y, 6.125, 3_000);
            let snap = {
                let mut req = self.current_export_request_for(&self.viewport, false);
                req.width = cw;
                req.height = ch;
                let progress = std::sync::atomic::AtomicU32::new(0);
                let cancel = std::sync::atomic::AtomicBool::new(false);
                fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel).ok()
            };
            let (res, pass) = match snap {
                Some(r) => {
                    let want = fractadyne_export::to_srgb8_dithered(&r.pixels, r.width);
                    let meta = self.view_metadata();
                    let path = std::env::temp_dir().join("fractadyne-selftest-snapshot.png");
                    match fractadyne_export::write_png(&path, r.width, r.height, &r.pixels, Some(&meta))
                        .and_then(|()| fractadyne_export::read_png_rgba8(&path))
                    {
                        Ok((w, h, got)) => {
                            let same = w == r.width && h == r.height && got == want;
                            let back = fractadyne_export::read_png_metadata(&path)
                                .ok()
                                .flatten()
                                .unwrap_or_default();
                            let framed = crate::meta_get(&back, "center_re")
                                == crate::meta_get(&meta, "center_re")
                                && crate::meta_get(&back, "upp_log2") == crate::meta_get(&meta, "upp_log2");
                            let (max, mean) = img_diff(&want, &got);
                            (
                                format!("{w}x{h}, maxD {max}, meanD {mean:.3}, framing recovered: {framed}"),
                                same && framed && frame::coherent(&want),
                            )
                        }
                        Err(e) => (format!("write/read failed: {e}"), false),
                    }
                }
                None => ("render_export failed".to_string(), false),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "checklist",
                name: "snapshot-matches-the-view".into(),
                params: format!("{cw}x{ch} PNG written, decoded, compared byte for byte"),
                result: res,
                threshold: "decoded pixels identical; embedded centre + depth match the view",
                pass,
            });

            // --- step 78: a 4K export with supersampling completes ---
            // Larger than any window, and supersampled, so it goes down the tiled path. What
            // fails here is not subtlety: a truncated buffer, a clamped size, or a band of
            // untouched pixels where a tile never ran.
            goto(self, SEA_X, SEA_Y, 6.125, 3_000);
            let big = {
                let mut req = self.current_export_request_for(&self.viewport, false);
                req.width = 3840;
                req.height = 2160;
                req.ss = 2;
                let progress = std::sync::atomic::AtomicU32::new(0);
                let cancel = std::sync::atomic::AtomicBool::new(false);
                fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel).ok()
            };
            let (res, pass) = match big {
                Some(r) => {
                    let px = fractadyne_export::to_srgb8_dithered(&r.pixels, r.width);
                    let full = px.len() == (r.width as usize) * (r.height as usize) * 4;
                    // Every horizontal band must carry image, not just the frame as a whole:
                    // a missing tile leaves a flat strip that a whole-frame stddev hides.
                    let rows = r.height as usize / 8;
                    let stride = r.width as usize * 4;
                    let mut flat_band = None;
                    for band in 0..8 {
                        let a = band * rows * stride;
                        let b = ((band + 1) * rows * stride).min(px.len());
                        if a < b && !frame::coherent(&px[a..b]) {
                            flat_band = Some(band);
                        }
                    }
                    (
                        format!(
                            "{}x{} ss{} ({} px), flat band: {}",
                            r.width, r.height, r.ss, px.len() / 4,
                            flat_band.map_or("none".to_string(), |b| b.to_string())
                        ),
                        full && r.width == 3840 && r.height == 2160 && flat_band.is_none(),
                    )
                }
                None => ("render_export failed".to_string(), false),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "checklist",
                name: "export-4k-complete".into(),
                params: "3840x2160, ss 2x, 1.3e6x".into(),
                result: res,
                threshold: "full-size buffer, every eighth of the frame carries image",
                pass,
            });

            // --- step 80: a deep export ---
            // Corpus location 08 (6.6e43×, 60,000 iterations), verbatim.
            const X43: &str = "-6.70209187903253724099340233845986400901890228472988919658169553187602139279518e-1";
            const Y43: &str = "4.58060975296945872909213676106313996238241655922637652387687460587764642477807e-1";
            goto(self, X43, Y43, 43.9477217539083, 60_000);
            let deep = shoot(self, device, queue, 960, 540);
            let (res, pass) = match &deep {
                Some((px, orbit_len)) => {
                    let (sd, b) = frame::coherence(px);
                    let meta = self.view_metadata();
                    // The framing the file would carry must be the view that was rendered.
                    let l2 = crate::meta_get(&meta, "upp_log2").parse::<f64>().unwrap_or(0.0);
                    let framed = (l2 - self.viewport.units_per_pixel.log2()).abs() < 1.0e-9;
                    (
                        format!("stddev {sd:.1}, {b} buckets, orbit_len {orbit_len}, framing recorded: {framed}"),
                        frame::coherent(px) && *orbit_len > 0 && framed,
                    )
                }
                None => ("render_export failed".to_string(), false),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "checklist",
                name: "deep-export-matches-the-view".into(),
                params: "6.6e43x, 60,000 iterations, 960x540".into(),
                result: res,
                threshold: "coherent, real reference orbit, depth recorded exactly",
                pass,
            });

            // --- step 105: rapid switching settles on the final choice ---
            // The failure this guards is a STALE frame: switch formula, method and palette
            // faster than the caches turn over and the picture can end up showing one of the
            // earlier choices. The proof is an equality — the frame after the whole switching
            // storm must equal the frame you get by setting only the final selection.
            self.viewport.reset_to(-0.5, 0.0);
            self.viewport.set_size(cw as f64, ch as f64);
            self.render_cfg.max_iter = 2_000;
            let order = [
                crate::FractalKind::Mandelbrot, crate::FractalKind::Tricorn,
                crate::FractalKind::BurningShip, crate::FractalKind::Multibrot3,
                crate::FractalKind::Celtic, crate::FractalKind::Buffalo,
            ];
            let mut switched = None;
            for round in 0..3 {
                for (i, f) in order.iter().enumerate() {
                    self.fractal = *f;
                    self.coloring.color_method = crate::ColorMethod::from_u32(
                        ((i + round) % crate::ColorMethod::ALL.len()) as u32,
                    );
                    self.coloring.palette_idx = (i + round) % fractadyne_color::PRESETS.len();
                    self.invalidate_refs();
                    // Render only the last one; the point is the state left behind, and
                    // rendering all 18 would make this the slowest check in the suite.
                    if round == 2 && i == order.len() - 1 {
                        switched = shoot(self, device, queue, cw, ch).map(|(px, _)| px);
                    }
                }
            }
            let (ff, fm, fp) = (self.fractal, self.coloring.color_method, self.coloring.palette_idx);
            // The control: the same final selection, arrived at without the storm.
            self.fractal = crate::FractalKind::Mandelbrot;
            self.coloring.color_method = crate::ColorMethod::from_u32(0);
            self.coloring.palette_idx = 0;
            self.invalidate_refs();
            let control = shoot(self, device, queue, cw, ch).map(|(px, _)| px);
            self.fractal = ff;
            self.coloring.color_method = fm;
            self.coloring.palette_idx = fp;
            self.invalidate_refs();
            let clean = shoot(self, device, queue, cw, ch).map(|(px, _)| px);
            // Anti-vacuity: the control render above used a DIFFERENT selection, and its frame
            // must differ from the final one. Without this, an equality check would pass just as
            // happily if every selection rendered the same picture.
            let (res, pass) = match (&switched, &clean, &control) {
                (Some(a), Some(b), Some(c)) => {
                    let (max, mean) = img_diff(a, b);
                    let other = frame::distance(a, c);
                    (
                        format!(
                            "{} / {} / palette {fp}: maxD {max}, meanD {mean:.3}; another selection differs by meanD {other:.2}",
                            ff.name(), fm.label()
                        ),
                        frame::coherent(a) && a == b && other >= 1.0,
                    )
                }
                _ => ("render failed".to_string(), false),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "checklist",
                name: "rapid-switching-settles-on-the-final-choice".into(),
                params: format!("{} switches of formula x method x palette", order.len() * 3),
                result: res,
                threshold: "identical to a clean render of the final choice, and different from another",
                pass,
            });

            // The embedded deep centres are copies of the comparison corpus's own locations, so
            // that a rung failing here is a location we have independently rendered correctly
            // against Fraktaler-3. A copy can drift from its source silently, so when the
            // corpus is present (a repo checkout, not a release tarball) check that it has not.
            let corpus = std::fs::read_to_string(anchored("validation/corpus/locations.toml")).ok();
            if let Some(text) = corpus {
                let missing: Vec<&str> = [("X500", X500), ("Y500", Y500), ("X43", X43), ("Y43", Y43)]
                    .iter()
                    .filter(|(_, v)| !text.contains(*v))
                    .map(|(n, _)| *n)
                    .collect();
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "checklist",
                    name: "deep centres still match the comparison corpus".into(),
                    params: "validation/corpus/locations.toml".into(),
                    result: if missing.is_empty() {
                        "all four embedded centres found verbatim".into()
                    } else {
                        format!("not in the corpus: {}", missing.join(", "))
                    },
                    threshold: "every embedded deep centre appears in the corpus verbatim",
                    pass: missing.is_empty(),
                });
            }

            // Put the app back the way this group found it.
            self.viewport = saved_vp;
            self.render_cfg.max_iter = saved_iter;
            self.render_cfg.auto_iter = saved_auto;
            self.fractal = saved_fractal;
            self.julia_mode = saved_julia;
            self.coloring.palette_idx = 0;
            self.coloring.color_method = crate::ColorMethod::from_u32(0);
            self.invalidate_refs();
        }

        // ---- golden-image regression ----
        // Every render-affecting field is pinned explicitly below (per spec + hard-coded coloring
        // state), so the goldens depend only on the spec and never on the loaded session / current
        // defaults. Fields gated off here (light/de/duotone/binary, orbit-trap) don't reach the
        // output, so their sub-parameters are left as-is.
        let bless = self.selftest.bless; // from new()'s expanded args (honors @response-file)
        let report_path = std::env::args()
            .position(|a| a == "--out" || a == "-o")
            .and_then(|i| std::env::args().nth(i + 1))
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| anchored("validation/report.md"));
        let out_base = report_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        // Canonical committed reference set — always read (and, on --bless, write) the goldens from
        // `validation/golden`, regardless of where --out writes the report. The old
        // `out_base/golden` derivation silently reported "no golden" (a fake maxΔ 255 fail) when
        // --out pointed away from validation/. The `current/` side-by-side renders still go by --out.
        // `anchored` (D2.6) finds the repo tree when the suite runs from another directory.
        let golden_dir = anchored("validation/golden");
        let current_dir = out_base.join("current");
        let _ = std::fs::create_dir_all(&golden_dir);
        if !bless {
            let _ = std::fs::create_dir_all(&current_dir);
        }
        // (name, fractal, cx, cy, zoom, iter, method, palette). The Mandelbrot views exercise deep
        // zoom / coloring; the per-family overviews guard each formula's escape dispatch across the
        // CPU orbit and the direct-mode shader. (The deep-zoom views are all Mandelbrot, so without
        // these a non-Mandelbrot formula regression would render wrong yet pass — see fractal.rs.)
        // (name, fractal, center_x, center_y, zoom, max_iter, color_method, palette_idx, relief)
        type GoldenSpec =
            (&'static str, FractalKind, &'static str, &'static str, f64, u32, u32, usize, bool);
        let specs: &[GoldenSpec] = &[
            ("home", FractalKind::Mandelbrot, "-0.5", "0.0", 1.0, 800, 0, 0, false),
            ("seahorse", FractalKind::Mandelbrot, SX, SY, 2.0e3, 1500, 0, 1, false),
            ("seahorse-stripe-1e6", FractalKind::Mandelbrot, SX, SY, 1.0e6, 4000, 1, 1, false),
            // ⭐⭐RELIEF LIGHTING — the ONLY golden that turns it on, added 2026-08-25 because
            // nothing covered it at all. A change to the shading math altered every relief-lit
            // image in the app and this suite stayed 17/17, which is the definition of an
            // uncovered feature. ⚠`light_anim` MUST be pinned off below: "Rotate light" advances
            // the angle with wall-clock time, so an animated light makes the golden
            // non-deterministic exactly the way `palette_anim` would.
            ("seahorse-relief-1e6", FractalKind::Mandelbrot, SX, SY, 1.0e6, 4000, 0, 1, true),
            ("elephant", FractalKind::Mandelbrot, "0.2925755", "-0.0149977", 1.5e3, 1500, 0, 2, false),
            ("multibrot3", FractalKind::Multibrot3, "0.0", "0.0", 0.8, 800, 0, 0, false),
            ("multibrot4", FractalKind::Multibrot4, "0.0", "0.0", 0.8, 800, 0, 0, false),
            ("multibrot5", FractalKind::Multibrot5, "0.0", "0.0", 0.8, 800, 0, 0, false),
            ("tricorn", FractalKind::Tricorn, "0.0", "0.0", 0.8, 800, 0, 0, false),
            ("burning-ship", FractalKind::BurningShip, "-0.5", "-0.5", 0.7, 800, 0, 0, false),
            ("celtic", FractalKind::Celtic, "-0.5", "0.0", 0.8, 800, 0, 0, false),
            ("buffalo", FractalKind::Buffalo, "-0.5", "-0.5", 0.7, 800, 0, 0, false),
            ("phoenix", FractalKind::Phoenix, "0.0", "0.0", 0.7, 800, 0, 0, false),
            ("newton", FractalKind::Newton, "0.0", "0.0", 0.7, 400, 0, 0, false),
            // Deep mode-0 (df32 perturbation, 1e6×) views at a bisected boundary coordinate (see
            // core's dump_deep_boundary_coords). These exercise the bignum reference orbit (step_bf)
            // + series approximation + the df32-perturbation shader branch — the deep pipeline the
            // shallow overviews don't touch. Limited to the polynomial families: the abs families
            // (Burning Ship / Celtic / Buffalo) show fold glitch-speckle at deep perturbation zoom
            // (awaiting multi-reference glitch correction), and Tricorn/Phoenix need better deep
            // coordinates — a clean deep tier for those (and a mode-2 / floatexp tier) is future work.
            ("mandelbrot-1e6", FractalKind::Mandelbrot, "-7.219621882920463979621343199249635039400777157391994056859e-1", "2.406540627640154659873781066416545013133592385797331352286e-1", 1.0e6, 3000, 0, 0, false),
            ("multibrot3-1e6", FractalKind::Multibrot3, "2.19533102209775940218788168856401426185991366731348781648e-1", "7.317770073659198278104833118192370226116695264984596408352e-1", 1.0e6, 3000, 0, 0, false),
            ("multibrot4-1e6", FractalKind::Multibrot4, "2.28757960884408080137002307307431367850187620104115769219e-1", "7.625265362813602953424916065993043372187655480595946595141e-1", 1.0e6, 3000, 0, 0, false),
            ("multibrot5-1e6", FractalKind::Multibrot5, "2.320768669674853369085651557338865001525750889159483426277e-1", "7.735895565582844849904484291320284693154748744446630197764e-1", 1.0e6, 3000, 0, 0, false),
        ];
        // 1920x1080, raised from 320x240 (2026-08-22). 27x the pixels: a rendering
        // regression that survives 2M pixels is not one worth calling a golden, and the
        // old 76,800-pixel frames were coarse enough that fine filament structure fell
        // between samples entirely.
        //
        // NOT 4K, deliberately. 17 goldens at 3840x2160 is ~100-200 MB of tracked binary in
        // a PUBLIC repo and git keeps every version, so each re-bless doubles it - and
        // `--selftest` is the gate run constantly, where 108x the pixels is felt on every
        // run. 1080p buys the detection sensitivity without either cost.
        //
        // WARNING: GOLDEN_MEAN_* tolerances were calibrated at 320x240. A mean over 2M
        // pixels is a different statistic from a mean over 76,800 - a localized defect is
        // diluted 27x in the mean while maxD is unchanged. If a cross-GPU run starts
        // passing things it used to catch, the MEAN bound is why; re-derive it rather than
        // assuming it carried over.
        let (gw, gh) = (1920u32, 1080u32);
        // (name, max Δ, mean Δ, checksum, pass, reproduce, status). `status` is "" for a normal
        // compared golden (show maxΔ/meanΔ); otherwise a distinct reason (MISSING / SIZE MISMATCH /
        // RENDER ERROR) so those never masquerade as a pixel-diff failure.
        let mut goldens: Vec<(String, u32, f64, u64, bool, String, &'static str)> = Vec::new();
        // Are we on the card these goldens were blessed on? An ABSENT marker means strict — a
        // missing file must never silently loosen the release gate; it only ever loosens when we
        // positively know the hardware differs. (Goldens blessed before this file existed simply
        // stay strict until the next --bless, which is the safe direction.)
        let blessed_gpu = std::fs::read_to_string(golden_dir.join("BLESSED-GPU.txt"))
            .ok()
            .map(|s| s.trim().to_string());
        let cross_gpu = blessed_gpu
            .as_deref()
            .is_some_and(|g| g != self.gpu_name.trim());
        if !bless {
            if let Some(g) = &blessed_gpu {
                if cross_gpu {
                    eprintln!(
                        "[selftest] goldens were blessed on {g}; this is {}. Comparing with the \
                         cross-GPU tolerance (meanΔ ≤ {GOLDEN_MEAN_CROSS_GPU}) — differences \
                         within it are EXPECTED, not defects.",
                        self.gpu_name
                    );
                }
            }
        }
        for &(name, fractal, cx, cy, zoom, iter, method, palette, relief) in specs {
            // A filter matches goldens by group tag or by individual spec name
            // (`--selftest-filter multibrot3-1e6` re-renders one golden in seconds).
            if !(want("goldens") || filter.as_ref().is_some_and(|f| name.contains(f.as_str()))) {
                continue;
            }
            self.fractal = fractal;
            self.julia_mode = false;
            self.coloring.color_method = crate::ColorMethod::from_u32(method);
            self.coloring.palette_idx = palette;
            self.coloring.use_custom_palette = false;
            self.coloring.use_duotone = false;
            self.coloring.use_binary = false;
            self.coloring.cycle = 0.27;
            self.coloring.offset = 0.1;
            self.coloring.stripe_freq = 6.0;
            self.coloring.trap_type = crate::TrapType::Point; // orbit-trap shape — unused by smooth/stripe, pinned for determinism
            // Pin the palette animation OFF: active_stops() returns the *random* palette when this is
            // Random, so leaving it at whatever the loaded session had would make the goldens
            // non-deterministic (random colors) regardless of palette_idx.
            self.anim.palette_anim = crate::PaletteAnim::Off;
            self.julia_c = (0.0, 0.0); // unused (julia off) — pinned so nothing leaks from the session
            // Relief lighting is OFF for every golden except the one that exists to cover it.
            // Angle and strength are pinned rather than inherited: both are session state, and a
            // golden that renders at whatever angle the last session left would drift on every
            // bless. `light_anim` off for the same reason `palette_anim` is — it is a clock.
            self.effects.light = relief;
            self.effects.light_angle = 2.281;
            self.effects.light_height = 1.2;
            self.effects.light_anim = false;
            self.effects.de = false;
            self.render_cfg.auto_iter = false;
            self.render_cfg.max_iter = iter;
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
                "fractadyne --render --out {name}.png --fractal \"{}\" --center {cx} {cy} \
                 --zoom {zoom} --size {gw} --iter {iter} --ss 1 --method {} --palette {palette} \
                 --no-watermark",
                fractal.name(),
                crate::ColorMethod::from_u32(method).key()
            );
            let progress = std::sync::atomic::AtomicU32::new(0);
            let cancel = std::sync::atomic::AtomicBool::new(false);
            match fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel) {
                Ok(r) => {
                    // Must match `write_png` exactly (same dither, same width) or every golden
                    // fails on the conversion rather than on the render.
                    let srgb = fractadyne_export::to_srgb8_dithered(&r.pixels, r.width);
                    let sum = fnv1a64(&srgb);
                    let png_path = golden_dir.join(format!("{name}.png"));
                    if bless {
                        let _ = fractadyne_export::write_png(&png_path, r.width, r.height, &r.pixels, Some(&reproduce));
                        goldens.push((name.to_string(), 0, 0.0, sum, true, reproduce, ""));
                        // Record WHICH GPU blessed these, so a later run on different hardware can
                        // tell "this is a different card" from "this is broken" and widen its
                        // tolerance accordingly. Written once per bless, beside the images.
                        let _ = std::fs::write(golden_dir.join("BLESSED-GPU.txt"), &self.gpu_name);
                    } else {
                        let cur_path = current_dir.join(format!("{name}.png"));
                        let _ = fractadyne_export::write_png(&cur_path, r.width, r.height, &r.pixels, Some(&reproduce));
                        match fractadyne_export::read_png_rgba8(&png_path) {
                            Ok((w, h, gpx)) if w == r.width && h == r.height => {
                                let (max, mean) = img_diff(&srgb, &gpx);
                                // On the blessing GPU, hold to the strict tolerance. On any other,
                                // compare on the mean alone against the cross-GPU threshold — see
                                // the constants for why maxΔ carries no signal off-reference, and
                                // label the row so a pass is never mistaken for an exact match.
                                let (pass, status) = if cross_gpu {
                                    (mean <= GOLDEN_MEAN_CROSS_GPU, "CROSS-GPU")
                                } else {
                                    (max <= GOLDEN_MAX_STRICT && mean <= GOLDEN_MEAN_STRICT, "")
                                };
                                goldens.push((name.to_string(), max, mean, sum, pass, reproduce, status));
                            }
                            // Golden exists but was recorded at a different size — not a render diff.
                            Ok((w, h, _)) => goldens.push((
                                name.to_string(), 0, 0.0, sum, false,
                                format!("{reproduce}  [golden is {w}×{h}, expected {}×{}]", r.width, r.height),
                                "SIZE MISMATCH",
                            )),
                            // No golden on disk at the canonical path (or unreadable) — needs an initial --bless.
                            Err(_) => goldens.push((
                                name.to_string(), 0, 0.0, sum, false,
                                format!("{reproduce}  [no golden at {} — run --selftest --bless]", png_path.display()),
                                "MISSING GOLDEN",
                            )),
                        }
                    }
                }
                Err(e) => goldens.push((name.to_string(), 0, 0.0, 0, false, format!("render failed: {e}"), "RENDER ERROR")),
            }
        }

        // bench-matrix rendering-pipeline sanity check (design/bench-matrix.md): assert each
        // deterministic path's EXACT signature (mode / skip / orbit-len / eff-iter / GPU event
        // counters) matches the blessed baseline. This is the machine-independent algorithmic-
        // regression tripwire — any build touching the rendering pipeline that changes a path's
        // executed work trips it here. Runs LAST: it dirties render config (fractal / coloring /
        // deep zoom) and nothing after needs the clean hermetic state.
        if want("bench-matrix") {
            let base = anchored("benchmarks/bench-matrix-baseline.json");
            for mc in self.bench_matrix_selftest_checks(device, queue, &base) {
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "bench-matrix",
                    name: mc.name,
                    params: "path signature vs baseline".to_string(),
                    result: mc.detail,
                    threshold: "exact",
                    pass: mc.pass,
                });
            }
        }

        // A filter that matched no group and no golden runs zero checks; with the `0 == 0`
        // pass math below that would print "ALL CHECKS PASSED", exit 0, and overwrite the
        // committed report — a false green for any script keyed on the exit code (and it
        // catches `--selftest-filter` with a missing value that swallowed the next flag).
        // Fail loudly WITHOUT rewriting the report.
        if filter.is_some() && checks.is_empty() && goldens.is_empty() {
            eprintln!(
                "[selftest] --selftest-filter '{}' matched no checks or goldens — nothing ran. \
                 Use --selftest-list for the group tags.",
                filter.as_deref().unwrap_or("")
            );
            crate::exit(2);
        }

        // ---- build the human-readable + verifiable report ----
        let sys = gather_system_info(None);
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
        md.push_str(&format!("- **Config:** {cfg_echo}\n"));
        if let Some(f) = &filter {
            md.push_str(&format!(
                "- **⚠ FILTERED RUN** (`--selftest-filter {f}`): partial suite, groups share state — not a release verdict\n"
            ));
        }
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
            let verdict = if bless {
                "📷 recorded"
            } else if g.4 {
                "✅ match"
            } else if !g.6.is_empty() {
                g.6 // MISSING GOLDEN / SIZE MISMATCH / RENDER ERROR — not a pixel diff
            } else {
                "❌ differ"
            };
            md.push_str(&format!(
                "| {} | {} | {:.3} | `{:016x}` | {} | `{}` |\n",
                g.0, g.1, g.2, g.3, verdict, g.5
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
            let label = if bless { "REC " } else if g.4 { "PASS" } else { "FAIL" };
            if g.6.is_empty() {
                println!("  [{label}] golden {} — maxΔ {} meanΔ {:.2}", g.0, g.1, g.2);
            } else {
                // Distinct reason (MISSING / SIZE MISMATCH / RENDER ERROR) — not a pixel-diff fail.
                println!("  [{label}] golden {} — {}", g.0, g.6);
            }
        }
        println!("{}", "=".repeat(48));
        println!("checks {checks_pass}/{}, goldens {gold_pass}/{} — {}", checks.len(), goldens.len(),
            if ok { "OK" } else { "FAILURES PRESENT" });
        println!("report → {}\n", report_path.display());
        ok
    }
}
