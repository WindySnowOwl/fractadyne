//! `--bench-matrix`: a path-coverage performance + regression suite.
//!
//! The standardized benchmark ([`crate::scripting::begin_standard_bench`]) is a single Seahorse
//! dive to 1e12× — great for one comparable score, but it barely touches the deep floatexp / BLA /
//! rebasing machinery and never varies fractal or coloring. This suite instead runs a MATRIX of
//! short renders, each pinned to exercise one rendering path, and reports for every segment:
//!
//! * the **CPU bignum-setup split** (`ref` / `series` / `bla` ms — from [`crate::profile::ProfSetup`],
//!   the reference build that dominates cold deep renders), and
//! * the **pure-GPU pass times** (`gpu-it` / `gpu-col`, via `TIMESTAMP_QUERY`), and
//! * the **deterministic shader event counters** (`rebase` / `ext` / `bla` / `maxiter` — the
//!   machine-independent "which path actually ran, and how hard" signal).
//!
//! Timings are hardware-dependent; the counters + mode/skip/orbit-length are EXACT and reproducible
//! (same math ⇒ same counts). So regression detection is two-tier: a change in any deterministic
//! field is an **algorithmic regression** (flagged hard, machine-independent), while a slower time
//! vs. a blessed baseline is a **performance regression** (flagged soft, only meaningful on the same
//! GPU). `--bless` records the baseline; a plain run compares against it.
//!
//! Glitch correction is deliberately excluded (data-dependent cost + the known deep-interior
//! multi-ref pathology) so every segment is a single deterministic dispatch.

use crate::fractal::FractalKind;
use fractadyne_core::Viewport;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::time::Instant;

const LOG2_10: f64 = std::f64::consts::LOG2_10;
/// Render size for every segment — small so the whole matrix runs in seconds, big enough that the
/// path machinery (references, BLA, rebasing) is genuinely exercised.
const SIZE: u32 = 384;
/// Baseline file (repo-relative). Written by `--bless`, read for regression comparison.
const BASELINE_PATH: &str = "benchmarks/bench-matrix-baseline.json";
/// A segment slower than `baseline × (1 + this)` is flagged as a performance regression. Generous
/// because run-to-run variance on the small CPU bignum builds (~10-30 ms) is real — only
/// *substantial* slowdowns should warn.
const TIMING_REGRESSION: f64 = 0.35;
/// Ignore timing deltas below this — sub-ms jitter isn't a regression.
const TIMING_FLOOR_MS: f64 = 1.0;

// ── Canonical high-precision centers (enough significant digits for the depth they're used at) ──
/// Seahorse Valley, ~34 digits — good to ~1e30×. Structure at every scale in that range.
const SEAHORSE: (&str, &str) = (
    "-0.743643887037158704752191506114774",
    "0.131825904205311970493132056385139",
);
/// Deep dip-carrying interior orbit (validation corpus 14), ~160 digits — good to ~1e148×. The
/// heavy-rebasing / extended-range-sample regime of every recent deep-zoom pathology.
const DEEP_INTERIOR: (&str, &str) = (
    "-0.3158354656090698908113251908145989842764104941136552011217533774266655202463327904910559501703762081531934176786217990113494418705307973163264218287292234362119",
    "0.6533553743954627788289923830392687875350977003260517837408108019649970888461393846103786781501651324966145060684808980380361143296058258024081840162818693511972",
);
/// Misiurewicz (4,1) exact center, ~330 digits — good past 1e300×. Deep floatexp with a large
/// bignum reference (stresses the cold reference-build lever hardest).
const M41: (&str, &str) = (
    "-0.101096363845622161025785445738622565463805442826253483876931177660780840740470584274821219810516779033404531908556741193971546144260911882355703907680301615758418143435454482306810666402941937826741189160781952644899539699533446682339312669900981755715042520653886699186134433396160061056277538514425960956070356822164327995613303692671476",
    "0.9562865108091415007710960577299774358098333365105291700343143215005246590657167325269784107873398072043444724926469284366752406567465722656200815719741551313831228054884443810571677870203738395055477279309548311953190385170024877917670113105649462054553485859930615004427397447762045980085973915977603864276670744042253335354323566859404712",
);

