//! `--gputest` — verify the shader's df32/floatexp primitives against CPU oracles, per machine.
//!
//! The WGSL error-free transforms assume round-to-nearest with a FUSED fma; everything deep is
//! built on them, and until now they were verified only end-to-end by goldens blessed on one
//! RTX 3080. This harness runs `fs_gputest` (which calls the SAME shader functions the renderer
//! uses) on hash-derived inputs and checks every op family against an f64 or exact-EFT oracle —
//! so a driver/hardware where the assumption fails (fast-math fma contraction, flushed
//! denormals, wrong rounding) is caught as "two_prod residual wrong on THIS machine", not as an
//! unexplained black frame from a stranger's GPU. Row 9 (Julia-form accumulation, per-pixel
//! identity only in z₀) is the regression canary for the 2026-08-13 Julia precision class.
//!
//! Input construction uses integer hashing + bitcast only (no floating arithmetic), so this
//! module reproduces the shader's inputs bit-exactly.

use eframe::wgpu;

/// Mirrors WGSL `gt_hash` (wrapping u32 arithmetic — identical on both sides).
fn gt_hash(x0: u32) -> u32 {
    let mut x = x0.wrapping_mul(747796405).wrapping_add(2891336453);
    x = ((x >> ((x >> 28) + 4)) ^ x).wrapping_mul(277803737);
    (x >> 22) ^ x
}
/// Mirrors WGSL `gt_f32`: sign|exponent|mantissa from hash bits, exponent in [emin, emin+espan).
fn gt_f32(seed: u32, emin: i32, espan: u32) -> f32 {
    let mant = gt_hash(seed) & 0x007F_FFFF;
    let eb = (127 + emin + (gt_hash(seed ^ 0x9E37_79B9) % espan) as i32) as u32;
    let sign = (gt_hash(seed ^ 0x85EB_CA6B) & 1) << 31;
    f32::from_bits(sign | (eb << 23) | mant)
}
/// Mirrors WGSL `gt_df`: a NORMALIZED pair, `lo`'s exponent derived from `hi`'s actual exponent
/// so `|lo| <= ulp(hi)/2` (integer/bit math only — see the WGSL for why no EFT is used here).
fn gt_df(seed: u32, emin: i32, espan: u32) -> (f32, f32) {
    let hi = gt_f32(seed, emin, espan);
    let he = ((hi.to_bits() >> 23) & 0xFF) as i32 - 127;
    (hi, gt_f32(seed ^ 0xDEAD_BEEF, he - 25, 1))
}
fn df64(d: (f32, f32)) -> f64 {
    d.0 as f64 + d.1 as f64
}

struct OpCheck {
    /// The row's ASCII name in the `[FAIL] <backend>: <id>` lines the battery holds against a
    /// card's declared failures (scripts/gpu-validate-expected.json). The battery matches a
    /// declared failure as a PREFIX, so no id may begin with another (`two_sum` would also cover a
    /// `two_sum_armored`): `ids_are_prefix_free` holds them to that.
    id: &'static str,
    name: &'static str,
    max_err: f64,
    tol: f64,
    fails: u32,
    detail: String,
    /// For a failing decision row (13–17): what this backend's arithmetic was found to be, appended
    /// to its `[FAIL]` line, so a change of arithmetic is a new failure rather than the same one.
    tag: String,
}

