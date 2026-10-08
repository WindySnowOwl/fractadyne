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
/// A built-in family's boundary point between `c = 0` (inside) and the first of `outside`·{1, 2, 4}
/// that escapes, by 48 bisections of the f64 orbit (`fractadyne_core::orbit_points`) — a view rich
/// in escaping pixels for a family nobody hand-picked one for.
fn family_boundary(formula: u32, outside: (f64, f64), budget: u32) -> Option<(f64, f64)> {
    let bail2 = 256.0 * 256.0;
    let escapes = |c: (f64, f64)| {
        let pts = fractadyne_core::orbit_points((0.0, 0.0), c, formula, budget as usize, bail2);
        pts.last().is_some_and(|z| z.0 * z.0 + z.1 * z.1 > bail2)
    };
    let mut inside = (0.0, 0.0);
    let mut outside = [1.0, 2.0, 4.0].iter().map(|k| (outside.0 * k, outside.1 * k)).find(|&o| escapes(o))?;
    if escapes(inside) {
        return None;
    }
    for _ in 0..48 {
        let mid = ((inside.0 + outside.0) * 0.5, (inside.1 + outside.1) * 0.5);
        if escapes(mid) {
            outside = mid
        } else {
            inside = mid
        }
    }
    Some(inside)
}

/// A view for the abs-family checks: a boundary point ([`family_boundary`] along a ray from 0)
/// whose view at `mag` (3/mag wide) has smooth escaping pixels to compare. A boundary can be dust
/// there — Burning Ship 3's third-quadrant ray lands in a stretch that is noise at 30×, every pixel
/// "steep" — so the rays are tried in turn, third quadrant first (where the |Im| folds engage), and
/// the first is taken whose view, sampled 24 × 24 on the CPU, has at least 15% samples that escape
/// within 2 iterations of their neighbours ONE CHECK PIXEL away (`px` of the view's width), the
/// test the check's own "steep" applies. Where no ray reaches that (a hairy boundary — Multibrot 7,
/// the Tricorns), the ray with the most such samples, if 2% or more.
fn family_view(formula: u32, mag: f64, px: u32, budget: u32) -> Option<(f64, f64)> {
    const G: usize = 24;
    let bail2 = 256.0 * 256.0;
    let dwell = |c: (f64, f64)| {
        let pts = fractadyne_core::orbit_points((0.0, 0.0), c, formula, budget as usize, bail2);
        pts.last().filter(|z| z.0 * z.0 + z.1 * z.1 > bail2).map(|_| pts.len() as i64)
    };
    let mut best: Option<(usize, (f64, f64))> = None;
    // Never along a symmetry axis: on the real axis an orbit stays real, so the |Im| folds never
    // engage, and a Tricorn's axes end in parabolic points (Tricorn 3 at 0.3849, and at 0.3849i by
    // its fourfold symmetry: interior beside stripes of escape time diverging, every pixel "steep").
    // The families' axes lie at whole multiples of 30°, 36° or 45°; rays at 3.75° past a multiple
    // of 7.5° meet none of them. Forty-eight of them: Tricorn 3's best of 24 was a view of 625
    // comparable pixels, where the direct path's f32-quantised c (two ulps a pixel at |c| ≈ 1, the
    // shader compiler folds df32 to f32) moved the mean past the bound.
    for t in (0..48).map(|k| (183.75 + 7.5 * k as f64).to_radians()) {
        let Some(at) = family_boundary(formula, (t.cos(), t.sin()), budget) else { continue };
        let span = 3.0 / mag;
        let step = span / f64::from(px);
        let smooth = (0..G * G)
            .filter(|&k| {
                let (i, j) = ((k % G) as f64, (k / G) as f64);
                let p = (at.0 + span * ((i + 0.5) / G as f64 - 0.5), at.1 + span * (0.5 - (j + 0.5) / G as f64));
                let Some(v) = dwell(p) else { return false };
                [(step, 0.0), (0.0, step)].iter().all(|&(dx, dy)| dwell((p.0 + dx, p.1 + dy)).is_some_and(|w| (w - v).abs() <= 2))
            })
            .count();
        if smooth * 100 >= G * G * 15 {
            return Some(at);
        }
        if best.is_none_or(|(n, _)| smooth > n) {
            best = Some((smooth, at));
        }
    }
    best.filter(|(n, _)| n * 100 >= G * G * 2).map(|(_, at)| at)
}

/// Deep points of the fold families, good to ~2^-112, whose 1e-30 neighbourhood is mostly STABLE
/// (a sub-pixel nudge leaves a pixel's count alone) — the core's `FOLD_DEEP_FIXTURES`, which record
/// how they were found. A fold's chaotic boundary is noise at any depth (bisected points first tried
/// had no stable pixel at all), and a check there would compare noise with noise.
const FOLD_DEEP: [(FractalKind, &str, &str); 16] = [
    (FractalKind::Celtic, "-7.523183301672266119158619702700220349889660622278404512203e-1", "-2.14835958763750446308384443362044395573177458962387293481e-1"),
    (FractalKind::Celtic3, "-1.390111097603047035481381903268406231192188759277730469272e-1", "-1.208364211005726539769261155232087101766648293103934130593e+0"),
    (FractalKind::Buffalo3, "4.608237148512231691928989032718950631004591668346324319821e-1", "1.154303315171477277006095095403463761887149114811342024179e-1"),
    (FractalKind::Buffalo5, "4.029273516218120856749052413703955216672657505935849857889e-1", "-5.821723262584372350783097339293223693418500863674671803413e-1"),
    (FractalKind::BurningShip, "1.510426975372399155276004915912049480568178198670688953146e-1", "4.221360602684940155668847406639261668137033173647796363826e-1"),
    (FractalKind::Tricorn, "-1.245181750030167260019164378722004235435653851940847122794e+0", "-2.037605720180463320107795585296334647328774720528326395773e-2"),
    (FractalKind::Buffalo, "-1.510426975372401197787866671960570444323379772291351660331e-1", "4.221360602684941728921263977360891137815433991908003217154e-1"),
    (FractalKind::BurningShip3, "-9.15383227746129862363246806792022038019889900108861939843e-1", "-1.665596356636845051404128728242821066413195064087217571441e-1"),
    (FractalKind::BurningShip4, "-1.059059912399875949806604684756823369616419345477182022818e+0", "-6.941445398749050928167618993678418040977879789587693625044e-2"),
    (FractalKind::BurningShip5, "-1.069304830077022750409631353845070556935692686852387744345e+0", "-7.008594136830775532707863552269324262159198773198077894119e-2"),
    (FractalKind::Tricorn3, "-8.301165086600040786410117965340966101130685955529819862755e-1", "-7.279926396365465868944042532991377115141048551447173006979e-1"),
    (FractalKind::Tricorn4, "-2.596843675593252253654911553479302337079929899170186367197e-1", "-5.265878052427495866136872211038079448051950717593488419814e-1"),
    (FractalKind::Tricorn5, "-9.033536580960661097439171803785319140128294786602977017732e-1", "-4.272544604520101032739291828337394992900827230127916689602e-1"),
    (FractalKind::Celtic4, "-4.234565860024477641147504048357300264219746679250861041278e-1", "-4.09820898134915444566944199392919060263917505185772053164e-1"),
    (FractalKind::Celtic5, "-2.281553024011103909013555011760308970573078526808823516802e-1", "-5.773582579137733907699085669559048183803546885637493650668e-1"),
    (FractalKind::Buffalo4, "-3.071721506556603906348160925396570822822053282996192416967e-1", "4.14173699729186134054547251993249825273527878456300080967e-1"),
];

/// Deep boundary points of Multibrot 3–8, good to ~2^-112 — the core's fixtures
/// (`MULTIBROT_DEEP_FIXTURES` in its tests, and `scorer_matches_oracle_multibrot_6_to_8`, which
/// records how they were found). A [`family_boundary`] point is f64, good to ~1e-15 and garbage
/// past that: at 1e30× its reference escapes early and every pixel with it, so a chunked render
/// agrees trivially (the corpus-07 note in "iter-chunk"). These keep their digits, and their 1e-30
/// neighbours escape hundreds to thousands of steps apart.
const MULTIBROT_DEEP: [(FractalKind, &str, &str); 6] = [
    (
        FractalKind::Multibrot3,
        "-7.542988421659047012682575462407727648742957195599927956575e-2",
        "1.150837642332408274574812674609472971463521952952769779238e+0",
    ),
    (
        FractalKind::Multibrot4,
        "6.332712744727028788597360205303212675970690061507204539421e-1",
        "2.149666311355967986884147897121404542351347657064834463711e-1",
    ),
    (
        FractalKind::Multibrot5,
        "6.077504591516768732823798182613786336218861548355426472285e-1",
        "3.983406962035189108405800236915385332818636675353539031671e-2",
    ),
    (
        FractalKind::Multibrot6,
        "6.857394901829179081616429758180505352925479174432123629563e-1",
        "4.494574077574449791211463579695434838514006741731437234958e-2",
    ),
    (
        FractalKind::Multibrot7,
        "8.476326589810416770591545692023465481479590290010075317289e-1",
        "1.686046188662333201185254044899073603644196631087163205889e-1",
    ),
    (
        FractalKind::Multibrot8,
        "7.83482963062576633425633295165828974709342271037759608728e-1",
        "2.659566285584885991743283420348254644023437952052363285336e-1",
    ),
];

