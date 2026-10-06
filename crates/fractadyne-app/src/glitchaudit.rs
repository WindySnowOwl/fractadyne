//! `--render … --glitch-audit [N]`: at the pixels glitch correction CHANGES, which image is right?
//!
//! Measured 2026-09-27 on the benchmark corpus (RTX 3080, 4K): correction costs up to 5× the
//! render — 4.6e1105 takes 76 s with it and 15.6 s with `--no-glitch` — to change 0.004–0.02% of
//! the pixels, though strongly (up to 254/255). About 95% of the pixels it flags come back
//! unchanged. Fraktaler-3's and Imagina's pictures match ours without any correction. So either
//! the uncorrected pixels are wrong and our rebasing misses something worth fixing, or they are
//! right and the correction is spending minutes rewriting correct pixels. The two answers call for
//! opposite work, and a picture cannot tell them apart.
//!
//! This renders the view's raw iteration buffer twice — plain (`glitch_on = 0`, exactly
//! `--no-glitch`) and corrected (the export's own `render_corrected_iter`) — and runs the CPU
//! arbitrary-precision dwell oracle (`naive_dwell_bf`, which knows nothing of references,
//! rebasing or BLA) at a sample of the pixels that differ.
//!
//! ⭐**It scores a CONTROL**: the same number of pixels where the two renders AGREE. If the oracle
//! disagrees with both there, the pixel→c mapping or the oracle is wrong and no verdict is given.
//! ⭐**Sub-pixel sensitivity is its own verdict.** The GPU samples a pixel's c to ~1e-4 px (its step
//! is effectively f32 on NVIDIA), so the oracle is also run at ±`STENCIL_PX` in x and y. A pixel
//! whose answer changes inside that stencil is `sensitive`: either render can be "right" there,
//! and scoring it would be scoring noise.

use crate::render::CorrectionBudget;

/// Differing pixels examined (and the same number of controls) when no count is given.
pub(crate) const DEFAULT_SAMPLES: usize = 24;

/// Which two renders the audit compares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AuditKind {
    /// `--glitch-audit`: plain (`glitch_on = 0`) against glitch-corrected.
    Correction,
    /// `--tail-audit`: the all-floatexp mode-2 loop (`TAIL_DF32=0`) against the df32 tail phase.
    Tail,
    /// `--renorm-audit`: the perturbation loop alone (`RENORM=0`) against THE RENORMALIZED STEP.
    Renorm,
}

impl AuditKind {
    /// Column labels for the first and second render.
    fn labels(self) -> (&'static str, &'static str) {
        match self {
            AuditKind::Correction => ("uncorrected", "corrected"),
            AuditKind::Tail => ("floatexp", "df32 tail"),
            AuditKind::Renorm => ("perturbation", "renormalized"),
        }
    }
}
/// The oracle stencil's offset in pixels: well above the GPU's ~1e-4 px sampling error, well
/// below the pixel.
const STENCIL_PX: f64 = 1.0e-3;
/// Two smooth iteration counts agree within half an iteration. The smooth FRACTION varies
/// continuously with the exact sample point, which the GPU holds to ~1e-4 px, and it is stored as
/// f32 (one ulp is 0.03 at 300,000 iterations). So at a deep view a correct render's fraction
/// differs from the oracle's by tenths of an iteration even where both escape on the same step.
/// Measured 2026-09-28 at 1.2e148: up to 0.375 at control pixels where two renders agree bit for
/// bit. A wrong pixel is off by whole iterations, or interior. (0.05 until then, which fitted only
/// shallow counts and failed that view's controls.)
const MATCH_TOL: f32 = 0.5;

/// One pixel's value: escaped at a smooth iteration count, or interior (never escaped).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Value {
    Escaped(f32),
    Interior,
}