/// One op row's verdict against its oracle. `err` is relative to the oracle magnitude (or the
/// input magnitude for cancellation-prone adds); EFT rows demand exactness and use fails only.
fn check_rows(w: u32, h: u32, px: &[f32]) -> Vec<OpCheck> {
    let texel = |ix: u32, op: u32| -> [f32; 4] {
        let base = ((op * w + ix) * 4) as usize;
        [px[base], px[base + 1], px[base + 2], px[base + 3]]
    };
    let mut out = Vec::new();
    let _ = h;

    // Rows 0/1 — the error-free transforms — and row 10, two_sum again but with bitcast armor
    // (see the WGSL `gt_opaque`): if plain two_sum fails while the armored one is exact, the
    // machine's compiler is REASSOCIATING float expressions (folding the residual away); if
    // both fail, the rounding itself is off. EXACT by theorem under round-to-nearest + real
    // fma: s must be fl(a+b) / fl(a·b) and e the exact residual (always representable in f32).
    for (op, id, name) in [
        (0u32, "two_sum", "two_sum (exact EFT)"),
        (1u32, "two_prod", "two_prod (exact EFT — fma canary)"),
        (10u32, "armored_two_sum", "two_sum, bitcast-armored (reassociation discriminator)"),
        (11u32, "quick_two_sum", "quick_two_sum (exact EFT — df_mul/df_div depend on it)"),
    ] {
        let mut fails = 0u32;
        let mut detail = String::new();
        for ix in 0..w {
            let s = ix + 100003 * op;
            let (mut a, mut b) = (gt_f32(s * 2 + 1, -8, 16), gt_f32(s * 2 + 2, -8, 16));
            if op == 11 && a.abs() < b.abs() {
                std::mem::swap(&mut a, &mut b); // quick_two_sum requires |a| >= |b|
            }
            let got = texel(ix, op);
            let (es, ee) = if op == 1 {
                let fp = a * b;
                (fp, (a as f64) * (b as f64) - fp as f64)
            } else {
                let fs = a + b;
                (fs, (a as f64 + b as f64) - fs as f64)
            };
            if got[0] != es || got[1] as f64 != ee {
                fails += 1;
                if detail.is_empty() {
                    detail = format!(
                        "first: a={a:e} b={b:e} got=({:e},{:e}) want=({es:e},{ee:e})",
                        got[0], got[1]
                    );
                }
            }
        }
        out.push(OpCheck { id, name, max_err: 0.0, tol: 0.0, fails, detail, tag: String::new() });
    }

    // Generic relative-error rows.
    struct Rel {
        op: u32,
        id: &'static str,
        name: &'static str,
        tol: f64,
    }
    for r in [
        Rel { op: 2, id: "df_add", name: "df_add", tol: 2f64.powi(-42) },
        Rel { op: 3, id: "df_mul", name: "df_mul", tol: 2f64.powi(-42) },
        Rel { op: 4, id: "df_div", name: "df_div", tol: 2f64.powi(-40) },
        Rel { op: 5, id: "c_sqr", name: "c_sqr (complex square)", tol: 2f64.powi(-40) },
        Rel { op: 6, id: "fe_mul", name: "fe_mul (floatexp)", tol: 2f64.powi(-38) },
        Rel { op: 7, id: "fe_add", name: "fe_add (floatexp align/renorm)", tol: 2f64.powi(-38) },
        Rel { op: 8, id: "accum_mandelbrot", name: "64-step df32 accumulation (Mandelbrot form)", tol: 2f64.powi(-33) },
        Rel { op: 9, id: "accum_julia", name: "64-step df32 accumulation (Julia form, z0-carried)", tol: 2f64.powi(-33) },
    ] {
        let mut max_err = 0f64;
        let mut fails = 0u32;
        let mut detail = String::new();
        for ix in 0..w {
            let s = ix + 100003 * r.op;
            let ar = gt_df(s * 8 + 3, -4, 8);
            let ai = gt_df(s * 8 + 4, -4, 8);
            let br = gt_df(s * 8 + 5, -4, 8);
            let bi = gt_df(s * 8 + 6, -4, 8);
            let got = texel(ix, r.op);
            // (oracle value(s), got value(s), magnitude scale for the relative error)
            let (checks, scale): (Vec<(f64, f64)>, f64) = match r.op {
                2 => {
                    let v = df64(ar) + df64(br);
                    (vec![(v, got[0] as f64 + got[1] as f64)], df64(ar).abs() + df64(br).abs())
                }
                3 => {
                    let v = df64(ar) * df64(br);
                    (vec![(v, got[0] as f64 + got[1] as f64)], v.abs())
                }
                4 => {
                    let v = df64(ar) / df64(br);
                    (vec![(v, got[0] as f64 + got[1] as f64)], v.abs())
                }
                5 => {
                    let (re, im) = (df64(ar), df64(ai));
                    let (vr, vi) = (re * re - im * im, 2.0 * re * im);
                    let mag = re * re + im * im;
                    (
                        vec![
                            (vr, got[0] as f64 + got[1] as f64),
                            (vi, got[2] as f64 + got[3] as f64),
                        ],
                        mag.max(1e-30),
                    )
                }
                6 => {
                    let ea = (gt_hash(s ^ 17) % 200) as i32 - 100;
                    let eb2 = (gt_hash(s ^ 23) % 200) as i32 - 100;
                    let (are, aim) = (df64(ar), df64(ai));
                    let (bre, bim) = (df64(br), df64(bi));
                    let sc = 2f64.powi(ea + eb2);
                    let vr = (are * bre - aim * bim) * sc;
                    let vi = (are * bim + aim * bre) * sc;
                    let e_out = got[2] as i32;
                    let gre = (got[0] as f64 + got[1] as f64) * 2f64.powi(e_out);
                    let gim = got[3] as f64 * 2f64.powi(e_out);
                    let mag = (are.abs() + aim.abs()) * (bre.abs() + bim.abs()) * sc;
                    // im has only the hi limb in the output — looser check for it.
                    (
                        vec![(vr, gre), (vi * 2f64.powi(-24), gim * 2f64.powi(-24))],
                        mag.max(1e-300),
                    )
                }
                7 => {
                    let ea = (gt_hash(s ^ 31) % 80) as i32 - 40;
                    let (are, aim) = (df64(ar), df64(ai));
                    let (bre, bim) = (df64(br), df64(bi));
                    let sa = 2f64.powi(ea);
                    let vr = are * sa + bre;
                    let vi = aim * sa + bim;
                    let e_out = got[2] as i32;
                    let gre = (got[0] as f64 + got[1] as f64) * 2f64.powi(e_out);
                    let gim = got[3] as f64 * 2f64.powi(e_out);
                    let mag = are.abs() * sa + bre.abs() + aim.abs() * sa + bim.abs();
                    (
                        vec![(vr, gre), (vi * 2f64.powi(-24), gim * 2f64.powi(-24))],
                        mag.max(1e-300),
                    )
                }
                8 | 9 => {
                    // Mirror the contracted recurrence z ← z²·2⁻¹ + c in f64.
                    let cr8 = gt_df(s * 8 + 3, -3, 3);
                    let ci8 = gt_df(s * 8 + 4, -3, 3);
                    let (cre, cim, mut zr, mut zi) = if r.op == 8 {
                        (df64(cr8), df64(ci8), 0f64, 0f64)
                    } else {
                        (-0.7436f32 as f64, 0.1318f32 as f64, df64(cr8), df64(ci8))
                    };
                    for _ in 0..64 {
                        let (qr, qi) = (zr * zr - zi * zi, 2.0 * zr * zi);
                        zr = qr * 0.5 + cre;
                        zi = qi * 0.5 + cim;
                    }
                    let mag = zr.abs().max(zi.abs()).max(1e-6);
                    (
                        vec![
                            (zr, got[0] as f64 + got[1] as f64),
                            (zi, got[2] as f64 + got[3] as f64),
                        ],
                        mag,
                    )
                }
                _ => (vec![], 1.0),
            };
            for (want, got_v) in checks {
                let err = (got_v - want).abs() / scale.max(1e-300);
                if err > max_err {
                    max_err = err;
                    if err > r.tol {
                        detail = format!("worst ix={ix}: want {want:e} got {got_v:e}");
                    }
                }
                if err > r.tol {
                    fails += 1;
                }
            }
        }
        out.push(OpCheck { id: r.id, name: r.name, max_err, tol: r.tol, fails, detail, tag: String::new() });
    }

    // Row 12 — df_mul dissected. Not a tolerance check: it reports WHICH intermediate came back
    // zero when it should not have, which is the difference between "multiplication is broken"
    // and "this one line was optimized away". A stack that keeps all three is arithmetically
    // sound here; a stack that zeroes any one of them tells us precisely what its compiler did.
    {
        let (mut py_zero, mut e_zero, mut lo_zero, mut nonzero_expected) = (0u32, 0u32, 0u32, 0u32);
        for ix in 0..w {
            let s = ix + 100003 * 12;
            // Same operands the shader's row 12 uses: `ar` (seed +3) and `br` (seed +5).
            let (a, b) = (gt_df(s * 8 + 3, -4, 8), gt_df(s * 8 + 5, -4, 8));
            let got = texel(ix, 12);
            // Exact f32 mirror (Rust neither reassociates nor auto-contracts).
            let p_hi = a.0 * b.0;
            let p_lo = a.0.mul_add(b.0, -p_hi);
            let e = p_lo + (a.0 * b.1 + a.1 * b.0);
            let s_hi = p_hi + e;
            let r_lo = e - (s_hi - p_hi);
            if p_lo != 0.0 || e != 0.0 || r_lo != 0.0 {
                nonzero_expected += 1;
            }
            if p_lo != 0.0 && got[1] == 0.0 {
                py_zero += 1;
            }
            if e != 0.0 && got[2] == 0.0 {
                e_zero += 1;
            }
            if r_lo != 0.0 && got[3] == 0.0 {
                lo_zero += 1;
            }
        }
        let fails = py_zero + e_zero + lo_zero;
        let detail = if fails == 0 {
            String::new()
        } else {
            format!(
                "of {nonzero_expected} sets expecting non-zero: two_prod residual p.y zeroed \
                 {py_zero}x, cross-term sum e zeroed {e_zero}x, quick_two_sum residual zeroed \
                 {lo_zero}x  <- the non-zero count names the folded step"
            )
        };
        out.push(OpCheck {
            id: "dissected_df_mul",
            name: "df_mul dissected (which intermediate got folded?)",
            max_err: 0.0,
            tol: 0.0,
            fails,
            detail,
            tag: String::new(),
        });
    }
    out.extend(decision_rows(&texel));
    out
}