/// Misiurewicz (4,3), the "north antenna" — Newton-refined to 200 digits and verified
/// (`f^7(0,c) == f^4(0,c)` to 1e-260). **|λ| = 3.49331342397577021**, i.e. 0.5432 decades per
/// self-similar rung.
///
/// ⭐**WHY THIS CENTRE CARRIES THE MODE-BOUNDARY PAIRS.** At a Misiurewicz point the neighbourhood
/// is self-similar under magnification by |λ|: the picture at |λ|^n and at |λ|^(n+1) is the SAME
/// picture. And this particular λ lands its rungs either side of both arithmetic thresholds:
///
/// | boundary | rung below | rung above |
/// |---|---|---|
/// | direct → df32 (1e4)      | n=7  → 1e3.8027  | n=8  → 1e4.3459  |
/// | df32 → floatexp (1e28)   | n=51 → 1e27.7051 | n=52 → 1e28.2484 |
///
/// So each pair renders the same scene at the same iteration count with the ONLY variable being
/// the arithmetic mode. Every other zoom-band segment confounds "deeper" with "different scene";
/// these two pairs do not, which is what makes them a control rather than another sample.
/// Six device losses sit on the 0→2 crossover, and a mode switch resets the budget to unmeasured
/// — this is the segment that prices that transition honestly.
///
/// ⚠The `MISIUREWICZ_POI` table in `main.rs` stores this point at **30 digits**, which is fine for
/// its purpose (jump there at 1e4) and NOT enough here: the n=52 rung at 1e28.2 needs ~43. Use
/// this constant for anything deep.
const M43: (&str, &str) = (
    "-0.17300671609209016477613828946770372458888764012137959744393547643020180353562520419771537653705885405166925649896461064949139318317341128327473274939564013861471073570747748645616285570542524292",
    "1.062752280849242560235197612681136690229320617854029710218025383997229644009518733109036931633002903184739525797020239071889093827262374176800484266893692470946462136107956050977216146050440014861",
);

/// Period-3 nucleus on the real axis (the "airplane"), Newton-refined and verified
/// (`f^3(0,c) == 0` to 1e-260). Superattracting, so the interior is max-iter by construction.
///
/// ⭐**A DIFFERENT COST REGIME, NOT A DIFFERENT DEPTH.** Every other segment here is a
/// boundary/filament field where escape is fast for most pixels. A minibrot centred in frame is
/// the opposite: interior pixels run the FULL iteration count and nothing skips them. That is the
/// regime behind the e100 crash and crash-1787158916 (`bla_skip` collapsing to 0 made nominal cost
/// equal real cost at exactly the wrong moment), and the suite has had exactly one sample of it
/// (`deep-interior-1e148`).
#[allow(dead_code)]
const NUC_P3_AIRPLANE: (&str, &str) = (
    "-1.75487766624669276004950889635852869189460661777279314398928397064608065512808109073822709284225030928424250332290264124860835485690618977164965994632469966813104274946982261440371534869499455838",
    "0.0",
);

/// Period-3 nucleus off the real axis (the Douady rabbit), Newton-refined and verified. The
/// complex-plane counterpart to the airplane: same period, same superattracting interior, but a
/// centre with a non-zero imaginary part, so it also exercises the asymmetric reference path.
#[allow(dead_code)]
const NUC_P3_RABBIT: (&str, &str) = (
    "-0.12256116687665361997524555182073565405269669111360342800535801467695967243595945463088645357887481240949056222668565608737687927706348295237058440387498128664371745018100604771442604213417107462",
    "0.744861766619744236593170428604392367240163084906824574201847592154415217837839767791143754932964152277675885011581938530388519940683928434900568426896172321353023546715478213108387944760024089761",
);

/// Minibrot nucleus of period **145**, embedded in the M(4,3) neighbourhood at rung n=51 —
/// i.e. at magnification |λ|^51 = 1e27.7051, the SAME depth as `m43-n51-df32`.
///
/// ⭐**THIS IS THE SUITE'S FIRST MATCHED-DEPTH COST-REGIME CONTROL.** Paired with the `M43`
/// segment at the identical magnification and iteration count, the only thing that differs is
/// what is in frame: a filament/spar field where most pixels escape fast, against a minibrot
/// whose interior runs the full count and which BLA cannot skip. Every other cost comparison in
/// this matrix confounds "harder" with "deeper" or with "another scene"; this pair does not.
///
/// **Found by continuation, not by search.** At a Misiurewicz point the neighbourhood maps to
/// itself under `c → c_M + (c − c_M)/λ` with λ COMPLEX, and the embedded minibrots' periods step
/// by the cycle length (3) per rung. So each rung's nucleus is predicted from the previous one
/// and Newton only polishes it. Re-detecting the period at every rung instead — the obvious
/// approach — dies at 1e12: it keeps re-converging on the same shallow minibrots, which then sit
/// outside the shrinking view.
///
/// ✅Verified: `f^145(0,c)` residual 1e-182, and `|c − c_M| / span` came out **1.205 at every one
/// of the 37 rungs** from n=23 to n=59 — the nucleus holds the same relative position in frame at
/// every depth, which is the self-similarity showing up as a measured invariant.
const NUC_M43_N51_P145: (&str, &str) = (
    "-0.17300671609209016477613828892344937818294446303608009957806281757183195507977072742348294212708588386692297082665678755113272002744005837554630926432536299603122427863386902200227993785523168373931237",
    "1.0627522808492425602351976134607669691825780821170443838668797303774200334994733831709142678496588175434975862964984603506843104167050701809784184697102877667661049049374093963121168780435528196423078",
);

