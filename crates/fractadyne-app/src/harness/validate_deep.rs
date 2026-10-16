//! `--validate-deep`: the extreme-depth precision self-consistency battery (1e1000 to 1e1000000x). Dispatched from `cli::run_headless`.

use crate::version_string;

pub(crate) fn run(args: &[String]) -> bool {
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
        "{:>12} {:>11} {:>7} {:>7} {:>13} {:>13} {:>9}  result",
        "magnif.", "bits", "limbs", "k", "agree(bits)", "rt(bits)", "time(s)"
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
                 arbitrary-precision dwell oracle is computationally infeasible this deep, and \
                 rendering a specific feature at these magnitudes would need a center coordinate of \
                 roughly that many decimal digits. (The live renderer's scale is an extended-range \
                 `FloatExp`, not `f64`, so there is no 1e308× scale wall — the practical limit is \
                 coordinate precision + iteration/compute cost.) Independent per-pixel cross-checks \
                 (`--selftest`, `--crosscheck-f3`) cover the renderable depth range.\n",
        );
        if let Err(e) = std::fs::write(&path, md) {
            eprintln!("report write failed: {e}");
        } else {
            println!("report → {}", path.display());
        }
    }
    println!("validate-deep: {}", if all_ok { "PASS" } else { "FAIL" });
    crate::exit(if all_ok { 0 } else { 1 });
}
