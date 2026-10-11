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
        let mut s = Schedule::new(
            p,
            formula::MANDELBROT,
            crate::to_f64(&cx),
            crate::to_f64(&cy),
            SchedState::fresh(p, formula::MANDELBROT, true),
        );
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

/// Only the `z^d + c` families from `Z_0 = 0` schedule: another start and a fold family keep full
/// precision, and their tails say so.
#[test]
fn only_a_polynomial_orbit_from_zero_is_scheduled() {
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
        let (_, _, t) = reference_orbit_t(&zero, &zero, &cx, &cy, formula::MULTIBROT3, 200, p);
        assert!(t.sched.is_some(), "a Multibrot from zero is");
    });
}

/// The Misiurewicz points `M_{2,1}` of the `z^d + c` families, `d = 3..8`: `c = (ω − 1)^{1/(d−1)}`
/// with `ω = e^{2πi/d}` sends `0 → c → ωc`, a repelling fixed point (`|f'| = 2d·sin(π/d)`, 5.2 to
/// 6.1), so at 1,024 bits the orbit sits on it with `|dZ/dc|` growing 2.4 to 2.6 bits a step until
/// the rounding catches up, ~400 steps in. 320 digits (mpmath at 340). Mandelbrot's is `c = −2`.
const MISIUREWICZ: [(u32, &str, &str); 6] = [
        (3, "0.34062501931660664019439424403783088897721010254912530311970263261451737734879523436309889492913997531775946013057131997210225898564851667878502098754323200513772455965447466229899084218178487905964421090396559260244381123923871972707855442113120782319314079222550676114876128181435452752857530367056555141991023326475371", "1.2712298784187062391356129910210649767284143282267776782136791436934807913447441969146767160282631933396985181626638437283262450381644243328982863195606645889473223966573714015302679689506918934527700571712003809231292205368581525859863376676655240443359592661903330133245688152158896891887950070201114338828176934784966"),
        (4, "0.79370052598409973737585281963615413019574666394992650490414288091260825281210958663677210663111047851146738084066100895174882994907637613907000552227072330968775913928121843664525624253614616872488713768230358376855333190923785587617578753085228013639621325438390785723470347549812242252548193893501115864616130471248354", "0.79370052598409973737585281963615413019574666394992650490414288091260825281210958663677210663111047851146738084066100895174882994907637613907000552227072330968775913928121843664525624253614616872488713768230358376855333190923785587617578753085228013639621325438390785723470347549812242252548193893501115864616130471248354"),
        (5, "0.88782619963293125207493343584964025860010057081957893650735537147182013233834496528609542358153672454901132600172811813416273618767407465092937496217929359664965059137589262283457439394150569660365493384475603193516862114929504073980647322305691690552132679875176772045352548334117997067846484916131286335631478297481588", "0.54406059486634087480801666343537393643455287617417513810392536889256020478908409269419158681482455580902962622763862055128545368925913836960703312866491780921361348356821041735074927948498561807945348877382177259892033410848408701185890924249415453950225601351277876977306750453297554341858931574392337107409826907022988"),
        (6, "0.91354545764260089550212757198531717794081045937747454506099978806511488210263127168178400089329999498758288627658095385708406697448266303925245503211326212718861165222893830181635712414358974070235341024415224116142678697103860643233346912911780151757385046794678015785163206937556300657590560523220814337071921460495777", "0.40673664307580020775398599034149761292313965106617343629428635281703777652454640507080217052061959371896447773316821407222547052880237139654514044115462752131583925372381174202358781962403867958791869557733280539435304700006255212501556097884247420023201651555190174259784706058704882618926414443396708037432163650973774"),
        (7, "0.92183291533870412270516436976232071889881898903902592021854594164986217570593116163697761867407417435361315090940913586824745849378704907131123285258868999336469532271409850940468722877365929713877556288394829599876736582916040194934827956446365744945284132742854030047896823657143175170952093995349234553848322981973913", "0.32256328804747096785500678811458667896508407978758566184946632362078029086161324922904118384897966800301413164944692893081267130311918604784517546856073740664585900114855955349638522616468381297564048325405662586494569499070678786823680836066786098659527326480860725772872730344798854324335795352232447318063047515916370"),
        (8, "0.92490254217078572922021806539685306368191381737091372990110705708603546824515721868906999554290287741851830897502409377091753836399975431560466711954288573418908170770920805839329188432590746571359490884373810980136764764285419580821188377133327356373738824902118069856399779219203760616398122715411646753989395098819393", "0.26645983211302913922712526044686278033145712424735605685030584321090314404621051699052077688749315420778975971662211252355259468045099651211171168221119905557682436754164866441908408712465560294240440003596810615445511257110413464808531793008583876672910334567454832593805364643038026207365023839220200182083206714188337"),
];