/// The same ladder one rung deeper: period **148** at |λ|^52 = 1e28.2484, matching
/// `m43-n52-fe`. Period steps by exactly the cycle length, so this and `NUC_M43_N51_P145`
/// straddle the df32→floatexp threshold as an interior-regime pair, mirroring the filament-regime
/// pair on `M43`. Four segments, two depths, two cost regimes, one arithmetic boundary.
const NUC_M43_N52_P148: (&str, &str) = (
    "-0.17300671609209016477613828968086492263222461633159461350151396646757272565776747890496804435693173209396615447468249693735678404193195389654559498706732625582328961673621848801329031469756953684956742",
    "1.0627522808492425602351976125118914168351481967386057092408603462880065133013660108479767697883824279792457362691209904052076797116947765222391392158185731241429722491655792590185276866057337474176758",
);

/// One matrix segment: a pinned render that exercises a specific path.
pub(crate) struct Segment {
    pub group: &'static str,
    pub name: String,
    pub fractal: FractalKind,
    pub cx: String,
    pub cy: String,
    pub zoom_log10: f64,
    pub iter: u32,
    pub method: u32,
    pub sa: bool,
    pub bla: bool,
    /// Fast + stable enough to assert as a rendering-pipeline sanity check in `--selftest`.
    pub deterministic: bool,
}

fn seg(
    group: &'static str,
    name: &str,
    fractal: FractalKind,
    c: (&str, &str),
    zoom_log10: f64,
    iter: u32,
    method: u32,
    sa: bool,
    bla: bool,
    deterministic: bool,
) -> Segment {
    Segment {
        group,
        name: name.to_string(),
        fractal,
        cx: c.0.to_string(),
        cy: c.1.to_string(),
        zoom_log10,
        iter,
        method,
        sa,
        bla,
        deterministic,
    }
}

