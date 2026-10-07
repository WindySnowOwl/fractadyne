//! THE PRECISION SCHEDULE against the full-precision build (see `Schedule` in `reference.rs`).

use super::*;

/// A point ~1e-271 from the Misiurewicz point `c = i`, at 1,024 bits: its orbit follows i's
/// pre-periodic one (`|2Z|` = 2√2 and 2, ~1.25 bits of derivative a step) for ~700 steps with
/// inexact arithmetic all the way, so the schedule falls far below full precision while the orbit
/// is still accurate. (The seahorse point the extend test uses grew too slowly to leave full.)
fn near_i(p: usize) -> (BigFloat, BigFloat) {
    let im = format!("1.{}95", "0".repeat(271));
    (crate::parse_bf_prec("7.07e-272", p).unwrap(), crate::parse_bf_prec(&im, p).unwrap())
}

/// `log2|dZ_k/dc|` along packed samples (`dZ_{k+1}/dc = 2·Z_k·dZ_k/dc + 1`), extended range.
fn log2_derivatives(orbit: &[[f32; 4]]) -> Vec<f64> {
    let mut out = Vec::with_capacity(orbit.len());
    let (mut dr, mut di, mut de) = (0.0f64, 0.0f64, 0i32);
    for s in orbit {
        let m = dr.hypot(di);
        out.push(if m > 0.0 { m.log2() + de as f64 } else { f64::NEG_INFINITY });
        let (zr, zi) = sample_xy(s);
        // 2·Z·D at D's scale 2^de, then the + 1.
        let (tr, ti) = (2.0 * (zr * dr - zi * di), 2.0 * (zr * di + zi * dr));
        let (nr, ni, ne) = if m == 0.0 || de < -1000 {
            (1.0, 0.0, 0) // D ≈ 0: D' = 1
        } else if de > 1000 {
            (tr, ti, de) // the + 1 is far under D's last bit
        } else {
            (tr + (-de as f64).exp2(), ti, de)
        };
        let k = nr.hypot(ni).log2().floor() as i32;
        let sc = (-k as f64).exp2();
        (dr, di, de) = (nr * sc, ni * sc, ne + k);
    }
    out
}

/// The scheduled build stores the full-precision build's samples bit for bit over the steps where
/// either is accurate (`log2|dZ/dc|` under the precision less a guard; past that both have lost
/// the orbit and nothing reads it), and the schedule really ran low there. Then the extend
/// contract: a scheduled prefix extended to the same length equals the scheduled fresh build.
#[test]
fn a_scheduled_orbit_stores_the_full_precision_samples() {
    let p = 1024;
    let (cx, cy) = near_i(p);
    let zero = BigFloat::from_f64(0.0, p);
    let probe = with_orbit_schedule(false, || reference_orbit(&zero, &zero, &cx, &cy, formula::MANDELBROT, 3_000, p).0);
    let d = log2_derivatives(&probe);
    let n = d.iter().rposition(|&l| l < p as f64 - 96.0).expect("an accurate stretch") as u32;
    assert!(n > 400, "too short a stretch to say anything ({n} steps)");

    let (full, full_len, full_tail) =
        with_orbit_schedule(false, || reference_orbit_t(&zero, &zero, &cx, &cy, formula::MANDELBROT, n, p));
    let (sched, sched_len, sched_tail) =
        with_orbit_schedule(true, || reference_orbit_t(&zero, &zero, &cx, &cy, formula::MANDELBROT, n, p));
    assert!(full_tail.sched.is_none(), "the unscheduled build carries no schedule");
    let st = sched_tail.sched.expect("the scheduled build carries its state");
    assert!(st.cur * 2 < p as u32, "the schedule never ran low: {} of {p} bits at step {n}", st.cur);
    assert_eq!(full_len, sched_len);
    let differ = full.iter().zip(&sched).filter(|(a, b)| a.map(f32::to_bits) != b.map(f32::to_bits)).count();
    assert_eq!(differ, 0, "{differ} of {full_len} samples differ (to step {n})");

    let (prefix, _, tail) =
        with_orbit_schedule(true, || reference_orbit_t(&zero, &zero, &cx, &cy, formula::MANDELBROT, n / 2, p));
    assert!(tail.sched.is_some(), "the prefix carries its schedule");
    let (ext, ext_len, ext_tail) = extend_reference_orbit(&prefix, &tail, &cx, &cy, formula::MANDELBROT, n, p);
    assert_eq!(ext_len, sched_len);
    assert!(ext == sched, "the extension is not the scheduled build's continuation");
    assert_eq!(ext_tail.sched, sched_tail.sched, "the extension ends in the fresh build's state");
}

