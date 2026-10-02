use super::*;

#[test]
fn the_status_readouts_keep_their_width() {
    let (o1, s1) = status_readouts(7, Some(12));
    let (o2, s2) = status_readouts(4096, Some(999_999_999));
    assert_eq!(o1.chars().count(), o2.chars().count());
    assert_eq!(s1.chars().count(), s2.chars().count());
    let (_, s3) = status_readouts(7, None);
    assert_eq!(s1.chars().count(), s3.chars().count(), "a walk not yet landed keeps the width");
}

#[test]
fn nesting_counts_the_deepest_bracket() {
    let s = LSystem::parse("angle 20\naxiom X\nX = F[+X[-X]]F[-X]+X\n").unwrap();
    assert_eq!(nesting(&s.axiom), 0);
    assert_eq!(nesting(s.rule(b'X').unwrap()), 2);
}

#[test]
fn the_default_system_is_in_the_library() {
    let st = LSystemState::default();
    assert_eq!(st.system.name, DEFAULT_SYSTEM);
    assert!(st.tables.grows());
}

#[test]
fn walk_values_stay_in_the_palette_range() {
    // Every colouring writes a value in [0, 1] (the shader reads < 0 as "no line").
    // In f64 at 100 px a unit, and deep at 2^200 px a unit (the dragon at order ~400, 2^400
    // segments: an index past f64's integers).
    let st = LSystemState::default();
    let zero = || BigFloat::from_f64(0.0, 128);
    for (upp_log2, deep) in [(0.01f64.log2(), false), (-200.0, true)] {
        let order = st.order_for(upp_log2);
        assert_eq!(st.needs_deep(upp_log2, order), deep, "at 2^{upp_log2} px a unit");
        let mut big = None;
        for colouring in Colouring::ALL {
            let key = WalkKey {
                tables: 1,
                order,
                centre: [zero(), zero()],
                upp_log2,
                size: [400, 300],
                colouring,
                margin: 1.0,
                deep,
            };
            let (segs, stats, used) = walk_segments(&st.drawn_system(), &st.tables, big.clone(), &key, 4.0);
            assert_eq!(used.is_some(), deep);
            big = used;
            assert!(stats.segments > 0 && segs.len() as u64 == stats.segments, "{colouring:?} deep {deep}: {stats:?}");
            for s in &segs {
                assert!((0.0..=1.0).contains(&s.value), "{colouring:?}: {}", s.value);
            }
        }
    }
}

#[test]
fn a_centre_difference_divides_at_any_scale() {
    let p = 4096;
    let d = fractadyne_core::bf_sub(&BigFloat::from_f64(1.0, p), &BigFloat::from_f64(1.0 - 2f64.powi(-40), p), p);
    assert_eq!(ratio_f64(&d, -40.0 - 3.0), 8.0);
    // 2^-1100 / 2^-1102: both outside f64, the ratio inside.
    let mut tiny = BigFloat::from_f64(-1.0, p);
    tiny.set_exponent(tiny.exponent().unwrap() - 1100);
    assert_eq!(ratio_f64(&tiny, -1102.0), -4.0);
    assert_eq!(ratio_f64(&BigFloat::from_f64(0.0, p), -5.0), 0.0);
}