// ---- Rows 13–17: the counters' decisions (U26) ------------------------------------------------
//
// The rebase and BLA-skip counters count decisions, and a decision is shader arithmetic: a sum of
// two squares a compiler may fuse into a multiply-add either way, a sqrt or log2 that need not be
// correctly rounded, subnormals flushed or kept. Each row runs the renderer's own predicate on
// near-tie inputs and names every modeled arithmetic that reproduces all 1,024 decisions. The
// first model is the reference (IEEE single precision, unfused, subnormals flushed as GPUs do);
// matching it is a PASS. Any other outcome is a FAIL whose tag names what the backend does, so a
// card declares its arithmetic in gpu-validate-expected.json and a change of it is a NEW failure.
// A decision whose exact answer is not in doubt (far from the tie) must match exactly whatever
// the arithmetic: getting one of those wrong is reported as WRONG.

/// How `x*x + y*y` is evaluated: rounded per operation, or fused into one multiply-add.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SumSq {
    Plain,
    FmaX,
    FmaY,
}

impl SumSq {
    const ALL: [SumSq; 3] = [SumSq::Plain, SumSq::FmaX, SumSq::FmaY];
    fn text(self) -> &'static str {
        match self {
            SumSq::Plain => "x*x+y*y",
            SumSq::FmaX => "fma(x,x,y*y)",
            SumSq::FmaY => "fma(y,y,x*x)",
        }
    }
}

