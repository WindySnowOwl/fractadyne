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
    let st = LSystemState::default();
    for colouring in Colouring::ALL {
        let key = WalkKey {
            tables: 1,
            order: 8,
            centre: [0.0, 0.0],
            upp: 0.01,
            size: [400, 300],
            colouring,
            margin: 1.0,
        };
        let (segs, stats) = walk_segments(&st.tables, &key, 4.0);
        assert!(stats.segments > 0 && segs.len() as u64 == stats.segments);
        for s in &segs {
            assert!((0.0..=1.0).contains(&s.value), "{colouring:?}: {}", s.value);
        }
    }
}
