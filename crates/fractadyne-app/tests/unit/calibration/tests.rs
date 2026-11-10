use super::*;

/// [`factor_of`] over a whole table; a test keeps every formula in the table.
fn factor(t: &Table, formula: FractalKind, mode: RenderMode) -> f64 {
    factor_of(&t.formula, formula, mode)
}

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

/// The fixed per-pass cost: the default has one, a calibrated card's own value wins, and a card
/// without one falls back to the default's — never to nothing.
#[test]
fn the_fixed_pass_cost_is_the_cards_own_else_the_default() {
    let t = table();
    let d = lookup(&t, "no such adapter");
    assert!(d.pass_fixed_ms.is_some_and(|v| v > 0.0), "the default carries a pass_fixed_ms");
    assert!(!d.pass_fixed_calibrated);
    for name in ["NVIDIA GeForce RTX 3080", "AMD Radeon RX 6800 XT"] {
        let c = lookup(&t, name);
        assert!(c.pass_fixed_ms.is_some_and(|v| v > 0.0), "{name}");
        if c.pass_fixed_calibrated {
            // A measured value is a measurement, not the default copied in.
            assert!(t.adapter.iter().any(|e| e.slug == slug(name) && e.pass_fixed_ms.is_some()), "{name}");
        } else {
            assert_eq!(c.pass_fixed_ms, d.pass_fixed_ms, "{name}");
        }
    }
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

/// Every built-in formula has a measured row, so adding a formula forces a decision about its cost
/// rather than silently pricing it as Mandelbrot; and every row names a real formula. (`Custom` is
/// priced by its own estimate — see the test below.)
#[test]
fn every_formula_has_a_factor_row_and_every_row_is_a_formula() {
    let t = table();
    // The automata dispatch no escape-time pass, so they have no cost row either.
    for spec in FractalKind::SPECS.iter().filter(|s| s.kind != FractalKind::Custom && s.kind.is_escape_time()) {
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

/// A custom formula is priced by the estimate of the one applied, and before any is applied by a
/// factor above every built-in's — never as Mandelbrot, which a row lookup by name would give it.
#[test]
fn a_custom_formula_prices_by_its_estimate_never_as_mandelbrot() {
    let t = table();
    assert!(!t.formula.iter().any(|f| f.name == FractalKind::Custom.name()), "Custom must not have a row");
    let most = t.formula.iter().map(|f| f.direct.max(f.df32_pert)).fold(1.0f64, f64::max);
    set_custom_factor(f64::NAN);
    assert!(factor(&t, FractalKind::Custom, RenderMode::Direct) > most, "the unset default is above every built-in");
    set_custom_factor(2.25);
    assert_eq!(factor(&t, FractalKind::Custom, RenderMode::Direct), 2.25);
    set_custom_factor(0.4);
    assert!(factor(&t, FractalKind::Custom, RenderMode::Direct) > most, "an estimate below 1 is not trusted");
    set_custom_factor(f64::NAN);
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

/// The start-up probe's measurement can only RAISE a cost (lower a ceiling): a slower term wins, a
/// faster one, a zero or a non-finite one leaves the table's, and the flag says whether anything
/// moved.
#[test]
fn a_measurement_only_ever_lowers_a_ceiling() {
    let table = Costs { df32_pert: 9.9e-8, direct: 1.92e-8 };
    // Intel HD Graphics 4400 (2026-11-10): direct ~5.0e-6 ms/step, ~260× the default.
    let (c, slower) = slower_of(table, Costs { df32_pert: 0.0, direct: 5.0e-6 });
    assert!(slower);
    assert_eq!(c, Costs { df32_pert: 9.9e-8, direct: 5.0e-6 });
    // A faster card changes nothing.
    assert_eq!(slower_of(table, Costs { df32_pert: 2.76e-8, direct: 4.92e-9 }), (table, false));
    // Nothing measured, or nonsense, says nothing.
    assert_eq!(slower_of(table, Costs::default()), (table, false));
    assert_eq!(slower_of(table, Costs { df32_pert: f64::NAN, direct: f64::INFINITY }), (table, false));
    // A table term with no ceiling (0) takes a measured one: a ceiling where there was none.
    let (c, slower) = slower_of(Costs { df32_pert: 0.0, direct: 1.92e-8 }, Costs { df32_pert: 1e-6, direct: 0.0 });
    assert!(slower);
    assert_eq!(c.df32_pert, 1e-6);
}

/// An adapter the table does not list reads as the default and not its own entry; a listed one as
/// its own. The probe gates on this before anything is resolved.
#[test]
fn the_table_entry_says_whether_a_card_was_measured() {
    let listed = table_entry("NVIDIA GeForce RTX 3080").expect("the table parses");
    assert!(listed.own_entry);
    let unlisted = table_entry("Intel(R) HD Graphics 4400").expect("the table parses");
    assert!(!unlisted.own_entry);
    assert!(unlisted.source.starts_with("default"));
    assert!(unlisted.costs.direct > 0.0 && unlisted.costs.df32_pert > 0.0);
}