/// `x*x + y*y` in f32 as `v` evaluates it, results below f32's normal range flushed to zero
/// when `ftz` (as most GPUs do).
fn sumsq(x: f32, y: f32, v: SumSq, ftz: bool) -> f32 {
    let f = |a: f32| if ftz && a.abs() < f32::MIN_POSITIVE { 0.0 } else { a };
    match v {
        SumSq::Plain => f(f(x * x) + f(y * y)),
        SumSq::FmaX => f(x.mul_add(x, f(y * y))),
        SumSq::FmaY => f(y.mul_add(y, f(x * x))),
    }
}

/// Mirrors WGSL `gt_rebase_case` (see it for the classes): (z.x, z.y, δz.x, δz.y) and the class.
fn gt_rebase_case(t: u32) -> ([f32; 4], u32) {
    let cls = gt_hash(t ^ 0x51ED_270B) % 8;
    let emin = if cls == 1 { -72 } else { -8 };
    let a = gt_f32(t.wrapping_mul(4).wrapping_add(1), emin, 8);
    let b = gt_f32(t.wrapping_mul(4).wrapping_add(2), emin, 8);
    if cls == 0 {
        return ([a, b, gt_f32(t.wrapping_mul(4).wrapping_add(3), -8, 8), gt_f32(t.wrapping_mul(4).wrapping_add(4), -8, 8)], cls);
    }
    let h = gt_hash(t ^ 0x2545_F491);
    ([a, b, f32::from_bits(b.to_bits() ^ (h & 15)), f32::from_bits(a.to_bits() ^ ((h >> 4) & 15))], cls)
}

/// `fe_norm`'s mantissa scaling for a pair with zero low limbs: by `2^-floor(log2(max(|x|, |y|)))`
/// (exact), returning the scaled pair and the shift.
fn fe_norm2(x: f32, y: f32) -> (f32, f32, i32) {
    let shift = ((x.abs().max(y.abs()).to_bits() >> 23) & 0xFF) as i32 - 127;
    let s = 2f32.powi(-shift);
    (x * s, y * s, shift)
}

/// The exact magnitude of (x, y), in f64 (the squares and their sum are exact for these inputs).
fn mag64(x: f32, y: f32) -> f64 {
    ((x as f64) * (x as f64) + (y as f64) * (y as f64)).sqrt()
}

/// One input of a decision row: its decision under each model, the exact answer, and whether
/// that answer is beyond doubt (far enough from the tie that no plausible arithmetic flips it).
struct Case {
    model: Vec<bool>,
    exact: bool,
    decisive: bool,
}

