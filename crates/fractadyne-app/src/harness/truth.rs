//! `--truth`: every pixel's escape count with no approximation a renderer makes (no GPU).
//! Dispatched from `cli::run_headless`. The arithmetic is `fractadyne_core::truth_at`
//! (full-step double-double perturbation, plain arbitrary-precision iteration wherever that will
//! not vouch for itself); this is the view, the threads, the self-check and the files.
//!
//!   fractadyne --truth DIR --center X Y --zoom-log2 L --size WxH --iter N
//!              [--truth-name NAME] [--truth-stencil] [--truth-check N] [--threads T]
//!              [--truth-direct]
//!
//! `--truth-direct`: no reference; every pixel by plain double-double iteration
//! (`fractadyne_core::truth_direct`), for a shallow view whose counts run too long to keep a
//! reference orbit (the π points, `scripts/pi_points.py`). Refused below a pixel step of 2^-80.
//!
//! Writes `DIR/NAME.truth.json` (the view, the reference, timings, the check) and
//! `DIR/NAME.truth.bin`: little-endian planes over the pixels in row order, top row first —
//! n2, n256, n4096 (i32 each; -1 never escaped), smooth256 (f32; -1 interior), flags (u8), then
//! the stencil's RANGE per radius: n2 lo, n2 hi, n256 lo, n256 hi, n4096 lo, n4096 hi (i32 each;
//! over the centre and the four points 1/1000 px away; lo -1 if any of them never escaped, hi -1
//! if none escaped; equal to the centre's count without --truth-stencil). The range is what lets a
//! sensitive pixel be judged too: a count outside every nearby point's answer is wrong wherever
//! the renderer sampled in that neighbourhood, so a filament a renderer thins or drops shows up
//! even where no single point is stable (format 2; found by the scorer's self-check, which missed
//! a planted thin filament with the stable/sensitive flag alone).
//! Flags:
//!   bit 0  plain iteration answered (the perturbation would not vouch for the pixel)
//!   bit 1  the stencil ran: the pixel was also evaluated 1/1000 px away in x and y, both ways
//!   bit 2  stable: all five points gave the same three counts (only meaningful with bit 1)
//!   bit 3  a check pixel: plain iteration also ran at its centre
//!   bit 4  the check agreed
//! The pixel mapping is the shader's (as `--glitch-audit`'s oracle): pixel (x, y)'s point is
//! centre + (x + ½ − w/2, h/2 − (y + ½)) · (span_mantissa / size) · 2^delta_exp.

use fractadyne_core::{BigFloat, FloatExp, TruthDwell};
use std::sync::atomic::{AtomicUsize, Ordering};

const STENCIL_PX: f64 = 1.0e-3;
const FLAG_PLAIN: u8 = 1;
const FLAG_STENCIL: u8 = 2;
const FLAG_STABLE: u8 = 4;
const FLAG_CHECKED: u8 = 8;
const FLAG_CHECK_OK: u8 = 16;

struct View {
    cx: BigFloat,
    cy: BigFloat,
    step_x: FloatExp,
    step_y: FloatExp,
    w: usize,
    h: usize,
    p: usize,
}

impl View {
    fn point(&self, x: f64, y: f64) -> (BigFloat, BigFloat) {
        (
            fractadyne_core::add_floatexp(&self.cx, self.step_x.mul_f64(x + 0.5 - self.w as f64 * 0.5), self.p),
            fractadyne_core::add_floatexp(&self.cy, self.step_y.mul_f64(self.h as f64 * 0.5 - (y + 0.5)), self.p),
        )
    }
}

fn counts(d: &TruthDwell) -> (i64, i64, i64) {
    (d.n2, d.n256, d.n4096)
}