/// The full path matrix: a banded Mandelbrot zoom sweep (each numeric regime + SA/BLA variants),
/// a per-coloring set (the iteration-skip-blocking methods), and a per-fractal set (formula paths).
pub(crate) fn matrix() -> Vec<Segment> {
    use FractalKind as F;
    let mut v = vec![
        // ── Zoom bands: f64 direct → df32 perturbation → floatexp (+ SA/BLA variants) ──
        seg("zoom-band", "direct-1e2", F::Mandelbrot, SEAHORSE, 2.0, 512, 0, true, true, true),
        seg("zoom-band", "df32-1e8", F::Mandelbrot, SEAHORSE, 8.0, 3_000, 0, true, true, true),
        seg("zoom-band", "df32-1e20", F::Mandelbrot, SEAHORSE, 20.0, 15_000, 0, true, true, true),
        seg("zoom-band", "floatexp-1e30-sa", F::Mandelbrot, SEAHORSE, 30.0, 30_000, 0, true, true, true),
        // Same view, SA off then BLA off — isolate each accelerator's contribution.
        seg("zoom-band", "floatexp-1e30-nosa", F::Mandelbrot, SEAHORSE, 30.0, 30_000, 0, false, true, true),
        seg("zoom-band", "floatexp-1e30-nobla", F::Mandelbrot, SEAHORSE, 30.0, 30_000, 0, true, false, true),
        // Deep interior: extended-range samples + heavy Zhuoran rebasing (the pathology regime).
        seg("zoom-band", "deep-interior-1e148", F::Mandelbrot, DEEP_INTERIOR, 148.077, 800_000, 0, true, true, false),
        // Extreme floatexp: a ~330-digit bignum reference — the cold reference-build lever.
        seg("zoom-band", "floatexp-1e300", F::Mandelbrot, M41, 300.0, 200_000, 0, true, true, false),
        // ── λ-LADDER: exactly self-similar rungs straddling both arithmetic-mode boundaries ──
        // Same centre, same iteration count, same scene (M(4,3) is self-similar under ×|λ|) — the
        // ONLY difference across each pair is which arithmetic the renderer selects. A cost ratio
        // here is the mode's price; anywhere else in this matrix it is confounded with the view.
        // ⚠Read the pairs, not the absolute numbers: n=7/8 is near the set where self-similarity
        // is still approximate, while n=51/52 is deep enough to be exact for this purpose.
        seg("lambda-ladder", "m43-n7-direct-1e3.80", F::Mandelbrot, M43, 3.8027, 1_500, 0, true, true, true),
        seg("lambda-ladder", "m43-n8-df32-1e4.35", F::Mandelbrot, M43, 4.3459, 1_500, 0, true, true, true),
        seg("lambda-ladder", "m43-n51-df32-1e27.71", F::Mandelbrot, M43, 27.7051, 30_000, 0, true, true, true),
        seg("lambda-ladder", "m43-n52-fe-1e28.25", F::Mandelbrot, M43, 28.2484, 30_000, 0, true, true, true),
        // ── COST REGIME at MATCHED DEPTH — the axis this matrix did not have ──
        // Same magnification and iteration count as the two `m43-n51/n52` segments above, same
        // neighbourhood, different scene: a period-145/148 minibrot instead of the spar field.
        // So `interior-n51 / m43-n51` is a pure cost-regime ratio with depth, scene family, mode
        // and ask all held fixed — and the n52 pair repeats it on the far side of the
        // df32→floatexp boundary. `bla-skip` and `maxiter` in the counters are where to read it.
        //
        // ⚠A previous revision put a period-3 minibrot at 1e2 here and called it the interior
        // regime. It measured 31% max-iter pixels against `direct-1e2`'s 39% — LESS
        // interior-dominated than the segment it was meant to contrast with — and direct mode has
        // no BLA to collapse in the first place. The regime needs depth, which needs a deep
        // nucleus; that is what these two constants are.
        seg("regime", "interior-n51-df32-1e27.71", F::Mandelbrot, NUC_M43_N51_P145, 27.7051, 30_000, 0, true, true, true),
        seg("regime", "interior-n52-fe-1e28.25", F::Mandelbrot, NUC_M43_N52_P148, 28.2484, 30_000, 0, true, true, true),
        // ── Coloring paths (Mandelbrot @1e20×): smooth vs the iteration-skip-blocking methods ──
        seg("coloring", "color-smooth", F::Mandelbrot, SEAHORSE, 20.0, 15_000, 0, true, true, true),
        seg("coloring", "color-stripe", F::Mandelbrot, SEAHORSE, 20.0, 15_000, 1, true, true, true),
        seg("coloring", "color-trap", F::Mandelbrot, SEAHORSE, 20.0, 15_000, 3, true, true, true),
        seg("coloring", "color-decomposition", F::Mandelbrot, SEAHORSE, 20.0, 15_000, 5, true, true, true),
    ];
    // ── Per-fractal formula paths: each family's home view (direct path, mix of interior/exterior) ──
    for kind in FractalKind::ALL {
        let (cx, cy) = kind.spec().default_center;
        v.push(Segment {
            group: "fractal",
            name: format!("fractal-{}", kind.name().to_lowercase().replace(' ', "")),
            fractal: kind,
            cx: format!("{cx:.17e}"),
            cy: format!("{cy:.17e}"),
            zoom_log10: 0.9, // ~8× — whole set in frame: interior (max-iter) + boundary + exterior
            iter: 2_000,
            method: 0,
            sa: true,
            bla: true,
            deterministic: true,
        });
    }
    v
}

/// Measured result for one segment (deterministic signature + timings). Timings kept are the two
/// the report prints and the baseline stores (`ref_ms`, `bla_ms`, `render_ms` → `gpu_ms`); the
/// per-pass GPU split lives in `--profile`, not here.
struct SegResult {
    name: String,
    mode: u32,
    sa_skip: u32,
    orbit_len: u32,
    eff_iter: u32,
    counters: [u64; 5], // rebase, ext, glitch, bla_skip, maxiter
    ref_ms: f64,
    bla_ms: f64,
    render_ms: f64,
}

