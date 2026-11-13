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
        // The row's contents, not its address: `METHODS` is a `const`, so `&METHODS[i]` points at a
        // fresh copy, the same address as `spec()`'s only when the optimizer merges them (it does in
        // release, not in a debug test build).
        assert_eq!((m.spec().key, m.spec().wgsl), (METHODS[i].key, METHODS[i].wgsl));
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
    // A statistic's key is for logs only (never stored), so it may share a method's ("trap"); its
    // shader name may not share anything.
    let mut stat_keys = std::collections::HashSet::new();
    for s in Stat::ALL {
        assert!(stat_keys.insert(s.spec().key), "statistic key {} twice", s.spec().key);
        assert!(seen.insert(s.spec().wgsl), "shader name {} twice", s.spec().wgsl);
    }
}

/// Each statistic's row is its own and its bit is its position: one bit each, none shared.
#[test]
fn every_statistic_has_its_own_row_and_bit() {
    assert_eq!(Stat::ALL.len(), STATS.len());
    let mut all = 0u32;
    for (i, s) in Stat::ALL.into_iter().enumerate() {
        assert_eq!((s.spec().key, s.spec().wgsl), (STATS[i].key, STATS[i].wgsl)); // contents: see above
        assert_eq!(s.bit(), 1 << i);
        assert_eq!(all & s.bit(), 0, "{s:?} shares a bit");
        all |= s.bit();
        assert!(s.spec().wgsl.starts_with("STAT_"), "{}", s.spec().wgsl);
    }
}

/// The set's operations, and what it says about the iteration.
#[test]
fn a_set_of_statistics() {
    use Stat::*;
    let st = StatSet::of(Stripe);
    assert!(StatSet::NONE.is_empty() && StatSet::default().is_empty() && !st.is_empty());
    assert!(st.has(Stripe) && !st.has(Trap));
    let both = st.with(Trap);
    assert_eq!(both, StatSet::of(Trap).union(st));
    assert_eq!(both.iter().collect::<Vec<_>>(), [Stripe, Trap]);
    assert_eq!(both.bits(), Stripe.bit() | Trap.bit());
    assert_eq!(both.to_string(), "stripe+trap");
    assert_eq!(StatSet::NONE.to_string(), "none");
    // Every statistic blocks the series; only the angle has no BLA aggregate.
    assert!(!StatSet::NONE.blocks_iter_skip() && StatSet::NONE.bla_aggregate());
    for s in Stat::ALL {
        assert!(StatSet::of(s).blocks_iter_skip(), "{s:?}");
        assert_eq!(StatSet::of(s).bla_aggregate(), s != Angle, "{s:?}");
    }
    let every = Stat::ALL.into_iter().fold(StatSet::NONE, StatSet::with);
    assert!(every.blocks_iter_skip() && !every.bla_aggregate());
    assert!(StatSet::of(Stripe).with(Triangle).with(Trap).bla_aggregate());
}

/// The full G-buffer (B3) is what BLA folds, and adding it to a frame that accumulates anything
/// changes none of that frame's rules: the series stays off and BLA stays as it was. That is what
/// makes the bundle cost only its operations, and the coloring matrix's `gbuffer` check rests on it
/// (the two renders must then agree channel for channel).
#[test]
fn the_gbuffer_changes_no_rule_of_a_frame_that_accumulates() {
    use Stat::*;
    assert_eq!(StatSet::GBUFFER, StatSet::of(Stripe).with(Triangle).with(Trap));
    for bits in 1u32..(1 << Stat::ALL.len()) {
        let s = Stat::ALL.into_iter().filter(|x| bits & x.bit() != 0).fold(StatSet::NONE, StatSet::with);
        let g = s.union(StatSet::GBUFFER);
        assert_eq!(g.blocks_iter_skip(), s.blocks_iter_skip(), "{s}");
        assert_eq!(g.bla_aggregate(), s.bla_aggregate(), "{s}");
    }
    // Every method that reads a statistic reads one in the G-buffer, or the angle.
    for m in ColorMethod::ALL {
        if let Some(r) = m.reads() {
            assert!(StatSet::GBUFFER.has(r) || r == Angle, "{m:?}");
        }
    }
    // For a frame that accumulates nothing it is not free: it turns the series off.
    assert!(StatSet::GBUFFER.blocks_iter_skip() && !StatSet::NONE.blocks_iter_skip());
}

