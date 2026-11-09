//! The weight stripe and triangle-inequality coloring blend their averages over N and N−1 terms
//! with (`band_pos` in the shader, coloring plan A5b): the pixel's position inside its iteration
//! band. It must be 1 where |z_N| has just passed the bailout R and 0 where it passed it by a whole
//! power, for every power the shader colors, or the blend does not meet itself at the band edge.
//! `fract(smit)` met that only at power 2 with R = 256, and drew a ring along every band elsewhere.

/// The shader's expression, word for word.
const EXPR: &str = "return clamp(1.0 - log(log(mag2) / log(bail2)) / max(log(power_f), 1.0e-3), 0.0, 1.0);";

/// `band_pos`, in f32 as the GPU runs it.
fn band_pos(mag2: f32, power_f: f32, bail2: f32) -> f32 {
    (1.0 - (mag2.ln() / bail2.ln()).ln() / power_f.ln().max(1.0e-3)).clamp(0.0, 1.0)
}

/// The old weight, `fract(smit)`, with smit = N + 1 − log_p(log2|z|) (N whole, so only its
/// fraction matters).
fn old_weight(mag2: f32, power_f: f32) -> f32 {
    let nu = (mag2.ln() * 0.5 / 2.0f32.ln()).ln() / power_f.ln();
    let smit = 1.0 - nu;
    smit - smit.floor()
}

#[test]
fn the_shader_blends_by_band_pos_everywhere() {
    let wgsl = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/mandelbrot.wgsl"));
    assert!(wgsl.contains(EXPR), "band_pos changed: update EXPR and this test's mirror");
    for (name, src) in [("mandelbrot.wgsl", wgsl), ("custom.rs", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/custom.rs")))] {
        assert!(!src.contains("aux_pack(aux, fract(smit)"), "{name} still blends by fract(smit)");
    }
    // The three escape paths and the custom module each blend by it.
    assert_eq!(wgsl.matches("aux_pack(aux, band_pos(").count(), 3);
}

#[test]
fn the_weight_is_one_and_zero_at_the_band_edges_for_every_power() {
    // The bailouts `bail2_of` gives: 256, and 128 for Multibrot 8 (formula 12).
    for (p, r) in [(2.0f32, 256.0f32), (3.0, 256.0), (4.0, 256.0), (5.0, 256.0), (6.0, 256.0), (7.0, 256.0), (8.0, 128.0)] {
        let bail2 = r * r;
        // |z_N| just past R: the pixel all but escaped one iteration later, weight 1 (all N terms).
        let at_r = band_pos(bail2 * 1.0001, p, bail2);
        // |z_N| = R^p: it all but escaped one iteration sooner, weight 0 (the N−1 terms).
        let at_rp = band_pos(r.powf(2.0 * p) * 0.9999, p, bail2);
        assert!(at_r > 0.999, "p {p}: {at_r} at |z| = R");
        assert!(at_rp < 0.001, "p {p}: {at_rp} at |z| = R^p");
        // Falling across the band, never wrapping inside it.
        let mut last = 1.0f32;
        for k in 1..100 {
            let mag2 = bail2 * (r.powf(2.0 * (p - 1.0))).powf(k as f32 / 100.0);
            let w = band_pos(mag2, p, bail2);
            assert!(w <= last + 1.0e-5, "p {p}: rises at step {k} ({last} → {w})");
            last = w;
        }
        // Past R^p (the +c overshoot) it stays 0 rather than wrapping to 1.
        assert_eq!(band_pos(r.powf(2.0 * p) * 4.0, p, bail2), 0.0);
        // The old weight agreed only where log_p(log2 R) is whole: p = 2 at R = 256.
        let agrees = (old_weight(bail2 * 1.0001, p) - at_r).abs() < 0.01;
        assert_eq!(agrees, p == 2.0, "p {p}: fract(smit) {} vs band_pos {at_r} at the edge", old_weight(bail2 * 1.0001, p));
    }
}