/// The deterministic (machine-independent) part of a baseline entry.
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
struct BaseSeg {
    mode: u32,
    sa_skip: u32,
    orbit_len: u32,
    eff_iter: u32,
    counters: [u64; 5],
    ref_ms: f64,
    gpu_ms: f64,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Baseline {
    version: String,
    gpu: String,
    utc: String,
    segments: std::collections::BTreeMap<String, BaseSeg>,
}

/// Least-squares fit of `now = scale x baseline + fixed` over `(baseline_ms, now_ms)` pairs.
///
/// ⭐⭐**The two numbers a list of percentages cannot separate.** A FIXED cost added to every
/// segment reads as +59% on a 16 ms one and +1.8% on a 500 ms one — one cause, printed as a
/// scattered set of regressions of wildly different sizes. A SCALE change is proportional and
/// really is "everything got slower". Splitting them is the difference between "eleven regressions"
/// and "one fixed cost, and the renderer is 7% FASTER per unit of work".
///
/// Measured 2026-09-05: scale 0.93–0.99, fixed **+9.4 to +10.2 ms**, cause = the per-call WGSL
/// compile (`shader_module` is rebuilt by every `render_export`, timed 8.9–9.8 ms) against a
/// baseline blessed before the shader grew to ~134 KB.
///
/// `None` when there are too few points to fit, or the baselines are all equal (a vertical fit).
fn scale_and_fixed(pts: &[(f64, f64)]) -> Option<(f64, f64)> {
    if pts.len() < 6 {
        return None;
    }
    let n = pts.len() as f64;
    let (sx, sy) = (pts.iter().map(|p| p.0).sum::<f64>(), pts.iter().map(|p| p.1).sum::<f64>());
    let sxx = pts.iter().map(|p| p.0 * p.0).sum::<f64>();
    let sxy = pts.iter().map(|p| p.0 * p.1).sum::<f64>();
    let denom = n * sxx - sx * sx;
    if denom.abs() < 1e-9 {
        return None;
    }
    let scale = (n * sxy - sx * sy) / denom;
    Some((scale, (sy - scale * sx) / n))
}

#[cfg(test)]
mod fit_tests;

fn median(v: &mut [f64]) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    v[v.len() / 2]
}

