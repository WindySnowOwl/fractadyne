//! Headless CLI modes, run before the GUI starts: --find-minibrot, --compare,
//! --crosscheck-f3, and --validate-deep. `run_headless` returns true when it handled a
//! mode (so `main` exits Ok); the validation modes set a process exit code directly.

use crate::{version_string, FractalKind};

/// Dispatch the headless CLI modes. Returns true if one ran (caller should exit).
pub(crate) fn run_headless(args: &[String]) -> bool {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print!("{}", crate::help::cli_help_text());
        return true;
    }

    if args.iter().any(|a| a == "--find-minibrot") {
        let val = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1));
        let two = |name: &str| {
            args.iter()
                .position(|a| a == name)
                .and_then(|i| Some((args.get(i + 1)?, args.get(i + 2)?)))
        };
        let formula = val("--fractal")
            .and_then(|s| FractalKind::from_name(s))
            .map(|k| k.formula_id())
            .unwrap_or(0);
        let center = two("--center")
            .and_then(|(x, y)| Some([fractadyne_core::parse_bf(x)?, fractadyne_core::parse_bf(y)?]))
            .unwrap_or([
                fractadyne_core::BigFloat::from_f64(-0.5, 64),
                fractadyne_core::BigFloat::from_f64(0.0, 64),
            ]);
        let mag = val("--zoom").and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0);
        match fractadyne_core::find_nucleus(&center, mag, formula, 100_000) {
            Some(n) => println!(
                "period {}\ncenter_x {}\ncenter_y {}",
                n.period,
                fractadyne_core::to_decimal_string(&n.cx),
                fractadyne_core::to_decimal_string(&n.cy),
            ),
            None => println!("no minibrot center found"),
        }
        return true;
    }

    // Headless A/B comparison (no GPU): diff two renders / exported iteration files.
    //   --compare A B [--out heatmap.png]
    if let Some(i) = args.iter().position(|a| a == "--compare") {
        let (a, b) = (args.get(i + 1), args.get(i + 2));
        let (Some(a), Some(b)) = (a, b) else {
            eprintln!("--compare needs two file paths");
            return true;
        };
        let out = args
            .iter()
            .position(|x| x == "--out" || x == "-o")
            .and_then(|j| args.get(j + 1))
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("compare_heatmap.png"));
        let load = |p: &str| -> Option<(u32, u32, Vec<f32>)> {
            let path = std::path::Path::new(p);
            match path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref() {
                Some("exr") => fractadyne_export::read_exr_rgba_f32(path),
                Some("png") => fractadyne_export::read_png_rgba8(path)
                    .map(|(w, h, bytes)| (w, h, bytes.iter().map(|&x| x as f32).collect())),
                _ => None,
            }
        };
        match (load(a), load(b)) {
            (Some((wa, ha, pa)), Some((wb, hb, pb))) if wa == wb && ha == hb => {
                let n = (wa as usize) * (ha as usize);
                // Channel 0 (smooth iteration for EXR, red for PNG) is the primary signal.
                // Channel 0 (smooth iteration / red) is always finite. The DE/normal
                // channels carry ±∞/1e30 sentinels for interior/unavailable, so the
                // all-channel stats skip non-finite and sentinel-scale diffs.
                let (mut max0, mut sum0, mut differ) = (0.0f64, 0.0f64, 0u64);
                let (mut maxall, mut sumall, mut nall) = (0.0f64, 0.0f64, 0u64);
                for k in 0..n {
                    let d0 = (pa[k * 4] - pb[k * 4]).abs() as f64;
                    max0 = max0.max(d0);
                    sum0 += d0;
                    if d0 > 1e-6 {
                        differ += 1;
                    }
                    for c in 0..4 {
                        let d = (pa[k * 4 + c] - pb[k * 4 + c]).abs() as f64;
                        if d.is_finite() && d < 1.0e20 {
                            maxall = maxall.max(d);
                            sumall += d;
                            nall += 1;
                        }
                    }
                }
                println!("Comparison: {a}  vs  {b}");
                println!("  size {wa}×{ha}");
                println!("  channel 0: max Δ {max0:.6}, mean Δ {:.6}, {differ}/{n} pixels differ", sum0 / n as f64);
                println!("  all channels (finite): max Δ {maxall:.6}, mean Δ {:.6}", sumall / nall.max(1) as f64);
                // Heatmap of |Δ channel 0|, normalized to the max (grayscale).
                let scale = if max0 > 0.0 { 1.0 / max0 as f32 } else { 0.0 };
                let mut heat = vec![0.0f32; n * 4];
                for k in 0..n {
                    let t = (pa[k * 4] - pb[k * 4]).abs() * scale;
                    heat[k * 4] = t;
                    heat[k * 4 + 1] = t;
                    heat[k * 4 + 2] = t;
                    heat[k * 4 + 3] = 1.0;
                }
                match fractadyne_export::write_png(&out, wa, ha, &heat, None) {
                    Ok(()) => println!("  heatmap → {}", out.display()),
                    Err(e) => eprintln!("  heatmap write failed: {e}"),
                }
            }
            (Some((wa, ha, _)), Some((wb, hb, _))) => {
                eprintln!("dimension mismatch: {wa}×{ha} vs {wb}×{hb}");
                return true;
            }
            _ => {
                eprintln!("failed to load one or both inputs (PNG/EXR only)");
                return true;
            }
        }
        return true;
    }

    // Cross-renderer validation against **Fraktaler-3** (no GPU): compare F3's raw
    // integer escape counts (EXR channel "N", UINT) against our *independent* CPU
    // arbitrary-precision dwell oracle at the identical complex coordinate of every
    // pixel. Two fully independent engines (F3's GPU perturbation vs our bignum CPU)
    // agreeing on exact integer iteration counts is the strongest external check.
    //
    //   --crosscheck-f3 raw.exr --center X Y --zoom-f3 Z [--iter K] [--er 256]
    //
    // Render the F3 side with (in a .f3.toml batch): render.save_exr = true,
    // render.exr_channels = ["N0"], image.subframes = 1, transform.exponential_map = false.
    if let Some(i) = args.iter().position(|a| a == "--crosscheck-f3") {
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
        let center = two("--center")
            .and_then(|(x, y)| Some([fractadyne_core::parse_bf(x)?, fractadyne_core::parse_bf(y)?]))
            .unwrap_or([
                fractadyne_core::BigFloat::from_f64(-0.5, 64),
                fractadyne_core::BigFloat::from_f64(0.0, 64),
            ]);
        let f3_zoom = val("--zoom-f3").and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0);
        let max_iter = val("--iter").and_then(|s| s.parse::<u32>().ok()).unwrap_or(10_000);
        let er = val("--er").and_then(|s| s.parse::<f64>().ok()).unwrap_or(256.0);
        let bailout2 = er * er;
        // Our magnification convention (height 3) vs F3's (height 4): mag = 0.75·zoom.
        let our_mag = 0.75 * f3_zoom;
        let prec = fractadyne_core::precision_for_magnification(our_mag).max(64);

        let Some((w, h, nch)) = fractadyne_export::read_exr_channel_f32(std::path::Path::new(file), "N")
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
        std::process::exit(if pass { 0 } else { 1 });
    }

    // Extreme-depth validation battery (no GPU, no external data): exercises the
    // arbitrary-precision arithmetic core at magnifications far beyond f64 range
    // (1e1000 … 1e1000000), via precision-doubling self-consistency + coordinate round-trip.
    // A per-pixel dwell oracle is infeasible this deep; these single-point checks are not.
    //   --validate-deep [--out report.md]
    if args.iter().any(|a| a == "--validate-deep") {
        use std::time::Instant;
        let out = args
            .iter()
            .position(|x| x == "--out" || x == "-o")
            .and_then(|j| args.get(j + 1))
            .map(std::path::PathBuf::from);
        // (decimal exponent, iteration count) — fewer iters as precision grows (cost ∝ bits·k).
        let battery: &[(f64, u32)] = &[(1_000.0, 20_000), (10_000.0, 4_000), (100_000.0, 800), (1_000_000.0, 200)];
        let guard = 256usize;
        let mut rows: Vec<String> = Vec::new();
        let mut all_ok = true;
        println!("Extreme-depth precision self-consistency (arbitrary-precision arithmetic core)");
        println!(
            "{:>12} {:>11} {:>7} {:>7} {:>13} {:>13} {:>9}  {}",
            "magnif.", "bits", "limbs", "k", "agree(bits)", "rt(bits)", "time(s)", "result"
        );
        for (exp, k) in battery.iter().copied() {
            let octaves = (exp * std::f64::consts::LN_10 / std::f64::consts::LN_2).ceil() as u64;
            let p = fractadyne_core::precision_for_octaves(octaves);
            let t = Instant::now();
            let agree = fractadyne_core::deep_consistency_bits(p, guard, k);
            let rt = fractadyne_core::deep_roundtrip_bits(p);
            let secs = t.elapsed().as_secs_f64();
            // Sound p-bit arithmetic agrees to ≈ p − log₂(k); allow a generous margin.
            let pass = agree >= p as i64 - 128 && rt >= p as i64 - 256;
            all_ok &= pass;
            let verdict = if pass { "PASS" } else { "FAIL" };
            println!(
                "      1e{:<5.0} {:>11} {:>7} {:>7} {:>13} {:>13} {:>9.2}  {}",
                exp, p, p / 64, k, agree, rt, secs, verdict
            );
            rows.push(format!(
                "| 1e{:.0} | {} | {} | {} | {} | {} | {:.2} | {} |",
                exp, p, p / 64, k, agree, rt, secs, verdict
            ));
        }
        if let Some(path) = out {
            let mut md = String::new();
            md.push_str("# Extreme-depth precision validation\n\n");
            md.push_str(&format!("Fractadyne {}\n\n", version_string()));
            md.push_str(
                "Precision-doubling self-consistency of the arbitrary-precision arithmetic core, at \
                 magnifications beyond `f64` range. Iterate `z²+c` (full-mantissa interior point) at \
                 precision `p` and at `p+256`; `agree` = leading base-2 bits that match (sound ≈ \
                 `p − log₂(k)`). `rt` = bits preserved through a decimal `to_string → parse` \
                 round-trip. No GPU, no external data.\n\n",
            );
            md.push_str("| magnification | bits | limbs | k iters | agree (bits) | round-trip (bits) | time (s) | result |\n");
            md.push_str("|---|---|---|---|---|---|---|---|\n");
            for r in &rows {
                md.push_str(r);
                md.push('\n');
            }
            md.push_str(&format!("\n**Overall: {}**\n", if all_ok { "PASS" } else { "FAIL" }));
            md.push_str(
                "\n## Scope\n\nThis validates the *arithmetic and precision machinery* at extreme \
                 bit-width (the depth-critical numerics), not a full rendered image: a per-pixel \
                 arbitrary-precision dwell oracle is computationally infeasible this deep, and the \
                 renderer's `f64` `units_per_pixel` caps live zoom near 1e308× regardless. \
                 Independent per-pixel cross-checks (`--selftest`, `--crosscheck-f3`) cover the \
                 renderable depth range.\n",
            );
            if let Err(e) = std::fs::write(&path, md) {
                eprintln!("report write failed: {e}");
            } else {
                println!("report → {}", path.display());
            }
        }
        println!("validate-deep: {}", if all_ok { "PASS" } else { "FAIL" });
        std::process::exit(if all_ok { 0 } else { 1 });
    }
    false
}

