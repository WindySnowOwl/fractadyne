use super::*;

/// The as-written reference is not the fused form: on enough probe inputs the two differ, or the
/// probe could not tell a fusing compiler from one that is not.
#[test]
fn the_probe_inputs_tell_fusion_apart() {
    let differ = (0..N)
        .filter(|&ix| {
            let s = ix + 100003 * 3;
            let (a, b) = (gt_df(s * 8 + 3, -4, 8), gt_df(s * 8 + 5, -4, 8));
            let p = a.0 * b.0;
            let pe = a.0.mul_add(b.0, -p);
            let fused = a.0.mul_add(b.0, pe + (a.0 * b.1 + a.1 * b.0));
            fused.to_bits() != as_written(ix).to_bits()
        })
        .count();
    assert!(differ >= 16, "only {differ} of {N} inputs separate fused from as-written");
}

/// A device never asked carries no barrier, and a verdict's barrier follows its count.
#[test]
fn verdicts_default_off() {
    assert!(!FusionVerdict { differ: 0, of: N }.barrier());
    assert!(FusionVerdict { differ: 1, of: N }.barrier());
}
