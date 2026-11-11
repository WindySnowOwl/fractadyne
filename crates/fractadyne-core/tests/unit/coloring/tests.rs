use super::*;

/// The keys are what files store: they never change, whatever the table's order.
#[test]
fn the_keys_are_the_persisted_ones() {
    let keys: Vec<_> = ColorMethod::ALL.iter().map(|m| m.key()).collect();
    assert_eq!(keys, ["smooth", "stripe", "triangle", "trap", "distance", "decomposition"]);
    let traps: Vec<_> = TrapType::ALL.iter().map(|t| t.key()).collect();
    assert_eq!(traps, ["point", "cross", "circle"]);
}

/// Each variant's row is its own, and the GPU number is the row's position.
#[test]
fn every_variant_has_its_own_row_and_number() {
    assert_eq!(ColorMethod::ALL.len(), METHODS.len());
    for (i, m) in ColorMethod::ALL.into_iter().enumerate() {
        assert_eq!(m.gpu_id() as usize, i);
        assert!(std::ptr::eq(m.spec(), &METHODS[i]));
        assert_eq!(ColorMethod::from_gpu_id(m.gpu_id()), Some(m));
        assert_eq!(ColorMethod::parse_key(m.key()), Some(m));
        assert!(m.spec().wgsl.starts_with("METHOD_"), "{}", m.spec().wgsl);
    }
    assert_eq!(TrapType::ALL.len(), TRAPS.len());
    for (i, t) in TrapType::ALL.into_iter().enumerate() {
        assert_eq!(t.gpu_id() as usize, i);
        assert_eq!(TrapType::from_gpu_id(t.gpu_id()), Some(t));
        assert_eq!(TrapType::parse_key(t.key()), Some(t));
        assert!(t.spec().wgsl.starts_with("TRAP_"), "{}", t.spec().wgsl);
    }
    assert_eq!(ColorMethod::from_gpu_id(ColorMethod::ALL.len() as u32), None);
    assert_eq!(TrapType::from_gpu_id(TrapType::ALL.len() as u32), None);
}

#[test]
fn keys_and_shader_names_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for m in ColorMethod::ALL {
        assert!(seen.insert(m.key()), "key {} twice", m.key());
        assert!(seen.insert(m.spec().wgsl), "shader name {} twice", m.spec().wgsl);
    }
    for t in TrapType::ALL {
        assert!(seen.insert(t.key()), "key {} twice", t.key());
        assert!(seen.insert(t.spec().wgsl), "shader name {} twice", t.spec().wgsl);
    }
}

/// An unknown key is `None` to the strict reader and the default to the lenient one.
#[test]
fn an_unknown_key_is_none_or_the_default() {
    assert_eq!(ColorMethod::parse_key("Stripe"), None);
    assert_eq!(ColorMethod::parse_key(""), None);
    assert_eq!(ColorMethod::from_key("plasma"), ColorMethod::Smooth);
    assert_eq!(TrapType::parse_key("square"), None);
    assert_eq!(TrapType::from_key("square"), TrapType::Point);
}

/// The predicate sets as they were when each lived in its own `matches!` (main.rs and the GPU
/// crate before A10). A change here changes what a method computes or which accelerators it
/// keeps, so it should be deliberate.
#[test]
fn the_predicate_sets_are_the_established_ones() {
    use ColorMethod::*;
    let set = |f: fn(ColorMethod) -> bool| -> Vec<ColorMethod> { ColorMethod::ALL.into_iter().filter(|m| f(*m)).collect() };
    assert_eq!(set(ColorMethod::needs_aux), [Stripe, TriangleIneq, OrbitTrap, Decomposition]);
    assert_eq!(set(ColorMethod::blocks_iter_skip), [Stripe, TriangleIneq, OrbitTrap, Decomposition]);
    assert_eq!(set(ColorMethod::bounded_value), [Stripe, TriangleIneq, Decomposition]);
    assert_eq!(set(ColorMethod::needs_distance_estimate), [Distance]);
    assert_eq!(set(ColorMethod::bla_aggregate), [Stripe, TriangleIneq, OrbitTrap]);
    // A BLA aggregate exists only for a statistic the aux lane accumulates.
    for m in ColorMethod::ALL {
        assert!(!m.bla_aggregate() || m.needs_aux(), "{m:?}");
    }
}

/// A family offers every method but Distance, which only one with a distance estimate offers.
#[test]
fn a_family_offers_distance_only_with_a_distance_estimate() {
    use crate::formula::{caps, BURNING_SHIP, CUSTOM, MANDELBROT, NEWTON, PHOENIX};
    for f in [MANDELBROT, PHOENIX, BURNING_SHIP, NEWTON, CUSTOM] {
        let c = caps(f);
        for m in ColorMethod::ALL {
            let want = m != ColorMethod::Distance || c.distance_estimate;
            assert_eq!(m.offered_for(c), want, "formula {f}, {m:?}");
        }
    }
    assert!(ColorMethod::Distance.offered_for(caps(MANDELBROT)));
    assert!(!ColorMethod::Distance.offered_for(caps(BURNING_SHIP)));
}

/// The shader gets one constant per method and per trap, named as the table says, with the
/// table's number.
#[test]
fn the_shader_constants_are_the_table() {
    let ids = wgsl_ids();
    for m in ColorMethod::ALL {
        let line = format!("const {}: u32 = {}u;", m.spec().wgsl, m.gpu_id());
        assert_eq!(ids.matches(&line).count(), 1, "{line}");
    }
    for t in TrapType::ALL {
        let line = format!("const {}: u32 = {}u;", t.spec().wgsl, t.gpu_id());
        assert_eq!(ids.matches(&line).count(), 1, "{line}");
    }
    let consts = ids.lines().filter(|l| l.starts_with("const ")).count();
    assert_eq!(consts, ColorMethod::ALL.len() + TrapType::ALL.len());
    // Every other line is a comment or blank, so the block can go anywhere at module scope.
    for l in ids.lines().filter(|l| !l.starts_with("const ")) {
        assert!(l.is_empty() || l.starts_with("//"), "{l:?}");
    }
}