impl Value {
    /// From an iteration buffer's channel 0 (negative = interior).
    pub(crate) fn from_channel(v: f32) -> Value {
        if v < 0.0 { Value::Interior } else { Value::Escaped(v) }
    }
    pub(crate) fn agrees(self, o: Value) -> bool {
        match (self, o) {
            (Value::Interior, Value::Interior) => true,
            (Value::Escaped(a), Value::Escaped(b)) => (a - b).abs() <= MATCH_TOL,
            _ => false,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Escaped(v) => write!(f, "{v:.3}"),
            Value::Interior => write!(f, "interior"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Verdict {
    /// The oracle agrees with the corrected render only: correction repaired the pixel.
    CorrectedRight,
    /// The oracle agrees with the uncorrected render only: correction broke the pixel.
    UncorrectedRight,
    /// The oracle agrees with both (possible only for controls, or within tolerance).
    BothRight,
    /// The oracle agrees with neither.
    Neither,
    /// The oracle's own answer changes within the stencil: not scorable.
    Sensitive,
}

/// Judge one pixel. `stable` = the oracle gave the same answer across the stencil.
pub(crate) fn verdict(oracle: Value, stable: bool, uncorrected: Value, corrected: Value) -> Verdict {
    if !stable {
        return Verdict::Sensitive;
    }
    match (oracle.agrees(uncorrected), oracle.agrees(corrected)) {
        (true, true) => Verdict::BothRight,
        (false, true) => Verdict::CorrectedRight,
        (true, false) => Verdict::UncorrectedRight,
        (false, false) => Verdict::Neither,
    }
}

/// Whether a set of oracle answers (centre + stencil) is stable: all interior, or all escaped at
/// the same integer iteration.
pub(crate) fn stable(answers: &[Option<(u32, f32)>]) -> bool {
    match answers.first() {
        None => false,
        Some(first) => answers.iter().all(|a| match (a, first) {
            (None, None) => true,
            (Some((n, _)), Some((m, _))) => n == m,
            _ => false,
        }),
    }
}

/// The export policy this audit produced (`FractadyneApp::correction_wanted` has the evidence):
/// glitch correction runs when the user's `setting` asks for it, EXCEPT for the holomorphic
/// families (Mandelbrot, Multibrot 3–5 — `FormulaCaps::export_glitch_correction` false) outside
/// Julia mode, where it repaired nothing and blackened what it gave up on.
pub(crate) fn correction_applies(setting: bool, formula_id: u32, julia: bool) -> bool {
    setting && (julia || fractadyne_core::formula::caps(formula_id).export_glitch_correction)
}

/// `count` indices spread evenly over `pool` (all of it when it is smaller).
pub(crate) fn spread(pool: &[usize], count: usize) -> Vec<usize> {
    if pool.len() <= count {
        return pool.to_vec();
    }
    (0..count).map(|k| pool[k * pool.len() / count]).collect()
}

struct Sample {
    index: usize,
    control: bool,
    uncorrected: Value,
    corrected: Value,
}

impl crate::FractadyneApp {
    /// Run the audit at the current view and export size; prints its table and returns the summary.
    pub(crate) fn run_glitch_audit(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        samples: usize,
        kind: AuditKind,
    ) -> Result<String, crate::error::AppError> {
        use crate::error::AppError;
        let req = match self.build_export_job() {
            crate::ExportJob::Single(r) => r,
            _ => return Err(AppError::Message("--glitch-audit needs a single view (not dual)".into())),
        };
        if self.julia_mode {
            // The oracles iterate Mandelbrot mode (z₀ = 0, c = the pixel).
            return Err(AppError::Message("--glitch-audit covers Mandelbrot-mode views, not Julia".into()));
        }
        let (w, h) = (req.width as usize, req.height as usize);
        println!(
            "Fractadyne audit ({:?}) — {}\n  {}x{} px, mode {}, iter {}",
            kind,
            crate::version_string(),
            w,
            h,
            req.mode,
            req.max_iter
        );

        // The two raw iteration buffers.
        let (la, lb) = kind.labels();
        let mut plain_req = req.clone();
        plain_req.ss = 1;
        plain_req.glitch_on = 0;
        let plain = |dev: &eframe::wgpu::Device, q: &eframe::wgpu::Queue| {
            fractadyne_gpu::render_iter_tiled(dev, q, &plain_req, crate::tunables::CORRECT_WORK_BUDGET, None, None, None)
        };
        let t0 = std::time::Instant::now();
        let (first, second) = match kind {
            AuditKind::Correction => {
                let a = plain(device, queue)?.pixels;
                let first_s = t0.elapsed().as_secs_f64();
                let t1 = std::time::Instant::now();
                let corrected = self
                    .render_corrected_iter(
                        device,
                        queue,
                        &self.viewport,
                        self.julia_mode,
                        req.width,
                        req.height,
                        64,
                        Some(&req),
                        CorrectionBudget::standard(),
                    )
                    .ok_or_else(|| AppError::Message("glitch correction declined this view".into()))?;
                println!(
                    "  {la} {first_s:.1} s · {lb} {:.1} s ({} references, {} still glitched)",
                    t1.elapsed().as_secs_f64(),
                    corrected.refs_used,
                    corrected.residual
                );
                (a, corrected.pixels)
            }
            AuditKind::Tail => {
                // The same plain render with the phase off, then on; the run's own setting is
                // restored after, whatever it was.
                fractadyne_gpu::set_tail_df32(false);
                let a = plain(device, queue);
                let first_s = t0.elapsed().as_secs_f64();
                fractadyne_gpu::set_tail_df32(true);
                let t1 = std::time::Instant::now();
                let b = plain(device, queue);
                fractadyne_gpu::set_tail_df32(crate::tunables::cost().tail_df32 == 1);
                println!("  {la} {first_s:.1} s · {lb} {:.1} s", t1.elapsed().as_secs_f64());
                (a?.pixels, b?.pixels)
            }
            AuditKind::Renorm => {
                // The same plain render without the step, then with the one this view's reference chose.
                if plain_req.rn.len == 0 {
                    return Ok("audit: VACUOUS — no renormalized step holds at this view".into());
                }
                println!("  step: {} iterations per renormalized step", plain_req.rn.len);
                let mut off_req = plain_req.clone();
                off_req.rn = fractadyne_gpu::Renorm::default();
                let a = fractadyne_gpu::render_iter_tiled(device, queue, &off_req, crate::tunables::CORRECT_WORK_BUDGET, None, None, None)?.pixels;
                let first_s = t0.elapsed().as_secs_f64();
                let t1 = std::time::Instant::now();
                let b = plain(device, queue)?.pixels;
                println!("  {la} {first_s:.1} s · {lb} {:.1} s", t1.elapsed().as_secs_f64());
                (a, b)
            }
        };

        // Which pixels differ (bit for bit), and the pools to sample from.
        let mut differ = Vec::new();
        let mut agree = Vec::new();
        for i in 0..w * h {
            let (a, b) = (first[4 * i], second[4 * i]);
            if a.to_bits() != b.to_bits() {
                differ.push(i);
            } else if a >= 0.0 {
                agree.push(i); // escaped controls: an interior control tells the oracle little
            }
        }
        println!(
            "  differing pixels: {} of {} ({:.4}%)",
            differ.len(),
            w * h,
            100.0 * differ.len() as f64 / (w * h) as f64
        );
        // How far apart, over EVERY differing pixel (the oracle below sees a sample): escaped in both
        // by |Δ smooth iteration|, or escaped in one and interior in the other.
        {
            let mut deltas: Vec<f32> = Vec::new();
            let (mut only_a, mut only_b) = (0usize, 0usize);
            for &i in &differ {
                match (Value::from_channel(first[4 * i]), Value::from_channel(second[4 * i])) {
                    (Value::Escaped(x), Value::Escaped(y)) => deltas.push((x - y).abs()),
                    (Value::Escaped(_), Value::Interior) => only_a += 1,
                    (Value::Interior, Value::Escaped(_)) => only_b += 1,
                    _ => {}
                }
            }
            deltas.sort_by(|x, y| x.total_cmp(y));
            let q = |f: f64| deltas.get(((deltas.len() as f64 - 1.0) * f).round() as usize).copied().unwrap_or(0.0);
            println!(
                "  escaped in both: {} — |Δ smooth| median {:.3e}, 90% {:.3e}, 99% {:.3e}, max {:.3e}; \
                 over 0.5: {}; escaped only in {la}: {only_a}, only in {lb}: {only_b}",
                deltas.len(),
                q(0.5),
                q(0.9),
                q(0.99),
                deltas.last().copied().unwrap_or(0.0),
                deltas.iter().filter(|&&d| d > MATCH_TOL).count(),
            );
        }
        if differ.is_empty() {
            return Ok(format!("audit: VACUOUS — {la} and {lb} agree bit for bit at every pixel of this view"));
        }
        // The step changes almost every pixel by a little (it is an approximation held to 2^-24 per
        // step), so its audit samples the pixels it changes BEYOND the match tolerance: those are
        // the ones a verdict is about.
        if kind == AuditKind::Renorm {
            differ.retain(|&i| !Value::from_channel(first[4 * i]).agrees(Value::from_channel(second[4 * i])));
            println!("  sampling the {} pixels that differ by more than {MATCH_TOL}", differ.len());
            if differ.is_empty() {
                return Ok(format!("audit: no pixel differs between {la} and {lb} by more than {MATCH_TOL}"));
            }
        }
        let mut picked: Vec<Sample> = Vec::new();
        for (pool, control) in [(&differ, false), (&agree, true)] {
            for i in spread(pool, samples) {
                picked.push(Sample {
                    index: i,
                    control,
                    uncorrected: Value::from_channel(first[4 * i]),
                    corrected: Value::from_channel(second[4 * i]),
                });
            }
        }

        // The oracle, at each pixel's centre and a ±STENCIL_PX stencil — the shader's own mapping:
        // c = centre + (x + ½ − w/2, h/2 − (y + ½)) · (span_mantissa / size) · 2^delta_exp.
        let p = self.viewport.precision + 64;
        let step_x = fractadyne_core::FloatExp::new(req.span_mantissa.x / w as f64, req.delta_exp);
        let step_y = fractadyne_core::FloatExp::new(req.span_mantissa.y / h as f64, req.delta_exp);
        let (cx0, cy0) = (&self.viewport.center_x, &self.viewport.center_y);
        let max_iter = req.max_iter;
        let formula = req.formula;
        let oracle_at = |i: usize, dx: f64, dy: f64| {
            let (x, y) = ((i % w) as f64, (i / w) as f64);
            let cx = fractadyne_core::add_floatexp(cx0, step_x.mul_f64(x + 0.5 + dx - w as f64 * 0.5), p);
            let cy = fractadyne_core::add_floatexp(cy0, step_y.mul_f64(h as f64 * 0.5 - (y + 0.5 + dy)), p);
            // Mandelbrot keeps the plain z²+c oracle; every other family iterates its own formula.
            if formula == fractadyne_core::formula::MANDELBROT {
                fractadyne_core::naive_dwell_bf(&cx, &cy, max_iter, 256.0 * 256.0, p)
            } else {
                fractadyne_core::formula_dwell(&cx, &cy, formula, max_iter, 256.0 * 256.0, p)
            }
        };
        let t2 = std::time::Instant::now();
        let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
        let results: Vec<(Value, bool)> = std::thread::scope(|s| {
            let chunk = picked.len().div_ceil(threads).max(1);
            let handles: Vec<_> = picked
                .chunks(chunk)
                .map(|part| {
                    let oracle_at = &oracle_at;
                    s.spawn(move || {
                        part.iter()
                            .map(|smp| {
                                let d = STENCIL_PX;
                                let answers: Vec<_> = [(0.0, 0.0), (d, 0.0), (-d, 0.0), (0.0, d), (0.0, -d)]
                                    .iter()
                                    .map(|&(dx, dy)| oracle_at(smp.index, dx, dy))
                                    .collect();
                                let v = match answers[0] {
                                    Some((_, sm)) => Value::Escaped(sm),
                                    None => Value::Interior,
                                };
                                (v, stable(&answers))
                            })
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            handles.into_iter().flat_map(|h| h.join().unwrap_or_default()).collect()
        });
        println!(
            "  oracle: {}-bit arbitrary precision, {} pixels × 5 points in {:.1} s",
            p,
            picked.len(),
            t2.elapsed().as_secs_f64()
        );

        // The table and the tallies.
        use std::collections::HashMap;
        let mut tally: [HashMap<Verdict, usize>; 2] = [HashMap::new(), HashMap::new()];
        println!("\n  kind     pixel (x,y)     {la:<14} {lb:<14} oracle         verdict");
        for (smp, &(oracle, st)) in picked.iter().zip(&results) {
            let v = verdict(oracle, st, smp.uncorrected, smp.corrected);
            *tally[smp.control as usize].entry(v).or_default() += 1;
            // `Verdict` is named for the correction audit: the second render is `CorrectedRight`.
            let named = match v {
                Verdict::CorrectedRight => format!("{lb} right"),
                Verdict::UncorrectedRight => format!("{la} right"),
                other => format!("{other:?}"),
            };
            println!(
                "  {:<8} ({:>5},{:>5})   {:<14} {:<14} {:<14} {named}",
                if smp.control { "control" } else { "differs" },
                smp.index % w,
                smp.index / w,
                smp.uncorrected.to_string(),
                smp.corrected.to_string(),
                oracle.to_string(),
            );
        }
        let get = |t: &HashMap<Verdict, usize>, v: Verdict| t.get(&v).copied().unwrap_or(0);
        let (d, c) = (&tally[0], &tally[1]);
        let controls_scored = c.values().sum::<usize>() - get(c, Verdict::Sensitive);
        let controls_ok = get(c, Verdict::BothRight);
        println!(
            "\n  controls: {controls_ok} of {controls_scored} scorable agree with the oracle ({} sensitive)",
            get(c, Verdict::Sensitive)
        );
        // ⚠The control decides whether anything below means something.
        if controls_scored == 0 || (controls_ok as f64) < 0.9 * controls_scored as f64 {
            return Err(AppError::Message(format!(
                "audit: INVALID — the oracle agrees with both renders at only {controls_ok} of \
                 {controls_scored} control pixels, so the pixel→c mapping or the oracle is wrong; no verdict"
            )));
        }
        Ok(format!(
            "audit: of {} differing pixels sampled — {lb} right {}, {la} right {}, \
             neither {}, both {}, sensitive {}",
            d.values().sum::<usize>(),
            get(d, Verdict::CorrectedRight),
            get(d, Verdict::UncorrectedRight),
            get(d, Verdict::Neither),
            get(d, Verdict::BothRight),
            get(d, Verdict::Sensitive),
        ))
    }
}

#[cfg(test)]
#[path = "glitchaudit_tests.rs"]
mod tests;
