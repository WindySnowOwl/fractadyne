//! `--pickcheck`: the reference-pick acceptance harness - both phase-2 scoring engines must elect the same reference. Dispatched from `cli::run_headless`.

use crate::FractalKind;

pub(crate) fn run(args: &[String]) -> bool {
    use fractadyne_core as fc;
    let val = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1));
    let (w, h) = val("--size")
        .map(|s| {
            s.split_once('x')
                .and_then(|(a, b)| Some((a.parse::<u32>().ok()?, b.parse::<u32>().ok()?)))
                .unwrap_or_else(|| {
                    eprintln!("--size wants WxH (e.g. 1280x720), got '{s}'");
                    crate::exit(1);
                })
        })
        .unwrap_or((1280, 720)); // the corpus render geometry
    let iter_override = val("--iter").map(|s| crate::arg_parse::<u32>("--iter", s, "a whole number"));
    // One rung: a location file, or the built-in e89 dendrite (the misiurewicz_outcomes
    // fixture — kept verbatim from crates/fractadyne-core/tests/misiurewicz_outcomes.rs,
    // the one ladder depth with no committed .fdn).
    enum Rung {
        File(&'static str),
        Given(String),
        Builtin { name: &'static str, cx: &'static str, cy: &'static str, log2mag: f64, iter: u32 },
    }
    const E89_CX: &str = "2.336541879936817878966215410838761608133707885474547567506859621515201675599324293839217233261500414218604921872973e-2";
    const E89_CY: &str = "8.2741173753632652070456275296875144057156928752279993918651650950463440957323066581594346138419917471379151772617142e-1";
    let ladder: Vec<Rung> = match val("--pickcheck").filter(|s| !s.starts_with('-')) {
        Some(path) => vec![Rung::Given(path.to_string())],
        None => vec![
            Rung::File("validation/corpus/locations/05-dendrite-8e17.fdn"),
            Rung::File("validation/corpus/locations/06-seahorse-1e24.fdn"),
            Rung::File("validation/corpus/locations/08-deep-6.6e43.fdn"),
            Rung::Builtin {
                name: "e89-dendrite (misiurewicz_outcomes)",
                cx: E89_CX,
                cy: E89_CY,
                log2mag: 297.12071983391957, // 2.77e89×
                iter: 100_000,
            },
            Rung::File("validation/corpus/locations/14-deep-1.2e148.fdn"),
            Rung::File("validation/corpus/locations/09-deep-6.1e500.fdn"),
            Rung::File("validation/corpus/locations/19-deep-1.3e726.fdn"),
            Rung::File("validation/corpus/locations/20-deep-1.2e1008.fdn"),
            Rung::File("validation/e4000-misiurewicz.fdn"),
        ],
    };
    let (mut matched, mut mismatched, mut skipped) = (0u32, 0u32, 0u32);
    println!("pickcheck: dual-run pick equivalence (walk vs perturb), {w}x{h}, {} rung(s)", ladder.len());
    for (ri, rung) in ladder.iter().enumerate() {
        // Resolve the rung to (name, cx, cy, log2mag, iter, formula, julia). A file that
        // cannot supply a usable input is a loud SKIP, never a silent default.
        let path: Option<&str> = match rung {
            Rung::File(p) => Some(p),
            Rung::Given(s) => Some(s.as_str()),
            Rung::Builtin { .. } => None,
        };
        let parsed: Result<(String, String, String, f64, u32, u32, bool), String> = match rung {
            Rung::Builtin { name, cx, cy, log2mag, iter } => Ok((
                (*name).to_string(),
                (*cx).to_string(),
                (*cy).to_string(),
                *log2mag,
                iter_override.unwrap_or(*iter),
                0,
                false,
            )),
            _ => {
                let p = path.unwrap();
                (|| {
                    let text = std::fs::read_to_string(p).map_err(|e| format!("{p}: {e}"))?;
                    let get = |k: &str| crate::meta_get(&text, k);
                    let upp_log2 = match get("upp_log2").parse::<f64>() {
                        Ok(v) if v.is_finite() => v,
                        _ => return Err(format!("{p}: no readable upp_log2 (depth) field")),
                    };
                    let iter = match iter_override.or_else(|| get("max_iter").parse::<u32>().ok()) {
                        Some(i) if i > 0 => i,
                        _ => return Err(format!("{p}: no readable max_iter (pass --iter N)")),
                    };
                    let formula = match get("fractal") {
                        s if s.is_empty() => 0, // absent → Mandelbrot, as the app's loader defaults
                        s => FractalKind::from_name(&s)
                            .map(|k| k.formula_id())
                            .ok_or_else(|| format!("{p}: unknown fractal '{s}'"))?,
                    };
                    // log2(magnification) from the stored per-pixel scale, at THIS render
                    // height: mag = REFERENCE_HEIGHT / (height · upp) — mirrors Viewport.
                    let log2mag = 2.0 - (h as f64).log2() - upp_log2;
                    Ok((p.to_string(), get("center_re"), get("center_im"), log2mag, iter, formula, get("julia") == "1"))
                })()
            }
        };
        let (name, cx_s, cy_s, log2mag, iter, formula, julia) = match parsed {
            Ok(t) => t,
            Err(why) => {
                println!("  [{}] SKIP {why}", ri + 1);
                skipped += 1;
                continue;
            }
        };
        // Inputs exactly as build_params derives them for a pick at this view.
        let mut vp = fc::Viewport::new(w as f64, h as f64);
        let precision = fc::precision_for_octaves(log2mag.max(0.0).ceil() as u64);
        let (cx, cy) = match (fc::parse_bf_prec(&cx_s, precision), fc::parse_bf_prec(&cy_s, precision)) {
            (Some(x), Some(y)) => (x, y),
            _ => {
                println!("  [{}] SKIP {name}: centre coordinates did not parse", ri + 1);
                skipped += 1;
                continue;
            }
        };
        vp.set_center_log2mag(cx.clone(), cy.clone(), log2mag);
        let gpu_iter = iter.min(crate::MAX_ITER_LIMIT).min(crate::zoom_iter_cap(log2mag).max(256));
        let span = vp.complex_span_fe();
        println!(
            "  [{}/{}] {name} @1e{:.1}x prec={precision}b iter={gpu_iter}",
            ri + 1,
            ladder.len(),
            log2mag / std::f64::consts::LOG2_10,
        );
        match fc::best_reference_dual(&[cx, cy], [span.0, span.1], formula, julia, [0.0, 0.0], gpu_iter, precision) {
            Err(why) => {
                println!("      SKIP: {why}");
                skipped += 1;
            }
            Ok(d) => {
                println!(
                    "      walk    {:8.2}s winner_len={} survivors={} scored={}{}{}",
                    d.walk_secs,
                    d.walk_diag.winner_len,
                    d.walk_diag.survivors,
                    d.walk_diag.deep_scored,
                    d.walk_diag.rescued.map(|r| format!(" RESCUED={r}")).unwrap_or_default(),
                    if d.walk_diag.fallback_escaper { " FALLBACK-ESCAPER" } else { "" },
                );
                println!(
                    "      perturb {:8.2}s winner_len={} survivors={} scored={} rb={} fb={}{}{}{}",
                    d.perturb_secs,
                    d.perturb_diag.winner_len,
                    d.perturb_diag.survivors,
                    d.perturb_diag.deep_scored,
                    d.perturb_diag.perturb_rebases,
                    d.perturb_diag.perturb_fallbacks,
                    d.perturb_diag.rescued.map(|r| format!(" RESCUED={r}")).unwrap_or_default(),
                    if d.perturb_diag.fallback_escaper { " FALLBACK-ESCAPER" } else { "" },
                    if d.perturb_diag.scan_perturb {
                        format!(" scan=perturb(rewalked={})", d.perturb_diag.scan_rewalked)
                    } else {
                        String::new()
                    },
                );
                // Phase 1 by perturbation must keep the bignum scan's survivors, not just the winner.
                let same_survivors = d.walk_diag.survivors == d.perturb_diag.survivors;
                if !same_survivors {
                    println!(
                        "      MISMATCH: phase 1 kept {} survivors by perturbation, {} by the bignum scan",
                        d.perturb_diag.survivors, d.walk_diag.survivors
                    );
                    mismatched += 1;
                } else if d.identical {
                    let speedup = if d.perturb_secs > 0.0 { d.walk_secs / d.perturb_secs } else { 0.0 };
                    println!("      MATCH ({speedup:.2}x)");
                    matched += 1;
                } else {
                    let trunc = |b: &fc::BigFloat| {
                        let s = fc::to_decimal_string(b);
                        s.chars().take(64).collect::<String>()
                    };
                    println!("      MISMATCH:");
                    println!("        walk    ({}, {})", trunc(&d.walk[0]), trunc(&d.walk[1]));
                    println!("        perturb ({}, {})", trunc(&d.perturb[0]), trunc(&d.perturb[1]));
                    mismatched += 1;
                }
            }
        }
    }
    println!("pickcheck: {matched} matched, {mismatched} mismatched, {skipped} skipped");
    if mismatched > 0 {
        crate::exit(1);
    } else if skipped > 0 || matched == 0 {
        crate::exit(2);
    }
    true
}