pub(crate) fn run(args: &[String]) -> bool {
    let i = args.iter().position(|a| a == "--truth").expect("dispatched on --truth");
    let Some(dir) = args.get(i + 1).filter(|s| !s.starts_with("--")) else {
        eprintln!("fractadyne: --truth needs an output folder.");
        crate::exit(2)
    };
    let val = |name: &str| args.iter().position(|a| a == name).and_then(|j| args.get(j + 1));
    let Some((xs, ys)) = args
        .iter()
        .position(|a| a == "--center")
        .and_then(|j| Some((args.get(j + 1)?, args.get(j + 2)?)))
    else {
        eprintln!("fractadyne: --truth needs --center X Y.");
        crate::exit(2)
    };
    let Some(zl) = val("--zoom-log2") else {
        eprintln!("fractadyne: --truth needs --zoom-log2 L.");
        crate::exit(2)
    };
    let zoom_log2: f64 = crate::arg_parse("--zoom-log2", zl, "a number");
    // After the zoom: the centre is parsed at the view's precision (see `arg_center`).
    let (cx, cy) = crate::arg_center("--center", xs, ys, zoom_log2);
    let (w, h) = match val("--size").map(|s| crate::parse_size(s)) {
        Some((Some(w), Some(h))) => (w as usize, h as usize),
        _ => {
            eprintln!("fractadyne: --truth needs --size WxH.");
            crate::exit(2)
        }
    };
    let Some(it) = val("--iter") else {
        eprintln!("fractadyne: --truth needs --iter N.");
        crate::exit(2)
    };
    let max_iter: u32 = crate::arg_parse("--iter", it, "a whole number");
    let name = val("--truth-name").cloned().unwrap_or_else(|| "view".to_string());
    let stencil = args.iter().any(|a| a == "--truth-stencil");
    let direct = args.iter().any(|a| a == "--truth-direct");
    let checks: usize = val("--truth-check").map(|s| crate::arg_parse("--truth-check", s, "a whole number")).unwrap_or(200);
    let threads: usize = val("--threads")
        .map(|s| crate::arg_parse("--threads", s, "a whole number"))
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(4, |n| n.get()));

    // The app's own view of these numbers, so a pixel here is the point the renderer draws.
    let mut vp = fractadyne_core::Viewport::new(w as f64, h as f64);
    vp.set_center_log2mag(cx, cy, zoom_log2);
    let p = vp.precision + 64;
    let scale = vp.gpu_scale();
    let view = View {
        step_x: FloatExp::new(scale.span_mantissa.x / w as f64, scale.delta_exp),
        step_y: FloatExp::new(scale.span_mantissa.y / h as f64, scale.delta_exp),
        cx: vp.center_x.clone(),
        cy: vp.center_y.clone(),
        w,
        h,
        p,
    };
    eprintln!(
        "truth: {name}  {w}x{h}  2^{zoom_log2:.3}  iter {max_iter}  {p}-bit  {threads} threads  stencil {}{}",
        if stencil { "on" } else { "off" },
        if direct { "  direct" } else { "" }
    );
    if direct {
        let step = view.step_x.to_f64().abs().min(view.step_y.to_f64().abs());
        let far = fractadyne_core::to_f64(&view.cx).abs().max(fractadyne_core::to_f64(&view.cy).abs());
        if !(step >= fractadyne_core::TRUTH_DIRECT_MIN_STEP && far <= 4.0) {
            eprintln!("fractadyne: --truth-direct is for shallow views (a pixel step of 2^-80 or more, centre within 4); this one's step is {step:e}.");
            crate::exit(2)
        }
    }

    // THE REFERENCE: the longest-lived of the centre and a 3x3 grid a quarter of the view out. The
    // choice is speed only (rebasing makes any reference correct), so lengths are walked first and
    // only the winner's orbit is kept. None with --truth-direct.
    let t0 = std::time::Instant::now();
    let offsets: Vec<(f64, f64)> = [0.0, -0.25, 0.25]
        .iter()
        .flat_map(|&fy| [0.0, -0.25, 0.25].iter().map(move |&fx| (fx, fy)))
        .collect();
    let lens: Vec<usize> = if direct {
        Vec::new()
    } else {
        std::thread::scope(|s| {
            let hs: Vec<_> = offsets
                .iter()
                .map(|&(fx, fy)| {
                    let view = &view;
                    s.spawn(move || {
                        let (x, y) = (w as f64 * (0.5 + fx) - 0.5, h as f64 * (0.5 - fy) - 0.5);
                        let (px, py) = view.point(x, y);
                        fractadyne_core::truth_reference(&px, &py, max_iter, p).z.len()
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().expect("a reference walk panicked")).collect()
        })
    };
    let best = (0..lens.len()).max_by_key(|&k| (lens[k], usize::MAX - k)).unwrap_or(0);
    let (fx, fy) = offsets[best];
    let r = (!direct).then(|| {
        let (rx, ry) = view.point(w as f64 * (0.5 + fx) - 0.5, h as f64 * (0.5 - fy) - 0.5);
        fractadyne_core::truth_reference(&rx, &ry, max_iter, p)
    });
    let ref_s = t0.elapsed().as_secs_f64();
    match &r {
        Some(r) => eprintln!(
            "truth: reference at ({fx:+.2}, {fy:+.2}) of the view: {} samples, {} ({ref_s:.1} s; candidates {lens:?})",
            r.z.len(),
            if r.escaped { "escaped" } else { "reached the cap" }
        ),
        None => eprintln!("truth: no reference: every pixel by plain double-double iteration"),
    }
    // One point's counts, by the reference or directly (`true`: plain iteration answered).
    let eval = |px: &BigFloat, py: &BigFloat| -> (TruthDwell, bool) {
        match &r {
            Some(r) => fractadyne_core::truth_at(r, px, py, max_iter, p),
            None => (fractadyne_core::truth_direct(px, py, max_iter), false),
        }
    };

    // EVERY PIXEL, rows handed out to the threads in order.
    let t1 = std::time::Instant::now();
    let next = AtomicUsize::new(0);
    let done = AtomicUsize::new(0);
    let mut px_out: Vec<TruthDwell> = vec![TruthDwell::INTERIOR; w * h];
    let mut flags: Vec<u8> = vec![0; w * h];
    let mut ranges: Vec<[i64; 6]> = vec![[-1; 6]; w * h];
    #[allow(clippy::type_complexity)]
    let rows: Vec<(usize, Vec<TruthDwell>, Vec<u8>, Vec<[i64; 6]>)> = std::thread::scope(|s| {
        let hs: Vec<_> = (0..threads)
            .map(|_| {
                let (view, eval, next, done) = (&view, &eval, &next, &done);
                s.spawn(move || {
                    let mut out = Vec::new();
                    loop {
                        let y = next.fetch_add(1, Ordering::Relaxed);
                        if y >= h {
                            break;
                        }
                        let mut row = Vec::with_capacity(w);
                        let mut fl = Vec::with_capacity(w);
                        let mut rg = Vec::with_capacity(w);
                        for x in 0..w {
                            let (px, py) = view.point(x as f64, y as f64);
                            let (d, plain) = eval(&px, &py);
                            let mut f = if plain { FLAG_PLAIN } else { 0 };
                            let c = counts(&d);
                            let mut range = [c.0, c.0, c.1, c.1, c.2, c.2];
                            if stencil {
                                // All four points, every pixel: their spread is the range a render
                                // is held to, so none may be skipped once one has disagreed.
                                let k = STENCIL_PX;
                                let mut same = true;
                                for &(dx, dy) in &[(k, 0.0), (-k, 0.0), (0.0, k), (0.0, -k)] {
                                    let (sx, sy) = view.point(x as f64 + dx, y as f64 + dy);
                                    let s = counts(&eval(&sx, &sy).0);
                                    same &= s == c;
                                    for (j, v) in [s.0, s.1, s.2].into_iter().enumerate() {
                                        range[2 * j] = range[2 * j].min(v);
                                        range[2 * j + 1] = range[2 * j + 1].max(v);
                                    }
                                }
                                f |= FLAG_STENCIL | if same { FLAG_STABLE } else { 0 };
                            }
                            row.push(d);
                            fl.push(f);
                            rg.push(range);
                        }
                        out.push((y, row, fl, rg));
                        let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                        if n % (h / 20).max(1) == 0 {
                            eprintln!("truth: {n}/{h} rows ({:.0} s)", t1.elapsed().as_secs_f64());
                        }
                    }
                    out
                })
            })
            .collect();
        hs.into_iter().flat_map(|h| h.join().expect("a truth thread panicked")).collect()
    });
    for (y, row, fl, rg) in rows {
        px_out[y * w..(y + 1) * w].copy_from_slice(&row);
        flags[y * w..(y + 1) * w].copy_from_slice(&fl);
        ranges[y * w..(y + 1) * w].copy_from_slice(&rg);
    }
    let px_s = t1.elapsed().as_secs_f64();
    let plain_n = flags.iter().filter(|&&f| f & FLAG_PLAIN != 0).count();

    // THE CHECK: plain arbitrary-precision iteration at pixels spread over the view must give the
    // same three counts, at the same point. A disagreement condemns the whole view.
    let t2 = std::time::Instant::now();
    let mut seed = 0x7275_7468u64 ^ (w as u64 * 1_000_003 + h as u64);
    let mut pick = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as usize % (w * h)
    };
    let mut sample: Vec<usize> = (0..checks.min(w * h)).map(|_| pick()).collect();
    sample.sort_unstable();
    sample.dedup();
    let results: Vec<(usize, TruthDwell)> = std::thread::scope(|s| {
        let chunk = sample.len().div_ceil(threads).max(1);
        let hs: Vec<_> = sample
            .chunks(chunk)
            .map(|part| {
                let view = &view;
                s.spawn(move || {
                    part.iter()
                        .map(|&k| {
                            let (px, py) = view.point((k % w) as f64, (k / w) as f64);
                            (k, fractadyne_core::truth_naive(&px, &py, max_iter, p))
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        hs.into_iter().flat_map(|h| h.join().expect("a check thread panicked")).collect()
    });
    let mut bad = Vec::new();
    for (k, d) in &results {
        flags[*k] |= FLAG_CHECKED;
        if counts(d) == counts(&px_out[*k]) {
            flags[*k] |= FLAG_CHECK_OK;
        } else {
            bad.push((*k, *d, px_out[*k]));
        }
    }
    let check_s = t2.elapsed().as_secs_f64();

    // Files.
    let dir = std::path::Path::new(dir);
    if let Err(e) = std::fs::create_dir_all(dir) {
        eprintln!("fractadyne: --truth: cannot create {}: {e}", dir.display());
        crate::exit(2)
    }
    let mut bin = Vec::with_capacity(w * h * 41);
    for plane in 0..3 {
        for d in &px_out {
            let v = [d.n2, d.n256, d.n4096][plane];
            bin.extend_from_slice(&(v.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32).to_le_bytes());
        }
    }
    for d in &px_out {
        bin.extend_from_slice(&(d.smooth256 as f32).to_le_bytes());
    }
    bin.extend_from_slice(&flags);
    for j in 0..6 {
        for rg in &ranges {
            bin.extend_from_slice(&(rg[j].clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32).to_le_bytes());
        }
    }
    let stable_n = flags.iter().filter(|&&f| f & FLAG_STABLE != 0).count();
    let escaped_n = px_out.iter().filter(|d| d.n2 >= 0).count();
    let json = format!(
        "{{\n \"format\": 2,\n \"name\": \"{name}\",\n \"width\": {w},\n \"height\": {h},\n \"zoom_log2\": {zoom_log2},\n \
         \"max_iter\": {max_iter},\n \"precision_bits\": {p},\n \"center_re\": \"{xs}\",\n \"center_im\": \"{ys}\",\n \
         \"planes\": [\"n2 i32\", \"n256 i32\", \"n4096 i32\", \"smooth256 f32\", \"flags u8\", \"n2_lo i32\", \"n2_hi i32\", \"n256_lo i32\", \"n256_hi i32\", \"n4096_lo i32\", \"n4096_hi i32\"],\n \
         \"direct\": {direct},\n \"reference\": {{\"view_offset\": [{fx}, {fy}], \"samples\": {}, \"escaped\": {}, \"candidate_samples\": {lens:?}}},\n \
         \"pixels\": {{\"escaped\": {escaped_n}, \"interior\": {}, \"plain_iteration\": {plain_n}, \"stencil\": {stencil}, \"stable\": {stable_n}}},\n \
         \"check\": {{\"pixels\": {}, \"disagree\": {}}},\n \
         \"seconds\": {{\"reference\": {ref_s:.2}, \"pixels\": {px_s:.2}, \"check\": {check_s:.2}}},\n \"threads\": {threads},\n \
         \"build\": \"{}\"\n}}\n",
        r.as_ref().map_or(0, |r| r.z.len()),
        r.as_ref().is_some_and(|r| r.escaped),
        w * h - escaped_n,
        results.len(),
        bad.len(),
        crate::sysinfo::version_string(),
    );
    let (jp, bp) = (dir.join(format!("{name}.truth.json")), dir.join(format!("{name}.truth.bin")));
    if let Err(e) = std::fs::write(&jp, json).and_then(|_| std::fs::write(&bp, &bin)) {
        eprintln!("fractadyne: --truth: cannot write {}: {e}", dir.display());
        crate::exit(2)
    }
    println!(
        "truth: {name}: {escaped_n} escaped, {} interior; plain iteration answered {plain_n}; {}reference {ref_s:.1} s, pixels {px_s:.1} s, check {check_s:.1} s",
        w * h - escaped_n,
        if stencil { format!("stable {stable_n} of {}; ", w * h) } else { String::new() }
    );
    if bad.is_empty() {
        println!("truth: CHECK PASS - plain iteration agrees at all {} sampled pixels -> {}", results.len(), jp.display());
    } else {
        for (k, want, got) in bad.iter().take(10) {
            println!("truth: CHECK FAIL at ({}, {}): plain {:?}, perturbation {:?}", k % w, k / w, counts(want), counts(got));
        }
        println!("truth: CHECK FAIL - {} of {} sampled pixels disagree; this view's truth is NOT to be used", bad.len(), results.len());
        crate::exit(1)
    }
    true
}
