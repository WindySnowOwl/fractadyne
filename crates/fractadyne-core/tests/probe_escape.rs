//! TEMPORARY diagnostic (not committed): measure the actual escape-time distribution at a deep
//! view by full CPU perturbation (floatexp δz + Zhuoran rebasing against the center's orbit).
//! Answers "what iteration cap does this corpus location actually need?" with data.

use fractadyne_core::{CFloatExp, FloatExp};

#[test]
fn probe_escape_times() {
    let Ok(spec) = std::env::var("PROBE_ESCAPE") else {
        return;
    };
    // spec: "label|cx|cy|mag_log10|max_iter|prec"
    let parts: Vec<&str> = spec.split('|').collect();
    let (label, cx, cy) = (parts[0], parts[1], parts[2]);
    let mag_log10: f64 = parts[3].parse().unwrap();
    let max_iter: u32 = parts[4].parse().unwrap();
    let prec: usize = parts[5].parse().unwrap();

    let cx = fractadyne_core::parse_bf(cx).expect("cx");
    let cy = fractadyne_core::parse_bf(cy).expect("cy");
    let zero = fractadyne_core::BigFloat::from_f64(0.0, prec);
    let (orbit, len) = fractadyne_core::reference_orbit(&zero, &zero, &cx, &cy, 0, max_iter, prec);
    let len = len as usize;
    // Decode once to f64 pairs (sample_xy handles the extended-range dip markers).
    let zs: Vec<(f64, f64)> = orbit.iter().take(len).map(fractadyne_core::sample_xy).collect();
    println!("[esc] {label}: reference len={len} (escaped early = shorter than max_iter)");

    // View half-height = 1.5·10^-mag_log10 (3-unit-high view at mag 1). Probe pixels at a few
    // fractions of the half-span in 8 directions.
    let half = FloatExp::from_f64(1.5).mul_pow2(-(mag_log10 * std::f64::consts::LN_10 / std::f64::consts::LN_2));
    let dirs: [(f64, f64); 8] = [
        (1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0),
        (0.7, 0.7), (-0.7, 0.7), (0.7, -0.7), (-0.7, -0.7),
    ];
    for frac in [1.0_f64, 0.5, 0.1] {
        let mut escapes: Vec<i64> = Vec::new();
        for (dx, dy) in dirs {
            let dc = CFloatExp {
                re: half.mul_f64(dx * frac),
                im: half.mul_f64(dy * frac),
            };
            let mut dz = CFloatExp { re: FloatExp::ZERO, im: FloatExp::ZERO };
            let mut n: usize = 0; // reference index
            let mut esc: i64 = -1;
            for it in 0..max_iter as usize {
                let (zr, zi) = zs[n];
                let z2 = CFloatExp { re: FloatExp::from_f64(2.0 * zr), im: FloatExp::from_f64(2.0 * zi) };
                dz = z2 * dz + dz * dz + dc;
                n += 1;
                let (zr1, zi1) = zs[n.min(len - 1)];
                let zfull = CFloatExp {
                    re: FloatExp::from_f64(zr1) + dz.re,
                    im: FloatExp::from_f64(zi1) + dz.im,
                };
                let zmag = zfull.abs();
                if zmag.to_f64() > 256.0 {
                    esc = it as i64 + 1;
                    break;
                }
                // Zhuoran rebasing: |Z+dz| < |dz|, or the reference ran out.
                if zmag.lt(dz.abs()) || n + 1 >= len {
                    dz = zfull;
                    n = 0;
                }
            }
            escapes.push(esc);
        }
        println!("[esc] {label} frac={frac}: escapes = {escapes:?}");
    }
}