impl crate::FractadyneApp {
    /// Measure one matrix segment (`reps` timed renders after a warm-up). Reuses the exact live
    /// request builder so the reference/SA/BLA cost and the shader path match the real pipeline.
    fn measure_segment(
        &mut self,
        s: &Segment,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        reps: u32,
    ) -> SegResult {
        // Pin the configuration this segment needs (deterministic — no session leakage).
        self.set_fractal(s.fractal);
        self.julia_mode = false;
        self.dual = false;
        self.coloring.color_method = crate::ColorMethod::from_u32(s.method);
        self.coloring.use_binary = false;
        self.coloring.use_duotone = false;
        self.coloring.use_custom_palette = false;
        self.render_cfg.auto_iter = false; // fixed count ⇒ reproducible counters
        self.render_cfg.max_iter = s.iter;
        self.render_cfg.series_approx = s.sa;
        self.render_cfg.use_bla = s.bla;
        self.render_cfg.glitch_correct = false; // single deterministic dispatch
        self.export.width = SIZE;
        self.export.ss = 1;
        self.invalidate_refs();

        let mut vp = Viewport::new(SIZE as f64, SIZE as f64);
        let cx = fractadyne_core::parse_bf(&s.cx)
            .unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(-0.5, 64));
        let cy = fractadyne_core::parse_bf(&s.cy)
            .unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(0.0, 64));
        vp.set_center_log2mag(cx, cy, s.zoom_log10 * LOG2_10);

        // Build the request once (records reference/series/bla timings in self.prof). Reset first:
        // direct mode never calls prof.set(), so without this it would inherit the previous
        // (perturbation) segment's stale reference time.
        self.prof.set(crate::profile::ProfSetup::default());
        let req = self.current_export_request_for(&vp, false);
        let setup = self.prof.get();
        let progress = AtomicU32::new(0);
        let cancel = AtomicBool::new(false);

        // Warm-up (shader/pipeline compile, first upload) — not counted.
        let _ = fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel);

        let mut render_ms = Vec::new();
        let mut counters = [0u64; 5];
        for _ in 0..reps.max(1) {
            let t = Instant::now();
            let res = fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel);
            render_ms.push(t.elapsed().as_secs_f64() * 1000.0);
            if let Ok(er) = &res {
                // Slots 0..5: rebase, ext, glitch, bla_skip, maxiter (deterministic).
                counters.copy_from_slice(&er.counters[0..5]);
            }
        }

        SegResult {
            name: s.name.clone(),
            mode: req.mode,
            sa_skip: req.sa_skip,
            orbit_len: req.orbit_len,
            eff_iter: req.max_iter,
            counters,
            ref_ms: setup.reference_ms,
            bla_ms: setup.bla_ms,
            render_ms: median(&mut render_ms),
        }
    }

    /// Run the full matrix, print the report, and either bless a baseline (`bless=true`) or compare
    /// against the existing one. Returns a process exit code: 2 on an algorithmic (deterministic)
    /// regression, 0 otherwise (performance regressions are warnings).
    pub(crate) fn run_bench_matrix(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        bless: bool,
        reps: u32,
        det_only: bool,
    ) -> i32 {
        let reps = reps.clamp(1, 20);
        let mut segs = matrix();
        if det_only {
            segs.retain(|s| s.deterministic);
        }
        println!(
            "Fractadyne path-matrix benchmark — {} segments × {reps} reps, {SIZE}px  ({})",
            segs.len(),
            self.gpu_name
        );
        println!(
            "  {:<22} {:>4} {:>8} {:>8} {:>8} {:>8} {:>8} {:>10} {:>10} {:>10} {:>10}",
            "segment", "mode", "eff-it", "sa-skip", "ref ms", "bla ms", "gpu ms", "rebase",
            "ext", "bla-skip", "maxiter"
        );

        let mut results = Vec::new();
        let mut cur_group = "";
        for s in &segs {
            if s.group != cur_group {
                cur_group = s.group;
                println!("  — {cur_group} —");
            }
            let r = self.measure_segment(s, device, queue, reps);
            println!(
                "  {:<22} {:>4} {:>8} {:>8} {:>8.2} {:>8.2} {:>8.2} {:>10} {:>10} {:>10} {:>10}",
                r.name, r.mode, r.eff_iter, r.sa_skip, r.ref_ms, r.bla_ms, r.render_ms,
                r.counters[0], r.counters[1], r.counters[3], r.counters[4]
            );
            results.push(r);
        }

        if bless {
            return self.bless_baseline(&results);
        }
        self.compare_baseline(&results)
    }

    fn bless_baseline(&self, results: &[SegResult]) -> i32 {
        let segments = results
            .iter()
            .map(|r| {
                (
                    r.name.clone(),
                    BaseSeg {
                        mode: r.mode,
                        sa_skip: r.sa_skip,
                        orbit_len: r.orbit_len,
                        eff_iter: r.eff_iter,
                        counters: r.counters,
                        ref_ms: r.ref_ms,
                        gpu_ms: r.render_ms,
                    },
                )
            })
            .collect();
        let baseline = Baseline {
            version: crate::version_string(),
            gpu: self.gpu_name.clone(),
            utc: crate::utc_string(now_unix()),
            segments,
        };
        let path = std::path::Path::new(BASELINE_PATH);
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        match serde_json::to_string_pretty(&baseline)
            .map_err(|e| e.to_string())
            .and_then(|j| std::fs::write(path, j).map_err(|e| e.to_string()))
        {
            Ok(()) => {
                println!("\nBlessed baseline ({} segments) → {BASELINE_PATH}", results.len());
                0
            }
            Err(e) => {
                eprintln!("\nFailed to write baseline {BASELINE_PATH}: {e}");
                1
            }
        }
    }

    fn compare_baseline(&self, results: &[SegResult]) -> i32 {
        let baseline = match load_baseline(std::path::Path::new(BASELINE_PATH)) {
            Ok(Some(b)) => b,
            Ok(None) => {
                println!(
                    "\nNo baseline at {BASELINE_PATH} — run `--bench-matrix --bless` to record one \
                     (needed to flag regressions)."
                );
                return 0;
            }
            Err(e) => {
                eprintln!("\nBaseline {BASELINE_PATH} is unreadable: {e}");
                return 1;
            }
        };
        let same_gpu = baseline.gpu == self.gpu_name;
        println!(
            "\nvs. baseline {} ({}, {}){}",
            baseline.version,
            baseline.gpu,
            baseline.utc,
            if same_gpu { "" } else { "  [different GPU — timings not compared]" }
        );

        let (mut drift, mut slower, mut missing) = (0u32, 0u32, 0u32);
        // Segments whose signature differs only because this is a different GPU (see below).
        let mut cross = 0u32;
        for r in results {
            let Some(b) = baseline.segments.get(&r.name) else {
                println!("  ? {:<22} not in baseline (new segment)", r.name);
                missing += 1;
                continue;
            };
            // Path-signature regression. These fields are deterministic for a given BUILD on a
            // given GPU, but they are NOT machine-independent, which this check assumed until an
            // RX 6800 XT run proved otherwise (2026-08-14): its shader compiler preserves the
            // df32 error-free transforms that NVIDIA folds, so escape decisions differ by a pixel
            // here and there, and rebase/bla_skip/eff_iter counts move with them. Seven of
            // twenty-two segments reported "ALGORITHMIC DRIFT" on a perfectly healthy card.
            //
            // So cross-GPU differences are reported but NOT counted as drift: on the blessing GPU
            // this stays the exact tripwire it was built to be, and on any other card it becomes
            // informational. (The same reasoning as the goldens' cross-GPU tolerance.)
            let cur_det = (r.mode, r.sa_skip, r.orbit_len, r.eff_iter, r.counters);
            let base_det = (b.mode, b.sa_skip, b.orbit_len, b.eff_iter, b.counters);
            if cur_det != base_det {
                if same_gpu {
                    drift += 1;
                    println!("  ✗ {:<22} ALGORITHMIC DRIFT", r.name);
                } else {
                    cross += 1;
                    println!("  ~ {:<22} differs (cross-GPU, expected)", r.name);
                }
                if r.mode != b.mode {
                    println!("      mode {} → {}", b.mode, r.mode);
                }
                if r.sa_skip != b.sa_skip {
                    println!("      sa_skip {} → {}", b.sa_skip, r.sa_skip);
                }
                if r.orbit_len != b.orbit_len {
                    println!("      orbit_len {} → {}", b.orbit_len, r.orbit_len);
                }
                if r.eff_iter != b.eff_iter {
                    println!("      eff_iter {} → {}", b.eff_iter, r.eff_iter);
                }
                if r.counters != b.counters {
                    println!("      counters {:?} → {:?}", b.counters, r.counters);
                }
            }
            // Performance regression (same GPU only): a timing meaningfully above baseline.
            if same_gpu {
                for (label, cur, base) in [
                    ("ref", r.ref_ms, b.ref_ms),
                    ("gpu", r.render_ms, b.gpu_ms),
                ] {
                    if cur > base * (1.0 + TIMING_REGRESSION) && cur - base > TIMING_FLOOR_MS {
                        slower += 1;
                        println!(
                            "  ⚠ {:<22} {label} {:.2}ms → {:.2}ms  (+{:.0}%)",
                            r.name,
                            base,
                            cur,
                            (cur / base - 1.0) * 100.0
                        );
                    }
                }
            }
        }

        // ⭐⭐**Say whether the slow segments share a FIXED cost, because that is not a regression
        // and it is what made this half get read past.** Fitting `now = a·base + k` separates the
        // two things a reader cannot tell apart from a list of percentages:
        //   * `a` away from 1 — everything scaled: a genuinely slower (or faster) renderer or box.
        //   * `k` away from 0 — a constant added per segment, which swamps a 16 ms segment (+59%)
        //     and vanishes on a 500 ms one (+1.8%). The SAME k produces both, so a per-segment
        //     percentage list looks like a scattered regression when it is one number.
        // Measured 2026-09-05: a = 0.99, k = +9.4 ms, and the cause is the per-call WGSL compile
        // (`shader_module` is rebuilt by every `render_export`; timed at 8.9–9.8 ms). The baseline
        // predates the shader's growth to ~134 KB. See TODO.md.
        // ⚠This REPORTS, it does not subtract: masking a fixed cost would hide a real one appearing.
        if same_gpu {
            let pts: Vec<(f64, f64)> = results
                .iter()
                .filter_map(|r| {
                    baseline.segments.get(&r.name).and_then(|b| {
                        (b.gpu_ms > 0.0).then_some((b.gpu_ms, r.render_ms))
                    })
                })
                .collect();
            if let Some((a, k)) = scale_and_fixed(&pts) {
                {
                    println!(
                        "\n  shape: now ≈ {a:.2}× baseline {} {:.1} ms fixed, over {} segments",
                        if k >= 0.0 { "+" } else { "−" },
                        k.abs(),
                        pts.len()
                    );
                    if k.abs() > 3.0 {
                        println!(
                            "         a FIXED {:.0} ms/segment dominates the cheap segments and vanishes on the dear \
                             ones — read the warnings above as that one cost, not as N regressions.",
                            k.abs()
                        );
                    }
                    if (a - 1.0).abs() > 0.10 {
                        println!(
                            "         and a {:.0}% SCALE change, which IS proportional — that part is a real \
                             speed difference.",
                            (a - 1.0) * 100.0
                        );
                    }
                }
            }
        }

        println!(
            "\n{} segments · {drift} algorithmic drift · {slower} slower{} · {missing} new{}",
            results.len(),
            if same_gpu { "" } else { " (skipped — diff GPU)" },
            if cross > 0 {
                format!(" · {cross} cross-GPU differences (expected)")
            } else {
                String::new()
            }
        );
        if drift > 0 {
            println!(
                "ALGORITHMIC REGRESSION: a rendering path changed its exact output signature. If \
                 intended, re-bless with `--bench-matrix --bless`."
            );
            2
        } else {
            if cross > 0 {
                println!(
                    "The baseline was recorded on {}; this is {}. Signature differences on \
                     another GPU are EXPECTED — escape decisions, and the rebase/skip counts that \
                     follow from them, legitimately differ between vendors. This is not a \
                     regression, and the tripwire remains exact on the baseline's own card.",
                    baseline.gpu, self.gpu_name
                );
            }
            println!("No algorithmic regressions.{}", if slower > 0 { " (performance warnings above.)" } else { "" });
            0
        }
    }
}