/// Grade a decision row: `cases` in texel order (four per texel), `names` the models (index 0 the
/// reference), `gpu` the GPU's decisions.
fn grade_decisions(id: &'static str, name: &'static str, names: &[String], cases: &[Case], gpu: &[bool]) -> OpCheck {
    let wrong = cases.iter().zip(gpu).filter(|(c, &g)| c.decisive && g != c.exact).count();
    let first_wrong = cases.iter().zip(gpu).position(|(c, &g)| c.decisive && g != c.exact);
    let mismatch: Vec<usize> =
        (0..names.len()).map(|m| cases.iter().zip(gpu).filter(|(c, &g)| c.model[m] != g).count()).collect();
    let matching: Vec<&str> = (0..names.len()).filter(|&m| mismatch[m] == 0).map(|m| names[m].as_str()).collect();
    let near = cases.iter().filter(|c| !c.decisive).count();
    let (fails, tag) = if wrong > 0 {
        (wrong as u32, format!("WRONG on {wrong} decisive inputs"))
    } else if mismatch[0] == 0 {
        (0, String::new())
    } else if !matching.is_empty() {
        (mismatch[0] as u32, format!("decides like {}", matching.join(" | ")))
    } else {
        (mismatch[0] as u32, format!("matches no modeled arithmetic: {} of {} differ from {}", mismatch[0], cases.len(), names[0]))
    };
    let detail = format!(
        "{} decisions ({near} near ties); differ from each model: {}{}{}",
        cases.len(),
        names.iter().zip(&mismatch).map(|(n, k)| format!("{n} {k}")).collect::<Vec<_>>().join(", "),
        if tag.is_empty() { String::new() } else { format!("  => {tag}") },
        first_wrong.map(|i| format!(" (first: input {i}, texel {} slot {})", i / 4, i % 4)).unwrap_or_default()
    );
    OpCheck { id, name, max_err: 0.0, tol: 0.0, fails, detail, tag }
}

/// The `t` of every decision in row `op`, in texel order (four per texel), as the WGSL forms it.
fn row_ts(op: u32) -> Vec<u32> {
    (0..GT_W).flat_map(|ix| (0..4u32).map(move |k| (ix + 100003 * op) * 4 + k)).collect()
}

/// Columns per row of `fs_gputest` (`gputest` in the GPU crate).
const GT_W: u32 = 256;

/// The decision rows, 13 to 17.
const DECISION_ROWS: std::ops::RangeInclusive<u32> = 13..=17;

