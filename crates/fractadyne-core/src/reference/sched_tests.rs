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
    let a = with_orbit_schedule(false, || orbit_length_bf(&zero, &zero, &cx, &cy, formula::MANDELBROT, 800_000, p, false));
    let b = with_orbit_schedule(true, || orbit_length_bf(&zero, &zero, &cx, &cy, formula::MANDELBROT, 800_000, p, false));
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

/// What schedules: the `z^d + c` families and the folds from `Z_0 = 0` (c plane) or as a Julia
/// view; a c-plane walk from another start, and Phoenix, keep full precision, and their tails say so.
#[test]
fn which_orbits_are_scheduled() {
    let p = 1024;
    let (cx, cy) = near_i(p);
    let zero = BigFloat::from_f64(0.0, p);
    let other = BigFloat::from_f64(0.25, p);
    with_orbit_schedule(true, || {
        let (_, _, t) = reference_orbit_t(&other, &zero, &cx, &cy, formula::MANDELBROT, 200, p);
        assert!(t.sched.is_none(), "a start off zero is not scheduled");
        let (_, _, t) = reference_orbit_t(&zero, &zero, &cx, &cy, formula::TRICORN, 200, p);
        assert!(t.sched.is_some_and(|s| s.j.is_some()), "the tricorn is, by its Jacobian");
        let (_, _, t) = reference_orbit_t(&zero, &zero, &cx, &cy, formula::PHOENIX, 200, p);
        assert!(t.sched.is_none(), "Phoenix is not");
        let (_, _, t) = reference_orbit_t(&zero, &zero, &cx, &cy, formula::MANDELBROT, 200, p);
        assert!(t.sched.is_some(), "the Mandelbrot from zero is");
        let (_, _, t) = reference_orbit_t(&zero, &zero, &cx, &cy, formula::MULTIBROT3, 200, p);
        assert!(t.sched.is_some(), "a Multibrot from zero is");
        let (_, _, t) = reference_orbit_julia_t(&other, &zero, &cx, &cy, formula::MANDELBROT, 200, p);
        assert!(t.sched.is_some_and(|s| s.julia), "a Julia view is, in its own plane");
        let (_, _, t) = reference_orbit_julia_t(&other, &zero, &cx, &cy, formula::TRICORN, 200, p);
        assert!(t.sched.is_some_and(|s| s.julia && s.j.is_some()), "a tricorn Julia view is, in its plane");
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

/// Repelling fixed points `β` of `z^d + c` (`β^d + c = β`) for two Julia constants: a Julia orbit
/// from `β` stays put while `|dZ/dZ_0|` grows `log2|d·β^{d−1}|` bits a step (1.6 for d = 2, 2.05 for
/// d = 3), so at 1,024 bits the schedule falls far while the orbit is accurate. 320 digits (mpmath).
const JULIA_BETA: [(u32, &str, &str, &str, &str); 2] = [
        (2, "-0.8", "0.156", "1.5275031186435346107897464024449153375667459478117081603398751970032030110974513871892240841872097036285278118446470216567675511048976381591052049755871002178668204260624975532103883153157850045642586716683757126554935558516329825172798984230759160894866588240765086337258890754472162281949012509562911231362152794511317", "-0.075912178352287867071814194826348046366422194847978022539732593449186891503078959907318388553620835791521773673509239408146903775435705768770946476723892232618119845275553360281913106439290191996395355249148696879548476966056803740776194271345874761975802848949142559058010559243202318523468404390439539001577743141061347"),
        (3, "0.4", "0.3", "-1.1704074112365126914112728416701441973107854513040361277264055611820071716701862728166014491780241452073254690305035070792468780817691622722567595378677543416231543729687718918107278290307877257959096332398247228734619520439070379937625122340250184885003376167683854768953729905446766476966917786722521361038741887820044", "-0.096768060724085084544219802220566855051561723002067248723755909438584355132952627120669361723819874365286468333162893722172984236615610486253061358044852989366167596840881643072854333732875911888017613919988914237939124886474913239959240081980254584466631039981404420991707079940540998999110187868288138663962856400706887"),
];

/// `log2|dZ_k/dZ_0|` along a Julia view's packed samples (`D_0 = 1`, `D_{k+1} = d·Z_k^{d−1}·D_k`).
fn log2_derivatives_julia(orbit: &[[f32; 4]], d: u32) -> Vec<f64> {
    let one = CFloatExp { re: FloatExp::from_f64(1.0), im: FloatExp::ZERO };
    let mut dz = one;
    let mut out = Vec::with_capacity(orbit.len());
    for s in orbit {
        out.push(dz.abs().log2());
        let (zr, zi) = sample_xy(s);
        let z = CFloatExp { re: FloatExp::from_f64(zr), im: FloatExp::from_f64(zi) };
        let mut zk = one;
        for _ in 1..d {
            zk = zk * z;
        }
        dz = (zk * dz).mul_f64(f64::from(d));
    }
    out
}

/// P2: a Julia view's scheduled build stores the full-precision build's samples bit for bit over
/// the accurate stretch, the schedule really ran low (in the Julia plane), and a scheduled prefix
/// extended to the same length is the fresh build — the extension resumes the plane from the tail.
#[test]
fn a_scheduled_julia_orbit_stores_the_full_precision_samples() {
    let p = 1024;
    for (d, cre, cim, bre, bim) in JULIA_BETA {
        let f = if d == 2 { formula::MANDELBROT } else { multibrot(d) };
        let cx = crate::parse_bf_prec(cre, p).unwrap();
        let cy = crate::parse_bf_prec(cim, p).unwrap();
        let z0x = crate::parse_bf_prec(bre, p).unwrap();
        let z0y = crate::parse_bf_prec(bim, p).unwrap();
        let probe = with_orbit_schedule(false, || reference_orbit_julia_t(&z0x, &z0y, &cx, &cy, f, 2_000, p).0);
        let dd = log2_derivatives_julia(&probe, d);
        // The accurate stretch ends where |dZ/dZ_0| first reaches the precision less a guard. (Not
        // the LAST step under it: off β the orbit can settle on an attracting cycle, where the
        // derivative shrinks again and the schedule rightly climbs back to full.)
        let n = dd.iter().position(|&l| l >= p as f64 - 96.0).expect("the derivative outgrows the precision") as u32 - 1;
        assert!(n > 250, "d={d}: too short a stretch to say anything ({n} steps)");

        let (full, full_len, full_tail) =
            with_orbit_schedule(false, || reference_orbit_julia_t(&z0x, &z0y, &cx, &cy, f, n, p));
        let (sched, sched_len, sched_tail) =
            with_orbit_schedule(true, || reference_orbit_julia_t(&z0x, &z0y, &cx, &cy, f, n, p));
        assert!(full_tail.sched.is_none(), "d={d}: the unscheduled build carries no schedule");
        let st = sched_tail.sched.expect("the scheduled build carries its state");
        assert!(st.julia, "d={d}: the state must say it is a Julia walk");
        assert!(st.cur * 2 < p as u32, "d={d}: the schedule never ran low: {} of {p} bits at step {n}", st.cur);
        assert_eq!(full_len, sched_len, "d={d}");
        let differ = full.iter().zip(&sched).filter(|(a, b)| a.map(f32::to_bits) != b.map(f32::to_bits)).count();
        assert_eq!(differ, 0, "d={d}: {differ} of {full_len} samples differ (to step {n})");

        let (prefix, _, tail) =
            with_orbit_schedule(true, || reference_orbit_julia_t(&z0x, &z0y, &cx, &cy, f, n / 2, p));
        assert!(tail.sched.is_some_and(|s| s.julia), "d={d}: the prefix carries its Julia schedule");
        let (ext, ext_len, ext_tail) = extend_reference_orbit(&prefix, &tail, &cx, &cy, f, n, p);
        assert_eq!(ext_len, sched_len, "d={d}");
        assert!(ext == sched, "d={d}: the extension is not the scheduled build's continuation");
        assert_eq!(ext_tail.sched, sched_tail.sched, "d={d}: the extension ends in the fresh build's state");
    }
}

/// Misiurewicz-type points of the fold families (P3), shapes × powers 2 and 3: `c` with
/// `Z_2 = f(f(0))` a repelling fixed point of `f` (`|f'|` 4 to 6), the orbit clear of every fold and
/// off the real axis (`local/bench-ladder/fold_points.py`, Newton in mpmath at 340 digits).
const FOLD_POINTS: [(&str, u32, &str, &str); 8] = [
        ("burning-ship", 2, "1.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000", "-1.7320508075688772935274463415058723669428052538103806280558069794519330169088000370811461867572485756756261414154067030299699450949989524788116555120943736485280932319023055820679748201010846749232650153123432669033228866506722546689218379712270471316603678615880190499865373798593894676503475065760507566183481296061009"),
        ("burning-ship", 3, "1.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000", "-1.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        ("tricorn", 2, "1.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000", "1.7320508075688772935274463415058723669428052538103806280558069794519330169088000370811461867572485756756261414154067030299699450949989524788116555120943736485280932319023055820679748201010846749232650153123432669033228866506722546689218379712270471316603678615880190499865373798593894676503475065760507566183481296061009"),
        ("tricorn", 3, "-1.0441117142463501868771331103076476605968950057534141242733042992734528074469165658942249007653771111773747269329491435636282869652336849070788322253924269425624774793826426985230531039695948115026653969530874318783387804836902817947370450497966349530185414687490372535224470563850199737957473186735423697400773083146264", "-8.0117509680620082647188938328443644646922038530786550352484142124445555454112318606881917105952431994243066011692197996271071947126525500565208544531491058623041475684481019051431498658100949000777021014849897718412557074734016471329381887452339188416050793704351480927708999393777371661774960007087075031238622011954338e-1"),
        ("celtic", 2, "-1.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000", "1.7320508075688772935274463415058723669428052538103806280558069794519330169088000370811461867572485756756261414154067030299699450949989524788116555120943736485280932319023055820679748201010846749232650153123432669033228866506722546689218379712270471316603678615880190499865373798593894676503475065760507566183481296061009"),
        ("celtic", 3, "-2.3805545409929041668386720768381798431983872908076334538132571362802441878067997857378211857201748925377592590496223833877847095236305053975531660282994467929673968279838293916798056863802359671540260891291897144142460829481212639403877677875925964170975305506432789647038561128691510866573149658363869923354055900733632e-1", "-1.3500795688941438838424863554373537763012030732892347816415214133803821262695984079630586147474194172087427607756875141594066526767987036303709410693916871826260964448686915536067380874075599295769396734451053536849603602633539918154787186776412378096595300312752645783115919554976185582761226281766246918118121014737207"),
        ("buffalo", 2, "-1.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000", "-1.7320508075688772935274463415058723669428052538103806280558069794519330169088000370811461867572485756756261414154067030299699450949989524788116555120943736485280932319023055820679748201010846749232650153123432669033228866506722546689218379712270471316603678615880190499865373798593894676503475065760507566183481296061009"),
        ("buffalo", 3, "-1.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000", "-1.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
];

fn fold_formula(name: &str, d: u32) -> u32 {
    match (name, d) {
        ("burning-ship", 2) => formula::BURNING_SHIP,
        ("burning-ship", _) => formula::BURNING_SHIP3,
        ("tricorn", 2) => formula::TRICORN,
        ("tricorn", _) => formula::TRICORN3,
        ("celtic", 2) => formula::CELTIC,
        ("celtic", _) => formula::CELTIC3,
        ("buffalo", 2) => formula::BUFFALO,
        _ => formula::BUFFALO3,
    }
}

/// `log2 σ_min` of a fold walk's Jacobian along its packed samples, as the schedule reckons it: the
/// c plane from `J_0 = 0` (`J_{k+1} = J_step(Z_k)·J_k + I`), a Julia view from `J_0 = I` (no `+ I`).
fn log2_fold_contraction(orbit: &[[f32; 4]], f: u32, julia: bool) -> Vec<f64> {
    let (shape, d) = fold_kind(f).unwrap();
    let zero = [FloatExp::ZERO; 4];
    let mut j = if julia { MAT_ID } else { zero };
    let mut out = Vec::with_capacity(orbit.len());
    for s in orbit {
        out.push(if j == zero { f64::NEG_INFINITY } else { log2_sigma_min(j) });
        let (zr, zi) = sample_xy(s);
        let z = CFloatExp { re: FloatExp::from_f64(zr), im: FloatExp::from_f64(zi) };
        j = mat_mul(fold_step_jacobian(shape, d, z), j);
        if !julia {
            j[0] = j[0] + FloatExp::from_f64(1.0);
            j[3] = j[3] + FloatExp::from_f64(1.0);
        }
    }
    out
}

/// One scheduled-vs-full comparison of a fold walk (see the tests below).
fn assert_fold_schedule(label: &str, f: u32, z0: (&BigFloat, &BigFloat), c: (&BigFloat, &BigFloat), julia: bool, p: usize) {
    let build = |on: bool, n: u32| {
        with_orbit_schedule(on, || {
            if julia {
                reference_orbit_julia_t(z0.0, z0.1, c.0, c.1, f, n, p)
            } else {
                reference_orbit_t(z0.0, z0.1, c.0, c.1, f, n, p)
            }
        })
    };
    let probe = build(false, 2_000).0;
    let k = log2_fold_contraction(&probe, f, julia);
    // The accurate stretch ends where the Jacobian first outgrows the precision less a guard.
    let n = k.iter().position(|&l| l >= p as f64 - 96.0).expect("the Jacobian outgrows the precision") as u32 - 1;
    assert!(n > 200, "{label}: too short a stretch to say anything ({n} steps)");
    let (full, full_len, full_tail) = build(false, n);
    let (sched, sched_len, sched_tail) = build(true, n);
    assert!(full_tail.sched.is_none(), "{label}: the unscheduled build carries no schedule");
    let st = sched_tail.sched.expect("the scheduled build carries its state");
    assert!(st.j.is_some() && st.julia == julia, "{label}: not a fold state of this plane ({st:?})");
    assert!(st.cur * 2 < p as u32, "{label}: the schedule never ran low: {} of {p} bits at step {n}", st.cur);
    assert_eq!(full_len, sched_len, "{label}");
    let differ = full.iter().zip(&sched).filter(|(a, b)| a.map(f32::to_bits) != b.map(f32::to_bits)).count();
    assert_eq!(differ, 0, "{label}: {differ} of {full_len} samples differ (to step {n})");
    let (prefix, _, tail) = build(true, n / 2);
    let (ext, ext_len, ext_tail) = extend_reference_orbit(&prefix, &tail, c.0, c.1, f, n, p);
    assert_eq!(ext_len, sched_len, "{label}");
    assert!(ext == sched, "{label}: the extension is not the scheduled build's continuation");
    assert_eq!(ext_tail.sched, sched_tail.sched, "{label}: the extension ends in the fresh build's state");
}

/// P3, the c plane: every fold shape at powers 2 and 3 from its Misiurewicz-type point — samples bit
/// for bit with the full-precision build over the accurate stretch, the schedule low, extend == fresh.
#[test]
fn a_scheduled_fold_orbit_stores_the_full_precision_samples() {
    let p = 1024;
    let zero = BigFloat::from_f64(0.0, p);
    for (name, d, x, y) in FOLD_POINTS {
        let (cx, cy) = (crate::parse_bf_prec(x, p).unwrap(), crate::parse_bf_prec(y, p).unwrap());
        assert_fold_schedule(&format!("{name} d={d}"), fold_formula(name, d), (&zero, &zero), (&cx, &cy), false, p);
    }
}

/// P3, Julia views: the walk from the repelling fixed point `Z_2 = f(f(0))` of two of those `c`s
/// (computed here in bignum), where the derivative is with respect to `Z_0`.
#[test]
fn a_scheduled_fold_julia_orbit_stores_the_full_precision_samples() {
    let p = 1024;
    let zero = BigFloat::from_f64(0.0, p);
    for (name, d, x, y) in FOLD_POINTS.into_iter().filter(|(n, d, _, _)| (*n == "tricorn" && *d == 2) || (*n == "burning-ship" && *d == 3)) {
        let f = fold_formula(name, d);
        let (cx, cy) = (crate::parse_bf_prec(x, p).unwrap(), crate::parse_bf_prec(y, p).unwrap());
        let (z1x, z1y) = step_bf(&zero, &zero, &cx, &cy, f, p);
        let (z2x, z2y) = step_bf(&z1x, &z1y, &cx, &cy, f, p);
        assert_fold_schedule(&format!("{name} d={d} julia"), f, (&z2x, &z2y), (&cx, &cy), true, p);
    }
}