/// Load the baseline: `Ok(None)` if the file is simply absent, `Err` if present-but-unreadable.
fn load_baseline(path: &std::path::Path) -> Result<Option<Baseline>, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).map(Some).map_err(|e| e.to_string()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// One selftest row: a deterministic segment's path signature vs. the baseline.
pub(crate) struct MatrixCheck {
    pub name: String,
    pub pass: bool,
    pub detail: String,
}

impl crate::FractadyneApp {
    /// Run the fast/deterministic matrix segments and compare each one's machine-independent path
    /// signature (mode / sa_skip / orbit_len / eff_iter / counters) against the blessed baseline —
    /// an algorithmic-regression tripwire for `--selftest`. One `MatrixCheck` per segment. With no
    /// baseline present, returns a single passing note (nothing to diff against yet). Dirties render
    /// config, so the selftest must call this LAST.
    pub(crate) fn bench_matrix_selftest_checks(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        baseline_path: &std::path::Path,
    ) -> Vec<MatrixCheck> {
        let baseline = match load_baseline(baseline_path) {
            Ok(Some(b)) => b,
            Ok(None) => {
                return vec![MatrixCheck {
                    name: "bench-matrix baseline".into(),
                    pass: true,
                    detail: format!(
                        "no baseline at {} — skipped (bless with `--bench-matrix --bless`)",
                        baseline_path.display()
                    ),
                }]
            }
            Err(e) => {
                return vec![MatrixCheck {
                    name: "bench-matrix baseline".into(),
                    pass: false,
                    detail: format!("baseline unreadable: {e}"),
                }]
            }
        };
        let same_gpu = baseline.gpu == self.gpu_name;
        let mut out = Vec::new();
        if !same_gpu {
            out.push(MatrixCheck {
                name: "bench-matrix baseline GPU".into(),
                pass: true,
                detail: format!(
                    "baseline recorded on {}; this is {} — signature differences below are \
                     EXPECTED and reported, not failed",
                    baseline.gpu, self.gpu_name
                ),
            });
        }
        for s in matrix().iter().filter(|s| s.deterministic) {
            let r = self.measure_segment(s, device, queue, 1);
            match baseline.segments.get(&r.name) {
                Some(b) => {
                    let cur = (r.mode, r.sa_skip, r.orbit_len, r.eff_iter, r.counters);
                    let base = (b.mode, b.sa_skip, b.orbit_len, b.eff_iter, b.counters);
                    // Exact on the GPU that blessed the baseline; informational on any other.
                    // These signatures are deterministic per build+GPU but NOT machine-
                    // independent: an RX 6800 XT reported twelve of these as DRIFT purely because
                    // its compiler keeps the df32 error-free transforms NVIDIA folds, moving
                    // escape decisions and the counts that follow. See the `--bench-matrix`
                    // comparison for the full reasoning.
                    let pass = cur == base || !same_gpu;
                    let detail = if cur == base {
                        format!("mode {} eff-it {} sa-skip {} counters ok", r.mode, r.eff_iter, r.sa_skip)
                    } else {
                        format!(
                            "{} mode {}→{} sa-skip {}→{} orbit {}→{} counters {:?}→{:?}",
                            if same_gpu { "DRIFT" } else { "cross-GPU (expected)" },
                            b.mode, r.mode, b.sa_skip, r.sa_skip, b.orbit_len, r.orbit_len,
                            b.counters, r.counters
                        )
                    };
                    out.push(MatrixCheck { name: r.name, pass, detail });
                }
                None => out.push(MatrixCheck {
                    name: r.name,
                    pass: true,
                    detail: "new segment (not in baseline)".into(),
                }),
            }
        }
        out
    }
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
