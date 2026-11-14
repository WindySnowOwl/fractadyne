//! The palette anti-aliasing's host side (coloring plan C8): the word the color pass reads, and the
//! log base its distance-sized footprint divides by, which has to be the smooth count's own.

use super::{aa_ln_p, aa_scale, PaletteAa};
use fractadyne_core::formula;

#[test]
fn each_mode_has_its_own_word_and_off_is_the_default() {
    assert_eq!(PaletteAa::default(), PaletteAa::Off);
    assert_eq!(PaletteAa::Off.gpu_word(), 0);
    assert_eq!(PaletteAa::Neighbors.gpu_word(), 1);
    assert_eq!(PaletteAa::Distance.gpu_word(), 2);
    // The shader's two tests: any anti-aliasing at all, then the distance footprint on top.
    let wgsl = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/mandelbrot.wgsl"));
    assert!(wgsl.contains("if (cu.aa_palette != 0u) {"), "the color pass's anti-aliasing test changed");
    assert!(
        wgsl.contains("if (cu.aa_palette == 2u && cu.color_method == METHOD_SMOOTH && c0 < INTERIOR_COORD) {"),
        "the distance footprint's test changed"
    );
}

#[test]
fn the_footprint_divides_by_the_log_of_the_escape_degree_where_there_is_a_distance() {
    let ln = |d: f32| d.ln();
    assert_eq!(aa_ln_p(formula::MANDELBROT), ln(2.0));
    assert_eq!(aa_ln_p(formula::MULTIBROT3), ln(3.0));
    assert_eq!(aa_ln_p(formula::PHOENIX), ln(2.0));
    // No distance estimate: 0, which leaves the neighbors' footprint.
    assert_eq!(aa_ln_p(formula::BURNING_SHIP), 0.0);
    assert_eq!(aa_ln_p(formula::NEWTON), 0.0);
    assert_eq!(aa_ln_p(formula::CUSTOM), 0.0);
    for id in (0..formula::COUNT).chain([formula::CUSTOM, formula::LIFE, formula::LSYSTEM]) {
        let want = if formula::caps(id).distance_estimate { ln(formula::power(id) as f32) } else { 0.0 };
        assert_eq!(aa_ln_p(id), want, "formula {id}");
        if formula::caps(id).distance_estimate {
            assert!(aa_ln_p(id) > 0.5, "formula {id}: a distance but no log base");
        }
    }
}

#[test]
fn a_tap_under_half_a_pixel_is_point_sampled() {
    // Progressive supersampling's sample k takes 1/sqrt(k+1) of a texel's footprint.
    assert_eq!(aa_scale(0), 1.0);
    assert_eq!(aa_scale(3), 0.5);
    assert!((aa_scale(8) - 1.0 / 3.0).abs() < 1e-6);
    // The shader's gate, and what it leaves filtered: ss 1 and 2, and at ss 1 the first four samples.
    let wgsl = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/mandelbrot.wgsl"));
    assert!(wgsl.contains("const AA_MIN_TAP: f32 = 0.499;"), "the smallest filtered tap changed");
    assert!(wgsl.contains("if (min(s, share) < AA_MIN_TAP) {"), "the gate changed");
    let filtered = |ss: u32, k: u32| aa_scale(k) / ss as f32 >= 0.499;
    assert!(filtered(1, 0) && filtered(2, 0) && !filtered(3, 0) && !filtered(4, 0));
    assert_eq!((0..64).filter(|k| filtered(1, *k)).count(), 4);
    assert_eq!((0..64).filter(|k| filtered(2, *k)).count(), 1);
}
