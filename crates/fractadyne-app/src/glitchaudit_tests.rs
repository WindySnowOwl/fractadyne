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
fn spread_takes_evenly_and_never_more_than_asked() {
    let pool: Vec<usize> = (0..100).collect();
    let s = spread(&pool, 4);
    assert_eq!(s, vec![0, 25, 50, 75]);
    assert_eq!(spread(&pool[..3], 10), vec![0, 1, 2]);
    assert!(spread(&[], 5).is_empty());
}
