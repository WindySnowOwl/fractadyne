//! Which formulas get a distance estimate is decided twice: by the shader, at each place it writes
//! the distance (`var de = 1.0e30;` then a test on the formula id), and by the host's
//! `FormulaCaps::distance_estimate`, which the panel reads to gray out Distance coloring, relief
//! lighting and the glow (coloring plan step A3). This holds the two together.

use fractadyne_core::formula;

/// The shader's test, as each of its distance writes spells it.
const GATE: &str = "iu.formula <= 3u || iu.formula == 8u || fam_holo(iu.formula)";

#[test]
fn the_shader_writes_a_distance_exactly_where_the_caps_say() {
    let wgsl = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/mandelbrot.wgsl"));
    // Every distance write: the line after `var de = 1.0e30;` is the gate.
    let lines: Vec<&str> = wgsl.lines().map(str::trim).collect();
    let gates: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0] == "var de = 1.0e30;")
        .map(|w| w[1])
        .collect();
    // The direct loop, mode 1 and mode 2. A fourth write, or one with another test, needs a look.
    assert_eq!(gates.len(), 3, "distance writes: {gates:?}");
    for g in &gates {
        assert_eq!(*g, format!("if ({GATE}) {{"), "a distance write tests something else");
    }
    assert!(
        wgsl.contains("fn fam_holo(f: u32) -> bool { return f >= 10u && f <= 12u; }"),
        "fam_holo changed: update GATE's reading below and FormulaCaps::distance_estimate"
    );
    // GATE, read for every id the host can ask about.
    let shader = |id: u32| id <= 3 || id == 8 || (10..=12).contains(&id);
    for id in (0..formula::COUNT).chain([formula::CUSTOM, formula::LIFE, formula::LSYSTEM]) {
        assert_eq!(
            formula::caps(id).distance_estimate,
            shader(id),
            "formula {id}: the panel and the shader disagree about its distance estimate"
        );
    }
    // A custom formula's module replaces these loops and writes no distance at all.
    let custom = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/custom.rs"));
    assert!(custom.contains("let de = 1.0e30;"), "a custom formula now writes a distance: update caps(CUSTOM)");
}

/// `FormulaCaps::orbit_statistics` against the shader: Newton's converged pixel returns no
/// statistics (`AUX_NONE`), which is why the panel does not offer the methods that read them there.
#[test]
fn the_shader_records_the_statistics_exactly_where_the_caps_say() {
    let wgsl = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/mandelbrot.wgsl"));
    let at = wgsl.find("if (newton) {\n            // Color by convergence speed").or_else(|| wgsl.find("if (newton) {\r\n            // Color by convergence speed"));
    let branch = &wgsl[at.expect("the direct path's Newton escape branch")..];
    let branch = &branch[..branch.find("\n        }").expect("its end")];
    assert!(
        branch.contains("return FragOut(vec4<f32>(f32(iter), 0.0, 0.0, 1.0e30), AUX_NONE);"),
        "Newton's converged pixel now returns statistics: update caps(NEWTON).orbit_statistics"
    );
    for id in 0..formula::COUNT {
        assert_eq!(formula::caps(id).orbit_statistics, id != formula::NEWTON, "formula {id}");
    }
    // A custom formula's module keeps the statistics in its own copy of the step.
    let custom = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/custom.rs"));
    assert!(
        custom.contains("aux_step(&aux, zf, cmag, power_f); }}"),
        "a custom module no longer records the statistics: update caps(CUSTOM)"
    );
    assert!(formula::caps(formula::CUSTOM).orbit_statistics);
}