/// Dense-row oracle: escape times along a horizontal run of `npix` ADJACENT pixels centred on the
/// view centre, at the render's real per-pixel step. This is the CPU "truth" (f64 reference via
/// sample_xy, floatexp δz + Zhuoran rebasing — same algorithm as the shader). If the row varies
/// SMOOTHLY, the true escape field is smooth and any GPU speckle is a shader bug; if the row is
/// itself noisy, the perturbation algorithm is unstable here (a shared CPU/GPU accuracy problem).
/// spec: "label|cx|cy|mag_log10|max_iter|prec|npix|height_px"
#[test]
fn probe_escape_row() {
    let Ok(spec) = std::env::var("PROBE_ROW") else {
        return;
    };
    let parts: Vec<&str> = spec.split('|').collect();
    let (label, cx, cy) = (parts[0], parts[1], parts[2]);
    let mag_log10: f64 = parts[3].parse().unwrap();
    let max_iter: u32 = parts[4].parse().unwrap();
    let prec: usize = parts[5].parse().unwrap();
    let npix: usize = parts.get(6).and_then(|s| s.parse().ok()).unwrap_or(64);
    let height_px: f64 = parts.get(7).and_then(|s| s.parse().ok()).unwrap_or(180.0);

    let cx = fractadyne_core::parse_bf(cx).expect("cx");
    let cy = fractadyne_core::parse_bf(cy).expect("cy");
    let zero = fractadyne_core::BigFloat::from_f64(0.0, prec);
    let (orbit, len) = fractadyne_core::reference_orbit(&zero, &zero, &cx, &cy, 0, max_iter, prec);
    let len = len as usize;
    let zs: Vec<(f64, f64)> = orbit.iter().take(len).map(fractadyne_core::sample_xy).collect();
    println!("[row] {label}: reference len={len}");

    // Per-pixel c-space step = view_height / height_px; view half-height = 1.5·10^-mag_log10.
    let half = FloatExp::from_f64(1.5)
        .mul_pow2(-(mag_log10 * std::f64::consts::LN_10 / std::f64::consts::LN_2));
    let step = half.mul_f64(2.0 / height_px);
    let mut escapes: Vec<i64> = Vec::with_capacity(npix);
    for i in 0..npix {
        let off = i as f64 - (npix as f64 - 1.0) / 2.0; // pixels from centre along +x
        let dc = CFloatExp { re: step.mul_f64(off), im: FloatExp::ZERO };
        let mut dz = CFloatExp { re: FloatExp::ZERO, im: FloatExp::ZERO };
        let mut n: usize = 0;
        let mut esc: i64 = -1;
        for it in 0..max_iter as usize {
            let (zr, zi) = zs[n];
            let z2 = CFloatExp { re: FloatExp::from_f64(2.0 * zr), im: FloatExp::from_f64(2.0 * zi) };
            dz = z2 * dz + dz * dz + dc;
            n += 1;
            let (zr1, zi1) = zs[n.min(len - 1)];
            let zfull = CFloatExp { re: FloatExp::from_f64(zr1) + dz.re, im: FloatExp::from_f64(zi1) + dz.im };
            let zmag = zfull.abs();
            if zmag.to_f64() > 256.0 {
                esc = it as i64 + 1;
                break;
            }
            if zmag.lt(dz.abs()) || n + 1 >= len {
                dz = zfull;
                n = 0;
            }
        }
        escapes.push(esc);
    }
    // SAME row in plain f64 δz (53-bit mantissa vs FloatExp's 24-bit). Valid here because at e148
    // δz ~1e-148 and every intermediate fits f64's ~1e-308 range — so this isolates *mantissa
    // precision* as the variable (extended-range floatexp vs full-precision f64), the suspected
    // cause of the deep-interior speckle.
    let stepf = step.to_f64();
    let mut escapes64: Vec<i64> = Vec::with_capacity(npix);
    for i in 0..npix {
        let off = i as f64 - (npix as f64 - 1.0) / 2.0;
        let (dcx, dcy) = (stepf * off, 0.0);
        let (mut dzx, mut dzy) = (0.0f64, 0.0f64);
        let mut n: usize = 0;
        let mut esc: i64 = -1;
        for it in 0..max_iter as usize {
            let (zr, zi) = zs[n];
            let ndzx = 2.0 * (zr * dzx - zi * dzy) + (dzx * dzx - dzy * dzy) + dcx;
            let ndzy = 2.0 * (zr * dzy + zi * dzx) + 2.0 * dzx * dzy + dcy;
            dzx = ndzx;
            dzy = ndzy;
            n += 1;
            let (zr1, zi1) = zs[n.min(len - 1)];
            let (zfx, zfy) = (zr1 + dzx, zi1 + dzy);
            let zmag2 = zfx * zfx + zfy * zfy;
            if zmag2 > 256.0 * 256.0 {
                esc = it as i64 + 1;
                break;
            }
            if zmag2 < dzx * dzx + dzy * dzy || n + 1 >= len {
                dzx = zfx;
                dzy = zfy;
                n = 0;
            }
        }
        escapes64.push(esc);
    }
    // THIRD row: same f64 arithmetic but each δz component rounded to a 24-bit (f32) mantissa every
    // step — simulating the GPU mode-2 floatexp kernel (f32 mantissa + extended exponent). If THIS
    // row is speckle where the full-f64 one is smooth, the GPU's f32 δz precision is the real cause
    // of the 14/15 noise (an interior reference — verified in the render, len=800001 — does NOT fix
    // it, so it is not reference coverage).
    fn f32mant(x: f64) -> f64 {
        if x == 0.0 || !x.is_finite() {
            return x;
        }
        let e = x.abs().log2().floor();
        let s = 2f64.powf(e);
        ((x / s) as f32 as f64) * s
    }
    let mut escapes32: Vec<i64> = Vec::with_capacity(npix);
    for i in 0..npix {
        let off = i as f64 - (npix as f64 - 1.0) / 2.0;
        let (dcx, dcy) = (f32mant(stepf * off), 0.0);
        let (mut dzx, mut dzy) = (0.0f64, 0.0f64);
        let mut n: usize = 0;
        let mut esc: i64 = -1;
        for it in 0..max_iter as usize {
            let (zr, zi) = zs[n];
            dzx = f32mant(2.0 * (zr * dzx - zi * dzy) + (dzx * dzx - dzy * dzy) + dcx);
            dzy = f32mant(2.0 * (zr * dzy + zi * dzx) + 2.0 * dzx * dzy + dcy);
            n += 1;
            let (zr1, zi1) = zs[n.min(len - 1)];
            let (zfx, zfy) = (zr1 + dzx, zi1 + dzy);
            let zmag2 = zfx * zfx + zfy * zfy;
            if zmag2 > 256.0 * 256.0 {
                esc = it as i64 + 1;
                break;
            }
            if zmag2 < dzx * dzx + dzy * dzy || n + 1 >= len {
                dzx = f32mant(zfx);
                dzy = f32mant(zfy);
                n = 0;
            }
        }
        escapes32.push(esc);
    }
    let jumps = escapes.windows(2).filter(|w| (w[0] - w[1]).abs() > 2000).count();
    let jumps64 = escapes64.windows(2).filter(|w| (w[0] - w[1]).abs() > 2000).count();
    let jumps32 = escapes32.windows(2).filter(|w| (w[0] - w[1]).abs() > 2000).count();
    println!("[row] {label}: f32-MANTISSA (GPU-like) escapes = {escapes32:?}");
    println!("[row] {label}: jumps>2000 — f64={jumps64}  f32-mant={jumps32} (f32>>f64 over covered px => f32 δz precision is the cause)");
    println!("[row] {label}: floatexp(f32-mant) escapes = {escapes:?}");
    println!("[row] {label}: f64(53-bit-mant) escapes = {escapes64:?}");
    println!(
        "[row] {label}: adjacent-|Δ|>2000 jumps — floatexp={jumps}/{n1}  f64={jumps64}/{n1}  (f64<<floatexp => f32 mantissa is the culprit)",
        n1 = npix - 1
    );
}

