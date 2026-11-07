use super::*;

/// A stored orbit: `transient` distinct samples, then a cycle of `period` distinct samples repeated
/// until it is `len` long.
fn orbit(transient: usize, period: usize, len: usize) -> Vec<[f32; 4]> {
    (0..len)
        .map(|n| {
            let k = if n < transient { n } else { transient + (n - transient) % period };
            [k as f32, 0.5 * k as f32, -(k as f32), 1.0]
        })
        .collect()
}

/// The canonical index the shader compares (`cyc_n`), from the fold.
fn canon(n: u32, fold: [u32; 4]) -> u32 {
    let [end, by, ..] = fold;
    if by == 0 || n < end {
        return n;
    }
    let n0 = end - by;
    n0 + (n - n0) % by
}

/// ⭐The case it exists for: a reference that settles onto a cycle is folded from where the
/// repetition starts, with the smallest period, and every index it folds reads the same sample.
#[test]
fn a_reference_that_settles_onto_a_cycle_is_folded_from_where_it_repeats() {
    let o = orbit(10, 3, 101);
    let fold = ref_fold(&o, 101, 100);
    assert_eq!(fold, [13, 3, 0, 0], "repeats from 10 with period 3");
    for n in 0..101u32 {
        assert_eq!(o[n as usize], o[canon(n, fold) as usize], "index {n}");
    }
}

/// A fixed point is a cycle of one.
#[test]
fn a_fixed_point_folds_by_one() {
    let o = orbit(7, 1, 51);
    assert_eq!(ref_fold(&o, 51, 50), [8, 1, 0, 0]);
}

/// A reference shorter than the iteration count (partial, or escaped) is never folded: its
/// end-of-orbit wrap can come before `max_iter`, and that step reads the index, not the sample.
#[test]
fn a_reference_shorter_than_the_iteration_count_is_not_folded() {
    let o = orbit(10, 3, 101);
    assert_eq!(ref_fold(&o, 101, 101), [0; 4], "one sample short");
    assert_eq!(ref_fold(&o, 101, 5000), [0; 4]);
    // ...and a longer one is.
    assert_eq!(ref_fold(&o, 101, 60), [13, 3, 0, 0]);
}

/// An orbit whose last sample never came before has no fold.
#[test]
fn an_orbit_that_never_repeats_has_no_fold() {
    let o = orbit(200, 1, 200); // every sample distinct
    assert_eq!(ref_fold(&o, 200, 199), [0; 4]);
    assert_eq!(ref_fold(&[], 0, 0), [0; 4]);
}

/// Equality is in all four words, by bits: a sample that matches in its high words only is not a
/// repeat.
#[test]
fn a_repeat_must_match_in_every_word() {
    let mut o = orbit(10, 3, 101);
    // Break the repetition at 40 in a low word only: the fold must start after it.
    o[40][3] = f32::from_bits(1.0f32.to_bits() + 1);
    let fold = ref_fold(&o, 101, 100);
    assert_eq!(fold[1], 3);
    assert!(fold[0] - fold[1] > 40, "the fold starts after the broken sample: {fold:?}");
    for n in 0..101u32 {
        assert_eq!(o[n as usize].map(f32::to_bits), o[canon(n, fold) as usize].map(f32::to_bits), "index {n}");
    }
}

/// The shader keeps `cyc_n` step by step rather than by `%` (mode 0's loop in mandelbrot.wgsl): one
/// on per step, 0 after a rebase, back a period at `fold_end`. This replays that rule, as written
/// there, over a run with rebases, against the closed form.
#[test]
fn the_shaders_step_rule_keeps_the_canonical_index() {
    for fold in [[0, 0, 0, 0], [13, 3, 0, 0], [8, 1, 0, 0]] {
        let [end, by, ..] = fold;
        let mut ref_n = 0u32;
        let mut cyc_n = 0u32;
        for step in 1..500u32 {
            ref_n += 1;
            if step % 97 == 0 {
                ref_n = 0; // a rebase
            }
            cyc_n = if ref_n == 0 { 0 } else { cyc_n + 1 };
            if cyc_n == end {
                cyc_n -= by;
            }
            assert_eq!(cyc_n, canon(ref_n, fold), "fold {fold:?}, step {step}");
        }
    }
}
