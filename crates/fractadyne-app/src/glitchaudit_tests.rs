use super::*;

#[test]
fn a_verdict_names_the_render_the_oracle_agrees_with() {
    let o = Value::Escaped(100.2);
    assert_eq!(verdict(o, true, Value::Escaped(100.2), Value::Escaped(97.0)), Verdict::UncorrectedRight);
    assert_eq!(verdict(o, true, Value::Escaped(97.0), Value::Escaped(100.21)), Verdict::CorrectedRight);
    assert_eq!(verdict(o, true, Value::Interior, Value::Escaped(3.0)), Verdict::Neither);
    assert_eq!(verdict(o, true, Value::Escaped(100.2), Value::Escaped(100.2)), Verdict::BothRight);
    // Interior against interior agrees; interior against escaped does not.
    assert_eq!(verdict(Value::Interior, true, Value::Interior, Value::Escaped(5.0)), Verdict::UncorrectedRight);
}

#[test]
fn an_unstable_oracle_is_never_scored() {
    // Whatever the renders say, a pixel whose true answer changes within the stencil is not evidence.
    assert_eq!(
        verdict(Value::Escaped(1.0), false, Value::Escaped(1.0), Value::Escaped(9.0)),
        Verdict::Sensitive
    );
}

#[test]
fn stability_compares_integer_escape_counts_and_interior() {
    assert!(stable(&[Some((10, 9.1)), Some((10, 9.4)), Some((10, 8.9))]));
    assert!(!stable(&[Some((10, 9.1)), Some((11, 9.4))]));
    assert!(stable(&[None, None, None]));
    assert!(!stable(&[None, Some((10, 9.0))]));
    assert!(!stable(&[]));
}

#[test]
fn correction_never_runs_for_the_audited_families_and_otherwise_follows_the_setting() {
    // Mandelbrot and Multibrot 3–5 in Mandelbrot mode: off whatever the setting says.
    for f in 0..=3 {
        assert!(!correction_applies(true, f, false), "formula {f}");
    }
    // Julia views of the same families, and every other family: the user's setting decides.
    assert!(correction_applies(true, 0, true));
    assert!(!correction_applies(false, 0, true));
    for f in 4..=8 {
        assert!(correction_applies(true, f, false), "formula {f}");
        assert!(!correction_applies(false, f, false), "formula {f}");
    }
    // The ids are the shader's: a formula list reorder would silently move the boundary.
    use crate::fractal::FractalKind;
    assert_eq!(FractalKind::Multibrot5.formula_id(), 3);
    assert_eq!(FractalKind::Tricorn.formula_id(), 4);
}

#[test]
fn spread_takes_evenly_and_never_more_than_asked() {
    let pool: Vec<usize> = (0..100).collect();
    let s = spread(&pool, 4);
    assert_eq!(s, vec![0, 25, 50, 75]);
    assert_eq!(spread(&pool[..3], 10), vec![0, 1, 2]);
    assert!(spread(&[], 5).is_empty());
}