/// Where a frame carries the G-buffer: in floatexp, for the methods that read one of its
/// statistics; everywhere else, and for every other method, what the method reads. A change here
/// changes what every deep stripe, triangle or trap frame computes, so it should be deliberate.
#[test]
fn a_deep_frame_of_a_gbuffer_method_carries_the_gbuffer() {
    use ColorMethod::*;
    for m in ColorMethod::ALL {
        assert_eq!(m.orbit_stats_at(false), m.orbit_stats(), "{m:?} outside floatexp");
        let deep = m.orbit_stats_at(true);
        match m {
            Stripe | TriangleIneq | OrbitTrap => assert_eq!(deep, StatSet::GBUFFER, "{m:?}"),
            _ => assert_eq!(deep, m.orbit_stats(), "{m:?}"),
        }
        // It changes none of the frame's rules (see the test above).
        assert_eq!(deep.blocks_iter_skip(), m.orbit_stats().blocks_iter_skip(), "{m:?}");
        assert_eq!(deep.bla_aggregate(), m.orbit_stats().bla_aggregate(), "{m:?}");
        assert!(deep.has(m.reads().unwrap_or(Stat::Angle)) || m.reads().is_none(), "{m:?}");
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
/// crate before A10), and as the method rows said them before B2 moved the iteration's to
/// [`StatSet`]: through a method's own set they are the same sets. A change here changes what a
/// method computes or which accelerators it keeps, so it should be deliberate.
#[test]
fn the_predicate_sets_are_the_established_ones() {
    use ColorMethod::*;
    let set = |f: fn(ColorMethod) -> bool| -> Vec<ColorMethod> { ColorMethod::ALL.into_iter().filter(|m| f(*m)).collect() };
    // needs_aux
    assert_eq!(set(ColorMethod::reads_aux), [Stripe, TriangleIneq, OrbitTrap, Decomposition]);
    assert_eq!(set(|m| !m.orbit_stats().is_empty()), [Stripe, TriangleIneq, OrbitTrap, Decomposition]);
    // blocks_iter_skip
    assert_eq!(set(|m| m.orbit_stats().blocks_iter_skip()), [Stripe, TriangleIneq, OrbitTrap, Decomposition]);
    assert_eq!(set(ColorMethod::bounded_value), [Stripe, TriangleIneq, Decomposition]);
    assert_eq!(set(ColorMethod::needs_distance_estimate), [Distance]);
    // bla_aggregate (only ever asked of a method that reads the aux lane)
    assert_eq!(set(|m| m.reads_aux() && m.orbit_stats().bla_aggregate()), [Stripe, TriangleIneq, OrbitTrap]);
    // Each reads its own statistic, and accumulates exactly it.
    assert_eq!(Stripe.reads(), Some(Stat::Stripe));
    assert_eq!(TriangleIneq.reads(), Some(Stat::Triangle));
    assert_eq!(OrbitTrap.reads(), Some(Stat::Trap));
    assert_eq!(Decomposition.reads(), Some(Stat::Angle));
    for m in ColorMethod::ALL {
        assert_eq!(m.orbit_stats().iter().collect::<Vec<_>>(), m.reads().into_iter().collect::<Vec<_>>(), "{m:?}");
    }
}

/// A family offers what it computes: Distance only with a distance estimate, the methods that read
/// the orbit's statistics only where the iteration records them (not Newton).
#[test]
fn a_family_offers_what_it_computes() {
    use crate::formula::{caps, BURNING_SHIP, CUSTOM, MANDELBROT, NEWTON, PHOENIX};
    use ColorMethod::*;
    let offered = |f: u32| -> Vec<ColorMethod> { ColorMethod::ALL.into_iter().filter(|m| m.offered_for(caps(f))).collect() };
    assert_eq!(offered(MANDELBROT), ColorMethod::ALL);
    assert_eq!(offered(PHOENIX), ColorMethod::ALL);
    assert_eq!(offered(BURNING_SHIP), [Smooth, Stripe, TriangleIneq, OrbitTrap, Decomposition]);
    assert_eq!(offered(CUSTOM), [Smooth, Stripe, TriangleIneq, OrbitTrap, Decomposition]);
    assert_eq!(offered(NEWTON), [Smooth]);
    // Each with its own reason.
    assert_eq!(Distance.lacks(caps(BURNING_SHIP)), Some(Lacks::DistanceEstimate));
    assert_eq!(Stripe.lacks(caps(NEWTON)), Some(Lacks::OrbitStatistics));
    assert_eq!(Decomposition.lacks(caps(NEWTON)), Some(Lacks::OrbitStatistics));
    assert_eq!(Distance.lacks(caps(NEWTON)), Some(Lacks::DistanceEstimate));
    assert_eq!(Smooth.lacks(caps(NEWTON)), None);
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
    for s in Stat::ALL {
        let line = format!("const {}: u32 = {}u;", s.spec().wgsl, s.bit());
        assert_eq!(ids.matches(&line).count(), 1, "{line}");
    }
    let consts = ids.lines().filter(|l| l.starts_with("const ")).count();
    assert_eq!(consts, ColorMethod::ALL.len() + TrapType::ALL.len() + Stat::ALL.len());
    // Every other line is a comment or blank, so the block can go anywhere at module scope.
    for l in ids.lines().filter(|l| !l.starts_with("const ")) {
        assert!(l.is_empty() || l.starts_with("//"), "{l:?}");
    }
}

/// The capability table as it stands (coloring plan B4): the methods colored from the main lane go
/// everywhere; the statistics keep BLA but lose the series, the walk, glitch correction, the
/// normalized export and strips; decomposition's final angle loses BLA too. A change here changes
/// what a menu or a dialog offers, so it should be deliberate.
#[test]
fn the_capability_table_is_the_established_one() {
    use Capability::*;
    use ColorMethod::*;
    for m in ColorMethod::ALL {
        let can: Vec<Capability> = Capability::ALL.into_iter().filter(|c| m.can(*c)).collect();
        let want: &[Capability] = match m {
            Smooth | Distance => &Capability::ALL,
            Stripe | TriangleIneq | OrbitTrap => &[Bla],
            Decomposition => &[],
        };
        assert_eq!(can, want, "{m:?}");
        // The deep G-buffer changes none of it: the iteration's answers read the same off the
        // deep frame's statistics.
        let deep = m.orbit_stats_at(true);
        assert_eq!(m.can(Series), !deep.blocks_iter_skip(), "{m:?}");
        assert_eq!(m.can(Bla), deep.bla_aggregate(), "{m:?}");
        assert_eq!(m.can(LiveWalk), deep.is_empty(), "{m:?}");
    }
    let keys: Vec<&str> = Capability::ALL.iter().map(|c| c.key()).collect();
    let mut unique = keys.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), keys.len(), "two capabilities share a key");
}

/// The method list's headings (coloring plan B6, design 9.1): the grouping the plan settled, every
/// method under exactly one heading, and no heading without a method (the list would show an empty
/// title).
#[test]
fn every_method_is_under_one_heading_and_no_heading_is_empty() {
    let listed: Vec<(MethodGroup, Vec<&str>)> =
        MethodGroup::ALL.iter().map(|g| (*g, g.methods().map(|m| m.key()).collect())).collect();
    assert_eq!(
        listed,
        [
            (MethodGroup::EscapeCount, vec!["smooth"]),
            (MethodGroup::Distance, vec!["distance"]),
            (MethodGroup::OrbitStatistics, vec!["stripe", "triangle", "trap"]),
            (MethodGroup::Angle, vec!["decomposition"]),
        ]
    );
    let total: usize = listed.iter().map(|(_, ms)| ms.len()).sum();
    assert_eq!(total, ColorMethod::ALL.len(), "a method is under no heading or under two");
}