/// Row `op`'s id, name, model names (the reference first) and cases, GPU-free.
fn row_cases(op: u32) -> (&'static str, &'static str, Vec<String>, Vec<Case>) {
    let ftz_text = |ftz: bool| if ftz { "subnormals flushed" } else { "subnormals kept" };
    let ts = row_ts(op);
    match op {
        // `rebase_df32(dot(zf, zf), zf, dz)`. The dot and the explicit δz sum may each be fused
        // either way; the lift's two dots go as the first.
        13 => {
            let mut names = Vec::new();
            let mut models = Vec::new();
            for ftz in [true, false] {
                for vz in SumSq::ALL {
                    for vd in SumSq::ALL {
                        names.push(format!("dot {}, dz {}, {}", vz.text(), vd.text(), ftz_text(ftz)));
                        models.push((vz, vd, ftz));
                    }
                }
            }
            let cases = ts
                .iter()
                .map(|&t| {
                    let ([zx, zy, dx, dy], cls) = gt_rebase_case(t);
                    let decide = |(vz, vd, ftz): (SumSq, SumSq, bool)| {
                        let (z2, d2) = (sumsq(zx, zy, vz, ftz), sumsq(dx, dy, vd, ftz));
                        if z2 == 0.0 && d2 == 0.0 {
                            let l = 2f32.powi(64);
                            sumsq(zx * l, zy * l, vz, ftz) < sumsq(dx * l, dy * l, vz, ftz)
                        } else {
                            z2 < d2
                        }
                    };
                    let (ez, ed) = (mag64(zx, zy), mag64(dx, dy));
                    // Subnormal squares keep only a few bits: the tiny class is beyond doubt only far out.
                    let margin = if cls == 1 { 2f64.powi(-6) } else { 2f64.powi(-16) };
                    Case {
                        model: models.iter().map(|&m| decide(m)).collect(),
                        exact: ez < ed,
                        decisive: (ez - ed).abs() > margin * ez.max(ed),
                    }
                })
                .collect();
            ("rebase_df32", "rebase test, df32 path (mode 0)", names, cases)
        }
        // `sf_lt(fe_abs_sf(z), fe_abs_sf(dz))` on normalized mantissas (squares never subnormal).
        14 => {
            let names = SumSq::ALL.iter().map(|v| format!("sqrt({}) correctly rounded", v.text())).collect();
            let cases = ts
                .iter()
                .map(|&t| {
                    let ([zx, zy, dx, dy], _) = gt_rebase_case(t);
                    let (nzx, nzy, sz) = fe_norm2(zx, zy);
                    let (ndx, ndy, sd) = fe_norm2(dx, dy);
                    let decide = |v: SumSq| {
                        let mz = sumsq(nzx, nzy, v, true).sqrt() as f64 * 2f64.powi(sz);
                        let md = sumsq(ndx, ndy, v, true).sqrt() as f64 * 2f64.powi(sd);
                        mz < md
                    };
                    let (ez, ed) = (mag64(zx, zy), mag64(dx, dy));
                    Case {
                        model: SumSq::ALL.iter().map(|&v| decide(v)).collect(),
                        exact: ez < ed,
                        decisive: (ez - ed).abs() > 2f64.powi(-16) * ez.max(ed),
                    }
                })
                .collect();
            ("rebase_floatexp", "rebase test, floatexp (mode 2)", names, cases)
        }
        // `tail_rebase`: sqrt(dot(a, a)) < sqrt(dot(b, b)), one dot form for both sides.
        15 => {
            let mut names = Vec::new();
            let mut models = Vec::new();
            for ftz in [true, false] {
                for v in SumSq::ALL {
                    names.push(format!("sqrt(dot {}) correctly rounded, {}", v.text(), ftz_text(ftz)));
                    models.push((v, ftz));
                }
            }
            let cases = ts
                .iter()
                .map(|&t| {
                    let ([zx, zy, dx, dy], cls) = gt_rebase_case(t);
                    let decide = |(v, ftz): (SumSq, bool)| sumsq(zx, zy, v, ftz).sqrt() < sumsq(dx, dy, v, ftz).sqrt();
                    let (ez, ed) = (mag64(zx, zy), mag64(dx, dy));
                    Case {
                        model: models.iter().map(|&m| decide(m)).collect(),
                        exact: ez < ed,
                        // A flushed square reads 0 < 0 whatever the margin: the tiny class is never decisive.
                        decisive: cls != 1 && (ez - ed).abs() > 2f64.powi(-16) * ez.max(ed),
                    }
                })
                .collect();
            ("rebase_tail", "rebase test, df32 tail (mode 2)", names, cases)
        }
        // `sf_lt(fe_abs_sf(dz), sf_norm(r, e))`, the BLA walk's validity test.
        16 => {
            let names = SumSq::ALL.iter().map(|v| format!("sqrt({}) correctly rounded", v.text())).collect();
            let cases = ts
                .iter()
                .map(|&t| {
                    let x = gt_f32(t.wrapping_mul(4).wrapping_add(1), -8, 8);
                    let ex = ((x.to_bits() >> 23) & 0xFF) as i32 - 127;
                    let y = if gt_hash(t ^ 0x1B87_3593) & 1 == 1 {
                        gt_f32(t.wrapping_mul(4).wrapping_add(2), ex - 13, 2)
                    } else {
                        0.0
                    };
                    let h = gt_hash(t ^ 0xCC9E_2D51);
                    let base = x.to_bits() & 0x7FFF_FFFF;
                    let rbits = if (h >> 8) % 8 == 0 {
                        if (h >> 11) & 1 == 1 { base - 0x80_0000 } else { base + 0x80_0000 }
                    } else {
                        (base + h % 9).wrapping_sub(4)
                    };
                    let r = f32::from_bits(rbits);
                    let (cx, cy) = if (h >> 12) & 1 == 1 { (y, x) } else { (x, y) };
                    let (nx, ny, sh) = fe_norm2(cx, cy);
                    let decide = |v: SumSq| (sumsq(nx, ny, v, true).sqrt() as f64) * 2f64.powi(sh) < r as f64;
                    let e = mag64(x, y);
                    Case {
                        model: SumSq::ALL.iter().map(|&v| decide(v)).collect(),
                        exact: e < r as f64,
                        decisive: (e - r as f64).abs() > 2f64.powi(-16) * e,
                    }
                })
                .collect();
            ("bla_radius", "BLA validity test (mode 2)", names, cases)
        }
        // `tail_no_skip`: log2(max|dz|) > log2(r) + k + 0.01, as written (left to right) or with
        // the constant folded into k first; log2 correctly rounded.
        _ => {
            let names = vec![
                "log2 correctly rounded, (log2(r)+k)+0.01".to_string(),
                "log2 correctly rounded, log2(r)+(k+0.01)".to_string(),
            ];
            let cases = ts
                .iter()
                .map(|&t| {
                    let j = (gt_hash(t ^ 0x7FEB_352D) % 60) as i32 - 30;
                    let kk = (gt_hash(t ^ 0x846C_A68B) % 41) as i32 - 20;
                    let jit = gt_hash(t ^ 0x2C1B_3C6D) % 97;
                    let sign = (gt_hash(t ^ 0x1656_67B1) & 1) << 31;
                    let m = f32::from_bits(sign | ((((127 + j + kk) as u32) << 23) + 58300 + jit));
                    let none = gt_hash(t ^ 0x9E37_79B1) % 16 == 0;
                    // The larger component is always |m| (the other sits a binade below it).
                    let lm = (m.abs() as f64).log2() as f32;
                    let lr = j as f32; // log2 of 2^j, exact
                    let (k32, c) = (kk as f32, 0.01f32);
                    let thr = [(lr + k32) + c, lr + (k32 + c)];
                    let exact_thr = (j + kk) as f64 + c as f64;
                    // An ulp of the LARGEST intermediate, log2(r) = j: one ulp of error there moves
                    // the threshold by f32::EPSILON·|j| (measured: input 557, j = -18, on all three
                    // RTX 3080 backends, was 1.6e-6 past a threshold its 0.01-sized ulp called safe).
                    let ulp = f32::EPSILON as f64 * (j.abs().max((j + kk).abs()).max(1)) as f64;
                    Case {
                        model: thr.iter().map(|&th| none || lm > th).collect(),
                        exact: none || (m.abs() as f64).log2() > exact_thr,
                        decisive: none || ((m.abs() as f64).log2() - exact_thr).abs() > 8.0 * ulp,
                    }
                })
                .collect();
            ("tail_bla_exit", "BLA exit test, df32 tail (mode 2)", names, cases)
        }
    }
}

