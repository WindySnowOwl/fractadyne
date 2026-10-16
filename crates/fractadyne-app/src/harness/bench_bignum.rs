//! `--bench-bignum`: reference-orbit cost per precision, and which bignum backend ran (CPU only). Dispatched from `cli::run_headless`.

pub(crate) fn run(args: &[String]) -> bool {
    use std::time::Instant;
    let val = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1));
    // ⚠ABSENT and SUPPLIED-BUT-UNREADABLE must not collapse into one default — that is the
    // silent-CLI-default class closed in `01add37`. `--iters 1O` (letter O) must be a message,
    // not a table of numbers for a run nobody asked for. `arg_parse` is the shared helper.
    let scale: f64 = match val("--iters") {
        None => 1.0,
        Some(s) => crate::arg_parse("--iters", s, "a positive number"),
    };
    if !scale.is_finite() || scale <= 0.0 {
        eprintln!("fractadyne: --iters must be positive and finite (got {scale}).");
        crate::exit(2);
    }
    // An INTERIOR point (well inside the main cardioid), irrational so every limb is
    // populated and the multiplies do real carry work — a dyadic centre like -0.5 would
    // produce sparse mantissas and understate the cost.
    //
    // ⚠Interior, not boundary, and that choice is load-bearing: an interior orbit cannot
    // escape at ANY precision, so every row does the full `iters` of real bignum work. A
    // boundary centre escaped at 64 bits (where truncation makes it a different point) and
    // spent the rest of the row on infinities — fast, meaningless, and it made the whole
    // command exit 1 permanently. The escape guard below stays as the tripwire; it was seen
    // firing on exactly that case before this centre was chosen.
    const CX: &str = "-0.287935866751537269847162833107461";
    const CY: &str = "0.007071067811865475244008443621048";

    let backends = fractadyne_core::available_backends();
    println!("Reference-orbit cost per arbitrary-precision backend (CPU only, no GPU)");
    println!("build contains: {}", fractadyne_core::built_in_backends());
    println!(
        "comparing: {}",
        backends.iter().map(|b| b.name()).collect::<Vec<_>>().join(" vs ")
    );
    println!();

    let mut header = format!("{:>8} {:>7} {:>10} {:>14}", "bits", "words", "iters", "bounded");
    for b in &backends {
        header.push_str(&format!(" {:>14}", format!("{} ns/it", b.name())));
    }
    if backends.len() > 1 {
        header.push_str(&format!(" {:>9} {:>16}", "speedup", "identical?"));
    }
    println!("{header}");

    let mut any_unbounded = false;
    let mut any_divergence = false;
    let mut worst: f64 = f64::MAX;
    let mut best: f64 = 0.0;
    for &bits in &[64usize, 128, 256, 576, 1088, 2112, 3776, 8256] {
        // Fewer iterations as the precision (and so the per-step cost) grows.
        let base = if bits <= 256 {
            60_000.0
        } else if bits <= 1088 {
            20_000.0
        } else if bits <= 3776 {
            4_000.0
        } else {
            1_500.0
        };
        let iters = ((base * scale) as u32).max(16);
        let (Some(cx), Some(cy)) =
            (fractadyne_core::parse_bf_prec(CX, bits), fractadyne_core::parse_bf_prec(CY, bits))
        else {
            eprintln!("fractadyne: could not parse the benchmark centre at {bits} bits");
            crate::exit(2);
        };
        let z0 = fractadyne_core::BigFloat::from_f64(0.0, bits);

        // Every backend runs the SAME work in ONE process, warm-up then median of three.
        // Run-to-run variance on these is real, and a comparison across two builds would be
        // measuring the builds as much as the backends.
        let mut ns: Vec<f64> = Vec::with_capacity(backends.len());
        let mut orbits: Vec<Vec<[f32; 4]>> = Vec::with_capacity(backends.len());
        let mut mag = 0.0f64;
        let mut len = 0u32;
        for &b in &backends {
            let mut ts = Vec::with_capacity(3);
            let mut last_orbit = Vec::new();
            for rep in 0..4 {
                let t = Instant::now();
                let (orbit, l) = fractadyne_core::reference_orbit_in(
                    b,
                    &z0,
                    &z0,
                    &cx,
                    &cy,
                    fractadyne_core::formula::MANDELBROT,
                    iters,
                    bits,
                );
                let dt = t.elapsed().as_secs_f64();
                if rep > 0 {
                    ts.push(dt);
                }
                len = l;
                let last = orbit.last().copied().unwrap_or([0.0; 4]);
                let (x, y) = fractadyne_core::sample_xy(&last);
                mag = (x * x + y * y).sqrt();
                last_orbit = orbit;
            }
            ts.sort_by(|a, b| a.partial_cmp(b).unwrap());
            ns.push(ts[ts.len() / 2] * 1.0e9 / len.max(1) as f64);
            orbits.push(last_orbit);
        }

        // A speed ratio between backends that computed DIFFERENT orbits is meaningless, and
        // the ratio is the number people quote. Assert identity per row rather than trusting
        // the unit tests to have happened to cover this precision.
        let identical = orbits.windows(2).all(|w| {
            w[0].len() == w[1].len()
                && w[0].iter().zip(&w[1]).all(|(a, b)| {
                    a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
                })
        });
        any_divergence |= !identical;

        // `reference_orbit` stops early on escape, so a short orbit IS the escape signal.
        let bounded = len >= iters && mag.is_finite() && mag <= 1.0e6;
        any_unbounded |= !bounded;
        let flag = if bounded { format!("|z|={mag:.2}") } else { format!("ESCAPED@{len}") };

        let mut line =
            format!("{:>8} {:>7} {:>10} {:>14}", bits, bits.div_ceil(64), len, flag);
        for v in &ns {
            line.push_str(&format!(" {v:>14.1}"));
        }
        if backends.len() > 1 {
            let sp = ns[0] / ns[ns.len() - 1];
            worst = worst.min(sp);
            best = best.max(sp);
            line.push_str(&format!(
                " {:>8.2}x {:>16}",
                sp,
                if identical { "BYTE-IDENTICAL" } else { "*** DIFFERS ***" }
            ));
        }
        println!("{line}");
        if !bounded {
            println!(
                "         ^^ INVALID ROW - an escaped orbit times infinity arithmetic, not bignum"
            );
        }
    }

    if backends.len() > 1 && worst.is_finite() {
        println!();
        println!(
            "speedup range ({} over {}): {worst:.2}x .. {best:.2}x",
            backends[backends.len() - 1].name(),
            backends[0].name()
        );
    }

    // Printed AFTER the work, from what actually iterated. `MIXED` is the CORRECT answer for
    // this command and a failure for `--selftest`: a comparison must exercise every backend,
    // whereas a verdict must be attributable to exactly one.
    println!();
    println!("backends exercised: {}", fractadyne_core::backend_status_line());
    println!("tunables: {}", crate::tunables::status_line());
    if any_divergence {
        println!();
        println!("BACKENDS DISAGREED - the ratios above compare different computations");
        crate::exit(1);
    }
    if any_unbounded {
        println!("\nat least one row is INVALID (see above) — do not quote this table");
        crate::exit(1);
    }
    crate::exit(0);
}
