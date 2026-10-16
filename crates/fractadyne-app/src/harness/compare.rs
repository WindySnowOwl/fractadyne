//! `--compare A B`: diff two renders or exported iteration files (no GPU). Dispatched from `cli::run_headless`.

pub(crate) fn run(args: &[String], i: usize) -> bool {
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
            Some("exr") => fractadyne_export::read_exr_rgba_f32(path).ok(),
            Some("png") => fractadyne_export::read_png_rgba8(path)
                .ok()
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