/// What reference does `best_reference` actually pick here, and how long is its orbit vs the
/// view's deepest pixel? If the chosen reference escapes before the deepest pixel does, the
/// past-reference-length pixels rebase into speckle — the 14/15 noise.
/// spec: "label|cx|cy|mag_log10|max_iter|prec"
#[test]
fn probe_best_ref() {
    let Ok(spec) = std::env::var("PROBE_BESTREF") else {
        return;
    };
    let parts: Vec<&str> = spec.split('|').collect();
    let (label, cx, cy) = (parts[0], parts[1], parts[2]);
    let mag_log10: f64 = parts[3].parse().unwrap();
    let max_iter: u32 = parts[4].parse().unwrap();
    let prec: usize = parts[5].parse().unwrap();

    let center = [fractadyne_core::parse_bf(cx).unwrap(), fractadyne_core::parse_bf(cy).unwrap()];
    let mag_log2 = mag_log10 * std::f64::consts::LN_10 / std::f64::consts::LN_2;
    // Full view span (3-unit-high at mag 1), 16:9.
    let span = [
        FloatExp::from_f64(3.0 * 16.0 / 9.0).mul_pow2(-mag_log2),
        FloatExp::from_f64(3.0).mul_pow2(-mag_log2),
    ];
    let zero = fractadyne_core::BigFloat::from_f64(0.0, prec);
    let clen = fractadyne_core::reference_orbit(&zero, &zero, &center[0], &center[1], 0, max_iter, prec).1;
    let refpt = fractadyne_core::best_reference(&center, span, 0, false, [0.0, 0.0], max_iter, prec);
    let rlen = fractadyne_core::reference_orbit(&zero, &zero, &refpt[0], &refpt[1], 0, max_iter, prec).1;
    // How far (in pixels) did best_reference move from the centre?
    let dx = fractadyne_core::sub_f64(&refpt[0], &center[0], prec);
    let dy = fractadyne_core::sub_f64(&refpt[1], &center[1], prec);
    let step = span[1].mul_f64(1.0 / 180.0).to_f64();
    println!(
        "[bestref] {label}: center_len={clen}  best_reference_len={rlen}  max_iter={max_iter}  \
         chosen_ref_offset≈({:.0},{:.0})px",
        dx / step, dy / step
    );
    println!(
        "[bestref] {label}: {}",
        if rlen >= max_iter { "chosen ref is INTERIOR (covers all pixels — good)" }
        else { "chosen ref ESCAPES before max_iter — pixels escaping later than it rebase => NOISE" }
    );

    // PROTOTYPE the proposed fix: hill-climb from best_reference's pick toward a longer-surviving
    // (ideally interior) neighbour, shrinking the step on a barren round. Report whether it reaches
    // an interior reference and how many orbit-length scorings it cost.
    let score = |px: &fractadyne_core::BigFloat, py: &fractadyne_core::BigFloat, cap: u32| -> u32 {
        fractadyne_core::reference_orbit(&zero, &zero, px, py, 0, cap, prec).1
    };
    let mut cur = refpt.clone();
    let mut cur_len = rlen;
    let mut hx = span[0].mul_f64(0.01);
    let mut hy = span[1].mul_f64(0.01);
    let mut calls = 0u32;
    let offs = [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0), (1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
    'climb: for _round in 0..10 {
        let mut improved = false;
        for (ox, oy) in offs {
            let px = fractadyne_core::add_f64(&cur[0], hx.mul_f64(ox).to_f64(), prec);
            let py = fractadyne_core::add_f64(&cur[1], hy.mul_f64(oy).to_f64(), prec);
            let len = score(&px, &py, max_iter);
            calls += 1;
            if len > cur_len {
                cur = [px, py];
                cur_len = len;
                improved = true;
                if cur_len >= max_iter {
                    break 'climb;
                }
            }
        }
        if !improved {
            hx = hx.mul_f64(0.5);
            hy = hy.mul_f64(0.5);
        }
    }
    let cdx = fractadyne_core::sub_f64(&cur[0], &center[0], prec) / step;
    let cdy = fractadyne_core::sub_f64(&cur[1], &center[1], prec) / step;
    println!(
        "[climb] {label}: after {calls} scorings -> len={cur_len} (max_iter={max_iter}) offset≈({cdx:.0},{cdy:.0})px -- {}",
        if cur_len >= max_iter { "REACHED INTERIOR (fix works!)" } else { "still escaping (need a different approach)" }
    );
}