fn multibrot(d: u32) -> u32 {
    match d {
        3 => formula::MULTIBROT3,
        4 => formula::MULTIBROT4,
        5 => formula::MULTIBROT5,
        _ => formula::MULTIBROT6 + (d - 6),
    }
}

/// `log2|dZ_k/dc|` along packed samples for `z^d + c` (`dZ_{k+1}/dc = d·Z_k^{d−1}·dZ_k/dc + 1`).
fn log2_derivatives_d(orbit: &[[f32; 4]], d: u32) -> Vec<f64> {
    let one = CFloatExp { re: FloatExp::from_f64(1.0), im: FloatExp::ZERO };
    let mut dz = CFloatExp::ZERO;
    let mut out = Vec::with_capacity(orbit.len());
    for s in orbit {
        out.push(dz.abs().log2());
        let (zr, zi) = sample_xy(s);
        let z = CFloatExp { re: FloatExp::from_f64(zr), im: FloatExp::from_f64(zi) };
        let mut zk = one;
        for _ in 1..d {
            zk = zk * z;
        }
        dz = (zk * dz).mul_f64(f64::from(d)) + one;
    }
    out
}

/// The Mandelbrot test above, for Multibrot 3 to 8 at their Misiurewicz points: the scheduled build
/// stores the full-precision build's samples bit for bit over the accurate stretch, the schedule
/// really ran low there, and a scheduled prefix extended to the same length is the fresh build.
#[test]
fn a_scheduled_multibrot_orbit_stores_the_full_precision_samples() {
    let p = 1024;
    let zero = BigFloat::from_f64(0.0, p);
    for (d, x, y) in MISIUREWICZ {
        let f = multibrot(d);
        assert_eq!(formula_power(f), Some(d));
        let cx = crate::parse_bf_prec(x, p).unwrap();
        let cy = crate::parse_bf_prec(y, p).unwrap();
        let probe = with_orbit_schedule(false, || reference_orbit(&zero, &zero, &cx, &cy, f, 2_000, p).0);
        let dd = log2_derivatives_d(&probe, d);
        let n = dd.iter().rposition(|&l| l < p as f64 - 96.0).expect("an accurate stretch") as u32;
        assert!(n > 250, "d={d}: too short a stretch to say anything ({n} steps)");

        let (full, full_len, full_tail) =
            with_orbit_schedule(false, || reference_orbit_t(&zero, &zero, &cx, &cy, f, n, p));
        let (sched, sched_len, sched_tail) =
            with_orbit_schedule(true, || reference_orbit_t(&zero, &zero, &cx, &cy, f, n, p));
        assert!(full_tail.sched.is_none(), "d={d}: the unscheduled build carries no schedule");
        let st = sched_tail.sched.expect("the scheduled build carries its state");
        assert!(st.cur * 2 < p as u32, "d={d}: the schedule never ran low: {} of {p} bits at step {n}", st.cur);
        assert_eq!(full_len, sched_len, "d={d}");
        let differ = full.iter().zip(&sched).filter(|(a, b)| a.map(f32::to_bits) != b.map(f32::to_bits)).count();
        assert_eq!(differ, 0, "d={d}: {differ} of {full_len} samples differ (to step {n})");

        let (prefix, _, tail) = with_orbit_schedule(true, || reference_orbit_t(&zero, &zero, &cx, &cy, f, n / 2, p));
        assert!(tail.sched.is_some(), "d={d}: the prefix carries its schedule");
        let (ext, ext_len, ext_tail) = extend_reference_orbit(&prefix, &tail, &cx, &cy, f, n, p);
        assert_eq!(ext_len, sched_len, "d={d}");
        assert!(ext == sched, "d={d}: the extension is not the scheduled build's continuation");
        assert_eq!(ext_tail.sched, sched_tail.sched, "d={d}: the extension ends in the fresh build's state");
    }
}