/// A power family's iteration texture at `req`'s view as the CPU computes it — the f64 orbit of
/// each pixel centre by the built-in's own step and policy (escape at 256, at 128 for Multibrot 8;
/// the smooth value unclamped, as the shader's) — in `render_iter`'s layout (the smooth value at
/// every fourth float, −1 for interior). The truth a deep check judges a perturbation render by
/// where the direct path's f32 c is too coarse.
fn cpu_family_iter(req: &fractadyne_gpu::ExportRequest, formula: u32, n: u32) -> Vec<f32> {
    let nn = n as usize;
    let d = f64::from(fractadyne_core::formula::power(formula));
    let bail2 = if formula == fractadyne_core::formula::MULTIBROT8 { 128.0 * 128.0 } else { 256.0 * 256.0 };
    let centre = (req.center[0] as f64 + req.center[2] as f64, req.center[1] as f64 + req.center[3] as f64);
    let scale = 2f64.powi(req.delta_exp);
    let (sx, sy) = (req.span_mantissa.x / n as f64, req.span_mantissa.y / n as f64);
    let budget = req.max_iter as usize;
    let rows: Vec<usize> = (0..nn).collect();
    let threads = std::thread::available_parallelism().map_or(4, |t| t.get());
    let parts: Vec<Vec<f32>> = std::thread::scope(|s| {
        let handles: Vec<_> = rows
            .chunks(nn.div_ceil(threads).max(1))
            .map(|part| {
                s.spawn(move || {
                    let mut out = Vec::with_capacity(part.len() * nn * 4);
                    for &j in part {
                        for i in 0..nn {
                            let c = (
                                centre.0 + sx * ((i as f64 + 0.5) - n as f64 * 0.5) * scale,
                                centre.1 + sy * (n as f64 * 0.5 - (j as f64 + 0.5)) * scale,
                            );
                            let pts = fractadyne_core::orbit_points((0.0, 0.0), c, formula, budget, bail2);
                            let (x, y) = *pts.last().unwrap();
                            let m2 = x * x + y * y;
                            let v = if m2 > bail2 {
                                (pts.len() - 1) as f64 + 1.0 - (m2.ln() * 0.5 / 2f64.ln()).ln() / d.ln()
                            } else {
                                -1.0
                            };
                            out.extend_from_slice(&[v as f32, 0.0, 0.0, 0.0]);
                        }
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("a CPU thread panicked")).collect()
    });
    parts.concat()
}

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
            "numeric", "symmetry", "abs-family", "custom-formula", "life", "lsystem", "multibrot-sa", "bla", "aux-bla",
            "consistency", "counters", "iter-budget", "iter-chunk", "renorm", "live-split", "worker", "nr-zoom", "coords",
            "curated-poi", "ref-pick", "ref-reuse", "ref-overlap", "orbit-cache", "script", "metadata",
            "display", "catalog", "goldens", "bench-matrix", "live-res", "appearance",
            "checklist",
        ];
        /// Groups that run ONLY when named. ⛔Kept out of `GROUPS` above so a bare
        /// `--selftest` never pays for them — each costs minutes, not milliseconds.
        const OPT_IN_GROUPS: &[(&str, &str)] = &[
            (
                "deep-location",
                "the deepest tracked location (9.98e60205×) — ~9 HOURS: a ~200,000-bit orbit to a 2,000,000 ask",
            ),
            (
                "fe-df32-probe",
                "diagnostic: where df32 and floatexp disagree, which one a bignum oracle sides with (~1 min)",
            ),
        ];
        if self.selftest.list {
            println!("selftest groups (use with --selftest-filter <substr>):");
            for g in GROUPS {
                println!("  {g}");
            }
            println!("opt-in groups (run ONLY when named, and NOT part of a bare --selftest):");
            for (g, why) in OPT_IN_GROUPS {
                println!("  {g}  — {why}");
            }
            crate::exit(0);
        }
        // (A filter that runs ZERO checks is rejected AFTER the suite — see the guard just
        // before the report is written. Doing it post-hoc matches on what actually ran, so
        // it can't drift from the group/golden name lists the way a pre-flight check would.)
        let want = |tag: &str| -> bool {
            filter.as_ref().is_none_or(|f| tag.to_ascii_lowercase().contains(f.as_str()))
        };
        // ⛔⭐⭐**OPT-IN groups run ONLY when the filter NAMES them** — the opposite of `want`,
        // which runs everything when there is no filter. For a check whose cost is minutes rather
        // than milliseconds, "on unless excluded" is the wrong default: it would turn the gate
        // people run constantly into one they learn to skip, and a skipped gate is no gate.
        let opt_in = |tag: &str| -> bool {
            filter.as_ref().is_some_and(|f| tag.to_ascii_lowercase().contains(f.as_str()))
        };
        if let Some(f) = &filter {
            eprintln!("[selftest] FILTERED RUN (--selftest-filter {f}): group state is shared — use full runs for verdicts");
        }
        // Seahorse Valley — detailed at every depth tested; coordinate precise enough.
        const SX: &str = "-0.743643887037151";
        const SY: &str = "0.131825904205330";
        const N: u32 = 220;
        // Validation corpus location 07 (43 digits): structure-rich at 9.3e27×, where it escapes
        // on every pixel. Used by the df32-ceiling checks and the fe-df32-probe; see (C) below.
        const CRX: &str = "-1.178853950372678747911373866849720956148855";
        const CRY: &str = "0.1853420232408490265512092752061929308714979";

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
            const CHUNK_FAMILY_COUNTS_MIN: usize = 20;
            let mandel: &[(&str, &str, f64, u32, u32, bool, u32, &str)] = &[
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
            type ChunkCase = (FractalKind, String, String, f64, u32, u32, bool, u32, String);
            let mut cases: Vec<ChunkCase> = mandel
                .iter()
                .map(|&(x, y, m, it, ch, tr, md, d)| (FractalKind::Mandelbrot, x.into(), y.into(), m, it, ch, tr, md, d.into()))
                .collect();
            // ⭐Multibrot 3–8 (design/power-families.md, phases 2–3): the chunk passes carry their
            // arms, which must be fs_iterate's to the bit in every mode — direct and df32 at a
            // `family_view` boundary (an f64 centre is exact enough there), the rebase storm, and
            // floatexp at 1e30× on a `MULTIBROT_DEEP` point, where their mode-2 row must show BLA
            // skips as Mandelbrot's do AND escapes spread over many counts (below).
            for (kind, dx, dy) in MULTIBROT_DEEP {
                let f = kind.formula_id();
                let name = kind.name();
                let Some(at) = family_view(f, 2.0e4, N, 3_000) else {
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "IterChunk",
                        name: "chunked render is bit-identical".into(),
                        params: format!("{name}: a view"),
                        result: "no ray from 0 reaches a boundary with smooth escaping pixels at 2e4×".into(),
                        threshold: "a view to test at",
                        pass: false,
                    });
                    continue;
                };
                let (ax, ay) = (format!("{:.17}", at.0), format!("{:.17}", at.1));
                cases.push((kind, ax.clone(), ay.clone(), 2.0e3, 2_000, 137, false, 1, format!("{name} 2e3×, 2000 iter, chunk 137")));
                cases.push((kind, ax.clone(), ay.clone(), 2.0e4, 3_000, 517, false, 0, format!("mode0 {name} 2e4×, 3000 iter, chunk 517")));
                cases.push((kind, ax, ay, 2.0e4, 20_000, 700, true, 0, format!("mode0 {name} 97-sample ref (rebase storm), 20k iter, chunk 700")));
                cases.push((kind, dx.into(), dy.into(), 1.0e30, 21_000, 7_000, false, 2, format!("mode2 {name} deep boundary 1.3e30×, 21k iter, 3 passes")));
            }
            let prev_fractal = self.fractal;
            for (kind, cx, cy, mag, max_iter, chunk, truncate, want_mode, desc) in &cases {
                self.fractal = *kind;
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
                            // A Multibrot's deep row also shows the other thing a trivial agreement
                            // lacks: orbits that part — escapes over many counts.
                            let family = *kind != FractalKind::Mandelbrot;
                            let bla_ok = *want_mode != 2 || *truncate || bla > 0;
                            let mut counts: Vec<u32> =
                                a.iter().step_by(4).filter(|v| **v >= 0.0).map(|v| *v as u32).collect();
                            let escaped = counts.len();
                            counts.sort_unstable();
                            counts.dedup();
                            let spread_ok = !family || *want_mode != 2 || counts.len() >= CHUNK_FAMILY_COUNTS_MIN;
                            (
                                diffs == 0 && bla_ok && spread_ok,
                                format!(
                                    "mode {} — {diffs} texels differ (max Δ {maxd:.3e}), bla_skip {bla}, rebase {reb}{}",
                                    req.mode,
                                    if family {
                                        format!(", {escaped} escaped over {} counts, sa_skip {}", counts.len(), req.sa_skip)
                                    } else {
                                        String::new()
                                    }
                                ),
                            )
                        }
                        _ => (false, "render failed".into()),
                    }
                };
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "IterChunk",
                    name: "chunked render is bit-identical".into(),
                    params: desc.clone(),
                    result,
                    threshold: "0 texels differ (mode 2: and BLA engaged; a Multibrot's escapes over ≥ 20 counts)",
                    pass,
                });
            }
            self.fractal = prev_fractal;
        }

        // ⭐⭐(0.3.0-beta.3) THE SPLIT LIVE REFRESH. A moving df32 frame dearer than one displayed
        // frame's share renders as k passes of `fs_iterate` over `vs_split_tiles`, one checkerboard
        // set of 16-texel tiles each, composing in the live texture while the hold serves the last
        // complete frame (`MandelbrotParams::split`). Two claims, each with a vacuous way to pass:
        // - the k sets compose the frame BIT FOR BIT against one pass: a tile no set covers stays
        //   cleared and shows as a difference, and so does an edge the clamp misses — the frame is
        //   220², not a multiple of 16;
        // - ONE set alone writes exactly its share and nothing else: a `discard` in the fragment
        //   stage composed the frame just as well and cost every pass the whole frame, because a
        //   demoted helper invocation runs on through the loop. Only geometry that never reaches
        //   the rasterizer is free, and the texel count is what shows the geometry did the work.
        if want("live-split") {
            let saved = (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.coloring.color_method);
            self.render_cfg.max_iter = 20_000;
            self.render_cfg.auto_iter = false;
            self.coloring.color_method = crate::ColorMethod::Smooth;
            let bit_exact = |a: &[f32], b: &[f32]| -> usize {
                a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x.iter().zip(*y).any(|(p, q)| p.to_bits() != q.to_bits())).count()
            };
            // Seahorse Valley in df32 perturbation (the mode the split serves) and in direct mode
            // (the same `fs_iterate`, no reference): the geometry must not care which.
            for (label, mag, mode) in [("df32 1e8x", 1.0e8, 0u32), ("direct 3e3x", 3.0e3, 1u32)] {
                let mut vp = Viewport::new(N as f64, N as f64);
                vp.center_x = fractadyne_core::parse_bf(SX).unwrap();
                vp.center_y = fractadyne_core::parse_bf(SY).unwrap();
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mag));
                vp.precision = fractadyne_core::precision_for_magnification(mag);
                let mut req = self.current_export_request_for(&vp, false);
                req.width = N;
                req.height = N;
                req.ss = 1;
                let control = render(&req);
                let mut parts = Vec::new();
                let mut all_zero = control.is_some() && req.mode == mode;
                for k in [2u32, 3, 4, 8] {
                    let s = fractadyne_gpu::render_iter_split(device, queue, &req, k, None)
                        .map_err(|e| eprintln!("[selftest] GPU ERROR (render_iter_split): {e}"))
                        .ok();
                    match (&control, &s) {
                        (Some(c), Some(s)) if c.len() == s.pixels.len() => {
                            let d = bit_exact(c, &s.pixels);
                            all_zero &= d == 0;
                            parts.push(format!("k={k}: {d}"));
                        }
                        _ => {
                            all_zero = false;
                            parts.push(format!("k={k}: render failed"));
                        }
                    }
                }
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "LiveSplit",
                    name: format!("split passes compose the frame bit for bit ({label})"),
                    params: format!("{N}×{N} (not a multiple of 16), 20,000 iter, mode {}", req.mode),
                    result: format!("texels differing from one pass — {}", parts.join(", ")),
                    threshold: "0 at every k, in the named mode",
                    pass: all_zero,
                });
                // One set alone, over the cleared texture: exactly the texels of its tiles.
                let (k, j) = (4u32, 1u32);
                let expected: usize = (0..N)
                    .flat_map(|y| (0..N).map(move |x| (x, y)))
                    .filter(|&(x, y)| (x / 16 + y / 16) % k == j)
                    .count();
                let one = fractadyne_gpu::render_iter_split(device, queue, &req, k, Some(j))
                    .map_err(|e| eprintln!("[selftest] GPU ERROR (render_iter_split, one set): {e}"))
                    .ok();
                let (pass, result) = match (&one, &control) {
                    (Some(o), Some(c)) => {
                        // A written texel matches the control; an unwritten one stays all zero.
                        let written = o.pixels.chunks(4).filter(|t| t.iter().any(|v| v.to_bits() != 0)).count();
                        let stray = o
                            .pixels
                            .chunks(4)
                            .zip(c.chunks(4))
                            .enumerate()
                            .filter(|(i, (t, ctl))| {
                                let (x, y) = ((*i as u32) % N, (*i as u32) / N);
                                let mine = (x / 16 + y / 16) % k == j;
                                if mine {
                                    t.iter().zip(ctl.iter()).any(|(p, q)| p.to_bits() != q.to_bits())
                                } else {
                                    t.iter().any(|v| v.to_bits() != 0)
                                }
                            })
                            .count();
                        (
                            stray == 0 && written == expected,
                            format!("{written} texels written of {expected} in set {j} of {k}; {stray} misplaced"),
                        )
                    }
                    _ => (false, "render failed".into()),
                };
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "LiveSplit",
                    name: format!("one split set writes only its own tiles ({label})"),
                    params: format!("{N}×{N}, set {j} of {k}"),
                    result,
                    threshold: "its tiles' texels, equal to one pass; nothing else",
                    pass,
                });
            }
            // ⭐The COST claim, which neither check above can see: a `discard` in the fragment stage
            // passed both (it composed the frame and wrote only its set) and still cost every pass
            // the whole frame. At a size that fills the GPU (220² is all fixed cost), one set of four
            // must time well under every tile drawn through the SAME geometry (k = 1) — measured
            // ~0.9–1.0 with the discard, 0.47 with the tile geometry on the RTX 3080 (a quarter of
            // 1024² sits at the occupancy knee, so a set cannot reach its bare 0.25 here; the live
            // frame's sets measured 0.3–0.4). Wall clock, submission to completion, best of three:
            // the RX 6800 XT's GPU timestamps read a COLD whole pass (a new pipeline's first) at
            // 1.75 ms and a quarter at 79 (warmed they agree with the wall — the checks below).
            const SPLIT_COST_N: u32 = 1024;
            let mut vp = Viewport::new(SPLIT_COST_N as f64, SPLIT_COST_N as f64);
            vp.center_x = fractadyne_core::parse_bf(SX).unwrap();
            vp.center_y = fractadyne_core::parse_bf(SY).unwrap();
            vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (SPLIT_COST_N as f64 * 1.0e8));
            vp.precision = fractadyne_core::precision_for_magnification(1.0e8);
            let mut req = self.current_export_request_for(&vp, false);
            req.width = SPLIT_COST_N;
            req.height = SPLIT_COST_N;
            req.ss = 1;
            let best = |k: u32, j: u32| {
                (0..3)
                    .filter_map(|_| {
                        fractadyne_gpu::render_iter_split(device, queue, &req, k, Some(j))
                            .map_err(|e| eprintln!("[selftest] GPU ERROR (render_iter_split, cost): {e}"))
                            .ok()
                            .map(|r| r.iterate_ms)
                    })
                    .filter(|ms| *ms > 0.0)
                    .fold(f64::INFINITY, f64::min)
            };
            let full = best(1, 0);
            let one = best(4, 1);
            let ratio = one / full;
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "LiveSplit",
                name: "one split set costs about its share of the pass".into(),
                params: format!("{SPLIT_COST_N}×{SPLIT_COST_N} df32 1e8x, set 1 of 4, wall, best of 3"),
                result: format!("one set {one:.2} ms against every tile {full:.2} ms = {ratio:.2}"),
                threshold: "< 0.6 of every tile (0.25 = exactly its share)",
                pass: full.is_finite() && one.is_finite() && ratio < 0.6,
            });
            // ⭐The PRICE's instrument. A live refresh is priced from GPU timestamps of its own pass
            // (`MandelbrotParams::live_timing`), so they must describe that pass. On the RX 6800 XT
            // a COLD pass (a new pipeline's first) read 1.75 ms whose split twin took ~190 ms by
            // the wall clock; warmed, this check reads 1.01 (triangle) and 1.00 (tiles) there, 0.99
            // and 0.99 on the RTX 3080. Both pipelines the live view draws with — the full-screen
            // triangle and the split tile geometry — timed both ways on the same pass, median of
            // three after a warm-up: the timestamp over the wall, which also holds the submission,
            // so a faithful one sits at or a little under 1. Without timestamps the live refresh
            // never probes, and there is nothing to check.
            let median = |mut v: Vec<f64>| -> f64 {
                v.sort_by(|a, b| a.total_cmp(b));
                v.get(v.len() / 2).copied().unwrap_or(f64::NAN)
            };
            let mut walls = [f64::NAN; 2];
            for (i, (label, tiles)) in [("full-screen triangle", false), ("split tile geometry", true)].into_iter().enumerate() {
                let t = fractadyne_gpu::time_iter_pass(device, queue, &req, tiles, 3)
                    .map_err(|e| eprintln!("[selftest] GPU ERROR (time_iter_pass): {e}"))
                    .unwrap_or_default();
                let ts = median(t.iter().map(|r| r[0]).collect());
                let wall = median(t.iter().map(|r| r[1]).collect());
                walls[i] = wall;
                let ratio = ts / wall;
                let (pass, result) = if t.is_empty() {
                    (false, "render failed".to_string())
                } else if ts.is_nan() {
                    (true, format!("no GPU timestamps (wall {wall:.2} ms): the live refresh does not probe"))
                } else {
                    (
                        (0.5..=1.05).contains(&ratio),
                        format!("timestamp {ts:.2} ms against wall {wall:.2} ms = {ratio:.2}"),
                    )
                };
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "LiveSplit",
                    name: format!("GPU timestamps describe their own pass ({label})"),
                    params: format!("{SPLIT_COST_N}×{SPLIT_COST_N} df32 1e8x, one pass, median of 3"),
                    result,
                    threshold: "timestamp 0.5–1.05 of the wall",
                    pass,
                });
            }
            // …and the split refresh's geometry must not make the frame itself dearer: every tile
            // through `vs_split_tiles` against the one full-screen triangle, by the wall clock.
            let geo = walls[1] / walls[0];
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "LiveSplit",
                name: "the split tile geometry costs what the full-screen pass does".into(),
                params: format!("{SPLIT_COST_N}×{SPLIT_COST_N} df32 1e8x, every tile, wall, median of 3"),
                result: format!("tiles {:.2} ms against the triangle {:.2} ms = {geo:.2}", walls[1], walls[0]),
                threshold: "≤ 1.3",
                pass: geo.is_finite() && geo <= 1.3,
            });
            (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.coloring.color_method) = saved;
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
            // ⚠Since beta.150 a mode-2 tile is sized for GPU occupancy (1024 samples), and this
            // whole 220-px frame would be ONE tile — no cross-tile state reuse, no per-tile
            // counter epochs, nothing this case exists for. The pre-occupancy tile, 70 px
            // (sqrt(2e10 / 4M)), keeps it at 16 tiles. The occupancy tile itself is the next case.
            req.tile_px_max = Some(70);
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
                            // The tile count is part of the claim: at one tile this case tests
                            // nothing it is named after (see `tile_px_max` above).
                            diffs == 0 && a.max_dispatch_ms > 0.0 && a.tiles_total == 16,
                            format!(
                                "{diffs} texels differ; {} tiles; max dispatch {:.0}ms vs control {:.0}ms",
                                a.tiles_total, a.max_dispatch_ms, b.max_dispatch_ms
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
                let t = fractadyne_gpu::render_iter_tiled(device, queue, &req, 20_000_000_000, None, None, None)
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

                // ⭐⭐(beta.150) THE OCCUPANCY TILE AND ITS STEP-BOUNDED PASSES. The same frame
                // unpinned is ONE tile (the old nominal sizing made it 16), and its passes stop each
                // pixel after `step_cap` executed steps and resume from state. Everything the claim
                // rests on is asserted, because each has a vacuous way to pass:
                // - one tile: otherwise occupancy sizing did not engage;
                // - more passes than tiles: otherwise no pass ever stopped a running pixel, and the
                //   resume path this exists to prove never ran;
                // - every pass within `STEP_MAX_PX_STEPS`: the bound that replaces the tile budget;
                // - bit-identical to the single-dispatch control: where passes split never
                //   changes a pixel.
                // ⚠512², not the 220² above: a pass's step cap is its pixel-step budget over its
                // AREA, and at 220² (≈2.7k steps) every pixel here finished inside ONE pass — the
                // case read "1 tile, 1 passes" and failed, correctly. At 512² (≈500 steps, still
                // one occupancy tile) the slow pixels must resume.
                const OCC_N: u32 = 512;
                let mut occ_req = req.clone();
                occ_req.tile_px_max = None;
                occ_req.width = OCC_N;
                occ_req.height = OCC_N;
                let o = fractadyne_gpu::render_export(device, queue, &occ_req, &progress, &cancel)
                    .map_err(|e| eprintln!("[selftest] GPU ERROR (render_export, occupancy): {e}"))
                    .ok();
                let occ_ctl =
                    fractadyne_gpu::render_export_unchunked(device, queue, &occ_req, &progress, &cancel)
                        .map_err(|e| eprintln!("[selftest] GPU ERROR (render_export_unchunked, occupancy): {e}"))
                        .ok();
                let (pass, result) = match (&o, &occ_ctl) {
                    (Some(o), Some(b)) if o.pixels.len() == b.pixels.len() => {
                        let diffs = bit_exact(&o.pixels, &b.pixels);
                        let bound = fractadyne_gpu::STEP_MAX_PX_STEPS as u64;
                        (
                            diffs == 0
                                && o.tiles_total == 1
                                && o.chunk_passes > o.tiles_chunked
                                && o.max_dispatch_work > 0
                                && o.max_dispatch_work <= bound,
                            format!(
                                "{diffs} texels differ; {} tile(s), {} passes; largest pass {} \
                                 pixel-steps (ceiling {bound})",
                                o.tiles_total, o.chunk_passes, o.max_dispatch_work
                            ),
                        )
                    }
                    _ => (false, "render failed".into()),
                };
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "IterChunk",
                    name: "occupancy tile: step-bounded passes resume bit-identically".into(),
                    params: "corpus07 1e30x, 4M iter, 512px in one tile".into(),
                    result,
                    threshold: "0 texels differ, 1 tile, passes > tiles, pass <= ceiling",
                    pass,
                });

                // ⭐⭐PACKED TAILS (`Packer` in fractadyne-gpu's export.rs): once at most half of a
                // step-bounded tile runs, its running pixels iterate in a dense grid, and when that
                // grid thins out too they go back to the tile and pack again; the last grid is
                // written back before the resolve. The same frame with packing switched off must
                // match the packed render bit for bit, and the packed render must have packed
                // TWICE in its one tile — once proves the gather and the final write-back, the
                // second the repack between them; a tile that never thinned that far would pass
                // vacuously. 1024² is still one tile, and its smaller step cap (the pixel-step
                // ceiling over 1M samples) gives the slow pixels the passes to thin out over.
                const PACK_N: u32 = 1024;
                let mut pack_req = occ_req.clone();
                pack_req.width = PACK_N;
                pack_req.height = PACK_N;
                let packed = fractadyne_gpu::render_export(device, queue, &pack_req, &progress, &cancel)
                    .map_err(|e| eprintln!("[selftest] GPU ERROR (render_export, packed): {e}"))
                    .ok();
                fractadyne_gpu::set_tile_pack(false);
                let flat = fractadyne_gpu::render_export(device, queue, &pack_req, &progress, &cancel)
                    .map_err(|e| eprintln!("[selftest] GPU ERROR (render_export, unpacked): {e}"))
                    .ok();
                fractadyne_gpu::set_tile_pack(crate::tunables::cost().tile_pack == 1);
                let (pass, result) = match (&packed, &flat) {
                    (Some(p), Some(f)) if p.pixels.len() == f.pixels.len() => {
                        let diffs = bit_exact(&p.pixels, &f.pixels);
                        (
                            diffs == 0 && p.tiles_total == 1 && p.packs >= 2 && f.packs == 0,
                            format!(
                                "{diffs} texels differ; {} tile(s), {} passes, packed {} times (unpacked twin: {})",
                                p.tiles_total, p.chunk_passes, p.packs, f.packs
                            ),
                        )
                    }
                    _ => (false, "render failed".into()),
                };
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "IterChunk",
                    name: "packed tails: a packed tile matches its unpacked twin".into(),
                    params: "corpus07 1e30x, 4M iter, 1024px in one tile, TILE_PACK 1 vs 0".into(),
                    result,
                    threshold: "0 texels differ, 1 tile, packed at least twice",
                    pass,
                });
            }
        }

        // ⭐⭐A SECOND GPU'S SAMPLE (`gpu_worker`, design/multi-gpu-live.md L2). The live view folds
        // supersampling samples rendered on another device into its running average, so a sample
        // must not depend on the device that made it. The rig is a TWIN device on this adapter:
        // the worker's own instance, device, queue and thread, through the export renderer, must
        // give the bytes this device gives for the same jittered request. Two controls keep it
        // honest, because each claim has a vacuous way to pass:
        // - the jitter must change the picture (a twin that ignored it would agree about an
        //   unjittered frame, and every sample of a run would be the same sample);
        // - a whole-pixel jitter must be a whole-pixel SHIFT: the units and sign the live view
        //   uses (`px_offset = jitter · ss`), through the iterate AND the colour pass.
        if want("worker") {
            use std::sync::atomic::{AtomicBool, AtomicU32};
            const WW: u32 = 192;
            const WH: u32 = 120;
            let mag = 1.0e30;
            let mut vp = Viewport::new(WW as f64, WH as f64);
            vp.center_x = fractadyne_core::parse_bf(CRX).unwrap();
            vp.center_y = fractadyne_core::parse_bf(CRY).unwrap();
            vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (WH as f64 * mag));
            vp.precision = fractadyne_core::precision_for_magnification(mag);
            let saved_iter = self.render_cfg.max_iter;
            let saved_auto = self.render_cfg.auto_iter;
            let saved_method = self.coloring.color_method;
            self.render_cfg.max_iter = 20_000;
            self.render_cfg.auto_iter = false;
            self.coloring.color_method = crate::ColorMethod::Smooth;
            let mut req = self.current_export_request_for(&vp, false);
            self.render_cfg.max_iter = saved_iter;
            self.render_cfg.auto_iter = saved_auto;
            self.coloring.color_method = saved_method;
            req.width = WW;
            req.height = WH;
            req.ss = 2;
            req.jitter = [0.31, -0.22];
            let progress = AtomicU32::new(0);
            let cancel = AtomicBool::new(false);
            let bit_exact = |a: &[f32], b: &[f32]| -> usize {
                a.iter().zip(b).filter(|(x, y)| x.to_bits() != y.to_bits()).count()
            };
            let local = |r: &fractadyne_gpu::ExportRequest| {
                fractadyne_gpu::render_export(device, queue, r, &progress, &cancel)
                    .map_err(|e| eprintln!("[selftest] GPU ERROR (render_export, worker case): {e}"))
                    .ok()
            };
            let mine = local(&req).map(crate::gpu_worker::sample_of);
            let worker = self
                .render_state
                .as_ref()
                .map(|rs| rs.adapter.get_info())
                .ok_or_else(|| "no window adapter".to_string())
                .and_then(|info| crate::gpu_worker::Worker::spawn("same", &info));
            // One job of run `run`, waited for: its sample, `None` if it was cancelled, or why not.
            type Answer = Result<Option<std::sync::Arc<fractadyne_gpu::AccumSample>>, String>;
            let ask = |wk: &crate::gpu_worker::Worker, run: u64| -> Answer {
                if !wk.submit(crate::gpu_worker::Job { view: 0, run, index: 1, req: req.clone() }) {
                    return Err("the worker refused the job".into());
                }
                let t0 = std::time::Instant::now();
                loop {
                    if let Some(d) = wk.try_recv() {
                        return d.err.map_or(Ok(d.sample), Err);
                    }
                    if t0.elapsed().as_secs() > 120 {
                        return Err("no answer in 120 s".into());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            };
            let theirs = worker
                .as_ref()
                .map_err(|e| e.clone())
                .and_then(|wk| ask(wk, 1)?.ok_or_else(|| "cancelled".to_string()));
            let (pass, result) = match (&mine, &theirs) {
                (Some(m), Ok(t)) if m.rgba.len() == t.rgba.len() && [m.width, m.height] == [t.width, t.height] => {
                    let diffs = bit_exact(&m.rgba, &t.rgba);
                    (diffs == 0, format!("{diffs} of {} channels differ ({}×{})", m.rgba.len(), t.width, t.height))
                }
                (_, Err(e)) => (false, format!("the worker did not deliver: {e}")),
                _ => (false, "render failed, or the sizes differ".into()),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Worker",
                name: "a twin device's jittered sample is bit-identical".into(),
                params: format!("corpus07 1e30x, 20k iter, {WW}×{WH} ss 2, jitter (0.31, -0.22), mode {}", req.mode),
                result,
                threshold: "0 channels differ (bit for bit)",
                pass,
            });

            // Control 1: the jitter reaches the picture.
            let mut flat = req.clone();
            flat.jitter = [0.0, 0.0];
            let unjittered = local(&flat).map(crate::gpu_worker::sample_of);
            let (pass, result) = match (&mine, &unjittered) {
                (Some(m), Some(u)) if m.rgba.len() == u.rgba.len() => {
                    let diffs = bit_exact(&m.rgba, &u.rgba);
                    (diffs > 0, format!("{diffs} of {} channels differ from the unjittered frame", m.rgba.len()))
                }
                _ => (false, "render failed".into()),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Worker",
                name: "the jitter changes the sample (control)".into(),
                params: "the case above, jitter (0, 0)".into(),
                result,
                threshold: "some channels differ",
                pass,
            });

            // Control 2: jitter (1, 0) at ss 1 samples each pixel's right-hand neighbour. The edge
            // columns are left out: their colour reads a neighbour one frame has and the other not.
            let mut a = req.clone();
            a.ss = 1;
            a.jitter = [0.0, 0.0];
            let mut b = a.clone();
            b.jitter = [1.0, 0.0];
            let (ra, rb) = (local(&a), local(&b));
            let (pass, result) = match (&ra, &rb) {
                (Some(ra), Some(rb)) if ra.pixels.len() == rb.pixels.len() => {
                    let (w, h) = (ra.width as usize, ra.height as usize);
                    let (mut shifted, mut aligned) = (0usize, 0usize);
                    for y in 0..h {
                        for x in 1..w - 2 {
                            for k in 0..4 {
                                let bv = rb.pixels[(y * w + x) * 4 + k].to_bits();
                                shifted += (bv != ra.pixels[(y * w + x + 1) * 4 + k].to_bits()) as usize;
                                aligned += (bv != ra.pixels[(y * w + x) * 4 + k].to_bits()) as usize;
                            }
                        }
                    }
                    (
                        shifted == 0 && aligned > 0,
                        format!("{shifted} texels differ from the frame shifted one pixel ({aligned} from the unshifted)"),
                    )
                }
                _ => (false, "render failed".into()),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Worker",
                name: "a whole-pixel jitter is a whole-pixel shift (control)".into(),
                params: format!("corpus07 1e30x, {WW}×{WH} ss 1, jitter (1, 0) vs (0, 0)"),
                result,
                threshold: "0 differ shifted, some differ unshifted",
                pass,
            });

            // A cancelled run's job answers without a sample, even when the cancel reaches the
            // worker before the job does (the app cancels on a run's reset, queued or rendering);
            // and the cancel does not stick: the next run's job renders.
            let say = |a: &Answer| match a {
                Ok(Some(_)) => "a sample".to_string(),
                Ok(None) => "no sample".to_string(),
                Err(e) => e.clone(),
            };
            let (pass, result) = match &worker {
                Err(e) => (false, format!("no worker: {e}")),
                Ok(wk) => {
                    wk.cancel(0, 2);
                    let (cancelled, next) = (ask(wk, 2), ask(wk, 3));
                    (
                        matches!((&cancelled, &next), (Ok(None), Ok(Some(_)))),
                        format!("cancelled run's job: {}; next run's job: {}", say(&cancelled), say(&next)),
                    )
                }
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Worker",
                name: "a cancelled run's job answers without a sample; the next run's renders".into(),
                params: "the twin above, cancel(view 0, run 2) before run 2's job is sent".into(),
                result,
                threshold: "no sample, then a sample",
                pass,
            });

            // ⭐ONE STILL ACROSS DEVICES (design/multi-gpu.md Phase 2): split by tile between this
            // device and a twin, the image must be the bytes this device renders alone — the split
            // keeps the single device's tile grid (mode 2 step-bounded), and a tile's pixels depend
            // on nothing but the request. Control: both devices really rendered tiles (a split that
            // left everything to one would pass vacuously). Then the twin fails after one tile: its
            // tile goes back for this device, and the image must not change.
            let mut s = req.clone();
            s.jitter = [0.0, 0.0];
            s.tile_px_max = Some(48); // several tiles at 192×120
            let alone = local(&s);
            let twin = self
                .render_state
                .as_ref()
                .map(|rs| rs.adapter.get_info())
                .ok_or_else(|| "no window adapter".to_string())
                .and_then(|info| crate::gpu_worker::open_headless("same", &info, crate::gpu_choice::backends(), "fractadyne.selftest"));
            let split = |lose: Option<(usize, u32)>| -> Result<(fractadyne_gpu::ExportResult, Vec<fractadyne_gpu::DeviceShare>), String> {
                let t = twin.as_ref().map_err(|e| e.clone())?;
                fractadyne_gpu::render_export_multi_with(&[(device, queue), (&t.device, &t.queue)], &s, &progress, &cancel, lose)
                    .map_err(|e| e.to_string())
            };
            for (lose, name, threshold) in [
                (None, "a still split across this device and a twin is bit-identical, both rendering", "0 channels differ; both devices rendered tiles"),
                (Some((1, 1)), "a split whose twin fails after one tile hands it back, bit-identical", "0 channels differ; the twin stopped, every tile rendered"),
            ] {
                let (pass, result) = match (&alone, split(lose)) {
                    (Some(a), Ok((r, shares))) if a.pixels.len() == r.pixels.len() => {
                        let diffs = bit_exact(&a.pixels, &r.pixels);
                        let tiles: Vec<u32> = shares.iter().map(|x| x.tiles).collect();
                        let ok = diffs == 0
                            && r.tiles_total == a.tiles_total
                            && a.tiles_total > 2
                            && match lose {
                                None => tiles.iter().all(|&k| k > 0),
                                Some(_) => shares[1].error.is_some() && tiles.iter().sum::<u32>() == a.tiles_total,
                            };
                        (ok, format!("{diffs} of {} channels differ; tiles per device {tiles:?} of {}{}", a.pixels.len(), a.tiles_total,
                            shares[1].error.as_ref().map_or(String::new(), |e| format!("; the twin stopped: {e}"))))
                    }
                    (_, Err(e)) => (false, format!("the split did not render: {e}")),
                    _ => (false, "render failed, or the sizes differ".into()),
                };
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Worker",
                    name: name.into(),
                    params: format!("corpus07 1e30x, 20k iter, {WW}×{WH} ss 2, 48-px tiles, mode {}", s.mode),
                    result,
                    threshold,
                    pass,
                });
            }

            // ⭐THE LIVE RENDERER ON ANOTHER DEVICE (design/multi-gpu-live.md L3). A motion refresh
            // from a second GPU is adopted as the window's frozen frame, so it must be the frame the
            // window's device renders from the same params, and the export renderer is not that
            // frame (a separately compiled program, a few hundred pixels apart). `LiveTwin` runs the
            // window's own `prepare` headless: one on this device and one on the twin are given the
            // app's own params for a deep view (`build_params`), one pass and then a chunked walk,
            // and must agree bit for bit in both planes. The G-buffer then crosses: adopted into a
            // view on this device it must read back unchanged, and the next frame at the same key
            // must not iterate it again. Controls: the frame really is this view's (escaped texels,
            // and a one-pixel jitter changes it).
            {
                const LW: u32 = 192;
                const LH: u32 = 120;
                const LITER: u32 = 20_000;
                let saved_vp = self.viewport.clone();
                let saved = (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.coloring.color_method);
                self.render_cfg.max_iter = LITER;
                self.render_cfg.auto_iter = false;
                self.coloring.color_method = crate::ColorMethod::Smooth;
                self.viewport.set_size(LW as f64, LH as f64);
                self.viewport.set_center_log2mag(
                    fractadyne_core::parse_bf(CRX).unwrap(),
                    fractadyne_core::parse_bf(CRY).unwrap(),
                    100.0, // 1.3e30×: floatexp (mode 2), the chunked mode
                );
                self.ref_cache[0].ref_pt = None;
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
                self.ref_cache[0].ref_pt = None;

                // The frame's own params, as a whole frame at its full size (the app's walk state —
                // tile, chunk window, split, reprojection — is set per pass below).
                let mut base = pr.headless();
                base.tile = None;
                base.chunk_range = None;
                base.chunk_idx = 0;
                base.split = [1, 0];
                base.ss = 1;
                base.jitter = [0.0, 0.0];
                base.resolution = [LW, LH];
                let max_iter = base.max_iter;
                let render = |dev: &eframe::wgpu::Device, q: &eframe::wgpu::Queue, passes: &[fractadyne_gpu::MandelbrotParams]| -> Result<fractadyne_gpu::GBuffer, String> {
                    let mut t = fractadyne_gpu::LiveTwin::new(dev, q);
                    for p in passes {
                        t.frame(dev, q, p).map_err(|e| e.to_string())?;
                    }
                    t.gbuffer(dev, q, base.view_id).map_err(|e| e.to_string())
                };
                let on_twin = |passes: &[fractadyne_gpu::MandelbrotParams]| -> Result<fractadyne_gpu::GBuffer, String> {
                    let t = twin.as_ref().map_err(|e| e.clone())?;
                    render(&t.device, &t.queue, passes)
                };
                let differ = |a: &fractadyne_gpu::GBuffer, b: &fractadyne_gpu::GBuffer| -> Option<usize> {
                    ([a.width, a.height] == [b.width, b.height] && a.iter.len() == b.iter.len() && a.aux.len() == b.aux.len())
                        .then(|| bit_exact(&a.iter, &b.iter) + bit_exact(&a.aux, &b.aux))
                };
                let escaped = |g: &fractadyne_gpu::GBuffer| g.iter.chunks(4).filter(|t| t[0] >= 0.0).count();
                let one = [base.clone()];
                let here_one = render(device, queue, &one);
                // The walk's windows end INSIDE this frame's escapes (the quintiles of the one-pass
                // frame's counts), so pixels pause and resume from state; then the rest of the
                // limit, then the walk's empty tail. (Fifths of the limit ended past every pixel's
                // escape here: a walk that paused nothing, which the check below now refuses.)
                let mut counts: Vec<f32> = here_one
                    .as_ref()
                    .map(|g| g.iter.chunks(4).map(|t| t[0]).filter(|v| *v >= 0.0).collect())
                    .unwrap_or_default();
                counts.sort_by(|a, b| a.total_cmp(b));
                let mut cuts: Vec<u32> = (1..5)
                    .filter_map(|k| counts.get(counts.len() * k / 5).map(|v| *v as u32))
                    .filter(|&c| c > 0 && c < max_iter)
                    .collect();
                cuts.dedup();
                cuts.push(max_iter);
                let step = cuts[0];
                let mut walk = Vec::new();
                let mut s0 = 0;
                for e in cuts {
                    let mut p = base.clone();
                    p.chunk_range = Some([s0, e]);
                    p.chunk_idx = walk.len() as u32;
                    walk.push(p);
                    s0 = e;
                }
                let mut tail = base.clone();
                tail.chunk_range = Some([max_iter, max_iter]);
                tail.chunk_idx = walk.len() as u32;
                walk.push(tail);
                let here_walk = render(device, queue, &walk);
                let twin_one = on_twin(&one);
                let twin_walk = on_twin(&walk);
                let params = format!(
                    "corpus07 1.3e30x, {max_iter} iter, {LW}×{LH}, mode {}, the app's params; walk {:?} + tail",
                    base.mode,
                    walk.iter().filter_map(|p| p.chunk_range).filter(|r| r[0] < r[1]).map(|r| r[1]).collect::<Vec<_>>()
                );
                // A walk is only a walk if pixels pause in it: some must escape after the first window.
                let paused = |g: &fractadyne_gpu::GBuffer| g.iter.chunks(4).filter(|t| t[0] > step as f32).count();
                let walk_note = match (&here_walk, &here_one) {
                    (Ok(w), Ok(o)) => format!(
                        "; {} escaped after the first window; the walk and the one pass differ in {} channels here",
                        paused(w),
                        differ(w, o).map_or("all".to_string(), |d| d.to_string())
                    ),
                    _ => String::new(),
                };
                for (name, here, there, walked) in [
                    ("a live frame on a twin device is bit-identical (one pass)", &here_one, &twin_one, false),
                    ("a live chunked walk on a twin device is bit-identical", &here_walk, &twin_walk, true),
                ] {
                    let (pass, result) = match (here, there) {
                        (Ok(a), Ok(b)) => match differ(a, b) {
                            Some(d) => (
                                d == 0 && have_ref && base.mode == 2 && (!walked || paused(a) > 0),
                                format!(
                                    "{d} of {} channels differ; {} of {} texels escaped{}",
                                    a.iter.len() * 2,
                                    escaped(a),
                                    a.iter.len() / 4,
                                    if walked { walk_note.as_str() } else { "" }
                                ),
                            ),
                            None => (false, format!("sizes differ: {}×{} and {}×{}", a.width, a.height, b.width, b.height)),
                        },
                        (Err(e), _) => (false, format!("this device: {e}")),
                        (_, Err(e)) => (false, format!("the twin: {e}")),
                    };
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Worker",
                        name: name.into(),
                        params: params.clone(),
                        result: if have_ref { result } else { format!("{result}; the reference never arrived") },
                        threshold: "0 channels differ (iter + aux planes), mode 2; a walk pauses pixels",
                        pass,
                    });
                }

                // Control: the frame is this view's — escaped texels, and a whole-pixel jitter moves it.
                let mut shifted = base.clone();
                shifted.jitter = [1.0, 0.0];
                let here_shifted = render(device, queue, &[shifted]);
                let (pass, result) = match (&here_one, &here_shifted) {
                    (Ok(a), Ok(b)) => {
                        let d = differ(a, b).unwrap_or(0);
                        (d > 0 && escaped(a) > 0, format!("{} texels escaped; {d} channels differ from the frame jittered one pixel", escaped(a)))
                    }
                    _ => (false, "render failed".into()),
                };
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Worker",
                    name: "the twins drew this view (control)".into(),
                    params: "the one-pass frame above, and it at jitter (1, 0)".into(),
                    result,
                    threshold: "some escaped; some differ",
                    pass,
                });

                // Adoption: the twin's walk installed in a view on this device reads back unchanged;
                // and a frame adopted under these params is not iterated again at the same key (a
                // DIFFERENT frame, the jittered one, is adopted, so a re-render would show).
                let (pass, result) = match (&twin_walk, &here_shifted) {
                    (Ok(g), Ok(other)) => {
                        let adopt = |g: &fractadyne_gpu::GBuffer, then: &[fractadyne_gpu::MandelbrotParams]| -> Result<fractadyne_gpu::GBuffer, String> {
                            let mut t = fractadyne_gpu::LiveTwin::new(device, queue);
                            let mut p = base.clone();
                            p.adopt = Some(std::sync::Arc::new(g.clone()));
                            t.frame(device, queue, &p).map_err(|e| e.to_string())?;
                            for p in then {
                                t.frame(device, queue, p).map_err(|e| e.to_string())?;
                            }
                            t.gbuffer(device, queue, base.view_id).map_err(|e| e.to_string())
                        };
                        match (adopt(g, &[]), adopt(other, &one)) {
                            (Ok(a), Ok(b)) => {
                                let (da, db) = (differ(g, &a), differ(other, &b));
                                (
                                    da == Some(0) && db == Some(0),
                                    format!(
                                        "adopted walk: {} channels differ; adopted jittered frame after a frame at its key: {}",
                                        da.map_or("size differs".to_string(), |d| d.to_string()),
                                        db.map_or("size differs".to_string(), |d| d.to_string()),
                                    ),
                                )
                            }
                            (Err(e), _) | (_, Err(e)) => (false, e),
                        }
                    }
                    _ => (false, "the frames to adopt did not render".into()),
                };
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Worker",
                    name: "a twin's frame adopted here reads back unchanged and is not re-iterated".into(),
                    params: "the twin's walk above; the jittered frame adopted under the unjittered params".into(),
                    result,
                    threshold: "0 channels differ, both",
                    pass,
                });
            }
        }

        // ⭐⭐THE RENORMALIZED STEP (`fractadyne_core::RenormStep`) through every path that runs it. The
        // step is not bit-identical to the perturbation loop (it is an approximation, judged against
        // the oracle by `--renorm-audit`), but it must be bit-identical to ITSELF however a frame is
        // split: the single dispatch runs it to the end, a chunked tile may pause a pixel inside it
        // and resume it from state. At the ladder's period-15,248 minibrot (2.1e57×) the step is the
        // parent's, 953 iterations, perturbed about a 16-step u-orbit — the case it exists for.
        // Each claim asserts the step ENGAGED first: a view where it does not would pass vacuously.
        // ⚠The centre is the 2.1e57 scene's own. This case first ran at the PARENT's nucleus (the
        // 1.3e53 scene's centre) at this zoom: there the reference's c′ is 2^-143, the view is the
        // middle of the u-map's main cardioid, and every pixel settled at once — the step engaged,
        // and nothing ever paused inside it.
        if want("renorm") {
            const RX: &str = "-2.804105430550454669840777002898397927204098765083451958014259277838710256889075333271267217529135e-2";
            const RY: &str = "6.948927538996523858929943394989672880373767486737755680968675269305405323393245987015207130043252e-1";
            const RN_N: u32 = 192;
            let mag = 2.143e57;
            let mut vp = Viewport::new(RN_N as f64, RN_N as f64);
            vp.center_x = fractadyne_core::parse_bf(RX).unwrap();
            vp.center_y = fractadyne_core::parse_bf(RY).unwrap();
            vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(4.0 / (RN_N as f64 * mag));
            vp.precision = fractadyne_core::precision_for_magnification(mag);
            let saved_iter = self.render_cfg.max_iter;
            let saved_auto = self.render_cfg.auto_iter;
            let saved_method = self.coloring.color_method;
            self.render_cfg.max_iter = 457_440;
            self.render_cfg.auto_iter = false;
            self.coloring.color_method = crate::ColorMethod::Smooth;
            let mut req = self.current_export_request_for(&vp, false);
            req.width = RN_N;
            req.height = RN_N;
            req.ss = 1;
            self.render_cfg.max_iter = saved_iter;
            self.render_cfg.auto_iter = saved_auto;
            self.coloring.color_method = saved_method;
            let bit_exact = |a: &[f32], b: &[f32]| -> usize {
                a.iter().zip(b).filter(|(x, y)| x.to_bits() != y.to_bits()).count()
            };
            let engaged = req.mode == 2 && req.rn.len > 0;
            use std::sync::atomic::{AtomicBool, AtomicU32};
            let progress = AtomicU32::new(0);
            let cancel = AtomicBool::new(false);
            let a = engaged
                .then(|| fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel).ok())
                .flatten();
            let b = engaged
                .then(|| fractadyne_gpu::render_export_unchunked(device, queue, &req, &progress, &cancel).ok())
                .flatten();
            let took = a.as_ref().map_or(0, |r| r.counters[fractadyne_gpu::CTR_RENORM]);
            let (pass, result) = match (&a, &b) {
                (Some(a), Some(b)) if a.pixels.len() == b.pixels.len() => {
                    let diffs = bit_exact(&a.pixels, &b.pixels);
                    (
                        diffs == 0 && took > 0,
                        format!("{diffs} texels differ; step {} its, {took} sampled px took it", req.rn.len),
                    )
                }
                _ if !engaged => (false, format!("the step did not engage (mode {}, step {})", req.mode, req.rn.len)),
                _ => (false, "render failed".into()),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Renorm",
                name: "renormalized step: chunked export matches its single dispatch".into(),
                params: "ladder p15248 2.1e57x, 457,440 iter, 192px".into(),
                result,
                threshold: "step engaged; 0 texels differ",
                pass,
            });
            // Windows of 40,000 iterations (42 renormalized steps) end INSIDE the step for every
            // pixel that takes more, so pixels pause there and resume from state.
            let mut windows = Vec::new();
            let w = if engaged {
                fractadyne_gpu::render_iter_chunked_timed(device, queue, &req, 40_000, &mut windows).ok()
            } else {
                None
            };
            let u = engaged.then(|| fractadyne_gpu::render_iter(device, queue, &req).ok()).flatten();
            let (pass, result) = match (&w, &u) {
                (Some(w), Some(u)) if w.pixels.len() == u.pixels.len() => {
                    let diffs = bit_exact(&w.pixels, &u.pixels);
                    (diffs == 0 && windows.len() > 1, format!("{diffs} texels differ; {} windows", windows.len()))
                }
                _ if !engaged => (false, "the step did not engage".into()),
                _ => (false, "render failed".into()),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Renorm",
                name: "renormalized step: pixels paused inside it resume bit-identically".into(),
                params: "ladder p15248 2.1e57x, 40,000-iteration windows, raw".into(),
                result,
                threshold: "0 texels differ, more than one window",
                pass,
            });

            // ⭐⭐THE U-SPACE BLA (`fractadyne_core::renorm_bla_gpu`) at the ladder's period-121,984
            // minibrot (1.7e66×): the same parent step, now with a 128-step u-reference and a view
            // ~1e-13 of c′ deep, where the u-space tree skips. Its step-capped chunked passes pause
            // pixels between skips and must match the single dispatch bit for bit (both compile the
            // tree in), and the tree must have SKIPPED: fewer executed steps than the same render
            // with its tree withheld. 1024² (one tile): a first pass is priced before any is measured
            // at ~127 trips a pixel there, under what these pixels take, so pixels pause between
            // skips; at 192² every pixel finished in the first pass and nothing was resumed.
            const UN: u32 = 1024;
            const UX: &str = "-2.8041054305504546698407770028983979272040987650834522647373826069780910294289008745460822093586627899398116964168839e-2";
            const UY: &str = "6.9489275389965238589299433949896728803737674867377557295375615363433163450543217966821361887684294277878069845792365e-1";
            let umag = 1.682674e66;
            let mut uvp = Viewport::new(UN as f64, UN as f64);
            uvp.center_x = fractadyne_core::parse_bf(UX).unwrap();
            uvp.center_y = fractadyne_core::parse_bf(UY).unwrap();
            uvp.units_per_pixel = fractadyne_core::FloatExp::from_f64(4.0 / (UN as f64 * umag));
            uvp.precision = fractadyne_core::precision_for_magnification(umag);
            self.render_cfg.max_iter = 3_659_520;
            self.render_cfg.auto_iter = false;
            self.coloring.color_method = crate::ColorMethod::Smooth;
            let mut ureq = self.current_export_request_for(&uvp, false);
            ureq.width = UN;
            ureq.height = UN;
            ureq.ss = 1;
            self.render_cfg.max_iter = saved_iter;
            self.render_cfg.auto_iter = saved_auto;
            self.coloring.color_method = saved_method;
            let tree = ureq.mode == 2 && ureq.rn.len > 0 && !ureq.rn_bla.is_empty();
            let exec = |r: &fractadyne_gpu::ExportResult| {
                r.counters[fractadyne_gpu::CTR_STEP_EXEC] + (r.counters[fractadyne_gpu::CTR_STEP_EXEC + 1] << 32)
            };
            // A breadcrumb between the three 1024² renders: each stamps liveness, so a GPU shared
            // with another application (one full run took 19 s here, against 2.5) cannot read as a
            // wedged frame loop to the watchdog.
            let a = tree.then(|| fractadyne_gpu::render_export(device, queue, &ureq, &progress, &cancel).ok()).flatten();
            crate::diag::breadcrumb("selftest: u-space BLA, chunked render done".into());
            let b = tree
                .then(|| fractadyne_gpu::render_export_unchunked(device, queue, &ureq, &progress, &cancel).ok())
                .flatten();
            crate::diag::breadcrumb("selftest: u-space BLA, single dispatch done".into());
            let mut plain_req = ureq.clone();
            plain_req.rn_bla = std::sync::Arc::new(Vec::new());
            let c = tree
                .then(|| fractadyne_gpu::render_export(device, queue, &plain_req, &progress, &cancel).ok())
                .flatten();
            let (pass, result) = match (&a, &b, &c) {
                (Some(a), Some(b), Some(c)) if a.pixels.len() == b.pixels.len() => {
                    let diffs = bit_exact(&a.pixels, &b.pixels);
                    let (ea, ec) = (exec(a), exec(c));
                    (
                        diffs == 0 && a.chunk_passes > a.tiles_total && ea * 2 < ec,
                        format!(
                            "{diffs} texels differ; {} passes; {} nodes; sampled steps {ea} with the tree, {ec} without",
                            a.chunk_passes,
                            ureq.rn_bla.len() / 4
                        ),
                    )
                }
                _ if !tree => (false, format!("no u-space tree (mode {}, step {})", ureq.mode, ureq.rn.len)),
                _ => (false, "render failed".into()),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Renorm",
                name: "u-space BLA: chunked export matches its single dispatch, and skips".into(),
                params: "ladder p121984 1.7e66x, 3,659,520 iter, 1024px in one tile".into(),
                result,
                threshold: "0 texels differ, passes > tiles, steps under half without the tree",
                pass,
            });
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
        // (D5) ⭐⭐**Reference REUSE renders the same image as a fresh pick.** The GUI export
        // could skip `pick_reference` — documented as ~7 s of a ~15 s extreme-depth render — by
        // extending the reference the live view already holds, and `try_reuse_reference` exists to
        // do exactly that. Its doc claims perturbation is invariant to which valid in-view
        // reference is used, and then says the render "isn't perfectly invariant" at extreme
        // depth. Both cannot be true; nothing measured which.
        //
        // ⚠⚠**No other gate can see this.** The F3 corpus and the goldens run headless `--render`,
        // where there is no live view and `reuse` is always `None` — 38/38 maxD 0 stays green
        // however wrong the reuse path gets. This is the only thing standing under it.
        if want("ref-reuse") {
            let mag = 1.0e30;
            const RRX: &str = "-0.743643887037158704752191506114774";
            const RRY: &str = "0.131825904205311970493132056385139";
            let mut vp = Viewport::new(N as f64, N as f64);
            vp.center_x = fractadyne_core::parse_bf(RRX).unwrap();
            vp.center_y = fractadyne_core::parse_bf(RRY).unwrap();
            vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mag));
            vp.precision = fractadyne_core::precision_for_magnification(mag);
            let saved_iter = self.render_cfg.max_iter;
            let saved_auto = self.render_cfg.auto_iter;
            self.render_cfg.max_iter = 200_000;
            self.render_cfg.auto_iter = false;
            let (pass, result) = self.selfcheck_reference_reuse(device, queue, &vp, N as u32, 20_000);
            self.render_cfg.max_iter = saved_iter;
            self.render_cfg.auto_iter = saved_auto;
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "RefReuse",
                name: "a reused reference renders the same as a fresh pick".into(),
                params: "corpus07 1e30x, 200k iter, extend vs fresh pick".into(),
                result,
                threshold: "reuse engaged AND 0 texels differ",
                pass,
            });
        }

        // ⭐**The overlapped pick + build equals the sequential one** (render.rs `pick_and_build`):
        // the centre's build and series walk run beside the pick, and the pick's centre rescue
        // reads its score off that build. Every fresh reference goes through it — export and live
        // cold starts alike — so a mismatch here is a wrong picture everywhere. Each view is built
        // both ways and compared field by field; the overlap must also have ENGAGED somewhere
        // (centre build used AND the series skip taken from the parallel walk), or the identity
        // compared the sequential path with itself.
        if want("ref-overlap") {
            // (label, centre x, centre y, log2 magnification, max_iter)
            let views: [(&str, &str, &str, f64, u32); 3] = [
                (
                    // The bench 4.6e1105 centre, truncated, at 2^150: the pick is the centre and
                    // its reference is a short escaper (9,736 of 250k), so SA runs — the view
                    // that exercises the whole overlap (centre build AND parallel series skip).
                    "bench-10 centre @2^150 (centre, short escaper)",
                    "2.88551201093059871274071303800151400053376951368725797081040550949502145160849912266356e-1",
                    "1.22837636274455449095906129335365068997904564092657092682711609329657475248504117230298e-2",
                    150.0,
                    250_000,
                ),
                (
                    "bench 6.6e43 (short escaper)",
                    "-6.70209187903253724099340233845986400901890228472988919658169553187602139279518e-1",
                    "4.58060975296945872909213676106313996238241655922637652387687460587764642477807e-1",
                    145.9911716006,
                    60_000,
                ),
                (
                    "corpus07 1e30 (survivor)",
                    "-0.743643887037158704752191506114774",
                    "0.131825904205311970493132056385139",
                    30.0 * std::f64::consts::LOG2_10,
                    200_000,
                ),
            ];
            let mut engaged = false;
            let mut centre_taken = false;
            for (label, x, y, log2mag, iter) in views {
                let mag = 2f64.powf(log2mag);
                let mut vp = Viewport::new(N as f64, N as f64);
                vp.center_x = fractadyne_core::parse_bf(x).unwrap();
                vp.center_y = fractadyne_core::parse_bf(y).unwrap();
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mag));
                vp.precision = fractadyne_core::precision_for_magnification(mag);
                let (pass, result) = match self.selfcheck_ref_overlap(&vp, iter) {
                    Ok((spec, sa_from, from_build, summary)) => {
                        engaged |= spec == "centre" && sa_from == "overlap";
                        centre_taken |= from_build;
                        (true, summary)
                    }
                    Err(e) => (false, e),
                };
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "RefOverlap",
                    name: format!("overlapped pick + build is byte-identical: {label}"),
                    params: format!("{iter} iter, sequential vs overlapped fresh build"),
                    result,
                    threshold: "point, orbit, series skip, BLA, precision, tail all identical",
                    pass,
                });
            }
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "RefOverlap",
                name: "the overlap engaged (centre build + parallel series skip used)".into(),
                params: "across the views above".into(),
                result: format!("engaged={engaged}"),
                threshold: "at least one view took both from the overlap",
                pass: engaged,
            });
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "RefOverlap",
                name: "the pick took its walk of the centre from the centre build".into(),
                params: "across the views above".into(),
                result: format!("taken={centre_taken}"),
                threshold: "at least one view's phase 2 read the centre (and its samples) off the build",
                pass: centre_taken,
            });
        }

        // ⭐⭐**The on-disk orbit cache renders the same image as a fresh pick.** The cache exists
        // so an extreme location costs seconds instead of an hour to return to, and its failure
        // mode is a WRONG picture arrived at quickly: a blob that decodes into a subtly different
        // reference renders fine and wrong, and nothing downstream would notice. The codec's unit
        // tests pin the bytes; only a render can pin the picture, so this does — the same
        // identity check as `ref-reuse`, with the reference sent through the store's own write,
        // lookup and load on the way, plus an UNAIDED arm: a worker given no hint must find the
        // entry itself, because a lookup that silently misses builds fresh and renders the same
        // picture, which is the one failure the identity arm cannot see.
        //
        // At 1e30, like `ref-reuse`, so it runs in every bare `--selftest` even though the payoff
        // is at e60205 — a gate people learn to skip is no gate. Against a scratch directory; the
        // cache itself stays OFF for the rest of the run, as for every task invocation.
        if want("orbit-cache") {
            let mag = 1.0e30;
            const OCX: &str = "-0.743643887037158704752191506114774";
            const OCY: &str = "0.131825904205311970493132056385139";
            let mut vp = Viewport::new(N as f64, N as f64);
            vp.center_x = fractadyne_core::parse_bf(OCX).unwrap();
            vp.center_y = fractadyne_core::parse_bf(OCY).unwrap();
            vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mag));
            vp.precision = fractadyne_core::precision_for_magnification(mag);
            let saved_iter = self.render_cfg.max_iter;
            let saved_auto = self.render_cfg.auto_iter;
            self.render_cfg.max_iter = 200_000;
            self.render_cfg.auto_iter = false;
            let (pass, result) = self.selfcheck_orbit_cache(device, queue, &vp, N as u32, 20_000);
            self.render_cfg.max_iter = saved_iter;
            self.render_cfg.auto_iter = saved_auto;
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "OrbitCache",
                name: "an orbit from the disk cache renders the same as a fresh pick".into(),
                params: "corpus07 1e30x, 200k iter; written, found, loaded, extended; then unaided".into(),
                result,
                threshold: "found AND reuse engaged AND 0 texels differ AND the unaided worker hit",
                pass,
            });
        }

        // ⭐⭐**"The export does not match what I see."** Magnification is HEIGHT-anchored, so a
        // render on a wider canvas at the same height must contain the narrower one pixel for
        // pixel — the extra width shows MORE of the plane on either side and moves nothing. That
        // identity is what an export inherits when its aspect differs from the live canvas, and
        // it is what a centring, mapping or mirroring defect would break.
        //
        // ⚠⚠**Two depths, two thresholds, and the difference is the point.** The direct (f64)
        // path is EXACTLY width-independent and is gated at zero. The perturbation path is not:
        // measured, a handful of isolated pixels out of ~300k change with canvas width, because
        // glitch detection is per-pixel and its neighbourhood shifts. Gating that arm at zero
        // would be gating a defect we have not fixed; gating it loosely would still catch the
        // failure this check exists for, which is GROSS — a mis-centred or mirrored frame differs
        // in tens of thousands of pixels, not tens. ⛔So the loose bound is deliberately far
        // below any real misframing and far above the measured noise.
        // ⭐⭐**The EXPORT dialog's aspect override — a different code path from the one above.**
        // `selfcheck_width_independence` exercises `build_export_request`; the chosen aspect is
        // applied later, in `build_export_job`, and NOTHING covered it. That gap is how a
        // width-anchored rule (`span.y = span.x · h/w`) sat there silently cropping the top and
        // bottom off every 16:9 export from a taller window.
        //
        // ⚠Deliberately ARITHMETIC, not rendered. The property is about the complex rectangle
        // the job asks for, and a rendered comparison could not align pixels anyway once the
        // resolution changes with the aspect — so it would have to re-derive this same rule to
        // know where to look, and a check that re-derives the rule it is checking proves nothing.
        // ⭐⭐**The embedded thumbnail, end to end** — and above all, that it NEVER builds a
        // reference orbit. The first version called `current_export_request_for`, which builds
        // one synchronously; at 9.98e60205× with 2,000,000 iterations that is minutes of bignum
        // on the UI thread, and the app went "(Not Responding)" on Save .fdn. Reported from a
        // real session, and the deeper the view the more certain the hang.
        if want("view-thumb") {
            const VTX: &str = "-0.743643887037158704752191506114774";
            const VTY: &str = "0.131825904205311970493132056385139";
            let saved_vp = self.viewport.clone();
            let saved_iter = self.render_cfg.max_iter;
            let saved_auto = self.render_cfg.auto_iter;
            self.viewport.width_px = 1200.0;
            self.viewport.height_px = 900.0;
            self.viewport.center_x = fractadyne_core::parse_bf(VTX).unwrap();
            self.viewport.center_y = fractadyne_core::parse_bf(VTY).unwrap();
            self.viewport.units_per_pixel =
                fractadyne_core::FloatExp::from_f64(3.0 / (900.0 * 1.0e30));
            self.viewport.precision = fractadyne_core::precision_for_magnification(1.0e30);
            self.render_cfg.max_iter = 60_000;
            self.render_cfg.auto_iter = false;
            // ⚠⚠**`install_recompute` writes PERF counters, not just the cache.** Leaving
            // `rate_count`/`recompute_total` bumped told a later check ("unmeasured budget bounds
            // the FIRST dispatch") that a measurement had happened, and it failed — a leak in
            // this block, not a defect in that one. Second time this exact shape has bitten in
            // this file; restore everything a check MUTATES, not just what it obviously owns.
            let saved_perf = (
                self.perf.recompute_ms,
                self.perf.recompute_total,
                self.perf.rate_count,
            );
            // ⚠⚠**And the CHUNK PROGRESSION.** `install_recompute` bumps `orbit_id`, which is part
            // of `chunk_sig` — the view identity the cursor belongs to — so installing a reference
            // here silently restarts the progression for every later check. Measured: it flipped
            // the live-budget arm frame from a chunked 244-iteration dispatch to an unchunked 256,
            // pushing 3.998e8 steps to 4.195e8 and failing a bound it had nothing to do with.
            let saved_chunk = (
                self.perf.chunk_cursor,
                self.perf.chunk_idx,
                self.perf.chunk_sig,
                self.perf.chunk_pending,
                self.perf.chunk_dirty,
                self.perf.chunk_last_range,
                self.perf.chunk_inflight,
            );
            // ⛔⭐⭐**And the REFERENCE CACHE itself** — the piece that actually mattered. The
            // live-budget check keys its chunk progression on the orbit identity, so leaving the
            // cache disturbed made its arm frame dispatch an UNCHUNKED 256 iterations instead of a
            // chunked 244: 3.998e8 steps against 4.195e8, failing a bound with nothing to do with
            // thumbnails. ⭐Snapshot and put back rather than clear — the orbit is an `Arc`, so
            // this costs almost nothing.
            let saved_cache = self.ref_cache.clone();

            // ⛔⭐⭐**GUARD FIRST, with NOTHING resident.** This is the state the bug lived in:
            // a deep view whose reference cannot be borrowed. The answer must be "no thumbnail",
            // never "build one here".
            self.invalidate_refs();
            let t0 = std::time::Instant::now();
            let refused = self.render_view_thumbnail(device, queue);
            let guard_ms = t0.elapsed().as_secs_f64() * 1000.0;

            // Now give it what the LIVE view would have had, and it must borrow that.
            let mut why: Option<String> = None;
            if refused.is_some() {
                why = Some("with no resident reference it rendered anyway — it built one".into());
            }
            let vp_now = self.viewport.clone();
            let mut bytes = 0usize;
            let mut dims = (0u32, 0u32);
            let mut borrow_ms = 0.0f64;
            if false { why = None; } else if let Some(inputs) = // BISECT2
                self.export_reference_inputs_for(&vp_now, false, crate::render::IterBudget::current(self))
            {
                let res = crate::render::recompute_worker(inputs);
                self.install_recompute_for_selftest(0, res);
                let t1 = std::time::Instant::now();
                let b64 = self.render_view_thumbnail(device, queue);
                borrow_ms = t1.elapsed().as_secs_f64() * 1000.0;
                match &b64 {
                    None => {
                        if why.is_none() {
                            why = Some("a resident reference was NOT borrowed".into());
                        }
                    }
                    Some(b) => {
                        bytes = b.len();
                        let bare = format!("{}thumb={b}\n", self.view_metadata());
                        let doc = crate::export::wrap_view_text(&bare);
                        if crate::export::view_checksum_state(&doc)
                            != crate::export::ChecksumState::Match
                        {
                            why = Some("the checksum failed on a view carrying a thumbnail".into());
                        }
                        match crate::export::decode_embedded_thumbnail(&doc) {
                            None => {
                                if why.is_none() {
                                    why = Some("the embedded thumbnail did not decode".into());
                                }
                            }
                            Some((w, h, rgba)) => {
                                dims = (w, h);
                                if (w, h) != (crate::VIEW_THUMB_W, crate::VIEW_THUMB_H) {
                                    why = Some(format!("decoded {w}x{h}, expected 128x96"));
                                } else if rgba.len() != (w * h * 4) as usize {
                                    why = Some(format!("decoded {} bytes of pixels", rgba.len()));
                                } else if rgba.chunks(4).all(|p| p[..3] == rgba[..3]) {
                                    // ⚠⚠A flat thumbnail is what a broken render looks like, and it
                                    // decodes perfectly. Without this the check passes on a blank.
                                    why = Some("every pixel is the same colour — a blank render".into());
                                }
                            }
                        }
                    }
                }
            } else if why.is_none() {
                why = Some("could not build the reference the LIVE view would hold".into());
            }

            self.viewport = saved_vp;
            self.render_cfg.max_iter = saved_iter;
            self.render_cfg.auto_iter = saved_auto;
            self.ref_cache = saved_cache;
            self.perf.recompute_ms = saved_perf.0;
            self.perf.recompute_total = saved_perf.1;
            self.perf.rate_count = saved_perf.2;
            self.perf.chunk_cursor = saved_chunk.0;
            self.perf.chunk_idx = saved_chunk.1;
            self.perf.chunk_sig = saved_chunk.2;
            self.perf.chunk_pending = saved_chunk.3;
            self.perf.chunk_dirty = saved_chunk.4;
            self.perf.chunk_last_range = saved_chunk.5;
            self.perf.chunk_inflight = saved_chunk.6;
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "View format",
                name: "a thumbnail borrows the live reference, never builds one".into(),
                params: "1e30x, 128x96, ss=2; refused with none resident, then borrowed".into(),
                result: why.clone().unwrap_or_else(|| {
                    format!(
                        "refused in {guard_ms:.1}ms; borrowed and rendered {}x{} in {borrow_ms:.0}ms; \
                         {:.1} KB of base64",
                        dims.0,
                        dims.1,
                        bytes as f64 / 1024.0
                    )
                }),
                threshold: "None when nothing is resident; a real 128x96 when it is",
                pass: why.is_none(),
            });
        }
        // ⭐⭐**THE DEEPEST LOCATION WE TRACK — 9.98e60205×, and OPT-IN.**
        //
        // ⛔Not in the default sweep, deliberately. `--selftest` is the gate run constantly;
        // a 60,231-digit centre needs a ~200,000-bit reference orbit (measured: ~9 hours to a
        // 2,000,000 ask), and paying for that on every run would make the gate something people
        // skip. Run it with `--selftest-filter deep-location` when the deep pipeline is what
        // changed.
        //
        // ⚠⚠**It asserts DETERMINISM and LIVENESS, not a blessed hash.** Deep floatexp output
        // is hardware-dependent (an RTX 3080 renders all-black where a 3070 renders detail —
        // an open bug), so pinning pixels would fail honestly-different GPUs and teach everyone
        // to ignore it. What IS portable: the same GPU must render the same thing twice, and
        // it must render SOMETHING. The all-black failure is exactly the shape a flatness test
        // catches, and it is the one that has actually happened.
        if opt_in("deep-location") {
            const DEEP_FDN: &str = "validation/spiral-9.98e60205.fdn";
            let path = anchored(DEEP_FDN);
            let saved_vp = self.viewport.clone();
            let saved_iter = self.render_cfg.max_iter;
            let saved_auto = self.render_cfg.auto_iter;
            let saved_cache = self.ref_cache.clone();
            let mut why: Option<String> = None;
            let mut note = String::new();

            match std::fs::read_to_string(&path) {
                // ⚠LOUD, not skipped: a missing data file must not quietly shrink the suite.
                Err(e) => why = Some(format!("{} unreadable: {e}", path.display())),
                Ok(text) => {
                    let report = self.load_view_metadata(&text);
                    let l2 = self.viewport.log2_magnification();
                    if let Some(n) = report.note() {
                        why = Some(format!("the location did not load cleanly: {n}"));
                    } else if !(199_990.0..200_010.0).contains(&l2) {
                        // The depth is the whole point of this fixture; if it did not survive
                        // the load, everything below would be testing a shallower view.
                        why = Some(format!("loaded at log2mag {l2:.1}, expected ~200000"));
                    } else {
                        self.render_cfg.max_iter = 2_000_000;
                        self.render_cfg.auto_iter = false;
                        let n = 96u32;
                        // ⭐⭐**Build the orbit ONCE, render from it twice.** A ~200,000-bit
                        // reference at two million iterations is minutes of arbitrary-precision
                        // arithmetic; building it per render doubled the slowest check in the
                        // suite and bought nothing, because what is under test here is whether
                        // the SHADER is deterministic at extreme depth.
                        //
                        // ⚠So this does NOT cover reference-build determinism — real coverage it
                        // does not provide, and not free to add: it would mean paying that build
                        // twice. The F3 corpus and the ref-reuse check cover the orbit at depths
                        // where the cost is bearable.
                        let vp_now = self.viewport.clone();
                        // ⭐⭐**Announce it.** This one step runs for HOURS, and a check that
                        // prints nothing for that long is indistinguishable from a hung one —
                        // which is how people learn to kill a gate instead of waiting for it.
                        //
                        // ⚠The first banner said "30-60+ minutes" — the author's LIVE experience,
                        // where the orbit stops at `LIVE_REF_CAP`. This check builds EXPORT-grade
                        // to the full ask, and that was measured 2026-09-08 (`FRACTADYNE_TRACE=ref`):
                        // candidate scoring 5.77 h (101 survivors) + orbit 3.02 h (escaped at
                        // 1,645,896) + BLA 2 s = **8.8 hours** on the author's machine.
                        eprintln!(
                            "[selftest] deep-location: building a ~200,000-bit reference orbit at \
                             9.98e60205x ({} iterations). EXPECT ~9 HOURS (measured: pick 5.8 h + \
                             orbit 3.0 h).",
                            self.render_cfg.max_iter
                        );
                        let t0 = std::time::Instant::now();
                        let built = self
                            .export_reference_inputs_for(
                                &vp_now,
                                false,
                                crate::render::IterBudget::current(self),
                            )
                            .map(crate::render::recompute_worker);
                        let build_ms = t0.elapsed().as_secs_f64() * 1000.0;
                        let (mut a, mut b) = (None, None);
                        let mut orbit_len = 0u32;
                        match built {
                            None => why = Some("no reference inputs for this view".into()),
                            Some(res) => {
                                orbit_len = res.orbit_len;
                                self.install_recompute_for_selftest(0, res);
                                // Both renders BORROW that orbit, so the time below is the
                                // shader alone.
                                a = self.selfcheck_deep_render(device, queue, n);
                                b = self.selfcheck_deep_render(device, queue, n);
                            }
                        }
                        match (a, b) {
                            (Some(a), Some(b)) => {
                                let diffs = a
                                    .iter()
                                    .zip(b.iter())
                                    .filter(|(x, y)| x.to_bits() != y.to_bits())
                                    .count();
                                // ⚠⚠A uniformly flat frame is what the all-black failure looks
                                // like, and two flat frames agree perfectly. Determinism alone
                                // would pass on it.
                                let flat = a.chunks(4).all(|px| px[..3] == a[..3]);
                                let lo = a.iter().step_by(4).cloned().fold(f32::MAX, f32::min);
                                let hi = a.iter().step_by(4).cloned().fold(f32::MIN, f32::max);
                                if diffs != 0 {
                                    why = Some(format!("{diffs} of {} texels differ between two identical renders", a.len()));
                                } else if flat {
                                    why = Some(format!("every texel is the same colour (red {lo:.3}..{hi:.3}) — a blank render"));
                                } else {
                                    note = format!(
                                        "{n}×{n} twice, bit-identical; red spans {lo:.3}..{hi:.3}; \
                                         orbit {orbit_len} built in {:.1}s",
                                        build_ms / 1000.0
                                    );
                                }
                            }
                            _ => why = Some("the deep render failed".into()),
                        }
                    }
                }
            }

            self.viewport = saved_vp;
            self.render_cfg.max_iter = saved_iter;
            self.render_cfg.auto_iter = saved_auto;
            self.ref_cache = saved_cache;
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Deep location",
                name: "the deepest tracked location renders, twice the same".into(),
                params: format!("{DEEP_FDN}, 9.98e60205×, 2,000,000 iter, 96×96"),
                result: why.clone().unwrap_or(note),
                threshold: "loads clean at ~2^200000; two renders bit-identical; not blank",
                pass: why.is_none(),
            });
        }

        if want("export-contain") {
            let saved_w = self.export.width;
            let saved_aspect = self.export.aspect.clone();
            let saved_dual = self.dual;
            self.dual = false;
            self.export.width = 1920;
            // ⛔⭐⭐**PIN THE CANVAS — the first version of this check did not, and it PASSED
            // against the very width-anchored rule it was written to catch.** Whether the old
            // rule crops depends entirely on the ambient window aspect: `span_y = span_x/aspect`
            // shrinks the vertical span only when the export is WIDER than the window, so a
            // session that happened to boot wide made every tested aspect narrower, every
            // assertion vacuous, and the check green against a known defect.
            let (saved_cw, saved_chh) = (self.viewport.width_px, self.viewport.height_px);
            self.viewport.width_px = 1200.0;
            self.viewport.height_px = 900.0;
            let (cw, ch) = (self.viewport.width_px, self.viewport.height_px);
            let canvas_aspect = cw / ch;
            let (base_x, base_y) = {
                let sm = self.viewport.gpu_scale().span_mantissa;
                (sm.x, sm.y)
            };
            let aspect_of = |key: &str, fallback: f64| -> f64 {
                if key == "window" {
                    fallback
                } else {
                    crate::EXPORT_ASPECTS
                        .iter()
                        .find(|(k, _)| *k == key)
                        .map(|(_, r)| *r)
                        .unwrap_or(fallback)
                }
            };
            let (mut wider, mut narrower) = (0usize, 0usize);
            let mut worst: Option<String> = None;
            let mut checked = 0usize;
            for key in ["window", "16:9", "2:1", "1:1", "9:16", "32:9"] {
                // ⚠⚠**A key that is not in the table silently becomes the WINDOW aspect** — both
                // here and in `export_height`. The first draft asked for "21:9", which the table
                // spells "64:27", so that row tested nothing at all and the branch counter was the
                // only thing that noticed. Refuse the typo instead of absorbing it.
                if key != "window" && !crate::EXPORT_ASPECTS.iter().any(|(k, _)| *k == key) {
                    worst = Some(format!("{key:?} is not an EXPORT_ASPECTS key"));
                    break;
                }
                self.export.aspect = key.to_string();
                let h = self.export_height();
                let job = self.build_export_job();
                let crate::ExportJob::Single(req) = &job else {
                    worst = Some("dual job from a single view".into());
                    break;
                };
                let (sx, sy) = (req.span_mantissa.x, req.span_mantissa.y);
                let mut fail = |why: String| {
                    if worst.is_none() {
                        worst = Some(format!("{key}: {why}"));
                    }
                };
                // 1. CONTAINS the window view — neither axis may shrink. This is the whole point.
                if sx < base_x * (1.0 - 1.0e-9) || sy < base_y * (1.0 - 1.0e-9) {
                    fail(format!(
                        "CROPS: span {sx:.6}x{sy:.6} vs window {base_x:.6}x{base_y:.6}"
                    ));
                }
                // 2. TIGHT — the binding axis is equal, not merely >=. A rule that grew both
                //    axes would also "contain", and would zoom out for no reason.
                let tight = (sx - base_x).abs() <= base_x * 1.0e-9
                    || (sy - base_y).abs() <= base_y * 1.0e-9;
                if !tight {
                    fail(format!("not tight: {sx:.6}x{sy:.6} vs {base_x:.6}x{base_y:.6}"));
                }
                // 3. ISOTROPIC texels — the other way to get this wrong is a stretched fractal.
                let (stepx, stepy) = (sx / req.width.max(1) as f64, sy / h.max(1) as f64);
                if (stepx - stepy).abs() > stepx.abs() * 1.0e-9 {
                    fail(format!("anisotropic texels: {stepx:.9} vs {stepy:.9}"));
                }
                // 4. "Match window" must be EXACTLY an identity, or the default drifts.
                if key == "window"
                    && ((sx - base_x).abs() > base_x * 1.0e-12
                        || (sy - base_y).abs() > base_y * 1.0e-12)
                {
                    fail(format!("window aspect is not an identity: {sx:.9}x{sy:.9}"));
                }
                // ⭐Record which BRANCH this aspect took, so the check can prove below that it
                // exercised both. Contain and the rule it replaced agree on every aspect
                // narrower than the window; only the wider ones discriminate.
                if aspect_of(key, canvas_aspect) > canvas_aspect * (1.0 + 1.0e-9) {
                    wider += 1;
                } else if aspect_of(key, canvas_aspect) < canvas_aspect * (1.0 - 1.0e-9) {
                    narrower += 1;
                }
                checked += 1;
            }
            self.export.width = saved_w;
            self.export.aspect = saved_aspect;
            self.dual = saved_dual;
            self.viewport.width_px = saved_cw;
            self.viewport.height_px = saved_chh;
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Framing",
                name: "an export CONTAINS the window view at any aspect".into(),
                params: format!("window {cw:.0}x{ch:.0} (aspect {canvas_aspect:.3}), {checked} aspects"),
                result: worst.clone().unwrap_or_else(|| {
                    format!(
                        "{checked} aspects ({wider} wider, {narrower} narrower than the window): \
                         contained, tight, isotropic; window = identity"
                    )
                }),
                threshold: "no axis shrinks; binding axis exact; isotropic; BOTH branches tried",
                // ⚠⚠The branch counts are part of the VERDICT, not decoration: without a wider
                // aspect in the set this check cannot fail, whatever the rule under it does.
                pass: worst.is_none() && checked == 6 && wider >= 2 && narrower >= 2,
            });
        }

        if want("width-independence") {
            const WIX: &str = "-0.743643887037158704752191506114774";
            const WIY: &str = "0.131825904205311970493132056385139";
            let (h, narrow, wide) = (128u32, 160u32, 240u32);
            let saved_iter = self.render_cfg.max_iter;
            let saved_auto = self.render_cfg.auto_iter;
            self.render_cfg.auto_iter = false;
            for (label, mag, iter, tol, bound) in [
                ("direct f64 (1e2x)", 1.0e2_f64, 2_000u32, 0usize, "0 texels differ"),
                (
                    "perturbation (1e30x)",
                    1.0e30_f64,
                    60_000u32,
                    64usize,
                    "<= 64 texels differ (measured glitch noise; a misframing differs in 10,000s)",
                ),
            ] {
                let mut vp = Viewport::new(wide as f64, h as f64);
                vp.center_x = fractadyne_core::parse_bf(WIX).unwrap();
                vp.center_y = fractadyne_core::parse_bf(WIY).unwrap();
                // ⚠upp from the HEIGHT — the anchor the whole check rests on.
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (h as f64 * mag));
                vp.precision = fractadyne_core::precision_for_magnification(mag);
                self.render_cfg.max_iter = iter;
                let (diffs, compared, note) =
                    self.selfcheck_width_independence(device, queue, &vp, h, narrow, wide);
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Framing",
                    name: format!("a wider canvas contains the narrower one — {label}"),
                    params: format!(
                        "{narrow}px vs {wide}px at h={h}, {iter} iter, {compared} texels compared"
                    ),
                    result: note,
                    threshold: bound,
                    pass: diffs <= tol,
                });
            }
            self.render_cfg.max_iter = saved_iter;
            self.render_cfg.auto_iter = saved_auto;
        }

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

        // ---- fe-df32-probe (OPT-IN, diagnostic) ----
        // Checks (B) and (C2) below can say that df32 (mode 0) and floatexp (mode 2) DISAGREE on a
        // view; they cannot say which one is wrong. This renders the same two views both ways and
        // asks the independent bignum oracle about the pixels where the paths differ by more than
        // 2 iterations (the pixels those checks count), plus a control sample where they agree.
        // Written for the RX 6800 XT under Linux/Mesa RADV, where beta.116 measured 6.4% of pixels
        // differing at 1e10× and 4.4% at 9.3e27× (RTX 3080: 0; the same card under Windows: 0.5%).
        // ⚠On a card where the paths agree exactly (the RTX 3080) there is nothing to arbitrate:
        // the "where they differ" rows sample nothing and say so, and only the control, which
        // proves the oracle and the pixel-to-coordinate mapping line up, carries a verdict.
        if opt_in("fe-df32-probe") {
            const DIFF_SAMPLES: usize = 160;
            const CTRL_SAMPLES: usize = 40;
            let nn = N as usize;
            let views: [(&str, &str, &str, f64); 2] = [
                ("seahorse 1e10× (check B)", SX, SY, 1.0e10),
                ("corpus loc 07 9.3e27× (check C2)", CRX, CRY, 7.0e27), // 3/4-scaled, see `make`
            ];
            for (label, cx_s, cy_s, mag) in views {
                let mut a = make(self, cx_s, cy_s, mag);
                a.mode = 0;
                let mut b = a.clone();
                b.mode = 2;
                let (Some(aa), Some(bb)) = (render(&a), render(&b)) else {
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Probe",
                        name: format!("fe-df32-probe: {label}"),
                        params: "render mode 0 and mode 2".into(),
                        result: "render failed".into(),
                        threshold: "both render",
                        pass: false,
                    });
                    continue;
                };
                let dw = |px: &[f32], i: usize, j: usize| px[(j * nn + i) * 4];
                // The `oracle` closure's ill-conditioning test, applied to EITHER render: a
                // 4-neighbour flips interior/exterior or jumps by more than 2 iterations.
                let steep = |i: usize, j: usize| -> bool {
                    [aa.as_slice(), bb.as_slice()].iter().any(|&px| {
                        let g = dw(px, i, j);
                        [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)].iter().any(|&(di, dj)| {
                            let (ni, nj) = (i as isize + di, j as isize + dj);
                            if ni < 0 || nj < 0 || ni as usize >= nn || nj as usize >= nn {
                                return false;
                            }
                            let gn = dw(px, ni as usize, nj as usize);
                            (g < 0.0) != (gn < 0.0) || (g >= 0.0 && gn >= 0.0 && (g - gn).abs() > 2.0)
                        })
                    })
                };
                // Differ = what (B)/(C2) count (>2 iterations, or interior against escaped).
                // Agree = both interior, or within half an iteration.
                let (mut differ, mut agree) = (Vec::new(), Vec::new());
                for j in 0..nn {
                    for i in 0..nn {
                        let (ga, gb) = (dw(&aa, i, j), dw(&bb, i, j));
                        let both_out = ga >= 0.0 && gb >= 0.0;
                        if (ga < 0.0) != (gb < 0.0) || (both_out && (ga - gb).abs() > 2.0) {
                            differ.push((i, j));
                        } else if !both_out || (ga - gb).abs() < 0.5 {
                            agree.push((i, j));
                        }
                    }
                }
                // Evenly spread over the frame, not the first k in scan order.
                let spread = |v: &[(usize, usize)], k: usize| -> Vec<(usize, usize)> {
                    if v.len() <= k {
                        v.to_vec()
                    } else {
                        (0..k).map(|t| v[t * v.len() / k]).collect()
                    }
                };
                // Same pixel-to-coordinate mapping, bailout and tolerance as `oracle`, with the
                // renders' own iteration cap. One oracle evaluation per pixel serves both paths.
                let prec = fractadyne_core::precision_for_magnification(mag);
                let cx = fractadyne_core::parse_bf(cx_s).unwrap();
                let cy = fractadyne_core::parse_bf(cy_s).unwrap();
                let step = (3.0 / mag) / N as f64;
                let half = N as f64 / 2.0;
                let max = a.max_iter;
                let truth = |i: usize, j: usize| {
                    let cre = fractadyne_core::add_f64(&cx, ((i as f64 + 0.5) - half) * step, prec);
                    let cim = fractadyne_core::add_f64(&cy, (half - (j as f64 + 0.5)) * step, prec);
                    fractadyne_core::naive_dwell_bf(&cre, &cim, max, 65536.0, prec)
                };
                let hit = |g: f32, t: Option<(u32, f32)>| match (g >= 0.0, t) {
                    (false, None) => true,
                    (true, Some((_, s))) => (g - s).abs() < 0.75,
                    _ => false,
                };
                // [smooth, steep] × [df32 only, floatexp only, both, neither] right.
                let mut d = [[0u32; 4]; 2];
                for (i, j) in spread(&differ, DIFF_SAMPLES) {
                    let t = truth(i, j);
                    let k = match (hit(dw(&aa, i, j), t), hit(dw(&bb, i, j), t)) {
                        (true, false) => 0,
                        (false, true) => 1,
                        (true, true) => 2,
                        (false, false) => 3,
                    };
                    d[steep(i, j) as usize][k] += 1;
                }
                // Control on smooth pixels only: a steep pixel is ill-conditioned for any renderer.
                let (mut cn, mut c0, mut c2) = (0u32, 0u32, 0u32);
                for (i, j) in spread(&agree, CTRL_SAMPLES * 4)
                    .into_iter()
                    .filter(|&(i, j)| !steep(i, j))
                    .take(CTRL_SAMPLES)
                {
                    let t = truth(i, j);
                    cn += 1;
                    c0 += hit(dw(&aa, i, j), t) as u32;
                    c2 += hit(dw(&bb, i, j), t) as u32;
                }
                let [sm, st] = d;
                let (sn, tn) = (sm.iter().sum::<u32>(), st.iter().sum::<u32>());
                let pct = differ.len() as f64 * 100.0 / (nn * nn) as f64;
                eprintln!(
                    "[fe-df32-probe] {label}: {} of {} px differ ({pct:.2}%), {max} iter. Right where \
                     they differ (smooth | steep): df32 only {}|{}, floatexp only {}|{}, both {}|{}, \
                     neither {}|{}",
                    differ.len(),
                    nn * nn,
                    sm[0],
                    st[0],
                    sm[1],
                    st[1],
                    sm[2],
                    st[2],
                    sm[3],
                    st[3],
                );
                // ⚠With no smooth samples there is NO verdict, and the row must not read as one.
                // On the RX 6800 XT under Linux every differing pixel sampled was steep, and a bare
                // "PASS 0/0 smooth" hid the actual finding: the oracle sided with NEITHER path at
                // ~90% of them. The row stays a pass (nothing was judged) but says so first.
                for (path, only) in [("df32 (mode 0)", 0usize), ("floatexp (mode 2)", 1usize)] {
                    let (rs, rt) = (sm[only] + sm[2], st[only] + st[2]);
                    let judged = if sn == 0 { "NOT JUDGED (no smooth samples): " } else { "" };
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Probe",
                        name: format!("fe-df32-probe: {path} vs bignum where the paths differ — {label}"),
                        params: format!(
                            "{} of {} px differ ({pct:.2}%); sampled {sn} smooth + {tn} steep",
                            differ.len(),
                            nn * nn
                        ),
                        result: format!("{judged}oracle agrees on {rs}/{sn} smooth, {rt}/{tn} steep"),
                        threshold: "≥90% of smooth samples (none sampled: nothing to arbitrate)",
                        pass: sn == 0 || rs as f64 >= 0.9 * sn as f64,
                    });
                }
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Probe",
                    name: format!("fe-df32-probe: control, both paths vs bignum where they agree — {label}"),
                    params: format!("{cn} smooth samples"),
                    result: format!("df32 {c0}/{cn}, floatexp {c2}/{cn}"),
                    threshold: "≥90% each: the oracle and the pixel mapping line up",
                    pass: cn > 0 && c0 as f64 >= 0.9 * cn as f64 && c2 as f64 >= 0.9 * cn as f64,
                });
            }
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
            // takes ~986k rebases, so the oracle checks real dwell values (CRX/CRY, defined with
            // SX/SY at the top).
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
                    let tiled = fractadyne_gpu::render_iter_tiled(device, queue, &g, 2_000_000_000, None, None, None)
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
                    let full = fractadyne_gpu::render_iter_tiled(device, queue, &g, 2_000_000_000, None, None, None)
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
            // ⛔⭐A glitch-corrected export keeps its supersampling. Correction runs for every
            // non-holomorphic family (and every Julia view), and it used to iterate and colour at ONE
            // sample a pixel whatever the export asked for: a `--ss 2` Burning Ship came out at 1×,
            // and nothing said so. The corrected export at ss=2 must match the plain export at ss=2
            // (the same `fs_color` taps; correction only touches flagged pixels) far more closely
            // than the plain export at ss=1 does — a corrected frame that ignored ss would sit on
            // the ss=1 one instead. The ss=1→2 difference is also the proof the view has sub-pixel
            // detail for supersampling to change.
            // ⚠The view is the antenna's armada at 25×, structured detail in the direct path, where
            // the corrected export is its supersampled base frame coloured — the code that lost
            // `ss`. A deep Burning Ship view is chaotic dust (the abs-family view below is a flat
            // field at 2,000 iterations, ss changes nothing), and the correction loop may move a
            // chaotic pixel legitimately, so a deep view would test the noise, not the plumbing.
            {
                use std::sync::atomic::{AtomicBool, AtomicU32};
                let saved_correct = self.render_cfg.glitch_correct;
                self.render_cfg.glitch_correct = true;
                self.fractal = FractalKind::BurningShip;
                let mag = 25.0;
                let mut vp = Viewport::new(N as f64, N as f64);
                vp.center_x = fractadyne_core::parse_bf("-1.755").unwrap();
                vp.center_y = fractadyne_core::parse_bf("-0.03").unwrap();
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mag));
                vp.precision = fractadyne_core::precision_for_magnification(mag);
                let mut req2 = self.current_export_request_for(&vp, false);
                req2.width = N;
                req2.height = N;
                req2.ss = 2;
                let mut req1 = req2.clone();
                req1.ss = 1;
                let (progress, cancel) = (AtomicU32::new(0), AtomicBool::new(false));
                let plain = |r: &fractadyne_gpu::ExportRequest| {
                    fractadyne_gpu::render_export(device, queue, r, &progress, &cancel)
                        .map_err(|e| eprintln!("[selftest] GPU ERROR (render_export): {e}"))
                        .ok()
                };
                let applies = self.correction_wanted(false);
                let corrected = self.render_export_corrected(
                    device, queue, &vp, false, N, N, Some(&req2), crate::render::CorrectionBudget::UNBOUNDED,
                );
                let (p1, p2) = (plain(&req1), plain(&req2));
                // Mean absolute RGB difference per pixel, linear 0–1.
                let mean_d = |a: &[f32], b: &[f32]| -> f64 {
                    let s: f64 = a
                        .chunks_exact(4)
                        .zip(b.chunks_exact(4))
                        .map(|(x, y)| (0..3).map(|c| (x[c] - y[c]).abs() as f64).sum::<f64>() / 3.0)
                        .sum();
                    s / (a.len() / 4).max(1) as f64
                };
                let (pass, result) = match (&corrected, &p1, &p2) {
                    (Some(c), Some(p1), Some(p2)) if c.pixels.len() == p2.pixels.len() && p1.pixels.len() == p2.pixels.len() => {
                        let to_ss2 = mean_d(&c.pixels, &p2.pixels);
                        let ss_effect = mean_d(&p1.pixels, &p2.pixels);
                        (
                            applies && c.ss == 2 && ss_effect > 1.0e-3 && to_ss2 < 0.1 * ss_effect,
                            format!(
                                "corrected vs plain ss=2: mean Δ {to_ss2:.5}; plain ss=1 vs ss=2: {ss_effect:.5}; corrected reports ss={}",
                                c.ss
                            ),
                        )
                    }
                    _ => (
                        false,
                        format!(
                            "a render failed or the sizes differ (corrected {}, plain ss=1 {}, plain ss=2 {})",
                            corrected.is_some(),
                            p1.is_some(),
                            p2.is_some()
                        ),
                    ),
                };
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Glitch",
                    name: "a corrected export keeps its supersampling".into(),
                    params: format!("Burning Ship {mag}× at -1.755, -0.03, {N}×{N}, ss=2, mode {}, correction applies: {applies}", req2.mode),
                    result,
                    threshold: "corrected ss=2 within a tenth of the plain ss=1→2 difference of the plain ss=2 export, which must exceed 0.001",
                    pass,
                });
                self.render_cfg.glitch_correct = saved_correct;
            }

            // (family, center, mag) — boundary-detail regions rich in escaping pixels. The power
            // families (design/power-families.md B3) each on its own boundary, at a view with smooth
            // escaping pixels to compare (`family_view`).
            let mut abs_cases: Vec<(FractalKind, String, String, f64)> = vec![
                (FractalKind::BurningShip, "-1.7548".into(), "-0.0312".into(), 1.0e5),
                (FractalKind::Celtic, "-1.2566".into(), "0.0480".into(), 1.0e5),
                (FractalKind::Buffalo, "-1.7548".into(), "-0.0312".into(), 1.0e5),
            ];
            for kind in FractalKind::ALL.into_iter().filter(|k| k.power_family().is_some()) {
                match family_view(kind.formula_id(), 1.0e5, N, self.render_cfg.max_iter) {
                    Some(at) => abs_cases.push((kind, format!("{:.17}", at.0), format!("{:.17}", at.1), 1.0e5)),
                    None => push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Abs-family deep zoom",
                        name: format!("{} perturbation vs CPU", kind.name()),
                        params: String::new(),
                        result: "no ray from 0 reaches a boundary with smooth escaping pixels at 1e5×".into(),
                        threshold: "a view to test at",
                        pass: false,
                    }),
                }
            }
            for (fractal, cx, cy, mag) in abs_cases {
                self.fractal = fractal;
                let mut vp = Viewport::new(N as f64, N as f64);
                vp.center_x = fractadyne_core::parse_bf(&cx).unwrap();
                vp.center_y = fractadyne_core::parse_bf(&cy).unwrap();
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mag));
                vp.precision = fractadyne_core::precision_for_magnification(mag);
                let mut pert = self.current_export_request_for(&vp, false);
                pert.width = N;
                pert.height = N;
                pert.ss = 1;
                let mut direct = pert.clone();
                direct.mode = 1; // force the trusted direct df32 path
                // A power family against the CPU's f64 orbit instead: the direct path's c is
                // f32-quantised here (the compiler folds df32; two ulps a pixel at |c| ≈ 1), and at
                // power 8 that quarter pixel moved the mean dwell past the bound (0.89 on a view
                // where the 1e6× twins check matched the CPU to a median 0.0002).
                let power_family = fractal.power_family().is_some();
                let truth = if power_family {
                    Some(cpu_family_iter(&pert, fractal.formula_id(), N))
                } else {
                    st_render_iter(device, queue, &direct)
                };
                let against = if power_family { "CPU" } else { "direct" };
                if let (Some(a), Some(b)) = (st_render_iter(device, queue, &pert), truth) {
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
                        name: format!("{} perturbation vs {against}", fractal.name()),
                        params: format!("{mag:.0e}× at {cx}, {cy}, mode {} vs {against}, n={n}", pert.mode),
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

        // ---- custom formulas: generated shader modules (design/custom-formulas.md phase 2) ----
        // (1) A built-in's step generated from the formula IR must render BIT FOR BIT as the
        //     built-in (direct mode, smooth-iteration channel) — the generated code is the built-in's
        //     own helper calls, so any difference is a splice or codegen bug.
        // (2) Formulas with no built-in: the GPU against the IR's f64 interpreter (which the core
        //     tests hold bit-identical to every built-in), pixel by pixel at the shader's own pixel
        //     centres.
        if want("custom-formula") {
            use fractadyne_core::ir;
            self.julia_mode = false;
            self.coloring.color_method = crate::ColorMethod::Smooth;
            self.coloring.use_custom_palette = false;
            self.render_cfg.auto_iter = false;
            self.render_cfg.max_iter = 1000;
            let nn = N as usize;
            let view = |cx: f64, cy: f64, span: f64| {
                let mut vp = Viewport::new(N as f64, N as f64);
                vp.center_x = fractadyne_core::BigFloat::from_f64(cx, 64);
                vp.center_y = fractadyne_core::BigFloat::from_f64(cy, 64);
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(span / N as f64);
                vp.precision = 64;
                vp
            };
            // The power families each over the whole set (design/power-families.md B1): the
            // built-in's chain is the generated module's, helper for helper.
            let twins: Vec<(FractalKind, f64, f64, f64)> = [
                (FractalKind::Mandelbrot, -0.745, 0.112, 0.02),
                (FractalKind::Multibrot3, 0.0, 0.0, 3.0),
                (FractalKind::Tricorn, -0.2, 0.0, 3.5),
                (FractalKind::BurningShip, -1.7548, -0.0312, 0.05),
            ]
            .into_iter()
            .chain(FractalKind::ALL.into_iter().filter(|k| k.power_family().is_some()).map(|k| (k, 0.0, 0.0, 3.0)))
            .collect();
            for (fractal, cx, cy, span) in twins {
                self.fractal = fractal;
                let mut base = self.current_export_request_for(&view(cx, cy, span), false);
                base.width = N;
                base.height = N;
                base.ss = 1;
                base.mode = 1;
                let step = ir::builtin_step(fractal.formula_id()).expect("every built-in has a step");
                let built = fractadyne_gpu::custom::build(&ir::Formula::single(step), &[]);
                let mut gen = base.clone();
                gen.formula = fractadyne_core::formula::CUSTOM;
                gen.custom = built.as_ref().ok().map(|s| std::sync::Arc::new(s.clone()));
                let name = format!("generated {} = built-in", fractal.name());
                if let Err(e) = &built {
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Custom formula (GPU)",
                        name,
                        params: format!("direct, span {span}"),
                        result: format!("build failed: {e}"),
                        threshold: "builds",
                        pass: false,
                    });
                    continue;
                }
                let d = fractadyne_core::formula::power(fractal.formula_id());
                if fractal.power_family().is_some() && d >= 7 {
                    // From power 7 the two part by POLICY, not by step: a generated module tames an
                    // escaping value past 1e15 (`custom_tame`), which z⁷ from |z| = 256 reaches, and
                    // Multibrot 8 escapes at 128 (`bail2_of`) where the module escapes at 256. Leaving
                    // those pixels out by the CPU orbit failed on chaotic ones (the GPU's orbit is not
                    // the CPU's there), so the built-in is judged against the CPU interpreter with ITS
                    // policy instead, as the iterated formulas below are: under 1% disagree (|Δ| > 0.01
                    // or status) and none of the escapes within 20 steps. Powers to 6 stay bit for bit.
                    let Some(a) = st_render_iter(device, queue, &base) else { continue };
                    let bail2 = if d == 8 { 128.0 * 128.0 } else { 256.0 * 256.0 };
                    let centre = (base.center[0] as f64 + base.center[2] as f64, base.center[1] as f64 + base.center[3] as f64);
                    let scale = 2f64.powi(base.delta_exp);
                    let (sx, sy) = (base.span_mantissa.x / N as f64, base.span_mantissa.y / N as f64);
                    // The CPU's smooth value at c (−∞ = interior) and its step count.
                    let cpu_at = |c: (f64, f64)| {
                        let pts = fractadyne_core::orbit_points((0.0, 0.0), c, fractal.formula_id(), base.max_iter as usize, bail2);
                        let (x, y) = *pts.last().unwrap();
                        let mag2 = x * x + y * y;
                        let steps = pts.len() - 1;
                        let v = if mag2 > bail2 {
                            steps as f64 + 1.0 - (mag2.ln() * 0.5 / 2f64.ln()).ln() / f64::from(d).ln()
                        } else {
                            f64::NEG_INFINITY
                        };
                        (v, steps)
                    };
                    let agree = |g: f64, cpu: f64| if cpu.is_finite() { g >= 0.0 && (g - cpu).abs() < 0.01 } else { g < 0.0 };
                    const PROBE: f64 = 1.0e-5;
                    let (mut escaped, mut disagree, mut early, mut early_bad) = (0u64, 0u64, 0u64, 0u64);
                    for k in 0..nn * nn {
                        let (i, j) = ((k % nn) as f64, (k / nn) as f64);
                        let c = (
                            centre.0 + sx * ((i + 0.5) - N as f64 * 0.5) * scale,
                            centre.1 + sy * (N as f64 * 0.5 - (j + 0.5)) * scale,
                        );
                        let (cpu, steps) = cpu_at(c);
                        let g = a[k * 4] as f64;
                        let same = agree(g, cpu);
                        escaped += cpu.is_finite() as u64;
                        disagree += (!same) as u64;
                        // An early escape must agree — unless the CPU's own value does not survive
                        // c ± PROBE: at power 8 an f32 error grows ~1,000× a step, and an orbit landing
                        // by the bailout radius escapes a step apart on the two (an escape-step tie,
                        // as the Chaotic kind below judges).
                        if cpu.is_finite()
                            && steps <= 20
                            && [(PROBE, 0.0), (-PROBE, 0.0), (0.0, PROBE), (0.0, -PROBE)]
                                .iter()
                                .all(|p| (cpu_at((c.0 + p.0, c.1 + p.1)).0 - cpu).abs() < 0.01)
                        {
                            early += 1;
                            early_bad += (!same) as u64;
                        }
                    }
                    let px = (nn * nn) as u64;
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Custom formula (GPU)",
                        name: format!("{}: direct built-in = CPU interpreter", fractal.name()),
                        params: format!("direct, span {span}, {escaped} escaped px, {early} stable within 20 it"),
                        result: format!(
                            "{disagree} px disagree ({:.3}%), {early_bad} of the stable early escapes",
                            disagree as f64 * 100.0 / px as f64
                        ),
                        threshold: "<1% disagree, 0 stable early, >10% escaped",
                        pass: disagree * 100 < px && early_bad == 0 && early > 0 && escaped * 10 > px,
                    });
                    continue;
                }
                if let (Some(a), Some(b)) = (st_render_iter(device, queue, &base), st_render_iter(device, queue, &gen)) {
                    let (mut escaped, mut differ) = (0u64, 0u64);
                    for k in 0..nn * nn {
                        escaped += (a[k * 4] >= 0.0) as u64;
                        differ += (a[k * 4].to_bits() != b[k * 4].to_bits()) as u64;
                    }
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Custom formula (GPU)",
                        name,
                        params: format!("direct, span {span}, {escaped} escaped px"),
                        result: format!("{differ} px differ"),
                        threshold: "0 differ, >10% escaped",
                        pass: differ == 0 && escaped * 10 > (nn * nn) as u64,
                    });
                }
            }

            // (2) GPU vs the CPU interpreter at the shader's own pixel centres. A pixel DISAGREES when
            // its escape status differs or its smooth values are more than `tol` apart. Three kinds:
            //  - Iterated df32 formulas: only the boundary's chaotic pixels may part company (<1%), and
            //    none that escape within EARLY iterations — too few steps for df32 rounding to grow.
            //  - Each f32-tier function EVALUATED ONCE per pixel: `z' = z + 8·(f(c) + 1 + 2i)` grows
            //    linearly, escapes after a few steps, and its smooth value encodes |f(c) + 1 + 2i|
            //    (the offset makes a sign error in either component change it). Nothing is amplified,
            //    so only f32-vs-f64 rounding separates the two. (A first cut, `z' = 100·f(c)`, escaped
            //    at n = 1, where the smooth value is negative: 0 of 484,000 pixels escaped on either
            //    side, and the check could not have seen a wrong function.)
            //  - `sin z + c` iterated, a stress test of the overflow guard. The family expands by
            //    |cos z| ≈ cosh(Im z) per step, so f32 rounding reaches O(1) within ~10 iterations
            //    (measured: a pixel's GPU and CPU orbits escaped at |z| ≈ 2.7e3 and 6e23) and only
            //    loose agreement is possible. Its gate is that NO smooth value is non-finite: before
            //    `custom_tame`, 3,044 pixels escaped as −∞, which the colour pass paints as interior.
            //    Agreement is judged only where the CPU's own value survives c ± 1e-5 (an error per
            //    step about f32 range reduction's at |z| ≈ 256); elsewhere the orbit is chaos on any
            //    GPU. A bound on ALL pixels (<5%) was one GPU's calibration: the RTX 3080 disagreed on
            //    3.0% and the RX 6800 XT on 7.1%, while the 3080 disagreed on 0 of 43,692 stable pixels
            //    (90%). A sin off by 1e-4 (1e-5) relative fails it: 5.9% (0.17%) of stable pixels.
            //  - Formulas whose own dynamics amplify rounding even in early escapes, judged like the
            //    stress test on the pixels stable under c ± PROBE: Magnet I lingers by the repelling
            //    point |z| ≈ 4 of its far map z²/4 (×2 per step: GPU and CPU orbits escaping at
            //    steps 14–20 measured 0.01–0.6 apart; 0 of 44,160 stable pixels disagree), and
            //    Barnsley M1 branches on the sign of Re z. Its stable pixels still part company at
            //    the rim of its oval (RTX 3080: 32 of 47,156, every one CPU-interior and GPU-escaped
            //    after 110–297 steps, mirror-symmetric in the four quadrants): there each step
            //    multiplies by |c| > 1, and an orbit kept bounded in exact arithmetic is kicked off
            //    by rounding injected EVERY step — which a one-time move of c does not model.
            enum Kind {
                Iterated,
                Once,
                Stress,
                Chaotic,
            }
            let hybrid = ir::Formula::new(vec![
                ir::builtin_step(fractadyne_core::formula::MANDELBROT).unwrap(),
                ir::builtin_step(fractadyne_core::formula::BURNING_SHIP).unwrap(),
            ])
            .unwrap();
            let quad_param = {
                let mut b = ir::Builder::new();
                let z = b.push(ir::Op::Z);
                let s = b.push(ir::Op::Sqr(z));
                let p = b.push(ir::Op::Param(0));
                let pz = b.push(ir::Op::Mul(p, z));
                let t = b.push(ir::Op::Add(s, pz));
                let c = b.push(ir::Op::C);
                let out = b.push(ir::Op::Add(t, c));
                ir::Formula::single(b.finish(out).unwrap())
            };
            let sine = {
                let mut b = ir::Builder::new();
                let z = b.push(ir::Op::Z);
                let s = b.push(ir::Op::Func(ir::Func::Sin, z));
                let c = b.push(ir::Op::C);
                let out = b.push(ir::Op::Add(s, c));
                ir::Formula::single(b.finish(out).unwrap())
            };
            // Each f32-tier function once: z' = z + 8·(f(c) + 1 + 2i) (a complex power takes the
            // exponent 1.5+0.5i).
            let once = |f: Option<ir::Func>| {
                let mut b = ir::Builder::new();
                let z = b.push(ir::Op::Z);
                let c = b.push(ir::Op::C);
                let v = match f {
                    Some(f) => b.push(ir::Op::Func(f, c)),
                    None => {
                        let w = b.push(ir::Op::Const(1.5, 0.5));
                        b.push(ir::Op::Pow(c, w))
                    }
                };
                let k = b.push(ir::Op::Const(1.0, 2.0));
                let s = b.push(ir::Op::Add(v, k));
                let t = b.push(ir::Op::Scale(s, 8.0));
                let out = b.push(ir::Op::Add(z, t));
                ir::Formula::single(b.finish(out).unwrap())
            };
            use ir::Func as F;
            let mut cpu_cases: Vec<(String, ir::Formula, Vec<(f64, f64)>, (f64, f64, f64), Kind)> = vec![
                ("hybrid Mandelbrot/Burning Ship".into(), hybrid, vec![], (-0.5, 0.0, 3.5), Kind::Iterated),
                ("z² + p·z + c (parameter)".into(), quad_param, vec![(0.25, -0.1)], (-0.3, 0.0, 3.5), Kind::Iterated),
                ("sin z + c, iterated (f32 tier)".into(), sine, vec![], (0.0, 0.0, 6.0), Kind::Stress),
            ];
            for f in [F::Exp, F::Log, F::Sqrt, F::Sin, F::Cos, F::Tan, F::Sinh, F::Cosh, F::Tanh] {
                cpu_cases.push((format!("{f:?}"), once(Some(f)), vec![], (0.0, 0.0, 12.0), Kind::Once));
            }
            cpu_cases.push(("c^(1.5+0.5i)".into(), once(None), vec![], (0.0, 0.0, 12.0), Kind::Once));
            // Fractint's sections (phase 3), on the shader's init and bailout slots: an init section
            // and a variable kept from step to step (Manowar), an if block (Barnsley M1), the
            // formula's own test ending an orbit at escape or at a fixed point (Magnet I), and
            // `maxit` (the uniform: read as 0, nothing would escape) with the four roundings.
            for (label, src, view_at, kind) in [
                ("Manowar (init, variables, bailout)", "z = c, z1 = c:\nt = z\nz = z*z + z1 + c\nz1 = t\n|z| <= 4", (-0.15, 0.0, 0.8), Kind::Iterated),
                (
                    "Barnsley M1 (if block)",
                    "z = c:\nif (real(z) >= 0)\n z = (z - 1)*c\nelse\n z = (z + 1)*c\nendif\n|z| <= 4",
                    (0.0, 0.0, 4.0),
                    Kind::Chaotic,
                ),
                (
                    "Magnet I (escape or converge)",
                    "z = sqr((z^2 + c - 1)/(2*z + c - 2))\n|z| <= 100 && |z - 1| > 0.000001",
                    (1.3, 0.0, 4.4),
                    Kind::Chaotic,
                ),
                (
                    "maxit and rounding",
                    "z = z^2 + c*(maxit/1000) + floor(c*3)/16 + ceil(c*2)/32 + trunc(c*5)/64 + round(c*7)/128",
                    (-0.5, 0.0, 3.5),
                    Kind::Iterated,
                ),
            ] {
                match ir::parse::parse(src) {
                    Ok(f) => cpu_cases.push((label.into(), f, vec![], view_at, kind)),
                    Err(e) => push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Custom formula (GPU)",
                        name: format!("{label}: GPU = CPU interpreter"),
                        params: String::new(),
                        result: format!("does not read: {e}"),
                        threshold: "reads",
                        pass: false,
                    }),
                }
            }
            // The functions-once cases report as ONE check (their total, and the worst function).
            let (mut once_n, mut once_bad, mut once_escaped, mut once_worst) = (0u64, 0u64, 0u64, (0u64, String::new()));
            for (label, formula, params, (cx, cy, span), kind) in cpu_cases {
                self.fractal = FractalKind::Mandelbrot;
                let mut req = self.current_export_request_for(&view(cx, cy, span), false);
                req.width = N;
                req.height = N;
                req.ss = 1;
                req.mode = 1;
                req.formula = fractadyne_core::formula::CUSTOM;
                if matches!(kind, Kind::Once) {
                    req.max_iter = 64; // linear growth: n ≈ 32/|f(c) + 1 + 2i|
                }
                if matches!(kind, Kind::Chaotic) {
                    req.max_iter = 300; // the probe runs five orbits a pixel; the interior runs to the cap
                }
                let shader = match fractadyne_gpu::custom::build(&formula, &params) {
                    Ok(s) => s,
                    Err(e) => {
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "Custom formula (GPU)",
                            name: format!("{label}: GPU = CPU interpreter"),
                            params: String::new(),
                            result: format!("build failed: {e}"),
                            threshold: "builds",
                            pass: false,
                        });
                        continue;
                    }
                };
                let power = shader.power as f64;
                let precision = shader.precision;
                req.custom = Some(std::sync::Arc::new(shader));
                let Some(gpu) = st_render_iter(device, queue, &req) else { continue };
                // The shader's pixel centre: centre + step·((i + ½) − N/2)·2^delta_exp (y flipped).
                let centre = (req.center[0] as f64 + req.center[2] as f64, req.center[1] as f64 + req.center[3] as f64);
                let scale = 2f64.powi(req.delta_exp);
                let (sx, sy) = (req.span_mantissa.x / N as f64, req.span_mantissa.y / N as f64);
                let bail2 = 256.0 * 256.0;
                let tol = if matches!(kind, Kind::Once) { 1.0e-3 } else { 0.01 };
                const EARLY: usize = 20;
                let (mut escaped, mut disagree, mut early, mut early_bad, mut nonfinite) = (0u64, 0u64, 0u64, 0u64, 0u64);
                // Escaped with a smooth value the clamp did not flatten: the pixels that can show a
                // wrong value at all.
                let mut informative = 0u64;
                // The disagreements by kind: escape status, the escape step (|Δ| ≥ ½), or the value
                // at the same step (a last point 1% apart in log|z|: an orbit that parted earlier).
                let (mut bad_status, mut bad_step, mut bad_value) = (0u64, 0u64, 0u64);
                // Stress only: the pixels whose CPU value survives c ± PROBE and c ± PROBE·i — an
                // error per step about the size of f32 range reduction at |z| ≈ 256 — and how many
                // of those the GPU gets wrong. A pixel that fails the probe is chaos on any GPU.
                const PROBE: f64 = 1.0e-5;
                let (mut stable, mut stable_bad) = (0u64, 0u64);
                // The CPU's smooth value at c (−1 = interior) and its step count.
                let own_test = formula.has_bailout();
                let cpu_at = |c: (f64, f64)| {
                    let pts = ir::orbit_points(&formula, (0.0, 0.0), c, &params, req.max_iter as usize, bail2)
                        .expect("parameters supplied");
                    // The generated step's overflow guard, mirrored (it can only touch the last point).
                    let (x, y) = fractadyne_gpu::custom::tame_f64(*pts.last().unwrap());
                    let mag2 = x * x + y * y;
                    let steps = pts.len() - 1;
                    let nu = |mag2: f64| (mag2.ln() * 0.5 / 2f64.ln()).ln() / power.ln();
                    // …and its smooth value, clamped at 0 as the generated module clamps it. A formula's
                    // own test ends the orbit where it says (escaped, unless it ran to the cap); the
                    // module's log-log term applies only past |z| = 2, the step count below.
                    let v = if own_test {
                        match (steps < req.max_iter as usize, mag2 > 4.0) {
                            (false, _) => -1.0,
                            (true, true) => (steps as f64 + 1.0 - nu(mag2)).max(0.0),
                            (true, false) => steps as f64,
                        }
                    } else if mag2 > bail2 {
                        (steps as f64 + 1.0 - nu(mag2)).max(0.0)
                    } else {
                        -1.0
                    };
                    (v, steps)
                };
                let agree = |a: f64, b: f64| if a < 0.0 || b < 0.0 { (a < 0.0) == (b < 0.0) } else { (a - b).abs() < tol };
                for j in 0..nn {
                    // The interpreter walks every pixel to the cap; Manowar's took 9.3–10.4 s with
                    // no breadcrumb, at the watchdog's 10 s window, and a busier run tripped it.
                    if j > 0 && j % 64 == 0 {
                        crate::diag::breadcrumb(format!("selftest: {label}: CPU interpreter, row {j} of {nn}"));
                    }
                    for i in 0..nn {
                        let c = (
                            centre.0 + sx * ((i as f64 + 0.5) - N as f64 * 0.5) * scale,
                            centre.1 + sy * (N as f64 * 0.5 - (j as f64 + 0.5)) * scale,
                        );
                        let (cpu, steps) = cpu_at(c);
                        let g = gpu[(j * nn + i) * 4] as f64;
                        nonfinite += (!g.is_finite()) as u64;
                        escaped += (cpu >= 0.0) as u64;
                        informative += (cpu > 0.5) as u64;
                        let same = agree(cpu, g);
                        disagree += (!same) as u64;
                        if matches!(kind, Kind::Stress | Kind::Chaotic)
                            && [(PROBE, 0.0), (-PROBE, 0.0), (0.0, PROBE), (0.0, -PROBE)]
                                .iter()
                                .all(|d| agree(cpu, cpu_at((c.0 + d.0, c.1 + d.1)).0))
                        {
                            stable += 1;
                            stable_bad += (!same) as u64;
                        }
                        if !same {
                            if cpu < 0.0 || g < 0.0 {
                                bad_status += 1;
                            } else if (cpu - g).abs() >= 0.5 {
                                bad_step += 1;
                            } else {
                                bad_value += 1;
                            }
                        }
                        if cpu >= 0.0 && steps <= EARLY {
                            early += 1;
                            early_bad += (!same) as u64;
                        }
                    }
                }
                let px = (nn * nn) as u64;
                let frac = disagree as f64 / px as f64;
                match kind {
                    Kind::Once => {
                        once_n += px;
                        once_bad += disagree;
                        once_escaped += informative;
                        if disagree >= once_worst.0 {
                            once_worst = (disagree, label);
                        }
                    }
                    Kind::Iterated => push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Custom formula (GPU)",
                        name: format!("{label}: GPU = CPU interpreter"),
                        params: format!("direct, span {span}, {precision:?}, {escaped} escaped px, {early} within {EARLY} it"),
                        result: format!(
                            "{disagree} px disagree ({:.3}%): status {bad_status}, step {bad_step}, value {bad_value}; \
                             {early_bad} of the early escapes",
                            frac * 100.0
                        ),
                        threshold: "<1% disagree, 0 early, >10% escaped",
                        pass: frac < 0.01 && early_bad == 0 && early > 0 && escaped * 10 > px,
                    }),
                    Kind::Stress => push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Custom formula (GPU)",
                        name: format!("{label}: overflow guard"),
                        params: format!("direct, span {span}, {precision:?}, {escaped} escaped px"),
                        result: format!(
                            "{nonfinite} non-finite, {disagree} px disagree ({:.3}%): status {bad_status}, step {bad_step}, \
                             value {bad_value}; {early_bad} of {early} escaping within {EARLY} it; \
                             {stable_bad} of {stable} stable under c ± {PROBE:e}",
                            frac * 100.0
                        ),
                        threshold: "0 non-finite, <0.1% of stable px disagree, ≥75% stable, >10% escaped",
                        pass: nonfinite == 0 && stable_bad * 1000 < stable && stable * 4 >= px * 3 && escaped * 10 > px,
                    }),
                    Kind::Chaotic => push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Custom formula (GPU)",
                        name: format!("{label}: GPU = CPU interpreter"),
                        params: format!("direct, span {span}, {precision:?}, {escaped} escaped px"),
                        result: format!(
                            "{stable_bad} of {stable} px stable under c ± {PROBE:e} disagree; all px: {disagree} ({:.3}%): \
                             status {bad_status}, step {bad_step}, value {bad_value}; {nonfinite} non-finite",
                            frac * 100.0
                        ),
                        threshold: "<0.1% of stable px disagree, ≥75% stable, >10% escaped, 0 non-finite",
                        pass: nonfinite == 0 && stable_bad * 1000 < stable && stable * 4 >= px * 3 && escaped * 10 > px,
                    }),
                }
            }
            let once_frac = once_bad as f64 / once_n.max(1) as f64;
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Custom formula (GPU)",
                name: "f32-tier functions, each once: GPU = CPU interpreter".into(),
                params: format!("10 functions, span 12, {once_escaped} of {once_n} px escaped with smooth > 0.5"),
                result: format!(
                    "{once_bad} px disagree ({:.4}%), worst {} ({})",
                    once_frac * 100.0,
                    once_worst.1,
                    once_worst.0
                ),
                threshold: "<0.1% disagree (|Δ| > 1e-3), >25% informative",
                pass: once_frac < 0.001 && once_escaped * 4 > once_n,
            });

            // A formula with sections is not resumable: the chunk pass neither runs its init section
            // nor carries its variables, so a chunked path must fall back to the single pass for it.
            // The export path once did not (`--render` drew the Lambda parameter plane black and
            // Spider blank), and the checks above, single passes, could not see it. The control, a
            // formula without sections, must chunk — or "did not chunk" would prove nothing.
            for (label, src, (cx, cy, span), want_chunked) in [
                ("z² + c (control)", "z = z^2 + c", (-0.5, 0.0, 3.0), true),
                ("Manowar (sections)", "z = c, z1 = c:\nt = z\nz = z*z + z1 + c\nz1 = t\n|z| <= 4", (-0.15, 0.0, 0.8), false),
            ] {
                self.fractal = FractalKind::Mandelbrot;
                let mut req = self.current_export_request_for(&view(cx, cy, span), false);
                req.width = N;
                req.height = N;
                req.ss = 1;
                req.mode = 1;
                req.max_iter = 400;
                req.formula = fractadyne_core::formula::CUSTOM;
                let built = ir::parse::parse(src).map_err(|e| e.to_string()).and_then(|f| {
                    fractadyne_gpu::custom::build(&f, &[]).map_err(|e| e.to_string())
                });
                let name = format!("{label}: chunked render = single pass");
                let shader = match built {
                    Ok(s) => s,
                    Err(e) => {
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "Custom formula (GPU)",
                            name,
                            params: String::new(),
                            result: format!("build failed: {e}"),
                            threshold: "builds",
                            pass: false,
                        });
                        continue;
                    }
                };
                req.custom = Some(std::sync::Arc::new(shader));
                let single = st_render_iter(device, queue, &req);
                let mut passes = Vec::new();
                let chunked = fractadyne_gpu::render_iter_chunked_timed(device, queue, &req, 64, &mut passes)
                    .map_err(|e| eprintln!("[selftest] GPU ERROR (render_iter_chunked_timed): {e}"))
                    .ok();
                let (pass, result) = match (&single, &chunked) {
                    (Some(a), Some(r)) if a.len() == r.pixels.len() => {
                        let differ = a.iter().zip(&r.pixels).filter(|(x, y)| x.to_bits() != y.to_bits()).count();
                        let escaped = a.chunks(4).filter(|p| p[0] >= 0.0).count();
                        (
                            differ == 0 && passes.is_empty() != want_chunked && escaped * 10 > a.len() / 4,
                            format!("{} chunk passes, {differ} texels differ, {escaped} escaped px", passes.len()),
                        )
                    }
                    _ => (false, "render failed".into()),
                };
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Custom formula (GPU)",
                    name,
                    params: format!("direct, span {span}, 400 iterations, 64 a pass"),
                    result,
                    threshold: if want_chunked { "chunked, 0 differ, >10% escaped" } else { "not chunked, 0 differ, >10% escaped" },
                    pass,
                });
            }

            // (3) PERTURBATION (mode 0), which takes a custom formula past the f32 wall.
            // (a) The generated perturbed step (`ir::perturb`) against each built-in's hand-written
            //     one, on the SAME reference orbit — the built-in's own request with only the formula
            //     and module swapped, SA and BLA off in both. The rule table orders the arithmetic
            //     differently ((2Z + δ)·δ against 2Z·δ + δ²), so the renders need not be identical:
            //     on a main component's boundary (where bisection lands) escape by `max_iter` is
            //     chaotic, and 2–14% of pixels flip between the two — evenly, the CPU interpreter
            //     siding with each about half the time (measured). So the CPU interpreter in f64
            //     settles every pixel, and the generated step must be right no less often than the
            //     hand-written one (beyond the ±3σ of a fair coin), and its smooth value no further
            //     off.
            self.render_cfg.auto_iter = false;
            self.render_cfg.max_iter = 2000;
            let max_iter = 2000u32;
            let bail2 = 256.0 * 256.0;
            // Every helper below takes the iteration BUDGET: the ring formulas run 2,000, while the
            // functions and division run 60 (see (b)), where single precision can follow them.
            let cpu_smooth = |formula: &ir::Formula, params: &[(f64, f64)], c: (f64, f64), power: f64, budget: u32| -> f64 {
                let pts = ir::orbit_points(formula, (0.0, 0.0), c, params, budget as usize, bail2).unwrap();
                let (x, y) = fractadyne_gpu::custom::tame_f64(*pts.last().unwrap());
                let mag2 = x * x + y * y;
                if mag2 > bail2 {
                    ((pts.len() - 1) as f64 + 1.0 - (mag2.ln() * 0.5 / 2f64.ln()).ln() / power.ln()).max(0.0)
                } else {
                    -1.0
                }
            };
            // Over a set of pixels, split across threads: a 2000-iteration frame on one core takes
            // long enough (10 s) to trip the hang watchdog.
            let cpu_pixels = |formula: &ir::Formula, params: &[(f64, f64)], power: f64, cs: &[(f64, f64)], budget: u32| -> Vec<f64> {
                let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
                std::thread::scope(|s| {
                    let parts: Vec<_> = cs
                        .chunks(cs.len().div_ceil(threads).max(1))
                        .map(|part| s.spawn(move || part.iter().map(|&c| cpu_smooth(formula, params, c, power, budget)).collect::<Vec<_>>()))
                        .collect();
                    parts.into_iter().flat_map(|h| h.join().expect("a CPU interpreter thread panicked")).collect()
                })
            };
            // Every view sits ON its formula's boundary, found by bisecting between an interior
            // point (c = 0) and an escaping one. Hand-picking a point for a formula nobody has
            // explored is how a check ends up comparing a flat frame — hand-picked views here were
            // all interior (Tricorn) or had a reference escaping in under 20 steps (three others).
            let boundary = |formula: &ir::Formula, params: &[(f64, f64)], outside: (f64, f64), budget: u32| -> Option<(f64, f64)> {
                // Escaped = the last point is past the bailout, an escape ON the last step included,
                // as the GPU, the CPU interpreter and the bignum oracle all count it. (`pts.len() <=
                // budget` missed the last step: for `√(z⁴ + c)` the bisection then settled on the
                // curve between escape at step 59 and at 60, whose "inside" end the bignum oracle
                // calls escaped — no deep bracket along any ray.)
                let escapes = |c: (f64, f64)| {
                    let pts = ir::orbit_points(formula, (0.0, 0.0), c, params, budget as usize, bail2).unwrap();
                    pts.last().is_some_and(|z| z.0 * z.0 + z.1 * z.1 > bail2)
                };
                // The escaping end: the direction's first multiple that escapes (the Buffalo set
                // still holds −0.9−0.7i).
                let mut inside = (0.0, 0.0);
                let mut outside = [1.0, 2.0, 4.0].iter().map(|k| (outside.0 * k, outside.1 * k)).find(|&o| escapes(o))?;
                if escapes(inside) {
                    return None;
                }
                for _ in 0..48 {
                    let mid = ((inside.0 + outside.0) * 0.5, (inside.1 + outside.1) * 0.5);
                    if escapes(mid) { outside = mid } else { inside = mid }
                }
                Some(inside)
            };
            let no_boundary = |name: String, outside: (f64, f64)| SelfCheck {
                category: "Custom formula (GPU)",
                name,
                params: String::new(),
                result: format!("no boundary between 0 and 4×({}{:+}i)", outside.0, outside.1),
                threshold: "a boundary to test at",
                pass: false,
            };
            let bf = |v: f64| fractadyne_core::BigFloat::from_f64(v, 64);
            // Each family at 1e6× on its own boundary, and Mandelbrot's seahorse valley at 1e8×.
            // The built-ins bisect into the THIRD quadrant: with Im c > 0 the Burning Ship's |Im|
            // fold never engages (Im z' = 2|x||y| + Im c stays positive), and a mutant dropping
            // that fold from the generated step passed every first-quadrant view (measured).
            let third = (-0.9, -0.7);
            let mut pert_twins = vec![(
                FractalKind::Mandelbrot,
                fractadyne_core::parse_bf("-0.743643887037158704752191506114774").unwrap(),
                fractadyne_core::parse_bf("0.131825904205311970493132056385139").unwrap(),
                1.0e8,
            )];
            // The power families too (design/power-families.md B2, B3): the built-in's generic
            // binomial perturbation and its folds against the generated module and the CPU.
            let power_families = FractalKind::ALL.into_iter().filter(|k| k.power_family().is_some());
            for fractal in [
                FractalKind::Mandelbrot,
                FractalKind::Multibrot3,
                FractalKind::Multibrot4,
                FractalKind::Multibrot5,
                FractalKind::Tricorn,
                FractalKind::BurningShip,
                FractalKind::Celtic,
                FractalKind::Buffalo,
            ]
            .into_iter()
            .chain(power_families)
            {
                let step = ir::builtin_step(fractal.formula_id()).expect("every built-in has a step");
                match boundary(&ir::Formula::single(step), &[], third, max_iter) {
                    Some(at) => pert_twins.push((fractal, bf(at.0), bf(at.1), 1.0e6)),
                    None => push_check(
                        &mut checks,
                        &mut last_check_t,
                        no_boundary(format!("generated {} perturbation = built-in", fractal.name()), third),
                    ),
                }
            }
            for (fractal, cx, cy, mag) in pert_twins {
                self.fractal = fractal;
                let at = (fractadyne_core::to_f64(&cx), fractadyne_core::to_f64(&cy));
                let mut vp = Viewport::new(N as f64, N as f64);
                vp.center_x = cx;
                vp.center_y = cy;
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mag));
                vp.precision = fractadyne_core::precision_for_magnification(mag).max(64);
                let mut base = self.current_export_request_for(&vp, false);
                base.width = N;
                base.height = N;
                base.ss = 1;
                base.sa_skip = 0;
                base.bla_on = 0;
                let formula = ir::Formula::single(ir::builtin_step(fractal.formula_id()).expect("every built-in has a step"));
                let Ok(shader) = fractadyne_gpu::custom::build(&formula, &[]) else { continue };
                let power = shader.power as f64;
                let mut gen = base.clone();
                gen.formula = fractadyne_core::formula::CUSTOM;
                gen.custom = Some(std::sync::Arc::new(shader));
                let (Some(a), Some(b)) = (st_render_iter(device, queue, &base), st_render_iter(device, queue, &gen)) else {
                    continue;
                };
                let scale = 2f64.powi(base.delta_exp);
                let (sx, sy) = (base.span_mantissa.x / N as f64, base.span_mantissa.y / N as f64);
                // Both renders against the CPU, pixel by pixel: right = the same status and within 2
                // iterations; and among pixels both get right, how far each smooth value is off (an
                // error in the smooth value moves every pixel, so its median shows it; chaos moves
                // both renders alike).
                let cs: Vec<(f64, f64)> = (0..nn * nn)
                    .map(|k| {
                        let (i, j) = (k % nn, k / nn);
                        (
                            at.0 + sx * ((i as f64 + 0.5) - N as f64 * 0.5) * scale,
                            at.1 + sy * (N as f64 * 0.5 - (j as f64 + 0.5)) * scale,
                        )
                    })
                    .collect();
                let close = |v: f32, cpu: f64| if v < 0.0 || cpu < 0.0 { (v < 0.0) == (cpu < 0.0) } else { (v as f64 - cpu).abs() <= 2.0 };
                let (mut escaped, mut base_only, mut gen_only) = (0u64, 0u64, 0u64);
                let (mut err_base, mut err_gen) = (Vec::new(), Vec::new());
                let cpu_gen = cpu_pixels(&formula, &[], power, &cs, max_iter);
                // Each side against the CPU under ITS OWN policy. A generated module tames an escaping
                // value past 1e15 and escapes at 256; a power family's built-in does neither from power
                // 7 (z⁷ needs no tame; Multibrot 8 escapes at 128): judged by the module's policy, the
                // built-in looked 1.6–3.6× worse in the smooth value where it was the more exact one.
                let cpu_base = if fractal.power_family().is_some() {
                    let id = fractal.formula_id();
                    let b2 = if id == fractadyne_core::formula::MULTIBROT8 { 128.0 * 128.0 } else { bail2 };
                    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
                    std::thread::scope(|s| {
                        let parts: Vec<_> = cs
                            .chunks(cs.len().div_ceil(threads).max(1))
                            .map(|part| {
                                s.spawn(move || {
                                    part.iter()
                                        .map(|&c| {
                                            let pts = fractadyne_core::orbit_points((0.0, 0.0), c, id, max_iter as usize, b2);
                                            let (x, y) = *pts.last().unwrap();
                                            let m2 = x * x + y * y;
                                            if m2 > b2 {
                                                (pts.len() - 1) as f64 + 1.0 - (m2.ln() * 0.5 / 2f64.ln()).ln() / power.ln()
                                            } else {
                                                -1.0
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                })
                            })
                            .collect();
                        parts.into_iter().flat_map(|h| h.join().expect("a CPU thread panicked")).collect::<Vec<f64>>()
                    })
                } else {
                    cpu_gen.clone()
                };
                for (k, (cb, cg)) in cpu_base.into_iter().zip(cpu_gen).enumerate() {
                    let (x, y) = (a[k * 4], b[k * 4]);
                    escaped += (x >= 0.0) as u64;
                    let (bx, gy) = (close(x, cb), close(y, cg));
                    base_only += (bx && !gy) as u64;
                    gen_only += (gy && !bx) as u64;
                    if bx && gy && cb >= 0.0 && cg >= 0.0 {
                        err_base.push((x as f64 - cb).abs());
                        err_gen.push((y as f64 - cg).abs());
                    }
                }
                let median = |v: &mut Vec<f64>| {
                    if v.is_empty() {
                        return f64::INFINITY;
                    }
                    let mid = v.len() / 2;
                    *v.select_nth_unstable_by(mid, f64::total_cmp).1
                };
                let (med_base, med_gen) = (median(&mut err_base), median(&mut err_gen));
                let px = (nn * nn) as u64;
                let allowance = 3.0 * ((base_only + gen_only) as f64).sqrt() + 3.0;
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Custom formula (GPU)",
                    name: format!("generated {} perturbation = built-in", fractal.name()),
                    params: format!(
                        "{mag:.0e}× at {:.9}{:+.9}i, mode {}, ref {}, {:.1}% escaped",
                        at.0,
                        at.1,
                        base.mode,
                        base.orbit_len,
                        escaped as f64 * 100.0 / px as f64
                    ),
                    result: format!(
                        "right only in built-in {base_only} / only in generated {gen_only} px; median |Δ| to CPU {med_base:.5} / {med_gen:.5} over {} px",
                        err_base.len()
                    ),
                    // Which side is under test: the generated module, against a built-in of long
                    // standing — or, for a power family (design/power-families.md B2), the new
                    // built-in, against the generated module that passed its own gates.
                    threshold: if fractal.power_family().is_some() {
                        "mode 0, ref >100, 10–99% escaped; built-in no worse (3σ; median ×1.25 + 1e-4)"
                    } else {
                        "mode 0, ref >100, 10–99% escaped; generated no worse (3σ; median ×1.25 + 1e-4)"
                    },
                    pass: base.mode == 0
                        && base.orbit_len > 100
                        && escaped * 10 > px
                        && escaped * 100 < px * 99
                        && if fractal.power_family().is_some() {
                            (gen_only as f64) <= base_only as f64 + allowance && med_base <= med_gen * 1.25 + 1.0e-4
                        } else {
                            (base_only as f64) <= gen_only as f64 + allowance && med_gen <= med_base * 1.25 + 1.0e-4
                        },
                });
            }

            // (b) Formulas no built-in covers, at 1e6× — past the f32 wall, where the direct path is
            //     blocks — on the GPU's perturbation path against the CPU interpreter in f64 (which
            //     resolves this view's 1.4e-8 pixel easily). The reference orbit is the IR's own
            //     bignum one at the view centre, which sits on the formula's boundary as above.
            //     ⚠The BUDGET differs by class, and the tolerance with it. The ring formulas run
            //     2,000 iterations to 0.01 of the smooth value. Functions and division run 60, on
            //     status and within 2 iterations: on a boundary view their long orbits are chaotic
            //     transients that no single-precision path follows (measured at 2,000: direct f32
            //     iteration at 1e2× missed 50% of `sin z + c`'s smooth values, perturbation at 1e6×
            //     33%, the f64 perturbed step on the CPU none of the sampled ones), and an explosive
            //     escape (sin z jumping past f32's range) leaves a smooth value that moves ~0.1
            //     between f32 and f64. At 60, perturbation's status disagreed on 10 of 48,400.
            let mag = 1.0e6;
            let pert_cases: Vec<(&str, ir::Formula, Vec<(f64, f64)>, u32)> = vec![
                (
                    "hybrid Mandelbrot/Burning Ship",
                    ir::Formula::new(vec![
                        ir::builtin_step(fractadyne_core::formula::MANDELBROT).unwrap(),
                        ir::builtin_step(fractadyne_core::formula::BURNING_SHIP).unwrap(),
                    ])
                    .unwrap(),
                    vec![],
                    2000,
                ),
                ("z² + p·z + c", ir::parse::parse("z^2 + p1*z + c").unwrap(), vec![(0.25, -0.1)], 2000),
                ("|z|·z + conj(z)² + c", ir::parse::parse("|z|*z*0.3 + conj(z)^2 + c").unwrap(), vec![], 2000),
                // Functions and division: their perturbed forms run the small-argument helpers
                // and the δ-quotient on the GPU.
                ("sin z + c", ir::parse::parse("sin(z) + c").unwrap(), vec![], 60),
                ("sin z + cos z·cos z + c", ir::parse::parse("z = sin(z) + cos(z)*cos(z) + c").unwrap(), vec![], 60),
                // ½·exp z − ½ and ½·sinh z have attracting fixed points at c = 0 (plain exp z + c
                // escapes there, sinh z + c is parabolic) — each exercises a small-argument helper.
                ("½·exp z − ½ + c", ir::parse::parse("0.5*exp(z) - 0.5 + c").unwrap(), vec![], 60),
                ("½·sinh z + c", ir::parse::parse("0.5*sinh(z) + c").unwrap(), vec![], 60),
                ("z²·tanh z + c", ir::parse::parse("z*z*tanh(z) + c").unwrap(), vec![], 60),
                ("z² + c/(z + 2)", ir::parse::parse("z^2 + c/(z + 2)").unwrap(), vec![], 60),
                // The branch-cut functions and fixed-exponent powers (`DiffLog`, `DiffSqrt`,
                // `DiffPow`): principal values, which jump across the negative real axis. At the
                // functions' budget: their log, exp and pow run in f32 on the GPU, and at 2,000
                // iterations 8–18% of these pixels disagreed with f64 (250–400 early escapers
                // called interior) — the chaos of long orbits that single precision cannot follow,
                // not the rules: the same formulas' deep bignum checks showed 0 of 1,024.
                // `log(z + 0.5)`: 0 must be interior to bisect from (with `+ 2` the real orbit of
                // c = 0 climbs without a fixed point), and the argument must go NEGATIVE on the real
                // axis for the cut view below to cross log's own cut — with `log(z + 1)` it never
                // did there (a crossing test that ignored crossings passed it). `√(z⁴ + c)`, not
                // `z² + c·√(z + 1)`: that one's bisected point sat in an interior sliver thinner
                // than a 1e6× pixel (all 48,400 escaped, in f64 and on the GPU alike).
                ("z^2.5 + c", ir::parse::parse("z^2.5 + c").unwrap(), vec![], 60),
                ("z² + 0.1·log(z + ½) + c", ir::parse::parse("z^2 + 0.1*log(z + 0.5) + c").unwrap(), vec![], 60),
                ("√(z⁴ + c)", ir::parse::parse("sqrt(z^4 + c)").unwrap(), vec![], 60),
                ("z^p + c, p = 2.2 + 0.3i", ir::parse::parse("z^p1 + c").unwrap(), vec![(2.2, 0.3)], 60),
            ];
            let first = (0.9, 0.7);
            // The rays a boundary is looked for along, in order: exp-type sets escape to the right
            // and fold back elsewhere, so one ray does not serve every formula.
            let rays = [first, (-0.9, -0.7), (-0.9, 0.7), (0.9, -0.7), (1.2, 0.2)];
            for (label, formula, params, budget) in &pert_cases {
                let budget = *budget;
                // The ring formulas' tight tolerance; the others' status-and-2-iterations (above).
                let tol = if budget > 60 { 0.01 } else { 2.0 };
                let Some(at) = rays.iter().find_map(|&r| boundary(formula, params, r, budget)) else {
                    push_check(&mut checks, &mut last_check_t, no_boundary(format!("{label}: perturbed GPU = CPU interpreter"), first));
                    continue;
                };
                self.fractal = FractalKind::Mandelbrot;
                let mut vp = Viewport::new(N as f64, N as f64);
                vp.center_x = bf(at.0);
                vp.center_y = bf(at.1);
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mag));
                vp.precision = fractadyne_core::precision_for_magnification(mag).max(64);
                let mut req = self.current_export_request_for(&vp, false);
                let zero = fractadyne_core::BigFloat::from_f64(0.0, vp.precision);
                let Ok((mut orbit, _, _)) =
                    ir::reference_orbit(formula, &zero, &zero, &vp.center_x, &vp.center_y, params, budget, vp.precision)
                else {
                    continue;
                };
                let len = fractadyne_gpu::custom::trim_reference(&mut orbit);
                let Ok(shader) = fractadyne_gpu::custom::build(formula, params) else { continue };
                let power = shader.power as f64;
                req.width = N;
                req.height = N;
                req.ss = 1;
                req.mode = 0;
                req.sa_skip = 0;
                req.bla_on = 0;
                req.max_iter = budget;
                req.orbit = std::sync::Arc::new(orbit);
                req.orbit_len = len;
                req.ref_offset = fractadyne_gpu::RefOffset::ZERO;
                req.formula = fractadyne_core::formula::CUSTOM;
                req.custom = Some(std::sync::Arc::new(shader));
                let Some(gpu) = st_render_iter(device, queue, &req) else { continue };
                // The floatexp step on the same view: mode 2 forced, the df32 tail off (it would run
                // every step at 1e6×). Its explosive escapes are the ones the deep views of (c),
                // ~1e-11 wide, never meet: an untamed floatexp step passed every (c) check.
                let mut fe_req = req.clone();
                fe_req.mode = 2;
                fractadyne_gpu::set_tail_df32(false);
                let gpu_fe = st_render_iter(device, queue, &fe_req);
                fractadyne_gpu::set_tail_df32(true);
                let Some(gpu_fe) = gpu_fe else { continue };
                let scale = 2f64.powi(req.delta_exp);
                let (sx, sy) = (req.span_mantissa.x / N as f64, req.span_mantissa.y / N as f64);
                let cs: Vec<(f64, f64)> = (0..nn * nn)
                    .map(|k| {
                        let (i, j) = (k % nn, k / nn);
                        (
                            at.0 + sx * ((i as f64 + 0.5) - N as f64 * 0.5) * scale,
                            at.1 + sy * (N as f64 * 0.5 - (j as f64 + 0.5)) * scale,
                        )
                    })
                    .collect();
                let cpu = cpu_pixels(formula, params, power, &cs, budget);
                let px = (nn * nn) as u64;
                for (gpu, mode_text) in [(&gpu, "mode 0"), (&gpu_fe, "mode 2 forced, df32 tail off")] {
                    let (mut escaped, mut disagree, mut missed) = (0u64, 0u64, 0u64);
                    for (k, &cpu) in cpu.iter().enumerate() {
                        let g = gpu[k * 4] as f64;
                        escaped += (cpu >= 0.0) as u64;
                        let same = if cpu < 0.0 || g < 0.0 { (cpu < 0.0) == (g < 0.0) } else { (cpu - g).abs() < tol };
                        disagree += (!same) as u64;
                        missed += (cpu >= 0.0 && cpu < budget as f64 - 10.0 && g < 0.0) as u64;
                    }
                    let frac = disagree as f64 / px as f64;
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Custom formula (GPU)",
                        name: if mode_text == "mode 0" {
                            format!("{label}: perturbed GPU = CPU interpreter")
                        } else {
                            format!("{label}: floatexp perturbed GPU = CPU interpreter")
                        },
                        params: format!("1e6× at {:.9}{:+.9}i, {budget} iter, {mode_text}, ref {len}, {escaped} escaped px", at.0, at.1),
                        result: format!("{disagree} px disagree ({:.3}%), {missed} early escapes called interior", frac * 100.0),
                        // ⚠The 2% bound alone let through a NaN that froze escaping orbits:
                        // `z²·tanh z`'s perturbed step as sinh(δ)·sech·sech is inf·0 once δ is large,
                        // and 72 pixels escaping by iteration 43 rendered interior (0.149%). Pixels
                        // escaping 10 or more iterations before the budget are far from any rounding
                        // edge — measured 0–3 called interior per case with the step correct — so
                        // those get a bound of their own.
                        threshold: if budget > 60 {
                            "<2% disagree (status, or |Δ| ≥ 0.01); ≤10 escaping 10+ iter early called interior; >10% escaped, some interior"
                        } else {
                            "<2% disagree (status, or > 2 iter); ≤10 escaping 10+ iter early called interior; >10% escaped, some interior"
                        },
                        pass: frac < 0.02 && missed <= 10 && escaped * 10 > px && escaped < px,
                    });
                }
            }

            // (c) The APP's deep pipeline, end to end: the custom formula applied as the app holds
            //     it, and the export request built exactly as for any view — mode selection
            //     (`render_mode`), the reference from the IR (`render::custom_reference`), no SA, BLA
            //     or glitch correction — at 1e12×, 1e20× (past f64) and 1e40× (past f32's exponent
            //     floor, where the app picks floatexp perturbation; the short-budget formulas
            //     shallower, see below). The truth is the IR interpreter in bignum at each sampled
            //     pixel's own c (a 32×32 grid). The view is on the formula's boundary: the f64
            //     bisection above, continued in bignum along the same ray, since an f64 point is
            //     only good to ~1e-16.
            //     Every view is rendered three ways against that one truth: as the app picks the
            //     mode; with the floatexp path (mode 2) FORCED and its df32 tail OFF, so the
            //     generated floatexp step runs every step — at these depths the tail would take
            //     over at once (|δz| ≥ 2^-60) and the floatexp step of a function or quotient
            //     formula would never run; and that forced render again in resumable passes,
            //     which must match it bit for bit.
            let deep_prec = fractadyne_core::precision_for_magnification(1.0e40).max(64) + 32;
            let big = |v: f64| fractadyne_core::BigFloat::from_f64(v, deep_prec);
            // The escape is read off the orbit's samples at the GPU's bailout (256²): the reference
            // walk itself runs on to |z|² > 1e12. Also returned: how close any deciding sample came
            // to the bailout (relative, in |z|²) — a pixel within 1e-4 of it is decided by rounding
            // in ANY single-precision escape test, and is no test of the perturbation. (Measured: a
            // view bisected onto the curve |z₂₀₀₀| = 256 had every pixel within 1e-8 of it at
            // 1e20×; f64 perturbation and bignum split them, the GPU's f32 test called all interior.)
            let big_eval = |formula: &ir::Formula, params: &[(f64, f64)], c: &[fractadyne_core::BigFloat; 2], power: f64, budget: u32| {
                let z = big(0.0);
                let (orbit, _, _) = ir::reference_orbit(formula, &z, &z, &c[0], &c[1], params, budget, deep_prec)
                    .expect("a perturbable formula evaluates in bignum");
                let mut margin = f64::INFINITY;
                for (n, s) in orbit.iter().enumerate().skip(1) {
                    // Tamed as the GPU and the f64 interpreter tame an escaping value (an explosive
                    // step can pass f32's range, where the packed sample holds inf or NaN).
                    let (x, y) = fractadyne_gpu::custom::tame_f64(fractadyne_core::sample_xy(s));
                    let mag2 = x * x + y * y;
                    margin = margin.min((mag2 / bail2 - 1.0).abs());
                    if mag2 > bail2 {
                        let smooth = (n as f64 + 1.0 - (mag2.ln() * 0.5 / 2f64.ln()).ln() / power.ln()).max(0.0);
                        return (smooth, margin);
                    }
                }
                (-1.0, margin)
            };
            let big_smooth = |formula: &ir::Formula, params: &[(f64, f64)], c: &[fractadyne_core::BigFloat; 2], power: f64, budget: u32| {
                big_eval(formula, params, c, power, budget).0
            };
            // The boundary point along `toward`, in bignum, to within 1e-3 of a view at `mag`: the
            // f64 bisection, then a bracket in bignum (the f64 verdict need not hold there — on
            // `|z|·z + conj(z)² + c` the two disagreed for 1e-12 around it), then halvings of a
            // ≤1e-6 bracket. An f64 offset from one base point resolves only ~2e-22 of a 1e-6
            // bracket, so the base moves to the inside end whenever the halvings stall, and the
            // next round bisects the remaining bracket afresh — ~50 bits a round.
            let deep_boundary = |formula: &ir::Formula, padded: &[(f64, f64)], power: f64, toward: (f64, f64), budget: u32, mag: f64| {
                let at = boundary(formula, padded, toward, budget)?;
                let hyp = toward.0.hypot(toward.1);
                let dir = (toward.0 / hyp, toward.1 / hyp);
                let point = |base: &[fractadyne_core::BigFloat; 2], d: f64| {
                    [
                        fractadyne_core::add_f64(&base[0], d * dir.0, deep_prec),
                        fractadyne_core::add_f64(&base[1], d * dir.1, deep_prec),
                    ]
                };
                let mut base = [big(at.0), big(at.1)];
                let escapes = |base: &[fractadyne_core::BigFloat; 2], d: f64| {
                    big_smooth(formula, padded, &point(base, d), power, budget) >= 0.0
                };
                let steps = (6..=15).rev().map(|k| 10f64.powi(-k));
                let mut d_in = std::iter::once(0.0).chain(steps.clone().map(|d| -d)).find(|&d| !escapes(&base, d))?;
                let mut d_out = steps.clone().find(|&d| escapes(&base, d))?;
                let target = 1.0e-3 * 3.0 / mag;
                loop {
                    for _ in 0..64 {
                        let mid = 0.5 * (d_in + d_out);
                        if mid <= d_in || mid >= d_out {
                            break;
                        }
                        if escapes(&base, mid) { d_out = mid } else { d_in = mid }
                    }
                    base = point(&base, d_in);
                    let w = d_out - d_in;
                    if w <= target {
                        return Some(base);
                    }
                    (d_in, d_out) = (0.0, w);
                }
            };
            // Whether a 1e20× view at `centre` can be judged at all: a 3×3 probe, each point
            // decidable in f32 (see `big_eval`). A boundary piece where the escape time varies
            // smoothly is, at that depth, all one level curve of |z_n| — measured on the first
            // quadrant of `|z|·z + conj(z)² + c`: 0 of 1,024 samples decidable.
            // And MIXED: both statuses among the nine. A bisected point can be the edge of an
            // escaping sliver thinner than the sample spacing — measured, the Mandelbrot/Burning
            // Ship hybrid's at 1e40×: every one of 1,024 samples interior in bignum.
            // And STABLE: each keeps its status and escape (±2) with c moved 1e-12 of a pixel. The
            // same hybrid's third-quadrant view is chaotic — at 1e12× six sampled pixels escaped at
            // 470–1,714 in bignum, and at c ± 1e-12 px changed status or moved by 90–1,300
            // iterations; f64 perturbation and the GPU each gave other values again (83% of samples
            // "disagreed"). No finite precision can follow such a view, so it tests nothing.
            let decidable_at = |formula: &ir::Formula,
                                padded: &[(f64, f64)],
                                power: f64,
                                centre: &[fractadyne_core::BigFloat; 2],
                                budget: u32,
                                mag: f64,
                                need_mixed: bool| {
                let w = 3.0 / mag;
                let nudge = 1.0e-12 * w / N as f64;
                let pairs: Vec<((f64, f64), (f64, f64))> = (0..9)
                    .map(|k| {
                        let (i, j) = ((k % 3) as f64 - 1.0, (k / 3) as f64 - 1.0);
                        let at = |d: f64| {
                            [
                                fractadyne_core::add_f64(&centre[0], 0.4 * w * i + d, deep_prec),
                                fractadyne_core::add_f64(&centre[1], 0.4 * w * j + d, deep_prec),
                            ]
                        };
                        (big_eval(formula, padded, &at(0.0), power, budget), big_eval(formula, padded, &at(nudge), power, budget))
                    })
                    .collect();
                let ok = pairs.iter().filter(|(e, _)| e.1 >= 1.0e-4).count();
                let escaped = pairs.iter().filter(|(e, _)| e.0 >= 0.0).count();
                let stable = pairs
                    .iter()
                    .filter(|((a, _), (b, _))| if *a < 0.0 || *b < 0.0 { (*a < 0.0) == (*b < 0.0) } else { (a - b).abs() <= 2.0 })
                    .count();
                ok >= 8 && stable >= 8 && (!need_mixed || (escaped >= 1 && escaped <= 8))
            };
            // (b)'s ring cases, except that `|z|·z + conj(z)² + c` has no such view along any of the
            // four rays (its boundary is smooth there, measured), so conj comes in through a
            // Mandelbrot/Tricorn hybrid instead; and functions and division at (b)'s short budget,
            // where the bignum oracle is also affordable (sin and cos cost 110–560 µs a bignum
            // iteration, measured at 128–512 bits — 1,000× a ring step).
            let deep_cases: Vec<(&str, ir::Formula, Vec<(f64, f64)>, u32)> = vec![
                pert_cases[0].clone(),
                pert_cases[1].clone(),
                (
                    "hybrid Mandelbrot/Tricorn",
                    ir::Formula::new(vec![
                        ir::builtin_step(fractadyne_core::formula::MANDELBROT).unwrap(),
                        ir::builtin_step(fractadyne_core::formula::TRICORN).unwrap(),
                    ])
                    .unwrap(),
                    vec![],
                    2000,
                ),
                pert_cases[3].clone(),
                pert_cases[4].clone(),
                pert_cases[8].clone(),
            ];
            // The branch-cut functions and powers (at the short budget: a bignum `log`, `sqrt` or
            // power also costs ~1 ms an iteration at 1e40×'s precision — atan, ln, exp — and a
            // 2,000-iteration bisection took minutes a ray; measured, the group ran past 10 min).
            // (label, formula, params, budget, the rays a view is looked for along)
            let mut deep_cases: Vec<(String, ir::Formula, Vec<(f64, f64)>, u32, Vec<(f64, f64)>)> = deep_cases
                .into_iter()
                .chain(pert_cases[9..].iter().cloned())
                .map(|(l, f, p, b)| (l.to_string(), f, p, b, rays.to_vec()))
                .collect();
            // ⭐ACROSS THE CUT: formulas again, bisected along the NEGATIVE REAL AXIS, so the view's
            // centre — the reference — sits ON the branch cut (its upper side) with half the pixels
            // below it, and the orbit keeps returning to the axes. The views above missed sqrt's
            // cut: a sqrt that never took the difference branch passed them all (planted,
            // measured). A log whose crossings were never seen failed only the complex power's —
            // with `log(z + 1)`, whose argument stays positive on the axis; with `log(z + 0.5)` its
            // own deep view fails too (50%).
            // ⚠A crossing's jump is O(1) (`−2i√x` for sqrt, `2πi` for log), unlike a fold's, which
            // is as small as the reference's distance from the fold. In single precision it
            // swallows the pixel's own offset, so every pixel below then follows the same
            // conjugate-of-reference orbit: measured at 1e12×, exactly half of `√(z⁴ + c)`'s and
            // `z^2.5 + c`'s samples wrong (512 of 1,024, 493 of 986). The rule is exact (core's
            // `the_branch_cut_functions_jump_where_their_principal_values_do`); what fails is the
            // precision a single reference leaves. So these views test the crossing where a pixel
            // survives it — 1e5× and shallower, still perturbed (the direct path gives way at
            // ~1e4–1e5×) — and the limit is stated in Help.
            // Not `z^2.5 + c` nor the log formula: with the reference ON the cut, their orbits pass
            // near later crossings, where a pixel carries an O(1) δ and is then iterated at f32's
            // precision — 10–15% of samples off at 1e4–1e5×, while the f64 perturbation of the same
            // reference agreed with bignum on every one sampled. That is the single reference's
            // limit (a pixel on the other branch wants a reference of its own), not the rule's:
            // the log formula's ordinary deep view crosses too, and the planted "never a crossing"
            // broke it at 50%. The complex power covers the same `cf_pow_diff`.
            deep_cases.extend(
                pert_cases[11..]
                    .iter()
                    .map(|(l, f, p, b)| (format!("{l} across the cut"), f.clone(), p.clone(), *b, vec![(-1.0, 0.0)])),
            );
            let mut fe_depth_cases: Vec<String> = Vec::new();
            for (label, formula, params, budget, rays) in &deep_cases {
                let budget = *budget;
                self.render_cfg.max_iter = budget;
                let mut padded = params.clone();
                padded.resize(fractadyne_core::ir::parse::MAX_PARAMS, (0.0, 0.0));
                let Ok(shader) = fractadyne_gpu::custom::build(formula, &padded) else { continue };
                let power = shader.power as f64;
                // The depths. The ring formulas at 1e12×, 1e20× and the deepest of 1e40× and 1e32×
                // (both floatexp as the app picks it) where the view can be judged, else 1e20×
                // alone. At the short budget the view sits on the level curve |z₆₀| = bailout,
                // smooth at these scales, and the band of pixels within 1e-4 of it (in |z|², see
                // `big_eval`) has a FIXED width in c: 27% of a 1e12× view of `sin z + c`
                // (measured: probe margins 6e-5–4e-4 at 1.2e-12 off the curve), more of the
                // quotient's (8e-6–4e-5). Those test at the deepest of 1e12…1e9× where the view can
                // be judged — chosen by the bignum oracle alone, never by what the GPU renders, and
                // every one 1e4× past the f32 wall. The boundaries are found lazily, ray by ray (a
                // bignum sin bisection costs ~1.5 s).
                // Across the cut, only where single precision can still hold a pixel past the jump
                // (see the note at `deep_cases`): 1e5× or 1e4×, the shallowest perturbed depth.
                // After a crossing the pixel carries an O(1) δ and each later step evaluates its
                // function on full-size values in f32: the pixel is iterated at about the DIRECT
                // path's precision. And the view need not hold both statuses: every smooth value on
                // the far side depends on the branch (the planted sqrt broke an ALL-escaping view at
                // 50%).
                let across = label.ends_with("across the cut");
                let candidates: &[f64] = if across {
                    &[1.0e5, 1.0e4]
                } else if budget > 60 {
                    &[1.0e40, 1.0e32, 1.0e20]
                } else {
                    &[1.0e12, 1.0e11, 1.0e10, 1.0e9, 1.0e8, 1.0e7]
                };
                let mut boundaries: Vec<Option<Option<[fractadyne_core::BigFloat; 2]>>> = vec![None; rays.len()];
                let mut found = None;
                'depth: for &mag in candidates {
                    for (r, &toward) in rays.iter().enumerate() {
                        let c = boundaries[r]
                            .get_or_insert_with(|| deep_boundary(formula, &padded, power, toward, budget, candidates[0]));
                        if let Some(c) = c.as_ref().filter(|c| decidable_at(formula, &padded, power, c, budget, mag, !across)) {
                            found = Some((c.clone(), toward, mag));
                            break 'depth;
                        }
                    }
                }
                let Some((centre, toward, deepest)) = found else {
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Custom formula (GPU)",
                        name: format!("{label}: deep app pipeline = bignum"),
                        params: String::new(),
                        result: format!(
                            "no boundary view decidable in f32 at {:.0e}×–{:.0e}× along {} rays",
                            candidates[0],
                            candidates[candidates.len() - 1],
                            rays.len()
                        ),
                        threshold: "a boundary to test at",
                        pass: false,
                    });
                    continue;
                };
                let mut mags: Vec<f64> = if budget > 60 { vec![1.0e12, 1.0e20, deepest] } else { vec![deepest] };
                mags.dedup();
                if deepest >= crate::tunables::PERT_FE_THRESHOLD {
                    fe_depth_cases.push(format!("{label} at {deepest:.0e}×"));
                }
                self.fractal = FractalKind::Custom;
                self.julia_mode = false;
                self.custom = Some(std::sync::Arc::new(crate::custom_formula::CustomFormula {
                    source: label.to_string(),
                    params: padded.clone(),
                    formula: formula.clone(),
                    shader: std::sync::Arc::new(shader),
                }));
                for &mag in &mags {
                    let mut vp = Viewport::new(N as f64, N as f64);
                    vp.center_x = centre[0].clone();
                    vp.center_y = centre[1].clone();
                    vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mag));
                    vp.precision = fractadyne_core::precision_for_magnification(mag).max(64);
                    let mut req = self.current_export_request_for(&vp, false);
                    req.width = N;
                    req.height = N;
                    req.ss = 1;
                    let (mode, orbit_len, has_custom) = (req.mode, req.orbit_len, req.custom.is_some());
                    let want_mode = if mag >= crate::tunables::PERT_FE_THRESHOLD { 2 } else { 0 };
                    let Some(gpu) = st_render_iter(device, queue, &req) else { continue };
                    // The floatexp step on every step: mode 2 forced, the df32 tail off (a
                    // process-wide switch, restored at once), single pass and in passes. A mode-0
                    // request carries everything mode 2 reads (the offsets are mantissas at
                    // `delta_exp` on both paths).
                    let mut fe_req = req.clone();
                    fe_req.mode = 2;
                    fractadyne_gpu::set_tail_df32(false);
                    let fe_single = st_render_iter(device, queue, &fe_req);
                    // Odd windows, several per render at either budget.
                    let window = if budget > 60 { 517 } else { 17 };
                    let mut passes = Vec::new();
                    let fe_chunked = fractadyne_gpu::render_iter_chunked_timed(device, queue, &fe_req, window, &mut passes)
                        .map_err(|e| eprintln!("[selftest] GPU ERROR (render_iter_chunked, custom floatexp): {e}"))
                        .ok();
                    fractadyne_gpu::set_tail_df32(true);
                    let (Some(fe_single), Some(fe_chunked)) = (fe_single, fe_chunked) else { continue };
                    let bit_diffs = |a: &[f32], b: &[f32]| {
                        if a.len() == b.len() {
                            a.iter().zip(b).filter(|(x, y)| x.to_bits() != y.to_bits()).count()
                        } else {
                            usize::MAX
                        }
                    };
                    let chunk_diffs = bit_diffs(&fe_single, &fe_chunked.pixels);
                    // In mode 2 as the app runs it (the tail on), the floatexp steps hand over to
                    // the df32 tail mid-orbit — in passes too, where a pass can end on either side.
                    let mut app_passes = Vec::new();
                    let app_chunk_diffs = if mode == 2 {
                        fractadyne_gpu::render_iter_chunked_timed(device, queue, &req, window, &mut app_passes)
                            .map(|r| bit_diffs(&gpu, &r.pixels))
                            .unwrap_or(usize::MAX)
                    } else {
                        0
                    };
                    let scale = 2f64.powi(req.delta_exp);
                    let (sx, sy) = (req.span_mantissa.x / N as f64, req.span_mantissa.y / N as f64);
                    const G: usize = 32;
                    let samples: Vec<(usize, [fractadyne_core::BigFloat; 2])> = (0..G * G)
                        .map(|k| {
                            let (i, j) = ((k % G) * nn / G + nn / (2 * G), (k / G) * nn / G + nn / (2 * G));
                            let c = [
                                fractadyne_core::add_f64(&centre[0], sx * ((i as f64 + 0.5) - N as f64 * 0.5) * scale, deep_prec),
                                fractadyne_core::add_f64(&centre[1], sy * (N as f64 * 0.5 - (j as f64 + 0.5)) * scale, deep_prec),
                            ];
                            (j * nn + i, c)
                        })
                        .collect();
                    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
                    let evals: Vec<(f64, f64)> = std::thread::scope(|s| {
                        let parts: Vec<_> = samples
                            .chunks(samples.len().div_ceil(threads))
                            .map(|part| {
                                let padded = &padded;
                                s.spawn(move || part.iter().map(|(_, c)| big_eval(formula, padded, c, power, budget)).collect::<Vec<_>>())
                            })
                            .collect();
                        parts.into_iter().flat_map(|h| h.join().expect("a bignum oracle thread panicked")).collect()
                    });
                    // An explosive escape's smooth value moves ~0.1 between f32 and exact (see
                    // (b)), so the median bound is the ring formulas' 0.01 only for them.
                    let med_tol = if budget > 60 { 0.01 } else { 0.25 };
                    // Judged as the twin check judges: the same status and within 2 iterations.
                    // Single-precision perturbation drifts from the exact value on pixels escaping
                    // late in a chaotic region (Mandelbrot/Tricorn at 1e12×: 0.01–11 iterations on
                    // 47 of 1,024, all escaping past iteration 1,860), as the built-ins do there;
                    // what this check exists for — the pipeline's mode, reference and module — gets
                    // most pixels wrong when it breaks. An offset in the smooth value moves every
                    // pixel, so the median error over pixels escaped in both is held to 0.01.
                    // → (disagree, judged, escaped, median |Δ|) of one render against the truth.
                    let judge = |gpu: &[f32]| {
                        let (mut escaped, mut disagree, mut judged, mut errs) = (0usize, 0usize, 0usize, Vec::new());
                        for ((k, _), &(cpu, margin)) in samples.iter().zip(&evals) {
                            if margin < 1.0e-4 {
                                continue;
                            }
                            judged += 1;
                            let g = gpu[k * 4] as f64;
                            escaped += (cpu >= 0.0) as usize;
                            let same = if cpu < 0.0 || g < 0.0 { (cpu < 0.0) == (g < 0.0) } else { (cpu - g).abs() <= 2.0 };
                            disagree += (!same) as usize;
                            if cpu >= 0.0 && g >= 0.0 {
                                errs.push((cpu - g).abs());
                            }
                        }
                        let median = if errs.is_empty() {
                            f64::INFINITY
                        } else {
                            let mid = errs.len() / 2;
                            *errs.select_nth_unstable_by(mid, f64::total_cmp).1
                        };
                        (disagree, judged, escaped, median)
                    };
                    let n = samples.len();
                    // ≥3/4 of the samples decidable, for enough of them to judge by: a view on the
                    // budget's own level curve (as the bisection puts it, since it counts an escape
                    // on the last step) keeps a band around it that no f32 escape test decides, a
                    // fixed width in c — measured 16–20% of the short-budget views of `√(z⁴ + c)`
                    // at 1e10× and the sin/cos formula at 1e12× (822 and 855 of 1,024 judged).
                    let judged_ok = |(disagree, judged, escaped, median): (usize, usize, usize, f64)| {
                        judged * 4 >= n * 3
                            && disagree * 50 < judged
                            && median < med_tol
                            && if across { escaped > 0 } else { escaped * 10 > judged && escaped < judged }
                    };
                    let tol_text = if budget > 60 { "0.01" } else { "0.25" };
                    let app = judge(&gpu);
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Custom formula (GPU)",
                        name: format!("{label}: deep app pipeline = bignum at {mag:.0e}×"),
                        params: format!(
                            "boundary toward {}{:+}i, {budget} iter, mode {mode}, ref {orbit_len}, {} of {n} sampled px decidable in f32, {} of them escaped",
                            toward.0, toward.1, app.1, app.2
                        ),
                        result: format!(
                            "{} of {} disagree ({:.2}%), median |Δ| {:.5}{}",
                            app.0,
                            app.1,
                            app.0 as f64 * 100.0 / app.1.max(1) as f64,
                            app.3,
                            if mode == 2 {
                                format!("; in {} passes {app_chunk_diffs} texels differ", app_passes.len())
                            } else {
                                String::new()
                            }
                        ),
                        threshold: format!(
                            "mode {want_mode} with the custom module; ≥75% decidable; <2% differ in status or by >2 iter; median |Δ| < {tol_text}; {}{}",
                            if across { "some escaped (either side of the cut)" } else { ">10% escaped, some interior" },
                            if want_mode == 2 { "; in ≥2 passes 0 texels differ" } else { "" }
                        )
                        .leak(),
                        pass: mode == want_mode
                            && has_custom
                            && judged_ok(app)
                            && (mode != 2 || (app_passes.len() >= 2 && app_chunk_diffs == 0)),
                    });
                    let fe = judge(&fe_single);
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Custom formula (GPU)",
                        name: format!("{label}: floatexp step (mode 2 forced, df32 tail off) = bignum at {mag:.0e}×"),
                        params: format!(
                            "{budget} iter, ref {orbit_len}, the same samples ({} decidable, {} escaped), and the render in passes of {window}",
                            fe.1, fe.2
                        ),
                        result: format!(
                            "{} of {} disagree ({:.2}%), median |Δ| {:.5}; in {} passes {} texels differ",
                            fe.0,
                            fe.1,
                            fe.0 as f64 * 100.0 / fe.1.max(1) as f64,
                            fe.3,
                            passes.len(),
                            chunk_diffs
                        ),
                        threshold: format!(
                            "as the app pipeline's (median |Δ| < {tol_text}); ≥2 passes, 0 texels differ"
                        )
                        .leak(),
                        pass: judged_ok(fe) && passes.len() >= 2 && chunk_diffs == 0,
                    });
                }
            }
            // The fallback above must not quietly drop the floatexp depth for every formula.
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Custom formula (GPU)",
                name: "deep app pipeline reaches the floatexp depth".into(),
                params: String::new(),
                result: if fe_depth_cases.is_empty() { "none".into() } else { fe_depth_cases.join(", ") },
                threshold: "at least one formula tested where the app picks floatexp (≥1e28×)",
                pass: !fe_depth_cases.is_empty(),
            });
            self.render_cfg.max_iter = max_iter;

            // (d) RESUMABLE PASSES: a custom module carries its own chunk pass (`fs_iterate_chunk`
            //     with the generated step spliced in), so a custom view splits its iterations
            //     across passes like a built-in. As for the built-ins ("chunked render is
            //     bit-identical"): odd window sizes, so boundaries land mid-phase for a hybrid, and a
            //     rebase storm (a 97-sample reference) so they land on rebases. Also covered: z_{n-1}
            //     carried across passes (Phoenix's step reads it; the chunk pass keeps it where a
            //     built-in keeps its derivative) and Julia mode (where that slot starts at 1, not 0).
            //     ⚠Each render must SHOW passes: `render_iter_chunked_timed` falls back to one
            //     unbounded dispatch out of scope, which would agree trivially.
            let phoenix = ir::Formula::single(ir::builtin_step(fractadyne_core::formula::PHOENIX).unwrap());
            let hybrid = pert_cases[0].1.clone();
            let chunk_cases: Vec<(&str, ir::Formula, Vec<(f64, f64)>, bool, f64, u32, u32, bool, u32)> = vec![
                // (label, formula, params, julia, magnification, max_iter, window, truncate, mode)
                ("Phoenix step (reads z_{n-1}), direct", phoenix.clone(), vec![], false, 1.0, 2_000, 137, false, 1),
                ("sin z + c (f32 functions), direct", ir::parse::parse("sin(z) + c").unwrap(), vec![], false, 1.0, 600, 37, false, 1),
                ("hybrid Mandelbrot/Burning Ship, direct", hybrid.clone(), vec![], false, 1.0, 2_000, 137, false, 1),
                ("z³ − p·z + c (a parameter), direct", ir::parse::parse("z^3 - p1*z + c").unwrap(), vec![(0.4, 0.0)], false, 1.0, 2_000, 137, false, 1),
                // Julia mode with a step that reads z_{n-1}: the slot carrying it starts at 1 there.
                ("Phoenix step, Julia, direct", phoenix, vec![], true, 1.0, 2_000, 137, false, 1),
                ("hybrid Mandelbrot/Burning Ship, perturbed 1e6×", hybrid.clone(), vec![], false, 1.0e6, 3_000, 517, false, 0),
                ("hybrid Mandelbrot/Burning Ship, 7-sample reference (rebase storm)", hybrid, vec![], false, 1.0e6, 3_000, 517, true, 0),
            ];
            for (label, formula, params, julia, mag, max_iter, window, truncate, want_mode) in chunk_cases {
                let mut padded = params.clone();
                padded.resize(fractadyne_core::ir::parse::MAX_PARAMS, (0.0, 0.0));
                let Ok(shader) = fractadyne_gpu::custom::build(&formula, &padded) else { continue };
                // Perturbed cases sit on the formula's boundary (as (b)); direct ones at home.
                let at = if mag > 1.0e4 { boundary(&formula, &padded, first, max_iter) } else { Some((-0.5, 0.0)) };
                let Some(at) = at else {
                    push_check(&mut checks, &mut last_check_t, no_boundary(format!("{label}: chunked = single pass"), first));
                    continue;
                };
                self.fractal = FractalKind::Custom;
                self.julia_mode = julia;
                self.julia_c = (0.56667, 0.0); // the classic Phoenix Julia constant (p = −0.5)
                self.custom = Some(std::sync::Arc::new(crate::custom_formula::CustomFormula {
                    source: label.to_string(),
                    params: padded.clone(),
                    formula: formula.clone(),
                    shader: std::sync::Arc::new(shader),
                }));
                self.render_cfg.max_iter = max_iter;
                let mut vp = Viewport::new(N as f64, N as f64);
                vp.center_x = bf(at.0);
                vp.center_y = bf(at.1);
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * mag));
                vp.precision = fractadyne_core::precision_for_magnification(mag).max(64);
                let mut req = self.current_export_request_for(&vp, julia);
                req.width = N;
                req.height = N;
                req.ss = 1;
                let built_len = req.orbit_len;
                if truncate {
                    // 7 samples, not the built-ins' 97: here pixels Zhuoran-rebase about every ten
                    // iterations, so a 97-sample cut was never reached (measured: the same 14.9M
                    // rebases, the same image). Odd, so the end-of-orbit rebase alternates phase.
                    let short: Vec<[f32; 4]> = req.orbit.iter().take(7).copied().collect();
                    req.orbit = std::sync::Arc::new(short);
                    req.orbit_len = 7;
                }
                let single = st_render_iter(device, queue, &req);
                let mut passes = Vec::new();
                let chunked = fractadyne_gpu::render_iter_chunked_timed(device, queue, &req, window, &mut passes)
                    .map_err(|e| eprintln!("[selftest] GPU ERROR (render_iter_chunked, custom): {e}"))
                    .ok();
                let (pass, result) = match (&single, &chunked) {
                    _ if req.mode != want_mode || req.custom.is_none() => {
                        (false, format!("ran in mode {} (custom module {}), not mode {want_mode}", req.mode, req.custom.is_some()))
                    }
                    (Some(a), Some(r)) if a.len() == r.pixels.len() => {
                        let diffs = a.iter().zip(&r.pixels).filter(|(x, y)| x.to_bits() != y.to_bits()).count();
                        let px = a.len() / 4;
                        let escaped = (0..px).filter(|&k| a[k * 4] >= 0.0).count();
                        let reb = r.counters[fractadyne_gpu::CTR_REBASE];
                        (
                            diffs == 0 && passes.len() >= 2 && escaped * 10 > px && escaped < px,
                            format!(
                                "mode {}, ref {built_len}→{}, {} passes — {diffs} texels differ; {escaped} of {px} px escaped, rebase {reb}",
                                req.mode,
                                req.orbit_len,
                                passes.len()
                            ),
                        )
                    }
                    _ => (false, "render failed".into()),
                };
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Custom formula (GPU)",
                    name: format!("{label}: chunked = single pass"),
                    params: format!("{mag:.0e}×, {max_iter} iter, window {window}"),
                    result,
                    threshold: "0 texels differ, ≥2 passes, >10% escaped, some interior",
                    pass,
                });
            }
            self.julia_mode = false;
            self.fractal = FractalKind::Mandelbrot;
        }

        // ---- Life (design/automata.md): the GPU tile stepper equals the CPU one cell for cell
        // (10 rule kinds × plane / torus / bounded), the tile set follows a glider, a full pool
        // stops instead of dropping cells, LifeWiki's long-run facts hold on the GPU, and the
        // display pass writes the values it is defined to. Integer automata: every check exact. ----
        if want("life") {
            if fractadyne_gpu::life::life_available(device) {
                use fractadyne_gpu::life::check;
                let mut outcomes = check::stepper_matches_cpu(device, queue);
                outcomes.extend(check::tile_set_and_pool(device, queue));
                outcomes.extend(check::known_facts(device, queue));
                outcomes.extend(check::display(device, queue));
                for o in outcomes {
                    let (pass, result) = match o.result {
                        Ok(s) => (true, s),
                        Err(e) => (false, e),
                    };
                    push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Life",
                        name: o.name,
                        params: o.params,
                        result,
                        threshold: "exact",
                        pass,
                    });
                }
            } else {
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "Life",
                    name: "the device runs the Life stepper".into(),
                    params: "compute shaders, 7 storage buffers".into(),
                    result: "this adapter has no compute shaders (a GL backend?)".into(),
                    threshold: "available",
                    pass: false,
                });
            }
        }

        // ---- L-systems (design/lsystems.md): the culling walk draws what the naive way (build
        // the word, run a turtle) draws, for every system in the library; at its home view it
        // draws no more than its pixels allow; and the segment pass covers what its model says,
        // texel for texel. ----
        if want("lsystem") {
            use fractadyne_core::lsystem::{self as ls, library, reference};
            let (mut compared, mut bad, mut worst) = (0usize, Vec::new(), 0.0f64);
            let mut heavy = Vec::new();
            let mut systems = 0;
            for e in library::SYSTEMS {
                let Ok(s) = e.system() else {
                    bad.push(format!("{}: does not parse", e.name));
                    continue;
                };
                // (Parametric and context-sensitive systems are built as words: the next check.)
                if s.expanded.is_some() {
                    continue;
                }
                let t = ls::Tables::new(&s);
                // The highest order up to 6 that draws, small enough to build the word for.
                let Some(order) = (1..=6u32).rev().find(|&n| (1.0..=50_000.0).contains(&t.axiom_entry(n).n)) else { continue };
                systems += 1;
                // A pixel a step, so the tolerance below is in steps.
                let u = t.step(order);
                let view = ls::View { centre: [0.0, 0.0], upp: u[0].hypot(u[1]), size: [f64::INFINITY; 2], margin: 0.0 };
                let mut walked = Vec::new();
                ls::walk(&t, &view, &ls::WalkOptions { order, lod_px: 0.0, budget: u64::MAX }, &mut |g| walked.push(*g));
                let word = reference::expand(&s, order, 2_000_000).unwrap_or_default();
                let want = reference::draw(&s, &word, [0.0, 0.0], [u[0] / view.upp, u[1] / view.upp]);
                compared += want.len();
                if walked.len() != want.len() {
                    bad.push(format!("{} order {order}: {} segments, the reference {}", e.name, walked.len(), want.len()));
                    continue;
                }
                for (g, w) in walked.iter().zip(&want) {
                    let d = (g.a[0] - w.a[0]).abs().max((g.a[1] - w.a[1]).abs()).max((g.b[0] - w.b[0]).abs()).max((g.b[1] - w.b[1]).abs());
                    worst = worst.max(d);
                    if d > 1e-6 || g.index != w.index || g.depth != w.depth || g.colour != w.colour {
                        bad.push(format!("{} order {order}: segment {} differs by {d:.2e}", e.name, g.index));
                        break;
                    }
                }
                // The home view at the order that follows the zoom: at most 4 segments a pixel.
                let home = ls::framing_order(&t, 20_000.0);
                if let Some(b) = ls::bounds(&t, home, 1 << 22) {
                    let upp = ((b[2] - b[0]).max(b[3] - b[1]) / 600.0).max(1e-12);
                    let size = [(b[2] - b[0]) / upp + 2.0, (b[3] - b[1]) / upp + 2.0];
                    let v = ls::View { centre: [(b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0], upp, size, margin: 1.0 };
                    let order = t.auto_order(1.0 / upp, 3.0).unwrap_or_else(|| s.order.unwrap_or(6)).min(t.max_depth);
                    let stats = ls::walk(&t, &v, &ls::WalkOptions { order, lod_px: 1.5, budget: 20_000_000 }, &mut |_| {});
                    if stats.segments as f64 > 4.0 * size[0] * size[1] {
                        heavy.push(format!("{}: {} segments for {:.0} pixels", e.name, stats.segments, size[0] * size[1]));
                    }
                }
            }
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "L-system",
                name: "the culling walk draws what the naive turtle draws".into(),
                params: format!("{systems} library systems, order ≤ 6, {} segments", crate::commas(&compared.to_string())),
                result: if bad.is_empty() { format!("all equal; worst end-point difference {worst:.1e} px") } else { bad.join("; ") },
                threshold: "same segments, in order, ends within 1e-6 px",
                pass: bad.is_empty() && compared > 0,
            });
            // Parametric and context-sensitive systems: the words The Algorithmic Beauty of Plants
            // prints (equation 1.7, Figure 1.34) and works through (the signal of Figure 1.30), and
            // Hogeweg and Hesper's plant (Figure 1.31a) worked by hand from its productions.
            {
                let cases: [(&str, &str, u32, &str); 3] = [
                    (
                        "ABOP 1.7",
                        "angle 90\naxiom B(2)A(4,4)\nA(x,y) : y <= 3 = A(x*2, x+y)\nA(x,y) : y > 3 = B(x)A(x/y, 0)\n\
                         B(x) : x < 1 = C\nB(x) : x >= 1 = B(x-1)\n",
                        4,
                        "CB(1)A(8,7)",
                    ),
                    ("ABOP 1.30a", "angle 45\nignore +-\naxiom b[+a]a[-a]a[+a]a\nb < a = b\n", 2, "b[+b]b[-b]b[+a]a"),
                    (
                        "ABOP 1.31a",
                        "angle 22.5\nignore +-F\naxiom F1F1F1\n0 < 0 > 0 = 0\n0 < 0 > 1 = 1[+F1F1]\n0 < 1 > 0 = 1\n\
                         0 < 1 > 1 = 1\n1 < 0 > 0 = 0\n1 < 0 > 1 = 1F1\n1 < 1 > 0 = 0\n1 < 1 > 1 = 0\n\
                         * < + > * = -\n* < - > * = +\n",
                        5,
                        "F1F1F1F1[-F0F1]F1",
                    ),
                ];
                let mut wrong = Vec::new();
                for (name, text, order, want) in cases {
                    let got = ls::LSystem::parse(text).ok().and_then(|s| {
                        let e = s.expanded.clone()?;
                        Some(ls::expand::expand(&s, &e, order, ls::EXPAND_BUDGET).word.text())
                    });
                    if got.as_deref() != Some(want) {
                        wrong.push(format!("{name} at {order}: {got:?}, not {want}"));
                    }
                }
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "L-system",
                    name: "parametric and context-sensitive systems derive as ABOP prints them".into(),
                    params: "equation 1.7 at 4, Figure 1.30a at 2, Figure 1.31a at 5".into(),
                    result: if wrong.is_empty() { "all three words as printed".into() } else { wrong.join("; ") },
                    threshold: "the same words, module for module",
                    pass: wrong.is_empty(),
                });
            }
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "L-system",
                name: "a home view costs what its pixels cost".into(),
                params: "every library system framed at 600 px, the order that follows the zoom".into(),
                result: if heavy.is_empty() { "all within 4 segments a pixel".into() } else { heavy.join("; ") },
                threshold: "≤ 4 segments a pixel",
                pass: heavy.is_empty(),
            });
            // Unlimited zoom (phase 3): the Koch curve zoomed 3^100 (5e47×) about its END — where
            // the coordinates are 1 − 3^-100 and every bit counts — is the Koch curve again, 100
            // orders up, segment for segment.
            {
                let s = library::find("Koch curve").expect("in the library").system().expect("parses");
                let t = ls::Tables::new(&s);
                let (n, k, size) = (6u32, 100u32, [300.0, 200.0]);
                let upp: f64 = 0.3 / 300.0;
                let mut shallow = Vec::new();
                ls::walk(&t, &ls::View { centre: [0.8, 0.05], upp, size, margin: 1.5 }, &ls::WalkOptions { order: n, lod_px: 0.0, budget: u64::MAX }, &mut |g| shallow.push(*g));
                let upp_log2 = upp.log2() - f64::from(k) * 3f64.log2();
                let p = ls::deep_precision(&t, n + k, upp_log2, 1.0);
                let result = ls::BigTables::new(&s, &t, p, n + k).map(|bt| {
                    // The centre exactly, as the coordinate field reads an expression.
                    let expr = |s: String| fractadyne_core::parse_bf_prec(&s, p).expect("an expression");
                    let cx = expr(format!("1 - 0.2*3^-{k}"));
                    let cy = expr(format!("0.05*3^-{k}"));
                    let view = ls::DeepView { centre: [cx, cy], upp_log2, size, margin: 1.5 };
                    let mut deep = Vec::new();
                    ls::deep_walk(&t, &bt, &view, &ls::WalkOptions { order: n + k, lod_px: 0.0, budget: u64::MAX }, ls::switch_px(&t), &mut |g| deep.push(*g));
                    let inner = |g: &ls::Segment| g.a[0].abs() < 140.0 && g.a[1].abs() < 90.0;
                    let near = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3;
                    let want: Vec<&ls::Segment> = shallow.iter().filter(|g| inner(g)).collect();
                    let matched = want.iter().filter(|g| deep.iter().any(|d| near(g.a, d.a) && near(g.b, d.b))).count();
                    (matched, want.len(), deep.len())
                });
                let (pass, result) = match result {
                    Some((m, w, d)) => (m == w && w > 100, format!("{m} of {w} segments in place to 1e-3 px ({d} drawn deep, {} shallow)", shallow.len())),
                    None => (false, "the deep tables could not be built".into()),
                };
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "L-system",
                    name: "the Koch curve zoomed 3^100 about its end is itself".into(),
                    params: format!("order 6 at 300×200 vs order 106 at 5e47×, {p}-bit tables"),
                    result,
                    threshold: "every inner segment matched to 1e-3 px",
                    pass,
                });
            }
            for o in fractadyne_gpu::lsystem::check::coverage(device, queue) {
                let (pass, result) = match o.result {
                    Ok(s) => (true, s),
                    Err(e) => (false, e),
                };
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "L-system",
                    name: o.name,
                    params: o.params,
                    result,
                    threshold: "exact, edge texels aside",
                    pass,
                });
            }
            // Image export draws the view offscreen in tiles: the tiles must make the one picture.
            let (pass, result) = match crate::lsystem_view::export::tiling_check(device, queue) {
                Ok(s) => (true, s),
                Err(e) => (false, e),
            };
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "L-system",
                name: "an image export is the same in tiles as whole".into(),
                params: "mango leaf, 333×221, 2× supersampled, 64-texel tiles vs one".into(),
                result,
                threshold: "edge pixels only, ≤ 0.5%",
                pass,
            });
            // A tour of an L-system: "Tour from current view" of a plant zoomed in, read back by the
            // tour reader — starting at the plant's framed home with its system, order and angle,
            // ending at the view — and the end's frame drawn as the tour renderer draws it.
            {
                use std::f64::consts::{LN_10, LN_2};
                let (fractal, viewport) = (self.fractal, self.viewport.clone());
                self.fractal = FractalKind::LSystem;
                if let Some(s) = library::find("Plant (ABOP 1.24a)").and_then(|e| e.system().ok()) {
                    self.lsystem.set_system(s);
                }
                self.lsystem.set_angle(Some(30.0));
                self.lsystem.fixed_order = Some(5);
                self.viewport = fractadyne_core::Viewport::new(640.0, 360.0);
                let ([hx, hy], home_l10) = self.lsystem_home_view();
                // 16× in on the home view's centre: past "deep", so the tour recentres, then dives.
                let want_log2 = home_l10 * std::f64::consts::LOG2_10 + 4.0;
                let bf = |v: f64| fractadyne_core::BigFloat::from_f64(v, 128);
                self.viewport.set_center_log2mag(bf(hx), bf(hy), want_log2);
                let text = self.build_dive_script("", 4.0);
                let (pass, result) = match crate::scripting::parse_tour_text(&text) {
                    Err(e) => (false, format!("the tour does not parse: {e}")),
                    Ok(pb) => {
                        let (a, b) = (pb.sample(0.0), pb.sample(pb.total));
                        let start = a.fractal == FractalKind::LSystem
                            && a.ls.system.as_deref() == Some(&self.lsystem.system)
                            && a.ls.order == Some(5)
                            && a.ls.angle == Some(30.0)
                            && (a.logmag / LN_10 - home_l10).abs() < 1e-6;
                        let end = (b.logmag / LN_2 - want_log2).abs() < 1e-6;
                        self.apply_tour_lsystem(&b.ls);
                        self.viewport = fractadyne_core::Viewport::new(320.0, 180.0);
                        self.viewport.set_center_log2mag(b.cx, b.cy, b.logmag / LN_2);
                        let bg = self.interior_color();
                        let lit = self.lsystem_tour_frame(device, queue, [320, 180]).map_or(0, |r| {
                            r.pixels.chunks(4).filter(|p| (0..3).any(|k| (p[k] - bg[k]).abs() > 1e-3)).count()
                        });
                        (
                            start && end && lit > 300,
                            format!("start at the plant's home with its system, order and angle {start}; ends at the view {end}; the end's frame drew {lit} pixels"),
                        )
                    }
                };
                self.fractal = fractal;
                self.viewport = viewport;
                self.lsystem = crate::lsystem_view::LSystemState::default();
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "L-system",
                    name: "a tour from an L-system view starts at its home and draws its end".into(),
                    params: "ABOP 1.24a at 30°, order 5, 16× in: Tour from current view, read back".into(),
                    result,
                    threshold: "start, end, > 300 pixels drawn",
                    pass,
                });
            }
        }

        // ---- series approximation engages for the Multibrot families ----
        // The order-3 coefficient recurrence for z^d is validated exactly in fractadyne-core;
        // here we confirm the app actually selects SA for these formulas (skip > 0) and the
        // GPU render is finite and bit-consistent with an SA-off render (the seed shader code
        // is formula-agnostic, already validated for Mandelbrot in modes 0 and 2).
        // ⛔A SEED PAST THE BAILOUT (found 2026-10-01, design/power-families.md phase 2). The walk
        // stops at the REFERENCE's escape, |Z|² > 1e12, so a skip could land two samples before
        // the reference's end with |Z| far past 256; the GPU's first test of the seeded pixel, a
        // step later, squared |z| past f32: |z|² = ∞, every smooth value −∞, a black frame —
        // Multibrot 5 at its first row's view here (a released family), Multibrot 6 at its deep
        // point at 1e45×. `sa_seed_max2` bounds the seed. These views are featureless, every pixel
        // escaping on the reference's own step, so ONE interior or non-finite pixel is the bug.
        if want("multibrot-sa") {
            let prev = self.fractal;
            self.julia_mode = false;
            self.coloring.color_method = crate::ColorMethod::Smooth;
            self.coloring.use_custom_palette = false;
            self.render_cfg.auto_iter = false;
            self.render_cfg.max_iter = 30_000;
            self.render_cfg.series_approx = true;
            // BLA subsumes SA in floatexp: with a tree built no seed is walked (phase 3). The
            // seed is what this asks about, so the tree is kept out of these views.
            let saved_bla = self.render_cfg.use_bla;
            self.render_cfg.use_bla = false;
            let mut rows: Vec<(FractalKind, String, String, f64)> = Vec::new();
            for kind in [
                FractalKind::Multibrot4,
                FractalKind::Multibrot5,
                FractalKind::Multibrot6,
                FractalKind::Multibrot7,
                FractalKind::Multibrot8,
            ] {
                if let Some(at) = family_view(kind.formula_id(), 1.0e5, N, 3_000) {
                    rows.push((kind, format!("{:.17}", at.0), format!("{:.17}", at.1), 1.0e40));
                }
            }
            let (k6, x6, y6) = MULTIBROT_DEEP[3];
            debug_assert!(k6 == FractalKind::Multibrot6);
            rows.push((k6, x6.into(), y6.into(), 7.5e44));
            for (kind, x, y, mag) in rows {
                self.fractal = kind;
                let mut req = make(self, &x, &y, mag);
                req.max_iter = 30_000;
                let name = format!("{} SA seed stays in f32 range ({mag:.1e}×, featureless)", kind.name());
                let params = format!("mode {}, skip {} of a {}-sample reference", req.mode, req.sa_skip, req.orbit_len);
                match render(&req) {
                    Some(px) => {
                        let v: Vec<f32> = px.iter().step_by(4).copied().collect();
                        let bad = v.iter().filter(|x| !(x.is_finite() && **x >= 0.0)).count();
                        let engaged = req.sa_skip > 0 && req.orbit_len < req.max_iter;
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "Series approximation",
                            name,
                            params,
                            result: format!("{bad} of {} pixels interior or non-finite", v.len()),
                            threshold: "SA engaged, the reference escapes, every pixel escapes finite",
                            pass: engaged && bad == 0,
                        });
                    }
                    None => push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Series approximation",
                        name,
                        params,
                        result: "render failed".into(),
                        threshold: "SA engaged, the reference escapes, every pixel escapes finite",
                        pass: false,
                    }),
                }
            }
            self.render_cfg.use_bla = saved_bla;
            self.fractal = prev;
        }
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
            // (fractal, centre, a boundary view). The INTERIOR view (0.2, 0.1) is the original
            // three rows: every pixel stays interior through a skip of 3,999, and SA on and off
            // must agree exactly. ⚠It has no escaping pixel, so it could not see an SA that
            // misplaces an escape (found 2026-10-01, design/power-families.md phase 2, when
            // Multibrot 6–8 joined). So every power also gets a BOUNDARY view (`family_view`).
            // There SA on and off part on thousands of pixels — 4,555 of Multibrot 3's, 13,078 of
            // 4's — nearly all steep, and by the CPU's f64 orbit EQUALLY right: at the pixels
            // smooth by the CPU's own neighbours, SA on was wrong at 182 / 81 / 242 / 31 / 0 / 3
            // (Multibrot 3–8), SA off at 190 / 76 / 226 / 32 / 0 / 3. A 4,000-step boundary view is
            // chaotic, and a seed within 2^EPS of the stepped δ is one rounding more. So the
            // boundary row asks the question that has an answer: is SA any worse than no SA,
            // against an independent truth, where that truth is sure of itself.
            let mut sa_cases: Vec<(FractalKind, String, String, bool)> = vec![
                (FractalKind::Multibrot3, "0.2".into(), "0.1".into(), false),
                (FractalKind::Multibrot4, "0.2".into(), "0.1".into(), false),
                (FractalKind::Multibrot5, "0.2".into(), "0.1".into(), false),
            ];
            for kind in [
                FractalKind::Multibrot3,
                FractalKind::Multibrot4,
                FractalKind::Multibrot5,
                FractalKind::Multibrot6,
                FractalKind::Multibrot7,
                FractalKind::Multibrot8,
            ] {
                match family_view(kind.formula_id(), 1.0e7, N, self.render_cfg.max_iter) {
                    Some(at) => sa_cases.push((kind, format!("{:.17}", at.0), format!("{:.17}", at.1), true)),
                    None => push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Series approximation",
                        name: format!("{} SA no worse than SA-off vs CPU @1e7× (boundary)", kind.name()),
                        params: String::new(),
                        result: "no ray from 0 reaches a boundary with smooth escaping pixels at 1e7×".into(),
                        threshold: "a view to test at",
                        pass: false,
                    }),
                }
            }
            for (fractal, cx, cy, boundary) in sa_cases {
                let name = if boundary {
                    format!("{} SA no worse than SA-off vs CPU @1e7× (boundary)", fractal.name())
                } else {
                    format!("{} SA engages + matches SA-off @1e7×", fractal.name())
                };
                self.fractal = fractal;
                let mut vp = Viewport::new(N as f64, N as f64);
                vp.center_x = fractadyne_core::parse_bf(&cx).unwrap();
                vp.center_y = fractadyne_core::parse_bf(&cy).unwrap();
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
                    (Some(a), Some(b)) if skip > 0 && mode == 0 && !boundary => {
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
                            name,
                            params: format!("mode {mode}, skip {skip} of {} iter, {esc} escaped", on.max_iter),
                            result: format!("{mism} mismatch, {}", if finite { "finite" } else { "NON-FINITE!" }),
                            threshold: "skip>0, mode 0, finite, 0 mismatch",
                            pass: finite && mism == 0,
                        });
                    }
                    (Some(a), Some(b)) if skip > 0 && mode == 0 => {
                        let finite = a.iter().step_by(4).all(|v| v.is_finite());
                        let esc = a.iter().step_by(4).filter(|v| **v >= 0.0).count();
                        let nn = N as usize;
                        let t = cpu_family_iter(&on, fractal.formula_id(), N);
                        // The truth is sure of a pixel whose CPU neighbours agree with it (same
                        // class, within 2 iterations: the BLA check's "steep", inverted).
                        let sure = |k: usize| -> bool {
                            let (i, j) = (k % nn, k / nn);
                            let g = t[k * 4];
                            [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)].iter().all(|&(di, dj)| {
                                let (ni, nj) = (i as isize + di, j as isize + dj);
                                if ni < 0 || nj < 0 || ni as usize >= nn || nj as usize >= nn {
                                    return true;
                                }
                                let gn = t[(nj as usize * nn + ni as usize) * 4];
                                (g < 0.0) == (gn < 0.0) && (g < 0.0 || (g - gn).abs() <= 2.0)
                            })
                        };
                        let wrong = |px: &[f32], k: usize| {
                            let (p, q) = (px[k * 4], t[k * 4]);
                            (p < 0.0) != (q < 0.0) || (p >= 0.0 && (p - q).abs() > 0.5)
                        };
                        let (mut n_sure, mut on_wrong, mut off_wrong) = (0u64, 0u64, 0u64);
                        for k in (0..nn * nn).filter(|&k| sure(k)) {
                            n_sure += 1;
                            on_wrong += u64::from(wrong(&a, k));
                            off_wrong += u64::from(wrong(&b, k));
                        }
                        // Two draws of the same sensitive set differ by tens (226 against 242);
                        // a seed that is wrong is wrong at thousands of pixels.
                        let bound = off_wrong + off_wrong / 4 + 10;
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "Series approximation",
                            name,
                            params: format!(
                                "mode {mode}, skip {skip} of {} iter, {esc} escaped, CPU sure of {n_sure} px",
                                on.max_iter
                            ),
                            result: format!(
                                "wrong vs CPU: SA on {on_wrong}, SA off {off_wrong}, {}",
                                if finite { "finite" } else { "NON-FINITE!" }
                            ),
                            threshold: "skip>0, mode 0, finite, escapes, CPU sure of ≥ 2,000 px, SA on ≤ 1.25 × SA off + 10",
                            pass: finite && esc > 0 && n_sure >= 2_000 && on_wrong <= bound,
                        });
                    }
                    _ => push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "Series approximation",
                        name,
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
            // Multibrot 3–8 (design/power-families.md phase 3, B6): the same question at each
            // power's deep chaotic point, whose pixels escape hundreds to thousands of steps apart —
            // level 0's A = d·Z^(d−1) and radius 2·eps·|Z|/(d−1), and the merge every power shares.
            // And every fold family (phase 4, B7) at a deep point whose pixels are mostly stable,
            // through its real 2×2 tree. A BLA that is no faster proves nothing, so the skips must
            // SHOW (`CTR_BLA_SKIP`). Glitch detection off in both renders: the fold families'
            // requests carry it for the export's corrector, and the question is plain BLA against
            // plain perturbation.
            let prev = self.fractal;
            self.render_cfg.max_iter = 30_000;
            for (kind, x, y) in MULTIBROT_DEEP.iter().chain(FOLD_DEEP.iter()).copied() {
                self.fractal = kind;
                let mut vp = Viewport::new(N as f64, N as f64);
                vp.center_x = fractadyne_core::parse_bf(x).unwrap();
                vp.center_y = fractadyne_core::parse_bf(y).unwrap();
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * 1.0e30));
                vp.precision = fractadyne_core::precision_for_magnification(1.0e30);
                let mut on = self.current_export_request_for(&vp, false);
                on.width = N;
                on.height = N;
                on.ss = 1;
                on.glitch_on = 0;
                let mut off = on.clone();
                off.bla_on = 0;
                let (bon, mode) = (on.bla_on, on.mode);
                let name = format!("{} BLA == non-BLA @1e30× (deep boundary)", kind.name());
                // A fold row's CHAOS FLOOR: the render without BLA, shifted a thousandth of a pixel.
                let fold = fractadyne_core::fold_shape(kind.formula_id()).is_some();
                let shifted = fold.then(|| {
                    let mut s = off.clone();
                    let [rh, ih, rl, il] = s.ref_offset.to_array();
                    let px = s.span_mantissa.x / N as f64;
                    s.ref_offset = fractadyne_gpu::RefOffset::from_df32(rh as f64 + rl as f64 + 1.0e-3 * px, ih as f64 + il as f64);
                    s
                });
                match (
                    fractadyne_gpu::render_iter(device, queue, &on)
                        .map_err(|e| eprintln!("[selftest] GPU ERROR (render_iter): {e}"))
                        .ok(),
                    st_render_iter(device, queue, &off),
                ) {
                    (Some(ra), Some(b)) => {
                        let a = &ra.pixels;
                        let skips = ra.counters[fractadyne_gpu::CTR_BLA_SKIP];
                        let count = |a: &[f32]| {
                            let (mut mism, mut esc) = (0u64, 0u64);
                            for j in 0..nn {
                                for i in 0..nn {
                                    if steep(&b, i, j) {
                                        continue;
                                    }
                                    let k = j * nn + i;
                                    let (va, vb) = (a[k * 4], b[k * 4]);
                                    match (va < 0.0, vb < 0.0) {
                                        (false, false) => {
                                            esc += 1;
                                            if (va - vb).abs() > 0.5 {
                                                mism += 1;
                                            }
                                        }
                                        (true, true) => {}
                                        _ => mism += 1,
                                    }
                                }
                            }
                            (mism, esc)
                        };
                        let (mism, esc) = count(a);
                        // ⚠A fold's pixel can be "smooth" (its neighbours within 2 iterations) and still
                        // turn on the DIRECTION of an error far below a pixel — the core traced one
                        // where BLA's state was 4e-12 off and a δc nudge moving it 7e-11 changed
                        // nothing. So a fold row is held to its CHAOS FLOOR, the mismatches of the
                        // same render shifted a thousandth of a pixel: measured 2026-10-01, BLA's
                        // 95 / 1 / 2 / 1 against the shift's 178 / 1 / 6 / 4 (Celtic, Burning Ship
                        // 3, 4, Celtic 5), and 0 wherever the shift gave 0.
                        let floor = shifted.as_ref().and_then(|s| st_render_iter(device, queue, s)).map(|p| count(&p).0);
                        let (ok, threshold) = match floor {
                            Some(fl) => (mism <= fl + fl / 4, "bla engaged and skipping, escapers>100, mismatch ≤ 1.25 × a 0.001-px shift's"),
                            None => (mism == 0, "bla engaged and skipping, escapers>100, 0 mismatch"),
                        };
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "BLA",
                            name,
                            params: format!("mode {mode}, bla_on {bon}, {skips} skips, {esc} smooth escapers"),
                            result: match floor {
                                Some(fl) => format!("{mism} mismatch (a 0.001-px shift: {fl})"),
                                None => format!("{mism} mismatch"),
                            },
                            threshold,
                            pass: bon == 1 && mode == 2 && skips > 0 && esc > 100 && ok,
                        });
                    }
                    _ => push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "BLA",
                        name,
                        params: format!("mode {mode}, bla_on {bon}"),
                        result: "render failed".into(),
                        threshold: "bla engaged and skipping, escapers>100, 0 mismatch",
                        pass: false,
                    }),
                }
            }
            // ⭐The folds have no series approximation, so a deep view's INTERIOR ground through every
            // iteration until the 2×2 tree (Burning Ship inside its main body at 1e30×, 30,000
            // iterations: 5.1 s of GPU without it, 4 ms with). Inside each fold's main body (c =
            // 0.1 − 0.05i: off both axes, and off Re z² = 0, where Celtic's fold radius would be 0),
            // every pixel interior in both renders, and the tree carrying them: ≥ 5 skips a pixel.
            for kind in FractalKind::ALL.into_iter().filter(|k| fractadyne_core::fold_shape(k.formula_id()).is_some()) {
                self.fractal = kind;
                let mut vp = Viewport::new(N as f64, N as f64);
                vp.center_x = fractadyne_core::parse_bf("0.1").unwrap();
                vp.center_y = fractadyne_core::parse_bf("-0.05").unwrap();
                vp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.0 / (N as f64 * 1.0e30));
                vp.precision = fractadyne_core::precision_for_magnification(1.0e30);
                let mut on = self.current_export_request_for(&vp, false);
                on.width = N;
                on.height = N;
                on.ss = 1;
                on.glitch_on = 0;
                let mut off = on.clone();
                off.bla_on = 0;
                let name = format!("{} BLA carries a deep interior @1e30×", kind.name());
                let threshold = "bla engaged, every pixel interior in both, ≥ 5 skips a pixel";
                match (
                    fractadyne_gpu::render_iter(device, queue, &on)
                        .map_err(|e| eprintln!("[selftest] GPU ERROR (render_iter): {e}"))
                        .ok(),
                    st_render_iter(device, queue, &off),
                ) {
                    (Some(ra), Some(b)) => {
                        let n = (nn * nn) as u64;
                        let int_a = ra.pixels.iter().step_by(4).filter(|v| **v < 0.0).count() as u64;
                        let int_b = b.iter().step_by(4).filter(|v| **v < 0.0).count() as u64;
                        let skips = ra.counters[fractadyne_gpu::CTR_BLA_SKIP];
                        push_check(&mut checks, &mut last_check_t, SelfCheck {
                            category: "BLA",
                            name,
                            params: format!("mode {}, bla_on {}, {skips} skips over {n} px", on.mode, on.bla_on),
                            result: format!("interior {int_a} with BLA, {int_b} without"),
                            threshold,
                            pass: on.bla_on == 1 && on.mode == 2 && int_a == n && int_b == n && skips >= 5 * n,
                        });
                    }
                    _ => push_check(&mut checks, &mut last_check_t, SelfCheck {
                        category: "BLA",
                        name,
                        params: format!("mode {}, bla_on {}", on.mode, on.bla_on),
                        result: "render failed".into(),
                        threshold,
                        pass: false,
                    }),
                }
            }
            self.fractal = prev;
            self.render_cfg.max_iter = 5000;
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
                // the budget bound applies to (the full ask is honoured across frames). ⭐The SA
                // seed covers [0, sa_skip) in ONE O(1)/pixel polynomial evaluation (the shader's
                // `start_iter==0` branch), so a first window [0, sa_skip+step) LOOPS only `step`
                // real iterations — the GPU cost is `e - max(s, sa_skip)`, not the nominal `e - s`.
                // Measuring the raw range would price the free skip prefix as work (beta.99).
                let disp_iter = pr
                    .chunk_range
                    .map(|[s, e]| e.saturating_sub(s.max(pr.sa_skip)).max(1))
                    .unwrap_or(ITER);
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
                        // Real work past the SA seed (see the FIRST-dispatch check): [0, sa_skip)
                        // is a free O(1)/pixel seed, so the pass costs `e - max(s, sa_skip)` loops.
                        let d = pr.chunk_range.map(|[s, e]| {
                            (pr.resolution[0] as u64)
                                * (pr.resolution[1] as u64)
                                * (pr.ss as u64).pow(2)
                                * (e.saturating_sub(s.max(pr.sa_skip)).max(1) as u64)
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

        // ---- curated Navigate-menu landmarks: each must BE what it claims ----
        // Nothing else gates the FAMOUS / MISIUREWICZ_POI coordinate STRINGS, so a mistyped digit
        // would ship as a menu jump to blank space with no test to catch it. Re-derive each landmark
        // from its stored centre: deep minibrot nuclei via `find_nucleus` (the period must match and
        // the solve must stay on the named atom), and the Misiurewicz points via `detect_misiurewicz`
        // (the (preperiod, period) written into the name). Cheap — a few shallow orbit walks plus one
        // 998-period nucleus solve — so it belongs in the default suite, not the opt-in tier.
        if want("curated-poi") {
            // (a) Deep minibrot nuclei carried in FAMOUS (only the entries with a claimed period).
            let mut bad = Vec::new();
            for &(idx, seed_l2, want_p) in crate::FAMOUS_DEEP_NUCLEI {
                let (name, cx, cy, _mag) = crate::FAMOUS[idx];
                let (Some(x), Some(y)) =
                    (fractadyne_core::parse_bf(cx), fractadyne_core::parse_bf(cy))
                else {
                    bad.push(format!("{name}: centre did not parse"));
                    continue;
                };
                match fractadyne_core::find_nucleus(&[x.clone(), y.clone()], seed_l2, 0, 100_000) {
                    Some(n) if n.period == want_p => {
                        // Landed on the atom we named, not some unrelated far component: the refined
                        // nucleus must sit within the atom's own width of the stored centre.
                        let moved = (fractadyne_core::sub_f64(&n.cx, &x, 256).powi(2)
                            + fractadyne_core::sub_f64(&n.cy, &y, 256).powi(2))
                        .sqrt();
                        if moved > 2f64.powf(-seed_l2 + 2.0) {
                            bad.push(format!("{name}: p{} but moved {moved:.1e}", n.period));
                        }
                    }
                    Some(n) => bad.push(format!("{name}: period {} (want {want_p})", n.period)),
                    None => bad.push(format!("{name}: no nucleus")),
                }
            }
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Curated",
                name: "deep minibrot menu entries re-solve to their period".into(),
                params: format!("{} nucleus entries", crate::FAMOUS_DEEP_NUCLEI.len()),
                result: if bad.is_empty() { "all exact".into() } else { bad.join("; ") },
                threshold: "period matches, solve within one atom width",
                pass: bad.is_empty(),
            });

            // (b) Misiurewicz points of interest — the (k,p) is written into each name as "(k,p)".
            let mut mbad = Vec::new();
            for &(name, cx, cy, mag) in crate::MISIUREWICZ_POI {
                let want_kp: Option<(u32, u32)> = (|| {
                    let inside = name.rsplit_once('(')?.1.split_once(')')?.0;
                    let (a, b) = inside.split_once(',')?;
                    Some((a.trim().parse::<u32>().ok()?, b.trim().parse::<u32>().ok()?))
                })();
                let Some((wk, wp)) = want_kp else {
                    mbad.push(format!("{name}: name carries no (k,p)"));
                    continue;
                };
                let (Some(x), Some(y)) =
                    (fractadyne_core::parse_bf(cx), fractadyne_core::parse_bf(cy))
                else {
                    mbad.push(format!("{name}: centre did not parse"));
                    continue;
                };
                // Select the pair whose feature is the size of the framed view (~4/mag tall, the
                // REFERENCE_HEIGHT convention), so a point can't be mistaken for a coarser feature
                // it sits inside.
                let span_l2 = (4.0_f64 / mag).log2();
                match fractadyne_core::detect_misiurewicz_at_scale(
                    &x,
                    &y,
                    0,
                    2_000,
                    32,
                    256,
                    Some(span_l2),
                ) {
                    Some((k, p)) if k == wk && p == wp => {}
                    Some((k, p)) => mbad.push(format!("{name}: got ({k},{p})")),
                    None => mbad.push(format!("{name}: not detected")),
                }
            }
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Curated",
                name: "Misiurewicz menu entries re-derive their (k,p)".into(),
                params: format!("{} points", crate::MISIUREWICZ_POI.len()),
                result: if mbad.is_empty() { "all match".into() } else { mbad.join("; ") },
                threshold: "detected (preperiod,period) == name",
                pass: mbad.is_empty(),
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

            // The INTERACTIVE lookahead's queue length follows the zoom-rate slider: enough
            // `PREFETCH_OCT` slots to cover `PREFETCH_RUNWAY_S` of zoom, floored at the tour's
            // `PREFETCH_SLOTS`, capped at `PREFETCH_SLOTS_MAX` — a 4× dive must not run its queue
            // dry in a second, and a 0.25× one must not hold a dozen orbits resident for nothing.
            let oct = |zr: f64| crate::ZOOM_RATE * zr / std::f64::consts::LN_2;
            let (s1, s2, s4) = (
                crate::render::prefetch_slots_for(oct(1.0)),
                crate::render::prefetch_slots_for(oct(2.0)),
                crate::render::prefetch_slots_for(oct(4.0)),
            );
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "Script",
                name: "interactive lookahead queue scales with zoom rate".into(),
                params: format!("zoom rate 1×/2×/4× · runway {}s", crate::PREFETCH_RUNWAY_S),
                result: format!("{s1}/{s2}/{s4} slots"),
                threshold: "6 / 8 / 12",
                pass: s1 == 6 && s2 == 8 && s4 == 12,
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

            // A Custom view carries its formula — a multi-line source with a comment, and its
            // parameters — through the real writer and reader; and a view whose formula no longer
            // compiles says so and is NOT shown under whatever formula happened to be loaded.
            {
                let src = "t = sqr(z)\nz = t + p1*conj(t) + c ; hybrid";
                let made = crate::custom_formula::CustomFormula::compile(src, &[(0.25, -0.125)]).expect("compiles");
                let key = made.shader.key;
                self.custom = Some(std::sync::Arc::new(made));
                self.fractal = FractalKind::Custom;
                let blob = self.view_metadata();
                // Scramble: another formula, another family.
                self.custom =
                    Some(std::sync::Arc::new(crate::custom_formula::CustomFormula::compile("z^3 + c", &[]).expect("compiles")));
                self.fractal = FractalKind::Mandelbrot;
                let rt = self.load_view_metadata(&blob);
                let back = self.custom.as_ref().map(|c| (c.source.clone(), c.params[0], c.shader.key));
                let round_trip = rt.note().is_none()
                    && self.fractal == FractalKind::Custom
                    && back == Some((src.to_string(), (0.25, -0.125), key));
                let broken = blob
                    .lines()
                    .map(|l| if l.starts_with("formula=") { "formula=z = z^2 +" } else { l })
                    .collect::<Vec<_>>()
                    .join("\n");
                self.fractal = FractalKind::Mandelbrot;
                let rb = self.load_view_metadata(&broken);
                let refused = self.fractal == FractalKind::Mandelbrot
                    && rb.problems.iter().any(|p| p.contains("does not compile"));
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "View format",
                    name: "a_custom_view_round_trips_its_formula".into(),
                    params: "Custom view → scramble → load; then a broken formula line".into(),
                    result: format!("round trip {round_trip}, broken one refused {refused}"),
                    threshold: "formula, parameters and shader key restored; broken formula reported, view not switched",
                    pass: round_trip && refused,
                });
                self.fractal = FractalKind::Mandelbrot;
            }

            // A Life view carries its universe: the rule, every cell (Generations states and negative
            // coordinates included) at the generation it holds, and the generation to run on to.
            {
                let pattern = fractadyne_core::life::parse_rle("#CXRLE Pos=-70,-3\nA2.BC$3.A$CBA!").expect("parses");
                let opened = self.life_open_view(pattern, "B2/S345/C4", "round trip".into(), 37, 37, true);
                let want = self.life.loaded.cells();
                let blob = self.view_metadata();
                let format3 = blob.lines().any(|l| l == "format_version=3");
                // Scramble: another pattern and rule, another family.
                self.life_open_library("Glider");
                self.fractal = FractalKind::Mandelbrot;
                let rt = self.load_view_metadata(&blob);
                let round_trip = rt.note().is_none()
                    && self.fractal == FractalKind::Life
                    && self.life.loaded.cells() == want
                    && self.life.loaded.generation() == 37
                    && self.life.loaded.rule().canonical() == "B2/S345/C4"
                    && self.life.pattern_name == "round trip";
                // A view asking for a later generation runs on to it.
                let later = blob.lines().map(|l| if l.starts_with("generation=") { "generation=500" } else { l }).collect::<Vec<_>>().join("\n");
                let rl = self.load_view_metadata(&later);
                let runs_on = rl.note().is_none() && self.life.target == 500 && self.life.loaded.generation() == 37;
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "View format",
                    name: "a_life_view_round_trips_its_universe".into(),
                    params: "Life view (Star Wars, 3 states, x < 0) → scramble → load; then generation=500".into(),
                    result: format!("opened {}, format 3 {format3}, round trip {round_trip}, runs on {runs_on}", opened.is_ok()),
                    threshold: "rule, cells, generation and name restored; format 3; target generation 500",
                    pass: opened.is_ok() && format3 && round_trip && runs_on,
                });
                // The session keeps the universe whichever family is on screen, and restoring it
                // does not switch to it.
                let saved = self.life_lines();
                self.life_open_library("Glider");
                self.fractal = FractalKind::Mandelbrot;
                let restored = self.apply_life_lines(&saved, false);
                let kept = restored.is_ok()
                    && self.fractal == FractalKind::Mandelbrot
                    && self.life.loaded.cells() == want
                    && self.life.loaded.rule().canonical() == "B2/S345/C4";
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "View format",
                    name: "the session keeps the Life universe".into(),
                    params: "life_lines → scramble → apply (not shown)".into(),
                    result: format!("restored {restored:?}, cells/rule kept and view not switched: {kept}"),
                    threshold: "cells and rule restored; family unchanged",
                    pass: kept,
                });
                self.fractal = FractalKind::Mandelbrot;
            }

            // An L-system view carries its system (text with every command, escaped onto one
            // line), the fixed order, the angle override, the line width and the colouring.
            {
                let text = "name Round trip\nangle /7\nheading 12.5\ndraw G\naxiom F[+G]@IQ2\\30C3\nF = F-G<2+F\nG = GG\n";
                let opened = self.lsystem_open_text(text, "unnamed");
                self.lsystem.fixed_order = Some(5);
                self.lsystem.set_angle(Some(61.5));
                self.lsystem.width = 2.5;
                self.lsystem.colour = Some(fractadyne_core::lsystem::Colouring::Heading);
                let want = self.lsystem.system.clone();
                let blob = self.view_metadata();
                let format4 = blob.lines().any(|l| l == "format_version=4");
                let one_line = blob.lines().filter(|l| l.starts_with("lsystem=")).count() == 1;
                // Scramble: another system and settings, another family.
                self.lsystem_open_library("Hilbert curve");
                self.lsystem.width = 1.0;
                self.fractal = FractalKind::Mandelbrot;
                let rt = self.load_view_metadata(&blob);
                let l = &self.lsystem;
                let round_trip = rt.note().is_none()
                    && self.fractal == FractalKind::LSystem
                    && l.system == want
                    && l.fixed_order == Some(5)
                    && l.angle == Some(61.5)
                    && l.width == 2.5
                    && l.colour == Some(fractadyne_core::lsystem::Colouring::Heading);
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "View format",
                    name: "an_lsystem_view_round_trips_its_system".into(),
                    params: "L-system view (every command, order 5, angle 61.5°, width 2.5, heading colours) → scramble → load".into(),
                    result: format!("opened {}, format 4 {format4}, one line {one_line}, round trip {round_trip}", opened.is_ok()),
                    threshold: "system, order, angle, width, colouring restored; format 4",
                    pass: opened.is_ok() && format4 && one_line && round_trip,
                });
                let saved = self.lsystem_lines();
                self.lsystem_open_library("Koch curve");
                self.fractal = FractalKind::Mandelbrot;
                let restored = self.apply_lsystem_lines(&saved, false);
                let kept = restored.is_ok() && self.fractal == FractalKind::Mandelbrot && self.lsystem.system == want;
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "View format",
                    name: "the session keeps the L-system".into(),
                    params: "lsystem_lines → scramble → apply (not shown)".into(),
                    result: format!("restored {restored:?}, system kept and view not switched: {kept}"),
                    threshold: "system restored; family unchanged",
                    pass: kept,
                });
                self.lsystem = Default::default();
                self.fractal = FractalKind::Mandelbrot;
            }

            // ⛔A location imported from another renderer keeps the iteration count it asks for. The
            // .kfr importer clamped it to 50,000 while the Imagina one did not, so the corpus's
            // 1.2e148 .kfr, which asks 800,000, rendered every pixel at the cap (near-black) — and
            // the release check that compares the standard and accelerated builds on it compared two
            // black images. Both importers, through their real file readers: a count above the old
            // cap is kept, and one below the floor of 64 is raised to it. 5,000,000 also passes the
            // parsers' old 1,000,000 cap, under the app's own 10,000,000.
            {
                let dir = std::env::temp_dir().join(format!("fd-selftest-imports-{}", std::process::id()));
                let _ = std::fs::create_dir_all(&dir);
                let (kfr, imagina) = (dir.join("asked.kfr"), dir.join("asked.txt"));
                let mut seen = Vec::new();
                let mut pass = true;
                for (asked, want) in [(800_000u32, 800_000u32), (5_000_000, 5_000_000), (20, 64)] {
                    let wrote = std::fs::write(&kfr, format!("Re: -0.75\r\nIm: 0.1\r\nZoom: 1E30\r\nIterations: {asked}\r\n")).is_ok()
                        && std::fs::write(&imagina, format!("Location:\n\tSize: 2e-30\n\tRe: -0.75\n\tIm: 0.1\n\tIterations: {asked}\n")).is_ok();
                    for (name, path) in [(".kfr", &kfr), ("Imagina", &imagina)] {
                        // Scramble first, so a load that leaves the count alone cannot pass.
                        self.render_cfg.max_iter = 1234;
                        self.render_cfg.auto_iter = true;
                        let loaded = if name == ".kfr" { self.load_kfr_file(path) } else { self.load_imagina_file(path) };
                        let ok = wrote && loaded.is_ok() && self.render_cfg.max_iter == want && !self.render_cfg.auto_iter;
                        pass &= ok;
                        seen.push(format!("{name} {}→{}", crate::grouped_count(f64::from(asked)), if loaded.is_ok() { crate::grouped_count(f64::from(self.render_cfg.max_iter)) } else { "refused".into() }));
                    }
                }
                let _ = std::fs::remove_dir_all(&dir);
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "View format",
                    name: "an imported location keeps its iteration count".into(),
                    params: ".kfr and Imagina text files asking 800,000, 5,000,000 and 20 iterations".into(),
                    result: seen.join(", "),
                    threshold: "800,000 and 5,000,000 kept and 20 raised to 64 by both importers; automatic iterations off",
                    pass,
                });
                self.fractal = FractalKind::Mandelbrot;
            }

            // ⭐A custom view travels in EVERY format that carries a view, with a comment past
            // Latin-1 in its formula: that made the PNG export FAIL and the EXR export PANIC (both
            // containers are Latin-1), and "Tour from current view" wrote a tour the parser refused.
            // The view text must be ASCII (the formula escaped), come back verbatim from the real
            // PNG and EXR writers, and the generated tour must show the same formula; a built-in
            // view still writes format 1, so older builds read it as they always did.
            {
                let src = "t = sqr(z)\nz = t + p1*conj(t) + c ; √ hybrid, café 😀";
                let params = (0.25, -0.125);
                let made = crate::custom_formula::CustomFormula::compile(src, &[params]).expect("compiles");
                self.custom = Some(std::sync::Arc::new(made));
                self.fractal = FractalKind::Custom;
                let blob = self.view_metadata();
                // The formula lines (the notes field may hold Latin-1, which both containers take).
                let ascii = blob
                    .lines()
                    .filter(|l| l.starts_with("formula"))
                    .all(|l| l.chars().all(|c| c.is_ascii() && !c.is_ascii_control()));
                let format2 = blob.lines().any(|l| l == "format_version=2");
                let restores = |app: &mut Self, text: Option<String>| -> bool {
                    let Some(text) = text else { return false };
                    app.custom = None;
                    app.fractal = FractalKind::Mandelbrot;
                    let r = app.load_view_metadata(&text);
                    r.note().is_none()
                        && app.fractal == FractalKind::Custom
                        && app.custom.as_ref().is_some_and(|c| c.source == src && c.params[0] == params)
                };
                let dir = std::env::temp_dir().join(format!("fd-selftest-formats-{}", std::process::id()));
                let _ = std::fs::create_dir_all(&dir);
                let (png, exr) = (dir.join("custom.png"), dir.join("custom.exr"));
                let pixels = vec![0.5f32; 4 * 4 * 4];
                let png_ok = fractadyne_export::write_png(&png, 4, 4, &pixels, Some(&blob)).is_ok()
                    && restores(self, fractadyne_export::read_png_metadata(&png).ok().flatten());
                let exr_ok = fractadyne_export::write_exr(&exr, 4, 4, &pixels, Some(&blob)).is_ok()
                    && restores(self, fractadyne_export::read_exr_metadata(&exr).ok().flatten());
                let _ = std::fs::remove_dir_all(&dir);
                // The tour: "Tour from current view", parsed back by the tour reader.
                let tour_ok = restores(self, Some(blob.clone())) && {
                    let text = self.build_dive_script("", 5.0);
                    match crate::scripting::parse_tour_text(&text) {
                        Ok(pb) => {
                            let s = pb.sample(0.0);
                            s.fractal == FractalKind::Custom
                                && s.custom.as_ref().is_some_and(|c| c.source == src && c.params[0] == params)
                        }
                        Err(e) => {
                            eprintln!("[selftest] the generated tour does not parse: {e}");
                            false
                        }
                    }
                };
                self.fractal = FractalKind::Mandelbrot;
                let plain1 = self.view_metadata().lines().any(|l| l == "format_version=1");
                push_check(&mut checks, &mut last_check_t, SelfCheck {
                    category: "View format",
                    name: "a_custom_view_travels_in_every_format".into(),
                    params: "formula with a √ / é / emoji comment: view text, PNG, EXR, tour".into(),
                    result: format!(
                        "ASCII {ascii}, format 2 {format2}; PNG {png_ok}, EXR {exr_ok}, tour {tour_ok}; built-in view format 1 {plain1}"
                    ),
                    threshold: "all true",
                    pass: ascii && format2 && png_ok && exr_ok && tour_ok && plain1,
                });
            }

            // ⭐A coordinate ENTERED AS AN EXPRESSION travels with the view and is re-derived on
            // load, so a reopened file can be zoomed deeper than it was saved without the centre
            // freezing at the digits a plain decimal would carry. Round-tripped through the real
            // writer and reader; the centre must come back as `1/3` to far more than the ~15 digits
            // a shallow decimal holds.
            self.center_expr = crate::CenterExpr::capture(
                "1/3", "0",
                fractadyne_core::parse_bf_prec("1/3", 300).unwrap(),
                fractadyne_core::parse_bf_prec("0", 300).unwrap(),
                300,
            );
            self.viewport.center_x = fractadyne_core::parse_bf_prec("1/3", 300).unwrap();
            self.viewport.center_y = fractadyne_core::parse_bf_prec("0", 300).unwrap();
            self.viewport.units_per_pixel = fractadyne_core::FloatExp::from_f64(1.0).mul_pow2(-200.0);
            self.viewport.precision = fractadyne_core::precision_for_octaves(200);
            let xblob = self.view_metadata();
            let has_expr_key = xblob.contains("center_re_expr=1/3");
            self.viewport.center_x = fractadyne_core::parse_bf("0.5").unwrap();
            self.center_expr = None;
            let xr = self.load_view_metadata(&xblob);
            let third = fractadyne_core::parse_bf_prec("1/3", 400).unwrap();
            let x_err = fractadyne_core::to_f64(&fractadyne_core::bf_sub(
                &self.viewport.center_x, &third, 400,
            )).abs();
            let expr_rt_ok =
                xr.note().is_none() && has_expr_key && self.center_expr.is_some() && x_err < 1.0e-30;
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "View format",
                name: "coordinate expression round-trips".into(),
                params: "center_re_expr=1/3 → save → scramble → load".into(),
                result: format!("has key {has_expr_key}, kept {}, |c−1/3| {x_err:.2e}", self.center_expr.is_some()),
                threshold: "key written; expression kept; centre re-derived (|c−1/3| < 1e-30)",
                pass: expr_rt_ok,
            });

            // ⭐The reader must re-derive the expression at the VIEW'S precision, not trust a short
            // decimal beside it: a deep file whose `center_re` holds only 10 digits but carries the
            // exact expression must reconstruct the centre to the depth, and a saved offset must be
            // re-applied on top of the re-derived anchor.
            let deep_expr = "app=Fractadyne\nformat_version=1\ncenter_re=0.3333333333\ncenter_im=0\n\
                             center_re_expr=1/3\ncenter_im_expr=0\nupp_log2=-400\n";
            let _ = self.load_view_metadata(deep_expr);
            let third_deep = fractadyne_core::parse_bf_prec("1/3", 600).unwrap();
            let deep_err = fractadyne_core::to_f64(&fractadyne_core::bf_sub(
                &self.viewport.center_x, &third_deep, 600,
            )).abs();
            let off_blob = "app=Fractadyne\nformat_version=1\ncenter_re=0\ncenter_im=0\n\
                            center_re_expr=1/3\ncenter_im_expr=0\ncenter_re_offset=1e-40\n\
                            center_im_offset=0\nupp_log2=-200\n";
            let _ = self.load_view_metadata(off_blob);
            let want = fractadyne_core::bf_add(
                &fractadyne_core::parse_bf_prec("1/3", 400).unwrap(),
                &fractadyne_core::parse_bf_prec("1e-40", 400).unwrap(),
                400,
            );
            let off_err = fractadyne_core::to_f64(&fractadyne_core::bf_sub(
                &self.viewport.center_x, &want, 400,
            )).abs();
            // A shallow decimal centre sits ~1e-10 from 1/3, and a dropped offset ~1e-40 away — both
            // far above these bars, so a regression to either shows up here.
            let deep_ok = deep_err < 1.0e-40 && off_err < 1.0e-60;
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "View format",
                name: "expression re-derived at view precision + offset".into(),
                params: "deep view, 10-digit center_re, exact expr; anchor + 1e-40 offset".into(),
                result: format!("|c−1/3| {deep_err:.2e}; |c−(1/3+1e-40)| {off_err:.2e}"),
                threshold: "re-derived deep (< 1e-40) and offset applied (< 1e-60)",
                pass: deep_ok,
            });
            // ⭐The LIVE hook `update()` runs every frame: while the centre still sits on the
            // expression's point, a deeper zoom re-derives it to the precision that depth needs;
            // once panned off, it neither re-derives nor drops (the offset is materialised at save
            // time). Drive `refresh_center_expr` directly — selftest has no frame loop.
            self.viewport.center_x = fractadyne_core::parse_bf_prec("1/3", 100).unwrap();
            self.viewport.center_y = fractadyne_core::parse_bf_prec("0", 100).unwrap();
            self.viewport.units_per_pixel = fractadyne_core::FloatExp::from_f64(1.0).mul_pow2(-400.0);
            self.viewport.precision = fractadyne_core::precision_for_octaves(400);
            self.center_expr = crate::CenterExpr::capture(
                "1/3", "0",
                self.viewport.center_x.clone(), self.viewport.center_y.clone(), 100,
            );
            self.refresh_center_expr(); // on-point, view now far deeper than 100 bits → re-derives
            let live_third = fractadyne_core::parse_bf_prec("1/3", 700).unwrap();
            let live_err = fractadyne_core::to_f64(&fractadyne_core::bf_sub(
                &self.viewport.center_x, &live_third, 700,
            )).abs();
            let grew = self.center_expr.as_ref().map(|c| c.prec).unwrap_or(0) > 400;
            // Pan off the point: the hook must NOT re-derive (centre stays where the pan left it) and
            // must NOT drop the anchor (kept so the offset can be written at save time).
            self.viewport.center_x = fractadyne_core::bf_add(
                &self.viewport.center_x,
                &fractadyne_core::parse_bf_prec("0.01", 700).unwrap(),
                700,
            );
            let moved = self.viewport.center_x.clone();
            self.refresh_center_expr();
            let held = self.center_expr.is_some()
                && fractadyne_core::bf_sub(&self.viewport.center_x, &moved, 700).is_zero();
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "View format",
                name: "live zoom re-derives on-point, holds off-point".into(),
                params: "on-point deep zoom, then pan; refresh_center_expr()".into(),
                result: format!("grew {grew}, |c−1/3| {live_err:.2e}, held-off-point {held}"),
                threshold: "re-derived deep on-point (< 1e-40); unchanged + kept off-point",
                pass: grew && live_err < 1.0e-40 && held,
            });

            // ⭐The exact-points menu (`EXPRESSION_POI`): the main-cardioid bulb roots, given as
            // transcendental expressions. Verify each evaluates to a point genuinely ON the cardioid
            // boundary — where the fixed point z=(1−√(1−4c))/2 has multiplier |2z|=1 — and that each
            // is kept as an expression (never a bare decimal). A typo in a formula moves the point
            // off the boundary and fails here rather than shipping a menu item that lands on nothing.
            let z0 = fractadyne_core::parse_bf("0").unwrap();
            let mut worst_mult = 0.0_f64;
            let mut poi_all_expr = true;
            for (_, re, im, _) in crate::EXPRESSION_POI {
                let cr = fractadyne_core::to_f64(&fractadyne_core::parse_bf_prec(re, 256).unwrap());
                let ci = fractadyne_core::to_f64(&fractadyne_core::parse_bf_prec(im, 256).unwrap());
                // w = 1 − 4c; principal complex √; multiplier μ = 2z = 1 − √w, and |μ| = 1 on the
                // cardioid boundary.
                let (wa, wb) = (1.0 - 4.0 * cr, -4.0 * ci);
                let r = wa.hypot(wb);
                let sr = ((r + wa) * 0.5).max(0.0).sqrt();
                let si = wb.signum() * ((r - wa) * 0.5).max(0.0).sqrt();
                worst_mult = worst_mult.max(((1.0 - sr).hypot(si) - 1.0).abs());
                poi_all_expr &=
                    crate::CenterExpr::capture(re, im, z0.clone(), z0.clone(), 64).is_some();
            }
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "View format",
                name: "exact bulb-root expressions land on the cardioid".into(),
                params: format!("{} EXPRESSION_POI entries; |2z| at each", crate::EXPRESSION_POI.len()),
                result: format!("worst ||μ|−1| {worst_mult:.2e}; all kept as expressions {poi_all_expr}"),
                threshold: "on the boundary (< 1e-9) and every point preserved as an expression",
                pass: worst_mult < 1.0e-9 && poi_all_expr,
            });
            self.center_expr = None;

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
                // ⭐The report now names the LINE too ("bogus_field (line 10)"), so match the
                // key rather than the whole rendered string.
                && hr.unknown.iter().any(|u| u.starts_with("bogus_field"));
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

            // The custom gradient travels WITH the view. Before this field, a `.fdn` saved on a
            // hand-built gradient carried only `palette=<index>`, so reopening it landed on
            // whichever preset sat at that index: the geometry survived and the colour did not.
            // Round-tripped through the real writer and reader, not just the codec.
            let palette_was = (
                self.coloring.custom_segments.clone(),
                self.coloring.custom_palette.clone(),
                self.coloring.use_custom_palette,
                self.coloring.custom_palette_flat,
            );
            let rich = vec![
                fractadyne_state::PaletteSegment {
                    left: 0.0, mid: 0.113_712_3, right: 0.25,
                    left_color: [0.0, 0.0, 0.0, 1.0],
                    right_color: [0.937_254_9, 0.203_921_6, 0.101_960_8, 1.0],
                    blend: 1, space: 1, blend_params: [0.0; 4],
                },
                fractadyne_state::PaletteSegment {
                    left: 0.25, mid: 0.9, right: 1.0,
                    left_color: [0.937_254_9, 0.203_921_6, 0.101_960_8, 1.0],
                    right_color: [0.043_137_3, 0.180_392_2, 0.941_176_4, 1.0],
                    blend: 5, space: 0, blend_params: [0.17, 0.67, 0.83, 0.33],
                },
            ];
            self.coloring.custom_segments = rich.clone();
            self.coloring.use_custom_palette = true;
            self.coloring.custom_palette_flat = false;
            let cblob = self.view_metadata();
            // Scramble: back to a preset, gradient forgotten — exactly the state a fresh launch
            // would be in when the file is opened.
            self.coloring.custom_segments.clear();
            self.coloring.use_custom_palette = false;
            let cr = self.load_view_metadata(&cblob);
            let back = self.coloring.custom_segments.clone();
            // ⭐Name the first field that differs. A bare "did not round-trip" sends the next
            // person back to a debugger for something the check already knows.
            let mut diff: Option<String> = None;
            for (i, (a, b)) in back.iter().zip(rich.iter()).enumerate() {
                for (f, ok) in [
                    ("left", a.left == b.left),
                    ("mid", a.mid == b.mid),
                    ("right", a.right == b.right),
                    ("left_color", a.left_color == b.left_color),
                    ("right_color", a.right_color == b.right_color),
                    ("blend", a.blend == b.blend),
                    ("space", a.space == b.space),
                    ("blend_params", b.blend != 5 || a.blend_params == b.blend_params),
                ] {
                    if !ok && diff.is_none() {
                        diff = Some(format!("seg {i} {f}: {a:?} vs {b:?}"));
                    }
                }
            }
            let same = back.len() == rich.len()
                && back.iter().zip(rich.iter()).all(|(a, b)| {
                    a.left == b.left && a.mid == b.mid && a.right == b.right
                        && a.left_color == b.left_color && a.right_color == b.right_color
                        && a.blend == b.blend && a.space == b.space
                        // ⚠`blend_params` is meaningful ONLY for kind 5. Every other kind
                        // normalizes it to `BEZIER_IDENTITY` on the way through `Blend`, so
                        // demanding it round-trip on a Curved segment would be asserting that a
                        // field nothing reads keeps a value nothing wrote.
                        && (b.blend != 5 || a.blend_params == b.blend_params)
                });
            let embed_ok = cr.note().is_none() && same && self.coloring.use_custom_palette
                && !self.coloring.custom_palette.is_empty();
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "View format",
                name: "custom gradient round-trips in the view".into(),
                params: "2 segments: off-centre mid, Bézier blend, HSV space, alpha".into(),
                result: match diff.clone().or_else(|| cr.note().map(|n| format!("note: {n}"))) {
                    Some(d) => format!("MISMATCH {d}"),
                    None => format!(
                        "{} segments back, custom {}, {} derived stops",
                        back.len(),
                        self.coloring.use_custom_palette,
                        self.coloring.custom_palette.len()
                    ),
                },
                threshold: "every field bit-identical; custom palette re-selected",
                pass: embed_ok,
            });

            // ⛔⭐⭐**Every key the WRITER emits must be one the READER knows.** `palette_custom`
            // was added to the writer and its own load branch, and both worked — but the key was
            // missing from `KNOWN_VIEW_KEYS`, so every load reported "ignored unknown field(s):
            // palette_custom" about a field it had just honoured. Nothing else could see that: the
            // gradient arrived correctly and the warning was cosmetic-looking noise. This check
            // exists so the NEXT field added to the writer cannot repeat it.
            let unknown_emitted: Vec<String> = cblob
                .lines()
                .filter_map(|l| l.split_once('='))
                .map(|(k, _)| k.to_string())
                .filter(|k| !crate::export::KNOWN_VIEW_KEYS.contains(&k.as_str()))
                .collect();
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "View format",
                name: "every emitted key is a known key".into(),
                params: format!("{} keys written", cblob.lines().count()),
                result: if unknown_emitted.is_empty() {
                    "all known".into()
                } else {
                    format!("writer emits unknown: {}", unknown_emitted.join(", "))
                },
                threshold: "writer ⊆ KNOWN_VIEW_KEYS",
                pass: unknown_emitted.is_empty(),
            });

            // ⚠⚠And the field must be ABSENT on a preset view. A `palette_custom=` written for
            // every export would bloat every PNG's metadata and, worse, would pin a stale gradient
            // onto files whose author was using a preset.
            self.coloring.use_custom_palette = false;
            let preset_blob = self.view_metadata();
            let absent = !preset_blob.contains("palette_custom");
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "View format",
                name: "preset views carry no embedded gradient".into(),
                params: "use_custom_palette = false".into(),
                result: if absent { "absent".into() } else { "PRESENT".to_string() },
                threshold: "no palette_custom key",
                pass: absent,
            });

            // A corrupt gradient must leave the live palette ALONE and say so — not render a
            // colour nobody chose. The shape here parses as numbers but does not cover 0..1.
            self.coloring.custom_segments = rich.clone();
            self.coloring.use_custom_palette = true;
            let bad = "app=Fractadyne\nformat_version=1\ncenter_re=-0.5\ncenter_im=0\n\
                       palette_custom=0.3,0.5,0.9,0,0,0,1,1,1,1,1,0,0,0,0,0,0\n";
            let br = self.load_view_metadata(bad);
            let kept = self.coloring.custom_segments == rich
                && br.clamped.iter().any(|c| *c == "custom palette");
            push_check(&mut checks, &mut last_check_t, SelfCheck {
                category: "View format",
                name: "a corrupt gradient is refused, not applied".into(),
                params: "palette_custom spanning 0.3..0.9".into(),
                result: format!(
                    "{} segments kept; clamped [{}]",
                    self.coloring.custom_segments.len(), br.clamped.join(", ")
                ),
                threshold: "live gradient untouched + reported clamped",
                pass: kept,
            });

            // ⚠⚠Put the palette back. These checks install a gradient to prove it travels, and a
            // later check ("gradient-edit-changes-the-image") renders a preset and then edits ONE
            // stop — with our gradient still live it saw no change and failed, which is a leak in
            // this block, not a defect in that one.
            self.coloring.custom_segments = palette_was.0;
            self.coloring.custom_palette = palette_was.1;
            self.coloring.use_custom_palette = palette_was.2;
            self.coloring.custom_palette_flat = palette_was.3;
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
                app.render_export_normalized(dev, q, &app.viewport, false, cw, ch, 1, crate::render::NormRange::OwnFrame, None, None, u64::MAX)
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
        let args = crate::effective_args();
        let report_path = args
            .iter()
            .position(|a| a == "--out" || a == "-o")
            .and_then(|i| args.get(i + 1))
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
            // The power families (design/power-families.md), one overview per shape: every power is
            // the same generic code with its own (shape, d), and each is held to its generated module
            // and the CPU by the custom-formula and abs-family groups, so a golden apiece would add
            // ~33 MB of history for no case those checks do not already make.
            ("multibrot7", FractalKind::Multibrot7, "0.0", "0.0", 1.2, 800, 0, 0, false),
            ("burning-ship3", FractalKind::BurningShip3, "0.0", "0.0", 1.2, 800, 0, 0, false),
            ("tricorn4", FractalKind::Tricorn4, "0.0", "0.0", 1.2, 800, 0, 0, false),
            ("celtic5", FractalKind::Celtic5, "0.0", "0.0", 1.2, 800, 0, 0, false),
            ("buffalo3", FractalKind::Buffalo3, "0.0", "0.0", 1.2, 800, 0, 0, false),
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
            // ⭐The first golden PAST 1e6× — the period-998 Seahorse minibrot at its own atom size
            // (~1.6e15×), the new "Seahorse minibrot ·998" Navigate-menu destination. Every other
            // deep golden stops at 1e6×, so nothing gated the perturbation pipeline at the depth the
            // whole app exists for. Safe as a golden because this is the `render_export` path — one
            // arithmetic backend, byte-identical (why the F3 corpus holds maxD 0), df32 perturbation
            // (mode 0), well below the ~1e300× floatexp tier whose LIVE rendering is hardware-varying.
            // 25k iter: enough that the minibrot's exterior escapes and the body resolves rather than
            // flooding black (the adaptive appetite here is ~12k — see the black-minibrot arc).
            ("seahorse-998", FractalKind::Mandelbrot, "-0.7436438870371588707780645434936425750476099623212550602141", "0.1318259042053122928210973548747672652629885996790429749374", 1.597e15, 25000, 0, 1, false),
        ];
        // ⭐CUSTOM FORMULAS (design/custom-formulas.md), one per path a generated module can take:
        // the direct step with f32 functions; the df32 perturbed step with functions, `log`
        // (`DiffLog`), a complex power (`DiffPow`) and `sqrt` (`DiffSqrt`); the floatexp step at
        // 1e40× and at 1e100×. Deep views are the custom-formula group's own: bisected in bignum,
        // checked decidable and stable. The "cut" view sits ON the negative real axis, so half its
        // pixels take the other branch of the power; `z^p1`'s seam there is the formula's own
        // discontinuity (a complex exponent has no conjugate symmetry), matched by bignum at 0 of
        // 1,024 samples, and a power that never saw the crossing broke this golden (meanΔ 43).
        // `√(z⁴ + c)` IS the Mandelbrot set in w = z² — its seahorse renders as the built-in's
        // does, through a sqrt with the cut crossed at every turn. The log view is `log(z + 1)`'s,
        // whose argument stays off the cut: the group's crossing log (`log(z + 0.5)`) is chaos at
        // any structured depth, which a golden must not be. Short iteration counts where functions
        // run: their long orbits are chaos no single-precision GPU follows alike, and a golden must
        // hold on another card.
        // (name, source, params, center_x, center_y, zoom, max_iter, palette_idx)
        type CustomGoldenSpec =
            (&'static str, &'static str, &'static [(f64, f64)], &'static str, &'static str, f64, u32, usize);
        let custom_specs: &[CustomGoldenSpec] = &[
            ("custom-sincos", "z = sin(z) + cos(z)*cos(z + pi) + c", &[], "0.0", "0.0", 0.45, 100, 0),
            ("custom-sincos-1e6", "z = sin(z) + cos(z)*cos(z + pi) + c", &[], "0.8433651341985982", "0.6559506599322431", 1.0e6, 150, 0),
            ("custom-log-1e6", "z = z^2 + 0.3*log(z + 1) + c", &[], "3.12358602939656217655721264291684642451682702136906079104549174840776970540901e-1", "2.4294558006417705509271924826816340358332255239354372640620070150641041095696e-1", 1.0e6, 200, 1),
            ("custom-cpow-cut-1e5", "z = z^p1 + c", &[(2.2, 0.3)], "-8.74282550499493813679694360896665518897154704382566209070609675180207887024153e-1", "0.0", 1.0e5, 200, 0),
            ("custom-sqrt-seahorse-1e6", "z = sqrt(z^4 + c)", &[], SX, SY, 1.0e6, 1500, 0),
            ("custom-zpc-1e40", "z = z^2 + p1*z + c", &[(0.25, -0.1)], "2.5825378554724790808958856226538137436477114325577125141614518618614686040981e-1", "2.00864055425637292686986232460580778294541444653850380864716747871520178888717e-1", 1.0e40, 2000, 2),
            ("custom-zsq-spiral-1e100", "z = z^2 + c", &[], "-2.8041054305504546698407770028983979273643258419006230007410381499044388400475119315630293940283589087269554184451138185325406436e-2", "6.94892753899652385892994339498967288039114990163797857613653087250435024223067409755982283759024506296110477464801459921366328420e-1", 1.0e100, 60000, 0),
        ];
        let all_specs: Vec<(GoldenSpec, Option<(&'static str, &'static [(f64, f64)])>)> = specs
            .iter()
            .map(|s| (*s, None))
            .chain(custom_specs.iter().map(|&(name, src, params, cx, cy, zoom, iter, palette)| {
                ((name, FractalKind::Custom, cx, cy, zoom, iter, 0, palette, false), Some((src, params)))
            }))
            .collect();
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
        for &((name, fractal, cx, cy, zoom, iter, method, palette, relief), custom) in &all_specs {
            // A filter matches goldens by group tag or by individual spec name
            // (`--selftest-filter multibrot3-1e6` re-renders one golden in seconds).
            if !(want("goldens") || filter.as_ref().is_some_and(|f| name.contains(f.as_str()))) {
                continue;
            }
            self.fractal = fractal;
            if let Some((src, params)) = custom {
                match crate::custom_formula::CustomFormula::compile(src, params) {
                    Ok(c) => self.custom = Some(std::sync::Arc::new(c)),
                    Err(e) => {
                        goldens.push((name.to_string(), 0, 0.0, 0, false, format!("formula does not compile: {e}"), "RENDER ERROR"));
                        continue;
                    }
                }
            }
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
            let family = match custom {
                Some((src, params)) if params.is_empty() => format!("--formula \"{src}\""),
                Some((src, params)) => format!(
                    "--formula \"{src}\" --formula-params \"{}\"",
                    params.iter().map(|(re, im)| format!("{re},{im}")).collect::<Vec<_>>().join(";")
                ),
                None => format!("--fractal \"{}\"", fractal.name()),
            };
            let reproduce = format!(
                "fractadyne --render --out {name}.png {family} --center {cx} {cy} \
                 --zoom {zoom} --size {gw} --iter {iter} --ss 1 --method {} --palette {palette} \
                 --no-watermark",
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
        // The custom goldens leave a custom formula selected; nothing after them should inherit it.
        self.custom = None;
        self.fractal = FractalKind::Mandelbrot;

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
        // A `|` in a cell (a formula's `|z|`) would split the row: escape it.
        let cell = |s: &str| s.replace('|', "\\|");
        for c in &checks {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                cell(c.category),
                cell(&c.name),
                cell(&c.params),
                cell(&c.result),
                cell(c.threshold),
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
