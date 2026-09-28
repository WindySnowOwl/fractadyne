use super::*;

fn table() -> Table {
    parse(TABLE).expect("validation/calibration/ceilings.toml must parse — it is compiled in")
}

#[test]
fn slug_is_the_lowercase_hyphenated_adapter_name() {
    assert_eq!(slug("AMD Radeon RX 6800 XT"), "amd-radeon-rx-6800-xt");
    assert_eq!(slug("NVIDIA GeForce RTX 3080"), "nvidia-geforce-rtx-3080");
    // Punctuation and runs of it collapse; the ends are trimmed.
    assert_eq!(slug("  Intel(R) Arc(TM) A770  Graphics "), "intel-r-arc-tm-a770-graphics");
}

#[test]
fn the_calibrated_cards_are_found_and_anything_else_gets_the_default() {
    let t = table();
    for name in ["NVIDIA GeForce RTX 3080", "AMD Radeon RX 6800 XT"] {
        let c = lookup(&t, name);
        assert!(c.source.starts_with("calibrated for"), "{name}: {}", c.source);
        assert!(c.costs.df32_pert > 0.0 && c.costs.direct > 0.0 && c.knee_px > 0, "{name}: {c:?}");
    }
    let unknown = lookup(&t, "Some Future GPU 9000");
    assert!(unknown.source.starts_with("default"), "{}", unknown.source);
    assert!(unknown.costs.df32_pert > 0.0 && unknown.costs.direct > 0.0);
}

/// An adapter the table has never seen must never get a HIGHER ceiling than a card that was
/// measured: the default's costs are at least every calibrated card's, and its knee at least as
/// large.
#[test]
fn the_default_is_at_least_as_conservative_as_every_calibrated_card() {
    let t = table();
    let d = lookup(&t, "no such adapter");
    for name in ["NVIDIA GeForce RTX 3080", "AMD Radeon RX 6800 XT"] {
        let c = lookup(&t, name);
        assert!(d.costs.df32_pert >= c.costs.df32_pert, "{name}");
        assert!(d.costs.direct >= c.costs.direct, "{name}");
        assert!(d.knee_px >= c.knee_px, "{name}");
    }
}

/// Every formula has a measured row, so adding a formula forces a decision about its cost rather
/// than silently pricing it as Mandelbrot; and every row names a real formula.
#[test]
fn every_formula_has_a_factor_row_and_every_row_is_a_formula() {
    let t = table();
    for spec in FractalKind::SPECS {
        assert!(
            t.formula.iter().any(|f| f.name == spec.name),
            "no [[formula]] row for {:?} in validation/calibration/ceilings.toml",
            spec.name
        );
    }
    for f in &t.formula {
        assert!(FractalKind::from_name(&f.name).is_some(), "row {:?} names no formula", f.name);
    }
}

#[test]
fn a_factor_below_one_never_lifts_the_ceiling() {
    let t = table();
    assert_eq!(factor(&t, FractalKind::Mandelbrot, RenderMode::Df32Pert), 1.0);
    // Tricorn measures 0.39 in df32 perturbation; it prices as Mandelbrot.
    assert_eq!(factor(&t, FractalKind::Tricorn, RenderMode::Df32Pert), 1.0);
    // Multibrot 5 is the costliest direct formula measured.
    assert!((factor(&t, FractalKind::Multibrot5, RenderMode::Direct) - 1.77).abs() < 1e-9);
    for spec in FractalKind::SPECS {
        for mode in [RenderMode::Direct, RenderMode::Df32Pert] {
            assert!(factor(&t, spec.kind, mode) >= 1.0);
        }
    }
}

/// The definition: a ceiling-sized dispatch at the calibrated cost takes the target.
#[test]
fn a_ceiling_sized_dispatch_at_the_calibrated_cost_takes_the_target() {
    let cost = 4.95e-8;
    let c = ceiling(cost, 1.0, 524_288, 4_000_000).unwrap();
    let ms = c as f64 * cost;
    assert!((ms - TARGET_MS).abs() < 0.01, "{ms} ms");
    // A formula 1.77x as costly gets 1/1.77 the steps, and still takes the target.
    let c5 = ceiling(cost, 1.77, 524_288, 4_000_000).unwrap();
    assert!((c5 as f64 * cost * 1.77 - TARGET_MS).abs() < 0.01);
}

/// Below the knee the GPU is not saturated and a dispatch costs its WINDOW, so the ceiling scales
/// with the pixel count there and is flat above it.
#[test]
fn below_the_knee_the_ceiling_scales_with_pixels() {
    let full = ceiling(2.76e-8, 1.0, 262_144, 262_144).unwrap();
    assert_eq!(ceiling(2.76e-8, 1.0, 262_144, 4_000_000).unwrap(), full);
    let half = ceiling(2.76e-8, 1.0, 262_144, 131_072).unwrap();
    assert!((half as f64 / full as f64 - 0.5).abs() < 1e-6, "{half} vs {full}");
    // A tiny dispatch still gets a positive ceiling, never zero.
    assert!(ceiling(2.76e-8, 1.0, 262_144, 1).unwrap() >= 1);
}

#[test]
fn an_unmeasured_cost_is_no_ceiling_and_capping_never_raises() {
    assert_eq!(ceiling(0.0, 1.0, 262_144, 1_000_000), None);
    assert_eq!(ceiling(f64::NAN, 1.0, 262_144, 1_000_000), None);
    assert_eq!(capped(5_000, None), 5_000);
    assert_eq!(capped(5_000, Some(9_000)), 5_000);
    assert_eq!(capped(5_000, Some(3_000)), 3_000);
}

/// The numbers the field cards run with, pinned so a table edit that moves them is a visible
/// decision: at a saturated size, Mandelbrot.
#[test]
fn the_calibrated_ceilings_are_the_ones_measured() {
    let t = table();
    let at = |name: &str, df32: bool| {
        let c = lookup(&t, name);
        ceiling(if df32 { c.costs.df32_pert } else { c.costs.direct }, 1.0, c.knee_px, 4_000_000).unwrap()
            as f64
    };
    let near = |v: f64, want: f64| (v / want - 1.0).abs() < 0.01;
    assert!(near(at("AMD Radeon RX 6800 XT", true), 8.08e9), "{}", at("AMD Radeon RX 6800 XT", true));
    assert!(near(at("NVIDIA GeForce RTX 3080", true), 1.449e10));
    assert!(near(at("NVIDIA GeForce RTX 3080", false), 7.48e10));
}

/// Floatexp has no step ceiling (BLA), and the perturbation and direct modes do, whichever adapter
/// this process resolved.
#[test]
fn floatexp_has_no_ceiling_and_the_other_modes_do() {
    init("NVIDIA GeForce RTX 3080");
    assert_eq!(ceiling_for(RenderMode::Floatexp, FractalKind::Mandelbrot, 4_000_000), None);
    assert!(ceiling_for(RenderMode::Df32Pert, FractalKind::Mandelbrot, 4_000_000).is_some());
    assert!(ceiling_for(RenderMode::Direct, FractalKind::Mandelbrot, 4_000_000).is_some());
    // The costliest formula gets the lower ceiling.
    assert!(
        ceiling_for(RenderMode::Direct, FractalKind::Multibrot5, 4_000_000)
            < ceiling_for(RenderMode::Direct, FractalKind::Mandelbrot, 4_000_000)
    );
}