fn decision_rows(texel: &dyn Fn(u32, u32) -> [f32; 4]) -> Vec<OpCheck> {
    DECISION_ROWS
        .map(|op| {
            let (id, name, names, cases) = row_cases(op);
            let gpu: Vec<bool> = (0..GT_W).flat_map(|ix| texel(ix, op).map(|v| v == 1.0)).collect();
            grade_decisions(id, name, &names, &cases, &gpu)
        })
        .collect()
}

#[cfg(test)]
#[path = "../tests/unit/gputest/decision_row_tests.rs"]
mod decision_row_tests;

/// A tee: every line goes to stdout (so a terminal and the Diagnostics dialog's stream see it as
/// it happens) and, when a report file was asked for, is also retained VERBATIM for that file —
/// the dialog trims each streamed line, which would wreck the column alignment, so the on-disk
/// copy has to come from here rather than from captured stdout.
struct Sink {
    lines: Vec<String>,
    save: bool,
}
impl Sink {
    fn new(save: bool) -> Self {
        Sink { lines: Vec::new(), save }
    }
    fn line(&mut self, s: String) {
        println!("{s}");
        if self.save {
            self.lines.push(s);
        }
    }
}

/// Grade one device's results and print the verdict table, then one `[FAIL] <label>: <id>` line
/// per failing row: the lines scripts/gpu-validate.ps1 holds against the card's declared
/// failures. Returns failing op-family count.
fn report(sink: &mut Sink, label: &str, w: u32, h: u32, px: &[f32]) -> usize {
    sink.line(format!("  {:<52} {:>12} {:>10} {:>6}", "op", "max rel err", "tolerance", ""));
    let checks = check_rows(w, h, px);
    let mut failed = 0usize;
    for c in &checks {
        let ok = c.fails == 0;
        if !ok {
            failed += 1;
        }
        let verdict = if ok { "PASS" } else { "FAIL" };
        if c.tol == 0.0 {
            // A decision row (13–17) grades agreement with a model, not exactness of a value.
            let decision = c.detail.contains("decisions (");
            sink.line(format!(
                "  {:<52} {:>12} {:>10} {:>6}",
                c.name,
                match (ok, decision) {
                    (true, false) => "exact".to_string(),
                    (true, true) => "as IEEE".to_string(),
                    (false, false) => format!("{} wrong", c.fails),
                    (false, true) => format!("{} differ", c.fails),
                },
                if decision { "IEEE f32" } else { "exact" },
                verdict
            ));
        } else {
            sink.line(format!("  {:<52} {:>12.2e} {:>10.1e} {:>6}", c.name, c.max_err, c.tol, verdict));
        }
        if (!ok || c.detail.contains("decisions (")) && !c.detail.is_empty() {
            sink.line(format!("      {}", c.detail));
        }
    }
    for c in checks.iter().filter(|c| c.fails > 0) {
        if c.tag.is_empty() {
            sink.line(format!("  [FAIL] {label}: {}", c.id));
        } else {
            sink.line(format!("  [FAIL] {label}: {} - {}", c.id, c.tag));
        }
    }
    if failed == 0 {
        sink.line(format!("  All {} op families within tolerance.", checks.len()));
    } else {
        sink.line(format!(
            "  {failed} op FAMILY(IES) FAILED — the shader's extended-precision arithmetic does \
             not hold on this stack."
        ));
    }
    failed
}