/// Probe (ignored): the 1.2e148 corpus centre walked at the pick's 557 bits with and without the
/// schedule — length, and where the samples first part.
#[test]
#[ignore]
fn probe_schedule_at_the_1e148_centre() {
    let kfr = include_str!("../../../../validation/corpus/locations/14-deep-1.2e148.kfr");
    let field = |k: &str| kfr.lines().find_map(|l| l.strip_prefix(k)).map(str::trim).unwrap().to_string();
    let p = 557;
    let cx = crate::parse_bf_prec(&field("Re:"), p).unwrap();
    let cy = crate::parse_bf_prec(&field("Im:"), p).unwrap();
    let zero = BigFloat::from_f64(0.0, p);
    let a = with_orbit_schedule(false, || orbit_length_bf(&zero, &zero, &cx, &cy, formula::MANDELBROT, 800_000, p));
    let b = with_orbit_schedule(true, || orbit_length_bf(&zero, &zero, &cx, &cy, formula::MANDELBROT, 800_000, p));
    println!("length: full {a}, scheduled {b}");
    let n = a.min(b).min(20_000);
    let (fa, _, _) = with_orbit_schedule(false, || reference_orbit_t(&zero, &zero, &cx, &cy, formula::MANDELBROT, n, p));
    let (fb, _, tb) = with_orbit_schedule(true, || reference_orbit_t(&zero, &zero, &cx, &cy, formula::MANDELBROT, n, p));
    let first = fa.iter().zip(&fb).position(|(x, y)| x.map(f32::to_bits) != y.map(f32::to_bits));
    let d = log2_derivatives(&fa);
    println!("first differing sample {first:?}; log2|D| there {:?}; final sched {:?}", first.map(|k| d[k]), tb.sched);
    if let Some(k) = first {
        for j in k.saturating_sub(3)..(k + 3).min(fa.len()) {
            println!("  {j}: full {:?} sched {:?} log2|D| {:.1}", sample_xy(&fa[j]), sample_xy(&fb[j]), d[j]);
        }
        // Replay the schedule's decisions on the full orbit's samples.
        let mut s = Schedule::new(p, crate::to_f64(&cx), crate::to_f64(&cy), SchedState::fresh(p, formula::MANDELBROT, true));
        let mut cur = vec![s.bits()];
        for smp in fa.iter().skip(1) {
            let (x, y) = sample_xy(smp);
            s.advance(CFloatExp { re: FloatExp::from_f64(x), im: FloatExp::from_f64(y) }, x, y);
            cur.push(s.bits());
        }
        let lo = (0..k).min_by_key(|&j| cur[j]).unwrap();
        println!("  lowest bits before {k}: {} at step {lo} (|Z| {:e}, log2|D| {:.1})", cur[lo], {
            let (x, y) = sample_xy(&fa[lo]);
            x.hypot(y)
        }, d[lo]);
        for j in (k.saturating_sub(4000)..k).step_by(250) {
            println!("    step {j}: {} bits, log2|D| {:.1}", cur[j], d[j]);
        }
    }
}

/// Only Mandelbrot from `Z_0 = 0` schedules: another start and another family keep full precision,
/// and their tails say so.
#[test]
fn only_a_mandelbrot_orbit_from_zero_is_scheduled() {
    let p = 1024;
    let (cx, cy) = near_i(p);
    let zero = BigFloat::from_f64(0.0, p);
    let other = BigFloat::from_f64(0.25, p);
    with_orbit_schedule(true, || {
        let (_, _, t) = reference_orbit_t(&other, &zero, &cx, &cy, formula::MANDELBROT, 200, p);
        assert!(t.sched.is_none(), "a start off zero is not scheduled");
        let (_, _, t) = reference_orbit_t(&zero, &zero, &cx, &cy, formula::TRICORN, 200, p);
        assert!(t.sched.is_none(), "the tricorn is not scheduled");
        let (_, _, t) = reference_orbit_t(&zero, &zero, &cx, &cy, formula::MANDELBROT, 200, p);
        assert!(t.sched.is_some(), "the Mandelbrot from zero is");
    });
}
