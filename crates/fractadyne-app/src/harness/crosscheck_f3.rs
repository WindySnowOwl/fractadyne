//! `--crosscheck-f3`: Fraktaler-3's raw escape counts against this app's CPU bignum oracle (no GPU). Dispatched from `cli::run_headless`.

pub(crate) fn run(args: &[String], i: usize) -> bool {
    let Some(file) = args.get(i + 1) else {
        eprintln!("--crosscheck-f3 needs an EXR path");
        return true;
    };
    let val = |name: &str| args.iter().position(|a| a == name).and_then(|j| args.get(j + 1));
    let two = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|j| Some((args.get(j + 1)?, args.get(j + 2)?)))
    };
    // This mode is a CORRECTNESS ORACLE: it decides whether Fraktaler-3 and our independent
    // bignum dwell computation agree. A silently defaulted coordinate or magnification here
    // does not produce a wrong picture, it produces a wrong VERDICT — and one that reads as
    // "the other renderer disagrees with us" rather than "you mistyped a flag".
    let center = match two("--center") {
        Some((x, y)) => {
            let (cx, cy) = crate::arg_center("--center", x, y);
            [cx, cy]
        }
        None => [
            fractadyne_core::BigFloat::from_f64(-0.5, 64),
            fractadyne_core::BigFloat::from_f64(0.0, 64),
        ],
    };
    let f3_zoom = val("--zoom-f3")
        .map(|s| {
            let z = crate::arg_parse::<f64>("--zoom-f3", s, "a number");
            // `f64::from_str` returns +inf for an overflowing literal instead of an error,
            // and the corpus's deepest locations are far past that. `spacing` would be 0 and
            // every sample would land on the same coordinate.
            if z.is_finite() && z > 0.0 {
                z
            } else {
                eprintln!(
                    "fractadyne: --zoom-f3: \"{s}\" is not a finite positive \
                         magnification (this comparison works in f64, so it cannot reach past \
                         ~1e308x)."
                );
                crate::exit(2)
            }
        })
        .unwrap_or(1.0);
    let max_iter = val("--iter")
        .map(|s| crate::arg_parse::<u32>("--iter", s, "a whole number"))
        .unwrap_or(10_000);
    let er = val("--er")
        .map(|s| crate::arg_parse::<f64>("--er", s, "a number"))
        .unwrap_or(256.0);
    let bailout2 = er * er;
    // Our magnification convention (height 3) vs F3's (height 4): mag = 0.75·zoom.
    let our_mag = 0.75 * f3_zoom;
    let prec = fractadyne_core::precision_for_magnification(our_mag).max(64);

    let Ok((w, h, nch)) = fractadyne_export::read_exr_channel_f32(std::path::Path::new(file), "N")
    else {
        eprintln!(
            "could not read EXR channel \"N\" from {file}. Channels present: {:?}\n\
                 (Fraktaler-3 batch must set render.exr_channels = [\"N0\"] and render.save_exr = true.)",
            fractadyne_export::list_exr_channels(std::path::Path::new(file)).unwrap_or_default()
        );
        return true;
    };
    let (wf, hf) = (w as f64, h as f64);
    // F3 pixel spacing; saved EXR is vertically flipped (vertical_flip defaults false
    // ⇒ save_exr flips), so saved row y maps to kernel row h-1-y.
    let spacing = 4.0 / f3_zoom / hf;
    let cx0 = &center[0];
    let cy0 = &center[1];
    // F3 interior sentinel: N0 = 0xFFFFFFFF (reads as ~4.29e9 in f32); exterior n = N0 - 1024.
    let is_interior_f3 = |v: f32| v > 2.0e9;
    let n_f3 = |v: f32| (v - 1024.0).round() as i64;

    eprintln!("Fraktaler-3 cross-check: {file}");
    eprintln!("  {w}×{h}, F3 zoom {f3_zoom:e} (our mag {our_mag:e}), iter {max_iter}, escape_radius {er}");
    eprintln!("  oracle: independent arbitrary-precision CPU dwell ({prec}-bit), bailout² {bailout2}");

    // F3 jitters every sample by a deterministic hash-based triangular sub-pixel offset
    // (anti-aliasing reconstruction, applied even at subframes=1). To compare integer
    // counts exactly we must sample our oracle at F3's *actual* point, not the pixel
    // centre — so replicate the kernel's jitter (hybrid.h: burtle_hash/triangle/wrap;
    // for subframe 0, dx == dy == triangle(burtle_hash(ix)/2³²)).
    let burtle_hash = |mut a: u32| -> u32 {
        a = a.wrapping_add(0x7ed5_5d16).wrapping_add(a << 12);
        a = (a ^ 0xc761_c23c) ^ (a >> 19);
        a = a.wrapping_add(0x1656_67b1).wrapping_add(a << 5);
        a = a.wrapping_add(0xd3a2_646c) ^ (a << 9);
        a = a.wrapping_add(0xfd70_46c5).wrapping_add(a << 3);
        a = (a ^ 0xb55a_4f09) ^ (a >> 16);
        a
    };
    let triangle = |h: f64| -> f64 {
        let orig = h * 2.0 - 1.0;
        let v = (orig / orig.abs().sqrt()).max(-1.0);
        v - if orig >= 0.0 { 1.0 } else { -1.0 }
    };
    let (wi, hi) = (w as i64, h as i64);

    // Pass 1: our oracle escape count at each pixel's exact (jittered) c (interior ⇒ -1).
    let oracle_n: Vec<i64> = (0..(w as usize * h as usize))
        .map(|k| {
            let (x, y) = ((k % w as usize) as i64, (k / w as usize) as i64);
            // Saved EXR is vertically flipped ⇒ kernel (i, j) = (x, h-1-y).
            let (ki, kj) = (x, hi - 1 - y);
            let ix = ((kj * wi + ki) & 0xffff_ffff) as u32;
            let jit = triangle(burtle_hash(ix) as f64 / 4_294_967_296.0);
            let ox = ((ki as f64 + 0.5 + jit) - wf / 2.0) * spacing;
            let oy = ((kj as f64 + 0.5 + jit) - hf / 2.0) * spacing;
            let cx = fractadyne_core::add_f64(cx0, ox, prec);
            let cy = fractadyne_core::add_f64(cy0, oy, prec);
            match fractadyne_core::naive_dwell_bf(&cx, &cy, max_iter, bailout2, prec) {
                Some((n, _)) => n as i64,
                None => -1,
            }
        })
        .collect();

    // Pass 2: compare, excluding ill-conditioned boundary pixels (a 4-neighbour
    // flips interior/exterior, or our oracle's count jumps by >2 — those pixels are
    // sub-pixel-sensitive and the two engines legitimately sample slightly differently).
    let idx = |x: usize, y: usize| y * w as usize + x;
    let (mut interior_ok, mut interior_tot) = (0u64, 0u64);
    let (mut exact, mut within1, mut smooth_tot, mut boundary, mut maxd) = (0u64, 0u64, 0u64, 0u64, 0i64);
    let (mut worst, mut worst_at) = (0i64, (0usize, 0usize));
    for y in 0..h as usize {
        for x in 0..w as usize {
            let k = idx(x, y);
            let o = oracle_n[k];
            let fi = is_interior_f3(nch[k]);
            let oi = o < 0;
            // Boundary detection via oracle neighbourhood.
            let mut steep = false;
            for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let no = oracle_n[idx(nx as usize, ny as usize)];
                if (no < 0) != oi || (no >= 0 && o >= 0 && (no - o).abs() > 2) {
                    steep = true;
                }
            }
            // Exclude ill-conditioned pixels from BOTH metrics. A pixel sitting on the
            // max-iteration cliff (a 4-neighbour flips interior/exterior) is `steep`, so
            // this also keeps near-cliff membership flips — legitimately ambiguous to the
            // last ULP — out of the membership stat.
            if steep {
                boundary += 1;
                continue;
            }
            if fi || oi {
                interior_tot += 1;
                if fi == oi {
                    interior_ok += 1;
                }
                continue;
            }
            smooth_tot += 1;
            let d = (n_f3(nch[k]) - o).abs();
            if d == 0 {
                exact += 1;
            }
            if d <= 1 {
                within1 += 1;
            }
            maxd = maxd.max(d);
            if d > worst {
                worst = d;
                worst_at = (x, y);
            }
        }
    }
    let pct = |a: u64, b: u64| if b == 0 { 100.0 } else { 100.0 * a as f64 / b as f64 };
    eprintln!(
        "  interior/exterior membership: {interior_ok}/{interior_tot} agree ({:.3}%)",
        pct(interior_ok, interior_tot)
    );
    eprintln!(
        "  smooth-region exterior counts: exact {exact}/{smooth_tot} ({:.3}%), |Δ|≤1 {within1}/{smooth_tot} ({:.3}%)",
        pct(exact, smooth_tot),
        pct(within1, smooth_tot)
    );
    eprintln!("  max |Δn| (non-boundary) {maxd} at pixel {worst_at:?}; boundary pixels excluded {boundary}");
    // PASS: membership ≥99.5%, and ≥99% of smooth-region exterior pixels match within 1.
    let pass = pct(interior_ok, interior_tot) >= 99.5 && pct(within1, smooth_tot) >= 99.0;
    println!("crosscheck-f3: {}", if pass { "PASS" } else { "FAIL" });
    crate::exit(if pass { 0 } else { 1 });
}