/// `--gputest`: run the shader's primitive self-test on EVERY backend this machine offers.
///
/// Headless by construction — the test renders to an offscreen texture, so it needs no window and
/// no surface. That matters twice over: it runs on a build server or over SSH, and it can reach
/// backends the windowed path cannot (DX12 surface creation fails under this app's eframe setup,
/// which is why the 2026-08-13 folded-EFT investigation could not get a DX12 data point).
///
/// Sweeping rather than testing one device is the whole point: the arithmetic is compiled by the
/// BACKEND's shader compiler (DX12 → HLSL via FXC/DXC, Vulkan → SPIR-V, GL → GLSL), so "is df32
/// real on this machine" can have a different answer per backend, and the comparison localizes
/// the blame to a specific compiler instead of "the GPU".
///
/// Returns the number of backends that failed (0 = every available backend is sound).
pub(crate) fn run_gputest_sweep(out: Option<&std::path::Path>) -> usize {
    let candidates: [(wgpu::Backends, &str); 3] = [
        (wgpu::Backends::DX12, "DX12"),
        (wgpu::Backends::VULKAN, "Vulkan"),
        (wgpu::Backends::GL, "OpenGL"),
    ];
    let mut sink = Sink::new(out.is_some());
    // Which backends exist in THIS BINARY is a build-time fact (wgpu gates each behind a Cargo
    // feature), and it is not obvious from the outside: a backend that was never compiled in
    // looks exactly like a missing GPU at runtime. State it, so a report that omits a backend
    // says which of the two happened.
    let compiled = wgpu::Instance::enabled_backend_features();
    sink.line(format!(
        "Fractadyne GPU primitive self-test — {} · sweeping every available backend\n\
         Verifies the renderer's own df32/floatexp helpers against CPU oracles. The error-free\n\
         transforms (two_sum/two_prod) must be EXACT: they are what makes df32 more than f32.\n\
         Backends compiled into this binary: {compiled:?}\n",
        crate::version_string()
    ));
    let mut ran = 0usize;
    let mut failed = 0usize;
    for (backends, label) in candidates {
        if !compiled.contains(backends) {
            sink.line(format!("{label}: not compiled into this binary — skipped\n"));
            continue;
        }
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });
        let Some(adapter) = crate::gpu_worker::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })) else {
            sink.line(format!("{label}: no adapter — skipped\n"));
            continue;
        };
        let info = adapter.get_info();
        // Downlevel (GL) adapters cannot satisfy the default limits; ask for what they have.
        let limits = wgpu::Limits::default().using_resolution(adapter.limits());
        let dev = crate::gpu_worker::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("gputest.device"),
                required_features: wgpu::Features::empty(),
                required_limits: limits,
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ));
        let (device, queue) = match dev {
            Ok(d) => d,
            Err(e) => {
                sink.line(format!("{label}: device request failed ({e}) — skipped\n"));
                continue;
            }
        };
        sink.line(format!("── {label} · {} · driver: {} {}", info.name, info.driver, info.driver_info));
        ran += 1;
        match fractadyne_gpu::gputest(&device, &queue) {
            Ok((w, h, px)) => {
                if report(&mut sink, label, w, h, &px) > 0 {
                    failed += 1;
                }
            }
            Err(e) => {
                sink.line(format!("  GPU run failed: {e}"));
                sink.line(format!("  [FAIL] {label}: gpu run failed"));
                failed += 1;
            }
        }
        sink.line(String::new());
    }
    if ran == 0 {
        sink.line("No usable backend found — nothing tested.".to_string());
    } else if failed == 0 {
        sink.line(format!("{ran} backend(s) tested, all sound."));
    } else {
        sink.line(format!(
            "{failed} of {ran} backend(s) FAILED. Include this whole report in a bug report — a\n\
             failing two_sum means the shader compiler is folding the error-free transforms, and\n\
             every extended-precision path silently degrades to plain f32."
        ));
    }
    // Persist the verbatim report when asked (the Diagnostics dialog's "Open results" and the
    // attach-to-report path both point at this file). A write failure must not change the verdict
    // the caller exits on, so it is reported and swallowed.
    if let Some(path) = out {
        let body = sink.lines.join("\n") + "\n";
        if let Err(e) = std::fs::write(path, body) {
            eprintln!("gputest: could not write report to {}: {e}", path.display());
        }
    }
    if ran == 0 {
        return 1;
    }
    failed
}
