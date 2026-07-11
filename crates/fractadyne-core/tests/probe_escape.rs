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
